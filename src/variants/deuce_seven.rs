use crate::{
    cards::Card,
    variants::DeuceSevenRank,
    variants::{PokerType, PokerVariant},
};

#[derive(Clone, Copy, Debug)]
pub struct DeuceSeven;

impl DeuceSeven {}

impl PokerVariant for DeuceSeven {
    type HandRank = DeuceSevenRank;

    fn poker_type(&self) -> PokerType {
        PokerType::Draw
    }

    fn max_cards(&self) -> usize {
        5
    }

    fn hole_cards(&self) -> usize {
        5
    }

    fn to_string(&self) -> String {
        "2-7 single draw".to_string()
    }

    fn key(&self) -> &'static str {
        "deuce_seven"
    }

    fn evaluate_hand(&self, cards: &[Card]) -> DeuceSevenRank {
        DeuceSevenRank::evaluate(cards)
    }
}
