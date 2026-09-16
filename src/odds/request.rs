// The sampling loop draws nine or more cards a deal, millions of deals over,
// so the generator is a real share of the work. `SmallRng` is Xoshiro256++
// here, which is not cryptographic and does not need to be: nothing is being
// hidden from anyone, and it passes the statistical tests that matter to a
// Monte Carlo. What a seed reproduces is unchanged -- see `run_batch` -- but
// which deals a given seed draws is not promised across versions.
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{
    cards::{Card, CardSet, Rank},
    error::{EquityError, GameError, PokerError},
    hand::Hand,
    notation::{parse_board, parse_dead, parse_hand, parse_hand_up_to, HandSpec},
    sampler::{is_feasible, shape::ShapePlan, SlotSampler},
    variants::{EquityCalculation, PokerType, PokerVariant},
};

use super::{chunk::ChunkResult, default_threads};

/// One equity question: a game, some hands, a board and some dead cards.
///
/// Hands may be partly specified -- `AKs`, `2 c`, `A**` -- and the board may
/// be short. Everything is settled at construction: the slot counts, whether
/// any deal satisfies the request, and the samplers each seat will use. What
/// remains at sampling time is drawing and evaluating.
#[derive(Debug, Clone)]
pub struct EquityRequest<V: PokerVariant + EquityCalculation> {
    variant: V,
    /// One prepared sampler per alternative, per seat.
    seats: Vec<Vec<SlotSampler>>,
    /// What to deal in what order, tightest first. `seats.len()` stands for
    /// the board.
    ///
    /// Which goes first cannot change the answer -- each draws from its own
    /// list of holdings regardless of what is left, and a draw that clashes
    /// throws the whole deal away -- but it changes how often a deal has to
    /// be thrown away at all. Something that will take any card will happily
    /// take the one card a fussier hand was waiting for, and a named board
    /// card is the fussiest thing at the table.
    order: Vec<usize>,
    /// The board's slots, padded out with wildcards to the game's full board.
    board: SlotSampler,
    board_slots: usize,
    available: CardSet,
    /// Per participant, in `order`'s numbering, the cards it holds in every
    /// deal that satisfies the request. `seats.len()` stands for the board.
    certain: Vec<CardSet>,
    /// Every card spoken for by somebody, which is the union of the above.
    spoken_for: CardSet,
    /// The raw slots behind `seats`, plus the board's last, in `order`'s
    /// numbering. The samplers above were built against the opening deck;
    /// weighted dealing needs the slots themselves so it can count what a
    /// participant may take from a deck that has already been dealt from.
    slots: Vec<Vec<Vec<CardSet>>>,
    /// `ln` of how many holdings each alternative had against the opening
    /// deck, in the same numbering. The weight of a deal is measured against
    /// this, which keeps every weight in `0..=1` and, more importantly, keeps
    /// alternatives weighted against each other exactly as they are today.
    opening: Vec<Vec<f64>>,
    /// One shape plan per alternative, built once. Which groups the slots cut
    /// the deck into never changes as the deal goes on, so the plan is built
    /// here and only re-weighed against the live deck, which is what makes a
    /// weighted deal affordable at all.
    plans: Vec<Vec<Option<ShapePlan>>>,
    dealing: Dealing,
    players: usize,
    threads: usize,
}

/// Buffers a weighted deal reuses, so the sampling loop allocates nothing.
#[derive(Debug, Default)]
pub(crate) struct WeighingScratch {
    groups: Vec<CardSet>,
    cumulative: Vec<u128>,
}

/// How a request's deals are drawn.
///
/// The choice is made once, when the request is built, and never revisited:
/// chunks merge by addition and two chunks drawn different ways do not merge
/// at all, so a request that changed its mind halfway would silently corrupt
/// the sums it was pouring into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dealing {
    /// Draw every participant against the opening deck and throw the whole
    /// deal away when they clash. Uniform by rejection, and what almost every
    /// request uses.
    Rejecting,
    /// Draw each participant from what is actually left, and weight the deal
    /// to undo the bias that introduces. Reserved for the spots where
    /// rejection has all but stopped working.
    Weighted,
}

/// The acceptance below which weighted dealing may be considered at all.
///
/// A hard floor rather than a comparison, and deliberately so. Weighting is
/// newer and subtler than rejection, so it is kept away from every spot that
/// already works well: a bug in the weighted path can then never reach a
/// hand anybody actually holds.
///
/// Where the floor sits is measured rather than chosen -- see
/// `measure_both_paths` in the tests, which times the two paths against each
/// other spot by spot. Below a twentieth, weighting wins by between two and
/// eleven times. Above it the two are within a fifth of each other or
/// rejection is ahead outright, and a fifth is not worth moving a spot that
/// already works onto a newer path.
const WEIGHTING_FLOOR: f64 = 0.05;

/// How much better weighted dealing must be before it is worth the change.
const WEIGHTING_MARGIN: f64 = 1.5;

/// How many draws rejection is given to show what it can do.
///
/// Enough to see a rate at the floor: a spot keeping one deal in twenty
/// produces [`SETTLED_DRAWS`] usable ones in this many attempts.
const CALIBRATION_DRAWS: u32 = 800;

