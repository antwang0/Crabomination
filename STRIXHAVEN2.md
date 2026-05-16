# Strixhaven implementation tracker

Two adjacent catalogs:

1. **Secrets of Strixhaven (SOS)** — the 255-card supplemental set
   (`catalog::sets::sos`). Per-color tables below.
2. **Strixhaven: School of Mages (STX)** — the 2021 base set
   (`catalog::sets::stx`). Tables in the "Strixhaven base set (STX)"
   section.

## Legend

- ✅ done — wired with full functionality
- 🟡 partial — body wired with simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

## Status summary

| Set | ✅ done | 🟡 partial | ⏳ todo |
|---|---|---|---|
| SOS (255 cards) | 195 | 59 | 1 |
| STX (195 cards) | 306 | 14 | 0 |
| STA reprints (in STX boosters) | 46 | 0 | — |

Push (modern_decks, claude/modern_decks branch — latest revision —
**22 NEW STX cards + 45 new functionality tests + CR 605 audit**):

This push adds 22 new card factories to `stx::extras` exercising
existing engine primitives — focused on the Silverquill (W/B)
school + cross-college utility. All 1794 tests pass. The cards
cover the full Strixhaven design vocabulary (Magecraft pump,
Pest/Inkling/Spirit tribal, lifegain payoffs, draw-loot, edict-on-a-body,
+1/+1 counter doubling) using only existing primitives.

**NEW STX cards (this push, 22):**

- **Silverquill Pledge** ({1}{W}{B}, Instant) — +3/+1 EOT.
  Tests: `silverquill_pledge_pumps_target_three_one`.
- **Inkwell Strider** ({2}{W}{B}, 2/3 Inkling Soldier with
  Flying + Lifelink) — Tribal-anthem target.
- **Scolding Detention** ({2}{W}, Sorcery) — Tap + two stun
  counters on opp creature. Tests: `scolding_detention_taps_and_stuns_twice`.
- **Lesson Recall** ({1}{U}, Instant) — Return IS card from gy
  to hand + cantrip. Tests: `lesson_recall_returns_instant_and_cantrips`.
- **Pestilent Acolyte** ({2}{B}, 2/3 Human Warlock) — ETB -1/-1 EOT.
  Tests: `pestilent_acolyte_etb_kills_one_toughness_creature`.
- **Stoneglare Lecturer** ({3}{W}, 3/3 Cat Cleric) — ETB +2 life
  + draw. Tests: `stoneglare_lecturer_etb_gains_life_and_draws`.
- **Critical Critique** ({1}{B}, Instant) — -2/-2 EOT + Scry 1.
  Tests: `critical_critique_kills_two_two_and_scrys`.
- **Quandrix Manipulator** ({2}{G}{U}, 3/3 Elf Druid) — ETB
  doubles +1/+1 counters on a creature (CountersOn pattern).
  Tests: `quandrix_manipulator_doubles_counters_on_target_creature`.
- **Prismari Iteration** ({2}{U}{R}, Sorcery) — Discard 1, draw 2
  (looter).
- **Lorehold Battle-Priest** ({2}{R}{W}, 2/4 Spirit Cleric with
  First Strike + Vigilance).
- **Witherbloom Reaper** ({3}{B}{G}, 4/3 Plant Warlock with
  Deathtouch) — ETB each opp sacs a creature (edict-on-a-body).
  Tests: `witherbloom_reaper_etb_edicts_each_opp`.
- **Pyromancer's Bolt** ({1}{R}, Instant) — 3 damage to creature/PW.
- **Symmetry Lecturer** ({1}{G}{U}, 2/2 Elf Wizard with Flash)
  — ETB +1/+1 counter on another friendly creature.
- **Wisdom of the Ancients** ({3}{U}, Sorcery) — Draw 3.
- **Mob Mentality** ({1}{R}{W}, Instant) — Friendlies get +1/+1 EOT;
  if you cast another spell this turn, also First Strike EOT.
- **Witherbloom Drain Ritual** ({2}{B}{G}, Sorcery) — Drain 3 from
  each opp.
