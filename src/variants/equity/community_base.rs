use crate::{
    cards::{Card, Deck},
    error::{EquityError, GameError, PokerError},
    hand::Hand,
    odds::EquityCalculator,
};

use super::EquityCalculation;

/// Shared by the games that deal a board: hold'em, the Omaha family, short
/// deck. They differ only in how many cards a player holds, so the dealing
/// and the validation live here once.
pub trait CommunityCardGame: EquityCalculation {
    /// The checks every community game shares: enough players, equal hand
    /// sizes, and a board that still has something to deal.
    fn validate_community(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        // At least 2 players for a meaningful simulation
        let players = calculator.players();
        if players.len() < 2 {
            return Err(EquityError::NoPlayers.into());
        }

        // Make sure all players have the same amount of cards
        let first_hand_size = players[0].1.num_cards();
        if !players
            .iter()
            .all(|(_, hand, _)| hand.num_cards() == first_hand_size)
        {
            return Err(EquityError::UnequalHandSizes.into());
        }

        // Number of community cards have to be 0, 3, or 4. With 5 there's nothing to simulate
        let num_cards = calculator.community_cards().len();
        if num_cards != 0 && num_cards != 3 && num_cards != 4 {
            return Err(EquityError::InvalidCommunityCards(num_cards).into());
        }

        Ok(())
    }

    /// Fills the board out to five cards from `deck`.
    fn deal_community_cards(
        &self,
        calculator: &EquityCalculator<Self>,
        mut deck: Deck,
    ) -> Result<Vec<Card>, PokerError> {
        let mut community_cards = calculator.community_cards().to_vec();
        // Deal community cards, assuming all players have all their cards already
        for _ in community_cards.len()..5 {
            let card = deck.deal().ok_or(GameError::NotEnoughCards)?;
            community_cards.push(card);
        }

        Ok(community_cards)
    }

    /// Each player's hole cards with the board added, in seat order.
    fn build_final_hands(
        &self,
        calculator: &EquityCalculator<Self>,
        community_cards: Vec<Card>,
    ) -> Result<Vec<Hand<Self>>, PokerError> {
        calculator
            .players()
            .iter()
            .map(|(_, hand, _)| {
                let mut final_hand = hand.clone();
                final_hand
                    .add_cards(community_cards.clone())
                    .map_err(PokerError::from)?;
                Ok(final_hand)
            })
            .collect()
    }
}
