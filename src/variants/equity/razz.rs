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
        shares: &mut [f64],
    ) -> Result<(), PokerError> {
        let final_players = self.deal_cards(deck, calculator)?;
        let final_hands = final_players
            .into_iter()
            .map(|cards| Hand::new_with_cards(*self, cards))
            .collect::<Result<Vec<_>, _>>()?;

        // In razz the lowest hand takes the whole pot.
        let winners = &self.rank_hands(&final_hands)?[0];
        let share = 1.0 / winners.len() as f64;
        for &seat in winners {
            shares[seat] += share;
        }

        Ok(())
    }
}
