//! What a request works out before any card is dealt.

use crate::{
    cards::Card,
    error::PokerError,
    odds::{run_chunk, run_exact, EquityRequest},
};


/// Eight-handed stud runs the deck out -- eight sevens is fifty-six against
/// fifty-two cards -- so the last card is not dealt to each player. One goes
/// face up in the middle and everyone counts it as their seventh.
///
/// Rare enough that most players never see it happen, and a rule all the
/// same. Seven-handed is forty-nine cards and needs none of this.
#[test]
fn test_eight_handed_stud_shares_the_last_card() -> Result<(), PokerError> {
    use crate::variants::{Razz, Stud, StudHiLo};

    // Seven seats fit, so nothing is shared and a board is still an error.
    let seven = ["Ah2c3d", "4s5h6c", "7d8s9h", "TcJdQs", "Kh2d3c", "4h5c6d", "7s8h9c"];
    let request = EquityRequest::from_text(Stud, &seven, "", "")?;
    assert_eq!(request.hole_cards(), 7, "seven seats, seven cards each");
    assert_eq!(request.board_cards(), 0, "nothing in the middle");
    assert!(
        EquityRequest::from_text(Stud, &seven, "Ks", "").is_err(),
        "a seven-handed stud game has no board to put a card on"
    );

    // The eighth seat is what tips it: six each and one shared.
    let eight = [
        "Ah2c3d", "4s5h6c", "7d8s9h", "TcJdQs", "Kh2d3c", "4h5c6d", "7s8h9c", "TdJsQh",
    ];
    let request = EquityRequest::from_text(Stud, &eight, "", "")?;
    assert_eq!(request.hole_cards(), 6, "six private cards a seat");
    assert_eq!(request.board_cards(), 1, "and one in the middle");

    // The shared card may be named, like any other board card.
    let known = EquityRequest::from_text(Stud, &eight, "Ks", "")?;
    assert_eq!(known.board_cards(), 1);

    // Razz and the split-pot game deal the same way, so they share the rule.
    assert_eq!(EquityRequest::from_text(Razz, &eight, "", "")?.board_cards(), 1);
    assert_eq!(EquityRequest::from_text(StudHiLo, &eight, "", "")?.board_cards(), 1);

    // And it deals: every seat ends up with seven cards to be scored from,
    // six of its own and the one in the middle.
    let result = run_chunk(&request, 2_000, 5)?;
    assert_eq!(result.samples, 2_000, "eight-handed stud deals without running short");
    let total: f64 = result.equities().iter().map(|player| player.equity).sum();
    assert!(
        (total - 1.0).abs() < 1e-9,
        "the pot is shared out exactly once, got {}",
        total
    );

    Ok(())
}


/// Keeping one seat's named cards out of another seat's pool must not change
/// an answer. It throws away only deals that were going to be rejected
/// anyway -- a seat cannot hold a card another seat has already named -- so
/// the spots that count are all still reachable and still equally likely.
///
/// Three stud seats holding six cards each, one still to come: small enough
/// to walk outright at 34 x 33 x 32 deals, so the sampled answer has an exact
/// one to be checked against.
#[test]
fn test_reserving_named_cards_does_not_bias_the_deal() -> Result<(), PokerError> {
    use crate::variants::Stud;

    let hands = ["Ah2c3d4s5h6c", "7d8s9hTcJdQs", "Kh2d3c4h5c6d"];
    let request = EquityRequest::from_text(Stud, &hands, "", "")?;

    let exact = run_exact(&request)?.expect("thirty-six thousand deals is walkable");
    let sampled = run_chunk(&request, 300_000, 11)?;

    assert!(
        sampled.acceptance() > 0.999,
        "every deal should be usable, kept {:.4}",
        sampled.acceptance()
    );

    for seat in 0..hands.len() {
        let gap = (exact.equities()[seat].equity - sampled.equities()[seat].equity).abs();
        let slack = 5.0 * sampled.equities()[seat].std_error;
        assert!(
            gap <= slack,
            "seat {} walked to {:.5} and sampled to {:.5}, {:.5} apart against {:.5} of slack",
            seat,
            exact.equities()[seat].equity,
            sampled.equities()[seat].equity,
            gap,
            slack
        );
    }

    Ok(())
}


