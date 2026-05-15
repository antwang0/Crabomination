# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.
See `CUBE_FEATURES.md` (cube-card implementation status) and
`STRIXHAVEN2.md` (Secrets-of-Strixhaven status).

## MagicCompRules coverage audit

Periodic spot-check of the rules document
(`crabomination/MagicCompRules 20260116.txt`). Each rule below has a
status tag (✅ wired, 🟡 partial, ⏳ todo) plus a short note.

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

- ⏳ **AutoDecider auto-targeting for additional_targets slots 1+** —
  push modern_decks promoted 8 SOS cards from single-target to
  multi-target via the `additional_targets: Vec<Target>` action shape +
  `Selector::TargetFiltered { slot, filter }`. The cast API + resolution
  paths read every slot correctly, but `auto_target_for_effect_avoiding`
  only fills slot 0. Slots 1+ stay empty under AutoDecider, so the
  optional creature halves on Cost of Brilliance / Vibrant Outburst /
  Render Speechless / Homesickness / Rabid Attack default to "skip" in
  bot games. Wiring suggested shape: extend
  `auto_target_for_effect_avoiding` to walk every slot index used by
  the effect tree and return `Vec<Target>` for slots 1+. Until landed,
  scripted tests carry the multi-target end-to-end coverage.

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

- ⏳ **`Effect::CopyUnlessPaid { what, mana_cost }` — opp-spell tax-or-
  copy gate (Wandering Archaic)** — push XXXVI lands the body of
  Wandering Archaic as an unconditional copy of every opp-cast
  instant/sorcery via the new opp-spell wire in
  `fire_spell_cast_triggers` (`EventScope::OpponentControl`). The
  printed Oracle's "that player may pay {2}. If they don't, …" tax-or-
  copy gate is engine-wide ⏳: it needs the opp's auto-decider to pay
  {2} from their pool (mid-trigger-push, not at resolution), and a new
  `Effect::CopyUnlessPaid` that branches on the payment. Wiring needs:
  (a) a yes/no decision on the opp's side at trigger-push time; (b)
  the engine's mana-pool pay path used from within the trigger
  dispatcher; (c) the trigger's `controller` (the listener, not the
  caster) stays the source's controller — only the `pay` step
  consults the *caster's* pool. Same primitive could power Mindbreak
  Trap-style "if your opp casts 3+ spells" tax-or-counter gates if
  generalized to `IfPaid { cost, then, else }`.

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

- ⏳ **Permanent-spell copy → token flag (CR 707.10f)** — `Effect::
  CopySpell`'s resolved permanent should set `CardInstance.is_token =
  true` when the copied spell is a permanent spell (Creature /
  Artifact / Enchantment / Planeswalker / Battle / Land). Currently
  only the stack copy is flagged; the resolved permanent isn't, so
  it persists past SBA cleanup of token-copies. Not currently
  blocking any catalog card (no copy-permanent-spell card wired) but
  needed for future Mizzium Transreliquat / Sakashima of a Thousand
  Faces-style permanent copy effects.

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
The engine has no replacement-effect primitive.  Many real cards need one:
- ETB replacements (Containment Priest, Torpor Orb, Rest in Peace)
- Damage replacements (protection, preventing damage)
- Draw replacements (Leyline of the Void)
- Death replacements (Kalitas, Oubliette)
Until this lands, cards with "instead" clauses are either stubbed or collapsed
into a close approximation.

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
  separate primitives (Ward, exile-until-X, copy-spell).
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

### Commander (1v1 or 4-player)
- 100-card singleton decks built around a legendary creature commander
- Command zone with commander-tax mechanic
- 40 starting life
- Commander damage loss condition
- Color-identity deck-construction enforcement
- Multiplayer turn order and attack direction

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

