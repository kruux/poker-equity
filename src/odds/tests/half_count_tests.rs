//! Each half of a split pot, counted on its own.
//!
//! In a hi/lo game a seat's equity cannot say how it is made: taking every
//! low and never the high is half the pot, and so is splitting both halves
//! every time. The per-half counts are what tell those apart, so the spots
//! here are chosen in pairs that share an equity and differ in the halves.
//!
//! Every river spot is one deal, so each count is a whole deal or nothing,
//! and every figure was worked out by hand from the hands as written.

use crate::{
    error::PokerError,
    odds::{run_chunk, run_exact, ChunkResult, EquityRequest},
    variants::{EquityCalculation, Holdem, OmahaHiLo, PokerVariant, StudHiLo},
};

/// Walks a spot outright, panicking if it is too large to.
fn walked<V>(variant: V, hands: &[&str], board: &str) -> Result<ChunkResult, PokerError>
where
    V: PokerVariant + EquityCalculation,
{
    let request = EquityRequest::from_text(variant, hands, board, "")?;
    Ok(run_exact(&request)?.expect("small enough to walk"))
}

/// One seat's four half counts, in the order high win, high tie, low win,
/// low tie.
fn halves(result: &ChunkResult, seat: usize) -> [f64; 4] {
    [
        result.high_win_count[seat],
        result.high_tie_count[seat],
        result.low_win_count[seat],
        result.low_tie_count[seat],
    ]
}

/// The case the counts exist for: two seats on half the pot each, one by
/// taking the low and nothing else, the other by taking the high.
#[test]
fn test_omaha_half_the_pot_from_one_half() -> Result<(), PokerError> {
    // Ace-four makes 7-4-3-2-A for the only low. Kings full is the high:
    // king-queen uses the board's pair for trips, and the ace-four seat's
    // best high is only the board's kings with its ten-nine.
    let result = walked(OmahaHiLo, &["Ac4d9sTs", "KdQcJcJd"], "2c 3d 7h Kh Ks")?;
    let equities = result.equities();
    assert_eq!(equities[0].equity, 0.5);
    assert_eq!(equities[1].equity, 0.5);
    assert_eq!(halves(&result, 0), [0.0, 0.0, 1.0, 0.0], "the low alone");
    assert_eq!(halves(&result, 1), [1.0, 0.0, 0.0, 0.0], "the high alone");
    Ok(())
}

/// The same half pot each, from splitting both halves.
#[test]
fn test_omaha_half_the_pot_from_splitting_both() -> Result<(), PokerError> {
    // Mirror images: the same ace-four low and the same trip kings with a
    // queen, in different suits, so both halves divide.
    let result = walked(OmahaHiLo, &["Ac4dKdQc", "As4cKcQd"], "2c 3d 7h Kh Ks")?;
    let equities = result.equities();
    assert_eq!(equities[0].equity, 0.5);
    assert_eq!(equities[1].equity, 0.5);
    for seat in 0..2 {
        assert_eq!(halves(&result, seat), [0.0, 1.0, 0.0, 1.0], "seat {seat}");
    }
    Ok(())
}

/// Quartered: the high shared, the low taken alone by one of the two.
#[test]
fn test_omaha_quartered() -> Result<(), PokerError> {
    // Both seats play a king and an ace for trip kings, ace and seven. Both
    // make a low too, but 7-4-3-2-A beats 8-7-4-3-A, so the ace-deuce takes
    // the low alone: 75% and 25%.
    let result = walked(OmahaHiLo, &["Ac2dKdQc", "KcAd8s8d"], "3c 4d 7h Kh Ks")?;
    let equities = result.equities();
    assert_eq!(equities[0].equity, 0.75);
    assert_eq!(equities[1].equity, 0.25);
    assert_eq!(
        halves(&result, 0),
        [0.0, 1.0, 1.0, 0.0],
        "a shared high, the low"
    );
    assert_eq!(
        halves(&result, 1),
        [0.0, 1.0, 0.0, 0.0],
        "a shared high only"
    );
    Ok(())
}

