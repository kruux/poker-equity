//! The one place the build's own optimisation is asserted.
//!
//! Performance gates in CI are flaky and end up muted, which is worse than
//! not having them, so `cargo run --release --bin benchmark` reports numbers
//! and asserts nothing. This checks the one thing a timing was ever a proxy
//! for -- that the optimiser was involved at all -- by asking cargo instead
//! of a stopwatch.
//!
//! It used to measure hands a second and compare against a floor. A floor
//! has to sit above every unoptimised build and below every optimised one,
//! on hardware nobody has seen, which made it a number tuned to whichever
//! machine last complained. `OPT_LEVEL` is the fact itself: cargo sets it for
//! the build script, `build.rs` writes it down, and this reads it.

include!(concat!(env!("OUT_DIR"), "/opt_level.rs"));

/// Fails if this build is unoptimised, whatever machine it runs on.
#[test]
fn test_the_build_was_optimised() {
    assert_ne!(
        OPT_LEVEL, "0",
        "built at opt-level 0 -- the equity tests deal a hundred thousand \
         hands apiece and want minutes rather than seconds like this"
    );
}
