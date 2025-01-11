use crate::{
    cards::{Card, Rank},
    variants::{HighHandRank, PokerType, PokerVariant},
};

#[derive(Clone, Copy, Debug)]
pub struct SevenCardStud;

impl PokerVariant for SevenCardStud {
    type HandRank = HighHandRank;

    fn poker_type(&self) -> PokerType {
        PokerType::Stud
    }

    fn max_cards(&self) -> usize {
        7
    }

    fn to_string(&self) -> String {
        "Stud".to_string()
    }

    fn sorted_ranks(&self, cards: &[Card]) -> Vec<Rank> {
        let mut ranks: Vec<Rank> = cards.iter().map(|card| card.rank()).collect();
        ranks.sort_by(|a, b| a.cmp(b)); // Higher is better
        ranks.reverse(); // Descending order
        ranks
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        let rank_counts = &self.rank_counts(cards);

        if let Some((rank, _)) = self.is_straight_flush(cards) {
            return HighHandRank::StraightFlush(rank);
        }

        if let Some(rank) = self.is_four_of_kind(rank_counts) {
            return HighHandRank::FourOfAKind(rank);
        }

        if let Some((trips, pair)) = self.is_full_house(rank_counts) {
            return HighHandRank::FullHouse(trips, pair);
        }

        if let Some(ranks) = self.is_flush(cards) {
            return HighHandRank::Flush(ranks);
        }

        if let Some(rank) = self.is_straight(cards) {
            return HighHandRank::Straight(rank);
        }

        if let Some(rank) = self.is_three_of_kind(rank_counts) {
            return HighHandRank::ThreeOfAKind(rank);
        }

        if let Some((high_pair, low_pair, kicker)) = self.is_two_pair(rank_counts) {
            return HighHandRank::TwoPair(high_pair, low_pair, kicker);
        }

        if let Some((pair_rank, kickers)) = self.is_pair(rank_counts) {
            return HighHandRank::Pair(pair_rank, kickers);
        }

        let sorted = self.sorted_ranks(cards);
        return HighHandRank::HighCard(self.first_n_ranks(&sorted, 5));
    }
}
