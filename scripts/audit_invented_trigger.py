#!/usr/bin/env python3
"""Does a card's OWN literal declare a trigger its printed text never says?

The invented-ability column that ENGINE_BACKLOG's "CLOSED WITH A REASON
2026-09-12" asked for and did not build. That note killed the idea of
inverting `audit_oracle_verbs`, and the reason was structural: its `have` set
expands helpers transitively and consults a global helper table across every
set file, which is an *over-approximation on purpose*. An over-approximated
`have` costs a false negative in the MISSING direction and a false **positive**
in the INVENTED one — 14,350 rows over 17,026 cards when it was measured. Its
closing sentence names the fix: "an invented-ability column needs a `have` set
built from the card's OWN literal with no helper expansion and no global
table — a different reader, not a flag on this one."

This is that reader, for one family: the **trigger event**. A factory that
writes `EventKind::Attacks` inside its own braces is making that claim in its
own literal; a factory that gets its trigger from `shortcut::backup(…)` writes
no `EventKind::` at all and is simply skipped. The skip is the point — it is a
false negative, which this direction can afford, and it is what keeps the
helper over-attribution that sank the other attempt out of the reading.

Three gates:

1. the factory's own body text names `EventKind::<Variant>` (helpers are not
   followed, and a `StepBegins(TurnStep::X)` is read down to its step);
2. the printed text — **every face, reminder text kept** — contains none of
   the phrasings that spell that event. The phrase sets are deliberately
   *lenient* (`Attacks` accepts the bare word "attack"): a lenient oracle side
   trades findings for precision, which is the trade this direction wants;
3. the variant is in `PHRASES`. An event with no entry is counted as
   unmodelled and listed by `--unmodelled`, never silently passed.

    python3 scripts/audit_invented_trigger.py              # findings
    python3 scripts/audit_invented_trigger.py --count      # totals only
    python3 scripts/audit_invented_trigger.py --unmodelled # events with no phrase set
    python3 scripts/audit_invented_trigger.py --check      # exit 1 on any finding
"""

import json
import os
import re
import sys
import unicodedata
from collections import Counter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crabomination_catalog", "src")
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")

# Gate 2. Lenient by design — see the docstring. A phrase set that is too
# tight manufactures findings on correct cards, which is the one cost this
# column cannot absorb.
PHRASES = {
    # "puts a creature ONTO THE BATTLEFIELD" (Overburden) and "when you PLAY
    # ANOTHER LAND" (City of Traitors) are both spelled as an enters trigger.
    "EntersBattlefield": ("enter", "onto the battlefield", "play another land"),
    "CreatureEntersBattlefield": ("enter", "onto the battlefield"),
    # "Whenever you PLAY A CARD" (Recycle, Search the City, Juju Bubble) is one
    # printed clause that the engine must spell as two events.
    "SpellCast": ("cast", "spell", "play a card", "plays a card"),
    "Attacks": ("attack",),
    "YouAttack": ("attack",),
    "AttacksAndIsntBlocked": ("attack", "block"),
    "Blocks": ("block",),
    "BecomesBlocked": ("block",),
    "CreatureDied": ("dies", "die", "graveyard"),
    "PermanentDied": ("dies", "die", "graveyard"),
    "CreatureOrArtifactDied": ("dies", "die", "graveyard"),
    # A leaves-the-battlefield trigger is printed four other ways: "is put
    # into a graveyard from the battlefield" (Mephitic Draught, Spine of Ish
    # Sah, a Role token), "dies", and "when you lose control of" (Coffin
    # Queen, Duplicity) — losing control is a leave in the engine's terms.
    "PermanentLeavesBattlefield": (
        "leave", "put into a graveyard", "put into exile", "lose control",
        "loses control", "dies", "die",
    ),
    "CreatureLeavesBattlefieldNotDying": ("leave", "lose control", "loses control"),
    "DealsCombatDamage": ("damage",),
    "DealsCombatDamageToPlayer": ("damage",),
    "DealsCombatDamageToCreature": ("damage",),
    "DealsCombatDamageToPlaneswalker": ("damage",),
    "DealtCombatDamage": ("damage",),
    "ControllerDealtCombatDamage": ("damage",),
    "DealsDamage": ("damage",),
    "DealsDamageToPlayer": ("damage",),
    "DealsDamageToCreature": ("damage",),
    "DealtDamage": ("damage",),
    "PlayerDamaged": ("damage",),
    "LandPlayed": ("land", "play a card", "plays a card"),
    "LifeGained": ("life",),
    "LifeLost": ("life", "lose", "loses"),
    "Tapped": ("tap",),
    "BecomesUntapped": ("untap",),
    "TappedForMana": ("tap", "mana"),
    "CardDrawn": ("draw",),
    "FirstCardDrawnThisTurn": ("draw",),
    "CardDiscarded": ("discard",),
    "OpponentCausedYouToDiscard": ("discard",),
    "BecameTarget": ("target",),
    "PermanentSacrificed": ("sacrific",),
    "CreatureSacrificed": ("sacrific",),
    "CardCycled": ("cycl",),
    "TurnedFaceUp": ("face up", "morph", "manifest", "disguise", "cloak", "megamorph"),
    "CounterAdded": ("counter",),
    "AnyCounterAdded": ("counter",),
    "CardLeftGraveyard": ("graveyard",),
    "PutIntoGraveyard": ("graveyard",),
    "CardMilled": ("mill", "graveyard"),
    "CardExiled": ("exile",),
    "CommittedCrime": ("crime",),
    "Expend": ("expend",),
    "DoorUnlocked": ("door", "unlock"),
    "AbilityActivated": ("abilit", "activate"),
    "DayNightChanged": ("day", "night"),
    "ClassLevelReached": ("level",),
    "AuraAttached": ("attach", "aura", "enchant"),
    "Transformed": ("transform",),
    "TokenCreated": ("token",),
    "ScriedOrSurveiled": ("scry", "surveil"),
    "Proliferated": ("proliferate",),
    "Explored": ("explore",),
    "SpellCountered": ("counter",),
    "CrewsOrSaddles": ("crew", "saddle"),
}

