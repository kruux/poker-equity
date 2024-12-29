use super::{PokerType, PokerVariant};

pub struct Draw;

pub trait Draw: PokerVariant {
    fn poker_type(&self) -> PokerType {
        PokerType::Draw
    }

    fn valid_hand_size(&self) -> u32 {
        5 // Default handsize 5. Cant think of a draw game where you have more right now
    }

    // fn evaluate_hand(&self, cards: &[Card]) -> HandRank
}
