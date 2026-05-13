#!/usr/bin/env python3
"""Audit STRIXHAVEN2.md "Strixhaven base set (STX)" status markers.

Sibling to `audit_strixhaven2.py` (which audits the SOS supplemental
set). This one checks the STX 2021 base set table at the bottom of
STRIXHAVEN2.md against the actual `crabomination/src/catalog/sets/stx/`
catalog tree.

Findings reported:
- Cards marked ✅/🟡 in STRIXHAVEN2.md but missing from
  `crabomination/src/catalog/sets/stx/` (false positives — promised but
  not implemented).
- Cards present in the catalog but marked ⏳ in the doc (false
  negatives — implemented but tracker is stale).
- Tally of statuses + a per-section breakdown.

Tracked in TODO.md push XXIX as a follow-up to the SOS audit script.
"""

import re
import sys
from pathlib import Path
from collections import defaultdict

REPO = Path(__file__).resolve().parent.parent
DOC = REPO / "STRIXHAVEN2.md"
STX_DIR = REPO / "crabomination" / "src" / "catalog" / "sets" / "stx"

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

# ── Collect catalog name strings ──────────────────────────────────────────
# Same approach as audit_strixhaven2.py: capture every double-quoted
# string in the catalog tree. Matches `name: "Foo"`, `bury_in_books("Foo")`
# style helpers, and any in-line string literal — the cross-check below
# filters against the doc's card-name list.
STRING_RE = re.compile(r'"((?:[^"\\]|\\.)*)"')
catalog_strings: set[str] = set()
for src in STX_DIR.glob("*.rs"):
    for line in src.read_text(encoding="utf-8").splitlines():
        for m in STRING_RE.finditer(line):
            catalog_strings.add(m.group(1))

# Some STX 2021 cards live in `decks/modern.rs` (e.g., Silverquill
# Apprentice, Brilliant Plan, Path of Peril) because they were added as
# part of cube/modern packages. Walk that file too so the audit doesn't
# false-positive on cards that exist but live outside the stx/ tree.
MODERN_FILE = REPO / "crabomination" / "src" / "catalog" / "sets" / "decks" / "modern.rs"
if MODERN_FILE.exists():
    for line in MODERN_FILE.read_text(encoding="utf-8").splitlines():
        for m in STRING_RE.finditer(line):
            catalog_strings.add(m.group(1))

# ── Parse STRIXHAVEN2.md STX-base-set tables ──────────────────────────────
SUBSECTION_RE = re.compile(r"^###\s+(.+?)\s*$")
ROW_RE = re.compile(
    r"^\|\s*(?P<name>.+?)\s*"
    r"\|\s*(?P<cost>.*?)\s*"
    r"\|\s*(?P<status>.+?)\s*"
    r"\|\s*(?P<notes>.+?)\s*"
    r"\|\s*$"
)

# STX 2021 sub-sections (per the "Strixhaven base set (STX)" section).
STX_SUBSECTIONS = {
    "Silverquill (W/B)", "Witherbloom (B/G)", "Lorehold (R/W)",
    "Quandrix (G/U)", "Prismari (U/R)",
    "Mono-color staples (`stx::mono`)", "Shared / multi-college",
    "Iconic / legendary (`stx::iconic` + `stx::legends`)",
}

per_section_status: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
status_by_card: dict[str, str] = {}
notes_by_card: dict[str, str] = {}
section_by_card: dict[str, str] = {}
in_stx_base = False
current_section = None

for line in DOC.read_text(encoding="utf-8").splitlines():
    if line.startswith("## Strixhaven base set"):
        in_stx_base = True
        continue
    if line.startswith("## ") and in_stx_base:
        # Ran past the STX section into a trailing top-level header.
        break
    if not in_stx_base:
        continue
    sub = SUBSECTION_RE.match(line)
    if sub:
        current_section = sub.group(1).strip()
        continue
    if current_section not in STX_SUBSECTIONS:
        continue
    if not line.startswith("|") or line.startswith("|---"):
        continue
    m = ROW_RE.match(line)
    if not m:
        continue
    name = m.group("name").replace("\\|", "|").strip()
    if name.lower() == "card":
        continue
    status = m.group("status").strip()
    notes = m.group("notes").strip()
    status_by_card[name] = status
    notes_by_card[name] = notes
    section_by_card[name] = current_section
    per_section_status[current_section][status] += 1


