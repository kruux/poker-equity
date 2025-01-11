use std::collections::HashMap;

use crate::{hand::Hand, variants::SevenCardStud};

use super::{EquityCalculation, StudEquity};

impl StudEquity for SevenCardStud {}
impl EquityCalculation for SevenCardStud {
    fn validate(
        &self,
        calculator: &crate::odds::EquityCalculator<Self>,
    ) -> Result<(), crate::error::PokerError> {
        self.validate_stud(calculator)
    }

    fn run_single_simulation(
        &self,
        deck: crate::cards::Deck,
        calculator: &crate::odds::EquityCalculator<Self>,
    ) -> Result<std::collections::HashMap<String, f64>, crate::error::PokerError> {
        let final_players = self.deal_cards(deck, calculator)?;

        // Store all players with Hand.
        let mut final_hands: Vec<(String, Hand<SevenCardStud>)> = Vec::new();
        for (name, cards) in final_players {
            let final_hand = Hand::new_with_cards(SevenCardStud, cards)?;
            final_hands.push((name, final_hand));
        }

        // Check winners
        let rankings = self.rank_hands(&final_hands)?;
        let winners = &rankings[0];

        // Calculate equity
        let equity_share = 1.0 / (winners.len() as f64);
        let mut equity_map: HashMap<String, f64> = HashMap::new();
        for winner in winners {
            equity_map.insert(winner.clone(), equity_share);
        }

        Ok(equity_map)
    }
}
