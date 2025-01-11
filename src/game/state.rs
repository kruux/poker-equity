// use crate::cards::{Card, Deck};
// use crate::error::{GameError, PokerError};
// use crate::poker::Player;
// use crate::variants::PokerVariant;

// pub struct GameState<V: PokerVariant> {
//     players: Vec<Player<V>>,
//     deck: Deck,
// }

// impl<V: PokerVariant> GameState<V> {
//     pub fn new(variant: V, max_players: usize, player_names: Vec<&str>) -> Self {
//         let deck = Deck::new();

//         let mut players: Vec<Player<V>> = Vec::with_capacity(max_players);
//         for name in player_names {
//             players.push(Player::new(variant, name.to_string()));
//         }

//         Self { players, deck }
//     }

//     pub fn deal_initial_hands(&mut self) -> Result<(), PokerError> {
//         for _ in 0..5 {
//             for player in &mut self.players {
//                 if let Some(card) = self.deck.deal() {
//                     player.recieve_card(card)?;
//                 } else {
//                     return Err(GameError::NotEnoughCards.into());
//                 }
//             }
//         }

//         Ok(())
//     }

//     pub fn find_player(&self, name: &str) -> Result<&Player<V>, GameError> {
//         self.players
//             .iter()
//             .find(|p| p.name() == name)
//             .ok_or(GameError::PlayerNotFound(name.to_string()))
//     }

//     fn find_player_mut(&mut self, name: &str) -> Result<&mut Player<V>, GameError> {
//         self.players
//             .iter_mut()
//             .find(|p| p.name() == name)
//             .ok_or(GameError::PlayerNotFound(name.to_string()))
//     }

//     pub fn discard_and_draw(
//         &mut self,
//         player_name: &str,
//         cards_to_discard: Vec<Card>,
//     ) -> Result<(), PokerError> {
//         let num_cards = cards_to_discard.len();

//         self.player_discard(player_name, cards_to_discard)?;

//         self.player_draw(player_name, num_cards)?;

//         Ok(())
//     }

//     fn player_discard(
//         &mut self,
//         player_name: &str,
//         cards_to_discard: Vec<Card>,
//     ) -> Result<(), PokerError> {
//         let player = self.find_player_mut(player_name)?;
//         player.discard(&cards_to_discard)?;
//         Ok(())
//     }

//     fn player_draw(&mut self, player_name: &str, num_cards: usize) -> Result<(), PokerError> {
//         // Have to split drawing cards from deck and giving it to the player into two because of mutability
//         let mut new_cards = Vec::with_capacity(num_cards);
//         for _ in 0..num_cards {
//             if let Some(card) = self.deck.deal() {
//                 new_cards.push(card)
//             } else {
//                 return Err(GameError::NotEnoughCards.into());
//             }
//         }

//         let player = self.find_player_mut(player_name)?;
//         for card in new_cards {
//             player.recieve_card(card)?;
//         }

//         Ok(())
//     }

//     pub fn players(&self) -> &Vec<Player<V>> {
//         &self.players
//     }
// }
