"""Checks the Python binding against the same figures the Rust tests use.

Build the module first:

    cargo build --release --features python
    cp target/release/libpoker_equity.so <somewhere>/poker_equity.so

then run this with that directory on PYTHONPATH:

    PYTHONPATH=<somewhere> python3 tests/python/test_binding.py

It is not wired into `cargo test`, because building an extension module and
loading it into an interpreter is a different job from running the library's
own tests, and pyo3's extension-module feature deliberately leaves the Python
symbols unresolved so a Rust test binary cannot link it.
"""

import sys

import poker_equity as pc

failures = []


def check(condition, message):
    if condition:
        print(f"  ok   {message}")
    else:
        print(f"  FAIL {message}")
        failures.append(message)


def close(actual, expected, tolerance, message):
    check(abs(actual - expected) < tolerance, f"{message}: {actual:.4f} ~ {expected:.4f}")


print("every game is reachable by key")
keys = {v["key"] for v in pc.variants()}
check(len(keys) == 14, f"fourteen games, got {len(keys)}")
check("omaha_five_hi_lo" in keys, "five-card Omaha hi/lo is there")
check(not any(k[0].isdigit() for k in keys), "no key starts with a digit")

print("\nexact results carry no error bar")
result = pc.exact_from_text("holdem", ["AhAd", "KsKc"], "2c 7d 9h")
check(result["exact"], "reported as exact")
check(result["samples"] == 990, f"990 boards, got {result['samples']}")
close(result["players"][0]["equity"] * 100, 91.6162, 0.001, "aces against kings on 2-7-9")
check(result["players"][0]["std_error"] == 0.0, "standard error is exactly zero")

print("\nsampling lands on the same answer")
sampled = pc.chunk_from_text("holdem", ["AhAd", "KsKc"], "2c 7d 9h", samples=200_000, seed=11)
close(sampled["players"][0]["equity"] * 100, 91.6162, 0.5, "within half a point of exact")

print("\nequities divide one pot")
for key, hands, board in [
    ("holdem", ["AhKh", "QsQd"], ""),
    ("omaha", ["AhAdKsQc", "JhTc9s8d"], ""),
    ("omaha_hi_lo", ["Ah2c3d4s", "KhKsQhQs"], "5c 6d 8h"),
    ("stud", ["Ah2c3d", "Qs Qd Js"], ""),
    ("deuce_seven", ["Th8c4s2h", "9d7h4h2d"], ""),
    ("badugi", ["Ac2d3h", "4s6s7d"], ""),
]:
    result = pc.chunk_from_text(key, hands, board, samples=20_000, seed=5)
    total = sum(p["equity"] for p in result["players"])
    close(total, 1.0, 1e-9, f"{key} equities sum")

print("\nbatches spread across threads")
check(pc.default_thread_count() >= 1, "a default thread count is offered")
one = pc.chunk_from_text("holdem", ["AhKh", "QsQd"], samples=200_000, seed=5, threads=1)
many = pc.chunk_from_text("holdem", ["AhKh", "QsQd"], samples=200_000, seed=5, threads=4)
check(one["samples"] == many["samples"] == 200_000, "both ran every deal asked for")
close(one["players"][0]["equity"], many["players"][0]["equity"], 0.01,
      "one thread and four agree")

print("\nthe raw sums cross, so batches can be merged here")
a = pc.chunk_from_text("holdem", ["AhKh", "QsQd"], samples=100_000, seed=1)
b = pc.chunk_from_text("holdem", ["AhKh", "QsQd"], samples=100_000, seed=2)
for field in ("share_sum", "share_square_sum", "low_share_sum",
              "win_count", "tie_count", "scoop_count"):
    check(field in a, f"{field} is returned")
merged = sum(x + y for x, y in zip(a["share_sum"][:1], b["share_sum"][:1]))
close(merged / (a["samples"] + b["samples"]),
      a["players"][0]["equity"], 0.01, "hand-merged batches give the same equity")

print("\nexact answers from masks, not only from text")
hands = [[[1 << pc.card_index("AhKh"[i:i+2]) for i in (0, 2)]],
         [[1 << pc.card_index("QsQd"[i:i+2]) for i in (0, 2)]]]
board = [1 << pc.card_index(c) for c in ("2c", "7d", "9h")]
walked = pc.exact("holdem", hands, board, 0)
check(walked is not None and walked["exact"], "a river-ish spot comes back exact")
check(walked["samples"] == 990, f"990 run-outs, got {walked['samples']}")
check(walked["players"][0]["std_error"] == 0.0, "and carries no error bar")

print("\nthe mask API agrees with the text API")
masks = [[[1 << pc.card_index("Ah"), 1 << pc.card_index("Kh")]],
         [[1 << pc.card_index("Qs"), 1 << pc.card_index("Qd")]]]
by_mask = pc.chunk("holdem", masks, [], 0, 200_000, 3)
by_text = pc.chunk_from_text("holdem", ["AhKh", "QsQd"], samples=200_000, seed=3)
close(by_mask["players"][0]["equity"], by_text["players"][0]["equity"], 1e-12,
      "identical, deal for deal")

print("\ncard indices are rank * 4 + suit")
check(pc.card_index("2c") == 0, "the deuce of clubs is zero")
check(pc.card_index("Ah") == 12 * 4 + 2, "the ace of hearts")
check(pc.card_name(0) == "2c", "and back again")
check(pc.full_deck() == (1 << 52) - 1, "the deck is fifty-two bits")

print("\nthe parser hands back masks")
check(len(pc.parse_hand_field("AKs", 2)) == 4, "AKs is four combinations")
check(len(pc.parse_hand_field("22+", 2)) == 78, "22+ is every pair")
check(pc.parse_hand_field("2 c", 2)[0] == [0xF, 0x1111111111111], "a deuce and a club")
check(len(pc.parse_hand_field("Ah 2c 3d", 7, allow_short=True)[0]) == 3,
      "a stud field may be short")
check(pc.parse_dead_cards("Ah Kd") == (1 << pc.card_index("Ah")) | (1 << pc.card_index("Kd")),
      "dead cards are one mask")

print("\nbad input raises, with the span")
for bad, slots in [("AhXh", 2), ("AhAh", 2), ("22s", 2), ("top 15%", 2)]:
    try:
        pc.parse_hand_field(bad, slots)
        check(False, f"{bad!r} should have raised")
    except ValueError as error:
        check("\n" in str(error), f"{bad!r} raises with an underlined span")

try:
    pc.chunk_from_text("no_such_game", ["AhKh", "QsQd"])
    check(False, "an unknown game should raise")
except ValueError as error:
    check("no game called" in str(error), "an unknown game names itself")

print()
if failures:
    print(f"{len(failures)} checks failed")
    sys.exit(1)
print("all checks passed")
