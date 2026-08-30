
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

/// What the equity engine needs from a game beyond how to score a hand:
/// how to check a request makes sense, how to deal one hand, and how to split
/// the pot afterwards.
///
/// Most of it has a default that suits every game that awards the whole pot
/// to one hand; a split game overrides the awarding.
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
        let winners = self.winning_seats(hands);
        if winners.is_empty() {
            return Ok(());
        }
        let share = 1.0 / winners.len() as f64;
        for seat in winners {
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
        let low_winners = self.best_low_seats(hands);
        let high_half = if low_winners.is_empty() {
            1.0
        } else {
            let each = 0.5 / low_winners.len() as f64;
            for seat in low_winners {
                shares[seat] += each;
                low_shares[seat] += each;
            }
            0.5
        };

        let winners = self.winning_seats(hands);
        if winners.is_empty() {
            return Ok(());
        }
        let each = high_half / winners.len() as f64;
        for seat in winners {
            shares[seat] += each;
        }
        Ok(())
    }

    /// The seats holding the best hand, which is the set that shares the pot.
    ///
    /// One pass over the table, and nothing allocated: only the best score
    /// decides anything, so there is no reason to place the rest of the
    /// seats. Each hand is scored once rather than compared -- comparing two
    /// `Hand`s rescores both -- and the score is a table lookup for every
    /// game but badugi, which is where the tables earn their keep.
    ///
    /// Returns an empty set for an empty table.
    fn winning_seats(&self, hands: &[Hand<Self>]) -> Seats {
        let mut best = u32::MAX;
        let mut winners = Seats::NONE;
        for (seat, hand) in hands.iter().enumerate() {
            // Lower scores are better hands.
            let score = self.score(hand.cards());
            if score < best {
                best = score;
                winners = Seats::only(seat);
            } else if score == best {
                winners.add(seat);
            }
        }
        winners
    }

    /// The seats holding the best qualifying low, which share the low half.
    ///
    /// Seats without a qualifying low are not in the set, so an empty set is
    /// how a split game learns that nobody made a low and the high hand takes
    /// the whole pot.
    fn best_low_seats(&self, hands: &[Hand<Self>]) -> Seats {
        let mut best = u32::MAX;
        let mut winners = Seats::NONE;
        for (seat, hand) in hands.iter().enumerate() {
            let Some(score) = self.low_score(hand.cards()) else {
                continue;
            };
            if score < best {
                best = score;
                winners = Seats::only(seat);
            } else if score == best {
                winners.add(seat);
            }
        }
        winners
    }
}

/// A set of seats at one table, as one bit each.
///
/// The deal loop asks who won millions of times over, so the answer has to
/// come back without touching the allocator. Thirty-two bits is room to
/// spare: every game here deals a player at least two cards, so fifty-two
/// cards cannot seat more than twenty-six.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Seats(u32);

impl Seats {
    /// Nobody.
    pub const NONE: Self = Seats(0);

    /// Just the one seat.
    pub fn only(seat: usize) -> Self {
        debug_assert!(seat < 32, "seat {} is past the width of the set", seat);
        Seats(1 << seat)
    }

    /// Adds a seat to the set.
    pub fn add(&mut self, seat: usize) {
        debug_assert!(seat < 32, "seat {} is past the width of the set", seat);
        self.0 |= 1 << seat;
    }

    /// Whether the seat is in the set.
    pub fn contains(&self, seat: usize) -> bool {
        seat < 32 && self.0 >> seat & 1 == 1
    }

    /// How many seats are in the set.
    pub fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    /// Whether the set names nobody at all.
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl IntoIterator for Seats {
    type Item = usize;
    type IntoIter = SeatIter;

    fn into_iter(self) -> SeatIter {
        SeatIter(self.0)
    }
}

/// Walks the seats of a [`Seats`] set in order, lowest first.
pub struct SeatIter(u32);

impl Iterator for SeatIter {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.0 == 0 {
            return None;
        }
        let seat = self.0.trailing_zeros() as usize;
        // Clears the lowest bit that is set.
        self.0 &= self.0 - 1;
        Some(seat)
    }
}
