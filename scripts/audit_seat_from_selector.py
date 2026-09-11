#!/usr/bin/env python3
"""Which asking arms re-derive the ASKED SEAT on their re-run.

An arm that suspends hands `ask_seat_*` an `Effect` to re-run when the answer
comes back, and almost every one hands it the arm's own `effect`. That is right
for the arm's *work* and wrong for the arm's *seat* whenever the seat came out of
a selector: the re-run resolves the selector again, against a board the first
pass — or an earlier sibling in the same `Seq` — has already changed.

Ghost Quarter is the worked example (ENGINE_BACKLOG's seventeenth find). Its
ability is `Seq[Destroy target land, MayDoBy{ ControllerOf(Target(0)), … }]`, so
by the time the arm asks, the land the selector points at is in a graveyard; on
the re-run the selector could resolve to nothing at all, the arm returned at its
own `let Some(seat)`, and the compensation search never happened. A sweep found
it as an answer-log leak, which is the symptom of returning past a replayed
answer.

The fix per arm is one rebuild: pass a continuation whose `who` is
`PlayerRef::Seat(seat)` instead of the selector. It is always safe — it is the
same seat the first pass asked — and it is what `Effect::Sacrifice`'s
`per_seat_continuation` has always done.

This flags, per match arm or fn: a seat bound from `resolve_player(who…)` /
`resolve_players(who…)` together with an `ask_seat_*` whose continuation
argument is the bare `effect`. Multi-seat loops are flagged too: the same
re-derivation applies to every seat in the list.

Reading at the seventeenth pass: **27 arms open, 0 demonstrated.** `MayDoBy` is
absent because it is fixed, and it is the only one with a shipped card that
actually reaches the failure: Ghost Quarter and Volatile Fault both put a
`Destroy`/`Move` of slot 0 *before* an ask whose `who` reads slot 0. A catalog
scan for that shape over the other 26 finds nothing — which is a fact about the
CATALOG, not about the code, and is exactly why they are listed rather than
allowlisted. A new card pairing any of them with an earlier Destroy of the same
slot ends the reprieve, and the fix is six lines per arm.

The scan that produced that reading, for whoever repeats it: every
`who: PlayerRef::(ControllerOf|OwnerOf)(Selector::Target(N))` in the catalog
whose enclosing card has an earlier `Destroy` / `Exile` / `Sacrifice`, then read
each hit to see whether the arm it feeds is one of the 27 (most feed
`Effect::Search`, whose resume applies the pick rather than re-running the arm).
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "crabomination/src/game/effects/mod.rs"

ASKS = (
    "self.ask_seat_bool(",
    "self.ask_seat_amount(",
    "self.ask_seat_cards_logged(",
    "self.ask_seat_target_logged(",
    "self.ask_seat_option(",
    "self.ask_seat_cards(",
    "self.choose_up_to_cards(",
)

def indent(line: str) -> int:
    return len(line) - len(line.lstrip())


def main() -> int:
    lines = SRC.read_text().split("\n")
    # Split into scopes: each `Effect::Variant .. => {` arm, and each `fn`.
    scopes = []  # (name, start, end)
    starts = []
    for i, line in enumerate(lines):
        st = line.strip()
        m = re.match(r"(?:pub(?:\(crate\))? )?fn ([a-z0-9_]+)", st)
        if m:
            starts.append((m.group(1), i, indent(line)))
            continue
        if "=> {" in st:
            v = re.search(r"Effect::([A-Za-z0-9_]+)", st)
            if v:
                starts.append((v.group(1), i, indent(line)))
    for k, (name, i, ind) in enumerate(starts):
        end = starts[k + 1][1] if k + 1 < len(starts) else len(lines)
        scopes.append((name, i, end))

    flagged = []
    for name, a, b in scopes:
        body = lines[a:b]
        text = "\n".join(body)
        if not re.search(r"resolve_players?\(who", text):
            continue
        # An ask whose continuation argument is the bare `effect`.
        asked = None
        for j, line in enumerate(body):
            if any(p in line for p in ASKS):
                window = "\n".join(body[j : j + 14])
                if re.search(r"^\s+effect,\s*$", window, re.M) or re.search(
                    r"\beffect\)", window
                ):
                    asked = a + j + 1
                    break
        if asked is None:
            continue
        flagged.append((name, a + 1, asked))

    print(
        f"{len(flagged)} arm(s) ask a selector-derived seat with a bare "
        f"continuation (`MayDoBy` is absent because it is fixed)\n"
    )
    for name, arm_ln, ask_ln in flagged:
        print(f"  {name:44} arm at {arm_ln}, ask at {ask_ln}")
    if flagged:
        print(
            "\nEach re-derives its seat on the re-run. Pass a continuation whose "
            "`who` is `PlayerRef::Seat(seat)` — the seat the first pass actually "
            "asked — the way `Effect::MayDoBy` now does."
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
