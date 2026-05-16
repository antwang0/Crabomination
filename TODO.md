# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.
See `CUBE_FEATURES.md` (cube-card implementation status) and
`STRIXHAVEN2.md` (Secrets-of-Strixhaven status).

## MagicCompRules coverage audit

Periodic spot-check of the rules document
(`crabomination/MagicCompRules 20260116.txt`). Each rule below has a
status tag (✅ wired, 🟡 partial, ⏳ todo) plus a short note.

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
  alone" / "blocking alone" — ⏳ (no `is_attacking_alone()` predicate;
  cards like Marauding Raptor / Battlemastery would need this).
  (h) **506.6** "had to attack" — ⏳ (no requirement-vs-choice
  tracking; cards like Brave the Sands' "creatures you control can
  block as though they could block two" don't reach the predicate).
  (i) **506.7** "cast only [before/after] [point]" timing — ⏳ (no
  cast-time predicate that gates on declare-attackers / declare-
  blockers step phase; cards like Pyrohemia, Tibalt's Trickery,
  Burst of Speed-style "play only during combat" would need it).
  No new tests added — the combat framework is exercised by every
  combat-damage test in the suite (CreatureDied via SBA, Sparring
  Regimen's per-attacker counters, Hofri/Quintorius anthems on
  attacking creatures). Promote to ✅ when the first-strike
  damage-step split and 506.5/.6/.7 land.

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

- 🟡 **CR 614.16 — "If an effect would create tokens / put counters,
  replacement effects apply" (token half)** (push modern_decks audit,
  claude/modern_decks branch): "Some replacement effects apply 'if an
  effect would create one or more tokens' or 'if an effect would put
  one or more counters on a permanent.' These replacement effects apply
  if the effect of a resolving spell or ability creates a token or puts
  a counter on a permanent, and they also apply if another replacement
  or prevention effect does so, even if the original event being
  modified wasn't itself an effect." Push (modern_decks) lands the
  **token half** of CR 614.16 via the new `StaticEffect::DoubleTokens`
  primitive. Wiring shape: a per-controller continuous static effect
  that doubles the count of every `Effect::CreateToken` resolution. The
  resolver in `game/effects/mod.rs` (`Effect::CreateToken` handler)
  queries `GameState::token_doublers_for(seat)` (in `game/mod.rs`) and
  multiplies the evaluated count by `2^k` where k is the number of
  active `DoubleTokens` permanents the controller has on the
  battlefield. Stacking multiplies (2 Adrix → 4×, 3 → 8×, ...) — the
  CR 614.13 "multiple replacement effects apply in any order" framing
  in this case collapses to "multiply" because every Adrix does the
  same doubling regardless of order. Adrix and Nev, Twincasters is the
  canonical exerciser. Tests: `adrix_and_nev_doubles_token_creation`,
  `adrix_and_nev_does_not_double_opponent_tokens`. The counter half
  ("if an effect would put one or more counters on a permanent" —
  Doubling Season, Hardened Scales, Branching Evolution) is still ⏳
  pending a `StaticEffect::DoubleCounters` sibling that hooks into the
  `Effect::AddCounter` resolver.

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

## Suggested next-up tasks

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

