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

**The two directions are not equally strong.** INVENTED is a proof — the
engine carries a mechanic the printed card does not have, which is a wrong
card in play — and it reads **0** after the pass that closed it. MISSING is a
*reading list*: it cannot see a mechanic spelled as a bespoke triggered
ability instead of a keyword, so a row is a card to read against its oracle,
not a proven gap. Spot-checked, the big buckets are real (Gibbering Kami has
no Soulshift 3 at all; Scurry Oak's own doc says "Evolve" and its body has
only the counter trigger) — and four of them, `spree` / `soulshift` /
`channel` / `evolve`, have **no `Keyword::` variant and no field in the
engine**, so those are missing-mechanic work rather than missing-card work.
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
# The same question for the mechanics stored outside `keywords`. ⚠ Every
# mechanic with its own field has to be listed here or the *missing* direction
# reports it on every card that has it — `affinity_filter` alone was 39 rows.
# ⚠ Every key must be a real `CardDefinition` field. Ten of these were not
# (`cycling_cost`, `escape_cost`, `madness_cost`, `evoke_cost`, `disturb_cost`,
# `overload_cost`, `warp_cost`, `impending`, `spree_modes`, `channel_abilities`)
# — those mechanics carry their cost on the `Keyword::` variant itself — and a
# key that names no field is a check that silently never runs.
FIELD_WORD = {
    "foretell_cost": "foretell",
    "plot_cost": "plot",
    "buyback_additional_cost": "buyback",
    "entwine_additional_cost": "entwine",
    "splice_extra_cost": "splice",
    "affinity_filter": "affinity",
    "affinity_graveyard_filter": "affinity",
    "mutate": "mutate",
    "bestow": "bestow",
    "adventure": "adventure",
    "companion": "companion",
    "omen": "omen",
    "gift": "gift",
    "station": "station",
    "saga_chapters": "saga",
    "room": "room",
    "case": "case",
    "soulbond_bonus": "soulbond",
    "kicker_action_cost": "kicker",
    "flashback_additional_cost": "flashback",
    "prototype": "prototype",
}

# Deliberate modellings, checked once so they do not have to be re-checked.
# The card does not print the keyword; the engine's keyword is the closest
# primitive to what it does print.
ALLOWED = {
    # "you may pay {N} any number of times" (Adversary cycle) is multikicker
    # in everything but name.
    ("intrepid_adversary", "kicker"),
    ("bloodthirsty_adversary", "kicker"),
    # "cast from your graveyard by paying {3}{R} and exiling four other cards"
    # is Escape 4 spelled out.
    ("squee_dubious_monarch", "escape"),
    # Pit Scorpion predates the keyword: "whenever this deals damage to a
    # player, that player gets a poison counter". Poisonous is combat-only, so
    # this is an approximation and not an equivalence.
    ("pit_scorpion", "poisonous"),
    # `affinity_filter` is the engine's cost-reduction primitive, and these
    # cards spell the reduction out instead of printing the keyword ("costs
    # {1} less to cast for each creature on the battlefield").
    ("carapace_forger", "affinity"),
    ("temur_battlecrier", "affinity"),
    ("spectral_denial", "affinity"),
    ("rowdy_research", "affinity"),
    ("static_snare", "affinity"),
    ("gargantuan_leech", "affinity"),
    ("blasphemous_act", "affinity"),
    ("vanquish_the_horde", "affinity"),
    ("sea_gods_scorn", "affinity"),
    ("stone_idol_trap", "affinity"),
}

# ⚠ Some of those fields are *implementation vehicles* rather than the printed
# keyword: `affinity_graveyard_filter` is how "costs {1} less for each instant
# in your graveyard" is modelled and no such card prints "Affinity", and
# `kicker_action_cost` / `flashback_additional_cost` are riders on cards whose
# printed keyword is spelled some other way. They answer the MISSING question
# and must not answer the INVENTED one.
FIELD_ONLY_MISSING = {
    "affinity_graveyard_filter", "kicker_action_cost", "flashback_additional_cost",
    "saga_chapters", "room", "case", "station", "gift", "soulbond_bonus",
}

