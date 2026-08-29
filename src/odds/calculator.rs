use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use crate::{
    cards::{Card, CardSet, Deck},
    error::{EquityError, GameError, PokerError},
    hand::Hand,
    variants::{CommunityCardGame, EquityCalculation, PokerVariant},
};

/// The older equity API, which owns its players and runs to a fixed count.
///
/// [`EquityRequest`](crate::odds::EquityRequest) and
/// [`run_chunk`](crate::odds::run_chunk) are the newer pair, and are what a
/// caller driving the loop itself should use. This one remains because it
/// models the draw games directly -- a player hands over a whole hand and the
/// cards it means to throw -- and because its tests encode a lot of what each
/// game's rules actually are.
pub struct EquityCalculator<V: PokerVariant + EquityCalculation> {
    players: Vec<(String, Hand<V>, Vec<Card>)>, // (name, current_hand, cards_to_discard)
    dead_cards: Vec<Card>,
    num_simulations: usize,
    community_cards: Vec<Card>,
    variant: V,
}

impl<V: PokerVariant + EquityCalculation> EquityCalculator<V> {
    /// A calculator for `variant` that will run `num_simulations` deals.
    pub fn new(variant: V, num_simulations: usize) -> Self {
        EquityCalculator {
            players: Vec::new(),
            dead_cards: Vec::new(),
            num_simulations,
            community_cards: Vec::new(),
            variant,
        }
    }

    /// Seats a player with the cards they hold.
    pub fn add_player(&mut self, name: String, hand: Hand<V>) -> Result<(), PokerError> {
        self.players.push((name, hand, vec![]));
        Ok(())
    }

    /// Seats a player in a draw game, with the cards they mean to throw.
    ///
    /// Passing `None` stands pat. The discards leave the deck along with the
    /// rest of the hand, so a thrown card cannot come back.
    pub fn add_draw_player(
        &mut self,
        name: String,
        hand: Hand<V>,
        cards_to_discard: Option<Vec<Card>>,
    ) -> Result<(), PokerError> {
        self.players
            .push((name, hand, cards_to_discard.unwrap_or_default()));
        Ok(())
    }

    /// The seated players, as name, hand, and cards to discard.
    pub fn players(&self) -> &[(String, Hand<V>, Vec<Card>)] {
        &self.players
    }

    /// Removes the cards from the deck
    /// Takes cards out of the deck without giving them to anyone.
    ///
    /// Errors if one is already in a hand or already dead, since that would
    /// be the same card twice.
    pub fn add_dead_cards(&mut self, cards: Vec<Card>) -> Result<(), PokerError> {
        // Check if any card is already in hands or dead_cards
        for card in &cards {
            // Check hands
            for (_, hand, _) in &self.players {
                if hand.cards().contains(card) {
                    return Err(GameError::DuplicateCard(*card).into());
                }
            }
            // Check existing dead cards
            if self.dead_cards.contains(card) {
                return Err(GameError::DuplicateCard(*card).into());
            }
        }

        self.dead_cards.extend(cards);
        Ok(())
    }

    /// Validation that will be checked for every poker variant
    /// - At least 2 players
    /// - No duplicated cards
    /// The checks every game shares: at least two players, and no card in
    /// two places at once.
    pub fn validate_base(&self) -> Result<(), PokerError> {
        if self.players.len() < 2 {
            return Err(EquityError::NoPlayers.into());
        }

        // Check for duplicated cards
        let mut seen_cards = HashSet::new();
        // Check all players
        for (_, hand, _) in &self.players {
            for card in hand.cards() {
                if !seen_cards.insert(card) {
                    return Err(GameError::DuplicateCard(*card).into());
                }
            }
        }
        // Check all dead cards
        for card in &self.dead_cards {
            if !seen_cards.insert(card) {
                return Err(GameError::DuplicateCard(*card).into());
            }
        }
        // Check all community cards
        for card in &self.community_cards {
            if !seen_cards.insert(card) {
                return Err(GameError::DuplicateCard(*card).into());
            }
        }

        Ok(())
    }

