"""Generate COMMANDER_BACKLOG.md — Commander cards the catalog is missing,
ranked by how much Commander actually plays them.

Three lists, all against the live catalog (`cargo run --bin dump_cards`):

1. EDHREC's most-built commanders (past two years) that are not in the catalog.
2. The most-played Commander-legal cards (Scryfall's `edhrec_rank`) that are
   not in the catalog.
3. Cards whose rules text names a commander, the command zone or colour
   identity — the ones that need Commander-specific primitives — split into
   missing and "in the catalog, check the clause" (several drop it; see
   INCOMPLETE_CARDS.md section 7).

Sources are fetched once into a cache directory (default /tmp/crab_commander):
Scryfall's `oracle_cards` bulk export and EDHREC's public commander pages.

Usage:
    python scripts/commander_backlog.py                 # fetch (if needed) + write
    python scripts/commander_backlog.py --refresh       # re-download the sources
    python scripts/commander_backlog.py --catalog FILE  # reuse a dump_cards JSON
"""
import argparse
import datetime
import gzip
import json
import re
import subprocess
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
OUT = REPO / "COMMANDER_BACKLOG.md"
HEADERS = {"User-Agent": "crabomination-fetch/1.0", "Accept": "application/json"}
EDHREC = "https://json.edhrec.com/pages/"
COMMANDER_PAGES = 3  # 100 commanders a page
STAPLE_WINDOW = 1000
MECHANIC_WINDOW = 5000
BASICS = {"Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes"}
SKIP_LAYOUTS = {"token", "emblem", "art_series", "double_faced_token"}

# Oracle-text flags: a hint at what a card needs, not a verdict.
FLAGS = [
    ("commander", r"\bcommander|command zone|color identity"),
    ("partner", r"^partner|friends forever|choose a background|doctor's companion"),
    ("monarch", r"\bmonarch\b"),
    ("initiative", r"\binitiative\b|undercity"),
    ("dungeon", r"\bventure into\b"),
    ("vote", r"\bvote"),
    ("goad", r"\bgoad"),
    ("multiplayer", r"each other player|for each opponent|each opponent who|opponents? of your choice"),
]


def fetch_json(url):
    req = urllib.request.Request(url, headers=HEADERS)
    with urllib.request.urlopen(req) as r:
        return json.load(r)


def load_scryfall(cache, refresh):
    path = cache / "oracle_cards.jsonl.gz"
    if refresh or not path.exists():
        index = fetch_json("https://api.scryfall.com/bulk-data")
        entry = next(e for e in index["data"] if e["type"] == "oracle_cards")
        req = urllib.request.Request(entry["jsonl_download_uri"], headers=HEADERS)
        with urllib.request.urlopen(req) as r:
            path.write_bytes(r.read())
    with gzip.open(path, "rt") as f:
        return [json.loads(line) for line in f]


def load_commanders(cache, refresh):
    path = cache / "edhrec_commanders.json"
    if refresh or not path.exists():
        page = fetch_json(EDHREC + "commanders/year.json")
        cl = page["container"]["json_dict"]["cardlists"][0]
        views, more = list(cl["cardviews"]), cl.get("more")
        for _ in range(COMMANDER_PAGES - 1):
            if not more:
                break
            nxt = fetch_json(EDHREC + more)
            views += nxt["cardviews"]
            more = nxt.get("more")
        path.write_text(json.dumps(views))
    return json.loads(path.read_text())


def load_catalog(path):
    if path is None:
        out = subprocess.run(
            ["cargo", "run", "-q", "--bin", "dump_cards"],
            cwd=REPO, capture_output=True, text=True, check=True,
        ).stdout
        cards = json.loads(out)
    else:
        cards = json.loads(Path(path).read_text())
    return {c["name"] for c in cards}


def oracle(c):
    if "oracle_text" in c:
        return c["oracle_text"]
    return "\n".join(f.get("oracle_text", "") for f in c.get("card_faces", []))


def rules_text(c):
    """Oracle text without reminder text — Partner's "(You can have two
    commanders…)" would otherwise flag every partner as commander-aware."""
    return re.sub(r"\([^)]*\)", "", oracle(c)).lower()


def flags(c):
    text = rules_text(c)
    return ", ".join(k for k, pat in FLAGS if re.search(pat, text, re.M))


def front(name):
    return name.split(" // ")[0]


