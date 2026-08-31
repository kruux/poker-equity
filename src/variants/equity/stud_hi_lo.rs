use super::EquityCalculation;
use crate::{error::PokerError, hand::Hand, variants::StudHiLo};

/// Seven-card stud hi/lo splits its pot, so it overrides the awarding. The
/// dealing and the scoring are the same as plain stud's.
impl EquityCalculation for StudHiLo {
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
}
