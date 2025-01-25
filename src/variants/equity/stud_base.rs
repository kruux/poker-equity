use crate::{
    cards::Card,
    error::{EquityError, GameError, PokerError},
    odds::EquityCalculator,
};

use super::EquityCalculation;

/// Has code that will be reused when simulating all stud based games
pub(crate) trait StudCardGame: EquityCalculation {
    fn validate_stud(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        // At least 2 players for a meaningful simulation
        let players = calculator.players();
        if players.len() < 2 {
            return Err(EquityError::NoPlayers.into());
        }

        // Make sure all players have the same amount of cards and at least 3
        let first_hand_size = players[0].1.num_cards();
        if first_hand_size < 3 {
            return Err(EquityError::NotEnoughCards(first_hand_size).into());
        }

        if !players
            .iter()
            .all(|(_, hand, _)| hand.num_cards() == first_hand_size)
        {
            return Err(EquityError::UnequalHandSizes.into());
        }

        Ok(())
    }

    fn deal_cards(
        &self,
        mut deck: crate::cards::Deck,
        calculator: &EquityCalculator<Self>,
    ) -> Result<Vec<(String, Vec<Card>)>, PokerError> {
        // Create a new player vec with cloned values
        let mut players = calculator
            .players()
            .iter()
            .map(|(name, hand, _)| (name.clone(), hand.cards().to_vec()))
            .collect::<Vec<_>>();

        // Deal the remaining cards to reach 7
        let rounds_left = 7 - players[0].1.len();
        for _ in 0..rounds_left {
            for (_, cards) in players.iter_mut() {
                if let Some(card) = deck.deal() {
                    cards.push(card);
                } else {
                    return Err(GameError::NotEnoughCards.into());
                }
            }
        }
        Ok(players)
    }
}
