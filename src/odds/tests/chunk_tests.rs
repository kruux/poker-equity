use crate::error::{EquityError, PokerError};
use crate::odds::{run_chunk, run_exact, ChunkResult, EquityRequest};
use crate::variants::Holdem;

/// Enumerates a spot exactly, panicking if it is too large.
fn exact(hands: &[&str], board: &str, dead: &str) -> ChunkResult {
    let request = EquityRequest::from_text(Holdem, hands, board, dead).unwrap();
    run_exact(&request)
        .unwrap()
        .expect("this spot is small enough to enumerate")
}

fn sampled(hands: &[&str], board: &str, dead: &str, samples: u64, seed: u64) -> ChunkResult {
    let request = EquityRequest::from_text(Holdem, hands, board, dead).unwrap();
    run_chunk(&request, samples, seed).unwrap()
}

/// Published equities, asserted in exact mode where there is no error bar to
/// hide behind. Every figure here is the whole enumeration of the spot.
#[test]
fn test_published_reference_equities() {
    // The board cases enumerate in a thousand deals or fewer.
    let cases: [(&[&str], &str, f64); 3] = [
        (&["AhAd", "KsKc"], "2c 7d 9h", 91.6162),
        (&["AhAd", "KsKc"], "2c 7d 9h Ts", 95.4545),
        (&["AhKh", "QsJs"], "Th 9h 2c", 70.8081),
    ];
    for (hands, board, expected) in cases {
        let equities = exact(hands, board, "").equities();
        assert!(
            (equities[0].percent() - expected).abs() < 0.05,
            "{} on {} gave {:.4}%, expected {:.4}%",
            hands.join(" vs "),
            board,
            equities[0].percent(),
            expected
        );
        assert_eq!(equities[0].std_error, 0.0, "an exact result has no error bar");
    }
}

/// The preflop matchups everyone knows, enumerated in full. Slow enough to
/// keep out of the default run, definitive enough to be worth having.
#[test]
#[ignore = "enumerates 1.7M boards per case; run with --ignored"]
fn test_published_preflop_equities() {
    let cases: [(&[&str], f64); 5] = [
        (&["AhAd", "KsKc"], 81.2555),
        (&["AhAs", "KhKs"], 82.6366),
        (&["AhAd", "7s8s"], 76.9791),
        (&["AhKh", "2c2d"], 50.0842),
        (&["AhKs", "2c2d"], 46.9597),
    ];
    for (hands, expected) in cases {
        let equities = exact(hands, "", "").equities();
        assert!(
            (equities[0].percent() - expected).abs() < 0.05,
            "{} gave {:.4}%, expected {:.4}%",
            hands.join(" vs "),
            equities[0].percent(),
            expected
        );
    }
}

/// Exact against Monte Carlo, agreeing within four standard errors.
///
/// This is the only test that catches the multiplicity bias of PLAN section
/// 7.2, because comparing one Monte Carlo run to another compares two runs
/// that are wrong in the same way.
#[test]
fn test_sampling_agrees_with_enumeration() {
    let cases: [(&[&str], &str); 4] = [
        (&["AhAd", "KsKc"], "2c 7d 9h"),
        (&["AhKh", "QsJs"], "Th 9h 2c"),
        (&["AhAd", "KsKc", "7h8h"], "2c 7d 9s"),
        // A wildcard hand, where the sampler's general path is exercised.
        (&["A Kh", "QsJs"], "Th 9c 2c"),
    ];

    for (hands, board) in cases {
        let truth = exact(hands, board, "").equities();
        let guess = sampled(hands, board, "", 200_000, 4242).equities();

        for (seat, (exact_seat, sampled_seat)) in truth.iter().zip(&guess).enumerate() {
            let slack = 4.0 * sampled_seat.std_error + 1e-9;
            assert!(
                (exact_seat.equity - sampled_seat.equity).abs() <= slack,
                "{} on {}: seat {} enumerates to {:.4}% but samples to {:.4}% \
                 (four standard errors is {:.4}%)",
                hands.join(" vs "),
                board,
                seat,
                exact_seat.percent(),
                sampled_seat.percent(),
                slack * 100.0
            );
        }
    }
}

