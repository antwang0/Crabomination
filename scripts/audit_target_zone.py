#!/usr/bin/env python3
"""Targeted clauses that name a source ZONE whose filter does not.

`SelectionRequirement` has no implicit zone: `legal_targets_for_filter`
applies the same requirement to every zone an effect can reach, so
`target_filtered(SelectionRequirement::Creature)` on a "put target creature
card **from your graveyard** on top of your library" is satisfied by a
creature on the *battlefield* — and `auto_target_for_effect` prefers one,
because the battlefield is walked first. Mortuary Mire tucked live creatures
that way for as long as it shipped; Cremate exiled permanents off the
battlefield for {B}.

The filter language says the zone with `from_your_graveyard()` /
`from_any_graveyard()` / `InYourGraveyard` / `InOpponentGraveyard`, or the
effect variant carries it in its own name. This audit is the census of
definitions that carry neither while the oracle's *target* clause names a
graveyard.

    python3 scripts/audit_target_zone.py           # ranked list
    python3 scripts/audit_target_zone.py --count   # just the total
    python3 scripts/audit_target_zone.py --gate    # exit 1 on any row

⚠ The finding is the **target's source zone**, not the word "graveyard".
"Target player exiles a card from their graveyard" targets a *player*, and
"whenever target creature is put into your graveyard" names a destination —
neither constrains a filter. Both are skipped, and the skips are the part of
this script to re-read before widening it.

Reads `scripts/.scryfall_cache.json` (offline; `scripts/fetch_oracle.py`
fills it) and reuses `audit_dropped_may`'s body reader, which inlines the
same-file helpers a factory builds its abilities through.
"""

import importlib.util
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crabomination_catalog", "src")
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")

_spec = importlib.util.spec_from_file_location(
    "audit_dropped_may", os.path.join(ROOT, "scripts", "audit_dropped_may.py")
)
_adm = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_adm)

# Every way a definition can name a zone — the filter combinators, the
# `SelectionRequirement` atoms they expand to, and the effect variants that
# carry the zone in their own name (`ReturnFromGraveyard`, `ZoneDest::Exile`,
# …). One case-insensitive substring per zone covers the lot, which is what
# keeps this audit from going stale on a new variant.
#
# The graveyard column is the big one (27 rows when it was written); exile is
# the same defect and a much rarer wording. Hand and library were measured and
# are empty — a "target card in your hand" clause is vanishingly rare and
# every implemented one already says so.
ZONES = {
    "graveyard": ("graveyard", "Graveyard"),
    "exile": ("exile", "Exile"),
}

# The zone the ENGINE owns rather than the definition. A bespoke effect whose
# arm builds its own candidate list out of `players[*].graveyard` cannot be
# aimed at a battlefield permanent however its filter reads, so the finding is
# the reader's. Conditioned on the arm, re-checked every run: if the arm stops
# naming the zone the exemption is refused on stderr and the row comes back.
ENGINE_EFFECTS = os.path.join(ROOT, "crabomination", "src", "game", "effects", "mod.rs")
RESOLVER_ZONE = {
    # Effect variant: the token its arm must still carry
    "CommandTheDreadhorde": "graveyard",
    "WeldArtifacts": "graveyard",
}

# A definition with no targeted selector at all has not got the clause WRONG,
# it has not got the clause — which is `audit_incomplete`'s column, not this
# one. Reported separately so `--gate` means "every modelled clause names its
# zone".
TARGET_TOKENS = ("target_filtered", "TargetFiltered", "Selector::Target")


def resolver_zone():
    """`{variant}` for each exemption the engine still earns."""
    src = open(ENGINE_EFFECTS, encoding="utf-8").read()
    out = set()
    for variant, token in RESOLVER_ZONE.items():
        m = re.search(r"Effect::%s\s*(?:=>|[{(])" % re.escape(variant), src)
        if not m:
            print(f"# exemption REFUSED: no `Effect::{variant}` arm", file=sys.stderr)
            continue
        if token not in src[m.end() : m.end() + 4000]:
            print(
                f"# exemption REFUSED: `Effect::{variant}`'s arm no longer names "
                f"a {token}",
                file=sys.stderr,
            )
            continue
        out.add(variant)
    return out

