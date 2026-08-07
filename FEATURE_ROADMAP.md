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

- **Urza block complete** — `set_gaps.py usg`, `ulg` and `uds` all at zero
  (`sets::usg`, `usg2`, `usg3`, 246 USG cards). The echo, cycling-land,
  verse-counter, Aura recursion, Rune-of-Protection and Opal/Hidden/Veiled
  animation cycles, plus the utility shell. New primitives, by wave:
  `Effect::{BecomeCreatureLosingTypes, SetCardTypesTo}` (CR 205.1b — an
  animation that REPLACES the type line, reverted by timestamp),
  `Effect::GrantKeywordToMatchingThisTurn` (CR 611.2c),
  `Effect::DealDamageToEachPlayerPerPermanent`,
  `Effect::EachPlayerReturnsAMatchingPermanent`, `Effect::AtEndOfCombat`
  (CR 603.7a), `StaticEffect::WhileCondition` (CR 611.2 — the general
  predicate gate), `WardCost::SacrificeMatching`,
  `ActivatedAbility.half_life_cost` (CR 118.4), `EquipScale.exclude_host`,
  `CounterType::{Petal, Fungus}`, `StaticEffect::{ReduceDamageToYouBy,
  SetColorOfMatching}`, `DynamicPt::{ControllerLife,
  PermanentsControlledMatchingToughness}`; then
  `CardDefinition.state_trigger` (CR 603.8 — the general "When [condition],
  [effect]" state-triggered ability, latched per permanent),
  `DelayedKind::NextCleanupStep` (CR 514.3a — cleanup fires its step triggers
  before `do_cleanup`), `StaticEffect::{OpponentsPlayWithHandsRevealed,
  CyclingCostReduction, AddDamageFromColorSpells, LandsProduceColorInstead,
  PreventAllDamageToControllerFromOthersSources,
  ControllerAssignsAttackersCombatDamage (CR 510.1a),
  MayReplaceDrawWithRevealUntilKind (CR 121.2a)}`,
  `Effect::{PlayerReturnsPermanentUnlessPaysLife,
  ReturnCreaturesWithPowerGreaterThanHand, ExileLastCreatedTokensAtNextCleanup,
  SearchSameNameAs, ChooseColorThenDiscardMatching, PutAuraFromHandAttachedTo,
  RememberPermanentOnSource}`, `Selector::LeastToughnessAmongAll`,
  `PlayerRef::HighestLife`, `SelectionRequirement::IsSource`,
  `Value::ChosenNumberOfSource`, `Effect::PreventNextDamageDivided` (CR 615 —
  the prevention twin of `DealDamageDivided`),
  `Predicate::TriggerSourceIsSourcesChosenPermanent`,
  and `Keyword::{CantAttackUnlessGreaterPowerAttacks,
  CantBlockUnlessGreaterPowerBlocks}`. Correctness: CR 300.2a — a land card can
  only be *played*, never cast; CR 603.10 — `Value::CountersOn` reads the dead
  *subject's* LKI, not just the resolving source's. Tests in
  `classic_sets/{usg,usg2,usg3}`, `core_rules/{cr_recent54,cr_recent55}`.
  ULG / UDS (`sets::ulg`, `sets::uds`, 126 UDS cards) added: `Duration::
  WhileSourceTapped` (CR 611.2c), `Value::{PermanentsTappedThisEffect,
  CardsRevealedThisEffect}`, `PreventionTarget::Anything` +
  `Effect::PreventNextEventFromChosenSourceAnywhere` (Martyr's Cause),
  `Effect::{EachPlayerExilesHandDrawsSeven, IgnoreStaticFromSourceThisTurn,
  RevealAnyNumberFromHand, ExileAllCopiesOfTargetName, ExileAndReturnToOwner,
  DestroyEachMatchingWithManaValue, DestroyAllSharingNameWith,
  SkipPlayerDrawStep, DealDamageExcessTo}`, `StaticEffect::{
  MostPermanentsCantPlay, ControllerSkipsDrawStep}`,
  `ActivatedAbility.any_player` (CR 113.3d — surfaced end-to-end through
  `AbilityView.any_player` and the client's ability menu), and
  `CounterType::{Arrow, Infection}`, plus the closing wave's
  `Effect::{MayExileSelfThen, AttachAuraFromGraveyardTo,
  ExtraManaOnLandTapThisTurn, GuessColorCountInHand}`,
  `StaticEffect::UntapOnlyChosenTypeWhileUntapped` (Storage Matrix, gated in
  `do_untap`) and `WardCost::DiscardMatching`. Correctness: CR 120.4a excess damage is
  now a *split* (the creature takes exactly lethal) rather than full damage
  plus a bonus. Tests in `classic_sets/{ulg,uds}`, `core_rules/cr_recent53`.
- **Mirrodin (MRD) complete** (`set_gaps.py mrd` at zero). The
  primitives that closure added: `GameEvent`/`EventKind::{LibraryShuffled,
  TappedForMana}` (CR 103.2c / 605) behind a central
  `GameState::shuffle_library`; `Effect::{LookAtHand, ChooseStepToSkipThisTurn,
  ControlPlayerNextTurn, SearchExileThenTokensPerCard,
  BottomThenRevealUntilCreature, ExileTopGreatestManaValueTakesExtraTurn,
  GainAllActivatedAbilitiesOf, ExchangeControlWithSharedType,
  SearchRevealPunishSameNameCasters}`; `StaticEffect::{
  SpellTaxPerControllerPermanent, NoncreatureArtifactsAreCreatures,
  ProtectionFromExiledWithCardTypes}`;
  `Keyword::{CantAttackOrBlockUnlessPayPerCounter,
  CantBeBlockedIfDefenderControls}`; `CardDefinition::{
  entwine_additional_cost, equip_life_cost}`;
  `ActivatedAbility::exile_top_cost` (and its `condition` now sees the paid X);
  `EquipScale::count_host_attachments`; `ManaPayload::
  AnyTypeTriggerSourceProduces`; `PlayerRef::LowestLife`; `CounterType::Flood`.
  CR 723 player control ships end-to-end (state, `acting_seat_for`, server
  action routing, `PlayerView.controlled_by`); a hand you've looked at stays
  visible via `GameState.hands_revealed_to` and a client HUD chip. Tests in
  `recent_b/mrd`, `core_rules/cr_recent43`.
- **Mirrodin block complete** — MRD, DST and **Fifth Dawn (5DN)** all report
  zero `set_gaps.py` gaps. The 5DN closure (`decks::recent322`–`recent324`,
  ~120 cards) added: `Keyword::CantBeEquipped` (CR 702.6c),
  `StaticEffect::{MaxAttackersPerCombat, MaxBlockersPerCombat}` (CR 506.2 /
  509.1b — Silent Arbiter, honoured by the bot planner, the client's Attack
  All and a HUD chip), `StaticEffect::FiveColorAlternativeCost` (CR 118.9 —
  Fist of Suns, behind `effective_alternative_cost`),
  `StaticEffect::{PlayersSkipDraws, SharedFate}` (CR 121.2a draw
  replacements), `Effect::{GainControlWhileSourceTapped, DoubleUnspentMana,
  CounterAbilityAndDestroySource, RevealImprintDeployCreature,
  ImprintFromGraveyard, SpellweaverCopy, ReversalOfFortune,
  EachPlayerSacrificesUnlessDiscards, ReturnVictimAndAttachSelf,
  LiarsPendulum}`, `CardDefinition.sacrifice_when_you_control_no_other`
  (CR 603.8 state trigger), `ActivatedAbility.remove_counter_among_kinds`,
  `Value::SacrificedCount`, `SelectionRequirement::PowerAtMostYourCount`, and
  `PlayerRef::CounteredSpellController`. Correctness: CR 506.4 (a control
  change removes a permanent from combat) and CR 603.10a (a *static-granted*
  "when this dies" ability now fires from the death LKI snapshot — Endless
  Whispers). Tests in `recent_b/{fdn5,mrd}`, `core_rules/cr_recent44`.
- **Champions of Kamigawa opened** (`decks::recent325`, 40 cards —
  `set_gaps.py chk` 77 → 37): the Myojin cycle (divinity counters), the
  slow-dual land cycle, Forbidden Orchard, Untaidake / Hall of the Bandit
  Lord, Azami, Dosan, both Inames, Sachi, Shisato, The Unspeakable, Night of
  Souls' Betrayal, Mana Seism, Devouring Rage, Imi Statue, Orochi Hatchery,
  Tenza, Hankyu. New: `StaticEffect::MaxOneArtifactUntap` (CR 502.3 — Imi
  Statue) and `SpendRestriction::LegendarySpell` + `SpellKind.legendary`
  (Untaidake). Tests in `classic_sets/chk::gaps1`.
- **Saviors of Kamigawa** (`sets::sok` + `sets::sok2`, 123 cards —
  `set_gaps.py sok` 131 → 8): the Kirin spiritcraft cycle, Channel (an
  ability word — an activated ability with `from_hand` + `discard_self_cost`),
  Sweep, the Shinen channel cycle, the hand-size-matters shell, the Moonfolk
  land-bounce activations, the Ascendant flip cycle, and the Epic sorceries.
  New: `SelectionRequirement::ManaValueEqualsTriggerAmount`,
  `Effect::ReturnAnyNumberToHand` + `Value::PermanentsReturnedThisEffect`
  (Sweep), `StaticEffect::PumpPTByValue`, `ControllerMaxHandSizeIncreased`,
  `PreventAllDamageToYourCreatures`, an attacker filter on
  `CreaturesCantAttackController`, `EventKind::DealsDamage` /
  `DealsCombatDamage`, `EventScope::OpponentSourceDamagedYou`,
  `CardDefinition::flip_when_predicate` (CR 603.8 state-triggered flip),
  `Value::SpellsCastThisTurnTotal`, `CardInstance::damage_dealt_to_this_turn`,
  `Effect::BecomeBlocked` (CR 509.1h), `Effect::ExileTopUntilNonland` +
  `Value::LastExiledManaValue`, `EntersAsCopy::legendary`,
  `PlayerRef::MostCardsInHand`, `ActivatedAbility::sac_all_matching_cost`, and
  `Keyword::CantAttackOrBlockUnlessPayPerCardInEnchanterHand`.
  Tests in
  `classic_sets/sok` and `classic_sets/sok2`.
- **Betrayers of Kamigawa complete** (`sets::bok` + `sets::bok2`, 112 cards —
  `set_gaps.py bok` at zero). The `bok2` closure added: `Keyword::
  CounterFirstTargetingEachTurn` (the Glasskite cycle, enforced alongside Ward
  at the targeting hooks but firing on its own controller's spells too),
  non-mana Splice costs (`CardDefinition.splice_extra_cost` +
  `AdditionalCastCost::OpponentGainsLife`), `SelectionRequirement::
  SharesColorWithPermanentYouControl`, `WardCost::RemoveCounterFromPermanent`,
  `StaticEffect::{GenericAlternativeCostForFilter (CR 118.9 — Kentaro),
  ReplaceDrawWithLookN (CR 121.2a — Tomorrow)}`, `Effect::{
  ShuffleEverythingOwnedIntoLibrary, ExchangeCreatureControlWith,
  DoubleDamageFromSourceThisTurn, EnchantmentsBiteControllersAndHosts,
  CounterSpellDiscardSplicedNames}`, `Value::LastDiscardedManaValue`,
  `PreventNextFromChosenSourceToTeam.one_event`, and
  `DelayedTrigger.bound_subject` (a delayed body naming "that card" resolves
  at fire time — Shirei). Correctness: CR 702.6e (an equipment-GRANTED
  unattach cost detaches the granter), CR 710.1c/710.4 (a flip keeps its cost
  and colour; a flipped permanent reverts on any zone change) and CR 112.4 (a
  permanent spell pumped on the stack keeps the bonus). **Splice onto Arcane
  now ships end-to-end** — `HandAffordances.spliceable` /
  `ClientView.spliceable_hand`, the client's helper-tap picker under
  `HelperMechanic::Splice`, auto-aimed spliced clause targets, and bot
  candidates. Tests in `classic_sets/{bok,bok2}`, `core_rules/cr_recent50`.
- **Betrayers of Kamigawa, first wave** (`sets::bok`, 71 cards): the Genju cycle, the Spirit/Arcane spiritcraft payoffs, the
  soulshift bodies, the Samurai/bushido shells, and the utility spells. New:
  `Keyword::ProtectionFromMatching` (CR 702.16 — the general filtered form,
  gated at cast / damage / block, with the state-aware half in
  `block_barred_by_protection_filter`), `StaticEffect::{LegendRuleDoesntApply
  (CR 704.5j — Mirror Gallery), PreventAllDamageToAndFromEnchanted (CR 615 —
  Heart of Light), OpponentsWhoCastCantAttack, OpponentsWhoAttackedCantCast}`,
  `Effect::{EachPlayerNamesCard, EachPlayerRevealTopKeepIfNamed,
  ReturnSelfAttachedToChoiceOf}`, `ActivatedAbility.return_permanent_cost`, and
  `CastWithoutPayingImmediate.reduce_generic`. `EventScope::EnchantedBySource`
  now also matches `PermanentDied`, and an alternative cost's `exile_filter`
  resolves its X atoms against the declared X (the Shoal cycle's shape).
  The Baku and Shoal cycles ride existing plumbing (`ActivatedAbility.
  remove_counter_x`, `AlternativeCost.exile_filter`). Tests in
  `classic_sets/bok`.
- **Champions of Kamigawa complete** (`set_gaps.py chk` at zero — `sets::chk3`
  closed the last six). New: `Keyword::MayChooseNotToUntap` (CR 502.3, asked in
  `do_untap`), `Effect::GrantKeywordWhileSourceTapped` +
  `EffectDuration::WhileSourceTapped` (CR 611.2c),
  `Effect::{RevealLibraryNamedCountPunish, ExileHandThenReclaimLinked,
  SacrificeThenRevealUntilSharedType, AlternatingExileFromHand}`, and
  `StaticEffect::AllColorWordsBecomeChosen` (CR 612, four layer-3
  `ReplaceColorWord` rewrites over every permanent). Tests in
  `classic_sets/chk3`.
- **Ravnica block complete** (`set_gaps.py {rav,gpt,dis}` all at zero). RAV's
  last card, Master Warcraft, shipped with `GameState.combat_chooser` +
  `Effect::ChooseCombatThisTurn`: both declaration steps hand priority to an
  outside chooser, `declare_attackers`/`declare_blockers` gate their submitter
  on it, `CardDefinition.cast_only_before_attackers` enforces the printed
  timing, and `ClientView.declares_attacks`/`declares_blocks` drive every
  client combat gate.
- **Oath of the Gatewatch: 76 of 78 gap cards ship** (`sets::ogw::gaps{,2,3,4}`)
  — Cohort (via `tap_other_filter`), support, surge, the Oaths, Chandra
  Flamecaller, and the devoid shell. New engine:
  `Effect::EachDealsDamageEqualToPower` (Nissa's Judgment),
  `SpendRestriction::DevoidSpellsOnly` + `SpellKind.devoid` (Corrupted
  Crossroads), `StaticEffect::PlaneswalkersEnterWithExtraLoyalty` (Oath of
  Gideon), `Predicate::PlaneswalkerEnteredThisTurn` +
  `Player.planeswalkers_entered_this_turn` (Oath of Chandra), and
  `EquipScale.count_sharing_type_with_host` (Stoneforge Masterwork), and
  `StaticEffect::UntapSelfEachOtherUntapStep` (CR 502.3 — Endbringer), and
  `Predicate::OwnsSourceNamedCardInEveryZone` (Hedron Alignment's alternate
  win).
  Block modules: `sets::{rav::gaps21, rav::gaps22, gpt::gaps9, dis::gaps9}`
  (29 cards). Primitives from that sweep:
  `Effect::{ExileHandLinked, ReturnLinkedExilesToHand, LookExileAnyNumberRestBack,
  ExileFromGraveyardBecomeCopy, ReturnSameNameFromAllGraveyards, PutTopOnBottom,
  MayReturnSharingPermanentType, LookAtHandCastFree, ChangeTargetOfAbility
  (CR 115.7a/b), WarpWorld, ReturnSelfDeployBlocker,
  CopySpellForEachOtherLegalCreature, SearchOpponentLibraryForSameName,
  WishToLibrary, TokenUnlessOpponentLetsYouDraw, SearchAndCastFree,
  FlickerHostWithAuras, ReturnLinkedExilesToBattlefieldAttached,
  SacrificeEnchantedForExtraCombat, EyeOfTheStorm}`;
  `ActivatedAbility.unattach_cost` (CR 702.6 — Sunforger);
  `StaticEffect::{AnthemForColorSharedWithLibraryTop,
  OpponentsCantCastNamesExiledWithSource, CreatureSpellsMayPayExtraForCounters,
  YourISSpellsHaveReplicate, HasActivatedAbilitiesOfCounteredCreatures}`;
  `SelectionRequirement::{InCombatWithSource, SpellTargetsOnlySource}`;
  `EventKind`/`GameEvent::PermanentReturnedToHand`. Correctness: CR 103.7a
  (`skip_first_draw` was consumed by whichever draw step came first). Tests in
  `classic_sets/{rav,gpt,dis}`, `core_rules/cr_recent45`.
- **Darksteel (DST) complete** (`set_gaps.py dst` at zero). The primitives that
  closure added: `GameEvent::DamageDealt.from_card` +
  `EventScope::YourOtherSourceDamagedOpponent` (the printed "other than this
  permanent" exclusion), `TokenDefinition.dynamic_pt` on the attacking-token
  mint, `StaticEffect::{GrantColorless, PlayersMaySpendManaAsAnyColor,
  GainKeywordsFromExiledWith, ControllerCantCastCreatureSpells}`,
  `Effect::{TargetsExactlyX, PreventAllDamageFromTargetThisTurn,
  RedirectYourCombatDamageToTarget, AddCountersOfChosenKind,
  DamagedCreaturesDieThisTurn, CreatureDeathsDrainToughnessThisTurn,
  ReplaceControllerLossWithReset}`, and `target_slot_optional_x` threading the
  paid {X} through slot validation. Tests in `recent_b/dst`,
  `core_rules/cr_recent42`.
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

- **Darksteel/Fifth Dawn completion (modern_decks, this run — 100 cards).**
  `scripts/set_gaps.py dst` went ~90 → 15. New engine:
  `Keyword::Modular(N)` + `SelectionRequirement::HasModular` (CR 702.43),
  `Keyword::Sunburst` as a real CR 614.12 enters-with replacement off the
  cast's converge count (CR 702.44 — the five open-coded Sunburst cards
  migrated onto it, Pentad Prism's ETB trigger deleted),
  `SelectionRequirement::IsHostOfSource` (the mirror of `AttachedToSource`, so
  "{T}, Unattach this Equipment" taps the right creature),
  `Effect::AddCardTypeIndefinitely { until_eot }`,
  `Effect::LoseCardTypeUntilEot`, `Effect::PreventNextDamageWithCounters`,
  `StaticEffect::ReduceCombatDamageToControllerWhileUntapped`. Correctness:
  CR 704.8 — Persist/Undying now read the pre-SBA-sweep ±1/±1 pile.
  Cards in `decks::recent311`–`recent313`; tests in `recent_b/dst` and
  `core_rules/cr_recent41`.

- **Ravnica-block gap sweep (modern_decks, this run — 25 cards across RAV/GPT/DIS)**
  plus the CR 612 / 400 / 708 conformance pass. New engine:
  `Modification::AddColor` on a real continuous effect for Grave Betrayal's
  black Zombie (and Fractalize's printed type/colour rewrite now ships via
  `BecomeColor` + `BecomeCreatureType`), `StaticEffect::DiesToOwnersHandInstead`
  (Necromancer's Magemark) and `StaticEffect::AnthemForFilterIf` (a
  predicate-gated team anthem — Sword of the Paruns' tapped/untapped halves),
  `Keyword::AssignsDamageAsThoughUnblocked` (CR 510.1a — Predatory Focus),
  `Effect::{BottomHandThenDrawThatMany, LookTopExileOneOfN,
  EachPlayerKeepsNSacrificesRest, RevealTopDeployIfMatch,
  PreventAllDamageThisTurnWithCounters}`, `PreventionShield.counters_on_target`
  (Brace for Impact), `Value::{DamageDealtThisResolution, DamageTakenThisTurn}`
  + `Player.damage_taken_this_turn`, `CounterType::Plague`,
  `SelectionRequirement::PutIntoGraveyardFromBattlefieldThisTurn`
  (Gleancrawler), and `LandType::is_basic_type`. Fixes: CR 400.4a (a
  nonpermanent card can no longer be put onto the battlefield), CR 305.6/612
  (a basic's intrinsic mana ability follows its *computed* type line, so a
  rewritten Forest taps for blue and not green), `EventScope::EnchantedBySource`
  now covers block events, `evaluate_requirement_on_card` answers
  `IsEnchanted` / `IsEquipped` off live state, the CR 612 text-change auto
  picks are needs-aware instead of always White→Blue, and a divided-damage
  activated ability accepts a single target (CR 115.3). Client: the
  convoke/improvise/waterbend **helper-tap picker** (`HelperTapState`) closes
  the last M15-run UI gap; `ClientView.convokable_hand` + `KnownCard.
  has_convoke`/`has_improvise` back it. Tests in `classic_sets/{rav,gpt,dis}`
  and `core_rules/cr_recent40`. Wave 2 (+8 cards): Dream Leash, Auratouched
  Mage, Flame Fusillade, Pollenbright Wings, Chant of Vitu-Ghazi, Moonlight
  Bargain, Tunnel Vision, Concerted Effort — plus
  `Effect::{LookTopEachPayLifeOrBin, NameCardRevealUntilThenBin,
  ShareKeywordsAmongYourCreatures}` and an `EnchantedBySource` combat-damage
  trigger path (an Aura's "whenever enchanted creature deals combat damage to
  a player" now fires off the Aura, so `AttachedTo(This)` reaches the host).
  Tier-1 #1 gained a general **as-enters replacement**:
  `CardDefinition.as_enters_effect` resolves during the battlefield hop, before
  the first SBA sweep and before any ETB trigger, so a printed `*/*` body sized
  off that effect never dies as a 0/0 — with `Effect::TurnFaceDown` (CR 708.2a),
  `StaticEffect::SelfBasePtFromValue` (a state-driven CDA) and
  `Value::FaceDownCreatures`, this closes the last 🟡 engine row in
  `CUBE_FEATURES.md` (Ixidron).

- **BNG complete (modern_decks, this run — all 165 cards, `set_gaps.py bng` = 0)**
  plus **CR 303.4a "enchant player" Auras** (the Curse cycle + Psychic
  Possession). New primitives: `Effect::GainActivatedAbility { duration }` (the
  EOT activated-ability grant that also closed the Lorehold Apprentice /
  Evolution Vat duration approximations), `StaticEffect::ColoredCostReduction`
  ("costs {B}{R} less"), `Keyword::{CantBeBlockedByPowerLessThanCount,
  CantBeBlockedUnlessAllBlock, HexproofUnlessAttackingOrBlocking,
  AttackCostBounce}` (CR 509.1b / 508.1g), `CardInstance.attached_to_player` +
  `PlayerRef::EnchantedPlayer` + `StaticEffect::{EnchantedPlayerOneSpellPerTurn,
  DoubleDamageToEnchantedPlayer}`, `Effect::{RevealTopMayPutOntoBattlefield,
  PreventDamageToAndByUntilYourNextTurn, CounterSpellIfNameExiledWithSource,
  ExchangeControlWithTriggeringSpell, EachPlayerSplitsAndSacrificesRandomPile}`,
  `PreventionShield.redirect_to_player`, `SelectionRequirement::IsBestowed`,
  and `StaticEffect::PumpTeamByControlledPermanents.exclude_self`. CR 702.103f
  (bestowed Aura on an illegal host reverts to a creature) is fixed. Tests in
  `classic_sets/bng` and `core_rules/cr_recent37`.

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
  Convoke (CR 702.51 — each tapped creature pays {1} or one mana of a color
  it is) / Delve reduction; Commander tax; alternative (pitch) costs;
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
- **Lair lands (modern_decks — Planeshift):** `LandType::Lair` +
  `Effect::SacrificeSourceUnlessReturn` — "sacrifice this unless you return a
  non-Lair land you control to its owner's hand"; the bounce sibling of
  `SacrificeSourceUnlessSacrifice` (auto-picks the least useful land, a UI seat
  gets a real yes/no).
- **And/or kicker + the Flagbearer requirement (modern_decks — Apocalypse):**
  CR 702.32b "Kicker {A} and/or {B}" ships end to end —
  `CardDefinition.kicker_options`, `CardInstance.kicked_options`,
  `GameAction::CastSpellKickers`, `Predicate::SpellWasKickedWith`, a per-subset
  affordance (`ClientView.kicker_option_sets`), and bot + client casts that take
  the largest payable subset (the Volver cycle, Illuminate). CR 601.2c "must be
  chosen as a target" ships as `StaticEffect::FlagbearersMustBeTargeted`,
  enforced at cast and activation, preferred by the auto-targeter, and surfaced
  as `PermanentView.is_flagbearer`. Also: `Effect::SearchEachBasicLandType`,
  `Effect::ColoredManaBecomesThisTurn`, `Effect::SpellBecomesChosenColor` +
  `CardDefinition.color_override`, `Effect::OtherPlayerMayPayToCounter`,
  `Predicate::{TargetsHaveIdenticalColors, TargetSharesColorWithControlled}`,
  `SelectionRequirement::SharesColorWithSacrificed`,
  `DelayedTriggerKind::TargetsNextEndStep`, and `FlipCoinsChooseCount
  .stop_on_loss` (with the chosen count exposed as X).
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
- **Plot / trigger-subject primitives (recent309 — OTJ):**
  `ZoneDest::ExilePlotted` + `CounteredSpellZone::ExilePlotted` (CR 702.170 —
  the effect-granted half of plot, no plot cost paid: Kellan Joins Up, Jace
  Reawakened, Make Your Own Luck, Aven Interrupter);
  `StaticEffect::MayPlotFromLibraryTop` (Fblthp plots off the top for the
  card's own mana cost); `EventKind::YourInstantOrSorceryDealtDamageToPlayer` +
  `SelectionRequirement::ControlledByTriggerPlayer` ("target creature *that
  player* controls" — Satyr Firedancer);
  `StaticEffect::DoubleControllerLegendaryCreatureTriggers` (Annie Joins Up),
  with the whole subtype/supertype doubler family now applying on the cast ETB
  path as well as `fire_self_etb_triggers`;
  `Value::{OtherCreaturesOfTypeEnteredThisTurn, DistinctPowersAmongCreatures\
Controlled}` (Geralf, Selvala); `AffectedPermanents::All.owned_by_controller`
  ("creatures you control but don't own" — Laughing Jasper Flint).
- **Kamigawa CHK primitives (`sets::chk2`):** `Selector::AttachmentGranting`
  (CR 702.6e — a granted line naming its granting Aura/Equipment; Hankyu's aim
  counters), `StaticEffect::MaxOneUntapPerStep { filter }` (CR 502.3, generalizes
  the old `MaxOneNonbasicLandUntap` — Winter Moon + Imi Statue),
  `StaticEffect::PlayersCastOnlyOnOwnTurn` (CR 601 — Dosan, symmetric),
  `StaticEffect::PreventAllCombatDamageToAttached` (General's Kabuto),
  `StaticEffect::PumpPerBushido` (CR 702.44 — Takeno; per-permanent magnitude),
  `AnthemForFilter.all_players` (an anthem with no controller scoping),
  `SelectionRequirement::SharesColor/CreatureTypeWithAttachedHost` (Konda's
  Banner), `CardDefinition.attach_only_filter` (CR 301.5c, gates equip *and* the
  704.5n unattach sweep), `SpendRestriction::LegendarySpellsOnly` (Untaidake),
  `AdditionalCastCost::SacrificeAll` + `Value::SacrificedTotalPower` (Soulblast),
  `Effect::LookTopMayBottomAllElse` (Petals of Insight),
  `Effect::ReturnEachUnlessPays` (Cut the Tethers),
  `Effect::CreateTokenReturnSelfWhenItDies` + `DelayedKind::WhenTokenDies`
  (Tatsumasa), `StaticEffect::PlayersDrawExiledPlayable` (Uba Mask), and the
  `PlayerView.spell_cast_lock.off_turn_locked` flag behind the client's
  "⊘ off-turn" chip.
  `EventScope::FromYourGraveyard` now matches object events by the subject's
  controller (Blood Speaker), `EventKind::TappedForMana` matches `SelfSource`
  (Forbidden Orchard), and a self-scoped `SkipNextUntap` rider no longer stops
  an ability being a mana ability (CR 605.1a — the CHK slow duals).
- **Landfall / Rally / Awaken / Process (modern_decks — BFZ):** landfall now
  keys on a land *entering* rather than a land being *played* —
  `GameEvent::LandPlayed` gained a `played: bool` and is emitted from the
  battlefield-entry chokepoint too, so a fetched or reanimated land triggers
  Scythe Leopard and friends (the client log and the lands-played stat still
  read `played`). `shortcut::landfall` / `rally` / `rally_grant` are the shared
  trigger shapes. New primitives: `ActivatedAbility.process_cost` (Process as a
  real activation cost — Cryptic Cruiser), `Effect::LookTopKeepMatchingOnTop`
  (Fertile Thicket, Munda), `Effect::MoveWithinTotalManaValue` (March from the
  Tomb), `AdditionalCastCost::RevealFromHand` +
  `Value::RevealedForCostPower` (Titan's Presence),
  `Value::GreatestManaValueAmongPermanents` (Ugin's Insight),
  `SelectionRequirement::HasAwaken` (Halimar Tidecaller),
  `Effect::CopyForEachOtherTargetableCreature` (Zada, Hedron Grinder — a spell
  targeting only the source is copied once per other creature it could target).
  Emblem `AnthemForFilter` statics now reach the live anthem gather, so Gideon,
  Ally of Zendikar's −4 actually pumps. **Zendikar (ZEN) is complete**
  (`set_gaps.py zen` at zero). `sets::zen2` (96 cards) covers the seven
  board-state Traps, the Rally Allies, the landfall commons, the kicker
  creatures/spells, the Refuge land cycle and the small statics. New:
  `Predicate::{DamagedByCreaturesThisTurnAtLeast, LandsEnteredThisTurnAtLeast}`
  (+ a per-turn `Player.lands_entered_this_turn` tally),
  `StaticEffect::PumpTeamPerAttachmentOnSource` (Armament Master — an anthem
  scaled by the *source's own* attachments) and
  `CardDefinition.dies_to_library_bottom` (CR 614.6, the library sibling of
  `dies_to_exile` — Nissa's Chosen). Wave 2 needed no new engine: the
  Equipment triggers, the upkeep sacrifice-unless riders, Hellkite Charger's
  extra combat, Eldrazi Monument's three statics and Emeria's seven-Plains
  reanimation all ride existing primitives. `sets::zen3` closes the last 53:
  the three planeswalkers, Kalitas, Roil Elemental, Obsidian Fireheart,
  Eternity Vessel, Lullmage Mentor, World Queller, Gomazoa, Magosi, Oran-Rief,
  the Expeditions and both remaining Traps. New there:
  `EventKind::SpellCountered` (actor = the counterer),
  `Predicate::{CreatureSpellCounteredByOpponentThisTurn,
  NoncreaturePermanentDestroyedByOpponentThisTurn}`,
  `ActivatedAbility::{tap_permanents_cost, bounce_self_cost}`,
  `StaticEffect::MayLookAtOwnLibraryTop`,
  `Effect::RevealHandDiscardAllMatching`,
  `SelectionRequirement::IsSourceChosenCardType`,
  `Predicate::LastDiscardedWasColor`, and `CounterType::{Eon, Blaze}`.
  Tests in `classic_sets/zen2` + `classic_sets/zen3`.
- **Magic 2011 (M11) complete** (`set_gaps.py m11` at zero — `sets::m11::gaps`,
  60 cards): the Leylines
  (`OpeningHandEffect::StartInPlay`), the Servants, the Auras, Mitotic Slime's
  nested token deaths, Hoarding Dragon's linked exile, Stormtide Leviathan's
  world-flood, Mystifying Maze and the commons. New:
  `Keyword::{CantAttackUnlessLandCount, CantAttackUnlessOpponentDamaged}`, a
  `tapped` rider on `Effect::ExileReturnToOwnerNextEndStep`, and
  `StaticEffect::YourColorSpellDamageDoubled` (Fire Servant — the resolving
  spell's card types now ride `resolving_source` alongside its colors),
  `Effect::RandomHandCardDeployOrCastFree` (Wild Evocation) and
  `CardDefinition.sacrifice_when` — a general CR 603.8 "when [condition],
  sacrifice this" state trigger checked once per SBA pass (Phylactery Lich,
  plus `CounterType::Phylactery`) and `Effect::MassPolymorph`. The last three
  added `StaticEffect::{OpponentsWhoCastCantAttack, OpponentsWhoAttackedCantCast}`
  (Angelic Arbiter — gated in `declare_attackers` / the cast dispatch),
  `Effect::{EachPlayerNamesCard, EachPlayerRevealTopKeepIfNamed}` (Conundrum
  Sphinx — names stashed in `GameState.names_this_resolution` so every seat
  names before any reveal) and `Effect::ReturnSelfAttachedToChoiceOf` (Necrotic
  Plague); `SelectionRequirement::IsHostOfSource` is now source-precise in the
  static-grant walker instead of always answering `true`. Tests in
  `classic_sets/m11`.
- **Worldwake is complete** (`set_gaps.py wwk` at zero — `sets::wwk`
  rides the same landfall/Rally shapes plus Multikicker and
  `Effect::SwitchPT` / `BecomeCreature`; `sets::wwk2` closes the last 43).
  The closure added: the **Trap alternative cost** (`Predicate::{
  CastSpellThisTurnWith, CreatureEnteredThisTurnMatching}` on the existing
  `AlternativeCost.condition` gate, backed by a per-turn
  `Player.spell_casts_this_turn` profile list),
  `Effect::PreventNextFromChosenSourceToTeam` +
  `PreventionTarget::PlayerAndPermanents` (CR 615.7 — one shared "next N" pool
  around a seat *and* its permanents, redirecting the soak; Refraction Trap),
  `StaticEffect::WhileCountersAtLeast` (CR 611.2 — the Quest cycle's
  counter gate, peeled by a single `GameState::active_static` helper that
  replaced three ad-hoc unwrap loops), `Effect::{MustBlockTarget,
  DestroyThenVictimControllersMakeToken, TapAnyNumberThenCounters}`,
  `RevealMissDest::WithFind` (Treasure Hunt),
  `SelectionRequirement::SpellWithSingleTarget`, and
  `Value::CastSpellTimesKicked`. Correctness along the way: the Multikicker
  cost is now paid and stamped **before** the cast pipeline fires spell-cast
  triggers (CR 601.2f/h — Rumbling Aftershocks read a kick count of 0);
  `Value::TimesKicked` reaches a resolving *spell* via `EffectContext.
  kick_count` (Spell Contortion); an **animated land that dies is a creature
  dying** (CR 613.2 layer 4 — `GameState::permanent_is_creature` at the
  destroy/sacrifice funnels, so the Zendikons hand their land back);
  a "…deals combat damage to a player" trigger carries the damaged seat
  through resolution (`StackItem::Trigger.trigger_player`) so
  `ControlledByTriggerPlayer` target filters survive the CR 608.2b re-check;
  and a trigger's multi-target fan-out now peels `MayDo` / `CapTargetsAt` /
  `OptionalTargets` wrappers (Terastodon, Voyager Drake).
- **Player status / team primitives (modern_decks — MMQ):**
  `StaticEffect::ControllerHasShroud` (CR 702.18 — Ivory Mask; unlike hexproof
  it blocks the controller's own targeting and no ignore-hexproof static
  pierces it); CR 810.5/810.8d shared team **poison** (`effective_poison` /
  `poison_loss_threshold`, fifteen for a 2HG team) — both surfaced through
  `PlayerView` alongside the shared life pool, which the view had been reading
  past. `assign_teams` preserves the shared pool on a re-partition.
- **Combat / prevention / cost primitives (modern_decks — MMQ wave 5):**
  `StaticEffect::PreventAllCombatDamageToAndFromYourCreatures` (Statecraft — a
  controller-scoped both-directions combat seal read at both combat funnels)
  and `Effect::PreventNextDamageFromSourceThisTurn` (Barbed Wire — the
  deal-side mirror of `PreventNextDamage`, a floating shield keyed to the
  source). `AlternativeCost` gains `tap_creatures` (Orim's Cure, Ramosian
  Rally) and `opponent_gains_life` (Invigorate), both surfaced in the cast
  prompt and gated in the hand view. `Effect::SacrificeSourceUnlessPayValue`
  (Megatherium — the `Value`-scaled sibling of `SacrificeSourceUnlessPay`),
  `Effect::{AddAttackTaxThisTurn, AddBlockTaxThisTurn}` (War Tax / War
  Cadence — symmetric turn-scoped tolls charged to the *acting* player,
  surfaced in `ClientView` and the HUD's combat chip),
  `StaticEffect::PlayerDamageBecomesExileFromLibrary` (CR 614.1b — Crumbling
  Sanctuary), `Predicate::AllMatchingShareAColor` +
  `AnthemForFilterIf.all_players` (Common Cause),
  `Selector::ChosenCardInHand` (Assembly Hall),
  `Effect::{ShuffleAnyNumberFromHandThenDraw, EachPlayerRevealTopNKeepLandsExileRest}`
  (Credit Voucher, Clear the Land), `Effect::MayDoBy` ("that player may …",
  routed to another seat — Ley Line),
  `DynamicPt::PermanentsOfChosenColorOpponentsControl` (Chameleon Spirit) and
  `SelectionRequirement::HasChosenColorOfSource` (Story Circle),
  `StaticEffect::CreaturesYouControlAreChosenType` (Conspiracy — the
  creature-subtype sibling of Realmwright's land-type replacement) and
  `StaticEffect::NoSpellOrNonbasicLandSharingAPermanentName` (Cornered Market,
  gated at both the cast and land-play funnels) and
  `Effect::EachPlayerRevealTopAllEnterIfAllCreatures` (Game Preserve). Brawl
  and Shoving Match ride the existing `Effect::GainActivatedAbility` over
  `EachPermanent(Creature)`. Correctness:
  a triggered ability now carries the seat that **caused** its firing event
  (`TriggerCandidate`/`PendingTriggerPush.actor`, stamped from `event_actor`,
  read through `PlayerRef::TriggerEventPlayer`), so "that spell or ability's
  controller" on a `BecameTarget` trigger resolves to the caster instead of the
  targeted permanent's controller (Lava Runner).
- **Aura / counter / cost primitives (modern_decks — MMQ):** directional aura
  damage seals (`PreventAllDamageTo`/`ByEnchanted`, splitting Heart of Light's
  both-ways static); `EquipScale.count_all_controllers` + `.exclude_source`
  (Ancestral Mask); `CounterType::{Storage, Depletion}`;
  `GameState::apply_printed_etb_counters` — a land played from hand now gets its
  printed `enters_with_counters` (CR 614.1c), which only the cast and move paths
  applied; and inline-resolving **mana abilities** read the activation's X
  (`continue_ability_resolution_x`), so "remove any number of storage counters:
  add that much mana" works. `StaticEffect::AnyPlayerSpellsHaveFlash` (Vernal
  Equinox) and `EachPlayerMayPutPermanentFromHand.others_only` (Hunted Wumpus).
  `Selector::MatchingAmong { inner, filter }` narrows another selector's result
  set (Deathgazer's "the *nonblack* creature blocking or blocked by this");
  `StaticEffect::NoPlayerCanPlayLands` is the symmetric sibling of
  `ControllerCantPlayLands` (Territorial Dispute); and
  `Value::CardsMilledThisEffectMatching` is the filtered sibling of
  `CreatureCardsMilledThisEffect` (Saprazzan Breaker).
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
- **Prevention / cost / CDA primitives (modern_decks — MMQ closure + NMS):**
  `Effect::PreventNextDamageFromChosenSource` gained `to` / `gain_life` /
  `redirect_to` (Charm Peddler, Cho-Arrim Alchemist, General's Regalia) and now
  rechecks the chosen source's colour at damage time (CR 615.9), with CR 609.7b
  last-known-information for a source that has left every zone;
  `ActivatedAbility::{generic_cost_value, exile_permanent_cost}` +
  `Value::ExiledForCostManaValue` (Bargaining Table, Food Chain);
  `EventKind::OpponentCausedYouToDiscard` (Spiritual Focus);
  `Effect::{ChooseCardTypeRevealHandDamage, CoinFlipDestroyLoop, ThievesAuction,
  LoseLifePerControlled, MayDealPowerThenNoCombatDamage}`;
  `DynamicPt::{CreaturesOfSourceChosenType, LandsOfTypeInPlayPower}`;
  `PlayerRef::MostCreatures`; `Value::CardsNamedLikeSourceInAllGraveyards`;
  `SelectionRequirement::{IsSourceChosenCreatureType, SameNameAsTarget}` with
  `resolve_chosen_creature_type` / `resolve_target_name` concretizations;
  `CounterType::Winch`. `PlayerView`/`PermanentView` gained
  `prevention_next_instances` + `prevention_source_names` so a one-event shield
  no longer reads as a blanket fog.
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
  Everything}` — Hexdrinker). Voting (CR 701.38 — `Effect::Vote` with
  `VoteTally::{Majority, PerVote}` for will-of-the-council / council's-dilemma
  ballots, plus `WillOfTheCouncilExile` for Council's Judgment's permanent
  vote); `Decision::ChooseOption` renders the printed words in the client and
  `GameEvent::Voted` logs each vote.
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
- **Combat-memory / mana-replacement / token-lineage primitives (modern_decks —
  NMS closure):** `GameState.blocks_declared_this_turn` +
  `Selector::CreaturesBlockedBySourceThisTurn` (the blocked set survives the
  blocker's death, so an end-of-combat sweep still sees it — Defiant Vanguard);
  `Effect::ReplaceLandManaThisTurn` + `GameState.land_mana_replacements_this_turn`
  (CR 614 turn-scoped land-tap replacement, seat- and nonbasic-scoped — Pale
  Moon, Harvest Mage); `CardInstance.created_by` stamped from the resolving
  source with `Selector::{TokensCreatedBySource, CreatorOfSource}` (Saproling
  Burst's counter-scaled tokens and its LTB sweep);
  `Effect::{ExileTopThenRevealUntilNamed, RevealChosenCardsLowestCreaturesEnter,
  AttackingCreaturesBecomeBlocked}`; `LookPickToHand.rest_to_exile`;
  `PreventNextDamageFromChosenSource.whole_turn`;
  `PreventAllDamageThisTurn.redirect_to`;
  `Value::PermanentCountControlledByMatching` (battlefield-state filters read
  live, so `Untapped` works); and the CR 502.1 fix that only a permanent
  printing "you may choose not to untap this" holds itself tapped to keep a
  lock alive (Kill Switch untaps and releases).
- **Cost / tax / floating-watcher primitives (modern_decks — Prophecy waves
  1-3 + the STX/SOS approximation sweep):**
  `CardDefinition.self_cost_reduction_if` (a general predicate-gated flat
  discount — the whole Avatar cycle in one field);
  `AlternativeCost.discard_filters` (CR 601.2b "discard a [filter] card
  rather than pay this spell's mana cost", paid through the normal discard
  funnel — Abolish, Foil, Flameshot, Outbreak, Snag);
  `WardCost::GenericXFromCost` and `Effect::UnlessPlayerPays.if_paid` (the
  Rhystic cycle's "{X}" toll and its mirror branch);
  `Effect::TurnOffDamagePreventionThisTurn` + `CardInstance.damage_prevention_off_eot`
  (the Glittering cycle's any-player escape hatch);
  `Effect::EachPlayerSacrificesDownTo` (Keldon Firebombers);
  `Keyword::{CantAttackIfDefenderHasUntappedLand, CantBlockIfYouHaveUntappedLand}`;
  `Effect::GrantTriggeredAbilityThisTurnToMatching` +
  `GameState.turn_granted_triggers` (CR 611.2 floating "this turn, whenever a
  [filter] …" watchers that reach permanents entering later — Mage Hunters'
  Onslaught); `SelectionRequirement::NotSacrificedThisResolution`; and
  `Value::DistinctNamesControlledMatching` (generalizes the old
  lands-only distinct-name count). Engine fixes: `Effect::NameCreatureType`
  works for a resolving instant/sorcery, `Value::TotalCountersOn` shares
  `CountersOn`'s LKI/zone fallback chain and counts keyword counters, and
  `Keyword::CantBeBlockedIfDefenderControls` is mirrored into
  `blocker_can_block_attacker`.
- **Masques block complete, and the Apocalypse opening (modern_decks —
  Prophecy wave 4 + APC waves 1-2):** MMQ / NMS / PCY all report zero
  `set_gaps.py` gaps. The Prophecy closure added
  `Keyword::AttackBlockCostTapAnother` (CR 508.1g/509.1b — a tap-a-spare-body
  attack/block cost, honoured by `legal_attackers` / `legal_blockers` too),
  `StaticEffect::ActivationAdditionalSacrifice` (CR 602.5b — a static that
  bolts "Sacrifice a [filter]" onto matching permanents' activations),
  `StaticEffect::GrantKeywordWhileControllerControlsAtMost` (a grant gated on
  each recipient's *own* controller's board),
  `Effect::{HighestLifeWinsElseDraw, ExileTokensSharingNameWith,
  RedirectNextDamageBackAtSource}` and `CounterType::Omen`. APC added
  `CardDefinition.opponent_discard_deploys` (CR 614) and
  `Effect::BecomeChosenCreatureType`. Also new: CR 702.161 living metal and
  CR 702.162 / 701.28 convert (`AlternativeCost.converted` — `sets::bot`).
  Structural: the eight attacker-independent blocker gates now live in one
  `blocker_side_gates_allow_block`, shared by the affordance scan and the
  per-pair check (the scan had been missing all of them, Decayed included).

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
   path (CR 614.16), via `scaled_counter_count_on`. The draw branch is closed
   (skip, exile-and-play, redirect, doubling, empty-hand bonus, dredge,
   `MayReplaceDrawWithTutor` and `MayReplaceDrawWithRevealUntilKind`). Still to generalize: as-a-copy ETB. A *general* as-enters one-shot now
   ships (`CardDefinition.as_enters_effect`, resolved pre-SBA — Ixidron). (Devouring Hellion / Rescuer Sphinx's
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
5. ✅ **Copy of a permanent (clone).** `Effect::BecomeCopyOf` +
   `enters_as_copy` ship Clone, Phantasmal Image, Mirror Image, Stunt Double;
   token copies via `CreateTokenCopyOf` (CR 707.2 copiable values only, CR
   707.2e non-legendary rider — Helm of the Host); continuous "becomes a copy"
   via `BecomeCopyOfFor` (Mirrorform, Vesuva).
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
  "choose targets as it resolves". The prevention sibling ships as
  `Effect::PreventNextDamageDivided` (Serra's Hymn), sharing the same
  `Decision::DivideDamage`.
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
  "if excess damage was dealt this way" (Orbital Plunge). **CR 120.4a
  redirection ships**: `Effect::{DealDamageExcessToController, DealDamageExcessTo}`
  split the event before it happens, so the creature takes exactly lethal
  (deathtouch-aware via `lethal_damage_needed`). Remaining: the broader
  marking-interplay audit.)
- ✅ **Prevention funnel is single-entry** — CR 615.5/615.8: chosen-source
  shields (`damage_prevented_sources`, carrying a life-gain beneficiary and a
  one-instance flag) are applied inside `apply_prevention_shields`, and combat
  no longer short-circuits a fully-prevented dealer, so a shield's riders fire
  on combat damage and `DamagePrevented` is emitted uniformly (615.13).
  Hallow (turn-long + life refund), Awe Strike (next instance only).
- ✅ **Attack restrictions by board state** — `CantAttackUnlessLandCount`
  (Harbor Serpent's five Islands) and `CantAttackUnlessOpponentDamaged`
  (Bloodcrazed Goblin) join the CR 508.1a gate list in `declare_attackers`.
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
- 🟡 **Ante / conspiracy / sticker / attraction** zones. Ante ✅ (CR 407 — see
  "Recently closed"); dungeons ✅ (CR 309); **attractions ✅** (CR 717 — the
  command-zone Attraction deck + junkyard, `Effect::OpenAnAttraction`, the
  precombat-main roll-to-visit turn-based action, and Visit triggers keyed to
  each card's lit-up numbers). Remaining: conspiracy, sticker (CR 123).
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
- ✅ **Minimum-cost floor** (`StaticEffect::SpellCostFloor` via
  `apply_spell_cost_floor`, applied after every reduction — Trinisphere) and
  **cost-increase statics** (`extra_cost_for_spell` walks nine flavours plus
  `ColoredSpellTax` and the turn-scoped pool).
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
- ✅ **Snow-mana-only** (`ManaSymbol::Snow`, paid from the snow pool with
  `ManaError::InsufficientSnow`). Remaining ⏳: **mana-value-X** cost gates.

## Tier 6 — Combat fidelity

- ✅ **Damage assignment order** (Tier-1 #3) + **trample math** with
  multiple/deathtouch blockers — `default_damage_split` assigns lethal in
  order (deathtouch lethal = 1, CR 702.2e) and tramples the remainder
  (CR 510.1c/702.19g). Tests: `cr_702_2e_trample_deathtouch_*`,
  `cr_702_19g_*`.
- ✅ **Banding** (CR 509.2 / 510.1c / 702.22) — a banding blocker routes the
  attacker's combat-damage order + assignment to the *defending* player
  (Benalish Hero), and **attacking bands** ship:
  `GameAction::DeclareAttackersBanded` validates 702.22c/d, `attack_bands`
  persists the band (702.22e), removal from combat drops a member (702.22f),
  and a block on any member spreads across the band (702.22h). Surfaced to
  clients via `ClientView.attack_bands`. **"Bands with other [quality]"**
  ships as `Keyword::BandsWithOther(SelectionRequirement)` — band legality
  without plain banding (702.22d), the defender's damage division against a
  two-strong quality band (702.22j), the active player dividing a band
  blocker's damage (702.22k), and a payload-agnostic removal so "loses all
  'bands with other' abilities" (Shelkin Brownie, Tolaria) works.
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
  (`CantAttackOrBlockUnlessDelirium` — Patchwork Beastie), descend-gated
  (`CantAttackOrBlockUnlessDescend(n)` — The Ancient One, via `descend_count`)
  and cost-gated (CR 508.1g / 509.1d–f — `CantAttackOrBlockUnlessPay(n)`,
  Oppressive Rays; charged to the attacker's/blocker's own controller from the
  Propaganda tax pool). Open: granted must-attack with future-turn duration,
  multiplayer goad-target clause.
- ⏳ **Planeswalker / Battle as attack targets** UI + redirection.
- ✅ **Goad**, **Lure**, **Provoke**, **Ninjutsu swap**.
- ✅ **Multiplayer attack options** (CR 802 / 803) — `GameState.attack_option`
  picks between "every opponent is a defending player" (the Free-for-All
  default) and attack-left / attack-right, which narrow the legal defender to
  the nearest living opponent in that direction (a dead neighbour means no
  legal attack at all). Surfaced as `ClientView.attackable_players` and honored
  by the client's attacker-pick highlight. Tests `cr_802_*` / `cr_803_*`.
  ✅ **CR 801 limited range of influence** — a per-seat `range_of_influence`
  with a turn-start `range_matrix` snapshot (801.2/801.2c), enforced on
  attacks (801.3), targeting (801.4), activation (801.6) and effect fan-out
  (801.10); surfaced as `PlayerView.in_your_range`. ✅ **CR 809 Emperor**
  (`set_emperor_variant` — seating, 2/1 ranges, deploy creatures,
  adjacent-only attacks, a team falling with its emperor) and ✅ **CR 811
  Alternating Teams** (`set_alternating_teams`). Remaining ⏳: CR 807's
  rotating Grand Melee ranges.

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
- ✅ **Floating life deltas**; per-turn life-history sparkline (`I` toggles a
  per-seat, one-column-per-turn panel — `game_ui::life_graph`).
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
- 🟡 **Highlight legal plays** — `ClientView` carries castable/pitchable/kickable/spliceable
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
  combat so attacks-while-saddled riders fire. **Attacking into open mana is
  now respected**: the adopted default (`attack_search_sim`) lets both seats
  cast spells inside the attack/block simulations, so the crack-back removal
  and the defender's tricks are visible at declaration time — adopted at
  54.4 % [53.0, 55.8] over 4 794 fixed+cube games with dimir control (the
  blind search's documented −5.2 archetype) the biggest winner at 61.3 %.
  Race math (`atk-race`: an attack sim that ends inside burn range, any
  life ≤ 10, extends one turn cycle) is measured and **not adopted**:
  the pre-registered 4× decision run read 50.2 % [49.5 %, 51.0 %] over
  19 200 fixed+cube games — the first decider's +1.2 (with mono-red at
  54.8 %/400) collapsed to +0.2, mono-red back at 49.9 %, the
  block-search replication failure reproduced on a fresh hypothesis.
  Kept as a documented profile (`attack_search_race` doc).
  **Multi-blocker math landed and is adopted**: value gang-blocks
  (`gang` / `block_gang`, now `EvalWeights::default`). The greedy pass
  gangs only under lethal threat and `block_search` could only ever
  *remove* blockers, so "two 2/2s eat a 4/4 at 20 life" was in no search
  space the bot had; gangs are now candidates the block sim prices (dead
  blockers against dead attacker). Two independent 28 800-game sealed
  runs: 51.3 % [50.7, 51.9] and 51.1 % [50.5, 51.7], after a 9 600-game
  screening at 51.0 % — the only one of five play-side profiles tried in
  this push whose edge did not shrink at 3× the sample. Adopting it also
  switches on `block_search`, which measured null alone: the search had
  nothing to find while its only candidates were "block with one fewer
  creature".
- 🟡 **Planeswalker piloting** — emblem values are priced by what the
  emblem actually does (draw/damage/drain/token/lifegain shapes, static
  buffs, clamped 20–60) instead of a near-flat constant, and a doomed
  walker cashes out: when enemy board power covers its loyalty, the
  ability pick keeps only loyalty-spending finalists, so the bot takes
  the removal/ultimate now rather than plussing into a free kill.
- 🟡 **Per-card attribution is now within-archetype**
  (`CardAttribution::stratified_delta`). The raw in-minus-out delta is a
  *cross-archetype marginal*, not a card grade: a black card is played by
  the black builds and benched by every white, blue and red one, so its
  "out" group is a different deck rather than the same deck without it.
  That is what made Professor Dellian Fel read −2.4. Attribution is now
  pooled across colour-identity strata by inverse variance, and a card no
  stratum both plays and benches reports **no** within-archetype number
  rather than passing the marginal off as one. `recommend_pool` prints
  `within` first and labels `raw` as the confound.
- 🔴 **Play net as an evaluator: documented dead end.** Ten gate rounds
  across every lever available, and it has never won. Recorded here so
  the next person does not re-derive it.

  The strange part is that it is now the *better predictor* and still the
  worse player. On identical fresh positions the attention net scores AUC
  0.798 / log-loss 0.551 against `eval_material`'s 0.760 / 0.574, and it
  replicates on a second seed (0.761 vs 0.747). Then:

  | profile | win rate vs `gang` |
  |---|---|
  | `net` (replacement) | 44.8 % [43.7, 45.9] |
  | `net-blend` | 48.8 % [47.8, 49.8] |
  | `net-q10` / `net-q20` | 44.4 % / 44.4 % |
  | `netb-q10` / `netb-q20` | 48.0 % / 48.9 % |

  **Three explanations proposed, two tested, both refuted:**

  1. *"AUC is global, the search needs local discrimination, so the net
     must be worse locally."* `--pairwise` says no — on adjacent same-game
     snapshots the net orders 54.3 % of separated pairs correctly against
     the heuristic's 51.7 %. It is slightly **better** locally.
  2. *"The net manufactures differences: it separates 100 % of adjacent
     pairs where `eval_material` ties on 46.9 %, so an argmax search
     follows its noise."* Quantising the output onto a 0.1 / 0.05 grid
     makes it tie exactly like the heuristic — and moves the win rate by
     less than a point in either direction. Refuted.
  3. *Untested:* **distribution mismatch.** Every diagnostic samples the
     snapshot cadence (turn start / postcombat main / end step), but the
     search evaluates *simulated leaves* inside `simulate_attack_outcome`
     — a distribution the net is neither trained on nor measured at. A
     net better on snapshots and worse on sim leaves would produce
     exactly this pattern, and every instrument built so far would show
     the former while the gate measures the latter. Testable by pulling
     calibration positions from inside the search.

  Levers already exhausted: data volume, window reuse, capacity (round 4),
  snapshot coverage, target shape (MC → TD(λ)), architecture (pooling →
  attention), and output shaping (quantisation). Making the net a
  strictly better predictor did not make it a better player at any point.

  **Where the evidence points instead:** the *deck* net, which clears the
  house bar (61.7 %, 60.7 %) and is under-exploited. A decklist genuinely
  is an unordered set, so bag-of-cards is the right prior; a board state
  is a set of matchups, so it is the wrong one. Same architecture,
  opposite verdicts — see [`selfplay_train --use-deck-best`] and
  `CRAB_DECKNET=… recommend_pool`.
- 🟢 **Value net finally beats the heuristic as a predictor** — and the
  reason six gate rounds failed was **overfitting, not architecture**.

  `selfplay_train --calibrate N` scores the net and `eval_material` as
  *predictors of the winner* on identical fresh positions (log-loss /
  Brier / AUC, plus an output histogram). It answered in minutes what
  thousands of gate games never did:

  | | pooled λ0.7 3.7M rows | pooled λ0.7 9.1M rows | **attention** 9.1M rows | `eval_material` |
  |---|---|---|---|---|
  | AUC | 0.7369 | 0.7805 | **0.7978** | ~0.753–0.761 |
  | log-loss | 0.7473 | 0.5912 | **0.5505** | ~0.571–0.574 |
  | Brier | 0.2384 | 0.2007 | **0.1859** | ~0.196–0.197 |
  | outside [.05,.95] | — | 16.1 % | **9.8 %** | — |

  Decomposed: **data volume + lower window reuse is worth +0.044 AUC**
  (2.5× more fresh games at 1.68× reuse instead of 4.16×), and
  **attention adds +0.017 on top**. Overfitting was the dominant effect
  and the architecture the smaller half — the reverse of the working
  hypothesis.

  This retro-invalidates the earlier gates: all six trained a ~481 k-param
  net on a memorised 250 k window at 4.2× reuse, so the 42–45 %
  replacement results measured an overfit net, and round 4's "capacity is
  the bottleneck" conclusion came from a run that could not have shown a
  capacity effect. **Training MSE is not progress here** — 0.017 at λ=1
  was memorisation, and out-of-sample log-loss was *worse than predicting
  0.5 every time* (1.1210 vs 0.6933).

  Two failures, separately fixed. *Calibration*: MSE on hard 0/1 targets
  rewards large logits, pinning 70 % of positions in the extreme bins and
  handing the search a flat landscape where every candidate line scores
  the same — a better ranker made into a worse player by the shape of its
  output. Soft TD(λ) targets plus more data cut that to 9.8 %.
  *Knowledge*: fixed by data volume first, attention second.

  Caveats before anyone invests: single seed (replication running), and
  AUC is not win rate — better prediction still has to survive the ladder.
- 🟡 **Value-net rework** — three changes, none yet gate-measured:
  bootstrapped **λ-returns** (`SampleWindow::relabel_lambda`, shard v3
  carries trajectory + ply; λ = 1 reproduces the historical Monte Carlo
  target exactly, so every prior gate round stays reachable), because
  labelling a turn-2 state with the winner of a twenty-turn game is
  mostly labelling noise; **opening-move exploration** in
  `play_recorded_game`, because both seats played the same deterministic
  policy and the net only ever saw the band of positions that policy
  reaches; and `--calibrate`, which scores the net and `eval_material` as
  *predictors* (log-loss / Brier / AUC, plus an output histogram) on
  identical positions. Four gate rounds answered "is the net-piloted bot
  stronger" expensively without ever answering "does the net know more
  than the heuristic does" — and those have different fixes. A saturated
  sigmoid would make a better predictor into a worse player by handing
  the search a flat landscape, which the histogram is there to catch.
- 🟡 **Build net has a consumer** (`selfplay_train --use-deck-best`). The
  deck net cleared the house bar twice (61.7 %, 60.7 %) and nothing read
  the result back: every training game was still played with heuristic
  builds. Actors can now judge best-of-32 candidates with it.
- 🟡 **Sealed builder repaired** (`SimConfig::builder_v2`, the previous
  builder kept as the control) — three defects found together while
  investigating why a pool's bomb never appeared in a build: the card
  scorer had **no body, keyword or ability terms at all** (it ranked a
  {3}{U}{U} 5/5 flier with ward 2 *below* a vanilla {U}{U} two-drop),
  splash candidates weren't pip-limited (so a double-pip bomb got
  "splashed" off three sources), and basics were split by linear pip
  demand (so double costs were under-served). Now: `draft::card_quality`
  (body, evasion/deathtouch/lifelink/ward, and a `prepare_spell` bonus —
  a preparation card is two cards in one slot), single-pip splashes, and
  squared pip demand in the mana split. **Adopted**: 56.9 %
  [54.1, 59.7] and 58.5 % [55.7, 61.3] on independent seeds over 1 200
  head-to-head games each vs the builder it replaces, same pools and
  pilots (`selfplay_train --gate-builder-v2`).
- 🟢 **Paired ladder sampling** (`bot_ladder --paired`, the default;
  `--unpaired` is the control). Each shuffle is played twice with the
  seats swapped, so deal luck *cancels within the pair* instead of being
  averaged away across thousands of games. Under a true null 2 032 of
  2 400 sealed pairs split — a direct measurement that only ~13 % of this
  ladder's games were ever decided by anything a profile could influence.
  Realized within-pair correlation −0.63 … −0.74, so 14 400 paired games
  carry the precision of ~35 000–40 000 unpaired ones; the efficiency is
  measured and printed, not assumed. Also seeds the bot's tie-break
  jitter (`bot::set_jitter_seed`) — `--seed` never made a run
  reproducible before, and under a null that jitter was the only thing
  that could break a pair (rho −0.694 → −0.735). The residual is
  engine-level randomness inside card effects.

  Re-measured at ~4× resolution against the current default, 14 400
  games each (seed 43): `landseq2` 50.4 % [49.9, 50.9], `mull2` 49.8 %
  [49.3, 50.3], `look1` 50.4 % [49.9, 50.9] — three nulls **confirmed**
  rather than overturned, which is the useful outcome: those rejections
  were correct, not underpowered. `race2` 49.3 % [48.8, 49.9] is the one
  reversal, mildly *harmful* where the unpaired run read 50.2 %.
  `look2` (two plies of sequence lookahead) read 50.6 % [50.1, 51.1] on
  seed 43 and **did not replicate**: 50.1 % [49.6, 50.7] on seed 97,
  pooling to 50.4 % [50.0, 50.7] over 28 800 games. Not adopted. Note
  what the paired ladder bought even here — the first seed's edge was
  identified as unreplicated at 14 400 games rather than 60 000.
- 🟡 **Castability-aware mana payment** (`Player::smart_tap` /
  `GameState::source_redundancy`, `EvalWeights::legacy_tap` as the
  control) — auto-tap paid generic pips by activation-cost rank with
  *battlefield order* as the tiebreak, so casting `{2}{B}` off 8 Swamp /
  6 Forest / 3 Island would tap an Island and strand the blue cards the
  splash exists to cast. Generic pips now spend the most replaceable
  source (a Swamp with 7 backups before an Island with 2) and coloured
  pips the narrowest one (a basic before a dual). It never changes
  whether the *current* cost can be paid, only which of several
  interchangeable sources pays it.

  **Measured null.** 50.9 % [50.4, 51.4] on seed 43 did *not* replicate:
  49.7 % [49.2, 50.3] on seed 97, pooling to 50.3 % [49.95, 50.68] over
  28 800 paired games. The fifth "obvious" improvement in this series to
  evaporate on replication.

  The field is not the excuse. The natural defence — "generated sealed
  builds don't run thin splashes, so the case never comes up" — was
  checked and is false: 3 of 12 decks on seed 43 and 4 of 12 on seed 97
  run a colour on ≤4 sources. The failure mode is present roughly a
  third of the time and still doesn't move the win rate.

  **Off by default** (`smarttap` opts in). It was briefly left on for
  the reasoning — the change cannot make a cost unpayable, the order it
  replaces was an accident of `battlefield` iteration rather than a
  decision, and the client's human-facing auto-tap is the case that
  motivated it — and then turned off to match how every other null in
  this tier was handled. The code and the profile stay so it can be
  re-measured, ideally on a field built to stress thin splashes.

  It also carried a **quadratic regression**: the selection called
  `effective_mana_abilities` per candidate per colour *inside the
  per-pip loop*, invisible in a two-player 40-card game and fatal in
  4-player Commander (`bot_vs_bot_commander_demo_terminates` went from
  seconds to past its 600 s timeout). Fixed by building the source
  table once per auto-tap — 600 s → 0.78 s. Worth remembering as the
  cost side of shipping a null on reasoning.

  The flag exists purely for measurement: the behaviour lives in the
  engine, so without a per-player switch both seats of a mirror would
  get it and the ladder would be structurally unable to show anything —
  the same blindness as the point below.
- 🟡 **Determinized combat search** (`EvalWeights::determinize`,
  `det1`/`det3`) — `simulate_attack_outcome` and `simulate_block_outcome`
  clone the true `GameState`, so the rollout opponent casts the cards
  they are actually holding and both seats draw the real top of library.
  The bot has been searching with perfect information. Redealing the
  hidden zones first costs **48.9 % [48.4, 49.4]** at one redeal and
  **49.4 % [48.9, 49.9]** averaged over three.

  Read that as the price of honesty, not a verdict on the idea. Both
  arms are mirror bots and the *control cheats*; taking information away
  from one side and not the other is expected to cost win rate. The
  mirror ladder cannot measure this fairly — it never could, which is
  why nothing before now had caught it. Against a human in the client
  the cheating is indefensible whatever the number says, so the open
  question is which default the client ships, not whether the search
  should be able to read a hand.
- 🟡 **Land-drop sequencing** (`landseq` / `EvalWeights::land_urgency`) —
  missing colors weighted by how cheap the cards demanding them are, and
  a per-land check for whether *that* land turns on a cast this turn (so
  a tapland is nearly free with no play and expensive otherwise).
  **Measured and not adopted**: 50.3 % [49.6, 51.0] over 19 200 sealed
  games. Two methodology notes worth more than the result: measured on
  `--decks both` first it read 49.4 % and *could not have read anything
  else* (those archetypes play basics, so tapland timing never fires),
  and the sealed +1.4 at 4 800 games collapsed to +0.3 at 19 200 — the
  third such evaporation after `blk` and `atk-race`.
- 🟡 **Better sequencing** (hold-up, when to cast) — reactive
  deployment landed: the stack-response value bar drops 10 → 5 with 6+
  cards in hand so answers get spent instead of rotting in a clogged
  hand; instant-speed removal fires at a declared attacker during
  DeclareBlockers when the attacker is worth it
  (`pick_defensive_removal`, ward- and outcome-gated); and
  sacrifice-for-value abilities are cracked when the settled outcome
  beats staying pat (`pick_sacrifice_value`). Self-cost optional
  triggers are likewise judged by settled outcome
  (`decide_optional_by_outcome`). Remaining: land-drop choice, deliberate
  hold-up planning.
- 🟡 **Mulligan decisions** — `RandomBot` ships flood/screw mulligans with
  color-screw awareness. A quality-aware rule (`mull` — card-quality sum,
  a redundancy requirement at two lands, on-the-draw allowance) is
  **measured and not adopted**: 50.2 % [49.6, 50.8] over 28 800 sealed
  games. Its tests stay as documentation of two hands the shipped rule
  reads backwards (a two-lander living off one two-drop is kept; six
  lands and a bomb is shipped). Remaining: transitive fetch/dual
  sources.
- 🟡 **Targeting / mode / X-value choices** — mid-resolution modals are picked
  by settled-outcome eval (`decide_mode_by_outcome`), scry/surveil/rearrange
  order for real (`decide_scry` — flood to the bottom, bricks off the top,
  wants first), and targeting/affordability is ward-aware (CR 702.21:
  a tax the bot can't pay after the spell's own cost drops the candidate,
  a payable one is priced into the score; `bot_wont_cast_removal_into_*`
  tests). SOS college mirrors run on the ladder (`bot_ladder --decks sos`)
  and probe (`bot_probe --deck sos`). X sizing now splits spare mana across
  multi-X pips ({X}{X} paid 2X but was sized as one) and covers
  prepare-casts (`max_affordable_x_for_def`). **Simulations answer
  decisions with the bot's own policy table** (`decide_pending_policy` in
  every lookahead/combat sim — they used to assume an AutoDecider future:
  bad scries, declined tutors, mode 0). Remaining: X chosen by outcome
  eval rather than max-dump.
- 🟡 **SOS mechanic play** — Prepare: inset-spell candidates, response casts
  when removal targets the prepared body (`pick_prepare_response`, plus the
  own-main response-timing dispatch fix that also revived counterspells
  there), a re-prepare mana sink, and a Prepared-counter term in
  `permanent_value`. Paradigm: the free-copy prompt is a real suspension
  now, and the bot declines life-draining copies at a low total
  (`self_life_loss`). On-cast payoff steering: Opus (prefer 5+-mana casts)
  and Infusion (lifegain first) score nudges; Repartee offers a
  creature-aimed sibling candidate the outcome eval judges; Increment
  nudges casts that clear the smallest body's threshold
  (`increment_threshold`); Converge casts pre-float one source per missing
  college color so the payment drains distinct colors
  (`pick_converge_prefloat` — bot-side, the engine payment funnel is
  untouched). Prepare-cast X is sized like a hand cast. The Prismari /
  Quandrix ≈ 49 % split was probed per college (`bot_probe --deck
  sos:<college> --vs baseline`): the losing pattern is over-attacking on
  small boards (82 % of eligible, 78 % all-in in Prismari; 41-42 % of
  creatures tapped at DeclareBlockers vs 27 % in healthy Witherbloom) plus
  reactive spells rotting in hand (42 cleanup discards / 60 games; ONE
  instant-timing cast). Two hypotheses measured and killed on 1000-game
  SOS ladders each: `atk-hold` (hold_instants — 49.4 %, Prismari *worse*
  at 46.0) and `blk` (block search — 50.1 %, tapped blockers are the
  cause, not assignment). Open lead: attack restraint that respects the
  defender's open mana / lets the attack sim cast spells for both sides
  (the sim casts nothing today, `simulate_attack_outcome` doc). A real
  `ChooseColor` policy (hand-pip demand) also landed off the Quandrix
  probe (11 % of its decisions were first-legal-White).
- 🟡 **Learned evaluation (SOS sealed)** — the ML stack's Phase A shipped:
  `crabomination_nn` (dependency-free inference + shard format, opt-3 in
  debug via a per-package override; wasm-safe, no framework in the engine
  or client path), `crabomination_ml` (candle trainer: deep-sets value
  net over card embeddings + zone-pooled objects, auxiliary life-diff /
  game-length heads per the KataGo credit-assignment result), and
  `server/encode.rs` (observable-info-only encoder, SOS sealed vocab).
  A parity test pins the candle model and the engine forward pass to the
  same numbers. Phase B shipped too: the concurrent `selfplay_train`
  loop (actor threads + throttled learner + atomic checkpoints, ~10.6
  sealed games/s on 22 debug-build actors), the `net_eval` slot registry,
  the `net`/`net-blend` bot profiles, and `bot_ladder --decks sealed`
  (same-deck sealed mirrors — build quality cancels, rows measure
  piloting). Release builds approved for the ML tooling (~82 games/s
  generation, 7.7× debug; gates cost ~15 s). Gates so far, 1 200
  sealed-mirror games each vs `atk-sim`: full net replacement 43.6 / 42.3
  / 43.4 % across round 1 (25k games), round 2 (100k), and round 2's
  over-reused tail — **flat across a 4× data jump**, so data volume is
  not the constraint; heuristic+net blend 49.3 / 49.2 / 50.7 % — stable
  parity. The tail experiment also priced window over-reuse (loss 0.30 →
  0.14, zero strength change → the trainer now caps the tail). Round 3
  measured (mid-turn snapshot cadence, 10.5 M rows, per-head loss
  logging, capped tail): replacement 44.7 % [41.9, 47.5] — best yet but
  within noise; blend 49.3 % — parity unchanged; **blend at 3× loudness
  45.9 %** — amplifying the net hurts, i.e. where it disagrees with the
  heuristic it is more often wrong. Standing diagnosis: `eval_material`
  scores outcomes of resolved sims (a one-ply search with a perfect
  model), so the net must carry long-horizon signal to add value, and
  ~125 k params of pooled encoder doesn't yet. Round 4 (5× capacity
  ~600 k params, keyword object features, CUDA-ready `cuda` feature
  flag): 43.8 / 48.8 / 47.1 % — same bands, **but the CPU learner only
  managed 0.4 visits/row before the tail cap**, so capacity remains
  untested until the learner moves to the GPU (`pacman -S cuda`, then
  `--features cuda`). Next levers: GPU-scale training, search-improved
  targets. **Phase C's build net passed its gate — the first learned
  component to clear the house bar**: `DeckNet` (D(decklist)→win prob,
  ~30 k params, trained free off the self-play stream's decklist
  labels) judging best-of-32 builds beat the heuristic static judge
  over the same candidate sets 61.7 % [58.9, 64.4] and 60.7 %
  [57.9, 63.4] on independent seeds (1 200 games each,
  `selfplay_train --gate-builder`). Remaining Phase C wiring: use the
  net-judged builder for training-run decks and as `recommend_pool`'s
  instant surrogate. Play-net replacement/blend still not adopted.
- ⏳ **Difficulty levels**; optional **search-based AI** (MCTS over snapshots).

## Tier 14 — Replays, analysis & observability

- 🟡 **Action-log replay viewer** — the capture side ships: `CRAB_REPLAY_DIR`
  appends one JSONL replay per match (header/players, one line per broadcast
  event batch, footer). Remaining: the viewer.
- ✅ **Game history / match results persistence** — `CRAB_MATCH_LOG` appends
  one JSON line per finished match (lobby/bot/pair paths;
  `crabomination_server::history`); `CRAB_MATCH_LOG_MAX_BYTES` caps the live
  file and rotates it to `<path>.1`.
- ⏳ **Export game to shareable file** (formalize the audit-snapshot workflow).
- ⏳ **In-game "what happened" log filtering** (by player/zone/type).

## Tier 15 — Accessibility

- ⏳ **Colorblind-safe** indicators, **text scaling / high-contrast /
  reduced-motion**, **full keyboard play**, **screen-reader narration**, **"full
  control" mode** (never auto-skip).

## Tier 16 — Infra, correctness & content tooling

- ⏳ **Seeded / deterministic RNG** surfaced for reproducible games.
- ⏳ **Snapshot round-trip property tests** + **action-sequence fuzzing**.
- 🟡 **Crash-recovery / autosave** — a match that panics writes a `GameState`
  plus the panic message to `CRAB_CRASH_DUMP_DIR`
  (`crabomination_server::crash_dump`, atomic write + newest-N retention).
  The dumped state is the **live checkpoint**: the server hands every match a
  `SnapshotSink`, which the actor republishes after each accepted action, so a
  panic on turn 15 dumps turn 15 (falling back to the pre-match capture when
  nothing published). Remaining: resume-from-dump.
- ⏳ **Card-scripting DSL** to reduce catalog boilerplate.
- ⏳ **Set / Scryfall import pipeline** (`scripts/verify_cards.py` exists — extend).
- ⏳ **Card art / image pipeline**.
- ✅ **Rules-engine conformance suite** mapped to CR sections — `scripts/
  cr_coverage.py` generates `CR_COVERAGE.md` (section → title, subrules tested,
  test count, plus the untested-section gap list) from the `cr_<section>_` test
  names. Regenerate it after adding conformance tests; do not hand-edit.
- ✅ **Operator telemetry endpoint** — `CRAB_STATUS_BIND` HTTP `/healthz` +
  `/status` (uptime, rolling match stats, slot accounting).

---

## Suggested sequencing

0. **Next set to close.** Bloomburrow, Duskmourn, Outlaws of Thunder Junction,
   **Edge of Eternities**, **Final Fantasy** and **Conspiracy** (CNS) are all
   closed (`set_gaps.py blb dsk otj eoe fin cns` is empty). The Odyssey
   block, the Onslaught block (**ONS**,
   **LGN**, **SCG**), the Mirrodin block (**MRD**, **DST**, **5DN**), the
   Kamigawa block, **Mirrodin Besieged**, **New Phyrexia** (the Scars block
   is closed), **Legends** (273 cards, `sets::leg`–`leg7`), **Antiquities**
   (64 cards, `sets::atq`), **Arabian Nights** (63 cards, `sets::arn`) and
   **The Dark** (97 cards, `sets::drk`/`drk2`) and **Homelands** (`sets::hml`–
   `hml3`), **Conspiracy: Take the Crown** (CN2), **Murders at Karlov
   Manor** (MKM) and **Stronghold** (STH) are all at zero. The rest of the
   **whole Tempest block is closed** (`set_gaps.py tmp sth exo` is empty).
   Pick the next block from `scripts/set_gaps.py`.
1. **Replacement-effect framework** (Tier-1 #1) — highest-leverage primitive still
   open.
2. **Card-zoom + stops/auto-yield + combat-math preview** (Tier-7 #1–3) — the trio
   that most closes the Arena "feel" gap.
3. **Best-of-3 + sideboard + deck legality** (Tier 10) — makes constructed
   competitive.
4. **Static-ability framework** — broad correctness wins. (Mana provenance
   shipped; see "Already shipped".)
5. **Smarter AI blocking** (Tier 13) — biggest single-player upgrade.
6. Then the **Tier-4 mechanic sweep** and **Tier-3 object-model** features, batch
   by batch.
7. **Replays, spectator, social, accessibility** as the product matures.

## Recently closed (this push)

- **Tempest block closed** — TMP, STH and EXO all report zero gaps. The three
  Tempest closers: **Duplicity** (`ExileHandThenReclaimLinked` + a linked
  five-card face-down reserve), **Oracle en-Vec** (`Effect::
  AttackMandateNextTurn` + `GameState.attack_mandates` — a per-seat next-turn
  "only these attack, and they must" mandate armed at that seat's untap step
  and cashed in at its end step) and **Ertai's Meddling**
  (`Effect::ExileSpellWithDelayCounters` + `process_delayed_spells`, a
  `CounterType::Delay` tick on an exiled stack object that re-casts it for
  free when the last counter comes off).
- **Exodus (EXO) closed** — 69 cards (`set_gaps.py exo` at zero),
  `sets::exo2`, tests in `classic_sets/exo`. New primitives:
  `EventKind::DealsDamageToPlayer` (the combat-agnostic dealer-side sibling of
  `DealsCombatDamageToPlayer` — Soltari Visionary, Avenging Druid, Entropic
  Specter), `Effect::{OathCatchUp, MoveAllCountersOfKind,
  SacrificeEachUnlessPays, GainControlWhileSourceAttached}`,
  `Effect::SacrificeAllButOnePerType.include_land` (Cataclysm's land slot),
  `Effect::DiscardAnyNumber.filter` (Mind Maggots' creature-only pitch),
  `DynamicPt::ChosenPlayerTally` + `PlayerTally::NonbasicLandsControlled`,
  `EquipBonus.add_card_types` (layer-4 additive — Transmogrifying Licid),
  `SelectionRequirement::BlockedBySourceThisTurn` (survives combat teardown —
  Wall of Nets), and `Keyword::{CantAttackUnlessMoreLandsThanDefender,
  CantBlockUnlessMoreLandsThanAttacker}`. Correctness: a
  remove-a-counter-from-among cost now prefers a *non-source* donor, so an
  ability that feeds the source (Spike Rogue) can't auto-pay itself into a
  no-op. Cleanliness: the five-way seat comparison behind the Keeper/Oath
  cycles is now `GameState::player_tally`.
- **The client type-checks in cloud sessions** — `crabomination_client`'s
  three pkg-config-backed platform bits (`wayland`, `audio`, `gamepad`) are
  split into default-on features over an explicitly spelled-out Bevy feature
  list, so `cargo check -p crabomination_client --no-default-features` works
  on images without `wayland-client.pc` / `alsa.pc` / `libudev.pc`. That
  immediately caught two real build breaks left by the Tempest wave
  (`CounterType::Magnet` missing from the client's counter label + tooltip
  matches).

- **Tempest all but closed** — 37 more cards (`set_gaps.py tmp` 40 -> 3;
  only Duplicity, Ertai's Meddling and Oracle en-Vec remain, each tracked in
  TODO.md with the primitive it wants). Tests in `classic_sets/tmp`. New
  primitives, by wave: `MillShareAxis::AnyColor`,
  `Effect::{TopTwoGraveyardOpponentSplits, LockActivatedAbilitiesThisTurn}`
  (backed by `GameState.abilities_locked_this_turn`, enforced in
  `activate_ability`); then `StaticEffect::{MaxUntapsPerStep,
  NoInstantsOrAbilitiesDuringCombat, AttackTogether}` (CR 502.3 / 506 /
  508.1d), `DynamicPt::{EnteredTotals, TappedLandsChosenPlayerControls}` with
  `Effect::AsEntersSacrificeForTotalPt`,
  `Effect::{RedirectNextCombatDamageTo, GrantSacrificedLandTypesLandwalk,
  DestroyBlockPairWeakerSide, MoveChosenKeyword, ScrollRack,
  TokenCopyOfOpponentChoice}`,
  `Predicate::TriggerSourceIsSourceHost`,
  `SelectionRequirement::ControlledByActivePlayer` and
  `CounterType::Magnet`. `CardInstance::make_licid_aura` now keeps a Licid's
  non-attach activated abilities, so the riderful Licids work as Auras.
  Correctness: an `EventKind::PlayerDamaged` + `SelfSource` trigger matches the
  source as the *dealer* (it previously only matched damage dealt TO the
  source, so Shocker and Thalakos Dreamsower never fired); an equipped bonus is
  skipped for a host whose controller holds an `IgnoreStaticFromSourceThisTurn`
  pass. UI/server: `PermanentView.abilities_locked` +
  `AbilityView.gate_blocked` fold in source-level ability locks, and
  `systems::lock_badge` chips the locked permanent.
- **Tempest opened** — 201 cards (`set_gaps.py tmp` 241 -> 40), tests in
  `classic_sets/tmp`. New primitives: `EventSpec::causer_filter` (+ a `by`
  field on `GameEvent::BecameTarget`) so "becomes the target of a [filter]
  spell or ability" gates on the *targeting object*, not just its controller;
  `ActivatedAbility::remove_all_counters_cost` + `Value::CountersRemovedAsCost`
  ("Remove all [kind] counters from this:" as a real cost the body scales off);
  `Keyword::CanBlockShadow`; `SelectionRequirement::PowerAtMostSourceCounters`
  (with a CR 608.2b LKI fallback for a source that paid itself as the cost);
  `RevealMissDest::Exile`; and `CounterType::{Elixir, Pain}`.
- **CR 613.11 fix** — `effective_max_hand_size` folded set-to-N caps with
  `min`; game-rule-modifying continuous effects apply in *timestamp* order, so
  the newest cap wins. The live combat-anthem `PumpPT` pass now also peels
  `While*` gates through `active_static`, so a gated anthem (Watchdog) reaches
  live combat state instead of dropping to the pure walk.
- **MKM closed** — A Killer Among Us, Conspiracy Unraveler and Kaya, Spirits'
  Justice. New primitives: `Effect::NameCreatureTypeAmong` (a closed-list
  creature-type choice, clamped at the pending-answer),
  `CardDefinition.secret_chosen_type` (the server view withholds a secret
  choice from other seats), `StaticEffect::CastHandSpellsForCollectEvidence`
  (collect evidence N in place of a spell's mana cost), and an `if_cant`
  branch on `Effect::TurnFaceUpFree` (Etrata's exile-and-free-cast fallback).
- **Stronghold closed** — `set_gaps.py sth` is empty (110 cards), tests in
  `classic_sets/sth`. New primitives:
  `CardDefinition.buyback_additional_cost` ("Buyback—Sacrifice a land"),
  `StaticEffect::PreventUntapGlobal` (a prevent-untap that reaches every
  seat's untap step, optionally predicate-gated — Intruder Alarm, Walking
  Dream), and the **Licid** mechanic
  (`Effect::LicidAttach`/`LicidDetach` + `CardInstance::make_licid_aura`,
  which stashes the creature definition and rewrites the live one to
  Enchantment — Aura; the aura riders ride the printed `equipped_bonus`).
  The last seven cards added `EventKind::Regenerated` (CR 701.15),
  `ActivatedAbility.mana_cost_per_self_counter` and `.put_hand_on_library_cost`,
  `CardDefinition.copies_top_graveyard_creature` (a layer-1 copy re-synced
  each SBA pass — Volrath's Shapeshifter),
  `StaticEffect::DiscardColorSharingCardAlternativeCost` (Dream Halls) and
  `StaticEffect::AttackingPlayerChoosesBlocks` + `GameState::block_chooser`
  (Invasion Plans).
- **`resolution_causer`** — `discard_causer` generalized to "the controller of
  the resolving spell or ability", held for the whole outermost resolution and
  replayed over the batched `PermanentDied` dispatch, so
  `Predicate::CausedByOpponentSpellOrAbility` (was `DiscardCausedByOpponent`)
  reaches non-discard riders (Sacred Ground).
- **Reflexive-cost target walking** — `MaySacrifice` / `MaySacrificeSource` /
  `MayTap` / `MayDiscard` / `MayDiscardMatching` now recurse in every
  target-query walker, not just `requires_target`, so a targeted `then`
  branch actually gets a target slot.
- **SOS Special Guests** — `sos_mode::sos_special_guests` names the
  eleven-card SPG sheet (Magus of the Library and Library of Leng were the
  two not already catalogued, the latter on a new
  `StaticEffect::DiscardToLibraryTop`), and `generate_sos_pack` collates it
  on its own slot at the printed `SOS_SPECIAL_GUEST_RATE` (1 in 64) instead
  of through the colour buckets.
- **Free-cast hand affordance** — `HandAffordances.free_castable` dry-runs
  `CastFromZoneWithoutPaying` over the hand, projected as
  `PlayerView.free_castable_hand`, so Omniscience / Aluren / Conspiracy
  Unraveler cards finally get the client's alt-cast border.
- **Client**: a "FREE" chip (`systems::free_cast_badge`) over hand cards a
  standing static casts for nothing — the shared cyan alt-cast border can't
  say *which* alternative applies, let alone that it costs zero.
- **Exodus opened** — `sets::exo` ships 50 cards (`set_gaps.py exo` 119 → 69),
  tests in `classic_sets/exo`. New primitives:
  `StaticEffect::BuybackCostsLess` (Memory Crystal),
  `DynamicPt::PermanentsOnBattlefieldMatching` (the power sibling of the
  existing toughness variant — Dauthi Warlord),
  `SelectionRequirement::OpponentTallyDiffers` + `card::PlayerTally` (the
  Keeper/Oath cycles' "target opponent who <trails you>" restriction) and
  `Effect::TargetPlayerThen` (a filtered player slot for bodies that don't
  reference the target themselves).
- **Block chooser end to end** — `ClientView.block_chooser` mirrors
  `GameState::block_chooser`, so `declares_blocks` reaches the Invasion Plans
  case, and `bot::forced_blocks` submits only the CR 509.1c-forced blocks when
  the attacking seat is the chooser (it used to decline everything, which the
  engine rejects on a MustBlock board).
- **Client**: a regeneration-shield chip (`systems::regen_badge`) over
  permanents with live shields — the tooltip was the only tell.
- CR conformance: `core_rules/cr_recent89` covers CR 701.19a, CR 604.4,
  CR 611.2c and CR 509.1c's forced-block declaration.
- CR conformance: `core_rules/cr_recent87` covers CR 707.4, CR 116.2b/116.3
  and CR 717.2/717.4/717.5.

Older per-push entries are elided — `git log -p -- FEATURE_ROADMAP.md` is
the record.
