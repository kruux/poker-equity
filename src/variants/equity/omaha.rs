use super::{CommunityCardGame, EquityCalculation};
use crate::{
    cards::Deck,
    error::{EquityError, PokerError},
    odds::EquityCalculator,
    variants::{Courchevel, Omaha, OmahaFive, OmahaSix, PokerVariant},
};

/// Wires an Omaha variant into the equity engine. They differ only in how
/// many hole cards a player must hold.
macro_rules! omaha_equity {
    ($name:ident) => {
        impl CommunityCardGame for $name {}
        impl EquityCalculation for $name {
            fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
                self.validate_community(calculator)?;
                for (_, hand, _) in calculator.players() {
                    if hand.num_cards() != self.hole_cards() {
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
    };
}

omaha_equity!(Omaha);
omaha_equity!(OmahaFive);
omaha_equity!(OmahaSix);

impl CommunityCardGame for Courchevel {
    /// Courchevel deals the first board card face up before the betting, so
    /// unlike Omaha a one-card board is a real spot rather than a mistake,
    /// and a board with nothing on it cannot happen.
    fn validate_community(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        let players = calculator.players();
        if players.len() < 2 {
            return Err(EquityError::NoPlayers.into());
        }
        let first = players[0].1.num_cards();
        if !players.iter().all(|(_, hand, _)| hand.num_cards() == first) {
            return Err(EquityError::UnequalHandSizes.into());
        }

        let board = calculator.community_cards().len();
        if !matches!(board, 1 | 3 | 4) {
            return Err(EquityError::InvalidCommunityCards(board).into());
        }
        Ok(())
    }
}

impl EquityCalculation for Courchevel {
    fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        self.validate_community(calculator)?;
        for (_, hand, _) in calculator.players() {
            if hand.num_cards() != self.hole_cards() {
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
