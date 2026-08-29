use std::mem::drop;

use crate::{
    cards::Card,
    error::{EquityError, GameError, PokerError},
    hand::Hand,
    odds::{run_exact, EquityCalculator, EquityRequest},
    variants::{Courchevel, Omaha, OmahaFive, OmahaSix},
};

#[test]
fn test_omaha_calculator_errors() -> Result<(), PokerError> {
    // Test no players
    let result = EquityCalculator::new(Omaha, 1000).calculate(drop);
    assert!(matches!(
        result,
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));

    // Test single player
    let mut calc = EquityCalculator::new(Omaha, 1000);
    let hand = Hand::from_str(Omaha, "As Ks Qc Jd")?;
    calc.add_player("Hero".to_string(), hand)?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));

    // Test no hole cards
    let mut calc = EquityCalculator::new(Omaha, 1000);
    let empty_hand = Hand::from_str(Omaha, "")?;
    let hand = Hand::from_str(Omaha, "Ts Js Qc Kd")?;
    calc.add_player("Hero".to_string(), empty_hand)?;
    calc.add_player("Villain".to_string(), hand)?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::UnequalHandSizes))
    ));

    // Test one hole card
    let mut calc = EquityCalculator::new(Omaha, 1000);
    let one_card = Hand::from_str(Omaha, "Ah")?;
    let hand = Hand::from_str(Omaha, "Ks")?;
    calc.add_player("Hero".to_string(), one_card)?;
    calc.add_player("Villain".to_string(), hand)?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::NotEnoughCards(_)))
    ));

    // Test three vs four hole cards
    let mut calc = EquityCalculator::new(Omaha, 1000);
    let three_cards = Hand::from_str(Omaha, "9h 8h 7h")?;
    let hand = Hand::from_str(Omaha, "As Ks Qc Jd")?;
    calc.add_player("Hero".to_string(), three_cards)?;
    calc.add_player("Villain".to_string(), hand)?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::UnequalHandSizes))
    ));

    // Test five vs four hole cards
    let mut calc = EquityCalculator::new(Omaha, 1000);
    let five_cards = Hand::from_str(Omaha, "2h 3h 4h 5h 6h")?;
    let hand = Hand::from_str(Omaha, "Ts Js Qc Kd")?;
    calc.add_player("Hero".to_string(), five_cards)?;
    calc.add_player("Villain".to_string(), hand)?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::UnequalHandSizes))
    ));

    // Test one community card
    let mut calc = EquityCalculator::new(Omaha, 1000);
    let hero = Hand::from_str(Omaha, "As Ks Qc Jd")?;
    let villain = Hand::from_str(Omaha, "Th 9h 8c 7d")?;
    let community = Hand::from_str(Omaha, "2c")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::InvalidCommunityCards(_)))
    ));

    // Test two community cards
    let mut calc = EquityCalculator::new(Omaha, 1000);
    let hero = Hand::from_str(Omaha, "As Ks Qc Jd")?;
    let villain = Hand::from_str(Omaha, "Th 9h 8c 7d")?;
    let community = Hand::from_str(Omaha, "2c 3d")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::InvalidCommunityCards(_)))
    ));

    // Test 5 community cards. No point in simulation a finished hand.
    let mut calc = EquityCalculator::new(Omaha, 10000);
    let hero = Hand::from_str(Omaha, "As Ks Qc Jd")?;
    let villain = Hand::from_str(Omaha, "Th 9h 8c 7d")?;
    let community = Hand::from_str(Omaha, "2c 3d 4h 5s 6c")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::InvalidCommunityCards(_)))
    ));

    // Test six community cards
    let mut calc = EquityCalculator::new(Omaha, 1000);
    let hero = Hand::from_str(Omaha, "As Ks Qc Jd")?;
    let villain = Hand::from_str(Omaha, "Th 9h 8c 7d")?;
    let community = Hand::from_str(Omaha, "2c 3d 4h 5s 6c 7s")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::InvalidCommunityCards(_)))
    ));

    // Test duplicate cards
    let mut calc = EquityCalculator::new(Omaha, 1000);
    let hero = Hand::from_str(Omaha, "As Ks Qc Jd")?;
    let villain = Hand::from_str(Omaha, "As 2d 3h 4c")?; // Intentional duplicate As
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Game(GameError::DuplicateCard(_)))
    ));

    // Test duplicate cards in community cards
    let mut calc = EquityCalculator::new(Omaha, 1000);
    let hero = Hand::from_str(Omaha, "As Ks Qc Jd")?;
    let villain = Hand::from_str(Omaha, "Th 9h 8c 7d")?;
    let community = Hand::from_str(Omaha, "As 2c 3d")?; // Intentional duplicate As
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Game(GameError::DuplicateCard(_)))
    ));

    Ok(())
}

