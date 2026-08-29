use std::cmp::Ordering;

use super::table_index::{bucket_of, slot_of, BUCKET_COUNT, SLOT_COUNT};
use super::{
    DEUCE_SEVEN_FLUSH_SCORES, DEUCE_SEVEN_HAND_SCORES, HAND_DISPLACEMENTS, HIGH_FLUSH_SCORES,
    HIGH_HAND_SCORES, LOW_A5_HAND_SCORES, SHORT_DECK_FLUSH_SCORES, SHORT_DECK_HAND_SCORES,
};
use crate::cards::{Card, Suit};

/// A hand's strength as a single number, read from a lookup table.
///
/// Lower is better: the tables number hands from the royal flush down, so the
/// score is the hand's place in the standard ordering. `u16::MAX` means the
/// hand is too short to score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FastHandRank(pub u16);

/// The score reported for a hand that cannot be evaluated.
const NOTHING: u16 = u16::MAX;

// build.rs sizes the tables from the same constants the lookups use, so this
// only fires if the two ever stop agreeing -- and it fires at compile time
// rather than as a wrong answer.
const _: () = assert!(HIGH_HAND_SCORES.len() == SLOT_COUNT * 2);
const _: () = assert!(DEUCE_SEVEN_HAND_SCORES.len() == SLOT_COUNT * 2);
const _: () = assert!(SHORT_DECK_HAND_SCORES.len() == SLOT_COUNT * 2);
const _: () = assert!(LOW_A5_HAND_SCORES.len() == SLOT_COUNT * 2);
const _: () = assert!(HAND_DISPLACEMENTS.len() == BUCKET_COUNT * 2);
const _: () = assert!(HIGH_FLUSH_SCORES.len() == (1 << 13) * 2);

/// Base-five place values, one per rank.
///
/// A hand holds at most four of any rank, so the digits never carry and the
/// sum identifies the rank multiset uniquely. This is the key the rank table
/// is built around.
pub const RANK_KEYS: [u32; 13] = [
    1,
    5,
    25,
    125,
    625,
    3_125,
    15_625,
    78_125,
    390_625,
    1_953_125,
    9_765_625,
    48_828_125,
    244_140_625,
];

/// One bit per rank, for collecting a suit's ranks into a flush key.
pub const FLUSH_KEYS: [u32; 13] = [
    0x0001, 0x0002, 0x0004, 0x0008, 0x0010, 0x0020, 0x0040, 0x0080, 0x0100, 0x0200, 0x0400, 0x0800,
    0x1000,
];

/// Reads the `index`th score from a table of little-endian `u16`s.
fn score_at(table: &[u8], index: usize) -> u16 {
    let at = index * 2;
    u16::from_le_bytes([table[at], table[at + 1]])
}

/// Scores a hand against one kernel's tables.
///
/// `flushes` is `None` for a ranking where suits do not matter, which is the
/// ace-to-five low: there is no flush to look for and no table to look in.
///
/// Lower is better throughout. Hands of fewer than five or more than seven
/// cards come back as `NOTHING`, and the guard is not tidiness: the tables
/// hold nothing but scores, so a key that is not a hand would land on some
/// other hand's slot and be read as its own.
fn score(cards: &[Card], hands: &[u8], flushes: Option<&[u8]>) -> u16 {
    if !(5..=7).contains(&cards.len()) {
        return NOTHING;
    }

    if let Some(flushes) = flushes {
        // Each suit's ranks gathered into a thirteen-bit mask. Masks of fewer
        // than five bits are marked empty, so a miss costs one read.
        let mut suits = [0u32; 4];
        for card in cards {
            let rank = (card.rank().to_value() - 2) as usize;
            suits[card.suit() as usize] |= FLUSH_KEYS[rank];
        }
        for suit in suits {
            let found = score_at(flushes, suit as usize);
            if found != NOTHING {
                return found;
            }
        }
    }

    // No flush, so the hand is worth whatever its ranks are worth. The key is
    // far too sparse to index directly, so it goes through the displacement
    // in `table_index`.
    let key: u32 = cards
        .iter()
        .map(|card| RANK_KEYS[(card.rank().to_value() - 2) as usize])
        .sum();
    let displacement = score_at(HAND_DISPLACEMENTS, bucket_of(key));
    score_at(hands, slot_of(key, displacement))
}

