use std::{cmp::Ordering, fmt, mem::discriminant};

use crate::cards::Rank;

#[derive(Debug)]
pub enum DeuceSevenRank {
    StraightFlush(Rank),       // Rank of highest card
    FourOfAKind(Rank),         // Rank of quads
    FullHouse(Rank, Rank),     // Rank of trips then pair
    Flush(Vec<Rank>),          // Vec of rank in descending order
    Straight(Rank),            // Highest card
    ThreeOfAKind(Rank),        // Rank of trips
    TwoPair(Rank, Rank, Rank), // High pair, low pair, kicker
    Pair(Rank, Vec<Rank>),     // Rank of pair, vec of kickers in descending order
    HighCard(Vec<Rank>),       // High to low
}

impl PartialOrd for DeuceSevenRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // Non matching hands. E.g. pair vs trips
            (x, y) if discriminant(x) != discriminant(y) => {
                // Reverse order of hands in 2-7
                Some(other.hand_type_value().cmp(&self.hand_type_value()))
            }

            // Matching hand types
            (DeuceSevenRank::StraightFlush(r1), DeuceSevenRank::StraightFlush(r2)) => {
                r1.partial_cmp(r2)
            }
            (DeuceSevenRank::FourOfAKind(r1), DeuceSevenRank::FourOfAKind(r2)) => {
                r1.partial_cmp(r2)
            }
            (DeuceSevenRank::FullHouse(t1, p1), DeuceSevenRank::FullHouse(t2, p2)) => {
                match t1.partial_cmp(t2) {
                    Some(Ordering::Equal) => p1.partial_cmp(p2),
                    ord => ord,
                }
            }
            (DeuceSevenRank::Flush(ranks1), DeuceSevenRank::Flush(ranks2)) => {
                // Compare cards one by one
                for (a, b) in ranks1.iter().zip(ranks2.iter()) {
                    match a.partial_cmp(b) {
                        Some(Ordering::Equal) => continue,
                        ord => return ord,
                    }
                }
                Some(Ordering::Equal)
            }
            (DeuceSevenRank::Straight(r1), DeuceSevenRank::Straight(r2)) => r1.partial_cmp(r2),
            (DeuceSevenRank::ThreeOfAKind(r1), DeuceSevenRank::ThreeOfAKind(r2)) => {
                r1.partial_cmp(r2)
            }
            (DeuceSevenRank::TwoPair(h1, l1, k1), DeuceSevenRank::TwoPair(h2, l2, k2)) => {
                // h = higher pair, l = lower pair, k = kicker
                match h1.partial_cmp(h2) {
                    Some(Ordering::Equal) => match l1.partial_cmp(l2) {
                        Some(Ordering::Equal) => k1.partial_cmp(k2),
                        ord => ord,
                    },
                    ord => ord,
                }
            }
            (DeuceSevenRank::Pair(r1, k1), DeuceSevenRank::Pair(r2, k2)) => {
                match r1.partial_cmp(r2) {
                    Some(Ordering::Equal) => {
                        for (a, b) in k1.iter().zip(k2.iter()) {
                            match a.partial_cmp(b) {
                                Some(Ordering::Equal) => continue,
                                ord => return ord,
                            }
                        }
                        Some(Ordering::Equal)
                    }
                    ord => ord,
                }
            }
            (DeuceSevenRank::HighCard(ranks1), DeuceSevenRank::HighCard(ranks2)) => {
                for (a, b) in ranks1.iter().zip(ranks2.iter()) {
                    match a.partial_cmp(b) {
                        Some(Ordering::Equal) => continue,
                        ord => return ord,
                    }
                }
                Some(Ordering::Equal)
            }

            _ => None, // Should never occur since all scenarios are tested above
        }
    }
}

impl DeuceSevenRank {
    fn hand_type_value(&self) -> u8 {
        match self {
            DeuceSevenRank::StraightFlush(_) => 9,
            DeuceSevenRank::FourOfAKind(_) => 8,
            DeuceSevenRank::FullHouse(_, _) => 7,
            DeuceSevenRank::Flush(_) => 6,
            DeuceSevenRank::Straight(_) => 5,
            DeuceSevenRank::ThreeOfAKind(_) => 4,
            DeuceSevenRank::TwoPair(_, _, _) => 3,
            DeuceSevenRank::Pair(_, _) => 2,
            DeuceSevenRank::HighCard(_) => 1,
        }
    }
}

impl fmt::Display for DeuceSevenRank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeuceSevenRank::StraightFlush(r) => write!(f, "Straight Flush, {} high", r),
            DeuceSevenRank::FourOfAKind(r) => write!(f, "Four of a Kind, {}s", r),
            DeuceSevenRank::FullHouse(t, p) => write!(f, "Full House, {}s full of {}s", t, p),
            DeuceSevenRank::Flush(ranks) => write!(f, "Flush, {} high", ranks[0]),
            DeuceSevenRank::Straight(r) => write!(f, "Straight, {} high", r),
            DeuceSevenRank::ThreeOfAKind(r) => write!(f, "Three of a Kind, {}s", r),
            DeuceSevenRank::TwoPair(h, l, k) => {
                write!(f, "Two Pair, {}s and {}s with {} kicker", h, l, k)
            }
            DeuceSevenRank::Pair(r, _) => write!(f, "Pair of {}s", r),
            DeuceSevenRank::HighCard(ranks) => write!(f, "High Card {}", ranks[0]),
        }
    }
}