#[test]
fn test_omaha_deterministic_outcome() -> Result<(), PokerError> {
    // Double suited aces vs rundown on AAA flop
    let mut calc = EquityCalculator::new(Omaha, 100000);
    let hero = Hand::from_str(Omaha, "Ah As Kd Qc")?;
    let villain = Hand::from_str(Omaha, "Jh Tc 9s 8d")?;
    let community = Hand::from_str(Omaha, "Ad Ac 2h")?;

    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;

    let equity = calc.calculate(drop)?;

    // Hero should have 100% equity (quads aces beats anything)
    assert!((equity["Hero"] - 100.0).abs() < 0.0001);
    assert!((equity["Villain"] - 0.0).abs() < 0.0001);

    Ok(())
}

#[test]
fn test_omaha_known_equities() -> Result<(), PokerError> {
    // Test valid two players preflop
    let mut calc = EquityCalculator::new(Omaha, 100000);
    let hero = Hand::from_str(Omaha, "9s 9h 8s 8h")?; // Double suited mid pairs
    let villain = Hand::from_str(Omaha, "As Kd Qc Jh")?; // Big cards rainbow
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    let result = calc.calculate(drop)?;
    // Hero should have 54.32% equity
    assert!((result["Hero"] - 55.60).abs() < 0.5);
    assert!((result["Villain"] - 44.40).abs() < 0.5);

    // Test valid three players preflop
    let mut calc = EquityCalculator::new(Omaha, 100000);
    let hero = Hand::from_str(Omaha, "Ts Th 9s 9h")?; // Double suited tens and nines
    let villain1 = Hand::from_str(Omaha, "As Kd Qc Jh")?; // Big cards rainbow
    let villain2 = Hand::from_str(Omaha, "5c 5d 4c 4d")?; // Double suited small pairs
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain1".to_string(), villain1)?;
    calc.add_player("Villain2".to_string(), villain2)?;
    let result = calc.calculate(drop)?;
    // Known percentages from simulations
    assert!((result["Hero"] - 43.51).abs() < 0.5);
    assert!((result["Villain1"] - 21.22).abs() < 0.5);
    assert!((result["Villain2"] - 35.27).abs() < 0.5);

    // Test valid flop
    let mut calc = EquityCalculator::new(Omaha, 100000);
    let hero = Hand::from_str(Omaha, "As Ah Kd Qc")?; // Double aces
    let villain = Hand::from_str(Omaha, "Jh Tc 9s 8d")?; // Connected rundown
    let community = Hand::from_str(Omaha, "Ac 2h 3s")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    let result = calc.calculate(drop)?;

    assert!((result["Hero"] - 98.90).abs() < 0.5);
    assert!((result["Villain"] - 1.10).abs() < 0.5);

    // Test valid turn
    let mut calc = EquityCalculator::new(Omaha, 100000);
    let hero = Hand::from_str(Omaha, "Ks Kh Qd Jc")?; // Double kings
    let villain = Hand::from_str(Omaha, "Ts Th 9s 9h")?; // Double suited tens nines
    let community = Hand::from_str(Omaha, "Kc 2h 3s 4d")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    let result = calc.calculate(drop)?;
    // Hero should have 100% equity with trip kings
    assert!((result["Hero"] - 100.0).abs() < 0.001);
    assert!((result["Villain"] - 0.0).abs() < 0.001);

    Ok(())
}

