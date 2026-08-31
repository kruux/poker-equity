//! An equity engine for poker.
//!
//! Given a game, some hands -- which may be only partly known -- a board and
//! some dead cards, it reports what share of the pot each player wins.
//!
//! ```no_run
//! use poker_equity::{odds::{equity, EquityRequest, Target}, variants::Holdem};
//!
//! let request = EquityRequest::from_text(Holdem, &["AhAd", "KsKc"], "2c 7d 9h", "")?;
//! let result = equity(&request, Target::Exact)?;
//! assert_eq!(format!("{:.2}%", result.equities()[0].percent()), "91.62%");
//! # Ok::<(), poker_equity::error::PokerError>(())
//! ```
//!
//! See `README.md` for the notation and the rest of the API.

pub mod cards;
pub mod error;
pub mod hand;
pub mod notation;
pub mod odds;
#[cfg(feature = "python")]
pub mod py;
pub mod sampler;
pub mod variants;
