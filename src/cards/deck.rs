use rand::Rng;

use crate::error::CardError;

use super::{Card, CardSet};

/// The cards still available to deal.
///
/// This is a [`CardSet`] behind a dealing API, so removing known cards is a
/// bitwise operation rather than a scan. There is no shuffle: the set has no
/// order, and [`deal`](Deck::deal) draws uniformly from whatever remains,
/// which is the same distribution a shuffle-then-pop produced at a fraction of
/// the cost.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deck {
    remaining: CardSet,
}

impl Deck {
    /// A full fifty-two card deck.
    pub fn new() -> Self {
        Self {
            remaining: CardSet::FULL_DECK,
        }
    }

    /// A deck holding exactly `cards`.
    pub fn from_set(cards: CardSet) -> Self {
        Self { remaining: cards }
    }

    /// The cards still in the deck.
    pub fn remaining(&self) -> CardSet {
        self.remaining
    }

    /// Draws a card at random, or `None` when the deck is empty.
    pub fn deal(&mut self) -> Option<Card> {
        let count = self.remaining.len();
        if count == 0 {
            return None;
        }
        let card = self.remaining.nth(rand::thread_rng().gen_range(0..count))?;
        self.remaining.remove(card);
        Some(card)
    }

    /// Takes a known card out of the deck.
    ///
    /// Errors when the card has already gone, which is how a duplicate in the
    /// caller's input is caught.
    pub fn remove_card(&mut self, card: &Card) -> Result<(), CardError> {
        if !self.remaining.contains(*card) {
            return Err(CardError::CardNotFound(*card));
        }
        self.remaining.remove(*card);
        Ok(())
    }

    /// Takes every card in `cards` out of the deck, without checking whether
    /// they were there. One instruction, and the path the sampler uses.
    pub fn remove_all(&mut self, cards: CardSet) {
        self.remaining = self.remaining.without(cards);
    }

    /// How many cards are left.
    pub fn remaining_cards(&self) -> usize {
        self.remaining.len() as usize
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}
