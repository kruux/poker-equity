mod badugi;
mod deuce_seven;
mod equity;
mod holdem;
mod omaha;
mod omaha_hi_lo;
mod rankings;
mod razz;
mod short_deck;
mod stud;
mod stud_hi_lo;

pub use badugi::Badugi;
pub use deuce_seven::DeuceSeven;
pub(crate) use equity::CommunityCardGame;
pub use equity::EquityCalculation;
pub use equity::HasLow;
pub use equity::{SeatIter, Seats};
pub use holdem::Holdem;
pub use omaha::{Courchevel, Omaha, OmahaFive, OmahaSix};
pub use omaha_hi_lo::{CourchevelHiLo, OmahaFiveHiLo, OmahaHiLo};
pub use rankings::{deuce_seven_score, high_score, low_a5_score, short_deck_score};
pub use rankings::fast_to_high;
pub use rankings::high_to_fast;
pub use rankings::{
    DeuceSevenRank, FastHandRank, HiLoHandRank, HighHandRank, LowHandRank, OmahaHandRank,
    OmahaHiLoHandRank, BadugiHandRank, ShortDeckRank,
    RazzHandRank, StudHandRank, StudHiLoHandRank, FLUSH_KEYS, RANK_KEYS,
};
pub use razz::Razz;
pub use short_deck::ShortDeck;
pub use stud::Stud;
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

    /// The cards this game is played with. Every variant but short deck uses
    /// the full fifty-two.
    fn deck(&self) -> crate::cards::CardSet {
        crate::cards::CardSet::FULL_DECK
    }

    /// The largest number of cards a hand can hold, hole cards and board
    /// together. Used to size hands and to reject overfull ones.
    fn max_cards(&self) -> usize;

    /// How many cards a player holds privately.
    fn hole_cards(&self) -> usize;

    /// How many cards the shared board holds, or zero where there is none.
    fn board_cards(&self) -> usize {
        0
    }

    /// Names a hand. `cards` holds the private cards first and any shared
    /// board after them, and may be short, in which case the result is an
    /// incomplete rank that loses to any complete hand.
    ///
    /// This is the readable answer, for showing a player what they have. The
    /// sampling loop uses [`score`](Self::score) instead.
    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank;

    /// Where a hand stands against others, as a single number, **lower being
    /// better**.
    ///
    /// This is what the sampling loop compares, and for most games it is a
    /// lookup rather than a walk through the hand. Keeping it apart from
    /// [`evaluate_hand`](Self::evaluate_hand) is what lets the loop read a
    /// table while the readable form stays available -- and stays the
    /// independent thing the tables are checked against.
    fn score(&self, cards: &[Card]) -> u32;

    /// The low half's score, or `None` when the hand has no qualifying low.
    ///
    /// Only split games have a low half, so the default is `None`.
    fn low_score(&self, _cards: &[Card]) -> Option<u32> {
        None
    }

    /// The variant's display name.
    fn to_string(&self) -> String;

    /// A stable identifier for this game, for configuration files, database
    /// columns and anything else that needs to name a variant in text.
    ///
    /// The scheme is one word per idea, family first: `omaha`, `omaha_five`,
    /// `omaha_five_hi_lo`. Nothing starts with a digit, sizes are spelled out
    /// rather than abbreviated, and a split game always ends `_hi_lo`, so the
    /// keys sort into families and can be used as identifiers as they stand.
    fn key(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    mod badugi_tests;
    mod deuce_seven_tests;
    mod holdem_tests;
    mod omaha_hi_lo_tests;
    mod omaha_tests;
    mod razz_tests;
    mod short_deck_tests;
    mod stud_hi_lo_tests;
    mod stud_tests;
    mod variant_table_tests;
}
