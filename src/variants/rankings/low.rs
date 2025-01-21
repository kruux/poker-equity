use crate::cards::{Card, Rank};
use std::{cmp::Ordering, fmt};

#[derive(Debug, PartialEq, Clone)]
pub enum LowHandRank {
    Low(Vec<Rank>), // Unique ranks in descending order
}

impl LowHandRank {
    pub fn ranks(&self) -> &[Rank] {
        match self {
            LowHandRank::Low(ranks) => ranks,
        }
    }

    pub fn evaluate(cards: &[Card]) -> Self {
        let mut ranks: Vec<Rank> = cards.iter().map(|card| card.rank()).collect();

        // Sort ranks for low hand (Ace is lowest)
        ranks.sort_by(|a, b| {
            if *a == Rank::Ace {
                Ordering::Less
            } else if *b == Rank::Ace {
                Ordering::Greater
            } else {
                a.cmp(b)
            }
        });

        // Remove duplicates as they don't count in low hands
        ranks.dedup();

        // Take the best 5 ranks
        ranks.truncate(5);

        // Return all ranks in descending order
        ranks.reverse();
        Self::Low(ranks)
    }
}

impl PartialOrd for LowHandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (LowHandRank::Low(ranks1), LowHandRank::Low(ranks2)) => {
                // Compare length of ranks first
                match ranks1.len().cmp(&ranks2.len()) {
                    Ordering::Equal => {
                        // Compare each rank
                        for (r1, r2) in ranks1.iter().zip(ranks2.iter()) {
                            if &r1 == &r2 {
                                continue;
                            }
                            // Special case for ace.
                            if *r1 == Rank::Ace {
                                return Some(Ordering::Greater);
                            }
                            if *r2 == Rank::Ace {
                                return Some(Ordering::Less);
                            }
                            return Some(r1.cmp(r2).reverse()); // Normal comparison for non ace cards
                        }
                        Some(Ordering::Equal)
                    }
                    ord => Some(ord),
                }
            }
        }
    }
}

impl fmt::Display for LowHandRank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LowHandRank::Low(ranks) => write!(
                f,
                "{}",
                ranks
                    .iter()
                    .map(|r| r.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
}
