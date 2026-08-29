use crate::cards::{Card, CardSet};

use super::{
    rankings::{short_deck_score, HighHandRank, ShortDeckRank},
    PokerType, PokerVariant,
};

/// Hold'em over thirty-six cards, sixes and up.
///
/// A flush beats a full house, because a thirty-six card deck makes flushes
/// the scarcer hand. The ace plays low below the six, so `A-6-7-8-9` is the
/// low straight and the lowest straight flush is nine high. Everything else
/// keeps its usual place.
///
/// This is the PokerStars ordering. Rooms differ, which is why the ruleset is
/// named in the label rather than assumed.
#[derive(Debug, Clone, Copy)]
pub struct ShortDeck;

impl PokerVariant for ShortDeck {
    type HandRank = ShortDeckRank;

    fn poker_type(&self) -> PokerType {
        PokerType::Community
    }

    fn deck(&self) -> CardSet {
        CardSet::SHORT_DECK
    }

    fn max_cards(&self) -> usize {
        7
    }

    fn hole_cards(&self) -> usize {
        2
    }

    fn board_cards(&self) -> usize {
        5
    }

    fn score(&self, cards: &[Card]) -> u32 {
        short_deck_score(cards) as u32
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        ShortDeckRank(HighHandRank::evaluate_short_deck(cards))
    }

    fn to_string(&self) -> String {
        "Short Deck (PokerStars: flush over full house)".to_string()
    }

    fn key(&self) -> &'static str {
        "holdem_short_deck"
    }
}