/// A full ring of stud, played out rather than merely constructed.
///
/// Eight seats is the case the deck cannot cover, so this is where the shared
/// card has to actually work: every seat must be scored from seven cards, the
/// shared one must be the same card for all of them, no card may appear
/// twice, and the pot must come out whole.
#[test]
fn test_a_full_ring_of_stud_plays_out() -> Result<(), PokerError> {
    use crate::variants::{PokerVariant, Razz, Stud, StudHiLo};

    let eight = [
        "Ah2c3d", "4s5h6c", "7d8s9h", "TcJdQs", "Kh2d3c", "4h5c6d", "7s8h9c", "TdJsQh",
    ];

    let request = EquityRequest::from_text(Stud, &eight, "", "")?;
    let result = run_chunk(&request, 20_000, 17)?;

    assert_eq!(result.samples, 20_000, "a full ring deals every time");
    assert!(
        result.acceptance() > 0.999,
        "no deal should be thrown away, kept {:.4}",
        result.acceptance()
    );

    // The pot is shared out exactly once, deal after deal.
    let total: f64 = result.equities().iter().map(|player| player.equity).sum();
    assert!((total - 1.0).abs() < 1e-9, "the pot summed to {}", total);

    // Nobody is frozen out and nobody wins everything: with three cards named
    // and four to come, every seat has some chance.
    for (seat, player) in result.equities().iter().enumerate() {
        assert!(
            player.equity > 0.01 && player.equity < 0.50,
            "seat {} has {:.4} of the pot, which is not a real share",
            seat,
            player.equity
        );
    }

    // Name every card and the deal is settled: eight seats of six, plus the
    // one in the middle, is forty-nine of the fifty-two. So there is exactly
    // one deal to walk, and the answer can be checked against scoring each
    // seat's seven cards directly -- which is the real question, since it is
    // only right if the shared card reached all eight of them.
    let deck: Vec<Card> = (0..52).map(|i| Card::from_index(i).expect("a card")).collect();
    let fields: Vec<String> = (0..8)
        .map(|seat| {
            (0..6)
                .map(|card| deck[seat * 6 + card].to_string())
                .collect::<Vec<_>>()
                .join("")
        })
        .collect();
    let shared = deck[48];
    let named: Vec<&str> = fields.iter().map(|field| field.as_str()).collect();

    let settled = EquityRequest::from_text(Stud, &named, &shared.to_string(), "")?;
    let walked = run_exact(&settled)?.expect("one deal, since every card is named");
    assert_eq!(walked.samples, 1, "a fully named deal is one deal");

    // The same eight hands, scored without the engine's help.
    let scores: Vec<u32> = (0..8)
        .map(|seat| {
            let mut cards: Vec<Card> = (0..6).map(|card| deck[seat * 6 + card]).collect();
            cards.push(shared);
            Stud.score(&cards)
        })
        .collect();
    let best = *scores.iter().min().expect("eight seats");
    let winners: Vec<usize> = (0..8).filter(|&seat| scores[seat] == best).collect();
    let share = 1.0 / winners.len() as f64;

    for seat in 0..8 {
        let expected = if winners.contains(&seat) { share } else { 0.0 };
        assert!(
            (walked.equities()[seat].equity - expected).abs() < 1e-9,
            "seat {} took {:.4} of the pot, against {:.4} from scoring it by hand",
            seat,
            walked.equities()[seat].equity,
            expected
        );
    }

    // Razz and stud hi/lo deal identically, so they must survive a full ring
    // too -- and hi/lo has a low half to hand out on top.
    let razz = run_chunk(&EquityRequest::from_text(Razz, &eight, "", "")?, 20_000, 3)?;
    let total: f64 = razz.equities().iter().map(|player| player.equity).sum();
    assert!((total - 1.0).abs() < 1e-9, "razz pot summed to {}", total);

    let hi_lo = run_chunk(&EquityRequest::from_text(StudHiLo, &eight, "", "")?, 20_000, 3)?;
    let total: f64 = hi_lo.equities().iter().map(|player| player.equity).sum();
    assert!((total - 1.0).abs() < 1e-9, "stud hi/lo pot summed to {}", total);
    let low: f64 = hi_lo.equities().iter().map(|player| player.low_equity).sum();
    assert!(
        low > 0.0 && low <= 0.5 + 1e-9,
        "the low half is {} of the pot, which cannot be right",
        low
    );

    // A seat may still be short of cards: stud fields arrive street by
    // street, so a full ring on third street is three cards each.
    let third_street = ["Ah", "4s", "7d", "Tc", "Kh", "4h", "7s", "Td"];
    let early = EquityRequest::from_text(Stud, &third_street, "", "")?;
    assert_eq!(early.hole_cards(), 6);
    assert_eq!(early.board_cards(), 1);
    assert_eq!(run_chunk(&early, 5_000, 2)?.samples, 5_000);

    // Nine seats cannot be dealt at all, shared card or not: nine sixes and
    // one in the middle is fifty-five.
    let nine: Vec<&str> = eight.iter().copied().chain(["2s3h4c"]).collect();
    assert!(
        EquityRequest::from_text(Stud, &nine, "", "").is_err(),
        "nine seats is more stud than a deck holds however the last card is dealt"
    );

    Ok(())
}