/// How many usable deals settle it, so the calibration can stop early.
///
/// This many successes inside [`CALIBRATION_DRAWS`] *is* the floor rate, so
/// reaching it means rejection is already keeping more than a twentieth and
/// nothing further need be measured. It is what keeps the ordinary case
/// cheap: a spot that keeps every deal is decided in forty draws rather than
/// eight hundred, and building a request stays the microsecond affair it was
/// before any of this.
const SETTLED_DRAWS: u32 = (CALIBRATION_DRAWS as f64 * WEIGHTING_FLOOR) as u32;

/// How many draws weighting is given to show what it yields.
///
/// Fewer, because a weighted draw costs more than a rejected one and the
/// question it answers is coarse: whether the yield clears acceptance by half
/// again. That does not need a precise figure.
const WEIGHTED_CALIBRATION_DRAWS: u32 = 400;

/// The cards a participant holds in *every* deal the request admits.
///
/// A slot with one candidate takes that card whatever else happens. Where a
/// seat has alternatives it has to be certain under all of them -- `AKs`
/// names no card for sure, since the suit is still open, while `Ah2c3d`
/// names three.
///
/// This is what makes it safe to keep those cards away from everybody else:
/// if a seat holds a card in every valid deal, no other seat holds it in any
/// of them, so removing it from their pools removes no deal that counts.
fn certain_cards(alternatives: &[Vec<CardSet>]) -> CardSet {
    alternatives
        .iter()
        .map(|slots| {
            slots
                .iter()
                .filter(|slot| slot.len() == 1)
                .fold(CardSet::EMPTY, |all, slot| all.union(*slot))
        })
        .reduce(|all, alternative| all.intersection(alternative))
        .unwrap_or(CardSet::EMPTY)
}

/// Whether a slot is a whole rank with the suit left open -- `A`, and not
/// `Ah` or `c` or `*`.
fn names_only_a_rank(slot: CardSet) -> bool {
    Rank::all()
        .iter()
        .any(|rank| CardSet::of_rank(*rank) == slot)
}

/// Gives every open-suit rank slot one particular card of that rank.
///
/// Only sound where suits cannot affect scoring -- see
/// [`PokerVariant::suits_matter`] -- and there it costs nothing and saves a
/// great deal. `A23` in razz means an ace, a deuce and a three of no
/// particular suit, and sampling it means drawing three cards that are free
/// to be any of four apiece, then correcting for the hand that draws a second
/// ace among its open cards. Pinning them to `Ah 2c 3d` instead leaves the
/// deck holding exactly the same ranks in exactly the same numbers, so razz
/// cannot tell the difference -- and the slots become single cards, which the
/// sampler settles before it draws anything.
///
/// Returns `None` when the request should be left alone: a seat offering
/// several holdings, where the pinning would have to be consistent across
/// alternatives that are meant to be exclusive; or a rank with no card left
/// to give, which is a request the feasibility check should report in its own
/// words rather than one this should quietly mangle.
fn pin_open_suits(
    hands: &[HandSpec],
    board: &[CardSet],
    available: CardSet,
) -> Option<(Vec<HandSpec>, Vec<CardSet>)> {
    if hands.iter().any(|spec| spec.alternatives.len() != 1) {
        return None;
    }

    // A card named outright is already spoken for and cannot be handed to a
    // rank slot as well.
    let mut taken = CardSet::EMPTY;
    for slot in hands
        .iter()
        .flat_map(|spec| spec.alternatives[0].iter())
        .chain(board.iter())
    {
        if slot.len() == 1 {
            taken = taken.union(*slot);
        }
    }

    let pin = |slot: &mut CardSet, taken: &mut CardSet| -> Option<()> {
        if !names_only_a_rank(*slot) {
            return Some(());
        }
        let card = slot.intersection(available).without(*taken).iter().next()?;
        taken.insert(card);
        *slot = CardSet::from_cards(&[card]);
        Some(())
    };

    let mut pinned: Vec<HandSpec> = Vec::with_capacity(hands.len());
    for spec in hands {
        let mut slots = spec.alternatives[0].clone();
        for slot in slots.iter_mut() {
            pin(slot, &mut taken)?;
        }
        pinned.push(HandSpec::from_alternatives(vec![slots]));
    }

    let mut shared = board.to_vec();
    for slot in shared.iter_mut() {
        pin(slot, &mut taken)?;
    }

    Some((pinned, shared))
}

/// How many cards a stud hand holds on the street it starts from.
///
/// Two down and one up, and the game has no earlier street than that, so a
/// shorter field is a miscount rather than a hand caught mid-deal.
const THIRD_STREET: usize = 3;

/// Whether this deal runs the deck out, so that the last card is shared.
///
/// Seven-card stud gives every player seven cards, which is more than a deck
/// holds once eight sit down: eight sevens is fifty-six. The rule is that the
/// last card is not dealt to each player at all. One card goes face up in the
/// middle and every player counts it as their seventh, which brings the deal
/// back to `8 x 6 + 1 = 49`.
///
/// Rare enough that most players never see it, and a rule all the same. Razz
/// and the split-pot stud games share it, since they share the dealing.
///
/// Burn cards are not counted here, because the library does not model them:
/// nothing is hidden from anyone in an equity calculation, so a burnt card is
/// only a card that was never named.
fn last_card_is_shared<V: PokerVariant>(variant: V, players: usize) -> bool {
    matches!(variant.poker_type(), PokerType::Stud)
        && players * variant.hole_cards() > variant.deck().len() as usize
}