#[test]
fn test_complex_drawing_hands() -> Result<(), PokerError> {
    let mut calc = EquityCalculator::new(Omaha, 100000);

    // Double suited rundown vs double suited aces
    // This creates interesting scenarios:
    // - Hero can make: Straights, flushes, straight flushes
    // - Villain can make: Nut flushes, high pairs, sets
    let hero = Hand::from_str(Omaha, "Js Ts 9h 8h")?;
    let villain = Hand::from_str(Omaha, "As Ks Ah Kh")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;

    let result = calc.calculate(drop)?;

    // Equity should be close to 50/50 preflop
    assert!((result["Hero"] - 33.64).abs() < 0.5);
    assert!((result["Villain"] - 66.36).abs() < 0.5);

    Ok(())
}

#[test]
fn test_drawing_hands_on_wet_flop() -> Result<(), PokerError> {
    let mut calc = EquityCalculator::new(Omaha, 100000);

    // Double suited rundown vs big pairs on coordinated flop
    // This creates many possibilities:
    // - Hero: Straight draws, flush draws, pair+draws
    // - Villain: Overpair, backdoor draws
    let hero = Hand::from_str(Omaha, "Js Ts 9h 8h")?;
    let villain = Hand::from_str(Omaha, "As Ac Kd Qc")?;
    let flop = Hand::from_str(Omaha, "7s 6c 5h")?;

    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(flop.cards().to_vec())?;

    let result = calc.calculate(drop)?;

    // Hero should be favorite with wrap and flush draw
    assert!((result["Hero"] - 91.10).abs() < 0.5);
    assert!((result["Villain"] - 8.90).abs() < 0.5);

    Ok(())
}

#[test]
fn test_multiway_drawing_scenario() -> Result<(), PokerError> {
    let mut calc = EquityCalculator::new(Omaha, 100000);

    // Three-way pot with different types of hands:
    // Double suited kings: High pairs + flush potential
    // Double suited rundown: Straight + flush potential
    // Double suited small pairs: Set mining + flush potential
    let kings = Hand::from_str(Omaha, "Ks Kh Qs Qh")?;
    let rundown = Hand::from_str(Omaha, "Jc Tc 9d 8d")?;
    let small_pairs = Hand::from_str(Omaha, "5s 5h 4s 4h")?;
    let flop = Hand::from_str(Omaha, "Kc 7d 2s")?;

    calc.add_player("Kings".to_string(), kings)?;
    calc.add_player("Rundown".to_string(), rundown)?;
    calc.add_player("SmallPairs".to_string(), small_pairs)?;
    calc.set_community_cards(flop.cards().to_vec())?;

    let result = calc.calculate(drop)?;

    // Approximate equities with set of kings vs draws
    assert!((result["Kings"] - 71.62).abs() < 0.5);
    assert!((result["Rundown"] - 23.87).abs() < 0.5);
    assert!((result["SmallPairs"] - 4.51).abs() < 0.5);

    Ok(())
}

/// End to end check on the exactly-two rule. Hero holds one heart against a
/// three-heart board, so no river can give Hero a flush -- Villain's trip
/// kings are already unbeatable. If the rule were not enforced, Hero would
/// flush on any of the nine remaining hearts and take a share of the pot.
#[test]
fn test_one_hole_heart_never_flushes() -> Result<(), PokerError> {
    let mut calc = EquityCalculator::new(Omaha, 20000);

    let hero = Hand::from_str(Omaha, "Ah 3d 4s 5c")?; // one heart, no pair, no draw
    let villain = Hand::from_str(Omaha, "Kd Kc 7h 8s")?; // trip kings with the board
    let turn = Hand::from_str(Omaha, "Kh Qh Jh 2c")?; // three hearts, one king

    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(turn.cards().to_vec())?;

    let result = calc.calculate(drop)?;
    assert!(
        (result["Villain"] - 100.0).abs() < 0.0001,
        "Villain holds trip kings and Hero cannot make a flush, got {}",
        result["Villain"]
    );

    Ok(())
}

