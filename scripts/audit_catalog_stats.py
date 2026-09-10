#!/usr/bin/env python3
"""Catalog-wide stat audit vs the real Scryfall cache (scripts/.scryfall_cache.json).

Scans every card factory under crabomination_catalog/src/sets/ and checks the
printed-stat columns against the cache: mana cost, power/toughness, creature
subtypes, card / supertypes, keywords, activated-ability costs / timing /
tap-sac halves, loyalty, token P/T, trigger events / scopes / filters, and
the amounts an ability prints (`num`, INCOMPLETE_CARDS "Amounts").
Generalizes audit_stx_drift.py (cost+P/T, STX only) and audit_stx_types.py
(type+keywords, STX only) to the whole catalog.

  python3 scripts/audit_catalog_stats.py              # per-set summary table
  python3 scripts/audit_catalog_stats.py SET           # detail for one set (e.g. sos, thb)

Only card names present in the cache are checked; synthesized-only names are
skipped. Keyword check reads the top-level CardDefinition.keywords field only
(so conditional/granted keywords nested in statics/equip-bonuses/tokens aren't
flagged). DFC keyword/type refs union across faces.
"""
import json, re, sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SETS = REPO / "crabomination_catalog" / "src" / "sets"
CACHE = json.load(open(Path(__file__).resolve().parent / ".scryfall_cache.json"))
CACHE_LC = {k.lower(): v for k, v in CACHE.items() if isinstance(v, dict)}

COLOR = {"White": "W", "Blue": "U", "Black": "B", "Red": "R", "Green": "G"}
ENGINE_KW = {"Flying","Vigilance","Menace","Trample","Deathtouch","Lifelink","First strike",
 "Double strike","Reach","Defender","Haste","Hexproof","Flash","Ward","Indestructible",
 "Shroud","Skulk","Horsemanship","Protection","Prowess","Changeling","Fear","Intimidate"}
KMAP = {
    "FirstStrike": "First strike",
    "DoubleStrike": "Double strike",
    # Scryfall reports every parameterized protection as plain "Protection".
    "ProtectionFromCreatureType": "Protection",
    "ProtectionFromSpellSubtype": "Protection",
    "ProtectionFromCardType": "Protection",
    "ProtectionFromColoredSpells": "Protection",
    "ProtectionFromCreatures": "Protection",
    "ProtectionFromEverything": "Protection",
    "ProtectionFromInstants": "Protection",
    "ProtectionFromMonocolored": "Protection",
    "ProtectionFromMulticolored": "Protection",
    "ProtectionFromSpells": "Protection",
}

# Keywords the engine spells with a payload Scryfall reports plainly.
KMAP.update({
    "HexproofFromColor": "Hexproof",
    "HexproofFromMonocolored": "Hexproof",
    "HexproofFromMulticolored": "Hexproof",
    "HexproofFromAbilities": "Hexproof",
    "ProtectionFromMatching": "Protection",
    "ProtectionFromManaValueExcept": "Protection",
    "ProtectionFromManaValueParity": "Protection",
    "ProtectionFromOwnColors": "Protection",
})

KW_FN = re.compile(r"\nfn (\w+)\(\)\s*->\s*Keyword\s*\{\s*Keyword::(\w+)")


def kw_fn_table(text):
    """`{fn_name: variant}` for a file-local `fn ward_1() -> Keyword`."""
    return {m.group(1): m.group(2) for m in KW_FN.finditer(text)}


def top_level_items(vec_body):
    """The comma-separated elements of a `vec![…]` body, at bracket depth 0."""
    out, depth, cur = [], 0, ""
    for ch in vec_body:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur)
    return out


def code_keywords(vec_body, kwfns):
    """The PRINTED keyword of each top-level element.

    A flat `Keyword::(\w+)` scan over the whole vector reads the keyword inside
    a payload as if the card had it: `Keyword::CantBeBlockedExceptBy(Box::new(
    R::HasKeyword(Keyword::Flying)))` made Treetop Scout, Treetop Rangers,
    Elven Riders, Spire Tracer and Silhana Ledgewalker all read as fliers, and
    every one of them was already correct (2026-08-30).
    """
    out = set()
    for item in top_level_items(vec_body):
        item = item.strip()
        m = re.match(r"(?:\w+::)*Keyword::(\w+)", item)
        if m:
            out.add(KMAP.get(m.group(1), m.group(1)))
            continue
        m = re.match(r"(\w+)\(\)", item)
        if m and m.group(1) in kwfns:
            v = kwfns[m.group(1)]
            out.add(KMAP.get(v, v))
    return out & ENGINE_KW


def sym(call):
    call = call.strip()
    m = re.match(r"generic\((\d+)\)", call)
    if m: return "{%d}" % int(m.group(1))
    m = re.match(r"colorless\((\d+)\)", call)
    if m: return "{C}" * int(m.group(1))
    if call == "x()": return "{X}"
    for fn, s in [("w","W"),("u","U"),("b","B"),("r","R"),("g","G")]:
        if call == f"{fn}()": return "{%s}" % s
    m = re.match(r"hybrid\(Color::(\w+),\s*Color::(\w+)\)", call)
    if m: return "{%s/%s}" % (COLOR[m.group(1)], COLOR[m.group(2)])
    m = re.match(r"mono_hybrid\((\d+),\s*Color::(\w+)\)", call)
    if m: return "{%d/%s}" % (int(m.group(1)), COLOR[m.group(2)])
    m = re.match(r"phyrexian\(Color::(\w+)\)", call)
    if m: return "{%s/P}" % COLOR[m.group(1)]
    # The spelled-out `ManaSymbol::` forms a `ManaCost::new(vec![..])` carries.
    m = re.match(r"ManaSymbol::Generic\((\d+)\)", call)
    if m: return "{%d}" % int(m.group(1))
    m = re.match(r"ManaSymbol::Colored\(Color::(\w+)\)", call)
    if m: return "{%s}" % COLOR[m.group(1)]
    if call == "ManaSymbol::Colorless": return "{C}"
    return None

def norm(mc):
    """Mana cost as an order-insensitive symbol multiset. An empty cost and
    `{0}` are the same object (`cost(&[])` is how the catalog spells {0})."""
    syms = tuple(sorted(re.findall(r"\{[^}]+\}", mc or "")))
    return () if syms == ("{0}",) else syms

def vec_after(body, i):
    j = body.find("vec![", i)
    if j < 0 or j - i > 40: return None
    k, d = j + 5, 1
    while k < len(body) and d:
        if body[k] == "[": d += 1
        elif body[k] == "]": d -= 1
        k += 1
    return body[j + 5:k - 1]

def bracket_span(body, i):
    """`body[i]` is an opening `[`/`{`/`(`; the index one past its match."""
    pairs = {"[": "]", "{": "}", "(": ")"}
    close, d, k = pairs[body[i]], 1, i + 1
    while k < len(body) and d:
        if body[k] == body[i]: d += 1
        elif body[k] == close: d -= 1
        k += 1
    return k

def ability_literals(body):
    """Every `ActivatedAbility { .. }` literal in the card's own
    `activated_abilities: vec![..]`, as source text; `None` when the field is
    absent or any element is a helper call the scan cannot open."""
    m = own_field(body, r"activated_abilities:")
    if m is None:
        return None
    vec = vec_after(body, m.start())
    if vec is None:
        return None
    for item in top_level_items(vec):
        if item.strip() and not item.strip().startswith("ActivatedAbility {"):
            return None
    out, i = [], 0
    while True:
        j = vec.find("ActivatedAbility {", i)
        if j < 0:
            break
        end = bracket_span(vec, j + len("ActivatedAbility "))
        out.append(vec[j:end])
        i = end
    return out

def literal_depth1_field(lit, field):
    """`field:` at brace depth 1 of an `ActivatedAbility { .. }` literal — its
    own field, never one of a nested effect's."""
    depth, k = 0, len("ActivatedAbility {")
    while k < len(lit):
        c = lit[k]
        if c in "{[(":
            depth += 1
        elif c in "}])":
            depth -= 1
        elif depth == 0 and lit.startswith(field, k):
            return k
        k += 1
    return None

# The oracle's activation-timing riders. "only during your turn" is met by
# `condition: Some(Predicate::IsTurnOf(PlayerRef::You))` — an `All(..)` around
# it with a step list is "before attackers are declared" (Rag Man, Stern
# Marshal). `sorcery_speed: true` used to stand in for it and no longer does
# (2026-09-08: the last eight were re-shaped).
_TIMING = (
    ("only as a sorcery", "sorcery"),
    ("only during your turn", "your_turn"),
    # "only during your upkeep" (Mirror Universe, Firemane Angel) is a
    # your-turn rider plus the upkeep step, both read.
    ("only during your upkeep", "your_turn"),
    ("only during your upkeep", "upkeep"),
    ("only once each turn", "once"),
    # "no more than twice each turn" (Vampire Bats) — `max_activations_per_turn: Some(2)`.
    ("no more than twice each turn", "twice"),
    ("no more than three times each turn", "max3"),
)

def ability_timing(body):
    """Per literal, the timing riders it declares."""
    lits = ability_literals(body)
    if lits is None:
        return None
    out = []
    for lit in lits:
        flags = set()
        k = literal_depth1_field(lit, "sorcery_speed:")
        if k is not None and re.match(r"sorcery_speed:\s*true", lit[k:]):
            flags.add("sorcery")
        k = literal_depth1_field(lit, "once_per_turn:")
        if k is not None and re.match(r"once_per_turn:\s*true", lit[k:]):
            flags.add("once")
        k = literal_depth1_field(lit, "max_activations_per_turn:")
        if k is not None:
            m = re.match(r"max_activations_per_turn:\s*Some\((\d+)\)", lit[k:])
            if m:
                flags.add({"1": "once", "2": "twice"}.get(m.group(1), "max" + m.group(1)))
        k = literal_depth1_field(lit, "condition:")
        if k is not None:
            # The whole `Some(..)` expression — an `IsTurnOf(You)` nested in
            # an `All(..)` (Cao Cao) is the rider too.
            m = re.match(r"condition:\s*Some\(", lit[k:])
            if m:
                a = k + m.end() - 1
                cond = lit[a:bracket_span(lit, a)]
                # `Not(IsTurnOf(You))` is "an opponent's turn" (Maddening Imp).
                bare = re.sub(r"Not\(Box::new\(\s*(?:crate::effect::)?Predicate::IsTurnOf\((?:crate::effect::)?PlayerRef::You\)\)\)", "", cond)
                if re.search(r"IsTurnOf\((?:crate::effect::)?PlayerRef::You\)", bare):
                    flags.add("your_turn")
                if re.search(r"CurrentStepIs\((?:crate::game::(?:types::)?)?TurnStep::Upkeep\)|upkeep_only\(\)", cond):
                    flags.add("upkeep")
                if "upkeep_only()" in cond:  # atq.rs: IsTurnOf(You) + the upkeep step
                    flags.add("your_turn")
        out.append(frozenset(flags))
    return out

def ref_ability_timing(card, face=None):
    """Per oracle activation line, the timing riders it prints."""
    text = (face or card).get("oracle_text")
    if text is None:
        return None
    text = re.sub(r"\([^)]*\)", "", text)
    out = []
    for line in text.split("\n"):
        if not _ORACLE_ACT.match(line.strip()):
            continue
        out.append(frozenset(flag for phrase, flag in _TIMING if phrase in line))
    return out

def timing_mismatch(code, ref):
    """Multiset compare. A code `sorcery` no longer stands for a printed
    `your_turn` (the approximation was retired 2026-09-08)."""
    # A step list that merely includes the upkeep ("before attackers are
    # declared" — Cao Cao) is not an upkeep rider.
    if not any("upkeep" in f for f in ref):
        code = [frozenset(x for x in f if x != "upkeep") for f in code]
    return sorted(sorted(f) for f in code) != sorted(sorted(f) for f in ref)

def ability_tap_sac(body):
    """Per literal, which of the non-mana cost halves it declares: `tap`
    (`tap_cost: true`) and `sac` (`sac_cost: true`, the self-sacrifice)."""
    lits = ability_literals(body)
    if lits is None:
        return None
    out = []
    for lit in lits:
        flags = set()
        for field, flag in (("tap_cost", "tap"), ("sac_cost", "sac")):
            k = literal_depth1_field(lit, field + ":")
            if k is not None and re.match(field + r":\s*true", lit[k:]):
                flags.add(flag)
        # A self-sacrifice spelled as the effect's first step (the fetchlands'
        # `Move { This -> Graveyard }`) is the same shape as a cost here.
        if re.search(r"Move \{\s*what: Selector::This,\s*to: ZoneDest::Graveyard|Sacrifice \{\s*what: Selector::This", lit):
            flags.add("sac")
        out.append(frozenset(flags))
    return out

