use std::{cmp::Ordering, collections::HashMap, fmt, mem::discriminant};

use crate::cards::{Card, Rank, Suit};

/// Used for many of the usual game types, like hold em and stud
#[derive(Debug)]
pub enum HighHandRank {
    StraightFlush(Rank),
    FourOfAKind(Rank, Rank),
    FullHouse(Rank, Rank), // Higher trips/pair is better
    Flush(Vec<Rank>),
    Straight(Rank),
    ThreeOfAKind(Rank, Vec<Rank>),
    TwoPair(Rank, Rank, Rank),
    Pair(Rank, Vec<Rank>),
    HighCard(Vec<Rank>),
}

impl HighHandRank {
    pub fn evaluate(cards: &[Card]) -> Self {
        let rank_counts = Self::rank_counts(cards);

        if let Some((rank, _)) = Self::is_straight_flush(cards) {
            Self::StraightFlush(rank)
        } else if let Some((quad_rank, kicker)) = Self::is_four_of_kind(&rank_counts) {
            Self::FourOfAKind(quad_rank, kicker)
        } else if let Some((trips, pair)) = Self::is_full_house(&rank_counts) {
            Self::FullHouse(trips, pair)
        } else if let Some(ranks) = Self::is_flush(cards) {
            Self::Flush(ranks)
        } else if let Some(rank) = Self::is_straight(cards) {
            Self::Straight(rank)
        } else if let Some((trips_rank, kickers)) = Self::is_three_of_kind(&rank_counts) {
            Self::ThreeOfAKind(trips_rank, kickers)
        } else if let Some((high_pair, low_pair, kicker)) = Self::is_two_pair(&rank_counts) {
            Self::TwoPair(high_pair, low_pair, kicker)
        } else if let Some((pair_rank, kickers)) = Self::is_pair(&rank_counts) {
            Self::Pair(pair_rank, kickers)
        } else {
            let sorted = Self::sorted_ranks(cards);
            Self::HighCard(Self::first_n_ranks(&sorted, 5))
        }
    }

    fn is_straight_flush(cards: &[Card]) -> Option<(Rank, Suit)> {
        let cards_by_suit = Self::sorted_suits(cards);

        cards_by_suit
            .iter()
            .filter(|(_, cards)| cards.len() >= 5)
            .find_map(|(&suit, cards)| Self::is_straight(cards).map(|rank| (rank, suit)))
    }

    fn is_four_of_kind(rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, Rank)> {
        // Check for quads
        if let Some((&quad_rank, _)) = rank_counts.iter().find(|&(_, &count)| count == 4) {
            // Find the highest kicker among the non-quad cards
            let kicker = rank_counts
                .iter()
                .filter(|(&rank, _)| rank != quad_rank)
                .max_by_key(|(&rank, _)| rank.to_value())?
                .0;

            return Some((quad_rank, *kicker));
        }
        None
    }

    fn is_full_house(rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, Rank)> {
        let best_trips = rank_counts
            .iter()
            .filter(|&(_, &count)| count == 3)
            .max_by_key(|(rank, _)| *rank)
            .map(|(rank, _)| *rank)?;

        let best_pair = rank_counts
            .iter()
            .filter(|&(rank, &count)| *rank != best_trips && count >= 2)
            .max_by_key(|(rank, _)| *rank)
            .map(|(rank, _)| *rank)?;

        Some((best_trips, best_pair))
    }

    fn is_flush(cards: &[Card]) -> Option<Vec<Rank>> {
        let cards_by_suit = Self::sorted_suits(cards);

        cards_by_suit
            .iter()
            .filter(|(_, cards)| cards.len() >= 5)
            .map(|(_, cards)| {
                let sorted = Self::sorted_ranks(cards);
                Self::first_n_ranks(&sorted, 5)
            })
            .next()
    }

