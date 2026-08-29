use super::{EquityCalculation, StudCardGame};
use crate::{
    cards::Deck, error::PokerError, hand::Hand, odds::EquityCalculator, variants::StudHiLo,
};

impl StudCardGame for StudHiLo {}
impl EquityCalculation for StudHiLo {
    fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        self.validate_stud(calculator)
    }

    /// Half the pot for the best high hand and half for the best qualifying
    /// low, each split among ties. With nobody qualifying, the high hand
    /// scoops.
    fn award(&self, hands: &[Hand<Self>], shares: &mut [f64]) -> Result<(), PokerError> {
        let high_share = match self.rank_low_hands(hands)? {
            Some(low_places) => {
                let low_winners = &low_places[0];
                let low_share = 0.5 / low_winners.len() as f64;
                for &seat in low_winners {
                    shares[seat] += low_share;
                }
                0.5
            }
            None => 1.0,
        };

        let high_winners = &self.rank_hands(hands)?[0];
        let share = high_share / high_winners.len() as f64;
        for &seat in high_winners {
            shares[seat] += share;
        }
        Ok(())
    }

    fn award_detailed(
        &self,
        hands: &[Hand<Self>],
        shares: &mut [f64],
        low_shares: &mut [f64],
    ) -> Result<(), PokerError> {
        if let Some(low_places) = self.rank_low_hands(hands)? {
            let low_winners = &low_places[0];
            let low_share = 0.5 / low_winners.len() as f64;
            for &seat in low_winners {
                low_shares[seat] += low_share;
            }
        }
        self.award(hands, shares)
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
