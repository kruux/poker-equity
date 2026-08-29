use std::{cmp::Ordering, collections::HashMap, fmt};

use crate::cards::{Card, Rank, Suit};

use super::FastHandRank;

/// Used for many of the usual game types, like hold em and stud
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum HighHandRank {
    StraightFlush(Rank),
    FourOfAKind(Rank, Rank),
    FullHouse(Rank, Rank), // Higher trips/pair is better
    Flush([Rank; 5]),
    Straight(Rank),
    ThreeOfAKind(Rank, [Rank; 2]),
    TwoPair(Rank, Rank, Rank),
    Pair(Rank, [Rank; 3]),
    HighCard([Rank; 5]),
    Incomplete(usize),
}

/// The ace playing low in a full deck: A-5-4-3-2, a five-high straight.
const WHEEL: [Rank; 5] = [Rank::Ace, Rank::Five, Rank::Four, Rank::Three, Rank::Two];

/// The ace playing low in a short deck: A-9-8-7-6, a nine-high straight.
const SHORT_WHEEL: [Rank; 5] = [Rank::Ace, Rank::Nine, Rank::Eight, Rank::Seven, Rank::Six];

impl HighHandRank {
    /// Names the best five-card high hand in `cards`.
    ///
    /// Fewer than five cards gives an incomplete rank, which loses to any
    /// complete hand and orders among other incomplete ones by size. This is
    /// the slow, readable answer; the lookup tables are checked against it.
    pub fn evaluate(cards: &[Card]) -> Self {
        if cards.len() < 5 {
            return Self::Incomplete(cards.len());
        }

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
            Self::HighCard(Self::first_n_ranks::<5>(&sorted))
        }
    }

    /// Scores a hand under short-deck rules.
    ///
    /// The classification is the ordinary one -- no hand of seven cards can
    /// be both a flush and a full house, so the fact that a flush outranks
    /// one here changes only the comparison, not which category a hand falls
    /// into. What does change is the ace: it plays low below the six, so
    /// A-9-8-7-6 is a straight and the lowest straight flush is nine high.
    pub fn evaluate_short_deck(cards: &[Card]) -> Self {
        if cards.len() < 5 {
            return Self::Incomplete(cards.len());
        }

        let rank_counts = Self::rank_counts(cards);
        let straight_flush = Self::sorted_suits(cards)
            .iter()
            .filter(|(_, suited)| suited.len() >= 5)
            .find_map(|(_, suited)| Self::is_straight_with_wheel(suited, &SHORT_WHEEL));

        if let Some(rank) = straight_flush {
            Self::StraightFlush(rank)
        } else if let Some((quads, kicker)) = Self::is_four_of_kind(&rank_counts) {
            Self::FourOfAKind(quads, kicker)
        } else if let Some((trips, pair)) = Self::is_full_house(&rank_counts) {
            Self::FullHouse(trips, pair)
        } else if let Some(ranks) = Self::is_flush(cards) {
            Self::Flush(ranks)
        } else if let Some(rank) = Self::is_straight_with_wheel(cards, &SHORT_WHEEL) {
            Self::Straight(rank)
        } else if let Some((trips, kickers)) = Self::is_three_of_kind(&rank_counts) {
            Self::ThreeOfAKind(trips, kickers)
        } else if let Some((high, low, kicker)) = Self::is_two_pair(&rank_counts) {
            Self::TwoPair(high, low, kicker)
        } else if let Some((pair, kickers)) = Self::is_pair(&rank_counts) {
            Self::Pair(pair, kickers)
        } else {
            let sorted = Self::sorted_ranks(cards);
            Self::HighCard(Self::first_n_ranks::<5>(&sorted))
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

    fn is_flush(cards: &[Card]) -> Option<[Rank; 5]> {
        let cards_by_suit = Self::sorted_suits(cards);

        cards_by_suit
            .iter()
            .filter(|(_, cards)| cards.len() >= 5)
            .map(|(_, cards)| {
                let sorted = Self::sorted_ranks(cards);
                Self::first_n_ranks::<5>(&sorted)
            })
            .next()
    }

    /// The highest straight among `cards`, as the rank that tops it.
    ///
    /// Duplicate ranks are dropped before the walk. A pair alongside a
    /// straight neither makes nor breaks it, but leaving the duplicates in
    /// shifts the position of the run and so misnames the rank that tops it.
    fn is_straight(cards: &[Card]) -> Option<Rank> {
        Self::is_straight_with_wheel(cards, &WHEEL)
    }

    /// As [`is_straight`](Self::is_straight), but told which hand the ace
    /// plays low in. A full deck has A-5-4-3-2; a short deck has A-9-8-7-6,
    /// since it holds no card below the six.
    fn is_straight_with_wheel(cards: &[Card], wheel: &[Rank; 5]) -> Option<Rank> {
        let mut distinct = Self::sorted_ranks(cards);
        distinct.dedup();

        // Highest first, so the first run of five is the best straight.
        let mut run = 1;
        for i in 1..distinct.len() {
            if distinct[i - 1].to_value() == distinct[i].to_value() + 1 {
                run += 1;
                if run == 5 {
                    return Some(distinct[i - 4]);
                }
            } else {
                run = 1;
            }
        }

        // The wheel is the one straight a descending walk cannot see: the ace
        // sits at the top of the list and plays at the bottom of the hand.
        if wheel.iter().all(|rank| distinct.contains(rank)) {
            return Some(wheel[1]);
        }

        None
    }

    fn is_three_of_kind(rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, [Rank; 2])> {
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
            let best_kickers = Self::first_n_ranks::<2>(&kickers);

            return Some((trips_rank, best_kickers));
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

    fn is_pair(rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, [Rank; 3])> {
        if let Some((&pair_rank, _)) = rank_counts.iter().find(|&(_, &count)| count == 2) {
            let mut kickers: Vec<Rank> = rank_counts
                .iter()
                .filter(|&(_, &count)| count == 1)
                .map(|(rank, _)| *rank)
                .collect();
            kickers.sort_by(|a, b| b.cmp(a));
            let best_kickers = Self::first_n_ranks::<3>(&kickers);
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
                .or_default()
                .push(card);
        }
        cards_by_suit
    }

    fn sorted_ranks(cards: &[Card]) -> Vec<Rank> {
        let mut ranks: Vec<Rank> = cards.iter().map(|card| card.rank()).collect();
        ranks.sort_by(|a, b| b.cmp(a)); // Sort descending
        ranks
    }

    fn first_n_ranks<const N: usize>(ranks: &[Rank]) -> [Rank; N] {
        ranks[..N]
            .try_into()
            .expect("Slice doesn't contain enough elements")
    }
}

impl PartialOrd for HighHandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HighHandRank {
    /// Better hands compare greater. The category decides first; hands of the
    /// same category are separated by their tiebreak ranks, high card first.
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // An incomplete hand is ordered by how many cards it holds, which
            // no tiebreak can express.
            (Self::Incomplete(mine), Self::Incomplete(theirs)) => mine.cmp(theirs),
            _ => self
                .hand_type_value()
                .cmp(&other.hand_type_value())
                .then_with(|| self.tiebreak().cmp(&other.tiebreak())),
        }
    }
}

impl HighHandRank {
    /// The ranks that separate hands of the same category, most significant
    /// first, as rank values with unused slots left at zero.
    ///
    /// Comparing two of these arrays compares the ranks in order, which is
    /// exactly how hands of one category are ranked against each other.
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

impl HighHandRank {
    /// The category's place in the standard ranking, where a higher value is
    /// a better class of hand.
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
            HighHandRank::Incomplete(_) => 0,
        }
    }
}

impl From<FastHandRank> for HighHandRank {
    fn from(_fast_rank: FastHandRank) -> Self {
        // We can't convert from FastHandRank to HighHandRank anymore since FastHandRank
        // only contains a score. Instead, we'll need to evaluate the hand directly.
        unimplemented!("Cannot convert from FastHandRank to HighHandRank - use evaluate() instead")
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
            HighHandRank::Incomplete(n) => write!(f, "Incomplete hand ({} cards)", n),
        }
    }
}
