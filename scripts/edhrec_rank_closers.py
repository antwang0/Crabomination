#!/usr/bin/env python3
"""Rank missing cards by how many unseated Commander decks they help close.

    python3 scripts/edhrec_scan_top.py > scan.txt
    python3 scripts/edhrec_rank_closers.py scan.txt [--closes CARDS.txt] [CACHE_DIR]

Reads `edhrec_scan_top.py`'s output and Scryfall's `oracle_cards.jsonl.gz` in
CACHE_DIR (default /tmp/crab_commander; `commander_backlog.py` fetches it),
keeps decks whose commanders AND missing cards are all Commander-legal, and
prints each missing card's score Σ 1/missing over the decks it appears in —
clusters of cards shared by many near-complete decks rise to the top.
`--closes` takes a file of card names (one a line) and instead lists the decks
that set would leave at zero or one missing card.
"""
import ast
import collections
import gzip
import json
import os
import sys


def legality(cache):
    leg = {}
    with gzip.open(os.path.join(cache, "oracle_cards.jsonl.gz"), "rt") as f:
        for line in f:
            c = json.loads(line)
            ok = c["legalities"].get("commander") == "legal"
            leg[c["name"]] = ok
            if " // " in c["name"]:
                leg[c["name"].split(" // ")[0]] = ok
    return leg


def decks(scan, leg):
    for line in open(scan, encoding="utf-8"):
        rest = line.split(" ", 2)[2]
        name, miss = rest[: rest.index(" [")], ast.literal_eval(line[line.index("["):])
        if all(leg.get(c, False) for c in name.split(" // ") + miss):
            yield name, miss


def main():
    args = sys.argv[1:]
    closes = None
    if "--closes" in args:
        i = args.index("--closes")
        closes = set(open(args[i + 1], encoding="utf-8").read().split("\n")) - {""}
        del args[i : i + 2]
    scan, cache = args[0], (args[1] if len(args) > 1 else "/tmp/crab_commander")
    rows = list(decks(scan, legality(cache)))
    if closes is not None:
        for name, miss in rows:
            left = [m for m in miss if m not in closes]
            if miss and len(left) <= 1 and len(left) < len(miss):
                print(len(left), name, left)
        return
    print("legal decks:", len(rows), sorted(collections.Counter(len(m) for _, m in rows).items()))
    score, count = collections.Counter(), collections.Counter()
    for _, miss in rows:
        for c in miss:
            score[c] += 1.0 / len(miss)
            count[c] += 1
    for c, v in score.most_common(60):
        print(f"{v:.2f} {count[c]} {c}")


if __name__ == "__main__":
    main()
