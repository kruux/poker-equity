use std::{cmp::Ordering, fmt, mem::discriminant};

use crate::cards::Rank;

/// Used for many of the usual game types, like hold em and stud
#[derive(Debug)]
pub enum HighHandRank {
    StraightFlush(Rank),
    FourOfAKind(Rank),
    FullHouse(Rank, Rank), // Higher trips/pair is better
    Flush(Vec<Rank>),
    Straight(Rank),
    ThreeOfAKind(Rank),
    TwoPair(Rank, Rank, Rank),
    Pair(Rank, Vec<Rank>),
    HighCard(Vec<Rank>),
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
            (HighHandRank::FourOfAKind(r1), HighHandRank::FourOfAKind(r2)) => r2.partial_cmp(r1),
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
            (HighHandRank::ThreeOfAKind(r1), HighHandRank::ThreeOfAKind(r2)) => r2.partial_cmp(r1),
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
            HighHandRank::FourOfAKind(_) => 8,
            HighHandRank::FullHouse(_, _) => 7,
            HighHandRank::Flush(_) => 6,
            HighHandRank::Straight(_) => 5,
            HighHandRank::ThreeOfAKind(_) => 4,
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
            HighHandRank::FourOfAKind(r) => write!(f, "Four of a Kind, {}s", r),
            HighHandRank::FullHouse(t, p) => write!(f, "Full House, {}s full of {}s", t, p),
            HighHandRank::Flush(ranks) => write!(f, "Flush, {} high", ranks[0]),
            HighHandRank::Straight(r) => write!(f, "Straight, {} high", r),
            HighHandRank::ThreeOfAKind(r) => write!(f, "Three of a Kind, {}s", r),
            HighHandRank::TwoPair(h, l, k) => {
                write!(f, "Two Pair, {}s and {}s with {} kicker", h, l, k)
            }
            HighHandRank::Pair(r, _) => write!(f, "Pair of {}s", r),
            HighHandRank::HighCard(ranks) => write!(f, "High Card {}", ranks[0]),
        }
    }
}
