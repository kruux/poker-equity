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

/// Seven-card stud is scored as an ordinary high hand.
pub type StudHandRank = HighHandRank;
/// Razz is scored as an ace-to-five low, with no qualifier.
pub type RazzHandRank = LowHandRank;
/// A badugi is compared the same way a low is: more cards first, then lower.
pub type BadugiHandRank = LowHandRank;
/// Stud hi/lo carries a high hand and, when one qualifies, a low.
pub type StudHiLoHandRank = HiLoHandRank;
/// Omaha hi/lo carries both halves, each found over its own pairings.
pub type OmahaHiLoHandRank = HiLoHandRank;
/// Hold'em is scored as an ordinary high hand.
pub type HoldemHandRank = HighHandRank;
/// Omaha names its hand the same way, once the best pairing is found.
pub type OmahaHandRank = HighHandRank;
pub use deuce_seven::DeuceSevenRank;
pub use fast::{
    deuce_seven_score, high_score, high_score_from_parts, low_a5_score, low_a5_score_from_parts,
    rank_key, shared_suit, short_deck_score, FastHandRank, FLUSH_KEYS, RANK_KEYS,
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

