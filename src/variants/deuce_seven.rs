use crate::{
    cards::Card,
    variants::DeuceSevenRank,
    variants::{rankings::deuce_seven_score, PokerType, PokerVariant},
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

    fn score(&self, cards: &[Card]) -> u32 {
        deuce_seven_score(cards) as u32
    }

    fn evaluate_hand(&self, cards: &[Card]) -> DeuceSevenRank {
        DeuceSevenRank::evaluate(cards)
    }
}
