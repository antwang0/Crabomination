# Card / set backlog (archive)

Moved verbatim out of `TODO.md` when that file passed the ~1k-line size
trigger. Per-set closure notes and the residual approximations each closed
set still carries, plus the remaining gap lists. Card work is the lowest
priority in the current ML phase; this is where it waits.

## Noticed this run (Mirage wave 5)

- **Graveyard targets are dropped on the activated/triggered ability path.**
  A *cast* spell can target a graveyard card (Fates Reversal's test does it
  through `GameAction::CastSpell`), but the same `Target::Permanent(<gy card>)`
  handed to `GameAction::ActivateAbility` never reaches the resolution — Hakim,
  Loreweaver's Aura-recursion and Iridescent Drake's ETB both have to be tested
  by resolving the effect directly. `Effect::AttachAuraFromGraveyardTo` is now
  surfaced in `primary_target_filter` / `prefers_graveyard_target`, which was
  one half of it; the remaining half is in the action layer's target
  validation. ⏳
- **`MayPay` can't reach lands, so upkeep "pay {4}" riders need floated mana.**
  Purgatory's rent is unpayable unless the controller happens to have mana in
  the pool when the trigger resolves. The comment calls this deliberate ("mana
  abilities aren't activatable mid-resolve by default"), but for a UI seat the
  right behaviour is a real payment window. ⏳
- ✅ **Shadowbane** ships: `PreventNextFromChosenSourceToTeam` grew a
  `gain_life_colors` rider and `PreventionShield` a `gain_life_color` /
  `gain_life_to` pair. `PreventionShield` is `Copy`, so its gate is a single
  colour where `PreventedSource`'s is a `Vec` — the two prevention families
  still want unifying.

## Noticed this run (Mirage opened)

- **`legal_block_targets` recomputes `blocker_can_block_attacker` per pair.**
  It shares one frozen-layer gather, but the per-pair walk is still O(blockers
  x attackers) with a full requirement evaluation inside. Fine at real board
  sizes; worth a cheap keyword prefilter if a wide board ever shows up in a
  profile. ⏳
- **`Selector::TriggerSource` on a block trigger binds the source, not the
  partner.** It bit three cards this run (Dream Fighter, Crimson Roc,
  Catacomb Dragon) before they were rewritten onto
  `Selector::CreaturesInCombatWith(This)`; `combat_partner_punisher` already
  worked around it. Either bind the partner there or rename the selector —
  every `BecomesBlocked` / `Blocks` body that reads `TriggerSource` is
  suspect and wants an audit. ⏳
- **The parallel target-walker class is now ratcheted, not closed.**
  `Effect::MoveCounters` was in `requires_target` but neither
  `primary_target_filter` nor `target_filter_for_slot`, so Afiya Grove's
  trigger silently did nothing; `AttachAuraFromGraveyardTo` was the same.
  `core_rules/target_walkers` now serde-walks the whole catalog and fails if
  more than 39 effect bodies declare a `TargetFiltered` slot the walker can't
  answer — down from 164 after arms were added for LicidAttach (10 Licids),
  PreventNextDamageFromChosenSource's `to`/`redirect_to`, AttachSourceTo,
  RemoveFromCombat, SpellBecomesChosenColor, UnlockRoomDoor, HauntCreature,
  RevealRandomFromHand, TopTwoGraveyardOpponentSplits, CoffinExile,
  ExchangeOwnership, PlayerMayPayLifeElse and ReplaceYourNextDrawThisTurn.
  The residual 39 are a long tail (run the test to list them). A single
  derive-or-table over the three walkers would end the class outright. ⏳
- **`Selector::MatchingAmong` statics only resolve over `Selector::This`.**
  `eager_static_targets` handles the self-scoped shape (the one every real
  card uses); an inner `EachPermanent` still falls through to `None` and the
  static contributes nothing. ⏳
- **Mirage residue** — the last 17 cards, each blocked on one primitive:
  Celestial Dawn (a global colour/land-type rewrite), Forbidden Crypt (a draw
  replacement that reaches the graveyard), Bazaar of Wonders (name-matching
  counterspell static), Null
  Chamber (a *two-name* lock — `named_card` holds one), Tombstone Stairwell
  (token bookkeeping across both halves), Energy Vortex / Soul Echo
  (counter-priced upkeep taxes), Grinning Totem and Mangara's Tome (exile
  piles you may play from), Meddle (retarget-another), Illicit Auction (a
  life-bidding subgame), Mangara's Equity (a damage trigger filtered on both
  the dealer's colour and the recipient), Mindbender Spores (counters that
  carry granted abilities), Acidic Dagger and Barreling Attack (both want a
  `DelayedTriggerKind::BecomesBlocked` / "deals combat damage this turn"),
  Cycle of Life (a "creature you cast this turn" filter), and Phyrexian Purge
  (`CardDefinition.life_per_target`, the life twin of
  `cost_per_extra_target`). ⏳
- **Suq'Ata Firewalker uses `HexproofFromColor`.** The printed line is a
  shroud-from-red (its controller's red spells can't target it either);
  `HexproofFromColor` only stops opponents. ⏳

## Noticed last run (CR follow-ups)

- **`Effect::MustBlockSource.chooser` asks even when the pick is forced.**
  The seat is only consulted when more than one candidate matches, but a
  `wants_ui` seat with two identical creatures still gets a modal for a choice
  that changes nothing. Cheap to skip when every candidate is equivalent. ⏳
- **CR 800.4b's third clause is unmodelled.** "If an object would be put onto
  the battlefield or onto the stack under the control of a player who has left
  the game, that object remains in its current zone" — only the token and
  control-change halves are gated. ⏳
