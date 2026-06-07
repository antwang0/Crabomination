# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.
See `CUBE_FEATURES.md` (cube-card implementation status),
`STRIXHAVEN2.md` (Secrets-of-Strixhaven status), and `FEATURE_ROADMAP.md`
(prioritized engine functionality).

## Follow-ups noticed (not yet done)

- ⏳ **Discovered this run (allied-color card batch):**
  - **Evoke keyword** — `AlternativeCost.evoke_sacrifice` exists but no
    `shortcut::evoke`; blocks Inner-Flame Acolyte and the Lorwyn evoke cycle.
  - **Multikicker + kick-count `Value`** — Wolfbriar Elemental ("a 2/2 Wolf for
    each time it was kicked") needs a multikicker count surfaced as a `Value`.
  - **"Draw your second card each turn" trigger** — Faerie Vandal, Mad Ratter,
    Wavebreak Hippocamp ("first spell during each opponent's turn") want
    per-turn draw/cast-ordinal trigger events.
  - **Search-by-name / search-an-Aura filters** — Squadron Hawk (up to 3 by
    name), Heliod's Pilgrim (an Aura card). `Effect::Search` has no name-match
    or Aura-subtype filter yet.
  - **"Other [type] creatures get +X/+Y" anthem** — `StaticEffect::PumpPT`
    lacks an exclude-self option, so Veteran Swordsmith-style "other Soldiers"
    lords self-buff. Needs an `exclude_source` flag or `Not(This)` selector.

- ⏳ **Discovered this run (sagas / attack-tax / pillowfort batch):**
  - **Attack-tax interactive pay** — `AttackTaxToController` auto-pays from the
    active player's floating mana; a wants_ui player needs a real "pay {N}?"
    prompt during declare-attackers (and a per-attacker / partial-pay choice).
  - **DFC / read-ahead Sagas** — `saga_chapters` covers single-faced Sagas only;
    transforming saga-lands (The Everflowing Well) and read-ahead chapter choice
    are still ⏳.
  - **`AddCardType` one-shot effect** — needed for "becomes an artifact in
    addition to its other types" riders (Phyrexian Scriptures chapter I), which
    blocks faithfully shipping that Saga.
  - **Variable attack tax** — Sphere of Safety / Collective Restraint scale the
    tax by a board count (enchantments / basic land types); needs a `Value`-typed
    amount on `AttackTaxToController`.

- ✅ **`AdditionalCastCost::ReturnToHand { filter, count }`** — mandatory
  "return N permanents you control to hand" additional cast cost (auto-picks
  the lowest-impact matches). Devour in Flames ("return a land you control").
- ✅ **Emerge (CR 702.119).** `AlternativeCost.emerge` + `shortcut::emerge` —
  sacrifice a creature, reduce the emerge cost generically by its MV. Wretched
  Gryff ✅. Remaining emerge cards (Elder Deep-Fiend's "tap up to four",
  Distended Mindbender's reveal-and-choose-two) need their cast-trigger riders.
- ✅ **Awaken (CR 702.113) + Surge (702.108) + Rally — OGW/BFZ blockers.**
  All three ship via existing primitives + a small `AlternativeCost.marks_kicked`
  flag. Awaken/Surge live in `shortcut::{awaken, surge, animate_land}`; Rally is
  an `EntersBattlefield`/`YourControl` trigger filtered to `HasCreatureType(Ally)`.
  Wired Sheer Drop, Mire's Malice, Coastal Discovery, Roil Spout (Awaken);
  Comparative Analysis, Containment Membrane, Boulder Salvo, Goblin Freerunner,
  Reckless Bushwhacker, Tyrant of Valakut (Surge); Kor Bladewhirl, Tajuru
  Warcaller (Rally); Wall of Resurgence, Cyclone Sire (animate-land riders).
  - ⏳ **Awaken-cast UI targeting.** The client alt-cast modal now offers a
    direct "Cast" for plain alt costs (Surge/Awaken/Emerge), but doesn't yet
    drop into the targeting cursor for the awaken land (and any base target).
    Bots/tests pass targets explicitly; the human UI needs an alt-cast →
    targeting follow-up so Awaken's land slot can be chosen.
- ⏳ **OGW/BFZ cards skipped this batch (need a primitive).**
  - **Oblivion Sower** — process-onto-battlefield (target opp exiles top 4,
    then put any number of *their* land cards from exile onto the battlefield
    under your control). Needs a "play lands from opponent's exile" move.
  - **Processor Assault** — Process as a cast-time *additional cost* (not a
    trigger); needs the additional-cost-process hook.
  - **Vile Redeemer / Inverter of Truth / Conduit of Ruin** —
    per-creature-died token scaling, whole-library-exile, and
    tutor+cost-reduction respectively. (Cyclone Sire ✅ — animate-land on death.)
  - ✅ **Thought-Knot Seer** — `Effect::ExileChosenFromHand` (non-linked exile)
    + `PermanentLeavesBattlefield` LTB draw. The SBA lethal-damage path now
    also fires `PermanentLeavesBattlefield` self-source triggers.
  - ✅ **Kozilek's Pathfinder** — `Effect::CantBlockSourceThisTurn` +
    `GameState.cant_block_pairs` (per-pair block restriction).
  - ✅ **Walker of the Wastes** — `PumpSelfByControlledPermanents` +
    `HasName("Wastes")`; a basic **Wastes** land (`{T}: Add {C}`) was added.
- ⏳ **Client crate can't be built/linted in the web sandbox** (missing the
  `wayland-client` system library; `wayland-sys` build script panics). Engine,
  catalog, base, and server crates all build + clippy-clean + test-green.
  Client edits are kept mechanical (mirror existing arms) and rustfmt-checked
  only. Pending sandbox-side verification: `keyword_label` now phrases Devoid /
  Landcycling / CantBeCounteredIfXAtLeast instead of raw `{:?}`.
- ⏳ **Test harness: `check_state_based_actions()` doesn't dispatch
  *another-creature-died* watcher triggers.** A creature killed via raw
  `damage = N; check_state_based_actions()` fires its own death (SelfSource)
  triggers but not other permanents' "whenever another creature you control
  dies" watchers — those need the full event-dispatch path (kill via a damage
  spell + `drain_stack`, as the Grim Haruspex / Sifter of Skulls tests do).
  Worth auditing whether the direct-SBA path should also gather watcher
  triggers, or whether this is purely a test-only shortcut.
