use crate::cards::{Card, CardSet};

/// The largest number of slots that can be filled at once from `cards`, where
/// slot *i* accepts any card in `slots[i]`.
///
/// This is a maximum bipartite matching, found by augmenting paths. With the
/// thirty or so slots a full table produces it settles in microseconds, which
/// is why the sampler can afford to ask the question per deal.
pub fn maximum_matching(slots: &[CardSet], cards: &[Card]) -> usize {
    // `taken_by[c]` is the slot currently holding card `c`, if any.
    let mut taken_by: Vec<Option<usize>> = vec![None; cards.len()];
    let mut matched = 0;

    for slot in 0..slots.len() {
        let mut seen = vec![false; cards.len()];
        if augment(slot, slots, cards, &mut seen, &mut taken_by) {
            matched += 1;
        }
    }

    matched
}

/// Tries to seat `slot`, displacing earlier slots along the way if they can
/// move somewhere else.
fn augment(
    slot: usize,
    slots: &[CardSet],
    cards: &[Card],
    seen: &mut [bool],
    taken_by: &mut [Option<usize>],
) -> bool {
    for (index, &card) in cards.iter().enumerate() {
        if seen[index] || !slots[slot].contains(card) {
            continue;
        }
        seen[index] = true;
        let free = match taken_by[index] {
            None => true,
            Some(holder) => augment(holder, slots, cards, seen, taken_by),
        };
        if free {
            taken_by[index] = Some(slot);
            return true;
        }
    }
    false
}

/// Whether every slot can be filled at once from `cards`, one card each.
///
/// A count is not enough to decide this. Five deuces is caught by counting,
/// but two slots that both admit only the ace of hearts is not, and both are
/// requests no deal can satisfy.
pub fn has_perfect_matching(slots: &[CardSet], cards: &[Card]) -> bool {
    cards.len() >= slots.len() && maximum_matching(slots, cards) == slots.len()
}

/// Whether every slot could be filled from `available`, taken as a whole.
///
/// This is Hall's condition, decided the same way. It can prove a request
/// impossible; it never rejects one that is satisfiable.
pub fn is_feasible(slots: &[CardSet], available: CardSet) -> bool {
    if slots.iter().any(|slot| slot.intersection(available).is_empty()) {
        return false;
    }
    let cards: Vec<Card> = available.iter().collect();
    has_perfect_matching(slots, &cards)
}
