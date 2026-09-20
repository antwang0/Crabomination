#!/usr/bin/env python3
"""Catalog sites that hand a FAN-OUT `PlayerRef` to an effect arm that
resolves it SINGULARLY.

`GameState::resolve_player` answers a fan-out ref (`EachOpponent`,
`EachPlayer`, `EachTeammate`, …) with the **first seat of its set** and drops
the rest, silently. That is exact while the set holds one player — which is
what `EachOpponent` is in a duel, and why this class hides in a two-player
engine — and it is a defect the moment the set holds two:

* a printed "choose an opponent" becomes "the first opponent by seat index",
  so the table's seating makes a choice the card gives its controller
  (`RememberPlayerOnSource`, eight cards, fixed 2026-09-19);
* a printed "each player searches their library" searches seat 0's and
  nobody else's (`Search { who: EachPlayer }`, fixed earlier).

`resolve_player` carries a `debug_assert!` for exactly this, so the defect is
loud *when a game actually reaches the arm*. The gap this script closes is
coverage: the assertion only fires for cards a pod game plays, and the pod
decks are eight lists.

How it reads the pair:

1. **The singular arms.** Every `Effect::Name { … who … } =>` arm in
   `game/effects/mod.rs` whose body calls `self.resolve_player(who`. The arm
   is delimited by the next `Effect::` arm head at the same indent.
2. **The fan-out sites.** Every `Effect::Name { … who: PlayerRef::<fan-out>`
   in the catalog, for a `Name` from (1).

⚠ Two limits, both shared with every name-keyed script here. An arm that
resolves `who` singularly *behind a guard* that can only hold one seat is
still reported (there is no such arm today). And a catalog site that reaches
the effect through a helper taking the ref as a parameter is not seen —
`grep` for the helper if a count moves without a card moving.

Run: `python3 scripts/audit_singular_fanout.py [--gate]`
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
EFFECTS = ROOT / "crabomination/src/game/effects/mod.rs"
CATALOG = ROOT / "crabomination_catalog/src"

# `PlayerRef::is_fan_out`'s list, kept in sync by the test that reads it.
FAN_OUT = [
    "EachOpponent",
    "EachOpponentExceptTriggerer",
    "OpponentsWhoVotedDifferently",
    "EachPlayer",
    "EachPlayerWithoutMaxSpeed",
    "EachTeammate",
    "EachPlayerExceptControllerOf",
]

ARM_HEAD = re.compile(r"^(\s*)Effect::(\w+)\s*[{(]", re.M)

# The SELECTOR half of the same class: an arm that takes its `who` as a
# `Selector` and consumes only the first entity. `Selector::Player(<fan-out>)`
# in such a field has the identical failure — exact in a duel, first seat in a
# pod. Detected the same way, matched against a different spelling.
SELECTOR_CONSUMED_SINGLY = (
    "resolve_selector(who, ctx).into_iter().next()",
    "resolve_selector(who, ctx).first()",
)

# Sites left on a fan-out ref ON PURPOSE, each with the reason and what it is
# waiting on. `--gate` fails on anything outside this list, and on an entry
# whose site is gone (a stale allowlist hides the next regression).
ALLOWLIST = {
    ("ManifestDread", "unidentified_hovership"):
        'printed "the exiled card\'s OWNER manifests dread" — the ref wanted is '
        "the owner of this Vehicle's linked exile, which no `PlayerRef` names. "
        "`EachOpponent` stands in and is exact in a duel.",
}


def singular_arms():
    """{effect name} for arms that resolve a `who` field singularly."""
    src = EFFECTS.read_text()
    heads = [(m.start(), m.group(2)) for m in ARM_HEAD.finditer(src)]
    out = set()
    for i, (start, name) in enumerate(heads):
        stop = heads[i + 1][0] if i + 1 < len(heads) else len(src)
        body = src[start:stop]
        if "self.resolve_player(who" in body:
            out.add(name)
    return out


def singular_selector_arms():
    """{effect name} for arms whose `who: Selector` is consumed first-only."""
    src = EFFECTS.read_text()
    heads = [(m.start(), m.group(2)) for m in ARM_HEAD.finditer(src)]
    out = set()
    for i, (start, name) in enumerate(heads):
        stop = heads[i + 1][0] if i + 1 < len(heads) else len(src)
        body = src[start:stop]
        if any(pat in body for pat in SELECTOR_CONSUMED_SINGLY):
            out.add(name)
    return out


def fan_out_sites(names, selector_names=frozenset()):
    """[(effect, ref, file, line)] for catalog sites filling `who` with one."""
    alt = "|".join(sorted(names))
    refs = "|".join(FAN_OUT)
    # `Effect::Name { … who: PlayerRef::Each… }` — the field may sit on its own
    # line, so allow anything but a closing brace in between. The selector half
    # is the same clause wrapped: `who: Selector::Player(PlayerRef::Each…)`.
    alts = "|".join(sorted(selector_names)) or "(?!)"
    pat = re.compile(
        rf"Effect::({alt})\s*\{{[^}}]*?who:\s*PlayerRef::({refs})\b"
        rf"|Effect::({alts})\s*\{{[^}}]*?who:\s*Selector::Player\(PlayerRef::({refs})\b",
        re.S,
    )
    hits = []
    for path in sorted(CATALOG.rglob("*.rs")):
        src = path.read_text()
        for m in pat.finditer(src):
            line = src[: m.start()].count("\n") + 1
            effect = m.group(1) or m.group(3)
            ref = m.group(2) or m.group(4)
            hits.append((effect, ref, path.relative_to(ROOT), line))
    return hits


FN_HEAD = re.compile(r"^(?:pub )?fn (\w+)\(")


def factory_at(path, line):
    """The factory IDENT whose body holds `line`.

    The ident, not the printed name: every other script here keys on the first
    string literal in the body, and a body that ends in `..creature("Name", …)`
    rather than opening with a `name:` field is then filed under the PREVIOUS
    factory's name. Longhorn Firebeast came back as "Gurzigost".
    """
    src = (ROOT / path).read_text().split("\n")
    name = "?"
    for text in src[:line]:
        m = FN_HEAD.match(text)
        if m:
            name = m.group(1)
    return name


def main():
    gate = "--gate" in sys.argv[1:]
    names = singular_arms()
    sel_names = singular_selector_arms()
    hits = fan_out_sites(names, sel_names)
    print(
        f"{len(names)} effect arms resolve their `who` singularly, "
        f"{len(sel_names)} more consume a `who: Selector` first-only."
    )
    unexpected, seen = [], set()
    for effect, ref, path, line in sorted(hits, key=lambda h: (h[0], str(h[2]), h[3])):
        card = factory_at(path, line)
        key = (effect, card)
        mark = "ok " if key in ALLOWLIST else "NEW"
        if key in ALLOWLIST:
            seen.add(key)
        else:
            unexpected.append((effect, ref, path, line, card))
        print(f"  {mark} {effect} <- PlayerRef::{ref}   {card}   {path}:{line}")
    stale = sorted(set(ALLOWLIST) - seen)
    print(f"\n{len(hits)} catalog sites hand one a fan-out ref: "
          f"{len(unexpected)} unexpected, {len(stale)} stale allowlist entries.")
    if not gate:
        for (effect, card), why in sorted(ALLOWLIST.items()):
            print(f"\n  allowed: {effect} on {card} — {why}")
        return 0
    for effect, ref, path, line, card in unexpected:
        print(f"NEW: {effect} <- {ref} on {card} ({path}:{line})", file=sys.stderr)
    for effect, card in stale:
        print(f"STALE: allowlist keeps {effect} on {card}, site is gone", file=sys.stderr)
    return 1 if unexpected or stale else 0


if __name__ == "__main__":
    sys.exit(main())