- ⏳ **Eldrazi-titan pass leftovers (this run).** Remaining primitives:
  (a) **Process** ✅ — `Effect::Process { count, then }` (put N cards an
  opponent owns from exile into their graveyards; `then` is the "if you do"
  rider). Ships Wasteland Strangler, Mind Raker, Blight Herder. Still ⏳:
  Oblivion Sower (process puts *lands onto battlefield*, not graveyard) and
  Processor Assault (process as a cast-time *additional cost*, not a trigger).
  (b) **conditional static keyword grant** ✅ — Eldrazi Aggressor rides
  `StaticEffect::PumpSelfIf { keywords: [Haste], … }` gated on an
  `OtherThanSource` colorless-creature count.
  (c) **non-linked exile-from-opponent-hand** ("you choose a nonland
  card and exile it" + a separate LTB draw) — Thought-Knot Seer; (d) Reaver
  Drone ✅ — the `OtherThanSource` self-exclusion threads through the
  `SelectorCountAtLeast` upkeep-condition path correctly (verified by test).
- ⏳ **Hand of Emrakul / Spawnsire alt-cost & wish.** Hand of Emrakul's
  "sacrifice four Eldrazi Spawn rather than pay mana" alt-cost and Spawnsire's
  {20} cast-from-outside-the-game are both dropped (no sacrifice-N-of-a-type
  alt-cost / wish primitives).
- ⏳ **Goldvein Hydra death-treasure rider (LKI).** The dies trigger's
  `Value::PowerOf(This)` can't read the counter-boosted power because
  `died_card_snapshots` is cleared after the trigger dispatcher runs, before
  the stack trigger resolves (CR 603.10 full LKI for stack resolution is
  unmodeled). Carded without the rider. Fix needs either a snapshot kept alive
  through resolution or a captured-at-trigger-time value.
- ✅ **Collect Evidence which-cards picker.** A `wants_ui` controller now
  picks via `ChooseCards` (validated to clear the MV threshold, else declined);
  bots/tests keep the auto cheapest-pick. `collect_evidence_ui_picker_honors_chosen_cards`.
- ⏳ **"Up to one target" for Suspect (Reasonable Doubt).** Currently modeled
  as a required creature target; a true optional single-target slot would let
  it resolve with the counter clause alone.
- ✅ **Client suspect/goaded/monstrous badges.** `build_tooltip_body`
  (`systems/counter_tooltip.rs`) renders "(suspected …)" / "(goaded …)" /
  "(monstrous)" status lines from the wire flags. A 3D on-card glyph (vs.
  the hover tooltip) is still a possible follow-up.

- ✅ **Ferocious damage-can't-be-prevented rider (Wild Slash).** Shipped via
  `If(SelectorExists(EachPermanent(Creature ∧ ControlledByYou ∧
  PowerAtLeast(4))))` gating `DamageCantBePreventedThisTurn` — no new
  predicate needed (the `And`-composed requirement already expresses
  "you control a creature with power ≥ N"). Future Temur ferocious payoffs
  reuse the same gate.
- ✅ **Tap-down-target-player's-creatures (Sleep).** Shipped via
  `Selector::ControlledBy { who, filter }` (player-relative `EachPermanent`)
  + a synthesized player-target slot in `target_filter_for_slot`. Sleep taps
  + stuns every creature target player controls.
- ✅ **Color-change EOT (Crimson Wisps).** Shipped via `Effect::BecomeColor`
  (fixed-color layer-5 `SetColors`, sibling of `BecomeChosenColor`). Crimson
  Wisps grants haste + becomes red + cantrips.
- ✅ **Aura that grants +N/+N and a keyword.** The `simple_aura` helper
  (Attach + `equipped_bonus`) already covers plain creature Auras (Rancor,
  Spectral Flight). Shipped Untamed Hunger (+2/+1 menace), Mark of the Vampire
  (+2/+2 lifelink), Hammerhand (+1/+0 haste + can't block). The tap-down Auras
  Claustrophobia/Dehydration also ship via an aura-anchored
  `PreventUntap { applies_to: AttachedTo(This) }` (CR 502.3) + an ETB
  `Tap { AttachedTo(This) }`.
- **Look-at-hand riders (Peek, Telepathy).** Informational "look at target
  player's hand" has no mechanical primitive; only the cantrip half is
  modelable today.
- ✅ **Board-bounce to each card's owner (Aetherize / Evacuation).** Shipped
  via `PlayerRef::OwnerOfMoved`, resolved per-card in `place_card_in_dest`, so
  a single `Move { what: EachPermanent, to: Hand(OwnerOfMoved) }` routes each
  card to its own owner. Ships Aetherize / Evacuation. (AEther Gale's "six
  *target* nonland permanents" still needs a multi-target prompt.)
- **Evoke Incarnation faithfulness (MH2).** Subtlety's ETB targets any
  `IsSpellOnStack` rather than only creature/planeswalker spells (no
  card-type-on-stack filter yet). Endurance's "up to one target player"
  is narrowed to `EachOpponent` (no single-effect player-target slot —
  `ShuffleGraveyardIntoLibrary` takes a `PlayerRef`, not a targetable
  `Selector`). Add an `IsCreatureOrPlaneswalkerSpellOnStack` requirement
  (+ auto-target hook in `targeting.rs`) and a targetable player slot to
  promote both to fully faithful.
- **Graveyard-hate dies-trigger nuance.** `route_to_graveyard` /
  `ExileCardsBoundForGraveyard` redirect the *placement* to exile, but
  `remove_to_graveyard_with_triggers` still collects `CreatureDied` /
  LTB-to-graveyard triggers before the redirect. Under Rest in Peace a
  creature that's exiled-instead technically never "dies" (CR 700.4), so
  those dies-triggers shouldn't fire. Check `graveyard_exiled_for` before
  collecting dies-triggers to suppress them.
- **Modal 3-mode charms with per-mode targets** (Esper/Golgari/Azorius Charm).
  `ChooseMode` + per-mode `target_filter_for_slot_in_mode` works, but the
  2-color cube pools can't slot 3-color Esper Charm; add a guild-charm batch
  once a per-mode target picker / multicolor pool exists. Modes that need new
  primitives: "creatures gain lifelink EOT" mass keyword grant, "put attacking
  creature on top of library", split mill.
- **Oracle of Mul Daya / play-from-top-of-library.** Needs a
  "play lands from the top of your library" permission + top-card reveal.
- **Echo + ETB land destruction (Avalanche Riders).** Echo keyword exists;
  pair with `Effect::Destroy` over a land target.

- **Client modals for `ChooseMode` / `ChooseModes` / `DivideDamage` /
  `ChooseAmount` / `NameCard`.** `decision_ui.rs` only renders Scry / Search /
  PutOnLibrary / Discard / Mulligan / ChooseColor / Learn / OrderTriggers /
  ChooseTarget; the rest fall through `_ => {}`, so a networked human casting a
  modal spell (Commands, Callous Bloodmage) or an X-amount effect gets no
  picker and the seat degrades to the AutoDecider default. `ChooseMode` needs
  the mode label strings threaded onto `Decision::ChooseMode` (today it carries
  only `source` + `num_modes`); `effect_short_text` already renders each mode.
- **Amped Raptor energy free-cast (still 🟡).** Needs a `MayPlayPermission`
  alt-cost slot ("cast without paying mana by paying {E}{E}") + a cast-from-
  exile path that substitutes the energy cost.

- **Split-card follow-ups (CR 709 shipped this run).** The split primitive
  (`CardDefinition.split` + `CastSplitRight` / `CastSplitFused` / `CastAftermath`)
  and the bot/affordance wiring are in. Remaining:
  - **Client cast UI for the right/fused/aftermath halves.** The
    `splittable_right_hand` affordance now lights the cyan alt-cast border, but
    there's no modal to pick *which* half (left vs right vs fuse) — the click
    path only submits the left (`CastSpell`). Needs a small half-picker, like
    the MDFC face chooser.
  - **More split cards.** Easy faithful adds on the primitive: Dusk // Dawn
    (Dawn needs mass return-from-graveyard), Never // Return (needs targeted
    graveyard-card exile), Turn // Burn (Turn needs "becomes base 0/1, loses
    abilities"), Boom // Bust, Hide // Seek.
  - **Fused targeting** currently assumes each half is single-target (left →
    `target`, right → `additional_targets[0]`); a fusable card with a
    multi-target half would need the slot convention generalized.

- **Card primitives deferred this run (claude/modern_decks).** Real cards
  skipped for lack of a primitive — each is a small, reusable addition:
  - ✅ **"Whenever this blocks a creature, [affect that creature]"** — shipped
    via `effect::shortcut::blocks` + `Selector::BlockedAttacker` (resolves
    `block_map[source]`); Wall of Frost stuns the creature it blocks
    (`wall_of_frost_stuns_the_creature_it_blocks`).
  - ✅ **Rearrange-top-N** (look at top N, reorder, all stay on top — distinct
    from Scry which can bottom) — `Effect::RearrangeTop`; ships Index, Spire
    Owl, Sage Owl, and makes Ponder faithful. Tests in `modern.rs`.
  - **Protection-from-each-color as one keyword/state** (Metalcraft-gated
    multi-protection) — Etched Champion.
  - **Skyclave-Apparition-style "exile until leaves, then owner makes an X/X"**
    (linked-exile with a leave-replacement that mints a token instead of
    returning) — Skyclave Apparition.

- **Embalm/Eternalize token color + cost overrides.** `sets::akh` tokens ride
  `CreateTokenCopyOf` and gain a Zombie type (+4/4 for Eternalize), but the
  copy keeps the original's color and printed mana cost rather than becoming
  "white/black with no mana cost." Add `token_color: Option<Color>` +
  `strip_cost: bool` to `Effect::CreateTokenCopyOf` to make it faithful.
- **More AKH/HOU Embalm cards.** Vizier of Many Faces (embalm clone — needs the
  embalm-copy-any-creature path), Aven Wind Guide (token-anthem static),
  Heart-Piercer Manticore (ETB sac→ping). The existing `fanatic_of_rhonas`
  (decks::modern) is missing its real Eternalize {2}{G}{G} — upgrade it.
- **Earthshaker Khenra's "≤ its power" filter is fixed at 2.** The ETB
  can't-block uses `PowerAtMost(2)` (the printed power); the eternalized 4/4
  token still reads 2. A source-relative `PowerAtMostSource` requirement would
  make it exact.

- **Equip-granted triggers — general dispatch.** Skullclamp ✅ (the equipped
  creature's `CreatureDied` equip-grant is now collected on the death path in
  `resolve_stack`). Still ⏳: chaining `EquipBonus.triggered_abilities` (and
  Soulbond-granted triggers) into the general `dispatch_triggers_for_events`
  walk so *any* equip-granted trigger shape (ETB, attacks, draws, …) fires —
  today only `DealsCombatDamageToPlayer` (combat.rs) and `CreatureDied`
  (death path) are covered.
- **Ghost Quarter's basic-land search rider** is dropped (the destroyed land's
  controller may fetch a basic). Needs last-known-controller resolution after
  the land leaves; pairs with a `PlayerRef::ControllerOf(last-known)` lookup.

- **Soulbond pairing is auto-resolved (CR 702.95).** `apply_soulbond_pairing`
  pairs with the lowest-CardId eligible partner instead of prompting the
  controller. Add a `Decision::ChooseSoulbondPartner` (with a decline option)
  so a UI seat can pick / decline the pair.
- **Soulbond-granted triggered abilities only cover combat damage.**
  `SoulbondBonus.triggered_abilities` are dispatched via the combat
  `DealsCombatDamageToPlayer` hook only (enough for Tandem Lookout). A general
  path (chain them into `dispatch_triggers_for_events` like
  `granted_triggers_eot`) would cover any future soulbond trigger shape.
- **Dethrone (CR 702.105) has no catalog card.** The `dethrone()` shortcut +
  `Predicate::PlayerHasMostLife` are wired and tested, but the only printed
  Dethrone cards are complex (Marchesa, the Black Rose — needs "other creatures
  you control have dethrone" trigger-grant-to-filter + die-return recursion).
  Ship one when those primitives land.
- **Reconfigure unattach (CR 702.151) — ✅ engine.** `GameAction::Reconfigure
  { equipment, target: Option<CardId> }` attaches (`Some`) or detaches (`None`)
  for the reconfigure cost; unattach restores creature-ness. Remaining: a
  client UI affordance to trigger the unattach (the `E`-key equip flow only
  attaches today).
- **Warp / Miracle alt-cast keywords.** Two "cast-mode" keywords still dropped
  on their cards: Warp (Mightform Harmonizer, Pinnacle Emissary — cast cheaply,
  exile at end step, recast later — a Suspend/Plot-adjacent exile-and-recast),
  and Miracle as a real card-castable cost (Metamorphosis Fanatic — only the
  Lorehold "granted miracle" effect exists today, not a
  `CardDefinition.miracle_cost` field). **Offspring {N}** (CR 702.166) now ships
  via `Keyword::Offspring(cost)` reusing the Kicker pipeline (`has_kicker`
  returns the cost; `SpellWasKicked` gates an ETB 1/1 token-copy) — Thundertrap
  Trainer.
- **Card lookups now work offline.** `scripts/.scryfall_cache.json` has been
  expanded from 332 cards to the full Scryfall oracle set (~35.5k cards, every
  unique card keyed by name, with DFC/adventure front-face aliases), so the
  routine can implement any card without network access. Rebuild/refresh it
  with `python scripts/build_oracle_cache.py` (downloads the latest
  `oracle_cards` bulk and merges, preserving curated entries). Remaining card
  work: land monarch / Ascend / day-night payoff cards (the engine now
  supports all three) plus the long tail in `CUBE_FEATURES.md`.
- **Energy abilities as real costs.** `{E}{E}{E}: +1/+1` payoffs (Longtusk
  Cub, Bristling Hydra via `pay_energy_counter`) currently model the energy
  as an `Effect::PayEnergy` paid *at resolution* with `energy_cost: 0`, so
  they're technically activatable with no energy (the resolve no-ops). Now
  that `ActivatedAbility.energy_cost` exists, convert these to a true cost
  (gated up front). The bot's `pick_energy_payoff` now recognises both the
  `energy_cost`-bearing form and the resolve-time `Effect::PayEnergy` rider —
  remaining work is migrating the card definitions onto the real cost.

- **Energy-pay-to-cast-from-exile (Amped Raptor).** Needs a `MayPlay
  Permission` alt-cost slot ("cast without paying mana cost by paying {E}{E}")
  + a cast-from-exile path that substitutes the energy cost. Pairs with the
  existing `ExileTopAndGrantMayPlay` primitive.

- **Additional combat phase — main-phase variant (CR 505.1b).** The
  combat-phase loop ships (`Effect::AdditionalCombatPhase` +
  `GameState.additional_combat_phases`; Hellkite Charger-style combat-only
  activation re-loops Begin Combat at End of Combat). Still ⏳: main-phase
  sorceries that read "after this main phase, there is an additional combat
  phase followed by an additional main phase" (Relentless Assault, Aggravated
  Assault) — these need the extra combat (and main) inserted after the
  *current main phase*, not the End of Combat loop. Likely a small phase-queue
  on `GameState` consulted at both the main-phase and combat-phase exits.
- **Daybound / Nightbound DFC transform** (CR 702.145). The day/night game
  state + the CR 502.2 transition now ship (`GameState.day_night`,
  `Effect::BecomeDay`/`BecomeNight`, `Predicate::IsDay`/`IsNight`); what
  remains is the keyword that transforms a daybound/nightbound DFC as it
  becomes night/day, and the "any daybound permanent → it becomes day" /
  "any nightbound, no daybound → it becomes night" entry rules (702.145d/g).
- **The Initiative** (CR 726) reuses the monarch infrastructure (designation +
  combat-damage steal + leaves-game transfer) but needs Venture into the
  Dungeon / the Undercity (CR 701.49) for its payoff — implement the dungeon
  zone first, then the Initiative is a thin wrapper over the monarch pattern.
- **Client HUD for monarch / day-night / city's blessing — ✅ DONE.** The
  viewer's stat-chip row (`game_ui/player_stats.rs`) now spawns a crown chip
  (`👑`, CR 724) when the viewer is monarch, a `✦ blessed` chip (CR 700.6)
  when they have the city's blessing, and a `☀ day` / `☾ night` chip (CR 731)
  whenever the global day/night designation is set. Remaining: surface
  monarch on *opponents'* rows too (the chip row only renders the viewer
  today) and a board-center day/night ambient cue.