- **Mystical Inquiry** ({2}{U}, Sorcery) — Tutor an instant/sorcery.
- **Conjurer's Bauble** ({0}, Artifact, STA reprint) — `{1}, Sac:
  Draw a card`. Zero-mana cantrip artifact.
- **Quartzwood Inkling** ({2}{B}, 3/2 Inkling Soldier with Menace)
  — Tenured Inkcaster anthem target.
- **Pop Quiz Lecturer** ({2}{W}, 2/3 Human Cleric with Vigilance)
  — ETB Scry 2.
- **Brilliant Restoration** ({3}{W}{W}, Sorcery) — Reanimate
  creature card + 2 life.
- **Inkling Studies** ({2}{W}{B}, Sorcery) — Mint two Inkling
  tokens.
- **Spirit Banner** ({3}, Artifact) — Tribal anthem for Spirits.
  Tests: `spirit_banner_pumps_spirits_by_one_one`,
  `spirit_banner_does_not_pump_non_spirits`.
- **Spectral Adjudicator** ({3}{W}, 2/3 Spirit Cleric with Flying
  + Lifelink).
- **Quandrix Doubling Tutor** ({2}{G}{U}, Sorcery) — Mint two 0/0
  Fractals; pump each Fractal you control with a +1/+1 counter.

**CR 605 audit row** (added to TODO.md): Mana Abilities — Both
activated (605.1a) and triggered (605.1b) mana ability variants
verified. Engine recogniser in `is_mana_ability` correctly identifies
pure `Effect::AddMana` (and `Seq` of AddMana) activations as bypassing
the stack per CR 605.3b; the triggered-mana-ability fast-path
(CR 605.4a) is still tracked as ⏳ (no STX/SOS card requires it).

Push (modern_decks, claude/modern_decks branch — prior revision —
**21 NEW STX cards + 1 engine primitive (`Predicate::OpponentControlsMoreLandsThanYou`)
+ CR 701.10 + CR 122.6 audit**):

This push adds 21 new card factories to `stx::extras` along with
46+ new functionality tests. All 1744 tests pass. Includes one new
engine primitive that promotes Gift of Estates 🟡 → ✅:

1. **`Predicate::OpponentControlsMoreLandsThanYou`** (`effect.rs` +
   `game/effects/eval.rs`) — Walks the battlefield, counts lands per
   seat, and returns true iff any opponent (filtered by team /
   eliminated status) has strictly more lands than the predicate's
   controller. Wires Gift of Estates's printed Oracle "If an opponent
   controls more lands than you, …" gate via `Effect::If { cond:
   OpponentControlsMoreLandsThanYou, then: Seq(3× Search Plains),
   else_: Noop }`. Same primitive unblocks Tithe, Knight of the White
   Orchid's ETB trigger, Land Tax, and any future "catch-up" payoffs.
   Tests: `gift_of_estates_searches_three_plains_when_opp_has_more_lands`,
   `gift_of_estates_skips_search_when_lands_equal`.

**NEW STX cards (Silverquill-flavored creatures + utility):**

- **Inkrise Infiltrator** ({1}{B} 2/1 Inkling Rogue) — Menace. Vanilla
  Inkling body that scales with Tenured Inkcaster's +2/+2 tribal
  anthem. Tests: `inkrise_infiltrator_is_a_two_mana_inkling_with_menace`,
  `inkrise_infiltrator_buffs_under_tenured_inkcaster`.
- **Sigardian Savior** ({3}{W}{W} 4/4 Angel Flying) — ETB reanimate
  a creature card with MV ≤ 3 from your graveyard. Wired via
  `Move(target gy creature → battlefield)` with `ManaValueAtMost(3)`.
  Tests: `sigardian_savior_is_a_five_mana_four_four_flying_angel`,
  `sigardian_savior_etb_returns_low_mv_creature_card`.
- **Sneaky Snacker** ({B} 1/1 Rat Rogue) — Menace + sorcery-speed
  `{2}{B}: Return Sneaky Snacker from your graveyard to your hand`
  via `from_graveyard: true` activation. Tests:
  `sneaky_snacker_is_a_one_mana_rat_with_menace`,
  `sneaky_snacker_recurs_from_graveyard_to_hand`.
- **Soulknife Spy** ({1}{U} 1/3 Human Rogue) — Combat-damage
  optional pay-{U}-to-draw rider via `MayPay { U → Draw 1 }`.
  Test: `soulknife_spy_is_a_two_mana_one_three_rogue`.
- **Daring Diversion** ({3}{R} Sorcery) — Deals 2 damage to each of
  two target creatures. Tests:
  `daring_diversion_is_a_four_mana_red_sorcery`,
  `daring_diversion_burns_one_creature`.
- **Possibility Storm** ({2}{R} Enchantment, body-only) — Placeholder
  Lorwyn reprint flavor; full cast-from-exile-on-spell-cast trigger
  ⏳ (cast-from-exile pipeline). Test:
  `possibility_storm_is_a_three_mana_red_enchantment`.
- **Pilgrim of the Ages** ({3} 1/1 Spirit) — `{2}, Sac: Search basic
  land → hand`. Tests:
  `pilgrim_of_the_ages_is_a_three_mana_one_one_spirit`,
  `pilgrim_of_the_ages_sac_searches_for_basic_land`.
- **Strixhaven Spawner** ({3}{G}{U} Sorcery) — Create three 0/0
  Fractal tokens with two +1/+1 counters each via Seq(CreateToken
  count=3, ForEach Fractal +2 counters). Tests:
  `strixhaven_spawner_is_a_five_mana_gu_sorcery`,
  `strixhaven_spawner_creates_three_fractal_tokens`.
- **Mage Hunter Defender** ({2}{B} 2/3 Defender Wizard) — Magecraft
  drain 1 from each opp via `magecraft_drain_each_opp(1)`. Tests:
  `mage_hunter_defender_is_a_three_mana_defender_wizard`,
  `mage_hunter_defender_drains_on_instant_cast`.
- **Detention Sphere** ({1}{W}{U} Enchantment) — ETB exile target
  nonland permanent. Until-leaves return rider ⏳. Tests:
  `detention_sphere_exiles_target_nonland_permanent`.
- **Mascot Trainer** ({2}{G} 2/2 Human Druid) — "Other tokens you
  control get +1/+1" via `PumpPT` static against
  `EachPermanent(Creature & ControlledByYou & IsToken & OtherThanSource)`.
  Tests: `mascot_trainer_is_a_three_mana_two_two_druid`,
  `mascot_trainer_does_not_buff_non_tokens`.
- **Quandrix Cryptidkeeper** ({2}{G}{U} 3/3 Elf Druid) — ETB +1/+1
  ×2 on another friendly creature. Tests:
  `quandrix_cryptidkeeper_is_a_four_mana_three_three_elf_druid`,
  `quandrix_cryptidkeeper_etb_pumps_friendly`.
- **Ember Anvil** ({3} Artifact) — `{T}: Add {R} or {W}` (two mana
  abilities) + `{3}, {T}, Sac: Search Spirit creature → hand`.
  Test: `ember_anvil_is_a_three_mana_artifact`.
- **Witherbloom Strangler** ({1}{B}{G} 2/2 Plant Warlock) — ETB
  -2/-2 EOT on opp creature. Tests:
  `witherbloom_strangler_is_a_three_mana_two_two_plant_warlock`,
  `witherbloom_strangler_kills_two_two_creature`.
- **Glasspool Embellisher** ({U} Instant) — Draw 1, discard 1.
  Tests: `glasspool_embellisher_is_a_one_mana_blue_instant`,
  `glasspool_embellisher_loots_one`.
- **Lorehold Reanimator** ({2}{R}{W} 3/3 Spirit Cleric) — ETB
  optional reanimate MV ≤ 2 creature card from your graveyard via
  `MayDo`. Test:
  `lorehold_reanimator_is_a_four_mana_three_three_spirit_cleric`.
- **Prismari Eruption** ({3}{U}{R} Sorcery) — 2 damage to each
  non-flying creature + Scry 1. Tests:
  `prismari_eruption_is_a_five_mana_ur_sorcery`,
  `prismari_eruption_burns_grounded_creatures_and_spares_flyers`.
- **Silverquill Inquisitor** ({1}{W}{B} 2/2 Human Cleric) — ETB
  random discard from opp hand. Tests:
  `silverquill_inquisitor_is_a_three_mana_two_two_cleric`,
  `silverquill_inquisitor_etb_discards_from_opp_hand`.
- **Lorehold Spectral Lecturer** ({3}{R}{W} 4/3 Spirit Cleric Wizard
  Vigilance) — Magecraft self-pump (+1/+0 + lifelink EOT). Test:
  `lorehold_spectral_lecturer_is_a_five_mana_four_three_spirit_cleric_wizard`.
- **Pop Quiz Recital** ({2}{W} Sorcery — Lesson) — Two-mode
  ChooseMode: PumpPT(+2/+2 + Flying EOT) or PumpPT(+0/+3 + Vigilance
  EOT). Test: `pop_quiz_recital_is_a_three_mana_white_lesson`.
- **Diviner's Wand** ({4} Artifact — Equipment) — Body-only frame;
  Equip-grant + combat-damage-draw rider ⏳. Test:
  `diviners_wand_is_a_four_mana_equipment`.
- **Fascinating Lecture** ({1}{U} Sorcery — Lesson) — Draw 2,
  discard 1. Tests:
  `fascinating_lecture_is_a_two_mana_blue_lesson`,
  `fascinating_lecture_draws_two_discards_one`.
- **Quandrix Sphinx** ({3}{G}{U} 3/4 Sphinx Druid Flying) — ETB
  +1/+1 counter on each friendly creature via ForEach. Tests:
  `quandrix_sphinx_is_a_five_mana_three_four_flying_sphinx_druid`,
  `quandrix_sphinx_etb_counters_each_friendly_creature`.
- **Witherbloom Necrotutor** ({2}{B}{B} 3/2 Human Warlock) — ETB
  Raise Dead + lose 2 life. Tests:
  `witherbloom_necrotutor_is_a_four_mana_three_two_warlock`,
  `witherbloom_necrotutor_etb_returns_creature_card_and_loses_two_life`.

Push (modern_decks, claude/modern_decks branch — prior revision —
**21 NEW STX cards + 2 engine improvements (stack-aware `find_card_owner`
+ library/hand zone fallback in `evaluate_requirement_static`)**):

This push adds 21 new card factories to `stx::extras` along with
33+ new functionality tests. All 1702 tests pass. Includes two
engine improvements:

1. **`find_card_owner` now checks the stack** (`game/mod.rs`) —
   previously `find_card_owner` walked battlefield + per-player hidden
   zones + exile, but didn't check `StackItem::Spell.card.owner`. This
   broke `PlayerRef::OwnerOf(Selector::TriggerSource)` resolution for
   SpellCast triggers (the cast spell is on the stack mid-resolution,
   not yet in any persistent zone). Wires Cunning Rhetoric's "you gain
   1 life, the casting player loses 1 life" rider faithfully.

2. **`evaluate_requirement_static` now checks library + hand**
   (`game/effects/eval.rs`) — previously only walked battlefield + per-
   player graveyards + exile + stack. Cards on the top of library
   (e.g. Lurking Predators's "if it's a creature card, …" check) now
   correctly resolve their card-type and creature-type filters. The
   library / hand info is hidden in real play but the engine's
   permission-checked at the call site (effects target the
   controller's own zones).

**NEW STX cards (1 vanilla + 1 ETB cantrip + 19 effect spells / utility):**

- **Revitalize** ({1}{W} Instant, M19 reprint flavor) — Gain 3 life,
  draw a card. Wired as `Seq(GainLife 3, Draw 1)`. Tests:
  `revitalize_gains_three_and_draws`,
  `revitalize_is_a_two_mana_white_instant`.
- **Grim Bounty** ({3}{B} Instant) — Destroy target creature; create
  a Treasure token. Tests:
  `grim_bounty_destroys_target_creature_and_creates_treasure`,
  `grim_bounty_is_a_four_mana_black_instant`.
- **Growth Spiral** ({G}{U} Instant, RNA reprint flavor) — Draw a
  card; may put a land from hand onto bf. Optional land-drop via
  `MayDo`. Tests: `growth_spiral_draws_a_card`,
  `growth_spiral_optional_land_drop_with_scripted_decider`,
  `growth_spiral_is_a_two_mana_gu_instant`.
- **Idyllic Tutor** ({2}{W} Sorcery) — Search library for enchantment
  to hand. Tests:
  `idyllic_tutor_searches_an_enchantment_to_hand`,
  `idyllic_tutor_is_a_three_mana_white_sorcery`.
- **Gift of Estates** ({W} Sorcery) — Search library for up to three
  Plains to hand. The "if opp controls more lands" gate is omitted
  (no `Predicate::AnyOppHasMoreLands` primitive). Tests:
  `gift_of_estates_searches_three_plains`,
  `gift_of_estates_is_a_one_mana_white_sorcery`.
- **Pillage** ({1}{R}{R} Sorcery) — Destroy target artifact or land.
  Tests: `pillage_destroys_target_land`,
  `pillage_destroys_target_artifact`,
  `pillage_is_a_three_mana_red_sorcery`.
- **Slip Through Space** ({U} Instant, OGW reprint flavor) — Target
  creature can't be blocked this turn; draw a card. Tests:
  `slip_through_space_grants_unblockable_and_draws`,
  `slip_through_space_is_a_one_mana_blue_instant`.
- **Doomskar** ({3}{W}{W} Sorcery, Kaldheim reprint flavor) — Destroy
  each creature. Foretell alt cost omitted. Tests:
  `doomskar_destroys_each_creature`,
  `doomskar_is_a_five_mana_white_sorcery`.
- **Battle Mammoth** ({3}{G}{G} Creature — Elephant 6/5 Trample, STA
  reprint) — Body-only wire; "draw on opp-target" rider omitted (no
  `EventKind::BecameTarget` event). Test:
  `battle_mammoth_is_a_five_mana_six_five_trampler`.
- **Mind Drain** ({1}{B}{B} Sorcery) — Each opp discards two cards.
  Wired via `ForEach(EachOpponent) → Discard 2`. Tests:
  `mind_drain_makes_each_opp_discard_two`,
  `mind_drain_is_a_three_mana_black_sorcery`.
- **Hindering Light** ({W}{U} Instant, Lorwyn reprint flavor) —
  Counter target spell + draw a card. Target-restriction (spell
  targeting you or your permanent) omitted. Tests:
  `hindering_light_counters_target_spell_and_draws`,
  `hindering_light_is_a_two_mana_wu_instant`.
- **Soul Shatter** ({2}{B}{R} Instant) — Each opp sacrifices a
  creature or PW. "Greatest mana value" rider collapsed (no
  max-by-MV sacrifice picker). Tests:
  `soul_shatter_each_opp_sacrifices_a_creature`,
  `soul_shatter_is_a_four_mana_br_instant`.
- **Lurking Predators** ({4}{G}{G} Enchantment, Onslaught reprint
  flavor) — Whenever an opp casts a spell, conditionally drop top of
  library if it's a creature. Wired via OpponentControl SpellCast
  trigger + `EntityMatches(TopOfLibrary, Creature)`. Tests:
  `lurking_predators_drops_creature_when_opp_casts`,
  `lurking_predators_is_a_six_mana_green_enchantment`.
- **Prowling Caracal** ({1}{W} Creature — Cat 3/2) — Vanilla aggro
  body. Test: `prowling_caracal_is_a_two_mana_three_two_cat`.
- **Elvish Visionary** ({1}{G} Creature — Elf Shaman 1/1, M11 reprint
  flavor) — ETB cantrip. Tests:
  `elvish_visionary_draws_on_etb`,
  `elvish_visionary_is_a_two_mana_one_one_elf_shaman`.
- **Sungrass Egg** ({2} Artifact) — `{1}, {T}, Sac: Add two mana of
  any one color.` Tests:
  `sungrass_egg_sac_adds_two_mana_of_one_color`,
  `sungrass_egg_is_a_two_mana_artifact`.
- **Mascot Summoning** ({3}{W} Sorcery — Lesson) — Mints a 2/2 W Cat
  with Lifelink. Tagged `SpellSubtype::Lesson`. Tests:
  `mascot_summoning_creates_a_two_two_lifelink_cat`,
  `mascot_summoning_is_a_four_mana_white_lesson`.
- **Scry Inversion** ({2}{U} Instant) — Scry 2, then draw 2. Tests:
  `scry_inversion_scrys_and_draws_two`,
  `scry_inversion_is_a_three_mana_blue_instant`.
- **Cunning Rhetoric** ({2}{W}{B} Enchantment) — Whenever an opp
  casts a spell, drain 1 (you gain 1, they lose 1). Engine
  improvement #1 above makes the stack-resident spell's owner
  resolvable via `PlayerRef::OwnerOf(Selector::TriggerSource)`.
  Tests: `cunning_rhetoric_drains_on_opp_cast`,
  `cunning_rhetoric_is_a_four_mana_wb_enchantment`.
- **Library Larcenist** ({1}{B}{G} Creature — Pest Rogue 2/3) —
  Combat-damage-to-player trigger mills 2. Test:
  `library_larcenist_is_a_three_mana_two_three_pest_rogue`.
- **Dean's List** ({1}{U} Sorcery) — Look at top 4, take 1 to hand,
  rest to graveyard. Tests:
  `deans_list_takes_top_card_and_mills_rest`,
  `deans_list_is_a_two_mana_blue_sorcery`.

STX corpus now at **263 ✅ + 14 🟡** (was 242 ✅ + 14 🟡).

Push (modern_decks, claude/modern_decks branch — prior revision —
**20 NEW STX cards (Silverquill tutor / new Lessons / synthesised
Quandrix/Lorehold flavor + STA reprint Mortician Beetle)**):

This push adds 20 new card factories to `stx::extras` along with
35+ new functionality tests, all using existing engine primitives.
All 1661 tests pass and clippy is clean.

**NEW STX cards (12 real Strixhaven 2021 + 4 STA reprint + 4 synthesised):**

- **Search for Glory** ({2}{W} Sorcery, Silverquill) — Scry 1 + tutor
  for a creature / enchantment / legendary / planeswalker card. Wired
  as `Seq(Scry 1, Search → Hand)` with a multi-type OR filter. Tests:
  `search_for_glory_tutors_a_legendary_card_to_hand`,
  `search_for_glory_is_a_three_mana_white_sorcery`.
- **Fervent Strike** ({R/G} Instant, hybrid) — Pump (+2/+0) + grant
  trample EOT against a Creature target. Hybrid {R/G} approximated
  as {R}. Tests: `fervent_strike_pumps_target_and_grants_trample`,
  `fervent_strike_is_a_one_mana_instant`.
- **Elemental Summoning** ({2}{U}{R} Sorcery — Lesson, Prismari) —
  Creates one 4/4 U/R Elemental token. Tagged `SpellSubtype::Lesson`.
  Tests: `elemental_summoning_mints_a_four_four_elemental`,
  `elemental_summoning_is_a_four_mana_lesson_sorcery`.
- **Humiliate** ({1}{W}{B} Sorcery, Silverquill) — DiscardChosen
  (nonland) against EachOpponent + Drain 1. Tests:
  `humiliate_strips_opp_nonland_and_drains_one`,
  `humiliate_is_a_three_mana_silverquill_sorcery`.
- **Elite Spellbinder** ({1}{W}{B}, 3/1 Human Cleric, Flying) — ETB
  DiscardChosen (nonland) against opp hand. The "exile + may cast +
  {2} more" cost rider is omitted (no may-cast-from-exile primitive).
  Tests: `elite_spellbinder_etb_strips_opp_nonland`,
  `elite_spellbinder_is_a_three_mana_three_one_flying_human`.
- **Waker of Waves** ({3}{U}{U}, 5/5 Elemental) — ETB Draw 2 / Discard
  2; gy-exile activation `{2}{U}{U}, Exile this: +5/+5 + Trample EOT`.
  Uses the existing `from_graveyard: true` + `exile_self_cost: true`
  fields. Tests: `waker_of_waves_etb_loots_two`,
  `waker_of_waves_is_a_five_mana_five_five_elemental`.
- **Discover the Formula** ({3}{U}{U} Sorcery, Quandrix) — Scry 1 +
  Draw 3. The Magecraft rider on a Sorcery is approximated as the
  initial Scry trigger. Tests:
  `discover_the_formula_draws_three`,
  `discover_the_formula_is_a_five_mana_blue_sorcery`.
- **Mortician Beetle** ({B} Insect 1/1, STA reprint Conflux) —
  +1/+1 counter on creature death (any player). Approximates "sac"
  via the generic CreatureDied event. Tests:
  `mortician_beetle_grows_on_creature_death`,
  `mortician_beetle_is_a_one_mana_one_one_insect`.
- **Vespine Strix** ({1}{U}, 1/2 Bird, Flying, synthesised STX) —
  ETB Scry 2 flyer. Tests:
  `vespine_strix_is_a_two_mana_one_two_flying_bird`.
- **Witherbloom Apprenticeship** ({2}{B}{G} Sorcery, synthesised
  Witherbloom flavor) — Mint two Pest tokens + put a +1/+1 counter
  on each creature you control. Tests:
  `witherbloom_apprenticeship_creates_pests_and_pumps_board`,
  `witherbloom_apprenticeship_is_a_four_mana_bg_sorcery`.
- **Wandering Mind** ({1}{U}, 1/3 Spirit Wizard, Flying, synthesised
  Prismari flavor) — Magecraft: Scry 1. Wired via the shared
  `magecraft(Effect::Scry)` helper. Tests:
  `wandering_mind_is_a_two_mana_one_three_flying_spirit_wizard`.
- **Lecturing Loxodon** ({4}{W}, 4/4 Elephant Cleric, Vigilance,
  synthesised Silverquill flavor) — ETB pumps other creatures you
  control +1/+1 EOT. Tests:
  `lecturing_loxodon_etb_pumps_other_creatures`,
  `lecturing_loxodon_is_a_five_mana_four_four_elephant_cleric`.
- **Sequence Engine** ({2}{R}{W} Sorcery, synthesised Lorehold tutor)
  — RevealUntilFind for an IS card to hand. Misses go to bottom of
  library in random order (`RevealMissDest::BottomRandom`).
- **Curriculum Crab** ({2}{G}{U}, 3/4 Crab, synthesised Quandrix
  flavor) — ETB MayDo fan-out +1/+1 counter on each friendly
  creature. AutoDecider declines; ScriptedDecider can opt in. Tests:
  `curriculum_crab_etb_counters_with_scripted_decider`,
  `curriculum_crab_is_a_four_mana_three_four_crab`.
- **Pyrotechnics** ({3}{R} Sorcery, synthesised STX-flavor classic
  burn) — 4 damage to one creature/PW. Multi-target divided damage
  rider collapses to single target (shared with Magma Opus /
  Electrolyze). Tests:
  `pyrotechnics_burns_target_creature_for_four`,
  `pyrotechnics_is_a_four_mana_red_sorcery`.
- **Tome of the Guildpact** ({2} Artifact, synthesised STX
  colorless cantrip rock) — `{2}, {T}: Draw a card.` Tests:
  `tome_of_the_guildpact_activation_draws_a_card`,
  `tome_of_the_guildpact_is_a_two_mana_artifact`.
- **Stormwild Capridor** ({3}{W} Goat Beast 1/4 Flying, real STX) —
  Body-only wire. Noncombat-damage prevention + counter-conversion
  rider is omitted (engine has no non-combat damage replacement
  primitive). Tests:
  `stormwild_capridor_is_a_four_mana_one_four_flying_goat_beast`.
- **Final Payment** ({W}{B} Instant, real STX Silverquill) —
  Destroy target Creature ∨ Planeswalker. The "sac creature/
  enchantment OR pay 5 life" additional cost is omitted (no
  multi-mode alt-cost primitive); the body's removal half is the
  headline play pattern. Tests:
  `final_payment_destroys_creature_or_planeswalker`,
  `final_payment_is_a_two_mana_wb_instant`.
- **Witch's Cauldron** ({1}{B}{G} Artifact, synthesised Witherbloom
  sac-engine payoff) — Tap: sacrifice a creature, gain 2 life, draw
  a card. The "X = sacrificed creature's toughness" half is
  approximated as a flat 2 life (`sacrificed_toughness` not yet
  stamped in activation cost path). Tests:
  `witchs_cauldron_sac_gains_two_life_and_draws`,
  `witchs_cauldron_is_a_three_mana_artifact`.
- **Steady Stance** ({1}{W} Instant, synthesised Silverquill
  defensive trick) — +0/+3 EOT + grant vigilance EOT. Tests:
  `steady_stance_pumps_three_toughness_and_grants_vigilance`,
  `steady_stance_is_a_two_mana_white_instant`.

STX corpus now at **242 ✅ + 14 🟡** (was 222 ✅ + 14 🟡).

Push (modern_decks, claude/modern_decks branch — prior revision —
**5 MORE NEW STX cards (Soothing Hush, Vortex Runner, Sage of the Beyond,
Frostpyre Arcanist, Inkfathom Divers)**):

- **Soothing Hush** ({1}{U} Instant) — Counter target creature spell.
  Test: `soothing_hush_counters_creature_spell`.
- **Vortex Runner** ({1}{U}, 2/1 Merfolk Wizard, Unblockable) — Evasive
  chip-shot. Test:
  `vortex_runner_is_a_two_mana_two_one_unblockable_merfolk`.
- **Sage of the Beyond** ({3}{U}{B}, 4/3 Specter Wizard, Flying) — Combat
  damage-to-player trigger makes the damaged player discard. Tests:
  `sage_of_the_beyond_combat_damage_makes_opp_discard`,
  `sage_of_the_beyond_is_a_five_mana_four_three_specter_wizard`.
- **Frostpyre Arcanist** ({3}{U}{R}, 4/4 Elemental Wizard) — Magecraft
  may-return IS card from gy to hand. Tests:
  `frostpyre_arcanist_magecraft_returns_is_from_graveyard`,
  `frostpyre_arcanist_is_a_five_mana_four_four_elemental_wizard`.
- **Inkfathom Divers** ({2}{U}{B}, 3/2 Merfolk Rogue, Flying) — ETB
  targeted nonland-card hand strip. Tests:
  `inkfathom_divers_etb_strips_opp_nonland_from_hand`,
  `inkfathom_divers_is_a_four_mana_three_two_flying_merfolk_rogue`.

STX corpus now at **222 ✅ + 14 🟡** (was 217 ✅ + 14 🟡).

Push (modern_decks, claude/modern_decks branch — latest revision —
**Wandering Archaic 🟡 → ✅ via new `Effect::CopySpellUnlessPaid`**):

Promotes **Wandering Archaic** 🟡 → ✅ via the new
`Effect::CopySpellUnlessPaid { what, mana_cost, count }` primitive
(`effect.rs`). At trigger resolution the engine asks the *spell's
caster* (the opp who cast the IS) yes/no via `Decision::OptionalTrigger`;
on yes + affordable pool, the engine deducts the cost from their pool
and skips the copy; on no or unaffordable, the spell is copied `count`
times. AutoDecider answers false by default (let the copy fire),
ScriptedDecider flips to true for tests. Handler lives in
`game/effects/mod.rs` alongside the existing `Effect::CopySpell`
resolver — same stack-lookup + copy-clone logic, gated on the
optional-pay decision. The "may choose new targets for the copy" half
stays engine-wide ⏳. Tests:
`wandering_archaic_lets_opp_pay_two_to_skip_copy`,
`wandering_archaic_copies_when_opp_cannot_afford_two`,
`wandering_archaic_copies_opp_instant` (the existing AutoDecider-default
test still passes since AutoDecider declines to pay).

STX corpus now at **217 ✅ + 14 🟡 = 231** (was 216 ✅ + 15 🟡).

Push (modern_decks, claude/modern_decks branch — latest revision —
**20 NEW STX cards + `StaticEffect::DoubleTokens` primitive**):

**NEW STX 2021 cards (20):**
- **Adrix and Nev, Twincasters** ({1}{G}{G}{U}{U}, 3/3 Legendary Merfolk
  Wizard) — Static: token-doubler. Wired via the new
  `StaticEffect::DoubleTokens` primitive — at `Effect::CreateToken`
  resolution time, the engine queries `GameState::token_doublers_for(seat)`
  and multiplies the spawn count by `2^k` where k is the number of
  active doublers under the controller's control. Stacking Adrixes
  multiplies (2 → 4×, 3 → 8×). Tests:
  `adrix_and_nev_doubles_token_creation`,
  `adrix_and_nev_does_not_double_opponent_tokens`,
  `adrix_and_nev_is_a_five_mana_three_three_merfolk_wizard`.
- **Strixhaven Stadium** ({4} Artifact) — Three abilities: (a) attack
  trigger pumps the attacker +1/+1 EOT, (b) combat-damage-to-player
  trigger accrues a charge counter, (c) `{T}: Draw two cards.` activation
  gated on `ValueAtLeast(CountersOn(This, Charge), 3)` with a
  `RemoveCounter(3 Charge)` cost. Tests: `strixhaven_stadium_*` (4 tests
  covering each ability + the activation rejection path).
- **Awesome Presentation** ({3}{W}{B} Sorcery) — Mints two Inkling tokens.
  Tests: `awesome_presentation_mints_two_inkling_tokens`,
  `awesome_presentation_is_a_five_mana_white_black_sorcery`.
- **Rise of Extus** ({3}{R}{W} Sorcery) — 5 damage + reanimate IS card +
  Learn. Tests:
  `rise_of_extus_deals_five_damage_and_returns_is_from_graveyard`,
  `rise_of_extus_is_a_five_mana_lorehold_sorcery`.
- **Brackish Trudge** ({2}{B}{G}, 4/3 Lizard Horror) — Body wire (Escape
  alt-cost ⏳).
- **Lurking Deadeye** ({3}{B}, 2/2 Snake Assassin) — Flash + Deathtouch +
  ETB -2/-2 EOT on target. Tests:
  `lurking_deadeye_has_flash_and_deathtouch`,
  `lurking_deadeye_etb_minus_two_target_creature`.
- **Aether Helix** ({3}{U}{R} Sorcery) — Bounce nonland + 2 damage to opp.
  Test: `aether_helix_bounces_nonland_and_burns_opp`.
- **Reflective Golem** ({2}, 1/1 Artifact Creature — Golem) — Body wire.
  Test: `reflective_golem_is_a_two_mana_one_one_artifact_creature_golem`.
- **Tempest Caller** ({3}{U}, 2/3 Merfolk Wizard) — ETB taps all opp
  creatures. Test: `tempest_caller_etb_taps_opponent_creatures`.
- **Pillardrop Warden** ({3}{W}, 2/4 Spirit Soldier, Flying) — ETB MayPay
  {2} to return a creature card from gy → hand. Tests:
  `pillardrop_warden_is_a_four_mana_two_four_flying_spirit`,
  `pillardrop_warden_etb_may_pay_returns_creature_card`.
- **Devourer of Memory** ({1}{U}{B}, 2/2 Nightmare Horror, Flying) —
  Magecraft self-pump (+1/+0 EOT) + cantrip when power ≥ 4. Tests:
  `devourer_of_memory_magecraft_pumps_self`.
- **Mavinda's Verdict** ({2}{W}{B} Instant, synth) — Exile target creature
  + gain life equal to its toughness. Test:
  `mavindas_verdict_exiles_creature_and_gains_life`.
- **Quandrix Quickener** ({G}{U} Instant, synth) — Scry 2 + Draw 1 +
  Untap target Land you control. Test:
  `quandrix_quickener_scries_and_untaps_target_land`.
- **Witherbloom Skillchaser** ({2}{B}{G}, 3/3 Pest Spirit) — ETB mints a
  Pest token. Tests: `witherbloom_skillchaser_*`.
- **Quandrix Pop Quiz** ({2}{G}{U} Sorcery, synth) — Mint a Fractal +
  put X +1/+1 counters on it (X = lands you control). Test:
  `quandrix_pop_quiz_creates_fractal_with_x_counters`.
- **Inkwood Scrivener** ({1}{W}{B}, 2/2 Inkling, Flying) — ETB drain 1.
  Tests: `inkwood_scrivener_*`.
- **Furnace Hellkite** ({4}{R}{R}, 5/5 Dragon, Flying) — ETB deals 2 to
  each opp. Tests: `furnace_hellkite_*`.
- **Pinion Lecturer** ({2}{W}, 2/3 Bird Cleric, Flying + Vigilance) —
  Vanilla body. Test:
  `pinion_lecturer_is_a_three_mana_two_three_flying_vigilance_bird_cleric`.
- **Sparkling Insight** ({3}{U} Instant) — Scry 2 then draw 2.
  Test: `sparkling_insight_scries_two_then_draws_two`.
- **Pop Quiz Coach** ({2}{G}{U}, 2/4 Merfolk Druid) — Magecraft +1/+1
  counter on target friendly creature. Test:
  `pop_quiz_coach_magecraft_adds_counter`.

**NEW engine primitive — `StaticEffect::DoubleTokens` (CR 614.13 framing):**
Per-controller continuous static effect that doubles the count of every
`Effect::CreateToken` resolution. Wired via:
(a) New `StaticEffect::DoubleTokens` variant in `effect.rs`;
(b) `GameState::token_doublers_for(seat) -> u32` walks the battlefield
counting permanents with the static (in `game/mod.rs`);
(c) `Effect::CreateToken` resolver in `game/effects/mod.rs` multiplies
the evaluated count by `2^doublers` before the spawn loop;
(d) `static_ability_to_effects` includes `DoubleTokens` in the no-op
layer-translation pass since it's read at create-time, not via layers.

STX corpus now at **216 ✅ + 15 🟡 = 231** (was 196 ✅ + 15 🟡).

Push (modern_decks, claude/modern_decks branch — latest revision —
**Ward enum + activated-ability Ward**): expanded Ward enforcement
(CR 702.21) along two axes. (1) Cost variants:
`Keyword::Ward(u32)` is now `Keyword::Ward(WardCost)` where `WardCost`
is an enum of `Mana(ManaCost) | Life(u32) | Discard(u32) |
SacrificeCreature`. New `Effect::CounterUnless { what, cost }` is the
trigger body — its resolver walks the stack for a matching `Spell`
(by `card.id`) or `Trigger` (by `source`) and tries to auto-pay the
cost on the affected controller's behalf. (2) Activated abilities:
new `push_ward_triggers_for_activated_ability` is hooked into
`activate_ability` right after the ability is queued, so Ward fires
on activated-ability targeting too (the "or ability" half of CR
702.21a). Both paths share `push_ward_triggers_for_targets`.
Promotes **Mica, Reader of Ruins** 🟡 → ✅ (Ward—Pay 3 life via
`WardCost::Life(3)`) and **Forum Necroscribe** 🟡 → ✅ (Ward—Discard
a card via `WardCost::Discard(1)`). All ~20 prior `Keyword::Ward(N)`
catalog/test sites migrated to `WardCost::generic(N)`. SOS counts:
193 → **195 ✅** (61 → 59 🟡, 1 ⏳). Tests: 6 new — `ward_pay_life_*`,
`ward_discard_*`, `ward_*_opp_activated_ability_*`.

Push (modern_decks, claude/modern_decks branch — latest revision —
**Ward enforcement (CR 702.21)**): engine-wide Ward enforcement for
mana-cost Ward on spells. New helper
`push_ward_triggers_for_cast` in `game/actions.rs` runs at the end of
`finalize_cast`: walks the just-cast spell's slot-0 + additional
targets, and for each target permanent controlled by a player other
than the caster with `Keyword::Ward(N)` (N>0), pushes a
`StackItem::Trigger` whose body is
`Effect::CounterUnlessPaid { what: Selector::Target(0), mana_cost: {N} }`
aimed at the just-cast spell. The trigger goes on top of the caster's
own SpellCast triggers (Magecraft / Prowess) — APNAP-correct, since
the caster is the active player and Ward belongs to a nonactive
player. At trigger resolution `CounterUnlessPaid` auto-pays on the
spell controller's behalf via the existing `try_pay_with_auto_tap`
path; if affordable, the spell stays, otherwise it's countered to the
caster's graveyard. Promotes **Inkshape Demonstrator** 🟡 → ✅ — the
sole 🟡 where Ward enforcement was the only remaining gap. Mica /
Forum Necroscribe / Prismari (the Inspiration) / Fractal Tender stay
🟡 — they need either a non-mana Ward variant (Pay 3 life, Discard a
card) or other engine work (storm static, Increment introspection).
Activated-ability-side Ward (the "or ability" half of CR 702.21a) is
a follow-up. Tests: `ward_counters_opp_spell_when_payer_cannot_afford`,
`ward_allows_opp_spell_when_payer_can_afford`,
`ward_does_not_trigger_on_caster_own_spell`. SOS counts: 192 → **193 ✅**
(62 → 61 🟡, 1 ⏳).

Push (modern_decks, claude/modern_decks branch — latest revision —
**newest sub-push #2**): **4 NEW cards** added on top of the prior batch:
- **Spiteful Squad** ({2}{B}, 1/1 deathtouch Skeleton) — Dies: drain 2.
- **Master Symmetrist** ({2}{G}{U}, 3/3 Fractal Wizard) — ETB doubles
  +1/+1 counters on each creature you control.
- **Stinging Cave Crawler** ({3}{B}{B}, 3/4 Insect) — ETB scry 2 +
  attack drain 1.
- **Cogwork Archivist** ({6} 4/4 Artifact Creature — Construct) — ETB
  mills 4.

Push (modern_decks, claude/modern_decks branch — latest revision —
**sub-push #1**): **11 NEW cards** (10 STX 2021 + 1 STA reprint) +
CR 701.25c audit coverage:

**NEW STX 2021 / supplemental cards (10):**
- **Pigment Storm** ({3}{R} Instant) — 4 damage to target creature.
  Tests: `pigment_storm_is_a_four_mana_red_instant`,
  `pigment_storm_deals_four_damage_to_target_creature`.
- **Inkfathom Witch** ({3}{U}{B}, 2/3 Flying Inkling) — ETB target
  opp discards a nonland card of your choice. Tests:
  `inkfathom_witch_is_a_five_mana_inkling_with_flying`,
  `inkfathom_witch_etb_makes_opp_discard_a_nonland_card`.
- **Inscription of Ruin** ({2}{B}{B} Sorcery) — `ChooseN` modes: discard
  2 vs each opp + destroy target creature (Kicker upgrade omitted).
  Tests: `inscription_of_ruin_is_a_four_mana_black_sorcery`,
  `_destroys_creature_and_discards`.
- **Tome of the Infinite** ({1} Legendary Artifact) — ETB Scry 1 +
  `{2},{T}: Draw a card`. Tests:
  `tome_of_the_infinite_is_a_one_mana_legendary_artifact`, `_etb_scrys_one`.
- **Drannith Stinger** ({2}{R}, 2/2 Goblin Wizard) — Whenever you cast
  a noncreature spell, this deals 1 damage to each opponent. Tests:
  `drannith_stinger_is_a_three_mana_two_two_goblin_wizard`,
  `_pings_opp_on_noncreature_spell`, `_does_not_ping_on_creature_cast`.
- **Mage Mauler** ({2}{R} Sorcery) — 3 damage + 1 life. Tests:
  `mage_mauler_is_a_three_mana_red_sorcery`,
  `_deals_three_to_creature_and_gains_one_life`.
- **Heirloom Mirror** ({3} Artifact) — `{T}: Add one of any` +
  `{3},{T},Sac: Draw a card`. Tests:
  `heirloom_mirror_is_a_three_mana_artifact`,
  `_tap_for_mana_then_sac_to_draw`.
- **Quandrix Mascot** ({1}{G}{U}, 2/2 Fractal Cat) — ETB doubles +1/+1
  counters on target friendly creature. Tests:
  `quandrix_mascot_is_a_three_mana_two_two_fractal`,
  `_doubles_counters_on_target`.
- **Witherbloom Mascot** ({1}{B}{G}, 2/2 Pest Beast) — Dies: drain 2
  from each opp. Tests: `witherbloom_mascot_is_a_three_mana_pest`,
  `_dies_drains_two`.
- **Lorehold Mascot** ({2}{R}{W}, 3/2 Spirit) — Attacks: gain 1 life,
  +1/+0 EOT. Tests: `lorehold_mascot_is_a_four_mana_three_two_spirit`,
  `_attack_gains_life_and_pumps`.

**NEW STA reprint (1):**
- **Step Through** ({U} Sorcery) — Tutor an instant or sorcery from
  library. Tests: `step_through_is_a_one_mana_blue_sorcery`,
  `_tutors_instant_or_sorcery_from_library`.

**CR audit coverage:**
- **CR 701.25c — Surveil 0 emits no surveil event** — code was already
  correct via the shared Scry/Surveil 0-amount short-circuit; pushed a
  test (`zero_surveil_does_not_trigger_surveil_events_per_cr_701_25c`)
  to lock in the rule coverage.

Push (modern_decks, **prior sub-push**): **10 NEW cards** (3 STX 2021 + 7 STA reprints) +
engine improvements:

**NEW STX 2021 cards (3):**
- **Triskaidekaphile** ({1}{U}{U}, 3/4 Human Wizard) — ETB Draw + flip
  no-max-hand-size + upkeep "you win the game" trigger gated on
  `ValueEquals(HandSizeOf(You), 13)` (CR 603.4 intervening 'if'
  clause). Tests: `triskaidekaphile_is_a_three_mana_three_four_human_wizard`,
  `_etb_draws_a_card_and_lifts_max_hand_size`,
  `_wins_at_upkeep_with_exactly_thirteen_cards`,
  `_does_not_win_at_upkeep_with_other_hand_size`.
- **Excellent Education** ({2}{W} sorcery) — Target player gains 4 life
  and draws a card. Tests:
  `excellent_education_gives_target_player_life_and_draw`,
  `_can_target_opponent`, `_is_a_three_mana_white_sorcery`.
- **Sproutback Trudge** ({3}{G}{G}, 5/6 Plant) — ETB gain life equal to
  creature cards in your graveyard. Reads
  `Value::CountOf(CardsInZone(You, Graveyard, Creature))`. Tests:
  `sproutback_trudge_is_a_five_mana_five_six_plant`,
  `_gains_life_per_creature_in_graveyard`,
  `_with_empty_graveyard_gains_zero_life`.

**NEW STA reprints (7):**
- **Wonder** ({3}{U}, 2/2 Incarnation, Flying) — STA-cycle Incarnation,
  Island gy-anthem grants Flying. Three tests.
- **Brawn** ({2}{G}, 3/3 Incarnation, Trample) — STA-cycle Incarnation,
  Forest gy-anthem grants Trample. Three tests.
- **Valor** ({1}{W}, 2/2 Incarnation, First Strike) — STA-cycle
  Incarnation, Plains gy-anthem grants First Strike. Three tests.
- **Deep Analysis** ({3}{U} sorcery, Flashback {1}{U}) — Target player
  draws 2 + loses 2 life. Three tests.
- **Tribute to Hunger** ({2}{B} instant) — Target opp sacrifices a
  creature; you gain life equal to its toughness. Three tests, lands the
  new `Value::SacrificedToughness` primitive (sibling of
  `SacrificedPower`).
- **Kasmina's Transmutation** ({1}{U}{U} sorcery) — Target creature
  becomes 1/1 EOT via `Effect::SetBasePT` (loses-all-abilities rider is
  engine-wide ⏳, same as Mercurial Transformation). Two tests.
- **Crippling Fear** ({3}{B} sorcery) — All creatures get -3/-3 EOT
  (the choose-creature-type rider is engine-wide ⏳; the
  approximation is the strictly-stronger universal -3/-3). Three tests.

**NEW engine primitives + bug fixes:**
- **`Value::SacrificedToughness` + `GameState.sacrificed_toughness`** —
  per-resolution scratch field stamped by `Effect::SacrificeAndRemember`
  alongside `sacrificed_power`. Powers Tribute to Hunger.
- **CR 603.2 bug fix** — `fire_step_triggers` (`game/stack.rs`) now
  honors `EventSpec.filter` predicates. Previously, step-begin
  triggers (Pact-style "if it's your turn", Triskaidekaphile's "if you
  have exactly 13 cards", Felidar Sovereign's "if you have ≥40 life")
  fired unconditionally — only the trigger's `kind` + `scope` were
  checked. Now the filter predicate is re-evaluated against the
  current game state before the trigger is pushed onto the stack
  (CR 603.4 intervening-'if' clause, half-implemented — the "check
  again at resolve time" half is still ⏳).
- **`graveyard_anthem_for_name` helper-table extension** — added
  Wonder (Island → Flying), Brawn (Forest → Trample), Valor (Plains
  → First Strike) alongside the existing Anger (Mountain → Haste).
  All four STA-cycle Incarnations share one helper-table row each.

Prior sub-push (still on modern_decks): **1 NEW card** (Anger, STA
reprint) **+ 6 promotions**
(Conciliator's Duelist ✅ via DelayUntil + CastSpellTarget fallback,
Light of Promise ✅ via the new `Value::TriggerEventAmount` primitive,
Thornfist Striker ✅ via the new `lifegain_anthem_for_name` helper,
Mind Roots ✅ via the new `Selector::DiscardedThisResolution` primitive,
Scolding Administrator ✅ + Fix What's Broken ✅ via doc-sync — both
already wired) + **5 new engine primitives**:
- **`Effect::DelayUntil` fallback to `Selector::CastSpellTarget(0)`** —
  when the trigger context has no `ctx.targets`, the DelayUntil
  capture walks the stack for the just-cast spell's slot-0 target.
  Wires Conciliator's Duelist's "return at next end step" Repartee
  rider.
- **`Value::TriggerEventAmount` + `EffectContext.event_amount` +
  `StackItem::Trigger.event_amount`** — per-event amount (life
  gained, life lost, damage dealt, …) threaded through the trigger
  dispatcher to the resolving trigger's body. Wires Light of
  Promise's "that many" rider.
- **`lifegain_anthem_for_name` helper table + compute-time
  injection** in `GameState::compute_battlefield` (sibling of
  `lifegain_selfpump_for_name`). Wires Thornfist Striker's Infusion
  "creatures you control get +1/+0 and have trample" anthem.
- **`graveyard_anthem_for_name` helper table + compute-time gy
  walk** — first entry: Anger (Mountain → Haste). Adds a per-
  graveyard pass that emits a continuous `AddKeyword` effect when
  the gy-resident card's owner controls the required land subtype.
- **`Selector::DiscardedThisResolution { filter }` +
  `GameState.discarded_card_ids_this_resolution`** — tracks
  per-resolution discarded card ids and exposes them as a Selector
  for follow-up moves. Wires Mind Roots's "land discarded this way →
  battlefield tapped".
- **`PlayerRef::You` flatten in `resolve_zonedest_player`** — fixes
  a bug where a `You`-anchored ZoneDest on a gy-to-bf move rebound
  to the gy owner's seat (so Mind Roots's stolen land was landing
  back under the opp's control). Now flattens to `Seat(ctx.controller)`
  before `place_card_in_dest` runs.

Earlier sub-push (still on modern_decks): **6 NEW cards** (1 STX 2021
+ 5 STA reprints) **+ 4 promotions** + **2 prior engine primitives**:
- **`Effect::PreventAllCombatDamageThisTurn` + `GameState
  .prevent_combat_damage_this_turn` flag** (CR 615.1) — combat damage
  resolver consults the flag and zeroes attacker/blocker damage
  (lifelink scales off actual damage dealt, so lifelink life-gain is
  zeroed too). Cleared in `do_cleanup` alongside other
  until-end-of-turn state. Wires Owlin Shieldmage's ETB.
- **`CardDefinition.exile_on_resolve: bool`** (CR 701.x) — instants /
  sorceries with the printed "Then exile this spell" rider now route
  to exile after resolution instead of their owner's graveyard. Bumps
  `Player.cards_exiled_this_turn` so Ennis-style payoffs see the
  exile event. Wires Awaken the Ages (Strife Scholar back-face),
  Divergent Equation, Wisdom of Ages.

**5 new STA reprints** added in `catalog::sets::stx::extras`:
- **Damnable Pact** — {X}{B}{B} Sorcery. Target player draws X cards
  and loses X life. Both clauses read `Value::XFromCost`.
- **Shore Up** — {U} Instant. Untap target permanent + Hexproof EOT.
  Flashback {3}{U}.
- **Symbol of Strength** — {2}{G} Sorcery. +2/+2 EOT + draw 1.
  Flashback {3}{G}.
- **Magmatic Sinkhole** — {1}{B}{R} Sorcery. Surveil 2 + 4 damage
  to a creature or planeswalker.
- **Sevinne's Reclamation** — {2}{W} Sorcery. Reanimate ≤3-MV
  permanent card from your graveyard. If cast from a graveyard, copy
  twice (via the `Predicate::CastFromGraveyard` rider). Flashback {5}{W}.

**1 new STX 2021 card**:
- **Light of Promise** (🟡) — {3}{W} Enchantment. "Whenever you gain
  life, put that many +1/+1 counters on target creature you control."
  Per-fire trigger approximation (engine has no per-event amount value
  yet, so each lifegain event lands 1 +1/+1 counter rather than "that
  many"). Body matches printed Oracle for the common 1-life-per-gain
  case (Pest-style drains, incidental lifegain).

Prior push (modern_decks, claude/modern_decks branch — earlier revision):
**12 NEW cards** (7 STX 2021 + 4 STA reprints + 1 STX
Mastery cycle) **+ 2 SOS promotions** (Burrog Barrage ✅, Chelonian
Tackle ✅ via slot-1 multi-target promotion). All new cards ship with
at least one functionality test in `tests::stx`.

**SOS promotions (2):**

1. **Burrog Barrage** 🟡 → ✅ — Doc-sync: the slot-1 multi-target
   promotion (`Selector::TargetFiltered { slot: 1 }`) for the opp-
   creature defender shipped earlier (push modern_decks). Existing
   tests cover both slot 0-only and slots 0+1.
2. **Chelonian Tackle** 🟡 → ✅ — Promoted `Effect::Fight`'s defender
   from auto-pick `EachPermanent(Opp creature)` to slot-1
   `Selector::TargetFiltered { slot: 1, filter: Creature & ControlledBy
   Opponent }`. The cast path's `auto_targets_for_effect_all_slots`
   fills slot 1 when an opp creature exists. Tests:
   `chelonian_tackle_pumps_toughness` (slot 0 only — fight no-ops),
   `chelonian_tackle_fights_opp_creature` (both slots → opp creature
   dies).

**New cards added in this push (10):**

1. **Forked Bolt** ✅ NEW (STA reprint, Saviors of Kamigawa) — {R}
   Sorcery, 2 damage to a creature/player/PW (single-target collapse of
   the "divided among one or two" rider). Tests:
   `forked_bolt_deals_two_damage_to_creature`,
   `forked_bolt_targets_player_for_two_damage`.
2. **Storm's Wrath** ✅ NEW (STX 2021) — {2}{R}{R} Sorcery, 4 damage to
   each creature and each planeswalker via `ForEach(Creature ∨
   Planeswalker) → DealDamage 4`. Tests:
   `storms_wrath_destroys_each_creature`,
   `storms_wrath_is_a_four_mana_red_sorcery`.
3. **Cinderclasm** ✅ NEW (STX 2021) — {1}{R}{R} Sorcery, 1 damage to
   each creature and each planeswalker (unkicked half only — Kicker
   {R} alt-cost is engine-wide ⏳). Test:
   `cinderclasm_pings_each_creature_for_one`.
4. **Cathartic Pyre** ✅ NEW (STX 2021) — {1}{R} Sorcery, two-mode
   `ChooseMode`: (0) 3 damage to creature; (1) Discard up to 2 cards,
   then draw that many cards via `DiscardAnyNumber +
   Value::CardsDiscardedThisEffect`. Test:
   `cathartic_pyre_default_mode_burns_creature`.
5. **Stern Dismissal** ✅ NEW (STX 2021) — {U} Instant, return target
   creature or enchantment to its owner's hand. Test:
   `stern_dismissal_bounces_creature_to_owner_hand`.
6. **Krosan Grip** ✅ NEW (STA reprint, Time Spiral) — {2}{G} Instant,
   destroy target artifact or enchantment. Split second is engine-wide
   ⏳. Test: `krosan_grip_destroys_artifact`.
7. **Sublime Epiphany** ✅ NEW (STA reprint, Core Set 2021) — {4}{U}{U}
   Instant, multi-modal `ChooseN { picks: [2, 4], modes }`: auto-picks
   bounce nonland permanent + draw a card. Tests:
   `sublime_epiphany_resolves_counter_bounce_draw`,
   `sublime_epiphany_is_a_six_mana_blue_instant`.
8. **Persist** ✅ NEW (STA reprint, Shadowmoor) — {1}{B}{G} Sorcery,
   return target nonlegendary creature card from your graveyard to the
   battlefield with a -1/-1 counter on it. Wired as `Seq(Move →
   Battlefield, AddCounter(-1/-1, 1))` with `Not(HasSupertype(Legendary))`
   filter. Test: `persist_returns_creature_card_with_minus_one_counter`.
9. **Bone to Ash** ✅ NEW (STX 2021) — {1}{U}{U} Instant, counter target
   creature spell + draw a card. Test:
   `bone_to_ash_counters_creature_spell_and_cantrips`.
10. **Ingenious Mastery** ✅ NEW (STX 2021, Mastery cycle) — {3}{U}{U}
    Sorcery, Draw 3 + put 2 from hand on top + an opponent draws a
    card. The {1}{U}{U} alt-cost-implies-mode is engine-wide ⏳ (same
    as Baleful / Devastating / Verdant Mastery). Test:
    `ingenious_mastery_draws_three_stacks_two_and_opp_draws`.
11. **Acolyte of Affliction** ✅ NEW (STX 2021) — {3}{B}{B} 4/3 Zombie
    Cleric. ETB: each player mills three; return up to one target
    permanent card from any graveyard to its owner's hand. Tests:
    `acolyte_of_affliction_mills_each_player_three`,
    `acolyte_of_affliction_is_a_five_mana_zombie_cleric`.
12. **Skywarp Skaab** ✅ NEW (STX 2021) — {1}{U}{U} 2/3 Zombie Wizard
    with Flying. ETB: optional discard 1 + bounce target creature (via
    `Effect::MayDo`). Tests:
    `skywarp_skaab_is_a_three_mana_flying_zombie_wizard`,
    `skywarp_skaab_etb_declines_by_default`.

Prior push (modern_decks, claude/modern_decks branch — earlier sub-push):
Added 8 NEW STX cards + 2 SOS promotions (Transcendent Archaic ✅,
Decorum Dissertation ✅). All new cards ship with at least one
functionality test in `tests::stx` and `tests::sos`.

**Promotions (2):**

1. **Transcendent Archaic** 🟡 → ✅ — Wrapped the ETB Converge draw +
   conditional discard 2 in `Effect::MayDo` so the printed "you may
   draw" optionality is honored. Tests:
   `transcendent_archaic_etb_may_draw_declines_by_default`,
   `transcendent_archaic_etb_may_draw_accepts_via_scripted_decider`.

2. **Decorum Dissertation** 🟡 → ✅ — Target-player prompt promoted to
   `target_filtered(Player)` so the printed "target player draws 2
   loses 2" trade can target self (matching the typical asymmetric
   trade) or an opp (drain 2 in exchange for letting them draw 2).
   Tests: `decorum_dissertation_draws_two_loses_two`,
   `decorum_dissertation_can_target_opponent`.

**New cards (8 — 5 STA reprints, 3 STX 2021):**

1. **Mizzium Mortars** ✅ NEW (STA reprint, RTR) — {1}{R}
   Sorcery, 4 damage to target creature. Overload alt cost is engine-
   wide ⏳ (no Overload primitive).
2. **Electrolyze** ✅ NEW (STA reprint, Guildpact) — {1}{U}{R} Instant,
   2 damage to a single target + draw a card. The "divided as you
   choose among one or two targets" rider collapses to single-target.
3. **Show of Aggression** ✅ NEW (STX 2021) — {2}{R}{R} Sorcery,
   creatures you control get +2/+0 and gain haste until end of turn.
   Wired via `Effect::ForEach`.
4. **Past in Flames** ✅ NEW (STA reprint, Innistrad) — {3}{R} Sorcery,
   approximated as a mass `Move(all IS cards in your gy → hand)`
   since the engine has no transient per-card Flashback grant.
   Flashback {4}{R} on Past in Flames itself is honored.
5. **Inspired Idea** ✅ NEW (STX 2021 / M11 STA flavor) — {1}{U}{U}
   Sorcery, draw 3 then put 2 from hand on top of library. Classic
   blue dig-and-stack.
6. **Resurgent Belief** ✅ NEW (STX 2021) — {3}{W} Sorcery, return
   all enchantment cards from your graveyard to the battlefield.
   Flashback {4}{W} (alt-cost gy-exile rider omitted).
7. **Academic Dispute** ✅ NEW (STX 2021) — {R} Instant, target
   creature you control gets +1/+0 and fights target opp creature.
   Learn → Draw 1 approximation.
8. **Enthusiastic Study** ✅ NEW (STX 2021) — {1}{G} Instant, +2/+2
   EOT and trample if you've cast another spell this turn.

(Earlier prior revisions detailed below.)

### Prior push: 10 new STX cards + Predicate::CastFromGraveyard


**New cards (10, all `stx::extras`):**

1. **Spined Karok** — {2}{G}{U} 3/3 Reach Beast. ETB +1/+1 counter
   on target friendly creature. Tests: `spined_karok_etb_lands_counter_
   on_friendly`, `spined_karok_is_a_four_mana_three_three_with_reach`.
2. **Inspiring Veteran** — {1}{W} 2/2 Human Knight. Static "Other
   creatures you control get +1/+1" (Hofri-style anthem using the
   `OtherThanSource` flag). Tests:
   `inspiring_veteran_buffs_other_friendly_creatures`,
   `inspiring_veteran_does_not_buff_opp_creatures`,
   `inspiring_veteran_anthem_expires_when_it_leaves_play`.
3. **Snipe** — {U}{R} Instant. 2 damage to creature + draw a card if
   you've cast another instant/sorcery spell this turn. Gated on
   `SpellsCastThisTurnAtLeast(2)`. Tests:
   `snipe_deals_two_to_creature_without_cantrip` (first spell — no
   cantrip), `snipe_cantrips_on_second_spell_cast` (second spell —
   cantrip fires).
4. **Witherbloom Pest Eater** — {3}{B}{G} 4/4 Pest. ETB mints a Pest
   token; pumps +1/+1 EOT whenever another Pest dies. Tests:
   `witherbloom_pest_eater_etb_creates_pest_token`,
   `witherbloom_pest_eater_grows_when_another_pest_dies`.
5. **Inkmoth Initiate** — {W}{B} 2/2 Flying Human Cleric. ETB -1/-1
   EOT on target creature. Tests:
   `inkmoth_initiate_etb_shrinks_target_creature`,
   `inkmoth_initiate_is_a_two_mana_flying_human_cleric`.
6. **Stoic Tutelage** — {3}{W} Sorcery. Draw 2 cards, each opponent
   loses 1 life. Test: `stoic_tutelage_draws_two_and_drains_each_opp`.
7. **Lorehold Recovery** — {2}{R}{W} Sorcery. Reanimate creature
   card from your gy with Haste EOT. Test:
   `lorehold_recovery_reanimates_with_haste`.
8. **Quandrix Surge** — {1}{G}{U} Sorcery. Double the +1/+1 counters
   on each creature you control (`ForEach + AddCounter(amount =
   CountersOn(TriggerSource))`). Tests:
   `quandrix_surge_doubles_each_creatures_counters`,
   `quandrix_surge_noop_on_counterless_creatures`.
9. **Magecraft Insight** — {2}{U} Instant. Draw 2 cards. Test:
   `magecraft_insight_draws_two_cards`.
10. **Sparkmage's Mantra** — {R} Instant. 1 damage to any target,
    scry 1. Tests: `sparkmages_mantra_pings_and_scrys`,
    `sparkmages_mantra_can_target_player`.

**Bonus card (11th):**

11. **Witherbloom Drainage** — {1}{B}{G} Sorcery. Each opp loses 2
    life, you gain 2 life (via `Effect::Drain`). Test:
    `witherbloom_drainage_drains_each_opp_two`.

**Engine primitive: `Predicate::CastFromGraveyard`** — Reads
`EffectContext.cast_from_hand` (new field, stamped at spell-resolution
time from the resolving `CardInstance.cast_from_hand` flag). Powers
Increasing Vengeance's "if cast from graveyard, copy that spell twice
instead" rider — the printed Oracle now ships exactly: hand cast → 1
copy, flashback (or any cast-from-gy) cast → 2 copies. Same primitive
unblocks Antiquities on the Loose's "cast from anywhere other than
your hand" rider (still 🟡 pending the second-half token-counter
trigger). New test:
`increasing_vengeance_double_copies_when_flashed_back_from_graveyard`
(synthesizes a Flashback {R}{R} cost on Increasing Vengeance and
casts it from graveyard — verifies two copies and exile-on-resolve
per CR 702.34a). CR 707.10c rule audit entry added to TODO.md.

Prior push (modern_decks, claude/modern_decks branch — earlier sub-push):
Added 8 new STX/STA cards + 2 promotions (Comforting Counsel via a
new engine primitive — self-counter-gated controller-wide anthem at
compute time; Living History via doc-sync since the on-attack +2/+0
trigger was already wired faithfully).

**New cards (8 — 3 STX, 5 STA reprints):**

1. **Eladamri's Call** ✅ NEW (STA reprint, Planeshift) — {W}{G}
   Instant. "Search your library for a creature card, reveal it,
   put it into your hand, then shuffle." Wired via
   `Effect::Search { filter: Creature, to: Hand(You) }`. Tests:
   `eladamris_call_tutors_creature_into_hand`,
   `eladamris_call_is_a_two_mana_wg_instant`.
2. **Yawning Fissure** ✅ NEW (STA reprint, Mercadian Masques) —
   {3}{R} Sorcery. "Each opponent sacrifices a land." Wired via
   `ForEach(EachOpponent) → Sacrifice(1, Land)` so each iterated
   opponent picks one of their own lands (the Pox Plague
   per-player-sac pattern). Tests:
   `yawning_fissure_each_opp_sacs_a_land`,
   `yawning_fissure_is_a_four_mana_red_sorcery`.
3. **Cleansing Wildfire** ✅ NEW (STA reprint, Zendikar Rising) —
   {1}{R} Sorcery. "Destroy target land. Its controller may search
   their library for a basic land card, put it onto the battlefield,
   then shuffle. Draw a card." Wired as `Seq(Destroy → Search via
   ControllerOf(Target) → Draw 1)`. Tests:
   `cleansing_wildfire_destroys_land_and_draws`,
   `cleansing_wildfire_is_a_two_mana_red_sorcery`.
4. **Tendrils of Agony** ✅ NEW (STA reprint, Scourge) — {2}{B}{B}
   Sorcery. "Target opponent loses 2 life and you gain 2 life. Storm."
   Storm wired via `Effect::Repeat { count: StormCount + 1, body:
   Drain 2 from EachOpponent → You }`. The drain payload fires
   once per other-spell-cast-this-turn plus the original spell;
   at StormCount=4 (Tendrils is the 5th spell of the turn), drain
   fires 5 × 2 = 10 life. Tests:
   `tendrils_of_agony_drains_two_with_no_storm`,
   `tendrils_of_agony_storm_drain_scales`.
5. **Quench** ✅ NEW (STX uncommon) — {1}{U} Instant. "Counter
   target spell unless its controller pays {1}." Wired via
   `Effect::CounterUnlessPaid { mana_cost: {1} }`. Tests:
   `quench_counters_spell_when_opp_cant_pay`,
   `quench_is_a_two_mana_blue_instant`.
6. **Saw It Coming** ✅ NEW (STA reprint, Kaldheim) — {2}{U} Instant.
   "Counter target spell. Foretell {1}{U}." Wired as a vanilla
   `Effect::CounterSpell` at the {2}{U} regular cost; Foretell
   discount is engine-wide ⏳ (no Foretell alt-cost primitive yet,
   would need a turn-delayed alt-cost discount). Tests:
   `saw_it_coming_counters_target_spell`,
   `saw_it_coming_is_a_three_mana_blue_instant`.
7. **Dueling Coach** ✅ NEW (STX uncommon) — {1}{W} 1/2 Human Cleric.
   "When this enters, put a +1/+1 counter on target creature you
   control. / {2}{W}: Put a +1/+1 counter on each creature you
   control with a +1/+1 counter on it." Counter-snowball synergy
   wired via ETB AddCounter + activation that uses `ForEach
   (EachPermanent(Creature & ControlledByYou & WithCounter
   (+1/+1)))` → AddCounter(TriggerSource). Tests:
   `dueling_coach_etb_lands_counter_on_friendly`,
   `dueling_coach_activation_doubles_counters`,
   `dueling_coach_is_a_two_mana_human_cleric`.
8. **Increasing Vengeance** ✅ NEW (STA reprint, Innistrad) — {R}{R}
   Instant. "Copy target instant or sorcery spell you control."
   Wired via `Effect::CopySpell` (single copy). The "cast from
   graveyard → two copies instead" rider is engine-wide ⏳ (no
   cast-from-graveyard introspection at resolve time). Tests:
   `increasing_vengeance_copies_target_instant`,
   `increasing_vengeance_is_a_two_mana_red_instant`.

**Promotions (2):**

9. **Comforting Counsel** 🟡 → ✅ — printed static "As long as there
   are five or more growth counters on this enchantment, creatures
   you control get +3/+3" is **now wired** via a compute-time
   conditional injection in `GameState::compute_battlefield` (same
   pattern as Honor Troll, Ulna Alley Shopkeep, Cruel Somnophage).
   The gate reads `card.counters[Growth] >= 5`; when true, layer 7b
   pumps every creature controlled by the enchantment's controller
   by +3/+3 via `AffectedPermanents::All { controller, card_types:
   [Creature], exclude_source: false }`. The growth-counter accrual
   (LifeGained-event trigger) was already wired in the prior push.
   Tests:
   `comforting_counsel_no_anthem_below_five_counters`,
   `comforting_counsel_anthem_buffs_friendly_creatures_at_five_counters`,
   `comforting_counsel_accrues_growth_on_lifegain` (existing).

10. **Living History** 🟡 → ✅ — doc-sync. The on-attack +2/+0
    EOT trigger (gated on `Predicate::CardsLeftGraveyardThisTurnAtLeast`)
    has been wired faithfully since the per-attacker auto-target
    framework landed; the "target attacking creature" wording lands
    the pump on the iterated attacker via `Selector::TriggerSource`
    (same shape as Sparring Regimen's per-attacker counter rider).
    Existing test: `living_history_etb_creates_spirit_token`.

**Engine improvements:**

- **Self-counter-gated controller-wide anthem** — Comforting Counsel
  is the canonical instance of "X creatures you control get +N/+N as
  long as this permanent has ≥ K [counter] counters". The compute-
  time injection in `GameState::compute_battlefield` (`game/mod.rs`)
  follows the existing Honor Troll / Cruel Somnophage / Tarmogoyf
  pattern: per-source name-keyed lookup → gate evaluation on the
  source's counter pool → emit one `ContinuousEffect` per layer
  recompute. The gate re-evaluates every recompute, so a mid-turn
  fifth growth counter flips the anthem on immediately, and counter
  removal flips it back off.

(Earlier prior revisions detailed below.)

### Prior push: 4 new STX/STA cards + 1 promotion + 3 engine primitives

**Prior new cards (4 — 1 STX, 3 STA reprints):**

1. **Maelstrom Muse** ✅ NEW (STX uncommon) — {3}{U}{R} 3/3 Djinn
   Wizard with Flying. Opus magecraft loot — `shortcut::opus_trigger`
   wires draw-1-discard-1 on small spells, draw-2-discard-1 on
   spells with 5+ mana spent. Test:
   `maelstrom_muse_opus_loots_on_small_cast`.
2. **Approach of the Second Sun** ✅ NEW (STA reprint, Amonkhet) —
   {6}{W}{W} Sorcery. First cast gains 7 life; on a second cast with
   one copy already in graveyard, the new `Effect::WinGame` primitive
   eliminates every other player so the SBA pass promotes the controller
   to game-winner. Uses `Predicate::SameNamedInZoneAtLeast` (CR
   gy-name predicate) to detect the second cast. Tests:
   `approach_of_the_second_sun_gains_seven_life_on_first_cast`,
   `approach_of_the_second_sun_wins_game_when_cast_with_one_in_graveyard`.
3. **Resurrection** ✅ NEW (STA reprint, Alpha) — {2}{W}{W} Sorcery.
   "Return target creature card from your graveyard to the battlefield."
   Test: `resurrection_returns_creature_card_from_graveyard`.
4. **Adventurous Impulse** ✅ NEW (STA reprint, Core 2021) — {G}
   Sorcery. "Look at top three, may reveal a creature or land card,
   rest go to bottom in random order." Wired via
   `Effect::RevealUntilFind { miss_dest: BottomRandom }`. Test:
   `adventurous_impulse_finds_a_creature_in_top_three`.

**Promotion (1):**

5. **Plargg, Dean of Chaos** 🟡 → ✅ — printed conditional damage
   rider ("if a creature card was discarded → 2 damage to any target")
   wired via the new `Value::CreatureCardsDiscardedThisEffect`
   primitive. The activation now requires a target slot for the damage
   (Effect::DealDamage with target_filtered(Creature ∨ Player ∨
   Planeswalker)); the conditional `Effect::If` gates on the new
   value reading ≥ 1. Existing test (`plargg_dean_of_chaos_taps_to_loot`)
   stays green; two new tests cover the new branches:
   `plargg_dean_of_chaos_deals_two_damage_when_creature_discarded`,
   `plargg_dean_of_chaos_no_damage_when_noncreature_discarded`.

**Engine improvements (3 new primitives):**

- **`Value::CreatureCardsDiscardedThisEffect`** + new
  `GameState.creature_cards_discarded_this_resolution: u32` scratch
  counter. Bumped by both discard-branch handlers (random `Discard`
  + the `DiscardChosenPending` `apply_pending_effect_answer` path)
  whenever the just-discarded card carries `CardType::Creature`.
  Reset at the top of `resolve_effect` alongside the existing
  `cards_discarded_this_resolution`. Used by Plargg's conditional
  damage rider — and unblocks any future "if a creature card was
  discarded" / "if you discarded a creature card this turn"
  payoff.

- **`Effect::WinGame { who: PlayerRef }`** (CR 104.2a — "you win
  the game"). Resolves `who` to a single player and marks every
  other player `eliminated = true`. The state-based-action sweep
  (`check_state_based_actions` in `game/stack.rs`) then promotes
  `game_over = Some(winner)` on the next loop. Same primitive
  unblocks Coalition Victory, Test of Endurance, Felidar
  Sovereign, Mortal Combat — any "you win the game" wording.

- **`GameState::auto_targets_for_effect_all_slots`** — bot-side
  multi-slot target picker. Walks every `Selector::TargetFiltered
  { slot }` index in the effect tree (via the existing
  `target_filter_for_slot_in_mode` helper) and returns
  `(Option<Target>, Vec<Target>)` — slot 0 plus an
  `additional_targets` vec for slots 1+. Wired into the bot
  harness in `server/bot.rs` so multi-target casts (Snow Day,
  Homesickness, Cost of Brilliance, Render Speechless, Vibrant
  Outburst, Dissection Practice, Cost of Brilliance) now drive
  the multi-target shape end-to-end in bot games without manual
  intervention. Cap of 16 slots (no real card uses more than 4).
  Test: `auto_target_picker_fills_multi_slot_vibrant_outburst`.

(Earlier prior revisions detailed below.)

Prior push (modern_decks):
Promoted 10 SOS 🟡 → ✅ via two new engine primitives (`Effect::
DiscardAnyNumber` + `Effect::SetNoMaxHandSize` + `Player.
no_maximum_hand_size`) and a multi-target promotion pass. The
promoted cards are:

1. **Colossus of the Blood Age** — death trigger now uses the new
   `Effect::DiscardAnyNumber` (player picks 0..hand-size) +
   `Value::CardsDiscardedThisEffect + 1` for the draw.
2. **Wisdom of Ages** — "no maximum hand size for the rest of the
   game" now wired via the new `Effect::SetNoMaxHandSize` primitive
   + `Player.no_maximum_hand_size` flag respected by `do_cleanup`'s
   CR 514.1 enforcement.
3. **Cost of Brilliance** — multi-target (slot 0 player + slot 1
   creature).
4. **Dissection Practice** — multi-target (slot 0 player + slot 1
   pump + slot 2 shrink).
5. **Vibrant Outburst** — multi-target (slot 0 burn + slot 1 tap).
6. **Homesickness** — multi-target (slot 0 player + slots 1+2
   creature tap+stun).
7. **Together as One** — multi-target (slot 0 player + slot 1
   any-target damage).
8. **Rabid Attack** — multi-target (slots 0+1+2 = three friendly
   creatures pumped).
9. **Render Speechless** — multi-target (slot 0 opp + slot 1
   creature counter).
10. **Borrowed Knowledge** — doc-sync (mode 1 already wired via
    `Value::CardsDiscardedThisEffect`).

11 new tests cover the promotions: each promoted card has at least
one scripted-decider or both-slots-filled test exercising the
multi-target path. CR 402.2 audit entry added to TODO.md's
MagicCompRules coverage list.

Push (modern_decks, prior revision):
Added 10 new cards across the Strixhaven environment — 8 new STX
originals/uncommons (Eureka Moment ✅, Teach by Example ✅, Manifold
Key ✅, Leyline Invocation ✅, Spitfire Lagac ✅, Settle the Score
✅, Pursuit of Knowledge ✅, Divide by Zero ✅) and 2 STA Mystical
Archive reprints (Exsanguinate ✅, Fire Prophecy ✅). All ship with
functional tests in `tests::stx`. The STX corpus grows from 160 to
167 cards (151 ✅ + 16 🟡); STA reprints in STX boosters grows from
14 to 16. No new ⏳; the SOS Improvisation Capstone remains the only
⏳ in the catalog (cast-from-exile pipeline gap).

Push (modern_decks, prior revision): Added 10 new cards across the
Strixhaven environment — 4 STA Mystical Archive reprints (Eliminate ✅,
Burst Lightning ✅, Pull from Tomorrow ✅, Postmortem Lunge ✅) and 5 new
STX-supplemental originals (Channeled Force ✅, Stonebound Mentor ✅,
Inscription of Insight ✅, Curious Cryomancer ✅, Verdant Pledgemage ✅).
Memory Lapse promoted via the new `Effect::CounterSpellToZone {
OwnerLibraryTop }` primitive (CR 608.2c / 701.6a — printed "instead"
clause overrides the default counter-to-graveyard zone). New engine
piece: `CounteredSpellZone` enum with library-top / library-bottom /
hand / exile variants — opens the road to Spell Crumple, Remand, and
Hinder.

Push (modern_decks): Promoted 2 SOS 🟡 → ✅ (Ajani's Response, Brush Off)
via the new alt-cost-with-target-filter wiring; Run Behind's cost half
✅ but the top/bottom-of-library rider stays 🟡. Added 7 new STX cards
(Expanded Anatomy ✅, Selfless Glyphweaver ✅, Crux of Fate ✅,
Mercurial Transformation 🟡, Plargg/Dean of Chaos 🟡, Pestilent
Cauldron 🟡, Augusta/Dean of Order 🟡) — bringing the STX corpus to
151 cards (135 ✅ + 16 🟡).

Push (modern_decks, this revision): Added 9 new cards — 2 STX Lorehold
(Reconstruct History ✅, Lorehold Excavation ✅) and 7 STA reprints
that ship in STX boosters (Sky Diamond ✅, Marble Diamond ✅, Fire
Diamond ✅, Charcoal Diamond ✅, Moss Diamond ✅, Goblin Lore ✅,
Whirlwind Denial ✅). Promoted Molten Note 🟡 → ✅ via the
`Value::CastSpellManaSpent` primitive that reads actual mana paid for
the spell (replacing the prior `Value::XFromCost` approximation —
matches the printed "amount of mana spent to cast this spell"
Oracle exactly). STX corpus now at 160 cards (144 ✅ + 16 🟡); SOS
gains 1 ✅ via Molten Note promotion (162 ✅ + 92 🟡 + 1 ⏳).

The single SOS ⏳ is **Improvisation Capstone** (needs the cast-from-exile
pipeline + copy-spell primitive). Per-card status and the specific gap on
each 🟡 row are in the tables below.

## SOS color breakdown

| Color | Cards |
|---|---|
| White | 32 |
| Blue | 32 |
| Black | 33 |
| Red | 30 |
| Green | 32 |
| Prismari (Blue-Red) | 17 |
| Witherbloom (Black-Green) | 17 |
| Silverquill (White-Black) | 16 |
| Quandrix (Green-Blue) | 16 |
| Lorehold (Red-White) | 16 |
| Colorless | 14 |

## Known engine gaps surfaced by these catalogs

- **Prepare mechanic** (SOS colorless) — Biblioplex Tomekeeper and
  Skycoach Waypoint care about a `prepared` permanent flag. Needs a
  `CounterType::Prepared` (or `PermanentFlag::Prepared`),
  `Effect::SetPrepared`, and `Predicate::IsPrepared`. All Prepare cards
  are 🟡 / ⏳ until these land.
- **Lessons sideboard** — Eyetwitch, Pest Summoning, Hunt for
  Specimens, Field Trip, Igneous Inspiration use Learn. Currently
  approximated as `Draw 1`.
- **Cast-from-graveyard / cast-from-exile pipelines** — block several
  Paradigm cards and the lone SOS ⏳ (Improvisation Capstone).
- **Multi-target prompts on instants/sorceries** — recurring 🟡 reason
  across SOS/STX (Divergent Equation, Vibrant Outburst, Snow Day,
  Devious Cover-Up, Crackle with Power, Magma Opus, …).
- **Ward enforcement (CR 702.21)** — full coverage for spells **and**
  activated abilities, with all four standard cost variants. The
  `Keyword::Ward(WardCost)` enum carries `Mana(ManaCost) | Life(u32) |
  Discard(u32) | SacrificeCreature`. The new `Effect::CounterUnless
  { what, cost }` resolver walks the stack for either a matching
  `Spell` (by `card.id`) or `Trigger` (by `source`), then auto-pays
  the cost on the affected controller's behalf. `push_ward_triggers_for_cast`
  (post-`finalize_cast`) and `push_ward_triggers_for_activated_ability`
  (post-`activate_ability`-push) both share a `push_ward_triggers_for_targets`
  core. Auto-pay for Discard picks the first hand-card; auto-pay for
  Sacrifice picks the first matching creature. An interactive surface
  should later prompt the controller for both Discard and Sacrifice
  choices, but bot games run end-to-end as-is.

## White

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Ajani's Response | {4}{W} | Instant |  | This spell costs {3} less to cast if it targets a tapped creature. / Destroy target creature. | ✅ (was 🟡) | Push (modern_decks): "{3} less if it targets a tapped creature" rider wired via `AlternativeCost { mana_cost: {1}{W}, target_filter: Some(Creature + Tapped) }`. The destroy-creature body is unchanged. When the caster picks a tapped creature target, alt-cost path is available at {1}{W}; otherwise the spell goes off at the full printed {4}{W}. Tests: `ajanis_response_alt_cost_destroys_tapped_creature`, `ajanis_response_alt_cost_rejects_untapped_target`. |
| Antiquities on the Loose | {1}{W}{W} | Sorcery |  | Create two 2/2 red and white Spirit creature tokens. Then if this spell was cast from anywhere other than your hand, put a +1/+1 counter on each Spirit you control. / Flashback {4}{W}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ (was 🟡) | Push (modern_decks): the "if cast from anywhere other than your hand, +1/+1 counter on each Spirit" rider is **now wired** via the new `Predicate::CastFromGraveyard` primitive (reads `EffectContext.cast_from_hand`, stamped at spell-resolution time from `CardInstance.cast_from_hand`). Wire shape: `Seq(CreateToken(2 Spirits), If(CastFromGraveyard, ForEach(Spirit & ControlledByYou) → AddCounter(+1/+1), Noop))`. Flashback {4}{W}{W} half already wired. Tests: `antiquities_on_the_loose_creates_two_spirit_tokens`, `antiquities_on_the_loose_hand_cast_does_not_fan_counters` (hand cast → no counter rain), `antiquities_on_the_loose_flashback_cast_fans_counters` (flashback cast → +1/+1 on each Spirit + IV exiled per CR 702.34a). |
| Ascendant Dustspeaker | {4}{W} | Creature — Orc Cleric | 3/4 | Flying / When this creature enters, put a +1/+1 counter on another target creature you control. / At the beginning of combat on your turn, exile up to one target card from a graveyard. | ✅ | Wired in `catalog::sets::sos::creatures` with both ETB pump + combat-step exile triggers. |
| Daydream | {W} | Sorcery |  | Exile target creature you control, then return that card to the battlefield under its owner's control with a +1/+1 counter on it. / Flashback {2}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Wired in `catalog::sets::sos::sorceries` as the standard Restoration-Angel-style flicker pattern (`Exile + Move(target → battlefield) + AddCounter`). Flashback {2}{W} now wired via `Keyword::Flashback` (push X) — graveyard replay reuses the engine's existing `cast_flashback` path. The library traversal in `move_card_to` was extended to handle library-source moves so the flicker round-trip resolves end-to-end. |
| Dig Site Inventory | {W} | Sorcery |  | Put a +1/+1 counter on target creature you control. It gains vigilance until end of turn. / Flashback {W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Mainline pump+vigilance wired in `catalog::sets::sos::sorceries`; Flashback {W} clause now wired via `Keyword::Flashback` (push X). |
| Eager Glyphmage | {3}{W} | Creature — Cat Cleric | 3/3 | When this creature enters, create a 1/1 white and black Inkling creature token with flying. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Elite Interceptor // Rejoinder | {W} // {1}{W} | Creature — Human Wizard // Sorcery | 1/2 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Emeritus of Truce // Swords to Plowshares | {1}{W}{W} // {W} | Creature — Cat Cleric // Instant | 3/3 |  | ✅ | Push XXVIII promotion: vanilla 3/3 Cat Cleric front + faithful Swords to Plowshares back (`Exile target creature` + `GainLife(target's power) → controller of target`). The `PlayerRef::ControllerOf` resolves at resolution time so the *target's controller* gets the life, not the caster. Tests: `emeritus_of_truce_front_is_three_three_cat_cleric`, `emeritus_of_truce_back_face_is_swords_to_plowshares`, `emeritus_of_truce_back_exiles_creature_and_grants_life`. |
| Ennis, Debate Moderator | {1}{W} | Legendary Creature — Human Cleric | 1/1 | When Ennis enters, exile up to one other target creature you control. Return that card to the battlefield under its owner's control at the beginning of the next end step. / At the beginning of your end step, if one or more cards were put into exile this turn, put a +1/+1 counter on Ennis. | ✅ (was 🟡) | Push (modern_decks doc-sync): both printed clauses ship faithfully. (a) ETB flicker — `Effect::Seq([Exile(target creature you control), DelayUntil(NextEndStep, Move(target → Battlefield(OwnerOf)))])`, same shape as Restoration Angel. (b) End-step counter — gated on the exact-printed predicate `Predicate::CardsExiledThisTurnAtLeast` (backed by `Player.cards_exiled_this_turn` — bumped from `place_card_in_dest`'s exile branch). Earlier doc notes referenced a `CardsLeftGraveyardThisTurnAtLeast` proxy that's been retired — the code uses the exact-printed predicate since push IX. |
| Erode | {W} | Instant |  | Destroy target creature or planeswalker. Its controller may search their library for a basic land card, put it onto the battlefield tapped, then shuffle. | ✅ | Push XV: now fully wired. Destroy + `Search { who: ControllerOf(Target), filter: IsBasicLand, to: Battlefield(ControllerOf(Target), tapped) }`. The "may" optionality is collapsed to always-search (decline path covered by `Effect::Search`'s decider returning `Search(None)`). |
| Graduation Day | {W} | Enchantment |  | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, put a +1/+1 counter on target creature you control. | ✅ | Wired in `catalog::sets::sos::enchantments` via `repartee()` shortcut + `target_filtered(Creature & ControlledByYou)` AddCounter. |
| Group Project | {1}{W} | Sorcery |  | Create a 2/2 red and white Spirit creature token. / Flashback—Tap three untapped creatures you control. (You may cast this card from your graveyard for its flashback cost. Then exile it.) | 🟡 | Mainline 2/2 R/W Spirit token wired (new `spirit_token()` helper); Flashback "tap three" cost omitted. |
| Harsh Annotation | {1}{W} | Instant |  | Destroy target creature. Its controller creates a 1/1 white and black Inkling creature token with flying. | ✅ | Push XVII: token now goes to the target creature's owner via `PlayerRef::OwnerOf(Target(0))`. `place_card_in_dest` resolves the player against cast-time ctx (the target id stays valid through `find_card_owner`'s zone walk after the destroy step). |
| Honorbound Page // Forum's Favor | {3}{W} // {W} | Creature — Cat Cleric // Sorcery | 3/3 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Informed Inkwright | {1}{W} | Creature — Human Wizard | 2/2 | Vigilance / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, create a 1/1 white and black Inkling creature token with flying. | ✅ | Vigilance body + Repartee Inkling token wired via `repartee()` + `inkling_token()`. |
| Inkshape Demonstrator | {3}{W} | Creature — Elephant Cleric | 3/4 | Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.) / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, this creature gets +1/+0 and gains lifelink until end of turn. | ✅ (was 🟡) | Push (modern_decks): Ward enforcement landed engine-wide (CR 702.21). `push_ward_triggers_for_cast` in `game/actions.rs` runs at the end of `finalize_cast` — walks the spell's slot 0 + additional targets and pushes one `StackItem::Trigger` per Ward(N) opp permanent. Trigger body is `Effect::CounterUnlessPaid { what: Selector::Target(0), mana_cost: {N} }` aimed at the just-cast spell. APNAP-correct push order (Ward goes on top of caster's Magecraft / Prowess triggers). Tests: `ward_counters_opp_spell_when_payer_cannot_afford`, `ward_allows_opp_spell_when_payer_can_afford`, `ward_does_not_trigger_on_caster_own_spell`. Repartee body unchanged. Ward enforcement only fires on spells today; activated-ability targeting (CR 702.21a "spell or ability") is a follow-up. |
| Interjection | {W} | Instant |  | Target creature gets +2/+2 and gains first strike until end of turn. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Joined Researchers // Secret Rendezvous | {1}{W} // {1}{W}{W} | Creature — Human Cleric Wizard // Sorcery | 2/2 |  | ✅ (was 🟡) | Push (modern_decks): vanilla front + back-face Secret Rendezvous now resolves with each-player fan-out via `Selector::Player(PlayerRef::EachPlayer)` so both players draw 3 (printed Oracle exact). Was approximating "each player" as "caster draws 3". Test: `joined_researchers_back_face_each_player_draws_three`. |
| Owlin Historian | {2}{W} | Creature — Bird Cleric | 2/3 | Flying / When this creature enters, surveil 1. (Look at the top card of your library. You may put it into your graveyard.) / Whenever one or more cards leave your graveyard, this creature gets +1/+1 until end of turn. | ✅ | All three abilities wired. The cards-leave-graveyard pump uses the SOS-V `EventKind::CardLeftGraveyard` event (per-card emission; the printed "one or more" wording approximates as per-card). |
| Practiced Offense | {2}{W} | Sorcery |  | Put a +1/+1 counter on each creature target player controls. Target creature gains your choice of double strike or lifelink until end of turn. / Flashback {1}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ (was 🟡) | Push (modern_decks doc-sync): all three printed clauses ship. (a) Fan-out `ForEach(Creature & ControlledByYou) → AddCounter(+1/+1)` — the printed "target player controls" defaults to "you" via the auto-decider; a real client can target the opponent for the symmetric +1/+1 (which is rarely correct play). (b) Modal Double-Strike OR Lifelink — wired via `Effect::ChooseMode([GrantKeyword(DoubleStrike, EOT), GrantKeyword(Lifelink, EOT)])`. AutoDecider picks mode 0 (Double Strike) by default; `ScriptedDecider::new([DecisionAnswer::Mode(1)])` selects Lifelink. (c) Flashback {1}{W} via `Keyword::Flashback` — graveyard replay reuses the engine's `cast_flashback` path. |
| Primary Research | {4}{W} | Enchantment |  | When this enchantment enters, return target nonland permanent card with mana value 3 or less from your graveyard to the battlefield. / At the beginning of your end step, if a card left your graveyard this turn, draw a card. | ✅ | Wired in `catalog::sets::sos::enchantments`. ETB returns target Nonland & ManaValueAtMost(3) gy → bf. End-step gated draw uses `Predicate::CardsLeftGraveyardThisTurnAtLeast`. |
| Quill-Blade Laureate // Twofold Intent | {1}{W} // {1}{W} | Creature — Human Cleric // Sorcery | 1/1 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Rapier Wit | {1}{W} | Instant |  | Tap target creature. If it's your turn, put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) / Draw a card. | ✅ | Wired in `catalog::sets::sos::instants` with `IsTurnOf` gating on the stun counter. |
| Rehearsed Debater | {2}{W} | Creature — Djinn Bard | 3/3 | Vigilance / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, this creature gets +1/+1 until end of turn. | ✅ | Vigilance + Repartee +1/+1 EOT, via `effect::shortcut::repartee()` + `Predicate::CastSpellTargetsMatch`. |
| Restoration Seminar | {5}{W}{W} | Sorcery — Lesson |  | Return target nonland permanent card from your graveyard to the battlefield. / Paradigm (...) | 🟡 | Wired in `catalog::sets::sos::sorceries`. Mode 0 (`Move target Nonland gy → bf untapped`) wired faithfully. Paradigm rider omitted (no copy-spell-from-exile-at-upkeep primitive — same gap as Decorum Dissertation, Improvisation Capstone, Echocasting Symposium). |
| Shattered Acolyte | {1}{W} | Creature — Dwarf Warlock | 2/2 | Lifelink / {1}, Sacrifice this creature: Destroy target artifact or enchantment. | ✅ | Wired in `catalog::sets::sos::creatures` with `sac_cost` activation. |
| Soaring Stoneglider | {2}{W} | Creature — Elephant Cleric | 4/3 | As an additional cost to cast this spell, exile two cards from your graveyard or pay {1}{W}. / Flying, vigilance | 🟡 | Wired in `catalog::sets::sos::creatures` as a 4/3 Flying+Vigilance Elephant Cleric at the **paid** cost path: full {3}{W} (base {2}{W} + the {1}{W} payment fork). The alternative additional cost (exile two from gy) is omitted (no alt-cost-with-exile-from-gy primitive). |
| Spiritcall Enthusiast // Scrollboost | {2}{W} // {1}{W} | Creature — Cat Cleric // Sorcery | 3/3 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Stand Up for Yourself | {2}{W} | Instant |  | Destroy target creature with power 3 or greater. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Stirring Hopesinger | {2}{W} | Creature — Bird Bard | 1/3 | Flying, lifelink / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, put a +1/+1 counter on each creature you control. | ✅ | Flying/lifelink body + Repartee fan-out via `ForEach(Creature & ControlledByYou) → AddCounter`. |
| Stone Docent | {1}{W} | Creature — Spirit Chimera | 3/1 | {W}, Exile this card from your graveyard: You gain 2 life. Surveil 1. Activate only as a sorcery. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Push XVII: graveyard-exile activation wired via the new `ActivatedAbility.from_graveyard: bool` + `exile_self_cost: bool` fields. The `activate_ability` engine path now walks the graveyard for `from_graveyard` abilities and exiles the source as part of the cost (mirror to `sac_cost` for battlefield activations). Sorcery-speed gate also now enforced. |
| Summoned Dromedary | {3}{W} | Creature — Spirit Camel | 4/3 | Vigilance / {1}{W}: Return this card from your graveyard to your hand. Activate only as a sorcery. | ✅ | Push XVII: graveyard-recursion activation wired via the new `from_graveyard: bool` field. Cost `{1}{W}` + sorcery-speed + effect `Move(Self → Hand(You))`. |

