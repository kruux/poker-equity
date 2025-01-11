use std::collections::HashMap;

use crate::{
    cards::Card,
    error::{EquityError, GameError},
    hand::Hand,
    variants::SevenCardStud,
};

use super::EquityCalculation;

impl EquityCalculation for SevenCardStud {
    fn validate(
        &self,
        calculator: &crate::odds::EquityCalculator<Self>,
    ) -> Result<(), crate::error::PokerError> {
        // Atleast 2 players for a meaningful simulation
        let players = calculator.players();
        if players.len() < 2 {
            return Err(EquityError::NoPlayers.into());
        }

        // Make sure all players have the same amount of cards and atleast 3
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

    fn run_single_simulation(
        &self,
        mut deck: crate::cards::Deck,
        calculator: &crate::odds::EquityCalculator<Self>,
    ) -> Result<std::collections::HashMap<String, f64>, crate::error::PokerError> {
        // Create a new player vec with cloned values so we can run multiple
        // simulations without changing starting hands
        let mut players: Vec<(String, Vec<Card>)> = calculator
            .players()
            .iter()
            .map(|(name, hand, _)| (name.clone(), hand.cards().to_vec()))
            .collect();

        // Deal the cards remaining
        let rounds_left = 7 - players[0].1.len();
        for _ in 0..rounds_left {
            for (_, current_cards) in players.iter_mut() {
                if let Some(card) = deck.deal() {
                    current_cards.push(card);
                } else {
                    return Err(GameError::NotEnoughCards.into());
                }
            }
        }

        // Store all players with Hand. Could probably use Hand while dealing cards aswell
        let mut final_hands: Vec<(String, Hand<SevenCardStud>)> = Vec::new();
        for (name, cards) in players {
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
