use super::EquityCalculation;
use crate::{
    cards::Deck,
    error::{GameError, PokerError},
    hand::Hand,
    odds::EquityCalculator,
    variants::Badugi,
};

impl EquityCalculation for Badugi {
    fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError> {
        for (_, hand, cards_to_discard) in calculator.players() {
            for card in cards_to_discard {
                if !hand.cards().contains(card) {
                    return Err(GameError::InvalidDiscard(*card).into());
                }
            }
        }
        Ok(())
    }

    /// One draw, as for every draw game here: each player discards once and
    /// takes replacements from what is left. Triple draw needs a drawing
    /// strategy -- when to stand pat, when to break -- that no caller can
    /// reasonably be asked to supply, so it is out of scope.
    fn run_single_simulation(
        &self,
        mut deck: Deck,
        calculator: &EquityCalculator<Self>,
        shares: &mut [f64],
    ) -> Result<(), PokerError> {
        let mut final_hands: Vec<Hand<Badugi>> = Vec::with_capacity(shares.len());

        for (_, initial_hand, cards_to_discard) in calculator.players() {
            let mut cards = initial_hand.cards().to_vec();
            cards.retain(|card| !cards_to_discard.contains(card));

            for _ in cards.len()..4 {
                cards.push(deck.deal().ok_or(GameError::NotEnoughCards)?);
            }

            final_hands.push(Hand::new_with_cards(Badugi, cards)?);
        }

        self.award(&final_hands, shares)
    }
}
