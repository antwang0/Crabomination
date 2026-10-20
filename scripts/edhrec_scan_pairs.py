#!/usr/bin/env python3
"""The partner-pair half of `edhrec_scan_top.py`.

    python3 scripts/edhrec_scan_pairs.py [CACHE_DIR] [--legal] [--max N]

`edhrec_scan_top.py` skips every EDHREC entry with a seated commander, so a
" // " pair whose halves are both seated elsewhere never shows. This keys on
the seated commander SET instead: an entry is unseated unless that exact pair
has a seat. Average decks are cached in `CACHE_DIR/avg/` (shared with the
single scan); output is `missing rank name [cards]`, fewest first. `--legal`
keeps decks whose commanders and missing cards are all Commander-legal
(`oracle_cards.jsonl.gz` in CACHE_DIR); `--max N` drops decks missing more.
"""
import gzip
import importlib.util
import json
import os
import re
import sys
import time
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
spec = importlib.util.spec_from_file_location("edhrec_deck", os.path.join(ROOT, "scripts", "edhrec_deck.py"))
e = importlib.util.module_from_spec(spec)
spec.loader.exec_module(e)


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


def main():
    argv = sys.argv[1:]
    args = [a for i, a in enumerate(argv) if not a.startswith("--") and (i == 0 or argv[i - 1] != "--max")]
    cache = args[0] if args else "/tmp/crab_commander"
    legal_only = "--legal" in sys.argv
    cap = int(sys.argv[sys.argv.index("--max") + 1]) if "--max" in sys.argv else 99
    os.makedirs(os.path.join(cache, "avg"), exist_ok=True)
    fns, printed = e.factories(), e.by_printed_name()
    resolve = lambda n: e.slug(n, True) if e.slug(n, True) in fns else e.slug(n) if e.slug(n) in fns else printed.get(n, e.slug(n))
    decks_rs = open(os.path.join(ROOT, "crabomination", "src", "pod", "decks.rs"), encoding="utf-8").read()
    seated = {
        frozenset(f.strip().split("::")[-1] for f in m.group(1).split(",") if f.strip())
        for m in re.finditer(r"_COMMANDERS: &\[CardFactory\] = &\[([^\]]*)\]", decks_rs)
    }
    leg = legality(cache) if legal_only else {}
    out = []
    for t in json.load(open(os.path.join(cache, "edhrec_commanders.json"))):
        if " // " not in t["name"] or frozenset(resolve(n) for n in t["name"].split(" // ")) in seated:
            continue
        path = os.path.join(cache, "avg", t["slug"] + ".json")
        if not os.path.exists(path):
            req = urllib.request.Request(e.AVG_URL.format(t["slug"]), headers=e.HEADERS)
            try:
                open(path, "wb").write(urllib.request.urlopen(req, timeout=30).read())
            except Exception as ex:  # a 404 for a brand-new pair, a timeout
                print(f"skip {t['name']}: {ex}", file=sys.stderr)
                continue
            time.sleep(0.2)
        d = json.load(open(path))["deck"]
        rows = [tuple(x) for x in d["commander_v2"]] + [tuple(x) for v in d["cards"].values() for x in v]
        missing = [n for n, _ in rows if n not in e.BASICS and resolve(n) not in fns]
        if len(missing) > cap or (legal_only and not all(leg.get(c, False) for c in t["name"].split(" // ") + missing)):
            continue
        out.append((len(missing), t["rank"], t["name"], t["slug"], missing))
    for n, rank, name, slug, missing in sorted(out):
        print(n, rank, name, f"<{slug}>", missing)


if __name__ == "__main__":
    main()