def ref_ability_tap_sac(card, face=None):
    """Per oracle activation line, the `{T}` and "Sacrifice this ..." halves of
    its cost (the text before the colon)."""
    text = (face or card).get("oracle_text")
    if text is None:
        return None
    text = re.sub(r"\([^)]*\)", "", text)
    out = []
    for line in text.split("\n"):
        m = _ORACLE_ACT.match(line.strip())
        if not m:
            continue
        cost = m.group(1)
        flags = set()
        if "{T}" in cost:
            flags.add("tap")
        # "Sacrifice Kagemaro:" names the card by its pre-comma half;
        # "Sacrifice two lands and this artifact" is a self-sacrifice too.
        short = re.escape(card.get("name", "\0").split(",")[0])
        self_sac = r"[Ss]acrifice (?:[^:.]* and )?(?:this|~|" + short + r")"
        if re.search(self_sac, cost):
            flags.add("sac")
        # "{2}{W}: Sacrifice this enchantment. Scry 2." — the sacrifice is
        # the effect's first sentence, and `sac_cost: true` is how the
        # catalog spells that shape (the difference is only visible to a
        # response); accept it.
        effect = line.strip()[m.end():]
        if re.match(self_sac, effect):
            flags.add("sac")
        out.append(frozenset(flags))
    return out

_ORACLE_LOY = re.compile(r"^([+\u2212-]?)(\d+|X):\s")

def loyalty_costs(body):
    """The signed loyalty cost of every `LoyaltyAbility { .. }` literal in the
    card's own `loyalty_abilities: vec![..]` ("X" for an `x_cost: true` minus),
    plus the card's `base_loyalty`; `None` when the field is absent or an
    element is a helper call."""
    m = own_field(body, r"loyalty_abilities:")
    if m is None:
        return None
    vec = vec_after(body, m.start())
    if vec is None:
        return None
    for item in top_level_items(vec):
        if item.strip() and not item.strip().startswith("LoyaltyAbility {"):
            return None
    out, i = [], 0
    while True:
        j = vec.find("LoyaltyAbility {", i)
        if j < 0:
            break
        end = bracket_span(vec, j + len("LoyaltyAbility ")); lit = vec[j:end]; i = end
        kx = literal_depth1_field(lit, "x_cost:")
        if kx is not None and re.match(r"x_cost:\s*true", lit[kx:]):
            out.append("-X")
            continue
        kc = literal_depth1_field(lit, "loyalty_cost:")
        mc = re.match(r"loyalty_cost:\s*(-?\d+)", lit[kc:]) if kc is not None else None
        if not mc:
            return None
        out.append(str(int(mc.group(1))))
    bl = own_field(body, r"base_loyalty:\s*(\d+)")
    return out, (bl.group(1) if bl else None)

def ref_loyalty_costs(card, face=None):
    text = (face or card).get("oracle_text")
    if text is None:
        return None
    out = []
    for line in text.split("\n"):
        m = _ORACLE_LOY.match(line.strip())
        if not m:
            continue
        sign = "-" if m.group(1) in ("-", "\u2212") else ""
        out.append(("-X" if m.group(2) == "X" else str(int(sign + m.group(2)))))
    return out, (face or card).get("loyalty")

# "a 3/3 green Elephant creature token", "two 1/1 white Soldier creature tokens".
_ORACLE_TOKEN = re.compile(r"(\d+)/(\d+) [A-Za-z\-, ]*?tokens?")

def token_stats(body):
    """Every `TokenDefinition { .. }` literal's P/T in the card body (before
    `strip_token_literals` runs — this reader takes the raw body), as
    sorted "P/T" strings; `None` when a token literal names no P/T."""
    out, i = [], 0
    while True:
        j = body.find("TokenDefinition {", i)
        if j < 0:
            break
        end = bracket_span(body, j + len("TokenDefinition ")); lit = body[j:end]; i = end
        kp = literal_depth1_field(lit, "power:"); kt = literal_depth1_field(lit, "toughness:")
        mp = re.match(r"power:\s*(-?\d+)", lit[kp:]) if kp is not None else None
        mt = re.match(r"toughness:\s*(-?\d+)", lit[kt:]) if kt is not None else None
        if not (mp and mt):
            return None
        out.append(f"{mp.group(1)}/{mt.group(1)}")
    return out

def ref_token_stats(card, face=None):
    text = (face or card).get("oracle_text")
    if text is None:
        return None
    return [f"{p}/{t}" for p, t in _ORACLE_TOKEN.findall(text)]

# ── trigger events ──────────────────────────────────────────────────────────
# A `TriggeredAbility` literal's `EventKind` and the oracle's "When / Whenever
# / At the beginning of" clause, both folded to a coarse class so that the
# question is "does this card trigger on the printed *event*" — an upkeep
# trigger shipped at the end step, a "deals combat damage" shipped as
# "attacks" — and never "is the scope/filter right" (a different column).
_STEP_CLASS = {
    "Upkeep": "upkeep", "End": "end", "BeginCombat": "combat", "EndCombat": "endcombat",
    "PreCombatMain": "main", "PostCombatMain": "main", "Draw": "draw", "Untap": "untap",
    "Cleanup": "cleanup", "DeclareAttackers": "attackers", "DeclareBlockers": "blockers",
    "CombatDamage": "combatdamage", "FirstStrikeDamage": "combatdamage",
}
_KIND_CLASS = {
    "EntersBattlefield": "etb",
    "CreatureDied": "dies", "PermanentDied": "dies", "CreatureOrArtifactDied": "dies",
    "PermanentLeavesBattlefield": "leaves", "CreatureLeavesBattlefieldNotDying": "leaves",
    "PermanentReturnedToHand": "leaves",
    "PutIntoGraveyard": "to_graveyard", "LandPutIntoGraveyard": "to_graveyard", "CardMilled": "to_graveyard",
    "CardLeftGraveyard": "leaves_graveyard", "PutIntoHandFromGraveyard": "leaves_graveyard",
    "CardExiled": "exiled", "CardExiledFromPlayOrGraveyard": "exiled",
    "SpellCast": "cast", "SpellCopied": "copied", "SpellCountered": "countered",
    "Attacks": "attacks", "YouAttack": "attacks", "AttacksAndIsntBlocked": "attacks",
    "Blocks": "blocks", "BlocksNOrMore": "blocks",
    "BecomesBlocked": "blocked", "BecomesBlockedByNOrMore": "blocked",
    "DealsCombatDamageToPlayer": "combat_damage", "DealsCombatDamageToCreature": "combat_damage",
    "DealsCombatDamageToPlaneswalker": "combat_damage", "DealsCombatDamage": "combat_damage",
    "DealsDamage": "deals_damage", "DealsDamageToPlayer": "deals_damage", "DealsDamageToCreature": "deals_damage",
    "YourInstantOrSorceryDealtDamage": "deals_damage", "YourInstantOrSorceryDealtDamageToPlayer": "deals_damage",
    "YourSourceDealtNoncombatDamageEqualToToughness": "deals_damage",
    "DealtDamage": "dealt_damage", "DealtCombatDamage": "dealt_damage", "PlayerDamaged": "dealt_damage",
    "ControllerDealtCombatDamage": "dealt_damage", "PlayerDealtNoncombatDamage": "dealt_damage",
    "LandPlayed": "land",
    "LifeGained": "lifegain", "LifeLost": "lifeloss", "PaidLife": "lifeloss",
    "CardDrawn": "draws", "FirstCardDrawnThisTurn": "draws",
    "Tapped": "tapped", "TappedForMana": "tapped", "BecomesUntapped": "untapped",
    "BecameTarget": "targeted", "ChoseTargets": "targeted",
    "PermanentSacrificed": "sacrifice", "CreatureSacrificed": "sacrifice",
    "CardCycled": "cycle", "TurnedFaceUp": "faceup",
    "CardDiscarded": "discard", "OpponentCausedYouToDiscard": "discard", "DiscardedOneOrMore": "discard",
    "CounterAdded": "counter", "AnyCounterAdded": "counter", "CounterRemoved": "counter_removed",
    "PoisonAdded": "counter", "Proliferated": "proliferate",
    "CommittedCrime": "crime", "Expend": "expend", "DoorUnlocked": "door", "RoomFullyUnlocked": "door",
    "Transformed": "transform", "TokenCreated": "token", "BecameMonarch": "monarch",
    "ScriedOrSurveiled": "scry", "RolledDice": "dice", "WonCoinFlip": "coin", "LostCoinFlip": "coin",
    "Explored": "explore", "EnergyGained": "energy", "PhasesIn": "phase", "PhasesOut": "phase",
    "AuraAttached": "attach", "AuraAttachedToAny": "attach", "BecameAttached": "attach",
    "AbilityActivated": "activated", "ExhaustAbilityActivated": "activated", "AdaptAbilityActivated": "activated",
    "LibraryShuffled": "shuffle", "PlayerSearchedLibrary": "search",
    "Mutated": "mutate", "ManifestedDread": "manifest", "Foraged": "forage", "GiftGiven": "gift",
    "EvidenceCollected": "evidence", "Discovered": "discover", "Encountered": "encounter",
    "CrewsOrSaddles": "crew", "DayNightChanged": "daynight", "ClassLevelReached": "level",
    "RingTempted": "ring", "BecomesPlotted": "plot", "DungeonCompleted": "dungeon",
    "GainedControlOfThis": "control", "LostControlOfThis": "control", "VotingFinished": "vote",
    "CaseSolved": "case", "Regenerated": "regenerate", "CumulativeUpkeepUnpaid": "cumulative",
    "ChaosEnsues": "chaos", "PlaneswalkedAwayFrom": "planeswalk", "SetInMotion": "scheme",
    "VisitedAttraction": "attraction",
}

