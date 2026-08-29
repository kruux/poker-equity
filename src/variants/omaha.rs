use std::cmp::{min, Ordering};

use itertools::Itertools;

use crate::cards::Card;

use super::{rankings::OmahaHandRank, PokerType, PokerVariant};

#[derive(Debug, Clone, Copy)]
pub struct Omaha;

impl PokerVariant for Omaha {
    type HandRank = OmahaHandRank;

    fn poker_type(&self) -> PokerType {
        PokerType::Community
    }

    fn max_cards(&self) -> usize {
        9
    }

    fn hole_cards(&self) -> usize {
        4
    }

    fn board_cards(&self) -> usize {
        5
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        // PLO get's slightly tricky. Need to compare every 2 card combination from hand with every 3 card combination from the board
        let hole_cards = &cards[0..cards.len().min(4)];
        let board_cards = if cards.len() > 4 { &cards[4..] } else { &[] };

        let hole_combinations: Vec<_> = hole_cards.iter().combinations(2).collect();
        let board_combinations: Vec<_> = board_cards.iter().combinations(3).collect();

        // Check every possible evaluation from the combinations
        hole_combinations
            .iter()
            .flat_map(|hole_combo| {
                board_combinations.iter().map(move |board_combo| {
                    let hand_combo = hole_combo
                        .iter()
                        .chain(board_combo.iter())
                        .map(|&card| *card)
                        .collect::<Vec<Card>>();
                    OmahaHandRank::evaluate(&hand_combo)
                })
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap_or_else(|| {
                // If no valid hand is found, return an incomplete hand with the number of cards in the best hand
                let num_cards = min(hole_cards.len(), 2) + min(board_cards.len(), 3);
                OmahaHandRank::Incomplete(num_cards)
            })
    }

    fn to_string(&self) -> String {
        "Omaha".to_string()
    }
}
