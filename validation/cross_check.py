"""Asks an outside evaluator whether this library ranks hands correctly.

The library's own sweeps check its lookup tables against the evaluator that
names its hands. Both were written in this repository, from one reading of the
rules, so they catch a mistake in either -- but a rule misunderstood in *both*
survives every sweep. Only an independent implementation settles that.

This compares against pokerkit, which implements each of these rankings
separately and is maintained as a research library rather than as part of a
product. Every five-card hand is checked, so the result is a proof over that
space rather than a sample.

    python3 -m venv .venv && .venv/bin/pip install pokerkit
    cargo run --release --bin export_scores -- /tmp/scores
    .venv/bin/python validation/cross_check.py /tmp/scores

Nothing here is part of the library or its build. It is meant to be run once
before shipping, and again whenever a ranking changes.
"""

import itertools
import struct
import sys
from pathlib import Path

from pokerkit.hands import (
    BadugiHand,
    OmahaHoldemHand,
    RegularLowHand,
    ShortDeckHoldemHand,
    StandardHighHand,
    StandardLowHand,
)

# Cards are numbered rank * 4 + suit, so this list is in card-index order and
# both sides enumerate hands the same way.
DECK = [rank + suit for rank in "23456789TJQKA" for suit in "cdhs"]
SHORT_DECK = [card for card in DECK if card[0] not in "2345"]

# Which of pokerkit's rankings answers for each of ours, and why. Badugi is
# built through `from_game`, because a four-card holding is not itself a
# badugi -- pokerkit has to pick the subset that plays, as we do.
KERNELS = [
    ("high", DECK, 5, StandardHighHand, False,
     "the five-card high hand"),
    ("deuce_seven", DECK, 5, StandardLowHand, False,
     "the high hand upside down, ace high, straights not wrapping"),
    ("low_a5", DECK, 5, RegularLowHand, False,
     "ace-to-five, straights and flushes ignored"),
    ("short_deck", SHORT_DECK, 5, ShortDeckHoldemHand, False,
     "thirty-six cards, a flush over a full house"),
    ("badugi", DECK, 4, BadugiHand, True,
     "the largest rank- and suit-distinct subset"),
]


def betterness(evaluator, from_game):
    """A number for each hand where greater is always the better hand.

    pokerkit's `entry.index` counts upwards from the worst hand for a high
    ranking and from the best hand for a low one, while its comparison
    operators normalise the two. Rather than assume which way round a given
    ranking runs, this asks the operator.
    """
    build = evaluator.from_game if from_game else evaluator

    def rank(cards):
        return build("".join(cards))

    def index(cards):
        return rank(cards).entry.index

    return build, index


def read_scores(path):
    """Our score for each hand, in the order the hands enumerate."""
    data = path.read_bytes()
    return struct.unpack(f"<{len(data) // 2}H", data)


def check(name, deck, size, evaluator, from_game, description, scores):
    """Reports where we and pokerkit disagree about the order of two hands."""
    hands = list(itertools.combinations(deck, size))
    if len(hands) != len(scores):
        return [f"{name}: exported {len(scores)} scores for {len(hands)} hands"]

    build, index = betterness(evaluator, from_game)
    theirs = [index(hand) for hand in hands]

    # Work out which way this ranking's indices run by asking the comparison
    # operator about two hands that differ, rather than assuming.
    direction = 0
    for a in range(len(hands)):
        if theirs[a] != theirs[0]:
            better = build("".join(hands[a])) > build("".join(hands[0]))
            higher = theirs[a] > theirs[0]
            direction = 1 if better == higher else -1
            break
    if direction == 0:
        return [f"{name}: every hand ranks the same, which cannot be right"]
    theirs = [direction * value for value in theirs]

    # We score lowest-is-best, so sorting by our score walks their betterness
    # downwards, and hands we call equal they must call equal too.
    order = sorted(range(len(hands)), key=lambda i: scores[i])

    problems = []
    for position in range(len(order) - 1):
        a, b = order[position], order[position + 1]
        if scores[a] == scores[b]:
            if theirs[a] != theirs[b]:
                problems.append(
                    f"we rank {' '.join(hands[a])} and {' '.join(hands[b])} equal, "
                    f"pokerkit does not ({theirs[a]} against {theirs[b]})"
                )
        elif theirs[a] <= theirs[b]:
            problems.append(
                f"we rank {' '.join(hands[a])} above {' '.join(hands[b])}, "
                f"pokerkit ranks it below ({theirs[a]} against {theirs[b]})"
            )
        if len(problems) >= 8:
            problems.append("...")
            break

    print(f"  {len(hands):>9,} hands, {len(set(scores)):>5,} distinct values "
          f"against pokerkit's {len(set(theirs)):>5,} "
          f"(their index runs {'with' if direction > 0 else 'against'} ours)")
    return problems


