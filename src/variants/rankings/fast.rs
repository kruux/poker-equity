use std::cmp::Ordering;

use super::{FLUSH_RANKS, HAND_RANKS};
use crate::cards::Card;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FastHandRank(pub u16);

// Constants for the lookup-based evaluation
pub const RANK_KEYS: [u32; 13] = [
    1,           // 2 (5^0)
    5,           // 3 (5^1)
    25,          // 4 (5^2)
    125,         // 5 (5^3)
    625,         // 6 (5^4)
    3_125,       // 7 (5^5)
    15_625,      // 8 (5^6)
    78_125,      // 9 (5^7)
    390_625,     // T (5^8)
    1_953_125,   // J (5^9)
    9_765_625,   // Q (5^10)
    48_828_125,  // K (5^11)
    244_140_625, // A (5^12)
];

pub const FLUSH_KEYS: [u32; 13] = [
    0x0001, // 2
    0x0002, // 3
    0x0004, // 4
    0x0008, // 5
    0x0010, // 6
    0x0020, // 7
    0x0040, // 8
    0x0080, // 9
    0x0100, // T
    0x0200, // J
    0x0400, // Q
    0x0800, // K
    0x1000, // A
];

impl FastHandRank {
    pub fn evaluate(cards: &[Card]) -> Self {
        // Check for flushes by generating a key for each suit
        let mut suit_keys = [0u32; 4];
        for card in cards {
            let rank_idx = card.rank().to_value() - 2;
            suit_keys[card.suit() as usize] += FLUSH_KEYS[rank_idx as usize];
        }

        // Check if any suit forms a flush
        for key in suit_keys {
            if let Some(score) = FLUSH_RANKS.get(&key) {
                return Self(*score);
            }
        }

        // If no flush, calculate regular key
        let key = cards
            .iter()
            .map(|card| RANK_KEYS[(card.rank().to_value() - 2) as usize])
            .sum::<u32>();

        let score = HAND_RANKS.get(&key).unwrap_or(&u16::MAX);
        Self(*score)
    }
}

impl PartialOrd for FastHandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Lower scores are better hands
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for FastHandRank {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lower scores are better hands
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