impl<V: PokerVariant + EquityCalculation> EquityRequest<V> {
    /// Builds a request from masks, with no text involved.
    ///
    /// This is the entry point for a caller that already holds masks; the text
    /// form in [`from_text`](Self::from_text) parses into exactly this.
    pub fn from_masks(
        variant: V,
        hands: &[HandSpec],
        board: &[CardSet],
        dead: CardSet,
    ) -> Result<Self, PokerError> {
        if hands.len() < 2 {
            return Err(EquityError::NoPlayers.into());
        }

        // Where a player's cards arrive over time -- stud dealt street by
        // street, a draw game where cards are exchanged -- a field names what
        // the player holds now, and whatever is missing is still to come. A
        // five-card draw hand stands pat; a three-card one draws two. Cards
        // thrown away are named as dead, which is what keeps them out of the
        // deck without pretending they were never seen.
        //
        // Community games deal every hole card at once, so a short field
        // there is a miscount rather than a hand in progress. Name an unknown
        // card with a wildcard instead.
        // Eight-handed stud runs the deck out, so the seventh card is shared
        // rather than dealt: six cards a seat and one in the middle.
        let shared_last = last_card_is_shared(variant, hands.len());
        let hole = variant.hole_cards() - usize::from(shared_last);
        let shared = variant.board_cards() + usize::from(shared_last);

        // More seats than the deck can cover is a miscount, and almost always
        // a loop that ran one turn long. It is checked before anything is
        // asked about the fields, because the seat count is knowable without
        // reading them and it is the more useful thing to be told: a caller
        // who passed twenty-four hold'em hands wants to hear about the
        // twenty-four, not that their cards do not fit.
        let room = (variant.deck().len() as usize).saturating_sub(shared) / hole.max(1);
        if hands.len() > room {
            return Err(EquityError::TooManyPlayers {
                asked: hands.len(),
                room,
            }
            .into());
        }

        let cards_arrive_over_time =
            matches!(variant.poker_type(), PokerType::Draw | PokerType::Stud);

        // In stud every live player is on the same street: third street is
        // three cards for everyone at the table, fourth is four. So fields of
        // different lengths are not a table caught mid-deal, they are a
        // miscount. Draw games are the opposite -- a short field is how a
        // player says how many they are drawing, so five against four is a
        // pat hand against a one-card draw and entirely legal.
        if matches!(variant.poker_type(), PokerType::Stud) {
            let mut counts = hands.iter().map(|spec| spec.slot_count());
            let first = counts.next().flatten();
            if counts.any(|count| count != first) {
                return Err(EquityError::UnequalHandSizes.into());
            }
            // And third street is where a stud hand starts. One or two cards
            // is not an earlier street, it is a hand that was never dealt.
            if let Some(count) = first.filter(|count| *count < THIRD_STREET) {
                return Err(EquityError::NotEnoughCards(count).into());
            }
        }

        let hands: Vec<HandSpec> = hands
            .iter()
            .map(|spec| {
                let count = spec.slot_count().ok_or(EquityError::UnequalHandSizes)?;
                if count == hole {
                    return Ok(spec.clone());
                }
                if count > hole || !cards_arrive_over_time {
                    return Err(EquityError::NotEnoughCards(count));
                }
                Ok(HandSpec::from_alternatives(
                    spec.alternatives
                        .iter()
                        .map(|alternative| {
                            let mut slots = alternative.clone();
                            slots.resize(hole, CardSet::FULL_DECK);
                            slots
                        })
                        .collect(),
                ))
            })
            .collect::<Result<Vec<_>, EquityError>>()?;
        let hands = &hands[..];

        for spec in hands {
            if !spec.is_satisfiable() {
                return Err(EquityError::Infeasible("a hand".to_string()).into());
            }
        }

        let board_slots = shared;
        if board.len() > board_slots {
            return Err(EquityError::InvalidCommunityCards(board.len()).into());
        }
        // Courchevel's first board card is face up before the betting, so a
        // Courchevel question with nothing showing is not one.
        //
        // A wildcard counts, and deliberately: `*` is a card that has been
        // dealt and not yet seen, which is both a fair question and the way
        // to ask what the turned card was worth -- run the spot with the card
        // named, run it again with `*`, and the gap is the answer. It comes
        // back as the five-card Omaha number, because that is what Courchevel
        // with an unknown first card *is*. The error says as much, since the
        // difference between `""` and `"*"` is otherwise invisible.
        if board.len() < variant.least_board_cards() {
            return Err(EquityError::NotEnoughBoardCards {
                least: variant.least_board_cards(),
                found: board.len(),
            }
            .into());
        }

        // A dead card this game never dealt is a mistake about the game, and
        // it has to be caught here or not at all: taking a card out of a deck
        // that never held it is a no-op, so nothing downstream would notice.
        // The hand and board fields already refuse a deuce in short deck, and
        // the dead field should not be the one place it is welcome.
        if let Some(card) = dead.without(variant.deck()).iter().next() {
            return Err(GameError::NotInDeck(card).into());
        }

        // Anything in this game's deck and not dead is on offer. Known cards
        // are not removed here: they are slots with exactly one candidate, so
        // the matching handles them and the sampler cannot draw them twice.
        let available = variant.deck().without(dead);

        // Padding the board with wildcards makes a short board the same shape
        // as a full one, which is how a flop and a river spot share a path.
        let mut board_masks = board.to_vec();
        board_masks.resize(board_slots, CardSet::FULL_DECK);

        // Where suits cannot change the answer, a rank written without one is
        // given a suit here and stops being a choice at all. Razz is the only
        // game this fires for, and it is the game that needed it most: `A23`
        // against `A24` went from keeping a quarter of its deals to keeping
        // all of them, and four-handed razz from refusing the question to
        // answering it.
        let pinned;
        let hands = if variant.suits_matter() {
            hands
        } else if let Some((seats, shared)) = pin_open_suits(hands, &board_masks, available) {
            pinned = seats;
            board_masks = shared;
            &pinned[..]
        } else {
            hands
        };

        // Which cards are already spoken for. This does two jobs: it names a
        // card that two participants both claim, which is a mistake worth a
        // better message than "no deal is possible"; and at sampling time it
        // keeps a seat filling a free slot from taking a card another seat
        // has named, without which a seven-handed stud game cannot be dealt
        // at all.
        let mut certain: Vec<CardSet> = hands
            .iter()
            .map(|spec| certain_cards(&spec.alternatives))
            .collect();
        certain.push(certain_cards(std::slice::from_ref(&board_masks)));

        let mut spoken_for = CardSet::EMPTY;
        for claimed in &certain {
            let clash = spoken_for.intersection(*claimed);
            if let Some(card) = clash.iter().next() {
                return Err(GameError::DuplicateCard(card).into());
            }
            spoken_for = spoken_for.union(*claimed);
        }

        // A named card that is not on offer is one of two mistakes, and they
        // want different words: a card this game's deck never held, or one
        // the caller has already declared dead.
        if let Some(card) = spoken_for.without(available).iter().next() {
            return Err(if variant.deck().contains(card) {
                GameError::DuplicateCard(card)
            } else {
                GameError::NotInDeck(card)
            }
            .into());
        }

        Self::check_feasible(hands, &board_masks, available)?;

        let seats: Vec<Vec<SlotSampler>> = hands
            .iter()
            .map(|spec| {
                spec.alternatives
                    .iter()
                    .map(|slots| SlotSampler::new(slots, available))
                    .collect()
            })
            .collect();

        // How much choice each hand has: the fewest cards any of its holdings
        // could be built from. A hand naming its cards has almost none and
        // goes first; one that will take anything goes last. The board is in
        // the same reckoning, since a named flop is as fixed as a named hand
        // and a runout is as free as a wildcard.
        let freedom = |slots: &[CardSet]| -> u64 {
            slots
                .iter()
                .map(|slot| slot.intersection(available).len() as u64)
                .sum()
        };
        let mut order: Vec<usize> = (0..=hands.len()).collect();
        order.sort_by_key(|&which| {
            if which == hands.len() {
                freedom(&board_masks)
            } else {
                hands[which]
                    .alternatives
                    .iter()
                    .map(|slots| freedom(slots))
                    .min()
                    .unwrap_or(u64::MAX)
            }
        });

        // The slots themselves, kept alongside the samplers so a weighted
        // deal can count what is still takeable from a part-dealt deck. The
        // board goes last, in `order`'s numbering.
        let mut slot_lists: Vec<Vec<Vec<CardSet>>> =
            hands.iter().map(|spec| spec.alternatives.clone()).collect();
        slot_lists.push(vec![board_masks.clone()]);

        // One plan per alternative, built here and only re-weighed later.
        let plans: Vec<Vec<Option<ShapePlan>>> = slot_lists
            .iter()
            .enumerate()
            .map(|(which, alternatives)| {
                let pool = available
                    .without(spoken_for)
                    .union(certain[which].intersection(available));
                alternatives
                    .iter()
                    .map(|slots| {
                        // A participant with no slots -- the board of a game
                        // that deals none -- has nothing to plan.
                        if slots.is_empty() {
                            None
                        } else {
                            ShapePlan::build(slots, pool)
                        }
                    })
                    .collect()
            })
            .collect();

        // How many holdings each alternative had before a card was dealt.
        // Every weight is measured against this, which does two things: it
        // keeps weights in `0..=1`, where they cannot overflow; and it holds
        // the alternatives in the same proportion to each other that
        // rejection puts them in. Weighting by the live count alone would
        // quietly re-weight a range from per-alternative to per-combination,
        // which is a different answer rather than a faster one.
        //
        // A participant with no slots has exactly one way to take nothing, so
        // it contributes nothing. A plan that could not be built, or one
        // admitting no holding at all, leaves the weight undefined; both come
        // back as something that is not finite, which is what keeps weighting
        // away from this request altogether.
        let opening: Vec<Vec<f64>> = plans
            .iter()
            .zip(slot_lists.iter())
            .map(|(alternatives, slot_lists)| {
                alternatives
                    .iter()
                    .zip(slot_lists.iter())
                    .map(|(plan, slots)| match plan {
                        None if slots.is_empty() => 0.0,
                        None => f64::NAN,
                        Some(plan) => (plan.total() as f64).ln(),
                    })
                    .collect()
            })
            .collect();

        let mut request = Self {
            variant,
            seats,
            order,
            board: SlotSampler::new(&board_masks, available),
            board_slots,
            available,
            certain,
            spoken_for,
            slots: slot_lists,
            opening,
            plans,
            dealing: Dealing::Rejecting,
            players: hands.len(),
            threads: default_threads(),
        };
        request.dealing = request.choose_dealing();
        Ok(request)
    }

