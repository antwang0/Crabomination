# Feature Roadmap — MTGO / Arena / XMage parity

A prioritized capabilities roadmap (engine fidelity + UX + infra), derived from
a codebase analysis against the three reference clients. Per-card status lives
in `CUBE_FEATURES.md` / `DECK_FEATURES.md` / `STRIXHAVEN2.md`; the
approximations log is `TODO.md`.

Legend: ✅ done · 🟡 partial · ⏳ not started. Markers are a point-in-time read —
re-verify before picking up an item.

---

## Already shipped (don't re-propose)

A terse checklist. The exhaustive primitive-by-primitive list (and every card
exercising each) was elided in a compaction pass; recover it from
`git log -p -- FEATURE_ROADMAP.md`.

- **Multi-block (CR 509.1b / 509.2 / 510.1e):** `block_map` is blocker →
  `Vec<attacker>`; `Keyword::CanBlockAdditional(n)` / `CanBlockAnyNumber` cap
  the per-combat assignment; a creature blocking several attackers orders them
  and divides its combat damage (defending player decides, suspends for a UI
  seat). `PermanentView.blocking_attackers` is a Vec; the combat-math preview
  and the bot's block planner both follow; the client's damage order/assign
  modals are noun-aware (a blocker orders "attackers", not "blockers").
  Block triggers follow CR 509.3a–e (once per creature for the bare wordings,
  whole-partner-set for the per-object ones, count-gated for "N or more").
  Cards: Guardian of the Gateless, Knight of Sorrows, Valor Made Real,
  Lumbering Battlement, Lairwatch Giant. Tests in `core_rules/cr_recent35`.
- **Return to Ravnica block complete (RTR + GTC + DGM all at zero
  `set_gaps.py` gaps)** and **Theros (THS) complete**. The primitives those
  closures added: `Predicate::IsExtraTurn`,
  `StaticEffect::{UntapYoursEachUntapStepFiltered, GraveyardCardsUntargetable,
  CostReductionByValue}`, `CardDefinition.cast_condition`,
  `Effect::{TapAndLockWhileSourcePresent, TapBlockedByAndSkipUntap,
  DelayUntilWithCapture, ReturnSelfAttachedToTarget, ExileLinked,
  ChooseSector, SectorBlockLockThisTurn, SearchTheCityReturn}`,
  `Selector::{SacrificedCard, LastDamagerOf, CreaturesInChosenSector}`,
  `DelayedTriggerKind::NextCombat`, `Keyword::SpaceSculptor` +
  `CardInstance.sector` (CR 702.158 / 704.5u), and `DealDamageDivided.
  retaliate_to_source`. Engine bug fixed along the way: `fire_step_triggers`
  was never called for `TurnStep::EndCombat`, so every `DelayedKind::
  EndOfCombat` trigger silently never fired (CR 511.2).
- **RNA + GTC complete (both sets at zero `set_gaps.py` gaps):** Dovin Grand
  Arbiter, Gideon Champion of Justice, Teysa Karlov, Mass Manipulation,
  Lavinia Azorius Renegade, Mirror March, Amplifire, Lazav Dimir Mastermind,
  Illusionist's Bracers. New:
  `StaticEffect::{OpponentsCantCastNoncreatureAboveLandCount, PumpSelfByValue}`,
  `Effect::{FlipUntilLossThenTokenCopies, RevealUntilCreatureDoubleBasePt,
  CopyActivatedAbilityMayChooseTargets, ExileAnyNumberUntilSourceLeaves}`,
  `Value::CreaturesBlockedBy`, and a `StackItem::Trigger.activated` flag that
  distinguishes activated abilities from triggered ones on the stack.
- **THS batch (42 cards):** the three heroic Hoplites, the two-target pump
  instants, five Auras, six bestow creatures, the whole Ordeal cycle, and a
  second wave of commons/uncommons (Ephara's Warden, Fleshmad Steed,
  Blood-Toll Harpy, Benthic Giant, Crackling Triton, Boon of Erebos, Defend the
  Hearth, Lost in a Labyrinth, Dark Betrayal, Hunt the Hunter, Glare of Heresy,
  Lagonna-Band Elder, March of the Returned, Minotaur Skullcleaver, Fleetfeather
  Sandals, Flamecast Wheel, Decorated Griffin, Coastline Chimera — the first
  card to buy an extra block at instant speed — Breaching Hippocamp, Agent of
  Horizons). `EventScope::EnchantedBySource` now covers attack/tap events
  (it previously only handled deaths, exiles, and damage), and
  `Effect::prefers_friendly_target` judges a `Seq`/`If` by the children that
  actually declare a target — a non-targeting friendly prelude no longer aims a
  hostile payload at its own controller. Tests in `classic_sets/ths`.

- **RNA wave (modern_decks, this run, 100 cards):** Locket cycle, adapt/riot/
  spectacle/afterlife/addendum commons + uncommons, flash Auras, guildmages,
  the RNA split cycle (Depose/Consecrate/Warrant/Thrash/Collision/Carnival),
  wraths (Kaya's Wrath), Gate/Domri/Dovin payoffs, High Alert.
  Batch 9 (+25): Rix Maadi Reveler, Rafter Demon, Hackrobat, Gruul Spellbreaker,
  Smelt-Ward Ignus, Sphinx of New Prahv, Pestilent Spirit, Scuttlegator,
  Angelic Exaltation, Ethereal Absolution, Cry of the Carnarium, Pitiless
  Pontiff, Unbreakable Formation, Flames of the Raze-Boar, Swirling Torrent,
  Mesmerizing Benthid, Immolation Shaman, Screaming Shield, Clear the Stage,
  Domri's Nodorog, Bolrac-Clan Crusher, Dovin's Acuity, Dovin's Dismissal,
  Eyes Everywhere, Nikya of the Old Ways. New engine:
  `StaticEffect::TaxOpponentSpellsTargetingThis` (Sphinx of New Prahv) and
  `StaticEffect::ControllerCantCastNoncreatureSpells` (Nikya, a
  controller-scoped cast lock) and
  `StaticEffect::YourISSpellsHaveDeathtouch` (Pestilent Spirit — resolving I/S
  routes its damage through the deathtouch SBA via `resolving_spell_deathtouch_seat`,
  mirroring the lifelink-seat path). CR conformance in `core_rules/cr_recent33`
  (601.2f target tax, 702.2c spell-deathtouch, 508.1a defender-bypass).
  New engine: `EventKind::AdaptAbilityActivated` (CR 702.108c — adapt-ability
  activation event, `Effect::is_adapt` shape detector; finishes Gyre Engineer),
  `EquipBonus.remove_keywords` (CR 613.1f "loses flying" Auras — Sky Tether),
  a milled card now satisfies "put into a graveyard from anywhere" triggers
  (CR 701.15b — The Haunt of Hightower/Emrakul), and
  `StaticEffect::YourCreaturesCanAttackAsThoughNoDefender` (CR 508.1a team
  attack-despite-defender — High Alert, Assault Formation), wired through the
  bot planner and the client Attack-All panel. Bot now activates adapt
  abilities and weighs toughness-attackers by their real damage. Tests in
  `classic_sets/rna`, `core_rules/cr_recent31`, `core_rules/cr_recent32`.
  Batch 10 (+20, the batch-9 deferrals + rares): Angel of Grace, Rhythm of the
  Wild, Galloping Lizrog, Forbidding Spirit, Combine Guildmage, Verity Circle,
  Rumbling Ruin, Font of Agonies, Deputy of Detention, Prime Speaker Vannifar,
  Hydroid Krasis, Awaken the Erstwhile, Plaza of Harmony, Emergency Powers,
  Revival // Revenge, Rakdos the Showstopper, Ravager Wurm, Incubation Druid,
  Biomancer's Familiar, Kaya Orzhov Usurper. New engine primitives:
  `Effect::{DoubleP1P1CountersFromYourCreatures, TaxAttackersUntilYourNextTurn,
  CreaturesEnterWithExtraCounterThisTurn, OpponentWeakCreaturesCantBlockByYourCounters,
  EachPlayerDiscardsHandMakeTokens, CoinFlipEachCreatureDestroyOnTails}`,
  `CounterType::Blood` + `GameEvent`/`EventKind::PaidLife` (a "whenever you pay
  life" hook in `pay_life_cost`), `EventSpec::not_as_attacker` (the
  `PermanentTapped.as_attacker` flag — "becomes tapped, not as an attacker"),
  `StaticEffect::YourCreatureActivatedAbilitiesCostLess` (Training Grounds
  style), and on-cast triggers now carry the cast's X (`Value::XFromCost` in a
  "when you cast this spell" body — Hydroid Krasis). CR conformance in
  `core_rules/cr_recent34` (118.8 pay-life, 614.16 counter-doubling, 508.1g
  temporary attack tax).

