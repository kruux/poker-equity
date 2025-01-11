use std::mem::drop;

use crate::{
    error::{EquityError, PokerError},
    hand::Hand,
    odds::EquityCalculator,
    variants::SevenCardStud,
};

#[test]
fn test_stud_validation_errors() -> Result<(), PokerError> {
    let stud = SevenCardStud;

    // Less than 2 players
    let mut calc = EquityCalculator::new(stud, 100);
    let alice_hand = Hand::from_str(stud, "As Ks Qs")?;
    calc.add_player("Alice".to_string(), alice_hand)?;
    assert!(matches!(
        calc.calculate(),
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));

    // Unequal hand sizes
    let mut calc = EquityCalculator::new(stud, 100);
    let alice_hand = Hand::from_str(stud, "As Ks Qs")?;
    let bob_hand = Hand::from_str(stud, "Ad Kd")?;
    calc.add_player("Alice".to_string(), alice_hand)?;
    calc.add_player("Bob".to_string(), bob_hand)?;
    assert!(matches!(
        calc.calculate(),
        Err(PokerError::Equity(EquityError::UnequalHandSizes))
    ));

    // Less than 3 cards
    let mut calc = EquityCalculator::new(stud, 100);
    let alice_hand = Hand::from_str(stud, "As Ks")?;
    let bob_hand = Hand::from_str(stud, "Ad Kd")?;
    calc.add_player("Alice".to_string(), alice_hand)?;
    calc.add_player("Bob".to_string(), bob_hand)?;
    assert!(matches!(
        calc.calculate(),
        Err(PokerError::Equity(EquityError::NotEnoughCards(2)))
    ));

    Ok(())
}

#[test]
fn test_stud_100_percent() -> Result<(), PokerError> {
    let stud = SevenCardStud;
    let mut calc = EquityCalculator::new(stud, 100);

    // Player 1 has royal flush, Player 2 has worse hand
    let royal_hand = Hand::from_str(stud, "Ah Kh Qh Jh Th 9s 8s")?;
    let sf_hand = Hand::from_str(stud, "2c 3c 4c 5c 6c 7d 8d")?;

    calc.add_player("Royal".to_string(), royal_hand)?;
    calc.add_player("Lower".to_string(), sf_hand)?;

    let results = calc.calculate()?;
    assert_eq!(results["Royal"], 100.0);
    assert_eq!(results["Lower"], 0.0);

    Ok(())
}

#[test]
fn test_equity_identical_hands() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(SevenCardStud, 1000);

    // Two players both with 5-card royal flushes
    let alice_hand = Hand::from_str(SevenCardStud, "Ah Kh Qh Jh Th")?;
    let bob_hand = Hand::from_str(SevenCardStud, "As Ks Qs Js Ts")?;

    calculator.add_player("Alice".to_string(), alice_hand)?;
    calculator.add_player("Bob".to_string(), bob_hand)?;

    let results = calculator.calculate()?;

    // Should split equity
    assert!((results["Alice"] - 50.0).abs() < 1.0);
    assert!((results["Bob"] - 50.0).abs() < 1.0);

    Ok(())
}

#[test]
fn test_equity_quads_vs_pair() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(SevenCardStud, 10000);

    // Quad aces
    let alice_hand = Hand::from_str(SevenCardStud, "Ah Ac Ad As 2h")?;
    // Pair of kings
    let bob_hand = Hand::from_str(SevenCardStud, "Kh Kc 2c 3d 4s")?;

    calculator.add_player("Alice".to_string(), alice_hand)?;
    calculator.add_player("Bob".to_string(), bob_hand)?;

    let results = calculator.calculate()?;

    // Quads should always win here
    assert!((results["Alice"] - 100.0).abs() < 1.0);
    assert!((results["Bob"] - 0.0).abs() < 1.0);

    Ok(())
}

#[test]
fn test_equity_trips_vs_three_cards() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(SevenCardStud, 10000);

    // Three aces vs Q72 rainbow
    let alice_hand = Hand::from_str(SevenCardStud, "As Ah Ad")?;
    let bob_hand = Hand::from_str(SevenCardStud, "Qs 7h 2d")?;

    calculator.add_player("Alice".to_string(), alice_hand)?;
    calculator.add_player("Bob".to_string(), bob_hand)?;

    let results = calculator.calculate()?;

    // Known percentages from 600k simulations: 97.92% vs 2.08%
    assert!((results["Alice"] - 97.92).abs() < 1.0);
    assert!((results["Bob"] - 2.08).abs() < 1.0);

    Ok(())
}

#[test]
fn test_equity_trips_vs_flush_draw() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(SevenCardStud, 10000);

    // Three aces vs three cards to a flush
    let alice_hand = Hand::from_str(SevenCardStud, "As Ah Ad")?;
    let bob_hand = Hand::from_str(SevenCardStud, "7c 6c 5c")?;

    calculator.add_player("Alice".to_string(), alice_hand)?;
    calculator.add_player("Bob".to_string(), bob_hand)?;

    let results = calculator.calculate()?;

    // Known percentages from 600k simulations: 76.61% vs 23.39%
    assert!((results["Alice"] - 76.61).abs() < 1.0);
    assert!((results["Bob"] - 23.39).abs() < 1.0);

    Ok(())
}

#[test]
fn test_equity_three_way_trips_vs_draws() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(SevenCardStud, 10000);

    // Three deuces vs club flush draw vs heart flush draw
    let alice_hand = Hand::from_str(SevenCardStud, "2s 2h 2d")?;
    let bob_hand = Hand::from_str(SevenCardStud, "7c 6c 5c")?;
    let charlie_hand = Hand::from_str(SevenCardStud, "Th 9h 8h")?;

    calculator.add_player("Alice".to_string(), alice_hand)?;
    calculator.add_player("Bob".to_string(), bob_hand)?;
    calculator.add_player("Charlie".to_string(), charlie_hand)?;

    let results = calculator.calculate_with_updates(drop)?;

    // Known percentages from simulations: 58.98% vs 20.89% vs 20.13%
    assert!((results["Alice"] - 58.98).abs() < 1.0);
    assert!((results["Bob"] - 20.89).abs() < 1.0);
    assert!((results["Charlie"] - 20.13).abs() < 1.0);

    Ok(())
}

#[test]
fn test_close_equity_stud() -> Result<(), PokerError> {
    // 77x vs AJ9 with 2 cards in a flush draw
    let mut calculator = EquityCalculator::new(SevenCardStud, 10000);

    let alice_hand = Hand::from_str(SevenCardStud, "7h 7c 2s")?;
    let bob_hand = Hand::from_str(SevenCardStud, "Ah Jh 9d")?;

    calculator.add_player("Alice".to_string(), alice_hand)?;
    calculator.add_player("Bob".to_string(), bob_hand)?;

    let results = calculator.calculate()?;

    // Add expected percentages once you have them
    println!("Alice: {}", results["Alice"]);
    println!("Bob: {}", results["Bob"]);

    // Add equities
    assert!(false);

    Ok(())
}
