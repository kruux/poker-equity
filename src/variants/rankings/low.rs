use crate::cards::{Card, Rank};
use std::{cmp::Ordering, fmt};

/// An ace-to-five low hand: the five cards a holding plays for low.
///
/// Straights and flushes do not count and the ace is the lowest card, so a
/// hand is judged only on the ranks it plays. Pairing is what hurts, and it
/// hurts categorically: any hand with no pair beats any hand with a pair, one
/// pair beats two pair, two pair beats trips, and so on. Those are the
/// high-hand categories read upside down, which is why a paired low cannot be
/// reduced to its distinct ranks -- `5-4-3-2-2` and `5-4-4-3-2` are both
/// "five-four-three-two" but the first is a lower pair and wins.
#[derive(Debug, PartialEq, Clone)]
pub enum LowHandRank {
    /// The ranks played, worst first. Shorter than five only when the holding
    /// itself is.
    Low(Vec<Rank>),
}

/// Where a rank sits in a low hand. The ace is the lowest card, below the
/// deuce, rather than the highest.
fn low_value(rank: Rank) -> u8 {
    if rank == Rank::Ace {
        1
    } else {
        rank.to_value()
    }
}

/// One sortable value for a set of played ranks, where **lower is a better
/// low**.
///
/// The top bits hold how the ranks group up and the rest hold the ranks
/// themselves, so a single integer comparison decides both the category and
/// the tiebreak.
///
/// Group sizes are packed largest-first into five fixed slots, which makes the
/// packed value compare them the way a list would: `[1,1,1,1,1]` (no pair) is
/// smallest, then `[2,1,1,1]`, `[2,2,1]`, `[3,1,1]`, `[3,2]`, `[4,1]`. That
/// ordering *is* the lowball category order.
fn low_key(ranks: &[Rank]) -> u64 {
    let mut counts = [0u8; 15]; // indexed by Rank::to_value(), which is 2..=14
    for rank in ranks {
        counts[rank.to_value() as usize] += 1;
    }

    // Everything below stays on the stack. Naming a hand runs this over
    // every candidate five, so one allocation here is twenty-one a hand, and
    // it is the sweeps that check the lookup tables which pay for them.
    debug_assert!(ranks.len() <= MOST_RANKS, "more ranks than a deck holds one of");

    let mut groups = [0u8; MOST_RANKS];
    let mut distinct = 0;
    for &count in counts.iter() {
        if count > 0 {
            groups[distinct] = count;
            distinct += 1;
        }
    }
    groups[..distinct].sort_unstable_by(|a, b| b.cmp(a));

    let mut key: u64 = 0;
    for slot in 0..5 {
        key = (key << 3) | groups.get(slot).copied().unwrap_or(0) as u64;
    }

    // Within a category, the biggest group decides first -- the rank of the
    // pair before its kickers -- and a lower card is better.
    let played = ranks.len().min(MOST_RANKS);
    let mut ordered = [Rank::Two; MOST_RANKS];
    ordered[..played].copy_from_slice(&ranks[..played]);
    ordered[..played].sort_by(|a, b| {
        counts[b.to_value() as usize]
            .cmp(&counts[a.to_value() as usize])
            .then(low_value(*b).cmp(&low_value(*a)))
    });
    for slot in 0..5 {
        let value = ordered[..played]
            .get(slot)
            .map_or(0, |rank| low_value(*rank));
        key = (key << 4) | value as u64;
    }

    key
}

/// How many ranks a set can hold before the deck runs out of distinct ones.
const MOST_RANKS: usize = 13;

impl LowHandRank {
    /// The ranks played, worst first: highest card leads and the ace is last.
    pub fn ranks(&self) -> &[Rank] {
        match self {
            LowHandRank::Low(ranks) => ranks,
        }
    }

    /// A low hand from the ranks that are already known to play.
    ///
    /// Badugi picks which cards play by a rule of its own -- all ranks and
    /// all suits distinct -- but ranks them the same way once it has: more
    /// cards is better, then lower is better.
    pub fn from_played(ranks: Vec<Rank>) -> Self {
        Self::Low(Self::worst_first(ranks))
    }

    /// Highest card first, ace last.
    fn worst_first(mut ranks: Vec<Rank>) -> Vec<Rank> {
        ranks.sort_by_key(|rank| std::cmp::Reverse(low_value(*rank)));
        ranks
    }

    /// Scores the best low available from `cards`, playing five of them when
    /// there are five to play.
    pub fn evaluate(cards: &[Card]) -> Self {
        let ranks: Vec<Rank> = cards.iter().map(|card| card.rank()).collect();
        if ranks.len() <= 5 {
            return Self::Low(Self::worst_first(ranks));
        }

        // Play the best five. At seven cards that is twenty-one candidates,
        // which is cheaper to score outright than to reason about -- which
        // duplicate to keep depends on the whole holding.
        //
        // The five are walked by index rather than built, so the twenty-one
        // candidates cost nothing but the scoring.
        let mut best = [Rank::Two; 5];
        let mut best_key = u64::MAX;
        let count = ranks.len();
        for a in 0..count {
            for b in a + 1..count {
                for c in b + 1..count {
                    for d in c + 1..count {
                        for e in d + 1..count {
                            let five = [ranks[a], ranks[b], ranks[c], ranks[d], ranks[e]];
                            let key = low_key(&five);
                            if key < best_key {
                                best_key = key;
                                best = five;
                            }
                        }
                    }
                }
            }
        }
        Self::Low(Self::worst_first(best.to_vec()))
    }

    /// Whether this qualifies under an eight-or-better rule: five cards, no
    /// pair, and nothing above an eight.
    ///
    /// A qualifying low always has five distinct ranks, which is why the
    /// split games never meet a paired low.
    pub fn is_eight_or_better(&self) -> bool {
        let ranks = self.ranks();
        if ranks.len() != 5 {
            return false;
        }
        // A bit per rank, so counting the distinct ones needs nothing built.
        let mut seen: u16 = 0;
        for rank in ranks {
            seen |= 1 << rank.to_value();
        }
        // `ranks` is worst first, so the highest card leads.
        seen.count_ones() == 5 && low_value(ranks[0]) <= low_value(Rank::Eight)
    }
}

impl PartialOrd for LowHandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // More cards is better, so that a complete low beats a partial one.
        match self.ranks().len().cmp(&other.ranks().len()) {
            Ordering::Equal => {}
            ord => return Some(ord),
        }
        // Lower keys are better lows, so the comparison flips.
        Some(low_key(self.ranks()).cmp(&low_key(other.ranks())).reverse())
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
