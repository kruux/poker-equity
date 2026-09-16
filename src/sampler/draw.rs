use itertools::Itertools;
use rand::Rng;

use crate::cards::{Card, CardSet};

use super::matching::has_perfect_matching;
use super::shape::ShapePlan;

/// How many valid sets are worth holding in a list.
///
/// This bounds the *answer*, not the search. An Omaha `AA**` covers 6,961
/// hands and is trivially listable, but `C(52,4)` is 270,725 -- so a limit
/// read against the search space refuses a hand it could have listed a
/// hundred times over. The shapes in [`super::shape`] count the hands
/// without generating any, so the right number is the one being asked about.
const ENUMERATION_LIMIT: usize = 200_000;

/// How a sampler fills its slots.
#[derive(Debug, Clone)]
enum Strategy {
    /// Every slot admits every available card, so any draw is valid and no
    /// matching is needed. This is boards, stud deals, and most hands.
    Free,
    /// The valid sets, listed once at construction. Used when the slots are
    /// constrained but the space they cover is small, which is the case that
    /// would otherwise reject most of its draws.
    Listed(Vec<Vec<Card>>),
    /// Too many valid sets to hold, so they are drawn from directly: pick a
    /// shape in proportion to how many sets have it, then take that many
    /// cards from each group. Uniform over the sets, and nothing is drawn
    /// only to be thrown away.
    ///
    /// This is what a seven-card hand written as three ranks needs. `A23`
    /// covers 9,215,488 hands, which is far too many to list and no trouble
    /// at all to count.
    Shaped(ShapePlan),
    /// Draw a subset and keep it only if the slots can be filled from it.
    ///
    /// The last resort, for slots that cut the deck too finely to shape.
    /// Nothing the notation can write lands here.
    DrawAndTest { pool: CardSet },
}

/// Fills a group of slots with cards, uniformly over the *sets* the slots
/// admit.
///
/// The distinction matters and is easy to get wrong. Sampling one card per
/// slot and rejecting collisions is uniform over ordered assignments, but a
/// hand is an unordered set, and when slot masks overlap only partly the sets
/// do not have equal multiplicity. With masks `{A,B,C}` and `{A,B}`, the set
/// `{A,B}` is reachable two ways and `{C,A}` only one, so ordered sampling
/// over-weights it twofold. The equities come out self-consistent, stable and
/// wrong.
///
/// This draws a subset of the right size and keeps it only when the slots can
/// all be filled from it, which is uniform over valid sets by construction.
#[derive(Debug, Clone)]
pub struct SlotSampler {
    /// Cards that every valid deal must contain, taken from slots that admit
    /// exactly one card.
    fixed: Vec<Card>,
    /// What is left to draw for once the forced slots are settled.
    slots: Vec<CardSet>,
    /// The slots that want particular cards, drawn for by `strategy`.
    ///
    /// Empty when every slot takes anything, which is the ordinary case.
    constrained: Vec<CardSet>,
    /// How many slots take anything at all, drawn from whatever is left once
    /// the constrained ones have chosen.
    open: usize,
    /// Whether no two constrained slots want the same card, which is true of
    /// every field the notation can write -- named ranks do not overlap, nor
    /// do named suits. It makes the multiplicity a product rather than a
    /// walk over subsets.
    disjoint: bool,
    strategy: Strategy,
}

/// The most slots that will be split into constrained and open.
///
/// The multiplicity below walks the subsets of a drawn hand, so the width has
/// to be bounded. No game deals more than seven cards, and a wider slot list
/// falls back to drawing the hand whole.
const MOST_SPLIT_SLOTS: usize = 8;

/// Pulls out the slots that admit exactly one card.
///
/// A slot with a single candidate takes that card in every possible deal, so
/// the card and the slot can both be removed without changing which sets are
/// valid. Doing it first is what keeps a named board card from being sampled
/// for: three known flop cards among five board slots would otherwise mean
/// drawing five cards and hoping all three turn up.
///
/// Removing one card can leave another slot with a single candidate, so this
/// repeats until it settles.
fn extract_forced(slots: &[CardSet], available: CardSet) -> (Vec<Card>, Vec<CardSet>) {
    let mut open: Vec<CardSet> = slots.iter().map(|s| s.intersection(available)).collect();
    let mut fixed: Vec<Card> = Vec::new();

    while let Some(index) = open.iter().position(|slot| slot.len() == 1) {
        let Some(card) = open[index].iter().next() else {
            break;
        };
        fixed.push(card);
        open.remove(index);
        for slot in open.iter_mut() {
            slot.remove(card);
        }
    }

    (fixed, open)
}

