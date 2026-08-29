//! The eight-or-better threshold, worked out by `build.rs` rather than
//! written down: it is the count of lows that qualify, and qualifying lows
//! take the lowest scores.
include!(concat!(env!("OUT_DIR"), "/low_qualifier.rs"));
