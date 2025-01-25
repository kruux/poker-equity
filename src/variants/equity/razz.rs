use std::collections::HashMap;

use crate::{cards::Deck, error::PokerError, hand::Hand, odds::EquityCalculator, variants::Razz};

use super::{EquityCalculation, StudCardGame};

impl StudCardGame for Razz {}
impl EquityCalculation for Razz {
    fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        self.validate_stud(calculator)
    }

    fn run_single_simulation(
        &self,
        deck: Deck,
        calculator: &EquityCalculator<Self>,
    ) -> Result<HashMap<String, f64>, PokerError> {
        let final_players = self.deal_cards(deck, calculator)?;

        // Convert to hands and evaluate
        let mut final_hands = Vec::new();
        for (name, cards) in final_players {
            let hand = Hand::new_with_cards(*self, cards)?;
            final_hands.push((name, hand));
        }

        // For Razz, lowest hand(s) wins 100% of pot
        let rankings = self.rank_hands(&final_hands)?;
        let winners = &rankings[0];

        let equity_share = 1.0 / (winners.len() as f64);
        let mut equity = HashMap::new();
        for winner in winners {
            equity.insert(winner.clone(), equity_share);
        }

        Ok(equity)
    }
}