impl SlotSampler {
    /// Prepares to fill `slots` from `available`.
    ///
    /// `available` is what the deck holds before any of this deal is dealt;
    /// the cards other players take are excluded at draw time.
    pub fn new(slots: &[CardSet], available: CardSet) -> Self {
        let (fixed, slots) = extract_forced(slots, available);
        let free = available.without(CardSet::from_cards(&fixed));

        // Nothing to decide when every slot takes anything still on offer.
        let unconstrained = slots.iter().all(|slot| free.without(*slot).is_empty());

        // The slots that want particular cards are drawn for separately from
        // the ones that will take anything, and that split is what makes a
        // multi-way spot workable. A seat's picky cards are few -- three, for
        // a razz hand written as three ranks -- and only those have to be
        // drawn against the whole deck and thrown away when an earlier seat
        // has taken one. The four cards that take anything are drawn from
        // what is actually left, so they never collide with anybody.
        //
        // Drawing them from what is left is safe precisely because it is not
        // renormalised: the draw is uniform over `C(cards left, open slots)`,
        // and every seat takes the same number of cards whatever they are, so
        // that count is the same down every branch of the deal. It is
        // dividing by the number of hands a seat has left -- not drawing from
        // them -- that would favour the deals following a card-hungry seat.
        let constrained: Vec<CardSet> = if unconstrained || slots.len() > MOST_SPLIT_SLOTS {
            Vec::new()
        } else {
            slots
                .iter()
                .filter(|slot| !free.without(**slot).is_empty())
                .copied()
                .collect()
        };
        let open = slots.len() - constrained.len();
        let disjoint = constrained.iter().enumerate().all(|(index, slot)| {
            constrained[index + 1..]
                .iter()
                .all(|other| slot.is_disjoint(*other))
        });

        // What the picky slots are drawn from. Falls back to the whole slot
        // list when the split was refused, which keeps the old behaviour.
        let planned: &[CardSet] = if constrained.is_empty() {
            &slots
        } else {
            &constrained
        };
        let pool = planned
            .iter()
            .fold(CardSet::EMPTY, |all, slot| all.union(*slot))
            .intersection(free);

        // Counting comes before listing, and settles which of the two the
        // slots want. The shapes give the exact number of valid sets without
        // building one of them, so a hand small enough to hold is listed and
        // a hand too large to hold is drawn from by shape -- and neither
        // decision rests on `C(deck, slots)`, which is the size of a search
        // nobody has to do.
        let strategy = if unconstrained {
            Strategy::Free
        } else if let Some(plan) = ShapePlan::build(planned, pool) {
            // No hand at all is a listable answer -- the empty list -- and
            // says so at every draw. Seven slots wanting an ace apiece is
            // the shape of it.
            if plan.total() == 0 {
                Strategy::Listed(Vec::new())
            } else if plan.total() <= ENUMERATION_LIMIT as u128 {
                Strategy::Listed(plan.enumerate())
            } else {
                Strategy::Shaped(plan)
            }
        } else {
            Strategy::DrawAndTest { pool }
        };

        Self {
            fixed,
            slots,
            constrained,
            open,
            disjoint,
            strategy,
        }
    }

