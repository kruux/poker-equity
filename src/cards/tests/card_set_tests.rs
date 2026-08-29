use crate::cards::{Card, CardSet, Rank, Suit};
use crate::error::PokerError;

#[test]
fn test_full_deck_holds_every_card_once() {
    assert_eq!(CardSet::FULL_DECK.len(), 52);
    assert_eq!(CardSet::FULL_DECK.iter().count(), 52);

    // Every card built from a suit and rank is in the deck, and every card
    // the deck yields round-trips through its index.
    for suit in Suit::all() {
        for rank in Rank::all() {
            let card = Card::new(suit, rank);
            assert!(CardSet::FULL_DECK.contains(card), "{} missing", card);
            assert_eq!(Card::from_index(card.index()), Some(card));
            assert_eq!(card.rank(), rank);
            assert_eq!(card.suit(), suit);
        }
    }
}

#[test]
fn test_indices_are_rank_times_four_plus_suit() {
    // The layout fpdb's Python side produces, so the boundary needs no
    // translation.
    assert_eq!(Card::new(Suit::Club, Rank::Two).index(), 0);
    assert_eq!(Card::new(Suit::Spade, Rank::Two).index(), 3);
    assert_eq!(Card::new(Suit::Club, Rank::Three).index(), 4);
    assert_eq!(Card::new(Suit::Heart, Rank::Ace).index(), 12 * 4 + 2);
    assert_eq!(Card::from_index(52), None);
}

#[test]
fn test_rank_and_suit_masks() -> Result<(), PokerError> {
    let aces = CardSet::of_rank(Rank::Ace);
    assert_eq!(aces.len(), 4, "four aces");
    for card in aces.iter() {
        assert_eq!(card.rank(), Rank::Ace);
    }

    let clubs = CardSet::of_suit(Suit::Club);
    assert_eq!(clubs.len(), 13, "thirteen clubs");
    for card in clubs.iter() {
        assert_eq!(card.suit(), Suit::Club);
    }

    // A rank and a suit meet in exactly one card.
    let ace_of_clubs = aces.intersection(clubs);
    assert_eq!(ace_of_clubs.len(), 1);
    assert_eq!(
        ace_of_clubs.iter().next(),
        Some(Card::from_str("Ac")?[0]),
        "the ace of clubs"
    );

    // The four suits partition the deck.
    let all_suits = Suit::all()
        .iter()
        .fold(CardSet::EMPTY, |set, &suit| set.union(CardSet::of_suit(suit)));
    assert_eq!(all_suits, CardSet::FULL_DECK);

    Ok(())
}

#[test]
fn test_set_operations() -> Result<(), PokerError> {
    let hand = CardSet::from_cards(&Card::from_str("Ah Kh")?);
    let board = CardSet::from_cards(&Card::from_str("Qh Jh Th")?);

    assert_eq!(hand.len(), 2);
    assert!(hand.is_disjoint(board));

    // What is left to deal, which in the sampler is a single AND.
    let available = CardSet::FULL_DECK.without(hand).without(board);
    assert_eq!(available.len(), 47);
    for card in hand.iter().chain(board.iter()) {
        assert!(!available.contains(card), "{} should be gone", card);
    }

    // Repeats collapse, and removing an absent card is harmless.
    let doubled = CardSet::from_cards(&Card::from_str("Ah Ah Kh")?);
    assert_eq!(doubled, hand);
    let mut set = hand;
    set.remove(Card::from_str("2c")?[0]);
    assert_eq!(set, hand);

    Ok(())
}

#[test]
fn test_nth_walks_the_set_in_order() {
    // `nth` is how a card is drawn, so it must reach every card exactly once.
    let set = CardSet::of_rank(Rank::Seven);
    let drawn: Vec<Card> = (0..set.len()).filter_map(|i| set.nth(i)).collect();
    assert_eq!(drawn.len(), 4);
    assert_eq!(drawn, set.iter().collect::<Vec<Card>>());
    assert_eq!(set.nth(4), None, "past the end of the set");

    // And over the whole deck, so that an empty-ish set is no special case.
    let deck = CardSet::FULL_DECK;
    for index in 0..52 {
        assert_eq!(deck.nth(index), Card::from_index(index as u8));
    }
    assert_eq!(CardSet::EMPTY.nth(0), None);
}

#[test]
fn test_display_reads_as_cards() -> Result<(), PokerError> {
    let set = CardSet::from_cards(&Card::from_str("Ah Kh")?);
    let shown = set.to_string();
    assert!(shown.contains("Ah") && shown.contains("Kh"), "got {}", shown);
    assert_eq!(CardSet::EMPTY.to_string(), "");
    Ok(())
}
