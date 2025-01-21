use crate::{
    error::{EquityError, PokerError},
    hand::Hand,
    odds::EquityCalculator,
    variants::StudHiLo,
};

#[test]
fn test_not_enough_players() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(StudHiLo, 1);

    // Add just one player
    let hand = Hand::from_str(StudHiLo, "Ah Ac Ad")?;
    calculator.add_player("Alone".to_string(), hand)?;

    // Should error when calculating
    let result = calculator.calculate(drop);
    assert!(matches!(
        result,
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));

    Ok(())
}

#[test]
fn test_unequal_hand_sizes() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(StudHiLo, 1);

    // First player with 3 cards
    let hand1 = Hand::from_str(StudHiLo, "Ah Ac Ad")?;
    calculator.add_player("ThreeCards".to_string(), hand1)?;

    // Second player with 4 cards
    let hand2 = Hand::from_str(StudHiLo, "Kh Kc Kd Ks")?;
    calculator.add_player("FourCards".to_string(), hand2)?;

    // Should error when calculating
    let result = calculator.calculate(drop);
    assert!(matches!(
        result,
        Err(PokerError::Equity(EquityError::UnequalHandSizes))
    ));

    Ok(())
}

#[test]
fn test_too_few_cards() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(StudHiLo, 1);

    // Both players with only 2 cards
    let hand1 = Hand::from_str(StudHiLo, "Ah Ac")?;
    let hand2 = Hand::from_str(StudHiLo, "Kh Kc")?;

    calculator.add_player("TwoCards1".to_string(), hand1)?;
    calculator.add_player("TwoCards2".to_string(), hand2)?;

    // Should error when calculating
    let result = calculator.calculate(drop);
    assert!(matches!(
        result,
        Err(PokerError::Equity(EquityError::NotEnoughCards(2)))
    ));

    Ok(())
}

#[test]
fn test_high_only_equity() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(StudHiLo, 100000);

    // Two players with only high hands (no low possible)
    let aces = Hand::from_str(StudHiLo, "Ah Ac Ad Kh Qc")?;
    let kings = Hand::from_str(StudHiLo, "Ks Kc Kd Qh Jc")?;

    calculator.add_player("Aces".to_string(), aces)?;
    calculator.add_player("Kings".to_string(), kings)?;

    let results = calculator.calculate(drop)?;

    // Aces has around 75% equity
    assert!((results["Aces"] - 74.96).abs() < 0.5);
    assert!((results["Kings"] - 25.04).abs() < 0.5);

    Ok(())
}

#[test]
fn test_split_pot_equity() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(StudHiLo, 100000);

    // Player 1: Aces full (high) with no low
    let player1 = Hand::from_str(StudHiLo, "Ah Ac Ad Kh Kc")?;

    // Player 2: 8-6-4-3-2 low
    let player2 = Hand::from_str(StudHiLo, "8h 6c 4d 3h 2c")?;

    calculator.add_player("FullHouse".to_string(), player1)?;
    calculator.add_player("LowHand".to_string(), player2)?;

    let results = calculator.calculate(drop)?;

    // Player 1 should win high (50%), Player 2 should win low (50%)
    // Should always be the case so no randomness Add a fraction of percent uncertainty for floating point precision
    assert!((results["FullHouse"] - 50.0).abs() < 0.001);
    assert!((results["LowHand"] - 50.0).abs() < 0.001);

    Ok(())
}

#[test]
fn test_scoop_equity() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(StudHiLo, 100000);

    // Player 1: Has both good high (flush) and good low (8-6-4-3-2)
    let player1 = Hand::from_str(StudHiLo, "8h 7h 4h 3h 2h")?;

    // Player 2: Has lower high and no qualifying low
    let player2 = Hand::from_str(StudHiLo, "Kh Kc 9d 8c 7c")?;

    calculator.add_player("Flush".to_string(), player1)?;
    calculator.add_player("Pair".to_string(), player2)?;

    let results = calculator.calculate(drop)?;

    // Player 1 should win both high and low almost always (96.35%)
    assert!((results["Flush"] - 96.35).abs() < 0.5);
    assert!((results["Pair"] - 3.65).abs() < 0.5);

    Ok(())
}