    /// Forces weighted dealing on, whatever the calibration decided.
    ///
    /// For tests that check the weighted path itself. The policy that picks
    /// between the two is worth testing separately from the arithmetic it
    /// picks, and a test of the arithmetic should not break when the policy
    /// is retuned.
    #[cfg(test)]
    pub(crate) fn force_weighted(&mut self) -> bool {
        if self.opening.iter().flatten().any(|ln| !ln.is_finite()) {
            return false;
        }
        self.dealing = Dealing::Weighted;
        true
    }

    /// Forces rejection dealing on, whatever the calibration decided.
    ///
    /// For tests of the rejecting path in spots that would now be weighted.
    #[cfg(test)]
    pub(crate) fn force_rejecting(&mut self) {
        self.dealing = Dealing::Rejecting;
    }

    /// Whether deals are drawn against the live deck and weighted, rather
    /// than drawn against the opening deck and rejected when they clash.
    ///
    /// Almost always false. It turns true only for a request whose seats
    /// compete for the same cards so hard that rejection has nearly stopped
    /// working, and it is worth showing a caller alongside
    /// [`ChunkResult::effective_samples`], which is what such a run costs.
    pub fn is_weighted(&self) -> bool {
        self.dealing == Dealing::Weighted
    }

    /// The pool a participant draws from: everything still available, less
    /// the cards other participants have named, plus its own.
    ///
    /// Keeping another seat's named cards out of this pool removes no deal
    /// that counts -- a card one seat holds in every valid deal is held by no
    /// other seat in any of them -- so it costs nothing and saves a great
    /// deal of rejection.
    fn pool_for(&self, which: usize, available: CardSet) -> CardSet {
        available
            .without(self.spoken_for)
            .union(self.certain[which].intersection(available))
    }