# Oracle clause -> the classes a literal may spell it as. Read against the
# clause up to its first comma (the condition; an "if" rider is after it),
# every matching row unions in, and a clause no row reads leaves the card
# uncompared.
_ORACLE_TRIGGER_CLASSES = [
    (r"^at the beginning of .*upkeep", {"upkeep"}),
    (r"^at the beginning of .*end step", {"end"}),
    (r"^at the beginning of .*end of combat", {"endcombat"}),
    (r"^at the beginning of .*declare attackers", {"attackers"}),
    (r"^at the beginning of .*declare blockers", {"blockers"}),
    (r"^at the beginning of .*combat damage", {"combatdamage"}),
    (r"^at the beginning of (?:(?!end).)*\bcombat\b", {"combat"}),
    (r"^at the beginning of .*main phase", {"main"}),
    (r"^at the beginning of .*draw step", {"draw"}),
    (r"^at the beginning of .*untap step", {"untap"}),
    (r"^at the beginning of .*cleanup", {"cleanup"}),
    (r"^when(?:ever)? .*\benters\b(?! (?:a|your|the) graveyard)", {"etb"}),
    (r"^when(?:ever)? .*\bdies?\b", {"dies"}),
    (r"^when(?:ever)? .*put into (?:a|an|your|their|an opponent's) graveyard from the battlefield", {"dies"}),
    (r"^when(?:ever)? .*put into (?:a|an|your|their|an opponent's) graveyard(?! from the battlefield)", {"to_graveyard"}),
    (r"^when(?:ever)? .*\bmill(?:s|ed)?\b", {"to_graveyard"}),
    (r"^when(?:ever)? .*leaves? the battlefield", {"leaves"}),
    (r"^when(?:ever)? .*\battacks?\b", {"attacks"}),
    (r"^when(?:ever)? .*\bblocks?\b", {"blocks"}),
    (r"^when(?:ever)? .*(?:becomes?|is|are) blocked\b", {"blocked"}),
    (r"^when(?:ever)? .*deals? combat damage", {"combat_damage"}),
    (r"^when(?:ever)? .*\bdeals?\b(?! combat damage)", {"deals_damage"}),
    (r"^when(?:ever)? .*(?:is|are|you're|you are) dealt (?:combat |noncombat )?damage", {"dealt_damage"}),
    (r"^when(?:ever)? .*damage is dealt to", {"dealt_damage"}),
    (r"^when(?:ever)? .*\bcasts?\b", {"cast"}),
    (r"^when(?:ever)? .*\bcop(?:y|ies)\b", {"copied"}),
    (r"^when(?:ever)? .*(?:is|are) countered", {"countered"}),
    (r"^when(?:ever)? .*plays? a land", {"land"}),
    (r"^when(?:ever)? .*gains? life", {"lifegain"}),
    (r"^when(?:ever)? .*loses? life", {"lifeloss"}),
    (r"^when(?:ever)? .*pays? life", {"lifeloss"}),
    (r"^when(?:ever)? .*\bdraws?\b", {"draws"}),
    (r"^when(?:ever)? .*(?:becomes?|is|are) tapped", {"tapped"}),
    (r"^when(?:ever)? .*\btaps?\b", {"tapped"}),
    (r"^when(?:ever)? .*(?:becomes?|is|are) untapped", {"untapped"}),
    (r"^when(?:ever)? .*becomes? the target", {"targeted"}),
    (r"^when(?:ever)? .*\bsacrifices?\b", {"sacrifice"}),
    (r"^when(?:ever)? .*\bcycles?\b", {"cycle"}),
    (r"^when(?:ever)? .*turned face up", {"faceup"}),
    (r"^when(?:ever)? .*\bdiscards?\b", {"discard"}),
    (r"^when(?:ever)? .*counters? (?:is|are) put on|^when(?:ever)? .*\bput (?:a|an|one or more|two or more|\w+) (?:[\w+/\-]+ )?counters?\b", {"counter"}),
    (r"^when(?:ever)? .*counters? (?:is|are) removed|^when(?:ever)? .*\bremoves? (?:a|an|one or more|\w+) (?:[\w+/\-]+ )?counters?\b", {"counter_removed"}),
    (r"^when(?:ever)? .*\bproliferate\b", {"proliferate"}),
    (r"^when(?:ever)? .*leaves? (?:your|a|the) graveyard", {"leaves_graveyard"}),
    (r"^when(?:ever)? .*\bexiled?\b", {"exiled"}),
    (r"^when(?:ever)? .*commits? a crime", {"crime"}),
    (r"^when(?:ever)? .*\bexpend\b", {"expend"}),
    (r"^when(?:ever)? .*\bunlock", {"door"}),
    (r"^when(?:ever)? .*\btransforms?\b", {"transform"}),
    (r"^when(?:ever)? .*tokens? (?:is|are|enters?|would)|^when(?:ever)? .*creates? (?:a|an|one or more|two or more)\b.*\btokens?\b", {"token"}),
    (r"^when(?:ever)? .*\bmonarch\b", {"monarch"}),
    (r"^when(?:ever)? .*\b(?:scry|surveil)\b", {"scry"}),
    (r"^when(?:ever)? .*\broll", {"dice"}),
    (r"^when(?:ever)? .*\bflip", {"coin"}),
    (r"^when(?:ever)? .*\bexplores?\b", {"explore"}),
    (r"^when(?:ever)? .*\benergy\b", {"energy"}),
    (r"^when(?:ever)? .*phases? (?:in|out)", {"phase"}),
    (r"^when(?:ever)? .*becomes? attached|^when(?:ever)? .*\battach\b", {"attach"}),
    (r"^when(?:ever)? .*\bactivates?\b", {"activated"}),
    (r"^when(?:ever)? .*\bshuffles?\b", {"shuffle"}),
    (r"^when(?:ever)? .*\bsearch", {"search"}),
    (r"^when(?:ever)? .*\bmutates?\b", {"mutate"}),
    (r"^when(?:ever)? .*manifests? dread", {"manifest"}),
    (r"^when(?:ever)? .*\bforages?\b", {"forage"}),
    (r"^when(?:ever)? .*gives? a gift", {"gift"}),
    (r"^when(?:ever)? .*collects? evidence", {"evidence"}),
    (r"^when(?:ever)? .*\bdiscovers?\b", {"discover"}),
    (r"^when(?:ever)? .*\b(?:crews?|saddles?)\b", {"crew"}),
    (r"^when(?:ever)? .*becomes? (?:day|night)", {"daynight"}),
    (r"^when(?:ever)? .*\bregenerates?\b", {"regenerate"}),
    (r"^when(?:ever)? .*tempts? you", {"ring"}),
    (r"^when(?:ever)? .*becomes? plotted", {"plot"}),
    (r"^when(?:ever)? .*complete a dungeon", {"dungeon"}),
    (r"^when(?:ever)? .*(?:gain|lose) control of", {"control"}),
    (r"^when(?:ever)? .*\bvote\b", {"vote"}),
    (r"^when(?:ever)? .*solves? a case", {"case"}),
    (r"^when(?:ever)? .*cumulative upkeep", {"cumulative"}),
    (r"^when(?:ever)? .*chaos ensues", {"chaos"}),
    (r"^when(?:ever)? .*planeswalk away", {"planeswalk"}),
    (r"^when(?:ever)? .*set in motion", {"scheme"}),
    (r"^when(?:ever)? .*visit", {"attraction"}),
]
_ORACLE_TRIGGER_CLASSES = [(re.compile(p), frozenset(c)) for p, c in _ORACLE_TRIGGER_CLASSES]
_ORACLE_TRIG_LINE = re.compile(r"^(?:[^—\n]{1,40} — )?(when(?:ever)? |at (?:the beginning|end) of )", re.I)

# The engine's documented spellings of a clause, added to what the rows read:
# "a land enters" as `LandPlayed`; "a source deals damage to a player" as the
# recipient-keyed `PlayerDamaged` / `DealtDamage` with a dealer filter (never
# the combat-only kinds); "becomes blocked by a creature" as a per-blocker
# `Blocks` with `TriggerBlocksSource`; "cast a spell that targets" as
# `BecameTarget`.
def _lenient(cond, classes):
    if classes == {"etb"} and re.search(r"\bland\b", cond):
        classes.add("land")
    if classes == {"deals_damage"}:
        classes.add("dealt_damage")
    if classes == {"combat_damage"} and "to you" in cond:
        classes.add("dealt_damage")
    if classes == {"blocked"} and "becomes blocked by" in cond:
        classes.add("blocks")
    if classes == {"cast"} and "target" in cond:
        classes.add("targeted")
    return classes

def trigger_literals(body):
    """Every `TriggeredAbility { .. }` literal in the card's own
    `triggered_abilities: vec![..]`; `None` when the field is absent or any
    element is a helper call."""
    m = own_field(body, r"triggered_abilities:")
    if m is None:
        return None
    vec = vec_after(body, m.start())
    if vec is None:
        return None
    for item in top_level_items(vec):
        if item.strip() and not item.strip().startswith("TriggeredAbility {"):
            return None
    out, i = [], 0
    while True:
        j = vec.find("TriggeredAbility {", i)
        if j < 0:
            break
        end = bracket_span(vec, j + len("TriggeredAbility "))
        out.append(vec[j:end])
        i = end
    return out

def trigger_event_exprs(body):
    """Per literal, the text of its `event:` field (to the literal's next
    depth-0 comma); `None` when the vec has a helper call or a literal has
    no `event:`."""
    lits = trigger_literals(body)
    if lits is None:
        return None
    out = []
    for lit in lits:
        k = literal_depth1_field(lit, "event:")
        if k is None:
            return None
        expr, depth = lit[k + len("event:"):], 0
        for idx, ch in enumerate(expr):
            if ch in "([{":
                depth += 1
            elif ch in ")]}":
                depth -= 1
            if depth < 0 or (ch == "," and depth == 0):
                expr = expr[:idx]
                break
        out.append(expr)
    return out

def trigger_kinds(body):
    """Per literal, the class of its `event:` kind; `None` when any literal's
    event is a helper call or an unclassed kind."""
    exprs = trigger_event_exprs(body)
    if exprs is None:
        return None
    out = []
    for expr in exprs:
        m = re.search(r"EventKind::(\w+)", expr)
        if not m:
            return None
        if m.group(1) == "StepBegins":
            s = re.search(r"TurnStep::(\w+)", expr)
            cls = _STEP_CLASS.get(s.group(1)) if s else None
        else:
            cls = _KIND_CLASS.get(m.group(1))
        if cls is None:
            return None
        out.append(cls)
    return out

def ref_trigger_kinds(card, face=None):
    """Per oracle trigger line, the set of classes it accepts; `None` when a
    line reads as a trigger and no row classes it."""
    text = (face or card).get("oracle_text")
    if text is None:
        return None
    text = re.sub(r"\([^)]*\)", "", text)
    out = []
    for line in text.split("\n"):
        line = line.strip()
        m = _ORACLE_TRIG_LINE.match(line)
        if not m:
            continue
        cond = line[m.start(1):].split(",")[0].lower()
        classes = set()
        for rx, cls in _ORACLE_TRIGGER_CLASSES:
            if rx.match(cond):
                classes |= cls
        # An unread clause, or one that names two events ("enters or
        # attacks", "at the beginning of your upkeep and whenever …") that a
        # card may spell as one literal or two: the card is not compared.
        if not classes or (len(classes) > 1 and re.search(r"\b(?:or|and)\b", cond)) \
                or re.search(r"\b(?:and|or) (?:when(?:ever)?|at the beginning)\b", cond):
            return None
        out.append(frozenset(_lenient(cond, classes)))
    return out

def trigger_mismatch(code, ref):
    """No one-to-one assignment of literals to oracle lines accepts every
    literal's class (both lists are short; backtracking is fine)."""
    def fit(i, free):
        if i == len(code):
            return True
        return any(code[i] in ref[j] and fit(i + 1, free - {j}) for j in free)
    return not fit(0, frozenset(range(len(ref))))

# ── trigger scopes ──────────────────────────────────────────────────────────
# The same literals, read for their `EventScope` against the clause's
# subject: "this creature" / "you" / "another … you control" / "… you
# control" / "an opponent" / "a player, a creature". Whose event the trigger
# listens to — the column the `trig` class does not see. A literal on
# `AnyPlayer` *with a filter* is not compared: the filter is usually the scope
# ("a creature an opponent controls" as `AnyPlayer` + `ControlledByOpponent`).
_SCOPE_CLASS = {
    "SelfSource": "self", "YourControl": "yours", "AnotherOfYours": "another",
    "OpponentControl": "opp", "AnyPlayer": "any", "ActivePlayer": "active",
    "EnchantedBySource": "ench",
}

def trigger_scopes(body):
    exprs = trigger_event_exprs(body)
    if exprs is None:
        return None
    out = []
    for expr in exprs:
        m = re.search(r"EventScope::(\w+)", expr)
        cls = _SCOPE_CLASS.get(m.group(1)) if m else None
        # `YouAttack` is "whenever you attack" on either seat spelling; an
        # `AnyPlayer` narrowed by `.from_opponent()` is the opponent scope.
        if "EventKind::YouAttack" in expr and cls in ("self", "yours"):
            cls = "yours"
        elif cls == "any" and re.search(r"from_opponent|actor_is_opponent", expr):
            cls = "opp"
        elif cls is None or (cls == "any" and re.search(r"filter|dealt_by", expr)):
            return None
        out.append(cls)
    return out

def _clause(text):
    """The trigger condition: up to the first comma that is not inside a
    list ("Rabbits, Bats, and/or Mice", "nontoken, non-Angel")."""
    i = 0
    while True:
        j = text.find(",", i)
        if j < 0:
            return text
        rest = text[j + 1:].lstrip()
        if re.match(r"(?:and|or|and/or)\b|[\w'-]+,|non-?\w", rest):
            i = j + 1
            continue
        return text[:j]

def ref_trigger_scopes(card, face=None):
    """Per oracle trigger line, the scope classes its subject accepts. A
    step trigger's `ActivePlayer` is "your step" in `fire_step_triggers`, so
    "your upkeep" accepts it; a clause naming two subjects ("this creature or
    another Ally you control") and the targeting / attachment / "causes you
    to" shapes, whose scope is the *caster's*, are not compared."""
    text = (face or card).get("oracle_text")
    if text is None:
        return None
    text = re.sub(r"\([^)]*\)", "", text)
    full = (face or card).get("name", "").lower()
    short = full.split(",")[0].strip()
    this = r"(?:this \w+|it|" + re.escape(short) + r")\b"
    out = []
    for line in text.split("\n"):
        line = line.strip()
        m = _ORACLE_TRIG_LINE.match(line)
        if not m:
            continue
        cond = _clause(line[m.start(1):].lower())
        if re.search(r"becomes? the target|\btargets?\b|attached to|causes? you to|\bor (?:an)?other\b|enchanted|^when you control", cond) \
                or re.search(r"\b(?:and|or) (?:when(?:ever)?|at the beginning)\b", cond):
            return None
        classes = set()
        if cond.startswith("at "):
            rider = line.lower()
            if re.search(r"\byour\b", cond) or "if it's your turn" in rider:
                classes |= {"self", "yours", "active"}
            elif "if it's an opponent's turn" in rider or "if it's not your turn" in rider:
                classes |= {"opp"}
            elif re.search(r"enchanted", cond):
                classes |= {"ench"}
            elif re.search(r"opponent", cond):
                classes |= {"opp"}
            elif re.search(r"\b(?:each|the)\b", cond):
                classes |= {"any"}
        else:
            subj = re.sub(r"^when(?:ever)? ", "", cond)
            if re.match(this, subj) or re.search(r"\bthis (?!turn|way|game|combat|step|phase)\w+|\b" + re.escape(short) + r"\b", subj) \
                    or subj.startswith("you cast this spell"):
                classes |= {"self"}
            if re.match(r"you(?:'re| )", subj) and not subj.startswith("you cast this spell"):
                classes |= {"yours", "self"}
            if subj.startswith("enchanted "):
                classes |= {"ench"}
            # "a creature you control" is spelled `AnotherOfYours` by any
            # source that cannot be its own subject (an enchantment, an
            # Equipment); accept both for the whole family.
            if re.search(r"\b(?:you control|under your control|your graveyard|your library|your hand)\b", subj):
                classes |= {"another", "yours"}
            if re.search(r"\b(?:an|each|one or more) opponents?\b|opponents?(?:'s)? \b|you don't control|\battacks you\b", subj):
                classes |= {"opp"}
            # A damage event keyed on its recipient: "to you" is your seat
            # (`PlayerDamaged` / `ControllerDealtCombatDamage` on SelfSource or
            # YourControl), "to an opponent" the opponent's.
            if re.search(r"damage to you\b", subj):
                classes |= {"opp", "yours", "self"}
            if re.search(r"damage to an opponent\b", subj):
                classes |= {"opp", "yours"}
            if re.search(r"^you .*\banother\b", subj):
                classes |= {"another"}
            if not classes and re.match(r"(?:a|an|one or more|two or more|each|any) ", subj):
                classes |= {"any"}
        # "this or another …", "you or an opponent": subjects on different
        # seats, which a card may spell as one literal or two. "one or more"
        # and "Ninja or Rogue" are not that.
        seats = {frozenset({"self", "yours", "active", "another"}), frozenset({"opp"}), frozenset({"any"}), frozenset({"ench"})}
        spanned = sum(1 for s in seats if classes & s)
        if not classes or (spanned > 1 and re.search(r"\b(?:or|and)\b", cond)):
            return None
        out.append(frozenset(classes))
    return out

