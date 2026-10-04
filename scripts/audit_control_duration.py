#!/usr/bin/env python3
"""Printed "gain control of …" durations (CR 611.2, 611.2c) vs the definition.

Three printed shapes: "until end of turn" (threaten -> `duration: EndOfTurn`),
"for as long as …" (one of the `GainControlWhile*` variants), and no duration
(a permanent steal -> `duration: Permanent`). A row names a card whose
oracle prints one shape and whose definition holds no control change of that
shape. Exploratory; `--gate` exits 1 on any row.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_control_duration.py /tmp/all.tsv [--pod NAMES] [--gate]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

# The whole sentence: "Until end of turn, gain control of …" (Besmirch,
# Grab the Reins) puts the duration BEFORE the verb.
CLAUSE = re.compile(r"([^.]*?)gains? control of ([^.]*)", re.I)
SHAPES = {
    "eot": r"GainControl \{[^}]*duration: EndOfTurn|UntilEndOfTurn|Threaten|EndOfTurn",
    "while": r"GainControlWhile|WhileSource|WhileYouControl|WhileCounter",
    "perm": r"GainControl \{[^}]*duration: Permanent|ExchangeControl|GainControlAndReattach|Permanent",
}


def shape(clause):
    if re.search(r"until end of turn", clause, re.I):
        return "eot"
    if re.search(r"for as long as|until .* leaves", clause, re.I):
        return "while"
    if re.search(r"until (your|the|its|their) next|until the end of", clause, re.I):
        return None  # other bounded durations: read by hand
    return "perm"


# Reviewed rows: a bespoke effect carries the duration, or the clause is a
# trigger condition rather than a control change.
REVIEWED = {
    "Aethersnatch": "`GainControlOfSpell` — a spell, which keeps its new controller",
    "Akroan Horse": "`enters_under_opponent_control` — enters under the opponent, permanently",
    "Alicia Masters, Skilled Sculptor": "`OwnersGainControlOf` — owners regain control for good",
    "Brooding Saurian": "`OwnersGainControlOfNontokens` — owners regain control for good",
    "Dack Fayden, Helping Hand": "`DistributeControlAmongOpponents` — a permanent donation",
    "Debt of Loyalty": "`RegenerateThenGainControl` — a permanent steal",
    "Emrakul, the Promised End": "`ControlPlayerNextTurn` — CR 722, controlling a player",
    "Khârn the Betrayer": "`PreventDamageToSelfOpponentGainsControl` — a permanent donation",
    "Midnight Crusader Shuttle": "the 'if you gain control … this way' sentence; the steal is EndOfTurn",
    "Tahngarth, First Mate": "`Duration::EndOfCombat` ('until end of combat')",
    "Zidane, Tantalus Thief": "'an opponent gains control of a permanent from you' is a trigger event",
}


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    pod = None
    if "--pod" in sys.argv:
        path = sys.argv[sys.argv.index("--pod") + 1]
        pod = {ln.strip() for ln in open(path, encoding="utf-8")}
    out = []
    for line in open(sys.argv[1], encoding="utf-8"):
        if "\t" not in line:
            continue
        name, dbg = line.rstrip("\n").split("\t", 1)
        card = cache.get(name)
        if not isinstance(card, dict) or (pod is not None and name not in pod):
            continue
        if "Control" not in dbg:
            continue
        for m in CLAUSE.finditer(oracle_of(card)):
            s = shape(m.group(1) + " " + m.group(2))
            if s and not re.search(SHAPES[s], dbg):
                out.append(f"{name}\t{s}\t{m.group(2)[:90]}")
    out = sorted(set(out))
    flagged = {row.split("\t", 1)[0] for row in out}
    stale = sorted(set(REVIEWED) - flagged) if pod is None else []
    out = [row for row in out if row.split("\t", 1)[0] not in REVIEWED]
    print("\n".join(out))
    for name in stale:
        print(f"# REVIEWED is STALE: {name} no longer flagged", file=sys.stderr)
    print(f"# {len(out)} rows ({len(stale)} stale reviewed entries)", file=sys.stderr)
    if "--gate" in sys.argv and (out or stale):
        sys.exit(1)


if __name__ == "__main__":
    main()
