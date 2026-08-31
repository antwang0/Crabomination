#!/usr/bin/env python3
"""Does a card's effect tree do the VERBS its oracle text says?

Every printed *characteristic* in this catalog is cross-referenced against
`scripts/.scryfall_cache.json` by `audit_catalog_stats.py` — cost, P/T, card
types, supertypes, creature subtypes, keywords — and as of 2026-08-30 all of
those columns read zero. **Nothing had ever checked what a card DOES.** Two
wrong-shape cards fell out of the keyword pass sideways (Tempest Angler ships
an ETB scry for a printed "+1/+1 counter on noncreature cast"; Outcaster
Trailblazer ships a mana-value-5 draw for a printed ETB mana and a power-4
draw), and neither was visible to any column there.

This is the cheapest question that catches that class: **the oracle names a
verb and the effect tree has no primitive for it.** It is a *presence* check,
not a semantics one — "draws a card" against `Effect::Draw` — so it cannot see
a card that draws the wrong number or draws for the wrong player. It can see a
card that does not draw at all.

**Standing at 172 rows over 17,028 cards with oracle text (2026-08-31), and
the rate of real findings is high.** Six classes were spot-checked in the
source: Codespell Cleric is a body-only stub (its whole ETB counter ability is
absent), Fountain of Renewal drops its sac-for-a-card, Baral drops the trigger
that draws, Crawl from the Cellar drops its Zombie counter, Master of Death
drops its ETB surveil 2, Teferi drops "untap up to two lands". The
`counter_spell` rows are the ward-shaped approximation this catalog documents
and are the only class checked that was not real.

    python3 scripts/audit_oracle_verbs.py             # per-verb summary
    python3 scripts/audit_oracle_verbs.py draw        # the rows for one verb
    python3 scripts/audit_oracle_verbs.py --check     # exit 1 on any finding

**Reminder text is stripped before the oracle is read.** Every keyword's
reminder is a sentence full of verbs the card does not have — battle cry's
"each other attacking creature gets +1/+0", ripple's "you may cast spells…",
graft's "whenever another creature enters" — and leaving them in makes the
filter useless.

**A helper hides the effect it builds, so helpers are expanded one level.**
`crabomination_base/src/effect/shortcut.rs` has 227 of them and every set file
has its own; a card that calls `deal(2, target_any())` names no
`Effect::DealDamage` anywhere in its own body. The tables below are built by
scanning each helper's body for the variants it constructs, then attributing
those to any card that calls it.

**AND SO DOES AN ENGINE PRIMITIVE, WHICH IS WHERE MOST OF THE NOISE WAS.** A
one-card `Effect` variant is a helper on the far side of the crate boundary:
`Effect::MillDeployCreaturesUntilEndStep` (Random Encounter) and
`Effect::UnexpectedResults` both return a card to its owner's hand, and both
say so only inside `run_effect`. `primitive_bodies()` brace-matches the 933
`Effect::X => {` arms under `crabomination/src/game/effects/` and attributes
what each names — including bare `*_to_hand` / `*_to_graveyard` identifiers,
because the engine spells some verbs as a `GameState` flag rather than an
effect. **A destination is not an `Effect` variant either**: `Effect::Move {
to: ZoneDest::Hand(..) }` recorded only `Move`, so `DEST` now collects
`ZoneDest::` / `Zone::` / `CounteredSpellZone::` names in both tables.

**AND MOST OFTEN IT SPELLS THE VERB AS A `GameState` METHOD.** `Effect::Parley`'s
arm ends in `self.draw_one(p, events)` for every seat and names no `Effect::`
variant at all, so all five Parley cards read as missing the draw their oracle
prints. Every `self.<method>(` an arm calls is now attributed to the variant,
and the verb table names the lowercase engine spellings it cares about
(`draw_one`, `gain_life`, `deal_damage`, `destroy_target`, …). Worth 15 rows
across five verbs: `draw` 32 -> 23, `gain_life` 30 -> 27, `damage` 25 -> 23,
`untap` 3 -> 2.

**AND `shortcut.rs` IS NOT THE ONLY BASE-CRATE HELPER FILE.** Every
engine-baked token lives in `crabomination_base/src/tokens.rs`, and a token's
own triggered ability is the verb: twenty-odd STX cards mint
`stx_pest_token()`, whose "when this token dies, you gain 1 life" is a
`TriggeredAbility` inside that function and invisible to a scan of the set
file. Both files feed the helper table now — `gain_life` 27 -> 19.

**AND A HELPER DOES NOT HAVE TO LIVE IN THE CALLER'S FILE.** `painland()` is
in `crabomination_catalog/src/sets/mod.rs` and every set with a `shared.rs`
puts its cycle builders there, but the table below was built **per file**.
There is now a global one over every set file as well — worth 10 rows across
five verbs, and it is why a run takes ~4.5 min rather than ~1.

**Worked example of what that is worth: `return_to_hand` went 31 rows -> 5,
and all five of the survivors are real** (2026-08-31). The 26 that went were
four distinct false classes — the cost spellings (`bounce_other_filter`,
`return_self_cost`, the alternative-cost `return_to_hand`: the six Soratami,
Quirion Ranger, Wirewood Symbiote, Daze, Gush, Chthonian Nightmare), the
destination-in-a-helper class, the engine-primitive class, and the variants
the old `ReturnToHand|Bounce|Hand\b` regex simply did not name
(`ReturnSelf`, `ReturnEachUnlessPays`, `CounteredSpellZone::OwnerHand`,
`StaticEffect::DiesToOwnersHandInstead`). **Before working a verb, read six
of its rows in the source and fix the filter first** — the class of a false
positive is shared, so six rows find nearly all of them, and the same three
mechanisms took the whole file from 199 rows to 163.

The reading of a card (its body, its name, which `CardDefinition` literal is
the card's) is `audit_catalog_stats`'s, imported rather than re-derived —
four separate bugs came out of that reader on 2026-08-30 and a second copy
would have all four back.
"""

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from audit_catalog_stats import (  # noqa: E402
    CACHE_LC,
    FUNC,
    SETS,
    card_def_name,
    face_for,
    helper_table,
    inline_helper_call,
    set_of,
    strip_token_literals,
)

