use std::cmp::Ordering;

use crate::{
    cards::{Card, Rank, Suit},
    error::PokerError,
    hand::Hand,
    variants::{Omaha, OmahaFast, OmahaHandRank},
};

#[test]
fn test_empty_hand_comparison() -> Result<(), PokerError> {
    // Directly test incomplete Omaha hand rankings.
    let empty_rank = OmahaHandRank::Incomplete(0);
    let one_card_rank = OmahaHandRank::Incomplete(1);

    // Incomplete hands with the same card count should be equal.
    assert_eq!(empty_rank.partial_cmp(&empty_rank), Some(Ordering::Equal));
    // An incomplete hand with more cards beats one with fewer cards.
    assert_eq!(empty_rank.partial_cmp(&one_card_rank), Some(Ordering::Less));
    assert_eq!(
        one_card_rank.partial_cmp(&empty_rank),
        Some(Ordering::Greater)
    );

    // Now test by evaluating hands via Hand::from_str.
    // These functions should yield the appropriate Incomplete variant if there aren't enough cards.
    let empty_hand1 = Hand::from_str(Omaha, "")?;
    let empty_hand2 = Hand::from_str(Omaha, "")?;
    let one_card_hand = Hand::from_str(Omaha, "2h")?;

    // Evaluate and check that an empty hand yields Incomplete(0)
    match empty_hand1.evaluate() {
        OmahaHandRank::Incomplete(n) => {
            assert_eq!(n, 0, "Empty hand should evaluate to Incomplete(0)");
        }
        r => panic!("Expected Incomplete(0) for an empty hand, got {:?}", r),
    }

    // Evaluate and check that a one-card hand yields Incomplete(1)
    match one_card_hand.evaluate() {
        OmahaHandRank::Incomplete(n) => {
            assert_eq!(n, 1, "A one-card hand should evaluate to Incomplete(1)");
        }
        r => panic!("Expected Incomplete(1) for a one-card hand, got {:?}", r),
    }

    // Comparison of Hand instances.
    // Two empty evaluated hands should be equal.
    assert_eq!(empty_hand1, empty_hand2);
    assert!(!(empty_hand1 > empty_hand2));
    assert!(!(empty_hand1 < empty_hand2));

    // A hand with a card should beat an empty hand.
    assert!(one_card_hand > empty_hand1);
    assert!(empty_hand1 < one_card_hand);

    Ok(())
}

/// Scores an Omaha holding written as hole cards and board.
fn evaluate(hole: &str, board: &str) -> Result<OmahaHandRank, PokerError> {
    Ok(Hand::from_str(Omaha, &format!("{} {}", hole, board))?.evaluate())
}

/// A flush needs two suited cards *from the hand*. One hole heart alongside
/// four board hearts is not a flush, however tempting it looks.
#[test]
fn test_flush_needs_two_hole_cards() -> Result<(), PokerError> {
    assert_eq!(
        evaluate("Ah 2c 3d 4s", "Kh Qh Jh 9h 2s")?,
        OmahaHandRank::Pair(Rank::Two, [Rank::Ace, Rank::King, Rank::Queen]),
        "one heart in hand and four on the board is only a pair of deuces"
    );

    assert_eq!(
        evaluate("Ah 5h 3d 4s", "Kh Qh Jh 9c 2s")?,
        OmahaHandRank::Flush([Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Five]),
        "two hearts in hand and three on the board is a flush"
    );

    Ok(())
}

/// Exactly three cards come from the board, so a hand that is complete on the
/// board cannot be played as it stands.
#[test]
fn test_board_hands_cannot_be_played_whole() -> Result<(), PokerError> {
    assert_eq!(
        evaluate("Ac Ad Kc Kd", "Td 9c 8s 7h 6d")?,
        OmahaHandRank::Pair(Rank::Ace, [Rank::Ten, Rank::Nine, Rank::Eight]),
        "the board is a made straight, but two hole cards must come with it"
    );

    assert_eq!(
        evaluate("2c 3d 5h 6s", "9h 9c 9d 9s Kh")?,
        OmahaHandRank::ThreeOfAKind(Rank::Nine, [Rank::Six, Rank::Five]),
        "only three of the four board nines can be used"
    );

    Ok(())
}

