use crate::{
    cards::Card,
    variants::{PokerType, PokerVariant},
};

use super::{rankings::high_score, HighHandRank};

#[derive(Clone, Copy, Debug)]
/// Five cards, drawing once, where the best high hand wins.
///
/// Ranked exactly as hold'em ranks its best five, so it shares the high
/// table. The game has one draw by its own rules, so the single draw modelled
/// here is the whole game rather than a simplification of it.
pub struct FiveCardDraw;

impl PokerVariant for FiveCardDraw {
    type HandRank = HighHandRank;

    /// A draw game: each seat's missing cards are dealt from the deck.
    fn poker_type(&self) -> PokerType {
        PokerType::Draw
    }

    /// Five cards, all of them private.
    fn max_cards(&self) -> usize {
        5
    }

    /// Five cards, all of them private.
    fn hole_cards(&self) -> usize {
        5
    }

    /// The label a person reads.
    fn to_string(&self) -> String {
        "5-Card Draw".to_string()
    }

    /// The key the game is stored and configured under.
    fn key(&self) -> &'static str {
        "five_card_draw"
    }

    /// The high table's score, lower being better.
    fn score(&self, cards: &[Card]) -> u32 {
        high_score(cards) as u32
    }

    /// Names the high hand, or an incomplete rank while cards are missing.
    fn evaluate_hand(&self, cards: &[Card]) -> HighHandRank {
        HighHandRank::evaluate(cards)
    }
}
