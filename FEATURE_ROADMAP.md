# Feature Roadmap — MTGO / Arena / XMage parity

A prioritized capabilities roadmap (engine fidelity + UX + infra), derived from
a codebase analysis against the three reference clients. Per-card status lives
in `CUBE_FEATURES.md` / `DECK_FEATURES.md` / `STRIXHAVEN2.md`; the
approximations log is `CARD_BACKLOG.md`.

Legend: ✅ done · 🟡 partial · ⏳ not started. Markers are a point-in-time read —
re-verify before picking up an item.

---

## Already shipped (don't re-propose)

Moved to `SHIPPED.md` (size trigger). Check it before proposing anything.

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

Moved to `ML_NOTES.md` (size trigger): the bot/net gate history, the adopted
profiles and the documented dead ends.

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
   Manor** (MKM) and **Stronghold** (STH) are all at zero. The
   **whole Tempest block is closed** too (`set_gaps.py tmp sth exo` is empty).
   **Weatherlight (WTH) is closed** too (`set_gaps.py wth` at zero —
   `sets::wth` + `sets::wth2`, tests in `classic_sets/wth`), which finishes
   the Mirage block's third set and gives cumulative upkeep, banding and
   phasing their first real card coverage. **Visions (VIS) is closed** too (`set_gaps.py vis` at zero —
   `sets::vis` + `sets::vis2`, tests in `classic_sets/vis`), which finishes
   the Mirage block's second set. **Mirage (MIR) itself is the live front**
   (`set_gaps.py mir` at 17 after this push, `sets::mir`–`mir5`); Coldsnap
   (CSP) is open in parallel. Each of MIR's last 17 is blocked on one
   primitive — TODO.md → "Mirage residue" names them card by card.
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

- **Mirage waves 5–7** — 37 more cards (`set_gaps.py mir` 54 → 17),
  `sets::mir5`, tests in `classic_sets/mir`. New primitives, each forced by a
  card: `Effect::{ExileAtNextEndStep, SacrificeSourceUnlessSacrificeTotalPower,
  DiscardUnlessPutCardOnTop, AssignsNoCombatDamageThisTurn,
  LookTopPutOneOnBottom, AllLandsProduceChosenColorThisTurn}`,
  `Keyword::CantPhaseOut` (CR 702.26),
  `StaticEffect::ReduceSpellDamageBy`,
  `SelectionRequirement::HasChosenLandTypeOfSource`,
  `DynamicPt::ChosenPlayerGraveyardMatching`, and knobs on the existing
  prevention / look / return families (`exile_top_per_prevented`, `reflect`,
  `gain_life_colors`, `LookTopMayBottomAllElse.who`,
  `ReturnExiledBySourceToBattlefield.count`,
  `EachPlayerKeepsNSacrificesRest.filter`, `CatchUpBasicLands.target/tapped`).
  Correctness: 125 of the 164 silent target-walker gaps closed and the rest
  ratcheted by `core_rules/target_walkers`; CR 704.5m, 702.26c and the
  anywhere-shield redirect pinned in `core_rules/cr_recent98`.

- **Mirage (MIR) opened** — 221 cards across `sets::mir`–`mir4`
  (`set_gaps.py mir` 275 → 54), tests in `classic_sets/mir`. The slow-fetch
  cycle, the flanking knights, the Charms, the guildmages, the phasing shells
  (Crystal Golem, Dream Fighter, Teferi's Imp, Vaporous Djinn, Warping Wurm,
  Taniwha, Mist Dragon), the combat punishers and Chaosphere's inverted sky.
  New primitives, each forced by a card: `Keyword::{CantBlockPowerAtMost,
  CantBlockMatching, MustAttackIfAnotherAttacks}`,
  `Effect::{TopChosenFromHand, DestroyAllNoRegenGainControllerLifePerManaValue,
  EachPlayerCreatesTokenPerControlled}` and `SelectionRequirement::YouPlayer`.
  Three latent bugs fell out of it: `Selector::MatchingAmong` statics never
  applied (so "first strike while attacking" was inert on Soltari Lancer and
  Spirit of the Night), a card couldn't replace its *own* death, and
  `Effect::MoveCounters` bound no target on a triggered ability.