# ── trigger filters ─────────────────────────────────────────────────────────
# The type words of a literal's filter (`R::Creature`, `HasCreatureType(Goblin)`,
# `NotToken`, `HasKeyword(Flying)` …) against the type words of the clause
# ("another nontoken Goblin creature you control"). Neutral words the kind
# implies — permanent, card, spell, counter — are not compared, "creature" is
# implied by a creature-only kind on the code side and by a creature type on
# the oracle side, and a literal whose filter the reader cannot name (a
# `Not(..)`, a `HasName`, a power / mana-value bound, a helper) is skipped.
_CREATURE_TYPES = ['Advisor', 'Aetherborn', 'Alien', 'Ally', 'Angel', 'Antelope', 'Ape', 'Archer', 'Archon', 'Armadillo', 'Army', 'Artificer', 'Assassin', 'AssemblyWorker', 'Atog', 'Aurochs', 'Avatar', 'Azra', 'Badger', 'Balloon', 'Barbarian', 'Bard', 'Basilisk', 'Bat', 'Bear', 'Beast', 'Beaver', 'Beeble', 'Berserker', 'Bird', 'Bison', 'Blinkmoth', 'Boar', 'Book', 'Bringer', 'Brushwagg', 'Camel', 'Capybara', 'Carrier', 'Cat', 'Centaur', 'Chimera', 'Citizen', 'Cleric', 'Clown', 'Cockatrice', 'Construct', 'Coward', 'Coyote', 'Crab', 'Crocodile', 'Cyclops', 'Dalek', 'Dauthi', 'Demigod', 'Demon', 'Detective', 'Devil', 'Dinosaur', 'Djinn', 'Doctor', 'Dog', 'Dragon', 'Drake', 'Dreadnought', 'Drix', 'Drone', 'Druid', 'Dryad', 'Dwarf', 'Efreet', 'Egg', 'Elder', 'Eldrazi', 'Elemental', 'Elephant', 'Elf', 'Elk', 'Employee', 'Eye', 'Fae', 'Faerie', 'Ferret', 'Fish', 'Flagbearer', 'Fox', 'Fractal', 'Frog', 'Fungus', 'Gargoyle', 'Giant', 'Giraffe', 'Glimmer', 'Gnome', 'Goat', 'Goblin', 'God', 'Golem', 'Gorgon', 'Gremlin', 'Griffin', 'Hag', 'Halfling', 'Hamster', 'Harpy', 'Hellion', 'Hero', 'Hippo', 'Hippogriff', 'Homarid', 'Homunculus', 'Horror', 'Horse', 'Hound', 'Human', 'Hydra', 'Hyena', 'Illusion', 'Imp', 'Incarnation', 'Inkling', 'Insect', 'Jackal', 'Jellyfish', 'Juggernaut', 'Kangaroo', 'Kavu', 'Kirin', 'Kithkin', 'Knight', 'Kobold', 'Kor', 'Kraken', 'Lamia', 'Lammasu', 'Leech', 'Lemur', 'Leviathan', 'Lhurgoyf', 'Licid', 'Lion', 'Lizard', 'Llama', 'Mammoth', 'Manticore', 'Masticore', 'Mercenary', 'Merfolk', 'Metathran', 'Minion', 'Minotaur', 'Mite', 'Mole', 'Monger', 'Mongoose', 'Monk', 'Monkey', 'Moogle', 'Moonfolk', 'Mount', 'Mouse', 'Mutant', 'Myr', 'Mystic', 'Naga', 'Nautilus', 'Nephilim', 'Nightmare', 'Nightstalker', 'Ninja', 'Noble', 'Noggle', 'Nomad', 'Nymph', 'Octopus', 'Ogre', 'Ooze', 'Orc', 'Orgg', 'Otter', 'Ouphe', 'Ox', 'Oyster', 'Pangolin', 'Peasant', 'Pegasus', 'Performer', 'Pest', 'Phelddagrif', 'Phoenix', 'Phyrexian', 'Pilot', 'Pincher', 'Pirate', 'Plant', 'Platypus', 'Porcupine', 'Possum', 'Praetor', 'Processor', 'Qu', 'Rabbit', 'Raccoon', 'Ranger', 'Rat', 'Rebel', 'Reflection', 'Rhino', 'Robot', 'Rogue', 'Sable', 'Salamander', 'Samurai', 'Sand', 'Saproling', 'Satyr', 'Scarecrow', 'Scientist', 'Scion', 'Scorpion', 'Scout', 'Seal', 'Serpent', 'Servo', 'Shade', 'Shaman', 'Shapeshifter', 'Shark', 'Sheep', 'Siren', 'Skeleton', 'Skunk', 'Slith', 'Sliver', 'Sloth', 'Slug', 'Snail', 'Snake', 'Soldier', 'Soltari', 'Sorcerer', 'Spawn', 'Specter', 'Spellshaper', 'Sphinx', 'Spider', 'Spike', 'Spirit', 'Sponge', 'Squid', 'Squirrel', 'Starfish', 'Surrakar', 'Survivor', 'Symbiote', 'Tentacle', 'Thalakos', 'Thopter', 'Thrull', 'Tiefling', 'Toy', 'Treefolk', 'Trilobite', 'Troll', 'Turtle', 'Tyranid', 'Unicorn', 'Vampire', 'Varmint', 'Vedalken', 'Viashino', 'Villain', 'Volver', 'Wall', 'Warlock', 'Warrior', 'Weasel', 'Weird', 'Werewolf', 'Whale', 'Wizard', 'Wolf', 'Wolverine', 'Wombat', 'Worm', 'Wraith', 'Wurm', 'Yeti', 'Zombie', 'Zubera']
_FILTER_KEYWORDS = ["Flying", "Trample", "Haste", "Vigilance", "Deathtouch", "Lifelink", "Reach", "Menace",
                    "Defender", "Flash", "Hexproof", "Indestructible", "FirstStrike", "DoubleStrike"]
_CREATURE_KINDS = {"CreatureDied", "CreatureSacrificed", "CreatureLeavesBattlefieldNotDying", "Attacks", "Blocks",
                   "BecomesBlocked", "AttacksAndIsntBlocked", "DealsCombatDamageToPlayer", "DealsCombatDamageToCreature",
                   "DealsDamageToPlayer", "DealsDamageToCreature", "DealsDamage", "DealsCombatDamage", "DealtDamage",
                   "DealtCombatDamage", "CreatureOrArtifactDied", "Regenerated", "TurnedFaceUp", "Mutated", "Explored"}
_SUBJECT_KINDS = {"BecameTarget", "ChoseTargets", "PlayerDamaged", "PlayerDealtNoncombatDamage", "ControllerDealtCombatDamage",
                  "YourInstantOrSorceryDealtDamage", "YourInstantOrSorceryDealtDamageToPlayer", "YourSourceDealtNoncombatDamageEqualToToughness",
                  "LandPutIntoGraveyard", "AbilityActivated", "BlocksNOrMore", "BecomesBlockedByNOrMore", "StepBegins", "CardCycled",
                  "SpellCopied", "SpellCountered", "AuraAttached", "AuraAttachedToAny", "BecameAttached", "CounterAdded", "AnyCounterAdded",
                  "CounterRemoved", "TokenCreated", "CrewsOrSaddles", "DealtDamage", "DealtCombatDamage", "Transformed", "TurnedFaceUp"}
_SIMPLE_REQ = {"Creature": "creature", "Artifact": "artifact", "Enchantment": "enchantment", "Land": "land",
               "Planeswalker": "planeswalker", "Instant": "instant", "Sorcery": "sorcery", "NotToken": "nontoken",
               "IsToken": "token", "Noncreature": "noncreature", "Nonland": "nonland", "Colorless": "colorless",
               "Multicolored": "multicolored", "IsBasicLand": "basic", "Legendary": "legendary",
               "NonArtifact": "nonartifact", "NonEnchantment": "nonenchantment", "NonLegendary": "nonlegendary",
               "NonCreature": "noncreature", "NonLand": "nonland", "NonIsToken": "nontoken", "NonIsBasicLand": "nonbasic"}
_COLORS = {"White": "white", "Blue": "blue", "Black": "black", "Red": "red", "Green": "green"}
_UNREADABLE_REQ = re.compile(r"\b(?:Not|HasName|ToughnessAt|WithCounter|InYourGraveyard|InGraveyard|SpellTargetsMatching|IsHostOfSource|"
                             r"Tapped|IsAttacking|DamagedBySource|Any|Player|OpponentPlayer|IsSource|EntityMatches \{ what: Selector::(?!TriggerSource))")
_NOT_REQ = re.compile(r"(?:(?:R|SelectionRequirement)::)?Not\(Box::new\((?:R|SelectionRequirement)::(\w+)\)\)")
_BOUND_WORD = {"PowerAtLeast": "power>=%d", "PowerAtMost": "power<=%d", "ManaValueAtLeast": "mv>=%d", "ManaValueAtMost": "mv<=%d"}
_ORACLE_BOUND = re.compile(r"\b(power|mana value|toughness) (\d+) or (greater|less)\b")

def _plural(w):
    if w.endswith("f"): return {w, w[:-1] + "ves"}
    if w.endswith("y"): return {w, w[:-1] + "ies"}
    if w.endswith(("s", "x", "ch", "sh")): return {w, w + "es"}
    if w == "mouse": return {w, "mice"}
    if w == "merfolk" or w == "kithkin" or w == "moonfolk": return {w}
    return {w, w + "s"}
_LAND_TYPE_WORDS = {"plains", "island", "swamp", "mountain", "forest", "gate", "desert", "lair", "locus", "cave", "sphere", "town"}
_ORACLE_TYPE_WORDS = {}
for _w in _LAND_TYPE_WORDS:
    for _f in _plural(_w): _ORACLE_TYPE_WORDS[_f] = _w
for _w in list(_SIMPLE_REQ.values()) + list(_COLORS.values()) + ["equipment", "aura", "vehicle", "food", "clue", "treasure", "blood", "map", "powerstone", "saga", "mount"]:
    for _f in _plural(_w): _ORACLE_TYPE_WORDS[_f] = _w
for _ct in _CREATURE_TYPES:
    for _f in _plural(_ct.lower()): _ORACLE_TYPE_WORDS[_f] = _ct.lower()
for _k in _FILTER_KEYWORDS:
    _ORACLE_TYPE_WORDS[re.sub(r"(?<!^)(?=[A-Z])", " ", _k).lower()] = _k.lower()

PRED_FN = re.compile(r"\nfn (\w+)\(\)\s*->\s*(?:crate::effect::)?Predicate\s*\{")

def pred_fn_table(text):
    """`{fn_name: body}` for a file-local `fn cast_is_instant_or_sorcery() ->
    Predicate { .. }`, so a `.with_filter(helper())` reads as its body."""
    out = {}
    for m in PRED_FN.finditer(text):
        end = bracket_span(text, m.end() - 1)
        out[m.group(1)] = text[m.end():end - 1].strip()
    return out

# The members of a `Predicate::All(vec![..])` that only *condition* the trigger
# ("if it's your turn", "if it's the second spell") and name no type word; a
# conjunction of these plus one `EntityMatches` reads as the `EntityMatches`.
_CONDITION_PRED = re.compile(r"^(?:crate::effect::)?Predicate::(?:IsTurnOf|CurrentStepIs|SpellsCastThisTurnEquals|ValueAtMost|ValueAtLeast|"
                             r"ValueEquals|PlayerDrewAtLeastThisTurn|SourceClassLevelAtLeast|ExpendReached|AttackedWithCountAtLeast|"
                             r"DeliriumActive|Not\(Box::new\((?:crate::effect::)?Predicate::(?:IsTurnOf|CurrentStepIs)\b)")

