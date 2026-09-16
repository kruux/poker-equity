//! Grouping the deck by what the slots can tell apart.
//!
//! A slot list like `[A, 2, 3, *, *, *, *]` -- a razz hand written the way a
//! player writes it -- says nothing about suits, so the four aces are
//! interchangeable and so are the deuces, the threes, and the forty cards
//! that no named slot wants. That is four groups, not fifty-two cards, and
//! once the deck is seen that way a hand is fully described by *how many* it
//! takes from each group. There are thirty-two such counts, they can be
//! listed in microseconds, and every one carries the number of hands that
//! have that shape.
//!
//! That is enough to do both jobs exactly. Listing every valid hand becomes a
//! walk over the shapes rather than a search through `C(52,7)` candidates,
//! and where there are too many hands to list, picking a shape in proportion
//! to its weight and then drawing from each group is uniform over the hands
//! by construction -- with nothing drawn and thrown away.
//!
//! The general form of this is counting a permanent, which is #P-hard, so
//! there is no version of it that works for every conceivable slot list. It
//! works here because the notation only ever produces named cards, whole
//! ranks and whole suits, which cut the deck into a handful of groups.

use rand::Rng;

use crate::cards::{Card, CardSet};

use super::matching::has_perfect_matching;

/// How many shapes are worth listing before giving up on this approach.
///
/// A hand with a few named ranks makes tens of them; the bound is here for a
/// slot list built through the mask API that cuts the deck much finer than
/// the notation can, and it costs a fallback rather than an error.
const MOST_SHAPES: usize = 20_000;

/// One way to spread a hand across the groups: how many cards it takes from
/// each, and how many distinct hands do that.
#[derive(Debug, Clone)]
struct Shape {
    counts: Vec<u8>,
    weight: u128,
}

/// The groups a slot list cuts the deck into, and every shape it admits.
#[derive(Debug, Clone)]
pub(crate) struct ShapePlan {
    /// The groups, in a fixed order. Every card in one is interchangeable
    /// with every other as far as these slots are concerned.
    atoms: Vec<CardSet>,
    shapes: Vec<Shape>,
    /// Running totals of the shape weights, so a shape can be picked in
    /// proportion to how many hands it stands for with one binary search.
    cumulative: Vec<u128>,
    total: u128,
    size: usize,
}

/// Cuts `pool` into groups of cards that no slot can tell apart.
///
/// Two cards belong together when exactly the same slots accept them, so the
/// grouping is by that signature. The notation makes few of them: named
/// ranks and suits are broad, and everything no slot names falls into one
/// group at the end.
fn atoms_of(slots: &[CardSet], pool: CardSet) -> Vec<CardSet> {
    let mut signatures: Vec<(u64, CardSet)> = Vec::new();

    for card in pool.iter() {
        let mut signature: u64 = 0;
        for (index, slot) in slots.iter().enumerate() {
            if slot.contains(card) {
                signature |= 1 << index;
            }
        }
        // A card no slot accepts cannot appear in any valid hand.
        if signature == 0 {
            continue;
        }
        match signatures.iter_mut().find(|(seen, _)| *seen == signature) {
            Some((_, atom)) => atom.insert(card),
            None => signatures.push((signature, CardSet::from_cards(&[card]))),
        }
    }

    signatures.into_iter().map(|(_, atom)| atom).collect()
}

/// `n` choose `k` as a `u128`, saturating rather than wrapping.
///
/// The counts here are hand totals -- nine million for a razz hand written
/// with three ranks -- so they outgrow a `u32` but never come near the top of
/// a `u128`.
fn binomial(n: u32, k: u32) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut result: u128 = 1;
    for step in 0..k {
        result = result.saturating_mul((n - step) as u128) / (step + 1) as u128;
    }
    result
}

