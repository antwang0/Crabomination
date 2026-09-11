#!/usr/bin/env python3
"""Which asks use the SINGLE-slot resume channel inside a loop.

There are two resume channels. `scratch.resolution_answer_log` is a `Vec`
indexed by the arm's own `cursor`, so an arm can hold one answer per ask;
`scratch.stashed_resolution_answer` is ONE slot with no index, so it can hold
exactly one, for exactly one ask.

An arm that asks inside a loop therefore may not use the single slot. The
failure is not a dropped answer — it is worse. The arm re-runs from the top on
every resume, the FIRST iteration's ask takes the slot, and the answer it finds
there belongs to whichever later iteration actually suspended. `Words of Wind`
(`PlayerReturnsPermanentsToHand`, `EachPlayer`) did exactly that: seat 0's ask
swallowed seat 1's card picks, none of which are seat 0's cards, and the
forced-pick shortfall auto-filled the difference — so seat 0 bounced another of
its own permanents on every round trip and emptied its board while seat 1 was
still being asked (ENGINE_BACKLOG's sixteenth find).

This walks the engine for single-slot asks and reports the ones lexically inside
a `for` / `while` / `loop` in the same function or match arm. A hit is a defect
unless the loop provably runs at most once, which is what the allowlist below
records — with the reason, because "it happens to resolve to one player today"
is a property of the CATALOG, not of the code, and a new card can end it.

The cursor-indexed helpers (`ask_seat_bool`, `ask_seat_amount`,
`ask_seat_cards_logged`, `ask_seat_option`) are the fix and are not flagged.

Reading at the seventeenth pass: **1 single-slot ask in a loop, 1 allowlisted,
0 unexplained.** It flags `PlayerReturnsPermanentsToHand` on the tree before the
sixteenth find's fix, which is the check that it checks anything.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Every way an arm reaches the single-slot channel.
SINGLE_SLOT = (
    "take_opt_scratch!(self.stashed_resolution_answer)",
    "self.ask_seat_cards(",
    "self.choose_up_to_cards(",
)

FILES = [
    "crabomination/src/game/effects/mod.rs",
    "crabomination/src/game/effects/movement.rs",
    "crabomination/src/game/mod.rs",
    "crabomination/src/game/stack.rs",
    "crabomination/src/game/actions.rs",
    "crabomination/src/game/combat.rs",
]

# path:line -> why the loop around it runs at most once.
KNOWN = {
    "crabomination/src/game/effects/mod.rs:MayRepeat": (
        "the repeat ask's own continuation is a `MayDo` that consumes the slot "
        "itself, so the loop never re-runs this arm"
    ),
}


def indent(line: str) -> int:
    return len(line) - len(line.lstrip())


def enclosing_loop(lines, i):
    """The `for`/`while`/`loop` header above line `i` in the same arm, if any."""
    cur = indent(lines[i])
    j = i - 1
    while j >= 0:
        line = lines[j]
        if not line.strip():
            j -= 1
            continue
        k = indent(line)
        if k < cur:
            head = line.strip()
            if head.startswith(("for ", "while ", "loop {")):
                return j + 1, head
            # A match arm, a closure or a fn body starts a fresh scope: stop.
            if "=> {" in head or re.match(r"(pub(\(crate\))? )?fn ", head):
                return None
            cur = k
        j -= 1
    return None


def nearest_name(lines, i):
    """The `Effect::Variant` arm or `fn` this line sits in — the report's key."""
    j = i
    while j >= 0:
        m = re.search(r"Effect::([A-Za-z0-9_]+)", lines[j])
        if m and "=>" in lines[j]:
            return m.group(1)
        m = re.match(r"\s*(?:pub(?:\(crate\))? )?fn ([a-z0-9_]+)", lines[j])
        if m:
            return m.group(1)
        j -= 1
    return "?"


def main() -> int:
    hits, unexplained = [], []
    for rel in FILES:
        path = ROOT / rel
        if not path.exists():
            continue
        lines = path.read_text().split("\n")
        for i, line in enumerate(lines):
            if not any(p in line for p in SINGLE_SLOT):
                continue
            loop = enclosing_loop(lines, i)
            if not loop:
                continue
            name = nearest_name(lines, i)
            key = f"{rel}:{name}"
            hits.append((rel, i + 1, name, loop))
            if key not in KNOWN:
                unexplained.append((rel, i + 1, name, loop))

    print(f"{len(hits)} single-slot ask(s) inside a loop, "
          f"{len(hits) - len(unexplained)} allowlisted, {len(unexplained)} unexplained\n")
    for rel, ln, name, (loop_ln, head) in hits:
        key = f"{rel}:{name}"
        mark = "ok  " if key in KNOWN else "FLAG"
        print(f"  {mark} {rel}:{ln}  in {name}")
        print(f"       loop at {loop_ln}: {head[:78]}")
        if key in KNOWN:
            print(f"       why: {KNOWN[key]}")
        print()
    if unexplained:
        print("A loop over seats may not ask through the single-slot channel: use "
              "`ask_seat_cards_logged` / `ask_seat_bool` (cursor-indexed), and keep "
              "the mutations after every ask.")
    return 1 if unexplained else 0


if __name__ == "__main__":
    sys.exit(main())
