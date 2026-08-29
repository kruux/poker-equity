mod deuce_seven;
mod equity;
mod holdem;
mod holdem_fast;
mod omaha;
mod omaha_hi_lo;
mod omaha_fast;
mod rankings;
mod razz;
mod stud;
mod stud_hi_lo;

pub use deuce_seven::DeuceSeven;
pub(crate) use equity::CommunityCardGame;
pub use equity::EquityCalculation;
pub(crate) use equity::HasLow;
pub use holdem::Holdem;
pub use holdem_fast::HoldemFast;
pub use omaha::{Courchevel, Omaha, OmahaFive, OmahaSix};
pub use omaha_hi_lo::{CourchevelHiLo, OmahaFiveHiLo, OmahaHiLo};
pub use omaha_fast::OmahaFast;
pub use rankings::fast_to_high;
pub use rankings::high_to_fast;
pub use rankings::{
    DeuceSevenRank, FastHandRank, HiLoHandRank, HighHandRank, LowHandRank, OmahaHandRank,
    OmahaHiLoHandRank,
    RazzHandRank, StudHandRank, StudHiLoHandRank, FLUSH_KEYS, RANK_KEYS,
};
pub use razz::Razz;
pub use stud::SevenCardStud;
pub use stud_hi_lo::StudHiLo;

use crate::cards::Card;

/// How a variant deals its cards, which decides what the sampler has to fill
/// in for each simulation.
pub enum PokerType {
    /// Players hold a private hand and replace cards from the deck.
    Draw,
    /// Players hold a private hand of their own and share no cards.
    Stud,
    /// Players share a board and combine it with their hole cards.
    Community,
}

/// One poker variant: how many cards a hand holds and how to score it.
pub trait PokerVariant: Clone + Copy {
    /// The strength of a scored hand. Ordered best-last, so that `>` means
    /// "beats", whichever direction the underlying game ranks in.
    type HandRank: PartialOrd;

    /// How this variant deals.
    fn poker_type(&self) -> PokerType;

    /// The largest number of cards a hand can hold, hole cards and board
    /// together. Used to size hands and to reject overfull ones.
    fn max_cards(&self) -> usize;

    /// How many cards a player holds privately.
    fn hole_cards(&self) -> usize;

    /// How many cards the shared board holds, or zero where there is none.
    fn board_cards(&self) -> usize {
        0
    }

    /// Scores a hand. `cards` holds the private cards first and any shared
    /// board after them, and may be short, in which case the result is an
    /// incomplete rank that loses to any complete hand.
    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank;

    /// The variant's display name.
    fn to_string(&self) -> String;
}

#[cfg(test)]
mod tests {
    mod deuce_seven_tests;
    mod holdem_tests;
    mod omaha_hi_lo_tests;
    mod omaha_tests;
    mod razz_tests;
    mod stud_hi_lo_tests;
    mod stud_tests;
}