/// With no low on the board the high takes the whole pot, which counts as
/// winning the high half -- and nobody has anything on the low.
#[test]
fn test_omaha_without_a_low_the_high_is_everything() -> Result<(), PokerError> {
    // Three cards above eight on the board: no low is possible.
    let result = walked(OmahaHiLo, &["AcAd2c3d", "KcKd4c5d"], "9h Th Js 2s 3h")?;
    assert_eq!(result.equities()[0].equity, 1.0);
    assert_eq!(halves(&result, 0), [1.0, 0.0, 0.0, 0.0]);
    assert_eq!(halves(&result, 1), [0.0, 0.0, 0.0, 0.0]);
    assert_eq!(result.high_win_count[0], result.scoop_count[0]);
    Ok(())
}

/// Stud hi/lo tells the same two half pots apart.
#[test]
fn test_stud_half_the_pot_either_way() -> Result<(), PokerError> {
    // A six-low with no pair against queens full with no low.
    let apart = walked(StudHiLo, &["Ah2c3d4s6h9cKd", "QsQdQc8h8dTsJs"], "")?;
    assert_eq!(apart.equities()[0].equity, 0.5);
    assert_eq!(halves(&apart, 0), [0.0, 0.0, 1.0, 0.0], "the low alone");
    assert_eq!(halves(&apart, 1), [1.0, 0.0, 0.0, 0.0], "the high alone");

    // The same six-low and the same kings, suit for suit different.
    let together = walked(StudHiLo, &["Ah2c3d4s6hKcKd", "As2d3c4h6dKsKh"], "")?;
    assert_eq!(together.equities()[0].equity, 0.5);
    for seat in 0..2 {
        assert_eq!(halves(&together, seat), [0.0, 1.0, 0.0, 1.0], "seat {seat}");
    }
    Ok(())
}

/// Outside split games the high half is the pot, so the high counts are the
/// win and tie counts and the low counts are zero -- including on the
/// weighted path, where every count carries the deal's weight.
#[test]
fn test_a_game_without_a_low_counts_only_the_high() -> Result<(), PokerError> {
    let ordinary = EquityRequest::from_text(Holdem, &["AhKh", "QsQd", "7c6c"], "", "")?;
    let weighted = EquityRequest::from_text(Holdem, &["c c"; 5], "", "")?;
    assert!(weighted.is_weighted(), "this spot is dealt weighted");

    for result in [
        run_chunk(&ordinary, 20_000, 7)?,
        run_chunk(&weighted, 20_000, 7)?,
    ] {
        assert_eq!(result.high_win_count, result.win_count);
        assert_eq!(result.high_tie_count, result.tie_count);
        assert!(result.low_win_count.iter().all(|&count| count == 0.0));
        assert!(result.low_tie_count.iter().all(|&count| count == 0.0));
    }
    Ok(())
}

/// Over many sampled deals the counts agree with the low shares, which are
/// summed separately: a seat's low share is at least half a pot for each
/// low it took alone, and at most half a pot for each it took any of.
#[test]
fn test_sampled_halves_agree_with_the_low_shares() -> Result<(), PokerError> {
    let request =
        EquityRequest::from_text(OmahaHiLo, &["Ac2dKdQc", "As3sJhTh", "8h8c9d9s"], "", "")?;
    let result = run_chunk(&request, 50_000, 3)?;
    for seat in 0..3 {
        let low = result.low_share_sum[seat];
        let alone = result.low_win_count[seat];
        let any = alone + result.low_tie_count[seat];
        assert!(
            low >= 0.5 * alone - 1e-6,
            "seat {seat}: {low} below {alone} lows"
        );
        assert!(
            low <= 0.5 * any + 1e-6,
            "seat {seat}: {low} above {any} lows"
        );
        assert!(result.scoop_count[seat] <= result.high_win_count[seat]);
        assert!(
            result.high_win_count[seat] + result.high_tie_count[seat] <= result.weight_sum,
            "seat {seat} won more highs than there were deals"
        );
    }
    let outright: f64 = result.high_win_count.iter().sum();
    assert!(
        outright <= result.weight_sum,
        "one outright high a deal at most"
    );
    Ok(())
}
