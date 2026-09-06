#!/usr/bin/env python3
"""Summarize `deck_gauntlet` RESULT lines.

    gauntlet_summary.py results.txt [--vs INCUMBENT] [--pilot dflt] [--manifest DIR/manifest.tsv]

Groups RESULT lines by (deck, pilot, opp_pilot, field, pairs); pools seeds
(inverse-variance mean); prints win%, +/-1.96 se, seeds, and the delta vs
the incumbent's pooled cell of the same pilot/field/pairs with its own se.
"""
import math
import re
import sys
from collections import defaultdict


def parse(path):
    rows = []
    for line in open(path):
        if not line.startswith("RESULT "):
            continue
        kv = dict(re.findall(r"(\w+)=(\S+)", line))
        try:
            kv["win"] = float(kv["win"]); kv["se"] = float(kv["se"]); kv["games"] = int(kv["games"])
        except (KeyError, ValueError):
            continue
        rows.append(kv)
    return rows


def pool(cells):
    w = [1.0 / (c["se"] ** 2) if c["se"] > 0 else 0.0 for c in cells]
    if sum(w) == 0:
        return sum(c["win"] for c in cells) / len(cells), float("nan")
    p = sum(wi * c["win"] for wi, c in zip(w, cells)) / sum(w)
    return p, 1.0 / math.sqrt(sum(w))


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__); sys.exit(2)
    path = args[0]
    vs = None; pilot = None; manifest = {}
    i = 1
    while i < len(args):
        if args[i] == "--vs":
            vs = args[i + 1]; i += 2
        elif args[i] == "--pilot":
            pilot = args[i + 1]; i += 2
        elif args[i] == "--manifest":
            for line in open(args[i + 1]):
                if "\t" in line:
                    k, v = line.rstrip("\n").split("\t", 1); manifest[k] = v
            i += 2
        else:
            i += 1
    rows = parse(path)
    groups = defaultdict(list)
    for r in rows:
        if pilot and r["pilot"] != pilot:
            continue
        groups[(r["deck"], r["pilot"], r["opp_pilot"], r["field"], r["pairs"])].append(r)
    pooled = {k: pool(v) for k, v in groups.items()}
    inc = {}
    if vs:
        for k, (p, se) in pooled.items():
            if k[0] == vs:
                inc[k[1:]] = (p, se)
    out = []
    for k, cells in groups.items():
        p, se = pooled[k]
        seeds = ",".join(c["seed"] for c in cells)
        base = inc.get(k[1:])
        if base and k[0] != vs:
            dp = p - base[0]; dse = math.sqrt(se ** 2 + base[1] ** 2)
            delta = f"{dp:+6.2f} ±{1.96 * dse:4.2f} ({dp / dse if dse else float('nan'):+.1f}σ)"
        else:
            delta = ""
        out.append((k[1], k[2], p, k[0], se, seeds, sum(c["games"] for c in cells), delta))
    out.sort(key=lambda t: (t[0], t[1], -t[2]))
    print(f"{'pilot':<18}{'opp':<8}{'win%':>7} {'±':>5} {'games':>6}  {'seeds':<10} {'delta vs ' + (vs or '-'):<26} deck")
    for pl, op, p, deck, se, seeds, games, delta in out:
        note = manifest.get(deck, "")
        print(f"{pl:<18}{op:<8}{p:7.2f} {1.96 * se:5.2f} {games:6d}  {seeds:<10} {delta:<26} {deck}  {note}")


if __name__ == "__main__":
    main()