    /// The slots one participant is drawing with this deal, and where they go.
    fn alternative_for(&self, which: usize, rng: &mut SmallRng) -> usize {
        let count = self.slots[which].len();
        if count <= 1 {
            0
        } else {
            rng.gen_range(0..count)
        }
    }

    /// Deals every participant from what is left, reporting what the deal is
    /// worth.
    ///
    /// `None` is a rejection, exactly as a `false` from [`deal`](Self::deal)
    /// is: a participant found nothing it could still take. That happens far
    /// less often here, but it does happen, and the deal is thrown away
    /// whole. Nothing about the discard may depend on the weight -- it is the
    /// weight's independence from the discard that lets the completion rate
    /// cancel and keeps the answer honest.
    ///
    /// The weight is how many holdings each participant had to choose from,
    /// relative to what it had before any card was dealt. A participant that
    /// still had all its choices contributes one; one that was squeezed by
    /// the seats ahead of it contributes less, and the deal counts for less
    /// in proportion. That is exactly the bias drawing from a live deck
    /// introduces, undone.
    fn deal_weighted(
        &self,
        rng: &mut SmallRng,
        holes: &mut [Vec<Card>],
        board: &mut Vec<Card>,
        scratch: &mut WeighingScratch,
    ) -> Option<f64> {
        let mut available = self.available;
        board.clear();
        let mut ln_weight: f64 = 0.0;

        for &which in &self.order {
            let is_board = which == self.seats.len();
            if is_board && self.board_slots == 0 {
                continue;
            }

            let alternative = self.alternative_for(which, rng);
            let plan = self.plans[which][alternative].as_ref()?;
            let pool = self.pool_for(which, available);

            // Only the group sizes have changed, so the plan is re-weighed
            // rather than rebuilt. Rebuilding here is what made a weighted
            // deal cost hundreds of microseconds instead of hundreds of
            // nanoseconds.
            let total = plan.weigh(pool, &mut scratch.groups, &mut scratch.cumulative);
            if total == 0 {
                return None;
            }
            ln_weight += (total as f64).ln() - self.opening[which][alternative];

            let out = if is_board {
                &mut *board
            } else {
                &mut holes[which]
            };
            out.clear();
            plan.draw_weighed(&scratch.groups, &scratch.cumulative, total, rng, out);
            for &card in out.iter() {
                available.remove(card);
            }
        }

        Some(ln_weight.exp())
    }

