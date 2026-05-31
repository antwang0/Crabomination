# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.
See `CUBE_FEATURES.md` (cube-card implementation status),
`STRIXHAVEN2.md` (Secrets-of-Strixhaven status), and `FEATURE_ROADMAP.md`
(prioritized engine functionality).

## Follow-ups noticed (not yet done)

- **Tracker staleness** — CUBE_FEATURES.md / DECK_FEATURES.md carry many 🟡/⏳
  rows that are already fully implemented + tested in code (verified this run:
  Opposition, Omniscience, Horizon Canopy / Sunbaked Canyon / Waterlogged
  Grove, Dismember, Spectral Procession, the shock/fast/surveil/bridge/pathway
  land families). A reconciliation pass (verify factory + cube registration +
  test, then elide the row) would shrink both trackers substantially. Started
  here; not exhaustive.
- **Card sourcing is data-blocked** — api.scryfall.com is outside the network
  allowlist and `scripts/cards_dump.json` (319-card pool) is fully implemented,
  so brand-new cards can only be added for staples whose exact stats/text are
  known cold. This run added 24 classic core-set bodies (`lea`); further bulk
  card work needs a Scryfall-equivalent data source in the sandbox.
- **Multi-target "choose two"** — `Effect::ChooseN` now allocates a target
  slot per chosen mode (Steal the Show's "one or both" ships). Remaining:
  bundled multi-mode cards still wired as a single `ChooseMode` of pairs
  (Cryptic Command), and *divided* targeting within one mode/effect (Vibrant
  Outburst, Snow Day, Crackle with Power — split-N / divided-damage slots).
- **Dynamic P/T CDA generalization** — characteristic-defining `*/*` P/T
  (Nightmare = Swamps you control, Master of Etherium) is hand-wired per card in
  `compute_battlefield` (Tarmogoyf pattern). A `StaticEffect::SetPtFromValue`
  layer-7b primitive would let Nightmare-class cards drop in.
- **More combat keywords** — Frenzy/Afflict/Afterlife shipped this run as
  trigger shortcuts; Melee (CR 702.121, needs an "opponents attacked this
  combat" Value), Provoke, Dash, Boast remain ⏳.
- **"Becomes a copy" continuous layer-1 effects** — the one-shot copiers
  (Clone, Phantasmal Image, Mirror Image, Stunt Double, Spark Double) ship via
  `Effect::BecomeCopyOf`. Still open: continuous layer-1 "becomes a copy"
  effects (Helm of the Host loop, Mirrorform), copied enters-with-counters,
  Mockingbird's name-retention exception, and a real copy-target picker
  (auto-picks highest power today).
- **Overload (CR 702.96)** — Cyclonic Rift's `{6}{U}` mode. Needs an
  alt-cost that rewrites "target X" → "each X" at cast time (the alt-cost
  model can't yet swap a selector's target into an each-selector).
- **Linked-exile return as a stack trigger** — `return_linked_exiles`
  returns the card directly rather than via a stack-based "when ~ leaves"
  trigger. Fine for observable behavior; only matters for response windows
  on the return (e.g. a board-wipe race).
- **Nexus of Fate graveyard replacement** — needs a
  shuffle-instead-of-graveyard replacement once a leaves-graveyard
  replacement primitive exists (the rest of the extra-turn pipeline ships).
- **Auto-target through step-keyed triggers** — `auto_target_for_effect`
  does not fill the target slot of a `Seq`-wrapped `CreateTokenCopyOf`
  source when the trigger fires from the command zone (emblem) or a
  step-keyed battlefield trigger, so Saheeli Rai's -7 emblem creates +
  fires but its copy body resolves to 0 tokens. Fix: thread auto-target
  through the `fire_step_triggers` push path the way
  `dispatch_triggers_for_events` does (or surface `Decision::ChooseTarget`
  for bot/auto deciders there).
- **Choose-N modes ("choose two")** — still open per `FEATURE_ROADMAP.md`
  Tier 1 (additional cast costs, `GrantActivatedAbility` static, and "when
  target dies this turn" delayed trigger already shipped).
- **Echoing Truth same-name bounce** routes every copy to `OwnerOf(Target0)`;
  mixed-ownership same-named permanents would all go to the target's owner.
  Needs a per-moved-card owner destination to be fully correct.
- **Nykthos UI** — the `DevotionOfChosenColor` payload suspends on a
  `ChooseColor` for wants_ui players; a devotion preview on the chip would
  help (the count is shown in the HUD already).
- **Theros gods left to add** — Heliod (God of the Sun), Purphoros, Pharika,
  and the two-color Theros: Beyond Death gods all reuse
  `StaticEffect::NotCreatureWhileDevotionBelow`; not added this run pending
  Scryfall-verified ability text (api.scryfall.com is blocked here).
- **Client build deps** — building the client in the web sandbox needs
  `libwayland-dev libasound2-dev libudev-dev libxkbcommon-dev` (install via
  apt). Once present `cargo build/clippy -p crabomination_client` works.

## MagicCompRules coverage audit

Periodic spot-check of the rules document
(`crabomination/MagicCompRules 20260116.txt` and the newer
`MagicCompRules_20260417.txt`). Each rule below has a status tag (✅
wired, 🟡 partial, ⏳ todo) plus a short note.

- ✅ **CR 702.130 — Enrage**

- ✅ **CR 702.68 — Frenzy** (claude/modern_decks). `shortcut::frenzy(n)` —
  `AttacksAndIsntBlocked / SelfSource` pump of `This` +n/+0 EOT (built on
  the existing `on_unblocked` helper). Silent when blocked. Tests in
  `tests/combat_keywords.rs`.

- ✅ **CR 702.131 — Afflict** (claude/modern_decks). `shortcut::afflict(n)` —
  `BecomesBlocked / SelfSource` trigger that makes `PlayerRef::DefendingPlayer`
  lose n life (resolved while the source is still attacking).

- ✅ **CR 702.135 — Afterlife** (claude/modern_decks). `shortcut::afterlife(n)`
  — `CreatureDied / SelfSource` trigger minting n 1/1 white-and-black Spirit
  tokens with flying.

  (Bushido CR 702.45, Flanking CR 702.25, Rampage CR 702.23 already ship as
  `Keyword::*` combat-step rules wired in `combat.rs::declare_blockers`.)

- ✅ **CR 700.5 — Devotion** (claude/modern_decks). `Value::DevotionTo(colors)`
  counts colored mana symbols among your permanents (hybrid/Phyrexian count
  per half). `StaticEffect::NotCreatureWhileDevotionBelow` gates the Nyx
  gods' creature-ness via a layer-4 `RemoveCardType(Creature)`, resolved
  against live devotion in `gather_continuous_effects`. `ManaPayload::
  DevotionOfChosenColor` powers Nykthos. Surfaced in `PlayerView.devotion`.
  Cards: Gray Merchant of Asphodel, Nylea, Thassa, Erebos, Nykthos.

- ✅ **CR 508.0 — "Attacks only alone"** (claude/modern_decks).
  `Keyword::AttacksAlone` — `declare_attackers` rejects a multi-attacker
  batch containing it. Master of Cruelties. Test:
  `master_of_cruelties_cannot_attack_alongside_another_creature`.

- ✅ **CR 603.6e — Linked "exile until ~ leaves"** (claude/modern_decks).
  `Effect::ExileUntilSourceLeaves` / `ExileChosenUntilSourceLeaves` stamp
  `CardInstance.exiled_by`; `return_linked_exiles` (called from every
  battlefield-removal path) returns the card to battlefield/hand when the
  source leaves. Banisher Priest, Fiend Hunter, Oblivion Ring, Brain
  Maggot, Tidehollow Sculler. Snapshot round-trips the link.

- ✅ **CR 702.92 — Battle cry** (claude/modern_decks). `shortcut::battle_cry`
  — `Attacks/SelfSource` trigger pumping `IsAttacking ∧ OtherThanSource`
  +N/+0. Goblin Wardriver. Tests: `cr_702_92_battle_cry_pumps_other_attackers_only`.

- ✅ **CR 702.83 — Exalted** — `shortcut::exalted` (already engine-wired);
  added card users Akrasan Squire / Aven Squire + a multi-source stacking
  test (`cr_702_83b_multiple_exalted_stack_on_lone_attacker`).

- ✅ **CR 702.135 — Mentor** (claude/modern_decks). Attacks/SelfSource
  trigger adding a +1/+1 counter to a target `IsAttacking ∧
  PowerLessThanSource ∧ OtherThanSource` creature. Sunhome Stalwart.
  Test: `cr_702_135_mentor_counters_lesser_power_attacker`.

- ✅ **CR 702.100 — Evolve** (claude/modern_decks). `shortcut::evolve` —
  `EntersBattlefield/YourControl` trigger gated on the entering creature
  (`TriggerSource`) being another creature with the new
  `SelectionRequirement::GreaterPowerOrToughnessThanSource`, adding a
  +1/+1 counter to `This`. Cloudfin Raptor, Experiment One, Fathom Mage
  (Fathom Mage chains a `CounterAdded(+1/+1)/SelfSource` → Draw). Tests:
  `cloudfin_raptor_evolves_when_bigger_creature_enters`,
  `experiment_one_does_not_evolve_for_equal_creature`,
  `fathom_mage_draws_when_it_evolves`.

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

- ✅ **CR 701.21 — Sacrifice**

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

- ✅ **CR 501 — Beginning Phase**

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

- ✅ **CR 503 — Upkeep Step**

- ✅ **CR 606 — Loyalty Abilities**

- ✅ **CR 504 — Draw Step**

- ✅ **CR 505 — Main Phase**

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
  (i) **122.1h** finality counters — ✅ (`CounterType::Finality`; the
  Battlefield→Graveyard move at `stack.rs:1438` redirects to exile when
  the permanent has a finality counter. Tested in `tests/stx/part_21`,
  `part_22`). (j) **122.1i** rad counters — ⏳ (no
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

- ✅ **CR 405 — Stack**

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

- ✅ **CR 406 — Exile**

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
  (a) **707.1** — ✅ (`Effect::CopySpell` resolves at cast time,
  stamping `StackItem::Spell.is_token = true` for permanent-spell
  copies. Permanent copies on the battlefield ship via
  `Effect::CreateTokenCopyOf` (Cackling Counterpart — token copy) and
  `Effect::BecomeCopyOf` / the `CardDefinition.enters_as_copy` hook
  (Clone, Phantasmal Image, Mirror Image, Stunt Double — a one-shot
  definition rewrite that locks the copiable values in at copy time).
  (b) **707.2** copiable values = printed name, mana cost, color
  indicator, types, rules text, P/T, loyalty (modified by other copy
  effects) — ✅ for spell and permanent copies (the rewrite reads the
  source's current `CardDefinition`; counters / damage / status are
  instance state and stay with the copier, not copied).
  Test: `cr_707_2_clone_copies_printed_pt_not_counters`.
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
  (h) **707.5** "enters as a copy" picks up ETB triggers of the copied
  object — ✅ (the `enters_as_copy` hook applies the copy *before* the
  first SBA sweep, so a 0/0 copier never dies first; the spell-resolution
  path then re-reads the copied definition's `EntersBattlefield`/SelfSource
  triggers and pushes those instead of the copier's. Test:
  `cr_707_5_clone_fires_copied_etb_trigger`. Remaining gap: copied
  enters-with-counters / "as enters" choices (707.6) aren't re-applied.)
  (i) **707.6** copying doesn't snapshot "as it enters" choices — ⏳
  (Clone-on-Adaptive-Automaton creature-type prompt deferred to copy
  controller; Adaptive Automaton not in catalog).
  (j) **707.7** linked-abilities preservation — ⏳ (no Linked
  Abilities primitive in the catalog).
  (k) **707.8** copy MDFC: use currently-up face — ⏳ (no MDFC
  permanent copies; the engine's `back_face` is consulted on cast but
  not on copy).
  (l) **707.9** copy modifications/exceptions ("except its color is
  black", "except it has flying") — 🟡 (`EntersAsCopy` carries
  `extra_creature_types` / `extra_keywords` / `extra_triggered`, so
  Phantasmal Image's "Illusion + sacrifice-when-targeted" and Stunt
  Double's "has flash" exceptions are modeled; color/P-T/supertype
  exceptions and the spell-copy path don't take exceptions yet).
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
  the Choreographed Sparks two-mode trial; permanent copies via
  `clone_enters_as_a_copy_of_a_creature`, `mirror_image_*`,
  `stunt_double_*`, `cackling_counterpart_*`, and the 707.2 test.
  Remaining ⏳: in-place copy (707.4), copied ETB triggers re-firing
  (707.5), MDFC-face copy (707.8), and static copy effects (707.2c).

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

- ✅ **CR 107 — Numbers and Symbols**

- ✅ **CR 109 — Objects**

- ✅ **CR 110 — Permanents**

- ✅ **CR 111 — Tokens**

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

- ✅ **CR 511 — End of Combat Step**

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

- ✅ **CR 119 — Life**

- ✅ **CR 117 — Timing and Priority**

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

- ✅ **CR 614.16 — "If an effect would create tokens / put counters,

- ✅ **CR 113.10b — "Loses all abilities" continuous effects**

- ✅ **CR 603.4 — Intervening 'if' clause (both halves now wired)**

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
  **Non-combat prevention is now wired** as per-target shields:
  `GameState.prevention_shields: Vec<PreventionShield>` created by
  `Effect::PreventNextDamage` (CR 615.7) / `PreventAllDamageThisTurn`,
  consumed in `deal_damage_to_from::apply_prevention_shields` (emits
  `GameEvent::DamagePrevented`, CR 615.13). `DamageCantBePreventedThisTurn`
  (CR 615.12) suppresses all shields for the turn. Wires Impractical
  Joke's rider + Healing Salve's prevention mode. Tests:
  `impractical_joke_damage_cant_be_prevented`,
  `prevention_shield_stops_noncombat_damage`,
  `healing_salve_mode_one_prevents_next_three_damage`. The combat path
  now routes attacker→player and trample→player/PW damage through the
  shields too (`combat.rs::prevent_combat_to_target`, lifelink scales off
  the post-prevention amount per CR 702.15a); test
  `prevention_shield_stops_combat_damage_to_player`. Gaps still tracked:
  (a) **creature-vs-creature** combat damage isn't routed through shields
  (a creature-scoped fog won't stop block damage to/from a blocker);
  (b) damage **redirection** (Maze of Ith); (c) per-source "next N from
  source X" shields.

- ✅ **CR 120.6 — Marked damage persists until cleanup; lethal damage

- ✅ **CR 120.4 — Four-part damage-dealing sequence**

- ✅ **CR 120.5 — Damage doesn't destroy a creature directly; SBA

- ✅ **CR 120.7 — Source of damage tracking**

- ✅ **CR 613.4c / 613.7c — Layer 7c (counter / +N/+M) applies above

- ✅ **CR 608.3f / 707.10f — Permanent-spell copies are tokens**

- ✅ **CR 707.10c / 707.10 — Copy effects and "new targets"**

- ✅ **CR 700.4 — Definition of "dies"**

- ✅ **CR 104.2b — "An effect may state that a player wins the game"**

- ✅ **CR 701.13 — Exile**

- ✅ **CR 701.16 — Investigate**

- ✅ **CR 700.1 — Events**

- ✅ **CR 701.17 — Mill**

- ✅ **CR 402.2 — Maximum hand size enforced at cleanup, opt-out via
- ✅ **CR 119.9 — Zero-life-gain emits no event**

- ✅ **CR 119.6 — Zero or negative life loses the game**

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

- ✅ **CR 608.2c / 701.6a — Later text on a card may modify earlier

- ✅ **CR 109.3 / 121 — Power and toughness can be read off the
- ✅ **CR 605.3a / 605.3b — Mana abilities resolve immediately without
- ✅ **CR 514.1 — Cleanup-step discard down to max hand size**
- ✅ **CR 614.12 — "Enters with N counters" replacement effects**
- ✅ **CR 701.14 — Fight**

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

- ✅ **CR 701.21 — Sacrifice**

- ✅ **CR 603.10a — Die-trigger scope filtering for the dying card**

- ✅ **CR 701.26 — Tap and Untap**

- ✅ **CR 701.25c — Surveil 0 emits no surveil event**

- ✅ **CR 701.7 — Create (tokens)**

- ✅ **CR 701.22b — Scry 0 emits no scry event**

- ✅ **CR 120.8 — 0-damage event suppression**
- ✅ **CR 702.90b — Infect damage to a player adds poison counters**
- ✅ **CR 702.180c — Toxic N**. `Keyword::Toxic(N)` gives the defending
  player N poison counters when the creature deals combat damage to them,
  stacking with normal damage and Infect. Wired in combat (`AttackerInfo`
  + the player-damage path); 10-poison loss SBA already present.
- ✅ **CR 305.6/.7 + 613 — basic-land type changes**.
  `Effect::BecomeBasicLand` swaps the type line and intrinsic mana
  (`intrinsic_land_mana_abilities` derives `{T}: Add <color>` from computed
  basic land types); `Effect::ResetCreature` sets a creature's P/T + types
  and strips abilities via the layer system (Oko, Turn to Frog).

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

- ✅ **CR 702.34a — Flashback exile-on-resolve**
- ✅ **CR 601.2f — Cost reductions can't take the mana cost below {0},

- ✅ **CR 121.4 / 704.5b — Decking out loses the game**

- ✅ **CR 700.2b — Modal triggered-ability mode chosen at push-time**
- ✅ **CR 120.3c — Damage to a planeswalker removes loyalty counters**
- ✅ **CR 613.4b — Layer 7b set-P/T sublayer**
- ✅ **CR 700.2d — Modal "choose more than one"**
- ✅ **CR 506.4 — Permanent removed from combat on zone change**
- ✅ **CR 502.4 — No priority during untap step**
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
- ✅ **CR 605.1a — Mana abilities (activated)**
- ✅ **CR 605.4a — Mana abilities don't go on the stack**
- ✅ **CR 707.2 — Copy characteristics**
- ✅ **CR 707.10 — Spell copies**
- ✅ **CR 707.10a — State-based action**
- 🟡 **CR 706 — Casting spells**: `cast_spell` covers the main path.
  Gaps: choose-additional-cost ("kicker"/"buyback" alternatives are
  via `alternative_cost`, but only one alt-cost can be active at
  cast time; multi-alt cycles aren't generalized).
- ✅ **CR 509.1i — Block triggers fire on blocker declaration**
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
- ✅ **CR 119.4 — Pay-life-only-if-life-≥-cost**
- ✅ **CR 603.6c — Leaves-the-battlefield abilities check first zone**
- ✅ **CR 603.10a — Graveyard-leave triggers look back in time**
- ✅ **CR 121.5 — Put-into-hand is not a draw**
- ✅ **CR 121.2 — Drawing cards one at a time**
- ✅ **CR 121.4 — Decking out loses the game**
- ✅ **CR 122.3 — +1/+1 and -1/-1 counters cancel**
- ✅ **CR 122.1d — Stun counter prevents next untap**
- ✅ **CR 122.6a — Counters on enter-with-counters**
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

- ✅ **CR 614.12 — Enters-with-counters replacement effects**

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

- ✅ **CR 302 — Creatures**

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

- ⏳ **Future batch — focus on engine-feature-unlocking cards**: priority
  candidates are Helix Pinnacle (keyword counter), Walking Ballista
  (Nth-counter trigger), and cards that exercise CR 122.4 (counter cap)
  / 122.7 (Nth-counter threshold trigger). Each lands new engine
  capability tracked in the rules-audit section above.

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

- 🟡 **`effect::shortcut::magecraft_loot()` callsite reduction** (push
  claude/modern_decks batch 107 — partial pass). Eight inline
  `magecraft(Seq([Draw 1, Discard 1]))` callsites across `stx::prismari`
  (3) and `stx::quandrix` (5) collapsed onto the existing
  `magecraft_loot()` helper. Remaining ⏳ inline callsites may still
  exist in `stx::extras` and other set modules — future cleanup pass
  can run the same regex sweep there.

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

- ⏳ **`PlayerRef::Opponent` (single-opponent helper)** — engine has
  `EachOpponent` (all opps) and `Target(_)` (cast-time targeting) but
  no "the singular non-controller opp" ref. In 2-player games these
  collapse to the same player, but `Selector::Player(PlayerRef::
  Opponent)` would read more naturally for single-opp effects (e.g.
  "target opponent draws a card" in Baleful Mastery). Workaround
  today is `EachOpponent` which fan-outs in multiplayer.

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

- 🟡 **Lesson sideboard model** — primitive landed. `Player.sideboard`
  holds Lessons "outside the game"; `Effect::Learn { who }` surfaces
  `Decision::Learn` (reveal a Lesson into hand / discard-to-draw /
  decline) via `DecisionAnswer::Learn(LearnChoice)`, and falls back to
  `Draw 1` when no Lessons sideboard is configured (so existing
  no-sideboard games and tests are unchanged). **All** Strixhaven Learn
  cards are now wired to `Effect::Learn` — the four canonical ones plus the
  Lessons that themselves Learn (Guiding Voice, Mascot Interpretation,
  Reduce // Rubble, Lesson in Honor) and Professor of Symbology.
  `cube::build_cube_state` seats each player with the standard
  `cube::lessons_sideboard()` via `GameState::add_card_to_sideboard`, so
  Learn fetches in real cube games. Covered by
  `tests::game::{learn_fetches_a_lesson_from_the_sideboard,
  learn_rummage_discards_then_draws, learn_decline_does_nothing}` and
  `cube::tests::build_cube_state_gives_each_seat_a_lessons_sideboard`.
  The client UI suspend flow is wired: a `wants_ui` player's Learn suspends
  on `Decision::Learn` (`PendingEffectState::LearnPending`) and the client's
  `decision_ui::spawn_learn_modal` / `handle_learn_buttons` render the
  reveal-a-Lesson / discard-to-draw / decline modal, submitting
  `DecisionAnswer::Learn(LearnChoice)`. Covered by
  `tests::game::learn_ui_player_suspends_and_resumes_via_submit_decision`.
  Remaining: populate sideboards in the other deck-build paths (formats /
  draft).
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
- `PermanentLeftBattlefield(CardId)` — needed for general "LTB" abilities.
  (Linked exile-until-LTB now handled directly via `return_linked_exiles`
  / `CardInstance.exiled_by`, not via an event.)
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
and Simian Spirit Guide (exile from hand: add mana) ship as vanilla bodies;
the "exile from hand: add mana" half needs a from-hand activation zone (adding
an `ActivatedAbility.from_hand` flag parallel to `from_graveyard` would mean
touching ~240 literal constructors — migrate them to `..Default::default()`
first).

### Delirium-conditional static buffs
`Predicate::DeliriumActive` now gates spell effects (Unholy Heat). A
*continuous* delirium buff — "as long as you have delirium, this gets +2/+2
and has flying" (Dragon's Rage Channeler, Traverse the Ulvenwald-adjacent
cards) — needs a layer-system static whose application is gated on a
predicate. DRC isn't implemented yet pending this.

### Client build can't be verified in the web sandbox
`crabomination_client` links Bevy, which needs the system `wayland-client`
library that isn't present here, so `cargo build/clippy -p crabomination_client`
fails at the `wayland-sys` build script. Engine + server changes are fully
verified; client-only edits (e.g. `keyword_label`) are reviewed by hand.

### Damage-as-(-1/-1)-counters replacement
Soul-Scar Mage / Phyrexian Vatmother-style "if a source you control would
deal noncombat damage to a creature, it deals that much in -1/-1 counters
instead" needs a damage-replacement hook. Soul-Scar Mage ships as 1/2 Prowess
without it.

### Phyrexian mana
Mutagenic Growth ({G/P}), Gut Shot, Dismember, etc. — a mana symbol payable
with 2 life. Mutagenic Growth ships at the {G} cost (the life-pay alt is
omitted).

### Source-relative mana-value search filter
`Effect::Search`/`SelectionRequirement::ManaValueAtMost(u32)` only take a
constant. Rushed Rebirth ("search for a creature with *lesser* MV than the one
that died") drops the relative constraint — it fetches any creature. A
`ManaValueLessThanSource`-style filter (paired with the `WhenTargetDiesThisTurn`
captured-source MV) would make it faithful.

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

### Counter-Removal Activation Cost
`ActivatedAbility` has no "remove N counters of kind K from the source as
a cost" field. Blocks Experiment One's `Remove two +1/+1 counters:
Regenerate this` (currently shipped Evolve-only), Walking Ballista's
`Remove a +1/+1 counter: deal 1`, Hangarback Walker, and the -1/-1-counter
sac-engines. Add `counter_cost: Option<(CounterType, u32)>` to
`ActivatedAbility`, paid after tap/mana/life like `sac_cost`.

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

The latter is more general. (Tidehollow Sculler / Banisher Priest /
Fiend Hunter are now handled by the dedicated
`Effect::ExileUntilSourceLeaves` / `ExileChosenUntilSourceLeaves`
primitives — see FEATURE_ROADMAP Tier-1 #4.) The former is smaller
surface but introduces effect-side mutation of ctx.

### Spend-Restricted Mana — ✅ DONE
Strixhaven's "Spend this mana only to cast an instant or sorcery
spell" (Hydro-Channeler, Tablet of Discovery's {T}: Add {R}{R}
ability, Abstract Paintmage's PreCombatMain trigger, Great Hall of
the Biblioplex, Resonating Lute's land-grant) is now wired. The mana
pool holds a separate `restricted: Vec<(Color, u32, SpendRestriction)>`
bucket (kept out of `total()`/`amount()`); `ManaPool::pay_for_spell(cost,
kind)` folds in the entries whose restriction permits the spell's
`SpellKind`, draining them ahead of unrestricted mana of the same
color. The catalog tags mana via `ManaPayload::Restricted(inner,
restriction)`; the spell-cast path threads the `SpellKind`. Covered by
`crabomination_base::tests::mana::restricted_*` and
`tests::sos::{tablet_restricted_mana_*, abstract_paintmage_*,
great_hall_pay_one_life_*, resonating_lute_grants_lands_*}`.

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
- Split `game_ui.rs` further ⏳ — the initial split into
  `systems/game_ui/{mod,crest,player_stats,buttons,popups}.rs` shipped;
  still to pull out: `sync_game_visuals` → `visual_sync.rs` (~1.1K lines),
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
✅ Click-based per-attacker picking is wired (`game_ui/mod.rs`, the
"Attacker selection" block): click an own creature to toggle it into the
plan, click an opponent planeswalker / player disc / 2-D HUD chip to
reassign the last-added attacker's defender, Esc / right-click to clear,
and `A` / the Attack button submits the picked plan (falling back to
"attack all eligible at next opp" when the plan is empty). Selected
attackers render gizmo diamonds (`gizmos.rs`).

⏳ Bigger lift still open: **drag an arrow** from attacker to defender /
planeswalker as an alternative to click-to-assign.

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
- Engine was already N-player aware (`pass_priority` uses
  `alive_count`, turn rotation uses `next_alive_seat`, attack target
  validation is bounds-checked).

#### Phase D — Multiplayer combat ✅
Each attacking creature chooses a defending player or a planeswalker
controlled by one of them; in 2HG the choice is the defending *team*
and damage may be assigned to either teammate's creatures/planeswalkers.

#### Phase E — Priority & APNAP for N players ✅
- Note: triggers within a single declare-attackers / declare-blockers
  batch (`game/combat.rs:50, 110`) share one controller (the active
  player), so APNAP within those is moot. The fix is concentrated on
  the unified dispatcher because that's the only fan-out path where
  multiple controllers can produce simultaneous triggers from one
  event.

#### Phase F — Shared life pool & shared turns (2HG) 🟡 (shared pool ✅; shared turn / cross-team triggers ⏳)
The 2HG-specific consumer of the teams abstraction.

**Shared pool — done:**

**Polish — done:**

**Still ⏳ (low-impact polish):**
- ⏳ Shared turn priority (CR 810.5) — strict "active team's primary
  player first, can yield to teammate" ordering. Current rotation
  is per-seat; both teammates already get priority in the
  4-passes-to-advance loop, so this is cosmetic.

#### Phase G — Team-aware loss & game end ✅
**G-lite done** (independent of Phase F):

**Shared-life half — now done via Phase F-3:**

#### Phase H — Replacement-effect framework (Commander prerequisite) ✅
- Known limitation (acceptable for Phase H scope): inline
  `graveyard.push` / `hand.push` / `exile.push` sites outside the
  three wired entry points bypass the resolver. Effects routed
  through `Effect::Destroy`, `Effect::Exile`-from-battlefield, and
  `move_card_to` all hit the wired paths; ETB-triggered direct
  pushes are the main gap and likely don't need replacement-effect
  coverage for Commander.

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
| Grim Lavamancer | Exile-2-from-GY additional cost | Low | ✅ (done — push modern_decks batch 114, `exile_other_filter` count `(Any, 2)`) |
| Bloodtithe Harvester | Sac-Blood ping (sac_cost activation) | Low | ⏳ |
| Dread Return | Flashback sac-3-creatures cost | Medium | ⏳ |
| Swan Song | Correct Bird token controller | Low | ✅ (done in earlier push — `PlayerRef::ControllerOf(Target(0))`) |
| Frantic Search | Untap cap (up to 3) | Low | ✅ (done in earlier push — `up_to: Some(Value::Const(3))`) |
| Windfall | Dynamic draw-equal-to-max-discarded | Medium | ✅ (done — push modern_decks batch 115, `Value::MaxCardsDiscardedThisEffectByAnyPlayer`) |
| Balefire Dragon | Dynamic "that much damage" (use creature's power) | Medium | ⏳ |
| Dark Confidant | CMC-dependent life loss | High (needs card-CMC Value) | ✅ (done — push modern_decks batch 111, `LoseLife(ManaValueOf(TopOfLibrary))`) |
| Rofellos | Forest-count mana scaling | Medium | ✅ (done — `Times(Const(2), CountOf(Forest))`) |
| Tidehollow Sculler | Exile-until-LTB primitive | High | ✅ (done — `Effect::ExileChosenUntilSourceLeaves` + `return_linked_exiles`) |
| Ichorid | Graveyard-color trigger filter | Medium | ✅ (done — push modern_decks batch 112, opp-GY black-creature `SelectorExists` gate) |
| Coalition Relic | Charge-counter burst | Medium | ✅ (done — push modern_decks batch 110, Remove-3-charge-counters WUBRG burst wired) |
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

- 🟡 **Engine — multi-mode picker with per-mode targets (CR 700.2d)**
  — `Effect::ChooseN` now gives each target-bearing mode its own
  `ctx.targets` slot, assigned by the mode's position among the
  target-bearing modes in the card's default `picks` (mode 0 → slot 0,
  mode 1 → slot 1, …). Cast-time validation keys off the same ordering
  in `target_filter_for_slot_in_mode`, so a "choose one or both" spell
  resolves a target per chosen mode — **Steal the Show is done** (target
  player for mode 0, target creature for mode 1). Covered by
  `tests::sos::{steal_the_show_runs_both_modes_with_per_mode_targets,
  steal_the_show_scripted_pick_runs_only_the_chosen_mode}`.
  Remaining (still ⏳): the mode set is the card's **default** `picks`, so
  a decider that runs a different subset still supplies targets in the
  default-picks slot order — full *cast-time* mode selection (a
  `Decision::ModePicks` shape surfacing N (mode, target) tuples) would
  let a human pick modes first, then target only the chosen ones. Also
  unblocks Moment of Reckoning ("choose up to four; same mode more than
  once").

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

- ✅ **Card — Wilt in the Heat's death-replacement rider** — "if that
  creature would die this turn, exile it instead" is now wired via
  `Effect::ExileIfWouldDieThisTurn { what }`, which records affected
  permanents in `GameState::dies_to_exile_eot` (cleared at cleanup). The
  redirect rides the existing finality-counter path in
  `remove_from_battlefield_to_graveyard` — the single choke point for all
  deaths (SBA lethal / destroy / sacrifice) — so it correctly spares
  indestructible creatures and catches deaths from later-this-turn combat
  or removal. Covered by `tests::sos::wilt_in_the_heat_*`. (The same
  primitive is reusable for future Path-to-Exile-style "exile instead of
  destroy" cards.)

- 🟡 **Card — first-class "may play this turn" pipeline** — Suspend
  Aggression / Ark of Hunger / Tablet of Discovery / Practiced
  Scrollsmith / Improvisation Capstone all exile/mill cards then "may
  play that card this turn". These are **already wired** via
  `Effect::GrantMayPlay` + `Selector::LastMoved` (cast through
  `GameAction::CastFromZoneWithoutPaying`) — and Improvisation Capstone
  ships both clauses (exile + may-cast via `CastWithoutPayingImmediate`,
  plus the Paradigm recurrence), so **the SOS catalog has no ⏳ cards
  left**. The remaining nicety would be a dedicated `Effect::ExileMayPlay
  { what, until }` + a `Player` side-list keyed by `(CardId, Duration)`
  to replace the per-card `GrantMayPlay` chains with one primitive.

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

### Suggested next-up tasks (additions from batch 26)

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

### Suggested next-up tasks (additions from batch 28)

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

- ✅ **Spend-restricted mana primitive** — DONE (see "Spend-Restricted
  Mana — ✅ DONE" above). `ManaPool` gained a `restricted` bucket +
  `pay_for_spell(cost, kind)`; mana is tagged via
  `ManaPayload::Restricted`, and `pay_for_spell` walks restricted mana
  first when the spell's `SpellKind` matches. Finished Hydro-Channeler,
  Great Hall of the Biblioplex, Resonating Lute, Abstract Paintmage, and
  Tablet of Discovery.

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

- ✅ **"If it would die this turn, exile it instead"** — DONE via
  `Effect::ExileIfWouldDieThisTurn` (see the Wilt in the Heat entry
  above). The death redirect is keyed on `GameState::dies_to_exile_eot`
  in `remove_from_battlefield_to_graveyard`, the same path the finality
  counter uses. (Note: the broader `Effect::PreventDamageThisTurn`
  damage-*prevention* primitive — Impractical Joke's "damage can't be
  prevented" no-op — is still ⏳; the death-replacement shape Wilt needed
  is done.)

- ✅ **Miracle alt-cost** — DONE. Lorehold, the Historian's "Each instant
  and sorcery card in your hand has miracle {2}" is wired via
  `Effect::GrantMiracle { what, cost }`: a CardDrawn/YourControl trigger
  gated on the first-IS-draw-this-turn stamps an until-end-of-turn
  `may_play_until` permission **plus** a `CardInstance::granted_alt_cast_cost_eot`
  of {2}. `cast_from_zone_without_paying` charges that alt-cost before the
  cast. The same `granted_alt_cast_cost_eot` slot generalizes to the
  Avacyn Restored miracle cycle. Covered by
  `tests::sos::lorehold_the_historian_grants_miracle_two_on_first_is_draw`.

- ✅ **Granted flashback (per-instance, EOT)** — DONE. The SOS "Flashback"
  instant grants `CardInstance::granted_flashback_eot` (= the target's own
  mana cost) via `Effect::GrantFlashbackThisTurn`; `cast_flashback` reads
  it through `CardInstance::effective_flashback()`. Covered by
  `tests::sos::flashback_instant_grants_flashback_on_gy_is_card`.

- ✅ **Granted cascade + `Keyword::CantBeCopied`** — DONE. Quandrix, the
  Proof's "instant and sorcery spells you cast from your hand have cascade"
  is a SpellCast/YourControl trigger gated on `Predicate::CastFromHand`
  (now stamped from the actual cast spell's flag, not the trigger default)
  + an IS filter, firing `Effect::Cascade { max_mv:
  ManaValueOf(TriggerSource) }`. Choreographed Sparks's "this spell can't
  be copied" rider is `Keyword::CantBeCopied`, honored by
  `Effect::CopySpell`. Covered by
  `tests::sos::{quandrix_the_proof_grants_cascade_*,
  quandrix_the_proof_does_not_cascade_*, choreographed_sparks_cant_be_copied}`.

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

- **`dies_mint_token_and_drain` shortcut** ✅ — mirror of
  `etb_mint_token_and_drain` for the on-death event (`effect.rs`).
  Used by "Pest dies → another Pest enters + drain" patterns.
- **`magecraft_mint_and_drain` shortcut** ✅ — composite of
  `magecraft_mint_token` and `magecraft_drain` (`effect.rs`):
  `magecraft(Seq[CreateToken(count), Drain(amount)])` for spells-
  matter Pest aristocrat decks where each instant/sorcery makes a
  body AND drains the table. Test:
  `tests::stx::shortcut_magecraft_mint_and_drain_seq_mints_then_drains`.
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
- **Static-ability tooltip "click to expand"** — the
  `static_ability_labels` field is wired and the client tooltip renders
  it; remaining polish is a "click to expand" affordance for long static
  descriptions on midrange/finisher creatures with multi-clause statics.
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

- **Rofellos, Llanowar Emissary** ✅ — `{T}: Add {G} for each Forest you
  control` is wired via `ManaPayload::OfColor(Green,
  CountOf(EachPermanent(HasLandType(Forest) ∧ ControlledByYou)))`. Test:
  `tests::modern::rofellos_taps_for_green_per_forest`.

## New suggestions (modern_decks: layer/keyword + alt-cost run)

- **Fire `EventKind::Tapped`** — the variant exists but is never
  dispatched. Wire it through the tap paths (combat declare, ability/cost
  tap, `Effect::Tap`, manual tap) behind a single `tap_permanent` helper so
  "becomes tapped" triggers fire (Magda Brazen Outlaw's Treasure-on-Dwarf-
  tap, Inspired-mirror effects). Guard against trigger loops.
- **Blank-permanent stubs** — Pithing Needle (name-a-card +
  activated-ability lockout) and Possibility Storm (cast replacement) are
  the only two `audit_stubs` flags; both need new primitives.
- **Self target-aware cost reduction** — Brush Off ("costs {1}{U} less if
  it targets an instant or sorcery") needs a per-card cost reduction keyed
  on the spell's own chosen target (distinct from the existing
  `StaticEffect::CostReductionTargetingFilter`, which is a permanent-static).
- **Multi-zone same-name exile** — `Selector::SharingNameWith` only spans
  the battlefield; Crumble to Dust / Runo-style "exile every card with the
  same name from library/hand/graveyard" needs a zone-spanning variant.
- **Cryptic Command true "choose two"** — convert the bundled `ChooseMode`
  pairs to `ChooseN { picks:[2], … }` once per-mode independent targeting
  lands (the blocker is multi-target-per-chosen-mode at cast time).
- **Plunge into Darkness mode 1** — now that `LookPickToHand` exists, the
  "pay X life, look at top X, take one" half can use it with an X-driven
  count instead of the flat tutor approximation.
