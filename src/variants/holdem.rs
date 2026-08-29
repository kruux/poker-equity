use crate::cards::Card;

use super::{rankings::high_score, rankings::HoldemHandRank, PokerType, PokerVariant};

#[derive(Debug, Clone, Copy)]
pub struct Holdem;

impl PokerVariant for Holdem {
    type HandRank = HoldemHandRank;

    fn poker_type(&self) -> super::PokerType {
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

    fn score(&self, cards: &[Card]) -> u32 {
        high_score(cards) as u32
    }

    fn evaluate_hand(&self, cards: &[crate::cards::Card]) -> Self::HandRank {
        HoldemHandRank::evaluate(cards)
    }

    fn to_string(&self) -> String {
        "Hold 'em".to_string()
    }

    fn key(&self) -> &'static str {
        "holdem"
    }
}
