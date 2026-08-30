use std::mem::drop;

use crate::{
    error::{EquityError, GameError, PokerError},
    hand::Hand,
    odds::EquityCalculator,
    variants::Holdem,
};

#[test]
fn test_holdem_calculator_errors() -> Result<(), PokerError> {
    // Test no players
    let result = EquityCalculator::new(Holdem, 1000).calculate(drop);
    assert!(matches!(
        result,
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));

    // Test single player
    let mut calc = EquityCalculator::new(Holdem, 1000);
    let hand = Hand::from_str(Holdem, "Ah Kh")?;
    calc.add_player("Hero".to_string(), hand)?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));

    // Test no hole cards
    let mut calc = EquityCalculator::new(Holdem, 1000);
    let empty_hand = Hand::from_str(Holdem, "")?;
    let hand = Hand::from_str(Holdem, "Ah Kh")?;
    calc.add_player("Hero".to_string(), empty_hand)?;
    calc.add_player("Villain".to_string(), hand)?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::UnequalHandSizes))
    ));

    // Test one hole card
    let mut calc = EquityCalculator::new(Holdem, 1000);
    let one_card = Hand::from_str(Holdem, "Ah")?;
    let hand = Hand::from_str(Holdem, "Ks")?;
    calc.add_player("Hero".to_string(), one_card)?;
    calc.add_player("Villain".to_string(), hand)?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::NotEnoughCards(_)))
    ));

    // Test three vs two hole cards
    let mut calc = EquityCalculator::new(Holdem, 1000);
    let three_cards = Hand::from_str(Holdem, "Ah Kh Qh")?;
    let hand = Hand::from_str(Holdem, "2h 2d")?;
    calc.add_player("Hero".to_string(), three_cards)?;
    calc.add_player("Villain".to_string(), hand)?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::UnequalHandSizes))
    ));

    // Test three vs three hole cards
    let mut calc = EquityCalculator::new(Holdem, 1000);
    let three_cards = Hand::from_str(Holdem, "Ah Kh Qh")?;
    let three_cards2 = Hand::from_str(Holdem, "2h 2d 3d")?;
    calc.add_player("Hero".to_string(), three_cards)?;
    calc.add_player("Villain".to_string(), three_cards2)?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::NotEnoughCards(_)))
    ));

    // Test one community card
    let mut calc = EquityCalculator::new(Holdem, 1000);
    let hero = Hand::from_str(Holdem, "Ah Kh")?;
    let villain = Hand::from_str(Holdem, "2h 2d")?;
    let community = Hand::from_str(Holdem, "Ac")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::InvalidCommunityCards(_)))
    ));

    // Test two community cards
    let mut calc = EquityCalculator::new(Holdem, 1000);
    let hero = Hand::from_str(Holdem, "Ah Kh")?;
    let villain = Hand::from_str(Holdem, "2h 2d")?;
    let community = Hand::from_str(Holdem, "Ac Kc")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::InvalidCommunityCards(_)))
    ));

    // Test 5 community cards. No point in simulation a finished hand.
    let mut calc = EquityCalculator::new(Holdem, 10000);
    let hero = Hand::from_str(Holdem, "Ah Kh")?;
    let villain = Hand::from_str(Holdem, "2h 2d")?;
    let community = Hand::from_str(Holdem, "Ac Kc Qc Jc Tc")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::InvalidCommunityCards(_)))
    ));

    // Test six community cards
    let mut calc = EquityCalculator::new(Holdem, 1000);
    let hero = Hand::from_str(Holdem, "Ah Kh")?;
    let villain = Hand::from_str(Holdem, "2h 2d")?;
    let community = Hand::from_str(Holdem, "Ac Kc Qc Jc Tc 9c")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::InvalidCommunityCards(_)))
    ));

    // Test duplicate cards
    let mut calc = EquityCalculator::new(Holdem, 1000);
    let hero = Hand::from_str(Holdem, "Ah Kh")?;
    let villain = Hand::from_str(Holdem, "Ah 2d")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Game(GameError::DuplicateCard(_)))
    ));

    // Test duplicate cards in community cards
    let mut calc = EquityCalculator::new(Holdem, 1000);
    let hero = Hand::from_str(Holdem, "Ah Kh")?;
    let villain = Hand::from_str(Holdem, "2h 2d")?;
    let community = Hand::from_str(Holdem, "Kh Qc Jc")?;
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
fn test_holdem_deterministic_outcome() -> Result<(), PokerError> {
    // AK vs 77 on KKK flop - AK should win 100% of the time
    let mut calc = EquityCalculator::new(Holdem, 10000);
    let hero = Hand::from_str(Holdem, "Ah Kd")?;
    let villain = Hand::from_str(Holdem, "7h 7d")?;
    let community = Hand::from_str(Holdem, "Kh Ks Kc")?;

    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;

    let equity = calc.calculate(drop)?;

    // Hero should have 100% equity (AK beats 77 on KKK board)
    assert_eq!(equity["Hero"], 100.0);
    assert_eq!(equity["Villain"], 0.0);

    Ok(())
}