    /// How many ways the constrained slots could have been filled from
    /// `cards`, which is what the split has to be corrected for.
    ///
    /// Drawing the picky cards and then the open ones reaches a hand once per
    /// way of splitting it. A razz hand holding two aces could have had
    /// either as its named ace and the other among the open cards, so it
    /// arrives twice as often as it should. Keeping it with probability
    /// `1/multiplicity` puts that right, and the correction almost never
    /// fires: a hand with exactly one of each named rank counts one.
    ///
    /// No two constrained slots overlap in anything the notation can write,
    /// and then the count is just how many of the drawn cards each slot
    /// wants, multiplied. The general form walks the subsets instead.
    fn multiplicity(&self, cards: &[Card]) -> u32 {
        if self.disjoint {
            return self
                .constrained
                .iter()
                .map(|slot| cards.iter().filter(|card| slot.contains(**card)).count() as u32)
                .product();
        }

        // Sets, not assignments. Two slots both wanting an ace can be filled
        // from one pair of aces two ways round, and that is one way to have
        // drawn the hand, not two -- counting the orderings would throw away
        // half of every `AA**` for nothing.
        let width = cards.len();
        let mut chosen = [cards[0]; MOST_SPLIT_SLOTS];
        let mut ways = 0;
        for mask in 0..(1u32 << width) {
            if mask.count_ones() as usize != self.constrained.len() {
                continue;
            }
            let mut taken = 0;
            for (index, card) in cards.iter().enumerate() {
                if mask >> index & 1 == 1 {
                    chosen[taken] = *card;
                    taken += 1;
                }
            }
            if has_perfect_matching(&self.constrained, &chosen[..taken]) {
                ways += 1;
            }
        }
        ways
    }

    /// The same sampler, forced to draw and test.
    ///
    /// Both strategies must give the same distribution, so the tests run the
    /// uniformity checks through this as well as through the listed path.
    #[cfg(test)]
    pub(crate) fn forcing_draw_and_test(slots: &[CardSet], available: CardSet) -> Self {
        let slots = slots.to_vec();
        let pool = slots
            .iter()
            .fold(CardSet::EMPTY, |all, slot| all.union(*slot))
            .intersection(available);
        Self {
            fixed: Vec::new(),
            slots,
            constrained: Vec::new(),
            open: 0,
            disjoint: true,
            strategy: Strategy::DrawAndTest { pool },
        }
    }

    /// How many cards this fills, forced and drawn together.
    pub fn slot_count(&self) -> usize {
        self.fixed.len() + self.slots.len()
    }

    /// How many of those are settled before any draw.
    pub fn fixed_count(&self) -> usize {
        self.fixed.len()
    }

    /// Which strategy was chosen, for tests and diagnostics.
    pub fn strategy(&self) -> &'static str {
        match self.strategy {
            Strategy::Free => "free",
            Strategy::Listed(_) => "listed",
            Strategy::Shaped(_) => "shaped",
            Strategy::DrawAndTest { .. } => "draw-and-test",
        }
    }

    /// Fills `out` with one card per slot, or reports that this attempt found
    /// no valid set.
    ///
    /// A `false` return is a rejection, not an error: the caller draws again.
    /// Feasibility is settled before sampling starts, so rejections mean the
    /// draw was unlucky, never that the request was impossible.
    pub fn draw(&self, available: CardSet, rng: &mut impl Rng, out: &mut Vec<Card>) -> bool {
        out.clear();

        // The forced cards come first. Another seat may have taken one this
        // deal, which is a rejection like any other.
        let mut available = available;
        for &card in &self.fixed {
            if !available.contains(card) {
                out.clear();
                return false;
            }
            available.remove(card);
            out.push(card);
        }
        if self.slots.is_empty() {
            return true;
        }

        let drawn_from = out.len();
        let filled = match &self.strategy {
            Strategy::Free => draw_subset(available, self.slots.len(), rng, out),
            Strategy::Listed(sets) => {
                if sets.is_empty() {
                    false
                } else {
                    let set = &sets[rng.gen_range(0..sets.len())];
                    if set.iter().any(|card| !available.contains(*card)) {
                        false // taken by an earlier seat this deal
                    } else {
                        out.extend_from_slice(set);
                        true
                    }
                }
            }
            // Drawn against the deck the plan was built from, not against
            // what is left this deal, and rejected whole when an earlier seat
            // has taken one of the cards. Redrawing against the live deck
            // would be faster and would quietly favour the deals that
            // followed a card-hungry seat: how many hands a constrained seat
            // has left depends on *which* cards went before it, not merely
            // how many. This is the same discipline `Listed` follows.
            Strategy::Shaped(plan) => {
                plan.draw(rng, out);
                out[drawn_from..]
                    .iter()
                    .all(|card| available.contains(*card))
            }
            Strategy::DrawAndTest { pool } => {
                let pool = pool.intersection(available);
                if draw_subset(pool, self.slots.len(), rng, out) {
                    has_perfect_matching(&self.slots, &out[drawn_from..])
                } else {
                    false
                }
            }
        };

        if !filled {
            out.clear();
            return false;
        }

        // The slots that take anything are filled last, from what is left
        // rather than from the whole deck, so they never clash with a seat
        // that has already drawn.
        if self.open > 0 && !self.constrained.is_empty() {
            for &card in &out[drawn_from..] {
                available.remove(card);
            }
            if !draw_subset(available, self.open, rng, out) {
                out.clear();
                return false;
            }

            // A hand reachable several ways arrives that many times too
            // often, so it is kept one time in that many.
            let ways = self.multiplicity(&out[drawn_from..]);
            if ways > 1 && rng.gen_range(0..ways) != 0 {
                out.clear();
                return false;
            }
        }

        true
    }
}

