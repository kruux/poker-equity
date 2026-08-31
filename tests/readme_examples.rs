//! Runs the examples from `README.md`, so that they cannot quietly rot.
//!
//! Documentation that no longer compiles is worse than none: it costs a
//! reader their trust in the rest of it. These are the same snippets, with
//! the results printed so a change in the numbers shows up too.

use poker_equity::{cards::{Card, CardSet, Rank, Suit}, notation::HandSpec,
                       odds::{equity, equity_with_progress, run_chunk, ChunkResult,
                              EquityRequest, Target},
                       variants::{DeuceSeven, Holdem, Stud}};

#[test]
fn test_the_readme_still_works() {
    // opening example
    let request = EquityRequest::from_text(Holdem, &["AhAd", "KsKc"], "2c 7d 9h", "").unwrap();
    let result = equity(&request, Target::Exact).unwrap();
    println!("opening: {:.2}% / {:.2}%", result.equities()[0].percent(), result.equities()[1].percent());

    // the short version at the top of the README
    let wide = EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], "", "").unwrap();
    let short = equity(&wide, Target::Samples(500_000)).unwrap();
    let seats = short.equities();
    println!("short:   {:.2}% +/- {:.2}  /  {:.2}% +/- {:.2}",
        seats[0].percent(), seats[0].margin_percent(),
        seats[1].percent(), seats[1].margin_percent());
    assert!((seats[0].percent() - 46.20).abs() < 0.5, "the README quotes 46.20%");
    assert!((seats[1].percent() - 53.80).abs() < 0.5, "the README quotes 53.80%");

    // a plain sample count, and a precision target
    let counted = equity(&wide, Target::Samples(500_000)).unwrap();
    assert!(counted.samples >= 500_000);
    println!("500k:    {:.2}% +/- {:.2}",
        counted.equities()[0].percent(), counted.equities()[0].margin_percent());

    let precise = equity(&wide, Target::StandardError(0.001)).unwrap();
    assert!(precise.equities().iter().all(|p| p.std_error <= 0.001));
    println!("precise: {:.2}% after {} deals", precise.equities()[0].percent(), precise.samples);

    // watching it go
    let mut reports = 0;
    equity_with_progress(&wide, Target::Samples(400_000), |progress| {
        reports += 1;
        assert!(progress.acceptance > 0.0 && progress.acceptance <= 1.0);
    }).unwrap();
    println!("watched: {} progress reports", reports);

    // chunk loop
    let mut total = ChunkResult::empty(2);
    for seed in 0..8 { total.merge(&run_chunk(&request, 50_000, seed).unwrap()); }
    println!("merged:  {} deals", total.samples);

    // stud and draw
    EquityRequest::from_text(Stud, &["Ah2c3d", "QsQdJs"], "", "").unwrap();
    EquityRequest::from_text(DeuceSeven, &["7h5c4d3s", "9h8c6d5h2c"], "", "Kd").unwrap();

    // mask API
    let ace_of_hearts = Card::new(Suit::Heart, Rank::Ace);
    assert_eq!(ace_of_hearts.index(), 50);
    let queen_of_spades = CardSet::from_cards(&Card::parse_field("Qs").unwrap());
    let masks = EquityRequest::from_masks(
        Holdem,
        &[
            HandSpec::from_slots(&[CardSet::of_rank(Rank::Ace), CardSet::of_suit(Suit::Club)]),
            HandSpec::from_slots(&[queen_of_spades, CardSet::FULL_DECK]),
        ],
        &[],
        CardSet::EMPTY,
    ).unwrap();
    println!("masks:   {:.2}%", run_chunk(&masks, 20_000, 1).unwrap().equities()[0].percent());

    // the mask layout claims
    assert_eq!(CardSet::of_rank(Rank::Ace).bits(), 0b1111u64 << 48);
    assert_eq!(CardSet::of_suit(Suit::Club).bits(), 0x1_1111_1111_1111u64);
    assert_eq!(CardSet::FULL_DECK.bits(), (1u64 << 52) - 1);
    println!("layout:  all three constants check out");

    // The figure the README opens with.
    assert_eq!(
        format!("{:.2}%", result.equities()[0].percent()),
        "91.62%",
        "the README's opening example quotes this"
    );
}