/// Shares are shares: whatever else is true, they divide one pot.
#[test]
fn test_equities_sum_to_one() {
    for result in [
        exact(&["AhAd", "KsKc"], "2c 7d 9h", ""),
        exact(&["AhAd", "KsKc", "7h8h"], "2c 7d 9s", ""),
        sampled(&["AKs", "22", "JhTh"], "", "", 50_000, 1),
    ] {
        let total: f64 = result.equities().iter().map(|player| player.equity).sum();
        assert!(
            (total - 1.0).abs() < 1e-9,
            "equities summed to {:.12}",
            total
        );
    }
}

/// Seating order carries no meaning, so reversing it reverses the answer.
#[test]
fn test_permuting_players_permutes_equities() {
    let forward = exact(&["AhAd", "KsKc", "7h8h"], "2c 7d 9s", "").equities();
    let backward = exact(&["7h8h", "KsKc", "AhAd"], "2c 7d 9s", "").equities();

    for (seat, player) in forward.iter().enumerate() {
        let mirrored = backward[forward.len() - 1 - seat];
        assert!(
            (player.equity - mirrored.equity).abs() < 1e-12,
            "seat {} moved from {:.6}% to {:.6}% by reordering",
            seat,
            player.percent(),
            mirrored.percent()
        );
    }
}

/// Two hands that differ only by a swap of suits must split the pot exactly.
#[test]
fn test_a_hand_against_its_own_mirror_is_even() {
    // Suited AK against suited AK: exchanging hearts for spades maps one to
    // the other, so neither can hold an edge.
    let equities = exact(&["AhKh", "AsKs"], "", "").equities();
    assert!(
        (equities[0].equity - 0.5).abs() < 1e-12,
        "expected exactly half, got {:.10}%",
        equities[0].percent()
    );
}

/// A wildcard narrowed to a single card is that card.
///
/// This tests the wildcard path against the concrete path directly: with
/// three aces dead, `A Kh` can only be `Ah Kh`, and must give the same answer.
#[test]
fn test_a_wildcard_narrowed_to_one_card_equals_naming_it() {
    let dead = "Ad Ac As";
    let wildcard = exact(&["A Kh", "QsJs"], "Th 9c 2c", dead).equities();
    let named = exact(&["Ah Kh", "QsJs"], "Th 9c 2c", dead).equities();

    for (seat, (loose, exactly)) in wildcard.iter().zip(&named).enumerate() {
        assert!(
            (loose.equity - exactly.equity).abs() < 1e-12,
            "seat {}: the wildcard gave {:.10}% and the named card {:.10}%",
            seat,
            loose.percent(),
            exactly.percent()
        );
    }
}

/// A dead card the hands cannot use moves the answer only slightly, and never
/// changes who is ahead.
#[test]
fn test_an_irrelevant_dead_card_barely_moves_anything() {
    let without = exact(&["AhAd", "KsKc"], "2c 7d 9h", "").equities();
    let with = exact(&["AhAd", "KsKc"], "2c 7d 9h", "3s").equities();
    assert!(
        (without[0].equity - with[0].equity).abs() < 0.01,
        "a dead three moved the answer from {:.4}% to {:.4}%",
        without[0].percent(),
        with[0].percent()
    );
}

