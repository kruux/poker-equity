use crate::{
    cards::{Card, Rank},
    error::PokerError,
    hand::Hand,
    variants::{Badugi, BadugiHandRank},
};

fn badugi(cards: &str) -> Result<Vec<Rank>, PokerError> {
    let BadugiHandRank::Low(ranks) = Hand::from_str(Badugi, cards)?.evaluate();
    Ok(ranks)
}

/// A badugi is the largest set of cards sharing neither a rank nor a suit.
#[test]
fn test_a_badugi_repeats_neither_rank_nor_suit() -> Result<(), PokerError> {
    // Four suits, four ranks: all four cards play.
    assert_eq!(
        badugi("Ac 2d 3h 4s")?,
        vec![Rank::Four, Rank::Three, Rank::Two, Rank::Ace],
        "the best hand in the game"
    );

    // Two clubs, so one of them has to go: a three-card badugi.
    assert_eq!(
        badugi("Ac 2c 3h 4s")?.len(),
        3,
        "two clubs cannot both play"
    );

    // Two deuces, likewise.
    assert_eq!(
        badugi("Ac 2d 2h 4s")?.len(),
        3,
        "two deuces cannot both play"
    );

    // All one suit: only one card plays.
    assert_eq!(badugi("Ac 2c 3c 4c")?, vec![Rank::Ace], "a one-card badugi");

    Ok(())
}

/// When several subsets are the same size, the lowest one plays.
#[test]
fn test_the_lowest_of_the_largest_subsets_plays() -> Result<(), PokerError> {
    // Ac and Kc clash. Dropping the king leaves A-2-3, which is lower than
    // the K-2-3 that dropping the ace would leave.
    assert_eq!(
        badugi("Ac Kc 2d 3h")?,
        vec![Rank::Three, Rank::Two, Rank::Ace],
        "the king is the card to drop"
    );

    // Both nines clash with nothing else, so the lower cards are kept.
    assert_eq!(
        badugi("9c 9d 2h 3s")?,
        vec![Rank::Nine, Rank::Three, Rank::Two],
        "one nine plays alongside the three and deuce"
    );

    Ok(())
}

/// More cards beats fewer, however low the shorter hand is, and within a size
/// the lower hand wins with the ace playing low.
#[test]
fn test_badugi_hands_compare_by_size_then_by_height() -> Result<(), PokerError> {
    let four_card_king = Hand::from_str(Badugi, "Kc Qd Jh Ts")?;
    let three_card_wheel = Hand::from_str(Badugi, "Ac 2c 3h 4s")?;
    assert!(
        four_card_king > three_card_wheel,
        "any four-card badugi beats any three-card one"
    );

    let lower = Hand::from_str(Badugi, "Ac 2d 3h 5s")?;
    let higher = Hand::from_str(Badugi, "Ac 2d 3h 6s")?;
    assert!(lower > higher, "a five-high badugi beats a six-high one");

    // The ace plays low, so it is the best card to hold, not the worst.
    let with_ace = Hand::from_str(Badugi, "Ac 2d 3h 4s")?;
    let with_king = Hand::from_str(Badugi, "Kc 2d 3h 4s")?;
    assert!(with_ace > with_king, "the ace is the lowest card");

    Ok(())
}

/// End to end: a pat four-card badugi against a hand drawing one.
///
/// Villain throws a spade and takes one card from the forty-four left, so
/// there is nothing to sample -- every deal is walked and the figure is
/// exact.
#[test]
fn test_a_pat_badugi_beats_a_drawing_hand() -> Result<(), PokerError> {
    use crate::odds::{run_exact, EquityRequest};

    // Hero stands pat with a near-perfect badugi. Villain holds two spades
    // and throws one away, which is named as a dead card.
    let request = EquityRequest::from_text(Badugi, &["Ac2d3h5s", "4s7d8h"], "", "6s")?;
    let result = run_exact(&request)?.expect("one card from forty-four");
    let shares = result.equities();

    assert_eq!(result.samples, 44, "one deal per card left in the deck");
    assert!(
        (shares[0].percent() + shares[1].percent() - 100.0).abs() < 1e-9,
        "equities must divide one pot"
    );
    assert!(
        shares[0].percent() > 90.0,
        "a five-high badugi is a long way ahead of a one-card draw, got {:.2}%",
        shares[0].percent()
    );

    Ok(())
}

/// Badugi's two answers must agree, and here they can be made to agree over
/// every holding there is. `score` packs the ordering straight out of two
/// bitmasks; `evaluate` names the badugi by trying all fifteen subsets. They
/// are written separately, so any four cards they place differently is a bug
/// in one of them.
///
/// All 270,725 four-card holdings, which is cheap enough to do outright.
#[test]
fn test_badugis_score_and_name_agree_everywhere() {
    use crate::variants::PokerVariant;

    /// The ordering `score` produces, read off a named badugi instead.
    fn key_from_name(cards: &[Card]) -> u32 {
        let BadugiHandRank::Low(played) = Badugi.evaluate_hand(cards);
        let mut key = (4 - played.len().min(4)) as u32;
        for slot in 0..4 {
            let value = played.get(slot).map_or(0, |rank| {
                if *rank == Rank::Ace {
                    1
                } else {
                    rank.to_value()
                }
            });
            key = (key << 4) | value as u32;
        }
        key
    }

    let deck: Vec<Card> = (0..52)
        .map(|i| Card::from_index(i).expect("a card"))
        .collect();
    let mut checked = 0;

    for a in 0..deck.len() {
        for b in a + 1..deck.len() {
            for c in b + 1..deck.len() {
                for d in c + 1..deck.len() {
                    let hand = [deck[a], deck[b], deck[c], deck[d]];
                    assert_eq!(
                        Badugi.score(&hand),
                        key_from_name(&hand),
                        "scored and named differently: {:?}",
                        hand
                    );
                    checked += 1;
                }
            }
        }
    }

    assert_eq!(checked, 270_725, "every four-card holding");
}