BASE = Path(__file__).resolve().parent.parent / "crabomination_base" / "src"

# ── The verb table ──────────────────────────────────────────────────────────
#
# Each entry is (oracle regex, code regex). The code regex is matched against
# the card's own body **plus the names of every `Effect` / `StaticEffect`
# variant its helpers construct**, so a card that calls `deal(2, target_any())`
# still answers for `DealDamage`.
#
# Deliberately conservative, and matched on the variant *name* rather than an
# enumerated set: the engine spells one verb many ways (`TapUpToValue`,
# `UntapAllYoursEachUntapStep`, `CounterUnlessPaid`) and an exhaustive list
# goes stale the first time somebody adds a primitive. A false negative costs
# nothing; a false positive costs a reader's time.
VERBS = {
    "draw": (
        r"\bdraws? (?:a card|\w+ cards|that many cards|cards equal)",
        r"Draw|Miracle|draw_one|draw_cards",
    ),
    # `opponent_gains_life` is the **cost** spelling —
    # "rather than pay this spell's mana cost, you may have an opponent gain
    # 3 life" (the Nemesis/Mercadian free spells: Invigorate, Reverent
    # Silence, Skyshroud Cutter). Gaining life as the price of casting is what
    # the oracle sentence says, so a card that spells it in `AlternativeCost`
    # is not missing the verb — the same rule `return_to_hand` wrote for
    # `return_self_cost`.
    "gain_life": (
        r"\bgains? (?:\d+|X|that much|twice) life\b",
        r"GainLife|Drain|SetLife|Lifelink|LifeGain|gain_life"
        r"|opponent_gains_life",
    ),
    "lose_life": (
        r"\bloses? (?:\d+|X|that much) life\b",
        r"LoseLife|LoseHalfLife|Drain|SetLife|PayLife|DealDamage|lose_life",
    ),
    "damage": (
        r"\bdeals \d+ damage\b|\bdeals X damage\b|\bdeals that much damage\b",
        r"Damage|Fight|deal_damage",
    ),
    "destroy": (
        r"\bdestroys? (?:target|all|each|up to|that)\b",
        r"Destroy|Sacrifice|Exile|destroy_permanent|destroy_target",
    ),
    "counter_spell": (
        r"\bcounters? (?:target|that) spell\b",
        r"Counter(?:Spell|Target|Unless|All|Top)|Effect::Counter\b",
    ),
    "token": (
        r"\bcreates? (?:a|an|two|three|four|five|X|\d+)\b[^.]{0,80}\btoken",
        r"Token|create_token",
    ),
    "scry": (r"\bscry \d", r"Scry"),
    "surveil": (r"\bsurveil \d", r"Surveil"),
    "mill": (r"\bmills? (?:\w+ cards?|that many)", r"Mill|Dredge"),
    "search_library": (
        r"\bsearch(?:es)? (?:your|their) library\b",
        r"Search|Tutor|Fetch|Zone::Library",
    ),
    # `Hand\b` was the whole variant side and it matched almost nothing a
    # bounce card actually writes: the nine `Return…Hand` variants, yes, but
    # not `ReturnSelf` (Field of Reality), not `ReturnEachUnlessPays` (Cut the
    # Tethers), and none of the **cost** spellings — `bounce_other_filter`
    # (the six Soratami, Quirion Ranger, Wirewood Symbiote),
    # `return_self_cost` (Chthonian Nightmare), `return_permanent_cost` (the
    # Tempest "return this enchantment: do X" cyclers — Attunement, Broken
    # Fall), the alternative-cost `return_to_hand` (Daze, Gush). Returning something to hand as the price
    # of an ability is what the oracle sentence says, so a card that spells it
    # in a cost field is not missing the verb. **`ReturnSelf` is NOT on the
    # list** — the whole `ReturnSelf*` family returns from the GRAVEYARD to the
    # BATTLEFIELD, and putting it here on the strength of its name hid the
    # bug it was covering for (Field of Reality's `{1}{U}` bounce was
    # `Effect::ReturnSelf`, a no-op for a permanent that is not in a
    # graveyard). `ReturnEachUnlessPays` is on it because its arm delegates to
    # `run_each_unless_pays(.., sacrifice: false)`, which bounces.
    "return_to_hand": (
        r"\breturns? [^.]{0,60}\bto (?:its|their) owner'?s? hand\b",
        r"To\w{0,12}Hand|ReturnEachUnlessPays|Bounce|bounce_"
        r"|ZoneDest::Hand|Zone::Hand|CounteredSpellZone::OwnerHand"
        r"|ExileReturnZone::Hand"
        r"|return_self_cost|_to_hand|return_to_hand|return_eot"
        r"|return_permanent_cost",
    ),
    "counters": (
        r"\+1/\+1 counter|\-1/\-1 counter",
        # `Counters\b` (plural) is what catches the *doublers*
        # (`ExtraPlusOneCounters`, `DoublePlusOneCounters`) without also
        # catching `CounterUnlessPaid`, which is the other verb entirely.
        r"CounterType::|AddCounter|RemoveCounter|Counters\b|counters\b|Endure|Adapt"
        r"|Bolster|Monstrosity|Proliferate|Amplify|Graft|Modular|Evolve|Bloodthirst"
        r"|Outlast|Renown|Mentor|Undying|Persist|Fabricate|Reinforce|Scavenge"
        r"|Earthbend|Incubate|Support|Training|Backup",
    ),
    "tap_target": (
        r"\btaps? (?:target|up to|all|each)\b",
        r"Tap",
    ),
    "untap": (
        r"\buntaps? (?:target|all|each|up to)\b",
        r"Untap|untap_permanent",
    ),
}