    fn is_straight(cards: &[Card]) -> Option<Rank> {
        let ranks = Self::sorted_ranks(cards);
        if ranks.len() < 5 {
            return None;
        }

        let mut rank_values: Vec<u8> = ranks.iter().map(|r| r.to_value()).collect();
        rank_values.dedup();

        if ranks[0] == Rank::Ace {
            rank_values.push(1);
        }

        let mut consecutive = 0;
        for i in 0..rank_values.len() - 1 {
            if rank_values[i] == rank_values[i + 1] + 1 {
                consecutive += 1;
            } else {
                consecutive = 0;
            }
            if consecutive >= 4 {
                return Some(ranks[i - 3]);
            }
        }
        None
    }

    fn is_three_of_kind(rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, Vec<Rank>)> {
        // Check for trips
        if let Some((&trips_rank, _)) = rank_counts.iter().find(|&(_, &count)| count == 3) {
            // Get all non-trips ranks and take the highest two as kickers
            let mut kickers: Vec<Rank> = rank_counts
                .iter()
                .filter(|(&rank, &count)| count == 1 && rank != trips_rank)
                .map(|(&rank, _)| rank)
                .collect();

            // Sort by value descending
            kickers.sort_by_key(|rank| std::cmp::Reverse(rank.to_value()));
            // Take only the highest two kickers
            kickers.truncate(2);

            return Some((trips_rank, kickers));
        }
        None
    }

    fn is_two_pair(rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, Rank, Rank)> {
        let mut pairs: Vec<Rank> = rank_counts
            .iter()
            .filter(|&(_, &count)| count == 2)
            .map(|(rank, _)| *rank)
            .collect();

        if pairs.len() < 2 {
            return None;
        }

        pairs.sort_by(|a, b| b.cmp(a));
        let high_pair = pairs[0];
        let low_pair = pairs[1];

        let kicker = rank_counts
            .iter()
            .filter(|&(rank, _)| *rank != high_pair && *rank != low_pair)
            .map(|(rank, _)| *rank)
            .max_by(|a, b| a.cmp(b))?;

        Some((high_pair, low_pair, kicker))
    }

    fn is_pair(rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, Vec<Rank>)> {
        if let Some((&pair_rank, _)) = rank_counts.iter().find(|&(_, &count)| count == 2) {
            let mut kickers: Vec<Rank> = rank_counts
                .iter()
                .filter(|&(_, &count)| count == 1)
                .map(|(rank, _)| *rank)
                .collect();
            kickers.sort_by(|a, b| b.cmp(a));
            let best_kickers = Self::first_n_ranks(&kickers, 3);
            return Some((pair_rank, best_kickers));
        }
        None
    }

    fn rank_counts(cards: &[Card]) -> HashMap<Rank, usize> {
        let mut counts = HashMap::new();
        for card in cards {
            *counts.entry(card.rank()).or_insert(0) += 1;
        }
        counts
    }

    fn sorted_suits(cards: &[Card]) -> HashMap<Suit, Vec<Card>> {
        let mut cards_by_suit: HashMap<Suit, Vec<Card>> = HashMap::new();
        for &card in cards {
            cards_by_suit
                .entry(card.suit())
                .or_insert_with(Vec::new)
                .push(card);
        }
        cards_by_suit
    }

    fn sorted_ranks(cards: &[Card]) -> Vec<Rank> {
        let mut ranks: Vec<Rank> = cards.iter().map(|card| card.rank()).collect();
        ranks.sort_by(|a, b| b.cmp(a)); // Sort descending
        ranks
    }

    fn first_n_ranks(ranks: &[Rank], n: usize) -> Vec<Rank> {
        ranks.iter().take(n).cloned().collect()
    }
}

