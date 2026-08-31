#!/usr/bin/env python3
"""Does a card carry exactly the *mechanic* keywords it prints?

`audit_catalog_stats.py` checks keywords too, but only the ~20 evergreen ones
it lists in `ENGINE_KW`, and only what sits in `CardDefinition.keywords`. Two
whole populations fall through that gate:

* **mechanic keywords** — Morph, Flashback, Escape, Madness, Bestow, Cycling,
  Kicker, Suspend, … — which live in `keywords` and are never compared;
* **keywords stored in their own field** — `foretell_cost`, `plot_cost`,
  `buyback_additional_cost`, `entwine_additional_cost`, `splice_extra_cost` —
  which are not in `keywords` at all.

Both directions matter and both have found real defects. Daru Lancer shipped
Plainscycling {2} and no Morph, where the card prints Morph {2}{W}{W} and no
cycling; Mistwalker, Beskir Shieldmate and Ravenous Lindwurm each shipped a
Foretell the card does not print.

  python3 scripts/audit_keyword_drift.py             # both directions
  python3 scripts/audit_keyword_drift.py --invented  # body has, card does not
  python3 scripts/audit_keyword_drift.py --missing   # card prints, body lacks

⚠ Reminder text is stripped before the oracle is searched, or every Reach
creature reads as missing Flying ("can block creatures with flying").
⚠ Multi-face cards are skipped: the body splits over `back_face` and the
comparison would need to follow it.
"""
import argparse
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crabomination_catalog" / "src"
CACHE = json.load(open(pathlib.Path(__file__).resolve().parent / ".scryfall_cache.json"))
CACHE_LC = {k.lower(): v for k, v in CACHE.items() if isinstance(v, dict)}

# `Keyword::Variant` -> the word the card prints. Evergreen keywords that
# `audit_catalog_stats` already checks are deliberately absent.
KW_WORD = {
    "Morph": "morph", "Megamorph": "megamorph", "Disguise": "disguise",
    "Cycling": "cycling", "Landcycling": "cycling", "TypeCycling": "cycling",
    "Flashback": "flashback", "Escape": "escape", "Bestow": "bestow",
    "Madness": "madness", "Dash": "dash", "Kicker": "kicker",
    "Multikicker": "kicker", "Evoke": "evoke", "Unearth": "unearth",
    "Embalm": "embalm", "Eternalize": "eternalize", "Prowl": "prowl",
    "Ninjutsu": "ninjutsu", "Suspend": "suspend", "Overload": "overload",
    "Retrace": "retrace", "Miracle": "miracle", "Soulbond": "soulbond",
    "Soulshift": "soulshift", "Modular": "modular", "Graft": "graft",
    "Sunburst": "sunburst", "Affinity": "affinity", "Convoke": "convoke",
    "Delve": "delve", "Improvise": "improvise", "Emerge": "emerge",
    "Prototype": "prototype", "Disturb": "disturb", "Blitz": "blitz",
    "Casualty": "casualty", "Cleave": "cleave", "Channel": "channel",
    "Boast": "boast", "Impending": "impending", "Warp": "warp",
    "Spree": "spree", "Adapt": "adapt", "Monstrosity": "monstrosity",
    "Outlast": "outlast", "Renown": "renown", "Exploit": "exploit",
    "Awaken": "awaken", "Surge": "surge", "Escalate": "escalate",
    "Mutate": "mutate", "Forecast": "forecast", "Rebound": "rebound",
    "Bloodthirst": "bloodthirst", "Devour": "devour",
    "Annihilator": "annihilator", "Infect": "infect", "Wither": "wither",
    "Persist": "persist", "Undying": "undying", "Evolve": "evolve",
    "Extort": "extort", "Battalion": "battalion", "Bloodrush": "bloodrush",
    "Cipher": "cipher", "Dredge": "dredge", "Haunt": "haunt",
    "Storm": "storm", "Replicate": "replicate", "Conspire": "conspire",
    "Rampage": "rampage", "Bushido": "bushido", "Flanking": "flanking",
    "Shadow": "shadow", "Provoke": "provoke", "Amplify": "amplify",
    "Absorb": "absorb", "Vanishing": "vanishing", "Fading": "fading",
    "Echo": "echo", "Phasing": "phasing", "Banding": "banding",
    "Exalted": "exalted", "Frenzy": "frenzy", "Poisonous": "poisonous",
    "Toxic": "toxic", "Skulk": "skulk", "Crew": "crew",
    "Afflict": "afflict", "Ingest": "ingest", "Myriad": "myriad",
    "Riot": "riot", "Mentor": "mentor", "Afterlife": "afterlife",
    "Spectacle": "spectacle", "Adamant": "adamant", "Foretell": "foretell",
}
# The same question for the mechanics stored outside `keywords`.
FIELD_WORD = {
    "foretell_cost": "foretell",
    "plot_cost": "plot",
    "buyback_additional_cost": "buyback",
    "entwine_additional_cost": "entwine",
    "splice_extra_cost": "splice",
}