- **CR 800.4a's exile clause is unmodelled, and blocked on a missing marker.**
  Objects a departed player still controls once their own objects have left
  and control-changing effects have ended (a Bribery'd creature) should be
  exiled; today they revert to their owner. The blocker is that a
  *permanent*-duration steal (Mind Control, Gruul Charm) registers nothing —
  `Effect::GainControl` only pushes a `TempControl` entry for non-permanent
  durations — so the engine can't tell "a control effect that is now ending"
  from "no effect at all". Wants a `CardInstance.control_effect_from:
  Option<usize>` stamped by every control change, which would also give
  CR 800.4a's revert step a precise rule instead of "revert everything". ⏳

## Noticed this run (Visions closed)

- **Two Coldsnap cards want one primitive each.** Brooding Saurian ("at the
  beginning of each end step, each player gains control of all nontoken
  permanents they own") needs a bulk control-revert effect — the same shape
  CR 800.4a's revert step wants. Goblin Furrier ("prevent all damage this
  creature would deal to snow creatures") needs `StaticEffect::
  PreventThisDamageToColor` generalized to a `SelectionRequirement`. ⏳

- **`StaticEffect::CostReduction` has no scope knob.** It is controller-only,
  so Helm of Awakening needed a whole sibling variant
  (`AllPlayersSpellsCostLess`). The `AnthemFor*` family has the same problem
  (TODO below) — worth one shared `Scope { You, Opponents, AllPlayers }` enum
  across the reduction, tax and anthem families. ⏳
- ✅ **Ward-cost filters are evaluated one way.** Every filtered `WardCost` arm
  (`DiscardMatching`, `SacrificeMatching`, `ExileTopFromGraveyardMatching`,
  `ReturnMatchingFromGraveyardToHand`, alongside the two that already did) now
  goes through `evaluate_requirement_static` with `ctx.source`, so
  `OtherThanSource` / `IsSource` read right in all of them.
- ✅ **A floating attack pump can freeze its amount.** `Effect::
  PumpAttackersThisTurn` evaluates its `power`/`toughness` at resolution and
  installs a `turn_granted_triggers` watcher carrying `Value::Const`, so Song
  of Blood's "for each creature card milled this way" survives to combat. The
  general "freeze this Value now" wrapper is still worth having — every other
  delayed body still re-reads its `Value` at fire time. ⏳
- ✅ **`primary_target_filter` now walks the "unless you pay" fallback arms.**
  `PayManaOrElse` / `PayEnergyOrElse` / `PayEnergyOrElseValue` were surfaced by
  `requires_target` and `target_filter_for_slot` but not by
  `primary_target_filter`, so a *triggered* ability whose only target lives in
  the fallback (Knight of the Mists' "destroy target Knight") never had one
  bound and silently did nothing. Another instance of the parallel-walker class
  in the P3 audit section.
- **`AffectedPermanents::AllOpponents` still drops some filter leaves.** Colour,
  creature type and counter are decomposed now (Heat Wave forced it), but the
  `ControlledByOpponent` branch in `affected_from_requirement` still discards
  every *other* leaf it doesn't have a field for — a `Not(...)`, a power gate,
  a token flag. The `CardMatch` fallback handles those on the non-opponent
  path; the opponent path should route through it too. ⏳
- ✅ **Per-source damage marking.** `CardInstance.damage_by_source_this_turn`
  tallies every source's contribution alongside the total, and the lethal-damage
  SBA reads it for `Keyword::SurvivesSplitLethalDamage` (Ogre Enforcer). CR
  120.6's per-source questions now have a home too.
- ✅ **Visions closes at zero.** The last six shipped with one primitive each:
  `StaticEffect::DrawsRevealedTaxed` (Breathstealer's Crypt),
  `Effect::MayRepeat` (Forbidden Ritual — the costless sibling of
  `MayPayRepeatedly`), `Keyword::SurvivesSplitLethalDamage` (Ogre Enforcer),
  `Effect::TruceThisTurnAndNext` + `GameState.truce_until_turn` (Peace Talks),
  `Effect::DrainDefendersLandsForManaNextMain` (Pygmy Hippo) and
  `Effect::PumpAttackersThisTurn` (Song of Blood).
- ✅ **CR 514.2 reaches the phased-out zone.** Cleanup was clearing marked
  damage and "until end of turn" effects only for permanents on the
  battlefield; a phased-out permanent kept both across the turn boundary. Both
  loops now chain `phased_out`. Regression:
  `cr_514_2_cleanup_clears_phased_out_damage`.
- **Peace Talks' truce is coarse.** `truce_active()` short-circuits the
  permanent- and player-target checks and rejects every attack declaration; it
  does not distinguish "targets of spells and activated abilities" from
  *triggered* ability targets, which the printed card leaves legal. Worth a
  `TargetSource` argument on the legality check. ⏳
- **`Effect::MayRepeat` caps at a card-supplied `max`.** Forbidden Ritual's
  "any number of times" is 8 in practice. A genuinely unbounded loop needs a
  mandatory-loop guard like the one `mandatory_loop_watch` already runs. ⏳

## Noticed last run (Weatherlight closed)

- **`Effect::AtEndOfCombat` carries one target, and now one subject.** It
  captures `ctx.targets.first()` and `ctx.trigger_source`; a body that needs a
  *second* target slot at end of combat still loses it. `AtNextEndStep` has a
  slot-remapping hack for exactly this — worth generalizing both onto a shared
  captured-context struct. ⏳
- **Doomsday's five picks aren't ordered.** `Effect::Doomsday` takes the
  chosen five and stacks them in pick order; the printed card lets you order
  them freely on top. Wants an ordered `Decision::OrderCards` (which
  `Decision::Scry` half-implements). ⏳
- **`Effect::CoinFlipDoubleOrPreventNextDamage` auto-picks the source.** The
  printed Desperate Gambit lets you choose any source you control; the
  resolver takes the highest-power permanent. Needs a real chosen-source
  prompt (the `PreventNextDamageFromChosenSource` machinery has the shape). ⏳
- **Lotus Vale / Scorched Ruins enter, then pay.** The printed replacement is
  "if this land would enter, sacrifice two untapped lands *instead*"; the
  catalog models it as an ETB pay-or-sacrifice, so the land is briefly on the
  battlefield and an ETB watcher sees it. Wants `CardDefinition
  .enters_only_if_cost_paid`. ⏳
- **`Effect::TapLandsSharingProductionWith` reads printed mana abilities.**
  `GameState::colors_produced_by` walks `definition.activated_abilities`, so a
  land whose mana ability is *granted* (Dryad Arbor-style statics) isn't seen
  by Mana Web. Computed abilities would close it. ⏳
- **Board-wide `CreatureDied` listeners need a real kill in tests.** Neither
  `destroy_permanent(id, false, &mut events)` nor
  `remove_to_graveyard_with_triggers` dispatches to `EventScope::AnyPlayer`
  listeners on *other* permanents — only a death that runs through the action
  path (SBA after a bolt) does. Worth making the fixture helpers dispatch the
  same way so tests don't have to route through combat/burn. ⏳
- **CR 123 (Stickers) is the last untested CR section.** Nothing about
  stickers is modelled — name/ability/P-T/art stickers, ticket costs, the
  sticker-sheet setup in CR 103. Only worth doing alongside Unfinity cards. ⏳
- **`GameState::check_state_based_actions` isn't reached by a bare
  `PassPriority` with an empty stack.** Tests that arm a CR 603.8 state
  trigger have to call it by hand (`classic_sets/wth`'s `settle`). Worth
  confirming whether that matches CR 704.3 or is a fixture-only gap. ⏳

## Also this run (Exodus closed)

- **Kor Chant's chosen source collapses to "the next damage event."**
  `Effect::RedirectNextDamageTo` is a one-shot per-permanent redirect with no
  source restriction, so a different source can consume the redirect. A
  `RedirectAllDamageFromChosenSourceThisTurn` (the redirect twin of
  `PreventNextDamageFromChosenSource`) would close it. ⏳
- **`Effect::OathCatchUp` auto-picks the biggest lead in multiplayer.** The
  printed wording is "that player chooses target player who …", a real target
  choice by the upkeep player. Exact in 1v1. ⏳
- **Volrath's Dungeon's buy-off is an `any_player` activation** gated on
  `IsTurnOf(You)`, which reads the *activating* seat, so the "only during
  their turn" clause is right but a third player in multiplayer can also buy
  the pass. Same shape as Volrath's Curse (below). ⏳
- **Ertai's Meddling returns a free cast, not a copy.** The printed line puts
  the exiled card back "as a copy of the original spell", keeping its targets
  and modes; `process_delayed_spells` re-casts it from exile with an
  auto-picked target instead. Wants the delayed trigger to carry the original
  cast's target/mode/X stamp. ⏳

## Noticed last run (Tempest down to 3)

- **CR 123 (Stickers) is the last untested CR section** and the only one the
  engine doesn't model at all — no sticker sheets, no `{TK}` tickets, no
  name/ability/P-T/art stickers. `CR_COVERAGE.md` is otherwise at 145/146.
  Scoping note: name stickers are a text-changing effect (CR 613.1c) and P/T
  stickers are layer 7b, so both could ride the existing layer machinery; the
  sheet/ticket economy in CR 123.2–123.3 is the bulk of the work. ⏳
- **Tempest's last three cards: DONE.** Duplicity, Ertai's Meddling and
  Oracle en-Vec all ship; `set_gaps.py tmp` is zero. Ertai's return is a free
  cast from exile with an auto-picked target rather than a literal copy of the
  original spell (targets/modes are re-chosen); Oracle en-Vec's mandate is
  enforced at `declare_attackers` and swept at the end step. ✅
- **Booby Trap's reveal clause is board-wide.** The printed line is "the
  chosen player reveals each card they draw";
  `StaticEffect::OpponentsPlayWithHandsRevealed` reveals every opponent's
  whole hand. Exact in 1v1 for the *trigger*, loose for the reveal. ⏳
- **`PlayerRef::EachOpponent` stands in for "choose an opponent"** on Pallimud
  and Booby Trap — exact in 1v1, first-opponent in multiplayer. A real
  `Effect::ChooseOpponentForSource` (a `ChooseOption` over the opponent seats,
  stamped on `chosen_player`) would close both. ⏳
- **Volrath's Curse's shrug-off is an `any_player` activation.** The printed
  clause is a special action for the *host's* controller; modeled as an
  activated ability anyone may pay, so a third player could also buy the pass
  in multiplayer. The `statics_ignored_this_turn` check now covers equipped
  bonuses, not just the Damping-Engine cost tax. ⏳
- **Client cloud type-check: DONE.** `crabomination_client` now spells Bevy's
  default feature set out explicitly and gates the three pkg-config-backed
  bits behind default-on features, so `cargo check -p crabomination_client
  --no-default-features` works in the cloud. Run it before finishing any
  client edit — it caught two `CounterType::Magnet` match breaks the moment it
  landed. It still does NOT link or run the client, so visual changes remain
  eyeball-only from a cloud session. ✅
- **`PowerAtMostSourceCounters` is source-only.** The `evaluate_requirement_on_card`
  path has no source in scope, so it answers `false` there; only the
  `evaluate_requirement_static` path (targeting, the one that matters) reads
  the counters. Same shape as `ManaValueEqualsCountersOnSource`. ⏳
- **Precognition scries the opponent's library.** The printed card gives *you*
  the look-and-bottom decision; `Effect::Scry { who: Target(0) }` hands it to
  the library's owner. Wants a "look at target player's top card, you decide"
  effect. ⏳

## Noticed last run (Stronghold closed)

- **`copies_top_graveyard_creature` re-syncs by name.** The SBA pass skips the
  swap when the current definition's name already equals the target's, so two
  same-named creature cards on top of the graveyard are indistinguishable.
  Harmless in practice, wrong in principle. ⏳
- **Exodus is opened, not closed** (`set_gaps.py exo` = 69). The Keeper cycle
  shipped; the **Oath cycle** still wants the same comparison on an "at the
  beginning of each player's upkeep, THAT player chooses target player …"
  trigger, which needs the trigger to run once per seat with that seat as the
  controller. What else is left clusters on: coin-flip combat effects
  (Fighting Chance, Mogg Assassin), Cataclysm / Limited Resources / Fade Away
  (per-player keep-N sacrifices), and Mind Over Matter / Null Brooch /
  Reconnaissance. ⏳
- **`dispatch_triggers_for_events(&[PermanentEntered { .. }])` does not fire an
  ETB trigger** for a permanent put down with `add_card_to_battlefield`, so
  tests for ETB cards must resolve `def.triggered_abilities[0].effect`
  directly. Worth finding out which precondition the synthetic event misses —
  the idiom reads like it should work and is used in `akh`/`bfz`. ⏳
- **The repo is not `cargo fmt` clean.** Running `cargo fmt --all` rewrites
  ~700 files. Format new code by hand; don't run it across the workspace. ⏳
- **`StaticEffect::PreventUntapGlobal`'s untap-preview path is quadratic.**
  When any global prevent is live, `do_untap`'s preview re-walks
  `untap_prevented_by_static` for every battlefield permanent instead of
  intersecting filter sets the way `PreventUntap` does. Fine at real board
  sizes; worth folding together if it ever shows up. ⏳
- **Heartstone reduces only *your* creatures' activated abilities.** The
  printed line is global; `StaticEffect::YourCreatureActivatedAbilitiesCostLess`
  is the closest existing static. ⏳

## Noticed this run (MKM closed)

- **Kaya, Spirits' Justice picks the first exiled creature card, not a
  chosen one.** The exile trigger fires once per batch (the event kind isn't
  in the fan-out list) and `Selector::TriggerSource` binds the first matching
  card; the printed "choose a creature card from among them" would want the
  whole batch published to the effect. ⏳
- **Conspiracy Unraveler's evidence alt-cost is hand-only.** The printed line
  covers spells you cast from anywhere;
  `StaticEffect::CastHandSpellsForCollectEvidence` is read only on the
  own-hand branch of `cast_from_zone_without_paying`. ⏳
- **A card's name is worth greping before writing it.** Magnifying Glass and
  Thinking Cap were already in `decks::recent247`/`recent245`; `set_gaps.py`
  correctly omitted them, but a duplicate got written anyway and glob
  re-export silently shadowed it. Check `set_gaps.py` output, not the set
  list. ⏳
- **`Effect::CopyCardAndCastFree` routes the copy through hand.** It mints the
  copy as a token in the caster's hand and free-casts it from there, so a
  "cast from graveyard/exile" watcher doesn't see the real zone. ⏳
- **`Effect::DeployExiledCreature` always takes the "may".** The printed line
  is optional; free upside, so the engine deploys whenever a creature card is
  among the exiles. ⏳
- **`Effect::GrantKeywordsToSpell` only reaches lifelink and deathtouch.**
  The grant is recorded per stack item and read where `resolving_spell_*_seat`
  is stamped; other keywords on a spell would need their own read sites. ⏳
- **Coveted Falcon's unmask hands permanents to `EachOpponent`**, not to a
  chosen target opponent — the printed line pairs one player slot with any
  number of permanent slots, which the multi-kind target machinery can express
  but `ApplyToTargets` can't thread through. ⏳
- **Aurelia's Vindicator caps its exile at five slots.** `TargetsExactlyX`
  truncates to the paid X, but the static slot ceiling is a constant; a very
  large X under-delivers. ⏳
- **Doppelgang caps at six target permanents** for the same reason. ⏳
- **`AnthemForFilter` still spells out eight fields at every call site**
  (~hundreds). `LookPickToHand` got the boxing treatment; this one
  is the remaining offender. ⏳
- **`Effect::SacrificeAtNextEndStep` resolves its selector eagerly.** That is
  right for Pull (the reanimated permanents are known then) but a caller
  wanting "sacrifice whatever matches at end of step" would need a filter
  variant. ⏳
- **Tenth District Hero's Mileva anthem is gated on a Level counter**, not on
  the printed "if this creature is named Mileva" — a name-change layer would
  be the faithful wiring. ⏳
- **Jetsam's free cast is capped at one per opponent by count**, not by
  "one from each opponent's graveyard": in multiplayer the caps are the same
  number but could all come from one graveyard. ⏳

## Noticed this run (CN2 closed; MKM opened)

- **A triggered ability's declared target isn't scriptable from a test.** A
  `ScriptedDecider` `Target` answer doesn't reach the trigger-target picker, so
  Unyielding Gatekeeper's regression drives the effect directly with a stamped
  `ctx.targets` instead of going through `TurnFaceUp`. ⏳
- **Turning face up for {X} has no client prompt.** The new "Turn face up
  {cost}" menu entry submits `TurnFaceUp` (X = 0); Aurelia's Vindicator and
  Warbreak Trumpeter need a `ChooseAmount` before `TurnFaceUpForX`. ⏳
- **CR 401.4 is unimplemented.** When an effect puts two or more cards in the
  same library position at once, their owner should get to order them; the
  engine places them one at a time in iteration order. ⏳
- **No per-card graveyard-arrival stamp.** "Put there from anywhere this
  turn" filters (Reenact the Crime) have nothing to read; a `graveyard_turn`
  on `CardInstance`, stamped in `Player::send_to_graveyard`, would close it. ⏳
- **Spire Phantasm's draft-time guess is a heuristic.** The pod notes a hit
  only when the pack it names from has one card left; a real guess wants the
  drafter to name a card and the next drafter to reveal. ⏳
- **Regicide's color restriction is a resolution-time gate, not a targeting
  one.** Cast-time target validation looks the source up by id, and a spell
  mid-cast is in neither hand nor stack, so `HasDraftNotedColorOfSource` can't
  see its own name there. A `source_name` on the validation path would close
  it. ⏳
- **Canal Courier's "attack different players" unblockable clause is
  dropped** — a multiplayer-shaped restriction with no engine hook yet. ⏳
- **Sovereign's Realm's "play basic lands from outside the game" is a fetch.**
  `Effect::BasicLandFromOutsideGameToHand` puts one basic of a chosen color in
  hand; the printed line is a turn-scoped *play* permission, so the fetched
  land survives the turn and can be played later. ⏳
- **Spy Kit's name grant bypasses the layer system.** `has_all_creature_names`
  scans the battlefield for the Equipment instead of emitting a layer-3
  text-changing effect (CR 613.1c), because a computed lookup inside an anthem
  filter recurses. Same reason `HasDraftNotedCreatureTypeOfSource` reads
  *printed* creature types. ⏳
- **`Effect::AddCountersUpTo` asks once for the whole selector.** "Put up to N
  counters on each" would want a per-target ask; no printed card needs it yet,
  and non-UI seats take the maximum (it's pure upside). ⏳
- **Spectral Grasp's block half is a flat `CantBlock`** — exact at two players,
  too wide in multiplayer, where it should only stop blocks against the Aura
  controller's creatures. Wants a filtered can't-block static. ⏳
- **`Effect::ExileSelfReturnTransformed` on the stack** (a resolving spell that
  puts itself onto the battlefield transformed) now ships as its own routing
  claim, `PutResolvingSpellOnBattlefieldTransformed`, rather than by teaching
  the effect to reach the stack. A future card wanting the same shape mid-
  resolution (not at end of resolution) would still need the reach. ⏳
- **Shinryu's "when the chosen player loses the game, you win" is dropped.**
  There's no player-loses event to hang it on, and the clause is redundant in a
  two-player game. The choose-an-opponent half ships via
  `Effect::RememberPlayerOnSource`. ⏳
- **The Attraction junkyard renders in the exile browser** (`V`), as a per-owner
  section below the exile piles. The pile tooltips still don't mention it. ⏳
- **The bot now offers graveyard "return transformed" activations**, but has no
  evaluation of *whether* the back face is worth the mana — the gate is purely
  "can this replay itself". ⏳

## Noticed this run (EOE/FIN closure + CR 717 Attractions)

- **Joshua's ETB loots for exactly two.** The printed "discard up to two cards,
  then draw that many" has no up-to variant; the catalog ships the mandatory
  two-for-two. ⏳
- **Attraction lights are one canonical set per card.** CR 717.1 says the same
  Attraction can print different light combinations; the catalog ships the
  Scryfall-canonical set, so two copies of one Attraction are identical. ⏳
- **`Effect::OpenAnAttraction` doesn't shuffle at game start.** CR 717.2 wants
  the Attraction deck shuffled with the library; `seat_attraction_deck` keeps
  the given order (deterministic for tests). ⏳
- **The dexterity / sticker Attractions are unimplemented** — Cover the Spot,
  Dart Throw, Guess Your Fate, Scavenger Hunt, Squirrel Stack, The
  Superlatorium, Trivia Contest need CR 123 stickers and out-of-game physical
  actions. CR 123 is the last big untested rules section. ⏳
- **`MayPlayPermission` has no surcharge field.** Lightstall Inquisitor stamps
  `granted_alt_cast_cost_eot = printed cost + {1}`, which is exact for a fixed
  cost but stamps X = 0 on an `{X}` card. ⏳
- **"Each land played this way enters tapped"** (Lightstall Inquisitor) rides
  the engine-wide may-play land gap and is dropped. ⏳

## Noticed this run (one-primitive backlog; BLB/DSK/OTJ closed)

- **`Effect::RevealTopExileOnePerCardType` asks once, not per type.** Portent
  of Calamity's "for each card type, you may exile a card of that type" is one
  `ChooseCards` prompt capped at the distinct-type count, with the per-type cap
  enforced by claiming a fresh type per pick. The printed wording is a sequence
  of per-type choices; a card that cares about the difference would need the
  cursor-driven multi-ask. ⏳
- **Its free cast auto-picks the priciest exiled nonland.** "You may cast a
  spell from among the exiled cards" isn't offered as a choice of which. ⏳
- **`Effect::EachPlayerDoes` bodies fall to `AutoDecider` for opponents.**
  Rottenmouth Viper's "unless that player sacrifices … or discards" therefore
  reads as "the opponent always pays the 4 life" for non-UI seats — the
  conservative default, but not a real evaluation. ⏳
- **`PlayExiledWithSourceForLife` covers casts, not land plays.** Valgavoth's
  "you may play cards exiled with this" shares the engine-wide may-play land
  gap; an exiled land in his pile can't be played. ⏳
- **Nothing offers Valgavoth's exile plays to a bot.** The affordance and bot
  candidate generators don't walk `exiled_with` grants, so only a UI seat (via
  the new `ExileCardView.play_for_life` badge) can use the pile. ⏳
- **`Predicate::CastSpellFirstMatchingThisTurn` resolves cast ids through
  `find_card_anywhere`.** A cast card that has ceased to exist (a copy) simply
  doesn't count, which is right today but would need a stored snapshot if a
  card ever asks about copies. ⏳

## Noticed this run (DSK/BLB gap wave — Rooms, legends, Ral)

- **Convoke reaches activated abilities** — `ActivatedAbility.convoke` rides
  the waterbend helper slot on `GameAction::ActivateAbilityWaterbend`, and a
  helper pays a coloured pip of its own colour (CR 702.51b) or generic
  (Heirloom Epic). ✅ Residual: the helper→pip assignment is greedy rather than
  a real player choice; no printed card can tell the difference today. ⏳
- **BLB / DSK / OTJ are closed** (`set_gaps.py blb dsk otj` is empty). The
  thirteen one-primitive cards all shipped in `decks::recent329`. ✅
- **`Effect::LockOrUnlockRoomDoor` picks for the controller.** Marina Vendrell's
  "lock or unlock" opens a locked door when there is one and otherwise re-locks
  the right door; a `wants_ui` seat should be asked which. ⏳
- **`StaticEffect::FreeExileCastOncePerTurn` fires automatically.** Warped Space
  waives the first exile cast's stamped cost with no prompt — strictly better
  for the caster, but the printed "you may" is a choice. ⏳
- **The Rat token's swarm bonus is a static, not the printed reminder.** Vren's
  tokens carry `PumpSelfByControlledPermanents` rather than a copiable ability
  string; a token-copy effect reproduces the bonus, which matches CR 707.2. ⏳
- **`install-client-deps.sh` still doesn't fire in scheduled sessions** —
  confirmed again 2026-08-05: `-p crabomination_client` died in
  `wayland-sys`'s build script until the hook was run by hand, after which it
  builds clean. The install belongs in the environment image. ⏳
- **Clippy is minutes, not hours, on a warm `target/`.** Corrected 2026-08-05:
  `CARGO_INCREMENTAL=0 cargo clippy --all-targets` over the five non-client
  crates finished in **4m31s** after a normal `cargo build` had already warmed
  the tree, and the tree stayed at 7 GB. The old ~10 h figure was a cold
  incremental run that also filled the disk. Still run the two scopes
  separately — the non-client crates, then `-p crabomination_client` (Bevy
  from scratch is the expensive half — ~25 min of dependency compiles on
  2026-08-05, so start that scope early). Both were clean that run. The client
  run caught
  **five compile errors in client test code that no other gate saw**: a
  `PermanentView` literal missing a newly-added field and four
  `board_status_strip` call sites missing a newly-added argument. A
  `cargo build --workspace --all-targets` run started *before* those fields
  existed had reported success, which is what hid them — re-run the build
  after the last edit, not just once mid-session. Note also that
  `cargo test -p crabomination_client` needs its own non-clippy Bevy build on
  top of all that (~3 h more). It does finish — the 9 board-status unit tests
  ran green this run — but only if it's started early. ⏳

## Noticed this run (TDM + OTJ closed, BLB/DSK batch)

- **The Aura's identity is reachable from an attach trigger** —
  `EventKind::AuraAttachedToAny` binds the *Aura* as the subject with no host
  gate, so a body can read both objects (Eriette, the Beguiler). ✅
- **Cards deferred from the BLB/DSK sweep.** Vren, Ygra, Marvin, The
  Mindskinner and Eluge shipped in the DSK/BLB gap wave; Portent of Calamity,
  Wishing Well and Dragonhawk are still open (see that run's section above). ⏳
- **`Selector::CardExiledWithSource` now spans both linkage styles.** It reads
  the plain `exiled_with` stamp *and* the CR 603.6e `exiled_by` return link. If
  a card ever wants only one of the two, the selector needs splitting. ⏳
- **`MayCastPermanentFromHandFree`'s bot policy is "take the most expensive".**
  Kellan, the Kid's free cast auto-picks the highest mana value in hand, which
  is right for a bot but ignores board state. ⏳
- **`Effect::CounterOnMatchingOfEachColor` spreads greedily.** Call the Spirit
  Dragons' auto-pick prefers a Dragon that hasn't taken a counter yet, which
  maximises the five-recipient win check but isn't a real evaluation. ⏳

## Noticed this run (DFT closed to 2, TDM to 6 / CR 400.7)

- **`install-client-deps.sh` doesn't fire in scheduled sessions.** Confirmed
  again this run: the hook is present and correct (running it by hand
  installs libwayland/alsa/libudev and the client then builds clean), but the
  container had no `wayland-client.pc` at the first build. Scheduled/cron
  sessions evidently skip SessionStart hooks — the install belongs in the
  environment image, or the build should shell out to the script itself. ⏳
- **`target/` fills the container's 252 GB disk.** A full
  `cargo build --workspace --all-targets` plus a `cargo clippy` run reached
  29 GB and hit ENOSPC mid-run; deleting `target/debug/incremental` freed
  16 GB. Worth setting `CARGO_INCREMENTAL=0` for CI-shaped runs. ⏳
- **`granted_abilities_for` now serves off-battlefield cards.** Instance grants
  on a graveyard/hand/exile card are surfaced so Cursecloth Wrappings' embalm
  activates. The *static*-granted lists (`GrantActivatedAbilityFromGraveyard`,
  Necrotic Ooze) still early-return for those zones, which is correct today but
  is the seam to widen if a card ever grants abilities into an opponent's
  graveyard. ⏳
- **Demonic Junker and Riptide Gearhulk are heads-up shaped.** "For each
  player, destroy up to one target creature that player controls" is two
  filtered `OptionalTargets` slots (yours, theirs). A three-player game gets
  one slot per *side*, not per opponent — a per-opponent target multiplier is
  the general fix. ⏳
- **Skyseer's Chariot's tax also hits mana abilities.** That matches the
  printed wording ("activated abilities of sources with the chosen name"), but
  it is the only `ActivationTax`-family static that isn't gated on
  `!is_mana_ability`; re-check if a later card wants the narrower scope. ⏳

## Noticed this run (Homelands closed / Conspiracy + CR 726)

- **The bot has no ballot policy.** `Decision::ChooseOption` falls through to
  `AutoDecider`, which always votes for the first option — so every bot seat
  votes with the ballot's author. The option *effects* aren't visible at the
  decision layer; giving the bot a real vote needs either the effects on the
  decision or a per-card hint. ⏳
- **`Effect::Vote` ties always go to the later option.** That matches every
  printed two-word ballot ("…or the vote is tied"), but a three-option ballot
  or one whose tie-break isn't the last choice would need the tie-winner
  declared explicitly. `VoteTally::AllTied` covers the "each choice with the
  most votes" wording (Council Guardian). ⏳
- **Backup Plan's extra hand is auto-kept.** CR 103.4 lets the player pick
  which of their hands to keep; `start_mulligan_phase` scores them
  (`opening_hand_score`) and keeps the best. A UI pick is a follow-up. ⏳
- **Deal Broker's post-draft trade is unimplemented** — only its `{T}:` loot
  ships. `DraftPod` has the pick loop it would hook into. ⏳
- **Hand-card `CardDiscarded` / `SelfSource` triggers don't dispatch.** The
  trigger walker doesn't reach cards in hand, so "when a spell or ability an
  opponent controls causes you to discard this card, …" never fires (Guerrilla
  Tactics, Alliances — left out of the catalog for that reason; Pure Intentions'
  same-shaped trigger is likewise unexercised, its test covers only the cast
  half). ⏳
- **Grenzo's Rebuttal auto-picks for every seat.** Each player's three choices
  go through `Decision::ChooseTarget` on the shared decider rather than a
  seat-routed suspend, so a UI opponent doesn't get asked. ⏳
- **Tiered / Spree bot ranking is still shallow.** `castable_actions` now
  offers every tier plus the all-modes Spree combination, but the search picks
  among them by generic board eval — there's no cost/impact heuristic. ⏳
- **`FREE_ACTIVATION_REPEAT_CAP` is per (source, ability), not per loop.** A
  fragmented loop alternating between *two* free abilities still spins; the
  fingerprint would catch it but the key changes each activation. ⏳

## Noticed this run (CNS draft shell / CR 706.6 / CR 116.2j)

- **Draft-matters cards are single-seat.** `DraftPod` implements CR 905.2 for
  the human seat and the mandatory notes for every seat, but bots never spend
  Cogwork Grinder / Agent of Acquisitions / Whispergear Sneak / Lore Seeker
  (they do use Cogwork Librarian, which is unambiguous value). Giving the bot a
  policy needs a pick-EV estimate the greedy scorer doesn't have. ⏳
- **Paliano's three colors are auto-chosen.** `DraftPod::paliano_colors` picks
  each seat's strongest unchosen color rather than prompting the drafter or
  their neighbours. ⏳
- **`extra_dice_for` reads printed statics only.** A `RollExtraDiceIgnoreLowest`
  wrapped in `WhileCondition` / `WhileClassLevelAtLeast` wouldn't be seen (no
  printed card needs it — Pixie Guide and Barbarian Class L1 are both
  unconditional). Same shape as `damage_halvers`. ⏳
- **The bot reveals hidden agendas immediately.** `GameAction::RevealConspiracy`
  is taken at the bot's first main phase because the agenda is already named at
  seating time; a bot that could *choose* the name later would want to wait. ⏳
- **TODO gap lists rot.** Most of the "still open, needs one primitive" card
  lists under the recent-set sections were already shipped by later runs — a
  spot check found 21 of 24 present. Re-run `scripts/set_gaps.py` (now a single
  pass over the catalog rather than a `grep -r` per card) before trusting them.

## Noticed this run (recent325 gap batch)

- **Miasma Demon's slot count is a static ceiling.** The printed card is a
  reflexive trigger ("When you do, up to that many target creatures…"); the
  catalog models it as `DiscardAnyNumber` + `CapTargetsAt` over a five-slot
  `ApplyToTargets`, so the targets are declared before the discard and merely
  truncated afterwards. A real reflexive-target primitive would fix this and
  several other "when you do, up to that many target …" cards. ⏳
- **Cards dropped from this batch, each blocked on one primitive.** Shipped
  since (`decks::recent327`): Undead Sprinter (`flashback_condition` now
  gates every graveyard-cast flavor + `Predicate::CreatureDiedThisTurnMatching`),
  Hedge Shredder (`CardMilled`/`YourControl` land filter), Leyline of
  Resonance (`Predicate::CastSpellTargetsOnlyOneMatching`), Leyline of
  Transformation (`StaticEffect::OwnedCardsOffBattlefieldAreChosenTypeToo`),
  Cursed Recording. Still open: Osteomancer Adept (forage-cast from the
  graveyard — the surcharge static is life-only), Heirloom Epic (convoke on
  an activated ability), Rottenmouth Viper (sacrifice-any-number additional
  cost with per-permanent reduction). ⏳

## Noticed this run (DFT closed / BLB Season cycle)

- **Aetherdrift is at zero.** Gonti, Night Minister and Mimeoplasm, Revered
  One shipped with `Effect::{ExileTopFaceDownGrantPlay,
  AsEntersExileFromYourGraveyard, BecomeCopyOfExiledCard}`; The Aetherspark's
  "can't be attacked" half is `Keyword::CantBeAttacked`.
- **Six BLB/DSK cards were written and then cut** rather than shipped
  half-wired — the definitions are in `git log -p` on this run's commits.
  Each needs one thing:
  - *Eluge, the Shoreless Sea* — `DynamicPt::BasePlusLandsOfTypeControlled`
    now counts *computed* land types (`lands_of_computed_type`), but Eluge's
    P/T still reads 1 with a flooded Mountain in play: the CDA path that
    `computed_permanent` actually uses isn't the `game/mod.rs:9410` block the
    fix landed in. Find the live CDA site first. ⏳
  - *Ygra, Eater of All* — a `CreatureDied`/`AnyPlayer` trigger with a
    `TriggerSource` filter never fires for an opponent's creature; the
    granted Food subtype is also gone by trigger time (LKI). ⏳
  - *Zoraline, Cosmos Caller* and *Kastral, the Windcrested* — same shape:
    an `Attacks`/`DealsCombatDamageToPlayer` trigger scoped `YourControl`
    with a `TriggerSource` creature-type filter didn't fire off the source
    itself. Worth a focused test on the dispatcher rather than per-card
    workarounds. ⏳
  - *Wishing Well* — the `{T}` activation errored; the reflexive
    `CastWithoutPayingImmediate` over a `ManaValueEqualsCountersOnSource`
    target slot needs the target enumerator to see the new requirement. ⏳
  - *Starforged Sword* — the gift-gated ETB `AttachSourceTo` left
    `attached_to` unset. ⏳
- **`Effect::ChooseModesByPoints`** (the Season cycle's "choose up to five
  {P} worth of modes") shares Spree's cast plumbing; the bot offers each
  single mode plus the all-modes combination when the prices fit. A
  cost/impact ranking over point spends is still missing. ⏳
- **`DelayedTrigger.expires_after_turn`** gives any delayed watcher an
  "until the end of your next turn" window. Only Season of the Bold uses it
  so far; several "until your next turn" cards could move onto it. ⏳
- **Client: `Keyword::CantBeAttacked` is honored at the attack-redirect
  click** and has tooltip text, but there's no *visual* tell that a walker
  can't be attacked (no greyed frame or cursor change). ⏳

## SOS Special Guests (SPG)

Shipped: `sos_mode::sos_special_guests()` names the eleven-card sheet (nine
were already catalogued for other sets; Magus of the Library and Library of
Leng are new in `sets::sos::spg`), `all_sos_cards` includes it, and
`generate_sos_pack` collates it on its own slot at the printed
`SOS_SPECIAL_GUEST_RATE` (1 in 64) rather than through the colour buckets.

- Adding the sheet grew `Vocab::sos_sealed`, which **invalidates previously
  trained value nets** (embedding rows are index-pinned). Retrain. ⏳
- **Library of Leng always takes the "may"** — a forced discard goes to the
  top of the library, never to the graveyard, so a self-mill deck can't
  decline. ⏳

## Planeshift — closed

`set_gaps.py pls` is at **zero** (`sets::pls`, `sets::pls2`; tests in
`classic_sets/pls`, `pls2`). See FEATURE_ROADMAP.md for the primitives it
shipped.

## Invasion — closed

`set_gaps.py inv` is at **zero** (280 → 233 → 136 → 70 → 40 → 10 → 1 → 0;
`sets::inv::{gaps,gaps2,gaps3,gaps4,gaps5}`, tests in `classic_sets/inv_gaps`–
`inv_gaps5`). Primitives it shipped: `CardDefinition.{flash_surcharge (CR
601.2b), cast_only_during_combat}`, `Effect::{SwapTappedState,
SeparateIntoPiles, ChooseOneAmong, SacrificeSelected, GrantChosenTypeLandwalk,
BidLifeToCounterTargetSpell (CR 601.2b life bidding),
RevealTopTakeNamedExileRest, EachPlayerKeepsOneOfEachBasicTypeSacrificesRest,
PlayerCantPlayLandsThisTurn, RestartGame (CR 727)}`,
`Selector::{SeparatedPile, SharingColorWith, Both}`, `PlayerRef::OpponentOf`,
`StaticEffect::{ColoredSpellTax, PreventSmallDamageToThis, CapLargeDamage,
PreventDamageBetweenSharedColorCreatures,
RedirectChosenColorSpellDamageToController,
YourBasicLandsProduceChosenColorInstead,
CantCastSharingColorWithLastCastSpell}`,
`Predicate::{ColorIsMostCommonAmongPermanents, ControlsLandOfEachBasicType,
ControlsCreatureOfEachColor}`, `SelectionRequirement::{SharesMostCommonColor,
HasNonManaActivatedAbility, SharesNameWithAnotherPermanent,
ManaValueEqualsChosenNumber, TargetsALandYouControl}`,
`WardCost::ManaCostOfAttached`, `CounterType::Hourglass`,
`Effect::RevealTopGreatestMayChangeTargets`, and
`GameState::ask_seat_cards_logged` (a replay-logged `ChooseCards` ask, so an
arm can chain a card pick with further seat questions).

### Residuals in what shipped

- **The pile-split bodies must stay non-interactive** — a body that suspends
  would restart the split on its re-run (`Effect::SeparateIntoPiles`).
- **Kangee's feather counters** are a real `CounterType::Feather`; **Darigaaz,
  the Igniter** now reveals the hand (`Effect::RevealHand`) but still counts it
  live rather than from a snapshot — indistinguishable within one resolution.

The rest of the Invasion residual list is closed: Psychic Battle fires once per
targeting decision (`EventKind::ChoseTargets`) and repoints every slot of a
spell *or* ability (CR 115.7c); Barrin's Spite enforces the same-controller
pairing at cast (`SelectionRequirement::SameControllerAsTargetSlot`); Atalya
spends only white on X (`ActivatedAbility.x_mana_color`); Protective Sphere
reads the activation's mana colours (`SharesColorWithManaSpent`); Samite
Ministration refunds life for black/red sources
(`PreventedSource.gain_life_colors`); Pledge of Loyalty is a continuous
protection grant that can't shed itself (`EquipBonus.protection_keeps_self`);
Prison Barricade's kicked defender bypass is wired.

### Noticed this run (Odyssey wave) — not tackled

- **Interactive bodies inside a pile split** (carried forward): park the split
  result in the resume context, or debug-assert against a suspending body.
- **CR 605.3c** — "once a player begins to activate a mana ability, it can't be
  activated again until it has resolved" isn't modelled (mana abilities resolve
  synchronously, so it has never mattered; it would for a future split-payment
  UI).
- **`Effect::RevealTopOfLibrary`'s reveal ends on a draw, a shuffle, or any
  library insertion. Still open: a scry that reorders the top in place doesn't
  clear it (the scry path doesn't route through `place_card_in_dest`).
- **Earnest Fellowship** ("each creature has protection from its colors") needs
  a self-referential protection filter — a `SharesColorWithSelf` requirement.
- **Dwarven Recruiter / Aether Burst / Cultural Exchange** each need one
  primitive: search-any-number-onto-the-library-top, an as-you-cast target
  count, and a multi-way control exchange.
- **Dreamwinder's attack gate is any Island**, not the defender's, and its
  `{U}`, sacrifice-an-Island land-animation half is dropped (no one-shot
  "target land becomes an Island").
- **The remaining 53 ODY gaps** each want one primitive: Bomb Squad / Mine
  Layer (counter-driven board watchers), Delaying Shield / Nefarious Lich
  (damage replacements), Obstinate Familiar (an optional controller-scoped
  draw skip), Balancing Act, Cultural Exchange, Haunting Echoes, Impulsive
  Maneuvers, Charmed Pendant, Catalyst Stone (a flashback-cost modifier),
  Aura Graft, Seize the Day (an extra combat phase), Predict, Bamboozle.

## Torment — closed

`set_gaps.py tor` is at **zero** (143 → ~48 → 0); `sets::tor` + `sets::tor2`
ship 121 cards (tests in `classic_sets/{tor,tor2}`): the Cephalid self-mill
shell, the Threshold bodies, Chainer, the Nightmare Horrors, the Madness Auras,
the Tainted land cycle, the sweepers, the Dreams cycle and the Possessed cycle.

Primitives it shipped, by wave: `CounterType::Shred`; `CardDefinition::
flashback_additional_cost` (+ `AdditionalCastCost::DiscardXFromCost`) replacing
the old name-keyed flashback-rider table; `WardCost::BottomFromGraveyard`;
`Effect::FlipUntilLoss` (CR 705.1); `ExileReturnZone::Graveyard`;
`StaticEffect::OpponentsCantCastMatching`; `Effect::NextSpellCantBeCountered`
(+ `PlayerView::uncounterable_next` and its HUD chip);
`Value::CardsInAllGraveyardsMatching`. Then the closing wave:
`Effect::RememberPlayerOnSource` + `PlayerRef::ChosenPlayerOfSource` +
`CardInstance.chosen_player`; `Effect::{AnyPlayerMayExileFromGraveyard,
RedirectDrawsThisTurn, DamageBecomesThisTurn, DamageTargetPlayerMayRedirect,
CopySpellForEachOtherTarget, RevealAndReplayNamedPermanent,
CopyEachCreatureToken}`; `Selector::DestroyedThisResolution`;
`AdditionalCastCost::DiscardXRandomFromCost`;
`SelectionRequirement::{PowerLessThanYourGraveyardCount, resolve_is_source}`;
`Predicate::TriggerSourceIsSelf`.

Engine fixes it forced:
- `statics_granted_triggers_for` matched `StaticEffect::GrantTriggeredAbility`
  *literally*, so a trigger granted under a gating wrapper (`WhileCondition`,
  `WhileYourTurn`, …) never surfaced. It now peels through `active_static` —
  and so do `granted_abilities_for` and the `PumpSelfByValue` layer walk.
- CR 611.2b — `remove_effects_from_source` swept `EffectDuration::Indefinite`
  effects too, so a `Duration::Permanent` type stamp died with its source.
- `Effect::ExileUntilSourceLeaves` routed every `Target` through the
  battlefield path, so a graveyard pick silently no-oped (Gravegouger).
- CR 601.2b — `DiscardXFromCost` was concretized only on the flashback cast
  path, so a main-phase "discard X cards" cost discarded nothing.
- CR 603.10a — a permanent's own self-granting static now survives into its
  death LKI ("Threshold — when this dies, …", Reborn Hero).
- CR 400.7 — `ZoneDest::Battlefield { controller: OwnerOfMoved }`.

Open follow-ups:
- **Shambling Swarm** distributes its three -1/-1 counters but doesn't remove
  them at the next end step; that needs a "the counters *this effect* placed"
  selector (a `Selector::CounteredThisResolution` twin of
  `DamagedThisResolution`).
- **Hypnox** gates its hand-exile on `Predicate::TriggerSourceEnteredByCast`,
  which is "cast from anywhere" rather than the printed "cast from your hand".
- **Radiate** copies for each other legal target of the *whole spell*; the
  printed "targets only a single permanent or player" pre-check is enforced by
  the target filter, but a spell whose one slot is a player target is copied
  onto permanents too.

## Legions — closed

`set_gaps.py lgn` is at **zero** (122 → 9 → 0; `sets::lgn`, tests in
`classic_sets/lgn`). The last nine each wanted one primitive; see
FEATURE_ROADMAP.md → "Recently closed" for what shipped.

## Scourge — closed

`set_gaps.py scg` is at **zero** (116 → 39 → 32 → 0; `sets::scg` + `sets::scg2`,
tests in `classic_sets/{scg,scg2}`, 116 cards). The closing wave's primitives
are listed in FEATURE_ROADMAP.md → "Recently closed".

Residual, not blocking the count:
- **Decree of Annihilation**'s cycling half is shipped; the cast half exiles
  hands via a `CardsInZone` sweep that should be audited.
- **Grip of Chaos** retargets at the three stack-push sites (spell, activated
  ability, triggered ability). A spell put on the stack by *another* effect
  (a copy, a cascade free cast) isn't re-randomized.
- **Forgotten Ancient**'s upkeep distribution is engine-chosen (spread evenly
  over the matching creatures) rather than a real "any number, your choice"
  prompt; needs a `Decision::DistributeCounters`-style pick to be faithful.
- **Parallel Thoughts**' draw replacement is offered as a yes/no
  `OptionalTrigger`; the AutoDecider declines it, so bots never mine the pile.

## Mirrodin Besieged — closed

`set_gaps.py mbs` is at **0** (98 → 5 → 0; `sets::mbs`, 98 cards, tests in
`recent_b/mbs`). The closing wave added `Effect::AnyPlayerMayAccept` (the
general "any player may …, if no one does …" shape, with
`PlayerRef::AcceptingPlayer` — Judgment's punisher family now rides it),
`Effect::KnowledgePool`, `StaticEffect::HasActivatedAbilitiesOfExiledWithSelf`
(Myr Welder) and `SelectionRequirement::SameNameAsAPermanent`.

## Legends — opened

`set_gaps.py leg` is at **9** (277 → … → 64 → 43 → 9; `sets::leg`–`leg6`,
264 cards, tests in `classic_sets/leg`–`leg6`). Wave 7 shipped the last
creatures, artifacts, Auras and spells; its primitives are listed in
FEATURE_ROADMAP.md → "Recently closed".

Still open (each blocked on one primitive):

- **All Hallow's Eve** — exile-self-with-counters plus an upkeep trigger that
  fires *from exile*; suspend's shape, but a mass reanimation for every player.
- **Arboria** — "creatures can't attack a player unless that player cast a
  spell or put a nontoken permanent onto the battlefield during their last
  turn": needs a per-player "was active last turn" flag plus a defender-scoped
  attack restriction static.
- **Backdraft** — half the damage dealt by *one specific sorcery spell* this
  turn; nothing tallies damage per spell object.
- **Chains of Mephistopheles** — a draw replacement scoped to "except the first
  draw in each of their draw steps", with a discard-then-draw / mill branch.
- **Equinox** — grant a land "{T}: counter target spell **if it would destroy a
  land you control**"; no predicate reads a spell's effect tree.
- **Knowledge Vault** — exile-face-down linked to the source, with a
  return-to-hand cash-in and a leaves-the-battlefield bin.
- **Land Equilibrium** — a land-ETB replacement scoped to opponents with at
  least as many lands as you.
- **Reverberation** — redirect *all damage a specific sorcery spell would deal
  this turn* to its controller.
- **Wall of Caltrops** — "if at least one other Wall is blocking that creature
  and no non-Wall is": needs a predicate over the blockers of the trigger's
  subject.

### Noticed but not tackled (wave 7)

- **`kill()`-style direct `resolve_effect` calls don't dispatch other
  permanents' triggers** — only the dying object's own. Tests that need an
  Aura/watcher to see a sacrifice must call
  `dispatch_triggers_for_events` (or go through `perform_action`). Worth a
  shared test helper.
- **Halfdane's "until the end of your next upkeep" is modelled as indefinite** —
  correct while he keeps re-triggering, wrong once he leaves. Wants a
  `Duration::UntilYourNextUpkeep`.
- **Cocoon's pupa counters ride the enchanted creature, not the Aura**, so the
  untap lock can read them; proliferate sees them on the wrong object.
- **Nova Pentacle's redirect target is chosen by the activator**, not "of an
  opponent's choice".
- **Juxtapose breaks ties in battlefield order**, not by the printed
  "their controller chooses one of them"; `Nebuchadnezzar` and Petra Sphinx
  name the densest card rather than prompting a seat.
- **Falling Star** settles for one random creature; the printed dexterity
  clause has no analog.
- **Chained `AbilityCostChoice` suspends lose earlier picks** — each replay
  `take()`s every pending cost choice up front, so an ability with two
  interactive cost choices auto-picks the first on the second replay. Only
  matters for a card with two such costs; none ships yet.

### Noticed but not tackled (wave 4/5)

- **Wall of Shadows' "can't be the target of spells that can target only
  Walls"** has no engine analog — there is no "this filter is Wall-only"
  introspection on a spell's target filter.
- **North Star is modelled as a whole-turn permission**, not the printed "for
  one spell this turn"; nothing tracks "the next spell you cast" for a spend
  permission yet.
- **Imprison** ships only its can't-attack / can't-block halves; the printed
  "pay {1} or destroy this Aura" clauses need a repeated pay-or-else rider on
  someone else's activation.
- **Bronze Tablet folds its self-exile into a sacrifice**; the printed
  exile-both-then-swap needs a two-object exile with a linked return.

## New Phyrexia — closed

`set_gaps.py nph` is at **zero** (111 → 30 → 10 → 2 → 0; `sets::nph`, 111
cards, tests in `recent_b/nph`). Phyrexian mana, Infect, Living weapon,
Metalcraft, the five Shrines and the Chancellor cycle's opening-hand reveal
(`OpeningHandEffect::RevealForDelayedTrigger`) all rode existing primitives.
New across the four waves: `Effect::PermanentsEnterTappedThisTurn` (CR 614 —
Due Respect), `Value::{Negate, CountersRemovedThisEffect}`,
`SelectionRequirement::HasPhyrexianManaInCost` (CR 107.4f),
`StaticEffect::PumpSelfByExiledWithStats`, **mana provenance**
(`ManaPool::add_from_creature` / `pay_creature_only` +
`SpellKind::creature_mana_only` + a creature-only auto-tap — Myr Superion),
and `StaticEffect::ArtifactsAreEquipment` with
`Modification::AddArtifactSubtype` (Bludgeon Brawl's granted Equipment
subtype, computed equip {X} and computed +X/+0).

## Judgment — closed

`set_gaps.py jud` is at **0** (`sets::jud`/`jud2`, tests in
`classic_sets/jud`).

## Odyssey — closed

`set_gaps.py ody` is at **zero** (274 → 53 → 30 → 10 → 5 → 0;
`sets::ody::gaps`–`gaps12`, tests in `classic_sets/ody`). Threshold rides
`Predicate::ThresholdActive`; flashback and the Aura/EquipBonus shell were
already in place. Primitives across the run: `Effect::{RevealHand,
RevealTopOfLibrary, PlayerCantCastMatchingThisTurn, SacrificeSourceUnlessCost,
ExileFromGraveyard, BalanceMatching, SearchSameNameToBattlefield,
ExileLibraryCardsNamedLikeExiledThisResolution, RevealTopChooseToGraveyard,
GainControlAndReattachAura, MillAddManaForColoredSymbols}`,
`StaticEffect::{ControllerMaxHandSizeReduced, ReduceColorDamageToYouBy,
ControllerMaySkipDraws, FlashbackCostReduction, OpponentFlashbackTax,
ReplaceDamageToYouWithCountersOnSource, ReplaceDamageToYouWithGraveyardExile,
LifeGainBecomesDraw}`, `WardCost::{ExileFromGraveyard, DamageFromSource}`,
`Keyword::ProtectionFromOwnColors`, `EventKind::PermanentDestroyedByEffect`
(CR 701.7), `SelectionRequirement::{ToughnessAtMostGraveyardCount, HasFlashback,
SharesCardTypeWithExiledBySource, WithCounterAtLeast}`,
`CounterType::{Feather, Mine, Delay}`, `CreatureType::Mystic`,
`Value::{CardsDrawnThisEffect, CardsNamedLikeTriggerSpellInAllGraveyards}`,
`DynamicPt::CardTypeInAllGraveyards`, `AdditionalCastCost::DiscardRandom`,
`ActivatedAbility::exile_from_hand_cost`,
`CardDefinition::counts_as_named_in_graveyard` and `ManaCost::colored_symbols`.

Residual approximations, all documented in-source:
- **Graceful Antelope's land change is permanent.** The printed clause is
  "until this creature leaves the battlefield"; the engine has no such
  duration (`Duration::WhileSourceRemains` would need SBA unwind like
  `GainControlWhileSourceRemains`).
- **Impulsive Maneuvers doubles/prevents for the turn**, not "the next time
  that creature would deal combat damage".
- **Liquid Fire's split is chosen at resolution**, not as the printed
  additional cost (no cast-time choose-a-number cost exists).
- **Karmic Justice retaliates against any opponent**, not specifically the one
  whose effect destroyed the permanent — exact in two-player.
- **Aegis of Honor binds its source at activation** (the shield's chosen-source
  model), so it must be activated in response to the burn spell.
- **Bomb Squad's four-counter check is not a real state trigger** — it runs at
  the end of each of its own two abilities, which covers every way the counters
  can accumulate.

## Apocalypse — closed

`set_gaps.py apc` is at **zero** (123 → 88 → 55 → 43 → 26 → 11 → 0; `sets::apc`
+ `sets::apc2`, tests in `classic_sets/apc{,2}`). Primitives across the run:
`CardDefinition.opponent_discard_deploys` (CR 614 — Dodecapod),
`Effect::BecomeChosenCreatureType`, `StaticEffect::FlagbearersMustBeTargeted`
(CR 601.2c — enforced at cast and activation, preferred by the auto-targeter,
surfaced as `PermanentView.is_flagbearer`), the **and/or kicker** mechanic
(CR 702.32b — `CardDefinition.kicker_options`, `CardInstance.kicked_options`,
`GameAction::CastSpellKickers`, `Predicate::SpellWasKickedWith`, an affordance
per payable subset, plus bot and client casts),
`Predicate::{TargetsHaveIdenticalColors, TargetSharesColorWithControlled}`,
`Effect::{SearchEachBasicLandType, ColoredManaBecomesThisTurn,
SpellBecomesChosenColor, OtherPlayerMayPayToCounter}`,
`CardDefinition.color_override`, `SelectionRequirement::SharesColorWithSacrificed`,
`DelayedTriggerKind::TargetsNextEndStep`, `FlipCoinsChooseCount.stop_on_loss`,
and `CreatureType::{Metathran, Flagbearer, Volver}`.

Residuals in what shipped:

- **Unnatural Selection drops "other than Wall"** — the type prompt isn't
  restricted.
- **Reef Shaman / Tundra Kavu / Shimmering Mirage ride
  `LandsBecomeChosenBasicType`**, whose choice runs through the synchronous
  decider; Tundra Kavu's printed "Plains or Island" isn't narrowed to two.
- **Suffocating Blast's two targets are one slot each**, so it can't be cast
  with only the counter half legal (the printed spell needs both).
- **Emblazoned Golem's `{X}` kicker takes any mana** — the printed "spend only
  colored mana on X, at most one of each color" spend restriction isn't modeled.
- **Tahngarth's Glare's second rearrangement is made by the library's owner**,
  not by the opponent as printed.
- **Ice Cave asks each other seat in turn order** and takes the first willing
  payer rather than running a real "any other player may" window.
- **The client's and/or kicker cast takes the largest payable subset** on
  right-click; a per-subset picker modal is still open (`ClientView
  .kicker_option_sets` already carries every payable combination).

## Prophecy — closed

`set_gaps.py pcy` is at **zero** (134 → 98 → 68 → 43 → 0; `sets::pcy` …
`sets::pcy4`, tests in `classic_sets/pcy{,2,3,4}`). Primitives across the four
waves: `CardDefinition.self_cost_reduction_if`, `AlternativeCost.discard_filters`
(CR 601.2b), `WardCost::GenericXFromCost`,
`Effect::TurnOffDamagePreventionThisTurn`,
`Keyword::{CantAttackIfDefenderHasUntappedLand, CantBlockIfYouHaveUntappedLand}`,
then the closing wave's `Keyword::AttackBlockCostTapAnother` (CR 508.1g/509.1b),
`StaticEffect::{ActivationAdditionalSacrifice, GrantKeywordWhileControllerControlsAtMost}`,
`Effect::{HighestLifeWinsElseDraw, ExileTokensSharingNameWith,
RedirectNextDamageBackAtSource}` and `CounterType::Omen`.

Residuals in what shipped:

- **Copper-Leaf Angel eats one land per activation**, not the printed
  "Sacrifice X lands" for X counters (no `{X}`-scaled sacrifice cost).
- **Endbringer's Revel's "as a sorcery" is `sorcery_speed`**, which also
  requires an empty stack — right in practice, stricter than printed for the
  non-active player.
- **Every `UnlessPlayerPays` tax auto-declines under AutoDecider**, so bots
  never buy off a Rhystic card (Excise, Wild Might, Withdraw). Engine-wide
  policy, not a card gap.
- **`PlayerRef::EachOpponent` resolves to the first opponent**, so "any
  player may pay" is exact heads-up only.
- **Rhystic Cave's "activate only as an instant"** is dropped (a mana ability
  has no timing gate to hang it on).
- **Reveille Squad's untap is a `MayDo`**, so a headless seat declines it.
- **Slicer, Hired Muscle drops "it can't be sacrificed this turn"** — there is
  no sacrifice lock (the card lives in `sets::bot`, not PCY, but the gap is
  the same shape).
- **Forgotten Harvest exiles "one or more" lands** (`MayExileFromYourGraveyard`
  has no count), rather than exactly one.
- **Denying Wind / Search for Survivors auto-decline under AutoDecider** —
  the shared `Effect::Search` bot policy.

## Nemesis — closed

`set_gaps.py nms` is at **zero** (129 → 44 → 22 → 16 → 0; `sets::nms` …
`sets::nms4`, tests in `classic_sets/nms{,2,3,4}`).

Residuals in what shipped:

- **Angelic Favor drops "cast this spell only during combat"** — there's no
  combat-only cast window (`cast_only_after_blockers` is narrower).
- **Wandering Eye uses `OpponentsPlayWithHandsRevealed`**, so its controller's
  own hand stays hidden (the printed line is symmetric).
- **Blinding Angel's skip is `SkipNextCombatPhase`**, which is exact heads-up.
- **Rising Waters' upkeep untap auto-picks the land** (an `up_to: 1` untap),
  rather than prompting the active player.
- **Stronghold Gambit's per-player pick runs through the synchronous decider**
  — a `wants_ui` seat takes the auto-pick (their cheapest creature) instead of
  a modal, the same multi-ask gap as Thieves' Auction.
- **Divining Witch names via `Effect::NameCard`'s suggestion heuristic** for
  headless seats (the densest name in the caster's own library).
- **Overlaid Terrain's granted land ability is a second mana ability**, so a
  land keeps its printed tap-for-one alongside the granted tap-for-two.

## Mercadian Masques — closed

`set_gaps.py mmq` is at **zero** (283 → 65 → 13 → 0; `sets::mmq` … `sets::mmq6`,
tests in `classic_sets/mmq{,2,3,4,5,6}`).

Residuals in what shipped:

- **Volcanic Wind's X is read at resolution**, not "as you cast this spell", so
  a creature that dies in response shrinks the total.
- **Mercadia's Downfall reads `ControlledByOpponent`** rather than "defending
  player" — exact heads-up, wrong in multiplayer.
- **Ley Line's target is picked by the enchantment's controller.**
  `Effect::MayDoBy` routes the *may* and the counter to the active player, but
  the trigger's target slot is still filled at push time by the trigger's own
  controller.
- **`Selector::ChosenCardInHand` auto-picks the first match** (Assembly Hall's
  reveal): selector resolution has no decision hook.
- **Unnatural Hunger's sacrifice is `WardCost::SacrificeMatching`**, so the
  bite fires only when the payer declines *or* can't pay — the printed "of
  their choice" pick is the auto-picker's.
- **Toymaker / Karn's Touch animate at printed mana value**; a cost-changing
  effect on the artifact wouldn't move the body.
- **Conspiracy drops its off-battlefield half** — creature *spells* you control
  and creature cards you own outside the battlefield keep their printed types.
- **Bargaining Table's `EachOpponent`** resolves to the first opponent, so in
  multiplayer the printed "an opponent's hand" choice isn't offered.
- **Caller of the Hunt names its type as it enters** (`as_enters_effect`)
  rather than as an additional cast cost, so a countered Caller never locks a
  type in.
- **Thieves' Auction's repeated draft resolves through the decider
  synchronously** — a `wants_ui` seat takes the auto-pick instead of a modal
  per claim (`ask_seat_cards` allows only one ask per resolution).
- **Charisma's control grab is keyed to the Aura remaining**, which is right,
  but the Aura falling off mid-combat-damage isn't re-checked until the
  trigger resolves.

Carried over, still open:

- **A real prompt for `EachPlayerMayPutPermanentFromHand`** — the last
  auto-picked "may" on a Show and Tell-shaped effect.
- **Wave of Reckoning is a sequential `ForEach`**, not simultaneous.
- **The Rebel/Mercenary tutor filters read `PermanentCard + HasCreatureType`.**
- **`Effect::Search` auto-declines under AutoDecider** (bot policy, not a card
  gap), so every tutor-chain test scripts its pick.
- **A multi-ask sibling of `ask_seat_cards`** — a resolution that needs several
  routed card picks (Thieves' Auction) currently can't suspend more than once.

## Noticed this run (modern_decks — Urza's Saga closure)

`set_gaps.py usg` is at **zero**: the whole Urza block (USG / ULG / UDS) is
closed. Residuals in what the last three waves shipped:

- **Temporal Aperture's free cast rides the card, not the library top.**
  `GrantMayPlay` stamps `may_play_until` on the card, so drawing it or moving
  it off the top doesn't revoke the permission the way "for as long as that
  card remains on top" should.
- **Abundance's land/nonland pick is a bot policy** (dig for whichever kind the
  hand is short on) rather than a real per-draw prompt; the yes/no is a real
  decision, the *kind* is not.
- **Contamination replaces every land's mana**, including its controller's, and
  is not restricted to the first active static (multiple copies are
  idempotent, which is right, but a second colour would be ignored).
- **Okk's partner check reads the declared batch**, so a creature that gains
  power *after* attackers are declared can't retroactively free it (correct)
  but one that loses power can't retroactively lock it either (CR 508.1a is a
  declaration-time check, so this is right — noted only because it looks
  asymmetric).
- **`Effect::UnlessPlayerPays` auto-declines for bots**, so the upkeep-tax
  bodies (Endless Wurm, Child of Gaea, Drifting Djinn, Contamination, Veiled
  Apparition, Power Taint) always take the punishment under AutoDecider. Same
  policy gap as Masticore.
- **Opal Titan's protection rider is dropped** — "protection from each of that
  spell's colours" needs a keyword grant computed from the trigger's spell.
- **Soul Sculptor's blanking is indefinite**, not "until a player casts a
  creature spell".
- **Metrognome's forced-discard trigger is dropped** — there's no "an opponent
  caused you to discard this card" event.
- **Somnophore reads `ControlledByOpponent`** rather than "that player"; exact
  heads-up, wrong in multiplayer.
- **`Effect::GrantKeywordToMatchingThisTurn` matches card-locally**, so a
  creature granted flying by an Aura is still stopped by Falter (CR 613.8
  dependency ordering isn't modeled).

Worth doing next, in rough order of leverage:

- **A real "for as long as it remains on top" permission** — a library-top
  linked `may_play` that revokes on any zone/order change.
- ✅ ~~**The client cannot be built in this environment**~~ — it can, after
  `apt-get install libwayland-dev libasound2-dev libudev-dev libxkbcommon-dev`.
  Worth baking those into the container image: without them the client crate
  silently rots (it had been broken since `CounterType::Fungus` landed, because
  two exhaustive counter-label matches were never updated). `cargo clippy -p
  crabomination_client --all-targets` is now part of the end-of-run sweep.
  Caveat: a full debug build of the client is ~2.5 GB of `target/`, which can
  exhaust the session's disk allowance — `cargo clean -p crabomination_client`
  afterwards.

## Noticed this run (modern_decks — Urza block closure)

`set_gaps.py ulg` and `set_gaps.py uds` are both at zero. **Urza's Saga (USG)
is opened at 254 -> 201** (`sets::usg`, the 53 commons/uncommons that ride
existing primitives). What the remaining USG gaps want, roughly in order of
leverage:

- **A per-player scaling damage/`Value`** — Acidic Soil ("damage to each player
  equal to the number of lands they control") and Disorder both need the amount
  evaluated once per affected player, which `Effect::DealDamage`'s single
  `Value` can't express.
- **"Creatures without flying can't block this turn"** (Falter) — a turn-scoped
  filtered block restriction; the engine only has `CantBlockSourceThisTurn`.
- **"Each player returns a creature they control"** (Curfew) — a per-player
  choose-and-move.
- **Combat-state triggers for "becomes blocked"** exist (`EventKind::
  BecomesBlocked`), but Cave Tiger / Dromosaur also want the blocks-side
  wording on the same body.

Residuals in the rest of what shipped:

- **Iridescent Drake can't be cast at its Aura.** The ETB body works
  (`Effect::AttachAuraFromGraveyardTo`), but a graveyard card isn't a legal
  `Target::Permanent`, so a real cast fizzles the trigger's target. Wants a
  graveyard-card target slot (the same gap Body Snatcher's reanimate half has).
- **Storage Matrix's type choice is auto-picked.** `do_untap` picks whichever
  of artifact / creature / land would free the most permanents; the printed
  card lets each player choose. Wants a real untap-step decision hook.
- **Scrying Glass's colour is auto-picked** from the guesser's own hand
  (`best_color_for_hand`) — only the number is a real prompt.
- **Goblin Festival's handoff goes to `EachOpponent`.** Correct heads-up;
  multiplayer should let the flipper choose which opponent.

- **`Effect::RevealAnyNumberFromHand` reveals nothing visible.** The count is
  right, but no `hands_revealed_to` entry or event, so an opponent never learns
  what the Scent / Seer player showed. Same gap as
  `RevealHandDiscardAllMatching`.
- **Memory Jar's stash is public.** The hands are exiled face down but the
  exile zone is projected in full, so the view leaks them until they return.
- **Encroach discards every matching nonbasic land**, not the one card the
  printed card lets you choose (`RevealHandDiscardAllMatching`).
- **`Effect::ExileAllCopiesOfTargetName` only walks the battlefield and the
  stack for its subject.** A card already in a graveyard can't be the target,
  which is right for the five printed cards but blocks a future reprint shape.
- **`WardCost::DiscardMatching` auto-pays with the first matching card.** No
  prompt, so a UI seat never chooses which creature Body Snatcher eats.
- **`ActivatedAbility.any_player` has no bot policy.** The affordance probe and
  the client menu surface it, but the bot planner treats it like any other
  ability, so it'll happily sacrifice a permanent to Damping Engine even when
  it isn't the locked seat.

## Noticed this run (modern_decks — Mirrodin block closure)

The whole Mirrodin block (MRD / DST / 5DN) now reports zero `set_gaps.py`
gaps. Follow-ups that came out of it:

- **Static-granted triggers still can't be re-homed from the graveyard.**
  `statics_granted_dying_triggers` closes the "when this dies" case (Endless
  Whispers) by walking the death LKI snapshot, but the *dispatcher's* general
  path still evaluates grant filters against `Target::Permanent`. Any other
  leaves-the-battlefield grant (exile, bounce) will hit the same wall.
- ✅ ~~**`Selector::AttachmentGranting` is still missing.**~~ — shipped; it
  resolves to the attachments on the ability's source that grant abilities
  (Hankyu's aim counters land on the Equipment). Residual: two granters on one
  host are indistinguishable, so both match. Rakdos Riteknife / Scythe of the
  Wretched can be moved onto it.
- **Liar's Pendulum's guess is a bot policy stub.** The named card auto-picks
  the first card in hand and the guesser answers through the generic
  `OptionalTrigger` decision. A real `Decision::Guess` (also wanted by Master
  of Predicaments) would let a UI seat bluff.
- **Shared Fate's exile is one-sided in multiplayer.** `draw_one` exiles off
  the first opponent with a non-empty library rather than letting the drawing
  player choose which opponent.
- **Spellweaver Helix imprints without targeting.** `ImprintFromGraveyard`
  picks the graveyard with the most matches and takes the first N; the printed
  "two *target* sorcery cards" is a resolution-time auto-pick.
- **Endless Whispers hands the corpse to the first opponent.** The printed
  "choose target opponent" isn't a real target slot (the delayed trigger's one
  target slot already carries the captured card).
- **`Effect::ApplyToTargets` slots don't auto-fill for triggers.** An ETB
  trigger whose body is `ApplyToTargets { max_targets: 2 }` only ever gets
  slot 0 filled — `auto_extra_distinct_slot_targets` bails on it because
  `distinct_target_count` is `Some`. Worth splitting the divide-effect check
  from the independent-slot check.
- **Next set gaps:** Kamigawa block is the biggest remaining chunk. CHK is
  down to ~15 after the `sets::chk2` wave; `bok` 112, `sok` 131 untouched.
- **`run_effect`'s stack frame is load-bearing.** New non-trivial arms go in
  `#[inline(never)]` helpers. `Effect::SearchUpToN` is now iterative on the
  non-suspending path (it used to recurse once per pick and Grozoth's 20-card
  chain would overflow a test thread's stack), but any other per-item
  recursion will hit the same ceiling.
- **CR 723 player control is state + routing only.** `acting_seat_for` lets
  the controller send actions for the controlled seat and CR 723.4 shares the
  hand, but the *client* still renders from its own seat — it shows the
  "⛓ seat N" chip without switching its action UI to the controlled board.
  Word of Command / Opposition Agent's limited-duration control (CR 723.2) is
  not modeled.
- **Quicksilver Elemental drops its colour-relaxation rider.** "Spend blue as
  though it were any colour to pay this creature's activation costs" needs a
  source-scoped payment relaxation; the engine only has the table-wide
  `PlayersMaySpendManaAsAnyColor` (Mycosynth Lattice).
- **`Effect::SearchExileThenTokensPerCard` auto-takes every match** (Myr
  Incubator). Correct for a rational player, but a `wants_ui` seat gets no
  pick.

## Noticed this run (modern_decks — Kamigawa CHK gap wave)

`sets::chk2` closed 71 CHK gaps and `sets::chk3` closed the last 6 — CHK is
complete. Follow-ups:

- ✅ ~~**Still-open CHK cards**~~ — `set_gaps.py chk` is at zero
  (`sets::chk3`, tests `classic_sets/chk3`).
- **Hankyu's removal is a resolution step, not a cost.** The printed line is
  "{T}, Remove all aim counters from Hankyu:"; the catalog does the removal at
  the head of the resolution so the damage can read the count. A real
  `remove_all_counters_from: Option<(Selector, CounterType)>` activation cost
  (plus a `Value::CountersRemovedThisCost`) would make it exact.
- **`Selector::AttachmentGranting` can't disambiguate two granters.** Two
  Hankyus on one creature both match; the counters land on both.
- **Konda's Banner's shared-trait filters read printed characteristics.** They
  are evaluated inside the layer gather, so the computed view would recurse —
  a granted colour/type (Changeling, Swirl the Mists) isn't seen. Same
  restriction applies to `StaticEffect::PumpPerBushido` (printed bushido only,
  so Sensei Golden-Tail's grant doesn't feed Takeno).
- **Kusari-Gama reads "defending player" as "the opponent".** Its splash hits
  the opponent's non-blocking creatures, which is exact at two players only.
- **`audit_catalog_stats.py` reads the right object now.** Four fixes: a
  flip/DFC back-face definition is compared against its own `card_faces` entry
  (not the front's stats); nested `TokenDefinition { … }` literals are stripped
  so a token's name/P/T/subtypes don't shadow the card's; every field is read
  at the *top level* of the function's own `CardDefinition` literal, so a
  `StaticAbility { PumpPT { power: 3 } }` no longer reads as printed power; and
  the parameterized `ProtectionFrom*` keywords map to Scryfall's plain
  "Protection" (plus a `Assembly-Worker` / `Time Lord` type alias). That took
  the catalog-wide P/T column from ~120 rows of noise to a real list of 30,
  **all now fixed**, along with the four CHK bugs the first pass exposed.
- **Remaining catalog stat drift**: the cost column is down to **6 rows**, all
  deliberate `{X}` modelling (Lunar Frenzy, Form a Posse, March of Otherworldly
  Light, Primal Might, Bond of Agony, Overrule are coded without an `x()` pip
  because X rides the cast's `x_value`) — either teach the audit to accept that
  shape or move the six onto `x()`. Still open: **17 type + 35 keyword rows in
  `decks`** plus a small tail (`mod_set` 1+6, `one` 5+1, `dis` 3, `mh3b` 2, …).
  Some type rows are deliberate party-synergy widenings (Stonework Packbeast,
  Tajuru Paragon carry every party class for the party tests), so this column
  wants a card-by-card read rather than another mechanical sweep. Run
  `python3 scripts/audit_catalog_stats.py <set>` for the detail.

## Noticed this run (modern_decks — CHK closure + Ravnica block)

- **The prepare-spell copy isn't materialized in exile.** CR 722.3c says the
  copy lives in exile while the permanent stays prepared;
  `GameAction::CastPrepareSpell` mints it at cast time instead. Nothing that
  looks at exile can see or interact with it (`cr_recent45::cr_722_*`).
- **`Effect::RevealLibraryNamedCountPunish` doesn't show the library.**
  Mindblaze's "target player reveals their library" is resolved by the engine's
  own count; there's no reveal state for a UI seat to look at.
- **`Effect::AlternatingExileFromHand` never suspends.** Struggle for Sanity
  drives both sides through the synchronous decider, so a UI seat doesn't get
  its own picks.
- **`Effect::ChangeTargetOfAbility` retargets the whole slot vector** (CR
  115.7c) — each additional slot is repointed against its own filter, keeping
  the current target when nothing else is legal. Remaining: the chooser is
  all-or-nothing per slot rather than "any subset".
- **`Effect::WarpWorld` deploys in printed name order, not player choice.**
  The two waves are correct (artifact/creature/land, then enchantment) but a
  player never chooses the order within a wave, and Auras revealed this way
  have no attach step.
- **`StaticEffect::AllColorWordsBecomeChosen` only rewrites keywords.** Swirl
  the Mists reaches `Keyword::Protection(color)` through the layer-3
  `ReplaceColorWord` modification; colour words inside *ability text*
  (`StaticEffect`/`Effect` colour parameters) are untouched.
- **`Effect::LookAtHandCastFree` auto-picks.** Mindleech Mass takes the
  priciest nonland card rather than prompting; the look is recorded in
  `hands_revealed_to` so a UI seat can at least see the hand.
- **`granted_replicate_cost` doesn't stack with printed replicate.** A card
  that already has `Keyword::Replicate` keeps its printed cost under Djinn
  Illuminatus (correct), but two Djinns don't grant two instances.

## Noticed this run (modern_decks — Ravnica-block gap sweep)

- **Divided-damage abilities and the UI cursor.** `Effect::DealDamageDivided`
  now reports slots past the first as optional (`min_targets_in_mode` = 1), so
  an activated divided-damage ability accepts one target (Living Inferno). The
  client still can't *collect* more than one target for an ability, so the
  multi-target half of those abilities is bot/auto-only.
- **`AnthemForFilterIf`'s predicate is source-scoped.** The new conditional
  anthem evaluates its `Predicate` with an ability context anchored on the
  source, so predicates that need a per-affected-permanent subject (rather
  than the anthem's own source) can't be expressed yet.
- **Rakdos Riteknife's granted line lives on the Equipment.** The printed
  ability is granted to the equipped creature; `equipped_bonus.
  activated_abilities` has no way to name the granting Equipment from the
  host's context (`Selector::This` binds to the host), so the counter-add is
  modeled as an Equipment ability with a tap-an-equipped-creature cost.
  Wants a `Selector::AttachmentGranting` (or equivalent).
- **Chant of Vitu-Ghazi's lifegain rider** — `PreventAllCombatDamageThisTurn`
  is a turn flag, not a set of shields, so there is no per-point hook to gain
  life from. Either give the flag a `gain_life_for: Option<usize>` companion or
  express the fog as prevention shields (which already carry `gain_life`).
- **Concerted Effort drops landwalk and protection** from its shared set: both
  are parameterized keywords (`Landwalk(LandType)` / `Protection(Color)`), so
  "share it if a creature you control has that ability" needs the *instance*
  discovered at resolution rather than a fixed keyword list.
- **Tunnel Vision / the NameCard family auto-pick the densest name.** A
  `wants_ui` caster should be prompted for the name (the same residual as
  Petrified Hamlet's `NameCard`).
- **The Ravnica block is complete** — RAV, GPT and DIS all report zero
  `set_gaps.py` gaps.
- **Breath of Fury re-attaches to the first legal creature.** The printed line
  lets the controller choose which creature the Aura moves to.
- **Sunforger's search auto-picks the priciest legal instant.** No prompt for a
  `wants_ui` caster; the same residual as the other search-and-cast effects.
- **Eye of the Storm skips copies.** A copy that would join the pile is
  ignored (CR 707.10a — it would cease to exist off the stack anyway);
  without that guard the free copies re-trigger forever.
- **Flickerform's return isn't a choice.** Every Aura exiled this way comes
  back attached; the printed "if you do" only gates on the host returning,
  which is modeled, but an Aura that could no longer legally enchant the host
  should stay in exile.

## Tier 4 — remaining SOS/SOA audit simplifications (2026-07)

The 2026-07 SOS/SOA correctness audit fixed every WRONG card and the
tier 1–3 simplifications (see the `claude/modern_decks` commits ending
at "Simplification tiers 2-3"). What's left all needs real engine
machinery:

- **Batch-trigger coalescing** — the highest-value item; CR-correct
  "whenever ONE OR MORE [things happen]" triggers fire once per batch,
  not once per thing. Design: group simultaneous events at emission time
  (`CardsLeftGraveyard { count }`, an attacker-declaration batch event)
  with per-event fallbacks for single subjects. Fixes Garrison Excavator
  (over-mints a Spirit per card on delve/mass exile), Berta, Wise
  Extrapolator (fires per +1/+1 counter instead of per batch), and
  Living History ("whenever you attack" currently fires per attacker
  and pumps every attacker instead of one target attacker, once).
  Benefits every future "one or more" card.
- **Reflexive "when you do" sub-triggers** — Rubble Rouser's "{T},
  Exile a card…: Add {R}. When you do, deal 1 to each opponent" resolves
  cost + rider as one lump; the printed reflexive trigger should go on
  the stack separately (respondable).
- **Zaffai's special action** — "once during each of your turns, you may
  cast an instant/sorcery from your hand for free" is approximated as a
  precombat-main grant on one pre-picked card. Faithful support: a
  whole-hand, once-consumable may-play permission valid any time during
  your turn.
- ✅ ~~**Fractalize's type/color rewrite**~~ — wired via the existing
  `Effect::BecomeColor` (layer 5) + `Effect::BecomeCreatureType` (layer 4)
  alongside the base-P/T override.
- Multiplayer "target player" collapses (Ral Zarek, Guest Lecturer's
  −1/−7 and friends) — tracked in the dedicated multiplayer worklist
  table at the bottom of this file.

**Remaining DIS/RTR gap cards (each blocked on one engine primitive):**
- **Filtered exile-free-cast** — Epic Experiment (RTR): exile top X, free-cast
  I/S with MV ≤ X, rest to graveyard (a filtered `ExileTopAndGrantMayPlay` that
  bins non-cast cards).
- ✅ ~~**Colour-add layer**~~ — Grave Betrayal's reanimated creature now carries
  a real layer-5 `Modification::AddColor(Black)` continuous effect, sourced to
  the creature so it is swept when that creature leaves.
- Others needing bespoke work: Experiment Kraj (dynamic "has all activated
  abilities of counter-bearing creatures"), Rakdos Riteknife (blood counters),
  Search the City (name-replay → extra turn), the DIS split cards (Bound //
  Determined, Odds // Ends, Research // Development), and the block of
  planeswalkers (Jace AoT, Vraska the Unseen, Gideon CoJ, Domri already done).

**WAR (War of the Spark) — COMPLETE.** `scripts/set_gaps.py war` → 0. The final
ten bombs shipped in `modern_decks` (Tezzeret Master of the Bridge, God-Eternal
Kefnet, Nissa Who Shakes the World, Nicol Bolas Dragon-God, Bolas's Citadel,
Feather, Finale of Promise, Deliver Unto Evil, Gideon's Sacrifice, Niv-Mizzet
Reborn). Residual nuances left as follow-ups:
- **Kefnet's drawn-I/S copy is cast free rather than for {2} less** — needs a
  paid-with-discount copy-cast path (`cast_card_for_free` is free-only; the
  copy would want a granted alt-cost = `card_cost.reduce_generic(2)`).
- **Bolas's Citadel's sac-ten** uses `sac_other_filter (Nonland, 10)`, so the
  Citadel itself can't be one of the ten (real card allows it).
- Loyalty `x_cost` abilities still don't stamp X into *target-legality* filters
  (resolution-time `CardsInZone` picks do); a genuinely targeted "target … with
  mana value X" loyalty ability can't gate on X at activation.

**Addendum (RNA) mechanic** — SHIPPED via `Predicate::YourMainPhase` (an
instant resolves in the step it was cast, so a resolution-time "your main
phase" check faithfully captures the Addendum trigger). Arrester's Zeal /
Arrester's Admonition ride it. RNA's `catalog::sets::rna` is now deep;
`scripts/set_gaps.py rna` shows the remaining gaps, all rares/mythics needing
new primitives: Amplifire (reveal-until-creature, base P/T = 2× until next
turn — needs `SetBasePowerToughness` + the reveal capture); Captive Audience;
Domri, Chaos Bringer (the riot-mana rider is the blocker) and Theater of
Horrors (a may-play permission gated on "an opponent lost life this turn") are
the last two RNA cards; everything else in this list shipped. Ravager Wurm's
land-destroy mode still wants a non-mana-ability land filter. Biomancer's Familiar ships the cost-reduction static; its {T}
adapt-reset rider still needs an "adapt as though no counters" primitive.

**Remaining DGM (Dragon's Maze) gap cards** — `catalog::sets::dgm`. The
`dgm::gaps`/`gaps2` waves shipped the guild legends/mythics and easy commons
(Sire of Insanity, Savageborn Hydra, Exava, Ruric Thar, Lavinia, Blood Baron,
Mirko Vosk, Tajic, Vorel, Zhur-Taa Ancient, Smelt-Ward Gatekeepers, Scion of
Vitu-Ghazi, Rot Farm Skeleton, Gleam of Battle, Debt to the Deathless,
Obzedat's Aid, Drown in Filth, Blast of Genius, Pyrewild Shaman, Maze's End,
Aetherling, Dragonshift, Krasis Incubation, Armed//Dangerous, Protect//Serve,
Down//Dirty; gaps3 added Blaze Commando, Teysa, Deadbridge Chant, Progenitor
Mimic, Council of the Absolute; gaps4 added Notion Thief, Varolz, Boros
Battleshaper). DGM is now at zero `set_gaps.py` entries.
- The remaining Fuse splits (Catch // Release — multi-type edict; Flesh // Blood
  — exile-gy-and-counter-by-power).

**Batched-ETB infrastructure (blocks several OTJ legends):** "whenever one or
more [creatures/tokens] enter" triggers currently fire per-permanent, not
once per simultaneous batch. A real batch-ETB dispatch (dedup at trigger
dispatch, mirroring the death/leave-graveyard batch machinery) would unblock:
- Satoru, the Infiltrator (OTJ) — "Satoru and/or nontoken creatures you control
  enter, if none were cast or no mana was spent → draw"; needs the batch plus a
  per-batch "none were cast / no mana spent" predicate (per-creature
  `Not(SourceWasCast)` misses the free-cast "no mana spent" case).
- Kambal, Profiteering Mayor (OTJ) — "one or more tokens you control enter →
  drain 1" (once per batch, not per token) + "opponents' tokens enter → copy
  each as a tapped token, once each turn".
- Geralf, the Fleshwright (OTJ) — "Zombie enters → +1/+1 per other Zombie that
  entered this turn" (needs a zombies-entered-this-turn batch count).

**Other OTJ legends still open (each needs one primitive):**
- ✅ ~~Vraska, the Silencer~~ — shipped (recent290): `on-opponent's-nontoken-
  creature-dies` → `MayPay {1}` → `Move(TriggerSource → battlefield tapped,
  You)` + `BecomeTreasure(LastMoved)`. The reanimate-the-dying-creature
  mechanism is the same one Witherbloom Necromancer/Minion's Return use.
- Geralf/Breeches — "cast your second/Nth spell each turn" is actually already
  expressible: a `SpellCast` trigger + `Predicate::SpellsCastThisTurnEquals`
  (Cori-Steel Cutter's Flurry uses exactly this). Breeches still needs the
  "exile top of each opponent's library, you may play them, any color" impulse
  primitive; Geralf still needs batch-ETB (below).
- Artist's Talent (BLB) — last unshipped Talent; needs level-gated
  cost-reduction + noncombat-damage-replacement on the non-layer static paths.

**M15 (Magic 2015) — near-complete.** `sets::m15` ships the convoke shell,
the Paragon and Soul cycles, the land-type cycle, and the common/uncommon
core; `set_gaps.py m15` is down to the planeswalkers (Ajani Steadfast, Garruk
Apex Predator, Nissa Worldwaker, Jace the Living Guildpact), the bespoke rares
(Avacyn Guardian Angel, Kurkesh, Jalira, Master of Predicaments, Might Makes
Right, Goblin Kaboomist, Mercurial Pretender, Spirit Bonds, Waste Not,
Necromancer's Stockpile, Boonweaver Giant, Brood Keeper, Constricting Sliver,
Aetherspouts, Aggressive Mining, Stain the Mind, Spectra Ward, First Response,
Feast on the Fallen, Avarice Amulet, Shield of the Avatar, Burning Anger,
Ensoul Artifact, Chief Engineer, Genesis Hydra's reveal half) and **The Chain
Veil**, which needs per-turn loyalty-activation tracking
(`Player.loyalty_activated_this_turn` + a "reset the per-walker activation
flag" effect).

**Two M15 rares bounced off a primitive gap this run** (written, tested,
reverted rather than left half-wired):
- **Waste Not** — `EventKind::CardDiscarded` + `EventScope::OpponentControl`
  with an `EntityMatches { what: TriggerSource, filter }` gate never fires;
  the discard event's trigger source doesn't resolve as a matchable card for
  the type split (Zombie / mana / draw). Needs the discard event to carry the
  discarded card as a filterable `EntityRef::Card`.
- **Yisan, the Wanderer Bard** — `R::ManaValueEqualsSourceCounters(Verse)`
  isn't rewritten to a concrete MV inside `Effect::Search`, so the tutor
  matches nothing. `resolve_source_power`-style rewriting exists; `Search`
  needs the counter-count sibling.

**Spectra Ward's CR 704.5n exception.** The printed "This effect doesn't
remove Auras" carves the enchanting Aura out of the protection legality check.
The engine's SBA drops any Aura whose host has protection from its color, so
Spectra Ward would fall off on resolution — it's the one M15 Aura left out.
Wants a per-`EquipBonus` "ignore this bonus for the attachment-legality check"
flag (or a linked-Aura exemption in `check_state_based_actions`).

**Noticed this run (JOU wave 2/3), deferred each on one primitive:**
- **"You control target player during their next turn"** (Worst Fears,
  Mindslaver, Emrakul). Needs the decision + priority router to hand another
  seat's turn to a controller (`Player.controlled_by` consulted by
  `player_with_priority`, `ask_seat_*`, and the bot policy). The only JOU gap.
- **Per-target cost reduction.** `cost_reduction_for_spell` only sees target
  slot 0, so Battlefield Thaumaturge's "for EACH creature it targets" is a
  flat {1}. Threading the whole chosen target list into the cost hook would
  also un-approximate Strive/Fireball interactions with cost statics.
- **Exile-with-attached-Auras.** Silence the Believers, Deicide's Aura rider,
  and O-Ring-adjacent cards want a "and all Auras attached to them" mover;
  today the Auras fall off to SBA and land in the graveyard.
- **"Blocks or becomes blocked by"** as one event. Godsend, and every future
  "whenever this blocks or becomes blocked" card, currently ride
  `EventKind::Blocks` only.

**Noticed this run (recent290), deferred each on one primitive:**
- **Undead Sprinter** (BR 2/2, cast from graveyard if a *non-Zombie* creature
  died this turn, enters with a +1/+1 counter) — needs (a) a typed per-turn
  death tally (`creature_types_died_this_turn` + a `NonTypeCreatureDiedThisTurn`
  predicate; only aggregate `creatures_died_this_turn` counts exist) and (b) a
  conditional own-cost graveyard-cast keyword for a *creature* (Flashback exiles
  on resolution, so it's wrong for a permanent; Escape needs the exile cost).
- **Tarrian's Journal // The Tomb of Aclazotz** — a Book↔Cave transforming DFC
  with a graveyard-cast grant on the land back; no Book/Cave transform-to-land
  primitive.
- **Rotisserie Elemental** (skewer-counter impulse), **Sentinel of Lost Lore**
  (exile-Adventure modal) — still deferred (see WOE section).
- **RTR gap remainder — deferred, each on one primitive:** Slaughter Games
  (name a card → exile all copies from a target opponent's gy/hand/library +
  shuffle — a `NameCard` suspend/resume sweep). Angel of Serenity (exile up to
  three battlefield creatures *and/or* graveyard creature cards, linked to the
  source, returning to hand on LTB — multi-target ExileUntilLeaves across zones).
  Sphinx of the Chimes
  (discard-two-same-name activation cost). Epic Experiment (exile top X, free-cast
  only I/S with MV≤X, rest to graveyard — a filtered `ExileTopAndGrantMayPlay`). Tablet of
  the Guilds (choose-two-colors + cast-of-chosen-color lifegain). Guild Feud
  (top-of-library fight), Grave Betrayal (mass reanimate replacement), Search the
  City (extra-turn combo), Azor's Elocutors (filibuster-counter win). Legends:
  Mercurial Chemister, Trostani, Isperia, Vraska the Unseen, Jace AoT, Rakdos LoR.
- **Gatecrash (GTC) — in progress (modern_decks):** fifteen waves shipped
  (gtc..gtc15). Wave 15 (this run) closed most of the remaining gaps —
  Alms Beast, Hold the Gates, Way of the Thief, Diluvian Primordial, Five-Alarm
  Fire (reusing `CounterType::Charge` for blaze counters; two triggers cover the
  player/creature combat-damage split), Simic Manipulator, Tin Street Market,
  Armored Transport, Vizkopa/Duskmantle Guildmage, Mystic Genesis, Borborygmos,
  Obzedat, Ooze Flux, Mark for Death; see FEATURE_ROADMAP "Already shipped".
  **Wave 16 (this run) shipped 8:** Aurelia's Fury (new
  `damaged_this_resolution` scratch + `Selector::DamagedThisResolution`),
  Glaring Spotlight (`StaticEffect::IgnoreOpponentsCreatureHexproof`), Nightveil
  Specter (`ExileTopAndGrantMayPlay` + `WhileExiled`), Bane Alley Broker
  (`ExileChosenFromHand` link/face-down flags + `OwnerOfMoved`), Signal the
  Clans (`Effect::SignalTheClans`), Unexpected Results (`Effect::UnexpectedResults`
  + `return_resolving_spell_to_hand`), Soul Ransom (`GainControlWhileSourceRemains`
  + `opponents_only`/`discard_cost`/`SacrificeSource`), Vizkopa Confessor
  (`Effect::PayLifeRevealExileFromHand` — pay-any-life → reveal-N → exile one;
  `EachOpponent` is exact in 1v1).
  **Still open (each needs a real new primitive):**
  (Guardian of the Gateless, Gideon Champion of Justice, Lazav, and
  Illusionist's Bracers all shipped — GTC has zero gaps.)
  Note: `Effect::ExileReturnNextEndStep` always returns under You **with a
  +1/+1 counter** (Semester's End shape); the plain flickers that reuse it
  (Cloudshift-likes) shouldn't add a counter — audit and split them onto the
  new `ExileReturnToOwnerNextEndStep` / a counter-less You variant.
- **Ravnica guild remainder (recent291 follow-ups):** shipped Simic Guildmage
  (`Effect::MoveCounter` + `Effect::Attach` aura-restitch), Golgari Guildmage,
  Necromantic Thirst (`EquipBonus.triggered_abilities` combat-damage trigger),
  and Vigor Mortis's real "+1/+1 if {G} was spent" rider
  (`Predicate::ManaSpentOfColorAtLeast`). Simic's "same controller" targeting
  clause is still approximated (both slots open). **Still deferred:** Gaze of
  the Gorgon (regenerate + delayed "destroy all creatures that blocked/were
  blocked by it" at next end of combat — needs a per-turn blocked-by relation
  tracked in combat). Golgari Brownscale's "gain 2 when returned to hand from
  graveyard" now ships via the new `EventKind::PutIntoHandFromGraveyard`
  (emitted at the `movement.rs` gy→hand chokepoint + the dredge return,
  dispatched as a SelfSource trigger off the card now in hand).
- **Ravnica batch 2 (recent292) discovered gaps:** "if {R} was spent to cast
  this creature" ETB riders now ship via `Predicate::SourceCastWithColorSpent`
  (reads the permanent's own `cast_mana_spent_by_color` — Gruul Scrapper,
  Steamcore Weird). **Radiance now ships** (`Effect::RadianceDamage { subject,
  amount }` — damages the chosen creature + each other creature sharing a
  computed color; Cleansing Beam, Wojek Embermage). The *untap+pump* Radiance
  variant now ships too via `Selector::RadianceGroup { subject }` (subject +
  color-sharers as a reusable set — Rally the Righteous).
- **Ravnica batch 6 (recent296) — shipped:** Rally the Righteous
  (`Selector::RadianceGroup`), Vertigo Spawn (`EventKind::Blocks` +
  `Selector::BlockedAttacker` tap + skip-untap — the "no selector for the blocked
  attacker" note was stale, `BlockedAttacker` reads `block_map`), Souls of the
  Faultless (new `EventKind::DealtCombatDamage` combat-only recipient event +
  `PlayerRef::CombatDamagerController`, which reads a `CardInstance`
  `combat_damager_controller` stamp that survives combat teardown), plus Tin
  Street Hooligan, Petrahydrox, Shadow Lance, Shielding Plax, Dowsing Shaman,
  Poison the Well, Congregation at Dawn, Peregrine Mask.
- **Ravnica batches 7–8 (recent297–298) — shipped:** Wojek Siren (Radiance
  pump via `RadianceGroup`), Flame-Kin Zealot, Agrus Kos, Sunhome Guildmage,
  Necromancer's Assistant, Mark of Eviction, Golgari Germination, Corpse
  Blockade, Vulturous Zombie, Grave-Shell Scarab, Vindictive Mob, Seed Spark.
  **Still open Ravnica cards noticed but not built (each on one primitive):**
  Sadistic Augermage (each player puts a hand card on top of library on death —
  needs a "each player tucks a chosen card" effect), Woodwraith Corrupter
  (animate a target Forest into a 4/4 that's still a land — land-animation of a
  *targeted* land), Gobhobbler Rats (Hellbent conditional keyword-grant static),
  Gatherer of Graces "+1/+1 per Aura attached" (per-aura self-scaling P/T).
- **Ravnica/Dissension batches 11–14 (recent301–304) — shipped:** the Eidolon
  cycle (5, via new **`FromYourGraveyard`-scoped SpellCast dispatch** in
  `fire_spell_cast_triggers` — "whenever you cast a multicolored spell, return
  this from your graveyard"), Sadistic Augermage (`EachPlayerPutsHandCardOnTop`),
  Gobhobbler Rats (**`GrantActivatedAbility.condition`** + **`Selector::This`
  self-grant** — Hellbent regenerate), Perplex (**`WardCost::DiscardHand`** +
  `CounterUnless`), Terraformer (**`LandsBecomeChosenBasicType`**), Skeletonize
  (`WhenTargetDiesThisTurn` → Skeleton token), plus Haazda Exonerator, Ogre
  Gatecrasher, Whiptail Moloch, Utvara Scalper, Gnat Alley Creeper, Silkwing
  Scout, Vesper Ghoul, Patagia Viper, Squealing Devil, Slaughterhouse Bouncer,
  Transguild Courier, Wakestone Gargoyle, Ragamuffyn, Soulsworn Jury, Stoic
  Ephemera, Demon's Jester, Minister of Impediments, Flame-Kin War Scout, Rakdos
  Ragemutt, Delirium/Vision Skeins, Psychotic Fury, Might of the Nephilim
  (`Value::Times`×`ColorCountOf`), Stomp and Howl; Guildpact gaps (recent305):
  Battering Wurm, Caustic Rain, Daggerclaw Imp, Dryad Sophisticate (nonbasic
  landwalk), Harrier Griffin, Gristleback (`Value::PowerOf`), Frazzle (nonblue
  counter), Abyssal Nocturnus (opponent-discard payoff). CR conformance in
  `core_rules/cr_recent7` (305.7 / 202.2b / 701.15).
- **Dissension gap batch 2 (dis/gaps) — shipped:** Kill-Suit Cultist (new
  `Effect::ReplaceNextDamageWithDestroy` — a one-event `PreventionShield.destroy`
  rider; the shield-application path now `destroy_permanent`s the target after
  soaking, a helper extracted from the `Effect::Destroy` arm), Nettling Curse
  (enchanted-creature attacks/blocks life-drain — the equip-bonus trigger
  dispatch *already* covered combat-kind triggers via `equip_granted_triggers_for`,
  so the old "only step-kind" note was stale), Riot Spikes (+2/-1 aura), Street
  Savvy (+0/+2), Proper Burial (`ToughnessOf` now reads dead-creature LKI via
  `as_card_id`, matching `PowerOf`), Rain of Gore (`LifeGainBecomesLoss::EachPlayer`),
  Skullmead Cauldron (two activated, discard cost), Celestial Ancient
  (cast-enchantment → team +1/+1), Nihilistic Glee (discard-drain + hellbent draw),
  Slithering Shade (Defender + `{B}` pump + hellbent `CanAttackIgnoringDefenderWhile`),
  Ocular Halo (EquipBonus grants `{T}: draw` + `{W}` vigilance grant), Sprouting
  Phytohydra (`DealtDamage`→`MayDo` token copy of self), Ratcatcher (Fear +
  upkeep may-tutor a Rat), Cytospawn Shambler / Cytoplast Manipulator (Graft +
  counter-gated grant / `GainControlWhileSourceRemains`), Paladin of Prahv
  (Lifelink + a real Forecast rider — `Effect::GainLifeWhenTargetDealsDamageThisTurn`
  registers a `DelayedKind::SourceDealsDamageThisTurn` watcher fired from all
  three damage paths). Plumes of Peace, Govern the Guildless, and Sky Hussar
  also ship their real Forecast abilities via the `forecast()` shortcut. Wit's
  End (discard whole hand),
  Weight of Spires (`NonbasicLandCountControlledBy(ControllerOf(Target))`),
  Tidespout Tyrant (cast-a-spell → bounce), Taste for Mayhem (+2/+0 plus a
  hellbent +2/+0 via the new `EquipBonus.conditional_pt` — a predicate-gated
  layer-7c attached-creature pump). Server/UI: the `destroy` shield now surfaces as
  `PermanentView.doomed_next_damage` (a danger badge, not `has_prevention_shield`)
  with a client tooltip line; fixed a latent non-exhaustive `WardCost::DiscardHand`
  match in the client.
  **Still open Dissension cards (each on one primitive):**
  Valor Made Real / "can block any number" — needs a `Keyword::CanBlockAnyNumber`
  + a blocker→multiple-attacker relaxation (the `block_map` is blocker→single
  attacker today); Gaze of the Gorgon (regenerate + delayed destroy-all-blocked —
  needs a per-turn blocked-by relation in combat). `EquipBonus.conditional_pt`
  (predicate-gated attached-creature pump) now exists — reuse it for other
  condition-gated aura P/T riders.
- **Dissension gap batch 3 (dis/gaps) — shipped:** Nightcreep, Demonfire,
  Biomantic Mastery, Leafdrake Roost, Brain Pry, Grand Arbiter Augustin IV,
  Magewright's Stone, Hellhole Rats, Blessing of the Nephilim, Voidslime,
  Cytoshape, Rakdos the Defiler, Dread Slag, Avatar of Discord, Omnibian,
  Unliving Psychopath, Govern the Guildless, Anthem of Rakdos, Plumes of Peace,
  Freewind Equenaut. New engine primitives: player-target slots surfaced from
  `Creature/PermanentCountControlledBy` values, `NameCardTargetDiscardsOneOrYouDraw`,
  `StaticEffect::OpponentSpellsCostMore`, `EquipScale.count_host_colors`,
  `Effect::CounterSpellOrAbility`, `DynamicPt::BaseMinusPerCardInHand`,
  `StaticEffect::DoubleYourSourcesDamageWhileHellbent`.
  **Forecast (CR 702.56) shipped** via the `forecast()` shortcut for Plumes of
  Peace, Govern the Guildless, Writ of Passage, Sky Hussar, and Paladin of Prahv
  (whose rider rides the new `Effect::GainLifeWhenTargetDealsDamageThisTurn` /
  `DelayedKind::SourceDealsDamageThisTurn`, fired from all three damage paths).
  Also shipped this pass: Karoo bounce-lands (Azorius Chancery / Rakdos Carnarium
  / Simic Growth Chamber), Flaring Flame-Kin, Haazda Shield Mate, Prahv, Jagged
  Poppet, Palliation Accord (`CounterType::Palliation`), Pain Magnification (rode
  a fix binding `event_subject` to the damaged player), Rakdos Augermage, Drekavac
  (`Effect::MayDiscardMatching`), Crypt Champion (`Effect::EachPlayerReanimateCreatureMaxMv`).
- **Dissension/Ravnica gap batch (dis/gaps2, rav/gaps) — shipped:** Protean Hulk
  (`Effect::SearchLibraryCreaturesUpToTotalManaValue`), Swift Silence
  (`Effect::CounterAllOtherSpellsDrawPer`), Lyzolda (`Predicate::SacrificedWasColor`
  off a new `sacrificed_colors` scratch), Stormscale Anarch
  (`Predicate::LastDiscardedWasMulticolored`), and the split cards Crime // Punishment,
  Hit // Run (`SacrificeAndRemember` now surfaces a player-target slot), Rise // Fall
  (`Effect::RevealRandomDiscardNonland`). RAV/GPT: the seven guild bounce-lands,
  Benevolent Ancestor, Carrion Howler, Conclave Phalanx, Dogpile, Dimir Cutpurse,
  Clinging Darkness, Consult the Necrosages.
  **Shipped since (this run):** Azorius Ploy now uses the new
  `Effect::PreventCombatDamageByTargetThisTurn` (outgoing-only combat-damage
  prevention, mirror of `PreventCombatDamageToTargetThisTurn`). Plus a GPT/RAV
  wave — Giant Solifuge, Crystal Seer, Izzet Chronarch, Drowned Rusalka, Crash
  Landing, Hissing Miasma, Agent of Masks, Exhumer Thrull, Benediction of Moons
  (`Value::PlayerCount`), Burning-Tree Shaman/Bloodscale, Culling Sun, Ghostway,
  Glass Golem, Goliath Spider, Grayscaled Gharial, Centaur Safeguard, Greater
  Forgeling, Goblin Fire Fiend, Blazing Archon
  (`StaticEffect::CreaturesCantAttackController`), Sell-Sword Brute, Screeching
  Griffin, Roofstalker Wight, Sewerdreg, Infectious Host, Loxodon Gatekeeper,
  Oathsworn Giant, Moroii, Keening Banshee, Primordial Sage, Junktroller, Ivy
  Dancer, Lore Broker, and the Hunted cycle.
  **Still-deferred RAV/GPT/DIS cards (each on one primitive):**
  **Selesnya Sagittars** (the multi-block keyword it wants now ships —
  `Keyword::CanBlockAdditional`); Sabertooth Alley Cat
  ("creatures without defender can't block this" mass restriction); Drake Familiar
  (ETB "sacrifice unless you return an enchantment to hand" — a
  return-a-permanent-else-sacrifice reflexive cost, the non-mana sibling of
  `Effect::MayPay`); Razia's Purification (each player keeps 3 permanents,
  sacrifices the rest — generalize `EachPlayerKeepsOneSacrificeRest` to a keep-N
  count); "another target creature" spells could use a `DifferentFromTarget(slot)`
  requirement so the two slots can't collapse (Carom/Razia ship with the two-slot
  idiom today but don't enforce distinctness); the complex Magemarks —
  Beastmaster's (becomes-blocked +1/+1-per-blocker rider), Necromancer's
  (return-to-hand death replacement over your enchanted creatures); Living Inferno
  (two-way divided-damage fight); Orzhov Pontiff (`Effect::ChooseMode` covers the
  modal; needs Haunt ETB + haunt-death wiring). (Shipped this run: Indentured Oaf,
  Molten Sentry, Spawnbroker, Spectral Searchlight, Carom + Razia's redirect,
  Shadow of Doubt, Conjurer's Ban, Droning Bureaucrats, Belltower Sphinx via
  `PlayerRef::LastDamagerControllerOf`.)
  **Still-deferred DIS/RAV cards (need new primitives):** Simic Basilisk (grant "destroy at end
  of combat on combat damage to a creature" until EOT); Ignorant Bliss
  (exile hand, delayed return next end step); Kindle the Carnage (repeatable
  random-discard damage loop); Bronze Bombshell (needs a `GameEvent::ControlChanged`
  + `EventKind::ControlChanged` emitted from every control-change site);
  Muse Vessel (needs "may play a card exiled with this source");
  Evolution Vat (grant a counter-doubling activated ability until EOT); Azorius
  Aethermage (needs a "permanent returned to your hand" trigger); Vigean Intuition
  / Fertile Imagination (choose-a-card-type at resolution + type-routed reveal);
  Isperia / Momir Vig / Experiment Kraj (complex legendaries); the remaining split
  cards Trial // Error (return all blocking/blocked-by), Odds // Ends (coin-flip
  counter-or-copy), Research // Development (outside-the-game / repeated may-draw),
  Bound // Determined (return-up-to-colors-of-sacrificed from graveyard). Narrow
  approximation: Stormscale Anarch's discard is chosen-lowest, not random;
  Magewright's Stone's target is any creature; Cytoshape's copy isn't nonlegendary.
- **Dissension gap batch (dis/creatures) — shipped:** Assault Zeppelid, Sky
  Hussar (ETB untap-all), Stalking Vengeance (death→power damage), Azorius Herald
  (unblockable + sac-unless-{U} via `SourceCastWithColorSpent`).
  `Effect::DealDamageEqualToPower` now hits a player/PW target and reads a dead
  `TriggerSource`'s power from its die snapshot (CR 603.10). ~70 Dissension cards
  remain — enumerate with `python3 scripts/set_gaps.py dis`.
  **Noticed engine nit:** a land animated to a basic type via
  `LandsBecomeChosenBasicType` doesn't tap for the new color through
  `GameAction::ActivateAbility { ability_index: 0 }` — the printed mana ability is
  stripped by the `RemoveAllAbilities` layer but the derived intrinsic ability
  isn't surfaced at index 0. Type-line change is correct; mana-ability re-index is
  the gap.
- **Ravnica batches 9–10 (recent299–300) — shipped:** Woodwraith Corrupter
  (`Effect::BecomeCreature` on a *targeted* Forest — permanent land-animation),
  Bond of Agony (`additional_cost_pay_x_life` → each opponent loses X), Enemy of
  the Guildpact, Court Hussar (`LookPickToHand` dig-3), Overrule
  (`CounterUnlessPaid { extra_generic: XFromCost }` + gain X life), Thundersong
  Trumpeter (CantAttack+CantBlock EOT grants), Grozoth (MV-9 `SearchUpToN`,
  `LoseKeywordThisTurn` defender-drop, transmute).
- **Ravnica batches 3–5 (recent293–295) discovered gaps:** aura/equipment-granted
  *step* triggers now fire (`fire_step_triggers` walks `EquipBonus.triggered_abilities`
  — Pillory of the Sleepless). **Still deferred, each on one primitive:**
  Perplex (counter unless its controller discards their *whole* hand — needs a
  `WardCost::DiscardHand` or a dedicated counter-unless-empty-hand effect);
  Terraformer (choose a basic land type, *your* lands become it EOT — no
  "become chosen basic type" over a group); Nettling Curse (an Aura granting an
  *attacks/blocks* trigger — only step-kind equip-bonus triggers are dispatched,
  not combat-kind); Skeletonize (delayed "when a creature dealt damage this way
  dies, make a token").

**Tooling — client build in headless/CI:** the GUI crate needs `libwayland-dev`,
`libasound2-dev`, `libudev-dev`, and `libxkbcommon-dev` to compile (wayland-sys/
alsa-sys/libudev build scripts). A SessionStart hook now `apt-get install`s these
(`.claude/hooks/install-client-deps.sh`) so `cargo build/test -p
crabomination_client` and its unit tests run in web sessions.

**Shipped (recent286 — Class enchantments, CR 716):** the level-up mechanic
(`CardInstance.class_level`, `Effect::AdvanceClassLevel`,
`Predicate::SourceClassLevelIs`/`SourceClassLevelAtLeast`,
`StaticEffect::WhileClassLevelAtLeast`, `StaticEffect::WhileYourTurn` (CR 611.2
turn-gate wrapper), `EventKind`/`GameEvent::ClassLevelReached`,
`Value::OpponentsWithHandSizeAtMost`; server view + client "Lvl N" chip surface
the level). Cards: the Bloomburrow Talent cycle (Stormchaser's / Gossip's /
Hunter's / Scavenger's / Bandit's / **Blacksmith's** / **Builder's** /
**Caretaker's** / **Innkeeper's**) + AFR Wizard / Cleric / Warlock Class.
Innkeeper's L3 counter-doubling is now level-gated (`counter_doublers_for` peels
`WhileClassLevelAtLeast`); its L2 "permanents with counters have ward" works via
counter filters (`R::WithAnyCounter`/`WithCounter`) now honored on the layer
CardMatch path.
**Still open (Class cards):** Artist's Talent
and AFR Paladin/Druid (level-2/3 cost-reduction, damage-replacement,
and extra-land-permission are statics read outside the layer system, so
`WhileClassLevelAtLeast` can't gate them — needs level-gating on the
cost/replacement/land-play paths); AFR Barbarian/Bard/
Sorcerer (dice-roll replacement statics). Ranger/Rogue/Fighter/Monk each have
one cost/permission/play-from-exile level that needs the same level-gating on
non-layer static paths (Rogue L2's "menace anthem" is already a clean
`WhileClassLevelAtLeast` case).

**Shipped (recent283–285, 8 cards + primitives):** `EventKind::GiftGiven`
(spell + permanent gifts; Jolly Gerbils), `Predicate::SourceGiftPromised`
(permanent-gift ETB gate; Scrapshooter, Kitnap), `Value::
DistinctManaValuesInGraveyard` (Aven Heartstabber), `Value::
GreatestPowerControlledAndGraveyard` (Ambitious Dragonborn), `Hamster` type.
Cards: Aven Heartstabber, Ambitious Dragonborn, Jolly Gerbils, Argivian
Cavalier, Scrapshooter, Kitnap, Parting Gust, Starfall Invocation. Bot now
promises gifts (`CastGift` candidates); client recap surfaces "gifts given".

**Discovered — Bloomburrow legends backlog (each needs new engine work):**
Rottenmouth Viper (variable additional-sac cost reduction + per-blight-counter
edict-or-discard), Vren the Relentless (per-turn "creatures exiled under
opponents' control" counter feeding an end-step token generator),
Muerra/Camellia/Wick (first-main mana-per-Raccoon trigger; forage payoffs;
Rat/Snail conditional token), Dragonhawk / The Infamous Cruelclaw (impulse +
delayed "for each still-exiled" damage / discard-to-cast alt-cost),
"Season of …" sorceries (delayed multi-turn modal). Starforged Sword (gift
Equipment) needs the permanent-gift ETB + self-attach. (Class/level-up now
ships — 6 of 7 Talents done, only Artist's Talent remains; see the CR 716 note
above.)

**Shipped (recent235–238, DSK/OTJ gap batch — 20 cards):** manifest-dread now
exposes the manifested creature on `Selector::LastMoved` (Slimy Aquarium, Weight
Room); `Effect::TapAnyNumberThenPumpPerTapped` (Orphans of the Wheat);
`AdditionalCastCost::ExileFromGraveyard` gained a `count` (Abhorrent Oculus);
`Player.spells_cast_from_hand_this_turn` + `Predicate::NoSpellCastFromHandThisTurn`
(Prairie Dog — casts from exile/gy/command don't count); `Effect::
GrantExtraPlusOneCountersThisTurn` (temporary Hardened Scales — Prairie Dog);
`StaticEffect::PumpTeamIf` delirium anthem (The Swarmweaver). Rooms: Surgical
Suite, Underwater Tunnel, Moldering Gym, Greenhouse, Walk-In Closet, Grand
Entryway, Derelict Attic, Funeral Room, Painter's Studio, Ticket Booth,
Restricted Office, Bottomless Pool-half work. Server surfaces
`avg_decisive_turns`/`avg_draw_turns`; client match summary tracks life lost.

**DSK/OTJ still open:** Miasma Demon (discard any number, then it deals damage
to up to that many target creatures — needs `Effect::ApplyToTargets` with a
runtime `Value` max, not a `u8`). (Emergent Haunting, Come Back Wrong, Veteran
Survivor, Getaway Glamer, Trial of Agony, Freestrider Commando all shipped —
recent239/240/274.)

**Shipped (recent267–270 gap batches — 26 cards + primitives):**
`ActivatedAbility.cost_reduction_per_graveyard` ("costs {1} less per [filter]
card in your graveyard" — Battlefield Butcher). Cards on existing primitives:
Akki Scrapchomper, Argothian Opportunist, Ashnod's Intervention, Gnawing
Crescendo, Angelic Intervention, Alabaster Host Intercessor, Aether Channeler,
Aggressive Sabotage, Argivian Phalanx, Artillery Blast, Automatic Librarian,
Antagonize, Attended Socialite, Backup Agent, Angelic Observer, Armor of
Shadows, Arms of Hadar, A Little Chat, Gilded Scuttler, Go Forth, Hearts on
Fire, Hungry Megasloth, Phantasmal Shieldback, Razorgrass Invoker, Black Market
Tycoon, Balduvian Atrocity. Also fixed a **client build break** (CounterType::
Unlock was unhandled in `counter_tooltip.rs`) and added a server
`turn_count_mode_bucket` stat (modal game length).

**Noticed this run (recent267–270), each blocked on one primitive:**
- **Damage-to-you replacement → mill** — Angel of Suffering (prevent damage to
  you, mill twice that many) needs a player-damage replacement hook.
- ✅ ~~**"Second time this ability resolved this turn"**~~ — Harvestrite Host
  ships via `Effect::NthResolutionThisTurn { branches }` over the per-turn
  `GameState.ability_resolutions_this_turn` tally (recent275).

Shipped (recent283): Enlist was already wired (`shortcut::enlist()` — Argivian
Cavalier); `EventKind::GiftGiven` (Jolly Gerbils);
`Value::DistinctManaValuesInGraveyard` (Aven Heartstabber);
`Value::GreatestPowerControlledAndGraveyard` (Ambitious Dragonborn); `Hamster`
creature type.

**Resolved (recent176):** ETB *triggered abilities* now thread the cast's X.
`CardInstance.cast_x_value` is stamped at resolution and the auto-target picker
concretizes `{X}`-from-cost filters via `auto_target_for_effect_avoiding_set_x`,
so a trigger filtered by `ManaValueAtMostXFromCost` (Dune Drifter) picks a legal
target. Shipped: Dune Drifter.

**Discovered during recent214 (each blocked on one primitive):**
- **MayPay body player-targeting** — a triggered `Effect::MayPay { body: Drain {
  from: Player(Target(0)) } }` doesn't declare/auto-target the player slot, so
  the trigger fizzles ("target player loses N"). Kalastria Highborn was modeled
  as each-opponent instead. Fix: recurse into MayPay/MayPayGeneric bodies in
  `auto_targets_for_effect_all_slots` + the target-declaration walk.
- **Per-slot optional target — shipped (recent229).** `Effect::OptionalTargets
  { min, body }` marks slots `>= min` declinable for a `body` whose slots come
  from *distinct* effects (the case `ApplyToTargets` can't express). Ships Primal
  Might (min 1: required pumped creature + optional fight target) and Boom Box
  (min 0: three optional destroy slots). Prayer of Binding and Immersturm
  Predator's exile can now wrap their optional slot the same way.
- **Gate Colossus** — "whenever a Gate you control enters, put this from your
  graveyard on top of your library" is a from-graveyard trigger on a
  non-recursion permanent (`EventScope::FromYourGraveyard` fires the ability but
  the card body isn't a recursion shape); needs generalizing.
- **Drakuseth, Maw of Flames** — "4 damage to any target and 3 to each of up to
  two other targets" needs cross-effect target-slot allocation (single-target
  `DealDamage` slot 0 + `ApplyToTargets` slots 1–2 excluding slot 0).
- **Ordeal cycle / Nine-Lives Familiar** — need a "when you sacrifice this"
  self-trigger and a delayed return-with-counter-decrement at the next end step.
- **Base-toughness anthem shipped** — `StaticEffect::SetBaseToughnessForMatching`
  + `Modification::SetToughness` (layer 7b); Maha, Its Feathers Night ships.
- **Persistent mana — shipped (recent225).** `Effect::AddManaKeptThisTurn` +
  `Player.kept_mana_this_turn` re-seed the pool on every step/phase empty and
  clear at cleanup (CR 500.4/500.5 exception). Ships Savage Ventmaw; reusable for
  other "you don't lose this mana as steps and phases end" riders. Surfaced to
  the client as `PlayerView.kept_mana` (🔒 HUD chip).
- **Also shipped this batch (recent225/226):** `Effect::ReturnSelfTapped`
  (plain self-return-tapped rider — Fake Your Own Death),
  `Value::CreatureCardsMilledThisEffect` (Dread Summons), and
  `StaticEffect::SuppressCreatureEtbTriggers.also_artifacts` (artifact ETB
  suppression — Doorkeeper Thrull). Bot: attack planner now ignores opponents
  with a computed `CantBlock`.
- **Still-open FDN "one primitive each":** Hoarding Dragon (search-to-exile
  linked recursion, below), Desecration Demon (each-opponent-may-sacrifice at
  each combat + reflexive tap/counter on self), Steel Hellkite (per-source
  combat-damage tracking + X-value destroy), Primal Might (optional single fight
  target, below), Nine-Lives Familiar (revival delayed return), Gate Colossus /
  Drakuseth (above).
- **Buildable gap cards still open:** Tumbleweed Rising (X/X token where X =
  greatest power — needs a fixed-at-creation evaluated P/T, not `dynamic_pt`),
  Unscrupulous Contractor / Victimize (reflexive-sacrifice chains that target a
  player / return two gy targets), Fear of Burning Alive (delirium rider),
  Hotshot Investigators ("if you controlled it, investigate" — needs a "you
  controlled the returned target" predicate), Clandestine Meddler
  (suspected-attackers → surveil trigger), Shifting Grift (two-target ExchangeControl
  Spree modes — the spree slot-assignment assumes one target per mode), One Last Job
  (aura-attach-from-graveyard Spree mode). Shipped across recent229–234: Primal
  Might, Boom Box, Out Cold, Prizefight, Harvester of Misery, Krovod Haunch,
  Wickerfolk Thresher, Resilient Roadrunner, Giant Beaver, Ornery Tumblewagg,
  Volcanic Spite, Lilysplash Mentor, Rampaging Soulrager, Haunted Screen, Fear of
  Infinity, Metamorphic Blast, Return the Favor, Trash the Town, Unfortunate
  Accident, Thunder Lasso.
- **Search-to-exile linked recursion** — Hoarding Dragon ("search an artifact,
  exile it; when this dies, return the exiled card to hand") needs the search's
  `ZoneDest::Exile` to stamp `exiled_with = source` so a death trigger can read
  `Selector::CardExiledWithSource`.
- **Supply / Incubation + more counter types — shipped (recent219/220).**
  `CounterType::{Incubation, Revival, Stash, Divinity, Fellowship, Bait, Supply}`
  with client display + tooltip entries. Cards: Drake Hatcher (incubation),
  Stocking the Pantry (supply), Myojin of Night's Reach (divinity). Still on
  those counters but not yet built: Nine-Lives Familiar (revival — needs the
  delayed return below), Tinybones (stash — needs play-from-exile-you-don't-own),
  Banner of Kinship (fellowship — needs choose-creature-type-on-enter + a
  per-counter chosen-type anthem), Fishing Pole (bait — needs an equipped-creature
  "becomes untapped" granted trigger).
- **`CastSpellSharesChosenColorOfSource` shipped (recent221)** — a cast trigger
  gated on the source's `chosen_color` (Diamond Mare). Reusable for other
  chosen-color payoffs.
- **Destroy vs. statically-granted indestructible — fixed.** `Effect::Destroy`
  now reads the computed keyword set (layer-6 grants) instead of the raw
  instance, matching the SBA lethal-damage path (Myojin, Shielded by Faith, …).

**Next-up cards noticed this run (recent185–190), each blocked on one
primitive:**
- **BLB Gift instants/sorceries** — Sazacap's Brew, Dewdrop Cure, and Consumed
  by Greed now ship (`decks::gift`, tests in `tests/modern.rs`). Remaining:
  **Cruelclaw's Heist** — reveal opponent's hand, exile a chosen nonland card,
  and (if gifted) may cast it from exile paying any type of mana. Needs a
  hand-reveal-choose-exile + grant-may-play-from-exile-any-color primitive
  (sibling of `ExileTopAndGrantMayPlay`, but sourced from an opponent's hand).
- **Say Its Name** (DSK) — front half (mill 3 + gy recur) is clean; the
  activated tutor exiles three same-named gy copies to fetch Altanak — a
  named-card gy-exile cost.

**Card-gap tooling:** `scripts/set_diff.py <setcode> [--oracle]` diffs a
Scryfall set against the catalog (substring match on quoted names, robust to
helper-factory positional names) and lists genuinely-missing cards. Current
counts: TDM 17 (mostly hard multicolor legends/enchantments — Ugin, Shiko,
Kotis, Jeskai Revelation, Call the Spirit Dragons…), DFT 34, DSK ~79, BLB ~71,
FDN ~110 remaining after `recent206`/`recent207`. **Easy FDN leftovers** (all
existing primitives): `recent209`–`recent213` cleared ~46 (River's Rebuke
`Selector::ControlledBy { who: PlayerRef::Target(0) }` mass-bounce, Ancestor
Dragon, Fynn deathtouch→poison, Guildgate cycle, Heraldic Banner, Wildwood
Scourge, Ajani, Aurelia, …). **Still open:** Biogenic Upgrade (distribute +
double counters at resolution), Demonic Pact (rotating "choose one not chosen"
modal upkeep — needs per-source chosen-mode exclusion), Dread Summons (mill-X →
Zombie-per-milled-creature — needs a "token per creature card milled" count),
Desecration Demon (opponent-may-sac-to-tap — Punisher heuristic over-sacs, so
deferred), Hoarding Dragon (ETB tutor-to-exile linked to a death-return),
Steel Hellkite (per-source combat-damage tracking), Painful Quandary (ships,
but the Punisher heuristic always discards rather than choosing the 5-life
loss). Test-helper note: `move_card_to_battlefield_for_test` doesn't dispatch
*other* permanents' ETB triggers (watchers like Lathliss) — cast the entrant to
exercise those.

**DFT gaps (recent168–174 shipped ~40 cards). Remaining, each needing one
primitive or a heavier build:**
- **Demonic Junker** — ETB "for each player, destroy up to one target creature
  that player controls" needs a per-player multi-target destroy with a
  "creature-you-controlled-was-destroyed" rider.
- **Ancient Vendetta** — name-a-card then exile up to four copies from an
  opponent's graveyard *and* hand *and* library (a cross-zone name-exile).
- **Skyserpent Seeker** — exhaust "reveal until two land cards, put them onto
  the battlefield tapped, rest to the bottom in a random order."
- **Cursecloth Wrappings / Wickerfolk Indomitable** — grant-embalm-to-a-gy-card
  and graveyard-cast-with-additional-cost respectively.
- **Tyrant cycle (Sundial/Tyrox/Kalakscion/Terrian)** — Scryfall oracle text is
  genuinely empty on Scryfall too (confirmed via a live `cards/named` fetch,
  2026-07-12 — incomplete Aetherdrift preview data). Not implementable until
  Scryfall backfills the rules text; keep skipping.
- **Wickerfolk Indomitable** — cast from graveyard by paying 2 life + sacrificing
  an artifact/creature (an Escape-like graveyard-cast permission with arbitrary
  additional costs — generalize `Keyword::Escape` to non-exile costs).
- **Rise from the Wreck** — return up to one each of four *distinctly-filtered*
  graveyard cards to hand (creature / Mount / Vehicle / no-abilities creature);
  needs a multi-slot up-to-one graveyard return (each `ApplyToTargets` currently
  owns only slot 0).
- **Push the Limit** — return *all* Mount/Vehicle cards from your graveyard to
  the battlefield, sacrifice them at the next end step, animate your Vehicles +
  team haste; needs a "return-all-matching-from-graveyard" effect plus a
  delayed sacrifice of the moved set (`Selector::LastMoved` doesn't survive to
  the end step).
- **Skyseer's Chariot** — name-a-card on ETB + "activated abilities of sources
  with the chosen name cost {2} more" (an activated-ability *tax* keyed on a
  chosen name; the `OtherExhaustActivationCostReduction` sibling shipped for
  Boom Scholar, but the named-tax + name-on-enter halves are still open).

See `CUBE_FEATURES.md` (cube-card implementation status),
`STRIXHAVEN2.md` (Secrets-of-Strixhaven status), and `FEATURE_ROADMAP.md`
(prioritized engine functionality). **The correctness-audit section below
outranks everything else in this file** — its P0 tier is game-deciding or
state-corrupting in ordinary play.

## Theros block — BNG and JOU COMPLETE

`sets::bng`/`bng2`/`bng3` ship all 165 BNG cards; `sets::jou`/`jou2`/`jou3`
ship all of JOU but **Worst Fears**, which needs a real
"you control target player during their next turn" primitive (decision +
priority routing for another seat — the Mindslaver shape). That's the only
`set_gaps.py jou` entry left.

Residual BNG approximations, each wanting one small primitive:
- **Satyr Firedancer** — `EventKind::YourInstantOrSorceryDealtDamage` doesn't
  carry *which* player was hit, so "target creature that player controls"
  reads as "target creature an opponent controls".
- **Perplexing Chimera** — the exchange ships, but the printed "you may choose
  new targets for the spell" rider is dropped.
- **Champion of Stray Souls** — "return X TARGET creature cards" is modelled as
  a resolution-time `MoveChosen` pick, because `Effect::ApplyToTargets` inside
  an *activated ability* isn't collected at activation time (the cast path does
  collect it). Fixing that would also un-approximate future X-target
  activations.
- **Whims of the Fates** — the three piles are a shuffled round-robin split, not
  a player-chosen one.

Residual JOU approximations:
- **Godsend**'s exile trigger fires on "blocks", not on "blocks or becomes
  blocked", and auto-picks which combat partner to exile.
- **Battlefield Thaumaturge** discounts {1} when the spell targets *a*
  creature; the printed "for each creature it targets" needs the full
  cast-time target list at cost time (`cost_reduction_for_spell` sees slot 0
  only).
- **Silence the Believers** exiles the creatures but not the Auras attached to
  them (they hit the graveyard as an SBA instead).

## TDM (Tarkir: Dragonstorm) gaps — good easy-card source

`decks::tdm` shipped ~55 cards (Siege cycle, Abzan Monument, Breaching
Dragonstorm, Dragonstorm Forecaster, Hundred-Battle Veteran, Anafenza, Felothar,
Lotuslight Dancers, Eshki, Narset, Revival of the Ancestors, Kishla Village,
Dracogenesis, Death Begets Life, Herd Heirloom, Yathan Roadwatcher, Great
Arashin City among them). Shipped since: **Nature's Rhythm** (X-search-to-
battlefield + Harmonize, which is already a keyword), **Smile at Death** (upkeep
up-to-two graveyard reanimate via `ApplyToTargets`), **Roar of Endless Song**
(Saga — Elephants then a team P/T double via `ForEach` + `Value::PowerOf/
ToughnessOf`). Shipped since: **Windcrag Siege** (Mardu = new
`StaticEffect::DoubleControllerAttackTriggers` Isshin-style doubler; Jeskai =
upkeep Goblin), **United Battlefront** (existing `LookTopPutMatchingOntoBattlefield`,
max 2), **Static Snare** cost rider (`affinity_filter` now counts attacking
creatures — `evaluate_requirement_on_card` learned `R::IsAttacking`),
**Maelstrom of the Spirit Dragon** (new `SpendRestriction::DragonOrOmenSpell`
+ `SpellKind.omen`, threaded through `cast_omen`), and **Neriv, Heart of the
Storm** (new `StaticEffect::DoubleDamageFromCreaturesEnteredThisTurn` in
`scale_damage_to`). Reverberating Summons, Stillness in Motion, Rite of Renewal and Dalkovan
Encampment shipped in `decks::recent325`. `set_gaps.py tdm` is down to 10, all
of them multicolour rares/mythics (Call the Spirit Dragons, Jeskai Revelation,
Kotis, Mardu Siegebreaker, New Way Forward, Shiko, Taigam, Thunder of Unity,
Ugin Eye of the Storms) plus **Sidisi, Regent of the Mire**, which still needs
a sacrificed-MV → target-MV+1 reanimate link. (The Sibsig Ceremony shipped via
`Predicate::TriggerSourceEnteredByCast` + `CardInstance.entered_by_cast`.)

## Recent-set gaps (BLB / DSK / FDN) — good easy-card source

`scripts/set_gaps.py {blb,dsk,fdn}` still lists simple commons/uncommons, but
many are already implemented elsewhere in the catalog — **grep before adding**.
`decks::recent117-122` shipped several passes (threshold, landfall, first-lifegain,
kicker, changeling gy-tuck, gy-recursion, reveal-until-land ramp, begin-combat
pump, modal flash, punisher Aura, delirium fight/reanimate, delirium-discount
removal, surveil, Fact-or-Fiction). All four cards this section used to list as open (Come Back Wrong, Cathartic
Parting, Darkstar Augur, Feed the Cycle) shipped in later runs, as did
Corpseberry Cultivator via `EventKind::Foraged`. `decks::recent325` added
Miasma Demon, Stay Hidden Stay Silent, Chainsaw, Dissection Tools, Unidentified
Hovership, Leyline of Mutation, Silent Hallcreeper, Thornvault Forager,
Hoarder's Overflow, Festival of Embers and Camellia. What's left in DSK/BLB is
listed under "Noticed this run (recent325 gap batch)".

**OTJ (`decks::recent124-126`)** is down to **10** (2026-08-05 `set_gaps.py
otj`), all of them build-around legends: Another Round, Assimilation Aegis,
Breeches, Calamity, Eriette the Beguiler, Kellan the Kid, Lilah, Riku of Many
Paths, Taii Wakeen, The Gitrog Ravenous Ride. Next-up primitives to unblock
them: a "committed a crime this turn" gate on
activated abilities (Blood Hustler), and Spree-on-instant modal costs
(Metamorphic Blast, Getaway Glamer). (`Value::OtherSpellsCastThisTurn` shipped —
Thunder Salvo now uses it directly.)

`decks::recent149-152` shipped ~27 BLB/DSK/OTJ/WOE cards (modal removal, surveil-
riders, conditional threaten/exile-until-leaves, impulse Plotter, manifest-dread
Equipment, mana dorks, saddled Mount, Delirium self-reanimate, finality reanimator
+ Flashback, graveyard recursion, X-token maker, prowess Otter) plus the new
`Effect::ChooseNumberDestroyByPower` (Expel the Interlopers). Cards deliberately
skipped this run, each needing one primitive:
- **Gnawing Crescendo / Mardu-style "this turn when a creature dies"** — a
  duration-scoped delayed *triggered* ability granted to the player.
- **Spree** ships (`Effect::Spree` + `CastSpellSpree`; Jailbreak Scheme, Final
  Showdown, and the `decks::spree` cycle done). Remaining Spree cards each want a
  bespoke mode effect: Getaway Glamer (destroy-if-no-greater-power), Betrayer's
  Bargain (sac-or-pay choice), Lively Dirge (return-up-to-two-total-MV≤4),
  Great Train Heist (extra-combat + delayed treasure-on-damage).
- **DSK Rooms / doors** — `Predicate::UnlockedDoorsControlledAtLeast` (CR 709.5)
  counts unlocked doors among Rooms you control; ships Rampaging Soulrager
  (`PumpSelfIf` +3/+0). Remaining: Keys to the House's door mode, lock-a-door.
- **BLB Gift / Valiant / Offspring / Expend** ability words (Jolly Gerbils,
  Flowerfoot Swordmaster, Junkblade Bruiser, the Gift spells).
- **"Becomes the target of an opponent's spell/ability"** (Cactarantula).
  ("Becomes plotted" ✅ — `EventKind::BecomesPlotted`, a SelfSource self-trigger
  dispatched by `plot_card`; Aloe Alchemist + Longhorn Sharpshooter shipped.)
- **Delirium widens a modal to choose-one-or-more** (Let's Play a Game).

`decks::recent177-178` shipped ~14 FDN/BLB/DSK cards (Exemplar of Light,
Ashroot Animist, Arahbo, Bumbleflower's Sharepot, Celestial Armor, Strix
Lookout, Vanguard Seraph, Vampire Soulcaller, Turn Inside Out, Huskburster
Swarm, Marching Duodrone, Fiendish Panda, Quick-Draw Katana, Salvation Swan).
Documented per-card approximations: Huskburster's affinity drops the exiled-
creature half; Quick-Draw Katana's +2/+0 is always-on (only first strike is
turn-gated); Salvation Swan drops the returned creature's flying counter and
models "up to one target" as a single target. Skipped, each needing a primitive:
- **Coordinated Clobbering** — tap 1–2 of your creatures, each deals its power to
  one shared opponent's creature (needs a two-independent-slot fight).

`decks::recent183-184` shipped ~7 OTJ cards (Ferocification, Freestrider Lookout,
Fleeting Reflection, Full Steam Ahead, Hellspur Posse Boss, Kraum, At Knifepoint)
on existing primitives (begin-combat modal, crime dig, become-a-copy trick, team
pump, outlaw-haste/first-strike lords via `StaticEffect::GrantKeyword` + `R::IsOutlaw`,
Flurry). Approximations: Fleeting Reflection's copy target modeled as required;
At Knifepoint's first strike always-on (not "during your turn"). Still-open OTJ:
Boom Box (three independent up-to-one destroy slots), Emergent Haunting (spell-gated
self-animate), Hollow Marauder (per-opponent-discard conditional draw).

`decks::recent179-182` shipped ~22 FDN/DSK/BLB/TDM cards (Songcrafter Mage,
Twinblade Blessing, Tragic Banshee, Midnight Snack, Uncharted Voyage, Raise the
Past, Sylvan Scavenging, Ravenous Amulet, Zul Ashur, Twinflame Tyrant, High Fae
Trickster, Electroduplicate, Fear of Falling, Possessed Goat, Hired Claw,
Mistbreath Elder, Plumecreed Mentor, Azure Beastbinder, Byrke, Dreamdew Entrancer,
Finneas, Gev) on the new `GrantHarmonizeThisTurn` / `activate_once` /
`BecomeColor.additive` primitives plus existing ones. Documented approximations:
Fear of Falling's debuff modeled until-end-of-turn (not "until your next turn");
Mistbreath Elder drops the "otherwise return this" fallback; Gev omits the
enters-with-extra-counters static (no primitive yet). Still open, each needing a
primitive:
- **Gev's enters-with-extra-counters** — a static granting your creatures extra
  ETB +1/+1 counters scaled by a `Value` (opponents who lost life this turn).

## Final Fantasy (`sets::fin`) — COMPLETE

Every single-faced FIN card is implemented (`python3 scripts/fin_gaps.py`
reports 0 missing), including the Vanille ↔ Fang meld into Ragnarok
(`Effect::Meld`). Remaining known approximations, each documented on its
factory doc comment:
- Zidane (opponent-gains-control → Treasure rider dropped), Y'shtola
  (lost-4-life end-step draw omitted), Rydia (Summon-Saga reanimation
  activated ability omitted), Squall (creature-*spell*-targeted half dropped).
- Gogo's "this ability can't be copied" rider unmodeled; copies keep targets.
- Memories Returning / Choco / Gilgamesh picks are auto-heuristics (no
  interactive multi-pick UI yet).
- Stolen Uniform's lose-control unattach is modeled as a next-end-step
  unattach.
- Bahamut/Dion meld pair still wants a second `Effect::Meld` card wiring.

## Noticed this run (Mirrodin block completion)

**Darksteel is done** (`scripts/set_gaps.py dst` = 0) — every card that was
blocked on a primitive shipped with that primitive. Mirrodin is at 96 (was
194).

- **Mirrodin remainder, grouped by the primitive each still wants:**
  - **Non-mana Entwine costs.** `Keyword::Entwine` only carries a `ManaCost`,
    so "Entwine—Sacrifice two/three lands" can't be expressed: Solar Tide,
    Betrayal of Flesh. (The seven mana-entwine modals all ship.)
  - **Non-mana Equip costs.** `Keyword::Equip` is likewise mana-only, so
    "Equip—Pay 3 life" (Nightmare Lash) has no home.
  - **Imprint payoffs beyond keyword theft.** Mirror Golem (protection from
    each of the exiled card's card types), Mourner's Shield (colour-matched
    prevention), Extraplanar Lens (name-matched land doubling), Soul Foundry
    (token copy of the exiled card), Spellweaver Helix (name-matched free
    copy of the *other* exiled card).
  - **"Can't attack or block unless you pay N per counter"** — Myr Prototype.
  - **Reveal-and-guess** — Liar's Pendulum needs a name guess by an opponent.
  - **Control-another-player** (CR 723) — Mindslaver.
  - **Global type-changing** — March of the Machines (every noncreature
    artifact becomes an MV/MV artifact creature), Quicksilver Fountain (flood
    counters retype lands), Shared Fate (draw replacement into an opponent's
    library).
  - **Whole-board wipe with an exception** — Worldslayer.
  - Smaller ones: Confusion in the Ranks (shared-type control exchange),
    Fatespinner (opponent skips a chosen step), Psychogenic Probe (shuffle
    watcher), Scythe of the Wretched (damaged-creature reanimation +
    re-attach), Timesifter (highest-MV extra turns), Proteus Staff
    (bottom-then-reveal-until-creature), Power Conduit (counter shuffling),
    Vulshok Battlemaster (attach every Equipment on entry).
- **`add_card_to_battlefield` fires no ETB triggers**, by design. Several
  batch-1 tests had to cast the permanent instead. A
  `add_card_to_battlefield_entering` fixture that dispatches the entry
  triggers would make ETB tests much shorter.
- **`Effect::MayDo` inside a triggered ability consumes a scripted answer
  before any nested `CastWithoutPayingImmediate` ask**, so a Panoptic-Mirror-
  shaped test needs two `Bool(true)`s. Worth collapsing the double ask when
  the inner effect is itself optional.

## Noticed this run (OTJ gap batch, `decks::recent309`)

- **Cross-permanent death-trigger filters need the cast/SBA path.** A
  `CreatureDied` / `AnotherOfYours` watcher with a `HasSupertype(Legendary)`
  filter (Rakdos Joins Up) only fires when the creature dies through the
  damage → SBA → dispatch cycle; `remove_from_battlefield_to_graveyard_raw`
  and a bare `check_state_based_actions` after a manual damage poke don't
  reach it. Worth making the raw removal path dispatch too so tests and
  effect-driven sacrifices behave alike.
- **`Effect::CastWithoutPayingImmediate { copy: true }` ignores
  `exile_after`.** Kaervek, the Punisher's "exile … and copy it" leaves the
  original in the graveyard.
- **`EventKind::CommittedCrime` can fire twice for one crime** (Kaervek pays
  4 life for a single targeted burn spell). Suspect a double dispatch between
  target announce and resolution.
- **Equip-granted triggers can't read `Value::TriggerEventAmount` reliably** —
  they do get `event_amount` from combat, but `LookTopExileOneMayPlay` needed a
  `who` field before The Key to the Vault could dig its own library. Audit the
  other Gonti-shaped effects for the same hardcoded-opponent assumption.
- **OTJ still-open gaps** (each blocked on one primitive): Lilah, Undefeated
  Slickshot (a granted "exile this spell as it resolves; it becomes plotted"
  replacement), Kellan the Kid (a cast-from-a-zone-other-than-hand trigger),
  Another Round (a repeatable blink-all-your-creatures effect), Riku of Many
  Paths (a modes-chosen count), Calamity / The Gitrog (saddler-scoped
  copy/sacrifice bodies), Eriette (aura-attach control steal), Breeches
  (second-spell coin-flip copy), Assimilation Aegis (exile-until-leaves plus a
  becomes-a-copy-while-attached link), Taii Wakeen (a lethal-noncombat-damage
  event and a this-turn damage boost), Ertha Jo's activated-ability copier.

## Phyrexia: All Will Be One (`sets::one`) — COMPLETE

Every single-faced ONE card is implemented (`python3 scripts/set_gaps.py one`
reports 0 missing); tests in `crabomination/src/tests/one.rs`. Primitives
shipped across the runs: CR 702.150 Compleated (+ `{A/B/P}` PhyrexianHybrid
pips), CR 602.5g summoning-sickness activation gate, death-trigger doubling
(Drivnod), `EventKind::{PoisonAdded, BecameAttached}`, oil-activity turn
flags, per-counter cost reduction, prevention-with-mite-mint shields,
graveyard-lands ability borrowing, `Effect::BecomeTreasure`, loyalty-ability
grants (Ichormoon), and the CR 603.4 combat-damage intervening-if fix.
Remaining known approximations (each noted on its factory doc):
- Capricious Hellraiser exiles the top three graveyard cards (not random) and
  free-casts the original, not a copy.
- Rhuk's dies-half, Ria Ivor's target choice (auto-picks your biggest
  creature), Phyrexian Atlas multiplayer scoping, Nahiri 0's loyalty MV cap,
  Kaito's per-dealer (unbatched) bounce, Monument's name-count filter,
  Encroaching Mycosynth's off-battlefield halves, Green Sun's Twilight's
  one-per-type pick.

## Discovered follow-ups — EOE/TLA/DFT (`decks::recent166`)

- **Graveyard-functioning triggered abilities.** Wolfbat ("whenever you draw
  your second card each turn, return this from your graveyard…") and the
  Bloodghast/Prized-Amalgam/Vengevine family need triggered abilities that fire
  while the source is in the graveyard. The trigger walk only scans
  `self.battlefield`; add a graveyard scan gated by a per-ability
  `functions_from_graveyard` flag. Wolfbat deferred pending this.
- **Dual-zone tutor + `NoAbilities`/`Vanilla` filter.** Fang-Druid Summoner
  searches library *and/or* graveyard for a creature card with no abilities;
  modeled as library-only, filter dropped. Also blocks Delivery Moogle.
- **Restricted-mana variants.** White Lotus Hideout drops the Shrine half of
  "Lesson or Shrine"; Jasmine Dragon Tea Shop approximates "Ally spell/ability"
  as `CreatureOfType(Ally)`. Add `LessonOrShrineSpellsOnly` /
  `AllySpellsOrAbilities`.
- **Approximations dropped this batch (noted in factory docs):** Firebender
  Ascension's quest-copy; Ragost's "artifacts are Foods"; Secret Tunnel's
  two-shared-type unblockable; Basri's exert (plain tap); Grim Javelineer's
  death-gated surveil; Far Fortune / Hazoret max-speed riders; Coalstoke /
  Sothera / Requiem Monolith not yet added (delayed-exile reanimate, exile-edict,
  damage-triggered granted ability).

## Discovered follow-ups — Marvel's Spider-Man (`decks::spm`)

- **7 remaining EOE cards** still need primitives: Chorale of the Void
  (attack-reanimate from defender's graveyard + Void end-step sac), Famished
  Worldsire (Devour land), Lightstall Inquisitor (opponent exile-a-card +
  may-play w/ +{1} tax), Moonlit Meditation (token-creation replaced by copies
  of the enchanted permanent), Requiem Monolith (grant "damage → draw+lose"),
  Sothera the Supervoid (dies-edict + delayed-exile reanimate), The Dominion
  Bracelet (control an opponent's turn).
- **SPM DFCs / Sagas** not yet added: Peter Parker // Amazing Spider-Man, Miles
  Morales, Norman Osborn // Green Goblin, Gwen Stacy // Ghost-Spider (transform
  DFCs), Origin of Spider-Man (Saga), Spider-Punk (uncounterable + damage-can't-
  be-prevented team static), Spider-Man 2099 ("can't cast on turns 1–3").
- **Small SPM approximations shipped** (upgrade later): Pumpkin Bombardment
  models "discard a card or pay {2}" as a mandatory discard (needs an
  `AdditionalCastCost` OR-branch); Selfless Police Captain moves a single
  +1/+1 rather than its live counter count on LTB (needs LKI counter read);
  Cheering Crowd drops the "add {C} per counter" ramp rider; Spider-Ham drops
  the Animal May-Ham menagerie anthem.
- **Connive test harness.** Mob Lookout's targeted connive works in-game but a
  `cr_rules` conformance test needs the ETB-target vs. discard decision order
  pinned in a `ScriptedDecider` (dropped this run; replaced with CR 601.2f).

## Discovered follow-ups — AFR venture batch (`decks::afr`)

- Rooms resolve inline (no stack round-trip); Tomb's two pay-or-lose rooms are
  flat life loss; Mad Wizard's Lair free-cast collapsed to the draws.
- Ellywick −2 drops the "if it's legendary, gain 3 life" rider; her emblem
  approximates "+2/+2 per differently named dungeon" as a flat +2/+2 while ≥1
  completed. Emblem `PumpTeamIf` statics now evaluate live (new in this run).
- Skipped (need primitives): Find the Path (aura granting the host land a mana
  ability), Thieves' Tools (host-power-gated unblockable — `ConditionalEquipBonus`
  covers it; just not implemented), Midnight Pathlighter (once-per-batch
  "one or more creatures deal combat damage" venture), Hama Pashar (room
  abilities trigger an additional time).

## Discovered follow-ups — Kamigawa: Neon Dynasty (`decks::recent95`–`recent101`)

- **Prosperous Thief** — the printed trigger fires off any Ninja/Rogue you
  control dealing combat damage; modeled as this creature's own combat damage.
- **Jukai Preserver channel** — "up to two target creatures" modeled as a single
  target (no up-to-N multi-target AddCounter yet).
- **Explosive Entry** — the printed "up to one" on each target is modeled as
  required targets (no per-slot optional-target marker yet).
- **Blade of the Oni** — grants the Demon type via `add_creature_types` but the
  "black in addition to its colors" clause overwrites colors (no add-color rider).
- **Blossom Prancer** (not yet added) — the "if you didn't put a card into hand,
  gain 4 life" branch needs a `LookPickToHand`-with-else conditional.
- **Still needing primitives:** Weaver of Harmony (copy an activated/triggered
  ability from an enchantment source), Kami of Bamboo Groves' Conjure, Careful
  Cultivation's conditional Aura rider, Kami of Industry (reanimate-with-haste-
  then-sac-EOT from a graveyard target), Ninja's Kunai (Equipment granting an
  activated ability whose cost sacrifices *the Equipment*), Isshin
  (attack-caused triggers fire an extra time), Kotose (exile-all-copies-by-name),
  Kosei (conditional granted combat-damage ability), Tatsunari (named-token
  gating + unblockable-except-flying), the NEO transform DFC Sagas (Fable,
  Michiko's Reign, …), and the Kaito planeswalkers.
- **recent100 batch (18 new cards):** Golden-Tail Trainer
  (`StaticEffect::CostReductionBySourcePower`), Walking Skyscraper +
  Sky-Blessed Samurai (`SelfCostReducedPerPermanentMatching` now evaluates
  board-state filters like `IsModified` via `evaluate_requirement_static`),
  Traproot Kami (`DynamicPt::ForestsInPlay`), Risona
  (`EventKind::ControllerDealtCombatDamage` — "whenever combat damage is dealt
  to you"), plus Kami of Terrible Secrets, Bamboo Grove Archer, Master's Rebuke,
  Tempered in Solitude, Akki Ember-Keeper, Thundering Raiju, Scrapyard
  Steelbreaker, Atsushi, Junji, Chishiro, Unstoppable Ogre, You Are Already
  Dead. Selfless Samurai's "another" clause and Moon-Circuit Hacker's
  discard-unless-entered rider are now wired.
- **recent101 batch (9 new cards):** Coiling Stalker, Sunblade Samurai, Moonsnare
  Specialist, Undercity Scrounger (`ActivatedAbility.condition` on
  `CreaturesDiedThisTurnTotalAtLeast`), Season of Renewal, Assassin's Ink
  (stacked `SelfCostReducedIfControlEach`), Mnemonic Sphere, Suit Up, Careful
  Consideration (self-cast approximation of "target player draws four…", with the
  main-phase discard discount honored via `CurrentStepIs`).

## Discovered follow-ups — Equipment / Voltron (`decks::recent94`)

- **Stonehewer Giant / Nazahn ETB tutor auto-attach.** Both search an Equipment
  onto the battlefield/hand but drop the "attach it to a creature you control"
  rider (the searched card isn't a target, so it lands unattached). Needs a
  `Search`-then-`Attach` variant that threads the found card into a follow-up
  attach.
- **Grafted Wargear unattach → sacrifice.** The "whenever this becomes unattached,
  sacrifice that permanent" rider is dropped — no unattach-event trigger yet.
- **Nazahn "Hammer of Nazahn to battlefield" branch.** Modeled as a plain tutor to
  hand; the named-card-to-battlefield special case is elided.
- **O-Naginata attach restriction.** "Attach only to a creature with power 3+" is
  dropped (no equip-target power gate).
- **Bigger Voltron cards not yet done:** Halvar, God of Battle // Sword of the
  Realms (DFC God + combat move-an-Aura/Equipment + Equipment back face),
  Ardenn (attach any number at combat), Champion of Lambholt (global
  "power-less creatures can't block yours" static), Bruenor Battlehammer
  (per-attached-Equipment team pump + first-equip-free), Armored Skyhunter
  (look-top-6, put an Aura/Equipment onto the battlefield + attach).
- **Client compile-verify.** `crabomination_client` can't build in the headless
  cloud env (`wayland-sys`); the `attached_to_name` tooltip line was reviewed by
  hand — re-verify on a GUI host.

## Discovered follow-ups — experience + Izzet legends (`decks::experience`, `decks::recent91`)

Shipped: the experience-counter framework (`Player.experience`,
`Effect::AddExperience`, `Value::ControllerExperience`,
`StaticEffect::CostReductionPerControllerExperience`,
`DynamicPt::ControllerExperience`, `PlayerView.experience`) with Mizzix, Ezuri
Claw of Progress, Daxos, Kalemne. Izzet legends (Kykar, Niv-Mizzet Parun, The
Locust God, Izzet Guildmage, Veyran, Charmbreaker Devils, Pyromancer Ascension)
on `Effect::ReturnRandomFromGraveyard` +
`SelectionRequirement::SharesNameWithControllerGraveyardCard`. Engine
improvement: `evaluate_requirement_static` now resolves stack-spell targets for
control/ownership filters, so an **activated** "copy target spell you control"
ability validates its target (Izzet Guildmage).

Batch 3–4 (`decks::recent92`, `recent93`): Firemind Vessel, Thousand-Year
Storm, Swarm Intelligence, Mirari, Niv-Mizzet Dracogenius, Jhoira, Arjun,
Electrodominance, Galecaster Colossus, Gadwick, Sphinx of Lost Truths, Rielle.

Still deferred / noticed:
- **Rielle's** first-discard-each-turn "draw that many" trigger, **Arjun's**
  hand→library-bottom (modeled as discard), **Firemind Vessel's**
  different-colors constraint, and **Niv-Mizzet Dracogenius's** damage-to-a-
  player restriction (fires on any damage) are all approximated.
- **Ral, Storm Conduit / Saheeli, the Gifted / Jaya Ballard** (planeswalkers),
  **Sea Gate Stormcaller** (delayed copy-next-spell), **Goblin Dark-Dwellers /
  Finale of Promise** (free cast-from-graveyard), **Curious Homunculus** (DFC
  transform on 3+ I/S in gy) remain — each needs a non-trivial primitive.
- **Melek, Izzet Paragon** — needs a "cast from library" provenance so the
  copy trigger fires only for top-of-library I/S casts (top-reveal +
  cast-from-top statics already exist).
- **Thousand-Year Storm** — "copy for each other I/S cast before it this turn"
  wants a `Value` counting I/S casts this turn (only all-spell `StormCount`
  exists).
- **Veyran / Mizzix runaway gate / Daxos live token** — Veyran's magecraft
  trigger-doubling half, Mizzix's "mv > experience" gate, and a live-CDA token
  P/T are all approximated (see the card docs).
- **Meren of Clan Nel Toth** — end-step "reanimate if mv ≤ experience, else to
  hand" wants a targeted graveyard-reanimate with a per-target mv branch.

## Discovered follow-ups — Izzet spells-matter sweep (`decks::recent90`)

Shipped: 40 cards (the Izzet spells-matter core — Adeliz, Balmor, Bloodwater
Entity, Improbable Alliance, Runaway Steam-Kin, Harmonic Prodigy, Spellheart
Chimera, Roil Eruption, Dualcaster Mage, Naru Meha, Docent of Perfection,
Beacon Bolt, Archaeomancer, Magmatic Insight, Niv-Mizzet the Firemind, Cinder
Pyromancer, Mystic Retrieval, Deprive, Cerebral Vortex, Chandra's Spitfire —
plus classic-frame filler: Cloud Sprite/Cloud Pirates/Skywinder Drake,
Cinder Elemental, Living Lightning, Needle Drop, Rise from the Tides, Storm
Fleet Aerialist, Flamewave Invoker, Goblin Taskmaster, Fireslinger, Orcish
Cannoneers, Jackal Pup, Rummaging Goblin, Dwarven Trader, Peel from Reality,
Consume Spirit, Vessel of Nascency, Ridgetop Raptor, Warden of Evos Isle).
New primitives: `DoubleControllerTriggersOfType`,
`PlayerDealtNoncombatDamage` event + `DamageDealt.combat`,
`AdditionalCastCost::Discard.filter`; `PermanentView.dealt_damage_this_turn`
for precise client targeting; server `damage_wins` stat.

Still deferred / noticed but not tackled:
- The experience framework ships the full Commander 2015 cycle
  (`decks::experience`: Mizzix, Ezuri Claw of Progress, Daxos, Kalemne, **Meren
  of Clan Nel Toth** — end-step conditional reanimate-or-hand on
  `Effect::If`/`ManaValueOf`/`ControllerExperience`, enabled by
  `R::OwnedByYou` now resolving any-zone cards). Mizzix's runaway gate ("mv >
  your experience") is approximated as any I/S cast; Daxos's token P/T is a
  mint-time snapshot rather than a live CDA.
- **Zada, Hedron Grinder / Wort, the Raidmother** — Zada needs
  "copy target spell for each other creature it could target, each copy a
  different target"; Wort needs a "your R/G I/S spells have Conspire" granted
  static (the `Conspire` keyword itself already ships).
- **Deputy of Detention** — `ExileUntilSourceLeaves` doesn't do the "and all
  other permanents that player controls with the same name" sweep (Detention
  Sphere is modeled the same single-target way).
- **Doubled ETB triggers pick the same target** — two fires of a single-target
  ETB (e.g. an Archaeomancer doubled by Harmonic Prodigy) auto-target the same
  graveyard card, so the second fizzles. The trigger auto-targeter should avoid
  targets already claimed by sibling copies in the same batch.
- **Charmbreaker Devils** — needs a "return a random `[filter]` card from your
  graveyard to hand" effect (no random-graveyard-return primitive yet).

## Discovered follow-ups — retro commons sweep (`decks::recent71`–`recent78`)

Shipped (recent77–78): 43 classic-frame cards; `PumpPT` statics now
live-recompute over `IsAttacking`/`IsModified` (Orcish Oriflamme); `AttachedTo`
LKI now consults `leaves_bf_lki` so a `sac_cost` ability reads the enchanted
creature (Carapace); the bot's block heuristic folds Rampage (CR 702.23) into its
gang-block / second-blocker math; the client keyword strip surfaces the N on
count-scaling keywords (Rmp2 / Tox3 / Ann2); `Predicate::ActivePlayerControls`
gates "at the upkeep of enchanted [permanent]'s controller" Aura pings (Warp
Artifact / Cursed Land / Wanderlust).

Still deferred / noticed but not tackled (recent77–78):
- **Meekstone / Winter Orb family** — "creatures with power ≥3 don't untap"
  needs a filtered untap-skip static (only `LandsDontUntapNextUntapStep` exists).
- **Dragon Whelp** — firebreathing with "if activated 4+ times this turn,
  sacrifice at next end step" needs a per-turn activation counter + self-sac.
- **Balduvian Horde / Bull Elephant** — ETB "sacrifice unless you {discard at
  random / return two Forests}" needs an ETB sacrifice-unless-alt-cost rider.
- **Kormus Bell / Living Lands** — animate-all-lands-of-type continuous effect.
- **Scavenging Ghoul** — corpse counters per creature died + remove-to-regen.

Still deferred / noticed but not tackled:
- **Sea Drake** ("ETB return two target lands you control") needs an exactly-N
  own-permanent bounce with cast-time targeting; skipped.
- **Mtenda Lion / other "defending player may pay {U} to prevent this creature's
  combat damage"** need a pay-to-prevent combat rider.
- **Zombie Cannibal** ("combat damage to a player → exile a card from that
  player's graveyard") needs a defending-player-scoped graveyard-exile target.
- **Serrated Arrows** stores arrowhead counters as `CounterType::Charge` (no
  dedicated Arrowhead kind); the view tooltip reads "charge counter". Add a
  named counter kind if flavor fidelity matters.
- **Thicket Basilisk / Cockatrice** (destroy on block/blocked-by a non-Wall
  creature at end of combat) need a blocks-or-blocked-by delayed-destroy chain;
  skipped this wave.
- (✅ Sea Serpent ships via `Keyword::CanAttackOnlyIfDefenderControls` + a
  no-Island upkeep sacrifice, matching Dandân. Other Leviathans can reuse it.)
- **Bull Elephant / other "ETB unless you bounce two Forests"** upkeep-tax ETBs
  need a "sacrifice unless you return N matching lands" alt-cost ETB.

## Discovered follow-ups — ability-word conditions (`decks::abilitywords`, `recent68`)

Shipped: `Predicate::{ThresholdActive, MetalcraftActive, FerociousActive,
HellbentActive, FormidableActive}` (CR 207.2c ability words) + PlayerView flags
+ shared "✦ …" HUD chips; ~24 cards across `decks::abilitywords` and
`decks::recent68`. Fixed two latent catalog bugs on the way: Galvanic Blast's
non-metalcraft mode dealt 1 (should be 2), and Temur Battle Rage granted +1/+1 &
trample with double strike gated on Ferocious (should be double strike base,
trample gated on Ferocious). Also added a server guard: `usize_from_env_min`
rejects a `0`/sub-floor `CRAB_MAX_CONNS[_PER_IP]` misconfig. Still deferred:
- **Client build not verified here** — the Bevy client can't compile headless
  (missing `wayland-client`), so the new `StatChipKind::AbilityWord` chips and the
  `debug_export` literal update are code-reviewed but not run. Verify on a desktop.
- **Hellbent chip noise** — "✦ hellbent" lights whenever a hand is empty (common);
  consider gating it on controlling a hellbent-relevant permanent.
- **Atarka Monument** & animate-artifact "becomes a Dragon" abilities need an
  animate-self effect before shipping faithfully (skipped this wave).
- **Vulshok Replica / Barrage Ogre** deal to "any target" as an approximation of
  "target player or planeswalker" — narrow once such a selector lands.

## Discovered follow-ups — Spree / OTJ sweep (`decks::spree`, `decks::recent66`)

Shipped: **Spree** (CR 702.172 — `Effect::Spree` + `GameAction::CastSpellSpree`,
choose 1+ modes at cast, fold per-mode mana, run chosen modes at resolution;
server affordance `spreeable` + client highlight), **Read Ahead** (CR 702.155 —
`CardDefinition.read_ahead` + `saga_enter_advance` starting-chapter choice),
**Frenzy** as a first-class `Keyword::Frenzy(n)` combat rule (CR 702.35; Frenzy
Sliver now a real lord). 8 Spree spells + 13 OTJ staples with tests.
Still deferred:
- **Spree client per-mode UI** — ✅ shipped: right-click a spreeable card
  opens a mode picker (checkboxes; radio for Tiered via
  `KnownCard.spree_single_mode`) and casts `CastSpellSpree`. Remaining:
  per-mode *target* picking for multi-target mode sets (today the single
  armed targeting pass covers one target).
- **Spree/Escalate cast-time mode selection** for bots/auto-target — bots don't
  yet choose Spree modes, so a bot casting a Spree spell resolves the default
  (cheapest) mode only.
- **Trash the Town mode 3 / Final Showdown mode 1** — grant-a-triggered-ability
  and lose-all-abilities Spree modes need those effect primitives before those
  two Spree cards can ship faithfully.

## Discovered follow-ups — monarch/white sweep (`decks::recent53`)

Shipped (with reusable primitives): monarch-linked exile (`ExileUntilOpponentMonarch`),
per-count enters-with-counter (`TypeEntersWithCountersPerControlled` — Giada),
mill-then-branch-by-type (`MillThenBranchByType` — Old Rutstein), choose-a-number
+ chosen-MV noncreature lock (`ChooseNumberForSource` /
`NoncreatureSpellsWithChosenManaValueCantBeCast` — Sanctum Prelate),
remove-counters-from-among-creatures cost (`remove_counter_among_filter` — Hopeful
Initiate), monarch-gated `ExtraManaOnLandTap` (Regal Behemoth). Cards: By Force,
Palace Jailer, Loxodon Smiter, Leonin Vanguard, Marchesa's Decree, Custodi Lich,
Thorn of the Black Rose, Throne Warden, Skyline Despot, Keeper of Keys, Judith,
Gallant Cavalry, Valiant Knight, Adriana (melee grant).
Still deferred for want of a primitive:
- **Custodi Squire / Ballot Broker / Grudge Keeper** — vote-for-a-graveyard-card
  and additional-vote / vote-mismatch payoffs (voting is only wired for
  `WillOfTheCouncilExile` today).
- **Odric, Lunarch Marshal** — conditional team keyword-sharing across 12
  keywords ("your creatures gain X if any of yours has X").
- **Evolved Sleeper** — level-up-style activated abilities that permanently set
  the source's creature types + base P/T.
- **Sanctum Prelate** UI number-choice — the ETB choose-a-number currently uses
  the AutoDecider default for bots; UI suspend is a follow-up.

## Discovered follow-ups — counters / artifacts sweep (`decks::recent54`–`recent55`)

Shipped: `StaticEffect::OtherCreaturesEnterWithCountersEqualToSourcePower`
(Master Biomancer); faithful Melee via `Value::OpponentsAttackedThisCombat`
(CR 702.122). Still deferred for want of a primitive:
- **Massacre Girl** — needs a "this turn, whenever a creature dies, each other
  creature gets −1/−1" delayed chain trigger (the ETB −1/−1 sweep alone is only
  half the card).
- **Chandra, Acolyte of Flame** — 0-abilities incl. haste tokens that
  self-sacrifice next end step, and a −2 "cast an I/S from your graveyard this
  turn" grant.
- **Master Biomancer** — the "enters as a Mutant in addition to its types" layer
  rider is omitted (only the counters are wired).
- **Ingenious Smith** — "rest on the bottom in a random order" is approximated by
  `LookPickToHand { rest_to_graveyard: false }` (bottom, deterministic order).

## Discovered follow-ups — lifegain-matters sweep (`decks::recent56`)

Shipped primitives: `Player.starting_life` + `Predicate::PlayerLifeAtLeastAboveStarting`
(CR 103.4 — Speaker of the Heavens, Righteous Valkyrie; `PlayerView.starting_life`
surfaced + a start-relative HUD delta), `StaticEffect::LifeGainMultiplier` (CR 614
— Rhox Faithmender), `CardInstance.chosen_permanent` +
`Effect::ChoosePermanentForSource` + `Selector::ChosenPermanentOfSource` (Dauntless
Bodyguard — reads its stamp from die-LKI when sacrificed as the cost). GainLife/LoseLife
now surface `amount`-embedded target slots (Soul's Grace). Still deferred:
- **Heliod, Sun-Crowned / Daxos, Blessed by the Sun** — devotion-toughness CDA +
  gain-life-→-counter-on-target (both already exist as bodies elsewhere; the
  target rider / devotion-`*` toughness are the remaining faithful pieces).
- **Dawnbringer Cleric** — modal ETB with target-bearing modes (choose-one at
  resolution) isn't wired for creature ETBs.

## Discovered follow-ups — go-wide white sweep (`decks::recent57`)

Shipped with existing primitives (Requiem Angel, Angel of the Dawn, Elderfang
Disciple, Martial Coup, Beckon Apparition, Kytheon's Tactics, Rally the Ranks,
Captain's Claws, Ancestral Blade). Noticed but deferred for want of a primitive:
- **Trueheart Duelist** / **Selesnya Sagittars** (RAV) / **Valor Made Real**
  (DIS, "block any number this turn") — all want "can block an additional/any
  creature", i.e. a `Keyword::CanBlockAdditional(n)`/`CanBlockAny` wired into
  block declaration. Blocks are keyed blocker→attacker in a `HashMap` used at
  ~59 sites incl. combat-damage assignment, so this is a combat-model refactor
  (make `block_map` one-to-many + per-blocker damage-assignment order). Deferred
  this run as too invasive for a safe change.

## Discovered follow-ups — party sweep (`decks::recent58`)

Shipped `Value::PartyCount` (CR 700.18 — a max bipartite matching so a
multi-role creature fills only one slot; Changelings fill all). Squad Commander,
Kabira Outrider, Tajuru Paragon. Remaining ZNR party bits: the reveal-6
"shares a creature type with it" pick (Tajuru's kicked ETB is approximated as
"any creature card"); Kelsien's damage-party engine.

## Discovered follow-ups — spellslinger/tempo sweep (`decks::recent59`)

Sky Terror, Talrand's Invocation, Ondu Cleric, Aven Eternal (amass), Storm Fleet
Arsonist (raid), Metallurgic Summonings (the X/X Construct is minted as a 0/0 +
X +1/+1 counters read off the cast spell's mana value — no dynamic-P/T token
primitive needed after all; its exile-cost activated is approximated as a
sacrifice). Remaining: Docent of Perfection (transforming DFC + cast-trigger
Wizard count); Aeromunculus (a dedicated Adapt keyword vs the ad-hoc
counter-gated activated); Cursebound Witch (spellbook draft).

## Discovered follow-ups — deferred-clearing sweep (`decks::recent60`)

Shipped: `Effect::MayPayGenericUpTo { max, body }` (the X-cost sibling of
`MayPay` — prompt for a number ≤ cap and ≤ pool, spend generic, run `body`
with `event_amount` = paid). Cleared Jolrael, Mwonvuli Recluse (draw-2nd
trigger + `SetBasePT` team pump), Loyal Warhound (reused
`Predicate::OpponentControlsMoreLandsThanYou`), Well of Lost Dreams (the new
pay-up-to-X draw), Custodi Soulbinders (enters-with `CountOf - 1` +
`remove_counter_cost`).

## Discovered follow-ups — aggro batches (`decks::recent61`–`recent65`)

Red/white, Kaladesh artifacts, green midrange, blue tempo, and black aggro
(43 cards this run). Engine/UI/server fixes shipped alongside:
`keyword_is_friendly` now classifies `CantBlock`/`CantAttack`/`Decayed` as
hostile so "target creature can't block" auto-targets an opponent; client
`keyword_label` badges MustBlock/AttacksAlone/DealsNoCombatDamage/Exert/
Soulbond/spell-count-evasion; `view::ability_effect_label` surfaces
`MayPayGenericUpTo`; new `CreatureType::Hag`. Noticed but deferred:
- **Kalastria Highborn** — "whenever this or another Vampire dies, may pay {B}:
  target player loses 2 / you gain 2" needs a targeted reflexive drain inside a
  death-trigger `MayPay` (the target must be chosen after the cost).
- **Nether Traitor / Bloodghast-style recursion** — "when another creature is
  put into your graveyard, may pay {B}: return this from gy" — a graveyard-cast
  reflexive off a leaves-battlefield trigger.
- **Fathom Seer / Vodalian Mystic** — morph-flip cantrip and change-a-spell's-
  color are unbuilt (morph flip ships; the turned-face-up draw rider and stack
  color-change are the remaining pieces).

## Discovered follow-ups — TLA sweep (`decks::tla` batches 11–14)

Shipped: multi-kind permanent+player target slots (`Selector::ControlledBy
{ who: Target(n) }` — How to Start a Riot), `peak_per_ip` server telemetry,
Saddle HUD tag, `Value::ExcessDamageDealtThisResolution` (Razor Rings gains
life = excess; The Last Agni Kai adds {R} = excess — fight half faithful, the
"don't lose unspent red" rider dropped), Hei Bai (the move-all-counters LTB
uses the existing `Effect::MoveAllCounters`), Sun Warriors
(`Keyword::FirebendingCreaturesYouControl` — firebending X = creatures you
control), plus the `*`-power CDA cycle (Suki, Toph the Blind Bandit, Dragonfly
Swarm, Earthen Ally — new `DynamicPt` variants), Cycle of Renewal, Zuko's Exile,
Zuko's Conviction, Barrels of Blasting Jelly, Accumulate Wisdom, the mono-land
cycle (Abandoned Air Temple / Agna Qel'a / Ba Sing Se / Fire Nation Palace /
Realm of Koh), Price of Freedom, Leaves from the Vine (Saga), Rumble Arena,
Hakoda, Momo. Cleared this sweep (each shipped with a reusable primitive):
**Teo, Spirited Glider** + **Bitter Work** (`Predicate::AttackedWithCreatureMatching`),
**Sandbender Scavengers** (`SelectionRequirement::ManaValueAtMostSourcePower` +
`source_power_lki`), **Obsessive Pursuit** (existing `PermanentsSacrificedThisTurn`),
**Combustion Man** (`UnlessPlayerPays` + `WardCost::LifeSourcePower`),
**Katara, the Fearless** (`StaticEffect::DoubleControllerAllyTriggers`),
**Diligent Zookeeper** (`StaticEffect::PumpPTPerOwnCreatureType`), **Fire Lord
Zuko** (`SelectionRequirement::EnteredFromExileThisTurn`), **Raven Eagle**
(reused `Selector::ExiledThisResolution`), **Fatal Fissure** (`Effect::Earthbend`
auto-picks a controlled land in delayed bodies), **Serpent of the Pass**
(`StaticEffect::SelfCostReducedPerGraveyardCardMatching` + `SelfFlashIf`),
**Earth Rumble** (multi-slot earthbend+fight), **Allies at Last**
(`StaticEffect::SelfCostReducedPerPermanentMatching` — Affinity-for-type),
**Honest Work** (shrink-to-1/1 aura), **Bumi, King of Three Trials**
(`Effect::ChooseUpToN { max: Value, modes }` — choose up to a live count of
self-targeting modes), **Joo Dee, One of Many** (Surveil + `CreateTokenCopyOf`
self + sacrifice; the test pins determinism via an empty library + the
auto-picker's token-first sac heuristic). No primitive-blocked TLA cards remain;
the rest of the `set:tla` backlog is DFC bombs / Sagas / Vehicles / quest-counter
engines (see the Avatar card-backlog note below).

## Discovered follow-ups — Duskmourn/Foundations sweep (`decks::recent52`)

Shipped: `DynamicPt::CardTypesInControllerGraveyard` (Nethergoyf),
`SpendRestriction::AbilitiesOnly` (Omen Hawker), `Predicate::ValueIsPrime`
(Zimone). Deferred for want of a primitive:
- **Fear of Abduction** — needs `AdditionalCastCost::ExilePermanent` (exile a
  creature you control as you cast) + an exile-until-leaves that returns to
  *hand* (not battlefield).
- **Cursecloth Wrappings** — "{T}: target gy creature card gains embalm until
  EOT" needs a grant-embalm-to-a-graveyard-card effect (cf.
  `GrantFlashbackThisTurn`).
- **Caretaker's Talent** — Class leveling (`CardType::Class` + level-up
  activated abilities + "becomes level N" triggers) isn't modeled yet.
- **The Mindskinner** — "damage you'd deal to an opponent is prevented; they
  mill that much instead" needs a damage→mill replacement.
- **Niko, Light of Hope** / **Valgavoth, Terror Eater** — Shard tokens +
  exile-and-copy; Ward—sacrifice-three + play-from-exile-paying-life. Both want
  several primitives; left for a focused pass.
- **Winter, Misanthropic Guide** — the delirium "opponents' max hand size =
  7 − card types" downside is dropped (needs a delirium-gated dynamic
  `OpponentsMaxHandSizeReduced`); ward + symmetric upkeep draw ship.
- **Fear of the Dark** — the "if defending player controls no Glimmer" gate on
  the menace/deathtouch grant is approximated as unconditional.

## Discovered follow-ups — green/value sweep (`decks::recent46`–`recent49`)

Shipped: `StaticEffect::SelfCostReducedByTotalPower` (Ghalta) +
`SelfCostReducedPerCreatureInGraveyard` (Ghoultree); CDAs
`DynamicPt::LandsControlledPlusLandsInControllerGraveyard` (Multani) +
`CardTypesInOpponentsGraveyards` (Nighthawk Scavenger). Deferred for want of a
primitive:
- **Whisperwood Elemental** — the sacrifice grant ("face-up nontoken creatures
  you control gain death-manifest until EOT") needs a duration-scoped
  `GrantTriggeredAbility` over a creature set; the end-step manifest half is easy.
- **Genesis Hydra** — cast-trigger reveal-top-X + put a nonland permanent MV≤X +
  enters-with-X counters (an X-scaled `RevealTop…ToBattlefield`).

## Discovered follow-ups — utility-land / hatebear sweep (`decks::recent40`–`recent44`)

Cards confirmed absent and deferred this run for want of a mechanic:
- **Constant Mists** — Buyback with a *non-mana* cost ("Buyback—Sacrifice a
  land"). `Keyword::Buyback` only carries a `ManaCost`; needs a buyback variant
  that pays a sacrifice at cast time and still sets `bought_back`. Touches the
  central cast pipeline.
- **Daretti, Scrap Savant** — Goblin Welder's swap ships (`Effect::WeldArtifacts`);
  Daretti wants the same effect as a loyalty ability plus the discard/draw +1.
- **Dark Depths / Smokestack / Tangle Wire** — ice/soot/fade counter engines.

## Discovered follow-ups — TDM/DFT staples (`decks::recent29`/`recent30`)

New: `SelectionRequirement::WithAnyCounter` ("a creature with a counter on it");
`PlayerView.at_max_speed`; the bot now crews idle Vehicles (`pick_crew_vehicle`).

Per-card riders dropped (faithful headline shipped; complete when the engine
gains the missing piece):
- **Ainok Wayfarer** — "+1/+1 counter if you don't take a land" needs
  `LookPickToHand` to report whether a card was actually taken.
- **Iridescent Tiger** — "if you cast it" gate on the WUBRG ETB burst.
- **Embermouth Sentinel** — the "if you control a Dragon, onto the battlefield
  tapped instead" branch (conditional search destination).
- **Tersa Lightshatter** — graveyard-7 attack exile-and-play rider.
- **Earthrumbler / Glitch Ghost Surveyor / Aether Syphon** — the max-speed and
  exile-from-graveyard self-crew/draw riders.
- **Back on Track** — the Pilot token's "saddles/crews as though power +2" rider.
- **Bulwark Ox** — the saddled +1/+1-on-target half (the sac half ships).
- **Carrion Cruiser** — the gy-return is modeled as up-to-1 (`min:0`); the
  printed text is mandatory. A min-1 `ReturnGraveyardCardsToHand` would fix it.

Deferred TDM/DFT cards needing new engine primitives:
- **Sidisi, Regent of the Mire** — sac MV X → reanimate MV X+1 (cost-linked X).
- DFT "Mount that saddles/crews as though power greater" rider; bot saddle AI.

Shipped (`decks::recent102`): Stalwart Successor (`EventKind::AnyCounterAdded`
+ `with_per_subject_cap(1)`), Surrak, Elusive Hunter (BecameTarget +
`YourPermanentTargetedByOpponent`, creature-spell half dropped), Effortless
Master (cast-count-gated ETB counters).

## Discovered follow-ups — "becomes"/blink batch (modern_decks)

New this run: `Effect::BecomeCreatureType` (layer-4 set-creature-types one-shot),
`AdditionalCastCost::PayLife` (CR 119.4), `EquipBonus.set_land_types`,
`PermanentView.colors` (computed color surfaced to the client), the CR 613.8
type-lord recompute (`gate_types`), and a bot-friendly `ReturnGraveyardCardsToHand`.
Deferred riders:
- **Type-gated `CardMatch` lords** — the CR 613.8 recompute only covers
  `AllWithCreatureType`; a lord routed through `CardMatch` (disjunctive type
  filter) still reads printed types.
- **`EquipBonus` granted activated abilities** — Imprisoned in the Moon's
  "{T}: Add {C}" and Song of the Dryads' intrinsic mana are approximated (the
  resulting land carries no granted activated ability).
- **Essence Flux / Displace / Ghostly Flicker** — immediate multi-target blink
  + a "+1/+1 if Spirit" rider on the returned (new) object; the target ref
  doesn't survive exile→return, so the counter can't be applied.
- **Joraga Treespeaker** — `LevelBand` carries only P/T + keywords, not the
  level-gated activated mana ability / Elf-grant.
- **Curse of the Swine** — exile X *target* creatures + a 2/2 per exiled
  creature's controller (multi-target + per-target `ControllerOf` token).

## Discovered follow-ups — LCI / Craft / Descend (`sets::lci`)

Shipped: Craft (CR 702.169 — `shortcut::craft` + `craft_exile_cost`),
the **Discovered event** (CR 701.57 — `GameEvent::Discovered` / `EventKind::
Discovered`, value via `Value::TriggerEventAmount`) wiring "whenever you
discover" payoffs (Curator of Sun's Creation re-discovers once per turn),
Descend (`SelectionRequirement::ControllerDescend`, `Predicate::{DescendActive,
DescendedThisTurn}`, `DynamicPt::PermanentCardsInControllerGraveyard`, HUD chip),
`LandType::Cave` + the 10-card Cave subset (Captivating Cave, Volatile Fault,
Promising Vein, Forgotten Monument's grant, the 5 Hidden Discover caves,
Spelunking via `StaticEffect::LandsEnterUntapped`). Deferred from the set:
- **Craft "or pay 3 life" / "discard or sac" choices** are approximated as a
  flat discard (Bitter Triumph, Souls of the Lost) — needs a real either-cost
  `AdditionalCastCost` mode. Craft's per-object UI exile-choice is auto-picked.
- **Bonehoard Dracosaur, Quintorius Kand, Tarrian's Journal, the remaining
  craft DFCs** (Sunbird Standard's color-count CDA, exiled-card recast) need
  bespoke effects; deferred.
- **LCI cards still ⏳** (each needs the noted bespoke primitive — the LCI
  remainder is a hard legendary/artifact tail):
  - Starving Revenant — surveil-2 then draw/lose-3 per card kept on top.
  - The Skullspore Nexus — **batched** "one or more nontoken creatures die →
    token whose base P/T = their *total* power" (per-creature `CreatureDied`
    can't sum; needs CR 603.3e batch grouping). Cost-reduction-by-greatest-power
    and the {2},{T} double-power activated half are ready (`Value::
    GreatestPowerYouControl`, `Effect::DoublePower`).
  - Quintorius Kand — planeswalker; +1/−3 trivial, the −6 needs
    "exile any number from your gy, add {R} each, may-play this turn".
  - Zoyowa's Justice / Wail of the Forgotten / Cosmium Confluence — player-scoped
    Discover (`Discover` is always controller-only) and choose-N-with-repeats.
  - Bat Colony (Cave-mana-spent count), Song of Stupefaction (gy-count
    EquipScale), Intrepid Paleontologist (cast Dinosaurs from exile-with-source),
    Roaming Throne (chosen-type trigger-doubling), The Millennium Calendar
    (time-counter engine), Saheeli (token-copy + add-a-card-type rider).
  - Shipped this run: `Effect::AddCountersForPowerOverBase` (Okinec),
    `Effect::SacrificeOthersThenReanimate` (Bringer of the Last Gift),
    `AdditionalCastCost::TapPermanents` (Guardian of the Great Door; reused by
    Fear of Exposure), `Effect::MayTap` (Caparocti), `Effect::EscalatingThisTurn`
    + `ability_resolutions_this_turn` (Vito), `Effect::
    RevealTopNPutMatchingToBattlefield` (Gishath), `DynamicPt::
    CreaturesYouControlWithTypes` + `Value::TimesDescendedThisTurn` +
    `Player.descend_count_this_turn` (The Mycotyrant; surfaced in `PlayerView.
    descended_this_turn_count`), plus Chimil, Abuelo, Palani's Hatcher.
  - A 23-card BLB/FIN/DSK commons batch (no new primitives) lives in
    `sets::decks::recent27` (+ Fear of Exposure / Vicious Clown in `recent26`).
- ✅ **Reflexive targeted "when you do" triggers** (CR 603.7) — `Effect::Reflexive
  { body }` wraps a targeted payoff that's opaque to the cast/trigger-time target
  walk and auto-targets its body fresh at resolution. Composes with `MayPay`
  (Itzquinth, Firstborn of Gishath — pay {2}, then a Dinosaur bites another
  creature) and `MaySacrifice` (Glorifier of Suffering — sac a creature/artifact,
  then support 2). Both shipped in `sets::lci` with tests.
- **Molten Collapse / Abuelo's Awakening** deferred: Molten Collapse needs a
  descend-gated "choose both" modal (conditional extra mode with per-mode
  targets); Abuelo's Awakening needs reanimate-as-1/1-flying-Spirit + X counters.
- **Opponents-must-attack-you static** (Trove of Temptation) — a global force-
  attack (≥1 creature per opponent at you / your planeswalker each combat), a new
  `StaticEffect` honored by the attack-legality check and the bot's attack
  heuristic. Only the end-step Treasure half is tractable today, so the card is
  deferred rather than shipped at half fidelity.
- **Saheeli, the Sun's Brilliance** — token-copy that's "an artifact in addition
  to its other types" + sac-at-next-end-step. `CreateTokenCopyOf` lacks the
  add-a-card-type rider; deferred.
- **Master of the Hunt / banding** (CR 702.22) — the Wolves-of-the-Hunt token
  needs real banding; deferred.
- **"Descended this turn" reset** is at untap; double-check end-step descend
  payoffs see the flag (they fire in the same turn's end step — verified for
  Deep Goblin Skulltaker).
- **Cross-module name dedup** — many LCI commons are reprints already living in
  `decks::modern` / `decks::recent*` under the same `pub fn` name. When adding
  set cards, grep the *whole* catalog (`pub fn <name>(`), not just the set
  module — duplicate glob re-exports compile (warning only) but bloat the
  registry. An `audit_stubs`-style check that flags two factories with the same
  `name:` string would catch this automatically.

## Discovered follow-ups — this run (recent25/26, blight)

- ✅ **Ward-cost side effects fire triggers, with `TriggerSource` bound.** The
  ward-payment `CounterAdded`/sacrifice events *were* reaching the dispatcher;
  the real bug was `event_subject` having no `CounterAdded` arm, so
  `Selector::TriggerSource` was unbound on every counter-added trigger (filters
  over an empty selector pass vacuously). Auntie Ool's Ward—Blight off an
  opponent's creature silently *drew for her* instead of draining the opponent.
  Fixed by binding `CounterAdded`'s subject to the counter-receiving permanent.
- **Territorial Bruntar (EOE)** wants an "exile from top until a nonland card;
  you may cast it this turn" effect (impulse-until-nonland with a pay-cost
  may-play, *not* Cascade's free-cast). Would also unblock similar landfall
  impulse cards.
- **DFT/DSK still have ~70 gaps each** after this run's batches; the
  `set_gaps.py` over-reports helper-built cards (positional `name` args), so
  filter by the literal card-name string before judging a card missing.

## Discovered follow-ups — DSK / DFT staples (`decks::recent23`)

Mechanics deferred while batching the 20-card `recent23` wave:
- ✅ **Eerie** (DSK ability word — "whenever an enchantment you control enters
  and whenever you fully unlock a Room") — `shortcut::eerie(body)` returns the
  enchantment-ETB + `EventKind::RoomFullyUnlocked` trigger pair;
  `set_room_door_unlocked` emits `GameEvent::RoomFullyUnlocked` when both doors
  open. Cult Healer, Balemurk Leech, Optimistic Scavenger, and Unwilling Vessel
  (`CounterType::Possession` + a death trigger minting an X/X flying Spirit
  whose `dynamic_pt` reads the possession-counter count via CR 603.10 LKI) all
  ship.
- ✅ **Mount saddle / Pilot crew power bonus** (DFT, CR 702.122e/702.171 —
  "saddles Mounts and crews Vehicles as though its power were N greater") —
  `StaticEffect::CrewSaddlePowerBonus`, read by `crew`/`saddle`, the bot's
  `pick_crew`, and surfaced in `PermanentView.crew_power_bonus` (Cloudspire
  Captain, Deathless Pilot in `decks::recent24`). Also ✅ crew/saddle **by
  toughness** (`StaticEffect::SelfCrewsSaddlesWithToughness` — Interface Ace),
  and effect-driven saddle (`Effect::SetSaddled` — Guidelight Matrix).
- ✅ **Crew/saddle triggered event** (DFT, CR 702.122/702.171 — "whenever this
  creature saddles a Mount or crews a Vehicle during your main phase, …") —
  `EventKind::CrewsOrSaddles` + `GameEvent::VehicleCrewed { crew }` /
  `MountSaddled { riders }`; the crewer's `SelfSource` trigger fires with the
  crewed permanent as `Selector::TriggerSource`, gated to the controller's main
  phase. Canyon Vaulter, Reckless Velocitaur in `decks::recent24`.
- ✅ **"Untap all attackers each combat" rider** (Full Throttle) —
  `Effect::AtEachCombatThisTurn { body }` registers a turn-scoped
  `DelayedKind::EachCombatThisTurn` delayed trigger that re-fires the body at
  every Begin-Combat step and expires at cleanup. Full Throttle untaps
  `EachPermanent(Creature & AttackedThisTurn)`.
- **Spree** (DSK/OTJ, CR 702.172 — "choose one or more additional costs") — no
  variable additional-cost-mode primitive yet; blocked on cast-time modal
  selection (same as the guild Commands). Blocks Insatiable Avarice, Caught in
  the Crossfire.
- **Other genuinely-unimplemented CR keywords** (each needs a new subsystem):
  Read Ahead (702.155 — Saga enters-chapter choice), Living Metal (702.161 —
  needs the MTMTE transform DFCs). (Space Sculptor, 702.158, now ships.)
- ✅ **DSK deferred cards shipped** — Sawblade Skinripper
  (`Player.permanents_sacrificed_this_turn` + `Value::PermanentsSacrificedThisTurn`
  / `Predicate::PermanentsSacrificedThisTurnAtLeast`); Twitching Doll
  (`is_mana_ability` accepts an incidental self-`AddCounter` rider,
  `Value::TotalCountersOn`, `CounterType::Nest`, sac-cost `leaves_bf_lki` stash);
  Toby, Beastie Befriender (`Keyword::CantAttackOrBlockAlone` +
  `StaticEffect::PumpTeamIf` token-count anthem); Fanatic of the Harrowing now
  gates its draw on `Predicate::DiscardedThisEffect`.
- ✅ **Conditional enters-with-counters by cast zone** (Patched Plaything) —
  the ETB context now stamps `cast_from_hand` from the entering instance, so
  `enters_with_counters` can gate its `Value` on `Predicate::CastFromHand`
  (`Value::IfPred`). Both the cast-resolution and the generic
  `place_card_in_dest` paths thread the flag.

## Discovered follow-ups — Edge of Eternities (`sets::eoe`)

Warp / Void / Lander / **Station** shipped (see the rules-audit rows). Still open:
- **Legendary Spacecraft now ship** — Dawnsire (10+ attack-100, 20+ flying
  20/20), Infinite Guideline Station (ETB Robots per multicolored + 12+ flying/
  attack-draw), The Eternity Elevator (20+ mana-per-charge band). The Seriema
  was already shipped. `StationBand.triggers`/`.activated` cover counter-gated
  triggered + activated bands.
- **2026-06-27 batch (modern_decks):** 20 missing rares/lands — Possibility
  Technician (impulse ETB; "if you control a Kavu" gate dropped), Haliya, both
  Alpharaels, Roving Actuator (Void gy-recast), Tannuk (haste anthem; in-hand
  warp grant dropped), All-Fates Scroll (`Value::DifferentlyNamedLandsControlled`),
  Command Bridge, Secluded Starforge (pump dropped), Atomic Microsizer, Dyadrine,
  Zero Point Ballad (X≥6 reanimate dropped), Scout for Survivors
  (`Effect::ReturnGraveyardCreaturesUpToTotalManaValue`), Weftwalking (first-
  spell-free static dropped), Pull Through the Weft (land-return half dropped),
  Close Encounter (additional-cost choice → greatest power), Devastating
  Onslaught (`Effect::CreateTokenCopiesHasteSac`), Unravel
  (`Effect::CounterSpellDrawIfUnderpaid`). **Remaining EOE rares** (want new
  primitives): Famished Worldsire (Devour-land on a 0/0 — needs an enters-with-
  counters replacement so it survives SBA), Sothera, Chorale of the Void, Moonlit
  Meditation, Requiem Monolith, The Dominion Bracelet (control-an-opponent),
  Pull's land-return half. (Tapestry Warden ✅ via
  `Keyword::AssignsCombatDamageByToughness` — its Station-by-toughness half is
  still dropped.)
- **`set_gaps.py "set:eoe"` still lists ~85 cards** (the script's `name:`-regex
  over-reports helper-built cards as missing — Pulsar Squadron Ace now ships via
  `LookPickToHand{pick_filter: Spacecraft}`, so the "filtered impulse-reveal" note
  is resolved). Easy batches remaining: more Lander makers, Warp creatures, Void
  payoffs, vanilla/keyword commons. **Planets** (`LandType::Planet` — Adagia,
  Evendo, Kavaron, Susur Secundi, Uthros) ship as tapland + tap-for-color +
  Station. **`StationBand.activated` now exists** (CR 721.2a — surfaced via
  `granted_abilities_for` when charges ≥ `min`), so Evendo/Uthros (`12+` scaled
  mana), Susur Secundi (`12+` sac-a-creature draw), **Adagia** (`12+`
  legendary token-copy via `CreateTokenCopyOf { legendary: true }`) and
  **Kavaron** (`12+` sac-a-land → Robot + `Seq` team haste/pump) all ride
  faithful activated bands.
- **Client render of `PermanentView.station_charges`** (new — current charge
  count paired with `station_next_threshold` for an "N/M" Station chip). The
  view field + test ship; the Bevy chip render is the remaining desktop-only
  piece. The bot now **cracks Lander tokens for ramp** (`pick_crack_lander`)
  when it has spare mana and a basic still in library.
- **Recently shipped (modern_decks):** Pulsar Squadron Ace, Umbral Collar Zealot,
  Sunset Saboteur, Station Monitor, Virulent Silencer, Steelswarm Operator, Syr
  Vondam (Sunstar Exemplar), Starfield Shepherd, Timeline Culler, Tannuk
  (Memorial Ensign), Xu-Ifit, Monoist Circuit-Feeder, Space-Time Anomaly, Systems
  Override, Mutinous Massacre, Focus Fire, Scour for Scrap, Terminal Velocity,
  Melded Moxite, Squire's Lightblade, Auxiliary Boosters, Thaumaton Torpedo,
  Terrasymbiosis, Weapons Manufacturing, Syr Vondam (the Lucent), Starwinder,
  Pinnacle Starcage (ETB mass O-Ring; `{6}{W}{W}` dump-to-gy/make-Robots payoff
  dropped), Temporal Intervention (Void cost reduction + each-opponent discard;
  the "target opponent" single-target restriction is approximated to each
  opponent), Vote Out (Convoke destroy), Archenemy's Charm (modal exile / mass
  gy-return / counters+lifelink), **Illvoi Infiltrator** — now ships the new
  `Keyword::CantBeBlockedIfControllerCastSpells(n)` (CR 509.1b, enforced in
  `declare_blockers`), reusable for other "can't be blocked if you've cast N
  spells" cards. New deferrals from that batch:
  **Tannuk's "second landfall this turn → draw"** needs a per-source
  resolution-count predicate; **Timeline Culler's cast-from-graveyard warp**
  needs `AlternativeCost` to allow a graveyard cast zone; **Syr Vondam's
  "or is put into exile" branch** is dropped (dies-only); **Steelswarm
  Operator's** two restricted mana abilities both collapse to `ArtifactOnly`.
- **Astelli Reclaimer** ✅ — `CardInstance.cast_mana_spent` stamped at
  resolution; `Value::CastSpellManaSpent` + `SelectionRequirement::Mana-
  ValueAtMostCastManaSpent` read it on ETB. **Blade of the Swarm** ✅ (modal ETB:
  counters / put exiled warp card on owner's library bottom). Still open:
  **Sothera, the Supervoid** (each-opponent exile-a-creature on friendly death +
  end-step reanimate — wants an `EachPlayerExilesChosen` primitive). Emissary
  Escort ✅, Fungal Colossus ✅ (both already shipped — the notes were stale).
  Dark Endurance ✅, Genemorph Imago ✅,
  Memorial Vault (`{T}, Sac an artifact: impulse 1 + sacrificed MV` — now ships;
  `Value::SacrificedManaValue` is stamped for non-creature sac-cost permanents,
  not just creatures), Starport Security (tap-another; conditional discount
  dropped), Mm'menon the Right Hand (cast artifacts from library top; restricted
  mana grant dropped),
  Mental Modulation (`SelfCostReducedDuringYourTurn`) and Anticausal Vestige
  (LTB cheat-into-play via `ManaValueAtMostYourCount`) now ship. Orbital Plunge's
  "if excess damage" now reads real excess (`Predicate::ExcessDamageDealtThisResolution`,
  CR 120.10).
- **Newly-noticed deferrals (this run).** **EOE Auras now ship** — Hardlight
  Containment (enchant-artifact O-Ring), Cryoshatter (−5/−0 +
  destroy-on-tap/damage), Meltstrider's Resolve (ETB fight) and Pain for All
  (ETB ping = power) use the standard Aura shape (`Attach` + `equipped_bonus`).
  Remaining EOE auras: **Tractor Beam** ✅ (control-steal: ETB tap +
  `GainControlWhileSourceRemains` + `PreventUntap{AttachedTo(This)}`),
  **Chorale of the Void** (attack → reanimate from defender's graveyard),
  **Moonlit Meditation** (token-doubling replacement). **Territorial Bruntar** /
  Possibility Technician need filtered impulse-dig (exile-top-
  until-nonland). **Haliya, Ascendant Cadet** ships without
  its "counter-creatures deal combat damage → draw" rider (needs a
  combat-damage-from-filtered-creature trigger). **Excess-to-another-permanent
  redirection** (CR 120.4a — "excess damage is dealt to … instead") is the next
  excess-damage piece. **Equip-bonus *triggered* abilities on attack** (Atomic
  Microsizer) and Survey Mechan's distinct-land-name *activation* discount are
  unwired.
- **New deferrals (this run).** Shipped batch 2 carries approximations worth a
  later pass: **Sothera, the Supervoid** still unbuilt (wants an
  `Effect::EachPlayerExilesChosen` + an end-step "if a player has no creatures"
  mass-reanimate); **Quantum Riddler** drops the "draw +1 while hand ≤1" draw
  replacement; **Survey Mechan**'s {10} activation has no distinct-land-name
  discount and routes the draw to you (not "target player … loses 3 life");
  **Sami, Wildcat Captain**'s affinity is approximated to instant/sorcery spells
  (the static can't grant it to creature/artifact spells); **Loading Zone**'s
  counter-doubling isn't restricted to creatures/Spacecraft/Planets.
- Approximations in the shipped batches: All-Fates Stalker drops the "up to one
  non-Assassin" rider; Elegy Acolyte's combat trigger fires per-creature; Tidal
  Terror omits tap-two-to-be-unblockable; Larval Scoutlander's sacrifice option
  is land-only (the "or Lander" branch is dropped). Station's tap-a-creature
  cost now auto-picks the *highest*-power creature for non-UI seats (charges
  scale with the tapped power, CR 702.184a; `auto_pick_highest_power` gated on
  `TappedForCostPower`); other tap-another costs still tap the lowest.
- **Client tooltip "Station → N" line** (`PermanentView.station_next_threshold`)
  ships but is un-exercised in headless CI (the Bevy client can't build without
  wayland); verify it renders on a real desktop run.

## Discovered follow-ups — `decks::ltr` / The Ring batch

- **Ring-bearer choice for UI players.** `GameState::ring_tempts` auto-picks the
  controller's highest-power creature as Ring-bearer. A `wants_ui` player should
  be prompted (a `Decision::ChooseTarget`-style pick among their creatures);
  wire it through the suspend/rerun path like other resolution-time choices.
- **Client HUD: surface `PlayerView.{ring_temptations,ring_bearer}`** as a
  "The Ring ×N" chip + a bearer marker on the creature (fields are on the view;
  the client chip render is the remaining piece).
- **More LTR cards** beyond the current `decks::ltr` set (Sauron cycle,
  Frodo/Sam *Partner* mechanic, Orcish Bowmasters' opponent-extra-draw trigger,
  Old Man Willow's sac-on-attack, Lost to Legend's "Nth from top" tuck, …).
- **Frodo Baggins "must be blocked while Ring-bearer" rider.** Frodo's ETB
  legendary-tempt half ships; the conditional `MustBeBlocked`-while-bearer
  static is approximated away. Needs a Ring-bearer-gated keyword grant.
- ✅ **"Assigns combat damage equal to toughness" keyword**
  (`Keyword::AssignsCombatDamageByToughness`, read off the computed keyword set
  in `combat_damage_value`) — Doran, the Siege Tower (all creatures), Tapestry
  Warden (your creatures with T>P, via a `ToughnessGreaterThanPower` CardMatch
  static grant), Bill the Pony (Food-sac temporary grant). `decks::recent23`.
  Tapestry's "stations using toughness" half is dropped.
- **Choice additional cost "sacrifice a creature or pay {N}"** (Lash of the
  Balrog). `AdditionalCastCost` has no OR-of-costs variant yet.
- **Food-per-creature-sacrificed count** (Voracious Fell Beast): currently
  mints one Food regardless of how many opponents sacrificed.
- **"Whenever this or another creature you control dies" self-clause.** A
  `CreatureDied`/`YourControl` trigger on a creature does NOT fire for the
  creature's *own* death — the dispatch walks the battlefield and the dying
  creature is already gone. The other-creature half works. Wire the dying
  card's own YourControl death triggers via death-LKI (held back Warbeast of
  Gorgoroth, which would otherwise amass on its own death).
- **Observer death-trigger LKI counters.** An `AnotherOfYours`/`CreatureDied`
  trigger that reads the *dead* creature's counters (`MoveAllCounters` from
  `Selector::TriggerSource`) gets nothing: `leaves_bf_lki` is only stashed for a
  dying creature that has its *own* die-triggers, and `died_card_snapshots` is
  cleared at the end of `dispatch_triggers_for_events` — before the observer's
  trigger resolves on the stack. Fix: stash `leaves_bf_lki` for every dying
  creature an observer trigger might read, keyed/cleaned by trigger_source.
  The same gap affects `AnotherOfYours`/`PermanentLeavesBattlefield` observer
  triggers (a *non-death* exile/bounce of another permanent): the subject's
  controller lookup fails once it's gone, so the trigger doesn't fire. (Held
  back Buzzard-Wasp Colony's counter-inheritance, Host of the Hereafter's
  other-dies branch, and Suki, Courageous Rescuer's "another permanent leaves
  → Ally token" rider.)

## Discovered follow-ups — `decks::recent8`/`9`/`10` (Avatar / Lorwyn) batch

- **Avatar (`tla`) card backlog.** ~28 `set:tla` non-land cards remain (verified
  against the *whole* catalog, not just `tla.rs`). Shipped since: Airbender
  Ascension (quest engine + flicker), Appa Steadfast Guardian, Redirect
  Lightning, Zhao the Moon Slayer (counter-gated `LandTypeChangerWhileCounters`),
  Toph Hardheaded Teacher (`Effect::MayDiscard`), Crashing Wave, Avatar Destiny
  (`EquipScale.count_graveyard` + `EquipBonus.add_creature_types`). The remainder
  each need a non-trivial primitive: the Avatar planeswalker / DFC bombs (Aang,
  Ozai, Koh), **Firebender Ascension** (copy an attacking creature's own
  triggered ability at 4+ quest counters), DFC Sagas (The Legend of … // Avatar
  …), Vehicles with Exhaust animation (Invasion Submersible, Phoenix Fleet
  Airship). Toph the First Metalbender needs a "nontoken artifacts are lands"
  static. Solstice Revelations needs an impulse-cast-if-under-a-count effect.
  Avatar's Wrath needs an "airbend all creatures except the chosen target"
  selector.
  - ✅ **Exhaust** activated-ability keyword (CR 702.177) — already supported via
    `ActivatedAbility.exhaust`; now used by Rebellious Captives, Rough Rhino
    Cavalry, Mai Jaded Edge.
  - ✅ **"second time this resolved this turn → draw"** rider (South Pole Voyager)
    — `EscalatingThisTurn { modes: [Noop, Draw, Noop] }` (per-source, 2nd only).
  - ✅ **Conditional player-wide cost reduction gated on a graveyard count**
    (`StaticEffect::CostReductionWhile { filter, amount, condition }` — Gran-Gran).
  - ✅ **"Spend only to cast Lesson spells"** mana restriction
    (`SpendRestriction::LessonSpellsOnly` + a `lesson` flag on `SpellKind`) —
    Hermitic Herbalist.
  - ✅ **Dynamic Firebending X = source's power** (`Keyword::FirebendingPower`) —
    Firebending Student.
  - ✅ **Bounce-then-draw-if-you-controlled-it** (Boomerang Basics) — an
    `If(EntityMatches{target, ControlledByYou})` ahead of the bounce.
- ✅ **Waterbend (CR 701.67).** Shipped — completes the bending family
  (earthbend/airbend/blight). `CardDefinition.waterbend: Option<Waterbend>`
  (`GameAction::CastSpellWaterbend`) for the additional cast cost (mandatory +
  optional "you may waterbend"; `waterbend {X}` reads the chosen X via
  `Value::XFromCost`; provenance `cast_via_waterbend` →
  `Predicate::SpellWasWaterbend`), and `ActivatedAbility.waterbend: bool`
  (`GameAction::ActivateAbilityWaterbend`) for the ability cost. Helpers ride the
  `convoke_creatures` slot generalized to artifacts+creatures, clamped to the
  amount. `decks::avatar_water` (20 cards) + tests in `tests/avatar_water.rs`.
  `KnownCard.{has_waterbend,waterbend_amount}` surface it to the client.
  Follow-ups: bot helper-use heuristic (bot can already cast via mana); the
  client right-click "Cast (waterbend)" affordance + helper picker; `Ward—
  Waterbend` (The Unagi — approximated as Ward {N}); `Exhaust—Waterbend`
  (Invasion Submersible); Secret of Bloodbending's "control a player"; Foggy
  Swamp Visions' exile-gy-creatures-as-token-copies.
- **Blight as an activated/additional cost.** `Effect::Blight` is a resolution
  effect; cards that put it in a *cost* ("{T}, Blight 1: …" — Gristle Glutton,
  Dawnhand Dissident; "As an additional cost, blight 1" — Cinder Strike) need
  blight wired into the cost-payment path.
- **"Each opponent blights N."** `Effect::Blight` uses `ctx.controller`; High
  Perfect Morcant / Auntie Ool's `Ward—Blight` make an *opponent* blight. Needs
  a per-player blight and a `WardCost`-with-blight variant.
- **Multi-draw count-filter root cause.** "Draw your Nth card each turn" payoffs
  evaluate `CardsDrawnThisTurn` at batch-dispatch time, so a multi-card draw
  (Divination) leaves the count at N for *every* CardDrawn event in the batch.
  Worked around with `once_per_turn` on the ==N payoffs (recent8/9 + Mischievous
  Mystic + two Modern cards). The clean fix is to evaluate the filter against the
  per-event draw ordinal (carry it on `EventKind::CardDrawn`), which also fixes
  "draw your *first* card" payoffs that no-fire on a pure multi-draw turn.
- **Client: render `ExileCardView.may_play_alt_cost`** on the in-hand /
  battlefield castable affordances too (the exile-browser badge already shows
  "May play (you) for {2}").

## Discovered follow-ups — `decks::recent3` / `sets::fin` batch

Deliberately deferred this run (cards/features that need engine work beyond
existing primitives):
- ✅ ~~**Modal activated abilities — mode selection via
  `GameAction::ActivateAbility`.**~~ — the action carries `mode` end to end.
- **Ethersworn Canonist** — "one *nonartifact* spell per turn" needs a
  per-player nonartifact-spell counter (the existing `OneSpellPerTurn` static
  counts every spell). Add a filtered variant + tracker.
- **Steel Hellkite** — `{X}: destroy each nonland permanent with MV X whose
  controller was dealt combat damage by this creature this turn` needs a
  combat-damage-by-source player set + an X-filtered mass destroy.
- **Minas Tirith** — "enters tapped unless you control a legendary creature"
  needs a conditional `EntersTapped` (no predicate-gated enters-tapped yet).
- **Fabled Passage** — search a basic tapped, then untap it if you control 4+
  lands; needs a conditional untap of the just-fetched land.
- **Den Protector / Prismatic Strands** — megamorph turn-face-up gy-return; and
  prevent-all-damage-of-a-chosen-color with white-creature-tap flashback.
- **The Ring tempts you (LTR)** — level 2's defender-chosen forced block and the
  interactive Ring-bearer choice are UI-bound; deferred until those are modeled.

## Discovered follow-ups — `decks::recent` batch

Card riders deliberately approximated/omitted this batch (each card otherwise
plays its headline pattern):
- ✅ **Screaming Nemesis** — `Effect::LifeGainLockGame { who }` permanently
  sets `Player.cannot_gain_life`; wired as `Selector::Target(0)` after the
  damage redirect so it only fires when a player was hit (CR 119.7).
- ✅ **Lumbering Worldwagon / `*`-power Vehicles** — `DynamicPt::
  LandsControlledPower { base_p, base_t }` tracks lands in *power* only while
  toughness stays fixed. Worldwagon ships (`*`/4 Vehicle, Crew 4, enters-or-
  attacks basic-land fetch).
- **Tarrian's Soulcleaver** — equip-granted "whenever another artifact/creature
  is put into a graveyard, counter the equipped creature" needs an
  `EquipBonus.triggered_abilities` entry keyed on a non-SelfSource gy-put event;
  unbuilt.
- ✅ **Cache Grab** — `Effect::MillThenToHand { amount, filter }` mills, then
  puts one card matching `filter` (`SelectionRequirement::PermanentCard`) from
  among those milled into hand. Squirrel→Food rider approximated to controlling
  a Squirrel ("returned a Squirrel this way" half omitted).
- **`Effect::VillainousChoice` / `Effect::TimeTravel` UI** — both resolve via a
  bot heuristic (lesser-self-harm option / advance own suspended cards); per-
  object UI prompting for a `wants_ui` player is a follow-up. No real card wires
  VillainousChoice yet (Sycorax Commander, Ensnared by the Mara want it).
- ✅ **Leaves-the-battlefield-without-dying trigger** — `GameEvent::
  CreatureLeftWithoutDying` + `EventKind::CreatureLeavesBattlefieldNotDying`
  fire on a creature's non-graveyard exit (bounce / exile). Dour Port-Mage's
  draw rider and Three Tree Scribe both ship. (Approximations: the self-leave
  half of Three Tree Scribe doesn't fire — the source is gone from the
  battlefield by dispatch; Dour Port-Mage draws per-creature on a mass bounce
  rather than once per batch.)
- **Krydle of Baldur's Gate** — the attack pay-{2}-for-unblockable rider is
  omitted (wants a `MayPay`-on-attack); the combat-damage drain ships.
- **Thornplate Intimidator** — its "target opponent" Punisher is modeled as
  each opponent (1v1-faithful; multiplayer hits all opponents).
- **The Ring tempts you / Ring-bearer** (LTR) — not yet modeled; see
  FEATURE_ROADMAP Tier 4. Per-player ring level (1–4) + designated Ring-bearer
  with cumulative attack/blocked/combat-damage granted abilities.
- **Questing Beast** — combat-damage-can't-be-prevented ships
  (`StaticEffect::ControllerCreaturesCombatDamageCantBePrevented`, CR 615.12);
  only the deals-damage-→-planeswalker redirect rider remains (needs
  planeswalkers as attack targets).
- **The Necrobloom** — the 7-different-named-lands Zombie upgrade and the
  "lands in your graveyard have dredge 2" grant omitted.
- **Lightning Axe / Demand Answers** — modal additional costs ("discard OR pay
  {5}" / "sac an artifact OR discard") collapsed to the discard branch, taken at
  resolution (Deadly-Dispute style). A general modal-additional-cost layer would
  make these faithful.
- **Optimistic Scavenger** — the "fully unlock a Room" half of Eerie omitted.
- **Hangar Scrounger** — the Backup *grant* of its tapped-loot ability to the
  backed-up creature is omitted (the +1/+1 counter still lands).
- **Bristlebud Farmer** — the attack "sacrifice a Food → mill three, grab a
  permanent" rider omitted (the ETB two-Food still fires).
- **Tersa Lightshatter** — "discard up to two, then draw that many" variable
  rummage + the 7+-graveyard attack exile-and-play rider need a variable-loot
  primitive; unbuilt for now.
- **Karakyk Guardian** — conditional "hexproof while it hasn't dealt damage"
  omitted (no lifetime damage-dealt tracking); ships flying/vig/trample.
- **Temur Battlecrier** — the "during your turn" gate on its power≥4 Affinity
  reduction is approximated as always-on (`affinity_filter`).
- **Sarkhan, Soul Aflame** — the become-a-copy trigger keeps the copied Dragon's
  name; the "name stays Sarkhan, legendary in addition" override is approximated.
- **Take the Fall** — the outlaw check uses a controlled-outlaw selector count.
- Not implemented for want of a mechanic: **Spree** (Insatiable Avarice, Three
  Steps Ahead — cast-time per-mode additional costs), **Intensity** (Static
  Discharge), **Channel** (Touch the Spirit Realm's Channel ability).
- ✅ **Speed / "Start your engines!"** (CR 702.179) — `Player.speed` (0–4) +
  `Keyword::StartYourEngines` (sets speed to 1 on entry) + life-loss increment
  (once per your turn, capped at 4) + `Predicate::SpeedAtLeast` for "Max speed
  —" riders. Surfaced in `PlayerView.speed`. Cards: Nesting Bot, Burnout
  Bashtronaut, Swiftwing Assailant, Risen Necroregent, Embalmed Ascendant, Vnwxt,
  Zahur, The Speed Demon. CR 702.179f (no speed = 0 for speed-referencing
  effects) verified: `cr_702_179f_no_speed_counts_as_zero`.
  Remaining: client HUD speed chip; the rest of the DFT speed pool.
- (✅ Caustic Bronco ships — `RevealTopToHandLoseMv { who }` generalized the old
  Sorin-only opponents-lose-MV effect, and `Predicate::SourceSaddled` picks the
  branch.)
- **Sovereign Okinec Ahau** — deferred: attack "for each creature you control
  with power greater than that creature's base power, add the difference in
  +1/+1 counters" needs a per-creature base-power-comparison value.
- **Inti, Seneschal of the Sun** — the discard trigger fires per discarded card
  (`CardDiscarded`/`YourControl`) rather than once per "you discard one or more
  cards" event; a discard-batch event would make it faithful.
- **Client** — the hand hint shows no Kicker/Offspring *cost* label; the
  right-click "cast with the optional cost" path now works (`CastSpellKicked`),
  but surfacing the cost (a `kicker_cost_label` on `KnownCard`) is a follow-up.

## Discovered follow-ups — MID/VOW batch (modern_decks)

Riders deliberately approximated/omitted while shipping the Innistrad batch
(each card otherwise plays its headline pattern):
- **Disturb-into-Aura back faces** ✅ shipped — `GameAction::CastDisturb` now
  carries a `target`, so an Aura back face is cast targeting a creature (engine +
  bot wired; Kindly Ancestor, Twinblade Geist, Mischievous Catgeist). Remaining:
  the client targeting affordance (the GUI never built these recasts directly),
  and the Aura backs that grant a *targeted* triggered ability (Distracting Geist
  // Clever Distraction's "attacks → tap target", Gutter Skulker's attacking-alone
  unblockable static).
- **Olivia, Crimson Bride** — the reanimated creature's "when you don't control a
  legendary Vampire, exile this" rider is omitted (needs a granted
  count-gated delayed exile).
- **Henrika Domnathi** — begin-combat "choose one that hasn't been chosen
  before" needs per-source mode-history tracking across turns.
- **Gisa, Glorious Resurrector** ✅ shipped — the redirect now stamps
  `exiled_with` and `Effect::ReturnExiledBySourceToBattlefield { decayed }` mass-
  reanimates them at upkeep. **Toxrill** still wants slime-counter team-shrink
  (per-end-step counter + scaling `-X/-X` static + sacrifice-at-zero).
- **Skipped this run for want of a primitive:** Spectral Adversary (the
  scale-with-kicks *phase-out* rider — `ApplyToTargets` takes a fixed `max_targets`,
  not a runtime `Value`); Mulch (`MillThenToHand` picks one card, not *all*
  matching — needs a "mill N, all matching → hand" variant). **Moonrager's Slash**
  ✅ shipped — new `CardDefinition.self_cost_reduction_if_night` ("{N} less if it's
  night"). A fully general `cost_reduction_if_predicate` is still future work
  (e.g. Geistlight Snare's two stacked board conditions).
- **Sigarda's Vanguard** ✅ shipped approximated (up-to-three targets gain double
  strike); the printed "any number of creatures with *different powers*" coven-
  style distinct-power `ChooseN` is still the only gap.
- **Counter-threshold transform DFCs** — Smoldering Egg // Ashmouth Dragon,
  Poppet Stitcher // Poppet Factory, Voldaren Bloodcaster // Bloodbat Summoner
  all "transform when N+ counters/tokens accrue". Needs an `Effect::If` over a
  `Value` reading counters-on-this / token-count plus a remove-and-`Transform`
  tail (no `transform-at-threshold` helper yet).
- **Lier, Disciple of the Drowned** ✅ shipped — new
  `StaticEffect::GraveyardInstantsSorceriesHaveFlashback` (graveyard I/S gain
  flashback = mana cost), wired into the flashback-cast path + graveyard view.
- **Search Party Captain** ✅ shipped — new
  `StaticEffect::SelfCostReducedPerCreatureAttackedThisTurn` ("{1} less per
  creature you attacked with this turn").
- **"Sacrifice a creature or pay {N}" additional cast costs** — Morkrut
  Behemoth (body only) and Eaten Alive (mandatory-sacrifice approximation via
  `SacrificeAndRemember`, dropping the pay-instead branch) both want a real
  "pay alt-cost OR sacrifice" additional-cost modal at cast.
- **Spend-restriction mana from creatures** — Unblinking Observer ("{T}: Add
  {U}. Spend only to pay a disturb cost or cast an instant/sorcery") is not yet
  shipped. (Cobbled Lancer ✅ — additional-cost graveyard-exile + the {3}{U}
  exile-from-graveyard cantrip both ride existing primitives.)
- **Cleave** ✅ — Lunar Rejection ships via the existing `AlternativeCost`
  cleave (strips the bracketed Wolf/Werewolf restriction). Lantern Flare still
  deferred (its cleave changes an `{X}` damage spell into a board wipe).
- **`SelectionRequirement::HasDisturb`** ✅ — payload-agnostic "card has Disturb"
  predicate (Shipwreck Sifters' discard payoff).
- **Patchwork Crawler** — "has all activated abilities of cards exiled with it"
  needs a granted-abilities-from-linked-exile primitive.
- **Damage-redirect enrage on a werewolf DFC** — Ill-Tempered Loner //
  Howlpack Avenger ("when dealt damage, deal that much to any target" + a
  `{1}{R}: +2/+0` firebreathing ability on both faces); needs `werewolf_dfc`
  extended to carry activated abilities, plus an enrage-redirect-to-any-target.
- **Augur of Autumn** ✅ shipped (play-lands-from-top); its coven cast-creatures-
  from-top rider is the only remaining gap.

## Discovered follow-ups — missing-card sweep (modern_decks)

Real cards confirmed absent, deferred for want of a mechanic:
- **Owner-choice top/bottom of library** for a battlefield permanent — Revenge
  of the Drowned, Diver Skaab ("its owner puts it on top or bottom"). Need a
  `ZoneDest`/effect that bounces a permanent to its owner's library with a
  top-or-bottom decision (the countered-spell `OwnerLibraryTopOrBottom` zone
  exists but isn't reachable from a generic `Move`).
- ✅ **Temporary "sacrifice at the next end step" tokens** — Hungry for More
  ships; the minted token carries an end-step self-sacrifice trigger
  (`StepBegins(End)` → `Move(This → Graveyard)`), same shape as Dress Down.
- **Spend-restricted "cast from your graveyard" mana** — Rootcoil Creeper's
  second ability (its ramp half is currently dropped); add a
  `SpendRestriction::GraveyardCastOnly` to `mana.rs`.
- **Both-dynamic-P/T tokens** — Seize the Storm's `*/*` Elemental (P=T= I/S in
  gy + flashback-in-exile). `DynamicPt::InstantsSorceriesInGraveyardAndExile`
  only drives power; want a variant (or token CDA) that sets both.
- ✅ **Assign-combat-damage-by-toughness** — `Keyword::AssignsCombatDamageByToughness`
  read in `combat_damage_value` (Doran, Tapestry Warden, Bill the Pony,
  `decks::recent23`). Ancient Lumberknot and other "assigns combat damage equal
  to its toughness" cards can now ride it directly.
- **Cast-time "choose one or both" modal with a targeted mode** — Markov
  Retribution (mode 2 targets a Vampire + another creature). `Effect::ChooseN`
  exists but per-mode target-slot derivation for the targeted half is fiddly.
- **Shared-card-type cost reduction off linked exile** — Cemetery Prowler
  ("spells cost {1} less for each card type they share with cards exiled with
  this creature"); needs a cost-reduction static keyed on `exiled_with`.
- **Cast-from-top once-per-turn static** — Cemetery Illuminator's "once each
  turn you may cast a spell from the top of your library" (play-from-top exists
  for lands/permanents; the once-per-turn any-spell variant doesn't).
- **Class enchantments** (Bandit's/Stormchaser's/Innkeeper's Talent, and the
  whole DSK/BLB Class cycle) — need a Class type with sorcery-speed pay-to-
  level-up (CR 714-adjacent) layering on abilities per level band. `level_bands`
  exists for Leveler *creatures* only. Roadmap Tier 3 "Classes/Cases/Backgrounds".
- **Adversary cycle** — the "pay {cost} any number of times" ETB payment is
  modeled as **Multikicker** (paid at cast, functionally identical for these).
  Intrepid Adversary ✅ (`CounterType::Valor` + `PumpPTPerCounterOnSource` team
  anthem) and Bloodthirsty Adversary ✅ (kick-scaled +1/+1) ship. Remaining:
  Bloodthirsty's "exile up to that many I/S of MV ≤3 from your graveyard and
  copy them, cast free" value rider; the other three Adversaries (Dauntless,
  Quilled, Spineless) still want their kick-scaled payoffs.
- **Valgavoth, Terror Eater** — Ward—Sac 3 ships (`WardCost::SacrificePermanents`),
  but the opponent-graveyard→exile replacement and "play exiled cards paying life
  = MV" recursion engine are unbuilt.
- **Kaervek, the Punisher / "commit a crime" payoffs** — no crime-tracking event.
- **Loot, Exuberant Explorer** — dig-6 put-creature-with-MV≤lands needs an
  MV-≤-dynamic-count filter on `LookPickToHand`.
- ✅ **Day/night transition trigger** — `EventKind::DayNightChanged` (matched to
  `GameEvent::DayNightChanged { was_transition }`, true only on a real day↔night
  flip, not on establishing day from neither) fires "whenever day becomes night
  or night becomes day" with `EventScope::AnyPlayer` (Brimstone Vandal).
- ✅ **"Shares a card type with the exiled card" trigger** —
  `Predicate::SharesCardTypeWithExiledBySource` compares the just-cast spell /
  just-played land (via `ctx.trigger_source`) against the card this source
  exiled (`exiled_with == source`); Cemetery Gatekeeper (ping the player) and
  Cemetery Protector (mint a Human) ship, splitting the printed "plays a land
  or casts a spell" into a LandPlayed + SpellCast trigger pair.

Newer deferrals (modern_decks, counter/aristocrat/aggro batches):
- **Per-color mana-spent tracking (Adamant, CR 702.137).** Slaying Fire /
  Searing Barrage ship with the Adamant rider dropped (base damage only). Needs
  the cast pipeline to record how much of each color was actually spent so a
  `Predicate::SpentAtLeastNOfColor` can gate the bonus. Roadmap Tier 5.
- **I/S-filtered graveyard→exile static (Dryad Militant, Yixlid Jailer).**
  `StaticEffect::ExileCardsBoundForGraveyard` has `colors`/`opponents_only`/
  `own_only` but no *card-type* filter; add one so "if an instant or sorcery
  would be put into a graveyard, exile it instead" can be modeled.
- **Winding Constrictor "counters you'd get" clause.** `ExtraCounterAllKinds`
  covers counters placed on your artifacts/creatures; the player-counter half
  (poison/energy/experience you'd *get* → +1) is approximated away.
- **PermanentLeavesBattlefield for noncreatures.** `EventKind::
  PermanentLeavesBattlefield` only matches `GameEvent::CreatureDied`, so a
  noncreature artifact going to a graveyard via *destroy* doesn't fire it
  (Chromatic Star's cantrip is latently dead on the non-sacrifice path).
  Terrarion / Implement / Disciple of the Vault / Marionette Master sidestep
  this by keying on `PermanentSacrificed` instead. A canonical once-per-leave
  event dispatched in the same LKI window as `CreatureDied` would fix it
  properly (the naive mid-SBA dispatch clobbers `died_card_snapshots`).
- **Deferred real cards** wanting unbuilt primitives: Serra Paragon
  (once-per-turn play-from-graveyard with a delayed-exile rider), Enduring
  Tenacity (lifegain→opponent-loses scaled by amount + Enduring dies-return-as-
  enchantment), Sheltered by Ghosts (Aura with a second ETB exile-until-leaves
  target), Fear of Missing Out (Delirium first-attack token rider),
  Bandit's/Innkeeper's Talent (Class type).

## Noticed this run (modern_decks — Judgment closure, Vanguard, Onslaught)

- **`audit_catalog_stats.py` now resolves file-local card helpers.** Most
  classic sets build cards through `fn creature(name, cost, types, p, t)` and
  pass the printed stats positionally, so the audit's `name:` / `power:` field
  scans skipped them — 10 678 cards checked before, **14 602** after. The first
  run of the widened audit found and fixed 20 cost / 6 P/T / 4 type drifts
  (Wind Spirit, Panther Warriors, Arrest, Untamed Hunger, Ophidian Eye, Sedge
  Sliver, Homing Sliver, Spawn of Rix Maadi, Gore-House Chainwalker, Warden of
  Geometries, Cultivator Drone, Curse of the Pierced Heart, Curse of
  Bloodletting, Hammer of Purphoros, Bronze Sable, Void Grafter) and six
  `{X}` spells that spelled X as `generic(0)` or omitted it entirely
  (Lunar Frenzy, Form a Posse, March of Otherworldly Light, Primal Might, Bond
  of Agony, Overrule) — those never offered the caster an X prompt.
- ⏳ **3 TYPE + 46 KEYWORD drifts remain**, down from 33/52. Run
  `python3 scripts/audit_catalog_stats.py <set>` for the list; each needs a
  per-card read (some are deliberate — a granted keyword modelled as a static
  isn't printed on the card, and Silhana Ledgewalker's flagged "Flying" is the
  audit's regex reading the nested `CantBeBlockedExceptBy(HasKeyword(Flying))`).
  The 3 remaining type rows are Changeling-style cards whose "every creature
  type" is modelled by listing a handful (Stonework Packbeast, Tajuru Paragon).
- ⏳ **Fabricated real-name bodies surfaced by the stat sweep.** Correcting the
  printed stats made a few name/body mismatches obvious — Mai, Scornful Striker
  is printed "whenever a player casts a noncreature spell, they lose 2 life" but
  is modelled as an attack drain; Surging Sentinels and Descendant of Storms
  likewise carry synthesized bodies. Same class as the STX fabricated-name row.
- ⏳ **Vanguard statics from the command zone.** `seat_vanguard` applies
  `NoMaximumHandSize` at seating; a general "a command-zone avatar's static
  abilities apply" pass would need the ~100 battlefield-scanning static
  gathers to also walk command zones (Birds of Paradise Avatar, Dauntless
  Escort Avatar, Ixidor-style face-down lords).
- ⏳ **`Effect::ApplyToTargets.max_targets` is a `u8`.** "X target creatures"
  currently rides `TargetsExactlyX` wrapping a fixed ceiling (Wave of
  Indifference uses 8). A `Value`-typed ceiling would drop the magic number.
- ⏳ **Onslaught is at 219 gaps.** The remaining bulk is Morph (engine support
  ships) and the tribal commons; Legions and Scourge follow.

## Noticed this run (Legends closure / ATQ opening)

- **Optional-pay triggers never fire for bot seats.** `AutoDecider` answers
  `Decision::OptionalTrigger` with `false`, so `Effect::MayPay` bodies (Urza's
  Miter, Tablet of Epityr, Urza's Chalice) are dead weight in bot games even
  when the mana is floating. Wants a cheap "pay it if you can afford it and the
  body is pure upside" policy rather than a blanket decline.
- **CR 616.1e beyond draws.** The player now chooses among competing *draw*
  replacements; the ETB, damage and counter-placement families still apply in a
  fixed order.
- **Five hand-written `EventSpec { .. }` literals** in `decks::{modern,
  tarkir, tdm}` spell out every field instead of using
  `EventSpec::new(..).with_filter(..)`, so each new `EventSpec` field has to be
  hand-patched into them. Convert them to the builder.
- **`attackable_players_for` is defender-scoped.** It drops defenders no
  attacker could legally hit, but attacker-*filtered* prohibitions (a
  `CreaturesCantAttackController { filter: Some(..) }`) still only surface as a
  rejected `declare_attackers`. A per-attacker legality preview would let the
  client grey out individual creatures.

## Antiquities (ATQ) — remaining 9

`sets::atq` ships 55 of the set's 64 gaps. Each of the rest wants one primitive
the engine doesn't have yet:

- **Clockwork Avian / Tetravus** — `+1/+0` counters with a printed cap, and
  an upkeep counters↔tokens exchange linked to the tokens this creature made.
- **Primal Clay / Urza's Avenger** — as-enters choice of a whole P/T + keyword
  + subtype profile (`enters_as_choice` only sets P/T), and a repeatable
  `{0}`-shrink-for-a-chosen-keyword.
- **Xenic Poltergeist** — the one-shot half of Titania's Song, scoped to a
  single artifact "until your next upkeep"; `Duration` has no such window yet.
  (Titania's Song itself ships; its "keeps working until end of turn if this
  leaves the battlefield" rider is dropped.)
- **Transmute Artifact** — sacrifice-then-search with a pay-the-difference
  branch.
- **Tawnos's Coffin** — exile a creature *and its Auras*, noting counters, and
  return the whole bundle re-attached when the Coffin untaps or leaves.
- **Golgothian Sylex** — needs per-card set provenance ("originally printed in
  Antiquities") in `CardDefinition`.
- **Goblin Artisans** — coin flip that counters *your own* artifact spell,
  with the "isn't the target of another Goblin Artisans" linked restriction.

## New TODO suggestions (push modern_decks)

### Foundations (fdn) card gaps — each needs one primitive
- **Infernal Vessel / Nine-Lives Familiar** — a self-return-on-death that
  re-enters with counters and a type/loop-guard (Vessel becomes a Demon; the
  Familiar loses a revival counter). Needs a self-`CreatureDied` → `Move` from
  graveyard + `AddCreatureTypes`/`AddCounter` on `LastMoved` with a death-LKI
  check that survives the return (else infinite loop).
- **Raise the Past / Dewdrop Cure** — mass "return all/each creature card with
  MV ≤ N from your graveyard to the battlefield" (only per-total-power/MV caps
  exist today, not a per-card MV filter).
- **High Fae Trickster** — a flash-granting static already exists
  (`StaticEffect::ControllerSpellsHaveFlash`); a straightforward add next run
  (4/2 flash flyer + that static).
- **Banner of Kinship** — choose-a-type + fellowship-counter-scaled anthem
  (`AnthemForChosenType` scaled by counters on the source).
- **Quick-Draw Katana / Celestial Armor** — Equipment with a "during your turn"
  conditional `EquipBonus`, and flash-Equipment ETB-attach + a temporary
  hexproof/indestructible grant.
- **Fiery Annihilation** — damage + exile-attached-Equipment + a per-target
  "if it would die, exile it instead" death replacement on the *target*.
- **Consumed by Greed / Dewdrop Cure** — Gift spells with a "if the gift was
  promised, instead …" enhanced branch (the Gift machinery exists; these want
  the promised-branch to widen a graveyard return).

### Client GUI follow-ups
- ✅ **Graveyard click-to-cast** — badge-bearing tiles in the graveyard browser
  now submit their recast on click (Flashback / Mayhem / Harmonize / Disturb /
  Retrace / Escape via `graveyard_recast_click`; targets auto-picked, Escape
  fodder auto-chosen). Remaining nicety: a real target-picker step for Aura
  Disturb backs, and manual Escape-fodder selection.
- **Surface new combat riders in tooltips.** The reminder/ability panel should
  note `Keyword::CantBeBlockedByPowerLess` (Formation Breaker) and a
  turn-scoped "can attack despite defender" badge once
  `attack_despite_defender_this_turn` holds for a permanent (Krotiq Nestguard).
  Also surface `SetBasePtIf`'s conditional base P/T (Snowmelt Stag) in the
  hover P/T readout. Unverifiable headless — pair with a desktop session.

### Tarkir: Dragonstorm follow-ups
- **Omen cycle is complete** — all 17 Dragon Omen cards ship (`decks::omen`),
  seeking via the new `Effect::Seek` (CR 701.52 random library pick). The cards
  were enumerated from the offline `scripts/.scryfall_cache.json` (36k entries).
  Whirlwing Stormbrood's "cast sorceries/Dragon spells as though they had flash"
  is now faithful (reuses `StaticEffect::ControllerSpellsHaveFlash`). Pearl Lake
  Warden's "look at/cast this from the top of your library" is now faithful too
  (`TopOfLibraryRevealed` + `PlayFromLibraryTop { HasName }`).
- **`Effect::JoinCombatAttacking`** currently attacks the source's defender (else
  the first opponent). Add a chosen-defender/planeswalker variant for cards that
  reanimate "attacking a player or planeswalker of your choice".
- **Unmodeled TDM keywords:** **Flurry, Mobilize (+ `mobilize_value` for "Mobilize
  X"), Renew, Harmonize are done.** Harmonize ships as `Keyword::Harmonize(cost)`
  + `GameAction::CastHarmonize` (graveyard recast, optional tap-a-creature generic
  discount, exile-after; bot + graveyard-browser badge). Still open: a conditional
  self-keyword-while static ("has hexproof if it hasn't dealt damage yet" — Karakyk
  Guardian; needs a `source-hasn't-dealt-damage` predicate); and **Mardu
  Thunderkite-style perpetual keyword grants** (perpetual effects aren't modeled).
- **Web-slinging (CR 702.188) done** via the alternative-cost primitive
  (`AlternativeCost.mana_cost` + `return_to_hand` of one tapped creature) —
  Spider-Man, Web-Slinger / Amazing Spider-Girl / Silk / Spider-Man India. Still
  deferred: the "if this spell was cast using web-slinging" provenance riders
  (Spiders-Man, Scarlet Spider) — would need a `cast_via_web_slinging` mark like
  the new `cast_via_mayhem` flag.
- **Noticed (TDM Renew/Mobilize batches):** Sibsig's Artisan's "perpetually gains
  this ability" Renew rider and Rot-Curse Rakshasa's "X target creatures get a
  decayed counter" (multi-target Renew) are deferred — perpetual ability-grants
  and divided-counter Renew aren't wired.
- **Highspire Bell-Ringer / "second spell each turn costs {1} less"** — a
  cost-reduction keyed on the second-spell condition; no static for it yet.
- **Deferred TDM cards (noticed this run, want a primitive):**
  - **Hundred-Battle Veteran** — "+2/+4 while 3+ different *kinds* of counters
    among creatures you control" needs a distinct-counter-kind-count predicate;
    plus cast-from-graveyard-with-finality.
  - **Sage of the Skies** — "when you cast this, if you've cast another spell
    this turn, copy this spell" — a cast-trigger conditional `CopySpell` on self.
  - **Furious Forebear** — "when a creature you control dies while this is in
    your graveyard, may pay {1}{W} to return it" — a from-graveyard death
    trigger (source in graveyard, not on battlefield).
  - **Abzan Monument** — sac payoff mints an X/X Spirit where X = greatest
    toughness among your creatures; needs a `Value::GreatestToughnessYouControl`
    (only `GreatestPowerYouControl` selector exists) + create-token-with-Value-PT.
  - **Behold-a-Dragon (Exhale cycle)** — currently approximated as a
    Dragon-*control* conditional; the printed cost also lets you *reveal* a
    Dragon from hand. A faithful additional-cost "behold" + `cast_via_behold`
    provenance flag would tighten the riders.
- **Search-by-`ControllerOf(target)`** only resolved through a
  `Selector::TargetFiltered{slot}` inside the `ControllerOf` (plain
  `Selector::Target(0)` came back empty during trigger resolution — Magmatic
  Hellkite). Worth auditing why `Target(0)` doesn't resolve in that context.

### Content — Theros Beyond Death (THB) is the active set being filled
Regenerate the remaining list with `cargo run -p crabomination_catalog
--example dump_names thb` diffed against a `set:thb` Scryfall name dump. The
earlier "still deferred" list (type/PT-change auras, scaled negative pump,
conditional mana, land-search count, the planeswalkers, the demigods) all
shipped — see git. **Genuinely-absent THB cards remaining** (each wants the
primitive noted):
- **Pile-split — fully interactive.** Atris + Fact or Fiction now ship via
  `Effect::FactOrFiction` (a value heuristic: opponent isolates the single
  highest-MV card, you keep the higher-value pile). A genuinely interactive
  `Decision::SplitPiles` (opponent chooses the partition, you choose the pile)
  remains a refinement — would need a two-step inline decision shape.
- **Allure of the Unknown** ✅ — `Effect::AllureOfTheUnknown` reveals the top
  six, the opponent exiles the highest-MV nonland (heuristic) with a free
  `may_play_until`, the rest go to your hand. Remaining nicety: an interactive
  opponent pick rather than the value heuristic.
- **Bronzehide Lion** — dies → returns as an Aura granting indestructible
  (creature→Aura transform on return). Unique; no primitive yet.
- **Athreos, Shroud-Veiled** — death-reanimate ships; the "or is put into
  exile" half of the return trigger is still approximated (counters clear on
  the exile zone-change before the trigger reads them — would need an
  exile-from-battlefield LKI snapshot).
- **Storm Herald** ✅ — `Effect::ReanimateAurasExileEot` returns gy Auras
  attached to your creatures (auto-picks a legal creature per aura) + a
  per-aura `NextEndStep` delayed exile. Remaining nicety: the "if those Auras
  would leave, exile instead" replacement rider (currently a plain delayed
  exile).
Shipped this run: Ashiok's Erasure (`StaticEffect::OpponentsCantCastNamed` +
`Effect::CounterSpellExileNameLock`, linked counter-exile returning to hand on
leave), Entrancing Lyre (`CardInstance.untap_locked_by` tap-lock +
`SelectionRequirement::PowerAtMostXFromCost`), Haktos the Unscarred
(`Keyword::ProtectionFromManaValueExcept` wired into targeting/damage/blocking,
random ETB via d3), Medomai's Prophecy (chosen-name Saga;
`DelayedKind::YourNextNamedSpellThisTurn` + `Effect::OnYourNextNamedSpellThisTurn`
for the name-gated chapter-III draw; chapter IV's look-at-each-top is
informational and modeled as `Noop`).

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

### Engine — Cast-from-exile pipeline (partly shipped)

`ActivatedAbility.from_exile` now supports pay-cost recasts from exile
(Squee, the Immortal), and `CastWithoutPayingImmediate { copy }` casts copies
from exile (Capricious Hellraiser). The remaining general shape:


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
- `Effect::MayCopyThisSpell` (the Chain cycle) asks the *affected* player
  through the installed decider rather than suspending for that seat's UI —
  the same known class as the decision-plumbing audit below. Wire the
  cross-seat suspend when the pending machinery grows a per-seat prompt.
- `PlayerView.face_down_cast_cost` reports the real morph price (the flat {3}
  less any `FaceDownSpellsCostLess` static — Dream Chisel), so the client can
  label the "cast face down" affordance instead of assuming {3}. Client-side
  rendering of that label is still to do.
- Trigger-filter debug logging (`TriggerFiltered { source, kind, scope, reason }`);
  a mana-paid-for-optional audit event; per-cast-face metrics. (Snapshot
  round-trip coverage for each run's new `#[serde(default)]` fields now lands
  with the fields — see `core_rules/cr_recent48`.) (Ward is now
  factored into hostile auto-targeting — un-warded candidates first.)

## 1v1-collapsed "target player" effects (multiplayer worklist)

The engine plays 1v1, so several printed "target player" clauses are
collapsed to `You` / `EachOpponent` — observably identical in two-player
games, wrong in multiplayer. When a multiplayer push lands, convert each
to a real player-target slot (the Time Warp fix in the 2026-07 STX audit
is the template: `PlayerRef::Target(0)` + a `Player` slot filter):

| Card | Where | Printed scope | Collapsed to |
|---|---|---|---|
| Inquisition of Kozilek | `decks/spells.rs` | target player | EachOpponent |
| Tendrils of Agony | `stx/extras_01.rs` | target player loses 2 | Drain EachOpponent |
| Callous Bloodmage (mode 3) | `stx/witherbloom.rs` | target player's graveyard | ExilePlayerGraveyard EachOpponent |
| Quandrix Command (mode 3) | `stx/quandrix.rs` | target player shuffles ≤3 target cards | You, no card targeting |
| Primal Command (mode 2) | `decks/modern.rs` | target player shuffles graveyard | You only |
| Tempted by the Oriq | `stx/extras_00.rs` | per-opponent steal | single steal (max_targets 1) |
| Multiple Choice (X=2) | `stx/mono.rs` | "may choose a player" (any, incl. self) | EachOpponent returns |
| Devastating Mastery (alt rider) | `stx/silverquill.rs` | "an opponent chooses" | EachOpponent (= the opponent in 1v1) |

Also multiplayer-sensitive but structural: `Punisher`-heuristic choices
(the affected player auto-picks; UI prompting for opponents is tracked
above), and `EachPlayerKeepsOneSacrificeRest` auto-picks by highest MV.

## Noticed this run (modern_decks BNG completion + enchant-player)

- ✅ ~~**`EventKind::YourInstantOrSorceryDealtDamage` doesn't carry the
  victim**~~ — shipped as a sibling event,
  `YourInstantOrSorceryDealtDamageToPlayer`, which fires once per damaged
  player with that player bound as the trigger subject;
  `SelectionRequirement::ControlledByTriggerPlayer` reads it off
  `GameState.trigger_event_player_scratch` (stamped at both target-enumeration
  and resolution time). Satyr Firedancer is faithful.
- **Perplexing Chimera's "you may choose new targets".** The control exchange
  ships; re-targeting the stolen spell does not
  (`CopySpellMayChooseTargets` has the prompt shape to borrow).
- **Whims of the Fates piles are engine-chosen.** A shuffled round-robin split;
  the printed card lets each player build their own three piles. Wants a
  `Decision::PartitionPermanents`.
- **CR coverage gaps.** `scripts/cr_coverage.py` → `CR_COVERAGE.md` maps CR
  section → conformance test; 113 sections are covered, 33 still have none.
  The highest-value untested blocks left are 407 (Ante), 713/717 (substitute /
  Attraction cards), 727–732 (restart / subgames / shortcuts), the rest of the
  8xx multiplayer rules (801 limited range, 804-809/811) and the 9xx casual
  variants.

## Noticed this run (modern_decks — Kamigawa closure + Urza's Legacy)

New suggestions from this run's engine work:

- **Top-of-library orientation needs an invariant, not a convention.** Index 0
  is the top everywhere, but `ExileTopUntilNonland` and Shared Fate's draw
  replacement both read `library.last()`/`pop()` — the BOTTOM. Both are fixed;
  a `fn library_top(&self, p)` / `fn take_library_top(&mut self, p)` pair would
  make the next one a compile-time choice instead of a silent bug.
- **`ask_seat_amount`'s replay log and `ask_seat_cards`'s stash are two
  different suspend contracts.** `ask_seat_bool`/`ask_seat_amount` replay a log
  and support several asks per resolution; `ask_seat_cards` stashes one answer
  and must be the arm's first ask. Pain's Reward needed the log shape.
  Converging `ask_seat_cards` onto the log would let a multi-ask arm mix card
  and amount picks.
- **The `AnthemFor*` family has three scope spellings.** `AnthemForFilter` and
  `AnthemForChosenType` now both carry `opponents` + `all_players`;
  `AnthemForChosenColor` still has neither. Worth a shared `AnthemScope` enum.


SOK's last eight gaps shipped, so the whole Kamigawa block (CHK/BOK/SOK) is at
zero `set_gaps.py` gaps. Urza's Legacy went 106 -> 13 across two waves.

- ✅ ~~**`PlayerDamaged` triggers can't reach the damage source.**~~ —
  `event_subject` now binds the DEALER for `EventKind::PlayerDamaged` (the
  receiver-side wording), leaving the dealer-side kinds binding the damaged
  player. No Mercy ships, and Michiko Konda's edict now hits only the damage
  source's controller instead of every opponent.
- **Remaining ULG gaps (6), each blocked on one primitive:** Angel's Trumpet
  (tap-all-that-didn't-attack + damage per tapped), Aura Flux (grant an upkeep
  tax to OTHER enchantments), Damping Engine (a permanent-leader lock with a
  sacrifice-to-ignore out — `Predicate::PlayerControlsMostOf` is the read
  half), Martyr's Cause (a chosen-source damage shield), Memory Jar
  (exile-hand, draw-seven, return at the next end step), Thran Weaponry (a
  while-this-stays-tapped anthem).
- **`EventSpec::with_filter` silently REPLACES.** Every shared card-builder
  that adds a gate has to fold the caller's filter in by hand (the ULG Opal
  cycle's `opal()` helper does). A `with_extra_filter` that `All`-composes
  would make the safe thing the default.
- **The Epic copy doesn't re-choose targets.** CR 702.50a's copies "may choose
  new targets"; `process_epic` reuses the original's. Eternal Dominion /
  Neverending Torment / Undying Flames all want it.
- **Erayo's Essence counters the first *opponent* spell via `once_per_turn`.**
  Exact at two players; in multiplayer it should be per-opponent, which needs a
  per-actor trigger budget rather than the per-trigger one.
- **Sasaya's Essence doubles by `ExtraManaKind::Mirror`, not by same-named
  land count.** One extra mana per tap instead of one per other same-named
  land you control.
- **Whims of the Fates piles are engine-chosen** (carried over): a shuffled
  round-robin split; the printed card lets each player build their own three
  piles. Wants a `Decision::PartitionPermanents`.

## Noticed this run (modern_decks — Betrayers of Kamigawa closure)

- ✅ ~~**SOK's Sweep cards**~~ — `Effect::ReturnAnyNumberToHand` +
  `Value::PermanentsReturnedThisEffect`.
- ✅ ~~**SOK's Epic sorceries**~~ — all four ship. Residual: the epic copy
  reuses the original's targets rather than re-choosing them each upkeep.
- ✅ ~~**Ashes of the Fallen wants a graveyard type-grant static.**~~ —
  `StaticEffect::YourGraveyardCreaturesHaveChosenType` + `graveyard_type_grants`,
  read by the hidden-zone card evaluator.

- **CR 712.4 doesn't fire on the direct move path.** `place_card_in_dest`
  reverts a flip card (CR 710.4) but deliberately leaves `revert_transform` /
  `revert_prototype` to `place_card_at_resolved_zone`: a defeated battle
  transforms *through* exile, and reverting there would undo it. Wants the
  battle-defeat path to stamp the back face after the hop rather than before.
- **Opal-Eye's redirect is team-scoped.** `PreventNextFromChosenSourceToTeam`
  shields the controller and their permanents; the printed card moves the
  chosen source's next damage onto Opal-Eye no matter who it was aimed at.
  Wants a shield whose target set is "any", not `PlayerAndPermanents`.
- **Flames of the Blood Hand's unpreventable rider is global.**
  `DamageCantBePreventedThisTurn` suppresses every shield for the turn, not
  just the Flames damage. Wants a per-source prevention lock.
- **The splice picker can't pre-pick per-clause targets.** The client's
  `HelperMechanic::Splice` submits an empty `additional_targets` and lets
  `cast_spell_spliced` auto-aim each spliced clause. A UI seat should get one
  targeting pass per targeting splicer.

## Noticed this run (modern_decks — OGW closure + BFZ)

- **`Effect::CopyForEachOtherTargetableCreature` skips modal/X re-choice.**
  Zada's copies inherit the original's mode, X and converged value verbatim
  (CR 707.10 says they should) but the copy count is fixed at cast time, so a
  creature entering in response isn't counted — which is correct — while one
  leaving still leaves its copy on the stack to fizzle.
- **Lithomancer's Focus** ships as a plain +2/+2; "prevent all damage colorless
  sources would deal to that creature" needs a prevention shield filtered by
  the damage *source's* characteristics (the existing shields are per-source-id
  or unconditional).
- ✅ ~~**Modal activated abilities pick mode 0.**~~ —
  `GameAction::ActivateAbility` now carries a `mode`; a submitted mode is
  authoritative and drives per-slot target validation, the server view lists
  each mode's short text (`AbilityView.modes`), and the client's ability menu
  expands a modal ability into one row per mode. Loyalty abilities still
  auto-pick their extra targets.
- ✅ ~~**Worldwake is half-done**~~ — `set_gaps.py wwk` is at zero (`sets::wwk2`).
- **`Effect::MoveWithinTotalManaValue` auto-picks.** March from the Tomb takes
  the cheapest matches first to maximize the count; the printed card lets the
  caster choose which cards fit the budget.

## Noticed this run (modern_decks — WWK closure)

- **Trap alternative costs are a two-sided read.** `Predicate::
  CastSpellThisTurnWith` reads `Player.spell_casts_this_turn` (colors + cast-half
  types, cleared at cleanup); an opponent who casts a spell that is *countered*
  still turns the Trap on, which matches the printed "cast" wording.
- **`Effect::ChooseNewTargetsForSpell` picks the new target itself.** CR 115.7a
  is honored (the spell moves off its original target, and the auto-pick sends a
  hostile spell away from the chooser), but the pick goes through the
  synchronous decider — a UI seat never sees a prompt. Wants the
  `CopySpellMayChooseTargets` suspend shape.
- **A combat-damage trigger still gets one target slot.** Cards whose body wants
  a slot beyond the damaged player (Sword of Sinew and Steel-style "and destroy
  up to one artifact") auto-pick via `auto_target_for_effect_avoiding_set_x`;
  `fire_combat_damage_triggers` never fills slots 1+.
- **`Effect::MayDoElse` can't see whether its body did anything.** Vapor Snare
  gates on "you control a land" before offering the bounce so a landless
  controller sacrifices; a body that resolves to nothing after an accepted "yes"
  would still count as done.
- **Marshal's Anthem / Voyager Drake reanimate-per-kick pick their own targets.**
  `CapTargetsAt` truncates the auto-filled slot list at resolution; a UI seat
  never gets to choose *which* graveyard creatures come back.

## Noticed this run (modern_decks — M11 closure / Betrayers of Kamigawa)

- **BOK is at 41 gaps.** What's left needs one primitive each:
  - Shining Shoal — the four other Shoals ship; this one also needs
    "the next X damage from a source of your choice is dealt to any target
    instead", a redirect the prevention layer doesn't express yet.
  - The Glasskites / Kira — "the first time each turn this becomes the target
    of a spell or ability, counter it" needs a per-turn-per-object trigger
    budget plus a counter-the-triggering-object effect.
  - The non-mana splice costs (Horobi's Whisper, Hundred-Talon Strike, Roar of
    Jukai, Torrent of Stone, Veil of Secrecy) — `Keyword::Splice` only carries
    a `ManaCost`.
  - Chisei, Heart of Oceans — wants `Effect::RemoveCounterOfPresentKind`, the
    mirror of the existing `AddCounterOfPresentKind`.
  - Empty-Shrine Kannushi — protection from *the colors of permanents you
    control* is a live, board-derived keyword set.
  - Fumiko the Lowblood — bushido X (scaled off attacker count) and a
    "creatures your opponents control attack each combat if able" static.
- **Genju's return trigger is modeled as the Aura's own LTB.** The printed
  wording is "when enchanted land is put into a graveyard"; the Aura's
  `PermanentDied`/`EnchantedBySource` listener doesn't fire because the
  dispatcher only walks battlefield sources and the orphaned Aura is already in
  the graveyard by then. Over-triggers if the Aura alone is destroyed.
- **Genju of the Fens drops its granted "{B}: +1/+1".** `Effect::BecomeCreature`
  grants keywords, not activated abilities.
- **Forked-Branch Garami's two soulshifts share one target.** The auto-targeter
  reuses the first legal Spirit, so the second trigger fizzles; a live seat
  would pick the other.
- **The client can't be compiled in the cloud sandbox** (`wayland-sys`'s build
  script fails with no display libs), so `crabomination_client` edits ship
  unverified. Clippy/test sweeps use `--workspace --exclude crabomination_client`.

## Noticed this run (modern_decks — ZEN / M11)

- **Blazing Torch's throw sacrifices by name.** The granted ability's
  "Sacrifice Blazing Torch" is modeled as `Effect::Sacrifice` with a
  `HasName` filter, so a second Torch you control could be the one sacrificed.
  Wants a `Selector::AttachmentGranting` sacrifice.
- ✅ ~~**Three M11 cards remain**~~ — shipped; `set_gaps.py m11` is at zero.
  Residual: Conundrum Sphinx's name prompt feeds a bot the densest library
  name (the same auto-pick residual as the rest of the `NameCard` family).
- ✅ ~~**`Effect::Search { to: Exile }` doesn't stamp `exiled_with`**~~ —
  `Effect::SearchExileLinked` runs the search chain and stamps each pick
  (CR 607.2). Plain `Search { to: Exile }` still doesn't; Hoarding Dragon's
  `ExileWithSource { LastMoved }` pairing is unchanged.
- **`Effect::RevealHandDiscardAllMatching` reveals nothing visible.** The
  discard is correct but the reveal is knowledge-only — no `hands_revealed_to`
  entry, so a UI seat never sees the hand it just stripped.

## Noticed this run (modern_decks — Planeshift closure + CR 802/803/706.8)

Planeshift is at zero `set_gaps.py` gaps (86 cards this run); Invasion went
280 → 233 (47 cards, `sets::inv::gaps`).

- **`Effect::PutCardsFromHandOnBottom` auto-picks.** Sawtooth Loon's "put two
  cards from your hand on the bottom" uses the synchronous decider, so a UI
  seat is never prompted — the same residual as
  `PutCardFromHandOnTopOfLibrary`. Both want the `ask_seat_cards` suspend, but
  they sit after a `Draw` in a `Seq`, and the stash-and-rerun resume would
  replay the draw.
- **`Effect::RevealRandomFromHand` reveals nothing visible.** The mana value is
  stamped for `Value::LastRevealedManaValue`, but nothing enters
  `hands_revealed_to` (that field is whole-hand and permanent, so it's the
  wrong shape for a single-card reveal). Wants a per-card revealed set.
- **Guard Dogs' chosen permanent is collapsed.** The printed "choose a
  permanent you control, prevent the damage if the target shares a color with
  it" is modeled as `Predicate::TargetSharesColorWithControlled` over *any*
  permanent you control — strictly more permissive.
- **Goblin Game's hidden counts aren't hidden.** `Effect::GoblinGame` asks each
  seat in turn order via `ask_seat_amount`, so a later seat could in principle
  see an earlier answer through the replay log. Simultaneous reveal wants a
  batched ask.
- **CR 806/807 seat rotation isn't modeled.** `AttackOption::{AttackLeft,
  AttackRight}` reads seat index order; Grand Melee's rotating range of
  influence (CR 807) and the limited-range option (CR 801) are still open.
- ✅ ~~**CR 600 is a bare heading**~~ — `scripts/cr_coverage.py` now drops
  sections with no numbered clauses of their own (600, 802/803 came off the
  gap list by being tested), so the untested list is 33 real gaps rather than
  34 with a phantom.
- **Invasion's remaining 233 gaps** are mostly the rare/uncommon shell: the
  Dragon legends and their Lairs' payoffs, the "most common color among all
  permanents" cycle (Barrin's Unmaking, Goham/Halam Djinn — wants a
  `Value::MostCommonColorAmongPermanents` read), Aether Rift, Bend or Break,
  Cauldron Dance (a combat-only double reanimate) and the Blind Seer /
  Atalya utility rares.
- **`R::HasChosenColorOfSource` is false in hidden zones.** The hidden-zone
  card evaluator (`eval.rs`, the `R::HasChosenColorOfSource | …` arm) returns
  `false` because it carries no source, so a hand/library/graveyard filter can
  never read the source's chosen colour. That blocks the faithful Addle
  ("choose a color … discard a card of that color"), which ships as
  `DiscardChosen` over the whole revealed hand instead. Two halves to fix:
  thread the source into that evaluator, and have
  `Effect::ChooseColorForSelf` keep the pick in per-resolution scratch (a
  resolving *spell* has no battlefield permanent to stamp).

## Noticed this run (modern_decks — Antiquities closure / Arabian Nights)

- **Arabian Nights is at 3.** Each needs one primitive:
  - Shahrazad — CR 729 subgames. `selfplay.rs` could host a bot-vs-bot nested
    game using the players' libraries as decks, but nothing models a subgame's
    zones, priority, or the "loses half their life, rounded up" payout yet.
  - Eye for an Eye — "the next damage from a source of your choice is *still*
    dealt to you and mirrored to that source's controller". `PreventionShield`
    has `reflect` (Deflecting Palm), which *replaces* the damage; the mirror
    wants a shield that passes the damage through and duplicates it, which
    means threading a non-preventing branch through `apply_prevention`.
  - Aladdin's Lamp — a one-shot draw replacement of the `DrawDig::LookN`
    shape. That dig is a permanent-scoped static (`look_instead_of_drawing`);
    there's no "replace your *next* draw this turn with a look-N".
- **Guardian Beast's "other players can't gain control" is dropped.** The
  indestructible + can't-be-enchanted halves ride `AnthemForFilterIf` with a
  `CantBeTargetedByAuras` grant; there's no control-change prohibition keyword.
- **Jihad chooses a colour but not an opponent.** The printed card names both;
  the anthem gates on *any* opponent showing a nontoken permanent of the
  chosen colour, which is strictly more permissive with three or more players.
- **Magnetic Mountain's untap toll is one prompt per creature.**
  `Effect::MayPayRepeatedly` loops the yes/no, so the printed "choose any
  number, then pay {4} for each" is a sequence of single-creature buys — the
  same total cost, but a bot that stops early leaves creatures down.
- **`Effect::AddCounterCapped` ignores counter doublers.** The clamp is
  applied against the target's live pool and then the counters are placed
  directly, so a Doubling Season would not double them (and could not push the
  total past the printed cap anyway). Correct for Clockwork Avian; revisit if
  a capped-counter card ever wants the CR 614.16 interaction.
- **Tawnos's Coffin notes counters on the exiled object itself.** The counter
  map of a card in exile is otherwise unread, so `CoffinExile` re-stamps it
  after the hop and `CoffinReturn` reads it back. A second effect that cared
  about counters in exile would see them.
- **CR 801.10 is enforced at the selector, not per-effect-clause.**
  `resolve_selector` and `resolve_players` filter out-of-range entities, which
  covers sweepers and "each player" fan-outs. CR 801.11 (a spell only *sees*
  information inside its controller's range — Coat of Arms) is not modelled:
  counting predicates still walk the whole board.
- **CR 807's rotating Grand Melee ranges are still open**, as is CR 801.5c
  ("the closest appropriate player to the left makes the choice when nobody in
  range can").

## Noticed this run (modern_decks — ARN/DRK closure, CR 315 conspiracies)

- **Grand Melee's turn markers (CR 807.4) are not modelled.**
  `set_grand_melee_variant` ships the 807.2 default options and 807.3 random
  seating, but several players taking turns simultaneously needs a turn-marker
  ring in the turn loop, plus 807.4e–i's marker removal and extra-turn rules.
- **Subgames are bot-only and bounded.** `GameState::play_subgame` (CR 729)
  pilots the nest with `RandomBot` and caps it at 4 000 actions and two levels
  of nesting; a `wants_ui` seat can't take priority inside a subgame, and a
  stalled nest counts as a draw (so every player pays Shahrazad's toll). CR
  729.4's "the subgame's ante/cards return" is modelled by never touching the
  outer zones: the nest gets fresh instances and the outer libraries are
  reshuffled on the way out.
- **Sorrow's Path's re-block skips `becomes blocked`.** The swap reuses the
  existing `block_map` entries, so CR 509.3a triggers don't fire a second
  time. That matches the printed intent but isn't derived from the rules.
- **Frankenstein's Monster's "instead of onto the battlefield" is post-hop.**
  The as-enters effect routes the permanent to its owner's graveyard rather
  than intercepting the zone change, so a leaves-the-battlefield watcher would
  see it. Fixing it properly wants `apply_as_enters_effect` to be able to veto
  the hop.
- **Remaining Conspiracy (CNS/CN2) cards.** Six of 24 are unimplemented:
  Advantageous Proclamation and Sovereign's Realm (deck-construction statics —
  the format legality pass has no hook for "your minimum deck size is reduced
  by five" or "your starting deck can't have basic lands"), Backup Plan (a
  second opening hand, drawn before mulligans), Emissary's Ploy and Unexpected
  Potential (both want a *filtered* spend-mana-as-any-color permission —
  `relax_cost_colors_for` is global, with no handle on the spell being cast),
  Echoing Boon (a copy trigger keyed on the spell's target) and Summoner's Bond
  (double agenda — two named cards per conspiracy; `named_card` holds one).
  Worldknit ships with its card-pool gate dropped: the engine has no card-pool
  concept to read.
- **Conspiracy (CNS) is opened, not closed.** 16 regular cards ship; the rest
  need one of two mechanics. **Voting** (CR 701.32 — will of the council /
  council's dilemma) has no engine support at all: it wants a seat-ordered
  poll with a public tally and a tie rule, plus a `Decision::Vote` and its
  client modal. **Draft-matters** (Cogwork Librarian, Aether Searcher, Canal
  Dredger's and Cogwork Spy's draft clauses, Agent of Acquisitions) needs the
  draft engine to expose per-pick hooks; the two cards that ship drop those
  clauses.
- **`GameState::seat_static_sources` is the seat-scoped twin of
  `all_static_sources`.** Controller-scoped statics that still walk
  `self.battlefield` directly won't see a conspiracy; the two cast gates and
  the ability-grant walks were converted, the rest were left alone. Convert
  more as conspiracies need them.

## Homelands — open at 30

`set_gaps.py hml` is at 30 (`sets::hml`, 70 cards). What's left needs real
primitives rather than more of the same:

- **Baron Sengir / Hazduhr / Daughter of Autumn** — a `+2/+2` counter kind and
  an "the next N damage to target X is dealt to this instead" redirect.
- **Coral Reef / Orcish Mine / Trade Caravan** — counters that are spent as
  activation costs on an Aura or a creature with their own per-turn windows.
- **Giant Oyster / Marjhan / Black Carriage** — "for as long as this remains
  tapped" durations tied to an activation.
- **Chain Stasis** — a copy chain each player may buy into.
- **Baki's Curse** — damage scaled per Aura attached to each creature.
- **Leeches** — poison-counter removal plus damage equal to the amount removed.
- **Truce / Prophecy / Renewal / Headstone / Jinx** — the "draw a card at the
  beginning of the next turn's upkeep" rider needs a `DelayedKind::NextUpkeep`
  (the existing one is `YourNextUpkeep`).
- **Timmerian Fiends** — ante.

## Planechase — shipped, with the corners left open

CR 311 / 312 / 901 ship (`sets::ohop`, 15 cards). Deliberately not modelled:

- **Grand Melee multi-plane** (CR 901.14) — `planeswalk` assumes at most one
  face-up plane per planar controller.
- **Single communal planar deck** (CR 901.15) — each seat has its own deck.
- **Leaving the game** (CR 901.10) — a departing owner's face-up plane isn't
  swapped out for the new planar controller's.
- Chaos abilities that want "until a player planeswalks" as a duration
  (Agyrem, Eloren Wilds) need a `Duration::UntilAPlayerPlaneswalks`, so those
  planes aren't in the catalog yet.
- Pools of Becoming's "reveal the top three of your planar deck and trigger
  each of their chaos abilities" needs a planar-deck peek effect.
- The bot takes only the turn's one free roll; it never buys a later one, and
  it doesn't evaluate whether the face-up plane favours it.
