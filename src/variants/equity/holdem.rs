use std::collections::HashMap;

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
    ) -> Result<HashMap<String, f64>, PokerError> {
        let community_cards = self.deal_community_cards(calculator, deck)?;

        // Add the community cards to every players hand
        let final_hands = calculator
            .players()
            .iter()
            .map(|(name, hand, _)| {
                let mut final_hand = hand.clone();
                final_hand
                    .add_cards(community_cards.clone())
                    .map_err(PokerError::from)?;
                Ok((name.clone(), final_hand))
            })
            .collect::<Result<Vec<_>, PokerError>>()?;

        let rankings = self.rank_hands(&final_hands)?;
        let winners = &rankings[0];

        let equity_share = 1.0 / (winners.len() as f64);
        let mut equity_map = HashMap::new();
        for winner in winners {
            equity_map.insert(winner.clone(), equity_share);
        }

        Ok(equity_map)
    }
}