/// Sums, not averages, so chunks merge by addition and the caller decides
/// when to stop.
#[test]
fn test_chunks_merge_by_addition() {
    let request = EquityRequest::from_text(Holdem, &["AhAd", "KsKc"], "2c 7d", "").unwrap();

    let mut merged = ChunkResult::empty(2);
    for seed in 0..4u64 {
        merged.merge(&run_chunk(&request, 25_000, seed).unwrap());
    }
    assert_eq!(merged.samples, 100_000);

    // Merging four chunks must land where one long run does, up to the
    // difference between two independent samples.
    let single = run_chunk(&request, 100_000, 0).unwrap().equities();
    let together = merged.equities();
    let slack = 4.0 * (single[0].std_error + together[0].std_error);
    assert!(
        (single[0].equity - together[0].equity).abs() <= slack,
        "merged to {:.4}%, one run gave {:.4}%",
        together[0].percent(),
        single[0].percent()
    );
}

/// The error bar shrinks with the root of the sample count, and vanishes when
/// the answer is enumerated.
#[test]
fn test_the_error_bar_behaves() {
    let request = EquityRequest::from_text(Holdem, &["AhAd", "KsKc"], "", "").unwrap();
    let few = run_chunk(&request, 10_000, 5).unwrap().equities()[0].std_error;
    let many = run_chunk(&request, 160_000, 5).unwrap().equities()[0].std_error;

    // Sixteen times the samples is four times the precision.
    let ratio = few / many;
    assert!(
        (3.0..5.0).contains(&ratio),
        "error shrank by {:.2}x over sixteen times the samples, expected about 4x",
        ratio
    );

    assert_eq!(
        exact(&["AhAd", "KsKc"], "2c 7d 9h", "").equities()[0].std_error,
        0.0,
        "reporting a confidence interval on a deterministic answer is a bug"
    );
}

/// A request no deal satisfies is refused when it is built, not spun on.
#[test]
fn test_impossible_requests_are_refused() {
    // Both seats named the same card.
    let clash = EquityRequest::from_text(Holdem, &["AhKh", "AhQs"], "", "");
    assert!(
        matches!(clash, Err(PokerError::Equity(EquityError::Infeasible(_)))),
        "one ace of hearts cannot sit in two hands"
    );

    // The board wants a card that is dead.
    let dead_board = EquityRequest::from_text(Holdem, &["AhKh", "QsJs"], "2c 7d 9h", "2c");
    assert!(matches!(
        dead_board,
        Err(PokerError::Equity(EquityError::Infeasible(_)))
    ));

    // Five hearts wanted from a deck with only four left.
    let too_few = EquityRequest::from_text(
        Holdem,
        &["h h", "h h"],
        "h",
        "2h 3h 4h 5h 6h 7h 8h 9h Th",
    );
    assert!(matches!(
        too_few,
        Err(PokerError::Equity(EquityError::Infeasible(_)))
    ));
}

/// A hand field that does not match the game is caught before sampling.
#[test]
fn test_slot_counts_are_checked() {
    assert!(EquityRequest::from_text(Holdem, &["AhKhQh", "QsJs"], "", "").is_err());
    assert!(EquityRequest::from_text(Holdem, &["AhKh"], "", "").is_err());
}

/// A spot too wide to walk says so rather than trying.
#[test]
fn test_enumeration_declines_when_the_space_is_too_large() {
    // Two entirely unspecified hands preflop is far past the limit.
    let request = EquityRequest::from_text(Holdem, &["**", "**"], "", "").unwrap();
    assert!(
        run_exact(&request).unwrap().is_none(),
        "the caller should be told to sample instead"
    );
}

