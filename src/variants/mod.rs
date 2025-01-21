mod deuce_seven;
mod equity;
mod holdem;
mod rankings;
mod razz;
mod stud;
mod stud_hi_lo;

pub use deuce_seven::DeuceSeven;
pub use equity::EquityCalculation;
pub(crate) use equity::HasLow;
pub use holdem::Holdem;
pub use rankings::{
    DeuceSevenRank, HiLoHandRank, HighHandRank, LowHandRank, RazzHandRank, StudHandRank,
    StudHiLoHandRank,
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
    mod razz_tests;
    mod stud_hi_lo_tests;
    mod stud_tests;
}
