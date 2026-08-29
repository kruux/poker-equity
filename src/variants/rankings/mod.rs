mod deuce_seven;
mod fast;
mod hand_rank_table;
mod hi_lo;
mod high;
mod low;
mod rank_translation;

pub type StudHandRank = HighHandRank;
pub type RazzHandRank = LowHandRank;
pub type StudHiLoHandRank = HiLoHandRank;
pub type HoldemHandRank = HighHandRank;
pub type OmahaHandRank = HighHandRank;
pub use deuce_seven::DeuceSevenRank;
pub use fast::{FastHandRank, FLUSH_KEYS, RANK_KEYS};
pub(crate) use hand_rank_table::{FLUSH_RANKS, HAND_RANKS};
pub use hi_lo::HiLoHandRank;
pub use high::HighHandRank;
pub use low::LowHandRank;
pub use rank_translation::{fast_to_high, high_to_fast};

macro_rules! impl_hand_rank_eq {
    ($type:ty) => {
        impl PartialEq for $type {
            fn eq(&self, other: &Self) -> bool {
                // Check that both hands have the same HandRank and then make sure ranks and kickers match
                match (self, other) {
                    (Self::StraightFlush(r1), Self::StraightFlush(r2)) => r1 == r2,
                    (Self::FourOfAKind(r1, k1), Self::FourOfAKind(r2, k2)) => r1 == r2 && k1 == k2,
                    (Self::FullHouse(t1, p1), Self::FullHouse(t2, p2)) => t1 == t2 && p1 == p2,
                    (Self::Flush(ranks1), Self::Flush(ranks2)) => {
                        ranks1.len() == ranks2.len()
                            && ranks1.iter().zip(ranks2.iter()).all(|(a, b)| a == b)
                    }
                    (Self::Straight(r1), Self::Straight(r2)) => r1 == r2,
                    (Self::ThreeOfAKind(t1, k1), Self::ThreeOfAKind(t2, k2)) => {
                        t1 == t2 && k1.iter().zip(k2.iter()).all(|(a, b)| a == b)
                    }
                    (Self::TwoPair(h1, l1, k1), Self::TwoPair(h2, l2, k2)) => {
                        h1 == h2 && l1 == l2 && k1 == k2
                    }
                    (Self::Pair(r1, k1), Self::Pair(r2, k2)) => {
                        r1 == r2
                            && k1.len() == k2.len()
                            && k1.iter().zip(k2.iter()).all(|(a, b)| a == b)
                    }
                    (Self::HighCard(r1), Self::HighCard(r2)) => {
                        r1.len() == r2.len() && r1.iter().zip(r2.iter()).all(|(a, b)| a == b)
                    }
                    (Self::Incomplete(n1), Self::Incomplete(n2)) => n1 == n2,
                    _ => false, // Not the same HandRank type.
                }
            }
        }
    };
}

impl_hand_rank_eq!(DeuceSevenRank);
impl_hand_rank_eq!(HighHandRank);
