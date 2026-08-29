use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use crate::{
    cards::{Card, Deck},
    error::{EquityError, GameError, PokerError},
    hand::Hand,
    variants::{CommunityCardGame, EquityCalculation, PokerVariant},
};

pub struct EquityCalculator<V: PokerVariant + EquityCalculation> {
    players: Vec<(String, Hand<V>, Vec<Card>)>, // (name, current_hand, cards_to_discard)
    dead_cards: Vec<Card>,
    num_simulations: usize,
    community_cards: Vec<Card>,
    variant: V,
}

impl<V: PokerVariant + EquityCalculation> EquityCalculator<V> {
    pub fn new(variant: V, num_simulations: usize) -> Self {
        EquityCalculator {
            players: Vec::new(),
            dead_cards: Vec::new(),
            num_simulations,
            community_cards: Vec::new(),
            variant,
        }
    }

    pub fn add_player(&mut self, name: String, hand: Hand<V>) -> Result<(), PokerError> {
        self.players.push((name, hand, vec![]));
        Ok(())
    }

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

    pub fn players(&self) -> &[(String, Hand<V>, Vec<Card>)] {
        &self.players
    }

    /// Removes the cards from the deck
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

    pub fn dead_cards(&self) -> &[Card] {
        &self.dead_cards
    }

    pub fn community_cards(&self) -> &[Card] {
        &self.community_cards
    }
}

impl<V: PokerVariant + CommunityCardGame + EquityCalculation> EquityCalculator<V> {
    pub fn set_community_cards(&mut self, cards: Vec<Card>) -> Result<(), PokerError> {
        self.community_cards = cards;
        Ok(())
    }
}

impl<V: PokerVariant + EquityCalculation + Send + Sync> EquityCalculator<V> {
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

        let completed_sims = Arc::new(Mutex::new(0));
        let results = Arc::new(Mutex::new(HashMap::<String, f64>::new()));

        // Add all players to the results with 0 equity
        for (name, _, _) in &self.players {
            results.lock().unwrap().insert(name.clone(), 0.0);
        }

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
                let mut local_results = HashMap::new();

                for _ in 0..size {
                    // Create a new deck
                    let mut deck = Deck::new();

                    // Remove known cards from deck
                    for card in &self.dead_cards {
                        deck.remove_card(card)?;
                    }

                    // Remove all cards that are in players' hands
                    for (_, hand, _) in &self.players {
                        for card in hand.cards() {
                            deck.remove_card(card)?;
                        }
                    }
                    // Remove all community cards
                    for card in &self.community_cards {
                        deck.remove_card(card)?;
                    }

                    let sim_results = self.variant.run_single_simulation(deck, self)?;
                    for (name, equity) in sim_results {
                        *local_results.entry(name).or_insert(0.0) += equity;
                    }
                }

                // After chunk_size amount of simulations have been made update global results
                let mut global_results = results.lock().unwrap();
                for (name, equity) in local_results {
                    *global_results.entry(name).or_insert(0.0) += equity;
                }
                let mut completed = completed_sims.lock().unwrap();
                *completed += size;

                // Create progress update
                let progress = SimulationProgress {
                    completed_simulations: *completed,
                    total_simulations: self.num_simulations,
                    progress_percent: (*completed as f64 / self.num_simulations as f64) * 100.0,
                    current_results: global_results
                        .iter()
                        .map(|(name, equity)| (name.clone(), (*equity / *completed as f64) * 100.0))
                        .collect(),
                };

                // Call callback function with the latest results
                callback(progress);

                Ok(())
            })?;

        let final_results = results.lock().unwrap();
        let mut percentages = HashMap::new();
        for (name, equity) in final_results.iter() {
            percentages.insert(
                name.clone(),
                (*equity / self.num_simulations as f64) * 100.0,
            );
        }

        Ok(percentages)
    }
}
// Large number of simulations might take some time so continously update with the latest results
pub struct SimulationProgress {
    pub completed_simulations: usize,
    pub total_simulations: usize,
    pub progress_percent: f64,
    pub current_results: HashMap<String, f64>,
}
