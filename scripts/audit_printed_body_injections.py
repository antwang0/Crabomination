#!/usr/bin/env python3
"""Injection battery for `audit_printed_body.py`'s readers.

    python3 scripts/audit_printed_body_injections.py           # 45/45 as expected
    python3 scripts/audit_printed_body_injections.py -j 4      # fewer workers

**A GATE THAT CANNOT FAIL IS WORSE THAN NO GATE**, and this session proved the
point three times: `fn legend`'s injection passed because the resolver took
whichever definition came first; `fn aura`'s passed because "no literal at all"
was read as unreadable, which made the base recursion dead for every
pure-helper factory; and `fn sliver`'s passed because a cost chain the reader
gave up on skipped the card out of the TYPE and SUBTYPE columns too. Each was
invisible in a report that said 0.

So the injections are runnable rather than a list in a docstring. Each case
breaks ONE idiom in a COPY of the catalog, runs both oracle-backed audits
against that copy, and compares the row count to a clean reading. A case marked
`silent` is a NEGATIVE test: the reader is supposed to be looking somewhere
else, and a row there would be the bug — Bronzehide Lion's BACK face, where the
reader takes the returned literal rather than the first.

⚠ **IT USED TO PATCH THE REAL CATALOG IN PLACE**, which cost two things and
neither was obvious. It could not run beside anything else: a concurrent
`audit_doc_drift` read the Wall of Omens case mid-injection and reported the
card at 0/5, a "BODY WRONG" row indistinguishable from a shipped defect. And it
could not run its cases in parallel, because every one of them wrote the same
tree — 40 cases x two audits, serial, is over an hour for a gate that has to run
after every reader change. A per-worker copy under `CRAB_CATALOG_DIR` (24 MB
each) fixes both, and the real catalog is never written at all, so a killed run
leaves nothing behind.

Run it after touching any reader in `audit_printed_body.py`.
"""
import argparse, concurrent.futures, os, pathlib, queue, re, shutil, subprocess, sys, tempfile
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
 # The ADVENTURE half (CR 715) and the BACK FACE (CR 712) — two faces every
 # other column reads past, because the factory's `cost:`, `card_types:` and
 # P/T are the FRONT's. Five shipped defects between them on the first read.
 ("fires", "an adventure half's cost (modern Rider in Need)", "sets/decks/modern.rs",
  '            name: "Rider in Need",\n            cost: cost(&[generic(2), w()]),',
  '            name: "Rider in Need",\n            cost: cost(&[generic(3), w()]),'),
 ("fires", "an adventure half's card type (modern Shield's Might)", "sets/decks/modern.rs",
  "            name: \"Shield's Might\",\n            cost: cost(&[generic(1), g()]),\n"
  "            card_types: vec![CardType::Instant],",
  "            name: \"Shield's Might\",\n            cost: cost(&[generic(1), g()]),\n"
  "            card_types: vec![CardType::Sorcery],"),
 # CR 301.7: a Vehicle is not a creature until it is crewed.
 ("fires", "a back face's card types (mom Aetherwing, Golden-Scale Flagship)",
  "sets/decks/mom.rs",
  "        card_types: vec![CardType::Artifact],\n"
  "        supertypes: vec![Supertype::Legendary],\n        subtypes: Subtypes {\n"
  "            artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle],",
  "        card_types: vec![CardType::Artifact, CardType::Creature],\n"
  "        supertypes: vec![Supertype::Legendary],\n        subtypes: Subtypes {\n"
  "            artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle],"),
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
 ("fires", "a misspelled card name (recent77 Sabretooth Tiger)",
  "sets/decks/recent77.rs",
  '        "Sabretooth Tiger",', '        "Sabertooth Tiger",'),
 # A MULTI-FACE card is audited on the face its factory NAMES now, so the front
 # of a transform card has a cost column for the first time.
 ("fires", "a front face's cost (fin2 Sidequest: Catch a Fish)", "sets/fin2.rs",
  '        "Sidequest: Catch a Fish",\n        cost(&[generic(2), w()]),',
  '        "Sidequest: Catch a Fish",\n        cost(&[generic(3), w()]),'),
 # Negative: the BACK face's literal is passed as an argument to the helper
 # that builds the front, at brace depth 0 — so a depth-only reader took it as
 # the card and gave the Campsite's Land type line over the enchantment's
 # `{2}{W}`. Breaking the back face must stay invisible.
 ("silent", "an argument literal is not the card (fin2 Cooking Campsite)",
  "sets/fin2.rs",
  '            name: "Cooking Campsite",\n            card_types: vec![CardType::Land],',
  '            name: "Cooking Campsite",\n            card_types: vec![CardType::Creature],'),
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
 # ONE HALF OF THE P/T, the other half from the `..base`. `power: 1,
 # ..phantom(name, .., 2, ..)` used to read as UNREADABLE — a literal that
 # declared one half skipped the card — which took 70 creatures out of the
 # column, every 0-power card among them (`power:` is omitted because
 # `Default` is already 0). Both halves now walk the chain on their own.
 ("fires", "a P/T half taken from the base (jud Phantom Tiger)", "sets/jud.rs",
  '        power: 1,\n        ..phantom(\n            "Phantom Tiger",',
  '        power: 2,\n        ..phantom(\n            "Phantom Tiger",'),
 ("fires", "a P/T half taken from Default (modern Wall of Omens)",
  "sets/decks/modern.rs",
  '        name: "Wall of Omens",\n        cost: cost(&[generic(1), w()]),\n'
  "        card_types: vec![CardType::Creature],\n        subtypes: Subtypes {\n"
  "            creature_types: vec![CreatureType::Wall],\n"
  "            ..Default::default()\n        },\n        toughness: 4,",
  '        name: "Wall of Omens",\n        cost: cost(&[generic(1), w()]),\n'
  "        card_types: vec![CardType::Creature],\n        subtypes: Subtypes {\n"
  "            creature_types: vec![CreatureType::Wall],\n"
  "            ..Default::default()\n        },\n        toughness: 5,"),
 # AN ARGUMENT AFTER A `vec![..]` IN A MULTI-LINE `..base(..)` CALL. The
 # flattened body split any line with two `:` in it on top-level commas and
 # dropped the separators, so `vec![CreatureType::Phyrexian, ..],` fused with
 # the `3,` below it and `creature`'s `p`/`t` bound one argument short.
 ("fires", "an argument after a vec! in a base call (usg2 fn paladin)",
  "sets/usg2.rs",
  "            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Knight],\n"
  "            3,\n            3,",
  "            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Knight],\n"
  "            3,\n            4,"),
 # THE SIX SHAPES THAT REACH THEIR BASE THROUGH SOMETHING OTHER THAN
 # `..helper(`. Each was `notyped` — dropped before the FIRST column, so the
 # card was audited by nobody — and each of these six injections is silent
 # under the reader that skipped it.
 ("fires", "a tail after a block statement (jou2 Aerial Formation)", "sets/jou2.rs",
  '        "Aerial Formation",\n        cost(&[u()]),\n        CardType::Instant,',
  '        "Aerial Formation",\n        cost(&[u()]),\n        CardType::Sorcery,'),
 ("fires", "a let-bound base (lands Irrigated Farmland)", "sets/decks/lands.rs",
  '        "Irrigated Farmland",\n        LandType::Plains,',
  '        "Irrigated Farmland",\n        LandType::Swamp,'),
 ("fires", "an if/else in a let (jud2 Burning Wish)", "sets/jud2.rs",
  'wish("Burning Wish", cost(&[generic(1), r()]), R::HasCardType(CardType::Sorcery), false)',
  'wish("Burning Wish", cost(&[generic(1), r()]), R::HasCardType(CardType::Sorcery), true)'),
 ("fires", "a tail that is the helper's own parameter (tmp Anoint)", "sets/tmp/spells.rs",
  '        instant(\n            "Anoint",', '        sorcery(\n            "Anoint",'),
 ("fires", "a conditional base in a literal (tor2 Restless Dreams)", "sets/tor2.rs",
  '        "Restless Dreams",\n        cost(&[b()]),\n        true,',
  '        "Restless Dreams",\n        cost(&[b()]),\n        false,'),
 ("fires", "a module-qualified base (zen3 Sejiri Refuge)", "sets/zen3.rs",
  '        ..super::wwk::tapped_etb_land(\n            "Sejiri Refuge",',
  '        keywords: vec![Keyword::Flying],\n'
  '        ..super::wwk::tapped_etb_land(\n            "Sejiri Refuge",'),
 # ⚠ THIS CASE FLIPPED, and the flip is the point. It was `silent`: a `mut`
 # parameter is mutated before the call it feeds, so binding it straight back
 # to the caller's argument would have reported 20-odd correct Allies, and the
 # reader bound it to `None` instead. `mut_param_arg` READS the mutation now, so
 # deleting the push is a defect the column can see — the five zen3 Allies
 # really do lose their Ally type with that line gone.
 ("fires", "a mut parameter's own push (zen3 fn ally)",
  "sets/zen3.rs", "    types.push(CreatureType::Ally);\n", ""),
 # A shorthand field inside a `Subtypes`-returning helper IS the parameter.
 ("fires", "a shorthand parameter in a Subtypes helper (bro fn construct)",
  "sets/bro.rs",
  "fn construct(creature_types: Vec<CreatureType>) -> Subtypes {\n"
  "    Subtypes {\n        creature_types,",
  "fn construct(creature_types: Vec<CreatureType>) -> Subtypes {\n"
  "    Subtypes {\n        creature_types: vec![],"),
 # The keyword column on a FACE: the whole-card `keywords` array survives the
 # face merge as `_card_keywords`, and the MISSING side is intersected with this
 # face's own oracle text, so a back-face keyword cannot be reported on the front.
 #
 # ⚠ IT TAKES THE TOKEN'S VIGILANCE TOO, IN THE SAME CASE, and that is not
 # padding. The MISSING side is filtered by `f"Keyword::{k}" not in raw` — "a
 # keyword the factory mentions ANYWHERE is not reported missing", because a
 # card can grant itself one through a static. Rider in Need's Knight token
 # mentions `Keyword::Vigilance` in the same `pub fn` block, so removing only
 # the card's line leaves the guard suppressing the row and the injection reads
 # SILENT against a reader that is working. It did: 43/44 on the first full run
 # after the token was fixed, on the case that had passed alone the hour before.
 # ⚠ And its pattern went STALE the moment the adventure column fixed Rider in
 # Need's cost ({1}{W} -> {2}{W}): the anchor still said `generic(1)` and the
 # battery reported BROKEN CASE (pattern occurs 0x) rather than a silent pass —
 # which is the only reason it was caught. An injection anchored on a value is
 # anchored on something a later fix can move.
 ("fires", "a keyword on an adventure face (modern Lonesome Unicorn)",
  "sets/decks/modern.rs",
  "        toughness: 3,\n        keywords: vec![Keyword::Vigilance],\n"
  "        adventure: Some(Box::new(Adventure {\n"
  '            name: "Rider in Need",\n'
  "            cost: cost(&[generic(2), w()]),\n"
  "            card_types: vec![CardType::Sorcery],\n"
  "            effect: Effect::CreateToken {\n"
  "                who: PlayerRef::You,\n"
  "                count: Value::Const(1),\n"
  "                definition: Box::new(TokenDefinition {\n"
  '                    name: "Knight".into(),\n'
  "                    power: 2,\n                    toughness: 2,\n"
  "                    card_types: vec![CardType::Creature],\n"
  "                    colors: vec![Color::White],\n"
  "                    subtypes: Subtypes {\n"
  "                        creature_types: vec![CreatureType::Knight],\n"
  "                        ..Default::default()\n                    },\n"
  "                    keywords: vec![Keyword::Vigilance],",
  "        toughness: 3,\n"
  "        adventure: Some(Box::new(Adventure {\n"
  '            name: "Rider in Need",\n'
  "            cost: cost(&[generic(2), w()]),\n"
  "            card_types: vec![CardType::Sorcery],\n"
  "            effect: Effect::CreateToken {\n"
  "                who: PlayerRef::You,\n"
  "                count: Value::Const(1),\n"
  "                definition: Box::new(TokenDefinition {\n"
  '                    name: "Knight".into(),\n'
  "                    power: 2,\n                    toughness: 2,\n"
  "                    card_types: vec![CardType::Creature],\n"
  "                    colors: vec![Color::White],\n"
  "                    subtypes: Subtypes {\n"
  "                        creature_types: vec![CreatureType::Knight],\n"
  "                        ..Default::default()\n                    },"),
 # ⚠ NEGATIVE, and it is the one that makes the case above safe: a nested
 # `TokenDefinition`'s keyword must stay invisible to the CARD-level column.
 ("silent", "a token's keyword is not the card's (modern Rider in Need's Knight)",
  "sets/decks/modern.rs",
  "                        creature_types: vec![CreatureType::Knight],\n"
  "                        ..Default::default()\n                    },\n"
  "                    keywords: vec![Keyword::Vigilance],\n",
  "                        creature_types: vec![CreatureType::Knight],\n"
  "                        ..Default::default()\n                    },\n"),
 # ⚠ THE COST OF A HELPER WHOSE CALL IS ITS TAIL. `fn ally(.., mut types, ..)
 # { types.push(CreatureType::Ally); creature(name, c, types, ..) }` opens with
 # a STATEMENT, and the cost resolver matched the call ANCHORED at the start of
 # the body — so it gave up on the whole card. 219 non-land factories with a
 # real printed cost read as `nonliteral` off a helper the type and subtype
 # columns followed fine.
 ("fires", "a cost through a helper whose call is its tail (bfz Hero of Goma Fada)",
  "sets/bfz/creatures.rs",
  '            "Hero of Goma Fada",\n            cost(&[generic(4), w()]),',
  '            "Hero of Goma Fada",\n            cost(&[generic(5), w()]),'),
 # ⚠ A PARAMETER CAN BE ONE ELEMENT OF THE SYMBOL LIST. `fn zubera(name,
 # color_pip: ManaSymbol, dies) { cost: cost(&[generic(1), color_pip]), .. }`
 # — the same "the binding is one element" shape `planeswalker_subtypes:
 # vec![sub]` needed, and binding only the whole expression left the card
 # unreadable.
 ("fires", "a cost whose pip is a bound parameter (chk Ember-Fist Zubera)",
  "sets/chk.rs",
  '        "Ember-Fist Zubera",\n        r(),',
  '        "Ember-Fist Zubera",\n        u(),'),
 # `ManaCost::default()` is an explicit `{0}`, a VALUE and not a gap — reading
 # it as unreadable skipped 31 zero-cost cards and took their COLOUR with them,
 # which is where five Pacts and six suspend cards were hiding.
 ("fires", "an explicit empty cost (mod_set Slaughter Pact)",
  "sets/mod_set/instants.rs",
  '        name: "Slaughter Pact",\n'
  "        color_indicator: vec![crate::mana::Color::Black],\n"
  "        cost: crate::mana::ManaCost::default(),",
  '        name: "Slaughter Pact",\n'
  "        color_indicator: vec![crate::mana::Color::Black],\n"
  "        cost: cost(&[generic(1)]),"),
 # ⚠ A SHORTHAND FIELD CAN BE A LOCAL, AND THE LOCAL CAN CARRY THE HELPER'S OWN
 # PUSH. `fn mount(types) -> Subtypes { let mut creature_types = types;
 # creature_types.push(Mount); Subtypes { creature_types, .. } }` — the Saddle
 # set — and `tla`'s `fn ally`, which is the same shape with the push behind a
 # `contains` guard. ⚠ That family HAS shipped this defect: `tla`'s helper was
 # `ct` under a misleading name until 2026-09-01 and ten callers had no Ally.
 ("fires", "a Subtypes helper's own push onto a local (recent66 fn mount)",
  "sets/decks/recent66.rs",
  "    creature_types.push(CreatureType::Mount);\n", ""),
 # A slice parameter (`fn ct(types: &[CreatureType])`) binds to `&[..]`, which
 # is the same list as `vec![..]` and was read as unreadable.
 ("fires", "a subtype list through a slice parameter (tla fn ct)",
  "sets/decks/tla.rs",
  "fn ct(types: &[CreatureType]) -> Subtypes {\n"
  "    Subtypes { creature_types: types.to_vec(), ..Default::default() }",
  "fn ct(types: &[CreatureType]) -> Subtypes {\n"
  "    let _ = types;\n"
  "    Subtypes { creature_types: vec![], ..Default::default() }"),
 # The layout gate: a transform card reaches the subtype comparison with its
 # own FACE's type line, and `layout == \"normal\"` used to drop it silently.
 ("fires", "a subtype on a transform face (modern Tormented Pariah)",
  "sets/decks/modern.rs",
  "        vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Werewolf],",
  "        vec![CreatureType::Human, CreatureType::Werewolf],"),
 # `notyped` 7 -> 0: the two shapes that reach their base through something the
 # base scan (which walks CALLS, anchored at the start of a depth-1 line) could
 # not see. Both cards were audited by NOBODY until the chain followed them, so
 # neither of these anchors is reachable by any other case.
 ("fires", "a nested literal as the base (war Prismite)", "sets/war.rs",
  # ⚠ ANCHORED ON THE NAME, because `creatures(vec![Golem])` occurs twice in
  # the file and an anchor that is not unique reports BROKEN rather than
  # failing — which is the battery working, and the reason to read its output
  # rather than its exit code.
  "            name: \"Prismite\",\n            cost: cost(&[generic(2)]),\n"
  "            subtypes: creatures(vec![CreatureType::Golem]),",
  "            name: \"Prismite\",\n            cost: cost(&[generic(2)]),\n"
  "            subtypes: creatures(vec![CreatureType::Wall]),"),
 ("fires", "a base spread that is not the start of its line (nms2 Arc Mage)",
  "sets/nms2.rs",
  "        vec![CreatureType::Human, CreatureType::Spellshaper],\n"
  "        cost(&[generic(2), r()]),",
  "        vec![CreatureType::Human],\n"
  "        cost(&[generic(2), r()]),"),
]
# ⚠ EVERY ROW KIND, or a column's cases cannot fire. The colour column was
# added with its row kind missing here, and its injection read "silent" — the
# battery's own version of the bug it exists to catch.
ROW = re.compile(r"^  (?:sub|kw|types|super|cost|p/t|color|loyalty|no-cost"
                 r"|adv-cost|adv-types|back-types|back-p/t) ", re.M)
