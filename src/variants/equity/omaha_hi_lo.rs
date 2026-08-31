use super::EquityCalculation;
use crate::{
    error::PokerError,
    hand::Hand,
    variants::{
        omaha::best_seats,
        rankings::{high_score_from_parts, low_a5_score_from_parts, EIGHT_OR_BETTER_LIMIT},
        CourchevelHiLo, OmahaFiveHiLo, OmahaHiLo, PokerVariant, Seats,
    },
};

/// Wires a split-pot Omaha variant into the equity engine: the shared board
/// as in [`super::omaha`], and a low half to hand out on top of it.
macro_rules! omaha_hi_lo_equity {
    ($name:ident) => {
        impl EquityCalculation for $name {
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

            /// Every seat is playing the same board, so its three-card halves
            /// are worked out once for the table instead of once a seat.
            fn winning_seats(&self, hands: &[Hand<Self>]) -> Seats {
                best_seats(hands, self.hole_cards(), high_score_from_parts, |_| true)
            }

            /// The low half of the same table. Suits never matter to a low, so
            /// the flush half of a pairing is ignored; and a qualifying low is
            /// the best kind of low there is, so the eight-or-better rule is
            /// one comparison against the threshold.
            fn best_low_seats(&self, hands: &[Hand<Self>]) -> Seats {
                best_seats(
                    hands,
                    self.hole_cards(),
                    |key, _| low_a5_score_from_parts(key),
                    |score| score < EIGHT_OR_BETTER_LIMIT as u32,
                )
            }
        }
    };
}

omaha_hi_lo_equity!(OmahaHiLo);
omaha_hi_lo_equity!(OmahaFiveHiLo);
omaha_hi_lo_equity!(CourchevelHiLo);