def _flatten_all(filt):
    """`.with_filter(Predicate::All(vec![A, B]))` -> the entity members joined
    as one filter text, or `None` when a member is neither an entity match
    nor a bare condition."""
    m = re.match(r"^(\.with_filter\(|filter: Some\()\s*(?:crate::effect::)?Predicate::All\(vec!\[", filt)
    if not m:
        return filt
    end = bracket_span(filt, m.end() - 1)
    members = [x.strip() for x in top_level_items(filt[m.end():end - 1]) if x.strip()]
    ents = [x for x in members if re.match(r"^(?:crate::effect::)?Predicate::(?:EntityMatches\b|CastSpellMatches\()", x)]
    if len(ents) != 1 or any(not _CONDITION_PRED.match(x) for x in members if x not in ents):
        return None
    return m.group(1) + ents[0] + filt[end:]

def trigger_filter_words(body, predfns=None):
    """Per literal, (kind, the filter's type words); `None` for a literal the
    reader cannot name."""
    exprs = trigger_event_exprs(body)
    if exprs is None:
        return None
    out = []
    for expr in exprs:
        m = re.search(r"EventKind::(\w+)", expr)
        if not m:
            return None
        kind = m.group(1)
        # The source's own type is not a filter ("When this creature enters");
        # a kind or scope that carries its subject (a targeting, a damage
        # recipient, an activation) is not read either.
        if re.search(r"EventScope::(?:SelfSource|EnchantedBySource|FromYourGraveyard)", expr) \
                or kind in _SUBJECT_KINDS or not re.search(r"EventScope::(?:YourControl|AnyPlayer|OpponentControl|AnotherOfYours|ActivePlayer)", expr) \
                or expr.lstrip().startswith("EventSpec {"):
            out.append(None)
            continue
        f = expr.find(".with_filter(")
        if f < 0:
            f = expr.find("filter: Some(")
        filt = expr[f:] if f >= 0 else ""
        if predfns and filt:
            filt = re.sub(r"\b([a-z_]\w*)\(\)", lambda m: predfns.get(m.group(1), m.group(0)), filt)
        if filt:
            filt = _flatten_all(filt)
            if filt is None:
                return None
        filt = filt.replace("IsToken.negate()", "NotToken").replace("Not(Box::new(R::IsToken))", "NotToken") \
                   .replace("Not(Box::new(SelectionRequirement::IsToken))", "NotToken")
        # `Not(Box::new(R::Creature))` is the oracle's "noncreature"; only the
        # simple type words have a printed negation, anything else stays
        # unreadable. `_NOT_REQ` rewrites the readable ones before the gate.
        filt = _NOT_REQ.sub(lambda m: "R::Non" + m.group(1) if "Non" + m.group(1) in _SIMPLE_REQ else m.group(0), filt)
        # The cast spell is the subject of a `SpellCast` literal, and its
        # filter is spelled `CastSpellMatches(R)` — the same read as an
        # `EntityMatches` on `TriggerSource`.
        head = re.compile(r"^(?:\.with_filter\(|filter: Some\()\s*(?:crate::effect::)?Predicate::(EntityMatches\b|CastSpellMatches\()")
        if filt and (not head.search(filt)
                     or _UNREADABLE_REQ.search(filt) or "negate()" in filt
                     or ("TriggerSource" not in filt and not head.search(filt).group(1).startswith("CastSpell"))
                     or not re.search(r"(?:R|SelectionRequirement)::", filt) or re.search(r"\b[a-z_]+\(\)", filt)):
            return None
        # "Whenever a Goblin deals combat damage" narrows the dealer, which
        # rides in `.dealt_by(..)` rather than the filter.
        d = expr.find(".dealt_by(")
        if d >= 0:
            filt += expr[d:]
        words = set()
        # A power / mana-value bound is a word of its own on both sides
        # ("power 4 or greater" / `PowerAtLeast(4)`).
        for fn, n in re.findall(r"(PowerAtLeast|PowerAtMost|ManaValueAtLeast|ManaValueAtMost)\((\d+)\)", filt):
            words.add(_BOUND_WORD[fn] % int(n))
        for r in re.findall(r"(?:R|SelectionRequirement)::(\w+)", filt):
            if r in _SIMPLE_REQ: words.add(_SIMPLE_REQ[r])
        for r in re.findall(r"HasCardType\(\s*CardType::(\w+),?\s*\)", filt):
            if r in _SIMPLE_REQ: words.add(_SIMPLE_REQ[r])
        for r in re.findall(r"HasSupertype\(\s*Supertype::(\w+),?\s*\)", filt):
            if r in _SIMPLE_REQ: words.add(_SIMPLE_REQ[r])
        # A land type implies "land" on both sides, as a creature type implies
        # "creature" ("whenever a Mountain becomes tapped" — Lifeblood).
        land_types = {t.lower() for t in re.findall(r"HasLandType\(\s*LandType::(\w+),?\s*\)", filt)}
        if land_types:
            words |= land_types | {"land"}
        words |= {c.lower() for c in re.findall(r"HasCreatureType\(\s*CreatureType::(\w+),?\s*\)", filt)}
        words |= {k.lower() for k in re.findall(r"HasKeyword\(Keyword::(\w+)\)", filt) if k in _FILTER_KEYWORDS}
        words |= {_COLORS[c] for c in re.findall(r"HasColor\(Color::(\w+)\)", filt) if c in _COLORS}
        words |= {s.lower() for s in re.findall(r"HasArtifactSubtype\(\s*ArtifactSubtype::(\w+),?\s*\)", filt)}
        words |= {s.lower() for s in re.findall(r"HasEnchantmentSubtype\(\s*EnchantmentSubtype::(\w+),?\s*\)", filt)}
        if kind in _CREATURE_KINDS or words & {c.lower() for c in _CREATURE_TYPES}:
            words.add("creature")
        if kind == "CreatureOrArtifactDied":
            words |= {"creature", "artifact"}
        if kind == "LandPlayed":
            words.add("land")
        out.append((kind, frozenset(words)))
    return out

def ref_trigger_filter_words(card, face=None):
    """Per oracle trigger line, the type words of its subject clause."""
    text = (face or card).get("oracle_text")
    if text is None:
        return None
    text = re.sub(r"\([^)]*\)", "", text)
    out = []
    for line in text.split("\n"):
        line = line.strip()
        m = _ORACLE_TRIG_LINE.match(line)
        if not m:
            continue
        cond = _clause(line[m.start(1):].lower())
        if cond.startswith("at ") or re.search(r"\bor (?:an)?other\b|\b(?:and|or) when|^when you control", cond):
            out.append(None)
            continue
        # The subject only: what it does something *to* / *by* / *with* is
        # the object ("deals combat damage to a player or planeswalker").
        cond = re.split(r"\b(?:to|by|from|into|onto|for|attacks|blocks|targets) \b", cond)[0]
        words = set()
        for what, n, how in _ORACLE_BOUND.findall(cond):
            if what != "toughness":
                words.add(("power" if what == "power" else "mv") + (">=" if how == "greater" else "<=") + n)
        for w in re.findall(r"[a-z][a-z'-]*", cond):
            if w in _ORACLE_TYPE_WORDS:
                words.add(_ORACLE_TYPE_WORDS[w])
            elif w.startswith("non-") and w[4:] in _ORACLE_TYPE_WORDS:
                words.add("non" + _ORACLE_TYPE_WORDS[w[4:]])
            elif w == "historic":
                words |= {"artifact", "legendary", "saga"}
        if words & _LAND_TYPE_WORDS:
            words.add("land")
        if words & set(c.lower() for c in _CREATURE_TYPES):
            words.add("creature")
        # A Blood / Clue / Food / Treasure is a token by construction.
        if words & {"blood", "clue", "food", "treasure", "map", "powerstone"}:
            words.discard("token")
        out.append(frozenset(words))
    return out

def filter_mismatch(code, ref):
    """Multiset compare of the type words, one-to-one; an oracle line the
    reader skipped (`None`) accepts anything."""
    def fit(i, free):
        if i == len(code):
            return True
        for j in free:
            if ref[j] is None or code[i] is None or code[i][1] == ref[j]:
                if fit(i + 1, free - {j}):
                    return True
        return False
    return not fit(0, frozenset(range(len(ref))))

def ability_mana_costs(body):
    """The mana cost of every `ActivatedAbility { .. }` literal in the card's
    own `activated_abilities: vec![..]`, each as `norm()`'s symbol tuple; a
    literal with no `mana_cost:` (or `ManaCost::default()`) reads `()`. Only
    the literal's own top-level field is read — a `cost(&[..])` nested in its
    effect (a `MayPay`) is not the ability's cost. `None` when the field is
    absent or every ability is a helper call the table cannot open. Found
    Manifold Key (2026-09-07): the card's cost was right and both abilities
    carried the Voltaic Key costs it was written from."""
    m = own_field(body, r"activated_abilities:")
    if m is None:
        return None
    vec = vec_after(body, m.start())
    if vec is None:
        return None
    # A helper call among the elements (`tutor_chain(6, ..)`, a mana
    # ability) is an ability this scan cannot open: compare nothing rather
    # than read the literals beside it as the whole card (Cateran Overlord,
    # Shifting Woodland read that way at the first run).
    for item in top_level_items(vec):
        if item.strip() and not item.strip().startswith("ActivatedAbility {"):
            return None
    out, i = [], 0
    while True:
        j = vec.find("ActivatedAbility {", i)
        if j < 0:
            break
        end = bracket_span(vec, j + len("ActivatedAbility "))
        lit = vec[j:end]
        i = end
        # The literal's own `mana_cost:` at depth 1: skip text inside any
        # nested braces/brackets before looking for it.
        depth, k, found = 0, len("ActivatedAbility {"), None
        while k < len(lit):
            c = lit[k]
            if c in "{[(":
                depth += 1
            elif c in "}])":
                depth -= 1
            elif depth == 0 and lit.startswith("mana_cost:", k):
                found = k
                break
            k += 1
        if found is None:
            # `generic_cost_value: Some(..)` is a value-defined {X} (Bargaining
            # Table's "X is the number of cards in an opponent's hand").
            out.append(("{X}",) if re.search(r"generic_cost_value:\s*Some", lit) else ())
            continue
        mc = re.match(r"mana_cost:\s*cost\(&\[", lit[found:])
        if mc:
            a = found + mc.end() - 1
            args = lit[a + 1:bracket_span(lit, a) - 1]
            syms = [sym(c) for c in re.split(r",(?![^()]*\))", args) if c.strip()]
            if not all(syms):
                return None
            out.append(norm("".join(syms)))
        elif re.match(r"mana_cost:\s*ManaCost::default\(\)", lit[found:]):
            out.append(())
        elif (mn := re.match(r"mana_cost:\s*ManaCost::new\(vec!\[", lit[found:])):
            # Blazing Rootwalla's shape (2026-09-10): the symbols spelled out.
            a = found + mn.end() - 1
            args = lit[a + 1:bracket_span(lit, a) - 1]
            syms = [sym(c) for c in re.split(r",(?![^()]*\))", args) if c.strip()]
            if not all(syms):
                return None
            out.append(norm("".join(syms)))
        else:
            return None
    return out

# An oracle activation line: `{cost}[, {cost}...]: effect`, with an optional
# ability-word prefix ("Delirium — {2}{G}{G}: .."). Reminder text is
# stripped first — "Cycling {2} ({2}, Discard this card: Draw a card.)" would
# otherwise read as a {2} ability the code spells as a `cycling` field.
_ORACLE_ACT = re.compile(r"^(?:[A-Z][a-z]+ — )?(\{[^:\n]*?):\s")

def ref_ability_mana_costs(card, face=None):
    """The mana-bearing activation costs the oracle prints, as `norm()`
    tuples — `{T}`-only, `{Q}` and `{E}` costs drop to `()` and are then
    ignored by the comparison, since the code's mana abilities are helper
    calls the literal scan never sees."""
    text = (face or card).get("oracle_text")
    if text is None:
        return None
    text = re.sub(r"\([^)]*\)", "", text)
    out = []
    for line in text.split("\n"):
        m = _ORACLE_ACT.match(line.strip())
        if not m:
            continue
        cost = m.group(1)
        syms = [s for s in re.findall(r"\{[^}]+\}", cost) if s not in ("{T}", "{Q}", "{E}")]
        out.append(norm("".join(syms)))
    return out

