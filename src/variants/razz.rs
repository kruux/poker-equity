use crate::{
    cards::Card,
    variants::{rankings::low_a5_score, PokerType, PokerVariant, RazzHandRank},
};

#[derive(Debug, Clone, Copy)]
/// Seven-card stud played for low, ace to five, with no qualifier.
///
/// Straights and flushes do not count and the ace is the lowest card, so the
/// best hand is `5-4-3-2-A`. Pairing hurts categorically: any hand with no
/// pair beats any hand with one.
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

    fn score(&self, cards: &[Card]) -> u32 {
        low_a5_score(cards) as u32
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        RazzHandRank::evaluate(cards)
    }
}
