use crate::{
    cards::Card,
    error::{EquityError, GameError, PokerError},
    odds::EquityCalculator,
};

use super::EquityCalculation;

/// Has code that will be reused when simulating all stud based games
pub(crate) trait StudCardGame: EquityCalculation {
    /// The checks every stud game shares: enough players, all with the same
    /// number of cards, and at least the three they start with.
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

    /// Deals every player out to seven cards, a round at a time, and returns
    /// their holdings in seat order.
    fn deal_cards(
        &self,
        mut deck: crate::cards::Deck,
        calculator: &EquityCalculator<Self>,
    ) -> Result<Vec<Vec<Card>>, PokerError> {
        let mut players: Vec<Vec<Card>> = calculator
            .players()
            .iter()
            .map(|(_, hand, _)| hand.cards().to_vec())
            .collect();

        let rounds_left = 7 - players[0].len();
        for _ in 0..rounds_left {
            for cards in players.iter_mut() {
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
