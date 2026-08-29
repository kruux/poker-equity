use std::fmt;

use crate::cards::{Card, CardSet, Rank, Suit};

/// One hand field: a list of alternatives, each a whole hand given as one
/// mask per slot.
///
/// Every form the notation admits collapses to this shape. `AhKh` is one
/// alternative of two single-card masks; `AKs` is four such alternatives;
/// `2 c` is one alternative of two many-card masks. The sampler never asks
/// which kind it was handed, which is what keeps wildcards off the fast path
/// as a special case.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HandSpec {
    pub alternatives: Vec<Vec<CardSet>>,
}

impl HandSpec {
    /// A spec with a single alternative, built straight from masks.
    ///
    /// This is the entry point for callers that never touch text.
    pub fn from_slots(slots: &[CardSet]) -> Self {
        Self {
            alternatives: vec![slots.to_vec()],
        }
    }

    /// A spec from several whole alternatives.
    pub fn from_alternatives(alternatives: Vec<Vec<CardSet>>) -> Self {
        Self { alternatives }
    }

    /// One alternative per concrete two-card combination.
    pub(crate) fn from_combos(combos: &[[Card; 2]]) -> Self {
        Self {
            alternatives: combos
                .iter()
                .map(|[a, b]| vec![CardSet::from_cards(&[*a]), CardSet::from_cards(&[*b])])
                .collect(),
        }
    }

    /// How many slots each alternative holds, or `None` when the spec is
    /// empty or its alternatives disagree.
    pub fn slot_count(&self) -> Option<usize> {
        let first = self.alternatives.first()?.len();
        self.alternatives
            .iter()
            .all(|alternative| alternative.len() == first)
            .then_some(first)
    }

    /// Every card any alternative could use.
    pub fn covered_cards(&self) -> CardSet {
        self.alternatives
            .iter()
            .flatten()
            .fold(CardSet::EMPTY, |all, slot| all.union(*slot))
    }

    /// Whether any alternative could actually be dealt: no slot may be empty.
    pub fn is_satisfiable(&self) -> bool {
        self.alternatives
            .iter()
            .any(|alternative| !alternative.is_empty() && alternative.iter().all(|s| !s.is_empty()))
    }
}

/// Renders one slot in the notation that produced it.
///
/// The grammar can only make four shapes -- a named card, a whole rank, a
/// whole suit, or anything at all -- so those round-trip. A mask built
/// directly through [`HandSpec::from_slots`] may be none of them, and is
/// listed in brackets instead.
fn write_slot(f: &mut fmt::Formatter, slot: CardSet) -> fmt::Result {
    if slot == CardSet::FULL_DECK {
        return write!(f, "*");
    }
    if slot.len() == 1 {
        if let Some(card) = slot.iter().next() {
            return write!(f, "{}", card);
        }
    }
    for rank in Rank::all() {
        if slot == CardSet::of_rank(rank) {
            return write!(f, "{}", Rank::to_char(rank));
        }
    }
    for suit in Suit::all() {
        if slot == CardSet::of_suit(suit) {
            return write!(f, "{}", Suit::to_char(suit));
        }
    }
    write!(f, "[{}]", slot)
}

impl fmt::Display for HandSpec {
    /// The canonical form: uppercase ranks, lowercase suits, slots separated
    /// by spaces and alternatives by commas.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, alternative) in self.alternatives.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            for (j, slot) in alternative.iter().enumerate() {
                if j > 0 {
                    write!(f, " ")?;
                }
                write_slot(f, *slot)?;
            }
        }
        Ok(())
    }
}
