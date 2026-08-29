use std::collections::HashMap;

use crate::{cards::Deck, error::PokerError, hand::Hand, odds::EquityCalculator};

use super::{LowHandRank, PokerVariant};

mod community_base;
mod deuce_seven;
mod holdem;
mod holdem_fast;
mod omaha;
mod omaha_fast;
mod razz;
mod stud;
mod stud_base;
mod stud_hi_lo;

pub(crate) use community_base::CommunityCardGame;
pub(crate) use stud_base::StudCardGame;

pub trait HasLow {
    fn low(&self) -> Option<&LowHandRank>;
}

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

    /// ## Only works for HiLo variants that implement HasLow.
    /// Returns an Option with a 2 dimensional array. It's 2 dimensional to handle any ties in any position.
    /// First position contain a vec with the winners.
    /// Second index a vec with the players in second.
    /// In case of no low hands, return None.
    fn rank_low_hands(
        &self,
        hands: &[(String, Hand<Self>)],
    ) -> Result<Option<Vec<Vec<String>>>, PokerError>
    where
        Self::HandRank: HasLow,
    {
        // Filter out low hands
        let mut low_hands: Vec<(String, LowHandRank)> = hands
            .iter()
            .filter_map(|(name, hand)| {
                hand.evaluate()
                    .low()
                    .cloned()
                    .map(|low| (name.clone(), low.clone()))
            })
            .collect();

        if low_hands.is_empty() {
            return Ok(None);
        }

        // Sort by strength, lower is better
        low_hands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Check for multiple winners and second places etc.
        let mut result: Vec<Vec<String>> = vec![];
        let mut current_group: Vec<String> = vec![low_hands[0].0.clone()];

        for i in 1..low_hands.len() {
            // .0 is player name, .1 is low hand rank
            let prev_hand = &low_hands[i - 1].1;
            let curr_hand = &low_hands[i].1;
            if curr_hand == prev_hand {
                current_group.push(low_hands[i].0.clone());
            } else {
                result.push(current_group);
                current_group = vec![low_hands[i].0.clone()];
            }
        }
        // Add last vec to result
        result.push(current_group);
        Ok(Some(result))
    }
}
