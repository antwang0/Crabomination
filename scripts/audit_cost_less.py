#!/usr/bin/env python3
"""Printed "costs {N} less" vs the definition's cost-reduction amounts.

Every `*Cost*` variant with a top-level `amount: N` (CostReduction,
AllPlayersSpellsCostLess, FirstMatchingSpellEachTurnCostsLess, ...) must name
an N the oracle prints as "{N} less"; a card whose oracle prints a flat
"{N} less" but whose definition carries no such amount is reported too.

    cargo build -p crabomination --bin dump_cards
    (for n in Less Reduction SelfCostReduced self_cost_reduction reduce_generic; do target/debug/dump_cards --grep $n; done) > /tmp/cl.tsv
    target/debug/dump_cards --shape > /tmp/shape.tsv
    python3 scripts/audit_cost_less.py /tmp/cl.tsv /tmp/shape.tsv [--gate]
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")


def spans(dbg, pat):
    for m in re.finditer(pat, dbg):
        i = j = m.end()
        depth = 0
        while j < len(dbg):
            ch = dbg[j]
            if ch in "({[":
                depth += 1
            elif ch in ")}]":
                if depth == 0:
                    break
                depth -= 1
            j += 1
        yield m.group(1), dbg[i:j]


def top_amount(body):
    """The variant's own `amount: N` / `per: N` — (N, is_per)."""
    depth = 0
    out = None
    for m in re.finditer(r"[({\[]|[)}\]]|(amount|per): (\d+)", body):
        t = m.group(0)
        if t in "({[":
            depth += 1
        elif t in ")}]":
            depth -= 1
        elif depth == 0:
            out = (int(m.group(2)), m.group(1) == "per")
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit(__doc__)
    cache = {k: v for k, v in json.load(open(CACHE)).items() if isinstance(v, dict)}
    by_name = {}
    for k, v in cache.items():
        by_name.setdefault(k.lower(), v)
        by_name.setdefault(k.split(" // ")[0].lower(), v)
    rows = []
    for line in open(args[0]):
        name, dbg = line.rstrip("\n").split("\t", 1)
        card = by_name.get(name.lower())
        if not card:
            continue
        faces = card.get("card_faces") or []
        oracle = card.get("oracle_text") or "\n".join(f.get("oracle_text", "") for f in faces)
        oracle = re.sub(r"\([^)]*\)", "", oracle)
        printed = {int(n) for n in re.findall(r"\{(\d+)\} less", oracle)}
        # "{1} less for each" is a per-unit reduction, not a flat amount.
        flat = {int(n) for n in re.findall(r"\{(\d+)\} less(?! (?:to cast |to activate )?for each)", oracle)}
        got = set()
        for variant, body in spans(dbg, r"(\w*(?:Cost|Discount)\w*) \{ "):
            if not re.search(r"Less|Reduc|Discount", variant):
                continue
            a = top_amount(body)
            if a is not None:
                got.add((variant, a))
        for field, body in spans(dbg, r"((?:self_)?cost_reduction_\w+): Some\(\("):
            m = re.search(r", (\d+)$", body)
            if m:
                got.add((field, (int(m.group(1)), field.endswith("_per"))))
        for field, body in spans(dbg, r"((?:self_)?cost_reduction_\w+): \["):
            depth, start = 0, 0
            for i, ch in enumerate(body):
                if ch in "({[":
                    depth += 1
                    if depth == 1:
                        start = i + 1
                elif ch in ")}]":
                    depth -= 1
                    m = re.search(r", (\d+)$", body[start:i]) if depth == 0 else None
                    if m:
                        got.add((field, (int(m.group(1)), False)))
        for n in re.findall(r"reduce_generic: (\d+)", dbg):
            if n != "0":
                got.add(("reduce_generic", (int(n), False)))
        raw = card.get("oracle_text") or "\n".join(f.get("oracle_text", "") for f in faces)
        per_each = {int(n) for n in re.findall(r"\{(\d+)\} less(?: to cast| to activate)? for each", raw)}
        if "{X} less" in raw:
            per_each.add(1)
        for variant, (a, is_per) in sorted(got):
            want = per_each if is_per else printed
            if a not in want:
                rows.append(f"{name:40} {variant} {'per' if is_per else 'amount'} {a} vs printed {sorted(want) or 'none'}")
    # A catalog card that prints a flat "{N} less" but reached neither grep.
    seen = {l.split("\t", 1)[0] for l in open(args[0]) if re.search(r"Less|Reduc|Discount|(?:self_)?cost_reduction_\w+: (?:Some|\[\()|reduce_generic: [1-9]", l)}
    for line in open(args[1]) if len(args) > 1 else []:
        name = line.split("\t", 1)[0]
        card = by_name.get(name.lower())
        if name in seen or not card:
            continue
        oracle = card.get("oracle_text") or "\n".join(f.get("oracle_text", "") for f in card.get("card_faces") or [])
        flat = re.findall(r"\{(\d+)\} less(?! (?:to cast |to activate )?for each)", re.sub(r"\([^)]*\)", "", oracle))
        if flat:
            rows.append(f"{name:40} printed {sorted(set(flat))} less; no reduction in def")
    for r in rows:
        print(r)
    print(f"{len(rows)} rows")
    if "--gate" in sys.argv and rows:
        sys.exit(1)


main()
