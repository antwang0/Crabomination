#!/usr/bin/env python3
"""Oracle vs definition: activated-ability cost flags.

Reads `dump_cards --activated` (one row per printed activated ability) and
counts, per card face, the oracle's "{T}", "{Q}", "Sacrifice this", "Pay N
life", "Activate only as a sorcery" and "Activate only once each turn"
against the definition's `tap_cost` / `untap_self_cost` / `sac_cost` /
`life_cost` / `sorcery_speed` / `once_per_turn`. Prints each face whose
counts differ; `--pod` restricts to cards in `pod/decks.rs`.

    target/debug/dump_cards --activated > /tmp/act.tsv
    python3 scripts/audit_activated_costs.py /tmp/act.tsv [--pod]
"""
import gzip, json, re, sys
from collections import defaultdict

ORACLE = "/tmp/crab_commander/oracle_cards.jsonl.gz"
QUOTE = re.compile(r'"[^"]*"|“[^”]*”')
REMINDER = re.compile(r"\([^)]*\)")
# A cost is built from these pieces only; anything else before the colon
# (a trigger, a keyword line, a quoted grant) is not an activated ability.
COST_WORD = re.compile(
    r"^(\{[^}]+\}|sacrifice|pay|discard|exile|remove|tap|untap|return|put|reveal|"
    r"collect|mill|forage|behold|waterbend|earthbend|expend|blight|,|\s)+",
    re.I,
)


def faces():
    out = {}
    try:
        with gzip.open(ORACLE, "rt") as f:
            cards = [json.loads(line) for line in f]
    except FileNotFoundError:
        # Offline: the committed per-card cache (scripts/oracle.py's).
        import os
        cache = os.path.join(os.path.dirname(os.path.abspath(__file__)), ".scryfall_cache.json")
        cards = [v for v in json.load(open(cache)).values() if isinstance(v, dict)]
    for d in cards:
        for face in d.get("card_faces") or [d]:
            if "oracle_text" in face:
                out.setdefault(face["name"], face["oracle_text"])
    return out


def oracle_features(name, text):
    feats = defaultdict(int)
    for line in text.split("\n"):
        line = QUOTE.sub("", REMINDER.sub("", line)).strip()
        if ":" not in line:
            continue
        # Ability words and flavour words ("Lethal Voice — …") precede the cost.
        line = re.sub(r"^[^:{]*? — ", "", line)
        cost, _, rest = line.partition(":")
        if not cost or not COST_WORD.match(cost) or re.match(r"^[+−-]?\d+$|^[+−-]X$", cost.strip()):
            continue
        low = cost.lower()
        feats["tap"] += "{t}" in low
        feats["untap"] += "{q}" in low
        short = name.split(",")[0].lower()
        feats["sac"] += bool(re.search(rf"sacrifice (this|{re.escape(name.lower())}|{re.escape(short)})\b", low))
        m = re.search(r"pay (\d+) life", low)
        feats["life"] += int(m.group(1)) if m else 0
        feats["sorcery"] += "activate only as a sorcery" in rest.lower()
        feats["once"] += "activate only once each turn" in rest.lower()
    return feats


def snake(name):
    """The catalog's factory name for a card: `Sol Ring` -> `sol_ring`."""
    return re.sub(r"[^a-z0-9]+", "_", name.lower().replace("'", "")).strip("_")


def main():
    rows = defaultdict(lambda: defaultdict(int))
    for line in open(sys.argv[1]):
        name, _i, tap, untap, sac, sorc, once, life, _mv = line.rstrip("\n").split("\t")
        r = rows[name]
        r["tap"] += int(tap)
        r["untap"] += int(untap)
        r["sac"] += int(sac)
        r["sorcery"] += int(sorc)
        r["once"] += int(once)
        r["life"] += int(life)
    only = None
    if "--pod" in sys.argv:
        idents = set(re.findall(r"\b[a-z][a-z0-9_]*\b", open("crabomination/src/pod/decks.rs").read()))
        only = {n for n in rows if snake(n) in idents}
    oracle = faces()
    n = 0
    for name in sorted(rows):
        if only is not None and name not in only:
            continue
        text = oracle.get(name)
        if text is None:
            continue
        o, d = oracle_features(name, text), rows[name]
        diff = {k: (o[k], d[k]) for k in ("tap", "untap", "sac", "sorcery", "once", "life") if o[k] != d[k]}
        # Only a definition that has FEWER than printed is suspect; extra flags
        # come from abilities the oracle grants elsewhere (level up, equip…).
        diff = {k: v for k, v in diff.items() if v[0] > v[1]}
        if diff:
            n += 1
            print(f"{name}\t" + " ".join(f"{k} oracle {a} def {b}" for k, (a, b) in diff.items()))
    print(f"# {n} faces", file=sys.stderr)


if __name__ == "__main__":
    main()
