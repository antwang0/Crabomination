#!/usr/bin/env python3
"""Printed "target player [exiles / discards / sacrifices / mills] …" whose
body targets the OBJECT instead of the player: at three seats the wrong seat's
card can be taken, player hexproof is ignored, and no "becomes the target"
trigger sees the player (CR 115.1). The player-target shape is
`TargetPlayerThen` / `R::Player` in a slot, then the verb over `Target(0)`.

    python3 scripts/audit_target_player_acts.py
"""

import glob
import json
import re
import sys
import unicodedata

ROOT = __file__.rsplit("/scripts/", 1)[0]
CACHE = json.load(open(f"{ROOT}/scripts/.scryfall_cache.json"))
NAMED = re.compile(r'(?:name:\s*|\b(?:legend|creature|spell|permanent|sorcery|instant|land)\(\s*)"([^"]+)"')
# "target player" followed, within the clause, by an act of theirs.
PRINTED = re.compile(
    r"target (?:player|opponent)[^.;]{0,60}?\b(exiles|discards|sacrifices|mills|reveals|puts)\b", re.I
)
# A body that names the player as a target: a player slot, or the player-target shell.
PLAYER_TARGET = re.compile(
    r"TargetPlayerThen|R::Player\b|SelectionRequirement::Player\b|R::OpponentPlayer\b"
    r"|SelectionRequirement::OpponentPlayer\b|PlayerRef::Target\(|ForEachOpponentTarget|ForEachPlayerTarget"
    r"|Selector::Target\(|Selector::Player\(PlayerRef::Target"
)

# Reviewed and left as an object target, with the reason.
REVIEWED = {}


def snake(name):
    n = unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode().lower()
    return re.sub(r"[^a-z0-9]+", "_", n.replace("'", "").replace(",", "")).strip("_")


def main():
    bodies = {}
    for path in glob.glob(f"{ROOT}/crabomination_catalog/src/sets/**/*.rs", recursive=True):
        src = open(path, encoding="utf-8").read()
        cuts = [m.start() for m in re.finditer(r"\n(?:pub(?:\([a-z]+\))? )?fn ", src)] + [len(src)]
        for a, b in zip(cuts, cuts[1:]):
            body = src[a:b]
            for m in NAMED.finditer(body):
                bodies.setdefault(m.group(1), []).append((path, body))
    seats = open(f"{ROOT}/crabomination/src/pod/decks.rs", encoding="utf-8").read()
    rows = []
    for name, card in CACHE.items():
        if not isinstance(card, dict):
            continue
        hit = PRINTED.search(card.get("oracle_text") or "")
        front = name.split(" // ")[0]
        if not hit or front in REVIEWED or front not in bodies:
            continue
        if any(PLAYER_TARGET.search(b) for _, b in bodies[front]):
            continue
        rows.append((len(re.findall(rf"\b{snake(front)}\b", seats)), front, hit.group(0)))
    for n, name, clause in sorted(rows, key=lambda r: (-r[0], r[1])):
        print(f"{n:4}  {name}  — {clause}")
    print(f"# {len(rows)} cards", file=sys.stderr)


if __name__ == "__main__":
    main()
