# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.
See `CUBE_FEATURES.md` (cube-card implementation status),
`STRIXHAVEN2.md` (Secrets-of-Strixhaven status), and `FEATURE_ROADMAP.md`
(prioritized engine functionality). **The correctness-audit section below
outranks everything else in this file** — its P0 tier is game-deciding or
state-corrupting in ordinary play.

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
  - Bonehoard Dracosaur — upkeep exile-top-2-may-play ✅ (turn-scoped
    `Player.play_from_top_this_turn` + `Effect::GrantPlayFromTopThisTurn` /
    `ExileTopAndGrantMayPlay` now ship). Still needs the per-type "if you
    exiled a land → 3/1 Dino; if a nonland → Treasure" rider, which wants an
    `exiled-this-resolution` selector (no single exile funnel today).
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
  Captain, Deathless Pilot in `decks::recent24`).
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
  Read Ahead (702.155 — Saga enters-chapter choice), Space Sculptor (702.158 —
  sector assignment), Living Metal (702.161 — needs the MTMTE transform DFCs).
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

## Discovered follow-ups — `decks::recent8`/`9`/`10` (Avatar / Lorwyn) batch

- **Waterbend (CR 701.67).** Not implemented. "Waterbend [cost]" is a generic-
  mana sub-cost where each generic may be paid by tapping an untapped artifact
  or creature — Convoke restricted to the generic portion of a *sub*-cost, and
  taking artifacts as well as creatures. Appears as an additional cast cost
  (Crashing Wave: "waterbend {X}") and as an activated-ability cost (Flexible
  Waterbender: "Waterbend {3}: …"). Needs a cost-path extension; the
  `cast_spell_with_convoke` plumbing is the closest primitive to generalize.
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
- **Modal activated abilities — mode selection via `GameAction::ActivateAbility`.**
  `pick_trigger_mode` picks the mode at push time (deferred to UI players for
  `wants_ui`), but `ActivateAbility` carries no `mode` field, so scripted/test
  callers can't force a specific mode — they get the decider's pick (mode 0).
  Phoenix Down's second mode (exile a Skeleton/Spirit/Zombie) is therefore only
  reachable interactively. Add a `mode: Option<usize>` to `ActivateAbility`.
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
  Bashtronaut, Swiftwing Assailant, Risen Necroregent, Embalmed Ascendant.
  Remaining: client HUD speed chip; the rest of the DFT speed pool.
- **Caustic Bronco** — deferred: needs a "reveal top → hand, then lose life =
  its mana value unless saddled, else each opponent loses that much" composite
  (only the opponents-lose-MV half, `RevealTopToHandOpponentsLoseMv`, exists).
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

- ⏳ **Auto-targeter doesn't see `SelectionRequirement::Not(Land)`.** A
  `target_filtered(Not(Box::new(Land)).and(ControlledByOpponent))` ETB exile
  silently fizzles under the bot/auto-target path, while the canonical
  `SelectionRequirement::Nonland` form is targeted correctly (caught wiring
  Sheltered by Ghosts). The auto-target candidate scan should treat
  `Not(Land)` as `Nonland`, or card defs should always prefer `Nonland`. Audit
  catalogs for `Not(Box::new(SelectionRequirement::Land))` in target filters.

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
  - **Conflagrate** ✅ — `flashback_additional_cost_for_name` takes the cast's
    X (Flashback—discard X cards).
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
  + `GameEvent::PermanentPhasedIn`. The side-zone model (`GameState.phased_out`)
  is the hook.
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
  - ✅ **More split cards.** Dusk // Dawn, Never // Return, Turn // Burn
    (`ResetCreature` + `BecomeColor`), Hide // Seek (Seek =
    `Search{Target opp, → Exile}` + `GainLife(ManaValueOf(LastMoved))`;
    the searched player's decider answers the pick) all ship; Boom // Bust
    already existed.
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

## MagicCompRules coverage audit

Periodic spot-check against the rules document (`MagicCompRules_20260417.txt`).
One line per rule: status (✅ wired · 🟡 partial · ⏳ todo) plus the still-open
gap. The full per-clause accounting (every sub-rule, code line, and test name)
was elided in a doc-compaction pass — recover it from
`git log -p -- TODO.md`. Markers are a point-in-time read; re-verify before
picking an item up.

### Done (✅) — wired

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
  turn" via `Predicate::PlayerDrewAtLeastThisTurn` + `once_per_turn`)
