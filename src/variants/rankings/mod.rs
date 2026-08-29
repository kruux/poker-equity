mod deuce_seven;
mod fast;
mod hand_rank_table;
mod hi_lo;
mod high;
mod low;
mod rank_translation;
mod short_deck;

pub type StudHandRank = HighHandRank;
pub type RazzHandRank = LowHandRank;
pub type BadugiHandRank = LowHandRank;
pub type StudHiLoHandRank = HiLoHandRank;
pub type OmahaHiLoHandRank = HiLoHandRank;
pub type HoldemHandRank = HighHandRank;
pub type OmahaHandRank = HighHandRank;
pub use deuce_seven::DeuceSevenRank;
pub use fast::{FastHandRank, FLUSH_KEYS, RANK_KEYS};
pub(crate) use hand_rank_table::{FLUSH_RANKS, HAND_RANKS};
pub use hi_lo::HiLoHandRank;
pub use high::HighHandRank;
pub use low::LowHandRank;
pub use rank_translation::{fast_to_high, high_to_fast};
pub use short_deck::ShortDeckRank;