#[test]
fn test_holdem_known_equities() -> Result<(), PokerError> {
    // Test valid two players preflop
    let mut calc = EquityCalculator::new(Holdem, 100000);
    let hero = Hand::from_str(Holdem, "Ah Kh")?;
    let villain = Hand::from_str(Holdem, "2h 2d")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    let result = calc.calculate(drop)?;
    // Hero should have 49.70% equity
    assert!((result["Hero"] - 49.70).abs() < 0.8);
    assert!((result["Villain"] - 50.30).abs() < 0.8);

    // Test valid three players preflop
    let mut calc = EquityCalculator::new(Holdem, 100000);
    let hero = Hand::from_str(Holdem, "Ah Kh")?;
    let villain1 = Hand::from_str(Holdem, "2h 2d")?;
    let villain2 = Hand::from_str(Holdem, "Qc Qd")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain1".to_string(), villain1)?;
    calc.add_player("Villain2".to_string(), villain2)?;
    let result = calc.calculate(drop)?;
    // Hero should have 38.20% equity, villain1 should have 16.93% equity, villain2 should have 44.87% equity
    assert!((result["Hero"] - 38.20).abs() < 0.8);
    assert!((result["Villain1"] - 16.93).abs() < 0.6);
    assert!((result["Villain2"] - 44.87).abs() < 0.8);

    // Test valid flop
    let mut calc = EquityCalculator::new(Holdem, 100000);
    let hero = Hand::from_str(Holdem, "Ah Kh")?;
    let villain = Hand::from_str(Holdem, "2h 2d")?;
    let community = Hand::from_str(Holdem, "Ac Kc Qc")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    let result = calc.calculate(drop)?;
    // Hero should have 89.90% equity
    assert!((result["Hero"] - 89.90).abs() < 0.5);
    assert!((result["Villain"] - 10.10).abs() < 0.5);

    // Test valid turn
    let mut calc = EquityCalculator::new(Holdem, 100000);
    let hero = Hand::from_str(Holdem, "Ah Kh")?;
    let villain = Hand::from_str(Holdem, "2h 2d")?;
    let community = Hand::from_str(Holdem, "Ac Kc Qc Jc")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(community.cards().to_vec())?;
    let result = calc.calculate(drop)?;
    // Hero should have 84.09% equity
    assert!((result["Hero"] - 84.09).abs() < 0.6);
    assert!((result["Villain"] - 15.91).abs() < 0.6);

    Ok(())
}

#[test]
fn test_complex_drawing_hands() -> Result<(), PokerError> {
    let mut calc = EquityCalculator::new(Holdem, 100000);

    // AcTs vs 6c7c
    // This creates interesting scenarios:
    // - Hero can make: Ace high, pair of aces, pair of tens, two pair
    // - Villain can make: flush, straight, pair(s), two pair
    let hero = Hand::from_str(Holdem, "Ac Ts")?;
    let villain = Hand::from_str(Holdem, "6c 7c")?;
    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;

    let result = calc.calculate(drop)?;

    // Equity should be around 55.5% for AcTs vs 44.5% for 6c7c
    assert!((result["Hero"] - 59.791).abs() < 1.0);
    assert!((result["Villain"] - 40.209).abs() < 1.0);

    Ok(())
}

#[test]
fn test_drawing_hands_on_wet_flop() -> Result<(), PokerError> {
    let mut calc = EquityCalculator::new(Holdem, 100000);

    // AcTs vs 6c7c on Jc8c3h flop
    // This creates many possibilities:
    // - Hero: overcards, backdoor straight draws
    // - Villain: flush draw, open-ended straight draw
    let hero = Hand::from_str(Holdem, "Ac Ts")?;
    let villain = Hand::from_str(Holdem, "6c 7c")?;
    let flop = Hand::from_str(Holdem, "Jc 8c 3h")?;

    calc.add_player("Hero".to_string(), hero)?;
    calc.add_player("Villain".to_string(), villain)?;
    calc.set_community_cards(flop.cards().to_vec())?;

    let result = calc.calculate(drop)?;

    // Villain should be a favorite due to flush and straight draws
    assert!((result["Hero"] - 50.990).abs() < 0.9);
    assert!((result["Villain"] - 49.010).abs() < 0.9);

    Ok(())
}

#[test]
fn test_multiway_drawing_scenario() -> Result<(), PokerError> {
    let mut calc = EquityCalculator::new(Holdem, 100000);

    // Three-way pot with different types of hands:
    // AhKd: high cards
    // JsTs: straight draw
    // 5h5c: pocket pair
    let ak_off = Hand::from_str(Holdem, "Ah Kd")?;
    let jt_suited = Hand::from_str(Holdem, "Js Ts")?;
    let pocket = Hand::from_str(Holdem, "5h 5c")?;
    let flop = Hand::from_str(Holdem, "Qc 9h 4s")?;

    calc.add_player("HighCards".to_string(), ak_off)?;
    calc.add_player("StraightDraw".to_string(), jt_suited)?;
    calc.add_player("PocketPair".to_string(), pocket)?;
    calc.set_community_cards(flop.cards().to_vec())?;

    let result = calc.calculate(drop)?;

    // Approximate equities (verify these numbers):
    assert!((result["HighCards"] - 11.867).abs() < 0.9);
    assert!((result["StraightDraw"] - 50.374).abs() < 1.05);
    assert!((result["PocketPair"] - 37.759).abs() < 0.85);

    Ok(())
}