# `Effect::Foo` — the constructed variant, not a `match` arm or a doc mention.
CTOR = re.compile(r"\bEffect::(\w+)")
STATIC = re.compile(r"\bStaticEffect::(\w+)")
# A helper that *moves* a card names its destination, not an `Effect` variant:
# `Effect::Move { to: ZoneDest::Hand(..) }` recorded only `Move`, so
# `return_target_creature_to_hand` (Sweep Away) and `returns_to_hand`
# (Spreading Algae) answered nothing for the bounce verb and both read as
# missing it. `ExileReturnZone` is the same thing one step later: Ashiok's
# Erasure stamps the exile with `return_to: ExileReturnZone::Hand` and the
# leaves-the-battlefield return is the engine's, not the card's. Qualified,
# so a code regex still has to say which zone.
DEST = re.compile(r"\b((?:ZoneDest|CounteredSpellZone|ExileReturnZone|Zone)::\w+)")


ENGINE = (
    Path(__file__).resolve().parent.parent / "crabomination" / "src" / "game" / "effects"
)
ARM = re.compile(r"\n\s+Effect::(\w+)[^\n]*=>\s*\{")


def primitive_bodies():
    """`{variant: {tokens its `run_effect` arm names}}`.

    A one-card `Effect` variant is a helper the catalog cannot see into:
    `MillDeployCreaturesUntilEndStep` (Random Encounter) and
    `UnexpectedResults` both return a card to its owner's hand, and both said
    so only in the engine. Same brace-match as `helper_variants`, over the
    arms of `run_effect`; a variant with several arms gets their union.
    """
    out = {}
    for src in sorted(ENGINE.rglob("*.rs")):
        text = src.read_text()
        for m in ARM.finditer(text):
            i = text.rindex("{", m.start(), m.end())
            depth, j = 1, i + 1
            while j < len(text) and depth:
                if text[j] == "{":
                    depth += 1
                elif text[j] == "}":
                    depth -= 1
                j += 1
            body = text[i:j]
            out.setdefault(m.group(1), set()).update(
                set(CTOR.findall(body)) | set(STATIC.findall(body)) | set(DEST.findall(body))
                # The engine spells some verbs as a `GameState` flag rather
                # than an effect (`return_resolving_spell_to_hand`), so the
                # bare identifiers count too.
                | set(re.findall(r"\b(?:self\.)?(\w*(?:to_hand|to_graveyard)\w*)\b", body))
                # **And most of them it spells as a `GameState` METHOD**, which
                # is the same blindness one step further out: `Effect::Parley`'s
                # arm ends in `self.draw_one(p, events)` for every seat and
                # names no `Effect::` variant at all, so all five Parley cards
                # read as missing the draw their oracle prints. Take every
                # method the arm calls; the verb table's own regexes decide
                # which of them mean anything, and nearly all of those are
                # capitalised so a lowercase method name is inert unless a
                # verb asks for it by name.
                | set(re.findall(r"\bself\.(\w+)\s*\(", body))
            )
    return out