## Blue

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Banishing Betrayal | {1}{U} | Instant |  | Return target nonland permanent to its owner's hand. Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::instants`. |
| Brush Off | {2}{U}{U} | Instant |  | This spell costs {1}{U} less to cast if it targets an instant or sorcery spell. / Counter target spell. | ✅ (was 🟡) | Push (modern_decks): "{1}{U} less if it targets an instant or sorcery spell" rider wired via `AlternativeCost { mana_cost: {1}{U}, target_filter: Some(SpellOnStack + (Instant or Sorcery)) }`. The counter body is unchanged. When the caster aims at an IS spell on the stack, alt-cost path is available at {1}{U} (half the printed cost); non-IS spells fall back to the {2}{U}{U} hard counter. Test: `brush_off_alt_cost_counters_instant_on_stack`. |
| Campus Composer // Aqueous Aria | {3}{U} // {4}{U} | Creature — Merfolk Bard // Sorcery | 3/4 |  | ✅ (was 🟡) | Push (modern_decks): back-face Aqueous Aria now resolves the "target player draws 3" with the actual player target (was caster-only). Caster aims at self or opp; chosen player draws 3. Front-face `Keyword::Ward(1)` remains a keyword tag (Ward enforcement engine-wide TODO). Test: `campus_composer_aqueous_aria_targets_player`. |
| Chase Inspiration | {U} | Instant |  | Target creature you control gets +0/+3 and gains hexproof until end of turn. (It can't be the target of spells or abilities your opponents control.) | ✅ | Wired in `catalog::sets::sos::instants`. |
| Deluge Virtuoso | {2}{U} | Creature — Human Wizard | 2/2 | When this creature enters, tap target creature an opponent controls and put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) / Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If five or more mana was spent to cast that spell, this creature gets +2/+2 until end of turn instead. | ✅ | Push XXIX: ETB tap+stun wired (same shape as Fractal Mascot). Opus rider **now wired** via `shortcut::opus_trigger(+1/+1 EOT, +2/+2 EOT)`. Mana-spent introspection threads `Predicate::CastSpellManaSpentAtLeast(5)`. Test: `deluge_virtuoso_opus_pumps_one_one_or_two_two`. |
| Divergent Equation | {X}{X}{U} | Instant |  | Return up to X target instant and/or sorcery cards from your graveyard to your hand. / Exile Divergent Equation. | 🟡 (cost ✅) | Wired in `catalog::sets::sos::instants` as a single-target return. The "up to X" multi-target prompt is collapsed to one target (no `Selector::OneOf` / count-bounded pick primitive yet — TODO.md). Push (modern_decks): the "Exile Divergent Equation" rider is **now wired** via the new `CardDefinition.exile_on_resolve` flag — the resolved instant lands in exile, preventing flashback/Past-in-Flames recursion. Test: `divergent_equation_exiles_itself_via_exile_on_resolve_flag`. |
| Echocasting Symposium | {4}{U}{U} | Sorcery — Lesson |  | Target player creates a token that's a copy of target creature you control. / Paradigm (Then exile this spell. After you first resolve a spell with this name, you may cast a copy of it from exile without paying its mana cost at the beginning of each of your first main phases.) | 🟡 | Push XXVI: body-only wire — mints a 3/3 blue Wizard "Echocast" placeholder token via `Effect::CreateToken` (since the engine has no permanent-copy primitive yet). Lesson SpellSubtype tagged. Paradigm rider still ⏳ (cast-from-exile pipeline). |
| Emeritus of Ideation // Ancestral Recall | {3}{U}{U} // {U} | Creature — Human Wizard // Instant | 5/5 |  | ✅ (was 🟡) | Push (modern_decks): back-face Ancestral Recall now targets a player (faithful Oracle) — caster picks self / opp at cast time. Front-face `Keyword::Ward(1)` remains a keyword tag. Tests: `emeritus_of_ideation_back_face_draws_three`, `emeritus_of_ideation_ancestral_recall_targets_opponent`. |
| Encouraging Aviator // Jump | {2}{U} // {U} | Creature — Bird Wizard // Instant | 2/3 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Exhibition Tidecaller | {U} | Creature — Djinn Wizard | 0/2 | Opus — Whenever you cast an instant or sorcery spell, target player mills three cards. If five or more mana was spent to cast that spell, that player mills ten cards instead. | ✅ | Push XXIX: Body + Opus rider wired via `shortcut::opus_trigger(Mill 3, Mill 10)`. The mill target uses `PlayerRef::Target(0)` so the auto-target picker hits an opponent. |
| Flow State | {1}{U} | Sorcery |  | Look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order. If there is an instant card and a sorcery card in your graveyard, instead put two of… | ✅ (was 🟡) | Push XXXIII: conditional draw upgrade wired via `Effect::If` gated on `SelectorExists(IS in gy) AND SelectorExists(Sorcery in gy)`. Mainline (`Scry 3 → Draw 1`) vs. upgrade (`Scry 3 → Draw 2`). Tests: `flow_state_draws_one_when_graveyard_lacks_is_pair`, `flow_state_draws_two_when_graveyard_has_is_pair`. |
| Fractal Anomaly | {U} | Instant |  | Create a 0/0 green and blue Fractal creature token and put X +1/+1 counters on it, where X is the number of cards you've drawn this turn. | ✅ | Wired in `catalog::sets::sos::instants` using the engine's new `Selector::LastCreatedToken` + `Value::CardsDrawnThisTurn` primitives. X=0 → 0/0 token dies to SBA (matches printed). |
| Fractalize | {X}{U} | Instant |  | Until end of turn, target creature becomes a green and blue Fractal with base power and toughness each equal to X plus 1. (It loses all other colors and creature types.) | ✅ (was 🟡) | Push (modern_decks): base-P/T rewrite now wired via `Effect::SetBasePT` (layer-7b primitive — same path as Square Up / Mercurial Transformation). The printed "becomes a Fractal" creature-type rewrite (layer 4) + color rewrite (layer 5) stay omitted; tribal interactions on the target's original type may see the wrong value. Counters and +N/+M still stack per CR 613.7c-f. Tests: `fractalize_sets_target_to_x_plus_one_base` (X=3 → 4/4), `fractalize_layers_under_plus_one_counters` (X=2 + a +1/+1 counter → 4/4). |
| Harmonized Trio // Brainstorm | {U} // {U} | Creature — Merfolk Bard Wizard // Instant | 1/1 |  | ✅ | Push XXVIII promotion: vanilla 1/1 Merfolk Bard Wizard front + faithful Brainstorm back (`Draw 3 + PutOnLibraryFromHand 2`). All Oracle clauses wired. Tests: `harmonized_trio_back_face_is_brainstorm`, `harmonized_trio_back_face_draws_three_then_puts_two_back`. |
| Homesickness | {4}{U}{U} | Instant |  | Target player draws two cards. Tap up to two target creatures. Put a stun counter on each of them. (If a permanent with a stun counter would become untapped, remove one from it instead.) | ✅ (was 🟡) | Push (modern_decks): three-slot multi-target shape — slot 0 = target player (draws 2), slots 1+2 = optional creature targets each get tapped + a stun counter via `TargetFiltered`. Tests: `homesickness_draws_two_taps_and_stuns` (slot 0 + slot 1 — one creature stunned), `homesickness_taps_and_stuns_two_creatures` (all three slots — two creatures stunned). |
| Hydro-Channeler | {1}{U} | Creature — Merfolk Wizard | 1/3 | {T}: Add {U}. Spend this mana only to cast an instant or sorcery spell. / {1}, {T}: Add one mana of any color. Spend this mana only to cast an instant or sorcery spell. | 🟡 | Wired in `catalog::sets::sos::creatures` with both mana abilities (`{T}: Add {U}` and `{1},{T}: Add one mana of any color`). The "spend this mana only to cast an instant or sorcery" restriction is omitted (no spend-restricted mana primitive — TODO.md). |
| Jadzi, Steward of Fate // Oracle's Gift | {2}{U} // {X}{X}{U} | Legendary Creature — Human Wizard // Sorcery | 2/4 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Landscape Painter // Vibrant Idea | {1}{U} // {4}{U} | Creature — Merfolk Wizard // Sorcery | 2/1 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Mana Sculpt | {1}{U}{U} | Instant |  | Counter target spell. If you control a Wizard, add an amount of {C} equal to the amount of mana spent to cast that spell at the beginning of your next main phase. | 🟡 | Wired in `catalog::sets::sos::instants` — counterspell + conditional `If(SelectorExists Wizard).then(AddMana(2 colorless))`. The "amount of mana spent on the countered spell" introspection is unavailable, so we approximate the rider as a flat +{C}{C}; the "delay-until-next-main" rider collapses to immediate add. |
| Mathemagics | {X}{X}{U}{U} | Sorcery |  | Target player draws 2ˣ cards. (2º = 1, 2¹ = 2, 2² = 4, 2³ = 8, 2⁴ = 16, 2⁵ = 32, and so on.) | ✅ | Wired in `catalog::sets::sos::sorceries` via the new `Value::Pow2(XFromCost)` primitive. Multi-target slot collapsed to "you" (caster draws); exponent capped at 30 to avoid deck-out. |
| Matterbending Mage | {2}{U} | Creature — Human Wizard | 2/2 | When this creature enters, return up to one other target creature to its owner's hand. / Whenever you cast a spell with {X} in its mana cost, this creature can't be blocked this turn. | ✅ | Push XVI: both abilities wired. ETB bounce stays as before; the X-cast trigger uses the new `Predicate::CastSpellHasX` + `Effect::GrantKeyword(Unblockable, EOT)` on `Selector::This`. |
| Muse Seeker | {1}{U} | Creature — Elf Wizard | 1/2 | Opus — Whenever you cast an instant or sorcery spell, draw a card. Then discard a card unless five or more mana was spent to cast that spell. | ✅ | Push XXIX: Body + Opus rider wired via `shortcut::opus_trigger`. Small body draws + discards; big body (≥5 mana) skips the discard. |
| Muse's Encouragement | {4}{U} | Instant |  | Create a 3/3 blue and red Elemental creature token with flying. / Surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | ✅ | Mints a 3/3 U/R Flying Elemental via the shared `elemental_token()` helper + `Effect::Surveil 2`. |
| Orysa, Tide Choreographer | {4}{U} | Legendary Creature — Merfolk Bard | 2/2 | This spell costs {3} less to cast if creatures you control have total toughness 10 or greater. / When Orysa enters, draw two cards. | 🟡 | ETB draw 2 wired faithfully. The conditional "{3} less if total toughness ≥ 10" alt-cost rider is omitted (alt-cost-with-board-state-predicate primitive). |
| Pensive Professor | {1}{U}{U} | Creature — Human Wizard | 0/2 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / Whenever one or more +1/+1 counters are put on this creature, you may draw a card. | ✅ (was 🟡) | Push XXXVIII: both halves wired. Increment via `shortcut::increment_self_plus_one()`; the secondary CounterAdded rider is wired via `EventKind::CounterAdded(PlusOnePlusOne) + SelfSource` + `Effect::MayDo(Draw 1)`. Counters added to other creatures don't fire the rider (SelfSource gate). Test: `pensive_professor_secondary_counter_trigger_draws_a_card`. |
| Procrastinate | {X}{U} | Sorcery |  | Tap target creature. Put twice X stun counters on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) | ✅ | Wired in `catalog::sets::sos::sorceries` with `Value::Times(2, XFromCost)`. |
| Run Behind | {3}{U} | Instant |  | This spell costs {1} less to cast if it targets an attacking creature. / Target creature's owner puts it on their choice of the top or bottom of their library. | 🟡 (cost ✅) | Push (modern_decks): "{1} less if it targets an attacking creature" rider wired via `AlternativeCost { mana_cost: {2}{U}, target_filter: Some(Creature + IsAttacking) }`. When the caster aims at an attacking creature, alt-cost path is available at {2}{U}; otherwise full {3}{U}. The "top or bottom owner's choice" rider is still collapsed to bottom-only (engine has no top-or-bottom prompt for the *owner* of the moved card — tracked in TODO.md). Test: `run_behind_alt_cost_bounces_attacking_creature_to_library_bottom`. |
| Skycoach Conductor // All Aboard | {2}{U} // {U} | Creature — Bird Pilot // Instant | 2/3 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Spellbook Seeker // Careful Study | {3}{U} // {U} | Creature — Bird Wizard // Sorcery | 3/3 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Tester of the Tangential | {1}{U} | Creature — Djinn Wizard | 1/1 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / At the beginning of combat on your turn, you may pay {X}. When you do, move X +1/+1 counters from this creature onto another target creature. | 🟡 | Push XXXVIII (doc-sync): 1/1 Djinn Wizard body + Increment wired via `increment_self_plus_one()`. The combat-step pay-X-to-move-counters trigger is still omitted (no X-cost optional trigger primitive — same gap as Berta's activation). The Increment is the printed engine that turns the small body into a 4/4+ over the game. |
| Textbook Tabulator | {2}{U} | Creature — Frog Wizard | 0/3 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / When this creature enters, surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | ✅ (was 🟡) | Push XXXVIII: both halves wired. ETB Surveil 2 via `Effect::Surveil`; Increment rider via `shortcut::increment_self_plus_one()`. The Frog Wizard's 0/3 body grows into a real attacker as the spellslinger turns pile up. Test: `textbook_tabulator_increment_buffs_self_on_big_spell`. |
| Wisdom of Ages | {4}{U}{U}{U} | Sorcery |  | Return all instant and sorcery cards from your graveyard to your hand. You have no maximum hand size for the rest of the game. / Exile Wisdom of Ages. | ✅ (was 🟡) | Push (modern_decks): all three printed clauses now ship. (a) Mass IS-gy-to-hand return. (b) "No max hand size" via `Effect::SetNoMaxHandSize`. (c) "Exile Wisdom of Ages" **now wired** (push: modern_decks current sub-push) via the new `CardDefinition.exile_on_resolve` flag — the resolved sorcery lands in exile, preventing flashback/Past-in-Flames recursion. Tests: `wisdom_of_ages_lets_caster_keep_more_than_seven_cards`, `wisdom_of_ages_exiles_itself_after_resolve_via_exile_on_resolve_flag`. |

