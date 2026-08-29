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
    /// Cards that every valid deal must contain, taken from slots that admit
    /// exactly one card.
    fixed: Vec<Card>,
    /// What is left to draw for once the forced slots are settled.
    slots: Vec<CardSet>,
    strategy: Strategy,
}

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

        let pool = slots
            .iter()
            .fold(CardSet::EMPTY, |all, slot| all.union(*slot))
            .intersection(free);

        // Nothing to decide when every slot takes anything still on offer.
        let unconstrained = slots.iter().all(|slot| free.without(*slot).is_empty());

        let strategy = if unconstrained {
            Strategy::Free
        } else if let Some(sets) = list_valid_sets(&slots, pool) {
            Strategy::Listed(sets)
        } else {
            Strategy::DrawAndTest { pool }
        };

        Self {
            fixed,
            slots,
            strategy,
        }
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
        }
        filled
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

        let pool = match &self.strategy {
            Strategy::Free => available,
            Strategy::Listed(_) | Strategy::DrawAndTest { .. } => self
                .slots
                .iter()
                .fold(CardSet::EMPTY, |all, slot| all.union(*slot))
                .intersection(available),
        };

        let cards: Vec<Card> = pool.iter().collect();
        if cards.len() < self.slots.len() {
            return Some(Vec::new());
        }
        if binomial_capped(cards.len(), self.slots.len(), limit) > limit {
            return None;
        }

        let unconstrained = matches!(self.strategy, Strategy::Free);
        Some(
            cards
                .into_iter()
                .combinations(self.slots.len())
                .filter(|set| unconstrained || has_perfect_matching(&self.slots, set))
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