def helper_variants(text):
    """`{fn_name: {Effect variants it constructs}}` for one source file.

    One level: a helper that calls another helper contributes what it names
    directly. Resolved transitively below by iterating to a fixed point.
    """
    out = {}
    for m in re.finditer(r"\nfn (\w+)|\npub fn (\w+)|\npub\(crate\) fn (\w+)", text):
        name = m.group(1) or m.group(2) or m.group(3)
        i = text.find("{", m.end())
        if i < 0:
            continue
        depth, j = 1, i + 1
        while j < len(text) and depth:
            if text[j] == "{":
                depth += 1
            elif text[j] == "}":
                depth -= 1
            j += 1
        body = text[i:j]
        out[name] = set(CTOR.findall(body)) | set(STATIC.findall(body)) | set(
            DEST.findall(body)
        ) | {
            f"@{c}" for c in re.findall(r"\b(\w+)\s*\(", body)
        }
    return out


def close_helpers(tables):
    """Fixed-point expansion: a helper that calls a helper gets its variants."""
    merged = {}
    for t in tables:
        for k, v in t.items():
            merged.setdefault(k, set()).update(v)
    for _ in range(4):
        changed = False
        for k, v in merged.items():
            add = set()
            for c in [x[1:] for x in v if x.startswith("@")]:
                if c in merged and c != k:
                    add |= {x for x in merged[c] if not x.startswith("@")}
                    add |= {x for x in merged[c] if x.startswith("@")}
            if not add <= v:
                v |= add
                changed = True
        if not changed:
            break
    return {k: {x for x in v if not x.startswith("@")} | v for k, v in merged.items()}


REMINDER = re.compile(r"\([^()]*\)")


def oracle_text(card, face):
    """The card's rules text with reminder parentheticals stripped."""
    parts = []
    if face is not None:
        parts.append(face.get("oracle_text") or "")
    else:
        parts.append(card.get("oracle_text") or "")
        for f in card.get("card_faces") or []:
            parts.append(f.get("oracle_text") or "")
    txt = "\n".join(p for p in parts if p)
    prev = None
    while prev != txt:
        prev, txt = txt, REMINDER.sub(" ", txt)
    return txt.lower()