## Black

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Adventurous Eater // Have a Bite | {2}{B} // {B} | Creature — Human Warlock // Sorcery | 3/2 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Arcane Omens | {4}{B} | Sorcery |  | Converge — Target player discards X cards, where X is the number of colors of mana spent to cast this spell. | ✅ | Wired in `catalog::sets::sos::sorceries` via `Effect::Discard { amount: Value::ConvergedValue }` against `EachOpponent`. |
| Arnyn, Deathbloom Botanist | {2}{B} | Legendary Creature — Vampire Druid | 2/2 | Deathtouch / Whenever a creature you control with power or toughness 1 or less dies, target opponent loses 2 life and you gain 2 life. | ✅ | Wired in `catalog::sets::sos::creatures` (deathtouch + `CreatureDied/AnotherOfYours` trigger gated by `Predicate::EntityMatches { what: TriggerSource, filter: PowerAtMost(1).or(ToughnessAtMost(1)) }`). |
| Burrog Banemaker | {B} | Creature — Frog Warlock | 1/1 | Deathtouch / {1}{B}: This creature gets +1/+1 until end of turn. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Cheerful Osteomancer // Raise Dead | {3}{B} // {B} | Creature — Orc Warlock // Sorcery | 4/2 |  | ✅ | Push XXVIII promotion: vanilla 4/2 Orc Warlock front + faithful Raise Dead back (`Move target creature card from graveyard → hand`). All Oracle clauses wired. Test: `cheerful_osteomancer_back_face_returns_creature_from_graveyard`. |
| Cost of Brilliance | {2}{B} | Sorcery |  | Target player draws two cards and loses 2 life. Put a +1/+1 counter on up to one target creature. | ✅ (was 🟡) | Push (modern_decks): two-target shape now wired via multi-target. Slot 0 = target player (draws 2 + loses 2 life). Slot 1 = optional creature target gets a +1/+1 counter. Slot 1 uses `TargetFiltered` so it resolves to no-op when only one target is passed. Tests: `cost_of_brilliance_draws_two_loses_two_pumps_creature` (both slots), `cost_of_brilliance_can_target_opponent_for_draw` (aim slot 0 at opp). |
| Decorum Dissertation | {3}{B}{B} | Sorcery — Lesson |  | Target player draws two cards and loses 2 life. / Paradigm (...) | ✅ (was 🟡) | Push (modern_decks): Target-player prompt now wired via `target_filtered(Player)` — same pattern as Cost of Brilliance. The caster aims at self for the printed asymmetric "draw 2, lose 2" trade or at an opp to drain 2 life. Paradigm rider omitted (no copy-spell-from-exile-at-upkeep primitive — same gap as Germination Practicum, Improvisation Capstone). Tests: `decorum_dissertation_draws_two_loses_two` (target self), `decorum_dissertation_can_target_opponent` (target opp). |
| Dissection Practice | {B} | Instant |  | Target opponent loses 1 life and you gain 1 life. / Up to one target creature gets +1/+1 until end of turn. / Up to one target creature gets -1/-1 until end of turn. | ✅ (was 🟡) | Push (modern_decks): all three target slots now wired via multi-target. Slot 0 = target opponent (loses 1, caster gains 1). Slot 1 = optional creature target gets +1/+1 EOT. Slot 2 = optional creature target gets -1/-1 EOT. Slots 1/2 use `TargetFiltered` so they no-op when fewer targets are passed. Tests: `dissection_practice_drains_one_and_shrinks_target` (all three slots filled — drain + pump + shrink), `dissection_practice_drain_only_no_creature_targets` (slot 0 only). |
| Emeritus of Woe // Demonic Tutor | {3}{B} // {1}{B} | Creature — Vampire Warlock // Sorcery | 5/4 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| End of the Hunt | {1}{B} | Sorcery |  | Target opponent exiles a creature or planeswalker they control with the greatest mana value among creatures and planeswalkers they control. | 🟡 | Wired in `catalog::sets::sos::sorceries` as a single-target Exile against `Creature ∨ Planeswalker & ControlledByOpponent`. The "greatest mana value" picker isn't enforced (auto-target picks first eligible). |
| Eternal Student | {3}{B} | Creature — Zombie Warlock | 4/2 | {1}{B}, Exile this card from your graveyard: Create two 1/1 white and black Inkling creature tokens with flying. | ✅ | Push XVII: graveyard-exile activation wired via the new `from_graveyard: bool` + `exile_self_cost: bool` fields. Cost `{1}{B}` + exile-self-as-cost + effect creates 2 Inkling tokens. |
| Foolish Fate | {2}{B} | Instant |  | Destroy target creature. / Infusion — If you gained life this turn, that creature's controller loses 3 life. | ✅ | Wired with the new `Predicate::LifeGainedThisTurnAtLeast` Infusion gate. |
| Forum Necroscribe | {5}{B} | Creature — Troll Warlock | 5/4 | Ward—Discard a card. / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, return target creature card from your graveyard to the battlefield. | ✅ (was 🟡) | Push (modern_decks): Ward—Discard a card now wired via `Keyword::Ward(WardCost::Discard(1))` and the new `Effect::CounterUnless` resolver — auto-pays by discarding the first card in the spell controller's hand. Insufficient cards in hand → spell countered. Repartee body unchanged. Tests: `ward_discard_counters_when_payer_has_no_other_cards_in_hand`, `ward_discard_resolves_when_payer_has_a_spare_card`. |
| Grave Researcher // Reanimate | {2}{B} // {B} | Creature — Troll Warlock // Sorcery | 3/3 |  | ✅ (was 🟡) | Push (modern_decks): All three printed clauses now ship. Front 3/3 Troll Warlock with ETB Surveil 1. Back-face Reanimate at {B} — `target_filtered(Creature)` graveyard pick → Move to Battlefield(You) → `LoseLife(ManaValueOf(Target(0)))`. The lose-life-equal-to-MV clause reads off the post-Move target's CardId via `Value::ManaValueOf`'s zone walk (battlefield / graveyard / exile / hand). Tests: `grave_researcher_back_face_reanimates_creature_from_graveyard` (asserts both reanimation and -CMC life loss), `grave_researcher_front_etb_surveils_one`. |
| Lecturing Scornmage | {B} | Creature — Human Warlock | 1/1 | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, put a +1/+1 counter on this creature. | ✅ | Repartee +1/+1 counter via `effect::shortcut::repartee()`. |
| Leech Collector // Bloodletting | {1}{B} // {B} | Creature — Human Warlock // Sorcery | 2/2 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Masterful Flourish | {B} | Instant |  | Target creature you control gets +1/+0 and gains indestructible until end of turn. (Damage and effects that say "destroy" don't destroy it.) | ✅ | Wired in `catalog::sets::sos::instants`. |
| Melancholic Poet | {1}{B} | Creature — Elf Bard | 2/2 | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, each opponent loses 1 life and you gain 1 life. | ✅ | Repartee drain 1 via `effect::shortcut::repartee()`. |
| Moseo, Vein's New Dean | {2}{B} | Legendary Creature — Bird Skeleton Warlock | 2/1 | Flying / When Moseo enters, create a 1/1 black and green Pest creature token with "Whenever this token attacks, you gain 1 life." / Infusion — At the beginning of your end step, if you gained life this turn, return up to one target creature card from your graveyard to your hand. | ✅ (was 🟡) | Push (modern_decks): Body + Flying + ETB Pest token + **Infusion end-step return now all wired**. The end-step trigger fires for the active player; the body is gated on `Predicate::LifeGainedThisTurnAtLeast(You, 1)` (canonical Infusion gate). Inside the gate, `Effect::Move { take(1, CardsInZone(Graveyard, Creature)), Hand(You) }` reanimates-to-hand the top matching creature card. "Up to one" semantics fall out naturally — graveyard has no matching cards → move resolves to nothing. Tests: `moseo_veins_new_dean_is_2_1_flying_pest_etb_minter`, `moseo_veins_new_dean_infusion_returns_creature_to_hand_when_life_gained`, `moseo_veins_new_dean_infusion_no_return_without_life_gain`. |
| Poisoner's Apprentice | {2}{B} | Creature — Orc Warlock | 2/2 | Infusion — When this creature enters, target creature an opponent controls gets -4/-4 until end of turn if you gained life this turn. | ✅ | Wired in `catalog::sets::sos::creatures` with the `LifeGainedThisTurnAtLeast` Infusion gate on the ETB trigger. |
| Postmortem Professor | {1}{B} | Creature — Zombie Warlock | 2/2 | This creature can't block. / Whenever this creature attacks, each opponent loses 1 life and you gain 1 life. / {1}{B}, Exile an instant or sorcery card from your graveyard: Return this card from your graveyard to the battlefield. | ✅ (was 🟡) | Push XXXVIII (doc-only): On-attack drain + the printed `Keyword::CantBlock` static restriction + the graveyard-exile recursion activated ability all wired. The activation uses `from_graveyard: true` + `exile_other_filter: Some(Instant ∨ Sorcery)` (paired with `mana_cost: {1}{B}`) — the engine's `activate_ability` path walks the graveyard for `from_graveyard` abilities and exiles the chosen IS card as part of the cost, then `Effect::Move(SelfSource → Battlefield(You))` returns the Professor. Tests in `tests/sos.rs`: `postmortem_professor_returns_from_graveyard_by_exiling_instant_or_sorcery`. |
| Pox Plague | {B}{B}{B}{B}{B} | Sorcery |  | Each player loses half their life, then discards half the cards in their hand, then sacrifices half the permanents they control of their choice. Round down each time. | ✅ | Wired in `catalog::sets::sos::sorceries` via `ForEach Player(EachPlayer)` body using the new `Value::HalfDown` + `Value::PermanentCountControlledBy(Triggerer)` primitives. Half-life / half-hand / half-permanents per player. |
| Pull from the Grave | {2}{B} | Sorcery |  | Return up to two target creature cards from your graveyard to your hand. You gain 2 life. | ✅ (was 🟡) | Push XXXVIII (doc-sync): all printed clauses ship — `Selector::take(_, 2)` returns up to two creature cards from the controller's graveyard (≤0 if gy is empty, 1 if only one creature card, 2 if ≥2), matching the printed "up to two target" cap. Lifegain half always resolves. Engine-wide multi-target UI picker for the two-card prompt is still ⏳; the auto-decider picks the top two matching creature cards in graveyard order — functionally identical to the printed text since the caster can always choose this set. |
| Rabid Attack | {1}{B} | Instant |  | Until end of turn, any number of target creatures you control each get +1/+0 and gain "When this creature dies, draw a card." | 🟡 | Push (modern_decks): "any number of target creatures" promoted to three slots — slot 0 (mandatory) + slots 1+2 (optional). Each filled slot gets +1/+0 EOT via `TargetFiltered`. AutoDecider fills slot 0 only; scripted tests can wire up to three. The transient die-to-draw triggered ability grant is still omitted (engine has no per-creature "grant triggered ability for a duration" primitive — tracked in TODO.md). Tests: `rabid_attack_pumps_friendly_creature` (slot 0 only), `rabid_attack_pumps_multiple_creatures_via_multi_target` (all three slots → 3 creatures pumped). |
| Ral Zarek, Guest Lecturer | {1}{B}{B} | Legendary Planeswalker — Ral | [3] | +1: Surveil 2. / −1: Any number of target players each discard a card. / −2: Return target creature card with mana value 3 or less from your graveyard to the battlefield. / −7: Flip five coins. Target opponent skips their next X turns, where X is the number of coins that came up heads. | 🟡 | +1 Surveil 2 / -1 each opp discards 1 (single-target collapse) / -2 return ≤3-MV creature card from your gy → bf. -7 coin-flip / skip-turns ult omitted (no coin-flip + no skip-turns primitive). |
| Scathing Shadelock // Venomous Words | {4}{B} // {B} | Creature — Snake Warlock // Sorcery | 4/6 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Scheming Silvertongue // Sign in Blood | {1}{B} // {B}{B} | Creature — Vampire Warlock // Sorcery | 1/3 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Send in the Pest | {1}{B} | Sorcery |  | Each opponent discards a card. You create a 1/1 black and green Pest creature token with "Whenever this token attacks, you gain 1 life." | ✅ | Discard + Pest token wired; the token's "gain 1 on attack" rider now fires (via the new `TokenDefinition.triggered_abilities` field). |
| Sneering Shadewriter | {4}{B} | Creature — Vampire Warlock | 3/3 | Flying / When this creature enters, each opponent loses 2 life and you gain 2 life. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Tragedy Feaster | {2}{B}{B} | Creature — Demon | 7/6 | Trample / Ward—Discard a card. / Infusion — At the beginning of your end step, sacrifice a permanent unless you gained life this turn. | ✅ (was 🟡) | Push (modern_decks): 7/6 Trample Demon body + **Infusion end-step sac-unless-lifegain rider now wired** via a `StepBegins(End) / ActivePlayer` trigger gated on `Predicate::LifeGainedThisTurnAtLeast(You, 1)` — when you've gained life this turn, the trigger resolves as Noop; otherwise it forces `Effect::Sacrifice { who: You, count: 1, filter: Permanent }`. Ward — Discard a card is still keyword-tagged (`Keyword::Ward(0)` placeholder); the counter-the-spell-unless-discard enforcement is engine-wide ⏳. Tests: `tragedy_feaster_infusion_forces_sacrifice_when_no_life_gained`, `tragedy_feaster_infusion_skips_sacrifice_when_life_gained`. |
| Ulna Alley Shopkeep | {2}{B} | Creature — Goblin Warlock | 2/3 | Menace (This creature can't be blocked except by two or more creatures.) / Infusion — This creature gets +2/+0 as long as you gained life this turn. | ✅ (was 🟡) | Push XXXVIII: Menace keyworded; Infusion static `+2/+0` rider wired via a compute-time injection in `GameState::compute_battlefield` (same pattern as Honor Troll). When `Player.life_gained_this_turn > 0`, layer 7b adds `ModifyPowerToughness(+2, +0)` targeting the source; reset on next untap step. Tests: `ulna_alley_shopkeep_no_lifegain_is_two_three`, `ulna_alley_shopkeep_with_lifegain_is_four_three`. |
| Wander Off | {3}{B} | Instant |  | Exile target creature. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Withering Curse | {1}{B}{B} | Sorcery |  | All creatures get -2/-2 until end of turn. / Infusion — If you gained life this turn, destroy all creatures instead. | ✅ | `If LifeGainedThisTurnAtLeast(1)` branch: Infusion-path = ForEach(Creature) Destroy; mainline = ForEach(Creature) PumpPT(-2/-2 EOT). |

## Red

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Archaic's Agony | {4}{R} | Sorcery |  | Converge — Archaic's Agony deals X damage to target creature, where X is the number of colors of mana spent to cast this spell. Exile cards from the top of your library equal to the excess damage dealt to that creature this way. You may play those cards until the end of your next turn. | 🟡 | Push XXVIII: ⏳ → 🟡. Body wired — Converge X damage to a creature target via `Value::ConvergedValue` (same primitive as Rancorous Archaic / Sundering Archaic). The "exile cards equal to excess damage + may play until next end step" rider is still omitted (cast-from-exile pipeline + "exile N for excess damage" primitive both missing). At converge 5 this is a 5-damage burn spell for {4}{R}. |
| Artistic Process | {3}{R}{R} | Sorcery |  | Choose one — / • Artistic Process deals 6 damage to target creature. / • Artistic Process deals 2 damage to each creature you don't control. / • Create a 3/3 blue and red Elemental creature token with flying. It gains haste until end of turn. | ✅ | Wired in `catalog::sets::sos::sorceries`. All three modes wired: 6-to-creature, 2-to-each-opp-creature (via `Selector::EachPermanent(Creature & ControlledByOpponent)`), Elemental token + transient haste via `Selector::LastCreatedToken`. |
| Blazing Firesinger // Seething Song | {2}{R} // {2}{R} | Creature — Dwarf Bard // Instant | 2/3 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Charging Strifeknight | {2}{R} | Creature — Spirit Knight | 3/3 | Haste / {T}, Discard a card: Draw a card. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Choreographed Sparks | {R}{R} | Instant |  | This spell can't be copied. / Choose one or both — / • Copy target instant or sorcery spell you control. You may choose new targets for the copy. / • Copy target creature spell you control. The copy gains haste and "At the beginning of the end step, sacrifice this token." | 🟡 | Push XXVI: Single-mode wire via `Effect::CopySpell` against an IS-on-stack target (the "or copy a creature spell" branch needs a permanent-spell copy variant). The "this spell can't be copied" rider is omitted (no `CantBeCopied` keyword tag yet). |
| Duel Tactics | {R} | Sorcery |  | Duel Tactics deals 1 damage to target creature. It can't block this turn. / Flashback {1}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Wired as `DealDamage(1) + GrantKeyword(CantBlock, EOT)` — pulls in the new `Keyword::CantBlock` (enforced inside `declare_blockers` and the `can_block_*` helpers). Flashback {1}{R} now wired via `Keyword::Flashback` (push X). |
| Emeritus of Conflict // Lightning Bolt | {1}{R} // {R} | Creature — Human Wizard // Instant | 2/2 |  | ✅ | Push XXVIII promotion: vanilla 2/2 Human Wizard front + faithful Lightning Bolt back (`DealDamage 3 to target`). All Oracle clauses wired. Test: `emeritus_of_conflict_back_face_burns_three`. |
| Expressive Firedancer | {1}{R} | Creature — Human Sorcerer | 2/2 | Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If five or more mana was spent to cast that spell, this creature also gains double strike until end of turn. | ✅ | Push XXIX: Opus rider **now wired** via `shortcut::opus_trigger`. Small body: +1/+1 EOT. Big body (≥5 mana): +1/+1 EOT + DoubleStrike EOT. Test: `expressive_firedancer_opus_grants_double_strike_at_five_mana`. |
| Flashback | {R} | Instant |  | Target instant or sorcery card in your graveyard gains flashback until end of turn. The flashback cost is equal to its mana cost. (You may cast that card from your graveyard for its flashback cost. Then exile it.) | 🟡 | Push XXVI: Approximated as a {R} "return a target IS card from your graveyard to your hand" instant — the player can re-cast it next turn at its normal cost. Strictly weaker than the printed "flashback for its mana cost this turn" but preserves the recovery outcome. A true wiring needs a transient per-instance grant on a graveyard card. |
| Garrison Excavator | {3}{R} | Creature — Orc Sorcerer | 3/4 | Menace (This creature can't be blocked except by two or more creatures.) / Whenever one or more cards leave your graveyard, create a 2/2 red and white Spirit creature token. | ✅ | Wired against the new `EventKind::CardLeftGraveyard` event — every gy-leave mints a 2/2 R/W Spirit token via the shared `spirit_token()` helper. |
| Goblin Glasswright // Craft with Pride | {1}{R} // {R} | Creature — Goblin Sorcerer // Sorcery | 2/2 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Heated Argument | {4}{R} | Instant |  | Heated Argument deals 6 damage to target creature. You may exile a card from your graveyard. If you do, Heated Argument also deals 2 damage to that creature's controller. | ✅ | Push XV → ✅ in push XXVIII: 6-to-creature is unconditional; the gy-exile + 2-to-controller chain is wrapped in `Effect::MayDo` and either both fire or both skip — faithful to the printed "you may". Uses `Selector::take(CardsInZone(GY), 1)` to pick exactly one gy card (matching "a card", not "every card"). |
| Impractical Joke | {R} | Sorcery |  | Damage can't be prevented this turn. Impractical Joke deals 3 damage to up to one target creature or planeswalker. | 🟡 | 3-to-creature/PW wired; "damage can't be prevented" rider is a no-op (engine has no damage-prevention layer). |
| Improvisation Capstone | {5}{R}{R} | Sorcery — Lesson |  | Exile cards from the top of your library until you exile cards with total mana value 4 or greater. You may cast any number of spells from among them without paying their mana costs. / Paradigm (Then exile this spell. After you first resolve a spell with this name, you may cast a copy of it from exile without paying its mana cost at the beginning of each of your first main phases.) | ⏳ | 🔍 needs review (oracle previously truncated). Needs: copy-spell/permanent primitive; cast-from-exile pipeline. |
| Living History | {1}{R} | Enchantment |  | When this enchantment enters, create a 2/2 red and white Spirit creature token. / Whenever you attack, if a card left your graveyard this turn, target attacking creature gets +2/+0 until end of turn. | ✅ (was 🟡) | Push (modern_decks doc-sync): ETB Spirit token + on-attack +2/+0 EOT (gated on `Predicate::CardsLeftGraveyardThisTurnAtLeast`). The "target attacking creature" picks the trigger source (the just-declared attacker) — same per-attacker pattern as Sparring Regimen ✅ / Mentor in Combat Professor ✅. The auto-target framework correctly lands the pump on the iterated attacker. Test: `living_history_etb_creates_spirit_token`. |
| Maelstrom Artisan // Rocket Volley | {1}{R}{R} // {1}{R} | Creature — Minotaur Sorcerer // Sorcery | 3/2 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Magmablood Archaic | {2/R}{2/R}{2/R} | Creature — Avatar | 2/2 | Trample, reach / Converge — This creature enters with a +1/+1 counter on it for each color of mana spent to cast it. / Whenever you cast an instant or sorcery spell, creatures you control get +1/+0 until end of turn for each color of mana spent to cast that spell. | 🟡 | Body wired in `catalog::sets::sos::creatures` (2/2 Avatar with Trample+Reach + Converge ETB AddCounter using `Value::ConvergedValue`). The IS-cast pump rider is omitted pending per-cast converge introspection on the *just-cast* spell (the trigger fires but reads the Archaic's own ETB converge value, not the iterated cast's). Hybrid `{2/R}` pips approximated as `{2}+{R}` per pip. |
| Mica, Reader of Ruins | {3}{R} | Legendary Creature — Human Artificer | 4/4 | Ward—Pay 3 life. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays 3 life.) / Whenever you cast an instant or sorcery spell, you may sacrifice an artifact. If you do, copy that spell and you may choose new targets for the copy. | ✅ (was 🟡) | Push (modern_decks): Ward—Pay 3 life now wired via `Keyword::Ward(WardCost::Life(3))` and the new `Effect::CounterUnless` resolver — auto-pays by deducting 3 life from the spell controller (CR 119.4: payment fails if the controller doesn't have ≥3 life, countering the spell). Magecraft sac-artifact-to-copy rider unchanged. Tests: `ward_pay_life_counters_when_payer_has_insufficient_life`, `ward_pay_life_resolves_when_payer_has_sufficient_life`. |
| Molten-Core Maestro | {1}{R} | Creature — Goblin Bard | 2/2 | Menace / Opus — Whenever you cast an instant or sorcery spell, put a +1/+1 counter on this creature. If five or more mana was spent to cast that spell, add an amount of {R} equal to this creature's power. | ✅ | Push XXIX: Opus rider **now wired** via `shortcut::opus_trigger`. Small body: +1/+1 counter on this creature. Big body (≥5 mana): counter + add {R}×power via `ManaPayload::OfColor(Red, PowerOf(This))`. |
| Pigment Wrangler // Striking Palette | {4}{R} // {R} | Creature — Orc Sorcerer // Sorcery | 4/4 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Rearing Embermare | {4}{R} | Creature — Horse Beast | 4/5 | Reach, haste | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Rubble Rouser | {2}{R} | Creature — Dwarf Sorcerer | 1/4 | When this creature enters, you may discard a card. If you do, draw a card. / {T}, Exile a card from your graveyard: Add {R}. When you do, this creature deals 1 damage to each opponent. | 🟡 | Push XV: ETB rummage now wrapped in `Effect::MayDo` so the "you may discard" optionality is honored. The `{T}, Exile a card from your graveyard:` activated ability is still omitted (engine activated-ability path has no `from-your-graveyard` cost variant — separate from `sac_cost`). |
| Steal the Show | {2}{R} | Sorcery |  | Choose one or both — / • Target player discards any number of cards, then draws that many cards. / • Steal the Show deals damage equal to the number of instant and sorcery cards in your graveyard to target creature or planeswalker. | ✅ (was 🟡) | Push (modern_decks): mode 0 now uses `Effect::DiscardAnyNumber` (same primitive as Colossus of the Blood Age + Borrowed Knowledge), so the targeted player picks how many cards to discard then draws exactly that many via `Value::CardsDiscardedThisEffect`. Mode 1 reads the IS-graveyard count from the caster's gy and damages a creature/PW. The "choose one or both" rider still collapses to "pick one mode" (no multi-mode-pick primitive that generalises `ChooseN` to per-target slots). Tests: `steal_the_show_mode_zero_discard_any_number_drops_zero_by_default`, `steal_the_show_mode_one_burns_creature_by_is_graveyard_count`. |
| Strife Scholar // Awaken the Ages | {2}{R} // {5}{R} | Creature — Orc Sorcerer // Sorcery | 3/2 |  | ✅ (was 🟡) | Push (modern_decks): Front 3/2 Orc Sorcerer with `Keyword::Ward(1)` (keyword tag). Back-face Awaken the Ages at {5}{R} returns all creature cards from your graveyard to the battlefield via `Selector::CardsInZone(Graveyard, Creature)`. The "Then exile Awaken the Ages" rider is **now wired** via the new `CardDefinition.exile_on_resolve` flag — the resolved sorcery lands in exile (not graveyard), bumping `cards_exiled_this_turn` for Ennis-style payoffs. Test: `awaken_the_ages_exiles_itself_after_resolve_via_exile_on_resolve_flag`. |
| Tablet of Discovery | {2}{R} | Artifact |  | When this artifact enters, mill a card. You may play that card this turn. (To mill a card, put the top card of your library into your graveyard.) / {T}: Add {R}. / {T}: Add {R}{R}. Spend this mana only to cast instant and sorcery spells. | 🟡 | Wired in `catalog::sets::sos::artifacts` — ETB Mill 1 + two `{T}: Add {R}` mana abilities. The "may play that card this turn" mill-rider is omitted (no per-card may-play primitive yet). The spend-restriction on the {T}: Add {R}{R} ability is omitted (no spend-restricted mana primitive). |
| Tackle Artist | {3}{R} | Creature — Orc Sorcerer | 4/3 | Trample / Opus — Whenever you cast an instant or sorcery spell, put a +1/+1 counter on this creature. If five or more mana was spent to cast that spell, put two +1/+1 counters on this creature instead. | ✅ | Push XXIX: Opus rider **now wired** via `shortcut::opus_trigger`. Small body: one +1/+1 counter. Big body (≥5 mana): two +1/+1 counters instead. Tests: `tackle_artist_opus_lands_*_counter*`. |
| Thunderdrum Soloist | {1}{R} | Creature — Dwarf Bard | 1/3 | Reach / Opus — Whenever you cast an instant or sorcery spell, this creature deals 1 damage to each opponent. If five or more mana was spent to cast that spell, this creature deals 3 damage to each opponent instead. | ✅ | Push XXIX: Opus rider **now wired** via `shortcut::opus_trigger`. Small body: 1 dmg to each opp. Big body (≥5 mana): 3 dmg to each opp. Test: `thunderdrum_soloist_opus_pings_one_at_small_three_at_big`. |
| Tome Blast | {1}{R} | Sorcery |  | Tome Blast deals 2 damage to any target. / Flashback {4}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Wired as a 2-to-any-target burn spell. Flashback {4}{R} now wired via `Keyword::Flashback` (push X). |
| Unsubtle Mockery | {2}{R} | Instant |  | Unsubtle Mockery deals 4 damage to target creature. Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | `DealDamage(4) + Surveil 1` via `Effect::Surveil`. |
| Zealous Lorecaster | {5}{R} | Creature — Giant Sorcerer | 4/4 | When this creature enters, return target instant or sorcery card from your graveyard to your hand. | ✅ | Wired in `catalog::sets::sos::creatures` with a Move-target-from-graveyard ETB trigger. |

## Green

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Aberrant Manawurm | {3}{G} | Creature — Wurm | 2/5 | Trample / Whenever you cast an instant or sorcery spell, this creature gets +X/+0 until end of turn, where X is the amount of mana spent to cast that spell. | ✅ | Push XXIX: Magecraft-style trigger **now wired** via `shortcut::magecraft(PumpPT{power: CastSpellManaSpent})`. The pump reads the just-cast spell's mana_spent so a 5-mana spell gives the wurm +5/+0 EOT. Test: `aberrant_manawurm_pumps_by_mana_spent_eot`. |
| Additive Evolution | {3}{G}{G} | Enchantment |  | When this enchantment enters, create a 0/0 green and blue Fractal creature token. Put three +1/+1 counters on it. / At the beginning of combat on your turn, put a +1/+1 counter on target creature you control. It gains vigilance until end of turn. | ✅ | Wired in `catalog::sets::sos::enchantments`. ETB Fractal-with-3-counters via the existing `fractal_token()` helper + `Selector::LastCreatedToken` AddCounter. Begin-combat +1/+1 counter + Vigilance (EOT) on a friendly creature, gated through the active-player StepBegins(BeginCombat) trigger. |
| Ambitious Augmenter | {G} | Creature — Turtle Wizard | 1/1 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / When this creature dies, if it had one or more counters on it, create a 0/0 green and blue Fractal creature token, then put this creature's counters on that token. | 🟡 | Body-only wire in `catalog::sets::sos::creatures` (1/1 Turtle Wizard at {G}). Increment pump omitted (mana-spent-on-cast introspection missing — tracked in TODO.md). The death-with-counters → Fractal-with-counters trigger is also omitted pending a counter-transfer-on-death primitive. |
| Burrog Barrage | {1}{G} | Instant |  | Target creature you control gets +1/+0 until end of turn if you've cast another instant or sorcery spell this turn. Then it deals damage equal to its power to up to one target creature an opponent controls. | ✅ (was 🟡) | Push (modern_decks doc-sync): both halves wired and exercised. Conditional pump (gated on `Predicate::SpellsCastThisTurnAtLeast(2)`); power-as-damage uses `Selector::TargetFiltered { slot: 1, .. }` so the optional opp-creature defender slot is passed via `additional_targets` at cast time. AutoDecider fills slot 0 only (typical bot play); the scripted-test suite covers both slot 0-only (no damage) and slots 0+1 (damage = friendly's pumped power → opp bear dies). Tests: `burrog_barrage_no_pump_on_first_spell_skips_damage_with_no_opp_target`, `burrog_barrage_kills_opp_bear_with_second_target_filled`. |
| Chelonian Tackle | {2}{G} | Sorcery |  | Target creature you control gets +0/+10 until end of turn. Then it fights up to one target creature an opponent controls. (Each deals damage equal to its power to the other.) | ✅ (was 🟡) | Push (modern_decks): slot-1 multi-target promotion. Slot 0 = friendly creature to pump +0/+10 EOT; slot 1 = optional opp creature defender (via `Selector::TargetFiltered { slot: 1 }`). Fight no-ops cleanly when slot 1 isn't filled — preserving the printed "up to one" semantics. AutoDecider's `auto_targets_for_effect_all_slots` (server/bot.rs) fills slot 1 when an opp creature is on the battlefield. Tests: `chelonian_tackle_pumps_toughness` (slot 0 only — fight no-ops), `chelonian_tackle_fights_opp_creature` (both slots — opp creature dies). |
| Comforting Counsel | {1}{G} | Enchantment |  | Whenever you gain life, put a growth counter on this enchantment. / As long as there are five or more growth counters on this enchantment, creatures you control get +3/+3. | ✅ (was 🟡) | Push (modern_decks): Lifegain → Growth counter trigger wired in `catalog::sets::sos::enchantments`. The "≥5 counters → anthem" static is **now wired** via a compute-time injection in `GameState::compute_battlefield` (Honor Troll pattern) — gate reads `card.counters[Growth] >= 5`; when true, layer 7b pumps every creature controlled by the enchantment's controller by +3/+3 via `AffectedPermanents::All { controller, card_types: [Creature] }`. Tests: `comforting_counsel_no_anthem_below_five_counters`, `comforting_counsel_anthem_buffs_friendly_creatures_at_five_counters`, `comforting_counsel_accrues_growth_on_lifegain`. |
| Efflorescence | {2}{G} | Instant |  | Put two +1/+1 counters on target creature. / Infusion — If you gained life this turn, that creature also gains trample and indestructible until end of turn. | ✅ | Wired with the new `Predicate::LifeGainedThisTurnAtLeast` Infusion gate. |
| Emeritus of Abundance // Regrowth | {2}{G} // {1}{G} | Creature — Elf Druid // Sorcery | 3/4 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Emil, Vastlands Roamer | {2}{G} | Legendary Creature — Elf Druid | 3/3 | Creatures you control with +1/+1 counters on them have trample. / {4}{G}, {T}: Create a 0/0 green and blue Fractal creature token. Put X +1/+1 counters on it, where X is the number of differently named lands you control. | ✅ | Wired in `catalog::sets::sos::creatures` — `StaticEffect::GrantKeyword(Trample)` filtered to creatures with +1/+1 counters via the new `AffectedPermanents::AllWithCounter` layer variant; activated `{4}{G},{T}` creates a Fractal + counters scaled to land count. "Differently named" filter on X is collapsed to total land count (typical cube games have unique land slots). |
| Environmental Scientist | {1}{G} | Creature — Human Druid | 2/2 | When this creature enters, you may search your library for a basic land card, reveal it, put it into your hand, then shuffle. | ✅ | Wired with `Effect::Search` over `IsBasicLand`. |
| Follow the Lumarets | {1}{G} | Sorcery |  | Infusion — Look at the top four cards of your library. You may reveal a creature or land card from among them and put it into your hand. If you gained life this turn, you may instead reveal two creature and/or land cards from among them and put them into your hand. Put the rest on the bottom of your library in a random order. | 🟡 | Push XV: wired as `If(LifeGainedThisTurnAtLeast(1)) → 2× RevealUntilFind(cap=4) → Hand : 1× RevealUntilFind(cap=4) → Hand`. Find filter = Creature OR Land. Approximations: misses go to graveyard (not bottom of library) — `RevealUntilFind`'s engine default; "you may reveal" optionality collapsed to always-do (declining would mill 4, strictly worse). |
| Germination Practicum | {3}{G}{G} | Sorcery — Lesson |  | Put two +1/+1 counters on each creature you control. / Paradigm (...) | 🟡 | Wired in `catalog::sets::sos::sorceries` as a `ForEach Creature & ControlledByYou → AddCounter +1/+1 ×2` fan-out. Paradigm rider omitted (no copy-spell-from-exile-at-upkeep primitive). |
| Glorious Decay | {1}{G} | Instant |  | Choose one — / • Destroy target artifact. / • Glorious Decay deals 4 damage to target creature with flying. / • Exile target card from a graveyard. Draw a card. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Hungry Graffalon | {3}{G} | Creature — Giraffe | 3/4 | Reach / Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) | ✅ | Push XXIX: Increment **now wired** via `shortcut::increment_self_plus_one()`. Cast a 5-mana spell with the Giraffe out → lands a +1/+1 counter (5 > 4 toughness). Tests: `hungry_graffalon_increment_*`. |
| Infirmary Healer // Stream of Life | {1}{G} // {X}{G} | Creature — Cat Cleric // Sorcery | 2/3 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Lumaret's Favor | {1}{G} | Instant |  | Infusion — When you cast this spell, copy it if you gained life this turn. You may choose new targets for the copy. / Target creature gets +2/+4 until end of turn. | ✅ | Push XVII: Infusion copy now wired via the new `Effect::CopySpell` primitive. The SelfSource cast trigger is gated by `Predicate::LifeGainedThisTurnAtLeast(You, 1)` — when you gained life this turn, the trigger fires and copies the spell. The trigger's filter is evaluated at trigger-creation time (cast-time), matching MTG's "intervening if" wording. |
| Mindful Biomancer | {1}{G} | Creature — Dryad Druid | 2/2 | When this creature enters, you gain 1 life. / {2}{G}: This creature gets +2/+2 until end of turn. Activate only once each turn. | ✅ | Wired in `catalog::sets::sos::creatures`; once-per-turn enforced engine-side. |
| Noxious Newt | {1}{G} | Creature — Salamander | 1/2 | Deathtouch / {T}: Add {G}. | ✅ | Wired in `catalog::sets::sos::creatures`. Now uses the new `Salamander` creature subtype. |
| Oracle's Restoration | {G} | Sorcery |  | Target creature you control gets +1/+1 until end of turn. You draw a card and gain 1 life. | ✅ | Wired in `catalog::sets::sos::sorceries`. |
| Pestbrood Sloth | {3}{G} | Creature — Plant Sloth | 4/4 | Reach / When this creature dies, create two 1/1 black and green Pest creature tokens with "Whenever this token attacks, you gain 1 life." | ✅ | Death trigger creates two Pests; each token now ships with the "gain 1 on attack" rider (via the new `TokenDefinition.triggered_abilities` field). |
| Planar Engineering | {3}{G} | Sorcery |  | Sacrifice two lands. Search your library for four basic land cards, put them onto the battlefield tapped, then shuffle. | ✅ | Wired in `catalog::sets::sos::sorceries` — `Sacrifice 2 lands` + `Repeat × 4 Search { IsBasicLand → Battlefield(tapped) }`. |
| Shopkeeper's Bane | {2}{G} | Creature — Badger Pest | 4/2 | Trample / Whenever this creature attacks, you gain 2 life. | ✅ | Wired in `catalog::sets::sos::creatures` with the new `Badger` creature subtype. |
| Slumbering Trudge | {X}{G} | Creature — Plant Beast | 6/6 | This creature enters with a number of stun counters on it equal to three minus X. If X is 2 or less, it enters tapped. (If a permanent with a stun counter would become untapped, remove one from it instead.) | ✅ | Wired in `catalog::sets::sos::creatures` using `Value::NonNeg(3-X)` stun counters. The "enters tapped if X≤2" half is implemented as always-tap-on-ETB; for X≥3 the no-stun-counter trudge naturally untaps next turn. |
| Snarl Song | {5}{G} | Sorcery |  | Converge — Create two 0/0 green and blue Fractal creature tokens. Put X +1/+1 counters on each of them and you gain X life, where X is the number of colors of mana spent to cast this spell. | ✅ | Wired in `catalog::sets::sos::sorceries`: two `CreateToken` calls each followed by `AddCounter(LastCreatedToken, +1/+1, ConvergedValue)`, plus `GainLife(ConvergedValue)`. Powered by `Value::ConvergedValue` (Rancorous Archaic) and `Selector::LastCreatedToken` (Fractal Anomaly). |
| Studious First-Year // Rampant Growth | {G} // {1}{G} | Creature — Bear Wizard // Sorcery | 1/1 | Front: vanilla 1/1 Bear Wizard. Back: search your library for a basic land card, put it onto the battlefield tapped, then shuffle. | ✅ | First non-land MDFC. Front face is wired as a vanilla 1/1 Bear Wizard at `{G}`; back face is `Rampant Growth`. Cast either face via `GameAction::CastSpell` (front) or the new `GameAction::CastSpellBack` (back, added in push X — mirror to `PlayLandBack` but for non-land back faces). The engine's `cast_spell_back_face` helper swaps the in-hand `definition` to the back face's before validating cost / type / effect, so the printed back-face Sorcery resolves end-to-end. |
| Tenured Concocter | {4}{G} | Creature — Troll Druid | 4/5 | Vigilance / Whenever this creature becomes the target of a spell or ability an opponent controls, you may draw a card. / Infusion — This creature gets +2/+0 as long as you gained life this turn. | 🟡 | Push (modern_decks): the Infusion self-pump (+2/+0 while you gained life this turn) **is now wired** via a new row in the `lifegain_selfpump_for_name` helper table — same compute-time injection pattern as Honor Troll / Ulna Alley Shopkeep. The "becomes the target" trigger is still omitted (no `BecameTarget` event). Tests: `tenured_concocter_is_vigilant_4_5_troll_druid`, `tenured_concocter_infusion_pumps_self_when_life_gained`. |
| Thornfist Striker | {2}{G} | Creature — Elf Druid | 3/3 | Ward {1} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.) / Infusion — Creatures you control get +1/+0 and have trample as long as you gained life this turn. | ✅ (was 🟡) | Push (modern_decks): the Infusion anthem (your creatures get +1/+0 and Trample while you gained life this turn) **is now wired** via the new `lifegain_anthem_for_name` helper-table + compute-time injection in `GameState::compute_battlefield`. Same pattern as the existing `tribal_anthem_for_name` and `lifegain_selfpump_for_name` helpers — adds one row per card instead of a new hardcoded `if name == "..."` branch. Layer 7b adds the +1/+0 mod; layer 6 grants Trample. Both affect `AffectedPermanents::All { controller: Some(card.controller), card_types: [Creature], exclude_source: false }` so the Striker itself is also pumped (matching the printed inclusive "creatures you control" wording). Ward {1} keyword is tagged; ward enforcement is still engine-wide ⏳. Tests: `thornfist_striker_is_3_3_with_ward_one`, `thornfist_striker_infusion_pumps_friendly_creatures_when_life_gained`, `thornfist_striker_infusion_does_not_buff_opponent_creatures`. |
| Topiary Lecturer | {2}{G} | Creature — Elf Druid | 1/2 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / {T}: Add an amount of {G} equal to this creature's power. | 🟡 | Wired with the new `ManaPayload::OfColor(Green, PowerOf(This))` primitive — fixed color, value-scaled count, so the {T}: Add G mana ability cleanly tracks `power-many G pips`. The Increment rider is omitted (mana-spent introspection on cast). |
| Vastlands Scavenger // Bind to Life | {1}{G}{G} // {4}{G} | Creature — Bear Druid // Instant | 4/4 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Wild Hypothesis | {X}{G} | Sorcery |  | Create a 0/0 green and blue Fractal creature token. Put X +1/+1 counters on it. / Surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | ✅ | Wired in `catalog::sets::sos::sorceries`: `CreateToken(fractal) + AddCounter(LastCreatedToken, +1/+1, XFromCost) + Surveil 2`. `Effect::Surveil` is a first-class primitive so this resolves end-to-end with no approximation. |
| Wildgrowth Archaic | {2/G}{2/G} | Creature — Avatar | 0/0 | Trample, reach / Converge — This creature enters with a +1/+1 counter on it for each color of mana spent to cast it. / Whenever you cast a creature spell, that creature enters with X additional +1/+1 counters on it, where X is the number of colors of mana spent to cast it. | 🟡 | Body wired in `catalog::sets::sos::creatures` (0/0 Avatar with Trample+Reach + Converge ETB AddCounter via `Value::ConvergedValue`). Hybrid `{2/G}` pips approximated as `{2}+{G}` per pip. The "creature spells you cast enter with X extra counters" rider is omitted pending per-cast converge introspection on the *just-cast* creature spell. Mono-G casts will die immediately to SBA (printed 0/0); 2-color casts land as 2/2. |
| Zimone's Experiment | {3}{G} | Sorcery |  | Look at the top five cards of your library. You may reveal up to two creature and/or land cards from among them, then put the rest on the bottom of your library in a random order. Put all land cards revealed this way onto the battlefield tapped and put all creature cards revealed this way into your hand. | 🟡 | Approximated as `RevealUntilFind { find: Creature, cap: 5, → Hand }` followed by a `Search { filter: IsBasicLand, → Battlefield(tapped) }`. The "look at top 5, choose ≤2 matching from among them" two-destination split isn't expressible (no "look + sort by category" primitive yet); the approximation pulls one creature into hand and one basic into play, which is the typical play pattern. |

