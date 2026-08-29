use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    cards::{Card, CardSet},
    error::{EquityError, PokerError},
    hand::Hand,
    notation::{parse_board, parse_dead, parse_hand, HandSpec},
    sampler::{is_feasible, SlotSampler},
    variants::{EquityCalculation, PokerVariant},
};

use super::chunk::ChunkResult;

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
    /// The board's slots, padded out with wildcards to the game's full board.
    board: SlotSampler,
    board_slots: usize,
    available: CardSet,
    players: usize,
}

impl<V: PokerVariant + EquityCalculation> EquityRequest<V> {
    /// Builds a request from masks, with no text involved.
    ///
    /// This is the entry point fpdb uses; the text form in
    /// [`from_text`](Self::from_text) sits on top of it.
    pub fn from_masks(
        variant: V,
        hands: &[HandSpec],
        board: &[CardSet],
        dead: CardSet,
    ) -> Result<Self, PokerError> {
        if hands.len() < 2 {
            return Err(EquityError::NoPlayers.into());
        }

        let hole = variant.hole_cards();
        for spec in hands {
            match spec.slot_count() {
                Some(count) if count == hole => {}
                Some(count) => return Err(EquityError::NotEnoughCards(count).into()),
                None => return Err(EquityError::UnequalHandSizes.into()),
            }
            if !spec.is_satisfiable() {
                return Err(EquityError::Infeasible("a hand".to_string()).into());
            }
        }

        let board_slots = variant.board_cards();
        if board.len() > board_slots {
            return Err(EquityError::InvalidCommunityCards(board.len()).into());
        }

        // Anything in this game's deck and not dead is on offer. Known cards
        // are not removed here: they are slots with exactly one candidate, so
        // the matching handles them and the sampler cannot draw them twice.
        let available = variant.deck().without(dead);

        // Padding the board with wildcards makes a short board the same shape
        // as a full one, which is how a flop and a river spot share a path.
        let mut board_masks = board.to_vec();
        board_masks.resize(board_slots, CardSet::FULL_DECK);

        Self::check_feasible(hands, &board_masks, available)?;

        let seats = hands
            .iter()
            .map(|spec| {
                spec.alternatives
                    .iter()
                    .map(|slots| SlotSampler::new(slots, available))
                    .collect()
            })
            .collect();

        Ok(Self {
            variant,
            seats,
            board: SlotSampler::new(&board_masks, available),
            board_slots,
            available,
            players: hands.len(),
        })
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
        let specs = hands
            .iter()
            .map(|text| parse_hand(text, variant.hole_cards()))
            .collect::<Result<Vec<_>, _>>()?;
        let board = parse_board(board, variant.board_cards())?;
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

    /// Deals once, writing each seat's cards into `holes` and the shared
    /// cards into `board`. Returns false when the draw failed and should be
    /// retried.
    fn deal(
        &self,
        rng: &mut StdRng,
        holes: &mut [Vec<Card>],
        board: &mut Vec<Card>,
    ) -> bool {
        let mut available = self.available;

        for (seat, alternatives) in self.seats.iter().enumerate() {
            let sampler = &alternatives[rng.gen_range(0..alternatives.len())];
            if !sampler.draw(available, rng, &mut holes[seat]) {
                return false;
            }
            for &card in &holes[seat] {
                available.remove(card);
            }
        }

        if self.board_slots > 0 {
            if !self.board.draw(available, rng, board) {
                return false;
            }
        } else {
            board.clear();
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
    let seats = request.players();
    let mut result = ChunkResult::empty(seats);
    result.exact = true;

    // Each seat's possible holdings, listed once. A seat whose hand is too
    // loose to list makes the whole request too large to enumerate.
    let mut choices: Vec<Vec<Vec<Card>>> = Vec::with_capacity(seats);
    for alternatives in &request.seats {
        let mut sets = Vec::new();
        for sampler in alternatives {
            match sampler.all_sets(request.available, EXACT_DEAL_LIMIT) {
                Some(found) => sets.extend(found),
                None => return Ok(None),
            }
        }
        if sets.is_empty() {
            return Ok(None);
        }
        choices.push(sets);
    }

    let mut holes: Vec<Vec<Card>> = vec![Vec::new(); seats];
    let mut shares = vec![0.0f64; seats];
    let mut low_shares = vec![0.0f64; seats];
    let mut budget = EXACT_DEAL_LIMIT;

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
        return walk_boards(request, available, holes, shares, low_shares, result, budget);
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

    for board in boards {
        if *budget == 0 {
            return Ok(false);
        }
        *budget -= 1;

        let hands = holes
            .iter()
            .map(|hole| {
                let mut cards = hole.clone();
                cards.extend_from_slice(&board);
                Hand::new_with_cards(request.variant, cards)
            })
            .collect::<Result<Vec<_>, _>>()?;

        shares.iter_mut().for_each(|share| *share = 0.0);
        low_shares.iter_mut().for_each(|share| *share = 0.0);
        request
            .variant
            .award_detailed(&hands, shares, low_shares)?;
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
    let mut rng = StdRng::seed_from_u64(seed);

    let mut holes: Vec<Vec<Card>> = vec![Vec::new(); seats];
    let mut board: Vec<Card> = Vec::new();
    let mut shares = vec![0.0f64; seats];
    let mut low_shares = vec![0.0f64; seats];

    // Feasibility was settled at construction, so a rejected draw is only
    // ever bad luck. The cap keeps a pathological request from spinning
    // rather than deciding anything.
    let attempt_limit = samples.saturating_mul(1000).max(10_000);
    let mut attempts: u64 = 0;

    while result.samples < samples {
        attempts += 1;
        if attempts > attempt_limit {
            return Err(EquityError::Infeasible(
                "the request, which almost never yields a deal".to_string(),
            )
            .into());
        }

        if !request.deal(&mut rng, &mut holes, &mut board) {
            continue;
        }

        let hands = holes
            .iter()
            .map(|hole| {
                let mut cards = hole.clone();
                cards.extend_from_slice(&board);
                Hand::new_with_cards(request.variant, cards)
            })
            .collect::<Result<Vec<_>, _>>()?;

        shares.iter_mut().for_each(|share| *share = 0.0);
        low_shares.iter_mut().for_each(|share| *share = 0.0);
        request
            .variant
            .award_detailed(&hands, &mut shares, &mut low_shares)?;
        result.record(&shares, &low_shares);
    }

    Ok(result)
}