# EVERY row kind of the name audit too — `duplicate` and `mismatch` were
# added after this line and would have been invisible to it, which is the
# third time that exact omission has come up in this file.
NAME_ROW = re.compile(r"^  (?:spelling|unknown|duplicate|mismatch) +'", re.M)


def run(catalog):
    """Rows from BOTH oracle-backed catalog audits, against `catalog`.

    `audit_card_names.py` shares this file's name reader, and its own rows are
    the population `audit_printed_body` drops as `nocache` — so an injection
    that breaks a NAME is silent in one and loud in the other, and the battery
    has to see both or half of what it covers cannot fail. It imports
    `audit_printed_body` and reads `apb.CATALOG`, so one variable steers both.
    """
    env = dict(os.environ, CRAB_CATALOG_DIR=str(catalog))
    out = subprocess.run([sys.executable, "scripts/audit_printed_body.py", "--rows", "0"],
                         cwd=ROOT, capture_output=True, text=True, env=env).stdout
    names = subprocess.run([sys.executable, "scripts/audit_card_names.py"],
                           cwd=ROOT, capture_output=True, text=True, env=env).stdout
    rows = len(ROW.findall(out)) + len(NAME_ROW.findall(names))
    head = next((l for l in out.split("\n") if l.startswith("# compared")), "")
    return rows, head


