//! The one place a timing is asserted.
//!
//! Performance gates in CI are flaky and end up muted, which is worse than
//! not having them, so `cargo run --release --bin benchmark` reports numbers
//! and asserts nothing. The single exception is this floor, which exists to
//! catch an unoptimised build being shipped as a release one. It sits far
//! below anything a real build produces, so it can only fail if the
//! optimiser was not involved at all.

use std::time::Instant;

use poker_equity::{
    cards::Card,
    variants::{Holdem, PokerVariant},
};

/// Hands a second below which the build cannot plausibly be optimised.
///
/// Measured on one desktop core: about 1.8 M/s at opt-level 3, 1.4 M/s at the
/// 2 the test profile uses, and 127 k/s unoptimised. The gap is a factor of
/// eleven, and this floor sits in the middle of it rather than just under the
/// optimised figure, because a CI runner is the slowest machine that will
/// ever run this — the two-core ones manage 650 k/s optimised, which is still
/// twice the floor and five times what they would manage without.
const FLOOR: f64 = 300_000.0;

#[test]
fn test_the_build_was_optimised() {
    let seven = Card::parse_field("Ah Kh Qs Qd 2c 7d 9s").expect("valid cards");

    // Warm the tables so the measurement is of evaluation, not of the first
    // page faults.
    for _ in 0..10_000 {
        std::hint::black_box(Holdem.evaluate_hand(&seven));
    }

    let rounds = 500_000;
    let started = Instant::now();
    for _ in 0..rounds {
        std::hint::black_box(Holdem.evaluate_hand(&seven));
    }
    let per_second = rounds as f64 / started.elapsed().as_secs_f64();

    assert!(
        per_second > FLOOR,
        "only {:.0} hands a second, which is below the {:.0} an optimised \
         build cannot fail to beat -- this looks like a debug build",
        per_second,
        FLOOR
    );
}
