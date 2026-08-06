# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.

## Noticed this run (FIN closed; CN2 opened)

- **CN2 is down to 12.** CNS is closed; what's left in CN2 needs three things
  the CR 905.2b shell doesn't have yet: noted **creature types** (Paliano
  Vanguard) and **keywords** (Animus of Predation); a pre-game "exile a card
  you drafted that isn't in your deck" pile (Arcane Savant, Caller of the
  Untamed, Volatile Chimera); and the four Conspiracies' hidden/double agenda
  + start-of-game rules (Echoing Boon, Emissary's Ploy, Sovereign's Realm,
  Summoner's Bond). Spy Kit wants a text-changing "has all names" grant. ⏳
- **Spire Phantasm's draft-time guess is a heuristic.** The pod notes a hit
  only when the pack it names from has one card left; a real guess wants the
  drafter to name a card and the next drafter to reveal. ⏳
- **Expropriate and Selvala's Stampede are unimplemented.** Both are
  council's-dilemma sorceries whose per-vote bodies need a "choose a permanent
  owned by the voter" slot / a you-only put-permanent-from-hand. ⏳
- **Regicide's color restriction is a resolution-time gate, not a targeting
  one.** Cast-time target validation looks the source up by id, and a spell
  mid-cast is in neither hand nor stack, so `HasDraftNotedColorOfSource` can't
  see its own name there. A `source_name` on the validation path would close
  it. ⏳
- **Canal Courier's "attack different players" unblockable clause is
  dropped**, and Expropriate / Selvala's Stampede are unimplemented: all three
  are multiplayer-shaped council effects that need a per-vote "choose a
  permanent owned by the voter" slot. ⏳
- **`Effect::AddCountersUpTo` asks once for the whole selector.** "Put up to N
  counters on each" would want a per-target ask; no printed card needs it yet,
  and non-UI seats take the maximum (it's pure upside). ⏳
- **Spectral Grasp's block half is a flat `CantBlock`** — exact at two players,
  too wide in multiplayer, where it should only stop blocks against the Aura
  controller's creatures. Wants a filtered can't-block static. ⏳
- **Grenzo, Havoc Raiser's exile mode grants a free cast, not the printed
  any-color pay.** `ExileTopAndGrantMayPlay { pay_any_color: true }` covers the
  spend-as-any-color half but the card is still cast for free. ⏳
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
- **`Effect::LookPickToHand` still spells out eleven fields at every call
  site** (~156 of them). Boxing the payload into a `LookPick` struct with
  `Default` would let card definitions use `..Default::default()` like the
  rest of the catalog. ⏳
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

## Recommender: two builder defects fixed, one lesson recorded

Both were found by asking why Emeritus of Ideation never appeared in a
build for a pool that contained it (2026-08-04). Fixed behind
`SimConfig::builder_v2`; see FEATURE_ROADMAP Tier 13 for the measured
adoption. Recorded here because the *consequence* outlived the fix:

- Every recommendation produced before this — including per-card
  attribution tables — came from a builder that could not see power,
  toughness, keywords, or a card's attached preparation spell. Re-run
  any archived recommendation before trusting its card rankings.
- `recommend_pool`'s anchor lens (`per_card_attribution_within`) reports
  over the *surviving* population only. A shape eliminated in racing
  contributes no variants, so "0 variants play it" means "no survivor is
  that color", not "the card is bad". The lens now matches anchor names
  case-insensitively — an exact-match miss used to return an empty
  subset silently, which reads identically to a real negative result.

## Missing: SOS Special Guests (SPG)

The Secrets of Strixhaven **Special Guests (SPG) sheet is not implemented
at all**: no catalog module, and no SPG slot in `generate_sos_pack`, so
SOS sealed pools and drafts never contain them. Work items:

- Catalog the SPG card list (audit against Scryfall, same as the SOS base
  set) in a `catalog::sets::sos` submodule or sibling.
- Booster collation: add the SPG slot to `generate_sos_pack` at the
  printed rarity/rate so sealed pools are honest.
- Downstream: `sos_draft_pool()` feeds the ML vocabulary
  (`Vocab::sos_sealed`, `server/encode.rs`) — adding SPG cards grows the
  vocab, which **invalidates previously trained value nets** (embedding
  rows are index-pinned; the loader's vocab-size check will refuse old
  weights, which is the correct failure). Retrain after the sheet lands.
- `recommend_pool` and the sealed ladder mirrors pick the change up for
  free once the pack generator is right.

## Client / UI follow-ups (M15 run)

- ✅ ~~**Convoke/Improvise cast UI**~~ — shipped. Right-clicking a convokable
  hand card opens the helper picker (`HelperTapState` +
  `spawn_helper_tap_modal`); confirming submits `CastSpellConvoke` (or
  `CastSpellWaterbend`) with the ticked helpers, arming the targeting cursor
  first for targeted spells. Backed by `HandAffordances.convokable` (a
  full-helper dry-run probe) plus `KnownCard.has_convoke` / `has_improvise`
  so the picker knows which permanent types can help. Residual: the picker
  lists candidates but doesn't preview how much each tap saves.
- ✅ ~~**Interactive color choice for prevention.**~~ — shipped. Avacyn's
  `PreventAllDamageFromChosenColorThisTurn` suspends on a real
  `Decision::ChooseColor` for a `wants_ui` seat
  (`PendingEffectState::PreventFromChosenColorPending`); bots keep the
  synchronous decider. Test
  `cr_615_7_chosen_color_prevention_prompts_a_ui_seat`.
- **Master of Predicaments' hand pick.** The chosen card is auto-picked
  (the mana value furthest from the line) and the guess is asked of the
  resolving decider rather than routed to the guesser's seat.

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

## Discovered engine follow-ups (claude/modern_decks)

- **Noticed but not tackled this run:**
  - `Effect::ChooseUnchosenMode` auto-picks the first unused mode for bots and
    for a `wants_ui` seat alike (it uses the synchronous decider rather than a
    suspend). A human controller should get the real modal.
  - `apply_enters_under_opponent_control` picks the first alive opponent in seat
    order instead of asking; the printed text is "an opponent of your choice",
    which matters only in multiplayer.
  - `Selector::RandomAmong` re-rolls per resolution and can pick the source
    itself; a "chosen at random" that must exclude the source would need a
    filter-side `OtherThanSource` at the call site (Goblin Test Pilot doesn't).
- **Multi-block follow-ups — CLOSED.** Engine + client both ship (the
  order/assign modals are noun-aware via `damage_recipient_noun`, reading
  `PermanentView.attacking` / `.blocking_attackers`). CR 509.3a–e is now wired
  (see the CR audit). Umezawa's Jitte does *not* over-count: its
  `DealsCombatDamageToCreature` trigger isn't in the fan-out set, so it mints
  one instance per damage sub-step.
- **RNA/DGM cards deferred, each blocked on one primitive:**
  - **Domri, Chaos Bringer** — "+1: add {R} or {G}. If that mana is spent on a
    creature spell, it gains riot." Needs mana provenance (a rider attached to
    a specific mana unit, checked at the spell it pays for). Same blocker as
    the roadmap's "mana provenance" item.
  - **Captive Audience — SHIPPED** (`CardDefinition.enters_under_opponent_control`
    + `Effect::ChooseUnchosenMode` backed by `CardInstance.modes_chosen`).
  - **Theater of Horrors** — the exile half works with
    `ExileTopAndGrantMayPlay`, but "during your turn, if an opponent lost life
    this turn, you may play cards exiled with this" needs a CONDITION on
    `MayPlayPermission` (the struct is `Copy`, so it wants a small Copy-able
    gate enum rather than a `Predicate`).
  - **Melek, Izzet Paragon — SHIPPED** (`CardInstance.cast_from_library` +
    `Predicate::CastSpellFromLibrary`; the library-top cast hops through hand,
    so the origin rides `GameState.casting_from_library_top`).
  - **Goblin Test Pilot — SHIPPED** (`Selector::RandomAmong(filter)`).
  - **Plasm Capture — SHIPPED** (`Value::CounteredSpellManaValue` +
    `AddManaAtNextMainPhase { any_color }`); **Catch // Release — SHIPPED**
    (five-type edict off existing primitives). **Reap Intellect**,
    **Flesh // Blood**, and **Legion's Initiative** shipped too — DGM is
    complete.
- **`EffectDuration::UntilNextTurn` was never expired** — fixed; both it and
  `UntilYourNextTurn { player, installed_turn }` (CR 611.2b — Amplifire) now
  clear at the untap step of the turn they name. The 18 catalog sites were
  re-read against their oracle text: all of them print a real "until your next
  turn" clause, so none wanted the old permanence.
- **Erebos's Emissary (THS) — SHIPPED** (`Predicate::SourceIsBestowedAura`
  branches the pump between the source and its host).

- **RNA batch-7 leftovers (each needs one primitive):** Persistent Petitioners' "tap four untapped
  Advisors: mill 12" (a tap-N-other-of-a-type activation cost); Rakdos, the
  Showstopper (per-creature coin-flip destroy filtered by type). Opponent-threat
  displays in `player_stats.rs` still value a High Alert/Doran wall by power
  (0), not toughness — refine when convenient. (Pestilent Spirit's I/S-spell
  deathtouch shipped in batch 9 via `StaticEffect::YourISSpellsHaveDeathtouch`.)
- **RNA batch-9 deferrals — SHIPPED** (Galloping Lizrog remove-and-double,
  Combine Guildmage turn-scoped enters-with counter, Forbidding Spirit
  `TaxAttackersUntilYourNextTurn`, Font of Agonies blood counters +
  `EventKind::PaidLife` trigger, Verity Circle `EventSpec::not_as_attacker`,
  Angel of Grace `CantLoseThisTurn{damage_floor}` + gy-recur, Rhythm of the
  Wild riot anthem via `GrantTriggeredAbility`, Rumbling Ruin low-power
  can't-block). Still open: Ravager Wurm mode 2 — "destroy a land with a
  non-mana activated ability" (a land-with-nonmana-ability target filter);
- **Multi-block — SHIPPED.** `block_map` is now blocker → `Vec<attacker>` with
  `Keyword::CanBlockAdditional(n)` / `CanBlockAnyNumber`, blocker-side damage
  division (CR 510.1e), and a bot pass that spends spare block capacity. Still
  open on top of it: "blocks two or more creatures" batch counting (CR 509.3e),
  and the client has no UI yet for assigning a multi-blocker's damage split
  (the engine suspends correctly; the panel reuses the attacker-side modal).
- **New primitives that would unblock batches of gap cards (recent274–279 run):**
  - **Enlist** (CR 702.148) — no keyword yet; blocks the DMU Enlist commons
    (Barkweave Crusher, Coalition Warbrute, Argivian Cavalier, …). `Effect::Enlist`
    exists but no `Keyword::Enlist` + attack-time tap-a-nonattacker wiring.
  - **Backup N** (CR 702.164) — no keyword; blocks the MOM Backup commons
    (Chomping Kavu, Consuming Aetherborn, Cragsmasher Yeti, Archpriest of Shadows).
    Needs an ETB "put N +1/+1 counters on target; if another creature, it gains
    this creature's abilities until EOT" primitive.
  - **Player-curse Auras** — `PlayerStaticTarget::EnchantedPlayer` + a battlefield
    permanent→player attachment link so an Aura's static/trigger can scope to the
    enchanted player. Blocks Grievous Wound (can't-gain-life + damage→lose-half).
  - **Move a battlefield permanent to owner's library top/bottom (owner choice)** —
    `ZoneDest::OwnerLibraryTopOrBottom` is a countered-spell zone only; no
    permanent-move dest. Blocks Desynchronize, Diver Skaab's exploit rider.
  - **Edict-exile (target opponent exiles a permanent of a type, their choice)** —
    blocks Debt to the Kami's modal.
  - **"If you didn't put a card into your hand this way, gain N life"** — the
    inverse of `LookPickToHand.gain_life_if_pick`. Blocks Blossom Prancer.
  - **Blitz** field exists on `CardDefinition`; wire Caldaia Strongarm-style
    creatures (ETB counters + Blitz {cost}) once verified end-to-end.

- **Single-primitive cards scoped this run (each unblocks one card):**
  - Miasma Demon (DSK) — reflexive "discard any number; when you do, up to that
    many target creatures get -2/-2" (`Reflexive` + target count = cards
    discarded this way; the count-links-targets wiring is the gap).
  - Undead Sprinter (DSK) — conditional graveyard cast gated on "a non-Zombie
    creature died this turn" + enters-with-a-counter-if-cast-from-graveyard.
  - Tin Street Gossip (MKM) — `SpendRestriction::FaceDownOrTurnFaceUp` mana.
  - Public Thoroughfare (MKM) — "sacrifice unless you tap an untapped artifact
    or land" (tap-a-permanent as an alternative-to-sacrifice cost; convoke-kin).
  - Unyielding Gatekeeper (MKM) — turn-face-up exile branching on whether the
    caster controlled the exiled permanent (blink-or-give-opponent-a-token).

- **MKM Cases shipped (`decks::recent242`, 6 + Case File Auditor); remaining Cases
  need new primitives:** Case of the Gorgon's Kiss (solved = self-animates to a
  4/4 creature — needs a "this permanent becomes a creature" static, plus a
  "3+ creature cards to graveyards this turn" solve counter), Pilfered Proof
  (solved token-replacement adding a Clue), Locked Hothouse (extra-land static +
  play-from-top-of-library static), Ransacked Lab (solve = "4+ instant/sorcery
  spells cast this turn" — no I/S-specific per-turn count predicate yet), Stashed
  Skeleton (solve = "no suspected Skeletons you control" — `SelectionRequirement::
  IsSuspected` now ships, so only the per-controller solve counter remains),
  Burning Masks (solve = "3+ sources you controlled dealt damage this turn" —
  needs a distinct-damage-source-count tracker).
- **"Sacrificed an artifact this turn" — SHIPPED** (`recent248`):
  `Player.artifacts_sacrificed_this_turn` + `Predicate::SacrificedArtifactThisTurn`
  + `SelectionRequirement::ControllerSacrificedArtifactThisTurn` +
  `self_cost_reduction_if_sacrificed_artifact` power Suspicious Detonation and
  Furtive Courier's unblockable rider. Magnetic Snuffler still needs a
  "return an Equipment card from your graveyard to the battlefield attached to
  this creature" ETB effect (no reanimate-attached primitive yet); its
  "whenever you sacrifice an artifact → +1/+1" half is a
  `PermanentSacrificed`/`YourControl` trigger filtered to `R::Artifact`.
- **Cross-permanent death-stat triggers:** "whenever a creature dies, if its
  [power/toughness] was X" on a *different* permanent (Massacre Girl) reads the
  dying creature's death-time stat correctly through the trigger **filter** (the
  death snapshot backs `R::ToughnessAtMost`, etc.), but `Value::ToughnessOf(
  TriggerSource)` in the trigger **body** resolves empty (the LKI subject is only
  set for the dying creature's own die-triggers). Prefer filter-gating such cards
  until the resolving-LKI-subject plumbing covers cross-permanent watchers.
- **Collect evidence as an activated-ability cost:** ✅ shipped —
  `ActivatedAbility.collect_evidence_cost: Option<u32>`, pre-flight-gated on
  `graveyard_can_collect_evidence` and paid through the shared
  `collect_evidence_from_graveyard` exile path (emits
  `GameEvent::EvidenceCollected`). Forensic Researcher is fully modeled. Hedge
  Whisperer still blocked only on the "target land becomes a 5/5 *for as long as
  this creature remains tapped*" conditional land-animation duration (a
  source-tapped-gated continuous grant — no primitive yet).
- **Worldsoul's Rage ✅** (`decks::recent262`) — `Effect::DeployLandsFromHandAndGraveyard`
  (put up to X land cards from hand and/or graveyard onto the battlefield tapped,
  graveyard-first) + X damage to any target. Reusable for future "put lands from
  hand/graveyard" ramp.
- **Ill-Timed Explosion ✅** (`decks::recent262`) — `Value::GreatestDiscardedManaValueThisEffect`
  (greatest MV among cards discarded this resolution; maxed per discard). Draw
  two, may discard two, deal that MV to each creature. Reusable for other
  "greatest MV discarded this way" payoffs.
- **MKM remaining gaps (~50 cards):** legends (Delney, Etrata, Teysa, Judith,
  Kaya PW, Tolsimir's Wolf-attack lure, …), the remaining split cards (Flotsam //
  Jetsam, Push // Pull, Hustle // Bustle, Fuss // Bother ✅, Cease // Desist ✅),
  Disguise/Cloak value (Coveted Falcon, Fugitive Codebreaker), the reanimators
  (Relive the Past, Anzrag's Rampage), Krenko's Buzzcrusher (per-player land
  destruction + fetch), Officious Interrogation (per-target cost + investigate X),
  and the remaining lands (Public Thoroughfare, Branch of Vitu-Ghazi).
  `scripts/set_gaps.py mkm` lists them. Notable primitives still blocking cards:
  - **Wolf-attack lure** (Tolsimir) — "target creature blocks *that Wolf* if
    able" needs a MustBlock variant pointing at the trigger source, not the
    ability source (`MustBlockSource` binds `ctx.source`).
  - **Reflexive gy-target return** (Blood Spatter Analysis) — "sacrifice this if
    5+ bloodstain; when you do, return target creature card from your graveyard"
    needs the return target chosen only when the sacrifice fires, not every death.
    Also needs a Bloodstain counter type + a "whenever one or more creatures die,
    mill + add a counter" trigger.
  - **Tenth District Hero** — first ability is ready (`collect_evidence_cost` +
    `BecomeCreature` sets 4/4 Detective + vigilance); second ability blocks on a
    rename + "Other creatures you control have indestructible" anthem granted by
    a self-becomes effect.
  - **Sudden Setback** — "put target spell or nonland permanent on library, owner
    chooses top/bottom" needs a spell-or-permanent target (the `Target` enum has
    no Spell variant) + a library-owner-choice move effect.
  - **Tin Street Gossip / Goblin Maskmaker** — restricted / discounted mana for
    face-down casts needs a face-down-spell spend restriction + cost reduction.

- **FDN/DSK gap cards shipped (`decks::recent202`–`recent205`, 20):** Rite of the
  Dragoncaller, Koma World-Eater, Niv-Mizzet Visionary, Perforating Artist, Kiora
  the Rising Tide, Soulstone Sanctuary, Lunar Insight, Valkyrie's Call, Infernal
  Vessel, Fiery Annihilation, Violent Urge, Elenda Saint of Dusk, Quilled
  Greatwurm, Saw, Unable to Scream, Sporogenic Infection, Under the Skin, Don't
  Make a Sound, Keys to the House, Osseous Sticktwister. Approximations left:
  Fiery Annihilation's exile-attached-Equipment rider, Quilled Greatwurm's
  graveyard-cast, Elenda's hexproof-from-instants, Sporogenic Infection's
  "other than enchanted" sacrifice clause, Don't Make a Sound's reflexive
  surveil-2, Keys to the House's Room lock/unlock mode. Remaining FDN/DSK gaps
  needing new primitives: Drake Hatcher / Nine-Lives Familiar (incubation /
  revival counter types), Banner of Kinship (choose-type + fellowship-counter
  anthem), Alesha (reanimate MV ≤ source power), Tinybones / Abyssal Harvester
  (stash / gy-exile copy), Kykar (modal cast trigger), Zimone (double each kind
  of counter on up-to-2 targets), Miasma Demon / Orphans of the Wheat (discard-
  any-number / tap-any-number variable counts), Creeping Peeper
  (enchantment-only spend restriction).
- ✅ **Punisher empty-hand discard affordability** — `punisher_option_affordable`
  now rejects an `Effect::Discard` dodge when the chooser holds fewer cards than
  the discard count (CR 601.2 — can't choose a cost you can't pay), so a
  hand-empty opponent takes the penalty instead of "discarding" nothing
  (Perforating Artist's Raid, and every "loses N unless … or discards a card").
- ✅ **Punisher per-defaulter payoff target (CR 601.2b)** — the `otherwise`
  branch now binds the defaulting chooser as `PlayerRef::Triggerer`, so a
  per-chooser payoff ("deals 3 damage to each opponent who didn't") hits only
  the player who failed to pay, not every opponent — correct in multiplayer, not
  just 1v1 (Zoyowa Lava-Tongue; recent290). Cards whose chooser already *is* the
  triggerer (Painful Quandary) are unchanged.
- ✅ **`Value::DistinctManaValuesAmongControlledNonland`** — count of distinct MVs
  among nonland permanents you control (Lunar Insight).
- ✅ **`YourControl` self-death event-amount** — the SBA `die_triggers` push now
  sets `trigger_event_amount_scratch` and threads `.event_amount(mv)` from the
  dying creature's MV, so a self-death event-amount-relative target filter
  (`ManaValueLessThanEventAmount` — Jackdaw Savior's own death) enumerates and
  resolves against the dead MV. Whole suite green.
- 🟡 **Aristocrats self-death scope audit** — fixed Zulaport Cutthroat, Cruel
  Celebrant, Vengeful Bloodwitch (`AnotherOfYours`→`YourControl`; their oracle is
  "this *or* another creature you control dies", so their own death now drains).
  Both self-death funnels (the SBA lethal-damage `die_triggers` push **and** the
  destroy/sacrifice `remove_to_graveyard_with_triggers` path) now evaluate the
  trigger's `.with_filter` against the dying creature (bound as `TriggerSource`
  via the death snapshot), so a *filtered* `YourControl`/`AnyPlayer` "this or
  another [type] you control dies" trigger fires on self-death only when the
  source matches. Remaining (card work): sweep the ~49 `AnotherOfYours`
  CreatureDied cards and switch any whose oracle includes "this" to
  `YourControl` after verifying each against Scryfall.
- ✅ **`recent193`–`recent198` (OTJ/DSK/BLB/FDN, ~27 cards)** — recent193:
  Jackdaw Savior, Clement, Soul-Shackled Zombie (`PermanentEntered`→MV in
  `event_amount_for`; `ExileUpToNFromGraveyards` stamps `last_moved_cards`).
  recent194: Double Down, Mystical Tether, High Noon. recent195: Malcolm,
  Reach for the Sky, Tomb Trawler, Steer Clear. recent196: Slickshot
  Vault-Buster, Throw from the Saddle, Shepherd of the Clouds, Sheriff of Safe
  Passage. recent197: Seize the Secrets (new `self_cost_reduction_if_crime`),
  Take for a Ride, Silver Deputy. recent198: Baseball Bat. Plus the OTJ Desert
  painland cycle completion (7 lands) and Spree Final Showdown + Jailbreak
  Scheme. **Approximations:** Take for a Ride's crime-flash and Mystical
  Tether's flash-for-more riders dropped (no conditional-flash primitive).
  **Noticed:** the `AutoDecider` declines every `Decision::SearchLibrary`
  (`Search(None)`) — search ETBs are no-ops under pure auto-play; the bot has
  its own picker, but tests must script the pick. Enchant-**player** Auras
  (Grievous Wound) are unsupported — Auras only attach to permanents.
- ✅ **Enters-as-a-choice-of-stats** — `CardDefinition.enters_as_choice`
  (`Vec<EntersChoiceMode>`) is an as-enters replacement (CR 614) applied in
  `apply_enters_as_choice` before the first SBA sweep, so a printed `*/*` body
  never dies as a 0/0. The controller picks via a `ChooseMode` decision.
  Corrupted Shapeshifter (MH3) shipped.
- 🟡 **MH3 gaps still open** (`python3 scripts/set_gaps.py mh3`). Shipped since:
  the `{C}`-spent predicate (Drowner, Wumpus), Propagator Drone, Path of
  Annihilation, Deem Inferior, Snow-Covered Wastes, Imskir Iron-Eater
  (`Value::HalvedRoundDown`), Bespoke Battlewagon (energy Vehicle), Monstrous
  Vortex (`Effect::Discover`), Aether Revolt
  (`StaticEffect::NoncombatDamageToOpponentsBonus`), Idol of False Gods
  (`StaticEffect::SelfHasKeywordWhileCountersAtLeast`), Spymaster's Vault
  (targeted connive-X), Monumental Henge (dig-for-historic), Inventor's Axe
  (`CardDefinition.equip_energy_cost`), Emissary of Soulfire (exalted counters
  modeled as permanently-granted `exalted()` via `Effect::GrantTriggeredAbility`
  now honoring `Duration::Permanent`), Winter Moon
  (`StaticEffect::MaxOneNonbasicLandUntap`), Cursed Wombat
  (`StaticEffect::CounterAmplifierOncePerTurn` — once-per-turn per-permanent
  +1/+1 amplifier), Rush of Inspiration (energy modal DFC), Rosecot Knight
  (ETB dig for artifact/enchantment). **mh3d batch (20 cards) shipped:** Depth
  Defiler (`CastSpellWasKicked` choose-one/both), Expel the Unworthy
  (kicker-widens-target), Collective Resistance (mana-Escalate — fixed the
  `Escalate` cost overflow), Twisted Riddlekeeper + Herigast (Emerge, now used),
  Ugin's Binding, Abstruse Appropriation, Dog Umbra, Thief of Existence,
  Amphibian Downpour, Ondu Knotmaster // Throw a Line, Hydroelectric Specimen,
  Eladamri, Party Thrasher, Suppression Ray, Bloodsoaked Insight, Genku,
  Charitable Levy (`Predicate::SourceHasCountersAtLeast`), Emperor of Bones,
  Ripples of Undeath. **mh3e batch (12 cards, `sets::mh3e`, tests `tests/mh3e.rs`)
  shipped:** Vega (`SpellNotCastFromHand` trigger), Chthonian
  Nightmare (`ActivatedAbility.energy_x_cost` — pay X {E}, reanimate MV-X),
  Glimpse the Impossible (impulse-3 + per-card end-step Spawn), Argent Dais
  (`Predicate::AttackedWithCountAtLeast` + AnyPlayer attack observers), Lethal
  Throwdown (modal additional-sac + conditional draw), Jolted Awake
  (`Effect::PayEnergyValue`), Volatile Stormdrake (`Effect::PayEnergyOrElseValue`
  + ExchangeControl auto-target fix), Planar Genesis
  (`Effect::LookTopDeployLandOrHand`), Pyretic Rebirth (gy-return + MV burn),
  Reiterating Bolt (base bolt), Unstable Amulet (energy ETB + `SpellNotCastFromHand`
  ping + impulse), Izzet Generatorium (`StaticEffect::EnergyGainBonus` +
  `Player.energy_spent_this_turn`/`GameState::spend_energy` +
  `Predicate::EnergyPaidThisTurnAtLeast`). **Since:** Volatile Stormdrake now has
  `Keyword::HexproofFromAbilities` (CR 702.11d — opponents' abilities can't target
  it) and Reiterating Bolt has `Keyword::ReplicateEnergy(3)` (energy-paid Replicate,
  copy-per-payment). Still open, each needing one primitive:
  optional Exert + haste-if-spent-on-creature mana (Arena of Glory);
  alt-cost-by-energy permission (Primal Prayers); a "may reveal + else +1/+1
  counter" look-top rider (Rosecot Knight);
  two-independent-kickers (Wastescape Battlemage); the real Sundering Eruption //
  Volcanic Fissure (name collides with an existing fabricated `sundering_eruption`
  in `decks::modern` — replacing it means rewriting that card's two tests);
  sacrifice-count-driven search (The Hunger Tide Rises IV).
  **Other MH3 gaps worth doing next (existing-primitive-friendly):** Nissa's
  Pilgrimage (search-2-basics-split-to-bf+hand + spell-mastery-to-3 — needs a
  split-destination search), Powerbalance (opponent-cast → reveal-top free-cast
  if same MV), Baru, Wurmspeaker (Wurm anthem + cost-reduction-by-greatest-power),
  Shilgengar (Blood-sac engine + mass finality reanimate), Echoes of Eternity
  (colorless-trigger doubler + copy-colorless-spell-on-cast). Card-level
  approximations are noted on each mh3d/mh3e factory doc comment (Party Thrasher
  plays both exiled cards; Ripples has no {1}+3-life gate; Dog Umbra drops the
  opponent-control rider; Emperor drops the counter reanimation; Herigast drops
  the emerge-granting static; Pyretic Rebirth/Jolted Awake model "up to one"
  targets as required).
- ✅ **Nested modal after a payment picks at resolution** — `pick_trigger_mode`
  now unwraps reflexive-payment wrappers (`MayDo`/`MayPay*`/`PayEnergy*`) via
  `governing_modal` and stamps `MODE_PICK_DEFERRED`, so a modal buried behind a
  payment owns its own pick *after* the payment succeeds (CR 603.7): a UI seat
  gets the client modal, a bot/scripted decider decides at resolution (Voltstorm
  Angel's combat modal — test `voltstorm_angel_nested_modal_picks_second_mode`).
- ✅ **Reflexive "when you do" targets chosen after the payment** — Riddle Gate
  Gargoyle's on-attack `pay {E}{E}. When you do, target a creature gains lifelink`
  now wraps the payoff in `Effect::Reflexive`, so the target is picked at
  resolution *after* the {E}{E} is paid (CR 603.7).
- ✅ **Emerge wired** — `CardDefinition.emerge` + `emerge()` shortcut +
  `CastSpellAlternative` cost reduction by the sacrificed creature's MV
  (Wretched Gryff, Twisted Riddlekeeper, Herigast; CR 702.119 test in
  `tests/cr_rules.rs`).
- ✅ **No-mana-cost marker** — `CardDefinition.no_mana_cost` (replaces
  `suspend_only`; serde alias kept) rejects the pay-the-cost cast path per
  CR 601.3e; Ancestral Vision / Lotus Bloom / Crashing Footfalls / Living End
  stamped (they were castable from hand for free). `{0}` stays castable.
- ✅ **Echo control-change window** — `GameState::change_control` funnel
  (all steals/exchanges/reverts) resets `echo_paid` (CR 702.29b) and applies
  CR 302.6 summoning sickness in one place.
- ✅ **Echo pay prompt** — a `wants_ui` controller gets a real echo trigger
  (`Effect::EchoPayOrSacrifice`) with a pay-or-sacrifice ask; payment
  auto-taps. Bots/tests keep the synchronous path.
- ✅ **Yusri's ChooseAmount** now suspends for a `wants_ui` seat
  (`AmountAnswerPending`), same shape as `SacrificeAnyNumber`.

- ⏳ **Noticed this run (recent110/111 sweep):**
  - **Counter-placer attribution** still open (see All Will Be One entry) —
    `GameEvent::CounterAdded` has ~55 construction sites; a `placed_by`
    field is mechanical but wide.
  - **Skipped cards needing a primitive each:** Lightning Storm (any-player
    stack-only activated ability), Tibalt's Trickery (random 1–3 mill +
    exile-until-different-name free cast), Bottled Cloister (end-step hand
    exile / upkeep return), Cenn's Tactician (counter-gated multi-block),
    Nourishing Shoal (pitch-X alt cost reading the pitched card's MV),
    Prismatic Strands (prevent-by-color + tap-white-creature flashback
    cost), Abundance (draw-replacement dig), Experimental Frenzy
    (can't-play-from-hand static + top-of-library play), Mycosynth Lattice
    (all-colorless + spend-any halves). (Pili-Pala / Phyrexian Unlife /
    Salvage Titan / Qasali Ambusher shipped in `recent112` — {Q} costs via
    `ActivatedAbility.untap_self_cost`, `ControllerDoesntLoseFromLife`.)
  - **Approximations to revisit:** Tidebinder Mage's lock is a one-shot
    `SkipNextUntap` (printed: while you control it); Hypergenesis dumps all
    hand permanents at once (printed: alternating one-at-a-time loop);
    Molten Psyche's metalcraft burn reads the first opponent's draw count
    (exact in 1v1); Loaming Shaman shuffles the whole graveyard (printed:
    any number of target cards); Hurkyl's Recall bounces artifacts the
    target *controls* (printed: owns); Emrakul's cast-trigger mind-control
    turn unmodeled; Oath of Nissa's planeswalker any-color rider unmodeled;
    Balance auto-picks keeps (a wants_ui picker would be faithful).

- ✅ **`FromYourGraveyard`-scoped triggers fired from the battlefield too.**
  The battlefield gather didn't exclude the scope, so a Bloodghast-class
  trigger could fire while its card was in play (Voidwing Hybrid would have
  bounced itself). Fixed: the battlefield walk skips `FromYourGraveyard`;
  only the graveyard walk gathers them.
- ✅ **Poison placement now has one funnel.** `GameState::add_poison` routes
  AddPoison / AddCounter(Player) / proliferate / infect / toxic combat, with
  CR 614.16 scaling and Melira's `PoisonCappedAtOnePerTurn` cap applied in one
  place (proliferate poison previously skipped Constrictor scaling).
- ✅ **Equipped-state anthem filters live-resolve.** `GrantKeyword`/`PumpPT`
  statics over `IsEquipped`/`EquippedByAtLeast` join the `IsModified`/
  `IsAttacking` per-recompute `Specific` path (`requirement_needs_live_
  resolution`) instead of being silently dropped (Hexgold Hoverwings, Kemba).
- ✅ **Self-ETB trigger `EventSpec.filter` was dropped.** The inline
  spell-resolution path (`stack.rs`) collected `SelfSource` `EntersBattlefield`
  triggers by kind+scope only, discarding `event.filter`, so filtered self-ETB
  triggers (Corrupted, kicker/bargain-gated ETBs) fired unconditionally. Fixed:
  the collection now carries the filter and the execution loop re-evaluates it
  once the source is on the battlefield (CR 603.4), building a context that
  carries the cast-mode flags (`kicked`/`bargained`/`cast_from_hand`/mayhem) so
  cast-property intervening-ifs still read true. (Attack/etc. SelfSource triggers
  already went through the general dispatch, which evaluated filters.)

### Enchantress package follow-ups (recent114)
- **`EquipScale` breadth** — the P/T-per-count scale only counts the
  *controller's* battlefield and can't honor `OtherThanSource`, so "for each
  other enchantment on the battlefield" (Ancestral Mask) and "per card in your
  hand" (Empyrial Armor) aren't expressible. Add an `all_players` flag + a
  hand-count source, then wire those two Auras.
- **`ExtraManaKind::AnyColor`** — Fertile Ground / Market Festival / New
  Horizons want "add one/two mana of any color" on a triggered land-tap. Needs
  either a wildcard mana token or a player choice at the trigger; deferred.
- **Karmic Justice** — needs an event for "a spell/ability an opponent controls
  destroys a *noncreature* permanent you control" (destroyer + victim-type).
- **Aura re-attach riders** — Shielded by Faith / Ajani's Chosen's "attach to a
  creature that enters" clauses are dropped; want a `MayAttachOnCreatureEnters`.
- **Calix combat-copy** — the "copy a nonlegendary enchantment once per turn on
  combat damage" half is dropped; the constellation +1/+1 is modeled.

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

## Environment note

The `crabomination_client` (Bevy GUI) needs system libs the base image lacks.
They install cleanly via apt in the routine environment:
`apt-get update && apt-get install -y libwayland-dev libasound2-dev
libudev-dev libxkbcommon-dev`. After that `cargo build/test -p
crabomination_client` compiles (first build ~6 min). The GUI still can't be
*run* headless (no GPU/display — see the `verifier-client` skill), but client
code and its unit tests now compile and test here.

Two gotchas seen this run: `apt-get install` without a preceding `apt-get
update` 404s on a stale index (the `.claude/hooks/install-client-deps.sh` hook
already does the update, but its failures are silenced, so check
`pkg-config --exists wayland-client` before blaming the crate); and a full
`cargo test --workspace` including the client can fill the disk — `rm -rf
target/debug/incremental` reclaims several GB without a full rebuild.

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

## Engine correctness audit — 2026-06-11

Five-reviewer deep pass over the engine core (`game/mod.rs`, `effects/`,
`actions.rs`/`affordances.rs`, `stack.rs`/`combat.rs`/`layers.rs`/`types.rs`,
`crabomination_base`). Every finding was verified against call sites; known
approximations already logged elsewhere in this file were excluded. Line
numbers are as of commit `683d1416` — re-grep before fixing.

Two recurring failure modes generated most of these (see the P3 root-cause
items): effect arms **bypassing the rich centralized funnels** (death /
discard / zone-move / damage) for a bare cheaper helper, and **parallel
hand-maintained walkers drifting apart** with no exhaustiveness guard.

### P0–P1 — resolved (2026-06-11 audit)

All P0 (game-deciding / state-corrupting) and P1 (rules-visible) findings from
the five-reviewer pass are fixed and regression-tested. Per-finding detail (call
sites, CR clauses, test names) was elided in a compaction pass — recover it from
`git log -p -- TODO.md`. Classes closed: blocked-attacker-stays-blocked (510.1c),
trigger fizzle vs re-target (608.2b), cast-pipeline atomicity (`cast_atomically`),
pump-duration respect, the death-funnel-bypass family, life/draw/damage
replacement coverage, real coin-flip RNG, non-combat wither/infect/deathtouch,
per-source combat-damage aggregation, layer timestamps, and the hybrid-mana
solver. The two recurring root causes (effect arms bypassing the rich funnels;
parallel hand-maintained walkers drifting) are tracked in P3 below.

### P2 — open

- 🟡 **Deck-out loss is applied too eagerly (CR 104.3c / 704.5c).**
  `lose_to_empty_draw` sets `eliminated = true` *inside* the failed draw, so a
  player who is decked mid-resolution is immediately excluded from
  `resolve_players(EachOpponent/EachPlayer)` (which filters on `is_alive()`).
  A spell that decks an opponent and then references "each opponent" in the
  same resolution wrongly skips them (surfaced building Consumed by Greed;
  worked around in the test by giving the opponent a library). The deck-out is
  a state-based action and should be deferred to the next SBA sweep. A clean
  fix (flag `pending_deck_loss`, promote in `check_state_based_actions`) was
  prototyped but reverted: it also makes the decked player's permanents
  correctly leave via CR 800.4a, which breaks ~24 unrelated feature tests that
  cast a draw/looter spell into an empty `two_player_game()` library and rely
  on the caster's board persisting. Landing it needs those tests to seed a
  library first (or a shared harness with non-empty libraries).

### P2 — performance

- 🟡 **Uncached layer recomputation is the dominant engine cost.**
  Largely addressed via `GameState::with_frozen_layers` — a scoped,
  lazily-filled memo of the gathered continuous-effect set (sound by
  construction: the closure only holds `&GameState`; clones reset to
  unfrozen, so bot dry-runs stay correct). Frozen scopes now cover
  `resolve_selector` (every `EachPermanent`/`ControlledBy` filter),
  `legal_attackers`/`legal_blockers`, the bot's `pick_blocks`, the full
  client-view projection (`project_for`), and
  `damage_prevented_by_protection`. Test
  `frozen_layers_match_unfrozen_computation`. (A global generation-counter
  dirty-flag cache was rejected: `GameState` fields are mutated directly
  throughout tests/server, so invalidation can't be guaranteed.)
  Remaining: within a frozen scope `compute_battlefield` still re-applies
  layers per call (`apply_layers` over all permanents per blocker in
  `legal_blockers`); hoist `&[ComputedPermanent]` snapshots there if
  profiles still show it.
- ✅ **`static_str_serde::intern` leaks unboundedly**
  (`crabomination_base/src/static_str_serde.rs:38` via `tokens.rs:47`).
  Bare `Box::leak` with no dedup table, called once per token mint —
  including bot dry-run simulations — despite the module doc claiming the
  leak is bounded by unique names. Add the `HashSet<&'static str>` table.
- 🟡 **Affordance probing clones the world per candidate**
  (`affordances.rs`). `compute_hand_affordances` now builds **one**
  library-stripped template per sweep and threads it through every
  category's `_on` variant; keyword-gated categories (buyback / dash /
  blitz / …) pre-filter to matching hand cards before any dry-run.
  Remaining: each candidate still pays one `template.clone()` +
  `perform_action` dry-run — a non-mutating `validate_action` path would
  eliminate the per-candidate clone entirely (large refactor; only worth
  it if profiles show view projection hot).

### P3 — structural root causes (fix once, prevent the class)

- ✅ **Three battlefield→graveyard exits with divergent semantics** — the
  bare helper is now `remove_from_battlefield_to_graveyard_raw` with a
  doc warning (zone change only; use `remove_to_graveyard_with_triggers` /
  `sacrifice_one`). Strict Proctor's unpaid-tax sacrifice routes through
  `sacrifice_one`; the rich funnel also collects Equipment-granted dies
  triggers (CR 702.6e — Skullclamp via Destroy/sacrifice, not just SBA).
- 🟡 **Parallel hand-maintained walkers** — guard test
  `cr_601_2c_every_catalog_target_filter_is_surfaced` now serde-walks every
  catalog effect for `TargetFiltered` slots and asserts
  `target_filter_for_slot_in_mode_kicked` surfaces each one (caught + fixed
  `DiscardChosen` / `ManaClash` holes; ChooseN gets a cast-time fallback
  filter). `evaluate_requirement_static` no longer `unreachable!`s on
  zone-agnostic atoms (HasSpellSubtype/HasEnchantmentSubtype/…) — it delegates
  to `evaluate_requirement_on_card` against the located card. Remaining: the
  printed-vs-computed combat checks still lack guards, and the two requirement
  walkers should be unified rather than kept in delegation lockstep.
- ✅ **Card-name-keyed hack tables inside a ~720-line god function**
  (`gather_continuous_effects_inner`) — all retired. Werebear / Elvish
  Reclaimer / Honor Troll / Tenured Concocter / Ulna Alley Shopkeep ride
  `PumpSelfIf`, Thornfist Striker / Comforting Counsel ride `PumpTeamIf`,
  and the Judgment Incarnation cycle now prints
  `StaticEffect::GraveyardAnthem { land_type, keyword }` (zone-special:
  gathered from graveyards), so copies of every one keep their statics.
  CDA P/T rows were already the typed `DynamicPt` enum.
- ✅ **`StackItem::Trigger` literal push sites** — `TriggerPush` builder
  (`types.rs`) replaces all 26 hand-written 11-field literals; rider
  fields default and are set only where the site has a value.

## Follow-ups noticed (not yet done)

- ⏳ **Noticed this run (recent264 MOM/BRO batch):**
  - **Tapped token creation** — `Effect::CreateToken` has no `enters_tapped`
    flag (only `CreateTokenCopyOf` does), so "create a *tapped* Powerstone"
    (Argothian Opportunist, Koilos Roc) and similar tapped-token cards can't be
    modeled faithfully. Add a `tapped` field to `Effect::CreateToken`.
  - **Three-way library split on look** — `Effect::LookPickToHand` bottoms OR
    graveyards the rest, not "one to hand, one to graveyard, one to bottom"
    (Moment of Truth). Wants a per-pile routing look effect.
- ⏳ **Noticed this run (recent80 primitive batch):**
  - **Champion** (`Effect::Champion`) auto-picks the lowest-power creature to
    exile; the printed "you may instead sacrifice this" decline + a `wants_ui`
    picker is a follow-up.
  - **Run Away Together** stays "any two creatures" — a `distinct_controllers`
    flag on `Effect::ApplyToTargets` (enforced at cast-time targeting) would make
    it and similar "different players" spells faithful. (Deferred: the flag would
    have to be threaded through all 84 `ApplyToTargets` construction sites.)
  - **Goblin Recruiter** "any number" is capped at 10 via `SearchUpToN`; a true
    unbounded search-to-top would need an "any number" search count.
- ⏳ **Noticed this run (recent84–89, chosen-type/tribal batches):**
  - **Herald's Horn upkeep reveal** — the "look at top card; if it's a chosen-type
    creature, may reveal it to hand" rider is dropped (cost-reduction half is
    faithful). Wants a "top-card-of-chosen-type" reveal effect.
  - **Still-missing tribal payoffs needing new primitives:** Brass Herald
    (ETB reveal-4, keep chosen-type creatures), Belbe's Portal (put a chosen-type
    creature from hand onto the battlefield), Kindred Charge (token-copy each of
    your chosen-type creatures), Shared Animosity (attack: +1/+0 per other
    attacker sharing a type — a per-attacker shared-type count), Mirror Entity
    (set your team's base P/T to X + grant all types), Kindred Summons / Kindred
    Dominance (cast-time creature-type choice on a spell with no permanent to
    stamp `chosen_creature_type`).
- ⏳ **Noticed this run (recent81–83 batches):**
  - **Auto-targeter ignores target slots embedded in a `Value`.** A trigger
    whose only target lives inside `Value::PowerOf(Selector::TargetFiltered{..})`
    (e.g. Wall of Reverence's "gain life equal to the power of target creature
    you control") isn't auto-targeted, so it resolves as 0. Wall of Reverence is
    modeled with `Value::GreatestPowerControlledMatching` to sidestep this;
    the general fix is to walk `Value` trees for target slots in the auto-target
    candidate scan. Would also make Ballista Squad's "attacking or blocking"
    restriction expressible once an `IsAttacking`/`IsBlocking` requirement exists.
  - **`AutoDecider` declines every `Effect::MayDo`/`OptionalTrigger`,** so
    pure-upside "you may draw / gain / untap" triggers must be modeled as direct
    effects to fire under the bot-less test decider (the *bot* decider already
    accepts beneficial ones via `optional_trigger_beneficial`). Snake Umbra /
    Curious Obsession / Renewed Faith / Fecundity all use direct effects for this
    reason. A test-friendly "accept clearly-beneficial MayDo" AutoDecider policy
    would let those cards keep the printed "may" without breaking tests.
  - ✅ **Chosen-type *event* predicate** — `Predicate::TriggerObjectIsChosenType`
    matches an event subject's creature types against the source's
    `chosen_creature_type` (Changeling satisfies any). Ships Vanquisher's Banner's
    cast-of-type draw (now faithful), Kindred Discovery (enters/attacks → draw),
    and Door of Destinies (`AnthemForChosenType.per_counter` counter-scaled
    anthem + cast-of-type charge counter). Herald's Horn's chosen-type upkeep
    reveal still wants a top-card-of-chosen-type check.
- ✅ **Auto-targeter `Not(Land)` normalization.** All catalog target filters
  using `Not(Box::new(Land))` were rewritten to the canonical
  `SelectionRequirement::Nonland` (5 sites: modern.rs ×4, tla.rs ×1), so they
  auto-target correctly under the bot/auto-target path. Prefer `Nonland` in new
  card defs.

- ⏳ **Flash-loyalty client affordance.** Engine ships `CardDefinition.flash_loyalty`
  (CR 606.3b — The Wandering Emperor activates loyalty at instant speed the turn
  it enters). The client's loyalty-activation affordance should surface those
  abilities while the flash window is open (any priority), not only at sorcery
  speed. Engine + server (bot) paths are wired; only the client highlight is a
  follow-up.
- ⏳ **Prototype (CR 702.160) follow-ups.** The mechanic + 15 BRO cards ship
  (`CardDefinition.prototype` + `GameAction::CastPrototype`). Client click casts
  the prototype face only when the full cost is unaffordable; a modifier
  (Shift-click) to choose the prototype face when *both* are affordable is a
  follow-up. Deferred BRO prototype cards need primitives the engine still
  lacks: Hulking Metamorph (enter-as-copy with prototype P/T), Arcane Proxy
  (exile-and-cast I/S with MV ≤ power from gy), Woodcaller Automaton (untap +
  animate a land), Rootwire Amalgam (X/X token = 3× power), Forgefire/Warzone
  (perpetual). The bot always prefers the cheapest legal line; no value-eval of
  full-vs-prototype.
- ⏳ **Cast-time modal choice (CR 601.2b) for "choose two of four" cards.**
  `Effect::ChooseN` resolves the mode pick at *resolution* via the decider, so
  per-mode targets for an arbitrary pick can't be supplied at cast. The five STX
  guild Commands (Silverquill/Lorehold/Witherbloom/Quandrix/Prismari) therefore
  still resolve two fixed default modes. Real fix: choose modes during casting
  and gather each chosen mode's targets then (also unblocks Sublime Epiphany's
  mode-pick UI for arbitrary combinations). Oracle modes captured 2026-06-19.
- ✅ **Veil of Summer's hexproof-from-blue/black rider** — `Keyword::
  HexproofFromColor` + turn-scoped `Player.hexproof_from_colors_this_turn`
  (`Effect::GrantHexproofFromColorThisTurn`); gates spell *and* opponent-ability
  targeting of you and your permanents.
- ⏳ **Conditional-keyword statics beyond P/T** — `PumpSelfIf.keywords` covers the
  self case (Bloodghast's opp-≤10 haste). A team/granted conditional-keyword
  static (e.g. "creatures you control gain X while …") would generalize it.
- ⏳ **CHK cards/primitives deferred:**
  - `Effect::ApplyToTargets` now does "do X to each of up to N targets" — Yosei's
    "tap up to five target permanents that player controls" could be remodeled
    on it (filter `ControlledBy(targetPlayer)`), as could other "up to N" cards
    across sets (Frost Breath, Aether Tradewinds-style multi-bounce, etc.).
  - Pious Kitsune / Eight-and-a-Half-Tails devotion-counter conditional payoff.
  - Yosei taps **up to five** target permanents (modeled as tapping all of the
    target player's board); a true "up to N target permanents that player
    controls" clause needs the Tier-2 "up to N targets" work.
  - Sosuke's Warrior-damage destroy is **immediate** (printed "at end of
    combat"); wants a delayed end-of-combat destroy trigger.
  - Genju aura cycle (animate-a-land aura that returns to hand when the
    creature dies), Honden cycle's "Pious Kitsune / Eight-and-a-Half-Tails"
    devotion-counter conditional.
  - Kamigawa cards skipped this run for want of a primitive:
    Sokenzan Renegade / Kiyomaro
    (hand-size-gated keyword grants + "player with most cards" predicate);
    Takeno, Samurai General (anthem scaled by each Samurai's bushido total);
    Sachi, Daughter of Seshiro (granting "Shamans you control have {T}: Add
    {G}{G}" — group-granted mana ability).
  - Generalize "target player discards" auto-targeting so an ETB
    `Discard { who: Player(Target(0)) }` picks an opponent (Kemuri-Onna is
    modeled as `EachOpponent` to sidestep this).
  - Cranial Extraction (name a card → exile all copies from gy/hand/library);
    Cut the Tethers (per-Spirit
    "return unless pay {3}"); Petals of Insight (look-3, bottom-or-draw with
    conditional self-return); Devouring Greed / Devouring Rage (additional-cost
    "sacrifice any number of Spirits" that scales the spell — needs cast-time
    variable sac feeding `Value`).
  - Generalize `Player.zuberas_died_this_turn` into a type-filtered
    died-this-turn count if another tribe ever needs it.
- 🟡 **Bot: general value-activated-ability generator.** `pick_removal_ping`
  fires single-target "{cost}: deal damage to any target" abilities that kill
  an opposing creature outright (constant amount, or Kiku's
  damage-equal-to-its-power shape); `pick_removal_sacrifice` activates
  "Sacrifice this: destroy target creature" on favorable/even trades (Pus
  Kami). Remaining: X-value selection for scalable pings, and pointing a ping
  at the opponent's face for reach.
- ⏳ **THB cards still missing (need new primitives):**
  - **Aura-host-death trigger** (an Aura/enchantment-creature that triggers
    when its enchanted creature dies — there's no `EventScope::EnchantedBy`
    yet): Minion's Return (dies → return under your control), Dawn Evangel,
    Bronzehide Lion (dies → returns as an Aura), Hateful Eidolon (draw per
    Aura that was on it). LKI for the auras attached at death is the hard part.
  - **Aura-attach event** ("whenever an Aura you control becomes attached to a
    creature you control, …"): Siona's token half (Siona's ETB look-for-Aura
    *does* ship).
  - **Per-permanent ward-tax static** ("spells opponents cast targeting this
    cost {1} more" — `extra_cost_for_spell` can't see the cast's target yet):
    Callaphe's static half (its devotion power *does* ship).
  - **Pile-split decision** (Fact-or-Fiction style): Atris, Oracle of
    Half-Truths.
  - **Random choose + protection-from-mana-value**: Haktos the Unscarred.
  - **Continuous combat-damage-to-self replacement → counter**: Ironscale Hydra.
  - **Reveal-until-permanent → battlefield** end-step engine: Dreamshaper Shaman.
  - Aura-reanimation with exile-at-EOT (Storm Herald); reveal-6 opponent-exile
    (Allure of the Unknown); counter-and-Nevermore (Ashiok's Erasure);
    untap-lock tapper (Entrancing Lyre); combat-damage-prevention-except-
    enchanted fog (Inspire Awe); Medomai's Prophecy saga (chapter III delayed
    "first cast of named spell" trigger).
  - Heliod's Punishment ships without its task-counter self-removal timer (the
    lock is modeled as permanent).
- ⏳ **Tainted Pact UI**: the per-iteration "keep digging?" decision isn't
  wired for `wants_ui` players (AutoDecider takes the first card; a client
  modal + suspend/resume loop is the follow-up).

- ⏳ **Noticed this run (recent4 staples batch):** real gaps left for a
  follow-up, each needing a small new primitive:
  - **Smokestack / Tangle Wire** — "at each player's upkeep, sacrifice/tap N =
    counters on this" wants a counter-scaled per-upkeep cost (`Value::SourceCounters`
    over an active-player-only sacrifice/tap). Fading (CR 702.32) for Tangle Wire.
  - **Sanctum Prelate / Notion Thief / Hullbreacher** — chosen-number can't-cast
    gate (like Chalice) for Prelate; opponent-draw → you-draw / Treasure
    replacement (CR 121 / 614) for Thief/Hullbreacher.
  - **Outpost Siege** — Khans/Dragons mode-on-ETB + the two ongoing effects.
  - **Figure of Destiny** — activated set-base-P/T + add-creature-types gated on
    current type (leveler-adjacent, but conditional).
  - **Ancient Excavation / Insidious Dreams** — `Value::CardsInYourHand` and an
    additional-cost "discard X" with an X-bounded library search.
  - **It That Betrays** — "whenever an opponent sacrifices a nontoken permanent,
    put it onto the battlefield under your control" replacement.
- ⏳ **Noticed this run (modern_decks Kamigawa/Channel batch):**
  - **Ghost-Lit Drifter** deferred — its Channel grants flying to *X* target
    creatures, but `Effect::ApplyToTargets.max_targets` is a fixed `u8`, not a
    cast-time `Value`. A `Value`-bounded "up to N targets" would unblock it
    (and tighten Yosei's "up to five permanents that player controls").
  - **Kitsune Palliator** deferred — "{T}: prevent the next 1 damage to *each*
    creature and *each* player" needs a mass prevention-shield install
    (`PreventNextDamage` is single-target today).
  - **Ravenous (CR 702.156)** models the "draw if X≥5" clause off the resulting
    +1/+1 counter count; a counter-doubler would shift the threshold vs. printed
    X. A permanent-remembers-cast-X field would make it exact.

- ⏳ **Noticed this run (recent5 staples batch):** approximations left for a
  follow-up, each needing a small primitive:
  - **Plaguecrafter** drops the "each player who can't sacrifice, discards"
    rider (no sacrifice-or-discard fallback primitive).
  - **Misdirection** drops the printed "spell with a *single* target"
    restriction; **Venser** / **Hullbreaker Horror** model "target spell or
    permanent" as permanent-only (no bounce-a-spell-off-the-stack effect), and
    Hullbreaker drops its "up to one" mode choice.
  - **Skrelv, Defector Mite**'s grant is simplified to hexproof (no
    toxic-grant + unblockable-by-chosen-color + color choice).
  - **Flawless Maneuver** drops the free-if-you-control-a-commander alt cost
    (no `IsCommander` selector for `AlternativeCost.condition`).
  - **Neoform** counters every creature that entered this turn (no `Selector`
    for the just-searched permanent); exact only on a clean cast.
  - **Guardian Project** drops the same-name exclusion (no unique-name
    predicate).
  - **Deferred (not implemented):** Carpet of Flowers (once-per-turn main-phase
    "add X mana of one color = opp Islands"), Cultivator Colossus (etb
    put-land/draw loop), Plague Engineer (chosen-type opponents'-creatures
    -1/-1 static), Mystic Sanctuary (enters-tapped-unless-N-Islands +
    entered-untapped trigger), Wrenn and Seven, Reidane, Malevolent Hermit,
    Old-Growth Troll, Tarmogoyf Nest, Agadeem's Awakening, Joraga Treespeaker
    (LevelBand can't grant the `{T}: add {G}{G}` / Elf-lord ability — needs
    ability-granting level bands).

- ⏳ **MH3 batch shipped** (`catalog::sets::mh3`, tests `tests/mh3.rs`, 36
  cards): energy (Solstice Zealot, Tempest Harvester, Roil Cartographer,
  Solar Transformer, Phyrexian Ironworks, Hexgold Slith, Thriving Skyclaw,
  Conduit Goblin, Smelted Chargebug, Inspired Inventor), devoid/Eldrazi
  (Fanged Flames, Snapping Voidcraw, Unfathomable Truths, Titans' Vanguard,
  Skittering Precursor), plus Accursed Marauder, Faithful Watchdog, Wing It,
  Gift of the Viper, Mogg Mob, Retrofitted Transmogrant, Consuming Corruption
  (`Value::ColorCountOf` powers Breathe Your Last), Fowl Strike (Reinforce),
  Aerie Auxiliary, Scurrilous Sentry, Wither and Bloom, Fetid Gargantua,
  Dreadmobile (Vehicle/Crew), Proud Pack-Rhino, Warren Soultrader,
  Horrid Shadowspinner, Sarpadian Simulacrum, Serum Visionary, Nightshade
  Dryad, Null Elemental Blast. **Deferred — each wants one primitive:**
  Modular N / Fabricate N keywords (Arcbound Condor, Marionette Apprentice);
  exalted counter type (Emissary of Soulfire); colorless-or-abilities spend
  restriction (Sage of the Unknowable); continuous base-P/T anthem static
  (Kudo, King Among Bears); put-N-from-hand-on-top (Brainsurge); untap-count
  restriction static (Winter Moon); countered-spell-controller token mint
  (Strix Serenade); cast-or-cycle trigger (Drownyard Lurker). Also: the
  triggered-modal `AddCounter(Shield)` isn't auto-targeted (preference fn
  only auto-picks +1/+1) — a UI seat must pick the target.

- ✅ **THB batch shipped** (`catalog::sets::thb`, tests `tests/thb.rs`):
  Heliod's Intervention (`Effect::DestroyTargets` X-target destroy),
  Shark Typhoon (`TokenDefinition.dynamic_pt` mint rider + X-cycling via
  `GameAction::Cycle { x_value }`), Nyxbloom Ancient
  (`StaticEffect::ManaProductionTripled` — multiplier composes with Mana
  Reflection), Polukranos, Unchained (`Value::IfPred` escape counters +
  `StaticEffect::PreventDamageByRemovingCounters`), Elspeth Conquers Death
  (`Effect::SpellTaxUntilYourNextTurn` + reanimate chapter), plus Dream
  Trawler, Arasta, Daxos (`DynamicPt::DevotionToToughness`), Tymaret Calls
  the Dead, Thirst for Meaning, Shatter the Sky, Alseid, Mire Triton,
  Aphemia, Phoenix of Ash, Underworld Rage-Hound, Nessian Boar
  (`Predicate::TriggerBlocksSource`), Mystic Repeal, Ox of Agonas.
- ⏳ **Noticed this run (prowl / faeries / triggered-mana batch):**
  - **AutoDecider declines all `SearchLibrary` picks** (`Search(None)`) — a
    bot heuristic that takes the first eligible candidate would make
    fetch/tutor effects function under bots; many tests assume the decline,
    so flip carefully.
  - ✅ **Faerie batch shipped:** Mistbind Clique (`Effect::Champion`
    CR 702.77 with the self-sacrifice clause — Changeling Hero rides it
    too — + `Predicate::SourceChampionedSomething` + target-player tap),
    Oona, Queen of the Fae (`Effect::ExileTopMintPerChosenColor`), Faerie
    Macabre (`Effect::ExileUpToNFromGraveyards`), Rune Snag
    (`Value::SameNamedInAllGraveyards` into `CounterUnlessPaid.extra_generic`).
  - **`EventSpec::per_subject_cap` is per-turn**, so Spined Sliver won't
    re-trigger in a second combat phase the same turn.
  - **`ExtraManaOnLandTap` Mirror** mirrors the *first* produced pip; the
    printed Mana Flare lets the tapping player choose among produced types
    (matters only for multi-type productions).
  - **Notorious Throng X** uses `LifeLostThisTurn(EachOpponent)` (a max) —
    exact in 2P; multiplayer wants a damage-dealt-to-opponents sum.

- ⏳ **Noticed this run (gods / rope / split-second batch):** Rope client
  UI ✅ (`ServerMsg::Rope` + countdown banner), Nylea's may-bin reveal ✅,
  `AutoDecider` empty-`ChooseTarget` fallback ✅. Remaining:
  - ✅ **Opponent-owned `Search` decisions** are seat-routed:
    `PendingDecision::acting_player` answers `SearchLibrary` with the
    decision's named searcher (Boseiju test
    `boseiju_opponent_search_routes_to_the_searched_seat`).
  - **`Selector::LastCreatedTokens` + `GrantKeyword`** (Sokenzan) grants
    haste only to tokens minted in the same resolution — fine today; a
    "they gain haste" rider on `CreateToken` would be tidier.

- ⏳ **Noticed this run (slivers / seat-routed asks batch):**
  - **TemptingOffer ordering** — opponents now answer before the body
    runs (re-run idempotency); printed timing shows them the
    controller's result first.
  - **Statics-granted triggers from died-LKI snapshots** — the
    died-card Enrage walk only reads printed triggers; a granted
    "when this is dealt damage" wouldn't fire on lethal damage.
  - **Answer-log nesting** — `ask_seat_bool` users can't nest another
    log-using effect inside their own ask sequence (single shared log).

- ⏳ **Noticed (Modern staples batch, 2026-06-11):** 38 staples shipped
  across three waves (see git). The "deferred, each wanting one primitive"
  list is now almost fully shipped: Conspicuous Snoop ✅
  (`HasActivatedAbilitiesOfLibraryTop` + Goblin `PlayFromLibraryTop`),
  Alpine Moon ✅ (`NamedLandsNeutralized` + `NamedBySource` ability grant),
  Bring to Light ✅ (`ManaValueAtMostConverged` resolved in `Search`),
  Ad Nauseam ✅ (`RevealTopToHandLoseLifeRepeat`), Kataki ✅
  (`StaticEffect::GrantTriggeredAbility` — statics-granted triggers in both
  dispatchers), Porphyry Nodes ✅ (`Selector::LeastPowerAmongAll`),
  Shield of the Oversoul / Steel of the Godhead ✅
  (`EquipBonus.conditional`), Ravenous Trap ✅
  (`Player.cards_to_graveyard_this_turn` +
  `Predicate::CardsToGraveyardThisTurnAtLeast`), Spellskite ✅. Remaining:
  - **Ojer Taq** — DFC god (Pyxis of Pandemonium ✅ — face-down linked
    exile piles + deploy).
  - **Witchbane Orb** ships without the destroy-Curses ETB (player-attached
    Curses unmodeled). **Counterbalance**'s reveal is a MayDo (bots decline
    by default).
  - **Lightning Storm** — any-player stack activations (the "discard a land,
    choose new targets" response loop).

- ⏳ **Noticed this run (THB / splice / split-picker pass, 2026-06-12):**
  - **Splice UI/bot** — `CastSpellSpliced` is engine-only; the client has no
    splice picker and the bot never splices.
  - **Callaphe, Beloved of the Sea** — wants a "spells your opponents cast
    that target [your permanents] cost {1} more" static
    (`extra_cost_for_spell` doesn't see the cast's target today).
  - **Calix, Destiny's Hand** — -3 wants `ExileUntilSourceLeaves` anchored to
    a *chosen* permanent rather than the effect source.
  - **Hateful Eidolon / Bronzehide Lion** — die-with-attached-Aura LKI count
    and a dies→returns-as-Aura transform are both unmodeled.
  - **Tectonic Giant** mode 1 grants may-play on both impulsed cards (the
    printed "choose one of them" pick is dropped); the grant bills MV-generic
    rather than the card's real cost.
  - **Fused split casts with targets** — the client's half-picker greys the
    Fused button when either half targets (the targeting cursor collects one
    target; fused needs left + right slots).

- ⏳ **Noticed this run (ZNR MDFC + hexproof-from-color batch):**
  - **Dropped riders on shipped ZNR cards:** Hagra Mauling's "{1} less if an
    opponent controls no basic lands" cost reduction; Turntimber Symbiosis's
    "+3 counters if the deployed creature's MV ≤ 3" (the `LookPickToHand
    { to_battlefield }` primitive can't condition counters on the pick).
  - **ZNR cards still unimplemented** (each wants a new primitive):
    Valakut Awakening (put any number from hand on bottom, then draw that
    many +1 — no bottom-then-draw effect); Agadeem's Awakening (mass-reanimate
    any number of *distinct-MV* creatures ≤ X — no different-MV multi-target
    reanimation); Sea Gate Stormcaller (copy-your-next-cheap-I/S delayed
    trigger). Sporeweb Weaver / Garruk's Harbinger want a general
    "when this is dealt damage" trigger (non-combat enrage) + a combat-damage
    library-look.
- ⏳ **Noticed this run (claude/modern_decks, 2026-06-11 second pass):**
  `UnlessPlayerPays` per-seat routing ✅ (rhystic/Kataki taxes now prompt
  the taxed `wants_ui` seat via `ask_seat_bool`). Remaining:
  - **`RevealTopToHandLoseLifeRepeat` + Seek library pick** still answer
    through the single global decider (non-Bool decisions; the
    `ask_seat_bool` replay-log only covers yes/no questions).
    Kataki under AutoDecider still declines → bots sacrifice their
    artifacts even with open mana (needs a bot heuristic, not routing).
  - **`SacrificeOrPay` chooser** — the auto rule (sacrifice when a match
    exists, else fold the pay into the cost) is deterministic; a wants_ui
    "which half?" picker would make Bayou Groff interactive.

- ⏳ **Noticed this run (follow-ups sweep):** `Effect::MayPay` wants_ui
  suspend ✅ (seat-routed). Remaining:
  - **Nadu's granted ability** is modeled as a trigger on Nadu itself with
    a per-subject cap (behaviorally equivalent); a true "creatures you
    control have [triggered ability]" static grant framework is still open
    (matters for ability-reading effects).
  - **Karplusan Minotaur's lose-a-flip ping** lets the controller aim the
    damage; printed text has an opponent choose the target.
  - **`EventSpec::per_subject_cap`** only counts permanent subjects; a
    player-subject cap would need an EntityRef-keyed map.
- ⏳ **Noticed (modern_decks batches 4-6):** all the listed cards shipped
  (Nadu / Six / Ajani MDFC / Kozilek / Ulamog the Defiler / Springheart /
  Not Dead After All / Indomitable Creativity — each with its primitive).
  Remaining: none — Clash (CR 701.30) now prompts each `wants_ui` seat
  to bottom or keep via the seat-routed answer log.

- ⏳ **Noticed (staples expansion / audit):** The Ozolith, Soulless Jailer,
  Underworld Breach, Karn the Great Creator, and Sunken Citadel all shipped
  with their primitives. Remaining:
  - **Ulamog, the Ceaseless Hunger** cast trigger is modeled as two
    single-target exile triggers (multi-target triggers still unsupported —
    see the existing multi-target ETB note).
  - **Madcap Experiment** bills its reveal count as life loss rather than
    damage (`RevealUntilFind.life_per_revealed`); a damage rider would be
    more faithful vs prevention effects.
  - ✅ **`resolve_damage_assignment`** rejects non-trample under-assignment
    even with every blocker at lethal (CR 510.1d; test
    `cr_510_1d_non_trample_under_assignment_falls_back_to_default`).

- ⏳ **Noticed this run (multikicker / mill batch):**
  - **MayDo wants_ui suspend** ✅ — `Effect::MayDo` now suspends for a
    `wants_ui` controller via the stash-and-rerun path
    (`PendingEffectState::MayDoAnswerPending`); the client's existing
    OptionalTrigger yes/no modal answers it. Bots/tests still use the
    synchronous decider.
  - **Squad/Replicate/Multikicker stepper cap** — the bot probes kick counts
    1–4; an exact max-affordable computation would kick higher with big pools.

- ✅ **Staple/mill/landfall follow-up batch — all eight shipped:**
  - **Everflowing Chalice** ✅ — `Keyword::Multikicker` (CR 702.33c) +
    `GameAction::CastSpellMultikicked { times }` + `CardInstance.kick_count`
    read by `Value::TimesKicked`; client pay-times stepper generalized to
    Squad/Replicate/Multikicker (`PayTimesMechanic`). Hangarback's cast-X →
    ETB counters already worked (x_value threads into the ETB ctx).
  - **Archive Trap** ✅ — `Player.searched_library_this_turn` (stamped at the
    Search funnels, reset each turn) + `Predicate::SearchedLibraryThisTurn`
    gating the `AlternativeCost.condition` free cast.
  - **Dauthi Voidwalker** ✅ — `ExileCardsBoundForGraveyard.void_counter`
    stamps `CounterType::Void`; the sac ability rides `GrantMayPlay` over
    `InExile + WithCounter(Void)`.
  - **Chandra, Torch of Defiance** ✅ — `ExileTopAndGrantMayPlay.uncast_penalty`
    registers a next-end-step still-`InExile` check that runs the fallback.
  - **Scrap Trawler** ✅ — `SelectionRequirement::ManaValueLessThanEventAmount`;
    died events now carry the dying card's MV (`event_amount_for`) into
    `trigger_event_amount_scratch`.
  - **Torbran, Thane of Red Fell** ✅ — `StaticEffect::AddDamageToOpponents`;
    `scale_damage_to` is source-aware (`resolving_source` carries in-flight
    spell color/controller).
  - **Conflagrate** ✅ — `AdditionalCastCost::DiscardXFromCost` takes the
    cast's X (Flashback—discard X cards).
  - **Urza's Saga** ✅ — `Effect::GainActivatedAbility` →
    `CardInstance.granted_activated_abilities` (cleared on leave, CR 400.7);
    saga lands advance on the land drop (`place_land_card`).

- ✅ **Noticed-items sweep (meld batch) — all shipped:**
  - **Prized Amalgam** ✅ — `GameState.entered_from_graveyard_this_turn`
    (stamped at the gy→bf move funnel and every cast-from-graveyard site) +
    `SelectionRequirement::EnteredFromGraveyardThisTurn`; the return rides
    `DelayUntil(NextEndStep)` with an in-graveyard re-check.
  - ✅ **One-spell-per-turn lock** (`StaticEffect::OneSpellPerTurn` — Rule
    of Law, Eidolon of Rhetoric, Archon of Emeria). ⚠️ Audit 2026-06-11:
    reads the owner-untap-scoped `spells_cast_this_turn`, wrongly locking
    non-active players — see audit P1.
  - **Chord of Calling** ✅ — `SelectionRequirement::ManaValueAtMostXFromCost`
    concretized via `resolve_x(ctx.x_value)` in `Effect::Search`.
  - **Shadowspear** ✅ — `Effect::LoseKeywordThisTurn` +
    `CardInstance.removed_keywords_eot` (strips printed/granted/counter
    keywords for the turn; cleared at cleanup).
  - **All Is Dust** ✅ (`Effect::SacrificeAllMatching`) / **Oblivion
    Stone** ✅ (`CounterType::Fate`).
  - **Emrakul, the Aeons Torn** ✅ — `Keyword::ProtectionFromColoredSpells`
    cast-time targeting gate + `EventKind::PutIntoGraveyard` self-source
    graveyard trigger (shuffle gy into library) + cast-trigger extra turn.

- ⏳ **Noticed this run (claude/modern_decks):**
  - ✅ **Per-blocker combat-damage assignment modal** ships
    (`spawn_damage_assign_modal` — +/- steppers per blocker, total capped at
    attacker power; the old auto-answer fallback is gone).
  - ✅ **Controller-scoped damage doubling/halving** ships —
    `DoubleDamageToOpponents` / `HalveDamageToYou` + the target-aware
    `scale_damage_to` at both funnels (Gisela, Blade of Goldnight).
  - **Room rules corners** — lock-a-door effects (709.5g), "fully unlock"
    triggers (709.5i), and combined MV in non-stack zones (709.4b) are not
    modeled; door casts also skip the convoke/delve/alt-cost riders.
  - ✅ **Old-style factories converted** — `scripts/convert_to_default_style.py`
    rewrote all ~2.6k fully-specified `CardDefinition` literals to
    `..Default::default()` style (-74k lines); new `CardDefinition` fields no
    longer require catalog-wide patch scripts.

- ✅ **Meld** ships (CR 701.37 — see the rules-audit row); The Mightstone
  and Weakstone is now fully faithful (Legendary, artifact-only {C}{C}).
  (Rooms ✅ — CR 709.5, Unholy Annex // Ritual Chamber.)
- ✅ **This batch shipped** (was the "deferred, each wants one primitive"
  list): DFC sagas (`Effect::ExileSelfReturnTransformed` — Fable of the
  Mirror-Breaker), search statics (`OpponentsSearchTopN` / `SearchTax` —
  Aven Mindcensor, Leonin Arbiter), end the turn (CR 728 —
  `Effect::EndTheTurn`; Sundial, Day's Undoing), color-filtered gy-hate
  (`ExileCardsBoundForGraveyard.colors` — Sanctifier en-Vec), activation
  tax (`StaticEffect::ActivationTax` — Suppression Field), Reckoner
  Bankbuster (charge-empty payout via `remove_counter_cost` + If).
- ⏳ **Still deferred:**
  - ✅ **Hofri's token-leaves rider** ships
    (`DelayedKind::WhenCardLeavesBattlefield` +
    `Effect::WhenLastCreatedTokenLeaves`; the shared `on_left_battlefield`
    funnel fires it).
  - **Exalted Angel's printed trigger** is modeled as Lifelink (gains on
    any damage it deals — equivalent in practice).
  - **Eon Hub vs. suspend/pacts**: skipped upkeeps also skip suspend ticks
    and pact payments — correct per CR 614.10b, but worth a regression test
    when pact decks meet Eon Hub.
  - ✅ **Shipped this run:** `Effect::SwitchPT` (CR 613.7d) + Wandering
    Fumarole's `{0}` switch; Lavaclaw Reaches' firebreathing
    (`Predicate::SourceIsCreature` gates animated-state abilities);
    Street Wraith's life-payment Cycling (`Keyword::CyclingLife`); The One
    Ring's protection from everything
    (`Effect::PlayerProtectionUntilNextTurn`).
- ⏳ **Tempting offer / opponent-may wants_ui suspend** —
  `Effect::TemptingOffer` and the new `Effect::PlayersMayAccept` (Vexing
  Devil, Browbeat, Risk Factor) ask via the synchronous decider; a
  networked human seat gets the AutoDecider default (decline). Same family
  as the existing inline-picker gaps.
- ✅ **Any-color spend for exile-casts** — `GrantMayPlay.any_color` /
  `ExileTopAndGrantMayPlay.pay_any_color` stamp the cost as MV-generic
  (CR 609.4b). Gonti, Hostage Taker, Nassari.
- ✅ **Gather Specimens vs token mints** — every token-creation site now
  funnels through `mint_token_onto_battlefield`, which applies the ETB
  control replacement (and CR 111.2 ownership).
- ✅ **Grafdigger's Cage vs search-to-battlefield** — `SearchPending` /
  `PutFromZonesPending` consult the lockdown before placing creatures.
- ✅ **NameCard bot heuristic + wire** — `Decision::NameCard.suggestions`
  (most-common name in the relevant zone, `rank_names_by_frequency`);
  AutoDecider takes the top pick; the wire + a client picker modal ship.
- ✅ **Saheeli Rai -7 distinct names** — `SelectionRequirement::
  NameDiffersFromLastMoved` + search picks validated against the
  candidate set (`SearchPending.eligible`).
- ✅ **Search-pick UI eligibility** — `Decision/DecisionWire::SearchLibrary`
  carry an `eligible` set; Impulse reveals show every revealed card with
  non-pickable ones greyed in the client modal; the bot restricts picks.

- ✅ **Combat-damage-to-a-creature trigger dispatch (CR 510.2).**
  `resolve_combat_damage_with_filter` records every creature-vs-creature damage
  pair and, after all damage in the step is dealt, fires
  `DealsCombatDamageToCreature` triggers via the shared
  `fire_combat_damage_triggers` (printed SelfSource/AnyPlayer, equipment
  CR 702.6e, soulbond, YourControl, gy FromYourGraveyard), binding the damaged
  creature to slot 0. Umezawa's Jitte now charges when its equipped creature is
  blocked. (Fires once per damaged creature — a minor over-count for Jitte under
  multi-block.)
- ✅ **Cipher follow-ups.** Hidden Strings, Rubblehulk, and Trait Doctoring
  (CR 612 layer-3 text change) all ship.
- ✅ **UI render edits unblocked** — `apt-get install -y libwayland-dev
  libasound2-dev libudev-dev` makes the client build in the web sandbox;
  client edits now ship normally.
- ✅ **Aether Gust** — spell half rides `CounteredSpellZone::
  OwnerLibraryTopOrBottom`; permanent half rides the existing
  `LibraryPosition::OwnerChoice` Move dest.
- ✅ **Continuous "becomes a copy" (CR 707.2)** — `Effect::BecomeCopyOfFor`
  swaps the definition with a scheduled revert (`GameState.temporary_copies`,
  the Act-of-Treason plumbing pattern): reverts at duration end and on
  battlefield-leave; `non_legendary` strips Legendary (707.2e). Ships Echoing
  Equation, Vesuva, Thespian's Stage. Remaining ⏳: "while attached" aura
  copies (Mirrorform) want a WhileSourceOnBattlefield-style duration tied to
  the aura.
- ✅ **Reinforce/face-down client affordances.** `GameAction::Reinforce` (CR
  702.77) and `CastFaceDown`/`TurnFaceUp` are engine-complete. `reinforceable_hand`
  now ships (`PlayerView.reinforceable_hand` + `compute_hand_affordances`,
  dry-run-gated on a payable cost + creature target). `turn_up_able` ships.
- ⏳ **MKM Disguise riders dropped this run (each wants one small primitive).**
  - ✅ Granite Witness — "tap **or untap**" now ships via `ChooseMode([Tap, Untap])`.
  - ✅ Offender at Large — "**up to one** target" now rides `Effect::MayDo` (the
    controller may decline the pump). CR 115.1b.
  - Experiment Twelve / Pyrotechnic Performer — "or another creature you control
    is turned face up" collapses to a SelfSource-only trigger (no per-creature
    turned-up binding for other permanents).
  - Deferred (need new primitives): Coveted Falcon (control-swap + draw-per),
    Aurelia's Vindicator (X-cost Disguise + exile-up-to-X + return-on-leave),
    Concert Kaboomist (noncreature-spells-since-last-turn count), Boltbender
    (choose new targets), Polygraph Orb (collect evidence).
- ⏳ **Face-down follow-ups (this run shipped manifest + the 2/2 object).**
  - **Morph cast-face-down spell path** (CR 702.36): a `GameAction::CastFaceDown`
    that pays {3} and casts the card as a face-down 2/2 creature spell, reusing
    the new `CardInstance.face_up_def` swap + `turn_face_up_action`. No catalog
    Morph cards yet, so deferred.
  - Disguise (CR 702.166) ✅ (`Keyword::Disguise` + `facedown_disguise_definition`)
    and Cloak (CR 702.182) ✅ (`Effect::Cloak` + serialized `CardInstance.cloaked`).
    Follow-up ⏳: Hide in Plain Sight's full "look at top five, cloak two, rest to
    bottom random" selection is simplified to cloaking the top two.
  - **Manifest-dread "turn up if a creature card"** already works via
    `TurnFaceUp`; a face-down noncreature can't be turned up (correct).
- ⏳ **Cards deferred this run (each wants one small primitive):**
  - ✅ **Umezawa's Jitte** — ships via `EquipBonus.triggers_on_equipment` (the
    granted combat-damage trigger resolves with the Equipment as source, so the
    charge counters land on Jitte) + three `remove_counter_cost` activated
    abilities (+2/+2 / -1/-1 / gain 2). Charges on combat damage to a player
    **and to a creature** (CR 510.2 dispatch now ships) — fires when blocked.
  - ✅ **Leyline Binding** — Domain cost reduction ({1} less per basic land type)
    ships via `StaticEffect::SelfCostReducedByDomain` + `Value::DomainCount`;
    Tribal Flames reuses the Value for its X-damage. (Leyline Binding, Tribal Flames.)
  - ✅ **Orcish Bowmasters** — `Player.cards_drawn_this_step` +
    `Value::CardsDrawnThisStep` power the draw-step first-draw exemption.
  - ✅ **Restless lands cycle** — all ten ship (`restless_land` helper;
    Anchorage / Prairie / Vents landed last).
  - ✅ **Witch's Oven** — `Effect::WithSacrificedPt` re-stamps the
    cost-sacrificed creature's P/T at the ability's resolution.
- ✅ **Client Squad/Replicate stepper** — right-click a squadable/
  replicatable hand card → "pay N times" modal (`PayTimesState` +
  `spawn_pay_times_modal`); targeted spells arm the targeting cursor with
  `TargetingState.pending_pay_times` so the submit routes through
  `CastSpellSquad`/`CastSpellReplicate`. Hand highlights include both sets.
- 🟡 **Resolution-time target legality (CR 608.2b).** General now: every
  single-target spell whose primary target was a *battlefield permanent at
  cast time* (`CardInstance.cast_target_was_battlefield`, stamped in
  `finalize_cast`) fizzles on resolution if the target left the battlefield,
  stopped matching the (mode/kicker-aware) filter, or gained Hexproof/Shroud;
  a fizzled real card is countered into its owner's graveyard. Token copies
  keep the bare filter re-check. **Multi-target all-illegal fizzle ✅** —
  battlefield-aimed multi-target spells fizzle only when every slot is
  illegal (Arc Trail tests). Remaining ⏳: Aura spells (permanent path) and
  protection-from-color on resolution. (Audit follow-up closed — triggered
  abilities fizzle per CR 608.2b and flashbacked fizzles route to exile.)
- ⏳ **Demonstrate "you may" + opponent choice (CR 702.150).** `Effect::
  Demonstrate` always copies (the optional "you may" collapses) and auto-picks
  the lowest-seat opponent rather than prompting the caster. Fine for bots;
  a `wants_ui` caster should get a yes/no + opponent picker.
- ⏳ **Impending / Hideaway follow-ups (this run shipped the keywords).**
  - Impending (CR 702.183) ✅ — the client's Time-counter label reads
    `PermanentView.impending_counters` and badges "Impending N".
  - Hideaway (CR 702.76, `Effect::Hideaway`): the hidden-card pick auto-resolves
    to the highest-MV card rather than prompting. The Lorwyn land cycle ✅ —
    Mosswort Bridge / Spinerock Knoll / Windbrisk Heights ship with their
    printed gates (`Value::PowerOf` fan-out, `Value::LifeLostThisTurn`,
    `Value::CreaturesAttackedWithThisTurn`).
- ⏳ **Card riders dropped (each wants one small primitive):**
  Glissa Sunslayer ✅ (full combat-damage `ChooseMode` — draw/lose, destroy
  enchantment, remove-all-counters); Bristly Bill ✅; Nowhere to Run ✅;
  Get Lost / Sip of Hemlock use the destroyed permanent's *owner* for the
  follow-up (differs from "controller" only under control-stealing).

- ⏳ **Cube bombs still needing primitives.** Skyclave Apparition ✅,
  Grafdigger's Cage ✅ (`StaticEffect::GraveyardLibraryLockdown` — gates
  flashback/escape/Muldrotha/library-top/free-casts and gy/library →
  battlefield creature entries; search-to-battlefield pending states don't
  consult it yet), Hostage Taker ✅ + Gonti ✅ (paid casts from exile via
  `GrantMayPlay { pay_own_cost }` / `LookTopExileOneMayPlay` + the
  `WhileExiled` may-play duration — the any-color spend clause is still
  dropped). Remaining: Duplicant (imprint + P/T-from-exiled CDA).
- ⏳ **`EachOpponentPlaneswalker` was unneeded** — Saheeli's "each planeswalker
  they control" rides `EachPermanent(Planeswalker & ControlledByOpponent)` with
  damage-to-PW (CR 120.3c). Karn Liberated's -14 and Ugin's -X exile-by-MV
  still approximate (no X-aware `ManaValueAtMostX` requirement yet).
- ✅ **Client crate builds + clippy + tests in the web sandbox** once
  `apt-get install -y libwayland-dev libasound2-dev libudev-dev` is run (the
  wayland-sys / alsa-sys / libudev build scripts need those system libs).
  `cargo clippy --workspace --all-targets` and `cargo test --workspace` are
  both green this run.
- ⏳ **Dedicated immediate-blink primitive.** Restoration-style instant flicker
  is carded via `Exile { target } + Move { Target → Battlefield }` (Restoration
  Angel, Felidar Guardian). A single `Effect::FlickerImmediate { what }` would be
  cleaner (one trigger, no two-step target capture) but isn't required.
- ⏳ **Cast-from-exile (any color) rider on linked exile.** `ExileUntilSourceLeaves`
  has no may-play grant, so Hostage Taker ("exile … you may cast it, any mana
  type") and similar can only ship the exile half. Pair the linked-exile with a
  grant-may-play-from-exile + any-color spend permission.
- ✅ **Snow permanent count** `Value::SnowPermanentCountControlledBy` (CR
  205.4g) — Skred ("damage = snow permanents you control"). Marit Lage / other
  snow payoffs can reuse it.
- ✅ **Tap-N activation cost.** `ActivatedAbility.tap_n_filter` taps N matching
  untapped permanents (source eligible) as a cost — Heritage Druid. (An "X can't
  be blocked this turn" grant for Whirler Rogue-style payoffs is still ⏳.)
- ✅ **Cost-sacrifice P/T visible to the ability's resolution** —
  `activate_ability` wraps the queued effect in `Effect::WithSacrificedPt`,
  restoring the scratch at resolution (Witch's Oven's two-Food branch).
- ✅ **Put-permanent-from-hand-onto-battlefield effect** —
  `Effect::PutFromHandOntoBattlefield { who, filter, count, tapped, haste,
  sacrifice_eot }`: the controller picks up to `count` matching hand cards via
  `ChooseCards` (always optional) and they enter under their control, with
  optional haste + next-end-step sacrifice riders. Ships Sneak Attack, Through
  the Breach, Elvish Piper, Quicksilver Amulet, and the combat-damage
  drop-a-Goblin trigger (Goblin Lackey / Warren Instigator) off a
  `DealsCombatDamageToPlayer` trigger with a creature-type filter. ✅
- ✅ **`Value` arithmetic (count × k).** `Value::Times(a, b)` ships; Goblin
  Piledriver's "+2/+0 for each other attacking Goblin" rides it.
- ⏳ **Multi-target ETB / triggered abilities.** `StackItem::Trigger` carries a
  single `target`, so a triggered ability needing *two* targets (Vedalken
  Plotter's "exchange control of target land you control and target land an
  opponent controls") can't be auto-targeted for both slots. Spells already
  thread `additional_targets`; triggers need the same. (Switcheroo, a sorcery,
  exercises `Effect::ExchangeControl` cleanly meanwhile.)

- ✅ **Chosen-creature-type anthem static.** `StaticEffect::AnthemForChosenType
  { power, toughness, exclude_source }` reads the source's live
  `chosen_creature_type` (set at ETB via `Effect::NameCreatureType`) and emits a
  layer-7 pump over the controller's matching creatures in
  `gather_continuous_effects`. Ships Adaptive Automaton (`exclude_source`) and
  Patchwork Banner. Remaining: Metallic Mimic's enters-with-a-counter rider (a
  chosen-type ETB-counter replacement, not an anthem) and the "this is the
  chosen type in addition to its other types" self-type-add layer-4 effect.
- ✅ **Delirium / Threshold conditional static** — handled by the existing
  `StaticEffect::PumpSelfIf { condition, power, toughness, keywords }`:
  `Predicate::DeliriumActive` (Spineseeker Centipede +1/+2 + vigilance) and
  `Predicate::ValueAtLeast(GraveyardSizeOf(You), 7)` (Mind Drill Assailant +3/+0)
  both ride it — no new primitive needed.
- ✅ **Exile-self activation cost (graveyard + battlefield).** The gy/hand path
  (Stone Docent / Eternal Student) powers Daring Fiendbonder; `exile_self_cost`
  now also fires for a *battlefield* source via `move_card_to(.., Exile)` in
  `activate_ability` (Hanged Executioner's "{3}{W}, Exile this: exile target
  creature"). Daring Waverider's ETB cast-from-graveyard is a separate
  primitive (cast-IS-from-gy-for-free) still ⏳.
- ⏳ **Bloomburrow follow-ups (noticed this run):**
  - ✅ **Gift** (CR 702.165) ships (`CardDefinition.gift` + `GameAction::CastGift`
    + `CardInstance.gift_promised`; `TokenDefinition.tapped`; client right-click
    promise + `KnownCard.{has_gift,gift_label,gift_needs_target}`). Batch in
    `decks::gift` + Nocturnal Hunger upgraded. Remaining gift cards need new
    primitives: Coiling Rebirth (reanimate + 1/1 token-copy), Mind Spiral
    (draw-N + tap/stun), Pool Resources / Sazacap's Brew (Seek), Cruelclaw's Heist
    (exile-and-may-cast), Perch Protection (gift an extra turn). Also: the
    client's legal-target highlight for a promised gift still derives from the
    *base* effect, so a broadened gift target (Flood Maw's noncreature) isn't
    highlighted though the server accepts it.
  - ✅ **Survival** (CR 702.180) ships ("at your second main, if tapped …" —
    `StepBegins(PostCombatMain)`/`ActivePlayer` + tapped intervening-`if`;
    `decks::survival`). Remaining Survivors need primitives: Kona (put a
    permanent from hand onto the battlefield), Wary Zone Guard (enters tapped +
    perpetual +1/+1), Improvising Aerialist (perpetual flying), Veteran Survivor
    (exile-with-source count static), Rip / Effie (reveal-N-distinct-powers, seek).
  - **Expend** (CR 700.14) ships (`mana_spent_on_spells_this_turn` +
    `EventKind::Expend` + `Predicate::ExpendReached`; Roughshod Duo). Remaining:
    a `Value::ManaSpentOnSpellsThisTurn` reader for "expend 8" payoffs that
    scale, and bot awareness of expend thresholds when sequencing spells.
  - ✅ **Per-target scaled damage** — Sunspine Lynx ships via a `ForEach` over
    each player + `Value::NonbasicLandCountControlledBy(Triggerer)` (re-read per
    recipient). Also added `StaticEffect::DamageCantBePrevented` (CR 615.12,
    permanent-static prevention bypass).
  - **Equipment tokens** ship via `TokenDefinition.equipped_bonus` (Mabel's
    Cragflame). Remaining: token Equipment whose equip cost or granted abilities
    aren't expressible as a flat `EquipBonus` (e.g. activated-ability grants).
  - **Pawpatch Recruit** "whenever another creature you control becomes the
    target of an opponent's spell/ability, +1/+1 on a different creature" —
    needs the `YourPermanentTargetedByOpponent` scope wired to a +1/+1-on-another
    body (the engine has the scope; the "other than that creature" target
    constraint is the gap).
- ⏳ **Bargain / Eldraine follow-ups (this run):**
  - ✅ "This spell costs {N} less if it's bargained" — `StaticEffect::
    BargainCostReduction { amount }` folded into `cast_spell_bargain` via the
    transient `extra_cast_reduction` (Ice Out, Johann's Stopgap, Hamlet
    Glutton).
  - ✅ Cacophony Scamp / Heartfire Hero "when this dies, deals damage equal to
    its power" — CR 603.10 leaves-battlefield LKI now ships (`leaves_bf_lki` +
    `resolving_lki_source`; `Value::PowerOf`/`ToughnessOf` read the dying
    object's last-known counter-boosted P/T). Promotes Goldvein Hydra's
    death-treasure rider too.
  - ✅ Heartfire Hero **Valiant** — rides `BecameTarget + YourControl` +
    `once_per_turn` (CR 603.3d). Pawpatch Recruit's "another creature you
    control becomes targeted by an opponent" variant still ⏳.
  - **Gift** (Wilds of Eldraine; Sazacap's Brew, Coiling Rebirth) — promise an
    opponent a gift as an optional rider.
  - The bot never pays Bargain (always casts the base spell); a client
    "sacrifice for Bargain?" picker + bot fodder-choice are both unwired —
    `PlayerView.bargainable_hand` is surfaced but unused by the UI.
- ⏳ **Transform-DFC batch — dropped riders to revisit:**
  - ✅ Vildin-Pack Alpha's "when a Werewolf you control enters, you may
    transform it" (MayDo + `Transform { TriggerSource }`); ✅ Frenzied
    Trapbreaker's on-attack "destroy target artifact/enchantment defending
    player controls". Remaining: The Myriad Pools' "copy a permanent spell"
    cast trigger; Azcanta's "you *may* transform" (auto-transforms now);
    Search for Azcanta back-face dig ships but the "may reveal" is auto.
  - Daybound (CR 702.146): ETB "becomes day" ✅ and the cast-time "casting a
    daybound spell while neither day nor night makes it day" half ✅ (702.146e,
    in `finalize_cast`). The per-player night-entry rule beyond CR 502.2 is
    still ⏳.
  - Werewolf night→day check approximates "a player cast two or more spells
    last turn" as the global `spells_cast_last_turn >= 2`; a true per-player
    last-turn tally would be more faithful.
  - Manifest dread ✅ (Hauntwoods Shrieker; `Effect::Manifest`/`ManifestDread`
    + face-down 2/2 object + `GameAction::TurnFaceUp`). DFC sagas + Rooms
    (Unholy Annex) + meld (Westvale/Hanweir, Mightstone/Weakstone) + the Morph
    cast-face-down spell path still need their own subsystems on top.

- ✅ **Remaining STX printed cards** — all shipped (this run): layer-1 copy
  (Echoing Equation), Jadzi // Journey, Codie, Ecological Appreciation,
  Flamescroll // Revel. Historical blocker list below; only Kasmina's
  ability-sharing static + the inline `wants_ui` picker gaps remain.
- (historical) **Remaining STX printed cards (each needed a new primitive):**
  - ✅ **Hone counters + cast-from-exile** — `CounterType::Hone` +
    `Effect::HoneFromHand` + `GameState::process_hone` (upkeep tick → {4}-less
    cast-from-exile via a may-play grant). Nassari rides
    `ExileTopAndGrantMayPlay { EachOpponent }` + `CardInstance.cast_from_exile`
    + `Predicate::CastSpellFromExile`. Uvilda//Nassari shipped (Nassari's "any
    color" mana clause dropped).
  - **Continuous "becomes a copy of" (layer 1)** — until-EOT/permanent copy of
    a chosen permanent (Echoing Equation, Helm of the Host loop, Mirrorform).
  - **Fixed alternative cost "cast for {N} instead"** + **put-lands-from-hand-
    onto-battlefield** — Jadzi // Journey to the Oracle.
  - **`StaticEffect::CantCastPermanentSpells`** + a next-spell-cast reflexive
    impulse keyed to the cast spell's MV — Codie, Vociferous Codex.
  - **Up-to-N variable targets + opponent-split** — Ecological Appreciation.
  - **Variable-sacrifice cost reduction** ("sacrifice any number, {N} less
    each") — Awaken the Blood Avatar (currently 🟡: flat cost, sac dropped).
  - **Opponent-ability-activation trigger + spell-lock** — Flamescroll // Revel.
  - ✅ done this run: Plargg//Augusta, Extus//Awaken (🟡), Rowan//Will,
    Mila//Lukka, Valentin//Lisette (exile-instead + reflexive),
    Radiant Scrollwielder (non-combat lifelink, CR 702.15), Mascot Exhibition
    (corrected), tapped/untapped anthem filters, cross-type legend-rule fix.
  - **`Effect::Fateseal` / `Effect::DigToHandLoseLife` `wants_ui` suspend path**
    — both currently decide inline (the bot/scripted path); a networked human
    isn't prompted. Same gap as the existing inline pickers.
  - **Detain interactions** — `detained_by` blocks attack/block/activate and
    lifts at the detainer's next turn; a granted-static "permanents your
    opponents control enter detained" variant (Lavinia of the Tenth) is ⏳.

- ⏳ **Discovered this run (coin-flip / artifact batch — deferred cards):**
  - ✅ **Goblin Welder** ships (`Effect::WeldArtifacts` — the gy half is
    auto-picked, highest MV, rather than a second target).
  - **Squee, the Immortal** — needs a static "you may cast this from your
    graveyard or from exile" permission (a real cast onto the stack, unlike
    Gravecrawler's `from_graveyard` Move approximation).
  - **Karplusan Minotaur** — cumulative upkeep whose cost is a coin flip
    (CR 702.24 + 705) + the win/lose-flip "deal 1 to any target" pair.
  - (✅ Goblin Recruiter ships via `SearchUpToN` (Goblin filter) to library top;
    "any number" modeled as up to 10.)
  - **Cursed Scroll** — name-a-card + reveal-at-random-from-hand + conditional
    damage if the random card matches.
  - **Price of Progress / Pyromancer Ascension / Tibalt's Trickery /
    Daretti, Scrap Savant** — per-player-scaled damage, quest-counter spell
    copying, counter-and-cascade-from-exile, and a planeswalker, respectively.
  - **Grafted Wargear** — equip {0} with "when unattached, sacrifice the
    creature" (no on-unequip sacrifice hook yet).
- ⏳ **Discovered this run (modern_decks staples/cleave/multi-pick run):**
  - **Engineered Explosives / Zabaz** — both need a counter snapshot that
    survives the source's sacrifice-as-cost: EE's "destroy each nonland
    with MV equal to its charge counters" reads the sacrificed source's
    counters at resolution (extend the `sacrificed_power` scratch family
    with a counter map, or concretize `ManaValueEqualsSourceCounters` at
    activation); Zabaz additionally wants a modular-trigger counter-bonus
    replacement.
  - **Hogaak, Arisen Necropolis** — needs "you may cast from your
    graveyard" on the *main* cast path (today only `from_graveyard`
    activations and flashback leave the graveyard), plus a "can't spend
    mana on this" gate forcing full Convoke+Delve payment.
  - **Runed Halo / protection from a card name** — `named_card` exists for
    ability suppression but not as a protection quality.
  - **Tidebinder Mage** — "doesn't untap while you control this" wants a
    linked `PreventUntap` (stamped like `exiled_by`), not a stun counter.
  - **Hallowed Moonlight / Containment Priest as EOT grant** — needs a
    turn-scoped `ExileNontokenCreaturesNotCast` (flag on GameState, not a
    battlefield static).
  - **Cultivator Colossus** — repeat-until-decline ETB loop primitive.
  - **Fell Stinger** — exploit payoff is bound to the controller; a real
    "target player" inside an exploit `MayDo` needs trigger-target plumbing
    through the reflexive body.
  - **Shacklegeist** — "can block only creatures with flying" restriction
    (inverse of CantBlockFlying) not modeled; rider dropped.
- ⏳ **Discovered (modern_decks landfall/exile batch):**
  - ✅ **`Effect::NthResolutionThisTurn { branches }`** — runs `branches[n]`
    where `n` = times an escalating ability the controller owns has resolved
    this turn (`Player.escalating_resolutions_this_turn`, reset at untap).
    Ships Omnath, Locus of Creation's 1st/2nd/3rd-landfall escalation.
  - ✅ **`Effect::CatchUpBasicLands`** (Scholarship Sponsor), **`Effect::
    ExileFromHandTaxed`** (Elite Spellbinder, owner-may-play + tax), **hone
    counters** (`process_hone`, Uvilda // Nassari).
  - ✅ **Codie, Vociferous Codex** — `ControllerCantCastPermanentSpells` +
    `OnYourNextSpellCastThisTurn` + filtered `Discover` impulse.
  - **Awaken the Blood Avatar** variable-sacrifice cost reduction still ⏳
    (auto-path sacrifices 0; needs a cast-time "sacrifice N, {2} less each"
    decision threaded into the cost computation).
  - **Before adding a "new" card, grep the catalog for its name** — Omnath
    already existed in `decks/modern.rs`; nearly duplicated it.
- ⏳ **Discovered this run (STX sweep / extras_17):**
  - ✅ **"Sacrifice X or pay {N}" OR additional cost** —
    `AdditionalCastCost::SacrificeOrPay` (Bayou Groff faithful; a wants_ui
    "which half?" chooser is a follow-up).
  - ✅ **Generic `CardExiled` event** — `EventKind::CardExiled` maps to the
    `GameEvent::PermanentExiled` emitted by the central exile-placement funnel.
    Pair with `once_per_turn` + `IsTurnOf(You)` for "whenever one or more cards
    are put into exile during your turn" (Stonebinder's Familiar shipped).
  - ✅ **Turn-scoped ETB delayed trigger** — `Effect::CreaturesYouControl
    EnteringThisTurn` + `DelayedKind::CreatureYouControlEntersThisTurn`, fired
    from the dispatcher and expiring at cleanup; First Day of Class.
  - ✅ **`SelectionRequirement::EnteredThisTurn`** — `CardInstance.entered_turn`
    stamped centrally at every ETB (also at the movement ETB site so
    Emergent Sequence counts the land it just searched mid-resolution);
    Shaile // Embrose.
  - ✅ **X-scaled MV target filter** — `ManaValueAtMostXFromCost` is now
    resolved with the cast's X at cast-time validation and the CR 608.2b
    re-check; Confront the Past's MV≤X reanimate gate is faithful.
  - ✅ **Mastery alt-cost rider** — handled by the existing
    `AlternativeCost.effect_override` (the alt cast runs a different effect).
    Ships **Fervent Mastery** and **Verdant Mastery** (✅ this run — its
    `effect_override` now distributes basics opp-bf / your-bf×2 / hand on the
    {3}{G} alt-cast, vs your-bf×2 / hand×2 on the full cast). Baleful Mastery
    uses the same hook.
  - The STX "still wrong" list in *Suggested next-up tasks* was largely stale:
    Frost Trickster / Eager First-Year / Owlin Shieldmage / Promising Duskmage /
    Rise of Extus / Verdant Mastery / Illuminate History were already faithful.
    Re-verify before picking a sweep target.
- ⏳ **Phasing (CR 702.26) follow-ups**: a permanent that **enters phased out**
  (Reality Ripple-adjacent). **Granted phasing ✅** — `do_phasing` now reads
  computed keywords, so a layer-granted Phasing phases out at the untap step.
  **Mid-combat `Effect::PhaseOut` ✅** — removes the permanent from the combat
  arrays (702.26e). **"When this phases in" triggers ✅** — `EventKind::PhasesIn`
  + `GameEvent::PermanentPhasedIn`. **Linked "until [source] leaves" ✅** —
  `PhaseOut.until_source_leaves` + `CardInstance.phased_out_by`: skipped by
  the untap-step phase-in, returned by `on_left_battlefield` (Out of Time,
  with a time counter per phased permanent). Phased-out permanents surfaced
  per player via `PlayerView.phased_out` + a client HUD chip. The side-zone
  model (`GameState.phased_out`) is the hook.
- ✅ **Changeling (CR 702.73) honored in general type-filter eval** (this run).
  Both `effects/eval.rs` `R::HasCreatureType` sites now OR in
  `has_keyword(Changeling)`, matching the block-restriction path — a Changeling
  satisfies any creature-type filter (tribal lords/anthems, "sacrifice a
  Goblin", type-targeted removal). Avian / Game-Trail Changeling tested.
- ℹ️ **Client build needs system libs** — `apt-get install -y libwayland-dev
  libasound2-dev libudev-dev` unblocks `cargo build/clippy -p
  crabomination_client` in the web sandbox (wayland-sys / alsa-sys / libudev
  build scripts otherwise panic). Install them once per session, then the
  client compiles and clippy runs clean.
- ⏳ **Discovered this run (allied-color card batch):**
  - ✅ **Evoke keyword** — fully wired (`AlternativeCost.evoke_sacrifice` +
    ETB-then-sacrifice on the stack; Solitude/Fury/Mulldrifter tested). Now has
    `shortcut::evoke(mana_cost)` for terse card defs.
  - ✅ **Multikicker + `Value::TimesKicked`** — Wolfbriar Elemental, Joraga
    Warcaller, Apex Hawks, Gnarlid Pack, Skitter of Lizards, Lightkeeper of
    Emeria, Bloodhusk Ritualist all ship on `CastSpellMultikicked`.
  - ✅ **"Draw your second card each turn" triggers** — Faerie Vandal, Mad
    Ratter, Wavebreak Hippocamp already shipped in `decks/modern.rs` (the
    entry was stale; verified by grep).
  - ✅ **Search-by-name / search-an-Aura filters** — Squadron Hawk fetches
    up to three via `HasName`-filtered searches; Heliod's Pilgrim already
    rode `HasEnchantmentSubtype(Aura)`.

- ⏳ **Discovered this run (sagas / attack-tax / pillowfort batch):**
  - **Attack-tax interactive pay** — `AttackTaxToController` auto-pays from the
    active player's floating mana; a wants_ui player needs a real "pay {N}?"
    prompt during declare-attackers (and a per-attacker / partial-pay choice).
  - **DFC / read-ahead Sagas** — `saga_chapters` covers single-faced Sagas only;
    transforming saga-lands (The Everflowing Well) and read-ahead chapter choice
    are still ⏳.
  - ✅ **`AddCardType` one-shot effect** — `Effect::AddCardTypeIndefinitely`
    (layer-4 grant anchored to the permanent); Phyrexian Scriptures ships.
  - ✅ **Variable attack tax** — `AttackTaxToController.amount` was already
    `Value`-typed; Sphere of Safety existed, Collective Restraint now ships
    on `Value::DomainCount`.

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
- ✅ **Client crate now builds/lints in the web sandbox.** The previous
  `wayland-sys` panic was a missing system library; `apt-get install
  libwayland-dev libasound2-dev libudev-dev` lets the client build + clippy
  cleanly. Future runs should build the client too (a stale `CounterType::Ice`
  match arm had slipped in unbuilt — now fixed).
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
- ✅ **Goldvein Hydra death-treasure rider (LKI).** CR 603.10 leaves-battlefield
  LKI ships: `leaves_bf_lki` snapshots the dying object at every removal funnel
  (SBA lethal, destroy/sacrifice, `push_pending_trigger`) and survives until the
  trigger resolves, scoped by `resolving_lki_source`. `Value::PowerOf` /
  `ToughnessOf` read it (priority over the graveyard's printed P/T). Goldvein
  Hydra mints power-many Treasures; Cacophony Scamp / Heartfire Hero ping for
  last-known power. Remaining ⏳: LKI for other characteristics (color/types)
  read by leaves-battlefield bodies, and the tapped-Treasure rider.
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
- ✅ ~~Echo + ETB land destruction (Avalanche Riders)~~ — shipped in
  `decks::echo` (echo is now enforced at upkeep) along with Keldon Vandals,
  Deranged Hermit, Multani's Acolyte, Radiant's Dragoons, Ticking Gnomes,
  Great Whale, and the Urza's Legacy manlands.

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
  - ✅ **More split cards.** Dusk // Dawn, Never // Return, Turn // Burn
    (`ResetCreature` + `BecomeColor`), Hide // Seek (Seek =
    `Search{Target opp, → Exile}` + `GainLife(ManaValueOf(LastMoved))`;
    the searched player's decider answers the pick) all ship; Boom // Bust
    already existed.
  - **Fused targeting** currently assumes each half is single-target (left →
    `target`, right → `additional_targets[0]`); a fusable card with a
    multi-target half would need the slot convention generalized.

- **DSK/MKM gap cards deferred (recent240–241 follow-ups).** Each wants one
  small primitive (verified absent this run):
  - **Miasma Demon** — "discard any number; up to that many target creatures
    each get -2/-2." Needs a reflexive discard whose count caps a
    resolution-time multi-target debuff (`ApplyToTargets.max_targets` is a
    fixed `u8`; make it read a `Value`, or add a reflexive discard-then-targets
    effect).
  - **Grievous Wound** — enchant-*player* Aura with "enchanted player can't gain
    life" + "when dealt damage, they lose half their life." The `PlayerCannotGainLife`
    static and `LoseHalf` effect exist; needs a player-enchant Aura + a
    `PlayerRef::EnchantedPlayer` actor.
  - **Leyline of Transformation** — opening-hand + choose-a-creature-type static
    that adds the type to your creatures *and* spells/cards in other zones.
    Needs a continuous creature-type-add static keyed on `chosen_creature_type`.
  - **Leyline of Mutation** — "pay {W}{U}{B}{R}{G} rather than mana cost for
    spells you cast." Needs a general alt-cost static.
  - **Leyline of Resonance** — "copy your I/S that targets only a single
    creature you control." Needs a copy-on-cast static keyed on target shape.
  - **Leering Onlooker / Rubblebelt Maverick** — graveyard-activated abilities
    (`ActivatedAbility.from_graveyard` + `exile_self_cost` fields exist — wire a
    catalog card through them and confirm the activation path).
  - **Frantic Scapegoat** — the "when other creatures enter, if suspected, you
    may move the suspicion" rider (front haste + ETB-suspect ship; the reflexive
    suspect-another/`ClearSuspected`-self rider is dropped).
  - **Say Its Name** — the three-copy graveyard-exile combo that tutors Altanak
    (front mill+regrowth ships).
  - **Unidentified Hovership / Hedge Shredder / Dissection Tools / Chainsaw /
    Cursed Recording** — exile-remember-owner LTB manifest-dread; mill-lands-to-
    battlefield replacement; equip-cost-as-sacrifice; self-counter-scaled equip
    CDA; cast-count time-counter artifact.

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
- **More AKH/HOU Embalm cards.** Aven Wind Guide ✅ (token-scoped
  `GrantKeyword` anthems), Heart-Piercer Manticore ✅ (`MayDo` →
  `SacrificeAndRemember` → fling). Remaining: Vizier of Many Faces (embalm
  clone — needs the embalm-copy-any-creature path); `fanatic_of_rhonas`
  is missing its real Eternalize {2}{G}{G} — upgrade it.
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
- **Warp alt-cast keyword.** Warp (Mightform Harmonizer, Pinnacle Emissary —
  cast cheaply, exile at end step, recast later — a Suspend/Plot-adjacent
  exile-and-recast) is still dropped on its cards. **Miracle (CR 702.94) ✅** —
  `CardDefinition.miracle` + `maybe_grant_miracle` (first-draw alt-cost grant);
  Metamorphosis Fanatic can now wire its real miracle cost.
  **Offspring {N}** (CR 702.166) now ships
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
- **Daybound / Nightbound DFC transform** (CR 702.146) — ✅ DONE.
  `Keyword::{Daybound,Nightbound}` ride the transform engine (CR 712):
  `set_day_night` flips daybound→nightbound DFCs to their back face when it
  becomes night and back when it becomes day; a daybound permanent entering
  while it's neither day nor night makes it day (702.146e). Ships Village Watch
  // Village Reavers. Remaining ⏳: the "casting a daybound spell makes it day"
  half (only the ETB rule is wired), and the no-spells-cast night entry rule
  beyond the existing CR 502.2 turn check.
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
- **Suspend (CR 702.62) — ✅ DONE (primitive + haste + accelerant +
  granted suspend 702.62e via `Effect::GrantSuspend`/`granted_suspend`, and
  the CR 601.3e suspend-only cast gate `CardDefinition.suspend_only`).**
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
- ✅ **Entwine as a first-class cost** — `Keyword::Entwine(cost)` +
  `GameAction::CastSpellEntwine` ship (CR 702.41); an entwined `ChooseMode`
  runs every mode in order. Tooth and Nail, Barbed Lightning, Rude
  Awakening, Grab the Reins, Promise of Power. (Plunge into Darkness still
  rides its Kicker modelling — migrating it is optional.)
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
  **Renown** ✅ now keys off a real `CardInstance.renowned` flag
  (`Predicate::SourceIsRenowned` + `Effect::BecomeRenowned`), so unrelated
  +1/+1 counters no longer suppress it.
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
- **Stale "two-target prompt ⏳" notes** — several catalog doc-comments still
  claim multi-target sorcery prompts are unavailable; the slot-1+ picker
  (`auto_targets_for_effect_all_slots`) is wired and the bot uses it. Sweep
  and update the remaining notes (Channeled Force done this run).

- ✅ **OrderTriggers server suspend** — `continue_trigger_ordering` parks
  the dispatch in `ResumeContext::TriggerOrder` and sets `pending_decision`
  so a networked `wants_ui` seat is actually prompted; `submit_decision`
  applies the order and finishes via `push_ordered_trigger_candidates`.

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
- **Flashback with an additional cost** — ✅ `CardDefinition::
  flashback_additional_cost` + `cast_flashback` validation/payment; covers
  sacrifice, discard (incl. `DiscardXFromCost`), pay-life and
  exile-from-graveyard riders.
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
- **Theros gods** ✅ — the full THS-block pantheon ships (Heliod, Purphoros,
  Pharika, Karametra, Keranos, Xenagos, Athreos, Ephara, Iroas, Kruphix,
  Mogis, Phenax + the earlier Nylea/Thassa/Erebos), with new primitives
  `PreventDamageToYourAttackers` (Iroas), `UnspentManaBecomesColorless`
  (Kruphix), and `Predicate::AnotherCreatureEnteredControlLastTurn`
  (Ephara — per-turn `creatures_entered_{this,last}_turn` log). Remaining:
  the Theros: Beyond Death two-pip gods.
- **Client build deps** — building the client in the web sandbox needs
  `libwayland-dev libasound2-dev libudev-dev libxkbcommon-dev` (install via
  apt). Once present `cargo build/clippy -p crabomination_client` works.

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

## MagicCompRules coverage audit

Periodic spot-check against the rules document (`MagicCompRules_20260417.txt`).
One line per rule: status (✅ wired · 🟡 partial · ⏳ todo) plus the still-open
gap. The full per-clause accounting (every sub-rule, code line, and test name)
was elided in a doc-compaction pass — recover it from
`git log -p -- TODO.md`. Markers are a point-in-time read; re-verify before
picking an item up.

### Done (✅) — wired
- ✅ **CR 603.2 — "whenever you attack" trigger conditions** — the `YouAttack`
  dispatch in `combat.rs` now evaluates the ability's `EventSpec.filter` at fire
  time and applies the attack / permanent trigger doublers, both of which the
  path previously dropped (Military Intelligence, Dollmaker's Shop;
  `cr_recent81::cr_603_2_*`).
- ✅ **CR 400.7 — cast provenance doesn't follow a card between zones** — the
  leave-the-battlefield reset now clears `cast_from_hand` / `cast_from_exile` /
  `cast_from_library` / `cast_via_flashback` / `cast_from_suspend` /
  `cast_from_escape` alongside the granted-ability and per-object activation
  limits, so a reanimated permanent isn't still "cast from your hand" (Phage
  the Untouchable; `classic_sets/lgn::phage_only_survives_a_hand_cast`).
- ✅ **CR 603.4 — self-source ETB intervening 'if'** — `fire_self_etb_triggers`
  reads the trigger's condition against the entering permanent's cast flags
  (kicked / bargained / X / cast-from-hand). It was silently ignored for all 35
  filtered self-ETB cards (`cr_recent81::cr_603_4_*`).
- ✅ **CR 709.5c — Room door designations** — a re-locked door's abilities go
  inert (`GameState::relock_room_door` rebuilds the live definition), the
  distinct-name door count is a real predicate, and door-unlock payments carry
  a `SpellKind` so Room-restricted mana can fund them
  (`cr_recent81::cr_709_5c_*`).
- ✅ **CR 613.4 — a layer-7a CDA sees layer-4 land types** — counter-keyed
  requirements resolve off the card instance in every zone, so a `*/*`
  land-count body counts lands granted a type this turn (Eluge;
  `cr_recent81::cr_613_4_*`).
- ✅ **CR 506.2 — "can't be attacked"** — `Keyword::CantBeAttacked` +
  `permanent_cant_be_attacked`, checked at the planeswalker and battle
  attack-target gates and by the bot's walker-redirect picker (The
  Aetherspark; `cr_recent79::cr_506_2_*`).
- ✅ **CR 118 / 305 — playing a card from exile** — a `may_play_until` grant
  now covers land plays, not just casts (`may_play_grant_for` + the
  `play_land` exile branch; the bot spends impulse lands before they expire).
  `cr_recent79::cr_118_*`.
- ✅ **CR 614 — as-enters before enters-with-counters** — `as_enters_effect`
  resolves ahead of the counter specs, so a count can read what it did
  (Mimeoplasm's three counters per exiled creature;
  `cr_recent79::cr_614_*`).
- ✅ **CR 716.2 — Class-level statics reach the cost scan** —
  `cost_reduction_for_spell` unwraps `WhileClassLevelAtLeast` (Artist's
  Talent level 2; `cr_recent79::cr_716_2_*`).
- ✅ **CR 407 — Ante** — `Zone::Ante` + `Player.ante` + `ZoneDest::Ante`;
  `GameState::begin_ante_game` does the 407.2 opening ante and
  `award_ante_to` the winner-takes-all (fired from the game-over SBA).
  407.3's "remove this card from your deck" rides
  `CardDefinition.ante_only` → `DeckError::AnteCardOutsideAnteGame`, and
  `Effect::ExchangeOwnership` is the only ownership change in the engine.
  All nine printed ante cards ship in `sets::ante`; tests
  `core_rules/cr_recent72::cr_407_*`. ⏳ residual: Darkpact picks the first
  ante card rather than targeting one, and Bronze Tablet folds its
  exile-both into a sacrifice.
- ✅ **CR 211 / 212 / 313 / 902 — Vanguard** — `CardType::Vanguard` +
  `CardDefinition.{hand_modifier, life_modifier}`; `GameState::seat_vanguard`
  seats the avatar in the command zone and applies both modifiers (and its
  `NoMaximumHandSize` static). Its abilities function from there:
  activated via `ActivatedAbility.from_command_zone`, step triggers via
  `fire_step_triggers`, cast triggers via the SpellCast gather, other events
  via `dispatch_triggers_for_events`. `sets::vanguard` (8 avatars);
  `core_rules/cr_recent66`. ⏳ residual: general statics from the command zone.
- ✅ **CR 102 — Players** — two-player opponents, team membership and the
  no-teams "your team" collapse now carry conformance tests
  (`cr_102_{2,3,4}_*`).
- ✅ **CR 502.4 — "permanents don't untap"** — `StaticEffect::PermanentsDontUntap`
  short-circuits `do_untap` for every seat while still clearing summoning
  sickness (Mist of Stagnation; `cr_502_4_global_dont_untap_stops_every_seat`).
  CR 502.2's active-player-only untap is covered by
  `cr_502_2_only_the_active_player_untaps`.
- ✅ **CR 104.3d — can't lose / can't win** — Angel's Grace's turn-scoped
  `Player.cant_lose_this_turn` (+ damage-to-1 floor) and the permanent
  `ControllerCant{Lose,Win}Game` statics (Platinum Angel, Abyssal Persecutor)
  gate the SBA loss loop, `lose_to_empty_draw`, and the WinGame/LoseGame/
  PayOrLoseGame effects; Worship rides the same floor
  (`DamageWontReduceControllerLifeBelowOne`). `tests/recent109.rs`.
- ✅ **CR 615 — blocked-creature prevention** —
  `StaticEffect::PreventAllDamageToThisFromBlocked` lives in
  `apply_prevention_shields`, so it covers noncombat damage from a blocked
  attacker too (Wall of Vapor, Wall of Shadows;
  `cr_615_blocker_prevents_damage_from_the_creature_it_blocks`).
- ✅ **CR 113.11 — "can't have or gain [keyword]"** —
  `Modification::CantHaveKeyword` strips after every grant regardless of
  timestamp (the Theros Archetype cycle; `cr_113_11_*`).
- ✅ **CR 702.19c/f — trample over planeswalkers** —
  `Keyword::TrampleOverPlaneswalkers` spills excess past loyalty to the
  walker's controller; plain trample doesn't (Thrasta; `cr_702_19c_*`).
- ✅ **CR 702.2 / 702.4 / 702.111 — Deathtouch, Double Strike, Menace combat
  conformance** — granted deathtouch makes 1 combat damage lethal (Corpse
  Blockade), double strike deals in both combat-damage steps, and Menace
  rejects a lone blocker (`cr_recent6::cr_702_{2b,4b,111b}_*`).
- ✅ **CR 702.150 — Compleated** — `Keyword::Compleated` + `{A/B/P}`
  PhyrexianHybrid pips; life paid to Phyrexian pips at cast drops the entering
  planeswalker's loyalty (tests `compleated_*`, ONE's five compleated walkers).
- ✅ **CR 602.5g — summoning-sick {T} abilities** — creatures can't activate
  tap-cost abilities (mana abilities included) the turn they arrive unless
  hasty or exempted (`ControllerCreatureAbilitiesAsThoughHaste` — Tyvar);
  the auto-tap payment path honors it too.
- ✅ **CR 702.65 — Aura swap** — `Effect::AuraSwapFromHand` exchanges the Aura
  with a hand Aura on the same host (Arcanum Wings;
  `cr_702_65_aura_swap_exchanges_with_hand`).
- ✅ **CR 702.71 — Fortify** — `equip()` accepts Fortifications (land targets,
  CR 702.71c); `CardDefinition::has_fortify`. Darksteel Garrison
  (`cr_702_71_*`, `tests/recent110.rs`).
- ✅ **CR 702.24 — cumulative upkeep prompt** — a `wants_ui` controller gets a
  real pay-or-sacrifice trigger (`Effect::CumulativeUpkeepPayOrSacrifice`,
  mana/life kinds; `cr_702_24_wants_ui_prompt_pays_scaled_upkeep`); echo got
  the same shape (`Effect::EchoPayOrSacrifice`, CR 702.29).
- ✅ **CR 601.3e — no mana cost** — `CardDefinition.no_mana_cost` rejects the
  pay-the-cost cast path generally (replaces `suspend_only`; Ancestral
  Vision / Lotus Bloom / Crashing Footfalls / Living End / Restore Balance /
  Wheel of Fate / Hypergenesis); `{0}` stays castable
  (`cr_601_3e_no_mana_cost_rejected_but_zero_cost_castable`).
- ✅ **CR 702.29b — echo re-armed on control change** — the
  `GameState::change_control` funnel resets `echo_paid` and applies CR 302.6
  sickness at every steal/exchange/revert site
  (`cr_702_29b_stolen_echo_owed_by_new_controller`).

One line per wired rule; implementation detail (code symbols, tests) elided —
recover from `git log -p -- TODO.md`. A few rows carry a residual ⏳ gap inline.

- ✅ CR 701.31 — Voting / "will of the council" (`Effect::WillOfTheCouncilExile`:
  each player votes for one matching permanent, every permanent tied for most
  votes is exiled; untargeted so it ignores hexproof/shroud — Council's Judgment)
- ✅ CR 702.16 — Protection from instants (`Keyword::ProtectionFromInstants`,
  cast-time targeting gate) + protection from everything
  (`Keyword::ProtectionFromEverything`, every protection-check site) — Hexdrinker
- ✅ CR 603.4 — once-per-turn / per-subject trigger budget is now charged only
  *after* the intervening filter passes (Faerie Mastermind's "second card each
  turn" via `Predicate::PlayerDrewAtLeastThisTurn` + `once_per_turn`).
  Turn-scoped "until end of turn, whenever a creature you control dies / deals
  combat damage to a player" delayed triggers now ship
  (`DelayedKind::CreatureYouControlDies/DealsCombatDamageThisTurn` +
  `Effect::CreaturesYouControlDying/DealingCombatDamageThisTurn`, expiring at
  cleanup) — Waltz of Rage, Mistway Spy. Tests in `recent240`/`recent241`.
- ✅ CR 702.161 — Living metal (`Keyword::LivingMetal` emits a layer-4 Creature
  type while the controller is the active player, no crew — Slicer's Vehicle
  side; `cr_702_161_living_metal_animates_on_your_turn_only`)
- ✅ CR 702.162 / 701.28 — More Than Meets the Eye / convert
  (`AlternativeCost.converted` → `CardInstance.cast_converted`, flipped to the
  back face at ETB before the first SBA; `cr_702_162_*`)
- ✅ CR 702.148 — Cleave
- ✅ CR 701.15g — "it can't be regenerated this turn". `Effect::CantBeRegeneratedThisTurn`
  sets a transient `CardInstance.cant_regenerate_this_turn` consulted at both
  regeneration sites (the `Destroy` funnel and the lethal-damage SBA), so existing
  shields go inert and new ones do nothing (Rage of Purphoros). Surfaced to the
  client as `PermanentView.cant_regenerate` — the tooltip stops promising a save.
- ✅ CR 401.6 — cast-from-library provenance. The library-top cast hops the card
  through hand, so `GameState.casting_from_library_top` preserves the true origin
  for `CardInstance.cast_from_library` / `Predicate::CastSpellFromLibrary` (Melek).
- ✅ CR 702.47 — Splice
- ✅ CR 704.5k — world rule
- ✅ CR 614.5 / 701.10f — mana-production multipliers compose
- ✅ CR 702.64 — Absorb
- ✅ CR 704.5y — Role uniqueness SBA
- ✅ CR 701.30 — Clash, seat-routed
- ✅ CR 702.104 — Tribute, seat-routed
- ✅ CR 700.4 — "dies" under graveyard→exile replacements
- ✅ CR 702.31 — Horsemanship
- ✅ CR 702.14c — filtered landwalk (`Keyword::LandwalkFiltered` — artifact
  landwalk on Vectis Gloves; `cr_702_14c_artifact_landwalk`)
- ✅ CR 701.30 — Clash
- ✅ CR 510.1d — full damage assignment
- ✅ CR 701.37 / 712.16 — Meld
- ✅ CR 702.121 — Melee (`Keyword::Melee`; declare-attackers pumps +1/+1 per
  distinct opponent attacked this combat — Menagerie Liberator;
  `cr_702_121_melee_pumps_per_opponent_attacked`)
- ✅ CR 702.146 — Disturb
- ✅ CR 104.3c (with the 104.2 win override)
- ✅ "When this card is milled" triggers
- ✅ CR 701.9 — Discard *batching*. `GameEvent::DiscardedBatch { player, count }`
  / `EventKind::DiscardedOneOrMore` fire one "you discarded one or more cards"
  event per effect resolution (alongside the per-card `CardDiscarded`s), carrying
  the count via `Value::TriggerEventAmount` — Magmakin Artillerist deals that much
  to each opponent, once. Emitted from `resolve_effect` off the
  `cards_discarded_per_player_this_resolution` scratch; test
  `cr_701_9_discard_batch_fires_once_with_count`. The CR 514.1 cleanup
  discard-down now emits the batch too (both the deterministic and UI-resume
  paths; `cr_514_3_cleanup_discard_fires_batch_trigger`). Activation-cost discards
  (`discard_cost`, `discard_hand_cost`) emit the batch from `activate_ability`
  (`cr_701_9_cost_payment_discard_fires_the_batch`). Cycling and landcycling
  emit it too (`cr_701_9_cycling_fires_the_discard_batch`). Remaining: the other
  spell-level "discard this card" costs.
- ✅ CR 701.13 — Mill (incl. `Effect::MillThenToHand { amount, filter }` — mill,
  then pick one card matching `filter` from those milled this way to hand;
  Cache Grab, `SelectionRequirement::PermanentCard`; test
  `cache_grab_returns_a_milled_permanent`). `EventKind::CardMilled` binds its
  trigger subject to the milled card so "a creature card put into a graveyard
  from a library" triggers can filter on the milled card's type (Dreadhound).
- ✅ CR 502.2 / 731 — Day/Night transition trigger. `EventKind::DayNightChanged`
  (matched to `GameEvent::DayNightChanged { was_transition }`, true only on a
  real day↔night flip) fires "whenever day becomes night or night becomes day"
  with `EventScope::AnyPlayer` (Brimstone Vandal).
- ✅ Coven (Innistrad ability word) — `Predicate::CovenActive { who }` = control
  3+ creatures with different (computed) powers; gates attack triggers and
  "activate only if …" abilities (Sigarda Champion of Light, Dawnhart Mentor,
  Sungold Sentinel).
- ✅ CR 702.104 — Tribute
- ✅ CR 728 — Ending the Turn
- ✅ CR 500.7 — additional phases/steps. `AdditionalCombatPhase` (combat) plus
  new `Effect::AdditionalEndStep` (Y'shtola Rhul): `additional_end_steps` loops
  the End step in `advance_step`. First-occurrence gates via
  `combat_phases_this_turn` / `end_steps_this_turn` +
  `Predicate::IsFirst{CombatPhase,EndStep}ThisTurn` stop the extra phase from
  re-triggering (Genji Glove, Y'shtola). Repeated phases surfaced via
  `ClientView.extra_phase`. Tests `cr_500_7_*`, `cr_514_2_eot_pump_*`.
- ✅ CR 120.3 / 104.3c — drawing from an empty library via a draw *effect*
  (not just the draw step) loses the game, recorded as `LossCause::Decked`
  (`Effect::Draw` → `lose_to_empty_draw`; test `cr_120_3_overdraw_*`).
- ✅ CR 701.19 — Searching (incl. `Effect::SearchUpToN` count-search — Nylea's Intervention, Deathbellow War Cry; test `cr_701_19_search_up_to_n_picks_matches_only`)
- ✅ CR 714.4 — DFC sagas
- ✅ CR 702.103 — Jump-start
- ✅ CR 707.2 — continuous copies
- ✅ CR 707.10 — a spell copy is put on the stack, not cast: it bumps neither
  the storm count nor any cast watcher (`cr_707_10_spell_copy_is_not_cast`).
  `Effect::CopySpellForEachOtherTarget` (Radiate) rides the same path.
- ✅ CR 611.2 — a static under a duration/predicate wrapper surfaces its
  *granted activated ability* too, not just its granted trigger and its P/T
  (`granted_abilities_for` and the `PumpSelfByValue` layer walk both go
  through `active_static`; `cr_611_2_wrapped_grant_surfaces_only_while_open`).
- ✅ CR 601.2b — "discard X cards" as an additional cast cost is concretized
  on the main cast path, not only on flashback
  (`cr_601_2b_discard_x_cost_is_paid_when_cast_from_hand`, `_rejects_an_empty_hand`).
- ✅ CR 702.43 — Domain
- ✅ CR 702.6e — Equipment-granted triggered abilities
- ✅ CR 702.6 — Equip ability fidelity: sorcery-speed gate, equip-at-instant
  (`ControllerEquipAtInstantSpeed` — Leonin Shikari), equip-cost reduction
  (`EquipCostReduction` — Auriok Steelshaper), protection-gated attach
  (702.16f). Tests in `tests/recent12.rs`.
- ✅ CR 510.2 — combat damage to a creature dispatch
- ✅ CR 509.1d — block tax
- ✅ CR 702.46 — Cipher
- ✅ CR 702.41 — Affinity (for artifacts)
- ✅ CR 205.4g — Snow permanents
- ✅ CR 604.3 — Characteristic-defining P/T (artifact count)
- ✅ CR 702.176 — Bargain
- ✅ CR 601.2b — variable-sacrifice cost reduction
- ✅ CR 702.74 — Evoke
- ✅ CR 603.3d once-per-turn + exile triggers
- ✅ Cast-from-exile rider
- ✅ CR 702.26 — Phasing
- ✅ CR 702.77 — Champion
- ✅ CR 702.56 — Forecast
- ✅ CR 603.3b — Same-controller trigger ordering
- ✅ CR 702.124 — Addendum
- ✅ CR 601.2f — generic cost reduction (graveyard-Affinity)
- ✅ CR 702.32 — Kicker
- ✅ CR 702.164 — Backup
- ✅ CR 702.95 — Soulbond
- ✅ CR 702.134 — Mentor
- ✅ CR 702.105 — Dethrone
- ✅ CR 702.130 / 702.39 / 702.46 — Afflict / Provoke / Soulshift
- ✅ CR 702.68 / 702.69 / 702.70 — Frenzy / Gravestorm / Poisonous
- ✅ CR 702.139 — Revolt
- ✅ CR 702.79 / 702.92 — Persist / Undying
- ✅ CR 702.66 — "Spells you cast have delve" static
- ✅ CR 709 — Split Cards
- ✅ CR 510 — Combat Damage Step
- ✅ CR 120.10 — Excess damage — `Effect::DealDamageExcessToController` deals N
  to a creature and spills the overkill (past its remaining toughness) onto its
  controller (Flame Spill; `flame_spill_excess_hits_controller`). Combat
  damage→token scaled by `Value::TriggerEventAmount` (Quartzwood Crasher,
  `DealsCombatDamageToPlayer`; CR 510.2/119.3).
- ✅ CR 114 — Emblems
- ✅ CR 702.179 — Freerunning. Alt cost gated on `Predicate::DealtCombatDamageToPlayerThisTurn` (`Player.dealt_combat_damage_to_player_this_turn`, set in `fire_combat_damage_to_player_triggers`). ACR batch in `decks::freerunning` (Brotherhood Ambushers, Merciless Harlequin, Achilles Davenport, Eagle Vision, Distract the Guards, Chain Assassination, Restart Sequence, Viewpoint Synchronization, Escape Detection, Overpowering Attack). The "with an Assassin or commander" sub-clause is approximated as "with any creature." ⏳ remaining cards: Petty Larceny (exile-and-play-from-exile + any-color), Monastery Raid (Freerunning {X} + was-freerun provenance rider).
- ✅ CR 603.8 — state trigger "a player other than the owner controls it": the
  SBA pass pushes a latched sacrifice-and-burn trigger controlled by the thief
  (`CardDefinition::sacrifice_and_burn_when_stolen` — Bronze Bombshell;
  `bronze_bombshell_punishes_theft`).
- ✅ CR 121.2a — empty-hand draw replacement (`StaticEffect::EmptyHandDrawBonus`,
  consulted in `draw_one` only when the hand is empty at draw time — Blood
  Scrivener; `blood_scrivener_empty_hand_draw`).
- ✅ CR 702.99 — Extort granted to other creatures via
  `StaticEffect::GrantTriggeredAbility` (each instance triggers separately —
  Pontiff of Blight; `pontiff_of_blight_grants_extort`).
- ✅ CR 712 — Transforming Permanents
- 🟡 CR 708 — Face-Down Permanents
- ✅ CR 702.146 — Daybound/Nightbound
- ✅ CR 702.114 — Devoid
- ✅ CR 702.115 — Ingest
- ✅ CR 701.x — Process
- ✅ CR 208.2 / 613.7b — Set base P/T
- ✅ CR 702.21 — Ward (discard / life / compound `WardCost::ManaAndLife` — Ovika, Gisa / `WardCost::LifeSourcePower` = source's power — Phyrexian Fleshgorger; `cr_702_21_*`)
- ✅ CR 702.160 — Prototype (`CardDefinition.prototype` + `GameAction::CastPrototype`; the BRO cycle; `cr_702_160_*`)
- ✅ CR 702.125 — Undaunted: `StaticEffect::SelfCostReducedPerOpponent` folded into `cost_reduction_for_spell` (generic-only, {1} per opponent). Sublime Exhalation, Curtains' Call, Coastal Breach (`decks::recent12`). Tests in `tests/recent12.rs`.
- ✅ CR 702.163 — For Mirrodin! (living-weapon-shaped ETB: mint a 2/2 red Rebel + self-attach — Barbed Batterfist, Goldwarden's Helm; `cr_702_163_*`)
- ✅ CR 702.156 — Ravenous (`enters_with_counters: XFromCost` + an ETB draw gated on counter-count ≥ 5 — the W40K Tyranids Tyrant Guard / Termagant Swarm; `cr_702_156_*`)
- ✅ CR 702.38 — Amplify N: `enters_with_counters` scaled by `Value::CardsInHandMatching` (count the matching-type cards in hand, ×N; all auto-revealed). Canopy Crawler / Feral Throwback / Kilnmouth Dragon in `decks::recent17`; `cr_702_38_*`.
- ✅ CR 301.5 — equipped-by-count: `SelectionRequirement::EquippedByAtLeast(n)` gates `SelfHasKeywordWhile` (Balan's double strike); `Predicate::SourceIsEquipped` gates `PumpTeamIf` (Auriok Steelshaper's while-equipped anthem). `cr_301_5_*`, `tests/recent12.rs`.
- ✅ CR 509.1c — must-block grant (`Effect::MustBlockSource`, untap-free Provoke — Matsu-Tribe Decoy; `cr_509_1c_matsu_tribe_decoy_*`)
- ✅ CR 602.5b — Return-to-hand activation cost
- ✅ CR 602.5c — "Abilities can't be activated"
- ✅ CR 119.3 — Life gained this turn
- ✅ CR 603.3d — "Triggers only once each turn"
- ✅ CR 602.5 — "Only your opponents may activate"
- ✅ CR 602.5b — Discard-self activation cost
- ✅ CR 702.97 — Scavenge
- ✅ CR 702.53 — Transmute
- ✅ CR 122 / 614.13 — chosen-type enters-with-counter
- ✅ CR 122 — per-count enters-with-counter (`StaticEffect::TypeEntersWithCountersPerControlled` — Giada scales entering Angels by your Angel count; `giada_scales_entering_angels`)
- ✅ CR 724 — Monarch-linked exile (`Effect::ExileUntilOpponentMonarch` + `ExileLink.monarch_guard`; Palace Jailer returns the exile when the monarchy leaves, driven by `set_monarch`; `palace_jailer_*`)
- ✅ CR 602.5b — remove-counters-from-among-creatures activation cost (`ActivatedAbility.remove_counter_among_filter` — Hopeful Initiate; `hopeful_initiate_*`)
- ✅ CR 702.44 — Sunburst (`enters_with_counters: (PlusOnePlusOne, Value::ConvergedValue)` — Suntouched Myr)
- ✅ CR 508.1d — "attacks each combat if able" (`Keyword::MustAttack`) is enforced in `declare_attackers`; regression `cr_508_1d_must_attack_creature_is_forced_to_attack` (Volatile Rig)
- ✅ CR 705.1 — coin-flip win/loss branches (`Effect::FlipCoin`); Volatile Rig's dealt-damage flip, heads-branch regression `cr_705_1_volatile_rig_survives_flip_on_heads`
- ✅ CR 514.2 — "until end of turn" grants end at cleanup; a land-granted haste (Racecourse Fury) clears via `expire_end_of_turn_effects` — `cr_514_2_racecourse_haste_expires_at_cleanup`
- ✅ CR 701.x — impulse-exile-until-duplicate-name (`Effect::ExileUntilDuplicateName` — Tainted Pact)
- ✅ CR 702.96 — Overload via alt-cost `effect_override` (Mizzix's Mastery)
- ✅ CR 702.85 — Heroic (`shortcut::heroic` + `Predicate::CastSpellTargetsSource`)
- ✅ CR 700.5 — devotion cost reduction (`StaticEffect`-gated generic reduction)
- ✅ CR 702.80a / 702.90e / 702.2c — wither / infect / deathtouch on damage
- ✅ CR 702.78 — Conspire (tap-two-creatures additional cost → copy spell)
- ✅ CR 702.177 — Exhaust (activated ability usable only once per game)
- ✅ CR 702.189 — Firebending. `Keyword::Firebending(n)`; an attack-triggered
  mana ability adding n {R} that survives step/phase mana emptying until end of
  combat (`Player.firebending_kept_red`, re-seeded by `empty_mana_pools`,
  cleared at the end-of-combat empty). `decks::recent22`: Jeong Jeong the
  Deserter, Ran and Shaw, Sozin's Comet (grants firebending 5). **Firebending X
  = source's power** ✅ via `Keyword::FirebendingPower` (Firebending Student) —
  combat reads the attacker's computed power at attack time. **Firebending X
  = creatures you control** ✅ via `Keyword::FirebendingCreaturesYouControl`
  (Sun Warriors).
- ✅ CR 702.190 — Sneak. `Keyword::Sneak(cost)` + `shortcut::sneak`
  (`AlternativeCost`: flash timing gated on `CurrentStepIs(DeclareBlockers)`,
  `return_to_hand` an unblocked attacker). `SelectionRequirement::IsUnblocked`.
  Server `alt_cost_available` greys it out without a returnable attacker.
  `decks::recent22`: Donatello's Technique, Jennika's Technique.
- ✅ CR 702.54 — Bloodthirst. `Keyword::Bloodthirst(n)` display variant added
  over the existing `shortcut::bloodthirst` ETB-counter trigger
  (`Predicate::PlayerDamagedThisTurn`). Retrofitted onto the shipped cards;
  `decks::recent22` adds Bloodrage Vampire, Furyborn Hellkite.
- ✅ CR 702.55 — Haunt (already shipped: `Effect::HauntCreature` +
  `DelayedKind::WhenHauntedCreatureDies`; GPT creatures, tests in gpt.rs).
- ✅ CR 711 — Flip cards (whole CHK cycle; `flip_when_has_keyword` CR 603.8
  state-flip, `DamagedBySourceThisTurn` death-watch, two-target aura-move)
- ✅ CR 601.2c — two-target activated abilities (`ActivateAbility.additional_targets`
  threaded to `StackItem::Trigger`; Autumn-Tail, Kitsune Sage)
- ✅ CR 702.185 — Warp. `AlternativeCost.warp` + `CardInstance.warped`; cast for
  the warp cost, then a `NextEndStep` delayed trigger exiles the permanent and
  grants a `WhileExiled` may-play (recast from exile, full cost — matches
  702.185a). Sets `Player.warped_spell_this_turn` (702.185c). `shortcut::warp`;
  `eoe` batch; tests in `tests/eoe.rs`.
- ✅ CR 207.2c — Void (ability word). `Predicate::VoidActive { who }` = a nonland
  permanent left the battlefield this turn (`GameState.nonland_permanent_left_bf_
  this_turn`, set at every battlefield-leave funnel) OR `who` warped a spell this
  turn. Decode Transmissions (`Effect::If`), Elegy Acolyte / Kavaron Skywarden
  (end-step intervening-if). Tests in `tests/eoe.rs`.
- ✅ CR 111.10u — Lander token. `tokens::lander_token` ({2},{T},Sac: fetch a
  basic land tapped). Biomechan/Biotech/Galactic/Glacier/Kav makers, Edge Rover
  (each player), Dauntless Scrapbot, Emergency Eject (target's controller).
- ✅ CR 702.184 + 721 — Station / Spacecraft. `shortcut::station` (tap another
  untapped creature, sorcery-speed: charge counters = its power via
  `Value::TappedForCostPower` carried by `Effect::WithTappedPower`).
  `CardDefinition.station` bands (`StationBand{min,keywords,pt}`) grant keywords
  (L6) and, at a `{N+}` P/T threshold, add the Creature type (L4) + base P/T
  (L7a CDA). `ArtifactSubtype::{Spacecraft,Lander}`. 18 Spacecraft in `eoe`;
  tests in `tests/eoe.rs`. Counter-gated static **and** triggered bands now ship
  (`StationBand.statics` / `.triggers`).
- ✅ CR 701.53 — Incubate. `Effect::Incubate { who, amount }` mints an Incubator
  double-faced token (`TokenDefinition.back_face`) with N +1/+1 counters;
  `{2}: Transform` flips it to a 0/0 Phyrexian artifact creature (→ N/N). ONE
  cards in `sets::one` + Sunfall's Incubate-X; tests in `tests/one.rs`.
- ✅ CR 111 / 614.13 — `Effect::CreateToken` now mints for **every** matched
  player (EachPlayer / EachOpponent), with each player's own token-doublers
  applied; fixes a latent single-player bug (gift cycle, Edge Rover).
  Transient tokens: `Effect::ExileLastCreatedTokensAtNextEndStep` registers a
  `DelayedKind::NextEndStep` exile per token minted this resolution (chain it
  after `CreateToken` in a `Seq`) — Valduk, Keeper of the Flame now exiles its
  Elemental tokens at the next end step (faithful).
- ✅ CR 310 — Battle / Siege. `CardType::Battle` + `BattleSubtype::Siege`, defense
  counters (310.7), protector choice (310.6), attack-your-own-Siege
  (`AttackTarget::Battle`), combat **and noncombat** damage strip defense
  counters (310.10 — noncombat path added in `deal_damage_to_from`; Onakke
  Javelineer, `onakke_javelineer_damages_a_battle`), defeat→exile/transform SBA
  (704.5x via `defeat_battle`). 6 MOM Invasions in `decks::mom`. ⏳ multiplayer
  protector choice.

- ✅ **CR 609.4b — "spend mana as though it were mana of any color"** —
  `StaticEffect::PlayersMaySpendManaAsAnyColor` (Mycosynth Lattice) relaxes a
  cost's coloured/hybrid pips to generic at the payment funnel
  (`GameState::relax_cost_colors`, mirrored in the bot's affordability probe);
  the printed cost and the mana actually spent are unchanged
  (`cr_recent42::cr_609_4b_*`).
- 🟡 **CR 616 — Interaction of Replacement and/or Prevention Effects** —
  616.1c/616.1g ✅: the enters-as-a-copy replacement outranks the enters-tapped
  one, so tappedness is re-decided against the copied characteristics
  (`reapply_enters_tapped_after_copy`; Clone of Rusted Sentinel enters tapped —
  `cr_recent42::cr_616_1c_*`). **616.1e player choice ✅ for draws** — the
  competing "dig instead of drawing" replacements (Parallel Thoughts, Tomorrow,
  Archmage Ascension, Abundance) are enumerated as `DrawDig` and the drawing
  player picks which applies (`choose_draw_replacement` / `apply_draw_dig`); a
  declined optional pick drops out and the choice is offered again. A headless
  seat keeps the canonical order. `cr_recent74::cr_616_1e_*`. Remaining:
  616.1a self-replacement priority, and the same player choice for the
  non-draw replacement families (ETB, damage, counters).

### Partial (🟡) — remaining gap noted
- ✅ **CR 702.43 — Modular.** `Keyword::Modular(N)` is a real marker keyword
  alongside `enters_with_counters` + `shortcut::modular_dies()`;
  `SelectionRequirement::HasModular` is the value-agnostic filter (Arcbound
  Overseer). Tests `cr_recent41::cr_702_43*`.
- ✅ **CR 702.44 — Sunburst.** `Keyword::Sunburst` resolves in the
  permanent-entry path off the cast's converge count: +1/+1 counters when it
  enters as a creature, charge counters otherwise. A real CR 614.12
  replacement (a counter lock blanks it), so Pentad Prism's old ETB trigger is
  gone. "Modular—Sunburst" (Arcbound Wanderer) composes. Tests
  `cr_recent41::cr_702_44*`.
- ✅ **CR 704.8 — LKI across one SBA sweep.** Persist/Undying read the ±1/±1
  pile from before the 122.3 annihilation, so Young Wolf with a +1/+1 counter
  that takes three -1/-1 counters stays dead. Tests `cr_recent41::cr_704_8*`.

- ✅ **CR 808 — Team vs. Team** — teams partition seats (`assign_teams`,
  `same_team`, `teammates`) with per-seat resources (no shared hand, mana or
  life — `Team.shared_life` stays `None`, unlike CR 810 2HG), and 808.3a's
  attack-multiple-players default falls out of `declare_attackers` rejecting a
  teammate as defender. `cr_recent74::cr_808_*`. ⏳ residual: 808.4's
  center-seat first-player rule isn't modeled (seat 0 always starts).
- 🟡 **CR 509.2 / 510.1c — Banding** — a banding blocker routes the blocked
  attacker's damage order + assignment to the defending player, including
  banding *granted during the combat* (Wall of Caltrops' block trigger;
  `cr_509_2_banding_blocker_lets_defender_assign_damage`,
  `cr_recent74::cr_509_2_banding_gained_midcombat_still_routes_assignment`).
  Attacking bands (`declare_attackers_banded`) and "bands with other"
  (`Keyword::BandsWithOther` + `bands_with_other_qualities`) both ship.
  Remaining: the band-blocks-multiple damage-distribution corner.
- 🟡 **CR 303 — Auras** — characteristic-overriding Auras ✅ (`EquipBonus.{set_base_pt,set_card_types,set_creature_types,set_colors,remove_abilities}` install layer 4/5/6/7b continuous effects on the host — Ichthyomorphosis "0/1 blue Fish, no abilities", One with the Stars "becomes an enchantment", Heliod's Punishment "loses abilities + can't attack/block"; removal is ordered before the aura's own keyword grants so they survive — test `cr_613_aura_set_base_pt_then_counter`). **Aura/Equipment-granted step triggers ✅** (CR 702.6e — `fire_step_triggers` now dispatches `EquipBonus.triggered_abilities` whose kind is a step, sourced on the host and scoped to the host's controller; Pillory of the Sleepless's "enchanted creature has: at your upkeep, you lose 1 life" — `cr_702_6e_aura_granted_upkeep_trigger_keys_on_host_controller`). **CR 303.4a "enchant player" ✅** — `CardInstance.attached_to_player` anchors an Aura to a seat, `PlayerRef::EnchantedPlayer` and `EventScope::EnchantedBySource` read it, `StaticEffect::PumpPT` takes a `Selector::ControlledBy` anthem scope, and the orphan-Aura SBA leaves player-Auras alone (`catalog::sets::curses`; tests `core_rules/cr_recent37`). **CR 702.103f ✅** — a bestowed Aura that is unattached *or* attached to an illegal object reverts to a creature instead of dying. Remaining: replacement-style Aura ETB (enters attached under another rule).
- 🟡 **CR 603.10 — Last-Known Information** — full LKI for mid-resolution stack sources (e.g. lifelink 702.15c). Aura death LKI is now path-independent: `remove_to_graveyard_with_triggers` records `auras_at_death` before the host leaves, so `EventScope::EnchantedBySource` triggers fire on the destroy/sacrifice funnel as well as the lethal-damage SBA (`cr_603_10_enchanted_dies_trigger_fires_on_a_sacrifice`). (CR 603.6d "leaves the battlefield" self-source triggers now also fire on the lethal-damage SBA path, not just the destroy/sacrifice path — Thought-Knot Seer's LTB draw.) Sac-as-cost activated abilities that read the sacrificed source's own counters at resolution now stash `leaves_bf_lki` during cost payment (it outlives the per-dispatch `died_card_snapshots` clear) so `Value::TotalCountersOn { This }` reads the last-known total — Twitching Doll's "Spider per counter on it" (`twitching_doll_nests_then_sacs_for_spiders`). `SelectionRequirement::ControlledByYou` now falls back to `died_card_snapshots` for the LKI controller, so a graveyard-scoped "a creature you control dies" trigger fires only for your creatures — Furious Forebear (`cr_603_10_died_creature_controller_read_from_lki`). CR 603.10a self-death: both self-death funnels (SBA lethal-damage + destroy/sacrifice) now evaluate a filtered `YourControl`/`AnyPlayer` death trigger's `.with_filter` against the dying creature via the death snapshot, and the destroy/sacrifice path fires self-inclusive scopes (was SelfSource-only) so an aristocrat drains for its own sacrifice (`cruel_celebrant_drains_on_its_own_sacrifice`).
- 🟡 **CR 704 — State-Based Actions** — Saga SBA ✅ (`saga_chapters` reach
  final chapter → sacrifice, unless a chapter ability is still on the stack);
  spell-copy-off-stack identity ✅ (704.5d/e — the token-purge SBA sweeps
  copies from every non-stack zone; test
  `cr_704_5e_countered_spell_copy_ceases_to_exist`); Role uniqueness ✅
  (704.5y). Illegally-attached Aura ✅ (704.5n / 303.4f — an Aura whose live
  host fails its printed `aura_enchant_filter`, e.g. a "you control" Aura on a
  stolen creature, goes to the owner's graveyard; tests `cr_704_5n_*`).
  Zero-toughness → graveyard ✅ (704.5g, test
  `cr_704_5g_zero_toughness_creature_dies`). Battle-with-no-defense-counters
  defeat ✅ (704.5x via `defeat_battle`, `tests/mom.rs`). Speed SBA ✅ (704.5z —
  `check_state_based_actions` seeds speed 1 for engines controllers; test
  `cr_704_5z_engines_seed_speed_sba`). Multi-SBA "collapse into one
  replacement" ✅ (704.7 — `StaticEffect::ReplaceControllerLossWithReset` +
  `GameState::apply_loss_reset`; Lich's Mirror replaces a life *and* poison
  loss once, and covers the draw-from-empty loss too; `cr_recent42::cr_704_7_*`).
  Dungeon removal ✅ (CR 309.6 — room abilities use the stack and the
  finished dungeon leaves the game as the last one resolves;
  `cr_recent84::cr_309_6_*`).
- 🟡 **CR 613 — Interaction of Continuous Effects** — 613.7 timestamps ✅ (object timestamps stamped on entry/attach/face-up/transform from the shared effect counter; statics order by `object_timestamp()`; tests `cr_613_7_*`). Remaining: no dependency analyzer (613.8); CDA-first pre-pass (613.3). (EOT keyword grants now join the walk timestamped — audit P1 row closed. Static keyword-grant scopes now route a `ToughnessGreaterThanPower` leaf through the `CardMatch` dynamic path — read against printed P/T + counters per the `CardMatchPowerGated` approximation — so Tapestry Warden / Ancient Lumberknot grant their keyword only to your T>P creatures.) Layer-4 additive card-type static ✅ (`StaticEffect::AddCardTypeToMatching` — "nontoken artifacts you control are lands in addition to their other types", Toph, the First Metalbender; `toph_metalbender_artifacts_are_lands_and_end_step_earthbend`). **CR 613.2 computed-subtype consistency ✅** — `HasArtifactSubtype`/`HasLandType`/`HasSupertype` requirements now read a battlefield permanent's *computed* (post-layer) subtypes/supertypes, matching card-type and creature-type checks, so continuous subtype grants (Sugar Coat's Food, Vraska's Treasure, Song of the Dryads' Forest, the Ring-bearer's Legendary) are seen by aura-legality SBAs and filters (`blb::sugar_coat_makes_a_food`; fixed the Alpine Moon test that had leaned on the printed-subtype read). `EquipBonus.set_artifact_types` installs the layer-4 artifact-subtype override.
- 🟡 **CR 208 — Power/Toughness** — base-P/T-only checks (208.4b). 208.3 noncreature P/T now observable for `*`-power Vehicles: `DynamicPt::LandsControlledPower` sets power off a count while toughness stays printed, `computed_permanent()` reports it on a non-crewed (noncreature) Vehicle (Lumbering Worldwagon `*`/4; test `lumbering_worldwagon_power_tracks_lands`). Conditional base-P/T set ✅ (`StaticEffect::SetBasePtIf` — live layer-7b SetPowerToughness gated on a predicate; counters/+N stack on top per 613.7c/f — Snowmelt Stag "5/2 during your turn"; `snowmelt_stag_*`). CR 604.3 CDAs: `DynamicPt::LandsControlledPlusLandsInControllerGraveyard` (Multani, Yavimaya's Avatar), `DynamicPt::CardTypesInOpponentsGraveyards` (Nighthawk Scavenger), `DynamicPt::InstantsSorceriesInControllerGraveyard` (Enigma Drake), `DynamicPt::CreaturesControlledPower` (Suki `*`/4), `DynamicPt::PlusCountersOnLandsControlledPower` (Toph `*`/3), `DynamicPt::NoncreatureNonlandCardsInControllerGraveyard` (Dragonfly Swarm `*`/3), `DynamicPt::ColorsAmongAlliesControlledPower` (Earthen Ally `*`/2), `DynamicPt::EnchantmentsInPlay` (Yavimaya Enchantress `2/2`, +1/+1 per enchantment in play — `tests/recent72.rs`), `DynamicPt::ForestsInPlay` (Traproot Kami `0/*`, toughness = Forests on the battlefield — `tests/recent100.rs`), all live-recomputed by `computed_permanent()`; `tests/recent47.rs`, `tests/recent50.rs`, `tests/tla.rs`.
- 🟡 **CR 119 — Life** — 119.7 set-to-lowest ✅ (`Value::LowestLifeTotal` + Repay in Kind); exchange-life-totals ✅ (Soul Conduit, Mirror Universe, Magus of the Mirror); life-gain→loss replacement ✅ (`StaticEffect::LifeGainBecomesLoss`, Tainted Remedy); life-gain **bonus** replacement ✅ (119.10 — `StaticEffect::LifeGainBonus { target, amount }` folded into `adjust_life` via `life_gain_bonus_now`; Honor Troll's "gain that much plus 1"). 119.7 rest-of-game lifegain lock ✅ (`Effect::LifeGainLockGame` sets the permanent `Player.cannot_gain_life` flag, distinct from the turn-scoped lock — Screaming Nemesis via `Selector::Target(0)`; test `screaming_nemesis_redirects_damage`). Life-total-threshold statics ✅ (`Predicate::PlayerLifeAtLeast` gates a live self-anthem — Angel of Vitality's +2/+2 at 25+ life; `cr_119_*`, `tests/recent17.rs`). Life-vs-*starting*-total statics ✅ (`Predicate::PlayerLifeAtLeastAboveStarting` gates tiered self-pumps — Elenda, Saint of Dusk +1/+1/menace above starting, +5/+5 more at 10+ above; `elenda_scales_with_life`). Exact-life gate ✅ (`Predicate::PlayerLifeExactly` — Hidetsugu's Second Rite deals 10 only if the targeted player is at exactly 10; `hidetsugus_second_rite_needs_exactly_ten`). Redistribute-life-totals (119.7) is exact at two players — Reverse the Sands rides `ExchangeLifeTotals`, `reverse_the_sands_swaps_life_totals`; a true multiplayer redistribution (each player picks which total they get back) is still open. Remaining: per-source life-gain replacement breadth. (Audit follow-up closed: every `LifeGained` emitter now uses `adjust_life_applied`, and `SetLifeTotal`/`ExchangeLifeTotals` route through the funnel — so a can't-gain-life lock on the player who would gain blocks their half of an exchange while the other still loses; test `cr_119_7_exchange_life_totals_respects_cant_gain_life`.)
- ✅ **CR 504.1 — draw-step triggers** — `advance_step` never called
  `fire_step_triggers(TurnStep::Draw)`, so every "at the beginning of your
  draw step" ability was inert. Now fires after the turn-based draw
  (Armageddon Clock; `classic_sets/atq::armageddon_clock_ticks_up_and_burns_everyone`).
- 🟡 **CR 121 — Drawing a Card** — one-shot draw replacement ✅
  (`Effect::ReplaceYourNextDrawThisTurn` queues a charge on
  `Player.next_draw_replacements`; `draw_one` spends the front charge and
  resolves its body — auto-targeting it when it needs one — and unused charges
  clear at the turn boundary. The Onslaught Words cycle;
  `classic_sets/ons::words_cycle_replaces_the_next_draw`). Draw-count
  replacement (121.2a) ✅ via `StaticEffect::ControllerDrawsDoubled` in `draw_one` (Thought Reflection; stacks per 614.5, reentrancy-guarded); **condition-gated** draw doubling ✅ (`ControllerDrawsDoubledIf` — Vnwxt's max-speed draw-two; test `cr_121_2a_conditional_draw_replacement`). Draw-count board gates ✅ via `SelectionRequirement::ControllerDrewAtLeastThisTurn(n)` (reads `Player.cards_drawn_this_turn`), wired as a `SelfHasKeywordWhile` condition (Foggy Swamp Hunters lifelink/menace, June unblockable). Choose-to-draw (121.3 / 121.2b) ✅ — `GameState::may_choose_to_draw` stops `Effect::MayDo` / `Effect::MayPay` offering an optional draw to a capped player (a rules-declined `MayPay` still runs its `else_`), and the per-turn cap now gates `draw_one` itself so *every* draw source is capped, not just `Effect::Draw`'s count; an empty library deliberately doesn't block the choice. Chains of Mephistopheles ships as a global replacement in `draw_one` with a CR 614.5 reentrancy guard (`cr_recent74::cr_121_2a_chains_replaces_each_extra_draw_once`). Remaining: mid-cast face-down draw (121.8); reveal-on-draw (121.9).
- ✅ **CR 506.4 — Removed from combat.** A Gustcloak's "untap it and remove it
  from combat" pulls the attacker out before the damage step, so no combat
  damage is exchanged (`Effect::RemoveFromCombat` off `EventKind::BecomesBlocked`;
  `classic_sets/ons::gustcloak_sentinel_slips_its_blocker`, `gustcloak_savior_*`).
- ✅ **CR 205.3m — restricted creature-type choice.** `Effect::
  BecomeChosenCreatureType.excluded` (and `Decision::ChooseCreatureType.excluded`,
  surfaced on the wire) carry the printed "a creature type other than Wall"
  restriction: the excluded types are never offered and a decider that names one
  is overruled. One choice now covers the whole effect rather than one prompt per
  affected permanent (Imagecrafter, Mistform Mutant, Standardize;
  `cr_205_3m_imagecrafter_cant_name_wall`).
- ✅ **CR 613.8 — type-gated grants see a retype.** `GameState::
  shallow_creature_types` reads stored layer-4 `SetCreatureTypes`/
  `AddCreatureType` effects without a full layer pass, so a requirement walk
  running *inside* the layer gather (where the computed view is off-limits)
  still sees a retyped permanent — Mistform Wall keeps defender only while it is
  a Wall (`cr_613_8_type_gated_grant_sees_a_retype`). `SelectionRequirement::
  FaceDown` also joined the card-only set so face-down-matters anthems apply
  (Ixidor, Reality Sculptor).
- ✅ **CR 705.1 — Flipping a coin.** `Effect::FlipUntilLoss` runs its payoff
  once per won flip and stops on the first loss (Crazed Firecat); test
  `cr_recent64::cr_705_1_flip_until_loss_pays_per_win`.
- ✅ **CR 702.15 — Landwalk.** The evasion reads the *defending* player's
  lands, not the attacker's; test
  `cr_recent64::cr_702_15b_landwalk_keys_on_the_defender`.
- ✅ **CR 702.34a — Flashback additional costs.** Declared per card on
  `CardDefinition::flashback_additional_cost` (sacrifice / discard /
  `DiscardXFromCost` / pay-life / exile-from-graveyard) and paid on top of the
  flashback mana cost; test
  `cr_recent64::cr_702_34a_flashback_additional_cost_is_paid`.
- ✅ **CR 605.1b / 605.4a — triggered mana abilities** — a targetless
  mana-adding trigger fired from a mana ability resolves off-stack, so its mana
  reaches the pool before the payment in progress finishes (Overabundance).
  `TriggerCandidate`/`PendingTriggerPush` carry `from_mana_ability`; tests
  `cr_recent63::cr_605_{1b,4a,5a}_*`. Remaining ⏳: 605.3c ("can't be activated
  again until it has resolved") isn't modelled.
- 🟡 **CR 502 — Untap Step** — untap caps are now filtered (`StaticEffect::MaxOneUntapPerStep { filter }` — Winter Moon's nonbasic lands and Imi Statue's artifacts share one path; `imi_statue_caps_artifact_untaps_at_one`). CR 502.3 "doesn't untap while it has a [kind] counter" now reads the **computed** keywords at both untap gates, so a *granted* lock counts (Temporal Distortion's hourglass counters), not just a printed one — `cr_recent63::cr_502_3_counter_gated_permanent_doesnt_untap`. Phasing (502.1 / 702.26) ✅: `do_phasing`
  runs as a turn-based action at the top of the untap step, moving the active
  player's phasing permanents (and their attachments) to `GameState.phased_out`
  and phasing back in everything they control there — modelled as a side zone
  so every battlefield query ignores phased-out cards and no ETB/LTB fires, all
  state retained (Tolarian Drake). Targeted phase-out ✅ via `Effect::PhaseOut`
  (Vodalian Illusionist). Daybound/Nightbound DFC transform (502.2) ✅ — see
  CR 712 below.
  `StaticEffect::PreventUntap` honors `Selector::This` (Basalt/Grim Monolith)
  and `Selector::AttachedTo(This)` (Claustrophobia/Dehydration). Per-player
  one-step land-untap lock ✅ (502.3 — `Effect::LandsDontUntapNextUntapStep` +
  `Player.lands_dont_untap_next_untap`, consumed in `do_untap`; Bontu's Last
  Reckoning, `cr_502_3_bontus_lands_skip_one_untap_step`). Self-scoped
  untap-on-every-step ✅ (502.3 — `StaticEffect::UntapSelfEachUntapStep`, a
  `do_untap` follow-up pass untaps the source on each *other* player's untap
  step too, Stun counters still interpose; Thousand Moons Infantry,
  `thousand_moons_infantry_untaps_on_opponent_untap`).
- ✅ **CR 510 — Combat Damage Step** — blocker-side damage division ✅ (510.1e / 509.2 — a creature blocking several attackers orders them and divides its power; the defending player decides and a `wants_ui` seat suspends, mirroring the attacker-side order/assign pair). remains-blocked ✅ (`blocked_attackers`, 510.1c); excess non-trample damage assigned to the last blocker ✅ (510.1d); lethal accounts for marked damage ✅ (510.1c, double-strike tramplers); blocker strike-back per-source ✅ (702.90 / 615.6 — infect/deathtouch/scaling/shields/lifelink apply per blocker event, tests `cr_702_90_*`). **Assigns combat damage equal to toughness** ✅ (510.1c — `Keyword::AssignsCombatDamageByToughness`, read by `combat_damage_value` for attackers, blockers, and the cached-assignment path; Doran, Tapestry Warden, Bill the Pony, `tests/recent23.rs`). **"Whenever combat damage is dealt to you"** ✅ (`EventKind::ControllerDealtCombatDamage` — recipient-keyed `SelfSource` listeners fire off the damaged player's own permanents, carrying the amount as `event_amount`; Risona sheds an indestructible counter — `tests/recent100.rs`).
- ✅ **CR 702.158 — Space Sculptor.** `Keyword::SpaceSculptor`,
  `CardInstance.sector`, the CR 704.5u assignment SBA (opponents assign first;
  designations clear with the last sculptor per 702.158b),
  `Effect::ChooseSector` + `Selector::CreaturesInChosenSector` (702.158d), and
  the same-sector block lock. Space Beleren ships; tests
  `core_rules/cr_recent36`. Residual: the assignment and the sector pick are
  auto-decided rather than prompted.
- ✅ **CR 511 — End of Combat Step.** 511.2 "at end of combat" triggers now
  fire — `fire_step_triggers` was never called for `TurnStep::EndCombat`, so
  every `DelayedKind::EndOfCombat` registration silently expired. Test
  `cr_511_2_end_of_combat_delayed_trigger_fires`.
- ✅ **CR 506.4 — Removal from Combat.** 506.4c verified: an attacker whose
  planeswalker leaves combat stays an attacking creature and deals no combat
  damage (`cr_506_4c_attacker_survives_its_planeswalker_leaving`).
- ✅ CR 702.59 — Recover. The graveyard trigger now filters on the dying
  creature's *owner* (CR 702.59a "put into **your** graveyard"), and
  `recover_paying_half_life` covers the life-cost variant. All 7 printed
  Recover cards ship.
- 🟡 **CR 509 — Declare Blockers** — cost-to-block (509.1d-f). **509.3a–e ✅**: "whenever this blocks" / "becomes blocked" fire ONCE per creature (the `BlockerDeclared` fan-out dedupes on the trigger's own side of the pair), `Selector::BlockedAttacker` resolves every attacker a multi-blocker is blocking so the per-object wordings (509.3b/d) reach all of them from one instance, and `EventKind::{BlocksNOrMore,BecomesBlockedByNOrMore}` gate on the finished block assignment (509.3e — Lairwatch Giant). Tests `core_rules/cr_recent35::cr_509_3*`. **Multi-block ✅** (509.1b — `block_map` is blocker → `Vec<attacker>`; `Keyword::CanBlockAdditional(n)` / `CanBlockAnyNumber` set the per-combat cap; Guardian of the Gateless, Knight of Sorrows, Valor Made Real; tests `core_rules/cr_recent35`). Put-onto-battlefield-blocking (509.4) ✅ — `Effect::CreateTokenBlocking` + the `cast_only_after_blockers` gate (Flash Foliage; test `cr_509_4_flash_foliage_blocks_the_attacker`). Blocker legality now reads the computed view ✅ (509.1a — animated manlands / crewed Vehicles block). ("Can't be blocked except by N or more creatures" ✅ via `Keyword::CantBeBlockedExceptByN` — Pathrazer of Ulamog, generalizing Menace.) Per-pair block restriction (509.1b — "target creature can't block this creature this turn") ✅ via `Effect::CantBlockSourceThisTurn` + `GameState.cant_block_pairs` (Kozilek's Pathfinder); "must be blocked if able" (509.1c) ✅ via `Keyword::MustBeBlocked` (Loathsome Catoblepas). Power-based block restriction ✅ (`Keyword::CantBeBlockedByPowerLess` — Formation Breaker; inverse of Skulk, `formation_breaker_blocks_only_by_equal_or_greater_power`). The bot's block planner now satisfies the minimum-blocker count for Menace **and** `CantBeBlockedExceptByN(n)` (tops up or drops the block), so it never submits an illegal under-filled multi-block. Protection-by-mana-value block restriction ✅ (`Keyword::ProtectionFromManaValueExcept` — Haktos can't be blocked by a creature whose MV isn't the chosen number; test `cr_509_1b_protection_from_mv_restricts_blockers`). Protection-by-mana-value-**parity** ✅ (`Keyword::ProtectionFromManaValueParity { odd }` — Lavabrink Venturer's ETB odd/even choice; gates targeting, blocking, and combat-damage prevention CR 702.16e; tests `lavabrink_venturer_parity_protection`, `cr_702_16e_parity_protection_prevents_combat_damage`). Blocker-side "can block only creatures with flying" ✅ (`Keyword::CanBlockOnlyFlying` — Wanderlight Spirit, Shacklegeist, Pinnacle Emissary's Drone; test `cr_509_1b_can_block_only_flying_restriction`). Conditional attack/block gates (509.1a / 508.1a) ✅ — `Keyword::CantAttackOrBlockUnlessHandSizeAtMost(n)` (Hazoret the Fervent), `Keyword::CantAttackOrBlockUnlessDelirium` (Patchwork Beastie, via `GameState::delirium_active`), and `Keyword::CantAttackOrBlockUnlessDescend(n)` (The Ancient One, via `GameState::descend_count`), enforced in `declare_attackers` + `blocker_can_block_attacker` + `legal_attackers`/affordances and surfaced as client chips. "Can't attack or block alone" (509.1c) ✅ — `Keyword::CantAttackOrBlockAlone` rejects a lone-attacker / lone-blocker batch (Toby's Beast token; tests `cant_attack_or_block_alone_*`, `cant_block_alone_*`).
- 🟡 **CR 118 — Costs** — interactive mana-ability decline (118.3c); hybrid-pip per-reduction choice (118.7e); general unpayable-cost gate (118.6). Board-conditional self cost reduction ✅ (CR 601.2f — `StaticEffect::SelfCostReducedIfControlEach`, discounts a spell while you control a permanent matching each filter — Of One Mind's Human + non-Human). Opponent target-tax ✅ (`StaticEffect::TaxOpponentSpellsTargeting`, threaded through `extra_cost_for_spell` with the spell's chosen target — Jubilant Skybonder, Callaphe Beloved of the Sea). Mana-spent-vs-MV gate ✅ (`Effect::CounterSpellDrawIfUnderpaid` reads the countered spell's stored `mana_spent` against its mana value — Unravel draws only on a cost-reduced/alt-cast spell). Total-power self-reduction ✅ (`StaticEffect::SelfCostReducedByTotalPower` — Ghalta, Primal Hunger; `ghalta_costs_less_per_total_power`). Per-graveyard-creature self-reduction ✅ (`StaticEffect::SelfCostReducedPerCreatureInGraveyard` — Ghoultree; `ghoultree_costs_less_per_graveyard_creature`). Death-gated self-reduction ✅ (`StaticEffect::SelfCostReducedIfCreatureDiedThisTurn` — Bone Picker; `bone_picker_is_cheap_after_a_death`). Player-wide predicate-gated reduction ✅ (`StaticEffect::CostReductionWhile { filter, amount, condition }` — Gran-Gran's "noncreature spells you cast cost {1} less while 3+ Lessons in your gy"; generic-only clamp tested in `cr_601_2f_gran_gran_lesson_discount_is_generic_only`). Source-power-scaled reduction ✅ (`StaticEffect::CostReductionBySourcePower` — "Aura and Equipment spells cost {X} less, X = this creature's power" — Golden-Tail Trainer). Board-count "affinity for [type]" reduction (`SelfCostReducedPerPermanentMatching`) now evaluates board-state filters (`IsModified`, tapped, …) through `evaluate_requirement_static`, so Walking Skyscraper's "costs {1} less per modified creature" works; `tests/recent100.rs`. **CR 107.16 variable {E} cost ✅** — `ActivatedAbility.energy_x_cost` spends the activation's chosen `x_value` in energy and threads that X into resolution so `ManaValueExactlyXFromCost` gates the target (Chthonian Nightmare; `cr_107_16_variable_energy_cost_pays_chosen_x`). Value-amount energy pay/upkeep ✅ (`Effect::PayEnergyValue`, `Effect::PayEnergyOrElseValue` — Jolted Awake, Volatile Stormdrake). **CR 107.16 variable life cost ✅** — `ActivatedAbility.x_life_cost` drains the chosen X in life and threads that X into resolution (Krumar Initiate's "Pay X life: endure X"; `cr_107_16_pay_x_life_variable_activation_cost`). Card-level "costs {N} less if you've cast another spell this turn" ✅ (`self_cost_reduction_if_cast_spell` — Rally the Monastery).
- 🟡 **CR 113 — Abilities** — emblems+CDA zones (113.6); full ability removal (113.10b); "can't have" anti-grant (113.11). Counter-target-ability (113.9) ✅ — `Effect::CounterAbility` (Consign to Memory, Stifle) with precise targeting via `SelectionRequirement::HasAbilityOnStack`.
- 🟡 **CR 115 — Targets** — Aura subtype (115.1b); zero-target cast-time gate (115.6 — **blocked**: many targeted spells are cast with `target: None` and auto-target at resolution (counterspells → top of stack; "target player" discard → an opponent), so a naive "requires_target ⇒ reject None" gate breaks Pact of Negation / Pyroblast / Cabal Therapy / Metallurgic Summonings. A real fix must make cast-time supply the target for every targeted spell first); change-target corners (115.7a-d, cross-spell exchange). Same-target rejection *within one multi-target instance* (115.3) ✅ — `Effect::distinct_target_count` + a cast-time duplicate check reject the same object filling two divide/support slots (Forked Bolt); cross-clause sharing stays legal. "Up to N target" triggers now fill every slot ✅ (115.1c) on both the **Attacks** path (combat.rs — Lagorin's "up to two Mounts/Vehicles"; `cr_115_1c_attack_trigger_fills_all_target_slots`) and the **ETB** path (stack.rs's `auto_extra_targets_for` — Azorius Justiciar detains two; `cr_115_1c_etb_trigger_fills_all_target_slots`). "Counter target spell that targets you or a permanent you control" ✅ via `SelectionRequirement::SpellTargetsControllerOrControlled`, which reads a stack spell's chosen targets per CR 115.9b (Hindering Light; `cr_115_9b_target_filter_reads_the_current_targets`). **CR 601.2c "must be chosen as a target" ✅** — `StaticEffect::FlagbearersMustBeTargeted` + `flagbearer_violation` gate both the cast and activation paths, the auto-targeter prefers a Flagbearer, and `PermanentView.is_flagbearer` explains the rejection client-side (Standard Bearer, Coalition Honor Guard, Coalition Flag; `cr_recent60::cr_601_2c_*`).
- 🟡 **CR 116 — Special Actions** — Companion ✅ (116.2g / 702.139 —
  `GameAction::CompanionToHand`, {3} sorcery-speed sideboard→hand; deck-build
  restriction ✅ via `CardDefinition.companion` + `format::companion_restriction_met`,
  enforced by the server deck loader). (Foretell/Plot/Suspend ✅; manifest turn-face-up `GameAction::TurnFaceUp` ✅ — CR 708.5. Morph cast-face-down spell path still ⏳.)
- 🟡 **CR 105 — Colors** — type-line + color rewrite rider (105.3 second half).
  Color-count value (105.2 — `Value::ColorCountOf`, "for each of its colors";
  colorless/devoid counts 0 per 105.2c) ✅ — Breathe Your Last; tests
  `cr_105_2c_colorless_counts_zero_colors`, `breathe_your_last_gains_life_per_color`.
- ✅ **CR 705 — Flipping a Coin** — Mana Clash two-player flip-off loop (705.2), 705.3 advantage/Krark's Thumb, win-a-flip trigger (`EventKind::WonCoinFlip`/`GameEvent::CoinFlipWon`, Chance Encounter) and lose-a-flip trigger (`EventKind::LostCoinFlip`/`GameEvent::CoinFlipLost`, emitted on the tails path of FlipCoin + ManaClash). Sequential "flip until you lose or stop" ✅ via `Effect::FlipCoinsUntilLoseOrStop { tiers }` (a lost flip cancels everything; win-count tiers fire in order — Fiery Gambit). Per-flip `RemoveFromCombat`/`PhaseOut` payoffs ship Mijae Djinn, Ydwen Efreet, Frenetic Efreet; copy-or-bounce-your-spell on flip ships Krark, the Thumbless. Remaining ⏳: opponent-chooses-half flips (Karplusan Minotaur). (AutoDecider now flips a real random coin; scripted tests stay deterministic.)
- ✅ **CR 309 / 701.49 — Dungeons & Venture** — `base::dungeons` (all three
  AFR dungeons), `Effect::Venture` (enter/advance with `ChooseMode` branch
  picks; room abilities resolve inline), `Player.{dungeon,dungeons_completed}`,
  `EventKind::DungeonCompleted` (battlefield + graveyard dispatch — Dungeon
  Crawler), `Value::DungeonsCompleted` (Cloister Gargoyle). Tests `tests/afr.rs`.
  Remaining ⏳: room abilities don't use the stack; Tomb's two pay-or-lose
  rooms are flat life loss; Mad Wizard's Lair free-cast collapsed to the draws.
- 🟡 **CR 122 — Counters** — defense counters / Battle type (122.1g) ✅ (`CounterType::Defense`, CR 310). Counter-clear on zone change (122.2) ✅ strict — cleared at every zone-change funnel; dies-with-counters triggers read the `died_card_snapshots` / `leaves_bf_lki` LKI caches (Felisa, Ambitious Augmenter). `-0/-1` / `-1/-0` counter types ✅. Counter-removal as an activation gate ✅ — `CounterType::Fuse` + an `ActivatedAbility.condition` on `Value::CountersOn` ≥ N (Goblin Bomb's "remove five fuse counters: deal 20"). "Choose a kind of counter at random it doesn't have" ✅ via `Effect::AddRandomMissingCounter` (keyword counters + +1/+1, never duplicating a present kind; respects Solemnity — Crystalline Giant). Return-a-died-creature-with-a-keyword-counter ✅ — a `CreatureDied`/`AnotherOfYours` trigger `Move`s `Selector::TriggerSource` (its gy card) back to the battlefield, then `AddKeywordCounter` on `Selector::LastMoved` (Luminous Broodmoth's flying counter; `luminous_broodmoth_returns_with_flying`). CR 614.16 additive replacement for *every* counter kind ✅ — `StaticEffect::ExtraCounterAllKinds` (Winding Constrictor) adds one to any counter placed on your creatures, via `GameState::scaled_counter_count`; composes with Hardened Scales (+1/+1-only) and Doubling Season. The player-counter "counters you'd get" half now covers energy **and** experience (`AddExperience` honors `extra_any_kind_adders_for`; `cr_614_16_winding_constrictor_boosts_experience`). Poison now scales too ✅ — `GameState::scaled_player_counter_count` (adder + doublers) routes every poison site (AddPoison, AddCounter(Player), Infect/Toxic combat); `cr_614_16_winding_constrictor_boosts_poison`. Keyword counters granting the keyword via layers ✅; test `cr_122_1_keyword_counter_grants_keyword` (Gift of the Viper). "Enters with N counters" ✅ (`CardDefinition.enters_with_counters` — Argent Dais's two oil; `cr_122_1_permanent_enters_with_printed_counters`). CR 122.5 relocation now moves **keyword counters** too — `Effect::MoveAllCounters` drains the separate `keyword_counters` map alongside `counters` (Reluctant Role Model; `cr_122_5_move_all_counters_relocates_keyword_counters`). CR 122.6 "remove up to N counters" ✅ — `Effect::RemoveCountersUpTo { what, amount }` drains any kinds from a permanent (greedy) or poison from a player (Price of Betrayal; `price_of_betrayal_strips_permanent_counters`, `_strips_player_poison`).
- 🟡 **CR 401 — Library** — play-with-top-revealed + play/cast-from-top ✅
  (401.5/401.6 — `StaticEffect::{TopOfLibraryRevealed,PlayFromLibraryTop}` plus
  the turn-scoped `Player.play_from_top_this_turn` grant
  (`Effect::GrantPlayFromTopThisTurn` — The Belligerent), both honored by
  `library_top_playable` + `known_library_top`/HUD chip; Courser, Oracle of Mul
  Daya, Mystic Forge). Remaining: the mid-cast "new top stays hidden until
  the spell finishes" timing nuance (401.5 second sentence); multi-card
  same-position picker (401.4). (401.7 `LibraryPosition::FromTop` ✅.)
- 🟡 **CR 706 — Rolling a Die** — ignore-roll riders (the roll-extra-and-
  ignore-lowest replacement now also covers `Effect::RollAndStoreDice`, CR
  706.2 — `cr_recent82::cr_706_2_*`). Stored rolls (706.8) ✅
  (`CardInstance.stored_die_results`, `Effect::{RollAndStoreDice,
  RerollStoredResults}`, `Value::GreatestSameStoredResult` — Centaur of
  Attention; `cr_706_8_*`). Roll trigger (706.6) ✅ — `EventKind::RolledDice`/`GameEvent::DiceRolled { player, count, high }` fires once per roll instruction ("whenever you roll one or more dice"). Result-referencing effects ✅ via `Value::LastDieRoll` (706.4 — Ancient Copper Dragon). **Result-gated triggers ✅** — `Predicate::DieResultAtLeast(n)` filters a roll trigger on the roll's greatest result (Ground Pounder's "roll a 5+ → trample"), reading `DiceRolled.high` through `event_amount`. (modifier / reroll-at-most / doubles ✅.) Remaining ⏳: ignore/reroll-replacement riders; the CR 706.8b reroll is auto-chosen (keep the most common face, reroll the rest) rather than prompting per result.
- 🟡 **CR 707 — Copying Objects** — in-place copy (707.4); MDFC-face copy (707.8); static copy effects (707.2c); copied "as enters" choices (707.6); spell-copy exceptions (707.9). (Enter-as-copy "except it's also [type]" ✅ via `EntersAsCopy.extra_card_types` — Phyrexian Metamorph copies any artifact/creature and stays an artifact. Token-copies with haste + delayed sacrifice ✅ via `Effect::CreateTokenCopiesHasteSac` — Devastating Onslaught's X copies, CR 707.2 + 111 + 701.16. `CreateTokenCopyOf` now takes `override_colors` (exact-color copy — Ardyn's 5/5 black Demon) and `enters_tapped` (Sin's tapped copies; `cr_707_2_token_copy_enters_tapped`).)
- 🟡 **CR 205 / 613.4 — Adding subtypes** — `Effect::AddCreatureTypes` grants
  creature types *in addition* to a permanent's own via a layer-4 additive
  `AddCreatureType` (Jenova, Ancient Calamity's "becomes a Mutant in addition to
  its other types"; `jenova_buffs_and_grants_mutant`), complementing
  `BecomeCreatureType`'s full set. Remaining: adding card types/supertypes via
  the same one-shot shape.
- 🟡 **CR 506 — Combat Phase** — remove-from-combat ✅ (506.4 — `Effect::RemoveFromCombat` pulls a targeted attacker/blocker out of combat, releasing its blockers; Labyrinth of Skophos, test `cr_506_4_*`). **Skip-combat ✅** (`Effect::SkipNextCombatPhase` + `Player.skip_next_combat`; `advance_step` jumps Begin Combat → postcombat main when the active player has a charge — Stonehorn Dignitary; tests `cr_506_active_player_skips_their_combat_phase`, `cr_506_skip_only_eats_one_combat`). Surfaced in `PlayerView.skip_next_combat` + a "⚔ skip" client chip. "block as though" restrictions (506.6); combat-step cast-timing gates (506.7). `PlayerRef::DefendingPlayer` now resolves off the *triggering attacker* for `YourControl`-scoped Attacks triggers (not just the ability source), so "whenever a creature you control attacks, defending player loses N" fires correctly (Leeching Sliver, CR 509.2). Combat-damage-to-player triggers now carry the damage dealt as `event_amount` (CR 119.3), so `Value::TriggerEventAmount` riders scale by the hit (Visions of Brutality). Such triggers now also **auto-target a graveyard card** when their effect prefers one (`prefers_graveyard_target`) instead of always binding slot 0 to the damaged player — Efreet Flamepainter recasts an instant, Venerable Warsinger reanimates a creature. (`CopySpell` / `CastWithoutPayingImmediate` are now surfaced by `primary_target_filter`, so on-cast self-copy and gy-recast triggers auto-target correctly; `CastWithoutPayingImmediate` accepts a `Permanent` entity-ref for the targeted gy card.)
- ✅ **CR 508.1g — Attack tax** — `StaticEffect::AttackTaxToController { amount }`
  taxes attackers hitting the source's controller (Ghostly Prison, Propaganda).
  `declare_attackers` sums the tax across the batch and pays it from the
  attacker's pool, auto-tapping mana sources for any shortfall (atomic
  rollback if unpayable); block tax (509.1d) pays the same way per blocking
  player. Tests `cr_508_1g_*`, `cr_509_1d_block_tax_auto_taps_lands`.
- ✅ **CR 508.1a — Attack despite Defender (turn grant)** — `Effect::AttackDespiteDefenderThisTurn` + `GameState.attack_despite_defender_this_turn` (cleared at cleanup); `legal_attackers` / `declare_attackers` / `ignores_defender_for_attack` honor it. Krotiq Nestguard's activated ability (`krotiq_nestguard_can_attack_after_ability`). Static while-condition variant already shipped (`CanAttackIgnoringDefenderWhile` — Drowsing Tyrannodon).
- ✅ **CR 508 — "Whenever you attack"** — `EventKind::YouAttack` fires once per
  combat for the attacking player (not per-attacker), dispatched from
  `declare_attackers`; `shortcut::on_you_attack`. Replaced the
  `Attacks/YourControl + once_per_turn` approximation on Razorkin, Inti, Gut,
  Raffine, Most Valuable Slayer, Lionheart Glimmer.
- 🟡 **CR 508.1a — Attack restrictions** — the keyword gate list now covers "can't attack if it attacked during your last turn" (`Keyword::CantAttackIfAttackedLastTurn`, off the `attacked_own_turn` → `attacked_last_turn` untap roll-over) and a one-turn ban armed by an effect (`Effect::CantAttackNextTurn` + `CardInstance.attack_ban`, promoted at the bearer's untap and cleared the turn after). Both surface as `PermanentView.cant_attack_this_turn`. Tests `cr_508_1a_attacked_last_turn_restriction_lifts_after_one_turn`, `wall_of_dust_benches_what_it_blocks`. Remaining: the restriction list is a hand-written match rather than a general predicate.
- 🟡 **CR 508.3a — Put onto the battlefield attacking** — `Effect::CreateTokenAttacking`
  (tokens) and `Effect::JoinCombatAttacking { what }` (existing permanents — a
  reanimated/blinked creature joins combat tapped + attacking; Alesha, Who
  Smiles at Death reanimates via `Move→Battlefield` + `JoinCombatAttacking`).
  Remaining: choose the attacked defender/planeswalker (currently follows the
  source's attack, else the first opponent).
  (token attackers — Mobilize/Myriad) and `Effect::LookTopMayDeployAttacking`
  (deploy a real library card tapped-and-attacking with indestructible EOT,
  bottom the rest in random order per 401.4 — Winota) both join the current
  combat by pushing onto `attacking` past the declare-attackers gate. Remaining:
  a controller's-choice defender pick (currently follows the triggering creature).
- ✅ **CR 605 — Mana Abilities** — triggered mana abilities (605.1b/605.4a)
  resolve stack-free at the mana-ability fast path via
  `StaticEffect::ExtraManaOnLandTap` (Mana Flare, Vernal Bloom, Wild
  Growth, Utopia Sprawl; tests `cr_605_1b_*`). Board-state-**conditional**
  mana abilities (`Effect::If` with mana-ability branches) are now recognized
  as mana abilities by `is_mana_ability`/`effect_produces_color`, so Ilysian
  Caryatid taps for mana without using the stack (test
  `cr_605_conditional_mana_ability_pays_a_spell`).
- ✅ **CR 606 — Loyalty Abilities** — sorcery-speed, once-per-turn-per-walker gating ✅; loyalty-set effects ✅ (`Effect::SetLoyalty`); variable `-X` loyalty ✅ (606.5 — `LoyaltyAbility.x_cost`, `ActivateLoyaltyAbility { x_value }`, body reads `Value::XFromCost`; Kasmina); opponent loyalty-activation tax ✅ (`StaticEffect::OpponentLoyaltyActivationTax`, paid as extra generic mana — Eidolon of Obstruction, test `cr_606_eidolon_*`). Instant-speed-the-turn-it-entered ✅ (606.3b — `CardDefinition.flash_loyalty` + `entered_turn` gate skips the sorcery-speed check while it's the entry turn; The Wandering Emperor, test `wandering_emperor_flash_loyalty_window`). Remaining ⏳: unconditional "activate any time" riders; a UI `Decision::ChooseAmount` X prompt.
- 🟡 **CR 701.45 — Learn** — reveal-Lesson / discard-to-draw decision ✅; the in-graveyard "if you would learn, you may instead return this" replacement ✅ via `StaticEffect::MayReturnFromGraveyardInsteadOfLearn` consulted at the top of `Effect::Learn` (Retriever Phoenix). Remaining ⏳: Lesson sideboard population in some deck-build paths.
- ✅ **CR 701.10 — Double** — mana-doubling (701.10f) ✅ via `StaticEffect::ManaProductionDoubled` + `GameState.mana_production_doublers` (stamped around mana-ability resolution; `AddMana` multiplies pip output by `2^doublers`; rituals/spell-mana unaffected). Mana Reflection carded + tested. P/T-, counter-, life-doubling already ✅.
- ✅ **CR 701.12 — Exchange (control)** — `Effect::ExchangeControl { a, b }` swaps the controllers of two resolved permanents simultaneously (Switcheroo). Exchange-life-totals + exchange-hand/graveyard already ✅. Vedalken Plotter ✅ via `Effect::ExchangeControlChoosing` (controller picks their own permanent at resolution, the opponent's is the cast target). Remaining ⏳: an *until-end-of-turn* exchange variant.
- ✅ **CR 701.16 — Sacrifice** — `GameEvent::CreatureSacrificed`/`PermanentSacrificed` distinct from the lethal-damage/`Destroy` die path; `EventKind::CreatureSacrificed` triggers fire only on genuine sacrifice (Mortician Beetle). Targeted sacrifice of an already-chosen permanent ✅ via `Effect::SacrificePermanent { what }` (fires sacrifice + death triggers; Footsteps of the Goryo / Apprentice Necromancer sacrifice their reanimated creature at the next end step; `cr_701_16_targeted_sacrifice_fires_death_triggers`). Pay-mana-value-or-sacrifice-the-source ✅ via `Effect::SacrificeSourceUnlessPayManaValue` (Soul Tithe's upkeep tithe, granted to the enchanted permanent; the controller keeps it by auto-tapping its mana value, else it's sacrificed — `soul_tithe_*`). Remaining ⏳: batched multi-permanent sacrifice-cost picker. (Audit follow-up closed — the P1 death-funnel bypass family is fixed; all arms route through the shared funnels.)
- ✅ **CR 611.2 — Per-turn spell-cast locks by type** — `StaticEffect::OneNoncreatureSpellPerTurn` (Deafening Silence) and `OneNonartifactSpellPerTurn` (Ethersworn Canonist) join the existing `OneSpellPerTurn` (Rule of Law) lock at the central `perform_action` cast gate, counted via `Player.{noncreature,nonartifact}_spells_cast_this_game_turn` and read off the cast spell's types (`GameAction::cast_card_id`). Surfaced to clients via `PlayerView.spell_cast_lock`. Tests `cr_611_2_deafening_silence_locks_only_noncreature_spells`, `ethersworn_canonist_limits_nonartifact_spells`.
- ✅ **CR — Defense Grid spell tax** — `StaticEffect::SpellsCostMoreExceptOnControllerTurn { amount }` folded into `extra_cost_for_spell`, skipped on the caster's own turn (`defense_grid_taxes_off_turn_spells`).
- ✅ **CR 700.13 — Commit a crime** — `EventKind::CommittedCrime` /
  `GameEvent::CommittedCrime` fires once per spell-cast or ability-activation
  whose chosen targets include an opponent, a permanent/card an opponent
  controls or owns, or a spell they control (detected at the cast / activate
  choke points via `target_is_crime`). `Player.committed_crime_this_turn` +
  `Predicate::CommittedCrimeThisTurn` back "if you've committed a crime this
  turn" gates. Ships Gisa, Magda, Marchesa, Forsaken Miner, Nimble Brigand
  (`decks::recent20`). ⏳: "commit a crime" by an ability targeting a spell/
  ability an opponent controls (only spell targets are checked on the stack).
- ✅ **CR 701.60 — Suspect** — `Effect::Suspect { what }` + `Effect::ClearSuspected { what }` (the "no longer suspected" inverse) + `CardInstance.suspected`; a suspected creature gains computed Menace + CantBlock (injected in `gather_continuous_effects`). `Predicate::SourceIsSuspected` gates Repeat Offender's toggle. Ships Barbed Servitor, Repeat Offender, Reasonable Doubt, Absolving Lammasu (ETB clears, death suspects).
- ✅ **CR 119 (life-matters) — gained-or-lost-life gate** — `Predicate::PlayerGainedLifeThisTurn` (backed by `Player.life_gained_this_turn`) complements `PlayerLostLifeThisTurn`; powers end-step "if you gained or lost life this turn" payoffs (Starlit Soothsayer).
- ✅ **CR 702.176 — Valiant** — `shortcut::valiant()` consolidates the once-per-turn `BecameTarget + YourControl` trigger (Heartfire Hero, Nettle Guard, Veteran Guardmouse, Emberheart Challenger, + Seedglaive Mentor / Mouse Trapper / Flowerfoot Swordmaster / Whiskerquill Scribe).
- ✅ **CR 701.35 — Detain** — `Effect::Detain { what }` + `CardInstance.detained_by`; a detained permanent can't attack/block (combat gates) or have its abilities activated (`activate_ability` gate), lifting at the detainer's next turn (`do_untap`). Surfaced in `PermanentView.detained` + a client tooltip badge. Ships Lyev Skyknight. ⏳: granted "enters detained" statics. (Loyalty activation now honors `detained_by`; Detain's target filter is enforced at cast time.)
- ✅ **CR 701.29 — Fateseal** — `Effect::Fateseal { who, amount }`: look at the top N of a targeted opponent's library, the controller may bottom any (Scry's library-side mirror). Decided inline (the `wants_ui` suspend prompt is a follow-up).
- ✅ **CR 701.57 — Discover N** — `Effect::Discover { n }`: exile from top until a nonland MV≤N, cast it free or put in hand (controller's choice), bottom the rest. Ships Geological Appraiser, Trumpeting Carnosaur. (Cascade-adjacent; shares the bottom-the-rest tail.)
- ✅ **CR 701.59 — Collect Evidence N** — `Effect::CollectEvidence { amount, then }`: optionally exile graveyard cards totaling MV≥N, then run the reflexive payoff. A `wants_ui` controller picks via `ChooseCards` (sum-validated); bots/tests keep the auto cheapest-pick. Ships Sample Collector, Izoni.
- ✅ **CR 602.5b — Additional activation costs (cont.)** — two new cost forms on `ActivatedAbility`: `bounce_other_filter` ("Return a [filter] you control to its owner's hand:" — Quirion Ranger, Wirewood Symbiote) and `tap_n_filter` ("Tap N untapped [filter] you control:", source eligible — Heritage Druid). Both gate pre-payment + auto-pick lowest-power, surface in `ability_cost_label`, and are excluded from the bot's `is_free_mana_ability`. The whole cost-sacrifice batch now reaches resolution as one unit (`Effect::WithSacrificedPt` carries the batch count + total power), and `sac_any_number_filter` adds the "…and any number of [filter] you control" form — a `ChooseCards` modal for hand-paying seats, everything else takes all candidates; zero is a legal payment (`cr_602_5b_*`).
- ✅ **CR 701.16 / 614 — "Opponents can't make you sacrifice"** — `StaticEffect::OpponentsCantMakeYouSacrifice`, consulted in the `Effect::Sacrifice` resolver (skips a player whose opponent's effect would force a sacrifice; own-sacrifice unaffected). Ships Sigarda, Host of Herons + the sacrifice half of Tamiyo, Collector of Tales.
- 🟡 **CR 614 — Replacement Effects** — general "instead" framework. Damage *halving* ✅ (614.5 — `StaticEffect::HalveDamageDealt`, Ghosts of the Innocent; composed with doublers via `scale_damage` at both damage funnels). Skip-step (614.10) ✅ via `StaticEffect::SkipStep` consulted in `advance_step` — a skipped upkeep/draw never occurs (no turn-based actions, triggers, or priority); a skipped untap skips untapping/phasing/day-night but the turn still starts (Eon Hub, Stasis). Skip-*turn* ✅ (`Player.skip_turns`, Chronatog / Ral Zarek -7). Damage *redirection* (614.9) ✅ via `StaticEffect::RedirectDamageToSelf` at both damage funnels (Palisade Giant; one redirect per event per 614.5). (ETB-counters, token/counter/damage *doubling*, regen, EtbTriggerTax, Maze-of-Ith per-source prevention ✅. Creature-ETB / death **trigger suppression** ✅ via `StaticEffect::SuppressCreatureEtbTriggers { also_dies }` — Torpor Orb / Tocatli Honor Guard / Hushbringer; `etb_trigger_multiplier` returns 0 for creature entrants and the dies-trigger gather paths skip while a suppressor is in play.) Enters-*untapped* replacement ✅ — `StaticEffect::LandsEnterUntapped` overrides any enters-tapped effect for the controller's lands in `apply_enters_tapped_replacement` (Spelunking).
- 🟡 **CR 615 / 614.9 — Prevention & redirection** — source+target-scoped prevention ✅ (`PreventDamageToYourCreaturesFromYourSources` — Light of Sanction; `PreventThisDamageToColor` — Indentured Oaf's own damage to red creatures; both wired into the combat + noncombat funnels — `cr_recent14`). Damage **redirection** (614.9) ✅ — `Effect::RedirectNextDamage` + `PreventionShield.redirect_to` deals the soaked N to a chosen permanent (Carom, Razia); `RedirectControllerDamageToEquippedCreature` sends a player's damage to the equipped creature (Pariah's Shield). Global "combat damage can't be prevented" ✅ (`StaticEffect::CombatDamageCantBePrevented` — Frenzied Baloth; bypasses shields for any creature-sourced damage, sharing the Questing-Beast combat approximation). Source-scoped "damage dealt by this can't be prevented" ✅ (`StaticEffect::SourceDamageCantBePrevented` — Excruciator; keyed on the damage source in `apply_prevention_shields`, so only its own damage bypasses shields — `cr_615_12_excruciator_source_scoped_unpreventable`). Per-source / per-N shields ✅ (`PreventionShield.source` + `Effect::PreventNextDamageFromChosenSource` — Wojek Apothecary, Stave Off). Prevented damage can now be **redirected to a player**, not just a permanent (`PreventionShield.redirect_to_player` — Acolyte's Reward at face). Non-combat prevention breadth — Mending Hands ✅ (next-4 shield on any target); prevent-and-gain ✅ via `Effect::PreventNextDamageAndGainLife` + `PreventionShield.gain_life` (Reverse Damage, Candles' Glow — `candles_glow_prevents_and_gains`). Attachment-scoped combat fog ✅ (`StaticEffect::PreventAllCombatDamageToAttached` — General's Kabuto carries the prevention for its host). Player-scoped combat fog ✅ (`Effect::PreventAllCombatDamageToPlayerThisTurn` — "prevent all combat damage that would be dealt to you this turn", Druid's Deliverance; `GameState.combat_damage_prevented_to_players_this_turn`, honored in `prevent_combat_to_target` — `druids_deliverance_prevents_combat_damage_to_you`). Player+permanents noncombat prevention ✅ (`StaticEffect::PreventNoncombatDamageToYouAndYourPermanents` — The Wanderer; gates the noncombat funnel for both the controller and any permanent they control — `the_wanderer_prevents_noncombat_damage_to_you`). Source-of-your-choice prevention (615.7) ✅ via
  `Effect::PreventAllDamageFromChosenSourceThisTurn` +
  `GameState.damage_prevented_sources`, consulted at both damage funnels
  (Burrenton Forge-Tender; the source is chosen as the ability resolves,
  among stack spells and battlefield permanents). Per-shield source
  restriction ✅ — `PreventionShield.{source,one_event}` +
  `Effect::PreventNextDamageFromChosenSource` (the damage source is now
  threaded through `apply_prevention_shields` at both funnels; Circle of
  Protection cycle, Rune of Protection: Red/Black). Blanket controller immunity
  ✅ — `StaticEffect::PreventAllDamageToController` (Glacial Chasm) at the
  player-directed branch of both funnels; surfaced as `PlayerView
  .damage_fully_prevented` + a client "🛡 immune" chip. Your-creatures noncombat
  immunity ✅ — `StaticEffect::PreventNoncombatDamageToYourCreatures` (Mark of
  Asylum; noncombat-only because combat damage to creatures is marked off the
  shared funnel). Turn-scoped incoming-only combat prevention ✅ —
  `Effect::PreventCombatDamageToTargetThisTurn` + `GameState
  .combat_damage_prevented_to_this_turn`, consulted at the
  `combat_damage_prevented_to_self` chokepoint (Fleeting Flight; the creature
  still deals its own combat damage). Remaining ⏳: outgoing-only combat
  prevention; per-source combat shields for a single creature.
- 🟡 **CR 500 — Turn structure** — `Predicate::CurrentStepIs(TurnStep)` gates "activate only during [your] upkeep/end step" abilities (Mirror Universe, Magus of the Mirror). Extra **combat-phase** insertion ✅ (CR 505.1b — `AdditionalCombatPhase` at End of Combat + `AdditionalCombatPhaseAfterMain` post-main re-entry, Relentless Assault). Extra **upkeep steps** ✅ (CR 500.9 — `Effect::AdditionalUpkeepStep` + `Predicate::IsFirstUpkeepThisTurn`; Paradox Haze, `cr_500_9_*`). Remaining ⏳: extra draw/main steps (no card yet needs them).
- ✅ **CR 702.113 — Awaken** — rides `AlternativeCost { target_filter, effect_override }`: awaken cast adds the counters + a permanent-duration `BecomeCreature` on the targeted land (Part the Waterveil).
- 🟡 **CR 305 — Lands** — see git for the per-clause detail. `LandType::Cave`
  added (CR 305.6 land subtypes), unblocking the LCI Cave lands + Caves-matter
  payoffs (Forgotten Monument grant, Compass Gnome tutor, Gargantuan Leech
  affinity, Spelunking). One-shot additive basic-land-type grant ✅
  (`Effect::GainAllBasicLandTypes` — layer-4 `AddLandType` ×5 per resolved land,
  CR 305; Energybending, `energybending_fixes_lands_and_draws`). Counter-gated
  land-type static ✅ (CR 305.7 — `StaticEffect::LandTypeChangerWhileCounters`
  only materializes while the source holds ≥N of a counter kind; Zhao, the Moon
  Slayer — "nonbasic lands are Mountains while Zhao has a conqueror counter";
  `zhao_taps_nonbasics_and_conquers_to_mountains`). As-enters *chosen*-basic-type
  additive static ✅ (CR 305.6/305.7 — `Effect::ChooseBasicLandTypeForSource`
  stamps `CardInstance.chosen_land_type`, `StaticEffect::LandsYouControlAreChosenType`
  adds it to your lands with the intrinsic mana ability following; Realmwright,
  `cr_305_6_realmwright_land_taps_for_chosen_color`).
- 🟡 **CR 701.48 — Learn** — populate Lesson sideboards in the format / draft deck-build paths (engine + cube ✅).
- 🟡 **CR 702.15 — Lifelink** — LKI corner (702.15c): triggered-ability source leaving the battlefield mid-resolution.
- 🟡 **CR 701.34 — Proliferate** — permanents' counters + player poison ✅;
  player experience/energy ✅; "whenever you proliferate" triggers ✅
  (`EventKind::Proliferated`, fires once per instance, incl. from the
  graveyard — Voidwing Hybrid); "proliferate twice instead" ✅
  (`StaticEffect::ProliferateTwice`, 2^n for n Tekuthals). Remaining:
  per-player UI choice of which permanents/players to proliferate.
- 🟡 **CR 601 — Casting Spells** (logged as "CR 706 — Casting spells") — minor; see git. Symmetric off-turn cast lock ✅ (`StaticEffect::PlayersCastOnlyOnOwnTurn` — Dosan the Falling Leaf gates every seat that isn't the active player, its controller included; `dosan_locks_off_turn_casts_for_both_seats`). "Opponents can't cast from anywhere but their hands" ✅ via `StaticEffect::OpponentsCantCastFromAnywhereButHand`, checked in `cast_from_zone_blocked`. The foretell / plot / adventure-creature exile-cast paths now gate on it too (`cast_foretold`/`cast_plotted`/`cast_adventure_creature`; test `drannith_magistrate_blocks_foretold_cast`). Suspend's eventual cast gates on the same lock ✅ (`cast_card_for_free` → `cast_from_zone_blocked`; test `cr_702_62e_suspend_final_cast_blocked_by_drannith`). CR 601.2 "unless"-cost affordability: `punisher_option_affordable` now rejects an empty-hand `Discard` dodge (can't choose a cost you can't pay), so a hand-empty player takes the penalty (`perforating_artist_*`, `osseous_sticktwister_delirium_punisher`). CR 702.8 flash-timing: the cast-timing check now honors the `ControllerSorceriesAsFlash` static (was a no-op — only `ControllerSpellsHaveFlash` was consulted), so Teferi, Time Raveler's static and Hypersonic Dragon let their controller cast sorceries at instant speed (`teferi_static_grants_controller_sorceries_as_flash`); the six duplicated `flash_granted` blocks collapsed into one `battlefield_grants_flash` helper.
- ✅ **CR 702.29 — Cycling** — plain Cycling ✅. Typecycling/Landcycling
  (702.29e) ✅ via `Keyword::Landcycling(cost, LandType)` and the general
  `Keyword::Typecycling(cost, filter)` ("Basic landcycling" — Ash Barrens),
  both through `GameAction::Landcycle` (pay + discard → fetch a matching
  card to hand, shuffle; fires cycle triggers); surfaced in
  `KnownCard.has_landcycling` + a client Landcycle keybind. Ships Wirewood
  Guardian, Daru Lancer, the LTR cycle (Troll of Khazad-dûm, Lorien
  Revealed, Eagles of the North, Oliphaunt, Generous Ent), Ash Barrens.
  UI pick among multiple matches ✅ (`ResumeContext::ActionSearchPick` —
  a `wants_ui` cycler suspends before costs and picks the fetch).
- 🟡 **CR 117.1 — Order of priority** — APNAP corner cases; see git.
- 🟡 **CR 301 — Artifacts** — see git.
- ✅ **CR 701.8 — Destroy / 701.19 Regenerate** — `regeneration_shields` replace destruction on the SBA lethal-damage path, `Effect::Destroy`, and consume one shield (tap + remove-from-combat + heal). `DestroyNoRegen` bypasses. Toughness≤0 SBA correctly ignores shields.
- 🟡 **CR 800 — Multiplayer / leaving the game** — see git.
- 🟡 **CR 903 — Commander Variant** — 903.4d back-face identity ✅; 903.4
  color-indicator + activated-ability-cost + adventure/split-half identity ✅
  (`format::color_identity` unions them; `cr_903_4_identity_*`). Remaining:
  903.9 optional rider.

### Todo (⏳)
- ✅ **CR 314 / 900 / 904 — Archenemy.** `CardType::Scheme` +
  `Supertype::Ongoing`; `Player.scheme_deck`, `GameState.archenemy` and
  `seat_archenemy` (40 life, first turn, CR 904.5/904.6). CR 904.9's
  set-in-motion is a turn-based action at the archenemy's precombat main
  (`set_scheme_in_motion` + `EventKind::SetInMotion`); CR 904.10's sweep is an
  SBA (`sweep_finished_schemes`); CR 701.33 abandon ships as
  `Effect::AbandonThisScheme`. A face-up scheme's statics and step triggers
  function from the command zone (CR 904.8) — anthem gather and
  `fire_step_triggers` both walk it. `sets::arc` (8 schemes),
  `classic_sets/arc`. Residual ⏳: the CR 904.2 team/attack-multiple-players
  seating is left to the caller, and All in Good Time's "schemes can't be set
  in motion that turn" rider isn't modeled.
- ✅ **CR 612 — Text-Changing Effects** — layer-3 `Modification::ReplaceColorWord`
  / `ReplaceBasicLandType` + `Effect::ReplaceColorWord`/`ReplaceBasicLandType`
  (two ChooseColor prompts pick from/to; basics map 1:1 onto colors). Rewrites
  Protection-from-color, landwalk, and the type line (a swapped basic taps for
  the new color). Trait Doctoring (EOT + Cipher), Mind Bend (permanent).
  Remaining ⏳: full text-box swaps (Spy Kit, Volrath's Shapeshifter) and
  ability-text color words beyond keywords.

## Suggested next-up tasks

- ⏳ **A "next spell only" spend permission.** North Star grants CR 609.4b for
  the whole turn (`Player.may_spend_any_color_this_turn`); the printed card
  scopes it to one spell.
- ⏳ **`Duration::UntilYourNextUpkeep`** — Halfdane, Gabriel Angelfire and the
  rest of the "until your next upkeep" wordings currently round to
  `Permanent` / `UntilYourNextUntap`.

- ✅ ~~**Onslaught's last 4 gaps**~~ — shipped (`sets::ons4`); `set_gaps.py ons`
  is at zero. Each landed its primitive: `Effect::ReplaceCreatureTypeText`,
  `Keyword::DividesCombatDamageAmongDefenders`, `Effect::EachPlayerChoosesNumberHighestLoses`,
  and `GameEvent::ControlChanged` + `EventKind::GainedControlOfThis`.
- ⏳ **Legends is at 9 gaps** (`set_gaps.py leg`). Seven waves shipped 264
  cards; see "Legends — opened" above for what's left and why — each remaining
  card is blocked on one primitive.
- ⏳ **The client can't declare attacking bands.** The engine action
  (`GameAction::DeclareAttackersBanded`) and the `ClientView.attack_bands`
  read-back ship, and the tooltip names a creature's bandmates, but the
  attack UI has no band-grouping affordance — a human player can only attack
  unbanded.
- ⏳ **`Effect::ReplaceCreatureTypeText` rewrites the definition via a serde
  round-trip.** It substitutes any string value equal to the type's variant
  name, skipping the `name` key. That reaches every filter and effect body
  without a per-variant visitor, but a future enum with a unit variant named
  after a creature type would be caught in the same net.
- ⏳ **The auto-targeter fills only one graveyard slot of an "up to N target"
  trigger** (Celestial Gatekeeper). `auto_extra_targets_for` now peels a `Seq`
  whose only targeting member is the multi-target body, and the graveyard walk
  honors the `avoid` set, but the extra slot still comes back empty — the
  remaining break is somewhere between the peel and
  `auto_target_for_effect_avoiding_set`. A `wants_ui` seat picks both.
- ⏳ **Spy Network's "top card of that player's library" clause is dropped.**
  The hand and face-down halves ship (`LookAtHand` + `LookAtFaceDown`); a
  one-card library peek needs a `library_top_revealed_to` twin of
  `GameState.face_down_revealed_to`.
- ⏳ **`RevealTopOpponentChoosesToHand`'s opponent is a heuristic**, not a
  prompt — it hands over the lowest-mana-value eligible card. Fine for
  Karn's +1 and Animal Magnetism, but a real pick belongs on the opposing
  seat's decider.
- ⏳ **Bot matches aren't reproducible.** `RandomBot` draws from the global
  RNG, so `bot_vs_bot_commander_demo_terminates` varies 0.5s–15s+ run to run
  and occasionally blew its old 120s ceiling. The ceiling is now 600s; the
  real fix is a seeded RNG on `RandomBot` (and a seed printed on failure).
- ⏳ **`Effect::EachPlayerChoosesCreatureTypeThen` asks the synchronous
  decider for every seat**, so a UI player isn't prompted for their own
  Harsh Mercy / Patriarch's Bidding pick (same gap as `TemptingOffer`). The
  single-chooser `ChooseCreatureTypeThen` does suspend correctly.
- ⏳ **`Effect::HeadGames`' search is a single `ChooseCards` prompt**, not the
  standard `SearchLibrary` flow, so it doesn't route through the search-tax /
  can't-search statics. Fold it into `Effect::Search` once that path can
  search one player's library on another player's picks (CR 701.19a).
- ⏳ **`Effect::MayCopyThisSpell` prompts the affected seat through the
  installed decider**, not that seat's UI suspend — see the Server bullet
  above. Same for the chain's retarget (`repoint_copy_target`).
- ⏳ **The Chain cycle's toll is all-or-nothing per link.** The printed cards
  let the affected player decline the copy *after* paying (they may sacrifice
  a land, and only then choose whether to copy); the engine asks first and pays
  only on a yes. Observationally identical unless a sacrifice trigger changes
  the player's mind.
- ✅ ~~**Bot multi-block seeding**~~ — the spare-capacity pass now seeds from
  every legal blocker with extra capacity, not just the ones the scoring loop
  assigned, so an idle 0/N `CanBlockAnyNumber` wall soaks the whole swing
  (`bot_soaks_the_swing_with_an_idle_wall`).
- ⏳ **CR 121.8 / 121.9** — mid-cast face-down draw and reveal-on-draw, the two
  remaining CR 121 clauses.
- 🟡 **CR 115.7c** — "change any targets" now walks every declared slot
  (`Effect::ChangeTargetOfAbility`; test `cr_115_7c_reroute_repoints_every_slot`).
  Remaining: letting the chooser keep a *subset* of the current targets rather
  than repointing each slot that has an alternative.
- ⏳ **Sector designations are auto-assigned.** `GameState::assign_sectors`
  (CR 704.5u) spreads a player's creatures round-robin instead of asking; a
  `wants_ui` seat should get the real choice. `Effect::ChooseSector` likewise
  auto-picks the fullest sector for a bot/auto seat.
- ⏳ **Search the City's return is auto-picked.** With several exiled copies of
  a name, `Effect::SearchTheCityReturn` returns the first — the printed text
  lets the controller choose which.

- ⏳ **recent239 (DSK/OTJ/MKM) deferred, each blocked on one primitive:**
  - ✅ ~~**Collect-evidence additional cost**~~ — shipped
    (`AdditionalCastCost::CollectEvidence { amount, optional }` +
    `Predicate::SpellCollectedEvidence` + `self_cost_reduction_if_collect_evidence`).
    Bite Down on Crime (real {2}-less discount), Behind the Mask (4/3 vs 1/1),
    and Analyze the Pollen (widened search) are wired; auto-collects when the
    graveyard can afford it. Axebane Ferox's **Ward—Collect evidence 4** ships
    too (`WardCost::CollectEvidence`). Still open: an interactive collect prompt
    for UI casters (currently auto-collects).
  - ✅ ~~**"Whenever you manifest dread" trigger**~~ — shipped
    (`EventKind::ManifestedDread` + `GameEvent::ManifestedDread { player, milled }`
    + wire mirror + milled-card subject binding). Paranormal Analyst is wired.
  - ✅ ~~**Per-turn face-down activity flag**~~ — shipped
    (`Player.face_down_activity_this_turn` + `Predicate::FaceDownActivityThisTurn`,
    set on a face-down ETB or turn-face-up, CR 708). Oblivious Bookworm is wired.
  - **Type-filtered death tally** — "if a non-Zombie creature died this turn"
    (Undead Sprinter's graveyard-cast condition). Needs either a filtered
    death predicate or a small per-turn typed tally on `Player`.
  - **Tap-1-or-2-then-each-deals-power** — Coordinated Clobbering (needs
    explicit tapper target slots + a shared recipient slot).
  - ✅ ~~**Choose/reveal-creature-power additional cost**~~ — shipped
    (`AdditionalCastCost::ChooseOrRevealCreature` threads the chosen/revealed
    creature's power into the spell's X via `Value::XFromCost`). Monstrous
    Emergence is wired; auto-picks the highest-power creature.
  - **Dual-pile exile-return-to-hand linked to LTB** — Fear of Abduction (the
    additional-cost-exiled own creature and the ETB-exiled opponent creature
    both return to their owners' hands when it leaves).
- ⏳ **Newly-noticed primitives (RNA batch):**
  - **Your instants/sorceries have deathtouch** static — Pestilent Spirit
    ("Instant and sorcery spells you control have deathtouch"). No static
    grants deathtouch to a player's I/S spell damage yet.
  - **Opponent activates a nonmana ability of an artifact/creature/land →
    ping** — Immolation Shaman. `EventKind::AbilityActivated` exists but there
    is no scope/filter for "source is an artifact/creature/land, nonmana."
  - **Tap N untapped creatures of a type as a cost** — Persistent Petitioners'
    "Tap four untapped Advisors you control: target player mills twelve" (only
    its `{1},{T}: mill 1` half would ship without this). Also its
    "any number of copies in a deck" deckbuild waiver.
  - **Land animation with haste that stays a land** — Clan Guildmage's second
    mode ("target land becomes a 4/4 Elemental with haste; still a land").
  - **Move a +1/+1 counter between your creatures** — Combine Guildmage's
    second ability + its "creatures enter with an extra counter this turn."
  - **Riot as a granted static** (Rhythm of the Wild) — riot currently only
    ships as an intrinsic ETB trigger, not a "nontoken creatures you control
    have riot" anthem; plus its "creature spells can't be countered."
  - **Opening-hand reveal → first-upkeep bonus** (Sphinx of Foresight) —
    approximated as a recurring upkeep scry 1; the reveal-from-opening-hand
    path (an `OpeningHandEffect`) isn't wired for the scry-3 rider.
  - **Spells targeting this cost {2} more for opponents** (Sphinx of New Prahv)
    — a self-referential targeted-spell tax static.
- ⏳ **Newly-noticed primitives (discovered during the DSK/BLB gap batch):**
  - **Gift on a permanent (creature/artifact)** — the gift's `gifted_effect`
    resolves only on the instant/sorcery spell path; a Gift *creature*
    (Scrapshooter, Starforged Sword) needs the permanent-ETB path to check
    `card.gift_promised` and run `gifted_effect` as the ETB.
  - **Forage / cost-hybrid mana abilities** — Thornvault Forager's
    "{T}, Forage: add two mana" wants a forage additional cost on
    `ActivatedAbility` (only cast-cost `Effect::Forage` exists today).
  - **Enchant-player auras + `PlayerStaticTarget::Enchanted`** — Grievous Wound
    ("enchanted player can't gain life; when dealt damage, lose half life"):
    no player-attaching aura support today.
  - **"You gave a gift" trigger** (`EventKind::GaveGift`) — Jolly Gerbils.
  - **Delirium-gated modal count** ("choose one; if delirium, choose one or
    more instead") — Let's Play a Game.
  - **Per-turn ability-resolution count** ("draw if this is the second time
    this ability resolved this turn") — Harvestrite Host.
  - **"No mana spent to cast" ETB gate** — Freestrider Commando's
    enters-with-two-counters (verify `ctx.mana_spent` is threaded to a
    self-ETB trigger before wiring; the plot/reanimate cases both want 0).
  - **Type/ability rewrite auras** ("becomes a colorless Food artifact with …,
    loses all other card types and abilities") — Sugar Coat.
- ⏳ **Deferred cards from the recent156-161 waves (each blocked on one
  primitive):**
  - **Two-target "your creature deals damage = power to their creature"** —
    Felling Blow. The `Selector::Target(0/1)` shape works (Hunter's Edge) but
    the per-slot you-control / opponent target filters aren't declared, so it's
    approximated; wants explicit multi-target-slot filters.
  - ✅ ~~**Target-conditional cost reduction**~~ — shipped via
    `CardDefinition.self_cost_reduction_cost_if_target: (filter, cost)` (read
    in `extra_cost_for_spell`). Titanic Brawl ("{1} less if it targets a
    +1/+1-countered creature you control") uses it; Luminous Rebuke's
    tapped-creature discount is the same shape.
  - **Per-creature "prevent all combat/creature damage this turn" shield** —
    Fleeting Flight, Eerie Interference (fog scoped to one creature / player).
  - **Reflexive "discard N, then N targets get -2/-2"** — Miasma Demon links a
    variable discard count to a variable target count.
  - **"Your +1/+1-counter creatures have first strike during your turn"** —
    Inspiring Paladin's team clause (a PumpTeamIf gated on both a turn predicate
    and a per-creature counter filter).
- ⏳ **recent127-128 (OTJ/WOE) follow-ups / deferred:**
  - **Young Hero Role toughness gate** — the granted attack trigger fires
    unconditionally; the printed "if its toughness is 3 or less" wants a
    trigger-source toughness predicate.
  - ✅ ~~**Ego Drain**~~ — the "if you don't control a Faerie, exile a card from
    your hand" downside now fires (recent290; `Not(control-a-Faerie)` gate over
    `ExileFromHand`).
  - **Boneyard Desecrator** — the effect-path sacrifice (`SacrificeAndRemember`)
    doesn't stamp `sacrificed_was_outlaw` (only the activated `sac_other_filter`
    path does); wire the tuple if a spell ever needs it.
  - **Cactarantula / Consuming Ashes** (OTJ) still need a control-a-Desert cost
    reduction and a target-mana-value reflexive predicate, respectively. (Aloe
    Alchemist ✅ via the new `EventKind::BecomesPlotted` trigger.)
- ⏳ **recent131-134 (WOE waves 4-7) follow-ups / noticed:**
  - New primitives this run: `DynamicPt::NonlandPermanentsControlled` (Regal
    Bunnicorn `*/*`), `Keyword::CantBeBlockedByPowerAtLeast(N)` (Squeak By —
    the fixed-threshold mirror of `CantBeBlockedByPowerAtMost`), and the
    enchantment-matters idiom (`PermanentDied`/`EntersBattlefield` +
    `EntityMatches { TriggerSource, Enchantment/Aura }` — Wicked Visitor,
    Savior of the Sleeping, Ashiok's Reaper, Rimefur Reindeer, Tanglespan
    Lookout). Role tokens (Sorcerer/Cursed/Royal/Wicked) reused via
    `CreateTokenAttachedTo`; the Wicked Role's death-drain needed the engine to
    collect **`PermanentDied`/`SelfSource`** leave-triggers for non-creatures
    (previously only `CreatureDied`/`PermanentLeavesBattlefield` were gathered —
    fixed in `stack.rs`). Also new: `Value`-free `MayPay` reflexives on ETBs
    (Unassuming Sage, Snaremaster Sprite).
  - ✅ ~~**Dream Spoilers**~~ — shipped via a `Not(IsTurnOf(You))` SpellCast
    filter (the whose-turn predicate already existed); recent135.
  - ✅ ~~**Chancellor of Tales**~~ — shipped: `Predicate::CastSpellIsAdventure`
    (reads the cast spell's `adventuring` flag) + `CopySpellMayChooseTargets`;
    recent135.
  - ✅ ~~**Young Hero Role toughness gate**~~ — the granted attack trigger now
    carries `ValueAtMost(ToughnessOf(TriggerSource), 3)`; combat.rs was binding
    the Attacks-trigger source as `EntityRef::Card` (so `ToughnessOf` read 0) —
    fixed to `Permanent`. Role token helpers consolidated in `decks::woe_roles`.
  - ✅ ~~**Discerning Financier**~~ — shipped (recent290). Upkeep Treasure gated
    on `OpponentControlsMoreLandsThanYou`; the donate ability rides
    `GainControl { to: Some(EachOpponent) }` (the primitive already existed —
    Wishclaw Talisman) + draw. Also: `Effect::Punisher`'s `otherwise` now binds
    the defaulting chooser as `Triggerer` so a per-defaulter payoff is
    multiplayer-correct (Zoyowa Lava-Tongue).
  - ✅ ~~**Experimental Confectioner**~~ — Food-sac → Rat shipped via a
    `PermanentSacrificed`/`YourControl` trigger filtered on `HasArtifactSubtype(Food)`
    (recent135; test in recent138).
  - ✅ ~~**Break the Spell**~~ (destroy enchantment + conditional draw via
    `EntityMatches{Target, ControlledByYou|IsToken}`), ✅ ~~**A Tale for the Ages**~~
    (`AnthemForFilter{IsEnchanted}` — the gather now resolves non-card-only anthem
    filters against live state via `evaluate_requirement_static`), ✅ ~~**Moment of
    Valor**~~ (modal untap/pump/indestructible vs destroy-power-4). recent138.
  - recent136 deferred: ✅ ~~**Tangled Colony**~~ (X Rats = `Value::MarkedDamageOn`
    read via leaves-battlefield LKI; recent138), ✅ ~~**Torch the Tower**~~
    (Bargain 3-dmg + scry + `ExileIfWouldDieThisTurn`; modern.rs), ✅ ~~**Moonshaker
    Cavalry**~~ (already shipped in recent129), ✅ ~~**Gruff Triplets**~~ (ETB
    self-copy ×2 gated `NotToken` + dies +1/+1 to same-named; recent138), ✅ ~~**Specter
    of Mortality**~~ (`Effect::MayExileFromYourGraveyard { filter, then }` — reflexive
    variable graveyard-exile pins the exiled cards to `LastMoved`; recent138),
    **Rotisserie Elemental** (skewer-counter impulse), ✅ ~~**Howling
    Galefang**~~ (`Predicate::OwnExiledAdventureCard` + `SelfHasKeywordWhilePredicate`
    haste; recent138) / **Sentinel of Lost Lore** (adventure recursion modes).
- ⏳ **recent139 (WOE wave 12) noticed / deferred:**
  - **Gnawing Crescendo**'s "whenever a nontoken creature you control dies this
    turn, make a Rat" wants a delayed-death turn-scoped trigger sibling of
    `Effect::CreaturesYouControlEnteringThisTurn` (only the enters variant
    exists). The +2/+0 team-pump half is trivial once that lands.
  - **Eerie Interference** ("prevent all damage by creatures to you and your
    creatures this turn") wants a source-filtered scoped fog — the existing
    `PreventAllDamageThisTurn`/`PreventAllCombatDamageInvolving` don't gate on
    *dealer is a creature*.
  - **Expel the Interlopers** (destroy all creatures with power ≥ a chosen
    0–10) wants a dynamic power threshold in the destroy filter (filters take a
    fixed `i32`; the chosen number would need `PowerAtLeastValue`).
  - **Frantic Firebolt** approximates X = 2 + instant/sorcery cards in gy,
    dropping the "…or have an Adventure" graveyard contribution (no
    graveyard-card `HasAdventure` filter).
  - **Rotisserie Elemental** (skewer-counter impulse) and **Sentinel of Lost
    Lore** (exile-Adventure modal) still deferred. (Discerning Financier shipped
    — recent290.)
- ✅ ~~**recent141-145 (WOE waves 14-18) deferred cards**~~ — all shipped in
  `decks::recent146` (tests in `tests/recent146.rs`): Archon of the Wild Rose
  (state-aware `SetBasePtForFilter`/`GrantKeyword` for stateful `IsEnchanted`
  filters), Faunsbane Troll (`SelectionRequirement::AttachedToSource` sac-cost +
  `ExileIfWouldDieThisTurn`+`Fight`), Bitter Chill (existing `PreventUntap`
  AttachedTo), Syr Ginger (`SelfHasKeywordWhilePredicate`×3), Horned Loch-Whale
  (`StaticEffect::EntersTappedUnless`), Back for Seconds (bargain `If`/`MayDo`
  reanimation — the "up to two" targets are auto-picked), and Johann
  (`StaticEffect::PlayFromLibraryTopOncePerTurn` + `Player.cast_from_library_top_this_turn`).
- ⏳ **Noticed in recent146-148 (approximations worth revisiting):**
  - **Back for Seconds** returns only one card to hand if bargained but the
    reanimation is declined (the "up to two total" cap models the
    battlefield-put as *replacing* the second return); faithful when you take
    the reanimation. A true "choose up to two targets, then optionally redirect
    one" would need a post-target redirect step.
  - **Faebloom Trick / Twisted Sewer-Witch-style "when you do" reflexive taps**
    are modeled as a plain `Effect::Seq` (targets chosen up front) rather than a
    CR 603.7 reflexive trigger.
  - **ManifestDread + attach** (Cursed Windbreaker) attaches to "a face-down
    creature you control" because `Selector::LastMoved` is clobbered by the
    dread's second card going to the graveyard after the manifest. A
    `Selector::LastManifested` (or having `ManifestDread` stamp the manifested
    id) would let "attach to that creature" be exact when multiple face-downs
    exist.
  - **Johann once-per-turn** is a per-player flag, so two Johanns still grant
    only one top-of-library cast per turn (each printed ability is independently
    "once each turn").
- ⏳ **recent113 (MH1 + Eldrazi) follow-ups / deferred:**
  - **Vorinclex, Voice of Hunger** — needs a "whenever you/an opponent tap a
    land for mana" trigger (no `EventKind` for tap-land-for-mana yet); the
    mana-doubling half + opponent "that land doesn't untap next" half both
    hang on it. Praetor cycle is otherwise complete.
  - **It That Betrays** — "whenever an opponent sacrifices a nontoken
    permanent, put that card onto the battlefield under your control": needs a
    sacrifice-watching trigger + LKI of the sacrificed card for a reflexive
    reanimation (no such event today).
  - **Void Winnower** X-spell corner: the even-MV cast lock reads the *printed*
    mana value, so an `{X}` spell counts as MV 0 (even) regardless of the
    chosen X. Faithful for fixed-cost spells; thread the announced X to be
    exact.
  - ✅ ~~**Bellowing Elk**~~ — shipped via `Predicate::AnotherCreatureEnteredThisTurn`
    (self-excluding sibling of `CreatureEnteredThisTurn`); gated trample +
    indestructible statics. `decks::recent113`, test in `tests/recent131.rs`.
  - ✅ ~~**Windcaller Aven**~~ — cycle-trigger grants a target creature flying
    (`CardCycled`/`SelfSource` + `ApplyToTargets`). ✅ ~~**Twisted Reflection**~~ —
    modal Entwine using the existing `Effect::SwitchPT` (613.7d) + `PumpPT` -6/-0.
    Both in `decks::recent113`, tests in `tests/recent131.rs`.
- ⏳ **Deferred (noticed, not tackled):**
  - ✅ ~~Angel's Grace~~ — shipped (`Player.{cant_lose_this_turn,
    damage_floor_this_turn}` + `player_cant_{lose,win}_game` at every loss/win
    site; `decks::recent109`).
  - ✅ ~~Zabaz, the Glimmerwasp~~ — shipped: `Effect::ModularCounters` gives
    the modular death trigger its own funnel; `StaticEffect::
    ModularBonusCounters` adds the +1 (`tests/mh2h.rs`).
  - ✅ ~~Portent Tracker~~ — shipped: `Effect::AdjustBattleDefense`
    (CR 310.7; `decks::echo`, test `cr_310_7_portent_tracker_battle_defense`).
  - **Mycosynth Lattice** — "all permanents are artifacts" fits
    `AddCardTypeToMatching`, but the all-colorless + spend-any-color halves
    have no primitives.
  - **Glimpse of Tomorrow / Emperor of Bones** — shuffle-permanents-and-
    redeploy; exiled-with + counters-added reflexive return.
  - ✅ ~~Dies-redirects vs dies-triggers~~ — fixed: `death_was_replaced`
    (exile *or* library) gates dies-trigger dispatch, the `PermanentDied`
    synthesis, and `WhenCardDies` delayed watchers.
- ✅ **MH2 sweep — COMPLETE.** `python3 scripts/set_gaps.py mh2` reports 0
  missing (the script now checks full split-card names and skips `A-`
  Alchemy rebalances). ~180 cards across `decks::mh2b`–`mh2i`. Remaining
  per-card approximations are noted on their factory docs (Garth's copy is a
  real hand card; Ghost-Lit Drifter's channel hits one target; Chef's Kiss
  keeps the target when no hostile retarget exists).
- ⏳ **All Will Be One placer attribution** — `GameEvent::CounterAdded` carries
  no "who placed" seat, so the enchantment fires off counters landing on your
  permanents + poison hitting opponents (exact in two-player). Threading the
  placing controller through the counter funnel would make it exact in
  multiplayer and unlock "whenever an opponent puts a counter…" designs.
- ⏳ **Rhuk's dies-half** — "equipped creature … attacks or dies"; the dies
  half needs the victim's attachment list snapshotted before the equipment
  unattaches (LKI for attachments).
- ⏳ **CastWithoutPayingImmediate copy-mode** — Capricious Hellraiser should
  cast a *copy* (original stays exiled); a `copy: bool` rider on the effect
  would also serve future "copy it and you may cast the copy" cards.
- ✅ ~~Random graveyard exile selector~~ — `Selector::TakeRandom` already
  ships; wire Hellraiser/Sin onto it when revisited.

- ⏳ **recent34–38 follow-ups / deferred cards (this run):**
  - ✅ Quest cycle complete (`decks::quests`): Pure Flame
    (`Effect::DoubleYourSourcesDamageThisTurn` + the new
    `EventScope::YourSourceDamagedOpponent` over `DamageDealt.from_controller`),
    Nihil Stone, Ula's Temple, and the Holy Relic attach rider
    (Search → `Attach { LastMoved, GreatestPowerYouControl }`).
  - Approximations still to revisit: Pir's Whim (full friend/foe vote →
    you=friend/opponents=foe), Three Dreams (different-names search dropped).
    ✅ Gather the Pack (spell mastery's 2nd creature via `Effect::MillThenToHandN`
    + `Value::IfAtLeast` over I/S in gy), ✅ Hour of Promise (3+ Deserts Zombies),
    Golden Demise
    (Ascend + city's-blessing opponents-only), Yahenni's Expertise (MV≤3 free
    cast via `CastFromHandWithoutPaying`), and Goblin Assault ("Goblins attack
    each combat" via `GrantKeyword(MustAttack)`) are now faithful.
  - ✅ Pulmonic Sliver (`StaticEffect::DiesToLibraryTopInstead`), Goblin
    Welder, Gilt-Leaf Archdruid, Pyromancer Ascension, and Twilight Prophet
    (`RevealTopToHandLoseMv.you_gain`) all ship now. ✅ Bonehoard
    (`EquipScale.count_all_graveyards`), **Necropolis Fiend**
    (`ActivatedAbility.exile_other_x` — {X},{T}, exile X from gy: −X/−X), and
    **Caustic Bronco** (`RevealTopToHandLoseMv { who }`) now ship.
    (Fog Bank / Guard Gomazoa shipped via `StaticEffect::PreventAllCombatDamage-
    ToThis`; Wall of Denial shipped as Defender/Flying/Shroud per current oracle.)

- ⏳ **recent31 (multicolor staples) follow-ups / deferred cards:**
  - Dimir Charm mode 3 ("look at top three, put one back, rest into graveyard")
    is modeled as **mill 2** — wants a look-top-N-keep-one-rest-to-graveyard
    effect (a target-player surveil-to-graveyard variant).
  - Atarka's Command mode 3 ("put a land from your **hand**") reuses
    `PutFromHandOrGraveyardOntoBattlefield` (also allows graveyard) — wants a
    hand-only put primitive.
  - Foul-Tongue Invocation drops the "reveal a Dragon from hand" additional
    cost; the bonus 4 life is gated on controlling a Dragon instead.
  - **Deferred — need new primitives:** Necropolis Fiend ({X},{T}, exile X from
    gy: −X/−X — activated abilities have no `{X}` cost + Value-count gy-exile);
    Bonehoard (living-weapon equip
    +X/+X where X = creature cards in all graveyards — `EquipScale` counts
    battlefield permanents, not graveyards); Dromoka's Command (mode "prevent
    all damage target instant/sorcery would deal" — no prevent-spell-damage
    effect); Pyromancer Ascension (quest-counter + spell-copy enchantment);
    Crime // Punishment (split card).
- ⏳ **recent20 (OTJ) approximations / follow-ups:** (✅ Magda, the Hoardmaster's
  "Sacrifice three Treasures: make a 4/4 Scorpion Dragon" now ships via
  `sac_other_filter: Some((Treasure, 3))`.) Gisa's "Ward—{2}, Pay 2 life" is
  modeled as Ward—{2} (the life half is dropped — `WardCost` has no compound
  variant); Bovine Intervention mints the Ox before the destroy so
  `ControllerOf(Target)` still resolves. Also: CR 700.13 crime detection covers
  cast + activated-ability targets but not a *triggered* ability that targets an
  opponent's stuff as it's put on the stack, nor targeting a spell/ability an
  opponent controls beyond stack *spells* (abilities on the stack aren't
  checked). **Spree** (Lively Dirge) still ⏳ — multi-chosen additional costs.
- ⏳ **recent21 approximations:** Trick Shot drops the "2 to another target
  creature token" rider (just 6 to a creature); Patient Naturalist drops the
  "else create a Treasure" when no land is milled. Stingerback Terror / Canyon
  Crab / Bedrock Tortoise deferred (card-in-hand-scaled CDA P/T, a "didn't cast
  from hand this turn" flag, and assigns-damage-by-toughness, respectively).
- ⏳ **recent17–18 (Foundations) approximations to revisit:** Kitsa, Otterball
  Elite drops the "{2},{T}: copy target instant/sorcery you control" ability
  (needs a copy-spell activated ability gated on power ≥ 3); Run Away Together
  is modeled as any-two-creatures bounce (the "controlled by different players"
  restriction isn't enforced); Sky Crier / Dryad Greenseeker approximate
  "put into hand" as a draw; Angel of Finality / Burglar Rat target each
  opponent rather than a chosen player (1v1-faithful). (✅ Charmed Sleep ships as
  a `tap_down_aura` — ETB tap + `PreventUntap` on the host.)
- ⏳ **Cards noticed this run (recent12–15) but deferred — need new primitives:**
  Kutzil's Flanker (mode 1 wants a "creatures that left the battlefield under
  your control this turn" count); Caustic Bronco (attack-reveal life-loss/drain
  equal to the *revealed card's* mana value — a Value reading a just-revealed
  card); Mosswood Dreadknight (cast-from-graveyard-as-Adventure death rider);
  Ao, the Dawn Sky (dies-modal "look top 7, deploy nonland permanents with total
  MV ≤ 4"); Gix, Yawgmoth Praetor (combat-damage "may pay 1 life: draw" +
  discard-X exile-and-play); Valgavoth (opponent-graveyard-exile replacement +
  play-from-exile-paying-life); Battle Cry Goblin (Pack tactics — "if you
  attacked with total power ≥ N this combat"); Goblin Recruiter (search any
  number + arrange on top); Divergent Transformations / Seeds-cycle's last
  Undaunted card (polymorph-reveal-until-creature).

- ⏳ **Equipment-matters follow-ups** (`decks::recent12`): a
  **token-exile-at-next-end-step** delayed trigger (Valduk's Elementals
  currently persist); Nahiri's −8 currently drops the deployed permanent's
  haste + return-to-hand rider (search-to-battlefield only). Bruenor
  Battlehammer needs a per-creature "+2/+0 per Equipment attached to *it*"
  anthem + a free-first-equip-each-turn allowance. (While-equipped team anthems
  + conditional keyword-by-attached-count ✅ — Auriok Steelshaper, Balan.)

> **Reprioritized 2026-06-11:** the correctness-audit section at the top of
> this file outranks everything below. New-card/primitive work should wait
> behind at least the audit P0 tier (and the P3 root-cause refactors, which
> make every subsequent card batch safer to land).

- ℹ️ **Client builds headless once dev libs are installed.** `apt-get install -y
  libwayland-dev libasound2-dev libudev-dev libxkbcommon-dev` lets
  `crabomination_client` (Bevy) compile + clippy + `--no-run` its tests in the
  remote/headless env (the wayland-sys/alsa-sys/libudev/xkbcommon build scripts
  just need the `.pc` files). Runtime/GPU verification still needs the local
  `verifier-client` skill. Keyword chips + tooltips for the new protection
  keywords are compile-checked here.
- ⏳ **recent2 card approximations** (`decks::recent2`, all noted in doc
  comments): March of Otherworldly Light drops the "exile white cards to reduce
  cost" rider; Conduit of Worlds ships only the play-lands-from-graveyard static
  (not the {T} cast-from-graveyard half); Lord Skitter's Rat-ETB exiles a card
  rather than "up to one target"; Llanowar Greenwidow drops the Domain cost
  reduction + the exile-if-it-would-leave rider. Newer wave: Sunfall's Incubate
  now ships (`Effect::Incubate`, CR 701.53); Ossification is modeled as a standalone O-Ring (no enchant-a-basic
  rider); Steamcore Scholar drops the "unless you discard an I/S or flyer"
  reprieve; Subterranean Schooner explores any creature you control (not
  specifically the one that crewed it); Gathering Throng searches up to three;
  Hexgold Slith drops the optional pay-{E}-for-first-strike attack ability.
- ⏳ **Noticed this run (recent2 MOM/WOE/OTJ wave):** real-card primitives still
  missing — a Value for "noncreature spells a player cast this
  turn" (Magebane Lizard); **Spree** multi-additional-cost casting (Phantom
  Interference, Three Steps Ahead); chosen-card-type cost reduction (Stenn);
  cast-from-an-opponent's-graveyard on combat damage (Tinybones). Warren
  Warleader needs a "create a tapped, attacking token" mint + a "whenever you
  attack" (declare-attackers) trigger distinct from `Attacks/SelfSource`.

- ⏳ **Haunt / Ripple / Unearth follow-ups** (shipped this push):
  - Haunt's haunted-creature is auto-picked (prefers an opponent's) and the
    exile-haunting is modeled as a `route_to_graveyard` replacement, not a real
    targeted stack trigger — add a controller choice + a proper trigger.
    Combat-damage haunt (Souls of the Faultless) is unmodeled.
  - Ripple's free-cast prompts go through `Decision::OptionalTrigger`; the
    "Spells you cast have ripple N" static (Thrumming Stone) isn't wired.
  - Unearth models only the end-step exile, not "exile it if it would leave the
    battlefield" (same gap as Goryo's). A client affordance to surface
    graveyard-activated abilities (the bot already offers them) is missing.
  - Card approximations: Surging Æther's "target spell or permanent" → creature;
    Surging Sentinels' protection-on-white-cast rider dropped.

- ✅ **Per-color mana-spent tracking** (CR 702.137 Adamant / CR 601). The cast
  path stamps `CardInstance.cast_mana_spent_by_color` (pool diff per color);
  `EffectContext.mana_spent_by_color` reads it at resolution.
  `Predicate::ManaSpentOfColorAtLeast` (Slaying Fire's adamant 4, Searing
  Barrage's controller burn — both upgraded from approximations) and
  `Predicate::CastSpellNoColoredManaSpent` (Void Mirror counters generic-paid
  and free casts). Tests `cr_702_137_*`, `cr_601_void_mirror_*`.
- ✅ **Multi-slot targets on triggered abilities.** `auto_extra_distinct_slot_targets`
  fills slots 1.. of a trigger whose effect surfaces a *distinct* per-slot
  filter (gated on slot-0 ≠ slot-1 so same-filter "up to N" divide effects keep
  their own behavior). `Effect::Attach` is now in `primary_target_filter` so
  slot 0 picks the Equipment. Cards: Kor Outfitter (ETB), Brass Squire ({T}).
  Tests in `tests/modern.rs`.
- ⏳ **Missing keyword mechanics:** Sunburst-on-noncreature charge counters (the
  +1/+1 creature path ships via `Value::ConvergedValue`). (Haunt ✅ —
  `Effect::HauntCreature`; Ripple ✅ — `shortcut::ripple`/`Effect::Ripple`.)
- ✅ **Auto-targeter maximizes "up to N" triggered abilities** (CR 115.1c) —
  `GameState::auto_extra_targets_for` fills slots 1.. with distinct legal picks
  for an `Effect::ApplyToTargets` on a triggered ability; wired into both the
  self-source ETB push (`actions.rs`) and the general trigger push
  (`push_pending_trigger`). Gavony Silversmith now buffs two creatures.

- ✅ **Mutate (CR 702.140).** Shipped: `CardDefinition.mutate: Option<ManaCost>`,
  `GameAction::CastMutate { card_id, target, on_top }`, `CardInstance.mutate_stack`
  (component cards top-to-bottom; live `definition` = union of the top card's
  characteristics + every card's abilities), `EventKind::Mutated` /
  `GameEvent::Mutated`, leave-the-battlefield scatter (all three meld sites), and
  snapshot round-trip (union rebuilt on load). Cycle: Glowstone Recluse,
  Trumpeting Gnarr, Cubwarden, Cavern Whisperer, Dirge Bat, Migratory Greathorn,
  Boneyard Lurker, Pollywog Symbiote (`HasMutate` filter), Vulpikeet, Majestic
  Auricorn, Sawtusk Demolisher, Gemrazer, Insatiable Hemophage, Chittering
  Harvester, Regal Leosaur, Cloudpiercer, Sea-Dasher Octopus, Essence Symbiote,
  Porcuparrot (`Value::MutateCount`), Archipelagore (`Effect::TapUpToValue` —
  dynamic-count resolution-time picker). Tests in `tests/modern.rs`. Follow-ups:
  - ⏳ **Client cast-mutate UI + `mutatable` affordance** (host picker). Engine
    path is fully wired and tested; only the UI is missing.

- ⏳ **Ikoria cards deferred (need new primitives or are complex):**
  - **IKO walkers still missing:** Narset of the Ancient Way (restricted-mana +1
    spendable only on noncreature spells + discard-linked damage; −6 emblem) and
    Lukka, Coppercoat Outcast (+1 exile-top-3 with conditional cast-from-exile;
    −2 reveal-until-greater-MV deploy). Vivien, Monsters' Advocate now ships
    (cast-from-top static, +1 token+keyword-counter, −2 lesser-MV tutor via the
    new next-spell `event_amount` wiring).
  - ✅ **Winota, Joiner of Forces** — `Effect::LookTopMayDeployAttacking`
    (look top six, deploy a Human creature tapped-and-attacking with
    indestructible EOT, bottom the rest; auto-picks highest power). Test
    `winota_deploys_human_when_nonhuman_attacks`. Remaining ⏳: a `wants_ui`
    picker (currently auto-pick) and the "up to one" decline.
  - ✅ **Memory Leak** — `Effect::ExileChosenFromHandOrGraveyard` (cross-zone
    exile of a nonland from the target's hand or graveyard; auto-picks highest
    MV) + Cycling {1}. Test `memory_leak_exiles_highest_mv_across_zones`.
    Remaining ⏳: a `wants_ui` chooser (currently auto-pick).
  - **Other complex IKO holdouts** (next-run candidates):
    Kinnan (tap-for-mana doubling + big-creature dig), Quartzwood's faithful
    "any trampler you control" batch trigger, Sea Serpent
    (can't-attack-unless-defender-has-Island + sac-if-no-Islands), Titans' Nest
    (surveil + restricted exile-for-mana).
  - **Brokkos, Apex of Forever** ships with mutate+trample; the "cast from
    graveyard using its mutate ability" rider is dropped — `cast_mutate` only
    reads the hand. A `mutate_from_graveyard` flag + a graveyard cast path
    (mirroring `cast_escape`) would finish it.
  - **Glimpse the Cosmos** ships the dig-3-take-1; the "cast from graveyard
    while you control a Giant" rider is dropped — needs a board-conditional
    graveyard-cast permission (conditional flashback) primitive.
  - Client keyword label/tooltip arms for `ProtectionFromManaValueParity` are
    compile-verified — `crabomination_client` now builds headless via the
    pkg-config + linker shim recipe above (rustc 1.95, `LIBRARY_PATH=/tmp/pc`).
    Runtime/GPU verification still needs the local `verifier-client` skill.
  - **Approximations shipped this run** (dropped riders, all noted in the card
    doc comments): Gust of Wind / Tentative Connection / Mythos of Brokkos's
    "spent {X}{Y}" upgrades (no mana-provenance-by-color spend-tracking yet);
    Mythos of Nethroi's {G}{W} upgrade; Parcelbeast's "you may" on the land.

- ✅ **Catalog-wide stat sweep (2026-06-16) — same problem beyond STX.** The
  modern supplement (`decks/`, `mod_set/`) and small older sets carried the same
  synthesized-stat drift. New tooling `scripts/audit_catalog_stats.py` (cost +
  P/T + creature-type + keyword, all sets) and `scripts/fix_catalog_stats.py`
  (cost/P-T/type fixer with a custom-card exclude list) drove a sweep across
  `decks`/`mod_set`/`ths`/`kld`/`ktk`/`lea`/`dis`/`khm`/`sos`, regenerating coupled
  tests via `fix_test_mana.py` + `regen_test_assertions.py`. Catalog-wide drift:
  **cost 253→2, P/T 131→6, type 120→8, keyword 55→41** (full suite green, 8551).
  Lessons baked into the tooling: cost rebuilds use the *front* face (don't sum
  split halves), the cost field is found as the depth-1 `CardDefinition.cost`
  (never a nested `mana_cost:`/deferred-Pact/`GrantMiracle` cost), and the keyword
  audit reads only the top-level vec. **Keyword pass:** 13 clear simple-keyword
  bugs fixed (spurious/wrong/missing — e.g. Shriekmaw Menace→Fear, Mockingbird
  Flash→Flying, Loot +Double-strike/Haste); the other ~41 are deliberately left —
  conditional keywords modeled as base (Paradise Druid's untapped-only hexproof),
  keywords that *model an evasion ability* (Silhana/Signal Pest "blocked only
  by…" as Flying, Reality Smasher/Frost Titan counter-tax as Ward), DFC
  back-face keywords, Protection/Ward that need a quality/arg, and manlands
  (Mutavault). Those need real ability modeling, not a stat tweak. **Other
  remaining:** cost/PT/type leftovers are CDA P/T, the 3 synergy-coupled
  synthesized types, missing enum variants, and the 2 excluded customs (Cosmogoyf,
  Crabomination). Run `python3 scripts/audit_catalog_stats.py` for the live table.
- ⚠️ **Fabricated real-name STX cards (correctness sweep).** Many STX factories
  reuse *real* STX card names but carry invented cost/types/oracle text (the
  synthesizer collided with real names). **Cost + P/T are now fully swept**:
  `scripts/audit_stx_drift.py` reports 0 cost/PT drift across the whole `stx/`
  tree (148 mana-cost literals + 61 power/toughness literals corrected to the
  Scryfall cache this run, doc-comment titles synced via
  `scripts/fix_doc_costs.py`, coupled test fixtures rewritten via
  `scripts/fix_test_mana.py`). Re-run `python3 scripts/audit_stx_drift.py` to
  keep it at zero after adding cards.
  ✅ **Type-line + keyword sweep (2026-06-14/15).** `audit_stx_drift.py` only
  checks cost + P/T; it never inspects type line or keywords. Added
  `scripts/audit_stx_types.py` to cover those (top-level keyword field only, so
  it skips conditional/granted keywords nested in statics/equip-bonuses/tokens).
  Against a freshly-refetched real Scryfall cache it found **49 creature-type +
  ~15 real keyword drifts**. Fixed: **47 creature types** + **20 keywords**
  (Mavinda Cleric+Vigilance → Bird Advisor+Flying; Beledros Demon+Trample/Lifelink
  → Elder Dragon+Flying; Galazeth → Elder Dragon; Disciplined Duelist FirstStrike
  → DoubleStrike; Codespell Cleric → Vigilance; Spectacle Mage Prowess → Flying;
  Inkfathom Witch → Fear; Inkfathom Divers → Islandwalk; Lone Rider → First
  strike+Lifelink; etc. — two coupled tests in `tests/stx/part_25.rs` updated,
  one Intimidate test in `tests/modern.rs` given a Reach blocker). Full suite
  green (8551). Audits now clean except:
  - **3 creature types** — Eyetwitch, Quandrix Pledgemage, Silverquill Pledgemage
    are synthesized cards whose Pest/Fractal/Inkling *synergy tests* depend on the
    wrong type (retyping breaks the tests; needs card + test reworked together).
    (Eccentric Apprentice fixed — added `CreatureType::Tiefling`.)
  - **1 keyword** (Lone Rider) — a benign DFC artifact: the modeled front face is
    correct (First strike+Lifelink); the flagged Trample is the *back* face only.
  Note: several conditional/granted keywords were left as the catalog already
  models them correctly via statics (Leech Fanatic's your-turn lifelink, Sticky
  Fingers' aura-granted menace, Silverquill Pledgemage's magecraft flying) — the
  audit no longer flags those. Many fixed cards are fabricated-real-name
  collisions whose **bodies are still synthesized**; a correct stat block ≠ a
  faithful card.
  **Effect-body sweep complete**: Hofri Ghostforge, Fervent Mastery, and
  Strixhaven Stadium (point counters + ten-point `Effect::LoseGame`) are now
  faithful. ✅ this run: **Stonebinder's Familiar**
  (`EventKind::CardExiled` once-per-turn-during-your-turn trigger, retyped Spirit
  Dog), **Confront the Past** (faithful 2-mode: reanimate gy PW + remove 2X
  loyalty from an opp PW — the "MV X or less" reanimation gate is dropped, no
  X-aware MV target filter yet). Per card:
  replace the body with the Scryfall text and rewrite its test(s); watch for
  fixture coupling. Swept faithful this run: **Mage Duel** (+1/+2 then fight),
  **Tempted by the Oriq** (permanent MV≤3 steal), **Mentor's Guidance**
  (conditional copy-on-cast + scry/draw), **Bayou Groff** (Plant Dog 5/4 +
  sacrifice-a-creature additional cost; pay-{3} alternative dropped). Confirmed
  already-faithful (stale notes): Frost Trickster (Bird Wizard, ETB tap+stun),
  Eager First-Year (magecraft self-pump), Owlin Shieldmage (Flying + Ward 3
  life), Promising Duskmage (death-draw if +1/+1 counter).
  Bayou Groff is now faithful — `AdditionalCastCost::SacrificeOrPay`
  auto-sacrifices when a match exists, else folds the pay into the cost.
- ✅ **Remaining real STX (Strixhaven 2021) cards — complete.** A Scryfall
  `set:stx` diff vs the registered catalog now shows 0 unimplemented
  non-Arena cards (the last 13 — Deans, Culling Ritual, Professor Onyx,
  Zimone, … — existed but weren't registered; the crate-wide generated
  factory list closed that, and Zimone's fabricated body was rewritten
  faithful). Historical note: this run previously added the
  single-faced **Efreet Flamepainter** (`CastWithoutPayingImmediate` from gy on
  combat damage), **Thunderous Orator** (conditional keyword-share via
  `If` + `Predicate::SelectorExists`), **Venerable Warsinger** (combat-damage
  reanimation, MV gate fixed at 3), and **Ardent Dustspeaker** (impulse-draw
  two on attack; the gy-to-bottom enabler dropped). Still unimplemented,
  grouped by the primitive they're blocked on:
  - **Study / hone counters** — Kianne/Imbraham, Uvilda/Nassari Deans.
  - ✅ **Entered-this-turn filter** (`SelectionRequirement::EnteredThisTurn`,
    `CardInstance.entered_turn` stamped at every ETB via the dispatcher) —
    ships **Shaile // Embrose**, the first Dean MDFC. **First Day of Class** is
    also done (its own turn-scoped `Effect::CreaturesYouControlEnteringThisTurn`
    delayed trigger, CR 603.4).
  - **MDFC legends** — Codie/Extus/Blex/Jadzi + the rest of the Dean cycle.
  - ✅ **Group land-search** — `Effect::CatchUpBasicLands` (each player behind
    the land leader fetches basics up to the deficit, tapped, then shuffles).
    Ships Scholarship Sponsor.
  - **Variable-number-of-targets** — Ecological Appreciation ("up to four with
    different names" + opponent-chooses-two split).
  - ✅ **Draconic Intervention** — shipped via new
    `AdditionalCastCost::ExileFromGraveyard { filter }` (exiles a gy card, its MV
    becomes the spell's X) + `ExileIfWouldDieThisTurn` for the "exile instead"
    rider.
  - **Single-faced, still blocked**: Codie (can't-cast-permanents static +
    when-you-next-cast reflexive discover — needs a new delayed-trigger kind).
    ✅ Elite Spellbinder (`Effect::ExileFromHandTaxed` — exile a nonland from an
    opp's hand; owner may play it for +{2} while exiled; cost bug {1}{W}{B} →
    {1}{W}{W} fixed). Radiant Scrollwielder already ✅.
  Diff `set:stx` Scryfall names against the catalog string literals (note:
  helper-built names like the Snarl cycle are passed as `name` params, so
  grep the whole file, not just `name: "…"`).
- ✅ **Variable-X loyalty abilities** (CR 606.5) — `LoyaltyAbility.x_cost: bool`
  (Default-derived; literals migrated). `ActivateLoyaltyAbility { x_value }`
  threads the chosen X; `activate_loyalty_ability` clamps X to current loyalty,
  spends X, and stacks the effect with `x_value: X` so the body reads
  `Value::XFromCost`. Kasmina's -X Fractal is now faithful. Remaining ⏳: a
  `Decision::ChooseAmount` UI prompt for X (the bot commits full loyalty; the
  client doesn't yet build the loyalty action). Sorin/Saheeli -X ultimates can
  now reuse the same `x_cost` path.
- ✅ **`Effect::PayManaOrElse { mana_cost, otherwise }`** (this run) —
  the mana sibling of `PayEnergyOrElse`; pays from the floating pool when
  able, else runs the fallback (Archway Commons' "sacrifice unless pay
  {1}"). Remaining ⏳: a `wants_ui`/bot mid-resolution pay prompt (today a
  bot with no floating mana always takes the fallback, same limitation as
  `MayPay`).

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
  - ✅ **Rhystic "draw unless they pay X" rider** — Esper Sentinel ships
    faithful (`WardCost::GenericSourcePower` + `once_per_turn` first-spell
    gate, exact in 2P); Mystic Remora already rode `UnlessPlayerPays`.
  - ✅ **Power-gated keyword anthems** — `AffectedPermanents::
    CardMatchPowerGated` + a two-pass `compute_permanent` (CR 613.8);
    Temur Ascendancy's haste gate is faithful.
  - ✅ **"with no counters on it" target filter** — added
    `SelectionRequirement::HasNoCounters`; ships Heartless Act (modal:
    destroy a no-counter creature / remove-all counters).
  - ✅ **Typecycling / Landcycling** (CR 702.29e) — `Keyword::Landcycling`
    + `GameAction::Landcycle`; ships Wirewood/Daru/Shoreline/Twisted
    Abomination/Skirk. UI multi-match pick ✅.
  - ✅ **"Discard unless they discard an artifact" conditional discard** —
    `Effect::DiscardUnlessKind` (auto-keeps the lowest-MV matching card);
    Wrench Mind is faithful.
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
  the redistribute-life-totals clause (CR 119.7, last sentence) still wants a
  `Effect::DistributeLifeTotals` check. **Exchange-life-totals already respects
  the lock** — `Effect::ExchangeLifeTotals` routes through `adjust_life_applied`,
  so the gaining half is dropped for a can't-gain player (test
  `cr_119_7_exchange_life_totals_respects_cant_gain_life`). (c) Tainted Remedy's
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

- ✅ **CR 602 — Activating Activated Abilities** (push
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
  Push claude/modern_decks: `CardDefinition.discard_activated`
  (`GameAction::ActivateDiscardAbility`) models a "[cost], Discard this card:
  [effect]" from-hand activated ability — the discard-self *is* the cost, the
  card leaves hand before the (targetless) effect resolves. Magma Opus's
  {U/R}{U/R} Treasure mode rides it; surfaced in `PlayerView.
  discard_activatable_hand`, offered by the bot as a fallback value play.
  (b) **602.1b** — ✅ (`ActivatedAbility.condition` covers per-ability
  predicate gates ("Activate only if …"); `once_per_turn` /
  `sorcery_speed` / `from_graveyard` cover the canonical instructions.
  Per-opponent control restrictions ("Activate only if a player
  controls a Snow permanent") have no first-class slot but can be
  expressed as `condition: Predicate::…` for most. Conformance:
  `cr_602_1b_sorcery_speed_activation_needs_an_empty_stack` activates
  Ghost-Lit Stalker's printed sorcery-speed line and asserts both the
  rejection under a spell and the resolution once the stack clears.).
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

- ✅ **CR 603.4 — Intervening 'if' clause** — both halves wired. Trigger-time
  check drops triggers whose `event.filter` predicate is false; the survivors
  carry the predicate on `StackItem::Trigger.intervening_if`, re-checked in the
  resolver (`resolve_stack_item`) so a trigger fizzles when the condition is no
  longer true at resolution. Tested by `tests/sos.rs` (Cuboid Colony's
  mana-spent re-check).

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
  Update (was stale): the {0} graveyard-cast ability *is* wired
  (`silverquill.rs::mavinda_students_advocate`) — but as a {0}
  once-per-turn **activated** ability, not the printed static, and the
  "targets only a single creature" sub-filter is dropped (any IS card
  in your graveyard is eligible). The body is 2/3. (Its creature type and
  keywords were wrong — Human Cleric + Vigilance — and were corrected to
  Bird Advisor + Flying in the 2026-06-14 type/keyword sweep; see "Fabricated
  real-name STX cards".)

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

- ✅ **Triggered mana ability fast-path (CR 605.1b)** —
  `StaticEffect::ExtraManaOnLandTap` resolves stack-free right after the
  tapping ability (Mana Flare / Vernal Bloom / Wild Growth / Utopia
  Sprawl); Mana Reflection's doubling already rode
  `mana_production_doublers`.

- ✅ **CR 122.2-strict counter clearing on zone change** — counters
  clear at every zone-change funnel (`place_card_in_dest` all arms,
  `send_to_graveyard`, `route_to_graveyard`). Dies-with-counters
  readers use LKI instead: filter evaluation prefers
  `died_card_snapshots` (Felisa's WithCounter), and resolution-time
  `Value::CountersOn` consults `leaves_bf_lki` under
  `resolving_lki_source` (Ambitious Augmenter's transfer). Pinned by
  `cr_122_2_counters_cease_to_exist_on_zone_change` (now strict).

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

### Client build CAN be verified in the web sandbox (pkg-config + linker shim)
`crabomination_client` links Bevy, whose `wayland-sys`/`alsa-sys`/`libudev-sys`
build scripts call `pkg-config`, and the linker then wants `.so` dev symlinks —
the runtime `.so.N` files exist here but the `.pc` files and `.so` symlinks
don't. Since the Bevy 0.18→0.19 bump the toolchain floor is **rustc 1.95**
(`rustup toolchain install 1.95.0 && rustup override set 1.95.0`). Drop shims +
symlinks in a temp dir and point both `PKG_CONFIG_PATH` and `LIBRARY_PATH` at
it (pkgconf 1.8 rejects a `.pc` with no `Description:` field, and 0.19 added
the `libudev` dep):
```sh
mkdir -p /tmp/pc && cd /tmp/pc
for m in client cursor egl; do printf 'libdir=/usr/lib/x86_64-linux-gnu\nName: wayland-%s\nDescription: shim\nVersion: 1.22.0\nLibs: -L${libdir} -lwayland-%s\nCflags:\n' $m $m > wayland-$m.pc; done
printf 'libdir=/usr/lib/x86_64-linux-gnu\nName: alsa\nDescription: shim\nVersion: 1.2.0\nLibs: -L${libdir} -lasound\nCflags:\n' > alsa.pc
printf 'libdir=/usr/lib/x86_64-linux-gnu\nName: libudev\nDescription: shim\nVersion: 250\nLibs: -L${libdir} -ludev\nCflags:\n' > libudev.pc
ln -sf /usr/lib/x86_64-linux-gnu/libwayland-client.so.0 libwayland-client.so
ln -sf /usr/lib/x86_64-linux-gnu/libwayland-cursor.so.0 libwayland-cursor.so
ln -sf /usr/lib/x86_64-linux-gnu/libwayland-egl.so.1 libwayland-egl.so
ln -sf /usr/lib/x86_64-linux-gnu/libudev.so.1 libudev.so
ln -sf /usr/lib/x86_64-linux-gnu/libasound.so.2 libasound.so
PKG_CONFIG_PATH=/tmp/pc LIBRARY_PATH=/tmp/pc cargo clippy -p crabomination_client
```
Runtime/GPU verification (opening a window) still needs the local
`verifier-client` skill — only compile-checking works headless.

### Damage-as-(-1/-1)-counters replacement
Soul-Scar Mage / Phyrexian Vatmother-style "if a source you control would
deal noncombat damage to a creature, it deals that much in -1/-1 counters
instead" needs a damage-replacement hook. Soul-Scar Mage ships as 1/2 Prowess
without it. (Native Infect/Wither on the non-combat funnel shipped —
`deal_damage_to_from` lands -1/-1 counters / poison; CR 702.80a/702.90e.)

### Phyrexian mana
Mutagenic Growth ({G/P}), Gut Shot, Dismember, etc. — a mana symbol payable
with 2 life. Mutagenic Growth ships at the {G} cost (the life-pay alt is
omitted).

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
✅ Shipped as `ActivatedAbility.remove_counter_cost` (Walking Ballista's
`Remove a +1/+1 counter: deal 1`, Barkhide Troll's hexproof pump).
Experiment One's `Remove two: Regenerate` still pending a per-card pass.

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

### Prepare Mechanic (SOS)

The June 2026 rework replaced the incorrect MDFC model with the printed
mechanic (`prepare_spell` + `CastPrepareSpell`; see `.claude/prepared.md`).
All 36 preparation cards audited against Scryfall oracle. Residual
approximations (each documented at the card site):

| Card / Feature | Current Approximation | Correct Behaviour |
|---|---|---|
| Copy-in-exile object | The spell copy materializes at cast time | A copy sits in exile while the creature is prepared, so "cast from exile" zone-watch triggers should see it |
| Bot play | Bot casts prepare spells: main-phase candidates scored by the inset spell, instant insets fired in response to removal on the prepared body (`pick_prepare_response`), off-card re-prepare abilities used as a mana sink (`pick_reprepare`), the counter priced into `permanent_value`, and X-cost inset spells sized like hand casts (`max_affordable_x_for_def`) | — |
| Emeritus of Truce ETB | Inkling token minted for *you* | "Target player creates…" (needs a trigger-scoped target slot for `CreateToken`) |
| Harmonized Trio | Activation taps one other untapped creature | "Tap two untapped creatures you control" |
| Scrollboost | Single target +2/+2 | "One or two target creatures" |
| Secret Rendezvous | Each player draws 3 (equivalent in 1v1) | "You and target opponent" |
| Striking Palette | Armed window consumed by the next spell of any type (copy still gated to instant/sorcery) | "When you next cast an instant or sorcery spell this turn, copy it" |
| Bind to Life | Mill 7, then return a creature from the graveyard | "…from among the milled cards" (needs a scratch selector) |
| Oracle's Gift | Counters land on the freshly-minted batch only | "…on each Fractal you control" (pre-existing Fractals miss out) |
| Swords to Plowshares rider | Lifegain approximated off the target's controller-as-resolved | Printed: target's controller gains the life — verify the `PlayerRef::ControllerOf` shape is exact |

---

## Engine — Rollback / Undo system (plan)

Two deliverables share one mechanism: (a) **transactional action
application** inside the engine — every rejected `GameAction` restores the
exact pre-action state, structurally killing the audit-P0 partial-mutation
family (Squad/Casualty under-pay, `declare_attackers` mid-loop corruption,
back-face land corruption, madness mana loss); (b) **player-facing
undo/take-back** — instant in single-player vs the bot (the main UX win),
consent-gated in multiplayer. The same checkpoint recorder later feeds the
replay scrubber (Client UX Tier 3) and crash recovery.

**Approach: whole-state snapshots, not inverse commands.** `GameState` has
a hand-written `Clone` (`game/mod.rs:859`) and full serde; the affordance
prober and bot dry-runs already clone the state per candidate action, so
the cost profile is known-acceptable. Inverse ops for a ~9k-line effect
resolver would be unmaintainable and would inherit every funnel-bypass bug
the audit found.

### Phase 0 — prerequisites
- ⏳ **Seeded, serialized RNG.** Shuffles call thread-local `rand::rng()`
  inline (`game/mod.rs:2462`, `4495`, `5968`, `7239`; grep for stragglers).
  Add `GameState.rng` (e.g. `Pcg64`, serde via seed+stream state) and route
  every random site through it — otherwise undo lets a player re-roll
  shuffles/flips until they like the outcome, and bit-exact replay is
  impossible. Fold in the audit-P1 coin-flip fix (`Decision::CoinFlip`
  must draw from this RNG, not constant heads) while touching it.
- ⏳ **Serde fidelity.** Not needed for in-memory undo (which uses `Clone`),
  but required before any persisted history/replay: fix the audit-P1
  `CardInstanceWire` six dropped fields + `TokenDefinition.static_abilities`,
  and land the property-based round-trip test (see Infrastructure →
  Snapshot Round-Trip Test).

### Phase 1 — transactional `perform_action` — ✅ DONE (2026-07)
- ✅ Checkpoint at the top of `perform_action`, restore on `Err` — for
  EVERY action, not just human-submitted ones: `GameState`'s heavy zones
  are now `CowBox`-wrapped (`crate::cow` — Arc + make_mut copy-on-write),
  so the checkpoint costs reference bumps and a failing action only pays
  for zones it touched before erroring. Affordance probes skip the
  checkpoint via `perform_action_inner` (their state is discarded either
  way). Regression: `cow::tests::rejected_action_restores_state_exactly`.
- ✅ Suspension is not failure: restore happens only on `Err`, and
  `GameError::ManualTapRequired` is exempted — it deliberately leaves
  forced pips auto-tapped + mana floating for the client's pending-cast
  driver (pinned by `sos::mana_shapes` tests). The restore keeps the
  *live* decider (the checkpoint clone holds a blank one; swapping it in
  would wipe a `ScriptedDecider` mid-script).
- Per-call semantics: a failed resume restores to the *suspended* state,
  not to before the original action. Full multi-step atomicity across a
  suspend/resume chain remains future work if ever needed.
- Keep the targeted P0 fixes anyway (validate-before-mutate is still
  better); the transaction is the backstop that makes the *class*
  unexploitable.

### Phase 2 — engine history ring
- ⏳ `UndoHistory { ring: VecDeque<(UndoPoint, Box<GameState>)> }` on the
  server-side game session (not inside `GameState` — snapshots must not
  contain the history). Push at decision boundaries: before each accepted
  human `GameAction` and before each `Decision` answer. `UndoPoint` carries
  seat + monotonic id + a human label ("cast Lightning Bolt", "declared
  blockers") for the UI.
- ⏳ Cap (e.g. 32 entries) and measure real `GameState` sizes; if memory
  matters, serialize+compress entries older than the last few.

### Phase 3 — server protocol + consent
- ⏳ Wire actions: `RequestUndo { to: UndoPointId }` /
  `RespondUndo { accept }` + a pending-request broadcast. On accept:
  swap in the snapshot, bump a view generation, re-broadcast full per-seat
  views (the existing per-seat projection path is the resync mechanism).
- ⏳ Policy: single-player undo is unconditional and instant. Multiplayer
  requires every opponent's consent. Bot policy: auto-accept (configurable
  later). Optionally restrict to "within the current priority window /
  before new hidden information was revealed" as a server setting.
- **Hidden-information stance (documented, not solved):** information a
  player already saw stays seen (the casual-play standard). The Phase-0
  seeded RNG guarantees a restored pre-shuffle state re-shuffles
  identically, so undo cannot be used to fish randomness; it *can* still
  be used to act on glimpsed information — consent is the mitigation.

### Phase 4 — client UX
- ⏳ Undo button + keybind, greyed when no eligible `UndoPoint`; opponent
  banner with accept/decline; game-log entry ("Eric took back: cast …").
  Supersedes the bare "Undo / Take-Back" stub under Client — UX.

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
✅ Shipped — `V` toggles a browser listing exiled cards with per-card
source annotations (linked exile, cipher, foretell, …).

### Stun Counter Visualization
Static Prison and Rapier Wit add stun counters.  No indicator currently shows
that a permanent has a stun counter (i.e., won't untap next turn).  A small
badge or coloured ring on the card would communicate this clearly.

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
numerals. Once the glyph primitive exists every mana surface benefits
(the pip-style mana-pool HUD already ships in `player_stats.rs`).

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

### UI backlog — competitor-parity sweep (2026-06-11)

Prioritized ideas from a parity review against Arena / MTGO / Cockatrice /
XMage, after the decision-coverage + stops + import session shipped.
Cross-references the detailed entries below where one exists.

**In-game, high impact**
- ✅ **Stack as a visual zone** — `update_stack_panel` now renders
  card-art tiles: the top item gets a large gold-framed tile with a
  "resolves next" line, the rest smaller thumbnail rows, each with a
  controller-colored edge strip (green = yours / orange = opponent's);
  the footer offers a "Let resolve ▶" button while the viewer holds
  priority (else "Waiting for <name>…"). Hidden items show the cardback.
  Remaining ⏳: hover a tile → large preview, click → scroll the log.
- ⏳ **Undo / mana-tap rollback** — see "Engine — Rollback / Undo system
  (plan)"; the minimal client slice (un-tap floated mana before a cast
  commits) is worth shipping ahead of the full plan.
- 🟡 **Battlefield organization at scale** — ✅ identical tokens cascade
  into piles with a ×N count chip (`creature_card_transform` +
  `token_badge.rs`); same-name lands already stacked. Remaining ⏳: a
  visible aura/equipment → host link (today attachment info lives only in
  tooltips).
- ✅ **Attention pings** — pulsing cyan ring on the pending decision's
  source permanent (`draw_decision_source_ring`), low-life screen-edge
  vignette (≤5 life), Pass-button urgency pulse already existed.
- 🟡 **Cost-payment feedback** — ✅ the manual-tap banner live-updates
  with the remaining cost ("{1}{U} to go") as sources tap. Remaining ⏳:
  pre-highlighting which sources auto-tap would take.

**Quick wins**
- ✅ **Persist settings** — `config::ConfigStore` holds the live whole
  config; `persist_stops` / `persist_animation_speed` /
  `persist_player_name` mirror changes into `config.toml`
  (`gameplay.{stops_my,stops_opp,animation_speed,player_name}`), and
  startup seeds `StopConfig`, `AnimationSpeed`, and the menu name from it.
- 🟡 **Clickable game log** — ✅ hovering a log line that names a card
  previews it (`ui_card_hover`, also wired onto stack-panel tiles).
  Remaining ⏳: click → flash the permanent on the board.
- ⏳ **Finish the hover oracle panel** — `ui::hover_info_lines` shows type
  line + keyword reminders; add triggered/activated-ability short text
  (see "X-ray card inspector").
- ✅ **Game-over stats** — `systems/match_stats.rs` tracks turns and
  per-seat draws / spells / damage taken from the wire events; shown on the
  game-over modal.

**Bigger projects**
- 🟡 **Settings screen** — ✅ main-menu Settings panel
  (`systems/settings_menu.rs`): window mode (windowed / borderless),
  resolution presets, maximize-on-launch, render quality, animation
  speed, hand sorting — applied live and persisted. Remaining ⏳:
  keybind remapping, audio (once there is audio).
- ⏳ **Deck library** — save imported decks, list them in the menu, pick
  the opponent's deck, paste-from-clipboard import.
- ⏳ **Bo3 + sideboarding UI** — Learn/Lessons sideboard plumbing exists
  engine-side.
- ⏳ **Replay viewer** — see "Replay scrubber" (Tier 3 below).
- ⏳ **Accessibility pass** — colorblind-safe target rings (shape, not
  only color), text scaling, reduced-motion toggle, finish keyboard-only
  play; see "Theme variants".

### Conspire cast UI (follow-up)

Conspirable hand cards now highlight as alt-castable (`ClientView.
conspirable_hand`, surfaced via the legal-play chain in `systems/ui.rs`).
Remaining: a creature-picker flow to actually submit `CastSpellConspire`
(choose exactly two untapped creatures sharing a color, like the
sacrifice/convoke pickers) — until then the client can only cast such cards
without the conspire copy. Engine + affordance + server view all ship.

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
- Stop settings + auto-pass ✅ — per-step Auto/Stop/Skip overrides on
  the clickable phase chart (`systems/phase_bar.rs::StopConfig`), wired
  into `auto_advance_p0`; right-click = pass-until-step. Remaining:
  persistence via `config.rs` (in progress).
- Phase bar ✅ — kept the vertical chart (click = stop toggle,
  right-click = pass-until) rather than a horizontal strip; revisit the
  horizontal layout only if the left edge gets crowded.
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
from misclicks, especially during the targeting flow. **Full plan now lives at
"Engine — Rollback / Undo system (plan)"** (snapshot-based, four phases;
Phase 4 is this UI).

### Responsive Stack Display
The stack panel (bottom-center) is a fixed-width overlay.  On narrow windows
it can overlap the player panel.  Clamp its width to `min(420px, 40vw)` or
reposition it to the right sidebar.

### Per-Phase Auto-Stop Flags
✅ Shipped as click-to-cycle Auto/Stop/Skip on the phase chart, scoped to
your turns vs opponents' (`systems/phase_bar.rs`); right-click a step =
pass-until. Remaining: persist the configuration (see backlog above).

### Deck Browser
A pre-game or in-game panel listing the full deck composition (name + count
for each unique card) would help players understand the randomly-assembled cube
deck they are playing.

### Game Log Scrollback + Event Color-Coding
✅ Shipped: 200-entry scrollback, per-variant colors, color-blind glyphs,
turn dividers, ×N coalescing, player names. Remaining ⏳: clickable log
lines (hover-preview the named card — see backlog above) and event
filtering.

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
~~The bot never responds to spells on the stack.~~ `pick_stack_response`
now counters an opponent's spell when it targets the bot's permanents /
the bot, or costs 3+ — cheapest affordable counter first, `would_accept`
dry-run as the final gate (so Spell Snare's MV filter etc. are honored).
Future: respond with removal/protection instants, not just counters;
race-aware "is this worth a card" valuation.

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

Every row in this table has shipped (Bloodtithe Harvester's sac-a-Blood
ping, Dread Return's flashback sacrifice, Balefire Dragon's power-scaled
sweep, and Karn, Scion of Urza's real text included — earlier ⏳ marks
were stale). See git history for the per-card details.

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

## Decision-plumbing audit (2026-07): bare `decider.decide` sites

A sweep for the "Ad Nauseam pattern" (fixed 2026-07: a mid-resolution
choice consulted `self.decider` directly, so AutoDecider's blanket
defaults answered for every seat — bots AND wants_ui humans). ~125
direct `decide` call sites audited across `effects/mod.rs`,
`effects/movement.rs`, `combat.rs`, `stack.rs`, `game/mod.rs`,
`actions.rs`. ~45 are live bugs, in five classes. AutoDecider defaults
for reference: OptionalTrigger→no, ChooseAmount→0, ChooseCards→first
`min` (empty when min=0, the "up to N" case), ChooseColor→first legal
(≈ always White), ChooseMode→0.

**Class 1 — whole keywords dead for every seat** (bare OptionalTrigger,
auto-declined): Madness (`mod.rs:8510`, ~17 cards), Dredge
(`mod.rs:9022`, ~15 cards), Cascade (`effects/mod.rs:17508`), Ripple
(17587), Cipher (18311), Forage (17933), Collect Evidence (17775/17797
AND 17852/17867 — both the wants_ui and bot branches are broken),
Discover's free-cast half (17650), CastFromHandWithoutPaying (18150),
CastWithoutPayingImmediate (17995 — kills SOS Improvisation Capstone),
CastAnyOrderWithoutPaying (18098), CastFreeParadigmCopy (18259),
Obzedat-style exile-blink (5807), Amped Raptor energy-cast (4065 —
worse than no-op: exiles the top card, then never casts it).
**Possibility Storm (15340) is actively destructive**: the original
spell is gone and the dug card stays in exile.

**Class 2 — "choose up to N" resolves as zero** (ChooseCards min=0):
Command the Dreadhorde (6640), three reanimation piles (6560, 6596,
6793), tutor-to-total-MV (6689), tap-any-number pump (6370),
Archipelagore tap (6968), Aether Vial-style PutFromHandOntoBattlefield
(10884), DeployCreatureFromHandAttacking (10975), Fateseal (4931 —
Jace +2 is a no-op), mill-then-take (4486), dig-to-hand (4965 — still
pays the self-mill, takes nothing), MayExileFromYourGraveyard rider
(5968), graveyard-exile hate (5924, 19899), SearchSplitOpponentChooses
(11629).

**Class 3 — amount defaults to 0**: ChooseNumberDestroyByPower (5580)
— **destroys every creature including the controller's own board**
(worst single finding; Expel the Interlopers); MayPayGenericUpTo (2607
— Wildborn Preserver never pumps); Sanctum Prelate locks 0 (16317);
Read Ahead sagas always start at chapter I (stack.rs:770).

**Class 4 — inverted wants_ui gates**: the human branch calls `decide`
synchronously (no suspension) while the bot branch has a real
heuristic — interactive seats play WORSE than bots:
SacrificeSourceUnlessSacrifice (10656 — a human's Gitrog dies every
upkeep, a bot's survives), ReturnGraveyardCardsToHand (6832),
ShuffleGraveyardCardsIntoLibrary (6879), PlayerReturnsPermanentsToHand
(10571), DistributeCountersAmongLastCreated (13534), PayAnyEnergy
(3741 — polarity fully reversed: bots pay all, humans pay zero),
CollectEvidence (see class 1). Also stack.rs:67's modal-trigger gate
skips suspension whenever ANY mode requires a target.

**Class 5 — quality-of-play defaults** (playable but wrong):
ChooseColor → White everywhere it matters
(GrantProtectionFromChosenColor 8058 — Mother of Runes always names
white; extra-mana AnyColor actions.rs:1692; Oona 19945 — the intended
Blue fallback is unreachable); legend rule keeps the NEWEST copy
(stack.rs:2858 — sacrifices the aura'd/countered older copy); owner
tuck choices always pick bottom (movement.rs:1197, 1216); coin-flip
repeat loops always stop at one win (1893); `MoveChosen` (10520) has a
dead `up_to` ternary — both arms identical, so "up to N" is enforced
as "exactly N" for every seat.

**Bot-side mirror bugs** (`server/bot.rs`): un-introspectable
ask_seat_bool prompts fall into `optional_trigger_beneficial`'s
`.unwrap_or(true)` — blind YES to "Pay N life to deny…", "Accept the
tempting offer?" (always accepts opponents' offers), echo/cumulative
upkeep (pays forever), clash (always bottoms), tribute (always
counters). Root gap: the source lookup scans battlefield/graveyard/hand
but NOT the stack, so any resolving spell's self-costly MayDo gets
blanket-yes.

**STATUS (fixed on claude/modern_decks, 2026-07):** all five classes
plus the bot-side mirrors are addressed — suspensions (AmountAnswerPending,
new CardsAnswerPending + ask_seat_cards/choose_up_to_cards, new
MayCastExiledPending completion), DeciderKind::Auto policies where
suspension is out of architectural reach, and bot prompt policies
(life-tax guard, tempting-offer decline, upkeep-value check, stack-zone
source lookup). ScriptedDecider always retains authority (suspension and
policies engage only for the live AutoDecider).

Deliberate remainders (policy-only or unchanged, each documented at the
site): Madness/Dredge interactive modals need resumable discard/draw
flows; Fiery Gambit's flip-again loop; Read Ahead's chapter pick
(ETB-time, no suspension reach); Amped Raptor's energy free-cast and
Ripple's chained offers (policy yes); per-token counter distribution
(even split for all seats); owner tuck choices and the AnyColor
extra-mana pick (smart defaults, no agency); legend-keep is a smart
default — the client's ChooseLegendToKeep modal still needs an engine
suspension to ever fire; single-stash constraint limits multi-ui-player
loops (EachPlayer shuffles) to one suspension per resolution.

## Simulation throughput

The recommender's dominant cost is `would_accept` dry-runs. Two
stepping stones landed (2026-07): the bot's per-tick candidate sweep
shares one library-stripped `affordance_probe_template` (a light clone
per probe instead of a full one), and the main castable block validates
*lazily* in descending score order at the pick site — a typical tick
probes 1-3 candidates instead of the whole hand. Match-template cloning
+ factory elimination in `simulate_match_games` was neutral — setup was
never the bottleneck. The big lever LANDED (2026-07): the heavy zones
(battlefield / stack / exile / per-player library / hand / graveyard /
command / sideboard / continuous_effects) are `CowBox`-wrapped
(`crate::cow`), so every `GameState::clone` — probes, probe templates,
`evaluate_action_outcome`, the `perform_action` transaction checkpoint —
is reference bumps plus only the zones the action actually mutates.
Remaining scaling comes from the racing schedule (`racing_rounds` +
small `games_per_pairing`) and, if ever needed, early adjudication of
stalled games via `eval_material`.

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
