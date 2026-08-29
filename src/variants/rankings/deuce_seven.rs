use std::{cmp::Ordering, fmt};

use crate::cards::{Card, Rank};

use super::HighHandRank;

#[derive(Debug)]
pub enum DeuceSevenRank {
    StraightFlush(Rank),           // Rank of highest card
    FourOfAKind(Rank, Rank),       // Rank of quads
    FullHouse(Rank, Rank),         // Rank of trips then pair
    Flush([Rank; 5]),              // Vec of rank in descending order
    Straight(Rank),                // Highest card
    ThreeOfAKind(Rank, [Rank; 2]), // Rank of trips
    TwoPair(Rank, Rank, Rank),     // High pair, low pair, kicker
    Pair(Rank, [Rank; 3]),         // Rank of pair, vec of kickers in descending order
    HighCard([Rank; 5]),           // High to low
    Incomplete(usize),             // Incomplete hand
}

impl DeuceSevenRank {
    pub fn evaluate(cards: &[Card]) -> Self {
        // Use HighHandRank evaluation and handle the special cases
        match HighHandRank::evaluate(cards) {
            HighHandRank::StraightFlush(r) => {
                // A2345 flush
                if r == Rank::Five {
                    Self::Flush([Rank::Ace, Rank::Five, Rank::Four, Rank::Three, Rank::Two])
                } else {
                    Self::StraightFlush(r)
                }
            }
            HighHandRank::FourOfAKind(r, k) => Self::FourOfAKind(r, k),
            HighHandRank::FullHouse(t, p) => Self::FullHouse(t, p),
            HighHandRank::Flush(ranks) => Self::Flush(ranks),
            HighHandRank::Straight(r) => {
                // A2345 straight
                if r == Rank::Five {
                    Self::HighCard([Rank::Ace, Rank::Five, Rank::Four, Rank::Three, Rank::Two])
                } else {
                    return Self::Straight(r);
                }
            }
            HighHandRank::ThreeOfAKind(r, k) => Self::ThreeOfAKind(r, k),
            HighHandRank::TwoPair(h, l, k) => Self::TwoPair(h, l, k),
            HighHandRank::Pair(r, k) => Self::Pair(r, k),
            HighHandRank::HighCard(ranks) => Self::HighCard(ranks),
            HighHandRank::Incomplete(n) => Self::Incomplete(n),
        }
    }
    pub fn sorted_ranks(cards: &[Card]) -> Vec<Rank> {
        let mut ranks: Vec<Rank> = cards.iter().map(|card| card.rank()).collect();
        ranks.sort_by(|a, b| b.cmp(a)); // Sorts descending
        ranks
    }
}

impl PartialOrd for DeuceSevenRank {
    /// Deuce-to-seven is the high ranking upside down: the *worst* high hand
    /// wins, so both the category and the tiebreak ranks compare in reverse.
    /// The ace is forced high, and straights and flushes count against you --
    /// both handled in `evaluate` rather than here.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // An incomplete hand is ordered by how many cards it holds.
            (Self::Incomplete(mine), Self::Incomplete(theirs)) => mine.partial_cmp(theirs),
            _ => Some(
                other
                    .hand_type_value()
                    .cmp(&self.hand_type_value())
                    .then_with(|| other.tiebreak().cmp(&self.tiebreak())),
            ),
        }
    }
}

impl DeuceSevenRank {
    /// The ranks that separate hands of the same category, most significant
    /// first, as rank values with unused slots left at zero.
    fn tiebreak(&self) -> [u8; 5] {
        let value = |rank: &Rank| rank.to_value();
        match self {
            Self::StraightFlush(rank) | Self::Straight(rank) => [value(rank), 0, 0, 0, 0],
            Self::FourOfAKind(quads, kicker) => [value(quads), value(kicker), 0, 0, 0],
            Self::FullHouse(trips, pair) => [value(trips), value(pair), 0, 0, 0],
            Self::Flush(ranks) | Self::HighCard(ranks) => [
                value(&ranks[0]),
                value(&ranks[1]),
                value(&ranks[2]),
                value(&ranks[3]),
                value(&ranks[4]),
            ],
            Self::ThreeOfAKind(trips, kickers) => {
                [value(trips), value(&kickers[0]), value(&kickers[1]), 0, 0]
            }
            Self::TwoPair(high, low, kicker) => [value(high), value(low), value(kicker), 0, 0],
            Self::Pair(pair, kickers) => [
                value(pair),
                value(&kickers[0]),
                value(&kickers[1]),
                value(&kickers[2]),
                0,
            ],
            Self::Incomplete(_) => [0; 5],
        }
    }
}

impl DeuceSevenRank {
    /// The category's place in the standard high ranking. Deuce-to-seven
    /// compares these in reverse, so a *lower* value is the better hand.
    fn hand_type_value(&self) -> u8 {
        match self {
            DeuceSevenRank::StraightFlush(_) => 9,
            DeuceSevenRank::FourOfAKind(_, _) => 8,
            DeuceSevenRank::FullHouse(_, _) => 7,
            DeuceSevenRank::Flush(_) => 6,
            DeuceSevenRank::Straight(_) => 5,
            DeuceSevenRank::ThreeOfAKind(_, _) => 4,
            DeuceSevenRank::TwoPair(_, _, _) => 3,
            DeuceSevenRank::Pair(_, _) => 2,
            DeuceSevenRank::HighCard(_) => 1,
            DeuceSevenRank::Incomplete(_) => 0,
        }
    }
}

impl fmt::Display for DeuceSevenRank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeuceSevenRank::StraightFlush(r) => write!(f, "Straight Flush, {} high", r),
            DeuceSevenRank::FourOfAKind(r, _) => write!(f, "Four of a Kind, {}s", r),
            DeuceSevenRank::FullHouse(t, p) => write!(f, "Full House, {}s full of {}s", t, p),
            DeuceSevenRank::Flush(ranks) => write!(f, "Flush, {} high", ranks[0]),
            DeuceSevenRank::Straight(r) => write!(f, "Straight, {} high", r),
            DeuceSevenRank::ThreeOfAKind(r, _) => write!(f, "Three of a Kind, {}s", r),
            DeuceSevenRank::TwoPair(h, l, k) => {
                write!(f, "Two Pair, {}s and {}s with {} kicker", h, l, k)
            }
            DeuceSevenRank::Pair(r, _) => write!(f, "Pair of {}s", r),
            DeuceSevenRank::HighCard(ranks) => write!(f, "High Card {}", ranks[0]),
            DeuceSevenRank::Incomplete(n) => write!(f, "Incomplete hand ({} cards)", n),
        }
    }
}