- ✅ CR 702.148 — Cleave
- ✅ CR 702.47 — Splice
- ✅ CR 704.5k — world rule
- ✅ CR 614.5 / 701.10f — mana-production multipliers compose
- ✅ CR 702.64 — Absorb
- ✅ CR 704.5y — Role uniqueness SBA
- ✅ CR 701.30 — Clash, seat-routed
- ✅ CR 702.104 — Tribute, seat-routed
- ✅ CR 700.4 — "dies" under graveyard→exile replacements
- ✅ CR 702.31 — Horsemanship
- ✅ CR 701.30 — Clash
- ✅ CR 510.1d — full damage assignment
- ✅ CR 701.37 / 712.16 — Meld
- ✅ CR 702.146 — Disturb
- ✅ CR 104.3c (with the 104.2 win override)
- ✅ "When this card is milled" triggers
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
- ✅ CR 701.19 — Searching (incl. `Effect::SearchUpToN` count-search — Nylea's Intervention, Deathbellow War Cry; test `cr_701_19_search_up_to_n_picks_matches_only`)
- ✅ CR 714.4 — DFC sagas
- ✅ CR 702.103 — Jump-start
- ✅ CR 707.2 — continuous copies
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
- ✅ CR 712 — Transforming Permanents
- 🟡 CR 708 — Face-Down Permanents
- ✅ CR 702.146 — Daybound/Nightbound
- ✅ CR 702.114 — Devoid
- ✅ CR 702.115 — Ingest
- ✅ CR 701.x — Process
- ✅ CR 208.2 / 613.7b — Set base P/T
- ✅ CR 702.21 — Ward (discard / life / `WardCost::LifeSourcePower` = source's power — Phyrexian Fleshgorger; `cr_702_21_*`)
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
- ✅ CR 702.44 — Sunburst (`enters_with_counters: (PlusOnePlusOne, Value::ConvergedValue)` — Suntouched Myr)
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
  Deserter, Ran and Shaw, Sozin's Comet (grants firebending 5).
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
- ✅ CR 310 — Battle / Siege. `CardType::Battle` + `BattleSubtype::Siege`, defense
  counters (310.7), protector choice (310.6), attack-your-own-Siege
  (`AttackTarget::Battle`), combat damage strips defense counters (310.10),
  defeat→exile/transform SBA (704.5x via `defeat_battle`). 6 MOM Invasions in
  `decks::mom`; tests in `tests/mom.rs`. ⏳ multiplayer protector choice.

### Partial (🟡) — remaining gap noted
- 🟡 **CR 509.2 / 510.1c — Banding** — a banding blocker routes the blocked
  attacker's damage order + assignment to the defending player (Benalish Hero;
  test `cr_509_2_banding_blocker_lets_defender_assign_damage`). Remaining:
  attacking-band formation, "bands with other", and the band-blocks-multiple
  damage-distribution corner.
- 🟡 **CR 303 — Auras** — characteristic-overriding Auras ✅ (`EquipBonus.{set_base_pt,set_card_types,set_creature_types,set_colors,remove_abilities}` install layer 4/5/6/7b continuous effects on the host — Ichthyomorphosis "0/1 blue Fish, no abilities", One with the Stars "becomes an enchantment", Heliod's Punishment "loses abilities + can't attack/block"; removal is ordered before the aura's own keyword grants so they survive — test `cr_613_aura_set_base_pt_then_counter`). Remaining: replacement-style Aura ETB (enters attached under another rule) + bestow type-switch corners.
- 🟡 **CR 603.10 — Last-Known Information** — full LKI for mid-resolution stack sources (e.g. lifelink 702.15c). (CR 603.6d "leaves the battlefield" self-source triggers now also fire on the lethal-damage SBA path, not just the destroy/sacrifice path — Thought-Knot Seer's LTB draw.) Sac-as-cost activated abilities that read the sacrificed source's own counters at resolution now stash `leaves_bf_lki` during cost payment (it outlives the per-dispatch `died_card_snapshots` clear) so `Value::TotalCountersOn { This }` reads the last-known total — Twitching Doll's "Spider per counter on it" (`twitching_doll_nests_then_sacs_for_spiders`).
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
  defeat ✅ (704.5x via `defeat_battle`, `tests/mom.rs`). Dungeon / Speed SBAs
  remain; multi-SBA "collapse into one replacement" (704.7).
- 🟡 **CR 613 — Interaction of Continuous Effects** — 613.7 timestamps ✅ (object timestamps stamped on entry/attach/face-up/transform from the shared effect counter; statics order by `object_timestamp()`; tests `cr_613_7_*`). Remaining: no dependency analyzer (613.8); CDA-first pre-pass (613.3). (EOT keyword grants now join the walk timestamped — audit P1 row closed. Static keyword-grant scopes now route a `ToughnessGreaterThanPower` leaf through the `CardMatch` dynamic path — read against printed P/T + counters per the `CardMatchPowerGated` approximation — so Tapestry Warden / Ancient Lumberknot grant their keyword only to your T>P creatures.)
- 🟡 **CR 208 — Power/Toughness** — base-P/T-only checks (208.4b). 208.3 noncreature P/T now observable for `*`-power Vehicles: `DynamicPt::LandsControlledPower` sets power off a count while toughness stays printed, `computed_permanent()` reports it on a non-crewed (noncreature) Vehicle (Lumbering Worldwagon `*`/4; test `lumbering_worldwagon_power_tracks_lands`). Conditional base-P/T set ✅ (`StaticEffect::SetBasePtIf` — live layer-7b SetPowerToughness gated on a predicate; counters/+N stack on top per 613.7c/f — Snowmelt Stag "5/2 during your turn"; `snowmelt_stag_*`).
- 🟡 **CR 119 — Life** — 119.7 set-to-lowest ✅ (`Value::LowestLifeTotal` + Repay in Kind); exchange-life-totals ✅ (Soul Conduit, Mirror Universe, Magus of the Mirror); life-gain→loss replacement ✅ (`StaticEffect::LifeGainBecomesLoss`, Tainted Remedy); life-gain **bonus** replacement ✅ (119.10 — `StaticEffect::LifeGainBonus { target, amount }` folded into `adjust_life` via `life_gain_bonus_now`; Honor Troll's "gain that much plus 1"). 119.7 rest-of-game lifegain lock ✅ (`Effect::LifeGainLockGame` sets the permanent `Player.cannot_gain_life` flag, distinct from the turn-scoped lock — Screaming Nemesis via `Selector::Target(0)`; test `screaming_nemesis_redirects_damage`). Life-total-threshold statics ✅ (`Predicate::PlayerLifeAtLeast` gates a live self-anthem — Angel of Vitality's +2/+2 at 25+ life; `cr_119_*`, `tests/recent17.rs`). Remaining: redistribute-life-totals; per-source life-gain replacement breadth. (Audit follow-up closed: every `LifeGained` emitter now uses `adjust_life_applied`, and `SetLifeTotal`/`ExchangeLifeTotals` route through the funnel — so a can't-gain-life lock on the player who would gain blocks their half of an exchange while the other still loses; test `cr_119_7_exchange_life_totals_respects_cant_gain_life`.)
- 🟡 **CR 121 — Drawing a Card** — draw-count replacement (121.2a) ✅ via `StaticEffect::ControllerDrawsDoubled` in `draw_one` (Thought Reflection; stacks per 614.5, reentrancy-guarded). Remaining: choose-to-draw (121.3); mid-cast face-down draw (121.8); reveal-on-draw (121.9).
- 🟡 **CR 502 — Untap Step** — Phasing (502.1 / 702.26) ✅: `do_phasing`
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
- ✅ **CR 510 — Combat Damage Step** — remains-blocked ✅ (`blocked_attackers`, 510.1c); excess non-trample damage assigned to the last blocker ✅ (510.1d); lethal accounts for marked damage ✅ (510.1c, double-strike tramplers); blocker strike-back per-source ✅ (702.90 / 615.6 — infect/deathtouch/scaling/shields/lifelink apply per blocker event, tests `cr_702_90_*`). **Assigns combat damage equal to toughness** ✅ (510.1c — `Keyword::AssignsCombatDamageByToughness`, read by `combat_damage_value` for attackers, blockers, and the cached-assignment path; Doran, Tapestry Warden, Bill the Pony, `tests/recent23.rs`).
- 🟡 **CR 509 — Declare Blockers** — cost-to-block (509.1d-f); put-onto-battlefield-blocking (509.4); "blocks two or more" batch counting (509.3e). Blocker legality now reads the computed view ✅ (509.1a — animated manlands / crewed Vehicles block). ("Can't be blocked except by N or more creatures" ✅ via `Keyword::CantBeBlockedExceptByN` — Pathrazer of Ulamog, generalizing Menace.) Per-pair block restriction (509.1b — "target creature can't block this creature this turn") ✅ via `Effect::CantBlockSourceThisTurn` + `GameState.cant_block_pairs` (Kozilek's Pathfinder); "must be blocked if able" (509.1c) ✅ via `Keyword::MustBeBlocked` (Loathsome Catoblepas). Power-based block restriction ✅ (`Keyword::CantBeBlockedByPowerLess` — Formation Breaker; inverse of Skulk, `formation_breaker_blocks_only_by_equal_or_greater_power`). The bot's block planner now satisfies the minimum-blocker count for Menace **and** `CantBeBlockedExceptByN(n)` (tops up or drops the block), so it never submits an illegal under-filled multi-block. Protection-by-mana-value block restriction ✅ (`Keyword::ProtectionFromManaValueExcept` — Haktos can't be blocked by a creature whose MV isn't the chosen number; test `cr_509_1b_protection_from_mv_restricts_blockers`). Protection-by-mana-value-**parity** ✅ (`Keyword::ProtectionFromManaValueParity { odd }` — Lavabrink Venturer's ETB odd/even choice; gates targeting, blocking, and combat-damage prevention CR 702.16e; tests `lavabrink_venturer_parity_protection`, `cr_702_16e_parity_protection_prevents_combat_damage`). Blocker-side "can block only creatures with flying" ✅ (`Keyword::CanBlockOnlyFlying` — Wanderlight Spirit, Shacklegeist, Pinnacle Emissary's Drone; test `cr_509_1b_can_block_only_flying_restriction`). Conditional attack/block gates (509.1a / 508.1a) ✅ — `Keyword::CantAttackOrBlockUnlessHandSizeAtMost(n)` (Hazoret the Fervent), `Keyword::CantAttackOrBlockUnlessDelirium` (Patchwork Beastie, via `GameState::delirium_active`), and `Keyword::CantAttackOrBlockUnlessDescend(n)` (The Ancient One, via `GameState::descend_count`), enforced in `declare_attackers` + `blocker_can_block_attacker` + `legal_attackers`/affordances and surfaced as client chips. "Can't attack or block alone" (509.1c) ✅ — `Keyword::CantAttackOrBlockAlone` rejects a lone-attacker / lone-blocker batch (Toby's Beast token; tests `cant_attack_or_block_alone_*`, `cant_block_alone_*`).
- 🟡 **CR 118 — Costs** — interactive mana-ability decline (118.3c); hybrid-pip per-reduction choice (118.7e); general unpayable-cost gate (118.6). Board-conditional self cost reduction ✅ (CR 601.2f — `StaticEffect::SelfCostReducedIfControlEach`, discounts a spell while you control a permanent matching each filter — Of One Mind's Human + non-Human). Opponent target-tax ✅ (`StaticEffect::TaxOpponentSpellsTargeting`, threaded through `extra_cost_for_spell` with the spell's chosen target — Jubilant Skybonder, Callaphe Beloved of the Sea). Mana-spent-vs-MV gate ✅ (`Effect::CounterSpellDrawIfUnderpaid` reads the countered spell's stored `mana_spent` against its mana value — Unravel draws only on a cost-reduced/alt-cast spell).
- 🟡 **CR 113 — Abilities** — emblems+CDA zones (113.6); full ability removal (113.10b); "can't have" anti-grant (113.11). Counter-target-ability (113.9) ✅ — `Effect::CounterAbility` (Consign to Memory, Stifle) with precise targeting via `SelectionRequirement::HasAbilityOnStack`.
- 🟡 **CR 115 — Targets** — Aura subtype (115.1b); zero-target cast-time gate (115.6); change-target corners (115.7a-d, cross-spell exchange). Same-target rejection *within one multi-target instance* (115.3) ✅ — `Effect::distinct_target_count` + a cast-time duplicate check reject the same object filling two divide/support slots (Forked Bolt); cross-clause sharing stays legal. "Counter target spell that targets you or a permanent you control" ✅ via `SelectionRequirement::SpellTargetsControllerOrControlled`, which reads a stack spell's chosen targets (Hindering Light).
- 🟡 **CR 116 — Special Actions** — Companion ✅ (116.2g / 702.139 —
  `GameAction::CompanionToHand`, {3} sorcery-speed sideboard→hand; deck-build
  restriction ✅ via `CardDefinition.companion` + `format::companion_restriction_met`,
  enforced by the server deck loader). (Foretell/Plot/Suspend ✅; manifest turn-face-up `GameAction::TurnFaceUp` ✅ — CR 708.5. Morph cast-face-down spell path still ⏳.)
- 🟡 **CR 105 — Colors** — type-line + color rewrite rider (105.3 second half).
  Color-count value (105.2 — `Value::ColorCountOf`, "for each of its colors";
  colorless/devoid counts 0 per 105.2c) ✅ — Breathe Your Last; tests
  `cr_105_2c_colorless_counts_zero_colors`, `breathe_your_last_gains_life_per_color`.
- ✅ **CR 705 — Flipping a Coin** — Mana Clash two-player flip-off loop (705.2), 705.3 advantage/Krark's Thumb, win-a-flip trigger (`EventKind::WonCoinFlip`/`GameEvent::CoinFlipWon`, Chance Encounter) and lose-a-flip trigger (`EventKind::LostCoinFlip`/`GameEvent::CoinFlipLost`, emitted on the tails path of FlipCoin + ManaClash). Remaining ⏳: opponent-chooses-half flips (Karplusan Minotaur). (AutoDecider now flips a real random coin; scripted tests stay deterministic.)
- 🟡 **CR 122 — Counters** — defense counters / Battle type (122.1g) ✅ (`CounterType::Defense`, CR 310). Counter-clear on zone change (122.2) ✅ strict — cleared at every zone-change funnel; dies-with-counters triggers read the `died_card_snapshots` / `leaves_bf_lki` LKI caches (Felisa, Ambitious Augmenter). `-0/-1` / `-1/-0` counter types ✅. "Choose a kind of counter at random it doesn't have" ✅ via `Effect::AddRandomMissingCounter` (keyword counters + +1/+1, never duplicating a present kind; respects Solemnity — Crystalline Giant). Return-a-died-creature-with-a-keyword-counter ✅ — a `CreatureDied`/`AnotherOfYours` trigger `Move`s `Selector::TriggerSource` (its gy card) back to the battlefield, then `AddKeywordCounter` on `Selector::LastMoved` (Luminous Broodmoth's flying counter; `luminous_broodmoth_returns_with_flying`). CR 614.16 additive replacement for *every* counter kind ✅ — `StaticEffect::ExtraCounterAllKinds` (Winding Constrictor) adds one to any counter placed on your creatures, via `GameState::scaled_counter_count`; composes with Hardened Scales (+1/+1-only) and Doubling Season. The player-counter "counters you'd get" half is still approximated. Keyword counters granting the keyword via layers ✅; test `cr_122_1_keyword_counter_grants_keyword` (Gift of the Viper).
- 🟡 **CR 401 — Library** — play-with-top-revealed + play/cast-from-top ✅
  (401.5/401.6 — `StaticEffect::{TopOfLibraryRevealed,PlayFromLibraryTop}` plus
  the turn-scoped `Player.play_from_top_this_turn` grant
  (`Effect::GrantPlayFromTopThisTurn` — The Belligerent), both honored by
  `library_top_playable` + `known_library_top`/HUD chip; Courser, Oracle of Mul
  Daya, Mystic Forge). Remaining: the mid-cast "new top stays hidden until
  the spell finishes" timing nuance (401.5 second sentence); multi-card
  same-position picker (401.4). (401.7 `LibraryPosition::FromTop` ✅.)
- 🟡 **CR 706 — Rolling a Die** — stored rolls (706.8); ignore-roll riders. Roll trigger (706.6) ✅ — `EventKind::RolledDice`/`GameEvent::DiceRolled { player, count, high }` fires once per roll instruction ("whenever you roll one or more dice"). Result-referencing effects ✅ via `Value::LastDieRoll` (706.4 — Ancient Copper Dragon). **Result-gated triggers ✅** — `Predicate::DieResultAtLeast(n)` filters a roll trigger on the roll's greatest result (Ground Pounder's "roll a 5+ → trample"), reading `DiceRolled.high` through `event_amount`. (modifier / reroll-at-most / doubles ✅.) Remaining ⏳: stored rolls (706.8), ignore/reroll-replacement riders.
- 🟡 **CR 707 — Copying Objects** — in-place copy (707.4); MDFC-face copy (707.8); static copy effects (707.2c); copied "as enters" choices (707.6); spell-copy exceptions (707.9). (Enter-as-copy "except it's also [type]" ✅ via `EntersAsCopy.extra_card_types` — Phyrexian Metamorph copies any artifact/creature and stays an artifact. Token-copies with haste + delayed sacrifice ✅ via `Effect::CreateTokenCopiesHasteSac` — Devastating Onslaught's X copies, CR 707.2 + 111 + 701.16.)
- 🟡 **CR 506 — Combat Phase** — remove-from-combat ✅ (506.4 — `Effect::RemoveFromCombat` pulls a targeted attacker/blocker out of combat, releasing its blockers; Labyrinth of Skophos, test `cr_506_4_*`). "block as though" restrictions (506.6); combat-step cast-timing gates (506.7). `PlayerRef::DefendingPlayer` now resolves off the *triggering attacker* for `YourControl`-scoped Attacks triggers (not just the ability source), so "whenever a creature you control attacks, defending player loses N" fires correctly (Leeching Sliver, CR 509.2). Combat-damage-to-player triggers now carry the damage dealt as `event_amount` (CR 119.3), so `Value::TriggerEventAmount` riders scale by the hit (Visions of Brutality). Such triggers now also **auto-target a graveyard card** when their effect prefers one (`prefers_graveyard_target`) instead of always binding slot 0 to the damaged player — Efreet Flamepainter recasts an instant, Venerable Warsinger reanimates a creature. (`CopySpell` / `CastWithoutPayingImmediate` are now surfaced by `primary_target_filter`, so on-cast self-copy and gy-recast triggers auto-target correctly; `CastWithoutPayingImmediate` accepts a `Permanent` entity-ref for the targeted gy card.)
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
- ✅ **CR 701.16 — Sacrifice** — `GameEvent::CreatureSacrificed`/`PermanentSacrificed` distinct from the lethal-damage/`Destroy` die path; `EventKind::CreatureSacrificed` triggers fire only on genuine sacrifice (Mortician Beetle). Targeted sacrifice of an already-chosen permanent ✅ via `Effect::SacrificePermanent { what }` (fires sacrifice + death triggers; Footsteps of the Goryo / Apprentice Necromancer sacrifice their reanimated creature at the next end step; `cr_701_16_targeted_sacrifice_fires_death_triggers`). Remaining ⏳: batched multi-permanent sacrifice-cost picker. (Audit follow-up closed — the P1 death-funnel bypass family is fixed; all arms route through the shared funnels.)
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
- ✅ **CR 701.60 — Suspect** — `Effect::Suspect { what }` + `CardInstance.suspected`; a suspected creature gains computed Menace + CantBlock (injected in `gather_continuous_effects`). `Predicate::SourceIsSuspected` gates Repeat Offender's toggle. Ships Barbed Servitor, Repeat Offender, Reasonable Doubt.
- ✅ **CR 701.35 — Detain** — `Effect::Detain { what }` + `CardInstance.detained_by`; a detained permanent can't attack/block (combat gates) or have its abilities activated (`activate_ability` gate), lifting at the detainer's next turn (`do_untap`). Surfaced in `PermanentView.detained` + a client tooltip badge. Ships Lyev Skyknight. ⏳: granted "enters detained" statics. (Loyalty activation now honors `detained_by`; Detain's target filter is enforced at cast time.)
- ✅ **CR 701.29 — Fateseal** — `Effect::Fateseal { who, amount }`: look at the top N of a targeted opponent's library, the controller may bottom any (Scry's library-side mirror). Decided inline (the `wants_ui` suspend prompt is a follow-up).
- ✅ **CR 701.57 — Discover N** — `Effect::Discover { n }`: exile from top until a nonland MV≤N, cast it free or put in hand (controller's choice), bottom the rest. Ships Geological Appraiser, Trumpeting Carnosaur. (Cascade-adjacent; shares the bottom-the-rest tail.)
- ✅ **CR 701.59 — Collect Evidence N** — `Effect::CollectEvidence { amount, then }`: optionally exile graveyard cards totaling MV≥N, then run the reflexive payoff. A `wants_ui` controller picks via `ChooseCards` (sum-validated); bots/tests keep the auto cheapest-pick. Ships Sample Collector, Izoni.
- ✅ **CR 602.5b — Additional activation costs (cont.)** — two new cost forms on `ActivatedAbility`: `bounce_other_filter` ("Return a [filter] you control to its owner's hand:" — Quirion Ranger, Wirewood Symbiote) and `tap_n_filter` ("Tap N untapped [filter] you control:", source eligible — Heritage Druid). Both gate pre-payment + auto-pick lowest-power, surface in `ability_cost_label`, and are excluded from the bot's `is_free_mana_ability`.
- ✅ **CR 701.16 / 614 — "Opponents can't make you sacrifice"** — `StaticEffect::OpponentsCantMakeYouSacrifice`, consulted in the `Effect::Sacrifice` resolver (skips a player whose opponent's effect would force a sacrifice; own-sacrifice unaffected). Ships Sigarda, Host of Herons + the sacrifice half of Tamiyo, Collector of Tales.
- 🟡 **CR 614 — Replacement Effects** — general "instead" framework. Damage *halving* ✅ (614.5 — `StaticEffect::HalveDamageDealt`, Ghosts of the Innocent; composed with doublers via `scale_damage` at both damage funnels). Skip-step (614.10) ✅ via `StaticEffect::SkipStep` consulted in `advance_step` — a skipped upkeep/draw never occurs (no turn-based actions, triggers, or priority); a skipped untap skips untapping/phasing/day-night but the turn still starts (Eon Hub, Stasis). Skip-*turn* ✅ (`Player.skip_turns`, Chronatog / Ral Zarek -7). Damage *redirection* (614.9) ✅ via `StaticEffect::RedirectDamageToSelf` at both damage funnels (Palisade Giant; one redirect per event per 614.5). (ETB-counters, token/counter/damage *doubling*, regen, EtbTriggerTax, Maze-of-Ith per-source prevention ✅. Creature-ETB / death **trigger suppression** ✅ via `StaticEffect::SuppressCreatureEtbTriggers { also_dies }` — Torpor Orb / Tocatli Honor Guard / Hushbringer; `etb_trigger_multiplier` returns 0 for creature entrants and the dies-trigger gather paths skip while a suppressor is in play.) Enters-*untapped* replacement ✅ — `StaticEffect::LandsEnterUntapped` overrides any enters-tapped effect for the controller's lands in `apply_enters_tapped_replacement` (Spelunking).
- 🟡 **CR 615.1 / 615.12 — Prevention effects** — global "combat damage can't be prevented" ✅ (`StaticEffect::CombatDamageCantBePrevented` — Frenzied Baloth; bypasses shields for any creature-sourced damage, sharing the Questing-Beast combat approximation). Per-source / per-N shields (Wojek Apothecary, Stave Off); non-combat prevention breadth — Mending Hands ✅ (next-4 shield on any target); prevent-and-gain ✅ via `Effect::PreventNextDamageAndGainLife` + `PreventionShield.gain_life` (Reverse Damage). Source-of-your-choice prevention (615.7) ✅ via
  `Effect::PreventAllDamageFromChosenSourceThisTurn` +
  `GameState.damage_prevented_sources`, consulted at both damage funnels
  (Burrenton Forge-Tender; the source is chosen as the ability resolves,
  among stack spells and battlefield permanents). Per-shield source
  restriction ✅ — `PreventionShield.{source,one_event}` +
  `Effect::PreventNextDamageFromChosenSource` (the damage source is now
  threaded through `apply_prevention_shields` at both funnels; Circle of
  Protection cycle, Rune of Protection: Red/Black).
- 🟡 **CR 500 — Turn structure** — `Predicate::CurrentStepIs(TurnStep)` gates "activate only during [your] upkeep/end step" abilities (Mirror Universe, Magus of the Mirror). Extra **combat-phase** insertion ✅ (CR 505.1b — `AdditionalCombatPhase` at End of Combat + `AdditionalCombatPhaseAfterMain` post-main re-entry, Relentless Assault). Phasing-in of extra non-combat steps still ⏳.
- ✅ **CR 702.113 — Awaken** — rides `AlternativeCost { target_filter, effect_override }`: awaken cast adds the counters + a permanent-duration `BecomeCreature` on the targeted land (Part the Waterveil).
- 🟡 **CR 305 — Lands** — see git for the per-clause detail. `LandType::Cave`
  added (CR 305.6 land subtypes), unblocking the LCI Cave lands + Caves-matter
  payoffs (Forgotten Monument grant, Compass Gnome tutor, Gargantuan Leech
  affinity, Spelunking).
- 🟡 **CR 701.48 — Learn** — populate Lesson sideboards in the format / draft deck-build paths (engine + cube ✅).
- 🟡 **CR 702.15 — Lifelink** — LKI corner (702.15c): triggered-ability source leaving the battlefield mid-resolution.
- 🟡 **CR 701.34 — Proliferate** — see git for detail.
- 🟡 **CR 601 — Casting Spells** (logged as "CR 706 — Casting spells") — minor; see git. "Opponents can't cast from anywhere but their hands" ✅ via `StaticEffect::OpponentsCantCastFromAnywhereButHand`, checked in `cast_from_zone_blocked`. The foretell / plot / adventure-creature exile-cast paths now gate on it too (`cast_foretold`/`cast_plotted`/`cast_adventure_creature`; test `drannith_magistrate_blocks_foretold_cast`). Remaining ⏳: suspend's eventual cast.
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
- 🟡 **CR 903 — Commander Variant** — MDFC back-face color identity (903.4d); 903.9 optional rider.

### Todo (⏳)
- ✅ **CR 612 — Text-Changing Effects** — layer-3 `Modification::ReplaceColorWord`
  / `ReplaceBasicLandType` + `Effect::ReplaceColorWord`/`ReplaceBasicLandType`
  (two ChooseColor prompts pick from/to; basics map 1:1 onto colors). Rewrites
  Protection-from-color, landwalk, and the type line (a swapped basic taps for
  the new color). Trait Doctoring (EOT + Cipher), Mind Bend (permanent).
  Remaining ⏳: full text-box swaps (Spy Kit, Volrath's Shapeshifter) and
  ability-text color words beyond keywords.

## Suggested next-up tasks

- ⏳ **recent20 (OTJ) approximations / follow-ups:** Magda, the Hoardmaster
  drops the "Sacrifice three Treasures: make a 4/4 Scorpion Dragon" ability
  (needs a fixed-count sacrifice-cost picker); Gisa's "Ward—{2}, Pay 2 life" is
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
  opponent rather than a chosen player (1v1-faithful). Charmed Sleep deferred —
  wants an Aura that taps + sets `untap_locked_by` on the host at ETB.
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
  number + arrange on top); Gilt-Leaf Archdruid (tap-seven-untapped-Druids cost
  + gain control of all lands); Divergent Transformations / Seeds-cycle's last
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

- ⏳ **Per-color mana-spent tracking** (unblocks Adamant, CR 702.137; and any
  "if N mana of one color was spent" rider). `ConvergedValue` tracks *distinct*
  colors but not per-color counts; `mana_spent` is a bare `u32`. Thread a
  per-color breakdown from `PaymentReceipt` (`pool_before` − post-pay pool) onto
  the spell stack item + `EffectContext`, then add `Predicate::ManaSpentOfColorAtLeast`.
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
| Bot play | Bots never cast prepare spells (their enters-prepared creatures keep the counter) | Bot evaluates `prepare_castable` like other affordances |
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

### Phase 1 — transactional `perform_action`
- ⏳ Checkpoint at the top of the action entry point: clone the state,
  restore on `Err`. Start with a full clone per *human-submitted* action
  (bots/tests can opt out); optimize later only if profiling demands it.
- ⏳ **Suspension is not failure**: `suspend_signal`/`pending_decision`
  mid-action are legitimate non-`Err` exits — the checkpoint must restore
  only on `Err`, and a checkpoint taken before a multi-step suspended
  action must survive until the resume chain completes or errors.
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
- ⏳ **Game-over stats** — turns, damage dealt, cards drawn, from the
  event stream; show on the game-over modal.

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

### Client GUI follow-ups
- **Mayhem cast affordance.** `GraveyardCardView.mayhem_cost` now surfaces a
  castable Mayhem cost (only after the card was discarded this turn) and the bot
  enumerates `CastMayhem`. The client GUI graveyard browser still needs a
  right-click "Cast for Mayhem" entry wired to `GameAction::CastMayhem`
  (mirror the flashback/disturb affordances). The GUI crate can't build in the
  headless cloud env (wayland-sys), so this wasn't verifiable here.
- **Disturb-into-Aura target picker.** The engine + bot now route an enchant
  target through `GameAction::CastDisturb` for Aura back faces. The GUI graveyard
  browser is read-only (it never built any graveyard-recast action), so a
  right-click "Cast for Disturb" entry — and, when the back is an Aura, a
  target-selection step — still needs wiring. Unverifiable headless (wayland).
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
