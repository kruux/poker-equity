//! 5-card draw, end to end.
//!
//! The best high hand wins, ranked as hold'em ranks its best five, with one
//! draw -- which is the whole game, so nothing here is a simplification. As
//! in 2-7, a five-card field is a hand standing pat, a shorter one is a hand
//! drawing, and the discards are named as dead.
//!
//! Every figure was worked out apart from this engine: each spot was walked
//! outright in Python, every deal scored by pokerkit's `StandardHighHand`.
//! The one-card draws can be checked by hand as well -- nine flush cards of
//! forty-two unseen is 21.428571%, and eight straight cards is 19.047619%.

use super::support::assert_walked;
use crate::{
    error::{GameError, PokerError},
    odds::EquityRequest,
    variants::FiveCardDraw,
};

/// A 5-card draw spot: the hands as they stand, and the cards thrown away.
fn spot(hands: &[&str], dead: &str) -> Result<EquityRequest<FiveCardDraw>, PokerError> {
    EquityRequest::from_text(FiveCardDraw, hands, "", dead)
}

/// Two hands already made, where nothing is left to deal.
#[test]
fn test_pat_hands_are_settled_before_they_start() -> Result<(), PokerError> {
    // A flush beats two pair, and it is the only deal there is.
    assert_walked(
        &spot(&["AsAcKdKh2c", "7h6h5h3h2h"], "")?,
        &[0.0, 100.0],
        "two pair against a flush",
    )?;

    // The same nine-high straight in different suits. Suits never break a
    // tie in a high hand, so the pot divides.
    assert_walked(
        &spot(&["9h8c7d6s5h", "9c8d7h6c5s"], "")?,
        &[50.0, 50.0],
        "the same straight twice",
    )?;

    Ok(())
}

/// One seat drawing to a hand that beats a pat one.
#[test]
fn test_a_draw_against_a_pat_hand() -> Result<(), PokerError> {
    // Four hearts, having thrown away the two of clubs, against pat jacks and
    // tens. Pairing the ace or king is not enough; only the nine hearts left
    // among forty-two unseen cards win.
    assert_walked(
        &spot(&["AhKhQh7h", "JsJdTsTc3d"], "2c")?,
        &[21.428571, 78.571429],
        "a flush draw against two pair",
    )?;

    // An open-ended straight draw against pat aces: four tens and four fives,
    // eight of forty-two.
    assert_walked(
        &spot(&["9h8c7d6s", "AsAcQh4d2s"], "Kd")?,
        &[19.047619, 80.952381],
        "an open-ender against aces",
    )?;

    // Queens drawing three against pat kings. Any improvement at all -- two
    // pair, trips or better -- is enough, and it comes about three times in
    // ten.
    assert_walked(
        &spot(&["QsQd", "KhKc8s5d2c"], "7h 4c 3s")?,
        &[29.703833, 70.296167],
        "queens drawing three against kings",
    )?;

    Ok(())
}

/// Both seats drawing, and three seats at once.
#[test]
fn test_several_seats_drawing() -> Result<(), PokerError> {
    // Two pair drawing one against trips drawing two. Only a full house
    // wins for the two pair, and the trips can still fill up past it.
    assert_walked(
        &spot(&["JhJs4d4c", "9s9h9d"], "Ac Kd 2h")?,
        &[8.757259, 91.242741],
        "two pair drawing against trips drawing",
    )?;

    // A flush draw, a straight draw and pat two pair. The flush draw wins
    // whenever it gets there, so the straight draw is worth less than its
    // outs alone suggest.
    assert_walked(
        &spot(&["AhKhQh7h", "9s8c7d6s", "JsJdTsTc3d"], "2c Kd")?,
        &[24.324324, 12.312312, 63.363363],
        "a flush draw, a straight draw and two pair",
    )?;

    Ok(())
}

/// A card cannot be thrown away and held at the same time.
#[test]
fn test_a_card_cannot_be_dead_and_in_a_hand() -> Result<(), PokerError> {
    let refused = spot(&["AhKhQh7h", "JsJdTsTc3d"], "Ah").unwrap_err();
    assert!(
        matches!(&refused, PokerError::Game(GameError::DuplicateCard(card))
                 if card.to_string() == "Ah"),
        "the ace of hearts is both held and discarded, got {:?}",
        refused
    );
    Ok(())
}
