#!/usr/bin/env python3
"""Does a card factory's DOC COMMENT still describe the card its BODY builds?

The class this hunts is `CARD_BACKLOG`'s "a synthesised card wearing a printed
card's name": the characteristics were corrected against Scryfall at some
point and the abilities were not, and **the doc comment above the body is the
tell** — it still describes the card the body used to be. Greasewrench Goblin
is the worked example: the doc says `{1}{R}` 2/2 Haste with an on-death
Treasure, the body says `{R}` 2/1, and Scryfall says `{R}` 2/1 with an exhaust
ability the body did not have.

Comment-free auditing is not possible here — the comment *is* the subject — so
this reads the source only and reports where the two disagree. A hit is a card
to read against the oracle, not a card that is definitely wrong: a doc that
says "{2}{R} 3/3 // {4}{R} 5/5" for a split card is a shape this cannot parse
and is skipped rather than reported.

    python3 scripts/audit_doc_drift.py            # the table
    python3 scripts/audit_doc_drift.py --rows 0   # every row
"""
import argparse
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crabomination_catalog" / "src"

FN = re.compile(r"^pub fn ([a-z0-9_]+)\(\) -> CardDefinition \{", re.M)
# `/// Name — {2}{R} 3/3 …`; the em dash is the codebase's separator.
DOC_COST = re.compile(r"—\s*((?:\{[^}]{1,7}\})+)")
DOC_PT = re.compile(r"\b(\d{1,2})/(\d{1,2})\b")
BODY_COST = re.compile(r"^ {8}cost: cost\(&\[([^\]]*)\]\)", re.M)
BODY_POWER = re.compile(r"^ {8}power: (-?\d+)", re.M)
BODY_TOUGH = re.compile(r"^ {8}toughness: (-?\d+)", re.M)

SIMPLE = {"w": "W", "u": "U", "b": "B", "r": "R", "g": "G", "x": "X"}

CACHE = json.load(open(pathlib.Path(__file__).resolve().parent / ".scryfall_cache.json"))
CACHE_LC = {k.lower(): v for k, v in CACHE.items() if isinstance(v, dict)}
BODY_NAME = re.compile(r'^ {8}name: "([^"]*)"', re.M)


def oracle_cost(name: str):
    """The printed cost as this script's symbol list, or None."""
    c = CACHE_LC.get(name.lower()) or CACHE_LC.get(name.lower().split(" // ")[0])
    if not c or not c.get("mana_cost"):
        return None
    return doc_cost_symbols(c["mana_cost"])


def oracle_pt(name: str):
    c = CACHE_LC.get(name.lower()) or CACHE_LC.get(name.lower().split(" // ")[0])
    if not c:
        return None
    p, t = c.get("power"), c.get("toughness")
    if p is None or t is None or not str(p).lstrip("-").isdigit() or not str(t).lstrip("-").isdigit():
        return None
    return (int(p), int(t))


def body_cost_symbols(expr: str):
    """`generic(3), b(), b()` -> ['3', 'B', 'B'], or None if unparseable."""
    out = []
    for item in re.findall(r"([A-Za-z_:]+)\(([^()]*)\)", expr):
        name, arg = item[0].split("::")[-1], item[1].strip()
        if name in SIMPLE and not arg:
            out.append(SIMPLE[name])
        elif name == "generic" and arg.isdigit():
            out.append(arg)
        elif name == "colorless" and arg.isdigit():
            out.extend(["C"] * int(arg))
        elif name == "snow_mana" and not arg:
            out.append("S")
        elif name in ("hybrid", "phyrexian", "mono_hybrid", "phyrexian_hybrid"):
            return None  # renderable, but not worth the false positives
        else:
            return None
    # A bare `cost(&[])` is `{0}`; the doc writes nothing for it.
    return out or None


def doc_cost_symbols(text: str):
    """`{3}{B}{B}` -> ['3', 'B', 'B'], or None if it holds anything exotic."""
    out = []
    for sym in re.findall(r"\{([^}]{1,7})\}", text):
        if sym.isdigit() or sym in ("W", "U", "B", "R", "G", "C", "X", "S"):
            out.append(sym)
        else:
            return None
    return out or None


def norm(s: str) -> str:
    return re.sub(r"[^a-z0-9]", "", s.lower())