FN = re.compile(r"^pub fn ([a-z0-9_]+)\(\) -> CardDefinition \{", re.M)
BODY_NAME = re.compile(r'^ {8}name: "([^"]*)"', re.M)
REMINDER = re.compile(r"\([^)]*\)")


def oracle_words(name: str):
    """Printed text, reminder text stripped, or None when uncheckable."""
    c = CACHE_LC.get(name.lower())
    if not c or c.get("card_faces"):
        return None
    txt = (c.get("oracle_text") or "") + " " + (c.get("type_line") or "")
    return REMINDER.sub(" ", txt).lower()


def top_level_keywords(body: str):
    """The `keywords: vec![…]` at eight spaces, as variant names."""
    m = re.search(r"^ {8}keywords: vec!\[", body, re.M)
    if not m:
        return []
    i = body.index("[", m.start())
    depth, j = 0, i
    while j < len(body):
        if body[j] in "[({":
            depth += 1
        elif body[j] in "])}":
            depth -= 1
            if depth == 0:
                break
        j += 1
    return re.findall(r"Keyword::([A-Za-z]+)", body[i : j + 1])


def scan():
    invented, missing, checked = [], [], 0
    for path in sorted(CATALOG.rglob("*.rs")):
        src = path.read_text(encoding="utf-8")
        for m in FN.finditer(src):
            nxt = FN.search(src, m.end())
            body = src[m.end() : nxt.start() if nxt else len(src)]
            bn = BODY_NAME.search(body)
            if not bn:
                continue
            words = oracle_words(bn.group(1))
            if words is None:
                continue
            checked += 1
            have = {KW_WORD[k] for k in top_level_keywords(body) if k in KW_WORD}
            for field, word in FIELD_WORD.items():
                if re.search(rf"^ {{8}}{field}: Some", body, re.M):
                    have.add(word)
            for word in sorted(have):
                if word not in words:
                    invented.append((m.group(1), path.relative_to(ROOT), word))
            for word in sorted(set(KW_WORD.values()) | set(FIELD_WORD.values())):
                if word in have:
                    continue
                # The printed keyword line starts the ability, so require the
                # word at a line start or after a separator — "escape" inside
                # "escapes" or a flavour sentence is not a keyword line.
                if re.search(rf"(?:^|[.;\n] *){re.escape(word)}\b", words):
                    missing.append((m.group(1), path.relative_to(ROOT), word))
    return checked, invented, missing


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--invented", action="store_true")
    ap.add_argument("--missing", action="store_true")
    ap.add_argument("--rows", type=int, default=40)
    args = ap.parse_args()
    both = not (args.invented or args.missing)
    checked, invented, missing = scan()
    print(f"=== keyword drift over {checked} single-faced cached cards: "
          f"{len(invented)} invented, {len(missing)} missing")
    for label, rows, on in (("INVENTED (body has it, the card does not)", invented,
                             both or args.invented),
                            ("MISSING (the card prints it, the body does not)", missing,
                             both or args.missing)):
        if not on:
            continue
        print(f"\n--- {label}: {len(rows)}")
        shown = rows if args.rows == 0 else rows[: args.rows]
        for name, path, word in shown:
            print(f"  {word:<14} {name:<40} {path}")
        if len(shown) < len(rows):
            print(f"  … {len(rows) - len(shown)} more (--rows 0)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
