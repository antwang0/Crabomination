#!/usr/bin/env python3
"""Which effect loops would DROP their remaining iterations on a suspend.

A body that suspends sets `suspend_signal = (decision, pending, its OWN
remaining effect)` and returns `Ok(())`. A loop around it sees `Ok(())`, runs
the next iteration — which overwrites the signal — and the first iteration's
parked work is gone. In a duel "each opponent" is one iteration and the defect
is invisible; at four seats it drops three quarters of the effect, and a modal
spell drops every mode after the first that asks.

The fix is one line per loop: `splice_after_suspend(&mut self.suspend_signal,
|| tail)` appends the untouched remainder behind whatever the body parked, and
returns true when the loop must return. **The tail has to name its seats and
target slots INSIDE the effect** (`PlayerRef::Seat(q)`,
`Effect::BindTargetSlot`) — a parked continuation is resumed under the stack
item's `EffectContext`, not the sub-context the loop built.

This walks the engine for `run_effect` / `resolve_effect` calls lexically
inside a `for` / `while` / `loop`, and reports the ones whose loop body never
reads `suspend_signal`. Two things are not hits:

* an inline `&Effect::Variant { … }` literal whose variant is in `NEVER_ASKS`
  below — the engine built it, and it contains no ask and no card-supplied
  sub-effect, so it cannot suspend;
* an entry in `ALLOW`, which records the reason — because "the one catalog
  body is a token mint today" is a property of the CATALOG, not of the code,
  and a new card can end it.

The staleness half found three entries the day it was written: `TieredPayoff`
(the variant is gone), `DestroyAllNoRegenGainControllerLifePerManaValue` (the
arm destroys inline now, no `run_effect` at all) and
`ExileRandomGraveyardCopyTapped` (its `Move` literal wrapped onto its own
line, so `NEVER_ASKS` covers it and the entry never matched again).

Reading now: **13 sites, 13 allowlisted, 0 unexplained** — 4 of the
allowlist entries are OPEN, each with the primitive it is waiting on (see
ENGINE_BACKLOG's forty-third find). It read 18/18/0 at the find's closing tip,
15/15/0 once `Selector::ExactObjects` and `Effect::BindTargetObjects` closed
three of them, and 13/13/0 once `Effect::BindScratch` closed `Vote`'s
`PerVote` half and `RollDie`.

**An allowlist entry that stops matching fails too.** A site someone fixed
leaves its reason behind, and a stale reason is worse than none: the next
reader takes it for a live constraint.

⚠ **What it does NOT see: a sequential PAIR.** The class is "a second
`run_effect` after one that can suspend", and a loop is only its commonest
shape. The pile splits (`SeparateIntoPiles`, `ChooseOneAmong`) were the
known pair and are fixed (`run_piles_then_clear`), but this script did not
find them and would not find the next one: "is this arm's second statement
reachable after a suspend" is a dataflow question, not a lexical one.

**Injection:** deleting the `splice_after_suspend` call from
`Effect::ForEachOpponent` takes 0 -> **1 unexplained**, and deleting an
`ALLOW` entry's site takes it to **1 stale**. A gate that cannot fail is worse
than no gate; this one can, twice.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

FILES = [
    "crabomination/src/game/effects/mod.rs",
    "crabomination/src/game/effects/movement.rs",
    "crabomination/src/game/effects/commander.rs",
    "crabomination/src/game/effects/delayed.rs",
    "crabomination/src/game/mod.rs",
    "crabomination/src/game/stack.rs",
    "crabomination/src/game/actions.rs",
    "crabomination/src/game/combat.rs",
]

LOOP_HEAD = re.compile(r"^\s*(?:\}\s*)?(?:for\s|while\s|loop\s*\{)")
RUN = re.compile(r"\b\w+\.(run_effect|resolve_effect\w*)\(")
ARM = re.compile(r"^\s*(?:\|\s*)?Effect::([A-Za-z0-9_]+)")
FN = re.compile(r"^\s*(?:pub(?:\(crate\))?\s+)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)")
LITERAL = re.compile(r"^&Effect::([A-Za-z0-9_]+)")

# Variants the engine builds inline that contain no ask and no card-supplied
# sub-effect, so the loop around one cannot lose anything. Keep this short and
# provable by reading the arm; a variant with a `Box<Effect>` field does NOT
# belong here.
NEVER_ASKS = {
    "AddCounter",
    "AddManaAtNextMainPhase",
    "CounterSpell",
    "CreateToken",
    "CreateTokenCopyOf",
    "Destroy",
    # The draw path answers its own replacement offers through
    # `self.decider.decide` (`mod.rs` ~19,250) rather than suspending, so a
    # `Draw` in a loop cannot park anything. If a draw replacement ever gains
    # a seat-routed ask, take this out.
    "Draw",
    "GainLife",
    "GrantKeyword",
    "LoseLife",
    "ManifestDread",
    "Move",
    "SacrificeSource",
}

# (scope, argument) -> why this loop cannot drop anything.
#
# `scope` is the nearest enclosing `Effect::` match arm, or the function name
# when there is none. `argument` is the run call's first argument,
# whitespace-collapsed.
ALLOW = {
    # ── The loop provably runs at most once ───────────────────────────────
    ("PlayersMayAccept", "on_accept"): "the first acceptor returns; the loop ends there",
    ("AnyPlayerMayAccept", "accepted"): "ditto",
    ("AnyPlayerMayExileFromGraveyard", "then"): "ditto",
    # ── The effect run cannot ask ─────────────────────────────────────────
    ("DrainDefendersLandsForManaNextMain", "&ability.effect"):
        "`is_mana_ability_public` gates the pick: CR 605.1a mana abilities "
        "do not use the stack and do not ask",
    ("resolve_destroy_targets_polymorph", "&Effect::RevealUntilFind {"): "ditto",
    ("SearchExileLinked", "&Effect::Search {"):
        "the search asks, and the arm is a `for _ in 0..n` over one seat — "
        "OPEN, see ENGINE_BACKLOG: the tail needs the already-exiled picks' "
        "`exiled_with` stamps re-applied, which the count alone cannot carry",
    # ── Driven off the stack, where the resolver answers the ask itself ───
    ("process_exile_countdowns", "&fuse.effect"):
        "`resolve_effect_driven` — off the stack there is nothing to park a "
        "continuation on, so the ask is answered in place",
    ("apply_opening_hand_effects", "&extra"): "ditto",
    # ── OPEN, each waiting on a primitive (ENGINE_BACKLOG has all four) ───
    ("AnteTopOfLibrary", "inner"):
        "OPEN: the second pass antes AND runs the branch, so a tail would "
        "have to hoist every ante ahead of every branch. CR 407 ante is in "
        "no pool",
    ("FlipCoinsChooseCount", "per_win"):
        "OPEN: the tail would have to re-flip the remaining coins, and the "
        "arm's `wins` tally is read after the loop. ⚠ This pair was "
        "allowlisted under `OpponentRevealsPickToBattlefield` until "
        "`scope_at` learned to read a multi-line arm pattern",
    ("FlipCoinsChooseCount", "per_loss"): "ditto",
    ("MayPayRepeatedly", "body"):
        "OPEN: the arm's resume path is a re-run from the top over the "
        "answer log (`answer_already_acted_on`), which a spliced tail would "
        "double-count. Both catalog bodies are `Effect::Untap`, i.e. "
        "choiceless, so nothing is reachable through it today",
    ("SacrificeAnyNumber", "per_each"):
        "OPEN by the CATALOG, the same judgement `ForEachOpponent` used to "
        "carry: all six `per_each` bodies are choiceless today (draw, "
        "counters, drain, life), so no tail is reachable. The sacrifices are "
        "inline zone moves rather than an `Effect`, so the fix when one "
        "stops being choiceless is to HOIST every sacrifice ahead of every "
        "payoff — which is what CR 608.2 says the printed text does anyway — "
        "after which the tail is a plain `Seq` of `per_each`. ⚠ That hoist "
        "moves `Value::SacrificedCount` inside `per_each` from 1..n to n; no "
        "card reads it there today (Last-Ditch Effort's `per_each` is Noop)",
}


def strip_code(line: str) -> str:
    """Drop `//` comments and string bodies so braces inside them don't count."""
    out, i, in_str = [], 0, False
    while i < len(line):
        c = line[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
            i += 1
            continue
        if c == '"':
            in_str = True
            i += 1
            continue
        if c == "/" and i + 1 < len(line) and line[i + 1] == "/":
            break
        out.append(c)
        i += 1
    return "".join(out)


ARM_TAIL = re.compile(r"^\s*\}\s*=>")


def scope_at(lines, idx):
    """The nearest enclosing `Effect::` match arm, else the function name.

    An arm whose pattern is destructured over several lines has its `=>` on
    the closing brace, not on the `Effect::Variant {` line — so a one-line
    test walks straight past it and labels the site with whatever arm came
    before. That mislabelled the `FlipCoinsChooseCount` site as
    `OpponentRevealsPickToBattlefield`, i.e. sent the reader to an arm with
    no loop in it at all.
    """
    for j in range(idx, -1, -1):
        m = ARM.match(lines[j])
        if m and ("=>" in lines[j] or any(
            ARM_TAIL.match(lines[k]) for k in range(j + 1, min(j + 25, len(lines)))
        )):
            return m.group(1)
        m = FN.match(lines[j])
        if m:
            return m.group(1)
    return "?"


def first_arg(lines, idx):
    """The run call's first argument, as written."""
    m = RUN.search(lines[idx])
    rest = lines[idx][m.end():].strip()
    if not rest and idx + 1 < len(lines):
        rest = lines[idx + 1].strip()
    rest = re.sub(r"\s+", " ", rest)
    # Trim the trailing `, ctx, events)?;` the call site writes on one line.
    rest = re.split(r", &?(?:ctx|sub|sub_ctx|opt_ctx|land_ctx|\w*_ctx)\b", rest)[0]
    return rest.rstrip(",").rstrip(")").strip()


def audit(path):
    lines = (ROOT / path).read_text().split("\n")
    depths, d = [], 0
    for line in lines:
        depths.append(d)
        code = strip_code(line)
        d += code.count("{") - code.count("}")
    hits, loop_stack = [], []
    for i, raw in enumerate(lines):
        code = strip_code(raw)
        while loop_stack and depths[i] <= depths[loop_stack[-1]] and i != loop_stack[-1]:
            loop_stack.pop()
        if RUN.search(code) and loop_stack:
            start = loop_stack[-1]
            end = next((j for j in range(start + 1, len(lines))
                        if depths[j] <= depths[start]), len(lines))
            body = "\n".join(lines[start:end])
            arg = first_arg(lines, i)
            lit = LITERAL.match(arg)
            if lit and lit.group(1) in NEVER_ASKS:
                continue
            if "suspend_signal" not in body and "splice_after_suspend" not in body:
                hits.append((i + 1, scope_at(lines, i), arg))
        if LOOP_HEAD.match(code) and "{" in code:
            loop_stack.append(i)
    return hits


def main():
    unexplained, matched = [], set()
    for path in FILES:
        for line, scope, arg in audit(path):
            if (scope, arg) in ALLOW:
                matched.add((scope, arg))
                continue
            unexplained.append((path, line, scope, arg))
    for path, line, scope, arg in unexplained:
        print(f"{path}:{line}  {scope}  run_effect({arg})")
    stale = sorted(set(ALLOW) - matched)
    for scope, arg in stale:
        print(f"STALE allowlist entry, the site is gone: {scope}  run_effect({arg})")
    print(f"\n{len(unexplained) + len(matched)} asking run-in-loop sites, "
          f"{len(matched)} allowlisted, {len(unexplained)} unexplained, "
          f"{len(stale)} stale")
    return 1 if unexplained or stale else 0


if __name__ == "__main__":
    sys.exit(main())
