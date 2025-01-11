mod deuce_seven;
mod equity;
mod rankings;
// mod razz;
mod stud;

pub use deuce_seven::DeuceSeven;
pub use equity::EquityCalculation;
pub use rankings::{DeuceSevenRank, HighHandRank};
// pub use razz::Razz;
pub use stud::SevenCardStud;

use std::collections::HashMap;

use crate::cards::{Card, Rank, Suit};

pub enum PokerType {
    Draw, // 5 card draw
    Stud, // Stud games
}

pub trait PokerVariant: Clone + Copy {
    type HandRank: PartialOrd;

    fn poker_type(&self) -> PokerType;
    fn max_cards(&self) -> usize;
    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank;
    fn to_string(&self) -> String;
    fn sorted_ranks(&self, cards: &[Card]) -> Vec<Rank>;

    fn is_straight_flush(&self, cards: &[Card]) -> Option<(Rank, Suit)> {
        // Sort the cards by suit and check each suit for a straight
        let cards_by_suit = self.sorted_suits(cards);

        // Check each suit that qualify as flush for a straight
        cards_by_suit
            .iter()
            .filter(|(_, cards)| cards.len() >= 5) // Filter out non flush
            .find_map(|(&suit, cards)| self.is_straight(cards).map(|rank| (rank, suit)))
    }

    fn is_four_of_kind(&self, rank_counts: &HashMap<Rank, usize>) -> Option<Rank> {
        rank_counts
            .iter()
            .filter(|&(_, &count)| count == 4)
            .max_by_key(|(rank, _)| *rank)
            .map(|(rank, _)| *rank)
    }

    fn is_full_house(&self, rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, Rank)> {
        // First check trips.
        let best_trips = rank_counts
            .iter()
            .filter(|&(_, &count)| count == 3)
            .max_by_key(|(rank, _)| *rank)
            .map(|(rank, _)| *rank)?;

        // Find best pair. exclude best trips. But use >= 2. Could be AAAKKKQ
        let best_pair = rank_counts
            .iter()
            .filter(|&(rank, &count)| *rank != best_trips && count >= 2)
            .max_by_key(|(rank, _)| *rank)
            .map(|(rank, _)| *rank)?;

        Some((best_trips, best_pair))
    }

    fn is_flush(&self, cards: &[Card]) -> Option<Vec<Rank>> {
        let cards_by_suit = self.sorted_suits(cards);

        cards_by_suit
            .iter()
            .filter(|(_, cards)| cards.len() >= 5) // Filter out non flush
            .map(|(_, cards)| {
                let sorted = self.sorted_ranks(cards);
                self.first_n_ranks(&sorted, 5)
            })
            .next()
    }

    fn is_straight(&self, cards: &[Card]) -> Option<Rank> {
        let ranks = self.sorted_ranks(cards);
        if ranks.len() < 5 {
            return None;
        }

        // Convert to u8 for easier comparison
        let mut rank_values: Vec<u8> = ranks.iter().map(|r| r.to_value()).collect();

        // Remove duplicates so it's easier to check for straights
        rank_values.dedup();

        // If we have an ace. Add a 1 to the end to make A2345 count as a straight
        if ranks[0] == Rank::Ace {
            rank_values.push(1);
        }

        // Cards are sorted decrementally. E.g. KQJT9
        // so the next card should always be one lower
        let mut consecutive = 0;
        for i in 0..rank_values.len() - 1 {
            if rank_values[i] == rank_values[i + 1] + 1 {
                consecutive += 1;
            } else {
                consecutive = 0;
            }
            if consecutive >= 4 {
                // If 4 consecutive checks complete we have a 5 card straight
                // Back up 3 steps to get the highest card
                return Some(ranks[i - 3]);
            }
        }
        None
    }

    fn is_three_of_kind(&self, rank_counts: &HashMap<Rank, usize>) -> Option<Rank> {
        rank_counts
            .iter()
            .filter(|&(_, &count)| count >= 3)
            .max_by_key(|(rank, _)| *rank)
            .map(|(rank, _)| *rank)
    }

    /// Checks for two pair. Returns (high_pair, low_pair, kicker) if it finds two pair.
    /// Otherwise None
    fn is_two_pair(&self, rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, Rank, Rank)> {
        // This function needs to be reworked in case of 7 card games
        let mut pairs: Vec<Rank> = rank_counts
            .iter()
            .filter(|&(_, &count)| count == 2)
            .map(|(rank, _)| *rank)
            .collect();
        if pairs.len() < 2 {
            return None;
        }

        // Sort pairs by rank. Might be 3 pair
        pairs.sort_by(|a, b| b.cmp(a));
        let high_pair = pairs[0];
        let low_pair = pairs[1];

        // Find kicker
        let kicker = rank_counts
            .iter()
            .filter(|&(rank, _)| *rank != high_pair && *rank != low_pair)
            .map(|(rank, _)| *rank)
            .max_by(|a, b| a.cmp(b))?;

        Some((high_pair, low_pair, kicker))
    }

    fn is_pair(&self, rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, Vec<Rank>)> {
        if let Some((&pair_rank, _)) = rank_counts.iter().find(|&(_, &count)| count == 2) {
            let mut kickers: Vec<Rank> = rank_counts
                .iter()
                .filter(|&(_, &count)| count == 1)
                .map(|(rank, _)| *rank)
                .collect();
            kickers.sort_by(|a, b| b.cmp(a));
            // Limit kickers to three cards
            let best_kickers = self.first_n_ranks(&kickers, 3);
            return Some((pair_rank, best_kickers));
        }
        None
    }

    fn rank_counts(&self, cards: &[Card]) -> HashMap<Rank, usize> {
        let mut counts = HashMap::new();
        for card in cards {
            *counts.entry(card.rank()).or_insert(0) += 1;
        }
        counts
    }

    fn sorted_suits(&self, cards: &[Card]) -> HashMap<Suit, Vec<Card>> {
        let mut cards_by_suit: HashMap<Suit, Vec<Card>> = HashMap::new();

        for &card in cards {
            cards_by_suit
                .entry(card.suit())
                .or_insert_with(Vec::new)
                .push(card);
        }

        cards_by_suit
    }

    fn suit_counts(&self, cards: &[Card]) -> HashMap<Suit, usize> {
        let mut counts = HashMap::new();
        for card in cards {
            *counts.entry(card.suit()).or_insert(0) += 1;
        }
        counts
    }

    fn first_n_ranks(&self, ranks: &[Rank], n: usize) -> Vec<Rank> {
        ranks.iter().take(n).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    mod deuce_seven_tests;
    // mod razz_tests;
    mod stud_tests;
}
