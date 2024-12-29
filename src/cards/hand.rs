use std::cmp::{min, Ordering};
use std::collections::HashMap;
use std::fmt;
use std::mem::discriminant;

use crate::{
    cards::{Card, Rank, Suit},
    error::{CardError, PokerError},
};

#[derive(Debug)]
pub enum HandRank {
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

impl PartialEq for HandRank {
    fn eq(&self, other: &Self) -> bool {
        // Check that both hands have the same HandRank and then make sure ranks and kickers match
        match (self, other) {
            (HandRank::StraightFlush(r1), HandRank::StraightFlush(r2)) => r1 == r2,
            (HandRank::FourOfAKind(r1), HandRank::FourOfAKind(r2)) => r1 == r2,
            (HandRank::FullHouse(t1, p1), HandRank::FullHouse(t2, p2)) => t1 == t2 && p1 == p2,
            (HandRank::Flush(ranks1), HandRank::Flush(ranks2)) => {
                ranks1.len() == ranks2.len()
                    && ranks1.iter().zip(ranks2.iter()).all(|(a, b)| a == b)
            }
            (HandRank::Straight(r1), HandRank::Straight(r2)) => r1 == r2,
            (HandRank::ThreeOfAKind(r1), HandRank::ThreeOfAKind(r2)) => r1 == r2,
            (HandRank::TwoPair(h1, l1, k1), HandRank::TwoPair(h2, l2, k2)) => {
                h1 == h2 && l1 == l2 && k1 == k2
            }
            (HandRank::Pair(r1, k1), HandRank::Pair(r2, k2)) => {
                r1 == r2 && k1.len() == k2.len() && k1.iter().zip(k2.iter()).all(|(a, b)| a == b)
            }
            (HandRank::HighCard(r1), HandRank::HighCard(r2)) => {
                r1.len() == r2.len() && r1.iter().zip(r2.iter()).all(|(a, b)| a == b)
            }
            _ => false, // Not the same HandRank type.
        }
    }
}

impl PartialOrd for HandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // Non matching hands. E.g. pair vs trips
            (x, y) if discriminant(x) != discriminant(y) => {
                // Reverse order of hands in 2-7
                Some(other.hand_type_value().cmp(&self.hand_type_value()))
            }

