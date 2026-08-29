use super::EquityCalculation;
use crate::{
    cards::Deck,
    error::{GameError, PokerError},
    hand::Hand,
    odds::EquityCalculator,
    variants::DeuceSeven,
};

impl EquityCalculation for DeuceSeven {
    fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        // Check that cards_to_discard are actually in the hand for all players
        for (_, hand, cards_to_discard) in calculator.players() {
            for card in cards_to_discard {
                if !hand.cards().contains(card) {
                    return Err(GameError::InvalidDiscard(*card).into());
                }
            }
        }
        Ok(())
    }

    fn run_single_simulation(
        &self,
        mut deck: Deck,
        calculator: &EquityCalculator<Self>,
        shares: &mut [f64],
    ) -> Result<(), PokerError> {
        let mut final_hands: Vec<Hand<DeuceSeven>> = Vec::with_capacity(shares.len());

        for (_, initial_hand, cards_to_discard) in calculator.players() {
            let mut current_cards = initial_hand.cards().to_vec();
            current_cards.retain(|card| !cards_to_discard.contains(card));

            for _ in current_cards.len()..5 {
                let card = deck.deal().ok_or(GameError::NotEnoughCards)?;
                current_cards.push(card);
            }

            final_hands.push(Hand::new_with_cards(DeuceSeven, current_cards)?);
        }

        self.award(&final_hands, shares)
    }
}

