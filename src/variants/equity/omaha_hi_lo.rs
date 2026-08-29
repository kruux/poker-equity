use super::{CommunityCardGame, EquityCalculation};
use crate::{
    cards::Deck,
    error::{EquityError, PokerError},
    hand::Hand,
    odds::EquityCalculator,
    variants::{CourchevelHiLo, OmahaFiveHiLo, OmahaHiLo, PokerVariant},
};

/// Wires a split-pot Omaha variant into the equity engine.
macro_rules! omaha_hi_lo_equity {
    ($name:ident) => {
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
                let community_cards = self.deal_community_cards(calculator, deck)?;
                let final_hands = self.build_final_hands(calculator, community_cards)?;
                let mut low_shares = vec![0.0; shares.len()];
                self.award_hi_lo(&final_hands, shares, &mut low_shares)
            }
        }
    };
}

impl CommunityCardGame for OmahaHiLo {}
impl CommunityCardGame for OmahaFiveHiLo {}

/// As with Courchevel, the first board card is face up before the betting.
impl CommunityCardGame for CourchevelHiLo {
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

omaha_hi_lo_equity!(OmahaHiLo);
omaha_hi_lo_equity!(OmahaFiveHiLo);
omaha_hi_lo_equity!(CourchevelHiLo);