# ⚠ `alternative_cost: Some(..)` is the engine's one vehicle for the whole
# alternative-cost family, so a card that has it may be carrying any of these
# without a `Keyword::` of its own — Ondu Rising's Awaken is
# `alternative_cost: Some(awaken(..))`. Suppress the family for such a card in
# the MISSING direction rather than report every one of them.
ALT_FAMILY = {
    "awaken", "evoke", "madness", "prowl", "spectacle", "surge", "emerge",
    "escape", "dash", "blitz", "disturb", "overload", "warp", "impending",
    "plot", "prototype", "mutate", "bestow", "cleave", "casualty", "spree",
    "channel", "adapt", "foretell", "miracle", "morph", "megamorph",
    "disguise", "embalm", "eternalize", "unearth", "flashback", "retrace",
    "jump-start", "boast", "forecast", "ninjutsu",
}

# ⚠ The return type is spelled two ways in the tree — bare `CardDefinition`
# and the fully-qualified `crate::card::CardDefinition`. Requiring the bare
# form skipped 6,895 factories, Stone Idol Trap among them.
FN = re.compile(r"^pub fn ([a-z0-9_]+)\(\) -> (?:[A-Za-z_]+::)*CardDefinition \{", re.M)
# `name: "X",` and not `name: "X".into(),` — the latter is a
# `TokenDefinition`, and a factory that mints a token first would
# otherwise answer for the card (Magitek Armor's Hero token did).
BODY_NAME = re.compile(r'^ {8}name: "([^"]*)",$', re.M)
REMINDER = re.compile(r"\([^)]*\)")


def oracle_words(name: str):
    """Printed text, reminder text stripped, or None when uncheckable."""
    c = CACHE_LC.get(name.lower())
    # ⚠ Not every two-faced entry carries `card_faces`; some spell both halves
    # into one string, and the body only builds the front.
    if not c or c.get("card_faces") or " // " in (c.get("type_line") or "") \
            or " // " in (c.get("mana_cost") or ""):
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


def factory_body(src: str, open_brace_end: int) -> str:
    """The factory's text, ending at its own closing brace.

    ⚠ "to the next `pub fn … -> CardDefinition`" is wrong and was wrong here:
    a helper with arguments does not match, so the body ran on into the next
    two or three factories and lent them each other's fields. Three "invented
    Cycling" findings were that bug.
    """
    depth, j = 1, open_brace_end
    while j < len(src) and depth:
        ch = src[j]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
        j += 1
    return src[open_brace_end:j]


def card_literal(body: str, name_pos: int) -> str:
    """The `CardDefinition { … }` the matched `name:` belongs to.

    ⚠ A factory that mints a TOKEN first puts that token's `power:` and
    `toughness:` at the same eight spaces as the card's, and the first match
    wins. Seven "BODY WRONG" findings were Cat, Bird, Fox, Mutagen, Dragon
    Illusion and Elemental tokens answering for their makers.
    """
    start = body.rfind("CardDefinition {", 0, name_pos)
    if start < 0:
        return body
    i = body.index("{", start)
    depth, j = 0, i
    while j < len(body):
        if body[j] == "{":
            depth += 1
        elif body[j] == "}":
            depth -= 1
            if depth == 0:
                break
        j += 1
    return body[i : j + 1]


def scan():
    invented, missing, checked = [], [], 0
    for path in sorted(CATALOG.rglob("*.rs")):
        src = path.read_text(encoding="utf-8")
        for m in FN.finditer(src):
            body = factory_body(src, m.end())
            bn = BODY_NAME.search(body)
            if not bn:
                continue
            words = oracle_words(bn.group(1))
            if words is None:
                continue
            body = card_literal(body, bn.start())
            checked += 1
            have = {KW_WORD[k] for k in top_level_keywords(body) if k in KW_WORD}
            strict = set(have)
            for field, word in FIELD_WORD.items():
                if re.search(rf"^ {{8}}{field}: (Some|vec!\[[^\]]|&\[)", body, re.M):
                    have.add(word)
                    if field not in FIELD_ONLY_MISSING:
                        strict.add(word)
            for word in sorted(strict):
                if word not in words and (m.group(1), word) not in ALLOWED:
                    invented.append((m.group(1), path.relative_to(ROOT), word))
            has_alt = re.search(r"^ {8}alternative_cost: Some", body, re.M) is not None
            for word in sorted(set(KW_WORD.values()) | set(FIELD_WORD.values())):
                if word in have or (has_alt and word in ALT_FAMILY):
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