- **WAR set complete (modern_decks, this run, 10 bombs):** Tezzeret Master of
  the Bridge, God-Eternal Kefnet, Nissa Who Shakes the World, Nicol Bolas
  Dragon-God, Bolas's Citadel, Feather the Redeemed, Finale of Promise, Deliver
  Unto Evil, Gideon's Sacrifice, Niv-Mizzet Reborn — WAR now has zero
  `set_gaps.py` gaps. New engine: `StaticEffect::GrantAffinityToSpells`
  (generalized affinity), `LookTopPutMatchingOntoBattlefield.exile_rest`,
  `EventKind::FirstCardDrawnThisTurn` + SelfSource `CardExiled` triggers from
  LKI (God-Eternal recur now covers exile), `damage_redirect_this_turn` +
  `Effect::RedirectYourDamageToChosen`, `Effect::NivMizzetReveal`,
  `CardInstance.feather_exile_return` + `Effect::MarkExileReturnOnResolve`,
  `Effect::DeliverUntoEvil` (opponent-choose split),
  `StaticEffect::HasAllOtherPlaneswalkerLoyaltyAbilities`,
  `Effect::EachOpponentExilesHandCardOrPermanent`,
  `Effect::EachOpponentWithoutLegendaryLoses`,
  `StaticEffect::PlayFromLibraryTopPayLife` (Bolas's Citadel),
  `Effect::FinaleOfPromise`. Tests in `classic_sets/war`.
- **WAR walker/spell wave (this run, 20 cards):** the hybrid uncommon walker
  cycle is complete (Dovin, Nahiri, Vraska) plus Gideon Blackblade, Jace Arcane
  Strategist, Ajani the Greathearted, Sorin Vengeful Bloodlord, Vivien Champion
  of the Wilds, Arlinn Voice of the Pack, Sarkhan the Masterless, Davriel,
  Devouring Hellion, Mizzium Tank, Tomik, Gideon's Triumph, Narset's Reversal,
  Jace's Ruse, The Elderspell, Widespread Brutality, Awakening of Vitu-Ghazi.
  New engine: `blocked_this_turn` + `R::BlockedThisTurn`; `StaticEffect::
  PreventAllDamageToThis` (combat+noncombat, `WhileYourTurn`-aware);
  `EventKind::DealsCombatDamageToPlaneswalker` (fired at the combat loyalty
  site, routed through `fire_combat_damage_triggers`); `StaticEffect::
  TypedCreaturesEnterWithExtraCounter` (Arlinn); `StaticEffect::
  LandsUntargetableByOpponents` (Tomik). Correctness: `Selector::CardsInZone`
  now substitutes the resolving ability's X into its filter (`resolve_x`), and
  `mint_token_onto_battlefield` now applies typed ETB-counter statics (Metallic
  Mimic / Cathars' Crusade / Arlinn on **tokens**). Server/UI: PW-combat-damage
  trigger labels; `pt_modified` keys on the computed type so animated
  non-creatures (Awakening land, manlands) draw their P/T box. CR conformance
  (`cr_recent28`): 306.9, 509.1, 615.
- **WAR gap waves 6–8 (this run, 21 cards):** Bioessence Hydra, Charmed Stray,
  Jaya Venerated Firemage, Kaya's Ghostform, Command the Dreadhorde, Vivien's
  Grizzly, Mowu, Band Together, Ugin's Conjurant, Arlinn's Wolf, Domri's Ambush,
  Spark Harvest, Toll of the Invasion, Eternal Taskmaster, Living Twister,
  Lazotep Plating, Davriel's Shadowfugue, Ignite the Beacon, Nissa's Triumph,
  Desperate Lunge, Gideon's Battle Cry. New engine: `StaticEffect::YourColorSourcesDealExtraDamage`
  (Jaya — another color-source you control deals +N to any permanent/player);
  `EnchantedBySource` triggers now fire on the host being **exiled** as well as
  dying (Kaya's Ghostform — `CardExiled` trigger + `PermanentExiled`
  event_subject + LKI snapshot walk); `Effect::CommandTheDreadhorde` (mass
  graveyard reanimation + self-damage = total MV);
  `Effect::LookTopMayRevealMatchToHandElseBottom` (Vivien's Grizzly, Duskwatch
  Recruiter); `StaticEffect::ExtraPlusOneCounterOnSelf` (Mowu) via
  `scaled_counter_count_on`. Correctness: **proliferate now honors CR 614.16
  counter-placement replacements** (Hardened Scales / Doubling Season / Mowu
  scale a proliferated counter; previously it added exactly one). UI/server:
  effect labels for the two new effects; a Battle **defense-counter badge** in
  the client (`pt_label`, the white-diamond sibling of the loyalty badge). CR
  conformance (`cr_recent26`): 614.16, 603.6d/603.10a, 122.1c.
- **WAR opened + GTC gaps + aura-trigger fix (prior run, 27 cards):** new
  `catalog::sets::war` (25 WAR commons/uncommons — vanillas, ETB/death
  proliferate + amass, Flurry, loot, drain, turn-gated first strike, self-
  unblockable) and `gtc17` (Frenzied Tilling, Contaminated Ground). Correctness:
  the trigger dispatcher skipped a host that lost all abilities (CR 613 "is a
  Swamp") *entirely* — suppressing the attached Aura's own equip-granted
  triggers; equip/Aura-granted triggers now fire regardless of host strip
  (Contaminated Ground's tap-drain).
- **DGM gap wave 4 (this run, 3 cards):** `dgm::gaps4` — Notion Thief, Boros
  Battleshaper, Varolz. New engine: `StaticEffect::OpponentExtraDrawsRedirected`
  (Notion Thief — `draw_one` redirects an opponent's non-draw-step draw to the
  thief, guarded by `in_turn_based_draw` / `in_draw_redirect`);
  `StaticEffect::GraveyardCreaturesHaveScavenge` (Varolz — `graveyard_granted_
  abilities` surfaces virtual `from_graveyard` scavenge at index ≥ printed_count,
  wired into activation + the bot). Correctness: `auto_extra_distinct_slot_
  targets` now fills same-filter *distinct* trigger slots for `Seq`/
  `OptionalTargets` (only genuine divide effects skip), so Boros Battleshaper's
  two "target creature" slots — and Ral Zarek's untap-another — auto-fill.
  CR conformance (`cr_recent25`): 702.97a (scavenge sorcery-speed), 509.1c
  (must-block enforced), 121.2a (draw-replacement redirect). **Remaining DGM
  gaps:** Legion's Initiative (mass timed flicker — needs `NextCombat` delayed
  trigger), Reap Intellect / Plasm Capture, Melek (cast-from-library copy),
  Goblin Test Pilot / random targeting, Guardian of the Gateless / Valor Made
  Real (multi-block — see TODO.md), the remaining Fuse splits.
- **DGM gap wave 3 (prior run, 10 cards):** `dgm::gaps3` — Showstopper, Teysa
  Envoy of Ghosts, Scab-Clan Giant, Breaking // Entering, Council of the
  Absolute, Blaze Commando, Deadbridge Chant, Ral Zarek, Emmara Tandris,
  Beck // Call. New
  engine: `ControllerDealtCombatDamage` listeners now bind the *dealing*
  creature as `Selector::TriggerSource` (Teysa destroys the attacker);
  `EventKind::YourInstantOrSorceryDealtDamage` fires once per I/S resolution
  (Blaze Commando, via `resolving_spell_caster` + a one-shot guard);
  `StaticEffect::NamedSpellCostReduction` (Council's chosen-name discount,
  generic-only); `Effect::ChooseRandomGraveyardCardCreatureToBattlefieldElseHand`
  (Deadbridge upkeep); `StaticEffect::PreventAllDamageToYourCreatureTokens`
  (Emmara, both damage paths). CR conformance (`cr_recent24`): 606.3 (loyalty
  once/turn), 117.7c (generic-only reduction), 510 (combat-damage dealer).
  Server+UI: `PermanentView.activated_ability_labels` surfaces "{cost}: effect"
  in the hover tooltip (activated analogue of the trigger/static label blocks);
  new event/effect labels wired.
- **DGM gap waves + combat-damage-target fix (prior run, 26 cards):**
  `dgm::gaps`/`gaps2` — the guild legends/mythics and remaining commons (see
  TODO.md for the roster). New engine: `Effect::DoubleAllCountersOn` (Vorel),
  `Value::DistinctlyNamedGatesControlled` (Maze's End win). Correctness: Library
  Larcenist / Krydle used `PlayerRef::Triggerer` for their combat-damage "that
  player" clauses (resolves to the *controller*); switched to `DefendingPlayer`.
  Azor's Elocutors' reset now removes *all* filibuster counters (was 1). CR
  conformance (`cr_recent23`): 701.10e (double each counter), 702.102 (Fuse),
  509.1c (true Lure). UI: Maze's End "gates N/10" HUD chip
  (`PlayerView.mazes_end_gate_progress`) + Filibuster counter label/tooltip
  (fixes a non-exhaustive-match break in the client that predated this run).
- **DGM (Dragon's Maze) opened + DIS Bronze Bombshell (prior run, ~46 cards):**
  new `catalog::sets::dgm` module (keyword vanillas, the Gatekeeper cycle,
  Battalion/Unleash/Scavenge creatures, Zhur-Taa Druid, Maw of the Obzedat, Sin
  Collector, Trostani's Summoner, Pontiff of Blight, Blood Scrivener; plus
  Phytoburst, Weapon Surge, Riot Control, Punish the Enemy, Lyev Decree, Restore
  the Peace, Mindstatic, Uncovered Clues, Warped Physique, Morgue Burst, Gruul
  War Chant, Bred for the Hunt, the Sinister Possession / Runner's Bane Auras).
  New engine work: `CardDefinition::sacrifice_and_burn_when_stolen` (CR 603.8
  steal-penalty state trigger — Bronze Bombshell); `StaticEffect::
  EmptyHandDrawBonus` (CR 121.2a empty-hand draw replacement — Blood Scrivener);
  Pontiff rides `StaticEffect::GrantTriggeredAbility` for team extort (CR
  702.99). UI: Bronze Bombshell's steal-penalty is surfaced in the client
  tooltip. Server: `crab_catalog_cards` now counts distinct card *names*.

- **DIS/RTR gap wave (this run, 20 cards):** Momir Vig, Sphinx of the Chimes,
  Elemental Resonance, Vigean Intuition, Fertile Imagination, Aethermage's
  Touch, Infernal Tutor, Ignorant Bliss, Dovescape, Muse Vessel, Isperia the
  Inscrutable, Simic Basilisk, Evolution Vat, Kindle the Carnage (DIS); Slaughter
  Games, Guild Feud, Grave Betrayal, Angel of Serenity, Azor's Elocutors, Tablet
  of the Guilds (RTR). New engine work: `ActivatedAbility.discard_cost_same_name`
  (Sphinx); `Effect::AddManaEqualToPermanentCost` (Elemental Resonance);
  `Effect::NameCardExileMatchingAllZones` (Slaughter Games); a shared
  `choose_a_card_type` helper behind `Effect::ChooseTypeRevealTopPartition`
  (Vigean) and `Effect::FertileImagination`; `Effect::GuildFeud` (dueling
  reveal-deploy-fight); `Effect::AethermagesTouch` (flash-until-end-step deploy);
  `Effect::InfernalTutor` (Hellbent-aware); `Effect::IgnorantBliss` (hand blink
  + draw via a NextEndStep delayed trigger); `Effect::Dovescape` (counter →
  caster mints Birds per MV); `Effect::IsperiaReveal` (name → tutor flyer);
  `Effect::GraveBetrayal{Register,Reanimate}` (delayed reanimate opponents' dead
  under your control, as a Zombie with a +1/+1 counter); `Effect::KindleTheCarnage`
  (repeatable random-discard board burn); `CounterType::Filibuster` + Azor's
  win-at-5 loop; `CardInstance.chosen_colors` + `Effect::ChooseTwoColorsForSource`
  / `GainLifePerChosenColorOfCast` (Tablet). Simic Basilisk (Graft) and Evolution
  Vat (granted counter-doubler) and Angel of Serenity (up-to-3 exile-until-leaves)
  ride existing primitives. CR conformance (`cr_recent22`): 701.12b fight
  power-snapshot, 106.6 hybrid mana, 205.1 chosen card type. UI/server: readable
  labels for every new effect. Deferred (each blocked on one primitive — see
  TODO.md): Valor Made Real / Guardian of the Gateless (multi-block), Psychic
  Possession (player-Auras), Rakdos
  Lord of Riots (dynamic cost-reduction static), Epic Experiment (filtered
  exile-free-cast), Search the City, Experiment Kraj, the DIS split cards.
  Bronze Bombshell shipped: `CardDefinition::sacrifice_and_burn_when_stolen`
  (CR 603.8 state trigger, latched in `steal_penalty_armed`).
- **GTC wave 16 (prior run, 8 cards):** Aurelia's Fury, Nightveil Specter,
  Glaring Spotlight, Bane Alley Broker, Signal the Clans, Unexpected Results,
  Soul Ransom, Vizkopa Confessor. New engine work: `GameState.damaged_this_
  resolution` scratch + `Selector::DamagedThisResolution` (tap/lock the entities
  a resolution damaged — Aurelia's Fury); `StaticEffect::IgnoreOpponentsCreature
  Hexproof` in the target-legality check (Glaring Spotlight); `ExileChosenFrom
  Hand` gains `link_to_source`/`face_down` flags (Bane Alley Broker's stash);
  `Effect::SignalTheClans` (search-three-random-one); `Effect::UnexpectedResults`
  (+ `return_resolving_spell_to_hand`); `Effect::PayLifeRevealExileFromHand`
  (two-decision pay-life → reveal-N → exile-one, Vizkopa Confessor). Soul Ransom
  reuses `GainControlWhileSourceRemains` + `opponents_only`/`SacrificeSource`.
  CR conformance (`cr_recent21`): 601.3e noncreature-cast lock, 508.1a attack
  summoning-sickness, 601.3a play-a-card-from-exile grant. UI: `⊘ no noncreature`
  player chip (`PlayerView.cant_cast_noncreature`). Server: `concede_earliness_pct`
  gauge. Still open (hard): Guardian of the Gateless (multi-block), Gideon CoJ
  (self-animate PW), Lazav (copy-with-overrides), Illusionist's Bracers.
- **GTC wave 15 (this run, 15 cards):** Alms Beast (combat-partner lifelink via
  live block-map resolution of `CreaturesInCombatWith` in the `PumpTeamIf`
  gather), Hold the Gates, Way of the Thief, Diluvian Primordial (free-cast I/S
  from opponents' graveyards), Five-Alarm Fire, Simic Manipulator, Tin Street
  Market, Armored Transport, Vizkopa Guildmage, Duskmantle Guildmage, Mystic
  Genesis (MV-sized Ooze via mint-time `dynamic_pt`), Borborygmos Enraged,
  Obzedat, Ooze Flux, Mark for Death. New engine work:
  `StaticEffect::PreventAllCombatDamageToThisFromBlockers` (Armored Transport),
  `Effect::WheneverYouGainLifeThisTurn` + `DelayedKind::YouGainLifeThisTurn`
  (Vizkopa Guildmage), `Effect::WheneverCardEntersOpponentGraveyardThisTurn` +
  `DelayedKind::CardEntersOpponentGraveyardThisTurn` (Duskmantle Guildmage),
  `Effect::RevealTopTakeMatchingRestToGraveyard` (Borborygmos),
  `Effect::MayExileSelfReturnNextUpkeepHaste` (Obzedat),
  `ActivatedAbility.remove_counter_among_x` (Ooze Flux),
  `Selector::OtherCreaturesControlledByControllerOf` (Mark for Death). CR
  conformance: 702.15 (lifelink), 302.6 (tap summoning-sickness), 601.2d
  (divided damage). Server: `stalemate_grind_pct` gauge. Client: Unleash chip.
- **GTC waves 10–14 (prior run, 20 cards):** the five Primordial ETB Avatars,
  Molten/Sepulchral/Sylvan/Luminate steal-destroy-reanimate ETBs, Treasury
  Thrull + Hellkite Tyrant (combat-damage payoffs + 20-artifact WinGame), Lord
  of the Void (exile-7-put-creature via `ExiledThisResolution`), Duskmantle Seer
  (`ForEach EachPlayer` symmetric reveal-drain), Deathpact Angel (token-with-
  recur), Voidwalk (blink + Cipher), Clan Defiance (`ChooseModesCast` X-burn),
  Domri Rade (planeswalker), Undercity Plague, Gridlock, Thrull Parasite, One
  Thousand Lashes (upkeep-drain lock Aura), Frontline Medic (Battalion +
  `CounterUnlessPaid`). New engine work: **combat-damage triggers that also move
  a card now bind the damaged player** (`sel_find` surfaces `Player(Target(n))`;
  the dispatcher prefers the damaged player when slot 0 accepts one — Lord of
  the Void); **`Effect::ExileReturnToOwnerNextEndStep`** (owner-control flicker,
  no counter — Voidwalk); **loyalty abilities auto-fill additional target
  slots** (Domri's −2 two-target fight); **`Effect::RemoveAnyCounter`** (Thrull
  Parasite). CR conformance: 122.3 (counter annihilation), 606.3 (loyalty
  timing), 702.2 (deathtouch). Server: `bot_match_pct` ladder-composition gauge.
  Client: effect labels for the new effects + Rampage/Bushido/Annihilator/
  Absorb/Frenzy board chips.
- **GTC wave 9 (prior run, 4 cards):** Skyblinder Staff (Equipment), Razortip Whip
  (ping artifact), and two on-death Auras (Murder Investigation, Dying Wish) that
  scale by the host's power via the CR 603.10 die snapshot. All on existing
  primitives. Tests `classic_sets/gtc` (`gtc9_*`).
- **GTC wave 8 (this run, 8 cards):** the Dimir Cipher package, Spell Rupture,
  Angelic Skirmisher (each-combat keyword grant), graveyard hate, a typed edict,
  and Coerced Confession. New engine primitive: `Effect::MillThenDrawPerType`
  (mill N from a target, draw one per milled card matching a filter). Tests
  `classic_sets/gtc` (`gtc8_*`).
- **GTC wave 7 (this run, 23 cards):** the Simic Evolve package, Boros Battalion
  payoffs, Gruul Bloodrush/land-scaling, the Orzhov mode-sweep, and two land
  Auras. New engine primitive: **Realmwright** — `Effect::ChooseBasicLandTypeForSource`
  (as-enters basic-type choice, stamped on `CardInstance.chosen_land_type`) +
  `StaticEffect::LandsYouControlAreChosenType` (additive layer-4 land-type static;
  the intrinsic mana ability follows, CR 305.6). Renegade Krasis's "whenever this
  evolves" is modeled as a paired trigger. CR conformance: 305.6, 701.12 (fight
  simultaneity), 509.1c (can't-attack-alone). Server: `unresolved_pct` health-rate
  gauge on `/metrics`. Client: `req_short` now names Planeswalker + land-type
  blocker filters.
- **GTC waves 3–5 (prior run, 40 cards):** guild Keyrunes, Extort creatures,
  +1/+1-counter evasion lords, team haste/must-attack statics, colour-filtered
  another-ETB triggers, and a spread of tricks/removal. Engine: **`EachMatching`
  (zone selector) now runs `resolve_x` on its filter** like `EachPermanent`, so
  X-scaled graveyard/zone sweeps read the cast's X (Immortal Servitude's
  mass reanimate by mana value X). CR conformance: 509.1b (block restriction),
  508.1d (must-attack declaration), 615.6 (prevent-all-damage shields). Client:
  `req_short` names disjunctive block-restriction classes ("Fly/Rch") and no
  longer mislabels an And of two specific classes. Server: IP-spread + occupancy
  on the plaintext operator page.

- **RTR gap wave 11 (this run, 11 cards):** Conjured Currency (upkeep
  `ExchangeControl`), Jarad, Golgari Lich Lord (gy-count CDA + sac-drain +
  swamp/forest recursion), Volatile Rig (coin-flip sac + death blast), Mana Bloom
  (X charge-counter mana), Izzet Staticaster, Jarad's Orders (split search),
  Racecourse Fury + Security Blockade (land-aura granted abilities), Street
  Sweeper (`AttachedToMe` aura-destroy), Urban Burgeoning, Oak Street Innkeeper.
  New engine primitives: **`Effect::SameNameDamage`** (Izzet Staticaster's ping),
  **`StaticEffect::UntapAttachedEachUntapStep`** (aura untaps its host, CR 502.3 —
  Urban Burgeoning), **`StaticEffect::WhileNotYourTurn`** (mirror of `WhileYourTurn`,
  CR 611.2 — Oak Street Innkeeper). CompRules regressions: CR 508.1d (MustAttack),
  705.1 (coin-flip heads), 514.2 (until-EOT cleanup). Server: connection-saturation
  gauges (`occupancy_pct`/`global_cap`/`max_per_ip`) in `/metrics`. Client:
  Creature/Land blocker-class labels on evasion chips.
- **RTR gap waves 8–10 (prior run, 38 cards):** the guild legends/rares/mythics —
  Collective Blessing anthem, Armada Wurm, Isperia, Trostani + Wayfaring Temple
  (populate + CDA `PumpSelfByControlledPermanents`), Necropolis Regent (combat
  damage → counters via `Value::TriggerEventAmount`, CR 119.3), Hypersonic Dragon,
  the Azorius/Archon detain-two payoffs, Transguild Promenade, Havoc Festival,
  the guildmages, Counterflux (Overload counter-each), Grove of the Guardian
  (`tap_n_filter` + sac), Desecration Demon (`PlayersMayAccept` opponent-sac),
  Shrieking Affliction, Death's Presence, Pyroconvergence, Firemind's Foresight.
  Engine: **CR 702.8** — the cast-timing check now honors the
  `ControllerSorceriesAsFlash` static (was a no-op; Teferi, Time Raveler +
  Hypersonic Dragon) via a new `battlefield_grants_flash` helper collapsing six
  duplicated blocks; **CR 115.1c** — the ETB "up to N target" auto-targeter now
  maximizes slots like the Attacks path (Azorius Justiciar detains two). Server:
  `avg_concede_turn` metric (average turn a game was thrown in). Client: a
  "NoUntap" board chip for `PreventUntap`-locked permanents (Paralyzing Grasp).
- **RTR gap waves 5–7 (prior run, 56 cards):** the guild Keyrune mana-rock cycle
  (`ManaPayload::OfColors` tap + `BecomeCreature`/`BecomeColor` animate), the
  guildmage cycle, the populate spells, the Aura package (stat-drain + upkeep
  drain, aura-granted activated keyword abilities, ETB-token Auras, aura-on-land
  `EventKind::Tapped` mill), plus scavenge/unleash/detain/burn commons. New engine
  primitives: `Effect::PreventAllCombatDamageToPlayerThisTurn` (CR 615 player-scoped
  fog — Druid's Deliverance), `Effect::SacrificeSourceUnlessPayManaValue` (CR 701.16
  — Soul Tithe), and `PlayerRef::ControllerOf` now resolves a player entity to itself
  (Rakdos's Return's "opponent-or-planeswalker → that player discards"). Server: p95
  turn/duration tail metrics on `/status.json` + `/metrics`. Client: a "Detain"
  board chip (CR 701.35) and a fixed stale evasion-label test.
- **RAV/GPT/DIS + RTR wave (prior run, 61 cards):** damage-into-counters
  replacements (`ReplaceDamageToSelfWithCounters` — Phytohydra;
  `CombatDamageToPlayerBecomesCountersAndMill` — Szadek; both CR-614 "instead",
  fire through unpreventable); `Selector::CreaturesInCombatWith` (Trial // Error);
  `Effect::ExileTopSelfPumpIfCreature` (Bioplasm). Cards on existing primitives:
  Sabertooth Alley Cat, Yore-Tiller / Witch-Maw Nephilim, Orzhov Pontiff, Bioplasm.
  RTR set opened — ~53 commons/uncommons (vanilla + Scavenge + Unleash +
  firebreathe + coin flip + tokens + Overload + hybrids + token anthem + edict +
  detain + conditional trample). CR conformance: 614.6, 702.96, 702.98. Server:
  `concede_pct`. Client: `Eva+`/`Eva-` evasion chips.

- **RAV/GPT/DIS prevention & redirect wave (this run, 20 cards):** new CR 614.9
  redirection — `Effect::RedirectNextDamage` + `PreventionShield.redirect_to`
  (Carom, Razia) and `RedirectControllerDamageToEquippedCreature` (Pariah's
  Shield); CR 615 source/target-scoped prevention statics
  (`PreventDamageToYourCreaturesFromYourSources` — Light of Sanction,
  `PreventThisDamageToColor` — Indentured Oaf); `Effect::PreventSearchesThisTurn`
  (Shadow of Doubt). Cards on existing primitives: Overwhelm, Spawnbroker,
  Firemane Angel, Halcyon Glaze, Spelltithe Enforcer, Goblin Flectomancer,
  Trophy Hunter, Wojek Apothecary, Grifter's Blade, Spectral Searchlight,
  Molten Sentry, Svogthos, Conjurer's Ban, Droning Bureaucrats. CR conformance:
  614.9, 615, 701.10 (`cr_recent14`). Server: `blowout_win_pct`. Client: the
  spell-cast trigger chip distinguishes "Creature cast" from "Magecraft".
- **GPT gap wave 5 (this run, 6 cards):** Skarrg / Orzhova utility lands, Wreak
  Havoc (`Keyword::CantBeCountered` destroy), Parallectric Feedback
  (`Value::ManaValueOf` + `PlayerRef::ControllerOf` on a targeted stack spell),
  Quicken (`GrantSorceriesAsFlash` cantrip), Wurmweaver Coil (+6/+6 Aura +
  sac-for-Wurm).
- **RAV gap waves 8–10 (this run, 28 cards):** Stasis Cell (`StaticEffect::PreventUntap`
  doesn't-untap Aura + reattach), Savra (color-filtered `CreatureSacrificed`
  payoffs — `MayPayLife` edict / `MayDo` lifegain), Searing Meditation
  (`LifeGained` → `MayPay {2}` → 2 damage), Bathe in Light
  (`Effect::GrantProtectionFromChosenColor` over the Radiance group).
  `Selector::RadianceGroup`
  generalized to fan out over any card type the subject shares (Leave No Trace
  over enchantments, not just creatures); `StaticEffect::SourceDamageCantBePrevented`
  (Excruciator — source-scoped unpreventable, distinct from the global
  `DamageCantBePrevented`). Cards on existing primitives: the Radiance spells
  (Surge of Zeal, Incite Hysteria, Leave No Trace), Induce Paranoia / Flash
  Conscription (`ManaSpentOfColorAtLeast`), Hex (`ApplyToTargets` 6-slot destroy),
  Mnemonic Nexus, Helldozer, Tolsimir Wolfblood, Woodwraith Strangler,
  Transluminant (`AtNextEndStep`), Stone-Seeder Hierophant, the Duskmantle/Sunhome/
  Vitu-Ghazi utility lands, Copy Enchantment (`enters_as_copy`), Glare of Subdual,
  Voyager Staff (`ExileReturnNextEndStep`), Twilight Drover, Necroplasm
  (`DestroyEachCreatureWithManaValue`), Shambling Shell, Woebringer Demon
  (conditional edict via `ValueAtLeast`), Perilous Forays. CR conformance:
  615.12 (source-scoped unpreventable), 701.16 (edict sacrifice vs regeneration),
  702 (Radiance card-type scoping). Server: `winner_board_cv_pct`.
- **GPT gap wave 4 (this run):** `StaticEffect::AllNonlandPermanentsAreLegendary`
  (Leyline of Singularity, layer-4 supertype add over the whole board); the legend
  rule (CR 704.5j) now reads *computed* supertypes so continuous Legendary grants
  (this Leyline + the Ring's emblem) collapse duplicates. 14 cards reusing existing
  primitives: Storm Herd (`Value::LifeOf`), Sky Swallower (mass `GainControl`), the
  Magemark evasion aura (Infiltrator's), Teysa Orzhov Scion, Tibor and Lumia
  (color-spell-cast triggers), Earth Surge / Leyline of the Meek (symmetric two-sided
  anthems), Ulasht (`Value::Sum` enters-with-counters + modal remove-counter ability),
  Thunderheads (Replicate + transient token), Stratozeppelid, Schismotivate, To Arms!,
  Starved Rusalka. CR conformance: 704.5j (legend rule reads continuous
  supertypes), **701.15b** (goaded creatures must attack a non-goader player when
  able — enforced in `declare_attackers`), 611.2c (during-your-turn anthem gate).
  RAV gap waves 6–7 (this run, 25 more cards): `Effect::DestroyEachCreatureWithManaValue`
  (Sanguine Praetor), plus board-sweepers (Hammerfist Giant, Blockbuster), the
  Transmute pair (Dimir House Guard, Ethereal Usher), and a spread of simple
  activated/triggered creatures & spells reusing existing primitives (Cyclopean
  Snare, Festival of the Guildpact, Viashino Fangtail/Slasher, Undercity Shade,
  War-Torch Goblin, Tattered Drake, Surveilling Sprite, Zephyr Spirit, Votary of
  the Conclave, Torpid Moloch, Psychic Drain, Rolling Spoil, Quickchange, Ursapine,
  Tidewater Minion, Twisted Justice, Strands of Undeath, Wizened Snitches).
- **GPT/RAV gap wave 2 (this run):** `Predicate::FirstNoncreatureSpellThisTurn`
  (+ `GameState.noncreature_spells_cast_this_turn` tracking — Nullstone Gargoyle).
  ~20 cards reusing existing primitives: the Magemark aura anthems (Fencer's,
  Guardian's via `AnthemForFilter`+`IsEnchanted`), Skyrider Trainee, Order of the
  Stars, Ogre Savant / Revenant Patriarch (enters-if-color-spent), Boros
  Fury-Shield (`ManaSpentOfColorAtLeast` burn rider), Siege of Towers (Replicate +
  land animation), Sinstriker's Will (aura granting an activated ability), Nullstone
  Gargoyle, … Server: `crab_seat_win_share_pct` per-seat fairness gauge. Client:
  Morph/Megamorph + `ProtectionFromSpellSubtype` board tags. CR conformance:
  702.36, 702.16b, 702.107.
- **GPT/RAV gap batch (prior run):** `Effect::PreventCombatDamageByTargetThisTurn`
  (deal-side combat-damage prevention, routed through the per-dealer fog path so
  a prevented attacker's blockers still strike back — Azorius Ploy);
  `Value::PlayerCount` ("for each player" — Benediction of Moons);
  `StaticEffect::CreaturesCantAttackController` (absolute attack prohibition, the
  non-payable sibling of `AttackTaxToController` — Blazing Archon). Server:
  per-format wall-clock match duration (`format_duration_totals` +
  `crab_format_avg_duration_seconds`). Client: fixed a non-exhaustive
  `CounterType::Palliation` match that broke the headless-uncheckable Bevy build.
  ~40 cards across GPT/RAV/DIS (Giant Solifuge, Crystal Seer, Culling Sun,
  Ghostway, Burning-Tree Shaman/Bloodscale, the Hunted cycle, Loxodon Gatekeeper,
  Oathsworn Giant, Lore Broker, …). CR conformance: 615.1, 702.19e, 104.3a.

- **DIS/RAV gap batch (prior run):** `Effect::SearchLibraryCreaturesUpToTotalManaValue`
  (Protean Hulk), `Effect::CounterAllOtherSpellsDrawPer` (Swift Silence),
  `Effect::RevealRandomDiscardNonland` (Fall), `Predicate::SacrificedWasColor`
  (off a `sacrificed_colors` sacrifice scratch — Lyzolda) and
  `Predicate::LastDiscardedWasMulticolored` (Stormscale Anarch);
  `Effect::SacrificeAndRemember` now surfaces a player-target slot so
  target-player edict-with-mana-value payoffs auto-target (Hit // Run). Cards:
  the DIS split cards (Crime // Punishment, Hit // Run, Rise // Fall) + the RAV/GPT
  guild bounce-land cycle and simple guild spells/creatures. Also this run:
  `LossCause::Conceded` split out from `Other` (engine) → a `concede_wins`
  alternate-win sub-bucket on `/status.json` + `/metrics` (server) and a
  `PlayerView.loss_reason` label so the HUD can annotate *why* a seat was
  eliminated (client).

- **Dissension/Ravnica gap batches (recent301–304):** `FromYourGraveyard`-scoped
  `SpellCast` triggers now dispatch (the multicolored-recur Eidolon cycle);
  `StaticEffect::GrantActivatedAbility.condition` (Hellbent-gated granted
  abilities) + `Selector::This` self-grant; `WardCost::DiscardHand`
  (`CounterUnless`); `Effect::EachPlayerPutsHandCardOnTop`;
  `Effect::LandsBecomeChosenBasicType`. Client: filtered-evasion board chip names
  the dodged blocker class ("Eva-·Fly").

- **OTJ gap batch (recent288–289):** `StaticEffect::{ExileCastCostReduction,
  PlotCostReduction}` (exile-cast reduction threads foretell + adventure-creature
  paths that previously applied none — Doc Aurlock); `CardInstance.saddled_by` +
  `Effect::ExileAndReturnSelfWithSaddler` + `DelayedKind::EndOfCombat` (Fortune,
  Loyal Steed's saddle blink); `CardInstance.crewed_by` + `Value::SourceCrewerCount`
  (Luxurious Locomotive); `Effect::LookTopMayDeployLand` (Mobile Homestead, leaves
  a non-land on top); `Effect::ExileTopLandTokenElseMayPlay` (Bruse Tarl —
  land→token / nonland→impulse). Cards: Doc Aurlock, Fortune, Luxurious
  Locomotive, Mobile Homestead, Wylie Duke, Bruse Tarl. Server: `slow_game_pct`.
  Client: `Crew×N` board chip.

- **Class enchantments (CR 716) — recent286:** `CardInstance.class_level`
  (enters at level 1, battlefield-only), level-up modelled as sorcery-speed
  activated abilities (`Effect::AdvanceClassLevel`) gated on
  `Predicate::SourceClassLevelIs`; higher-level abilities gated on
  `Predicate::SourceClassLevelAtLeast` (triggers/activateds) and
  `StaticEffect::WhileClassLevelAtLeast` (layer statics, incl. live-recomputed
  grants). `EventKind`/`GameEvent::ClassLevelReached` drives "when this becomes
  level N" (Stormchaser's Talent). `Value::OpponentsWithHandSizeAtMost`. Class
  level surfaced in the server card view + a client "Lvl N" board chip. Cards:
  the Bloomburrow Talent cycle (Stormchaser's / Gossip's / Hunter's /
  Scavenger's / Bandit's / Blacksmith's) + AFR Wizard / Cleric / Warlock Class.
  Tests in `recent_b/recent286` + `core_rules/cr_recent2` (CR 716.2 level-gated
  statics). Blacksmith's L3 "during your turn" grant rides the new
  `StaticEffect::WhileYourTurn` (CR 611.2) turn-gate wrapper.
  Remaining AFR/Talent Classes (Artist, Paladin, Druid, …) need
  level-gating on the cost/replacement/land-permission paths (`WhileClassLevelAtLeast`
  only gates layer statics) — see TODO.md.

- **MKM recent254–261 (14 cards) + primitives:** `Effect::CollectEvidenceX`
  (choose-your-X collect evidence; threads the exiled total via `ctx.x_value` —
  Incinerator of the Guilty), `DynamicPt::InstantSorceryCardsInControllerGraveyard`
  and `StaticEffect::CostReductionFirstInstantOrSorcery` (Melek), and
  `ExtraManaKind::AnyColor` (tap-time color choice — Buried in the Garden). Cards:
  Melek, Incinerator of the Guilty, Cases (Ransacked Lab / Stashed Skeleton /
  Pilfered Proof), Insidious Roots, Assemble the Players, Alquist Proft, the
  Fuss // Bother and Cease // Desist splits, Living Conundrum (empty-library
  10/10 via `SetBasePtIf`), Anzrag the Quake-Mole (BecomesBlocked + extra
  combat), Buried in the Garden. Deadly Complication upgraded to a faithful
  one-or-both `ChooseModesCast` (was forced-both, unsuspect rider dropped).
- **MKM recent245–253 (50 cards):** recent252/253 add
  `ActivatedAbility.collect_evidence_cost` (CR 701.59 as an activation cost —
  finishing Forensic Researcher), `Value::DifferentlyNamedCreatureTokensControlled`
  (Audience with Trostani), `Value::OozesInExileAndGraveyard` (Slime Against
  Humanity), the `Unlock` counter, and `Effect::Cloak` now exposing the cloaked
  permanent on `Selector::LastMoved` (Cryptic Coat). Cards: Treacherous Greed,
  Flourishing Bloom-Kin, Concealed Weapon, Lumbering Laundry, Audience with
  Trostani, Krenko Baron of Tin Street, Cryptex, Detective's Satchel, Polygraph
  Orb, Undergrowth Recon, Dramatic Accusation, Lamplight Phoenix, Slime Against
  Humanity, Magnetic Snuffler, Cryptic Coat, and the Ravnica legends Trostani
  Three Whispers / Ezrim / Agrus Kos / Aurelia the Law Above. The **Clue
  Equipment** cycle (Wrench, Rope,
  Knife, Candlestick) + Thinking Cap on new `EquipBonus.activated_abilities`
  (equipment grants an activated ability to the equipped creature, CR 702.6e —
  surfaced in the view as "Equipped:" labels) and `EquipBonus.during_your_turn_pt`
  (Knife's turn-gated +1/+0). New `EventKind::EvidenceCollected` (emitted from all
  three collect paths — cast cost, `Effect::CollectEvidence`, Ward) drives
  "whenever you collect evidence" (Surveillance Monitor, Evidence Examiner). New
  `SelectionRequirement::IsSuspected` (CR 701.60 — Rune-Brand Juggler's sac-a-
  suspected). Cards also incl. Unscrupulous Agent, Undercity Eliminator, Furtive
  Courier, Chalk Outline, Soul Enervation, Convenient Target, Curious Inquiry,
  Due Diligence, Magnifying Glass, Escape Tunnel, Scene of the Crime, Massacre
  Girl (team wither + toughness-gated death draw via a death-snapshot filter).
  recent248 adds a per-turn **artifact-sacrifice tracker**
  (`Player.artifacts_sacrificed_this_turn` + `Predicate::SacrificedArtifactThisTurn`
  + `SelectionRequirement::ControllerSacrificedArtifactThisTurn` +
  `self_cost_reduction_if_sacrificed_artifact`) — Suspicious Detonation, Furtive
  Courier's unblockable rider, Deadly Complication. recent249–251 add
  `SelectionRequirement::ControllersTurn` (during-your-turn statics) and a suite
  of suspect/Detective/control/token cards (Clandestine Meddler, Forensic
  Gadgeteer, Pompous Gadabout, Coerced to Kill, Airtight Alibi, Kraul
  Whipcracker, Forensic Researcher). CR conformance: 702.90 (wither combat),
  509.1g (lone block), 701.59 (collect). Tests in `tests/recent_b/recent245`–`251`
  + `core_rules/cr_mkm_extra`.

- **recent240–241 (DSK + MKM gap batch, 31 cards):** CR 603.4 turn-scoped
  delayed triggers — `Effect::CreaturesYouControlDyingThisTurn` (Waltz of Rage)
  and `CreaturesYouControlDealingCombatDamageThisTurn` (Mistway Spy), plus
  `Value::CardsExiledWithSourceCount` (Veteran Survivor's exile-with-count
  static). DSK: Fear of Abduction, Say Its Name, Veteran Survivor, Coordinated
  Clobbering, Waltz of Rage. MKM: the Detective / Disguise / surveil-investigate
  suite (Loxodon Eavesdropper, Jaded Analyst, Projektor Inspector, Dog Walker,
  Forum Familiar, Sanguine Savior, Mistway Spy, Exit Specialist, Glint Weaver,
  Hotshot Investigators, Innocent Bystander, Rot Farm Mortipede, Snarling
  Gorehound, Sanitation Automaton, Frantic Scapegoat, Slice from the Shadows,
  Cerebral Confiscation, Caught Red-Handed). Tests in `tests/recent_b/recent240`
  / `recent241` + CR 603.4/701.13/701.60 conformance.

- **MKM Case mechanic + recent242–244 (24 cards):** the **Case** enchantment
  (`CardDefinition.case` + `CaseData.to_solve`/`solved_*` +
  `CardInstance.case_solved`; solved at the controller's end step via
  `process_case_solves`, `EventKind::CaseSolved` → Case File Auditor). New
  primitives: `Effect::GrantKeywords`, `Effect::EachControlledCreatureDealsDamage`,
  `Value::TotalPowerControlled`, `Value::DistinctColorsAmong`, `Predicate: Default`.
  Eight Cases (Shattered Pact, Trampled Garden, Crimson Pulse, Filched Falcon,
  Uneaten Feast, Gateway Express, Locked Hothouse) + Case File Auditor, plus 15
  gap cards incl. Wispdrinker Vampire (The Chase Is On, Galvanize, Red Herring, Vengeful Creeper, Rubblebelt
  Maverick, Leering Onlooker, Tunnel Tipster, Gravestone Strider, They Went This
  Way, Undercover Crocodelf, Sharp-Eyed Rookie, Curious Cadaver, Vitu-Ghazi
  Inspector, Torch the Witness, Extract a Confession).
  Client board-glance Case/Solved chip. Tests in `tests/recent_b/recent242`–`244`
  + `core_rules/cr_solve_mkm` (Solve, CR 702.2c multi-source deathtouch, CR 613.7).

- **recent239 (DSK/OTJ/MKM/LCI/BLB gap batch, 36 cards):** the **Survival** ability
  word (a PostCombatMain trigger gated on `R::Tapped` — Rootwise Survivor,
  Reluctant Role Model), `Effect::ManifestDreadRepeatThenCounters` (Valgavoth's
  Onslaught), `R::HasGreatestPowerAmongAllCreatures` (Getaway Glamer), and
  `MoveAllCounters` now relocating **keyword counters** too. Plot + dynamic-P/T
  tokens (Tumbleweed Rising, Outlaw Stitcher), saddle-triggered mill/pump
  (Stubborn Burrowfiend), may-sac and modal ETBs (Unscrupulous Contractor,
  Kutzil's Flanker), plus composition cards (Betrayer's Bargain, Untimely
  Malfunction, Omnivorous Flytrap, Norin, Altanak, Come Back Wrong, Trial of
  Agony, Bite Down on Crime). **Collect-evidence** additional cost + Ward
  (Bite Down on Crime, Behind the Mask, Analyze the Pollen, Axebane Ferox),
  the **manifest-dread** trigger (Paranormal Analyst), a per-turn **face-down
  activity** flag (Oblivious Bookworm), the **choose/reveal-creature** cast cost
  (Monstrous Emergence), Leyline of Hope (opening-hand + life-gated anthem), and
  Creeping Peeper (`SpendRestriction::EnchantmentSpell`). Plus the five
  Bloomburrow **village** lands (creature-restricted mana + kindred payoffs),
  Whiskervale Forerunner (Valiant five-deep dig), Hollow Marauder
  (`Predicate::LastDiscardedManaValueAtMost` + graveyard affinity), Feed the
  Cycle (`AdditionalCastCost::ForageOrPay`), Freestrider Commando (no-mana-spent
  ETB counters), Fear of Burning Alive (delirium noncombat-damage copy), and
  Crimestopper Sprite (self-ETB reads the collect-evidence flag). CR 122.5 /
  702.166 / 608.2b / 701.59 / 701.61 / 708 conformance tests. Tests in
  `tests/recent239.rs`.

- **recent235–238 (DSK/OTJ, 20 cards):** manifest-dread → `Selector::LastMoved`
  rider; `Effect::TapAnyNumberThenPumpPerTapped`;
  `AdditionalCastCost::ExileFromGraveyard { count }`;
  `Player.spells_cast_from_hand_this_turn` + `Predicate::NoSpellCastFromHandThisTurn`;
  `Effect::GrantExtraPlusOneCountersThisTurn` (transient Hardened Scales);
  `StaticEffect::PumpTeamIf` delirium anthems; DSK Rooms cycle (`RoomDoors` +
  `DoorUnlocked`). CR combat tests: Flanking/Rampage/Bushido.

- **recent215–218 (FDN/BLB/MKM/TDM gaps):** `StaticEffect::SetBaseToughnessForMatching`
  + `Modification::SetToughness` (layer-7b base-toughness anthem — Maha, Its
  Feathers Night); `Value::GreatestManaValueInGraveyard` (Wick's Patrol's
  `-X/-X`). 21 cards incl. `activate_once` once-per-game (Mild-Mannered
  Librarian), page-counter inline threshold (Mazemind Tome), upkeep clone
  (Extravagant Replication), `tap_n_filter` token taps (Baylen), expend-4 payoffs
  (Teapot Slinger, Byway Barterer, Wandertale Mentor), Raid reanimation (Alesha),
  suspect-on-attack (Rubblebelt Braggart), and 4 legends (Lathril, Ayli, Kykar,
  Garna). Tests in `tests/recent215`–`recent218` + CR 700.14/702.108/602.5
  conformance.

- **recent214 (FDN reprint gaps):** `Selector::AllTargets` (every declared
  permanent/player target — "then do X to each of those targets": Biogenic
  Upgrade distributes 3 counters among up-to-3 then doubles on each) and
  `StaticEffect::DoubleDamageFromControlledCreatures` (source-restricted CR
  614.2 doubler, combat + noncombat — Gratuitous Violence). 20 FDN cards
  (Herald of Faith, Arcanis, Confiscate control-Aura, Sphinx of the Final Word,
  Kalastria Highborn, Kargan/Kitesail conditional flyers, Surrak, Immersturm
  Predator, Wildborn Preserver, …). Tests in `tests/recent214` + CR 614.2
  conformance.

- **recent209–213 (FDN reprint gaps):** `StaticEffect::AnthemForChosenColor`
  (chosen-color anthem resolved live in `gather_continuous_effects` — Heraldic
  Banner), and the `EventKind::CounterAdded` trigger wired for a +1/+1-counter
  payoff (Wildwood Scourge). ~46 FDN staples on existing primitives (end-the-turn
  Time Stop, deathtouch→poison Fynn, Guildgate cycle, target-player mass bounce,
  Aurelia extra combat, Ajani). Tests in `tests/recent209`–`recent213` + CR 728 /
  115.7 / 702.2 conformance.

- **recent185–192 (BLB/DSK/FDN/OTJ/WOE/TDM gaps):** `Keyword::Melee`
  (CR 702.121 attack pump), `EventKind::BecomesPlotted` (CR 702.170 self-trigger
  from exile — Aloe Alchemist, Longhorn Sharpshooter),
  `Effect::ShuffleGraveyardCardsIntoLibrary` (graveyard-recursion rider —
  Cathartic Parting), `StaticEffect::ControlledCreatureTypesDealExtraDamage`
  (typed +1 damage — Valley Flamecaller), and an `AttackedThisTurn`-affinity
  correctness fix (`evaluate_requirement_on_card` reads the flag — Rowdy
  Research). Bot values Melee in its attack planner.

- **Harmonize-grant + once-per-game + additive-color primitives
  (recent179–182 — TDM/FDN/DSK/BLB):** `Effect::GrantHarmonizeThisTurn` +
  `CardInstance.granted_harmonize_eot` / `effective_harmonize` (Songcrafter Mage —
  a granted Harmonize mirroring the flashback grant, exercised by CR 702.180b /
  514.2 tests), `ActivatedAbility.activate_once` (a plain "Activate only once"
  once-per-game gate reusing `exhausted_abilities` without firing exhaust events —
  Possessed Goat), and `Effect::BecomeColor.additive` (layer-5 `AddColor` for
  "becomes [color] in addition to its other colors" — Possessed Goat).

- **Trigger-doubler + cast/damage primitives (TDM/FDN batch):**
  `StaticEffect::DoubleControllerAttackTriggers` (Isshin-style attack-caused
  trigger doubler — Windcrag Siege), `StaticEffect::
  DoubleDamageFromCreaturesEnteredThisTurn` (Neriv, in `scale_damage_to`),
  `SpendRestriction::DragonOrOmenSpell` + `SpellKind.omen` threaded through
  `cast_omen` (Maelstrom of the Spirit Dragon), `CardInstance.entered_by_cast`
  + `Predicate::TriggerSourceEnteredByCast` ("if you cast it" ETB gate — The
  Sibsig Ceremony), and `R::IsAttacking` now evaluable in `affinity_filter`
  (Static Snare's "{1} less per attacking creature").

- **Speed / exhaust-matters primitives (recent167 — DFT):**
  `Value::PlayerSpeed` reads a player's CR 702.179 speed (Momentum Breaker's
  "gain life equal to your speed"), and `EventKind::ExhaustAbilityActivated`
  fires "whenever you activate an exhaust ability" — backed by an `exhaust` flag
  on `GameEvent::AbilityActivated` (Adrenaline Jockey's +1/+1-on-exhaust half).
  ~22 DFT cards in `decks::recent167` (Surveyor cycle, Speed lands, Marketback
  Walker, exhaust Vehicles, Ooze Patrol, cycling payoffs, value creatures);
  tests in `tests/recent167.rs`.

- **Vehicle/Mount + DFT-gap primitives (recent168–172 — DFT):**
  `Predicate::SacrificedWasVehicle` (Hellish Sideswipe), `StaticEffect::
  SelfIsCreatureIf` (turn-gated is-a-creature — Midnight Mangler), `Effect::
  SetSaddled` + `Effect::AnimateAsCreature` (add Creature type for a duration
  keeping printed P/T — Guidelight Matrix), and `StaticEffect::
  SelfCrewsSaddlesWithToughness` (Interface Ace crews/saddles by toughness).
  ~40 DFT gap cards across `decks::recent168`–`recent174` (the Roads land
  cycle, exhaust/speed/anthem Vehicles, saddled Mounts, removal, tuck, ETB
  value); tests in `tests/recent168.rs`–`tests/recent174.rs`.
- **DFT-gap batch 2 primitives (recent175 — DFT):**
  `PlayerRef::EachPlayerWithoutMaxSpeed` (CR 702.179 — Outpace Oblivion),
  `GameEvent::DiscardedBatch` / `EventKind::DiscardedOneOrMore` (CR 701.9 discard
  batch — Magmakin Artillerist), `Value::MountsVehiclesEnteredThisTurn` (Cloudspire
  Coordinator), `StaticEffect::OtherExhaustActivationCostReduction` (Boom Scholar,
  promoted to faithful), and two fixes: `ControllerAttackedByOpponent` triggers now
  bind the attacker as `Selector::TriggerSource` (Sabotage Strategist), and
  self-source Attacks triggers fill every "up to N target" slot via
  `auto_extra_targets_for` (CR 115.1c — Lagorin). ~19 DFT gap cards; tests in
  `tests/recent175.rs` + `cr_rules.rs` (701.9, 702.179, 115.1c). Server:
  median/p90 match-duration gauges. Client: "🏁 MAX" speed chip at speed 4.

- **Impulse-until-nonland + impulse-cost fix (recent166 — EOE/TLA/DFT):**
  `Effect::ExileTopUntilNonlandMayPlay` exiles from the top of a library until a
  nonland card, then grants a may-play (free or pay-own-cost) with an optional
  MV gate that diverts the card to hand instead (Territorial Bruntar's landfall,
  Solstice Revelations). `Effect::ExileTopAndGrantMayPlay` gained a
  `pay_own_cost` flag so plain impulse spells (Light Up the Stage, Reckless
  Impulse, Wrenn's Resolve) charge the card's real cost rather than free-casting.

- **Core loop:** LIFO stack, multiplayer priority, state-based actions, delayed
  triggers, intervening-`if` (603.4), the layer system (613), split
  first-strike / regular combat-damage steps, APNAP ordering.
- **Keywords (~120):** evasion + combat (Flying/Reach/Menace/First+Double
  Strike/Trample/Deathtouch/Lifelink/Vigilance/Protection/Hexproof/Shroud/Ward/
  Bushido/Flanking/Rampage/Provoke/Melee/Dash/Boast/Afflict/Enlist/Mobilize/
  Myriad/Ninjutsu/Goad/Lure/Poisonous…); ETB/value (Persist/Undying/Riot/Fabricate/
  Afterlife/Explore/Exploit/Extort/Investigate/Embalm/Eternalize/Backup/
  Soulbond/Mentor); counter-matters (Proliferate/Bolster/Adapt/Training/Evolve/
  Modular/Graft/Outlast/Renown/Bloodthirst/Monstrosity/Devour/Amass); cast-mode
  + alt-cost (Kicker/Casualty/Connive/Offspring/Plot/Saddle/Blitz/Spectacle/
  Escalate/Spree/Buyback/Bestow/Foretell/Suspend/Flashback/Madness/Escape/Adventure/
  Cascade/Storm/Convoke/Delve/Squad/Encore); action Kicker (702.33f —
  `kicker_action_cost` non-mana kicker); Frenzy (702.35 combat rule) and Read Ahead
  (702.155 Saga starting-chapter choice); plus Fading/Vanishing, Cumulative Upkeep, Echo,
  Dredge, Retrace, Morph/Megamorph, Crew/Reconfigure, Changeling, Soulshift,
  Unleash, Devoid, Ingest, Absorb, Warp, Station (CR 702.184/721 — charge-counter
  `{N+}` Spacecraft/Planet striations: keyword + base-P/T + static, triggered
  **and activated** bands — `StationBand.activated`, surfaced via
  `granted_abilities_for` so a Planet's `12+ | {cost}: …` band is usable once
  charged), Incubate (CR 701.53 — Incubator DFC token via `TokenDefinition.back_face`).
- **Costs/mana:** colored/generic/colorless/hybrid/mono-hybrid/Phyrexian/snow/X;
  Convoke/Delve reduction; Commander tax; alternative (pitch) costs;
  energy-gated mana abilities; X-cost activated abilities.
- **Resource systems:** Energy {E}, Poison/Toxic, Devotion, Ascend/city's
  blessing, Monarch, Day/Night, coin-flip + die-roll randomization.
- **Objects:** tokens (Treasure/Clue/Blood/Food/Map/Army/Germ; colored via the
  CR 105.2c color indicator), counters
  (incl. keyword/shield/stun/finality/rad), planeswalkers + loyalty + emblems,
  MDFC, split // fuse // aftermath, adventure, command zone + Commander,
  manlands, living weapon, clones/token-copies/spell-copies.
- **Replacement effects:** enters-tapped, enters-with-counters,
  token/counter/damage/mana doubling, regeneration, EtbTriggerTax, Maze-of-Ith
  per-source prevention, prevention shields, finality exile-instead, fog
  (CR 615.1) incl. a per-dealer exception (`prevent_combat_damage_except` —
  Inspire Awe "except enchanted/enchantment creatures") and a turn-scoped
  incoming-only combat-damage prevention (`PreventCombatDamageToTargetThisTurn`
  — Fleeting Flight). Counters cease on
  zone change (122.2).
- **Statics (misc):** no-max-hand-size, play-lands-from-graveyard,
  artifact/creature non-mana-ability locks, spell-tax, two-player coin-flip-off
  (Mana Clash), reveal-top-land-else-hand, opponents'-turn cost reduction
  (`CostReductionDuringOpponentsTurn` — Naiad of Hidden Coves); per-turn
  spell-cast locks by type (`OneSpellPerTurn` / `OneNoncreatureSpellPerTurn` /
  `OneNonartifactSpellPerTurn` — Rule of Law / Deafening Silence / Ethersworn
  Canonist, surfaced via `PlayerView.spell_cast_lock`); off-turn spell tax
  (`SpellsCostMoreExceptOnControllerTurn` — Defense Grid).
- **WOE Adventure / Role / token primitives (modern_decks):**
  `Predicate::CastSpellIsAdventure` (reads the cast spell's `adventuring` flag —
  Chancellor of Tales), `Effect::CreateTokenAttachedToEach` (one Aura-token per
  matching permanent — Asinine Antics, Twisted Sewer-Witch), and the Young Hero
  Role toughness gate (`ValueAtMost(ToughnessOf(TriggerSource), 3)`), which
  surfaced a combat.rs bug: Attacks-trigger filters bound the source as
  `EntityRef::Card` so `ToughnessOf` read 0 — now `Permanent` (CR 506.5/603.4).
  The six Role Aura tokens are consolidated in `decks::woe_roles`.
- **WOE enchantment-matters / graveyard / combat-observer primitives (modern_decks,
  recent138-140):** `Predicate::OwnExiledAdventureCard` (CR 715 — haste while you
  own an exiled Adventure, Howling Galefang); `StaticEffect::AnthemForFilter` now
  resolves non-card-only filters (e.g. `IsEnchanted`) against live state via
  `evaluate_requirement_static`, so enchanted-matters anthems work (A Tale for the
  Ages); `Effect::MayExileFromYourGraveyard { filter, then }` (reflexive variable
  graveyard-exile pinning exiled cards to `Selector::LastMoved` — Specter of
  Mortality's team `-X/-X`); `Value::MarkedDamageOn(Selector)` (marked damage via
  CR 603.10 LKI — Tangled Colony's Rat count); and a `YourControl`/`OpponentControl`
  scope fix (`actor_for_scope`) so a `BecomesBlocked` observer watches the
  *attacker*'s controller (Tattered Ratter), not the blocker's. ~24 cards across
  `decks::recent138-140`, tests in `tests/recent138-140.rs` + `cr_rules.rs`
  (Bargain 702.166, sacrifice-watcher 701.21, basic trample 702.19b).
- **WOE deferred-card primitives (modern_decks, recent146-148):**
  `StaticEffect::EntersTappedUnless` (conditional enters-tapped — Horned
  Loch-Whale, Gingerbread Cabin); `StaticEffect::PlayFromLibraryTopOncePerTurn`
  + `Player.cast_from_library_top_this_turn` (Johann); state-aware
  `SetBasePtForFilter`/`GrantKeyword` gathering over *stateful* filters like
  `IsEnchanted` (Archon of the Wild Rose — resolved live in
  `gather_continuous_effects_inner`, pinned to matching ids);
  `SelectionRequirement::AttachedToSource` (source-precise "sac an Aura attached
  to this" cost — Faunsbane Troll). Server: `median_turns`/`turn_p90` in
  status.json + `/metrics`. Client HUD: `NoUntap` tag for
  `DoesntUntapWhileCounter`. Tests in `tests/recent146-148.rs`, `cr_rules.rs`
  (502.3, 613.7, 401.6).
- **Valiant / life-matters / suspect primitives (modern_decks, recent156-161):**
  `shortcut::valiant()` (CR 702.176 — once-per-turn `BecameTarget + YourControl`;
  consolidates the four existing Valiant cards + four new BLB Mice);
  `Predicate::PlayerGainedLifeThisTurn` (complements the lost-life gate — Starlit
  Soothsayer); `Effect::ClearSuspected` (the "no longer suspected" inverse of
  Suspect — Absolving Lammasu); `CreatureType::Lammasu`. ~40 cards across BLB /
  DSK / OTJ / MKM / Foundations gaps. Client HUD: power-threshold suffixes on the
  `CantBeBlockedByPowerAtMost` / `CantBlockPowerAtLeast` chips. Server:
  `percentile_bucket` ranks in f64 (f32 drifted past 2^24 samples).
- **MH3-energy / mana-persistence / token primitives (modern_decks, this run):**
  `Keyword::HexproofFromAbilities` (CR 702.11d — opponents' abilities can't target;
  Volatile Stormdrake), `Keyword::ReplicateEnergy(n)` (energy-paid Replicate,
  copy-per-payment; Reiterating Bolt), `StaticEffect::ManaPoolsNeverEmpty` (CR 500.4
  — Upwelling), `StaticEffect::UnspentColorManaPersists(color)` +
  `DynamicPt::BasePlusUnspentColorMana` (CR 106.4 — Omnath, Locus of Mana),
  `Effect::ExileSource` (temp tokens exiled at end step — Manaform Hellkite's Dragon
  Illusion via `Value::CastSpellManaSpent` + `TokenDefinition.dynamic_pt`), and
  kicker ETB counters (Untamed Kavu).
- **Marvel's Spider-Man + faithful Cosmogoyf (modern_decks, this run):**
  `decks::spm` (41 Standard cards on existing primitives — Spiders-matter,
  Villain value, LTB tokens, connive, `MayDo` auto-attach, `Value::XFromCost`
  counters+burn, modal spells, Ward/modified statics), `CreatureType::Performer`,
  and `DynamicPt::CardsYouOwnInExile`
  making Cosmogoyf the faithful EOE card (was a fabricated Tarmogoyf stand-in;
  the BRG demo now points at the real Tarmogoyf). Server `/status` + `/metrics`
  gained `turn_p10` and `win_life_delta_p90` (distribution tails). Client HUD:
  `ProtectionFromSpells`/`ProtectionFromColoredSpells`/`TrampleOverPlaneswalkers`
  tags. CR conformance: Menace two-blocker (702.111), lifelink on ability damage
  (702.15), target-conditional cost reduction (601.2f).
- **Trigger / damage-event primitives (recent90):**
  `StaticEffect::DoubleControllerTriggersOfType` (Harmonic Prodigy —
  "a triggered ability of a Shaman or another Wizard triggers an additional
  time"; generalizes Katara's Ally doubler and applies on both the Magecraft
  and general dispatch paths); `GameEvent::DamageDealt.combat` flag +
  `EventKind::PlayerDealtNoncombatDamage` (Chandra's Spitfire — "whenever an
  opponent is dealt noncombat damage"); `AdditionalCastCost::Discard.filter`
  ("discard a land card" — Magmatic Insight).
- **CDA / cost / predicate primitives (recent52):**
  `DynamicPt::CardTypesInControllerGraveyard` (Nethergoyf */1+*),
  `SpendRestriction::AbilitiesOnly` (Omen Hawker — abilities-only mana, surfaced
  as a mana ability in the view), `Predicate::ValueIsPrime` (Zimone's prime-land
  end-step check).
- **Delirium / Aura-damage / forage / metrics primitives (modern_decks — FDN/BLB/DSK/OTJ):**
  `CardDefinition.self_cost_reduction_if_delirium` (generic-only "costs {N} less
  while four+ card types are in your graveyard" — Drag to the Roots);
  `EventScope::EnchantedBySource` now matches `DamageDealt` as well as
  `CreatureDied`, so "when enchanted creature is dealt damage" Auras fire off the
  live `attached_to` host (Cracked Skull); `EventKind::Foraged` /
  `GameEvent::Foraged` (CR 701.61 — "whenever you forage", emitted from the Forage
  resolver with a `GameEventWire` mirror + "N forages" log phrase — Corpseberry
  Cultivator). Client `keyword_reminder` covers six previously-blank keywords;
  server `/metrics` gains `crab_peak_per_ip` +
  `crab_connections_refused_by_reason_total{global,per_ip}`. ~40 cards across
  `decks::recent120-126` (FDN/BLB/DSK/OTJ — threshold, landfall, first-lifegain,
  Delirium, Mounts/Saddle, outlaw/crime/plot); tests in
  `tests/recent120-126.rs`, `cr_rules.rs`.
- **Celebration / CDA / crime primitives (modern_decks — OTJ/WOE):**
  `DynamicPt::CardsDrawnThisTurnPower` (Duelist of the Mind, `*`/3);
  `Predicate::SacrificedWasOutlaw` (activated `sac_other_filter` path stamps the
  scratch — Boneyard Desecrator); `Predicate::CelebrationActive` +
  `Player.nonland_permanents_entered_this_turn` (WOE Celebration — Armory Mice,
  Belligerent of the Ball); `StaticEffect::SelfFlashIf` gated on controlling a
  land type (Colossal Rattlewurm); `CreateTokenAttachedTo` target slots are now
  surfaced at cast time (`query.rs` — Cut In's up-to-one Role). Bot `pick_saddle`
  now fires only in precombat main. ~30 cards across `decks::recent127-128`;
  tests in `tests/recent127-128.rs`, `cr_rules.rs`.
- **Multi-block tail (modern_decks):**
  `StaticEffect::SelfCanBlockAdditionalPerAttachedEquipment` (Kemba's Legion,
  folded into `max_blocks_on`), `StaticEffect::SelfCostReducedIfPredicate`,
  `CardDefinition.cast_only_before_blockers`, `Value::HalfLifeRoundedUp`, and
  `GameState::may_choose_to_draw` (CR 121.2b/121.3 — a capped player is never
  offered an optional draw, and the cap now gates `draw_one` itself).
- **Scry/Surveil-matters + graveyard CDA (modern_decks — FIN):**
  `EventKind::ScriedOrSurveiled` (CR 701.22/701.42 — "whenever you scry or
  surveil"; emitted from the scry/surveil resolution alongside
  ScryPerformed/SurveilPerformed, excludes RearrangeTop; Matoya, Archon Elder),
  and `DynamicPt::BasePlusNoncreatureNonlandInControllerGraveyard` (+N/+N per
  noncreature-nonland card in your graveyard — Xande, Dark Mage). Client log now
  skips blank-body events so internal trigger events don't emit empty rows.
  `LandType::Town` (the FIN "Land — Town" cycle — ten enters-tapped duals +
  Adventurer's Inn) with an Affinity-for-Towns payoff (Travel the Overworld via
  `affinity_filter: HasLandType(Town)`).
  `EventKind::CreatureOrArtifactDied` (CR 700.4 — "whenever a creature or
  artifact you control dies") backed by a `GameEvent::PermanentDied` synthesized
  from the single battlefield→graveyard chokepoint, so non-creature deaths that
  emit no `CreatureDied` still reach the payoff; exile-replaced "deaths" are
  filtered at dispatch (Judge Magister Gabranth, G'raha Tia). Diamond Weapon
  rides the existing `PreventAllCombatDamageToThis` ("Immune") + a graveyard
  Affinity (`affinity_graveyard_filter: PermanentCard`).
- **Life-cost / counter / loss primitives (modern_decks — FIN):**
  `Effect::MayPayLife` (CR 119.4 — "you may pay N life: …", gated on life ≥ N,
  paid as a life loss — Seymour Flux); `StaticEffect::DoublePlusOneCounters`
  (CR 614.16 +1/+1-only counter doubler — Branching Evolution, The Earth Crystal;
  composes multiplicatively with the all-kinds `DoubleCounters`); and an
  authoritative `Player.loss_cause` stamped at each elimination site (SBA life/
  poison/commander, empty-library draw, concession, "you lose" effects) so the
  server's win-kind stats read the true cause instead of guessing from final
  board state.
- **Anthem / keyword / copy primitives (modern_decks — FIN, this run):**
  `StaticEffect::AnthemForFilter` (fixed-filter team anthem: pump + keywords over
  a printed `SelectionRequirement` via `AffectedPermanents::CardMatch` — Balthier
  and Fran → Vehicles, Ardyn, the Usurper → Demons);
  `StaticEffect::SelfHasKeywordIf` (predicate-gated self keyword — Freya Crescent /
  Cloud, Planet's Champion "during your turn / while equipped");
  `EquipBonus.during_your_turn_keywords` (turn-gated equip keyword — Dragoon's
  Lance flying); `SpendRestriction::EquipmentOnly` (Freya's equip-only mana);
  `Effect::AddCreatureTypes` (additive layer-4 type grant — Jenova's Mutant);
  `CreateTokenCopyOf.{override_colors, enters_tapped}` (exact-color / tapped token
  copies — Ardyn's 5/5 black Demon). Also fixed `fire_spell_cast_triggers` to
  include statics-/equip-granted SpellCast triggers (Red Mage's Rapier, Black
  Mage's Rod), and unified poison scaling through
  `GameState::scaled_player_counter_count` (Winding Constrictor boosts poison).
- **ONE-completion primitives (modern_decks — Phyrexia: All Will Be One):**
  CR 702.150 Compleated + `{A/B/P}` PhyrexianHybrid pips; CR 602.5g
  summoning-sick `{T}`-ability gate (activation + auto-tap) with
  `ControllerCreatureAbilitiesAsThoughHaste` (Tyvar); CR 603.4 intervening-if
  on combat-damage triggers; death-trigger doubling (Drivnod);
  `EventKind::{PoisonAdded, BecameAttached}`; loyalty-ability grants
  (`PlaneswalkersHaveLoyaltyAbilities` — Ichormoon) + shared
  `effective_loyalty_abilities` surfaced in the wire view with
  `loyalty_uses_remaining`; `Effect::{BecomeTreasure, AuraSwapFromHand,
  GrantLoyaltyTwiceThisTurn, AddCounterOfPresentKind, BlockersPoisonedThisTurn,
  PreventNextDamageByTargetMintMites, OnYourNextInstantSorceryThisTurn}`;
  `CostReductionPerCounterOnSource`, `SetBasePtForFilter`,
  `AddCreatureTypeToMatching`, `HasActivatedAbilitiesOfGraveyardLands`,
  `PreventDamageToThisRedirect`, prevention shields with mite-mint riders,
  oil-activity turn flags, target-conditional spell tax
  (`cost_increase_if_targets`), Corrupted-gated flashback, CR 704.5z Speed
  SBA, CR 702.65 Aura swap.
- **LKI / equip / zone / saga primitives (modern_decks — FIN, this run):**
  CR 603.10 granted-type death LKI — `GameState::dying_snapshot` stamps a
  leaving permanent's *computed* (layer-4) creature types into the death
  snapshot, and `R::HasCreatureType` reads computed types for battlefield
  permanents, so "when a [type] you control dies" fires for granted types
  (Jenova's Mutant); plus a dead-*subject* LKI (`resolving_lki_subject` +
  `lki_snapshot`) so "draw equal to its power" reads the dead creature's
  counter-boosted P/T. `EquipBonus.set_base_pt_controller_life` (host base P/T =
  controller's live life total — Aettir and Priwen); `EquipScale` derives
  `Default`. `Effect::SearchLibraryOrGraveyard` (dual-zone tutor — Delivery
  Moogle). **Saga creatures** (Enchantment Creature — Saga) work through the
  existing saga machinery: chapters fire and the CR 714 saga rule sacrifices
  after the final chapter on a creature body (the FIN "Summon:" cycle).
- **Tiered / granted-trigger / subtype primitives (modern_decks — FIN, this run):**
  `Effect::Tiered { modes: Vec<SpreeMode> }` — the FIN "Choose one additional
  cost" modal (reuses Spree's `CastSpellSpree` cast plumbing; the validator
  enforces exactly one chosen mode — Fire/Ice/Thunder/Restoration Magic, Tifa's
  Limit Break). `fire_combat_damage_triggers` now also fires **instance-granted**
  (`GrantTriggeredAbility` on `granted_triggers_eot`) SelfSource combat-damage
  triggers alongside printed/statics-granted ones (CR 603.2e — Summon: Primal
  Odin's Zantetsuken). The `MayPay → Reflexive` path already fans out all target
  slots (Weapons Vendor attaches Equipment + creature). New `ArtifactSubtype::Book`
  (grimoire equipment — Summoner's Grimoire). Cards this run also exercise
  `SacrificeHalf` edicts + `CreatureSacrificed` payoffs (Zodiark), self-animating
  Vehicles (`sac_other_filter` + `BecomeCreature` — Phantom Train), flash tap-lock
  Auras (`PreventUntap` + granted `CantActivateAbilities` — Stuck in Summoner's
  Sanctum), and turn-gated equip keywords (`during_your_turn_keywords` — The
  Masamune).
- **Transform-DFC / trigger-gate primitives (modern_decks — FIN, this run):**
  `Predicate::PlayerLifeAtMostHalfStarting` (CR 103.4-relative "life ≤ half your
  starting total" — Cecil, Dark Knight's flip gate); `StaticEffect::
  AnthemForFilter.only_your_turn` (turn-gated team anthem — Yuna, Hope of Spira,
  who also rides `SelfHasKeywordIf` for herself + an end-step finality-counter
  reanimate); `Effect::ShuffleFilteredGraveyardIntoLibraryGainLife` (Elixir —
  reshuffle nonlands + gain life by count). Kefka, Ruler of Ruin's
  "during your turn" `LifeLost`/`OpponentControl` draw trigger rides the
  existing `Predicate::IsTurnOf(You)` via `EventSpec.filter`. Both new FIN
  legends are in-place transformers over the existing `Effect::Transform` +
  `back_face`. Bot: `sacrifice_keep_value` ranks tokens below every real card so
  a forced edict gives up a spare token before a land. Client oracle panel:
  `event_phrase` now names ~10 more trigger events (opponent-life-gain/loss,
  you-attack, unblocked, combat-damage-to-creature, enrage, scry/surveil,
  counter-added) instead of the generic "Triggered ability:".
- **WAR bomb primitives (modern_decks, this run):**
  `Effect::WheneverCreatureDiesThisTurn` (any-creature death-chain delayed
  trigger — Massacre Girl); `Effect::DeployCreatureFromHandAttacking`
  (Kaalia/Ilharg put-from-hand tapped-and-attacking + optional EOT return);
  `Effect::LockCreatureAndPlaneswalkerCasts` (Single Combat's "until end of your
  next turn" cast lock, surfaced as `SpellCastLock.creature_pw_locked`);
  `SelectionRequirement::ToughnessAtMostXFromCost` (Finale of Eternity);
  `Effect::GrantCreatureSpellsUncounterableThisTurn` (Domri); `EventKind::
  CounterRemoved(kind)` + `GameEvent::CounterRemoved` at every loyalty-removal
  chokepoint (Chandra, Fire Artisan's "loyalty counters removed" punisher);
  `EventKind::SpellCopied` hooking `GameEvent::SpellsCopied` (now carrying the
  copier) for "cast **or copy**" triggers (Ral, Storm Conduit);
  `Effect::ExileTopFaceDownTokenReturns` (Ugin, the Ineffable's +1 — a token
  whose departure returns the linked exiled card).
- **Toxic / oil / targeting primitives (modern_decks — ONE, this run):**
  `Keyword::HexproofExceptColors(Vec<Color>)` ("can't be targeted by nongreen
  spells/abilities opponents control" — Thrun, Breaker of Silence; enforced at
  both the spell-cast and ability-target protection gates);
  `SelectionRequirement::HasToxic` (value-agnostic "creature with toxic" filter —
  Slaughter Singer, Skrelv's Hive, backed by `CardInstance::has_toxic`);
  `DynamicPt::BasePlusCountersOnSelf { counter_type, .. }` (+1/+1 per oil counter
  on itself — Evolving Adaptive). Oil-count payoffs reuse `Value::CountOf` over
  `SelectionRequirement::WithCounter` (Kuldotha Cackler). 21 ONE cards in
  `sets::one`; tests in `tests/one.rs`.
- **Proliferate / poison / ward primitives (modern_decks — ONE, this run):**
  `EventKind::Proliferated` ("whenever you proliferate", once per instance,
  fires from graveyard scopes too — Scheming Aspirant, Ezuri, Voidwing Hybrid);
  `StaticEffect::ProliferateTwice` (CR 614 replacement — Tekuthal);
  `StaticEffect::PoisonCappedAtOnePerTurn` at a new `add_poison` funnel
  unifying every poison site with CR 614.16 scaling (Melira);
  `WardCost::ManaAndLife` (compound "Ward—{3}, Pay 3 life" — Ovika, Gisa);
  any-kind `remove_counter_among_filter` costs (Tekuthal); cumulative
  Toxic/Poisonous keyword stacking at layer 6 (CR 702.180b — Plague Nurse);
  per-axis `BasePlusCountersOnSelf` scaling (Exuberant Fuseling's +1/+0 per
  oil); `Effect::SacrificeLastCreatedTokensAtNextEndStep` (Urabrask's Forge);
  `WhenTargetDiesThisTurn` watches `PermanentDied` + carries a declared target
  filter (Melira's artifact watch); `FromYourGraveyard` triggers no longer
  fire from the battlefield; equipped-state anthem filters (`IsEquipped`/
  `EquippedByAtLeast`) live-resolve per recompute (Hexgold Hoverwings, Kemba);
  `LandType::Sphere`. Server lobby-phase chat; client lobby chat panel +
  Corrupted HUD chip. ~135 ONE cards in `sets::one` (143 tests).
- **Loss/win + layer primitives (modern_decks — this run):** CR 104.3d
  can't-lose/can't-win (Angel's Grace / Platinum Angel / Abyssal Persecutor /
  Worship's damage floor); CR 113.11 `CantHaveKeyword` anti-grant (Theros
  Archetypes); CR 702.19c trample over planeswalkers (Thrasta); CR 700.4
  death-redirect guard extended to library-top redirects; token-mint
  replacements (`ClueFoodTreasureMintsOneOfEach` — Academy Manufactor,
  `TokenCreationAddsTokenPerToken` — Chatterfang);
  `SelfCostReducedPerSpellCastThisTurn` (Thrasta);
  `DynamicPt::CreatureCardsInAllGraveyardsPower` (Necrogoyf);
  `Keyword::HexproofFromMonocolored` (Rokiric); graveyard-first slot
  auto-targeting for reanimation reflexives. **MH2 sweep complete**
  (`decks::mh2b`–`mh2i`, ~180 cards; `scripts/set_gaps.py mh2` → 0).
- **MH2-sweep primitives (modern_decks, this run):** per-color mana-spent
  tracking (CR 702.137 Adamant — `cast_mana_spent_by_color` +
  `Predicate::{ManaSpentOfColorAtLeast, CastSpellNoColoredManaSpent,
  CastSpellColorlessManaSpent}` — the last gates "if {C} was/wasn't spent"
  (Drowner of Truth), derived from total-minus-colored so a {C} on a generic
  pip counts; Void Mirror / Slaying Fire); `Keyword::LandwalkFiltered` (CR 702.14c artifact
  landwalk — Vectis Gloves); CR 903.4 color identity now unions color
  indicators + activated-ability + adventure/split-half costs;
  `Value::{CardTypesInGraveyard, CardTypesInAllGraveyards,
  CardsDiscardedThisTurn, LastDiscardedCardTypes}`;
  `Effect::{RevealUntilNonlandDamage, GainHexproofUntilYourNextTurn}` (player
  hexproof until your next turn — Blossoming Calm);
  `Predicate::{SacrificedWasArtifact, TriggerSourceEnteredFromGraveyard}`;
  `ActivatedAbility.{discard_hand_cost, cost_reduction_per_counter}` (Diamond
  Lion, Deepwood Denizen); trigger ctx now stamps `cast_from_hand` (escape
  riders on ETBs); ETB triggers surface `BecomeBasicLand` target filters.
- **MH2-completion primitives (modern_decks, this run):** CR 702.29 **echo
  enforcement** (`process_echo` upkeep turn-based action: auto-pay-or-
  sacrifice + `Keyword::EchoDiscard`; echo was previously display-only);
  CR 702.26 **linked phasing** (`PhaseOut.until_source_leaves` +
  `CardInstance.phased_out_by` — Out of Time, with a per-player HUD chip +
  `PlayerView.phased_out`); CR 702.16j **protection from a card type**
  (`Keyword::ProtectionFromCardType` + the player-side
  `YouAndCreaturesProtectionFromChosenCardType` static and
  `Effect::ChooseCardTypeForSource` — Serra's Emissary); CR 601.3e
  **suspend-only** cast gate (`CardDefinition.suspend_only`); CR 604.3
  **off-battlefield CDA types** (`creature_off_battlefield` — Grist);
  CR 702.62e **granted suspend** (`Effect::GrantSuspend` +
  `CardInstance.granted_suspend`); `Effect::{MoveCounters,
  FlipCoinsChooseCount, ModularCounters, OpponentRevealsPickToBattlefield,
  GlimpseOfTomorrow, GarthOneEye, ChefsKiss, GristPlusOne,
  PlayFromGraveyardThisTurn, ExileYourGraveyardBoundThisTurn,
  FreeSpellsFromHandThisTurn}; `StaticEffect::{LoyaltyAbilitiesCostExtra,
  ModularBonusCounters}`; `ActivatedAbility.remove_counter_x`;
  `R::{ProducesColorless, IsSnow}`; `EventKind::PermanentDied`;
  `AnthemForFilter.scale_by_counters_on_self`; LookPickToHand life riders;
  affinity-style counts honor explicit controller clauses;
  `SearchPickedBy` dest resolves "under YOUR control" to the effect
  controller.
- **MH3 batch-4 primitives (modern_decks — `sets::mh3d`, 20 cards):**
  cast-trigger "up to N target" maximization — `push_on_cast_triggers` now fills
  slots 1.. via `auto_extra_targets_for` (CR 115.1c; Twisted Riddlekeeper's
  emerge tap-lock); `Predicate::SourceHasCountersAtLeast { counter, n }`
  (source counter-threshold intervening-if — Charitable Levy's three-collection
  sacrifice); fixed a `1 + usize::MAX` overflow in the `Effect::Escalate`
  resolver on any non-discard (mana) escalate cost (Collective Resistance is the
  first such card). Cards ride existing Devoid/Emerge/Kicker/Storm/Adventure/
  Prototype/`ExileTopAndGrantMayPlay`/`PlayerRef::Target` player-slot plumbing.
  Tests in `tests/mh3d.rs`; server `/metrics` + `/status.json` now expose the
  CR 104.3 win-kind split; client counter tooltip gains Lore/Level/Fade/Age/
  Defense/Oil reminders.
- **MH3 batch-5 primitives (modern_decks — `sets::mh3e`, 12 cards):**
  `ActivatedAbility.energy_x_cost` (pay X {E} where X is the activation's
  `x_value`, threaded into resolution so `ManaValueExactlyXFromCost` gates the
  target — Chthonian Nightmare); `Effect::PayEnergyValue` /
  `Effect::PayEnergyOrElseValue` (Value-amount energy pay/upkeep — Jolted Awake,
  Volatile Stormdrake); `Predicate::AttackedWithCountAtLeast` + `AnyPlayer`
  attack observers in `declare_attackers` so "whenever two or more creatures
  attack" fires for a non-attacking controller (CR 508 — Argent Dais);
  `Effect::LookTopDeployLandOrHand` (ramp-preferring dig — Planar Genesis); and a
  fix to `ExchangeControl.primary_target_filter` (falls back to the `b` slot
  when `a` is the source) so ETB control-exchanges auto-target;
  `StaticEffect::EnergyGainBonus` + a centralized `GameState::spend_energy`
  chokepoint feeding `Player.energy_spent_this_turn` +
  `Predicate::EnergyPaidThisTurnAtLeast` (Izzet Generatorium). Server `/metrics`
  + `/status.json` now expose `crab_catalog_cards`. Tests in `tests/mh3e.rs`;
  CR conformance in `tests/cr_rules.rs` (107.16, 508, 122.1).
- **CDA / UI primitives (recent94 — Equipment/Voltron):**
  `DynamicPt::ArtifactsControlledPower` (power-only artifact CDA with fixed
  toughness — Akiri, Line-Slinger); `PermanentView.attached_to_name` surfaces an
  Aura/Equipment's host so the client tooltip shows "Equipping/Enchanting: …"
  without a battlefield scan.
- **Cost / combat primitives (modern_decks — GTC/RTR):**
  `ActivatedAbility.exile_spell_cost` ("Exile [a spell] you control:" — pulls the
  top-most matching spell off the stack, which won't resolve; Nivmagus Elemental),
  and `SelectionRequirement::IsBlocked` (a blocked attacker — Smite). Battalion
  rides the existing `Predicate::AttackingWithAtLeast(3)` gate on a SelfSource
  Attacks trigger; Bloodrush reuses `from_hand` + `discard_self_cost`.
- **Selectors/filters:** `Selector::BlockingCreatures` (every creature blocking
  the source attacker — Grasping Giant), `SelectionRequirement::HasPlaneswalkerType`
  (Sunlit Hoplite / Swimmer's Elspeth/Ashiok riders),
  `SelectionRequirement::ManaValueParity` (Extinction Event's odd/even sweep);
  CDA P/T for creatures-in-your-graveyard and other-flyers-you-control
  (`DynamicPt::BasePlusCreaturesInControllerGraveyard` — Fiend Artisan;
  `BasePlusOtherFlyersControlled` — Skycat Sovereign), and creature-cards-in-
  **all**-graveyards (`DynamicPt::CreatureCardsInAllGraveyards` — Lhurgoyf,
  Mortivore); `SelectionRequirement::OwnedByYou` (CR 108.3 — Gruul Charm's
  "gain control of all permanents you own"); `Effect::DestroyAndRemember`
  (destroy + record P/T like `SacrificeAndRemember` — Orzhov Charm);
  `Selector::GreatestPowerControlledMatching(filter)` read through `Value::PowerOf`
  + `Value::Max` for "N or the greatest power among [type] you control" floors
  (Triumphant Chomp).
- **Ability/trigger riders:** statics-granted triggered abilities (Kataki),
  conditional aura riders, rhystic taxes (Esper Sentinel), once-per-turn
  triggers (603.3d), opponents-only activations, discard-self cost,
  counter-to-exile, blink-return-EOT, "when enchanted creature dies" Aura LKI
  triggers (`EventScope::EnchantedBySource` — Minion's Return) and
  auras-on-dying-creature payoffs (`auras_at_death` +
  `Value::AurasYouControlledOnDyingSubject` — Hateful Eidolon, Dawn Evangel);
  name-gated first-cast-this-turn delayed trigger
  (`DelayedKind::YourNextNamedSpellThisTurn` — Medomai's Prophecy);
  next-spell delayed triggers expose the cast spell's mana value
  (`event_amount` → `ManaValueLessThanEventAmount` — Vivien, Monsters'
  Advocate's lesser-MV tutor); conditional defender-bypass
  (`StaticEffect::CanAttackIgnoringDefenderWhile` — Drowsing Tyrannodon);
  bounty-counter dies payoff (`CounterType::Bounty` — Chevill);
  token-created triggers (CR 111.10 — `EventKind::TokenCreated`, fires once per
  token incl. doubled tokens — Voldaren Bloodcaster's five-Blood transform);
  reflexive "when you do" payoffs (CR 603.7 — `Effect::Reflexive`, opaque to the
  cast/trigger-time target walk, auto-targets at resolution; composes with
  `MayPay`/`MaySacrifice` — Itzquinth, Glorifier of Suffering, Inti);
  counter-added triggers bind `Selector::TriggerSource` to the counter-receiving
  permanent (CR 122/603.6 — Auntie Ool's Ward—Blight drain off an opponent);
  team anthem scaled by a controlled/graveyard count
  (`StaticEffect::PumpTeamByControlledPermanents` — Warrior of Light's legendary
  anthem, Cid, Timeless Artificer's graveyard-aware Artificer count).
- **Protection / locks / piles (THB batch):** protection from each mana value
  other than N (`Keyword::ProtectionFromManaValueExcept`, all DEBT facets —
  Haktos); permanent opponents-can't-cast-named lock + linked counter-exile
  (`StaticEffect::OpponentsCantCastNamed` + `Effect::CounterSpellExileNameLock`
  — Ashiok's Erasure); source-tapped untap-lock (`CardInstance.untap_locked_by`
  + `SelectionRequirement::PowerAtMostXFromCost` — Entrancing Lyre); heuristic
  pile-split (`Effect::FactOrFiction` — Fact or Fiction, Atris); gy aura
  mass-reanimate + delayed exile (`Effect::ReanimateAurasExileEot` — Storm
  Herald); reveal-6 / opponent-exile / free may-play (`Effect::AllureOfTheUnknown`).
  Protection from instants / from everything (`Keyword::ProtectionFrom{Instants,
  Everything}` — Hexdrinker). Voting / "will of the council"
  (`Effect::WillOfTheCouncilExile`, untargeted — Council's Judgment, CR 701.31).
- **Formats/modes:** Standard, Commander, Brawl, Two-Headed Giant; vs-bot,
  networked TCP multiplayer, draft + cube, Learn/Lessons sideboard, full-state
  serde snapshots (save/restore + replay foundation).
- **Client:** 3D board, game-log panel (player names), targeting + decision UI
  (resolution-time modes/amounts/divided-damage/type modals), attack-all +
  per-attacker picking, priority-aware Pass/Respond with per-step stop/skip on a
  clickable phase chart, card-zoom hover preview + reminder panel, animations,
  keyboard cursor (WUBRG hotkeys), commander-damage HUD, legal-play
  highlighting, monarch/day-night/blessing chips, reconnect banner, decklist
  import.

---

- **Upkeep-cost prompts + control funnel (modern_decks, this run):**
  `GameState::change_control` (single funnel for steals/exchanges/reverts —
  CR 302.6 sickness + CR 702.29b echo re-arm); wants_ui pay-or-sacrifice
  prompts for echo (`Effect::EchoPayOrSacrifice`) and cumulative upkeep
  (`Effect::CumulativeUpkeepPayOrSacrifice`); `CardDefinition.no_mana_cost`
  (CR 601.3e general gate, replaces `suspend_only`); CR 702.71 **Fortify**
  (`equip()` accepts Fortifications onto lands — Darksteel Garrison);
  `Effect::Balance` (Restore Balance); `Effect::GenesisWave`;
  `SpendRestriction::InstantSorceryUncounterable` (Boseiju);
  `Value::LargestCreatureTypeCount`; ward-aware hostile auto-targeting
  (bot stops feeding wards); client game-over match-stats block.

- **Even-mana-value locks + cost/entry primitives (modern_decks, this run):**
  `StaticEffect::OpponentsCant{CastEvenMv,BlockWithEvenMv}` (Void Winnower —
  CR 601.3e cast gate + CR 509.1 block gate, "zero is even"; surfaced to the
  wire view as `PlayerView.even_mv_cast_locked` + a HUD chip so the client
  greys out illegal casts); `StaticEffect::CostReductionFirstCreatureSpell`
  (Conduit of Ruin — keyed off `Player.creatures_cast_this_turn`, distinct
  from the total-spell `CostReductionNthSpell`); `Predicate::
  CreatureEnteredThisTurn` (Zhalfirin Decoy — CR 603 activation gate reading
  `Player.creatures_entered_this_turn`). Card batch (`decks::recent113`, 36
  cards): the Void Winnower/Price of Progress/Conduit trio + a Modern Horizons
  staple sweep (Changeling Outcast, King of the Pride, Vesperlark, Mother
  Bear, Goblin War Party entwine, Orcish Hellraiser, Excavating Anurid
  threshold, Headless Specter hellbent, …).
- **MH3 batch 2 (modern_decks, `sets::mh3b` — 37 cards):** rides existing
  Eldrazi/colorless, adapt/modified, living-weapon, proliferate/modular/amass,
  energy, prowess, vanishing, and modal/overload primitives; one new filter
  `R::ManaValueAtMostDevotion(Color)` (MV ≤ your devotion — Grim Servant) and
  the `Trilobite` creature type. Cards exercise Annihilator + graveyard-recur
  (Eldrazi Ravager), cast-or-cycle spawn (Drownyard Lurker), second-draw /
  second-spell payoffs (Emrakul's Messenger, Dreamtide Whale), modified-dies
  manifest reading LKI (Guardian of the Forgotten), `DoubleCountersOnEach` +
  Overload (Fangs of Kalonia), living-weapon equipment (Colossal Dreadmask,
  Drossclaw), energy dies-triggers (Cyclops Superconductor), and
  owner's-choice graveyard recycling (Not Forgotten). Also surfaces CR 700.9
  `PermanentView.modified` for the client.
- **Tap-actor / restricted-mana / Adventure primitives (modern_decks — WOE waves
  14-16):** `GameEvent::PermanentTapped.actor` (who tapped it, `Some` for
  effect-driven taps) + `EventScope::YouTapped` ("whenever you tap …" — Sharae,
  Solitary Sanctuary; distinct from the tapped-permanent-controller scopes,
  gated per CR 603.3d once-per-turn); `SpendRestriction::HighMvOrX` (spend only
  on MV-5+ or `{X}` spells — Troyan, Gutsy Explorer; `SpellKind` now carries
  `mana_value`/`has_x`). All other cards ride existing primitives (Celebration,
  `SacrificeAnyNumber`, `CostReductionNthSpell`, `PlayFromLibraryTop`,
  `CreateTokenAttachedTo`, `MillThenToHandN`, `Value::LifeGainedThisTurn`,
  `CantBeBlockedByPowerAtMost`). ~27 cards across `decks::recent141-143`; tests
  in `tests/recent141-143.rs`, `cr_rules.rs` (509.1a tapped-can't-block, 702.19e
  deathtouch-trample, 603.3d once-per-turn). Client keyword strip splits
  power-gated evasion (`Eva</≤/≥`); server status/`/metrics` expose
  `min_turns`/`max_turns`/`turn_stddev`; view trigger labels cover tap-matters.

## Tier 1 — High-leverage engine primitives

Each unblocks a large swath of cards.

1. 🟡 **Replacement-effect framework.** `replacement.rs` models zone-change
   replacements (Commander → command zone); the rest is per-card. Shipped:
   enters-under-an-opponent's-control (`CardDefinition.enters_under_opponent_control`,
   applied at the battlefield hop before any ETB trigger reads a controller —
   Captive Audience), "can't be regenerated this turn" (CR 701.15g —
   `Effect::CantBeRegeneratedThisTurn` blanks existing and future shields),
   enters-tapped (`StaticEffect::EntersTapped`, incl. self-source) and the
   enters-*untapped* override (`StaticEffect::LandsEnterUntapped` — Spelunking),
   exile-instead
   for non-cast creatures (Containment Priest), opponent-creature-dies → exile
   (Valentin), graveyard → exile hate (`ExileCardsBoundForGraveyard` via
   `route_to_graveyard` — Rest in Peace, Leyline of the Void), counter-lock
   (Solemnity), counter/damage doubling (Doubling Season, Furnace of Rath),
   damage prevention as shields (`prevention_shields`), per-source combat shields
   (Maze of Ith), damage redirection (Palisade Giant), draw doubling (Thought
   Reflection), damage halving (Ghosts of the Innocent), creature-ETB control
   steal (Gather Specimens), as-enters choice-of-P/T-and-keyword
   (`enters_as_choice`, CR 614 — Corrupted Shapeshifter, applied before the
   first SBA so a printed `*/*` never dies as a 0/0), skip-step and skip-turn.
   Counter-placement replacements (Hardened Scales, Doubling Season, Mowu's
   self-scoped `ExtraPlusOneCounterOnSelf`) now also apply on the **proliferate**
   path (CR 614.16), via `scaled_counter_count_on`. Still to generalize:
   as-a-copy ETB, draw replacement breadth. (Devouring Hellion / Rescuer Sphinx's
   as-enters reflexive shape now ship via `devour` / a reflexive ETB.)
2. ✅ **Multi-pick / "choose N" decisions.** `Decision::ChooseModes`;
   pick-from-revealed via `Effect::LookPickToHand` (Impulse, Strategic Planning).
3. ✅ **Player-chosen combat damage assignment order.**
   `Decision::CombatDamageOrder` prompts the attacker (510.1c).
4. ✅ **Linked "until this leaves play" exile** (603.6e).
   `Effect::ExileUntilSourceLeaves` + `return_linked_exiles` (Banisher Priest,
   Fiend Hunter, Oblivion Ring, Brain Maggot, Tidehollow Sculler). Monarch-linked
   sibling (CR 724 — `Effect::ExileUntilOpponentMonarch` + `ExileLink.monarch_guard`,
   returns when the monarchy moves rather than when the source leaves; Palace Jailer).
5. 🟡 **Copy of a permanent (clone).** `Effect::BecomeCopyOf` +
   `enters_as_copy` ship Clone, Phantasmal Image, Mirror Image, Stunt Double;
   token copies via `CreateTokenCopyOf`; continuous "becomes a copy" ✅
   (`BecomeCopyOfFor` — Mirrorform, Vesuva). Remaining: Helm of the Host's
   per-combat mint is approximated (no layer-1 continuous copy).
6. ✅ **Copy-a-spell-on-the-stack.** `Effect::CopySpell` /
   `CopySpellMayChooseTargets` (new-target choice) — Storm cards, Reverberate, Fork.

## Tier 2 — Engine rules fidelity (beyond Tier 1)

- ✅ **APNAP trigger ordering** — inter-player (`apnap_rank`) plus
  same-controller ordering with a real server suspend (`ResumeContext::
  TriggerOrder`), so networked seats are prompted.
- ✅ **Block-trigger conformance (CR 509.3a–e)** — "whenever this blocks" /
  "becomes blocked" fire once per creature under a multi-block; the per-object
  wordings reach every partner from one instance; `EventKind::BlocksNOrMore` /
  `BecomesBlockedByNOrMore` gate on the finished assignment (Lairwatch Giant).
- 🟡 **Divided damage / counters** — `Effect::DealDamageDivided` +
  `Effect::DistributeCounters` (Jugan) share `Decision::DivideDamage` (the modal
  is noun-aware). Forked Bolt, Pyrokinesis, Crackle with Power. Remaining:
  "choose targets as it resolves".
- 🟡 **Targeting refinements:** resolution-time legality re-check (608.2b) ships
  for single/multi-target spells and Auras, and now resolves `{X}`-from-cost
  target filters (Hearth Kami's "artifact with mana value X" via
  `ManaValueExactlyXFromCost`). "Up to N targets" ships via
  `Effect::ApplyToTargets` (Sea God's Scorn bounce-3, Wrap in Flames
  1-to-each-of-3, Elemental Expressionism bounce-2); an **optional single slot
  alongside a required one** ships via `Effect::OptionalTargets { min, body }`
  (Primal Might's required pumped creature + optional fight target, Boom Box's
  three optional destroy slots). Protection now gates spells
  *and* abilities (CR 702.16c — `ability_target_has_protection`) across color /
  creatures / creature-type (Kitsune Riftwalker, Yawgmoth, Baneslayer) /
  spell-subtype / **multicolored** (`ProtectionFromMulticolored` — Stonecoil
  Serpent) / **monocolored** (`ProtectionFromMonocolored` — Guardian of the
  Guildpact), and combat damage (CR 702.16e — `damage_prevented_by_protection`
  on both attacker→blocker and blocker→attacker). Multi-kind slots ship —
  a spell can target a permanent in one slot and a *player* in another, with
  `Selector::ControlledBy { who: Target(n) }` declaring slot `n` as a player
  target (How to Start a Riot, Sokka's Haiku's spell+land slots). Ignore-hexproof
  statics ship: creature-only (`IgnoreOpponentsCreatureHexproof` — Glaring
  Spotlight) and broad players+permanents (`IgnoreOpponentsHexproof` — Kaya,
  Bane of the Dead); the server view surfaces player hexproof per-viewer and the
  client targeting filter mirrors both. Remaining: "target each".
- 🟡 **Continuous-effect breadth:** layer-3 text-changing ✅ (Trait Doctoring);
  land-type statics ✅ (Blood Moon, Urborg); layer-4 granted supertype ✅
  (`Modification::AddSupertype` — the Ring-bearer's Legendary rider, CR 701.54c);
  layer-4 set-creature-types ✅ as a one-shot (`Effect::BecomeCreatureType` —
  Turn to Frog / Snakeform / Polymorphist's Jest) **and** the CR 613.8 type-lord
  dependency (a retyped creature is now seen by `AllWithCreatureType` lords via
  a `gate_types` second pass); layer-4 add-creature-type + layer-7b
  `SetPowerToughnessToManaValue` animating non-Aura enchantments to `MV/MV`
  creatures ✅ (`StaticEffect::NonAuraEnchantmentsAreCreatures` — Opalescence,
  Starfield of Nyx; the 5+-enchantment gate is materialized state-aware).
  Remaining: CDA corners, full text-box swaps, "becomes a copy of" layer
  interaction, type-gated `CardMatch` lords.
- 🟡 **Static ability framework:** cost-reduction statics, "you may play"
  permissions, anthem stacking incl. disjunctive multi-type lords (Blex);
  devotion-gated god states (`NotCreatureWhileDevotionBelow`) + devotion
  bonuses (`StaticEffect::DevotionBonus` — Altar of the Pantheon, CR 700.5);
  keyword loss (`LoseKeyword` — Nowhere to Run); live-recompute `GrantKeyword`
  **and `PumpPT`** statics over combat state (`IsAttacking`/`IsModified` —
  Bone-Cairn Butcher's "attacking tokens have deathtouch" and Orcish
  Oriflamme's "attacking creatures you control get +1/+0"); turn-gated statics
  (`StaticEffect::WhileYourTurn`, CR 611.2 — general on both the live and pure
  gather paths; Blacksmith's Talent L3); an opponent-scoped spell tax
  (`OpponentSpellsCostMore` — Grand Arbiter Augustin IV, exempts the controller)
  and its turn-gated sibling that also taxes non-mana abilities
  (`OpponentActivityCostsMoreOnYourTurn` — Tithe Taker, spell half in
  `extra_cost_for_spell` + ability half in `effective_ability_mana_cost`)
  and a multicolored-only spend restriction (`SpendRestriction::MulticoloredSpell`
  — Pillar of the Paruns). Remaining: broader "you may play", devotion-gated
  non-type states.
- 🟡 **Replacement of life/draw/damage events** (ties to Tier-1 #1). Life-loss
  doubling (`OpponentLifeLossDoubledDuringYourTurn` — Bloodletter) and scoped
  unpreventable combat damage (`ControllerCreaturesCombatDamageCantBePrevented`
  — Questing Beast) now ride the `adjust_life` / prevention chokepoints.
  Noncombat-only damage doubling (`DoubleNoncombatDamageToOpponents` — Solphim,
  Mayhem Dominus) rides the `deal_damage_to_from` funnel and stacks with the
  global Furnace-of-Rath doubler (combat damage stays exempt). Life-gain
  replacements now cover both a flat bonus (`LifeGainBonus` — Honor Troll) and a
  multiplier (`LifeGainMultiplier` — Rhox Faithmender), multiplier applied first
  (CR 614), neither firing on a 0-gain (CR 119.10). Hellbent all-your-sources
  damage doubling (`DoubleYourSourcesDamageWhileHellbent` — Anthem of Rakdos)
  rides `scale_damage_to`, gated on the controller's empty hand.
- ✅ **Regeneration shields & "next time" prevention** as proper shields.
- 🟡 **Damage marking vs. wither/−1−1, lethal/indestructible** audited against
  CR 120/704. (Wither/Infect damage-as-counters already ships; lethal-by-power
  `StaticEffect::LethalDamageByPower` — Zilortha — now overrides the toughness
  threshold in the SBA. **Excess damage** (CR 120.10) is tracked per resolution
  in `deal_damage_to_from` — `Predicate::ExcessDamageDealtThisResolution` gates
  "if excess damage was dealt this way" (Orbital Plunge). Remaining: the broader
  marking-interplay audit, and excess-to-another-permanent redirection (120.4a).)
- 🟡 **Loyalty fidelity:** loyalty-set effects ✅, proliferate on loyalty ✅
  (`CounterType::Loyalty`, test `cr_701_34_proliferate_adds_loyalty_counter`),
  combat damage to a planeswalker removes loyalty ✅ (CR 306.9, test
  `cr_306_9_combat_damage_to_planeswalker_removes_loyalty`); multi-target
  loyalty abilities now auto-fill slots 1.. (`auto_extra_targets_for` — Domri
  Rade's −2 two-target fight). Remaining: "any time" activation riders;
  UI-chosen (rather than auto-picked) extra loyalty targets.
- ✅ **State-based action coverage:** ±1/±1 annihilation ✅, counter caps ✅,
  legend rule ✅, saga sacrifice ✅, world rule ✅, illegally-attached Aura ✅
  (704.5n — host fails the printed enchant filter). Dungeons ✅ (CR 309/701.49
  — `base::dungeons`, `Effect::Venture`, `decks::afr`; rooms resolve inline).
  Battle-defeat SBA ✅ (CR 704.5x — a Siege with no defense counters is defeated,
  `stack.rs`). No remaining SBA gap of note.

## Tier 3 — Object model & zones

- ✅ **Battle card type** (CR 310) — `CardType::Battle` + `BattleSubtype::Siege`,
  defense counters (CR 310.7), protector choice (CR 310.6), attack-your-own-Siege
  (`AttackTarget::Battle`), **both combat and noncombat** damage remove defense
  counters (CR 310.10 — the noncombat path mirrors the planeswalker loyalty
  strip in `deal_damage_to_from`; Onakke Javelineer's ping), defeat→exile/
  transform SBA (CR 704.5x). 6 MOM Invasions in `decks::mom`; tests in
  `tests/mom.rs`. Remaining: multiplayer protector choice.
- ✅ **Sagas** (714). `saga_chapters` + `saga_advance` (History of Benalia, The
  Eldest Reborn); DFC sagas ✅ (`ExileSelfReturnTransformed` — Fable of the
  Mirror-Breaker); Read Ahead ✅ (702.155 starting-chapter choice).
- ✅ **Split cards** (709) + **Fuse** — `CardDefinition.split`,
  `CastSplitRight`/`CastSplitFused` (Wear // Tear).
- ✅ **Adventure** (715) — `CardDefinition.adventure` + `CastAdventure` (Bonecrusher
  Giant, Brazen Borrower, Murderous Rider, …).
- 🟡 **Classes / Cases / Backgrounds.** **Rooms ship** (709.5 — `room` +
  `CastRoomDoor`/`UnlockRoomDoor`; Unholy Annex // Ritual Chamber). **Cases ship**
  (MKM — `CardDefinition.case` + `CaseData.to_solve`/`solved_*` +
  `CardInstance.case_solved`; solved at the controller's end step via
  `process_case_solves`, `EventKind::CaseSolved` drives "whenever you solve a
  Case"). Six Cases + Case File Auditor in `decks::recent242`. Remaining:
  Classes (levels) and Backgrounds.
- ✅ **Leveler cards** (702.87 — `level_bands`; Student of Warfare).
- ✅ **Transforming DFCs** (712) — `Effect::Transform` toggles the active face in
  place, round-trips through serde/snapshot (Delver, Concealing Curtains).
  Remaining: DFC sagas.
- ✅ **Meld** (701.37) — `Effect::Meld` + `meld_parts`, unmelds on leave (Urza +
  Mightstone/Weakstone → Urza, Planeswalker).
- ✅ **Flip cards** (Kamigawa, CR 711) — `flip_face` + `Effect::Flip` +
  `GameEvent::Flipped`; ki counters; flip in place, revert off-battlefield
  (711.6); `damaged_by_this_turn` source tracking; `flip_when_has_keyword`
  CR 603.8 state-triggered flip (Student of Elements). Whole CHK flip cycle
  ships (Cunning Bandit … Bushi Tenderfoot, Kitsune Mystic + Autumn-Tail's
  two-target aura-move, Nezumi Graverobber, Student of Elements).
  **Prototype** (CR 702.160) ✅ — `CardDefinition.prototype` +
  `GameAction::CastPrototype`: cast a colorless artifact creature for its
  smaller, colored prototype cost/size, keeping abilities/types (the BRO
  cycle: Goring Warplow, Steel Seraph, Phyrexian Fleshgorger, …).
  **Omen** ✅ (`CardDefinition.omen` + `omen_casting` — the Adventure-style
  alternate instant/sorcery half).
- 🟡 **Face-down permanents** (708) — `face_up_def` stashes the real card; Manifest
  / ManifestDread + `TurnFaceUp`; Morph/Megamorph cast-face-down ✅. Remaining:
  Disguise/Cloak edge cases (both core paths ship — see Tier 4).
- ⏳ **Ante / conspiracy / dungeon / sticker / attraction** zones (novelty only).
- ✅ **Emblems** as command-zone objects — `Player.emblems` + `CreateEmblem`,
  carrying both triggered and **static (anthem) abilities** (Vivien Reid's −8;
  synthesized into continuous effects in `gather_continuous_effects`).
- ⏳ **Sideboard zone** + "from outside the game" (wishes, companions).

## Tier 4 — Keyword & ability mechanics (the long tail)

Each a small targeted feature; sweep batch by batch.

- **High frequency / modern staples:** ✅ Madness, ✅ Escape, ✅ Adventure,
  ✅ Soulbond, ✅ Mutate (CR 702.140 — `CardDefinition.mutate` +
  `GameAction::CastMutate`; merges onto a non-Human host you own, unions
  abilities, scatters on leave, `EventKind::Mutated` triggers — the Ikoria
  cycle), ✅ Companion ({3} sideboard→hand + `companion`
  deck-construction validation, full Ikoria cycle),
  ✅ Foretell, ✅ Disturb, ✅ Daybound/Nightbound (keywords + day/night +
  502.2 transition + DFC auto-flip), ✅ Decayed, ✅ Blitz, ✅ Casualty, ✅ Connive,
  ✅ Backup, ✅ Bargain,
  ✅ Craft (CR 702.169 — `shortcut::craft`: sorcery-speed activated ability
    pairing `craft_exile_cost` (exile N other objects from among permanents you
    control and/or graveyard cards) with `Effect::ExileSelfReturnTransformed`;
    LCI batch in `sets::lci` — Tithing Blade, Visage of Dread, Spring-Loaded
    Sawblades, Waterlogged Hulk),
  ✅ Disguise/Cloak, ✅ Plot, ✅ Saddle,
  ✅ Gift (CR 702.165 — `CardDefinition.gift` + `GameAction::CastGift`; promise
  the gift and resolve the enhanced `gifted_effect`, incl. target-broadening —
  Into the Flood Maw, Long River's Pull),
  ✅ Survival (CR 702.180 — "at your second main phase, if tapped …" as a
  `StepBegins(PostCombatMain)`/`ActivePlayer` trigger under a tapped
  intervening-`if`; Bloomburrow Survivor batch),
  ✅ Omen (CR 702.183 — `CardDefinition.omen` + `GameAction::CastOmen` +
    `CardInstance.omen_casting`; cast the creature card as its instant/sorcery
    Omen half, which shuffles into the owner's library on resolution *or*
    counter via the `route_to_graveyard` funnel — the Tarkir Regent/Stormbrood
    Dragon cycle),
  ✅ Offspring, ✅ Impending, ✅ Ninjutsu, ✅ Embalm / Eternalize,
  ✅ Exhaust (activate-only-once activated abilities — Camera Launcher),
  ✅ Mayhem (CR 702.187 — `Keyword::Mayhem` + `GameAction::CastMayhem` reusing
    the flashback exile-after machinery, gated on `Player.discarded_this_turn`;
    "if the mayhem cost was paid" riders via `cast_via_mayhem`/`SpellWasMayhem`),
  ✅ Harmonize (CR 702.180 — `Keyword::Harmonize` + `GameAction::CastHarmonize`:
    graveyard recast with optional tap-a-creature generic discount, exile-after),
  ✅ Web-slinging (CR 702.188 — alt-cost: pay cost + return a tapped creature),
  ✅ Flurry (`shortcut::flurry` — "your second spell each turn" trigger over
    `SpellsCastThisTurnEquals`),
  ✅ Job Select (CR 702.182 — living-weapon-shaped Equipment minting a 1/1 Hero),
  ✅ Renew (graveyard-exile activated ability via `from_graveyard` +
    `exile_self_cost`), ✅ Mobilize / Mobilize X (`shortcut::mobilize`,
    `mobilize_value`), ✅ Seek (CR 701.52 — `Effect::Seek`, random library pick),
    ✅ Time Travel (CR 701.56 — `Effect::TimeTravel`: removes time counters from
    the player's suspended cards / adds to vanishing permanents; bot heuristic,
    per-object UI choice is a follow-up),
    ✅ Villainous Choice (CR 701.55 — `Effect::VillainousChoice`: each chooser
    in APNAP order takes the lesser-self-harm option; impossible options dodge),
    ✅ **The Ring tempts you / Ring-bearer** (CR 701.54 — `Effect::RingTempts`
    + `Player.{ring_temptations,ring_bearer}`; the four cumulative emblem
    abilities applied off the level: can't-be-blocked-by-greater-power (1+),
    attack-loot (2+), blocked-creature-sacrifice (3+, via
    `Effect::SacrificeAtEndOfCombat`), combat-damage drain (4+). `decks::ltr`
    LTR batch + `EventKind::RingTempted` for "choose a Ring-bearer" payoffs.
    Bearer auto-picked (highest power); per-player UI choice is a TODO.md
    follow-up).
- **Counter / +1+1 matters:** ✅ Proliferate, Bolster, Adapt, Training, Evolve,
  Mentor, Modular, Graft, Outlast, Renown, Bloodthirst, Monstrosity, Devour,
  Amass — all via `shortcut::*` builders.
- **Cast-from-elsewhere:** ✅ play-from-library-top statics (Courser, Oracle of
  Mul Daya, Mystic Forge), ✅ Suspend (creature-suspend haste + free-cast target
  UI are follow-ups), ✅ Forecast, ✅ Hideaway, ✅ Aftermath (`CastAftermath`),
  ✅ Unearth (CR 702.84 — `shortcut::unearth`: a `from_graveyard` sorcery-speed
  ability that returns the card with haste + an end-step exile; the bot offers
  graveyard-activated abilities, the client hover panel labels them).
- **Combat-flavor:** ✅ Bushido, Flanking, Rampage, Provoke, Battle Cry, Exalted,
  Frenzy, Melee, Dash, Boast, Afflict, Enlist, Mobilize, Myriad, Amass,
  Assigns-combat-damage-by-toughness (`AssignsCombatDamageByToughness`, CR 510.1c
  — Doran, Tapestry Warden, Bill the Pony).
- **Value/ETB:** ✅ Investigate, Fabricate, Riot, Raid, Afterlife, Explore, Squad,
  Forage, Endure, Exploit, Extort, Support, Suspect, Discover, Collect Evidence,
  Expend, Valiant, Cohort (Munda's Vanguard, Drana's Chosen — tap-another-Ally
  activation cost).
- **Leaves-battlefield LKI:** ✅ 603.10 — `Value::PowerOf`/`ToughnessOf` read a
  dying object's last-known P/T (Goldvein Hydra, Cacophony Scamp).
- **Spell-matters:** ✅ Escalate, Splice, Replicate, Cipher, Surge, Spectacle,
  Addendum, Demonstrate, Conspire; Overload ships as an alt-cost.
- **Resource systems:** ✅ Energy ({E} pool + HUD chip; Kaladesh set; energy-gated
  mana abilities); ⏳ Experience counters → actually **✅** (`CounterType::
  Experience`); ✅ Poison/Toxic, Devotion, Ascend/city's blessing; ✅ Monarch;
  ✅ Day/Night (502.2 turn-based transition + Daybound/Nightbound DFC auto-flip
  + `EventKind::DayNightChanged` "day becomes night / night becomes day"
  triggers — Brimstone Vandal); ✅ Coven (`Predicate::CovenActive` — 3+ creatures
  with different powers; HUD "✸ coven" chip);
  ✅ **Ability-word conditions** (CR 207.2c — `Predicate::{ThresholdActive,
  MetalcraftActive, FerociousActive, HellbentActive, FormidableActive}`,
  PlayerView flags + shared HUD chips; `sets::decks::abilitywords` /`recent68`);
  ✅ **Descend** (LCI — `SelectionRequirement::ControllerDescend(n)` +
  `Predicate::{DescendActive,DescendedThisTurn}` count permanent cards in the
  graveyard / "descended this turn" per CR 700.11; `DynamicPt::
  PermanentCardsInControllerGraveyard` for fathomless descent; PlayerView
  `descend_count` + HUD "⛏ descend N" chip; `sets::lci` batch);
  ✅ **Speed / "Start your engines!"** (CR 702.179 — `Player.speed` 0–4,
  `Keyword::StartYourEngines`, life-loss increment, `Predicate::SpeedAtLeast`
  for "Max speed —"; DFT batch in `decks::recent`); ✅ Ring-bearer (CR 701.54,
  `decks::ltr`);
  ✅ **Commit a crime** (CR 700.13 — `EventKind::CommittedCrime` fires when you
  cast a spell / activate an ability targeting an opponent, their permanents/
  cards, or a spell they control; `Player.committed_crime_this_turn` +
  `Predicate::CommittedCrimeThisTurn`), ✅ **Pack tactics**
  (`Predicate::AttackedWithTotalPowerAtLeast`), ✅ **Outlaws**
  (`SelectionRequirement::IsOutlaw` + `Predicate::ControlsOutlaw`) — OTJ batch in
  `decks::recent20`. ✅ **Corrupted** (CR 702.166 — `Predicate::CorruptedActive`:
  an opponent has 3+ poison; ONE batch in `sets::one` — Apostle of Invasion,
  Bonepicker Skirge, Vivisection Evangelist, Sinew Dancer, Fleshless Gladiator).
- **Fading family:** ✅ Fading, Vanishing (`process_fading_vanishing`). Remaining:
  Parallax Dementia's steal-on-leave rider.
- **Older mechanics:** ✅ Soulshift, Epic, Umbra armor, Affinity, Entwine, Buyback,
  Miracle, Bloodrush, Unleash, Scavenge, Transmute, Bestow, Tribute, Offering
  (CR 702.48 — `AlternativeCost.offering` + `ManaCost::reduce_by_cost`; the
  Kamigawa Patron cycle), ✅ Recover (CR 702.58 — `shortcut::recover`: a
  `CreatureDied / FromYourGraveyard` trigger gating a `MayPay` that returns the
  card from the graveyard or exiles it; Coldsnap I/S in `decks::recent`).
  Spiritcraft "cast a Spirit or Arcane spell" triggers
  ride `SelectionRequirement::HasSpellSubtype` + `shortcut::spiritcraft`.
  ✅ Blight (CR 701.68 — `Effect::Blight`: put N -1/-1 counters on a creature
  you control; `WardCost::Blight` is the Ward—Blight variant — Auntie Ool,
  Blighted Blackthorn, TLA).
  ✅ Haunt (CR 702.55 — `Effect::HauntCreature` + `DelayedKind::
  WhenHauntedCreatureDies`: a dying creature / resolved I/S is exiled haunting a
  creature, firing its haunt body when that creature dies; Guildpact cycle in
  `catalog::sets::gpt`). ✅ Ripple (CR 702.20 — `Effect::Ripple` +
  `shortcut::ripple`: a cast trigger that reveals the top N, free-casts
  same-named copies, and bottoms the rest; Coldsnap Surging cards).

## Tier 5 — Mana & cost system

- ✅ **Typed spend restrictions / provenance riders** — `SpellKind` +
  `SpendRestriction` (Cavern of Souls, Power Depot). Remaining ⏳: per-source
  restrictions beyond these (filter lands).
- ⏳ **Minimum-cost floor** (Trinisphere) and **cost-increase statics** beyond the
  first-spell tax. (Note: Trinisphere floor actually ships — see CUBE_FEATURES.)
- 🟡 **Conditional / additional costs** as a general modal layer. Card-intrinsic
  target-conditional reduction ships (`self_cost_reduction_if_target` — Ride's
  End's "{3} less if it targets a tapped permanent", generic-only / colored-pip
  safe). Board-state / per-turn-counter scaling reductions ship too
  (`self_cost_reduction_if_control` — Pearl of Wisdom;
  `StaticEffect::SelfCostReducedPer{Discard,CreatureAttacked}ThisTurn` — Hollow
  One, Search Party Captain). Source-power-scaled reduction of *other* spells
  ships (`StaticEffect::CostReductionBySourcePower` — Golden-Tail Trainer), and
  affinity-style `SelfCostReducedPerPermanentMatching` now honors board-state
  filters (Walking Skyscraper "per modified creature"). Remaining: per-mode
  Spree costs.
- ✅ **{X} in activated abilities** — `activate_ability` pays
  `mana_cost.with_x_value(x)` (Necropolis Fiend, Kasmina's `-X`). Remaining ⏳:
  **delve/convoke colored** contribution.
- ⏳ **Snow-mana-only** and **mana-value-X** cost gates.

## Tier 6 — Combat fidelity

- ✅ **Damage assignment order** (Tier-1 #3) + **trample math** with
  multiple/deathtouch blockers — `default_damage_split` assigns lethal in
  order (deathtouch lethal = 1, CR 702.2e) and tramples the remainder
  (CR 510.1c/702.19g). Tests: `cr_702_2e_trample_deathtouch_*`,
  `cr_702_19g_*`.
- 🟡 **Banding** (CR 509.2 / 510.1c) — a banding blocker routes the attacker's
  combat-damage order + assignment to the *defending* player (Benalish Hero).
  Remaining: attacking-band formation + "bands with other".
- ✅ **Multiple combat phases** — `AdditionalCombatPhase` (Hellkite Charger) +
  post-main insertion (Relentless Assault). First-combat detection
  (`combat_phases_this_turn` + `Predicate::IsFirstCombatPhaseThisTurn`) gates
  "if it's the first combat phase" riders so extra combats don't loop (Genji
  Glove). **Additional end steps** (CR 500.7 — `Effect::AdditionalEndStep` +
  `end_steps_this_turn` + `Predicate::IsFirstEndStepThisTurn`; Y'shtola Rhul).
  Repeated phases are surfaced to UIs via `ClientView.extra_phase`.
- ✅ **"Whenever you attack"** (CR 508) — `EventKind::YouAttack` fires once per
  combat for the attacking player (not per-attacker), via `shortcut::on_you_attack`.
  Replaces the old `Attacks/YourControl + once_per_turn` approximation on
  Razorkin Hordecaller, Inti, Gut, Raffine, Most Valuable Slayer, Lionheart Glimmer.
- 🟡 **"Must/can't attack/block" restrictions** — `Keyword::{CantAttack,CantBlock,
  AttacksAlone,CantAttackAlone,MustBeBlocked,AllMustBlock,MustAttack,MustBlock}`, Goad;
  power-based evasion (`CantBeBlockedByPowerLess` — Formation Breaker;
  fixed-threshold `CantBeBlockedByPowerAtMost(n)` — Questing Beast);
  turn-scoped defender-bypass grant (`AttackDespiteDefenderThisTurn` — Krotiq
  Nestguard); count-gated attack+block (`CantAttackOrBlockUnlessYouControlCount`
  — Topiary Stomper's "unless you control seven or more lands", with `attack_only` / `block_only` facets
  (Lambholt Pacifist / Olog-hai Crusher), honored in combat, affordances, bot,
  and the legal-blocker gate); hand-size-gated
  (`CantAttackOrBlockUnlessHandSizeAtMost` — Hazoret), delirium-gated
  (`CantAttackOrBlockUnlessDelirium` — Patchwork Beastie) and descend-gated
  (`CantAttackOrBlockUnlessDescend(n)` — The Ancient One, via `descend_count`).
  Open: granted
  must-attack with future-turn duration, multiplayer goad-target clause,
  cost-to-block (509.1d-f).
- ⏳ **Planeswalker / Battle as attack targets** UI + redirection.
- ✅ **Goad**, **Lure**, **Provoke**, **Ninjutsu swap**.

## Tier 7 — UI / UX core (the Arena "feel" gap)

1. ✅ **Card-zoom hover preview** — `hover_card_preview` (flips side to avoid
   covering the card); Alt-hold drives the centered detailed peek, which shows
   both faces of a DFC side by side plus the catalog rules-text panel (the
   small preview flags DFCs with a "hold Alt" hint line).
2. ✅ **Stops / auto-yield config** — `auto_advance_p0` smart default + per-step
   Stop/Skip overrides on the phase chart (`StopConfig`), separate for your turns
   vs. opponents'.
3. 🟡 **Combat math / damage preview** — `combat_preview` projects life swing +
   dying creatures (first/double strike — incl. double strike's two damage steps
   for face/trample/lifelink — deathtouch spread, trample, protection),
   layer-aware, with planeswalker-target rows; the client HUD flags a projected
   life total ≤ 0 with a "☠ LETHAL" tag. Remaining: multi-blocker damage-order
   nuance.
4. ⏳ **Undo / mana-tap rollback** — undo un-committed taps before a spell locks in.
5. ✅ **Targeting arrows on the stack** — `draw_stack_arrows` (primary +
   additional-target slots; counter magic points at its spell).
6. ✅ **Hold-priority toggle** — `H` / "Auto-pass" flips
   `FastForward::manual_priority`. Shift-hold-after-your-spell ⏳.
7. ✅ **Stack visualization** — the stack renders as a visual zone; per-item
   respond/resolve affordances ⏳.
8. ✅ **Phase bar / step indicator** — left-edge chart, clickable stop markers,
   right-click "pass until this step".
9. 🟡 **Resolution-time decisions for humans** — via the stash-and-rerun suspend:
   ✅ ChooseModes, modal triggers, MayDo, DivideDamage, ChooseAmount, creature-type
   choices, seat-routed yes/no asks (rhystic, Tribute, Browbeat, MayPay),
   CommanderRedirect (yes/no modal), ChooseLegendToKeep (pick-one modal),
   CoinFlip/DieRoll (client-rolled "Flip"/"Roll dN" button) — every
   `DecisionWire` variant now has a client UI (the match is exhaustive, no
   wildcard, so new variants are compile errors instead of client freezes).
   Remaining ⏳: modal triggers with targeting modes, non-Bool
   opponent-owned picks.

## Tier 8 — UI / UX quality-of-life

- ✅ Browsable **graveyard / exile** zones (`V` toggles exile, with source
  annotations); library shows a count chip only.
- ✅ **Search / Scry / Surveil / Mulligan** picker UIs (top/bottom toggles, reorder
  buttons). Drag-and-drop reorder ⏳.
- ✅ **London mulligan** bottoming; Serum Powder gets its own button.
- 🟡 **Floating life deltas** ✅; per-turn life-history graph ⏳.
- ✅ **Commander-damage HUD** (903.10a) — per-source `⚔ <cmdr> N/21` chip,
  amber→red near loss.
- ✅ **P/T + loyalty badges** — modified creatures get a floating `P/T` badge;
  planeswalkers always carry a `◆loyalty` badge (`systems/pt_label`).
- ⏳ **Hand sorting / auto-tap prefs / "play tapped land" prompt**.
- ✅ **Squad / Replicate pay-N stepper**; impending countdown badge; NameCard
  picker.
- ✅ **Reminder text & rules tooltips** — hover info panel from the catalog
  (type line, P/T, keyword reminders, oracle-ish ability panel).
- 🟡 **Hotkey legend** ✅ (F1 / `?`); remappable keys ⏳.
- 🟡 **Highlight legal plays** — `ClientView` carries castable/pitchable/kickable
  hand, activatable permanents, legal attackers/blockers (step-aware). Remaining:
  per-target hint layers.
- ⏳ **Animations & SFX** polish; board-state pings/alerts.
- ✅ **Settings menu** (window/resolution/quality/gameplay, persisted);
  audio/accessibility tabs ⏳.
- ✅ **Battlefield organization** — identical tokens pile with ×N badges.

## Tier 9 — Multiplayer & social

- ✅ **Lobby / matchmaking** — LAN lobby browser (create/join/spectate, host bot
  add/remove). Remaining ⏳: join-by-code over internet, quick-match.
- ✅ **Reconnect / resume** — resume tokens + backoff retry + full snapshot
  restore + a "reconnecting (N/10)…" client banner; tokens persist to disk /
  localStorage so a crashed client gets a menu "Rejoin Last Match" button
  (cleared on clean exit / match end).
- ✅ **Spectator mode** (read-only `ClientView` stream).
- ✅ **Player identity** — editable display name reaches every seat + log lines,
  persisted across launches.
- 🟡 **Chat** — free in-match chat ships (`T`), and lobby-phase chat relays
  to lobby members (same `T` input + a lobby panel). Remaining ⏳: emotes, mute.
- ✅ **Timers** — per-action rope (`CRAB_ACTION_TIMEOUT_SECS`) + per-game
  chess clock (`CRAB_CHESS_CLOCK_SECS`: per-seat match budget, flag fall
  concedes; `ServerMsg::Clock` + a client m:ss chip).
- ⏳ **Friends / invites / ratings / leaderboards**.
- ⏳ **Free-for-all politics** UI for 3+ player tables.

## Tier 10 — Formats & match structure

- ⏳ **Best-of-3 + sideboarding** flow.
- 🟡 **Deck legality validation** — size/copy/singleton/Commander-identity ✅,
  ban + restricted lists ✅ (`format::validate_deck`), companion deck
  restrictions ✅ (`format::companion_restriction_met`, CR 702.139c). Remaining:
  per-set legality pools (Standard rotation), Pauper rarity.
- ⏳ **More 60-card formats** (Modern/Pioneer/Legacy/Vintage/Pauper — mostly
  banlist/pool config).
- ⏳ **Limited match rules** (40-card, basic-land access).
- ⏳ **Multiplayer variants** (Planechase, Archenemy, Oathbreaker, Star, Emperor).
- ⏳ **Casual toggles** (free mulligans, vanguard).

## Tier 11 — Limited (draft / sealed)

- ✅/🟡 **Draft + cube** exist. Extend with:
- ⏳ **Sealed**, ⏳ **bot drafters** (signal/pick heuristics), ⏳ **draft variants**
  (Winston/Rochester/Grid/…), ⏳ **set-based draft**, ⏳ **draft replay / pick
  history**. ✅ **Deck export** — the deckbuilding screen's "Save Deck" writes
  the staged main+sideboard as importable decklist text to
  `<config_dir>/crabomination/decks/` (localStorage on wasm).

## Tier 12 — Deckbuilding & collection

- ⏳ **In-app deck builder** (search, curve view, legality, sample-hand).
- 🟡 **Import / export** — import ships (`decklist::parse_decklist`, Arena/MTGO
  text; menu "Play Deck vs Bot" loads and validates). Remaining ⏳: export,
  .dec/.cod, paste-from-clipboard, choosing opponent's deck.
- ⏳ **Deck stats** (curve, pips, type breakdown).
- ⏳ **Collection tracking**; ⏳ **Scryfall-like card search** over the catalog.

## Tier 13 — AI

- 🟡 **Smarter combat** — `server/bot.rs` blocking is heuristic (value trades,
  first-strike/deathtouch/trample/**indestructible** awareness — an
  indestructible body walls the biggest attacker for free and an indestructible
  attacker can't be cleanly traded — gang-block-to-survive, **and
  chump-blocking to save a planeswalker we control when its attackers are
  lethal to its loyalty — the life-threat calc counts only player-bound
  damage**); attacking has a suicide filter + evasion awareness + planeswalker
  redirection. Value-ping removal also aims an "any target" ping at an opponent's
  face when that hit is exactly lethal (reach for the win). The bot crews
  Vehicles (`pick_crew`) **and now saddles Mounts** (`pick_saddle`) before
  combat so attacks-while-saddled riders fire. Remaining: race math,
  multi-blocker math, attacking-into-open-mana respect.
- ⏳ **Better sequencing** (land drops, hold-up, when to cast).
- 🟡 **Mulligan decisions** — `RandomBot` ships flood/screw mulligans with
  color-screw awareness. Remaining: transitive fetch/dual sources.
- ⏳ **Targeting / mode / X-value choices** by evaluation.
- ⏳ **Difficulty levels**; optional **search-based AI** (MCTS over snapshots).

## Tier 14 — Replays, analysis & observability

- 🟡 **Action-log replay viewer** — the capture side ships: `CRAB_REPLAY_DIR`
  appends one JSONL replay per match (header/players, one line per broadcast
  event batch, footer). Remaining: the viewer.
- ✅ **Game history / match results persistence** — `CRAB_MATCH_LOG` appends
  one JSON line per finished match (lobby/bot/pair paths;
  `crabomination_server::history`).
- ⏳ **Export game to shareable file** (formalize the audit-snapshot workflow).
- ⏳ **In-game "what happened" log filtering** (by player/zone/type).

## Tier 15 — Accessibility

- ⏳ **Colorblind-safe** indicators, **text scaling / high-contrast /
  reduced-motion**, **full keyboard play**, **screen-reader narration**, **"full
  control" mode** (never auto-skip).

## Tier 16 — Infra, correctness & content tooling

- ⏳ **Seeded / deterministic RNG** surfaced for reproducible games.
- ⏳ **Snapshot round-trip property tests** + **action-sequence fuzzing**.
- ⏳ **Crash-recovery / autosave** from snapshots.
- ⏳ **Card-scripting DSL** to reduce catalog boilerplate.
- ⏳ **Set / Scryfall import pipeline** (`scripts/verify_cards.py` exists — extend).
- ⏳ **Card art / image pipeline**.
- ⏳ **Rules-engine conformance suite** mapped to CR sections.
- ✅ **Operator telemetry endpoint** — `CRAB_STATUS_BIND` HTTP `/healthz` +
  `/status` (uptime, rolling match stats, slot accounting).

---

## Suggested sequencing

1. **Replacement-effect framework** (Tier-1 #1) — highest-leverage primitive still
   open.
2. **Card-zoom + stops/auto-yield + combat-math preview** (Tier-7 #1–3) — the trio
   that most closes the Arena "feel" gap.
3. **Best-of-3 + sideboard + deck legality** (Tier 10) — makes constructed
   competitive.
4. **Static-ability framework + mana provenance** — broad correctness wins.
5. **Smarter AI blocking** (Tier 13) — biggest single-player upgrade.
6. Then the **Tier-4 mechanic sweep** and **Tier-3 object-model** features, batch
   by batch.
7. **Replays, spectator, social, accessibility** as the product matures.
