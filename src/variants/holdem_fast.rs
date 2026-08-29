use crate::cards::Card;

use super::{rankings::FastHandRank, PokerType, PokerVariant};

#[derive(Debug, Clone, Copy)]
pub struct HoldemFast;

impl PokerVariant for HoldemFast {
    type HandRank = FastHandRank;

    fn poker_type(&self) -> PokerType {
        PokerType::Community
    }

    fn max_cards(&self) -> usize {
        7
    }

    fn hole_cards(&self) -> usize {
        2
    }

    fn board_cards(&self) -> usize {
        5
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        FastHandRank::evaluate(cards)
    }

    fn to_string(&self) -> String {
        "Hold'em Fast".to_string()
    }

    fn key(&self) -> &'static str {
        "holdem"
    }
}