            // Matching hand types
            (HandRank::StraightFlush(r1), HandRank::StraightFlush(r2)) => r1.partial_cmp(r2),
            (HandRank::FourOfAKind(r1), HandRank::FourOfAKind(r2)) => r1.partial_cmp(r2),
            (HandRank::FullHouse(t1, p1), HandRank::FullHouse(t2, p2)) => {
                match t1.partial_cmp(t2) {
                    Some(Ordering::Equal) => p1.partial_cmp(p2),
                    ord => ord,
                }
            }
            (HandRank::Flush(ranks1), HandRank::Flush(ranks2)) => {
                // Compare cards one by one
                for (a, b) in ranks1.iter().zip(ranks2.iter()) {
                    match a.partial_cmp(b) {
                        Some(Ordering::Equal) => continue,
                        ord => return ord,
                    }
                }
                Some(Ordering::Equal)
            }
            (HandRank::Straight(r1), HandRank::Straight(r2)) => r1.partial_cmp(r2),
            (HandRank::ThreeOfAKind(r1), HandRank::ThreeOfAKind(r2)) => r1.partial_cmp(r2),
            (HandRank::TwoPair(h1, l1, k1), HandRank::TwoPair(h2, l2, k2)) => {
                // h = higher pair, l = lower pair, k = kicker
                match h1.partial_cmp(h2) {
                    Some(Ordering::Equal) => match l1.partial_cmp(l2) {
                        Some(Ordering::Equal) => k1.partial_cmp(k2),
                        ord => ord,
                    },
                    ord => ord,
                }
            }
            (HandRank::Pair(r1, k1), HandRank::Pair(r2, k2)) => match r1.partial_cmp(r2) {
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
            },
            (HandRank::HighCard(ranks1), HandRank::HighCard(ranks2)) => {
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

impl HandRank {
    fn hand_type_value(&self) -> u8 {
        match self {
            HandRank::StraightFlush(_) => 9,
            HandRank::FourOfAKind(_) => 8,
            HandRank::FullHouse(_, _) => 7,
            HandRank::Flush(_) => 6,
            HandRank::Straight(_) => 5,
            HandRank::ThreeOfAKind(_) => 4,
            HandRank::TwoPair(_, _, _) => 3,
            HandRank::Pair(_, _) => 2,
            HandRank::HighCard(_) => 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        Self {
            cards: Vec::<Card>::with_capacity(5),
        }
    }

    pub fn new_with_cards(cards: Vec<Card>) -> Self {
        Self { cards }
    }

    /// Takes a string e.g. "AdKc" and returns a Hand struct containing a Vec<Cards>
    /// Both "AhKh" and "Ah Kh will give the same result as whitespace is stripped"
    pub fn from_str(cards_str: &str) -> Result<Self, PokerError> {
        let cards_chars: Vec<char> = cards_str.chars().filter(|c| !c.is_whitespace()).collect();
        let mut cards: Vec<Card> = Vec::with_capacity(5);
        // Make sure it's even to avoid breaking the for loop
        let len = cards_chars.len();
        if len % 2 != 0 {
            return Err(CardError::InvalidFormat(
                "Both suit and rank have to be provided for every card".to_string(),
            )
            .into());
        } else if len > 10 {
            // More than 5 cards
            return Err(CardError::TooManyCards(len / 2).into());
        }
        for i in (0..len - 1).step_by(2) {
            let rank_char = cards_chars[i];
            let rank = Rank::from_char(rank_char)?;
            let suit_char = cards_chars[i + 1];
            let suit = Suit::from_char(suit_char)?;
            let card = Card::new(suit, rank);
            cards.push(card);
        }

        Ok(Self { cards })
    }

    pub fn compare(hand1: &Hand, hand2: &Hand) -> Result<Ordering, PokerError> {
        // Check for incomplete hands
        if hand1.num_cards() != 5 || hand2.num_cards() != 5 {
            return Err(
                CardError::IncompleteHand(min(hand1.num_cards(), hand2.num_cards())).into(),
            );
        }

        Ok(hand1.evaluate().partial_cmp(&hand2.evaluate()).unwrap())
    }

    pub fn add_card(&mut self, card: Card) -> Result<(), CardError> {
        let n = self.num_cards() + 1;
        if n > 5 {
            Err(CardError::TooManyCards(n))
        } else {
            self.cards.push(card);
            Ok(())
        }
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn discard(&mut self, cards_to_discard: &Vec<Card>) -> Result<(), CardError> {
        // Check we don't discard too many cards
        let discard_len = cards_to_discard.len();
        if discard_len > 5 {
            return Err(CardError::TooManyDiscards(discard_len));
        }

        // Make sure all cards we want to discard in the hand exist.
        for card in cards_to_discard {
            if !self.cards().iter().any(|hand_card| hand_card.matches(card)) {
                return Err(CardError::CardNotFound(*card));
            }
        }

        // Once we know the cards exist remove them
        self.cards.retain(|hand_card| {
            !cards_to_discard
                .iter()
                .any(|discard_card| hand_card.matches(discard_card))
        });

        Ok(())
    }

    /// Returns the number of cards the hand has
    pub fn num_cards(&self) -> usize {
        self.cards.len()
    }

    /// Returns HandRank which contains strength of hand
    pub fn evaluate(&self) -> HandRank {
        let rank_counts = &self.rank_counts();
        let is_flush = self.is_flush();

        // Check strongest to weakest according to normal hold em rules
        // Apparantly let chaining is unstable still. Use && instead of two ifs in the future
        if is_flush {
            if let Some(rank) = self.is_straight() {
                return HandRank::StraightFlush(rank);
            }
        }

        if let Some(rank) = self.is_four_of_kind(&rank_counts) {
            return HandRank::FourOfAKind(rank);
        }

        if let Some((trips, pair)) = self.is_full_house(&rank_counts) {
            return HandRank::FullHouse(trips, pair);
        }

        if is_flush {
            return HandRank::Flush(self.sorted_ranks());
        }

        if let Some(rank) = self.is_straight() {
            return HandRank::Straight(rank);
        }

        if let Some(rank) = self.is_three_of_kind(&rank_counts) {
            return HandRank::ThreeOfAKind(rank);
        }

        if let Some((high_pair, low_pair, kicker)) = self.is_two_pair(&rank_counts) {
            return HandRank::TwoPair(high_pair, low_pair, kicker);
        }

        if let Some((pair_rank, kickers)) = self.is_pair(&rank_counts) {
            return HandRank::Pair(pair_rank, kickers);
        }

        return HandRank::HighCard(self.sorted_ranks());
    }

    fn is_four_of_kind(&self, rank_counts: &HashMap<Rank, usize>) -> Option<Rank> {
        rank_counts
            .iter()
            .find(|&(_, &count)| count == 4)
            .map(|(rank, _)| *rank)
    }

    fn is_full_house(&self, rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, Rank)> {
        if let Some(&trips_rank) = rank_counts
            .iter()
            .find(|&(_, &count)| count == 3)
            .map(|(rank, _)| rank)
        {
            if let Some(&pair_rank) = rank_counts
                .iter()
                .find(|&(_, &count)| count == 2)
                .map(|(rank, _)| rank)
            {
                return Some((trips_rank, pair_rank));
            }
        }
        None
    }

    fn is_straight(&self) -> Option<Rank> {
        // Try to design it so it can handle 7 cards
        let ranks = self.sorted_ranks();
        if ranks.len() < 5 {
            return None;
        }

        // Cards are sorted decrementally. E.g. KQJT9
        // so the next card should always be one lower
        // This does not check A2345. That is not a straight in 2-7 however
        let mut consecutive = 0;
        for i in 0..ranks.len() - 1 {
            if ranks[i] as u8 == ranks[i + 1] as u8 + 1 {
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
            .find(|&(_, &count)| count == 3)
            .map(|(rank, _)| *rank)
    }

    /// Checks for two pair. Returns (high_pair, low_pair, kicker) if it finds two pair.
    /// Otherwise None
    fn is_two_pair(&self, rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, Rank, Rank)> {
        // This function needs to be reworked in case of 7 card games
        let pairs: Vec<Rank> = rank_counts
            .iter()
            .filter(|&(_, &count)| count == 2)
            .map(|(rank, _)| *rank)
            .collect();
        if pairs.len() == 2 {
            // Find kicker
            let kicker = rank_counts
                .iter()
                .find(|&(_, &count)| count == 1)
                .map(|(rank, _)| *rank)?;
            // Highest pair needs to be returned first.
            let (high_pair, low_pair) = if pairs[0].to_value() > pairs[1].to_value() {
                (pairs[0], pairs[1])
            } else {
                (pairs[1], pairs[0])
            };

            return Some((high_pair, low_pair, kicker));
        }
        None
    }

    fn is_pair(&self, rank_counts: &HashMap<Rank, usize>) -> Option<(Rank, Vec<Rank>)> {
        if let Some((&pair_rank, _)) = rank_counts.iter().find(|&(_, &count)| count == 2) {
            let mut kickers: Vec<Rank> = rank_counts
                .iter()
                .filter(|&(_, &count)| count == 1)
                .map(|(rank, _)| *rank)
                .collect();
            kickers.sort_by(|a, b| b.cmp(a));
            return Some((pair_rank, kickers));
        }
        None
    }

    fn rank_counts(&self) -> HashMap<Rank, usize> {
        let mut counts = HashMap::new();
        for card in &self.cards {
            *counts.entry(card.rank()).or_insert(0) += 1;
        }
        counts
    }

    fn is_flush(&self) -> bool {
        self.suit_counts().values().any(|count| *count >= 5)
    }

    fn suit_counts(&self) -> HashMap<Suit, usize> {
        let mut counts = HashMap::new();
        for card in &self.cards {
            *counts.entry(card.suit()).or_insert(0) += 1;
        }
        counts
    }

    fn sorted_ranks(&self) -> Vec<Rank> {
        let mut ranks: Vec<Rank> = self.cards.iter().map(|card| card.rank()).collect();
        ranks.sort_by(|a, b| b.cmp(a)); // Sorts descending
        ranks
    }
}

impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.evaluate() == other.evaluate()
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.evaluate().partial_cmp(&other.evaluate())
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, card) in self.cards.iter().enumerate() {
            if i > 0 {
                write!(f, " {}", card)?;
            }
        }

        Ok(())
    }
}