def md_escape(s):
    return s.replace("|", "\\|")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cache", default="/tmp/crab_commander")
    ap.add_argument("--refresh", action="store_true")
    ap.add_argument("--catalog", help="dump_cards JSON (default: run the bin)")
    args = ap.parse_args()
    cache = Path(args.cache)
    cache.mkdir(parents=True, exist_ok=True)

    cards = load_scryfall(cache, args.refresh)
    commanders = load_commanders(cache, args.refresh)
    catalog = load_catalog(args.catalog)

    def have(name):
        return name in catalog or front(name) in catalog

    # Art-series memorabilia share names with real cards ("Card // Card").
    by_name = {}
    for c in cards:
        if c.get("layout") in SKIP_LAYOUTS:
            continue
        by_name[c["name"]] = c
        by_name.setdefault(front(c["name"]), c)

    # EDHREC names a partner pair "A // B"; a DFC is one Scryfall card under
    # the same shape. A pair is covered when every half is.
    def halves(name):
        if name in by_name:
            return [name]
        return name.split(" // ")

    legal = [
        c for c in cards
        if c.get("legalities", {}).get("commander") == "legal"
        and c.get("layout") not in SKIP_LAYOUTS
        and c.get("edhrec_rank")
        and c["name"] not in BASICS
    ]
    legal.sort(key=lambda c: c["edhrec_rank"])

    lines = []
    w = lines.append
    w("# Commander backlog")
    w("")
    w("Generated by `scripts/commander_backlog.py` — do not hand-edit; re-run it.")
    w(f"Sources fetched {datetime.date.today().isoformat()}: Scryfall `oracle_cards`")
    w("(`edhrec_rank`, Commander legality, oracle text) and EDHREC's commander pages")
    w("(decks built, past two years). \"Flags\" are oracle-text hints at the")
    w("mechanic a card leans on, not a verdict on what it needs.")
    w("")
    w("## Coverage")
    w("")
    w("| Slice | In catalog |")
    w("| --- | --- |")
    for n in (100, 250, 500, 1000, 2000, 5000):
        top = legal[:n]
        w(f"| Top {n} Commander cards by EDHREC rank | {sum(have(c['name']) for c in top)} / {len(top)} |")
    for n in (100, len(commanders)):
        top = commanders[:n]
        covered = sum(all(have(h) for h in halves(v["name"])) for v in top)
        w(f"| Top {n} commanders by decks built | {covered} / {len(top)} |")
    w("")

    def cmdr_have(v):
        return all(have(h) for h in halves(v["name"]))

    missing_cmdrs = [v for v in commanders if not cmdr_have(v)]
    w(f"## 1. Most-built commanders not in the catalog ({len(missing_cmdrs)})")
    w("")
    w("A partner pair lists only its missing halves.")
    w("")
    w("| EDHREC # | Commander | Decks | Identity | Type | Flags |")
    w("| --- | --- | --- | --- | --- | --- |")
    for v in missing_cmdrs:
        for h in halves(v["name"]):
            if have(h):
                continue
            c = by_name.get(h) or by_name.get(front(h)) or {}
            ident = "".join(c.get("color_identity", [])) or "C"
            w(f"| {v['rank']} | {md_escape(h)} | {v['num_decks']:,} | {ident} "
              f"| {md_escape(c.get('type_line', ''))} | {flags(c) if c else ''} |")
    w("")

    missing_staples = [c for c in legal[:STAPLE_WINDOW] if not have(c["name"])]
    w(f"## 2. Top {STAPLE_WINDOW} Commander cards not in the catalog ({len(missing_staples)})")
    w("")
    w("| EDHREC rank | Card | Cost | Type | Flags |")
    w("| --- | --- | --- | --- | --- |")
    for c in missing_staples:
        w(f"| {c['edhrec_rank']} | {md_escape(c['name'])} | {c.get('mana_cost', '')} "
          f"| {md_escape(c['type_line'])} | {flags(c)} |")
    w("")

    commander_text = [
        c for c in legal[:MECHANIC_WINDOW]
        if re.search(FLAGS[0][1], rules_text(c))
    ]
    miss = [c for c in commander_text if not have(c["name"])]
    got = [c for c in commander_text if have(c["name"])]
    w(f"## 3. Cards that name a commander / the command zone / colour identity "
      f"(top {MECHANIC_WINDOW})")
    w("")
    w("These need Commander primitives (an \"is a commander\" selector, a")
    w("commander's-colour-identity mana source, \"if you control your commander\").")
    w("")
    w(f"### Missing ({len(miss)})")
    w("")
    w("| EDHREC rank | Card | Type | Oracle |")
    w("| --- | --- | --- | --- |")
    for c in miss:
        text = md_escape(" ".join(oracle(c).split()))
        w(f"| {c['edhrec_rank']} | {md_escape(c['name'])} | {md_escape(c['type_line'])} "
          f"| {text[:160]}{'…' if len(text) > 160 else ''} |")
    w("")
    w(f"### In the catalog — check the commander clause is modelled ({len(got)})")
    w("")
    w("| EDHREC rank | Card | Type |")
    w("| --- | --- | --- |")
    for c in got:
        w(f"| {c['edhrec_rank']} | {md_escape(c['name'])} | {md_escape(c['type_line'])} |")
    w("")

    OUT.write_text("\n".join(lines))
    print(f"Wrote {OUT} — {len(missing_cmdrs)} commanders, {len(missing_staples)} staples, "
          f"{len(miss)} + {len(got)} commander-text cards")


if __name__ == "__main__":
    main()
