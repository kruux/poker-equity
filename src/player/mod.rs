use crate::{cards::Card, variants::PokerType};

mod draw;

pub use draw::DrawPlayer;

pub trait Player {
    fn name(&self) -> &str;
    fn cards(&self) -> &[Card];
    fn poker_type(&self) -> PokerType;
}
