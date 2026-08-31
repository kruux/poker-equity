//! What a request works out before any card is dealt.

use crate::{
    cards::{Card, CardSet},
    variants::{EquityCalculation, PokerType, PokerVariant},
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
    use crate::error::EquityError;
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

    // A full ring is still short of cards on third street, and says so: six
    // apiece rather than seven, with the seventh in the middle.
    assert_eq!(request.hole_cards(), 6);
    assert_eq!(request.board_cards(), 1);

    // Third street is as early as a stud hand goes. Two cards is not an
    // earlier street, it is a hand that was never dealt.
    let two_each = ["Ah2c", "4s5h", "7d8s", "TcJd", "Kh2d", "4h5c", "7s8h", "TdJs"];
    assert!(matches!(
        EquityRequest::from_text(Stud, &two_each, "", ""),
        Err(PokerError::Equity(EquityError::NotEnoughCards(2)))
    ));

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
    use crate::variants::{Holdem, ShortDeck};

    for (label, hands, board, dead) in [
        ("a deuce in hand", &["2h3h", "QsQd"][..], "", ""),
        ("a five in hand", &["5h6h", "QsQd"][..], "", ""),
        ("a deuce on the board", &["AhKh", "QsQd"][..], "2c7d9h", ""),
        // Every field, including the one where it costs nothing to allow.
        // Taking a deuce out of a deck that never held one is a no-op, so
        // this is caught here or not at all -- and a caller who names one has
        // made the same mistake about the game either way.
        ("a deuce named dead", &["AhKh", "QsQd"][..], "", "2c"),
    ] {
        let error = EquityRequest::from_text(ShortDeck, hands, board, dead).unwrap_err();
        assert!(
            matches!(error, PokerError::Game(GameError::NotInDeck(_))),
            "{} gave {:?}",
            label,
            error
        );
    }

    // A card the short deck does hold is dead in the ordinary way, and the
    // full-deck games still take a deuce without complaint.
    assert!(EquityRequest::from_text(ShortDeck, &["AhKh", "QsQd"], "", "7c").is_ok());
    assert!(EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], "", "2c").is_ok());

    // And a real duplicate is still a duplicate.
    let error = EquityRequest::from_text(ShortDeck, &["AhKh", "AhQd"], "", "").unwrap_err();
    assert!(matches!(error, PokerError::Game(GameError::DuplicateCard(_))), "{:?}", error);

    Ok(())
}

/// The board is how much of it you know, not which street you are on.
///
/// A hold'em board may hold nought to five cards, and wildcards among them,
/// so "I know the flop and the river but not the turn" is a question that can
/// be asked. Restricting it to 0, 3 or 4 would model the streets and forbid
/// perfectly good questions -- including the simplest one, which is a
/// finished board and the question of who won.
///
/// Courchevel is the one game with a floor, because its first board card is
/// face up before the betting: a Courchevel hand showing nothing is not
/// Courchevel, it is five-card Omaha.
#[test]
fn test_how_much_of_the_board_may_be_known() -> Result<(), PokerError> {
    use crate::error::EquityError;
    use crate::variants::{Courchevel, CourchevelHiLo, Holdem, OmahaFive};

    for board in ["", "2c", "2c7d", "2c7d9h", "2c7d9hTs", "2c7d9hTs4c"] {
        assert!(
            EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], board, "").is_ok(),
            "a board of {:?} should be a fair question",
            board
        );
    }

    // Known flop and river, unknown turn.
    let gappy = EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], "2c7d9h*Ks", "")?;
    assert_eq!(gappy.board_cards(), 5);
    assert_eq!(run_chunk(&gappy, 1_000, 4)?.samples, 1_000);

    // Six is more board than there is.
    assert!(EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], "2c7d9hTs4c3d", "").is_err());

    // Courchevel needs its first card, and both split-pot and high forms
    // agree about that.
    assert!(matches!(
        EquityRequest::from_text(Courchevel, &["AhKh7c2d3c", "QsQdJsTd4h"], "", ""),
        Err(PokerError::Equity(EquityError::NotEnoughBoardCards { least: 1, found: 0 }))
    ));
    assert!(matches!(
        EquityRequest::from_text(CourchevelHiLo, &["Ah2c3d4s5c", "QsQdJsTd9h"], "", ""),
        Err(PokerError::Equity(EquityError::NotEnoughBoardCards { least: 1, found: 0 }))
    ));
    assert!(EquityRequest::from_text(Courchevel, &["AhKh7c2d3c", "QsQdJsTd4h"], "8s", "").is_ok());

    // A wildcard satisfies the rule, and is meant to: the card has been
    // dealt, it is simply not yet known. That is how "what was the turned
    // card worth?" is asked -- the same spot with `*` in its place.
    assert!(EquityRequest::from_text(Courchevel, &["AhKh7c2d3c", "QsQdJsTd4h"], "*", "").is_ok());

    // Five-card Omaha is the same game without that rule, so it may show
    // nothing at all.
    assert!(EquityRequest::from_text(OmahaFive, &["AhKh7c2d3c", "QsQdJsTd4h"], "", "").is_ok());

    Ok(())
}

