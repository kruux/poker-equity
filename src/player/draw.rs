use crate::{cards::Card, variants::PokerType};

use super::Player;

pub struct DrawPlayer {
    name: String,
    cards: Vec<Card>,
    cards_to_discard: Vec<Card>,
}

impl DrawPlayer {
    pub fn cards_to_discard(&self) -> &[Card] {
        &self.cards_to_discard
    }
}

impl Player for DrawPlayer {
    fn name(&self) -> &str {
        &self.name
    }
    fn cards(&self) -> &[Card] {
        &self.cards
    }
    fn poker_type(&self) -> PokerType {
        PokerType::Draw
    }
}
