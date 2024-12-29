use super::{PokerType, PokerVariant};

pub struct TexasHoldem;

impl PokerVariant for TexasHoldem {
    fn poker_type(&self) -> PokerType {
        PokerType::Holdem
    }
}
