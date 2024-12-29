use super::{PokerType, PokerVariant};

pub struct Omaha;

impl PokerVariant for Omaha {
    fn poker_type(&self) -> PokerType {
        PokerType::Omaha
    }
}
