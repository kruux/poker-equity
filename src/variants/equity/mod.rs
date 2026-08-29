
use crate::{cards::Deck, error::PokerError, hand::Hand, odds::EquityCalculator};

use super::{LowHandRank, PokerVariant};

mod badugi;
mod community_base;
mod deuce_seven;
mod holdem;
mod omaha;
mod omaha_hi_lo;
mod razz;
mod short_deck;
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

    /// Deals one complete hand from `deck` and *adds* each player's share of
    /// the pot to `shares`, which is indexed by seat in the order players were
    /// added. The shares from one deal sum to one, and a split game may add
    /// fractions other than halves.
    ///
    /// Adding rather than assigning lets the caller accumulate a whole chunk
    /// in one buffer, so nothing is allocated per deal.
    fn run_single_simulation(
        &self,
        deck: Deck,
        calculator: &EquityCalculator<Self>,
        shares: &mut [f64],
    ) -> Result<(), PokerError>;

    /// Adds each player's share of one deal's pot to `shares`, by seat.
    ///
    /// The default gives the whole pot to the best hand, split evenly among
    /// ties. A split game overrides this to divide the halves, and must
    /// compute the shares directly: quartering -- two players splitting the
    /// high while one of them also takes the low, for 75% and 25% -- is
    /// neither a win nor a tie in any countable sense.
    fn award(&self, hands: &[Hand<Self>], shares: &mut [f64]) -> Result<(), PokerError> {
        let winners = &self.rank_hands(hands)?[0];
        let share = 1.0 / winners.len() as f64;
        for &seat in winners {
            shares[seat] += share;
        }
        Ok(())
    }

    /// As [`award`](Self::award), and separately reports the low half alone.
    ///
    /// Only split games have a low half, so the default leaves `low_shares`
    /// untouched.
    fn award_detailed(
        &self,
        hands: &[Hand<Self>],
        shares: &mut [f64],
        _low_shares: &mut [f64],
    ) -> Result<(), PokerError> {
        self.award(hands, shares)
    }

    /// Splits the pot between the best high hand and the best qualifying
    /// low, each half shared among ties.
    ///
    /// With nobody qualifying for the low, the high hand takes it all. The
    /// shares are computed directly rather than derived from win and tie
    /// counts afterwards, because quartering -- two players splitting the
    /// high while one of them also takes the low, leaving 75% and 25% -- is
    /// neither a win nor a tie in any countable sense.
    fn award_hi_lo(
        &self,
        hands: &[Hand<Self>],
        shares: &mut [f64],
        low_shares: &mut [f64],
    ) -> Result<(), PokerError> {
        let high_half = match self.rank_low_hands(hands)? {
            Some(low_places) => {
                let winners = &low_places[0];
                let each = 0.5 / winners.len() as f64;
                for &seat in winners {
                    shares[seat] += each;
                    low_shares[seat] += each;
                }
                0.5
            }
            None => 1.0,
        };

        let winners = &self.rank_hands(hands)?[0];
        let each = high_half / winners.len() as f64;
        for &seat in winners {
            shares[seat] += each;
        }
        Ok(())
    }

    /// Places the players by hand strength, best first, as seat indices.
    ///
    /// Two dimensional so that ties are representable at any position:
    /// `result[0]` holds the winners, `result[1]` those in second, and so on.
    /// Players tie when they compare equal, which in a split game is not the
    /// same as their hands being identical.
    ///
    /// Each hand is scored once and the indices are sorted, rather than
    /// sorting the hands themselves -- comparing two `Hand`s rescores both.
    /// The score is a lookup for every game but badugi, so this is where the
    /// tables earn their keep.
    fn rank_hands(&self, hands: &[Hand<Self>]) -> Result<Vec<Vec<usize>>, PokerError> {
        if hands.is_empty() {
            return Ok(vec![]);
        }

        // Lower scores are better hands, so the seats sort ascending.
        let scores: Vec<u32> = hands.iter().map(|hand| self.score(hand.cards())).collect();
        let mut seats: Vec<usize> = (0..hands.len()).collect();
        seats.sort_by_key(|&seat| scores[seat]);

        Ok(group_ties(&seats, |a, b| scores[a] == scores[b]))
    }

    /// Places the players by their low hands, best first, in the same shape
    /// `rank_hands` returns.
    ///
    /// Players without a qualifying low are left out entirely. Returns `None`
    /// when nobody qualifies, which is how a split game learns that the high
    /// hand takes the whole pot.
    fn rank_low_hands(
        &self,
        hands: &[Hand<Self>],
    ) -> Result<Option<Vec<Vec<usize>>>, PokerError> {
        let lows: Vec<Option<u32>> = hands
            .iter()
            .map(|hand| self.low_score(hand.cards()))
            .collect();

        let mut seats: Vec<usize> = (0..hands.len()).filter(|&i| lows[i].is_some()).collect();
        if seats.is_empty() {
            return Ok(None);
        }

        seats.sort_by_key(|&seat| lows[seat]);
        Ok(Some(group_ties(&seats, |a, b| lows[a] == lows[b])))
    }
}

/// Splits `seats`, already ordered best first, into groups of equal strength.
fn group_ties(seats: &[usize], ties: impl Fn(usize, usize) -> bool) -> Vec<Vec<usize>> {
    let mut places: Vec<Vec<usize>> = Vec::new();
    let mut current = vec![seats[0]];
    for window in seats.windows(2) {
        let (previous, seat) = (window[0], window[1]);
        if ties(previous, seat) {
            current.push(seat);
        } else {
            places.push(std::mem::take(&mut current));
            current.push(seat);
        }
    }
    places.push(current);
    places
}
