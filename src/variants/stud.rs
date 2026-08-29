use crate::{
    cards::Card,
    variants::{PokerType, PokerVariant},
};

use super::{rankings::high_score, StudHandRank};

#[derive(Clone, Copy, Debug)]
/// Seven private cards, no board, the best five playing.
pub struct SevenCardStud;

impl PokerVariant for SevenCardStud {
    type HandRank = StudHandRank;

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
        "Stud".to_string()
    }

    fn key(&self) -> &'static str {
        "stud"
    }

    fn score(&self, cards: &[Card]) -> u32 {
        high_score(cards) as u32
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        StudHandRank::evaluate(cards)
    }
}