SRC = ROOT / "crabomination_catalog" / "src"
ap = argparse.ArgumentParser()
ap.add_argument("-j", "--jobs", type=int, default=min(6, os.cpu_count() or 1),
                help="cases in flight; each holds one 24 MB catalog copy")
ap.add_argument("-k", "--only", default=None,
                help="run only cases whose label contains this substring")
opts = ap.parse_args()
cases = [c for c in CASES if opts.only is None or opts.only in c[1]]
jobs = max(1, min(opts.jobs, len(cases)))

with tempfile.TemporaryDirectory(prefix="apb_inject_") as tmp:
    tmp = pathlib.Path(tmp)
    # One copy per worker, reused across that worker's cases: the copy is 24 MB
    # and ~1 s, the audit pair is ~2 min, so copying per CASE would be free too
    # — but a pool of `jobs` trees keeps the peak bounded whatever `-j` is.
    pool = [tmp / f"w{i}" for i in range(jobs)]
    for d in pool:
        shutil.copytree(SRC, d)
    base, line = run(pool[0])
    print(f"BASELINE {base} rows\n  {line}\n")

    # ⚠ A SLOT IS TAKEN, NOT COMPUTED. `slot = i % jobs` looks like it pins one
    # tree per worker and does not: with two workers, case 2 (slot 0) starts as
    # soon as case 1 finishes, while case 0 — also slot 0 — is still running.
    # Two injections in one tree is a silently wrong reading, so the slot comes
    # off a queue and goes back when the case is done.
    free = queue.Queue()
    for i in range(jobs):
        free.put(i)

    def one(case):
        expect, label, rel, old, new = case
        slot = free.get()
        try:
            return run_case(pool[slot], case)
        finally:
            free.put(slot)

    def run_case(root, case):
        expect, label, rel, old, new = case
        p = root / rel
        orig = p.read_text()
        n = orig.count(old)
        if n != 1:
            return None, f"BROKEN CASE     pattern occurs {n}x   {label}"
        try:
            p.write_text(orig.replace(old, new, 1))
            rows, _ = run(root)
        finally:
            # Restoring matters even on a copy: the slot is reused by the next
            # case, and a leftover patch would move ITS baseline silently.
            p.write_text(orig)
        delta = rows - base
        ok = (delta > 0) if expect == "fires" else (delta == 0)
        return ok, (f"{'ok' if ok else 'FAILED':8} expect {expect:6} "
                    f"got {delta:+5d} rows   {label}")

    bad = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as ex:
        futs = [ex.submit(one, c) for c in cases]
        for f in futs:
            ok, msg = f.result()
            bad += not ok
            print(msg)
print(f"\n{len(cases) - bad}/{len(cases)} as expected")
sys.exit(1 if bad else 0)
