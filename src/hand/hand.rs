use std::cmp::Ordering;
use std::fmt;

use crate::variants::PokerVariant;
use crate::{
    cards::{Card, Rank, Suit},
    error::{CardError, PokerError},
};

/// The cards a player holds, together with the game they are playing.
///
/// A hand carries its variant, so it knows how to score itself and how many
/// cards it may hold. Community cards are added to a copy of each player's
/// hand at showdown rather than stored separately.
#[derive(Clone, Debug)]
pub struct Hand<V: PokerVariant> {
    cards: Vec<Card>,
    variant: V,
}

impl<V: PokerVariant> Hand<V> {
    /// An empty hand.
    pub fn new(variant: V) -> Self {
        Self {
            cards: Vec::<Card>::with_capacity(variant.max_cards()),
            variant,
        }
    }

    /// A hand holding `cards`, or an error if that is more than the game
    /// deals.
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
        if !len.is_multiple_of(2) {
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

    /// Replaces the cards with `hole` followed by `board`, keeping whatever
    /// room the hand has already taken.
    ///
    /// The deal loop shows the same seats down over and over, and only the
    /// cards change. Writing over a hand rather than building a new one is
    /// what keeps a chunk of a million deals from asking the allocator a
    /// million times.
    pub fn refill(&mut self, hole: &[Card], board: &[Card]) -> Result<(), CardError> {
        let total = hole.len() + board.len();
        if total > self.variant.max_cards() {
            return Err(CardError::TooManyCards(total));
        }
        self.cards.clear();
        self.cards.extend_from_slice(hole);
        self.cards.extend_from_slice(board);
        Ok(())
    }

    /// Adds one card, or errors if the hand is already full.
    pub fn add_card(&mut self, card: Card) -> Result<(), CardError> {
        let n = self.num_cards() + 1;
        if n > self.variant.max_cards() {
            Err(CardError::TooManyCards(n))
        } else {
            self.cards.push(card);
            Ok(())
        }
    }

    /// Adds several cards, stopping at the first that will not fit.
    pub fn add_cards(&mut self, cards: Vec<Card>) -> Result<(), CardError> {
        for card in cards {
            self.add_card(card)?;
        }
        Ok(())
    }

    /// The cards held, in the order they were added: private cards first,
    /// then any shared board.
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    /// Throws cards away, erroring if the hand does not hold one of them.
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
