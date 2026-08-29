mod deuce_seven;
mod equity;
mod holdem;
mod holdem_fast;
mod omaha;
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
pub use omaha::Omaha;
pub use omaha_fast::OmahaFast;
pub use rankings::fast_to_high;
pub use rankings::high_to_fast;
pub use rankings::{
    DeuceSevenRank, FastHandRank, HiLoHandRank, HighHandRank, LowHandRank, OmahaHandRank,
    RazzHandRank, StudHandRank, StudHiLoHandRank, FLUSH_KEYS, RANK_KEYS,
};
pub use razz::Razz;
pub use stud::SevenCardStud;
pub use stud_hi_lo::StudHiLo;

use crate::cards::Card;

pub enum PokerType {
    Draw,      // 5 card draw
    Stud,      // Stud games
    Community, // Community card games
}

pub trait PokerVariant: Clone + Copy {
    type HandRank: PartialOrd;

    fn poker_type(&self) -> PokerType;
    fn max_cards(&self) -> usize;
    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank;
    fn to_string(&self) -> String;
}

#[cfg(test)]
mod tests {
    mod deuce_seven_tests;
    mod holdem_tests;
    mod omaha_tests;
    mod razz_tests;
    mod stud_hi_lo_tests;
    mod stud_tests;
}
