use std::cmp::Ordering;

use super::table_index::{bucket_of, slot_of, BUCKET_COUNT, SLOT_COUNT};
use super::{FLUSH_SCORES, HAND_DISPLACEMENTS, HAND_SCORES};
use crate::cards::Card;

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
const _: () = assert!(HAND_SCORES.len() == SLOT_COUNT * 2);
const _: () = assert!(HAND_DISPLACEMENTS.len() == BUCKET_COUNT * 2);
const _: () = assert!(FLUSH_SCORES.len() == (1 << 13) * 2);

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

impl FastHandRank {
    /// Scores a hand of five to seven cards.
    ///
    /// Shorter or longer hands cannot be scored and come back as `NOTHING`.
    /// The guard is not just tidiness: the rank table holds nothing but
    /// scores, so a key that is not a hand would land on some other hand's
    /// slot and read it as its own.
    pub fn evaluate(cards: &[Card]) -> Self {
        if !(5..=7).contains(&cards.len()) {
            return Self(NOTHING);
        }

        // A flush needs five cards of one suit, so each suit's ranks are
        // gathered into a thirteen-bit mask. Masks with fewer than five bits
        // are marked as nothing in the table, so the miss costs one read.
        let mut suits = [0u32; 4];
        for card in cards {
            let rank = (card.rank().to_value() - 2) as usize;
            suits[card.suit() as usize] |= FLUSH_KEYS[rank];
        }
        for suit in suits {
            let score = score_at(FLUSH_SCORES, suit as usize);
            if score != NOTHING {
                return Self(score);
            }
        }

        // No flush, so the hand is worth whatever its ranks are worth. The
        // key is far too sparse to index directly, so it goes through the
        // displacement in `table_index`.
        let key: u32 = cards
            .iter()
            .map(|card| RANK_KEYS[(card.rank().to_value() - 2) as usize])
            .sum();
        let displacement = score_at(HAND_DISPLACEMENTS, bucket_of(key));
        Self(score_at(HAND_SCORES, slot_of(key, displacement)))
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
