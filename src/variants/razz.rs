use std::cmp::Ordering;

use crate::{
    cards::{Card, Rank},
    variants::{LowHandRank, PokerType, PokerVariant},
};

#[derive(Debug, Clone, Copy)]
pub struct Razz;

impl PokerVariant for Razz {
    type HandRank = LowHandRank;

    fn poker_type(&self) -> PokerType {
        PokerType::Stud
    }

    fn max_cards(&self) -> usize {
        7
    }

    fn to_string(&self) -> String {
        "Razz".to_string()
    }

    fn sorted_ranks(&self, cards: &[Card]) -> Vec<Rank> {
        // Get the lowest 5 ranks. Sorted in ascending order.
        let mut ranks = cards.iter().map(|c| c.rank()).collect::<Vec<Rank>>();
        ranks.sort_by(|a, b| {
            // Ace is the lowest rank so it should be first
            if *a == Rank::Ace {
                return Ordering::Less;
            } else if *b == Rank::Ace {
                return Ordering::Greater;
            } else {
                a.cmp(b)
            }
        });
        ranks.dedup(); // Duplicates are not counted in razz
        ranks
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        let mut unique_ranks = self.sorted_ranks(cards);
        unique_ranks.truncate(5);
        unique_ranks.reverse(); // Once we picked the lowest 5 ranks, we need to reverse them to get them in descending order
        LowHandRank::Low(unique_ranks)
    }
}