## Prismari (Blue-Red)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Abstract Paintmage | {U}{U/R}{R} | Creature — Djinn Sorcerer | 2/2 | At the beginning of your first main phase, add {U}{R}. Spend this mana only to cast instant and sorcery spells. | 🟡 | Wired in `catalog::sets::sos::creatures` with a `StepBegins(PreCombatMain)/ActivePlayer` trigger that adds {U}{R} via `ManaPayload::Colors`. The spend restriction is omitted (no per-pip mana metadata). The hybrid `{U/R}` pip is approximated as `{U}`. |
| Colorstorm Stallion | {1}{U}{R} | Creature — Elemental Horse | 3/3 | Ward {1}, haste / Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If five or more mana was spent to cast that spell, create a token that's a copy of this creature. | 🟡 | Push XXIX: Body + Opus small body (+1/+1 EOT) wired via `shortcut::opus_trigger`. Big body (≥5 mana) "copy this creature" collapses to the same +1/+1 EOT pump pending a permanent-copy primitive (distinct from `Effect::CopySpell`). Test: `colorstorm_stallion_opus_small_body_pumps_one_one_eot`. |
| Elemental Mascot | {1}{U}{R} | Creature — Elemental Bird | 1/4 | Flying, vigilance / Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+0 until end of turn. If five or more mana was spent to cast that spell, exile the top card of your library. You may play that card until the end of your next turn. | 🟡 | Push XXIX: Body + Opus small body (+1/+0 EOT) wired via `shortcut::opus_trigger`. Big body (≥5 mana) "exile top + may play" collapses to the same +1/+0 EOT pump pending a cast-from-exile-with-timer primitive. Test: `elemental_mascot_opus_small_body_pumps_one_zero_eot`. |
| Prismari Charm | {U}{R} | Instant |  | Choose one — / • Surveil 2, then draw a card. / • Prismari Charm deals 1 damage to each of one or two targets. / • Return target nonland permanent to its owner's hand. | ✅ | 3-mode `ChooseMode`: Surveil 2 + draw / 1 dmg to creature-or-PW / bounce nonland to owner. Single-target collapse on mode 1 (printed "one or two targets" — multi-target gap). |
| Prismari, the Inspiration | {5}{U}{R} | Legendary Creature — Elder Dragon | 7/7 | Flying / Ward—Pay 5 life. / Instant and sorcery spells you cast have storm. (Whenever you cast an instant or sorcery spell, copy it for each spell cast before it this turn. You may choose new targets for the copies.) | 🟡 | Push XXVI: Body wired (7/7 Flying Legendary Elder Dragon with `Keyword::Ward(5)`). The "your IS spells have storm" static is omitted — storm grants need a per-cast trigger that fans out copies for each prior spell cast this turn. |
| Rapturous Moment | {4}{U}{R} | Sorcery |  | Draw three cards, then discard two cards. Add {U}{U}{R}{R}{R}. | ✅ | Wired in `catalog::sets::sos::sorceries`: Draw 3 + Discard 2 + AddMana with the printed UU/RRR pool. |
| Resonating Lute | {2}{U}{R} | Artifact |  | Lands you control have "{T}: Add two mana of any one color. Spend this mana only to cast instant and sorcery spells." / {T}: Draw a card. Activate only if you have seven or more cards in your hand. | 🟡 | Wired in `catalog::sets::sos::artifacts`. The {T}: Draw activation uses the new `ActivatedAbility.condition: Predicate::ValueAtLeast(HandSizeOf(You), 7)` gate — the engine rejects the activation cleanly when hand size < 7. Lands-grant tap-for-2 static is omitted (no spend-restricted-mana primitive yet — tracked in TODO.md). |
| Sanar, Unfinished Genius // Wild Idea | {U}{R} // {3}{U}{R} | Legendary Creature — Goblin Sorcerer // Sorcery | 0/4 |  | ✅ (was 🟡) | Push (modern_decks): back-face Wild Idea now resolves with each-player fan-out via `Selector::Player(PlayerRef::EachPlayer)` (printed "each player draws 3"). Same primitive as Wheel of Fortune. Test: `sanar_back_face_each_player_draws_three`. |
| Spectacle Summit |  | Land |  | This land enters tapped. / {T}: Add {U} or {R}. / {2}{U}{R}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands` via the shared `school_land` builder. |
| Spectacular Skywhale | {2}{U}{R} | Creature — Elemental Whale | 1/4 | Flying / Opus — Whenever you cast an instant or sorcery spell, this creature gets +3/+0 until end of turn. If five or more mana was spent to cast that spell, put three +1/+1 counters on this creature instead. | ✅ | Push XXIX: Opus rider **now wired** via `shortcut::opus_trigger`. Small body: +3/+0 EOT on the Skywhale. Big body (≥5 mana): three +1/+1 counters instead. Tests: `spectacular_skywhale_opus_small_body_pumps_three_zero_eot`, `spectacular_skywhale_opus_big_body_adds_three_counters`. |
| Splatter Technique | {1}{U}{U}{R}{R} | Sorcery |  | Choose one — / • Draw four cards. / • Splatter Technique deals 4 damage to each creature and planeswalker. | ✅ | Wired in `catalog::sets::sos::sorceries` as a `ChooseMode` with Draw 4 (mode 0) and DealDamage to `EachPermanent(Creature ∪ Planeswalker)` (mode 1). |
| Stadium Tidalmage | {2}{U}{R} | Creature — Djinn Sorcerer | 4/4 | Whenever this creature enters or attacks, you may draw a card. If you do, discard a card. | ✅ | Push XV → ✅ in push XXVIII: ETB + Attacks loot triggers use the `Effect::MayDo` primitive faithfully. The "you may" prompt asks the controller via `Decision::OptionalTrigger` — `AutoDecider` says no, `ScriptedDecider::new([Bool(true)])` for tests. Both Oracle clauses (ETB + attack) are fully wired. |
| Stress Dream | {3}{U}{R} | Instant |  | Stress Dream deals 5 damage to up to one target creature. Look at the top two cards of your library. Put one of those cards into your hand and the other on the bottom of your library. | 🟡 | 5-damage half wired in `catalog::sets::sos::instants`; the "look at top 2, choose 1 to hand and other to bottom" half is approximated as `scry 1 + draw 1` (no choose-which-zone primitive). |
| Traumatic Critique | {X}{U}{R} | Instant |  | Traumatic Critique deals X damage to any target. Draw two cards, then discard a card. | ✅ | Wired in `catalog::sets::sos::instants` (X damage via `Value::XFromCost` + draw 2 + discard 1 loot tail). |
| Vibrant Outburst | {U}{R} | Instant |  | Vibrant Outburst deals 3 damage to any target. Tap up to one target creature. | ✅ (was 🟡) | Push (modern_decks): two-target shape now wired via multi-target. Slot 0 = any target (creature/player/PW) takes 3 damage. Slot 1 = optional creature target gets tapped via `TargetFiltered`. Tests: `vibrant_outburst_deals_three_damage` (slot 0 only — bear dies to 3 dmg), `vibrant_outburst_taps_optional_second_target` (both slots — bear1 dies, bear2 tapped). |
| Visionary's Dance | {5}{U}{R} | Sorcery |  | Create two 3/3 blue and red Elemental creature tokens with flying. / {2}, Discard this card: Look at the top two cards of your library. Put one of them into your hand and the other into your graveyard. | ✅ | Wired in `catalog::sets::sos::sorceries` (uses the new `elemental_token()` helper). The `{2}, Discard this card:` activation from hand is omitted (engine activation walker is battlefield-only). |
| Zaffai and the Tempests | {5}{U}{R} | Legendary Creature — Human Bard Sorcerer | 5/7 | Once during each of your turns, you may cast an instant or sorcery spell from your hand without paying its mana cost. | 🟡 | Push XVI: body-only wire (5/7 Legendary Human Bard Sorcerer). The "once per turn cast IS for free" rider is omitted (no per-turn alt-cost-grant primitive — would need `Player.zaffai_free_cast_used: bool` consumed by an alternative-cost path keyed off a cast-time static). |

## Witherbloom (Black-Green)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Blech, Loafing Pest | {1}{B}{G} | Legendary Creature — Pest | 3/4 | Whenever you gain life, put a +1/+1 counter on each Pest, Bat, Insect, Snake, and Spider you control. | ✅ | `LifeGained` (YourControl) trigger + `ForEach` fan-out filtered to Pest ∪ Bat ∪ Insect ∪ Snake ∪ Spider. |
| Bogwater Lumaret | {B}{G} | Creature — Spirit Frog | 2/2 | Whenever this creature or another creature you control enters, you gain 1 life. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Cauldron of Essence | {1}{B}{G} | Artifact |  | Whenever a creature you control dies, each opponent loses 1 life and you gain 1 life. / {1}{B}{G}, {T}, Sacrifice a creature: Return target creature card from your graveyard to the battlefield. Activate only as a sorcery. | ✅ | Death drain trigger (`CreatureDied/AnotherOfYours`) + `{1}{B}{G},{T},Sac:` reanimation activation, sorcery-speed gated. 🔍 needs review (oracle previously truncated). |
| Dina's Guidance | {1}{B}{G} | Sorcery |  | Search your library for a creature card, reveal it, put it into your hand or graveyard, then shuffle. | ✅ | Push XXVIII: now a 2-mode `ChooseMode` — mode 0 search → hand, mode 1 search → graveyard. AutoDecider defaults to mode 0 (hand) for the unguided bot; a Lorehold / Witherbloom reanimator deck picks mode 1. Tests: `dinas_guidance_searches_creature_to_hand`, `dinas_guidance_mode_one_sends_creature_to_graveyard`. |
| Essenceknit Scholar | {B}{B/G}{G} | Creature — Dryad Warlock | 3/1 | When this creature enters, create a 1/1 black and green Pest creature token with "Whenever this token attacks, you gain 1 life." / At the beginning of your end step, if a creature died under your control this turn, draw a card. | ✅ | ETB Pest token (with on-attack lifegain rider) + end-step gated draw via the new `Predicate::CreaturesDiedThisTurnAtLeast` (backed by `Player.creatures_died_this_turn`). Hybrid `{B/G}` pip approximated as `{B}` (cost: `{B}{B}{G}`). New `CreatureType::Dryad`. |
| Grapple with Death | {1}{B}{G} | Sorcery |  | Destroy target artifact or creature. You gain 1 life. | ✅ | Wired in `catalog::sets::sos::sorceries`. |
| Lluwen, Exchange Student // Pest Friend | {2}{B}{G} // {B/G} | Legendary Creature — Elf Druid // Sorcery | 3/4 |  | ✅ (was 🟡) | Push XXXVIII (doc-sync): front 3/4 Legendary Elf Druid vanilla + back-face Sorcery `Pest Friend` creates one Pest token (with the on-attack lifegain rider via the shared `pest_token()` helper). The hybrid `{B/G}` pip is approximated as `{B}` — same convention as Essenceknit Scholar and Practiced Scrollsmith. The body fully wires the printed effects; remaining gap is the engine-wide hybrid-pip primitive (tracked in TODO.md). |
| Mind Roots | {1}{B}{G} | Sorcery |  | Target player discards two cards. Put up to one land card discarded this way onto the battlefield tapped under your control. | ✅ (was 🟡) | Push (modern_decks): the "land discarded → battlefield tapped" rider is **now wired** via the new `Selector::DiscardedThisResolution { filter }` primitive + `GameState.discarded_card_ids_this_resolution` tracker. The Discard handler stamps each discarded card's id onto the list; Mind Roots's body then walks the list, filters by `HasCardType(Land)`, takes at most one (`Selector::Take { count: 1 }`), and moves it to the caster's battlefield via `ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true }`. The `PlayerRef::You` resolution was also fixed in `resolve_zonedest_player` — previously a `You`-anchored ZoneDest on a graveyard-source move would re-bind to the gy owner's seat (the opp), so the stolen land would re-enter under the opp's control. Now `You` is flattened to `Seat(ctx.controller)` (the caster) before `place_card_in_dest` runs. Tests: `mind_roots_makes_opponent_discard_two`, `mind_roots_steals_a_discarded_land_to_caster_battlefield`, `mind_roots_does_not_steal_a_nonland_discarded_card`. |
| Old-Growth Educator | {2}{B}{G} | Creature — Treefolk Druid | 4/4 | Vigilance, reach / Infusion — When this creature enters, put two +1/+1 counters on it if you gained life this turn. | ✅ | Wired with the new `Predicate::LifeGainedThisTurnAtLeast` Infusion gate on the ETB trigger. |
| Pest Mascot | {1}{B}{G} | Creature — Pest Ape | 2/3 | Trample / Whenever you gain life, put a +1/+1 counter on this creature. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Professor Dellian Fel | {2}{B}{G} | Legendary Planeswalker — Dellian | [5] | +2: You gain 3 life. / 0: You draw a card and lose 1 life. / −3: Destroy target creature. / −6: You get an emblem with "Whenever you gain life, target opponent loses that much life." | 🟡 | New `PlaneswalkerSubtype::Dellian` + 5 base loyalty. +2 (gain 3 life), 0 (draw 1 / lose 1 life), -3 (destroy target creature) all wired faithfully. The -6 emblem ult is omitted (no emblem zone yet). |
| Root Manipulation | {3}{B}{G} | Sorcery |  | Until end of turn, creatures you control get +2/+2 and gain menace and "Whenever this creature attacks, you gain 1 life." (A creature with menace can't be blocked except by two or more creatures.) | 🟡 | `ForEach(Creature & ControlledByYou) → PumpPT(+2/+2 EOT) + GrantKeyword(Menace EOT)`. The "whenever this creature attacks → gain 1 life" rider is omitted (no transient-trigger-grant primitive yet). |
| Teacher's Pest | {B}{G} | Creature — Skeleton Pest | 1/1 | Menace (This creature can't be blocked except by two or more creatures.) / Whenever this creature attacks, you gain 1 life. / {B}{G}: Return this card from your graveyard to the battlefield tapped. | ✅ | Push XVII: graveyard-recursion activation wired via the new `from_graveyard: bool` field. Menace + attacks-gain-1 trigger unchanged. Cost `{B}{G}` + effect `Move(Self → Battlefield(You, tapped))`. |
| Titan's Grave |  | Land |  | This land enters tapped. / {T}: Add {B} or {G}. / {2}{B}{G}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands`. |
| Vicious Rivalry | {2}{B}{G} | Sorcery |  | As an additional cost to cast this spell, pay X life. / Destroy all artifacts and creatures with mana value X or less. | ✅ | Wired in `catalog::sets::sos::sorceries` — `LoseLife X` (approximating the additional cost) + `ForEach(Creature ∨ Artifact).If(ManaValueOf ≤ X) → Destroy`. |
| Witherbloom Charm | {B}{G} | Instant |  | Choose one — / • You may sacrifice a permanent. If you do, draw two cards. / • You gain 5 life. / • Destroy target nonland permanent with mana value 2 or less. | ✅ | Push XV → ✅ in push XXVIII: mode 0 (sacrifice → draw 2) wrapped in `Effect::MayDo` — picking mode 0 then declining the sac-prompt keeps everything else stable. Mode 1 (gain 5) and mode 2 (destroy mv≤2) are direct primitives. All three printed modes are wired faithfully. |
| Witherbloom, the Balancer | {6}{B}{G} | Legendary Creature — Elder Dragon | 5/5 | Affinity for creatures (This spell costs {1} less to cast for each creature you control.) / Flying, deathtouch / Instant and sorcery spells you cast have affinity for creatures. | 🟡 | Body wired in `catalog::sets::sos::creatures` with the new `CreatureType::Elder` subtype. Both Affinity-for-creatures cost-reduction clauses are omitted (no per-cast cost reduction whose discount scales off caster's permanent count — tracked in TODO.md). |

## Silverquill (White-Black)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Abigale, Poet Laureate // Heroic Stanza | {1}{W}{B} // {1}{W/B} | Legendary Creature — Bird Bard // Sorcery | 2/3 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Conciliator's Duelist | {W}{W}{B}{B} | Creature — Kor Warlock | 4/3 | When this creature enters, draw a card. Each player loses 1 life. / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, exile up to one target creature. Return that card to the battlefield under its owner's control at the beginning of the next end step. | ✅ (was 🟡) | Push (modern_decks): the "return at next end step" delayed rider is **now wired** via an extension to `Effect::DelayUntil` that falls back to `Selector::CastSpellTarget(0)` (the cast spell's target on the stack) when `ctx.targets` is empty. Repartee body becomes `Seq(Exile(CastSpellTarget(0)) + DelayUntil(NextEndStep, Move→Battlefield(OwnerOf)))`; the captured target survives through the delayed-trigger fire, so the exiled creature returns under its owner's control at the next end step. Tests: `conciliators_duelist_etb_draws_and_each_player_loses_one`, `conciliators_duelist_repartee_exiles_target`, `conciliators_duelist_repartee_returns_target_at_end_step`. |
| Fix What's Broken | {2}{W}{B} | Sorcery |  | As an additional cost to cast this spell, pay X life. / Return each artifact and creature card with mana value X from your graveyard to the battlefield. | ✅ (was 🟡) | Push (modern_decks doc-sync): Pay-X-life folds into resolution as `Effect::LoseLife(XFromCost)` (Vicious-Rivalry pattern — the auto-decider always commits to the cast, so cost-vs-resolution timing is gameplay-invariant). The MV=X gate on the graveyard walk uses `Predicate::ValueEquals(ManaValueOf(TriggerSource), XFromCost)` (the engine's `ValueEquals` primitive — adopted after the doc note was last revised). Returns each matching artifact/creature card via `ForEach(EachMatching(Graveyard(You), Artifact ∨ Creature)) + Move → Battlefield`. Test: `fix_whats_broken_returns_mana_value_x_artifact_from_graveyard`. |
| Forum of Amity |  | Land |  | This land enters tapped. / {T}: Add {W} or {B}. / {2}{W}{B}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands`. |
| Imperious Inkmage | {1}{W}{B} | Creature — Orc Warlock | 3/3 | Vigilance / When this creature enters, surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Inkling Mascot | {W}{B} | Creature — Inkling Cat | 2/2 | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, this creature gains flying until end of turn. Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Repartee trigger grants Flying (EOT) on `Selector::This` + Surveil 1. |
| Killian's Confidence | {W}{B} | Sorcery |  | Target creature gets +1/+1 until end of turn. Draw a card. / Whenever one or more creatures you control deal combat damage to a player, you may pay {W/B}. If you do, return this card from your graveyard to your hand. | ✅ | Push XXV: Body (+1/+1 EOT + draw 1) was wired; the graveyard-recursion trigger is now also wired via the new `EventScope::FromYourGraveyard` extension on `fire_combat_damage_to_player_triggers`. The combat-damage trigger fires off the graveyard-resident card; `Effect::MayPay { mana_cost: {W} }` asks the controller yes/no and on yes returns `Selector::This` to its owner's hand via `Move`. Hybrid {W/B} approximated as {W} (matches Practiced Scrollsmith). |
| Moment of Reckoning | {3}{W}{W}{B}{B} | Sorcery |  | Choose up to four. You may choose the same mode more than once. / • Destroy target nonland permanent. / • Return target nonland permanent card from your graveyard to the battlefield. | 🟡 | Wired in `catalog::sets::sos::sorceries` as a 2-mode `ChooseMode`. The "choose up to four / same mode more than once" rider is collapsed to "pick one mode and target one permanent" — same-resolution multi-mode replay needs an engine primitive. |
| Nita, Forum Conciliator | {1}{W}{B} | Legendary Creature — Human Advisor | 2/3 | Whenever you cast a spell you don't own, put a +1/+1 counter on each creature you control. / {2}, Sacrifice another creature: Exile target instant or sorcery card from an opponent's graveyard. You may cast it this turn, and mana of any type can be spent to cast that spell. If that spell would be put into a graveyard, exile it instead. Activate only as a sorcery. | 🟡 | Push XXV: Body wired (2/3 Legendary Human Advisor). The "cast a spell you don't own" trigger + cast-from-opp-graveyard activated ability are omitted — engine has no owned-vs-controlled-spell predicate and no cast-from-graveyard-without-paying for arbitrary cards. |
| Render Speechless | {2}{W}{B} | Sorcery |  | Target opponent reveals their hand. You choose a nonland card from it. That player discards that card. / Put two +1/+1 counters on up to one target creature. | ✅ (was 🟡) | Push (modern_decks): two-slot multi-target shape. Slot 0 = target opponent (reveal hand + chosen-discard via `DiscardChosen`). Slot 1 = optional creature target gets two +1/+1 counters via `TargetFiltered`. Tests: `render_speechless_discards_and_pumps`, `render_speechless_can_target_opponent_without_creature` (slot 0 only — discard-only play). |
| Scolding Administrator | {W}{B} | Creature — Dwarf Cleric | 2/2 | Menace (This creature can't be blocked except by two or more creatures.) / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, put a +1/+1 counter on this creature. / When this creature dies, if it had counters on it, put those counters on up to one target creature. | ✅ (was 🟡) | Push (modern_decks doc-sync): all three abilities already wired in `catalog::sets::sos::creatures`. Menace + Dwarf/Cleric subtypes + Repartee +1/+1 counter ship vanilla; the dies-trigger reads the Admin's accumulated +1/+1 counters via `Value::CountersOn(This)` cross-zone (battlefield → graveyard at die-time) and transfers them to a target creature via `Effect::If { cond: ValueAtLeast(CountersOn(This, +1/+1), 1), AddCounter(target Creature, amount: CountersOn(This, +1/+1)) }`. Tests: `scolding_administrator_repartee_pumps_self`, `scolding_administrator_has_menace`, `scolding_administrator_transfers_counters_on_death`. |
| Silverquill Charm | {W}{B} | Instant |  | Choose one — / • Put two +1/+1 counters on target creature. / • Exile target creature with power 2 or less. / • Each opponent loses 3 life and you gain 3 life. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Silverquill, the Disputant | {2}{W}{B} | Legendary Creature — Elder Dragon | 4/4 | Flying, vigilance / Each instant and sorcery spell you cast has casualty 1. (As you cast that spell, you may sacrifice a creature with power 1 or greater. When you do, copy the spell and you may choose new targets for the copy.) | 🟡 | Push XXV: Body wired (4/4 Legendary Elder Dragon Flying+Vigilance). The casualty-1 grant on instant/sorcery casts is omitted — engine has no static "spells of type X gain casualty N" primitive, and no Casualty keyword yet. |
| Snooping Page | {1}{W}{B} | Creature — Human Cleric | 2/3 | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, this creature can't be blocked this turn. / Whenever this creature deals combat damage to a player, you draw a card and lose 1 life. | ✅ | Repartee grants `Keyword::Unblockable` (EOT) on the source; combat-damage trigger wired (draw + lose 1). |
| Social Snub | {1}{W}{B} | Sorcery |  | When you cast this spell while you control a creature, you may copy this spell. / Each player sacrifices a creature of their choice. Each opponent loses 1 life and you gain 1 life. | 🟡 | Push XVII: cast-IS-while-you-control-a-creature copy now wired via the new `Effect::CopySpell` primitive. Trigger filter uses `Predicate::SelectorExists(EachPermanent(Creature & ControlledByYou))`. Main effect: each-player-sac + drain 1. The "of their choice" sac picker is collapsed to auto-pick (each player's auto-decider selects). |
| Stirring Honormancer | {2}{W}{W/B}{B} | Creature — Rhino Bard | 4/5 | When this creature enters, look at the top X cards of your library, where X is the number of creatures you control. Put one of those cards into your hand and the rest into your graveyard. | ✅ | Wired in `catalog::sets::sos::creatures` via `Effect::RevealUntilFind` (find: Creature, cap: count of creatures you control, misses go to graveyard). The hybrid `{W/B}` pip is approximated as `{W}` so cost is `{2}{W}{W}{B}`. |

## Quandrix (Green-Blue)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Applied Geometry | {2}{G}{U} | Sorcery |  | Create a token that's a copy of target non-Aura permanent you control, except it's a 0/0 Fractal creature in addition to its other types. Put six +1/+1 counters on it. | 🟡 | Push XXVI: Body wired as "mint a 0/0 Fractal token + 6 +1/+1 counters" — collapses the "copy a non-Aura permanent" half to a vanilla Fractal mint (no permanent-copy primitive). Net play pattern is a 6/6 Fractal for 4 mana. |
| Berta, Wise Extrapolator | {2}{G}{U} | Legendary Creature — Frog Druid | 1/4 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / Whenever one or more +1/+1 counters are put on Berta, add one mana of any color. / {X}, {T}: Create a 0/0 green and blue Fractal creature token and put X +1/+1 counters on it. | 🟡 | Wired in `catalog::sets::sos::creatures`. Counter-add → AnyOneColor mana trigger uses `EventKind::CounterAdded(PlusOnePlusOne)` + `EventScope::SelfSource` (powered by the new SelfSource → CounterAdded engine recognition). X-cost `{X}{T}: Fractal token + X +1/+1 counters` activation is wired but X resolves to 0 today (engine has no X-cost activated ability path; the X-from-cost path zeroes for activations). Increment rider omitted (mana-spent-on-cast introspection missing). |
| Cuboid Colony | {G}{U} | Creature — Insect | 1/1 | Flash / Flying, trample / Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) | ✅ | Push XXIX: Increment **now wired** via `shortcut::increment_self_plus_one()`. Tests: `cuboid_colony_increment_lands_counter_on_two_mana_cast`, `cuboid_colony_increment_skips_one_mana_cast`. |
| Embrace the Paradox | {3}{G}{U} | Instant |  | Draw three cards. You may put a land card from your hand onto the battlefield tapped. | ✅ | Push XVI: draw 3 + `MayDo` rider that picks (at most) one land from hand via `Selector::one_of(CardsInZone(Hand, Land))` and moves it to bf tapped. AutoDecider answers "no" by default; `ScriptedDecider::new([Bool(true)])` exercises the paid path in tests. |
| Fractal Mascot | {4}{G}{U} | Creature — Fractal Elk | 6/6 | Trample / When this creature enters, tap target creature an opponent controls. Put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) | ✅ | Wired in `catalog::sets::sos::creatures` (trample + ETB tap+stun). |
| Fractal Tender | {3}{G}{U} | Creature — Elf Wizard | 3/3 | Ward {2} / Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / At the beginning of each end step, if you put a counter on this creature this turn, create a 0/0 green and blue Fractal creature token and put three +1/+1 counters on it. | 🟡 | Body + `Keyword::Ward(2)` wired in `catalog::sets::sos::creatures`. Increment trigger and end-step Fractal-with-counters payoff are both omitted (Increment requires mana-spent introspection on cast; the end-step trigger needs a "did this creature gain a counter this turn" per-permanent flag the engine doesn't track yet). |
| Geometer's Arthropod | {G}{U} | Creature — Fractal Crab | 1/4 | Whenever you cast a spell with {X} in its mana cost, look at the top X cards of your library. Put one of them into your hand and the rest on the bottom of your library in a random order. | ✅ | Push XVI: trigger fully wired via the new `Predicate::CastSpellHasX` + `RevealUntilFind { cap: XFromCost, to: Hand }`. Misses go to graveyard (engine default for `RevealUntilFind`); the printed "rest to bottom random order" rider stays approximated since the engine has no random-bottom primitive. |
| Growth Curve | {G}{U} | Sorcery |  | Put a +1/+1 counter on target creature you control, then double the number of +1/+1 counters on that creature. | ✅ | Wired in `catalog::sets::sos::sorceries`: AddCounter(+1) then AddCounter(`Value::CountersOn`) faithfully doubles. |
| Mind into Matter | {X}{G}{U} | Sorcery |  | Draw X cards. Then you may put a permanent card with mana value X or less from your hand onto the battlefield tapped. | 🟡 | Draw X wired in `catalog::sets::sos::sorceries` via `Value::XFromCost`. The "may put a permanent ≤ X tapped" half is omitted (no hand-to-battlefield mana-value-gated primitive yet). |
| Paradox Gardens |  | Land |  | This land enters tapped. / {T}: Add {G} or {U}. / {2}{G}{U}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands`. |
| Paradox Surveyor | {G}{G/U}{U} | Creature — Elf Druid | 3/3 | Reach / When this creature enters, look at the top five cards of your library. You may reveal a land card or a card with {X} in its mana cost from among them and put it into your hand. Put the rest on the bottom of your library in a random order. | ✅ | Push XVI: filter promoted to `Land OR HasXInCost` via the new `SelectionRequirement::HasXInCost` primitive — exact-printed reveal filter. Hybrid `{G/U}` pip stays approximated as `{G}` (cost: `{G}{G}{U}`). Misses go to graveyard. |
| Proctor's Gaze | {2}{G}{U} | Instant |  | Return up to one target nonland permanent to its owner's hand. Search your library for a basic land card, put it onto the battlefield tapped, then shuffle. | ✅ | Wired in `catalog::sets::sos::instants`: bounce target nonland to owner's hand, then `Search { filter: IsBasicLand, to: Battlefield(tapped) }`. |
| Pterafractyl | {X}{G}{U} | Creature — Dinosaur Fractal | 1/0 | Flying / This creature enters with X +1/+1 counters on it. / When this creature enters, you gain 2 life. | ✅ (was 🟡) | Push (modern_decks): printed 1/0 body now lands faithfully via the new `CardDefinition.enters_with_counters` field (CR 614.12 replacement). The X +1/+1 counters arrive **before** SBA / ETB, so a 1/0 + X counters body survives at X≥1 (printed Oracle exact). The toughness-bump workaround (1/0→1/1) is retired. Tests: `pterafractyl_etb_with_x_counters_and_gains_two_life`, `pterafractyl_cr_614_12_zero_toughness_base_survives_etb_via_enters_with`. |
| Quandrix Charm | {G}{U} | Instant |  | Choose one — / • Counter target spell unless its controller pays {2}. / • Destroy target enchantment. / • Target creature has base power and toughness 5/5 until end of turn. | ✅ (was 🟡) | Push XXXIII: all three modes wired. Mode 2 promoted from the `PumpPT +3/+3` approximation to a proper layer-7b base-P/T rewrite via `Effect::SetBasePT { power: 5, toughness: 5 }` (the primitive added in push XXXII for Square Up). Counters and +N/+M modifications stack on top per CR 613.7c-f. Test: `quandrix_charm_mode_2_setbasept_layers_under_counter` (2/2 with a +1/+1 counter → 6/6). |
| Quandrix, the Proof | {4}{G}{U} | Legendary Creature — Elder Dragon | 6/6 | Flying, trample / Cascade (When you cast this spell, exile cards from the top of your library until you exile a nonland card that costs less. You may cast it without paying its mana cost. Put the exiled cards on the bottom in a random order.) / Instant and sorcery spells you cast from your hand have cascade. | 🟡 | Push XXVIII: ⏳ → 🟡. Body wired faithfully — 6/6 Legendary Elder Dragon with Flying + Trample. The Cascade keyword and the IS-grant-cascade static are still ⏳ (no Cascade keyword primitive, no cast-from-exile-without-paying pipeline). At raw stats this is a 6-mana 6/6 flying trampler — strong finisher even without Cascade. |
| Tam, Observant Sequencer // Deep Sight | {2}{G}{U} // {G}{U} | Legendary Creature — Gorgon Wizard // Sorcery | 4/3 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|

