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
        r"Draw|Miracle",
    ),
    "gain_life": (
        r"\bgains? (?:\d+|X|that much|twice) life\b",
        r"GainLife|Drain|SetLife|Lifelink|LifeGain",
    ),
    "lose_life": (
        r"\bloses? (?:\d+|X|that much) life\b",
        r"LoseLife|LoseHalfLife|Drain|SetLife|PayLife|DealDamage",
    ),
    "damage": (
        r"\bdeals \d+ damage\b|\bdeals X damage\b|\bdeals that much damage\b",
        r"Damage|Fight",
    ),
    "destroy": (
        r"\bdestroys? (?:target|all|each|up to|that)\b",
        r"Destroy|Sacrifice|Exile",
    ),
    "counter_spell": (
        r"\bcounters? (?:target|that) spell\b",
        r"Counter(?:Spell|Target|Unless|All|Top)|Effect::Counter\b",
    ),
    "token": (
        r"\bcreates? (?:a|an|two|three|four|five|X|\d+)\b[^.]{0,80}\btoken",
        r"Token",
    ),
    "scry": (r"\bscry \d", r"Scry"),
    "surveil": (r"\bsurveil \d", r"Surveil"),
    "mill": (r"\bmills? (?:\w+ cards?|that many)", r"Mill|Dredge"),
    "search_library": (
        r"\bsearch(?:es)? (?:your|their) library\b",
        r"Search|Tutor|Fetch|Zone::Library",
    ),
    "return_to_hand": (
        r"\breturns? [^.]{0,60}\bto (?:its|their) owner'?s? hand\b",
        r"ReturnToHand|Bounce|Hand\b|ZoneDest::Hand",
    ),
    "counters": (
        r"\+1/\+1 counter|\-1/\-1 counter",
        r"CounterType::|AddCounter|RemoveCounter|Endure|Adapt|Bolster|Monstrosity"
        r"|Proliferate|enters_with_counters|Amplify|Graft|Modular|Evolve|Bloodthirst",
    ),
    "tap_target": (
        r"\btaps? (?:target|up to|all|each)\b",
        r"Tap",
    ),
    "untap": (
        r"\buntaps? (?:target|all|each|up to)\b",
        r"Untap",
    ),
}

# `Effect::Foo` — the constructed variant, not a `match` arm or a doc mention.
CTOR = re.compile(r"\bEffect::(\w+)")
STATIC = re.compile(r"\bStaticEffect::(\w+)")


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
        out[name] = set(CTOR.findall(body)) | set(STATIC.findall(body)) | {
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
    shortcuts = helper_variants((BASE / "effect" / "shortcut.rs").read_text())
    findings = {v: [] for v in VERBS}
    checked = 0
    for src in sorted(SETS.rglob("*.rs")):
        text = src.read_text()
        local = helper_variants(text)
        table = close_helpers([shortcuts, local])
        for m in FUNC.finditer(text):
            nxt = FUNC.search(text, m.end())
            body = text[m.end() : nxt.start() if nxt else len(text)]
            cut = re.search(r"\n(?:pub(?:\([^)]*\))?\s+)?fn \w+", body)
            if cut:
                body = body[: cut.start()]
            raw = body
            body = strip_token_literals(body)
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
