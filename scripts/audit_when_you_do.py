#!/usr/bin/env python3
"""Printed "When you do, … target …" (CR 603.12 reflexive trigger) vs a
definition with no `Reflexive` / `ReflexiveTrigger` in its factory: the payoff
then targets as the outer trigger goes on the stack, before the cost or choice
that gates it. Ranked by pod seats. A card built through a shared helper that
holds the wrapper (`plan_payoff`) reads as a false positive — check the helper.

    python3 scripts/audit_when_you_do.py [--all]   # --all: untargeted payoffs too
"""

import glob
import json
import re
import sys
import unicodedata

ROOT = __file__.rsplit("/scripts/", 1)[0]
CACHE = json.load(open(f"{ROOT}/scripts/.scryfall_cache.json"))
# Built through a helper that holds the wrapper. A row here is not printed.
REVIEWED = {
    "Construct a Cosmic Cube": "plan_payoff",
    "Doom Reigns Supreme": "plan_payoff",
    "Villainous Syndication": "plan_payoff",
}
NAMED = re.compile(r'(?:name:\s*|\b(?:legend|creature|spell|permanent|plan|sorcery|instant)\(\s*)"([^"]+)"')


def snake(name):
    n = unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode().lower()
    return re.sub(r"[^a-z0-9]+", "_", n.replace("'", "").replace(",", "")).strip("_")


def factories():
    """`{card name: [factory body]}` over every catalog file."""
    out = {}
    for path in glob.glob(f"{ROOT}/crabomination_catalog/src/sets/**/*.rs", recursive=True):
        src = open(path, encoding="utf-8").read()
        cuts = [m.start() for m in re.finditer(r"\n(?:pub(?:\([a-z]+\))? )?fn ", src)] + [len(src)]
        for a, b in zip(cuts, cuts[1:]):
            body = src[a:b]
            for m in NAMED.finditer(body):
                out.setdefault(m.group(1), []).append(body)
    return out


def main():
    every = "--all" in sys.argv
    seats = open(f"{ROOT}/crabomination/src/pod/decks.rs", encoding="utf-8").read()
    bodies = factories()
    rows = []
    for name, card in CACHE.items():
        if not isinstance(card, dict):
            continue
        oracle = card.get("oracle_text") or ""
        low = oracle.lower()
        if "when you do" not in low:
            continue
        front = name.split(" // ")[0]
        if front in REVIEWED or front not in bodies or any("Reflexive" in b for b in bodies[front]):
            continue
        targeted = "target" in low.split("when you do", 1)[1][:200]
        if not (targeted or every):
            continue
        rows.append((len(re.findall(rf"\b{snake(front)}\b", seats)), front, targeted))
    for n, name, targeted in sorted(rows, key=lambda r: (-r[0], r[1])):
        print(f"{n:4}  {name}" + ("" if targeted else "  (untargeted)"))
    print(f"# {len(rows)} cards", file=sys.stderr)


if __name__ == "__main__":
    main()
