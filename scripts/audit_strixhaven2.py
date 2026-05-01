#!/usr/bin/env python3
"""Audit STRIXHAVEN2.md status markers against the actual SOS catalog.

Findings reported:
- Cards marked OK/PARTIAL in STRIXHAVEN2.md but missing from
  `crabomination/src/catalog/sets/sos/` (false positives — promised but
  not implemented).
- Cards present in the catalog but marked TODO in the doc (false
  negatives — implemented but tracker is stale).
- Tally of statuses + a per-section breakdown.
"""

import re
import sys
from pathlib import Path
from collections import defaultdict

REPO = Path(__file__).resolve().parent.parent
DOC = REPO / "STRIXHAVEN2.md"
SOS_DIR = REPO / "crabomination" / "src" / "catalog" / "sets" / "sos"

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

# ── Collect catalog names ──────────────────────────────────────────────────
# Some cards use `name: "Foo"` directly inside a `CardDefinition` literal;
# others are wired through helpers like `school_land("Foo", ...)` where the
# name is passed as the first argument. Capture any double-quoted string
# that appears anywhere in sos/*.rs and let the cross-check filter against
# the doc's card name list.
STRING_RE = re.compile(r'"((?:[^"\\]|\\.)*)"')
catalog_strings = set()
for src in SOS_DIR.glob("*.rs"):
    for line in src.read_text(encoding="utf-8").splitlines():
        for m in STRING_RE.finditer(line):
            catalog_strings.add(m.group(1))

# ── Parse STRIXHAVEN2.md SOS tables ────────────────────────────────────────
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

SOS_SECTIONS = {
    "White", "Blue", "Black", "Red", "Green",
    "Prismari (Blue-Red)", "Witherbloom (Black-Green)",
    "Silverquill (White-Black)", "Quandrix (Green-Blue)",
    "Lorehold (Red-White)", "Colorless",
}

per_section_status = defaultdict(lambda: defaultdict(int))
status_by_card = {}
notes_by_card = {}
section_by_card = {}
current_section = None

for line in DOC.read_text(encoding="utf-8").splitlines():
    sec = SECTION_RE.match(line)
    if sec:
        current_section = sec.group(1).strip()
        continue
    if current_section not in SOS_SECTIONS:
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
    return "✅" in s or "🟡" in s


# ── Cross-check ────────────────────────────────────────────────────────────

doc_implemented = {n for n, s in status_by_card.items() if is_implemented_status(s)}
doc_todo = {n for n, s in status_by_card.items() if "⏳" in s}

# A "catalog name" is a doc-listed card whose name string appears literally
# anywhere in the SOS source (covers `name: "Foo"`, `school_land("Foo", …)`,
# token-helpers, comments, etc.). This is conservative: it confirms the
# string is present, not that the card factory works — but missing strings
# are a strong negative signal.
doc_card_names = set(status_by_card)


def doc_name_matches_catalog(name: str) -> bool:
    """A doc row matches the catalog if either the full row name or any
    `//`-separated half is present in the catalog source. Lets MDFC rows
    (`Studious First-Year // Rampant Growth`) match a single-face
    factory under just one of the two halves (the front face is the
    canonical factory; the back face's CardDefinition is nested
    inside it via `back_face: Some(...)`)."""
    if name in catalog_strings:
        return True
    for half in name.split("//"):
        if half.strip() in catalog_strings:
            return True
    return False


catalog_card_names = {n for n in doc_card_names if doc_name_matches_catalog(n)}

# False positives: marked implemented in doc but no name-string in catalog.
false_positives = sorted(doc_implemented - catalog_card_names)
# False negatives: name-string is in catalog but doc still says ⏳.
real_false_negs = sorted(catalog_card_names & doc_todo)
# Strings present in catalog files that aren't in the doc — usually token
# definitions or helper labels; not strictly orphans but worth flagging.
catalog_extra = sorted(catalog_strings - doc_card_names)

print("=" * 70)
print("STRIXHAVEN2.md AUDIT — SOS catalog vs. doc status")
print("=" * 70)
print()
print(f"Doc rows: {len(status_by_card)}")
print(f"Doc-listed cards with a name-string in catalog: {len(catalog_card_names)}")
print(f"Marked implemented in doc (OK or PARTIAL): {len(doc_implemented)}")
print(f"In both (doc-implemented AND in catalog): {len(doc_implemented & catalog_card_names)}")
print()

print("─" * 70)
print(f"FALSE POSITIVES — marked OK/PARTIAL but NOT in catalog ({len(false_positives)})")
print("─" * 70)
if false_positives:
    for name in false_positives:
        st = status_by_card.get(name, "?")
        st_tag = "[OK]" if "✅" in st else "[PARTIAL]"
        print(f"  {st_tag} {name}")
else:
    print("  (none — all OK/PARTIAL cards have a catalog entry)")
print()

print("─" * 70)
print(f"FALSE NEGATIVES — in catalog but doc says TODO ({len(real_false_negs)})")
print("─" * 70)
if real_false_negs:
    for name in real_false_negs:
        print(f"  [TODO->should-update] {name}")
else:
    print("  (none — every catalog card has OK/PARTIAL status)")
print()

# Per-section summary
print("─" * 70)
print("PER-SECTION STATUS BREAKDOWN")
print("─" * 70)
print(f"{'Section':<30} {'OK':>5} {'PRT':>5} {'TODO':>5} {'TOT':>5}")
total_ok = total_prt = total_todo = total_total = 0
for sec in [
    "White", "Blue", "Black", "Red", "Green",
    "Prismari (Blue-Red)", "Witherbloom (Black-Green)",
    "Silverquill (White-Black)", "Quandrix (Green-Blue)",
    "Lorehold (Red-White)", "Colorless",
]:
    counts = per_section_status.get(sec, {})
    ok = sum(v for k, v in counts.items() if "✅" in k)
    prt = sum(v for k, v in counts.items() if "🟡" in k)
    todo = sum(v for k, v in counts.items() if "⏳" in k)
    tot = ok + prt + todo
    total_ok += ok
    total_prt += prt
    total_todo += todo
    total_total += tot
    print(f"{sec:<30} {ok:>5} {prt:>5} {todo:>5} {tot:>5}")
print("-" * 56)
print(f"{'TOTAL':<30} {total_ok:>5} {total_prt:>5} {total_todo:>5} {total_total:>5}")
print()

# Doc claims in the "Implementation Progress" header
print("─" * 70)
print("HEADER CLAIMS vs. ACTUAL")
print("─" * 70)
doc_text = DOC.read_text(encoding="utf-8")
claim_re = re.compile(r"-\s+(✅|🟡|⏳)\s+\S+:\s*(\d+)")
for line in doc_text.splitlines()[:60]:
    m = claim_re.match(line)
    if m:
        sym, n = m.group(1), int(m.group(2))
        actual = (
            total_ok if "✅" in sym else
            total_prt if "🟡" in sym else
            total_todo
        )
        sym_tag = "[OK]" if "✅" in sym else "[PARTIAL]" if "🟡" in sym else "[TODO]"
        delta = actual - n
        flag = " (drift)" if delta != 0 else ""
        print(f"  Header: {sym_tag} = {n}  | Actual: {actual}  | delta {delta:+d}{flag}")
