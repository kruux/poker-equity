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

/// Text is turned into masks once, when the request is built, and the answer
/// is the same either way in.
///
/// The sampling loop never sees a string: `from_text` parses into the same
/// `HandSpec` masks `from_masks` takes, so the two are the same request and
/// must deal the same cards from the same seed.
#[test]
fn test_text_and_masks_are_the_same_request() -> Result<(), PokerError> {
    use crate::cards::{Card, CardSet};
    use crate::notation::HandSpec;

    let card = |text: &str| CardSet::from_cards(&Card::from_str(text).unwrap());

    let by_text = EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], "2c 7d 9h", "3s")?;
    let by_masks = EquityRequest::from_masks(
        Holdem,
        &[
            HandSpec::from_slots(&[card("Ah"), card("Kh")]),
            HandSpec::from_slots(&[card("Qs"), card("Qd")]),
        ],
        &[card("2c"), card("7d"), card("9h")],
        card("3s"),
    )?;

    // Same seed, same deals, so any difference at all is a difference in the
    // request rather than in the sampling.
    let from_text = run_chunk(&by_text, 50_000, 99)?;
    let from_masks = run_chunk(&by_masks, 50_000, 99)?;
    assert_eq!(
        from_text.share_sum, from_masks.share_sum,
        "the two ways of asking gave different answers"
    );

    // And a wildcard survives the round trip: "A" is the four aces.
    let spec = crate::notation::parse_hand("A Kh", 2)?;
    assert_eq!(spec.alternatives[0][0], CardSet::of_rank(crate::cards::Rank::Ace));
    assert_eq!(spec.alternatives[0][0].len(), 4);

    Ok(())
}

/// No card can be dealt twice, wherever the two claims on it come from.
///
/// Two mechanisms share the job, and they catch different things. A field
/// that names a card twice is caught while it is read, so the error can point
/// at the second naming. A card claimed by two *different* fields is invisible
/// to the reader and is caught by the matching that settles the whole request
/// at once -- which a count could not do, since there are fifty-two cards and
/// only nine slots wanting them.
#[test]
fn test_a_card_cannot_be_in_two_places() {
    // Caught while reading, because both claims are in one field.
    for (hands, board, why) in [
        (vec!["AhAh", "QsJs"], "", "a hand naming the same card twice"),
        (vec!["AhKh", "QsJs"], "2c 2c 3d", "a board repeating a card"),
    ] {
        let refused = EquityRequest::from_text(Holdem, &hands, board, "");
        assert!(
            matches!(refused, Err(PokerError::Equity(EquityError::Notation(_)))),
            "the reader should have caught {}",
            why
        );
    }

    // Caught by the matching, because the two claims are in different fields
    // and nothing reading one of them can see the other.
    for (hands, board, dead, why) in [
        (vec!["AhKh", "AhQs"], "", "", "two seats holding the ace of hearts"),
        (vec!["AhKh", "QsJs"], "Ah 2c 3d", "", "a seat and the board sharing a card"),
        (vec!["AhKh", "QsJs"], "", "Ah", "a seat holding a card that is dead"),
        (vec!["AhKh", "QsJs"], "2c 3d 4h", "2c", "the board holding a dead card"),
    ] {
        let refused = EquityRequest::from_text(Holdem, &hands, board, dead);
        assert!(
            matches!(refused, Err(PokerError::Equity(EquityError::Infeasible(_)))),
            "the matching should have caught {}",
            why
        );
    }

    // The same request without a clash is fine.
    assert!(EquityRequest::from_text(Holdem, &["AhKh", "QsJs"], "2c 3d 4h", "5c").is_ok());
}

/// Wildcards that overlap are still a valid request, and the constraint is
/// enforced on the deal rather than on each card in isolation.
#[test]
fn test_overlapping_wildcards_deal_consistently() -> Result<(), PokerError> {
    use crate::cards::{Rank, Suit};

    // "A c" is any ace and any club. The ace of clubs satisfies either slot
    // but cannot satisfy both, so every deal has to hold two distinct cards
    // that between them cover an ace and a club.
    let request = EquityRequest::from_text(Holdem, &["A c", "QsJs"], "", "")?;
    let result = run_chunk(&request, 20_000, 5)?;
    assert_eq!(result.samples, 20_000, "the request is satisfiable");

    // Narrowing a wildcard until one card is left makes it that card.
    let narrowed = run_exact(&EquityRequest::from_text(
        Holdem,
        &["A Kh", "QsJs"],
        "Th 9c 2c",
        "Ad Ac As",
    )?)?
    .expect("small enough to enumerate");
    let named = run_exact(&EquityRequest::from_text(
        Holdem,
        &["Ah Kh", "QsJs"],
        "Th 9c 2c",
        "Ad Ac As",
    )?)?
    .expect("small enough to enumerate");
    assert_eq!(narrowed.share_sum, named.share_sum);

    // A wildcard with nothing left to admit is refused rather than spun on.
    let nothing_left = EquityRequest::from_text(
        Holdem,
        &["A Kh", "QsJs"],
        "",
        "Ah Ad Ac As",
    );
    assert!(
        matches!(nothing_left, Err(PokerError::Equity(EquityError::Infeasible(_)))),
        "no ace is left to fill the slot"
    );

    let _ = (Rank::Ace, Suit::Club);
    Ok(())
}