- **Per-attacker block legality** — `GameState::legal_block_targets` +
  `ClientView.legal_block_targets` / `block_is_legal`: the client now refuses
  an illegal blocker→attacker drop at the click instead of letting the server
  reject the whole declaration. `legal_blockers` alone only answered "can this
  block *something*".

- **Coldsnap (CSP) opened** — 32 cards (`set_gaps.py csp` 123 → 91),
  `sets::csp`, tests in `classic_sets/csp`. The snow tapland cycle, the `{S}`
  pump/keyword creatures, the cumulative-upkeep payoffs (Earthen Goo, Arctic
  Nishoba, Jötun Owl Keeper, Kjeldoran Javelineer) and the
  copies-in-graveyards pair (Feast of Flesh, Kjeldoran War Cry). New:
  `CreatureType::Aurochs`. The remaining pile leans on recover, ripple, the
  Rimewind cycle and a handful of one-off legends — Brooding Saurian ("each
  player gains control of all nontoken permanents they own") and Goblin Furrier
  (a filtered `PreventThisDamageToColor`) each want one primitive and are
  tracked in TODO.md.

- **CR conformance + the truce's UI tail** — `cr_recent96`: CR 120.8 (a source
  dealing 0 damage deals none, so damage triggers don't fire), CR 514.2 (a real
  gap — cleanup cleared marked damage and "until end of turn" effects only on
  the battlefield, so a phased-out permanent kept both across the turn
  boundary) and CR 121.5 (library → hand without the word "draw" isn't a draw).
  Server: `ClientView.truce_active` surfaces Peace Talks' two-turn lock;
  client: a "☮ truce" chip on the active player's row and a `1Src` keyword tag
  for Ogre Enforcer, so neither seat plans an attack or a target the server
  will refuse.

- **Visions (VIS) closed** — `set_gaps.py vis` 139 → 0 across
  `sets::vis` + `sets::vis2`, tests in `classic_sets/vis`. The Karoo bounce
  lands, the Charms, the flanking knights, the Chimera cycle, the world
  enchantments and every phasing card in the set. New primitives, by wave:
  `Keyword::DamageBecomesMinusCounters`, `StaticEffect::AllPlayersSpellsCostLess`;
  then counted return-to-hand costs (`WardCost::ReturnMatchingToHand` and
  `ActivatedAbility.return_permanent_cost` both carry an N — Bull Elephant's
  two Forests were a real bug),
  `ManaPayload::AnyTypeSacrificedLandProduces`,
  `CounteredSpellZone::CountererBattlefieldIfMatching`,
  `CounterType::{Death, Rust, Pressure}`; then `Effect::SwapPhasedState`
  (CR 702.26 simultaneity — Time and Tide),
  `StaticEffect::PlayersActOnlyOnTheirOwnTurn` (City of Solitude, gated at the
  action dispatch so casts *and* activations are covered),
  `Effect::{TopOfGraveyardToLibraryTop, LookTopMayPayLifeToBin}`,
  `AdditionalCastCost::ReturnToHand.count_x`; then
  `SelectionRequirement::ManaValueAtMostOwnCounters` and
  `LandsBecomeChosenBasicType.from_chosen_basic`. Correctness:
  `WardCost::ReturnMatchingToHand` evaluates its filter source-aware; CR 704.5n
  reads the *computed* type line, so an Equipment survives on an animated land;
  and `AffectedPermanents::AllOpponents` stopped dropping the colour /
  creature-type / counter leaves of an opponent-scoped static (Heat Wave). The
  closing wave added `Effect::{ReturnToHandAtYourNextUntapStep,
  ExileRandomFromHandMayPlayThisTurn}` and
  `Predicate::TappedLandForManaThisTurn`. The last six each took one named
  primitive: `StaticEffect::DrawsRevealedTaxed` (Breathstealer's Crypt, a
  CR 121.2a reveal-and-tax draw replacement), `Effect::MayRepeat` (Forbidden
  Ritual — the costless sibling of `MayPayRepeatedly`),
  `Keyword::SurvivesSplitLethalDamage` + a per-source damage tally on
  `CardInstance` read by the lethal-damage SBA (Ogre Enforcer),
  `Effect::TruceThisTurnAndNext` + `GameState.truce_until_turn` (Peace Talks —
  two turns where nothing attacks and nothing can be targeted),
  `Effect::DrainDefendersLandsForManaNextMain` (Pygmy Hippo) and
  `Effect::PumpAttackersThisTurn` (Song of Blood, whose amount is frozen at
  resolution so a later combat trigger can't re-read it as zero). Correctness:
  an activated ability's sacrifice cost now stamps `GameState.sacrificed_card`
  and `Effect::WithSacrificedPt` carries the permanent actually sacrificed
  rather than the ability's source — `Selector::SacrificedCard` read the wrong
  card for every sac-cost activation.

- **Weatherlight (WTH) closed** — 137 cards across `sets::wth` and
  `sets::wth2` (`set_gaps.py wth` at zero), tests in `classic_sets/wth`.
  Cumulative upkeep (CR 702.24) went from an unused keyword to a real
  mechanic: it reads *computed* keywords (Mana Chains' Aura-granted upkeep
  ticks), a mana upkeep auto-taps rather than draining an already-empty pool,
  `CumulativeUpkeepCost::{PutCounterOnSelf, Draw}` cover the always-payable
  kinds (Aboroth, Psychic Vortex), and `EventKind::CumulativeUpkeepUnpaid`
  fires *before* the sacrifice with the age count as its event amount (Heart
  of Bogardan). New primitives, by wave: `WardCost::{
  ExileTopFromGraveyardMatching, ReturnMatchingFromGraveyardToHand}`,
  `ActivatedAbility.exile_other_top`, `Effect::ExileBottomOfGraveyard`,
  `Keyword::CantBeTargetedBySpells`, `CounterType::Shell`,
  `Effect::RegenerateThenGainControl` (+ `CardInstance
  .regeneration_control_grant` — Debt of Loyalty only steals if the shield is
  really spent), `Effect::{PlayerCantActivateNonManaAbilitiesThisTurn,
  ChooseFromHandToTopOfLibrary, CastFromGraveyardTopThisTurn,
  GrantCreatureSpellsFlashThisTurn, CoinFlipDoubleOrPreventNextDamage,
  Doomsday, TapLandsSharingProductionWith,
  EachPlayerSacrificesGreatestManaValueUnlessPays}`,
  `AdditionalCastCost::ExileFromGraveyardXFromCost`, `EventKind::PhasesOut`,
  and `RevealTopDeployIfMatch.miss_to_graveyard` /
  `RevealTopOpponentBinsOne.rest_stay_on_top`. Correctness: the AutoDecider
  now takes library searches instead of declining every one;
  `Effect::AtEndOfCombat` carries the triggering object into its delayed
  trigger; `WardCost::SacrificeMatchingN` is source-aware (`OtherThanSource`
  reads right — Lotus Vale can't pay for itself); phase-out triggers fire
  while the permanent is still on the battlefield. Cleanliness: the six
  duplicated cast-timing blocks collapsed into `GameState::flash_granted_for`.

- **Tempest/Exodus follow-ups + CR conformance** — `EventKind::
  LostControlOfThis` (CR 800.4, with the trigger re-pointed at the seat that
  *lost* control, since the permanent already belongs to someone else by
  dispatch time — Duplicity's pile now bins on a control change too); attack
  mandates stack and drop creatures whose controller changed;
  `Effect::MustBlockSource.chooser` routes "defending player chooses" to a real
  seat (Crashing Boars); CR 800.4b gates on token creation and control
  changes to a departed player; CR 101.4 APNAP ordering for
  each-player-unless-pays taxes. UI/server: `PermanentView.must_block` badges
  a CR 509.1c conscription as "Blk!".
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
