use crate::cards::Card;

use super::{rankings::LowHandRank, PokerType, PokerVariant};

/// The best badugi in a holding.
///
/// A badugi is a set of cards with no two sharing a rank and no two sharing a
/// suit. The largest such set plays, so a four-card badugi beats any
/// three-card one however low; within a size the cards are compared from the
/// top down and lower wins, with the ace low. The best hand is a rainbow
/// A-2-3-4.
///
/// With four cards there are fifteen subsets to consider, so they are all
/// tried rather than reasoned about.
fn best_badugi(cards: &[Card]) -> LowHandRank {
    let mut best: Option<LowHandRank> = None;

    for subset in 1..(1u32 << cards.len()) {
        let chosen: Vec<Card> = cards
            .iter()
            .enumerate()
            .filter(|(index, _)| subset & (1 << index) != 0)
            .map(|(_, card)| *card)
            .collect();

        if !all_distinct(&chosen) {
            continue;
        }

        let rank = LowHandRank::from_played(chosen.iter().map(|card| card.rank()).collect());
        if best.as_ref().is_none_or(|found| rank > *found) {
            best = Some(rank);
        }
    }

    best.unwrap_or_else(|| LowHandRank::Low(Vec::new()))
}

/// Whether no two cards share a rank and no two share a suit.
fn all_distinct(cards: &[Card]) -> bool {
    for (index, card) in cards.iter().enumerate() {
        for other in &cards[index + 1..] {
            if card.rank() == other.rank() || card.suit() == other.suit() {
                return false;
            }
        }
    }
    true
}

/// Four cards, drawing for the lowest set of cards with no repeated rank and
/// no repeated suit.
#[derive(Debug, Clone, Copy)]
pub struct Badugi;

impl PokerVariant for Badugi {
    type HandRank = LowHandRank;

    fn poker_type(&self) -> PokerType {
        PokerType::Draw
    }

    fn max_cards(&self) -> usize {
        4
    }

    fn hole_cards(&self) -> usize {
        4
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        best_badugi(cards)
    }

    fn to_string(&self) -> String {
        "Badugi".to_string()
    }

    fn key(&self) -> &'static str {
        "badugi"
    }
}
