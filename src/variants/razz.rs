use crate::{
    cards::Card,
    variants::{PokerType, PokerVariant, RazzHandRank},
};

#[derive(Debug, Clone, Copy)]
pub struct Razz;

impl PokerVariant for Razz {
    type HandRank = RazzHandRank;

    fn poker_type(&self) -> PokerType {
        PokerType::Stud
    }

    fn max_cards(&self) -> usize {
        7
    }

    fn hole_cards(&self) -> usize {
        7
    }

    fn to_string(&self) -> String {
        "Razz".to_string()
    }

    fn key(&self) -> &'static str {
        "razz"
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        RazzHandRank::evaluate(cards)
    }
}
