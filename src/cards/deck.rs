use enum_iterator::all;
use rand::prelude::*;

use crate::error::CardError;

use super::{Card, Rank, Suit};

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::<Card>::new();
        for suit in all::<Suit>() {
            for rank in all::<Rank>() {
                let card: Card = Card::new(suit, rank);
                cards.push(card);
            }
        }
        let mut deck = Self { cards };
        deck.shuffle();
        return deck;
    }

    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn remove_card(&mut self, card: &Card) -> Result<(), CardError> {
        let initial_len = self.cards.len();
        self.cards.retain(|c| !c.matches(card));

        if self.cards.len() == initial_len {
            return Err(CardError::CardNotFound(*card));
        }

        Ok(())
    }

    pub fn cards(&self) -> &Vec<Card> {
        return &self.cards;
    }

    pub fn remaining_cards(&self) -> usize {
        self.cards.len()
    }
}
