use crate::cards::{Card, CardSet, Rank, Suit};
use crate::error::PokerError;
use crate::sampler::{has_perfect_matching, is_feasible, maximum_matching};

fn set(text: &str) -> CardSet {
    CardSet::from_cards(&Card::from_str(text).unwrap())
}

#[test]
fn test_counting_is_not_enough_to_decide_feasibility() -> Result<(), PokerError> {
    // Five deuces is caught by counting: only four exist.
    let five_deuces = vec![CardSet::of_rank(Rank::Two); 5];
    assert!(!is_feasible(&five_deuces, CardSet::FULL_DECK));
    assert!(is_feasible(&[CardSet::of_rank(Rank::Two); 4], CardSet::FULL_DECK));

    // Two slots that both admit only the ace of hearts is not caught by
    // counting -- two slots and fifty-two cards available -- but no deal
    // satisfies it.
    let same_card = vec![set("Ah"), set("Ah")];
    assert!(
        !is_feasible(&same_card, CardSet::FULL_DECK),
        "one card cannot fill two slots"
    );

    Ok(())
}

#[test]
fn test_maximum_matching_seats_what_it_can() -> Result<(), PokerError> {
    let cards: Vec<Card> = Card::from_str("Ah Ad Kh")?;

    // Three slots, three cards, all compatible.
    let anything = vec![CardSet::FULL_DECK; 3];
    assert_eq!(maximum_matching(&anything, &cards), 3);
    assert!(has_perfect_matching(&anything, &cards));

    // Two slots want an ace and one wants a king: all three seat.
    let mixed = vec![CardSet::of_rank(Rank::Ace), CardSet::of_rank(Rank::Ace), set("Kh")];
    assert!(has_perfect_matching(&mixed, &cards));

    // Three slots all wanting a king, with one king on offer.
    let all_kings = vec![CardSet::of_rank(Rank::King); 3];
    assert_eq!(maximum_matching(&all_kings, &cards), 1);
    assert!(!has_perfect_matching(&all_kings, &cards));

    Ok(())
}

/// The matching has to displace an earlier slot to seat a later one, which is
/// what the augmenting path is for. A greedy pass that never backtracks fails
/// this.
#[test]
fn test_matching_reseats_earlier_slots() -> Result<(), PokerError> {
    let cards = Card::from_str("Ah As")?;
    // The first slot could take either card; the second can only take Ah. A
    // greedy pass that gives Ah to the first slot has to hand it back.
    let slots = vec![CardSet::of_rank(Rank::Ace), set("Ah")];
    assert!(has_perfect_matching(&slots, &cards));
    Ok(())
}

#[test]
fn test_feasibility_shrinks_with_the_deck() -> Result<(), PokerError> {
    let two_aces = vec![CardSet::of_rank(Rank::Ace); 2];
    assert!(is_feasible(&two_aces, CardSet::FULL_DECK));

    // With three aces dead there is only one left, so two slots cannot fill.
    let short = CardSet::FULL_DECK.without(set("Ah Ad Ac"));
    assert!(!is_feasible(&two_aces, short));

    // A slot no remaining card can fill fails outright.
    let no_clubs = CardSet::FULL_DECK.without(CardSet::of_suit(Suit::Club));
    assert!(!is_feasible(&[CardSet::of_suit(Suit::Club)], no_clubs));

    Ok(())
}