# ── numbers ─────────────────────────────────────────────────────────────────
# The amounts an ability prints ("deals 3 damage", "draw two cards", "put
# two +1/+1 counters", "gets +2/+2") against the integer literals in the
# code's literal for the same ability. Only an oracle number the code does
# not carry anywhere in that literal is a row — the direction a wrong-amount
# defect reads (Dynavolt Tower's 4 for a printed 3; Witch's Cauldron's "gain
# the toughness" for a printed 1). Digits inside a mana symbol, a token's
# N/N (the `tok` column), a loyalty prefix, reminder text and "Choose N —"
# are not amounts. The word "one" is skipped: it is "one or more" / "one of
# them" far more often than an amount, and the code's 1 is usually implicit.
_NUM_WORDS = {"two": 2, "three": 3, "four": 4, "five": 5, "six": 6, "seven": 7,
              "eight": 8, "nine": 9, "ten": 10, "eleven": 11, "twelve": 12,
              "thirteen": 13, "fifteen": 15, "twenty": 20, "twice": 2}
_CAMEL_NUM = {"Zero": 0, "One": 1, "Two": 2, "Three": 3, "Four": 4, "Five": 5,
              "Six": 6, "Seven": 7, "Eight": 8, "Nine": 9, "Ten": 10}

def code_numbers(lit):
    """Every integer the literal spells: bare literals, a trailing digit on an
    identifier (`scry1`), and the number words of a CamelCase variant
    (`PlusOnePlusOne` -> 1). The literal's own `mana_cost:` is not an amount."""
    lit = re.sub(r"mana_cost:\s*cost\(&\[[^\]]*\]\)", "", lit)
    lit = re.sub(r"//[^\n]*", "", lit)
    nums = {int(n) for n in re.findall(r"(?<![\w.])-?\d+(?![\w.])", lit)}
    nums |= {int(n) for n in re.findall(r"(?<=[a-z_])(\d+)\b", lit)}
    # "Search your library for up to two ..." as two `search_*` calls.
    searches = len(re.findall(r"\bsearch_\w+\(|Effect::Search\b", lit))
    if searches > 1:
        nums.add(searches)
    for w, n in _CAMEL_NUM.items():
        if re.search(rf"[A-Z][a-z]*{w}(?=[A-Z]|\b)|\b{w}(?=[A-Z])|::{w.upper()}\b|\b{w.lower()}_", lit):
            nums.add(n)
    # "two target creatures" as a second target slot.
    nums |= {int(k) + 1 for k in re.findall(r"\bslot:\s*(\d+)", lit)}
    return {abs(n) for n in nums}

# Threshold clauses an ability word defines, met by a predicate helper
# (`FormidableActive`, `delirium()`, `coven()`), and the alternative-cost
# sentences a spell's `alternative_cost:` carries, not its `effect:`.
_ORACLE_IDIOMS = [
    r"four or more card types among cards in your graveyard",
    r"three or more artifacts",
    r"three or more creatures with different powers",
    r"a creature with power 4 or greater",
    r"(?:you have )?5 or less life",
    r"two or more instant and/or sorcery cards in your graveyard",
    r"creatures you control have total power 8 or greater",
    r"seven or more cards in your graveyard",
    r"[^.]*rather than pay[^.]*\.",
    r"[^.]*costs? [^.]*less to cast[^.]*\.",
    r"[^.]*for each [^.]*\.",
    r"\bwith (?:one|two|three|four|five|\d+) or more \w+ counters?\b",
    r"three or more poison counters",
    r"two or more tapped creatures",
    r"two or more nonland permanents entered the battlefield",
    r"(?:if )?two or more [^,.]* (?:are )?tied\b",
    r"(?:into|in) two piles",
    r"(?:can't|cannot) be 0\b",
    r"\bany number\b",
    r"\bat least (?:two|three|four) other\b",
    r"\bamong one, two, or three\b",
    r"\btoxic \d\b",
    r"\bcollect evidence \d+\b",
    # A granted or token ability in quotes is a nested literal the token
    # reader strips from the code.
    r"\"[^\"]*\"",
]

def oracle_numbers(text, name=None):
    """The amounts one oracle ability prints, as a set."""
    if name:
        text = text.replace(name, " ")
    text = re.sub(r"\{[^}]*\}", " ", text)
    text = re.sub(r"\bChoose (?:one|two|three|four|any number)\b[^.]*", " ", text)
    for rx in _ORACLE_IDIOMS:
        text = re.sub(rx, " ", text, flags=re.I)
    # A token's or a face's N/N is the `tok` / `P/T` column; a signed +N/+M
    # pump or counter is an amount.
    text = re.sub(r"(?<![+\-−])\b\d+/\d+\b", " ", text)
    text = re.sub(r"[+\-−](\d+)/[+\-−](\d+)", r" \1 \2 ", text)
    text = re.sub(r"\b(\d+),(\d{3})\b", r"\1\2", text)
    nums = {int(n) for n in re.findall(r"\b\d+\b", text)}
    for w, n in _NUM_WORDS.items():
        if re.search(rf"\b{w}\b", text, re.I):
            nums.add(n)
    return nums

def _oracle_ability_lines(text):
    """The oracle split into abilities: `(kind, effect text)` with kind
    `act` for a `{cost}: effect` line, `trig` for a When / Whenever / At
    line, `other` for the rest (keyword lines, statics, a spell's text). A
    `•` mode bullet joins the line above it; loyalty and level lines drop."""
    text = re.sub(r"\([^)]*\)", "", text)
    lines = []
    for line in text.split("\n"):
        line = line.strip()
        if not line:
            continue
        if line.startswith("•") and lines:
            lines[-1] = (lines[-1][0], lines[-1][1] + " " + line)
            continue
        if re.match(r"^(?:[+\-−]?\d+|0):", line) or line.upper().startswith("LEVEL "):
            continue
        # A keyword line with its number ("Suspend 4—{G}", "Dredge 3",
        # "Casualty 3", "Awaken 4—{4}{W}", "Ward 2") is a `keywords:` entry.
        if re.match(r"^(?:[A-Z][a-z]+(?: [a-z]+)? \d+\b|Cumulative upkeep|Kicker|Buyback|Cycling|Flashback|Overload|Escape|Prowl|Madness|Morph|Ninjutsu|Unearth|Bestow|Evoke|Equip|Fortify|Transmute|Channel|Splice|Recover|Retrace|Scavenge|Outlast|Dash|Surge|Emerge|Embalm|Eternalize|Jump-start|Spectacle|Afterlife|Riot|Mutate|Foretell|Boast|Disturb|Cleave|Blitz|Casualty|Prototype|Backup|Toxic|Craft|Plot|Offspring|Impending|Gift)\b", line):
            continue
        m = _ORACLE_ACT.match(line)
        if m:
            lines.append(("act", line[m.end():]))
        elif _ORACLE_TRIG_LINE.match(line):
            lines.append(("trig", line))
        else:
            lines.append(("other", line))
    return lines

def local_bindings(body):
    """The card body's own `let name = ..;` and `fn name(..) { .. }` items
    before its `CardDefinition`, as `name -> integer set`, so a literal that
    reads `effect: shrink.clone()` or `effect: gain()` carries the numbers
    the binding spells (Cabal Patriarch, Sangromancer)."""
    head = body.split("CardDefinition {", 1)[0]
    out = {}
    for m in re.finditer(r"\b(?:let|fn)\s+(\w+)\b", head):
        k = re.search(r"[\[({=]", head[m.end():])
        if not k:
            continue
        start = m.end() + k.start()
        if head[start] == "=":
            span = re.search(r";\n", head[start:])
            text = head[start: start + span.end()] if span else head[start:]
        else:
            text = head[start: bracket_span(head, start)]
        out[m.group(1)] = code_numbers(text)
    return out

def with_bindings(lit, bindings):
    nums = code_numbers(lit)
    for name, n in bindings.items():
        if re.search(rf"\b{name}\b", lit):
            nums |= n
    return nums

def ability_numbers(body, kind):
    """Per literal of the kind (`act` / `trig`), the integer set the code
    spells (its own and its local bindings'); `None` when the vec has a
    helper call."""
    lits = ability_literals(body) if kind == "act" else trigger_literals(body)
    if lits is None:
        return None
    b = local_bindings(body)
    return [with_bindings(l, b) for l in lits]

def spell_numbers(body):
    """The integer set of an instant's or sorcery's whole body (its
    `effect:`, a `gift:` / kicker branch, the local bindings), minus its
    own `cost:` and name; `None` when the card has no `effect:` literal."""
    if own_field(body, r"(?<![:\w])effect:(?!:)\s*(?:Some\()?") is None:
        return None
    body = re.sub(r"\bcost:\s*cost\(&\[[^\]]*\]\)", "", body)
    body = re.sub(r"\bname:\s*\"[^\"]*\"", "", body)
    return with_bindings(body, local_bindings(body))

def numbers_mismatch(code, ref):
    """No one-to-one assignment of literals to oracle abilities carries every
    oracle amount in the literal it is assigned to."""
    def fit(i, free):
        if i == len(ref):
            return True
        return any(ref[i] <= code[j] and fit(i + 1, free - {j}) for j in free)
    return not fit(0, frozenset(range(len(code))))

def toplevel_keywords(body):
    """Same rule as `toplevel_cost_args`: the card's own literal, not a bound
    one's."""
    m = own_field(body, r"keywords:")
    return vec_after(body, m.start()) if m else None

def toplevel_cost_args(body):
    """The depth-1 `cost: cost(&[...])` inner args of the card's own
    `CardDefinition` literal — never a nested cost (an ability's) and never a
    *bound* literal's (an MDFC back face in a preceding `let`). Goes through
    `own_field`, which owns both of those rules."""
    m = own_field(body, r"cost:\s*cost\(&\[")
    if m is None:
        return None
    start, d, kk = m.end(), 1, m.end()
    while kk < len(body) and d:
        if body[kk] == "[": d += 1
        elif body[kk] == "]": d -= 1
        kk += 1
    return body[start:kk-1]

def face_for(card, name):
    """The `card_faces` entry whose name matches `name`, else None.

    A flip / DFC cache entry carries the FRONT face's stats at the top level,
    so a back-face definition (Tok-Tok, Stabwhisker) has to be compared
    against its own face or every one of them reads as drift.
    """
    for f in card.get("card_faces") or []:
        if f.get("name", "").lower() == name.lower():
            return f
    return None

# Scryfall spells a few creature types with punctuation the enum drops.
TYPE_ALIAS = {"Assembly-Worker": "AssemblyWorker", "Time Lord": "TimeLord"}

def creature_subtypes_ref(card, face=None):
    faces = [face] if face else (card.get("card_faces") or [])
    for tl in [f.get("type_line") for f in faces] + ([] if face else [card.get("type_line")]):
        if tl and "Creature" in tl and "—" in tl:
            raw = tl.split("—", 1)[1].split("//")[0].strip()
            for k, v in TYPE_ALIAS.items():
                raw = raw.replace(k, v)
            return set(raw.split())
    return None

# CR 205.4 / 205.2a — the words left of the em dash, split into the two
# groups the engine models as `Supertype` and `CardType`. `Tribal` is the
# pre-2021 printing of `Kindred`. Anything outside these two sets (a Dungeon,
# a Sticker, an Attraction's "Artifact Attraction" oddities) makes the line
# unreadable for this audit and the card is SKIPPED, not flagged.
# The variant NAME, however the file spells the path. `modern.rs` writes
# `Sup::Legendary` (`use crate::card::Supertype as Sup`), and a scan keyed on
# `Supertype::` read an empty list on every one of those — 30-odd correct
# legends reported as missing the supertype (2026-08-30).
ST_VARIANT = re.compile(r"\b(?:\w+::)*(Legendary|Basic|Snow|World|Ongoing)\b")
CT_VARIANT = re.compile(
    r"\b(?:\w+::)*(Land|Creature|Artifact|Enchantment|Planeswalker|Battle|Instant"
    r"|Sorcery|Kindred|Scheme|Vanguard|Conspiracy|Plane|Phenomenon)\b"
)

REF_SUPERS = {"Basic", "Legendary", "Snow", "World", "Ongoing"}
REF_TYPES = {
    "Artifact", "Battle", "Creature", "Enchantment", "Instant", "Kindred",
    "Tribal", "Land", "Planeswalker", "Sorcery", "Scheme", "Vanguard",
    "Conspiracy", "Plane", "Phenomenon",
}


def type_line_ref(card, face=None):
    """`(supertypes, card_types)` off the type line, or None when unreadable.

    A split / MDFC line ("Creature — Human // Instant") describes two objects,
    so without a matched face there is no single answer and this returns None
    rather than a guess.
    """
    tl = (face or card).get("type_line") or ""
    if not tl or (face is None and "//" in tl):
        return None
    left = tl.split("—", 1)[0].strip()
    words = left.split()
    supers, types = set(), set()
    for w in words:
        if w in REF_SUPERS:
            supers.add(w)
        elif w in REF_TYPES:
            types.add("Kindred" if w == "Tribal" else w)
        else:
            return None
    return (supers, types) if types else None


def ref_keywords(card, face=None):
    # Scryfall lists flip-card keywords at the card level only, so a face with
    # an empty list carries no usable reference — signal "skip" with None.
    if face is not None:
        return set(face["keywords"]) & ENGINE_KW if face.get("keywords") else None
    out = set(card.get("keywords", []) or [])
    for f in card.get("card_faces", []) or []:
        out |= set(f.get("keywords", []) or [])
    return out & ENGINE_KW

