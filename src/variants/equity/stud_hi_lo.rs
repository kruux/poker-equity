use super::{EquityCalculation, StudCardGame};
use crate::{
    cards::Deck, error::PokerError, hand::Hand, odds::EquityCalculator, variants::StudHiLo,
};

impl StudCardGame for StudHiLo {}
impl EquityCalculation for StudHiLo {
    fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        self.validate_stud(calculator)
    }

    fn award(&self, hands: &[Hand<Self>], shares: &mut [f64]) -> Result<(), PokerError> {
        let mut low_shares = vec![0.0; shares.len()];
        self.award_hi_lo(hands, shares, &mut low_shares)
    }

    fn award_detailed(
        &self,
        hands: &[Hand<Self>],
        shares: &mut [f64],
        low_shares: &mut [f64],
    ) -> Result<(), PokerError> {
        self.award_hi_lo(hands, shares, low_shares)
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
        self.award(&final_hands, shares)
    }
}
