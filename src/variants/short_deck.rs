use crate::cards::{Card, CardSet};

use super::{rankings::{HighHandRank, ShortDeckRank}, PokerType, PokerVariant};

/// Hold'em over thirty-six cards, sixes and up.
///
/// Two rankings move, because the shorter deck changes how often each hand
/// comes up: a flush beats a full house, and trips beat a straight. The ace
/// plays low below the six, so `A-6-7-8-9` is the low straight.
///
/// Rooms differ on the details, which is why the ruleset is named here rather
/// than assumed.
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

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        ShortDeckRank(HighHandRank::evaluate_short_deck(cards))
    }

    fn to_string(&self) -> String {
        "Short Deck (PokerStars/GG: flush over full house, trips over straight)".to_string()
    }

    fn key(&self) -> &'static str {
        "6_holdem"
    }
}
