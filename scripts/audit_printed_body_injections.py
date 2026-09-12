#!/usr/bin/env python3
"""Injection battery for `audit_printed_body.py`'s readers.

    python3 scripts/audit_printed_body_injections.py    # 28/28 as expected

**A GATE THAT CANNOT FAIL IS WORSE THAN NO GATE**, and this session proved the
point three times: `fn legend`'s injection passed because the resolver took
whichever definition came first; `fn aura`'s passed because "no literal at all"
was read as unreadable, which made the base recursion dead for every
pure-helper factory; and `fn sliver`'s passed because a cost chain the reader
gave up on skipped the card out of the TYPE and SUBTYPE columns too. Each was
invisible in a report that said 0.

So the injections are runnable rather than a list in a docstring. Each case
breaks ONE idiom in the catalog, runs the audit, and restores the file. A case
marked `silent` is a NEGATIVE test: the reader is supposed to be looking
somewhere else, and a row there would be the bug — Bronzehide Lion's BACK face
(the reader takes the returned literal) and `fn ally`'s `mut types`, which is
pushed to before the call it feeds, so binding it to the call site would report
twenty-odd correct Allies.

Run it after touching any reader in `audit_printed_body.py`. It edits catalog
files in place and restores them; `git status` must be clean afterwards.
"""
import pathlib, re, signal, subprocess, sys
ROOT = pathlib.Path("/home/user/Crabomination")
CASES = [
 ("fires", "literal field (Archive Trap)", "sets/mod_set/instants.rs",
  "spell_subtypes: vec![crate::card::SpellSubtype::Trap],", "spell_subtypes: vec![],"),
 ("fires", "base-struct helper (modern fn sliver)", "sets/decks/modern.rs",
  "fn sliver(name: &'static str, c: ManaCost, power: i32, toughness: i32) -> CardDefinition {\n"
  "    CardDefinition {\n        name,\n        cost: c,\n"
  "        card_types: vec![CardType::Creature],\n        subtypes: Subtypes {\n"
  "            creature_types: vec![CreatureType::Sliver],",
  "fn sliver(name: &'static str, c: ManaCost, power: i32, toughness: i32) -> CardDefinition {\n"
  "    CardDefinition {\n        name,\n        cost: c,\n"
  "        card_types: vec![CardType::Creature],\n        subtypes: Subtypes {\n"
  "            creature_types: vec![],"),
 ("fires", "bound parameter (bng3 fn creature)", "sets/bng3.rs",
  "            creature_types: ct,", "            creature_types: vec![],"),
 ("fires", "two-link chain (lgn fn sliver -> fn creature)", "sets/lgn.rs",
  "    creature(name, c, vec![CreatureType::Sliver], p, t)",
  "    creature(name, c, vec![], p, t)"),
 ("fires", "pure-helper base (jou3 fn aura)", "sets/jou3.rs",
  "            enchantment_subtypes: vec![EnchantmentSubtype::Aura],\n", ""),
 ("fires", "inner-literal base (war ..CardDefinition { })", "sets/war.rs",
  "            subtypes: creatures(vec![CreatureType::Golem]),\n            power: 2,\n            toughness: 1,",
  "            subtypes: creatures(vec![]),\n            power: 2,\n            toughness: 1,"),
 ("fires", "Subtypes-returning helper (war fn creatures)", "sets/war.rs",
  "fn creatures(t: Vec<CreatureType>) -> Subtypes {\n    Subtypes {\n        creature_types: t,",
  "fn creatures(t: Vec<CreatureType>) -> Subtypes {\n    Subtypes {\n        creature_types: vec![],"),
 ("fires", "a bound CARD TYPE parameter (jou2 fn spell's `kind`)", "sets/jou2.rs",
  "fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {\n"
  "    CardDefinition {\n        name,\n        cost: mana,\n        card_types: vec![kind],",
  "fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {\n"
  "    CardDefinition {\n        name,\n        cost: mana,\n        card_types: vec![CardType::Land],"),
 ("fires", "a helper whose call is its TAIL, after a statement (bfz fn ally -> fn creature)",
  "sets/bfz/creatures.rs",
  "        card_types: vec![CardType::Creature],\n"
  "        subtypes: Subtypes { creature_types: types, ..Default::default() },",
  "        card_types: vec![CardType::Artifact],\n"
  "        subtypes: Subtypes { creature_types: types, ..Default::default() },"),
 ("fires", "a P/T through a bound parameter and a shorthand field (bfz fn creature)",
  "sets/bfz/creatures.rs",
  "        subtypes: Subtypes { creature_types: types, ..Default::default() },\n"
  "        power: p,\n        toughness: t,",
  "        subtypes: Subtypes { creature_types: types, ..Default::default() },\n"
  "        power: 0,\n        toughness: t,"),
 ("fires", "a station band's own P/T (eoe Rescue Skiff, CR 721)", "sets/eoe.rs",
  "            min: 10,\n            keywords: vec![Keyword::Flying],\n            pt: Some((5, 6)),",
  "            min: 10,\n            keywords: vec![Keyword::Flying],\n            pt: Some((5, 7)),"),
 ("fires", "a keyword removed from a literal (mod_set Glorybringer)",
  "sets/mod_set/creatures.rs",
  "        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::Exert],",
  "        keywords: vec![Keyword::Haste, Keyword::Exert],"),
 # The widened vocabulary: Prowess/Ward/Hexproof/Protection are only checked
 # since `audit_catalog_stats.py` proved the evergreen set was too narrow to
 # see the prowess class. Abbot of Keral Keep prints prowess on its own line.
 ("fires", "a WIDENED-vocabulary keyword removed (modern Abbot of Keral Keep)",
  "sets/decks/modern.rs",
  "            creature_types: vec![CreatureType::Human, CreatureType::Monk],\n"
  "            ..Default::default()\n        },\n        power: 2,\n        toughness: 1,\n"
  "        keywords: vec![Keyword::Prowess],",
  "            creature_types: vec![CreatureType::Human, CreatureType::Monk],\n"
  "            ..Default::default()\n        },\n        power: 2,\n        toughness: 1,\n"
  "        keywords: vec![],"),
 ("fires", "a keyword ADDED that the card does not print (ogw fn drone)",
  "sets/ogw/creatures.rs",
  "            creature_types: vec![CreatureType::Eldrazi, CreatureType::Drone],\n"
  "            ..Default::default()\n        },\n        power: p,\n        toughness: t,\n"
  "        keywords: vec![Keyword::Devoid],",
  "            creature_types: vec![CreatureType::Eldrazi, CreatureType::Drone],\n"
  "            ..Default::default()\n        },\n        power: p,\n        toughness: t,\n"
  "        keywords: vec![Keyword::Devoid, Keyword::Flying],"),
 # A TUPLE PARAMETER. `fn bestow_creature(name, mana, bestow_cost, pt: (i32,
 # i32), ct, kw, bonus)` closes `[^)]*` at the tuple, so the reader that
 # matched the parameter list that way bound `ct` to nothing and skipped all
 # eight bestow creatures out of the subtype column. Breaking ONE call site's
 # `ct` argument is silent under that reader and a row under a brace-matched
 # one.
 ("fires", "an argument after a tuple parameter (bng3 Ghostblade Eidolon)",
  "sets/bng3.rs",
  '        "Ghostblade Eidolon",\n        cost(&[generic(2), w()]),\n'
  "        cost(&[generic(5), w()]),\n        (1, 1),\n"
  "        vec![CreatureType::Spirit],",
  '        "Ghostblade Eidolon",\n        cost(&[generic(2), w()]),\n'
  "        cost(&[generic(5), w()]),\n        (1, 1),\n"
  "        vec![],"),
 # A WRAPPER'S BASE IS A PARAMETER, and the card is the ARGUMENT bound to it:
 # `landfall_self_pump(creature("Scythe Leopard", ..), (1, 1), vec![])`. The
 # chain every other reader follows is one level further out.
 ("fires", "a base bound to a parameter (bfz landfall_self_pump)",
  "sets/bfz/creatures.rs",
  '        creature("Scythe Leopard", cost(&[g()]), vec![CreatureType::Cat], 1, 1),',
  '        creature("Scythe Leopard", cost(&[g()]), vec![], 1, 1),'),
 # A helper that builds its cost from a SYMBOL SLICE — `cost: cost(cost_syms)`
 # over a `&[ManaSymbol]` parameter, the alpha/beta creature helpers.
 ("fires", "a cost through a symbol slice (lea fn body)", "sets/lea/creatures.rs",
  '        "Youthful Knight",\n        &[generic(1), w()],',
  '        "Youthful Knight",\n        &[generic(2), w()],'),
 # A CARD TYPE chosen by a bound flag: `card_types: vec![if sorcery {
 # CardType::Sorcery } else { CardType::Instant }]`, two braces deep, which is
 # deeper than the flattened body keeps.
 ("fires", "a card type behind a bound flag (recent314 Dismantle)",
  "sets/decks/recent314.rs",
  '        "Dismantle",\n        cost(&[generic(2), r()]),\n        true,',
  '        "Dismantle",\n        cost(&[generic(2), r()]),\n        false,'),
 # A binding that is ONE ELEMENT of the vec, not the whole field:
 # `planeswalker_subtypes: vec![sub]`, every planeswalker in War of the Spark.
 ("fires", "a subtype bound per element (war fn walker)", "sets/war.rs",
  '            "Tibalt, Rakish Instigator",\n            cost(&[generic(2), r()]),\n'
  "            PlaneswalkerSubtype::Tibalt,",
  '            "Tibalt, Rakish Instigator",\n            cost(&[generic(2), r()]),\n'
  "            PlaneswalkerSubtype::Jace,"),
 # A TUPLE parameter read FIELD BY FIELD: `fn bestow_creature(.., pt: (i32,
 # i32), ..) { ..creature(name, mana, pt.0, pt.1, ..) }`.
 ("fires", "a P/T through a tuple field (bng3 Ghostblade Eidolon)", "sets/bng3.rs",
  '        "Ghostblade Eidolon",\n        cost(&[generic(2), w()]),\n'
  "        cost(&[generic(5), w()]),\n        (1, 1),\n"
  "        vec![CreatureType::Spirit],\n        vec![Keyword::DoubleStrike],",
  '        "Ghostblade Eidolon",\n        cost(&[generic(2), w()]),\n'
  "        cost(&[generic(5), w()]),\n        (2, 1),\n"
  "        vec![CreatureType::Spirit],\n        vec![Keyword::DoubleStrike],"),
 # A SHORTHAND `keywords,` IS THE PARAMETER. `fn creature(.., keywords: Vec<
 # Keyword>)` is how most per-set files spell a creature, and calling the
 # shorthand unreadable skipped 439 factories out of the column that combat
 # reads every turn.
 ("fires", "a keyword through a shorthand field (recent316 Goblin Striker)",
  "sets/decks/recent316.rs",
  '        "Goblin Striker",\n        cost(&[generic(1), r()]),\n        1,\n        1,\n'
  "        vec![CreatureType::Goblin, CreatureType::Berserker],\n"
  "        vec![Keyword::FirstStrike, Keyword::Haste],",
  '        "Goblin Striker",\n        cost(&[generic(1), r()]),\n        1,\n        1,\n'
  "        vec![CreatureType::Goblin, CreatureType::Berserker],\n"
  "        vec![Keyword::Haste],"),
 # Negative: the shorthand reader is DEPTH 0. `simple_aura`'s keywords go to a
 # nested `EquipBonus { power, toughness, keywords, .. }` — the aura's BONUS,
 # not the aura's own field — so a keyword added there must stay invisible. A
 # depth-blind scan read sixteen auras and land animations as printing what
 # they grant.
 ("silent", "a nested EquipBonus shorthand is not the card's field (modern Flight)",
  "sets/decks/modern.rs",
  '    simple_aura("Flight", cost(&[u()]), 0, 0, vec![Keyword::Flying])',
  '    simple_aura("Flight", cost(&[u()]), 0, 0, vec![Keyword::Flying, Keyword::Trample])'),
 # THE PRINTED COLOUR is derived — cost pips plus the colour indicator, empty
 # under Devoid. A card whose cost carries no pip is colorless without the
 # indicator, which is what three shipped cards were.
 ("fires", "a missing colour indicator (sok Evermind)", "sets/sok.rs",
  "        color_indicator: vec![Color::Blue],\n", ""),
 # CR 306.5b — the STARTING LOYALTY. A walker at the wrong number is wrong in
 # every game it is in, and 0 kills it on entry (CR 704.5i).
 ("fires", "a wrong starting loyalty (bng3 Kiora, the Crashing Wave)", "sets/bng3.rs",
  "        base_loyalty: 2,", "        base_loyalty: 3,"),
 # The flag on a card that DOES print a cost makes it uncastable for ever —
 # the worse half of the `no_mana_cost` check, and the one that had no test.
 ("fires", "`no_mana_cost` on a card that prints one (recent302 Transguild Courier)",
  "sets/decks/recent302.rs",
  '        name: "Transguild Courier",\n        cost: cost(&[generic(4)]),',
  '        name: "Transguild Courier",\n        cost: cost(&[generic(4)]),\n'
  "        no_mana_cost: true,"),
 # A NAME the oracle does not know is audited by NOBODY — every other column
 # keys on it, and `audit_printed_body` drops the card as `nocache`. Two
 # shipped cards were misspelled that way and both had a wrong cost behind the
 # misspelling; `audit_card_names.py` is the column that sees them.
 ("fires", "a misspelled card name (recent71 Sabretooth Tiger)",
  "sets/decks/recent71.rs",
  '        name: "Sabretooth Tiger",', '        name: "Sabertooth Tiger",'),
 # Negative: the oracle's `keywords` array counts keywords the card GRANTS, so
 # the missing direction reads the printed keyword LINES instead. Steel Seraph
 # grants "flying, vigilance, or lifelink" and has only flying; dropping its
 # trigger must not make vigilance go missing.
 ("silent", "a granted keyword is not a missing one (bro Steel Seraph)",
  "sets/bro.rs",
  "                keyword: Keyword::Flying,\n                duration: Duration::EndOfTurn,",
  "                keyword: Keyword::Menace,\n                duration: Duration::EndOfTurn,"),
 # Negative: the reader must be looking at the RETURNED literal (the front
 # face), so emptying the BACK face's subtype must change nothing.
 ("silent", "the returned literal, not the first (thb Bronzehide Lion back face)",
  "sets/thb.rs",
  "            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],",
  "            enchantment_subtypes: vec![],"),
 # Negative: a `mut` parameter is mutated before the call it feeds, so binding
 # it to the caller's argument would report 20-odd correct Allies.
 ("silent", "a mut parameter is not its caller's argument (zen3 fn ally)",
  "sets/zen3.rs", "    types.push(CreatureType::Ally);\n", ""),
]
# ⚠ EVERY ROW KIND, or a column's cases cannot fire. The colour column was
# added with its row kind missing here, and its injection read "silent" — the
# battery's own version of the bug it exists to catch.
ROW = re.compile(r"^  (?:sub|kw|types|super|cost|p/t|color|loyalty|no-cost) ", re.M)
NAME_ROW = re.compile(r"^  (?:spelling|unknown) ", re.M)