    /// Decides how this request will be dealt, once and for all.
    ///
    /// Rejection is the default and keeps every spot it already handles. Two
    /// things have to be true before weighting takes over: acceptance must
    /// have fallen through the floor, and weighting must actually be better
    /// by a clear margin. Both are measured rather than guessed, from a fixed
    /// seed, so a given request always decides the same way.
    fn choose_dealing(&self) -> Dealing {
        // A plan that could not be built is a slot list too finely cut to
        // count, and weighting has no way to weigh it.
        if self.opening.iter().flatten().any(|ln| !ln.is_finite()) {
            return Dealing::Rejecting;
        }

        let mut rng = SmallRng::seed_from_u64(0x5EED_CA11);
        let mut holes: Vec<Vec<Card>> = vec![Vec::new(); self.players];
        let mut board: Vec<Card> = Vec::new();
        let mut scratch = WeighingScratch::default();

        // Stopped as soon as rejection has proved itself, which is almost
        // always long before the budget runs out.
        let mut kept = 0u32;
        for _ in 0..CALIBRATION_DRAWS {
            if self.deal(&mut rng, &mut holes, &mut board) {
                kept += 1;
                if kept >= SETTLED_DRAWS {
                    return Dealing::Rejecting;
                }
            }
        }
        let acceptance = kept as f64 / CALIBRATION_DRAWS as f64;
        if acceptance >= WEIGHTING_FLOOR {
            return Dealing::Rejecting;
        }

        // What weighting would yield: how often a deal completes, times how
        // much a completed deal is really worth. A pile of weighted deals in
        // which a few carry most of the total says less than its count
        // suggests, and that has to be paid for here, not discovered later.
        let mut weight_sum = 0.0;
        let mut weight_square_sum = 0.0;
        let mut completed = 0u32;
        for _ in 0..WEIGHTED_CALIBRATION_DRAWS {
            let Some(weight) = self.deal_weighted(&mut rng, &mut holes, &mut board, &mut scratch)
            else {
                continue;
            };
            completed += 1;
            weight_sum += weight;
            weight_square_sum += weight * weight;
        }
        if completed == 0 || weight_square_sum <= 0.0 {
            return Dealing::Rejecting;
        }

        let effective = weight_sum * weight_sum / weight_square_sum;
        let yielded = effective / WEIGHTED_CALIBRATION_DRAWS as f64;

        if yielded > acceptance * WEIGHTING_MARGIN {
            Dealing::Weighted
        } else {
            Dealing::Rejecting
        }
    }

    /// Builds a request from the library's notation.
    ///
    /// Each entry of `hands` is one player's field, `board` may be short and
    /// may hold wildcards, and `dead` must name exact cards.
    pub fn from_text(
        variant: V,
        hands: &[&str],
        board: &str,
        dead: &str,
    ) -> Result<Self, PokerError> {
        // Stud and draw fields may name fewer cards than the game deals; a
        // community field may not. See `from_masks`.
        let read = if matches!(variant.poker_type(), PokerType::Draw | PokerType::Stud) {
            parse_hand_up_to
        } else {
            parse_hand
        };
        let shared_last = last_card_is_shared(variant, hands.len());
        let hole = variant.hole_cards() - usize::from(shared_last);
        let specs = hands
            .iter()
            .map(|text| read(text, hole))
            .collect::<Result<Vec<_>, _>>()?;
        let board = parse_board(board, variant.board_cards() + usize::from(shared_last))?;
        Self::from_masks(variant, &specs, &board, parse_dead(dead)?)
    }

    /// Proves the request impossible, or admits it might be possible.
    ///
    /// The check is one-sided where a seat has several alternatives: each
    /// slot is relaxed to everything any alternative would accept, so a
    /// relaxed graph with no perfect matching means no real combination has
    /// one either. It never rejects a request some combination would satisfy.
    fn check_feasible(
        hands: &[HandSpec],
        board: &[CardSet],
        available: CardSet,
    ) -> Result<(), PokerError> {
        let mut slots: Vec<CardSet> = Vec::new();
        for spec in hands {
            let width = spec.slot_count().unwrap_or(0);
            for position in 0..width {
                slots.push(
                    spec.alternatives
                        .iter()
                        .fold(CardSet::EMPTY, |all, alt| all.union(alt[position])),
                );
            }
        }
        slots.extend_from_slice(board);

        if !is_feasible(&slots, available) {
            return Err(EquityError::Infeasible(
                "the hands, board and dead cards together".to_string(),
            )
            .into());
        }
        Ok(())
    }

    /// How many seats are in the hand.
    pub fn players(&self) -> usize {
        self.players
    }

    /// The variant being played.
    pub fn variant(&self) -> V {
        self.variant
    }

    /// How many cards each seat holds privately in this request.
    ///
    /// The game's own figure, except in an eight-handed stud game where the
    /// deck runs out and the last card is shared instead of dealt -- there it
    /// is one fewer, and [`board_cards`](Self::board_cards) is the one that
    /// went to the middle.
    pub fn hole_cards(&self) -> usize {
        self.variant.hole_cards() - usize::from(self.board_slots > self.variant.board_cards())
    }

    /// How many cards this request shares between the seats.
    ///
    /// Five for a community game, zero for stud and draw -- except for the
    /// eight-handed stud game that runs the deck out, where it is one.
    pub fn board_cards(&self) -> usize {
        self.board_slots
    }

    /// How many threads a run of this request spreads over.
    ///
    /// Starts at [`default_threads`](crate::odds::default_threads), which is
    /// polite rather than greedy -- several calculators may be open at once
    /// and one must not starve the others. Raise it with
    /// [`with_threads`](Self::with_threads) when the machine is yours.
    pub fn threads(&self) -> usize {
        self.threads
    }