impl ShapePlan {
    /// Works out the groups and every shape they admit, or `None` when the
    /// slots cut the deck too finely for this to be worth doing.
    ///
    /// A plan with a [`total`](Self::total) of zero is a real answer, not a
    /// failure: it says these slots admit no hand at all -- seven slots
    /// wanting an ace apiece, against four aces. `None` means something
    /// different, that the shapes could not be listed and the caller should
    /// fall back to searching.
    pub(crate) fn build(slots: &[CardSet], pool: CardSet) -> Option<Self> {
        let size = slots.len();
        if size == 0 {
            return None;
        }

        let atoms = atoms_of(slots, pool);

        let mut shapes = Vec::new();
        let mut counts = vec![0u8; atoms.len()];
        walk_shapes(slots, &atoms, 0, size, &mut counts, &mut shapes)?;

        let mut cumulative = Vec::with_capacity(shapes.len());
        let mut total: u128 = 0;
        for shape in &shapes {
            total = total.saturating_add(shape.weight);
            cumulative.push(total);
        }

        Some(Self {
            atoms,
            shapes,
            cumulative,
            total,
            size,
        })
    }

    /// How many groups the slots cut the deck into, and how many shapes
    /// those groups admit. For measuring where a plan's cost goes.
    #[cfg(test)]
    pub(crate) fn size_hint(&self) -> (usize, usize) {
        (self.atoms.len(), self.shapes.len())
    }

    /// How many distinct hands the slots admit, counted without listing them.
    pub(crate) fn total(&self) -> u128 {
        self.total
    }

    /// Every hand the slots admit, listed.
    ///
    /// A walk over the shapes rather than a search: each shape says how many
    /// cards to take from each group, so the hands are built rather than
    /// found and tested. Nothing is generated that is then discarded.
    pub(super) fn enumerate(&self) -> Vec<Vec<Card>> {
        let mut out = Vec::with_capacity(self.total.min(1 << 20) as usize);
        let mut hand = Vec::with_capacity(self.size);
        for shape in &self.shapes {
            self.build_hands(shape, 0, &mut hand, &mut out);
        }
        out
    }

    /// Lists the hands of one shape, group by group.
    fn build_hands(
        &self,
        shape: &Shape,
        atom: usize,
        hand: &mut Vec<Card>,
        out: &mut Vec<Vec<Card>>,
    ) {
        if atom == self.atoms.len() {
            out.push(hand.clone());
            return;
        }
        let take = shape.counts[atom] as usize;
        if take == 0 {
            self.build_hands(shape, atom + 1, hand, out);
            return;
        }
        let cards: Vec<Card> = self.atoms[atom].iter().collect();
        choose(&cards, take, 0, hand, &mut |hand| {
            self.build_hands(shape, atom + 1, hand, out)
        });
    }

    /// Draws one hand, uniformly over every hand the slots admit.
    ///
    /// A shape is picked in proportion to how many hands have it, and then
    /// that many cards are taken from each group. Every hand is reachable
    /// exactly one way, so the result is uniform with nothing rejected.
    pub(crate) fn draw(&self, rng: &mut impl Rng, out: &mut Vec<Card>) {
        self.draw_from(&self.atoms, &self.cumulative, self.total, rng, out);
    }

    /// Re-weighs the shapes against a deck that has been dealt from, and
    /// reports how many hands are left.
    ///
    /// Which groups exist, and which shapes can fill the slots, are settled
    /// by the *slots* alone: two cards sit in the same group when the same
    /// slots accept them, and dealing a card out moves no other card between
    /// groups. So a plan built once stays structurally right all the way down
    /// the deal, and only the group sizes change. That is the whole reason
    /// this is cheap: no walk, no matching, just a binomial per group per
    /// shape. A shape that has outrun its group weighs nothing and drops out
    /// on its own, since `C(fewer than we need, count)` is zero.
    ///
    /// `groups` and `cumulative` are scratch space owned by the caller, so a
    /// sampling loop allocates nothing per deal.
    pub(crate) fn weigh(
        &self,
        pool: CardSet,
        groups: &mut Vec<CardSet>,
        cumulative: &mut Vec<u128>,
    ) -> u128 {
        groups.clear();
        groups.extend(self.atoms.iter().map(|atom| atom.intersection(pool)));

        cumulative.clear();
        let mut total: u128 = 0;
        for shape in &self.shapes {
            let weight =
                groups
                    .iter()
                    .zip(shape.counts.iter())
                    .fold(1u128, |all, (group, &count)| {
                        if all == 0 {
                            0
                        } else {
                            all.saturating_mul(binomial(group.len(), count as u32))
                        }
                    });
            total = total.saturating_add(weight);
            cumulative.push(total);
        }
        total
    }

