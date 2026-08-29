use std::cmp::Ordering;
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

/// Implemented by hand ranks that carry a low half, so that split games can
/// ask for it without knowing the concrete rank type.
pub trait HasLow {
    /// The qualifying low, or `None` when the hand has none.
    fn low(&self) -> Option<&LowHandRank>;
}

pub trait EquityCalculation: PokerVariant
where
    Self: Sized,
{
    /// Rejects a request this variant cannot simulate, before any sampling
    /// starts. Each variant checks its own hand sizes and board rules on top
    /// of the shared checks in `EquityCalculator::validate_base`.
    fn validate(&self, calculator: &EquityCalculator<Self>) -> Result<(), PokerError>;

    /// Deals one complete hand from `deck` and returns each player's share of
    /// the pot, keyed by name. Shares sum to one, and a split game may return
    /// fractions other than halves.
    fn run_single_simulation(
        &self,
        deck: Deck,
        calculator: &EquityCalculator<Self>,
    ) -> Result<HashMap<String, f64>, PokerError>;

    /// Places the players by hand strength, best first.
    ///
    /// Two dimensional so that ties are representable at any position:
    /// `result[0]` holds the winners, `result[1]` those in second, and so on.
    /// Players tie when they compare equal, which in a split game is not the
    /// same as their hands being identical.
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

            // Group by the same relation the sort used, not by `==`. In a
            // split game those differ: two hands can rank equally for high
            // while holding different lows, and `==` would then split them
            // into separate places and hand one of them the whole high half.
            if prev_hand.partial_cmp(curr_hand) == Some(Ordering::Equal) {
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

    /// Places the players by their low hands, best first, in the same shape
    /// `rank_hands` returns.
    ///
    /// Players without a qualifying low are left out entirely. Returns `None`
    /// when nobody qualifies, which is how a split game learns that the high
    /// hand takes the whole pot.
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
            // As in `rank_hands`: group by the relation that sorted them.
            if curr_hand.partial_cmp(prev_hand) == Some(Ordering::Equal) {
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
