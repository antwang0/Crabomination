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


# `AutoDecider`'s answer per `Decision` variant, for the variants whose answer
# makes the asking effect do **nothing** — the population the MayPay find was
# in. The value is the default, so a reader can see why the row is degenerate;
# `None` means the default is a real answer (the site plays worse, not dead).
DEGENERATE = {
    "OptionalTrigger": "Bool(false) — declines",
    "ChooseAmount": "Amount(0) — pays/takes nothing",
    "ChooseCards": "the first `min` (empty when min = 0)",
    "ChooseTarget": None,
    "ChooseMode": None,
    "ChooseModes": None,
    "ChooseColor": None,
    "ChooseOption": None,
    "ChooseCreatureType": None,
    "ChooseCreatureTypePair": None,
    "Discard": None,
    "PutOnLibrary": None,
    "Scry": None,
    "SearchLibrary": None,
    "NameCard": None,
    "CoinFlip": None,
    "DieRoll": None,
    "CombatDamageOrder": None,
    "AssignCombatDamage": None,
    "OrderTriggers": None,
    "DivideDamage": None,
    "ChooseLegendToKeep": None,
    "CommanderRedirect": None,
    "Mulligan": None,
    "Learn": None,
    "?": None,
}
DECISION = re.compile(r"\bDecision::([A-Za-z0-9_]+)")
# A site that *names* the headless answer in a comment beside the ask has
# chosen it on purpose (Nameless Race pays as much as it can survive on a 0).
ACK = re.compile(
    r"AutoDecider|auto\b|headless|printed default|no opinion|synchronous", re.I
)
# A degenerate answer *inside a loop* usually declines a **repetition**, not the
# effect: Kindle the Carnage discards and damages once before it asks "again?",
# so a `false` leaves the printed minimum done. Worth a different column from a
# gate in front of the whole body — the loop shape is the one the 2026-07 audit
# over-counted.
LOOP = re.compile(r"^\s*(?:\}\s*)?(?:for |while |loop\s*\{|'[a-z_]+: loop)")


def in_loop(lines, i):
    """True when the ask on line `i` gates a *repetition*.

    Walks up while the indentation is shrinking, stopping at the enclosing
    `match` arm or `fn`. A loop found on the way only counts when its body runs
    a statement **before** the ask — Kindle the Carnage discards and damages,
    then asks "again?". A loop whose ask is its first statement (Fasting's
    per-offer prompt) gates that iteration's whole body, so it stays DEAD.
    """
    indent = len(lines[i]) - len(lines[i].lstrip())
    for j in range(i - 1, max(0, i - 120), -1):
        l = lines[j]
        if not l.strip():
            continue
        ind = len(l) - len(l.lstrip())
        if ind >= indent:
            continue
        indent = ind
        if LOOP.match(l):
            body = ind + 4
            heads = [
                k
                for k in range(j + 1, i + 1)
                if lines[k].strip()
                and not lines[k].lstrip().startswith("//")
                and len(lines[k]) - len(lines[k].lstrip()) == body
            ]
            if len(heads) > 1:
                return True
            # The other repeat shape: the ask is the loop's first statement but
            # guarded off the loop counter, so iteration 0 runs unasked
            # (`if i > 0 && !yes { break }` — MayRepeat, Trade Secrets).
            var = re.match(r"^\s*for (\w+) in ", l)
            prefix = "\n".join(lines[j : i + 1])
            return bool(var and re.search(rf"\b{var.group(1)}\s*>\s*0", prefix))
        if FN.match(l) or ARM.match(l):
            return False
    return False


def decision_kind(lines, i):
    """`(variant, degenerate, acknowledged)` for the ask on line `i`.

    The variant is read forward from the call — `decide(&Decision::X {` is the
    shape in ~90 % of sites — then backward for the `let d = Decision::X`
    two-statement form.
    """
    fwd = "\n".join(lines[i : i + 4])
    m = DECISION.search(fwd)
    if not m:
        m = DECISION.search("\n".join(lines[max(0, i - 12) : i + 1]))
    kind = m.group(1) if m else "?"
    body = "\n".join(lines[max(0, i - 8) : i + 12])
    degenerate = DEGENERATE.get(kind, "unknown variant")
    if kind == "ChooseCards" and re.search(r"min:\s*(?!0)[1-9a-z_]", body):
        degenerate = None  # a forced "choose exactly N" auto-picks the N
    # The comment that names the headless answer sits either above the ask or
    # on the `_ =>` fallback arm a few lines below it.
    comment = "\n".join(
        l for l in lines[max(0, i - 20) : i + 12] if l.lstrip().startswith("//")
    )
    return kind, degenerate, bool(ACK.search(comment)), in_loop(lines, i)


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
            entry = (path, i + 1, owner(i), hit, *decision_kind(lines, i))
            (plumbed if hit else bare).append(entry)

    total = len(bare) + len(plumbed)
    print(f"{total} `decider.decide` sites: {len(plumbed)} plumbed, {len(bare)} bare")
    degen = [e for e in bare if e[5]]
    gates = [e for e in degen if not e[7] and not e[6]]
    print(
        f"of the bare: {len(degen)} have a degenerate headless default — "
        f"{len(gates)} of those gate a whole effect body (DEAD), "
        f"{len([e for e in degen if e[7]])} gate a loop repetition (the printed "
        f"minimum still happens), {len([e for e in degen if e[6]])} name the "
        f"headless answer in a comment beside the ask"
    )
    for kind in sorted({e[4] for e in degen}):
        rows = [e for e in degen if e[4] == kind]
        print(f"\n--- {kind} -> {DEGENERATE[kind]}: {len(rows)}")
        for path, ln, fn, _, _, _, ack, loop in rows:
            tag = "repeat" if loop else "ack   " if ack else "DEAD  "
            print(f"    {tag}  {path}:{ln}  in {fn}")
    print("\n--- bare, non-degenerate default (quality of play, not a no-op)")
    for path, ln, fn, _, kind, _, _, _ in bare:
        if not DEGENERATE.get(kind):
            print(f"    bare  {path}:{ln}  in {fn}  [{kind}]")
    if verbose:
        for path, ln, fn, hit, *_ in plumbed:
            print(f"    ok    {path}:{ln}  in {fn}  ({', '.join(hit)})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