/// How many cards a field may name, which is not one rule but three.
///
/// A community game deals every hole card at once, so a short field is a
/// miscount. Stud deals street by street but deals to everybody at the same
/// time, so fields may be short and must all be short by the same amount --
/// third street is three cards for the whole table. A draw game is the
/// opposite again: a short field is how a player says how many they are
/// drawing, so five against four is a pat hand against a one-card draw and
/// is the point of the notation.
///
/// The middle rule is the one worth a test. `A23` against `2345` looks
/// harmless and is not a poker situation at all, and without this it was
/// quietly answered rather than refused.
#[test]
fn test_a_field_may_only_be_short_where_the_game_allows_it() -> Result<(), PokerError> {
    use crate::error::EquityError;
    use crate::variants::{Badugi, DeuceSeven, Holdem, Omaha, Razz, Stud, StudHiLo};

    // Stud: short is fine, unevenly short is not.
    assert!(EquityRequest::from_text(Stud, &["AsKsQs", "AdKdQd"], "", "").is_ok());
    for (label, hands) in [
        ("stud hi/lo, three against four", &["A23", "2345"][..]),
        ("stud, three against two", &["AsKsQs", "AdKd"][..]),
        ("razz, three against four", &["Ah2h3h", "4d5d6d7d"][..]),
    ] {
        let error = EquityRequest::from_text(StudHiLo, hands, "", "").unwrap_err();
        assert!(
            matches!(error, PokerError::Equity(EquityError::UnequalHandSizes)),
            "{} gave {:?}",
            label,
            error
        );
    }
    assert!(matches!(
        EquityRequest::from_text(Razz, &["Ah2h3h", "4d5d6d7d"], "", ""),
        Err(PokerError::Equity(EquityError::UnequalHandSizes))
    ));

    // Draw: uneven is the whole point. Pat against one, against two.
    assert!(EquityRequest::from_text(DeuceSeven, &["7d5h4c3s2h", "8h6d4s3c"], "", "").is_ok());
    assert!(EquityRequest::from_text(DeuceSeven, &["7d5h4c3s2h", "8h6d"], "", "").is_ok());
    assert!(EquityRequest::from_text(Badugi, &["Ac2d3h4s", "5s6h"], "", "").is_ok());

    // Community: every hole card is dealt at once, so short is always wrong.
    assert!(EquityRequest::from_text(Holdem, &["AhKh", "Qs"], "", "").is_err());
    assert!(EquityRequest::from_text(Omaha, &["AhKh7c2d", "QsQdJs"], "", "").is_err());

    // A field longer than the game deals is wrong everywhere.
    assert!(EquityRequest::from_text(Stud, &["AsKsQsJsTs9s8s7s", "AdKdQdJdTd9d8d7d"], "", "")
        .is_err());
    assert!(EquityRequest::from_text(DeuceSeven, &["7d5h4c3s2h9c", "7c5s4h3d2c9d"], "", "")
        .is_err());

    Ok(())
}


/// A card this game's deck never held is a different mistake from a card
/// used twice, and says so.
///
/// Short deck is thirty-six cards; a deuce is not one of them. Calling that
/// a duplicate would send someone hunting for the second deuce.
#[test]
fn test_a_card_outside_the_deck_says_so() -> Result<(), PokerError> {
    use crate::error::GameError;
    use crate::variants::ShortDeck;

    for (label, hands, board) in [
        ("a deuce in hand", &["2h3h", "QsQd"][..], ""),
        ("a five in hand", &["5h6h", "QsQd"][..], ""),
        ("a deuce on the board", &["AhKh", "QsQd"][..], "2c7d9h"),
    ] {
        let error = EquityRequest::from_text(ShortDeck, hands, board, "").unwrap_err();
        assert!(
            matches!(error, PokerError::Game(GameError::NotInDeck(_))),
            "{} gave {:?}",
            label,
            error
        );
    }

    // Naming it dead is harmless: it was never in the deck to remove.
    assert!(EquityRequest::from_text(ShortDeck, &["AhKh", "QsQd"], "", "2c").is_ok());

    // And a real duplicate is still a duplicate.
    let error = EquityRequest::from_text(ShortDeck, &["AhKh", "AhQd"], "", "").unwrap_err();
    assert!(matches!(error, PokerError::Game(GameError::DuplicateCard(_))), "{:?}", error);

    Ok(())
}
