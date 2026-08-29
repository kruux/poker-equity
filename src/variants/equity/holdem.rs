use super::{CommunityCardGame, EquityCalculation};
use crate::{
    cards::Deck,
    error::{EquityError, PokerError},
    odds::EquityCalculator,
    variants::Holdem,
};

impl CommunityCardGame for Holdem {}
impl EquityCalculation for Holdem {
    fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        // Validation reused by all community card games
        self.validate_community(calculator)?;

        // Make sure all players have 2 cards
        let players = calculator.players();
        for (_, hand, _) in players {
            if hand.num_cards() != 2 {
                return Err(EquityError::NotEnoughCards(hand.num_cards()).into());
            }
        }

        Ok(())
    }

    fn run_single_simulation(
        &self,
        deck: Deck,
        calculator: &EquityCalculator<Self>,
        shares: &mut [f64],
    ) -> Result<(), PokerError> {
        let community_cards = self.deal_community_cards(calculator, deck)?;
        let final_hands = self.build_final_hands(calculator, community_cards)?;
        self.award(&final_hands, shares)
    }
}