    /// The same request, run over `threads` threads.
    ///
    /// ```no_run
    /// # use poker_equity::{odds::{equity, EquityRequest, Target}, variants::Holdem};
    /// let request = EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], "", "")?
    ///     .with_threads(std::thread::available_parallelism()?.get());
    /// let result = equity(&request, Target::Samples(5_000_000))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// Held between one thread and as many as the machine reports, so
    /// `with_threads(usize::MAX)` reads as "all of it" and no count can ask
    /// for threads that are not there. More threads than cores buys nothing
    /// here: the loop is pure arithmetic and never waits on anything, so a
    /// thread with no core to run on only adds a context switch.
    ///
    /// Note that the deals a seed draws depend on how many threads divide
    /// them, so reproducing a run means matching this too.
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.set_threads(threads);
        self
    }

    /// Sets the thread count in place, for a request already built.
    pub fn set_threads(&mut self, threads: usize) {
        let most = std::thread::available_parallelism().map_or(1, |cores| cores.get());
        self.threads = threads.clamp(1, most);
    }

    /// Deals once, writing each seat's cards into `holes` and the shared
    /// cards into `board`. Returns false when the draw failed and should be
    /// retried.
    fn deal(&self, rng: &mut SmallRng, holes: &mut [Vec<Card>], board: &mut Vec<Card>) -> bool {
        let mut available = self.available;
        board.clear();

        // Tightest first, so that whatever will take any card does not take
        // the one card something fussier was waiting for.
        for &which in &self.order {
            let (sampler, out) = if which == self.seats.len() {
                if self.board_slots == 0 {
                    continue;
                }
                (&self.board, &mut *board)
            } else {
                let alternatives = &self.seats[which];
                (
                    &alternatives[rng.gen_range(0..alternatives.len())],
                    &mut holes[which],
                )
            };

            // Everybody else's named cards are off limits, so a free slot
            // cannot take one and spoil the deal for the seat that named it.
            let pool = available
                .without(self.spoken_for)
                .union(self.certain[which].intersection(available));

            if !sampler.draw(pool, rng, out) {
                return false;
            }
            for &card in out.iter() {
                available.remove(card);
            }
        }

        true
    }
}

/// The most deals exact mode will walk before declining.
///
/// A river spot, a heads-up turn and most all-in confrontations sit far below
/// this; a preflop field of wide ranges does not.
pub const EXACT_DEAL_LIMIT: usize = 50_000_000;

/// Enumerates every deal the request admits, rather than sampling them.
///
/// An exact result needs no error bar, and reporting a confidence interval on
/// a deterministic answer is a bug -- so [`ChunkResult::exact`] is set and the
/// standard error comes back as zero.
///
/// Returns `None` when the space is too large to walk, which is the caller's
/// signal to sample instead.
pub fn run_exact<V>(request: &EquityRequest<V>) -> Result<Option<ChunkResult>, PokerError>
where
    V: PokerVariant + EquityCalculation,
{
    run_exact_within(request, EXACT_DEAL_LIMIT)
}

/// Enumerates every deal, so long as there are no more than `limit` of them.
///
/// The limit is how much work the caller is willing to do. Asking for half a
/// million sampled deals and being handed an exact answer that took two
/// million is a worse deal than it looks, however good the answer.
pub fn run_exact_within<V>(
    request: &EquityRequest<V>,
    limit: usize,
) -> Result<Option<ChunkResult>, PokerError>
where
    V: PokerVariant + EquityCalculation,
{
    let seats = request.players();
    let mut result = ChunkResult::empty(seats);
    result.exact = true;

    // Each seat's possible holdings, listed once. A seat whose hand is too
    // loose to list makes the whole request too large to enumerate.
    //
    // The running product is what decides that, and it has to be checked as
    // the seats are listed rather than afterwards. The walk below skips a
    // holding that clashes with one already chosen, and a skip is work done
    // without a deal to show for it -- so on a table where the seats want the
    // same cards, the budget below barely moves while the search runs for
    // ever. Six stud seats with four cards to come each is 211,876 holdings
    // apiece and 9 x 10^30 combinations to sift; counting them first turns
    // that from a hang into an immediate "sample this instead".
    //
    // The product ignores those clashes, so it overstates the real number of
    // deals. That is the safe direction: it can only decline a spot that
    // could have been walked, and declining means sampling, which answers the
    // question either way.
    let mut choices: Vec<Vec<Vec<Card>>> = Vec::with_capacity(seats);
    let mut combinations: u128 = 1;
    for alternatives in &request.seats {
        let mut sets = Vec::new();
        for sampler in alternatives {
            match sampler.all_sets(request.available, limit) {
                Some(found) => sets.extend(found),
                None => return Ok(None),
            }
        }
        if sets.is_empty() {
            return Ok(None);
        }
        combinations = combinations.saturating_mul(sets.len() as u128);
        if combinations > limit as u128 {
            return Ok(None);
        }
        choices.push(sets);
    }

    let mut holes: Vec<Vec<Card>> = vec![Vec::new(); seats];
    let mut shares = vec![0.0f64; seats];
    let mut low_shares = vec![0.0f64; seats];
    let mut budget = limit;

    let walked = walk_seats(
        request,
        &choices,
        0,
        request.available,
        &mut holes,
        &mut shares,
        &mut low_shares,
        &mut result,
        &mut budget,
    )?;

    Ok(walked.then_some(result))
}