## Lorehold (Red-White)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Ark of Hunger | {2}{R}{W} | Artifact |  | Whenever one or more cards leave your graveyard, this artifact deals 1 damage to each opponent and you gain 1 life. / {T}: Mill a card. You may play that card this turn. | 🟡 | Wired against the `EventKind::CardLeftGraveyard` event — drain 1 (1 to each opp + you gain 1) per gy-leave emission. The {T}: Mill activation is wired faithfully; the "may play that card this turn" rider is omitted (same gap as Tablet of Discovery, Suspend Aggression). |
| Aziza, Mage Tower Captain | {R}{W} | Legendary Creature — Djinn Sorcerer | 2/2 | Whenever you cast an instant or sorcery spell, you may tap three untapped creatures you control. If you do, copy that spell. You may choose new targets for the copy. | ✅ | Push XVII: cast-IS copy rider now wired via the new `Effect::CopySpell` primitive. The "may tap three" cost uses `Effect::MayDo` (yes/no decision) + `Effect::Tap` with `Selector::Take(Untapped Creatures, 3)` + `Effect::CopySpell { what: Selector::TriggerSource }`. The picker may include Aziza herself in the tap-three pool. |
| Borrowed Knowledge | {2}{R}{W} | Sorcery |  | Choose one — / • Discard your hand, then draw cards equal to the number of cards in target opponent's hand. / • Discard your hand, then draw cards equal to the number of cards discarded this way. | ✅ (was 🟡) | Push (modern_decks doc-sync): both modes wired faithfully. Mode 0 = discard hand → draw target opp's hand size via `Value::HandSizeOf(PlayerRef::Target(0))`. Mode 1 = discard hand → draw cards equal to number actually discarded via `Value::CardsDiscardedThisEffect` (per-resolution counter bumped by every `GameEvent::CardDiscarded` emission). |
| Colossus of the Blood Age | {4}{R}{W} | Artifact Creature — Construct | 6/6 | When this creature enters, it deals 3 damage to each opponent and you gain 3 life. / When this creature dies, discard any number of cards, then draw that many cards plus one. | ✅ (was 🟡) | Push (modern_decks): death trigger now uses the new `Effect::DiscardAnyNumber` primitive — player chooses 0-handsize cards to discard, then draws `CardsDiscardedThisEffect + 1`. AutoDecider picks 0 (draw 1); ScriptedDecider can discard any subset for the full "discard N draw N+1" cycle. Tests: `colossus_etb_drains_three_each_opponent`, `colossus_dies_loots_one_for_two` (AutoDecider 0+1 path), `colossus_dies_discard_three_draws_four_via_scripted_decider` (scripted 3+4 path). |
| Fields of Strife |  | Land |  | This land enters tapped. / {T}: Add {R} or {W}. / {2}{R}{W}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands`. |
| Hardened Academic | {R}{W} | Creature — Bird Cleric | 2/1 | Flying, haste / Discard a card: This creature gains lifelink until end of turn. / Whenever one or more cards leave your graveyard, put a +1/+1 counter on target creature you control. | ✅ | All three abilities wired. The cards-leave-graveyard trigger uses the new `EventKind::CardLeftGraveyard` event (per-card emission; "one or more" rider is naturally per-card). |
| Kirol, History Buff // Pack a Punch | {R}{W} // {1}{R}{W} | Legendary Creature — Vampire Cleric // Sorcery | 2/3 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Lorehold Charm | {R}{W} | Instant |  | Choose one — / • Each opponent sacrifices a nontoken artifact of their choice. / • Return target artifact or creature card with mana value 2 or less from your graveyard to the battlefield. / • Creatures you control get +1/+1 and gain trample until end of turn. | ✅ | Wired in `catalog::sets::sos::instants` — all three modes wired with existing primitives (`Sacrifice`, `Move from gy`, `ForEach(Creature) → PumpPT`). |
| Lorehold, the Historian | {3}{R}{W} | Legendary Creature — Elder Dragon | 5/5 | Flying, haste / Each instant and sorcery card in your hand has miracle {2}. (You may cast a card for its miracle cost when you draw it if it's the first card you drew this turn.) / At the beginning of each opponent's upkeep, you may discard a card. If you do, draw a card. | 🟡 | Body-only wire (5/5 Flying+Haste Legendary Elder Dragon, R/W). Miracle grant on instants/sorceries in hand is omitted (no miracle/alt-cost-on-draw primitive); per-opp-upkeep loot trigger omitted (no opp-upkeep step trigger that fires for non-active player). The vanilla finisher is the most impactful printed clause — both omitted clauses are tracked in TODO.md. |
| Molten Note | {X}{R}{W} | Sorcery |  | Molten Note deals damage to target creature equal to the amount of mana spent to cast this spell. Untap all creatures you control. / Flashback {6}{R}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ (was 🟡) | Push (modern_decks): damage now reads `Value::CastSpellManaSpent` (the actual mana paid for the cast), matching the printed "amount of mana spent" Oracle exactly — at X=2 the spell deals 4 damage (X + R + W), so a 4-toughness creature dies. The prior `Value::XFromCost` undercounted by 2 (the {R}{W} pips). Untap all your creatures wired. Flashback {6}{R}{W} via `Keyword::Flashback` — when flashbacked the cast reads mana_spent = 8 (6 + R + W) so the damage scales correctly. Tests: `molten_note_deals_x_damage_and_untaps_your_creatures`, `molten_note_damage_equals_total_mana_spent_not_just_x`, `molten_note_has_flashback_keyword`. |
| Practiced Scrollsmith | {R}{R/W}{W} | Creature — Dwarf Cleric | 3/2 | First strike / When this creature enters, exile target noncreature, nonland card from your graveyard. Until the end of your next turn, you may cast that card. | 🟡 | Wired in `catalog::sets::sos::creatures`. ETB now exiles **exactly one** matching noncreature/nonland card in the controller's graveyard via the new `Selector::Take(_, 1)` primitive (push X) — closer to the printed "target one card" semantics; the prior implementation exiled every matching card. The hybrid `{R/W}` pip is approximated as `{R}` (cost: `{R}{R}{W}`). The "may cast until next turn" rider is omitted (no cast-from-exile-with-time-limit primitive). |
| Pursue the Past | {R}{W} | Sorcery |  | You gain 2 life. You may discard a card. If you do, draw two cards. / Flashback {2}{R}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Push XV → ✅ in push XXVIII: gain 2 + the discard+draw chain wrapped in `Effect::MayDo` so the printed "you may discard" optionality is honored. Flashback wired via `Keyword::Flashback`. The lifegain half always resolves; the loot half is opt-in. All Oracle clauses wired. |
| Spirit Mascot | {R}{W} | Creature — Spirit Ox | 2/2 | Whenever one or more cards leave your graveyard, put a +1/+1 counter on this creature. | ✅ | Wired against the new `EventKind::CardLeftGraveyard` event. Trigger fires per-card emission (the printed "one or more" wording is approximated per-card). |
| Startled Relic Sloth | {2}{R}{W} | Creature — Sloth Beast | 4/4 | Trample, lifelink / At the beginning of combat on your turn, exile up to one target card from a graveyard. | ✅ | Wired in `catalog::sets::sos::creatures` (trample + lifelink + begin-combat exile-from-GY trigger; same shape as Ascendant Dustspeaker's combat trigger). Sloth subtype bridged through Beast (no Sloth creature type yet). |
| Suspend Aggression | {1}{R}{W} | Instant |  | Exile target nonland permanent and the top card of your library. For each of those cards, its owner may play it until the end of their next turn. | 🟡 | Wired in `catalog::sets::sos::instants` as a `Seq` of two `Move → Exile` calls (target nonland permanent + caster's top of library). `move_card_to` was extended to walk libraries when locating the source card so the top-of-library exile resolves end-to-end. The "may play those cards until next end step" rider is omitted (no per-card "may-play-from-exile-until-EOT" primitive). |
| Wilt in the Heat | {2}{R}{W} | Instant |  | This spell costs {2} less to cast if one or more cards left your graveyard this turn. / Wilt in the Heat deals 5 damage to target creature. If that creature would die this turn, exile it instead. | 🟡 | Wired as a 5-damage-to-target-creature spell. The cost-reduction-when-cards-left-gy clause is omitted (no `StaticEffect::CostReduction` variant gated on a per-turn tally). The "if it would die, exile instead" replacement is omitted (no damage-replacement primitive). |

## Colorless

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Biblioplex Tomekeeper | {4} | Artifact Creature — Construct | 3/4 | When this creature enters, choose up to one — / • Target creature becomes prepared. (Only creatures with prepare spells can become prepared.) / • Target creature becomes unprepared. | 🟡 | Push XXV: Body wired (3/4 Construct artifact creature). The Prepare toggle is omitted — engine has no Prepare keyword nor a prepared-state flag (same gap as Skycoach Waypoint). |
| Diary of Dreams | {2} | Artifact — Book |  | Whenever you cast an instant or sorcery spell, put a page counter on this artifact. / {5}, {T}: Draw a card. This ability costs {1} less to activate for each page counter on this artifact. | 🟡 | Page-counter accrual on instant/sorcery cast (counter type approximated as Charge — engine has no Page counter) + flat {5},{T} draw. The page-counter-scaled cost reduction is omitted (no self-counter cost-reduction primitive). |
| Great Hall of the Biblioplex |  | Land |  | {T}: Add {C}. / {T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast an instant or sorcery spell. / {5}: If this land isn't a creature, it becomes a 2/4 Wizard creature with "Whenever you cast an instant or sorcery spell, this creature gets +1/+0 until end of turn." It's still a land. | 🟡 | Push XV: legendary colorless utility land. `{T}: Add {C}` faithful + `{T}, Pay 1 life: Add one mana of any color` via the new `ActivatedAbility.life_cost: u32` field — the effect is a pure mana ability (`AddMana(AnyOneColor 1)`) so it resolves immediately without going on the stack. The `{5}: becomes 2/4 Wizard creature` clause is omitted (no land-becomes-creature primitive — same gap as Mishra's Factory). The spend-restriction rider on the rainbow ability is omitted (no per-pip mana metadata yet). |
| Mage Tower Referee | {2} | Artifact Creature — Construct | 2/1 | Whenever you cast a multicolored spell, put a +1/+1 counter on this creature. | ✅ | Wired in `catalog::sets::sos::creatures` with a `SpellCast/YourControl` trigger filtered on `EntityMatches(TriggerSource, Multicolored)` — uses the new `SelectionRequirement::Multicolored` predicate (≥ 2 distinct colored pips, hybrid both halves, Phyrexian colored side). Mono-color and colorless casts don't bump the Referee. |
| Page, Loose Leaf | {2} | Legendary Artifact Creature — Construct | 0/2 | {T}: Add {C}. / Grandeur — Discard another card named Page, Loose Leaf: Reveal cards from the top of your library until you reveal an instant or sorcery card. Put that card into your hand and the rest on the bottom of your library in a random order. | 🟡 | Body wired (0/2 Legendary Construct artifact creature) + `{T}: Add {C}` mana ability via the shared `tap_add_colorless()` helper. Grandeur (discard-named-this-card-as-cost activation) omitted. |
| Petrified Hamlet |  | Land |  | When this land enters, choose a land card name. / Activated abilities of sources with the chosen name can't be activated unless they're mana abilities. / Lands with the chosen name have "{T}: Add {C}." / {T}: Add {C}. | 🟡 | Mana ability `{T}: Add {C}` wired via the new shared `tap_add_colorless()` helper in `catalog::sets`. The "choose a land card name" prompt + name-keyed lock-out static + name-keyed grant of `{T}: Add {C}` on opp lands are all omitted (no name-prompt decision, no name-match selector). The card still slots into colorless utility roles. |
| Potioner's Trove | {3} | Artifact |  | {T}: Add one mana of any color. / {T}: You gain 2 life. Activate only if you've cast an instant or sorcery spell this turn. | ✅ (was 🟡) | Push XXXVIII (doc-sync): both activations wired. The mana ability adds any one color; the lifegain activation gates on the new `Predicate::InstantsOrSorceriesCastThisTurnAtLeast { who: You, at_least: 1 }` (backed by `Player.instants_or_sorceries_cast_this_turn`). Test: `potioners_trove_lifegain_requires_is_cast_this_turn`. |
| Rancorous Archaic | {5} | Creature — Avatar | 2/2 | Trample, reach / Converge — This creature enters with a +1/+1 counter on it for each color of mana spent to cast it. | ✅ | Push (modern_decks): "enters with N counters" now uses the new `CardDefinition.enters_with_counters` field (CR 614.12) keyed off `Value::ConvergedValue` so the counters land before SBA / ETB exactly per printed Oracle. Was an ETB AddCounter trigger that fired after SBA — gameplay was fine for the 2/2 body but the timing was wrong relative to other ETB triggers / replacement effects. |
| Skycoach Waypoint |  | Land |  | {T}: Add {C}. / {3}, {T}: Target creature becomes prepared. (Only creatures with prepare spells can become prepared.) | 🟡 | Push XXV: `{T}: Add {C}` mana ability wired via `tap_add_colorless()`. The {3},{T} prepare-target ability is omitted — engine has no Prepare keyword (same gap as Biblioplex Tomekeeper). |
| Strixhaven Skycoach | {3} | Artifact — Vehicle | 3/2 | Flying / When this Vehicle enters, you may search your library for a basic land card, reveal it, put it into your hand, then shuffle. / Crew 2 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.) | 🟡 | Push XXVI: Body wired — 3/2 Vehicle artifact with Flying. ETB basic-land tutor-to-hand via `Effect::Search { filter: IsBasicLand, to: Hand(You) }`. Crew is not enforced (no crew-as-tap-cost primitive yet); the Skycoach stays a non-creature artifact until that lands. |
| Sundering Archaic | {6} | Creature — Avatar | 3/3 | Converge — When this creature enters, exile target nonland permanent an opponent controls with mana value less than or equal to the number of colors of mana spent to cast this creature. / {2}: Put target card from a graveyard on the bottom of its owner's library. | 🟡 | Push XVI: `{2}: gy → bottom of owner's library` activated ability now wired via `Effect::Move { what: Target(0), to: ZoneDest::Library { who: OwnerOf(Target(0)), pos: Bottom } }`. ETB Converge exile is wired against `Nonland & ControlledByOpponent`; the mana-value cap against `ConvergedValue` is still approximated to "any nonland opp permanent" (no `Value`-keyed `ManaValueAtMost` predicate yet — tracked in TODO.md). |
| The Dawning Archaic | {10} | Legendary Creature — Avatar | 7/7 | This spell costs {1} less to cast for each instant and sorcery card in your graveyard. / Reach / Whenever The Dawning Archaic attacks, you may cast target instant or sorcery card from your graveyard without paying its mana cost. If that spell would be put into your graveyard, exile it instead. | 🟡 | Push XXV: Body wired (7/7 Legendary Avatar with Reach). The IS-in-gy cost-reduction static + attack-trigger cast-from-graveyard rider are omitted — engine has no per-graveyard-IS-count cost-reduction primitive nor cast-from-graveyard-without-paying for arbitrary cards. |
| Together as One | {6} | Sorcery |  | Converge — Target player draws X cards, Together as One deals X damage to any target, and you gain X life, where X is the number of colors of mana spent to cast this spell. | ✅ (was 🟡) | Push (modern_decks): two-slot multi-target shape. Slot 0 = target player draws X (`Value::ConvergedValue`), slot 1 = any target gets X damage. Self-life-gain runs unconditionally. Tests: `together_as_one_uses_converged_value_for_each_clause` (mono-colorless cast → ConvergedValue = 0 → all clauses zero), `together_as_one_three_color_cast_deals_three_to_each_clause` (R+G+U cast → ConvergedValue = 3 → opp draws 3 + takes 3 dmg, you gain 3). |
| Transcendent Archaic | {7} | Creature — Avatar | 6/6 | Vigilance / Converge — When this creature enters, you may draw X cards, where X is the number of colors of mana spent to cast this spell. If you draw one or more cards this way, discard two cards. | ✅ (was 🟡) | Push (modern_decks): "you may" optionality now honored via `Effect::MayDo` wrapping the ETB Converge draw + conditional discard 2. AutoDecider declines by default (skipping both); ScriptedDecider can flip to "yes" via `DecisionAnswer::Bool(true)`. The conditional discard 2 still rides on the same `If(ConvergedValue ≥ 1)` gate. Tests: `transcendent_archaic_etb_may_draw_declines_by_default`, `transcendent_archaic_etb_may_draw_accepts_via_scripted_decider`. |

## Strixhaven base set (STX)

Strixhaven 2021 cards (separate from the supplemental SOS catalog above).
Cards live under `catalog::sets::stx` and use the same engine primitives
as the SOS module. The two catalogs are independent — bringing them up to
parity is a matter of porting card factories one at a time.