# `EventKind::StepBegins(TurnStep::X)` reads down to the step, since the
# printed wording is the step's name and not the variant's.
STEP_PHRASES = {
    "Untap": ("untap step",),
    "Upkeep": ("upkeep",),
    "Draw": ("draw step",),
    "PreCombatMain": ("main phase", "precombat main"),
    "PostCombatMain": ("main phase", "postcombat main"),
    "BeginCombat": ("combat",),
    "DeclareAttackers": ("attack",),
    "DeclareBlockers": ("block",),
    "FirstStrikeDamage": ("damage",),
    "CombatDamage": ("damage",),
    "EndCombat": ("combat",),
    "End": ("end step", "end of turn"),
    "Cleanup": ("cleanup", "end of turn"),
}

STEP_RE = re.compile(r"EventKind::StepBegins\(\s*(?:\w+::)*TurnStep::(\w+)")
KIND_RE = re.compile(r"EventKind::(\w+)")


def slug(name):
    """"Palani's Hatcher" -> "palanis_hatcher", the factory-name shape.

    ⚠ The apostrophe is DROPPED and accents are FOLDED before the separator
    pass. A naive slug turns "Palani's" into `palani_s` and "Andúril" into
    `and_ril`, neither of which matches the `pub fn`, so both cards fell back
    to the first string in the body — a token's name — and were scored against
    a *different card's* oracle.
    """
    folded = unicodedata.normalize("NFKD", name)
    folded = "".join(c for c in folded if not unicodedata.combining(c))
    return re.sub(r"[^a-z0-9]+", "_", folded.lower().replace("'", "")).strip("_")


# Implementation devices: the engine spells a printed clause with an event the
# card does not print, and the modelling is deliberate and documented in the
# factory. Each row is `(fn, label): reason`. A row here is NOT a pass for the
# card's body — it says the *event* is the right device for that clause.
ALLOW = {
    # CR 603.8 state triggers. "When you control no Islands, sacrifice this"
    # has no timing at all; the engine checks it at upkeep.
    ("dandan", "StepBegins(Upkeep)"): "CR 603.8 state trigger checked at upkeep",
    ("sea_serpent", "StepBegins(Upkeep)"): "CR 603.8 state trigger checked at upkeep",
    ("kukemssa_serpent", "StepBegins(Upkeep)"): "CR 603.8 state trigger checked at upkeep",
    ("merchant_ship", "StepBegins(Upkeep)"): "CR 603.8 state trigger checked at upkeep",
    ("city_in_a_bottle", "StepBegins(Upkeep)"): "CR 603.8 state trigger checked at upkeep",
    ("jihad", "StepBegins(Upkeep)"): "CR 603.8 state trigger checked at upkeep",
    ("goblins_of_the_flarg", "EntersBattlefield"): "CR 603.8 state trigger, rechecked as permanents enter",
    # Ascend (CR 702.131) is a continuous check; the engine re-reads the
    # city's blessing when a permanent enters and at upkeep.
    ("slippery_scoundrel", "EntersBattlefield"): "ascend re-check",
    ("slippery_scoundrel", "StepBegins(Upkeep)"): "ascend re-check",
    ("wayward_swordtooth", "EntersBattlefield"): "ascend re-check",
    ("wayward_swordtooth", "StepBegins(Upkeep)"): "ascend re-check",
    ("twilight_prophet", "EntersBattlefield"): "ascend re-check",
    # Documented approximations, each with its note in the factory.
    ("marionette_master", "PermanentSacrificed"): "destroy case approximated to the sacrifice path",
    ("watery_grasp", "StepBegins(Upkeep)"): "'doesn't untap' modelled as a re-tap, as Narcolepsy",
    ("wrenn_and_six", "StepBegins(Upkeep)"): "the retrace emblem collapsed to upkeep recursion",
    ("zaffai_and_the_tempests", "StepBegins(PreCombatMain)"): "'once each of your turns' as a main-phase grant",
    ("skizzik", "EntersBattlefield"): "the un-kicked end-step sacrifice registered as a delayed trigger on entry",
    ("triskaidekaphile", "EntersBattlefield"): "'no maximum hand size' flipped as a one-shot on entry",
    ("soul_ransom", "EntersBattlefield"): "'you control enchanted creature' taken at the attach",
    ("threads_of_disloyalty", "EntersBattlefield"): "'you control enchanted creature' taken at the attach",
}


