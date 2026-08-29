use super::{CommunityCardGame, EquityCalculation};
use crate::{
    cards::Deck,
    error::{EquityError, PokerError},
    odds::EquityCalculator,
    variants::{PokerVariant, ShortDeck},
};

impl CommunityCardGame for ShortDeck {}
impl EquityCalculation for ShortDeck {
    fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        self.validate_community(calculator)?;
        for (_, hand, _) in calculator.players() {
            if hand.num_cards() != self.hole_cards() {
                return Err(EquityError::NotEnoughCards(hand.num_cards()).into());
            }
            // A card below a six is not in this deck at all.
            for card in hand.cards() {
                if !self.deck().contains(*card) {
                    return Err(EquityError::Infeasible(format!(
                        "{}, which is not in a short deck",
                        card
                    ))
                    .into());
                }
            }
        }
        for card in calculator.community_cards() {
            if !self.deck().contains(*card) {
                return Err(EquityError::Infeasible(format!(
                    "{}, which is not in a short deck",
                    card
                ))
                .into());
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
