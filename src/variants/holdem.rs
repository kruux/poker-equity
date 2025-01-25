use super::{rankings::HoldemHandRank, PokerType, PokerVariant};

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

    fn evaluate_hand(&self, cards: &[crate::cards::Card]) -> Self::HandRank {
        HoldemHandRank::evaluate(cards)
    }

    fn to_string(&self) -> String {
        "Hold 'em".to_string()
    }
}