def defs_in(path):
    """(fn, card name, body) per `pub fn … -> CardDefinition`. The name comes
    from any string literal whose slug is the function's — most creature
    factories spread a `..creature("Name", …)` constructor and have no `name:`
    field of their own."""
    src = open(path, encoding="utf-8").read()
    for m in re.finditer(r"pub fn (\w+)\(\) -> CardDefinition \{", src):
        start = m.end() - 1
        depth, i = 0, start
        while i < len(src):
            if src[i] == "{":
                depth += 1
            elif src[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        body = src[start : i + 1]
        fn = m.group(1)
        keyed = re.findall(r'name:\s*"((?:[^"\\]|\\.)*)"', body)
        own = next(
            (n for n in re.findall(r'"((?:[^"\\]|\\.)*)"', body) if slug(n) == fn), None
        )
        if own is None and not keyed:
            continue
        yield fn, own or keyed[0], body


def printed_text(card):
    parts = [card.get("oracle_text") or ""]
    for face in card.get("card_faces") or []:
        parts.append(face.get("oracle_text") or "")
    return " \n ".join(parts).lower()


def claims(body):
    """{(label, phrases)} the body asserts in its own literal."""
    out = {}
    for m in STEP_RE.finditer(body):
        step = m.group(1)
        out[f"StepBegins({step})"] = STEP_PHRASES.get(step)
    for m in KIND_RE.finditer(body):
        kind = m.group(1)
        if kind == "StepBegins":
            continue
        out[kind] = PHRASES.get(kind)
    return out


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    lower = {k.lower(): v for k, v in cache.items() if isinstance(v, dict)}
    for k, v in list(cache.items()):
        if isinstance(v, dict):
            lower.setdefault(k.split(" // ")[0].lower(), v)

    hits, unmodelled = [], Counter()
    checked = uncached = notext = claim_count = allowed = 0
    seen_fns = set()
    for dirpath, _, files in os.walk(CATALOG):
        for f in files:
            if not f.endswith(".rs"):
                continue
            path = os.path.join(dirpath, f)
            for fn, name, body in defs_in(path):
                declared = claims(body)
                if not declared:
                    continue
                card = lower.get(name.lower())
                if card is None:
                    uncached += 1
                    continue
                text = printed_text(card)
                if not text.strip():
                    notext += 1
                    continue
                checked += 1
                for label, phrases in sorted(declared.items()):
                    if phrases is None:
                        unmodelled[label] += 1
                        continue
                    claim_count += 1
                    if any(p in text for p in phrases):
                        continue
                    if (fn, label) in ALLOW:
                        allowed += 1
                        seen_fns.add((fn, label))
                        continue
                    hits.append(
                        (
                            name,
                            fn,
                            os.path.relpath(path, ROOT),
                            label,
                            " ".join(text.split())[:110],
                        )
                    )

    if "--unmodelled" in sys.argv:
        for label, n in unmodelled.most_common():
            print(f"{n:5d}  {label}")
        print(f"# {len(unmodelled)} event kinds with no phrase set")
        return 0

    hits.sort()
    if "--count" not in sys.argv:
        for name, fn, path, label, text in hits:
            print(f"{path}::{fn}  [{label}]\n    {name}: “{text}”")
    # The staleness half (`audit_loop_splice`'s device): an ALLOW row whose
    # site is gone is a row nobody re-reads, so it fails rather than rots.
    stale = sorted(set(ALLOW) - seen_fns)
    for fn, label in stale:
        print(f"# ALLOW is STALE: {fn} no longer claims {label}", file=sys.stderr)
    print(
        f"# {len(hits)} triggers the printed card never says, "
        f"{claim_count} claims over {checked} bodies checked, "
        f"{allowed} allowed as implementation devices ({len(stale)} stale), "
        f"{sum(unmodelled.values())} claims on {len(unmodelled)} unmodelled event "
        f"kinds (--unmodelled), {uncached} bodies skipped as uncached, "
        f"{notext} with no printed text"
    )
    if "--check" in sys.argv and (hits or stale):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
