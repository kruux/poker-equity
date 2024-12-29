use crate::{cards::Card, game::GameState};

#[test]
fn game() {
    let mut players: Vec<&str> = Vec::new();
    players.push("Hero");
    players.push("Villain");
    let mut game = GameState::new(2, players);
    game.deal_initial_hands().unwrap();

    for player in game.players() {
        println!("{}", player);
    }

    assert_eq!(1, 1);
}

/// This test will first draw 5 initial cards and after that discard the last card
/// and draw a new one. It will test that the first 4 cards are the same and make sure
/// the last card is NOT the same
#[test]
fn discard() {
    //
    let playername = "Hero";
    let mut game = GameState::new(1, vec![playername]);

    game.deal_initial_hands().unwrap();
    let hand_before_discard: Vec<Card>;
    let last_card: Card;

    {
        let player = game.find_player(playername).unwrap();
        println!("{}", player);
        hand_before_discard = player.cards().to_vec();
        last_card = hand_before_discard[4];
    }

    game.discard_and_draw(playername, vec![last_card]).unwrap();
    let hand_after_discard: Vec<Card>;
    {
        let player = game.find_player(playername).unwrap();
        hand_after_discard = player.cards().to_vec();
        println!("{}", player);
    }

    // Make sure the first 4 cards matches
    for i in 0..4 {
        let before = hand_before_discard[i];
        let after = hand_after_discard[i];

        assert!(before.matches(&after));
    }
    // Make sure the last card differs after the draw
    let before = hand_before_discard[4];
    let after = hand_after_discard[4];
    assert!(!before.matches(&after));
}
