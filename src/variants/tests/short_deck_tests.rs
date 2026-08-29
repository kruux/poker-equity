use crate::{
    cards::{CardSet, Rank},
    error::PokerError,
    hand::Hand,
    odds::{run_exact, EquityRequest},
    variants::{HighHandRank, PokerVariant, ShortDeck},
};

fn rank(cards: &str) -> Result<HighHandRank, PokerError> {
    Ok(Hand::from_str(ShortDeck, cards)?.evaluate().0)
}

/// The deck itself: thirty-six cards, sixes and up.
#[test]
fn test_the_deck_is_sixes_and_up() {
    let deck = ShortDeck.deck();
    assert_eq!(deck.len(), 36);
    assert_eq!(deck, CardSet::SHORT_DECK);
    for card in deck.iter() {
        assert!(
            card.rank().to_value() >= Rank::Six.to_value(),
            "{} should not be in a short deck",
            card
        );
    }
}

/// The ace plays low below the six, so A-6-7-8-9 is a straight and the lowest
/// straight flush is nine high rather than five high.
#[test]
fn test_the_ace_plays_low_below_the_six() -> Result<(), PokerError> {
    assert_eq!(rank("Ah 6c 7d 8s 9h")?, HighHandRank::Straight(Rank::Nine));
    assert_eq!(
        rank("Ah 6h 7h 8h 9h")?,
        HighHandRank::StraightFlush(Rank::Nine)
    );
    // And still high, at the other end.
    assert_eq!(rank("Ah Kc Qd Js Th")?, HighHandRank::Straight(Rank::Ace));
    Ok(())
}

/// The two rankings that move on a short deck.
#[test]
fn test_a_flush_beats_a_full_house_and_trips_beat_a_straight() -> Result<(), PokerError> {
    let flush = Hand::from_str(ShortDeck, "Ah Kh Qh Jh 9h")?;
    let full_house = Hand::from_str(ShortDeck, "6h 6c 6d 7h 7c")?;
    assert!(flush > full_house, "a flush beats a full house here");

    let trips = Hand::from_str(ShortDeck, "6h 6c 6d Kh Qs")?;
    let straight = Hand::from_str(ShortDeck, "Th Jc Qd Ks Ah")?;
    assert!(trips > straight, "trips beat a straight here");

    // The categories that did not move still rank as they always did.
    let quads = Hand::from_str(ShortDeck, "6h 6c 6d 6s Kh")?;
    assert!(quads > flush, "quads still beat a flush");
    assert!(full_house > trips, "a full house still beats trips");
    assert!(straight > Hand::from_str(ShortDeck, "6h 6c 7d 7s Kh")?, "a straight still beats two pair");

    Ok(())
}

/// A seven-card hand can hold both a straight and trips. With trips ranking
/// above a straight, it has to be classed as trips, not re-scored afterwards.
#[test]
fn test_a_hand_holding_both_is_classed_as_the_better_one() -> Result<(), PokerError> {
    // 6-7-8-9-T is a straight, and the three sixes are trips.
    let both = rank("6h 7c 8d 9s Th 6c 6d")?;
    assert!(
        matches!(both, HighHandRank::ThreeOfAKind(Rank::Six, _)),
        "trips outrank the straight, so the hand plays as trips: {}",
        both
    );
    Ok(())
}

/// Cards that are not in the deck are refused rather than quietly dealt
/// around.
#[test]
fn test_cards_below_a_six_are_not_in_this_game() {
    let request = EquityRequest::from_text(ShortDeck, &["Ah2c", "KsKc"], "", "");
    assert!(
        request.is_err(),
        "the deuce of clubs is not in a short deck"
    );
}

/// End to end, enumerated: the whole short deck runs out in C(32,5) boards.
#[test]
fn test_short_deck_equity_enumerates() -> Result<(), PokerError> {
    let result = run_exact(&EquityRequest::from_text(
        ShortDeck,
        &["AhAd", "KsKc"],
        "",
        "",
    )?)?
    .expect("thirty-two cards leave a small enough space");

    assert_eq!(result.samples, 201_376, "C(32,5) boards");
    let equities = result.equities();
    assert!((equities[0].equity + equities[1].equity - 1.0).abs() < 1e-9);
    assert!(
        equities[0].percent() > equities[1].percent(),
        "aces are still ahead"
    );
    // Aces hold up less well over a short deck than a full one, where the
    // same matchup is 81.3%.
    assert!(
        (equities[0].percent() - 74.200).abs() < 0.01,
        "expected 74.200%, got {:.3}%",
        equities[0].percent()
    );

    Ok(())
}