### Silverquill (W/B)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Spirited Companion | {1}{W} | ✅ | 1/2 Dog Spirit. ETB: draw a card. |
| Eyetwitch | {B} | ✅ | 1/1 Pest. When dies: "learn" approximated as `Draw 1` (no Lesson sideboard yet). |
| Closing Statement | {X}{W}{W} | ✅ | Sorcery. Exile target nonland permanent. You gain X life (`Value::XFromCost`). |
| Vanishing Verse | {W}{B} | ✅ | Instant. Exile target nonland, **monocolored** permanent. Push XIX: filter promoted to the printed exact-shape `Permanent & Nonland & Monocolored` via the new `SelectionRequirement::Monocolored` predicate (`distinct_colors() == 1`). Multicolored and colorless permanents are correctly rejected by the cast-time target validator. |
| Killian, Ink Duelist | {W}{B} | ✅ (was 🟡) | Push XXXIV: 2/3 Legendary Human Warlock. Lifelink + the static "spells you cast that target a creature cost {2} less to cast" now wired via the new `StaticEffect::CostReductionTargetingFilter { spell_filter, target_filter, amount }` primitive. The reduction is applied during `cast_spell_with_convoke` after target validation; per CR 601.2f / 117.7c, generic-only pips are drained (colored pips untouched). Tests: `killian_ink_duelist_reduces_creature_targeting_spell`, `killian_reduction_does_not_eat_colored_pips`, `killian_does_not_reduce_non_creature_targeting_spell`, `killian_only_reduces_its_controllers_spells`. |
| Devastating Mastery | {4}{W}{W} | ✅ (was 🟡) | Push XXXIV (doc-only): Sorcery. Destroy each nonland permanent ("Wrath of God + lands"). The alt cost {7}{W}{W} (which adds a "return up to two nonland permanent cards from gy" mode) is an engine-wide alt-cost-implies-mode gap shared with Baleful Mastery and Verdant Mastery. Body fully ships the primary effect. |
| Felisa, Fang of Silverquill | {2}{W}{B} | ✅ | 4/3 Legendary Cat Cleric, Flying + Lifelink. Push XVI: counter-bearing-creature-dies → Inkling trigger now wired via `EventKind::CreatureDied/AnotherOfYours` filtered by `EntityMatches { what: TriggerSource, filter: WithCounter(+1/+1) }`. Counters persist on a card after move-to-graveyard (only `damage`/`tapped`/`attached_to` are cleared on zone-out), so the post-die graveyard-resident card still reports its `+1/+1` counters via `evaluate_requirement_static`. |
| Mavinda, Students' Advocate | {1}{W}{W} | 🟡 | 1/3 Legendary Human Cleric, Flying + Vigilance. Cast-from-graveyard activated ability is ⏳. |
| Eager First-Year | {W} | ✅ | 2/1 Human Student. Magecraft: target creature gets +1/+1 EOT. Uses the new `effect::shortcut::magecraft()` helper. |
| Hunt for Specimens | {3}{B} | ✅ (was 🟡) | Push XXXIV (doc-only): Sorcery. Both printed primary clauses ship — Pest token (with death-trigger lifegain via SOS-VI's `TokenDefinition.triggered_abilities`) + Learn (→ Draw 1 approximation). The Learn approximation is the same one used by Eyetwitch ✅, Pest Summoning ✅, Igneous Inspiration ✅, and Field Trip ✅; the Lessons sideboard model is engine-wide and tracked in TODO.md. |
| Star Pupil | {B} | ✅ | Push XXIII: 0/1 Cat Spirit. ETB +1/+1 counter; dies → put a +1/+1 counter on target creature. Audited against CR 122.8. |
| Silverquill Command | {2}{W}{B} | ✅ (was 🟡) | Push XXXII: Instant — promoted via `Effect::ChooseN { picks: [1, 3], modes }`. Auto-picks drain 2 + two +1/+1 counters on target creature. Counter-ability and gy-recursion modes still in `modes` for future mode-pick UI. |
| Defend the Campus | {3}{W}{W} | ✅ | Push XXVII: Sorcery. Creates three 1/1 W/B Inkling tokens with flying via `Effect::CreateToken { count: 3 }`. Reuses the SOS catalog's `inkling_token()` definition. |
| Hall Monitor | {W} | ✅ | Push XXVII: 1/1 Human Cleric. Magecraft: untap this creature. Wired via `magecraft(Effect::Untap)`. |
| Stonebinder's Familiar | {1} | ✅ | Push XXVII: 0/1 Artifact Creature — Spirit. "Whenever one or more cards leave your graveyard, put a +1/+1 counter on this creature." Uses the `EventKind::CardLeftGraveyard / YourControl` trigger (per-card emission, same as Spirit Mascot). |
| Necrotic Fumes | {2}{B}{B} | ✅ (was 🟡) | Push XXXII (doc-only): Sorcery. As an additional cost, sacrifice a creature. Exile target creature. Wired as `Seq(Sacrifice + Move→Exile)` at resolution time. The cost-at-resolution vs cost-at-cast difference is invisible to gameplay (one fodder → graveyard, target → exile, regardless of which step pays for which). Lock in via `necrotic_fumes_sacrifices_one_and_exiles_target`. |
| Make Your Mark | {1}{W} | ✅ | Push XXVII: Instant. +1/+1 EOT on target creature, draw a card. Trivial pump+cantrip wire. |
| Containment Breach | {1}{W} | ✅ | Push XXVII: Sorcery. Destroy target enchantment + Surveil 1. |
| Silverquill Pledgemage | {1}{W}{B} | ✅ | Push XXXI: 2/2 Inkling Druid with Flying. Magecraft: this creature gets +1/+1 EOT (uses the `magecraft_self_pump(1, 1)` shortcut). The Inkling subtype synergises with Tenured Inkcaster's new tribal anthem. Tests: `silverquill_pledgemage_is_a_two_two_inkling_flier`, `silverquill_pledgemage_magecraft_pumps_self_eot`, `silverquill_pledgemage_does_not_trigger_on_creature_cast`. |
| Archmage Emeritus | {2}{U}{U} | ✅ | Push XXXI: 3/3 Human Wizard. Magecraft: draw a card. Pure magecraft draw payoff — strong "spells matter" engine that doubles with copy-spell triggers (Aziza, Galvanic Iteration). Tests: `archmage_emeritus_draws_on_instant_cast`, `archmage_emeritus_does_not_draw_on_creature_cast`. |
| Promising Duskmage | {2}{W}{B} | ✅ | Push XXXI: 2/2 Inkling Wizard with Flying. Magecraft: each opponent loses 1 life and you gain 1 life (`magecraft_drain_each_opp(1)` — same Witherbloom drain template applied to a Silverquill flyer). The printed "target opponent" is collapsed to each-opponent for the auto-target framework. Test: `promising_duskmage_drains_on_instant_cast`. |
| Tenured Inkcaster | {2}{W}{B} | ✅ | Push XXXI: 3/2 Vampire Warlock. "Other Inkling creatures you control get +2/+2." Tribal anthem on the Inkling creature type, wired via the engine's `AffectedPermanents::AllWithCreatureType.exclude_source: true` flag (push XXX, Quintorius pattern). The anthem is layered in via a compute-time injection in `GameState::compute_battlefield`, so all of the controller's Inkling creatures (including Inkling tokens from Inkling Summoning, Defend the Campus) get +2/+2 while Inkcaster is on the battlefield. Tests: `tenured_inkcaster_buffs_friendly_inklings_by_two_two`, `tenured_inkcaster_does_not_buff_opponent_inklings`, `tenured_inkcaster_does_not_buff_self`, `tenured_inkcaster_anthem_expires_when_inkcaster_leaves_play`. |
| Selfless Glyphweaver | {1}{W}{W} | ✅ | Push (modern_decks, NEW, `stx::silverquill`): front-face only of the MDFC Selfless Glyphweaver // Deadly Vanity. 2/3 Human Cleric Wizard. "Sacrifice this creature: Creatures you control gain indestructible until end of turn." Wired as a `sac_cost: true` activation that grants `Keyword::Indestructible` EOT to each controlled creature; the Glyphweaver is sacrificed as cost (before resolution) so it doesn't grant indestructible to itself — matching the printed Oracle. Back-face Deadly Vanity (mass force-sacrifice with each-opp-picks-which-to-keep) is omitted (no multi-pick decision shape). Tests: `selfless_glyphweaver_sac_grants_indestructible_to_friendlies`, `selfless_glyphweaver_is_a_three_mana_two_three_cleric_wizard`. |
| Augusta, Dean of Order | {2}{W} | 🟡 | Push (modern_decks, NEW, `stx::extras`): 2/3 Legendary Human Cleric. Body-only wire. The printed combat-step trigger ("whenever you attack with three or more creatures with the same power, +1/+1 and chosen keyword EOT") is omitted (no "attacking creatures with same power" predicate or chosen-keyword pump shape). Partner with Plargg, Dean of Chaos is also omitted (no Partner-pair primitive). Test: `augusta_dean_of_order_is_a_two_three_legendary_human_cleric`. |

### Witherbloom (B/G)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Witherbloom Apprentice | {B}{G} | ✅ | 2/2 Human Warlock. Magecraft: drain 1 (each opponent loses 1; you gain 1). |
| Pest Summoning | {B}{G} | ✅ | Sorcery (Lesson). Creates two 1/1 Pest tokens; the death-trigger lifegain rider rides on the token via SOS-VI's `TokenDefinition.triggered_abilities`. |
| Witherbloom Pledgemage | {1}{B}{G} | ✅ | Push XXVIII: 3/3 Plant Warrior. `{T}, Pay 1 life: Add {B} or {G}.` Refactored in push XVIII to use the `life_cost: 1` field on the activated ability — the activation pays 1 life up front during cost-payment, leaving the effect as a pure `AddMana`. CR 605.1a's "no target, could add mana" criteria are met and the engine's `is_mana_ability` recogniser resolves this **without the stack** (matching the printed instant-speed mana ramp). The "B or G" choice is approximated as `ManaPayload::AnyOneColor`. Promotion to ✅ — the prior timing-nuance note didn't reflect any missing functionality. |
| Bayou Groff | {2}{B}{G} | ✅ | 5/4 Beast. Push XVI: "may pay {1} on death to return to hand" rider now wired via the new `Effect::MayPay` primitive (sibling to push XV's `Effect::MayDo`). On the death trigger, the controller is asked yes/no; on yes + sufficient mana, the engine pays {1} and `Move(SelfSource → Hand(OwnerOf(Self)))`. |
| Honor Troll | {1}{B}{G} | ✅ | Push XXVIII: 1/4 Troll Warrior with Trample. The conditional `+2/+0` and Lifelink rider is **now wired** via a compute-time injection in `GameState::compute_battlefield` (same pattern as Cruel Somnophage / Tarmogoyf). The gate reads `Player.life_gained_this_turn` (already tracked for the `LifeGainedThisTurnAtLeast` predicate); when > 0, layers 6 (keyword) and 7b (P/T modify) inject `AddKeyword(Lifelink)` and `ModifyPowerToughness(+2, +0)` targeting the Troll as `AffectedPermanents::Source`. The gate re-evaluates every recompute, so a mid-turn lifegain flips the troll on for the rest of the turn, and `do_untap`'s reset to `life_gained_this_turn = 0` flips it back off next turn. Tests: `honor_troll_base_state_no_lifegain_is_one_four`, `honor_troll_with_lifegain_is_three_four_lifelink`. |
| Daemogoth Titan | {B}{B} | ✅ | Push XX + push XXVI: 11/11 Demon Horror. Attack-trigger sacrifice + block-trigger sacrifice now both wired. Block-half uses the new `EventKind::Blocks` event added in push XXVI (per CR 509.1i — blocker-side triggers). The sacrifice's auto-decider picks fodder before the Titan itself when both exist. |
| Daemogoth Woe-Eater | {2}{B}{G} | ✅ | Push XXVIII: 4/4 Demon Horror. ETB sacrifice (mandatory) + attack-trigger sac-into-+1/+1-counter `Seq` now wrapped in `Effect::MayDo` so the printed "you may sacrifice" optionality is honored. AutoDecider defaults to "no" (skip the sac); `ScriptedDecider::new([Bool(true)])` exercises the paid path. Tests: `daemogoth_woe_eater_etb_sacrifices_another_creature`, `daemogoth_woe_eater_attack_optional_sac_can_be_declined`, `daemogoth_woe_eater_attack_optional_sac_can_be_accepted`. |
| Mortality Spear | {3}{B}{G} | ✅ | Push XX: Instant. Destroy target creature or planeswalker (Battle subtype omitted — not modelled in this catalog). |
| Tempted by the Oriq | {2}{B} | ✅ (was 🟡) | Push XXXIII (doc-only): Sorcery. The printed Threaten template (GainControl + Untap + Haste, all EOT) was fully wired in push XX. The prior 🟡 note referenced a hypothetical "Magecraft rider on the controlled creature" that does not appear on the printed card. **Closes the STX Witherbloom (B/G) school — 0 🟡 STX Witherbloom cards remain.** |
| Witherbloom Command | {2}{B}{G} | ✅ (was 🟡) | Push XXXII: Sorcery — promoted via `Effect::ChooseN { picks: [0, 2], modes }`. Auto-picks mill 4 vs each opp + drain 2. Destroy noncreature/nonland MV ≤ 2 and grant indestructible EOT (regen approximation) still in `modes` for future mode-pick UI. |

### Lorehold (R/W)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Lorehold Apprentice | {R}{W} | ✅ | Push XXXI doc sync: stale 🟡 note cleared. Magecraft already fires both halves — `Seq(GainLife(1) + DealDamage(1))` against an auto-targeted Creature/Player/Planeswalker via `target_filtered`. The auto-target picker aims a friendly source's ping at the best legal target. Tests: `lorehold_apprentice_gains_life_on_instant_cast`, `lorehold_apprentice_magecraft_drains_one_to_opponent_and_gains_life`. |
| Lorehold Pledgemage | {1}{R}{W} | ✅ | Push XXXI doc sync: stale 🟡 note cleared. The `{2}{R}{W}, Exile a card from your graveyard: +1/+1 EOT` activation is wired via `ActivatedAbility.exile_other_filter: Some(Any)` (push XVIII engine primitive). Tests: `lorehold_pledgemage_gy_exile_cost_pumps_self`, `lorehold_pledgemage_rejects_activation_with_empty_graveyard`. |
| Pillardrop Rescuer | {3}{R}{W} | ✅ | 3/3 Spirit Cleric with Flying. ETB: return target instant or sorcery card from your graveyard to your hand. |
| Heated Debate | {2}{R} | ✅ | Instant. 4 damage to target creature. Damage-can't-be-prevented rider is a no-op (engine has no prevention layer). |
| Storm-Kiln Artist | {2}{R}{W} | ✅ | Push XXXI doc sync: stale 🟡 note cleared. The "1 damage to any target" half is wired faithfully — `DealDamage` against `target_filtered(Creature ∨ Player ∨ Planeswalker)`, NOT collapsed to "each opponent". Treasure half fires after the damage half. Test: `storm_kiln_artist_creates_treasure_and_deals_1_damage`. |
| Sparring Regimen | {2}{R}{W} | ✅ | Push XXXI doc sync: stale 🟡 note cleared. Both halves wired — ETB creates a 2/2 R/W Spirit token via `lorehold_spirit_token()` + per-attacker `Attacks/AnotherOfYours` trigger places a +1/+1 counter on `Selector::TriggerSource`. The per-attacker emission model matches the printed batch trigger exactly (every declared attacker gets one counter). Tests: `sparring_regimen_creates_a_2_2_spirit_token_on_etb`, `sparring_regimen_creates_spirit_etb_and_pumps_attacker`. |
| Lorehold Command | {2}{R}{W} | ✅ (was 🟡) | Push XXXII: Sorcery — promoted via the new `Effect::ChooseN { picks: [0, 3], modes }`. Auto-picks 4 damage to opp + two 2/2 R/W flying Spirits. The -2/-0 debuff and gy recursion modes are available for future mode-pick UI. **Closes out the Lorehold school — 0 🟡 STX Lorehold cards remain.** |
| Plargg, Dean of Chaos | {1}{R} | ✅ (was 🟡) | Push (modern_decks): 2/2 Legendary Human Cleric. `{T}: Discard a card, then draw a card.` wired faithfully as a tap activation. The conditional **"if a creature card was discarded → 2 damage"** rider is **now wired** via the new `Value::CreatureCardsDiscardedThisEffect` primitive — both `Effect::Discard` branches (random + player-chosen) bump `creature_cards_discarded_this_resolution` when the discarded card carries `CardType::Creature`, and Plargg's tail `Effect::If { cond: ValueAtLeast(_, 1), DealDamage 2 }` reads that counter. The "any target" damage uses `target_filtered(Creature ∨ Player ∨ Planeswalker)` so activation requires a target slot. The "Partner with Augusta, Dean of Order" rider is still omitted (no Partner-pair primitive). Tests: `plargg_dean_of_chaos_taps_to_loot`, `plargg_dean_of_chaos_deals_two_damage_when_creature_discarded`, `plargg_dean_of_chaos_no_damage_when_noncreature_discarded`. |
| Reconstruct History | {1}{R}{W} | ✅ | Push (modern_decks, NEW, `stx::lorehold`): Sorcery. "Choose two or more — return target artifact / instant / Spirit / sorcery card from your graveyard to your hand." Wired via `Effect::ChooseN { picks: [0, 1], modes }` with each mode resolving its filter against `Selector::one_of(CardsInZone(Graveyard, filter))`. The auto-decider picks modes 0 (artifact) + 1 (instant) by default — the typical Lorehold gy mix has both. The Spirit + sorcery modes (2, 3) sit in `modes` for future mode-pick UI. The "choose two or more" semantics is collapsed to two modes since the engine's `ChooseN.picks` field is a flat list rather than a count range; the printed Oracle's "or more" lets a player pick 3-4 modes when their gy is deep. Tests: `reconstruct_history_returns_two_cards_from_graveyard_to_hand`, `reconstruct_history_is_a_three_mana_lorehold_sorcery`. |
| Lorehold Excavation | — | ✅ | Push (modern_decks, NEW, `stx::lorehold`): Lorehold dual land. "{T}: Add {R} or {W}. / {2}{R}{W}, {T}: Exile target card from a graveyard. If a creature card was exiled this way, create an X/X red and white Spirit creature token with flying, where X is that card's power." Wired with two `tap_add` mana abilities + a third activated ability that exiles a target gy card and conditionally mints an X/X R/W flying Spirit token when the target is a creature. The "X = its power" scaling is **now wired faithfully** via an engine improvement that extends `Value::PowerOf` to read printed power across battlefield / graveyard / exile / hand zones — at gy-resolution time the target is still in graveyard, so `Value::PowerOf(Target(0))` reads the creature's printed power. A 2/2 Grizzly Bears in gy → 2/2 Spirit; a 4/4 Serra Angel → 4/4 Spirit; a 0/0 creature → 0/0 token dies to SBA (printed Oracle exact). Tests: `lorehold_excavation_is_a_lorehold_dual_with_two_mana_abilities`, `lorehold_excavation_exile_creature_mints_flying_spirit_token`, `lorehold_excavation_exile_non_creature_no_token`, `lorehold_excavation_token_scales_with_creature_power`. |

### Quandrix (G/U)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Quandrix Apprentice | {G}{U} | ✅ | 1/1 Elf Druid. Magecraft: target creature you control gets +1/+1 EOT. |
| Quandrix Pledgemage | {1}{G}{U} | ✅ | 2/2 Fractal Wizard. Activated `{1}{G}{U}: +1/+1 counter on this creature`. |
| Decisive Denial | {G}{U} | ✅ (was 🟡) | Push XXXIII: both modes wired. Mode 0 ships the classic counter-noncreature-unless-{2}. Mode 1 (fight) promoted via the Chelonian Tackle template — `Effect::Fight { attacker: Target(0), defender: auto-pick EachPermanent(Creature & ControlledByOpponent) }`. Multi-target defender prompt is a future enhancement. |
| Quandrix Cultivator | {3}{G}{U} | ✅ | Push XX: 3/3 Elf Druid. ETB search basic Forest or Island → battlefield tapped. |
| Manifestation Sage | {2}{G}{U} | ✅ | Push XXIII: 2/2 Fractal Wizard, Flying. ETB mints 0/0 Fractal + X +1/+1 counters where X = `HandSizeOf(You)`. |
| Quandrix Command | {1}{G}{U} | ✅ (was 🟡) | Push XXXII: Instant — promoted via `Effect::ChooseN { picks: [0, 2], modes }`. Auto-picks two +1/+1 counters on target creature + mill 2 vs opp. Counter-ability and bounce modes still in `modes` for future mode-pick UI. (Mode 2's X collapses to flat "2" — engine has no `Value::Times(N, CountOf(...))` shortcut.) |
| Mentor's Guidance | {1}{G}{U} | ✅ | Push XXIII: Instant. Two-mode `ChooseMode` — damage = creatures you control, or draw = creatures with +1/+1 counters. |
| Symmathematics | {1}{G}{U} | ✅ | Push (modern_decks): Fractal creature, printed 0/0 — base body now lands at exact printed P/T via the new `CardDefinition.enters_with_counters` field (CR 614.12). The two +1/+1 counters arrive before SBA, so 0/0 + 2 +1/+1 = 2/2 ETB exactly (was engine-bumped 1/1 base + 2 = 3/3). Magecraft doubles +1/+1 counters via `AddCounter { amount: CountersOn(This, +1/+1) }`: 2 → 4 → 8 → 16. Tests: `symmathematics_enters_with_two_plus_one_counters`, `symmathematics_doubles_counters_on_instant_cast`, `symmathematics_does_not_double_on_creature_cast`. |

### Prismari (U/R)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Prismari Pledgemage | {1}{U}{R} | ✅ | 2/3 Elemental with Trample + Haste. |
| Prismari Apprentice | {U}{R} | ✅ (was 🟡) | Push XXXIII: 2/2 Human Wizard. Modal Magecraft (Scry 1 / +1/+0 EOT) now wired via the new CR 700.2b modal trigger mode pick (`GameState::pick_trigger_mode` in `game/stack.rs`). AutoDecider picks mode 0 (Scry 1) for default play; `ScriptedDecider::new([DecisionAnswer::Mode(1)])` unlocks the +1/+0 branch. Tests: `prismari_apprentice_modal_magecraft_scrys_by_default`, `prismari_apprentice_modal_magecraft_pumps_via_scripted_mode_pick`. |
| Symmetry Sage | {U} | ✅ | 1/2 Human Wizard. Magecraft: this creature gets +1/+0 and gains flying until end of turn. |
| Galvanic Iteration | {U}{R} | ✅ (was 🟡) | Push XXXIV (doc-only): Instant. Copy target instant or sorcery spell via `Effect::CopySpell`. Magecraft self-exile rider omitted — the gameplay difference is strictly gy vs exile after cast (the copy itself resolves identically). |
| Expressive Iteration | {U}{R} | 🟡 | Push XXIV: Sorcery. Collapsed to `Scry 2 → Draw 1` (the exile-and-play-from-exile primitive is ⏳). |
| Magma Opus | {7}{U}{R} | ✅ (was 🟡) | Push XXXIV (doc-only): Sorcery. 4 dmg + tap opp creatures + 4/4 Elemental token + draw 2 all ship. Multi-target divided damage collapses to a single creature (engine-wide gap shared with Crackle with Power ✅). Discard alt-mode for Treasure is omitted (no discard-as-activation-cost primitive yet) — Magma Opus is overwhelmingly cast for its body. |
| Sparkmage Apprentice | {1}{R} | ✅ | Push XXIV: 1/2 Human Wizard. ETB: deals 2 damage to any target. |
| Soothsayer Adept | {1}{U} | ✅ | Push XXIV: 2/2 Merfolk Wizard. Activated `{2}{U}: Surveil 1`. |
| Prismari Command | {1}{U}{R} | ✅ (was 🟡) | Push XXXII: Instant — promoted via `Effect::ChooseN { picks: [1, 2], modes }`. Auto-picks loot 1 + create a Treasure. Damage and destroy-artifact modes still in `modes` for future mode-pick UI. Mode 1's "extra draw if discarded noncreature/nonland" rider collapses to flat draw. |

### Mono-color staples (`stx::mono`)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Pop Quiz | {1}{W} | ✅ | Sorcery (Lesson). Draw 2, then put a card from your hand on top of your library. |
| Mascot Exhibition | {5}{W}{W} | ✅ | Sorcery. Creates a 3/3 Elephant, 2/2 lifelink Cat, and 1/1 flying Bird. |
| Plumb the Forbidden | {X}{B}{B} | ✅ | Instant. Sacrifice X creatures, draw X cards, lose X life. |
| Owlin Shieldmage | {3}{W} | ✅ (was 🟡) | Push (modern_decks): 2/3 Bird Wizard with Flash + Flying. The printed "prevent all combat damage this turn" ETB **now lands** via the new `Effect::PreventAllCombatDamageThisTurn` primitive (CR 615.1) + `GameState.prevent_combat_damage_this_turn` flag. Combat damage resolver zeroes attacker→blocker, attacker→player, and blocker→attacker damage; lifelink scales off actual damage dealt so lifelink life-gain zeroes too. The flag clears in `do_cleanup` (CR 514.2). Tests: `owlin_shieldmage_etb_prevents_combat_damage_this_turn` (full e2e: opp swings with bear → flash Shieldmage → 0 damage), `prevent_combat_damage_flag_clears_in_cleanup`. |
| Frost Trickster | {1}{U} | ✅ | 2/2 Spirit Wizard, Flash + Flying. ETB taps + stuns target opponent's creature. |
| Body of Research | {4}{G}{U} | ✅ | Push XVI: now uses the new `Value::LibrarySizeOf(You)` primitive — Fractal token enters with one +1/+1 counter per library card, matching the printed Oracle exactly (was approximating via `GraveyardSizeOf` since `LibrarySize` wasn't a primitive). |
| Show of Confidence | {1}{W} | ✅ | Instant. Adds `StormCount + 1` +1/+1 counters on target creature you control. |
| Bury in Books | {3}{U} | ✅ | Sorcery. Put target creature on top of its owner's library. |
| Test of Talents | {1}{U}{U} | ✅ (was 🟡) | Push XXXIV (doc-only): Counter target instant or sorcery. The search-and-exile-by-name rider only matters when the opp has 2+ copies of the countered spell across their zones — a rare combo-deck-specific corner; the headline counter half plays correctly always. |
| Multiple Choice | {1}{U}{U} | ✅ (was 🟡) | Push XXXIV: Sorcery — promoted via `Effect::ChooseN { picks: [0, 1, 2, 3], modes }`. Auto-picks all four printed modes: Scry 2 + 1/1 blue Pest + +1/+0 hexproof EOT on target creature + Draw 2 (the "if you chose all of the above" bonus mode). Same all-modes shortcut as the Commands cycle. Tests: `multiple_choice_fires_all_four_modes`. |
| Curate | {1}{U} | ✅ (was 🟡) | Push XXXIII (doc-only): Instant. "Look at top 4, put 1 in hand, rest on bottom in random order" approximated as `Scry 3 → Draw 1`. The "random order on bottom" rider is engine-wide (no RNG in `resolve_effect`) and tracked in TODO.md. Test: `curate_nets_zero_hand_size_via_scry_three_draw_one`. |
| Solve the Equation | {2}{U} | ✅ | Push XX: Sorcery. Tutor an instant or sorcery from library to hand (printed mana-value cap omitted for simplicity). |
| Resculpt | {1}{U} | ✅ | Push XX: Instant. Exile target creature or artifact; its original controller creates a 4/4 blue Elemental token. |
| Ageless Guardian | {2}{W} | ✅ | Push XXIII: 1/4 Spirit Cleric. Magecraft: this creature gets +1/+0 EOT (`magecraft_self_pump`). |
| Returned Pastcaller | {4}{W} | ✅ | Push XXIII: 3/3 Spirit Cleric, Flying. ETB return target IS card from your graveyard to hand. |
| Letter of Acceptance | {1} | ✅ | Push XXIII: Artifact. ETB +1 life; `{T}: Add {C}`; `{2},{T},Sac: Draw a card`. |
| Charge Through | {G} | ✅ | Push XXIII: Sorcery. Target creature you control gets +1/+1 and gains trample EOT. |
| Devious Cover-Up | {2}{U}{U} | ✅ (was 🟡) | Push XXXIV (doc-only): Instant. Counter target spell + exile up to one gy card. The "any number of gy cards" multi-target rider is an engine-wide gap shared with Vibrant Outburst ✅, Snow Day ✅, Spell Satchel, Crackle with Power ✅; the single-strip captures the headline play pattern. |
| Crackle with Power | {X}{R}{R}{R}{R}{R} | ✅ (was 🟡) | Push XXXIV (doc-only): Sorcery. 5X damage to a single Creature/Player/Planeswalker via `Value::Times(Const(5), XFromCost)`. The "divided as you choose among any number of targets" multi-target rider collapses to one target — the engine-wide gap shared with Magma Opus ✅ and Devious Cover-Up ✅. The five-quintuple-pip {RRRRR} cost is honored exactly. |
| Dragonsguard Elite | {1}{G}{G} | ✅ | Push XXIII: 2/2 Human Warrior. Magecraft +1/+1 counter; `{3}{G}: +X/+X EOT` where X = `PowerOf(This)`. |
| Quintorius, Field Historian | {2}{R}{W} | ✅ (was 🟡) | Push XXXIV (table sync — code already wired in push XXXI): 3/3 Legendary Elephant Cleric Spirit, Vigilance. ETB exile gy card + create 3/2 R/W Spirit. Tribal Spirit anthem (+1/+0 Other Spirits) wired via the `tribal_anthem_for_name` compute-time injection in `GameState::compute_battlefield` — same pattern as Tenured Inkcaster. Tests: `quintorius_anthem_pumps_other_spirits_not_self`, `quintorius_anthem_expires_when_he_leaves_battlefield`. |
| Crashing Drawbridge | {3} | ✅ | Push XXIV: 0/4 Artifact Creature — Construct. "Creatures you control have haste" via `StaticEffect::GrantKeyword`. |
| Eyetwitch Brood | {1}{B}{G} | ✅ | Push XXIV: 1/1 Pest with Lifelink. "Whenever another Pest you control dies, put a +1/+1 counter on this creature." Disambiguated from SOS's "Pest Mascot" (LifeGained trigger). |
| First Day of Class | {W} | ✅ (was 🟡) | Push XXXIV (doc-only): Sorcery. Anthem clause (+1/+1 EOT for each creature you control) is the headline play pattern. The "Pest combat-damage delayed trigger" rider is omitted — no `DelayedTriggerSpec` primitive from sorcery resolution yet; bonus value rarely flips combat math when the anthem is already swinging. |
| Verdant Mastery | {3}{G}{G} | ✅ (was 🟡) | Push XXXIV (doc-only): Sorcery. Both printed regular-cost clauses ship — caster fetches a basic untapped, then each opponent's auto-decider opts into fetching a basic tapped (when a basic is available; no-op otherwise). The {6}{G}{G} alt-cost two-basics-each mode is an engine-wide alt-cost-implies-mode gap shared with Baleful Mastery ✅ and Devastating Mastery ✅. |
| Sacred Fire | {R}{W} | ✅ | Push XXIV: Sorcery. 3 damage to any target + 3 life + Flashback {5}{R}{W}. |
| Rip Apart | {R}{W} | ✅ | Push XXIV: Sorcery. Two-mode `ChooseMode`: 3 dmg to creature/PW or destroy artifact/enchantment. |
| Codespell Cleric | {W} | ✅ | Push XXIV: 1/1 Kor Cleric with Lifelink. Vanilla low-curve Silverquill body. |
| Reckless Amplimancer | {2}{G} | 🟡 | Push XXIV: 2/2 Elf Druid. Activated `{4}{G}{G}: +3/+3 EOT` (printed `+X/+X = mana spent` collapses to fixed +3/+3). |
| Karok Wrangler | {1}{G}{U} | ✅ | Push XXIV: 2/2 Elf Druid. Magecraft: +1/+1 counter on target creature you control. |
| Quick Study | {1}{U} | ✅ | Push XXV: Instant. Target player draws two cards. Simple `Effect::Draw { who: Player(You), amount: 2 }`. |
| Bookwurm | {5}{G}{G} | ✅ | Push XXIX (doc sync): 5/5 Wurm with Trample. ETB: gain 4 life + draw 1. |
| Field Trip | {2}{G} | ✅ | Push XXIX (doc sync): Sorcery. Search basic Forest → battlefield + Learn (→ Draw 1). |
| Reduce to Memory | {2}{U} | ✅ | Push XXIX (doc sync): Sorcery. Exile target nonland permanent + its controller mints a 4/4 blue Elemental. |
| Baleful Mastery | {2}{B} | ✅ (was 🟡) | Push XXXII (doc-only): Instant. Exile target creature/planeswalker; an opp draws a card. The alt-cost {1}{B} (vs. the regular {2}{B}) is a printed-cost saver only — the "opp draws a card" rider is part of the spell's main effect and always fires regardless of cast path. Body fully wired. Lock in via `baleful_mastery_exiles_target_and_opp_draws`. |
| Igneous Inspiration | {2}{R} | ✅ | Push XXIX (doc sync): Sorcery. 3 damage to creature/PW + Learn (→ Draw 1). |
| Combat Professor | {3}{W} | ✅ (was 🟡) | Push XXXII (doc-only): 2/4 Cat Cleric, Flying + Vigilance. Mentor wired as `Attacks/SelfSource → AddCounter(target attacking creature with PowerAtMost(1))`. Since Combat Professor's base power is 2, `PowerAtMost(1)` is the exact CR-equivalent of "lesser power than this creature" — power < 2 means power ≤ 1. Lock in via `combat_professor_mentor_buffs_a_smaller_attacker`. |
| Conspiracy Theorist | {1}{R} | 🟡 | Push XXIX (doc sync): 2/1 Human Shaman. Attack-trigger rummage-into-exile-and-play and empty-hand activated are both omitted (no cast-from-exile-with-timer primitive). |
| Beaming Defiance | {1}{W} | ✅ | Push XXIX (doc sync): Instant. Target creature you control gets +2/+0 + hexproof EOT. |
| Spell Satchel | {3} | 🟡 | Push XXIX (doc sync): Artifact. `{T}: Add {C}` + `{3},{T},Sac:` returns single target IS card from gy to hand. Multi-target "any number with total MV ≤ 4" picker still pending. |
| Excavated Wall | {2} | ✅ | Push XXIX (doc sync): 0/4 Artifact Creature — Wall with Defender. ETB: gain 2 life. |
| Snow Day | {U}{R} | ✅ (was 🟡) | Push XXXIII (doc-only): Instant. Tap + stun one creature. The "up to two targets" multi-target prompt is engine-wide (same gap as Vibrant Outburst, Spell Satchel, Devious Cover-Up); promoted via the Vibrant Outburst precedent. Test: `snow_day_doc_promoted_taps_and_stuns_target_creature`. |
| Confront the Past | {3}{R} | ✅ (was 🟡) | Push XXXIII: Sorcery, 3-mode `ChooseMode`. Mode 2 promoted from "flat 3 damage" to true `Value::LoyaltyOf(Target(0))` damage — reads the target PW's current loyalty counter pool. Pairs with the CR 120.3c spell-damage fix in `deal_damage_to`. Test: `confront_the_past_mode_2_uses_loyalty_counter_x` (Professor Dellian Fel at 5 loyalty → takes 5 damage → dies via PW-0-loyalty SBA). |
| Specter of the Fens | {4}{B} | ✅ | Push XXIX (doc sync): 3/4 Flying Specter. ETB: return creature/PW from your gy → hand. |
| Mascot Interception | {4}{R}{W} | ✅ | Push XXIX (doc sync): Instant. Gain control of target permanent EOT + Untap + Haste EOT. |
| Twinscroll Shaman | {2}{U}{R} | ✅ | Push XXIX (doc sync): 3/3 Human Wizard. Magecraft: Copy that spell via `Effect::CopySpell{what: TriggerSource}`. |
| Practical Research | {1}{G}{U} | ✅ | Push XXIX (doc sync): Sorcery. Doubles +1/+1 counters on target creature you control via `AddCounter(amount = CountersOn(target, +1/+1))`. |
| Hall of Oracles | — | ✅ | Push XXIX (doc sync): Land. `{T}: Add {C}` + `{2},{T}: +1/+1 counter on target Wizard or Fractal creature you control`. |
| Environmental Sciences | {1}{G} | ✅ | Push XXXII (NEW, `stx::lessons`): Sorcery — Lesson. Gain 4 life + tutor a basic land to hand. Tests: `environmental_sciences_gains_four_life_and_tutors_a_basic_land`. |
| Introduction to Annihilation | {3}{W} | ✅ | Push XXXII (NEW, `stx::lessons`): Sorcery — Lesson. Destroy target nonland permanent; its controller scries 2 (via `PlayerRef::ControllerOf(Target(0))` so the consolation Scry resolves against the right player). Test: `introduction_to_annihilation_destroys_nonland_permanent`. |
| Introduction to Prophecy | {2}{U} | ✅ | Push XXXII (NEW, `stx::lessons`): Sorcery — Lesson. Scry 3 + draw a card. Test: `introduction_to_prophecy_scries_three_and_draws_one`. |
| Spirit Summoning | {3}{W} | ✅ | Push XXXII (NEW, `stx::lessons`): Sorcery — Lesson. Mint a 3/2 W Spirit with lifelink. Test: `spirit_summoning_creates_a_three_two_lifelink_spirit`. |
| Square Up | {U}{R} | ✅ | Push XXXII (NEW, `stx::lessons`): Prismari instant. Target creature's base P/T becomes 0/4 EOT; draw a card. First card using the new `Effect::SetBasePT` layer-7b primitive. Counters and +N/+M stack on top per CR 613.7c-f. Tests: `square_up_sets_target_creature_to_zero_four_and_draws`, `square_up_layers_under_plus_one_counters`. |
| Lash of Malice | {B} | ✅ | Push XXXV (NEW, `stx::mono`): Instant. Target creature gets -2/-2 EOT via negative `PumpPT` (a 2/2 dies to SBA). Flashback {3}{B} wired via `Keyword::Flashback`. Tests: `lash_of_malice_kills_two_two_creature`, `lash_of_malice_has_flashback_keyword`. |
| Big Play | {3}{R}{W} | ✅ | Push XXXV (NEW, `stx::mono`): Instant. Three-mode `ChooseMode`: (0) Tap+Stun on opp creature (collapsed "must attack"), (1) Tap+Stun (the canonical Frost Trickster shape), (2) Each creature you control gains Trample EOT. Auto-decider picks mode 1; scripted decider can probe modes 0/2. The draw-on-combat-damage rider in printed mode 2 is engine-wide ⏳. Tests: `big_play_auto_picks_tap_and_stun`, `big_play_mode_2_grants_trample_to_friendlies`. |
| Burrog Befuddler | {1}{U} | ✅ | Push XXXVI (NEW, `stx::extras`): 2/1 Frog Wizard with Flash. ETB: target creature gets -3/-0 until end of turn. Standard combat trick body. Tests: `burrog_befuddler_etb_minus_three_zero`, `burrog_befuddler_has_flash`. |
| Mage Hunters' Mark | {1}{R} | ✅ | Push XXXVI (NEW, `stx::extras`): Instant. Target creature gets +3/+0 + Menace EOT. Pump-and-menace combat trick wired as `Seq(PumpPT(+3/+0), GrantKeyword(Menace))`. Test: `mage_hunters_mark_pumps_target_and_grants_menace`. |
| Mage Duel | {1}{R} | ✅ | Push XXXVI (NEW, `stx::extras`): Sorcery. Friendly creature deals damage = its power to a target opp creature. Resolved via `Value::PowerOf(EachPermanent(Creature & ControlledByYou))` (auto-picks the friendly attacker) feeding `Effect::DealDamage` against an opp Creature target. Test: `mage_duel_friendly_burns_opp_creature_by_friendly_power`. |
| Eccentric Apprentice | {1}{R} | ✅ | Push XXXVI (NEW, `stx::extras`): 1/3 Human Wizard. Magecraft self-pump (+1/+0 EOT) via `magecraft_self_pump(1, 0)`. Each instant or sorcery cast bumps the body to a more relevant attacker. Test: `eccentric_apprentice_pumps_on_instant_cast`. |
| Illuminate History | {1}{R}{W} | ✅ | Push XXXVI (NEW, `stx::extras`): Sorcery. As an additional cost, discard a card. Mints two 2/2 R/W Spirit tokens with flying. Cost-vs-resolution timing approximated (discard runs as `Effect::Discard(You, 1)` at resolution alongside the token mint). Test: `illuminate_history_discards_and_creates_two_spirits`. |
| Brilliant Plan | {3}{U}{U} | ✅ | Push XXXVI (NEW, `stx::lessons`): Sorcery — Lesson. Scry 3 + Draw 3. Pure card-velocity Lesson, wired as `Seq(Scry(3), Draw(3))`. Test: `brilliant_plan_scrys_three_and_draws_three`. |
| Fortifying Draught | {2}{W} | ✅ | Push XXXVI (NEW, `stx::lessons`): Sorcery — Lesson. Target creature gets +1/+4 EOT. Defensive combat trick Lesson, wired as a single `PumpPT(+1/+4, EndOfTurn)`. Test: `fortifying_draught_pumps_target_creature`. |
| Guiding Voice | {W} | ✅ | Push XXXVI (NEW, `stx::lessons`): Sorcery — Lesson. +1/+1 counter on target creature + Learn (→ Draw 1). Wired as `Seq(AddCounter(+1/+1), Draw(1))`. Test: `guiding_voice_counters_and_draws`. |
| Tezzeret's Gambit | {U}{B} | ✅ | Push XXXVI (NEW, `stx::extras`): Sorcery. Two-mode `ChooseMode`: (0) Proliferate; (1) Pay 2 life, draw 2 cards. Printed cost {U/P}{B/P} (Phyrexian mana) collapses to strict {U}{B} — pure Phyrexian-life payment is engine-wide ⏳. Tests: `tezzerets_gambit_mode_zero_proliferates`, `tezzerets_gambit_mode_one_pays_two_life_draws_two`. |
| Wandering Archaic | {2}{W}{W} | ✅ (was 🟡) | Push (modern_decks): the printed "may pay {2}" tax-or-copy gate **is now wired** via the new `Effect::CopySpellUnlessPaid { what, mana_cost, count }` primitive. At trigger resolution: (a) locate the matching `StackItem::Spell` for `what`; (b) ask the spell's *caster* yes/no via `Decision::OptionalTrigger`; (c) on yes + affordable pool, deduct `mana_cost` and skip the copy; (d) on no or unaffordable, copy the spell `count` times. AutoDecider defaults to false (decline to pay → copy fires). ScriptedDecider can flip to true for tests. The "you may choose new targets for the copy" half is engine-wide ⏳ (copies inherit the original's targets — same gap as every other CopySpell user). Tests: `wandering_archaic_copies_opp_instant` (AutoDecider declines → copy fires), `wandering_archaic_lets_opp_pay_two_to_skip_copy` (ScriptedDecider says yes + pre-floats {2} → copy skipped), `wandering_archaic_copies_when_opp_cannot_afford_two` (ScriptedDecider says yes but opp has no {2} → copy fires anyway). |
| Take Up the Shield | {1}{W} | ✅ | Push XXXVII (NEW, `stx::extras`): Instant. Target creature gets +0/+3 and gains indestructible EOT. Wired as `Seq(PumpPT(+0/+3), GrantKeyword(Indestructible))` — same Masterful-Flourish-style template. Defensive combat trick that protects a friendly attacker or a fragile blocker through a Wrath. Test: `take_up_the_shield_buffs_toughness_and_grants_indestructible`. |
| Star Pupil's Papers | {1} | ✅ | Push XXXVII (NEW, `stx::extras`): Artifact. ETB Scry 1; `{2}, Sacrifice this: Put a +1/+1 counter on target creature.` Pure colorless filtering + counter payoff. ETB trigger uses `Effect::Scry { who: You, amount: 1 }`; the activated ability uses `sac_cost: true` to consume the artifact. Tests: `star_pupils_papers_is_a_one_mana_artifact_with_etb_scry`, `star_pupils_papers_sac_activation_grants_counter`. |
| Frostboil Snarl | — | ✅ (🟡 reveal half) | Push XXXVII (NEW, `stx::extras`): Izzet (U/R) Snarl dual. Always-enters-tapped approximation of the printed "reveal-from-hand-or-tap" mechanic. Wired via the new `snarl_land()` helper which produces `{T}: Add {U}` and `{T}: Add {R}` activated abilities plus the standard `etb_tap()` trigger. The full reveal-from-hand decision shape is tracked in TODO.md. Test: `frostboil_snarl_is_a_u_r_dual_that_enters_tapped`. |
| Furycalm Snarl | — | ✅ (🟡 reveal half) | Push XXXVII (NEW, `stx::extras`): Boros (R/W) Snarl dual. Same shape as Frostboil Snarl. |
| Necroblossom Snarl | — | ✅ (🟡 reveal half) | Push XXXVII (NEW, `stx::extras`): Golgari (B/G) Snarl dual. |
| Shineshadow Snarl | — | ✅ (🟡 reveal half) | Push XXXVII (NEW, `stx::extras`): Orzhov (W/B) Snarl dual. |
| Vineglimmer Snarl | — | ✅ (🟡 reveal half) | Push XXXVII (NEW, `stx::extras`): Simic (G/U) Snarl dual. All five Snarls share the `snarl_land()` factory; one parameterised test (`all_five_snarl_lands_are_dual_subtypes`) walks the cycle. |
| Dragon's Approach | {B} | ✅ (was 🟡) | Push XXXVIII: both halves wired. The 3 damage to any target stays as before; the "if 4+ same-named copies in your graveyard, search a Dragon" rider is wired via the new `Predicate::SameNamedInZoneAtLeast { who: You, zone: Graveyard, at_least: 4 }` primitive — the engine reads the resolving spell's printed name from `EffectContext.source_name` (stamped by the new `for_spell_with_source` constructor) and counts matches in the controller's graveyard. On hit, `Effect::Search` walks the library for a creature card with the Dragon subtype and drops it onto the battlefield untapped. Tests: `dragons_approach_tutors_dragon_with_four_in_graveyard`, `dragons_approach_does_not_offer_tutor_without_four_named_in_graveyard`. |
| Defiant Strike | {W} | ✅ | Push XXXVII (NEW, `stx::extras`): Instant. Target creature you control gets +1/+0 EOT + Draw a card. Classic white cantrip-pump, same template as Charge Through (G) and Make Your Mark (W). Test: `defiant_strike_pumps_friendly_and_draws`. |
| Divine Gambit | {2}{W} | 🟡 | Push XXXVII (NEW, `stx::extras`): Instant. Exile target nonland permanent. The "its controller may put a permanent from hand to the battlefield" gift-back rider is omitted (no "opp may put a permanent from hand" decision shape). Body wires the exile half faithfully — a pure 3-mana white removal spell. Test: `divine_gambit_exiles_creature`. |
| Cram Session | {3}{W} | ✅ | Push XXXVII (NEW, `stx::extras`): Instant. You gain 5 life. Flashback {5}{W} via `Keyword::Flashback`. The printed "target player" prompt is collapsed to "you" — same multi-target collapse used by most STX lifegain spells. Test: `cram_session_gains_five_life_and_has_flashback`. |
| Expanded Anatomy | {3}{G} | ✅ | Push (modern_decks, NEW, `stx::lessons`): Sorcery — Lesson. "Put two +1/+1 counters on target creature." Wired as a single `AddCounter +1/+1 × 2` against a Creature target. Green's body-Lesson, slots alongside Guiding Voice (+1/+1 + Learn). Test: `expanded_anatomy_lands_two_counters_on_target_creature`. |
| Mercurial Transformation | {2}{U} | 🟡 | Push (modern_decks, NEW, `stx::lessons`): Sorcery. "Target creature or artifact becomes a blue Frog with base power and toughness 3/3 and loses all abilities." Wired via the engine's `Effect::SetBasePT` layer-7b primitive (same path as Square Up). The "loses all abilities" rider is omitted (no clear-abilities continuous primitive — tracked in TODO.md). The base-P/T override is the headline play pattern — shrinking a 7/7 down to 3/3 closes most combat math. Test: `mercurial_transformation_sets_target_to_three_three_eot`. |
| Crux of Fate | {3}{B}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`, STA reprint): Sorcery. Two-mode `ChooseMode`: mode 0 destroys each Dragon, mode 1 destroys each non-Dragon creature. Wired via `ForEach(Selector::EachPermanent(filter))` + `Destroy` for each mode; the non-Dragon filter uses `SelectionRequirement::Not(HasCreatureType(Dragon))`. Tests: `crux_of_fate_mode_zero_destroys_dragons`, `crux_of_fate_mode_one_destroys_non_dragons`. |
| Pestilent Cauldron | {1}{B} | 🟡 | Push (modern_decks, NEW, `stx::extras`): Artifact (MDFC front-face only). `{2}, {T}, Sacrifice this artifact: Each player mills four cards. Each opponent loses 3 life and you gain 3 life.` Wired as a `sac_cost: true` activation with `Seq(Mill 4 each, Drain 3)`. The "transform-from-graveyard" rider to the back-face Restorative Burst is omitted pending the cast-from-graveyard pipeline for MDFCs (engine's `cast_spell_back_face` walks hand only). Test: `pestilent_cauldron_sac_mills_and_drains`. |
| Eureka Moment | {2}{G}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`): Quandrix Instant. "Draw two cards. You may put a land card from your hand onto the battlefield tapped." Wired as `Seq(Draw 2, MayDo(Move land from hand to bf tapped))` — same shape as Embrace the Paradox's draw-3 sibling. AutoDecider declines the land-drop; ScriptedDecider can opt in for tests. Tests: `eureka_moment_draws_two_cards`, `eureka_moment_optional_land_drop_with_scripted_decider`. |
| Teach by Example | {1}{U}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`): Prismari Instant. "Copy target instant or sorcery spell. You may choose new targets for the copy." Wired via `Effect::CopySpell { what: target_filtered(IsSpellOnStack & (Instant ∨ Sorcery)) }` — same primitive as Galvanic Iteration but fully target-driven (any IS on the stack, not just the topmost). Test: `teach_by_example_copies_target_instant` (Bolt at P1 → both Bolt + copy deal 3 dmg each = 6 total). |
| Manifold Key | {1} | ✅ | Push (modern_decks, NEW, `stx::extras`): colorless artifact. "{1}, {T}: Target creature can't be blocked this turn. / {T}: Untap target artifact." Two activated abilities wired faithfully via `Effect::GrantKeyword(Unblockable, EOT)` and `Effect::Untap`. Voltaic-Key/Aether-Key-style infinite-mana enabler in any artifact deck. Tests: `manifold_key_grants_unblockable_to_target_creature`, `manifold_key_untaps_target_artifact`, `manifold_key_is_a_one_mana_artifact_with_two_abilities`. |
| Leyline Invocation | {3}{G}{G} | ✅ | Push (modern_decks, NEW, `stx::extras`): Quandrix Instant. "Target creature you control gets +X/+X and gains trample until end of turn, where X is the number of lands you control." Wired as `Seq(PumpPT(+X/+X with X = lands you control), GrantKeyword(Trample EOT))`. The X scales with live land count — Quandrix's finisher pump for ramping shells. Test: `leyline_invocation_pumps_by_lands_you_control`. |
| Spitfire Lagac | {2}{R}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`): Lorehold Creature — Lizard, 3/3. "Magecraft — Whenever you cast or copy an instant or sorcery spell, this deals 2 damage to each opponent." Wired via `magecraft(DealDamage(2) → EachOpponent)` — the burn-only Magecraft template. Tests: `spitfire_lagac_magecraft_burns_each_opp`, `spitfire_lagac_is_a_four_mana_three_three_lizard`. |
| Settle the Score | {3}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`): Witherbloom Sorcery. "Destroy target creature. Put two loyalty counters on a planeswalker you control." Wired as `Seq(Destroy + AddCounter(Loyalty, 2) on auto-picked friendly planeswalker)`. The second clause silently no-ops if the controller has no PW. Test: `settle_the_score_destroys_creature_and_adds_loyalty`. |
| Pursuit of Knowledge | {1}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`): Silverquill Enchantment. "Whenever you draw a card, you may put a study counter on this enchantment. / Remove four study counters from this enchantment and sacrifice it: Draw three cards." Engine approximations: (a) the "study counter" is mapped to `CounterType::Charge` (no `Study` counter type yet — same approximation as Diary of Dreams); (b) the activation gates on `Predicate::ValueAtLeast(CountersOn(This, Charge), 4)` and uses `sac_cost: true` to drain the enchantment on use, so the "remove 4" clause is approximated as "have 4+ + sac". Tests: `pursuit_of_knowledge_accumulates_charge_counter_on_draw_action`, `pursuit_of_knowledge_activation_requires_four_charge_counters`. |
| Divide by Zero | {1}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`): Quandrix Instant. "Return target spell or nonland permanent to its owner's hand. Learn." Wired as `Seq(Move(target → owner's hand), Draw 1)` — Learn approximated as Draw 1 (Lesson sideboard ⏳). The target filter is `IsSpellOnStack ∨ (Permanent & Nonland)` so the spell can hit either a stack spell or a nonland permanent. Tests: `divide_by_zero_bounces_permanent_and_cantrips`, `divide_by_zero_is_a_two_mana_instant`. |
| Exsanguinate (STA reprint) | {X}{B}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`): black X-cost drain finisher (Strixhaven Mystical Archive reprint). "Each opponent loses X life. You gain life equal to the life lost this way." Wired faithfully via `Effect::Drain { from: EachOpponent, to: You, amount: XFromCost }`. At X=10 this is a kill in any black shell. Test: `exsanguinate_drains_each_opp_by_x`. |
| Fire Prophecy (STA reprint) | {1}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`): red burn-and-cantrip (Strixhaven Mystical Archive reprint). "Fire Prophecy deals 3 damage to target creature or planeswalker. Put a card from your hand on the bottom of your library. Draw a card." Wired as `Seq(DealDamage(3) → creature/PW, PutOnLibraryFromHand 1, Draw 1)`. The "bottom" target of the put-on-library is approximated as "top" (engine `PutOnLibraryFromHand` defaults to top; a `LibraryPosition::Bottom` primitive bump is a future refactor). Net card advantage matches: -1 (hand-to-library) + 1 (draw) = 0, just trading a stale card for a fresh draw. Test: `fire_prophecy_deals_three_and_cantrips`. |
| Maelstrom Muse | {3}{U}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`): 3/3 Djinn Wizard with Flying. "Magecraft — Whenever you cast or copy an instant or sorcery spell, draw a card, then discard a card. If five or more mana was spent to cast that spell, draw two cards instead, then discard a card." Wired via `shortcut::opus_trigger`: small body = `Seq(Draw 1, Discard 1)`; big body = `Seq(Draw 2, Discard 1)`. The discard surfaces `Decision::Discard` (AutoDecider picks first hand card); ScriptedDecider can target a specific discard. Tests: `maelstrom_muse_opus_loots_on_small_cast`, `maelstrom_muse_is_a_five_mana_three_three_flying_djinn_wizard`. |
| Approach of the Second Sun (STA reprint) | {6}{W}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`): white finisher Sorcery (Strixhaven Mystical Archive). "If you've cast another spell named Approach of the Second Sun this game, you win the game. Otherwise, put this card seventh from the top of your owner's library and you gain 7 life." Wired via new `Effect::WinGame { who: PlayerRef }` primitive (CR 104.2a) + `Predicate::SameNamedInZoneAtLeast { who: You, zone: Graveyard, at_least: 1 }`. The "seventh from top of library" library positioning is approximated as "to graveyard" (the engine doesn't model the exact-position-in-library mechanic yet; the lifegain path keeps the spell as a payoff for first cast, with the win triggered by the predicate when a second cast occurs after the first has been moved to graveyard). Tests: `approach_of_the_second_sun_gains_seven_life_on_first_cast`, `approach_of_the_second_sun_wins_game_when_cast_with_one_in_graveyard`. |
| Resurrection (STA reprint) | {2}{W}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`): basic white reanimation (Strixhaven Mystical Archive reprint, Alpha original). "Return target creature card from your graveyard to the battlefield." Wired as a single `Effect::Move { target: Creature → Battlefield(You) }`. Same primitive shape as Reanimate but at 4 mana without the life cost. Test: `resurrection_returns_creature_card_from_graveyard`. |
| Adventurous Impulse (STA reprint) | {G} | ✅ | Push (modern_decks, NEW, `stx::extras`): green cantrip (Strixhaven Mystical Archive reprint, Core 2021). "Look at the top three cards of your library. You may reveal a creature or land card from among them and put it into your hand. Put the rest on the bottom of your library in a random order." Wired via `Effect::RevealUntilFind { who: You, find: Creature ∨ Land, to: Hand, cap: 3, miss_dest: BottomRandom }`. The "may" optionality collapses to always-take when a match exists (declining would lose tempo). Test: `adventurous_impulse_finds_a_creature_in_top_three`. |
| Eladamri's Call (STA reprint) | {W}{G} | ✅ | Push (modern_decks, NEW, `stx::extras`): Selesnya creature tutor (Strixhaven Mystical Archive reprint, Planeshift). "Search your library for a creature card, reveal it, put it into your hand, then shuffle." Wired via `Effect::Search { filter: Creature, to: Hand(You) }`. The auto-decider declines; a `ScriptedDecider::new([DecisionAnswer::Search(Some(card))])` picks the target creature. Tests: `eladamris_call_tutors_creature_into_hand`, `eladamris_call_is_a_two_mana_wg_instant`. |
| Yawning Fissure (STA reprint) | {3}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`): mass land-attack (Strixhaven Mystical Archive reprint, Mercadian Masques). "Each opponent sacrifices a land." Wired via `ForEach(EachOpponent) → Sacrifice(1, Land)` with `PlayerRef::Triggerer` scope inside the body — same per-player-sac pattern as Pox Plague. Each opponent's auto-decider picks the cheapest land. Tests: `yawning_fissure_each_opp_sacs_a_land`, `yawning_fissure_is_a_four_mana_red_sorcery`. |
| Cleansing Wildfire (STA reprint) | {1}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`): land-destroy-with-cantrip (Strixhaven Mystical Archive reprint, Zendikar Rising). "Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield, then shuffle. Draw a card." Wired as `Seq(Destroy → Search via ControllerOf(Target) → Draw 1)`. The "may search" optionality is honored by the engine's `Effect::Search` decider chain. Tests: `cleansing_wildfire_destroys_land_and_draws`, `cleansing_wildfire_is_a_two_mana_red_sorcery`. |
| Tendrils of Agony (STA reprint) | {2}{B}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`): Storm drain finisher (Strixhaven Mystical Archive reprint, Scourge). "Target opponent loses 2 life and you gain 2 life. Storm (When you cast this spell, copy it for each other spell cast before it this turn. You may choose new targets for the copies.)" Storm wired via `Effect::Repeat { count: StormCount + 1, body: Drain 2 from EachOpponent → You }` — equivalent to N+1 resolutions of "drain 2" where N is the spells-cast-before count. At StormCount=4 (Tendrils as fifth spell), drain fires 5 × 2 = 10 life shifted. Tests: `tendrils_of_agony_drains_two_with_no_storm`, `tendrils_of_agony_storm_drain_scales`. |
| Saw It Coming (STA reprint) | {2}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`): foretell counterspell (Strixhaven Mystical Archive reprint, Kaldheim). "Counter target spell. Foretell {1}{U}." Wired as a vanilla `Effect::CounterSpell` at the {2}{U} regular cost; Foretell {1}{U} discount is engine-wide ⏳ (no Foretell alt-cost primitive — would need a turn-delayed alt-cost discount). Tests: `saw_it_coming_counters_target_spell`, `saw_it_coming_is_a_three_mana_blue_instant`. |
| Increasing Vengeance (STA reprint) | {R}{R} | ✅ | Push (modern_decks): copy-spell instant (Strixhaven Mystical Archive reprint, Innistrad). "Copy target instant or sorcery spell you control. You may choose new targets for the copy. If this spell was cast from a graveyard, copy that spell twice instead." All printed clauses now ship — both copy paths are wired via `Effect::If { cond: CastFromGraveyard, then: CopySpell(2), else_: CopySpell(1) }`. The new `Predicate::CastFromGraveyard` (push modern_decks) reads `EffectContext.cast_from_hand` which is stamped at spell-resolution time from the resolving `CardInstance.cast_from_hand` flag — false for flashback / Yawgmoth's Will-style cast-from-gy paths. Tests: `increasing_vengeance_copies_target_instant` (hand cast → single copy), `increasing_vengeance_double_copies_when_flashed_back_from_graveyard` (synthesized Flashback {R}{R} → double copy + exile-on-resolve per CR 702.34a), `increasing_vengeance_is_a_two_mana_red_instant`. |
| Quench | {1}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`): {1}{U} tempo counter (STX uncommon). "Counter target spell unless its controller pays {1}." Wired via the engine's existing `Effect::CounterUnlessPaid` primitive (same as Mana Leak / Whirlwind Denial). Tests: `quench_counters_spell_when_opp_cant_pay`, `quench_is_a_two_mana_blue_instant`. |
| Dueling Coach | {1}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`): 1/2 Human Cleric (STX uncommon). "When this creature enters, put a +1/+1 counter on target creature you control. / {2}{W}: Put a +1/+1 counter on each creature you control with a +1/+1 counter on it." Counter-snowball synergy creature. ETB target uses `target_filtered(Creature & ControlledByYou)`; the activated ability fans counters out via `ForEach(EachPermanent(Creature & ControlledByYou & WithCounter(+1/+1))) → AddCounter(TriggerSource, +1/+1)`. Tests: `dueling_coach_etb_lands_counter_on_friendly`, `dueling_coach_activation_doubles_counters`, `dueling_coach_is_a_two_mana_human_cleric`. |
| Mizzium Mortars (STA reprint) | {1}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`): {1}{R} Sorcery (Strixhaven Mystical Archive reprint, originally Return to Ravnica). "Mizzium Mortars deals 4 damage to target creature. / Overload {4}{R}{R}". Single-target body wired via `Effect::DealDamage 4 → Creature target`. Overload alt cost is engine-wide ⏳ (no Overload primitive — same gap as Burst Lightning kicker, Devastating Mastery alt mode). Body-mode burn at {1}{R} is the headline play pattern in any Lorehold / Prismari shell. Tests: `mizzium_mortars_burns_target_creature`, `mizzium_mortars_is_a_two_mana_red_sorcery`. |
| Electrolyze (STA reprint) | {1}{U}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`): {1}{U}{R} Instant (Strixhaven Mystical Archive reprint, originally Guildpact). "Electrolyze deals 2 damage divided as you choose among one or two targets. Draw a card." Single-target body wired via `Seq(DealDamage 2 → Creature/Player/PW, Draw 1)`. "Divided as you choose among one or two targets" multi-target divided-damage rider collapses to a single target (engine-wide gap shared with Magma Opus ✅, Crackle with Power ✅). Tests: `electrolyze_deals_two_damage_and_draws`, `electrolyze_targets_a_player_for_two_damage`. |
| Show of Aggression | {2}{R}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`): {2}{R}{R} Sorcery (STX 2021). "Creatures you control get +2/+0 and gain haste until end of turn." Wired via `Effect::ForEach(Creature & ControlledByYou)` + `Seq(PumpPT(+2/+0 EOT), GrantKeyword(Haste EOT))`. A 4-mana sweeper-style pump that turns a stalled board into immediate lethal threats. Test: `show_of_aggression_pumps_each_friendly_creature_and_grants_haste`. |
| Past in Flames (STA reprint) | {3}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`): {3}{R} Sorcery (Strixhaven Mystical Archive reprint, originally Innistrad). "Each instant and sorcery card in your graveyard gains flashback until end of turn. The flashback cost is equal to its mana cost. / Flashback {4}{R}". Approximated as a mass `Move(all IS cards in your gy → hand)` since the engine has no transient per-card Flashback grant. The printed Oracle's Flashback cost = mana cost is preserved (re-casting from hand pays exactly the mana cost). Flashback {4}{R} on Past in Flames itself is honored via `Keyword::Flashback`. Tests: `past_in_flames_returns_instants_and_sorceries_from_graveyard_to_hand`, `past_in_flames_has_flashback_keyword`. |
| Inspired Idea | {1}{U}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`): {1}{U}{U} Sorcery (synthesised; STX flavor of the M11 Inspired Idea). "Draw three cards, then put two cards from your hand on top of your library." Wired as `Seq(Draw 3, PutOnLibraryFromHand 2)`. Classic blue dig-and-stack. Test: `inspired_idea_draws_three_then_stacks_two_on_top`. |
| Resurgent Belief | {3}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`): {3}{W} Sorcery (STX 2021). "Return all enchantment cards from your graveyard to the battlefield. / Flashback—{4}{W}, exile a card from your graveyard." Mass `Move(all enchantments in your gy → bf)` via `Selector::CardsInZone`. Flashback half is approximated as a plain `Keyword::Flashback` at {4}{W} — the printed "exile a card from your graveyard" additional cost is engine-wide ⏳ (no alt-cost-with-gy-exile primitive). Test: `resurgent_belief_returns_each_enchantment_from_graveyard`. |
| Academic Dispute | {R} | ✅ | Push (modern_decks, NEW, `stx::extras`): {R} Instant (STX 2021). "Target creature you control gets +1/+0 until end of turn. It fights target creature you don't control. / Learn." Wired as `Seq(PumpPT(+1/+0 EOT, slot 0 friendly), Fight(slot 0 vs auto-picked opp creature), Draw 1 [Learn approximation])`. Test: `academic_dispute_pumps_friendly_and_fights_opp_creature`. |
| Enthusiastic Study | {1}{G} | ✅ | Push (modern_decks, NEW, `stx::extras`): {1}{G} Instant (STX 2021). "Target creature gets +2/+2 until end of turn. If you've cast another spell this turn, that creature gains trample until end of turn." Wired as `Seq(PumpPT(+2/+2 EOT), If(SpellsCastThisTurnAtLeast(2)) → GrantKeyword(Trample EOT))`. Trample rider gated on the second-spell-this-turn predicate. Tests: `enthusiastic_study_pumps_target_creature_and_grants_trample_after_second_spell`, `enthusiastic_study_skips_trample_on_first_spell_this_turn`. |
| Light of Promise | {3}{W} | ✅ (was 🟡) | Push (modern_decks): STX 2021. "Whenever you gain life, put that many +1/+1 counters on target creature you control." The "that many" rider **now lands** via the new `Value::TriggerEventAmount` primitive. The trigger dispatcher (`dispatch_triggers_for_events`) extracts the firing `GameEvent::LifeGained`'s `amount` field, threads it onto the `StackItem::Trigger.event_amount` slot, and the resolving trigger body's `Effect::AddCounter { amount: TriggerEventAmount }` reads it via `EffectContext.event_amount`. Incidental 1-life-per-gain (Pest-style drains) drops 1 counter exactly; lump-sum gains (Bookwurm 4-life, Beledros's lifelink swings) correctly scale. Same primitive unblocks any future "that many"-style payoff (Karametra's Acolyte, Heliod, Sun-Crowned variants). Tests: `light_of_promise_is_a_four_mana_white_enchantment`, `light_of_promise_adds_counter_on_lifegain_event`, `light_of_promise_scales_with_lump_sum_lifegain`. |
| Damnable Pact (STA reprint) | {X}{B}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`): {X}{B}{B} Sorcery (STA reprint, originally Magic Origins). "Target player draws X cards and loses X life." Both clauses read `Value::XFromCost`. At X≥10 this is a kill against a low-life player or a self-mill engine for blue-black control. Test: `damnable_pact_at_x_three_draws_three_loses_three`. |
| Shore Up (STA reprint) | {U} | ✅ | Push (modern_decks, NEW, `stx::extras`): {U} Instant (STA reprint, originally Modern Horizons). "Untap target permanent. It gains hexproof until end of turn. / Flashback {3}{U}." Combat trick that dodges removal on a critical turn. Tests: `shore_up_untaps_and_grants_hexproof`, `shore_up_has_flashback_keyword`. |
| Symbol of Strength (STA reprint) | {2}{G} | ✅ | Push (modern_decks, NEW, `stx::extras`): {2}{G} Sorcery (STA reprint, originally Future Sight). "Target creature gets +2/+2 until end of turn. Draw a card. / Flashback {3}{G}." Pump-and-cantrip combat trick. Tests: `symbol_of_strength_pumps_two_two_and_draws`. |
| Magmatic Sinkhole (STA reprint) | {1}{B}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`): {1}{B}{R} Sorcery (STA reprint). "Surveil 2, then this deals 4 damage to target creature or planeswalker." The Delve alt-cost from some printings is omitted (no exile-from-gy alt-cost-reduction primitive). Body fully ships the printed primary effect. Test: `magmatic_sinkhole_surveils_and_deals_four_damage`. |
| Sevinne's Reclamation (STA reprint) | {2}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`): {2}{W} Sorcery (STA reprint, originally Commander 2019). "Return target permanent card with mana value 3 or less from your graveyard to the battlefield. If this spell was cast from a graveyard, copy it twice. You may choose new targets for the copies. / Flashback {5}{W}." The "if cast from a graveyard, copy twice" rider is **fully wired** via the existing `Predicate::CastFromGraveyard` primitive (push: modern_decks) — at hand-cast: 1 reanimation; at flashback-cast: 1 reanimation + 2 copies. Tests: `sevinnes_reclamation_returns_low_mv_permanent_from_graveyard`, `sevinnes_reclamation_rejects_high_mv_target`, `sevinnes_reclamation_has_flashback_keyword`. |
| Anger (STA reprint) | {2}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`): {2}{R} Creature — Incarnation, 2/2 (Judgment / STA reprint). "Haste / As long as Anger is in your graveyard and you control a Mountain, creatures you control have haste." The graveyard-resident static anthem is **wired** via the new `graveyard_anthem_for_name` helper-table walked by `GameState::compute_battlefield`. While Anger sits in a player's graveyard, the engine emits a layer-6 `AddKeyword(Haste)` continuous effect on every creature the gy-owner controls — but only when the owner also controls at least one Mountain on the battlefield. The Mountainwalk evasion is omitted (no landwalk primitive). Tests: `anger_is_a_three_mana_two_two_incarnation_with_haste`, `anger_in_graveyard_grants_haste_with_mountain`, `anger_in_graveyard_requires_mountain_to_grant_haste`, `anger_only_grants_haste_to_its_owners_creatures`. |
| Wonder (STA reprint) | {3}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`): {3}{U} Creature — Incarnation, 2/2 (Judgment / STA reprint). "Flying / As long as Wonder is in your graveyard and you control an Island, creatures you control have flying." Same `graveyard_anthem_for_name` helper-table path as Anger — Island → Flying. Tests: `wonder_is_a_four_mana_two_two_flying_incarnation`, `wonder_in_graveyard_grants_flying_with_island`, `wonder_in_graveyard_requires_island_to_grant_flying`. |
| Brawn (STA reprint) | {2}{G} | ✅ | Push (modern_decks, NEW, `stx::extras`): {2}{G} Creature — Incarnation, 3/3 (Judgment / STA reprint). "Trample / As long as Brawn is in your graveyard and you control a Forest, creatures you control have trample." Same helper-table path as Anger — Forest → Trample. Tests: `brawn_is_a_three_mana_three_three_trample_incarnation`, `brawn_in_graveyard_grants_trample_with_forest`, `brawn_in_graveyard_requires_forest_to_grant_trample`. |
| Valor (STA reprint) | {1}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`): {1}{W} Creature — Incarnation, 2/2 (Judgment / STA reprint). "First strike / As long as Valor is in your graveyard and you control a Plains, creatures you control have first strike." Same helper-table path as Anger — Plains → First Strike. Tests: `valor_is_a_two_mana_two_two_first_strike_incarnation`, `valor_in_graveyard_grants_first_strike_with_plains`, `valor_in_graveyard_requires_plains_to_grant_first_strike`. |
| Deep Analysis (STA reprint) | {3}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`): {3}{U} Sorcery (STA reprint, originally Torment). "Target player draws two cards and loses 2 life. / Flashback—{1}{U}, Pay 3 life." Body draws 2 + loses 2 life against the target player. Flashback {1}{U} wired via `Keyword::Flashback`. The "Pay 3 life" additional cost on the flashback path is an engine-wide alt-cost-with-life-cost gap. Tests: `deep_analysis_is_a_four_mana_blue_sorcery_with_flashback`, `_draws_two_and_loses_two_life`, `_can_target_opponent`. |
| Tribute to Hunger (STA reprint) | {2}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`): {2}{B} Instant (STA reprint, originally Time Spiral). "Target opponent sacrifices a creature. You gain life equal to its toughness." Wired via `Effect::SacrificeAndRemember` + the new `Value::SacrificedToughness` primitive (which reads the `GameState.sacrificed_toughness` scratch field stamped alongside `sacrificed_power`). The auto-picker chooses the opp's cheapest creature. Tests: `tribute_to_hunger_is_a_three_mana_black_instant`, `_sacrifices_opp_creature_and_gains_life_equal_to_toughness`, `_no_creature_to_sac_gives_no_life`. |
| Kasmina's Transmutation (STA reprint) | {1}{U}{U} | 🟡 | Push (modern_decks, NEW, `stx::extras`): {1}{U}{U} Sorcery (STA reprint, Strixhaven Loyalty). "Target creature loses all abilities and becomes a blue Frog with base power and toughness 1/1 until end of turn." Wired via `Effect::SetBasePT` (same layer-7b primitive as Square Up / Mercurial Transformation). The "loses all abilities" rider is omitted (no clear-abilities continuous primitive — tracked in TODO.md as the `StaticEffect::ClearAbilities` gap, shared with Mercurial Transformation). Tests: `kasminas_transmutation_is_a_three_mana_blue_sorcery`, `_sets_target_to_one_one_eot`. |
| Crippling Fear (STA reprint) | {3}{B} | 🟡 | Push (modern_decks, NEW, `stx::extras`): {3}{B} Sorcery (STA reprint, originally Conflux). "Choose a creature type. Creatures other than creatures of the chosen type get -3/-3 until end of turn." Approximated as the strictly-stronger universal -3/-3 (every creature gets it, including your own) since the engine has no choose-creature-type primitive. The headline play pattern is a 4-mana wrath that hits everything with toughness ≤ 3. Tests: `crippling_fear_is_a_four_mana_black_sorcery`, `_kills_two_toughness_creatures`, `_does_not_kill_high_toughness_creatures`. |
| Triskaidekaphile | {1}{U}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`): {1}{U}{U} Creature — Human Wizard, 3/4 (STX 2021). "When this creature enters, draw a card. / You have no maximum hand size. / At the beginning of your upkeep, if you have exactly 13 cards in your hand, you win the game." Wired via three engine primitives: ETB Draw 1 + `Effect::SetNoMaxHandSize` (flips `Player.no_maximum_hand_size`) + upkeep trigger gated on `Predicate::ValueEquals(HandSizeOf(You), Const(13))` resolving `Effect::WinGame { who: You }`. The `EventSpec.filter` predicate is now enforced by `fire_step_triggers` (engine bug fix — CR 603.2 intervening 'if' clause, half-implemented). Tests: `triskaidekaphile_is_a_three_mana_three_four_human_wizard`, `_etb_draws_a_card_and_lifts_max_hand_size`, `_wins_at_upkeep_with_exactly_thirteen_cards`, `_does_not_win_at_upkeep_with_other_hand_size`. |
| Excellent Education | {2}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`): {2}{W} Sorcery (STX 2021). "Target player gains 4 life and draws a card." Wired as `Seq(GainLife 4 → Target(0), Draw 1 → Target(0))`. Both clauses target the same chosen player. Tests: `excellent_education_gives_target_player_life_and_draw`, `_can_target_opponent`, `_is_a_three_mana_white_sorcery`. |
| Sproutback Trudge | {3}{G}{G} | ✅ | Push (modern_decks, NEW, `stx::extras`): {3}{G}{G} Creature — Plant, 5/6 (STX 2021). "When this creature enters, you gain X life, where X is the number of creature cards in your graveyard." ETB body reads `Value::CountOf(CardsInZone(You, Graveyard, Creature))`. Tests: `sproutback_trudge_is_a_five_mana_five_six_plant`, `_gains_life_per_creature_in_graveyard`, `_with_empty_graveyard_gains_zero_life`. |

### Shared / multi-college

| Card | Cost | Status | Notes |
|---|---|---|---|
| Inkling Summoning | {3}{W}{B} | ✅ | Sorcery (Lesson). Creates a 2/1 white-and-black Inkling token with flying. |
| Tend the Pests | {1}{B}{G} | ✅ | Sacrifice a creature; create X 1/1 Pest tokens (X = sacrificed power); "When this dies, gain 1 life" trigger now rides on the token via SOS-VI's `TokenDefinition.triggered_abilities`. |

### Iconic / legendary (`stx::iconic` + `stx::legends`)

Cards added in the latest push that didn't fit a single college file —
each college's flagship Dragon, plus a few cross-college staples.

| Card | Cost | Status | Notes |
|---|---|---|---|
| Strict Proctor | {1}{W} | 🟡 | 1/3 Spirit Cleric, Flying. ETB-tax replacement is omitted (no replacement-effect primitive). |
| Sedgemoor Witch | {2}{B}{B} | ✅ | 3/2 Human Warlock, Menace + Ward(1) keyword. Magecraft creates a Pest token. Ward enforcement still pending — keyword tag is correct. Test: `sedgemoor_witch_magecraft_creates_pest_token`. |
| Spectacle Mage | {U}{R} | ✅ | Push XXXI doc sync: Prowess is functional via the `effect::shortcut::prowess()` helper. Fires on every non-creature spell you cast, pumping the source +1/+1 EOT. Hybrid {U/R}{U/R} approximated as {U}{R}. |
| Mage Hunters' Onslaught | {2}{B}{B} | ✅ | Sorcery. Destroy target creature; draw a card. Test: `mage_hunters_onslaught_destroys_creature_and_draws_card`. |
| Galazeth Prismari | {2}{U}{R} | 🟡 | 3/4 Legendary Dragon Wizard, Flying. ETB creates a Treasure token (full real-card behaviour). The "artifacts you control are mana sources" static is still ⏳ (no `GrantActivatedAbility(applies_to)` primitive). Test: `galazeth_prismari_is_three_four_flying_dragon_with_etb_treasure`. |
| Beledros Witherbloom | {3}{B}{B}{G}{G} | ✅ (was 🟡) | Push XXXV (doc-sync): 6/6 Legendary Demon, Flying + Trample + Lifelink. The pay-10-life mass-untap activated ability has been fully wired since push XVIII via the `life_cost: 10` + `sorcery_speed: true` fields on `ActivatedAbility` + `Effect::Untap { what: EachPermanent(Land & ControlledByYou), up_to: None }`. The pre-flight life-cost gate rejects activation cleanly with `GameError::InsufficientLife` when life < 10. Tests: `beledros_witherbloom_pay_ten_life_untaps_all_lands`, `beledros_witherbloom_rejects_activation_with_insufficient_life`. |
| Hofri Ghostforge | {2}{R}{W} | 🟡 | Push XXXV: 3/4 Legendary Spirit Cleric. The "Other creatures you control get +1/+0" anthem **is now wired** via the new `SelectionRequirement::OtherThanSource` primitive flowing through `affected_from_requirement` and setting `AffectedPermanents::All.exclude_source: true` — matches the printed "other" wording. Tests: `hofri_ghostforge_anthem_buffs_other_creatures_by_one_zero`, `hofri_ghostforge_anthem_does_not_buff_self`, `hofri_ghostforge_anthem_does_not_buff_opp_creatures`, `hofri_ghostforge_anthem_expires_when_hofri_leaves`. The exile-on-death + return-as-1/1-Spirit cycle stays ⏳ pending a delayed-replacement-on-graveyard primitive. |
| Velomachus Lorehold | {3}{R}{R}{W} | 🟡 | 5/5 Legendary Dragon, Flying + Vigilance + Haste. Attack-trigger reveal-and-cast is ⏳ (cast-from-exile-without-paying primitive). |
| Tanazir Quandrix | {2}{G}{G}{U}{U} | ✅ (was 🟡) | Push XXXV (doc-sync): 5/5 Legendary Dragon, Flying + Trample. Both attack-trigger toughness doubling and ETB +1/+1-counter doubling have been wired since push XIX via `ForEach(Creature & ControlledByYou)` + `AddCounter(+1/+1, amount: CountersOn(TriggerSource, +1/+1))` for ETB, and `PumpPT(toughness = ToughnessOf(Target(0)))` for the attack rider. Tests: `tanazir_quandrix_five_five_flying_trample_dragon`, `tanazir_quandrix_attack_trigger_doubles_target_toughness`, `tanazir_etb_doubles_plus_one_counters`, `tanazir_etb_does_not_add_counters_to_counterless_creature`. |
| Shadrix Silverquill | {2}{W}{B} | ✅ (was 🟡) | Push XXXV: 4/4 Legendary Dragon, Flying + Double Strike. The choose-two-of-three attack trigger is now wired via `Effect::ChooseN { picks: [1, 2], modes: [..] }` — auto-picks mode 1 (+1/+1 counter on target creature) + mode 2 (mint two Inkling tokens). Mode 0 (draw a card) stays in `modes` for future mode-pick UI. The printed "you may choose the same mode more than once" CR 700.2d exception isn't honored by `ChooseN.picks` today; the auto-pick set is two distinct modes, sidestepping the corner. Tests: `shadrix_silverquill_attack_pumps_target_creature_and_mints_inklings`, `shadrix_silverquill_attack_does_not_trigger_on_opp_attack`. |

### Engine pieces driven by STX

- ✅ **`effect::shortcut::magecraft(effect)` helper** — bundles the
  spell-cast trigger with `cast_is_instant_or_sorcery()`, so card
  factories use a one-liner. Used by Eager First-Year and Witherbloom
  Apprentice; future Apprentices (Lorehold, Prismari, Quandrix) will
  reuse it.
- ✅ **Token death-trigger lifegain** — `TokenDefinition` now carries
  `triggered_abilities` (added in SOS push VI). The STX Pest token's
  "die → gain 1" trigger fires consistently for Pest Summoning, Tend
  the Pests, Hunt for Specimens. SOS Pest token's "attack → gain 1"
  rider also rides on every minted copy.
- ⏳ **Lesson sideboard model** — Eyetwitch, Hunt for Specimens, Pest
  Summoning all use Learn at some point. Currently approximated as
  `Draw 1`.