- **Block-restriction follow-ups (CR 509.1b).** The `CantBeBlockedExceptBy`
  filter matcher (`blocker_matches_block_filter`) covers type/color/keyword/
  P-T; "except by Walls/multicolored/specific subtype" compose already. Still
  needing other primitives: Signal Pest / Goblin Piledriver, Soldier of the
  Pantheon ("protection from
  multicolored" — a non-color protection grant). Brimaz's block-token rider
  and Whirler Rogue's "tap an artifact: grant unblockable" activated cost are
  also still ⏳.
- **`AffectedPermanents::CardMatch` could absorb P/T-gated anthems** if its
  matcher read *computed* power/toughness (it's card-printed-only today, so
  power/toughness thresholds still fall through to `None` — the P/T-gated lord
  gap noted under "Anthem coverage" below).

- **Protection on *ability* targeting + damage from spell sources.** CR
  702.16e/f are wired for spell targeting, equip, and the combat/noncombat
  *permanent*-source damage paths, but `check_target_legality` (activated/
  triggered ability targets) doesn't yet reject a protected target, and a
  *spell* damage source (Pyroclasm-style mass damage) isn't color-known at
  damage time (the card is in transient ownership), so its protection-from-
  color prevention degrades. Thread the resolving spell's color into the
  damage path and add a protection check to `check_target_legality`.
  Also: "protection from artifacts/colorless" (Giver of Runes, Apostle's
  Blessing's artifact mode) needs a non-color protection grant.
- **Per-player "half their own X" generalization.** `Effect::LoseHalfLife`
  scales to each target's own life; the same per-player pattern would finish
  Lord Xander (mill half *their* library, sacrifice half *their* permanents)
  — generalize to `Effect::MillHalf`/`SacrificeHalf` or a context-bound
  current-player ref so `Mill`/`Sacrifice` can read each target's count.
- **Anthem `affected_from_requirement` coverage.** Color (`HasColor`),
  `IsToken`/`NotToken` (→ `AffectedPermanents::All.token`, ships Intangible
  Virtue / Always Watching) are decomposed, and the opponent path
  (`ControlledByOpponent`) composes with type filters regardless of And-tree
  order. Remaining: power/toughness thresholds still fall through to `None`
  (anthem silently doesn't apply) — needed for P/T-gated lords.
- **Plague Engineer / named-creature-type -1/-1.** Needs a
  `StaticEffect` that diminishes only a chosen creature type among opponents
  (the existing `DiminishCreaturesExceptChosenType` is the inverse). Dropped
  this run to avoid an inaccurate flat anthem.
- **"Can't be blocked except by …" restrictions — ✅ DONE (primitive).**
  `Keyword::CantBeBlockedExceptBy(filter)` / `CantBeBlockedBy(filter)` (CR
  509.1b) are read in `can_block_attacker_computed` via
  `blocker_matches_block_filter` (a computed-characteristic matcher: type,
  color, keyword, power/toughness thresholds). Ships Silhana Ledgewalker
  (except by flyers) and Steel Leaf Champion (not by power ≤ 2). Remaining
  consumers: Goblin Piledriver / Soldier of the Pantheon (these have other
  riders — protection-from-color is their real evasion), Signal Pest.
- **Choose-color-on-ETB mana rocks — ✅ DONE.** `Effect::ChooseColorForSelf`
  stamps `CardInstance.chosen_color` at ETB; `ManaPayload::ChosenColorOfSource`
  taps for it. Coldsteel Heart shipped. Star Compass (basic-land-type gated)
  can reuse the primitive once its condition is wired.
- **Unleash bot nuance.** `optional_trigger_beneficial` accepts the Unleash
  +1/+1 counter as pure upside, but the counter disables blocking
  (`Keyword::CantBlock`). A defensive bot should weigh board state before
  taking it.

- **Adventure / Plot client modals** (CR 715 / 702.170). Engine + bot +
  affordance hints (`adventurable_hand` / `plottable_hand`) ship, but a
  `wants_ui` human gets no modal to *choose* between casting the creature vs.
  the adventure half, or to plot a card / cast it from exile later. Wire a
  client cast-mode picker off the new affordance sets (mirror the kicker /
  bestow toggle). `CastAdventureCreature` / `CastPlotted` from exile also have
  no client surface yet.
- **Protection-from-chosen-color grant — ✅ DONE.**
  `Effect::GrantProtectionFromChosenColor { what, duration }` surfaces
  `Decision::ChooseColor` then grants `Keyword::Protection(color)` for the
  duration (Mother of Runes, Gods Willing wired). Spell-targeting protection
  now reads *computed* keywords so the granted protection is honored.
  Remaining: protection isn't checked on *ability* targeting
  (`check_target_legality`) or combat-damage prevention reads — extend those
  to read computed protection if a card needs it (Giver of Runes "protection
  from colorless" also needs a colorless option).
- **Suspend (CR 702.62) — ✅ DONE (primitive + haste + accelerant).**
  `Keyword::Suspend(n, cost)` + `GameAction::Suspend` + `process_suspend`
  ship the exile-with-time-counters → tick-at-upkeep → free-cast loop
  (Rift Bolt, Ancestral Vision, Lotus Bloom). A suspend-cast creature now
  gains haste (CR 702.62f) via `CardInstance.cast_from_suspend`; Deep-Sea
  Kraken's accelerant ships via `Keyword::SuspendAccelerant` +
  `process_suspend_accelerants` (opponent's cast ticks a time counter).
  Remaining: the free cast auto-targets via the AutoDecider's first-legal
  pick; a `wants_ui` human should be prompted for the targets (and X) of the
  cast spell. Also: no client affordance exists to suspend a card from hand.
- **One-shot spell-cost discount — ✅ DONE (primitive).**
  `Effect::GrantNextInstantOrSorceryDiscountThisTurn { amount }` pushes a
  `(amount, granted_at)` entry onto `Player.pending_is_discounts`;
  `cost_reduction_for_spell` adds it for IS spells while the player's
  `instants_or_sorceries_cast_this_turn` tally still equals `granted_at`, so it
  self-expires on the next IS cast with no consume hook. Cleared in lockstep
  with the tally each turn. A real consumer card (Thundertrap Trainer's dropped
  discount rider) has a synthesized catalog body, so the exact amount should
  be re-checked against the Scryfall cache.
- **Squad / Bargain keywords.** Squad (CR 702.157) needs "pay an
  additional cost any number of times" tracking + copy-of-self tokens (the
  `CreateTokenCopyOf` half exists). Bargain (CR 702.176) is an
  optional sacrifice-as-additional-cost (shares the unbuilt Casualty cost-mode
  primitive). Backup N (CR 702.164) is ✅ via `shortcut::backup(n, keywords)`
  (ETB +N/+N counters on target + EOT keyword grant; Conclave Sledge-Captain,
  Death-Greeter's Champion). Remaining: granting *triggered* abilities (not
  just keywords) to the backed-up creature.
- **Bot accepts beneficial Exploit/Devour.** `shortcut::exploit` /
  `devour` resolve their sacrifice via `MayDo` / `SacrificeAnyNumber`;
  `AutoDecider` and the current bot decline (the body is self-costly by
  `optional_trigger_beneficial`). A value-aware bot would accept when it
  controls a spare token/weak creature and the payoff outweighs it
  (`Decision::ChooseAmount` for devour, `OptionalTrigger` for exploit).
- **Client `Decision::ChooseCards` modal.** The new "exile any number of
  target cards" decision (`ExileAnyNumberFromGraveyards`, Devious Cover-Up)
  has wire + bot + AutoDecider support but no Bevy multi-select modal yet —
  a `wants_ui` human degrades to the AutoDecider "exile nothing". Add a
  graveyard multi-pick modal (mirrors the Discard hand-pick UI).
- **Buyback / Bestow client + bot.** `GameAction::CastSpellBuyback` (CR
  702.27) and `GameAction::CastBestow` (CR 702.103) are wired + tested and
  surfaced in `PlayerView.buyback_hand` / `bestowable_hand`. The bot now
  offers a Bestow line (enchant its sturdiest creature) in
  `main_phase_action`; **Buyback** is still bot-TODO, and the Bevy client
  still has no "pay buyback?" / "bestow on a creature?" affordance.
- **Foretell (CR 702.143) — ✅ DONE.** `CardDefinition.foretell_cost` +
  `GameAction::Foretell` (pay {2}, exile face-down, sorcery speed) +
  `GameAction::CastForetold` (cast from exile for the foretell cost on a
  later turn; gated by `GameState.foretold_this_turn`). Wired Saw It Coming,
  Doomskar, Behold the Multiverse; surfaced as `PlayerView.foretellable_hand`
  + cyan client highlight. Remaining: a client affordance to invoke Foretell /
  cast a foretold card (no Bevy modal yet), and AI never foretells.
- **"Exile any number of target cards" (graveyard hate).** ✅ Wired via
  `Effect::ExileAnyNumberFromGraveyards` + `Decision::ChooseCards`
  (AutoDecider exiles nothing; the bot exiles opponents' cards). Devious
  Cover-Up is now faithful. Remaining: extend `ChooseCards` to *battlefield*
  / hand "any number of target permanents" pickers (it's graveyard-only
  today) and surface a client multi-select modal.
- **Enduring cycle breadth.** `Effect::ReturnSelfAsEnchantment` handles the
  "return as enchantment" half (Enduring Innocence). The other Enduring
  cards (Vitality, Tenacity, Courage, Curiosity) keep distinct enchantment-
  side static abilities, which this primitive doesn't preserve/swap — extend
  it to carry the enchantment-side ability set when those cards are added.
- **Discard / exile-from-gy as real activation costs.** Psychic Frog (and
  similar) model "Discard a card:" / "Exile three cards from your graveyard:"
  as the first step of the resolved effect rather than a paid activation
  cost. Gameplay-equivalent today (nothing responds between cost and
  resolution), but a real cost (new `ActivatedAbility` fields) would gate
  activation on having the cards and let the cost be paid before the ability
  goes on the stack.
- **Ninjutsu client UI** — `GameAction::Ninjutsu` is wired + tested in the
  engine (Fallen Shinobi), but the Bevy client has no affordance to invoke
  it during the declare-blockers step (pick a ninja in hand + an unblocked
  attacker to return). Add a button/flow like Crew. The bot doesn't use
  Ninjutsu either (it would need a "swap up" heuristic).
- **Reuse `StaticEffect::PumpSelfByControlledPermanents`** — the new
  self-buff-scaled-by-controlled-permanents static (Karn's Construct token)
  also fits Master of Etherium, Tempered Steel-style self-counts, and any
  "this gets +1/+1 for each [type] you control" body currently stubbed as a
  fixed P/T. Apply opportunistically when real card data is available.
- **Client build in CI/web env** — `crabomination_client` (Bevy) fails to
  build here because `wayland-client` system libs aren't installed, so
  client-side changes can't be compiled/tested in this environment. UI
  parity is fed through the server `view.rs` projection (cost labels,
  static/triggered ability labels) which *is* testable.
- **`Decision::ChooseAmount` UI suspend** — `SacrificeAnyNumber` /
  `PayLifeLookTake` resolve the number-choice synchronously via the decider
  (AutoDecider picks 0). A `wants_ui` player should suspend on a number-picker
  modal instead of degrading to 0. Add a `ChooseAmountPending` suspend path +
  client widget (like the Learn modal).
- **Entwine as a first-class cost** — Plunge into Darkness models entwine {B}
  via `Keyword::Kicker` + a `SpellWasKicked` branch. A dedicated
  `Keyword::Entwine(cost)` (cost-line naming + "choose both") would be
  cleaner, but is functionally identical today. (CR 702.41.)
- **`SacrificeAnyNumber` reuse** — Devour and Fling-with-count can now ride
  `Effect::SacrificeAnyNumber` + `Value`-scaled payoffs.
- **Opponent-controlled pay-to-copy** — Chain Lightning's "the damaged player
  may pay {R}{R} to copy this spell." `Effect::CopySpell*` exist but are all
  controller-side; needs a copy offered to a different player.
- **Card-data audit vs Scryfall cache** (`cargo run --bin dump_cards` diffed
  against `scripts/.scryfall_cache.json`). The claude/modern_decks run fixed
  18 mana-cost bugs and 4 keyword bugs this way. **Remaining diffs are all
  legitimate** and should NOT be "fixed": X-spells store the base cost
  without `{X}` (Banefire, Earthquake, Mind Twist, Repeal, Prismatic
  Ending); free spells store an empty cost = `{0}` (Ornithopter, the Pacts,
  Zuran Orb); Adventure/MDFC fronts (Callous Sell-Sword, Cruel Somnophage);
  cost-reduction approximations (Blasphemous Act ships flat `{4}{R}` vs the
  printed `{8}{R}` minus a per-creature reduction the engine can't scale);
  colorless-pip approximations (Devourer of Destiny `{7}` for `{5}{C}{C}`);
  CDA P/T (Cosmogoyf, Lumra, Cruel Somnophage); and the custom card
  Crabomination. Re-run the audit after big card batches to catch new typos.

- **Multi-slot "up to two target" works** for explicit casts (proved by
  Read the Tides' modal bounce). Cards still collapsing it to one (Aether
  Helix's bounce, etc.) can adopt the two-slot `Move` pattern; the
  remaining gap is the *auto-target* picker only filling slot 0 for bots.

- **"May" triggers: bot now value-aware; human suspend still ⏳.**
  `AutoDecider` still declines every `Decision::OptionalTrigger`
  (`Bool(false)`), but **`RandomBot` now takes beneficial ones**
  (`optional_trigger_beneficial` — accept unless the matching `MayDo` body
  imposes a self-cost: lose life / sacrifice / discard). Tests:
  `bot_takes_beneficial_optional_trigger`,
  `bot_declines_self_costly_optional_trigger`. Remaining: a `wants_ui`
  suspend so a networked human is actually prompted (today they land on the
  AutoDecider `false` default), and revisiting `shortcut::provoke`'s
  collapse-to-mandatory now that bots can opt in.

- **AutoDecider declines all library searches** (`Decision::SearchLibrary
  → Search(None)` in `decision.rs`) — kept as-is so tests stay
  deterministic. The **bot** now overrides this: `RandomBot` handles
  `Decision::SearchLibrary` via `decide_library_search` (prefer a basic
  land toward the weakest color, else fetch the first candidate), so
  singleplayer tutors actually fix mana. Tests: `bot_search_*`. Remaining:
  a smarter non-land pick (fetch the best spell, not just the first).
- **Divided damage through a trigger fills only one slot.** Fury's evoke
  ETB (`DealDamageDivided { max_targets: 2 }`) auto-targets a single
  creature and dumps the whole total there; the multi-slot fill in
  `auto_targets_for_effect_all_slots` isn't reached from the trigger
  dispatch path. Thread the multi-slot picker through `fire_step_triggers`
  / trigger auto-target. (Single-slot auto-target through step/emblem
  triggers works — Saheeli Rai's -7 emblem copy body resolves correctly.)
- **Client kicker affordance.** `kickable_hand` (and `pitchable_hand`) now
  light up green as "playable now" via `update_castable_highlights` (unioned
  into the castable set alongside `dashable_hand`). Still wanted: a *distinct*
  "pay kicker?" badge/toggle that submits `GameAction::CastSpellKicked`
  (vs. the plain castable-green). Not compile-verified here (client can't
  build in this sandbox).
- **Provoke (targeted must-block).** `Keyword::AllMustBlock` (Lure) +
  `MustBeBlocked` (Academic Dispute) cover the untargeted 509.1c cases;
  Provoke's "that creature must block this + untap it" needs a per-blocker
  `CardInstance.must_block_attacker` link set by an attack trigger and
  cleared at end of combat.
- **Kicker — ✅ wired (CR 702.32, claude/modern_decks).**
  `GameAction::CastSpellKicked` folds the optional kicker cost into the
  spell's mana cost and stamps `CardInstance.kicked`;
  `Predicate::SpellWasKicked` reads it at resolution (via
  `EffectContext.kicked`) and `target_filter_for_slot_in_mode_kicked` makes
  cast-time target legality follow the `If(SpellWasKicked, …)` branch that
  will resolve. Tear Asunder promoted (exile artifact/enchantment, or any
  nonland permanent when kicked). Remaining: a client affordance to opt
  into the kick (a "pay kicker?" toggle on cast) and a bot heuristic to
  kick when profitable (today the bot only casts unkicked); more kicker
  cards (multikicker, kicker-with-different-effect riders).
- **Pitch affordance in client** — `pitchable_hand` cards (Force of Will /
  Spirit Guides) now light up green as "playable now" (unioned into
  `update_castable_highlights`), so a card uncastable for mana but pitchable
  still shows as playable. Still wanted: a *distinct* edge/badge separating
  pitch-castable from hard-castable. Not compile-verified here (client can't
  build in this sandbox).

- **Counter-mechanic follow-ons** (after Modular/Graft/Renown/Outlast/Melee/
  Bloodthirst this run): **Monstrosity** ✅ (`CardInstance.monstrous` +
  `Effect::Monstrosity` + `EventKind::BecameMonstrous`; Nessian Wilds Ravager,
  Ember Swallower). "As long as this is monstrous, …" statics ✅ via
  `Predicate::SourceIsMonstrous` + `StaticEffect::PumpSelfIf` (now multi-keyword
  — Fleecemane Lion gains hexproof + indestructible; Dragon's Rage Channeler's
  delirium grants flying + attacks-each-combat); **Devour** ✅ and **Amass** ✅ (`Effect::Amass` grows /
  creates a 0/0 black Army with N +1/+1 counters; `CreatureType::Army`).
  **Melee** is a
  flat +1/+1 — wants a per-combat attacked-opponent tally for multiplayer.
  **Renown** is gated on "no +1/+1 counters" as a renowned-once proxy; a real
  `renowned` flag would be more faithful for creatures that gain counters
  by other means.
- **Mulligan color-screw** — ✅ done (claude/modern_decks). `decide_mulligan`
  now unions the producible colors of the hand's lands (`land_color_output`:
  basic land types + `AddMana` payloads; "any color" → WUBRG) and only counts
  an early play whose colored pips are a subset. Test:
  `bot_mulligans_color_screwed_hands`. Remaining: dual/fetch lands that fetch
  off-color sources aren't followed transitively (a lone fetchland reads as
  colorless).
- **Client build (this env)** — `crabomination_client` can't compile here
  (`wayland-sys` build script fails: no system `wayland-client`). UI changes
  this run (keyword reminder-text additions in `counter_tooltip.rs`) are
  additive `&'static str` data and weren't compile-verified in this sandbox.
- **Divided damage** — ✅ shipped: `Effect::DealDamageDivided { total, filter,
  max_targets }` + `Decision::DivideDamage` (AutoDecider spreads evenly; UI/
  scripted deciders choose the split). Wired Forked Bolt, Pyrokinesis, Crackle
  with Power, Magma Opus, Electrolyze, Pyrotechnics, Pyromathematics,
  Lorehold Ignis/Bookburn, Arc/Forked Lightning, Chandra's Pyrohelix.
  Remaining: (a) a **client modal** so a networked human picks the split
  (today the inline decider resolves it — fine for bots/tests/AutoDecider;
  no resolution-time *suspend* path for `DivideDamage` yet), and (b)
  divided *non-damage* riders ("tap up to N", split-mill — Snow Day, Devious
  Cover-Up).
- **Network note (this run):** Scryfall (`scripts/fetch_cards.py`) returns
  HTTP 403 under the sandbox network policy, so new cards this run were limited
  to ones whose definitions are already in the repo (comments/md) or
  high-confidence staples. The Verge / Landscape / Horizon-canopy land cycles
  and other cube ⏳ entries still want Scryfall-verified definitions before
  wiring — re-run with network access.
- **Pool registration** — this run's new cards are wired into `cube.rs`
  color pools (blue: Aether Adept, Augury Owl, Cloudkin Seer, Merfolk Skydiver,
  Benthic Biomancer, Pteramander, Quandrix Cryptomancer; white: Pridemalkin;
  red: Arc/Forked Lightning, Chandra's Pyrohelix). Pridemalkin's "trample for
  countered creatures" static and the Verge/Landscape land cycles still want
  Scryfall-verified definitions.
- **`Effect::NameCard` for spells** — currently only stamps a *battlefield*
  permanent (`named_card`). Spoils of the Vault / Cabal Therapy name a card
  during *spell* resolution; that needs the chosen name captured into
  `EffectContext` (e.g. `EffectContext.named_card`) so a following Seq step
  (reveal-until-find by name, hand-discard-by-name) can read it. Pair with a
  `SelectionRequirement::HasNamedCardInContext`.
- **"Name a card"** primitive — ✅ base shipped: `Decision::NameCard`,
  `DecisionAnswer::NamedCard`, `Effect::NameCard`, `CardInstance.named_card`,
  and `activate_ability` ability-suppression for matching sources (Pithing
  Needle, Phyrexian Revoker). Remaining consumers that need the named value
  threaded into resolution: same-name exile (Crumble to Dust), reveal-until-
  find (Spoils of the Vault), hand-discard-by-name (Cabal Therapy). The
  client picker UI (free text over the catalog) is also still TODO.
- **Source-relative MV filter** — Rushed Rebirth's "lesser mana value" fetch
  (and similar) needs a `SelectionRequirement::ManaValueLessThan(Value)` that
  the `Search` resolver evaluates against a captured value.
- **Stale "two-target prompt ⏳" notes** — several catalog doc-comments still
  claim multi-target sorcery prompts are unavailable; the slot-1+ picker
  (`auto_targets_for_effect_all_slots`) is wired and the bot uses it. Sweep
  and update the remaining notes (Channeled Force done this run).

- **OrderTriggers server suspend** — CR 603.3b same-controller trigger
  ordering is wired engine-side (`order_same_controller_triggers` consults
  the decider synchronously) and the client modal exists
  (`spawn_order_triggers_modal` + `handle_trigger_reorder`). Remaining: a
  *suspend* path (park the candidate→queue dispatch in a `ResumeContext`,
  set `pending_decision = OrderTriggers`, resume on the client's answer) so
  a networked `wants_ui` human is actually prompted instead of degrading to
  the default order.

- **Tracker staleness** — CUBE_FEATURES.md / DECK_FEATURES.md carry many 🟡/⏳
  rows that are already fully implemented + tested in code (verified + promoted
  this run: Conclave Sledge-Captain, Temur Ascendancy, Trinisphere — all had
  the needed primitive wired but a stale "⏳ primitive" note). Earlier runs hit
  Opposition, Omniscience, the shock/fast/surveil/bridge/pathway land families.
  Many doc-comments still claim a primitive "doesn't exist yet" when it does
  (e.g. Stadium Tidalmage's `MayDo`, the SOS placeholder-copy cards vs
  `CreateTokenCopyOf`). A reconciliation pass would shrink both trackers.
- **Remaining 🟡 cube/deck partials are primitive- or data-blocked.** The
  cleanly-completable ones were finished this run (Cryptic Command,
  Kolaghan's Command, Master of Cruelties, Lotus Field, Coalition Relic,
  Wishclaw Talisman). What's left needs new engine primitives — split cards
  (Wear // Tear), name-a-card (Pithing Needle, Crumble to Dust), loyalty-set
  (Geyadrone), energy (Amped Raptor), divided damage / "any number of targets"
  (Pyrokinesis, the STX Outburst/Snow Day cycle), escalate (Collective
  Brutality), multi-player choice (Indulgent Tormentor) — or are synthesized
  bodies whose exact text should be re-derived from the Scryfall cache.
- **Remaining ⏳ cube cards are each blocked on a distinct new subsystem.**
  After this run's clean adds (Kestia, Brightglass, Korvold, Maelstrom Nexus,
  Conclave, Death-Greeter's, Shiko, Parallax Dementia, Mutable Explorer, Teval,
  Sab-Sunen), the rest of the missing list maps 1:1 to a sizable engine feature,
  grouped here so the next run can pick a subsystem and clear several at once:
  **dynamic/scaling equip bonus + Reconfigure + Living weapon** (Lion Sash,
  Nettlecyst, Sword of Body and Mind, Helm of the Host); **face-down permanents
  / manifest dread** (Hauntwoods Shrieker, Concealing Curtains); **Mutate**
  (Mutated Cultist + others); **ETB-control replacement** (Gather Specimens);
  **clone-many / continuous copy** (Mirrorform); **borrow activated abilities
  from graveyard/exile** (Necrotic Ooze, Agatha's Soul Cauldron); **cast-from-
  graveyard engine** (Muldrotha, The Gitrog Monster); **Saga + lore counters**
  (The Everflowing Well, Rediscover the Way); **Hideaway** (Shelldock Isle);
  **Storm cast-from-top** (Mind's Desire); **Companion** (Zirda); **DFC //
  Land** (Sink into Stupor, Unholy Annex); **phasing system** (Talon Gates);
  **all-colors / all-land-types static** (Leyline of the Guildpact);
  **tempting-offer multiplayer choice** (Tempt with Bunnies); **`LookPickToHand`
  take-N** (Consult the Star Charts); **parity attack-gate** (Sab-Sunen → ✅).
- **Flashback with an additional cost** — ✅ done this run.
  `flashback_additional_cost_for_name` (name-keyed, the `dynamic_pt_for_name`
  idiom) + `cast_flashback` validation/payment; `AdditionalCastCost::
  SacrificePermanent` gained a `count`. Lava Dart (sac a Mountain) + Dread
  Return (sac three creatures) promoted. Next flashback rider that needs it:
  pay-life / exile-from-gy flashback costs (none in the current pool).
- **Multi-target "choose two"** — `Effect::ChooseN` allocates a target slot
  per chosen mode; Cryptic Command (counter/bounce) and Kolaghan's Command
  (reanimate/any-target) now ship the faithful "choose two". Remaining:
  cast-time mode *selection* so a non-default pick routes its targets (see
  CR 700.2d below), and *divided* targeting within one mode/effect (Vibrant
  Outburst, Snow Day, Crackle with Power — split-N / divided-damage slots).
- **Dynamic P/T CDA generalization** — characteristic-defining `*/*` P/T
  (Nightmare = Swamps you control, Master of Etherium) is hand-wired per card in
  `compute_battlefield` (Tarmogoyf pattern). A `StaticEffect::SetPtFromValue`
  layer-7b primitive would let Nightmare-class cards drop in.
- **More combat keywords** — Frenzy/Afflict/Afterlife shipped this run as
  trigger shortcuts; Melee (CR 702.121, needs an "opponents attacked this
  combat" Value), Provoke, Dash, Boast remain ⏳.
- **"Becomes a copy" continuous layer-1 effects** — the one-shot copiers
  (Clone, Phantasmal Image, Mirror Image, Stunt Double, Spark Double,
  Mockingbird) ship via `Effect::BecomeCopyOf`. Mockingbird's name-retention
  exception (CR 707.2) is wired via `EntersAsCopy.keep_name`. Still open:
  continuous layer-1 "becomes a copy" effects (Helm of the Host loop,
  Mirrorform), copied enters-with-counters, and a real copy-target picker
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
  `StaticEffect::NotCreatureWhileDevotionBelow`; add them with ability text
  from the Scryfall cache.
- **Client build deps** — building the client in the web sandbox needs
  `libwayland-dev libasound2-dev libudev-dev libxkbcommon-dev` (install via
  apt). Once present `cargo build/clippy -p crabomination_client` works.

## MagicCompRules coverage audit

Periodic spot-check against the rules document (`MagicCompRules_20260417.txt`).
One line per rule: status (✅ wired · 🟡 partial · ⏳ todo) plus the still-open
gap. The full per-clause accounting (every sub-rule, code line, and test name)
was elided in a doc-compaction pass — recover it from
`git log -p -- TODO.md`. Markers are a point-in-time read; re-verify before
picking an item up.

### Done (✅) — wired, see git/code for detail
- ✅ **CR 603.3b — Same-controller trigger ordering** (server *suspend* path for a networked human still ⏳).
- ✅ **CR 702.124 — Addendum** (`shortcut::addendum` / `cast_during_your_main`: a resolution-time `IsTurnOf(You) ∧ main-phase` gate — exact since a main-phase cast resolves in the same step. Sphinx's Insight, Precognitive Perception).
- ✅ **CR 601.2f — generic cost reduction (graveyard-Affinity)** (`CardDefinition.affinity_graveyard_filter`: {1} less per matching graveyard card, generalizing the old per-name Dawning Archaic hook; clamped generic-only. The bot's `can_afford_in_state` folds in cost reductions too. Tolarian Terror, The Dawning Archaic).
- ✅ **CR 702.32 — Kicker** (client opt-in affordance + a kick-when-profitable bot heuristic still ⏳).
- ✅ **CR 702.164 — Backup** (`shortcut::backup` / `backup_with`).
- ✅ **CR 702.95 — Soulbond** (auto-pairs lowest-CardId partner; a controller "may"/decline prompt still ⏳).
- ✅ **CR 702.134 — Mentor** (`shortcut::mentor`).
- ✅ **CR 702.105 — Dethrone** (primitive only; no simple printed card exists yet).
- ✅ **CR 702.130 / 702.39 / 702.46 — Afflict / Provoke / Soulshift** (carded + tested).
- ✅ **CR 702.68 / 702.69 / 702.70 — Frenzy / Gravestorm / Poisonous**.
- ✅ **CR 702.139 — Revolt**.
- ✅ **CR 702.79 / 702.92 — Persist / Undying** (return on *any* death, not just lethal-damage SBA).
- ✅ **CR 702.66 — "Spells you cast have delve" static**.
- ✅ **CR 709 — Split Cards** + **702.102 Fuse** + **702.127 Aftermath** (~23 cards; client half-picker modal + multi-target fused halves still ⏳; 709.4/709.4b dual-name/combined-MV-in-non-stack-zones not modeled).
- ✅ **CR 510 — Combat Damage Step** (player damage-assignment order/over-assign; a `DecisionWire::AssignCombatDamage` client modal for a networked human still ⏳).
- ✅ **CR 114 — Emblems** (`Player.emblems` + `Effect::CreateEmblem`; supersedes the old ⏳ audit row — see FEATURE_ROADMAP Tier 3).
- ✅ **CR 702.114 — Devoid** — `Keyword::Devoid` CDA honored in `colors_from_card` (color base returns empty); colorless despite colored pips. Mist Intruder, Sludge Crawler, Reality Hemorrhage, Touch of the Void.
- ✅ **CR 702.115 — Ingest** — `shortcut::ingest()` combat-damage trigger + `Effect::ExileTopOfLibrary { who, amount }` (Mill routed to exile). Mist Intruder, Sludge Crawler.
- ✅ **CR 701.x — Process** (BFZ/OGW) — `Effect::Process { count, then }`: move N exile cards an opponent owns to their graveyards, run `then` only if any were processed ("if you do" rider, reading the trigger's target). Wasteland Strangler, Mind Raker, Blight Herder. Still ⏳: process-onto-battlefield (Oblivion Sower) and process-as-additional-cost (Processor Assault).
- ✅ **CR 208.2 / 613.7b — Set base P/T** — `Effect::SetBasePT` (layer-7b) now reads `Value::PowerOf/ToughnessOf(TriggerSource)` for "becomes that creature's P/T until end of turn." Eldrazi Mimic.
- ✅ **CR 702.21 — Ward (discard)** — the `WardCost::Discard(n)` payment menu is exercised by Reality Smasher (corrected from a placeholder Ward {2}).

### Partial (🟡) — remaining gap noted
- 🟡 **CR 303 — Auras** — replacement-style Aura ETB (enters attached under another rule) + bestow type-switch corners.
- 🟡 **CR 603.10 — Last-Known Information** — full LKI for mid-resolution stack sources (e.g. lifelink 702.15c). (CR 603.6d "leaves the battlefield" self-source triggers now also fire on the lethal-damage SBA path, not just the destroy/sacrifice path — Thought-Knot Seer's LTB draw.)
- 🟡 **CR 704 — State-Based Actions** — Saga SBA ✅ (`saga_chapters` reach
  final chapter → sacrifice, unless a chapter ability is still on the stack);
  Battle / Role / Dungeon / Speed SBAs remain; multi-SBA "collapse into one
  replacement" (704.7); strict spell-copy-off-stack identity (704.5e).
- 🟡 **CR 613 — Interaction of Continuous Effects** — no dependency analyzer (613.8); CDA-first pre-pass (613.3); Aura re-stamp on enchant (613.7e).
- 🟡 **CR 208 — Power/Toughness** — base-P/T-only checks (208.4b); noncreature-P/T API observability (208.3 / Vehicles).
- 🟡 **CR 119 — Life** — 119.7 set-to-lowest ✅ (`Value::LowestLifeTotal` + Repay in Kind); exchange-life-totals ✅ (Soul Conduit, Mirror Universe, Magus of the Mirror). Remaining: redistribute-life-totals; broad life-gain replacement (119.10).
- 🟡 **CR 121 — Drawing a Card** — choose-to-draw (121.3); draw-count replacement (121.2a); mid-cast face-down draw (121.8); reveal-on-draw (121.9).
- 🟡 **CR 502 — Untap Step** — Phasing (502.1); Daybound/Nightbound DFC transform (502.2). `StaticEffect::PreventUntap` honors `Selector::This` (self-referential — Basalt/Grim Monolith) and now `Selector::AttachedTo(This)` (aura-anchored "enchanted creature doesn't untap" — Claustrophobia/Dehydration).
- 🟡 **CR 509 — Declare Blockers** — cost-to-block (509.1d-f); put-onto-battlefield-blocking (509.4); "blocks two or more" batch counting (509.3e). ("Can't be blocked except by N or more creatures" ✅ via `Keyword::CantBeBlockedExceptByN` — Pathrazer of Ulamog, generalizing Menace.) Per-pair block restriction (509.1b — "target creature can't block this creature this turn") ✅ via `Effect::CantBlockSourceThisTurn` + `GameState.cant_block_pairs` (Kozilek's Pathfinder); "must be blocked if able" (509.1c) ✅ via `Keyword::MustBeBlocked` (Loathsome Catoblepas).
- 🟡 **CR 118 — Costs** — interactive mana-ability decline (118.3c); hybrid-pip per-reduction choice (118.7e); general unpayable-cost gate (118.6).
- 🟡 **CR 113 — Abilities** — emblems+CDA zones (113.6); counter-target-ability (113.9); full ability removal (113.10b); "can't have" anti-grant (113.11).
- 🟡 **CR 115 — Targets** — Aura subtype (115.1b); zero-target cast-time gate (115.6); change-target corners (115.7a-d, cross-spell exchange). Same-target rejection *within one multi-target instance* (115.3) ✅ — `Effect::distinct_target_count` + a cast-time duplicate check reject the same object filling two divide/support slots (Forked Bolt); cross-clause sharing stays legal.
- 🟡 **CR 116 — Special Actions** — morph / face-down (116.2b); Companion from outside the game (116.2g). (Foretell/Plot/Suspend special actions now ✅.)
- 🟡 **CR 105 — Colors** — type-line + color rewrite rider (105.3 second half).
- ✅ **CR 705 — Flipping a Coin** — Mana Clash two-player flip-off loop (705.2), 705.3 advantage/Krark's Thumb, win-a-flip trigger (`EventKind::WonCoinFlip`/`GameEvent::CoinFlipWon`, Chance Encounter) and lose-a-flip trigger (`EventKind::LostCoinFlip`/`GameEvent::CoinFlipLost`, emitted on the tails path of FlipCoin + ManaClash). Remaining ⏳: opponent-chooses-half flips (Karplusan Minotaur).
- 🟡 **CR 122 — Counters** — defense counters / Battle type (122.1g). Counter-clear on zone change (122.2) ✅ — `place_card_in_dest` clears `counters`/`keyword_counters` and re-seeds planeswalker base loyalty (CR 306.5b); `-0/-1` / `-1/-0` counter types ✅.
- 🟡 **CR 401 — Library** — cast-with-top-of-library-revealed recompute (401.5/401.6); multi-card same-position picker (401.4). (401.7 `LibraryPosition::FromTop` ✅.)
- 🟡 **CR 706 — Rolling a Die** — stored rolls (706.8); ignore-roll riders. Roll trigger (706.6) ✅ — `EventKind::RolledDice`/`GameEvent::DiceRolled { player, count }` fires once per roll instruction ("whenever you roll one or more dice"). Result-referencing effects ✅ via `Value::LastDieRoll` (706.4 — Ancient Copper Dragon, carded + tested). (modifier / reroll-at-most / doubles ✅.)
- 🟡 **CR 707 — Copying Objects** — in-place copy (707.4); MDFC-face copy (707.8); static copy effects (707.2c); copied "as enters" choices (707.6); spell-copy exceptions (707.9). (Enter-as-copy "except it's also [type]" ✅ via `EntersAsCopy.extra_card_types` — Phyrexian Metamorph copies any artifact/creature and stays an artifact.)
- 🟡 **CR 506 — Combat Phase** — "block as though" restrictions (506.6); combat-step cast-timing gates (506.7). Combat-damage-to-player triggers now carry the damage dealt as `event_amount` (CR 119.3), so `Value::TriggerEventAmount` riders scale by the hit (Visions of Brutality).
- 🟡 **CR 508.1g — Attack tax** — `StaticEffect::AttackTaxToController { amount }`
  taxes attackers hitting the source's controller (Ghostly Prison, Propaganda).
  `declare_attackers` sums the tax across the batch and auto-pays from the
  attacker's mana pool, rejecting the declaration if unpayable. Test
  `cr_508_1g_ghostly_prison_taxes_attackers`. Remaining ⏳: a wants_ui
  interactive pay prompt (today the attacker must have the mana floating), and
  cost-to-block (509.1d-f) is still open.
- 🟡 **CR 605 — Mana Abilities** — triggered-mana-ability fast-path (605.4a).
- ✅ **CR 701.10 — Double** — mana-doubling (701.10f) ✅ via `StaticEffect::ManaProductionDoubled` + `GameState.mana_production_doublers` (stamped around mana-ability resolution; `AddMana` multiplies pip output by `2^doublers`; rituals/spell-mana unaffected). Mana Reflection carded + tested. P/T-, counter-, life-doubling already ✅.
- ✅ **CR 701.16 — Sacrifice** — `GameEvent::CreatureSacrificed`/`PermanentSacrificed` distinct from the lethal-damage/`Destroy` die path; `EventKind::CreatureSacrificed` triggers fire only on genuine sacrifice (Mortician Beetle). Remaining ⏳: batched multi-permanent sacrifice-cost picker.
- ✅ **CR 701.60 — Suspect** — `Effect::Suspect { what }` + `CardInstance.suspected`; a suspected creature gains computed Menace + CantBlock (injected in `gather_continuous_effects`). `Predicate::SourceIsSuspected` gates Repeat Offender's toggle. Ships Barbed Servitor, Repeat Offender, Reasonable Doubt.
- ✅ **CR 701.57 — Discover N** — `Effect::Discover { n }`: exile from top until a nonland MV≤N, cast it free or put in hand (controller's choice), bottom the rest. Ships Geological Appraiser, Trumpeting Carnosaur. (Cascade-adjacent; shares the bottom-the-rest tail.)
- ✅ **CR 701.59 — Collect Evidence N** — `Effect::CollectEvidence { amount, then }`: optionally exile graveyard cards totaling MV≥N, then run the reflexive payoff. A `wants_ui` controller picks via `ChooseCards` (sum-validated); bots/tests keep the auto cheapest-pick. Ships Sample Collector, Izoni.
- 🟡 **CR 614 — Replacement Effects** — general "instead" framework; true damage *redirection* (614.9) + damage *halving*; general skip-step/turn (614.10). (ETB-counters, token/counter/damage *doubling*, regen, EtbTriggerTax, Maze-of-Ith per-source prevention ✅. Creature-ETB / death **trigger suppression** ✅ via `StaticEffect::SuppressCreatureEtbTriggers { also_dies }` — Torpor Orb / Tocatli Honor Guard / Hushbringer; `etb_trigger_multiplier` returns 0 for creature entrants and the dies-trigger gather paths skip while a suppressor is in play.)
- 🟡 **CR 615.1 — Prevention effects** — per-source / per-N shields (Wojek Apothecary, Stave Off); non-combat prevention breadth — Mending Hands ✅ (next-4 shield on any target); prevent-and-gain ✅ via `Effect::PreventNextDamageAndGainLife` + `PreventionShield.gain_life` (Reverse Damage). Remaining: source-of-your-choice restriction (the shield soaks any source's next hit).
- 🟡 **CR 500 — Turn structure** — `Predicate::CurrentStepIs(TurnStep)` gates "activate only during [your] upkeep/end step" abilities (Mirror Universe, Magus of the Mirror). Phasing / extra-step insertion still ⏳.
- 🟡 **CR 305 — Lands** — see git for the per-clause detail.
- 🟡 **CR 701.48 — Learn** — populate Lesson sideboards in the format / draft deck-build paths (engine + cube ✅).
- 🟡 **CR 702.15 — Lifelink** — LKI corner (702.15c): triggered-ability source leaving the battlefield mid-resolution.
- 🟡 **CR 701.34 — Proliferate** — see git for detail.
- 🟡 **CR 601 — Casting Spells** (logged as "CR 706 — Casting spells") — minor; see git.
- 🟡 **CR 702.29 — Cycling** — plain Cycling ✅. Typecycling/Landcycling
  (702.29e) ✅ via `Keyword::Landcycling(cost, LandType)` +
  `GameAction::Landcycle` (pay + discard → fetch a land of the named type to
  hand, shuffle; fires cycle triggers); surfaced in `KnownCard.has_landcycling`
  + a client Landcycle keybind. Ships Wirewood Guardian (Forestcycling), Daru
  Lancer (Plainscycling). Remaining: a UI search-prompt to choose *which*
  land among multiple matches (today it auto-fetches the first match).
- 🟡 **CR 117.1 — Order of priority** — APNAP corner cases; see git.
- 🟡 **CR 301 — Artifacts** — see git.
- ✅ **CR 701.8 — Destroy / 701.19 Regenerate** — `regeneration_shields` replace destruction on the SBA lethal-damage path, `Effect::Destroy`, and consume one shield (tap + remove-from-combat + heal). `DestroyNoRegen` bypasses. Toughness≤0 SBA correctly ignores shields.
- 🟡 **CR 800 — Multiplayer / leaving the game** — see git.
- 🟡 **CR 903 — Commander Variant** — MDFC back-face color identity (903.4d); 903.9 optional rider.

### Todo (⏳)
- ⏳ **CR 612 — Text-Changing Effects** — no `ReplaceWord`/text-box-swap primitive; no catalog card needs one (Mind Bend, Glamerdye, Spy Kit, Volrath's Shapeshifter, Exchange of Words).

## Suggested next-up tasks

- ⏳ **Discovered during the Eldrazi/devoid pass (not yet done):**
  - **Generalize variable-power CDA** (`*/N` from a count). Tarmogoyf, Vile
    Aggregate (`DynamicPt::ColorlessCreaturesControlled`, shipped this run),
    etc. are each a name-keyed row in `dynamic_pt_for_name`; a
    `Modification::SetPowerToughness` fed directly by a `Value` would drop the
    per-card name table entirely (e.g. Walker of the Wastes = lands named
    Wastes you control).
  - ✅ **"Defending player exiles N permanents they control"** (opponent-chosen)
    — `Effect::PlayerExilesPermanents { who, count, filter }`; the exile
    analogue of Annihilator's forced sac. Ships Bane of Bala Ged. The affected
    player auto-picks the weakest N; a human-defender chooser (a UI suspend
    like the Sacrifice path) is the remaining follow-up.
  - ✅ **Devoid-aware `Colorless` filter.** `SelectionRequirement::Colorless`
    now treats `Keyword::Devoid` as colorless (CR 702.114 CDA) at every static
    eval site (`eval.rs` ×2, `layers.rs`), so Devoid creatures with colored
    pips count for colorless-matters triggers/filters. Exercised by Flayer
    Drone (drains on a Devoid creature entering). Full color-setting effects
    (rare type/color changers) still read cost pips — a deeper follow-up.
- ⏳ **Discovered this run (modern_decks card pass), not yet done:**
  - **Rhystic "draw unless they pay X" rider** (Esper Sentinel, Mystic
    Remora) — needs a "first noncreature spell each turn" trigger + a
    pay-or-draw decision where X reads the source's power.
  - **Power-gated keyword anthems** (Temur Ascendancy "creatures with power
    4+ have haste") — `affected_from_requirement` drops `PowerAtLeast` (it's
    layer-7 computed); needs a second-pass / CR 613.8 dependency in
    `apply_layers`. Currently over-grants to all your creatures.
  - **MV-scaled token on LTB** (Skyclave Apparition's "owner creates an X/X
    Illusion where X = exiled card's MV") — needs a dynamic-P/T token mint.
  - ✅ **"with no counters on it" target filter** — added
    `SelectionRequirement::HasNoCounters`; ships Heartless Act (modal:
    destroy a no-counter creature / remove-all counters).
  - ✅ **Typecycling / Landcycling** (CR 702.29e) — `Keyword::Landcycling`
    + `GameAction::Landcycle`; ships Wirewood/Daru/Shoreline/Twisted
    Abomination/Skirk. Follow-up: a UI search-prompt to choose *which* land
    among multiple matches (today auto-fetches the first match), and
    nonbasic-type cycling (e.g. Mistcycling) once those land types exist.
  - **"Discard unless they discard an artifact" conditional discard**
    (Wrench Mind) — needs a discard whose count flexes on the discarder's
    revealed choice; today only fixed-count `Discard` ships.
  - ✅ **Fixed different-damage to N distinct targets** (Cone of Flame: 1/2/3
    to three targets) — already expressible as a `Seq` of
    `DealDamage { to: TargetFiltered { slot } }` (the Arc Trail shape extended
    to three slots). Shipped Cone of Flame; test
    `cone_of_flame_splits_one_two_three_across_three_targets`.

- 🟡 **Energy ({E}) follow-ups.** (b) **✅ "pay {E}{E} or sacrifice/bounce"
  rider** — `Effect::PayEnergyOrElse { amount, otherwise }` ships Lathnu
  Hellion (sac) and Greenbelt Rampager (bounce). (c) **✅ EnergyGained trigger
  event** — `EventKind::EnergyGained` (CR 107.16) fires "whenever you get one
  or more {E}"; Aetherborn Marauder wired. (d) **✅ damage→energy feedback** —
  Harnessed Lightning (deal 3; get {E}{E}{E} if it hit a permanent). (a)
  **✅ energy-gated mana abilities** — `ActivatedAbility.energy_cost` (CR
  107.16) gates an ability on {E}, spent up front like the mana/life
  pre-pay; Aether Hub (`{T}: Add {C}` + `{T}, Pay {E}: Add any color`) and
  Servant of the Conduit are now faithful. The affordance/bot paths gate via
  `would_accept`, so unpayable energy abilities are auto-excluded.

- ✅ **`ActivatedAbility` `..Default::default()` sweep + `remove_counter_cost`.**
  Swept the ~220 remaining full-field literals to `..Default::default()` and
  added `remove_counter_cost: Option<(CounterType, u32)>` (CR 602.5b "Remove a
  [kind] counter from this:") as a real cost paid in `activate_ability` before
  the effect goes on stack. Walking Ballista / Triskelion now pay the counter
  as a cost (can't be over-activated off the stack); test
  `walking_ballista_counter_is_a_real_cost_not_overactivatable`.

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
  The lose-life half (CR 119.8) is also ✅ — `StaticEffect::
  PlayerCannotLoseLife { target }` + `player_cannot_lose_life_now(seat)`
  drops negative deltas in `adjust_life` (covering both `Effect::LoseLife`
  and the damage path). Silverquill Lifeward (b146) ships "Your opponents
  can't lose life"; tests `cr_119_8_player_cannot_lose_life_blocks_lose_life_paths`,
  `cr_119_8_player_cannot_lose_life_blocks_burn_damage`. Remaining ⏳: (b)
  the redistribute-life-totals + exchange-life-totals clauses (CR 119.7,
  last sentence) need a check at `Effect::ExchangeLifeTotals` /
  `Effect::DistributeLifeTotals` resolve time. (c) Tainted Remedy's
  "instead, that player loses that much life" replacement is now ✅ via
  `StaticEffect::LifeGainBecomesLoss` + `life_gain_becomes_loss_now`
  (redirects positive deltas in `adjust_life`; Silverquill Reproach b209;
  test `cr_614_life_gain_becomes_loss_for_opponent`).

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

- 🟡 **Copy-token primitive** — `Effect::CreateTokenCopyOf { who, count,
  source, extra_creature_types, override_pt }` ships the token-copy half
  (Cackling Counterpart-style), and `Effect::BecomeCopyOf` ships the
  enter-as-a-copy half (Clone, Phantasmal Image, Mockingbird). Both carry
  `extra_creature_types`; the token variant also has `override_pt`.
  Remaining: a *continuous* layer-1 "becomes a copy" effect (Helm of the
  Host's per-combat haste-token loop, Mirrorform aura) — these still need a
  layer-1 copy effect rather than the one-shot definition rewrite.

- 🟡 **CR 602 — Activating Activated Abilities** (push
  claude/modern_decks — audit against `MagicCompRules_20260417.txt`).
  How the engine puts activated abilities on the stack and pays their
  costs. CR 602.1a is the costs/effect split (the colon).
  (a) **602.1a / 602.5b** — ✅ (`ActivatedAbility::mana_cost`, `tap_cost`,
  `sac_cost`, `life_cost`, `exile_self_cost`, `exile_other_filter`,
  `sac_other_filter`, `tap_other_filter`, and now `discard_cost`
  (`Option<(SelectionRequirement, u32)>` — "Discard a [filter] card:")
  between them cover the cost vocabulary; tap/mana/life/sac/discard are all
  paid in `activate_ability` before the effect goes on stack. Fauna Shaman
  rides `discard_cost`. Push claude/modern_decks: `from_hand` lets an ability
  be activated from the controller's hand — paired with `exile_self_cost` it
  models the Spirit Guides' "Exile this from your hand: Add {C}." pitch mana
  ability; tap costs are rejected from a hand source).
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
  `Effect::ChooseN { picks: Vec<u8>, modes: Vec<Effect> }`. Each
  target-bearing mode owns its own cast-time target slot, assigned in
  default-`picks` order (`target_filter_for_slot_in_mode` + the
  resolution-time `slot_of_mode` map both key off `picks`), so a
  "choose two" spell can take e.g. a spell target for one mode and a
  permanent target for another (Cryptic Command counter+bounce,
  Kolaghan's Command reanimate + any-target damage, Steal the Show,
  the five Strixhaven Commands). The auto-decider/UI run the default
  `picks`; a `ScriptedDecider` can pick any subset, but **targets only
  route correctly for mode-subsets of the default `picks`** (both the
  cast-time validation and the resolution slot map are keyed off the
  card's default `picks`, and the dense `target`+`additional_targets`
  vec can't represent a slot-1-only pick). Closing that needs cast-time
  mode selection: bump `GameAction::CastSpell.mode: Option<usize>` →
  carry the chosen ChooseN picks, validate/route slots against them
  rather than the default. Still ⏳.
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
✅ Non-DFC Sagas ship via `CardDefinition.saga_chapters` + `saga_advance`
(ETB chapter I, +1 lore each precombat main, final-chapter sacrifice SBA).
History of Benalia, The Eldest Reborn. Remaining ⏳: DFC/transforming sagas
(The Everflowing Well saga-land) and read-ahead chapter-choice variants.

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

Most prior approximations have been resolved (Windfall, Dark Confidant,
Biorhythm, Coalition Relic, Fellwar Stone, Static Prison, Rofellos, Grim
Lavamancer, Ichorid, Render Speechless — see `git log -p -- TODO.md` for the
per-card primitive + tests). Still open:

| Card / Feature | Current Approximation | Correct Behaviour |
|---|---|---|
| Spectral Procession | `{2}{W}` (most-permissive collapse of the three `{2/W}` hybrid pips onto the generic side) | Real Oracle `{(2/W)}{(2/W)}{(2/W)}`. Needs an engine-wide `ManaSymbol::HybridGeneric(u32, Color)` variant before the true hybrid cost is faithful. |

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

## Backlog — condensed session notes

The per-batch / per-push session logs (≈ batches 14–165, pushes VIII–XVII, and
the 2026-05 session notes) were append-only snapshots of "what to pick up next."
They were heavily self-redundant and many of their suggestions have since
shipped — **emblems, coin-flip / dice rolls, ETB-with-counters, manlands
(`BecomeCreature`), Ninjutsu, Learn/Lessons, Equipment/Reconfigure + living
weapon, Cascade, Storm, X-cost activated abilities, per-spell-type per-turn
tallies, and `cards_exiled_this_turn`** are all now wired (see the rules-audit
section above and FEATURE_ROADMAP). The logs were compacted in a doc-sweep; the
full per-batch text is in `git log -p -- TODO.md`.

The distinct still-open themes those logs surfaced, that aren't already captured
in the topical sections above, are:

### Engine
- **Damage-source identity tracking.** The umbrella behind several gaps:
  deathtouch and lifelink on *non-combat* damage (Fight / `DealDamage` from a
  deathtouch/lifelink creature), protection-from-color damage prevention from
  spell sources, Soul-Scar-Mage "damage as −1/−1 counters", and the
  damage-source *choice* primitive (CR 120.7 — Browbeat, Vendetta). All need
  `deal_damage_to` to carry the source's identity.
- **Ward cost variants** — `Ward—Pay N life` / `Ward—Discard` (Mica Reader,
  Tragedy Feaster) and **Ward on activated/triggered abilities** (CR 702.21a —
  tax in `activate_ability`); the bot's legal-action generator should also
  factor Ward into target affordability.
- **Counter subsystem extras** — counter-transfer-on-death (snapshot a dying
  creature's counter set → token; Ambitious Augmenter, SOS Increment payoffs);
  per-permanent `counters_added_this_turn` flag (Fractal Tender, Tester of the
  Tangential); `CounterAdded` scope filters (`AnotherOfYours`, `AnyPlayer` —
  Heliod, Vorinclex).
- **Optional-cost decisions** — `Effect::MayPay { mana_cost, body }` and
  `Effect::MayChoose { options }` (multi-option, vs. yes/no `MayDo`); plus a
  `wants_ui` suspend path so a human actually sees `MayDo` / `MayPay` prompts
  (today they default to AutoDecider's `false`).
- **Library look primitives** — `Effect::LookSplit { count, to_hand, to_bottom }`
  (Flow State, Stress Dream, Zimone's Experiment) and a `to_misses: ZoneDest`
  on `RevealUntilFind` (bottom-of-library instead of mill).
- **Cast-face / cast-zone introspection** — `Predicate::CastFace` + a
  `cast_zone: Zone` snapshot on the resolving `StackItem` (Lurrus/Yorion-style
  "if cast from a non-hand zone"; Antiquities on the Loose).
- **Multi-face MDFC** beyond two faces (`back_faces: Vec<…>` + face index).
- **`EventKind::Tapped` dispatch** — the variant exists but is never emitted;
  wire a single `tap_permanent` helper so "becomes tapped" triggers fire
  (Magda) — guard against trigger loops.
- **Multi-zone same-name exile** — `Selector::SharingNameWith` only spans the
  battlefield; Crumble to Dust needs a library/hand/graveyard-spanning variant.
- **`PlayerRef::ControllerOf` for stack items** — doesn't resolve for spells on
  the stack today (Coveted Jewel steal rider, some Swan Song-class effects).
- **Grandeur** — discard-another-card-with-this-name as an activation cost
  (`ActivatedAbility` cost-kind extension).
- **`DynamicPt::CountInZone { zone, filter, player }`** — generalize the
  Tarmogoyf-specific CDA formula (Wight of the Reliquary, Nighthowler, Master
  of Etherium).

### Content / pools
- **Deck-construction archetype weighting** — tribal subpools (Silverquill
  Inkling, Witherbloom Pest, Lorehold Spirit) and a per-school sealed-pool
  selector for `sos_mode`; the catalog has the lords/minters but the pool
  builders don't weight by archetype. Plus a cube color-pair depth audit
  (some pairs are much deeper than others).

### UI
- Ward-cost badge on permanents; ability-gate hint tooltip (surface the
  rejected `Predicate` in plain language); Prowess post-pump P/T preview;
  legendary crown/border indicator; `MayDo`/`MayPay` Yes-No prompt panel
  (gray "Yes" when unaffordable); life-cost portion colored on hybrid
  mana+life costs; MDFC back-face cost in the cast-button tooltip when flipped.

### Server
- Trigger-filter debug logging (`TriggerFiltered { source, kind, scope, reason }`);
  snapshot round-trip tests for new `#[serde(default)]` fields; a
  mana-paid-for-optional audit event; per-cast-face metrics; Ward factored into
  the bot's legal-action generation.