/// The base-five rank key of some cards.
///
/// Keys **add**: the key of a hand is the sum of the keys of any way of
/// splitting it. That is what lets Omaha score sixty hands from ten board
/// keys and six hole keys, rather than walking five cards sixty times.
pub fn rank_key(cards: &[Card]) -> u32 {
    cards
        .iter()
        .map(|card| RANK_KEYS[(card.rank().to_value() - 2) as usize])
        .sum()
}

/// The suit these cards share and their thirteen-bit rank mask, or `None`
/// when they are not all of one suit.
///
/// A five-card flush is five cards of one suit, so in Omaha it needs both
/// hole cards and all three board cards to share a suit -- which means the
/// question can be asked of each half separately and the answers combined.
pub fn shared_suit(cards: &[Card]) -> Option<(Suit, u16)> {
    let first = cards.first()?.suit();
    let mut mask = 0u16;
    for card in cards {
        if card.suit() != first {
            return None;
        }
        mask |= 1 << (card.rank().to_value() - 2);
    }
    Some((first, mask))
}

/// A high score from a rank key that has already been summed, and a flush
/// mask when the hand is five cards of one suit.
///
/// The caller has to know the hand is a flush, which in Omaha it does: only
/// two hole cards and three board cards play, so a flush is exactly the case
/// where both halves are of the same one suit.
pub fn high_score_from_parts(rank_key: u32, flush: Option<u16>) -> u16 {
    if let Some(mask) = flush {
        let found = score_at(HIGH_FLUSH_SCORES, mask as usize);
        if found != NOTHING {
            return found;
        }
    }
    let displacement = score_at(HAND_DISPLACEMENTS, bucket_of(rank_key));
    score_at(HIGH_HAND_SCORES, slot_of(rank_key, displacement))
}

/// An ace-to-five low score from a rank key. Suits never matter to this
/// ranking, so there is no flush to consider.
pub fn low_a5_score_from_parts(rank_key: u32) -> u16 {
    let displacement = score_at(HAND_DISPLACEMENTS, bucket_of(rank_key));
    score_at(LOW_A5_HAND_SCORES, slot_of(rank_key, displacement))
}

/// The five-card high hand: hold'em, Omaha, stud.
pub fn high_score(cards: &[Card]) -> u16 {
    score(cards, HIGH_HAND_SCORES, Some(HIGH_FLUSH_SCORES))
}

/// The high hand read upside down, for deuce-to-seven. The ace is always
/// high, so `A5432` is a bad high-card hand rather than a straight, and
/// straights and flushes count against you.
pub fn deuce_seven_score(cards: &[Card]) -> u16 {
    score(cards, DEUCE_SEVEN_HAND_SCORES, Some(DEUCE_SEVEN_FLUSH_SCORES))
}

/// The high hand over thirty-six cards: a flush beats a full house, and the
/// ace plays low below the six.
pub fn short_deck_score(cards: &[Card]) -> u16 {
    score(cards, SHORT_DECK_HAND_SCORES, Some(SHORT_DECK_FLUSH_SCORES))
}

/// The ace-to-five low. Suits never matter, so there is no flush to check.
pub fn low_a5_score(cards: &[Card]) -> u16 {
    score(cards, LOW_A5_HAND_SCORES, None)
}

impl FastHandRank {
    /// Scores a hand of five to seven cards against the high kernel.
    pub fn evaluate(cards: &[Card]) -> Self {
        Self(high_score(cards))
    }
}

impl PartialOrd for FastHandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FastHandRank {
    /// Better hands compare greater, which means lower scores: the table
    /// numbers hands from the royal flush down.
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Card, Rank, Suit};

    #[test]
    fn test_fast_hand_rank_basic() -> Result<(), crate::error::PokerError> {
        // Test a simple high card hand
        let hand = vec![
            Card::new(Suit::Spade, Rank::Ace),
            Card::new(Suit::Heart, Rank::King),
            Card::new(Suit::Diamond, Rank::Queen),
            Card::new(Suit::Club, Rank::Jack),
            Card::new(Suit::Spade, Rank::Nine),
        ];

        let rank = FastHandRank::evaluate(&hand);
        assert!(rank.0 > 0); // High card should have a high score (worse hand)

        Ok(())
    }
}