def is_implemented_status(s: str) -> bool:
    """Treat both ✅ and 🟡 as implemented (the body is wired even if a
    rider is omitted). ⏳ is the "not yet" status."""
    return "✅" in s or "🟡" in s


# ── Cross-check ────────────────────────────────────────────────────────────

doc_implemented = {n for n, s in status_by_card.items() if is_implemented_status(s)}
doc_todo = {n for n, s in status_by_card.items() if "⏳" in s}
doc_card_names = set(status_by_card)


def doc_name_matches_catalog(name: str) -> bool:
    """A doc row matches the catalog if either the full row name or any
    `//`-separated half (MDFC) is present in the catalog source."""
    if name in catalog_strings:
        return True
    for half in name.split("//"):
        if half.strip() in catalog_strings:
            return True
    return False


catalog_card_names = {n for n in doc_card_names if doc_name_matches_catalog(n)}

false_positives = sorted(doc_implemented - catalog_card_names)
real_false_negs = sorted(catalog_card_names & doc_todo)

print("=" * 70)
print("STRIXHAVEN2.md AUDIT — STX 2021 base-set catalog vs. doc status")
print("=" * 70)
print()
print(f"Doc rows (STX section): {len(status_by_card)}")
print(f"Doc-listed cards with a name-string in catalog: {len(catalog_card_names)}")
print(f"Marked implemented in doc (✅ or 🟡): {len(doc_implemented)}")
print(f"In both (doc-implemented AND in catalog): {len(doc_implemented & catalog_card_names)}")
print()

print("─" * 70)
print(f"FALSE POSITIVES — marked ✅/🟡 but NOT in catalog ({len(false_positives)})")
print("─" * 70)
if false_positives:
    for name in false_positives:
        st = status_by_card.get(name, "?")
        st_tag = "[OK]" if "✅" in st else "[PARTIAL]"
        print(f"  {st_tag} {name}  ({section_by_card.get(name, '?')})")
else:
    print("  (none — all ✅/🟡 cards have a catalog entry)")
print()

print("─" * 70)
print(f"FALSE NEGATIVES — in catalog but doc says ⏳ ({len(real_false_negs)})")
print("─" * 70)
if real_false_negs:
    for name in real_false_negs:
        print(f"  [TODO->should-update] {name}")
else:
    print("  (none — every catalog card has ✅/🟡 status)")
print()

print("─" * 70)
print("PER-SECTION STATUS BREAKDOWN")
print("─" * 70)
print(f"{'Section':<48} {'OK':>5} {'PRT':>5} {'TODO':>5} {'TOT':>5}")
total_ok = total_prt = total_todo = total_total = 0
for sec in [
    "Silverquill (W/B)", "Witherbloom (B/G)", "Lorehold (R/W)",
    "Quandrix (G/U)", "Prismari (U/R)",
    "Mono-color staples (`stx::mono`)", "Shared / multi-college",
    "Iconic / legendary (`stx::iconic` + `stx::legends`)",
]:
    counts = per_section_status[sec]
    # A status string that mixes ✅ and 🟡 (e.g. "✅ ← 🟡") counts as
    # ✅ — the leftmost emoji is the current state, the right-of-arrow
    # is historical. Same convention as the SOS audit script.
    ok = sum(v for k, v in counts.items() if "✅" in k)
    prt = sum(v for k, v in counts.items() if "🟡" in k and "✅" not in k)
    todo = sum(v for k, v in counts.items() if "⏳" in k and "✅" not in k and "🟡" not in k)
    tot = ok + prt + todo
    total_ok += ok
    total_prt += prt
    total_todo += todo
    total_total += tot
    print(f"{sec:<48} {ok:>5} {prt:>5} {todo:>5} {tot:>5}")
print("─" * 70)
print(f"{'TOTAL':<48} {total_ok:>5} {total_prt:>5} {total_todo:>5} {total_total:>5}")
print()

if false_positives or real_false_negs:
    sys.exit(1)
sys.exit(0)