def audit():
    # `shortcut.rs` is not the only base-crate helper file a card body reaches:
    # every engine-baked token lives in `tokens.rs`, and a token's own
    # triggered ability is the verb. Twenty-odd STX cards mint
    # `stx_pest_token()`, whose "when this token dies, you gain 1 life" is a
    # `TriggeredAbility` inside that function and invisible to a scan of the
    # set file. Read both.
    shortcuts = helper_variants(
        (BASE / "effect" / "shortcut.rs").read_text() + "\n" + (BASE / "tokens.rs").read_text()
    )
    # **And a helper does not have to live in the caller's file.** `painland()`
    # (five Tempest lands + the whole cycle elsewhere) is in
    # `crabomination_catalog/src/sets/mod.rs`, and every set with a `shared.rs`
    # puts its cycle builders there. The per-file table below cannot see either,
    # so build one global table over every set file as well. A name collision
    # between two sets' private helpers over-attributes, which costs a false
    # negative — the trade this file states it wants.
    catalog_wide = helper_variants(
        "\n".join(p.read_text() for p in sorted(SETS.rglob("*.rs")))
    )
    prims = primitive_bodies()
    findings = {v: [] for v in VERBS}
    checked = 0
    for src in sorted(SETS.rglob("*.rs")):
        text = src.read_text()
        local = helper_variants(text)
        table = close_helpers([shortcuts, catalog_wide, local])
        # `..creature("Viashino Sandswimmer", …)` carries the card's *name* in
        # a positional argument, so `card_def_name` read nothing and the card
        # was skipped outright — most of the classic sets. `audit_catalog_stats`
        # already solves this; use its splice rather than a second one.
        helpers, hconsts = helper_table(text)
        for m in FUNC.finditer(text):
            nxt = FUNC.search(text, m.end())
            body = text[m.end() : nxt.start() if nxt else len(text)]
            cut = re.search(r"\n(?:pub(?:\([^)]*\))?\s+)?fn \w+", body)
            if cut:
                body = body[: cut.start()]
            raw = body
            body = strip_token_literals(body)
            body = inline_helper_call(body, helpers, hconsts)
            nm = card_def_name(body)
            if not nm:
                continue
            card = CACHE_LC.get(nm.group(1).lower())
            if not card:
                continue
            face = face_for(card, nm.group(1))
            txt = oracle_text(card, face)
            if not txt:
                continue
            checked += 1
            # The *raw* body: `strip_token_literals` blanks the very literal a
            # "create a token" card is answered by.
            have = set()
            for call in re.findall(r"\b(\w+)\s*\(", raw):
                have |= table.get(call, set())
            for v in set(CTOR.findall(raw)) | have:
                have |= prims.get(v, set())
            hay = raw + " " + " ".join(sorted(have))
            for verb, (pat, code) in VERBS.items():
                if re.search(pat, txt) and not re.search(code, hay):
                    findings[verb].append(
                        (nm.group(1), src.name, m.group(1), set_of(src))
                    )
    return findings, checked


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    check = "--check" in sys.argv
    findings, checked = audit()
    if args:
        v = args[0]
        if v not in findings:
            sys.exit(f"no such verb '{v}' (have: {', '.join(sorted(findings))})")
        print(f"=== {v}: oracle says it, no primitive in the tree ({len(findings[v])})")
        for name, fname, fn, st in sorted(findings[v]):
            print(f"  {name:38} [{st}] {fname}::{fn}")
    else:
        print(f"{'verb':<16}{'missing':>9}")
        print("-" * 25)
        for v in sorted(findings, key=lambda v: -len(findings[v])):
            print(f"{v:<16}{len(findings[v]):>9}")
        print("-" * 25)
        print(f"{'TOTAL':<16}{sum(len(f) for f in findings.values()):>9}"
              f"   over {checked:,} cards with oracle text")
        print("\nRows for one verb:  python3 scripts/audit_oracle_verbs.py <verb>")
    if check and any(findings.values()):
        sys.exit(1)


if __name__ == "__main__":
    main()
