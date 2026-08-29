use std::mem::drop;

use crate::{
    cards::Card,
    error::{EquityError, PokerError},
    hand::Hand,
    odds::EquityCalculator,
    variants::Razz,
};

#[test]
fn test_razz_validation_not_enough_players() -> Result<(), PokerError> {
    let mut calc = EquityCalculator::new(Razz, 100);
    let hand = Hand::from_str(Razz, "As Ks Qs")?;
    calc.add_player("Alice".to_string(), hand)?;

    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));
    Ok(())
}

#[test]
fn test_razz_validation_unequal_hand_sizes() -> Result<(), PokerError> {
    let mut calc = EquityCalculator::new(Razz, 100);
    let hand1 = Hand::from_str(Razz, "As Ks Qs")?;
    let hand2 = Hand::from_str(Razz, "Ad Kd")?;
    calc.add_player("Alice".to_string(), hand1)?;
    calc.add_player("Bob".to_string(), hand2)?;

    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::UnequalHandSizes))
    ));
    Ok(())
}

#[test]
fn test_razz_validation_not_enough_cards() -> Result<(), PokerError> {
    let mut calc = EquityCalculator::new(Razz, 100);
    let hand1 = Hand::from_str(Razz, "As Ks")?;
    let hand2 = Hand::from_str(Razz, "Ad Kd")?;
    calc.add_player("Alice".to_string(), hand1)?;
    calc.add_player("Bob".to_string(), hand2)?;

    assert!(matches!(
        calc.calculate(drop),
        Err(PokerError::Equity(EquityError::NotEnoughCards(2)))
    ));
    Ok(())
}

#[test]
fn test_razz_equity_wheel_vs_king_low() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(Razz, 100000);

    // Player 1 with A2345
    let hand1 = Hand::from_str(Razz, "Ah 2h 3h 4h 5h")?;
    // Player 2 with K2345
    let hand2 = Hand::from_str(Razz, "Kh 2d 3d 4d 5d")?;

    calculator.add_player("Wheel".to_string(), hand1)?;
    calculator.add_player("King".to_string(), hand2)?;

    let results = calculator.calculate(drop)?;

    // Wheel should have 92.98% equity
    // Measured over eight million deals; one standard error at a hundred
    // thousand is 0.05, so the window below is six of them.
    assert!((results["Wheel"] - 93.035).abs() < 0.35);
    assert!((results["King"] - 6.965).abs() < 0.35);

    Ok(())
}

#[test]
fn test_razz_equity_tied_hands() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(Razz, 100000);

    // Two players with identical 23456
    let hand1 = Hand::from_str(Razz, "2h 3h 4h 5h 6h")?;
    let hand2 = Hand::from_str(Razz, "2d 3d 4d 5d 6d")?;

    calculator.add_player("Player1".to_string(), hand1)?;
    calculator.add_player("Player2".to_string(), hand2)?;

    let results = calculator.calculate(drop)?;

    // Should split equity 50-50
    assert!((results["Player1"] - 50.0).abs() < 0.5);
    assert!((results["Player2"] - 50.0).abs() < 0.5);

    Ok(())
}

#[test]
fn test_razz_equity_three_players() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(Razz, 100000);

    // Three players with initial 3-card hands
    let hand1 = Hand::from_str(Razz, "Ah 2h 3h")?;
    let hand2 = Hand::from_str(Razz, "4d 5d 6d")?;
    let hand3 = Hand::from_str(Razz, "7c 8c 9c")?;

    calculator.add_player("Low".to_string(), hand1)?;
    calculator.add_player("Mid".to_string(), hand2)?;
    calculator.add_player("High".to_string(), hand3)?;

    let results = calculator.calculate(drop)?;

    // Low should have best equity, High should have worst
    // One standard error here is about 0.17, so the window is six of them.
    // It used to be half a point around a figure that was itself a little
    // off, leaving one edge under three standard errors away.
    assert!((results["Low"] - 43.806).abs() < 1.0);
    assert!((results["Mid"] - 38.805).abs() < 1.0);
    assert!((results["High"] - 17.389).abs() < 1.0);

    // Sum should be 100%
    let total: f64 = results.values().sum();
    assert!((total - 100.0).abs() < 0.001);

    Ok(())
}

#[test]
fn test_razz_equity_drawing_hands() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(Razz, 100000);

    // Player 1 with 3 cards to a wheel (A23)
    let hand1 = Hand::from_str(Razz, "Ah 2h 3h")?;
    // Player 2 with 3 middling cards (567)
    let hand2 = Hand::from_str(Razz, "5h 6h 7h")?;

    calculator.add_player("LowDraw".to_string(), hand1)?;
    calculator.add_player("MidDraw".to_string(), hand2)?;

    let results = calculator.calculate(drop)?;

    // One standard error is about 0.12; six of them is the window below.
    assert!((results["LowDraw"] - 55.343).abs() < 0.75);
    assert!((results["MidDraw"] - 44.657).abs() < 0.75);

    Ok(())
}

#[test]
fn test_razz_dead_cards() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(Razz, 100000);

    // Player 1 with A23
    let hand1 = Hand::from_str(Razz, "Ah 2h 3h")?;
    // Player 2 with 456
    let hand2 = Hand::from_str(Razz, "4h 5h 6h")?;

    // Add low cards as dead cards to hurt Player 2's equity
    let dead_cards = Card::from_str("Ad 2d 3d")?;

    calculator.add_player("LowDraw".to_string(), hand1)?;
    calculator.add_player("MidDraw".to_string(), hand2)?;
    calculator.add_dead_cards(dead_cards)?;

    let results = calculator.calculate(drop)?;

    // LowDraw should have increased equity with the dead cards. One standard
    // error is about 0.16, so the window is six of them.
    assert!((results["LowDraw"] - 61.900).abs() < 1.0);
    assert!((results["MidDraw"] - 38.100).abs() < 1.0);

    Ok(())
}

/// End to end: two complete holdings sharing the same four distinct ranks are
/// not a chop. The lower pair takes the whole pot.
#[test]
fn test_paired_low_wins_outright() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(Razz, 10);

    let hero = Hand::from_str(Razz, "2c 2d 2h 3c 3d 4c 5d")?; // plays 5-4-3-2-2
    let villain = Hand::from_str(Razz, "4h 4s 4d 5h 5s 3h 2s")?; // plays 5-4-4-3-2

    calculator.add_player("Hero".to_string(), hero)?;
    calculator.add_player("Villain".to_string(), villain)?;

    // Both hands are complete, so there is nothing left to sample.
    let results = calculator.calculate(drop)?;
    assert!(
        (results["Hero"] - 100.0).abs() < 0.001,
        "Hero's pair of deuces beats Villain's pair of fours, got {}",
        results["Hero"]
    );

    Ok(())
}
