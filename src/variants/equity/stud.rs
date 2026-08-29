use crate::{
    cards::Deck,
    error::PokerError,
    hand::Hand,
    odds::EquityCalculator,
    variants::Stud,
};

use super::{EquityCalculation, StudCardGame};

impl StudCardGame for Stud {}
impl EquityCalculation for Stud {
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
            .map(|cards| Hand::new_with_cards(Stud, cards))
            .collect::<Result<Vec<_>, _>>()?;
        self.award(&final_hands, shares)
    }
}
