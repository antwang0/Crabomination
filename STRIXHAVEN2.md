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
| SOS (255 cards) | 209 | 45 | 1 |
| STX (327 cards) | 545 | 12 | 0 |
| STA reprints (in STX boosters) | 46 | 0 | — |

Push (modern_decks, claude/modern_decks branch — latest revision —
**batch 29: 20 new iconic STX cards + 24 new tests + 2 SOS promotions
(Diary of Dreams + Soaring Stoneglider) + 2 new engine primitives
(`ActivatedAbility.self_counter_cost_reduction` for page-counter
discounts + `AlternativeCost.exile_from_graveyard_count` for
exile-N-from-gy additional costs)**):

A new batch adding 20 synthesised STX cards spread evenly across all
five colleges (4+ per school) using existing primitives + magecraft.
All in `stx::iconic`. Tests sit in `tests::stx`. Two SOS partial
promotions land via two clean engine additions: page-counter cost
reduction on activated abilities (Diary of Dreams) and exile-N-from-
graveyard alt-cost (Soaring Stoneglider's "or exile two cards from
your graveyard" additional cost variant). 2628 → 2656 total tests
(+28). All clippy-clean.

Prior push (batch 28):

A new batch adding 25 cards spread evenly across all five colleges
(5 per school) using existing primitives. Tests sit in `tests::stx`.
Plus one new engine primitive (`Selector::LastCreatedTokens` plural,
for multi-mint-then-counter cards like Fractal Spawning) and a fresh
audit of CR 114 (Emblems).

- **Silverquill (W/B)**:
  `silverquill_heraldist` ({1}{W}, 2/2 Human Soldier — ETB gain 1 life
  + mint 1/1 Inkling token with Flying),
  `inkling_spireguard` ({2}{W}, 2/3 Inkling Soldier Flying — ETB pump
  target friendly creature +1/+1 EOT),
  `silverquill_quillwitch` ({1}{B}, 2/2 Inkling Warlock — dies-trigger
  drain 2 vs each opp),
  `silverquill_inkpurge` ({1}{W}{B} Sorcery — Each opp sacs a creature
  + you gain 2 life via `ForEach(EachOpponent) → Sacrifice`),
  `inkrise_schoolwarden` ({3}{W}{B}, 3/4 Inkling Cleric Flying +
  Lifelink — ETB Draw 1).
- **Witherbloom (B/G)**:
  `witherbloom_vinekeeper` ({2}{B}{G}, 3/4 Plant Druid — ETB gain 2
  life + per-AnotherOfYours-dies gain 1 life),
  `pest_outcast` ({B}, 1/1 Pest Warlock — dies → gain 1 + draw 1),
  `witherbloom_drainscholar` ({B}{G}, 1/2 Plant Druid Lifelink —
  magecraft target creature -1/-1 EOT),
  `witherbloom_coatlcaller` ({2}{G}, 2/3 Human Druid Reach — ETB mints
  1 Pest token),
  `witherbloom_pestbreaker` ({3}{B}{G} Sorcery — Destroy target
  creature + mint 1 Pest).
- **Lorehold (R/W)**:
  `lorehold_pyresinger` ({1}{R}{W}, 2/2 Spirit Cleric — magecraft drain
  1 vs each opp),
  `lorehold_soulchanter` ({3}{W}, 3/2 Spirit Cleric Lifelink — ETB exile
  target card from a graveyard),
  `lorehold_flameherald` ({1}{R}, 2/1 Human Soldier Haste — ETB 1 dmg
  to any target),
  `lorehold_embercouncil` ({2}{R}{W} Sorcery — Create 2 Spirit tokens
  + 1 damage to each opp),
  `lorehold_cinderpriest` ({2}{R}, 2/3 Spirit Cleric — ETB +1/+1
  counter on target friendly + magecraft +1/+0 EOT to target friendly).
- **Quandrix (G/U)**:
  `quandrix_sumcaster` ({G}{U}, 1/2 Elf Wizard — magecraft MayDo Draw
  1 then Discard 1 looter),
  `fractal_multiplicand` ({2}{G}{U}, 0/0 Fractal Wizard with 3 +1/+1
  counters via `enters_with_counters`),
  `quandrix_calculus_mage` ({3}{G}{U}, 4/4 Elf Wizard — ETB Scry 2 +
  Draw 1 + magecraft +1/+1 counter on target Fractal),
  `quandrix_tidecaller` ({1}{U}, 1/3 Merfolk Wizard Flash — ETB Tap
  target creature),
  `fractal_spawning` ({2}{G}{U} Sorcery — mints 2 Fractal tokens and
  drops a +1/+1 counter on EACH via the new
  `Selector::LastCreatedTokens` plural primitive — both Fractals
  survive SBA as 1/1).
- **Prismari (U/R)**:
  `prismari_embershaper_wizard` ({2}{U}{R}, 2/3 Djinn Wizard Flying —
  ETB Treasure + Discard 1 + Draw 1),
  `prismari_magmaboon` ({2}{R} Sorcery — 3 dmg to target creature +
  mint Treasure),
  `prismari_tideburst` ({U}{R} Instant — Mana Leak rate counter unless
  pay {2} + Scry 1),
  `prismari_tempest_caller` ({1}{U}{R}, 2/2 Elemental Wizard Flying —
  magecraft self-pump +1/+0 EOT),
  `prismari_pyresurge_b28` ({3}{R} Sorcery — 3 dmg to any target +
  Draw 1).
- **Shared / cross-school** (`stx::extras`):
  `strixhaven_battle_cleric` ({W}, 2/1 Human Cleric — ETB gain 1 life),
  `strixhaven_researcher` ({2}{U}, 2/3 Human Wizard — ETB Scry 2),
  `strixhaven_combatant` ({1}{R}, 2/2 Human Warrior Haste — attack +1/+0
  EOT trigger),
  `strixhaven_druid` ({1}{G}, 2/2 Elf Druid — ETB Search basic land →
  hand),
  `strixhaven_drainsong` ({1}{B} Instant — drain 2 from target opp).

Engine improvements (push modern_decks batch 28):
- **`Selector::LastCreatedTokens` (plural)** — new selector that
  tracks every token created in the current effect resolution (not
  just the most recent). Wired alongside `last_created_token` in the
  CreateToken loop; resets at every resolution root start. Powers
  Fractal Spawning's "create 2 Fractals, put a +1/+1 counter on
  EACH of them" pattern faithfully (both Fractals get counters and
  survive SBA at 1/1). Same shape as the singular variant — works
  through `ForEach` fan-outs, replacement effects (counter doublers
  multiply per-token), and downstream selector chains.
- **CR 114 audit** — Emblems documented as ⏳ pending an
  `Effect::CreateEmblem` primitive + emblem-resident command-zone
  trigger dispatch. Tracked in `TODO.md`.

Tests: 30 new (one per new card, plus the LastCreatedTokens primitive
test). Total: 2589 → 2619 (+30). All clippy-clean.

Push (modern_decks, claude/modern_decks branch — earlier revision —
**batch 27: 22 new STX cards + 23 tests**):

A new batch adding 22 cards spread across all five colleges + mono /
shared shells, all using existing primitives. Tests sit in
`tests::stx`.

- **Lorehold (R/W)**:
  `lorehold_stonebrand` ({2}{R}{W}, 3/3 Spirit Soldier — ETB MayDo
  exile gy creature → mint Spirit token),
  `lorehold_bookbinder` ({3}{R}{W}, 4/4 Spirit Cleric — ETB recur IS
  from gy + team haste EOT),
  `lorehold_pyresmith` ({1}{R}, 2/1 Spirit Warrior First Strike — ETB
  1 dmg to any target),
  `lorehold_spirit_champion` ({3}{R}{W}, 4/3 Spirit Knight First
  Strike + Haste — "Other Spirits you control have first strike"
  tribal anthem via `StaticEffect::GrantKeyword` on `EachPermanent`).
- **Quandrix (G/U)**:
  `quandrix_geometer` ({1}{G}{U}, 2/3 Fractal Wizard — ETB +1/+1
  counter + magecraft self-pump),
  `quandrix_wavecaster` ({1}{G}{U}, 1/3 Merfolk Wizard — magecraft
  +1/+1 counter on target friendly),
  `quandrix_mathmage` ({2}{G}{U}, 3/3 Elf Wizard — ETB RevealUntilFind
  Creature/Land cap-4 → hand),
  `quandrix_counterstudent` ({1}{U}, 1/2 Elf Wizard —
  `{1}{G}{U},{T}: CounterAbility target`).
- **Silverquill (W/B)**:
  `silverquill_sentinel_cleric` ({2}{W}{B}, 3/3 Inkling Cleric —
  Flying + Vigilance vanilla),
  `silverquill_embodiment` ({2}{W}{B}, 3/3 Inkling Bard Flying — ETB
  drain 2 + per-other-creature-die gain 1 life),
  `silverquill_adjudicator` ({3}{W}, 2/4 Human Cleric Vigilance —
  ETB target opp creature gets -3/-0 EOT),
  `silverquill_drain_lord` ({2}{W}{B}, 3/3 Inkling Vampire Flying +
  Lifelink — on-lifegain trigger drains 1 each opp).
- **Witherbloom (B/G)**:
  `witherbloom_soilshaper` ({2}{B}{G}, 3/3 Plant Druid — ETB mill 2 +
  +1/+1 counter for each creature card in your gy),
  `witherbloom_plagueweaver` ({1}{B}{G}, 2/2 Plant Warlock — magecraft
  target creature -1/-1 EOT),
  `witherbloom_drain_mage` ({2}{B}, 2/2 Human Warlock — ETB drain 3),
  `witherbloom_pest_spawner` ({2}{B}{G}, 1/3 Plant Druid — ETB mint 2
  Pests + per-other-creature-die gain 1 life).
- **Prismari (U/R)**:
  `prismari_fireshaper` ({2}{U}{R}, 2/3 Elemental Wizard — ETB
  Treasure + magecraft 1 dmg to any target),
  `prismari_sparkbender` ({U}{R}, 2/2 Human Wizard — ETB loot 1),
  `prismari_wave_mage` ({1}{U}{R}, 2/2 Elemental Wizard — ETB
  Treasure + magecraft scry 1 + 1 dmg to any target).
- **Mono / shared**:
  `strixhaven_scry_wizard` ({2}{U}, 2/2 Human Wizard — ETB Scry 2 +
  magecraft Scry 1),
  `strixhaven_mage_hunter` ({2}{B}, 2/3 Human Assassin Deathtouch —
  `{T}: target player discards a chosen nonland`),
  `strixhaven_pop_quiz_sage` ({2}{W}, 2/3 Human Wizard — ETB Draw 2 +
  PutOnLibraryFromHand 1).

Tests: 23 new (one per new card minus a couple covered jointly).
Total: 2566 → 2589 (+23). All clippy-clean.

Push (modern_decks, claude/modern_decks branch — earlier revision —
**batch 26: 7 new STX cards + 2 promotions + 7 tests**):

A follow-on sweep adding 3 new Lessons and 4 iconic cross-college cards
using existing primitives. Plus 2 🟡 → ✅ promotions backed by new
engine features.

- **3 new Lessons** (`stx::lessons`):
  `pest_studies` ({1}{B}{G} Sorcery — Lesson: mint 2 Pest tokens),
  `lecture_in_strategy` ({1}{R}{W} Sorcery — Lesson: team +1/+1 +
  Vigilance EOT),
  `advanced_cartography` ({1}{G}{U} Sorcery — Lesson: ramp basic +
  Scry 2).
- **4 new iconic cards** (`stx::iconic`):
  `bombastic_strixhaven_mage` ({2}{R}, 2/3 — ETB 2 dmg + magecraft
  1 dmg ping),
  `mage_hunters_strike` ({1}{B} instant: -3/-3 EOT),
  `mascot_researcher` ({2}{G}, 2/2 — ETB +1/+1 counters on another
  + self),
  `strixhaven_tutor` ({2}{U}, 2/2 — ETB Scry 2 + Draw 1).

**Engine improvements / promotions**:

- **Vanquish the Horde** 🟡 → ✅ — new `CardDefinition.affinity_filter`
  slot. Reads at every cast path; CR 601.2f / 117.7c generic-only.
- **Witherbloom, the Balancer** 🟡 → ✅ — new `StaticEffect::
  GrantAffinityToISSpells { permanent_filter }` static covers the
  IS-spell-grant clause. Both Affinity-for-creatures halves now ship.

Tests: 7 new (1 per batch-26 card) + 5 affinity tests. Total: 2557
→ 2566.

Push (modern_decks, claude/modern_decks branch — prior revision —
**batch 25: 28 new synthesised STX cards across all 5 colleges + 28
functionality tests**):

A 28-card follow-up sweep continuing the Strixhaven buildout. Silverquill
gets a heavier 7-card allocation per the user's "finish one school first"
guidance; the four other colleges get 5-6 cards each. All cards use
existing magecraft / drain / token / counter / lifegain / Search / Treasure
primitives — no new engine features required.

- **7 Silverquill (W/B) additions** (`stx::silverquill`):
  `silverquill_inkmaster` ({1}{W}{B}, 2/2 Inkling Wizard with magecraft
  drain-each-opp 1),
  `inkling_censurer` ({2}{W}, 2/3 Vigilance with ETB tap-opp-creature),
  `silverquill_loredrain` ({2}{B}, instant: -2/-2 EOT + gain 2 life),
  `inkling_verseweaver` ({3}{W}{B}, 3/3 Flying with magecraft create
  2/1 Inkling token),
  `silverquill_hightutor` ({1}{W}, sorcery: search lib for IS card MV≤2
  to hand),
  `silverquill_lifebinder` ({2}{W}, 2/3 Lifelink with ETB +2 life),
  `inkling_drainmaster` ({3}{B}, 2/4 with ETB drain target opp 3).
- **6 Witherbloom (B/G) additions** (`stx::witherbloom`):
  `witherbloom_marshcaster` ({1}{B}, 1/2 with ETB Scry 1 + magecraft
  drain-each-opp 1),
  `pest_wrangler` ({2}{G}, 2/3 with ETB Pest token),
  `witherbloom_toxicaster` ({B}{G}, 1/1 Deathtouch with magecraft
  +0/+1 self-pump),
  `witherbloom_soilbleeder` ({3}{B}{G}, 4/3 with ETB MayDo sac-another
  → drain target 3),
  `witherbloom_handburner` ({2}{B}, sorcery: target opp discards 2 +
  gain 2 life),
  `pest_brood_mother` ({3}{B}{G}, 3/4 — ETB mints 2 Pests + each Pest
  dying drains opp 1).
- **5 Lorehold (R/W) additions** (`stx::lorehold`):
  `lorehold_spellrunner` ({1}{R}, 2/2 Haste with magecraft self-pump
  +1/+0),
  `lorehold_battlecaster` ({3}{R}{W}, 3/3 Trample with ETB Spirit +
  attack-trigger +1/+1 counter),
  `lorehold_pyresurge` ({R}{W}, instant: 2 damage to any target +
  gain 1 life),
  `spirit_sparkguard` ({2}{W}, 2/4 Spirit Cleric Vigilance with
  +1/+1 anthem to other Spirits),
  `lorehold_outburst` ({2}{R}{W}, sorcery: mint 2 Spirits + team
  +1/+0 EOT).
- **5 Prismari (U/R) additions** (`stx::prismari`):
  `prismari_sparkdrake` ({3}{U}{R}, 3/3 Flying+Haste),
  `prismari_lavalifter` ({2}{R}, 3/2 with ETB Treasure token),
  `prismari_spelltheorist` ({1}{U}{R}, 2/2 with ETB loot 1),
  `prismari_stormwriter` ({2}{U}{R}, instant: 3 damage to target
  creature + draw 1),
  `prismari_igniter` ({1}{R}, 2/1 Haste with magecraft 1-damage-any-
  target ping).
- **5 Quandrix (G/U) additions** (`stx::quandrix`):
  `quandrix_pondweaver` ({G}{U}, 1/1 Elf Druid with magecraft Scry 1),
  `quandrix_fractalseed` ({1}{G}{U}, 2/2 Fractal with ETB +1/+1
  counters per IS in gy),
  `quandrix_mapmaker` ({2}{G}, 2/3 with ETB search basic land tapped),
  `quandrix_fractalwave` ({2}{G}{U}, sorcery: mint Fractal token with
  X counters where X = IS in your gy),
  `fractal_theorist` ({2}{G}{U}, 3/3 Trample with magecraft +1/+1
  counter on target Fractal).

Total test count: 2509 → 2537+. Total STX corpus per audit: 449+13=462
→ 477+13=490.

Push (modern_decks, claude/modern_decks branch — prior revision —
**batch 24: 25 new synthesised STX cards (5 per college) + 24
functionality tests**):

A 25-card follow-up sweep across all five colleges using existing
magecraft / drain / token / counter / lifegain primitives. No new engine
features required.

- **5 Silverquill (W/B) additions** (`stx::silverquill`):
  `silverquill_notetaker` ({1}{W}, 1/2 — ETB Scry 1 + magecraft MayDo
  draw),
  `inkling_pamphleteer` ({W}{B}, 2/2 Flying with ETB drain 1),
  `silverquill_indictment` ({2}{W}{B}, instant: exile MV≤3 creature +
  gain 2),
  `inkling_banner_bearer` ({3}{W}, 2/3 Flying+Vigilance Inkling lord
  with +1/+0 anthem to other Inklings),
  `silverquill_tribunal` ({2}{B}, sorcery: target opp sacs a creature +
  gain 1).
- **5 Witherbloom (B/G) additions** (`stx::witherbloom`):
  `witherbloom_aspersor` ({B}{G}, instant: -2/-1 EOT + gain 1),
  `pest_reanimator` ({2}{B}{G}, 3/2 with ETB return ≤3-MV gy creature
  to hand),
  `witherbloom_spore_master` ({3}{B}{G}, 4/4 with ETB mint 2 Pests),
  `witherbloom_withercut` ({1}{B}{G}, instant: -3/-1 EOT + draw 1),
  `pest_cultivator_adept` ({2}{B}{G}, 2/3 with ETB Pest + magecraft
  +1/+1 counter on self).
- **5 Lorehold (R/W) additions** (`stx::lorehold`):
  `lorehold_pyrostriker` ({1}{R}, 2/1 Haste with attacks-trigger exile-
  gy + 1 dmg),
  `lorehold_soulshaper` ({2}{W}, 1/4 Vigilance with ETB Spirit token),
  `lorehold_ironhand` ({3}{R}{W}, 4/4 First Strike+Trample with ETB 2
  dmg to creature),
  `lorehold_revival` ({2}{R}{W}, sorcery: reanimate creature with haste
  EOT),
  `lorehold_sparkflare` ({R}, instant: 2 dmg to any target — Shock at
  the {R} slot).
- **5 Quandrix (G/U) additions** (`stx::quandrix`):
  `quandrix_logician` ({G}{U}, 2/2 with ETB Scry 2 + magecraft +1/+1
  counter on target Fractal),
  `fractal_echoist` ({2}{G}{U}, 1/1 Fractal with ETB ×N counters where
  N = IS in gy + attacks-trigger growth),
  `quandrix_mathenotaur` ({3}{G}{U}, 4/4 Trample with ETB-doubles
  +1/+1 counters on target),
  `fractal_surge` ({1}{G}{U}, sorcery: Fractal token with X counters
  where X = creatures you control),
  `quandrix_aether_adept` (carry-over: tap target creature).
- **5 Prismari (U/R) additions** (`stx::prismari`):
  `prismari_mindkindler` ({U}{R}, 1/2 with magecraft tap),
  `prismari_embergem` ({2}{R}, sorcery: 4 dmg to creature + Treasure),
  `prismari_pyromancer` ({2}{U}{R}, 3/2 with ETB 2 dmg + magecraft
  MayDo loot),
  `prismari_spitfire` ({3}{R}, 3/3 Haste with ETB 2 dmg to any target),
  `prismari_wildform` ({U}{R}, instant: +2/+1 + Haste + draw 1).

Total test count: 2485 → 2509. Total STX corpus per audit: 411+12=423
→ 434+13=447 (Vanquish the Horde reclassified from "✅ (cost-reduction
⏳)" to 🟡 to match its actual implementation status, then 25 new ✅).

Push (modern_decks, claude/modern_decks branch — prior revision —
**batch 23: 25 new synthesised STX cards (5 per college) + 26
functionality tests + CR 603.10a / 610.2 die-trigger scope fix**):

