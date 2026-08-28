#!/usr/bin/env python3
"""Which `decider.decide` sites are *plumbed*, and which answer for every seat.

The bug this is a filter for is the 2026-07 "Ad Nauseam pattern": a
mid-resolution choice that consults `self.decider` directly. `AutoDecider`'s
blanket defaults then answer for **every** seat — bots and `wants_ui` humans
alike — and its defaults are the worst available answer often enough to kill a
whole keyword. `OptionalTrigger` reads *no*, so Cascade never cast anything;
`ChooseAmount` reads *0*, so "destroy all creatures with power N or greater"
read as a symmetric wrath.

A **plumbed** site is one that does at least one of three things within its
own statement region:

  * `seat_suspends(..)` + `suspend_signal` — a `wants_ui` seat gets a real
    suspended offer instead of a synchronous answer;
  * `stashed_resolution_answer` — it is the resume half of such an offer;
  * a `DeciderKind` branch — it distinguishes `Auto` from a real decider, so
    the headless default is chosen deliberately rather than inherited.

A **bare** site does none of the three. That is not automatically a bug: a
few are genuinely seat-independent (an engine-internal question with one
right answer), which is why the output is a triage list rather than a gate.
It *is* the population the 2026-07 audit walked by hand, and re-running it is
how that audit's "~45 live bugs" gets re-checked instead of re-quoted.

    python3 scripts/audit_decision_plumbing.py            # counts + bare list
    python3 scripts/audit_decision_plumbing.py --verbose  # + the plumbed ones

Exit status is 0 always: the bare count is a number to compare against the
last reading, not a pass/fail.
"""

import os
import re
import sys

BASE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

DIRS = [
    "crabomination/src/game",
    "crabomination/src/server",
    "crabomination/src",
]

CALL = re.compile(r"\bdecider\s*\.\s*decide\s*\(")
MARKERS = (
    # The resolution-time suspension pair.
    "seat_suspends",
    "suspend_signal",
    "stashed_resolution_answer",
    # The *action*-time one: `pending_decision` + a `ResumeContext`, which is
    # how combat ordering and the cast pipeline suspend. Different mechanism,
    # same property — the `wants_ui` seat is asked properly and only the
    # headless path reaches `decide`.
    "pending_decision",
    "wants_ui",
    # An explicit `Auto` branch: the headless default is chosen here rather
    # than inherited from `AutoDecider`.
    "DeciderKind",
)
# How far back a marker still counts as covering the call. The plumbed shape is
# a `match`/`if` whose arms are the suspend branch and the headless branch, and
# the widest one in the engine (`ChooseNumberDestroyByPower`) spans 26 lines
# from `seat_suspends` to `decide`.
LOOKBACK = 40
# The enclosing `fn` is reported so a finding names something greppable — and
# for `run_effect`, which is one `match` with ~1000 arms, the enclosing **arm**
# is what actually names it.
FN = re.compile(r"^\s*(?:pub(?:\(crate\))?\s+)?(?:async\s+)?fn ([a-z_0-9]+)")
ARM = re.compile(r"^\s{8,20}(?:\|\s*)?(Effect|StaticEffect|GameAction)::([A-Za-z0-9_]+)")


def rust_files():
    seen = set()
    for root in DIRS:
        for dirpath, _, files in os.walk(os.path.join(BASE, root)):
            for f in files:
                if not f.endswith(".rs"):
                    continue
                p = os.path.relpath(os.path.join(dirpath, f), BASE)
                if p not in seen:
                    seen.add(p)
                    yield p


def main() -> int:
    verbose = "--verbose" in sys.argv
    bare, plumbed = [], []
    for path in sorted(rust_files()):
        lines = open(os.path.join(BASE, path)).read().split("\n")
        fns = []
        for i, l in enumerate(lines):
            m = FN.match(l)
            if m:
                fns.append((i, m.group(1)))

        arms = []
        for i, l in enumerate(lines):
            m = ARM.match(l)
            if m and l.rstrip().endswith(("=> {", "{", "=>")):
                arms.append((i, f"{m.group(1)}::{m.group(2)}"))

        def owner(idx):
            best = "?"
            for s, n in fns:
                if s <= idx:
                    best = n
                else:
                    break
            if best in ("run_effect", "resolve_effect"):
                arm = None
                for s, n in arms:
                    if s <= idx:
                        arm = n
                    else:
                        break
                if arm:
                    return f"{best} / {arm}"
            return best

        for i, l in enumerate(lines):
            if not CALL.search(l) or l.strip().startswith("//"):
                continue
            window = "\n".join(lines[max(0, i - LOOKBACK) : i + 1])
            hit = [m for m in MARKERS if m in window]
            entry = (path, i + 1, owner(i), hit)
            (plumbed if hit else bare).append(entry)

    total = len(bare) + len(plumbed)
    print(f"{total} `decider.decide` sites: {len(plumbed)} plumbed, {len(bare)} bare")
    for path, ln, fn, _ in bare:
        print(f"    BARE  {path}:{ln}  in {fn}")
    if verbose:
        for path, ln, fn, hit in plumbed:
            print(f"    ok    {path}:{ln}  in {fn}  ({', '.join(hit)})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
