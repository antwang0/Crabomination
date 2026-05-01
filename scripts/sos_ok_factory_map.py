#!/usr/bin/env python3
"""For every ✅ SOS card, find its catalog factory function name.

Walks `crabomination/src/catalog/sets/sos/*.rs` looking for `pub fn xxx()`
followed by a `name: "Card Name"` literal OR a helper-function call with
"Card Name" as the first string argument. Outputs the (card name → factory
fn name) mapping grouped by section, suitable for hand-pasting into a Rust
module.
"""

import re
import sys
from pathlib import Path
from collections import defaultdict

REPO = Path(__file__).resolve().parent.parent
DOC = REPO / "STRIXHAVEN2.md"
SOS_DIR = REPO / "crabomination" / "src" / "catalog" / "sets" / "sos"

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

# Build (factory_fn_name, card_name_string) pairs from each .rs file.
FN_RE = re.compile(r"pub\s+fn\s+(\w+)\s*\(\s*\)\s*->\s*CardDefinition")
NAME_RE = re.compile(r'"((?:[^"\\]|\\.)+)"')

card_to_fn = {}
for src in sorted(SOS_DIR.glob("*.rs")):
    text = src.read_text(encoding="utf-8")
    # Find every fn definition and the next quoted string after it
    pos = 0
    while True:
        m = FN_RE.search(text, pos)
        if not m:
            break
        fn_name = m.group(1)
        # Find the first quoted string after this fn (skip fn name itself)
        body_start = m.end()
        # Stop at the next `pub fn` to avoid bleeding across cards
        next_fn = FN_RE.search(text, body_start)
        body_end = next_fn.start() if next_fn else len(text)
        body = text[body_start:body_end]
        # First quoted string is usually the card name
        nm = NAME_RE.search(body)
        if nm:
            card_name = nm.group(1)
            card_to_fn[card_name] = fn_name
        pos = body_end

# Also catalog the school_land helper which uses the *first arg* as the name.
# (Already covered by the "first quoted string" heuristic above.)

# ── Read OK cards from STRIXHAVEN2.md ──────────────────────────────────────
SECTION_RE = re.compile(r"^##\s+(.+?)\s*$")
ROW_RE = re.compile(
    r"^\|\s*(?P<name>.+?)\s*"
    r"\|\s*(?P<mana>.*?)\s*"
    r"\|\s*(?P<typ>.*?)\s*"
    r"\|\s*(?P<pt>.*?)\s*"
    r"\|\s*(?P<oracle>.*?)\s*"
    r"\|\s*(?P<status>.+?)\s*"
    r"\|\s*(?P<notes>.+?)\s*"
    r"\|\s*$"
)

SOS_SECTIONS = [
    "White", "Blue", "Black", "Red", "Green",
    "Prismari (Blue-Red)", "Witherbloom (Black-Green)",
    "Silverquill (White-Black)", "Quandrix (Green-Blue)",
    "Lorehold (Red-White)", "Colorless",
]

ok_by_section = defaultdict(list)
current = None
for line in DOC.read_text(encoding="utf-8").splitlines():
    sm = SECTION_RE.match(line)
    if sm:
        current = sm.group(1).strip()
        continue
    if current not in SOS_SECTIONS:
        continue
    if not line.startswith("|") or line.startswith("|---"):
        continue
    rm = ROW_RE.match(line)
    if not rm:
        continue
    name = rm.group("name").strip()
    if name.lower() == "card":
        continue
    if "✅" in rm.group("status"):
        ok_by_section[current].append(name)

# ── Output ─────────────────────────────────────────────────────────────────

missing = []
for sec in SOS_SECTIONS:
    cards = ok_by_section[sec]
    if not cards:
        continue
    print(f"\n// {sec} ({len(cards)})")
    for name in cards:
        fn = card_to_fn.get(name)
        if fn:
            print(f"    {fn},  // {name}")
        else:
            print(f"    // MISSING: {name}")
            missing.append((sec, name))

print(f"\nTotal OK with factory: {sum(len(v) for v in ok_by_section.values()) - len(missing)}")
if missing:
    print(f"\nMISSING factories ({len(missing)}):")
    for sec, name in missing:
        print(f"  [{sec}] {name}")
