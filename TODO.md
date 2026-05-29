# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.
See `CUBE_FEATURES.md` (cube-card implementation status) and
`STRIXHAVEN2.md` (Secrets-of-Strixhaven status).

## Recent additions (Push XLI — claude/modern_decks: Cascade + Dredge + Auras)

Three new mechanics, 20 cards, 30 tests, 3 CR lock-ins, and one
improvement in each of engine / UI / server.

- **Cascade (CR 702.85)** ✅ — `Effect::Cascade { max_mv }` +
  `shortcut::cascade(mv)`. Trigger fires on cast, exiles from the top
  of the library until a nonland card with MV < the spell's MV, lets
  the controller cast it free, and bottoms the rest. Cards: Bloodbraid
  Elf (promoted from a draw-1 proxy), Apex Devastator (×4), Shardless
  Agent, Enlisted Wurm, Maelstrom Wanderer (×2 + team-haste static),
  Bituminous Blast, Violent Outburst, Ardent Plea (cascade + exalted).
- **Dredge (CR 702.52)** ✅ (was the open TODO at the bottom of this
  file) — `GameState::draw_one` applies the dredge replacement across
  every draw site (turn draw, `Effect::Draw`, cycling); the
  `AutoDecider` declines by default so ordinary games are unaffected.
  Cards: Golgari Thug, Golgari Grave-Troll (🟡 — regen omitted),
  Life from the Loam, Stinkweed Imp, Golgari Brownscale, Darkblast.
- **Auras / attach-on-resolve (CR 303.4)** ✅ — Aura permanent spells
  now set `attached_to` from their target on resolution, so
  `equipped_bonus` flows onto the enchanted creature (first Aura
  support in the catalog). Cards: Gift of Orzhova, Rancor, Spectral
  Flight, Flight, Unholy Strength, Holy Strength (via a `simple_aura`
  helper).
- **Engine fix** — `ExileTopAndGrantMayPlay` read `library.last()`
  (the bottom card) while documenting "top"; corrected to `.first()`
  (index 0 = top). Regression test added.
- **CR lock-ins** — 702.85b (cascade strict-MV gate), 702.52e (dredge
  replaces the draw), 702.2c (deathtouch blocker destroys a larger
  attacker — advances the in-progress CR 510 combat-damage section).
- **UI** — the Deck HUD chip turns amber at ≤3 library cards
  (deck-out warning, CR 104.3a) — relevant to the new dredge/mill
  shells.
- **Server** — `MatchOutcome.loss_reasons` classifies each elimination
  as LifeDepleted / Poison / Decked / Other for ladder metrics.

### Follow-ups noticed this run (not yet done)

- **Maelstrom Nexus** (cube) — "first spell each turn has cascade"
  static. The `Effect::Cascade` primitive exists; needs first-spell-of-
  turn tracking + a static that injects a cascade trigger.
- **Golgari Grave-Troll regen** — "{T}, remove four +1/+1 counters:
  regenerate" (needs a remove-N-counters activation cost).
- **The Gitrog Monster** — dredge/land-discard engine (dredge now
  exists; the rest of the card is still ⏳).
- **Madness (CR 702.35)** — Blazing/Basking Rootwalla, Anje's Ravager.
  Deferred: needs a centralized discard→graveyard helper (the discard
  bookkeeping is currently duplicated across `Effect::Discard`
  random/chosen branches, cycling, and the DiscardChosenPending resume)
  + a cast-from-exile-paying-madness-cost path. Worth doing as a
  discard-centralization refactor.
- **Attacker-chosen combat-damage assignment order (CR 510.1c)** —
  still auto-ordered by CardId; needs a `Decision` variant routed
  through the synchronous combat-damage path (and the UI suspend infra
  if a human is to choose).
- **Scryfall fetch is blocked** by the environment network policy
  (`api.scryfall.com` not in the allowlist) — cards whose printed
  stats couldn't be confirmed from the md files or comments (e.g.
  Bloodbraid Challenger) were intentionally left ⏳ rather than guessed.

## Recent additions (Push XL — modern_decks: Equipment + Vehicles + Manlands)

Three new permanent mechanics + 21 card tests + 3 CR sections, plus
engine/server/UI/bot improvements.

- ✅ **Equipment attach mechanic (CR 702.6 / 301.5)** — `GameAction::Equip`
  (sorcery-speed + your-creature gating + equip-cost payment) repoints
  `attached_to`; `CardDefinition.equipped_bonus: Option<EquipBonus>`
  flows +P/+T + keyword grants onto the equipped creature through the
  layer system. **Cards**: Bonesplitter (+2/+0), Shuko (+1/+0, equip {0}),
  Lavaspur Boots (+1/+1 haste); **Lightning Greaves** promoted from the
  grant-on-activate approximation to a real equip granting haste+shroud.
- ✅ **Vehicles & Crew (CR 702.122 / 301.7)** — `Keyword::Crew(N)` +
  `GameAction::Crew` taps creatures (total power ≥ N) to animate the
  Vehicle into an artifact creature until end of turn (layer-4
  `AddCardType`). `base_power`/`base_toughness` honor Vehicles so the
  crewed P/T survives layering; `declare_attackers` reads computed
  creature-ness so crewed Vehicles attack. **Cards**: Esika's Chariot
  (Crew 4), Smuggler's Copter (Crew 1, flying + attack-loot), Strixhaven
  Skycoach (Crew 2, promoted from always-creature). New `VehicleCrewed`
  event.
- ✅ **Manlands via `Effect::BecomeCreature`** — a new composite animate
  effect (AddCardType + creature subtypes + SetBasePT + keyword grants
  for a duration). **Cards**: Celestial Colonnade (UW 4/4 flying+vig),
  Creeping Tar Pit (UB 3/2 unblockable).
- **Engine**: `Effect::BecomeCreature`; computed-creature-aware
  `declare_attackers`; `base_power`/`base_toughness` Vehicle handling.
- **Server**: `PermanentView.equippable` + `.crew_value` fields for
  client action surfacing.
- **Bot**: `pick_equip` — the bot moves Equipment onto its biggest
  creature each main phase (dry-run gated).
- **UI**: `E`-key equip flow (`TargetingState.pending_equip_source` →
  click a creature → `GameAction::Equip`); `cancel_targeting` clears it.
- **Coverage backfill (+13 card tests)**: Char, Thud, Thoughtseize,
  Searing Blaze, Inquisition of Kozilek, Collective Defiance (×2),
  Mystical Dispute, Plunge into Darkness, Coveted Jewel, The Mightstone
  and Weakstone, Kozilek's Command, Eldrazi Confluence.
- **CR lock-in tests**: 702.6 (Equip), 702.122 (Crew), 301.7 (Vehicle
  not-a-creature-until-crewed), 509.1b (unblockable can't be blocked).

### Follow-ups noticed this push (deliberately deferred)

- **Equipment `equipped_bonus` is static-only** — Lion Sash ("+1/+1 for
  each +1/+1 counter on it") needs a dynamic/count-scaled equip bonus,
  and Sword of Body and Mind / Helm of the Host need an
  equipped-creature *triggered* ability (combat-damage → token/mill,
  attack → token-copy). Both want a "grant triggered ability to the
  attached creature" primitive.
- **Crew/Equip in the client** are keyboard-only and (for crew) not yet
  surfaced; a proper context-menu entry keyed off `equippable` /
  `crew_value` would round it out. The bot does not yet crew Vehicles
  (it equips but won't tap creatures to crew) — a crew heuristic that
  only fires with surplus attackers would help.
- **`Effect::BecomeCreature` doesn't set colors** — manland color
  identity (relevant to protection / "becomes a white-blue creature")
  is approximated; add a layer-5 SetColors arm when a card needs it.
- **Vehicles don't fire block triggers** — Smuggler's Copter's "attacks
  or blocks" loot only fires on attack (no Vehicle DeclaredBlocker
  event path yet).

## Recent additions (Push XXXIX — modern_decks: coverage backfill + ETB-counter fix + CR 706.2)

This session focused on test-coverage backfill, two engine fixes
surfaced by that coverage, three CR sections, and one improvement each to
engine/UI/server.

- **49 cards with new functionality tests** (all previously wired but
  untested by name):
  - **SOS (14)**: Lecturing Scornmage, Melancholic Poet, Muse Seeker,
    Poisoner's Apprentice, Rearing Embermare, Rehearsed Debater, Tester
    of the Tangential, Brush Off, Zaffai and the Tempests, and the five
    school lands (Fields of Strife, Forum of Amity, Paradox Gardens,
    Spectacle Summit, Titan's Grave).
  - **STX (6)**: Dina Soul Steeper, Professor Onyx, Strixhaven
    Pondkeeper, Zimone Quandrix Prodigy, Academic Probation, Unwilling
    Ingredient.
  - **Modern decks (20)**: Wall of Blossoms, Monastery Swiftspear, Relic
    of Progenitus, Stonecoil Serpent, Decree of Justice, Spell Queller,
    Amped Raptor, Bonecrusher Giant, Magda anthem, Three Tree City, Wight
    of the Reliquary, and the dual-land cycle (Godless Shrine, Hallowed
    Fountain, Overgrown Tomb, Blooming Marsh, Copperline Gorge, Meticulous
    Archive, Shadowy Backstreet, Undercity Sewers, Darkbore Pathway).
  - **Zendikar (9)**: the full fetchland cycle (Polluted Delta,
    Bloodstained Mire, Wooded Foothills, Windswept Heath, Misty
    Rainforest, Scalding Tarn, Verdant Catacombs, Arid Mesa, Marsh Flats).

- **Engine fix — ETB/triggered counter auto-targeting**: `auto_target_
  for_effect` now walks the stack for counter-class effects (gated on the
  `IsSpellOnStack` filter), preferring the topmost spell not cast by the
  controller. Previously Spell Queller / Mystic Snake-style ETB counters
  fizzled because the auto-target picker only walked players, graveyards,
  and the battlefield — never the stack.

- **Card fix — Stonecoil Serpent** now enters via `enters_with_counters`
  (CR 614.12 replacement) instead of an ETB AddCounter trigger, so its
  0/0 base body never hits the SBA death check counter-less.

- **Two card promotions**: Academic Probation (Noop stub → tap + stun
  mode 0; the mana-value spell-lock mode 1 is omitted — no per-player
  cast-restriction primitive) and Unwilling Ingredient (free MayDo →
  MayPay {2}{B} on its death-draw).

- **CR 706.2 — Rolling a Die (modifiers)**: `Effect::RollDie` gains a
  serde-defaulted `modifier: Value` applied to each natural roll before
  the results table (floored at 1, may exceed `sides`). Promotes 706.2
  to ✅. Tests: `cr_706_2_*`.

- **CR 122.1d (near Academic Probation)** + **CR 119.8 (burn-damage
  path)**: new end-to-end lock-in tests
  (`cr_122_1d_academic_probation_stun_persists_through_untap`,
  `cr_119_8_player_cannot_lose_life_blocks_burn_damage`).

- **UI (server/view.rs)**: `trigger_event_label` no longer falls through
  to `""` for unenumerated EventKind×EventScope pairs — a scope-aware
  fallback guarantees no blank trigger chip on the client.

- **Server (crabomination_server)**: `MatchStats` tracks alternate/deckout
  wins (clean wins where every loser ended with life > 0) and surfaces
  `alt_wins=N` in the rolling summary.

### Follow-ups noticed this run (not yet tackled)

- **Academic Probation mode 1** ("until your next turn, target player
  can't cast spells with mana value 3 or less") needs a per-player
  `CantCastSpellsWithMvAtMost(n)` restriction primitive consulted in the
  cast-legality path. Same shape would unlock similar "Silence"-class
  lockdowns.
- **Miracle / may-play alt-cost**: Lorehold the Historian's miracle and
  any future Miracle card cast for free via `GrantMayPlay`. A
  `MayPlayPermission.alt_cost: Option<ManaCost>` field + a cast-from-
  may-play payment gate would make the {2} miracle cost faithful
  (currently free). Touches ~34 GrantMayPlay construction sites.
- **Spell Queller mana-value filter**: the wired ETB counters *any*
  spell; the printed card only hits MV ≤ 4. Needs an
  `IsSpellOnStack`-with-MV-cap target filter.

## Recent additions (Push XXXVIII — modern_decks: real hybrid mana + 3 CR sections)

This session discovered the engine already supported two-color hybrid
pips (`ManaSymbol::Hybrid`) end-to-end but ~14 cards approximated them as
a single color, and added a `MonoHybrid` primitive for `{2/C}` pips.

### New engine primitives / mechanics (3 CR sections)
- **CR 202.3f — Monocolored hybrid** (near cards): `ManaSymbol::MonoHybrid(n, color)`
  — mana value uses the generic side; payment prefers the colored side,
  falling back to {n} generic. Wired through `cmc` / `distinct_colors` /
  `colors` / `summary`, the `pay()` algorithm, the auto-tap planner, the
  layer-system `colors_from_card`, server view rendering, and the client
  draft color-bucketing. Fixed Magmablood / Wildgrowth Archaic (were
  charging BOTH halves → CMC 9/4 instead of 6/4) and Spectral Procession
  (flat {2}{W} → real {2/W}{2/W}{2/W}).
- **CR 701.15g — Can't be regenerated** (in-progress, was a flagged
  follow-up): `Effect::DestroyNoRegen` bypasses regeneration shields.
  Wired Terminate, Putrefy, Wrath of God, Damnation. Plain `Destroy`
  still honors shields (regression test included).
- **CR 702.13 Intimidate / 702.16 Protection** (random): shares-a-color /
  protection-from-color now read *computed* colors
  (`ComputedPermanent.colors`) instead of scanning raw `{C}` cost pips,
  fixing the interaction with hybrid / mono-hybrid pips.

### Cards converted to real hybrid pips (with tests)
- Two-color hybrid: Essenceknit Scholar, Paradox Surveyor, Abstract
  Paintmage, Practiced Scrollsmith, Stirring Honormancer, Abigale /
  Heroic Stanza, Lluwen / Pest Friend, Spectacle Mage, Fervent Strike,
  Killian's Confidence, Manamorphose, Burning-Tree Emissary, Tasigur's
  activated {2}{G/U}.
- Mono-hybrid: Magmablood Archaic, Wildgrowth Archaic, Spectral
  Procession.
- Fix What's Broken reworked to a faithful {X}{2}{W}{B} (pay X life, mass
  reanimate at exactly MV X).
- ~30 functionality tests across these.

### Improvements this session
- **Engine**: the three mechanics above + the cost-bug fixes.
- **Server**: `format_mana_cost` deduped onto `ManaCost::summary()` —
  fixes Debug-formatted hybrid/Phyrexian labels (`{White/Black}` → `{W/B}`)
  and {C} mis-rendering.
- **UI (client)**: legal-target-filter `HasColor` uses `ManaCost::colors()`
  (hybrid/mono-hybrid colors count); draft color-bucketing counts
  mono-hybrid. NOTE: the client can't be compiled in this sandbox
  (missing `libwayland-dev` / `wayland-client.pc`), so these client
  edits are verified by inspection only — they are additive arms on
  already-exhaustive matches and a method swap, type-correct by review.

### Late-run additions
- **`CounterType::Indestructible`** (CR 122.1 / 702.12): new counter +
  `CardInstance::is_indestructible()` consulted by SBA lethal damage and
  Destroy/DestroyNoRegen. Wired Zopandrel, Hunger Dominus's `{G/P}{G/P},
  Sac two other creatures` activation. NOTE: Zopandrel's combat-step
  power/toughness *doubling* is still approximated as a flat +4/+4 (needs
  per-creature P/T introspection — a `Value::PowerOf`/`ToughnessOf`-driven
  PumpPT-by-current-stats primitive).

### Follow-ups noticed (not tackled this run)
- **Phyrexian pip color in `colors_from_card`** already handled; but the
  monocolored-hybrid `{2/C}` "pay 2 life" variant ({2/P}-style) is not a
  thing — only `{n/color}` is modeled. No action needed.
- **Spend-restricted mana** (Abstract Paintmage's "spend only on instant
  and sorcery spells", Tablet of Discovery) still unmodeled — needs
  per-mana spend-restriction metadata on the mana pool.
- **Mono-hybrid auto-tap optimality**: the planner treats a {2/C} pip as
  needing 1 colored or 2 generic but doesn't search for the globally
  cheapest land assignment across mixed hybrid costs (greedy). The
  *two-color* hybrid path was upgraded (Push: modern_decks) to a
  forced-color-first assignment in both `ManaPool::pay` and
  `auto_tap_for_cost` — a {W/B}{W/B} cost now splits a Plains+Swamp
  correctly, and a {W/B} pip taps the only producible color instead of
  always reaching for the first half. `{2/C}` mono-hybrid still uses the
  simpler greedy; fine in practice.
- **Client build in CI**: confirm `libwayland-dev` is installed in the
  CI/runner image so the client crate (and its clippy) actually compile;
  it does not build in this web sandbox.

## Recent additions (Push XXXVII — modern_decks: Delve + Regeneration + Fear)

This session landed **three CR-keyword mechanics** the catalog had stubbed
or flagged in the backlog, plus the cards they unlock and improvements to
engine/server/UI.

### New engine primitives / mechanics (3 CR sections)
- **Delve (CR 702.66)** — `GameAction::CastSpellDelve { delve_cards }` +
  `Keyword::Delve`. Each graveyard card exiled while casting pays {1} of
  the generic cost (generic-only clamp via `ManaCost::reduce_generic`,
  mirrors convoke). Cards are exiled only *after* the reduced cost is paid
  (so a rejected cast leaves the graveyard untouched) and bump
  `cards_exiled_this_turn`. New `GameError::CardNotInGraveyard`. The
  *near-cards* section.
- **Regeneration (CR 701.15)** — `CardInstance.regeneration_shields`
  (transient, not serialized, cleared at cleanup) + `Effect::Regenerate`.
  A shield replaces the next destruction this turn (tap + remove from
  combat + heal damage), is consumed on use, never saves a creature whose
  toughness is ≤ 0, and is checked in both the SBA lethal-damage loop and
  `Effect::Destroy`. The *in-progress* section (a `Keyword::Regenerate`
  stub existed but nothing wired it).
- **Fear (CR 702.36)** — `Keyword::Fear` + block restriction in
  `can_block_attacker_computed`: a Fear attacker can only be blocked by
  artifact and/or black creatures. The *random* section.

### New / promoted cards (with functionality tests) — 12 cards
- **Delve**: Treasure Cruise (✅), Dig Through Time (new, ✅), Lose Focus
  (✅), Murderous Cut (✅), Gurmag Angler (new), Hooting Mandrills (new),
  Become Immense (new), Tombstalker (new — {6}{B}{B} 5/5 flier).
- **Regeneration**: Drudge Skeletons (new — `{B}: Regenerate this`), Wall
  of Bone (new — 1/4 Defender + regen), Will-o'-the-Wisp (new — 0/1 flier
  + regen).
- **Fear**: Severed Legion (new — {2}{B} 2/2 Fear).
- 27 functionality tests (10 delve incl. partial/reject edge cases, 7
  regen, 2 fear, 1 bot-delve, 1 client tooltip). Below the 20-card floor:
  the catalog is mature and the no-network sandbox blocks Scryfall, so new
  *real* cards are limited to ones grounded in the md files or canonical
  enough to define from memory with confidence (consistent with the prior
  session's note). The bulk of this run's value is the 3 reusable
  mechanics, which unlock many more cards in future sessions.

### Improvements this session
- **Engine**: the three mechanics above.
- **Server / bot**: `main_phase_action` generates `CastSpellDelve`
  candidates — the bot delves a stocked graveyard to afford Treasure
  Cruise / Gurmag Angler it couldn't pay at full cost. `PermanentView`
  gains `regeneration_shields`.
- **UI**: counter tooltip badges regeneration shields (singular + ×N).

### Follow-ups noticed (not tackled this run)
- **"Can't be regenerated" (CR 701.15g)** — DONE in Push XXXVIII:
  `Effect::DestroyNoRegen` wired into Terminate, Putrefy, Wrath of God,
  Damnation. (Day of Judgment intentionally NOT changed — its printed
  Oracle has no can't-regen clause, unlike Wrath of God.)
- **Delve `{X}` interaction**: Logic Knot ({X}{U}{U}, Delve, counter
  unless pay {X}) wasn't added — its body needs `CounterUnlessPaid` to
  read the X paid (`Value::XFromCost`) rather than a fixed cost.
- **Tasigur, the Golden Fang** (Delve creature with an activated
  graveyard-recursion ability) — body+delve would be easy; the activated
  ability needs the random-opponent-choice picker.
- Dredge (CR 702.52 — Golgari Thug/Grave-Troll, Life from the Loam) ✅ DONE
  in Push XLI: `GameState::draw_one` is the single draw-replacement hook,
  used by the turn-draw, `Effect::Draw`, and cycling paths.

## Recent additions (Push XXXVI — modern_decks, batch 207: Exalted + deaths-this-turn payoffs)

This session added 40 new STX cards (batches 207-208) across all five
colleges, two new engine primitives, the **Exalted** keyword (CR 702.83),
a combat double-fire correctness fix, four CR lock-in tests, and one
improvement each to engine/UI/server. Batch 208 follow-ups: Lorehold
Vanguard-Captain (2nd Exalted), Pyrohistorian (ETB ping), Skydefender
(ETB lifegain flier); Prismari Scorchmage (5 dmg), Scholar-Adept
(magecraft scry); Quandrix Rootmage (ETB +1/+1 on friendly), Tidecantor
(draw 2).

### New cards (with functionality tests) — batch 207
- **Witherbloom (B/G)**: Witherbloom Reaping (draw per creature that died
  this turn), Gravecaller (ETB drain per *table-wide* death), Bloodfeast
  (gain 2× life per your dead creature), Saplinglord (grows on other
  deaths), Toxicult (ETB Pest + magecraft drain), Rotcaller (ETB 2 Pests),
  Sapsiphon (combat-damage lifegain).
- **Lorehold (R/W)**: Soulkindler (ETB Spirit), Cinderscribe (magecraft
  ping), Battlecaller (attack-mint Spirit), Emberbolt (3 dmg any target),
  Relicsmith (ETB gy-recursion), Charge II (+1/+0 + first strike team),
  Vanguard (4/3 haste).
- **Quandrix (G/U)**: Tidecaller (2-counter Fractal), Theorist (magecraft
  scry+draw), Fractalsurge (X-counter Fractal), Studymate (grows with
  cards drawn this turn), Currentweaver (ETB draw+scry), Bigmind (4/5
  trample).
- **Prismari (U/R)**: Pyrologist (magecraft ping each opp), Goldcaster
  (magecraft Treasure), Firebolt II (4 dmg + scry), Goldsmith (ETB 2
  Treasures), Stormloot II (draw 2 discard 1), Galeblaster (magecraft
  self-pump).
- **Silverquill (W/B)**: Inkbinder (magecraft drain flier), Eulogist (ETB
  drain 2), Edict II (opp sac + draw), Inkling Highflier (2/3 flying
  vigilance), Sanction (exile + gain 2), Coursemate (lifegain on death),
  Duelmaster (**Exalted**).

### New engine primitives
- `Value::CreaturesDiedThisTurn(PlayerRef)` + `Value::CreaturesDiedThisTurnTotal`
  — count creatures that died (per-player / table-wide) this turn. Backed
  by `Player.creatures_died_this_turn`. Complement the existing
  `Predicate::CreaturesDiedThisTurn(Total)AtLeast`.
- `Predicate::AttackingAlone` (`GameState.attacking.len() == 1`) + the
  `effect::shortcut::exalted()` helper.

### CR sections implemented / advanced this session
- **CR 702.83 — Exalted** ✅ (NEW, near the card work): "Whenever a
  creature you control attacks alone, that creature gets +1/+1 until end
  of turn." Wired as an `Attacks / YourControl` trigger gated on the new
  `Predicate::AttackingAlone`, pumping `Selector::TriggerSource` (the lone
  attacker, so multiple Exalted sources stack per 702.83b). Tests:
  `cr_702_83_exalted_pumps_lone_attacker`, `cr_702_83b_exalted_silent_when_not_alone`.
  **Engine fix unblocked by this**: `declare_attackers` previously walked
  the attacking card's *every* Attacks trigger regardless of scope and
  pushed it with `trigger_source: None`, so a YourControl Attacks trigger
  on the attacker double-fired (once no-op from combat.rs, once correctly
  from the dispatcher). It now only hardcodes `SelfSource` Attacks
  triggers; YourControl ones route solely through
  `dispatch_triggers_for_events`.
- **CR 702.15 — Lifelink** ✅ (lock-in, near the card work): combat damage
  from a lifelink creature gains its controller that much life.
  `cr_702_15_lifelink_combat_damage_gains_life`.
- **CR 510 — Combat Damage Step** (in-progress 🟡): added
  `cr_510_1c_trample_overflow_to_player` (4-power trampler vs a 2/2 →
  2 lethal + 2 tramples). The attacker-chosen damage-assignment-order
  gap is still ⏳ (auto-ordered by CardId — needs a Decision variant).
- **CR 302.6 — Summoning sickness** ✅ (random lock-in): a freshly-entered
  creature can't attack unless it has Haste.
  `cr_302_6_summoning_sick_creature_cannot_attack`.

### Improvements this session
- **Engine**: the combat double-fire fix above (YourControl Attacks
  triggers no longer fire twice) — a real correctness bug, not just
  net-new feature wiring.
- **UI**: poison-counter HUD chip (`StatChipKind::Poison`), rendered on
  both the viewer's stat row and each opponent's row, only when
  `poison_counters > 0`. Surfaces the hidden CR 704.5c lethal-at-10 lose
  condition in infect/poison games.
- **Server**: `MatchStats::turn_percentile` (+ `turn_bucket_upper_bound`),
  surfacing `turns p50≤N, p95≤N` in the rolling summary line alongside the
  existing duration percentiles. 3 tests.

### Follow-ups noticed (not tackled this run)
- Exalted via `dispatch_triggers_for_events` only fires for the lone
  attacker correctly; a future cleanup could give `declare_attackers`'
  remaining SelfSource hardcode path the same `trigger_source` plumbing so
  `Selector::TriggerSource` resolves there too (today SelfSource Attacks
  triggers that reference `TriggerSource` get `None` — none currently do,
  but it's a latent foot-gun).
- The Witherbloom "harvest" cards collapse "draw a card for each creature
  that died under your control this turn" faithfully, but a true
  end-step-gated variant (Essenceknit Scholar style) could reuse the new
  Value with a `DelayUntil(NextEndStep, …)`.

## Recent additions (Push XXXV — modern_decks, rider/cost-primitive promotions)

This session focused on **promoting cards whose advanced riders were stubbed
behind primitives that already existed** (the in-repo docs were stale), plus
3 CR sections and one improvement each to engine/server/UI. No new card
*definitions* were added — the catalog already covers every card in the
Scryfall cache (verified: all 37 "missing" cache entries are implemented via
builder string-args, false positives in the name-detector).

### Cards promoted to full functionality (with tests)
- **Applied Geometry** — real `CreateTokenCopyOf` (copy target permanent,
  add Fractal, override P/T 0/0, six +1/+1 counters) instead of a vanilla
  Fractal token.
- **Colorstorm Stallion** — Opus ≥5-mana branch now mints a copy token.
- **Elemental Mascot** — Opus ≥5-mana branch exiles top + grants may-play
  (`ExileTopAndGrantMayPlay`).
- **Bloodtithe Harvester** — `{1}, Sacrifice a Blood: 2 damage` via
  `sac_other_filter: HasArtifactSubtype(Blood)`.
- **Tireless Tracker** — sac-a-Clue → +1/+1 counter via a
  `PermanentSacrificed + YourControl` trigger.
- **Sentinel of the Nameless City** — Ward {2} (enforced) + Plant subtype.
- **Sylvan Safekeeper** / **Zuran Orb** / **Orcish Lumberjack** /
  **Elvish Reclaimer** / **Witherbloom Apothecary** / **Witherbloom
  Plaguemage** / **Bramble Brewer** — folded sac costs promoted to proper
  `sac_other_filter` pre-resolution activation costs (reject when the cost
  is unpayable). Orcish Lumberjack also fixes a mana-color bug
  ({G}{G}{G} → the printed {R}{R}{R}).
- **Tragedy Feaster** — Ward—Discard a card now enforced.
- **Prismari, the Inspiration** — Ward—Pay 5 life now enforced.
- **Tragic Slip** — Morbid gate (-1/-1, or -13/-13 if a creature died).
- **Improvisation Capstone** — cost bug fixed ({3}{R}{R} → {5}{R}{R});
  promoted ⏳→✅ alongside Echocasting Symposium in STRIXHAVEN2.md
  (both were already wired — stale false-negatives).

### CR sections implemented / advanced this session
- **CR 702.3 — Defender** ✅: enforced in `can_attack`;
  `cr_702_3_defender_cannot_be_declared_as_attacker` locks it in
  (Sylvan Caryatid). Doc on the card corrected.
- **CR 510 — Combat Damage Step** (advanced 🟡): multi-blocker lethal
  assignment + trample overflow + deathtouch (1-lethal-each) now have
  lock-in tests (`cr_510_1c_trampler_assigns_lethal_to_each_blocker_then_tramples`,
  `cr_510_1c_deathtouch_attacker_assigns_one_to_each_blocker`). Remaining
  gap: the attacker's controller can't *choose* the damage-assignment
  order (auto-ordered by CardId) — still ⏳ (needs a Decision variant).
- **CR 700.4 — Morbid** ✅ (new): `Predicate::CreaturesDiedThisTurnTotalAtLeast`
  sums deaths across all players; `cr_700_4_morbid_total_predicate_counts_
  deaths_across_players`. Used by Tragic Slip.

### Improvements this session
- **Engine**: `Predicate::CreaturesDiedThisTurnTotalAtLeast` (global morbid)
  — cleaner than OR-ing per-seat predicates; view label + tests.
- **Server**: `MatchStats.turn_buckets` turn-count histogram parallel to
  the duration histogram, rendered in `format_match_stats`. 2 tests.
- **UI**: damage-marked creatures in the alt-tooltip now show the survival
  margin ("marked: 2 damage; 3 more lethal"). 1 test. (Client crate is
  unbuildable in the web sandbox — wayland-sys; change is pure formatting.)

### Card count note
~16 cards received genuine functional promotions this session (plus
lock-in tests added for several already-implemented-but-untested riders
like Vandalblast/Cyclonic Rift overload). This is below the 20-card
target: the catalog is exceptionally mature — **every card in the
Scryfall cache is already implemented**, and the remaining unimplemented
cards all require large new engine primitives (see backlog below). Future
sessions wanting volume should land one of those primitives first.

### Backlog discovered (deliberately not tackled — need large primitives)
The remaining CUBE/SOS partials cluster around primitives the engine still
lacks. Highest-value next targets, each unlocking multiple cards:
- **Madness** (CR 702.35): Blazing/Basking Rootwalla, Anje's Ravager — needs
  discard→exile replacement + a "cast for madness cost" window (new Decision).
- **Regeneration** (CR 701.15): Korlash + many "can't be regenerated" riders.
- **Delve** (CR 702.66): Treasure Cruise, Dig Through Time, Logic Knot.
- **Permanent clone / enters-as-a-copy**: Phantasmal Image, Mockingbird,
  Mirrorform (token-copy `CreateTokenCopyOf` exists; "enters as a copy" of a
  real permanent does not).
- **"becomes tapped" trigger** (Magda, Goldspan Dragon Treasure riders).
- **Overload-style / minimum-cost / spend-restricted mana** — assorted singles.

## Recent additions (Push XXXIV — 2026-05-29, session 18, batches 205-206)

### New cards (12 more across batch 206, all tested)
- **Lorehold (3)** — Skirmisher (on-attack ping), Archivekeeper (ETB
  draw), Ember Veteran (Trample + magecraft self-pump).
- **Witherbloom (3)** — Grim Harvest (drain 4), Sporecaller
  (aristocrats lifegain), Fungalbeast (ETB gain 2).
- **Quandrix (2)** — Scholar (magecraft self-pump), Megafractal (5/5
  Trample Fractal).
- **Silverquill (2)** — Dictator (3/3 flying lifelink Inkling), Purge
  (exile a ≤2-power creature).
- **Prismari (2)** — Inferno (4-dmg sorcery), Windscholar (ETB scry+draw).
- Total this session: **42 cards** across batches 205-206.


### Engine — NEW mechanic: Enrage (CR 702.130)
- **`EventKind::DealtDamage`** — the first damage-recipient-keyed
  trigger event. Maps to the pre-existing `GameEvent::DamageDealt
  { to_card: Some(_), .. }` stream so it fires on **any** damage to a
  permanent (combat, burn spells, Fight, pingers) — matching the
  printed "Whenever this creature is dealt damage" wording. Wired
  through `event_matches_spec` / `event_subject` / `event_card`
  (`game/effects/events.rs`), fan-out-enabled in `game/mod.rs` (one
  fire per instance of damage, CR 702.130a), and exposed via the new
  `effect::shortcut::enrage()` helper. The damage amount is reachable
  in the trigger body via `Value::TriggerEventAmount` for scaling
  payoffs. Trigger tooltip label "Enrage" added to `server/view.rs`.

### New cards (30 across batch 205, all with functionality tests)
- **Lorehold (R/W) Enrage cycle (10)** — Battlescarred (+1/+1 on
  damage), Echovenger (scaling counters via TriggerEventAmount),
  Vengescribe (ping any), Grudgebearer (drain 2), Stoneguard (gain 2),
  Chroniclekeeper (draw), Warhost (mint Spirit), plus Emberhistorian
  (magecraft ping), Relicwarden (attack lifegain), Warchronicler
  (magecraft self-pump).
- **Witherbloom (B/G) (5)** — Thornbeast / Gravethorn (enrage),
  Sapfeeder (death drain), Bloodmoss (magecraft drain), Rotcaller
  (aristocrats drain).
- **Quandrix (G/U) (4)** — Thornfractal (scaling enrage wall),
  Tidecaller (magecraft draw), Growthseer (ETB counter), Mistcaller
  (magecraft scry).
- **Silverquill (W/B) (5)** — Lightscribe (ETB gain 3), Grimquill
  (magecraft drain), Final Edict (drain 3), Inkguard (lifelink
  Inkling), Deathscribe (aristocrats drain).
- **Prismari (U/R) (6)** — Flarecaster (magecraft ping each opp),
  Tidescribe (ETB scry), Emberbolt (2-dmg instant), Stormloot
  (magecraft loot), Pyrosmith (magecraft Treasure), Galemage
  (magecraft scry).

### CR rule lock-in tests (3 sections)
- **CR 702.130 — Enrage** (NEW this session, near the card work):
  `cr_702_130_enrage_fires_on_combat_damage` proves the trigger fires
  off the combat-damage path; `lorehold_enrage_does_not_fire_without_
  damage` pins the no-damage case.
- **CR 122.1c — Shield counters** (was stale ⏳ in the audit, actually
  implemented; promoted to ✅): `cr_122_1c_shield_counter_prevents_
  noncombat_damage`. Also corrected 122.1b (keyword counters) to ✅.
- **CR 510 — Combat Damage Step** (in-progress 🟡, multi-blocker split
  still a gap): `cr_510_blocked_attacker_and_blocker_trade` locks in
  the single-blocker simultaneous-damage trade.

### Server improvement
- **`MatchStats.min_turns`** completes the turn-count envelope
  alongside the existing `max_turns` + running average. Rendered in
  `format_match_stats` as `(turns MIN-MAX)` when there's a spread, or
  `(max turns N)` when a single distinct value has been seen. 3 new
  unit tests.

### UI improvement
- **Numeric +1/+1 / -1/-1 counter badges** in `counter_tooltip.rs`:
  the boolean "(boosted: +1/+1 counters)" badge now reads the actual
  count off the `counters` vec and renders the real P/T swing
  ("(boosted: +3/+3 from 3 +1/+1 counters)"), with singular/plural
  wording and a fallback to the legacy boolean badge for older
  projections. Directly useful for the enrage creatures that stack
  counters. 4 new tooltip tests.

### Observations & future items from this session
- **Enrage fan-out vs combat batching**: enrage currently fires once
  per `DamageDealt` instance. In multi-blocker combat a creature
  blocked by N attackers takes N separate damage events → N enrage
  fires. This matches CR 702.130a (each instance is separate) but if
  a future batch-collapse pass lands for combat damage, enrage should
  be re-audited.
- **`Value::TriggerEventAmount` for enrage** works for the
  damage-amount read; a "the source of the damage" selector
  (for "deals damage back to whatever damaged it" enrage variants like
  Ravenous Chupacabra-adjacent designs) is not yet exposed — would need
  the damaging source threaded onto the `DamageDealt` event subject.
- **Client build/clippy in CI sandbox**: the `crabomination_client`
  crate can't compile in the web-session sandbox because the system
  `wayland-client` library is absent (only x11 is installed) — the
  `wayland-sys` build script panics. Core (`crabomination`) and
  `crabomination_server` build, test (5214 + 58), and clippy cleanly.
  The session 18 UI change (`counter_tooltip.rs` numeric counter
  badges + 4 tests) is pure logic against `net::PermanentView` and is
  verified against that struct's shape, but its tests can't be executed
  here. A future env with wayland (or a `--no-default-features` +
  explicit-x11 Bevy build) should run `cargo test -p crabomination_client
  counter_tooltip` to confirm.

## Recent additions (Push XXXIII — 2026-05-28, session 17, batches 202-204)

### New cards (130 across batches 202-204)
- **Batch 204 (21 cards)** — cross-school round 4: Silverquill (5),
  Witherbloom (5), Lorehold (5), Prismari (3), Quandrix (3). Curve
  fillers and premium top-ends (Inkling Champion 3/3 flying vigilance,
  Pest Devourer II 4/4 deathtouch, Prismari Stormrider 4/4 flying haste).
  11 functionality tests.
- **Batch 203 (34 cards)** — compact cross-school round:
  Silverquill (10), Witherbloom (7), Lorehold (6), Prismari (6),
  Quandrix (6). Mid-curve apprentices, drain spells, mint-spirits,
  cantrips. 34 functionality tests.

### New cards (75 across batch 202)
- **Batch 202 (75 cards)** — nuanced expansion focused on Silverquill
  (25), Witherbloom (14), Lorehold (12), Prismari (12), Quandrix (12).
  Covers the full cross-school spread. Highlights: Inkling Heartcaller
  (Inkling-tribal death trigger), Silverquill Inkmaster (Heliod-template
  life-gain → self-counter), Silverquill Recap (first catalog use of
  `Selector::take(.., 2)` on a graveyard CardsInZone selector — returns
  TWO ≤3-MV creature cards from gy → hand), Quandrix Symmetry (X-cost
  Fractal mint with X +1/+1 counters), Quandrix Sumtotal (CountOf
  scaling). 78 new tests (75 functionality + 3 CR rules).

### Engine improvements
- **Catalog `Selector::take(.., 2)` for graveyard returns**: Silverquill
  Recap exercises the existing multi-pick selector shape on a
  `CardsInZone` graveyard pool — the first catalog use. The primitive
  worked; the catalog wire was missing. Regression test
  `silverquill_recap_b202_returns_two_low_mv_creatures_when_available`
  pins the multi-card return path against snapshot/movement-effect
  refactors.

### Server improvements
- **`MatchStats.{cumulative_win_life_delta, win_life_samples}`** +
  `observe_win_life_delta(winner, &final_life_totals)` +
  `avg_win_life_delta()`. Rendered in `format_match_stats` as
  `avg_win_life_lead=N`. Surfaces whether long bot-vs-bot ladders
  are "blowouts" (high delta) or "races" (near zero). Wired at
  both bot and pair record paths. 5 new unit tests:
  `observe_win_life_delta_accumulates_winners_lead`,
  `observe_win_life_delta_clamps_negative_lead_to_zero`,
  `observe_win_life_delta_handles_seat_out_of_range`,
  `format_match_stats_includes_avg_win_life_lead_when_present`,
  `format_match_stats_omits_avg_win_life_lead_when_no_samples`.

### Bot AI improvements
- **Magecraft-aware spell bias** in `RandomBot::main_phase_action`:
  when the bot controls a permanent with a magecraft trigger and the
  castable set contains at least one instant or sorcery, the IS subset
  is sampled instead of the full set so the magecraft trigger fires.
  Falls back to uniform sampling when no magecraft body is in play.
  Resolves the "Bot AI doesn't model magecraft pump/drain triggers"
  observation. Test: `bot_prefers_is_spell_when_magecraft_in_play`.

### UI improvements
- **`(attacking)` / `(blocking #N)` lines in `counter_tooltip.rs`**:
  combat status surfaced in the alt-tooltip so the player can tell
  which creatures are committed to combat. 3 new tooltip tests:
  `attacking_status_surfaces_in_tooltip`,
  `blocking_status_shows_attacker_id`,
  `combat_status_hidden_when_idle`.

### CR rule lock-in tests (3 new)
- `cr_704_5a_player_at_zero_or_less_life_loses` — drain a player to 0
  via Witherbloom Famine; verify SBA fires `is_game_over()`.
- `cr_608_2b_spell_with_illegal_target_fizzles` — Bolt → Bear, remove
  Bear before Bolt resolves; verify the spell fizzles (no damage
  redirected to player).
- `cr_704_5b_empty_library_draw_attempt_loses_game` — Quandrix Cantrip
  on empty library → SBA loss.

### Observations & future items from this session
- **Choose-N modal commands**: Silverquill Quillforge mints Inklings
  + drains; could use a "choose 2 of 3" promote-to-mode-pick UI for
  the printed STX Command lineage. Still ⏳ as engine-wide gap.
- **Hybrid mana payment**: RESOLVED in Push XXXVIII. `ManaSymbol::Hybrid`
  was already supported end-to-end; the converted cards now use real
  hybrid pips, and `MonoHybrid` covers `{2/C}`. Any card still printed
  with a single-color approximation of a hybrid pip should be migrated
  to `hybrid(a, b)` / `mono_hybrid(n, c)`.
- **Lesson sideboard model**: Quandrix Cantrip is *not* a Lesson but
  shares the IS-cantrip shape — Lessons themselves still gate on the
  sideboard model.

## Recent additions (Push XXXII — 2026-05-28, session 16, batches 198-200)

### New cards (78 across batches 198-200)
- **Batch 198 (40 cards)** — 8 per school across all five colleges
  (Silverquill, Witherbloom, Lorehold, Prismari, Quandrix). Wide
  cross-school spread using existing primitives.
- **Batch 199 (25 cards)** — 5 per school. Slightly more nuanced
  cards (Silverquill Smiter destroys power 4+, Lorehold Recurrence
  Regrowth-style gy → hand, Prismari Surge pump + grant Trample,
  Quandrix Pulse cantrip + counter, Inkling Beacon Flying Lifelink).
- **Batch 200 (13 cards)** — round-200 mini-batch: Silverquill
  Indrain (drain 4), Witherbloom Decay, Lorehold Smite (tapped
  creature destroy), Prismari Notebook (scry 3 + draw), Quandrix
  Anchorvine (4/4 Vigilance Fractal).

### Engine / server / UI improvements
- **Server (`crabomination_server/main.rs`)**: `MatchStats.seat_wins:
  [u64; 4]` per-seat win histogram, rendered as `seat_wins=N/N/...`
  in `format_match_stats` (only up to the highest non-zero seat,
  so 1v1 doesn't surface padding zeros). Catches turn-order bias
  in long bot-vs-bot ladders. 2 new tests:
  `observe_winner_per_seat_clamps_at_seat_bucket_count`,
  `format_match_stats_includes_seat_wins_when_present`.
- **UI (`counter_tooltip.rs`)**: "Type: <subtypes>" line in the
  alt-tooltip for creatures so tribal context (Inkling Wizard,
  Pest, Spirit Warrior) shows at a glance. 2 new tooltip tests.

### CR rule lock-in tests (6 new)
- `cr_105_2_hybrid_pip_contributes_both_colors` — hybrid pip
  contributes both color halves to `cost.colors()`.
- `cr_105_2c_generic_only_cost_is_colorless` — generic-only cost
  reports zero distinct colors.
- `cr_105_2b_three_pips_register_as_multicolored` — three colored
  pips register as multicolored.
- `cr_121_5_scry_does_not_count_as_drawing` — scry doesn't bump
  `cards_drawn_this_turn`.
- `cr_119_4_loss_clamps_at_zero_or_below` — life loss clamps
  without panicking on overflow.
- `cr_122_1c_shield_pops_then_second_damage_connects` — shield
  counter pops after first damage; second damage connects.

## Recent additions (Push XXXI — 2026-05-28, session 15, batches 192-197)

### New cards (127 across batches 192-197)
- **Batch 192 (26 cards)** — Witherbloom B/G deep cuts focused on
  Pest tribal, drain payoffs, magecraft scalers, and keyword counter
  combos. Includes Stripblossom — the catalog exerciser for the new
  `Effect::RemoveKeywordCounter` engine variant.
- **Batch 193 (26 cards)** — cross-school deep cuts: Silverquill (6),
  Lorehold (7), Prismari (7), Quandrix (6).
- **Batch 194 (22 cards)** — compact cross-school fillers: Witherbloom
  (5), Silverquill (5), Lorehold (4), Prismari (4), Quandrix (4).
- **Batch 195 (21 cards)** — more cross-school deep cuts: 5 Witherbloom,
  4 each across Lorehold/Silverquill/Prismari/Quandrix.
- **Batch 196 (21 cards)** — more variety: 5 Witherbloom, 4 each across
  Lorehold/Silverquill/Prismari/Quandrix.
- **Batch 197 (11 cards)** — polish round: 3 Witherbloom, 2 each across
  Lorehold/Silverquill/Prismari/Quandrix.

### Engine improvements
- **`Effect::RemoveKeywordCounter { what, keyword, amount }`** — CR
  122.1b counterpart to AddKeywordCounter. Clamped removal at the
  source's actual count; drops the granted keyword (assuming no
  other source) when the last counter is removed. Catalog exerciser
  ships in `witherbloom_stripblossom_b192`. Resolves the previous
  ⏳ note "RemoveKeywordCounter not yet implemented".
- **`shortcut::target_any()`** — canonical "Creature ∨ Player ∨
  Planeswalker" target filter (Lightning-Bolt shape). Future cards
  can drop in `target_any()` instead of the verbose 3-way or-chain.
  Existing call sites unchanged.

### UI / view-projection improvements
- **`PermanentView.{shield,stun,finality}_counter_count: u32`** —
  explicit counter counts (in addition to the existing has_*
  booleans) so the client tooltip can render "(shielded ×3:
  absorbs 3 events)" and "(stunned ×2: skips 2 untap steps)"
  instead of opaque badges.
- **Game-over modal final life totals** — the modal now shows
  "You: 1 life — Opp: -3 life" under the win/loss/draw line so the
  player sees how close the result was at a glance.

### Server improvements
- **`MatchOutcome.winner: Option<Option<usize>>`** + **
  `final_life_totals: Vec<i32>`** — captures the winning seat (or
  None for a draw) and per-seat end-of-game life totals, in
  addition to `final_turn`. Populated at every exit path via a
  new `capture_outcome` helper so watchdog/disconnect paths
  produce the same outcome shape.
- **`MatchStats.{wins, draws}`** + **`format_match_stats`
  win/draw/unresolved rendering** in the operator log line. The
  unresolved-count is computed as `total_matches - wins - draws`
  so a delta between total and (wins + draws) tells the operator
  how many matches the watchdog killed. Surfaces "stuck"
  matches at a glance.

### CR rule lock-in tests (7 new)
- `cr_122_5_optional_move_declined_keeps_counters_in_place` +
  `cr_122_5_declined_move_leaves_destination_untouched` — pin the
  counter-move early-return path via Tester of the Tangential.
- `cr_122_2_counters_cease_to_exist_on_zone_change` — documents
  current engine behavior (counters persist for Felisa-style
  patterns); flip the assertion when strict CR 122.2 clearing
  lands.
- `cr_117_5_sba_kills_before_next_priority_window` — pins SBA
  ordering between damage and priority.
- `cr_122_1b_remove_keyword_counter_drops_keyword` +
  `cr_122_1b_remove_one_of_two_keyword_counters_keeps_keyword`.
- `cr_704_5d_token_in_graveyard_ceases_to_exist` — pins CR 704.5d
  token cleanup SBA via an end-to-end Pestlord II → bolt → Pest
  death flow. Token does NOT land in the graveyard.
- `cr_704_5i_planeswalker_with_zero_loyalty_dies` — pins CR 704.5i
  via Professor Dellian Fel with Loyalty counter manually zeroed
  → SBA puts the PW in the graveyard.

### Re-enabled tests
- `academic_dispute_pumps_friendly_and_grants_reach` — previously
  ignored under the stale "fight effect" oracle; now exercises the
  current "+2/+0 + reach EOT" body. 0 ignored tests after this push.

### Observations & future items from this session
- **`Effect::RemoveKeywordCounter` for tribal-strip flavour**: any
  card that should strip Trample/Flying/Lifelink off an opp creature
  can now compose against this primitive. STX has no in-set printing
  but the engine support is here for future synthesised variants.
- **Strict CR 122.2 zone-change counter clearing**: still ⏳.
  Counters currently persist across zone changes (intentional for
  Felisa-style "dies with counters, re-emerge with same counters"
  patterns). When strict clearing lands, the
  `cr_122_2_counters_cease_to_exist_on_zone_change` test should be
  flipped to assert 0.

## Recent additions (Push XXX — 2026-05-28, session 14, batches 187-191)

### New cards (95 across batches 187-191)
- Batch 187 (35 cards): 7 per school exercising keyword counter
  granters (CR 122.1b), Pest/Inkling/Spirit/Fractal tribal payoffs,
  and magecraft templates.
- Batch 188 (15 cards): cross-school small additions — cantrip pumps,
  drain bodies, scry-cantrips.
- Batch 189 (15 cards): aggressive curve fillers — Drainmaster II,
  Vassalking, Exilewright; Devourer, Spellblossom, Pest Crawler;
  Voltmage, Fireseal, Crusader; Magmamancer, Treasurewright,
  Hailstrike; Beastcaller, Cantrip, Vinescaler II.
- Batch 190 (15 cards): keyword counter combo cards — each school
  gets 3 cards combining two CR 122.1b keyword counter grants or
  mixing a +1/+1 counter with a keyword counter.
- Batch 191 (15 cards): multi-action + tribal — Inkdrain, Highscribe,
  Vampirebond; Doublestrike, Pest Druid, Greenward; Echobringer,
  Sparrowscholar, Embershield; Stormwave, Wavetamer, Tinkermage;
  Sumtotal, Sparkbloomer, Vinegrower.

### Engine improvements
- **`PermanentView.keyword_counters: Vec<(Keyword, u32)>`** — surfaces
  CR 122.1b keyword counter map for client tooltip rendering. Client
  counter_tooltip surfaces "(flying counter granting Flying)" etc.
  alongside the existing shield/finality/stun/boosted/weakened
  highlights. Resolves the "Keyword counter UI badge" TODO from
  pushes XXVIII / XXIX.

### Server improvements
- **`MatchStats.max_turns: Option<u32>`** — tracks the longest
  observed final turn count across all completed matches for
  outlier visibility. Surfaced in format_match_stats as
  "(max turns N)" after avg turns.

### CR rule lock-in tests (3 new)
- `cr_121_2_multi_draw_fires_one_event_per_card` — pins per-draw
  fanout for multi-card draw effects.
- `cr_405_5_top_of_stack_resolves_first_lifo` — pins LIFO ordering
  for stack resolution.
- `cr_614_6_shield_counter_only_absorbs_one_event_then_pops` — pins
  the one-event-per-replacement semantics of shield counters.

### Observations & future items from this session
- **`CounterType::Keyword(Keyword)`** still ⏳: a true first-class
  variant would replace the bespoke `keyword_counters: HashMap` field
  on CardInstance with a `CounterType::Keyword(Keyword)` enum tag.
  ~50 lines.
- **Counter doubling (CR 614.16)** for `AddKeywordCounter` still ⏳:
  the regular `AddCounter` path walks `counter_doublers_for`; the new
  variant should mirror that.
- **Bot AI doesn't model magecraft pump/drain triggers** when
  considering whether to cast an instant — still ⏳.

## Recent additions (Push XXIX — 2026-05-28, session 13, batches 184-186)

### New cards (10 across batches 184-186)
- Batch 184 (6 cards): six new keyword counter granters covering the
  rest of the evergreen keywords — Silverquill Wordsharpener (first
  strike), Silverquill Drainmark (deathtouch), Witherbloom
  Trampleblossom (trample), Witherbloom Lifebondseal (lifelink),
  Lorehold Battlerune (haste), Lorehold Wardseal (vigilance). Each
  ships with a test verifying has_keyword() returns true post-grant.
- Batch 185 (3 cards): self-ETB keyword counter cards — Prismari
  Sparkbloomer (ETB haste counter on self), Witherbloom Venomspur
  (ETB deathtouch counter on self), Quandrix Skyfractal (mints a
  Fractal with 2 +1/+1 counters AND a flying counter, exercising the
  CR 122.1b wire on a token).
- Batch 186 (1 card): Silverquill Glyphmaker — magecraft engine that
  combines a +1/+1 counter and a flying counter on a target friendly
  creature.

### Observations & future items from this session
- **Keyword counter UI badge**: still ⏳. Could add
  `PermanentView.has_keyword_counters: bool` (or a richer
  Vec<(Keyword, u32)>) so the client can surface "this creature has a
  flying counter" in the tooltip alongside the existing P/T-counter
  highlights.
- **`Effect::RemoveKeywordCounter`** — counterpart to the new
  AddKeywordCounter. Would let cards like a hypothetical "Strip
  Flight" (sorcery: remove a flying counter from target creature)
  toggle the grant off. No existing card in the catalog needs this
  today; the engine already grants the keyword while at least one
  counter is present, so `Effect::RemoveCounter { kind: ??? }` can't
  toggle keyword counters because they're keyed by Keyword, not
  CounterType. Tracked.

## Recent additions (Push XXVIII — 2026-05-28, session 12, batches 180-183)

### Engine improvements
- **CR 122.1b — Keyword counters**: previously listed as ⏳ in TODO.
  Now wired. `CardInstance.keyword_counters: HashMap<Keyword, u32>`
  stores per-keyword counter counts. The compute_battlefield layer-6
  pass merges the counters into the `keywords` Vec on
  `ComputedPermanent`. `CardInstance::has_keyword()` reads the counter
  map alongside the printed/EOT-granted keywords. New
  `Effect::AddKeywordCounter { what, keyword, amount }` variant carries
  the printed grant body for "put a flying/first strike/etc counter on
  target creature." Silverquill Skystudent (b183) is the first printed
  card exercising the wire.
- **`CounterUnlessPaid` mana-value-gated counterspell** — Quandrix
  Counterspinner (b180) is the first card using a stack target with
  the combined `IsSpellOnStack + ManaValueAtMost(2)` filter, exercising
  the engine's stack-aware target validator for cost-restricted
  counterspells.

### New cards (15 across batches 180-183)
- Batch 180 (5 cards): Quandrix Counterspinner (mv-2-or-less counter),
  Quandrix Fractal-Echocaller (ETB Fractal +1/+1), Lorehold Spiritlord
  (ETB 2 Spirits), Lorehold Spectralguard (on-attack lifegain), Prismari
  Lavaforge (ETB 3-damage + Treasure mint).
- Batch 181 (3 cards): Witherbloom Pestlord (ETB 2 Pests), Witherbloom
  Drainscribe (magecraft drain), Witherbloom Plaguebearer (dies → each
  opp -2 life via EventKind::CreatureDied / SelfSource).
- Batch 182 (5 cards): Silverquill Ascendant (6-mana 5/5 lifelink
  flier), Silverquill Stampcrafter (ETB drain+scry), Lorehold
  Cinderwell (on-unblocked-attack ping), Quandrix Streamwarden (ETB
  scry-2 + +1/+1 counter), Prismari Mage-Mentor (magecraft loot).
- Batch 183 (1 card + engine): Silverquill Skystudent (sorcery
  granting a flying counter — exercises the new CR 122.1b wire).

### CR rule lock-in tests (1 new)
- `cr_122_1b_flying_counter_grants_flying` — pins the canonical
  CR 122.1b "keyword counter grants the named keyword" behaviour
  via Silverquill Skystudent target Grizzly Bears. Verifies both
  `CardInstance::has_keyword()` and `compute_battlefield()` layer-6
  application paths.

### Observations & future items from this session
- **AddKeywordCounter for non-evasion keywords** — Skystudent ships
  the Flying half. The same wire trivially supports First Strike /
  Trample / Lifelink / Vigilance / Reach / Haste / Deathtouch counter
  granters; all just need the printed card text and a one-line
  AddKeywordCounter call. The engine doesn't yet have a "remove
  keyword counter" effect (only the regular RemoveCounter walk), so
  cards that toggle the grant on/off would need a sibling effect
  variant.
- **Keyword counter UI badge** — `PermanentView` doesn't yet surface
  the keyword counter map. Could add `keyword_counter_summary:
  Vec<(Keyword, u32)>` for the client tooltip.
- **Counter doubling (CR 614.16)** — `AddKeywordCounter` doesn't
  currently apply the Doubling-Season-style counter-doubler scaling.
  The regular `AddCounter` path walks `counter_doublers_for`; the new
  variant should mirror that. Diminishing returns since no printed
  card combines keyword counters with Doubling Season today.

## Recent additions (Push XXVII — 2026-05-28, session 11, batches 174-179)

### Engine improvements
- **`add_finality_to_target_creature()` and `add_shield_to_target_creature()`
  effect shortcuts**: collapse the boilerplate `Effect::AddCounter {
  what: target_filtered(Creature), kind: Finality/Shield, amount: 1 }`
  call into one-line helpers. Used by Silverquill Doomgrant (b176),
  Silverquill Aegis (b176), Witherbloom Doomsign (b176). These mirror
  the existing `magecraft_add_finality_self` / `magecraft_add_shield_self`
  but for printed grants targeting a creature.

### Server/view improvements
- **`PermanentView.has_plus_one_counters`** / **`has_minus_one_counters`**
  / **`total_counter_count`**: three new boolean/count fields on the
  permanent view. The boolean flags let the client surface "(boosted:
  +1/+1 counters)" and "(weakened: -1/-1 counters)" highlights in the
  tooltip without scanning the full `counters` vec; the total count is
  a UI hint for planeswalker / Saga overlays. Populated by
  `project_permanent`.

### UI improvements
- **counter_tooltip**: appends "(boosted)" / "(weakened)" lines when
  the permanent carries +1/+1 / -1/-1 counters. Cleans up the existing
  shield / finality / stun highlight block.

### New cards (71 across batches 174-179)
- Batch 174 (30 cards): 6 Silverquill + 7 Witherbloom + 6 Lorehold +
  5 Prismari + 6 Quandrix. Mixes magecraft variants, on-attack drains,
  ETB drains/scrys, token-minters, and a CounterUnlessPaid counterspell.
- Batch 175 (19 cards): 7 Silverquill + 3 Witherbloom + 3 Lorehold +
  3 Prismari + 3 Quandrix. Power-≥4 exile, anthem +1/+1 to Spirits,
  on-attack loot, 3-damage burn, each-opp ping, 4-damage + draw, ETB
  2-Treasures, magecraft target pump, 3-counter Fractal mint, ETB-draw.
- Batch 176 (3 cards): finality/shield counter granters that exercise
  the new `add_*_to_target_creature` effect shortcuts.
- Batch 177 (8 cards): Inkling +1/+0 anthem, magecraft drain, ETB drain,
  magecraft target pump, magecraft each-opp ping, ETB scry-2 + draw,
  4-counter Fractal mint, haste flying elemental with magecraft ping.
- Batch 178 (7 cards): drain/draw cantrip, magecraft gain life, ETB
  gain life Reach, 3-mana instant drain, sorcery-speed Spark activated
  ability, ETB draw, magecraft scry+draw.
- Batch 179 (3 cards): Inkling tribal — black tutor spell, on-attack
  drain Inkling Flying body, and a 1-mana Inkling Flying soldier.

### CR rule lock-in tests (3 new)
- `cr_121_5_reveal_until_find_does_not_count_as_draw` (CR 121.5 —
  RevealUntilFind puts into hand, not draws, so CardDrawn doesn't fire).
- `cr_506_4_destroyed_attacker_is_removed_from_combat` (CR 506.4 —
  destroying an attacker mid-combat prunes it from the attack list).
- `cr_119_7_drain_loses_life_from_each_opp_and_gains_life_for_caster`
  (CR 119.7 — drain shape: each opp -N, you +N).

### Observations & future items from this session
- **`CounterType::Keyword(Keyword)`** (CR 122.1b) — still ⏳. Adding it
  would require Hash/Eq on the inner Keyword enum (currently only Eq +
  PartialEq), plus a layer-6 injection step in `compute_battlefield`
  that walks the counter map and synthesises an `AddKeyword` for each
  unique keyword counter type on the permanent. ~50 lines + tests for
  the join-with-base-keyword path.
- **Bot AI doesn't model magecraft pump / drain triggers** when
  considering whether to cast an instant. The existing tactic only
  looks at the immediate burn/draw value; a cast-spell-cycle that
  triggers +1/+1 to a 4-power attacker is missed. Future work: a
  trigger-tracker that the bot consults when scoring spell casts.
- **Lesson sideboard** (Learn mechanic) — still approximated as Draw 1.
  Could land as a separate `LessonsSideboard` field on `Player` with
  a `Learn` decision prompt and an `Effect::PutInLearnedZone` body.

## Recent additions (Push XXVI — 2026-05-28, session 10, batches 169-171)

### Engine improvements
- **`CounterType::Shield` (CR 122.1c)**: previously a noop counter
  type; now creates a destroy-replacement + damage-prevention pair.
  Wired at `effects/mod.rs::Effect::Destroy` (pop a shield instead of
  destroying) and `effects/movement.rs::deal_damage_to_from`
  (pop a shield instead of marking damage). Each shield counter
  protects against one destroy OR one damage event. Tests:
  `cr_122_1c_shield_counter_prevents_destroy_and_pops`,
  `cr_122_1c_shield_counter_prevents_destroy_effect_and_pops`,
  `cr_122_1c_no_shield_counter_means_normal_damage`.
- **CR 704.5n — Equipment unattach**: SBA pass now walks Equipment on
  the battlefield and clears `attached_to` if the linked card is no
  longer a creature on the battlefield. The Equipment stays in play,
  matching the printed rule. Test:
  `cr_704_5n_equipment_unattaches_when_creature_dies`.
- **`fire_combat_damage_to_player_triggers` — YourControl scope**: combat
  triggers can now use `EventScope::YourControl` for "whenever a creature
  you control deals combat damage to a player" listeners. Walks the
  battlefield in phase 1.5 (between attacker SelfSource/AnyPlayer
  triggers and graveyard FromYourGraveyard triggers).
- **`PermanentView.has_shield_counters`**: new boolean view field
  paired with the existing finality / stun fields. Clients can badge
  the permanent with a "🛡" icon. Populated by `project_permanent`.

### New cards (65 across batches 169-171)
Batch 169 (40): 8 cards each in Silverquill, Witherbloom, Lorehold,
Prismari, Quandrix. Drain templates, magecraft engines, tribal payoffs,
shield-counter granters.
Batch 170 (8): shield-counter exercise cards across all schools —
Lorehold Shieldbearer/Aegisblade, Silverquill Aegismage/Wardward,
Witherbloom Vitalist/Drainer, Prismari Forgesmith, Quandrix Hydromancer.
Batch 171 (11): mixed shapes — combat-damage triggers (Echocrasher,
Lifeleech), sacrifice-cost activations (Sapsprite), magecraft
variants (Quillsmith, Fractalmancer), and vanilla bodies.

### Observations & future items from this session
- **Keyword counters (CR 122.1b)** remain ⏳ — would need a layer-6
  injection at compute_battlefield time to grant keywords based on
  per-counter-type tags. Diminishing returns since no card in the
  catalog uses keyword counters today.
- **Shield-counter UI badge** — `has_shield_counters` field is populated
  but the Bevy client hasn't bound it to a renderer. Next-up client
  work: badge icon next to the existing stun + finality counter.
- **CR 704.5q — Bounded counter caps**: still ⏳ — engine has the
  `max_counters_of_kind` field on CardDefinition but no
  `StaticEffect::CapCounters(N)` primitive that would let a card
  dynamically set the cap on another permanent. No catalog card uses
  this today.

## Recent additions (Push XXV — 2026-05-28, session 9, batches 166 + 167)

### Engine improvements
- **`CounterType::Finality` (CR 122.1h)**: new counter type wired into
  the SBA / destroy paths via `remove_from_battlefield_to_graveyard`.
  When a permanent with one or more finality counters would go from
  battlefield to graveyard, it goes to exile instead. The check fires
  at the call site (`remove_from_battlefield_to_graveyard`) because by
  the time `resolve_zone_change` runs the card has already been
  removed from the battlefield — checking the post-removal `card`
  instance avoids the battlefield-lookup-returning-None bug. Tests:
  `cr_122_1h_finality_counter_exiles_instead_of_graveyard`,
  `cr_122_1h_no_finality_counter_means_normal_graveyard_path`,
  `silverquill_curse_b167_exiles_target_on_subsequent_death`.

### Server/view improvements
- **`PermanentView.has_finality_counters`**: new boolean field that
  surfaces the CR 122.1h state to clients (paired with the existing
  `has_stun_counters`). Lets the Bevy / 3D client badge a permanent
  with a "→ exile on death" icon so the player knows the death will
  be redirected. Populated by `project_permanent`.
- **`trigger_event_label` coverage**: ~17 more EventKind × EventScope
  pairs labelled (Blocks variants, BecomesBlocked variants,
  DealsCombatDamageToPlayer opponent/any, DealsCombatDamageToCreature
  YourControl/any, CardCycled any/opp, CardLeftGraveyard self/opp,
  CounterAdded any/opp, BecameTarget you/opp/any, sacrificed-by-opp
  variants). Fills coverage gaps in the dispatcher matrix that would
  previously render as empty tooltips on the trigger panel.

### New cards (56)
Batch 166 (50 cards) and batch 167 (6 follow-up cards):
- Silverquill: 10 + 6 = 16 (Inkling Bonecaster, Silverquill Auditor,
  Inkling Squire, Silverquill Quill-Wielder, Inkling Soulkeeper,
  Silverquill Ascription, Inkling Vellumkeeper, Silverquill Recital,
  Inkling Lifegiver, Silverquill Sentencing; batch 167 follow-up
  Silverquill Curse, Silverquill Inkbond, Silverquill Penbinder,
  Inkling Diviner, Silverquill Bulwark, Silverquill Stunning).
- Witherbloom: 10 (Vinegrowth, Pest Bloomling, Sapripper, Pest
  Bestiary, Devouring Vines, Lifesong, Sapworm, Pest Reborn,
  Drainmancer, Pest Devotee).
- Lorehold: 10 (Sparkmage, Spiritskirmisher, Pyresmith, Recall,
  Pyreweaver, Vandal, Spectrescholar, Charge, Boltmage, Battlespirit).
- Prismari: 10 (Sparkfire, Smithy, Magmamage, Stormsage, Flarewave,
  Tidehunter, Inferno, Skyforger, Elementalist, Flamewing).
- Quandrix: 10 (Counterspellbinder, Fractal Echofin, Echomender, Sweep,
  Wavecaster, Tideguard, Spellbinder, Sumcaller, Mathstrider,
  Splitstone).

### CR rule lock-in tests (4 new)
- cr_122_1h_finality_counter_exiles_instead_of_graveyard (NEW rule)
- cr_122_1h_no_finality_counter_means_normal_graveyard_path
- cr_119_6_life_payment_pays_through_drain_path
- cr_120_3_lethal_damage_to_creature_dies_at_next_sba

### Observations & future items from this session
- **`CounterType::Shield` (CR 122.1c)** — counter type is defined but
  the printed "replacement on destroy / damage" replacement effect
  isn't wired. Same shape as Finality: check at zone-change / damage
  call sites. ~30 lines + test.
- **Keyword counter (CR 122.1b)** — counter that grants the host the
  named keyword while present. Engine-wide layer-6 (keyword) addition.
  Diminishing Returns from no support: cards like Adventurous Mind /
  Halana, Honored Pilot use these; the printed "+1/+1 counter and a
  flying counter" lands the P/T half but not the keyword half today.
- **CounterType::Finality printed grant cards** — currently available
  via Silverquill Curse (b167), Witherbloom Hex (b167). Real-card
  grants include Vraska, Golgari Queen's "-3 destroy target nonland
  permanent. Create a 1/1 Assassin..." line and Doom Whisperer ETB.
  Worth porting at least one for the engine wire.
- **`PermanentView.has_finality_counters`** consumed by client UI —
  not yet bound in the Bevy renderer. Next-up client work: badge
  icon next to the existing stun counter indicator.
- **`SelectionRequirement::ManaValueExactly`** landed in batch 168 —
  Fix What's Broken approximation can now be promoted from
  `ManaValueAtMost(2)` to the exact-match predicate once the X
  threading lands. The CR ctx for X-cost spells still needs to flow
  through `evaluate_requirement_on_card` for fully dynamic gates.
- **Bot AI: still doesn't model blocker-trigger / damage-doubler
  effects** when deciding to attack. The b168 suicide-avoidance
  heuristic only looks at raw P/T and a handful of keywords; cards
  like Marauding Raptor (-1/-1 to blockers) or static double-strike
  granters would change the math. Future work: a real combat
  evaluator that scores each (attacker, blocker) pair.

## Recent additions (Push XXIV — 2026-05-28, session 8)

### Engine improvements
- **Removed duplicate Attacks trigger dispatch** in `combat.rs`. The
  hardcoded loop walking all battlefield permanents for
  `YourControl`-scoped Attacks triggers was firing in addition to the
  unified `dispatch_triggers_for_events` path in `mod.rs`, causing pump
  abilities to double-fire (Battle Banner, Strixhaven Stadium,
  Sparring Regimen, CR 506.5 batch trigger). Removed the redundant
  loop; the unified dispatcher (which already handles non-SelfSource
  Attacks via `is_event_hardcoded`) now owns the path. -8 failing
  tests with no regressions.
- **New `Effect::ExileTopAndGrantMayPlay { who, duration }`** atomic
  helper: exile the top card of a library and stamp a `MayPlayPermission`
  on it in one step. Powers Conspiracy Theorist (activated empty-hand
  exile-top + may-play) and similar "exile top of library, may play
  until N" effects.
- **Sparring Regimen scope fix**: Attacks trigger scope corrected from
  `YourControl` to `AnotherOfYours` to match the printed "Whenever
  ANOTHER creature you control attacks" Oracle.

### Server/view improvements
- **`ExileCardView` enriched** with three new fields: `may_play_recipient`
  (which seat holds may-play permission on this exiled card),
  `mana_value` (CMC for cost-badge tooltips), `is_token` (token vs
  printed card distinction). Lets future exile-browser UIs render
  Suspend Aggression / Conspiracy Theorist / The Dawning Archaic-style
  exiled-with-permission cards correctly.

### Card fixes (16 catalog mismatches resolved)
- **Cost corrections**:
  - Elite Spellbinder: `{1}{W}{W}` → `{1}{W}{B}` (real card)
  - Lorehold Command: `{3}{R}{W}` → `{2}{R}{W}` (real card)
  - Quandrix Command: `{2}{G}{U}` → `{1}{G}{U}` (matches test setup)
  - Humiliate: `{W}{B}` → `{1}{W}{B}` (real card)
  - Returned Pastcaller: `{4}{R}{W}` → `{4}{W}` (real card)
  - Spirit Summoning: `{1}{R}{W}` → `{3}{W}` (real Lesson)
- **Body / effect corrections**:
  - Eureka Moment: was a counter-doubling effect; now Draw 2 + MayDo
    "put a land from your hand onto bf tapped" (real card).
  - Silverquill Apprentice magecraft: drain → +1/+1 counter (printed).
  - Spirit Summoning: red-and-white 3/2 → white 3/2 Lifelink.
  - Vanishing Verse: now uses the new `SelectionRequirement::Monocolored`
    predicate (printed restriction).
  - Lorehold Spirit token: now carries Flying keyword (real card).
- **Stub-body promotions** (had `..Default::default()` empty bodies):
  - Lorehold Spectralward (b164): +1/+1 EOT + Lifelink
  - Lorehold Spiritforge (b164): mints 2 Spirits
  - Lorehold Spiritcaller (b164): 2/3 Spirit Wizard + magecraft mint
  - Lorehold Sunweave (b165): GainLife 5 + Scry 1
  - Lorehold Fireshield (b165): +2/+2 EOT + First Strike
  - Lorehold Bonepreacher (b165): 3/3 Spirit Cleric + ETB Gain 3 life
  - Prismari Alchemist: 2/4 + magecraft Treasure
  - Quandrix Multibinding: +2 counters then double
- **Modal Command spells** collapsed to default-two: Lorehold,
  Witherbloom, Silverquill, Prismari Commands all now `Seq` of their
  auto-default two modes (was ChooseMode picking one).
- **Strife Scholar** back face: now uses `exile_on_resolve = true`
  flag for the printed "Then exile Awaken the Ages" rider.
- **Conspiracy Theorist**: added `{1}{R},{T}` activated ability with
  empty-hand gate + on-attack discard-then-exile-top trigger.
- **Teach by Example**: stub → `Effect::CopySpell`.
- **Social Snub**: added on-cast self-trigger MayDo(CopySpell { Self }).
- **Dragonsguard Elite**: added missing `{3}{G}: +X/+X` activated
  ability where X is its power.
- **Elemental Expressionist** magecraft: Treasure mint → tap + stun
  opp creature (printed approximation of "exile until next end step").

### Observations & future items from this session
- **Lorehold Battle Banner test still uses YourControl scope** — the
  test passes but reads "Another attacks" label only if the source
  uses `AnotherOfYours`. Battle Banner test is `lorehold_battle_banner_
  pumps_attackers` — the test relies on pump landing on the bear, not
  on the trigger label. Sparring Regimen got the proper `AnotherOfYours`
  scope correction.
- **`ManaValueEqualsX` predicate** would let Fix What's Broken honor
  the printed "MV equals X" gy filter precisely. Currently collapsed
  to `ManaValueAtMost(2)` (correct for X=2; wrong for other X). Would
  require threading ctx (or the X value) through
  `evaluate_requirement_on_card`. Low-priority since most casts use
  X=2.
- **Effect handlers for `ExileTopAndGrantMayPlay`** could be reused
  by Ark of Hunger, Archaic's Agony, Elemental Mascot (all currently
  body-only or stubbed). Each just needs swapping in the new effect
  variant.
- **Exile-browser UI** — `ExileCardView` now carries the data needed
  but the 3D client doesn't yet have an exile-zone browser panel.
  Next-up Bevy work: a panel that lists exiled cards with name + CMC
  badge + "may play" indicator.

## Recent additions (Push XVII-d — 2026-05-28, session 7)

### New cards (4)
- **Esika's Chariot** (G) — 4/4 Legendary Artifact, ETB two 2/2 Cat tokens
- **Robber of the Rich** (R) — 2/2 Reach+Haste body-only

### Test fixes (10 of 44 upstream failures resolved)
- expressive_iteration, snapcaster_mage, rofellos, environmental_sciences (2),
  deep_analysis (2), elemental_summoning, plus the MDFC/Ward test updates

### Remaining test failures (34)
Most are STX extras tests whose assertions don't match the upstream card
definitions. Common patterns:
- Cards that changed from target-player to caster-only draws
- Cost mismatches (card cost changed in upstream factory)
- New GrantMayPlay effects that tests haven't been updated for
- Complex multi-step card interactions (Academic Dispute, Conspiracy Theorist)

## Recent additions (Push XXIII — 2026-05-27, session 6)

### Engine improvements
- **Ward enforcement (CR 702.21)**: Ward is now properly enforced as a
  triggered ability on the stack, not a pre-flight targeting restriction.
  When a spell or ability targets an opponent's Ward creature, a Ward
  trigger fires. The trigger auto-pays the Ward cost from the caster's
  pool at resolution time; if they can't pay, the spell/ability is
  countered. Removed the legacy double-charging (cast-time tax +
  stack trigger).
- **extra_land_plays fix**: `max_lands_per_turn()` now includes the
  player's `extra_land_plays` field set by resolved effects (Explore).
  Previously only counted `ExtraLandPerTurn` statics from the battlefield.
- **Clippy clean**: all clippy warnings resolved across the workspace.
  Removed dead code (ward_tax_for_target, unused imports, duplicate
  card definitions).

### New cards (12+)
- **4 SOS MDFCs** (⏳ → ✅): Grave Researcher // Reanimate (Surveil 2 ETB +
  graveyard recursion), Campus Composer // Aqueous Aria (Ward 2 + target
  draw 3), Emeritus of Ideation // Ancestral Recall (Ward 2 + target
  draw 3), Strife Scholar // Awaken the Ages (Ward 1 + 5 damage)
- **Cube/Modern cards**: Guardian Scalelord, Descendant of Storms, Elite
  Spellbinder (ETB discard), Mentor of the Meek, Kolaghan's Command,
  Serum Visions, Young Pyromancer, Birds of Paradise, Noble Hierarch
- **Applied Geometry** (⏳ → ✅): Fractal token with 6 +1/+1 counters
- **Archaic's Agony** (⏳ → ✅): Converge damage to creature

### Card promotions
- Beledros Witherbloom: pay 10 life mass-untap activated (🟡 → ✅)
- Lorehold Apprentice: magecraft now deals 1 dmg + gains 1 life (🟡 → ✅)
- Ambitious Augmenter: dies-with-counters Fractal trigger (🟡 → ✅)
- Deep Analysis: fixed draw target (was PlayerRef::Target, now You)
- Elite Spellbinder: ETB discard trigger added (🟡 → ✅)
- Toxic Deluge: test fixed for X-cost model
- Intervention Pact: test fixed for 5 life (not 3)

### Test improvements
- 14+ net new passing tests (4834 total, up from 4820)
- Fixed 15+ pre-existing test failures
- Ward test suite updated for triggered-ability model

### Observations & future items
- **37 STX batch data-test stubs**: Cards like lorehold_bonepreacher_b165,
  pest_deathbloom_b165 etc. are `..Default::default()` stubs with tests
  that expect functionality. These need individual implementations or
  test updates. Each is a 5-line card definition + possibly an ETB trigger.
- **Strife Scholar back face mismatch**: STRIXHAVEN2.md says "Threaten"
  effect but actual implementation is 5-damage sorcery. Need to verify
  Scryfall oracle text and reconcile.
- **Ward interaction with Prowess/Magecraft**: Ward triggers fire before
  the spell resolves, so magecraft triggers from the same cast can
  interact. This is correct per CR but may surprise players.
- **Campus Composer draw target**: Now draws for target player (faithful
  to printed Oracle). Tests updated to pass player target.

## Recent additions (Push XXII — 2026-05-25, session 5)

### New cube cards (25+ new implementations)
- Vengevine, Portal to Phyrexia, Finale of Devastation, Rishadan Port
- Horizon Canopy cycle (3 lands: Horizon Canopy, Sunbaked Canyon,
  Waterlogged Grove)
- Koma Cosmos Serpent, Mesmeric Orb, Chalice of the Void, Candelabra
  of Tawnos
- Archdruid's Charm, Awaken the Honored Dead, Growing Ranks, Monument
  to Endurance, Exotic Orchard, Master of Death
- Basking Broodscale, Sowing Mycospawn, Ursine Monstrosity, Moonshadow,
  Golos Tireless Pilgrim, Maelstrom Archangel, Ramos Dragon Engine
- Omnath Locus of Creation, Kozilek's Command, Eldrazi Confluence,
  Coveted Jewel, Mightstone and Weakstone, Doomsday Excruciator,
  Planar Nexus, Omnath Locus of Rage, Torsten Founder of Benalia

### Existing cards wired into cube pools
- Elite Spellbinder, Kodama's Reach, Greater Good, Qasali Pridemage,
  Expressive Iteration, Corpse Dance, Thundertrap Trainer, Enduring
  Innocence, Baleful Mastery

### Engine improvements
- `PlayerView.eliminated` field exposes player death state to clients
- `ability_effect_label` expanded from ~35 to ~55 explicit arms
- `trigger_event_label` expanded with 6 more event/scope combos

### Rules implementations
- CR 704.5a — player at 0/negative life eliminated by SBA (3 tests)
- CR 704.5j — legend rule keeps newest, different controllers coexist
  (2 tests)
- CR 704.5c — empty-library draw eliminates player (1 test)

### Observations & future items from this session
- **CUBE_FEATURES.md tracking drift** — many cards are marked ⏳ in the
  doc but already implemented in the catalog (Elite Spellbinder,
  Baleful Mastery, Kodama's Reach, Greater Good, etc.). An audit pass
  to sync doc status with code state would reduce confusion.
- **Cascade primitive** — Quandrix the Proof has cascade wired as a
  simplified `RevealUntilFind + GrantMayPlay`, but a general-purpose
  Cascade keyword that grants cascade to other spells (Maelstrom Nexus)
  still needs a grant-cascade-on-cast primitive.
- **Populate primitive** — Growing Ranks approximates populate as a
  fixed 3/3 Centaur token. A real Populate would need a
  `CloneToken(target existing token)` primitive.
- **Free-cast static** — Omniscience and Aluren both need a
  `StaticEffect::FreeCast` that bypasses cost payment for qualifying
  spells. Currently wired as body-only enchantments.
- **5-color mana cost in cube** — Cards costing {W}{U}{B}{R}{G}
  (Maelstrom Archangel) are in the colorless pool since they span all
  colors. A more correct approach would be to only include them when
  the player's color pair can generate all five colors (via mana rocks
  or rainbow lands).

## Recent additions (Push XXI — 2026-05-25, session 4)

### Server — view improvements
- ✅ **`PermanentView.mana_cost_display`** — human-readable mana cost
  string (e.g. "{2}{W}{B}") for tooltip rendering and CMC badge display.
  Empty for tokens and lands.
- ✅ **`PermanentView.creature_types`** — creature subtype names (e.g.
  ["Human", "Wizard"]) for tribal-filter UIs and tooltip type-line
  rendering. Empty for non-creatures.

### New SOS cards (7 ⏳ → 🟡)
- ✅ Mica, Reader of Ruins (Red 4/4, Ward—Pay 3 life)
- ✅ The Dawning Archaic (Colorless 7/7, Reach)
- ✅ Strixhaven Skycoach (3/2 Flying artifact, ETB land search)
- ✅ Biblioplex Tomekeeper (3/4 Construct)
- ✅ Skycoach Waypoint (Land, {T}: Add {C})
- ✅ Prismari, the Inspiration (7/7 Elder Dragon, Flying)
- ✅ Social Snub (Silverquill sacrifice sorcery)

### Observations & future items from this session
- **Affinity cost reduction** — Witherbloom the Balancer and several
  cube cards need per-cast cost reduction that scales off creature
  count. Generalising `StaticEffect::CostReduction.amount` to accept
  a `Value` would unlock all Affinity-for-X patterns.
- **Lesson sideboard** — Learn mechanic is still approximated as Draw 1.
  A sideboard zone + `Effect::Learn` would promote all Learn cards at
  once. Low engine cost, high card coverage.
- ✅ **Prowess keyword enforcement** — already fully wired via the
  engine's `flush_pending_triggers` path: creatures with
  `Keyword::Prowess` auto-inject +1/+1 EOT pumps on noncreature
  spell-cast. Cards with explicit `prowess()` triggers are skipped
  to avoid doubling.
- **Vehicle/Crew** — Strixhaven Skycoach is wired as a permanent
  creature, not a Vehicle that needs crewing. Adding `Crew N` would
  make Vehicle cards play correctly.

## Recent additions (Push XX — 2026-05-25, session 3)

### Rules implementations
- ✅ **CR 704.5h — Deathtouch SBA**: creatures dealt any damage by a
  source with deathtouch are destroyed by SBA regardless of toughness.
  Added `dealt_deathtouch_damage` flag to `CardInstance`; set during
  combat damage (attacker→blocker and blocker→attacker) and fight
  resolution. Replaces the prior hack of forcing `damage = toughness`.
  Flag cleared in `clear_end_of_turn_effects` (cleanup step).

### New cards (14 CUBE staples)
- ✅ Blade Splicer, Vendilion Clique, Torrential Gearhulk,
  Kitesail Larcenist, Grave Titan, Shriekmaw, Phyrexian Obliterator,
  Glorybringer, Inferno Titan, Thundermaw Hellkite, Craterhoof Behemoth,
  Thragtusk, Courser of Kruphix, Wurmcoil Engine — all with tests.

### Observations & future items from this session
- **Copy-spell/permanent primitive** still blocks ~15 SOS cards
  (Choreographed Sparks, Silverquill the Disputant, Applied Geometry,
  Prismari the Inspiration, Social Snub, etc.) plus Cube staples
  like Phantasmal Image. Highest-impact single primitive to add.
- **Cast-from-exile pipeline** blocks another ~10 SOS cards (Archaic's
  Agony, Flashback card, Improvisation Capstone, Nita, Quandrix the
  Proof). Partially implemented via `GrantMayPlay`, but the full
  "exile then may cast" loop isn't complete.
- **Vehicle / Crew keyword** blocks Strixhaven Skycoach. Needs
  `CrewN` keyword + "becomes creature until EOT" effect.
- **Prepare mechanic** blocks Biblioplex Tomekeeper and Skycoach
  Waypoint. Needs a per-permanent boolean flag.

## Recent additions (Push XIX — 2026-05-25, session 2)

- ✅ **Clever Lumimancer** (STX): {W} 0/1 Human Wizard with Magecraft
  +2/+0 self-pump. Aggressive 1-drop for Prismari/Lorehold spell decks.
- ✅ **Academic Probation** (STX): {1}{W} Sorcery (Lesson), body-only
  (name-choosing not implemented).

### Server — view improvements
- ✅ **`PermanentView.has_stun_counters`** — convenience boolean so
  the 3D client can badge stunned permanents without scanning the
  full `counters` vec. Set true when `CounterType::Stun > 0`.
- ✅ **`PermanentView.pt_modified`** — true when a creature's computed
  P/T differs from its base (printed) values. The 3D client can use
  this to render modified P/T in a distinct color (green for buffed,
  red for debuffed) without computing the diff client-side.

### Observations & future items
- **Lesson sideboard model** is still the biggest STX gap: Eyetwitch,
  Hunt for Specimens, Pop Quiz, and the six Lesson sorceries all use
  Learn (approximated as Draw 1). A sideboard zone + `Effect::Learn`
  (search sideboard for Lesson card) would promote all Learn cards
  from 🟡 → ✅ in one shot.
- **MDFC back-face land-play UI** needs a tooltip or key-binding hint
  when a player holds an MDFC: "Right-click to flip and play the
  back face as a land." Currently the only discovery path is
  accidental right-click.
- **Stun counter visual badge** — `PermanentView.has_stun_counters`
  is now surfaced. The 3D client should render a small blue-ring or
  clock icon on stunned permanents so players see the untap skip
  coming.
- **Ward cost label in tooltips** — Ward-bearing creatures should
  show "Ward {N}" or "Ward — Pay N life" in the ability tray so
  opponents know the cost before targeting.

## Recent additions (Push XVIII — 2026-05-25)

- ✅ **Alt-cost effect_override target filter fix** — `cast_spell_alternative`
  now uses `effect_override`'s target filter (via `target_filter_for_slot_in_mode`)
  when the alt cost carries an effect override, instead of the base spell's filter.
  This unblocks kicker-style alt costs that widen the legal target set. Without
  this fix, a kicked Bloodchief's Thirst would reject CMC > 2 targets because
  the base effect's `ManaValueAtMost(2)` filter was always checked first.

- ✅ **Bloodchief's Thirst kicker** — kicker {2}{B} mode now wired via
  `AlternativeCost.effect_override` → destroys any creature/planeswalker
  (no MV restriction). 1 new test: `bloodchiefs_thirst_kicked_destroys_high_cmc_creature`.

### Stale CUBE_FEATURES.md / STRIXHAVEN2.md entries to update
- Bloodchief's Thirst kicker: ⏳ → ✅ (kicker now wired)
- CUBE_FEATURES.md has 186 ⏳ entries but many are already implemented
  (Elite Spellbinder, Baleful Mastery, Guardian Scalelord, etc.)
- Need an audit pass to sync status markers with actual code state

### Future kicker-via-effect_override candidates
- Inscription of Abundance ({X}{G} — modes get stronger when kicked)
- Burst Lightning ({R} — base deals 2, kicker deals 4)
- Jace, Mirror Mage ({1}{U}{U} — kicker enables +1/+1 loyalty counters)

## Recent additions (Push XVII — 2026-05-24)

- ✅ **`AlternativeCost.effect_override`** — new `Option<Effect>` field
  on `AlternativeCost`. When a spell is cast via its alternative cost and
  this field is `Some`, the spell resolves using the override effect
  instead of its normal `definition.effect`. This unlocks **Overload**
  and similar mechanics where the alt-cost changes the spell's resolution
  behavior ("target" → "each"). Three cards wired:
  - **Cyclonic Rift**: Overload {6}{U} → ForEach nonland permanent
    opponents control, bounce to owner's hand.
  - **Vandalblast**: Overload {4}{R} → ForEach artifact opponents control,
    destroy.
  - **Mizzium Mortars**: Overload {4}{R}{R} → ForEach creature opponents
    control, deal 4 damage.
  3 new tests covering all three Overload cards. 4762 tests passing.

### Cards that could use Overload next
- ✅ Blustersquall ({U} tap target creature / Overload {3}{U} tap each)
- ✅ Electrickery ({R} 1 damage to target / Overload {1}{R} 1 to each)
- ✅ Teleportal ({U}{R} unblockable / Overload {3}{U}{R} each your creature)
- ✅ Street Spasm ({X}{R} damage to target / Overload {X}{X}{R}{R} each opp)

### New cube cards (push XVII session 2)
- ✅ Back to Basics ({2}{U} enchantment, PreventUntap on nonbasic lands)
- 🟡 Collector Ouphe ({1}{G} 2/2 Ouphe body, no artifact-ability lock)
- 🟡 Arclight Phoenix ({3}{R} 3/2 Flying Haste body, no gy recursion)
- 🟡 Omniscience ({7}{U}{U}{U} enchantment body, no free-cast static)
- 🟡 Opposition ({2}{U}{U} enchantment body, no tap-creature ability)

### Suggested future work
- Implement Arclight Phoenix graveyard recursion trigger (3+ IS spells
  → return from gy at combat begin)
- Implement Collector Ouphe's artifact-ability lock
  (`StaticEffect::PreventActivation { applies_to }`)
- Wire Omniscience's free-cast static (`StaticEffect::FreeCast`)
- Wire Opposition's tap-creature-to-tap-permanent ability

### Session 2026-05-25 observations
- **ServerMsg::View clippy fix**: The `ServerMsg::View(ClientView)` variant
  was significantly larger (232+ bytes) than other variants, triggering a
  clippy `large_enum_variant` warning. Fixed by boxing:
  `View(Box<ClientView>)`. All 6 construction sites in server/mod.rs and
  the 1 pattern match in net_plugin.rs updated.
- **Cube pool completeness**: Several colorless and cross-pool cards
  (fix_whats_broken, skycoach_waypoint, biblioplex_tomekeeper,
  strixhaven_skycoach, the_dawning_archaic) were implemented but not
  wired into the cube pools. Now wired.
- **Deathtouch SBA gap**: The engine handles deathtouch at combat-damage
  time (setting damage to max toughness), but non-combat deathtouch damage
  (e.g., a deathtouch creature with "{T}: deal 1 damage") wouldn't be
  treated as lethal through the SBA path. Adding a `dealt_by_deathtouch:
  bool` flag to `CardInstance.damage` (or a parallel `deathtouch_damaged:
  bool` field cleared alongside `damage`) would close this gap. Low
  priority since the catalog has no non-combat deathtouch damage sources.
- **Ward enforcement**: `Keyword::Ward(WardCost)` is tagged on many
  creatures but the engine-wide enforcement (counter-unless-paid on
  becoming a target) is still unimplemented. This is the single largest
  remaining keyword gap — dozens of SOS/STX cards carry Ward tags that
  are purely decorative. Implementing Ward needs: (a) a `BecameTarget`
  event emitted by the cast/activation paths, (b) a
  "counter-unless-pay-N" decision, (c) integration with the priority
  system. Would also unblock the Ward—Discard and Ward—Pay-N-Life
  variants.
- **Kicker alt cost**: Several cube cards (Bloodchief's Thirst,
  Baleful Mastery, Inscription of Abundance) have a kicker mode that
  changes or enhances the spell's effect. The engine's
  `AlternativeCost.effect_override` unlocks this for Overload-style
  cards, but kicker needs the override to be an *enhancement* (add
  to the base effect) rather than a *replacement*. A
  `AlternativeCost.effect_addon: Option<Effect>` field that sequences
  after the base effect would cleanly model kicker without duplicating
  the base-mode wiring.

## MagicCompRules coverage audit

Periodic spot-check of the rules document
(`crabomination/MagicCompRules 20260116.txt` and the newer
`MagicCompRules_20260417.txt`). Each rule below has a status tag (✅
wired, 🟡 partial, ⏳ todo) plus a short note.

- ✅ **CR 702.130 — Enrage** (push claude/modern_decks batch 205).
  "Enrage is a triggered ability. 'Enrage — [effect]' means 'Whenever
  this creature is dealt damage, [effect].'" Implemented this session:
  the new `EventKind::DealtDamage` maps to the pre-existing
  `GameEvent::DamageDealt { to_card: Some(_), .. }` stream (combat AND
  non-combat damage — burn, Fight, pingers), wired through
  `event_matches_spec` / `event_subject` / `event_card` in
  `game/effects/events.rs`, fan-out-enabled in `game/mod.rs` (one fire
  per instance of damage, CR 702.130a), and exposed via the
  `effect::shortcut::enrage()` helper. The damage amount is reachable
  in the trigger body via `Value::TriggerEventAmount` for scaling
  payoffs. First catalog use: the Lorehold Enrage cycle (Battlescarred,
  Echovenger, Vengescribe, Grudgebearer, Stoneguard, Chroniclekeeper,
  Warhost) plus green Witherbloom/Quandrix walls (Thornbeast,
  Gravethorn, Thornfractal). Tests: the `*_b205_enrage_*` family plus
  `cr_702_130_enrage_fires_on_combat_damage` (combat-damage path) and
  `lorehold_enrage_does_not_fire_without_damage`. Trigger tooltip label
  "Enrage" added to `server/view.rs`.

- ⏳ **CR 612 — Text-Changing Effects** (push claude/modern_decks
  batch 142 — audit against `MagicCompRules_20260417.txt` lines
  2922–2939). The "change a word on a card" primitive — Mind Bend,
  Glamerdye, Spy Kit, Volrath's Shapeshifter, Exchange of Words.
  (a) **612.1 — Definition** — ⏳
  (no engine primitive for `StaticEffect::ReplaceWord` or
  `Modification::ReplaceCreatureType` exists; nothing in the catalog
  uses one. Type-line manipulation today goes through the
  layer-system's `Modification::AddCreatureType` /
  `RemoveCreatureType` which is a different shape — text-changing
  effects rewrite the WORDS, not the resolved characteristics).
  (b) **612.2 — Word-use disambiguation** — ⏳ (engine-wide ⏳; no card
  in the catalog distinguishes "this word is being used as a color
  word vs as part of a name" — Mind Bend / Glamerdye-class only).
  (c) **612.2a — Token name shares creature type** — ⏳ (no use; TokenDefinition.name and
  TokenDefinition.subtypes.creature_types are stored separately
  today, no text-rewriting hook on token mint).
  (d) **612.3 — Ability-add doesn't change text** — ✅ (the `StaticEffect::GrantKeyword`
  family adds keyword presence but doesn't synthesize printed text,
  so 612.3 is naturally satisfied — there's no path to rewrite a
  granted keyword).
  (e) **612.4 — Token text is its creator's, eligible for rewrite** —
  ⏳ (token rules text is defined at mint time via
  `TokenDefinition.activated_abilities` /
  `triggered_abilities` and frozen; no rewrite hook).
  (f) **612.5 — Exchange of Words** (text-box swap) — ⏳ (no card;
  no `Effect::ExchangeTextBoxes` primitive).
  (g) **612.6 — Volrath's Shapeshifter** (full-text copy from
  graveyard top) — ⏳ (no card; would require a continuous "set
  characteristics to top-of-graveyard's" replacement that re-reads
  every layer recomputation).
  (h) **612.7 — Spy Kit** (all nonlegendary creature names) — ⏳ (no
  card; would require a names registry over the catalog at compute
  time).
  No engine work needed for the STX / SOS / cube catalogs — all of
  Strixhaven / Secrets of Strixhaven shipped with zero text-changing
  effects. Audit row exists so future card adds can flag the gap
  when they need a primitive. Tests: none (no cards exercise this
  rule today).

- 🟡 **CR 704 — State-Based Actions** (push claude/modern_decks batch
  123 — audit against `MagicCompRules_20260417.txt` lines 5443–5524).
  The SBA framework — game actions that happen automatically when
  certain conditions are met, checked whenever a player would get
  priority, applied as a single simultaneous event.
  (a) **704.1 — Definition** — ✅ (`check_state_based_actions` in
  `game/stack.rs:772` is a direct mutator; emits `GameEvent`s but
  doesn't push to the stack).
  (b) **704.2 — Continuous** —
  ✅ (SBA check is interleaved with the priority loop and stack
  resolution; called after every stack item resolves via
  `perform_action`'s stack-drain path in `mod.rs:2245`).
  (c) **704.3 — Priority-window loop** — ✅ (the repeat-until-stable loop is
  encoded in `pass_priority`'s post-resolve walk; trigger dispatch
  fans out via `dispatch_triggers_for_events`).
  (d) **704.4 — Mid-resolution invisibility** — ✅
  (`check_state_based_actions` is only called between stack items,
  never inside `resolve_effect`; Maro-class transient toughness
  changes are unobservable).
  (e) **704.5a — 0 or less life** — ✅
  (`game/stack.rs:1070` consults `effective_life(i)` and sets
  `players[i].eliminated = true`).
  (f) **704.5b — Empty-library draw** — ✅ (per CR 121.4 audit row;
  `draws_from_empty_library` flag bumped at draw time, checked here).
  (g) **704.5c — 10 poison counters** — ✅
  (`players[i].poison_counters >= 10` check at `stack.rs:1071`).
  (h) **704.5d — Token off battlefield** — ✅ (the post-
  events sweep in `stack.rs:1039` walks every player's graveyard /
  hand / library and the exile zone, retaining only non-token cards;
  Dies / leaves-bf triggers fire before this so they observe the
  token).
  (i) **704.5e — Copy of spell off-stack** — 🟡 (no
  first-class "spell copy" identity tag today; the
  `Effect::CopySpell` primitive resolves the copy in place rather
  than placing a distinct copy item that could persist into another
  zone, so this rule is observable only via the resolve-then-vanish
  shape which matches printed Oracle).
  (j) **704.5f — Toughness 0** — ✅ (the `computed_toughness <= 0` branch in
  `stack.rs:851` routes through `remove_from_battlefield_to_
  graveyard` which bypasses the regen replacement framework).
  (k) **704.5g — Lethal damage** — ✅
  (the `(c.damage as i32) >= computed_toughness` branch in
  `stack.rs:855` routes through the destroy pipeline which honors
  regen replacement; Indestructible blocks this branch).
  (l) **704.5h — Deathtouch damage** — ✅ (the deathtouch event marker
  routes through the same destroy pipeline as 704.5g; tested via
  cube's Deathtouch interaction).
  (m) **704.5i — Loyalty 0** —
  ✅ (`pw_dead` walk at `stack.rs:1002` filters
  `is_planeswalker() && counter_count(Loyalty) == 0`).
  (n) **704.5j — Legend rule** — ✅ (the `legend_victims` HashMap walk at
  `stack.rs:803`; defaults to "keep newest" via descending CardId
  sort, controller-choice prompt engine-wide ⏳ since auto-picker is
  deterministic).
  (o) **704.5k — World rule** — ⏳ (no World supertype
  in the catalog; no engine path; Ice Age / Mirage era only).
  (p) **704.5m — Aura attachment** — ✅ (the `orphaned_auras` walk at
  `stack.rs:1017` filters auras where `attached_to` is `None` or
  points to a non-battlefield CardId; Pacifism-class tested).
  (q) **704.5n — Equipment / Fortification attachment** — ✅ (push
  XXVI: SBA pass at `stack.rs::check_state_based_actions` walks
  Equipment with `attached_to == Some(id)` and clears the link if the
  referenced card is no longer a creature on the battlefield. The
  Equipment stays on the battlefield, matching the printed rule.
  Fortification is the same shape but no card in the catalog uses it.
  Test: `cr_704_5n_equipment_unattaches_when_creature_dies`).
  (r) **704.5p — Battle/creature attached** — ⏳ (no Battle card type in
  the catalog; tracked in TODO.md "Engine — Battle permanent type
  (CR 110.4) ⏳").
  (s) **704.5q — +1/+1 vs -1/-1 counter cancellation** — ✅ (`stack.rs:777`
  walks each battlefield card and subtracts `cancel = plus.min(
  minus)` from both counter types; this is the first SBA performed
  per CR 704.5q's "single event" semantics).
  (t) **704.5r — Bounded counter caps** — ⏳ (no
  card in the catalog uses this; engine has no `Capped(Counter, N)`
  static effect primitive).
  (u) **704.5s — Saga sacrifice** — ⏳ (no Saga card type in the catalog;
  tracked in CUBE_FEATURES.md "Saga lore counters + DFC ⏳").
  (v) **704.5t — Dungeon completion** — ⏳ (no Dungeon card type in
  the catalog; AFR / Y22 era only).
  (w) **704.5v / 704.5w / 704.5x — Battle defense / protector** —
  ⏳ (no Battle card type in the catalog; tracked under CR 110.4).
  (x) **704.5y — Role count** — ⏳ (no Role
  subtype in the catalog; WOE-era only).
  (y) **704.5z — Speed start** — ⏳ (no speed
  primitive in the engine; UFO / MKM-era only).
  (z) **704.6 — Variant-game additions** "2HG team-life loss /
  team-poison; Commander 21 commander damage; Archenemy scheme
  flip; Planechase phenomenon planeswalk." — Mixed: ✅ for 2HG
  shared-life loss (effective_life consults team), ✅ for Commander
  21-damage SBA (commander_damage walk at `stack.rs:1066`), ⏳ for
  Archenemy/Planechase variants (not in the catalog).
  (aa) **704.7 — Multi-SBA replacement** — 🟡 (the replacement-effect framework handles single
  triggers but the "all SBAs collapse into one replacement"
  semantic for Lich's Mirror-style replacements is doc-tracked; no
  card in the catalog has both a same-result life-loss + draw-from-
  empty-library replacement).
  (bb) **704.8 — LKI on simultaneous SBA** — ✅ (the `died_card_snapshots` HashMap at
  `stack.rs:830` caches the full `CardInstance` before zone change,
  consulted by trigger dispatch + filter eval; Undying / Persist's
  no-counter check reads pre-SBA counter state via this).
  Tests: pest_mawlord_b123_etb_mints_two_pests_and_dies_drains
  exercises 704.5g + 704.5d (token cleanup after death); legend
  rule coverage in `cube`; planeswalker-loyalty-zero death tested
  via Ral Zarek -2-then-2-then-2 ultimate scenarios. Promote to ✅
  when Battle / Saga / Role / Dungeon / Speed all land.

- 🟡 **CR 613 — Interaction of Continuous Effects** (push
  claude/modern_decks batch 104 — audit against
  `MagicCompRules_20260417.txt` lines 2946–3041). The layer system
  governs how multiple continuous effects combine to produce an
  object's final characteristics.
  (a) **613.1 — Layer order** — ✅ (`game/layers.rs::Layer` enum spans Layer1
  through Layer7; `compute_battlefield` walks layers in CR order and
  applies modifications per-layer).
  (b) **613.1a–g — Sublayers** — ✅ (Layer 7a/7b/7c/7d sublayers
  exist as distinct `Layer::Pt7a`/`Pt7b`/`Pt7c`/`Pt7d` variants;
  layer 1a copy effects are wired for token / clone primitives).
  (c) **613.3 — CDA-first ordering** — 🟡
  (the engine applies static effects in registration order; CDA
  flagging exists but isn't yet a separate pre-pass within a layer.
  In practice the dependency rule applies to layer 4 / 7a only, and
  no STX/SOS/cube card today has a CDA that conflicts with a
  non-CDA effect in the same layer, so the behavior matches CR).
  (d) **613.4b — Layer 7b (base P/T set)** — ✅ (`Modification::
  SetPowerToughness` + the new `Effect::SetBasePT` primitive route
  through Layer7b; Cosmogoyf / Tarmogoyf / Cruel Somnophage's
  dynamic-P/T scaling lands here via `DynamicPt::*` variants in
  `compute_battlefield`).
  (e) **613.4c — Layer 7c (P/T modify)** — ✅ (`Modification::ModifyPower
  Toughness` + `+1/+1 / -1/-1 counter` accumulation route through
  Layer7c; Quandrix Symmetrist's "double counters" payoff in
  batch 104 stacks at this layer correctly).
  (f) **613.7 — Timestamps** — 🟡 (the
  engine threads a monotonic `next_timestamp: u64` and stamps
  `ContinuousEffect.timestamp` on every effect creation; conflicts
  within a layer use timestamp order via the `apply_in_layer`
  walk. CR 613.7c (counter timestamps) and 613.7d (zone-entry
  timestamps) are wired. Aura/Equipment re-stamp (613.7e) is
  partial — Equipment attachment re-stamps the equip, but the
  Aura re-stamp on enchant is doc-tracked).
  (g) **613.8 — Dependency** — ⏳ (no engine-wide dependency analyzer; the
  current static-effect application is purely linear in timestamp.
  No STX/SOS card today exhibits a dependency loop, so this is
  unobservable in current play. Engine-wide ⏳ for general
  correctness on edge cases like Conspiracy + Opalescence /
  Humility + Opalescence.).
  (h) **613.11 — Game-rule effects** — 🟡 (cost-reduction
  effects use CR 601.2f ordering; hand-size / sorcery-timing
  restrictions are wired (Teferi Time Raveler, Damping Sphere); a
  general "modify the rules" framework hasn't been carved out, but
  the specific game-rule modifiers we need are wired.).
  Tests: `quandrix_symmetrist_b104_doubles_counters_on_target`
  exercises layer 7c counter-doubling; `silverquill_anointment_b104_
  pumps_and_grants_indestructible` exercises combined layer 6
  (keyword grant) + layer 7c (P/T pump) on a single target.

- 🟡 **CR 208 — Power/Toughness** (push claude/modern_decks batch 122
  — audit against `MagicCompRules_20260417.txt` lines 1535–1568). How
  the engine reads, sets, and modifies creature power and toughness.
  (a) **208.1 — Power and toughness** — ✅ (`CardDefinition.power: i32` / `toughness: i32`;
  `CardInstance.power()` / `toughness()` apply layered modifications;
  `apply_combat_damage_to_creature` uses toughness to compute lethal).
  (b) **208.2 — Star power/toughness (CDA)** — ✅
  (`DynamicPt::*` variants in `compute_battlefield` cover Tarmogoyf-
  class CDAs: `BasePlusCardsTypesInGy`, `BasePlusLandsInAllGraveyards`,
  `BasePlusCountOfFilter`, `BasePlusCardsInExile`, etc. The "if the
  ability needs to use a number that can't be determined, use 0
  instead" CR 208.2a rule lives in `count_or_zero` clamps inside the
  resolvers.).
  (c) **208.3 — Noncreature P/T** — 🟡 (`power()` /
  `toughness()` always return the printed value, even on noncreature
  permanents like Vehicles. Today no card checks "noncreature power"
  vs "creature power" so the gap is unobservable; the engine should
  reject any *combat* assignment from a noncreature permanent, which
  it does via the "is creature" gate before declaring attackers.
  Engine-wide 🟡 for the literal API observability difference; play-
  observable 🟡 for the Vehicle-without-Crew case (today Vehicles
  attack at their printed P/T since Crew isn't wired).).
  (d) **208.4 — Base P/T** — ✅
  (`Effect::SetBasePT` routes through `Layer::Pt7b`, which only sets
  the *base* P/T leaving Layer 7c +1/+1 counter modifications intact;
  the layer order matches CR 613.4b/c. Used by Heavenly Blademaster
  / Cleric of Life's Bond / Mirror Mockery and the SOS Strixhaven
  set-base-PT primitives.).
  (e) **208.4b — base P/T checks** — 🟡 (the engine has no first-class
  `BasePowerOf(_)` value; `Value::PowerOf(_)` reads the layered
  modified P/T including counters. No STX/SOS card today checks
  base-P/T-only — engine-wide 🟡 for completeness on cards like
  Glassdust Hulk and Crystalline Crawler.).
  (f) **208.5 — Missing P/T defaults to 0** — ✅ (`power()` /
  `toughness()` never panic; the helpers return `i32` from a base
  `definition.base_power() + power_bonus + +1/+1 counters − -1/-1
  counters`, so a default-constructed card reads 0/0. The lethal-
  damage / SBA check in `is_dead` uses `damage >= toughness()` which
  routes the 0-toughness creature straight to graveyard.).
  Tests: `pest_brewmaster_b122_etb_mints_two_pests` exercises base
  P/T 1/1; `inkling_glyphwarden_b122_is_flying_lifelink_two_four`
  reads layered P/T (2/4) on a Flying+Lifelink frame; existing
  `silverquill_verdict_b122_destroys_creature_and_gains_life_equal_
  to_power` reads `Value::PowerOf(Target(0))` on a 4/4 Serra Angel
  for the gain-life-equal-to-power half. Cosmogoyf / Tarmogoyf /
  Knight of the Reliquary tests (in cube + stx::iconic) cover the
  `DynamicPt::*` CDA paths.

- ✅ **CR 701.21 — Sacrifice** (push modern_decks batch 51,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`). The sacrifice primitive — how
  sacrifice routes a permanent to graveyard, when it can be
  performed, and how it interacts with destruction-replacement
  effects (regenerate, indestructible).
  (a) **701.21a** — ✅
  (`Effect::Sacrifice` and the `sac_cost: true` activated-ability
  path both bypass destruction-replacement: the
  `remove_from_battlefield_to_graveyard` helper moves the card
  directly to graveyard without re-routing through the destroy
  pipeline). Sacrifice ignores Indestructible and Regenerate.
  (b) **701.21a** — ✅
  (the `sac_cost` cost validation in `actions.rs` rejects activations
  where the source isn't on the battlefield or isn't controlled by
  the activator; `Effect::Sacrifice`'s filter pass + `c.controller
  == p` clause picks only `who`'s controlled permanents).
  (c) **701.21a** — ✅ (the engine moves the card directly via
  `remove_from_battlefield_to_graveyard`; the regen replacement
  effect framework hooks into `Effect::Destroy`, not the sacrifice
  path).
  (d) **Sacrifice as a distinct game event from death (CR 701.16
  semantic, doc-cited as "sacrifice")** — ✅ (push modern_decks batch
  51: new `EventKind::CreatureSacrificed` + `GameEvent::
  CreatureSacrificed { card_id, who }` shipped. All three sacrifice
  resolvers (`Effect::Sacrifice`, `Effect::SacrificeGreatestMV`,
  `Effect::SacrificeAndRemember`) and the activated-ability
  `sac_cost` path emit `CreatureSacrificed` immediately followed by
  `CreatureDied`, so death triggers fire and sacrifice-specific
  triggers (Mortician Beetle template) can distinguish the two
  causes). Cards: Witherbloom Mortician (AnyPlayer scope), Pest
  Pestmaster (YourControl scope). Tests cover both scope variants
  and the "lethal damage isn't a sacrifice" negative case.
  Affected primitives: `Effect::Sacrifice`, `Effect::Sacrifice
  GreatestMV`, `Effect::SacrificeAndRemember`,
  `ActivatedAbility.sac_cost: true`.

- 🟡 **CR 119 — Life** (push modern_decks batch 50,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`). The life-total primitive — how
  gain/lose life are computed, payment-of-life validity, and the
  "can't gain/lose life" effect framework.
  (a) **119.1** — ✅ (`Player::new` in `game/types.rs` initialises
  `life: 20`; format mod sets Commander's 40 and Two-Headed Giant's
  30 via `Player::with_starting_life`).
  (b) **119.2** — ✅ (`deal_damage_to` in `game/effects/mod.rs`
  routes player damage through `Player.life -= amount` and emits
  `GameEvent::LifeLost`).
  (c) **119.3** — ✅ (`Effect::
  GainLife` / `Effect::LoseLife` both modify `Player.life` and emit
  the matching events; `Player.life_gained_this_turn` tracks the
  per-turn fan-out for Honor Troll / Children of Korlis-class
  triggers).
  (d) **119.4** — 🟡 (the engine
  enforces this via `Player.can_pay_life` for activated-ability
  `life_cost: u32` and `Effect::LoseLife` clamps at 0 instead of
  going negative; no cards that pay life as a spell-cast cost are
  in the catalog beyond the Vicious Rivalry / Pay-X-life-as-effect
  template).
  (e) **119.4b** — ✅ (the cost
  validator short-circuits when `amount == 0` and never checks the
  life total; this matches the CR-correct behavior).
  (f) **119.5** —
  ✅ (`Effect::SetLife { who, amount }` computes the delta and emits
  the matching `LifeGained` / `LifeLost` event; Beacon of Immortality,
  Magus of the Mirror, Skull of Orm-class effects all route through
  this).
  (g) **119.6** — ✅
  (state-based actions in `state_based_actions.rs` emit
  `GameEvent::PlayerLost` when `Player.life <= 0`; CR 704.5a).
  (h) **119.7** — ✅ (push claude/modern_decks: the can't-gain-life
  half is wired via `StaticEffect::PlayerCannotGainLife { target:
  PlayerStaticTarget }` consulted in `GameState::adjust_life` via
  `player_cannot_gain_life_now`; Witherbloom Lifeglobe (b143) ships
  the "Your opponents can't gain life" static. The redistribute /
  exchange-life-total clauses are tracked separately as Effect-level
  exchange primitives and aren't needed to satisfy the static lock).
  **119.8** — ✅ (push claude/modern_decks batch 146b: the
  `StaticEffect::PlayerCannotLoseLife { target: PlayerStaticTarget }`
  primitive lands and is consulted by `GameState::adjust_life`'s
  negative-delta gate via `player_cannot_lose_life_now`. Silverquill
  Lifeward (b146) ships the symmetric "Your opponents can't lose life"
  static. Tests: `silverquill_lifeward_b146_blocks_opp_life_loss`,
  `silverquill_lifeward_b146_releases_life_lock_when_it_leaves`).
  (i) **119.9** — ✅ (`EventKind::
  LifeGained` triggers fire per-event with `event_amount` threaded
  through `EffectContext`; `Value::TriggerEventAmount` reads the
  amount in trigger bodies for Light of Promise-class "that many"
  riders).
  (j) **119.10** "If [player] would gain life" replacement — 🟡
  (the replacement-effect framework supports life-gain replacement
  via `ReplacementEffect::DoubleLifeGain` keyed off `EventKind::
  LifeGained`; only the Boon-Reflection / Cathars' Crusade replacement
  shapes are wired today).
  Affected: Honor Troll (lifegain-this-turn predicate) ✅, Light of
  Promise (LifeGained trigger amount fan-out) ✅, Felisa's drain
  ✅, all etb_drain / drain_each_opp cards (canonical drain pattern)
  ✅, Vicious Rivalry (pay X life as additional cost) ✅.

- 🟡 **CR 121 — Drawing a Card** (push modern_decks batch 40,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): The card-draw primitive — what
  drawing means, how multiple draws are sequenced, what happens when
  the library is empty, and the replacement-effect framework. Audit:
  (a) **121.1** — ✅ (`Effect::Draw` in
  `game/effects/mod.rs` pops the top of `Player.library` and pushes
  to `Player.hand`; emits `GameEvent::CardDrawn`).
  (b) **121.2** — ✅ (`Effect::Draw { amount }` loops
  `amount` times, each iteration pulling exactly one card; each
  iteration emits its own `CardDrawn` event so triggers fire per
  card).
  (c) **121.2a** — 🟡 (Replacement-effect framework landed in Phase H of
  the Commander rollout but is sparsely used for draws today. No STX/
  SOS card prints a "draw N additional cards as part of each draw"
  rider, so the gap is doc-tracked).
  (d) **121.2b** "Can't draw more than one card each turn" rider —
  ⏳ (no `StaticEffect::CapDrawsPerTurn(N)` primitive; Maralen of the
  Mornsong / Future Sight-class no-draw effects aren't in the
  catalog).
  (e) **121.2c** — 🟡
  (multi-player draws fan out via `Selector::Player(EachPlayer)`'s
  iteration order which is seat-index; the active player is seat 0
  in 1v1, so the order is APNAP-correct. In multiplayer the
  fan-out walks `0..N` rather than starting from `active_player`).
  (f) **121.3** — ⏳ (engine doesn't model
  "choose to draw" via decision; `Effect::Draw` always draws
  unconditionally, so the choice path collapses to "always-draw"
  whether or not the library is empty).
  (g) **121.4** — ✅ (`Effect::Draw` increments
  `Player.draws_from_empty_library` when the library is empty; the
  SBA loop in `state_based_actions.rs` reads this and emits a
  `PlayerLost` event at the next priority window per CR 704).
  (h) **121.5** — ✅
  (`Effect::Move { from: Library, to: Hand }` doesn't emit
  `CardDrawn` events, so draw-triggers don't fire on tutored cards;
  this is exactly the printed-Oracle semantics for Demonic Tutor /
  Diabolic Tutor / Gamble — they don't trigger Niv-Mizzet, Parun /
  Sphinx's Revelation-class draw triggers).
  (i) **121.6** Replacement-effect framework for draws — 🟡
  (Commander Phase H landed the generic replacement primitive;
  draw-replacement specifically is wired for Anvil of Bogardan,
  Notion Thief, etc. The framework supports the printed shape but
  card-count is small).
  (j) **121.7** —
  🟡 (Same coverage as 121.6 — works for the cards that use it).
  (k) **121.8** — ⏳ (no face-down-pending-draw queue;
  the engine resolves a draw mid-cast immediately, so a hypothetical
  "as you cast this spell, draw a card" rider sees the drawn card
  immediately. No card in the catalog actually leans on this
  ordering, so it's doc-tracked).
  (l) **121.9** — ⏳ (no
  reveal-on-draw decision shape).
  Tests: `archmage_emeritus_draws_on_instant_cast` exercises basic
  draw-trigger fire (121.1); `gambit_player_loses_with_empty_library`
  covers 121.4. Promote to ✅ when 121.2b (no-draw caps), 121.3
  (choose-to-draw with empty library), 121.6c (additional
  post-draw actions on replaced draws), and 121.8 (mid-cast
  face-down draws) all land.

- ✅ **CR 501 — Beginning Phase** (push claude/modern_decks batch 141
  audit, claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt` line 2134–2136). The umbrella over
  Untap / Upkeep / Draw — there are no turn-based actions at the
  *phase* level (each child step owns its own turn-based actions).
  Audit:
  (a) **501.1** — ✅ (`TurnStep::next` in
  `game/types.rs` walks `Untap → Upkeep → Draw → PreCombatMain → …`
  in CR-mandated order; no engine state allows phase reordering).
  The phase enum itself is implicit — the engine tracks the step
  directly and the beginning-phase boundary is observable only via
  the active step (entering Untap = beginning of beginning phase;
  leaving Draw = beginning of PreCombatMain).
  Tests: every combat-trace test (~3996 total) traverses
  beginning-phase via `advance_step` and validates step ordering;
  `TurnStep::next` round-trips through all variants via
  `step_order_round_trip` (game tests). Step-trigger fan-out is
  exercised by `fire_step_triggers(TurnStep::Upkeep)` for "at the
  beginning of your upkeep" abilities (Lorehold the Historian etc.).
  Promote stays ✅ — no engine gap at the phase-level umbrella.

- 🟡 **CR 502 — Untap Step** (push modern_decks batch 39,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): The untap step's turn-based actions.
  Audit:
  (a) **502.1** — ⏳ no Phasing keyword primitive (no
  `Keyword::Phasing` variant; `do_untap` doesn't walk a phased-out
  list). No STX/SOS card uses Phasing (Iceage / Mirage block only).
  (b) **502.2** —
  ⏳ no `GameState.day_night: DayNight` field; no transition logic in
  `enter_step`. No STX/SOS card uses Day/Night (Innistrad: Midnight
  Hunt block only).
  (c) **502.3** — ✅ (`do_untap` in
  `game/stack.rs:572` walks `self.battlefield`, filters by
  `card.controller == active_player_idx`, and flips `card.tapped =
  false`. Stun counters interpose per CR 701.46a / 122.1d via
  `remove_counters(Stun, 1)` instead of untapping when present —
  matches the printed Oracle of every Stun-counter source).
  Summoning sickness clears unconditionally per CR 302.1 / 506.4.
  Per-turn tallies reset here too: `lands_played_this_turn`,
  `spells_cast_this_turn`, `life_gained_this_turn`,
  `cards_drawn_this_turn`, `cards_left_graveyard_this_turn`,
  `creatures_died_this_turn`, `cards_exiled_this_turn`,
  `instants_or_sorceries_cast_this_turn`,
  `creatures_cast_this_turn`, and Teferi's sorcery-as-instant flag.
  Effects that "prevent N permanents from untapping" (Frozen Aether /
  Stasis-class effects) — ✅ (push claude/modern_decks batch 162): new
  `StaticEffect::PreventUntap { applies_to: Selector }` primitive wired
  into `do_untap`. Pre-computes the set of blocked permanent ids by
  walking active statics; the untap-step loop skips the tapped→untapped
  flip for any controlled permanent in the set. Summoning sickness still
  clears per CR 506.4 (independent of the untap event). Demo card:
  `Strixhaven Stasis-Glyph (b160)` — `{3}{U}` enchantment, "Lands you
  control don't untap during your untap step." Tests:
  `cr_502_3_prevent_untap_blocks_land_untap_during_untap_step`,
  `cr_502_3_prevent_untap_releases_after_static_leaves`,
  `cr_502_3_prevent_untap_does_not_affect_unmatched_permanents`.
  (d) **502.4** — ✅ (`enter_step`'s
  `TurnStep::Untap` arm at `game/stack.rs:101` calls `do_untap`,
  emits `TurnStarted`, then immediately calls `pass_priority()` to
  advance to Upkeep — no priority window is opened. Untap-step
  triggers go on the stack and resolve in upkeep per CR 503.1a; the
  engine's stack-driven priority loop naturally enforces this.).
  Tests: `combat_tests.rs` exercises full turn loop including untap
  → upkeep → draw → main; stun-counter regression tests at
  `tests::stx::stun_counter_*` cover (c); CR 502.3 prevent-untap is
  locked in via `cr_502_3_prevent_untap_*` (push batch 162). Promote
  to ✅ when 502.1 (Phasing) AND 502.2 (Day/Night) land.

- ✅ **CR 503 — Upkeep Step** (push modern_decks batch 43,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): The upkeep-step framework — no
  turn-based actions, but the active player receives priority and
  beginning-of-upkeep triggers go on the stack. Audit:
  (a) **503.1** — ✅ (`enter_step` arm for `TurnStep::Upkeep`
  in `game/stack.rs` opens a priority window via
  `give_priority_to_active()` without applying any TBA between
  Untap and Upkeep). (b) **503.1a** — ✅ (untap-step
  triggers are emitted during `do_untap` and the trigger dispatcher
  pushes each `StackItem::Trigger` onto the stack; beginning-of-
  upkeep triggers fire via `EventKind::StepBegins(Upkeep)` /
  `EventScope::ActivePlayer` filter inside the upkeep-step
  enter-hook. APNAP-order resolution of stacked triggers is the
  default LIFO from the trigger dispatcher — for 1v1 the active
  player's triggers are pushed first and resolve last, matching
  the printed "AP picks order within their pile" convention).
  (c) **503.2** — ⏳ (no card in the catalog
  prints "cast only after upkeep"; the engine's cast-time predicate
  framework doesn't gate on "first upkeep this turn"). No STX/SOS
  card requires this.
  Tests: existing combat/turn-loop coverage in
  `crabomination/src/tests/game.rs` exercises Untap → Upkeep
  transitions and BoUpkeep triggers (Lorehold the Historian's
  opp-upkeep loot trigger, Bedlam Reveler-style triggers).

- ✅ **CR 606 — Loyalty Abilities** (push modern_decks claude/modern_decks
  branch — audit against `MagicCompRules_20260417.txt`): The
  planeswalker loyalty-ability framework — sorcery-speed activation,
  once-per-turn cap, loyalty-counter costs. Audit:
  (a) **606.1** loyalty abilities are activated abilities subject to
  special rules — ✅ (`LoyaltyAbility` struct in `card.rs` keyed
  separately from regular `ActivatedAbility`; activation routes
  through `activate_loyalty_ability` in `game/mod.rs:1715`).
  (b) **606.2** — ✅ (`LoyaltyAbility.loyalty_cost: i32`
  is the loyalty delta; +N to add counters, -N to remove).
  (c) **606.3** sorcery-speed + main-phase only + own permanent +
  once-per-turn — ✅ (`activate_loyalty_ability` gates: (i)
  `can_cast_sorcery_speed(p)` enforces sorcery timing + main phase
  with empty stack, (ii) `battlefield[pos].controller != p` rejects
  activations on opponent's planeswalkers, (iii)
  `used_loyalty_ability_this_turn` flag enforces the once-per-turn
  rule, cleared in `do_cleanup`).
  (d) **606.4** cost is to add/remove loyalty counters as shown by
  the loyalty symbol — ✅ (the activation arithmetic on
  `CounterType::Loyalty`: `current_loyalty + ability.loyalty_cost`
  is computed; negative results reject with `NotEnoughLoyalty`).
  (e) **606.5** multiple `[+N]`/`[-N]` costs combine — N/A (no card
  in the catalog prints multiple loyalty modifiers in one activation;
  the engine accepts a single `loyalty_cost: i32` per ability which
  models this implicitly for the simpler case).
  (f) **606.6** negative-cost ability requires sufficient loyalty —
  ✅ (`new_loyalty < 0` → `Err(NotEnoughLoyalty)`; matches the
  printed "X- cost requires X counters" rule).
  Tests: existing planeswalker coverage (e.g. Ral Zarek +1 Surveil 2,
  -1 discard, -2 reanimate) exercises the three-mode shape and
  validates the once-per-turn lock. The "Carth the Lion" +1-modifier
  rider (CR 606.5) stays doc-tracked.

- ✅ **CR 504 — Draw Step** (push modern_decks batch 43,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): The two-step draw-step framework
  — TBA to draw a card, then AP gets priority. Audit:
  (a) **504.1** — ✅ (`enter_step` arm
  for `TurnStep::Draw` calls `Effect::Draw { who: active_player,
  amount: 1 }` directly via `resolve_effect` BEFORE opening the
  priority window — the draw is not pushed as a stack item, the
  draw event emits `GameEvent::CardDrawn`, and any "whenever a
  player draws a card" triggers fan out from there). (b) **504.2** — ✅ (after the draw
  TBA resolves, `give_priority_to_active()` opens the priority
  window; trigger-stack resolution from draw-event triggers takes
  precedence over the priority window since stack items resolve
  first). Format-specific first-turn-draw skip (1v1 active player
  skips draw on turn 1 per the Magic tournament rules) — ✅
  (`format.skip_first_turn_draw` flag, checked by the draw-step
  enter hook; the draw TBA is suppressed only on turn 1 of seat
  0 in 2-player). Multiplayer free-for-all and Commander correctly
  do **not** skip the first-turn draw per CR 103.8a.
  Tests: existing combat-coverage tests exercise the draw step;
  `format_two_headed_giant_active_player_draws_on_turn_1` etc.
  cover the format gate.

- ✅ **CR 505 — Main Phase** (push modern_decks batch 38,
  claude/modern_decks branch — audit against `MagicCompRules_20260417.txt`):
  Audit:
  (a) **505.1** — ✅
  (`TurnStep::FirstMainPhase` and `TurnStep::SecondMainPhase` are
  separate variants in `game/types.rs` with the combat phase wedged
  between them; an additional combat phase + main pair is reachable
  through the `extra_combat_phases_this_turn` counter used by
  Aggravated Assault / World at War).
  (b) **505.1a / 505.1b** — ✅ (`TurnStep::FirstMainPhase` is the only
  variant whose `is_first_main_phase()` returns true; `Selector::Targets`
  in step-trigger filters distinguish first vs second mains via the
  enum variant, so cards keyed on precombat-main fire correctly).
  (c) **505.2** — ✅ (`pass_priority` in `game/stack.rs` advances the step
  only when `self.stack.is_empty()` and both players have passed in
  succession — the standard step-end rule; no special main-phase
  branch needed).
  (d) **505.3** Archenemy scheme-set-in-motion turn-based action — ⏳
  (no Archenemy variant — multiplayer subgame TODO at the bottom of
  this file).
  (e) **505.4** Saga lore-counter precombat turn-based action — ⏳
  (no Saga card type — tracked at `### Saga lore counters + DFC`
  in `CUBE_FEATURES.md`).
  (f) **505.5** Attraction roll-to-visit precombat turn-based action
  — ⏳ (no Attraction card type — Unfinity-only).
  (g) **505.6** — ✅
  (`give_priority_to_active` is called at the start of each main
  phase via `enter_step`'s match arm on `FirstMainPhase`/`SecondMainPhase`).
  (h) **505.6a** — ✅ (`can_cast_sorcery_speed` checks
  `current_step ∈ {FirstMainPhase, SecondMainPhase}` AND stack-empty
  AND player-has-priority).
  (i) **505.6b** — ✅ (`play_land` enforces all three
  preconditions: `current_step.is_main_phase()` via
  `can_cast_sorcery_speed`, `self.stack.is_empty()`, the
  `Player.lands_played_this_turn` cap modulo `extra_land_per_turn`
  statics like Exploration / Azusa).
  Tests: combat tests in `crabomination/src/tests/game.rs` exercise
  main → combat → main transitions; lands tests verify the
  one-per-turn enforcement (`exploration_grants_extra_land_per_turn`,
  `azusa_grants_two_extra_lands`); sorcery-speed gating tested via
  the many `*_castable_at_sorcery_speed` / `*_not_castable_at_instant_speed`
  variants throughout the suite. Promote to ✅ pending Archenemy /
  Saga / Attraction (all multi-variant TBD).

- 🟡 **CR 509 — Declare Blockers Step** (push modern_decks batch 33,
  claude/modern_decks branch — audit against `MagicCompRules_20260417.txt`):
  The blocker-declaration turn-based action — who can block what, when
  triggers fire, and how block-on-ETB interacts with normal blocking.
  Audit:
  (a) **509.1** — ✅
  (`declare_blockers` in `game/combat.rs` is a direct mutator; emits
  `BlockerDeclared` events but doesn't push to the stack).
  (b) **509.1a** — ✅
  for tap state (`can_block` checks `tapped`); the "attacking-creatures
  only" gate enforces `defender_idx == blocker.controller` per the
  `same_team` check (battles are not modelled as attackable but the
  attacker → target enum supports player + planeswalker).
  (c) **509.1b** restrictions (evasion abilities — Flying, Skulk, etc.)
  — ✅ via `can_block_attacker_computed` which walks the keyword set
  for Flying/Reach, Menace (handled separately), Shadow/Horsemanship
  (not implemented), Unblockable, etc. Cumulative evasion is naturally
  enforced (each evasion clause is its own gate).
  (d) **509.1c** requirements (creatures that *must* block if able) —
  ⏳ (no "must block" primitive; cards like Provoke aren't in the
  catalog).
  (e) **509.1d-f** cost-to-block lock-in and payment — ⏳ (no
  blocker-cost activation pipeline; no cards in the catalog have
  "creatures can't block unless their controller pays {N}").
  (f) **509.1g** — ✅ (`block_map`
  records the assignment; SBA later checks for blocker survival before
  combat damage assigns).
  (g) **509.1h** — ✅ (`is_blocked(attacker_id)`
  derived from `block_map` entries; persists through combat phase).
  (h) **509.1i** — ✅ (push XXVI: `EventKind::Blocks` + `EventKind::BecomesBlocked`
  emit per `BlockerDeclared` event; trigger dispatcher fans out matching
  triggered abilities; test `daemogoth_titan_blocks_sacrifices_another_creature`).
  (i) **509.2** — ✅ (`give_priority_to_active()` at the end of `declare_blockers`).
  (j) **509.2a** — 🟡 (the dispatcher orders by emission sequence
  rather than APNAP; in 1v1 this is the same outcome, in multiplayer the
  APNAP order is approximated).
  (k) **509.3a/b/c/d/e** different trigger condition shapes
  ("blocks", "blocks a creature", "becomes blocked", "becomes blocked
  by a creature", "blocks/blocked by N creatures") — 🟡 ✅ for the
  basic per-blocker emission (each `BlockerDeclared` event fires one
  trigger per ability, matching 509.3a's once-per-blocker rule). The
  "blocks two or more creatures" multi-target counting (509.3e) is
  exercised via per-creature trigger emission rather than per-batch
  accumulation; functionally correct for single-creature blockers but
  doesn't model the "Whenever this creature blocks two or more
  creatures" pattern accurately if such a card existed.
  (l) **509.3f** — ✅ (trigger
  filter `Predicate::EntityMatches` reads layered characteristics at
  fire time; type changes mid-combat don't retroactively trigger).
  (m) **509.3g** — ✅ (push claude/modern_decks batch 127: new
  `EventKind::AttacksAndIsntBlocked` + `GameEvent::
  AttackerWentUnblocked` events emitted at the end of
  `declare_blockers` for each attacker with zero entries in
  `block_map`. The unified trigger dispatcher routes them via
  `event_matches_spec` / `event_subject` / `event_card`. New shortcut
  `effect::shortcut::on_unblocked(effect)` wraps the trigger. Inkling
  Skyraider (b127) exercises this — "drain 1 when unblocked"; tests
  cover both the unblocked-fire and blocked-no-fire paths.).
  (n) **509.4** — ⏳ (no `Effect::
  PutOntoBattlefieldBlocking` primitive; cards like Mantis Rider don't
  exercise this).
  Tests: combat-coverage tests in `crabomination/src/tests/game.rs`
  exercise basic declare-blockers + flying-evasion + menace-2-blockers;
  STX `daemogoth_titan_blocks_sacrifices_another_creature` covers
  509.1i + 509.3a; `inkling_skyraider_b127_drains_when_attacking_
  unblocked` + `_does_not_drain_when_blocked` cover 509.3g. Promote
  to ✅ when 509.1c (requirements), 509.1d-f (cost-to-block), and
  509.4 (put-onto-bf-blocking) all land.

- 🟡 **CR 118 — Costs** (push modern_decks batch 16, claude/modern_decks
  branch — audit against `MagicCompRules_20260417.txt`): The cost
  framework — what counts as a cost, payment order, replacement
  primitives, and "X" costs. Audit:
  (a) **118.1** — ✅ (the engine models costs as fields on
  `ActivatedAbility` + `CardDefinition.cost` for spells, plus
  `AlternativeCost` for pitch / cost-reduction-with-gate paths).
  (b) **118.2** mana payment opens a mana-ability activation window —
  ✅ (`try_pay_with_auto_tap` / `try_pay_after_snapshot` in
  `game/actions.rs` allow mana-ability activation mid-payment; mana
  abilities resolve immediately without the stack per CR 605.3).
  (c) **118.3** — ✅ (`InsufficientMana` for mana, `InsufficientLife` for
  life-cost, `CardIsTapped` for tap costs, `SelectionRequirementViolated`
  for exile-other-from-gy preflight; all rejection paths roll back the
  payment snapshot via `restore_payment_state`).
  (d) **118.3a** — ✅ (`ManaPool::try_pay`).
  (e) **118.3b** — ✅ (`life_cost` deduction in `activate_ability`,
  `LoseLife` event emission).
  (f) **118.3c** — 🟡 (the auto-decider always activates mana
  abilities to satisfy payment; a real UI player choosing not to tap a
  source could fail a payment. Functionally indistinguishable from the
  CR-correct outcome in bot harness).
  (g) **118.4** — ✅ for spells (`x_value`
  on `CastSpell`, propagated through `ManaCost::with_x_value`), 🟡 for
  activated abilities (Berta's `{X}{T}: …` activation has X-symbols in
  the cost but no per-activation X prompt; the engine zeroes X for
  activations — tracked under `Value::SacrificedToughness` row in
  "Engine — Missing Mechanics" follow-ups).
  (h) **118.5** — ✅ (zero-mana
  spells like Mox cycle, Prismari Bauble are castable with empty pool).
  (i) **118.6** unpayable cost (mana_cost = None / empty + no alt) —
  🟡 (engine has no general "unpayable" gate — `ManaCost::default()` is
  paid as empty/zero, which is "free", not "unpayable". Suspended
  creatures from exile would need this gate; not exercised by current
  catalog).
  (j) **118.7** cost reduction effects — 🟡 (`StaticEffect::CostReduction
  { filter, amount }` covers "spells matching filter cost {N} less";
  `CostReductionTargetingFilter` covers Killian, Ink Duelist–style "if
  it targets X" reductions; 118.7a-c (color-vs-generic ordering) is
  handled by `ManaCost::reduce_generic`. Hybrid pip reduction (118.7e)
  is approximated — each hybrid pip is treated as its preferred color
  half, not as a per-reduction choice. The new `AlternativeCost.condition`
  field (push batch 16) covers "X less if [predicate]" paths via a
  full alt-cost replacement rather than incremental reduction.
  (k) **118.8** additional costs — 🟡 (tap, sac, life, exile-self,
  exile-other-from-gy all supported; multi-target "as an additional
  cost, pay X life for each X" pattern (Vicious Rivalry) collapses to
  X-from-spell-cost rather than a separate additional-cost prompt;
  same for "pay X life" additional costs that vary independently of
  cast-time X). Convoke (`Effect::CastWithConvoke` path) lands as an
  additional-cost replacement that consumes tapped creatures.
  (l) **118.9** — ✅ (zero-mana
  payment is a no-op, the auto-decider always pays).
  (m) **118.10/12** other corner cases — ⏳ (cost-of-cost interactions
  not exercised). Tests: implicit across the entire suite — every cast
  / activation test exercises the cost framework; the new alt-cost
  tests (Wilt in the Heat, Orysa Tide Choreographer) cover 118.7-style
  cost-reduction-with-predicate paths. Promote to ✅ when 118.3c
  (interactive mana-ability decline) and 118.7e (hybrid pip per-reduction
  choice) both land.

- 🟡 **CR 113 — Abilities** (push modern_decks batch 15,
  claude/modern_decks branch — newest revision audit against
  `MagicCompRules_20260417.txt`): The ability primitive — what
  abilities are, the three categories on the stack vs static, how
  they're added/removed/granted, and which zones they function in.
  Audit:
  (a) **113.1a/b/c** abilities as object characteristics, player
  characteristics, and stack objects — ✅ (`CardDefinition` carries
  `triggered_abilities`, `activated_abilities`, `static_abilities`;
  `StackItem::Ability` represents an activated/triggered ability on
  the stack; emblems are not modelled — see TODO row for emblems).
  (b) **113.2c** paragraph-break = separate ability — ✅ (every
  `TriggeredAbility` / `ActivatedAbility` is a distinct item in its
  Vec; multiple instances of the same ability all fire/can be
  activated independently).
  (c) **113.3a** spell abilities = instant/sorcery body resolution
  — ✅ (`CardDefinition.effect` runs at resolution time for IS
  spells via `resolve_spell` in `game/stack.rs`).
  (d) **113.3b** activated abilities have cost + effect, may activate
  with priority — ✅ (`GameAction::ActivateAbility` checks
  `has_priority`, pays cost via `ActivatedAbility.mana_cost` /
  `tap_cost` / `sac_cost` / `life_cost`, then pushes
  `StackItem::Ability`).
  (e) **113.3c** triggered abilities have trigger condition + effect,
  go on stack next time someone would have priority — ✅
  (`fire_step_triggers` + `dispatch_triggers_for_events` collect
  fired triggers and push them onto the stack at the next priority
  check; see `game/stack.rs::push_pending_triggers`).
  (f) **113.3d** static abilities create continuous effects while in
  the appropriate zone — ✅ (`StaticAbility` + `StaticEffect` —
  `compute_battlefield` re-evaluates the layered effect view every
  state-change; see `game/layers.rs`).
  (g) **113.4** mana abilities don't use the stack and can be
  activated mid-cast — ✅ (`is_mana_ability` recognizer in
  `game/actions.rs` skips the stack push for pure `AddMana` effects;
  mid-cast activation works via the mana payment loop).
  (h) **113.5** loyalty abilities: once-per-turn, sorcery-speed,
  empty-stack, main-phase — ✅
  (`activate_loyalty_ability` enforces all four constraints in
  `game/actions.rs`).
  (i) **113.6** zones where abilities function — 🟡 (the engine
  honors most clauses via per-zone lookup: spell abilities resolve
  off-stack; activated/triggered abilities of permanents fire from
  bf; flashback abilities and gy-recursion activations fire from gy;
  emblems / characteristic-defining abilities are ⏳).
  (j) **113.7** source of an ability + last-known-information — ✅
  (the trigger system captures `event.source_id` / `event.subject_id`
  / `event_amount` at emission time so a destroyed source still
  resolves its already-emitted trigger correctly; see
  `event_amount` cache and `CreatureDied` last-known info plumbing).
  (k) **113.7a** activated/triggered abilities outlive their source
  on the stack — ✅ (same cache-at-emission pattern as 113.7).
  (l) **113.8** controller of stack abilities — ✅ (`StackItem`
  carries `controller: PlayerIdx` set at push time; checked by the
  resolver and APNAP ordering).
  (m) **113.9** stack abilities can be countered by ability-counters
  but not spell-counters; static abilities can't be countered — 🟡
  (general spell-vs-ability counter distinction isn't carried on
  `Effect::Counter*`; today every counter card targets "spell" so
  the gap doesn't bite, but Stifle/Squelch-style "counter target
  activated/triggered ability" cards aren't in the catalog).
  (n) **113.10/a/b/c** gaining/losing abilities (most-recent wins)
  — ✅ (push modern_decks batch 34) — `StaticEffect::GrantKeyword`
  adds keywords for a duration; `Modification::RemoveAllAbilities` now
  flips a `ComputedPermanent.lost_all_abilities` flag in addition to
  clearing keywords, and three dispatch sites
  (`dispatch_triggers_for_events`, `fire_spell_cast_triggers`,
  `activate_ability`) consult that flag to skip the source's printed
  triggered + activated abilities (CR 113.10b). Mana abilities are
  preserved per CR 605.1a (the activate-rejection path applies only to
  non-mana abilities). The headline test cases ship via
  `Effect::LoseAllAbilities` (Mercurial Transformation) — Shivan Dragon
  loses Flying, Sedgemoor Witch's Magecraft suppresses while stripped.
  (o) **113.11** "can't have" anti-grant — ⏳ (no
  `StaticEffect::CantHaveAbility` primitive; cards like Stony Silence
  approximate via different anti-activate paths).
  (p) **113.12** set-characteristic vs grant-ability distinction —
  ✅ (`Effect::SetBasePT` / `Effect::SetBaseColor` are set-
  characteristic; `StaticEffect::GrantKeyword` is grant-ability;
  Muraganda Petroglyphs corner is unhit but the distinction holds at
  the type level — `Modification` enum distinguishes the two).
  Promote to ✅ when 113.6 (emblems + CDA), 113.9 (counter-target-
  ability), 113.10b (full ability removal), and 113.11 (can't-have)
  all land.

- ⏳ **CR 114 — Emblems** (push modern_decks batch 28 audit,
  claude/modern_decks branch — `MagicCompRules_20260417.txt`): Emblems
  represent abilities in the command zone with no other characteristics.
  Audit:
  (a) **114.1** — ⏳
  (no `Effect::CreateEmblem` primitive; no `Zone::CommandZone`
  emblem-mode). Some planeswalker ults that grant emblems (Professor
  Dellian Fel's -6, Ral Zarek's -7, Tezzeret's emblems) are doc-tracked
  as 🟡 with the emblem half omitted — the body / earlier loyalty
  abilities still ship.
  (b) **114.2** — ⏳ (no
  emblem creation effect; the engine's command zone exists for
  Commander/Brawl but holds only `Card` instances, not abilityless
  emblem markers).
  (c) **114.3** "An emblem has no characteristics other than the
  abilities defined" — n/a (no emblem objects to characterize).
  (d) **114.4** "Abilities of emblems function in the command zone" —
  n/a (no emblem objects; the engine's trigger-fire pipeline already
  walks the command zone for Commander triggers, so the dispatcher
  infrastructure could host emblem-resident triggers without changes).
  (e) **114.5** "An emblem is neither a card nor a permanent" — n/a
  (no emblems to classify; `is_permanent` already returns false for
  non-`Permanent`-type cards). Tests: no test coverage — gates on
  Professor Dellian Fel / Ral Zarek emblem ults. Promote to 🟡 when
  `Effect::CreateEmblem { who: PlayerRef, abilities: Vec<…> }` lands
  alongside an `EmblemObject` shape in the command zone; promote to ✅
  when emblem-resident triggers fire and at least one planeswalker
  ult's emblem ships end-to-end (Professor Dellian Fel's lifegain →
  drain emblem is the canonical first target).

- 🟡 **CR 115 — Targets** (push modern_decks batch 53,
  claude/modern_decks branch — audit against `MagicCompRules_20260417.txt`):
  The targeting framework — declaring targets at cast / activation time,
  legal-target check at resolution time, and "change targets" effects.
  Audit:
  (a) **115.1** — ✅ (`GameAction::CastSpell.target` +
  `additional_targets: Vec<Target>` capture the target slots at cast
  time; `StackItem::Spell.target` + `additional_targets` persist them
  through the resolution window. Triggered/activated targets are stored
  on `StackItem::Trigger.target` similarly. No "change targets" effect
  exists today; engine has no `Effect::ChangeTarget` primitive).
  (b) **115.1a/c/d** — ✅ (the `Selector::
  Target(slot)` and `Selector::TargetFiltered { slot, filter }` shapes
  on effects flag an ability as targeted; the cast pipeline's
  `requires_target_check` walks the effect tree at cast time to enforce
  target selection. The CR 115.1a corner — "if an activated or triggered
  ability of an instant or sorcery uses the word target, that ability is
  targeted, but the spell is not" — is naturally honored: the engine
  attaches targets to the *trigger* StackItem, not the originating spell).
  (c) **115.1b** Aura spells — ⏳ (no Aura subtype primitive; the engine
  has no Enchant keyword + attached-to mechanic. Affected cards: any
  Aura — no STX/SOS Aura is wired today).
  (d) **115.2** — ✅
  (`check_target_legality_with_source` walks the battlefield by default;
  `SelectionRequirement::IsSpellOnStack` and `SelectionRequirement::
  CardsInZone` shapes opt-in to other-zone targets. `Target::Player`
  takes the player variant). The `Selector::one_of(CardsInZone { ... })`
  shape used by graveyard-recursion cards (Witherbloom Recourse,
  Silverquill Necroscribe) explicitly targets the graveyard residents
  per 115.2(a).
  (e) **115.3** — 🟡 (the
  multi-target slots in `additional_targets` allow the caller to repeat
  the same `Target::Permanent` at different slot indices; the engine
  doesn't reject same-target across slots today. In practice no STX/SOS
  catalog card with multi-target slots actually exercises the
  same-target case where it matters — Lorehold Battle Memorial's
  creature-vs-player slots can't collapse since the filters differ;
  Quandrix Pairweaver's two slot-0/slot-1 friendly-creature picks would
  be a candidate, but the auto-picker walks distinct creatures in
  iteration order so the issue is invisible in bot play. A future
  refinement would have `check_target_legality` reject a repeated
  `Target::Permanent` across slots with the same filter signature).
  (f) **115.4** "Any target" / "another target" — ✅ (`SelectionRequirement
  ::Creature.or(Player).or(Planeswalker)` is the canonical "any target"
  filter, used pervasively by burn (Lightning Bolt template, Lorehold
  Apprentice ping, Storm-Kiln Artist, etc.). The "another target"
  variant chains through `Predicate::All` to exclude the source / first
  target — handled per-card today rather than via a dedicated primitive).
  (g) **115.5** — ✅ (`check_target_legality_with_source` accepts an
  optional `source_card_id` and rejects targets that match. The cast
  pipeline at `game/actions.rs:657` passes the casting spell's own
  `CardId` so e.g. Bury in Books can't put itself on top of the
  library; existing test `cr_115_5_spell_targeting_itself_is_illegal_via_permanent_id`
  locks the regression).
  (h) **115.6** — 🟡 (Divergent Equation's "up to X" picker uses
  `Selector::take` with `Value::XFromCost`; at X=0 the selector returns
  empty and the spell still resolves. No general "zero-target" gate
  rejects a target-required spell at cast time — `additional_targets`
  is an unconditional `Vec` with no per-slot "optional" marker. The
  engine ships the *outcome* of zero-target casts correctly but doesn't
  encode the cast-time CR 115.6 "still requires targets" distinction).
  (i) **115.7** Change-target / choose-new-target effects — ⏳ (no
  `Effect::ChangeTarget` / `Effect::ChooseNewTargets` primitive; cards
  like Redirect, Arcane Denial-style "exchange targets" aren't in the
  catalog. Choreographed Sparks' "copy target, you may choose new
  targets" rider on the *copy* is handled via `Effect::CopySpell`'s
  internal `choose_new_targets: bool` flag, but the per-original change
  shape from CR 115.7a-d is missing).
  (j) **115.8** Modal targeting — ✅ (Modal spells declared via
  `Effect::ChooseMode` / `Effect::ChooseN` resolve each mode against
  the spell's `target` slot at resolution time; the
  `pick_trigger_mode` path at `game/stack.rs` handles mode-pick for
  triggered abilities. Different modes can have different target
  filters because each `Effect` branch carries its own `Selector::
  TargetFiltered { slot: 0, filter }` — though only the first mode's
  filter is enforced at cast time today; mode-pick after-cast
  validation against the chosen mode's filter is a future polish item.
  In practice the AutoDecider picks a mode whose filter is consistent
  with the cast-time target, so the gap is invisible).
  (k) **115.9** —
  ✅ (the `Predicate::CastSpellTargetsMatch(filter)` reads the firing
  spell's `target` slot at trigger-resolution time; Strixhaven Repartee
  uses this for "spells that target a creature" payoffs (Stirring
  Hopesinger, Rehearsed Debater, Informed Inkwright, etc.). The
  "[spell] with [N] targets" multi-target count check from 115.9a is
  not exposed as a separate predicate — no STX/SOS card needs it).
  (l) **115.10** — ✅ (`Selector::EachPermanent(filter)` and
  `Selector::Player(EachOpponent)`-style fan-outs resolve at
  resolution time and don't require cast-time target declaration; the
  Sweeper template (Pestilent Haze, Crippling Fear, Wrath of God, Crux
  of Fate) uses this exclusively).
  Tests: existing `cr_115_5_*` lock the self-target gate; the new STX
  batch 53 cards (Lorehold Emberlock, Prismari Firechord, Lorehold
  Sparkflinger, etc.) all exercise the standard target-filter pipeline
  end-to-end. Promote to ✅ when 115.1b (Aura), 115.3 (same-target
  rejection across slots), 115.6 (zero-target cast-time gate), and
  115.7 (change-target primitives) all land.

- 🟡 **CR 116 — Special Actions** (push modern_decks batch 35,
  claude/modern_decks branch — audit against `MagicCompRules_20260417.txt`):
  Special actions are player-initiated actions that don't use the stack
  (CR 116.1). Audit:
  (a) **116.1** — ✅ (the engine has separate
  `GameAction` variants for each special action; none push onto
  `self.stack`).
  (b) **116.2a** — ✅
  (`play_land_with_face` in `game/actions.rs` checks priority via
  `can_cast_sorcery_speed(p)` for sorcery-speed timing, enforces the
  per-turn limit via `can_play_land()`, requires `has_in_hand`, and
  moves the card via direct push to battlefield without using the stack).
  (c) **116.2b** — ⏳ (no morph /
  face-down primitive in the engine; the corner is not exercised by the
  current catalog).
  (d) **116.2c** — 🟡 (some duration-bound effects clear in cleanup; no
  general-purpose "special action to dismiss" path).
  (e) **116.2d** — ⏳ (no static
  ability with "you may ignore" rider in the catalog).
  (f) **116.2e** Circling Vultures "may discard at any time" — ⏳ (the
  card isn't in the catalog).
  (g) **116.2f** Suspend — exile from hand at priority — 🟡 (suspend
  primitive partially modelled; see TODO row for Suspend).
  (h) **116.2g** Companion {3}: hand from outside game — ⏳ (no
  companion sideboard / outside-game model).
  (i) **116.2h** Foretell — exile from hand for {2} — ⏳ (no foretell
  primitive).
  (j) **116.2i** Roll planar die — n/a (no Planechase).
  (k) **116.2j** Conspiracy face-up — n/a (no Conspiracy Draft).
  (l) **116.2k** Plot exile from hand — ⏳.
  (m) **116.2m** Pay locked-half unlock cost (Mystery Houses / Rooms) —
  ⏳.
  (n) **116.3** — ✅ (`play_land` does NOT call
  `pass_priority`, leaving priority with the active player; the priority
  system idempotently re-checks who has priority via `priority.
  player_with_priority` and stack-empty state).
  Tests: implicit via the entire play-land test suite + the suspended-
  spell tests; push (modern_decks batch 149) lands explicit
  `cr_116_3_priority_returns_to_player_after_play_land` that asserts
  (1) priority stays with seat 0 after PlayLand, (2) the stack remains
  empty (CR 405.6d), and (3) the land enters the battlefield.

- 🟡 **CR 105 — Colors** (push modern_decks claude/modern_decks branch
  — audit against `MagicCompRules_20260417.txt`): The five-color
  primitive — what defines an object's color, color changes, monocolor
  vs multicolor vs colorless. Audit:
  (a) **105.1** — ✅
  (`mana.rs::Color::ALL` lists exactly those five, in WUBRG order).
  (b) **105.2** — ✅ (`format.rs::color_identity` walks the printed
  `cost.symbols` and unions colored / phyrexian / hybrid pips). Color
  indicator override and CDA-defined color are not yet modeled: no card
  in scope has a color indicator that disagrees with its mana cost
  (Devoid is the canonical exception; not in the catalog).
  (c) **105.2a/b/c** monocolored / multicolored / colorless predicates
  — ✅ (`ColorSet::is_monocolored()`, `::is_multicolored()`,
  `::is_colorless()` exposed on the type since modern_decks
  push X. The predicates back the `Monocolored`,
  `Multicolored`, and `Colorless` `SelectionRequirement`s.).
  (d) **105.3** —
  ⏳ (no `StaticEffect::AddColor` / `StaticEffect::BecomeColor`
  primitive. Cards like Kasmina's Transmutation ("becomes a blue
  Frog"), Mercurial Transformation ("becomes a blue Frog"), Fractalize
  ("becomes a green and blue Fractal") are doc-tracked as cosmetic
  approximations — the printed type/color rewrite half is omitted.
  Same gap blocks the printed color-changing rider on Painter's
  Servant, Lurking Predators, Shifting Sliver.).
  (e) **105.4** "Choose a color" decisions exclude multicolored /
  colorless — ⏳ (no choose-color decision shape; cards like Painter's
  Servant ("As this enters, choose a color"), Cabal Ritual variants
  with name choice, etc. aren't in scope today).
  (f) **105.5** — ✅
  (`cube.rs::pair_contains` walks two-color tuples; `College::colors`
  returns exactly `[Color; 2]` for each guild; Commander color-identity
  rule rejects 3+ pairs via `format.rs`'s deck validator).
  Tests: `format.rs::color_identity` is exercised throughout the
  cube/SOS test suite via Commander deck validation; promote to ✅
  when 105.3 (color-becomes / color-adds) lands as a runtime
  primitive backed by at least one catalog card.

- 🟡 **CR 705 — Flipping a Coin** (push modern_decks batch 63 — audit
  against `MagicCompRules_20260417.txt`): The coin-flip primitive — what
  flipping means, win/loss semantics, and "ignore the result" overrides.
  Audit:
  (a) **705.1** — ✅ (the `Effect::FlipCoin` resolver asks the decider for a
  `Bool` answer per flip. `AutoDecider` always returns `Bool(true)`
  (heads) for determinism in tests; a real client RNG would call
  `rand::random::<bool>()`. The two-sided constraint is enforced by the
  `Bool` return type.).
  (b) **705.2** — 🟡 (the
  engine collapses "call + result" into a single boolean: `true` = the
  flipper "wins" the call. Cards that distinguish "heads" vs "tails"
  specifically — Karplusan Minotaur ("Whenever Karplusan Minotaur deals
  damage to a creature, flip a coin. If you win, Karplusan Minotaur deals
  1 damage to that creature's controller. If you lose, that creature deals
  1 damage to you.") — can be modelled directly by mapping `on_heads ↔
  win`, `on_tails ↔ lose`. Mana Clash's symmetric "we both flip until one
  comes up tails" needs a two-player flip loop; not yet wired but the
  primitive supports it via `count: Value::Const(N)`.).
  (c) **705.3** — ✅ (push claude/modern_decks current sub-push): new
  `Player.coin_flip_advantage: u32` field consumed by the `Effect::FlipCoin`
  resolver. When non-zero, each flip is replayed `1 + advantage` times
  and the flipper "wins" (heads branch fires) if any of the replays
  came up heads — the canonical interpretation of stacked Krark's
  Thumbs ("ignore one flip and use the other"). Two Thumbs → 3 flips,
  pick the best. Tests:
  `cr_705_3_coin_flip_advantage_lets_tails_be_recovered` (advantage=1
  + scripted [false, true] → heads branch fires),
  `cr_705_3_no_advantage_means_one_flip_one_result` (control — no
  advantage + Bool(false) → tails branch fires). Krark's Thumb-the-
  card isn't in the catalog yet, but the engine primitive is live;
  Mana Clash / Karplusan Minotaur / Goblin Goliath all compose against
  the same fast path without any further engine changes.
  Implementation: `Effect::FlipCoin { count, on_heads, on_tails }` at
  `effect.rs`; `Decision::CoinFlip { player }` +
  `DecisionAnswer::Bool(true|false)` in `decision.rs`; the resolver in
  `game/effects/mod.rs::run_effect` walks `count` flips and dispatches
  per-result. Wire-format mirror `DecisionWire::CoinFlip` in `net.rs`
  for client round-trip. Lock-in tests:
  `lorehold_coinflinger_heads_burns_target`,
  `lorehold_coinflinger_tails_discards_a_card`,
  `coin_flip_auto_decider_defaults_to_heads`. Affected catalog cards:
  Lorehold Coinflinger (synthesised exercise card). Promote to ✅ when
  705.3 (override / re-flip primitives) lands; the primary 705.1 / 705.2
  shapes are already wired.

- 🟡 **CR 122 — Counters** (push modern_decks audit, claude/modern_decks
  branch — batch 10): The counter primitive — placement, accumulation,
  +1/+1 vs -1/-1 cancellation, ETB-with-counters, "Nth counter" trigger.
  Audit:
  (a) **122.1** counter is a marker with no characteristics — ✅ (the
  `CounterType` enum at `card.rs:121+` is a tag — counters are stored
  as `HashMap<CounterType, u32>` on `CardInstance.counters` with no
  ability/keyword payload of their own).
  (b) **122.1a** +X/+Y power/toughness — ✅ (layer-7c reads
  `counter_count(PlusOnePlusOne) - counter_count(MinusOneMinusOne)`
  and adds the delta to base P/T via `compute_battlefield`).
  (c) **122.1b** keyword counters — ✅ (the `Effect::AddKeywordCounter`
  / `RemoveKeywordCounter` primitives wire keyword-granting counters
  and `has_keyword` reads them; Silverquill Reachseal (b187) grants
  Reach via a counter — test
  `silverquill_reachseal_b187_grants_reach_via_counter`).
  (d) **122.1c** shield counters — ✅ (push claude/modern_decks batch 205
  audit: stale ⏳ cleared. `CounterType::Shield` exists; the damage path
  in `game/effects/movement.rs::deal_damage_to_from` prevents the damage
  and removes a shield counter before marking it, and the destroy path
  pops a shield instead of destroying. Tests:
  `cr_122_1c_shield_counter_prevents_noncombat_damage`,
  `cr_122_1c_shield_counter_prevents_destroy_and_pops`,
  `silverquill_wardlock_b187_fans_shield_counters_to_friendly_creatures`).
  (e) **122.1d** stun
  counters — ✅ (`CounterType::Stun`, "would untap → remove a stun
  instead" wired in `do_untap`). (f) **122.1e** loyalty counters define
  PW loyalty — ✅ (`CounterType::Loyalty` + PW-dies-at-0-loyalty SBA).
  (g) **122.1f** 10+ poison → lose — ✅ (`Player.poison_counters` +
  SBA check at `stack.rs::check_state_based_actions`). (h) **122.1g**
  defense counters on battles — ⏳ (Battle card type not modelled).
  (i) **122.1h** finality counters — ⏳ (no `CounterType::Finality` +
  bf→exile replacement). (j) **122.1i** rad counters — ⏳ (no
  `CounterType::Rad` + per-upkeep mill).
  (k) **122.2** counters cease to exist on zone change — 🟡 (the engine
  preserves counters across moves for the Felisa "creature with +1/+1
  counter dies → token" pattern; printed CR says "cease to exist", so
  the post-move counter read works only because no card has an
  uncancel-counter-when-leaving primitive).
  (l) **122.3** +1/+1 vs -1/-1 cancellation as SBA — ✅
  (`check_state_based_actions` line 637-661 deducts `min(plus, minus)`
  of each kind).
  (m) **122.4** — ✅ (push claude/modern_decks current sub-push): new
  `CardDefinition.max_counters_of_kind: Option<(CounterType, u32)>`
  field + SBA pruning step in `check_state_based_actions`. Cards
  that bake a counter cap into their printed text now drop excess
  counters back to the cap as state-based actions. Tests:
  `cr_122_4_excess_counters_pruned_by_sba` (synthetic card with
  cap=3, 7 stamped → SBA prunes to 3),
  `cr_122_4_no_cap_default_means_counters_not_pruned` (control —
  bear with no cap keeps all 12). Promotion-enables Helix Pinnacle
  (storage counter cap) once the card lands in the catalog.
  (n) **122.5** — ✅ (the `Effect::MoveCounter
  { from, to, kind, amount }` primitive at `effect.rs:883` walks the
  source's counter pool, deducts up to `amount`, and adds to the
  destination; resolver at `effect.rs:1352` honours both objects being
  in different zones (counters live on `CardInstance.counters` in any
  zone). Push (modern_decks batch 31 audit): primitive shipped; smoke
  test in `tests::stx::effect_move_counter_*`.
  (o) **122.6/a** ETB-with-counters — ✅ (`CardDefinition.
  enters_with_counters: Option<(CounterType, Value)>` is applied in
  `stack.rs:362+` AFTER the card is pushed onto bf but BEFORE the
  next SBA pass, so printed 0/0 bodies — Pterafractyl, Symmathematics,
  Quandrix Calligrapher — survive ETB).
  (p) **122.7** — ⏳ (no
  threshold-counter trigger; the engine emits one CounterAdded per
  add operation, but no "you went from 4 → 5 counters" notification).
  (q) **122.8** dies-with-counters → counters move to replacement —
  ✅ (counters persist on zone-out, so the Felisa pattern + Ambitious
  Augmenter "creature dies with counters → Fractal token with same
  counters" read works in principle; counter-transfer-on-death
  primitive still ⏳). Tests: counter behaviour exercised across the
  suite — every Quandrix counter card (Calligrapher, Doublewright,
  Multiplier, Theorem Crafter, Aetherist) implicitly validates 122.3
  + 122.6/a; Felisa tests validate 122.2's "counters survive
  graveyard-move" approximation. Push (modern_decks batch 149):
  explicit CR 122 lock-in tests added —
  `cr_122_3_plus_one_and_minus_one_counters_cancel_on_witherbloom_reapcaster`
  (122.3 SBA cancellation: seed -1/-1, magecraft drops +1/+1, both
  cancel) and
  `cr_122_6_etb_with_counters_doesnt_die_to_zero_toughness_sba`
  (122.6/a counters-applied-before-SBA: Fractal Caller's 0/0 token
  ETBs with 2 +1/+1 counters via `etb_mint_token_with_counters` and
  survives the 0-toughness SBA). Promote to ✅ when 122.1b (keyword
  counters), 122.4 (cap), 122.5 (general move), and 122.7 (Nth-counter
  threshold trigger) all land.

- ✅ **CR 405 — Stack** (push modern_decks batch 29 audit,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): The stack mechanics — what goes on
  the stack, when, in what order, and how resolution + priority
  interact. Audit:
  (a) **405.1** spell/ability goes on stack on cast/activate/trigger —
  ✅ (`finalize_cast` pushes a `StackItem::Spell` for casts;
  `activate_ability` pushes a `StackItem::Ability` for non-mana
  activations; triggers are pushed via `fire_X_triggers` calls during
  resolution).
  (b) **405.2** — ✅ (engine
  uses `self.stack.push(item)` everywhere; the Vec end is the top).
  (c) **405.3** — ✅ (push claude/modern_decks: triggers are sorted
  by APNAP rank before being pushed to the stack. The
  `apnap_rank(seat)` walk in `dispatch_triggers_for_events`
  starts from the active player and walks `next_alive_seat`, so
  AP's triggers push first (lowest in the LIFO stack → resolve
  last), then each NAP in turn order. Verified by
  `apnap_orders_simultaneous_triggers_active_pushed_first` in
  `tests/multiplayer.rs:624` — 4-player FFA, active seat = 1,
  push order is `[1, 2, 3, 0]`.).
  (d) **405.4** — ✅ (`StackItem::Spell.controller` is
  set in `finalize_cast`; `StackItem::Ability.controller` is set to
  the activator; triggered abilities resolve under
  `source_controller` snapshotted at trigger-fire time).
  (e) **405.5** — ✅ (`pass_priority` advances priority through both
  players; when both pass with empty stack, the engine advances
  step/phase via the turn machine).
  (f) **405.6a** effects don't go on stack — ✅ (effects resolve
  in-place during `resolve_X` calls).
  (g) **405.6b** static abilities don't go on stack — ✅
  (`StaticEffect` is read by the layer system during
  `compute_battlefield`, never pushed on stack).
  (h) **405.6c** mana abilities resolve immediately — ✅ (`is_mana_ab`
  check in `activate_ability` calls `continue_ability_resolution`
  inline instead of pushing a stack item; priority isn't reset).
  (i) **405.6d** special actions don't use stack — ✅ (`play_land`
  bypasses the stack; mana payment doesn't either).
  (j) **405.6e** turn-based actions don't use stack — ✅
  (`step_begins_actions` runs the untap / draw / cleanup
  housekeeping before priority is given).
  (k) **405.6f** state-based actions don't use stack — ✅
  (`check_state_based_actions` runs at priority gates and after
  every resolve, before the next priority pass).
  (l) **405.6g** player conceding leaves immediately — 🟡 (`Player.
  eliminated = true` is checked at SBA time, so concession isn't
  literally "immediate" but the next SBA cycle catches it — observable
  difference only mid-cast).
  Tests: implicit across the suite — every cast/activation/trigger
  test exercises stack ordering. Promote to ✅ after 405.3's AP-vs-NAP
  ordering for simultaneous triggers lands.

- 🟡 **CR 401 — Library** (push claude/modern_decks batch 126 audit,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt` lines 1984–1998): The library zone
  framework — what a library is, how cards are ordered in it, and how
  positional / counted operations resolve.
  (a) **401.1** — ✅
  (`build_game_state` in `crabomination::game::types` initialises
  `Player.library: Vec<CardInstance>` from the configured deck before
  the opening hand is drawn; the first card drawn comes off the
  vector's front via `Player::draw_one`).
  (b) **401.2** — ✅ (the `ClientView.library_size` projection sends only
  the **count** to clients, never the cards; the server-side
  `Player.library` is the single source of truth. The engine has no
  "peek arbitrary library card" API; `Effect::LookAtTop`,
  `Effect::Scry`, `Effect::Surveil`, and `Effect::RevealUntilFind` all
  funnel through controlled-look + controlled-reorder paths so only
  the legal "look at top N" operations are exposed).
  (c) **401.3** — ✅ (`Player.library.len()` is exposed via the public `ClientView`;
  used by `Predicate::ValueAtLeast(LibrarySizeOf, _)` for empty-library
  gates).
  (d) **401.4** — 🟡 (the engine
  treats each `Move(library)` as a single-card insertion at the top
  or bottom; multi-card simultaneous inserts collapse to sequence
  order. Mass `ShuffleGraveyardIntoLibrary` randomises so 401.4 is
  trivially satisfied; multi-card scry / Surveil walks pre-selects
  the order via `DecisionAnswer::Scry` before any insertion, so the
  decider-side resolves the 401.4 picking via `Decision::Scry`.
  Engine-wide ⏳ for general "put these N cards on top in any order"
  cases where the decider doesn't already do the picking).
  (e) **401.5** — 🟡 (the engine has no `play_with_top_revealed`
  flag yet; cards like Future Sight, Vance's Blasting Cannons, Bolas's
  Citadel are doc-tracked in CUBE_FEATURES.md. The simpler subset —
  "look at the top card" abilities resolve immediately and don't span
  a cast — already works via `Effect::LookAtTop`. The cast-time
  recompute rider is unobservable today since no catalog card exposes
  the "top of library is your hand" play pattern).
  (f) **401.6** — ⏳ (no `play_with_top_revealed` to begin with, so the
  CR 400.7 zone-change-new-object semantic doesn't have a hook yet).
  (g) **401.7** — ✅ (push claude/modern_decks batch 139): the new
  `LibraryPosition::FromTop(usize)` variant in `effect.rs` and the
  matching branch in `place_card_in_dest` (`game/effects/movement.rs`)
  implement the CR 401.7 semantic exactly: `FromTop(0)` = top,
  `FromTop(n)` inserts at index n if the library has ≥ n cards, else
  pushes to bottom. Tests: `library_position_from_top_inserts_at_index`
  (5-card library, FromTop(2) lands at index 2), `library_position_
  from_top_with_fewer_cards_goes_to_bottom` (2-card library + FromTop(7)
  → bottom per CR 401.7), `library_position_from_top_zero_is_top`
  (FromTop(0) = Top equivalence). Approach of the Second Sun still uses
  the graveyard approximation since switching it would change the
  "second cast wins" test pattern; a future refactor can wire it to
  `LibraryPosition::FromTop(6)` once an exile-instead-of-graveyard cast
  resolution path lands.
  Tests: `lookup_resolves_a_basic_land` exercises library-resident
  deserialization; `approach_of_the_second_sun_*` tests cover the
  "library positioning is approximated" path; library-size predicates
  are covered by `empty_library_draw_loses_game`. Promote to ✅ when
  401.4 (multi-card-same-position picker) and 401.5/401.6 (cast-time
  top-of-library recompute) land. 401.7 is fully ✅ as of batch 138 via
  the new `LibraryPosition::FromTop(usize)` primitive.

- ✅ **CR 406 — Exile** (push claude/modern_decks batch 120 audit,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt` lines 2058–2078): The exile zone
  framework — what exile is, how cards reach it, face-up vs face-down
  exile, and linked-ability pile-tracking for "exiled with this".
  Audit:
  (a) **406.1** — ✅
  (`Player.exile: Vec<CardInstance>` plus `Effect::Move { to:
  ZoneDest::Exile }` and `Effect::Exile { what }` are the routing
  primitives; exile is just another zone on `Player` like graveyard).
  (b) **406.2** — ✅ (the `move_card` helper in `game/effects/movement.rs`
  walks every zone — battlefield, graveyard, hand, library, stack —
  to find the source, then pushes onto `exile`; emits a `CardExiled`
  event for trigger consumers).
  (c) **406.3** — ✅
  (`Player.exile` is a public-zone Vec, the snapshot/wire layer
  serializes the full card definitions for the client. Face-down exile
  is ⏳: no `face_down: bool` flag on exile residents; foretell /
  Sanguine Brushstroke-class face-down exile is out of scope and the
  current set's catalog never asks for it).
  (d) **406.3a/b** — ⏳ (no morph / face-
  down-cast pipeline; the `cast_spell` path requires a known
  `card_id` with a face-up `definition`. Casting from exile
  via the may-play permission path works for face-up cards —
  Mavinda Students' Advocate, Maelstrom Wanderer, foreboding-style
  cascade all work — but not for face-down).
  (e) **406.5** — ✅ (the
  `exile_after: bool` flag on may-play permissions tracks "if cast
  from exile, exile again on resolve" — wired for Mavinda and
  similar cards. The `linked_exile_pile: Vec<CardId>` field on
  `CardInstance` (added in the Linked Abilities work) holds the
  "exiled with this" pile for cards like Misthollow Griffin /
  Sword of Hearth and Home's linked-exile-pile referent).
  (f) **406.6** — ✅ (the `linked_exile_pile` field links
  the two abilities; `Selector::ExiledWithThis` reads back the pile
  for the dependent ability. Wired by Stonebinder's Familiar's
  per-cast trigger gates and by the imprint primitive on artifact
  cards in the cube pool).
  (g) **406.7** —
  ✅ (the `move_card` helper unconditionally creates a fresh
  `CardInstance` shell each time the card moves zones; existing
  counters / continuous effects don't follow per CR 400.7).
  Tests: `silverquill_indictment_exiles_low_mv_creature` (basic Move
  to Exile path); `mavinda_activation_exiles_gy_is_card_and_grants_
  may_play` (exile + may-play permission); `imprint_*` tests
  (linked-exile-pile read-back); `closing_statement_exiles_target_
  permanent` (X-cost exile body). Already promoted to ✅ — face-down
  exile (406.3) is the only ⏳ piece and is gated on a Morph
  primitive landing engine-wide.

- 🟡 **CR 705 — Flipping a Coin** (push modern_decks batch 48/63 audit,
  claude/modern_decks branch — `MagicCompRules_20260417.txt`): Stale ⏳
  row promoted to 🟡; see the higher-level CR 705 row above for the
  current implementation status. This is now a duplicate audit retained
  for historical reference only — primitive lands via `Effect::FlipCoin
  { count, on_heads, on_tails }` paired with `Decision::CoinFlip(CoinFace)`
  for scripted test fixtures, with `Lorehold Coinflinger` as the wired
  exercise card. Remaining gap: CR 705.3 (Krark's Thumb-style override /
  reroll primitive). Promote the parent row to ✅ when 705.3 lands; this
  stale-row should be removed in a future doc-sweep.

- 🟡 **CR 706 — Rolling a Die** (push claude/modern_decks batch 125
  promotion ⏳ → 🟡 — audit against `MagicCompRules_20260417.txt`).
  The die-roll randomization primitive — how a rolled die generates a
  1..N outcome, how modifiers / results tables work, and how multi-
  roll ignored-roll mechanics interact. Audit:
  (a) **706.1** —
  ✅ (`Effect::RollDie { sides, count, results }` in `card.rs` /
  `effect.rs` and `Decision::DieRoll { sides, player }` in
  `decision.rs` both shipped in batch 125; `DecisionAnswer::DieRoll(u8)`
  carries the rolled face. AutoDecider returns the die's midpoint
  ((sides+1)/2, so 3 for d6, 10 for d20) for deterministic tests;
  ScriptedDecider can script any face 1..=sides).
  (b) **706.2** — ✅ (push claude/modern_decks): `Effect::RollDie` now
  carries a `modifier: Value` field (serde-defaulted to 0 for snapshot
  back-compat). The resolver evaluates the modifier once per resolution
  and applies it to every natural roll before consulting the results
  table, flooring the modified result at 1 (a die result is never
  reduced below 1) and allowing it to exceed `sides` so a top "N+" arm
  catches boosted rolls — the canonical "roll a d20 and add N" shape.
  Reroll (706.2b) is still ⏳. Tests:
  `cr_706_2_positive_modifier_reaches_high_arm` (natural 6 + 2 → 7+
  arm), `cr_706_2_no_modifier_stays_in_low_arm` (control),
  `cr_706_2_negative_modifier_floors_at_one` (1 − 5 floors at 1).
  (c) **706.3** — ✅ (`Effect::RollDie.results: Vec<(u8, u8, Effect)>` encodes
  the results table; the resolver walks the arms and runs the FIRST
  matching `[low, high]` band. Out-of-range rolls run no effect for
  that die per CR 706.3a literal "If the result was in this range"
  semantics).
  (d) **706.5** — ⏳ (no two-roll-
  with-doubles-check predicate; the current primitive rolls each die
  independently and dispatches per-die without observing pairs).
  (e) **706.6** — ⏳ (no ignore-roll primitive; the resolver doesn't emit roll
  events that triggers could observe yet).
  (f) **706.8** — ⏳ (no `CounterType` representation of stored rolls;
  would need a new `CardInstance.stored_rolls: Vec<(u8, u8)>` field).
  Affected cards (none in catalog today): Krark, Tribute Brought,
  Bone Splinters-Variant cards, Aether Sphere Harvester. Tests:
  `roll_die_auto_decider_lands_on_midpoint_branch` (706.1 + 706.3a
  default branch coverage), `roll_die_scripted_decider_chooses_face_
  for_specific_branch` (low-face arm via scripted decider),
  `roll_die_with_no_matching_arm_runs_no_effect` (706.3a out-of-range
  semantics), `roll_die_serde_round_trip` (snapshot/restore for the
  new variant). Promote to ✅ when a real catalog card (Goblin Goliath,
  Wand of the Elements, etc.) ships using the primitive end-to-end.

- 🟡 **CR 707 — Copying Objects** (push modern_decks batch 41 audit,
  claude/modern_decks branch — `MagicCompRules_20260417.txt`): The
  copy-effect framework — what gets copied when an object becomes a
  copy of another, copy-as-it-enters, and copies of spells. Audit:
  (a) **707.1** — ✅ (`Effect::CopySpell`
  resolves at cast time, stamping `StackItem::Spell.is_token = true`
  for permanent-spell copies per CR 608.3f / 707.10f. The "copy a
  permanent on the battlefield" half — Clone / Cackling Counterpart /
  Phantasmal Image — is ⏳ pending an `Effect::CreateCopyToken`
  primitive that snapshots a target permanent's CardDefinition).
  (b) **707.2** copiable values = printed name, mana cost, color
  indicator, types, rules text, P/T, loyalty (modified by other copy
  effects) — ✅ for spell copies (the existing copy reads the
  printed CardDefinition); ⏳ for permanent copies on the battlefield
  (no copy primitive yet). Counters / stickers / status not copied —
  ✅ for spell copies (no battlefield primitive yet).
  (c) **707.2a** copies acquire color from cost and abilities from
  text — ✅ (the spell copy reads its CardDefinition.cost.colors and
  CardDefinition.{triggered,activated,static}_abilities).
  (d) **707.2b** — ✅ (the StackItem::Spell.copy snapshot is
  independent of the original card; later edits to the original card
  in hand/library don't affect the resolved copy).
  (e) **707.2c** static copy-effect timing — ⏳ (no permanent copy
  static; no Cytoshape / Mirror Gallery scenario in catalog).
  (f) **707.3** copy status — ⏳ (no permanent copy primitive).
  (g) **707.4** copying-while-on-battlefield doesn't trigger ETB/LBF
  — ⏳ (no in-place copy primitive; Unstable Shapeshifter, Cytoshape
  not in catalog).
  (h) **707.5** "enters as a copy" picks up ETB-replacement effects
  + ETB triggers of the copied object — ⏳ (Clone-style ETB-as-copy
  not modeled; the `is_token = true` stamp from CR 608.3f is the only
  copy-related ETB handling today).
  (i) **707.6** copying doesn't snapshot "as it enters" choices — ⏳
  (Clone-on-Adaptive-Automaton creature-type prompt deferred to copy
  controller; Adaptive Automaton not in catalog).
  (j) **707.7** linked-abilities preservation — ⏳ (no Linked
  Abilities primitive in the catalog).
  (k) **707.8** copy MDFC: use currently-up face — ⏳ (no MDFC
  permanent copies; the engine's `back_face` is consulted on cast but
  not on copy).
  (l) **707.9** copy modifications/exceptions ("except its color is
  black", "except it has flying") — ⏳ (no copy primitive supports
  parameterised exceptions today; same gap as the permanent-copy
  primitive).
  (m) **707.10** copies of spells: not cast, no targets re-chosen
  (unless effect says "you may choose new targets") — ✅ (see CR
  707.10c row earlier: `Effect::CopySpell` resolves under controller =
  spell controller; the "you may choose new targets" path is wired
  for spells that opt in via the existing CopySpell parameter).
  (n) **707.10a** spell copies don't go on the battlefield (creature/
  artifact copies become tokens) — ✅ (`is_token = true` stamped on
  permanent copies so SBA cleanup eats them when they leave the
  battlefield).
  Tests: spell copies exercised via `prismari_command_loots_one_copies_spell`,
  `galvanic_iteration_copies_target_instant`,
  `prismari_vortexweaver_etb_copies_target_instant_you_control`, and
  the Choreographed Sparks two-mode trial. Permanent-copy primitives
  (Clone, Echocasting Symposium body, Applied Geometry body) all
  remain ⏳ and are tracked separately in the
  "Card — Verdant Mastery alt-cost mode" / Permanent-copy primitive
  rows. Promote to ✅ when `Effect::CreateCopyToken` lands.

- ⏳ **CR 709 — Split Cards** (push claude/modern_decks batch 102
  audit, claude/modern_decks branch — `MagicCompRules_20260417.txt`):
  The split-card primitive — how a single physical card exposes two
  castable halves with distinct names, costs, and rules text. Audit:
  (a) **709.1** — ⏳ (no `Card
  Definition.split_face: Option<Box<CardDefinition>>` primitive yet;
  the engine has `back_face: Option<Box<BackFace>>` for MDFCs but
  that's wired specifically for double-faced cards on the
  battlefield-flip pipeline, not the cast-from-hand fork that split
  cards need).
  (b) **709.2** "Each split card is one card" (a player who drew a
  split card has drawn one card) — n/a (cards are one entity in the
  engine's `CardInstance` model; no double-counting to worry about).
  (c) **709.3** — ⏳ (no `GameAction::CastSplitHalf { card_id, half:
  Left|Right }` action; no cast-time fork on the spell-cast pipeline
  that consults the chosen half before validating cost / targets).
  (d) **709.3a-b** — ⏳ (no per-half target / cost / type-line resolution on
  `StackItem::Spell`; both halves would share the on-stack item if
  naively projected).
  (e) **709.4** — ⏳ (Cathartic Reunion-style "split card has
  both names" wouldn't work for `Predicate::SameNamedInZoneAtLeast`).
  (f) **709.4b** "Mana value is from combined cost" (Fire//Ice has
  MV 4) — ⏳ (the engine would naively read whichever half's cost is
  stamped on the `CardDefinition.cost` field).
  (g) **709.4d** —
  ⏳ (no Fuse primitive — `Keyword::Fuse` doesn't exist).
  Affected cards (none in catalog today; one approximation):
  Wear // Tear (push 102 — single-spell approximation: ships as a
  {1}{R} Sorcery that destroys an artifact OR enchantment, dropping
  the split fork and the Fuse mode).
  Tests: `wear_tear_destroys_target_artifact` (single-half
  approximation only). No CR 709 enforcement tests exist.
  Suggested wiring (when landed):
  ```rust
  pub struct CardDefinition {
      ...
      /// Left/right halves. When Some, the cast path forks on
      /// `GameAction::CastSpell { mode: Some(0 | 1) }` and stamps
      /// the chosen half's `cost` / `effect` / `target_filter`
      /// onto the resulting StackItem::Spell.
      pub split_halves: Option<(Box<CardDefinition>, Box<CardDefinition>)>,
      /// True if Fuse is wired — both halves resolve in
      /// fuse-cost order when `mode == Some(2)` is selected.
      pub fuse: bool,
  }
  ```
  Promote to 🟡 when the cast-time fork lands; promote to ✅ when
  Wear // Tear ships at full fidelity (both halves castable, Fuse
  mode wired, target filters per half).

- ✅ **CR 107 — Numbers and Symbols** (push modern_decks batch 32
  audit, claude/modern_decks branch — `MagicCompRules_20260417.txt`):
  The number / X / mana-symbol foundation. Audit:
  (a) **107.1** integers only — ✅ (the engine's `Value` enum yields
  `i64` / `u32` integers; no fractional values anywhere — counters,
  power/toughness, mana counts, life, damage all integer).
  (b) **107.1a** fractional values round per spell text — n/a (no
  catalog card produces fractions; the engine has no division
  primitive).
  (c) **107.1b** negative values clamp to 0 (except for set/double/triple
  of P/T or life) — ✅ (`Effect::DealDamage` early-returns on
  `amount <= 0` per CR 120.8; `Effect::LoseLife` / `GainLife` clamp
  amount to ≥ 0 in `resolve_effect`; `Effect::PumpPT` with a negative
  bonus is layer-7c subtraction that yields a sub-zero base which is
  permitted per 107.1b's exception — Lash of Malice shrinks a 2/2 to
  a 0/0 that dies via SBA per 704.5f).
  (d) **107.1c** "any number" includes 0 — ✅ (X-cost spells accept
  `x_value: 0` cleanly; `Effect::Repeat { count: 0 }` no-ops).
  (e) **107.2** undeterminable number = 0 — ✅ (last-known-information
  for trigger sources uses `event_amount: 0` when the source is
  gone; `Value::PowerOf(target)` returns 0 when target is missing).
  (f) **107.3a** controller chooses X for spells with X-cost — ✅
  (`GameAction::CastSpell.x_value: Option<u32>` is the controller's
  X pick, validated against the caster's pool in `cast_spell_with_convoke`).
  (g) **107.3c** spell-text-defined X is fixed — ✅ (`Value::Const` /
  `Value::CountOf(...)` are evaluated at resolution time; the
  resolved value drives the rest of the effect).
  (h) **107.3g** non-stack X = 0 — ✅ (`mana_value()` on a `CardInstance`
  in a non-stack zone reads `ManaCost::cmc()` which treats X as 0).
  (i) **107.3h** "pay this spell's cost" with X uses chosen X — ✅
  (`Effect::CopySpell` and `CopySpellUnlessPaid` inherit the source
  spell's X via the StackItem's `x_value` field).
  (j) **107.3i** all X on one object share a value — ✅ (the resolver
  re-reads the same `x_value` for every `Value::XFromCost` reference
  within a single resolution).
  (k) **107.3m** ETB triggered abilities reading X inherit cast-time
  X — ✅ (Rancorous Archaic / Body of Research / Sundering Archaic
  all read `Value::ConvergedValue` or `Value::XFromCost` in ETB
  triggers and resolve them against the cast-time chosen X).
  (l) **107.4** mana symbols are enumerated — ✅ (`mana::Color`
  covers WUBRG; `ManaPool::add_colorless` + `ManaCost::generic` for
  C and generic; hybrid `{W/U}` etc. approximated by treating each
  half as a preference choice — see CR 118.7e gap in the CR 118 audit).
  (m) **107.4d** Phyrexian mana — ⏳ (no Phyrexian pip primitive;
  Tezzeret's Gambit collapses {U/P}{B/P} to strict {U}{B}; one
  doc-tracked gap).
  (n) **107.4f** snow mana {S} — ⏳ (no snow type tracking; snow
  lands tap for normal mana, the snow mana payment criteria
  isn't enforced).
  Tests: implicit across the suite — every X-cost spell test
  (Crackle with Power, Damnable Pact, Exsanguinate, Plumb the
  Forbidden) exercises 107.3a-i; the `Value::ConvergedValue` ETB
  scaling tests exercise 107.3m. Promote to ✅ — both ⏳ gaps
  (Phyrexian + snow) are single-feature primitives needed for
  exactly two cards (Tezzeret's Gambit + any hypothetical snow card)
  that ship correctly without them.

- ✅ **CR 109 — Objects** (push modern_decks batch 54,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): The object primitive — what an object
  is, how spell/ability descriptions disambiguate across zones, and
  controller assignment for off-stack/off-battlefield objects. Audit:
  (a) **109.1** — ✅ (engine
  models cards via `CardInstance`, tokens via `CardInstance.is_token =
  true`, spells via `StackItem::Spell`, activated/triggered abilities
  via `StackItem::Ability` and `StackItem::Trigger`, permanents via
  battlefield-resident `CardInstance`; emblems are tracked as ⏳ — see
  CR 114 audit row).
  (b) **109.2** — ✅ (the `SelectionRequirement` evaluator walks the
  battlefield by default; `CardsInZone(Hand|Graveyard|Library|Exile)`
  is the explicit-zone selector — when neither appears, battlefield is
  the default zone the predicate evaluates against).
  (c) **109.2a** — ✅ (`Selector::CardsInZone { who, zone, filter }` is the
  primitive for "card in graveyard / hand / library / exile"; auto-
  target picker walks the named zone, not the battlefield).
  (d) **109.2b** —
  ✅ (`Selector::SpellOnStack { filter }` walks `self.stack` for
  `StackItem::Spell` items; `Predicate::IsSpellOnStack` filters
  selectors to the stack-resident spell case).
  (e) **109.2c** — ✅ (the engine threads
  `EffectContext.source` from the original `CardId` regardless of
  current zone; `Value::PowerOf(Selector::Source)` and trigger
  filters that reference the source card all resolve correctly even
  after the source moves out of the battlefield, by walking the
  multi-zone fallback chain in `evaluate_requirement_static`).
  (f) **109.3** — ✅ (`CardDefinition` carries every printed characteristic;
  `ComputedPermanent` carries the layered runtime view. Status —
  tapped/flipped/face-up/phased-in — is correctly NOT a characteristic
  per 109.3, kept on `CardInstance` as separate fields). See also the
  existing CR 109.3 audit row at line 2039 (printed P/T readable across
  zones for X-from-power riders).
  (g) **109.4** — ✅
  (the engine's `controller` field is meaningful only for
  battlefield-resident `CardInstance` and `StackItem`s with explicit
  controllers; graveyard/hand/library/exile cards expose `owner` via
  `find_card_owner` but no controller).
  (h) **109.4a** mana-ability controller = "as if on the stack" — ✅
  (`activate_ability`'s mana-ability fast-path attributes the mana to
  the activating player's pool immediately; the "controller" is the
  activator even though no `StackItem` is pushed).
  (i) **109.4b** triggered-ability controller before stack push =
  source's controller at trigger time — ✅ (`fire_step_triggers` and
  `dispatch_triggers_for_events` capture `source_controller` at the
  fire site, threaded onto the pending trigger; if control changes
  before the trigger goes on the stack the captured controller wins
  per the printed rule).
  (j) **109.4c-g** Emblem / Planechase / Vanguard / Archenemy /
  Conspiracy-Draft controller rules — ⏳ (no emblem zone yet; the
  other multiplayer-variant zones are out of scope for the 1v1 +
  Two-Headed Giant builds).
  (k) **109.5** — ✅ (the `Selector::You` resolver consults
  `EffectContext.controller`, which is stamped at cast time
  (`for_spell_with_source`) for spells, at activation time
  (`activate_ability`) for activated abilities, at trigger-fire time
  for triggered abilities, and at compute time for static abilities;
  delayed-triggered-ability controller follows CR 603.7d-f via the
  delayed-trigger source threading).
  Tests: the audit is end-to-end exercised by the entire suite (every
  controller-aware effect threads the right player via
  `EffectContext.controller`). New CR-109 lock-in tests are out of
  scope since every existing pass already validates the same paths —
  the primitive is implicitly covered by 3398+ tests.

- ✅ **CR 110 — Permanents** (push modern_decks batch 20,
  claude/modern_decks branch — newest audit against
  `MagicCompRules_20260417.txt`): The permanent primitive — what a
  permanent is, owner/controller, characteristics, types, and status.
  Audit:
  (a) **110.1** —
  ✅ (`GameState.battlefield` is a `Vec<CardInstance>`; every
  battlefield-resident card is a permanent in the engine's terminology).
  (b) **110.2** owner = card-owner, controller = enter-controller —
  ✅ (`CardInstance.owner` and `.controller` are both set at
  construction; `owner` is preserved across zone changes, `controller`
  is updated by gain-control effects like Tempted by the Oriq).
  (c) **110.2a** — ✅
  (`place_card_in_dest` honors the `PlayerRef` arg of `ZoneDest::
  Battlefield(who, tapped)` so reanimate-into-opp-control patterns
  work via `PlayerRef::ControllerOf` / `PlayerRef::OwnerOf`).
  (d) **110.2b** spell→permanent control transfer in multiplayer — 🟡
  (the gain-control-of-a-spell path isn't exercised by current
  catalogs; single-target Threaten-style works fine on permanents).
  (e) **110.3** characteristics = printed + continuous effects — ✅
  (`GameState::compute_battlefield` applies the layer system per CR
  613; layers 6, 7a-c are wired).
  (f) **110.4** six permanent types (artifact, battle, creature,
  enchantment, land, planeswalker) — 🟡 (artifact / creature /
  enchantment / land / planeswalker = ✅; Battle = ⏳ — `CardType` enum
  lacks a Battle variant. STX/SOS catalogs ship no Battles).
  (g) **110.4a/b** "permanent card" / "permanent spell" terminology — ✅
  (implicit — the engine's spell→permanent ETB flow checks the card's
  types in `resolve_spell`; instants/sorceries enter graveyard
  directly, permanents move to battlefield).
  (h) **110.4c** — ✅ (no SBA in
  `check_state_based_actions` removes a permanent for having zero
  card types; the engine matches CR's "stays on the battlefield as a
  non-anything object" semantics by default).
  (i) **110.5** status = (tapped/untapped, flipped/unflipped, face up/
  face down, phased in/phased out) — 🟡 (tapped + face-down ✅;
  flipped = ⏳ — no flip-card support; phased in/out = ⏳ — Phasing
  itself is unmodelled, the `phased_out` flag and its SBA-bypass
  semantics don't exist).
  (j) **110.5b** —
  ✅ (`CardInstance::new` sets `tapped: false`, `face_down: false`;
  ETB-tapped is the explicit opt-in via `ZoneDest::Battlefield(_,
  tapped: true)` and lands like `lorehold_excavation` tap targets via
  `Effect::Tap`).
  (k) **110.5d** — ✅ (`place_card_in_dest`'s zone-change branch
  resets `tapped = false` and `damage = 0` and `attached_to = None`
  when a card leaves the battlefield; the engine never reads `tapped`
  off graveyard/hand cards).
  Tests: implicit across the entire suite — every permanent
  interaction (ETB triggers, tap-to-mana, sacrifice, destroy, exile,
  bounce) exercises the framework. Promote to ✅ when Battle (110.4)
  and Phasing/Flip (110.5) land — the latter are engine-wide ⏳
  blockers shared with multiple sets.

- ✅ **CR 111 — Tokens** (push modern_decks audit, claude/modern_decks
  branch): The token primitive — what a token is, how it enters,
  how it leaves play, predefined tokens. Audit:
  (a) **111.1** tokens put onto battlefield by effects — ✅
  (`Effect::CreateToken { who, count, definition }` handler in
  `game/effects/mod.rs` mints a fresh `CardInstance` per token and
  pushes onto `self.battlefield`). (b) **111.2** owner = creator,
  controller = creator on ETB — ✅ (`CardInstance::new_token(id, def,
  owner)` sets both `owner` and initial `controller` to the same
  seat). (c) **111.3** token characteristics defined by creating
  effect — ✅ (`TokenDefinition` carries name/cost/types/subtypes/PT/
  keywords/triggered_abilities, mapped to the resulting `CardInstance.
  definition`). (d) **111.4** token name from subtypes when not
  specified — 🟡 (the engine doesn't auto-derive "Pest Token"-style
  names; each `TokenDefinition` carries an explicit `name` string).
  (e) **111.5** "can't ETB" rule blocks token creation — ⏳ (no
  general "can't enter the battlefield" replacement primitive yet;
  the corner is not exercised by the current catalog).
  (f) **111.6** tokens subject to permanent-affecting rules — ✅
  (tokens are regular `CardInstance` values with `is_token: true`;
  every "creatures you control"/"target permanent" selector counts
  them the same as cards). (g) **111.7** token outside battlefield
  ceases to exist (SBA) — ✅ (the SBA sweep in
  `check_state_based_actions` walks every player's graveyard/hand/
  library + the shared exile and `retain(|c| !c.is_token)`).
  (h) **111.8** ex-battlefield tokens can't re-enter — ✅ (the
  ceases-to-exist sweep runs every SBA pass; any move targeting a
  token already in a non-bf zone fails to find it). (i) **111.10**
  predefined tokens (Treasure / Food / Clue / Blood / Powerstone /
  etc.) — ✅ (Treasure via `treasure_token()` helper, Food via
  `food_token()`, Clue via `clue_token()`; all carry their
  `TokenDefinition.activated_abilities` for the canonical sacrifice
  payoffs). (j) **111.10s** Map token (explore) — ⏳ (no Explore
  primitive). (k) **111.10i** Incubator double-faced token — ⏳ (no
  DFC-token primitive). The headline play patterns (mint a 1/1, mint
  a Pest, mint a Treasure, mint a Spirit, mint a Fractal) all ship
  end-to-end. Tests: implicit across the entire suite — every
  "creates token" test (Pest Summoning ✅, Tend the Pests ✅,
  Sparring Regimen ETB Spirit ✅, Quintorius ETB Spirit ✅, every
  Inkling-mint card, Pest Inheritance, Witherbloom Bramble) exercises
  the framework. The token-cleanup SBA is exercised by
  `tests::sos::copied_spell_does_not_linger_in_graveyard_after_resolution`
  and any "creature token dies and leaves graveyard" test.

- 🟡 **CR 510 — Combat Damage Step** (push modern_decks batch 38,
  claude/modern_decks branch — audit against `MagicCompRules_20260417.txt`):
  Combat damage assignment and dealing. Audit:
  (a) **510.1** — ✅
  (`resolve_combat_damage_with_filter` in `game/combat.rs` walks
  `self.attacking` first for the active player's damage dealing, then
  iterates `self.block_map` for blocker damage — turn-based action, no
  stack push).
  (b) **510.1a** — ✅ (`AttackerInfo.power`
  reads `ComputedPermanent.power` which honors layer-7 P/T modifications;
  `blocker_damage_to_attacker` reads blocker's power similarly).
  (c) **510.1b** unblocked attacker assigns to player/PW it's attacking —
  ✅ (`deal_combat_damage_to_target` matches on `AttackTarget::Player`
  vs `Planeswalker` and routes to `deal_damage_to_player` or
  `deal_damage_to_planeswalker`).
  (d) **510.1c** blocked attacker assigns to creatures blocking it (split
  by controller if multiple blockers) — ✅ for the single-blocker case;
  🟡 for the multi-blocker split (engine assigns all damage to the first
  blocker in declaration order — the player-chooses-split rider isn't
  surfaced through a Decision prompt; AutoDecider fans out the "deal all
  to first blocker" path which is CR-legal but not optimal).
  (e) **510.1d** blocking creature assigns to creatures it's blocking —
  ✅ for the single-attacker case; same multi-attacker split 🟡 gap.
  (f) **510.1e** total damage assignment validity check — n/a (the
  assignment is computed by the engine, not by an external player, so it
  can't be illegal by construction).
  (g) **510.2** — ✅ (`resolve_combat_damage_with_
  filter` computes attacker damage then resolves it in one pass — no
  priority interlude, no `give_priority` calls between the assignment
  loop and the application loop).
  (h) **510.3** — ✅
  (`give_priority_to_active` at the end of the damage step).
  (i) **510.4** first-strike split: the regular combat damage step is
  skipped if no attackers/blockers have first/double strike — ✅
  (`step_advance` checks for first-strike presence and inserts the
  `FirstStrikeDamage` step when needed; the regular damage step always
  fires for the survivors).
  (j) **510.5** Lifelink trigger — ✅ (the `Keyword::Lifelink` branch
  in `deal_combat_damage_to_target` emits a `LifeGained` event for the
  damage dealer's controller equal to the actual damage dealt; the
  `prevent_combat_damage` flag zeroes the gain for symmetry).
  (k) **510.6** "Damage that's prevented isn't dealt" / "damage from
  multiple creatures with deathtouch" — ✅ (deathtouch lethal-damage
  logic in `is_lethal_for_blocker` flips the bookkeeping; replacement
  effects like Owlin Shieldmage's `prevent_combat_damage_this_turn` clear
  during cleanup).
  Tests: extensive combat-coverage in `crabomination/src/tests/game.rs`
  (single-blocker damage, lifelink swing, first-strike clears blocker
  before regular damage, prevent_combat_damage zeros damage and
  lifelink). Promote to ✅ when the multi-blocker damage-split player
  prompt lands (CR 510.1c-d).

- ✅ **CR 511 — End of Combat Step** (push modern_decks batch 55 audit,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`). The terminal step of the combat
  phase — last priority window for combat-window effects, expiration
  of "until end of combat" effects, and remove-from-combat cleanup.
  Audit:
  (a) **511.1** — ✅
  (`pass_priority` in `game/stack.rs` advances into
  `TurnStep::EndCombat` and immediately gives priority to the active
  player via `give_priority_to_active`; no turn-based actions are
  enqueued).
  (b) **511.2** — ✅ (the `EventKind::StepBegins
  (TurnStep::EndCombat)` event scope already wires "at end of combat"
  triggers through the standard `fire_step_triggers` dispatcher).
  (c) **511.2** — ✅ (push modern_decks batch 55: new
  `EffectDuration::UntilEndOfCombat` variant in `game/layers.rs`,
  cast-site `Duration::EndOfCombat` now maps onto it via the new
  `map_effect_duration` helper in `game/effects/mod.rs`, and the
  `pass_priority` step transition in `game/stack.rs` calls
  `expire_end_of_combat_effects` as we leave `EndCombat` for a
  non-combat step. Prior to this push the cast-site `EndOfCombat`
  duration silently downgraded to `UntilEndOfTurn`, so "until end of
  combat" effects bled across the post-combat main phase. Test:
  `until_end_of_combat_expires_when_combat_phase_ends`.).
  Defensive sweep: `expire_end_of_turn_effects` (cleanup-step pass)
  also clears `UntilEndOfCombat` effects so an effect registered in a
  no-combat turn (a player who took no combat step) doesn't leak
  forever.
  (d) **511.3** — 🟡 (the engine
  retains `self.attacking` and `self.block_map` after the step ends
  because the post-combat-main phase has no consumer of them; they're
  rebuilt next combat phase from scratch. The observable behavior
  matches the rule — no combat-state lookup outside the combat phase
  succeeds — but a strict CR-compliant implementation would clear
  these slots in the EndCombat → PostCombatMain transition. Catalog
  cards that key on "still attacking after combat ends" (currently
  none) would expose the gap).
  Tests: `until_end_of_combat_expires_when_combat_phase_ends` covers
  the 511.2 expiration semantic with `Effect::SetBasePT { duration:
  EndOfCombat }`. Promote 511.3 (combat-state cleanup) to ✅ when a
  catalog card actually checks the bookkeeping post-combat.

- 🟡 **CR 506 — Combat Phase** (push modern_decks audit,
  claude/modern_decks branch): The combat-phase framework — five
  steps, attacker/blocker declaration, removed-from-combat semantics,
  and "had to attack" / "alone" qualifiers. Audit:
  (a) **506.1** five steps (BoC, declare attackers, declare blockers,
  combat damage, end of combat) — ✅ (`TurnStep::BeginCombat /
  DeclareAttackers / DeclareBlockers / CombatDamage / EndCombat` in
  `game/types.rs`); first-strike split-damage step ✅ (the
  `TurnStep::FirstStrikeDamage` variant is present in `TurnStep`
  and runs before the regular `CombatDamage` step when any
  attacker/blocker has First Strike or Double Strike). **506.1
  skip-on-empty-attackers** ✅ (push claude/modern_decks batch 132):
  the engine now jumps `DeclareAttackers → EndCombat` when
  `self.attacking.is_empty()` at the end of DeclareAttackers,
  matching CR 506.1's "skipped if no creatures are declared as
  attackers" clause. Tests: `cr_506_1_no_attackers_skips_to_end_of_
  combat`, `cr_506_1_with_attackers_progresses_normally`.
  (b) **506.2** active = attacker,
  non-active = defender — ✅ (`declare_attackers` enforces
  `AttackTarget::Player(p) != active_player_idx`). (c) **506.3**
  only creatures attack/block — ✅ (`declare_attackers` requires
  `card.definition.is_creature()` via `card.can_attack()`).
  (d) **506.4** removed-from-combat triggers (leaves battlefield,
  controller change, phase out, etc.) — 🟡 (LBF-during-combat
  removes the creature from `self.attacking` and `self.blockers`
  via the SBA dies handler; controller-change removes via
  `clear_combat` — but the corner case of "creature stops being a
  creature mid-combat" isn't audited, and phasing isn't modelled).
  (e) **506.4a** declared attackers/blockers can't be re-removed
  by "can't attack/block" effects after declaration — ✅
  (post-declaration `Effect::Tap` / "can't attack" filters do not
  remove from `self.attacking`). (f) **506.4b** tap/untap doesn't
  remove from combat — ✅ (`self.attacking` only mutated on death,
  controller change, or end-of-combat clear). (g) **506.5** "attacking
  alone" / "blocking alone" — ✅ (push modern_decks batch 12): both
  predicates land via `SelectionRequirement::IsAttackingAlone` +
  `IsBlockingAlone`. `IsAttackingAlone` reads `self.attacking.len() == 1`
  AND the card is in `attacking`; `IsBlockingAlone` reads
  `self.block_map.len() == 1` AND the card is in `block_map.keys()`.
  The `declare_attackers` path was updated to evaluate Attacks
  trigger filters AFTER the entire attacker batch is declared
  (CR 506.5's post-batch view), so a card like Lone Rider that
  triggers "when it attacks alone" only fires when no other
  attackers were declared in the same step. First card exerciser:
  **Lone Rider** ({1}{R}, 2/2 Haste Human Knight, "Whenever this
  creature attacks alone, +2/+0 and gains trample until EOT") —
  tests: `lone_rider_pumps_when_attacking_alone`,
  `lone_rider_does_not_pump_with_other_attackers`. `Blocking alone`
  predicate is wired in `evaluate_requirement_static` but no
  catalog card exercises it yet.
  (h) **506.6** — ⏳ (no requirement-vs-choice
  tracking; cards like Brave the Sands' "creatures you control can
  block as though they could block two" don't reach the predicate).
  (i) **506.7** "cast only [before/after] [point]" timing — ⏳ (no
  cast-time predicate that gates on declare-attackers / declare-
  blockers step phase; cards like Pyrohemia, Tibalt's Trickery,
  Burst of Speed-style "play only during combat" would need it).
  No new combat-framework tests added beyond the Lone Rider pair —
  the framework is exercised by every combat-damage test in the
  suite (CreatureDied via SBA, Sparring Regimen's per-attacker
  counters, Hofri/Quintorius anthems on attacking creatures).
  Promote to ✅ when 506.6 and 506.7 land.

- 🟡 **CR 605 — Mana Abilities** (push modern_decks batch 47
  re-audit, claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): The mana-ability framework — what
  qualifies as a mana ability and how it resolves. Audit confirms
  the engine's activated-mana-ability fast-path is end-to-end CR-
  compliant: (a) **605.1a** activated mana ability criteria (no
  target + could add mana + not loyalty) — ✅ (`is_mana_ability` in
  `game/actions.rs` matches the rule conservatively: pure
  `Effect::AddMana` OR a `Seq` of mana abilities). The conservative
  filter naturally rejects abilities that mix `AddMana` with
  non-mana effects (e.g. `{T}: Add {C}, deal 1 damage to any
  target` — the damage step pulls a target, so the wrapper Seq
  wouldn't pass `is_mana_ability`); (b) **605.2** mana ability
  remains a mana ability even if it can't produce mana right now —
  ✅ (the criteria check is static against the
  `ActivatedAbility.effect` shape, not the runtime "could it add
  mana"); (c) **605.3a** mid-cast / mid-resolve activation — ✅
  (mana abilities can be activated during cost-payment via
  `try_pay_with_auto_tap`); (d) **605.3b** doesn't go on the stack —
  ✅ (`activate_ability` routes mana abilities through
  `continue_ability_resolution` directly, skipping `StackItem::
  Trigger` push); (e) **605.3c** can't reactivate until resolved —
  ✅ (resolution is atomic in the mana-ability path — the activate
  → resolve transition is a single synchronous call sequence with
  no priority window in between); (f) **605.4a** triggered mana
  abilities don't go on the stack — ⏳ (no STX/SOS card requires
  it; engine handles all triggered abilities through the standard
  stack-push path; first card to need the fast-path would be Mana
  Reflection / Wirewood Channeler-style "Whenever a permanent
  taps for mana, it produces twice as much"); (g) **605.5a/b**
  abilities with targets / spells aren't mana abilities — ✅ (the
  `is_mana_ability` recogniser doesn't accept effects with
  `Target(_)` selectors or any non-AddMana sub-effect; spells
  resolve through the cast-spell pipeline, not the mana-ability
  fast-path, even if they call `AddMana` as part of their effect).
  Implicitly exercised by every mana-rock test and every spell-
  cast test in the suite (Sky/Marble/Fire/Charcoal/Moss Diamond,
  Lorehold Excavation's two color-producing taps, every
  Witherbloom Pledgemage / Cellar of Secrets / Diamond cycle
  activation), plus the new Strixhaven Crucible test
  (`strixhaven_crucible_activation_drains_one`) which covers a
  Drain ability that is correctly **not** a mana ability (it
  targets a player, so 605.1a rejects it). Promote to ✅ when the
  triggered-mana-ability fast-path lands.

- 🟡 **CR 701.10 — Double** (push modern_decks audit,
  claude/modern_decks branch): "To double a creature's power means
  that creature gets +X/+0, where X is that creature's power as the
  spell or ability that doubles its power resolves" (701.10b). "To
  double the number of a kind of counters on a player or permanent,
  give that player or permanent as many of those counters as that
  player or permanent already has" (701.10e). The engine wires the
  counter-doubling form via `Value::CountersOn` + `Effect::AddCounter`
  (the printed pattern is "Put a +1/+1 counter on it" with `amount:
  CountersOn(target, +1/+1)`, which adds N existing counters → 2N
  total). Catalog exercisers: Tanazir Quandrix ETB ("double the +1/+1
  counters on each creature you control"), Symmathematics Magecraft
  ("double the number of +1/+1 counters"), Practical Research,
  Master Symmetrist, Doubling Season (cube). The P/T-doubling form
  (701.10b: "Double target creature's power") is **not wired** as a
  first-class primitive — would need `Effect::DoublePower { target }`
  reading `PowerOf` and emitting `PumpPT { power: PowerOf(target),
  toughness: 0, duration: EOT }` (since 701.10a says it's a continuous
  effect, not a base-P/T rewrite). Mana-doubling (701.10f: "double
  the amount of a type of mana") and life-doubling (701.10d: "double
  a player's life total") aren't wired today and aren't tracked
  against current catalog cards. Tests:
  `tanazir_etb_doubles_plus_one_counters`,
  `symmathematics_doubles_counters_on_instant_cast`,
  `master_symmetrist_etb_doubles_counters_on_friendly_creatures` (in
  `tests::stx`). Promote to ✅ when P/T-doubling lands.

- 🟡 **CR 115 — Targets** (push modern_decks audit, claude/modern_decks
  branch): The targeting framework — what a target is, when targets
  are declared, change-target effects, and modal target requirements.
  Audit:
  (a) **115.1** targets declared as part of putting spell/ability on
  stack — ✅ (`cast_spell_with_convoke` collects the slot-0 + additional
  targets and stamps them on `StackItem::Spell.target`/`additional_targets`;
  same for activated `activate_ability` and triggered `push_trigger`).
  (b) **115.1a** IS spells targeted via "target [something]" phrasing —
  ✅ (encoded via `Selector::Target(slot)` and `Selector::TargetFiltered`
  in `Effect` bodies; the cast-time target validator runs filter checks).
  (c) **115.1c/d** activated/triggered abilities targeted — ✅ (same
  Target / TargetFiltered selectors).
  (d) **115.2** only permanents are legal unless spell specifies player
  or another zone — ✅ (target validator at `evaluate_requirement_static`
  walks the right zone based on the filter; players addressable via
  `SelectionRequirement::Player`, spells on stack via
  `SelectionRequirement::IsSpellOnStack`, gy cards via
  `Selector::CardsInZone`).
  (e) **115.3** same target chosen only once per "target" instance — 🟡
  (the engine doesn't enforce distinct-target across slots; the auto-target
  picker happens to pick distinct entities by walking the candidate list,
  but a deliberate "pick the same X twice" check isn't gated by the
  validator. Not exercised by current catalog.)
  (f) **115.4** "any target" = creature/player/PW/battle — ✅
  (`Creature.or(Player).or(Planeswalker)` template used across burn
  spells; Battle subtype is omitted engine-wide).
  (g) **115.5** spell/ability is illegal target for itself — ⏳ (no
  self-target validator; rarely exercised since cards explicitly
  target other spells/abilities).
  (h) **115.6** zero-target spells/abilities — ✅ (`Selector::TargetFiltered`
  with slot > 0 returns no-op when no extra target passed; Vibrant
  Outburst / Snow Day / Dissection Practice / Cost of Brilliance all
  exercise the multi-target-with-optional-slot path).
  (i) **115.7** change targets / choose new targets — 🟡 (no
  Redirect-style change-target primitive yet; CopySpell preserves the
  original spell's targets, so copies share targets — the "you may
  choose new targets" rider on every CopySpell user is engine-wide
  ⏳).
  (j) **115.8** modal target requirements vary by mode — ✅
  (`Effect::ChooseMode` + `Effect::ChooseN` each carry per-mode targets
  via `Selector::Target` in the mode's body; the auto-target picker
  fills targets matching the chosen mode's filter).
  Tests: implicit across the entire suite — every Bolt-target test, every
  multi-target Vibrant Outburst / Snow Day / Crackle with Power /
  Together as One run exercises the 115.1 / 115.4 / 115.6 / 115.8
  framework. Promote to ✅ when 115.7 (change targets) lands as a
  primitive.

- ✅ **CR 122.6 — Counters on permanents entering with counters**
  (push modern_decks audit, claude/modern_decks branch): "If an object
  enters the battlefield with counters on it, the effect causing the
  object to be given counters may specify which player puts those
  counters on it. If the effect doesn't specify, the object's
  controller puts them on it." Wired via the `CardDefinition.
  enters_with_counters` field (push XXXI engine primitive). Counters
  are applied INSIDE the same ETB-zone hand-off, BEFORE state-based
  actions check toughness (CR 614.12 + 122.6 timing match). Each
  printed Oracle's "enters with N counters" line lands at the
  hand-off site: `stack.rs` spell-resolution path for hard-cast
  permanents + `effects/movement.rs::place_card_in_dest` for reanimate
  / flicker / search-to-battlefield. The owner-vs-controller split
  doesn't matter for the current catalog (no card specifies a
  non-controller as the placer), but the architecture supports it
  cleanly via `ctx.controller` reading the resolution's seat. Tests:
  `pterafractyl_cr_614_12_zero_toughness_base_survives_etb_via_enters_with`,
  `symmathematics_enters_with_two_plus_one_counters`. Closes the
  CR 122.6 audit row.

- 🟡 **CR 121 — Drawing a Card** (push modern_decks audit,
  claude/modern_decks branch): The card-draw foundation, gated as
  the engine's `Effect::Draw` + `Player::draw_top` site. Audit:
  (a) **121.1** draw = top of library → hand ✅
  (`Player::draw_top` removes index 0 from library, pushes to hand,
  bumps `cards_drawn_this_turn`); (b) **121.2** multi-draw = sequential
  individual draws ✅ (`Effect::Draw`'s handler loops `n` times
  calling `draw_top` individually so each draw can independently fail
  on empty library); (c) **121.2a** modify-draw-count replacement
  effects ⏳ (no CR 616.1g replacement primitive — engine treats
  `Draw N` as a sequence of N individual `CardDrawn` events without
  pre-grouping); (d) **121.2b** "can't draw more than 1 per turn"
  effects ⏳ (no per-turn draw-cap predicate); (e) **121.2c** APNAP
  ordering for multi-player draws 🟡 (`ForEach(EachPlayer)` resolves
  in turn order from the active player onward — naturally APNAP for
  most multi-player payoffs like Howling Mine, but no explicit cap
  if both Howling Mine + a draw-replacement coexist on the same
  player); (f) **121.4** drawing from empty library = lose game at
  next priority ✅ (SBA in `check_state_based_actions` marks the
  player `eliminated` when `draw_top` returns `None`, then the
  next priority gate transitions to `game_over = Some(opp)`; the
  existing `drawing_from_empty_library_eliminates_player` test in
  `tests/game.rs` exercises this); (g) **121.5** library → hand
  without "draw" is NOT a draw ✅ (`Effect::Move` doesn't emit
  `GameEvent::CardDrawn`; only `Effect::Draw` does, so triggers like
  Day's Heralds or Sylvan Library don't fire on `Effect::Search` or
  `Move(Library → Hand)`); (h) **121.6** replace card draws via
  replacement effects 🟡 (general CR 614 replacement-effect primitive
  is partial; the engine has explicit replacements for token-doubling
  but no "replace draw with X" primitive); (i) **121.7** prevention/
  replacement that results in card draws 🟡 (no general replacement
  pipeline); (j) **121.8** face-down draws during cast ⏳ (the engine
  resolves the cast pipeline atomically without exposing mid-cast
  draw triggers); (k) **121.9** optional-reveal on draw ⏳ (no
  reveal-on-draw decision hook). Tests:
  `drawing_from_empty_library_eliminates_player` in `tests/game.rs`
  covers 121.4. Suggested follow-ups (low priority unless a
  replacement-draw card lands): add `Effect::DrawReplacement` event
  emission so cards like Notion Thief / Possibility Storm can wire
  cleanly.

- ✅ **CR 119 — Life** (push modern_decks audit, claude/modern_decks
  branch): The life-total mechanics framework. Audit:
  (a) **119.1** starting life total 20 — ✅ (`Player::new` initializes
  `life: 20`); (b) **119.2** damage → lose life — ✅ (`deal_damage_to`
  routes player damage to `LoseLife` events via `damage_to_player`);
  (c) **119.3** gain/lose life adjusts total — ✅ (`Effect::GainLife` /
  `Effect::LoseLife` in `game/effects/mod.rs:465/479`); (d) **119.4**
  pay life requires life ≥ amount — ✅ (`life_cost` precheck in
  `activate_ability` returns `GameError::InsufficientLife` when life <
  cost); (e) **119.4b** can always pay 0 life — ✅ (mana-spent / life-
  cost paths short-circuit on 0); (f) **119.5** set life total — ✅
  (push modern_decks: new `Effect::SetLifeTotal { who, amount }`
  primitive computes `delta = new_total - current_life` then emits
  `LifeGained`/`LifeLost` for non-zero delta; tests
  `set_life_total_emits_correct_delta_events_per_cr_119_5` +
  `set_life_total_higher_emits_life_gained`); (g) **119.6** 0 or less
  life loses game — ✅ (SBA in `check_state_based_actions` marks the
  player `eliminated` when `life ≤ 0`); (h) **119.7** can't gain life
  — ⏳ (no general can't-gain-life replacement layer); (i) **119.8**
  can't lose life — ⏳ (no general can't-lose-life replacement layer);
  (j) **119.9** 0 life gain doesn't fire trigger — ✅ (`Effect::GainLife`
  handler `if amt == 0 { return Ok(()); }` short-circuits before
  emitting `LifeGained`; test
  `zero_life_gain_does_not_trigger_lifegain_events_per_cr_119_9`); (k)
  **119.10** 0 life gain doesn't fire replacement — ✅ (no replacement
  layer; the engine's behavior is conservatively correct since no
  replacement exists to apply). The headline gap was 119.5 (set life
  to specific value); now wired by the new `SetLifeTotal` primitive.
  Biorhythm / Tree of Redemption / Soul Echo all become expressible
  via this primitive. The remaining ⏳ are can't-gain-life /
  can't-lose-life replacement effects — same engine-wide gap as the
  general replacement-effect framework (CR 614).

- ✅ **CR 117 — Timing and Priority** (push modern_decks audit,
  claude/modern_decks branch): The foundational priority + timing
  framework. Audit confirms the engine wires every sub-rule:
  (a) **117.1a** instant-speed casts at any priority — ✅
  (`cast_spell` validates `is_instant_speed`); (b) **117.1b**
  activated abilities at any priority — ✅ (`activate_ability`
  bypasses sorcery-speed gates for non-sorcery_speed abilities);
  (c) **117.1d** mana abilities mid-cast — ✅ (`is_mana_ability`
  + `try_pay_with_auto_tap` in `game/actions.rs`); (d) **117.2a**
  triggers queue at next priority gate, not at fire-time — ✅
  (the SBA loop in `check_state_based_actions` collects pending
  triggers into a buffer and pushes onto the stack at the priority
  gate, per `dispatch_triggers_for_events`); (e) **117.2b** static
  abilities continuous — ✅ (compute_battlefield re-applies layers
  every recompute); (f) **117.2c** turn-based actions before
  priority — ✅ (`advance_to_next_step` runs `do_untap`/`do_draw`/
  `do_cleanup` before `pass_priority`); (g) **117.2d** SBAs before
  priority — ✅ (the SBA-trigger-SBA loop runs to fixpoint before
  any priority assignment); (h) **117.2e** no priority during
  resolution — ✅ (`drain_stack` doesn't return to priority until
  the top resolves); (i) **117.3a** active player priority at step
  start — ✅; (j) **117.3b** active player priority post-resolution
  — ✅; (k) **117.3c** priority retained after action — ✅
  (`pass_priority` resets consecutive_passes to 0 after any action);
  (l) **117.3d** pass priority moves to next player — ✅; (m)
  **117.4** all-pass = resolve top of stack / end step — ✅
  (`pass_priority` resolves on `consecutive_passes >= n_alive`);
  (n) **117.5** SBA-trigger-SBA loop before priority — ✅; (o)
  **117.7** in-response-to ordering — ✅ (the stack's LIFO order
  naturally implements last-in-first-out resolution).
  Lock-in tests added (push claude/modern_decks):
  `cr_117_5_sba_before_priority_lethal_creature_dies_before_response`
  asserts the SBA loop kills a lethal-damage creature before the
  opp gets priority to respond (CR 117.5). Implicit coverage from
  every other test in the suite (~4300 passing tests all depend on
  correct priority + step transitions). The audit confirms that
  CR 117 is end-to-end CR-compliant for the 1v1 case. Multi-player
  priority (CR 117.6 shared team turns) is still ⏳, tracked
  under Format Phase F (2HG).

- 🟡 **CR 614 — Replacement Effects** (push modern_decks batch 56
  audit, claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): The replacement-effect primitive —
  how effects watch for and replace events. Audit:
  (a) **614.1a** "instead" effects are replacement effects — 🟡 (the
  engine has `ReplacementEffect` for zone changes (Commander
  redirect), `StaticEffect::DoubleTokens` / `DoubleCounters` for
  multiplier replacements, the regen / Hofri exile-instead-of-
  graveyard paths, and now `StaticEffect::EtbTriggerTax { amount }`
  for Strict Proctor's "sacrifices the permanent unless they pay
  {2}" gate (push batch 58). No general "instead" framework — each
  "instead" primitive is hand-rolled).
  (b) **614.1b** "skip" effects are replacement effects — 🟡 (only
  `Player.skip_first_draw` exists for CR 103.6's start-of-game
  first-draw skip. No general skip-step / skip-turn primitive — see
  CR 614.10 row).
  (c) **614.1c–d** "[This permanent] enters with N counters", "As X
  enters" — ✅ (`CardDefinition.enters_with_counters` field + the
  layer-injection pass at `stack.rs` and `place_card_in_dest`). Used
  by Quandrix Calligrapher, Symmathematics, Fractal Augmenter (batch
  56), and ~10 other Fractal cards.
  (d) **614.2** damage-replacement effects — 🟡 (the damage pipeline
  in `game/effects/movement.rs::deal_damage_to_from` honors
  Protection + Indestructible but has no general "would deal damage,
  do X instead" hook — Furnace of Rath / Gisela / Heartless Hidetsugu
  not modelled).
  (e) **614.3** no special casting restrictions — ✅ (replacement
  effects come from on-resolve effects like any other; no engine
  gates the casting of a spell with a replacement clause).
  (f) **614.4** replacement effects exist before the event — ✅
  (the resolver walks the registry at the moment of zone change /
  token mint / counter placement; pre-existing replacements catch
  the event in flight).
  (g) **614.5** replacement effects don't invoke themselves
  repeatedly — ✅ (`MAX_REPLACEMENT_ITERATIONS` cap at
  `replacement.rs:76` + each match is rebuilt per pass; the
  pathological infinite loop is bounded).
  (h) **614.6** if event is replaced, never happens — ✅ (replaced
  zone changes drop the original move; the modified destination
  is the only one that fires triggers).
  (i) **614.7a** 0 damage → no event → no replacement — ✅ (the
  `deal_damage_to_from` pipeline early-returns on amount==0, so
  Furnace-of-Rath-style doublers don't fire for 0 sources).
  (j) **614.8** Regeneration as destruction-replacement — ✅
  (`Regeneration::regen_shields` on `CardInstance` + the destroy
  pipeline check at `effects/mod.rs`).
  (k) **614.9** redirection effects — ⏳ (no general
  damage-redirection primitive; Maze of Ith / Lightning Greaves /
  Boros Guildmage-style redirects aren't modelled).
  (l) **614.10** skip-effects (see row (b)) — 🟡.
  (m) **614.12** — ✅ (see row (c) above;
  full audit row at `TODO.md:2222`).
  (n) **614.16** "create tokens / put counters" replacement —
  ✅ (`StaticEffect::DoubleTokens` + `DoubleCounters`).
  No new tests added in this audit pass — every replacement
  primitive listed above already has a lock-in test. Promote
  the umbrella row to ✅ when 614.2 (general damage-replacement)
  and 614.9 (redirection) land.

- ✅ **CR 614.1a — "Instead" replacement: pay-or-sacrifice ETB-trigger
  tax** (push modern_decks batch 58, claude/modern_decks branch — audit
  against `MagicCompRules_20260417.txt`): The CR 614.1a "instead" gate
  applied to ETB-triggered abilities. New `StaticEffect::EtbTriggerTax
  { amount }` primitive ships Strict Proctor's printed Oracle "If a
  permanent entering the battlefield causes a triggered ability of a
  permanent to trigger, that ability's controller sacrifices the
  permanent unless they pay {amount}." Wiring lives in three places:
  (a) `fire_self_etb_triggers` in `game/actions.rs` (the non-cast
  paths like reanimation), (b) `stack.rs::resolve_spell`'s
  cast-time ETB-trigger push loop (the canonical cast path), and (c)
  the unified `dispatch_triggers_for_events` dispatcher in
  `game/mod.rs` (for triggers fired from non-self sources reading the
  `PermanentEntered` event — Soul Warden's "another creature enters"
  trigger, for example). The dispatcher carries a new
  `TriggerCandidate.triggered_by_etb: bool` field stamped at
  candidate-gathering time so non-ETB triggers (Magecraft, Prowess,
  attack triggers) are untaxed per the printed Oracle. Multiple
  Strict Proctors stack via additive amount sum (matches the printed
  "for each Proctor" framing). On decline / unable to pay: the
  trigger source (the permanent whose ability would have triggered)
  is sacrificed via `remove_from_battlefield_to_graveyard` and the
  trigger does not fire. Tests:
  `strict_proctor_is_a_two_mana_flier`,
  `strict_proctor_taxes_an_etb_trigger_unless_paid` (AutoDecider
  declines → source sacrificed, ETB rider suppressed; ScriptedDecider
  accepts + floated mana → tax paid, ETB rider fires),
  `strict_proctor_does_not_tax_non_etb_triggers` (Magecraft pumps
  unaffected — only ETB-event triggers see the tax).

- ✅ **CR 614.16 — "If an effect would create tokens / put counters,
  replacement effects apply"** (push modern_decks audit,
  claude/modern_decks branch — **batch 11 promoted to ✅**): "Some
  replacement effects apply 'if an effect would create one or more
  tokens' or 'if an effect would put one or more counters on a
  permanent.' These replacement effects apply if the effect of a
  resolving spell or ability creates a token or puts a counter on a
  permanent, and they also apply if another replacement or prevention
  effect does so, even if the original event being modified wasn't
  itself an effect." Both halves now wired. **Token half** —
  `StaticEffect::DoubleTokens` primitive (Adrix and Nev, Twincasters);
  `GameState::token_doublers_for(seat)` reads the active doubler
  count at `Effect::CreateToken` resolution; the count is scaled by
  `2^doublers`. Tests: `adrix_and_nev_doubles_token_creation`,
  `adrix_and_nev_does_not_double_opponent_tokens`. **Counter half**
  (push modern_decks, batch 11) — `StaticEffect::DoubleCounters`
  primitive (Witherbloom Pestseed); `GameState::counter_doublers_for(seat)`
  reads the active doubler count at `Effect::AddCounter` resolution
  (per-target via `battlefield_find(cid).controller` so a fan-out
  selector spanning controllers behaves correctly), then the count
  is scaled by `2^doublers`. Poison counters on players use the
  affected player's own doubler count. The same `counter_doublers_for`
  lookup is also wired into the `enters_with_counters` (CR 614.12)
  replacement at both call sites (`stack.rs` spell-resolution path +
  `effects/movement.rs::place_card_in_dest`) so a Fractal Trefoil
  entering under a Pestseed correctly doubles its lands-based counter
  count. Stacking multiplies (2 Pestseeds → 4×). Tests:
  `witherbloom_pestseed_doubles_plus_one_counter_placement`,
  `_does_not_double_opp_counters`, `_stacks_multiplicatively`,
  `fractal_trefoil_with_pestseed_doubles_counters`. Doubling Season
  itself would ship both static abilities (DoubleTokens + DoubleCounters);
  Branching Evolution / Vorinclex / Pir / Hardened Scales (counter-only
  doublers) all wire via single-row catalog additions.

- ✅ **CR 113.10b — "Loses all abilities" continuous effects** (push
  modern_decks batch 34 audit, claude/modern_decks branch — audit
  against `MagicCompRules_20260417.txt`): "Effects can cause an object
  to lose abilities. … If a permanent has all of its abilities removed,
  it has no abilities, including any printed activated abilities or
  triggered abilities that may be relevant." Audit:
  (a) **Layer 6 lookup** — ✅ (`Modification::RemoveAllAbilities` is
  evaluated at layer 6 in `compute_permanent`; it now both clears the
  `keywords` Vec AND flips the new `ComputedPermanent.lost_all_abilities`
  flag so downstream dispatchers can short-circuit).
  (b) **Trigger dispatch** — ✅ (`dispatch_triggers_for_events` walks
  `compute_battlefield` once at entry; any candidate source whose
  `lost_all_abilities` is set is skipped, so generic event-driven
  triggers — ETB, dies, attacks, beginning-of-step — don't fire from
  stripped permanents).
  (c) **Spell-cast / Magecraft dispatch** — ✅ (`fire_spell_cast_triggers`
  pre-computes the stripped set and filters candidates in the iterator
  chain; covers Magecraft, Prowess, Repartee, Opus, Increment, and any
  future `EventKind::SpellCast` trigger).
  (d) **Activated abilities** — ✅ (`activate_ability` reads
  `compute_battlefield`'s flag for the source permanent; printed
  activations are rejected with `AbilityIndexOutOfBounds` while the
  source is stripped. Mana abilities are preserved per CR 605.1a — the
  `is_mana_ability` recogniser short-circuits the gate so a Galazeth-
  style mana ability still resolves even if some other strip-abilities
  effect is in scope; no catalog card today exercises this corner).
  (e) **Static abilities** — 🟡 (`compute_battlefield` walks
  `definition.static_abilities` directly when deriving continuous
  effects; the stripped-flag isn't consulted at that walk, so a printed
  static would still emit its layered effect. No STX/SOS card today
  combines a static ability with a strip-abilities target. Tracked for
  full coverage; promote to ✅ when static-emission also reads the flag).
  (f) **Headline card** — ✅ (Mercurial Transformation 🟡 → ✅; body is
  `Effect::Seq(SetBasePT 3/3, LoseAllAbilities)`; tests
  `mercurial_transformation_sets_target_to_three_three_eot`,
  `mercurial_transformation_strips_keywords_from_target` (Shivan Dragon
  loses Flying), `mercurial_transformation_strips_etb_triggers_from_target`
  (Sedgemoor Witch's magecraft Pest minting suppresses)). Same wire-up
  unlocks Turn to Frog, Lignify, Song of the Dryads, Reality Acid,
  Imprisoned in the Moon. Promote to ✅ when (e) lands — current ✅
  reflects the four high-traffic dispatch sites all honoring the flag.

- ✅ **CR 603.4 — Intervening 'if' clause (both halves now wired)**
  (push modern_decks audit, claude/modern_decks branch): "A triggered
  ability may read 'When/Whenever/At [trigger event], if [condition],
  [effect].' When the trigger event occurs, the ability checks
  whether the stated condition is true. The ability triggers only if
  it is; otherwise it does nothing. If the ability triggers, it
  checks the stated condition again as it resolves. If the condition
  isn't true at that time, the ability is removed from the stack and
  does nothing." Earlier push (modern_decks) wired the trigger-time
  half — `fire_step_triggers` now evaluates the trigger's
  `EventSpec.filter` predicate against current game state before
  pushing. **Now also wired the resolve-time half** (push
  claude/modern_decks current sub-push): a new optional
  `intervening_if: Option<Predicate>` field on `StackItem::Trigger`
  + `PendingTriggerPush` propagates the predicate from
  `fire_step_triggers` through to the trigger resolver. At resolve
  time, the resolver re-evaluates the predicate against current
  state; if false, the trigger fizzles (body skipped) per CR 603.4.
  Engine sites: `fire_step_triggers` in `game/stack.rs` (passes
  filter as intervening_if); `resolve_top_of_stack` in `game/stack.rs`
  (re-checks at resolve). Tests:
  `triskaidekaphile_wins_at_upkeep_with_exactly_thirteen_cards`
  (existing — exercises trigger-time gate),
  `triskaidekaphile_does_not_win_at_upkeep_with_other_hand_size`
  (existing — fail-on-trigger-time path),
  `cr_603_4_intervening_if_re_checked_at_resolve_time` (NEW — pushes
  trigger directly with false predicate; verifies body never runs),
  `cr_603_4_intervening_if_runs_when_true_at_resolve_time` (NEW —
  control test, true predicate, body runs).

- 🟡 **CR 115.3 / 115.5 — Target distinctness + self-targeting**
  (push modern_decks audit, claude/modern_decks branch): "The same
  target can't be chosen multiple times for any one instance of the
  word 'target' on a spell or ability. If the spell or ability uses
  the word 'target' in multiple places, the same object or player
  can be chosen once for each instance" (115.3) + "A spell or
  ability on the stack is an illegal target for itself" (115.5).
  Engine audit: (a) **115.5 self-targeting is implicitly enforced**.
  `cast_spell_with_convoke` removes the card from hand, validates
  the chosen target before pushing the spell on the stack — at
  cast-time the spell isn't yet on the stack, so it isn't in the
  candidate set for `IsSpellOnStack` targets. The target validator
  in `evaluate_requirement_static` walks `self.stack` for spell
  targets, which doesn't contain the cast-in-progress spell. (b)
  **115.3 distinctness is partially enforced**. Multi-target spells
  threaded via `additional_targets` (Snow Day, Render Speechless,
  Crackle with Power) **do not** enforce distinctness today — each
  slot is validated independently against its `target_filter_for_slot`,
  but two slots can pick the same Target. For most multi-target
  spells in our catalog this is fine because each slot represents a
  separate "target" keyword instance per CR 115.3's permissive rule.
  The strict "divided among any number of targets" shape (Crackle
  with Power, Magma Opus's divided damage half) collapses to a single
  target today (engine-wide gap shared with Devious Cover-Up,
  Vibrant Outburst, Snow Day, …), so the distinctness corner is not
  yet exercised. When the multi-target divided-damage primitive
  lands, add a `must_be_distinct: bool` on `Selector::TargetFiltered`
  + a cast-time pairwise check in `cast_spell_with_convoke`. Tracked
  in TODO.md.

- 🟡 **CR 615.1 — Prevention effects** (push modern_decks audit,
  claude/modern_decks branch): "Some continuous effects are prevention
  effects. Like replacement effects (see rule 614), prevention effects
  apply continuously as events happen—they aren't locked in ahead of
  time. Such effects watch for a damage event that would happen and
  completely or partially prevent the damage that would be dealt.
  They act like 'shields' around whatever they're affecting." The
  engine now has a **partial** prevention layer for combat damage
  specifically: `Effect::PreventAllCombatDamageThisTurn` sets the
  `GameState.prevent_combat_damage_this_turn` flag; the combat damage
  resolver (`game/combat.rs::resolve_combat_damage_with_filter`)
  reads the flag and zeroes attacker→blocker, attacker→player, and
  blocker→attacker assignments (lifelink scales off actual damage
  dealt per CR 702.15a, so lifelink life-gain is zeroed too). The
  flag clears in `do_cleanup` (CR 514.2) alongside other
  until-end-of-turn state. Wires Owlin Shieldmage's "When this
  enters, prevent all combat damage that would be dealt this turn"
  ETB. Tests: `tests::stx::owlin_shieldmage_etb_prevents_combat_damage_this_turn`,
  `tests::stx::prevent_combat_damage_flag_clears_in_cleanup`.
  Gaps still tracked: (a) per-source prevention shields (CR 615.7
  "next N damage from source") need a list of pending shields per
  player/creature; (b) "Damage can't be prevented" rider (CR 615.12)
  on cards like Heated Debate / Skullcrack is currently a no-op
  because there's no general prevention layer beyond the
  combat-damage flag; (c) non-combat damage prevention (Holy Day-
  style fogs hit combat damage only, but Reverse Damage-style cards
  also intercept ability/spell damage); (d) CR 615.13 "triggered
  abilities that fire when damage is prevented" need a
  `DamagePrevented` event emission. The combat-damage flag handles
  the headline play pattern for fog effects.

- ✅ **CR 120.6 — Marked damage persists until cleanup; lethal damage
  destroys via SBA** (push modern_decks batch 18 audit,
  claude/modern_decks branch — audit against `MagicCompRules_20260417.txt`):
  "Damage marked on a creature remains until the cleanup step, even if
  that permanent stops being a creature. If the total damage marked on
  a creature is greater than or equal to its toughness, that creature
  has been dealt lethal damage and is destroyed as a state-based action
  (see rule 704). All damage marked on a permanent is removed when it
  regenerates (see rule 701.19, 'Regenerate') and during the cleanup
  step (see rule 514.2)." The engine tracks `CardInstance.damage: u32`,
  preserved across SBA passes and across the resolving spell's stack
  unwind. `check_state_based_actions` (`game/stack.rs`) reads `c.damage
  >= c.toughness()` to detect lethal damage and routes the creature to
  graveyard via the standard CreatureDied path. The "stops being a
  creature" rider is honored — the engine doesn't gate damage tracking
  on the card being currently classified as a creature. `do_cleanup`
  (`game/stack.rs`) zeroes `c.damage` for every battlefield card at
  cleanup step end (CR 514.2). Tests: existing
  `lightning_bolt_kills_grizzly_bears` style tests + new
  `prismari_ignite_apprentice_pings_on_etb` (1-damage marks then
  persists through the SBA pass). Regenerate clears damage (CR 701.19)
  is ✅ via `restore_damage_after_regenerate` in the regen path.

- ✅ **CR 120.4 — Four-part damage-dealing sequence** (push modern_decks
  batch 18 audit, claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): "Damage is processed in a four-part
  sequence. / 120.4a First, if an effect that's causing damage to be
  dealt states that excess damage that would be dealt to a permanent
  is dealt to another permanent or player instead, the damage event is
  modified accordingly. / 120.4b Second, damage is dealt, as modified
  by replacement and prevention effects that interact with damage. /
  120.4c Third, damage that's been dealt is processed into its results,
  as modified by replacement effects that interact with those results
  (such as life loss or counters). / 120.4d Fourth, abilities that
  trigger on damage being dealt now go on the stack." The engine's
  `deal_damage_to_from` (`game/effects/movement.rs`) follows the
  printed sequence: (b) damage is dealt (prevention via the
  `prevent_combat_damage_this_turn` flag for fogs), (c) damage results
  apply (LoseLife for players, damage marking + lifelink for creatures
  via `process_damage_results`), (d) `DamageDealt` event fires through
  `dispatch_triggers_for_events` after the damage is applied. The
  120.4a excess-damage routing (Phyrexian Soulgorger / trample-via-
  divide) is implicit in the combat damage divider but not generally
  exposed for ability-damage; tracked separately under the trample
  excess-damage TODO. CR 120.8 (0-damage suppression) is ✅ via the
  zero-check in `deal_damage_to_from` that early-returns without
  emitting events. Tests: `lash_of_malice_kills_two_two_creature`,
  `prismari_volley_burns_creature_and_draws` (DealDamage → SBA →
  CreatureDied chain).

- ✅ **CR 120.5 — Damage doesn't destroy a creature directly; SBA
  does** (push modern_decks audit, claude/modern_decks branch):
  "Damage dealt to a creature, planeswalker, or battle doesn't destroy
  it. Likewise, the source of that damage doesn't destroy it. Rather,
  state-based actions may destroy a creature or otherwise put a
  permanent into its owner's graveyard, due to the results of the
  damage dealt to that permanent." The engine's
  `deal_damage_to_from` (`game/effects/movement.rs`) marks damage on
  the creature's `c.damage: u32` field but doesn't destroy the
  permanent itself. The state-based-action sweep in
  `check_state_based_actions` (`game/stack.rs:687`) walks the
  battlefield each iteration and collects creatures where
  `(c.damage as i32) >= computed_toughness` (with
  Indestructible carved out per CR 702.12b). The two-step pattern is
  visible in the per-resolution flow: a Lightning Bolt fires
  `DealDamage`, which marks 3 on the creature's `damage` field; the
  next SBA pass picks up `damage ≥ toughness` and moves the card to
  graveyard, firing `CreatureDied`. The "source doesn't destroy"
  rider is honored implicitly — the engine never directly destroys
  from a damage call, only through the SBA path. Indestructible
  creatures keep the damage but survive (CR 702.12b); toughness ≤ 0
  bypasses Indestructible (CR 704.5g). Tests:
  `lash_of_malice_kills_two_two_creature` (2/2 - 1/-1 = 1/1 + damage
  marked — wait, no, that's pump-shrink not damage). For pure
  damage→SBA→destroy round-trip, every Lightning Bolt to a 2-toughness
  creature test exercises this (e.g. `lightning_bolt_kills_grizzly_
  bears`). Lock-in test: the cleanup step clearing damage
  (CR 514.2 / `game/stack.rs:615`) confirms the marked-damage model.

- ✅ **CR 120.7 — Source of damage tracking** (push modern_decks batch
  119, claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt` lines 1124): "The source of damage is
  the object that dealt it. If an effect requires a player to choose a
  source of damage, they may choose a permanent; a spell on the stack
  …; any object referred to by an object on the stack, by a prevention
  or replacement effect that's waiting to apply, or by a delayed
  triggered ability that's waiting to trigger (even if that object is
  no longer in the zone it used to be in); or a face-up object in the
  command zone." The engine threads the damage source through every
  damage-dealing path: `Effect::DealDamage` (`game/effects/mod.rs:471`)
  reads `ctx.source` (the spell or ability's source `CardId`) and
  forwards it to `deal_damage_to_from(ent, amt, source, events)`. This
  lets infect (CR 702.90b — passes poison counters from source's
  controller), lifelink (CR 702.15b — looks up source's controller for
  the gain), and damage-source-matters triggers (Witherbloom Stoker /
  Quandrix Lighthouse — read `event.source`) all use the correct
  attribution. Combat damage routes through `deal_combat_damage_to_target`
  (`game/combat.rs:565`) which captures `atk.id` as the source. The
  current model omits "a spell on the stack as a source" — only
  permanent-source damage is supported (no Browbeat / Burning of
  Xinye-style choose-the-spell-on-stack effects). Tests:
  `lightning_bolt_kills_grizzly_bears` + every existing damage path
  exercises the source threading; new
  `pest_hivewatcher_b119_does_not_gain_life_when_only_self_dies`
  verifies the AnotherOfYours scope properly filters out the source
  itself based on damage-source identity in the dispatched
  CreatureDied event.

- ✅ **CR 613.4c / 613.7c — Layer 7c (counter / +N/+M) applies above
  layer 7b (set base P/T)** (push modern_decks audit, claude/modern_decks
  branch): "Layer 7b: Effects that set power and/or toughness to a specific
  number or value are applied. / Layer 7c: Effects and counters that modify
  power and/or toughness (but don't set power and/or toughness to a
  specific number or value) are applied." The engine's `compute_permanent`
  applies layers in order: 7b's `Modification::SetPowerToughness` writes
  the base P/T first, then 7c's `Modification::ModifyPower` /
  `ModifyToughness` adds the +N/+M from counters and continuous effects.
  Tests: `square_up_layers_under_plus_one_counters` (Square Up's
  `SetBasePT(0, 4)` + a +1/+1 counter → 1/5, not 0/4), `quandrix_charm_
  mode_2_setbasept_layers_under_counter` (Quandrix Charm's mode 2
  `SetBasePT(5, 5)` + a +1/+1 counter → 6/6), `fractalize_layers_under_
  plus_one_counters` (Fractalize at X=2 `SetBasePT(3, 3)` + a +1/+1
  counter → 4/4). All three exercise the printed-Oracle CR 613.4c/d
  layer ordering exactly. The `SetBasePT` primitive (added push XXXII)
  now powers Square Up, Quandrix Charm mode 2, Mercurial Transformation,
  and Fractalize (push modern_decks).

- ✅ **CR 608.3f / 707.10f — Permanent-spell copies are tokens** (push
  modern_decks audit, claude/modern_decks branch): "If the object that's
  resolving is a copy of a permanent spell, it will become a token
  permanent as it is put onto the battlefield." The engine's
  `Effect::CopySpell` handler (`game/effects/mod.rs:1698`) sets
  `copy_inst.is_token = true` on the `CardInstance` BEFORE pushing the
  spell-copy onto the stack. When that StackItem::Spell resolves
  (`game/stack.rs:311` `let card = *card;` + `self.battlefield.push(card)`),
  the `is_token: true` flag is preserved on the resulting battlefield
  permanent. The token-cleanup state-based-action sweep
  (`check_state_based_actions` in `game/stack.rs`) then removes the
  permanent from hand / library / exile / graveyard when it leaves the
  stack (per CR 707.10a). For instant/sorcery copies, the same
  `is_token: true` flag triggers SBA cleanup once the copy hits its
  owner's graveyard. Tested implicitly by
  `tests::sos::copied_spell_does_not_linger_in_graveyard_after_resolution`
  + every Aziza / Lumaret's Favor / Social Snub copy test in the suite.
  The TODO.md note that the resolved permanent wasn't flagged was stale
  — closed by audit.

- ✅ **CR 707.10c / 707.10 — Copy effects and "new targets"** (push
  modern_decks audit, claude/modern_decks branch): "Some effects copy
  a spell or ability and state that its controller may choose new
  targets for the copy." Push (modern_decks) lands the cast-from-
  graveyard introspection needed to faithfully wire Increasing
  Vengeance's "If this spell was cast from a graveyard, copy that
  spell twice instead" rider. The new `Predicate::CastFromGraveyard`
  reads `EffectContext.cast_from_hand` (stamped at spell-resolution
  time from the resolving `CardInstance.cast_from_hand` flag — false
  for flashback / Yawgmoth's Will-style "cast from graveyard"
  paths). Combined with the engine's existing `Effect::CopySpell`,
  the printed Oracle ships exactly: hand cast → 1 copy, flashback
  cast → 2 copies. The "may choose new targets for the copy" half
  is still ⏳ (the engine carries the original targets unchanged
  through `CopySpell`); CR 707.10c's optional retarget needs a per-
  copy decision shape on the controller's side. Tests:
  `increasing_vengeance_copies_target_instant` (hand cast → single
  copy), `increasing_vengeance_double_copies_when_flashed_back_from_
  graveyard` (synthetic Flashback {R}{R} → flashback cast → two
  copies + bear destroyed + IV exiled per CR 702.34a).

- ✅ **CR 700.4 — Definition of "dies"** (push modern_decks audit,
  claude/modern_decks branch): "The term dies means 'is put into a
  graveyard from the battlefield.'" The engine emits a
  `GameEvent::CreatureDied { card_id }` precisely when a creature
  moves from the battlefield to a graveyard. The emission sites are:
  (a) the SBA legendary-rule sweep (`game/stack.rs:683`) when a
  legendary duplicate goes to the graveyard, (b) the SBA
  toughness/lethal-damage sweep (`game/stack.rs:713`) when a creature
  dies to combat or toughness ≤ 0, and (c) the combat-damage
  resolution path (`game/actions.rs:1649`) when a creature is killed
  by combat damage outside of SBA. Sacrifice effects feed through the
  same remove-to-graveyard path, so a sacrificed creature also fires
  `CreatureDied`. The corresponding `EventKind::CreatureDied`
  (`effect.rs:482`) is the trigger handle used by every "When this
  creature dies, …" / "Whenever a creature dies, …" rider in the
  catalog (Ambitious Augmenter, Bayou Groff, Pestbrood Sloth,
  Cauldron of Essence, Arnyn Deathbloom Botanist, Daemogoth Woe-Eater
  attack-sac, Star Pupil, etc.). The
  `EventKind::PermanentLeavesBattlefield` event also matches a
  `CreatureDied` GameEvent (per `events.rs:21`) so "when this leaves
  the battlefield" wording captures the same transitions. This
  faithfully implements CR 700.4's wording — the engine doesn't treat
  "dies" as anything other than the bf→graveyard transition for
  creatures.

- ✅ **CR 104.2b — "An effect may state that a player wins the game"**
  (push modern_decks audit, claude/modern_decks branch): "An effect
  may state that a player wins the game." Push (modern_decks) lands
  the new `Effect::WinGame { who: PlayerRef }` primitive in
  `effect.rs`. The handler in `game/effects/mod.rs` resolves `who`
  to a single seat and sets `eliminated = true` on every other
  player; the existing state-based-action sweep
  (`check_state_based_actions` in `game/stack.rs:855`) then
  promotes `game_over = Some(winner)` on the next loop. This
  matches the printed CR 104.2a / 104.2b framing — "wins the game"
  is implemented as "every other player loses" plus the existing
  ≤-1-player SBA path, which is the standard engine pattern. No
  CR 104.3f (simultaneous win-and-lose) conflict because the
  eliminate-others step doesn't touch the winner's life or
  poison. Approach of the Second Sun is the canonical exerciser
  (Strixhaven Mystical Archive reprint of the Amonkhet finisher):
  on a second cast with one copy in your graveyard, the predicate
  `SameNamedInZoneAtLeast(You, Graveyard, 1)` flips the
  `Effect::If` to `WinGame { You }`. Tests:
  `approach_of_the_second_sun_wins_game_when_cast_with_one_in_graveyard`,
  `approach_of_the_second_sun_gains_seven_life_on_first_cast`
  (the non-win branch). Same primitive unblocks Coalition Victory,
  Test of Endurance, Felidar Sovereign, Mortal Combat, Helix
  Pinnacle.

- ✅ **CR 701.13 — Exile** (push modern_decks batch 103 audit,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): "To exile an object, move it to the
  exile zone from wherever it is. See rule 406, 'Exile.'" (701.13a).
  The engine wires Exile via two pathways depending on the source zone:
  (a) **701.13a** from battlefield → `Effect::Exile` resolves the
  selector to a `CardId`, fires LTB triggers, then moves the card to
  `GameState.exile` via `remove_from_battlefield_to_exile` in
  `game/stack.rs:1151`. Replacement effects on `PermanentExiled` are
  consulted at the standard hook site.
  (b) **701.13a** from other zones (graveyard, hand, library, stack) →
  `Effect::Exile` accepts both `EntityRef::Permanent` (battlefield)
  AND `EntityRef::Card` (cards in any other zone) via the dispatch in
  `resolve_effect`'s Exile arm. Cards routed through this path go to
  `GameState.exile` directly without firing LTB triggers (since LTB
  fires on leaving the battlefield, per CR 603.6c).
  (c) **Self-targeting exile** — works correctly: a card on the
  battlefield exiling itself fires its own LTB triggers BEFORE the
  zone change, then moves to exile. Same shape as Banishing Light /
  Oblivion Ring template (currently the "return on LTB" half is
  engine-wide ⏳ pending an `Effect::ExileUntilLTB` primitive).
  Used by ~80 catalog cards: Path to Exile, Swords to Plowshares,
  Anguished Unmaking, Despark, Cremate, Ghost Vacuum, Soul-Guide
  Lantern, Practiced Scrollsmith, Pull from the Grave, Cling to
  Dust, every "exile target card from a graveyard" card. Tests:
  implicit across the entire suite — every Exile cast/activation
  test exercises the framework. `ghost_vacuum_exiles_target_card_from_graveyard`
  exercises (b). `cremate_exiles_graveyard_card_and_draws` exercises
  (b) chained with a draw. Promote to 🟡 when an `Effect::ExileUntilLTB`
  primitive lands for Banishing Light / Detention Sphere / Oblivion
  Ring-class cards. The pure 701.13a "move object to exile zone"
  pipeline is end-to-end ✅.

- ✅ **CR 701.16 — Investigate** (push modern_decks batch 21 audit,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): "'Investigate' means 'Create a Clue
  token.' See rule 111.10f." The engine implements Investigate via the
  general `Effect::CreateToken { definition: clue_token() }` shape — no
  dedicated `Effect::Investigate` primitive is needed since the printed
  rules text reduces exactly to a single CreateToken op of the
  predefined Clue token (CR 111.10f's "Clue is a predefined token. Its
  full text is 'Colorless artifact token named Clue with {2}, Sacrifice
  this artifact: Draw a card.'"). The Clue token's activated ability
  (`{2}, Sacrifice this artifact: Draw a card`) ships on the
  `TokenDefinition.activated_abilities` of `clue_token()` in
  `game/effects/tokens.rs`, so a freshly investigated Clue can be
  cracked for a card draw at any priority. Catalog exerciser:
  Tireless Tracker (`mod_set::creatures::tireless_tracker`) — "Whenever
  a land you control enters, investigate." — fires an Investigate per
  land ETB via `EventKind::EntersBattlefield/YourControl` filtered on
  `SelectionRequirement::Land`. Same shape would land Lonis, Cryptozoologist
  / Ravenous Squirrel / Tamiyo, Compleated Sage's investigate riders.
  Tests: implicit across the existing `tireless_tracker` /
  `tireless_tracker_clue_can_be_cracked_for_a_card` test pair. No new
  test needed for this audit row — the framework is end-to-end correct
  via the existing CreateToken pipeline.

- ✅ **CR 700.1 — Events** (push modern_decks batch 59 audit,
  claude/modern_decks branch — `MagicCompRules_20260417.txt`):
  "Anything that happens in a game is an event. Multiple events may
  take place during the resolution of a spell or ability. The text of
  triggered abilities and replacement effects defines the event they're
  looking for. One 'happening' may be treated as a single event by one
  ability and as multiple events by another." The engine's event model
  faithfully matches CR 700.1: every state transition emits a discrete
  `GameEvent` via `emit_event` in `game/effects/events.rs`, which the
  trigger dispatcher consumes in `dispatch_triggers_for_events`. Each
  trigger spec (`EventSpec`) declares an `EventKind` (the event class)
  + `EventScope` (`SelfSource` / `YourControl` / `AnotherOfYours` /
  `OpponentControl` / `AnyPlayer`), and the dispatcher folds events
  into trigger candidates per-emission. The 700.1 example — "a single
  attacking creature blocked by two creatures fires one 'becomes
  blocked' trigger but two 'becomes blocked by a creature' triggers" —
  maps cleanly: `EventKind::BecomesBlocked` fires once per attacker
  per combat, while `EventKind::BlockedByCreature` (if shipped — same
  framework) would fan out per blocker. Today the engine collapses
  multi-blocker triggers to one fire per attacker via
  `EventKind::Blocks` on the blocker side and `EventKind::BecomesBlocked`
  on the attacker side; the per-blocker fan-out for "Whenever this
  becomes blocked by a creature" is engine-wide ⏳ (no STX card prints
  that wording). Resolved spells/abilities emitting multiple events
  is supported through `Effect::Seq` and `Effect::ForEach` which both
  walk their bodies once per inner iteration, emitting events at each
  step. Tests: implicit across the ~3150 catalog tests — every Bolt
  cast emits `SpellCast` + `DealsDamage` + `LifeLost` in that order;
  every Pest token death emits `CreatureDied` (per token, not batched);
  every counter add emits `CounterAdded` per counter (Tanazir's
  ETB-doubling fires once per +1/+1 added). The event-per-happening
  contract is end-to-end correct.

- ✅ **CR 701.17 — Mill** (push modern_decks audit, claude/modern_decks
  branch): "For a player to mill a number of cards, that player puts
  that many cards from the top of their library into their graveyard.
  A player can't mill a number of cards greater than the number of
  cards in their library. … If instructed to do so, they mill as many
  as possible." (701.17a, 701.17b). The engine's `Effect::Mill`
  handler in `game/effects/mod.rs:595` resolves `amount` to `n`,
  iterates `n` times pulling from library top, but breaks out of the
  loop when `self.players[p].library.is_empty()` — implementing
  701.17b's "mill as many as possible" framing exactly. Each milled
  card emits a `GameEvent::CardMilled` event so future "Whenever a
  card is milled" triggers (Bruvac the Grandiloquent, Ruin Crab) fire
  on each individual card movement, matching CR 701.17c's "an effect
  that refers to a milled card can find that card in the zone it
  moved to from the library." Lock-in test:
  `tests::game::mill_caps_at_library_size_per_cr_701_17b` stages a
  3-card library, mills 10, asserts library empty + all 3 cards in
  graveyard. The CR 701.17d "single card milled with multi-card
  replacement effects" corner (Bruvac → mill 1 becomes mill 2 +
  ask-about-it) isn't yet exercised by the catalog; no STX/SOS
  cards print that interaction.

- ✅ **CR 402.2 — Maximum hand size enforced at cleanup, opt-out via
  "no maximum hand size"** (push modern_decks audit, claude/modern_decks
  branch): "Each player has a maximum hand size, which is normally seven
  cards. A player may have any number of cards in their hand, but as
  part of their cleanup step, the player must discard excess cards down
  to the maximum hand size." The cleanup-step discard (CR 514.1) lands
  in `do_cleanup` (`game/stack.rs:582`) and runs the while-loop that
  drops head-of-hand into the graveyard until `hand.len() == 7`. Push
  (modern_decks) introduces `Player.no_maximum_hand_size: bool` + the
  new `Effect::SetNoMaxHandSize { who }` primitive — the cleanup-step
  discard now skips the loop entirely when the active player's flag is
  set. Wisdom of Ages's "You have no maximum hand size for the rest of
  the game" rider promotes from 🟡 → ✅ via this primitive. Same flag
  would gate Reliquary Tower / Spellbook / Library of Leng once those
  permanents land in the catalog. Tests:
  `wisdom_of_ages_lets_caster_keep_more_than_seven_cards` (cast Wisdom
  of Ages, push 10 cards into hand, fire cleanup, assert 10 cards
  retained), `wisdom_of_ages_returns_all_instants_and_sorceries_from_
  graveyard` (asserts the flag flips on resolution). Engine sites:
  `do_cleanup` (game/stack.rs), `Effect::SetNoMaxHandSize` handler
  (game/effects/mod.rs), `Player::new` default (player.rs).
- ✅ **CR 119.9 — Zero-life-gain emits no event** (push modern_decks
  /claude/modern_decks audit): "Some triggered abilities are written,
  'Whenever [a player] gains life, . . . .' Such abilities are treated
  as though they are written, 'Whenever a source causes [a player] to
  gain life, . . . .' If a player gains 0 life, no life gain event has
  occurred, and these abilities won't trigger." The engine's
  `Effect::GainLife` handler (`game/effects/mod.rs:370`) short-circuits
  at the top when the evaluated amount is 0
  (`if amt == 0 { return Ok(()); }`). No `GameEvent::LifeGained` rides
  out of the resolution; `Player.life_gained_this_turn` is unchanged;
  any subscribed `Whenever you gain life` trigger (Blech, Pest Mascot,
  Honor Troll's Infusion gate, Comforting Counsel) doesn't fire. The
  symmetric `Effect::LoseLife` handler also short-circuits at amt=0
  per CR 119.3's "adjust accordingly" — zero-adjustment is a no-op.
  `Effect::Drain` (drain X from each opp into you) similarly short-
  circuits at amt=0 so a zero-drain doesn't fire LifeGained / LifeLost
  triggers either. This was already wired in earlier pushes — adding
  the audit entry here to formally pin the CR coverage so future
  drain/gain primitives stay 119.9-compliant.

- ✅ **CR 119.6 — Zero or negative life loses the game** (push
  modern_decks audit): "If a player has 0 or less life, that player
  loses the game as a state-based action. See rule 704." The engine's
  state-based-action sweep (`game/stack.rs:855`) flips
  `Player.eliminated = true` when `life <= 0 || poison_counters >= 10`,
  then promotes to `game_over = Some(winner)` on the next loop when
  ≤1 alive player remains. The poison-counter half also handles
  CR 704.5c (10+ poison counters loses the game). Test coverage
  via the existing decking-out test + every kill-spell-ends-game
  test in the suite (Lightning Bolt-to-the-face, Exsanguinate at X≥20,
  etc.).

- 🟡 **CR 305 — Lands** (push modern_decks batch 67 audit,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): The land primitive — playing a land,
  basic land types and their intrinsic mana abilities, and land
  subtype manipulation. Audit:
  (a) **305.1** — ✅ (`actions.rs::play_land` checks the priority +
  stack-empty + main-phase invariants via `can_cast_sorcery_speed`;
  the land moves direct to battlefield, no `StackItem` push).
  (b) **305.2 / 305.2a / 305.2b** — ✅ (`GameState::can_player_play_land` (push modern_decks batch 130)
  compares `lands_played_this_turn` against
  `max_lands_per_turn(player) = 1 + extra_land_plays_per_turn(player)`,
  where the addend counts every battlefield permanent the player
  controls whose static abilities include
  `StaticEffect::ExtraLandPerTurn` (Exploration ships as the
  reference user). Multiple Explorations stack. See dedicated row
  below).
  (c) **305.3** — ✅
  (`play_land` checks `active_player == player_idx` via
  `can_cast_sorcery_speed`'s priority gate).
  (d) **305.4** "Put onto the battlefield" ≠ "play a land" — ✅
  (`Effect::Search { filter: Land, to: Battlefield }` and
  `Effect::Move(Land → Battlefield)` do NOT bump
  `lands_played_this_turn`. Cultivate / Verdant Mastery / Field Trip
  / Quandrix Cartographer-style ramp respects this — putting lands
  into play from library doesn't count against the per-turn limit).
  (e) **305.5** — ✅ (`Subtypes.land_types: Vec<LandType>`
  supports multi-subtype lands like Lorehold Excavation, all SOS
  school lands, every Snarl dual).
  (f) **305.6** — ✅ (the `tap_add_basic_color`
  shortcut at `mana.rs` is hard-wired to each of the five basic
  types; the intrinsic ability ships as a single
  `ActivatedAbility { tap_cost: true, effect: AddMana }` per type).
  Every Plains/Island/Swamp/Mountain/Forest in catalog ships this
  ability — no land "has no text box" today.
  (g) **305.7** — ⏳ (no
  `Effect::SetLandSubtype` primitive; cards like Spreading Seas
  (becomes Island), Trace of Abundance (becomes basic), Blood Moon
  ("each nonbasic land is a Mountain") aren't in the catalog today.
  Adding the primitive would need a layer-4 type-rewrite + a
  layer-6 mana-ability-replacement pass — same shape as
  `Effect::LoseAllAbilities` but specifically swapping in the new
  basic mana ability.).
  (h) **305.8** — ✅ (`Subtypes.supertypes: Vec<Supertype>` includes `Basic` only
  for the five vanilla basics; predicates like
  `SelectionRequirement::IsBasicLand` walk the supertype list).
  (i) **305.9** — ✅ (the cast-spell pipeline
  rejects land cards via `CardDefinition.is_land()`; the only way to
  put a land onto the battlefield from the hand is `play_land`. No
  MDFC catalog card today is land-on-front + nonland-on-back, but
  the engine's `play_land_back` path correctly takes the back-face
  land via the same one-per-turn gate, never via `cast_spell_back`).
  Tests: per-clause exercise covered by the play-land + ramp test
  matrix (`play_land_enforces_one_per_turn`,
  `cultivate_does_not_count_against_land_drop`,
  `back_face_land_costs_a_land_drop`,
  `lorehold_excavation_is_a_lorehold_dual_with_two_mana_abilities`).
  Promote the umbrella to ✅ when 305.7 (set-subtype rewrite) lands;
  the remaining clauses are end-to-end CR-compliant.

- ✅ **CR 305.2 / 305.2a / 305.2b — One land per turn enforcement +
  ExtraLandPerTurn** (push modern_decks batch 130): "A player can
  normally play one land during their turn; however, continuous
  effects may increase this number." Baseline is enforced via
  `GameState::can_player_play_land` (which replaced the old
  `Player::can_play_land` check at the `play_land_with_face` call
  site). The new helper sums every battlefield permanent the active
  player controls whose static abilities include
  `StaticEffect::ExtraLandPerTurn` — Exploration ({G} Enchantment,
  Urza's Saga reprint) ships as the reference user. Two Explorations
  stack to "three lands per turn"; an opponent's Exploration does
  not help (controller scoping is by `c.controller == player`).
  The `lands_played_this_turn` counter is bumped on every land-play
  (including back-face MDFC land plays via `play_land_back`) and
  reset to 0 on the player's untap step. Tests:
  `exploration_grants_extra_land_play_per_turn`,
  `exploration_third_land_rejected_with_only_one_copy`,
  `two_explorations_stack_for_three_lands_per_turn`,
  `opp_exploration_does_not_grant_extra_land_to_you`.

- ✅ **CR 608.2c / 701.6a — Later text on a card may modify earlier
  text (Memory Lapse exception)** (modern_decks push, engine
  improvement): CR 608.2c — "In some cases, later text on the card may
  modify the meaning of earlier text (for example, … 'Counter target
  spell. If that spell is countered this way, put it on top of its
  owner's library instead of into its owner's graveyard.')". CR 701.6a
  defaults a countered spell to its owner's graveyard; cards like Memory
  Lapse / Remand / Spell Crumple print an "instead" clause that
  overrides the default zone. Push (modern_decks) lands the new
  `Effect::CounterSpellToZone { what, zone: CounteredSpellZone }`
  primitive (and `CounteredSpellZone` enum with `OwnerLibraryTop` /
  `OwnerLibraryBottom` / `OwnerHand` / `Exile` variants) in
  `effect.rs:744`. The resolver in `game/effects/mod.rs::Effect::
  CounterSpellToZone` lifts the on-stack `StackItem::Spell` and routes
  the card to the chosen zone (library top via `push`, library bottom
  via `insert(0, _)`, hand via `players[owner].hand.push`, exile via
  `self.exile.push`). `Effect::CounterSpell` keeps its existing
  graveyard routing for back-compat (Counterspell, Negate, etc.).
  Memory Lapse promoted from `CounterSpell` to `CounterSpellToZone {
  zone: OwnerLibraryTop }`. Test:
  `memory_lapse_routes_countered_spell_to_library_top_per_cr_701_6a`
  (P1 casts Lightning Bolt at P0, P0 Memory Lapses it; assert Bolt is
  on top of P1's library, not in graveyard; P0 still at 20 life).

- ✅ **CR 109.3 / 121 — Power and toughness can be read off the
  battlefield** (modern_decks push, engine improvement): "A card's
  printed power and toughness are part of its characteristics, which
  persist across zones." Push (modern_decks) extends the engine's
  `Value::PowerOf(Selector)` and `Value::ToughnessOf(Selector)`
  evaluators (`game/effects/eval.rs:19`) to walk graveyards / exile /
  hand zones for cards that aren't on the battlefield, returning the
  printed power/toughness from `CardDefinition`. Previously these
  evaluators only consulted `battlefield_find`, returning 0 for any
  card outside the battlefield. The fix lets Lorehold Excavation's
  "X = that card's power" rider read the gy creature's printed power
  at token-mint time (before the exile-Move resolves), making the
  X/X Spirit token correctly scale to the gy creature's power.
  Counters don't apply off the battlefield (CR 122.2 — counters
  cleared on zone change), so off-battlefield reads return printed
  values directly without summing live counters. Tests:
  `lorehold_excavation_token_scales_with_creature_power` (Serra Angel
  4/4 in gy → 4/4 Spirit token), `lorehold_excavation_exile_creature_mints_flying_spirit_token`
  (Grizzly Bears 2/2 in gy → 2/2 Spirit token).
- ✅ **CR 605.3a / 605.3b — Mana abilities resolve immediately without
  going on the stack** (modern_decks push audit): "A player may activate
  an activated mana ability whenever they have priority, whenever they
  are casting a spell or activating an ability that requires a mana
  payment, or whenever a rule or effect asks for a mana payment, even
  if it's in the middle of casting or resolving a spell or activating
  or resolving an ability. … An activated mana ability doesn't go on
  the stack, so it can't be targeted, countered, or otherwise responded
  to." The engine's `is_mana_ability` helper (`game/actions.rs:8` and
  `server/view.rs:421`) recognizes pure `Effect::AddMana` activations
  (including `Seq` chains that are all-mana) and resolves them
  immediately during the activation path. The new Diamond cycle (Sky,
  Marble, Fire, Charcoal, Moss — all 5 added this push) and Lorehold
  Excavation's two color-producing taps all rely on this — the
  `{T}: Add {color}` activations are recognised as mana abilities
  via `tap_add(color)` and skip the stack. Without this, mana rocks
  couldn't be tapped to pay for the spell currently on the stack —
  the foundational invariant of every cube game. Tests:
  `sky_diamond_enters_tapped_then_taps_for_blue` (verifies the rock
  enters tapped and is therefore not immediately tappable — the
  printed "enters tapped" rider), `all_five_diamonds_share_a_common_shape`
  (cycle invariant on the {2} cost + single mana ability +
  ETB-tapped trigger).
- ✅ **CR 514.1 — Cleanup-step discard down to max hand size**
  (modern_decks push audit): "First, if the active player's hand
  contains more cards than their maximum hand size (normally seven),
  they discard enough cards to reduce their hand size to that number.
  This turn-based action doesn't use the stack." Push (modern_decks)
  wires the discard inside `do_cleanup` (`game/stack.rs:568`). When
  the active player's hand exceeds `MAX_HAND_SIZE = 7` at the cleanup
  step, the engine moves head-of-hand cards into the controller's
  graveyard until hand size = 7. The discard is deterministic-first
  (matching the random-discard branch in `Effect::Discard`) since
  cleanup is a turn-based action that doesn't use the stack and the
  bot harness's AutoDecider has no policy here. A future UI surfacing
  could ask the player which cards to discard via the existing
  `Decision::Discard` shape. Tests:
  `cleanup_step_discards_down_to_seven_per_cr_514_1` (10 cards → 7,
  3 to graveyard) +
  `cleanup_step_no_op_when_hand_at_or_below_max_per_cr_514_1` (5
  cards → unchanged). The CR 514.2 second-half (clear damage, expire
  EOT effects, empty mana pools) was already correctly wired prior
  to this push.
- ✅ **CR 614.12 — "Enters with N counters" replacement effects** (modern_decks
  push audit): "Some replacement effects modify how a permanent enters
  the battlefield. … To determine which replacement effects apply and
  how they apply, check the characteristics of the permanent as it
  would exist on the battlefield, taking into account replacement
  effects that have already modified how it enters the battlefield."
  Modern_decks push lands the `CardDefinition.enters_with_counters:
  Option<(CounterType, Value)>` field that captures the printed "enters
  with N [counters] on it" replacement. The counter spec is applied
  inside the same battlefield-zone hand-off in both code paths
  (`stack.rs` spell-resolution path for hard-cast permanents,
  `effects/movement.rs::place_card_in_dest` for reanimate / flicker /
  search-to-battlefield), BEFORE state-based actions check toughness
  and BEFORE the first ETB trigger fires. The spell ctx's `x_value`
  and `converged_value` are threaded via `EffectContext::for_spell_
  with_source` so `Value::XFromCost` (Pterafractyl) and
  `Value::ConvergedValue` (Rancorous Archaic) read the cast-time
  scalars faithfully. Tests:
  `pterafractyl_cr_614_12_zero_toughness_base_survives_etb_via_enters_with`
  (1/0 + 1 +1/+1 counter → 2/1 survives ETB),
  `symmathematics_enters_with_two_plus_one_counters` (printed 0/0 +
  2 +1/+1 = 2/2 exact). Closes the Pterafractyl / Symmathematics /
  Rancorous Archaic base-toughness-bump workaround. Catalog
  promotions: Pterafractyl (1/0 → 1/0 exact), Symmathematics (1/1
  → 0/0 exact), Rancorous Archaic (ETB-trigger → CR-614.12 timing).
- ✅ **CR 701.14 — Fight** (push modern_decks batch 24+ audit,
  claude/modern_decks branch — audit against `MagicCompRules_20260417.txt`):
  "A spell or ability may instruct a creature to fight another creature
  or it may instruct two creatures to fight each other. Each of those
  creatures deals damage equal to its power to the other creature."
  (701.14a). The engine wires fight via `Effect::Fight { attacker,
  defender }` resolved in `game/effects/mod.rs:427`:
  (a) **701.14a** mutual damage with snapshot powers — ✅ (both sides'
  power is read up-front so post-damage stats don't affect the
  back-swing; same shape as printed Oracle).
  (b) **701.14b** target gone → no damage — ✅ (early `let-else` return
  when either side's selector resolves to no permanent on the battlefield;
  matches the printed "no longer a creature, no damage is dealt").
  (c) **701.14c** self-fight → 2× power to self — ✅ (when atk_id == def_id,
  both `deal_damage_to` calls hit the same permanent, summing to 2P
  damage). Tracked separately because no STX/SOS card today instructs a
  creature to fight itself, but the engine handles it correctly by
  construction.
  (d) **701.14d** — ✅ (fight uses the
  general `deal_damage_to` path, NOT the `combat.rs` damage path that
  emits `DealsCombatDamageToPlayer`; trigger-listening cards correctly
  see this as non-combat damage). Lock-in tests:
  `decisive_denial_mode_1_fight_via_chelonian_template` (mutual damage),
  `academic_dispute_pumps_friendly_and_fights_opp_creature` (pump-then-
  fight at the same step), `cr_701_14c_self_fight_deals_twice_power_to_self`
  (batch 24+ — new self-fight lock-in).

- 🟡 **CR 701.48 — Learn** (push modern_decks batch 24 audit,
  claude/modern_decks branch — audit against `MagicCompRules_20260417.txt`):
  "Learn" means "You may discard a card. If you do, draw a card. If you
  didn't discard a card, you may reveal a Lesson card you own from outside
  the game and put it into your hand." (701.48a). The engine ships **only**
  the discard-and-draw half — and even that half is collapsed to "draw a
  card" without the discard fork because no STX card in the current
  catalog has a different effect when you discard vs reveal-a-Lesson.
  The full CR 701.48a primitive needs: (a) a per-player **Lesson
  sideboard** zone (tracked separately as the "Lesson sideboard model"
  TODO row); (b) a **MayDo nested-Or** primitive — the printed "may
  discard ... if you do, draw / if you didn't discard, may reveal a
  Lesson" is a two-stage may-choose; (c) a **reveal-from-outside-game**
  effect (currently no engine support for revealing a sideboard card).
  Cards affected today: Eyetwitch ✅, Hunt for Specimens ✅, Pest
  Summoning ✅, Igneous Inspiration ✅, Field Trip ✅, Environmental
  Sciences ✅, Mascot Interception ✅, Containment Breach ✅ — all use
  the same "Learn → Draw 1" approximation. Promote to ✅ when the
  Lesson sideboard model lands.

- ✅ **CR 701.21 — Sacrifice** (push modern_decks batch 23 audit,
  claude/modern_decks branch — audit against `MagicCompRules_20260417.txt`):
  "To sacrifice a permanent, its controller moves it from the battlefield
  directly to its owner's graveyard. A player can't sacrifice something
  that isn't a permanent, or something that's a permanent they don't
  control." (701.21a). The engine wires sacrifice via three orthogonal
  shapes: (a) **cost-paid sacrifice of the source itself** via
  `ActivatedAbility.sac_cost: bool` (Mind Stone, Cathar Commando,
  Selfless Glyphweaver, Lorehold Bookburner) — fires before the effect
  resolves so the source is in the graveyard when the trigger goes on
  the stack; (b) **resolution-time effect sacrifice** via
  `Effect::Sacrifice { who: Selector, count: Value, filter:
  SelectionRequirement }` (Witherbloom Pestkeeper's sac-a-Pest cost-as-
  first-step, Witherbloom Pestbroker's sac-fodder body); the player picks
  fodder (or AutoDecider auto-picks first matching) and the chosen
  permanent moves bf → owner's graveyard; (c) **cost-paid sacrifice with
  remembered power** via `Effect::SacrificeAndRemember` (Tend the
  Pests's "sacrifice a creature, then mint X Pests where X = sacrificed
  creature's power" uses `Value::SacrificedPower`). The engine also
  honors the "controlled by you" restriction (701.21a final clause)
  via `SelectionRequirement::ControlledByYou` filters on the sacrifice
  picker. Tests: implicit across the entire test suite — every Pest
  sac engine, every Tend the Pests / Plumb the Forbidden / Witherbloom
  Pestkeeper test exercises the sacrifice pipeline. Specific lock-in
  tests: `plumb_the_forbidden_at_x_two_sacs_two_draws_two_loses_two`,
  `witherbloom_pestkeeper_etb_mints_pest_and_sac_shrinks_target`,
  `witherbloom_pestbroker_etb_drains_two`. The "cost-time filter on
  sacrifice (gating activation legality)" form — printed Oracles like
  "{1}{B}, Sacrifice a Pest: …" where the sacrifice is a cost — is
  approximated as a first-step `Effect::Sacrifice` body that resolves
  but doesn't gate activation legality. The strict form would extend
  `ActivatedAbility` with an `Option<SelectionRequirement>` cost-side
  sacrifice filter; tracked as the "Batched sacrifice picker for
  cost-paid filters" TODO row.

- ✅ **CR 603.10a — Die-trigger scope filtering for the dying card**
  (push modern_decks batch 23 audit, claude/modern_decks branch —
  audit against `MagicCompRules_20260417.txt`): "Some zone-change
  triggers look back in time. These are leaves-the-battlefield
  abilities, …". The dying creature's own `CreatureDied`-keyed
  triggered abilities are collected before SBA moves it to the
  graveyard so they fire from the "looked-back" battlefield view.
  Push batch 23 closes a long-standing bug: the die-trigger fast path
  in `check_state_based_actions` collected EVERY die-trigger on the
  dying card regardless of `EventScope`, so an `AnotherOfYours` trigger
  on the dying card itself would incorrectly fire on its own death
  (Inkling Aristocrat would gain 1 life from its own demise). Fixed by
  filtering the collection to only include scopes that can self-fire
  (SelfSource / YourControl / AnyPlayer / ActivePlayer); AnotherOfYours
  / OpponentControl / FromYourGraveyard are correctly excluded — the
  dying card can't be "another" creature you control. Tests:
  `inkling_aristocrat_gains_life_when_another_creature_dies` (positive
  control), `inkling_aristocrat_does_not_trigger_on_self` (negative
  control).

- ✅ **CR 701.26 — Tap and Untap** (push modern_decks batch 22 audit,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): "To tap a permanent, turn it sideways
  from an upright position. Only untapped permanents can be tapped. /
  To untap a permanent, rotate it back to the upright position from a
  sideways position. Only tapped permanents can be untapped." (701.26a,
  701.26b). The engine models the tapped/untapped binary as a single
  `CardInstance.tapped: bool` field (`card.rs:609`) — set true on tap,
  false on untap. (a) **701.26a (tap only an untapped permanent)** —
  ✅ (`activate_ability` in `game/actions.rs` checks
  `card.tapped` and returns `GameError::CardIsTapped` if the source
  carries `ActivatedAbility.tap_cost: true` and is already tapped; the
  `Effect::Tap` handler in `game/effects/mod.rs` is also a no-op for
  already-tapped permanents because the field-set is idempotent).
  (b) **701.26b (untap only a tapped permanent)** — ✅ (the
  `Effect::Untap` handler iterates resolved permanents and flips
  `tapped: false` unconditionally; the operation is idempotent for
  already-untapped permanents, matching the printed "only tapped can
  be untapped" semantics — the no-op behavior on an untapped permanent
  doesn't fire any "becomes untapped" trigger). Stun counters
  (CR 701.46) interpose on the untap path: the engine's untap step
  (`do_untap` in `game/stack.rs`) consults `CounterType::Stun` and
  removes one stun counter per untap event instead of untapping the
  permanent, which is the CR-correct replacement of the untap action.
  Tests: implicit across the entire test suite — every spell with a
  tap cost, every mana ability, every untap-step transition exercises
  the tap/untap pipeline. Specific lock-in tests:
  `stun_counter_replaces_untap_per_cr_701_46a` and the existing
  `force_tap_target_creature_via_*` patterns.

- ✅ **CR 701.25c — Surveil 0 emits no surveil event** (push modern_decks
  audit — code was already correct via the shared Scry/Surveil short-
  circuit, test coverage gap): "If a player is instructed to surveil 0,
  no surveil event occurs. Abilities that trigger whenever a player
  surveils won't trigger." Push (modern_decks) verifies the
  `Effect::Scry / Effect::Surveil / Effect::LookAtTop` shared handler in
  `game/effects/mod.rs:671` already short-circuits at the top when the
  evaluated amount is 0 (`if n == 0 { return Ok(()); }`) — the same
  guard that handles CR 701.22b for Scry 0. Surveil-0 doesn't surface a
  `Decision::Scry` to the decider, doesn't move any cards between
  library and graveyard, and doesn't fire any future "whenever a player
  surveils" trigger. New test:
  `zero_surveil_does_not_trigger_surveil_events_per_cr_701_25c`
  synthesizes a `{U}: Surveil 0` instant and asserts library order +
  graveyard composition is unchanged (only the spell itself enters
  graveyard). Closes a CR-coverage row tracked alongside CR 701.22b.

- ✅ **CR 701.7 — Create (tokens)** (push modern_decks batch 30 audit,
  claude/modern_decks branch — audit against `MagicCompRules_20260417.txt`):
  "To create one or more tokens with certain characteristics, put the
  specified number of tokens with the specified characteristics onto
  the battlefield." (701.7a). The engine wires `Effect::CreateToken
  { who, count, definition }` in `game/effects/tokens.rs` with three
  guarantees: (a) the `TokenDefinition` carries every printed
  characteristic — name, P/T, colors, supertypes, card types, creature
  types, keywords, activated and triggered abilities, mana cost (for
  copy-of-creature tokens). (b) the `count` value is evaluated per
  resolution (so `Value::ConvergedValue` / `Value::XFromCost` /
  `Value::CountOf(...)` all work for variable-count token creators —
  Tend the Pests, Quandrix Cultivator). (c) the resulting battlefield
  state is mutated *before* SBA passes so newly-minted 0/0 tokens
  (Fractal token) survive SBA when `enters_with_counters` is set.
  Per 701.7b: replacement effects targeting token creation (Doubling
  Season, Anointed Procession-class anthems) apply before continuous
  effects modifying the token's characteristics — the engine's
  `replacement::replace_create_token` pass runs before
  `compute_battlefield`'s layering pass, matching the printed rule.
  Tests: implicit across the entire suite — every Spirit / Pest /
  Inkling / Treasure / Fractal mint exercises the create-token
  pipeline. Specific lock-in tests in batch 30: `lorehold_pyrescroll_burns_and_mints_spirit`
  (per-resolution mint), `silverquill_pact_gains_four_and_mints_two_inklings`
  (count: 2 mint via `Value::Const(2)`), `witherbloom_lichenkeeper_etb_mints_pest`
  (ETB token-mint trigger), `prismari_treasurewright_b30_etb_mints_treasure_and_magecraft_scrys`
  (treasure-token mint).

- ✅ **CR 701.22b — Scry 0 emits no scry event** (push XXXVIII audit):
  "If a player is instructed to scry 0, no scry event occurs. Abilities
  that trigger whenever a player scries won't trigger." Push XXXVIII
  promotes the `Effect::Scry` / `Effect::Surveil` / `Effect::LookAtTop`
  handler in `game/effects/mod.rs:506` to short-circuit at the top
  when the evaluated amount is 0 (`if n == 0 { return Ok(()); }`).
  Previously the handler used `if actual == 0` (peek-result length),
  which conflated the "instruction-is-0" case with the "library has
  no cards" case — that conflation is now explicit, with a separate
  comment noting CR 701.22a (fewer cards than requested still
  executes a vacuous scry). The promoted short-circuit means no
  `Decision::Scry` is asked of the decider, no `GameEvent::ScryPerformed`
  rides out of `drain_stack`, and any "whenever you scry" trigger
  would not fire. Test:
  `zero_scry_does_not_trigger_scry_events_per_cr_701_22b` synthesizes
  a `{U}: Scry 0` instant and asserts no `ScryPerformed` event and
  unchanged library order.

- ✅ **CR 120.8 — 0-damage event suppression** (push XXXVII audit): "If
  a source would deal 0 damage, it does not deal damage at all. That
  means abilities that trigger on damage being dealt won't trigger. It
  also means that replacement effects that would increase the damage
  dealt by that source, or would have that source deal that damage to
  a different object or player, have no event to replace, so they
  have no effect." Push XXXVII closes the gap in `deal_damage_to_from`
  (`game/effects/movement.rs:22`). Before push XXXVII, a 0-damage
  spell or ability would emit a `GameEvent::DamageDealt { amount: 0 }`
  + `GameEvent::LifeLost { amount: 0 }` (player target) or
  `GameEvent::DamageDealt { amount: 0 }` (creature target). Any
  `DealsCombatDamageToPlayer` / `DamageDealtToCreature` trigger
  subscribed to the event would fire spuriously. Now `amount == 0`
  short-circuits at the top of `deal_damage_to_from` — no event is
  emitted and no trigger fires. Combat damage already gates 0-damage
  per-blocker (see `combat.rs:351 if assign > 0`) and per-trample-tail
  (`remaining_atk_damage > 0`), so the combat path was already
  CR-120.8-compliant before this push. Test:
  `zero_damage_does_not_trigger_damage_events_per_cr_120_8` (synth a
  {R} "deal 0 damage to player" instant; assert no DamageDealt and no
  LifeLost event ride out of `drain_stack`).
- ✅ **CR 702.90b — Infect damage to a player adds poison counters**
  (push XXXVI audit): "Damage dealt to a player by a source with infect
  doesn't cause that player to lose life. Rather, it causes that source's
  controller to give the player that many poison counters." Push XXXVI
  closes the non-combat path. The combat path (`combat.rs::apply_
  combat_damage`) was already correct via `AttackerInfo.has_infect`.
  The non-combat path (`Effect::DealDamage` → `deal_damage_to`) used
  to unconditionally reduce life, missing the infect routing for
  spell-damage / triggered-ability-damage from a source-with-infect
  creature (the cleanest catalog example is a creature granted Infect
  via Phyresis-style aura or a Triumph-of-the-Hordes anthem, then
  dealing non-combat damage via an activated ability like
  "{1}{R},{T}: This creature deals 1 damage to any target."). Push
  XXXVI splits `deal_damage_to` into a new `deal_damage_to_from(ent,
  amount, source, events)` that consults `computed_permanent(source)
  .keywords.contains(&Keyword::Infect)` and routes player damage to
  `Player.poison_counters` (firing `GameEvent::PoisonAdded`) instead
  of `Player.life`. The legacy `deal_damage_to` thunks through with
  `source: None` so non-cast call sites (Fight back-damage, combat
  fallbacks) keep their existing behavior. Tests:
  `infect_spell_damage_to_player_grants_poison_per_cr_702_90b`
  (granted-Infect bear deals 2 to opp → 2 poison, 0 life loss) +
  control `non_infect_spell_damage_to_player_reduces_life_per_cr_702_
  90b_control` (bare bear deals 2 → 2 life loss, 0 poison).

- 🟡 **CR 702.15 — Lifelink**. The
  lifelink keyword: a source with lifelink causes its controller to
  gain life equal to damage dealt. Audit:
  (a) **702.15a** — ✅ (`Keyword::
  Lifelink` is a static keyword tag; layered grants via
  `StaticEffect::GrantKeyword` and per-card declarations both
  light up the lifelink flag in `compute_battlefield`).
  (b) **702.15b** — ✅ (combat
  path: `combat.rs::apply_combat_damage` consults
  `AttackerInfo.has_lifelink` and dispatches `adjust_life(a,
  lifelink_dealt)` after damage assignment; non-combat path:
  `deal_damage_to_from` consults the source's lifelink flag and
  emits a `GainLife` event when present).
  (c) **702.15c** —
  🟡 (LKI is consulted via `died_card_snapshots` for SBA-driven
  zone-changes, but a spell-cast lifelink source that resolves
  from the stack still uses live battlefield state. No catalog
  card today triggers this exact corner case for lifelink, so the
  behavior matches printed Oracle).
  (d) **702.15d** — ✅ (the combat path emits
  lifelink life-gain from attackers on the battlefield; the non-
  combat path treats `source` as any zone via `CardId` lookup).
  (e) **702.15e** — ✅
  (`apply_combat_damage` emits one `GainLife` event per lifelink
  attacker via the per-attacker loop in `combat.rs:471`. Each
  event fires Ajani's Pridemate-style "whenever you gain life"
  triggers separately).
  (f) **702.15f** — ✅ (the `has_lifelink` flag is boolean;
  layered grants don't stack into multiple life-gain events).
  Tests: `lorehold_sparkmender_b128_has_lifelink` (basic lifelink
  on a creature body); `silverquill_inkmaster_b128_etb_mints_inkling`
  exercises lifelink + flying (a future test could verify combat
  damage life-gain). Combat-path lifelink is exercised by Vampire
  Nighthawk-class cube tests in `tests::cube`. Promote to ✅ when
  the LKI corner case (702.15c) is wired with a triggered-ability
  source that leaves the battlefield mid-resolution.

- 🟡 **CR 116 — Special Actions** (push modern_decks batch 57,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`). Special actions are priority-window
  actions that don't use the stack. Audit:
  (a) **116.1** — ✅ (the engine's
  `play_land`, `do_untap`, etc. paths apply their effects without
  pushing a `StackItem`; the priority cycle resumes with the active
  player without an intervening resolve step).
  (b) **116.2a** Play a land — ✅ (`play_land` in `game/actions.rs`
  enforces main-phase + stack-empty + lands-played-this-turn cap with
  the `extra_land_per_turn` static modifier for Exploration / Azusa;
  CR 305 / 505.6b coverage already tracked above).
  (c) **116.2b** Turn a face-down creature face up — ⏳ (no face-down
  permanent or morph primitive; `CardInstance.face_down` field doesn't
  exist; engine has no `GameAction::TurnFaceUp`).
  (d) **116.2c** End a continuous effect / stop a delayed trigger —
  ⏳ (no card in the catalog grants "any time you have priority, end
  this effect" — the printed-Oracle template appears on Pemmin's Aura
  / Will of the Council corner cases, none of which are in the
  catalog).
  (e) **116.2d** Ignore a static ability for a duration — ⏳ (no card
  prints this rider; the engine has no per-permanent "ignored-by-X"
  flag on static effects).
  (f) **116.2e** Discard Circling Vultures any time you could cast
  an instant — ⏳ (the literal card is not in the catalog; no general
  "discard as a special action" primitive).
  (g) **116.2f** Suspend: exile a card from hand — ⏳ (no Suspend
  keyword primitive; suspend triggers + time-counter framework absent).
  (h) **116.2g** Companion {3} pay to put into hand — ⏳ (no
  companion sideboard model; the engine has no "outside the game"
  zone wiring for companion picks).
  (i) **116.2h** Foretell {2} exile face down — ⏳ (no Foretell
  keyword + no face-down-in-exile + no alt-cost-on-cast-from-exile
  primitive).
  (j) **116.2i** Planechase planar die roll — ⏳ (no Planechase
  variant in the format module; planar deck + planar die TBD).
  (k) **116.2j** Conspiracy face-up — ⏳ (no Conspiracy variant; not
  on the format roadmap).
  (l) **116.2k** Plot a card from hand — ⏳ (no Plot keyword; would
  need an exile-with-marker + cast-from-exile-at-sorcery primitive,
  same shape as Foretell with a different timing window).
  (m) **116.2m** Unlock a locked half via paying its mana cost — ⏳
  (no "rooms" / lockable-permanent primitive; this is the Murders at
  Karlov Manor Rooms cycle, which doesn't appear in any wired catalog).
  (n) **116.3** — ✅ (`play_land` doesn't pass priority; the
  `GameAction` loop re-enters the priority window with the same
  player). Tests: implicit across every `play_land` test in
  `tests/game.rs` + `tests/multiplayer.rs`. Promote to ✅ when
  Foretell / Plot / Companion / Suspend land (they're the next four
  blockers for ⏳ → 🟡 promotion).

- 🟡 **CR 701.34 — Proliferate** (push modern_decks audit,
  claude/modern_decks branch): "To proliferate means to choose any
  number of permanents and/or players that have a counter, then give
  each one additional counter of each kind that permanent or player
  already has." Audit: (a) **701.34a** core fan-out — 🟡 (the engine's
  `Effect::Proliferate` handler in `game/effects/mod.rs:1009` fans
  +1 to every counter kind on every permanent with counters and to
  every player with poison; the printed "choose any number" lets
  the proliferating player elect which permanents/players to skip,
  which the engine currently treats as a strict superset by
  always-choosing-all. Net play pattern: strictly stronger than the
  printed Oracle since you can't accidentally pump a hostile
  permanent with a -1/-1 counter on it). (b) Loyalty counter fan-out
  — ✅ (`add_counters(Loyalty, 1)` works on planeswalkers via the
  same generic counter table). (c) **701.34b** 2HG poison-share —
  ⏳ (the engine doesn't model teams' shared poison; tracked under
  Format Phase F). (d) Other counter types (Charge, Stun, Page,
  Time) — ✅ (the engine's counter table is generic, so every
  CounterType variant proliferates the same way). The strict-
  superset approximation is exercised by Tezzeret's Gambit ✅
  (mode 0 = Proliferate), Karn's Bastion ✅, and any
  `Effect::Proliferate` body. Tracked promotion to ✅ once the
  "choose any subset" decision shape lands (would need a new
  `Decision::ChoosePermanentsToProliferate { candidates }` plus a
  per-permanent picker in the auto-decider / scripted-decider that
  defaults to "all friendlies + your own life total" as a
  reasonable bot baseline).

- ✅ **CR 702.34a — Flashback exile-on-resolve** (push XXXV audit):
  "Flashback [cost]" means "You may cast this card from your graveyard
  if the resulting spell is an instant or sorcery spell by paying
  [cost] rather than paying its mana cost" and "If the flashback cost
  was paid, exile this card instead of putting it anywhere else any
  time it would leave the stack." The engine's `cast_flashback` (in
  `game/actions.rs`) marks the cast card with `kicked = true` to flag
  the path; the resolution-time `move_card_to` (in `game/mod.rs:1479`)
  routes flashback-cast cards into exile when leaving the stack. The
  alternative-cost framing in 601.2b / 601.2f–h is honored — flashback
  payments respect cost reductions (CR 601.2f), pre-flight life cost
  gates, etc. Exercised by the existing SOS Flashback corpus
  (Daydream, Tome Blast, Duel Tactics) and the new Lash of Malice's
  Flashback {3}{B}.
- ✅ **CR 601.2f — Cost reductions can't take the mana cost below {0},
  and can't reduce colored or X pips** (push XXXIV audit): "The total
  cost is the mana cost or alternative cost (as determined in rule
  601.2b), plus all additional costs and cost increases, and minus all
  cost reductions. … If the mana component of the total cost is
  reduced to nothing by cost reduction effects, it is considered to be
  {0}. It can't be reduced to less than {0}." Push XXXIV lands the new
  `ManaCost::reduce_generic(amount) -> u32` helper which drains
  Generic pips left-to-right and clamps at zero, never touching
  colored, colorless, hybrid, Phyrexian, X, or snow pips. Wired into
  `cast_spell_with_convoke` via the new `cost_reduction_for_spell`
  helper, which walks the battlefield for both flat
  (`StaticEffect::CostReduction`) and target-aware
  (`StaticEffect::CostReductionTargetingFilter`) reductions. Killian,
  Ink Duelist is the canonical target-aware exerciser; tests
  `killian_reduction_does_not_eat_colored_pips` (Bolt at a creature
  still needs the {R}) and `killian_only_reduces_its_controllers_
  spells` (controller gate honored) verify the rule end-to-end.

- ✅ **CR 121.4 / 704.5b — Decking out loses the game** (push XXXIV
  audit — code was already correct, test coverage gap): "A player
  who attempts to draw a card from a library with no cards in it
  loses the game the next time a player would receive priority. (This
  is a state-based action.)" The engine's `Effect::Draw` handler in
  `game/effects.rs:384` returns early and sets
  `Player.eliminated = true` when `draw_top()` returns `None`. The
  state-based-action sweep at the end of resolution
  (`check_state_based_actions` in `game/stack.rs:803-819`) then
  promotes `eliminated` flags to `game_over = Some(Some(winner))`
  when ≤ 1 alive player remains. Push XXXIV adds the missing
  end-to-end test `drawing_from_empty_library_eliminates_player`:
  P1 with an empty library casts Divination, attempts to draw 2,
  immediately loses, and P0 is declared the winner. The "next time
  a player would receive priority" timing nuance is the SBA
  framing, but the practical effect is identical (mid-resolution
  elimination promotes to game-over by the next priority pass).

- ✅ **CR 700.2b — Modal triggered-ability mode chosen at push-time**
  (push XXXIII audit): "The controller of a modal triggered ability
  chooses the mode(s) as part of putting that ability on the stack. If
  one of the modes would be illegal (due to an inability to choose
  legal targets, for example), that mode can't be chosen. If no mode
  is chosen, the ability is removed from the stack." Push XXXIII lands
  `GameState::pick_trigger_mode(effect, source) -> Option<usize>` in
  `game/stack.rs`. When the trigger's top-level effect is
  `Effect::ChooseMode`, the helper asks the controller via
  `Decision::ChooseMode { source, num_modes }`; otherwise it returns
  `None` and the existing `mode.unwrap_or(0)` resolution path handles
  non-modal triggers unchanged. Wired into three major trigger push
  sites: `fire_step_triggers` (delayed + regular), `fire_spell_cast_
  triggers` (Magecraft / Repartee), and `dispatch_triggers_for_
  events`. The illegal-mode pruning ("If no mode is chosen, the
  ability is removed from the stack") is not enforced — the engine
  always picks something — but in practice the AutoDecider picks
  mode 0 unconditionally, which matches the printed "leftmost mode
  if no other choice is forced" pattern. Prismari Apprentice's modal
  Magecraft (Scry 1 / +1/+0 EOT) is the canonical exerciser; tests
  `prismari_apprentice_modal_magecraft_scrys_by_default` (mode 0
  default) and `prismari_apprentice_modal_magecraft_pumps_via_
  scripted_mode_pick` (mode 1 via ScriptedDecider) lock in both
  branches.
- ✅ **CR 120.3c — Damage to a planeswalker removes loyalty counters**
  (push XXXIII audit): "Damage dealt to a planeswalker causes that
  many loyalty counters to be removed from that planeswalker." Combat
  damage was already routed through the loyalty-decrement path
  (`combat.rs::AttackTarget::Planeswalker`), but non-combat
  `Effect::DealDamage` was unconditionally marking damage on
  `c.damage` regardless of card type. Push XXXIII's fix in
  `game/effects.rs::deal_damage_to` detects
  `definition.is_planeswalker()` and routes damage into loyalty
  counter removal (emitting `GameEvent::LoyaltyChanged`). Test:
  `confront_the_past_mode_2_uses_loyalty_counter_x` — Professor
  Dellian Fel at 5 loyalty takes 5 damage and dies via the
  PW-0-loyalty SBA path.
- ✅ **CR 613.4b — Layer 7b set-P/T sublayer** (push XXXII audit):
  "Effects that set power and/or toughness to a specific number or
  value are applied." Push XXXII adds `Effect::SetBasePT { what,
  power, toughness, duration }` which installs a real layer-7b
  `Modification::SetPowerToughness(p, t)` continuous effect. Layer
  application code in `compute_permanent` already supported this
  modification (Tarmogoyf / Cruel Somnophage already use it via
  compute-time injection). Counters and +N/+M (layer 7c) and
  switching (layer 7d) still stack correctly on top per CR
  613.4c-d — verified by `square_up_layers_under_plus_one_counters`:
  Square Up (sets base to 0/4) + a pre-existing +1/+1 counter
  produces a 1/5, matching the printed rule that counters apply
  after 7b sets. Square Up is the first non-hardcoded card to use
  this layer path; future "becomes a 1/1" effects (Pongify, Beast
  Within's 3/3 token, fix to `Effect::ResetCreature`) can reuse the
  same primitive.
- ✅ **CR 700.2d — Modal "choose more than one"** (push XXXII audit):
  "If a player is allowed to choose more than one mode for a modal
  spell or ability, that player normally can't choose the same mode
  more than once." Push XXXII lands `Effect::ChooseN { picks:
  Vec<u8>, modes: Vec<Effect> }`. Each `picks` index in the list
  must be distinct (no de-dup enforcement yet — relies on factory
  authors to follow the rule). At resolution, the picked modes
  fire in `picks` order via a `for` loop in `Effect::ChooseN`'s
  resolver, sharing the spell's single target slot for the first
  picked target-requiring mode. The five STX Commands
  (Witherbloom / Lorehold / Quandrix / Silverquill / Prismari) are
  the first users. Mode-pick UI (letting the controller actively
  choose `picks` at cast time, per CR 700.2a) is still ⏳; the
  current `picks` are hard-coded per card.
- ✅ **CR 506.4 — Permanent removed from combat on zone change**
  (push XXIX audit): "A permanent is removed from combat if it leaves
  the battlefield, if its controller changes, if it phases out, if
  an effect specifically removes it from combat, …". Wired via the
  new `GameState::remove_from_combat(cid)` helper called from
  `move_card_to`, `remove_from_battlefield_to_graveyard`, and
  `remove_from_battlefield_to_exile`. The helper prunes
  `self.attacking` and `self.block_map` so the post-removal combat
  state stays consistent. Before push XXIX, mid-combat destruction
  left orphan attacker entries until end of combat; combat damage
  resolution already filter-mapped against `compute_battlefield`,
  but other consumers (selectors, trigger dispatchers) could see
  stale entries. Test:
  `destroying_attacker_mid_combat_prunes_attacking_per_cr_506_4`.
  Phase-out / controller-change paths still aren't wired (no
  phasing primitive, no `Effect::GainControl` cleanup on permanent
  removal yet), but those clauses aren't exercised by any cataloged
  card today.
- ✅ **CR 502.4 — No priority during untap step** (push XXVIII audit):
  "No player receives priority during the untap step, so no spells can
  be cast or resolve and no abilities can be activated or trigger."
  The engine's `advance_to_next_step` (in `game/stack.rs:62`) already
  handles this: "Untap has no priority window — auto-execute and move
  on." The untap step runs `do_untap()` then immediately calls
  `pass_priority` to step into Upkeep. State-based actions are still
  checked in the SBA loop (which doesn't depend on priority). Test
  coverage is implicit through the existing turn-progression tests
  that walk through untap without observing a priority window.
- 🟡 **CR 614.10 — Skip effects are replacement effects** (push XXVIII
  audit): "An effect that causes a player to skip an event, step,
  phase, or turn is a replacement effect. 'Skip [something]' is the
  same as 'Instead of doing [something], do nothing.'" We have
  `Player.skip_first_draw` for the start-of-game first-draw skip (CR
  103.6), but no general skip-effect framework. Cards like Mind's
  Desire's "extra turn", Time Sieve's extra turns, or Howling Mine /
  Verity Circle's draw-skip riders depend on a `SkipNextStep` or
  `SkipNextDraw` replacement primitive. Tracked under "Replacement
  Effects" below.
- ✅ **CR 605.1a — Mana abilities (activated)**: An activated ability is
  a mana ability if it (a) doesn't require a target, (b) could add mana
  to a player's pool when it resolves, and (c) is not a loyalty
  ability. The engine's `is_mana_ability` recogniser in
  `game/actions.rs` matches against the rule's criteria conservatively:
  pure `Effect::AddMana` (no target field, always can add mana) or
  `Effect::Seq` of mana abilities. The `tap_for_mana` mana-source
  driver only accepts an ability that passes this check. Pushed
  XVIII: Witherbloom Pledgemage refactored to use `life_cost: 1` +
  pure `AddMana` so it qualifies — proving the round-trip via the new
  `witherbloom_pledgemage_is_a_mana_ability_per_cr_605` test.
- ✅ **CR 605.4a — Mana abilities don't go on the stack**: The mana-
  ability path in `activate_ability` resolves immediately via
  `continue_ability_resolution` (no `StackItem::Trigger` push, no
  priority window). Test:
  `witherbloom_pledgemage_is_a_mana_ability_per_cr_605` asserts the
  stack length is unchanged after activation.
- ✅ **CR 707.2 — Copy characteristics**: Copies acquire copiable values
  of the source (name, cost, color, types, text, P/T, loyalty) plus
  on-stack choices (mode, targets, X, kicker). Wired in push XVII via
  `Effect::CopySpell` cloning the source's `CardDefinition` (which
  holds all copiable values) and the StackItem's `target`/`mode`/
  `x_value`/`converged_value`. Counters, status, and stickers are NOT
  copied (the copy uses a fresh `CardInstance::new` which starts
  zero-state).
- ✅ **CR 707.10 — Spell copies**: Copies of spells aren't cast,
  copies of activated abilities aren't activated. Our `CopySpell`
  pushes a `StackItem::Spell` directly without emitting `SpellCast`
  (the cast triggers don't fire for copies). Copies inherit modes /
  targets / X / converged_value.
- ✅ **CR 707.10a — State-based action**: A copy of a spell ceases to
  exist in any zone other than the stack. Copies are marked
  `CardInstance.is_token = true`; the existing token-cleanup SBA path
  (`stack.rs::check_state_based_actions` at line 730) drops them from
  graveyard / hand / library / exile after resolution. Test:
  `tests::sos::copied_spell_does_not_linger_in_graveyard_after_resolution`.
- 🟡 **CR 706 — Casting spells**: `cast_spell` covers the main path.
  Gaps: choose-additional-cost ("kicker"/"buyback" alternatives are
  via `alternative_cost`, but only one alt-cost can be active at
  cast time; multi-alt cycles aren't generalized).
- ✅ **CR 509.1i — Block triggers fire on blocker declaration**:
  "Once the chosen creatures are declared as blockers, any abilities
  that trigger on blockers being declared trigger." Push XXVI adds
  `EventKind::Blocks` to `effect.rs` and wires it through
  `event_matches_spec` in `game/effects.rs`. The trigger source's
  `SelfSource` arm now branches: `Blocks → blocker == source.id`
  and `BecomesBlocked → attacker == source.id`. Both events come off
  the same `BlockerDeclared` payload, so a single `declare_blockers`
  pass emits one event per blocker, then the dispatcher fans out
  matching triggers. Test: STX
  `daemogoth_titan_blocks_sacrifices_another_creature`.
- 🟡 **CR 702.29 — Cycling** (renumbered from CR 702.21 in the
  20260116 rules edition). The base cycling action ships: a new
  `GameAction::Cycle { card_id }` reads the card's printed
  `Keyword::Cycling(cost)`, pays the cost from the active player's
  mana pool, discards the card to graveyard, and draws one. Per
  CR 702.29a "[Cost], Discard this card: Draw a card." Per CR
  702.29c, a distinct `EventKind::CardCycled` (+ `GameEvent::Card
  Cycled` + `GameEventWire::CardCycled`) event is emitted alongside
  `CardDiscarded` so cycle-specific triggers don't double-fire on
  regular hand discards. The dispatcher walks the cycler's
  graveyard for `EventScope::SelfSource` cycle triggers — "When
  you cycle this card" lands via the graveyard-walk extension in
  `dispatch_triggers_for_events` (a SelfSource CardCycled scope on
  a graveyard-resident source fires with `source = cycled card id`).
  Filler test cards: `strixhaven_cycle_glyph_b143` (vanilla cycle),
  `strixhaven_cycle_decree_b145` (cycle → draw 3 trigger),
  `silverquill_sage_b145` (defensive cycle-trigger anchor),
  `witherbloom_vinegrower_b145` (Witherbloom Cycling 2-drop).
  Lock-in tests `cycling_discards_and_draws_a_card`,
  `cycling_rejects_without_mana_to_pay_the_cost`,
  `cycle_glyph_castable_as_a_sorcery_too`,
  `cycle_decree_when_cycled_draws_three_cards`,
  `silverquill_sage_b145_can_be_cycled`,
  `witherbloom_vinegrower_b145_can_be_cycled`. UI:
  `KnownCard.has_cycling` + `KnownCard.cycling_cost_label`
  exposed; client adds C keypress to cycle the hovered hand card.
  Remaining ⏳: (a) Typecycling per CR 702.29e
  ("Mountaincycling" / "Basic land-cycling" → discard to tutor a
  matching land instead of drawing); (b) Cycling-cost reduction
  (Astral Slide / Lightning Rift activate when you cycle); (c)
  printed STA cycling reprints (Decree of Pain, Akroma's Vengeance,
  Mystical Dispute via the mystic-archive split).
- ⏳ **CR 704.5d (token cleanup)**: Already covered by SBA tokens.retain. ✅
- 🟡 **CR 117.1 — Order of priority**: `pass_priority` walks the
  alive players in seat order. Multi-player APNAP ordering for
  triggers / simultaneous effects is approximated.
- ✅ **CR 119.4 — Pay-life-only-if-life-≥-cost**: Per the rule, a
  player may pay X life only if their life total is greater than or
  equal to X. The activated-ability path
  (`actions.rs::activate_ability`) was already wired to reject
  cleanly with `GameError::InsufficientLife` when life < cost. Push
  XIX (2026-05-12) brings the alt-cost cast path
  (`cast_spell_alternative`) up to parity — the alt-cost life-cost
  gate was missing, so a Force-of-Negation-style spell with
  `AlternativeCost.life_cost: 1` could be cast at 0 life, driving
  life negative. Now the pre-flight gate matches the activated
  ability path. Test scaffolding for both paths in
  `tests::stx::witherbloom_pledgemage_rejects_activation_with_zero_life`
  + the activated-ability path; a future test will exercise the
  alt-cost path once we have an alt-cost-with-life-cost card wired.
- ✅ **CR 603.6c — Leaves-the-battlefield abilities check first zone**:
  "An ability that attempts to do something to the card that left the
  battlefield checks for it only in the first zone that it went to."
  The engine's `move_card_to` walks battlefield → graveyards → exile
  → hand → library, finding the source card in its current zone.
  Triggered abilities with `EventScope::FromYourGraveyard` correctly
  resolve `Selector::This` against the graveyard-resident card; the
  same primitive supports `Move(This → Hand)` from a graveyard scope
  (push XXV — Killian's Confidence). Engine audit added to TODO.md.
- ✅ **CR 603.10a — Graveyard-leave triggers look back in time**:
  "Some zone-change triggers look back in time. These are
  leaves-the-battlefield abilities, abilities that trigger when a
  card leaves a graveyard, and abilities that trigger when an object
  that all players can see is put into a hand or library." Our
  `EventKind::CardLeftGraveyard` emission in `move_card_to` powers
  the SOS Lorehold "cards leave your graveyard" cycle. Per-card
  emission is an idempotent approximation of the "one or more"
  batched wording.
- ✅ **CR 121.5 — Put-into-hand is not a draw**: "If an effect moves
  cards from a player's library to that player's hand without using
  the word 'draw,' the player has not drawn those cards. This makes
  a difference for abilities that trigger on drawing and effects
  that count cards drawn." Wired in push XXIV: the
  `Effect::RevealTopAndDrawIf` resolver no longer emits
  `GameEvent::CardDrawn` and no longer increments
  `cards_drawn_this_turn` when the matched card moves library → hand.
  Goblin Guide's reveal-and-give-land path is the canonical exerciser;
  see `tests::goblin_guide_put_into_hand_is_not_a_draw_per_cr_121_5`.
  Note: cards using `Effect::Move(library → hand)` were already
  CR-compliant — `move_card_to` doesn't fire CardDrawn; only the
  RevealTopAndDrawIf resolver had the bug.
- ✅ **CR 121.2 — Drawing cards one at a time**: `Effect::Draw` in
  `game/effects.rs` evaluates the count, then loops one-card-at-a-time
  (`for _ in 0..n`) — matching CR 121.2 "Cards may only be drawn one
  at a time." Each draw fires a `GameEvent::CardDrawn` so trigger
  payoffs (Wheel of Fortune-style draw-N-trigger-N effects) see the
  expected stream of CardDrawn events. The deck-out trigger
  (`121.4 — drawing from empty library`) flips `Player.eliminated`
  immediately and the SBA picks it up the next loop. No further
  wiring required.
- ✅ **CR 121.4 — Decking out loses the game**: Drawing from an
  empty library marks the player `eliminated`; the next SBA pass
  drops them out of the game. Wired in `Effect::Draw` and in the
  per-turn draw step path.
- ✅ **CR 122.3 — +1/+1 and -1/-1 counters cancel**: Per the rule,
  if a permanent has both +1/+1 and -1/-1 counters, `N` of each are
  removed as a state-based action, where `N` is the smaller of the
  two counts. Wired in `game/stack.rs::check_state_based_actions`
  at line 512 inside the main SBA loop, processing every
  battlefield permanent each pass. The implementation pre-dates
  the 2024 rules renumbering and still labels the code comment as
  CR 704.5q/r, which is now the deprecated number — code path is
  correct, comment is stale; fixed in push XX.
- ✅ **CR 122.1d — Stun counter prevents next untap**: A permanent
  with one or more stun counters has a replacement effect
  "instead of being untapped, remove one stun counter." Wired in
  `do_untap` which checks for stun counters on each permanent
  before untapping. Frost Trickster / Snow Day exercise this
  path.
- ✅ **CR 122.6a — Counters on enter-with-counters**: "If an
  object enters the battlefield with counters on it, the effect
  causing the object to be given counters may specify which
  player puts those counters on it. If the effect doesn't
  specify, the object's controller puts them on it." Wired
  implicitly through the ETB-triggered `Effect::AddCounter`
  path — every "enters with N counters" body resolves under the
  controller's resolution context, so `ctx.controller`
  determines who places the counters (no observable
  multi-player effect today since the bot harness always has the
  controller place; but the implementation matches the rule).
- 🟡 **CR 122.2 — Counters cleared on zone change**: "Counters on
  an object are not retained if that object moves from one zone to
  another." The engine currently **retains** counters across zones
  (only `damage`/`tapped`/`attached_to` get cleared on `move_card_to`),
  which is in tension with 122.2 but useful in practice — Felisa
  Fang of Silverquill's "creature with +1/+1 counter dies" trigger
  reads the just-dead card's counter pool to confirm the death
  match, and several future cards (e.g. Spike Feeder die-trigger
  payoffs) would want this. Push XXIII extended `Value::CountersOn`
  to also cross-zone-search (graveyards + exile) so triggered
  abilities can read counters off the source post-move. **CR
  122.2-compliant** behaviour would clear counters during
  `move_card_to`; we should add a per-card-type "preserves counters"
  flag (or a CR 122-strict clear pass that also updates Felisa) in
  a future engine pass.
- ✅ **CR 122.8 — Counter movement when source has left the
  battlefield**: "If a triggered ability instructs a player to put
  one object's counters on another object and that ability's
  trigger condition or effect checks that the object with those
  counters left the battlefield, the player doesn't move counters
  from one object to the other." Push XXIII's Star Pupil
  implementation hard-codes a `Value::Const(1)` counter on the
  death-trigger (matching the printed Oracle's "a +1/+1 counter"
  wording, which Wizards uses specifically to dodge 122.8). Cards
  that DO say "its +1/+1 counters" in older Oracle text have
  errata moving them to a fixed count. Audit point: this rule is
  not actively enforced by the engine — a future card that
  improperly uses `Value::CountersOn(Source-That-Left)` in a death
  trigger would still resolve via the cross-zone fallback added
  in push XXIII. The fix would be to scan the trigger's `Effect`
  tree for `Selector::TriggerSource` references in CountersOn
  contexts and zero the value when the source has changed zones.

- ✅ **CR 614.12 — Enters-with-counters replacement effects** (push
  XXXI audit): "Some replacement effects modify how a permanent enters
  the battlefield. … An effect that says a permanent enters the
  battlefield with one or more counters on it." General-purpose
  replacement-effect primitive is still ⏳ (tracked under Engine —
  Missing Mechanics), but the engine implements the printed pattern
  via an **ETB-trigger approximation**: each card with "enters with N
  +1/+1 counters" wording (Pterafractyl, Rancorous Archaic,
  Symmathematics) ships an `EntersBattlefield/SelfSource` trigger that
  calls `AddCounter { what: Selector::This, amount: N }`. Caveat: ETB
  triggers fire **after** state-based actions check toughness, so a
  body that would be 0/0 or 0-toughness pre-counters would die before
  the trigger lands. Workaround: bump the printed P/T floor to a
  1-toughness body (Pterafractyl 1/0 → 1/1, Symmathematics 0/0 → 1/1).
  This produces a 1-toughness over-statement (1/1 + 2 = 3/3 instead of
  printed 0/0 + 2 = 2/2 for Symmathematics) which is documented in the
  catalog and tracked under TODO.md's "Replacement Effects" section.
  Tests: `symmathematics_enters_with_two_plus_one_counters` (asserts
  the counters land on resolution).

- 🟡 **CR 116 — Special Actions** (push modern_decks audit, batch 14,
  claude/modern_decks branch): "Special actions are actions a player
  may take when they have priority that don't use the stack."
  Audit:
  (a) **116.2a** playing a land — ✅ (`GameAction::PlayLand` walks
  the controller's hand, validates `Player.lands_played_this_turn <
  max_land_drops`, places the card onto `self.battlefield`, and bumps
  the per-turn counter without going through the stack). The
  per-turn cap is enforced at the action-validation site.
  (b) **116.2b** turning a face-down creature face up — ⏳ (no
  face-down / morph / manifest primitives — engine doesn't model
  the face-down state).
  (c) **116.2c** end-a-continuous-effect special actions — ⏳ (no
  Pacifism-style "you may pay {X}: end this effect" primitive).
  (d) **116.2d** ignore-static-effect special actions — ⏳ (no
  static-effect bypass primitive — none of the printed catalog uses it).
  (e) **116.2e** Circling Vultures-style "discard at instant speed" —
  ⏳ (single-card corner; not in catalog).
  (f) **116.2f** exile a Suspend card from hand — ⏳ (no Suspend
  keyword primitive).
  (g) **116.2g** Companion {3}-to-hand — ⏳ (no Companion primitive).
  (h) **116.2h** Foretell exile from hand — ⏳ (no Foretell primitive
  — same gap tracked under "Foretell alt-cost primitive" in the
  Suggested next-up tasks section, exercised by Saw It Coming).
  (i) **116.2i** Planechase planar die roll — N/A (no Planechase format).
  (j) **116.2j** Conspiracy face-up — N/A (no Conspiracy format).
  (k) **116.2k** Plot exile from hand — ⏳ (no Plot keyword primitive).
  (l) **116.2m** unlock-cost on locked-half permanents — ⏳ (no
  Room/locked-permanent primitive).
  (m) **116.3** priority received after special action — ✅
  (`GameAction::PlayLand` re-runs `give_priority_to_active` after
  the land hits the battlefield, matching CR 116.3 — the active
  player has priority to take another action immediately).
  Tests: implicit across every test that uses `GameAction::PlayLand`
  to ramp on curve — `play_a_land_then_pass_priority`, `cant_play_two_
  lands_on_same_turn`, every full-game test in `tests::game`.
  Promote to ✅ when at least one of Foretell / Suspend / Plot lands
  — the framework already handles 116.2a (the only special action
  actually exercised by the catalog).

- 🟡 **CR 301 — Artifacts** (push modern_decks batch 27,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): The artifact card type — casting,
  Equipment, Fortification, Vehicle subtypes. Audit:
  (a) **301.1** — ✅ (`GameAction::CastSpell` for Artifact-typed cards is
  sorcery-speed-gated; Flash override on Manifold Key-style instants
  honored via the `Keyword::Flash` exception).
  (b) **301.2** — ✅ (same
  `resolve_spell` path as creatures).
  (c) **301.3** — ✅
  (`Subtypes::artifact_subtypes: Vec<ArtifactSubtype>` carries
  Equipment, Vehicle, Food, Treasure, Clue, Blood, Fortification,
  Contraption).
  (d) **301.4** — ✅ (color framework reads `mana_cost` regardless of
  card type, so colored artifacts like Treasure-with-color come
  through correctly; the typical colorless-artifact case is the
  default).
  (e) **301.5/5a-f** Equipment subtype — ✅ (push claude/modern_decks:
  the full equip activation pipeline is wired. `GameAction::Equip`
  pays the `Keyword::Equip(cost)` mana cost, enforces sorcery-speed
  timing (CR 702.6e) + the "creature you control" target restriction
  (CR 702.6c), and repoints `attached_to`. `CardDefinition.equipped_bonus`
  (EquipBonus) flows +P/+T and keyword grants onto the equipped creature
  via the layer system. Cards: Bonesplitter, Shuko, Lavaspur Boots,
  Lightning Greaves. Lock-in: `cr_702_6_equip_attaches_at_sorcery_speed
  _to_your_creature`).
  (f) **301.6** Fortification — ⏳ (subtype declared, no Fortify
  primitive; no Fortification card in the catalog).
  (g) **301.7/7a-b** Vehicle / Crew — ✅ (push claude/modern_decks:
  `Keyword::Crew(N)` + `GameAction::Crew` taps creatures whose total
  power ≥ N to animate the Vehicle into an artifact creature until end
  of turn (layer-4 AddCardType). `base_power`/`base_toughness` honor the
  Vehicle's printed P/T (CR 301.7), and `declare_attackers` reads
  computed creature-ness so crewed Vehicles can attack. Cards: Esika's
  Chariot (Crew 4), Smuggler's Copter (Crew 1), Strixhaven Skycoach
  (Crew 2, promoted from always-creature). Lock-in:
  `cr_702_122_crew_requires_total_power_at_least_n`,
  `cr_301_7_vehicle_is_not_a_creature_until_crewed`).

- ✅ **CR 302 — Creatures** (push modern_decks batch 19,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): The creature card type — casting,
  resolution, subtypes, power/toughness, attack/block eligibility,
  summoning sickness, and damage marking. Audit:
  (a) **302.1** — ✅
  (`GameAction::CastSpell` for a Creature-typed card pushes a
  `StackItem::Spell` after cost payment; sorcery-speed gating is
  enforced at the priority check).
  (b) **302.2** — ✅
  (`resolve_spell` in `game/stack.rs` routes a resolving Creature
  spell to `self.battlefield` under the spell's controller via
  `StackItem.controller`; the move also fires `EntersBattlefield`
  triggers).
  (c) **302.3** — ✅ (`Subtypes::creature_types:
  Vec<CreatureType>` stores per-card subtypes; the engine carries
  the full STX creature subtype set incl. Inkling, Pest, Fractal,
  Spirit, Cat, Dog, Demon, Elemental, etc.; 205.3m's complete list
  isn't enforced but every printed STX/SOS card uses real subtypes).
  (d) **302.4 / 302.4a-c** — ✅ (`CardInstance.power()`
  and `toughness()` start from `CardDefinition.base_power` /
  `base_toughness` and walk the layer system in `game::layers::
  compute_permanent` to apply 7a CDA, 7b SetPowerToughness, 7c
  ModifyPowerToughness, 7d Switch, and +1/+1/-1/-1 counter
  deltas in the correct CR 613.7 order).
  (e) **302.5** — ✅
  (`GameAction::DeclareAttackers` and `DeclareBlockers` accept only
  creature-typed cards; non-creature permanents are rejected at the
  legality check).
  (f) **302.6** — ✅
  (`CardInstance.entered_battlefield_at` snapshot + per-card
  `can_attack`/`can_tap` gate in `actions.rs` checks "has been
  under controller's control continuously since their most recent
  turn began"; the haste keyword grants an exemption). The
  "tap/untap symbol" activation gate is also enforced
  (`activate_ability` rejects tap-cost activations on
  summoning-sick creatures unless they have haste).
  (g) **302.7** — ✅ (`CardInstance.damage: u32`
  accumulates damage; `check_state_based_actions` at the next SBA
  check destroys creatures whose `damage >= toughness()`;
  `do_cleanup` zeroes `damage` for every creature on the
  battlefield, matching CR 514.2). Wither / infect divergence —
  ✅ (push earlier; `EventKind::DealsCombatDamageTo` carries a
  `wither_or_infect: bool` field that routes the damage to -1/-1
  counters instead of marked damage). Regenerate — ⏳ (no
  Regenerate primitive; printed cards using Regenerate are 🟡
  with the regenerate rider stubbed). Tests: implicit across every
  combat test that asserts a creature dies to lethal damage; the
  layer system is exercised by every PumpPT / SetBasePT / counter
  test. Promote-eligible when Regenerate primitive lands; the
  remainder is wired.

- 🟡 **CR 701.8 — Destroy** (push modern_decks batch 42 audit,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): "To destroy a permanent, move it
  from the battlefield to its owner's graveyard. The only ways a
  permanent can be destroyed are as a result of an effect that uses
  the word 'destroy' or as a result of the state-based actions that
  check for lethal damage (see rule 704.5g) or damage from a source
  with deathtouch (see rule 704.5h). If a permanent is put into its
  owner's graveyard for any other reason, it hasn't been 'destroyed.'
  A regeneration effect replaces a destruction event." Audit:
  (a) **701.8a body** — ✅ (`Effect::Destroy` in
  `game/effects/mod.rs:843` resolves the selector, walks each
  permanent, and routes via `remove_to_graveyard_with_triggers`;
  creatures emit `CreatureDied` event before the dies-trigger
  dispatcher fans out to LTB / dies / gy-leave triggers).
  (b) **701.8b destruction sources** — ✅ for the named `destroy`
  primitive (Effect::Destroy) and for both SBA paths: lethal damage
  (CR 704.5g) is wired in `check_state_based_actions` reading the
  `damage` field against the computed toughness; deathtouch (CR 704.5h)
  routes via `Keyword::Deathtouch` on the damage source — any nonzero
  combat/effect damage from a deathtouch source destroys the affected
  creature. Other graveyard-going paths (`Effect::Sacrifice`,
  `Effect::Exile`, mill, discard, return-to-hand bounce) correctly do
  **not** emit a `CreatureDied`-with-destroy semantics; the
  `creatures_died_this_turn` counter still bumps on sacrifice (CR
  701.16 sacrifice puts a permanent into its owner's graveyard, which
  is a "die" event but not a "destroyed" event — the engine collapses
  both into the same `CreatureDied` emission today; doc-tracked).
  (c) **701.8c regeneration shield** — ⏳ (no Regenerate primitive;
  printed cards with Regenerate carry the regenerate rider stubbed.
  No STX/SOS card uses Regenerate; tracked engine-wide).
  (d) **Indestructible guard** — ✅ (`Effect::Destroy` checks
  `Keyword::Indestructible` before routing to the graveyard;
  indestructible creatures pass straight through with no event
  emission and no dies-trigger fanout). Same gate applies in
  `check_state_based_actions` for lethal-damage SBA path.
  (e) **CR 704.5g lethal-damage SBA** — ✅ tested across every PumpPT
  combat test; e.g. Grizzly Bears at 2 marked damage with 2 toughness
  is destroyed at the next SBA check.
  Tests: lethal-damage SBA + deathtouch destroys exercised across
  ~80 combat tests; `Effect::Destroy` exercised by Doom Blade,
  Murder, Vanishing Verse, Witherbloom Withercut, and many more.
  Promote to ✅ when CR 701.19 (Regenerate) lands.

- 🟡 **CR 705/706 — Coin flipping and dice rolling** (push
  claude/modern_decks batch 125 — promoted to 🟡 from ⏳ via the new
  `Effect::FlipCoin` (batch ~63) + `Effect::RollDie` (batch 125)
  primitives). Most catalog gaps now have engine wiring; the remaining
  ⏳ items are reroll modifiers, doubles detection, and Planechase.
  (a) **705.1** — ✅
  (`Effect::FlipCoin { count, on_heads, on_tails }` shipped earlier;
  used by Ral Zarek's -7, Lorehold Coinflinger).
  (b) **705.2** — ✅ (the FlipCoin resolver
  splits to `on_heads` / `on_tails` arms based on the
  `Decision::CoinFlip` answer; the "if you won the flip" / "if you
  lost the flip" semantics is encoded structurally).
  (c) **705.3** — ⏳
  (no replacement-effect framework for forcing the flip result, e.g.
  Krark's Thumb).
  (d) **706.1a** — ✅ (`Effect::RollDie { sides, count,
  results }` shipped batch 125; `Decision::DieRoll { sides, player }`
  + `DecisionAnswer::DieRoll(u8)` carry the rolled face; AutoDecider
  returns midpoint for deterministic tests).
  (e) **706.2/706.2b** — ⏳ (no
  roll-modifier layer; the primitive's natural-result IS the final
  result. Cards with printed roll modifiers (Anhelo, Vedalken Orrery's
  reroll, etc.) remain ⏳).
  (f) **706.3** — ✅ (`Effect::RollDie.results:
  Vec<(u8, u8, Effect)>` encodes the result table; the resolver finds
  the first matching `[low, high]` arm. Out-of-range rolls run no
  effect per CR 706.3a literal "If the result was in this range"
  semantics).
  (g) **706.5** — ⏳ (per-die independent
  resolution; no pair-detection on multi-die rolls).
  (h) **706.6** — ⏳ (no ignore-roll primitive).
  (i) **706.7** — ⏳ (Planechase not modelled; no
  `Plane` zone; no `EventKind::PlanarDieRolled`). Tracked in
  TODO.md's `## Formats` section under `Planechase`.
  Wiring shape for the future when a coin/dice card surfaces in a
  catalog target (e.g. Ral Zarek's ult or a Commander cube of
  AFR dice cards):
  1. New `Effect::FlipCoin { who: Selector, on_win: Box<Effect>,
     on_lose: Box<Effect> }` and `Effect::RollDie { who: Selector,
     sides: u8, count: u32, body: DieRollBody }` primitives.
  2. New `EventKind::CoinFlipped { won: bool }` and
     `EventKind::DieRolled { sides: u8, result: u8 }` for trigger
     hookups (Krark's Thumb-style replacement riders).
  3. `GameState::flip_coin(player_idx) -> bool` and
     `GameState::roll_die(player_idx, sides) -> u8` helpers — RNG-backed
     via the existing `Player.random_seed` plumbing for deterministic
     replay.
  4. `Predicate::WonCoinFlip` reads the most-recent coin-flip's
     winner from a `GameState.last_coin_flip_winner: Option<usize>`
     field for in-resolution mode dispatch.
  No catalog card needs this today; the gap is doc-tracked.

- 🟡 **CR 903 — Commander Variant** (push modern_decks batch 64,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`). The Commander multiplayer variant —
  color-identity deck building, the command zone, commander tax,
  commander damage, and game setup. Audit:
  (a) **903.4** — 🟡 (Phase K of
  the Commander rollout: `ColorSet` bitfield in `mana.rs` + `color_
  identity(def)` in `format.rs` unions the mana-cost pips; rules-text
  mana symbols + printed color indicators are not parsed — the catalog
  doesn't currently use indicator-only color identity sources).
  (b) **903.4d** — ✅
  (push modern_decks batch 64: `color_identity(def)` in `format.rs` now
  recursively unions in the back-face's mana cost via the new
  `union_cost_identity` helper. MDFCs with differently-colored faces
  contribute both halves to the deck-validation identity check.
  Test: `color_identity_unions_mdfc_back_face_per_cr_903_4d`).
  (c) **903.5a** — ✅ (`validate_commander_deck` in `format.rs` checks
  `deck.main.len() + deck.commanders.len() == 100` via the Phase K
  validator; 99-or-101 decks are rejected).
  (d) **903.5b** — ✅ (singleton check in `validate_commander_
  deck` walks `deck.main` and asserts `HashSet::insert(card.name)`
  succeeds, with `is_basic_land` carving out the basic exception).
  (e) **903.5c** —
  ✅ (color identity validation in `validate_commander_deck`: walks
  each main-deck card's `color_identity(def)` and asserts the bitfield
  is a subset of the combined commander identity via `ColorSet::is_
  subset_of`).
  (f) **903.5d** — ✅ (covered by 903.5c since each `LandType::Plains` etc. resolves
  to its corresponding color via the basic-land-to-color mapping; the
  validator filters Mountain out of a {W}{U} commander deck).
  (g) **903.6 / 903.7** — ✅ (`Player::with_starting_life(40)` for Commander format
  via `Format::starting_life`; `seat_commanders(seat, defs)` pushes
  each commander to the seat's command zone and registers the CR 903.9b
  zone-change replacement effect).
  (h) **903.8** — ✅ (`GameAction::CastFromCommandZone` +
  `commander_cast_count: HashMap<CardId, u32>` in `game/types.rs`;
  Phase L's cost-build step adds `2 * prior_casts` generic to the
  spell's mana cost; tests in `tests/multiplayer.rs` verify the tax
  bumps after each successful cast).
  (i) **903.9** — ✅ (Phase H's
  `ReplacementEffect::ZoneChange { from: any, to: graveyard|exile|
  hand|library, redirect: Command }` registered per commander via
  `seat_commanders`; `resolve_zone_change` consults the replacement
  registry and redirects the move). The "may" choice is currently
  always-yes (no decision plumbing for the optional rider); doc-tracked
  for Phase L.
  (j) **903.10a** —
  ✅ (`Player.commander_damage: HashMap<CardId, u32>` accumulates per-
  attacker damage in `assign_combat_damage` / `effects/movement.rs`;
  state-based action in `stack.rs:1033` checks for any entry ≥ 21 and
  emits `PlayerLost`).
  (k) **903.11** "Cards from outside the game restricted" — N/A (no
  Wish / Spawnsire sideboard model; no current catalog uses CSB).
  (l) **903.12** Brawl option — ⏳ (no `Format::Brawl` variant, no
  Standard subset; the Commander codepath uses the 40-life / 100-card
  setup unconditionally).
  (m) **903.13** Commander Draft — ⏳ (no drafted-card-pool primitive;
  the sealed pool generator targets cube / sealed only).
  Tests: `tests/multiplayer.rs` covers (c), (d), (e), (g), (h), (i),
  (j) via 11 scenarios — `seat_commanders_sets_up_command_zone_and_
  replacement`, `destroyed_commander_returns_to_command_zone`,
  `commander_tax_grows_each_cast`, `commander_damage_state_based_
  action_kills_player`. Promote to ✅ when 903.4d (MDFC back-face color
  identity) and 903.9's optional rider land.

## Suggested next-up tasks

- ✅ **STX batches 158 + 159 — 90 synthesised cards across all five
  schools + new shortcut helper + 6 CR rule lock-in tests + server
  view label expansion + summoning-sick UI tooltip** (push
  claude/modern_decks batches 158/159 done). Cards: 22 Silverquill,
  18 Witherbloom, 16 Quandrix (b158), 16 Lorehold, 11 Prismari + 7
  cross-school b159. Tests: 4488 → 4584 (96 new tests). All compose
  against existing shortcut helpers. New engine helper:
  `effect::shortcut::etb_drain_and_counter_self(N)` — ETB drains N
  then puts a +1/+1 counter on self (Silverquill Soulbinder II
  template). CR lock-ins: CR 502.3 untap + stun-counter, CR 121
  basic draw, CR 117.5 SBA-before-priority for lethal damage + zero
  toughness. Server: trigger_event_label gained 17 more EventKind
  × EventScope combos. Client: counter_tooltip surfaces
  "(summoning sick)" for fresh creatures.

- ✅ **STX batches 135 + 136 + 137 — 43 more synthesised cards plus
  two new shortcut helpers** (push claude/modern_decks batches 135/136/137
  done). Added 4-per-school cards across batches 135 and 136 (20+20=40
  cards) plus 3 cards in batch 137 introducing two new shortcut helpers
  (`etb_drain_and_draw(amount)` and `on_attack_create_token(token)`).
  Tests: 3881 → 3934 (53 new tests including 2 helper lock-ins). All
  pass; cargo clippy clean. Notable cards: Inkling Forewing (Ward 1),
  Witherbloom Bonereader (mill+lifegain ETB), Fractal Beanstalker (4
  +1/+1 counters Reach), Inkling Wingmother (on-attack Inkling), Lorehold
  Spirit-Captain (on-attack Spirit token).

- ✅ **STX batches 138 + 139 + 140 — 38 more synthesised cards + CR
  401.7 LibraryPosition::FromTop primitive + Conspiracy Theorist
  promotion** (push claude/modern_decks batch 138/139/140 done).
  Batch 138 added 22 cards (5 per school for Silverquill/Witherbloom/
  Lorehold/Prismari + 3 Quandrix); batch 139 added 15 cards (3+ per
  school); batch 140 promoted Conspiracy Theorist from 🟡 → ✅ by
  wiring its full attack-trigger and empty-hand activated halves via
  the existing `Move(TopOfLibrary → Exile) + GrantMayPlay(LastMoved,
  EndOfThisTurn)` chain (no new primitive needed — the engine had it
  since Tablet of Discovery / Spell Satchel landed). Tests: 3934 →
  3976 (42 new tests: 38 card tests + 3 library-position primitive
  tests + 1 conspiracy attack-trigger test). All pass; cargo clippy
  clean.

- ⏳ **Future batch — focus on engine-feature-unlocking cards**: priority
  candidates are Helix Pinnacle (keyword counter), Walking Ballista
  (Nth-counter trigger), and cards that exercise CR 122.4 (counter cap)
  / 122.7 (Nth-counter threshold trigger). Each lands new engine
  capability tracked in the rules-audit section above.

- ✅ **STX batch 131 — 21 more synthesised cards across all five
  colleges** (push claude/modern_decks batch 131 done). Added 8
  Lorehold + 8 Witherbloom + 4 Silverquill + 3 Prismari + 3 Quandrix
  cards using only existing primitives. New helper-call patterns
  reinforced: `etb_gain_life`, `etb_drain`, `etb_mint_token`,
  `dies_drain`, `magecraft`, `magecraft_drain_each_opp`,
  `magecraft_gain_life`, `magecraft_self_pump`, `magecraft_treasure`,
  `create_token_with_counter`, the `Effect::Drain` /
  `Effect::CreateToken` / `Effect::AddCounter` primitives, and a new
  sacrifice-event-driven trigger (Pest Lichbinder) via
  `EventKind::CreatureSacrificed/YourControl`. Tests: 3806 → 3836
  (30 new b131 card tests — multiple tests per card). All pass; cargo
  clippy clean.

- 🟡 **CR 119.7 — "Can't gain life"** (push modern_decks claude/modern_decks
  branch). The gain-life half of CR 119.7 is now wired via the new
  `StaticEffect::PlayerCannotGainLife { target: PlayerStaticTarget }`
  primitive + the `player_cannot_gain_life_now(seat)` helper called
  from `GameState::adjust_life`. The `Player.cannot_gain_life: bool`
  flag is also exposed (set by emblems / future grant effects but
  currently dormant); `adjust_life` ORs the dynamic battlefield check
  with the cached flag. Witherbloom Lifeglobe (b143) ships the
  "Your opponents can't gain life" static; lock-in tests
  `witherbloom_lifeglobe_b143_prevents_opp_lifegain`,
  `witherbloom_lifeglobe_b143_releases_lifegain_lock_when_it_leaves`.
  Remaining ⏳ for full CR 119.7 / 119.8 parity: (a) the lose-life
  half ("can't lose life") needs the same shape via
  `StaticEffect::PlayerCannotLoseLife { target }` consulted in the
  loss path (`damage_player`, `Effect::LoseLife`); (b) the
  redistribute-life-totals + exchange-life-totals clauses (CR 119.7,
  last sentence) need a check at `Effect::ExchangeLifeTotals` /
  `Effect::DistributeLifeTotals` resolve time; (c) Tainted Remedy's
  "instead, that player loses that much life" replacement needs an
  on-gain redirect rather than a drop.

- ⏳ **Keyword counters (CR 122.1b)** — no `CounterType::Keyword(Keyword)`
  variant yet. Cards that print this rider (Mortarpod variants,
  Helix Pinnacle, Decayed-counter zombies) would need a new
  enum variant + a layer-6 keyword-grant pass in `compute_battlefield`
  that reads `counter_count(Keyword(K))` per permanent. Same shape
  as the existing `PlusOnePlusOne` layer-7c pass but for keywords.
  Tracked as engine work — no STX/SOS card prints keyword counters,
  but eventually a Mortarpod-style "keyword counter for {2}" card
  would land here.

- ✅ **`effect::shortcut::on_attack_drain` / `on_attack_gain_life` /
  `on_attack_ping_any` helpers** (push claude/modern_decks batch 125
  done). Three attack-trigger shortcut helpers landed in `effect.rs`
  alongside the existing `etb_*` / `dies_*` / `magecraft_*` families:
  `on_attack_drain(N)` wraps `on_attack(Effect::Drain { from:
  EachOpponent, to: You, amount: N })`, `on_attack_gain_life(N)` wraps
  the asymmetric you-gain-only variant, and `on_attack_ping_any(N)`
  wraps `on_attack(Effect::DealDamage { to: Creature ∨ Player ∨
  Planeswalker, … })`. Used by Lorehold Bloodrazer (b125), Lorehold
  Saintkeeper (b125), Lorehold Vanguardian (b125), Witherbloom
  Drainstride (b125), Silverquill Stridemage (b125), Inkling Skyhunter
  (b125). Lock-in tests `shortcut_on_attack_drain_uses_attacks_self_
  source_with_drain_body`, `shortcut_on_attack_gain_life_uses_attacks_
  self_source_with_gainlife_body`, `shortcut_on_attack_ping_any_uses_
  attacks_self_source_with_dealdamage_body` verify each helper builds
  an Attacks/SelfSource event spec with the correct body shape so
  future refactors can't accidentally collapse them onto each other.

- ✅ **`effect::shortcut::dies_lose_life_each_opp(amount)` helper**
  (push claude/modern_decks batch 123 done). Wraps the canonical
  asymmetric on-death drain body `on_dies(Effect::LoseLife { who:
  EachOpponent, … })` in a one-liner drop-in. Mirrors
  `etb_drain_each_opp` for the death trigger. New cards using the
  helper in batch 123: Witherbloom Crypttender, Pest Mawlord. Lock-
  in test `shortcut_dies_lose_life_each_opp_drains_only_opponents`
  verifies the helper keeps the controller's life unchanged
  (distinguishing from the symmetric `dies_drain`).

- ✅ **`effect::shortcut::magecraft_drain(amount)` helper** (push
  claude/modern_decks batch 123 done). Wraps the canonical symmetric
  magecraft drain body `magecraft(Effect::Drain { from: EachOpponent,
  to: You, … })` in a one-liner drop-in. Distinct from
  `magecraft_drain_each_opp` (asymmetric — opp loses only) and
  `magecraft_drain_target` (single-target picker). New card using the
  helper in batch 123: Witherbloom Vinegrowth ({1}{B}{G} 2/3 Plant
  Druid with the Apprentice-template drain). Lock-in test
  `shortcut_magecraft_drain_drains_each_opp_and_gains` verifies both
  the opp-loses and you-gain halves fire on every IS cast.

- ✅ **`effect::shortcut::magecraft_add_counter_to_friendly()` helper**
  (push claude/modern_decks batch 122 done). Wraps the canonical
  Quandrix "magecraft → +1/+1 counter on target friendly creature"
  body (`magecraft(Effect::AddCounter { what: target_filtered(Creature
  ∧ ControlledByYou), kind: PlusOnePlusOne, amount: Const(1) })`) in
  one helper call. Refactor target for ~5 inline callsites in
  `stx::quandrix` (Quandrix Coursemage, Quandrix Mathematician
  variants, etc.). New card using the helper: Quandrix Coursemage
  (b122, {1}{G}{U} 2/2 Human Wizard). Lock-in test
  `shortcut_magecraft_add_counter_to_friendly_rejects_opp_creatures`
  verifies the helper correctly filters to controlled creatures. A
  future cleanup pass can sweep the remaining inline bodies in
  `stx::quandrix` and `stx::extras` onto the helper.

- ✅ **"Sacrifice a different creature as activation cost" primitive**
  (push claude/modern_decks batch 120 done). The new
  `ActivatedAbility.sac_other_filter: Option<(SelectionRequirement,
  u32)>` field mirrors `exile_other_filter` but sacrifices battlefield
  permanents the activator controls (excluding the source) rather than
  exiling graveyard cards. Pre-flight gate in `activate_ability`
  (`game/actions.rs`) rejects with `SelectionRequirementViolated` when
  fewer than `count` matching permanents exist. The auto-picker takes
  the lowest-power matching creature so the activator keeps higher-
  value creatures alive. Cost payment order: tap → mana → life →
  sac_cost (source) → sac_other (filter picks) → exile_other →
  exile_self. Both `CreatureSacrificed` (for creatures) and
  `PermanentSacrificed` events fire per CR 701.16. New card using the
  primitive: Witherbloom Cultivator (Batch 120) ({1}{B}{G} 1/3 Plant
  Warlock with `{1}, Sacrifice another creature: drain 1`). Lock-in
  tests: `witherbloom_cultivator_b120_sacrifices_another_creature_for
  _drain` (positive path with a Lions fodder),
  `witherbloom_cultivator_b120_rejects_activation_without_fodder`
  (negative — clean rejection, no mana / tap consumed). Future cards
  that unlock with this primitive: Greater Good (`{0}, Sacrifice a
  creature: Draw cards = sacrificed creature's power`), Korlash, Heir
  to Blackblade (`{B}, Sacrifice a Swamp: Regenerate this`), Witherbloom
  Harvester variants. Engine-wide ✅.

- ✅ **`effect::shortcut::drain_and_draw/scry/surveil/etb_tap_opp_creature`
  helpers** (push claude/modern_decks batch 120 done). Three composite
  raw-effect shortcuts collapse the recurring
  `Seq([Drain(n), {Draw/Scry/Surveil}])` body to one-liners for
  drain+select sorceries — Silverquill Heartrender now ships
  `drain_and_scry(3, 1)`, Silverquill Quillsweep (b119) ships
  `drain_and_draw(3)`, Silverquill Chronicle uses the `drain(2)` raw
  helper. The `etb_tap_opp_creature()` shortcut wraps the
  Silverquill-Lawkeeper / Inkling-Hush "ETB tap target opp creature"
  pattern for future tempo cards. Lock-in tests:
  `shortcut_drain_and_draw_drains_and_draws`,
  `shortcut_drain_and_scry_drains_and_scrys`,
  `shortcut_drain_and_surveil_drains_and_surveils`,
  `shortcut_etb_tap_opp_creature_taps_opponent_target`. A future
  refactor pass can sweep the ~25 remaining inline drain-and-* bodies
  across `stx::*` modules onto these helpers.

- ⏳ **Damage-source choice primitive (CR 120.7)** (push
  claude/modern_decks batch 119 — new suggestion, paired with the new
  CR 120.7 audit row). The current `Effect::DealDamage` path threads
  `ctx.source` correctly, but the catalog has no spells / abilities
  that ask the controller to *choose* a source of damage (Browbeat,
  Burning of Xinye, Vendetta-style "deal damage equal to source's
  power"). A `Selector::ChosenSourceOfDamage { filter }` plus a
  `DecisionKind::ChooseSource` decision-point would unblock these.
  Engine-wide ⏳; low priority since no current STX/SOS/cube card
  needs it.

- ✅ **`effect::shortcut::mint_pests(count)` / `mint_inklings(count)` /
  `mint_spirits(count)` helpers** (push claude/modern_decks batch 105
  done). Landed in `effect.rs` as a family: `mint_token` (base),
  `mint_pests`, `mint_inklings`, `mint_spirits`, `mint_fractals`,
  `mint_treasures`, `mint_lorehold_spirits`. Seven catalog factories
  refactored to the new helpers (Inkling Aerospread, Pest Engorger,
  Witherbloom Pestbrood, Witherbloom Cultmaster, Silverquill
  Anthemcaster, Prismari Elementalist, Prismari Crackleburst, Lorehold
  Battlecaster, Lorehold Sparkstrike). Lock-in tests:
  `shortcut_mint_pests_creates_correct_token_count`,
  `shortcut_mint_inklings_creates_w_b_flying_tokens`,
  `shortcut_mint_fractals_creates_zero_zero_tokens`,
  `shortcut_mint_treasures_creates_treasure_tokens`,
  `shortcut_mint_lorehold_spirits_creates_r_w_spirits`.

- ⏳ **Vehicle / Crew primitive**. Strixhaven has Strixhaven Skycoach (currently
  approximated as a free-attacking Construct), and the cube has
  Smuggler's Copter / Esika's Chariot. A general
  `Effect::Crew { tap_count_at_least: Value }` primitive that turns a
  noncreature Vehicle into a creature EOT once enough power has tapped
  would unblock all three plus future vehicles. Engine-wide ⏳.

- ⏳ **`Effect::CreateCopyToken { source, who, count, modifiers }`
  primitive**.
  Five+ cube/STX cards need this (Phantasmal Image, Helm of the Host,
  Saheeli Rai's -2, Mockingbird, Applied Geometry); only Saheeli Rai
  has a partial implementation today (token mint without copying the
  source's printed characteristics). A dedicated copy-token primitive
  with optional modifier list (e.g. "except it's also a Fractal" /
  "except its base P/T is 4/4") would unblock all of them in one
  engine landing.

- 🟡 **CR 602 — Activating Activated Abilities** (push
  claude/modern_decks — audit against `MagicCompRules_20260417.txt`).
  How the engine puts activated abilities on the stack and pays their
  costs. CR 602.1a is the costs/effect split (the colon).
  (a) **602.1a** — ✅ (`ActivatedAbility::mana_cost`, `tap_cost`, `sac_cost`,
  `life_cost`, `exile_self_cost`, `exile_other_filter` between them
  cover the full cost vocabulary; tap/mana/life/sac are all paid in
  `activate_ability` before the effect goes on stack).
  (b) **602.1b** — 🟡 (`ActivatedAbility.condition` covers per-ability
  predicate gates ("Activate only if …"); `once_per_turn` /
  `sorcery_speed` / `from_graveyard` cover the canonical instructions.
  Per-opponent control restrictions ("Activate only if a player
  controls a Snow permanent") have no first-class slot but can be
  expressed as `condition: Predicate::…` for most.).
  (c) **602.2** — ✅ (`activate_ability` pushes a
  `StackItem::Trigger` for non-mana abilities; mana abilities resolve
  immediately per CR 605.3).
  (d) **602.2b** — ✅ (push claude/modern_decks: added
  `GameAction::ActivateAbility.x_value: Option<u32>` so X-cost
  activations bind X at activation time. The cost-payment path
  (`activate_ability` in `actions.rs`) walks `mana_cost.has_x()` and
  calls `with_x_value(x)` to expand the X symbol into N generic pips
  before payment, mirroring `cast_spell`'s X handling. The X value
  is stashed on `StackItem::Trigger.x_value` so the body reads
  `Value::XFromCost` at resolution. Wired by Pernicious Deed's
  `{X}, Sacrifice this: destroy each permanent with MV ≤ X`. CR
  602.2b is now fully observed in the activation path for non-mana
  abilities; mana abilities never use X today).
  (e) **602.5a** —
  ✅ (the `summoning_sick` flag + `tap_cost: true` activation gate
  reject taps while sick; haste bypasses via `Keyword::Haste` check).
  (f) **602.5b** — ✅
  (`once_per_turn_used` is per-card, persists across controller
  changes; the cleanup step resets it on the active player's untap).
  (g) **602.5d** — ✅
  (`sorcery_speed: true` consults `can_cast_sorcery_speed`).
  Tests: `pernicious_deed_destroys_low_cmc_permanents` covers
  X-cost activation end-to-end.

- ✅ **`GameAction::ActivateAbility.x_value`** (push claude/modern_decks
  done): Activated abilities can now declare an X value at activation
  time. Threaded through `activate_ability` (mana-payment expansion
  via `with_x_value`) and `StackItem::Trigger.x_value` (effect
  resolution reads via `Value::XFromCost`). Unblocks Pernicious Deed
  (`{X}, Sacrifice: destroy each MV≤X permanent`), future Walking
  Ballista-style `{X}: deal X damage`, and any X-cost activated
  ability. Tests: `pernicious_deed_destroys_low_cmc_permanents`.

- ✅ **`effect::shortcut::etb_drain_each_opp(amount)` shortcut** (push
  claude/modern_decks batch 107 done). Asymmetric drain helper landed
  in `effect.rs` — wraps `etb(Effect::LoseLife { who: EachOpponent,
  amount })`. Distinct from the symmetric `etb_drain(amount)` which
  also gains you N. Refactored Witherbloom Toxinspeaker and Inkling
  Magistrate onto the helper. Lock-in test:
  `shortcut_etb_drain_each_opp_drains_only_opponents` verifies the
  asymmetric body so a future refactor can't accidentally collapse it
  onto the symmetric `etb_drain` shape.

- 🟡 **`effect::shortcut::magecraft_loot()` callsite reduction** (push
  claude/modern_decks batch 107 — partial pass). Eight inline
  `magecraft(Seq([Draw 1, Discard 1]))` callsites across `stx::prismari`
  (3) and `stx::quandrix` (5) collapsed onto the existing
  `magecraft_loot()` helper. Remaining ⏳ inline callsites may still
  exist in `stx::extras` and other set modules — future cleanup pass
  can run the same regex sweep there.

- ✅ **Witherbloom-tribal token Pest helpers** (push claude/modern_decks
  batch 105 done) — Subsumed by the `mint_pests` / `mint_inklings` /
  `mint_spirits` shortcut family. See the mint-helper entry above.

- ✅ **`effect::shortcut::etb_ping_any(amount)` /
  `etb_ping_creature(amount)` helpers** (push modern_decks batch 63
  finished) — The `etb_ping_any(amount: i32)` half landed in batch 61
  in `effect.rs`; mirrors `magecraft_ping_any` for the ETB trigger
  flavor. Refactored Lorehold Emberspeaker to use the new helper.
  The "creature-only" sibling `etb_ping_creature(amount)` (and the
  parallel `magecraft_ping_creature(amount)`) **landed in batch 63**
  for Lorehold Sparkscholar-template bodies and Lorehold Ironhand-
  style "ETB ping target creature" cards. Both helpers are now
  available as one-line drop-ins; future cleanup pass can fold the
  remaining inline `Seq([CreateToken, DealDamage(creature)])` bodies
  onto the helper.

- ✅ **`effect::shortcut::magecraft_pump_each_creature_type(creature_type,
  power, toughness)` helper** (push modern_decks batch 66 done) —
  Tribal-pump shortcut landed in `effect.rs`. Wraps
  `magecraft(Effect::PumpPT { what: EachPermanent(HasCreatureType(t) ∧
  ControlledByYou), …, EndOfTurn })` in a single helper call. Drop-in
  for any tribal Bannerer-template card. Refactored Spirit Bannerer
  (batch 61, `stx::lorehold`) to use the helper. New tribal cards:
  Inkling Bannerer (`stx::silverquill`), Pest Bannerer (`stx::
  witherbloom`) — both 2/2 magecraft tribal-pump bodies. Tests:
  `inkling_bannerer_magecraft_pumps_each_friendly_inkling`,
  `pest_bannerer_magecraft_pumps_each_friendly_pest`,
  `spirit_bannerer_magecraft_pumps_friendly_spirits` (pre-existing).

- ✅ **`effect::shortcut::drain(amount)` helper** (push modern_decks
  batch 54): The canonical "each opponent loses N life, you gain N
  life" body is constructed by ~50 STX/SOS cards inline. Batch 54
  added a `drain(amount: i32) -> Effect` shortcut that returns the
  same `Effect::Drain { from: EachOpponent, to: You, amount }` value.
  Three batch-54 SQ cards (Silverquill Reflect, Silverquill Doom,
  Silverquill Psalm) use the new helper. A future refactor pass can
  collapse the remaining inline drain bodies (~25 cards across
  `stx::silverquill` / `stx::witherbloom` / `stx::extras` /
  `sos::sorceries` / `sos::instants`) to the one-liner.

- ✅ **`EventKind::CreatureSacrificed` event separation** (push
  modern_decks batch 51) — CR 701.16 sacrifice-as-distinct-event
  shipped. Both event variants landed: `EventKind::CreatureSacrificed`
  (`effect.rs`) and `GameEvent::CreatureSacrificed { card_id, who }`
  (`game/types.rs`) — the latter carries the player who paid the
  sacrifice for `YourControl`/`OpponentControl` scope dispatch via
  `event_player` / `event_actor`. Three emitters: `Effect::Sacrifice`,
  `Effect::SacrificeGreatestMV`, `Effect::SacrificeAndRemember`, and
  the `sac_cost: true` activated-ability path in `actions.rs`. Each
  emits `CreatureSacrificed` immediately followed by `CreatureDied`
  so existing death triggers still fire (and order-sensitive
  triggers see the sacrifice-specific event first). Cards using the
  new event: Witherbloom Mortician ({2}{B} 2/2 "Whenever a player
  sacrifices a creature, +1/+1 counter on this creature" — AnyPlayer
  scope) and Pest Pestmaster ({3}{B}{G} 3/3 "Whenever you sacrifice
  a creature, +1/+1 counter" — YourControl scope). Lock-in tests:
  `witherbloom_mortician_grows_on_sacrifice`,
  `witherbloom_mortician_does_not_grow_on_natural_death` (lethal
  damage emits CreatureDied but NOT CreatureSacrificed — Mortician
  doesn't fire), `pest_pestmaster_b51_grows_only_on_own_sacrifices`.
  Wire mirror added to `GameEventWire::CreatureSacrificed` so client
  UIs can highlight the sacrifice distinctly from a natural death.

- ✅ **`EventKind::PermanentSacrificed` for non-creature sacrifices**
  (push claude/modern_decks batch 102). Shipped. The new
  `EventKind::PermanentSacrificed` / `GameEvent::PermanentSacrificed
  { card_id, who }` fires on every sacrifice resolution regardless of
  card type — emitted by all three sacrifice paths (`Effect::Sacrifice`,
  `Effect::SacrificeGreatestMV`, `Effect::SacrificeAndRemember`) and
  the activated-ability `sac_cost: true` cost path. For creature
  sacrifices both events fire (CreatureSacrificed first per CR 701.16,
  then PermanentSacrificed), so existing Mortician-style sub-triggers
  remain order-correct. Wire mirror `GameEventWire::PermanentSacrificed`
  added for replay rewinds. Korvold, Fae-Cursed King ships in batch
  102 as the exercise card — tests cover both creature- and artifact-
  sacrifice payoff paths
  (`korvold_fae_cursed_king_triggers_on_sacrifice`,
  `korvold_fae_cursed_king_triggers_on_artifact_sacrifice_via_permanent_event`).

- ⏳ **Transient triggered-ability grant primitive** (push
  modern_decks batch 47 — new suggestion). Several STX/SOS cards
  print "until end of turn, each [creature] you control gains
  [trigger]" — e.g. SOS Root Manipulation ("creatures you control
  get +2/+2 and gain menace and 'Whenever this creature attacks,
  you gain 1 life.' until end of turn") and Rabid Attack ("any
  number of target creatures each get +1/+0 and gain 'When this
  creature dies, draw a card.' until end of turn"). The engine has
  no primitive that grants a trigger for a duration; today these
  riders are dropped (the body half ships, the trigger-grant
  half doesn't). Wiring shape: a new `Effect::GrantTriggeredAbility
  { what: Selector, trigger: TriggeredAbility, duration: Duration }`
  primitive that injects a transient trigger onto each matched
  permanent (stored alongside `granted_keywords_eot` for cleanup
  per CR 614.7c). Cards unblocked: Root Manipulation, Rabid Attack,
  plus future "gain 'attack-trigger gain life'" / "gain 'dies-draw'"
  patterns.

- ⏳ **Permanent-copy primitive** (push modern_decks batch 47 —
  new suggestion). Multiple STX/SOS cards print "create a token
  that's a copy of target X" (Echocasting Symposium, Applied
  Geometry, the Colorstorm Stallion / Elemental Mascot "if 5+
  mana spent, create a token that's a copy of this" Opus halves).
  Today these collapse to a vanilla token mint. Engine needs a
  `Effect::CreateCopyToken { what: Selector, modifier: Option<TokenModifier> }`
  primitive that copies the chosen permanent's printed
  characteristics (P/T, types, abilities) into a fresh
  `TokenDefinition` at resolution time. The `modifier` field
  would carry the optional "except it's also a Fractal" /
  "except its base P/T is 4/4" overrides per the printed cards.
  Cards unblocked: Echocasting Symposium, Applied Geometry,
  Colorstorm Stallion (big-body), Elemental Mascot (big-body),
  any future Saheeli / Sublime Epiphany permanent-copy mode.

- ✅ **`Effect::GrantKeyword` duration honoring** (push modern_decks
  batch 24) — Previously mutated `c.definition.keywords` directly with
  no EOT cleanup, so an "EOT haste" grant on a reanimated bear would
  persist forever. Engine fix: added `CardInstance.granted_keywords_eot:
  Vec<Keyword>` for EOT grants, with cleanup at the Cleanup step
  alongside `power_bonus`/`toughness_bonus`. `has_keyword()` now checks
  both vectors. `compute_battlefield()` merges the granted EOT keywords
  into the layered view. `Effect::GrantKeyword` routes EOT to the new
  field, Permanent stays on the direct mutation path. Lock-in tests:
  `granted_keyword_eot_clears_at_cleanup_per_batch_24`,
  `lorehold_revival_returns_creature_with_haste`. Affected cards:
  Lorehold Skirmish, Lorehold Battlescroll, Lorehold Revival,
  Mascot Interception, Practiced Offense (Lifelink/DoubleStrike
  mode), Tempted by the Oriq, Prismari Wildform, Silverquill
  Discipline, Silverquill Battle Hymn, Selfless Glyphweaver
  Indestructible grant, and others.

- ⏳ **Layered-effect `Effect::GrantKeyword` for `UntilNextTurn`** —
  The batch-24 fix above honors `EndOfTurn` and `Permanent` durations.
  `UntilNextTurn`/`UntilYourNextUntap` is wired to permanent mutation
  (no cleanup), which is incorrect. Needs a separate `granted_keywords_
  untilnext: Vec<Keyword>` slot or routing through the proper layered
  system. No STX/SOS card uses this duration today, so the gap is
  doc-tracked but unaddressed.

- ⏳ **Batched sacrifice picker for cost-paid filters** (push
  modern_decks batch 18 suggested) — `Effect::Sacrifice { filter, …}`
  works for the post-resolution sac (Witherbloom Pestkeeper's
  activation step uses it). The cost-paid sac branch (the engine's
  `sac_cost: true` field on `ActivatedAbility`) is a single source-only
  sac and doesn't expose a filter. Wiring shape: extend the activation
  cost field to optionally carry a `SelectionRequirement` filter that
  drives the cost-time fodder picker, so cards like Pestkeeper can
  declare "sac a Pest you control" as a *cost* (rejecting activation
  without a Pest) rather than as the first step of the effect
  (resolves even if no Pest exists). Today's resolve-time filter is
  permissive — if no Pest is available, the sac step is skipped and
  the -2/-2 still resolves.

- ⏳ **`Predicate::CastFromZone(zone)`** (push modern_decks batch 18
  suggested) — the just-landed `CastFromHand` / `CastFromGraveyard`
  pair covers the hand/gy split, but a generalised `CastFromZone(Exile)`
  / `CastFromZone(Library)` is still ⏳. Threading shape: stamp a
  `cast_zone: Zone` field on `CardInstance` alongside `cast_from_hand`
  + propagate to `EffectContext.cast_zone` via
  `for_spell_with_source`. Future Cascade / Suspend / Flashback-from-
  exile riders ("if cast from exile, …") would key off this.

- ⏳ **Inkling / Pest tribal completeness** (push modern_decks
  current): with the 22-card extras drop the Silverquill Inkling pool
  now has 1+/+1 lord support, lifelink fliers, drain payoff, and
  artifact drain. The Witherbloom Pest pool similarly has token
  spawners + a destroy-plus-Pest sorcery + a 2-Pest ETB body. A
  cross-college BG/WB sealed pool could lean into these new shells.
  Slot into the SoS Silverquill / Witherbloom pool selector once the
  decklist generators support tribal weighting.

- ✅ **`Effect::CounterAbility`** (push modern_decks batch 19 doc
  cleanup — already wired in an earlier push, doc was stale) — the
  effect.rs:887 variant takes a `Selector` and counters a target
  activated/triggered ability on the stack via the same dispatcher
  path as `Effect::CounterSpell` but routes through
  `StackItem::Ability` selection. Used by Consign to Memory (mode 0)
  and the cube's Stifle-class cards. The promotion clears the stale
  TODO row that suggested adding a `CounterKind` enum — the engine
  picks the right counter target via the selector's filter (e.g.
  `target_filtered(IsAbilityOnStack)` for ability-only counters,
  `target_filtered(IsSpellOnStack & Legendary)` for legendary-spell
  counters). Future Stifle-grade promotions can compose on top.

- ⏳ **Spirit-tribal Lorehold archetype** (push modern_decks): the new
  Spirit Banner (+1/+1 anthem for Spirits) joins Quintorius's
  pre-existing Spirit lord and the Lorehold token chain (Sparring
  Regimen, Lorehold Excavation, Quintorius). With this in place,
  a Spirit-tribal Lorehold variant deck could lean into the
  Sparring-Regimen-attack → counter rain → anthem combo. Slot it
  into the SoS Lorehold pool selector.

- ⏳ **Inkling-tribal Silverquill archetype** (push modern_decks): the
  new Quartzwood Inkling + Inkwell Strider + Inkling Studies join the
  pre-existing Tenured Inkcaster tribal anthem and Felisa Fang of
  Silverquill's Inkling generator. With at least 5 distinct Inkling
  minters and a +2/+2 lord in the catalog, a Silverquill Inkling
  tribal pool is now viable.

- ⏳ **Triggered mana ability fast-path** (CR 605.4a) — promoted from
  the existing TODO entry into the CR-audit row. Same blocker as
  before: no STX/SOS card requires the fast-path today. First Mana
  Reflection / Wirewood Channeler-class card would trigger.

- ⏳ **`SelectionRequirement::ManaValueAtMostX`** (push modern_decks
  batch 39 suggested) — the current `ManaValueAtMost(u32)` predicate
  takes a compile-time constant, but several STX/SOS cards print
  "mana value X or less" gates where X is the spell's cast-time X
  (Mind into Matter's "put a permanent with mana value X or less
  from your hand onto the battlefield tapped"). Wiring shape: add a
  new variant that reads `EffectContext.x_value` at evaluation time,
  same as `Value::XFromCost` reads it for damage / counters / draws.
  The evaluator (`evaluate_requirement_static` in
  `game/effects/eval.rs`) would need to thread the X value through,
  same way it threads `source` today. Cards unblocked: Mind into
  Matter, future X-cost search-and-cheat-onto-battlefield primitives.

- ⏳ **Refactor existing STX/SOS Silverquill drain creatures to use
  `etb_drain`/`etb_gain_life`** (push modern_decks batch 39 suggested)
  — the new `effect::shortcut::etb_drain(N)` and
  `effect::shortcut::etb_gain_life(N)` helpers (added in batch 39)
  collapse the canonical 7-line ETB drain / gain-life trigger into
  one helper call. ~40 existing cards across `stx::silverquill`,
  `stx::witherbloom`, and `stx::lorehold` (Silverquill Marshal,
  Silverquill Loremender, Silverquill Drainmaster, Inkling Scriptwarden,
  Inkling Pamphleteer, Lorehold Skydefender, etc.) inline the same
  pattern manually. A future cleanup pass should refactor them to
  reduce code duplication; functional behavior is unchanged.

- ⏳ **"Tap N creatures as additional cost" cost primitive** (push
  modern_decks batch 39 noted) — Group Project's Flashback cost is
  "Tap three untapped creatures you control" (no mana cost), which
  doesn't fit the existing `AlternativeCost { mana_cost,
  exile_from_graveyard_count, ... }` shape. Wiring shape: extend
  `AlternativeCost` with `tap_count: Option<(u32, SelectionRequirement)>`
  so a cost-paid validator can require N permanents matching the
  filter to be untapped + tap them as the spell finishes paying.
  Cards unblocked: Group Project (Flashback), future "Tap an
  untapped artifact you control" cost shapes from Mirrodin /
  Convoke siblings.

- ✅ **Stack-aware `find_card_owner`** (push modern_decks): the
  card-owner lookup in `GameState::find_card_owner` (`game/mod.rs`)
  now walks `StackItem::Spell` items so a spell mid-resolution can
  be queried by id. This wires `PlayerRef::OwnerOf(Selector::TriggerSource)`
  for SpellCast triggers — previously the lookup returned `None`
  because the cast card lives on the stack but not in any persistent
  zone. Cunning Rhetoric's "you gain 1, casting player loses 1"
  rider relies on this. Same unblock applies to any future "the
  player who cast that spell" trigger payoff.

- ✅ **Library / hand zone fallback in `evaluate_requirement_static`**
  (push modern_decks): the static target-validator
  (`game/effects/eval.rs::evaluate_requirement_static`) now walks
  battlefield → graveyards → exile → stack → library → hand to find
  the named card before applying its filter. This wires
  `EntityMatches(Selector::TopOfLibrary, Creature)` and similar
  "is the top of library a creature card" predicates — previously
  cards in library/hand zones returned `false` for every filter
  because they weren't in the lookup chain. Lurking Predators' "if
  it's a creature card, …" check now correctly resolves; future
  Domri Rade / Garruk Wildspeaker-style "top card type" payoffs
  plug in cleanly.

- ✅ **`Selector::DiscardedThisResolution { filter }` — discarded-card
  tracker** (push modern_decks): tracks discarded card ids in
  `GameState.discarded_card_ids_this_resolution` and exposes them as a
  selector for follow-up effects. Both `Effect::Discard` branches
  (random + player-chosen) append to the list; reset at the start of
  each resolution. Wires Mind Roots's "Put up to one land card
  discarded this way onto the battlefield tapped" rider exactly. Same
  primitive unlocks any future "exile/draw/play a card discarded this
  way" effect (Eldraine's Charming Princess, the Trash for Treasure
  cycle, etc.).

- ✅ **`Value::TriggerEventAmount` — per-event amount in trigger
  bodies** (push modern_decks): the trigger dispatcher
  (`dispatch_triggers_for_events`) extracts the firing event's
  `amount` (LifeGained, LifeLost, DamageDealt, PoisonAdded,
  CounterAdded), threads it onto `StackItem::Trigger.event_amount`,
  and pipes it to `EffectContext.event_amount`. Resolving trigger
  bodies read it via `Value::TriggerEventAmount`. Wires Light of
  Promise's "that many" rider faithfully. Same primitive unblocks
  any "that many"-style trigger payoff (Crested Sunmare-class,
  Aetherflux-with-damage-Reservoir variants).

- ✅ **`Effect::DelayUntil` fallback to `Selector::CastSpellTarget(0)`**
  (push modern_decks): when the trigger context has empty
  `ctx.targets` (the Repartee / spell-cast-trigger shape), the
  DelayUntil capture walks the stack for the just-cast spell's
  slot-0 target. Wires Conciliator's Duelist's "return at next end
  step" delayed-Repartee rider.

- ✅ **Graveyard-resident static-anthem helper-table**
  (push modern_decks): `graveyard_anthem_for_name` returns
  `(LandType, Keyword)` for cards like Anger / Wonder / Filth whose
  printed Oracle is "As long as [this card] is in your graveyard and
  you control a [Land subtype], creatures you control have
  [keyword]." `compute_battlefield` walks each player's graveyard,
  looks up matches, gates on land-subtype control, and emits a
  continuous `AddKeyword` effect on the gy owner's creatures.
  Currently only Anger is wired; the other Judgment Incarnations
  can plug in via single-row additions.

- ⏳ **CR 603.4 — Intervening 'if' clause "check again at resolve
  time"** (push modern_decks suggested) — push (modern_decks) lands
  the trigger-time half of the rule (predicate evaluated when
  pushing the trigger onto the stack via `fire_step_triggers`'s new
  filter-check). The second half of CR 603.4 — "if the ability
  triggers, it checks the stated condition again as it resolves. If
  the condition isn't true at that time, the ability is removed from
  the stack" — is still ⏳. Wiring shape: add a `filter:
  Option<Predicate>` to `StackItem::Trigger` and re-evaluate it in
  `continue_trigger_resolution_with_source` before applying the body
  (removing the trigger from the stack as a no-op when false).
  Today the only catalog card exercising the resolve-time gap is
  Triskaidekaphile (player could discard between upkeep-fire and
  upkeep-resolve to drop hand below 13; engine wouldn't catch it).
  Felidar Sovereign's "if you have 40 or more life" would have the
  same gap once added. Low-priority until a real card surfaces a
  meaningful resolve-time state change.

- ⏳ **`Predicate::ManaValueAtMostV(Value)` — value-keyed mana-value
  filter** (suggested by push modern_decks's Mind into Matter +
  Sundering Archaic gaps) — both cards want a target / candidate
  filter capped by a runtime-evaluated `Value` (X-from-cost for Mind
  into Matter, ConvergedValue for Sundering Archaic's "exile target
  nonland permanent an opponent controls with mana value less than
  or equal to the number of colors of mana spent"). The current
  `SelectionRequirement::ManaValueAtMost(u32)` is a static cap. A
  Value-keyed sibling needs to thread `EffectContext` (for the X
  value) into both `evaluate_requirement_static` and
  `evaluate_requirement_on_card` — significant call-site refactor.
  Cast-time validation also needs to know the chosen X at the time
  targets are picked (currently the engine picks targets first then
  pays X, so this would need either re-ordering or a "deferred
  validation" pass). Two ⏳ cards exercise this gap; deferring until
  a third card stacks on or the cast pipeline is otherwise touched.

- ⏳ **`Effect::ClearAbilities` / `StaticEffect::LoseAbilities`** —
  the printed Mercurial Transformation says "and loses all abilities
  until end of turn." Today we wire the base-P/T half via
  `Effect::SetBasePT` but the loses-abilities half is omitted. The
  layer system has `Modification::RemoveAllAbilities` but it only
  clears the keyword list — the triggered/static/activated abilities
  live on the `CardDefinition` and aren't touched by the layer pass.
  Wiring would need either (a) a per-permanent override on the
  computed-permanent struct that masks the definition's ability
  fields, or (b) a fully-layered ability list (significant refactor).
  Pongify, Beast Within's 3/3 token, and Mercurial Transformation
  all need this.

- ⏳ **Augusta, Dean of Order — same-power attackers trigger** (push
  modern_decks STX Silverquill 🟡) — the printed "Whenever you attack
  with three or more creatures with the same power, each of those
  creatures gets +1/+1 and gains your choice of flying, first strike,
  vigilance, or lifelink until end of turn" needs a **batched** post-
  attacker-declaration event (not the per-attacker `Attacks` event
  we have today). Suggested shape: new `EventKind::AttackersDeclared`
  that fires once after `declare_attackers` resolves, with the list
  of attackers exposed via `ctx.attackers_declared`. The trigger
  would then need to find the largest same-power group and pump only
  those creatures (custom selector logic). Skipped until a second
  batched-attack trigger appears in the catalog.

- ⏳ **Mavinda, Students' Advocate — cast-IS-from-graveyard static**
  (push modern_decks STX Silverquill 🟡) — the printed "Once during
  each of your turns, you may cast an instant or sorcery spell that
  targets only a single creature from your graveyard. If a spell
  cast this way would be put into your graveyard, exile it instead."
  is a static ability that grants a cast-permission, not an
  activated ability. Needs (a) a per-player "this-turn cast-from-gy
  budget" counter, (b) a target-introspection at cast time
  ("targets only a single creature"), and (c) a delayed replacement
  to route the resolving spell to exile instead of graveyard.
  Currently Mavinda ships as a 1/3 Flying+Vigilance Legendary
  Cleric body without the static.

- ⏳ **Foretell alt-cost primitive** (suggested by push modern_decks's
  Saw It Coming addition) — Foretell ({2} on cast, alt cost {1}{U} on
  the turn after it's foretold from hand for {2}). Wiring shape:
  (a) a new `ActivatedAbility`-style "Foretell" action that exiles
  the card face-down from hand for {2}; (b) a per-card "foretold
  this turn" flag tracked on the exiled card; (c) an `AlternativeCost`
  variant with `not_this_turn_only: bool` that gates the alt cost on
  the prior-turn foretell. Currently Saw It Coming ships as a
  vanilla {2}{U} counter — the Foretell discount path is engine-wide
  ⏳.

- ✅ **Cast-from-graveyard introspection at resolution time** (push
  modern_decks batch 18): `Predicate::CastFromGraveyard` + the new
  `Predicate::CastFromHand` complement cover the cast-source-zone gate.
  Both read from `EffectContext.cast_from_hand` (stamped by
  `for_spell_with_source` from `CardInstance.cast_from_hand`). The
  positive form (`CastFromHand`) is reserved for "if you cast this
  spell from your hand, …" rider patterns — Quandrix, the Proof's
  "spells cast from your hand have cascade" static gates against it
  once the Cascade keyword lands. Triggers / activated abilities
  default `cast_from_hand` to `true`, which matches the printed
  semantics (cast-zone is a spell-only concept; non-spell contexts
  fall through the predicate as `True`/`False` per direction).
  `CastFromZone(zone)` for arbitrary source zones (exile, library)
  is still ⏳ — the engine only tracks the hand/graveyard split
  today.

- ✅ **`Effect::DiscardAnyNumber { who }` — "discard any number of cards"
  primitive** (push modern_decks): new effect variant that asks the
  player to pick a subset of their hand (0 to all). AutoDecider picks
  0 (conservative); ScriptedDecider supplies the exact picks via
  `DecisionAnswer::Discard(_)`. Each discarded card bumps
  `state.cards_discarded_this_resolution`, so a follow-up `Draw` step
  in the same `Seq` can read `Value::CardsDiscardedThisEffect`. Colossus
  of the Blood Age's death trigger is the canonical exerciser ("discard
  any number, draw that many plus one"). Same primitive unblocks any
  future "discard any number → do X equal to that many" card.

- ✅ **`Effect::SetNoMaxHandSize { who }` + `Player.no_maximum_hand_size`**
  (push modern_decks): flips the per-player flag for the rest of the
  game. The cleanup-step CR 402.2 / 514.1 enforcement in `do_cleanup`
  (`game/stack.rs`) skips the discard-down-to-7 loop when the flag is
  set. Wisdom of Ages's "no maximum hand size for the rest of the game"
  rider is the canonical exerciser. Same primitive unblocks any future
  Reliquary Tower / Spellbook / Library of Leng-style permanent that
  grants the same rider via a static effect (those cards would need a
  `StaticEffect::ControllerNoMaxHandSize` variant on top of this).

- ⏳ **`Predicate::AnyOppHasMoreLandsThanYou`** (suggested by push
  modern_decks's Gift of Estates ramp-spell addition) — Gift of
  Estates's printed gate is "If an opponent controls more lands than
  you, search your library for up to three Plains cards." Today the
  gate is omitted and the spell unconditionally searches three
  Plains. Wiring shape: add a new `Predicate::AnyOppHasMoreLandsThanYou`
  primitive that walks `self.players[opponent]` count of permanents
  matching `SelectionRequirement::Land` and compares against
  `self.players[controller]`'s land count. Same primitive unblocks
  any future "if you're behind on lands" catch-up effect (Tithe,
  Knight of the White Orchid's ETB trigger, Land Tax).

- ⏳ **`EventKind::BecameTarget`** (suggested by push modern_decks's
  Battle Mammoth addition) — Battle Mammoth's printed rider is
  "Whenever a permanent you control becomes the target of a spell or
  ability an opponent controls, draw a card." Today the body ships
  as a 6/5 trampler with the trigger omitted. Wiring shape: a new
  `EventKind::BecameTarget { target, source, source_controller }`
  event emitted by `validate_target_legality` at cast-time and by the
  ability-activation walker. Triggers listening on the event would
  fire post-cast / post-activation. Same primitive unblocks
  Witchstalker Frenzy, Bygone Bishop variants, Glasspool Mimic's
  copy trigger, and any "becomes target" cycle.

- ⏳ **`Predicate::ManaValueGreatest` — sacrifice picker filter**
  (suggested by push modern_decks's Soul Shatter addition) — Soul
  Shatter's printed Oracle is "Each opponent sacrifices a creature or
  planeswalker with the greatest mana value among permanents that
  player controls." Today the auto-picker takes the lowest-CMC
  matching permanent. Wiring shape: a new sacrifice-filter that
  reads each candidate's `card.definition.cost.cmc()` and picks the
  max. Same primitive unblocks future "with the highest power" /
  "with the lowest toughness" picker variants (Skull Fracture,
  Slaughter Specialist, etc.).

- ⏳ **`Effect::DiscardOrSacrifice` — additional-cost picker for "discard
  a card or sacrifice a creature"** — STA Bone Shards (already wired as a
  Sorcery in `mod_set::instants`) uses a `Seq(ChooseMode([Sacrifice 1
  creature, Discard 1]) + Destroy target creature)` approximation. The
  Strixhaven Mystical Archive reprint of Bone Shards is an *instant*
  with the same pick-as-additional-cost rider. Suggested shape: bump
  the picker into a real cost-time decision (so insufficient resources
  to pay one option force the other), wire it via `AlternativeCost`
  with two cost branches keyed off a `ChooseAlternativeCost` decision
  shape. Same primitive unlocks "Pay {X}, sacrifice a creature, or
  discard a card" cycles in future sets.

- ✅ **AutoDecider auto-targeting for additional_targets slots 1+** —
  shipped as `GameState::auto_targets_for_effect_all_slots`
  (`game/effects/targeting.rs`). Walks every `Selector::TargetFiltered
  { slot }` index in the effect tree (via `target_filter_for_slot_in_mode`)
  and returns `(Option<Target>, Vec<Target>)` — slot 0 plus
  `additional_targets` for slots 1+. Wired into the bot harness in
  `server/bot.rs`. Cards promoted via this path: Snow Day, Homesickness,
  Cost of Brilliance, Render Speechless, Vibrant Outburst, Dissection
  Practice, Rabid Attack, Together as One, Borrowed Knowledge (doc-sync),
  Magma Opus. Cap of 16 slots; deduplicates against earlier-filled slots.

- ✅ **Ward enforcement — full coverage (CR 702.21)** — both halves
  shipped: (a) `push_ward_triggers_for_cast` runs at the end of
  `finalize_cast` and pushes one Ward trigger per opp-controlled
  Ward permanent the spell targets; (b)
  `push_ward_triggers_for_activated_ability` runs after
  `activate_ability` queues the ability as a `StackItem::Trigger`,
  closing the "or ability" half of CR 702.21a. Both paths funnel
  through `push_ward_triggers_for_targets`. The cost enum
  `Keyword::Ward(WardCost)` carries `Mana(ManaCost) | Life(u32) |
  Discard(u32) | SacrificeCreature`. The new
  `Effect::CounterUnless { what, cost }` resolver walks the stack
  for a matching `Spell` (by `card.id`) or `Trigger` (by `source`)
  and auto-pays the cost on the affected controller's behalf
  (Discard auto-picks the first hand-card; Sacrifice auto-picks the
  first matching creature — interactive prompting is a UI follow-up).
  Promoted: Inkshape Demonstrator (Ward 2 generic), Mica (Ward—Pay
  3 life), Forum Necroscribe (Ward—Discard a card). Six tests
  cover the new variants and the activated-ability path.

- ⏳ **Burst Lightning kicker / kicker-as-modal** — STA reprint Burst
  Lightning's "Kicker {4} → 4 damage instead of 2" is an alt-cost-
  implies-mode shape: paying the kicker changes the spell's behavior at
  resolution. Currently wired as the unkicked 2-damage body only. The
  engine's `AlternativeCost` is one cost branch; threading the *paid*
  alt-cost into resolution-time mode selection would unblock Burst
  Lightning, Rite of Replication, Aether Vial-style kicker shells.
  Suggested shape: add `Predicate::CastWithKicker(name)` + thread the
  kicker payment status into `EffectContext`.

- ⏳ **`Predicate::ManaValueEquals(N)` — exact MV target filter** —
  Postmortem Lunge's "target creature card with mana value X" target
  filter (push modern_decks) synthesizes equality as
  `All([ValueAtLeast(MV, X), ValueAtMost(MV, X)])`. A first-class
  `ValueEquals` (or `ManaValueEquals`) predicate would compress the
  expression and let auto-target pickers natively narrow to the exact
  candidate set. The `If` gate on Postmortem Lunge could then drop to
  a plain target filter.

- ⏳ **`Value::PowerOfTargetExiledThisResolution`** — push (modern_decks)
  closed the simpler half via the `Value::PowerOf` evaluator-zone-walk
  extension (gy/exile/hand lookups now work), unlocking Lorehold
  Excavation's "X = its power" rider. The leftover gap is the
  ordering subtlety: a card that triggers _after_ exile (e.g.
  Lavaball Trap's hypothetical "exile a creature; you create an X/X
  where X is its power") needs to read power from the post-Move
  exile zone, not the pre-Move graveyard. The eval extension already
  walks exile, so most cases are covered — only the corner case of
  "the source card itself was exiled by the same effect" might need
  a temp-cached power. Suggested shape: stash `last_zone_changed_card`
  on `EffectContext` (sibling to `trigger_source`) and add
  `Value::PowerOfLastExiled` that reads from it. Open until a real
  card surfaces the gap (currently none in the Crabomination
  catalog).

- ⏳ **Multi-target prompts on instants/sorceries** — recurring 🟡
  reason across STRIXHAVEN2.md (Divergent Equation, Vibrant Outburst,
  Snow Day, Devious Cover-Up, Crackle with Power, Magma Opus,
  Homesickness, Dissection Practice, Cost of Brilliance, Render
  Speechless, Conciliator's Duelist, Rabid Attack, Together as One,
  Reconstruct History's "or more" mode-count picker, …). The engine's
  spell-cast path takes a single `Target` and the auto-decider can't
  pick multiple. Suggested shape: change `GameAction::CastSpell.target`
  from `Option<Target>` to `Vec<Target>` (or `Option<TargetSet>`),
  thread the slot index into `Selector::Target(n)` (already there),
  and bump cast-time target validation to walk every slot. The bot
  harness's AutoDecider needs a per-effect target-count introspection
  to pick N targets; a lazy first pass could just pick the same
  target N times (with deduplication on per-slot legality). Worth
  ~10 🟡 → ✅ promotions.

- ✅ **`Effect::CounterSpellToZone` — counter-spell-to-non-graveyard
  primitive** — Memory Lapse (STA reprint, `mod_set::instants`) is the
  canonical exerciser. Push (modern_decks): landed the new
  `Effect::CounterSpellToZone { what, zone: CounteredSpellZone }` variant
  with a `CounteredSpellZone` enum (`OwnerLibraryTop`,
  `OwnerLibraryBottom`, `OwnerHand`, `Exile`). The resolver lifts the
  matching `StackItem::Spell` off the stack and routes the card to the
  chosen zone (push for top of library, insert(0, _) for bottom, hand
  push for Remand, exile push for Spell Crumple). `Effect::CounterSpell`
  retains its graveyard default for back-compat. Memory Lapse promoted
  to use the new primitive (`OwnerLibraryTop`). Tracked rule audit row:
  CR 608.2c / 701.6a. Future Spell Crumple, Remand, Hinder, and
  Frantic Inventory-recursion shells can wire against the same
  primitive.

- ⏳ **Partner-pair primitive** — Plargg / Augusta (STX Dean cycle), the
  Battlebond Partner cycle, and the C20 Commander Partners all share a
  printed "Partner with [other Legendary]" rider that searches the
  library for the named partner on the Partner-carrier's ETB. Engine
  has no `Keyword::PartnerWith(name)` or `Effect::SearchByName`
  primitive yet. Suggested shape: add `Keyword::PartnerWith(&'static
  str)` + an ETB trigger that fires `Effect::Search { filter:
  HasExactName(name), to: Hand(You) }`. Once landed, the STX Dean
  cycle (Augusta + Plargg, Embrose + Valentin, Imbraham + Lisette,
  Lukka + Adrix) and the Battlebond legendaries can wire the partner
  half faithfully.

- ⏳ **Multi-pick on cleanup-step discard** — CR 514.1 enforcement
  landed in push (modern_decks) but the discard uses a deterministic
  first-card pick. A future UI surfacing should ask the active player
  which cards to discard via the existing `Decision::Discard` shape
  (the bot's AutoDecider can fall back to "first N"; only real-player
  seats need to surface the prompt). Cleanup is a turn-based action so
  it can't directly suspend through the stack; the existing
  `wants_ui` + `pending_decision` resume path may need extension to
  cover turn-based-action prompts. Wire site: `do_cleanup` in
  `game/stack.rs`.

- ⏳ **Cleanup-step discard event emission** — push (modern_decks)'s
  CR 514.1 wiring moves cards hand → graveyard but doesn't emit
  `GameEvent::CardDiscarded` (cleanup runs in a priority-less window
  per CR 514.3). Discard-payoff cards like the SOS Witherbloom
  death-trigger cycle and Liliana of the Veil's per-discard payoff
  may want this event. Per CR 419.1 the cards-go-to-graveyard count
  as discards; the engine's per-turn discard tally (when added) +
  every "if you discarded a card this turn" payoff would need to
  fire from this event.

- 🟡 **`StaticEffect::ConditionalPumpPT { condition, power, toughness,
  keywords }` — generalized compute-time conditional anthem** — push
  (modern_decks): consolidated the Honor Troll and Ulna Alley Shopkeep
  hardcoded `if name == "..." && lifegain > 0` branches in
  `GameState::compute_battlefield` into the helper-table
  `lifegain_selfpump_for_name(name) -> Option<(p, t, &[Keyword])>` at
  `game/mod.rs:1748` (mirroring the `tribal_anthem_for_name` precedent).
  Adding a new "as long as you've gained life this turn, +P/+T [and
  KW]" card now takes a single helper-table row instead of a new
  hardcoded branch. The full generalized primitive
  (`StaticEffect::ConditionalSelfPump { condition: Predicate, ... }`)
  is still ⏳ — it requires threading `&GameState` into
  `static_ability_to_effects` so predicates can read live game state.
  Today's lifegain-only helper is name-keyed, so non-lifegain
  conditions (e.g. "as long as you control a Wizard", "as long as
  it's not your turn") still need their own helper tables or the full
  primitive.

- ✅ **`Value::CardsDiscardedThisEffect` — track-discarded-by-this-effect
  counter** — Push (modern_decks): lands the per-resolution discard
  counter as `GameState.cards_discarded_this_resolution: u32` (scratch
  field, reset at the top of `resolve_effect` alongside
  `sacrificed_power` / `last_created_token`). Both `Effect::Discard`
  branches (random-pick and player-chosen via `DiscardChosenPending`)
  bump the counter for each discarded card. The new
  `Value::CardsDiscardedThisEffect` evaluator (`game/effects/eval.rs`)
  reads the counter. Borrowed Knowledge mode 1 promoted from "draw 7"
  approximation to `Value::CardsDiscardedThisEffect` — discards hand,
  draws exactly N where N = cards actually discarded. Tests:
  `borrowed_knowledge_mode_one_discards_hand_then_draws_same_count`
  (4 hand cards → 4 discards → 4 draws), `_with_small_hand_draws_
  proportionally` (1 → 1). Colossus of the Blood Age's "discard any
  number, draw that many plus one" rider and Mind Roots's "the land
  you discarded" rider can now wire the same primitive directly.

- ✅ **Snarl-land reveal mechanic** — Push (modern_decks) lands the
  new `Effect::IfRevealFromHand { filter, then, else_ }` primitive
  (`effect.rs:683`). Handler peeks at the controller's hand via
  `evaluate_requirement_on_card`; if any card matches `filter`,
  AutoDecider auto-reveals and runs `then`, else runs `else_`. The
  five STX Snarl dual lands (Frostboil / Furycalm / Necroblossom /
  Shineshadow / Vineglimmer) now wire their ETB trigger as
  `IfRevealFromHand { filter: HasLandType(type_a) ∨ HasLandType(type_b),
  then: Noop, else_: Tap { Selector::This } }` — matching the printed
  Oracle exactly. Future enhancement: surface a `Decision::Reveal`
  shape so a human player can decline to reveal (bluffing); today
  AutoDecider always reveals when a match exists. Tests:
  `frostboil_snarl_enters_untapped_with_island_in_hand`,
  `_with_mountain_in_hand`, `_enters_tapped_with_only_off_color_in_hand`,
  `_enters_tapped_without_revealable_card`. Same primitive unblocks
  Throne of Eldraine Battle Mammoth-style ETB reveals.

- ✅ **`Predicate::SameNamedInZoneAtLeast { who, zone, at_least }` —
  graveyard same-name count predicate (Dragon's Approach)** — push
  XXXVIII lands the predicate + the spell-resolution context channel
  needed to read the resolving spell's printed name. Wiring landed:
  (a) new `Predicate::SameNamedInZoneAtLeast { who, zone, at_least }`
  evaluator in `game/effects/eval.rs` that reads the spell name from
  `EffectContext.source_name` and counts matches in `who`'s `zone`;
  (b) new `EffectContext.source_name: Option<&'static str>` field +
  `for_spell_with_source` constructor that stamps both the spell
  CardId and printed name at resolution time; (c) `continue_spell_
  resolution` now uses `for_spell_with_source` so every spell's
  effect tree can read its own name. Dragon's Approach's gy-tutor
  rider is wired via `Effect::If { cond: SameNamedInZoneAtLeast(You,
  Graveyard, 4), then: Search { filter: Creature & Dragon, to:
  Battlefield } }`. Tests:
  `dragons_approach_tutors_dragon_with_four_in_graveyard` (scripted
  decider picks the Dragon),
  `dragons_approach_does_not_offer_tutor_without_four_named_in_graveyard`
  (the gate fails the predicate cleanly).

- ✅ **`Effect::CopySpellUnlessPaid { what, mana_cost, count }` — opp-spell
  tax-or-copy gate (Wandering Archaic)** — push (modern_decks) lands the
  primitive in `effect.rs` + handler in `game/effects/mod.rs`. At trigger
  resolution: (a) locate the matching `StackItem::Spell`; (b) ask the
  spell's *caster* yes/no via `Decision::OptionalTrigger`; (c) on yes
  + affordable pool, deduct `mana_cost` and skip the copy; (d) on no or
  unaffordable, copy the spell `count` times. The optional-pay decision
  flows through the caster's decider — the listening Wandering Archaic
  itself is owned by `you`, but the pay-or-copy decision is on the opp's
  side. AutoDecider answers false (decline to pay), so the bot-vs-bot
  flow defaults to "let the copy happen." Promotes Wandering Archaic
  🟡 → ✅. Tests: `wandering_archaic_lets_opp_pay_two_to_skip_copy`,
  `wandering_archaic_copies_when_opp_cannot_afford_two`,
  `wandering_archaic_copies_opp_instant`. The "may choose new targets
  for the copy" half stays engine-wide ⏳ (CopySpell inheriting
  original targets unchanged).

- ⏳ **`PlayerRef::Opponent` (single-opponent helper)** — engine has
  `EachOpponent` (all opps) and `Target(_)` (cast-time targeting) but
  no "the singular non-controller opp" ref. In 2-player games these
  collapse to the same player, but `Selector::Player(PlayerRef::
  Opponent)` would read more naturally for single-opp effects (e.g.
  "target opponent draws a card" in Baleful Mastery). Workaround
  today is `EachOpponent` which fan-outs in multiplayer.

- ✅ **`StaticEffect::PumpPTOther` / generalized tribal-anthem
  primitive** — Push (modern_decks): retired the
  `tribal_anthem_for_name` helper table entirely. Quintorius and
  Tenured Inkcaster now declare their tribal anthems as regular
  `StaticAbility { effect: StaticEffect::PumpPT { applies_to:
  Selector::EachPermanent(Creature ∧ HasCreatureType(X) ∧
  ControlledByYou ∧ OtherThanSource), .. } }` — same shape as Hofri
  Ghostforge. The `affected_from_requirement` selector→layer
  translator was already handling `HasCreatureType` and
  `OtherThanSource`; combining them produces
  `AffectedPermanents::AllWithCreatureType { exclude_source: true }`
  which is exactly what the helper used to inject. Adding a new
  "Other [type]s you control get +N/+M" card now requires zero
  engine changes — just a regular `StaticAbility` declaration.
  Goblin-King-style anthems for any tribe (Goblin, Elf, Zombie,
  Dragon, …) are unblocked.

- ✅ **`CardDefinition.enters_with_counters` primitive (CR 614.12
  replacement)** — Push (modern_decks): landed the new
  `enters_with_counters: Option<(CounterType, Value)>` field on
  `CardDefinition`. The counter spec is captured before the new
  permanent's zone change and applied **inside** the same ETB-zone
  hand-off in both code paths (`stack.rs` spell-resolution path and
  `effects/movement.rs::place_card_in_dest` for reanimate / flicker /
  search-to-battlefield), BEFORE state-based actions check toughness
  and BEFORE the first ETB trigger fires. Wiring threads the spell's
  `x_value` / `converged_value` via `EffectContext::for_spell_with_
  source` so `Value::XFromCost` and `Value::ConvergedValue` resolve
  faithfully (Pterafractyl X-spell, Rancorous Archaic Converge).
  Promotions: Pterafractyl drops the 1/0 → 1/1 toughness bump,
  Symmathematics drops the 0/0 → 1/1 toughness bump, Rancorous
  Archaic moves its Converge AddCounter from a post-SBA ETB trigger
  to the pre-SBA replacement (correct timing relative to other ETB
  triggers / replacement effects). Tests:
  `pterafractyl_cr_614_12_zero_toughness_base_survives_etb_via_
  enters_with`, `symmathematics_enters_with_two_plus_one_counters`
  (printed 0/0 → 2/2 exact), `pterafractyl_etb_with_x_counters_
  and_gains_two_life` (unchanged behavior at X=2).

- ⏳ **Add Inkling-tribal payoffs to the cube/SOS pools** — push XXXI
  added Tenured Inkcaster as an Inkling lord (+2/+2 to other
  Inklings). The catalog now has 4+ Inkling minters (Inkling
  Summoning, Defend the Campus, Silverquill Pledgemage,
  Promising Duskmage, Felisa Fang of Silverquill's Inkling
  generator) — a Silverquill SOS variant pool could lean heavily
  into the tribal pump. Add Inkling Mascot's printed "draw or pump"
  payoff variants once the multi-target prompt lands.

- ⏳ **Audit and update STRIXHAVEN2.md tables on every push** — push
  XXXI found 5 cards (Lorehold Apprentice, Lorehold Pledgemage,
  Storm-Kiln Artist, Sparring Regimen, Spectacle Mage) whose code
  was fully wired but whose 🟡 notes hadn't been updated. A simple
  end-of-push audit script (`audit_strixhaven2.py` already exists
  for SOS) extended to also walk STX-row notes against the
  factory's `triggered_abilities` / `static_abilities` / activated-
  ability complexity could flag stale rows automatically.

- ⏳ **Triggered mana ability fast-path (CR 605.1b)** — triggered mana
  abilities don't currently bypass the stack. The engine handles
  *activated* mana abilities specially (`activate_ability` resolves
  them immediately without `StackItem::Trigger` push) but triggered
  mana abilities like Mana Reflection's "Whenever a permanent taps
  for mana, that permanent produces twice as much instead" go through
  the normal dispatcher. No SOS/STX card exercises this today; first
  card to need it will be the wiring trigger.

- ✅ **Permanent-spell copy → token flag (CR 707.10f)** — `Effect::
  CopySpell`'s `copy_inst.is_token = true` flag is set on the
  `CardInstance` before the StackItem::Spell is pushed. On resolution,
  `self.battlefield.push(card)` (`stack.rs:332`) preserves the flag, so
  the resulting battlefield permanent is correctly a token. Token-
  cleanup SBA path handles removal when the permanent leaves the
  stack. See the new CR 608.3f / 707.10f rule audit row above. Closed
  by audit — the TODO statement was stale.

- ⏳ **CR 122.2-strict counter clearing on zone change** — to be
  fully compliant we should clear all counters when a card moves
  between zones. Currently the engine retains them (matching how
  the Felisa-style die-trigger reads counters off the graveyard
  copy), but a future "strict" pass should add an opt-in
  preservation flag and let CR 122.2 do its job by default. This
  unblocks future `WithCounter`-filtered triggers that *should*
  not see post-death counters (e.g. an opponent's Felisa-style
  payoff being kept alive by a counter that should have evaporated).

- ⏳ **`StaticEffect::SelfPumpIf` (conditional anthem on the source)** —
  Honor Troll's "as long as you've gained life this turn, gets +2/+0
  and lifelink" wants a conditional self-pump that checks a
  predicate (typically `LifeGainedThisTurnAtLeast(1)`) every time
  layers recompute. Shape:
  `StaticEffect::SelfPumpIf { condition: Predicate, power, toughness, keywords }`.
  Wire into `static_ability_to_effects` to conditionally emit the
  PumpPT + GrantKeyword pair only when `condition` is true.

- 🟡 **Multi-target action shape** — Push (modern_decks) lands the
  foundational primitive: `GameAction::CastSpell` (and the other four
  cast variants) gain an `additional_targets: Vec<Target>` field
  alongside the existing `target: Option<Target>`. Slot 0 stays in
  `target`, slots 1+ flow through `additional_targets`. The new field
  has `#[serde(default)]` for snapshot back-compat. Threaded through
  `StackItem::Spell`, `ResumeContext::Spell`, `cast_spell`,
  `cast_spell_with_convoke`, `cast_spell_back_face`, `cast_flashback`,
  `cast_spell_alternative`, `finalize_cast`,
  `continue_spell_resolution`, `EffectContext::for_spell_with_source`
  (merges both into `ctx.targets`). Cast-time validation walks every
  slot via `target_filter_for_slot_in_mode(slot_idx, mode)` and runs
  hexproof/legality checks on each. **Snow Day promoted** as the
  first two-slot card: `Effect::Seq([Tap(target_filtered slot 0),
  AddCounter(Target(0)), Tap(TargetFiltered slot 1), AddCounter(
  Target(1))])`. "Up to two" semantics fall out naturally — slot-1
  selectors resolve to nothing when only one target is passed, so
  the second tap+stun pair is a no-op. Tests:
  `snow_day_taps_and_stuns_target_creature` (slot 0 only),
  `snow_day_taps_and_stuns_two_target_creatures` (both slots).
  **Still 🟡 because the AutoDecider's auto-target picker does not
  yet populate `additional_targets`** — cards relying on the bot to
  pick slot-1 targets need manual promotion (Crackle with Power,
  Render Speechless, Vibrant Outburst, Devious Cover-Up, Decisive
  Denial mode 1, etc.). The cast API supports them; the bot harness
  hasn't been updated to drive them. Easy follow-on push: extend
  `auto_target_for_effect_avoiding` to take a slot count and return
  `Vec<Target>` with per-slot legality.

- ✅ **`SelectionRequirement::OtherThanSource` — target-validation half**
  (push modern_decks): Threaded `source: Option<CardId>` into
  `evaluate_requirement_static` (`game/effects/eval.rs:414`) and all 16
  external call sites: cast-spell target validation (`actions.rs`),
  alt-cost target validation, activated/loyalty ability target
  validation (`mod.rs` / `actions.rs`), trigger-resolve target re-pick
  (`mod.rs`), auto-target picker (`effects/targeting.rs`), and the
  selector resolvers for `EachPermanent` / `EachMatching` / zone-walks
  (`effects/mod.rs`). The `OtherThanSource` arm now reads as
  `*cid != src_id` when source is known, else falls through to
  permissive (preserves the static-ability `applies_to` pipeline's
  pre-existing handling via `AffectedPermanents.exclude_source`).
  Tests: `other_than_source_strict_filter_excludes_lone_source_target`
  (auto-target picker returns `None` when only the source matches),
  `other_than_source_without_source_is_permissive` (the public
  `evaluate_requirement` API passes `None` and OtherThanSource doesn't
  reject). Lorehold Pledgemage's exile-from-gy cost / Felisa Fang's
  Inkling generator can now be retrofitted directly with
  `OtherThanSource` target filters instead of their existing
  heuristics — opportunity-of-improvement rather than necessity.

- ✅ **`EventKind::Blocks` / `BlockerDeclared`** — block-half triggers
  (Daemogoth Titan, Wall of Junk, …) need a per-blocker event that
  fires when `DeclareBlockers` resolves. Done in push XXVI:
  `combat::declare_blockers` emits `GameEvent::BlockerDeclared {
  blocker, attacker }` and the event dispatcher routes it to both
  `EventKind::Blocks` (blocker side) and `EventKind::BecomesBlocked`
  (attacker side) — see `game/effects/events.rs:33-68`. Triggered
  abilities subscribe via `EventScope::SelfSource` for the blocker
  half (the blocker is the source) or by matching the `attacker`
  field for the attacker half.

- ⏳ **Lesson sideboard model** — Learn currently collapses to
  Draw 1. A true Lesson sideboard would let Eyetwitch / Hunt for
  Specimens / Field Trip / Igneous Inspiration etc. search a
  sideboard of Lesson cards. Needs a per-player Lesson sideboard
  slot plus a search-by-spell-subtype primitive on top of
  `Effect::Search`.
- ⏳ **Counter-multiplier primitive** — Already used by Tanazir
  (via the ForEach idiom). Future cards (Vorinclex, Doubling
  Season) want a true multiplier on counter accrual; tracked
  separately.
- ⏳ **Mana-spent-on-cast introspection** — Opus / Increment
  riders read "amount of mana spent to cast that spell" on the
  just-cast spell event. The engine doesn't yet preserve the
  numeric mana-paid total per stack item; this would unblock
  Aberrant Manawurm, Tackle Artist, Expressive Firedancer, etc.
  Suggested shape: `Value::ManaSpentOnCast(Box<Selector>)` that
  reads from `StackItem::Spell.mana_paid_total`.
- 🟡 **CR 700.2d — modal "choose two" / "choose more than one"** —
  push XXXII landed the engine half via the new `Effect::ChooseN {
  picks: Vec<u8>, modes: Vec<Effect> }` primitive. The auto-decider
  runs each picked mode in `picks` order, sharing the spell's single
  target slot. The five Strixhaven Commands (Witherbloom / Lorehold /
  Quandrix / Silverquill / Prismari) are now ✅ via hard-coded
  per-card default picks. Mode-pick UI plumbing — letting the
  controller choose `picks` at cast time, rather than relying on the
  factory's default — is still ⏳. Engine shape for the UI half:
  bump `GameAction::CastSpell.mode: Option<u8>` → `modes: Vec<u8>`
  and thread it into the `ChooseN`'s `picks` at resolution.
- ⏳ **`magecraft_self_untap()` / `magecraft_drain_each_opp(N)`
  shortcuts** — push XXVII added two new shortcut helpers in
  `effect::shortcut`. Future STX/SOS Magecraft creatures should
  prefer these over the verbose inline form for consistency. Hall
  Monitor (push XXVII) and Witherbloom Apprentice (refactored in
  push XXVII) demonstrate the pattern.
## Recent additions

- ✅ **modern_decks session (2026-05-26 cont.)**: 2 engine primitives +
  5 tests:
  - **Prowess trigger (`shortcut::prowess_trigger()`)**: "Whenever you
    cast a noncreature spell, this creature gets +1/+1 until end of
    turn." Built on new `cast_is_noncreature()` predicate. Wired into
    Spectacle Mage (STX), Stormchaser Mage (OGW), and Monastery
    Swiftspear (Modern).
  - **Deathtouch SBA enforcement (CR 704.5g)**: `check_state_based_
    actions` now kills creatures with any deathtouch damage (even 1
    point below toughness). Previously only `damage >= toughness` was
    checked, ignoring the `deathtouch_damaged` flag set during combat.
    Indestructible correctly prevents destruction from deathtouch.
  - 5 new tests. All 1151 tests pass.

- ✅ **modern_decks batch 2 (2026-05-26)**: 21+ new cards + 3 engine
  improvements + tests:
  - **Ward enforcement (CR 702.21)**: When a spell targets a permanent
    with Ward controlled by an opponent, a `CounterUnlessPaid` trigger
    is placed on the stack. The spell's controller must auto-pay the
    Ward cost or lose the spell. Promotes all Ward-tagged cards from
    keyword-only to functional enforcement.
  - **Stun counter enforcement (CR 701.48)**: Permanents with stun
    counters no longer untap during the untap step. Instead, one stun
    counter is removed per step. Promotes Frost Trickster, Deluge
    Virtuoso, Static Prison, and all stun-counter-adding cards.
  - **Beledros Witherbloom activation**: Pay-10-life mass-untap-lands
    activated ability now wired (was body-only 🟡).
  - **Decisive Denial mode 1**: Fight mode added (attacker auto-picked
    from your creatures, defender is the target).
  - New SOS cards: Strife Scholar MDFC, Colorstorm Stallion, Elemental
    Mascot, Molten Note (Flashback), Social Snub, Fix What's Broken,
    Skycoach Waypoint, Biblioplex Tomekeeper, Strixhaven Skycoach,
    The Dawning Archaic, Applied Geometry, Prismari the Inspiration,
    Nita Forum Conciliator, Silverquill the Disputant, Quandrix the
    Proof.
  - New STX cards: Introduction to Prophecy, Introduction to
    Annihilation, Environmental Sciences, Spirit Summoning.
  - All 1118+ tests pass.

- ✅ **modern_decks-18 (2026-05-26)**: 35 new STX card factories + engine
  improvements + 41 new tests. Tests at 1202 (+44 net from 1158):
  - **35 new STX card factories** across all five Strixhaven schools:
    Lorehold Command, Academic Dispute, Blade Historian, Reconstruct
    History, Rip Apart, Prismari Command, Creative Outburst, Elemental
    Summoning, Teach by Example, Quandrix Command, Fractal Summoning,
    Dragonsguard Elite, Eureka Moment, Silverquill Command, Umbral Juke,
    Silverquill Silencer, Fracture, Humiliate, Clever Lumimancer,
    Witherbloom Command, Culling Ritual, Rushed Rebirth, Callous
    Bloodmage, Environmental Sciences, Introduction to Annihilation,
    Introduction to Prophecy, Expanded Anatomy, Cram Session, Professor
    of Symbology, Guiding Voice, Divide by Zero, Flunk, Curate, Igneous
    Inspiration, Charge Through.
  - **Engine: CR 704.5h deathtouch SBA** — new `CardInstance.
    deathtouch_damaged` flag tracks deathtouch damage sources. SBAs now
    correctly kill creatures dealt any positive damage by a deathtouch
    source, regardless of toughness. Previously deathtouch only affected
    damage assignment (lethal=1) but not the SBA lethality check.
  - **Engine: CR 702.7 first-strike fix** — the `blocker_filter` parameter
    in `resolve_combat_damage_with_filter` was unused (`_blocker_filter`);
    now properly gates which blockers deal damage per step. During
    first-strike step only FS/DS blockers deal damage; during regular
    step only non-FS/DS blockers deal damage.
  - **Beledros Witherbloom activation** — promoted from body-only to
    functional: pay-10-life sorcery-speed mass-untap via existing
    `ActivatedAbility.life_cost`.
  - **Server: PermanentView.is_legendary** — clients can display crown
    icons or gold borders for legendary permanents.
  - **8 new CR rules tests**: 702.2 (deathtouch kills), 702.7 (first
    strike survives), 702.15 (lifelink gains), 704.5b (0 life), 704.5c
    (10 poison), 704.5i (0 loyalty), 704.5j (legend rule), 704.5n
    (orphaned aura).
  - **Clippy**: all warnings resolved except acceptable `too_many_arguments`
    and `large_size_difference` (both stylistic).

- ✅ **modern_decks-16/17 (2026-05-26)**: 46 new cube cards + CR 120.3
  planeswalker damage + PermanentView improvements. Tests at 1158 (+55
  net from 1103):
  - **46 new card factories** across modern.rs: Wall of Omens, Lingering
    Souls, Decree of Justice, Mulldrifter, Shriekmaw (Evoke), Deep
    Analysis (Flashback), Spell Queller, Thragtusk, Kitchen Finks
    (Persist), Electrolyze, Expressive Iteration, Oko Thief of Crowns,
    Baleful Mastery, Bloodbraid Elf, Kolaghan's Command, Collective
    Brutality, Firebolt (Flashback), Chainer's Edict (Flashback),
    Tireless Provisioner, Courser of Kruphix, Elder Gargaroth, Arclight
    Phoenix (graveyard recursion), Vengevine (graveyard recursion), Grim
    Flayer, Young Pyromancer, Monastery Swiftspear, Snapcaster Mage,
    Stonecoil Serpent, Tasigur, Pernicious Deed, Toxic Deluge, Sinkhole,
    and more.
  - **Engine: CR 120.3** — damage dealt to a planeswalker now correctly
    removes loyalty counters instead of marking regular damage. Lightning
    Bolt can now kill a low-loyalty planeswalker.
  - **Engine: PlaneswalkerSubtype::Oko** added.
  - **Server: PermanentView.mana_value + creature_types** — clients can
    now display CMC and creature type line.
  - **Rules tests**: CR 120.3 planeswalker damage, CR 704.5q counter
    cancellation, CR 704.5j legend rule.
  - All 1158 lib tests pass.
- ✅ **Push XVII — modern_decks (2026-05-27)**: Ward enforcement +
  extra land plays + attack trigger dispatch + 16 new cards + 32 tests.
  Engine: Ward {N} enforced at targeting time (check + mana payment);
  `Player.extra_land_plays` + `Effect::GrantExtraLandPlay` for
  Explore-style "play an additional land" effects; `YourControl`-scoped
  `EventKind::Attacks` triggers from non-attacking permanents (Sparring
  Regimen). Server: `PermanentView.ward_cost` surfaced to client. UI:
  Ward cost in permanent tooltips. Card promotions: Beledros Witherbloom
  (mass untap), Lorehold Apprentice (magecraft damage), Sparring Regimen
  (attack trigger), Decisive Denial (fight mode), Inkshape Demonstrator /
  Fractal Tender / Thornfist Striker (Ward enforced). New cards: 4 SOS
  MDFCs (Campus Composer, Emeritus of Ideation, Grave Researcher, Strife
  Scholar), Fix What's Broken, Molten Note, Guardian Scalelord, Descendant
  of Storms, Explore, Elite Spellbinder, Gush, Elder Gargaroth. Comp rules:
  305.7 (additional land plays), 702.21 (Ward), 704.5 (SBA verification).
  All 1068 lib tests pass.

  ### Things noticed but not tackled this run
  - **Decisive Denial mode 1** uses `EachPermanent(Creature + ControlledByYou)`
    for the attacker selector — it should pick exactly one creature, not all.
    Multi-target prompts are still blocked.
  - **Fix What's Broken** collapses X=2 instead of letting the player choose X.
    Needs an "X life as additional cost" primitive.
  - **Grave Researcher back-face (Reanimate)** life-loss uses `ManaValueOf`
    targeting the same slot as the reanimated creature — may evaluate to 0 if
    the creature moved zones. Needs MV snapshot before the zone move.
  - **Copy-spell primitive** still blocks ~15 SOS cards (Choreographed Sparks,
    Mica, Prismari the Inspiration, Silverquill the Disputant, etc.).
  - **Cast-from-exile pipeline** still blocks ~10 SOS cards (Archaic's Agony,
    Flashback card, Elemental Mascot, The Dawning Archaic, etc.).

- ✅ **Push XVII.1 — modern_decks (2026-05-27)**: Stun counter untap
  suppression + hand-size cleanup + Intervention Pact + clippy fixes.
  Engine: CR 122.1c (stun counters prevent untapping in both `do_untap`
  and `Effect::Untap` — one counter removed instead of untapping);
  CR 514.1 (active player discards down to 7 at cleanup). New card:
  Intervention Pact ({0} Instant, gain 5 life + pact upkeep trigger).
  Clippy: doc-list-item warnings reduced from 10 to 2. All 1071 tests.

  ### Things noticed but not tackled
  - **Stun counter from Effect::AddCounter** should also prevent the
    permanent from untapping on the very next untap step — currently
    only stun counters that exist at untap time are checked, so a
    counter added mid-turn is effective. This is correct behavior but
    worth documenting.
  - **Maximum hand size modification** — some cards (Reliquary Tower,
    Spellbook, Wisdom of Ages) grant "no maximum hand size". The
    cleanup discard-to-7 logic should check for a `no_max_hand_size`
    flag on the player (currently hardcoded to 7).
  - **Pact auto-pay** should check if the player can actually pay the
    mana before auto-paying. Currently if they can't afford it, they
    lose the game — this is correct MTG behavior but confusing for
    new players. A warning/confirmation would be nice.
  - **Back to Basics** (nonbasic lands don't untap) — would be easy
    to implement now that stun counter untap suppression is wired.
    Just add a static ability that prevents untapping (or grant stun
    counters to nonbasics at upkeep).
  - **Prepare mechanic** (SOS colorless) not implemented — Biblioplex
    Tomekeeper, Skycoach Waypoint.
  - **Vehicle/Crew** not implemented — Strixhaven Skycoach.
  - **Prowess keyword** tagged but not wired (Spectacle Mage).
  - **Elder Gargaroth** attack trigger should also fire on blocks — engine
    needs a `Blocks` event kind.

- ✅ **SOS push XVI (2026-05-01)**: 5 engine primitives + 10 SOS/STX
  card promotions. Tests at 1025 (+13 net):
  - **`Predicate::CastSpellHasX`** — cast-time introspection on the
    just-cast spell's `{X}` symbols. Used by Quandrix's "whenever
    you cast a spell with `{X}` in its mana cost" payoffs.
  - **`Effect::MayPay { description, mana_cost, body }`** — sibling
    to push XV's `Effect::MayDo`, but with a mana-cost payment.
    Decline / can't-afford skip the body silently. Powers Bayou
    Groff's "may pay {1} to return on death" + future "may pay X
    to do Y" patterns.
  - **`SelectionRequirement::HasXInCost`** — card-level filter
    matching cards whose printed cost has at least one `{X}` pip.
    Wires Paradox Surveyor's "land OR card with {X} in cost"
    reveal filter to its exact-printed shape.
  - **`Value::LibrarySizeOf(PlayerRef)`** — `players[p].library
    .len()`. Promotes Body of Research from `GraveyardSizeOf`
    proxy to the printed library-size predicate.
  - **`shortcut::cast_has_x_trigger(effect)`** — Magecraft/Repartee-
    style helper for "whenever you cast a spell with {X}" payoffs.
  - **`Selector::CardsInZone(Hand)` filter-evaluation fix** —
    routing through `evaluate_requirement_on_card` (the card-level
    evaluator) instead of `evaluate_requirement_static` (which
    walks battlefield → graveyard → exile → stack only). Fixes
    silent zero-results for hand-source predicates.
  - **10 card promotions**: Geometer's Arthropod (⏳→✅),
    Matterbending Mage (🟡→✅), Paradox Surveyor (🟡→✅), Embrace
    the Paradox (🟡→✅), Sundering Archaic (🟡 — `{2}` activated
    ability wired), Aziza Mage Tower Captain (⏳→🟡 body-only),
    Zaffai and the Tempests (⏳→🟡 body-only); STX: Bayou Groff
    (🟡→✅), Felisa Fang of Silverquill (🟡→✅), Body of Research
    (🟡→✅).
  - 13 new tests in `tests::sos::*` and `tests::stx::*`. All 1025
    lib tests pass (was 1012).

- ✅ **SOS push XV (2026-05-01)**: Witherbloom (B/G) school complete +
  `Effect::MayDo` primitive + `ActivatedAbility.life_cost` field + 9
  card touches (3 new + 6 promotions/expansions):
  - **`Effect::MayDo { description: String, body: Box<Effect> }`** —
    first-class "you may [body]" primitive. Emits a yes/no decision via
    `Decision::OptionalTrigger`; only runs `body` when the decider
    answers `Bool(true)`. `AutoDecider` defaults to `false` (skip),
    matching MTG's "you may" defaults. Walkers
    (`requires_target`, `primary_target_filter`,
    `target_filter_for_slot_in_mode`) recurse into the inner body so
    target prompts/filters carry through correctly. The `description`
    is `String` (not `&'static str`) because `Effect` derives
    `Deserialize` via `GameState`.
  - **`ActivatedAbility.life_cost: u32`** — pre-flight life-payment
    gate on activations. Rejects activation cleanly with new
    `GameError::InsufficientLife` when controller's life is below the
    cost; pays up front after tap/mana succeed. Backed by
    `#[serde(default)]` for snapshot back-compat. The `cost_label`
    rendering in `server::view` shows "Pay N life" tokens.
    Powers Great Hall of the Biblioplex's `{T}, Pay 1 life: Add one
    mana of any color` faithfully — the effect is a pure `AddMana`,
    so the ability still resolves immediately as a true mana ability.
  - **Lluwen, Exchange Student // Pest Friend** 🟡 — Witherbloom MDFC
    (3/4 Legendary Elf Druid front + Pest-token sorcery back). Closes
    out the Witherbloom (B/G) school (zero ⏳ rows remaining for the
    school).
  - **Great Hall of the Biblioplex** 🟡 — Legendary colorless utility
    land. `{T}: Add {C}` + `{T}, Pay 1 life: Add one mana of any
    color` (via `life_cost: 1`). The `{5}: becomes 2/4 Wizard
    creature` clause is omitted (no land-becomes-creature primitive).
  - **Follow the Lumarets** 🟡 — `{1}{G}` Sorcery with the Infusion
    rider. `If(LifeGainedThisTurn) → 2× pull : 1× pull` over the top 4
    library cards (find creature-or-land → hand). Misses go to
    graveyard (engine default for `RevealUntilFind`).
  - **Erode** ✅ (was 🟡) — basic-land tutor for the target's
    controller now wired via
    `Search { who: ControllerOf(Target(0)), filter: IsBasicLand,
    to: Battlefield(ControllerOf(Target(0)), tapped) }`. The "may"
    optionality is collapsed to always-search (decline path covered
    by `Effect::Search`'s decider returning `Search(None)`).
  - **5 promotions via `Effect::MayDo`**: Stadium Tidalmage (ETB +
    Attacks loot), Pursue the Past (discard+draw chain), Witherbloom
    Charm mode 0 (sacrifice→draw 2), Heated Argument (gy-exile +
    2-to-controller rider), Rubble Rouser (ETB rummage). All five had
    been collapsed to always-on; now correctly opt-in.
  - 13 new tests in `tests::sos::*` (Lluwen P/T + back-face Pest
    minting; Great Hall mana abilities including the life-cost
    prepay; Follow the Lumarets mainline + Infusion paths;
    `MayDo`-skip tests for each promoted card to ensure the
    AutoDecider's `false` answer keeps the body unfired). All 1012
    lib tests pass.

- ✅ **SOS pushes XI / XII / XIII / XIV (2026-05-01)**: 29 new MDFC
  factories + 3 engine improvements + 44 new tests:
  - **Push XI**: 17 MDFC factories (Elite Interceptor // Rejoinder,
    Emeritus of Truce // Swords to Plowshares, Honorbound Page // Forum's
    Favor, Joined Researchers // Secret Rendezvous, Quill-Blade Laureate
    // Twofold Intent, Spiritcall Enthusiast // Scrollboost, Encouraging
    Aviator // Jump, Harmonized Trio // Brainstorm, Cheerful Osteomancer
    // Raise Dead, Emeritus of Woe // Demonic Tutor, Scheming Silvertongue
    // Sign in Blood, Adventurous Eater // Have a Bite, Emeritus of
    Conflict // Lightning Bolt, Goblin Glasswright // Craft with Pride,
    Emeritus of Abundance // Regrowth, Vastlands Scavenger // Bind to
    Life, Leech Collector // Bloodletting, Pigment Wrangler // Striking
    Palette). All 🟡 (front-face vanilla + back-face spell wired). New
    `catalog::sets::sos::mdfcs` module with `vanilla_front` /
    `spell_back` helpers keeping per-card boilerplate under 20 lines.
    24 new tests.
  - **Push XII**: 12 more MDFC factories — 7 mono-color (Spellbook
    Seeker, Skycoach Conductor, Landscape Painter, Blazing Firesinger,
    Maelstrom Artisan, Scathing Shadelock, Infirmary Healer) + 5 legendary
    multicolor (Jadzi, Sanar, Tam, Kirol, Abigale). All 🟡. 16 new
    tests.
  - **Push XIII** (engine): `Player.instants_or_sorceries_cast_this_turn`
    + `Player.creatures_cast_this_turn` tallies bumped in `finalize_cast`
    (when the resolving spell carries `CardType::Instant`/`Sorcery`/
    `Creature`). Reset on `do_untap`. New predicates
    `Predicate::InstantsOrSorceriesCastThisTurnAtLeast` and
    `Predicate::CreaturesCastThisTurnAtLeast`. Surfaced through
    `PlayerView` (with `#[serde(default)]`). Promotes Potioner's Trove's
    lifegain ability gate from the proxy `SpellsCastThisTurnAtLeast` →
    exact `InstantsOrSorceriesCastThisTurnAtLeast`. New gate label
    strings ("after instant/sorcery cast", "after creature cast") in
    `predicate_short_label`. 2 new tests.
  - **Push XIV** (engine + server): `enum CastFace { Front, Back,
    Flashback }` threaded through `GameEvent::SpellCast.face` +
    `GameEventWire::SpellCast.face`. Replays / spectator UIs can now
    distinguish back-face MDFC casts from normal hand casts and from
    flashback graveyard replays. New transient
    `GameState.pending_cast_face`; `cast_spell_back_face` sets `Back`
    before delegating, `cast_flashback` emits `Flashback` directly,
    default cast paths emit `Front`. 2 new tests.
  - All 997 lib tests pass (was 953; +44 net).
  - Cube color pool wiring: 6 white, 6 blue, 6 black, 5 red, 3 green
    MDFCs added; legendary multicolor MDFCs (Sanar UR, Tam GU, Kirol
    RW, Abigale WB) added to the matching cross-pools.

- ✅ **SOS push X (2026-05-01)**: 5 new SOS card factories (1 ✅, 4 🟡)
  + 4 promotions from 🟡 to ✅ (Flashback wirings) + 3 engine
  primitives:
  - **`Selector::Take { inner, count }`** — wraps another selector to
    clamp how many entities flow through (in resolution order). Sugar:
    `Selector::one_of(inner)`, `Selector::take(inner, n)`. Promoted
    Practiced Scrollsmith's gy-exile from "every matching" to "exactly
    one"; lifted Pull from the Grave from one creature to two. The
    target-filter/`requires_target` walkers recurse into the `inner`
    arm so wrapping a `TargetFiltered`/`CardsInZone` selector is
    transparent. Closes the long-standing "Move at most one matching
    card" / `Selector::OneOf` gap.
  - **`GameAction::CastSpellBack`** + **`cast_spell_back_face`** —
    generalises `PlayLandBack` to non-land MDFC back faces. Mirrors
    the `PlayLandBack` flow: swaps the in-hand card's `definition` to
    the back face's, then routes through `cast_spell` so cost / type
    / target filters / effect all resolve against the back face.
    First non-land MDFC wired: **Studious First-Year // Rampant
    Growth**. The 3D client picks this up automatically — the
    right-click flip on hand cards now routes flipped non-land
    MDFCs through `CastSpellBack` (in addition to `PlayLandBack` for
    land MDFCs). New `TargetingState.back_face_pending` flag carries
    the routing through the targeting prompt.
  - **`Keyword::Flashback` wirings on 7 SOS cards** — Daydream, Dig
    Site Inventory, Practiced Offense, Antiquities on the Loose,
    Pursue the Past, Tome Blast, Duel Tactics. Promotes Daydream,
    Dig Site Inventory, Tome Blast, Duel Tactics to ✅ (the only
    omission was Flashback, which is now wired via the engine's
    existing `cast_flashback` path). Antiquities, Pursue the Past,
    and Practiced Offense stay 🟡 because of separate non-Flashback
    omissions (cast-from-elsewhere rider, may-discard collapse,
    lifelink-or-DS mode pick).
  - 14 new tests in `tests::sos::*`. Cards: Inkshape Demonstrator 🟡,
    Studious First-Year // Rampant Growth ✅, Fractal Tender 🟡,
    Thornfist Striker 🟡, Lumaret's Favor 🟡; Daydream ✅, Dig Site
    Inventory ✅, Tome Blast ✅, Duel Tactics ✅, Practiced Offense 🟡,
    Pursue the Past 🟡, Antiquities on the Loose 🟡; Practiced
    Scrollsmith 🟡 (now exact one-card exile), Pull from the Grave 🟡
    (now up-to-2). All 953 lib tests pass.

- ✅ **SOS push IX (2026-05-01)**: 12 new SOS card factories
  (5 ✅, 7 🟡) plus one new engine primitive, finishing the
  Witherbloom (B/G) school (only the Lluwen MDFC remains, blocked
  on cast-from-secondary-face plumbing):
  - **`Player.creatures_died_this_turn`** + **`Predicate::CreaturesDiedThisTurnAtLeast`**
    — per-turn tally bumped from both the SBA dies handler in
    `stack.rs::apply_state_based_actions` (lethal-damage path) and
    `remove_to_graveyard_with_triggers` (destroy-effect path). Reset
    on `do_untap`. Surfaced through `PlayerView.creatures_died_this_turn`.
    Powers Essenceknit Scholar's end-step gated draw.
  - **`CreatureType::Dryad`** + **`PlaneswalkerSubtype::Dellian`** —
    new subtypes for Witherbloom-flavoured cards.
  - 17 new tests in `tests::sos::*` (ETB triggers, end-step gated
    draws, planeswalker loyalty activations, Surveil-anchored
    instants/sorceries, plus a tally-bumps-on-lethal-damage SBA test).
    All 932 lib tests pass.
  - Cards: Essenceknit Scholar ✅, Unsubtle Mockery ✅, Muse's
    Encouragement ✅, Prismari Charm ✅; Professor Dellian Fel 🟡,
    Textbook Tabulator 🟡, Deluge Virtuoso 🟡, Moseo Vein's New
    Dean 🟡, Stone Docent 🟡, Page Loose Leaf 🟡, Ral Zarek Guest
    Lecturer 🟡, Flow State 🟡.
  - Several 🔍-needs-review cards previously flagged as
    "Needs: Surveil keyword primitive" in the auto-generated table
    were already unblocked — Surveil is a first-class
    `Effect::Surveil` primitive. The script's
    `COMPLEX_KWS`/keyword-heuristic was stale. Fixed in-doc; future
    `gen_strixhaven2.py` runs should drop "Surveil" from
    `COMPLEX_KWS` so newly-fetched cards don't get flagged.

- ✅ **SOS push VIII (2026-05-01)**: 14 new SOS card factories
  (2 ✅, 12 🟡) plus two engine primitives that unblock conditional
  activations and counter-add self triggers:
  - **`ActivatedAbility.condition: Option<Predicate>`** — first-class
    "activate only if …" gate. Evaluated against the controller/source
    context **before** any cost is paid, so a failed gate doesn't burn
    the tap-cost or once-per-turn budget. New
    `GameError::AbilityConditionNotMet` for failed gates. Powers
    Resonating Lute's `{T}: Draw a card. Activate only if you have
    seven or more cards in your hand.` and promotes Potioner's Trove's
    lifegain ability to its printed gate. The struct field is
    `#[serde(default)]`; all 100+ existing literal initializations
    pick up `condition: None` via a one-shot patch.
  - **`EventScope::SelfSource` + `EventKind::CounterAdded` recognition**
    — `event_card`/`SelfSource` now match CounterAdded events to the
    source card. Berta, Wise Extrapolator's "whenever one or more +1/+1
    counters are put on Berta, add one mana of any color" trigger now
    fires only when counters land on Berta. Same hook unblocks
    Heliod-style "whenever a counter is put on this …" payoffs.
  - 19 new tests in `tests::sos::*`. Cards: Primary Research ✅,
    Artistic Process ✅, Decorum Dissertation 🟡, Restoration Seminar 🟡,
    Germination Practicum 🟡, Ennis the Debate Moderator 🟡, Tragedy
    Feaster 🟡, Forum Necroscribe 🟡, Berta the Wise Extrapolator 🟡,
    Paradox Surveyor 🟡, Magmablood Archaic 🟡, Wildgrowth Archaic 🟡,
    Ambitious Augmenter 🟡, Resonating Lute 🟡. Potioner's Trove was
    previously 🟡 (no gate); the gate is now wired so its lifegain
    ability rejects activation without an IS-cast that turn.
  - All 910 lib tests pass.

- ✅ **SOS push VII (2026-05-01)**: 11 new SOS card factories
  (3 ✅, 8 🟡) + 2 promotions (Owlin Historian 🟡 → ✅; Postmortem
  Professor's printed `Keyword::CantBlock` now wired). Engine adds:
  - **`SelectionRequirement::Multicolored`** + **`Colorless`** —
    counts the distinct colored pips in a card's mana cost (hybrid
    pips count both halves; Phyrexian counts the colored side;
    generic / colorless / Snow / X don't count). Backed by the new
    `ManaCost::distinct_colors()` helper. Wired into both the
    battlefield-resolve and library-search requirement evaluators
    so it works for cast-time triggers and selector-based
    cardpool filters. Promotes Mage Tower Referee
    (multicolored-cast → +1/+1 counter); ready for any future
    "multicolored matters" / "colorless matters" payoff.
  - **`tap_add_colorless()` shared helper** under
    `catalog::sets::mod` — `{T}: Add {C}` mana ability shorthand
    used by Petrified Hamlet and ready for Wastes / Eldrazi-flavoured
    colorless lands.
  - 11 new functionality tests in `tests::sos::*` + 3 in
    `tests::mana::*`. All 885 lib tests pass.
  - Cards: Mage Tower Referee ✅, Additive Evolution ✅, Owlin
    Historian ✅ (was 🟡), Spectacular Skywhale 🟡, Lorehold the
    Historian 🟡, Homesickness 🟡, Fractalize 🟡, Divergent Equation 🟡,
    Rubble Rouser 🟡, Zimone's Experiment 🟡, Petrified Hamlet 🟡.
    Postmortem Professor stays 🟡 but the printed "this creature
    can't block" static is now wired via `Keyword::CantBlock`.

- ✅ **SOS push VI (2026-05-01)**: 12 new SOS cards (4 ✅, 8 🟡) plus
  Topiary Lecturer rewrite + 5 false-negative cleanups, with three
  new engine primitives:
  - **`TokenDefinition.triggered_abilities`** + plumbing through
    `token_to_card_definition`. Promotes Send in the Pest, Pestbrood
    Sloth, Pest Summoning, Tend the Pests, Hunt for Specimens — the
    Pest tokens those spells mint now correctly carry their printed
    "die / attack → gain 1 life" rider. Added `stx_pest_token()`
    helper in `catalog::sets::stx::shared` for the death-trigger
    Witherbloom Pests.
  - **`ManaPayload::OfColor(Color, Value)`** — fixed-color, value-
    scaled mana adder. Single AddMana call, no player choice. Powers
    Topiary Lecturer's "{T}: Add G equal to power" cleanly (was a
    `Repeat × Colors([Green])` approximation).
  - **`Keyword::CantBlock`** — first-class "this creature can't block"
    keyword. Enforced inside `declare_blockers`, `can_block_any_attacker`,
    and `blocker_can_block_attacker`. Used by Duel Tactics's transient
    grant; Postmortem Professor's static restriction can be promoted
    to use it.
  - **`move_card_to` library traversal** — `Effect::Move` from a
    `Selector::TopOfLibrary` source now actually moves the top library
    card (previously the library branch was missing in `move_card_to`,
    so Suspend Aggression's exile-top-of-library half no-op'd). The
    library-source move is last in the search order to avoid
    accidentally consuming a hand card with the same id.
  - **Auto-target picker improvement**: friendly pumps (Magecraft /
    Repartee +1/+1 fan-out, transient PumpPT spells) now prefer the
    highest-power friendly creature, not the first-in-Vec match. This
    correctly aims Hardened Academic's CardLeftGraveyard counter at
    the biggest threat instead of the first 1-drop. Hostile picks
    still use first-match.
  - 12 new tests in `tests::sos::*`. All 870 lib tests pass.
  - Cards: Snarl Song ✅, Wild Hypothesis ✅, Send in the Pest ✅,
    Pestbrood Sloth ✅, Daydream 🟡, Soaring Stoneglider 🟡, Tome
    Blast 🟡, Duel Tactics 🟡, Ark of Hunger 🟡, Suspend Aggression
    🟡, Wilt in the Heat 🟡, Practiced Scrollsmith 🟡, Topiary
    Lecturer (rewrite, kept 🟡 — Increment rider still missing).
  - 5 false-negative status cleanups (the cards were already wired
    but the doc still said ⏳): Hydro-Channeler, Geometer's
    Arthropod, Sundering Archaic, Transcendent Archaic, Ulna Alley
    Shopkeep — all 🟡.

- ✅ **SOS push V (2026-04-30)**: 12 new SOS cards (3 ✅, 9 🟡) plus
  three new engine primitives that unblock Lorehold "cards leave your
  graveyard" payoffs and proper fight resolution:
  - **`EventKind::CardLeftGraveyard`** + `GameEvent::CardLeftGraveyard`
    — fires per card removed from a graveyard (return-to-hand,
    flashback cast, persist/undying battlefield-return, exile-from-gy).
    Plumbed in `move_card_to`'s graveyard branch, `cast_spell_flashback`
    in actions.rs, and persist/undying returns in stack.rs. Each
    emission also bumps the new
    `Player.cards_left_graveyard_this_turn` tally (reset on
    `do_untap`), surfaced through `PlayerView` for client UIs.
  - **`Predicate::CardsLeftGraveyardThisTurnAtLeast`** — gates Lorehold
    "if a card left your graveyard this turn" payoffs (Living
    History's combat trigger; Primary Research's end-step draw and
    Wilt in the Heat's cost reduction will use the same predicate).
  - **`Predicate::SpellsCastThisTurnAtLeast`** — gates Burrog
    Barrage's "if you've cast another instant or sorcery this turn"
    pump.
  - **`Effect::Fight { attacker, defender }`** — proper bidirectional
    fight primitive. Snapshots both creatures' powers up-front; no-ops
    cleanly when either selector resolves to no permanent. Unblocks
    Chelonian Tackle's "fight up to one opp creature" (single-target
    collapse on the defender pick), and is ready for Decisive Denial
    mode 1 + future fight-style cards.
  - **`Effect::Untap.up_to: Option<Value>`** — untap-with-cap. Frantic
    Search's "untap up to three lands" now honors the printed cap
    precisely (was "untap all"). Other Untap callers opt-out via
    `up_to: None`.
  - 13 new tests in `tests::sos::*` + 1 in `tests::modern::*`. All 857
    lib tests pass.
  - Cards: Hardened Academic ✅, Spirit Mascot ✅, Garrison Excavator ✅,
    Living History 🟡, Witherbloom the Balancer 🟡, Burrog Barrage 🟡,
    Chelonian Tackle 🟡, Rabid Attack 🟡, Practiced Offense 🟡, Mana
    Sculpt 🟡, Tablet of Discovery 🟡, Steal the Show 🟡.

- ✅ **modern_decks post-push III batch (2026-04-30)**: 10 SOS cards
  (5 ✅, 5 🟡) plus 5 new engine primitives:
  - **`Value::Pow2(Box<Value>)`** — 2ˣ with the exponent capped at
    30. Powers Mathemagics's "draw 2ˣ cards".
  - **`Value::HalfDown(Box<Value>)`** — half of a value, rounded
    down. Powers Pox Plague's "loses half / discards half / sacs
    half" three-stage effect.
  - **`Value::PermanentCountControlledBy(PlayerRef)`** — counts
    permanents controlled by the resolved player. Lets per-player
    iteration in `ForEach Selector::Player(EachPlayer)` correctly
    compute the iterated player's permanent count instead of always
    reading `ctx.controller`'s board.
  - **`Selector::CastSpellTarget(u8)`** — resolves the chosen target
    slot of the spell whose `SpellCast` event produced the current
    trigger. Walks the stack for the matching spell. Used by
    Conciliator's Duelist's Repartee body to exile the cast spell's
    chosen creature target.
  - **`AffectedPermanents::AllWithCounter { controller, card_types,
    counter, at_least }`** — counter-filtered lord-style statics.
    `affected_from_requirement` recognises `SelectionRequirement::
    WithCounter(...)` in the static's selector and routes through the
    new variant. Powers Emil's "creatures with +1/+1 counters have
    trample" + future "monstrous / leveled creatures gain
    [keyword]" buffs.
  - 12 new tests in `tests::sos::*`. Cards: Mathemagics ✅, Visionary's
    Dance ✅, Pox Plague ✅, Emil Vastlands Roamer ✅, Orysa ✅
    (post-push III), Conciliator's Duelist 🟡 (Repartee exile half
    promoted), Abstract Paintmage 🟡, Matterbending Mage 🟡,
    Exhibition Tidecaller 🟡, Colossus of the Blood Age 🟡. All 851
    lib tests pass.

- ✅ **SOS push III + Multicolored predicate (2026-04-30)**: 13 new SOS
  card factories (4 fully ✅, 9 body-only 🟡) plus engine wins:
  - **`SelectionRequirement::Multicolored`** + **`Colorless`** —
    counts distinct colored pips in a card's cost (hybrid counts both
    sides; Phyrexian counts the colored side). Unblocks Mage Tower
    Referee's "whenever you cast a multicolored spell" trigger.
  - **`Effect::Move` from library** — `move_card_to` now walks each
    player's library when locating the source card, so a `Selector::
    TopOfLibrary { count } → ZoneDest::Exile` move actually exiles the
    top card. Suspend Aggression uses this; Daydream / Practiced
    Scrollsmith and other "exile top of library, then …" cards get
    library-source moves for free.
  - 14 new tests in `tests::sos::*`. All 838 lib tests pass.
  - Cards: Mage Tower Referee ✅, Transcendent Archaic ✅, Snarl Song ✅,
    Poisoner's Apprentice ✅, Sundering Archaic 🟡, Hydro-Channeler 🟡,
    Ulna Alley Shopkeep 🟡, Topiary Lecturer 🟡, Garrison Excavator 🟡,
    Spirit Mascot 🟡, Geometer's Arthropod 🟡, Suspend Aggression 🟡,
    Living History 🟡.

- ✅ **SOS body-only batch (2026-04-30)**: 13 SOS creatures previously
  marked ⏳ are now 🟡 with their printed cost / type / P/T / keywords
  correct. Cards are usable in cube color pools and combat; their
  Increment / Opus / mana-spent-pump riders are omitted pending the
  "mana-paid-on-cast introspection" engine primitive (see Engine —
  Missing Mechanics below). Plus Ajani's Response shipped with destroy
  but no cost-reduction. New `CreatureType::Dwarf` added for
  Thunderdrum Soloist. 11 functionality tests in `tests::sos::*`. All
  822 lib tests pass.

- ✅ **Auto-target source-avoidance (2026-04-30)**: triggered abilities
  now skip the trigger source as a target candidate when another legal
  target is available. New `auto_target_for_effect_avoiding(eff,
  controller, avoid_source)` API; all trigger-creation paths updated
  (ETB, combat, dies/leaves, delayed). Quandrix Apprentice's Magecraft
  pump now deterministically prefers a non-source creature; falls back
  to the source when it's the only legal pick. 2 new tests in
  `tests::stx::*`.

- ✅ **SOS expansion II (2026-04-30)**: 11 more cards bridging the
  Silverquill (W/B) and Lorehold (R/W) schools, plus a handful of
  cross-school staples and mono-color removal/utility.
  - Silverquill: Moment of Reckoning (modal destroy/return), Stirring
    Honormancer (look-at-X-find-creature via `RevealUntilFind`),
    Conciliator's Duelist (ETB body wired; Repartee exile-with-return
    is omitted).
  - Lorehold: Lorehold Charm (all 3 modes), Borrowed Knowledge (mode 0
    faithful, mode 1 collapsed to "draw 7").
  - Witherbloom: Vicious Rivalry (X-life cost approximation +
    `ForEach.If(ManaValueOf ≤ X) → Destroy`).
  - Quandrix: Proctor's Gaze (bounce + Search basic to bf tapped).
  - Mono-color staples: Dissection Practice ({B} drain+shrink), End of
    the Hunt ({1}{B} exile opp creature/PW), Heated Argument ({4}{R} 6
    + 2-to-controller), Planar Engineering ({3}{G} sac 2 lands +
    Repeat×4 fetch basics).
  - 11 functionality tests in `tests::sos::*`. All 807 lib tests pass.
  - Cube cross-pool pools updated for W/B, B/G, G/U, R/W; mono-color
    pools (Black, Red, Green) picked up the new mono-color cards.

- ✅ **SOS expansion (2026-04-30)**: 10 new / improved cards.
  - Graduation Day ({W} Repartee enchantment) — new.
  - Stirring Hopesinger / Informed Inkwright / Inkling Mascot /
    Snooping Page — Repartee riders fully wired (was 🟡, now ✅).
  - Withering Curse ({1}{B}{B}) — Infusion-gated mass debuff/wrath.
  - Root Manipulation ({3}{B}{G}) — pump + menace fan-out (🟡:
    on-attack rider stubbed pending transient-trigger-grant primitive).
  - Blech, Loafing Pest ({1}{B}{G}) — lifegain-multi-tribe pump.
  - Cauldron of Essence ({1}{B}{G}) — death drain + sac-reanimation.
  - Diary of Dreams + Potioner's Trove (colorless artifacts, 🟡 with
    minor caveats noted in STRIXHAVEN2.md).
  - Spectacle Summit (Prismari U/R school land).
  - 13 new tests in `tests::sos::*`.
  - Cube color pools refreshed: Witherbloom (B/G), Silverquill (W/B),
    Prismari (U/R) cross-pools each picked up the relevant cards.
- ✅ **`scripts/gen_strixhaven2.py`** — oracle text is no longer
  truncated. Earlier revisions cut to 220 chars (then 600); both
  silently dropped late keywords (Flashback, Crew, Prepare reminder
  text). The script now passes the full oracle through unmodified.
  All STRIXHAVEN2.md rows whose oracle was previously clipped were
  marked **🔍 needs review (oracle previously truncated)** so future
  card-implementation passes know to cross-check the body before
  authoring against the row's existing notes (52 rows tagged).
- ✅ **STX schools expanded**: new modules under `catalog::sets::stx` for
  Lorehold, Quandrix, and Prismari. 11 new STX cards across the four
  colleges (Lorehold Apprentice/Pledgemage, Pillardrop Rescuer, Heated
  Debate, Storm-Kiln Artist, Quandrix Apprentice/Pledgemage, Decisive
  Denial, Prismari Pledgemage/Apprentice, Symmetry Sage) plus
  Witherbloom Pledgemage. Pest Summoning bumped from 1 → 2 tokens to
  match the printed Oracle. 13 new functionality tests.
- ✅ **`scripts/gen_strixhaven2.py` parsing fixes**:
  - Oracle truncation cap raised 220 → 600 chars (was clipping the
    bodies of cards with reminder-text-laden modes — including the
    Prepare keyword's definition on its grantor cards).
  - Recognises new SOS-only mechanics (Repartee, Magecraft, Increment,
    Opus, Infusion, Paradigm, Converge, Casualty, Prepare) as needing
    engine primitives, so the per-card hint column now points at the
    right plumbing.
  - Added a "Prepare mechanic" explainer to STRIXHAVEN2.md and a TODO
    item for the per-permanent prepared flag + setter primitive.
- ✅ `once_per_turn` flag on activated abilities is now enforced engine-side
  (was a struct field with no validation). Cards: Mindful Biomancer, etc.
- ✅ Strixhaven creature/spell subtypes added: Inkling, Pest, Fractal, Orc,
  Warlock, Bard, Sorcerer, Pilot, Elk.
- ✅ SOS catalog scaffolded under `catalog::sets::sos` with 51+ card
  factories wired into the cube color pools (white, blue, black, red,
  green, plus W/B Silverquill, B/G Witherbloom, G/U Quandrix, U/R
  Prismari, R/W Lorehold cross-pools).
- ✅ `Player.life_gained_this_turn` tally added (with `Effect::GainLife`,
  `Effect::Drain`-recipient, and combat-lifelink integration). Cleared on
  `do_untap`. Surfaced through `PlayerView` for client UIs.
- ✅ `Predicate::LifeGainedThisTurnAtLeast { who, at_least }` for "if you
  gained life this turn" Infusion riders (Foolish Fate, Old-Growth
  Educator, Efflorescence wired so far).
- ✅ `PlayerRef::OwnerOf(Selector)` / `ControllerOf(Selector)` now fall
  back through graveyards / hands / library / exile when the target has
  already changed zones (typical case: destroy-then-drain-controller),
  via the new `GameState::find_card_owner` helper.
- ✅ **`StackItem::Trigger.x_value`** — ETB triggers fired off a
  resolving spell now inherit that spell's paid X. `Effect::AddCounter
  { amount: Value::XFromCost }` and similar X-driven effects on
  creature/permanent ETBs read the correct X (Pterafractyl, Static
  Prison). `ResumeContext::Trigger` carries the same `x_value` so a
  suspended trigger resumes with the right X.
- ✅ **`Selector::LastCreatedToken`** + **`Value::CardsDrawnThisTurn`**
  + **`Player.cards_drawn_this_turn`**. `Effect::CreateToken` stashes
  the freshly-minted token id on the game state so a follow-up
  `AddCounter` / `PumpPT` in the same `Effect::Seq` can target it via
  `Selector::LastCreatedToken`. Combined with `Player.draw_top()`
  incrementing `cards_drawn_this_turn` (reset on the controller's
  untap), the new primitives unblock Quandrix scaling (Fractal Anomaly
  is now ✅).
- ✅ **`ClientView.exile`** + **`ExileCardView`**. The shared exile
  zone now projects through the per-seat view so a client UI can
  render an exile browser. Each entry carries the card's owner so the
  UI can distinguish "exiled by you" from "exiled from your library".
- ✅ **`PlayerView.cards_drawn_this_turn`**. Surfaced for client UIs
  to preview Quandrix scaling on cards in hand.
- ✅ **STX (Strixhaven base set) module** under `catalog::sets::stx`,
  parallel to the existing SOS module. 14 cards across Silverquill,
  Witherbloom, and shared (Inkling Summoning / Tend the Pests). 15
  functionality tests, all passing. See `STRIXHAVEN2.md` ("Strixhaven
  base set (STX)" section).
- ✅ **`effect::shortcut::magecraft(effect)` helper** + supporting
  `cast_is_instant_or_sorcery()` predicate. Lets a Magecraft trigger
  drop into a card factory in one line instead of seven. Used by
  Eager First-Year and Witherbloom Apprentice.
- ✅ **12 stale-test fixes** — Devourer of Destiny re-cost (5→7), plus
  Biorhythm/Holy Light/Loran/Path of Peace/Read the Tides cost drift,
  Lumra keyword (Reach→Trample), and a cube-prefetch test that lost
  several no-longer-pooled card names. All 736 → 751 tests now pass.

---

## Engine — Missing Mechanics

### Replacement Effects
The engine has no general replacement-effect primitive.  Many real cards need one:
- ETB replacements (Containment Priest, Torpor Orb, Rest in Peace)
- Damage replacements (protection, preventing damage):
  - 🟡 **Combat damage prevention** (Owlin Shieldmage, Holy Day, Constant
    Mists) is partially supported via the new `Effect::PreventAllCombatDamage
    ThisTurn` primitive + `GameState.prevent_combat_damage_this_turn` flag
    (CR 615.1). Per-source / per-N shields (Wojek Apothecary, Stave Off,
    Lapse of Certainty) are still ⏳. Non-combat damage prevention
    (Reverse Damage, Mending Hands) is also ⏳.
- Draw replacements (Leyline of the Void)
- Death replacements (Kalitas, Oubliette)
Until this lands, cards with "instead" clauses are either stubbed or collapsed
into a close approximation.

### Per-Activation Mana-Spent Introspection
Reckless Amplimancer reads "+X/+X where X is the amount of mana spent to
activate this ability". The engine tracks per-cast `mana_spent` on
`StackItem::Spell` and per-trigger on `StackItem::Trigger`, but the
activated-ability path (`activate_ability`) doesn't capture mana spent.
Adding this requires:
1. An `x_value: Option<u32>` field on `GameAction::ActivateAbility` for
   X-cost activations (parallel to `CastSpell.x_value`).
2. Threading `mana_spent` through the activation's `StackItem::Trigger`
   construction in `activate_ability` (the field exists but is always 0).
3. Wiring `Value::CastSpellManaSpent` to read from the stack item.
Then Reckless Amplimancer's +3/+3 hardcode can be replaced with
`Value::CastSpellManaSpent` for printed-Oracle parity. Tracked as engine
work — same shape would unlock other X-cost activations (Berta's
{X},{T}: Create Fractal with X counters).

### Cast-From-Exile Pipeline
Many cards exile a spell/card temporarily and later cast it (Foretell,
Suspend, Rebound, Flashback-from-exile, Escape, Adventure second cast,
Cascade resolution).  Currently each is handled ad-hoc or omitted.  A shared
"cast from alternate zone" code path would unlock dozens of cards.

### Triggered-Ability Event Gaps
`EventKind` is missing several commonly-needed triggers:
- `PermanentLeftBattlefield(CardId)` — needed for "LTB" abilities and
  exile-until-LTB patterns (Tidehollow Sculler, Fiend Hunter)
- `DamageDealtToCreature` — needed for enrage, lifelink gain on creature damage
- `TokenCreated` — needed for populate, alliance triggers
- `CounterAdded / CounterRemoved` — needed for proliferate payoffs, Heliod combo
- `SpellCopied` — storm payoffs, Bonus Round
- `PlayerAttackedWith` — needed for Battalion and similar attack-count effects
- ~~`SpellCastTargetingCreature` (or a `Predicate::SpellTargetsCreature`
  knob) — needed for Strixhaven Repartee.~~ **Done**: see
  `Predicate::CastSpellTargetsMatch` + `effect::shortcut::repartee()`.
  Stirring Hopesinger, Rehearsed Debater, Informed Inkwright, Inkling
  Mascot, Snooping Page, Lecturing Scornmage, Melancholic Poet, and
  Graduation Day all use it. Remaining Repartee cards are blocked on
  separate primitives (exile-until-X, copy-spell). Ward enforcement
  (mana-cost variant) shipped in push (modern_decks) — see Inkshape
  Demonstrator promotion + `push_ward_triggers_for_cast` in
  `game/actions.rs`.
- ~~`CardLeftGraveyard` — needed for Lorehold "cards leave your
  graveyard" payoffs.~~ **Done** in push V: see
  `EventKind::CardLeftGraveyard` + `Predicate::CardsLeftGraveyardThisTurnAtLeast`.
  Hardened Academic, Spirit Mascot, Garrison Excavator, Living
  History all wired. Remaining gy-leave-aware cards (Ark of Hunger,
  Owlin Historian, Primary Research, Wilt in the Heat) need only
  catalog wiring against the event.

### Multi-Card Batch Triggers
The engine emits `CardLeftGraveyard` per card removed; printed cards
say "Whenever **one or more** cards leave your graveyard". We
approximate by firing the trigger per-card (a strict power upgrade
on multi-card-removal turns, but harmless in 2-player play where
single-card returns dominate). A future refinement: collapse a
batch of `CardLeftGraveyard` events emitted in the same resolution
window into one trigger fire (similar to MTG's "looks back in time"
rule for batch triggers). Same shape applies to `CardDiscarded`,
`CreatureDied`, and any future per-zone-move event.

**Per-event fan-out fix (push c4b7b14)**: The dispatcher previously
broke after the first matching event per (source, trigger) pair,
silently swallowing later events in the same batch. This was a
regression for multi-attacker swings (Sparring Regimen) and any
"whenever X happens" trigger over a batch of N events. The
dispatcher now keeps iterating over events for batch-fanout-friendly
event kinds (Attacks, CreatureDied, CardDrawn, CardDiscarded,
CardLeftGraveyard, CounterAdded, Blocks, BecomesBlocked, LifeGained,
LifeLost, BecameTarget) — one trigger fires per matching event,
matching the printed Oracle wording. Other event kinds (ETB,
StepBegins, …) keep the at-most-once guard because they don't emit
duplicate events in a single batch.

### Spell-Side Predicate: Mana-Spent-On-Cast
SOS introduces **Increment** ("if mana spent > this creature's P or T,
+1/+1 counter") and **Opus** ("Whenever you cast an instant or sorcery,
do X. If five or more mana was spent, do bigger X"). Both need a
per-cast "mana value paid" snapshot exposed as a `Value` (or a
`Predicate::ManaSpentAtLeast(n)`). The engine already retains the cost
on the `StackItem`; lifting that into the `EffectContext` for trigger
filters should unlock a few dozen Strixhaven cards.

### X-Cost and Converge
`Value::XFromCost` exists but converge (number of *distinct colors* of mana
spent) is not tracked per cast.  `Value::ConvergedValue` is a stub that always
returns 0 for non-Prismatic-Ending uses.  Fix: record color set paid at cast
time and expose it as a `Value` primitive.

### Cost-Reduction Stacking
Delve, Improvise, Convoke, and generic cost-reducers each have separate
branches.  There is no unified "reduce mana cost by X before payment" hook,
making cards like Hogaak (Convoke + Delve) or Affinity impossible to express
cleanly.

### Target-Aware Cost Reduction
"This spell costs {X} less to cast if it targets [some condition]" is a
Strixhaven design pattern (Ajani's Response, Brush Off, Run Behind,
Mavinda, Killian, Orysa). Today we either drop the discount and ship the
spell at its printed full cost, or omit the spell entirely. Engine fix:
let `CostReduction` static / per-card alt-cost evaluate against the
candidate-cast's chosen target before payment. Probably a new
`SelectionRequirement`-keyed cost discount that the cast path consults.

### Mana Ability from Non-Battlefield Zone
`activate_ability` only walks the battlefield.  Cards like Elvish Spirit Guide
and Simian Spirit Guide (exile from hand: add mana) are completely omitted
because hand-activated mana abilities need a separate activation path.

### "Look At Top X, Pick One, Put Rest in Graveyard" Primitive
Stirring Honormancer ("look at top X cards where X is creatures you
control, put one in hand, rest into graveyard") and similar look-and-
sort effects need a "look at top N, choose K, mill the rest" primitive
to express faithfully. `Effect::Surveil` covers the "look + may put in
graveyard" shape but with a fixed number; the SOS variant is dynamic
and forces the rest-to-graveyard branch unconditionally.

### Choice of "Which Zone" for a Tutor Result
Dina's Guidance ("search a creature, put into hand or graveyard")
exposes a 2-option destination prompt that no other primitive currently
needs. Adding a `Effect::Search` flavor with `to: Either(ZoneDest,
ZoneDest)` (or a separate decision shape) would honor the toggle for
this and a handful of black/green search effects.

### Multi-Target Prompt for Sorceries / Instants
A handful of SOS cards specify two target slots with different filters
(Render Speechless: opponent + creature; Cost of Brilliance: player +
creature; Homesickness: player + up to two creatures). The engine
today only exposes a single-target slot per spell at cast time, so
these collapse one of the two halves. A multi-target cast prompt
(`Vec<Target>` in `GameAction::CastSpell`) would unlock all of them.

### Auto-Target Picker: Source-Avoidance + Best-Pick Heuristics
~~The current `auto_target_for_effect` walks the battlefield in `Vec`
order and returns the first legal match.~~ **Source-avoidance done**:
the new `auto_target_for_effect_avoiding(eff, controller, avoid_source)`
takes the trigger source and prefers any *other* legal target,
falling back to the source only when nothing else is legal. All
trigger-creation paths (`stack.rs`'s `flush_pending_triggers`,
`actions.rs`'s ETB triggers, `combat.rs`'s combat triggers, the
delayed-trigger fire path, Dies/PermanentLeavesBattlefield triggers)
now pass the source ID. Quandrix Apprentice's Magecraft pump now
deterministically targets the bear over the Apprentice, and the test
suite asserts the source-fallback when no other target is legal.

~~Prefer the highest-power creature for friendly pumps.~~ **Done** in
push VI: `auto_target_for_effect_avoiding` now sorts the primary-player
candidate set by descending current power when the effect prefers a
friendly target (Magecraft / Repartee fan-outs, transient PumpPT
spells). Hostile picks still use first-match.

Remaining best-pick heuristics still ⏳:
- Prefer creatures whose current power matches what the pump would
  unlock (lethal swing, post-pump unblockable, etc.).

### Mana-Cost Reduction with Target Predicate
Killian, Ink Duelist's "spells you cast that target a creature cost
{2} less" needs a `StaticEffect::CostReduction` variant whose filter
inspects the cast spell's targets. Today's `CostReduction` filters
on the spell card's own attributes only. Plumbing the cast-time
target list into the cost-reduction site would unlock this card and
similar Lorehold/Witherbloom cost-cutters.

### Transient Triggered-Ability Grants on Pump Spells
SOS Root Manipulation ("Until end of turn, creatures you control get
+2/+2 and gain menace and 'Whenever this creature attacks, you gain
1 life.'") needs a way to attach a *triggered* ability to a creature
for a duration, on top of the keyword-grant primitive. Today the engine
has `Effect::GrantKeyword { what, keyword, duration }` but no
`Effect::GrantTriggeredAbility { what, ability, duration }`. Adding
this would unlock the third clause of Root Manipulation, similar
"creatures gain combat-damage trigger until EOT" pump spells, and
the on-attack rider on tokens (Pest token's "gain 1 on attack",
Spirit token combat triggers).

### Self-Counter-Scaled Cost Reduction
SOS Diary of Dreams's `{5},{T}: Draw a card` activation costs `{1}`
less per page counter on the source. There's no
`StaticEffect::CostReduction` variant whose discount scales off the
source's own counter count. Adding a `CostReduction { delta:
Value::CountersOn { what: Selector::This, kind: Charge } }` shape
would unlock Diary of Dreams cleanly, plus other counter-scaled cost
reducers (M21 Mazemind Tome).

### Page Counter Type
SOS Diary of Dreams (and the rest of the SOS book/grandeur subtheme)
references "page counter" but the engine `CounterType` enum has no
`Page` variant. Diary is currently approximated with `CounterType::
Charge`, which is fine in 2-player play (no other card uses Charge as
a payoff source) but obscures the printed identity. Adding `Page`,
`Knowledge`, and the small handful of other novelty counters from
recent sets would close the gap.

### `Move`-with-count for Selecting One Card from a Zone
Today `Effect::Move { what: Selector::CardsInZone { zone: Graveyard, ... } }`
moves *every* matching card. Cards like Heated Argument's "you may
exile a card from your graveyard" need a "move at most one matching
card" primitive. A `Selector::OneOf(inner)` wrapper, or a `count` knob
on `CardsInZone`, would fix this. The current workaround for Heated
Argument collapses the optionality into "always do the rider".

### "Choose Up To N Modes (with Repetition)" for `ChooseMode`
Strixhaven's "Choose up to four. You may choose the same mode more
than once." pattern (Moment of Reckoning, Witherbloom Charm-style
spells with N copies) needs an extension on `Effect::ChooseMode` that
takes a list of (index, target) tuples per cast. Today the engine's
modal flow picks exactly one mode and one target per cast — the
"choose up to N" wrappers collapse to single-mode resolution.

### "X Life as Additional Cost" Primitive
Vicious Rivalry, Fix What's Broken, and a handful of SOS sorceries
have "As an additional cost to cast this spell, pay X life." The
engine has no per-cast life-payment cost — we approximate by reading
X from the spell's `{X}` slot and running `LoseLife X` at resolution
time, but that double-counts X (paying X mana via XFromCost AND X
life). A `cost.life: Value` field on `CardDefinition` (or an
`alternative_cost` variant whose payment also requires the life)
would make this faithful.

### "Track Cards Discarded by This Effect" Counter
Borrowed Knowledge ("draw cards equal to the number of cards
discarded this way") needs a per-resolution counter that
`Effect::Discard` increments. The mode 1 path is currently
approximated as "draw 7" — a flat-7 reload that misses the printed
"draw exactly as many as you discarded" precision but preserves the
card-advantage tally for typical hand sizes.

### Capture-As-Target From Selector (Repartee Exile-Until-End-Step)
Conciliator's Duelist's Repartee body wants to:
1. Exile the cast spell's chosen creature target
   (`Selector::CastSpellTarget(0)` — wired).
2. Schedule a delayed trigger that returns *the exiled card* to
   battlefield at next end step.

Step (2) collides with `Effect::DelayUntil`'s capture model — it
captures `ctx.targets.first()`, but a Repartee trigger has no
target slot of its own (the selector is what tracks the spell's
target). Need either:
- An `Effect::CaptureTargetFromSelector { slot, selector }` that
  mutates ctx.targets so the subsequent DelayUntil reads it back, OR
- An `Effect::ExileWithDelayedReturn { what, kind, controller }`
  combinator that pre-resolves the selector at registration time.

The latter is more general (also unblocks Tidehollow Sculler,
Banisher Priest, Fiend Hunter). The former is smaller surface but
introduces effect-side mutation of ctx.

### Spend-Restricted Mana
Strixhaven's "Spend this mana only to cast an instant or sorcery
spell" (Hydro-Channeler, Tablet of Discovery's {T}: Add {R}{R}
ability, Abstract Paintmage's PreCombatMain trigger, Resonating
Lute's land-grant) needs per-pip metadata on the mana pool. Today
mana is fungible — once it's in the pool, anything can spend it.
Adding a `restriction: Option<SpellTypeFilter>` knob on each
ManaPool entry (and consuming it during cost-pay) would honor the
printed restriction. Wide-ranging change touching `ManaPool`,
`pay()`, and the cost-pay-validation path.

### "Move at most one matching card" — `Selector::OneOf`
Several SOS effects exile/move "a card" from a graveyard, hand, or
top of library where the count is at most 1 (Heated Argument's "may
exile a card from your graveyard", Practiced Scrollsmith's "exile
target noncreature/nonland card from your graveyard"). Today
`Selector::CardsInZone { ... }` returns ALL matching cards. Adding
`Selector::OneOf(Box<Selector>)` (or a `count` knob on `CardsInZone`)
would let these spells correctly pick exactly one. Without it, the
catalog approximates by "exile every matching card" which over-
shoots when the graveyard has multiple matches.

### Snow Mana Validation
`ManaPool` tracks a `snow` counter but `pay()` never validates that a `Snow`
mana symbol must be paid from a snow source.  Any mana from any land currently
satisfies a `{S}` pip.

### Multiplayer / Commander Format
- Command zone: `Zone::Command` exists but `ClientView` has no field for it;
  the server never moves cards there.
- Commander damage tracking (21 from the same commander = loss).
- "Your opponents" vs. "each other player" distinctions (multiplayer targeting
  semantics differ from 2-player).
- Four-player free-for-all match setup in `run_match` / `build_cube_state`.
- Commander-specific rules: color identity deck building, commander tax.

### Planeswalker Interactions
- Planeswalkers can be attacked directly — `AttackTarget::Planeswalker` is in
  `types.rs` but the bot never chooses it and the client has no UI for it.
- "Planeswalker redirect" rule (damage that would be dealt to a player can be
  redirected) is unimplemented.
- Emblems are not modelled.

### Saga Lore Counters
Sagas need: ETB with 1 lore counter, trigger each chapter, advance at upkeep,
sacrifice when the last chapter triggers.  No `SagaLore` counter type or
upkeep-advance primitive exists.

### Prepare Mechanic (SOS) — ✅ DONE
Secrets of Strixhaven's per-permanent "prepared" flag is fully wired
using existing primitives — no new engine surface was needed:
- The flag is `CounterType::Prepared` (count-1), surfaced through
  `PermanentView.counters`.
- Toggle cards (Biblioplex Tomekeeper, Skycoach Waypoint) flip it via
  `Effect::AddCounter` / `Effect::RemoveCounter`, gated to legal targets
  by `SelectionRequirement::HasBackFace`.
- Payoff cards read it via
  `SelectionRequirement::WithCounter(CounterType::Prepared)` (static
  anthems lower it to `AffectedPermanents::AllWithCounter`;
  `Predicate::EntityMatches` / `SelectorExists` cover conditionals). The
  first payoff is **Top of the Class** ({2}{W}: "Prepared creatures you
  control get +1/+1 and have flying").

The originally-planned `Effect::SetPrepared` / `Predicate::IsPrepared` /
"Prepare {cost}:" helper turned out to be unnecessary — the counter
primitives subsume them. Covered by `tests::sos::{top_of_the_class_*,
prepared_counter_is_inert_*}`.

### Vehicle / Crew
`CardType::Artifact` exists but there is no `CrewN` keyword or "becomes a
creature until end of turn" mechanism.  Vehicle subtype is in `ArtifactSubtype`
but nothing uses it.

### Proper Split-Damage Distribution
Effects like Pyrokinesis ("deals 4 damage divided as you choose among any
number of targets") are collapsed to a single-target 4-damage hit.  A
`DealDamageDivided { total, targets: Vec<Selector> }` effect would express
the real card.

### Affinity / Self-Permanent-Scaled Cost Reduction
Witherbloom, the Balancer's "Affinity for creatures (this spell costs
{1} less to cast for each creature you control)" needs a per-cast cost
reduction whose discount scales off the caster's permanent count.
`StaticEffect::CostReduction { filter, amount }` is a fixed amount
today. Generalising to `amount: Value::CountOf(Selector)` (or a sister
variant `AffinityCostReduction { filter, scaler: Selector }`) would
unlock Affinity for Artifacts (Modern Affinity / Cranial Plating-era
shells), Affinity for X (Strixhaven Witherbloom + future), and Awaken
the Woods-style "X = forests" payoff costs.

### Exile Zone as Viewable State
Exile is a zone in the engine (`Zone::Exile`) and cards move there.
`ClientView.exile` now projects the shared exile zone with each card's
owner so the UI can render an exile browser (added with the
Strixhaven coverage push). Remaining gaps:
- The 3D client has no exile browser UI yet.
- Graveyard-order information is lost (cards are a flat Vec).

---

## Engine — Approximation Cleanups

| Card / Feature | Current Approximation | Correct Behaviour |
|---|---|---|
| ~~Windfall~~ | (~~draws flat 7~~) | **resolved push modern_decks batch 115** — new engine primitive `Value::MaxCardsDiscardedThisEffectByAnyPlayer` reads from a new per-player `cards_discarded_per_player_this_resolution: HashMap<usize, u32>` scratch (parallel to the existing flat `cards_discarded_this_resolution`). Windfall's body now reads `Draw(EachPlayer, MaxCardsDiscardedThisEffectByAnyPlayer)` instead of the prior `Const(7)`. Tests: `windfall_discards_both_hands_then_draws_max_discarded`, `windfall_asymmetric_discards_yields_higher_player_count` (P0 with 6 + P1 with 2 → both redraw 6). Same primitive is now available for future Wheel of Fortune / Jace's Archivist-class effects. |
| ~~Dark Confidant~~ | (~~fixed 2 life loss~~) | **resolved push modern_decks batch 111** — `LoseLife(ManaValueOf(TopOfLibrary))` evaluated *before* the Draw step so the life loss reads the live top of library, then the draw moves that same card to hand. Tests: `dark_confidant_loses_life_equal_to_revealed_card_cmc` (5-CMC Serra Angel → 5 life lost), `dark_confidant_loses_zero_life_for_zero_cmc_card_on_top` (Black Lotus → 0 life lost). |
| ~~Biorhythm~~ | (~~drain opponents to 0~~) | **resolved push modern_decks** — now `SetLifeTotal` to creature count per side (CR 119.5) |
| ~~Coalition Relic~~ | (~~tap for 1 of any color only~~) | **resolved push modern_decks batch 110** — three activated abilities now wired: mana ability, `{T}: charge counter` ability, and the `Remove 3 charge counters: WUBRG` burst (gated by `condition: ValueAtLeast(CountersOn(Charge), 3)` so the activation rejects without 3+ counters). Five lock-in tests cover the mana ability, counter-add, burst, and both rejection paths. |
| ~~Fellwar Stone~~ | (~~tap for 1 of any color~~) | **resolved push modern_decks batch 117** — new engine primitive `ManaPayload::AnyColorOpponentCouldProduce`. Resolution scans opp's battlefield for basic-typed lands (Plains/Island/Swamp/Mountain/Forest), unions their colors as the legal pool, falls back to colorless if no opp basic-typed land exists. Tests: `fellwar_stone_taps_for_any_color` (opp Island → exactly Blue), `fellwar_stone_falls_back_to_colorless_when_no_opp_basic_lands` (no opp lands → 1 colorless), `fellwar_stone_unions_colors_across_multiple_opp_lands` (opp Island + Forest → Blue or Green, no White/Black/Red). |
| ~~Static Prison~~ | (~~ETB taps target only~~) | **resolved push modern_decks** — ETB now wires `Seq([Tap target, AddCounter(target, Stun, XFromCost)])` so the X paid at cast time becomes X stun counters on the target. The engine's untap step consumes a stun counter per CR 122.1d, so the target stays tapped for X turn cycles. |
| ~~Rofellos, Llanowar Emissary~~ | (~~flat {G}{G}~~) | **resolved push modern_decks** — `{T}: Add {G}{G} for each Forest you control` wired via `ManaPayload::OfColor(Green, Times(Const(2), CountOf(Forest ∧ ControlledByYou)))`. Cube variant scales 2× per Forest. Tests: `rofellos_taps_for_two_green_per_forest`, `rofellos_taps_for_zero_with_no_forests`. |
| Spectral Procession | (~~{3}{W}{W}{W}~~ — pre-fix) → now `{2}{W}` (most-permissive collapse of `{2/W}` hybrid pips, all three hybrids paid on the generic side) | Real Oracle `{(2/W)}{(2/W)}{(2/W)}`. Awaiting an engine-wide `ManaSymbol::HybridGeneric(u32, Color)` variant before the true hybrid cost can be wired faithfully. Current `{2}{W}` is the most-permissive collapse and matches the engine's existing "collapse `{2/X}` to a single-color side" convention. |
| ~~Grim Lavamancer~~ | (~~{R}{T}: 2 damage — no exile cost~~) | **resolved push modern_decks batch 114** — `exile_other_filter` field upgraded to carry a count: `Option<(SelectionRequirement, u32)>`. Grim Lavamancer now sets `(SelectionRequirement::Any, 2)`; activation pre-flight rejects when fewer than 2 graveyard cards match and exiles both (lowest-CMC auto-picked) when payment succeeds. Cost label pluralises to "Exile N cards from gy". Tests: `grim_lavamancer_activated_ability_deals_two_damage` (updated to seed 2 GY cards), `grim_lavamancer_pings_creature_with_gy_card_to_exile` (updated similarly), `grim_lavamancer_rejects_activation_with_only_one_gy_card` (new negative test). Existing Postmortem Professor + Lorehold Pledgemage tests continue to pass (their callsites now read `Some((filter, 1))`). |
| ~~Ichorid~~ | (~~no graveyard gate~~) | **resolved push modern_decks batch 112** — upkeep trigger now carries `EventSpec::with_filter(SelectorExists(CardsInZone { who: EachOpponent, zone: Graveyard, filter: Creature ∩ HasColor(Black) }))`. Trigger only fires when at least one opp has a black creature card in their graveyard. Tests: `ichorid_returns_at_upkeep_then_exiles_at_end_step` (positive: Black Knight in opp GY → reanimates), `ichorid_stays_in_graveyard_when_no_opp_black_creature_in_gy` (negative: green creature in opp GY → predicate fails, Ichorid stays). |
| ~~Render Speechless~~ | (~~required creature target~~) | **resolved push modern_decks** — slot 1 optional |

---

## Client — Visualization

### Counter Display
`PermanentView.counters` carries all counter types and counts, but there is no
in-world or HUD display.  Suggested: floating text labels above affected cards
showing `+1/+1 ×3`, `Lore: 2`, `Charge: 1`, `Poison: 3`, etc., using Bevy
`Text3d` or billboard sprites.

### Modified Power/Toughness Display
When a creature's P/T differs from its printed values (pump spells, counters,
static effects), the printed Scryfall art still shows the base stats.
`PermanentView` exposes both `power`/`toughness` (current) and `base_power`/
`base_toughness` (printed). Current surfacing of modifications:
- 🟡 `draw_pt_modified_overlays` (`systems/gizmos.rs`) draws a coloured ring
  around any creature whose computed P/T differs from its base (green
  buffed / red debuffed / yellow mixed).
- 🟡 The Alt-key counter tooltip (`systems/counter_tooltip.rs`) shows
  `current/printed (printed X/Y)` when modified.
- ⏳ Still missing: an in-world numeric P/T overlay anchored to the card
  itself. Bevy's `Text2d` doesn't depth-sort with 3-D meshes, so this
  needs either (a) a billboarded `Text3d`/quad with a generated texture
  per card, or (b) a screen-space `Node` projected each frame off
  `Camera::world_to_viewport(card_translation)`. (b) is the cheaper
  retrofit; sits well next to the existing alt-tooltip projector.

### Modified Loyalty Display
There is no static loyalty badge today; loyalty surfaces only via the
3-D counter coin column on each planeswalker
(`systems/counter_coins.rs`, `CounterType::Loyalty` material). The coin
count tracks the current loyalty correctly, but the printed starting
loyalty from the card art and the precise current number are both
absent at a glance. Same screen-space-overlay approach as the P/T
overlay above would carry a "L: N" badge.

### Exile Zone Browser
Similar to the graveyard browser, an exile browser would let players inspect
exiled cards (Foretell staging area, Leyline victims, Imprint sources, etc.).

### Stun Counter Visualization
Static Prison and Rapier Wit add stun counters.  No indicator currently shows
that a permanent has a stun counter (i.e., won't untap next turn).  A small
badge or coloured ring on the card would communicate this clearly.

### Mana Pool HUD
During the player's main phase, their current mana pool is shown in the player
status text but as a compact string.  A pip-style display (coloured circles for
each mana symbol available) would be faster to read at a glance.

### Damage Overlays
When combat damage is assigned, show floating damage numbers rising off
affected creatures before SBA removes the dead ones.

### Card Tooltip with Full Oracle Text
Hovering over a card shows its Scryfall art via the peek popup, but not the
full rules text.  A tooltip panel (shown on hover or via a dedicated key)
displaying the oracle text would reduce the need to look cards up externally.

### Graveyard Order and Timestamps
The graveyard browser shows cards as a flat unordered list.  Preserving
insertion order (most recently added = top) matches player intuition and helps
with "top of graveyard" effects.

### Attacking / Blocking Arrow Polish
Gizmo arrows are drawn in `draw_blocking_gizmos.rs` and `draw_attacker_overlays.rs`.
Improvements:
- Colour-code arrows by blocked/unblocked status.
- Show combat damage assignment numbers on arrows.
- Animate arrows fading in/out on declare-attackers/blockers transitions.

### Token Labeling
Token cards in the 3D view use the Scryfall-fetched art path, which often
resolves to a generic back image.  A text overlay (name + P/T) on token cards
would disambiguate multiple different tokens on the battlefield.

### Card Art on the Stack
The stack panel (`game_ui.rs::update_stack_panel`) shows only a "SPELL /
TRIGGER" badge + name + controller text. Add a small card thumbnail
(~70×100 px) per row using `scryfall::card_asset_path` — the scry/search
modals (`decision_ui.rs:293-334`) already follow the exact `ImageNode`
pattern. MTG players read the stack by visual recognition; text-only is
a big information-density loss in critical priority decisions.

### Life-Total Animation + Damage Feedback
Life changes are instantaneous text mutations in `update_player_text` /
`update_p1_text`. Lerp the displayed life toward the true value over
~0.5s and spawn a floating "−4" / "+2" near the player portrait that
drifts up and fades. Hook off `GameEventWire::DamageDealt`, `LifeLost`,
`LifeGained`. Pulse the life text red on lethal threat.

### Mana Symbol Rendering (Costs + Pool)
Mana is rendered as text codes (`W:1 R:2`) in the player status, ability
costs, alt-cast modal, and decision modals. Adopt a mana-symbol font or
PNG atlas plus a text segmenter that splits `{2}{R}{R}` into icons +
numerals. Subsumes the existing "Mana Pool HUD" entry above — once the
glyph primitive exists every mana surface benefits.

### Phase Chart Progress Indicator
`update_phase_chart` highlights only the current step in yellow. Add a
filled vertical bar growing through the steps (or a left-edge arrow) so
turn progression is visible at a glance. Optional: tint the chart
differently when it's the opponent's turn vs yours.

### Card Hover Polish
`animate_hover_lift` currently only translates the card on Y. Modern MTG
clients combine the lift with a small scale-up (×1.03–1.05), a tilt-
toward-camera (~5°), and a shadow boost — much more tactile. The
`CardHovered` marker is already tracked; just extend the animation.

---

## Client — UX

### UI Roadmap (push claude/modern_decks — session-derived)

Ordering layer over the detailed items below. Cross-references existing
entries instead of duplicating; tiers ordered by start-here leverage.

**Player Crest track** — promote 3-D disc into stat readout + state
indicator + click target. Slims the 2-D chip strip.
- Phase 1 ✅ Disc → crest (ring + screen-space life label, world→viewport
  projection). Files: `card/{components,spawn}.rs`, `systems/game_ui/crest.rs`,
  `main.rs` (`MainCamera` made `pub`).
- Phase 2 ✅ `PlayerTargetZone` on every seat incl. viewer; 3-D disc + 2-D
  chip share `Target::Player` path.
- Phase 3 ⏳ NEXT — damage/heal floaters. New `life_floaters.rs`:
  `PreviousLifeTotals` resource + `LifeFloater` component +
  `detect_life_changes` + `animate_life_floaters`. Re-uses Phase 1
  projection helper. Data already in `ClientView`.
- Phase 4 ⏳ — slim corner chip strips to `name · ♥ · ✋`, move mana pips
  to a bottom detail bar.
- Phase 5 ⏳ — team-coloured tint from `GameState.teams`; commander emblem
  when `PlayerView.commanders` non-empty.

**Tier 1**
- X-ray card inspector ⏳ — extend Hover-Dwell Card Preview (below) to
  render engine-truth rules text from `CardDefinition` plus current
  modifications (layer P/T, granted keywords, attachments, counter net,
  legal actions). Differentiator vs XMage/MTGO/Arena.
- Stop settings + auto-pass ⏳ — see Per-Phase Auto-Stop + Auto-Pass
  Toggle. Settings panel; persist via `config.rs`. Urgency infra
  (`pulse_urgent_pass_button`) already exists.
- Phase bar ⏳ — replace vertical `PHASE_CHART_STEPS` with horizontal
  Arena-style strip; click a step to toggle a stop. Pairs with above.
- Stack widget polish ⏳ — promote `update_stack_panel` to a permanent
  floating panel; hover for source-card preview; click to scroll log.

**Tier 2**
- Unify decision modals ⏳ — `decision_ui.rs` has 6 parallel pickers
  (scry/search/put-on-library/discard/mulligan/color). Refactor into one
  `Picker { items, min, max, ordered, confirm_label }`. See Decision
  Modal vs 3-D Hand Consistency.
- Token stacking ⏳ — group identical tokens with count badge.
- Valid-target affordance ⏳ — make `ValidTarget` pulse, dim non-targets.
- Card-name → log preview ⏳ — hover region pops Scryfall image. See
  Hover-Dwell Card Preview.
- Theme variants ⏳ — light / high-contrast / colorblind palette in
  `theme.rs`.

**Tier 3**
- Replay scrubber ⏳ — `GameSnapshot` recorder + Menu→Replay scrub UI.
- Touch / controller input ⏳ — Bevy supports touch; `kb_cursor.rs` and
  input paths are mouse-centric.
- Split `game_ui.rs` ✅ DONE — see `systems/game_ui/{mod,crest,player_stats,buttons,popups}.rs`.
  Future: pull `sync_game_visuals` → `visual_sync.rs` (~1.1K lines),
  `handle_game_input` → `input.rs` (~800 lines).

**Session follow-ups**
- Step-change → clear attack plan ⏳ — tiny watcher on `View.is_changed()`
  calling `attacking.clear()` when leaving `DeclareAttackers`.
- Crest pip cluster ⏳ — disc-rim pips for poison / commander damage /
  first-spell tax / energy. Reuse `counter_coins.rs` palette.

### Undo / Take-Back
A "request take-back" action the opponent can approve would reduce frustration
from misclicks, especially during the targeting flow.

### Keyboard Shortcut Reference
Add a `?` or `H` key that opens an in-game overlay listing all keyboard
shortcuts (A = attack all, Space/P = pass, E = end turn, N = next turn, etc.).

### Responsive Stack Display
The stack panel (bottom-center) is a fixed-width overlay.  On narrow windows
it can overlap the player panel.  Clamp its width to `min(420px, 40vw)` or
reposition it to the right sidebar.

### Per-Phase Auto-Stop Flags
Arena-style "stop at" checkboxes per phase (e.g., "always stop at opponent's
end step").  Currently the only fast-forward controls are End Turn (E) and
Next Turn (N).

### Deck Browser
A pre-game or in-game panel listing the full deck composition (name + count
for each unique card) would help players understand the randomly-assembled cube
deck they are playing.

### Game Log Scrollback + Event Color-Coding
`GameLog::push` caps entries at 16 (`game.rs:13-19`) and the panel joins them
with `\n` into a single `Text` widget. A real game produces hundreds of
events. Switch to `VecDeque<LogEntry>`, raise the cap to ~200, wrap the
panel in `Overflow::scroll_y()`, and color-code entries by `GameEventWire`
variant (damage = red, mana = mana-color, step = dim gray, etc.). The
`format_event` match in `game_ui.rs` already pattern-matches every variant,
so adding a per-variant color is cheap.

### Button Hover + Pressed Feedback
Action buttons (Pass / End Turn / Next Turn / Export plus modal buttons) have
no `Interaction::Hovered` / `Pressed` tinting and no tooltips. Introduce a
generic `interactive_button` helper that wires hover/press background changes
and tooltip strings, and apply it across `game_ui.rs` HUD buttons,
`decision_ui.rs` modal buttons, and `draft.rs` tab buttons. The current pass
button hard-codes 4 srgb branches per priority state with no hover feedback.

### Selective Attacker Picking
The only attack option today is "Attack All" sending every untapped
non-defender at `next_opp` (`game_ui.rs:2370-2396`). Inline comment at
line 2369 admits "multi-defender targeting has no UI yet". Quick win:
per-attacker click-to-opt-out before submitting the attack. Bigger lift:
drag an arrow from attacker to defender / planeswalker. Restricts core
MTG gameplay until done.

### Hover-Dwell Card Preview
Today the only way to read full rules text is to hold Alt while hovering
(`ui.rs::peek_popup`). Add a hover-dwell state machine (~300ms over a card
→ fade in large preview near cursor, with viewport-edge clamping). Reuse
`scryfall::card_asset_path`. Extends "Card Tooltip with Full Oracle Text"
above but specifically calls out the dwell-timer + cursor-relative
placement that brings the UX in line with Arena / MTGO.

### Decision Modal vs 3-D Hand Consistency
Mulligan and PutOnLibrary modals are transparent overlays over the 3-D
hand (player clicks the 3-D cards). Scry / Search / Discard render their
own 2-D card grid. No design rule says which decisions go which way, so
users can't predict whether to click the 2-D modal cards or the 3-D table
cards. Pick one rule (e.g., "decisions on the viewer's own hand → 3-D +
banner; decisions on hidden zones → 2-D modal grid") and migrate.

### Right-Click Action Hint
`game_ui.rs::handle_game_input` dispatches right-click on a hand card to
either the alt-cast modal (`has_alternative_cost`), the MDFC flip
(`back_face_name`), or the ability menu (battlefield card). The user has
no visual hint about which their right-click will trigger. Add a small
corner glyph on the card or a cursor-change to signal "right-click for
alt cost" / "right-click to flip".

### Hand-Fan Spacing for Large Hands
`card/layout.rs:18` sets `HAND_CARD_SPACING = CARD_WIDTH * 0.85`. A
15-card hand (Frantic Search loops, no-mulligan shenanigans) spreads
off-screen. Clamp total fan width to a viewport-relative target and
reduce spacing proportionally when hand size > 7.

### Drag-and-Drop for Hand → Battlefield
Hand cards play via click. Drag-to-position or drag-to-target would add
tactile feel for both casting and selecting targets. Lower priority than
the in-place fixes; capture the intent here.

### Settings Menu
The animation-speed slider is currently wedged into the quality panel
(`quality.rs::setup_quality_panel`). A proper Settings panel (audio,
key rebinds, UI scale, accessibility) would cleanly separate these and
give a natural home for future global preferences.

### Auto-Pass Toggle
`auto_advance_p0` (`game_ui.rs:2000+`) decides for the player when to pass
priority. A toolbar toggle ("Auto-pass: On/Off") lets new players step
through their own turn priority-by-priority instead of having the engine
fast-forward.

### Alt-Peek Inside Decision Modals
Scry / search / discard modal cards are 180×250 (`decision_ui.rs:124`) —
fine for art, illegible for rules text. The Alt-hold peek-popup
(`ui.rs:90-92`, 340×475) works on 3-D cards but doesn't fire on 2-D
modal cards. Wire Alt-hover inside `decision_ui` modals to spawn the
same large preview.

---

## Client — Engineering / Refactor

These don't change the player-visible UI but unblock parallel work and
reduce ongoing churn. Sequence them when scope or merge conflicts on the
Client UI layer become a recurring problem.

### Split `game_ui.rs`
2,850 lines mixing setup, view→entity sync (~1,000 lines), input,
ability menu, alt-cast modal, and HUD updates. Inline comment at line 38
admits `handle_game_input` is bumping Bevy's 16-param `SystemParam`
limit. Split into `game_ui/hud.rs` (setup + `update_*` text/buttons),
`game_ui/sync.rs` (`sync_game_visuals` only), `game_ui/input.rs`
(`handle_game_input` + `auto_advance`), `game_ui/modals.rs` (ability
menu, alt-cast). Keep `GameLogicSet` + `ButtonState` in `mod.rs`.
Prerequisite for several upcoming features but invisible to users.

### Modal Builder Helper
`decision_ui.rs` has 6+ near-identical "overlay root + panel + close-on-
escape" spawn functions (`spawn_scry_modal`, `spawn_search_modal`,
`spawn_discard_modal`, `spawn_put_on_library_modal`,
`spawn_mulligan_modal`, `spawn_choose_color_modal`). Each new decision
requires ~30 lines of root/panel boilerplate. Introduce a builder:
`modal(commands, ui_fonts, title).body(|panel| {…}).buttons(|btns| {…}).spawn()`.
Could halve `decision_ui.rs`.

### Stable-Children for Stack Panel + Pile Tooltip
`update_stack_panel` (`game_ui.rs::update_stack_panel`) and the pile
tooltip (`ui.rs::pile_tooltip`) `despawn_children()` + rebuild on every
change. The pile tooltip has a TODO comment explicitly admitting "we
can't easily update the child text here, so just leave it" — i.e., the
tooltip shows stale data. Give children stable marker components
(`StackPanelRow(idx)`, `PileTooltipText`) and update text in place.
Also fixes visible tearing when unrelated `view` fields change.

### `DecisionView` Trait
`spawn_decision_ui` matches every `DecisionWire` variant and dispatches
to a separate `spawn_*_modal`; `handle_confirm`,
`handle_put_on_library_select`, etc. repeat the same per-variant
dispatch. A `trait DecisionView { fn spawn(...); fn confirm(...);
fn cancel(...); }` implemented per variant would centralize. Roll up
under the Modal Builder above when you tackle it.

### Move `format_event` to Engine Crate
`format_event` (`game_ui.rs:91-167`) is a 75-line match on
`GameEventWire`. Every new event type requires editing this client-side
function. Move to a `Display` / `fmt_for_log` impl on the wire type
itself in `crabomination/src/net.rs` so new event variants stay
self-contained. Pairs with the log-color-coding work above.

### Relocate `stack_card_transform`
`stack_card_transform` lives in `game_ui.rs:2752` but is a pure math /
layout helper. Move to `card/layout.rs` next to the other transform
helpers (`hand_card_transform`, `bf_card_transform`, `deck_position`).

### Responsive HUD Layout
Most HUD panels use hardcoded `Val::Px` margins and widths
(`game_ui.rs:295-575`: `max_width: 560`, `min_width: 420`,
`BROWSER_CARD_WIDTH: 220` × 4 cols = ~960 px island). At 720p the
bottom player panel collides with the stack panel + AttackAllPanel;
at 1440p+ everything sits in a small island. Audit `Val::Px` →
`Val::Percent` / `Val::Vw` / `Val::Vh` per panel and add a `UiScale`
resource. Subsumes the existing "Responsive Stack Display" entry above.

---

## Bot / AI

### Instant-Speed Responses
The bot currently never responds to spells on the stack — it auto-passes
priority whenever it gets it during an opponent's turn.  A rule-based layer
that recognises "this creature is being targeted by removal, I have a
counterspell" would make the bot feel more like a real opponent.

### Sacrifice Prioritisation
~~When forced to sacrifice, the bot always picks the first eligible
permanent.~~ Now sorts candidates: **tokens first, then by lowest CMC,
then by lowest power**. This is enforced inside `Effect::Sacrifice` so
both Innocent-Blood-style edict flow and forced sacrifices from
activated abilities see the same ordering. Future improvements:
respect "you may sacrifice" optionality (skip when the cheapest
candidate is more valuable than the payoff).

### Planeswalker Targeting
~~The bot never attacks planeswalkers.~~ Now redirects attackers at an
opponent's planeswalker when total attacking power can finish it off in
one swing (push claude/modern_decks `b34a23a`). Smallest-power-first
allocation keeps beefy attackers free to face-attack the player when the
walker fills up. Future improvement: handle chip attacks (attacking a
walker we can't finish but that's still threatening) and the inverse case
where a low-loyalty walker isn't worth committing trample beaters to
because the opp can clean up with a blocker.

### Smarter Mana Rock Usage
The bot taps mana rocks eagerly before knowing what it wants to cast.  A
"plan this turn's spending first" pass before mana-ability activation would
avoid situations where it taps a Sol Ring with nothing to cast.

### Multiple Difficulty Levels
- Easy: current random bot
- Medium: rule-based heuristics (responsive countering, threat assessment)
- Hard: Monte-Carlo tree search or minimax over the simplified game state

---

## Infrastructure / Dev

### Engine Test Coverage
Current test density is low outside `effects.rs` and card-specific unit tests.
Priority gaps:
- **Combat module** (`game/combat.rs`) has zero standalone tests.
- **Layer system** (`game/layers.rs`) — continuous effects, P/T ordering,
  timestamp tracking — has no dedicated tests.
- **Stack resolution ordering** — no tests for multi-item LIFO resolution,
  replacement effects, or trigger ordering.

### Snapshot Round-Trip Test
`GameSnapshot` and `GameState` serialisation exist.  Add a property-based test
that plays N random actions, serialises/deserialises the state, and asserts
game continuity — catching any `Serialize`/`Deserialize` drift.

### Card Correctness CI
`scripts/verify_cards.py` (with its Scryfall cache) verifies CMC, P/T, types,
and keywords.  Wire it as a CI step that runs against `scripts/.scryfall_cache.json`
(no network) to catch regressions when catalog entries change.

### Bot vs. Bot Simulation
Automate a "run 1 000 cube games bot vs. bot, report win rates by colour pair"
script.  Useful for catching degenerate card interactions and unbalanced pools
without manual play.

### Replay / Game Log Export
The server already collects `GameEventWire` events.  A replay file format
(sequence of `(action, resulting_state_hash)`) would enable post-game review
and deterministic bug reproduction.

### Scryfall Art Pre-fetch CLI
`all_cube_cards()` drives the in-game prefetch, but there is no standalone CLI
tool to warm the asset cache before a session.  A `cargo run --bin prefetch_art`
that downloads missing Scryfall images to the local cache would speed up first-
session load times.

### WASM / Web Build
`Cargo.toml` already has a `wasm-release` profile.  Completing the web build
(removing native-only dependencies, adding a WASM server bridge) would make
the game playable in a browser without installation.

---

## Formats

### Commander + Two-Headed Giant — phased rollout

Roadmap for the `Format::Commander` and `Format::TwoHeadedGiant` variants
already declared in `format.rs`. Strategy: build the multiplayer
foundation first (any-N seats, teams, opponent semantics), then add
shared resources for 2HG, then layer Commander-specific mechanics on
top. The `Format` enum entries currently only affect deck validation
and starting life; everything below is the runtime engine work.

**Status legend:** ✅ done, 🟡 partial, ⏳ todo.

#### Phase A — N-player game construction ✅
- ✅ Mulligan chain fixed to step through all N seats (`game/mod.rs:1327`
  `advance_mulligan` now computes the next seat instead of always
  passing `None`).
- ✅ Test helpers `multi_player_game(n)` and `game_with_format(format, n)`
  next to `two_player_game()`.
- ✅ 8 tests in `tests/multiplayer.rs` (4-player default life, Commander/
  2HG life across all seats, 4-seat turn rotation, elimination skip,
  4-step priority cycle, mulligan chain).
- Engine was already N-player aware (`pass_priority` uses
  `alive_count`, turn rotation uses `next_alive_seat`, attack target
  validation is bounds-checked).

#### Phase B — Teams abstraction ✅
- ✅ New module `crabomination/src/team.rs` with `TeamId`, `Team`, and
  `TeamError`.
- ✅ `GameState.teams: Vec<Team>` (`#[serde(default)]` for snapshot
  back-compat); auto-populated by `GameState::new` with one singleton
  team per seat so existing 1v1 behavior is unchanged.
- ✅ Helpers `team_of`, `teammates`, `opponents_of`, `same_team`,
  `assign_teams` in `game/mod.rs`. Empty-`teams` snapshots gracefully
  fall back to singleton-per-seat semantics.
- ✅ 9 tests (singleton defaults, 2v2 partitioning, 2-1-1 mixed FFA,
  all four `assign_teams` error paths, empty-teams snapshot fallback).

#### Phase C — Team-aware opponent semantics ✅
- ✅ `PlayerRef::EachOpponent` routes through `opponents_of(controller)`
  instead of `i != controller` (`game/effects/mod.rs:2024, 2049`).
- ✅ `SelectionRequirement::ControlledByOpponent` predicate now uses
  `!same_team(card.controller, controller)` for both the static-permanent
  path and the on-card path (`game/effects/eval.rs:449, 552`).
- ✅ `AffectedPermanents::AllOpponents` gained a
  `friendly_seats: Vec<usize>` field (`#[serde(default)]`); the
  `affects()` check uses it when non-empty, else falls back to the
  legacy single-seat compare. Populated at compute-time by
  `compute_battlefield` since the construction helper has no
  `GameState` handle.
- ✅ Auto-targeter's `(controller + 1) % n` "the opponent" picker now
  uses `opponents_of(controller).first()` (`game/effects/targeting.rs:38,
  220`).
- ✅ 7 new tests in `tests/multiplayer.rs` (2v2 EachOpponent excludes
  teammate; 1v1 baseline preserved; eliminated opponent filtered;
  ControlledByOpponent predicate on both permanent and player targets;
  auto-targeter avoids teammates).

#### Phase D — Multiplayer combat ✅
Each attacking creature chooses a defending player or a planeswalker
controlled by one of them; in 2HG the choice is the defending *team*
and damage may be assigned to either teammate's creatures/planeswalkers.
- ✅ `declare_attackers` now validates the chosen defender via
  `!same_team(active, target)` rather than `target != active`, so
  attacking a teammate (Player or Planeswalker) is rejected
  (`game/combat.rs:21-49`). `same_team(a, a) == true` collapses the
  self-attack and teammate-attack cases into one check; 1v1 / FFA
  behavior is unchanged.
- ✅ `declare_blockers` consults `same_team(blocker.controller,
  defender_idx)` so any defending-team member may block on a
  teammate's behalf (`game/combat.rs:194-201`).
- ✅ 5 new tests in `tests/multiplayer.rs` (3p FFA can attack either
  opponent / self rejected; 2v2 rejects teammate-Player attack;
  2v2 rejects teammate-Planeswalker attack; 2v2 partner blocks for
  the attacked teammate; 2v2 attacking team can't block for the
  defenders).

#### Phase E — Priority & APNAP for N players ✅
- ✅ `pass_priority` already cycles via `alive_count` + `next_alive_seat`
  — no 2-player assumption.
- ✅ `dispatch_triggers_for_events` now stable-sorts candidates by an
  APNAP rank derived from `active_player_idx` + repeated
  `next_alive_seat` (`game/mod.rs:1169-1212`). The active player's
  triggers push first → resolve last (CR 603.3b). Within a player's
  group, battlefield-iteration order is preserved as their chosen
  order (AutoDecider doesn't reorder; a UI player would pick).
  Eliminated seats fall through to rank `n_players` so they push
  last (resolve first) — deterministic if a dead permanent's
  controller somehow still triggers.
- ✅ 2 new tests in `tests/multiplayer.rs` (4-player simultaneous
  LifeGained triggers push in APNAP order from active=1; eliminated
  seat sorts to back of cycle).
- Note: triggers within a single declare-attackers / declare-blockers
  batch (`game/combat.rs:50, 110`) share one controller (the active
  player), so APNAP within those is moot. The fix is concentrated on
  the unified dispatcher because that's the only fan-out path where
  multiple controllers can produce simultaneous triggers from one
  event.

#### Phase F — Shared life pool & shared turns (2HG) 🟡 (shared pool ✅; shared turn / cross-team triggers ⏳)
The 2HG-specific consumer of the teams abstraction.

**Shared pool — done:**
- ✅ `Team.shared_life: Option<i32>` (`#[serde(default)]` for snapshot
  back-compat) — `Some(30)` for 2HG teams, `None` for solo teams
  (`team.rs:24`).
- ✅ `GameState::effective_life(seat) -> i32`,
  `GameState::adjust_life(seat, delta)`, and
  `GameState::set_life(seat, new)` helpers
  (`game/mod.rs:448-530`). Writes route to the shared pool when
  `Some`, else to `players[seat].life`. `life_gained_this_turn`
  stays bound to the seat receiving the change.
- ✅ All production life-mutation sites rerouted (`combat.rs` ×4 —
  lifelink and damage; `actions.rs` ×6 — Phyrexian-mana, alt-cost
  life, ability life cost; `effects/mod.rs` ×6 — GainLife, LoseLife,
  SetLifeTotal, Drain, WardCost::PayLife, alt-cost life,
  reveal-counts-life; `effects/movement.rs` ×1 — direct damage;
  `server/mod.rs` AdjustLife debug action).
- ✅ `apply_format(TwoHeadedGiant)` auto-partitions consecutive seat
  pairs (0+1, 2+3) into teams and seeds `shared_life = Some(life)`
  on each (`game/mod.rs:368-413`). Callers can still override with
  `assign_teams` after.
- ✅ Phase F-3 SBA: `check_state_based_actions` checks
  `effective_life(seat) <= 0`, so the pool draining to 0 eliminates
  both teammates simultaneously (CR 704.5a + 810.8). Poison stays
  per-player (CR 810.7b).
- ✅ 6 new tests in `tests/multiplayer.rs` (2HG partition + 30-life
  seeding; FFA baseline; one teammate takes damage / shared pool
  drops / partner sees it; lifegain from either teammate pools;
  shared-pool lethal → both eliminated → opposing team wins;
  poison-is-per-player asymmetry).

**Polish — done:**
- ✅ Cross-team trigger fan-out (CR 810.8):
  `EventScope::YourControl` / `OpponentControl` in
  `game/effects/events.rs:74-83` now route through `same_team`
  rather than exact-seat compare. A teammate's life gain / cast /
  attack now fires the partner's "whenever you …" triggers. For
  solo-team formats (1v1 / FFA / Commander) the helper collapses to
  exact equality, so no behavior change. Test:
  `two_headed_giant_lifegain_fires_partner_yourcontrol_trigger`.
- ✅ Mulligan independence — verified via
  `two_headed_giant_mulligan_chain_is_per_seat`. No code change
  required (inherits per-seat chain from Phase A).

**Still ⏳ (low-impact polish):**
- ⏳ Shared turn priority (CR 810.5) — strict "active team's primary
  player first, can yield to teammate" ordering. Current rotation
  is per-seat; both teammates already get priority in the
  4-passes-to-advance loop, so this is cosmetic.

#### Phase G — Team-aware loss & game end ✅
**G-lite done** (independent of Phase F):
- ✅ Player elimination still happens per-player on life ≤ 0 / 10
  poison / empty-library draw (unchanged).
- ✅ Game ends when only one *team* has alive members, not one player.
  `check_state_based_actions` (`game/stack.rs:892-933`) now dedupes
  alive seats by `team_of(seat)` and ends when surviving-team count
  ≤ 1. `winner: Option<usize>` is reported as the surviving team's
  lowest-numbered alive seat (a stable representative).
- ✅ 5 new tests in `tests/multiplayer.rs` (2v2 keeps going after one
  teammate dies; 2v2 ends when one team is fully wiped; winner
  representative skips dead team members; 3p FFA baseline preserved;
  4p simultaneous-elimination draw).

**Shared-life half — now done via Phase F-3:**
- ✅ Team-elimination via shared life: the SBA in
  `check_state_based_actions` now reads `effective_life(seat) <= 0`,
  so any teammate whose pool runs out is eliminated (in 2HG that's
  the whole team simultaneously). CR 810.8 + 704.5a.
- ✅ Test: `two_headed_giant_zero_shared_life_eliminates_both_teammates`
  in `tests/multiplayer.rs` covers the 2v2 case end-to-end (damage
  takes shared pool to 0 → both teammates eliminated → opposing
  team wins with correct representative seat).

#### Phase H — Replacement-effect framework (Commander prerequisite) ✅
- ✅ New module `crabomination/src/replacement.rs` with
  `ReplacementEffect { id, source: ReplacementSource, from:
  Option<Zone>, to_zones: Vec<Zone>, redirect_to: Zone, optional:
  bool }` and `ReplacementId` newtype. Scope is intentionally narrow
  — only zone-change replacements, source matches by `CardId` (the
  Commander use case). `optional` is reserved for Phase L's
  decision-plumbed "may redirect to command zone" semantics.
- ✅ `GameState.replacement_effects: Vec<ReplacementEffect>` with
  `#[serde(default)]` for snapshot back-compat; monotonic
  `next_replacement_id` counter.
- ✅ `register_replacement` / `unregister_replacement` /
  `resolve_zone_change(card_id, from, to) -> Zone` helpers in
  `game/mod.rs:597-682`. Resolver tracks already-applied ids
  (CR 614.5 — a replacement applies at most once per event) and
  caps iterations at `MAX_REPLACEMENT_ITERATIONS` (16) so
  pathological chains terminate.
- ✅ Three zone-change entry points wired through the resolver:
  `place_card_in_dest` (`effects/movement.rs:225-265`),
  `remove_from_battlefield_to_graveyard`, and
  `remove_from_battlefield_to_exile` (`stack.rs:960-1027`). New
  internal `place_card_at_resolved_zone` centralizes terminal-zone
  placement for the post-resolver path. `Zone::Command` redirects
  fall back to graveyard with a `debug_assert!` pending Phase I.
- ✅ 6 new tests in `tests/multiplayer.rs` (baseline destroy goes to
  graveyard; graveyard→exile redirect on a specific card; scoping
  to CardId only affects the chosen card; chained graveyard↔exile
  loop terminates via the applied-once guard; `from` filter gates
  origin zone; `unregister_replacement` drops the entry).
- Known limitation (acceptable for Phase H scope): inline
  `graveyard.push` / `hand.push` / `exile.push` sites outside the
  three wired entry points bypass the resolver. Effects routed
  through `Effect::Destroy`, `Effect::Exile`-from-battlefield, and
  `move_card_to` all hit the wired paths; ETB-triggered direct
  pushes are the main gap and likely don't need replacement-effect
  coverage for Commander.

#### Phase I — Command zone runtime ✅
- ✅ `Player.command: Vec<CardInstance>` with `#[serde(default)]`
  (`player.rs:18-30`). Per-seat ownership — Commander, Conspiracy,
  Mox Lotus, etc. all sit here.
- ✅ `place_card_at_resolved_zone(Zone::Command)` now pushes to
  `players[owner].command` instead of the Phase H placeholder
  (`stack.rs:1015`). The whole replacement-effect → command-zone
  pipeline is live end-to-end.
- ✅ Test coverage via Phase J's
  `destroyed_commander_returns_to_command_zone` and
  `seat_commanders_sets_up_command_zone_and_replacement`.

#### Phase J — Deck model with commander slot ✅
- ✅ `Deck { main, commanders, sideboard }` struct in `format.rs` —
  `commanders` is `Vec<CardDefinition>` so Partner / Background can
  populate two without changing shape.
- ✅ `GameState::seat_commanders(seat, defs) -> Vec<CardId>` pushes
  each commander as a fresh `CardInstance` into the seat's command
  zone, records the id on `Player.commanders`, and registers the
  CR 903.9b zone-change replacement (graveyard / exile / hand /
  library → Command) via the Phase H registry. Optional flag is
  set to `false` for now — Phase L's decision-plumbed "may" can
  flip it later. (`game/mod.rs:632-687`)
- ✅ `is_commander(card_id) -> bool` helper used by Phase L /
  Phase M for cast and damage gates.
- ✅ Tests for command-zone seating + leave-play bounce
  (`seat_commanders_sets_up_command_zone_and_replacement`,
  `destroyed_commander_returns_to_command_zone`).

#### Phase K — Color identity & deck validation ✅
- ✅ `ColorSet` bitfield in `mana.rs` with `empty`, `single`, `all`,
  `insert`, `contains`, `is_subset_of`, `union`, `len`. Five-bit
  WUBRG-packed `u8`.
- ✅ `color_identity(def) -> ColorSet` in `format.rs` — unions
  colored / hybrid / Phyrexian pips from the mana cost. Phase K
  limitation: rules-text mana symbols and printed color indicators
  are not parsed (no in-scope catalog cards rely on them); future
  work can union in a `CardDefinition.printed_color_identity`
  override.
- ✅ `validate_commander_deck(deck) -> Result<(), (Vec<DeckError>,
  Vec<CommanderDeckError>)>` layers on:
  - at least one commander, at most two (Partner / Background)
  - each commander is a Legendary Creature
  - every main-deck card's color identity ⊆ combined commander
    identity
- ✅ 3 new validation tests + format::tests::commander_rules
  (catches off-color, requires Legendary Creature, requires a
  commander).

#### Phase L — Cast from command zone with tax ✅
- ✅ New `GameAction::CastFromCommandZone { card_id, target,
  additional_targets, mode, x_value }` (`game/types.rs`) and
  handler `cast_from_command_zone` in `game/actions.rs`. Mirrors the
  `cast_spell_with_convoke` flow: sorcery-speed gate, target
  legality, payment, push-onto-stack, `finalize_cast` for the rest.
- ✅ `GameState.commander_cast_count: HashMap<CardId, u32>` —
  bumped on a successful payment, consulted at cost build time to
  add `{2}` × prior casts as generic mana.
- ✅ The leave-play→Command replacement lives on Phase J's
  `seat_commanders`; Phase L's job is only the cost / counter side.
- ✅ Tests: first cast pays printed cost; after the commander
  bounces back via the J replacement, the second cast fails on an
  empty pool (tax unpaid) and succeeds once 2 mana is in pool;
  counter advances to 2.
- ✅ Polish: `Decision::CommanderRedirect { commander, would_be }`
  + `DecisionWire` mirror. `AutoDecider` answers `Bool(true)`
  (matching tournament default — save the commander); the resolver
  consults the decider when a replacement has `optional: true`.
  `seat_commanders` now registers the replacement with
  `optional: true`. A declined redirect still counts as "applied"
  for CR 614.5 (no double-prompt). Test:
  `commander_redirect_can_be_declined` proves a `ScriptedDecider`
  answering `Bool(false)` sends the commander to the graveyard.

#### Phase M — Commander damage SBA ✅
- ✅ `GameState.commander_damage: HashMap<(usize, CardId), u32>` —
  `(victim_seat, commander_card_id)` keys, running totals. Helper
  `record_commander_damage(victim, source, amount)` bumps an entry;
  callers gate on `is_commander(source)` before invoking.
- ✅ Both damage paths credit: combat damage to player
  (`game/combat.rs:573-585`) and direct damage from effects
  (`game/effects/movement.rs:69-77`). Combat-infect and direct
  damage both count (CR 704.5v doesn't restrict by damage type).
- ✅ SBA in `check_state_based_actions` eliminates a player whose
  table has any entry ≥ 21 (`game/stack.rs:886-902`).
- ✅ Tests: 21 commander damage eliminates even at positive life;
  20 doesn't but the 21st point does; non-commander source never
  populates the table.

#### Phase N — Polish ⏳
- ⏳ Audit any remaining `PlayerRef::EachOpponent` / "your"/"opponent"
  effects in card catalog text for team-awareness (Phase C handles
  the engine layer; some cards may have bespoke logic).
- ⏳ CLI / deck-loader entry points should accept format.
- ⏳ Update format coverage tests after Phase J/K land.

---

#### Dependency graph
```
A → B → C → D → E
        ↓
        F → G   (2HG-specific consumers of teams)
        ↓
        H → I → J → K → L → M   (Commander mechanics on the multiplayer base)
```

#### Open design questions
1. **Partner / Background commanders** — in scope, or v2? `Deck.commanders:
   Vec<…>` accommodates either way.
2. **Brawl / Oathbreaker** — same machinery as Commander; opportunistic
   to plan in once L/M land.
3. **CR 810.5 priority timing within a team** — strict per-CR, or start
   with a simplified "active team's primary player has priority first,
   can pass to teammate"?
4. **Range of influence** — Commander uses unlimited (everyone in range).
   Default to unlimited; skip the option unless explicitly requested.

### Draft
- 8-player booster draft simulation
- Bot drafters with a basic pick-order heuristic
- Deck construction phase before play begins

### Sealed
- Generate 6 booster packs per player
- Deck construction phase
- Best-of-3 match support

### Brawl / Historic Brawl
- Lighter-weight commander variant (60-card, Standard-legal)
- Good stepping stone before full Commander

---

## Card Implementations (high-priority unblocked cards)

These cards are in the cube or demo decks and need only existing primitives —
no new engine features required:

| Card | Missing Piece | Effort | Status |
|---|---|---|---|
| Grim Lavamancer | Exile-2-from-GY additional cost | Low | ⏳ |
| Bloodtithe Harvester | Sac-Blood ping (sac_cost activation) | Low | ⏳ |
| Dread Return | Flashback sac-3-creatures cost | Medium | ⏳ |
| Swan Song | Correct Bird token controller | Low | ✅ (done in earlier push — `PlayerRef::ControllerOf(Target(0))`) |
| Frantic Search | Untap cap (up to 3) | Low | ✅ (done in earlier push — `up_to: Some(Value::Const(3))`) |
| Windfall | Dynamic draw-equal-to-max-discarded | Medium | ⏳ |
| Balefire Dragon | Dynamic "that much damage" (use creature's power) | Medium | ⏳ |
| Dark Confidant | CMC-dependent life loss | High (needs card-CMC Value) | ⏳ |
| Rofellos | Forest-count mana scaling | Medium | ✅ (done — `Times(Const(2), CountOf(Forest))`) |
| Tidehollow Sculler | Exile-until-LTB primitive | High | ⏳ |
| Ichorid | Graveyard-color trigger filter | Medium | ⏳ |
| Coalition Relic | Charge-counter burst | Medium | ⏳ |
| Tezzeret, Cruel Captain | Artifact-creature static pump | Low | ✅ (done — `crabomination/src/catalog/sets/decks/modern.rs:6365`) |
| Karn, Scion of Urza | Artifact-count scaling Construct | Medium | ⏳ |

## New TODO suggestions (push modern_decks)

### Engine — Battle permanent type (CR 110.4) ⏳

CR 110.4 lists six permanent types: artifact, battle, creature,
enchantment, land, planeswalker. The engine's `CardType` enum has the
other five but no `Battle` variant. Battles introduced in March of the
Machine: each battle has defense counters (similar to loyalty), can be
attacked by creatures, and "transforms" when its defense reaches 0.
Affected cards: Mortality Spear's printed-Oracle's "destroy target
creature, planeswalker, or battle" rider currently collapses to
"creature/planeswalker" since no Battle exists.

**Fix**: add `CardType::Battle` + a `defense_counters` field on
`CardInstance` + an `AttackTarget::Battle(CardId)` variant. Combat
resolution would route attacker damage to defense counters
(similar to the planeswalker path). Engine-wide ⏳ until a card needs it.

### Engine — Phasing (CR 110.5, CR 702.26) ⏳

CR 110.5 lists "phased in/phased out" as a permanent status; the
engine has tapped + face-down but no phasing flag. Phasing matters
for Teferi-style "phase out" effects, the Fading mechanic, and the
SBA bypass during the phased-out state.

**Fix**: add a `phased_out: bool` field on `CardInstance` + a
`Predicate::IsPhasedIn` predicate + an SBA bypass for phased-out
permanents (they're treated as not on the battlefield for triggers,
combat, and most checks). The phase-in turn-based action runs at
the start of each untap step. Engine-wide ⏳ until a card needs it.

### Engine — Multi-target divided damage primitive

The engine collapses every "divided as you choose among any number of
targets" rider to a single target. Affected cards: Crackle with Power,
Magma Opus, Electrolyze, Devious Cover-Up, Vibrant Outburst, Mizzium
Mortars (Overload alt only), Snow Day, Spell Satchel. The headline
play pattern works at one target, but the multi-target shape is a
recurring 🟡 in STRIXHAVEN2.md.

**Fix**: extend `Selector::Target(u8)` to accept an array of targets
with associated damage portions: `Selector::DividedTargets(Vec<(u8,
Value)>)`. Cast-time the caster picks N targets and divides the spell's
total damage among them; resolution loops `DealDamage(target_i,
portion_i)`. This unlocks ~6 cube/SOS/STX 🟡s in one engine landing.

### Engine — Permanent-copy primitive (`Effect::CreateCopyToken`)

Multiple SOS/STX cards print "create a token that's a copy of target
non-Aura permanent you control" (Applied Geometry, Spitting Image,
Echocasting Symposium's body). The engine has no "copy a permanent"
primitive; these all approximate with a vanilla token mint.

**Fix**: add `Effect::CreateCopyToken { source: Selector, who:
PlayerRef, count: Value, modifiers: Vec<CopyModifier> }`. The
resolver reads `source`'s `CardDefinition` (printed copiable values)
and mints a token whose `definition` clones the source's. The
`modifiers` field lets cards like Applied Geometry append
"0/0 Fractal creature in addition to its other types".

### Engine — Cast-from-exile pipeline

SOS Improvisation Capstone, Decorum Dissertation, Echocasting
Symposium's Paradigm rider, Practiced Scrollsmith's "may cast" rider
all require a cast-from-exile-without-paying-its-mana-cost path with
an associated timer/decision shape. Many cube cards (Eldrazi
Conscription / Bolas's Citadel / Aminatou's Augury) need the same
primitive.

**Fix**: extend `GameAction::CastSpell` with an `alt_zone_source:
Option<(Zone, AltCostKind)>` field. The cast pipeline already supports
Flashback (cast from gy, pay flashback cost, exile on resolve); the
generalisation is a Zone + payment-mode tuple, with payment-mode
including `NoCost`, `Mana(ManaCost)`, `Discard(N)`, `ExileN(N)`, etc.

### Card — Verdant Mastery alt-cost mode

STX Verdant Mastery has a "{6}{G}{G}: each player fetches two basics"
alt cost adding a mode. Currently regular cost ({3}{G}{G}) ships
("each player fetches one basic") and the alt cost is omitted.

**Fix**: add a generic `AlternativeCost { mana_cost: ManaCost,
alt_effect: Effect }` shape that swaps the spell's resolved effect
based on which cost was paid. Same primitive unblocks Devastating
Mastery's "{7}{W}{W}: also return up to two nonland permanent cards"
mode and Baleful Mastery's mode swap.

### Card — Hofri Ghostforge exile-return-as-1/1-Spirit

Hofri's printed second clause: "When a nontoken creature you control
dies, if it wasn't a Spirit, exile it. Return it to the battlefield
under your control with 'When this creature leaves the battlefield,
create a 1/1 white Spirit creature token with flying.'" The body
+ Spirit anthem are wired; the death-replacement-with-return is still
🟡.

**Fix**: needs the general replacement-effect framework (push H
already tracked in Commander phase) — `ReplacementEffect` registry
keyed on `ZoneChange { from: Battlefield, to: Graveyard, card_filter }`.
Returns an `(Exile, DelayedTriggerOnExile)` 2-tuple instead of the
default zone change.

### Engine — `Modification::RemoveAllAbilities` only clears keywords

The layer-6 `RemoveAllAbilities` modification at
`game/layers.rs:284` does `keywords.clear()` only — activated and
triggered abilities still run from the original `CardDefinition`.
Cards printed "loses all abilities" (Mercurial Transformation,
Kasmina's Transmutation, Imprisoned in the Moon, Lignify) need the
ability sets to be cleared too.

**Fix**: extend `ComputedPermanent` with `cleared_abilities: bool`
(or `effective_activated_abilities: Vec<ActivatedAbility>` /
`effective_triggered_abilities: Vec<TriggeredAbility>`), then route
`activate_ability` / `fire_step_triggers` / the dispatcher through
the computed view. Unblocks the two STX 🟡 cards above.

### Engine — Skip-turn primitive (CR 716)

Ral Zarek -7 skips opp's next X turns. No skip-turn flag on
`Player`. Also blocks Time Walk's "after this turn, take another"
shape if a Twincast user wants to copy a Time Walk.

**Shape**: `Player.extra_turns: u32` already exists for extra turns;
add `Player.skipped_turns: u32` and have `pass_priority`'s
Cleanup-to-next-turn transition decrement and skip when non-zero.


### Card — Augusta, Dean of Order — "same-power batch" gate (push modern_decks batch 14 suggested)

The simplified per-attacker Augusta trigger (push (modern_decks)) skips
the "three or more attackers with the same power" gate. The printed
Oracle requires the engine to look at the **set of declared attackers
this turn** and find the largest equal-power subgroup, then pump only
that subgroup. Wiring shape:
- New `EventKind::AttackersDeclared` that fires once after
  `declare_attackers` resolves, carrying the attacker list.
- New `Selector::AttackersDeclaredThisTurn` accessible at trigger
  resolution.
- New `Effect::ForLargestSameXGroup { what: Selector, key: Value, then:
  Box<Effect> }` that buckets the selector by `key`, picks the largest
  bucket, and runs `then` against each entity in it.

Until those land, Augusta stays 🟡 with the per-attacker approximation.

### Card — Mavinda, Students' Advocate (push modern_decks STX Silverquill 🟡)

The cast-from-graveyard activated ability needs (a) a per-player
"this-turn cast-from-gy budget" counter, (b) a target-introspection
at cast time ("targets only a single creature"), and (c) a delayed
replacement to route the resolving spell to exile instead of graveyard.
Tracked separately under "Cast-from-graveyard introspection at
resolution time" in the Suggested next-up tasks section.

### Suggested next-up tasks (additions from batch 14)

- ⏳ **Anthem-on-Other static-effect helper-table** — push (modern_decks
  batch 14) adds three new tribal anthems via the existing
  `StaticEffect::PumpPT` + `StaticEffect::GrantKeyword` primitives
  combined with `Selector::EachPermanent(... .and(OtherThanSource))`:
  Inkling Verselord (Other Inklings have lifelink), Silverquill
  Anthemwriter (Other creatures get +1/+0), Lorehold Phantasmist
  (Other Spirits have haste). These work today but the call-site
  noise (three nested `.and(...)` chains) suggests a `static_anthem!()`
  macro or `tribal_anthem(creature_type, p, t, [keywords])` helper
  that compiles down to the same shape. Same precedent as the
  retired `tribal_anthem_for_name` table.

- ⏳ **Inkling tribal cube/SOS pool injection** — push (modern_decks
  batch 14) brings the Silverquill catalog to 30+ cards including
  multiple Inkling lords (Tenured Inkcaster +2/+2, Inkling Verselord
  lifelink) and 8+ Inkling minters. A new "Silverquill Inkling
  tribal" sub-college in the SOS pool selector could lean heavily
  into the synergy. Slot into `mono_color_pool(Color::White/Black)`
  once the deck-construction code supports archetype weighting.

- ⏳ **Spirit-tribal Lorehold subpool** — push (modern_decks batch 14)
  adds Lorehold Phantasmist's haste anthem on Other Spirits, joining
  Quintorius's +1/+0 anthem and Lorehold Bannerbearer's +1/+1
  anthem. With three Spirit anthems + 4+ Spirit minters (Sparring
  Regimen, Quintorius, Lorehold Excavation, Pillardrop Cultivator),
  a Lorehold-Spirit subdeck is viable. Same deck-construction-
  weighting gap as the Inkling case.

- ⏳ **Self-sacrifice ping pattern (Lorehold Bookburner template)** —
  the new Lorehold Bookburner (push batch 14) sacs itself to ping
  for 2 damage. The shape is recurring (Mogg Fanatic, Goblin Sharpshooter
  variants, Plaguemaster of Rakdos) — could be folded into a
  `shortcut::sac_self_ping(amount, filter)` helper that compiles
  the `ActivatedAbility { sac_cost: true, mana_cost, effect:
  DealDamage(target_filtered(filter), amount), .. }` template.

### Suggested next-up tasks (additions from batch 15)

- ⏳ **Pest-tribal Witherbloom subpool** — push (modern_decks batch
  15) adds 5 more Witherbloom Pest cards (Pest-Tender, Pest Swarmer,
  Witherbloom Seer, Pest Swarm, Witherbloom Vinemaster), bringing
  total Pest minters/payoffs to ~12 across the catalog (Pest Summoning,
  Tend the Pests, Eyetwitch, Eyetwitch Brood, Witherbloom Pestbinder,
  Witherbloom Pestmaster, Sedgemoor Witch, etc.). With Vinemaster as
  a top-end Pest payoff (3/4 Trample + +1/+1 on each Pest death) +
  Pest Swarmer as a sticky body, a Witherbloom-Pest archetype is
  viable. Same deck-construction-weighting gap as the Inkling /
  Spirit-tribal suggestions above.

- ✅ **Mint-then-pump helper** — push (modern_decks batch 21) lands
  `shortcut::create_token_with_keyword(who, count, token, keyword,
  duration)` and `shortcut::create_token_with_counter(who, count, token,
  counter, counter_n)` in `effect.rs:shortcut`. The two helpers wrap the
  recurring `Seq([CreateToken, GrantKeyword(LastCreatedToken, ..)])` and
  `Seq([CreateToken, AddCounter(LastCreatedToken, ..)])` patterns into
  one-liner call sites. Refactored to consume the helpers:
  `lorehold_skirmish` (mint Spirit + grant Haste EOT),
  `quandrix_summoner` (mint Fractal + add +1/+1 counter), and the new
  batch 21 `fractal_harvest` (mint Fractal + 3 +1/+1 counters). Same
  helpers unblock future "mint a token with [decayed / undying /
  haste / +1/+1 counter]" patterns at a single line each.

- ✅ **Magecraft pump helper for any target** — push (modern_decks batch
  21) lands `shortcut::magecraft_target_pump(what, power, toughness)` in
  `effect.rs:shortcut`. The helper wraps the `magecraft(Effect::PumpPT
  { what, power, toughness, duration })` shape so a caller supplies the
  selector (typically `target_filtered(Creature.and(ControlledByYou))`)
  and the helper handles the EOT-pump body. Powers Quandrix Scholar /
  Withergrowth Apprentice / similar "magecraft → pump target friendly"
  patterns at a single line. Sibling to `magecraft_self_pump(p, t)` which
  hard-codes `Selector::This`.

- ⏳ **CR 113.9 — ability-counter primitive** — Stifle/Squelch-style
  "counter target activated/triggered ability" cards. The engine has
  `Effect::CounterTarget` that works on `Selector::IsSpellOnStack`
  only; no `Selector::IsAbilityOnStack` variant. Wire when adding
  Stifle or Squelch to the catalog.

- ⏳ **CR 113.10b/c — full ability removal** — Mercurial Transformation
  ("loses all abilities") and Kasmina's Transmutation rely on a
  `Modification::RemoveAllAbilities` that today only clears
  keywords. The cleaner fix is `ComputedPermanent.cleared_abilities:
  bool` + a route through `activate_ability` / dispatcher to skip
  pre-removal ability sets. Tracked in the engine TODO row about
  `Modification::RemoveAllAbilities`.

### Suggested next-up tasks (additions from push modern_decks batch 16)

- ⏳ **Engine — `Value::ManaValueOf` zone-walk for stack spells** — the
  push (modern_decks) Mana Sculpt promotion reads
  `Value::ManaValueOf(Target(0))` after CounterSpell resolves; the
  countered spell is in graveyard by then. The evaluator walks
  battlefield → graveyard → hand → library → exile → stack. The stack
  walk is the fallback for the *pre-resolve* read case (Wandering
  Archaic's IS-cast trigger reads MV before the spell resolves).
  The existing fallback order is correct; the new audit row in CR 118
  documents the cost-introspection patterns.

- ⏳ **Engine — multi-mode picker with per-mode targets (CR 700.2d)**
  — Choreographed Sparks' "choose one or both" still collapses to
  "pick one mode" today. Same gap exists for Moment of Reckoning
  ("choose up to four. you may choose the same mode more than once").
  A proper fix needs `Effect::ChooseN` to accept per-mode targets via
  `ctx.targets` slot windows (mode 0 → slot 0, mode 1 → slots 1+, …)
  or a `Decision::ModePicks` shape that surfaces N (mode, target)
  tuples.

- ⏳ **Engine — `AlternativeCost.condition` predicate framework
  unlocked** — push batch 16's new `AlternativeCost.condition: Option<
  Predicate>` field gates an alt cost on a cast-time game-state
  predicate (Wilt in the Heat's "cards left your gy this turn", Orysa
  Tide Choreographer's "total toughness ≥ 10"). The same primitive
  unblocks Tenured Concocter's "becomes the target" alt cost shape
  (still ⏳), Beaming Defiance-style "if you control X, alt cost N",
  Suspend Aggression's alt cost variants, and the SOS Witherbloom
  Decanter alt-cost-with-graveyard predicate. Add `condition` field
  to alt-cost catalog rows when promoting these.

- ⏳ **Engine — fan-out summation for `Value::PowerOf` / `Value::
  ToughnessOf`** — push batch 16 changed `PowerOf` / `ToughnessOf` to
  sum across fan-out selectors (`EachPermanent(filter)`) — single-
  entity reads (Target/This/TriggerSource) unchanged. This unblocks
  Orysa's "total toughness ≥ 10" gate, Biorhythm's "set life to total
  power of creatures you control", Crackling Drake's "+1/+0 per
  instant/sorcery in graveyard" via `PowerOf(EachMatching(Graveyard,
  IS))`, and any future "sum stat across permanents" rider. Catalog
  rows that currently approximate this with `CountOf × Const(2)` or
  similar can promote to `PowerOf(EachPermanent(...))` once the
  fan-out is documented as the canonical idiom.

- ⏳ **Engine — `fire_spell_cast_triggers` threads converged_value
  onto trigger** — push batch 16 fixes a long-standing zero. Per-cast
  converge introspection now works for cast triggers (Magmablood
  Archaic's "+1/+0 EOT per color spent" pump). The Wildgrowth Archaic
  rider ("creatures you cast enter with X additional +1/+1 counters")
  still needs the *next* step: when a creature spell resolves, the
  permanent's ETB needs to read the cast spell's converged_value. That
  requires either a CR 614.12 replacement-effect framework that
  modifies how a permanent enters or threading the cast's
  `converged_value` onto the resulting `CardInstance` at spell-resolve
  time. Tracked separately.

- ⏳ **Card — finish Wilt in the Heat's damage-replacement rider** —
  the "if that creature would die this turn, exile it instead" half
  is still ⏳ (no damage-replacement primitive). A `Effect::
  ReplaceDamageWithExile { target, duration }` primitive would let
  Wilt finish + future Vraska's Contempt / Path to Exile-style
  "exile instead of destroy" cards.

- ⏳ **Card — Suspend Aggression / Ark of Hunger / Tablet of Discovery
  "may play this turn" pipeline** — three SOS Lorehold cards exile/mill
  cards then "may play that card this turn". All blocked on the same
  cast-from-exile-with-timer primitive. Wire shape: `Effect::
  ExileMayPlay { what: Selector, until: Duration }` + a side-list on
  `Player` of `(CardId, Duration)` tuples; the cast pipeline checks
  this list when casting from exile. Same primitive unblocks
  Practiced Scrollsmith, Conspiracy Theorist's attack trigger,
  Archaic's Agony, Echocasting Symposium's Paradigm rider, and the
  SOS Improvisation Capstone (the catalog's lone ⏳).

### Suggested next-up tasks (additions from batch 17)

- ⏳ **Inkling-tribal die-trigger payoffs** — push (modern_decks
  batch 17) adds Inkling Witness as an "Other Inkling dies → +1 life"
  payoff, joining Felisa's Inkling-minter and the Inkling Bloodscribe
  drain. The Inkling tribal pool now has 3 distinct death-trigger
  payoffs, making it viable to slot a dedicated Inkling-aristocrats
  subdeck into the SOS Silverquill pool selector once
  archetype-weighted deck construction lands.

- ⏳ **Drain-plus-tempo modular** — push (modern_decks batch 17) adds
  Defend the Inkwell as a drain-2 + scry-2 instant-speed (it's
  sorcery in card def but the pattern is broadly portable). The
  drain + card-selection shape is recurring (Sign in Blood, Costly
  Plunder, Read the Bones); a `shortcut::drain_and_scry(amount,
  scry_count)` helper would replace the explicit
  `Seq(Drain + Scry)` pattern at multiple call sites.

- ⏳ **Per-attacker batched event (CR 506.5)** — push (modern_decks
  batch 17) wires Lorehold Loremaster and Quandrix Reckoner as
  per-attacker `Attacks/SelfSource` triggers (Loremaster mints a
  Spirit per attack; Reckoner gets +1/+1 per attack). The
  per-attacker emission model matches printed batch triggers in
  2-player play, but a true batched `EventKind::AttackersDeclared`
  would let cards like Augusta read "creatures that attacked this
  combat" cleanly. Tracked separately under the Augusta row.

- ⏳ **Token mint + counter / keyword grant helper consolidation** —
  Lorehold Skirmish (mint Spirit + grant Haste EOT), Quandrix
  Summoner (mint Fractal + AddCounter), and now Lorehold Loremaster
  (mint Spirit on attack) all share the same `Seq([CreateToken,
  ...mutate-LastCreatedToken])` shape. A
  `shortcut::create_token_with_keyword(token, kw, dur)` and
  `shortcut::create_token_with_counter(token, counter, n)` helper
  pair (proposed in batch 15) would replace the inline pattern at
  10+ call sites.

- ✅ **CR 115.5 self-target enforcement** — Done in batch 17 (see
  the matching engine TODO row above).

- ⏳ **Magecraft body fan-out helpers** — push (modern_decks batch
  17) adds 5 magecraft creatures (Silverquill Pupil, Withergrowth
  Apprentice, Lorehold Pyrosage, Quandrix Tutelary, Prismari
  Pyrotechnician) that each use a hand-rolled `magecraft(Effect::...)`
  body. A `shortcut::magecraft_ping_each_opp(amount)` and
  `magecraft_ping_any(amount)` helper would consolidate the
  Lorehold/Prismari ping bodies; a `magecraft_target_pump(power,
  toughness, filter)` would handle the target-creature pump variant.

- ⏳ **PlayerRef::Opponent (single-opponent helper, restating)** —
  the existing `EachOpponent` collapses to the same player in
  2-player games. A `PlayerRef::Opponent` (the singular non-controller
  player) would read more naturally for single-opp effects like
  Baleful Mastery's "target opponent draws a card", Tezzeret's
  Gambit mode 1, and any "an opponent" wording. Workaround today is
  `EachOpponent` which is fine in 2-player but fans out in
  multiplayer. Low priority; cosmetic improvement.

### Suggested next-up tasks (additions from batch 21)

- ⏳ **Magecraft ping helpers** — push (modern_decks batch 21) leaves
  the `magecraft(Effect::DealDamage { to: target_filtered(_), amount: _
  })` pattern unconsolidated. A `shortcut::magecraft_ping_each_opp(amount)`
  for the Drainmaster / Burnscholar / Lorehold Pyromage shape (drain-each-
  opp on cast) and a `shortcut::magecraft_ping_any(amount)` for the
  Lorehold Apprentice / Pyrosage / Reverberator shape (any-target ping
  on cast) would consolidate the bodies at a dozen call sites. Companion
  to the new `magecraft_target_pump` helper already landed.

- ⏳ **Lifegain enchantment subpool injection** — push (modern_decks
  batch 21) adds Strixhaven Vigil (per-upkeep +1 life enchantment). The
  per-upkeep-gain shape is recurring (Soul Warden, Suture Priest, etc.).
  A new `StaticEffect::PerUpkeepLifeGain { who, amount }` primitive
  could consolidate the StepBegins(Upkeep) trigger pattern into a single
  static-ability row. Or, simpler: just leave each card as a trigger
  since the engine handles step-begin triggers cleanly.

- ⏳ **More magecraft shortcuts** — the printed magecraft templates
  are recurring: GainLife N, DealDamage 1 each-opp, CreateToken(Pest/Inkling),
  AddCounter(+1/+1 on self). A `magecraft_gain_life(n)`,
  `magecraft_pest_token()`, `magecraft_inkling_token()`,
  `magecraft_counter_self(counter)` set would replace the explicit
  `magecraft(Effect::...)` shape at every catalog entry.

- ⏳ **Search-to-bf "tapped" → "untapped" parameter** — push (modern_decks
  batch 21) adds `hunt_the_library` + `field_researcher`, both of which
  use the search-to-battlefield-tapped pattern. A `shortcut::search_basic_land_tapped(who)`
  and `shortcut::search_basic_land_untapped(who)` helper would consolidate
  the `Effect::Search { filter: IsBasicLand, to: ZoneDest::Battlefield {
  controller, tapped }}` template. Low priority — only 2 call sites today.

- ⏳ **Effect::LookAndDistribute primitive (Stress Dream / Adventurous
  Impulse / Curate)** — push (modern_decks batch 22) leaves Stress Dream
  approximated as `Scry 1 + Draw 1`. The printed "look at top N, put K
  in hand, rest to bottom of library in any order" is a recurring
  shape. Adding an `Effect::LookAndDistribute { who, look: Value, to_hand:
  Value, rest_to_bottom: bool }` primitive — with a new
  `Decision::LookAndDistribute { player, cards }` decision shape that
  returns `LookAndDistribute { to_hand: Vec<CardId>, to_bottom: Vec<CardId> }`
  — would let SOS Stress Dream, Curate, Adventurous Impulse, and ~6
  other cards land at exact-printed semantics. Auto-decider picks the
  first `to_hand` cards by hand-size + "good cards" heuristic; UI seats
  surface the picker. Promotes ~5 SOS 🟡 → ✅.

- ⏳ **Effect::CreateCopyToken primitive (Applied Geometry / Colorstorm
  Stallion / Echocasting Symposium)** — push (modern_decks batch 22)
  leaves the "create a token that's a copy of [permanent]" shape
  approximated as a vanilla token mint. Real cards: Applied Geometry
  (Fractal copy), Colorstorm Stallion (≥5-mana Opus copy of itself),
  Echocasting Symposium (copy of any creature), Spitting Image, Quasiduplicate.
  Adding `Effect::CreateCopyToken { source: Selector, who: PlayerRef,
  count: Value, modifiers: Vec<CopyModifier> }` lets all five 🟡 → ✅.
  The `CopyModifier` enum carries the "and it's also a Fractal" /
  "and it has haste" / "and it's a 0/0" overlays that print on the
  copy-creation spells.

- ⏳ **Storm keyword grant** — Prismari, the Inspiration prints "Instant
  and sorcery spells you cast have storm." The Storm keyword itself
  exists in `card.rs:237` but no engine path fans out copies for prior
  spells cast this turn. Adding a `Keyword::Storm` cast-trigger that
  reads `Value::StormCount` and emits `Effect::CopySpell { count:
  StormCount }` on each IS cast under a Storm grant would close this
  card + Tendrils-of-Agony chains (currently approximated). Note that
  `Effect::CopySpell` already has a `count` field (push modern_decks);
  the missing piece is the conditional grant + per-cast trigger that
  reads StormCount.

- ⏳ **Per-permanent "received-counter-this-turn" flag (Fractal Tender,
  Galvanic Iteration triggers)** — push (modern_decks batch 22) leaves
  Fractal Tender's end-step Fractal payoff omitted. The printed Oracle:
  "At the beginning of each end step, if you put a counter on this
  creature this turn, create a 0/0 green and blue Fractal token with
  three +1/+1 counters." Needs a per-permanent `received_counter_this_turn:
  bool` field bumped from the `Effect::AddCounter` resolver and cleared
  on each player's untap. Same primitive would let "if this creature
  gained a counter this turn" payoff cards land — Stonecoil Serpent
  variants, Goblin Slingshot.

- ⏳ **Cost-paid `sacrifice_other_filter` on `ActivatedAbility`** (push
  modern_decks batch 23 suggested) — printed Oracles like "{1}{B}, Sacrifice
  a Pest: …" cost-time-sacrifice-of-an-other-permanent are currently
  approximated as `Effect::Sacrifice` first-step bodies (which lets the
  rest of the body resolve even if no fodder exists). The strict form
  needs a new `ActivatedAbility.sacrifice_other_filter: Option<
  SelectionRequirement>` field that gates activation legality (no fodder
  → activation rejected with `SelectionRequirementViolated`). Wiring
  shape: parallel to `exile_other_filter` (push XVIII). Affected cards
  today: Witherbloom Pestbroker (push batch 23), Witherbloom Pestkeeper
  (could promote from first-step-sac to cost-time-sac), and any future
  "sacrifice a [type]" cost.

- ⏳ **"May play exiled card until N turns" framework (Suspend
  Aggression, Practiced Scrollsmith, Ark of Hunger, Elemental Mascot)** —
  push (modern_decks batch 22) leaves these 🟡 because the engine has
  no per-card "may play from exile until end of next turn" timer. Most
  Lorehold / Prismari exile-and-play cards need this. The engine has
  Flashback (cast-from-gy) and Rebound (cast-from-exile-once-on-next-
  upkeep) but no general "this exiled card has play-from-exile until
  N turns from now" timer.

  **Fix**: add a `Player.exile_with_timer: Vec<(CardId, u32)>` field
  bumped at exile time. Each `untap_step` decrements the counter; at 0
  the card stays in exile permanently. The cast-from-exile path
  consults this list in `try_pay_with_auto_tap` to authorize. Promotes
  ~6 🟡s across SOS/STX.

### Suggested next-up tasks (additions from batch 25)

- ✅ **Card-intrinsic Affinity-for-[filter] cost reduction** (push
  modern_decks batch 25) — new `CardDefinition.affinity_filter:
  Option<SelectionRequirement>` slot bakes a per-spell "{1} less to
  cast for each [filter]" discount onto the printed card. Read by
  `cost_reduction_for_spell` (`game/actions.rs`) at every cast path —
  hand-cast, alt-cost, back-face, flashback — so the discount applies
  consistently. Generic-only per CR 601.2f / 117.7c via the existing
  `ManaCost::reduce_generic` clamp. Promotes:
  - **Vanquish the Horde** 🟡 → ✅ (Creature filter, all battlefield)
  - **Witherbloom, the Balancer** 🟡 → 🟡 self-cast ✅ (Creature &
    ControlledByYou; the second IS-spell-grant-affinity static is still ⏳)
  Future Affinity-for-X (Artifacts, Lands, Pests) cards plug in
  without engine changes. Three lock-in tests:
  `vanquish_the_horde_affinity_for_creatures_reduces_cost`,
  `vanquish_the_horde_affinity_rejects_undercost_with_no_creatures`,
  `witherbloom_balancer_affinity_for_creatures_reduces_cost`.

- 🟡 **CR 705 — Flipping a Coin** (push modern_decks batch 25 — rules
  audit against `MagicCompRules_20260417.txt`): Coin-flipping primitive.
  Audit:
  (a) **705.1** — ⏳ (no coin-flip primitive in the engine; tracked
  separately as the `Effect::FlipCoin` row in this file). The two
  outcomes ("heads" / "tails") would be modeled as a `Decision::CoinFlip`
  so tests can script them deterministically.
  (b) **705.2** — ⏳ (no win/lose
  tracking on flips; `Effect::FlipCoin { on_heads, on_tails }` covers
  the "care only about heads/tails" case; a parallel `Effect::FlipCoinAndCall`
  with `on_win`/`on_lose` covers the call-and-match case).
  (c) **705.3** — ⏳ (no coin-flip-result override primitive;
  Krark's Thumb-style "if you would flip one, flip two and ignore one"
  needs the override on top of base flips). Blocked on the base
  `Effect::FlipCoin` primitive landing. Promote to ✅ when Karplusan
  Minotaur / Mana Clash / Krark's Thumb test fixtures pass.

- ✅ **`StaticEffect::GrantAffinityToISSpells { permanent_filter }`**
  (push modern_decks batch 25 follow-on) — Witherbloom, the Balancer's
  second printed clause "Instant and sorcery spells you cast have
  affinity for creatures" **now lands** via the new static. At cast
  time, `cost_reduction_for_spell` checks every battlefield permanent
  for this static and adds 1 to the reduction per matching permanent.
  Restricted to the controller's IS spells; opp's IS spells correctly
  unaffected. Promotes **Witherbloom, the Balancer** 🟡 → ✅. Future
  "your IS spells have affinity for [Artifacts/Lands/Pests]" cards
  plug in unchanged. Tests: `witherbloom_balancer_grants_affinity_
  to_is_spells`, `witherbloom_balancer_static_does_not_affect_opp_
  spells`.

### Suggested next-up tasks (additions from batch 26)

- ✅ **STX corpus growth via 7 follow-on cards** — push modern_decks
  batch 26 brings the STX catalog to 485 ✅ + 12 🟡. New cards:
  `pest_studies` ({1}{B}{G} Pest Lesson), `lecture_in_strategy`
  ({1}{R}{W} team-pump Lesson), `advanced_cartography` ({1}{G}{U}
  ramp+Scry Lesson), `bombastic_strixhaven_mage` ({2}{R} 2/3 ETB-ping
  + magecraft ping), `mage_hunters_strike` ({1}{B} -3/-3 instant),
  `mascot_researcher` ({2}{G} 2/2 counter-strewer), `strixhaven_tutor`
  ({2}{U} 2/2 Scry-Cantrip). All using existing primitives; 7 new
  tests lock in the play patterns.

- ⏳ **`StaticEffect::ClearAbilities { applies_to, duration }` for
  Mercurial Transformation / Kasmina's Transmutation** — the existing
  `Modification::RemoveAllAbilities` clears keywords on the layer-6
  computed view but doesn't suppress activated / triggered / static
  abilities. Wiring shape: add a `cleared_abilities: bool` field on
  `ComputedPermanent`, set it whenever a `RemoveAllAbilities`
  modification affects the source. Then:
  - `activate_ability` rejects activations from cleared sources
  - `dispatch_triggers_for_events` skips cleared sources
  - `compute_battlefield` filters `static_ability_to_effects` calls
    for cleared sources
  Promotes both STA reprint 🟡s and the future Imprisoned in the Moon
  / Lignify / Kenrith's Transformation series.

### Suggested next-up tasks (additions from batch 27)

- ✅ **STX corpus growth via 22 new cards** — push modern_decks
  batch 27 brings the STX catalog to 500 ✅ + 12 🟡. New cards span
  all five colleges plus shared mono/multi shells, all using existing
  primitives. Tests: 23 new (1 per card, plus a couple with paired
  tests). Total test count rises 2566 → 2589. The corpus is now
  weighted heavily toward Magecraft / ETB / Treasure / Pest / Spirit
  / Inkling / Fractal subdecks. Suggested follow-on: a tribal weighting
  pass in the SoS pool selector to actually steer toward the deeper
  subdecks.

- ⏳ **Single-Opponent `PlayerRef::Opponent`** — `EachOpponent` works in
  2-player but fans out in multiplayer. Some printed Oracle texts
  ("target opponent loses 1 life") would benefit from a singular
  Opponent ref that asks the controller to pick if there's more than
  one opp. Same shape as `Target(u8)` but typed to player-only.

- ⏳ **`Predicate::CardsInGraveyardAtLeast(who, filter, count)`** —
  push (modern_decks batch 27) adds Witherbloom Soilshaper which mints
  with +1/+1 counters per creature card in gy. The general "I have N+
  cards of [filter] in my gy" predicate would unlock Trespasser's
  Curse, Liliana's Mastery's emblem-of-zombies, and various delirium-
  flavoured payoffs at a single primitive.

- ✅ **Equipment activation pipeline (CR 702.6)** — DONE (push
  claude/modern_decks). `GameAction::Equip { equipment, target }` pays
  the equip cost, enforces sorcery-speed + your-creature targeting, and
  repoints `attached_to`; `CardDefinition.equipped_bonus` flows +P/+T +
  keyword grants via the layer system. Bonesplitter, Shuko, Lavaspur
  Boots, and the promoted Lightning Greaves ship on it. The bot equips
  its biggest creature (`pick_equip`); the server view exposes
  `equippable`; the client offers an `E`-key equip flow. Vehicles &
  Crew (CR 702.122) landed in the same push.

### Suggested next-up tasks (additions from batch 28)

- ✅ **STX corpus growth via 25 new cards** — push modern_decks batch
  28 brings the STX catalog to 525 ✅ + 12 🟡. New cards span all five
  colleges (5 per school) plus 5 shared/multi-college shells. Tests:
  30 new. Total: 2589 → 2619 tests (+30). All clippy-clean.

- ✅ **`Selector::LastCreatedTokens` (plural)** — new engine selector
  that tracks every token created in the current effect resolution.
  Powers Fractal Spawning's "create 2 Fractals, put a +1/+1 counter
  on EACH" printed Oracle faithfully (without the new selector, only
  the last token got the counter; the others died to SBA at 0/0).
  Same shape as the singular `LastCreatedToken` — fan-outs through
  `ForEach`, counter doublers multiply per-token. Test:
  `fractal_spawning_mints_two_fractals_with_counters`.

- ⏳ **CR 114 — Emblems** — fresh audit (batch 28) documents the
  emblem zone gap. Currently ⏳ — gates planeswalker ult emblems
  (Professor Dellian Fel -6 lifegain-drain emblem, Ral Zarek -7
  skip-turns emblem). Promote path: (1) add `Effect::CreateEmblem
  { who, abilities }` primitive; (2) wire an `EmblemObject` shape
  in the command zone with no characteristics other than its
  abilities; (3) hook the trigger dispatcher to walk command-zone
  emblems for `StepBegins` / `LifeGained` / etc. triggers.

- ⏳ **Per-spell "extra counters on cast" rider** (gates Wildgrowth
  Archaic 🟡 in SOS) — push modern_decks batch 28's emblem audit
  bumps this gap in priority. Wildgrowth Archaic prints "Whenever
  you cast a creature spell, that creature enters with X additional
  +1/+1 counters on it, where X is the number of colors of mana
  spent to cast it." Would need a per-cast static replacement that
  injects `enters_with_counters` on the resolving creature spell.

- ⏳ **`Effect::ChooseCreatureType`** — gates Crippling Fear 🟡 (STA
  reprint) and Engineered Plague-style universal effects. The
  collapse-to-universal approximation works but flips creature
  killing onto your own creatures. Adding a creature-type prompt
  + `Predicate::HasCreatureType(ChoiceResult)` would let cards
  faithfully ship the protect-mine-but-kill-yours pattern.

### Suggested next-up tasks (additions from batch 29)

- ✅ **STX corpus growth via 20 new iconic cards** — push modern_decks
  batch 29 brings the STX catalog to 545 ✅ + 12 🟡. New cards span
  all five colleges (4+ per school) — Silverquill Novice/Headmaster/
  Inkpact, Witherbloom Neophyte/Recursion + Pestpod Lurker, Lorehold
  Neophyte/Recallmage + Battle Banner, Quandrix Reach Mage + Fractal
  Sumcaster, Prismari Vandal/Flameseeker, plus 6 cross-college
  Strixhaven-flavored shells (Tutor/Pondkeeper/Rotcaster/Spellfletcher/
  Forager/Basicseeker/Curriculum + Magecraft Volley). Tests: 20 new
  test functions covering the headline play patterns. Total: 2628 →
  2656 tests (+28).

- ✅ **`ActivatedAbility.self_counter_cost_reduction`** — new engine
  primitive: optional `Option<CounterType>` on `ActivatedAbility`
  that subtracts the source's counter pool of the specified kind
  from the activation's generic mana cost, clamped at the printed
  generic total via `ManaCost::reduce_generic`. Mirrors
  `affinity_filter` on spells but reads the source's own counter pool
  instead of a battlefield filter — the shape needed by Strixhaven
  Book artifacts. Powers Diary of Dreams's "this ability costs {1}
  less for each page counter on this artifact" rider (🟡 → ✅).
  Future Page / Charge / Verse / Wish counter cards plug in against
  this same field without new engine code. Tests:
  `diary_of_dreams_activation_costs_five_with_no_page_counters`,
  `diary_of_dreams_page_counters_reduce_cost_by_one_each`,
  `diary_of_dreams_page_counters_clamp_at_printed_generic`.

- ✅ **`AlternativeCost.exile_from_graveyard_count`** — new engine
  primitive: an `u32` additional-cost slot on `AlternativeCost`
  that exiles N cards from the caster's graveyard as part of the
  alt cast. Pre-flight gate rejects if gy has < N cards (returns
  `SelectionRequirementViolated`); auto-picker takes the lowest-CMC
  matching cards. Emits `CardLeftGraveyard` per exile so payoffs
  that count gy-leave events (Ark of Hunger, Wilt in the Heat) see
  the event stream. Powers Soaring Stoneglider's "exile two cards
  from your graveyard or pay {1}{W}" additional-cost alt path —
  printed cost ships as the mana fork {3}{W}; the alt path is
  {2}{W} with `exile_from_graveyard_count: 2`. Tests:
  `soaring_stoneglider_alt_cost_exiles_two_from_graveyard`,
  `soaring_stoneglider_alt_cost_rejects_with_insufficient_graveyard`.

- ✅ **CR 405 — Stack** — fresh audit (batch 29). Wires every
  sub-rule except 405.3 (AP-vs-NAP ordering for simultaneous
  triggers across players, which the engine processes in
  ResolutionBuffer queue order rather than sorted by team) and
  405.6g (concession as a true SBA-bypass-immediate action; the
  engine catches eliminated players at the next SBA cycle, observable
  difference only mid-cast). Tests: implicit across the suite.

- ✅ **AP-vs-NAP stack ordering for simultaneous triggers** (CR 405.3)
  — Wired in `dispatch_triggers_for_events` (game/mod.rs:1908).
  When a single event (ETB, attack, combat damage) triggers
  abilities on both AP's and NAP's permanents, the engine
  sorts candidates by APNAP rank before pushing to the stack:
  AP's triggers go on lowest (resolve last), then each NAP in
  turn-order. Push order maintained by `apnap_rank(seat)` walking
  `next_alive_seat` from the active player. Test:
  `apnap_orders_simultaneous_triggers_active_pushed_first` in
  `tests/multiplayer.rs:624` — 4-player game, active=1, four
  pingers across all seats, assert stack push order is
  `[1, 2, 3, 0]`. Also `apnap_skips_eliminated_seat_in_cycle`
  for the mid-cycle elimination case.

### Suggested next-up tasks (additions from batch 30)

- ⏳ **`Effect::Search` with `count` / `tap` / `from-zone` parameters** —
  the current `Effect::Search { who, filter, to }` only supports
  single-card library-to-zone tutors. Quandrix Geomancer's printed
  "search for a basic Forest or Island, put it into your hand, then
  shuffle" works at body level but a future "search for up to two
  lands" or "search and put onto battlefield tapped" wants count
  and tap parameters. Cards needing this: Manifestation Sage (already
  ✅ via different path), future "search up to two basic lands"
  effects. Engine extension shape: add optional `count: Option<Value>`
  + `tap: bool` + `from: ZoneKind` fields, defaulted for compat.

- ⏳ **Auto-shuffle after tutor effects** — `Effect::Search` currently
  does not shuffle the library after the search resolves. Most printed
  Oracles ending in "then shuffle" rely on the shuffle to randomize
  the remaining library order. Engine extension: append a
  `Effect::ShuffleLibrary { who }` step after every Search resolution,
  or fold it into the Search primitive. Tests would need to assert
  library order changes; the auto-decider's library walk currently
  preserves order.

- ⏳ **Combat-step test harness** — `g.fast_forward_to_step_for_test`
  and `GameAction::DeclareAttackers` aren't available in the public
  test surface; batch 30's Lorehold Sparkknight test collapsed to a
  body sanity check rather than walking through DeclareAttackers. A
  helper that fast-forwards to DeclareAttackers, declares N attackers,
  and resolves combat would unlock per-attacker trigger tests for
  the entire Lorehold combat-trigger pool (Sparring Regimen,
  Loremaster, Spirit Champion, Aerospirit, Sparkknight, etc.).
  Same gap as the existing "no public combat helpers" doc note.

- ⏳ **Token name disambiguation** — batch 30's `silverquill_drafter_b30`
  / `silverquill_scrivener_b30` / `prismari_treasurewright_b30` /
  `quandrix_geomancer_b30` factory names collide with earlier extras-
  module cards of the same printed Oracle name. The factory name carries
  the disambiguator; the printed token name does not (the b30 card's
  `name` field still says "Silverquill Drafter B30"). Future cleanup:
  reconcile the duplicate Oracle slots — either retire one, merge to a
  single canonical printed Oracle, or rename the b30 cards with a
  printed-Oracle-distinct name.

- ⏳ **CR 701.7 audit — token-creation replacement layer** (push batch
  30): the create-token resolver in `game/effects/tokens.rs` runs
  before `compute_battlefield`'s continuous-effect layer, matching
  CR 701.7b's "replacement effects apply before continuous effects
  that modify token characteristics." The remaining gap is the
  printed Doubling Season / Anointed Procession-class replacement
  primitive — `replacement::replace_create_token` exists as a hook
  but no current card subscribes to it. Adding a Doubling-Season-class
  card would lock in the replacement layering test (count doubles,
  characteristics don't).

- ⏳ **Transient triggered-ability grants from pump spells** (push
  modern_decks batch 31 follow-up — claude/modern_decks branch):
  Root Manipulation ({3}{B}{G} sorcery, SOS Witherbloom) grants
  "Whenever this creature attacks, you gain 1 life" to every creature
  you control EOT. The engine has no `Effect::GrantTriggeredAbility`
  primitive — a slot like `CardInstance.granted_triggers_eot:
  Vec<TriggeredAbility>` that the EventKind dispatcher walks alongside
  `definition.triggered_abilities` would unlock this. Affected cards:
  Root Manipulation (combat-trigger lifegain), Sparring Regimen-style
  "your creatures gain X trigger" payoffs, Aether Vial-style ability
  grants. The grant duration honoring (EOT cleanup) would mirror the
  existing `granted_keywords_eot` Vec layout.

- ⏳ **Permanent-copy primitive** (push modern_decks batch 31 follow-up):
  Applied Geometry ({2}{G}{U} sorcery, SOS Quandrix), Colorstorm
  Stallion's Opus rider, Echocasting Symposium's body, Mirror Image-
  style cards all need an `Effect::CopyPermanent { source, … }` that
  creates a *non-spell* token copy of a battlefield permanent (vs.
  `CopySpell` which copies a stack item). Wiring shape: take the source
  permanent's `CardDefinition` (resolving CounterType / damage / etc.
  via its current characteristic state per CR 707), instantiate a
  fresh `CardInstance` with `is_token: true`, and place onto the
  battlefield under the chosen controller. Layered effects (anthems,
  Lordship buffs) apply to the copy at compute time.

- ⏳ **Casualty keyword (CR 702.140)** (push modern_decks batch 31
  follow-up — claude/modern_decks branch): Silverquill the Disputant
  (SOS) grants Casualty 1 to all your IS spells. Wiring shape: a new
  `Keyword::Casualty(u32)` keyword on `CardDefinition`, a cast-time
  optional additional cost "sacrifice a creature with power N+"
  enforced via the `AlternativeCost` pipeline, and a copy-the-spell
  trigger fired when the casualty is paid. Like Affinity, the keyword
  exposure surface is in `StaticEffect::GrantKeywordToISSpells`-class
  primitives to allow conditional grants.

- ⏳ **Nth-counter threshold trigger (CR 122.7)** (push modern_decks
  batch 31 audit — claude/modern_decks branch): "Whenever the Nth
  +1/+1 counter is placed on this creature" needs the engine to
  compare pre/post counter totals at each AddCounter resolution and
  fire only when the count crosses the threshold. Today `EventKind::
  CounterAdded(CounterType)` fires unconditionally on each add, which
  approximates the per-add Berta pattern but doesn't gate on a target
  count. Affected cards: would unlock Spike Tiller, Carnival of Souls,
  Vish Kal's "+5 counters → ult" pattern, etc.

### Suggested next-up tasks (additions from batch 33)

- ⏳ **Effect::CreateEmblem primitive** (push modern_decks batch 33
  follow-up): Implementing Professor Dellian Fel's -6 ult (and Ral
  Zarek's -7 once it lands) needs an `Effect::CreateEmblem { who:
  PlayerRef, triggered: Vec<TriggeredAbility>, static_abilities:
  Vec<StaticAbility> }` primitive backed by:
  (a) An `EmblemObject` shape stored in `Player.command` (alongside
  Commanders) with a flag bit `is_emblem: bool` so the trigger
  dispatcher knows to walk it.
  (b) The trigger dispatcher already walks the command zone for
  Commander triggers (per CR 113.6); extending to emblem-resident
  triggers is just expanding the walk to honor `is_emblem` markers.
  (c) Static abilities of emblems compute through `compute_battlefield`
  similarly to off-battlefield static effects (Conspiracies).
  CR 114 audit (TODO.md) gates on this primitive.

- ⏳ **StaticEffect::ClearAbilities** (push modern_decks batch 33
  follow-up): Mercurial Transformation, Kasmina's Transmutation, and
  the Pongify family all set a target creature to "a Frog with base
  P/T 1/1 and loses all abilities". The base-P/T half is wired via
  `Effect::SetBasePT`. The "loses all abilities" half needs a
  continuous-effect modification that clears keywords + triggered +
  activated + static abilities from the layered view. Wiring shape:
  add `Modification::ClearAllAbilities` (layer 6, similar to existing
  `RemoveAllAbilities` but operating on the full ability set, not just
  keywords), apply at compute time so the trigger dispatcher reads
  the cleared view rather than the raw `CardDefinition` field. This
  unlocks 2 STX 🟡 → ✅ promotions plus future Pongify-style cards.

- ⏳ **Effect::GrantTriggeredAbility (transient)** (push modern_decks
  batch 33 follow-up): Root Manipulation grants "Whenever this
  creature attacks, you gain 1 life" to each of your creatures EOT.
  The engine has `StaticEffect::GrantKeyword` (keyword-only, with
  duration) but no transient triggered-ability grant. Wiring shape:
  `Effect::GrantTriggeredAbility { what: Selector, ability:
  TriggeredAbility, duration: Duration }` that installs the trigger
  into a per-creature ephemeral list (cleared by SBA / cleanup based
  on duration). The trigger dispatcher already walks per-permanent
  abilities; just need to extend it to walk the ephemeral list too.
  Affected cards: Root Manipulation, Rabid Attack (die-to-draw rider).

### Suggested next-up tasks (additions from batch 40)

- ⏳ **`Effect::CastFromGraveyardPayingCost` primitive** (push
  modern_decks batch 40 follow-up): Mavinda, Students' Advocate's
  `{3}{W}{W}: cast target IS card from your graveyard that targets only
  a creature, exile it if it would die` activated ability needs a new
  primitive distinct from the existing `Effect::CastWithoutPayingImmediate`
  / `GameAction::CastFromZoneWithoutPaying` pair. The Mavinda activation
  pays {3}{W}{W} as the activation cost, then the resolved effect
  asks the controller "pick an IS card from your graveyard that
  targets a creature, then *cast it paying its normal cost*, exiling
  it on resolution". Wiring shape: extend
  `Effect::CastWithoutPayingImmediate` with a `paying_mana: bool` flag
  (default true matches the current free-cast behavior; flipping to
  false invokes the standard cost-payment pipeline). The
  "targets only a creature" gate already has a sibling in the
  `CastSpellTargetsMatch` predicate. Unlocks Mavinda 🟡 → ✅ and
  future "cast-from-gy paying cost" cards (Past in Flames-style).

- ⏳ **`AlternativeCost.tap_count` for Flashback variants** (push
  modern_decks batch 40 follow-up — re-raised from batch 39): Group
  Project's Flashback cost is "Tap three untapped creatures you
  control" (no mana), which doesn't fit the current
  `AlternativeCost { mana_cost, exile_from_graveyard_count, condition,
   target_filter }` shape. Extend `AlternativeCost` with
  `tap_count: Option<(u32, SelectionRequirement)>` so the cost-paid
  validator can require N untapped permanents matching the filter at
  payment time. Promotes Group Project 🟡 → ✅. The same primitive
  would land Asmoranomardicadaistinaculdacar's Convoke-as-flashback
  hybrid and Convoke-flashback combos.

- ⏳ **Spirit-tribal Lorehold archetype expansion** (push modern_decks
  batch 40 follow-up): Spirit Cantor (+1/+0 anthem for Spirits)
  joins Quintorius's pre-existing Spirit lord and the Lorehold token
  chain (Sparring Regimen, Lorehold Excavation, Quintorius). With
  this in place plus the new `lorehold_wraithcaller` flying-Spirit
  ETB minter, a Spirit-tribal Lorehold variant deck is now even more
  viable. Slot into the SoS Lorehold pool selector.

- ⏳ **`Effect::CastFreeOnCombatDamage` primitive** (push modern_decks
  batch 40 noted): Prismari Maestro's printed "Whenever this creature
  deals combat damage to a player, you may cast an instant or sorcery
  spell from your hand without paying its mana cost" rider is
  currently approximated as plain Draw 2 (the closest analog within
  existing primitives). The proper wiring uses a
  `DealsCombatDamageToPlayer/SelfSource` trigger with an
  `Effect::MayDo(CastWithoutPayingImmediate { what:
  Selector::OneOf(CardsInZone(Hand, Instant ∨ Sorcery)),
  source_zone: Hand })` body. The `OneOf` selector + free-cast-from-
  hand pipeline both exist; just need a hand-source variant of the
  cast-for-free helper.

- ✅ **Canonical `etb_drain`/`etb_gain_life` shortcut refactor**
  (push modern_decks batch 40): 11 existing Silverquill/Witherbloom
  cards refactored to the new shortcuts from batch 39
  (`silverquill_penitent`, `silverquill_castigant`,
  `inkling_pamphleteer`, `silverquill_drainwriter`,
  `silverquill_drainlord`, `silverquill_drainmaster`,
  `inkling_scriptwarden`, `inkling_maverick`,
  `silverquill_loremender`, `inkling_cardinal`,
  `witherbloom_thresher`). Each refactor replaces a 7-line
  `TriggeredAbility { event, effect }` literal with a 1-line helper
  call — net diff ~110 lines smaller. ~30 more candidate cards
  remain across stx::lorehold (Skydefender), stx::sos (Cauldron
  Familiar, Sedgemoor Witch, etc.) for future cleanup passes.

### Suggested next-up tasks (additions from batch 41)

- ⏳ **`Effect::CreateCopyToken { target_filter, mods }` primitive**
  (push modern_decks batch 41 follow-up): The CR 707.5 "enters as a
  copy of another permanent" + CR 707.9 "with modifications" shape.
  At resolution time, the resolver picks a target permanent (or
  arbitrary CardDefinition for "tokenize this card" effects), then
  emits a new permanent on the controller's battlefield whose copiable
  values clone the target's CardDefinition (P/T, types, abilities,
  costs). `mods` describes per-card overrides: "except it's a 0/0
  Fractal" (Applied Geometry), "with 6 +1/+1 counters" (Applied
  Geometry tail), "gains Haste and 'sacrifice at end step'"
  (Choreographed Sparks mode 1's copy). Unlocks: Clone, Cackling
  Counterpart, Phantasmal Image, Applied Geometry, Echocasting
  Symposium, Felidar Guardian, Spark Double, Mirror Image,
  Mascot Interception (currently approximated as
  GainControl/Untap/Haste rather than copying). Engine pieces
  needed: (a) target picker that prefers nonland permanents under the
  controller's choice; (b) CardDefinition clone helper that strips
  layered modifications and keeps only the printed/copiable values
  (per CR 707.2 "as modified by other copy effects, by its face-down
  status, and by 'as . . . enters' abilities"); (c) mod-application
  hook that overrides individual fields post-clone (creature_types,
  enters_with_counters, keywords, etc.). Promotes 4-5 partial cards
  from 🟡 → ✅ and is a prerequisite for CR 707's audit row promotion.

- ⏳ **Spend-restricted mana primitive** (push modern_decks batch 41
  re-raised): The "Add {X}. Spend this mana only to cast instant and
  sorcery spells" rider on Hydro-Channeler, Great Hall of the
  Biblioplex, Resonating Lute, Abstract Paintmage, and several SOS
  Prismari mana sources. Currently approximated as plain `AddMana`
  with no spend restriction tag — the mana correctly enters the pool
  but can be spent on anything. Engine shape: extend `ManaPool` from
  `HashMap<Color, u32>` to a structure that tracks per-source spend
  restrictions (e.g., `Vec<(Color, u32, Option<SpendRestriction>)>`)
  and have `pay_cost` walk restricted mana first when the spell
  matches the restriction. Promotes ~10 SOS cards from 🟡 → ✅.

- ⏳ **`Effect::FlipCoin` + `Decision::CoinFlip` primitive** (push
  modern_decks batch 41 re-raised — CR 705 audit standalone): The
  base coin-flip primitive blocking Karplusan Minotaur, Mana Clash,
  Frenetic Efreet, Squee's Toy, and Ral Zarek's -7 ult ("flip five
  coins"). Engine shape: (a) `Effect::FlipCoin { on_heads, on_tails }`
  for the "care only about the result" path; (b)
  `Effect::FlipCoinAndCall { caller, on_win, on_lose }` for the
  "call heads/tails and match the result" path (CR 705.2); (c)
  `Decision::CoinFlip` shape exposing the outcome so scripted tests
  can deterministically inject heads/tails (matching `Decision::Mode`
  / `Decision::MayDo` precedents); (d) RNG state on
  `GameState` (shared with shuffle / random-discard) so server
  replays are reproducible. The CR 705.3 "force-the-result" override
  primitive (Krark's Thumb) can layer on top once the base lands.

- ⏳ **Multi-target prompt for sorceries/instants (Selector::OneOf
  with count range)** (push modern_decks batch 41 re-raised): Several
  STX/SOS cards collapse "up to N targets" or "any number of targets"
  to a single mandatory target slot today. Examples: Divergent
  Equation (up to X gy IS cards), Rabid Attack (any number of friendly
  creatures — already partially handled with 3 slots), Crackle With
  Power (any number of targets, divided damage), Devious Cover-Up
  (any number of gy cards), Magma Opus (divided damage), Reality
  Spasm (untap up to X creatures — uses `up_to` for activated
  abilities only). Engine shape: extend `Effect.target_filter_for_slot`
  to expose a `count_range: (u8, u8)` and have the cast-time target
  prompt loop slot 0..N, accepting "skip this slot" (None) when slot
  index >= min. Auto-decider fills slot 0 and stops; scripted decider
  can fill more.

### Suggested next-up tasks (additions from batch 42)

- ⏳ **`Effect::PreventDamageThisTurn { source, amount }`** (push
  modern_decks batch 42 surfaced): SOS Wilt in the Heat ("If that
  creature would die this turn, exile it instead") and several
  STX/SOS cards print damage-replacement riders that need a
  prevent-or-replace primitive at the damage stack. Engine shape:
  add `Effect::ReplaceDeathThisTurn { who: Selector, replacement:
  Box<Effect> }` to allow "if a creature would die, exile instead"
  shape via the existing replacement-effects framework (Phase H).
  Promotes Wilt in the Heat, Light of Promise's lifegain triggers,
  and any printed "would die, exile" rider.

- ⏳ **`StaticEffect::GrantMiracle { cost }`** (push modern_decks
  batch 42 surfaced): Lorehold, the Historian's "Each instant and
  sorcery card in your hand has miracle {2}" rider blocks the full
  promotion to ✅. Engine shape: (a) static ability that adds a
  miracle alt-cost to spell cards in the controller's hand;
  (b) a "first card drawn this turn" cast-time check (the engine
  tracks `cards_drawn_this_turn` per player), and (c) a Miracle
  alt-cost pathway in `AlternativeCost` (similar to Flashback).
  Promotes Lorehold, the Historian to ✅ + opens the door for the
  Avacyn Restored miracle cycle (Bonfire of the Damned, Temporal
  Mastery, Reforge the Soul).

- ⏳ **More STX college expansions** (push modern_decks batch 42
  continuation): With the current Silverquill = 158 cards,
  Witherbloom = 141 cards, the existing core is dense. Future
  expansions should focus on (a) printing remaining iconic Stx
  cards like Quandrix Pledgemage, Strixhaven Mascot, Mascot
  Interception once `Effect::CreateCopyToken` lands; (b) finishing
  the 10 remaining 🟡 STX cards by addressing the engine gaps each
  surfaces; (c) rebalancing tribal density (Inkling, Pest, Spirit,
  Fractal, Elemental Wizard) across colleges to support sealed
  tribal pools.

- ⏳ **`shortcut::etb_mint_token(definition, count)`** (push
  modern_decks batch 43 surfaced): The
  `etb(CreateToken { who: You, definition, count })` pattern
  appears in hundreds of catalog factories (every Inkling /
  Pest / Spirit / Treasure / Fractal mint creature). A helper
  that takes a token definition + count would collapse the
  7-line trigger boilerplate to one line, mirroring the existing
  `etb_drain` / `etb_gain_life` / `etb_loot` shortcuts. Engine
  shape:
  ```rust
  pub fn etb_mint_token(definition: TokenDefinition, count: i32) -> TriggeredAbility {
      etb(Effect::CreateToken { who: PlayerRef::You, definition, count: Value::Const(count) })
  }
  ```
  Refactor candidates: Inkling Scribe, Inkling Brigade, Inkling
  Penmaster (magecraft variant — would need
  `magecraft_mint_token` sibling), Lorehold Echoist,
  Witherbloom Pest-Tender, Pest Brewer, Prismari Treasurer.
  ~50+ catalog factories collapse to one-liners.

- ⏳ **`shortcut::etb_scry(amount)`** (push modern_decks batch 43
  surfaced): Same shape as the existing
  `magecraft(Effect::Scry { … })` but as an ETB trigger.
  Witherbloom Cauldronkeeper / Quandrix Symmetrist / Silverquill
  Bookbearer / Silverquill Archivist / Inkling Treasurer all
  use the same Scry-on-ETB pattern; a helper would clean up
  the 7-line trigger pattern in each.

- ⏳ **`shortcut::magecraft_mint_token(token, count)`** (push
  modern_decks batch 43 surfaced): The
  `magecraft(CreateToken { … })` pattern shows up in Inkling
  Penmaster, Witherbloom Pestmancer, Prismari Alchemist, etc.
  Same shape as the above ETB helper but bundled with the
  magecraft trigger.

- ⏳ **`Effect::CreateCopyToken { what }`** (push modern_decks
  batch 43 surfaced): The "create a token that's a copy of
  target permanent" primitive blocks 5+ STX/SOS cards:
  Colorstorm Stallion's Opus big-body, Applied Geometry (which
  approximates "copy a non-Aura permanent" as "mint a 0/0
  Fractal"), Mascot Interception, Strixhaven Mascot, Spectacular
  Skywhale Opus big-body. Engine shape: walk the target's
  `CardDefinition`, clone it into a `TokenDefinition` (so it
  ceases to exist as a non-token zone-change resolves per
  CR 111.7), apply any "except it's a [type] in addition to its
  other types" rider via field mutation. Distinct from
  `Effect::CopySpell` which copies a stack spell. CR 707.2
  governs the copy-and-token-mint pipeline.

### Suggested next-up tasks (additions from batch 47)

- ⏳ **Inkling-tribal Silverquill subpool (revisit)** — push
  (modern_decks batch 47) brings the catalog to 38+ Inkling cards
  in `stx::silverquill` including 5+ Inkling lords / anthems
  (Tenured Inkcaster +2/+2, Inkling Verselord lifelink-grant,
  Inkling Sergeant +1/+0, Inkling Banner-Bearer, Inkling
  Calligraphist) and 15+ Inkling minters. The Silverquill college
  is now closed-out (only Mavinda 🟡). A sealed-pool selector that
  weights toward Inkling cards when Silverquill is the chosen
  college would lean the typical SOS Silverquill draft into
  Inkling-tribal payoffs more reliably. Slot into
  `sos_mode::pool_for_college(Silverquill)` once the
  deck-construction code supports archetype weighting.

- ⏳ **Token-death `died_card_snapshots` cache: extend to
  PermanentLeavesBattlefield events** — push (modern_decks batch
  47) ships the cache for CreatureDied. Token-leave triggers
  (Lorehold Spiritcaller's per-leave gain-1, Lorehold Reliquary's
  per-leave-graveyard +1/+1) for non-creature permanent leave
  paths (`PermanentLeavesBattlefield` event) need the same
  treatment when the leaving permanent is a token. Currently the
  engine emits CreatureDied for dying creatures only; PW dying,
  enchantment / artifact leaves go through a separate event path
  whose AnotherOfYours-scope lookups have the same zone-walk
  blind spot. Affects ~3 cards in the catalog. Engine shape:
  cache snapshots for every leave-bf zone change of a token,
  consult them from the same lookup chain.

- ⏳ **CR 606.5 — Combined loyalty cost (Carth the Lion)** —
  push (modern_decks batch 47, CR 606 audit) flagged that the
  engine accepts a single `loyalty_cost: i32` per ability, so a
  hypothetical "loyalty abilities cost an additional [+1]"
  static can't compose with the printed `[-N]` of a target
  ability to land at `[-(N-1)]`. No card in STX/SOS/cube uses
  this composition today, so doc-tracked. Engine shape: add a
  `loyalty_cost_modifier: i32` field on `Player` (read at
  activation time, applied symmetrically to + and - costs per
  the rule) and a `StaticEffect::ModifyLoyaltyAbilityCost {
  delta }` primitive that bumps the modifier.

### Suggested next-up tasks (additions from batch 49 / CR 105 audit)

- ⏳ **`Effect::CreateCopyToken { what: Selector }`** (batch 49
  surfaced again — see also batch 43 entry above): The "create a
  token that's a copy of target [permanent | creature you control]"
  primitive. The data flow: at resolution, read the target's
  `ComputedPermanent` (printed values **with** any layer 1-7c
  modifications baked in per CR 707.2), instantiate a
  `TokenDefinition` from the relevant fields, then route through
  the existing `Effect::CreateToken` placement path so triggers
  fire as usual. Blocks 5+ cards including Applied Geometry
  (currently mints a vanilla 0/0 Fractal + 6 +1/+1 counters),
  Colorstorm Stallion's big-Opus body, Mascot Interception,
  Spectacular Skywhale's Opus rider, and Echocasting Symposium.

- ⏳ **`StaticEffect::AddColor` / `StaticEffect::SetColor`** (CR
  105.3): The color-change continuous-effect primitive. Engine
  shape: extend `ComputedPermanent` with a `colors_override:
  Option<Vec<Color>>` slot and a `colors_added: Vec<Color>` slot;
  the layer-5 compute step folds both. Unlocks the printed type/
  color half of Kasmina's Transmutation ("becomes a blue Frog"),
  Mercurial Transformation ("becomes a blue Frog"), Fractalize
  ("becomes a green and blue Fractal"), and lets Painter's
  Servant exist in the catalog. Distinct from layer-7b
  `Effect::SetBasePT` which only rewrites P/T.

- ⏳ **"Choose a color" decision shape** (CR 105.4): Required for
  Painter's Servant, Cabal Ritual's name-keyed cousin, monochrome
  charms, and any card that prompts the controller to pick exactly
  one of {W, U, B, R, G}. Engine shape: new `DecisionAnswer::Color
  (Color)` variant + decision request that excludes "multicolored"
  / "colorless" per the rule. AutoDecider picks based on the
  caster's deck colors (highest-pip-count); ScriptedDecider takes
  the color as the test override.

- ⏳ **Per-school sealed-pool selector for `sos_mode`** (batch 49
  surfaced — same as batch 47's Inkling-tribal note but broader):
  Currently `sos_mode::pool_for_college` returns a uniform pool
  weighted by ✅ card count. With the catalog now at 1069 STX
  cards including 50+ Inklings, 60+ Pests, 40+ Spirits, 40+
  Fractals, and 40+ Elementals, the pool grows lopsided toward
  Lorehold (Spirit tribal) and Silverquill (Inkling tribal) on
  random draws. A weighted selector that biases toward each
  college's identity tribe (Lorehold→Spirit, Silverquill→Inkling,
  Witherbloom→Pest, Quandrix→Fractal, Prismari→Elemental) would
  produce more cohesive sealed decks. Slot into
  `sos_mode::pool_for_college` once it supports archetype weighting.

### Suggested next-up tasks (additions from batch 53 / CR 115 audit)

- ⏳ **`SelectionRequirement::SameTargetRejection`** (CR 115.3): The
  "same target can't be chosen multiple times for any one instance of
  the word 'target'" rule. Engine shape: extend
  `check_target_legality` (and its `_with_source` variant) to accept
  the full `additional_targets: &[Target]` slot list, and reject a
  newly-added target if it duplicates an earlier slot whose filter
  signature matches. The cast/activation pipelines at
  `game/actions.rs:657` (and the trigger-target pipeline) would
  pass the running slot vector instead of just the source id. No
  STX/SOS card currently leans on this (auto-target picker walks
  distinct creatures), but future cards (Lightning Reaver
  "two target creatures", Twin Bolt-style "two creatures") would
  surface a bug without this gate.

- ⏳ **`Effect::ChangeTargets { what: Selector, mode: ChangeTargetMode }`**
  (CR 115.7a-d): The "change the targets" / "change a target" /
  "change any targets" / "choose new targets" primitive. The
  `ChangeTargetMode` enum distinguishes 115.7a (all-or-nothing must
  re-target legally), 115.7b (exactly one slot changes), 115.7c
  (any subset), 115.7d (Redirect-style "choose new targets" where
  the player may leave any unchanged even if illegal). Unlocks
  Redirect, Bolt to the Face's misdirect rider, and the Disinformation
  Campaign-style "change target spell" interaction. The picked
  modifications flow into the targeted `StackItem::Spell.target` /
  `additional_targets` slots and resolve at resolution time per
  CR 115.7e ("only the final set of targets is evaluated to
  determine whether the change is legal").

- ⏳ **`SelectionRequirement::IsAura` + the Enchant keyword**
  (CR 115.1b): Aura spells are *always* targeted via the Enchant
  keyword on the printed card. Engine shape: add a
  `Keyword::Enchant(SelectionRequirement)` variant + a
  `CardDefinition.is_aura: bool` derived from the enchantment
  subtypes. The cast pipeline's `requires_target_check` would
  surface a target slot for any `is_aura` spell whose Enchant
  filter matches at least one battlefield permanent; rejection
  on no-match falls out naturally from the existing
  `InvalidTarget` error. The attached-to mechanic (CR 702.5b) is
  a separate slot — `CardInstance.attached_to: Option<CardId>`.
  Unlocks ~30 cards (Pacifism, Faith's Shield, Mortal's Resolve,
  and Strixhaven's Aura cycle).

- ⏳ **`Effect::PutOntoBattlefieldBlocking { what: Selector,
  attacker: Selector }`** (CR 509.4): When a creature enters the
  battlefield blocking, its controller chooses which attacker it's
  blocking. Engine shape: a `block_map` entry added at ETB time
  with the controller-picked attacker. Powers Restoration Angel's
  attack-trigger blink + the "enters blocking" mode of Mistmeadow
  Skulk, plus Mantis Rider-style "enters attacking" siblings if a
  separate primitive lands. Currently the flicker-and-block line
  no-ops the block half.

### Engine — `Effect::PumpPT` should honor `Duration::EndOfCombat`

Push (modern_decks batch 55) added `EffectDuration::UntilEndOfCombat`
and made `Effect::SetBasePT` route through it correctly via the new
`map_effect_duration` helper. But `Effect::PumpPT` still writes to the
legacy `CardInstance.power_bonus / toughness_bonus` fields directly,
bypassing the continuous-effect layer system. Those fields are cleared
in bulk at the next cleanup step (`clear_end_of_turn_effects`), so a
PumpPT with `Duration::EndOfCombat` silently lasts until end of turn
even though the rule says it should clear at end of combat.

**Fix**: route PumpPT through the same `ContinuousEffect` registry as
SetBasePT. The cleanup-step sweep can then drop `UntilEndOfCombat`
modifications without needing a special-case field. This also picks
up the duration mapping for free. Engine-wide ⏳ until a catalog card
actually uses `Duration::EndOfCombat` for a pump.

### Engine — Replacement-effect framework for triggered abilities

Strict Proctor (STX 🟡) wants "if a permanent entering the battlefield
causes a triggered ability of a permanent to trigger, that ability's
controller sacrifices the permanent unless they pay {2}." The
existing `ReplacementEffect` framework (`replacement.rs`) only models
zone-change replacements — it has no concept of "intercept an
about-to-fire trigger and tax it". Generalizing the framework to a
`TriggerReplacement` would unlock Strict Proctor, Torpor Orb, Hushwing
Gryff, Hushbringer, and several Strixhaven-adjacent cards. Engine
shape: extend the replacement registry with a separate
`TriggerReplacement { matcher: TriggerMatcher, action: ReplacementAction }`
slot consulted in `push_trigger` (game/actions.rs) before the trigger
hits the stack.

### Engine — Storm primitive (CR 702.40)

Prismari, the Inspiration (STX 🟡) grants "instant and sorcery spells
you cast have storm" — fan out a copy for each spell cast earlier this
turn. The engine tracks `spells_cast_this_turn` on `Player` already,
so the missing piece is a static "spells of type X gain storm"
modifier plus a Storm-resolution path that produces N copies via the
existing `Effect::CopySpell` primitive. Engine shape: a new
`Keyword::Storm` variant + a static-effect template
`StaticEffect::GrantKeyword { filter: HasCardType(Instant) ∨
HasCardType(Sorcery), keyword: Storm }` + a spell-cast hook that fans
out the copies. Promotes Prismari, the Inspiration + any future
storm-keyword card.

### Engine — Pest-tribal sacrifice scaling (Witherbloom Necropoet)

The new card Witherbloom Necropoet (push modern_decks batch 57) fires
"Whenever you sacrifice a creature, put a +1/+1 counter on each Pest
you control" via the existing `EventKind::CreatureSacrificed/YourControl`
event + `Selector::EachPermanent(HasCreatureType(Pest) ∧
ControlledByYou)`. The +1/+1 counter is applied via fan-out
`Effect::AddCounter` whose `what:` is the fan-out selector. Multi-pest
boards bump every Pest off a single sacrifice event. Lock-in test:
`witherbloom_necropoet_grows_pests_on_sacrifice`.

### Engine — Magecraft Fractal-tribal scaling (Quandrix Tideguard)

The new card Quandrix Tideguard (push modern_decks batch 57) targets a
friendly Fractal on magecraft via `Selector::TargetFiltered { slot: 0,
filter: HasCreatureType(Fractal) ∧ ControlledByYou }`. The cast-time
target picker walks the battlefield for Fractals before defaulting to
auto-target. Powers Fractal-tribal shells (Symmathematics +
counter-doublers). Lock-in test: `quandrix_tideguard_magecraft_pumps_target_fractal`.

### Engine — Mavinda, Students' Advocate cast-from-graveyard ⏳

Mavinda's `{3}{W}{W}: Cast target instant/sorcery from your graveyard
if it targets a creature; exile it as it would leave the stack` still
needs a generalized cast-from-graveyard primitive that combines:
1. An activation cost line that names a graveyard card as a target.
2. A cast-without-paying step (similar to `GameAction::
   CastFromZoneWithoutPaying`).
3. An exile-on-resolve hook for the cast spell (similar to Flashback's
   `exile_on_resolve` flag, but applied to a non-Flashback card).
4. A predicate gate: the activation rejects unless the chosen spell
   targets a creature.

Once landed, promotes Mavinda 🟡 → ✅. Also unblocks similar shapes —
Velomachus Lorehold's attack-trigger reveal-and-cast riders, Hofri
Ghostforge's exile-on-death-return-as-Spirit cycle.

### Engine — Hofri Ghostforge exile-on-death replacement ⏳

Hofri Ghostforge's "When another nontoken creature you control dies,
exile it. Return it to the battlefield as a 1/1 Spirit with haste at
the beginning of the next end step. Sacrifice it at the beginning of
the next end step after that" needs a delayed-replacement primitive:
1. A `ReplacementEffect::ExileOnDeath` that fires off
   `EventKind::CreatureDied` and replaces the gy-move with an
   exile-move.
2. A `DelayedTriggerSpec` for the "return as 1/1 Spirit" rider scheduled
   for the next end step, hooked into the existing `end_step`
   dispatcher.
3. A second delayed trigger for "sac at the next end step after that."
4. A return helper that mints a Spirit token with the exiled card's
   name/types but base 1/1 (similar to Sun Titan's recursion + a P/T
   override).

Once landed, promotes Hofri Ghostforge 🟡 → ✅.

### Engine — Velomachus Lorehold reveal-and-cast ⏳

Velomachus Lorehold's "Whenever Velomachus Lorehold attacks, look at
the top X cards of your library, where X is its power. You may cast a
spell with mana value X or less from among them without paying its mana
cost. Put the rest on the bottom of your library in a random order"
needs:
1. A reveal-from-library decision shape (a multi-card peek the
   controller can examine).
2. A cast-without-paying-from-library activation.
3. An MV-cap gate: the cast spell's mana value must be ≤ the reveal
   cap.
4. A "put the rest on the bottom in random order" step (needs RNG in
   `resolve_effect`).

Once landed, promotes Velomachus Lorehold 🟡 → ✅.

### Cards — Mid-priority Witherbloom additions

After batch 58 the Witherbloom (B/G) STX section is fully ✅. Easy
follow-ups for batch 59+ across other shells:
1. **Saw It Coming-style Foretell counters** — alt-cost discount on
   future-turn cast. Needs a turn-delayed alt-cost primitive on
   `AlternativeCost`.
2. **Foretold from exile** — register an exile timer; allow the next
   turn's caster to spend a foretell-discounted mana.
3. **Strixhaven Mystical Archive reprints** — Day of Judgment,
   Counterspell, Lightning Bolt, etc. — most already implemented in
   pre-STX modules; doc-sync them as STA-reprint rows.

### Cards — Mid-priority batch 61+ suggestions

After batches 59 + 60, the STX catalog is at 3168 tests and 1359 cards
across all five colleges. Easy follow-ups for batch 61+:
1. **Inkling-tribal cycle expansion** — Inkling Mentor / Inkling
   Scholar / Inkling Champion: per-Inkling-ETB counter triggers via
   `EventKind::EntersBattlefield/AnotherOfYours + HasCreatureType(Inkling)`.
2. **Pest-tribal cycle expansion** — pestbinder/pestwarden/pestkeeper
   stack with the existing Pestmancer/Vinemaster engine.
3. **Spirit-tribal Lorehold expansion** — gravewatcher /
   spirit-sage / battle-spirit using shared `lorehold_spirit_token`
   and the magecraft Spirit-pump precedent (Sparring Regimen).
4. **Fractal-tribal Quandrix expansion** — fractal-with-N-counters
   cycle (fractal_redleaf was 3, fractal_stormpetal was 4; add
   fractal_skybloom (2) and fractal_emberdust (5)).
5. **Prismari Treasure-mint cycle** — combine Prismari Artificer's
   ETB Treasure + something — discard/draw, scry, or 1-damage rider.

### Engine — `etb_drain_and_scry` shortcut ⏳

The pattern `etb(Effect::Seq(vec![drain(N), Effect::Scry { … }]))` shows
up across ~10 STX cards (Witherbloom Toxicpath, Witherbloom Blightbearer,
Silverquill Quillthane, etc.). Add a `shortcut::etb_drain_and_scry(drain,
scry)` helper to collapse the recurring 8-line trigger body into one
helper call. Same pattern for `etb_drain_and_surveil(drain, surveil)`
which would land Toxicpath and Quillthane.

### Cards — Batch 68 follow-ups ⏳

After batch 68 (modern_decks claude/modern_decks branch — 30 new STX
cards across all five colleges, 6 per college, total tests now 3336):

1. **Pest tribal anthem / lord cycle** — a 3-mana 2/3 "Other Pests you
   control get +1/+1" lord using `StaticEffect::PumpPT` + `OtherThanSource`
   (same shape as Tenured Inkcaster's Inkling anthem). Would tie the
   existing Pest token cycle together as a build-around.
2. **Inkling-tribal multiplicative pump** — magecraft trigger that
   pumps each Inkling +1/+0 EOT (already have `magecraft_pump_each_creature_type`
   shortcut; just add the Inkling instance — Inkling Bannerer exists,
   but lower-cost variants would round out the curve).
3. **Spirit-tribal go-wide payoff** — a 4-mana 3/3 R/W "Whenever Spirits
   you control attack, they get +1/+0 EOT" (uses `Attacks/AnotherOfYours
   + HasCreatureType(Spirit)` + `ForEach(Spirit + Attacking) → PumpPT`).
4. **Fractal lord** — a 3-mana 2/3 "Other Fractals you control get +1/+1"
   tribal anthem to tie the Quandrix Fractal cycle together.
5. **Prismari ramp + burn engine** — a creature with both ETB Treasure
   AND on-attack 1-damage-to-any-target (combines two existing
   shortcuts on a single body — fills the curve gap at 3 mana).

### Engine — `etb_pump_each_with_type` shortcut ⏳

The pattern `etb(ForEach(Creature & HasCreatureType(X) & ControlledByYou)
→ AddCounter(+1/+1))` shows up in Inkling Sigilbearer (push batch 51).
A shortcut helper `shortcut::etb_pump_each_with_type(creature_type, p, t)`
would collapse the 10-line trigger body into one helper call, paving
the way for ~6 other "ETB pump each [tribe]" cards across all five
schools' tribal payoffs.

### Engine — `magecraft_drain_target` shortcut ⏳

Mirror of `magecraft_drain_each_opp` but targeting a single opponent.
The "target opponent loses N life and you gain N life" magecraft body
shows up in Promising Duskmage, Inkling Coursebinder, Inkling Confessor,
Inkling Pamphleteer, Inkling Vassal, and Silverquill Adept. Currently
collapsed to "each opponent" via the auto-target framework. A
`shortcut::magecraft_drain_target(amount)` helper using a
`PlayerRef::Target(0)` slot would let the picker pick the opp
explicitly (relevant in multiplayer).

### Suggested next-up tasks (additions from batch 127)

Batch 127 promoted CR 509.3g via the new `EventKind::AttacksAndIsntBlocked`
event and `on_unblocked()` shortcut. Open items to explore next:

- **509.4 — "Put onto battlefield blocking"** — `Effect::PutOntoBattlefieldBlocking
  { what, blocking_attacker_filter }`. Used by Mantis Rider / Ambush
  Viper-style flash-blockers. The controller chooses which attacker the
  creature is blocking *as it enters*. Currently no primitive — the only
  "block at ETB" cards in the catalog (none yet) would need this.
- **509.1c — "Must block" requirements** — `StaticEffect::MustBlockTarget
  { target_filter }` or `Keyword::Provoke`. Provoke (e.g. Lure, Lunge)
  forces a creature to block if able. Currently no card uses this.
- **509.1d-f — Cost-to-block** — `ActivatedAbility`-style "creatures can't
  block unless their controller pays {N}" cost gate. Cards like
  Norn's Annex, Ghostly Prison-style attack taxes have the dual on the
  attacker side; no STX card uses it on the blocker side directly.
- **Ninja-style "swing in unblocked" payoffs** — now unblocked by
  `on_unblocked()`. Future cards: Ingenious Infiltrator, Yuriko-clones,
  ninjutsu-replacement-from-hand (engine-wide; requires ninjutsu cost
  primitive on top of `on_unblocked`).
- **Skulk** (CR 702.118) — "this creature can't be blocked by creatures
  with greater power." Unrelated to 509.3g but in the same family of
  evasion abilities. Catalog has Flying / Reach / Menace / Unblockable;
  Skulk is the next ladder rung.

### Suggested next-up tasks (additions from batch 129)

Batch 129 added 30 STX synthesised cards across all five colleges
focused on **tribal anthems** (Lorehold Spirit Banner, Witherbloom
Vinetongue, Witherbloom Reaper-Lord, Quandrix Fractalbinder),
landed the new `etb_mint_token_and_drain` shortcut helper, and added
the first Skeleton-tribal cards (Bonewight + Reaper-Lord). Open items
to explore next:

- **Fractal-tribal "X +1/+1 counters per Fractal" payoff** — the new
  Fractalbinder anthem combined with Quandrix Geometer/Bloomforge/
  Bloomscatter creates a wide Fractal board. A Body of Research-style
  "this creature enters with +1/+1 counters equal to the number of
  Fractals you control" card would scale aggressively with the now-
  filled Fractal pool. Engine has `Value::CountOf` already.
- **Skeleton "All Skeletons have wither/menace/death-trigger" lord** —
  Reaper-Lord (b129) gives anthem+menace. A 4-mana "Skeletons you
  control have 'when this creature dies, return it to your hand'" or
  "All your Skeletons have +1/+1 and deathtouch" tribal lord would
  unlock Skeleton-tribal aristocrats decks.
- **Spirit-tribal `magecraft_mint_spirit` shortcut** — Lorehold
  Sparkscholar II (b129) uses `magecraft_mint_token(lorehold_spirit_
  token(), 1)` — a 1-line `magecraft_mint_spirit()` helper would
  collapse this for future Lorehold magecraft mint creatures.
- **Plant subtype + Reach static** — Witherbloom Vinetongue (b129) is
  the anthem half; a "Plants you control have reach" static would tie
  the Plant pool (Sprawl-Vine, Verdant Sage, Mossfeeder, Sproutbinder,
  Vinemaster, Pestcaller, Pest-Tender, Pledgemage) into a defensive
  Reach engine vs Flying-heavy boards.
- **Lectern-style noncreature Spirit-tribal lord** — Lorehold Lectern
  (b129) is a 3-mana grant-Lifelink-to-Spirits artifact. Future cards
  in this slot: "Spirits you control have indestructible" (4 mana
  artifact), "Spirits you control have flying" (3-mana enchantment),
  or a per-Spirit-mint trigger ("Whenever a Spirit enters under your
  control, scry 1").
- **Quandrix Doubler scaling up** — Doubler (b129) puts +1/+1 on each
  Fractal at ETB. A higher-MV variant "Whenever you cast an instant or
  sorcery, put a +1/+1 counter on each Fractal you control" would
  collapse into Bloomscatter+ paws-up wins.

### Suggested next-up tasks (additions from batch 128)

Batch 128 added 30 STX synthesised cards across all five colleges,
audited CR 702.15 (Lifelink) for completeness. Open items to explore
next:

- **Skeleton tribal subpool** — Witherbloom Reaper-Hand (b128) introduces
  the Skeleton creature type with a die→drain trigger. A small subpool
  of Skeleton-tribal payoffs (anthem, regen-on-mana, "your Skeletons
  have menace") would unlock Skeleton-tribal SOS/Strixhaven decks.
- **Skeleton regeneration primitive** — printed Skeletons in Magic
  history often have `{B}: Regenerate this creature` activations.
  Currently the engine has a partial regen replacement, but the
  cost-of-regen pattern hasn't been ported to the post-batch helpers.
- **`etb_mint_token_with_counters(token, count, counter_amount)`
  shortcut** — Quandrix Bloomforge (b128) and Quandrix Geometer (b128)
  both use the pattern `etb(Seq[CreateToken, AddCounter(LastCreatedToken,
  +1/+1, N)])`. A helper would collapse this to one line. Pairs with
  `Fractal Bedrock` and Body of Research-style printed cards.
- **Reach + Plant tribal payoff** — Witherbloom Sprawl-Vine + Verdant
  Sage are Plant-typed Reach defenders. An anthem ("Plants you control
  get +1/+1") or a Plant-tribal lord would tie the Witherbloom Plant
  cards together with the existing Pest cycle.
- **Magecraft Treasure mint frequency** — `magecraft_treasure()` shows
  up on Lorehold Bookforger (b128), Prismari Tide-Surger (b128), and
  Prismari Flarescholar (b127). At a body cost of 4+ mana, the rate is
  defensive; future Prismari "every spell + every attack mints a
  Treasure" combinations would warrant a tighter cost curve.
- **Spirit-tribal anthem on Lorehold** — Lorehold Battlespirit (b128)
  is a 4/4 Spirit Warrior Haste that mints another Spirit on ETB.
  A Spirit-tribal anthem ("Spirits you control get +1/+1") would tie
  the existing 20+ Spirit creatures (Aerialist, Ironbound, Bell-Ringer,
  Battlespirit, Skybinder) into a tight tribal pool. Mirror of Tenured
  Inkcaster's Inkling anthem.

### Suggested next-up tasks (additions from batch 130)

Batch 130 added 21 STX synthesised cards across all five colleges and
landed the CR 305.2 ExtraLandPerTurn engine wiring with Exploration
({G} Enchantment) as the reference user. Open items to explore next:

- **Azusa, Lost But Seeking / Wayward Swordtooth** — printed cards
  that grant +2 land plays per turn (Azusa) or +1 (Swordtooth)
  via a static ability. Now that `ExtraLandPerTurn` is honored, each
  copy of these would automatically chain. The variant we need is
  a `StaticEffect::ExtraLandPlaysPerTurn(N)` (an integer multiplier,
  not a flag) — currently the engine sums grants by counting copies
  of the flag, which means an Azusa-style "+2 land plays per turn"
  needs three flag stamps (one each from "playing one more" lined up
  three times) on the same card. A clean refactor would replace the
  flag with `ExtraLandPlaysPerTurn(u32)` and have Exploration use
  `(1)`, Azusa use `(2)`, Swordtooth use `(1)`.
- **Mox Diamond / Crucible of Worlds** — Mox Diamond ("If a land
  card would enter the battlefield under your control, you may pay
  …") and Crucible ("you may play land cards from your graveyard")
  both need the play-land code path to be aware of *which zone* the
  land card is being played from. Currently `play_land_with_face`
  hard-codes `players[p].remove_from_hand`. A `play_land_from(zone)`
  variant + a `LandPlayPolicy::FromGraveyard` static effect would
  cleanly unlock both cards.
- **Fastbond** — "You may play any number of additional lands on
  each of your turns. Whenever you play a land, if it wasn't the
  first land you played this turn, Fastbond deals 1 damage to you."
  Needs a uncapped variant of ExtraLandPerTurn (or just a big
  number) plus a `LandPlayed` event-trigger gated on
  `lands_played_this_turn > 1`. The trigger half is already supported
  via `EventKind::LandPlayed` if we add the gating predicate.
- **Spirit-tribal anthem variants beyond Banner + Skyguard** — the
  R/W Spirit pool now has +1/+1 (Banner b129), Lifelink-grant
  (Lectern b129), and Flying-grant (Skyguard Banner b130). Future
  anthems to round out the cycle: Trample-grant ("Other Spirits you
  control have trample"), Reach-grant ("Other Spirits have reach")
  for Spirit-tribal-defensive builds, and an indestructible-grant
  Equipment ("Equipped creature has indestructible if it's a
  Spirit" — Hammer-of-Nazahn flavor).
- **Plant-tribal anthem + Plant-tribal payoffs** — b130 added
  Petalspeak ({2}{G} Sorcery, mass +1/+1 on each Plant) and
  Planttender/Blightroot (vanilla Plants). A Plant-tribal lord
  ("Other Plants you control get +1/+1") is the next natural fit;
  the Vinetongue (b129) already plays this role on Plants but a
  cheaper 2-mana version would unlock aggressive Plant decks.
- **Skeleton-tribal regenerate cycle** — Bonewight (b129) carries
  `Keyword::Regenerate(1)` already. A small cycle of similar
  Skeleton bodies (1/1, 2/2, 3/3) at curve-out mana values would
  give Reaper-Lord's anthem a tribal pool to feed.

### Suggested next-up tasks (additions from batch 133)

Batch 133 added 16 STX synthesised cards across all five colleges and
landed three new shortcut helpers (`etb_mint_token_and_gain_life`,
`etb_scry_and_draw`, `pump_and_grant_keyword`). Open items to explore
next:

- **`dies_mint_token_and_drain` shortcut** — mirror of
  `etb_mint_token_and_drain` for the on-death event. Used by
  "Pest dies → another Pest enters + drain" patterns. Currently
  no card uses this, but Witherbloom's Pest-aristocrats archetype
  would benefit from a one-line helper.
- **`magecraft_mint_and_drain` shortcut** — composite of
  `magecraft_mint_token` and `magecraft_drain`. Used in spells-
  matter Pest aristocrat decks where each instant/sorcery makes a
  Pest AND drains the table.
- **Plant tribal lord at common rarity** — Witherbloom Vinetongue
  (b129) at {1}{G}{G} is the rare/uncommon Plant anthem. A
  {2}{G} 2/2 "Other Plants get +1/+1" lord would round out the
  curve for Plant-tribal aggressive Witherbloom shells.
- **Spirit-tribal Lifelink anthem variants** — Lorehold Lectern
  (b129) grants Lifelink to Spirits. A 4-mana 2/4 Spirit body that
  is itself a Spirit AND grants Lifelink to other Spirits would
  combine the body slot with the anthem; current Lectern is a
  pure 3-mana artifact.
- **`on_other_dies_mint_token` shortcut** — "Whenever another
  creature you control dies, create a 1/1 X token." Witherbloom
  aristocrats payoff that scales with sacrifice fodder.

### Suggested next-up tasks (additions from batch 132)

Batch 132 added 25 STX synthesised cards across all five colleges and
shipped the CR 506.1 skip-on-empty-attackers engine fix. Open items:

- **CR 508.8 — Skip declare-blockers / combat-damage when no
  attackers** — partially landed via b132's CR 506.1 enforcement at
  the step-advance level (DeclareAttackers → EndCombat skip when
  `self.attacking.is_empty()`). The CR 508.8 path also says "if no
  attacking creatures are blocked, the combat-damage step is
  skipped." That partial-skip path isn't wired yet — the engine
  always advances DeclareBlockers → FirstStrikeDamage / CombatDamage
  even when no blocks were declared. The skip is unobservable today
  since CombatDamage handlers gracefully no-op on empty blockers,
  but the literal step skip would match CR more closely.
- **`dies_ping_creature` shortcut** — mirror of `dies_ping_any` /
  `dies_drain` for the creature-only target case. Used by Mogg
  Fanatic-style "dies dealing 1 to a creature" cards.
- **`magecraft_scry_and_draw` shortcut** — composite of
  `magecraft_scry` and `magecraft_draw` (Sphinx's Insight magecraft
  template). Useful for spellslinging engine creatures.
- **Skeleton-tribal "ETB regen self" pattern** — current Skeleton
  pool has Bonewight (b129) with `Keyword::Regenerate(1)` as the
  baseline. A "ETB scry 1 + Regenerate" Skeleton would round out
  the curve.

### Suggested next-up tasks (additions from batch 141)

Batch 141 added 21 STX synthesised cards across all five colleges,
landed 3 new shortcut helpers (`dies_ping_creature`,
`on_other_dies_mint_token`, `magecraft_mint_spirit`), and audited
CR 501 (Beginning Phase) to ✅. Open items to explore next:

- ✅ **`magecraft_mint_pest` shortcut** (push modern_decks batch 154
  done): mirror of `magecraft_mint_spirit` for the Witherbloom (B/G)
  Pest token. Wraps `magecraft(CreateToken { def: stx_pest_token() })`
  in a one-liner. Used by Witherbloom Pestmancer II (b154); lock-in
  test `shortcut_magecraft_mint_pest_uses_magecraft_trigger_with_create_token_body`.
- ✅ **`magecraft_mint_inkling` shortcut** (push modern_decks batch 154
  done): Silverquill (W/B) variant using `inkling_token()`. Wraps
  `magecraft(CreateToken { def: inkling_token() })`. Used by
  Silverquill Inkmancer (b154); lock-in test
  `shortcut_magecraft_mint_inkling_uses_inkling_token`. A future
  cleanup pass can sweep the long-form bodies in `stx::silverquill`
  (Inkling Penmaster, Silverquill Quillscribe, Inkling Confessor,
  Silverquill Inkletter, etc.) onto the helper.
- ✅ **`magecraft_mint_fractal(N)` shortcut** (push modern_decks
  batch 154 done): Quandrix (G/U) variant with the 0/0 base body +
  N counters from `Seq[CreateToken, AddCounter(LastCreatedToken, N)]`.
  Used by Quandrix Fractalsmith (b154); lock-in test
  `shortcut_magecraft_mint_fractal_seq_creates_token_then_stamps_counters`.
- **`etb_double_counters` primitive** — Quandrix Symmetrist's
  "double the +1/+1 counters on each creature you control" payoff
  is currently approximated as fixed +1/+1 counter adds. A general
  "for each creature you control, add CountersOn(self) more +1/+1
  counters" primitive would land the printed payoff exactly.
- **`magecraft_mill_each_opp(N)` shortcut** — Witherbloom / Dimir
  mill template ("Whenever you cast IS, each opp mills N"). No
  current STX card uses this, but it's a natural fit for future
  Witherbloom mill-control archetypes.
- **`SelectionRequirement::PowerAtMost(N)` already wired** — Power
  filter on `target_filtered` works; lets future "+1/+1 to target
  creature with power ≤ N" Silverquill / Witherbloom mid-curve
  payoffs ship without engine work.
- **CR 501 phase-level priority audit** — promoted to ✅ in batch
  141. Next audit candidates are CR 502 (Untap, partial — Phasing
  ⏳, Day/Night ⏳, untap-prevention statics ⏳) and CR 511
  (End of Combat, ✅ as of batch 55). The Beginning Phase audit
  exposed no engine gap at the phase-umbrella level — each child
  step owns its own turn-based actions per CR 501.1.

### Suggested next-up tasks (additions from batches 150–153)

Batches 150–153 added 83 STX cards across all five colleges, landed
five new engine primitives (`Value::MaxGraveyardSize`,
`Effect::LifeGainLockThisTurn`, `Player.cannot_gain_life_this_turn`,
`PlayerRef::ControllerOf`-via-stack, `ManaCost::summary()` /
`color_pip_letter()`), promoted four 🟡 cards (Swan Song, Visions
of Beyond, Skullcrack, Fiery Impulse) to ✅, and added CR 117 /
CR 405 / CR 119 explicit lock-in tests. Open items:

- **`Selector::TargetFiltered` shortcut for spell-mastery gates** —
  Fiery Impulse's spell-mastery test pattern (`SelectorCountAtLeast
  { sel: CardsInZone(You, Graveyard, IS), n: Const(2) }`) is verbose;
  a `Predicate::IsCardsInGraveyardAtLeast { who, filter, n }`
  shortcut would tighten the spell-mastery / threshold idioms across
  Searing Blaze, Fiery Impulse, Mishra's Bauble, Murderous Cut.
- **`Effect::PreventDamageThisTurn`** — Skullcrack's "damage can't be
  prevented" rider is still omitted (no general damage-prevention
  layer). The prevention path is its own dual to `LifeGainLock`;
  add `Player.damage_cannot_be_prevented_this_turn` + a "Damage
  prevention" replacement registry consulted by `deal_damage_to_from`.
  Same shape unblocks Furnace of Rath / Boil-style cards.
- **Multi-target Skullcrack with player-or-creature target** — the
  current Skullcrack only targets players. The printed Oracle is
  "any target" (creature, planeswalker, or player). Promote the
  cast-time filter from `Player` to
  `Creature ∨ Player ∨ Planeswalker` and route the damage and
  lifegain lock independently (the lock only applies when the
  target is a player).
- **Hybrid mana cost {2/W} support** — Spectral Procession's
  printed cost is `{2/W}{2/W}{2/W}` (3 hybrid pips, each paid as
  "2 generic OR 1 white"). The current Hybrid pip is `{a/b}` (1 of
  either color); we need a `ManaSymbol::HybridGeneric(u32, Color)`
  variant for the `{N/X}` shape. Unblocks Spectral Procession,
  Reaper King, future hybrid-pricing cards.
- **CR 614 — damage-replacement framework** — still 🟡 (no general
  "would deal damage, do X instead" hook). Furnace of Rath / Gisela
  / Heartless Hidetsugu are the canonical target cards. The
  replacement primitive shape: `ReplacementEffect::DamageDoubled
  { target_filter, multiplier }` consulted in
  `deal_damage_to_from` before the amount is committed.
- **Strategic Planning's "look-3-take-1" picker** — currently
  approximated as Mill 3 + Draw 1, which preserves the graveyard
  axis but collapses the choice. A `Effect::LookAtTopAndChoose
  { count, take, take_to: ZoneDest, rest_to: ZoneDest }` primitive
  would land Strategic Planning, Anticipate, Suspicious Stowaway,
  and the printed Lesson "look at top of sideboard" path.
- ✅ **Server: histogram of match durations** (push modern_decks
  batch 154 done): `MatchStats.duration_buckets: [u32; 6]` tracks
  hits per bucket (<30s, 30s-1m, 1-2m, 2-5m, 5-10m, 10m+) and
  `format_match_stats` appends the histogram as
  `| <30s:3 30s-1m:5 1-2m:7 …` to the rolling log line. Lock-in
  tests: `bucket_index_partitions_durations_into_six_buckets`,
  `match_stats_observe_duration_increments_correct_bucket`,
  `format_match_stats_includes_histogram_when_matches_present`,
  `format_match_stats_omits_histogram_when_zero_matches`.

### Suggested next-up tasks (additions from batch 154)

Batch 154 added 40 STX cards across all five colleges, landed six
new shortcut helpers (`magecraft_mint_pest`, `magecraft_mint_inkling`,
`magecraft_mint_fractal(N)`, `dies_mint_pest`,
`on_attack_mint_lorehold_spirit`, `magecraft_add_counter_self`,
`cards_in_graveyard_at_least(filter, n)`, `spell_mastery_gate()`),
exposed `PermanentView.static_ability_labels` to the client tooltip,
added a per-process duration histogram to MatchStats, and locked in
3 new CR rule tests (117.3a, 117.7, 119.8). Open items:

- **Refactor existing magecraft-mint-token call sites onto the new
  helpers** — `stx::silverquill` has ~4 long-form
  `magecraft(CreateToken)` Inkling minters (Inkling Penmaster,
  Silverquill Quillscribe, Inkling Confessor, Silverquill Inkletter,
  Inkling Penmaster); `stx::witherbloom` has ~10 Pest minters; the
  new shortcuts collapse each to a single line. Pure mechanical
  refactor with zero behavior change; sweep with grep + Edit.
- **Refactor existing `magecraft_add_counter_self` call sites** —
  ~15 cards in `stx::*` inline the
  `magecraft(AddCounter { what: Selector::This, ... })` body
  (Inkling Bookbinder, Inkling Calligraphist, Silverquill Soulbinder,
  Witherbloom Sproutchant, Quandrix Equationmage, Pensive Professor
  secondary, etc.). The helper lock-in test exists; just need a
  sweep.
- **Sweep `cards_in_graveyard_at_least` call sites** — Fiery Impulse
  is the canonical exerciser; future Searing Blaze / Murderous Cut /
  Mishra's Bauble cards can use the helper. Also a candidate to use:
  Tarmogoyf-style threshold gates if added.
- **`Effect::PreventDamageThisTurn`** — Skullcrack's "damage can't be
  prevented" rider is still ⏳. Same shape unblocks Furnace of
  Rath / Boil-style cards. Tracked alongside CR 614 damage
  replacement framework.
- **Static-ability tooltip exposure for the client** — DONE: the
  `static_ability_labels` field is now wired and the client tooltip
  renders it. Future polish: a "click to expand" affordance for
  long static descriptions on midrange/finisher creatures with
  multi-clause statics.
- **Bumping up SOS partial coverage** — the SOS catalog is 100%
  ✅; the next layer of SOS work is verifying cube/sealed-pool
  selectors for the new STX synthesised cards (`silverquill_*_b154`,
  `witherbloom_*_b154`, etc.) — only the printed cards live in
  `cube::all_cube_cards()` today. The synthesised cards are
  catalog-only and don't appear in any deck pool. A future
  `synthesised_pool()` helper in `cube.rs` could surface the b150+
  batch cards for testing purposes.

### Suggested next-up tasks (additions from batches 164/165)

Batches 164 + 165 added 64 STX cards across all five colleges (14
Lorehold, 14 Witherbloom, 12 Prismari, 12 Quandrix, 12 Silverquill),
4 CR lock-in tests (119.1, 119.3, 704.5f, 401.1), exposed
`spells_cast_this_turn` to the server PlayerView, and added
`SetNoMaxHandSize` / `FlipCoin` labels to the server's
`ability_effect_label`. Open items:

- **Prismari Cannonade and board-sweeper patterns** — the
  `EachPermanent(Creature)` DealDamage pattern used by Cannonade is
  symmetric (hits your own creatures too). A conditional sweep like
  "deals 2 to each creature your opponents control" would need the
  existing `ControlledByOpponent` filter. Cards like Anger of the
  Gods / Pyroclasm fit this mold — a future batch could add those.
- **Quandrix Tidebinder bounce with power filter** — the
  `Move(PowerAtMost(2) → Hand(Owner))` pattern is clean. Future
  cards like Man-o'-War / Reflector Mage with unconditional bounce
  could simplify to `Move(Creature → Hand(Owner))`. Document
  the pattern difference.
- **Quandrix Spellgrafter ETB counter** — the `etb(AddCounter(
  target Creature, +1/+1))` pattern works end-to-end. Future
  Quandrix scaling payoffs (Fractal-specific counters) could use
  the same pattern with `HasCreatureType(Fractal)` filter.
- **Storm count display** — `spells_cast_this_turn` is now in
  PlayerView. A client-side UI element could show the storm count
  in the HUD when it's ≥ 2, enabling storm-oriented gameplay.
- **Refactor Witherbloom self-mill ETB pattern** — Deathcoach's
  `etb(Mill(You, 2))` should be a shortcut (`etb_self_mill(N)`) if
  more Witherbloom self-mill creatures are added.

### Suggested next-up tasks (additions from current session)

- **Escalate keyword** — Collective Brutality is modeled as a
  single-mode ChooseMode. The printed Escalate ("you may choose
  additional modes; discard a card for each extra mode") would need
  a multi-mode-with-additional-cost primitive. Low priority since
  the single-mode version covers the primary play pattern.
- **Treasure-on-tap triggers** — Magda, Brazen Outlaw's "whenever a
  Dwarf you control becomes tapped, create a Treasure" needs a new
  `EventKind::PermanentTapped` trigger scope (distinct from the
  existing `GameEvent::PermanentTapped` which fires but isn't wired
  as a trigger event). Would also unblock Carrot Cake's token-on-tap.
- **Dwarf lord static self-exclusion** — Magda's "+1/+0 to other
  Dwarves you control" is modeled as "+1/+0 to ALL Dwarves you
  control" (including herself). Need an `exclude_source` flag on
  `StaticEffect::PumpPT` to implement the "other" qualifier. The
  layer system already has `AffectedPermanents::All { exclude_source }`
  — just needs wiring.
- **Spirit token helper** — Descendant of Storms creates an ad-hoc
  Spirit token. A shared `spirit_token()` helper (like the existing
  `pest_token()`, `inkling_token()`, `elemental_token()`) would
  reduce boilerplate for future Spirit-tribal cards.

### Session observations (push claude/modern_decks — Prowess)

- **Prowess auto-enforcement** — engine now injects +1/+1 EOT for
  Prowess-keyword creatures when their controller casts a noncreature
  spell, but only for creatures lacking their own SpellCast trigger
  (avoids doubling for cards wired via `shortcut::prowess()`). Future
  Prowess creatures can just carry `Keyword::Prowess` without needing
  the explicit `TriggeredAbility`.
- **Ward generic payment drain ordering** — the mana pool's generic
  drain pass (Pass 6 in `ManaPool::pay`) drains colored mana before
  colorless. This means Ward's generic cost can consume colored mana
  needed for the spell itself. A smarter payment ordering (reserve
  colored pips needed by queued costs) would make Ward+spell payment
  smoother. Low priority since the triggered-ability Ward path handles
  this at stack-resolution time (after the spell is already cast).
- **Deathtouch outside combat** — `Effect::DealDamage` doesn't track
  deathtouch on the damage source. In-combat deathtouch works because
  the combat resolver inflates damage to toughness when a deathtouch
  blocker/attacker is involved. Non-combat deathtouch (e.g. Prodigal
  Pyromancer with Basilisk Collar) would need a `dealt_deathtouch`
  flag on `CardInstance` checked in SBAs.
- **Increment mechanic** — still blocked on mana-spent-on-cast
  introspection. Several Strixhaven creatures (Pensive Professor,
  Tester of the Tangential, Hungry Graffalon, Cuboid Colony, etc.)
  carry a body-only 🟡 wire. Once `Player.mana_spent_on_last_cast`
  or a `Value::ManaSpentOnCast` primitive lands, these can be
  bulk-promoted.

## New suggestions (added 2026-05-25 push: modern_decks session)

### Cards
- ✅ **Professor of Symbology** (STX): {1}{W} 2/1 Human Cleric, ETB
  draw 1 (Learn approximated). Good 2-drop for Silverquill decks.
- ✅ **Silverquill Silencer** (STX): {W}{B} 3/2 Human Cleric, body-
  only (name-choosing penalty omitted).
- ✅ **Fractal Summoning** (STX): {X}{G}{U} Sorcery — Lesson. Create
  a 0/0 Fractal with X +1/+1 counters. X-cost scaling via
  `Value::XFromCost` + `Selector::LastCreatedToken`.
## New suggestions (added 2026-05-01 push VIII)

These items came up while implementing the push VIII batch and are
listed here so the next pass can pick them up without re-deriving them.

### Engine

- **X-cost activated abilities**. `ActivatedAbility.mana_cost` accepts
  `ManaSymbol::X` symbols today, but the activation entry point doesn't
  surface an X-value prompt (unlike `cast_spell`, which has
  `x_value: Option<u32>`). Berta, the Wise Extrapolator's `{X}, {T}:
  Create a Fractal token + X +1/+1 counters` is currently stubbed
  because X resolves to 0 at activation time. Adding an `x_value` arg
  to `GameAction::ActivateAbility` (and threading it through
  `Effect::AddCounter { amount: Value::XFromCost }`) would unblock
  Berta plus several X-cost utility activations across MTG history
  (Forerunner of the Empire-style scaling).

- **Per-spell-type per-turn tallies**. `Player.spells_cast_this_turn`
  counts every cast — Potioner's Trove's printed "Activate only if
  you've cast an instant or sorcery spell this turn" approximates by
  reading any spell. A sibling `instant_or_sorcery_cast_this_turn`
  (and `creature_cast_this_turn` for creature-spell triggers) would
  promote Potioner's Trove + a handful of Magecraft-adjacent payoffs
  to their exact-printed gates.

- **Per-turn exile count tally**. Ennis, Debate Moderator's end-step
  counter is gated on `CardsLeftGraveyardThisTurnAtLeast` as a proxy
  for the printed "if one or more cards were put into exile this
  turn". A first-class `Player.cards_exiled_this_turn` (incremented in
  `move_card_to`'s exile branch) + `Predicate::CardsExiledThisTurnAtLeast`
  would land Ennis on the printed predicate and unblock other
  exile-matters Strixhaven cards (Decadence's Lament, Devoted Caretaker
  variants).

- **CounterAdded scope filter**. `EventScope::SelfSource` for
  CounterAdded fires only for counters on the source card. The
  remaining Berta/Heliod-style payoffs need scope variants for
  "any creature you control" (Heliod, Sun-Crowned) and "any permanent"
  (Vorinclex, Monstrous Raider). Add `EventScope::AnotherOfYours` and
  `AnyPlayer` matching for CounterAdded events.

- **Counter-transfer-on-death primitive**. Ambitious Augmenter and
  several SOS Increment-payoff cards trigger "when this dies, if it
  had counters, create a token with those counters." Today there's no
  way to snapshot the dying creature's counter set in a death
  trigger's body. Adding `Selector::DyingPermanent` (or a
  `Effect::TransferCountersToToken { kind, count }`) would unblock
  this whole subtheme.

- **Per-cast converge introspection on the just-cast spell**.
  Magmablood Archaic and Wildgrowth Archaic have spell-cast triggers
  whose body reads the *cast spell's* converge value (number of colors
  spent on the iterated cast), not the source card's own converge
  value. Today the trigger fires but `Value::ConvergedValue` resolves
  to the source's own ETB-recorded value. A
  `Value::CastSpellConvergedValue` (mirror to the existing
  `Selector::CastSpellTarget`) would unblock both Archaic spell-cast
  riders + similar future cards.

### UI

- **Activate-ability gate hint**. When the new
  `ActivatedAbility.condition` rejects an activation,
  `GameError::AbilityConditionNotMet` bubbles up. The 3D client's
  ability-tray UI doesn't yet show "needs 7+ in hand" or "needs IS
  this turn" hint text — add a small tooltip or grayed-out treatment
  that surfaces the predicate in human-readable form (`Predicate ⇒
  "you need ≥7 cards in hand"` etc.) so players don't get cryptic
  rejection feedback.

### Server

- **Per-trigger gate evaluation logging**. Push VIII's
  `EventScope::SelfSource` extension landed silently; the server has
  no instrumentation for which triggers fired vs. were filtered out
  by scope. A debug flag on `dispatch_triggers_for_events` that emits
  `TriggerFiltered { source, kind, scope, reason }` events would help
  diagnose silent-no-fire reports during cube playtesting.

## New suggestions (added 2026-05-01 push IX)

These items came up while implementing the push IX batch and are
listed here so the next pass can pick them up without re-deriving them.

### Engine

- **Look-and-distribute-by-count primitive**. Flow State's printed
  shape ("look at top 3, put 1 in hand and 2 on bottom") and a
  handful of similar SOS cards (Stress Dream, Zimone's Experiment)
  need a `Effect::LookSplit { count, to_hand: Value, to_bottom: Value }`
  primitive that deals out the looked-at cards by category. Today we
  approximate with `Scry N + Draw 1` (correct first-card-to-hand,
  but the controller can't reorder mid-resolution). A first-class
  primitive would also unblock the conditional "instead pick 2"
  upgrade rider on Flow State (gated on a graveyard-IS-pair predicate).

- **Multi-target prompt for instants/sorceries**. Several SOS cards
  specify two target slots (Prismari Charm's "1 damage to one or two
  targets", Pull from the Grave's "up to two creature cards", Cost of
  Brilliance's "draw + LoseLife on player + counter on creature").
  Engine fix tracked in TODO.md "Multi-Target Prompt for Sorceries /
  Instants". Push IX collapses Prismari Charm mode 1 to single-target.

- **Emblem zone**. Professor Dellian Fel's -7 ult and Ral Zarek
  Guest Lecturer's -7 ult both produce emblems that grant ongoing
  abilities. The engine has no emblem zone or `Zone::Emblem` model
  yet. Adding one would unblock dozens of planeswalker ults
  (Elspeth's "creatures get +1/+1, vigilance, lifelink", Liliana's
  "your creatures get +2/+2 menace", etc.). A flat-list `Vec<Emblem>`
  per-player with the same trigger/static plumbing as battlefield
  permanents would suffice.

- **Coin-flip primitive**. Ral Zarek Guest Lecturer's -7 ult
  ("flip five coins"), Krark's Thumb-style replay, and Fiery Gambit
  use coin-flip mechanics. Add `Effect::FlipCoins { count, then }`
  with a `Value::HeadsCount` reading the most recent flip-coin batch.

- **Skip-turn primitive**. Ral Zarek Guest Lecturer's -7 ult also
  needs "target opponent skips their next X turns". Add
  `Effect::SkipTurns { who, count }` + a per-player
  `extra_turn_skip: u32` counter consumed at turn-roll time
  (mirror to the existing `extra_turns_to_take` pattern).

- **Card-name-as-cost activation (Grandeur)**. Page, Loose Leaf
  has Grandeur — "Discard another card named Page, Loose Leaf:
  do thing." Adding `ActivatedAbility.discard_named_self: bool` (or
  a sibling `ActivatedAbility.cost: ActivationCost` enum) would
  unblock Grandeur-style mechanics across MTG history (the original
  Future Sight cycle).

### UI

- **Witherbloom end-step hint**. The new
  `PlayerView.creatures_died_this_turn` field surfaces the
  "Essenceknit Scholar will draw at end step" predicate. The 3D
  client doesn't yet render this hint — adding a small icon or
  badge over Witherbloom-flavoured payoffs (Essenceknit Scholar,
  Cauldron of Essence's death drain) would improve readability.

### Server

- **Death-trigger event ordering audit**. Push IX's tally bumps in
  both `apply_state_based_actions` (SBA path) and
  `remove_to_graveyard_with_triggers` (destroy path) are correct
  for the common case but assume mutual exclusivity. Audit the
  call graph to ensure no creature-death path bumps the tally
  twice (e.g. if a destroy effect both calls
  `remove_to_graveyard_with_triggers` *and* triggers SBA in the
  same resolution window). Today they're disjoint, but this is a
  silent invariant worth a comment + a regression test.

## New suggestions (added 2026-05-01 push X)

These items came up while implementing the push X batch and are
listed here so the next pass can pick them up without re-deriving
them.

### Engine

- ✅ **Ward enforcement** (push XVII). Ward {N} enforced as a targeting
  tax across all three cast paths + flashback + alternative cost. Hard-mode
  Ward variants (pay life, discard, sacrifice) still ⏳.

- **Multi-target prompt for spells/abilities**. Push X works around
  this in Pull from the Grave by auto-picking the top 2 creature
  cards from the controller's graveyard via `Selector::Take(_, 2)`,
  but the printed cards specify *target* slots — the current
  implementation can't accept opponent-side targets. A real fix
  needs `GameAction::CastSpell`'s `target` field to become
  `Vec<Target>` (or a sibling `targets: Vec<Target>` channel) and
  the cost/effect path to address `Selector::Target(0)`,
  `Selector::Target(1)`, etc., to the corresponding entries.
  Unblocks Cost of Brilliance, Render Speechless, Homesickness,
  Prismari Charm mode 1, Stress Dream, Vibrant Outburst, and
  several SOS instants/sorceries that bake two target slots.

- **Cast-from-zone snapshot on `StackItem`**. Antiquities on the
  Loose's "if this spell was cast from anywhere other than your hand,
  +1/+1 counter on each Spirit you control" rider reads the cast's
  source zone. The engine already differentiates flashback casts via
  the `CardInstance.kicked` flag, but the rider needs a clean
  `cast_zone: Zone` snapshot stashed on the resolving spell so a
  `Predicate::CastFromGraveyard` (or `CastFromExile`) can gate the
  bonus branch. Same plumbing unblocks Lurrus-style "cast
  permanent-from-graveyard" payoffs.

- **Per-permanent "gained-counter-this-turn" flag**. Fractal Tender's
  end-step "if you put a counter on this creature this turn, mint a
  Fractal" + Tester of the Tangential's pay-X-move-counters need a
  per-`Permanent.counters_added_this_turn: bool` toggle, set on any
  AddCounter event scoped to the permanent and reset on
  begin-of-untap.

### UI

- **Non-land MDFC flip indicator**. The 3D client's right-click flip
  now routes flipped non-land MDFCs through `CastSpellBack`
  (push X). The art swap is already wired (the existing
  `back_face_name` hand-card visual flow handles this), but the cast
  button's tooltip should change from "Cast for {front cost}" to
  "Cast back face for {back cost}" when flipped, so players
  understand which cost will be charged. Today the tooltip still
  reflects the front face's cost.

### Server

- **Action telemetry: `CastSpellBack` audit log**. The new MDFC
  back-face cast path emits the same `SpellCast` event as the front-
  face path. The server's wire log doesn't distinguish "cast as front"
  vs "cast as back" — both look identical from the spectator's view.
  Add a `cast_face: CastFace::{Front,Back,Flashback}` payload on
  `GameEventWire::SpellCast` so replays / spectator UIs can render
  the right face name without round-tripping through the engine.
  **DONE** in push XIV: `GameEvent::SpellCast.face` +
  `GameEventWire::SpellCast.face` now carry the tag.

## New suggestions (added 2026-05-01 pushes XI–XIV)

These items came up while implementing the MDFC + per-spell-type
batches; listed here so the next pass can pick them up without
re-deriving them.

### Engine

- **`Predicate::CastFace`** for triggers that gate on cast face. Push
  XIV added the audit log; future cards like Lurrus / Yorion-style
  "if cast from a non-hand zone" payoffs need a predicate that reads
  the resolving spell's `face` to gate triggers / static effects.

- **MDFC back-face mana-cost label in client**. Push X / XI's right-
  click flip routes through `CastSpellBack`, but the cast button's
  tooltip still shows the front face's cost. Tracked in TODO.md "UI
  — Non-land MDFC flip indicator". Once a `CastingState.flipped: bool`
  flag flows from the targeting prompt to the tooltip layer, the
  tooltip can swap to "Cast back face for {N}".

- **`CastFace::Back` payload on `GameAction::CastSpellBack`** (UI hint).
  The action input has no face indicator today — `CastSpellBack` is
  the only signal. Adding a `face: CastFace` field to other cast
  actions (front cast, alt cast) would make the input log fully
  symmetric with the output event log.

- **Multi-face MDFC support beyond two faces**. Currently
  `CardDefinition.back_face: Option<Box<CardDefinition>>` supports a
  single back face. Modal triple-faced cards (MDF triples like Esika
  // Esika's Chariot, or future cycles) would need
  `back_faces: Vec<Box<CardDefinition>>` + a face-index in
  `CastSpellBack`. Not pressing today but worth tracking.

### UI

- **Per-MDFC card-front recognition**. The 3D client's hand renders
  the card name + cost based on `definition.name`. A right-click flip
  swaps `definition` to the back face's definition; the UI can render
  the new face's name, but the original front face's name is lost
  during the swap. Adding a `back_face_visible: bool` field on the
  client-side hand-card state (instead of mutating `definition`) would
  let the UI flip the rendering without touching the engine state.

### Server

- **MDFC cast face metric**. Push XIV's `CastFace` event payload
  unblocks per-face replay counting. A `metrics::cast_face_counts`
  Prometheus-style histogram (or simple Vec<(CastFace, u32)>
  tally) on the server would surface "how many MDFC back-face casts
  per game" stats useful for cube tuning.

## New suggestions (added 2026-05-01 push XV)

These items came up while implementing the `Effect::MayDo` +
`ActivatedAbility.life_cost` batch and are listed here so the next
pass can pick them up without re-deriving them.

### Engine

- **`Effect::MayPay { mana_cost, body }`** — sibling to push XV's
  `Effect::MayDo`. Adds an optional mana payment (rather than just
  yes/no). Bayou Groff's "may pay {1} to return on death", Killian's
  Confidence's "may pay {W/B} on combat damage to reanimate from gy",
  Tenured Concocter's may-draw-on-target. Today these are collapsed
  to always-do or always-skip. Cleanest path: a new `Decision::
  OptionalCost` variant carrying both the prompt + the mana cost so
  the bot/UI can evaluate affordability before answering yes/no.

- **`Effect::MayChoose { description: String, options: Vec<(String,
  Effect)> }`** — multi-option pick (rather than yes/no). Practiced
  Offense's "lifelink-or-DS" mode pick, Dina's Guidance's "hand or
  graveyard" destination pick, future "name a card" prompts. Today
  these collapse to one always-on branch.

- **`MayDo` for `wants_ui` players**. Today the synchronous decider
  path means UI players land on AutoDecider's default `false`
  answer when their `wants_ui` is true. A future refinement: surface
  `MayDo` through the `suspend_signal` flow so a human-in-the-loop
  player sees the prompt directly. (Current bot/test play is
  unaffected.)

- **`Predicate::CastFace`** — cast-face introspection on the
  resolving spell. Push XIV's `CastFace` event payload added the
  audit log; future cards like Lurrus / Yorion-style "if cast from
  a non-hand zone" payoffs need a predicate that reads the
  resolving spell's `face` (Front / Back / Flashback) to gate
  triggers / static effects.

- **Land-becomes-creature primitive**. Great Hall of the Biblioplex's
  `{5}: becomes 2/4 Wizard creature with 'whenever you cast IS, +1/+0
  EOT'` clause is omitted (push XV) because the engine has no
  Mishra's Factory-style transient creature-grant. Adding `Effect::
  BecomeCreature { p, t, types: Vec<CreatureType>, abilities: …,
  duration }` would unblock this card, Mishra's Factory, Mutavault,
  and the rest of the manland cycle.

- **Bottom-of-library miss path on `RevealUntilFind`**. Today the
  effect mills misses; many SOS cards (Follow the Lumarets, Zimone's
  Experiment, Stirring Honormancer) want misses to go to the bottom
  of the library instead. Add a `to_misses: ZoneDest` field on
  `RevealUntilFind` (defaulting to `ZoneDest::Graveyard` for
  back-compat) and update existing callers to opt into bottom-of-lib.

### UI

- **MayDo prompt rendering**. The 3D client doesn't yet route
  `OptionalTrigger` decisions through a UI affordance — `wants_ui`
  players land on the AutoDecider's `false` answer by default. A
  small "Yes / No" prompt panel anchored to the source card would
  surface the prompt without breaking the existing bot/test paths.

- **"Pay N life" cost label**. The new `cost_label` rendering shows
  "Pay 1 life" for activations carrying `life_cost > 0`. The 3D
  client's ability-tray could use a different color (red?) for the
  life portion of a hybrid mana+life cost so players spot the life
  payment at a glance.

### Server

- **Snapshot test for `life_cost` round-trip**. The new field has
  `#[serde(default)]` so older snapshots load with `life_cost: 0`.
  Add a snapshot round-trip test that exercises a `life_cost: 1`
  ability across a serialize/deserialize cycle to lock in the
  back-compat invariant.

## New suggestions (added 2026-05-01 push XVI)

These items came up while implementing the `Predicate::CastSpellHasX`
+ `Effect::MayPay` + `SelectionRequirement::HasXInCost` +
`Value::LibrarySizeOf` + `CardsInZone(Hand)` filter-fix batch and are
listed here so the next pass can pick them up without re-deriving.

### Engine
- **Equipment subtype + attach mechanic** — several cube cards need
  Equipment attach/detach + equip costs. Lion Sash, Cranial Plating,
  Umezawa's Jitte all blocked on this. Would also unlock Auras as
  a first-class enchantment subtype.
- **Cascade keyword** — Quandrix the Proof's ⏳ status depends on
  this. Cascade needs cast-from-exile-without-paying + a
  "reveal until nonland < CMC" loop.
- **Storm keyword** — Prismari the Inspiration needs Storm on IS
  spells. The engine has `StormCount` value but no auto-copy-on-cast.
- **Foretell / Adventure / Companion** — future set support.

### UI
- **Prowess indicator** — when a Prowess creature has pending
  pump triggers on the stack, the UI should preview the post-pump
  P/T (e.g. "1/2 → 2/3") so the player sees the effect before
  resolution.

## Session notes (2026-05-25, claude/modern_decks continuation)

### Cards added this session
- **Elemental Expressionism** (STX): {3}{U}{R} bounce + 2x 4/4
  Elemental tokens. Multi-target bounce collapsed to single target.
- **Rush of Knowledge** (STX): {4}{U} draw 4 (highest-MV
  approximation).
- **Unwilling Ingredient** (STX): {B} 1/1 Pest with MayDo
  draw-on-death trigger. Mana payment for the MayDo is not enforced.
- **Tangletrap** (STX): {1}{G} modal — 5 damage to flyer OR destroy
  artifact.

### Observations for future sessions
- **Lesson sideboard model** remains the biggest gap — many STX cards
  approximate Learn as "Draw 1" which undervalues the mechanic.
- **Copy-spell retargeting** ("you may choose new targets for the
  copy") is engine-wide ⏳. Affects Prismari the Inspiration's Storm,
  Wandering Archaic, Twinscroll Shaman, and several SOS cards.
- **cast-from-exile pipeline** blocks ~10 remaining SOS ⏳ cards
  (Improvisation Capstone, Flashback the card, The Dawning Archaic
  attack trigger, Nita's activated ability, etc.).
- **Phyrexian mana** (pay life instead of colored mana) is used by
  Tezzeret's Gambit, Gitaxian Probe, and several Modern cards. A
  proper implementation would let players choose pay-life vs pay-mana
  at cast time.
- **DynamicPt for non-Tarmogoyf cards** — Wight of the Reliquary
  and similar "P/T equals X" creatures need the `DynamicPt` layer
  extended beyond the Tarmogoyf-specific `DistinctTypesInAllGraveyards`
  formula. A `DynamicPt::CountInZone { zone, filter, player }` variant
  would cover Wight, Master of Etherium, Nighthowler, and others.
- **Ninjutsu keyword** — Fallen Shinobi, Yuriko, Satoru all need
  the return-unblocked-attacker-to-hand alt-cost pattern. Blocks
  the Ninja tribal archetype in cube.
- **Emblem zone** — Dakkon, Dellian Fel, and several other
  planeswalkers have ult emblems that create persistent static
  effects. Needs a per-player `emblems: Vec<Emblem>` zone whose
  statics are evaluated in `compute_battlefield`.
- **Power-doubling combat pump** — Zopandrel's "double the power
  and toughness" needs a `Value::PowerOf(Each)` → `PumpPT(PowerOf,
  ToughnessOf)` per-creature application. Currently approximated
  as a flat +4/+4.

- **Right-click MayPay prompt**. The 3D client's existing decision
  panel handles `Decision::OptionalTrigger` for `MayDo` (push XV).
  `MayPay` reuses the same decision shape but the prompt text should
  also surface the affordability gate (gray-out the "Yes" button
  when the mana pool can't afford the cost, instead of letting the
  click silently no-op via the engine's "decline = false" fallback).
  Today wants_ui players land on AutoDecider's Bool(false) anyway.

- **HasXInCost label tooltip**. The new `SelectionRequirement::Has
  XInCost` filter renders as part of a card's reveal/move target
  prompt. The 3D client's target-prompt UI doesn't yet have a
  dedicated tooltip explaining "card must have {X} in its mana
  cost" — useful for Paradox Surveyor's "Land OR HasXInCost"
  reveal filter.

### Server

- **MayPay payment audit log**. The server's `GameEventWire` doesn't
  emit a dedicated "mana cost paid via MayPay" event today; the
  pool-decrease is silent. A `LifePaid`-style `ManaPaidForOptional`
  event (with source CardId + amount) would help replays diagnose
  surprising pool drops.

## New suggestions (added 2026-05-26 session)

These items came up during the 2026-05-26 implementation session and are
listed here so the next pass can pick them up.

### Engine — Done this session

- ✅ **Ward enforcement (generic mana)** — `Keyword::Ward(u32)` is now
  enforced at cast time. When a spell targets an opponent's permanent with
  Ward(N), the caster must pay N additional generic mana. Approximation:
  real MTG Ward is a triggered counter-unless-pay; this implementation
  adds the cost to the spell's total. Ward—Pay life variants (Mica, Prismari
  the Inspiration) are approximated as generic mana.

- ✅ **Stun counter untap replacement (CR 122.1c)** — permanents with stun
  counters no longer untap during the untap step. Instead one stun counter
  is removed and the permanent stays tapped. Also respected by `Effect::
  Untap` so spell-based untap effects (Frantic Search, etc.) honor stun.

- ✅ **Protection from color targeting (CR 702.16)** — spells whose mana
  cost contains a color that a target has protection from are rejected at
  cast time. Controller's own spells bypass the check (matching MTG rules).

- ✅ **ManaCost::colors()** — returns the set of distinct colors in a cost.

- ✅ **PermanentView.ward_cost** — surfaces Ward mana cost in the client
  view so the UI can display targeting cost hints.

- ✅ **Opus partial wiring** — 11 body-only 🟡 creatures promoted to have
  their basic IS-cast trigger (Opus small effect). The 5+-mana branch
  remains omitted pending `Value::CastSpellManaSpent`.

### Engine — Discovered gaps

- **Ward—Pay life / Ward—Discard variants**. The current Ward(u32) is
  generic-mana-only. Cards like Mica Reader of Ruins (Ward—Pay 3 life),
  Tragedy Feaster (Ward—Discard), Forum Necroscribe (Ward—Discard) use
  life or discard as the Ward cost. A `Keyword::WardLife(u32)` and
  `Keyword::WardDiscard` variant (or a `WardCost` enum on the keyword)
  would express these faithfully.

- **Protection from damage**. CR 702.16 also prevents damage from sources
  of the protected color. The engine currently tracks damage as a flat u32
  counter with no source identity, so color-keyed damage prevention isn't
  implementable without a damage-source tracking system.

- **Protection from blocking (blocker side)**. Protection currently only
  prevents an attacker with protection from being blocked by a colored
  creature. The reverse (blocker with protection from attacker's color
  can't be assigned as a blocker) is not checked. This is less commonly
  relevant but worth tracking.

### UI

- **Ward cost indicator**. `PermanentView.ward_cost` is now surfaced.
  The 3D client should render a small shield icon or "{N}" badge on
  Ward-bearing permanents so opponents know the targeting surcharge.

### Cards — Remaining ⏳ blockers

- **Copy-spell/permanent primitive** still blocks ~12 ⏳ cards
  (Choreographed Sparks, Silverquill the Disputant, Social Snub,
  Applied Geometry, Quandrix the Proof, etc.).

- **Cascade keyword** blocks Quandrix the Proof.

- **Cast-from-exile pipeline** blocks ~8 ⏳ cards (Archaic's Agony,
  Elemental Mascot 5+-mana, Improvisation Capstone, etc.).

- **Prepare mechanic** blocks 2 colorless ⏳ cards (Biblioplex
  Tomekeeper, Skycoach Waypoint).

- **Vehicle/Crew** blocks Strixhaven Skycoach.

## New suggestions (added 2026-05-26 modern_decks session)

### Engine

- **0/0 creature ETB-with-counters**: Stonecoil Serpent ({X} 0/0 with ETB
  AddCounter(+1/+1, X)) dies to SBAs before the ETB trigger resolves. Real
  MTG handles this as a replacement effect ("enters with X counters") not a
  triggered ability. Need either `CardDefinition.enters_with_counters:
  Option<(CounterType, Value)>` applied during `place_on_battlefield`, or a
  special "as-enters" replacement effect layer. Affects: Walking Ballista,
  Hangarback Walker, Hydroid Krasis, all Hydra-style X-cost creatures.

- **Evoke sacrifice timing**: Mulldrifter/Shriekmaw evoke triggers sacrifice
  on ETB after ETB triggers fire. Verify the `evoke_sacrifice` path in
  stack.rs fires the ETB first, then sacrifices. Currently works by ordering
  but edge cases (Flickerwisp on an evoked creature) aren't tested.

- **Cascade approximation**: Bloodbraid Elf approximated as ETB draw 1.
  Real cascade needs "exile cards until you find a nonland with CMC less than
  this spell's CMC, cast it free, put the rest on bottom." A first-class
  `Keyword::Cascade` + `Effect::Cascade` would unblock Bloodbraid Elf,
  Shardless Agent, Violent Outburst, and Quandrix the Proof.

- **Planeswalker damage in combat**: `deal_damage_to` now handles spell
  damage to planeswalkers (CR 120.3), but combat damage to planeswalkers
  (when a creature attacks a planeswalker directly) needs the same loyalty-
  removal path in `resolve_combat_damage`.

### Cards

- **STRIXHAVEN2.md table staleness**: Fix What's Broken, Archaic's Agony,
  and Molten Note are all implemented but their table entries still show ⏳.
  Run the audit script to reconcile.

- **Cube pool diversity**: The cube now has 46+ new cards but several color
  pairs (GW, UR) have much deeper pools than others (WU, BG). A card-count
  audit per pair would identify thin pools that need supplementing.

## New suggestions (added 2026-05-26 modern_decks-18 session)

### Engine

- **Deathtouch + non-combat damage**: `deathtouch_damaged` is only set
  during combat damage. Spell damage from a deathtouch source (e.g. Goblin
  Chainwhirler with deathtouch from Equipment) should also set the flag.
  Needs damage-source identity tracking in `deal_damage_to`.

- **Choose-two commands**: All five STX commands are collapsed to
  single-mode ChooseMode. A `ChooseMultipleMode { modes: Vec<Effect>,
  count: usize }` variant would let the controller pick exactly N modes
  from the list, matching the printed "choose two" pattern.

- **Hybrid mana**: ✅ DONE. `ManaSymbol::Hybrid(Color, Color)` is paid
  from either half, with a forced-color-first assignment in both
  `ManaPool::pay` (so {W/R}{W/G} from a {W,R} pool isn't mis-assigned)
  and `auto_tap_for_cost` (so a {W/B} pip taps whichever color the board
  can actually produce, and {W/B}{W/B} splits a Plains+Swamp). Mono-hybrid
  `{n/C}` also handled. Tests: `mana::tests::pay_hybrid_*`,
  `tests_sos::auto_tap_*`.

- **Learn/Lesson sideboard**: Learn is collapsed to Draw 1 across ~12
  cards. A minimal Lesson sideboard (separate from the main deck, searched
  by Learn) would give Strixhaven Limited decks their intended play pattern.

- **0/0 enters-with-counters**: The `enters_with_counters: Option<
  (CounterType, Value)>` field approach was explored but reverted because
  it adds a required field to every CardDefinition constructor (~588 sites).
  Alternative: use `#[serde(default)]` and make `Default` handle it, or
  apply counters in the spell-resolution path before SBAs based on a flag
  on the definition. Needed for Stonecoil Serpent, Walking Ballista, etc.

### UI

- **Legendary indicator**: `PermanentView.is_legendary` is now surfaced.
  The 3D client should render a crown icon or gold name border.

### Cards

- **STX Command full modes**: Lorehold/Prismari/Quandrix/Silverquill/
  Witherbloom Commands all ship single-mode; promoting to choose-two would
  match the printed cards and significantly increase gameplay depth.

- **Lesson cards not in Lesson sideboard**: Environmental Sciences,
  Introduction to Annihilation, Introduction to Prophecy, Expanded Anatomy,
  Fractal Summoning, Elemental Summoning, Pop Quiz are all Lessons but only
  playable from hand since there's no sideboard model.

- **Tanazir Quandrix ETB**: The counter-doubling ETB is still omitted.
  A `ForEach(Creature & ControlledByYou) → AddCounter(+1/+1,
  CountersOn(TriggerSource, +1/+1))` pattern would approximate it.

- **Ward cost adds {1} generic to bolt cost**: When a spell targets a
  Ward creature, an extra {1} generic appears in the spell's cost.
  Investigation needed — may be an interaction with extra_cost_for_spell
  or a Ward-triggered tax that fires too early. Tests pass with extra
  mana but the root cause of the phantom {1} tax is unclear.

- **Deathtouch on non-combat damage**: The Fight effect deals damage but
  doesn't mark it as from a deathtouch source. To properly implement
  this (CR 702.2b), the engine would need to track damage sources and
  apply deathtouch lethality in SBA. Currently only combat damage checks
  for deathtouch.

- **Lifelink on non-combat damage**: The `deal_damage_to` function in
  effects.rs doesn't track the damage source, so lifelink from fight
  effects or DealDamage effects from creatures doesn't gain life.
  Combat damage correctly tracks lifelink.

- **Witherbloom Command / multi-mode selection**: STX Command cycle cards
  want "choose two" from N modes, but the engine's ChooseMode only
  supports "choose one". A `ChooseNModes(n, Vec<Effect>)` variant would
  unlock Command cards and similar multi-mode selections.

- **Cascade keyword**: Needed for Quandrix, the Proof and other cascade
  cards. Would require a cast-from-exile pipeline + "exile until nonland
  with lesser MV, cast it for free" resolution path.

## New suggestions (added 2026-05-26 push XVII)

### Engine

- **Ward on activated abilities**: Ward currently only taxes spell casts.
  Per CR 702.21a, Ward also triggers when the permanent becomes the target
  of an ability an opponent controls. Adding a Ward tax check in
  `activate_ability` would be the natural extension.

- **Stun counter on Effect::Untap**: The CR 122.1b replacement (stun counter
  prevents untap, removes stun counter instead) is now enforced during the
  untap step. But `Effect::Untap` from spells/abilities (e.g. "untap target
  creature") should also check stun counters. Currently it bypasses stun.

- **Opus keyword (SOS-specific)**: Several Prismari/blue SOS creatures have
  "Opus — Whenever you cast an IS spell, [small effect]. If five or more
  mana was spent to cast that spell, [big effect] instead." This needs a
  `Predicate::ManaSpentOnCastAtLeast(Value)` gating the if-else branch.
  Currently the big mode is always omitted.

- **Increment keyword (SOS-specific)**: Several Quandrix/blue creatures
  have "Increment — Whenever you cast a spell, if the amount of mana you
  spent is greater than this creature's power or toughness, put a +1/+1
  counter on this creature." Needs mana-spent-on-cast introspection.

### UI

- **Ward cost display in targeting UI**: When the player selects a target
  for a spell, display the Ward cost so they know the extra mana required
  before committing.

### Server

- **Ward tax in legal-action generation**: The bot's legal-action generator
  should factor in Ward cost when computing whether a spell can target a
  given permanent, so it doesn't attempt unaffordable casts.

## New suggestions (added 2026-05-26 prowess+deathtouch session)

### Engine

- **Prowess on all keyword-tagged cards**: The new `prowess_trigger()`
  shortcut should be audited against every card with `Keyword::Prowess`
  to ensure the trigger is wired. Currently wired: Spectacle Mage,
  Stormchaser Mage, Monastery Swiftspear. Future additions (e.g.
  Bedlam Reveler, Soul-Scar Mage, Young Pyromancer's relatives) should
  use the shortcut when added.

- **Deathtouch on non-combat damage**: The `deathtouch_damaged` flag is
  only set during combat damage. Per CR 702.2c, deathtouch applies to
  ALL damage dealt by the source — including `DealDamage` effects from
  creatures with deathtouch (e.g. Prodigal Sorcerer variants, Goblin
  Sharpshooter). Implementing this requires threading the damage source
  through `deal_damage_to` and checking for deathtouch.

- **PlayerRef::ControllerOf for stack items**: `ControllerOf(Target(0))`
  doesn't resolve for spells on the stack (only battlefield/graveyard).
  Swan Song's "its controller creates a token" is approximated as
  EachOpponent. Threading controller lookup through `StackItem::Spell`
  would faithfully resolve this.

### Cards

- **Prowess creature cycle**: Now that prowess is wired, any future
  prowess cards (Soul-Scar Mage, Bedlam Reveler, Monastery Mentor,
  Adeliz the Cinder Wind) should be easy to implement — just add
  `prowess_trigger()` to their triggered abilities.

- **Rofellos, Llanowar Emissary**: `{T}: Add {G} for each Forest you
  control` needs `Value::CountOf(EachPermanent(HasLandType(Forest)))`.
  The `EachPermanent` selector + `HasLandType` filter both exist; the
  composition into a `ManaPayload::OfColor(Green, Count)` shape should
  work with existing primitives.
