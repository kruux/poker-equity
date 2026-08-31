use super::EquityCalculation;
use crate::{
    hand::Hand,
    variants::{
        omaha::best_seats, rankings::high_score_from_parts, Courchevel, Omaha, OmahaFive,
        OmahaSix, PokerVariant, Seats,
    },
};

/// Wires an Omaha variant into the equity engine.
///
/// Only the winners are overridden. Every seat is playing the same board, so
/// its ten three-card halves are worked out once for the table rather than
/// once a seat -- which is most of the work in an Omaha showdown, and the
/// more seats there are the more it saves.
macro_rules! omaha_equity {
    ($name:ident) => {
        impl EquityCalculation for $name {
            fn winning_seats(&self, hands: &[Hand<Self>]) -> Seats {
                best_seats(hands, self.hole_cards(), high_score_from_parts, |_| true)
            }
        }
    };
}

omaha_equity!(Omaha);
omaha_equity!(OmahaFive);
omaha_equity!(OmahaSix);
omaha_equity!(Courchevel);