# A target clause runs to the end of its sentence. Only the part before the
# effect's *destination* can name the target's source zone, so a clause is cut
# at the first "to your hand" / "onto the battlefield" / "into … graveyard".
DESTINATION = re.compile(r"\b(to|onto|into|on top of|on the bottom of)\b")

# The target is a PLAYER — their graveyard is what the effect reaches, not
# what the filter has to name.
PLAYER_TARGET = re.compile(r"^target (player|opponent|creature's controller)\b")


def source_spans(oracle, zone):
    """The oracle's target clauses whose TARGET sits in `zone`."""
    out = []
    for span in re.findall(r"target [^.;\n]*", oracle):
        if PLAYER_TARGET.match(span):
            continue
        head = span
        m = DESTINATION.search(span)
        if m:
            head = span[: m.start()]
        if zone not in head:
            continue
        # "target card from a graveyard", "target creature card in your
        # graveyard" — the noun is a *card*, which is what a zone-bearing
        # filter selects. "target creature ... graveyard" with no "card" is a
        # battlefield target with a graveyard rider (Saffi Eriksdotter), and
        # "becomes the target of a spell … exile the top card of your library"
        # names no source zone at all.
        if "card" not in head:
            continue
        out.append(span)
    return out


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    lower = {k.lower(): v for k, v in cache.items() if isinstance(v, dict)}
    for k, v in list(cache.items()):
        if isinstance(v, dict):
            lower.setdefault(k.split(" // ")[0].lower(), v)

    exempt_variants = resolver_zone()
    hits, unmodelled, checked, uncached, exempted = [], [], 0, 0, 0
    for dirpath, _, files in os.walk(CATALOG):
        for f in files:
            if not f.endswith(".rs"):
                continue
            for fn, name, body, path in _adm.defs_in(os.path.join(dirpath, f)):
                card = lower.get(name.lower())
                if card is None:
                    uncached += 1
                    continue
                oracle = re.sub(r"\([^)]*\)", " ", card.get("oracle_text") or "").lower()
                # A clause can name two ("from your graveyard **or** exiled
                # card with flashback you own" — Sorceress's Schemes), and a
                # body that says one of them is not a body that says both.
                named = [z for z in ZONES if source_spans(oracle, z)]
                if not named:
                    continue
                checked += 1
                missing = [
                    z for z in named if not any(tok in body for tok in ZONES[z])
                ]
                if not missing:
                    continue
                zone = missing[0]
                spans = source_spans(oracle, zone)
                if any(f"Effect::{v}" in body for v in exempt_variants):
                    exempted += 1
                    continue
                row = (name, fn, os.path.relpath(path, ROOT), f"[{zone}] {spans[0][:90]}")
                if any(tok in body for tok in TARGET_TOKENS):
                    hits.append(row)
                else:
                    unmodelled.append(row)

    hits.sort()
    unmodelled.sort()
    if "--count" not in sys.argv:
        for name, fn, path, span in hits:
            print(f"{path}::{fn}\n    {name}: “{span}”")
        if unmodelled:
            print("\n# clause not modelled at all — `audit_incomplete`'s column:")
            for name, fn, path, span in unmodelled:
                print(f"{path}::{fn}\n    {name}: “{span}”")
    print(
        f"# {len(hits)} modelled clauses that name a zone and whose filter does "
        f"not, {len(unmodelled)} whose clause is not modelled at all, {checked} with "
        f"such a clause, {uncached} skipped as uncached, {exempted} exempt because "
        f"the RESOLVER owns the zone "
        f"({', '.join(sorted(exempt_variants)) or 'none'})"
    )
    if "--gate" in sys.argv and hits:
        sys.exit(1)


if __name__ == "__main__":
    main()
