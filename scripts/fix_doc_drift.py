#!/usr/bin/env python3
"""Rewrite the doc comments `audit_doc_drift.py` classifies as **doc rot**.

The auditor's two verdicts are not symmetrical. `BODY WRONG` means the
oracle backs the doc and the body is a defect — that is card work and is
never touched here. `doc rot` means the oracle backs the *body*: the
characteristics were corrected at some point and the comment above them was
not, so the comment now describes a card that does not exist. This script
fixes only that direction, and only the two fields the auditor adjudicates
against Scryfall — the mana cost run after the em dash, and the single
`N/M` power/toughness pair.

Why bother with comments: the doc comment is the **tell** this codebase uses
to find body bugs. Greasewrench Goblin was found because its doc said `{1}{R}`
2/2 and its body said `{R}` 2/1. With 338 stale docs in the tree that signal
is noise, and every future drift lands in a list nobody can read. Fixing the
class once turns the auditor's count into a live alarm.

    python3 scripts/fix_doc_drift.py --dry-run   # what it would rewrite
    python3 scripts/fix_doc_drift.py             # rewrite

⚠ It rewrites only what it can place unambiguously. If the doc's cost run or
its `N/M` does not occur exactly once in the raw comment block, the row is
skipped and printed — a doc that spells the same cost twice needs a human to
decide which occurrence is the card's own.
"""
import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from audit_doc_drift import (  # noqa: E402
    BODY_COST,
    BODY_NAME,
    BODY_POWER,
    BODY_TOUGH,
    CATALOG,
    DOC_COST,
    DOC_PT,
    FN,
    ROOT,
    body_cost_symbols,
    card_literal,
    doc_cost_symbols,
    factory_body,
    norm,
    oracle_cost,
    oracle_pt,
)


def spell(symbols) -> str:
    return "".join("{" + s + "}" for s in symbols)


def scan_file(path: Path):
    """`(doc_line_range, [(old, new)])` for every doc-rot factory in `path`."""
    src = path.read_text(encoding="utf-8")
    lines = src.split("\n")
    out = []
    for m in FN.finditer(src):
        line_no = src[: m.start()].count("\n")
        doc = []
        i = line_no - 1
        while i >= 0 and (
            lines[i].lstrip().startswith("///") or lines[i].lstrip().startswith("#[")
        ):
            if lines[i].lstrip().startswith("///"):
                doc.append(lines[i].lstrip()[3:].strip())
            i -= 1
        if not doc:
            continue
        doc_first, doc_last = i + 1, line_no - 1
        doc_text = " ".join(reversed(doc))
        body = factory_body(src, m.end())
        bn = BODY_NAME.search(body)
        card_name = bn.group(1) if bn else None
        if bn:
            body = card_literal(body, bn.start())
        head = doc_text.split(" — ")[0].split(" – ")[0].strip()
        if not card_name or norm(card_name) != norm(head):
            continue

        subs = []
        dc, bc = DOC_COST.search(doc_text), BODY_COST.search(body)
        if dc and bc:
            d, b = doc_cost_symbols(dc.group(1)), body_cost_symbols(bc.group(1))
            if d and b and sorted(d) != sorted(b) and oracle_cost(card_name) == b:
                subs.append((spell(d), spell(b)))
        bp, bt = BODY_POWER.search(body), BODY_TOUGH.search(body)
        if bp or bt:
            pts = DOC_PT.findall(doc_text)
            if len(pts) == 1:
                dp, dt = int(pts[0][0]), int(pts[0][1])
                p = int(bp.group(1)) if bp else 0
                t = int(bt.group(1)) if bt else 0
                if (dp, dt) != (p, t) and oracle_pt(card_name) == (p, t):
                    subs.append((f"{dp}/{dt}", f"{p}/{t}"))
        if subs:
            out.append((m.group(1), doc_first, doc_last, subs))
    return src, lines, out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    rewritten, skipped = 0, []
    for path in sorted(CATALOG.rglob("*.rs")):
        src, lines, rows = scan_file(path)
        if not rows:
            continue
        changed = False
        for fname, first, last, subs in rows:
            block = "\n".join(lines[first : last + 1])
            new_block = block
            ok = True
            for old, new in subs:
                # `\b` around the P/T pair so `3/1` does not match inside
                # `13/1`; the cost run is already self-delimiting.
                pat = rf"\b{re.escape(old)}\b" if "/" in old else re.escape(old)
                hits = len(re.findall(pat, new_block))
                if hits != 1:
                    skipped.append(f"{path.relative_to(ROOT)}::{fname} "
                                   f"({old} -> {new}: {hits} occurrences)")
                    ok = False
                    break
                new_block = re.sub(pat, new, new_block)
            if not ok or new_block == block:
                continue
            lines[first : last + 1] = new_block.split("\n")
            rewritten += 1
            changed = True
        if changed and not args.dry_run:
            path.write_text("\n".join(lines), encoding="utf-8")

    print(f"{'would rewrite' if args.dry_run else 'rewrote'} {rewritten} doc comment(s)")
    if skipped:
        print(f"skipped {len(skipped)} the substitution could not place uniquely:")
        for s in skipped[:20]:
            print(f"  {s}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
