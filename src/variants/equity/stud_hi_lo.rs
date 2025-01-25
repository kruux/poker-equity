use std::collections::HashMap;

use super::{EquityCalculation, StudCardGame};
use crate::{
    cards::Deck, error::PokerError, hand::Hand, odds::EquityCalculator, variants::StudHiLo,
};

impl StudCardGame for StudHiLo {}
impl EquityCalculation for StudHiLo {
    fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        self.validate_stud(calculator)
    }

    fn run_single_simulation(
        &self,
        deck: Deck,
        calculator: &EquityCalculator<Self>,
    ) -> Result<HashMap<String, f64>, PokerError> {
        // Deal remaining cards
        let final_players = self.deal_cards(deck, calculator)?;

        // Convert to hands
        let mut final_hands = Vec::<(String, Hand<StudHiLo>)>::new();
        for (name, cards) in final_players {
            let hand = Hand::new_with_cards(*self, cards)?;
            final_hands.push((name, hand));
        }

        let mut equity_map = HashMap::new();

        // Get high winners
        let high_rankings = self.rank_hands(&final_hands)?;
        let high_winners = &high_rankings[0];
        let high_share: f64; // Set's to 1 or 0.5 depending on if there are low hands

        // Get low winners and add equity if there are any
        let low_rankings = self.rank_low_hands(&final_hands)?;
        match low_rankings {
            Some(low_rankings) => {
                high_share = 0.5;
                let low_winners = &low_rankings[0];
                let low_eq = 0.5 / low_winners.len() as f64;
                for winner in low_winners {
                    *equity_map.entry(winner.clone()).or_insert(0.0) += low_eq;
                }
            }
            None => {
                high_share = 1.0;
            }
        }

        // Add high winners
        let high_eq = high_share / high_winners.len() as f64;
        for winner in high_winners {
            *equity_map.entry(winner.clone()).or_insert(0.0) += high_eq;
        }

        Ok(equity_map)
    }
}
