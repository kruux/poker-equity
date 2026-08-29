use std::{cmp::Ordering, fmt};

use super::HighHandRank;

/// A short-deck hand, which is a normal high hand with the categories in a
/// different order.
///
/// Two of them move, both because a thirty-six card deck changes how often
/// each comes up: a flush beats a full house, and trips beat a straight.
/// Rooms differ on this, so the ruleset is named in the variant's label.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ShortDeckRank(pub HighHandRank);

impl ShortDeckRank {
    /// The category's place in the short-deck ranking, higher being better.
    fn category(&self) -> u8 {
        match self.0 {
            HighHandRank::StraightFlush(_) => 9,
            HighHandRank::FourOfAKind(_, _) => 8,
            HighHandRank::Flush(_) => 7,
            HighHandRank::FullHouse(_, _) => 6,
            HighHandRank::ThreeOfAKind(_, _) => 5,
            HighHandRank::Straight(_) => 4,
            HighHandRank::TwoPair(_, _, _) => 3,
            HighHandRank::Pair(_, _) => 2,
            HighHandRank::HighCard(_) => 1,
            HighHandRank::Incomplete(_) => 0,
        }
    }

    /// The hand as it would be named in a full-deck game.
    pub fn as_high(&self) -> &HighHandRank {
        &self.0
    }
}

impl PartialOrd for ShortDeckRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ShortDeckRank {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.0, &other.0) {
            (HighHandRank::Incomplete(mine), HighHandRank::Incomplete(theirs)) => mine.cmp(theirs),
            // Within a category the tiebreak is unchanged, so the high hand's
            // own comparison decides it.
            _ if self.category() == other.category() => self.0.cmp(&other.0),
            _ => self.category().cmp(&other.category()),
        }
    }
}

impl fmt::Display for ShortDeckRank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