def main():
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    directory = Path(sys.argv[1])

    failed = False
    for name, deck, size, evaluator, from_game, description in KERNELS:
        path = directory / f"{name}.bin"
        if not path.exists():
            print(f"{name}: {path} is missing; run export_scores first")
            failed = True
            continue

        print(f"{name} -- {description}")
        problems = check(
            name, deck, size, evaluator, from_game, description, read_scores(path)
        )
        if problems:
            failed = True
            print(f"  DISAGREES with pokerkit:")
            for problem in problems:
                print(f"    {problem}")
        else:
            print("  agrees with pokerkit on every hand")
        print()

    for name, evaluator, has_board, description in SAMPLED:
        path = directory / f"{name}.tsv"
        if not path.exists():
            print(f"{name}: {path} is missing; run export_scores first")
            failed = True
            continue

        print(f"{name} -- {description}")
        problems = check_sampled(name, evaluator, has_board, description, path)
        if problems:
            failed = True
            print("  DISAGREES with pokerkit:")
            for problem in problems:
                print(f"    {problem}")
        else:
            print("  agrees with pokerkit on every hand")
        print()

    if failed:
        print("at least one ranking disagrees")
        return 1
    print("every ranking agrees with pokerkit")
    return 0

# Hands too numerous to enumerate, sampled from a fixed seed on the Rust side
# and written out with their scores. These cover what the five-card sweeps
# cannot: holdings that hold more than they play, and Omaha's rule that
# exactly two hole cards play with exactly three of the board.
SAMPLED = [
    ("seven_high", StandardHighHand, 0,
     "seven cards, the best five of them"),
    ("seven_low", RegularLowHand, 0,
     "seven cards, the best five-card ace-to-five low"),
    ("seven_short_deck", ShortDeckHoldemHand, 0,
     "seven short-deck cards"),
    ("omaha", OmahaHoldemHand, 1,
     "exactly two hole cards with exactly three of the board"),
]


def split_cards(text):
    """Splits 'AhKh' into ['Ah', 'Kh']."""
    return [text[i:i + 2] for i in range(0, len(text), 2)]


def check_sampled(name, evaluator, has_board, description, path):
    """Compares sampled hands, which arrive with their cards written out."""
    hands, ours = [], []
    for line in path.read_text().splitlines():
        parts = line.split("\t")
        ours.append(int(parts[-1]))
        hands.append(parts[:-1])

    def build(hand):
        if has_board:
            return evaluator.from_game(hand[0], hand[1])
        return evaluator.from_game(hand[0])

    theirs = [build(hand).entry.index for hand in hands]

    direction = 0
    for a in range(len(hands)):
        if theirs[a] != theirs[0]:
            better = build(hands[a]) > build(hands[0])
            direction = 1 if better == (theirs[a] > theirs[0]) else -1
            break
    theirs = [direction * value for value in theirs]

    order = sorted(range(len(hands)), key=lambda i: ours[i])
    problems = []
    for position in range(len(order) - 1):
        a, b = order[position], order[position + 1]
        shown_a, shown_b = " ".join(hands[a]), " ".join(hands[b])
        if ours[a] == ours[b]:
            if theirs[a] != theirs[b]:
                problems.append(f"we rank {shown_a} and {shown_b} equal, pokerkit does not")
        elif theirs[a] <= theirs[b]:
            problems.append(f"we rank {shown_a} above {shown_b}, pokerkit ranks it below")
        if len(problems) >= 8:
            problems.append("...")
            break

    print(f"  {len(hands):>9,} hands, {len(set(ours)):>5,} distinct values "
          f"against pokerkit's {len(set(theirs)):>5,}")
    return problems

if __name__ == "__main__":
    sys.exit(main())