/// A hand shorter than the game deals means the rest are still to come: a
/// five-card draw hand stands pat, a three-card one draws two.
#[test]
fn test_a_short_draw_hand_draws_the_difference() -> Result<(), PokerError> {
    use crate::variants::DeuceSeven;

    // Both stand pat, so there is nothing left to deal and one deal settles
    // it. 7-5-4-3-2 is the best hand in the game.
    let pat = EquityRequest::from_text(
        DeuceSeven,
        &["7h5c4d3s2h", "8h6c5d3h2c"],
        "",
        "",
    )?;
    let settled = run_exact(&pat)?.expect("two pat hands need no deal");
    assert_eq!(settled.samples, 1, "nothing is drawn");
    assert_eq!(
        settled.equities()[0].percent(),
        100.0,
        "seven-five is the nuts"
    );

    // Hero keeps four and draws one; villain stands pat with a nine low.
    let drawing = EquityRequest::from_text(DeuceSeven, &["7h5c4d3s", "9h8c6d5h2c"], "", "")?;
    let result = run_exact(&drawing)?.expect("one card to come is a small space");
    let equities = result.equities();
    assert!(
        (equities[0].equity + equities[1].equity - 1.0).abs() < 1e-9,
        "equities divide one pot"
    );
    assert!(
        equities[0].percent() > 0.0 && equities[0].percent() < 100.0,
        "a one-card draw is neither dead nor certain, got {:.2}%",
        equities[0].percent()
    );

    Ok(())
}

/// The two APIs describe the draw differently and must agree.
///
/// The old one takes a whole hand plus the cards to throw; the new one takes
/// what is kept, with the thrown cards named as dead. Both leave the same
/// deck to draw from, so the answers have to match.
#[test]
fn test_the_two_ways_of_writing_a_draw_agree() -> Result<(), PokerError> {
    use crate::cards::Card;
    use crate::hand::Hand;
    use crate::odds::EquityCalculator;
    use crate::variants::DeuceSeven;

    // Hero throws the king; villain throws the king.
    let mut old = EquityCalculator::new(DeuceSeven, 200_000);
    old.add_draw_player(
        "Hero".to_string(),
        Hand::from_str(DeuceSeven, "Th 8c Kd 4s 2h")?,
        Some(Card::from_str("Kd")?),
    )?;
    old.add_draw_player(
        "Villain".to_string(),
        Hand::from_str(DeuceSeven, "9d 7h Ks 4h 2d")?,
        Some(Card::from_str("Ks")?),
    )?;
    let old_result = old.calculate(drop)?;

    // The same spot: what each keeps, with the discards dead.
    let new = EquityRequest::from_text(
        DeuceSeven,
        &["Th8c4s2h", "9d7h4h2d"],
        "",
        "Kd Ks",
    )?;
    let new_result = run_exact(&new)?.expect("one card each is a small space");
    let equities = new_result.equities();

    assert!(
        (old_result["Hero"] - equities[0].percent()).abs() < 0.5,
        "the old API gave Hero {:.3}% and the new one {:.3}%",
        old_result["Hero"],
        equities[0].percent()
    );

    Ok(())
}

/// Stud works the same way: three cards known, four still to come.
#[test]
fn test_a_short_stud_hand_is_dealt_out() -> Result<(), PokerError> {
    use crate::variants::SevenCardStud;

    let short = EquityRequest::from_text(SevenCardStud, &["AhKhQh", "2c3d4s"], "", "")?;
    let spelled_out = EquityRequest::from_text(
        SevenCardStud,
        &["Ah Kh Qh * * * *", "2c 3d 4s * * * *"],
        "",
        "",
    )?;

    let short = run_chunk(&short, 60_000, 21)?.equities();
    let spelled = run_chunk(&spelled_out, 60_000, 21)?.equities();
    assert!(
        (short[0].equity - spelled[0].equity).abs() < 1e-12,
        "a short field and one written out with wildcards are the same request"
    );

    Ok(())
}

/// A community game deals every hole card at once, so a short field there is
/// a miscount rather than a hand in progress.
#[test]
fn test_community_games_still_want_every_hole_card() {
    assert!(
        EquityRequest::from_text(Holdem, &["Ah", "QsJs"], "", "").is_err(),
        "one hole card is a typo in hold'em, not a hand still being dealt"
    );
    assert!(
        EquityRequest::from_text(Holdem, &["A *", "QsJs"], "", "").is_ok(),
        "an unknown hole card is written as a wildcard"
    );
}
