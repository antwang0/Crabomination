#!/usr/bin/env python3
"""Which of EDHREC's most-built commanders could be the next pod seats.

    python3 scripts/edhrec_scan_top.py [CACHE_DIR]

Reads EDHREC's top-300 list from `CACHE_DIR/edhrec_commanders.json` (written
by `scripts/commander_backlog.py`; default /tmp/crab_commander), skips every
commander that already has a seat (`*_COMMANDERS` in `pod/decks.rs`), fetches
the rest's average decks into `CACHE_DIR/avg/` (cached), and prints them by
missing-card count, fewest first: `missing rank name [cards]`. Build the
cheapest with `scripts/edhrec_deck.py --file ... --rust ...`.
"""
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


def main():
    cache = sys.argv[1] if len(sys.argv) > 1 else "/tmp/crab_commander"
    os.makedirs(os.path.join(cache, "avg"), exist_ok=True)
    fns, printed = e.factories(), e.by_printed_name()
    resolve = lambda n: e.slug(n, True) if e.slug(n, True) in fns else e.slug(n) if e.slug(n) in fns else printed.get(n, e.slug(n))
    decks_rs = open(os.path.join(ROOT, "crabomination", "src", "pod", "decks.rs"), encoding="utf-8").read()
    seated = {f.strip() for m in re.finditer(r"_COMMANDERS: &\[CardFactory\] = &\[([^\]]*)\]", decks_rs) for f in m.group(1).split(",")}
    out = []
    for t in json.load(open(os.path.join(cache, "edhrec_commanders.json"))):
        if any(resolve(n) in seated for n in t["name"].split(" // ")):
            continue
        path = os.path.join(cache, "avg", t["slug"] + ".json")
        if not os.path.exists(path):
            req = urllib.request.Request(e.AVG_URL.format(t["slug"]), headers=e.HEADERS)
            try:
                open(path, "wb").write(urllib.request.urlopen(req, timeout=30).read())
            except Exception as ex:  # a 404 for a brand-new commander, a timeout
                print(f"skip {t['name']}: {ex}", file=sys.stderr)
                continue
            time.sleep(0.3)
        d = json.load(open(path))["deck"]
        rows = [tuple(x) for x in d["commander_v2"]] + [tuple(x) for v in d["cards"].values() for x in v]
        missing = [n for n, _ in rows if n not in e.BASICS and resolve(n) not in fns]
        out.append((len(missing), t["rank"], t["name"], missing))
    for m, rank, name, miss in sorted(out):
        print(m, rank, name, miss)


if __name__ == "__main__":
    main()
