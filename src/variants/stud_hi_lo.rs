use super::{HighHandRank, LowHandRank, StudHiLoHandRank};
use crate::{
    cards::{Card, Rank},
    variants::{PokerType, PokerVariant},
};

#[derive(Debug, Clone, Copy)]
pub struct StudHiLo;
impl StudHiLo {
    /// Returns an Option<LowHandRank> if the hand qualifies for a low.
    /// Returns None if the low hand doesn't have 5 cards 8 or lower.
    fn qualify_for_low(&self, low_rank: &LowHandRank) -> Option<LowHandRank> {
        // Need at least 5 cards
        let ranks = low_rank.ranks();
        if ranks.len() < 5 {
            return None;
        }

        // Check if highest card is 8 or lower
        if let Some(highest_low) = ranks.get(0) {
            if highest_low.to_value() <= Rank::Eight.to_value() || *highest_low == Rank::Ace {
                return Some(low_rank.clone());
            }
        }

        None
    }
}

impl PokerVariant for StudHiLo {
    type HandRank = StudHiLoHandRank;

    fn poker_type(&self) -> PokerType {
        PokerType::Stud
    }

    fn max_cards(&self) -> usize {
        7
    }

    fn to_string(&self) -> String {
        "Stud Hi/Lo".to_string()
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        let high = HighHandRank::evaluate(cards);
        let low = self.qualify_for_low(&LowHandRank::evaluate(cards));

        StudHiLoHandRank { high, low }
    }
}