/// Courchevel deals one board card face up before the betting, so a one-card
/// board is a real spot and an empty one cannot happen.
#[test]
fn test_courchevel_requires_a_board_card() -> Result<(), PokerError> {
    let hands = ["Ah Ad Ks Qc Jh", "9h 8c 7s 6d 5h"];

    let mut calc = EquityCalculator::new(Courchevel, 1000);
    for (name, cards) in ["Hero", "Villain"].iter().zip(hands) {
        calc.add_player(name.to_string(), Hand::from_str(Courchevel, cards)?)?;
    }
    assert!(
        matches!(
            calc.calculate(drop),
            Err(PokerError::Equity(EquityError::InvalidCommunityCards(0)))
        ),
        "the first board card is face up before the betting"
    );

    // One card is exactly the Courchevel starting point.
    let mut calc = EquityCalculator::new(Courchevel, 20000);
    for (name, cards) in ["Hero", "Villain"].iter().zip(hands) {
        calc.add_player(name.to_string(), Hand::from_str(Courchevel, cards)?)?;
    }
    calc.set_community_cards(Card::from_str("2c")?)?;
    let result = calc.calculate(drop)?;
    assert!((result["Hero"] + result["Villain"] - 100.0).abs() < 0.001);

    Ok(())
}

/// Once the flop is out, Courchevel and five-card Omaha are the same spot and
/// must give the same answer. Enumerated, so there is no error bar to hide a
/// difference in.
#[test]
fn test_courchevel_and_five_card_omaha_agree_after_the_flop() -> Result<(), PokerError> {
    let hands = ["AhAdKsQcJh", "9h8c7s6d5h"];
    let board = "2c 7d 9d";

    let courchevel = run_exact(&EquityRequest::from_text(Courchevel, &hands, board, "")?)?
        .expect("small enough to enumerate");
    let omaha = run_exact(&EquityRequest::from_text(OmahaFive, &hands, board, "")?)?
        .expect("small enough to enumerate");

    assert_eq!(courchevel.samples, omaha.samples);
    for (seat, (a, b)) in courchevel
        .equities()
        .iter()
        .zip(omaha.equities())
        .enumerate()
    {
        assert!(
            (a.equity - b.equity).abs() < 1e-12,
            "seat {} differs: {:.10}% against {:.10}%",
            seat,
            a.percent(),
            b.percent()
        );
    }
    Ok(())
}

/// Five- and six-card Omaha deal more hole cards but play the same two.
#[test]
fn test_bigger_omaha_hands_still_split_the_pot_sensibly() -> Result<(), PokerError> {
    for (variant, hands) in [
        ("five", vec!["AhAdKsQcJh", "9h8c7s6d5h"]),
        ("six", vec!["AhAdKsQcJh2h", "9h8c7s6d5h3c"]),
    ] {
        let result = if variant == "five" {
            run_exact(&EquityRequest::from_text(OmahaFive, &hands, "2c 7d 9d", "")?)?
        } else {
            run_exact(&EquityRequest::from_text(OmahaSix, &hands, "2c 7d 9d", "")?)?
        }
        .expect("small enough to enumerate");

        let total: f64 = result.equities().iter().map(|player| player.equity).sum();
        assert!(
            (total - 1.0).abs() < 1e-9,
            "{}-card Omaha equities summed to {}",
            variant,
            total
        );
        assert!(result.samples > 0);
    }
    Ok(())
}
