use crate::{
    cards::Rank,
    error::PokerError,
    hand::Hand,
    odds::{run_exact, EquityRequest},
    variants::{
        CourchevelHiLo, HasLow, HighHandRank, LowHandRank, OmahaFiveHiLo, OmahaHiLo, PokerVariant,
    },
};

/// The two halves are searched separately, so a player can win the high with
/// one pair of hole cards and the low with another.
#[test]
fn test_the_two_halves_may_use_different_hole_cards() -> Result<(), PokerError> {
    // Kings and queens for the high; ace and deuce for the low.
    let hand = Hand::from_str(OmahaHiLo, "Ah 2c Kh Qh Kd Qc 5s 6d 7h")?;
    let rank = hand.evaluate();

    assert_eq!(
        rank.high,
        HighHandRank::TwoPair(Rank::King, Rank::Queen, Rank::Seven),
        "the high plays the king and queen from hand"
    );
    assert_eq!(
        rank.low(),
        Some(&LowHandRank::Low(vec![
            Rank::Seven,
            Rank::Six,
            Rank::Five,
            Rank::Two,
            Rank::Ace
        ])),
        "the low plays the ace and deuce from the same hand"
    );

    Ok(())
}

/// A low needs three board cards of eight or lower, so a high board cannot be
/// split however good the hand is.
#[test]
fn test_a_high_board_makes_no_low() -> Result<(), PokerError> {
    let rank = Hand::from_str(OmahaHiLo, "Ah 2c 3d 4s Kd Qc 9h Jc Th")?.evaluate();
    assert_eq!(
        rank.low(),
        None,
        "the nut low draw is worth nothing without low cards on the board"
    );
    Ok(())
}

/// Eight-or-better means eight or better, and the qualifier is on the card
/// that leads the low.
#[test]
fn test_the_low_qualifier_is_enforced() -> Result<(), PokerError> {
    // A-2-3-4-9 does not qualify: the nine is too high.
    let misses = Hand::from_str(OmahaHiLo, "Ah 2c Kh Qh 3d 4s 9c Jd Th")?.evaluate();
    assert_eq!(misses.low(), None, "a nine-low does not qualify");

    // Swap the nine for an eight and it does.
    let makes = Hand::from_str(OmahaHiLo, "Ah 2c Kh Qh 3d 4s 8c Jd Th")?.evaluate();
    assert!(makes.low().is_some(), "an eight-low qualifies");

    Ok(())
}

/// The whole pot goes to the high hand when nobody makes a low, and splits
/// when somebody does.
#[test]
fn test_the_pot_splits_only_when_a_low_exists() -> Result<(), PokerError> {
    // A board with no low: the high hand scoops.
    let high_board = run_exact(&EquityRequest::from_text(
        OmahaHiLo,
        &["Ah2c3d4s", "KhKsQhQs"],
        "Kd Qc 9h Jc Th",
        "",
    )?)?
    .expect("a full board enumerates to one deal");
    let equities = high_board.equities();
    assert_eq!(equities[1].percent(), 100.0, "kings full scoops");
    assert_eq!(equities[0].low_equity, 0.0, "there is no low half to win");

    // A low needs *three* board cards of eight or lower, so 5-6-8 splits the
    // pot where 5-6-9 would not.
    let split = run_exact(&EquityRequest::from_text(
        OmahaHiLo,
        &["Ah2c3d4s", "KhKsQhQs"],
        "5c 6d 8h Jc Th",
        "",
    )?)?
    .expect("a full board enumerates to one deal");
    let equities = split.equities();
    assert!(
        (equities[0].low_equity - 0.5).abs() < 1e-12,
        "A-2 on a 5-6-8 board is the nut low, worth exactly half"
    );
    assert!(
        (equities[0].equity - 0.5).abs() < 1e-12,
        "and the high half goes to the kings"
    );

    Ok(())
}

/// Shares still divide one pot, halves and all.
#[test]
fn test_split_equities_sum_to_one() -> Result<(), PokerError> {
    for hands in [
        vec!["Ah2c3d4s", "KhKsQhQs"],
        vec!["AhAd2c3d", "KhKsQhQs", "7h8s9dTc"],
    ] {
        let result = run_exact(&EquityRequest::from_text(
            OmahaHiLo, &hands, "5c 6d 9h", "",
        )?)?
        .expect("small enough to enumerate");
        let total: f64 = result.equities().iter().map(|player| player.equity).sum();
        assert!((total - 1.0).abs() < 1e-9, "summed to {}", total);
    }
    Ok(())
}

/// The split-pot family deals what it should.
#[test]
fn test_the_split_family_deals_what_it_should() {
    assert_eq!((OmahaHiLo.hole_cards(), OmahaHiLo.board_cards()), (4, 5));
    assert_eq!(
        (OmahaFiveHiLo.hole_cards(), OmahaFiveHiLo.board_cards()),
        (5, 5)
    );
    assert_eq!(
        (CourchevelHiLo.hole_cards(), CourchevelHiLo.board_cards()),
        (5, 5)
    );
}

/// Courchevel hi/lo is five-card Omaha hi/lo once the flop is out.
#[test]
fn test_courchevel_hi_lo_matches_five_card_omaha_hi_lo() -> Result<(), PokerError> {
    let cards = "Ah 2c 3d Ks Qh 5c 6d 9h Jc Th";
    assert_eq!(
        Hand::from_str(CourchevelHiLo, cards)?.evaluate(),
        Hand::from_str(OmahaFiveHiLo, cards)?.evaluate()
    );
    Ok(())
}
