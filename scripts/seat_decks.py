#!/usr/bin/env python3
"""Append EDHREC average decks as new Commander pod seats.

    python3 scripts/seat_decks.py CACHE_DIR SLUG [SLUG ...]

For each SLUG (an average deck already cached at `CACHE_DIR/avg/SLUG.json` by
`edhrec_scan_top.py`), writes the seat's `*_COMMANDERS` / `*_MAIN` consts to
the end of `crabomination/src/pod/decks.rs`, a `PodDeck` at the end of
`target_decks()` and a one-seed row at the end of
`cr_903_5a_the_official_precon_seats_play_pod_games`' seat table (in
`pod/mod.rs`). Seat numbers, consts and seeds continue from what is there. A
deck with a card that has no factory is refused (`edhrec_deck.py` reports it).
"""
import datetime
import importlib.util
import io
import json
import os
import re
import sys
from contextlib import redirect_stdout

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
spec = importlib.util.spec_from_file_location("edhrec_deck", os.path.join(ROOT, "scripts", "edhrec_deck.py"))
e = importlib.util.module_from_spec(spec)
spec.loader.exec_module(e)
DECKS = os.path.join(ROOT, "crabomination", "src", "pod", "decks.rs")
MOD = os.path.join(ROOT, "crabomination", "src", "pod", "mod.rs")
WUBRG = "WUBRG"


def esc(s):
    """A Rust string-literal body (Miles "Tails" Prower)."""
    return s.replace("\\", "\\\\").replace('"', '\\"')


def identity(names):
    cache = json.load(open(os.path.join(ROOT, "scripts", ".scryfall_cache.json"), encoding="utf-8"))
    ci = set()
    for n in names:
        c = cache.get(n) or next((v for k, v in cache.items() if k.split(" // ")[0] == n), None)
        if c is None:
            sys.exit(f"{n}: not in the Scryfall cache (scripts/fetch_cards.py)")
        ci.update(c.get("color_identity", []))
    return "".join(x for x in WUBRG if x in ci) or "C"


def main():
    cache, slugs = sys.argv[1], sys.argv[2:]
    fns = e.factories()
    decks_rs = open(DECKS, encoding="utf-8").read()
    mod_rs = open(MOD, encoding="utf-8").read()
    seat = max(int(n) for n in re.findall(r"// Seat (\d+):", mod_rs))
    seed = max(int(n, 16) for n in re.findall(r'\("(?:[^"\\]|\\.)+", &\[0x([0-9A-F]+)\]\),\n        \];', mod_rs))
    pods, rows = [], []
    for slug in slugs:
        page = json.load(open(os.path.join(cache, "avg", slug + ".json"), encoding="utf-8"))
        cmd_names = [n for n, _ in page["deck"]["commander_v2"]]
        first = cmd_names[0].split(", ")[0].split(" // ")[0]
        const = re.sub(r"[^A-Z0-9]+", "_", e.slug(first).upper()).strip("_")
        base, n = const, 2
        while re.search(rf"\b{const}_COMMANDERS\b", decks_rs):
            const, n = f"{base}_{n}", n + 1
        title = " and ".join(cmd_names) if len(cmd_names) > 1 else cmd_names[0]
        label = f"{title} ({identity(cmd_names)})"
        buf = io.StringIO()
        with redirect_stdout(buf):
            e.rust(page, fns, const, title, f"(EDHREC, fetched {datetime.date.today()}).", identity(cmd_names).replace("C", ""))
        decks_rs += buf.getvalue()
        seat += 1
        seed += 1
        pods.append(
            f"        // Seat {seat}: {title}'s EDHREC average deck. `--pod-decks {seat}`.\n"
            f"        PodDeck {{\n"
            f"            name: \"{esc(label)}\",\n"
            f"            commanders: decks::{const}_COMMANDERS,\n"
            f"            main: decks::{const}_MAIN,\n"
            f"        }},\n"
        )
        rows.append(f'            ("{esc(title)}", &[0x{seed:04X}]),\n')
        print(f"seat {seat}: {label} ({const})")
    end = mod_rs.index("    ]\n}\n\n/// `seats` decks for a pod")
    mod_rs = mod_rs[:end] + "".join(pods) + mod_rs[end:]
    m = list(re.finditer(r'\("(?:[^"\\]|\\.)+", &\[0x[0-9A-F]+\]\),\n(        \];)', mod_rs))[-1]
    mod_rs = mod_rs[: m.start(1)] + "".join(rows) + mod_rs[m.start(1):]
    open(DECKS, "w", encoding="utf-8").write(decks_rs)
    open(MOD, "w", encoding="utf-8").write(mod_rs)


if __name__ == "__main__":
    main()