#[test]
fn test_three_way_equity() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(StudHiLo, 100000);

    // Player 1: Best high hand only
    let player1 = Hand::from_str(StudHiLo, "Ah Ac Ad Kh Qc")?;

    // Player 2: Middle strength with 8-6-4-3-2 low
    let player2 = Hand::from_str(StudHiLo, "8h 6c 4d 3h 2c")?;

    // Player 3: Better low (7-4-3-2-A) but weak high
    let player3 = Hand::from_str(StudHiLo, "7h 4c 3d 2h As")?;

    calculator.add_player("Aces".to_string(), player1)?;
    calculator.add_player("EightLow".to_string(), player2)?;
    calculator.add_player("BetterLow".to_string(), player3)?;

    let results = calculator.calculate(drop)?;

    assert!((results["Aces"] - 38.35).abs() < 0.5);
    assert!((results["EightLow"] - 13.37).abs() < 0.5);
    assert!((results["BetterLow"] - 48.28).abs() < 0.5);

    Ok(())
}

#[test]
fn test_low_split_equity() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(StudHiLo, 100000);

    // Player 1: Best high and 8-6-4-3-2 low
    let player1 = Hand::from_str(StudHiLo, "8h 6c 4d 3h 2c Kh Kd")?;

    // Player 2: Weaker high but same 8-6-4-3-2 low
    let player2 = Hand::from_str(StudHiLo, "8d 6h 4c 3d 2h Qh Qd")?;

    calculator.add_player("BestHigh".to_string(), player1)?;
    calculator.add_player("SameLow".to_string(), player2)?;

    let results = calculator.calculate(drop)?;

    println!("BestHigh: {}", results["BestHigh"]);
    println!("SameLow: {}", results["SameLow"]);
    // Player 1 gets high (50%) plus half of low (25%) = 75%
    // Player 2 gets half of low (25%)
    assert!((results["BestHigh"] - 75.0).abs() < 0.001);
    assert!((results["SameLow"] - 25.0).abs() < 0.001);

    Ok(())
}

#[test]
fn test_six_player_three_card_equity() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(StudHiLo, 100000);

    // Two high pairs
    let kings = Hand::from_str(StudHiLo, "Kh Kc 9d")?; // Best high-only hand
    let queens = Hand::from_str(StudHiLo, "Qh Qc 8d")?; // Second high-only hand

    // Four different low draws
    let ace_low = Hand::from_str(StudHiLo, "Ah 3c 7d")?; // Ace + two low cards
    let low2 = Hand::from_str(StudHiLo, "2h 4c 6d")?;
    let low3 = Hand::from_str(StudHiLo, "3h 5c 4d")?;
    let low4 = Hand::from_str(StudHiLo, "4h 6c 8h")?;

    calculator.add_player("Kings".to_string(), kings)?;
    calculator.add_player("Queens".to_string(), queens)?;
    calculator.add_player("A37".to_string(), ace_low)?;
    calculator.add_player("246".to_string(), low2)?;
    calculator.add_player("345".to_string(), low3)?;
    calculator.add_player("468".to_string(), low4)?;

    let results = calculator.calculate(drop)?;

    // Make sure equity is 100%
    let total_equity: f64 = results.values().sum();
    assert!((total_equity - 100.0).abs() < 0.001);

    assert!((results["Kings"] - 18.67).abs() < 0.5);
    assert!((results["Queens"] - 13.00).abs() < 0.5);
    assert!((results["A37"] - 14.60).abs() < 0.5);
    assert!((results["246"] - 17.73).abs() < 0.5);
    assert!((results["345"] - 21.96).abs() < 0.5);
    assert!((results["468"] - 14.04).abs() < 0.5);

    Ok(())
}
