#!/usr/bin/env python3
"""Regenerate the SOS card tables in STRIXHAVEN2.md from fresh Scryfall data.

Preserves all hand-edited content:
- Everything *above* the first SOS color section (`## White`).
- Everything *below* the last SOS color section (`## Colorless`), which is
  the STX section and any trailing engine notes.
- Per-card **Status** and **Notes** columns of every card row whose name
  matches between the old and new tables. New cards default to ⏳ + heuristic.

Only the Mana Cost / Type / P/T / Oracle Text columns get refreshed from
Scryfall — those reflect the printed card and shouldn't drift.

The script expects two paginated dumps from Scryfall's `set:sos` query:
`sos_p1.json` and `sos_p2.json`, both at the repo root. They are not
checked in (the cache lives in `scripts/.scryfall_cache.json`); to
regenerate them run:

    curl -s 'https://api.scryfall.com/cards/search?q=set:sos&page=1' \
      -o sos_p1.json
    curl -s 'https://api.scryfall.com/cards/search?q=set:sos&page=2' \
      -o sos_p2.json

If either file is missing the script exits cleanly with a hint rather
than crashing — so re-running locally without first re-fetching is
non-destructive.
"""

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
OUT_PATH = REPO / "STRIXHAVEN2.md"

# ── Load Scryfall data ─────────────────────────────────────────────────────


def _load(path: Path) -> list:
    if not path.exists():
        print(
            f"[skip] {path.name} not found at {path}.\n"
            "Re-fetch with `curl 'https://api.scryfall.com/cards/search?"
            "q=set:sos&page=N' -o sos_pN.json` and rerun.",
            file=sys.stderr,
        )
        sys.exit(0)
    return json.loads(path.read_text(encoding="utf-8")).get("data") or []


p1 = _load(REPO / "sos_p1.json")
p2 = _load(REPO / "sos_p2.json")
all_cards = sorted(p1 + p2, key=lambda c: c["name"])

# ── Section ordering (Strixhaven colleges + mono + colorless) ──────────────

SECTION_ORDER = {
    ("W",):    (0,  "White"),
    ("U",):    (1,  "Blue"),
    ("B",):    (2,  "Black"),
    ("R",):    (3,  "Red"),
    ("G",):    (4,  "Green"),
    ("R","U"): (5,  "Prismari (Blue-Red)"),
    ("B","G"): (6,  "Witherbloom (Black-Green)"),
    ("B","W"): (7,  "Silverquill (White-Black)"),
    ("G","U"): (8,  "Quandrix (Green-Blue)"),
    ("R","W"): (9,  "Lorehold (Red-White)"),
    ():        (10, "Colorless"),
}

SOS_SECTION_NAMES = {name for _, name in SECTION_ORDER.values()}

def section_of(card):
    ci = tuple(sorted(card.get("color_identity", [])))
    return SECTION_ORDER.get(ci, (11, "Other"))

by_section = defaultdict(list)
for c in all_cards:
    key = section_of(c)
    by_section[key].append(c)

# ── Card field helpers ─────────────────────────────────────────────────────

def mana(c):
    return c.get("mana_cost") or ""

def type_line(c):
    return c.get("type_line", "")

def oracle(c):
    """Format oracle text for the per-card row.

    Multi-line oracle blocks get joined with " / " so the row stays on one
    table line. Full oracle text preserved verbatim — earlier revisions
    truncated to 220 / 600 chars, which silently dropped late keywords.
    """
    return (c.get("oracle_text") or "").replace("\n", " / ")

def pt(c):
    if "power" in c and "toughness" in c:
        return f"{c['power']}/{c['toughness']}"
    if "loyalty" in c:
        return f"[{c['loyalty']}]"
    return ""

# ── Implementation-note heuristic (default Notes for unseen cards) ─────────

