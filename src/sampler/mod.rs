//! Drawing deals, and deciding up front whether a request has any.
//!
//! Two things need care. A hand is an unordered set, so the sampler must be
//! uniform over sets rather than over ordered assignments; and a request no
//! deal can satisfy must be refused up front rather than spun on.

mod draw;
mod matching;

pub use draw::SlotSampler;
pub use matching::{has_perfect_matching, is_feasible, maximum_matching};

#[cfg(test)]
mod tests {
    mod draw_tests;
    mod matching_tests;
}