/// Straights obey the same split: two from the hand, three from the board.
#[test]
fn test_straights_use_two_from_hand_and_three_from_board() -> Result<(), PokerError> {
    assert_eq!(
        evaluate("Jh Tc 2d 3s", "9h 8c 7d 2h 4c")?,
        OmahaHandRank::Straight(Rank::Jack),
        "jack and ten from the hand with nine, eight and seven from the board"
    );

    // The ace plays low here just as it does anywhere else.
    assert_eq!(
        evaluate("Ah 2c Kd Qs", "3h 4d 5c Jh 9s")?,
        OmahaHandRank::Straight(Rank::Five),
        "ace and deuce from the hand with three, four and five from the board"
    );

    Ok(())
}

/// A full house takes its pair from the hand and its trips from the board, or
/// the other way about -- but never four cards from one side.
#[test]
fn test_full_house_across_the_split() -> Result<(), PokerError> {
    assert_eq!(
        evaluate("Ah Ac 7d 8s", "Kh Kd Ks 2c 3h")?,
        OmahaHandRank::FullHouse(Rank::King, Rank::Ace),
        "board trips with a pocket pair"
    );

    assert_eq!(
        evaluate("Ah Ac 7d 8s", "Ad Kd Ks 2c 3h")?,
        OmahaHandRank::FullHouse(Rank::Ace, Rank::King),
        "pocket pair filling in against the board's ace and kings"
    );

    Ok(())
}

/// The rule can only ever cost a player, never help: an Omaha hand is no
/// better than the same seven cards played without the restriction.
#[test]
fn test_the_rule_never_improves_a_hand() -> Result<(), PokerError> {
    // Held as hold'em these seven cards are a heart flush; in Omaha they are
    // not, because only one heart is in the hand.
    let restricted = evaluate("Ah 2c 3d 4s", "Kh Qh Jh 9h 2s")?;
    let unrestricted = crate::variants::HighHandRank::evaluate(&crate::cards::Card::from_str(
        "Ah 2c 3d 4s Kh Qh Jh 9h 2s",
    )?);
    assert!(
        restricted < unrestricted,
        "Omaha's split cost this hand its flush: {} against {}",
        restricted,
        unrestricted
    );

    Ok(())
}

/// `OmahaFast` reads the generated table where `Omaha` walks the hand, so the
/// two must order every pair of holdings the same way. Deals come from a
/// fixed-seed generator, so a failure reproduces exactly.
#[test]
fn test_fast_and_slow_omaha_agree() -> Result<(), PokerError> {
    let mut deck: Vec<Card> = Vec::with_capacity(52);
    for suit in Suit::all() {
        for rank in Rank::all() {
            deck.push(Card::new(suit, rank));
        }
    }

    let mut state: u64 = 0x2545F4914F6CDD1D;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    for deal in 0..20_000 {
        // Thirteen distinct cards: four each to two players, then a board.
        let mut chosen = [0usize; 13];
        let mut taken = 0;
        while taken < 13 {
            let pick = (next() % 52) as usize;
            if !chosen[..taken].contains(&pick) {
                chosen[taken] = pick;
                taken += 1;
            }
        }
        let card = |i: usize| deck[chosen[i]];
        let board: Vec<Card> = (8..13).map(card).collect();

        let mut hands = Vec::new();
        for seat in 0..2 {
            let mut cards: Vec<Card> = (seat * 4..seat * 4 + 4).map(card).collect();
            cards.extend(board.iter().copied());
            hands.push(cards);
        }

        let slow = Hand::new_with_cards(Omaha, hands[0].clone())?
            .evaluate()
            .partial_cmp(&Hand::new_with_cards(Omaha, hands[1].clone())?.evaluate());
        let fast = Hand::new_with_cards(OmahaFast, hands[0].clone())?
            .evaluate()
            .partial_cmp(&Hand::new_with_cards(OmahaFast, hands[1].clone())?.evaluate());

        assert_eq!(
            slow, fast,
            "deal {} ordered differently by the two evaluators: {:?} against {:?}",
            deal, hands[0], hands[1]
        );
    }

    Ok(())
}