/// Exact mode declines a spot it cannot walk, rather than trying.
///
/// The walk chooses a holding for each seat in turn and skips any that clashes
/// with one already chosen. A skip is work done with no deal to show for it,
/// so counting deals is no limit at all on a table whose seats want the same
/// cards: six stud seats with four cards to come each is 211,876 holdings
/// apiece, and sifting those combinations for the few that do not clash ran
/// for as long as it was left to.
///
/// Counting the combinations first is what makes the answer immediate. It is
/// an overcount, since it ignores the clashes, but it errs towards declining
/// -- and declining means sampling, which answers the question anyway.
#[test]
fn test_a_spot_too_large_to_walk_is_declined_rather_than_attempted() -> Result<(), PokerError> {
    use crate::variants::{Holdem, Stud, StudHiLo};

    let crowded = EquityRequest::from_text(
        StudHiLo,
        &["KhKc9d", "QhQc8d", "Ah3c7d", "2h4c6d", "3h5c4d", "4h6c8h"],
        "",
        "",
    )?;
    assert!(
        run_exact(&crowded)?.is_none(),
        "six seats with four cards to come each cannot be walked"
    );
    // And it is still a question, just a sampled one.
    assert_eq!(run_chunk(&crowded, 5_000, 5)?.samples, 5_000);

    // Two stud seats two cards from home is 741,321 combinations at the
    // outside, so that one is walked.
    let settled = EquityRequest::from_text(Stud, &["AhKhQhJhTh", "AsKsQsJsTs"], "", "")?;
    assert_eq!(
        run_exact(&settled)?.expect("small enough to walk").samples,
        671_580
    );

    // The board is counted separately from the seats, so a preflop hold'em
    // spot with both hands named is walked however many boards there are.
    let preflop = EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], "", "")?;
    assert_eq!(
        run_exact(&preflop)?.expect("one board at a time").samples,
        1_712_304
    );

    Ok(())
}

/// How many seats each game can actually deal, and what it says past that.
///
/// The deck is the limit and it is not the same limit twice: hold'em seats
/// twenty-three because two cards and a five-card board leave room for
/// twenty-three, six-card Omaha seats seven, and stud seats eight only
/// because the eighth seat is what triggers the shared last card.
///
/// A caller asking for more than that has almost always looped one time too
/// many, so the refusal names the count rather than the cards. It used to
/// come back as "no deal satisfies this request", which sends the reader to
/// look at their hands when the mistake is in their seat count.
#[test]
fn test_each_game_seats_what_its_deck_allows() -> Result<(), PokerError> {
    use crate::error::EquityError;
    use crate::variants::{
        Badugi, Courchevel, DeuceSeven, Holdem, Omaha, OmahaFive, OmahaHiLo, OmahaSix, Razz,
        ShortDeck, Stud, StudHiLo,
    };

    /// Deals `seats` distinct fields from this game's own deck, plus whatever
    /// board it insists on, so the only thing under test is the seat count.
    fn table<V: PokerVariant + EquityCalculation>(
        variant: V,
        seats: usize,
    ) -> (Vec<String>, String) {
        let mut deck = variant.deck().iter();
        let board: String = (0..variant.least_board_cards())
            .filter_map(|_| deck.next())
            .map(|card| card.to_string())
            .collect();
        // A field is however many cards the game deals, except where the deck
        // runs out and the last card is shared instead.
        let hole = variant.hole_cards()
            - usize::from(
                matches!(variant.poker_type(), PokerType::Stud)
                    && seats * variant.hole_cards() > variant.deck().len() as usize,
            );
        let hands = (0..seats)
            .map(|_| (0..hole).filter_map(|_| deck.next()).map(|c| c.to_string()).collect())
            .collect();
        (hands, board)
    }

    macro_rules! seats {
        ($variant:expr, $room:expr) => {{
            let (hands, board) = table($variant, $room);
            let refs: Vec<&str> = hands.iter().map(|h| h.as_str()).collect();
            let full = EquityRequest::from_text($variant, &refs, &board, "")?;
            assert_eq!(full.players(), $room, "{} seats {}", $variant.key(), $room);
            // A full table still deals, rather than merely being accepted.
            assert_eq!(run_chunk(&full, 200, 5)?.samples, 200);

            // One seat more than there is room for cannot be written out in
            // text at all -- there are not enough distinct cards to name, so
            // the notation refuses the empty field first. The masks are where
            // a caller can actually express it, by handing over the same
            // holding more times than the deck can seat.
            let one_more = vec![
                crate::notation::parse_hand(refs[0], full.hole_cards())?;
                $room + 1
            ];
            let board_masks = crate::notation::parse_board(&board, $variant.board_cards())?;
            let over = EquityRequest::from_masks($variant, &one_more, &board_masks, CardSet::EMPTY);
            assert!(
                matches!(
                    over,
                    Err(PokerError::Equity(EquityError::TooManyPlayers { asked, room }))
                        if asked == $room + 1 && room == $room
                ),
                "{} should refuse {} seats by name, got {:?}",
                $variant.key(),
                $room + 1,
                over.map(|_| "accepted")
            );
        }};
    }

    // Two hole cards and a five-card board: (52 - 5) / 2.
    seats!(Holdem, 23);
    // The same game on thirty-six cards.
    seats!(ShortDeck, 15);
    // Omaha, by how many cards a seat holds.
    seats!(Omaha, 11);
    seats!(OmahaFive, 9);
    seats!(OmahaSix, 7);
    seats!(OmahaHiLo, 11);
    // Courchevel holds five like five-card Omaha; its face-up card is part of
    // the same five-card board.
    seats!(Courchevel, 9);
    // Stud would seat seven at seven cards each. The eighth seat is exactly
    // what makes the last card shared, and six each plus one in the middle is
    // forty-nine.
    seats!(Stud, 8);
    seats!(StudHiLo, 8);
    seats!(Razz, 8);
    // Draw games have no board at all.
    seats!(DeuceSeven, 10);
    seats!(Badugi, 13);

    Ok(())
}