/// Two seats asking for the same thing must split the pot exactly, and the
/// order they are dealt in must not show.
///
/// Seats are dealt one after another, each drawing from its own list of valid
/// holdings and rejecting anything an earlier seat already took, with the
/// whole deal retried when that happens. That is what keeps it uniform over
/// complete deals rather than favouring whoever was dealt first -- and this
/// is the test that would notice if it stopped being true.
#[test]
fn test_seats_asking_for_the_same_thing_split_evenly() -> Result<(), PokerError> {
    for spec in ["A *", "A c", "* *"] {
        let request = EquityRequest::from_text(Holdem, &[spec, spec], "", "")?;

        // Averaged over seeds, because one run of a quarter-million deals
        // wanders a standard error or two on its own.
        let mut lean = 0.0;
        let seeds = 4;
        for seed in 0..seeds {
            let equities = run_chunk(&request, 250_000, seed * 7919)?.equities();
            lean += (equities[0].equity - 0.5) / equities[0].std_error;
        }
        lean /= seeds as f64;

        assert!(
            lean.abs() < 1.5,
            "{:?} against itself leans {:+.2} standard errors toward the seat \
             dealt first, which symmetry forbids",
            spec,
            lean
        );
    }
    Ok(())
}

/// A wildcard needs a card left to satisfy it, and that is settled before
/// sampling starts rather than discovered by spinning.
#[test]
fn test_a_wildcard_needs_something_left_to_fill_it() {
    // "2 *" wants a deuce and any second card.
    assert!(
        EquityRequest::from_text(Holdem, &["2 *", "QsJs"], "", "2c 2d 2h").is_ok(),
        "one deuce is still enough"
    );
    assert!(
        matches!(
            EquityRequest::from_text(Holdem, &["2 *", "QsJs"], "", "2c 2d 2h 2s"),
            Err(PokerError::Equity(EquityError::Infeasible(_)))
        ),
        "with every deuce dead there is nothing to fill the slot"
    );

    // Two seats both wanting a deuce need two deuces between them.
    assert!(EquityRequest::from_text(Holdem, &["2 *", "2 *"], "", "2c 2d").is_ok());
    assert!(
        matches!(
            EquityRequest::from_text(Holdem, &["2 *", "2 *"], "", "2c 2d 2h"),
            Err(PokerError::Equity(EquityError::Infeasible(_)))
        ),
        "one deuce cannot fill two seats"
    );

    // And the board counts as a claim like any other.
    assert!(
        matches!(
            EquityRequest::from_text(Holdem, &["2 *", "QsJs"], "2c 2d 2h", "2s"),
            Err(PokerError::Equity(EquityError::Infeasible(_)))
        ),
        "the board and the dead cards together take every deuce"
    );
}

/// A deal that cannot fill every slot is thrown away entire and drawn again.
///
/// That happens when hands compete for the same cards -- a seat that will
/// take anything taking the last five another seat named. Dealing the
/// fussiest hands first, the board among them, keeps it rare, and rejecting
/// the *whole* deal rather than redrawing one seat is what keeps the result
/// uniform.
#[test]
fn test_a_deal_that_cannot_be_filled_is_drawn_again() -> Result<(), PokerError> {
    // Nothing competes, so nothing is thrown away.
    let plain = EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], "", "")?;
    assert_eq!(run_chunk(&plain, 20_000, 1)?.acceptance(), 1.0);

    // A seat taking any card alongside one that wants a five: dealing the
    // fussier seat first means the wildcard cannot take its card first.
    for hands in [["* *", "5 *"], ["5 *", "* *"]] {
        let request = EquityRequest::from_text(Holdem, &hands, "", "")?;
        let kept = run_chunk(&request, 20_000, 2)?.acceptance();
        assert!(
            kept > 0.99,
            "{:?} kept only {:.1}% of deals; the fussier seat should be dealt first",
            hands,
            kept * 100.0
        );
    }

    // Where every seat is equally fussy there is nothing to reorder, so
    // deals really are thrown away -- but the answer still arrives.
    let contended = EquityRequest::from_text(Holdem, &["5 *", "5 *", "5 *", "5 *"], "", "")?;
    let result = run_chunk(&contended, 20_000, 3)?;
    assert_eq!(result.samples, 20_000, "the answer still arrives");
    assert!(result.attempts > result.samples, "and deals were thrown away");

    Ok(())
}

/// Dealing order changes how often a deal is thrown away, and nothing else.
///
/// Checked against enumeration, which builds holdings directly and never
/// touches the dealing path at all.
#[test]
fn test_dealing_order_does_not_move_the_answer() -> Result<(), PokerError> {
    let board = "Kh Qd 9c 3s 2h";
    for hands in [
        vec!["* *", "5 *"],
        vec!["5 *", "* *"],
        vec!["A c", "* *"],
    ] {
        let request = EquityRequest::from_text(Holdem, &hands, board, "")?;
        let exact = run_exact(&request)?.expect("a full board leaves little to walk");
        let sampled = run_chunk(&request, 300_000, 17)?;

        for (seat, (walked, drawn)) in exact.equities().iter().zip(sampled.equities()).enumerate() {
            assert!(
                (walked.equity - drawn.equity).abs() <= 4.0 * drawn.std_error,
                "{:?} seat {}: enumerating gives {:.4}% and dealing gives {:.4}%",
                hands,
                seat,
                walked.percent(),
                drawn.percent()
            );
        }
    }
    Ok(())
}
