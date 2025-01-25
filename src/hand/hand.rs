use std::cmp::Ordering;
use std::fmt;

use crate::variants::PokerVariant;
use crate::{
    cards::{Card, Rank, Suit},
    error::{CardError, PokerError},
};

#[derive(Clone, Debug)]
pub struct Hand<V: PokerVariant> {
    cards: Vec<Card>,
    variant: V,
}

impl<V: PokerVariant> Hand<V> {
    pub fn new(variant: V) -> Self {
        Self {
            cards: Vec::<Card>::with_capacity(variant.max_cards()),
            variant,
        }
    }

    pub fn new_with_cards(variant: V, cards: Vec<Card>) -> Result<Self, CardError> {
        if cards.len() > variant.max_cards() {
            return Err(CardError::TooManyCards(cards.len()));
        }
        Ok(Self { cards, variant })
    }

    /// Takes a string e.g. "AdKc" and returns a Hand struct containing a Vec<Cards>
    /// Both "AhKh" and "Ah Kh will give the same result as whitespace is stripped"
    pub fn from_str(variant: V, cards_str: &str) -> Result<Self, PokerError> {
        let cards_chars: Vec<char> = cards_str.chars().filter(|c| !c.is_whitespace()).collect();
        let max_cards = variant.max_cards();
        let mut cards: Vec<Card> = Vec::with_capacity(max_cards);
        // Make sure it's even to avoid breaking the for loop
        let len = cards_chars.len();
        if len == 0 {
            return Ok(Self { cards, variant });
        }
        if len % 2 != 0 {
            return Err(CardError::InvalidFormat(
                "Both suit and rank have to be provided for every card".to_string(),
            )
            .into());
        } else if len > max_cards * 2 {
            // More than max cards
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

        Ok(Self { cards, variant })
    }

    pub fn add_card(&mut self, card: Card) -> Result<(), CardError> {
        let n = self.num_cards() + 1;
        if n > self.variant.max_cards() {
            Err(CardError::TooManyCards(n))
        } else {
            self.cards.push(card);
            Ok(())
        }
    }

    pub fn add_cards(&mut self, cards: Vec<Card>) -> Result<(), CardError> {
        for card in cards {
            self.add_card(card)?;
        }
        Ok(())
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
    pub fn evaluate(&self) -> V::HandRank {
        self.variant.evaluate_hand(&self.cards)
    }
}

// <V: PokerVariant> Hand<V>
impl<V: PokerVariant> PartialEq for Hand<V> {
    fn eq(&self, other: &Self) -> bool {
        self.evaluate() == other.evaluate()
    }
}

impl<V: PokerVariant> PartialOrd for Hand<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.evaluate().partial_cmp(&other.evaluate())
    }
}

impl<V: PokerVariant> fmt::Display for Hand<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, card) in self.cards.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", card)?;
        }

        Ok(())
    }
}
