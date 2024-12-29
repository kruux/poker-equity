mod draw;
mod holdem;
mod omaha;
mod stud;

pub use draw::DeuceSeven;
pub use holdem::TexasHoldem;
pub use omaha::Omaha;
pub use stud::{Razz, SevenCardStud, StudHiLo};

#[derive(Debug)]
pub enum PokerType {
    Holdem,
    Omaha,
    Stud,
    Draw,
}

pub trait PokerVariant {
    fn poker_type(&self) -> PokerType;
}
