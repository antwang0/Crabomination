#!/usr/bin/env python3
"""Per-deck residuals of the Commander pod decks, read off the card docs.

For every `PodDeck` in `crabomination/src/pod/mod.rs::target_decks`, resolve
its `*_COMMANDERS` / `*_MAIN` factory lists (`pod/decks.rs`) to the catalog
factory and read that factory's `///` doc comment. A card whose doc names a
gap — "Residual", "Approximation", "approximated", "not modelled", "is the
engine's pick" — counts against its deck; a bare "⚠" marks history, not a
gap. The doc is the source the trackers summarise, so this keeps
DECK_FEATURES' deck table honest; read the flagged doc before believing it
(`audit_incomplete` found docs ~30% stale).

  scripts/pod_residuals.py             # per-deck counts, complete decks too
  scripts/pod_residuals.py --cards     # every flagged card with its doc line
  scripts/pod_residuals.py --summary   # one line: decks complete / partial
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MARKERS = re.compile(
    r"residual|approximat|not modell?ed|engine's pick|engine picks|isn't modell?ed|is dropped|are dropped",
    re.I,
)


def factory_docs():
    """fn name -> doc comment text, over the whole catalog."""
    docs = {}
    for f in (ROOT / "crabomination_catalog" / "src").rglob("*.rs"):
        lines = f.read_text(encoding="utf-8").splitlines()
        for i, line in enumerate(lines):
            m = re.match(r"\s*pub fn ([a-z0-9_]+)\(\) -> CardDefinition", line)
            if not m:
                continue
            j = i - 1
            doc = []
            while j >= 0 and (lines[j].strip().startswith("///") or lines[j].strip().startswith("#[")):
                if lines[j].strip().startswith("///"):
                    doc.append(lines[j].strip()[3:].strip())
                j -= 1
            docs.setdefault(m.group(1), " ".join(reversed(doc)))
    return docs


def deck_lists():
    src = (ROOT / "crabomination" / "src" / "pod" / "decks.rs").read_text(encoding="utf-8")
    consts = {}
    for m in re.finditer(r"pub const ([A-Z0-9_]+): &\[CardFactory\] =\s*&\[(.*?)\];", src, re.S):
        body = re.sub(r"//[^\n]*", "", m.group(2))
        consts[m.group(1)] = [t.strip() for t in body.split(",") if t.strip()]
    mod = (ROOT / "crabomination" / "src" / "pod" / "mod.rs").read_text(encoding="utf-8")
    body = mod[mod.index("pub fn target_decks"):]
    decks = []
    for m in re.finditer(
        r'PodDeck\s*\{\s*name:\s*"((?:[^"\\]|\\.)+)",\s*commanders:\s*decks::([A-Z0-9_]+),\s*main:\s*decks::([A-Z0-9_]+)',
        body,
    ):
        decks.append((m.group(1).replace('\\"', '"'), consts.get(m.group(2), []) + consts.get(m.group(3), [])))
    return decks


def main():
    docs = factory_docs()
    decks = deck_lists()
    show_cards = "--cards" in sys.argv
    complete = 0
    for i, (name, cards) in enumerate(decks, 1):
        flagged = sorted({c for c in cards if MARKERS.search(docs.get(c, ""))})
        complete += not flagged
        if "--summary" in sys.argv:
            continue
        print(f"{i:3} {name}: {len(cards)} cards, {len(flagged)} with residuals")
        if show_cards:
            for c in flagged:
                hit = MARKERS.search(docs[c])
                lo = max(0, hit.start() - 60)
                print(f"      {c}: …{docs[c][lo:hit.end() + 100]}…")
    print(f"{complete} / {len(decks)} pod decks carry no residual in their card docs")


if __name__ == "__main__":
    main()
