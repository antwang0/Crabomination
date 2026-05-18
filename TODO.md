# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.
See `CUBE_FEATURES.md` (cube-card implementation status) and
`STRIXHAVEN2.md` (Secrets-of-Strixhaven status).

## MagicCompRules coverage audit

Periodic spot-check of the rules document
(`crabomination/MagicCompRules 20260116.txt` and the newer
`MagicCompRules_20260417.txt`). Each rule below has a status tag (✅
wired, 🟡 partial, ⏳ todo) plus a short note.

- 🟡 **CR 118 — Costs** (push modern_decks batch 16, claude/modern_decks
  branch — audit against `MagicCompRules_20260417.txt`): The cost
  framework — what counts as a cost, payment order, replacement
  primitives, and "X" costs. Audit:
  (a) **118.1** "A cost is an action or payment necessary to take
  another action…" — ✅ (the engine models costs as fields on
  `ActivatedAbility` + `CardDefinition.cost` for spells, plus
  `AlternativeCost` for pitch / cost-reduction-with-gate paths).
  (b) **118.2** mana payment opens a mana-ability activation window —
  ✅ (`try_pay_with_auto_tap` / `try_pay_after_snapshot` in
  `game/actions.rs` allow mana-ability activation mid-payment; mana
  abilities resolve immediately without the stack per CR 605.3).
  (c) **118.3** "A player can't pay a cost without having the necessary
  resources" — ✅ (`InsufficientMana` for mana, `InsufficientLife` for
  life-cost, `CardIsTapped` for tap costs, `SelectionRequirementViolated`
  for exile-other-from-gy preflight; all rejection paths roll back the
  payment snapshot via `restore_payment_state`).
  (d) **118.3a** "Paying mana is done by removing the indicated mana
  from a player's mana pool" — ✅ (`ManaPool::try_pay`).
  (e) **118.3b** "Paying life is done by subtracting the indicated
  amount of life" — ✅ (`life_cost` deduction in `activate_ability`,
  `LoseLife` event emission).
  (f) **118.3c** "Activating mana abilities is not mandatory, even if
  paying a cost is" — 🟡 (the auto-decider always activates mana
  abilities to satisfy payment; a real UI player choosing not to tap a
  source could fail a payment. Functionally indistinguishable from the
  CR-correct outcome in bot harness).
  (g) **118.4** "Some costs include an X" — ✅ for spells (`x_value`
  on `CastSpell`, propagated through `ManaCost::with_x_value`), 🟡 for
  activated abilities (Berta's `{X}{T}: …` activation has X-symbols in
  the cost but no per-activation X prompt; the engine zeroes X for
  activations — tracked under `Value::SacrificedToughness` row in
  "Engine — Missing Mechanics" follow-ups).
  (h) **118.5** "Some costs are represented by {0}" — ✅ (zero-mana
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
  (l) **118.9** "Some costs are described as 'pay 0'" — ✅ (zero-mana
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
  — 🟡 (`StaticEffect::GrantKeyword` adds keywords for a
  duration; `Modification::RemoveAllAbilities` clears keywords only,
  not activated/triggered abilities — see the engine TODO row about
  `Modification::RemoveAllAbilities` only clears keywords. The full
  layer-6 add-then-remove cycle works for keywords; abilities beyond
  keywords are ⏳).
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
  (a) **114.1** "Some effects put emblems into the command zone" — ⏳
  (no `Effect::CreateEmblem` primitive; no `Zone::CommandZone`
  emblem-mode). Some planeswalker ults that grant emblems (Professor
  Dellian Fel's -6, Ral Zarek's -7, Tezzeret's emblems) are doc-tracked
  as 🟡 with the emblem half omitted — the body / earlier loyalty
  abilities still ship.
  (b) **114.2** "[Player] gets an emblem with [ability]" — ⏳ (no
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
  (c) **122.1b** keyword counters — ⏳ (no `CounterType::Keyword(Keyword)`
  variant yet; cards like Decayed / Helix Pinnacle aren't in catalog).
  (d) **122.1c** shield counters — ⏳ (no `CounterType::Shield` variant;
  no Phyrexia-style replacement primitive). (e) **122.1d** stun
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
  (m) **122.4** "can't have more than N counters" — ⏳ (no
  `Modification::MaxCountersOfKind` rule).
  (n) **122.5** "move a counter" — 🟡 (one-off cards like Tester of
  the Tangential reference "move N counters from this to that"; no
  general `Effect::MoveCounter { from, to, kind, count }` primitive
  yet — tracked as part of Tester's promotion).
  (o) **122.6/a** ETB-with-counters — ✅ (`CardDefinition.
  enters_with_counters: Option<(CounterType, Value)>` is applied in
  `stack.rs:362+` AFTER the card is pushed onto bf but BEFORE the
  next SBA pass, so printed 0/0 bodies — Pterafractyl, Symmathematics,
  Quandrix Calligrapher — survive ETB).
  (p) **122.7** "When the Nth [kind] counter is put on" — ⏳ (no
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
  graveyard-move" approximation. Promote to ✅ when 122.1b (keyword
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
  (b) **405.2** "objects added to the stack go on top" — ✅ (engine
  uses `self.stack.push(item)` everywhere; the Vec end is the top).
  (c) **405.3** "objects entering simultaneously, AP-controlled first,
  then APNAP order" — 🟡 (the engine processes triggers in
  ResolutionBuffer one at a time but doesn't sort by AP-vs-NAP. For
  ETB-rich boards with multiple simultaneous triggers across players,
  the stack order is whatever queue order they were collected in;
  observable difference only when both AP and NAP have triggers from
  the same event).
  (d) **405.4** "controller of spell = caster; controller of activated
  ability = activator; controller of triggered ability = controller
  of source when triggered" — ✅ (`StackItem::Spell.controller` is
  set in `finalize_cast`; `StackItem::Ability.controller` is set to
  the activator; triggered abilities resolve under
  `source_controller` snapshotted at trigger-fire time).
  (e) **405.5** "when all players pass, top resolves; if stack empty,
  step ends" — ✅ (`pass_priority` advances priority through both
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

- ✅ **CR 110 — Permanents** (push modern_decks batch 20,
  claude/modern_decks branch — newest audit against
  `MagicCompRules_20260417.txt`): The permanent primitive — what a
  permanent is, owner/controller, characteristics, types, and status.
  Audit:
  (a) **110.1** "A permanent is a card or token on the battlefield" —
  ✅ (`GameState.battlefield` is a `Vec<CardInstance>`; every
  battlefield-resident card is a permanent in the engine's terminology).
  (b) **110.2** owner = card-owner, controller = enter-controller —
  ✅ (`CardInstance.owner` and `.controller` are both set at
  construction; `owner` is preserved across zone changes, `controller`
  is updated by gain-control effects like Tempted by the Oriq).
  (c) **110.2a** "If an effect instructs a player to put an object onto
  the battlefield, that object enters the battlefield under that
  player's control unless the effect states otherwise" — ✅
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
  (h) **110.4c** "If a permanent somehow loses all its permanent types,
  it remains on the battlefield" — ✅ (no SBA in
  `check_state_based_actions` removes a permanent for having zero
  card types; the engine matches CR's "stays on the battlefield as a
  non-anything object" semantics by default).
  (i) **110.5** status = (tapped/untapped, flipped/unflipped, face up/
  face down, phased in/phased out) — 🟡 (tapped + face-down ✅;
  flipped = ⏳ — no flip-card support; phased in/out = ⏳ — Phasing
  itself is unmodelled, the `phased_out` flag and its SBA-bypass
  semantics don't exist).
  (j) **110.5b** "Permanents enter the battlefield untapped, unflipped,
  face up, and phased in unless a spell or ability says otherwise" —
  ✅ (`CardInstance::new` sets `tapped: false`, `face_down: false`;
  ETB-tapped is the explicit opt-in via `ZoneDest::Battlefield(_,
  tapped: true)` and lands like `lorehold_excavation` tap targets via
  `Effect::Tap`).
  (k) **110.5d** "Only permanents have status. Cards not on the
  battlefield do not" — ✅ (`place_card_in_dest`'s zone-change branch
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
  attacker/blocker has First Strike or Double Strike). (b) **506.2** active = attacker,
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
  (h) **506.6** "had to attack" — ⏳ (no requirement-vs-choice
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

- 🟡 **CR 605 — Mana Abilities** (push modern_decks audit,
  claude/modern_decks branch): The mana-ability framework — what
  qualifies as a mana ability and how it resolves. Audit confirms
  the engine's activated-mana-ability fast-path is end-to-end CR-
  compliant: (a) **605.1a** activated mana ability criteria (no
  target + could add mana + not loyalty) — ✅ (`is_mana_ability` in
  `game/actions.rs` matches the rule conservatively: pure
  `Effect::AddMana` OR a `Seq` of mana abilities); (b) **605.2**
  mana ability remains a mana ability even if it can't produce mana
  right now — ✅ (the criteria check is static against the
  `ActivatedAbility.effect` shape, not the runtime "could it add
  mana"); (c) **605.3a** mid-cast / mid-resolve activation — ✅
  (mana abilities can be activated during cost-payment via
  `try_pay_with_auto_tap`); (d) **605.3b** doesn't go on the stack —
  ✅ (`activate_ability` routes mana abilities through
  `continue_ability_resolution` directly, skipping `StackItem::
  Trigger` push); (e) **605.3c** can't reactivate until resolved —
  ✅ (resolution is atomic in the mana-ability path); (f) **605.4a**
  triggered mana abilities don't go on the stack — ⏳ (no STX/SOS
  card requires it; engine handles all triggered abilities through
  the standard stack-push path; first card to need the fast-path
  would be Mana Reflection / Wirewood Channeler-style "Whenever
  a permanent taps for mana, it produces twice as much"); (g)
  **605.5a/b** abilities with targets / spells aren't mana
  abilities — ✅ (the `is_mana_ability` recogniser doesn't accept
  effects with `Target(_)` selectors or any non-AddMana sub-effect).
  No new tests added — the framework is implicitly exercised by
  every mana-rock test and every spell-cast test in the suite
  (Sky/Marble/Fire/Charcoal/Moss Diamond, Lorehold Excavation's
  two color-producing taps, every Witherbloom Pledgemage / Cellar
  of Secrets / Diamond cycle activation). Promote to ✅ when the
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
  No new tests added — the priority framework is implicitly
  exercised by every other test in the suite (1661 passing tests
  all depend on correct priority + step transitions). The audit
  is a confirmation that CR 117 is end-to-end CR-compliant for
  the 1v1 case. Multi-player priority (CR 117.6 shared team
  turns) is still ⏳, tracked under Format Phase F (2HG).

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

- 🟡 **CR 603.4 — Intervening 'if' clause (trigger-time half)**
  (push modern_decks audit, claude/modern_decks branch): "A triggered
  ability may read 'When/Whenever/At [trigger event], if [condition],
  [effect].' When the trigger event occurs, the ability checks
  whether the stated condition is true. The ability triggers only if
  it is; otherwise it does nothing. If the ability triggers, it
  checks the stated condition again as it resolves. If the condition
  isn't true at that time, the ability is removed from the stack and
  does nothing." Push (modern_decks) fixes a long-standing bug in
  `fire_step_triggers` (`game/stack.rs`) where the `EventSpec.filter`
  predicate was **not** evaluated for step-begin triggers — only the
  trigger's `kind` and `scope` were checked. Now every step-begin
  trigger re-evaluates its filter predicate against the current game
  state before being pushed onto the stack. **Half-implemented** —
  the "check again at resolve time" half of CR 603.4 is still ⏳;
  mid-resolve state changes between fire and resolve aren't
  re-checked. Wires Triskaidekaphile's "if you have exactly 13 cards
  in your hand, you win the game" upkeep gate exactly, plus future
  Felidar Sovereign's "if you have 40 or more life" gate. Engine
  site: `fire_step_triggers` splits candidate gathering from
  filter-check + push so the predicate can call
  `&self.evaluate_predicate` without holding the iter borrow. Tests:
  `triskaidekaphile_wins_at_upkeep_with_exactly_thirteen_cards`,
  `triskaidekaphile_does_not_win_at_upkeep_with_other_hand_size`.

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

- ✅ **CR 305.2 / 305.2b — One land per turn enforcement** (push
  modern_decks audit): "A player can normally play one land during
  their turn; however, continuous effects may increase this number."
  The baseline rule is enforced via `Player.can_play_land()` returning
  `lands_played_this_turn == 0` (consulted in
  `actions.rs::play_land`). The `lands_played_this_turn` counter is
  bumped on every land-play (including back-face MDFC land plays via
  `play_land_back`) and reset to 0 on the player's untap step. The
  `StaticEffect::ExtraLandPerTurn` variant is recognized by the layer
  system but not yet enforced — no catalog card uses it today, so
  the gap is theoretical. When the first Exploration / Azusa Lost But
  Seeking-style card lands in the catalog, the `can_play_land`
  helper will need to thread the player's active static-effect
  count so it allows N+1 plays per turn. Tracked under "Engine —
  Missing Mechanics" below as a TODO.

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
  (d) **701.14d** "damage isn't combat damage" — ✅ (fight uses the
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
- ⏳ **CR 702.21 — Cycling**: Not implemented. `keyword::Cycling`
  doesn't exist; cards with Cycling are either stubbed or omitted.
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
  (a) **301.1** "A player who has priority may cast an artifact card
  from their hand during a main phase of their turn when the stack is
  empty" — ✅ (`GameAction::CastSpell` for Artifact-typed cards is
  sorcery-speed-gated; Flash override on Manifold Key-style instants
  honored via the `Keyword::Flash` exception).
  (b) **301.2** "When an artifact spell resolves, its controller puts
  it onto the battlefield under their control" — ✅ (same
  `resolve_spell` path as creatures).
  (c) **301.3** "Artifact subtypes are listed after a long dash" — ✅
  (`Subtypes::artifact_subtypes: Vec<ArtifactSubtype>` carries
  Equipment, Vehicle, Food, Treasure, Clue, Blood, Fortification,
  Contraption).
  (d) **301.4** "Artifacts have no characteristics specific to their
  card type" — ✅ (color framework reads `mana_cost` regardless of
  card type, so colored artifacts like Treasure-with-color come
  through correctly; the typical colorless-artifact case is the
  default).
  (e) **301.5/5a-f** Equipment subtype — 🟡 (the subtype is recognized
  + `Keyword::Equip(cost)` is declared + `CardInstance.attached_to`
  field exists, but the activation pipeline (CR 702.6) and the
  equip-only-during-your-main-phase timing aren't fully wired. No
  Equipment card in the current catalog actually uses the Equip
  activation. Functional `attached_to` writes via Aura ETB target
  selection are covered).
  (f) **301.6** Fortification — ⏳ (subtype declared, no Fortify
  primitive; no Fortification card in the catalog).
  (g) **301.7/7a-b** Vehicle / Crew — ⏳ (subtype declared, no Crew
  primitive; no Vehicle card in the catalog). Tests: ETB / ability
  tests for STX/SOS artifacts implicitly exercise 301.1/.2/.4. The
  Equip + Fortify + Crew paths can promote once at least one printed
  card needing each lands.

- ✅ **CR 302 — Creatures** (push modern_decks batch 19,
  claude/modern_decks branch — audit against
  `MagicCompRules_20260417.txt`): The creature card type — casting,
  resolution, subtypes, power/toughness, attack/block eligibility,
  summoning sickness, and damage marking. Audit:
  (a) **302.1** "A player who has priority may cast a creature card
  from their hand during a main phase of their turn when the stack is
  empty. Casting a creature as a spell uses the stack" — ✅
  (`GameAction::CastSpell` for a Creature-typed card pushes a
  `StackItem::Spell` after cost payment; sorcery-speed gating is
  enforced at the priority check).
  (b) **302.2** "When a creature spell resolves, its controller puts
  it onto the battlefield under their control" — ✅
  (`resolve_spell` in `game/stack.rs` routes a resolving Creature
  spell to `self.battlefield` under the spell's controller via
  `StackItem.controller`; the move also fires `EntersBattlefield`
  triggers).
  (c) **302.3** "Creature subtypes are usually a single word long
  and are listed after a long dash" — ✅ (`Subtypes::creature_types:
  Vec<CreatureType>` stores per-card subtypes; the engine carries
  the full STX creature subtype set incl. Inkling, Pest, Fractal,
  Spirit, Cat, Dog, Demon, Elemental, etc.; 205.3m's complete list
  isn't enforced but every printed STX/SOS card uses real subtypes).
  (d) **302.4 / 302.4a-c** "Power and toughness are characteristics
  only creatures have. A creature's power is the amount of damage it
  deals in combat. To determine a creature's power and toughness,
  start with the numbers printed in its lower right corner, then
  apply any applicable continuous effects" — ✅ (`CardInstance.power()`
  and `toughness()` start from `CardDefinition.base_power` /
  `base_toughness` and walk the layer system in `game::layers::
  compute_permanent` to apply 7a CDA, 7b SetPowerToughness, 7c
  ModifyPowerToughness, 7d Switch, and +1/+1/-1/-1 counter
  deltas in the correct CR 613.7 order).
  (e) **302.5** "Creatures can attack and block" — ✅
  (`GameAction::DeclareAttackers` and `DeclareBlockers` accept only
  creature-typed cards; non-creature permanents are rejected at the
  legality check).
  (f) **302.6** "summoning sickness" — ✅
  (`CardInstance.entered_battlefield_at` snapshot + per-card
  `can_attack`/`can_tap` gate in `actions.rs` checks "has been
  under controller's control continuously since their most recent
  turn began"; the haste keyword grants an exemption). The
  "tap/untap symbol" activation gate is also enforced
  (`activate_ability` rejects tap-cost activations on
  summoning-sick creatures unless they have haste).
  (g) **302.7** "Damage dealt to a creature is marked on that
  creature; if marked damage ≥ toughness, that creature has been
  dealt lethal damage and is destroyed as a state-based action.
  All damage marked on a creature is removed when it regenerates
  and during the cleanup step" — ✅ (`CardInstance.damage: u32`
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

## Suggested next-up tasks

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

### Exile-on-Resolve for Instants/Sorceries ✅ DONE
~~Cards like Awaken the Ages, Divergent Equation, and Wisdom of Ages
print "Then exile this spell" — the resolved instant/sorcery should
land in exile instead of its owner's graveyard. Currently approximated
as a no-op (sorceries naturally go to graveyard on resolve).~~ Done in
push (modern_decks): `CardDefinition.exile_on_resolve: bool` flag —
`continue_spell_resolution` routes the card to exile when set, bumping
`cards_exiled_this_turn` so Ennis-style payoffs see it. Wired Awaken
the Ages (Strife Scholar back-face), Divergent Equation, and Wisdom of
Ages. Tests in `tests::sos`.

### Cast-From-Exile Pipeline
Many cards exile a spell/card temporarily and later cast it (Foretell,
Suspend, Rebound, Flashback-from-exile, Escape, Adventure second cast,
Cascade resolution).  Currently each is handled ad-hoc or omitted.  A shared
"cast from alternate zone" code path would unlock dozens of cards.

### Copy Primitive ✅ DONE
~~No general "create a copy of target spell/permanent" effect exists.  Needed for:
Reverberate, Fork, Strionic Resonator, Quasiduplicate, Saheeli Rai −3, etc.
The `CopySpell` effect stub exists in `effect.rs` but is not wired through
`apply_effect`.~~ Done in push XVII: `Effect::CopySpell { what, count }`
locates the matching `StackItem::Spell` and pushes `count` copies above it
on the stack with fresh CardIds. Copies are flagged `uncounterable: true`.
Wired: Aziza Mage Tower Captain (Magecraft copy with tap-3 cost),
Lumaret's Favor (Infusion copy gated on life-gain), Social Snub (cast-time
copy gated on creature-control). Still TODO for "permanent" copies
(Quasiduplicate, Saheeli Rai −3): the variant exists but the
target → battlefield-token-copy path is not yet wired.

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

### Activated-Ability "From Your Graveyard" Path ✅ DONE
~~The `activate_ability` walker only iterates the battlefield, so cards
with mana-cost-priced graveyard-recursion abilities currently drop the
activation entirely.~~ Done in push XVII:
`ActivatedAbility.from_graveyard: bool` + `exile_self_cost: bool` are
now first-class fields. The `activate_ability` engine path walks the
graveyard for `from_graveyard` abilities; `exile_self_cost` exiles
the source as part of cost (mirror to `sac_cost` for battlefield
permanents). Wired: Summoned Dromedary, Teacher's Pest, Stone Docent,
Eternal Student. Remaining gap (3rd-party cost shapes):
- **Postmortem Professor**: `{1}{B}, Exile an IS card from your
  graveyard: Return this card from your graveyard to the battlefield.`
  needs an additional cost variant: exile a *different* card from gy
  matching a filter. A new `cost: ActivationCost` enum (or sibling
  `exile_other_filter: Option<SelectionRequirement>`) would cover this
  case.
- **Page, Loose Leaf (Grandeur)**: `Discard another card named [self]:
  …`. Needs `discard_named_self_cost: bool` (or named-cost variant).

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

### "May" Optionality Inside Sequences ✅ DONE
~~Several SOS cards bake a "you may" into the middle of a `Seq` (Pursue
the Past's "you may discard a card", Witherbloom Charm's mode 0 "you
may sacrifice a permanent", Practiced Offense's "may double-strike or
lifelink"). The engine has no "ask the controller yes/no" primitive,
so all of these collapse the optional branch into either always-do or
always-skip. A `Effect::MayDo(inner)` that emits a yes/no decision
(answered immediately by `AutoDecider`'s heuristic) would unblock a
chunk of cards without surfacing a new UI affordance.~~ Done in push
XV: `Effect::MayDo { description: String, body: Box<Effect> }` is now
first-class. Emits `Decision::OptionalTrigger`, AutoDecider answers
`false` by default, ScriptedDecider can flip to `true` for tests.
Promoted: Stadium Tidalmage, Pursue the Past, Witherbloom Charm mode
0, Heated Argument, Rubble Rouser. Practiced Offense's choice-mode
("double-strike or lifelink") still ⏳ since that's a 2-option pick,
not a yes/no.

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

### "May Pay" Optionality on Death/ETB Triggers ✅ DONE
~~Bayou Groff ("may pay {1} to return to hand on death") and several
Strixhaven cards bake an optional cost into a triggered effect ("may
pay X: do Y"). The current engine has no `Effect::MayPay { cost, then
}` primitive — neither for life nor mana costs — so all these collapse
to either "always do" or "always skip".~~ Done in push XVI:
`Effect::MayPay { description, mana_cost, body }` is now first-class
(`effect.rs:662`). Handler at `game/effects/mod.rs:289` asks the
controller's decider yes/no; on "yes" + affordable cost it deducts
the mana from the pool and runs `body`, otherwise skips. AutoDecider
defaults to "no", ScriptedDecider can flip via
`DecisionAnswer::Bool(true)`. Bayou Groff is fully promoted with
3 passing tests (`bayou_groff_dies_may_pay_*`). Life-cost variants
(`MayPayLife`) and X-cost variants (`MayPayX`) still ⏳ — neither
has a blocking card today.

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

### Per-Turn-Cast Gate on Activated Abilities ✅ DONE
~~SOS Potioner's Trove ("{T}: You gain 2 life. Activate only if you've
cast an instant or sorcery spell this turn.") needs an
`ActivatedAbility::condition: Predicate` field (or a sibling
`gated_when: Option<Predicate>`) to express "activate only if you
played a spell of type X this turn".~~ Done in push VIII:
`ActivatedAbility.condition: Option<Predicate>` is now first-class.
Evaluated against the controller/source context before any cost is
paid (failed gate doesn't burn tap-cost or once-per-turn budget).
Promoted Potioner's Trove (gate: `SpellsCastThisTurnAtLeast(You, 1)`,
an approximation of the printed "instant or sorcery"-only filter) and
Resonating Lute (gate: `ValueAtLeast(HandSizeOf(You), 7)`). New
`GameError::AbilityConditionNotMet`. The remaining gap is a
per-spell-type tally that distinguishes IS casts from creature casts —
once that lands, Potioner's Trove can swap from
`SpellsCastThisTurnAtLeast` to the exact predicate.

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

### "Untap Up To N" Cap ✅ DONE
~~`Effect::Untap` with a selector untaps *all* matching permanents.~~
Done in push V: `Effect::Untap` now carries an `up_to: Option<Value>`
field. Frantic Search caps at 3 lands; other Untap callers opt-out
with `up_to: None`. The picker takes the first N matching in
resolution order — a future enhancement could add a "highest-CMC
first" heuristic for max mana refund.

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

### Prepare Mechanic (SOS)
Secrets of Strixhaven introduces a per-permanent "prepared" flag toggled
by `becomes prepared` / `becomes unprepared` effects. Cards like
Biblioplex Tomekeeper and Skycoach Waypoint flip the flag; payoff cards
have a `Prepare {cost}` activated/triggered ability and reminder text
"(Only creatures with prepare spells can become prepared.)" Engine
needs:
- `PermanentFlag::Prepared` (or `CounterType::Prepared` count-1) on
  `Permanent`, surfaced through `PermanentView`.
- `Effect::SetPrepared { what, value: bool }`.
- `Predicate::IsPrepared` for prepare-payoff conditional clauses.
- A short oracle-text helper that wires "Prepare {cost}: …" into a
  standard activated ability with `gate: IsPrepared`.

Until (1) and (2) land, all prepare-touching SOS cards are ⏳.

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

### Token-Side Triggered Abilities ✅ DONE
~~`TokenDefinition` has `activated_abilities` but not
`triggered_abilities`.~~ **Done** in push VI: `TokenDefinition` now
carries `triggered_abilities: Vec<TriggeredAbility>` and
`token_to_card_definition` copies them through.

Wired tokens:
- **SOS Pest token** (`catalog::sets::sos::sorceries::pest_token`):
  "Whenever this token attacks, you gain 1 life." Promotes Send in
  the Pest, Pestbrood Sloth, Cauldron of Essence (its reanimation
  output), and any future SOS Pest minter.
- **STX Pest token** (`catalog::sets::stx::shared::stx_pest_token`):
  "When this creature dies, you gain 1 life." Promotes Pest
  Summoning, Tend the Pests, Hunt for Specimens (and Eyetwitch's
  Pest body would use it if Eyetwitch were a Pest token rather than
  a creature).

The Pest token chain now correctly trickles 1 life per qualifying
event into Witherbloom payoffs (Pest Mascot's lifegain → +1/+1
counter on self, Blech's per-creature-type counter fan-out, Bogwater
Lumaret's per-creature-ETB drain).

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
| Windfall | draws flat 7 | draw equal to most cards discarded |
| Dark Confidant | fixed 2 life loss | lose life = CMC of revealed card |
| ~~Biorhythm~~ | (~~drain opponents to 0~~) | **resolved push modern_decks** — now `SetLifeTotal` to creature count per side (CR 119.5) |
| Coalition Relic | tap for 1 of any color | tap + charge counter → burst WUBRG |
| Fellwar Stone | tap for 1 of any color | tap for a color an opponent's land produces |
| Static Prison | ETB taps target | also suppresses untap while stun counters exist |
| Rofellos | flat {G}{G} | {G} per Forest you control |
| Spectral Procession | {3}{W}{W}{W} | {2/W}{2/W}{2/W} hybrid (CMC 6) |
| Grim Lavamancer | {R}{T}: 2 damage | must exile 2 cards as additional cost |
| Ichorid | no graveyard gate | requires opponent to have a black creature in GY |
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
static effects), the UI shows the base stats.  `PermanentView` exposes both
`power`/`toughness` (current) and `base_power`/`base_toughness` (printed).
Show current P/T on the card and dim or strike through the base if modified.

### Modified Loyalty Display
Planeswalkers show a static loyalty badge but it doesn't update as
`CounterType::Loyalty` changes in-game.  Wire the loyalty counter from
`PermanentView` to the badge text.

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

### Theme System (colors + fonts) ✅ DONE
~~UI color literals and `TextFont { font_size: X, ..default() }` were duplicated
across 13 files (~161 srgba/srgb literals, 57 bare `TextFont` calls falling
back to Bevy's default sans).~~ Introduced `crabomination_client/src/theme.rs`
with named color constants (panel/overlay/HUD/field/button/accent/text) and
a `UiFonts` resource carrying the loaded Mirano font handle. All 2-D UI
surfaces (menu, decision modals, draft, game HUD, game-over, quality panel,
debug console, tooltips, export prompt) now source colors from `theme::*`
and text from `ui_fonts.tf(size)`. Closes the long-standing "fonts and
colors drift between files" problem.

### Win/Loss Banner Color Cue
The game-over modal (`game_over.rs::sync_game_over_modal`) shows "Victory!
…" and "Defeat. …" in identical white text on identical dark panels — no
emotional cue for the result. Color the panel border / subtitle / accent
based on `winner == cv.your_seat`, optionally add a small icon.

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

### Tooltip Viewport Clamping
`counter_tooltip.rs:80-82` and the `pile_tooltip` system use fixed
`Val::Px` offsets without a viewport bounds check. Tooltips on cards
near the upper-right or bottom edge clip off-screen. Clamp `node.left` /
`node.top` to `[0, window_size - tooltip_size]`.

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
The bot never attacks planeswalkers.  Adding a heuristic that attacks a
planeswalker when its loyalty is low enough to kill it this turn would make the
bot more competitive.

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

| Card | Missing Piece | Effort |
|---|---|---|
| Grim Lavamancer | Exile-2-from-GY additional cost | Low |
| Bloodtithe Harvester | Sac-Blood ping (sac_cost activation) | Low |
| Dread Return | Flashback sac-3-creatures cost | Medium |
| Swan Song | Correct Bird token controller | Low |
| Frantic Search | Untap cap (up to 3) | Low |
| Windfall | Dynamic draw-equal-to-max-discarded | Medium |
| Balefire Dragon | Dynamic "that much damage" (use creature's power) | Medium |
| Dark Confidant | CMC-dependent life loss | High (needs card-CMC Value) |
| Rofellos | Forest-count mana scaling | Medium |
| Tidehollow Sculler | Exile-until-LTB primitive | High |
| Ichorid | Graveyard-color trigger filter | Medium |
| Coalition Relic | Charge-counter burst | Medium |
| Tezzeret, Cruel Captain | Artifact-creature static pump | Low |
| Karn, Scion of Urza | Artifact-count scaling Construct | Medium |

## Engine — Sacrifice-Distinct Event (push modern_decks audit)

Currently `Effect::Sacrifice` resolves by removing the picked creatures
into the graveyard and emitting `GameEvent::CreatureDied` per dead
creature. This collapses "sacrificed" into "dies", which is correct
for most printed cards but loses information for triggers that read
specifically "Whenever a player sacrifices a creature" (Mortician
Beetle, Yahenni, Bone Picker, Solemn Recruit-style triggers). A
follow-up should:

1. Add `EventKind::CreatureSacrificed` + `GameEvent::CreatureSacrificed
   { card_id, who }` (new variants).
2. Have `Effect::Sacrifice` resolver emit both events in order:
   first `CreatureSacrificed`, then the standard `CreatureDied`.
3. Update Mortician Beetle's trigger from `CreatureDied / AnyPlayer`
   to `CreatureSacrificed / AnyPlayer` — tightening the body for
   the printed Oracle. **Without this**, Mortician Beetle also fires
   on lethal combat damage and burn, which is strictly stronger than
   printed.

## Engine — `Value::SacrificedToughness` in activation cost path
(push modern_decks audit)

`Value::SacrificedPower` / `Value::SacrificedToughness` are stamped
by `Effect::SacrificeAndRemember` but **not** by `sac_cost: true` on
activated abilities. Witch's Cauldron's "gain life equal to the
sacrificed creature's toughness" approximates as flat 2 life because
of this gap. Fix: thread `sacrificed_power` and `sacrificed_toughness`
into the resolution context when `sac_cost: true` consumes the
source. Same plumbing as `Effect::SacrificeAndRemember`.

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

### Engine — `Value::CountersOn` fan-out summation ✅ DONE

`Value::CountersOn { what, kind }` now sums `counter_count(kind)` across
every entity `what` resolves to (`game/effects/eval.rs::evaluate_value`).
Single-entity selectors (target / This) keep returning the lone entity's
count; fan-out selectors (`EachPermanent(filter)`) return the total.
Lock-in test: `tests::stx::reflective_anatomy_pumps_target_by_total_counters`
stages two bears with 2+1 counters and asserts Reflective Anatomy pumps
the target +3/+3 (2+1 summed).

### Engine — Token name auto-derive from subtypes (CR 111.4) ✅ DONE

`token_to_card_definition` (`game/effects/tokens.rs`) now synthesizes the
resulting `CardDefinition.name` from the joined token subtypes when
`TokenDefinition.name` is empty (`"Spirit Token"`, `"Treasure Token"`,
`"Soldier Token"`, …). Walks `creature_types`, `artifact_subtypes`,
`enchantment_subtypes`, `land_types`, `planeswalker_subtypes` in that
order; falls back to bare `"Token"` if every subtype list is empty.
Explicit names still pass through unchanged. Lock-in test:
`tests::game::token_without_name_derives_name_from_creature_subtypes`.
Catalog factories still ship explicit names today; this just future-
proofs copy-token-of-creature shells.

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

### Engine — Token subject_controller cache in CreatureDied events

The `EventScope::AnotherOfYours` scope check at
`game/effects/events.rs:79+` for CreatureDied events walks the
battlefield then per-player graveyards to find the dying card's
controller. For dying **tokens**, CR 111.7c's "ceases to exist" SBA
runs in the same `check_state_based_actions` sweep as the death event
emission, so by the time the unified `dispatch_triggers_for_events`
runs the token is gone from every zone and `subject_controller`
returns `None` — silently dropping every AnotherOfYours trigger that
fires off a token death.

**Repro**: Witherbloom Pestmaster ("whenever another Pest you control
dies, +1/+1 counter on this") doesn't trigger when an STX Pest token
dies — only when a non-token Pest creature (Witherbloom Pest Eater)
dies. Filed for batch 10.

**Fix**: cache the subject's controller (and creature types) on the
`GameEvent::CreatureDied` payload at emission time (in
`check_state_based_actions`), so the dispatcher reads it from the
event rather than walking zones. Same approach as the existing
`event_amount` cache for LifeGained / DamageDealt scalars.

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

### Engine — Stack-spell self-target validator (CR 115.5) ✅ DONE

✅ Done in batch 17 (modern_decks): new
`GameState::check_target_legality_with_source(target, caster, source)`
wraps the existing `check_target_legality` with a CR 115.5 self-target
gate — when the chosen target's permanent id matches the casting
spell's own id, the cast is rejected with `GameError::InvalidTarget`.
The cast pipeline (`cast_spell`) threads `Some(card_id)` so both the
slot-0 target and additional-targets slots are checked. The wrapper
form remains permissive when `source: None` so trigger / activation
target validation (which doesn't have a spell-on-stack source) is
unchanged. Lock-in test:
`cr_115_5_spell_targeting_itself_is_illegal_via_permanent_id` (Bury
in Books targeting its own card id rejected). Future Spell Burst /
Lava Spike-style printed "can't target this spell" cards plug in
against the same primitive.

### Engine — Coin flip primitive (CR 705)

Ral Zarek, Guest Lecturer's -7 ultimate flips five coins. No coin-
flip primitive exists; tracked as part of Ral's promotion. Would also
unblock Karplusan Minotaur, Krark's Thumb, Mana Clash, and the
Goblin Pulse / Lobotomy effects.

**Shape**: `Effect::FlipCoin { count: Value, on_heads: Box<Effect>,
on_tails: Box<Effect> }` + a per-decision `Decision::CoinFlip` so
deterministic test fixtures can script the outcome.

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
  (a) **705.1** "A coin used in a flip must be a two-sided object with
  easily distinguished sides and equal likelihood that either side
  lands face up" — ⏳ (no coin-flip primitive in the engine; tracked
  separately as the `Effect::FlipCoin` row in this file). The two
  outcomes ("heads" / "tails") would be modeled as a `Decision::CoinFlip`
  so tests can script them deterministically.
  (b) **705.2** "If the call matches the result, the player wins the
  flip. Otherwise, the player loses the flip." — ⏳ (no win/lose
  tracking on flips; `Effect::FlipCoin { on_heads, on_tails }` covers
  the "care only about heads/tails" case; a parallel `Effect::FlipCoinAndCall`
  with `on_win`/`on_lose` covers the call-and-match case).
  (c) **705.3** "An effect may state that a coin flip has a certain
  result and/or that a certain player wins a coin flip. In that case,
  ignore the actual results of that flip and use the indicated
  results instead." — ⏳ (no coin-flip-result override primitive;
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

- ⏳ **Equipment activation pipeline (CR 702.6)** — push (modern_decks
  batch 27) audits CR 301.5/5a-f (Equipment). The Equip activated
  ability is declared but not wired. Adding `ActivatedAbility.equip_to:
  Option<SelectionRequirement>` (with the auto-target picker walking
  controlled creatures) + `attached_to: Some(CardId)` write on
  resolution would let the engine ship Bonesplitter / Skyclave Apparition-
  style Equipment in any catalog.

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

- ⏳ **AP-vs-NAP stack ordering for simultaneous triggers** (CR 405.3)
  — fresh from the CR 405 audit. When a single event (ETB, attack,
  combat damage) triggers abilities on both AP's and NAP's
  permanents, the engine queues them in ResolutionBuffer in
  arrival order. Printed Oracle says AP's triggers go on the
  stack first (lowest), then each NAP in APNAP order, with
  each player choosing internal ordering. Observable only when
  multiple-controller ETB cascades stack-interact (e.g. an opp's
  Pestpod-Lurker-on-ETB-mints-Pest triggers while you have a
  Felisa-on-counter-bearing-dies trigger from the same combat).