/// A hand written without suits must sample as well as one written with them,
/// and must give the same answer.
///
/// Razz ranks on ranks alone, and `A23` against `A24` leaves the deck holding
/// exactly the rank multiset that `Ah2c3d` against `As2d4c` leaves. So the
/// two questions have one answer, and the named form is an independent check
/// on the unsuited one rather than merely a second opinion.
///
/// This is the regression for a sampler that drew seven cards blind and kept
/// the 0.4% that happened to hold an ace, a deuce and a three -- correct, and
/// three hundred times slower than naming the suits.
#[test]
fn test_unsuited_fields_sample_as_well_as_named_ones() -> Result<(), PokerError> {
    use crate::odds::run_batch;
    use crate::variants::Razz;

    let unsuited = EquityRequest::from_text(Razz, &["A23", "A24"], "", "")?;
    let named = EquityRequest::from_text(Razz, &["Ah2c3d", "As2d4c"], "", "")?;

    let loose = run_batch(&unsuited, 200_000, 9, 2)?;
    let tight = run_batch(&named, 200_000, 9, 2)?;

    assert!(
        loose.acceptance() > 0.15,
        "an unsuited razz hand kept only {:.4} of its draws",
        loose.acceptance()
    );

    let gap = (loose.equities()[0].equity - tight.equities()[0].equity).abs();
    let slack = 5.0 * (loose.equities()[0].std_error.powi(2)
        + tight.equities()[0].std_error.powi(2))
    .sqrt();
    assert!(
        gap <= slack,
        "suits do not matter in razz, yet A23/A24 came to {:.5} and Ah2c3d/As2d4c to {:.5}",
        loose.equities()[0].equity,
        tight.equities()[0].equity
    );

    Ok(())
}

/// The shapes fix what a seat can be dealt, not how many seats can be dealt
/// at once, so the games that used to refuse an ordinary question now answer
/// it.
#[test]
fn test_open_suits_are_answerable_in_every_game() -> Result<(), PokerError> {
    use crate::odds::run_batch;
    use crate::variants::{Badugi, DeuceSeven, Omaha, OmahaFive, OmahaHiLo, Stud};

    macro_rules! answers {
        ($variant:expr, $hands:expr) => {{
            let request = EquityRequest::from_text($variant, &$hands, "", "")?;
            let result = run_batch(&request, 20_000, 5, 2)?;
            assert!(
                result.acceptance() > 0.1,
                "{:?} kept only {:.5} of its draws",
                $hands,
                result.acceptance()
            );
        }};
    }

    // Every one of these used to give up and report the question impossible.
    answers!(Omaha, ["AA**", "KK**"]);
    answers!(OmahaHiLo, ["AA**", "KK**"]);
    answers!(OmahaFive, ["AA***", "KK***"]);
    answers!(Badugi, ["A23", "A24"]);
    answers!(DeuceSeven, ["A234", "A235"]);
    answers!(Stud, ["A23", "A24"]);

    Ok(())
}
