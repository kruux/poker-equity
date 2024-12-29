use std::collections::HashMap;

use crate::{
    cards::{Card, Deck, Hand},
    error::{GameError, PokerError},
};
// use crate::game::GameState;

pub struct EquityCalculator {
    players: Vec<(String, Hand, Vec<Card>)>, // (name, current_hand, cards_to_discard)
    dead_cards: Vec<Card>,
    num_simulations: usize,
    results: HashMap<String, f64>,
}

impl EquityCalculator {
    pub fn new(num_simulations: usize) -> Self {
        EquityCalculator {
            players: Vec::new(),
            dead_cards: Vec::new(),
            num_simulations,
            results: HashMap::new(),
        }
    }

    pub fn add_player(
        &mut self,
        name: String,
        hand: Hand,
        cards_to_discard: Vec<Card>,
    ) -> Result<(), PokerError> {
        // Check that cards_to_discard are actually in the hand
        for card in &cards_to_discard {
            if !hand.cards().contains(card) {
                return Err(GameError::InvalidDiscard(*card).into());
            }
        }

        self.players.push((name, hand, cards_to_discard));
        Ok(())
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

    pub fn calculate(&mut self) -> Result<&HashMap<String, f64>, PokerError> {
        // Clear previous results
        self.results.clear();

        // Initialize win counts
        let mut win_counts: HashMap<String, f64> = self
            .players
            .iter()
            .map(|(name, _, _)| (name.clone(), 0.0))
            .collect();

        // Run simulations
        for _ in 0..self.num_simulations {
            let rankings = self.run_single_simulation()?;

            let winners = &rankings[0];
            let equity_share = 1.0 / (winners.len() as f64); // Important to split up the equity in ties

            for winner in winners {
                *win_counts.get_mut(winner).unwrap() += equity_share;
            }
        }

        // Convert counts to percentages
        for (name, wins) in win_counts {
            self.results
                .insert(name, (wins as f64) / (self.num_simulations as f64) * 100.0);
        }

        Ok(&self.results)
    }

    fn run_single_simulation(&self) -> Result<Vec<Vec<String>>, PokerError> {
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

        // For each player, remove their cards and deal new ones
        let mut final_hands: Vec<(String, Hand)> = Vec::new();

        for (name, initial_hand, cards_to_discard) in &self.players {
            // Create new hand by discarding specified cards
            let mut current_cards = initial_hand.cards().to_vec();
            current_cards.retain(|card| !cards_to_discard.contains(card));

            // Check how many cards are needed
            let cards_needed = 5 - current_cards.len();

            // Draw new cards
            for _ in 0..cards_needed {
                if let Some(card) = deck.deal() {
                    current_cards.push(card);
                } else {
                    return Err(GameError::NotEnoughCards.into());
                }
            }

            let final_hand = Hand::new_with_cards(current_cards);
            final_hands.push((name.clone(), final_hand));
        }

        // Compare hands to find winner
        self.rank_hands(&final_hands)
    }

    /// Returns a 2 dimensional array. It's 2 dimensional to handle any ties in any position.
    /// First position contain a vec with the winners.
    /// Second index a vec with the players in second.
    fn rank_hands(&self, hands: &[(String, Hand)]) -> Result<Vec<Vec<String>>, PokerError> {
        if hands.is_empty() {
            return Ok(vec![]);
        }

        // Sort hands and group by strength
        let mut sorted_hands = hands.to_vec();
        sorted_hands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Create result vec and add the player in first place to another vec
        // Don't add the winner until we make sure no other has an equal hand
        let mut result: Vec<Vec<String>> = vec![];
        let mut current_group: Vec<String> = vec![sorted_hands[0].0.clone()];

        // Compare the next hands with previous.
        for i in 1..sorted_hands.len() {
            let prev_hand = &sorted_hands[i - 1].1;
            let curr_hand = &sorted_hands[i].1;

            if prev_hand == curr_hand {
                // Tied with previous hand. Add to current vec
                current_group.push(sorted_hands[i].0.clone());
            } else {
                // Last hand was stronger. Push vector and create a new with current hand
                result.push(current_group);
                current_group = vec![sorted_hands[i].0.clone()];
            }
        }
        // Add last vec to result
        result.push(current_group);

        Ok(result)
    }
}