impl PartialOrd for HighHandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // Non matching hands. E.g. pair vs trips
            (x, y) if discriminant(x) != discriminant(y) => {
                // Standard order (higher hand types are better)
                Some(self.hand_type_value().cmp(&other.hand_type_value()))
            }

            // Matching hand types
            (HighHandRank::StraightFlush(r1), HighHandRank::StraightFlush(r2)) => {
                r2.partial_cmp(r1)
            }
            (HighHandRank::FourOfAKind(r1, k1), HighHandRank::FourOfAKind(r2, k2)) => {
                match r2.partial_cmp(r1) {
                    Some(Ordering::Equal) => k2.partial_cmp(k1),
                    ord => ord,
                }
            }
            (HighHandRank::FullHouse(t1, p1), HighHandRank::FullHouse(t2, p2)) => {
                match t2.partial_cmp(t1) {
                    Some(Ordering::Equal) => p2.partial_cmp(p1),
                    ord => ord,
                }
            }
            (HighHandRank::Flush(ranks1), HighHandRank::Flush(ranks2)) => {
                for (a, b) in ranks1.iter().zip(ranks2.iter()) {
                    match b.partial_cmp(a) {
                        Some(Ordering::Equal) => continue,
                        ord => return ord,
                    }
                }
                Some(Ordering::Equal)
            }
            (HighHandRank::Straight(r1), HighHandRank::Straight(r2)) => r2.partial_cmp(r1),
            (HighHandRank::ThreeOfAKind(t1, k1), HighHandRank::ThreeOfAKind(t2, k2)) => {
                match t2.partial_cmp(t1) {
                    Some(Ordering::Equal) => {
                        for (r1, r2) in k1.iter().zip(k2.iter()) {
                            match r2.partial_cmp(r1) {
                                Some(Ordering::Equal) => continue,
                                ord => return ord,
                            }
                        }
                        Some(Ordering::Equal)
                    }
                    ord => ord,
                }
            }
            (HighHandRank::TwoPair(h1, l1, k1), HighHandRank::TwoPair(h2, l2, k2)) => {
                match h2.partial_cmp(h1) {
                    Some(Ordering::Equal) => match l2.partial_cmp(l1) {
                        Some(Ordering::Equal) => k2.partial_cmp(k1),
                        ord => ord,
                    },
                    ord => ord,
                }
            }
            (HighHandRank::Pair(r1, k1), HighHandRank::Pair(r2, k2)) => match r2.partial_cmp(r1) {
                Some(Ordering::Equal) => {
                    for (a, b) in k1.iter().zip(k2.iter()) {
                        match b.partial_cmp(a) {
                            Some(Ordering::Equal) => continue,
                            ord => return ord,
                        }
                    }
                    Some(Ordering::Equal)
                }
                ord => ord,
            },
            (HighHandRank::HighCard(ranks1), HighHandRank::HighCard(ranks2)) => {
                for (a, b) in ranks1.iter().zip(ranks2.iter()) {
                    match b.partial_cmp(a) {
                        Some(Ordering::Equal) => continue,
                        ord => return ord,
                    }
                }
                Some(Ordering::Equal)
            }
            _ => None, // Should never occur since all cases are covered
        }
    }
}

impl HighHandRank {
    fn hand_type_value(&self) -> u8 {
        match self {
            HighHandRank::StraightFlush(_) => 9,
            HighHandRank::FourOfAKind(_, _) => 8,
            HighHandRank::FullHouse(_, _) => 7,
            HighHandRank::Flush(_) => 6,
            HighHandRank::Straight(_) => 5,
            HighHandRank::ThreeOfAKind(_, _) => 4,
            HighHandRank::TwoPair(_, _, _) => 3,
            HighHandRank::Pair(_, _) => 2,
            HighHandRank::HighCard(_) => 1,
        }
    }
}

impl fmt::Display for HighHandRank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HighHandRank::StraightFlush(r) => write!(f, "Straight Flush, {} high", r),
            HighHandRank::FourOfAKind(r, _) => write!(f, "Four of a Kind, {}s", r),
            HighHandRank::FullHouse(t, p) => write!(f, "Full House, {}s full of {}s", t, p),
            HighHandRank::Flush(ranks) => write!(f, "Flush, {} high", ranks[0]),
            HighHandRank::Straight(r) => write!(f, "Straight, {} high", r),
            HighHandRank::ThreeOfAKind(r, _) => write!(f, "Three of a Kind, {}s", r),
            HighHandRank::TwoPair(h, l, k) => {
                write!(f, "Two Pair, {}s and {}s with {} kicker", h, l, k)
            }
            HighHandRank::Pair(r, _) => write!(f, "Pair of {}s", r),
            HighHandRank::HighCard(ranks) => write!(f, "High Card {}", ranks[0]),
        }
    }
}
