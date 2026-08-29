//! The library's own notation: what a hand, a board and a set of dead cards
//! look like as text.
//!
//! Every pattern becomes a [`CardSet`](crate::cards::CardSet), so a named
//! card and a wildcard are the same kind of thing to everything downstream.
//! The grammar is specified in PLAN section 3 and pinned by the conformance
//! fixture in `tests/fixtures/notation.tsv`, which fpdb's own parser reads
//! too.

mod error;
mod hand_spec;
mod parser;
mod range;

pub use error::{NotationError, NotationErrorKind};
pub use hand_spec::HandSpec;
pub use parser::{parse_board, parse_dead, parse_hand, parse_hand_up_to};

#[cfg(test)]
mod tests {
    mod conformance_tests;
    mod parser_tests;
}