/// Chooses a holding for each seat in turn, then walks the boards that go
/// with it. Returns false when the walk ran past its budget.
#[allow(clippy::too_many_arguments)]
fn walk_seats<V>(
    request: &EquityRequest<V>,
    choices: &[Vec<Vec<Card>>],
    seat: usize,
    available: CardSet,
    holes: &mut Vec<Vec<Card>>,
    shares: &mut [f64],
    low_shares: &mut [f64],
    result: &mut ChunkResult,
    budget: &mut usize,
) -> Result<bool, PokerError>
where
    V: PokerVariant + EquityCalculation,
{
    if seat == choices.len() {
        return walk_boards(
            request, available, holes, shares, low_shares, result, budget,
        );
    }

    for holding in &choices[seat] {
        let cards: CardSet = holding.iter().copied().collect();
        // Another seat already holds one of these cards.
        if cards.len() as usize != holding.len() || !cards.intersection(available).eq(&cards) {
            continue;
        }
        holes[seat] = holding.clone();
        if !walk_seats(
            request,
            choices,
            seat + 1,
            available.without(cards),
            holes,
            shares,
            low_shares,
            result,
            budget,
        )? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Walks every board that can follow the holdings already chosen.
fn walk_boards<V>(
    request: &EquityRequest<V>,
    available: CardSet,
    holes: &[Vec<Card>],
    shares: &mut [f64],
    low_shares: &mut [f64],
    result: &mut ChunkResult,
    budget: &mut usize,
) -> Result<bool, PokerError>
where
    V: PokerVariant + EquityCalculation,
{
    let boards = if request.board_slots == 0 {
        vec![Vec::new()]
    } else {
        match request.board.all_sets(available, *budget) {
            Some(found) => found,
            None => return Ok(false),
        }
    };

    let mut hands: Vec<Hand<V>> = (0..holes.len())
        .map(|_| Hand::new(request.variant))
        .collect();

    for board in boards {
        if *budget == 0 {
            return Ok(false);
        }
        *budget -= 1;

        for (hand, hole) in hands.iter_mut().zip(holes.iter()) {
            hand.refill(hole, &board)?;
        }

        shares.iter_mut().for_each(|share| *share = 0.0);
        low_shares.iter_mut().for_each(|share| *share = 0.0);
        request.variant.award_detailed(&hands, shares, low_shares)?;
        result.record(shares, low_shares);
    }
    Ok(true)
}

/// Runs one batch of deals.
///
/// Pure and stateless: no callbacks, no cancellation token, no interior
/// mutability. The caller loops, merges, repaints and decides when to stop,
/// which makes cancellation instant at chunk granularity with nothing
/// threaded through the sampling loop.
pub fn run_chunk<V>(
    request: &EquityRequest<V>,
    samples: u64,
    seed: u64,
) -> Result<ChunkResult, PokerError>
where
    V: PokerVariant + EquityCalculation,
{
    let seats = request.players();
    let mut result = ChunkResult::empty(seats);
    let mut rng = SmallRng::seed_from_u64(seed);

    let mut holes: Vec<Vec<Card>> = vec![Vec::new(); seats];
    let mut board: Vec<Card> = Vec::new();
    let mut shares = vec![0.0f64; seats];
    let mut low_shares = vec![0.0f64; seats];

    // The seats are the same all chunk long; only their cards change. Each
    // hand is built once, with room for the most cards this game deals, and
    // written over deal by deal.
    let mut hands: Vec<Hand<V>> = (0..seats).map(|_| Hand::new(request.variant)).collect();
    let mut scratch = WeighingScratch::default();

    // Feasibility was settled at construction, so a rejected draw is only
    // ever bad luck. The cap keeps a pathological request from spinning
    // rather than deciding anything.
    let attempt_limit = samples.saturating_mul(1000).max(10_000);
    let mut attempts: u64 = 0;

    while result.samples < samples {
        attempts += 1;
        if attempts > attempt_limit {
            // Not an infeasible request, and it must not say so: feasibility
            // was proved at construction. The seats are competing for the
            // same cards faster than a draw can find a deal that suits them
            // all.
            return Err(EquityError::SamplingStalled {
                attempts,
                found: result.samples,
            }
            .into());
        }

        result.attempts += 1;
        let weight = match request.dealing {
            Dealing::Rejecting => {
                if !request.deal(&mut rng, &mut holes, &mut board) {
                    // The cards drawn could not fill every slot, so the whole
                    // deal goes back. Rejecting the deal entire rather than
                    // re-drawing one seat is what keeps the result uniform
                    // over deals.
                    continue;
                }
                1.0
            }
            Dealing::Weighted => {
                match request.deal_weighted(&mut rng, &mut holes, &mut board, &mut scratch) {
                    Some(weight) => weight,
                    None => continue,
                }
            }
        };

        for (hand, hole) in hands.iter_mut().zip(holes.iter()) {
            hand.refill(hole, &board)?;
        }

        shares.iter_mut().for_each(|share| *share = 0.0);
        low_shares.iter_mut().for_each(|share| *share = 0.0);
        request
            .variant
            .award_detailed(&hands, &mut shares, &mut low_shares)?;
        result.record_weighted(&shares, &low_shares, weight);
    }

    Ok(result)
}