COMPLEX_KWS = {
    "Cascade", "Dredge", "Madness", "Morph", "Megamorph", "Ninjutsu",
    "Splice", "Storm", "Suspend", "Transmute", "Vanishing", "Fading",
    "Buyback", "Echo", "Entwine", "Kicker", "Multikicker", "Replicate",
    "Retrace", "Overload", "Cipher", "Bestow", "Tribute", "Constellation",
    "Strive", "Ferocious", "Threshold", "Hellbent", "Chroma",
    "Metalcraft", "Scavenge", "Populate", "Battalion", "Bloodrush",
    "Extort", "Evolve", "Unleash", "Detain", "Devotion", "Heroic",
    "Renown", "Exploit", "Dash", "Rebound", "Emerge", "Escalate",
    "Meld", "Investigate", "Crew", "Fabricate", "Improvise", "Revolt",
    "Embalm", "Eternalize", "Explore", "Enrage", "Ascend", "Discover",
    # Surveil is a first-class engine primitive (`Effect::Surveil`); do
    # not flag cards with surveil text as needing engine work.
    "Spectacle", "Adapt", "Afterlife", "Addendum",
    "Jump-start", "Mentor", "Proliferate", "Riot", "Undergrowth",
    "Amass", "Escape", "Mutate", "Companion", "Foretell", "Boast",
    "Learn", "Magecraft", "Ward", "Cleave", "Disturb", "Decayed",
    "Reconfigure", "Blitz", "Casualty", "Connive", "Hideaway",
    "Read ahead", "Enlist", "Backup", "Encore", "Prototype", "Squad",
    "Training", "Bargain", "Celebration", "Expend", "Plot", "Saddle",
    "Spree", "Valiant", "Gift", "Impending", "Offspring", "Disguise",
    "Commit", "Gleam",
    "Repartee", "Increment", "Opus", "Infusion", "Paradigm", "Converge",
    "Prepare",
}

def impl_note(c):
    oracle_text = (c.get("oracle_text") or "").lower()
    kws = c.get("keywords", [])
    tl = type_line(c).lower()
    hints = []

    for kw in kws:
        if kw in COMPLEX_KWS:
            hints.append(f"{kw} keyword primitive")

    # SOS-specific oracle-text patterns.
    if "repartee —" in oracle_text:
        hints.append("spell-targets-creature predicate (Repartee)")
    if "magecraft —" in oracle_text:
        hints.append("Magecraft trigger (cast-or-copy instant/sorcery)")
    if "increment (" in oracle_text:
        hints.append("Increment (mana-spent vs. P/T self-counter)")
    if "opus —" in oracle_text:
        hints.append("Opus (≥5-mana-spent gate on cast)")
    if "infusion —" in oracle_text:
        hints.append("Infusion (life-gained-this-turn gate)")
    if "paradigm (" in oracle_text:
        hints.append("Paradigm (exile-self / cast-copy follow-up)")
    if "converge" in oracle_text:
        hints.append("Converge (distinct-colors-paid value)")
    if "casualty " in oracle_text:
        hints.append("Casualty (sac-on-cast copy)")
    if "prepared" in oracle_text or "prepare " in oracle_text:
        hints.append("Prepare keyword primitive (prepared-state toggle)")

    if "copy" in oracle_text:
        hints.append("copy-spell/permanent primitive")
    if "exile" in oracle_text and "cast" in oracle_text:
        hints.append("cast-from-exile pipeline")
    if "from your graveyard" in oracle_text and "cast" in oracle_text:
        hints.append("cast-from-graveyard")
    if "transform" in oracle_text:
        hints.append("transform/DFC primitive")
    if "populate" in oracle_text:
        hints.append("populate (token-copy) primitive")
    if "saga" in tl:
        hints.append("Saga lore-counter mechanism")
    if "adventure" in tl:
        hints.append("Adventure cast mode")
    if "vehicle" in tl:
        hints.append("Vehicle crew primitive")
    if "commander" in oracle_text or "command zone" in oracle_text:
        hints.append("commander-zone awareness")

    if not hints:
        return "Standard primitives — should be straightforward to wire."
    return "Needs: " + "; ".join(dict.fromkeys(hints)) + "."


# ── Parse existing STRIXHAVEN2.md to harvest hand-edited Status / Notes ────


SECTION_HEADER_RE = re.compile(r"^##\s+(.+?)\s*$")
ROW_RE = re.compile(
    r"^\|\s*(?P<name>.+?)\s*"        # name
    r"\|\s*(?P<mana>.*?)\s*"          # mana
    r"\|\s*(?P<typ>.*?)\s*"           # type
    r"\|\s*(?P<pt>.*?)\s*"            # P/T
    r"\|\s*(?P<oracle>.*?)\s*"        # oracle
    r"\|\s*(?P<status>.+?)\s*"        # status
    r"\|\s*(?P<notes>.+?)\s*"         # notes
    r"\|\s*$"
)


