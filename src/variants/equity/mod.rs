use std::collections::HashMap;

use crate::{cards::Deck, error::PokerError, hand::Hand, odds::EquityCalculator};

use super::PokerVariant;

mod deuce_seven;
mod razz;
mod stud;
mod stud_base;

pub(crate) use stud_base::StudEquity;

pub trait EquityCalculation: PokerVariant
where
    Self: Sized,
{
    fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError>;

    fn run_single_simulation(
        &self,
        deck: Deck,
        calculator: &EquityCalculator<Self>,
    ) -> Result<HashMap<String, f64>, PokerError>;

    /// Returns a 2 dimensional array. It's 2 dimensional to handle any ties in any position.
    /// First position contain a vec with the winners.
    /// Second index a vec with the players in second.
    fn rank_hands(&self, hands: &[(String, Hand<Self>)]) -> Result<Vec<Vec<String>>, PokerError> {
        if hands.is_empty() {
            return Ok(vec![]);
        }

        // Sort hands and group by strength
        let mut sorted_hands = hands.to_vec();
        sorted_hands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Create result vec and add the player in first place to another vec
        // Don't add the winner until we make sure no other has an equal hand
        let mut result: Vec<Vec<String>> = vec![];
        let mut current_group: Vec<String> = vec![sorted_hands[0].0.clone()];

        // Compare the next hands with previous.
        for i in 1..sorted_hands.len() {
            let prev_hand = &sorted_hands[i - 1].1;
            let curr_hand = &sorted_hands[i].1;

            if prev_hand == curr_hand {
                // Tied with previous hand. Add to current vec
                current_group.push(sorted_hands[i].0.clone());
            } else {
                // Last hand was stronger. Push vector and create a new with current hand
                result.push(current_group);
                current_group = vec![sorted_hands[i].0.clone()];
            }
        }
        // Add last vec to result
        result.push(current_group);

        Ok(result)
    }
}
