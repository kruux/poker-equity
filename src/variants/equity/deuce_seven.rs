use std::collections::HashMap;

use super::EquityCalculation;
use crate::{
    cards::Deck,
    error::{GameError, PokerError},
    hand::Hand,
    odds::EquityCalculator,
    variants::DeuceSeven,
};

impl EquityCalculation for DeuceSeven {
    fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        // Check that cards_to_discard are actually in the hand for all players
        for (_, hand, cards_to_discard) in calculator.players() {
            for card in cards_to_discard {
                if !hand.cards().contains(card) {
                    return Err(GameError::InvalidDiscard(*card).into());
                }
            }
        }
        Ok(())
    }

    fn run_single_simulation(
        &self,
        mut deck: Deck,
        calculator: &EquityCalculator<Self>,
    ) -> Result<std::collections::HashMap<String, f64>, PokerError> {
        // For each player, remove their cards and deal new ones
        let mut final_hands: Vec<(String, Hand<DeuceSeven>)> = Vec::new();

        for (name, initial_hand, cards_to_discard) in calculator.players() {
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

            let final_hand = Hand::new_with_cards(DeuceSeven, current_cards)?;
            final_hands.push((name.clone(), final_hand));
        }

        // Compare hands to find winner
        let rankings = self.rank_hands(&final_hands)?;
        let winners = &rankings[0];

        // Calculate equity
        let equity_share = 1.0 / (winners.len() as f64); // Important to split up the equity in ties
        let mut equity: HashMap<String, f64> = HashMap::new();
        for winner in winners {
            equity.insert(winner.clone(), equity_share);
        }

        Ok(equity)
    }
}

