use itertools::Itertools;
use rand::Rng;

use crate::cards::{Card, CardSet};

use super::matching::has_perfect_matching;

/// Above this many candidate sets the sampler draws and tests rather than
/// listing them. `C(24, 6)` is about 135k, so every constrained hand a real
/// request produces sits well under it.
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
    /// Draw a subset and keep it only if the slots can be filled from it.
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
    slots: Vec<CardSet>,
    strategy: Strategy,
}

impl SlotSampler {
    /// Prepares to fill `slots` from `available`.
    ///
    /// `available` is what the deck holds before any of this deal is dealt;
    /// the cards other players take are excluded at draw time.
    pub fn new(slots: &[CardSet], available: CardSet) -> Self {
        let slots = slots.to_vec();
        let pool = slots
            .iter()
            .fold(CardSet::EMPTY, |all, slot| all.union(*slot))
            .intersection(available);

        // Nothing to decide when every slot takes anything on offer.
        let unconstrained = slots
            .iter()
            .all(|slot| available.without(*slot).is_empty());

        let strategy = if unconstrained {
            Strategy::Free
        } else if let Some(sets) = list_valid_sets(&slots, pool) {
            Strategy::Listed(sets)
        } else {
            Strategy::DrawAndTest { pool }
        };

        Self { slots, strategy }
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
            slots,
            strategy: Strategy::DrawAndTest { pool },
        }
    }

    /// How many cards this fills.
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Which strategy was chosen, for tests and diagnostics.
    pub fn strategy(&self) -> &'static str {
        match self.strategy {
            Strategy::Free => "free",
            Strategy::Listed(_) => "listed",
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
        match &self.strategy {
            Strategy::Free => draw_subset(available, self.slots.len(), rng, out),
            Strategy::Listed(sets) => {
                if sets.is_empty() {
                    return false;
                }
                let set = &sets[rng.gen_range(0..sets.len())];
                if set.iter().any(|card| !available.contains(*card)) {
                    return false; // taken by an earlier seat this deal
                }
                out.extend_from_slice(set);
                true
            }
            Strategy::DrawAndTest { pool } => {
                let pool = pool.intersection(available);
                if !draw_subset(pool, self.slots.len(), rng, out) {
                    return false;
                }
                has_perfect_matching(&self.slots, out)
            }
        }
    }
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

/// Lists every set of cards that can fill `slots`, or `None` when there are
/// too many to be worth holding.
fn list_valid_sets(slots: &[CardSet], pool: CardSet) -> Option<Vec<Vec<Card>>> {
    let size = slots.len();
    let cards: Vec<Card> = pool.iter().collect();
    if size == 0 || cards.len() < size {
        return Some(Vec::new());
    }
    if binomial(cards.len(), size)? > ENUMERATION_LIMIT {
        return None;
    }
    Some(
        cards
            .into_iter()
            .combinations(size)
            .filter(|set| has_perfect_matching(slots, set))
            .collect(),
    )
}

/// `n` choose `k`, or `None` if it overflows past anything we would keep.
fn binomial(n: usize, k: usize) -> Option<usize> {
    if k > n {
        return Some(0);
    }
    let k = k.min(n - k);
    let mut result: usize = 1;
    for step in 0..k {
        result = result.checked_mul(n - step)? / (step + 1);
        if result > ENUMERATION_LIMIT {
            return Some(result);
        }
    }
    Some(result)
}