def parse_existing(path: Path):
    """Return (prefix_lines, suffix_lines, per_card_overrides).

    `prefix_lines` is everything BEFORE the first SOS section header.
    `suffix_lines` is everything from the first non-SOS section after
    the SOS sections to EOF.
    `per_card_overrides` maps card name → (status, notes).
    """
    if not path.exists():
        return [], [], {}

    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()

    overrides = {}
    in_sos_section = False
    seen_sos = False
    first_sos_idx = None
    last_sos_end_idx = None
    current_section = None

    for i, line in enumerate(lines):
        m = SECTION_HEADER_RE.match(line)
        if m:
            sec_name = m.group(1).strip()
            if sec_name in SOS_SECTION_NAMES:
                if first_sos_idx is None:
                    first_sos_idx = i
                seen_sos = True
                in_sos_section = True
                current_section = sec_name
            else:
                if in_sos_section:
                    # First non-SOS section after SOS block — end of SOS region.
                    last_sos_end_idx = i
                in_sos_section = False
                current_section = sec_name

        # Parse rows inside SOS sections only.
        if in_sos_section and line.startswith("|") and not line.startswith("|---"):
            row_match = ROW_RE.match(line)
            if row_match:
                name = row_match.group("name").replace("\\|", "|")
                # Skip the header row "Card | Mana Cost | Type | …"
                if name.lower() == "card":
                    continue
                status = row_match.group("status").strip()
                notes = row_match.group("notes").strip()
                overrides[name] = (status, notes)

    if first_sos_idx is None:
        # File exists but has no SOS sections — treat entire file as prefix.
        return lines, [], overrides

    if last_sos_end_idx is None:
        # SOS sections run to end of file.
        last_sos_end_idx = len(lines)

    prefix = lines[:first_sos_idx]
    suffix = lines[last_sos_end_idx:]
    return prefix, suffix, overrides


prefix_lines, suffix_lines, overrides = parse_existing(OUT_PATH)


# ── Generate markdown ──────────────────────────────────────────────────────

def escape(s):
    return s.replace("|", "\\|")

new_section_lines = []

for key in sorted(by_section):
    _, sec_name = key
    cards = by_section[key]

    new_section_lines += [
        f"## {sec_name}",
        "",
        "| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |",
        "|---|---|---|---|---|---|---|",
    ]

    for c in cards:
        name = c["name"]
        existing = overrides.get(name)
        if existing:
            status, notes = existing
        else:
            status, notes = "⏳", impl_note(c)

        row = (
            f"| {escape(name)} "
            f"| {escape(mana(c))} "
            f"| {escape(type_line(c))} "
            f"| {pt(c)} "
            f"| {escape(oracle(c))} "
            f"| {status} "
            f"| {notes} |"
        )
        new_section_lines.append(row)

    new_section_lines.append("")


# Stitch prefix + new tables + suffix.
if prefix_lines:
    out_lines = list(prefix_lines)
    if out_lines and out_lines[-1] != "":
        out_lines.append("")
else:
    # No prefix: emit a default header.
    out_lines = [
        "# Secrets of Strixhaven — Implementation Plan",
        "",
        "All **255** first-print cards from Scryfall set `sos` (*Secrets of Strixhaven*).",
        "Card definitions are pulled directly from Scryfall and included inline.",
        "",
        "## Legend",
        "",
        "- ✅ done — wired in `crate::catalog` with full functionality",
        "- 🟡 partial — exists with simplified or stub effect; key behavior missing",
        "- ⏳ todo — not yet implemented",
        "",
    ]

out_lines.extend(new_section_lines)

if suffix_lines:
    if out_lines and out_lines[-1] != "":
        out_lines.append("")
    out_lines.extend(suffix_lines)

# Trim trailing blank lines, leave exactly one terminating newline.
while out_lines and out_lines[-1].strip() == "":
    out_lines.pop()
out_lines.append("")

OUT_PATH.write_text("\n".join(out_lines), encoding="utf-8")

# ── Stats ──────────────────────────────────────────────────────────────────

statuses = defaultdict(int)
unmatched = []
matched = 0
new_cards = []
for c in all_cards:
    if c["name"] in overrides:
        st, _ = overrides[c["name"]]
        statuses[st] += 1
        matched += 1
    else:
        statuses["⏳"] += 1
        new_cards.append(c["name"])

# Map emoji to ASCII tags for console-safe output.
def _ascii(s: str) -> str:
    return (
        s.replace("✅", "[OK]")
         .replace("\U0001f7e1", "[PARTIAL]")
         .replace("⏳", "[TODO]")
         .replace("\U0001f50d", "[REVIEW]")
    )

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
print(f"Written {OUT_PATH} -- {len(out_lines)} lines, {len(all_cards)} SOS cards")
print("Status mix: " + ", ".join(
    f"{_ascii(k)}={v}" for k, v in sorted(statuses.items(), key=lambda x: -x[1])
))
print(f"Preserved {matched}/{len(all_cards)} hand edits.")
if new_cards:
    print(f"New cards (defaulted to TODO): {new_cards}")

new_names = {c["name"] for c in all_cards}
removed = sorted(set(overrides) - new_names)
if removed:
    print(f"WARNING: dropped from regenerated file (no Scryfall match): {removed}")
