#!/usr/bin/env python3
"""Grant durations the printed text names vs the `Duration`s the definition uses.

A printed "until end of turn" shipped as `Duration::Permanent` (the Embodiment
cycle's land animation) outlives its turn; a permanent grant shipped as
`EndOfTurn` evaporates. Presence check per card (TODO NEXT, Commander item 13):

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_durations.py /tmp/all.tsv [--pod]

Rows: `eot_missing` — the oracle says "until end of turn" / "this turn" but the
definition has no end-of-turn duration anywhere; `permanent_only` — the oracle
says "until end of turn" and every duration in the definition is `Permanent`.
Both columns are leads (bespoke effects carry their own durations), not a gate.

`--gate` checks one exact class and exits 1 on any hit: "create … token(s).
It gains / They gain KEYWORDS until end of turn" where the token definition
carries one of those keywords — a grant baked on permanently (Windcrag Siege's
lifelink Goblin, Mardu Monument's menace Warriors). Fix with
`shortcut::tokens_gain_until_eot`.
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of, pod_idents, snake  # noqa: E402

EOT = re.compile(r"until end of turn")
EOT_DBG = re.compile(r"\b(EndOfTurn|EndOfThisTurn|ThisTurn|EndOfCombat)\b|_this_turn|_until_eot|UntilEot|Eot\b")


KW = {"haste": "Haste", "lifelink": "Lifelink", "flying": "Flying", "trample": "Trample",
      "menace": "Menace", "vigilance": "Vigilance", "first strike": "FirstStrike",
      "deathtouch": "Deathtouch", "hexproof": "Hexproof", "indestructible": "Indestructible"}
TOKEN_EOT = re.compile(r"create[^.]*token[^.]*\.\s*(?:It|They|Those tokens|That token|The token|The tokens)"
                       r" gains? ([a-z ,]+?) until end of turn", re.I)
TOKEN_KWS = re.compile(r'TokenDefinition \{ name: "[^"]*", power: -?\d+, toughness: -?\d+, keywords: \[([^\]]*)\]')


def baked_token_grants(name, oracle, dbg):
    bad = []
    for m in TOKEN_EOT.finditer(oracle):
        kws = [v for k, v in KW.items() if k in m.group(1).lower()]
        toks = TOKEN_KWS.findall(dbg)
        bad += [k for k in kws if any(re.search(r"\b" + k + r"\b", t) for t in toks)]
    return bad


def main():
    rows = [l.rstrip("\n").split("\t", 1) for l in open(sys.argv[1], encoding="utf-8") if "\t" in l]
    pod = pod_idents() if "--pod" in sys.argv else None
    cache = json.load(open(CACHE, encoding="utf-8"))
    lower = {k.lower(): v for k, v in cache.items()}
    out = []
    gate = "--gate" in sys.argv
    for name, dbg in rows:
        if pod is not None and snake(name) not in pod:
            continue
        card = cache.get(name) or lower.get(name.lower())
        if not isinstance(card, dict):
            continue
        if gate:
            bad = baked_token_grants(name, oracle_of(card), dbg)
            if bad:
                out.append(f"{name}\tbaked_token_grant\t{','.join(bad)}")
            continue
        oracle = oracle_of(card).lower()
        if not EOT.search(oracle):
            continue
        durs = re.findall(r"duration: (\w+)", dbg)
        if not EOT_DBG.search(dbg):
            out.append(f"{name}\teot_missing\t{','.join(sorted(set(durs)))}")
        elif durs and all(d == "Permanent" for d in durs):
            out.append(f"{name}\tpermanent_only")
    print("\n".join(sorted(out)))
    print(f"# {len(out)} rows", file=sys.stderr)
    if gate and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