def scan():
    rows = []
    skipped = [0]
    seen = 0
    for path in sorted(CATALOG.rglob("*.rs")):
        src = path.read_text(encoding="utf-8")
        lines = src.split("\n")
        for m in FN.finditer(src):
            name = m.group(1)
            line_no = src[: m.start()].count("\n")
            # The contiguous `///` block immediately above.
            doc = []
            i = line_no - 1
            while i >= 0 and (lines[i].lstrip().startswith("///") or lines[i].lstrip().startswith("#[")):
                if lines[i].lstrip().startswith("///"):
                    doc.append(lines[i].lstrip()[3:].strip())
                i -= 1
            if not doc:
                continue
            doc_text = " ".join(reversed(doc))
            # The body: to the next top-level `pub fn` or end of file.
            nxt = FN.search(src, m.end())
            body = src[m.end() : nxt.start() if nxt else len(src)]
            seen += 1
            bn = BODY_NAME.search(body)
            card_name = bn.group(1) if bn else None
            # ⚠ A factory that builds a *second* definition first — the flip
            # side (Goka the Unjust above Initiate of Blood), a DFC back, a
            # token — puts that definition's fields where the top-level regex
            # looks. Require the first `name:` to be the one the doc names, or
            # skip: without this the three CHK flip cards read as body bugs
            # and all three are correct.
            head = doc_text.split(" — ")[0].split(" – ")[0].strip()
            if not card_name or norm(card_name) != norm(head):
                continue

            problems = []
            dc = DOC_COST.search(doc_text)
            bc = BODY_COST.search(body)
            if dc and bc:
                d = doc_cost_symbols(dc.group(1))
                b = body_cost_symbols(bc.group(1))
                if d and b and sorted(d) != sorted(b):
                    o = oracle_cost(card_name) if card_name else None
                    # Only report what the oracle can adjudicate. A card the
                    # cache spells over two faces (Adventure, MDFC, flip) has
                    # a doc that legitimately names both halves, and every one
                    # of those read as a finding before this gate.
                    if o and sorted(o) == sorted(d):
                        problems.append(
                            f"cost doc {''.join('{'+x+'}' for x in d)} vs body "
                            f"{''.join('{'+x+'}' for x in b)} [BODY WRONG]")
                    elif o and sorted(o) == sorted(b):
                        problems.append(
                            f"cost doc {''.join('{'+x+'}' for x in d)} vs body "
                            f"{''.join('{'+x+'}' for x in b)} [doc rot]")
                    else:
                        skipped[0] += 1
            bp, bt = BODY_POWER.search(body), BODY_TOUGH.search(body)
            if bp or bt:
                # Only compare when the doc names exactly one N/M pair, so a
                # doc that spells a second body (adventure, MDFC) is skipped.
                pts = DOC_PT.findall(doc_text)
                if len(pts) == 1:
                    dp, dt = int(pts[0][0]), int(pts[0][1])
                    p = int(bp.group(1)) if bp else 0
                    t = int(bt.group(1)) if bt else 0
                    if (dp, dt) != (p, t):
                        o = oracle_pt(card_name) if card_name else None
                        if o == (dp, dt):
                            problems.append(f"P/T doc {dp}/{dt} vs body {p}/{t} [BODY WRONG]")
                        elif o == (p, t):
                            problems.append(f"P/T doc {dp}/{dt} vs body {p}/{t} [doc rot]")
                        else:
                            skipped[0] += 1
            if problems:
                rows.append((name, path.relative_to(ROOT), "; ".join(problems)))
    return seen, rows, skipped[0]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rows", type=int, default=40, help="0 = no cap")
    args = ap.parse_args()
    seen, rows, skipped = scan()
    buckets = {"BODY WRONG": 0, "doc rot": 0}
    for _, _, why in rows:
        for k in buckets:
            if f"[{k}]" in why:
                buckets[k] += 1
                break
    print(f"=== doc/body drift: {len(rows)} of {seen} factories with a parseable doc")
    print(f"    against the scryfall cache: "
          f"{buckets['BODY WRONG']} BODY WRONG (a real defect), "
          f"{buckets['doc rot']} doc rot; {skipped} the oracle could not "
          f"adjudicate (two-faced or `*`), dropped")
    rows.sort(key=lambda r: 0 if "[BODY WRONG]" in r[2] else 1)
    shown = rows if args.rows == 0 else rows[: args.rows]
    for name, path, why in shown:
        print(f"  {name:<44} {why}")
        print(f"  {'':<44} {path}")
    if len(shown) < len(rows):
        print(f"  … {len(rows) - len(shown)} more (use --rows 0)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
