#!/usr/bin/env python3
"""Card names cited in an ENGINE doc comment that the catalog does not ship.

The definition enums (`StaticEffect`, `Effect`, `Keyword`, `Predicate`, …)
document nearly every variant with the card it was written for. That example
is a **claim about the catalog**, and it is the one claim in the repo that
nothing checks: no ratchet reads prose, the compiler does not care, and a
doc comment naming a card the engine has never seen looks exactly like one
naming a card it ships.

Two ways it goes wrong, and the second is the expensive one:

1. **It reads as "implemented".** A run looking for gaps greps the engine,
   finds `StaticEffect::ControllerDrawsDoubled` documented as "Thought
   Reflection, Alhammarret's Archive", and crosses the Archive off. It was
   never in the catalog.
2. ⚠ **It pre-approves an approximation.** `DoubleDamageDealt` carried
   "Fiery Emancipation as ×2 stacking" — a note, written before the card
   existed, authorising half of a card whose entire printed text is the
   multiplier. Whoever implemented it later would have found the decision
   already made, in a doc comment, by nobody.

Both were real, both on this branch, and both were found by hand while
implementing the cards. This is the census that finds the rest.

**Not every hit is a defect.** A doc may cite a card as the *canonical*
printing of a rule the engine models from a different card, or as a card the
variant deliberately does not cover. Those belong in `ALLOW` with the reason.
What a hit always means is: the name is not in the catalog, so the reader of
that doc cannot check it against anything.

Run: `python3 scripts/audit_doc_card_names.py`
"""

import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crabomination_catalog/src"
# The files whose doc comments are read: the definition enums an engine
# reader treats as the index of what is modelled.
DEFS = [
    "crabomination_base/src/effect/abilities.rs",
    "crabomination_base/src/effect.rs",
    "crabomination_base/src/card.rs",
]
DOC = re.compile(r"^[ \t]*///(.*)$", re.M)

# card name -> why the citation is fine although the card is not in the catalog.
# Keep this to reasons a gate cannot express.
ALLOW: dict[str, str] = {}

# ⚠⚠ **THE GATE IS THE SECOND LIST, NOT THE FIRST.** Citing an absent card is
# usually harmless design rationale ("every printed instance of the shape"),
# and 29 such rows would be noise. What is never harmless is a doc block that
# names an absent card **and** writes down an approximation for it: that is a
# decision about a card, made before the card existed, by nobody, and the next
# run finds it already taken. `DoubleDamageDealt` carried "Fiery Emancipation
# as ×2 stacking" for a card whose entire printed text is ×3.
#
# So the first list prints and the second list fails.
APPROX = re.compile(
    r"approximat|as ×\d|as x\d|stand-in|close enough|not modelled|unmodelled"
    r"|simplif|rather than the printed|does not model",
    re.I,
)


def shipped_names(cache_names):
    """Every cached card name that appears as a string literal in the catalog.

    ⚠ The literal, not `name: "…"`: a card built through a per-file
    `equipment()` / `creature()` helper passes its name as an **argument**, so
    a `name:`-keyed search reports every one of them as missing.
    """
    src = "\n".join(p.read_text() for p in CATALOG.rglob("*.rs"))
    return {n for n in cache_names if f'"{n}"' in src}


def doc_block(lines, ln):
    """The whole `///` run the 1-based line `ln` belongs to.

    The approximation and the card name are rarely on the same line — "Fiery
    Emancipation as ×2 stacking" was, but only because the doc was short.
    """
    i = ln - 1
    start, end = i, i
    while start > 0 and lines[start - 1].strip().startswith("///"):
        start -= 1
    while end + 1 < len(lines) and lines[end + 1].strip().startswith("///"):
        end += 1
    return "\n".join(lines[start : end + 1])


def main():
    cache = json.loads((ROOT / "scripts/.scryfall_cache.json").read_text())
    names = [n for n, v in cache.items() if isinstance(v, dict)]
    shipped = shipped_names(names)

    # ⚠ **A shorter card name is a prefix of a longer one more often than you
    # would guess, and both directions bite.** "Elesh Norn" cited while "Elesh
    # Norn, Grand Cenobite" ships is the shipped card under its short name;
    # "Battle Cry" is matched inside "Battle Cry Goblin", which also ships.
    # Every word-boundary prefix of a shipped name is therefore not a gap.
    covered = set()
    for s in shipped:
        head = s.split(",")[0]
        covered.add(head)
        words = head.split()
        for i in range(1, len(words)):
            covered.add(" ".join(words[:i]))

    # Only multi-word names long enough not to collide with ordinary prose or a
    # keyword ("Battle Cry", "Training Grounds" are both).
    candidates = [
        n
        for n in names
        if len(n) >= 8 and " " in n and n not in shipped and n not in covered
    ]
    by_first_word: dict[str, list[str]] = {}
    for n in candidates:
        by_first_word.setdefault(n.split()[0], []).append(n)

    hits: dict[str, list[tuple[str, int]]] = {}
    blocks: dict[tuple[str, int], str] = {}
    for f in DEFS:
        text = (ROOT / f).read_text()
        stem = f.split("/")[-1]
        lines = text.splitlines()
        for m in DOC.finditer(text):
            line = m.group(1)
            words = set(re.findall(r"[A-Z][\w'’-]*", line))
            for w in words:
                for n in by_first_word.get(w, ()):
                    # ⚠ A word boundary after the name, or "Mind Rake" matches
                    # inside "Mind Raker's discard" and reports a card the doc
                    # never mentioned.
                    if re.search(rf"{re.escape(n)}(?![\w'’-])", line):
                        ln = text[: m.start()].count("\n") + 1
                        hits.setdefault(n, []).append((stem, ln))
                        blocks[(stem, ln)] = doc_block(lines, ln)

    rows = sorted((n, w) for n, w in hits.items() if n not in ALLOW)
    allowed = sorted(n for n in hits if n in ALLOW)
    for name, where in rows:
        stem, ln = where[0]
        extra = f"  (+{len(where) - 1} more)" if len(where) > 1 else ""
        print(f"- {name}  [{stem}:{ln}]{extra}")
    stale = sorted(set(ALLOW) - set(allowed))
    for name in stale:
        print(f"STALE allowlist entry, the card now ships: {name}")

    # The gate: an absent card whose doc block also writes down an
    # approximation for it.
    preapproved = []
    for name, where in sorted(hits.items()):
        for stem, ln in where:
            block = blocks.get((stem, ln), "")
            if APPROX.search(block):
                preapproved.append((name, stem, ln, block))
                break
    for name, stem, ln, block in preapproved:
        print(f"\n! {name}  [{stem}:{ln}] — absent from the catalog, and this doc")
        print("  already writes down an approximation for it:")
        for line in block.splitlines():
            if APPROX.search(line):
                print("     ", line.strip()[:160])

    print(
        f"\n{len(rows) + len(allowed)} card names cited in an engine doc comment "
        f"and absent from the catalog, {len(allowed)} allowlisted, {len(rows)} "
        f"unexplained, {len(stale)} stale — **{len(preapproved)} of them with an "
        f"APPROXIMATION already written down**, which is the gate"
    )
    return 1 if stale or preapproved else 0


if __name__ == "__main__":
    sys.exit(main())