def run():
    """Rows from BOTH oracle-backed catalog audits.

    `audit_card_names.py` shares this file's name reader, and its own rows are
    the population `audit_printed_body` drops as `nocache` — so an injection
    that breaks a NAME is silent in one and loud in the other, and the battery
    has to see both or half of what it covers cannot fail.
    """
    out = subprocess.run([sys.executable, "scripts/audit_printed_body.py", "--rows", "0"],
                         cwd=ROOT, capture_output=True, text=True).stdout
    names = subprocess.run([sys.executable, "scripts/audit_card_names.py"],
                           cwd=ROOT, capture_output=True, text=True).stdout
    rows = len(ROW.findall(out)) + len(NAME_ROW.findall(names))
    head = next((l for l in out.split("\n") if l.startswith("# compared")), "")
    return rows, head


# ⚠ THE BATTERY EDITS THE CATALOG IN PLACE. A run that dies between the write
# and the restore leaves a broken card behind, and the next audit reads it as a
# finding — so refuse to start on a dirty catalog, and restore on a signal.
dirty = subprocess.run(["git", "status", "--porcelain", "crabomination_catalog"],
                       cwd=ROOT, capture_output=True, text=True).stdout.strip()
if dirty:
    print("catalog is dirty — commit or stash it first, or a killed run left a "
          "patch behind:\n" + dirty)
    sys.exit(2)
PENDING = {}


def restore(*_):
    for p, orig in PENDING.items():
        p.write_text(orig)
    PENDING.clear()
    sys.exit(130)


for sig in (signal.SIGINT, signal.SIGTERM):
    signal.signal(sig, restore)
base, line = run()
print(f"BASELINE {base} rows\n  {line}\n")
bad = 0
for expect, label, rel, old, new in CASES:
    p = ROOT / "crabomination_catalog/src" / rel
    orig = p.read_text()
    n = orig.count(old)
    if n != 1:
        print(f"BROKEN CASE     pattern occurs {n}x   {label}"); bad += 1; continue
    PENDING[p] = orig
    try:
        p.write_text(orig.replace(old, new, 1))
        rows, _ = run()
    finally:
        p.write_text(orig)
        PENDING.pop(p, None)
    delta = rows - base
    ok = (delta > 0) if expect == "fires" else (delta == 0)
    bad += not ok
    print(f"{'ok' if ok else 'FAILED':8} expect {expect:6} got {delta:+5d} rows   {label}")
print(f"\n{len(CASES) - bad}/{len(CASES)} as expected")
sys.exit(1 if bad else 0)
