use super::{PokerType, PokerVariant};

pub trait StudGame: PokerVariant {
    fn up_cards(&self) -> u32 {
        4
    }
    fn down_cards(&self) -> u32 {
        3
    }
    fn poker_type(&self) -> PokerType {
        PokerType::Stud
    }
}