    /// Draws one hand from groups and weights worked out by [`weigh`].
    ///
    /// [`weigh`]: Self::weigh
    pub(crate) fn draw_weighed(
        &self,
        groups: &[CardSet],
        cumulative: &[u128],
        total: u128,
        rng: &mut impl Rng,
        out: &mut Vec<Card>,
    ) {
        self.draw_from(groups, cumulative, total, rng, out);
    }

    fn draw_from(
        &self,
        groups: &[CardSet],
        cumulative: &[u128],
        total: u128,
        rng: &mut impl Rng,
        out: &mut Vec<Card>,
    ) {
        debug_assert!(total > 0, "a plan with no hands should never be drawn from");
        let wanted = rng.gen_range(0..total);
        let index = cumulative.partition_point(|reached| *reached <= wanted);
        let shape = &self.shapes[index];

        for (group, &count) in groups.iter().zip(shape.counts.iter()) {
            let mut left = *group;
            for _ in 0..count {
                let picked = rng.gen_range(0..left.len());
                if let Some(card) = left.nth(picked) {
                    left.remove(card);
                    out.push(card);
                }
            }
        }
    }
}

/// Walks every way of spreading `left` cards across the groups from `atom` on,
/// keeping those the slots can actually be filled from.
///
/// Returns `None` once there are more shapes than are worth holding, which is
/// the caller's signal to fall back.
fn walk_shapes(
    slots: &[CardSet],
    atoms: &[CardSet],
    atom: usize,
    left: usize,
    counts: &mut Vec<u8>,
    shapes: &mut Vec<Shape>,
) -> Option<()> {
    if shapes.len() > MOST_SHAPES {
        return None;
    }
    if atom == atoms.len() {
        if left > 0 {
            return Some(());
        }
        // Cards within a group are interchangeable, so one representative
        // set settles whether every shape like it can fill the slots.
        let mut sample: Vec<Card> = Vec::with_capacity(slots.len());
        for (group, &count) in atoms.iter().zip(counts.iter()) {
            sample.extend(group.iter().take(count as usize));
        }
        if has_perfect_matching(slots, &sample) {
            let weight = atoms
                .iter()
                .zip(counts.iter())
                .fold(1u128, |all, (group, &count)| {
                    all.saturating_mul(binomial(group.len(), count as u32))
                });
            if weight > 0 {
                shapes.push(Shape {
                    counts: counts.clone(),
                    weight,
                });
            }
        }
        return Some(());
    }

    // Nothing below can supply more than the groups left hold, so a branch
    // that still needs more than that has nowhere to go. Without this the
    // walk descends into every shortfall and only notices at the leaf.
    let reachable: usize = atoms[atom..].iter().map(|group| group.len() as usize).sum();
    if reachable < left {
        return Some(());
    }

    let most = (atoms[atom].len() as usize).min(left);
    for take in 0..=most {
        counts[atom] = take as u8;
        walk_shapes(slots, atoms, atom + 1, left - take, counts, shapes)?;
    }
    counts[atom] = 0;
    Some(())
}

/// Calls `found` with every way of taking `take` cards from `cards`.
///
/// The chosen cards are pushed onto `hand` and popped again, so one buffer
/// carries the whole walk and nothing is allocated per combination.
fn choose(
    cards: &[Card],
    take: usize,
    from: usize,
    hand: &mut Vec<Card>,
    found: &mut impl FnMut(&mut Vec<Card>),
) {
    if take == 0 {
        found(hand);
        return;
    }
    // Stop once too few cards are left to finish the choice.
    for index in from..=cards.len() - take {
        hand.push(cards[index]);
        choose(cards, take - 1, index + 1, hand, found);
        hand.pop();
    }
}