    /// The cards taken out of the deck.
    pub fn dead_cards(&self) -> &[Card] {
        &self.dead_cards
    }

    /// The board so far, which may be short.
    pub fn community_cards(&self) -> &[Card] {
        &self.community_cards
    }
}

impl<V: PokerVariant + CommunityCardGame + EquityCalculation> EquityCalculator<V> {
    /// Sets the board. Only community games have one.
    pub fn set_community_cards(&mut self, cards: Vec<Card>) -> Result<(), PokerError> {
        self.community_cards = cards;
        Ok(())
    }
}

impl<V: PokerVariant + EquityCalculation + Send + Sync> EquityCalculator<V> {
    /// Runs every deal and returns each player's equity as a percentage.
    ///
    /// `callback` is handed the running totals after each chunk. Work is
    /// spread across rayon, and the deck each deal starts from is worked out
    /// once rather than rebuilt per deal.
    pub fn calculate<F>(&self, callback: F) -> Result<HashMap<String, f64>, PokerError>
    where
        F: Fn(SimulationProgress) + Send + Sync,
    {
        if self.players.len() < 2 {
            return Err(EquityError::NoPlayers.into());
        }
        // Validate that simulation is ok. Every poker variant have to implement its own validation
        self.validate_base()?;
        self.variant.validate(self)?;

        // Everything the deal cannot touch, worked out once rather than per
        // simulation: the deck each deal starts from is the same every time.
        let mut known = CardSet::from_cards(&self.dead_cards);
        known = known.union(CardSet::from_cards(&self.community_cards));
        for (_, hand, _) in &self.players {
            known = known.union(CardSet::from_cards(hand.cards()));
        }
        let starting_deck = Deck::from_set(self.variant.deck().without(known));

        let seats = self.players.len();
        let completed_sims = Arc::new(Mutex::new(0));
        // Shares accumulate by seat, so nothing is keyed by name until the end.
        let totals = Arc::new(Mutex::new(vec![0.0f64; seats]));

        let chunk_size = 10000;
        let mut chunks = vec![chunk_size; self.num_simulations / chunk_size];
        let remainder = self.num_simulations % chunk_size;
        if remainder > 0 {
            chunks.push(remainder)
        };

        // Run the simulations in paralell
        chunks
            .par_iter()
            .try_for_each(|&size| -> Result<(), PokerError> {
                let mut local = vec![0.0f64; seats];

                for _ in 0..size {
                    self.variant
                        .run_single_simulation(starting_deck, self, &mut local)?;
                }

                // After chunk_size amount of simulations have been made update global results
                let mut global = totals.lock().unwrap();
                for (total, local) in global.iter_mut().zip(local) {
                    *total += local;
                }
                let mut completed = completed_sims.lock().unwrap();
                *completed += size;

                // Create progress update
                let progress = SimulationProgress {
                    completed_simulations: *completed,
                    total_simulations: self.num_simulations,
                    progress_percent: (*completed as f64 / self.num_simulations as f64) * 100.0,
                    current_results: self
                        .players
                        .iter()
                        .zip(global.iter())
                        .map(|((name, _, _), total)| {
                            (name.clone(), (total / *completed as f64) * 100.0)
                        })
                        .collect(),
                };

                // Call callback function with the latest results
                callback(progress);

                Ok(())
            })?;

        let totals = totals.lock().unwrap();
        let percentages = self
            .players
            .iter()
            .zip(totals.iter())
            .map(|((name, _, _), total)| {
                (name.clone(), (total / self.num_simulations as f64) * 100.0)
            })
            .collect();

        Ok(percentages)
    }
}
// Large number of simulations might take some time so continously update with the latest results
/// The running totals, handed to the callback between chunks.
pub struct SimulationProgress {
    pub completed_simulations: usize,
    pub total_simulations: usize,
    pub progress_percent: f64,
    pub current_results: HashMap<String, f64>,
}