def set_of(path):
    rel = path.relative_to(SETS)
    return rel.parts[0] if len(rel.parts) > 1 else rel.stem

FUNC = re.compile(r"pub fn (\w+)\(\)\s*->\s*CardDefinition\s*\{")
CTOR = re.compile(r"\b(CardDefinition|TokenDefinition)\s*\{")
NAME = re.compile(r'name:\s*"((?:[^"\\]|\\.)*)"')

def strip_token_literals(body):
    """`body` with every balanced `TokenDefinition { … }` span blanked out.

    Token literals carry their own name/power/toughness/subtypes/keywords; left
    in place they shadow the card's own fields for every token-minting card.
    """
    out, i = [], 0
    for m in re.finditer(r"\bTokenDefinition\s*\{", body):
        if m.start() < i:
            continue
        out.append(body[i:m.start()])
        j, d = m.end(), 1
        while j < len(body) and d:
            if body[j] == "{":
                d += 1
            elif body[j] == "}":
                d -= 1
            j += 1
        i = j
    out.append(body[i:])
    return "".join(out)

def own_field(body, field):
    """Match for `field:` at the top level of the function's own
    `CardDefinition { … }` literal.

    A nested struct can carry the same key — `StaticAbility { PumpPT { power: 3
    } }` shadows a card's printed `power:` — so the scan tracks brace depth from
    the CardDefinition literal and only accepts depth-1 hits.
    """
    for cm in re.finditer(r"\bCardDefinition\s*\{", body):
        # Skip a literal that is BOUND rather than returned — `let aura_back =
        # CardDefinition { … }`, `back_face: Some(CardDefinition { … })`. Both
        # carry the same `name:` as the card, so the first-literal scan read
        # Bronzehide Lion's Aura back face as the card itself and called a
        # Creature an Enchantment (2026-08-30).
        before = body[: cm.start()].rstrip()
        if before[-1:] in ("=", "("):
            continue
        depth, i = 1, cm.end()
        while i < len(body) and depth:
            ch = body[i]
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    break
            elif depth == 1:
                m = re.compile(field).match(body, i)
                if m:
                    return m
            i += 1
    return None

def card_def_name(body):
    """First `name:` belonging to a CardDefinition, skipping token literals.

    A card that mints a token defines the token first (Roxanne's Meteorite,
    Crib Swap's Shapeshifter), so the naive "first name in the body" reads the
    token's name and audits the wrong card.
    """
    return own_field(body, r'name:\s*"((?:[^"\\]|\\.)*)"')

# ── Local card-shape helpers ────────────────────────────────────────────────
#
# Most set modules build their cards through a file-local helper
# (`fn creature(name, cost, types, p, t) -> CardDefinition`) and pass the
# printed stats positionally: `..creature("Aven Brigadier", cost(&[…]), …)`.
# Those cards carry no top-level `name:` / `power:` field, so the audit used to
# skip them entirely — most of the classic sets went unchecked. `HELPERS`
# records, per file, which positional parameter of each helper feeds which
# CardDefinition field; `inline_helper_call` then splices the call's arguments
# into a synthetic literal the existing field scans can read.

HELPER_DEF = re.compile(
    r"\nfn (\w+)\(([^)]*)\)\s*->\s*CardDefinition\s*\{", re.S
)
FIELD_FROM_PARAM = {
    "name": "name",
    "cost": "cost",
    "power": "power",
    "toughness": "toughness",
    "creature_types": "creature_types",
}

# Literals that can legitimately carry a *printed* characteristic. Anything
# else nested inside the helper's `CardDefinition { … }` — an `Effect`, a
# `StaticAbility`, an `ActivatedAbility` — carries its own `cost:` / `power:`
# and must not claim the helper's parameter of that name.
PRINTED_NEST = ("CardDefinition", "Subtypes")
LIT_NAME = re.compile(r"([A-Za-z_][A-Za-z_0-9]*)\s*$")


def own_helper_fields(body, field_re):
    """Every `field_re` match inside the helper's own `CardDefinition { … }`
    whose enclosing literals are all in `PRINTED_NEST`.

    `helper_table` used a flat `findall`, so `elder_dragon`'s
    `SacrificeSourceUnlessPay { cost: upkeep_cost }` claimed the `cost`
    mapping and the four Legends Elder Dragons read as cost drift against
    their own upkeep (2026-08-30). `creature_types` genuinely sits one level
    down inside `Subtypes { … }`, so this is a name check, not a depth cap.
    """
    pat = re.compile(field_re)
    if not re.search(r"\bCardDefinition\s*\{", body):
        # A helper that only forwards (`bfz::ally` -> `creature(name, …)`) has
        # no literal to be inside of; the flat scan is the whole scan there,
        # and it is what maps `name` for the 106 cards those helpers build.
        yield from pat.finditer(body)
        return
    for cm in re.finditer(r"\bCardDefinition\s*\{", body):
        # Same bound-literal rule as `own_field`.
        if body[: cm.start()].rstrip()[-1:] in ("=", "("):
            continue
        stack, i = ["CardDefinition"], cm.end()
        while i < len(body) and stack:
            ch = body[i]
            if ch == "{":
                # A struct literal is `Name {`; anything else (`if cond {`, a
                # bare block, a closure body) is transparent and inherits the
                # frame it sits in — `tor2::dreams` builds its base through
                # `..if sorcery_speed { sorcery(name, …) }`.
                nm = LIT_NAME.search(body[:i])
                lit = nm.group(1) if nm and nm.group(1)[:1].isupper() else stack[-1]
                stack.append(lit)
                i += 1
                continue
            if ch == "}":
                stack.pop()
                i += 1
                continue
            if all(n in PRINTED_NEST for n in stack):
                m = pat.match(body, i)
                if m:
                    yield m
            i += 1


VECFN = re.compile(
    r"\nfn (\w+)\(\)\s*->\s*Vec<(Supertype|CardType)>\s*\{\s*vec!\[([^\]]*)\]", re.S
)


def vec_fn_table(text):
    """`{fn_name: literal}` for a file-local `fn legendary() -> Vec<Supertype>`.

    `chk2.rs` writes `supertypes: legendary()` on 26 cards. A reader that only
    understands `vec![Supertype::Legendary]` sees an empty list there and
    reports every one of them as a missing supertype — which is how this audit
    first read 25 correct Kamigawa legends as defects (2026-08-30).
    """
    return {m.group(1): m.group(3) for m in VECFN.finditer(text)}


def field_vec(body, field, vecfns):
    """The `vec![…]` body for `field` at depth 1, resolving a `field: helper()`
    indirection through `vecfns`. `None` when the field is absent."""
    m = own_field(body, re.escape(field) + r":")
    if m is None:
        return None
    v = vec_after(body, m.start())
    if v is not None:
        return v
    call = re.match(re.escape(field) + r":\s*(\w+)\(\)", body[m.start():])
    return vecfns.get(call.group(1)) if call else None


def helper_table(text):
    """`({helper_name: {field: positional_index}}, {helper_name: {field: literal}})`.

    The second map is for the fields a helper sets to a *constant* rather than
    from a parameter — `card_types`, `supertypes` — which is how every
    `fn creature` / `fn sorcery` / `fn legend` in this catalog spells them.
    """
    out, consts = {}, {}
    for m in HELPER_DEF.finditer(text):
        params = [
            p.split(":")[0].strip()
            for p in re.split(r",(?![^<>()]*[>)])", m.group(2))
            if p.strip()
        ]
        j, depth = m.end(), 1
        while j < len(text) and depth:
            depth += (text[j] == "{") - (text[j] == "}")
            j += 1
        body = text[m.end() : j]
        mapping = {}
        for field in ("name", "cost", "power", "toughness", "creature_types"):
            # `power: p,` (explicit) or `name,` (shorthand field init). Take the
            # LAST assignment: a helper that also builds a DFC back face
            # (`vanilla_werewolf`) writes the back's fields first and returns
            # the front, so the first match would audit the wrong face.
            hits = [
                m.group(1)
                for m in own_helper_fields(body, r"\b" + field + r":\s*(\w+)\s*[,}]")
            ] or [
                m.group(1) for m in own_helper_fields(body, r"\b(" + field + r")\s*,")
            ]
            hits = [h for h in hits if h in params]
            if hits:
                mapping[field] = params.index(hits[-1])
        # A helper that layers on another helper (`legend` → `creature`)
        # forwards its own parameters; resolve one level.
        base = re.search(r"\.\.(\w+)\(", body)
        if base and base.group(1) in out:
            for field, idx in out[base.group(1)].items():
                arg = split_args(call_args(body, base.group(1)))
                if idx < len(arg) and arg[idx].strip() in params:
                    mapping.setdefault(field, params.index(arg[idx].strip()))
        # `card_types` and `supertypes` are almost never a helper *parameter*
        # — a `fn creature(...)` writes `card_types: vec![CardType::Creature]`
        # as a literal, and a `fn legend(...)` adds
        # `supertypes: vec![Supertype::Legendary]` on top of it. Record those
        # so the two type columns can see the ~11 k cards built through a
        # helper instead of skipping them.
        for field in ("card_types", "supertypes"):
            for mm in own_helper_fields(body, r"\b" + field + r":\s*vec!\["):
                v = vec_after(body, mm.start())
                # A *constant* is a plain list of `CardType::X`. `recent311`'s
                # `spell` helper writes `vec![if sorcery { Sorcery } else
                # { Instant }]`, which is a parameter wearing a literal's
                # clothes — capturing it read 85 cards as both types at once.
                var = ST_VARIANT if field == "supertypes" else CT_VARIANT
                if v is not None and re.fullmatch(r"\s*(?:[\w:]+\s*,?\s*)+", v) and var.findall(v):
                    consts.setdefault(m.group(1), {})[field] = v
                break
        if base and base.group(1) in consts:
            for field, v in consts[base.group(1)].items():
                consts.setdefault(m.group(1), {}).setdefault(field, v)
        if mapping:
            out[m.group(1)] = mapping
    return out, consts

def call_args(body, fname):
    """The raw argument text of the first `fname(` call in `body`."""
    m = re.search(r"\b" + re.escape(fname) + r"\s*\(", body)
    if not m:
        return ""
    i, depth = m.end(), 1
    while i < len(body) and depth:
        depth += (body[i] == "(") - (body[i] == ")")
        i += 1
    return body[m.end() : i - 1]

def split_args(argtext):
    out, depth, cur = [], 0, ""
    for ch in argtext:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur)
    return out

def inline_helper_call(body, helpers, consts=None):
    """Splice a `..helper(args)` call's fields into the body's own literal.

    Returns `body` unchanged when it already carries its own `name:` field or
    uses no known helper — the field scans then behave exactly as before.

    **Every spliced field is guarded on the body not already having one at
    depth 1.** The splice is inserted at the *front* of the card's own
    `CardDefinition { … }`, and a depth-1 scan takes the first hit, so an
    unguarded splice would shadow a card that overrides the helper
    (`CardDefinition { power: 3, ..creature("X", …, 2, 2) }`) and manufacture
    the drift the audit is looking for.
    """
    consts = consts or {}
    if card_def_name(body) is not None:
        return body
    m = re.search(r"\.\.(\w+)\(", body)
    fname = m.group(1) if m else None
    if fname is None:
        # A bare `helper("Name", …)` tail with no struct wrapper.
        m2 = re.search(r"^\s*(\w+)\(", body, re.M)
        fname = m2.group(1) if m2 else None
    if fname not in helpers and fname not in consts:
        return body
    args = split_args(call_args(body, fname))
    fields = []
    for field, lit in consts.get(fname, {}).items():
        if own_field(body, re.escape(field) + r":") is None:
            fields.append(f"{field}: vec![{lit}],")
    for field, idx in helpers.get(fname, {}).items():
        if idx >= len(args):
            continue
        if own_field(body, re.escape(field) + r":") is not None:
            continue
        fields.append(f"{field}: {args[idx].strip()},")
    if not fields:
        return body
    cd = body.find("CardDefinition {")
    if cd < 0:
        return "CardDefinition {" + "".join(fields) + "}" + body
    at = cd + len("CardDefinition {")
    return body[:at] + "".join(fields) + body[at:]