- ⏳ **Cast-from-graveyard introspection at resolution time** (suggested
  by push modern_decks's Increasing Vengeance addition + Antiquities
  on the Loose's stale 🟡) — cards like Increasing Vengeance ("if
  this was cast from graveyard, copy twice"), Antiquities on the
  Loose ("if cast from anywhere other than hand, +1/+1 counter on
  each Spirit"), and Goryo's Vengeance (Flashback-from-grave rider)
  need to read the spell's cast-source zone at resolution time. The
  engine already tracks `Card.cast_from_hand: bool` on the resolving
  spell. The gap is a `Predicate::CastFromHand` / `Predicate::
  CastFromZone(zone)` and a wiring of `EffectContext.cast_zone` so
  triggers and conditionals can read it. Adding these unlocks the
  "if cast from graveyard" rider on at least three current 🟡 cards
  + future Flashback-cared payoffs.

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
| Biorhythm | drain opponents to 0 | set each player's life to creature count |
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

#### Phase C — Team-aware opponent semantics 🟡 (code complete, untested)
**Code complete; suite blocked by an unrelated in-progress `WardCost`
refactor — see "Blockers" below.**
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

#### Phase D — Multiplayer combat ⏳
Each attacking creature chooses a defending player or a planeswalker
controlled by one of them; in 2HG the choice is the defending *team*
and damage may be assigned to either teammate's creatures/planeswalkers.
The `AttackTarget` enum already supports `Player(usize)` /
`Planeswalker(CardId)` (`game/types.rs:73`) and
`declare_attackers` already bounds-checks the target
(`game/combat.rs:24-28`).
- ⏳ Validate the chosen defender is an *opponent* (use
  `opponents_of(attacker_controller)`), not just "not self."
- ⏳ Block declaration: any creature controlled by a defending-team
  player may block (covers 2HG without code changes once the team
  abstraction is consulted).
- ⏳ Tests: 3-player FFA attack, 2v2 attack where partner blocks for
  the targeted player.

#### Phase E — Priority & APNAP for N players ⏳
- ⏳ Audit `pass_priority` for any 2-player assumptions in trigger
  ordering (the rotation itself already iterates via `next_alive_seat`).
- ⏳ Simultaneous triggers from one event must be sorted in APNAP order
  (CR 603.3b), each player ordering their own triggers.
- ⏳ Test: 4-player "deals damage to each player" event produces
  triggers from each player's permanents, resolved AP-first.

#### Phase F — Shared life pool & shared turns (2HG) ⏳
The 2HG-specific consumer of the teams abstraction. Depends on Phase E.
- ⏳ `Team.shared_life: Option<i32>` — `Some(30)` for 2HG, `None` for
  solo teams (each player keeps their own life).
- ⏳ Route all life mutations through a single helper that writes to
  the team's shared pool when set, otherwise the player's own life.
- ⏳ Shared turn: one *team* takes a turn; both members get priority
  in seat order during each priority window (CR 810.5). The active
  "player" for "you"-effects remains a specific seat — the partner
  acts on the active player's turn via timing rules.
- ⏳ Mulligan: each player mulligans independently (current 2HG rules
  per CR 103.5).
- ⏳ Tests: shared life updates from either teammate's damage; life-gain
  by one teammate triggers "whenever you gain life" for both (CR 810.8).

#### Phase G — Team-aware loss & game end ⏳
- ⏳ Player elimination still happens per-player on their own loss
  condition; team elimination triggers when all members have lost
  (in 2HG, also when shared life ≤ 0 even with players "in").
- ⏳ Game ends when only one *team* remains, not one player. Update
  `check_state_based_actions` (`game/stack.rs:899`).
- ⏳ Test: 2v2 ends correctly when shared life hits 0; 3p FFA continues
  with 2 after one elimination.

#### Phase H — Replacement-effect framework (Commander prerequisite) ⏳
Phase I's "commander goes to command zone instead of [graveyard | exile
| hand | library]" is the first true replacement effect. Build the
infrastructure here rather than as a one-off.
- ⏳ `ReplacementEffect` registry keyed on event
  (`ZoneChange { from, to, card_filter }`, `DamageDealt {…}`, etc.).
- ⏳ Resolve at the point of state change with a "would do X" hook
  returning a (possibly modified) event.
- ⏳ Scope tight initially — zone-change replacements only; expand as
  more cards demand it.

#### Phase I — Command zone runtime ⏳
- ⏳ Instantiate the already-declared `Zone::Command` (`card.rs:151`,
  `ZoneRef::Command` in `effect.rs:56`). Add
  `command: Vec<CardId>` to `Player` (per-player; `ZoneRef::Command`
  already expects a `player` field).
- ⏳ Zone-search & move-to-zone handling for `Zone::Command` in stack/
  effect resolution.

#### Phase J — Deck model with commander slot ⏳
- ⏳ Introduce `Deck { main, commanders: Vec<CardDefinition>,
  sideboard }`. `Vec` for commanders supports Partner / Background.
- ⏳ `Deck::load(format, …)` validates via existing `validate_deck()`
  plus the Commander-specific checks below.
- ⏳ On `GameState::new`, push each player's commander(s) into their
  command zone before opening hand draw.

#### Phase K — Color identity & deck validation ⏳
- ⏳ `color_identity(card) -> ColorSet` — union of mana-cost colors,
  mana symbols in rules text, and color indicators.
- ⏳ Commander legality: every card's color identity ⊆ commander's
  color identity; commander must be legendary creature (or carry a
  `can_be_commander` attribute).
- ⏳ Validation tests.

#### Phase L — Cast from command zone with tax ⏳
Depends on Phase H (replacement framework).
- ⏳ New action `CastFromCommandZone { card }` paying
  `mana_cost + {2} × prior_casts_this_game`.
- ⏳ Per-player counter `commander_cast_count: HashMap<CardId, u32>`.
- ⏳ Replacement effect: when commander would leave the battlefield to
  any of {graveyard, exile, hand, library}, owner *may* redirect it
  to the command zone instead (CR 903.9).
- ⏳ Reuses the `CastSpellAlternative` plumbing (`game/actions.rs:129`).

#### Phase M — Commander damage SBA ⏳
- ⏳ `commander_damage: HashMap<(victim, source), u32>` somewhere
  (on `GameState` or per-player).
- ⏳ Combat / direct damage from a commander accumulates here in
  addition to normal life loss.
- ⏳ New SBA in `check_state_based_actions`: if any entry ≥ 21, victim
  loses (CR 704.5v).

#### Phase N — Polish ⏳
- ⏳ Audit any remaining `PlayerRef::EachOpponent` / "your"/"opponent"
  effects in card catalog text for team-awareness (Phase C handles
  the engine layer; some cards may have bespoke logic).
- ⏳ CLI / deck-loader entry points should accept format.
- ⏳ Update format coverage tests after Phase J/K land.

---

#### Blockers
- **Unfinished `WardCost` refactor** (branch `claude/modern_decks`,
  unstaged WIP): `card.rs` introduced `enum WardCost` and changed
  `Keyword::Ward(u32)` → `Keyword::Ward(WardCost)`. ~9 call sites
  still pass `u32` and don't compile:
  `crabomination/src/catalog/sets/sos/creatures.rs:5444, 5609`,
  `crabomination/src/catalog/sets/sos/mdfcs.rs:1095, 1128, 1240`,
  `crabomination/src/catalog/sets/stx/iconic.rs:76`,
  `crabomination/src/game/actions.rs:809, 818`. The 7 catalog sites
  are mechanical (`Keyword::Ward(N)` → `Keyword::Ward(WardCost::generic(N))`);
  the two `game/actions.rs` sites need real API decisions on how the
  Ward trigger consumes the new variant. Phase C's tests cannot run
  until this is resolved.

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

