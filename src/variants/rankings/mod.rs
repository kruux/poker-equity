mod deuce_seven;
mod fast;
mod hand_rank_table;
mod hi_lo;
mod high;
mod low;
mod rank_translation;
mod low_qualifier;
mod short_deck;
mod table_index;

pub type StudHandRank = HighHandRank;
pub type RazzHandRank = LowHandRank;
pub type BadugiHandRank = LowHandRank;
pub type StudHiLoHandRank = HiLoHandRank;
pub type OmahaHiLoHandRank = HiLoHandRank;
pub type HoldemHandRank = HighHandRank;
pub type OmahaHandRank = HighHandRank;
pub use deuce_seven::DeuceSevenRank;
pub use fast::{
    deuce_seven_score, high_score, low_a5_score, short_deck_score, FastHandRank, FLUSH_KEYS,
    RANK_KEYS,
};
pub(crate) use hand_rank_table::{
    DEUCE_SEVEN_FLUSH_SCORES, DEUCE_SEVEN_HAND_SCORES, HAND_DISPLACEMENTS, HIGH_FLUSH_SCORES,
    HIGH_HAND_SCORES, LOW_A5_HAND_SCORES, SHORT_DECK_FLUSH_SCORES, SHORT_DECK_HAND_SCORES,
};
pub use hi_lo::HiLoHandRank;
pub use high::HighHandRank;
pub use low::LowHandRank;
pub use rank_translation::{fast_to_high, high_to_fast};
pub use short_deck::ShortDeckRank;
pub(crate) use low_qualifier::EIGHT_OR_BETTER_LIMIT;