def audit():
    per_set = {}      # set -> dict(checked, cost[], pt[], type[], kw[])
    for src in sorted(SETS.rglob("*.rs")):
        s = set_of(src)
        d = per_set.setdefault(s, {"checked": 0, "cost": [], "pt": [], "type": [], "ct": [], "st": [], "kw": [], "abil": [], "timing": [], "tapsac": [], "loy": [], "tok": [], "trig": [], "scope": [], "filt": [], "num": []})
        text = src.read_text()
        helpers, hconsts = helper_table(text)
        vecfns = vec_fn_table(text)
        kwfns = kw_fn_table(text)
        predfns = pred_fn_table(text)
        for m in FUNC.finditer(text):
            nxt = FUNC.search(text, m.end()); body = text[m.end():nxt.start() if nxt else len(text)]
            # Stop at the next TOP-LEVEL `fn`, not only at the next `pub fn`:
            # a private helper defined between two card factories is otherwise
            # inside this card's body, and its own `CardDefinition { card_types:
            # vec![CardType::Land] }` answered for the card above it. That read
            # Dominaria's Judgment and Pay No Heed — both `instant(...)` one-liners
            # — as Lands (2026-08-30).
            cut = re.search(r"\n(?:pub(?:\([^)]*\))?\s+)?fn \w+", body)
            if cut:
                body = body[: cut.start()]
            raw_body = inline_helper_call(body, helpers, hconsts)
            body = strip_token_literals(body)
            body = inline_helper_call(body, helpers, hconsts)
            nm = card_def_name(body)
            if not nm: continue
            card = CACHE_LC.get(nm.group(1).lower())
            if not card: continue
            face = face_for(card, nm.group(1))
            d["checked"] += 1
            tag = (nm.group(1), src.name, m.group(1))
            # cost. A card with no `cost:` at all and a `..Default::default()`
            # tail is castable for free — the same "an omitted field is a
            # value" reading the P/T block below spells out. Lands and the
            # other genuinely costless faces answer `""` on both sides.
            cargs = toplevel_cost_args(body)
            if (cargs is None
                    and own_field(body, r"cost:") is None
                    and own_field(body, r"\.\.Default::default\(\)") is not None):
                cargs = ""
            if cargs is not None:
                syms = [sym(c) for c in re.split(r",(?![^()]*\))", cargs) if c.strip()]
                if all(syms):
                    got = "".join(syms)
                    refs = [card.get("mana_cost")] + [f.get("mana_cost") for f in card.get("card_faces", []) or []]
                    if face is not None and face.get("mana_cost") is not None:
                        refs = [face.get("mana_cost")]
                    refs = [r for r in refs if r]
                    if refs and all(norm(got) != norm(r) for r in refs):
                        d["cost"].append((tag, got, "|".join(refs)))
            # P/T. ⚠ **An omitted field is a value, not an absence.** Requiring
            # both `power:` and `toughness:` skipped every card that spells one
            # of them and lets `..Default::default()` supply 0 for the other —
            # and 0 is a printed characteristic like any number. Paradise Druid
            # shipped as a 0/2 (printed 2/1) with its doc comment agreeing, so
            # `audit_doc_drift` could not see it either. The `..Default::
            # default()` tail is the gate: without it the missing field could
            # come from a helper this file's table does not know, and a
            # defaulted 0 would be invented rather than read.
            pm = own_field(body, r"power:\s*(-?\d+)")
            tm = own_field(body, r"toughness:\s*(-?\d+)")
            defaulted = own_field(body, r"\.\.Default::default\(\)") is not None
            # ⚠ A Spacecraft's printed P/T is its station band's, and the card
            # is a 0/0 non-creature until it is stationed. Scryfall reports the
            # band number in `power`/`toughness`, so all 21 EOE Spacecraft read
            # as 0/0-vs-N/M defects without this gate.
            if own_field(body, r"station:\s*vec!\[") is not None:
                pm = tm = None
                defaulted = False
            got_p = pm.group(1) if pm else ("0" if defaulted else None)
            got_t = tm.group(1) if tm else ("0" if defaulted else None)
            ref_pt = face if face is not None and face.get("power") is not None else card
            numeric = lambda v: v is not None and re.fullmatch(r"-?\d+", str(v))
            if got_p and got_t and numeric(ref_pt.get("power")) and numeric(ref_pt.get("toughness")):
                if (got_p, got_t) != (str(ref_pt["power"]), str(ref_pt["toughness"])):
                    d["pt"].append((tag, f"{got_p}/{got_t}", f"{ref_pt['power']}/{ref_pt['toughness']}"))
            # creature subtypes
            ctm = next(own_helper_fields(body, r"creature_types:"), None)
            ctv = vec_after(body, ctm.start()) if ctm else None
            ref_ct = creature_subtypes_ref(card, face)
            if ctv is not None and ref_ct is not None:
                code_ct = set(re.findall(r"CreatureType::(\w+)", ctv))
                if code_ct and ref_ct and code_ct != ref_ct and not code_ct.issubset(ref_ct):
                    d["type"].append((tag, sorted(code_ct), sorted(ref_ct)))
            # card types and supertypes (CR 205.2a / 205.4)
            ref_tl = type_line_ref(card, face)
            if ref_tl is not None:
                ref_supers, ref_types = ref_tl
                ctypes = field_vec(body, "card_types", vecfns)
                if ctypes is not None:
                    code_types = set(CT_VARIANT.findall(ctypes))
                    # A command-zone object shares its name with a normal card
                    # (the Vanguard avatar "Maraxus of Keld" and the Legends
                    # creature), and the cache is keyed by name, so the
                    # reference is about the other one. Skip, don't flag.
                    if code_types & {"Vanguard", "Scheme", "Plane", "Phenomenon", "Conspiracy"} \
                            and not (code_types & ref_types):
                        code_types = set()
                    if code_types and code_types != ref_types:
                        d["ct"].append((tag, sorted(code_types), sorted(ref_types)))
                    stv = field_vec(body, "supertypes", vecfns) or ""
                    code_supers = set(ST_VARIANT.findall(stv))
                    if code_types and code_supers != ref_supers:
                        d["st"].append((tag, sorted(code_supers), sorted(ref_supers)))
            # activated-ability mana costs: the multiset of the card's
            # mana-bearing activation costs against the oracle's. Only
            # compared when the code's literal count matches the oracle's
            # activation-line count, so a helper-built ability (a mana
            # ability, an `equip`) does not read as a missing one.
            abil = ability_mana_costs(body)
            ref_abil = ref_ability_mana_costs(card, face)
            if abil is not None and ref_abil is not None and len(abil) == len(ref_abil):
                got_ab = sorted(a for a in abil if a)
                ref_ab = sorted(a for a in ref_abil if a)
                if got_ab != ref_ab:
                    d["abil"].append((tag, ["".join(a) or "{0}" for a in abil], ["".join(a) or "{0}" for a in ref_abil]))
            # activation timing: "Activate only as a sorcery" / "only once each
            # turn" against the literal's own flags, as multisets, same gate.
            tim = ability_timing(body)
            ref_tim = ref_ability_timing(card, face)
            if tim is not None and ref_tim is not None and len(tim) == len(ref_tim):
                if timing_mismatch(tim, ref_tim):
                    d["timing"].append((tag, [sorted(f) or ["-"] for f in tim], [sorted(f) or ["-"] for f in ref_tim]))
            # {T} / "Sacrifice this" halves of the cost against tap_cost /
            # sac_cost, as multisets, same gate.
            ts = ability_tap_sac(body)
            ref_ts = ref_ability_tap_sac(card, face)
            if ts is not None and ref_ts is not None and len(ts) == len(ref_ts):
                if sorted(sorted(f) for f in ts) != sorted(sorted(f) for f in ref_ts):
                    d["tapsac"].append((tag, [sorted(f) or ["-"] for f in ts], [sorted(f) or ["-"] for f in ref_ts]))
            # loyalty: the signed costs as a multiset plus the base loyalty.
            loy = loyalty_costs(body)
            ref_loy = ref_loyalty_costs(card, face)
            if loy is not None and ref_loy is not None and ref_loy[0]:
                (got_l, got_b), (ref_l, ref_b) = loy, ref_loy
                if (len(got_l) == len(ref_l) and sorted(got_l) != sorted(ref_l)) or (
                    got_b is not None and ref_b is not None and str(ref_b).isdigit() and got_b != ref_b
                ):
                    d["loy"].append((tag, f"{got_l} base {got_b}", f"{ref_l} base {ref_b}"))
            # token P/T: every TokenDefinition literal's stats against the
            # oracle's "N/N … token" mentions, as multisets, when the counts
            # match (a token helper call is invisible to both sides).
            tok = token_stats(raw_body)
            ref_tok = ref_token_stats(card, face)
            if tok and ref_tok and len(tok) == len(ref_tok) and sorted(tok) != sorted(ref_tok):
                d["tok"].append((tag, sorted(tok), sorted(ref_tok)))
            # trigger events: each literal's EventKind class against the
            # oracle's When / Whenever / At-the-beginning lines, matched
            # one-to-one, same count gate.
            trg = trigger_kinds(body)
            ref_trg = ref_trigger_kinds(card, face)
            if trg is not None and ref_trg is not None and len(trg) == len(ref_trg):
                if trigger_mismatch(trg, ref_trg):
                    d["trig"].append((tag, trg, ["|".join(sorted(f)) for f in ref_trg]))
            # trigger scopes: each literal's EventScope class against the
            # clause's subject, one-to-one, same count gate.
            sc = trigger_scopes(body)
            ref_sc = ref_trigger_scopes(card, face)
            if sc is not None and ref_sc is not None and len(sc) == len(ref_sc):
                if trigger_mismatch(sc, ref_sc):
                    d["scope"].append((tag, sc, ["|".join(sorted(f)) for f in ref_sc]))
            # trigger filters: type words, one-to-one, same count gate.
            fw = trigger_filter_words(body, predfns)
            ref_fw = ref_trigger_filter_words(card, face)
            if fw is not None and ref_fw is not None and len(fw) == len(ref_fw):
                if filter_mismatch(fw, ref_fw):
                    d["filt"].append((tag, [sorted(c[1]) if c is not None else "-" for c in fw], [sorted(f) if f is not None else "-" for f in ref_fw]))
            # amounts: the oracle numbers of each activation / trigger line
            # (and a spell's text) must appear in the literal assigned to
            # it, one-to-one, same count gate. Statics are not read.
            ref_lines = _oracle_ability_lines((face or card).get("oracle_text") or "")
            cname = (face or card).get("name")
            for kind in ("act", "trig"):
                nums = ability_numbers(body, kind)
                ref_nums = [oracle_numbers(t, cname) for k, t in ref_lines if k == kind]
                if nums is not None and len(nums) == len(ref_nums) and any(ref_nums) \
                        and numbers_mismatch(nums, ref_nums):
                    d["num"].append((tag, [sorted(n) for n in nums], [sorted(n) for n in ref_nums]))
            tl = (face or card).get("type_line") or ""
            if re.search(r"\b(?:Instant|Sorcery)\b", tl):
                sn = spell_numbers(body)
                ref_sn = set().union(*[oracle_numbers(t, cname) for k, t in ref_lines if k == "other"]) if ref_lines else set()
                if sn is not None and ref_sn and not ref_sn <= sn:
                    d["num"].append((tag, sorted(sn), sorted(ref_sn)))
            # keywords (top-level only)
            kwv = toplevel_keywords(body)
            if kwv is not None:
                code_kw = code_keywords(kwv, kwfns)
                rk = ref_keywords(card, face)
                if rk is not None and code_kw != rk:
                    d["kw"].append((tag, sorted(code_kw), sorted(rk)))
    return per_set

def main():
    per_set = audit()
    detail = sys.argv[1] if len(sys.argv) > 1 else None
    
    if detail:
        d = per_set.get(detail)
        if not d: sys.exit(f"no such set '{detail}' (have: {', '.join(sorted(per_set))})")
        for dim in ("cost", "pt", "type", "ct", "st", "kw", "abil", "timing", "tapsac", "loy", "tok", "trig", "scope", "filt", "num"):
            print(f"\n=== {dim.upper()} drift in {detail} ({len(d[dim])}) ===")
            for tag, got, ref in d[dim]:
                print(f"  {tag[0]}  ({tag[1]}::{tag[2]})\n    code={got}  scryfall={ref}")
    else:
        dims = ("cost", "pt", "type", "ct", "st", "kw", "abil", "timing", "tapsac", "loy", "tok", "trig", "scope", "filt", "num")
        print(f"{'set':<12}{'checked':>8}{'cost':>6}{'P/T':>6}{'sub':>6}{'type':>6}{'super':>6}{'kw':>6}{'abil':>6}{'tim':>6}{'T/sac':>6}{'loy':>6}{'tok':>6}{'trig':>6}{'scope':>6}{'filt':>6}{'num':>6}")
        print("-" * 110)
        tot = {"checked": 0, **{k: 0 for k in dims}}
        for s in sorted(per_set, key=lambda s: -sum(len(per_set[s][k]) for k in dims)):
            d = per_set[s]
            if not d["checked"]: continue
            for k in tot: tot[k] += d["checked"] if k == "checked" else len(d[k])
            print(f"{s:<12}{d['checked']:>8}" + "".join(f"{len(d[k]):>6}" for k in dims))
        print("-" * 110)
        print(f"{'TOTAL':<12}{tot['checked']:>8}" + "".join(f"{tot[k]:>6}" for k in dims))
        print("\nDetail for a set:  python3 scripts/audit_catalog_stats.py <set>")


if __name__ == "__main__":
    main()