impl SlotSampler {
    /// Every set of cards these slots admit from `available`, or `None` when
    /// there are more than `limit` of them.
    ///
    /// This is what exact enumeration walks. It is the same set of deals the
    /// sampler draws from, listed rather than sampled, so the two modes
    /// cannot disagree about which deals are possible.
    pub fn all_sets(&self, available: CardSet, limit: usize) -> Option<Vec<Vec<Card>>> {
        let mut available = available;
        for &card in &self.fixed {
            if !available.contains(card) {
                return Some(Vec::new());
            }
            available.remove(card);
        }

        if self.slots.is_empty() {
            return Some(vec![self.fixed.clone()]);
        }

        let unconstrained = matches!(self.strategy, Strategy::Free);
        let pool = if unconstrained {
            available
        } else {
            self.slots
                .iter()
                .fold(CardSet::EMPTY, |all, slot| all.union(*slot))
                .intersection(available)
        };

        let cards: Vec<Card> = pool.iter().collect();
        if cards.len() < self.slots.len() {
            return Some(Vec::new());
        }

        // Constrained slots are counted before anything is built, so a hand
        // whose *answer* fits is walked even where the search it would have
        // taken does not. `AA**` is 6,961 hands out of 270,725 candidates,
        // and the old bound read the second number.
        let sets: Vec<Vec<Card>> = if unconstrained {
            if binomial_capped(cards.len(), self.slots.len(), limit) > limit {
                return None;
            }
            cards.into_iter().combinations(self.slots.len()).collect()
        } else if let Some(plan) = ShapePlan::build(&self.slots, pool) {
            if plan.total() > limit as u128 {
                return None;
            }
            plan.enumerate()
        } else {
            // Slots too finely cut to shape: back to searching and testing.
            if binomial_capped(cards.len(), self.slots.len(), limit) > limit {
                return None;
            }
            cards
                .into_iter()
                .combinations(self.slots.len())
                .filter(|set| has_perfect_matching(&self.slots, set))
                .collect()
        };

        Some(
            sets.into_iter()
                .map(|set| {
                    let mut whole = self.fixed.clone();
                    whole.extend(set);
                    whole
                })
                .collect(),
        )
    }
}

/// `n` choose `k`, stopping once it passes `limit`.
fn binomial_capped(n: usize, k: usize, limit: usize) -> usize {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut result: usize = 1;
    for step in 0..k {
        let Some(next) = result.checked_mul(n - step) else {
            return usize::MAX;
        };
        result = next / (step + 1);
        if result > limit {
            return result;
        }
    }
    result
}

/// Draws `count` distinct cards uniformly from `pool`.
///
/// Taking them one at a time without replacement makes every subset of that
/// size equally likely, which is the property the sampler rests on.
fn draw_subset(pool: CardSet, count: usize, rng: &mut impl Rng, out: &mut Vec<Card>) -> bool {
    let mut remaining = pool;
    for _ in 0..count {
        let left = remaining.len();
        if left == 0 {
            return false;
        }
        let Some(card) = remaining.nth(rng.gen_range(0..left)) else {
            return false;
        };
        remaining.remove(card);
        out.push(card);
    }
    true
}
