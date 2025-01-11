// use core::fmt;

// use crate::{cards::Card, error::CardError, hand::Hand, variants::PokerVariant};

// // pub enum PlayerStatus {
// //     Active,
// //     Folded, // Sitting out would count as folded
// // }

// pub struct Player<V: PokerVariant> {
//     name: String,
//     hand: Hand<V>,
// }

// impl<V: PokerVariant> Player<V> {
//     pub fn new(variant: V, name: String) -> Self {
//         Self {
//             name,
//             hand: Hand::new(variant),
//         }
//     }

//     // pub fn fold(&mut self) {
//     //     self.status = PlayerStatus::Folded;
//     // }

//     pub fn recieve_card(&mut self, card: Card) -> Result<(), CardError> {
//         self.hand.add_card(card)?;
//         Ok(())
//     }

//     pub fn recieve_cards(&mut self, cards: Vec<Card>) -> Result<(), CardError> {
//         for card in cards {
//             self.recieve_card(card)?;
//         }
//         Ok(())
//     }

//     pub fn discard(&mut self, cards_to_discard: &Vec<Card>) -> Result<(), CardError> {
//         self.hand.discard(cards_to_discard)
//     }

//     pub fn cards(&self) -> &[Card] {
//         self.hand.cards()
//     }

//     pub fn name(&self) -> &String {
//         &self.name
//     }

//     pub fn hand(&self) -> &Hand<V> {
//         &self.hand
//     }
// }

// impl<V: PokerVariant> fmt::Display for Player<V> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}: ", self.name)?;
//         write!(f, "{}", self.hand)?;
//         Ok(())
//     }
// }
