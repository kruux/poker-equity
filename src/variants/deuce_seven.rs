use crate::{
    cards::{Card, Rank},
    variants::DeuceSevenRank,
    variants::{PokerType, PokerVariant},
};

#[derive(Clone, Copy, Debug)]
pub struct DeuceSeven;

impl DeuceSeven {}

impl PokerVariant for DeuceSeven {
    type HandRank = DeuceSevenRank;

    fn poker_type(&self) -> PokerType {
        PokerType::Draw
    }

    fn max_cards(&self) -> usize {
        5
    }

    fn to_string(&self) -> String {
        "2-7 single draw".to_string()
    }

    fn sorted_ranks(&self, cards: &[Card]) -> Vec<Rank> {
        let mut ranks: Vec<Rank> = cards.iter().map(|card| card.rank()).collect();
        ranks.sort_by(|a, b| b.cmp(a)); // Sorts descending
        ranks
    }

    fn evaluate_hand(&self, cards: &[Card]) -> DeuceSevenRank {
        let rank_counts = &self.rank_counts(cards);

        if let Some((rank, _)) = self.is_straight_flush(cards) {
            // Make sure it's not A2345
            if rank != Rank::Five {
                return DeuceSevenRank::StraightFlush(rank);
            }
        }

        if let Some(rank) = self.is_four_of_kind(&rank_counts) {
            return DeuceSevenRank::FourOfAKind(rank);
        }

        if let Some((trips, pair)) = self.is_full_house(&rank_counts) {
            return DeuceSevenRank::FullHouse(trips, pair);
        }

        if let Some(ranks) = self.is_flush(cards) {
            return DeuceSevenRank::Flush(ranks);
        }

        if let Some(rank) = self.is_straight(cards) {
            // Make sure it's not A2345
            if rank != Rank::Five {
                return DeuceSevenRank::Straight(rank);
            }
        }

        if let Some(rank) = self.is_three_of_kind(&rank_counts) {
            return DeuceSevenRank::ThreeOfAKind(rank);
        }

        if let Some((high_pair, low_pair, kicker)) = self.is_two_pair(&rank_counts) {
            return DeuceSevenRank::TwoPair(high_pair, low_pair, kicker);
        }

        if let Some((pair_rank, kickers)) = self.is_pair(&rank_counts) {
            return DeuceSevenRank::Pair(pair_rank, kickers);
        }

        return DeuceSevenRank::HighCard(self.sorted_ranks(cards));
    }
}
