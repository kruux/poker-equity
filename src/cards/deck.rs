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
        for i in 0..52 as usize {
            let rnd = get_random_number(i as u32, 51) as usize;
            self.cards.swap(i, rnd);
        }
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

fn get_random_number(from: u32, to: u32) -> u32 {
    let mut rng = rand::thread_rng();
    let y: f64 = rng.gen();
    let offset = to - from + 1;
    let n: u32 = (y * offset as f64) as u32 + from;
    return n;
}