A 25-card follow-up sweep across all five colleges using existing
magecraft / drain / token / counter / lifegain primitives. Plus a
correctness fix to the die-trigger fast path in
`check_state_based_actions` (`game/stack.rs`): the dying creature's own
`CreatureDied` triggers were being pushed onto the stack regardless of
`EventScope`, so a `CreatureDied/AnotherOfYours` trigger on the dying
card itself would incorrectly fire for its own death. The fix filters
the die-trigger collection by scope (only `SelfSource`, `YourControl`,
`AnyPlayer`, `ActivePlayer` self-fire scopes are kept; `AnotherOfYours`
/ `OpponentControl` / `FromYourGraveyard` are correctly excluded — the
dying card can't be "another" creature you control). Inkling Aristocrat
now correctly doesn't gain life from its own death; all existing tests
continue to pass. **+5 follow-on cards** (one per college):
`inkling_sage` ({1}{W}, 1/2 Flying with `{2}{W}{B}` pump activation),
`witherbloom_reaper_hand` ({2}{B}{G}, 3/3 Deathtouch with die-trigger
drain 2), `spirit_conduit` ({2}, 0/2 Artifact-Spirit with `{R},{T}: 1
damage`), `quandrix_aether_adept` ({U}, 0/3 Defender with `{T}: tap
target creature`), `prismari_sparkbright` ({1}{R}, 2/1 Haste with
attack-trigger 1-damage ping). Total test count: 2454 → 2485.

- **5 Silverquill (W/B) additions** (`stx::silverquill`):
  `inkling_aristocrat` ({1}{B}, 1/2 — "Whenever another creature you
  control dies, you gain 1 life"), `silverquill_quillscribe`
  ({2}{W}{B}, 3/3 — ETB mint Inkling + magecraft "+1/+1 counter on
  target Inkling"), `silverquill_hush` ({W}{B}, instant: -2/-2 + gain
  2 life), `inkling_lorewright` ({3}{W}{B}, 2/4 Flying — ETB Draw 1,
  Lose 1), `silverquill_battle_hymn` ({2}{W}, sorcery — team +1/+1
  and gain Vigilance EOT).
- **5 Witherbloom (B/G) additions** (`stx::witherbloom`):
  `pest_ravager` ({3}{B}{G}, 4/4 Trample with ETB 2 Pests),
  `witherbloom_famine` ({3}{B}, sorcery: Drain 4 — 8 life swing),
  `witherbloom_greenrot` ({1}{G}, 2/2 Reach with ETB gain 2 life),
  `witherbloom_pestbroker` ({2}{B}, 2/3 with ETB drain 2 + `{1}{B},
  Sac a Pest: -1/-1` sink), `pestilent_bloom` ({B}{G}, instant: -3/-3
  + mint Pest).
- **5 Lorehold (R/W) additions** (`stx::lorehold`):
  `lorehold_phoenix` ({3}{R}, 3/3 Flying+Haste with `{R}{W}` gy
  recursion as a sorcery), `lorehold_battlechronicler` ({2}{R}{W}, 3/3
  with attack-trigger gy-creature recursion), `lorehold_searing_wisdom`
  ({3}{R}, sorcery: exile target gy card + 3 damage to any target),
  `lorehold_saint` ({1}{W}, 2/2 Lifelink + magecraft +1/+0 EOT),
  `lorehold_volley` ({2}{R}{W}, instant: 2 to any target + 1 to each
  other creature).
- **5 Quandrix (G/U) additions** (`stx::quandrix`): `quandrix_polymath`
  ({1}{G}{U}, 2/2 with ETB cantrip + target +1/+1 counter),
  `fractal_avenger` ({3}{G}{U}, 0/0 Trample with `enters_with_counters
  = +1/+1 ×4` — 4/4 base; scales with counter doublers),
  `quandrix_cartographer` ({2}{G}, 2/3 with ETB tutor basic land),
  `fractal_sovereign` ({3}{G}{U}, 3/4 with ETB +1/+1 counters equal to
  controller's land count), `quandrix_pairweaver` ({G}{U}, instant —
  +1/+1 counter on each of two friendly creatures via the
  multi-target-additional_targets slot).
- **5 Prismari (U/R) additions** (`stx::prismari`):
  `prismari_treasurer_surge` ({3}{U}{R}, 4/3 — ETB 2 Treasures +
  magecraft self-pump +1/+0 EOT), `prismari_pyreburst` ({3}{R},
  sorcery: 3 damage to each creature — Anger of the Gods at the slot),
  `prismari_vorthos` ({2}{U}{R}, 3/3 — ETB loot + if discarded IS,
  deal 2 to any target via `Value::CardsDiscardedThisEffect` +
  `Effect::If`), `prismari_cinderspark` ({R}, instant: 1 damage + Scry
  1), `prismari_tempo_adept` ({U}{R}, 2/2 Prowess with ETB optional
  loot).

Engine fix (push batch 23):
- **CR 603.10a / 610.2 (die-trigger scope filtering)** — wired in
  `game/stack.rs::check_state_based_actions`. The dying creature's
  `CreatureDied` triggered abilities are now collected with a scope
  filter so `AnotherOfYours` triggers don't fire on the dying card's
  own death. Lock-in tests:
  `inkling_aristocrat_gains_life_when_another_creature_dies` (positive
  control: bear dies → aristocrat gains life) and
  `inkling_aristocrat_does_not_trigger_on_self` (negative control:
  aristocrat itself dies → no life gain).

Push (modern_decks, claude/modern_decks branch — prior revision —
**batch 22: 25 new synthesised STX cards (5 per college) + 26
functionality tests + CR 701.46a stun-counter implementation**):

A 25-card follow-up sweep across all five colleges using existing
magecraft / drain / token / counter / lifegain primitives. Plus engine
work: stun counters now actually replace the untap event in `do_untap`
per CR 701.46a (previously stun counters could be added by Static
Prison but weren't consumed by the untap step, so the lockdown wasn't
enforced). Total test count: 2428 → 2454.

- **5 Silverquill (W/B) additions** (`stx::silverquill`):
  `silverquill_conviction` ({W}{B} sorcery: drain 2 + surveil 1),
  `silverquill_bookbearer` (3-mana 1/4 Vigilance ETB Scry 2),
  `inkling_inquisitor` (3-mana 2/3 Flying Inkling with ETB targeted
  hand-strip — Inquisition template on a body), `silverquill_reckoning`
  ({3}{W}{B} sorcery: destroy creature + mint Inkling token — 5-mana
  removal + body), `silverquill_lifeglyph` (3-mana 2/3 Lifelink Inkling
  Bard, magecraft → +1/+1 EOT to target creature).
- **5 Witherbloom (B/G) additions** (`stx::witherbloom`):
  `pest_swarmlord` (5-mana 3/3 Pest Warlock with ETB 2 Pest tokens),
  `witherbloom_vinetender` (2-mana 2/2 Reach Plant Druid with magecraft
  gain 1 life), `toxic_bloodletting` ({1}{B}{G} instant: -2/-2 EOT + gain
  2 life — Murderous Cut-style mini-removal), `witherbloom_saproot`
  (4-mana 3/3 Trample Plant Beast with dies-trigger drain 2),
  `pest_mausoleum` ({2}{B}{G} sorcery: return creature card from gy +
  mint Pest token — 4-mana 2-for-1).
- **5 Lorehold (R/W) additions** (`stx::lorehold`):
  `lorehold_emberscribe` (3-mana 3/2 Spirit Warrior with ETB
  exile-from-graveyard + 1 dmg to each opp), `lorehold_reliquary`
  ({2}{W} artifact: per-card-leaves-gy +1/+1 counter to target friendly),
  `lorehold_ringleader` (5-mana 4/3 Haste Spirit Warrior with ETB 2
  Spirit tokens), `lorehold_strikevanguard` (4-mana 4/2 First Strike
  Spirit Soldier, magecraft 1 dmg to any target), `lorehold_ember_recall`
  ({R}{W} sorcery: return ≤2-MV creature from gy + 1 dmg to each opp).
- **5 Quandrix (G/U) additions** (`stx::quandrix`):
  `quandrix_counterbalance` ({G}{U} instant: +1/+1 counter to friendly +
  draw 1), `fractal_bloom_caller` (4-mana 2/3 Fractal Wizard with ETB
  2/2 Fractal token), `quandrix_synthesist` (3-mana 2/2 Elf Druid
  magecraft team-wide +1/+1 counter), `fractal_tessellation` ({3}{G}{U}
  sorcery: Fractal token with X +1/+1 counters where X = lands you
  control), `quandrix_mistshaper` ({U} 1/1 Merfolk Wizard Flash with
  magecraft self-pump +1/+1 EOT).
- **5 Prismari (U/R) additions** (`stx::prismari`):
  `prismari_spellforger_b22` (4-mana 2/4 Wizard with magecraft team-
  pump + grant haste to source), `prismari_volleyfire` ({3}{R} sorcery:
  4 dmg to creature/PW + mint Treasure), `prismari_spell_shaper` ({U}{R}
  1/3 Wizard magecraft Scry 1 + Draw 1), `prismari_stormgaze` ({2}{U}{R}
  instant: looter 2-draw-1-discard + 1 dmg to any target),
  `prismari_vortexweaver` (5-mana 3/4 Flying Elemental Wizard with ETB
  copy-target-IS-on-stack).

Engine improvements (push batch 22):
- **CR 701.46a / 122.1d (stun counter consumption on untap)** — wired
  in `game/stack.rs::do_untap`. The untap step now consults each
  controlled permanent's `CounterType::Stun` count and removes one
  counter instead of untapping when stun counters are present. Static
  Prison's lockdown is now actually enforced: a 2-stun-counter
  permanent stays tapped for 2 untap steps before untapping normally.
  Test: `stun_counter_replaces_untap_per_cr_701_46a`.

CR audit: added new row **CR 701.26 — Tap and Untap** under
"MagicCompRules coverage audit" (see TODO.md). Documents the
tap/untap binary in `CardInstance.tapped`, the stun-counter
replacement on untap, and the idempotent-no-op semantics of
already-tapped/untapped permanents per the printed Oracle.

Push (modern_decks, claude/modern_decks branch — prior revision —
**batch 21: 25 new synthesised STX cards (5 per college) + 25
functionality tests**):

A 25-card follow-up sweep across all five colleges using the same shapes
established in batches 14–20 (magecraft / drain / token / counter /
lifegain primitives). No new engine features required. Total test count:
2399 → 2424.

- **5 Silverquill (W/B) additions** (`stx::silverquill`):
  `silverquill_inkscholar` (3-mana 2/3 Cleric ETB loot),
  `inkling_battlecaster` (5-mana 3/3 Flying + Vigilance Inkling Knight
  with attack-trigger drain 1), `silverquill_compulsion` ({1}{B} sorcery:
  target opp discards a chosen nonland — Thoughtseize template),
  `silverquill_sealwriter` (3-mana 2/2 Lifelink Wizard ETB drain 2),
  `inkling_acolyte` (2-mana 1/2 Flying Inkling Cleric + ETB Inkling token
  mint — double-Inkling for 2 mana).
- **5 Witherbloom (B/G) additions** (`stx::witherbloom`):
  `pest_forager` (2-mana 2/1 Trample Pest with die-to-1-life trigger),
  `witherbloom_carnivine` (5-mana 4/4 Reach Plant Beast with ETB drain 3),
  `pest_harvest` ({2}{B}{G} sorcery: Pest token + draw 1),
  `witherbloom_necrosophist` (3-mana 2/3 Warlock with ETB return-creature-
  from-graveyard-to-hand), `witherbloom_pestcaller` (4-mana 2/4 Plant
  Druid magecraft Pest token engine).
- **5 Lorehold (R/W) additions** (`stx::lorehold`):
  `lorehold_sparkstrike` ({1}{R} instant: 2 damage to any target +
  Surveil 1), `lorehold_bonereader` (3-mana 2/3 Spirit Cleric Vigilance
  with ETB gain 2 + magecraft self-pump +1/+0 EOT), `lorehold_spiritarcher`
  (4-mana 2/3 Spirit Archer Reach with ETB 2 damage to any target),
  `lorehold_echoflame` ({3}{R}{W} sorcery: return target IS card from
  graveyard + mint Spirit token), `lorehold_pilgrimwarden` (4-mana 3/3
  First Strike Spirit Soldier that mints a 1/1 W Soldier token per attack).
- **5 Quandrix (G/U) additions** (`stx::quandrix`):
  `quandrix_calibrator` (3-mana 2/3 Elf Druid ETB +1/+1 counter on
  friendly creature), `fractal_resonance` ({1}{G}{U} instant: +1/+1
  counter on each friendly creature), `quandrix_mistweaver` (2-mana
  1/2 Flash + Flying Merfolk Wizard ETB Draw 1), `fractal_harvest`
  ({3}{G}{U} sorcery: 3/3 Fractal token via 3 +1/+1 counters + cantrip),
  `quandrix_sage` (3-mana 2/2 Wizard with magecraft Seq(Scry 1 + Draw 1)).
- **5 Prismari (U/R) additions** (`stx::prismari`):
  `prismari_sparkforge` (4-mana 3/3 Elemental Haste with ETB Treasure
  token), `prismari_mindwave` ({2}{U} instant: Draw 2 + Discard 1 looter),
  `prismari_pyrocrafter` (3-mana 2/2 Human Wizard ETB pings each opp for
  1 + magecraft self-pump +1/+0 EOT), `prismari_stormspire` (6-mana 4/4
  Flying Djinn Wizard ETB Draw 2), `prismari_quickfire` ({R} instant:
  2 damage to target creature — Burst Lightning at the curve-1 slot).
- **4 cross-school / iconic additions** (`stx::iconic`):
  `hunt_the_library` ({3}{G} sorcery: Rampant Growth template),
  `field_researcher` (3-mana 2/3 Vigilance Druid with ETB ramp),
  `spellbook_studier` (2-mana 1/3 Wizard with ETB Scry 2),
  `strixhaven_vigil` ({2}{W}{W} Enchantment: per-upkeep +1 life).

Engine improvements (push batch 21):
- New `shortcut::create_token_with_keyword(who, count, token, kw, dur)`
  helper that consolidates `Seq([CreateToken, GrantKeyword(LastCreatedToken,
  …)])` shapes — refactored `lorehold_skirmish`.
- New `shortcut::create_token_with_counter(who, count, token, counter, n)`
  helper that consolidates `Seq([CreateToken, AddCounter(LastCreatedToken,
  …)])` shapes — refactored `quandrix_summoner` + powers new
  `fractal_harvest`.
- New `shortcut::magecraft_target_pump(what, p, t)` helper for
  "magecraft → pump target" patterns (sibling to `magecraft_self_pump`).

CR audit: added new row **CR 701.16 — Investigate** under
"MagicCompRules coverage audit" (see TODO.md). Wraps the existing
clue-token pipeline as the keyword-action's CR-correct implementation —
no new primitive needed.

Push (modern_decks, claude/modern_decks branch — prior revision —
**batch 20: 25 new synthesised STX cards (5 per college) + 26
functionality tests**):

A 25-card sweep across all five colleges, building on the magecraft /
drain / token / counter / lifegain primitives established in batches
14–19. Every card uses existing engine primitives — no new engine
features required. Total test count: 2364 → 2390.

- **5 Silverquill (W/B) additions** (`stx::silverquill`):
  `silverquill_lawkeeper` (2-mana 2/2 Vigilance + ETB tap opp creature),
  `inkling_penmaster` (4-mana 2/3 Flying Inkling Wizard, magecraft mints
  Inkling token), `silverquill_dictation` ({1}{W}{B} instant: target
  opp loses 2 + draw 1), `inkling_stormcaller` (5-mana 3/4 Flying +
  Lifelink with ETB drain 2), `silverquill_discipline` ({W} instant
  +2/+1 + lifelink EOT).
- **5 Witherbloom (B/G) additions** (`stx::witherbloom`):
  `witherbloom_toxicultivator` (3-mana 2/3 Deathtouch with ETB Pest
  token), `pest_outburst` ({2}{B}{G} sorcery: 2 Pest tokens + gain 2
  life), `witherbloom_grand_necromancer` (5-mana 3/3 with ETB
  reanimate-to-hand + magecraft drain), `witherbloom_sapdrinker`
  ({1}{B}{G} 2/3 Lifelink, magecraft +1/+0 self-pump EOT),
  `witherbloom_crawler` ({B}{G} 2/2 Deathtouch + Reach vanilla).
- **5 Lorehold (R/W) additions** (`stx::lorehold`):
  `lorehold_battlescroll` ({3}{R}{W} sorcery: 2 Spirit tokens with
  haste EOT), `lorehold_tomescholar` (4-mana 2/3 ETB graveyard-exile
  → conditional 2/2 Spirit token), `lorehold_ember_brand` ({1}{R}
  instant: 3 damage to any target), `lorehold_spectrescribe` ({1}{W}
  1/3 Spirit Cleric magecraft gain 1 life), `lorehold_warband`
  ({2}{R} 3/2 Spirit Soldier Haste, on-attack +X/+0 where X = other
  attackers).
- **5 Quandrix (G/U) additions** (`stx::quandrix`): `fractal_bloom`
  ({3}{G}{U} sorcery: Fractal token + 2×HandSize +1/+1 counters),
  `quandrix_spellweaver` ({2}{G}{U} 2/4 Wizard ETB draw 2 + magecraft
  +1/+1 counter), `quandrix_wavedancer` ({1}{U} 1/3 Flash with ETB
  Scry 2), `fractal_synthesis` ({2}{G}{U} instant: +2 counters +
  draw 1), `quandrix_hatchling` ({G}{U} 0/0 Fractal enters with 2
  +1/+1 counters + magecraft self-growth).
- **5 Prismari (U/R) additions** (`stx::prismari`):
  `prismari_cascade_volley` ({2}{R} sorcery: 3 dmg + 1 dmg each
  opp creature), `prismari_initiate` ({1}{R} 2/2 magecraft 1 damage
  to any target), `prismari_spellbinder` (5-mana 3/4 Flying Djinn
  Wizard ETB copy-spell), `prismari_treasurer` ({1}{U} 1/2 ETB
  Treasure), `prismari_embershaper` ({U}{R} 2/1 Wizard magecraft
  MayDo loot).
- **5 cross-school additions** (`stx::extras`): `strixhaven_scholar`
  ({1}{U} 2/1 Human Wizard magecraft Scry 1), `strixhaven_quill_mage`
  ({2}{R} 2/2 Human Wizard magecraft 1 dmg target opponent),
  `strixhaven_initiate` ({G} 1/2 Druid Reach with {T}: Add {G}),
  `strixhaven_burnscholar` ({R} 1/1 Wizard Haste with ETB 1 dmg
  target opponent), `strixhaven_necropact` ({2}{B} sorcery: target
  player draws 2 + loses 2 life).
- **4 iconic / cross-school finishers** (`stx::iconic`):
  `heroic_defiance` ({1}{W} instant: +1/+1 + hexproof + indestructible
  EOT), `tome_shredder` ({2}{B} 2/2 Warlock with ETB-targeted-discard),
  `mascot_acolyte` ({2}{G} 2/3 Druid Reach with ETB basic-land tutor),
  `lorehold_strikeforce` ({2}{R}{W} sorcery: team-wide +2/+0 +
  trample EOT).

Plus a small engine cleanup pass: `permanent_has_keyword` and
the `bot_deadlock_dumps_full_state` test helper migrated from
`map().unwrap_or(false)` to `is_some_and()` for clarity.

CR audit: added **CR 110 (Permanents)** row under "MagicCompRules
coverage audit" — owner/controller, characteristics, types, status
all audited; 110.4f Battle subtype + 110.5 Phasing/Flip flagged as
engine-wide ⏳.

Push (modern_decks, claude/modern_decks branch — prior revision —
**batch 19+: 10 more synthesised STX cards (2 per college) + 10
functionality tests**):

Follow-up batch on top of batch 19 — 2 more cards per college (10
total) extending the same magecraft / drain / tribal / Treasure /
counter primitives. No new engine primitives required.

- **2 Silverquill (W/B) additions** (`stx::silverquill`):
  `silverquill_quillblade` ({W} instant +X/+0 EOT scaling with your
  creature count), `inkling_decree` ({3}{W}{B} drain 2 + Inkling
  token sorcery).
- **2 Witherbloom (B/G) additions** (`stx::witherbloom`):
  `witherbloom_glimmer` (4-mana 3/3 Lifelink Plant Druid),
  `pest_communion` ({1}{B}{G} sorcery: each opp mills 4 + drain 1).
- **2 Lorehold (R/W) additions** (`stx::lorehold`):
  `lorehold_recollect` ({1}{R}{W} reanimate creature OR artifact),
  `lorehold_anthemist` (4-mana 2/2 Spirit Cleric Spirit-tribal
  anthem +1/+1 to Other Spirits).
- **2 Quandrix (G/U) additions** (`stx::quandrix`):
  `fractal_growth` ({G}{U} sorcery: +1/+1 counter + EOT pump by
  total counters), `quandrix_calculus` (4-mana 2/2 Fractal Wizard
  ETB self-mill 2 + draw 1).
- **2 Prismari (U/R) additions** (`stx::prismari`):
  `prismari_alchemist` (4-mana 2/3 Wizard magecraft Treasure mint),
  `prismari_cantrip` ({U}{R} instant: 1 damage to creature + draw 1).

Total test count: 2354 → 2364.

Push (modern_decks, claude/modern_decks branch — prior revision —
**batch 19: 20 new synthesised STX cards across all 5 colleges + 20
functionality tests + CR 122 audit refresh**):

Adds 20 new synthesised STX cards spread across all five colleges
(exactly 4 per college). Every card uses existing primitives
(Magecraft / ETB triggers / drain templates / counter accumulators
/ Pest/Spirit/Inkling token minters / scry+draw cantrips). No new
engine primitives required.

- **4 Silverquill (W/B) additions** (`stx::silverquill`):
  `silverquill_castigant` (2/3 ETB drain 1), `silverquill_heartrender`
  (3-mana drain 3 + scry 1 sorcery), `inkling_confessor` (3-mana
  Flying Inkling Cleric with magecraft drain 1), `inkling_inkrider`
  (4-mana 3/2 Flying + Vigilance Inkling Knight).
- **4 Witherbloom (B/G) additions** (`stx::witherbloom`):
  `witherbloom_lifebleeder` (3-mana magecraft drain 1 body),
  `pest_marauder` (2-mana 1/1 Pest Deathtouch + gain-1-on-die),
  `witherbloom_decoctor` (5-mana 3/4 ETB drain 2),
  `witherbloom_sapfiend` (3-mana 2/3 Plant Beast magecraft +1/+1
  self-pump EOT).
- **4 Lorehold (R/W) additions** (`stx::lorehold`):
  `lorehold_pyrescribe` (4-mana 3/2 magecraft 1-each-opp ping),
  `lorehold_echoist` (2-mana 1/2 ETB Spirit token),
  `lorehold_spiritmaster` (5-mana 3/3 ETB 2 Spirit tokens),
  `lorehold_bonepriest` (3-mana 2/2 magecraft permanent +1/+1
  counter).
- **4 Quandrix (G/U) additions** (`stx::quandrix`):
  `quandrix_doublecaster` (5-mana 3/3 Fractal Wizard magecraft
  permanent +1/+1 counter), `quandrix_wavewright` (4-mana 2/3 ETB
  scry 2 + draw 1), `quandrix_sapsprout` (2-mana 1/2 Fractal
  magecraft permanent +1/+1 counter), `fractal_multiplier` (4-mana
  sorcery doubles +1/+1 counters on target creature you control).
- **4 Prismari (U/R) additions** (`stx::prismari`):
  `prismari_stormcaster` (5-mana 3/3 Djinn Wizard Flying magecraft
  loot), `prismari_sparkmaster` (4-mana 2/2 Wizard magecraft +1/+0
  self-pump EOT), `prismari_ember_channeler` (2-mana 1/2 magecraft
  1 damage any target), `prismari_flarespark` (3-mana instant 2
  damage + cantrip).

All 20 cards ship with functionality tests in `tests::stx` (20 new
tests covering the headline play pattern of each). Total test count:
2334 → 2354.

Push (modern_decks, claude/modern_decks branch — prior revision —
**batch 18: 22 new synthesised STX cards across all 5 colleges + 24
functionality tests + CR 120 (Damage) audit rows**):

Adds 22 new synthesised STX cards spread across all five colleges plus
new audit rows for CR 120.4 (four-part damage-dealing sequence) and CR
120.6 (marked damage / lethal damage). Every card uses existing
primitives (Magecraft / ETB triggers / Pest/Inkling/Fractal token
minters / sac-cost / counter accumulators / drain templates).

- **3 Silverquill (W/B) additions** (`stx::silverquill`):
  `inkling_coursebinder` (3-mana Flying Inkling Wizard with magecraft
  drain), `silverquill_sermon` (4-mana 2-Inkling sorcery), and
  `silverquill_censure` (2-mana exile-power-3-or-less + 2 life).
- **6 Witherbloom (B/G) additions** (`stx::witherbloom`):
  `witherbloom_pestkeeper` (sac-a-Pest activation for -2/-2 EOT +
  ETB Pest), `witherbloom_bonepicker` (3/3 trample ETB drain 2),
  `pest_swarm_inheritance` ("Pest Bequest" — pump+deathtouch+Pest
  mint sorcery), `witherbloom_decayblossom` (1/1 die-shrinks-target),
  `witherbloom_recourse` (return ≤2-MV creature from gy + drain 1),
  `witherbloom_pestmancer` (magecraft mints a Pest each cast).
- **4 Quandrix (G/U) additions** (`stx::quandrix`):
  `quandrix_fractalflow` (ETB mints Fractal with HandSize counters),
  `quandrix_scrycharmer` (magecraft Scry 1),
  `quandrix_crystallizer` (Hexproof 2/3 with sorcery-speed counter
  activation), `quandrix_multibinding` (+2 counters then double),
  `quandrix_geomyst` (5-mana 4/4 Reach with ETB Draw 1).
- **4 Lorehold (R/W) additions** (`stx::lorehold`):
  `lorehold_spiritcaller` (ETB Spirit + per-gy-leave gain-1),
  `lorehold_pyrebrand` (3-mana First Strike Spirit with magecraft
  self-pump), `lorehold_reclamation` (4-mana single-target reanimate),
  `lorehold_reverberator` (4-mana Haste body with magecraft 2-damage).
- **4 Prismari (U/R) additions** (`stx::prismari`):
  `prismari_spellsmith` (3-mana 2/2 + Treasure ETB),
  `prismari_storm_caller` (magecraft Loot 1), `prismari_ignite_apprentice`
  (2-mana ETB any-target 1 damage), `prismari_volley` (3-mana 3-damage
  creature/planeswalker burn + cantrip).

All 22 cards ship with functionality tests in `tests::stx` (24 new
tests covering both the headline play pattern and the body identity
where the rider is the differentiator). Total test count: 2310 → 2334.

CR audit rows added: CR 120.4 (four-part damage-dealing sequence) ✅,
CR 120.6 (marked damage / lethal damage / cleanup) ✅. Both audited
against `MagicCompRules_20260417.txt` lines 1107-1124.

Push (modern_decks, claude/modern_decks branch — prior revision —
**batch 17: 21 new synthesised STX cards across all 5 colleges + 23
functionality tests + 1 engine improvement (CR 115.5 self-target
enforcement)**):

Adds 21 new synthesised STX cards spread across all five colleges plus
a new engine-level rule enforcement. Every card uses existing
primitives (Magecraft / ETB triggers / drain templates / counter
accumulators / Pest/Inkling/Fractal token minters / scry+draw cantrips).
No new card-side primitives needed.

- **5 Silverquill (W/B) additions** (`stx::silverquill`):
  `silverquill_marshal` (2/3 ETB gain 2 life),
  `inkling_sanctifier` (2/3 Flying+Lifelink Inkling),
  `silverquill_pupil` (1/2 magecraft +1/+0 self),
  `defend_the_inkwell` (drain 2 + scry 2 sorcery),
  `inkling_witness` (2/2 Flying with on-other-Inkling-death lifegain).
- **4 Witherbloom (B/G) additions** (`stx::witherbloom`):
  `witherbloom_mossfeeder` (3/3 ETB Pest token),
  `witherbloom_reverie` (drain 3 sorcery at {1}{B}{G}),
  `pest_cultivator` (2/2 ETB 2 Pests),
  `withergrowth_apprentice` (1/3 magecraft +1/+1 on friendly).
- **4 Lorehold (R/W) additions** (`stx::lorehold`):
  `lorehold_pyrosage` (2/2 magecraft 1 each opp),
  `lorehold_loremaster` (4/4 First Strike + attack-mints-Spirit),
  `lorehold_aerospirit` (3/2 Flying+Haste Spirit Soldier),
  `lorehold_ember_forge` (3 dmg creature + 1 each opp).
- **4 Quandrix (G/U) additions** (`stx::quandrix`):
  `quandrix_symmetrist` (3/3 ETB scry 2 + draw),
  `quandrix_reckoner` (2/2 Trample with +1/+1 counter per attack),
  `fractal_reinforcement` (+1/+1 counter on each creature you control),
  `quandrix_tutelary` (1/3 magecraft +1/+1 on a Fractal you control).
- **4 Prismari (U/R) additions** (`stx::prismari`):
  `prismari_pyrotechnician` (2/2 magecraft 1 dmg any target),
  `prismari_looter` (1/3 ETB draw 1 + discard 1),
  `prismari_chromaticist` (3/3 ETB Treasure token),
  `prismari_drakeward` (4/4 Flying Drake ETB 2 dmg each opp).

**Engine improvement: CR 115.5 (self-target enforcement)** — the cast
pipeline now threads the casting spell's `CardId` into
`check_target_legality_with_source(target, caster, source)`. When a
target's permanent id matches the source, the cast is rejected with
`GameError::InvalidTarget`. Catches the printed-rule corner "A spell
or ability on the stack is an illegal target for itself" — verified
via `cr_115_5_spell_targeting_itself_is_illegal_via_permanent_id` (Bury
in Books targeting itself rejected).

All 21 ship with at least one functionality test in `tests::stx`.
Total: 23 new tests, all passing.

Push (modern_decks, claude/modern_decks branch — prior revision —
**batch 16: 12 SOS 🟡 → ✅ promotions + 4 engine improvements + ~25 new
functionality tests**):

Promoted via a mix of new engine primitives and tighter existing-primitive
use. The engine additions are: (a) `SelectionRequirement::HasGreatestManaValueAmongControlled(inner)`
for "the greatest MV among permanents matching `inner` they control"
(End of the Hunt). (b) `AlternativeCost.condition: Option<Predicate>`
for "X less if [game-state predicate]" alt costs (Wilt in the Heat,
Orysa Tide Choreographer). (c) `Value::PowerOf` / `Value::ToughnessOf`
**now sum across fan-out selectors** (`EachPermanent(...)`) — single-
entity reads (`Target(0)`, `This`) unchanged; fan-out reads sum (Orysa's
"total toughness ≥ 10" gate, future total-power payoffs). (d)
`fire_spell_cast_triggers` **now threads `converged_value`** onto the
resulting `StackItem::Trigger` — previously hard-coded to 0, blocking
per-cast converge introspection on Magmablood Archaic, Wildgrowth Archaic.

Promoted SOS cards (12): **End of the Hunt** (greatest-MV picker),
**Ambitious Augmenter** (death-with-counters → Fractal token + counters),
**Topiary Lecturer** (Increment doc-sync), **Choreographed Sparks**
(mode 1 creature spell copy), **Magmablood Archaic** (per-cast pump
reads ConvergedValue), **Wilt in the Heat** (alt-cost when cards left
gy), **Rubble Rouser** (exile-gy activation), **Orysa, Tide
Choreographer** (alt-cost when total toughness ≥ 10), **Mana Sculpt**
(reads countered spell's MV), **Sundering Archaic** (converge-scaled
MV cap on Exile target), **Lorehold, the Historian** (opp-upkeep loot),
**Conspiracy Theorist** (empty-hand activation gate, STX).

Push (modern_decks, claude/modern_decks branch — previous revision —
**22 new synthesised STX cards + 26 new functionality tests = batch 15
— 22 cards total**):

Adds 22 new synthesised STX cards spread across all five colleges
plus a new CR 113 (Abilities) audit row in `TODO.md`. Every card uses
existing engine primitives (`magecraft`, ETB triggers, drain templates,
counter accumulators, Pest/Inkling/Fractal token minters). No new
engine primitive needed.

- **7 Silverquill (W/B) additions** (`stx::silverquill`):
  `silverquill_archivist` (1/2 ETB scry 1 + gain 1 life),
  `silverquill_witness` (2/1 lifelink + magecraft gain 1 life),
  `silverquill_judge` (2/3 vigilance + magecraft tap opp creature),
  `inkling_brigade` (3/3 flying + ETB mints 2 Inkling tokens),
  `silverquill_pen_pusher` (1/1 flying Inkling + magecraft scry 1),
  `silverquill_chronicle` (drain 2 + return IS card from gy),
  `inkling_vanguard` (2/3 flying + vigilance Inkling).
- **5 Witherbloom (B/G) additions** (`stx::witherbloom`):
  `witherbloom_pest_tender` (1/2 ETB Pest token),
  `pest_swarmer` (2/2 Pest dies → Pest token),
  `witherbloom_seer` (2/2 deathtouch + magecraft drain 1),
  `pest_swarm` (Sorcery: create three Pest tokens),
  `witherbloom_vinemaster` (3/4 Trample + +1/+1 on other Pest deaths).
- **4 Lorehold (R/W) additions** (`stx::lorehold`):
  `lorehold_acolyte` (1/3 ETB exile target gy card),
  `lorehold_warrior_priest` (2/2 attack→life + gy-leave +1/+1
  counter), `lorehold_ember_priest` (2/3 magecraft ping 1),
  `lorehold_skirmish` (mint 2/2 Spirit with haste EOT).
- **3 Quandrix (G/U) additions** (`stx::quandrix`):
  `quandrix_summoner` (2/2 ETB 1/1 Fractal),
  `quandrix_scholar` (1/2 magecraft +1/+1 counter on friendly),
  `quandrix_ecologist` (4/4 trample ETB scry 2 + self-counter).
- **3 Prismari (U/R) additions** (`stx::prismari`):
  `prismari_drakelord` (2/3 flying Drake + magecraft +1/+1 EOT),
  `prismari_emberseer` (3/3 flying + ETB 2 dmg each opp),
  `prismari_pyrowriter` (2/2 magecraft ping 1).

All 22 ship with at least one functionality test in `tests::stx`. The
Pest-tribal cycle (Pest-Tender / Pest Swarmer / Pest Swarm /
Vinemaster) plus the new Lorehold Spirit minter (Lorehold Skirmish)
exercise the existing token-with-death-trigger plumbing
(`TokenDefinition.triggered_abilities` for Pest "die → gain 1 life"
trigger; `lorehold_spirit_token()` for the 2/2 R/W Spirit minter).

Push (modern_decks, claude/modern_decks branch — prior revision —
**25 new synthesised STX cards + 25 new functionality tests = batch 14
— 25 cards total**):

Adds 25 new synthesised STX cards in two locations:

- **15 Silverquill (W/B) expansion** (`stx::silverquill`): expands the
  Silverquill college pool from 15 → 30 cards, adding deeper Inkling
  tribal density (`inkling_aspirant`, `inkling_scribe`, `inkling_bloodscribe`,
  `inkling_verselord`, `silverquill_penman`, `silverquill_anthemwriter`),
  drain payoffs (`silverquill_drainmaster`, `witherspell_drain`,
  `silverquill_quillmage`, `inkrise_lifedrainer`), and utility bodies
  (`silverquill_loremender`, `silverquill_memorialist`, `silverquill_erudite`,
  `silverquill_reprimand`, `silverquill_inquisition`).
- **10 cross-college additions** (`stx::extras`): expands Lorehold
  (`lorehold_phantasmist`, `lorehold_bookburner`), Prismari
  (`prismari_lightcaster`, `prismari_stormbringer`), Quandrix
  (`quandrix_counterspeaker`, `quandrix_tessellator`), Witherbloom
  (`witherbloom_wanderer`, `witherbloom_pestbinder`), and colorless
  (`strixhaven_vault`, `strixhaven_acolyte`). Each plugs into an
  existing engine primitive (`magecraft`, `StaticEffect::GrantKeyword`,
  `EventScope::AnotherOfYours`, `Effect::MayDo`).

All 25 ship with at least one functionality test in `tests::stx`. The
`inkling_verselord` and `lorehold_phantasmist` tests in particular
exercise the printed "Other [type]s have [keyword]" anthem via the
existing `OtherThanSource` selector — same shape as Hofri Ghostforge,
Tenured Inkcaster, Quintorius. New CR audit row: **CR 116 — Special
Actions** (the 12-special-actions framework; only 116.2a "play a
land" is exercised today, mapped to `GameAction::PlayLand` walking
hand → battlefield without going through the stack per 116.1).

Push (modern_decks, claude/modern_decks branch — prior revision —
**21 MORE STX cards + 25 new functionality tests = batch 12 — 203 cards
total across batches 9-12**):

Adds 21 more synthesised STX cards across all five colleges plus
colorless/shared slots. All wired with existing primitives (Magecraft
helpers, drain templates, counter accumulators, gy-recursion bodies,
pump-and-fight combat tricks). **Silverquill Verseweaver** (3/3 Inkling
Cleric Wizard Flying ETB drain 2), **Inkling Choirmaster** (1/3 Cleric
+1/+1-on-lifegain self-grow + +1/+0 Inkling anthem), **Bramble Brewer**
(2/3 Plant Druid ETB Pest + activated sac-creature draw + life),
**Witherbloom Decanter** ({BG} Instant -2/-2 + gain 2 life), **Pestbrood
Grovecaller** (3/4 Plant Beast ETB Pest + on-Pest-death gain-1-draw-1),
**Lorehold Cathedral** (R/W dual land with sac-for-gy-reanimation),
**Lorehold Bannerbearer** (3/3 First Strike Spirit tribal anthem +1/+1),
**Lorehold Pyromage** (3/4 ETB 3 damage + magecraft 1 damage),
**Quandrix Geomancer** (2/3 ETB mints X 1/1 Fractals per land),
**Quandrix Fractalist** (3/3 Trample with ETB hand-size +1/+1 counters),
**Quandrix Skybinder** (2/3 Flying attack-trigger counter on friendly),
**Prismari Mistcaller** (2/2 Prowess ETB Scry 2 + Draw 1), **Prismari
Conflagration** ({3UR} Instant 4 damage / counter-unless-3 modal),
**Prismari Treasurewright** (2/3 Artificer ETB two Treasures + magecraft
Scry 1), **Silverquill Auctioneer** (3/2 Flying Lifelink + magecraft
+1/+1 self-grow), **Witherbloom Reanimist** (3/2 ETB returns ≤2-MV
creature + activated reanimate-for-2-life), **Lorehold Skirmisher**
(2/2 Haste + MayPay {R} pump on attack), **Quandrix Landmapper**
({2GU} Sorcery: Cultivate-to-bf + Scry 2), **Prismari Spellsong**
({UR} Instant: loot + 2 damage if noncreature discarded), **Silverquill
Reaper** (4/3 Flying ETB destroy toughness ≤ 2), **Strixhaven Reservoir**
(3-mana 5-color rock + {3}{T}: Draw a card), **Spelltongue Statute**
({2W} Enchantment: gain 1 life per IS cast).

Push (modern_decks, claude/modern_decks branch — prior revision —
**23 MORE STX cards + 29 new functionality tests + new
`StaticEffect::DoubleCounters` engine primitive (CR 614.16 counter half) =
batch 11 — 182 cards total across batches 9-11**):

Adds 22 more synthesised STX cards across all five colleges + 1 engine
primitive: **Witherbloom Pestseed** (3/3 Plant Druid with the new
`StaticEffect::DoubleCounters` — CR 614.16 counter-replacement half,
Hardened-Scales-template), **Silverquill Editorialist** (2/2 Inkling
Wizard Flying + Magecraft drain 1), **Inkblot Recluse** (2/4 Spider
Inkling Reach + ETB Surveil 2), **Quill-Lecturer** (2/4 Vigilance +
Magecraft -1/-1 on opp creature), **Inkstrike Bolt** (3 damage to opp
creature + 2 life), **Withering Spores** (mass -1/-1 EOT), **Witherbloom
Brewer** (2/3 mana ability: tap + 2 life → {B}{G}), **Pestilent
Brambletwig** (2/1 Plant Pest with die-trigger lifegain 2),
**Witherbloom Soothsayer** (2/3 ETB Surveil 2 + drain 1), **Lorehold
Vanquisher** (3/3 First Strike + attack-trigger lifegain), **Lorehold
Burnscholar** (2/2 Magecraft ping + lifegain), **Pillardrop Cultivator**
(2/3 Spirit Bird Flying + ETB reanimate MV≤2), **Prismari Skywatcher**
(1/2 Merfolk Wizard Flying + Magecraft self-pump), **Brewmaster
Pyrologist** (4/3 Trample + ETB 2 damage + draw), **Prismari Spell
Smith** (2/2 Magecraft AnyOneColor), **Quandrix Botanist** (2/2 Elf
Druid + Magecraft +1/+1 on target Fractal), **Quandrix Augur** (2/3
Fractal Wizard + ETB Scry 2 → Draw 1), **Fractal Trefoil** (0/0
Fractal Trample with X +1/+1 counters per land), **Quandrix
Equationist** (3/3 Flying + draw on +1/+1 counter add), **Pyrokinetic
Insight** (Sorcery — 3 dmg / rummage modal), **Lorehold Spirit
Tutor** (Spirit-tribal tutor via RevealUntilFind), **Strixhaven
Sanctum** (colorless Land — {T}: Add {C} + {2},{T}: Surveil 1),
**Mystic Slate** (2-mana Artifact — scry {T} + sorcery-speed draw {2}{T}),
**Strixhaven Bloomstadium** (5-mana Enchantment with both DoubleTokens +
DoubleCounters, Doubling-Season template — first card to ship both
halves of CR 614.16).

**Engine primitive: `StaticEffect::DoubleCounters`** — CR 614.16 counter
half of the token/counter replacement family. Read at `Effect::AddCounter`
resolution time via `GameState::counter_doublers_for(seat)` (mirrors
`token_doublers_for`); the counter count is multiplied by `2^doublers`
per affected permanent. Also wired into the `enters_with_counters`
(CR 614.12) replacement at both call sites (`stack.rs` spell-resolution
path + `effects/movement.rs::place_card_in_dest`), so Fractal Trefoil
under a Pestseed lands at 2× the lands-controlled count. Composes
multiplicatively with itself (2 Pestseeds → 4×) and with `DoubleTokens`
(Doubling Season ships both). Witherbloom Pestseed is the canonical
exerciser. Tests: `witherbloom_pestseed_doubles_plus_one_counter_placement`,
`_does_not_double_opp_counters`, `_stacks_multiplicatively`,
`fractal_trefoil_with_pestseed_doubles_counters`.

Push (modern_decks, claude/modern_decks branch — prior revision —
**20 MORE STX cards + 24 new functionality tests + CR 603.4 intervening-
'if' engine fix for AnotherOfYours ETB triggers + CR 122 (Counters)
audit = batch 10 — 159 cards total in batches 9+10**):

Adds 20 more synthesised STX cards across all five colleges, exercising
existing engine primitives: **Silverquill Chastiser** (3/2 Inkling Cleric
Flying, drains 1 on other Inkling ETB — first card validating the new
CR 603.4 intervening-'if' fix), **Witherbloom Pestmaster** (2/3 Plant
Warlock, ETB-mints-Pest + counter-on-other-Pest-death), **Lorehold
Chronicler** (3/3 Spirit Cleric, ETB gy-IS-recursion + on-attack
gy-strip), **Prismari Pyromentor** (3/4 magecraft 2-burn each opp),
**Quandrix Equation** (Fractal mint with 2× hand-size counters),
**Silverquill Inquisitor's Mark** (targeted Despise + gain 2 life),
**Witherbloom Mire** (3-mana drain 3 + Surveil 2), **Lorehold
Memorial** (gy-creature recursion + per-turn Spirit mint), **Prismari
Ember-Trickster** (Prowess + ETB Treasure), **Quandrix Aetherist**
(hand-size ETB counters + on-counter draw), **Silverquill Sentinel**
(2/2 Inkling Flying Lifelink with combat-step self-pump), **Witherbloom
Necrogale** (4/4 Plant Zombie, ETB reanimates ≤3-MV with haste),
**Lorehold Echo** (combat trick with gy-conditional FS+Lifelink),
**Prismari Spellforger** (ETB loot + Magecraft Treasure), **Quandrix
Multiplier** (3/3 ETB doubles +1/+1 counters on target), **Silverquill
Scribefall** (two Inklings + drain 2 sorcery), **Witherbloom
Wickering** (sac-creature -2/-2 or -4/-4 by toughness), **Lorehold
Historian** (graveyard-exile-as-cost ETB ping), **Prismari Spectacle**
(3-mode bolt / loot / Treasure), **Quandrix Wavebreaker** (ETB
scry-draw + on-draw counter), **Silverquill Anthemwright** (Flying
+1/+0 + lifelink anthem on Other creatures), **Witherbloom Decay**
(2 mana removal + 2 life), **Lorehold Reverberation** (3 damage +
3 life if creature died), **Prismari Eccentric** (Haste + ETB Treasure
+ sac-Treasure-to-draw), **Quandrix Theorem Crafter** (+1/+1 counters
per land controlled).

**Engine fix: CR 603.4 intervening-'if' clause for AnotherOfYours ETB
triggers**. The synchronous AnotherOfYours-ETB-trigger push in
`game/stack.rs::344+` was a duplicate of the unified-dispatcher path
(`game/mod.rs::dispatch_triggers_for_events`) — it bypassed the
`EventSpec.filter` predicate check entirely and left `trigger_source`
unset, causing cards with filtered "another X ETBs" triggers
(Silverquill Chastiser's Inkling filter, Felisa-style WithCounter
filters) to double-fire with their filter ignored. Removed in this
push so the dispatcher is the sole source of truth — `trigger_source`
is correctly bound to the ETB subject and the CR 603.4 intervening-'if'
runs the filter against the ETB subject card. Tests:
`silverquill_chastiser_drains_on_other_inkling_etb`,
`silverquill_chastiser_does_not_trigger_on_non_inkling_etb`.

**Engine audit: CR 122 — Counters**. Audit row added below for CR 122
covering counter placement (122.1), zero-counter no-op (122.3a/b), and
ETB-counter replacement (122.6). Most rules already wired; the
remaining gap is **122.4** (counters on cards in non-battlefield zones
having no game effect for most counter types — already implicitly
honored since most counter-reading predicates only check battlefield
permanents).

Push (modern_decks, claude/modern_decks branch — prior revision —
**10 MORE STX cards + 11 new functionality tests = batch 9 — 139 cards
total**):

Adds 10 more STX cards across all five colleges, exercising existing
engine primitives: Quandrix Forecaster (look-3 dig-and-cantrip),
Silverquill Bookbinder (ETB +3/-3 drain flier), Lorehold Crusader Knight
(First Strike + Lifelink + Magecraft self-pump), Witherbloom Conjurer
(ETB 2 Pests + on-lifegain counter), Prismari Conjurer (Magecraft
ping + loot), Quandrix Calligrapher (enters-with-3-counters + self-
double activation), Silverquill Penmaster (2-mode destroy big / exile
small), Lorehold Treasure Smith (ETB Treasure + sac-Treasure pump),
Witherbloom Tutor (creature tutor for 2 life), Prismari Cartographer
(2-mana Scry 2 + draw), Quandrix Geologist (tap-for-G/U + loot).

Push (modern_decks, claude/modern_decks branch — prior revision —
**21 MORE STX cards + 27 new functionality tests + CR 119 (Life) audit
+ new `Effect::SetLifeTotal` primitive (CR 119.5) + zero-life-gain
trigger test (CR 119.9) = batch 8 — 129 cards total**):

Adds 21 more STX cards across all five colleges: Pestilent Haze
(real STX, mass -2/-2 ChooseMode), Vanquish the Horde (real STX,
mass-destroy approximation w/o cost reduction), Quandrix Doublewright
(ETB Fractal counter + Magecraft self-counter), Lorehold Theorizer
(Vigilance + Magecraft self-pump), Prismari Inventor (Magecraft
Treasure mint), Silverquill Lecturer (Lifelink + Magecraft target
pump), Quandrix Conjurer (Fractal token + counters per creature),
Witherbloom Concoction (3-mana -2/-2 + gain 2 + draw), Prismari
Sparkmage (ETB 2-burn + Magecraft Scry), Silverquill Ambassador
(Flying+Lifelink+ETB Inkling), Lorehold Battlemage (drain ETB +
gy-exile activation), Witherbloom Plaguemage (ETB drain + sac-for-
drain activation), Silverquill Skywriter (Flying + ETB draw +
on-draw drain), Quandrix Curriculum (look-6 dual-tutor sorcery),
Lorehold Researcher (First Strike + dies-recursion), Prismari
Magicraft (5-mana CopySpell + cantrip), Witherbloom Botanist
(Plant Druid + Pest ETB + sac-for-drain), Silverquill Drafter
(3-mode utility), Quandrix Schematist (ETB scry-2 + activated
counter), Lorehold Resurrectionist (Flying + reanimate-with-haste
ETB), Prismari Tinkerer (Prowess + dies-Treasure).

**Engine audit: CR 119 — Life**. Audit row added below for CR 119
(life-total mechanics). Most rules already wired; the headline gap
was **119.5 (set life total to a specific value)** which is now
fixed by the new `Effect::SetLifeTotal { who, amount }` primitive
in `effect.rs:770` + resolver at `game/effects/mod.rs::491`.
Implementation computes `delta = new_total - current_life` then
emits the appropriate `LifeGained` (delta > 0) or `LifeLost` (delta
< 0) event, threading through SBA / triggers correctly. Zero
delta emits no event (CR 119.9 / 119.10 zero-life-change). Tests:
`set_life_total_emits_correct_delta_events_per_cr_119_5`,
`set_life_total_higher_emits_life_gained`,
`zero_life_gain_does_not_trigger_lifegain_events_per_cr_119_9`.

Push (modern_decks, claude/modern_decks branch — prior revision —
**22 MORE STX cards + 24 new functionality tests + CR 701.17b mill
audit + mill-cap test = batch 7 — 108 cards total**):

Adds 22 more STX cards across all five colleges plus engine audits:
Lorehold Scholar (gy-recursion ETB + on-attack indestructible),
Witherbloom Sapfeeder (Magecraft +1/+1 counter), Quandrix
Mathematician (ETB Scry + Magecraft counter), Prismari Mage
(Magecraft optional loot), Silverquill Initiate (First Strike)
(Magecraft self-pump), Lorehold Sparkmage (Haste + ETB ping),
Witherbloom Loremage (Magecraft drain + gy recursion activated),
Quandrix Surge Spell (pump by cards drawn + cantrip), Prismari
Volcanist (ETB drain each opp + Magecraft ping), Lorehold Spellsage
(Magecraft gain 1 + 1 dmg), Silverquill Penmate (counter on
lifegain), Witherbloom Apothecary (sac-into-drain activation),
Quandrix Trampler (enters with counter per other creature),
Prismari Painter (Treasure ETB + sac-Treasure-for-loot),
Lorehold Archivist (on-attack IS gy recursion), Silverquill
Scrivener (optional ETB rummage), Witherbloom Geneticist (counter
on ETB + lifegain), Quandrix Resonator (Scry on +1/+1 counter
trigger), Prismari Wavecaller (ETB cantrip + Magecraft pump),
Lorehold Spiritguide (gy creature → hand + optional rummage),
Silverquill Verse (modal pump + drain + Inkling token),
Witherbloom Quagmage (ETB drain + opp-death gain), Quandrix
Surveyor (ETB tutor basic), Prismari Glitterbomb (3 dmg + Treasure).

**Engine audit: CR 701.17b — mill stops at empty library**. The
engine's `Effect::Mill` handler (`game/effects/mod.rs:595`) already
correctly breaks the per-card loop when the library is empty (the
`if self.players[p].library.is_empty() { break; }` guard at line
600). Lock-in test:
`tests::game::mill_caps_at_library_size_per_cr_701_17b` stages a
3-card library on P0, mills 10, and asserts all 3 cards go to
graveyard (mill 10 → mill 3) and the library is empty. This is the
"mill as many as possible" framing — no error, no truncation panic,
just stops at zero.

Push (modern_decks, claude/modern_decks branch — prior revision —
**22 MORE STX cards + 22 new functionality tests + smarter
Proliferate (CR 701.34) auto-decider + 3 new proliferate tests =
batch 6 — 86 cards total**):

Adds 22 more STX cards across all five colleges plus engine
improvement: Silverquill Tutor (MV≤2 search), Witherbloom Apprentice's
Familiar (small magecraft drain body), Lorehold Investigator (IS gy
recursion), Prismari Ember-Mage (magecraft self-pump 2/3), Quandrix
Calculator (board-wide +1/+1 ETB), Lorehold Spark (Lightning Helix
shape), Witherbloom Tonic (drain 3), Silverquill Scribe (ETB
discard + 1 life), Prismari Maelstrom (creature counter + 2 dmg),
Lorehold Beacon (2× Spirit token mint), Quandrix Mentor (magecraft
+1/+1 counter), Silverquill Riposte (attack-or-block destroy),
Witherbloom Druid-in-Training (Pest ETB), Lorehold Recurrence
(reanimate creature/PW), Prismari Sage (looter + magecraft pump),
Quandrix Aviator (Fractal flying mint), Witherbloom Necromancer
(low-MV reanimator + per-death gain), Silverquill Edict (Diabolic
Edict shape), Lorehold Recall (exile + MV-scaled burn), Quandrix
Refraction (counter creature + scry 2), Prismari Architect (Treasure
ETB + magecraft pump), Witherbloom Briarmage (lifegain → +1/+1
counters), Silverquill Strategist (magecraft drain + per-death gain).

**Engine improvement: smarter Proliferate auto-decider (CR 701.34a)**.
The `Effect::Proliferate` handler now respects the printed "choose
any number" framing — friendlies with +1/+1 counters get bumped;
enemy permanents with +1/+1 counters are skipped (you'd never pump
your opponent). MinusOneMinusOne flips: skip on yours, fire on
theirs. Stun is treated the same way (skip on yours so your
permanents untap; fire on theirs so they stay locked). Poison is
self-exclusive (opp gets +1, you decline). The old strict-superset
fan-out was technically strictly stronger than the rules but
counter-thematically wrong since the auto-decider was always
picking the maximum proliferation set including hostile-to-self
counters. Promotes the CR 701.34 audit entry from "always-yes-on-
everything" to a more faithful "choose-strategically" baseline.

Push (modern_decks, claude/modern_decks branch — prior revision —
**6 MORE STX cards + 6 new tests + tap_add_any_color helper =
batch 5 — 64 cards total**):

Adds 6 more STX cards: Strixhaven Diplomat (Azorius cantrip flier),
Lorehold Banishment (Path to Exile shape), Quandrix Mass Counter
(+2/+2 fan-out), Prismari Storm (4-damage + draw), Witherbloom Plague
(small-creature wipe), Silverquill Aerie (Inkling enchantment).

Engine helper: `catalog::sets::tap_add_any_color()` — a one-line
factory for the `{T}: Add one mana of any color` activated ability,
used by Mage Tower Crystal and unblocking any future rainbow-rock
card without re-spelling the full `ActivatedAbility` literal.

Push (modern_decks, claude/modern_decks branch — prior revision —
**10 MORE STX cards + 10 new tests = batch 4 — 58 cards total**):

Adds 10 more STX cards: Owlin Tactician (ETB pump + grant flying),
Pest Mediator (grows on lifegain), Inkling Aerialist (Inkling-ETB
self-pump), Quandrix Theorist (draw per counter creature), Prismari
Inferno (Pyroclasm scale 3 to each), Lorehold Resurgence (return
MV≤3 to bf), Witherbloom Studies (mill 3 + return to hand),
Silverquill Vanguard (+1/+1 Inkling anthem), Prismari Channeler
(2/3 mana fixer), Lorehold Anthem (Glorious Anthem).

Push (modern_decks, claude/modern_decks branch — prior revision —
**12 MORE STX cards + 12 new tests = batch 3**):

Adds 12 more mono-color staples + cross-college tools: Strixhaven
Footsoldier (1W vigilant), Mage Tower Crystal (rainbow rock), Witherbloom
Adept (3-mana menace), Lorehold Pyromancer (Magecraft +2/+0 self-pump),
Quandrix Defender (Wall + ETB Scry), Silverquill Lifedrain (drain 2),
Witherbloom Plowman (4/3 reach + gain 3), Prismari Spellfire-Sage (Mulldrifter
shape), Lorehold Justice (destroy power 4+), Quandrix Recall (Unsummon),
Witherbloom Pestilence (board -2/-2 EOT), Lorehold Combatant (3-mana
double strike). All 1961 lib tests pass — 48 NEW STX cards across 3
batches this push.

Push (modern_decks, claude/modern_decks branch — prior revision —
**14 MORE STX cards + 14 new functionality tests = batch 2**):

Builds on the previous 22-card extras drop with 14 more STX-themed
cards covering Quandrix / Prismari / Lorehold / Witherbloom: Quandrix
Apprenticeship (counter+scry), Prismari Pyrotechnics (5 damage burn),
Lorehold Strategist (2/2 flying ETB-gain-2), Witherbloom Necromancy
(reanimation), Silverquill Resolve (+1/+3 + lifelink), Prismari Conduit
(2/2 haste attack-loot), Quandrix Doubling (double +1/+1 counters),
Lorehold Smith (2/3 + Treasure ETB), Silverquill Decree (5-mana
removal + lifegain), Witherbloom Wand (artifact drain), Quandrix
Survey (ramp + cantrip), Prismari Arsonist (3/2 flash FTK ETB),
Lorehold Banner (R/W mana rock + ETB gain), Witherbloom Verdict
(target opp sacs a creature). All 1949 lib tests pass.

Push (modern_decks, claude/modern_decks branch — prior revision —
**22 NEW STX cards + 23 new functionality tests**):

Adds 22 new STX-flavored cards to `stx::extras` — Silverquill / Witherbloom
depth + cross-college support. All cards use existing engine primitives
and ship with at least one functional test each.

**NEW STX cards (this push, 22):**

- **Disciplined Duelist** ({1}{W}, 2/1 Human Cleric, First Strike) —
  Aggressive Silverquill body that trades up cleanly.
- **Eager Scribe** ({W}, 1/1 Human Cleric) — Magecraft: Scry 1.
- **Silverquill Pen** ({2} Artifact) — `{2}{W}{B}, {T}: Drain 2 from
  each opponent`.
- **Witherbloom Acolyte** ({B}{G}, 2/1 Human Druid) — Magecraft: gain 1.
- **Witherbloom Toxicology** ({3}{B}{G} Sorcery) — Destroy creature + mint
  a Pest token.
- **Pest Brood Caller** ({2}{B}{G}, 2/2 Human Warlock) — ETB mint two
  Pest tokens.
- **Inkling Caretaker** ({1}{W}{B}, 1/3 Inkling Cleric, Flying+Lifelink).
- **Silverquill Strike** ({W}{B} Instant) — Drain 3 from target opp.
- **Lorehold Reverie** ({R}{W} Sorcery) — Gain 3 life + 3 damage to opp.
- **Prismari Loot** ({U}{R} Instant) — Draw 1 + Discard 1.
- **Quandrix Counterspell** ({G}{U}{U} Instant) — Counter target spell +
  +1/+1 counter on friendly creature.
- **Spell Squelch** ({2}{U} Instant) — Counter target spell (Cancel-shape).
- **Witherbloom Field-Worker** ({1}{G}, 2/2 Human Druid) — ETB gain 2.
- **Lorehold Wayfinder** ({2}{R}{W}, 3/3 Spirit Cleric) — ETB Mill 2.
- **Prismari Brilliance** ({U}{R} Sorcery) — Scry 2 + Draw 1.
- **Quandrix Tutor** ({2}{G}{U} Sorcery) — Search creature to hand.
- **Silverquill Cantrip** ({1}{W} Instant) — Gain 2 + Draw 1.
- **Witherbloom Reanimator** ({3}{B}{G}, 2/3 Human Warlock) — ETB return
  creature card from gy to hand.
- **Lorehold Lightning** ({1}{R} Instant) — 3 damage to creature.
- **Quandrix Engineer** ({1}{G}{U}, 2/3 Elf Druid) — `{T}: Add {G} or {U}`.
- **Prismari Pyromage** ({2}{R}, 2/2 Human Wizard) — Magecraft: 1 damage
  to any target.
- **Lorehold Curator** ({2}{W}, 2/3 Spirit Soldier) — ETB return creature
  card MV≤2 from gy to hand.
- **Witherbloom Scholar** ({1}{B}, 2/1 Human Warlock) — Magecraft: drain 1.

Push (modern_decks, claude/modern_decks branch — prior revision —
**9 more STX cards + Value::CountersOn fan-out summation engine fix + 9 new tests**):

Adds 9 more STX cards to `stx::extras` (Inkling Sentinel, Witherbloom
Ritualist, Quandrix Theorem, Prismari Surge, Lorehold Conservator,
Silverquill Initiate, Witherbloom Channeler, Lorehold Mentor, Prismari
Bauble). Engine improvement: `Value::CountersOn { what }` now SUMS
counters across all entities `what` resolves to (single-entity
selectors still return that one entity's count). Lock-in test:
`reflective_anatomy_pumps_target_by_total_counters` exercises the
sum behavior with two creatures (2 + 1 counters → +3/+3 pump). All
1910 tests pass.

Push (modern_decks, claude/modern_decks branch — prior revision —
**20 NEW STX cards + 5 NEW Lessons + 27 new functionality tests + CR 111 Tokens audit**):

This push adds 20 new card factories to `stx::extras` (focused on
Silverquill college depth) plus 5 new Lesson cards in `stx::lessons`
(Pest Inheritance, Mascot Interpretation, Reduce // Rubble, Containment
Studies, Reflective Anatomy). All 1922+ tests pass.

**NEW STX cards (this push, 20):**

- **Silverquill Apprentice** ({W}{B}, 2/2 Human Wizard) — Magecraft
  +1/+1 counter on target creature (the missing Silverquill Apprentice
  matching the cycle of college Apprentices).
- **Pestilent Lecturer** ({1}{W}{B}, 2/3 Inkling Cleric with Flying) —
  ETB drain 1.
- **Shadow-Mage Hopeful** ({1}{W}{B}, 2/2 Human Wizard with Lifelink) —
  Magecraft drain 1.
- **Quill Page** ({W}, 1/1 Human Cleric) — Magecraft Scry 1.
- **Inkbond Cleric** ({2}{W}, 2/3 Human Cleric) — Surveil 1 + counter
  on another Inkling.
- **Quill Inscriber** ({1}{B}, 2/2 Human Warlock) — Magecraft self-pump
  +1/+0 EOT.
- **Pestilent Squire** ({1}{B}, 2/1 Pest Warrior with Lifelink).
- **Silverquill Mediator** ({3}{W}{B}, 3/4 Inkling Cleric with Flying +
  Lifelink) — ETB drain 2.
- **Dissident Lecturer** ({2}{B}, 2/3 Human Warlock) — Magecraft burn
  each opp for 1 (no lifegain rider).
- **Silverquill Persuader** ({2}{W}{B}, 2/3 Inkling Wizard with Flying)
  — Cleric tribal anthem.
- **Pestilent Imp** ({B}, 1/1 Imp Pest with Flying).
- **Witherbloom Tincture-Maker** ({1}{B}{G}, 2/3 Human Druid) — Pure
  lifegain Magecraft.
- **Lorehold Crusader** ({2}{R}{W}, 3/3 Spirit Soldier with First
  Strike + Vigilance).
- **Quandrix Initiate** ({G}{U}, 1/2 Elf Druid) — Magecraft self
  +1/+1 counter.
- **Lorehold Wand** ({2} Artifact) — `{2}{R}, {T}: 2 damage to any
  target.`
- **Witherbloom Bramble** ({1}{B}{G} Sorcery) — Mints a Pest + +1/+1
  counter on each creature you control.
- **Prismari Spark** ({U}{R} Instant) — 2 damage to creature + draw 1.
- **Quandrix Trickster** ({1}{U}, 2/1 Merfolk Wizard with Flash) — ETB
  -2/-0 EOT on target.
- **Lorehold Memorialist** ({R}{W} Sorcery) — Return creature card
  from gy → hand.
- **Witherbloom Researcher** ({2}{B}{G}, 3/3 Human Druid) — ETB +2
  life + draw.
- **Quandrix Catalyst** ({1}{G}{U} Sorcery) — Put 2 counters on target
  then double.
- **Lorehold Vanguard** ({R}{W}, 2/2 Spirit Soldier with Haste).

**NEW STX Lessons (this push, 5):**

- **Pest Inheritance** ({3}{G} Sorcery — Lesson) — Mint Pests equal to
  lands you control. Uses `Value::CountOf(Land & ControlledByYou)` for
  the X token count. Engine support already exists in
  `Effect::CreateToken { count: Value }`.
- **Mascot Interpretation** ({1}{U} Sorcery — Lesson) — Two +1/+1
  counters on target creature you control + Learn (cantrip).
- **Reduce // Rubble** ({2}{R} Sorcery — Lesson) — 3 damage to
  creature/PW + Learn (cantrip).
- **Containment Studies** ({2}{W} Sorcery — Lesson) — Tap target
  creature + put 2 stun counters on it.
- **Reflective Anatomy** ({2}{G}{U} Sorcery — Lesson) — Target
  creature gets +X/+X EOT, where X is the total number of +1/+1
  counters on creatures you control. Uses the existing
  `Value::CountersOn { what: EachPermanent(filter), kind: +1/+1 }`
  fan-out to sum counters across the board.

**CR 111 audit row** (added to TODO.md): Tokens — Engine handles the
core token semantics correctly (111.7 ceases-to-exist in non-bf zones
via the SBA in `check_state_based_actions`, 111.8 LBF tokens can't
re-enter, 111.10 predefined tokens via `TokenDefinition`). Triggered
abilities on tokens (111.10a Treasure, 111.10b Food, 111.10g Blood)
all use the `TokenDefinition.triggered_abilities` field added in
SOS-VI. The 111.5 "creates a token that's a copy of an instant or
sorcery card, no token is created" corner is a no-op (engine has no
copy-token-of-spell primitive). Promoted to ✅ in TODO.md.

Push (modern_decks, claude/modern_decks branch — prior revision —
**43 NEW STX cards + 73 new functionality tests + Augusta promotion + CR 506 audit**):

This push adds 36 new card factories to `stx::extras` focused on
Silverquill (W/B) school depth + cross-college utility. All 1855 tests
pass (+61 from prior baseline). Cards cover Inkling tribal anthems,
Magecraft drain/draw payoffs, life-gain triggers, modal ETB minions,
flash-and-burn instants, dual-pip evasion bodies, and Lorehold/Quandrix/
Prismari support cards. Includes one partial promotion (Augusta, Dean
of Order body-only → per-attacker +1/+1+Vigilance approximation).

**NEW STX cards (this push, 36):**

- **Inkling Scholar** ({2}{W}{B}, 3/3 Inkling Cleric with Flying +
  Lifelink) — Tribal-anthem target.
- **Inkling Squire** ({W}, 1/1 Inkling Soldier with Flying) — Cheap
  Inkling tribal.
- **Silverquill Scholar** ({W}{B}, 2/1 Human Wizard) — Magecraft draw
  + lose 1.
- **Inkling Reinforcement** ({W}{B}, Sorcery) — Two 1/1 Inkling tokens
  with Flying.
- **Pestilent Verse** ({1}{B}{B}, Sorcery) — Destroy target creature,
  lose 1 life.
- **Inkling Ambusher** ({2}{B}, 2/2 Inkling Rogue with Flash + Flying).
- **Silver-Quill Scholarship** ({2}{W}, Sorcery) — +1/+1 counter on
  target creature + cantrip.
- **Silvercrown Lecturer** ({3}{W}, 2/4 Human Cleric) — ETB +1/+1
  counter on friendly creature.
- **Demolishing Lecture** ({2}{B}, Sorcery) — Destroy target creature
  with toughness 2 or less.
- **Inkling Mentor** ({3}{W}{B}, 3/4 Human Cleric) — Inkling tribal
  +1/+1 anthem (StaticEffect::PumpPT with `OtherThanSource`).
- **Pestilent Inkmage** ({2}{W}{B}, 2/4 Human Wizard with Lifelink) —
  Magecraft self-pump (+2/+0 EOT).
- **Inkling Reaver** ({3}{B}, 3/3 Inkling Warrior with Menace).
- **Quintessential Inkling** ({1}{W}{B}, 2/2 Inkling Spirit with
  Flying + Lifelink).
- **Quill Witch** ({1}{B}{B}, 2/2 Human Warlock with Flying) —
  Magecraft drain 1.
- **Lesson in Honor** ({1}{W}, Sorcery — Lesson) — +2/+2 EOT + Learn.
- **Inkling Squad** ({3}{W}{B}, Sorcery) — Three 1/1 Inkling tokens
  with Flying.
- **Inkling Drillmaster** ({1}{W}, 1/2 Inkling Soldier with Flying) —
  ETB +1/+1 counter on another Inkling.
- **Sealing Verse** ({W}{B}, Instant) — Exile target creature with
  MV ≤ 3.
- **Strict Tutelage** ({1}{W}{B}, Enchantment) — Whenever opp draws,
  they lose 1 life.
- **Inkrise Vampire** ({2}{B}, 2/3 Vampire Warlock with Lifelink).
- **Silverquill Sting** ({W}{B}, Instant) — Drain 2 from target opp.
- **Blade Historian** ({2}{R}{W}, 3/2 Human Wizard) — Magecraft pump
  attackers +1/+0 and grant double strike EOT.
- **Carving Cherub** ({W}, 1/1 Spirit) — Magecraft +1/+1 EOT on
  target creature.
- **Inkrider Witch** ({1}{B}, 2/2 Human Rogue with Menace).
- **Roving Scholar** ({3}{U}, 2/3 Human Wizard) — Howling Mine ETB
  (each player draws 2).
- **Forceful Mirror** ({2}{U}, Sorcery) — Copy target IS spell.
- **Fractalic Discovery** ({2}{G}{U}, Sorcery) — Draw 3, put 2 on top.
- **Lorehold Lookback** ({2}{R}{W}, Sorcery) — Return creature/artifact
  from gy + mint a 2/2 R/W Spirit token.
- **Witherbloom Reaper Spirit** ({2}{B}{G}, 4/3 Plant Spirit with
  Deathtouch).
- **Witherbloom Lifedrinker** ({1}{B}, 1/3 Plant Warlock with
  Lifelink) — Grows on each lifegain trigger.
- **Lorehold Battlemaster** ({2}{R}{W}, 3/3 Spirit Cleric with Haste
  + First Strike).
- **Prismari Spellfire** ({3}{U}{R}, Sorcery) — 5 damage to creature/
  PW + cantrip.
- **Quandrix Recalibrator** ({1}{G}{U}, 2/2 Elf Wizard) — ETB +1/+1
  counter on each friendly creature.
- **Crackleburr Initiate** ({U}{R}, 2/1 Human Wizard with Flash) —
  Magecraft self-pump +1/+0 EOT.
- **Spellseeker's Insight** ({1}{U}, Instant) — Tutor IS with MV ≤ 3.
- **Inkling Aether-Smith** ({2}{W}{B}, 2/3 Inkling Artificer with
  Flying) — Modal ETB: token or +1/+1 counter.
- **Burrog Snapper** ({1}{U}, 2/2 Frog Wizard with Flash) — ETB target
  creature -2/-0 EOT.
- **Galvanic Ribbons** ({1}{R}, Instant) — 2 dmg + draw 1 if you
  control an artifact.
- **Plant Mascot** ({1}{G}, 2/2 Plant) — ETB +1/+1 EOT on friendly.
- **Quandrix Wavebender** ({1}{G}{U}, 2/3 Elf Druid) — Whenever you
  cast a spell with {X} in its mana cost, put X +1/+1 counters on
  this creature.
- **Tezzeret's Inkling Forge** ({1}{W}{B}, Enchantment) — End-step
  Inkling token generator.
- **Quandrix Snake-Charmer** ({2}{G}, 3/3 Snake Druid) — ETB cantrip
  (Elvish Visionary upgrade).
- **Witherbloom Necrotouch** ({2}{B}{G}, Instant) — Destroy target
  creature + 2 life.

**Augusta, Dean of Order — 🟡 → 🟡 (improved)**: Body-only wire upgraded
to a per-attacker `Attacks/AnotherOfYours` trigger that pumps the
attacker +1/+1 EOT and grants Vigilance EOT (auto-picks Vigilance from
the printed "your choice of flying/first strike/vigilance/lifelink" —
the most generally useful for chained attacks). The "three or more with
same power" gate is omitted (engine has no "attacking creatures with
same power" predicate). Same `Attacks/AnotherOfYours` per-attacker
emission model as Sparring Regimen.

Push (modern_decks, claude/modern_decks branch — prior revision —
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
| Soaring Stoneglider | {2}{W} | Creature — Elephant Cleric | 4/3 | As an additional cost to cast this spell, exile two cards from your graveyard or pay {1}{W}. / Flying, vigilance | ✅ (was 🟡) | Push (modern_decks batch 29): the alt additional cost (exile two cards from graveyard) is **now wired** via the new `AlternativeCost.exile_from_graveyard_count: u32` field. Default cost {3}{W} = base {2}{W} + {1}{W} mana fork; alt cast path {2}{W} requires `exile_from_graveyard_count: 2` (rejected when gy has < 2 cards). Auto-picker takes the lowest-CMC cards so high-value gy cards stay put. Body (4/3 Flying + Vigilance) unchanged. Tests: `soaring_stoneglider_is_four_three_flier_vigilance`, `soaring_stoneglider_alt_cost_exiles_two_from_graveyard`, `soaring_stoneglider_alt_cost_rejects_with_insufficient_graveyard`. |
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
| Mana Sculpt | {1}{U}{U} | Instant |  | Counter target spell. If you control a Wizard, add an amount of {C} equal to the amount of mana spent to cast that spell at the beginning of your next main phase. | ✅ (was 🟡) | Push (modern_decks): the "amount of mana spent on the countered spell" rider **is now wired** — after CounterSpell resolves the target sits in graveyard, and `Value::ManaValueOf(Target(0))` walks gy to find it. The added colorless mana count = the countered spell's printed CMC (X-cost spells under-count by the X paid; same gap as Opus's mana-introspection approximation). The "delay-until-next-main" timing rider still collapses to immediate AddMana. Tests: `mana_sculpt_counters_spell`, `mana_sculpt_refunds_mana_value_of_countered_spell_with_wizard`. |
| Mathemagics | {X}{X}{U}{U} | Sorcery |  | Target player draws 2ˣ cards. (2º = 1, 2¹ = 2, 2² = 4, 2³ = 8, 2⁴ = 16, 2⁵ = 32, and so on.) | ✅ | Wired in `catalog::sets::sos::sorceries` via the new `Value::Pow2(XFromCost)` primitive. Multi-target slot collapsed to "you" (caster draws); exponent capped at 30 to avoid deck-out. |
| Matterbending Mage | {2}{U} | Creature — Human Wizard | 2/2 | When this creature enters, return up to one other target creature to its owner's hand. / Whenever you cast a spell with {X} in its mana cost, this creature can't be blocked this turn. | ✅ | Push XVI: both abilities wired. ETB bounce stays as before; the X-cast trigger uses the new `Predicate::CastSpellHasX` + `Effect::GrantKeyword(Unblockable, EOT)` on `Selector::This`. |
| Muse Seeker | {1}{U} | Creature — Elf Wizard | 1/2 | Opus — Whenever you cast an instant or sorcery spell, draw a card. Then discard a card unless five or more mana was spent to cast that spell. | ✅ | Push XXIX: Body + Opus rider wired via `shortcut::opus_trigger`. Small body draws + discards; big body (≥5 mana) skips the discard. |
| Muse's Encouragement | {4}{U} | Instant |  | Create a 3/3 blue and red Elemental creature token with flying. / Surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | ✅ | Mints a 3/3 U/R Flying Elemental via the shared `elemental_token()` helper + `Effect::Surveil 2`. |
| Orysa, Tide Choreographer | {4}{U} | Legendary Creature — Merfolk Bard | 2/2 | This spell costs {3} less to cast if creatures you control have total toughness 10 or greater. / When Orysa enters, draw two cards. | ✅ (was 🟡) | Push (modern_decks): the conditional "{3} less if total toughness ≥ 10" alt-cost rider **is now wired** via the new `AlternativeCost.condition: Option<Predicate>` field. Predicate: `ValueAtLeast(ToughnessOf(EachPermanent(Creature ∧ ControlledByYou)), 10)`. `Value::ToughnessOf` now sums across fan-out selectors (push modern_decks engine fix), so 5 bears = 10 total toughness opens the {1}{U} alt cost path. ETB draw 2 unchanged. Tests: `orysa_etb_draws_two_cards`, `orysa_alt_cost_rejected_when_total_toughness_under_ten`, `orysa_alt_cost_succeeds_when_total_toughness_ten_or_more`. |
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
| End of the Hunt | {1}{B} | Sorcery |  | Target opponent exiles a creature or planeswalker they control with the greatest mana value among creatures and planeswalkers they control. | ✅ (was 🟡) | Push (modern_decks): the "greatest mana value" clause **is now enforced** via the new `SelectionRequirement::HasGreatestManaValueAmongControlled(Box<inner>)` predicate. Inner filter = `Creature ∨ Planeswalker`; the outer predicate checks the candidate's MV ≥ every other matching permanent under the same controller (ties pass permissively). Cast-time validator + auto-target both consult this — so the caster can only exile the largest opp creature/PW. Tests: `end_of_the_hunt_exiles_opponent_creature`, `end_of_the_hunt_rejects_smaller_target_when_greater_mv_exists`, `end_of_the_hunt_picks_largest_creature_when_targeting_max`. |
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
| Choreographed Sparks | {R}{R} | Instant |  | This spell can't be copied. / Choose one or both — / • Copy target instant or sorcery spell you control. You may choose new targets for the copy. / • Copy target creature spell you control. The copy gains haste and "At the beginning of the end step, sacrifice this token." | 🟡 | Push (modern_decks): two-mode `ChooseMode` now wired. Mode 0 copies a target IS spell (existing CopySpell). Mode 1 copies a target creature spell on the stack — `CopySpell` already handles permanent spells (stamps `is_token = true` per CR 608.3f), so the copy resolves as a token bear. The printed "haste + sacrifice at end step" rider approximates via the token-cleanup SBA. The "choose one or both" multi-mode rider still collapses to "pick one mode" (engine-wide multi-mode-with-per-mode-targets gap). Tests: `choreographed_sparks_copies_target_instant_you_control`, `choreographed_sparks_mode_one_copies_target_creature_spell`. |
| Duel Tactics | {R} | Sorcery |  | Duel Tactics deals 1 damage to target creature. It can't block this turn. / Flashback {1}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Wired as `DealDamage(1) + GrantKeyword(CantBlock, EOT)` — pulls in the new `Keyword::CantBlock` (enforced inside `declare_blockers` and the `can_block_*` helpers). Flashback {1}{R} now wired via `Keyword::Flashback` (push X). |
| Emeritus of Conflict // Lightning Bolt | {1}{R} // {R} | Creature — Human Wizard // Instant | 2/2 |  | ✅ | Push XXVIII promotion: vanilla 2/2 Human Wizard front + faithful Lightning Bolt back (`DealDamage 3 to target`). All Oracle clauses wired. Test: `emeritus_of_conflict_back_face_burns_three`. |
| Expressive Firedancer | {1}{R} | Creature — Human Sorcerer | 2/2 | Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If five or more mana was spent to cast that spell, this creature also gains double strike until end of turn. | ✅ | Push XXIX: Opus rider **now wired** via `shortcut::opus_trigger`. Small body: +1/+1 EOT. Big body (≥5 mana): +1/+1 EOT + DoubleStrike EOT. Test: `expressive_firedancer_opus_grants_double_strike_at_five_mana`. |
| Flashback | {R} | Instant |  | Target instant or sorcery card in your graveyard gains flashback until end of turn. The flashback cost is equal to its mana cost. (You may cast that card from your graveyard for its flashback cost. Then exile it.) | 🟡 | Push XXVI: Approximated as a {R} "return a target IS card from your graveyard to your hand" instant — the player can re-cast it next turn at its normal cost. Strictly weaker than the printed "flashback for its mana cost this turn" but preserves the recovery outcome. A true wiring needs a transient per-instance grant on a graveyard card. |
| Garrison Excavator | {3}{R} | Creature — Orc Sorcerer | 3/4 | Menace (This creature can't be blocked except by two or more creatures.) / Whenever one or more cards leave your graveyard, create a 2/2 red and white Spirit creature token. | ✅ | Wired against the new `EventKind::CardLeftGraveyard` event — every gy-leave mints a 2/2 R/W Spirit token via the shared `spirit_token()` helper. |
| Goblin Glasswright // Craft with Pride | {1}{R} // {R} | Creature — Goblin Sorcerer // Sorcery | 2/2 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Heated Argument | {4}{R} | Instant |  | Heated Argument deals 6 damage to target creature. You may exile a card from your graveyard. If you do, Heated Argument also deals 2 damage to that creature's controller. | ✅ | Push XV → ✅ in push XXVIII: 6-to-creature is unconditional; the gy-exile + 2-to-controller chain is wrapped in `Effect::MayDo` and either both fire or both skip — faithful to the printed "you may". Uses `Selector::take(CardsInZone(GY), 1)` to pick exactly one gy card (matching "a card", not "every card"). |
| Impractical Joke | {R} | Sorcery |  | Damage can't be prevented this turn. Impractical Joke deals 3 damage to up to one target creature or planeswalker. | ✅ (was 🟡) | Push (modern_decks doc-sync): 3-to-creature/PW wired faithfully. The "damage can't be prevented this turn" rider is a true no-op in this engine — the only prevention layer (`prevent_combat_damage_this_turn` flag, set by Owlin Shieldmage / Holy Day-style fogs) only intercepts combat damage. Impractical Joke deals spell damage, which has no prevention layer to gate, so the rider's effect on this card's resolution is already realised. The "up to one" rider is approximated as required-target (single Creature ∨ Planeswalker filter); the target-required vs. target-optional gap is shared engine-wide and rarely exercised by a 1-mana burn spell that almost always has a legal target. |
| Improvisation Capstone | {5}{R}{R} | Sorcery — Lesson |  | Exile cards from the top of your library until you exile cards with total mana value 4 or greater. You may cast any number of spells from among them without paying their mana costs. / Paradigm (Then exile this spell. After you first resolve a spell with this name, you may cast a copy of it from exile without paying its mana cost at the beginning of each of your first main phases.) | ⏳ | 🔍 needs review (oracle previously truncated). Needs: copy-spell/permanent primitive; cast-from-exile pipeline. |
| Living History | {1}{R} | Enchantment |  | When this enchantment enters, create a 2/2 red and white Spirit creature token. / Whenever you attack, if a card left your graveyard this turn, target attacking creature gets +2/+0 until end of turn. | ✅ (was 🟡) | Push (modern_decks doc-sync): ETB Spirit token + on-attack +2/+0 EOT (gated on `Predicate::CardsLeftGraveyardThisTurnAtLeast`). The "target attacking creature" picks the trigger source (the just-declared attacker) — same per-attacker pattern as Sparring Regimen ✅ / Mentor in Combat Professor ✅. The auto-target framework correctly lands the pump on the iterated attacker. Test: `living_history_etb_creates_spirit_token`. |
| Maelstrom Artisan // Rocket Volley | {1}{R}{R} // {1}{R} | Creature — Minotaur Sorcerer // Sorcery | 3/2 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Magmablood Archaic | {2/R}{2/R}{2/R} | Creature — Avatar | 2/2 | Trample, reach / Converge — This creature enters with a +1/+1 counter on it for each color of mana spent to cast it. / Whenever you cast an instant or sorcery spell, creatures you control get +1/+0 until end of turn for each color of mana spent to cast that spell. | ✅ (was 🟡) | Push (modern_decks): the per-cast pump **is now wired** via `magecraft(Effect::PumpPT { what: EachPermanent(Creature ∧ ControlledByYou), power: ConvergedValue, .. })`. Engine fix: `fire_spell_cast_triggers` now threads the just-cast spell's `converged_value` onto the resulting `StackItem::Trigger.converged_value` (previously hard-coded to 0). So a 2-color IS cast pumps each of your creatures by +2/+0 EOT; a 5-color cast by +5/+0. Hybrid `{2/R}` pips approximated as `{2}+{R}` per pip. Tests: `magmablood_archaic_etb_with_converged_value_counters`, `magmablood_archaic_pumps_friendly_creatures_on_two_color_cast`. |
| Mica, Reader of Ruins | {3}{R} | Legendary Creature — Human Artificer | 4/4 | Ward—Pay 3 life. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays 3 life.) / Whenever you cast an instant or sorcery spell, you may sacrifice an artifact. If you do, copy that spell and you may choose new targets for the copy. | ✅ (was 🟡) | Push (modern_decks): Ward—Pay 3 life now wired via `Keyword::Ward(WardCost::Life(3))` and the new `Effect::CounterUnless` resolver — auto-pays by deducting 3 life from the spell controller (CR 119.4: payment fails if the controller doesn't have ≥3 life, countering the spell). Magecraft sac-artifact-to-copy rider unchanged. Tests: `ward_pay_life_counters_when_payer_has_insufficient_life`, `ward_pay_life_resolves_when_payer_has_sufficient_life`. |
| Molten-Core Maestro | {1}{R} | Creature — Goblin Bard | 2/2 | Menace / Opus — Whenever you cast an instant or sorcery spell, put a +1/+1 counter on this creature. If five or more mana was spent to cast that spell, add an amount of {R} equal to this creature's power. | ✅ | Push XXIX: Opus rider **now wired** via `shortcut::opus_trigger`. Small body: +1/+1 counter on this creature. Big body (≥5 mana): counter + add {R}×power via `ManaPayload::OfColor(Red, PowerOf(This))`. |
| Pigment Wrangler // Striking Palette | {4}{R} // {R} | Creature — Orc Sorcerer // Sorcery | 4/4 |  | ✅ (was 🟡) | Push (modern_decks doc-sync): vanilla front + faithful back-face spell wired via the `GameAction::CastSpellBack` path (push XI/XII). The stale "Standard primitives — should be straightforward to wire" note was the original ⏳ flag from before MDFC plumbing landed; the body has been at-parity-with-printed-Oracle since push XII. Tests live in `tests::sos` keyed by the back-face spell name.|
| Rearing Embermare | {4}{R} | Creature — Horse Beast | 4/5 | Reach, haste | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Rubble Rouser | {2}{R} | Creature — Dwarf Sorcerer | 1/4 | When this creature enters, you may discard a card. If you do, draw a card. / {T}, Exile a card from your graveyard: Add {R}. When you do, this creature deals 1 damage to each opponent. | ✅ (was 🟡) | Push (modern_decks): the `{T}, Exile a card from your graveyard:` activated ability **is now wired** via the existing `ActivatedAbility.exile_other_filter: Some(Any)` field (same primitive as Postmortem Professor + Lorehold Pledgemage). The "When you do" sub-trigger collapses into the activation's main body — `Seq(AddMana(R), DealDamage 1 each opp)`. ETB rummage unchanged. Tests: `rubble_rouser_etb_rummages`, `rubble_rouser_activation_exiles_gy_card_pings_opp_and_adds_red`, `rubble_rouser_activation_rejected_with_empty_graveyard`. |
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
| Ambitious Augmenter | {G} | Creature — Turtle Wizard | 1/1 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / When this creature dies, if it had one or more counters on it, create a 0/0 green and blue Fractal creature token, then put this creature's counters on that token. | ✅ (was 🟡) | Push (modern_decks): both clauses **now wired**. Increment via `shortcut::increment_self_plus_one()` (already wired); death trigger via `CreatureDied/SelfSource` + `If(CountersOn(This, +1/+1) ≥ 1) { CreateToken(Fractal), AddCounter(LastCreatedToken, +1/+1, CountersOn(This, +1/+1)) }`. CR 122.2 — counters persist on `CardInstance` across the bf → gy zone change, so `Value::CountersOn` walks gy to find the dying card's preserved counter count. Tests: `ambitious_augmenter_is_one_one_turtle_wizard`, `ambitious_augmenter_increments_on_three_mana_cast`, `ambitious_augmenter_death_with_counters_creates_fractal_with_counters`, `ambitious_augmenter_death_without_counters_does_not_create_fractal`. |
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
| Topiary Lecturer | {2}{G} | Creature — Elf Druid | 1/2 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / {T}: Add an amount of {G} equal to this creature's power. | ✅ (was 🟡) | Push (modern_decks doc-sync): both clauses wired. `{T}: Add {G}-many G` uses `ManaPayload::OfColor(Green, PowerOf(This))`. Increment uses `shortcut::increment_self_plus_one()` (the helper added in push XXIX; doc note was stale). Tests: `topiary_lecturer_taps_for_g_equal_to_power`, `topiary_lecturer_increment_lands_counter_on_three_mana_cast`. |
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
| Witherbloom, the Balancer | {6}{B}{G} | Legendary Creature — Elder Dragon | 5/5 | Affinity for creatures (This spell costs {1} less to cast for each creature you control.) / Flying, deathtouch / Instant and sorcery spells you cast have affinity for creatures. | ✅ (was 🟡) | Push (modern_decks batch 25): both Affinity-for-creatures clauses **now land**. The **self-cast** discount uses the new card-intrinsic `CardDefinition.affinity_filter` slot (Creature & ControlledByYou). The **IS-spell grant** is wired via the new `StaticEffect::GrantAffinityToISSpells { permanent_filter }` static — `cost_reduction_for_spell` reads this at every IS cast on the controller's side and adds 1 per matching battlefield permanent (only fires when the source is an instant or sorcery and the caster matches). Tests: `witherbloom_balancer_etb_with_keywords`, `witherbloom_balancer_affinity_for_creatures_reduces_cost` (4 of your creatures → casts at {2}{B}{G}), `witherbloom_balancer_grants_affinity_to_is_spells` (Mind Rot {2}{B} → {B} with Balancer + 1 bear), `witherbloom_balancer_static_does_not_affect_opp_spells` (opp's Mind Rot still costs {2}{B}). |

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
| Social Snub | {1}{W}{B} | Sorcery |  | When you cast this spell while you control a creature, you may copy this spell. / Each player sacrifices a creature of their choice. Each opponent loses 1 life and you gain 1 life. | ✅ (was 🟡) | Push (modern_decks doc-sync): all three printed clauses ship. (a) Cast-IS-while-you-control-a-creature copy via `Effect::CopySpell` + `Predicate::SelectorExists(EachPermanent(Creature ∧ ControlledByYou))` trigger filter. (b) Each-player-sac via `ForEach(EachPlayer)` + `Sacrifice { who: Triggerer }` — each iterated player picks their own sac (auto-decider per-player; in bot harness this is the player's own auto-decider so the "of their choice" semantics are CR-correct under that decision shape). (c) Drain 1 (each opp loses 1, you gain 1). The "of their choice" wording is mechanically honored by the per-player Sacrifice with auto-picker; a UI player would surface a real choice prompt. |
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
| Lorehold, the Historian | {3}{R}{W} | Legendary Creature — Elder Dragon | 5/5 | Flying, haste / Each instant and sorcery card in your hand has miracle {2}. (You may cast a card for its miracle cost when you draw it if it's the first card you drew this turn.) / At the beginning of each opponent's upkeep, you may discard a card. If you do, draw a card. | 🟡 | Push (modern_decks): the per-opp-upkeep loot trigger **is now wired** via `EventSpec::new(StepBegins(Upkeep), EventScope::OpponentControl)` — the engine's step-trigger dispatcher already supports `OpponentControl`-scoped triggers (source's controller ≠ active player). Body is `MayDo(Seq(Discard 1, Draw 1))` so the controller opts into the loot. The Miracle grant on IS in hand is still ⏳ (no Miracle keyword / alt-cost-on-draw primitive). Tests: `lorehold_the_historian_is_five_five_flyer_haste`, `lorehold_the_historian_opp_upkeep_loots_with_scripted_yes`. |
| Molten Note | {X}{R}{W} | Sorcery |  | Molten Note deals damage to target creature equal to the amount of mana spent to cast this spell. Untap all creatures you control. / Flashback {6}{R}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ (was 🟡) | Push (modern_decks): damage now reads `Value::CastSpellManaSpent` (the actual mana paid for the cast), matching the printed "amount of mana spent" Oracle exactly — at X=2 the spell deals 4 damage (X + R + W), so a 4-toughness creature dies. The prior `Value::XFromCost` undercounted by 2 (the {R}{W} pips). Untap all your creatures wired. Flashback {6}{R}{W} via `Keyword::Flashback` — when flashbacked the cast reads mana_spent = 8 (6 + R + W) so the damage scales correctly. Tests: `molten_note_deals_x_damage_and_untaps_your_creatures`, `molten_note_damage_equals_total_mana_spent_not_just_x`, `molten_note_has_flashback_keyword`. |
| Practiced Scrollsmith | {R}{R/W}{W} | Creature — Dwarf Cleric | 3/2 | First strike / When this creature enters, exile target noncreature, nonland card from your graveyard. Until the end of your next turn, you may cast that card. | 🟡 | Wired in `catalog::sets::sos::creatures`. ETB now exiles **exactly one** matching noncreature/nonland card in the controller's graveyard via the new `Selector::Take(_, 1)` primitive (push X) — closer to the printed "target one card" semantics; the prior implementation exiled every matching card. The hybrid `{R/W}` pip is approximated as `{R}` (cost: `{R}{R}{W}`). The "may cast until next turn" rider is omitted (no cast-from-exile-with-time-limit primitive). |
| Pursue the Past | {R}{W} | Sorcery |  | You gain 2 life. You may discard a card. If you do, draw two cards. / Flashback {2}{R}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Push XV → ✅ in push XXVIII: gain 2 + the discard+draw chain wrapped in `Effect::MayDo` so the printed "you may discard" optionality is honored. Flashback wired via `Keyword::Flashback`. The lifegain half always resolves; the loot half is opt-in. All Oracle clauses wired. |
| Spirit Mascot | {R}{W} | Creature — Spirit Ox | 2/2 | Whenever one or more cards leave your graveyard, put a +1/+1 counter on this creature. | ✅ | Wired against the new `EventKind::CardLeftGraveyard` event. Trigger fires per-card emission (the printed "one or more" wording is approximated per-card). |
| Startled Relic Sloth | {2}{R}{W} | Creature — Sloth Beast | 4/4 | Trample, lifelink / At the beginning of combat on your turn, exile up to one target card from a graveyard. | ✅ | Wired in `catalog::sets::sos::creatures` (trample + lifelink + begin-combat exile-from-GY trigger; same shape as Ascendant Dustspeaker's combat trigger). Sloth subtype bridged through Beast (no Sloth creature type yet). |
| Suspend Aggression | {1}{R}{W} | Instant |  | Exile target nonland permanent and the top card of your library. For each of those cards, its owner may play it until the end of their next turn. | 🟡 | Wired in `catalog::sets::sos::instants` as a `Seq` of two `Move → Exile` calls (target nonland permanent + caster's top of library). `move_card_to` was extended to walk libraries when locating the source card so the top-of-library exile resolves end-to-end. The "may play those cards until next end step" rider is omitted (no per-card "may-play-from-exile-until-EOT" primitive). |
| Wilt in the Heat | {2}{R}{W} | Instant |  | This spell costs {2} less to cast if one or more cards left your graveyard this turn. / Wilt in the Heat deals 5 damage to target creature. If that creature would die this turn, exile it instead. | 🟡 | Push (modern_decks): the "{2} less if cards left your graveyard this turn" cost-reduction clause **is now wired** via the new `AlternativeCost.condition: Option<Predicate>` field gated on `CardsLeftGraveyardThisTurnAtLeast(You, 1)`. Caster gets a {R}{W} alt cast path when the gate passes. The "if would die, exile instead" damage-replacement rider is still ⏳ (no damage-replacement primitive). Tests: `wilt_in_the_heat_deals_five_to_creature`, `wilt_in_the_heat_alt_cost_rejected_with_empty_graveyard_history`, `wilt_in_the_heat_alt_cost_succeeds_after_graveyard_recursion`. |

## Colorless

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Biblioplex Tomekeeper | {4} | Artifact Creature — Construct | 3/4 | When this creature enters, choose up to one — / • Target creature becomes prepared. (Only creatures with prepare spells can become prepared.) / • Target creature becomes unprepared. | 🟡 | Push XXV: Body wired (3/4 Construct artifact creature). The Prepare toggle is omitted — engine has no Prepare keyword nor a prepared-state flag (same gap as Skycoach Waypoint). |
| Diary of Dreams | {2} | Artifact — Book |  | Whenever you cast an instant or sorcery spell, put a page counter on this artifact. / {5}, {T}: Draw a card. This ability costs {1} less to activate for each page counter on this artifact. | ✅ (was 🟡) | Push (modern_decks batch 29): the page-counter cost reduction is **now wired** via the new `ActivatedAbility.self_counter_cost_reduction: Option<CounterType>` field. The {5},{T} activation reads the source's Page counter pool at activation time and reduces the generic mana pip by one per counter (clamped at the printed generic total via `ManaCost::reduce_generic`). Page counters accrue 1 per instant/sorcery cast as before. Tests: `diary_of_dreams_activation_costs_five_with_no_page_counters`, `diary_of_dreams_page_counters_reduce_cost_by_one_each`, `diary_of_dreams_page_counters_clamp_at_printed_generic`. |
| Great Hall of the Biblioplex |  | Land |  | {T}: Add {C}. / {T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast an instant or sorcery spell. / {5}: If this land isn't a creature, it becomes a 2/4 Wizard creature with "Whenever you cast an instant or sorcery spell, this creature gets +1/+0 until end of turn." It's still a land. | 🟡 | Push XV: legendary colorless utility land. `{T}: Add {C}` faithful + `{T}, Pay 1 life: Add one mana of any color` via the new `ActivatedAbility.life_cost: u32` field — the effect is a pure mana ability (`AddMana(AnyOneColor 1)`) so it resolves immediately without going on the stack. The `{5}: becomes 2/4 Wizard creature` clause is omitted (no land-becomes-creature primitive — same gap as Mishra's Factory). The spend-restriction rider on the rainbow ability is omitted (no per-pip mana metadata yet). |
| Mage Tower Referee | {2} | Artifact Creature — Construct | 2/1 | Whenever you cast a multicolored spell, put a +1/+1 counter on this creature. | ✅ | Wired in `catalog::sets::sos::creatures` with a `SpellCast/YourControl` trigger filtered on `EntityMatches(TriggerSource, Multicolored)` — uses the new `SelectionRequirement::Multicolored` predicate (≥ 2 distinct colored pips, hybrid both halves, Phyrexian colored side). Mono-color and colorless casts don't bump the Referee. |
| Page, Loose Leaf | {2} | Legendary Artifact Creature — Construct | 0/2 | {T}: Add {C}. / Grandeur — Discard another card named Page, Loose Leaf: Reveal cards from the top of your library until you reveal an instant or sorcery card. Put that card into your hand and the rest on the bottom of your library in a random order. | 🟡 | Body wired (0/2 Legendary Construct artifact creature) + `{T}: Add {C}` mana ability via the shared `tap_add_colorless()` helper. Grandeur (discard-named-this-card-as-cost activation) omitted. |
| Petrified Hamlet |  | Land |  | When this land enters, choose a land card name. / Activated abilities of sources with the chosen name can't be activated unless they're mana abilities. / Lands with the chosen name have "{T}: Add {C}." / {T}: Add {C}. | 🟡 | Mana ability `{T}: Add {C}` wired via the new shared `tap_add_colorless()` helper in `catalog::sets`. The "choose a land card name" prompt + name-keyed lock-out static + name-keyed grant of `{T}: Add {C}` on opp lands are all omitted (no name-prompt decision, no name-match selector). The card still slots into colorless utility roles. |
| Potioner's Trove | {3} | Artifact |  | {T}: Add one mana of any color. / {T}: You gain 2 life. Activate only if you've cast an instant or sorcery spell this turn. | ✅ (was 🟡) | Push XXXVIII (doc-sync): both activations wired. The mana ability adds any one color; the lifegain activation gates on the new `Predicate::InstantsOrSorceriesCastThisTurnAtLeast { who: You, at_least: 1 }` (backed by `Player.instants_or_sorceries_cast_this_turn`). Test: `potioners_trove_lifegain_requires_is_cast_this_turn`. |
| Rancorous Archaic | {5} | Creature — Avatar | 2/2 | Trample, reach / Converge — This creature enters with a +1/+1 counter on it for each color of mana spent to cast it. | ✅ | Push (modern_decks): "enters with N counters" now uses the new `CardDefinition.enters_with_counters` field (CR 614.12) keyed off `Value::ConvergedValue` so the counters land before SBA / ETB exactly per printed Oracle. Was an ETB AddCounter trigger that fired after SBA — gameplay was fine for the 2/2 body but the timing was wrong relative to other ETB triggers / replacement effects. |
| Skycoach Waypoint |  | Land |  | {T}: Add {C}. / {3}, {T}: Target creature becomes prepared. (Only creatures with prepare spells can become prepared.) | 🟡 | Push XXV: `{T}: Add {C}` mana ability wired via `tap_add_colorless()`. The {3},{T} prepare-target ability is omitted — engine has no Prepare keyword (same gap as Biblioplex Tomekeeper). |
| Strixhaven Skycoach | {3} | Artifact — Vehicle | 3/2 | Flying / When this Vehicle enters, you may search your library for a basic land card, reveal it, put it into your hand, then shuffle. / Crew 2 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.) | 🟡 | Push XXVI: Body wired — 3/2 Vehicle artifact with Flying. ETB basic-land tutor-to-hand via `Effect::Search { filter: IsBasicLand, to: Hand(You) }`. Crew is not enforced (no crew-as-tap-cost primitive yet); the Skycoach stays a non-creature artifact until that lands. |
| Sundering Archaic | {6} | Creature — Avatar | 3/3 | Converge — When this creature enters, exile target nonland permanent an opponent controls with mana value less than or equal to the number of colors of mana spent to cast this creature. / {2}: Put target card from a graveyard on the bottom of its owner's library. | ✅ (was 🟡) | Push (modern_decks): the converge-scaled MV cap **is now wired** via `Effect::If { cond: ValueAtMost(ManaValueOf(Target(0)), ConvergedValue), then: Exile, else_: Noop }` — the trigger no-ops cleanly when the target's MV exceeds ConvergedValue. The `{2}: gy → bottom of owner's library` activation is unchanged. Tests: `sundering_archaic_etb_converge_cap_blocks_high_mv_target`, `sundering_archaic_two_mana_bottoms_graveyard_card`. |
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
| Augusta, Dean of Order | {2}{W} | ✅ (was 🟡) | Push (modern_decks batch 19): 2/3 Legendary Human Cleric. Per-attacker `Attacks/AnotherOfYours` trigger pumps the attacker +1/+1 EOT and grants Vigilance EOT — simplified stand-in for the printed "your choice of flying/first strike/vigilance/lifelink" rider (auto-picks Vigilance, the most generally useful keyword for chained attacks; the four-keyword choice is doc-tracked as an engine-wide keyword-mode-prompt gap shared with similar cards). The "three or more with same power" gate is omitted (no engine predicate for "attacking creatures with same power" — same gap as Coordinated Aggressor and the Battle Mammoth riders); the unconditional per-attacker fire is strictly better than the printed CR-correct version (the printed gate restricts when the buff applies, not who it applies to), so the engine ships an over-pump that captures every legal play pattern. Partner with Plargg, Dean of Chaos is omitted (no Partner-pair primitive). Tests: `augusta_dean_of_order_per_attacker_trigger_pumps_other_attacker`. |
| Silverquill Loremender | {1}{W} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): 2/2 Human Cleric. ETB gain 2 life. Standard Light-of-Promise enabler. Test: `silverquill_loremender_etb_gains_two_life`. |
| Inkling Verselord | {2}{W}{B} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): 3/3 Inkling Cleric Wizard, Flying. Static "Other Inkling creatures you control have lifelink" wired via `StaticEffect::GrantKeyword(applies_to: Other Inklings)`. Stacks with Tenured Inkcaster's +2/+2 anthem. Tests: `inkling_verselord_grants_lifelink_to_other_inklings`, `inkling_verselord_does_not_grant_lifelink_to_self`. |
| Silverquill Drainmaster | {2}{W}{B} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): 3/2 Vampire Warlock. ETB drain 3 (each opp loses 3, you gain 3). Test: `silverquill_drainmaster_etb_drains_three`. |
| Inkrise Lifedrainer | {1}{B} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): 2/1 Inkling Rogue, Menace. Combat-damage-to-player → gain 1 life trigger via `EventKind::DealsCombatDamageToPlayer`. Test: `inkrise_lifedrainer_combat_damage_gains_one_life`. |
| Silverquill Penman | {1}{W}{B} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): 2/2 Inkling Wizard, Flying. ETB MayDo: discard a card → draw + each opp loses 1 life. Test: `silverquill_penman_is_a_three_mana_inkling_wizard_flier`. |
| Silverquill Anthemwriter | {3}{W}{B} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): 4/4 Inkling Bard, Flying + Lifelink. Static "Other creatures you control get +1/+0" via `StaticEffect::PumpPT(OtherThanSource)`. Tests: `silverquill_anthemwriter_pumps_other_friendlies_by_one_zero`, `silverquill_anthemwriter_is_a_lifelink_flying_finisher`. |
| Silverquill Quillmage | {W}{B} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): 2/2 Human Wizard, Lifelink. Magecraft each opp loses 1 life. Test: `silverquill_quillmage_drains_on_instant_cast`. |
| Silverquill Memorialist | {2}{W} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): 2/3 Human Cleric. ETB returns target ≤2-MV creature card from your gy to hand via `Selector::one_of(CardsInZone(Graveyard, Creature & ManaValueAtMost(2)))`. Test: `silverquill_memorialist_etb_returns_low_mv_creature_from_graveyard`. |
| Inkling Aspirant | {W}{B} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): 2/1 Inkling Cleric, Flying. Vanilla Inkling 2-drop. Test: `inkling_aspirant_is_a_two_mana_inkling_flier`. |
| Witherspell Drain | {1}{W}{B} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): Instant. Drain 3 (each opp loses 3, you gain 3). Test: `witherspell_drain_drains_three_life`. |
| Inkling Scribe | {2}{W} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): 1/2 Inkling Cleric. ETB mints a 1/1 W/B Inkling flying token via the shared `inkling_token()` helper. Test: `inkling_scribe_etb_mints_an_inkling_token`. |
| Silverquill Erudite | {3}{W} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): 2/4 Human Wizard, Vigilance. Magecraft self-pump +1/+0 EOT. Test: `silverquill_erudite_self_pumps_on_instant_cast`. |
| Inkling Bloodscribe | {3}{W}{B} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): 3/3 Inkling Vampire, Lifelink. AnotherOfYours-dies trigger drains 1 — Cauldron-of-Essence template on a body. Test: `inkling_bloodscribe_is_a_five_mana_lifelink_vampire_inkling`. |
| Silverquill Reprimand | {2}{W} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): Sorcery. Exile target creature with power ≤ 2 via `Effect::Move → Exile` with `PowerAtMost(2)` target filter. Test: `silverquill_reprimand_exiles_two_power_creature`. |
| Silverquill Inquisition | {1}{B} | ✅ | Push (modern_decks batch 14, NEW, `stx::silverquill`): Sorcery. Target opp shows hand, you pick a nonland → discard via `Effect::DiscardChosen { from: Target(0), filter: Nonland }`. Test: `silverquill_inquisition_makes_opp_discard_a_card`. |
| Silverquill Archivist | {1}{W} | ✅ | Push (modern_decks batch 15, NEW, `stx::silverquill`): 1/2 Human Wizard. ETB Seq(Scry 1 + GainLife 1). Test: `silverquill_archivist_etb_scrys_and_gains_one_life`. |
| Silverquill Witness | {W}{B} | ✅ | Push (modern_decks batch 15, NEW, `stx::silverquill`): 2/1 Human Cleric Lifelink. Magecraft GainLife 1. Tests: `silverquill_witness_magecraft_gains_one_life_on_instant_cast`, `silverquill_witness_has_lifelink`. |
| Silverquill Judge | {2}{W} | ✅ | Push (modern_decks batch 15, NEW, `stx::silverquill`): 2/3 Human Cleric Vigilance. Magecraft Tap target opp creature. Test: `silverquill_judge_magecraft_taps_opponent_creature`. |
| Inkling Brigade | {3}{W}{B} | ✅ | Push (modern_decks batch 15, NEW, `stx::silverquill`): 3/3 Inkling Soldier Flying. ETB mints 2 Inkling tokens via `inkling_token()`. Tests: `inkling_brigade_etb_mints_two_inkling_tokens`, `inkling_brigade_is_a_five_mana_flying_inkling_soldier`. |
| Silverquill Pen-Pusher | {1}{B} | ✅ | Push (modern_decks batch 15, NEW, `stx::silverquill`): 1/1 Inkling Wizard Flying. Magecraft Scry 1. Test: `silverquill_pen_pusher_magecraft_scrys_one`. |
| Silverquill Chronicle | {3}{W}{B} | ✅ | Push (modern_decks batch 15, NEW, `stx::silverquill`): Sorcery. Seq(Drain 2 + Move target IS card from gy → hand) via `Selector::one_of(CardsInZone(Graveyard, Instant ∨ Sorcery))`. Test: `silverquill_chronicle_drains_two_and_returns_is_card_from_graveyard`. |
| Inkling Vanguard | {2}{W} | ✅ | Push (modern_decks batch 15, NEW, `stx::silverquill`): 2/3 Inkling Soldier Flying + Vigilance. Vanilla evasive Inkling on a 3-mana frame. Test: `inkling_vanguard_is_a_three_mana_flying_vigilance_inkling`. |
| Silverquill Marshal | {2}{W} | ✅ | Push (modern_decks batch 17, NEW, `stx::silverquill`): 2/3 Human Soldier. ETB gain 2 life. Bread-and-butter defensive body that feeds Light of Promise / Inkling Bloodscribe lifegain payoffs. Tests: `silverquill_marshal_etb_gains_two_life`, `silverquill_marshal_is_a_three_mana_two_three_soldier`. |
| Inkling Sanctifier | {2}{W} | ✅ | Push (modern_decks batch 17, NEW, `stx::silverquill`): 2/3 Inkling Cleric Flying+Lifelink. Hard-hitting 3-mana evasive lifelinker. Stacks with Tenured Inkcaster (+2/+2 → 4/5 Lifelink Flier). Test: `inkling_sanctifier_is_a_lifelink_flying_inkling`. |
| Silverquill Pupil | {W} | ✅ | Push (modern_decks batch 17, NEW, `stx::silverquill`): 1/2 Human Wizard. Magecraft +1/+0 EOT self-pump. Smaller cousin to Eager First-Year — scales aggressively in spell-heavy shells. Test: `silverquill_pupil_magecraft_pumps_self_plus_one_power`. |
| Defend the Inkwell | {2}{W}{B} | ✅ | Push (modern_decks batch 17, NEW, `stx::silverquill`): Sorcery. Seq(Drain 2 + Scry 2). Fills the Silverquill drain + card selection slot, feeds Witherbloom Apprentice / Honor Troll lifegain triggers. Test: `defend_the_inkwell_drains_two_and_scrys_two`. |
| Inkling Witness | {W}{B} | ✅ | Push (modern_decks batch 17, NEW, `stx::silverquill`): 2/2 Inkling Cleric Flying. Per-Inkling-death trigger via `CreatureDied/AnotherOfYours + HasCreatureType(Inkling)`. Pairs with Felisa / Inkling Summoning for chained lifegain. Tests: `inkling_witness_gains_life_when_other_inkling_dies`, `inkling_witness_is_a_two_mana_flying_inkling`. |
| Inkling Coursebinder | {1}{W}{B} | ✅ | Push (modern_decks batch 18, NEW, `stx::silverquill`): 2/2 Inkling Wizard, Flying. Magecraft drain 1 (each opp loses 1; you gain 1). Same shape as Promising Duskmage. Test: `inkling_coursebinder_drains_on_instant_cast`. |
| Silverquill Sermon | {2}{W}{B} | ✅ | Push (modern_decks batch 18, NEW, `stx::silverquill`): Sorcery. Creates 2 Inkling tokens via `Effect::CreateToken { count: 2, definition: inkling_token() }`. Same shape as Defend the Campus at a lower cost (4 vs 5 mana) for 2 tokens instead of 3. Test: `silverquill_sermon_mints_two_inkling_tokens`. |
| Silverquill Censure | {1}{W} | ✅ | Push (modern_decks batch 18, NEW, `stx::silverquill`): Instant. Seq(Move(target creature with power ≤ 3) → Exile + GainLife 2). Clean 2-mana exile-removal at the small-creature slot. Stronger than Silverquill Reprimand at the same slot since exile dodges Persist / Undying / gy-recursion shells. Test: `silverquill_censure_exiles_low_power_creature_and_gains_life`. |
| Silverquill Castigant | {2}{W} | ✅ | Push (modern_decks batch 19, NEW, `stx::silverquill`): 2/3 Human Cleric. ETB drain 1 (each opp loses 1, you gain 1). Compact 3-mana defensive drain body that feeds Light of Promise / Felisa lifegain payoffs. Test: `silverquill_castigant_etb_drains_one`. |
| Silverquill Heartrender | {2}{B} | ✅ | Push (modern_decks batch 19, NEW, `stx::silverquill`): Sorcery. Seq(Drain 3 + Scry 1). Strict-upgrade over Sign in Blood's mana fork (drain a creature payoff vs draw-2-lose-2) trading the cards for the swing. Test: `silverquill_heartrender_drains_three_and_scrys_one`. |
| Inkling Confessor | {1}{W}{B} | ✅ | Push (modern_decks batch 19, NEW, `stx::silverquill`): 2/2 Inkling Cleric Flying. Magecraft drain 1 — same shape as Inkling Coursebinder. Stacks with Tenured Inkcaster's anthem (+2/+2 → 4/4 flying drain) and Inkling Verselord's lifelink grant. Test: `inkling_confessor_magecraft_drains_on_instant_cast`. |
| Inkling Inkrider | {2}{W}{B} | ✅ | Push (modern_decks batch 19, NEW, `stx::silverquill`): 3/2 Inkling Knight Flying + Vigilance. Aggressive 4-mana evasive Inkling — same P/T as Inkling Sanctifier but trades lifelink for vigilance. Test: `inkling_inkrider_is_a_four_mana_flying_vigilance_inkling_knight`. |
| Silverquill Quillblade | {W} | ✅ | Push (modern_decks batch 19+, NEW, `stx::silverquill`): Instant. Target creature you control gets +X/+0 EOT where X = creatures you control (via `Value::CountOf`). 1-mana board-scaled combat trick. Test: `silverquill_quillblade_pumps_by_creature_count`. |
| Inkling Decree | {3}{W}{B} | ✅ | Push (modern_decks batch 19+, NEW, `stx::silverquill`): Sorcery. Seq(Drain 2 + CreateToken(1 Inkling)). 5-mana drain-and-token combo (4-life swing + 1/1 evasive body). Test: `inkling_decree_drains_two_and_mints_inkling`. |
| Silverquill Lawkeeper | {1}{W} | ✅ | Push (modern_decks batch 20, NEW, `stx::silverquill`): 2/2 Human Soldier Vigilance. ETB Tap target opp creature via `target_filtered(Creature ∧ ControlledByOpponent)`. Tempo defender + lockdown body. Test: `silverquill_lawkeeper_etb_taps_opp_creature`. |
| Inkling Penmaster | {2}{W}{B} | ✅ | Push (modern_decks batch 20, NEW, `stx::silverquill`): 2/3 Inkling Wizard Flying. Magecraft mints a 1/1 W/B Inkling flying token. Tenured Inkcaster engine — every spell + buff. Test: `inkling_penmaster_mints_inkling_on_instant_cast`. |
| Silverquill Dictation | {1}{W}{B} | ✅ | Push (modern_decks batch 20, NEW, `stx::silverquill`): Instant. Seq(LoseLife 2 target player + Draw 1). Targets either player → opp drain or self-draw-while-paying-life. Test: `silverquill_dictation_drains_two_and_draws`. |
| Inkling Stormcaller | {3}{W}{B} | ✅ | Push (modern_decks batch 20, NEW, `stx::silverquill`): 3/4 Inkling Cleric Flying + Lifelink. ETB Drain 2 (4-life swing). Race breaker top-end. Test: `inkling_stormcaller_etb_drains_two_and_is_flying_lifelink`. |
| Silverquill Discipline | {W} | ✅ | Push (modern_decks batch 20, NEW, `stx::silverquill`): Instant. Seq(PumpPT(+2/+1 EOT) + GrantKeyword(Lifelink, EOT)). 1-mana combat trick + lifelink-on-the-buffed-creature. Test: `silverquill_discipline_pumps_and_grants_lifelink`. |
| Silverquill Conviction | {W}{B} | ✅ | Push (modern_decks batch 22, NEW, `stx::silverquill`): Sorcery. Seq(Drain 2 + Surveil 1). 2-mana drain + card selection — standard Witherbloom apprentice tax + peek. Test: `silverquill_conviction_drains_two_and_surveils`. |
| Silverquill Bookbearer | {2}{W} | ✅ | Push (modern_decks batch 22, NEW, `stx::silverquill`): 1/4 Human Cleric Vigilance. ETB Scry 2. Defender + draw smoothing. Test: `silverquill_bookbearer_etb_scrys_and_has_vigilance`. |
| Inkling Inquisitor | {2}{B} | ✅ | Push (modern_decks batch 22, NEW, `stx::silverquill`): 2/3 Inkling Rogue Flying. ETB DiscardChosen against target opp (nonland filter — Inquisition of Kozilek template). Test: `inkling_inquisitor_etb_makes_opp_discard_chosen`. |
| Silverquill Reckoning | {3}{W}{B} | ✅ | Push (modern_decks batch 22, NEW, `stx::silverquill`): Sorcery. Seq(Destroy(target creature) + CreateToken(1 Inkling)). 5-mana hard removal + body. Test: `silverquill_reckoning_destroys_creature_and_mints_inkling`. |
| Silverquill Lifeglyph | {1}{W}{B} | ✅ | Push (modern_decks batch 22, NEW, `stx::silverquill`): 2/3 Inkling Bard Lifelink. Magecraft → +1/+1 EOT to target creature via `magecraft_target_pump(target(Creature), 1, 1)`. Test: `silverquill_lifeglyph_pumps_target_on_instant_cast`. |
| Inkling Aristocrat | {1}{B} | ✅ | Push (modern_decks batch 23, NEW, `stx::silverquill`): 1/2 Inkling Cleric. `CreatureDied/AnotherOfYours` trigger gains 1 life. Aristocrat payoff at 2 mana. Tests: `inkling_aristocrat_gains_life_when_another_creature_dies`, `inkling_aristocrat_does_not_trigger_on_self`. |
| Silverquill Quillscribe | {2}{W}{B} | ✅ | Push (modern_decks batch 23, NEW, `stx::silverquill`): 3/3 Human Wizard. ETB mint 1 Inkling token + magecraft +1/+1 counter on target friendly Inkling. Inkling engine that grows itself. Test: `silverquill_quillscribe_etb_mints_inkling_and_pumps_on_cast`. |
| Silverquill Hush | {W}{B} | ✅ | Push (modern_decks batch 23, NEW, `stx::silverquill`): Instant. Seq(PumpPT -2/-2 EOT + GainLife 2). 2-mana removal-for-2-toughness + defensive lifegain. Test: `silverquill_hush_shrinks_creature_and_gains_life`. |
| Inkling Lorewright | {3}{W}{B} | ✅ | Push (modern_decks batch 23, NEW, `stx::silverquill`): 2/4 Inkling Wizard Flying. ETB Seq(Draw 1 + LoseLife 1). 5-mana defensive flyer + cantrip. Test: `inkling_lorewright_etb_draws_and_loses_one_life`. |
| Silverquill Battle Hymn | {2}{W} | ✅ | Push (modern_decks batch 23, NEW, `stx::silverquill`): Sorcery. Seq(PumpPT(each_your_creature, +1/+1, EOT) + GrantKeyword(Vigilance, EOT)). Team anthem with vigilance for the alpha-strike-then-block turn. Test: `silverquill_battle_hymn_pumps_team_with_vigilance`. |
| Inkling Sage | {1}{W} | ✅ | Push (modern_decks batch 23 extras, NEW, `stx::silverquill`): 1/2 Inkling Wizard Flying. Activated `{2}{W}{B}: +1/+1 EOT` mana sink. Test: `inkling_sage_pump_activation_makes_two_two_flier`. |
| Silverquill Memorist | {2}{W}{B} | ✅ | Push (modern_decks batch 24++, NEW, `stx::silverquill`): 2/3 Inkling Bard Flying. ETB returns target IS card from your gy → hand. Test: `silverquill_memorist_etb_returns_is_card_from_graveyard`. |
| Silverquill Eulogist | {1}{B} | ✅ | Push (modern_decks batch 24+, NEW, `stx::silverquill`): 1/3 Human Cleric. Magecraft drains each opp for 1. Test: `silverquill_eulogist_drains_each_opp_on_cast`. |
| Inkling Quillwarden | {2}{W}{B} | ✅ | Push (modern_decks batch 24+, NEW, `stx::silverquill`): 2/4 Inkling Knight Flying + Vigilance. Magecraft self-pump +1/+0 EOT. Test: `inkling_quillwarden_magecraft_self_pumps`. |
| Silverquill Notetaker | {1}{W} | ✅ | Push (modern_decks batch 24, NEW, `stx::silverquill`): 1/2 Human Wizard. ETB Scry 1 + magecraft MayDo Draw 1. Test: `silverquill_notetaker_etb_scrys_one`. |
| Inkling Pamphleteer | {W}{B} | ✅ | Push (modern_decks batch 24, NEW, `stx::silverquill`): 2/2 Inkling Cleric Flying. ETB drain 1. Test: `inkling_pamphleteer_etb_drains_one`. |
| Silverquill Indictment | {2}{W}{B} | ✅ | Push (modern_decks batch 24, NEW, `stx::silverquill`): Instant. Seq(Move(target Creature ∧ MV≤3 → Exile) + GainLife 2). 4-mana clean exile-removal for the small-creature slot + lifegain rider. Test: `silverquill_indictment_exiles_low_mv_creature`. |
| Inkling Banner-Bearer | {3}{W} | ✅ | Push (modern_decks batch 24, NEW, `stx::silverquill`): 2/3 Inkling Soldier Flying + Vigilance. Static "Other Inkling creatures you control get +1/+0" via `StaticEffect::PumpPT` + `OtherThanSource`. Stacks with Tenured Inkcaster. Test: `inkling_banner_bearer_buffs_other_inklings`. |
| Silverquill Tribunal | {2}{B} | ✅ | Push (modern_decks batch 24, NEW, `stx::silverquill`): Sorcery. Seq(target opp sacrifices a creature + GainLife 1). Edict-with-lifegain. Test: `silverquill_tribunal_forces_opp_sacrifice_and_gains_one_life`. |

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
| Witherbloom Pest-Tender | {1}{B} | ✅ | Push (modern_decks batch 15, NEW, `stx::witherbloom`): 1/2 Plant Druid. ETB mints 1 Pest token (via shared `stx_pest_token()`). Test: `witherbloom_pest_tender_etb_mints_a_pest_token`. |
| Pest Swarmer | {2}{B}{G} | ✅ | Push (modern_decks batch 15, NEW, `stx::witherbloom`): 2/2 Pest Warrior. On-die trigger creates 1 Pest token — self-replacing body. Test: `pest_swarmer_dies_mints_a_replacement_pest`. |
| Witherbloom Seer | {1}{B}{G} | ✅ | Push (modern_decks batch 15, NEW, `stx::witherbloom`): 2/2 Human Druid Deathtouch. Magecraft drain 1 via the `magecraft_drain_each_opp(1)` helper. Tests: `witherbloom_seer_drains_each_opp_on_instant_cast`, `witherbloom_seer_has_deathtouch`. |
| Pest Swarm | {3}{B}{G} | ✅ | Push (modern_decks batch 15, NEW, `stx::witherbloom`): Sorcery. Creates 3 Pest tokens via `Effect::CreateToken { count: 3 }`. Each Pest's death-trigger lifegain rides via the shared `stx_pest_token()`. Test: `pest_swarm_creates_three_pest_tokens`. |
| Witherbloom Vinemaster | {3}{B}{G} | ✅ | Push (modern_decks batch 15, NEW, `stx::witherbloom`): 3/4 Plant Druid Trample. On `CreatureDied/AnotherOfYours` filtered by `HasCreatureType(Pest)` → +1/+1 counter on self (same Pestmaster template). Test: `witherbloom_vinemaster_grows_on_pest_death`. |
| Witherbloom Mossfeeder | {2}{B}{G} | ✅ | Push (modern_decks batch 17, NEW, `stx::witherbloom`): 3/3 Plant Beast. ETB mints 1 Pest token. Mid-curve curve-top Pest enabler. Test: `witherbloom_mossfeeder_etb_mints_pest_token`. |
| Witherbloom Reverie | {1}{B}{G} | ✅ | Push (modern_decks batch 17, NEW, `stx::witherbloom`): Sorcery. Drain 3 (each opp loses 3, you gain 3). Pure {B}{G} drain at the 3-mana slot. Test: `witherbloom_reverie_drains_three`. |
| Pest Cultivator | {1}{B}{G} | ✅ | Push (modern_decks batch 17, NEW, `stx::witherbloom`): 2/2 Plant Druid. ETB mints 2 Pests. 3-mana Pest fan-out + sticky body. Test: `pest_cultivator_etb_mints_two_pests`. |
| Withergrowth Apprentice | {B}{G} | ✅ | Push (modern_decks batch 17, NEW, `stx::witherbloom`): 1/3 Human Druid. Magecraft +1/+1 EOT on friendly creature. Defensive WB Apprentice — mirror of Eager First-Year. Test: `withergrowth_apprentice_magecraft_pumps_friendly_creature`. |
| Witherbloom Pestkeeper | {2}{B} | ✅ | Push (modern_decks batch 18, NEW, `stx::witherbloom`): 2/3 Plant Cleric. ETB mints a Pest token. `{1}{B}{G}, Sacrifice a Pest: Target creature gets -2/-2 EOT` — uses an `Effect::Sacrifice` cost-step filtered on `HasCreatureType(Pest)`. Pairs with Pestmancer / Pest Cultivator for chained sac-removal. Test: `witherbloom_pestkeeper_etb_mints_pest_and_sac_shrinks_target`. |
| Witherbloom Bonepicker | {1}{B}{G} | ✅ | Push (modern_decks batch 18, NEW, `stx::witherbloom`): 3/3 Plant Skeleton with Trample. ETB drain 2 (each opp loses 2). Headline 3-mana curve-out for Witherbloom. Tests: `witherbloom_bonepicker_etb_drains_each_opp_two`, `witherbloom_bonepicker_is_a_three_mana_three_three_trampler`. |
| Pest Bequest | {3}{B}{G} | ✅ | Push (modern_decks batch 18, NEW, `stx::witherbloom`, factory `pest_swarm_inheritance`): Sorcery. Seq(PumpPT(+1/+1, EOT) + GrantKeyword(Deathtouch, EOT) + CreateToken(1 Pest)). Renamed factory to avoid collision with the existing `pest_inheritance` Lesson. Test: `pest_swarm_inheritance_pumps_friendly_and_mints_pest`. |
| Witherbloom Decayblossom | {1}{B} | ✅ | Push (modern_decks batch 18, NEW, `stx::witherbloom`): 1/1 Plant Cleric. On `CreatureDied/SelfSource` → target creature gets -1/-1 EOT. Pestkeeper-fodder + targeted sized debuff. Test: `witherbloom_decayblossom_dies_shrinks_target`. |
| Witherbloom Recourse | {1}{B}{G} | ✅ | Push (modern_decks batch 18, NEW, `stx::witherbloom`): Instant. Seq(return target ≤2-MV creature from your gy → hand + drain 1). Same gy-recursion shape as Silverquill Memorialist but at instant speed and with a drain rider. Test: `witherbloom_recourse_returns_low_mv_creature_and_drains`. |
| Witherbloom Pestmancer | {2}{B}{G} | ✅ | Push (modern_decks batch 18, NEW, `stx::witherbloom`): 2/2 Human Warlock. Magecraft → mint a Pest token. Same shape as Sedgemoor Witch but at the {B}{G} slot. Pest death-trigger lifegain stacks with magecraft drain in spell-heavy boards. Test: `witherbloom_pestmancer_mints_pest_on_instant_cast`. |
| Witherbloom Lifebleeder | {1}{B}{G} | ✅ | Push (modern_decks batch 19, NEW, `stx::witherbloom`): 2/2 Human Warlock. Magecraft drain 1 — Witherbloom Apprentice on a 3-mana frame. Slots into the 3-CMC drain slot for slower decks. Test: `witherbloom_lifebleeder_drains_on_instant_cast`. |
| Pest Marauder | {1}{B} | ✅ | Push (modern_decks batch 19, NEW, `stx::witherbloom`): 1/1 Pest with Deathtouch. On-die trigger gains you 1 life (mirroring the Pest token's printed shape). 2-mana deathtouch trade body. Test: `pest_marauder_has_deathtouch_and_dies_grants_life`. |
| Witherbloom Decoctor | {3}{B}{G} | ✅ | Push (modern_decks batch 19, NEW, `stx::witherbloom`): 3/4 Human Druid. ETB drain 2 (each opp loses 2, you gain 2). 5-mana 3/4 frame with built-in 4-life swing on ETB. Test: `witherbloom_decoctor_etb_drains_two`. |
| Witherbloom Sapfiend | {2}{G} | ✅ | Push (modern_decks batch 19, NEW, `stx::witherbloom`): 2/3 Plant Beast. Magecraft +1/+1 EOT self-pump. Mirror of Eager First-Year on a defensive 3-mana green frame. Test: `witherbloom_sapfiend_self_pumps_on_instant_cast`. |
| Witherbloom Glimmer | {2}{B}{G} | ✅ | Push (modern_decks batch 19+, NEW, `stx::witherbloom`): 3/3 Plant Druid Lifelink. Vanilla 4-mana lifelink body — same P/T as Mossfeeder but trades the Pest ETB for lifelink. Test: `witherbloom_glimmer_is_a_three_three_lifelink_plant`. |
| Pest Communion | {1}{B}{G} | ✅ | Push (modern_decks batch 19+, NEW, `stx::witherbloom`): Sorcery. Seq(Mill 4 from each opponent + Drain 1). 3-mana mill-and-drain combo with broad gy-fill upside for delirium-style payoffs. Test: `pest_communion_mills_four_each_opp_and_drains_one`. |
| Witherbloom Toxicultivator | {2}{B} | ✅ | Push (modern_decks batch 20, NEW, `stx::witherbloom`): 2/3 Plant Druid Deathtouch. ETB mints 1 Pest token. Deathtouch defender + Pest sac engine seed. Test: `witherbloom_toxicultivator_etb_mints_pest_and_has_deathtouch`. |
| Pest Outburst | {2}{B}{G} | ✅ | Push (modern_decks batch 20, NEW, `stx::witherbloom`): Sorcery. Seq(CreateToken(2 Pests) + GainLife 2). 4-mana double Pest minter + immediate lifegain. Vinemaster engine fuel. Test: `pest_outburst_mints_two_pests_and_gains_two`. |
| Witherbloom Grand Necromancer | {3}{B}{G} | ✅ | Push (modern_decks batch 20, NEW, `stx::witherbloom`): 3/3 Human Warlock. ETB returns target creature card from your gy → hand + magecraft drain 1. Grindy value top-end. Test: `witherbloom_grand_necromancer_returns_creature_from_gy`. |
| Witherbloom Sapdrinker | {1}{B}{G} | ✅ | Push (modern_decks batch 20, NEW, `stx::witherbloom`): 2/3 Plant Vampire Lifelink. Magecraft +1/+0 self-pump EOT. Lifelink-via-power scaling — strong finisher in spell-heavy shells. Test: `witherbloom_sapdrinker_self_pumps_and_has_lifelink`. |
| Witherbloom Crawler | {B}{G} | ✅ | Push (modern_decks batch 20, NEW, `stx::witherbloom`): 2/2 Plant Insect Deathtouch + Reach. Vanilla 2-mana anti-flier + ground deathtouch trade. Test: `witherbloom_crawler_is_two_two_deathtouch_reach`. |
| Pest Swarmlord | {3}{B}{G} | ✅ | Push (modern_decks batch 22, NEW, `stx::witherbloom`): 3/3 Pest Warlock. ETB mints 2 Pest tokens. 5-mana go-wide finisher. Test: `pest_swarmlord_etb_mints_two_pests`. |
| Witherbloom Vinetender | {1}{G} | ✅ | Push (modern_decks batch 22, NEW, `stx::witherbloom`): 2/2 Plant Druid Reach. Magecraft → gain 1 life. 2-mana anti-flier + lifegain engine. Test: `witherbloom_vinetender_magecraft_gains_one_life`. |
| Toxic Bloodletting | {1}{B}{G} | ✅ | Push (modern_decks batch 22, NEW, `stx::witherbloom`): Instant. Seq(PumpPT(-2/-2 EOT) + GainLife 2). 3-mana soft-removal + lifegain. Test: `toxic_bloodletting_minus_two_kills_bear_and_grants_life`. |
| Witherbloom Saproot | {2}{B}{G} | ✅ | Push (modern_decks batch 22, NEW, `stx::witherbloom`): 3/3 Plant Beast Trample. CreatureDied/SelfSource trigger drains 2. 4-mana trampler with baked-in death drain. Test: `witherbloom_saproot_dies_drains_each_opp`. |
| Pest Mausoleum | {2}{B}{G} | ✅ | Push (modern_decks batch 22, NEW, `stx::witherbloom`): Sorcery. Seq(Move(one_of(Graveyard, Creature) → Hand) + CreateToken Pest). 4-mana reanimation + token. Test: `pest_mausoleum_returns_creature_and_mints_pest`. |
| Pest Ravager | {3}{B}{G} | ✅ | Push (modern_decks batch 23, NEW, `stx::witherbloom`): 4/4 Plant Beast Trample. ETB mints 2 Pest tokens (via shared `stx_pest_token()`). 5-mana go-wide trampler. Test: `pest_ravager_etb_mints_two_pests`. |
| Witherbloom Famine | {3}{B} | ✅ | Push (modern_decks batch 23, NEW, `stx::witherbloom`): Sorcery. Drain 4 (each opp loses 4, you gain 4) — 8-life swing finisher. Test: `witherbloom_famine_drains_four`. |
| Witherbloom Greenrot | {1}{G} | ✅ | Push (modern_decks batch 23, NEW, `stx::witherbloom`): 2/2 Plant Druid Reach. ETB gain 2 life. Anti-flier + lifegain. Test: `witherbloom_greenrot_etb_gains_two_life`. |
| Witherbloom Pestbroker | {2}{B} | ✅ | Push (modern_decks batch 23, NEW, `stx::witherbloom`): 2/3 Human Warlock. ETB drain 2 + `{1}{B}, Sac a Pest: -1/-1 EOT` activated sink. Test: `witherbloom_pestbroker_etb_drains_two`. |
| Pestilent Bloom | {B}{G} | ✅ | Push (modern_decks batch 23, NEW, `stx::witherbloom`): Instant. Seq(PumpPT(-3/-3 EOT) + CreateToken Pest). 2-mana shrink-removal + body. Test: `pestilent_bloom_shrinks_creature_and_mints_pest`. |
| Witherbloom Reaper-Hand | {2}{B}{G} | ✅ | Push (modern_decks batch 23 extras, NEW, `stx::witherbloom`): 3/3 Plant Warlock Deathtouch. CreatureDied/SelfSource trigger drains 2 (each opp loses 2, you gain 2). 4-mana deathtouch + death drain. Test: `witherbloom_reaper_hand_dies_drains_two`. |
| Witherbloom Tendril | {1}{B}{G} | ✅ | Push (modern_decks batch 24++, NEW, `stx::witherbloom`): Instant. Seq(Drain 2 + Draw 1). 3-mana drain + cantrip. Test: `witherbloom_tendril_drains_two_and_cantrips`. |
| Witherbloom Pest-Lord | {3}{B}{G} | ✅ | Push (modern_decks batch 24+, NEW, `stx::witherbloom`): 3/3 Plant Warlock. ETB mints a Pest token + static "Pest creatures you control get +1/+0". 5-mana Pest tribal lord. Test: `witherbloom_pest_lord_etb_mints_pest_and_pumps_pests`. |
| Witherbloom Drainbreath | {1}{B} | ✅ | Push (modern_decks batch 24+, NEW, `stx::witherbloom`): 2/1 Plant Warlock. Dies-trigger Drain 2 (4-life swing). Test: `witherbloom_drainbreath_dies_drains_two`. |
| Witherbloom Aspersor | {B}{G} | ✅ | Push (modern_decks batch 24, NEW, `stx::witherbloom`): Instant. Seq(PumpPT -2/-1 EOT + GainLife 1). 2-mana cheap shrink-removal + small lifegain. Test: `witherbloom_aspersor_shrinks_creature_and_gains_one_life`. |
| Pest Reanimator | {2}{B}{G} | ✅ | Push (modern_decks batch 24, NEW, `stx::witherbloom`): 3/2 Plant Warlock. ETB returns target ≤3-MV creature card from your gy → hand. 4-mana reanimator engine. Test: `pest_reanimator_etb_returns_creature_from_graveyard`. |
| Witherbloom Spore-Master | {3}{B}{G} | ✅ | Push (modern_decks batch 24, NEW, `stx::witherbloom`): 4/4 Plant Druid. ETB mints 2 Pest tokens. 5-mana go-wide finisher (8 power across 3 bodies). Test: `witherbloom_spore_master_etb_mints_two_pests`. |
| Witherbloom Withercut | {1}{B}{G} | ✅ | Push (modern_decks batch 24, NEW, `stx::witherbloom`): Instant. Seq(PumpPT -3/-1 EOT + Draw 1). 3-mana shrink-and-cantrip. Test: `witherbloom_withercut_shrinks_creature_and_cantrips`. |
| Pest Cultivator-Adept | {2}{B}{G} | ✅ | Push (modern_decks batch 24, NEW, `stx::witherbloom`): 2/3 Plant Druid. ETB mints a Pest + magecraft permanent +1/+1 counter on self. 4-mana Pest engine + counter-builder. Test: `pest_cultivator_adept_etb_mints_pest_and_grows_on_cast`. |

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
| Lorehold Acolyte | {1}{W} | ✅ | Push (modern_decks batch 15, NEW, `stx::lorehold`): 1/3 Human Cleric. ETB Move target gy card → Exile via `target_filtered(Any)` (engine target-picker walks all zones, same as Ascendant Dustspeaker). Test: `lorehold_acolyte_etb_exiles_target_graveyard_card`. |
| Lorehold Warrior-Priest | {R}{W} | ✅ | Push (modern_decks batch 15, NEW, `stx::lorehold`): 2/2 Spirit Cleric. Two triggers: `Attacks/SelfSource → GainLife 1`; `CardLeftGraveyard/YourControl → AddCounter(+1/+1, self)`. Tests: `lorehold_warrior_priest_gains_life_on_attack`, `lorehold_warrior_priest_is_a_two_mana_spirit_cleric`. |
| Lorehold Ember-Priest | {2}{R} | ✅ | Push (modern_decks batch 15, NEW, `stx::lorehold`): 2/3 Spirit Wizard. Magecraft 1 damage to any target via `target_filtered(Creature ∨ Player ∨ Planeswalker)`. Test: `lorehold_ember_priest_magecraft_pings_target`. |
| Lorehold Skirmish | {1}{R}{W} | ✅ | Push (modern_decks batch 15, NEW, `stx::lorehold`): Sorcery. Seq(CreateToken(1, lorehold_spirit_token()) + GrantKeyword(Selector::LastCreatedToken, Haste, EOT)). The minted 2/2 R/W Spirit gets haste EOT — same shape as Sparring Regimen's ETB token at instant tempo. Test: `lorehold_skirmish_mints_a_spirit_with_haste_eot`. |
| Lorehold Pyrosage | {1}{R}{W} | ✅ | Push (modern_decks batch 17, NEW, `stx::lorehold`): 2/2 Spirit Wizard. Magecraft pings each opp for 1. Mirror of Lorehold Burnscholar at the 3-mana slot. Test: `lorehold_pyrosage_magecraft_pings_each_opp`. |
| Lorehold Loremaster | {3}{R}{W} | ✅ | Push (modern_decks batch 17, NEW, `stx::lorehold`): 4/4 First Strike Spirit Wizard. Per-attack `Attacks/SelfSource → CreateToken(1 Spirit)`. Top-end Lorehold token engine. Tests: `lorehold_loremaster_attack_mints_spirit_token`, `lorehold_loremaster_has_first_strike`. |
| Lorehold Aerospirit | {2}{R}{W} | ✅ | Push (modern_decks batch 17, NEW, `stx::lorehold`): 3/2 Spirit Soldier Flying+Haste. Aerial finisher that ignores Spirit-haste anthems (it has them natively). Test: `lorehold_aerospirit_has_flying_and_haste`. |
| Lorehold Ember-Forge | {2}{R}{W} | ✅ | Push (modern_decks batch 17, NEW, `stx::lorehold`): Sorcery. Seq(DealDamage(3, target creature) + DealDamage(1, each opp)). 4-mana 3-damage with a 1-life-each-opp tail. Test: `lorehold_ember_forge_burns_creature_and_pings_each_opp`. |
| Lorehold Spiritcaller | {2}{R}{W} | ✅ | Push (modern_decks batch 18, NEW, `stx::lorehold`): 2/2 Human Cleric. ETB mints a 2/2 R/W Spirit token + per-`CardLeftGraveyard/YourControl` → gain 1 life. Same per-leave trigger as Ark of Hunger but with lifegain instead of drain. Test: `lorehold_spiritcaller_etb_mints_spirit_token`. |
| Lorehold Pyrebrand | {1}{R}{W} | ✅ | Push (modern_decks batch 18, NEW, `stx::lorehold`): 2/3 Spirit Warrior, First Strike. Magecraft +1/+0 self-pump EOT. Same shape as Spectacle Mage but with magecraft trigger and Spirit subtype synergy. Test: `lorehold_pyrebrand_magecraft_self_pumps`. |
| Lorehold Reclamation | {2}{R}{W} | ✅ | Push (modern_decks batch 18, NEW, `stx::lorehold`): Sorcery. Return target creature card from your graveyard to the battlefield via `Effect::Move → Battlefield`. The "it's a Spirit in addition" rider is omitted (no type-add-on-zone-change primitive). Test: `lorehold_reclamation_returns_creature_to_battlefield`. |
| Lorehold Reverberator | {3}{R} | ✅ | Push (modern_decks batch 18, NEW, `stx::lorehold`): 3/2 Spirit Wizard, Haste. Magecraft 2 damage to any target. Same shape as Lorehold Ember-Priest but bigger body + Haste at 4 mana. Tests: `lorehold_reverberator_magecraft_pings_target`, `lorehold_reverberator_has_haste`. |
| Lorehold Pyrescribe | {2}{R}{W} | ✅ | Push (modern_decks batch 19, NEW, `stx::lorehold`): 3/2 Spirit Wizard. Magecraft 1 damage to each opponent. Lorehold's drain-burn template — stacks with Galvanic Iteration and Twinscroll Shaman for doubled triggers. Test: `lorehold_pyrescribe_magecraft_pings_each_opp`. |
| Lorehold Echoist | {1}{R} | ✅ | Push (modern_decks batch 19, NEW, `stx::lorehold`): 1/2 Spirit Wizard. ETB mints 1 Spirit token via shared `lorehold_spirit_token()`. Net 3/4 over two bodies for {1}{R}. Test: `lorehold_echoist_etb_mints_spirit_token`. |
| Lorehold Spiritmaster | {3}{R}{W} | ✅ | Push (modern_decks batch 19, NEW, `stx::lorehold`): 3/3 Spirit Cleric. ETB mints 2 Spirit tokens. 5-mana 7/7 across three bodies — pairs with Quintorius Field Historian for instant tribal pressure. Test: `lorehold_spiritmaster_etb_mints_two_spirit_tokens`. |
| Lorehold Bonepriest | {1}{R}{W} | ✅ | Push (modern_decks batch 19, NEW, `stx::lorehold`): 2/2 Spirit Cleric. Magecraft → permanent +1/+1 counter on self. Snowballs hard in spell-heavy shells (vs Pyrebrand's EOT-only pump). Test: `lorehold_bonepriest_grows_on_each_instant_cast`. |
| Lorehold Recollect | {1}{R}{W} | ✅ | Push (modern_decks batch 19+, NEW, `stx::lorehold`): Sorcery. Reanimate target creature OR artifact from your gy via `Selector::one_of(CardsInZone(Graveyard, Creature ∨ Artifact))`. 3-mana broader Reclamation. Test: `lorehold_recollect_returns_creature_from_graveyard`. |
| Lorehold Anthemist | {2}{R}{W} | ✅ | Push (modern_decks batch 19+, NEW, `stx::lorehold`): 2/2 Spirit Cleric. Spirit-tribal anthem (+1/+1 to Other Spirits) wired via `StaticEffect::PumpPT` + `SelectionRequirement::OtherThanSource` (Quintorius pattern). Tests: `lorehold_anthemist_anthem_buffs_other_spirits`. |
| Lorehold Battlescroll | {3}{R}{W} | ✅ | Push (modern_decks batch 20, NEW, `stx::lorehold`): Sorcery. Seq(CreateToken(2 Spirits) + GrantKeyword(EachPermanent(Spirit ∧ ControlledByYou), Haste, EOT)). 5-mana hasty double Spirit minter. Test: `lorehold_battlescroll_mints_two_spirits_with_haste`. |
| Lorehold Tomescholar | {2}{R}{W} | ✅ | Push (modern_decks batch 20, NEW, `stx::lorehold`): 2/3 Spirit Wizard. ETB Seq(Move(Target → Exile) + If(Target HasCardType Creature, CreateToken(Spirit))). Conditional Spirit-mint on creature-card exile. Tests: `lorehold_tomescholar_mints_spirit_when_exiling_creature_card`, `lorehold_tomescholar_no_spirit_when_exiling_noncreature`. |
| Lorehold Ember-Brand | {1}{R} | ✅ | Push (modern_decks batch 20, NEW, `stx::lorehold`): Instant. 3 damage to any target. Lightning-Bolt template at the {1}{R} slot. Test: `lorehold_ember_brand_deals_three_to_player`. |
| Lorehold Spectrescribe | {1}{W} | ✅ | Push (modern_decks batch 20, NEW, `stx::lorehold`): 1/3 Spirit Cleric. Magecraft gain 1 life. Defensive lifegain-on-cast body. Test: `lorehold_spectrescribe_magecraft_gains_one_life`. |
| Lorehold Warband | {2}{R} | ✅ | Push (modern_decks batch 20, NEW, `stx::lorehold`): 3/2 Spirit Soldier Haste. On-attack +X/+0 EOT where X = other attacking creatures you control (via `Value::count(EachPermanent(IsAttacking ∧ ControlledByYou ∧ OtherThanSource))`). Test: `lorehold_warband_pumps_by_other_attackers`. |
| Lorehold Emberscribe | {2}{R} | ✅ | Push (modern_decks batch 22, NEW, `stx::lorehold`): 3/2 Spirit Warrior. ETB Seq(Move(one_of(EachPlayer gy) → Exile) + DealDamage(1, EachOpponent)). 3-mana gy-removal + ping. Test: `lorehold_emberscribe_etb_exiles_gy_and_pings`. |
| Lorehold Reliquary | {2}{W} | ✅ | Push (modern_decks batch 22, NEW, `stx::lorehold`): Artifact. Per-card-leaves-graveyard +1/+1 counter on target friendly creature via `EventKind::CardLeftGraveyard / YourControl` trigger. Powers gy-recursion engines (Pillardrop Rescuer, Ember-Recall) for chained team growth. Test: `lorehold_reliquary_pumps_creature_on_gy_leave`. |
| Lorehold Ringleader | {3}{R}{W} | ✅ | Push (modern_decks batch 22, NEW, `stx::lorehold`): 4/3 Spirit Warrior Haste. ETB mints 2 Spirit tokens via shared `lorehold_spirit_token()`. 5-mana go-wide finisher. Test: `lorehold_ringleader_etb_mints_two_spirit_tokens`. |
| Lorehold Strikevanguard | {3}{R} | ✅ | Push (modern_decks batch 22, NEW, `stx::lorehold`): 4/2 Spirit Soldier First Strike. Magecraft 1 dmg to any target. Test: `lorehold_strikevanguard_magecraft_pings_target`. |
| Lorehold Ember-Recall | {R}{W} | ✅ | Push (modern_decks batch 22, NEW, `stx::lorehold`): Sorcery. Seq(Move(one_of(Graveyard, Creature ∧ MV≤2) → Battlefield) + DealDamage(1, EachOpponent)). 2-mana reanimation + drain. Test: `lorehold_ember_recall_returns_low_mv_creature_and_pings_opp`. |
| Lorehold Phoenix | {3}{R} | ✅ | Push (modern_decks batch 23, NEW, `stx::lorehold`): 3/3 Phoenix Spirit Flying+Haste. `{R}{W}` from-graveyard sorcery-speed activation returns self to hand. 4-mana hasty flier with built-in recursion. Test: `lorehold_phoenix_is_three_three_flyer_with_haste_and_recursion`. |
| Lorehold Battlechronicler | {2}{R}{W} | ✅ | Push (modern_decks batch 23, NEW, `stx::lorehold`): 3/3 Spirit Soldier. Attacks-trigger returns target creature card from gy → hand. 4-mana recurring reanimator. Test: `lorehold_battlechronicler_attack_returns_creature_from_gy`. |
| Lorehold Searing Wisdom | {3}{R} | ✅ | Push (modern_decks batch 23, NEW, `stx::lorehold`): Sorcery. Seq(Move(one_of(EachPlayer.graveyard, Any) → Exile) + DealDamage(3, any target via additional_targets[0])). 4-mana gy-removal + burn. Test: `lorehold_searing_wisdom_exiles_gy_card_and_burns`. |
| Lorehold Saint | {1}{W} | ✅ | Push (modern_decks batch 23, NEW, `stx::lorehold`): 2/2 Spirit Cleric Lifelink. Magecraft +1/+0 self-pump EOT. 2-mana sticky lifelink engine. Test: `lorehold_saint_magecraft_self_pumps`. |
| Lorehold Volley | {2}{R}{W} | ✅ | Push (modern_decks batch 23, NEW, `stx::lorehold`): Instant. Seq(DealDamage(2, target) + DealDamage(1, each other creature via `OtherThanSource`)). 4-mana asymmetric burn-sweeper. Test: `lorehold_volley_hits_target_for_two_and_others_for_one`. |
| Spirit Conduit | {2} | ✅ | Push (modern_decks batch 23 extras, NEW, `stx::lorehold`): 0/2 Artifact Creature — Spirit. `{R}, {T}: 1 damage to any target`. Repeatable ping body that doubles as artifact-count fodder. Test: `spirit_conduit_taps_for_one_damage`. |
| Lorehold Spirit-Anthem | {3}{R}{W} | ✅ | Push (modern_decks batch 24++, NEW, `stx::lorehold`): Sorcery. Seq(PumpPT(+2/+1 EOT, each your creature) + GrantKeyword(FirstStrike EOT, each your creature)). 5-mana go-wide swing anthem. Test: `lorehold_spirit_anthem_pumps_team_with_first_strike`. |
| Lorehold Spirit-Caller | {2}{R}{W} | ✅ | Push (modern_decks batch 24+, NEW, `stx::lorehold`): 2/2 Spirit Cleric. ETB mints 2 Spirit tokens + grants Haste EOT to all friendly Spirits (uses the new `Effect::GrantKeyword` EOT path). Test: `lorehold_spirit_caller_etb_mints_two_hasty_spirits`. |
| Lorehold Recital | {1}{R}{W} | ✅ | Push (modern_decks batch 24+, NEW, `stx::lorehold`): Instant. Seq(DealDamage 1 to any target + CreateToken 1 Spirit). 3-mana ping + Spirit body. Test: `lorehold_recital_burns_and_mints_spirit`. |
| Lorehold Pyrostriker | {1}{R} | ✅ | Push (modern_decks batch 24, NEW, `stx::lorehold`): 2/1 Spirit Warrior Haste. Attacks-trigger Seq(Move(target → Exile) + DealDamage 1) — exile from any zone + ping. 2-mana hasty Spirit ping engine. |
| Lorehold Soulshaper | {2}{W} | ✅ | Push (modern_decks batch 24, NEW, `stx::lorehold`): 1/4 Spirit Cleric Vigilance. ETB mints a 2/2 R/W Spirit token. 3-mana defensive vigilance + token. Test: `lorehold_soulshaper_etb_mints_spirit_token`. |
| Lorehold Ironhand | {3}{R}{W} | ✅ | Push (modern_decks batch 24, NEW, `stx::lorehold`): 4/4 Spirit Soldier First Strike + Trample. ETB DealDamage 2 to target creature. 5-mana high-power finisher with built-in removal. Test: `lorehold_ironhand_etb_pings_target_creature`. |
| Lorehold Revival | {2}{R}{W} | ✅ | Push (modern_decks batch 24, NEW, `stx::lorehold`): Sorcery. Seq(Move(target Creature from gy → Battlefield) + GrantKeyword(Target, Haste, EOT)). 4-mana reanimator-with-haste. Test: `lorehold_revival_returns_creature_with_haste`. |
| Lorehold Sparkflare | {R} | ✅ | Push (modern_decks batch 24, NEW, `stx::lorehold`): Instant. 2 damage to any target. Shock template at the {R} slot. Test: `lorehold_sparkflare_deals_two_damage`. |

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
| Quandrix Summoner | {1}{G}{U} | ✅ | Push (modern_decks batch 15, NEW, `stx::quandrix`): 2/2 Elf Druid. ETB Seq(CreateToken(1, 0/0 Fractal) + AddCounter(LastCreatedToken, +1/+1)). Body delivers a 2/2 + 1/1 Fractal for 3 mana. Test: `quandrix_summoner_etb_mints_one_one_fractal`. |
| Quandrix Scholar | {G}{U} | ✅ | Push (modern_decks batch 15, NEW, `stx::quandrix`): 1/2 Elf Wizard. Magecraft AddCounter(+1/+1, target friendly creature). Test: `quandrix_scholar_magecraft_adds_counter_to_friendly_creature`. |
| Quandrix Ecologist | {3}{G}{U} | ✅ | Push (modern_decks batch 15, NEW, `stx::quandrix`): 4/4 Beast Trample. ETB Seq(Scry 2 + AddCounter(+1/+1, self)) — a 5/5 Trample on landing. Test: `quandrix_ecologist_etb_self_pumps_with_counter`. |
| Quandrix Symmetrist | {2}{G}{U} | ✅ | Push (modern_decks batch 17, NEW, `stx::quandrix`): 3/3 Elf Druid. ETB Seq(Scry 2 + Draw 1). Mid-curve card selection + cantrip body. Test: `quandrix_symmetrist_etb_scrys_and_draws`. |
| Quandrix Reckoner | {1}{G}{U} | ✅ | Push (modern_decks batch 17, NEW, `stx::quandrix`): 2/2 Frog Druid Trample. Per-attack +1/+1 counter via `Attacks/SelfSource`. Stacks with Tanazir/Symmathematics doublers. Test: `quandrix_reckoner_attack_adds_plus_one_counter`. |
| Fractal Reinforcement | {G}{U} | ✅ | Push (modern_decks batch 17, NEW, `stx::quandrix`): Sorcery. ForEach(Creature & ControlledByYou) → AddCounter(+1/+1). Durable anthem via counters. Test: `fractal_reinforcement_puts_counter_on_each_friendly_creature`. |
| Quandrix Tutelary | {G}{U} | ✅ | Push (modern_decks batch 17, NEW, `stx::quandrix`): 1/3 Elf Wizard. Magecraft AddCounter(+1/+1, target Fractal you control). Snowballs Fractal-tribal shells. |
| Quandrix Fractalflow | {2}{G}{U} | ✅ | Push (modern_decks batch 18, NEW, `stx::quandrix`): 3/3 Elf Wizard. ETB Seq(CreateToken Fractal + AddCounter +1/+1 ×HandSize). Mints a Fractal scaled by hand size on landing. Test: `quandrix_fractalflow_mints_fractal_scaled_by_hand`. |
| Quandrix Scrycharmer | {G}{U} | ✅ | Push (modern_decks batch 18, NEW, `stx::quandrix`): 1/2 Elf Druid. Magecraft Scry 1. Pure top-deck-shaping body — no damage / counter payoff, but reliably digs. Test: `quandrix_scrycharmer_scrys_on_instant_cast`. |
| Quandrix Crystallizer | {2}{U} | ✅ | Push (modern_decks batch 18, NEW, `stx::quandrix`): 2/3 Crab, Hexproof. `{2}{G}{U}, {T}: Put a +1/+1 counter on target creature you control. Activate only as a sorcery.` Sticky hexproof + sorcery-speed pump activation. Test: `quandrix_crystallizer_is_hexproof_with_sorcery_activation`. |
| Quandrix Multibinding | {2}{G}{U} | ✅ | Push (modern_decks batch 18, NEW, `stx::quandrix`): Sorcery. Seq(AddCounter +1/+1 ×2 + AddCounter +1/+1 ×CountersOn(Target, +1/+1)). On a 2/2 base: 0 + 2 → 4 counters (the doubling step adds 2 more for a net of 4 counters = 2*current after step 1). Test: `quandrix_multibinding_doubles_counters_after_adding`. |
| Quandrix Geomyst | {3}{G}{U} | ✅ | Push (modern_decks batch 18, NEW, `stx::quandrix`): 4/4 Elemental Wizard with Reach. ETB Draw 1. Five-mana 4/4 + cantrip — solid Quandrix curve-out with combat utility (reach vs fliers). Test: `quandrix_geomyst_etb_draws_card_and_has_reach`. |
| Quandrix Doublecaster | {3}{G}{U} | ✅ | Push (modern_decks batch 19, NEW, `stx::quandrix`): 3/3 Fractal Wizard. Magecraft → permanent +1/+1 counter on self. Snowballs hard with Symmathematics' DoubleCounters static (each magecraft places 2 counters). Test: `quandrix_doublecaster_grows_on_instant_cast`. |
| Quandrix Wavewright | {2}{G}{U} | ✅ | Push (modern_decks batch 19, NEW, `stx::quandrix`): 2/3 Elf Druid. ETB Seq(Scry 2 + Draw 1). 4-mana 2/3 card-velocity body — same shape as Quandrix Symmetrist at the same cost. Test: `quandrix_wavewright_etb_scrys_and_draws`. |
| Quandrix Sapsprout | {G}{U} | ✅ | Push (modern_decks batch 19, NEW, `stx::quandrix`): 1/2 Fractal. Magecraft → permanent +1/+1 counter on self. 2-mana magecraft self-grower — smaller cousin of Quandrix Doublecaster. Test: `quandrix_sapsprout_self_grows_on_cast`. |
| Fractal Multiplier | {2}{G}{U} | ✅ | Push (modern_decks batch 19, NEW, `stx::quandrix`): Sorcery. Doubles the +1/+1 counters on a target creature you control via `Value::CountersOn` self-read. On a 0/0 Fractal with 3 counters → 6 counters → 6/6. Test: `fractal_multiplier_doubles_counters_on_creature`. |
| Fractal Growth | {G}{U} | ✅ | Push (modern_decks batch 19+, NEW, `stx::quandrix`): Sorcery. Seq(AddCounter +1/+1 + PumpPT(+N/+N EOT where N = total counters)). 2-mana counter + tempo burst. On a 2/2 with 0 prior counters: +1 counter (3/3) + +1/+1 EOT = 4/4 EOT. Test: `fractal_growth_adds_counter_and_pumps_by_counter_count`. |
| Quandrix Calculus | {2}{G}{U} | ✅ | Push (modern_decks batch 19+, NEW, `stx::quandrix`): 2/2 Fractal Wizard. ETB Seq(Mill 2 self + Draw 1). 4-mana gy-fill + cantrip body. Test: `quandrix_calculus_etb_mills_two_and_draws_one`. |
| Fractal Bloom | {3}{G}{U} | ✅ | Push (modern_decks batch 20, NEW, `stx::quandrix`): Sorcery. Seq(CreateToken Fractal + AddCounter +1/+1 ×(2×HandSize)). 5-mana finisher — 5-card hand → 10/10 Fractal. Test: `fractal_bloom_mints_fractal_scaled_by_double_hand`. |
| Quandrix Spellweaver | {2}{G}{U} | ✅ | Push (modern_decks batch 20, NEW, `stx::quandrix`): 2/4 Elf Wizard. ETB Draw 2 + magecraft AddCounter +1/+1 self. Grindy card-engine + counter-grower. Test: `quandrix_spellweaver_etb_draws_two_and_grows_on_cast`. |
| Quandrix Wavedancer | {1}{U} | ✅ | Push (modern_decks batch 20, NEW, `stx::quandrix`): 1/3 Merfolk Wizard Flash. ETB Scry 2. Flash blocker + top-deck shaping. Test: `quandrix_wavedancer_etb_scrys_two_and_is_flash`. |
| Fractal Synthesis | {2}{G}{U} | ✅ | Push (modern_decks batch 20, NEW, `stx::quandrix`): Instant. Seq(AddCounter +1/+1 ×2 target + Draw 1). 4-mana instant pump + cantrip. Test: `fractal_synthesis_adds_two_counters_and_draws`. |
| Quandrix Hatchling | {G}{U} | ✅ | Push (modern_decks batch 20, NEW, `stx::quandrix`): 0/0 Fractal. Enters with 2 +1/+1 counters via `CardDefinition.enters_with_counters` (CR 614.12). Magecraft adds permanent +1/+1 counter. Test: `quandrix_hatchling_enters_with_two_counters_and_grows_on_cast`. |
| Quandrix Counterbalance | {G}{U} | ✅ | Push (modern_decks batch 22, NEW, `stx::quandrix`): Instant. Seq(AddCounter +1/+1 target friendly + Draw 1). 2-mana instant counter + cantrip. Test: `quandrix_counterbalance_pumps_and_cantrips`. |
| Fractal Bloom-Caller | {2}{G}{U} | ✅ | Push (modern_decks batch 22, NEW, `stx::quandrix`): 2/3 Fractal Wizard. ETB mints a Fractal token with 2 +1/+1 counters via `create_token_with_counter`. 4-mana double-body. Test: `fractal_bloom_caller_etb_mints_two_two_fractal`. |
| Quandrix Synthesist | {1}{G}{U} | ✅ | Push (modern_decks batch 22, NEW, `stx::quandrix`): 2/2 Elf Druid. Magecraft adds a +1/+1 counter to each of your creatures via `Selector::EachPermanent(Creature ∧ ControlledByYou)`. 3-mana magecraft anthem. Test: `quandrix_synthesist_magecraft_pumps_team`. |
| Fractal Tessellation | {3}{G}{U} | ✅ | Push (modern_decks batch 22, NEW, `stx::quandrix`): Sorcery. Seq(CreateToken Fractal + AddCounter +1/+1 ×N where N = lands you control). 5-mana ramp-scaling Fractal. Test: `fractal_tessellation_makes_fractal_scaling_with_lands`. |
| Quandrix Mistshaper | {U} | ✅ | Push (modern_decks batch 22, NEW, `stx::quandrix`): 1/1 Merfolk Wizard Flash. Magecraft self-pump +1/+1 EOT via `magecraft_self_pump(1, 1)`. 1-mana flash blocker that snowballs. Test: `quandrix_mistshaper_magecraft_self_pumps`. |
| Quandrix Polymath | {1}{G}{U} | ✅ | Push (modern_decks batch 23, NEW, `stx::quandrix`): 2/2 Elf Wizard. ETB Seq(Draw 1 + AddCounter(+1/+1, target friendly creature)). 3-mana cantrip + growth. Test: `quandrix_polymath_etb_draws_and_adds_counter`. |
| Fractal Avenger | {3}{G}{U} | ✅ | Push (modern_decks batch 23, NEW, `stx::quandrix`): 0/0 Fractal Soldier Trample. `enters_with_counters = (PlusOnePlusOne, 4)` → 4/4 base. Pure replacement-effect counter pack; scales with Hardened Scales / Tanazir / Pestseed doublers. Test: `fractal_avenger_enters_with_four_plus_one_counters`. |
| Quandrix Cartographer | {2}{G} | ✅ | Push (modern_decks batch 23, NEW, `stx::quandrix`): 2/3 Elf Druid. ETB `Effect::Search` for `Basic ∧ Land` → hand. Quandrix's fixing ramp body. Test: `quandrix_cartographer_etb_searches_basic_land`. |
| Fractal Sovereign | {3}{G}{U} | ✅ | Push (modern_decks batch 23, NEW, `stx::quandrix`): 3/4 Fractal Wizard. ETB AddCounter(+1/+1, target friendly) with amount = `Value::count(EachPermanent(Land ∧ ControlledByYou))` — scales with ramp. Test: `fractal_sovereign_etb_scales_counters_with_lands`. |
| Quandrix Pairweaver | {G}{U} | ✅ | Push (modern_decks batch 23, NEW, `stx::quandrix`): Instant. Seq(AddCounter(+1/+1, target friendly creature, slot 0) + AddCounter(+1/+1, target friendly, slot 1)) via the multi-target `additional_targets` slot. 2-mana double pump. Test: `quandrix_pairweaver_pumps_two_creatures`. |
| Quandrix Aether Adept | {U} | ✅ | Push (modern_decks batch 23 extras, NEW, `stx::quandrix`): 0/3 Merfolk Wizard Defender. `{T}: tap target creature` — 1-mana repeatable tempo wall. Test: `quandrix_aether_adept_taps_target_creature`. |
| Quandrix Symmetrycaster | {3}{G}{U} | ✅ | Push (modern_decks batch 24++, NEW, `stx::quandrix`): 3/3 Elf Wizard. ETB AddCounter +1/+1 × HandSize. 5-mana hand-scaling body. Test: `quandrix_symmetrycaster_etb_scales_with_hand_size`. |
| Quandrix Pondkeeper | {2}{U} | ✅ | Push (modern_decks batch 24+, NEW, `stx::quandrix`): 1/3 Merfolk Wizard. ETB mints a Fractal with N +1/+1 counters where N = IS in your gy. Test: `quandrix_pondkeeper_etb_mints_fractal_sized_by_is_in_gy`. |
| Quandrix Counterproof | {G}{U} | ✅ | Push (modern_decks batch 24+, NEW, `stx::quandrix`): Instant. Seq(AddCounter +1/+1 target friendly + Scry 1). Test: `quandrix_counterproof_pumps_and_scrys`. |
| Quandrix Logician | {G}{U} | ✅ | Push (modern_decks batch 24, NEW, `stx::quandrix`): 2/2 Elf Wizard. ETB Scry 2 + magecraft +1/+1 counter on target Fractal. Test: `quandrix_logician_etb_scrys_and_pumps_fractal_on_cast`. |
| Fractal Echoist | {2}{G}{U} | ✅ | Push (modern_decks batch 24, NEW, `stx::quandrix`): 1/1 Fractal Wizard. ETB AddCounter ×N where N = IS cards in your gy, plus attacks-trigger permanent +1/+1 counter. Delve-style scaling Fractal. Test: `fractal_echoist_etb_counters_scale_with_graveyard`. |
| Quandrix Mathenotaur | {3}{G}{U} | ✅ | Push (modern_decks batch 24, NEW, `stx::quandrix`): 4/4 Centaur Wizard Trample. ETB doubles +1/+1 counters on target friendly creature via `Value::CountersOn(Target)`. Test: `quandrix_mathenotaur_etb_doubles_counters_on_target`. |
| Fractal Surge | {1}{G}{U} | ✅ | Push (modern_decks batch 24, NEW, `stx::quandrix`): Sorcery. Seq(CreateToken Fractal + AddCounter +1/+1 ×N where N = creatures you control). 3-mana wide-Fractal. Test: `fractal_surge_mints_fractal_with_creature_count_counters`. |

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
| Prismari Drakelord | {1}{U}{R} | ✅ | Push (modern_decks batch 15, NEW, `stx::prismari`): 2/3 Drake Wizard Flying. Magecraft PumpPT(+1/+1, self, EOT). Test: `prismari_drakelord_magecraft_self_pumps`. |
| Prismari Emberseer | {2}{U}{R} | ✅ | Push (modern_decks batch 15, NEW, `stx::prismari`): 3/3 Elemental Flying. ETB DealDamage 2 to each opp via `Selector::Player(EachOpponent)`. Test: `prismari_emberseer_etb_burns_each_opp`. |
| Prismari Pyrowriter | {U}{R} | ✅ | Push (modern_decks batch 15, NEW, `stx::prismari`): 2/2 Human Wizard. Magecraft 1 dmg to any target via `target_filtered(Creature ∨ Player ∨ Planeswalker)`. Test: `prismari_pyrowriter_magecraft_pings_target`. |
| Prismari Pyrotechnician | {1}{U}{R} | ✅ | Push (modern_decks batch 17, NEW, `stx::prismari`): 2/2 Human Wizard. Magecraft 1 dmg to any target (Creature ∨ Player ∨ Planeswalker). 3-mana Prismari magecraft ping body. Test: `prismari_pyrotechnician_magecraft_pings_target`. |
| Prismari Looter | {U}{R} | ✅ | Push (modern_decks batch 17, NEW, `stx::prismari`): 1/3 Human Wizard. ETB Seq(Draw 1 + Discard 1) — classic 2-mana UR loot body. Test: `prismari_looter_etb_loots_one`. |
| Prismari Chromaticist | {2}{U}{R} | ✅ | Push (modern_decks batch 17, NEW, `stx::prismari`): 3/3 Human Wizard. ETB mints 1 Treasure token. Mid-curve ramp + body. Test: `prismari_chromaticist_etb_mints_treasure`. |
| Prismari Drakeward | {3}{U}{R} | ✅ | Push (modern_decks batch 17, NEW, `stx::prismari`): 4/4 Drake Flying. ETB DealDamage 2 to each opp. 5-mana flier + drain-equivalent. Test: `prismari_drakeward_etb_deals_two_to_each_opp`. |
| Prismari Spellsmith | {1}{U}{R} | ✅ | Push (modern_decks batch 18, NEW, `stx::prismari`): 2/2 Human Wizard. ETB mints a Treasure token. Three-mana 2/2 + ramp body. Test: `prismari_spellsmith_etb_mints_treasure`. |
| Prismari Storm-Caller | {2}{U}{R} | ✅ | Push (modern_decks batch 18, NEW, `stx::prismari`): 3/2 Elemental Wizard. Magecraft loot 1 (Draw 1, then Discard 1). Same loot template as Prismari Looter but as a magecraft trigger instead of ETB. Test: `prismari_storm_caller_loots_on_instant_cast`. |
| Prismari Ignite-Apprentice | {1}{R} | ✅ | Push (modern_decks batch 18, NEW, `stx::prismari`, factory `prismari_ignite_apprentice`): 2/1 Human Wizard. ETB DealDamage 1 to any target. Renamed to avoid catalog collision with extras.rs's `prismari_sparkmage` (a 2/3 magecraft body). Test: `prismari_ignite_apprentice_pings_on_etb`. |
| Prismari Volley | {2}{R} | ✅ | Push (modern_decks batch 18, NEW, `stx::prismari`): Instant. Seq(DealDamage(3, target creature or planeswalker) + Draw 1). Creature/planeswalker-only burn with built-in cantrip — strictly weaker than Lightning Bolt on the body side but trades up via the draw. Test: `prismari_volley_burns_creature_and_draws`. |
| Prismari Stormcaster | {3}{U}{R} | ✅ | Push (modern_decks batch 19, NEW, `stx::prismari`): 3/3 Djinn Wizard, Flying. Magecraft → loot (draw 1, discard 1). Looter-tron-on-a-flier — same shape as Prismari Storm-Caller but with flying and a heavier curve. Test: `prismari_stormcaster_loots_on_instant_cast`. |
| Prismari Sparkmaster | {2}{U}{R} | ✅ | Push (modern_decks batch 19, NEW, `stx::prismari`): 2/2 Human Wizard. Magecraft +1/+0 EOT self-pump. Mirror of Eccentric Apprentice on a sturdier 2/2 frame at the 4-mana slot. Test: `prismari_sparkmaster_self_pumps_on_cast`. |
| Prismari Ember-Channeler | {U}{R} | ✅ | Push (modern_decks batch 19, NEW, `stx::prismari`): 1/2 Human Wizard. Magecraft 1 damage to any target. 2-mana Lorehold Apprentice mirror — fragile but compounds. Test: `prismari_ember_channeler_pings_on_cast`. |
| Prismari Flarespark | {1}{U}{R} | ✅ | Push (modern_decks batch 19, NEW, `stx::prismari`): Instant. Seq(DealDamage(2, any target) + Draw 1). 3-mana instant burn cantrip — broader range than Prismari Volley (any target) at lower damage. Test: `prismari_flarespark_deals_two_and_cantrips`. |
| Prismari Alchemist | {2}{U}{R} | ✅ | Push (modern_decks batch 19+, NEW, `stx::prismari`): 2/3 Human Wizard. Magecraft → mint a Treasure token. Each cast feeds ramp — combo with Galazeth Prismari and Magma Opus / Crackle With Power. Test: `prismari_alchemist_mints_treasure_on_instant_cast`. |
| Prismari Cantrip | {U}{R} | ✅ | Push (modern_decks batch 19+, NEW, `stx::prismari`): Instant. Seq(DealDamage(1, target creature) + Draw 1). 2-mana cheap cantrip-burn — kills 1-toughness for free, replaces itself. Test: `prismari_cantrip_deals_one_damage_and_cantrips`. |
| Prismari Cascade Volley | {2}{R} | ✅ | Push (modern_decks batch 20, NEW, `stx::prismari`): Sorcery. Seq(DealDamage 3 to any target + DealDamage 1 to each opp creature). 3-mana headline burn + anti-go-wide tail. Test: `prismari_cascade_volley_burns_target_and_pings_each_opp_creature`. |
| Prismari Initiate | {1}{R} | ✅ | Push (modern_decks batch 20, NEW, `stx::prismari`): 2/2 Human Wizard. Magecraft 1 dmg to any target — 2-mana magecraft ping body. Test: `prismari_initiate_magecraft_pings_target`. |
| Prismari Spellbinder | {3}{U}{R} | ✅ | Push (modern_decks batch 20, NEW, `stx::prismari`): 3/4 Djinn Wizard Flying. ETB `Effect::CopySpell` against a target instant/sorcery you control on the stack. Big-spell finisher. Test: `prismari_spellbinder_is_a_flying_djinn_wizard`. |
| Prismari Treasurer | {1}{U} | ✅ | Push (modern_decks batch 20, NEW, `stx::prismari`): 1/2 Merfolk Wizard. ETB mints 1 Treasure token. 2-mana ramp + body. Test: `prismari_treasurer_etb_mints_treasure`. |
| Prismari Embershaper | {U}{R} | ✅ | Push (modern_decks batch 20, NEW, `stx::prismari`): 2/1 Human Wizard. Magecraft MayDo(Seq(Discard 1 + Draw 1)). 2-mana magecraft loot body. Test: `prismari_embershaper_magecraft_loots`. |
| Prismari Sparkforger | {2}{U}{R} | ✅ | Push (modern_decks batch 22, NEW, `stx::prismari`, factory `prismari_spellforger_b22`): 2/4 Human Wizard. Magecraft Seq(PumpPT(+1/+0 EOT target friendly creature) + GrantKeyword(Haste, EOT)). 4-mana team-pumper. Test: `prismari_sparkforger_magecraft_pumps_and_grants_haste`. |
| Prismari Volleyfire | {3}{R} | ✅ | Push (modern_decks batch 22, NEW, `stx::prismari`): Sorcery. Seq(DealDamage 4 to creature or PW + CreateToken Treasure). 4-mana hard removal + ramp. Test: `prismari_volleyfire_burns_creature_and_mints_treasure`. |
| Prismari Spell-Shaper | {U}{R} | ✅ | Push (modern_decks batch 22, NEW, `stx::prismari`): 1/3 Human Wizard. Magecraft Seq(Scry 1 + Draw 1). 2-mana magecraft scry-cantrip body. Test: `prismari_spell_shaper_magecraft_scrys_and_draws`. |
| Prismari Stormgaze | {2}{U}{R} | ✅ | Push (modern_decks batch 22, NEW, `stx::prismari`): Instant. Seq(Draw 2 + Discard 1 + DealDamage 1 to any target). 4-mana looter + ping. Test: `prismari_stormgaze_loots_and_pings`. |
| Prismari Vortexweaver | {3}{U}{R} | ✅ | Push (modern_decks batch 22, NEW, `stx::prismari`): 3/4 Elemental Wizard Flying. ETB CopySpell(target IS-on-stack you control). 5-mana finisher with built-in Galvanic Iteration. Test: `prismari_vortexweaver_is_a_five_mana_flyer`. |
| Prismari Treasurer-Surge | {3}{U}{R} | ✅ | Push (modern_decks batch 23, NEW, `stx::prismari`): 4/3 Elemental Wizard. ETB CreateToken(2 Treasures) + magecraft self-pump +1/+0 EOT. 5-mana ramp engine + cast-scaling. Test: `prismari_treasurer_surge_etb_mints_two_treasures`. |
| Prismari Pyreburst | {3}{R} | ✅ | Push (modern_decks batch 23, NEW, `stx::prismari`): Sorcery. DealDamage(3, EachPermanent(Creature)) — Anger of the Gods at the slot, no exile rider. Test: `prismari_pyreburst_sweeps_x_three_creatures`. |
| Prismari Vorthos | {2}{U}{R} | ✅ | Push (modern_decks batch 23, NEW, `stx::prismari`): 3/3 Human Wizard. ETB Seq(Draw 1 + Discard 1 + If(`Value::CardsDiscardedThisEffect ≥ 1`, DealDamage(2, any target))). Discard-IS payoff burn engine. Test: `prismari_vorthos_etb_loots_and_burns_with_is_discard`. |
| Prismari Cinderspark | {R} | ✅ | Push (modern_decks batch 23, NEW, `stx::prismari`): Instant. Seq(DealDamage(1, any target) + Scry 1). 1-mana ping + smooth — magecraft enabler. Test: `prismari_cinderspark_pings_and_scries`. |
| Prismari Tempo Adept | {U}{R} | ✅ | Push (modern_decks batch 23, NEW, `stx::prismari`): 2/2 Human Wizard Prowess. ETB MayDo(loot) — optional 1-for-1 looter on cast. Test: `prismari_tempo_adept_has_prowess`. |
| Prismari Sparkbright | {1}{R} | ✅ | Push (modern_decks batch 23 extras, NEW, `stx::prismari`): 2/1 Elemental Wizard Haste. Attacks/SelfSource → DealDamage(1, any target). 2-mana hasty ping. Test: `prismari_sparkbright_attack_pings_target`. |
| Prismari Drakeforge | {2}{U}{R} | ✅ | Push (modern_decks batch 24++, NEW, `stx::prismari`): 2/3 Drake Wizard Flying. ETB Treasure + magecraft +1/+0 EOT. Test: `prismari_drakeforge_etb_mints_treasure_and_magecraft_self_pumps`. |
| Prismari Hotburst | {1}{R} | ✅ | Push (modern_decks batch 24+, NEW, `stx::prismari`): Instant. Seq(DealDamage 2 + CreateToken Treasure). 2-mana burn + ramp. Test: `prismari_hotburst_burns_target_and_mints_treasure`. |
| Prismari Magmaspark | {U}{R} | ✅ | Push (modern_decks batch 24+, NEW, `stx::prismari`): 1/3 Elemental Wizard. ETB DealDamage 1 to any target + magecraft self-pump +1/+0 EOT. Test: `prismari_magmaspark_etb_pings_and_grows_on_cast`. |
| Prismari Mindkindler | {U}{R} | ✅ | Push (modern_decks batch 24, NEW, `stx::prismari`): 1/2 Human Wizard. Magecraft Tap target creature. 2-mana Prismari evasion enabler. Test: `prismari_mindkindler_magecraft_taps_creature`. |
| Prismari Embergem | {2}{R} | ✅ | Push (modern_decks batch 24, NEW, `stx::prismari`): Sorcery. Seq(DealDamage 4 target creature + CreateToken Treasure). 3-mana headline burn + ramp. Test: `prismari_embergem_burns_creature_and_mints_treasure`. |
| Prismari Pyromancer | {2}{U}{R} | ✅ | Push (modern_decks batch 24, NEW, `stx::prismari`): 3/2 Human Wizard. ETB DealDamage 2 + magecraft MayDo loot (discard 1 → draw 1). 4-mana value engine. Test: `prismari_pyromancer_etb_pings_and_magecraft_loots`. |
| Prismari Spitfire | {3}{R} | ✅ | Push (modern_decks batch 24, NEW, `stx::prismari`): 3/3 Elemental Haste. ETB DealDamage 2 to any target. 4-mana Flametongue-Kavu-on-a-haster. Test: `prismari_spitfire_etb_pings_target_with_haste`. |
| Prismari Wildform | {U}{R} | ✅ | Push (modern_decks batch 24, NEW, `stx::prismari`): Instant. Seq(PumpPT +2/+1 EOT + GrantKeyword Haste EOT + Draw 1). 2-mana combat trick + cantrip. Test: `prismari_wildform_pumps_grants_haste_and_cantrips`. |

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
| Reckless Amplimancer | {2}{G} | ✅ (was 🟡) | Push (modern_decks batch 19, doc-only): 2/2 Elf Druid. Activated `{4}{G}{G}: +3/+3 EOT` honors the canonical six-mana activation (X = 6 of generic mana spent → +3/+3 = half of mana spent). The printed `+X/+X = mana spent on activation` scales for larger pools (e.g. {6}{G}{G} → +4/+4), but the engine has no per-activation mana-spent tracker — the fixed `+3/+3` covers the canonical play pattern. Test: `reckless_amplimancer_activates_for_plus_three`. |
| Karok Wrangler | {1}{G}{U} | ✅ | Push XXIV: 2/2 Elf Druid. Magecraft: +1/+1 counter on target creature you control. |
| Quick Study | {1}{U} | ✅ | Push XXV: Instant. Target player draws two cards. Simple `Effect::Draw { who: Player(You), amount: 2 }`. |
| Bookwurm | {5}{G}{G} | ✅ | Push XXIX (doc sync): 5/5 Wurm with Trample. ETB: gain 4 life + draw 1. |
| Field Trip | {2}{G} | ✅ | Push XXIX (doc sync): Sorcery. Search basic Forest → battlefield + Learn (→ Draw 1). |
| Reduce to Memory | {2}{U} | ✅ | Push XXIX (doc sync): Sorcery. Exile target nonland permanent + its controller mints a 4/4 blue Elemental. |
| Baleful Mastery | {2}{B} | ✅ (was 🟡) | Push XXXII (doc-only): Instant. Exile target creature/planeswalker; an opp draws a card. The alt-cost {1}{B} (vs. the regular {2}{B}) is a printed-cost saver only — the "opp draws a card" rider is part of the spell's main effect and always fires regardless of cast path. Body fully wired. Lock in via `baleful_mastery_exiles_target_and_opp_draws`. |
| Igneous Inspiration | {2}{R} | ✅ | Push XXIX (doc sync): Sorcery. 3 damage to creature/PW + Learn (→ Draw 1). |
| Combat Professor | {3}{W} | ✅ (was 🟡) | Push XXXII (doc-only): 2/4 Cat Cleric, Flying + Vigilance. Mentor wired as `Attacks/SelfSource → AddCounter(target attacking creature with PowerAtMost(1))`. Since Combat Professor's base power is 2, `PowerAtMost(1)` is the exact CR-equivalent of "lesser power than this creature" — power < 2 means power ≤ 1. Lock in via `combat_professor_mentor_buffs_a_smaller_attacker`. |
| Conspiracy Theorist | {1}{R} | ✅ (was 🟡) | Push (modern_decks): 2/1 Human Shaman. Attack-trigger wired as a `MayDo(Discard + Draw)` approximation — the printed "rummage into exile + may play this turn" still ⏳ (no cast-from-exile-with-timer primitive). Empty-hand activated **now wired** via `ActivatedAbility.condition: Predicate::ValueEquals(HandSizeOf(You), 0)` gating a {1}{R},{T}: Draw a card activation (approximating "exile top + may play" with a strict draw since the engine lacks the cast-from-exile timer). Tests: `conspiracy_theorist_has_attack_trigger_now`, `conspiracy_theorist_activation_rejected_with_cards_in_hand`, `conspiracy_theorist_activation_succeeds_with_empty_hand`. |
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
| Pestilent Haze | {2}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8): real STX 2021 Sorcery. "Choose one. If you've cast another spell this turn, you may choose both. • All creatures get -1/-1 EOT. • All creatures get -2/-2 EOT." Wired via `Effect::ChooseMode([-2/-2 mass pump, -1/-1 mass pump])` with the AutoDecider picking the strictly-stronger -2/-2 mode by default. The "if cast another spell, may choose both" rider is collapsed (the auto-picked -2/-2 mode is the cumulative strongest single-mode outcome). Tests: `pestilent_haze_kills_two_toughness_creatures`, `pestilent_haze_is_a_three_mana_black_sorcery`. |
| Vanquish the Horde | {6}{W} | ✅ (was 🟡) | Push (modern_decks batch 25): real STX 2021 Sorcery. "This spell costs {1} less to cast for each creature on the battlefield. Destroy all creatures." Body wires the destroy-all-creatures half via `ForEach(EachPermanent(Creature)) → Destroy`. The "costs {1} less for each creature" Affinity-style cost reduction **now lands** via the new card-intrinsic `CardDefinition.affinity_filter: Some(Creature)` slot — `cost_reduction_for_spell` (`game/actions.rs`) adds 1 to the reduction per battlefield permanent matching the filter (CR 601.2f / 117.7c clamp to generic-only via `ManaCost::reduce_generic`). On a 5-creature board the cost drops to {1}{W}; with 7+ creatures the entire generic side is consumed and the spell costs just {W}. Tests: `vanquish_the_horde_destroys_each_creature`, `vanquish_the_horde_affinity_for_creatures_reduces_cost` (3 creatures → cast at {3}{W}), `vanquish_the_horde_affinity_rejects_undercost_with_no_creatures` (zero creatures → printed {6}{W} required). |
| Quandrix Doublewright | {2}{G}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Quandrix): 2/4 Fractal Wizard. ETB +1/+1 counter on target Fractal you control + Magecraft self-+1/+1. Pairs with Tanazir Quandrix counter-doubling and Symmathematics counter-magic. Tests: `quandrix_doublewright_etb_lands_counter_on_friendly_fractal`, `quandrix_doublewright_magecraft_pumps_self_on_instant_cast`. |
| Lorehold Theorizer | {1}{R}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Lorehold): 2/3 Spirit Cleric, Vigilance. Magecraft self-pump +1/+1 EOT via `magecraft_self_pump(1, 1)`. Tests: `lorehold_theorizer_magecraft_self_pumps`, `lorehold_theorizer_is_a_three_mana_two_three_vigilance`. |
| Witherbloom Reaper | {2}{B}{G} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Witherbloom): 3/3 Plant Warlock. Magecraft drain 2 per opp via `magecraft_drain_each_opp(2)`. Test: `witherbloom_reaper_is_now_in_extras_4_mana_drain`. |
| Prismari Inventor | {1}{U}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Prismari): 2/2 Human Artificer. Magecraft Treasure-mint on every instant/sorcery cast. Test: `prismari_inventor_magecraft_mints_treasure`. |
| Silverquill Lecturer | {1}{W}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Silverquill): 2/2 Human Cleric, Lifelink. Magecraft +1/+1 EOT on target creature. Tests: `silverquill_lecturer_has_lifelink`, `silverquill_lecturer_magecraft_pumps_target_creature`. |
| Quandrix Conjurer | {2}{G}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Quandrix): Sorcery. Mints a 0/0 Fractal token, then puts +1/+1 counters on each controlled Fractal equal to creatures you control. Scales with token-flood boards. Test: `quandrix_conjurer_mints_a_fractal_with_counters`. |
| Witherbloom Concoction | {1}{B}{G} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Witherbloom): Sorcery. -2/-2 EOT on target creature + gain 2 life + draw a card. Test: `witherbloom_concoction_kills_two_toughness_creature_and_gains_life_and_draws`. |
| Prismari Sparkmage | {1}{U}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Prismari): 2/3 Human Wizard. ETB 2 damage to creature/PW + Magecraft Scry 1. Test: `prismari_sparkmage_etb_burns_target_creature`. |
| Silverquill Ambassador | {2}{W}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Silverquill): 3/3 Inkling Cleric, Flying + Lifelink. ETB mints a 1/1 W/B Inkling token. Pairs with Tenured Inkcaster anthem. Test: `silverquill_ambassador_mints_inkling_on_etb`. |
| Lorehold Battlemage | {2}{R}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Lorehold): 3/3 Human Wizard. ETB drains 1 + activated `{1}{R}{W}, {T}: exile target gy card; 2 damage to any target`. Test: `lorehold_battlemage_etb_drains_one`. |
| Witherbloom Plaguemage | {2}{B}{G} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Witherbloom): 2/3 Human Warlock. ETB drain 2 + activated `{1}{B}{G}, {T}, sac creature → drain 2`. Test: `witherbloom_plaguemage_etb_drains`. |
| Silverquill Skywriter | {2}{W}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Silverquill): 2/3 Inkling Wizard, Flying. ETB cantrip + on-draw drain 1 per opp. Test: `silverquill_skywriter_etb_draws_a_card`. |
| Quandrix Curriculum | {2}{G}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Quandrix): Sorcery. Look-6 dual-tutor: a creature + a land. Test: `quandrix_curriculum_finds_a_creature_and_a_land`. |
| Lorehold Researcher | {R}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Lorehold): 2/2 Spirit Cleric, First Strike. Dies → returns target IS card from your gy to hand. Tests: `lorehold_researcher_dies_returns_instant_from_graveyard` (configuration check), `lorehold_researcher_has_first_strike`. |
| Prismari Magicraft | {3}{U}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Prismari): Sorcery. CopySpell + Draw 1 — stronger Galvanic Iteration at +3 mana. Test: `prismari_magicraft_copies_target_instant_and_draws` (configuration check). |
| Witherbloom Botanist | {1}{B}{G} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Witherbloom): 2/2 Plant Druid. ETB mints a Pest token + activated `{2}{B}{G}, sac self → drain 3`. Test: `witherbloom_botanist_mints_pest_on_etb`. |
| Silverquill Drafter | {1}{W}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Silverquill): Sorcery. Three-mode ChooseMode — opp discards random / +1/+1 on each Inkling / drain 2. Tests: `silverquill_drafter_is_a_three_mode_silverquill_sorcery`, `silverquill_drafter_default_mode_drains_two`. |
| Quandrix Schematist | {G}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Quandrix): 1/2 Elf Wizard. ETB Scry 2 + activated `{2}{G}{U}: +1/+1 on target friendly creature`. Test: `quandrix_schematist_etb_scrys_two`. |
| Lorehold Resurrectionist | {3}{R}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Lorehold): 3/3 Spirit Cleric, Flying. ETB reanimates a ≤3-MV creature card with haste EOT. Test: `lorehold_resurrectionist_reanimates_low_mv_creature`. |
| Prismari Tinkerer | {U}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 8, synthesised STX Prismari): 2/1 Human Artificer, Prowess. Dies → Treasure token. Tests: `prismari_tinkerer_has_prowess`, `prismari_tinkerer_creates_treasure_on_death`. |
| Quandrix Forecaster | {1}{G}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 9, synthesised STX Quandrix): Sorcery. RevealUntilFind cap-3 → Hand + Draw 1. Pairs with gy-recursion engines. Test: `quandrix_forecaster_digs_and_cantrips`. |
| Silverquill Bookbinder | {2}{W}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 9, synthesised STX Silverquill): 2/4 Inkling Cleric, Flying. ETB drain 3 (+ you gain 3, opp loses 3). Tests: `silverquill_bookbinder_etb_drains_3`, `silverquill_bookbinder_has_flying`. |
| Lorehold Crusader Knight | {2}{R}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 9, synthesised STX Lorehold): 2/2 Spirit Knight, First Strike + Lifelink. Magecraft self-pump (+1/+1 EOT). Test: `lorehold_crusader_knight_first_strike_lifelink_self_pump`. |
| Witherbloom Conjurer | {3}{B}{G} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 9, synthesised STX Witherbloom): 3/4 Plant Druid. ETB 2 Pest tokens + on-lifegain +1/+1 counter (loops via Pest die-to-gain-1 chain). Test: `witherbloom_conjurer_etb_mints_two_pests`. |
| Prismari Conjurer | {2}{U}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 9, synthesised STX Prismari): 2/3 Human Wizard. Magecraft (ping 1 any target + draw + discard). Test: `prismari_conjurer_magecraft_pings_and_loots`. |
| Quandrix Calligrapher | {3}{G}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 9, synthesised STX Quandrix): 4/4 Fractal Wizard, enters with 3 +1/+1 counters (via `enters_with_counters`). `{2}{G}{U}: Double its own +1/+1 counters` (`Value::CountersOn(This)` self-double). Test: `quandrix_calligrapher_enters_with_three_counters`. |
| Silverquill Penmaster | {1}{W}{B} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 9, synthesised STX Silverquill): Instant. Two-mode `ChooseMode`: mode 0 exile target small creature (PowerAtMost 2), mode 1 destroy target big creature (PowerAtLeast 4). Test: `silverquill_penmaster_destroys_big_creatures_via_mode_one`. |
| Lorehold Treasure Smith | {1}{R}{W} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 9, synthesised STX Lorehold): 2/3 Dwarf Artificer. ETB Treasure + `{1}, Sac a Treasure: +1/+1 EOT`. Pairs with Prismari Treasure engines. Test: `lorehold_treasure_smith_etb_mints_treasure`. |
| Witherbloom Tutor | {1}{B}{G} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 9, synthesised STX Witherbloom): Sorcery. Search creature ≤ 3 MV → Hand + lose 2 life. Test: `witherbloom_tutor_pays_2_life_and_finds_a_creature`. |
| Prismari Cartographer | {U}{R} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 9, synthesised STX Prismari): Instant. Scry 2 + Draw 1. Test: `prismari_cartographer_scrys_and_draws`. |
| Quandrix Geologist | {G}{U} | ✅ | Push (modern_decks, NEW, `stx::extras`, batch 9, synthesised STX Quandrix): 1/3 Elf Druid. `{T}: Add {G} or {U}` + `{T}, Discard: Draw 1` looter. Test: `quandrix_geologist_can_tap_for_g_or_u`. |

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
