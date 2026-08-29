use std::cmp::Ordering;

use itertools::Itertools;

use crate::cards::Card;

use super::{rankings::FastHandRank, PokerType, PokerVariant};

#[derive(Debug, Clone, Copy)]
pub struct OmahaFast;

impl PokerVariant for OmahaFast {
    type HandRank = FastHandRank;

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
        let hole_cards = cards.get(0..4).unwrap_or(&[]);
        let board_cards = cards.get(4..).unwrap_or(&[]);

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
                    FastHandRank::evaluate(&hand_combo)
                })
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap_or(FastHandRank(u16::MAX))
    }

    fn to_string(&self) -> String {
        "Omaha Fast".to_string()
    }

    fn key(&self) -> &'static str {
        "omaha"
    }
}
