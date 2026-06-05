# Feature Roadmap — MTGO / Arena / XMage parity

A prioritized, exhaustive summary of capabilities to add, derived from a
codebase analysis against the three reference clients. This is a
*capabilities* roadmap (engine fidelity + UX + infra); per-card status
lives in `CUBE_FEATURES.md` / `DECK_FEATURES.md` / `STRIXHAVEN2.md` and the
approximations log in `TODO.md`.

Legend: ✅ done · 🟡 partial · ⏳ not started. Markers reflect a point-in-time
read of the code and should be re-verified before picking up an item.

---

## Already shipped (don't re-propose)

- **Core loop:** real LIFO stack + multiplayer priority loop, state-based
  actions, delayed triggers, intervening-`if` (CR 603.4), the layer system
  (CR 613), split first-strike / regular combat-damage steps.
- **Keywords:** Flying, Reach, Menace, Haste, Vigilance, First/Double
  Strike, Trample, Lifelink, Deathtouch, Infect, Wither, Defender,
  Protection(color), Hexproof, Shroud, Indestructible, Regenerate, Persist,
  Undying, Flash, Flashback (+ tap-cost variant), Kicker, Convoke, Delve,
  Cascade, Cycling, Echo, Cumulative Upkeep, Retrace, Phasing, Dredge,
  Annihilator, Banding, Equip, Fortify, Morph, Megamorph, Prowess, Ward,
  Changeling, Storm, Rebound, Crew, Exert, Shadow, Horsemanship, Intimidate,
  Skulk, Fear, Unblockable, plus uncounterable riders.
- **Costs/mana:** colored / generic / colorless / hybrid / mono-hybrid /
  Phyrexian / snow / X symbols; Convoke/Delve generic reduction; Commander
  tax; alternative (pitch) costs.
- **Objects:** tokens (Treasure / Clue / Blood / Food), counters,
  planeswalkers + loyalty, MDFC front/back casting, command zone +
  Commander.
- **Formats:** Standard, Commander, Brawl, Two-Headed Giant (+ teams).
- **Modes:** singleplayer vs. bot, **networked TCP multiplayer**
  (`server/tcp.rs`, keepalive), draft + cube, full-state **serde snapshots**
  (save/restore + round-trip replay foundation).
- **Client:** 3D board, game-log panel, targeting UI, decision UI,
  attack-all, priority-aware Pass/Respond button, counter tooltips,
  animations, keyboard cursor (incl. WUBRG hotkeys on the ChooseColor modal).
- **Misc primitives (claude/modern_decks):** layer-5 color change
  (`Effect::BecomeChosenColor`, CR 105.3 — Wild Mongrel); reveal-top-and-take-
  one-per-card-type (`Effect::RevealTopTakeOnePerType` — Atraxa); reveal-top-
  and-replay-permanent (`Effect::RevealTopPutPermanentOntoBattlefield` — Chaos
  Warp); "opponents can't cast noncreature spells this turn"
  (`Effect::CantCastNoncreatureThisTurn` — Ranger-Captain of Eos);
  "becomes the target" triggers now fire for SelfSource and the new
  `EventScope::YourPermanentTargetedByOpponent` (Goldspan Dragon, Phantasmal
  Image, Battle Mammoth, Tenured Concocter); CR 704.5j legend-rule controller
  choice (`Decision::ChooseLegendToKeep`); CR 800.4a player-leaving cleanup
  (`objects_leave_with_player`); CR 121.2b per-turn draw cap
  (`StaticEffect::CapDrawsPerTurn`, surfaced in `PlayerView.draw_cap` + HUD);
  dynamic MV-vs-graveyard target gate
  (`SelectionRequirement::ManaValueAtMostControllerGraveyard` — Drown in the
  Loch); self-buff scaled by controlled permanents
  (`StaticEffect::PumpSelfByControlledPermanents` + `TokenDefinition.
  static_abilities` — Karn, Scion of Urza's Construct token gets +1/+1 per
  artifact you control); player-chosen counts (`Decision::ChooseAmount` +
  `Effect::SacrificeAnyNumber` / `Effect::PayLifeLookTake` — Plunge into
  Darkness, with entwine modeled as `Keyword::Kicker` + `SpellWasKicked`);
  reveal-until-the-named-card (`SelectionRequirement::NamedBySource` +
  `named_card_this_resolution` — Spoils of the Vault); per-source "dealt
  damage to you this turn" tracking (`Player.creatures_that_damaged_me_this_turn`
  + `SelectionRequirement::DealtDamageToControllerThisTurn` — Spear of Heliod;
  combat damage to a player now also opens the Bloodthirst window); the
  Theros god-weapon abilities (Spear/Whip/Hammer); Fading / Vanishing
  (`Keyword::Fading(N)` / `Vanishing(N)` + `process_fading_vanishing`); the
  World rule SBA (CR 704.5k); linked-exile return-tapped
  (`ExileReturnZone::BattlefieldTapped` — Parallax Tide); the Enduring
  self-revive (`Effect::ReturnSelfAsEnchantment` — Enduring Innocence);
  end-of-combat token cleanup (`AttackingTokenCleanup` +
  `attacking_token_cleanup` — Mobilize sacrifice / Myriad exile, CR 511.3);
  the Army token + amass (`Effect::Amass`, CR 701.43); the Map explore-token
  (`map_token` — CR 111.10s); conditional `Value::IfAtLeast` (Mossborn Hydra's
  X≥4 doubling); one-shot "next instant/sorcery costs {N} less"
  (`Effect::GrantNextInstantOrSorceryDiscountThisTurn` + `pending_is_discounts`);
  **temporary control reversion** (`GameState.temporary_control` +
  `revert_temporary_control`, CR 800.4 — Act of Treason / Threaten now snap
  back to the pre-steal controller at end of turn / end of combat instead of
  stealing permanently); **opponent-attacks-you control flip**
  (`EventScope::ControllerAttackedByOpponent`, an `Attacks` listener on the
  defending player's permanents binding the attacker's controller into the
  target slot — Coveted Jewel gains control + untaps);
  **protection-from-color completeness** (CR 702.16e damage prevention in both
  combat-damage paths + the noncommat `deal_damage_to_from` path, and 702.16f
  equip legality, via `is_protected_from`; targeting now reads *computed*
  protection; combat-math preview + bot block eval honor it);
  `Keyword::CanAttackOnlyIfYouControl` (Lovestruck Beast),
  `Keyword::MustBlock` (CR 509.1c), `Effect::LoseHalfLife` (Stingerback),
  `Effect::GrantProtectionFromChosenColor` (Mother of Runes / Gods Willing /
  Brave the Elements), `Predicate::SpellsCastThisTurnEquals` +
  `PlayerRef::Triggerer`→caster/owner (Ledger Shredder, Sheoldred);
  **color-filtered anthems** (`AffectedPermanents::All.color` decoded from a
  `HasColor` requirement — Honor of the Pure, Crusade, Bad Moon);
  **reveal-top-opponent-chooses + exile-with-counter return**
  (`Effect::RevealTopOpponentChoosesToHand` / `ReturnFromExileWithCounter`
  + `CounterType::Silver` — Karn, Scion of Urza's real +1/−1; the opponent's
  pick is a heuristic like `Punisher`). Karn and Tezzeret, Cruel Captain now
  match their printed oracle text (Tezzeret: artifact-ETB loyalty trigger +
  0 untap-and-buff + −3 artifact tutor + −7 combat-counters emblem with a
  `BecomeCreature` Robot rider). **Block restrictions** (CR 509.1b/g):
  `Keyword::CantBeBlockedExceptBy(filter)` / `CantBeBlockedBy(filter)` (Silhana
  Ledgewalker, Steel Leaf Champion) read in `can_block_attacker_computed` and
  honored by the bot's block legality gate; `CantBeBlockedByMoreThanOne`
  (Charging Rhino). `SelectionRequirement::IsNonbasicLand` +
  `AffectedPermanents::CardMatch` (a card-local matcher routing disjunctive /
  nonbasic-land filters the conjunctive walker can't flatten — Thalia, Heretic
  Cathar). Per-player **half-of-their-own** effects
  (`Effect::{MillHalf, DiscardHalf, SacrificeHalf}` — Lord Xander now faithful).
  **Energy {E} payoffs** (CR 107.16): the `EventKind::EnergyGained` trigger
  ("whenever you get one or more {E}" — Aetherborn Marauder) and
  `Effect::PayEnergyOrElse` ("sacrifice/return unless you pay {E}…" — Lathnu
  Hellion, Greenbelt Rampager). **Populate** (CR 701.32 — `Effect::Populate`).
  **Choose-a-color mana rocks** (`Effect::ChooseColorForSelf` +
  `ManaPayload::ChosenColorOfSource` + `CardInstance.chosen_color` — Coldsteel
  Heart). **Horizon-land + verge-land helpers** (`horizon_land` completes the
  six-card cycle; `verge_land` ships the conditional second-color mana ability
  for the five enemy/allied verges). **Energy-gated activated abilities**
  (`ActivatedAbility.energy_cost`, CR 107.16 — Aether Hub / Servant of the
  Conduit's `{T}, Pay {E}: Add any color` now faithful instead of collapsing
  the split; spent up front like the mana/life pre-pay, affordance/bot gated
  via `would_accept`). **Additional combat phase** (`Effect::Additional
  CombatPhase` + `GameState.additional_combat_phases`, CR 505.1b — loops the
  turn back to Begin Combat for combat-activated extra-combat effects like
  Hellkite Charger). **Static-granted coin-flip advantage**
  (`StaticEffect::CoinFlipAdvantage` + `coin_flip_advantage_now`, CR 705.3 —
  Krark's Thumb-style flip advantage from a battlefield permanent).
  **Predicate-gated self-anthem** (`StaticEffect::PumpSelfIf` — live-evaluated
  +P/+T and optional keyword; Carnage Interpreter, Keen-Eyed Curator).
  **"Exiled with" linkage** (`CardInstance.exiled_with` +
  `Effect::ExileTaggedWithSource` + `Value::DistinctCardTypesExiledWith`, CR 720
  — Keen-Eyed Curator's exiled-card-types threshold). **Reveal-top-take-matching**
  (`Effect::RevealTopTakeMatchingToHand` — reveal N, take all cards matching a
  filter to hand, bottom the rest randomly; Torsten, Thundertrap Trainer).
  **Exile-all-but-bottom-N** (`Effect::ExileLibraryExceptBottom` — Doomsday
  Excruciator). **Energy O-Ring drawback** (`Effect::PayEnergyOrElse` wired to
  Static Prison's "sacrifice unless you pay {E}").
  **"Enchanted" state** (`SelectionRequirement::IsEnchanted`, CR 303 — Kestia's
  board-wide enchanted-attacker trigger; surfaced in `PermanentView.attachments`
  + client tooltip). **Backup N** (`shortcut::backup(n, keywords)`, CR 702.164 —
  ETB +N/+N on target + EOT keyword grant; Conclave Sledge-Captain,
  Death-Greeter's Champion). **First-spell cascade static** (Maelstrom Nexus via
  `Effect::Cascade { max_mv: ManaValueOf(TriggerSource) }`). **Flurry**
  (copy-your-second-spell-if-it-targets via `CopySpellMayChooseTargets` else
  draw — Shiko and Narset). **Aura "when this leaves" → enchanted creature**
  (`AttachedTo` last-known-info, CR 603.10 — Parallax Dementia's destroy-on-leave).
  **Board-scaled equip bonus** (`EquipBonus.scale` / `EquipScale`, layer 7c —
  the attached creature gets +P/+T per matching permanent the controller
  controls) + **living weapon** (ETB mint-a-Germ-and-attach via `CreateToken` +
  `Attach`/`LastCreatedToken`) — Nettlecyst. **Parity value check**
  (`Predicate::ValueIsOdd` — Sab-Sunen's odd-counter draw). **Token Mutavault**
  (a land token with `{T}: Add {C}` + a `BecomeCreature` animate ability —
  Mutable Explorer).

---

## Tier 1 — High-leverage engine primitives

Each unblocks a large swath of cards and removes the most visible "that's
not how Magic works" moments.

1. 🟡 **Replacement-effect framework.** A `replacement.rs` framework exists
   but only models zone-change replacements (Commander "→ command zone
   instead", CR 903.9b); the rest is stubbed per-card. Still to generalize:
   ETB replacement (enters with counters / as a copy / under your
   control; **enters tapped** now ships via `StaticEffect::EntersTapped` +
   `apply_enters_tapped_replacement` (CR 614.13 — Imposing Sovereign,
   Authority of the Consuls, Blind Obedience, Kismet); "exile non-cast
   nontoken creatures instead" ships via
   `StaticEffect::ExileNontokenCreaturesNotCast` — Containment Priest),
   damage *redirection* (Maze of Ith), draw/skip replacement, and
   "if it would die, exile instead." Counter-doubling (Doubling Season,
   Hardened Scales — `StaticEffect::DoubleCounters`) and damage-**doubling**
   (Furnace of Rath — `StaticEffect::DoubleDamageDealt`, applied to both the
   non-combat and combat damage paths, CR 614.2) now ship as multiplier
   replacements. Damage **prevention** is modeled as shields
   (`prevention_shields` + `Effect::PreventNextDamage` /
   `PreventAllDamageThisTurn` / `DamageCantBePreventedThisTurn`, CR
   615.1/615.7/615.12) for the non-combat damage path, plus the existing
   `prevent_combat_damage_this_turn` flag for combat fogs. **Per-source
   combat shields** now ship via `Effect::PreventAllCombatDamageInvolving`
   + `GameState.combat_damage_prevented_creatures` (Maze of Ith — prevent
   all combat damage to and by a creature this turn, CR 614.9). Remaining:
   true damage *redirection* and damage *halving*.
2. ✅ **Multi-pick / "choose N" decisions.** `Decision::ChooseModes` is
   wired (`game/effects/mod.rs`, `DecisionAnswer::Modes`). "Pick from
   revealed cards" is also wired: `Effect::LookPickToHand` (Impulse /
   Strategic Planning / Flow State) presents the peeked top-of-library set
   through the `SearchLibrary` decision; auto-decider keeps the top card.
3. ✅ **Player-chosen combat damage assignment order.**
   `Decision::CombatDamageOrder { attacker, blockers }` prompts the attacker
   (`combat.rs`, CR 510.1c) instead of sorting by CardId. (Trample-over-
   lethal / deathtouch spread math rides on top — see Tier 6.)
4. ✅ **Linked "until this leaves play" exile** (CR 603.6e).
   `Effect::ExileUntilSourceLeaves` (target a permanent) and
   `Effect::ExileChosenUntilSourceLeaves` (pick from a hand) stamp
   `CardInstance.exiled_by`; `return_linked_exiles` (called from every
   battlefield-removal path) returns the card to battlefield or hand when
   the source leaves. Wired Banisher Priest, Fiend Hunter, Oblivion Ring,
   Brain Maggot, Tidehollow Sculler.
5. 🟡 **Copy of a permanent (clone).** `Effect::BecomeCopyOf` (CR 707.2,
   one-shot definition rewrite) + the `CardDefinition.enters_as_copy` ETB
   hook (applied before SBA so a 0/0 copier never dies first) ship Clone
   and Phantasmal Image (Illusion + sacrifice-when-targeted rider), Mirror
   Image, Stunt Double. Copied ETB triggers re-fire (CR 707.5). Token
   copies ride `CreateTokenCopyOf` (Cackling Counterpart). Remaining:
   "becomes a copy" *continuous* layer-1 effects (Helm of the Host's
   haste-token loop, Mirrorform aura) and copied enters-with-counters.
6. ✅ **Copy-a-spell-on-the-stack.** `Effect::CopySpell` /
   `CopySpellUnlessPaid` ship Storm / sac-to-copy cards (copy keeps the
   original's targets), and `Effect::CopySpellMayChooseTargets`
   (CR 707.12 / 115.7) adds **new-target choice** on the copy — the copy's
   controller may repoint the primary target (original offered first, so
   AutoDecider keeps it). Ships Reverberate, Fork.

## Tier 2 — Engine rules fidelity (beyond Tier 1)

- 🟡 **APNAP trigger ordering** — inter-player APNAP (`game/mod.rs`
  apnap_rank sort, CR 603.3b) plus same-controller ordering: a `wants_ui`
  controller orders their own simultaneous triggers via
  `Decision::OrderTriggers` (`order_same_controller_triggers`, consulted
  synchronously); AutoDecider keeps the default order. The client modal is
  wired (`spawn_order_triggers_modal` + `handle_trigger_reorder`).
  Remaining: a server-side *suspend* path so a networked human is actually
  prompted (today the dispatch consults the decider inline, so a remote
  seat degrades to the default order). Tracked in TODO.md.
- 🟡 **Divided damage** across N targets — `Effect::DealDamageDivided` +
  `Decision::DivideDamage` ship Forked Bolt, Pyrokinesis, Crackle with Power,
  Magma Opus (AutoDecider spreads evenly; UI/scripted deciders choose the
  split). Remaining: divided *non-damage* riders ("tap up to N", split-mill)
  and true "choose targets as it resolves".
- ⏳ **Targeting refinements:** "up to N targets", "target each", "another
  target", same-target-twice rules, protection re-check on resolution.
- ⏳ **Continuous-effect breadth:** characteristic-defining abilities,
  type/color/text-changing effects (CR 613 layers 1–6 corner cases),
  "becomes a copy of" layer interaction, set-P/T vs +N/+N ordering.
- 🟡 **Static ability framework:** cost-reduction statics, "you may play"
  permissions from permanents, "creatures you control have X", anthem
  stacking — wired. Devotion-gated creature states (Nyx gods) ship via
  `StaticEffect::NotCreatureWhileDevotionBelow` (CR 700.5). Remaining:
  broader "you may play" permissions and devotion-gated *non-type* states.
- 🟡 **Replacement of life/draw/damage events** (ties to Tier-1 #1).
- ⏳ **Regeneration shields & "the next time" prevention** as proper shields
  rather than instantaneous.
- ⏳ **Damage marking vs. wither/-1/-1**, lethal/indestructible interplay
  audited against CR 120 / 704.
- ⏳ **Loyalty fidelity:** activate at sorcery speed once/turn (have), but
  also loyalty-set effects, "can be activated any time" riders, proliferate
  on loyalty, attacking planeswalkers redirect rules.
- ⏳ **State-based action coverage audit:** legend rule for planeswalkers
  (post-2017 unified rule), world rule, +1/-1 counter annihilation,
  saga-chapter sacrifice, attached-Aura falls off, token ceases to exist.

## Tier 3 — Object model & zones

- ⏳ **Battle card type** (CR 110.4) + defense counters +
  `AttackTarget::Battle` (noted in `TODO.md`).
- ⏳ **Sagas** (lore counters, chapter abilities, DFC sagas).
- ⏳ **Split cards** (CR 709) + **Fuse**.
- ✅ **Adventure** (CR 715) — `CardDefinition.adventure` + `CardInstance.
  {adventuring,on_adventure}` + `GameAction::CastAdventure` /
  `CastAdventureCreature`. The adventure half resolves down the spell path
  (its own effect, instant/sorcery type for Prowess/Magecraft), then exiles
  the card with permission to cast the creature half from exile. Ships
  Bonecrusher Giant // Stomp, Brazen Borrower // Petty Theft, Murderous
  Rider // Swift End, Foulmire Knight // Profane Insight, Order of Midnight
  // Alter Fate, Rimrock Knight // Boulder Rush, Garenbrig Carver // Shield's
  Might.
- ⏳ **Classes / Rooms / Cases / Backgrounds** (enchantment subtypes with
  level/door mechanics).
- ⏳ **Leveler cards** (level-up counters).
- ⏳ **Flip cards** (Kamigawa), **Meld** (Mightstone/Weakstone et al.),
  **Prototype**, **Omen**.
- ⏳ **Face-down permanents** generalized (Morph exists as a keyword; needs
  the 2/2-face-down object, manifest, disguise/cloak, cloak).
- ⏳ **Ante / conspiracy / dungeon (venture) / sticker / attraction** zones
  (low priority; only for novelty formats).
- ✅ **Emblems** as command-zone objects (planeswalker ultimates) —
  `Player.emblems` + `Effect::CreateEmblem`; triggers dispatch event-keyed
  (`emblem_event_matches`) and step-keyed (`fire_step_triggers`); surfaced in
  `PlayerView.emblems`. Wired Dellian Fel -6, Dakkon -6, Saheeli Rai -7.
- ⏳ **Sideboard zone** + "from outside the game" (wishes, companions).

## Tier 4 — Keyword & ability mechanics (the long tail)

Grouped roughly by how many cards each unlocks. Each is a small, targeted
feature; sweep card-batch by card-batch.

- **High frequency / modern staples:** ✅ Madness (`Keyword::Madness`,
  discard→exile→offer-cast in `discard_card`/`offer_madness_cast`, CR
  702.35), ✅ Escape (`Keyword::Escape(cost, n)` + `GameAction::CastEscape`,
  CR 702.139 — cast from graveyard for escape cost + exile N other gy cards;
  instants/sorceries re-escape), ✅ Adventure (CR 715, see Tier 3),
  ⏳ Soulbond, ⏳ Mutate, ⏳ Companion, ✅ Foretell
  (`CardDefinition.foretell_cost` + `GameAction::Foretell` /
  `CastForetold` — CR 702.143: pay {2} to exile face-down, cast from exile
  for the foretell cost on a later turn; Saw It Coming, Doomskar, Behold the
  Multiverse), ⏳ Disturb,
  ⏳ Daybound/Nightbound, ✅ Blitz (`shortcut::blitz` /
  `AlternativeCost.blitz` — CR 702.152: alt-cost haste + "when this dies,
  draw a card" + sacrifice at next end step via `Effect::SacrificeSource`;
  Tenacious Underdog, Ardent Elementalist, Goldhound), ✅ Casualty
  (CR 702.153 — `Keyword::Casualty(n)` + `GameAction::CastSpellCasualty`:
  optional sacrifice-a-creature-of-power-≥-n additional cost that copies the
  spell on cast via `copy_stack_spell`; Cut of the Profits),
  ✅ Connive (`shortcut::connive` — CR 702.158, draw/discard +
  +1/+1-per-nonland via `Selector::DiscardedThisResolution`; Quandrix
  Cryptomancer), ⏳ Backup,
  ⏳ Bargain, ⏳ Craft, ⏳ Disguise/Cloak, ✅ Plot (CR 702.170 —
  `CardDefinition.plot_cost` + `GameState.{plotted_cards,plotted_this_turn}`
  + `GameAction::Plot` / `CastPlotted`: exile face-up for the plot cost, cast
  free on a later turn; Spinewoods Paladin, Vault Plunderer),
  ✅ Saddle (CR 702.171 — `Keyword::Saddle(n)` + `CardInstance.saddled` +
  `GameAction::Saddle` + `shortcut::attacks_while_saddled`; Stingerback
  Terror), ⏳ Gift,
  ⏳ Offspring, ⏳ Impending, ✅ Ninjutsu (`Keyword::Ninjutsu(cost)` +
  `GameAction::Ninjutsu` — declare-blockers special action that returns an
  unblocked attacker and swaps the ninja in tapped + attacking; Fallen
  Shinobi), ⏳ Embalm/Eternalize.
- **Counter / +1+1 matters:** ✅ Proliferate (`Effect::Proliferate` —
  reducer-wired + tested in `tests::classic`), ✅ Bolster
  (`shortcut::bolster` — CR 701.21, +N/+N on the controller's
  `Selector::LeastToughnessYouControl`),
  ✅ Adapt (`shortcut::adapt` — CR 702.108, +N/+N if no +1/+1 counters;
  Pteramander), ✅ Training (`shortcut::training` — CR 702.149, +1/+1 when
  attacking with a higher-power creature via `PowerGreaterThanSource`;
  Pridemalkin), ✅ Evolve (`shortcut::evolve` +
  `SelectionRequirement::GreaterPowerOrToughnessThanSource` — Cloudfin
  Raptor, Experiment One, Fathom Mage), ✅ Mentor (`shortcut` — Sunhome
  Stalwart, CR 702.135),
  ✅ Modular (`shortcut::modular_dies` — CR 702.43, enters with N +1/+1
  counters + last-known-info counter transfer on death; Arcbound cycle),
  ✅ Graft (`shortcut::graft` — CR 702.57, move-a-counter when another
  creature enters; Aquastrand Spider, Plaxcaster Frogling, Cytoplast
  Root-Kin), ✅ Outlast (`shortcut::outlast` — CR 702.97, sorcery-speed
  tap-to-grow + `AllWithCounter` anthems; Abzan Falconer, Ainok Bond-Kin,
  Tuskguard Captain, Mer-Ek Nightblade), ✅ Renown (`shortcut::renown` —
  CR 702.111, +N on first combat damage; Topan Freeblade, Stalwart Aven,
  Skyraker Giant),
  ✅ Bloodthirst (`shortcut::bloodthirst` — CR 702.54, ETB-`If` gated on
  `Predicate::PlayerDamagedThisTurn` + `Player.was_dealt_damage_this_turn`;
  Scab-Clan Mauler, Gorehorn Minotaurs, Bloodfray Giant),
  ✅ Monstrosity (`shortcut::monstrosity` + `Effect::Monstrosity` +
  `CardInstance.monstrous` + `EventKind::BecameMonstrous`; Nessian Wilds
  Ravager, Ember Swallower),
  ✅ Devour (`shortcut::devour(n)` — CR 702.83, ETB `SacrificeAnyNumber`
  over other creatures, each sacrifice dropping N +1/+1 counters on the
  devourer via `Selector::This`), ✅ Amass (`shortcut::amass(n)` /
  `Effect::Amass` — CR 701.43; see Combat-flavor list).
- **Cast-from-elsewhere:** ⏳ cast-from-top (Mind's Desire / Amped Raptor /
  Robber of the Rich), ✅ Suspend (`Keyword::Suspend(n, cost)` +
  `GameAction::Suspend` + `process_suspend` — CR 702.62: pay the suspend cost
  to exile from hand with N time counters, tick one off per owner's upkeep,
  free-cast when the last is removed; Rift Bolt, Ancestral Vision, Lotus
  Bloom. Creature-suspend haste + a UI prompt for the free cast's targets are
  TODO.md follow-ups), ⏳ Forecast,
  ⏳ Hideaway, ⏳ Aftermath.
- **Combat-flavor:** ✅ Bushido / ✅ Flanking / ✅ Rampage
  (`Keyword::{Bushido,Flanking,Rampage}` — combat-step rules in
  `declare_blockers`),
  ✅ Provoke (`shortcut::provoke` — CR 702.39, on-attack untap + force-block
  via `Effect::Provoke` + `CardInstance.must_block`),
  ✅ Battle Cry (`shortcut::battle_cry` — Goblin Wardriver),
  ✅ Exalted (`shortcut::exalted` — Akrasan/Aven Squire, Silverquill
  Duelmaster), ✅ Frenzy (`shortcut::frenzy` — CR 702.68),
  ✅ Melee (`shortcut::melee` — CR 702.121, +1/+1 on attack; per-opponent
  tally collapses to one in the common single-defender case),
  ✅ Dash (`shortcut::dash` — CR 702.110, alt-cost haste + return-to-hand
  at next end step; Khans `sets::ktk`),
  ✅ Boast (`shortcut::boast` — CR 702.142, once-per-turn activated ability
  gated on `Predicate::SourceAttackedThisTurn`; Kaldheim `sets::khm`),
  ✅ Afflict (`shortcut::afflict` — CR 702.131, drains DefendingPlayer),
  ✅ Enlist (`shortcut::enlist` / `Effect::Enlist` — CR 702.151, on-attack
  taps the highest-power eligible creature and adds its power EOT),
  ✅ Mobilize (`shortcut::mobilize(n)` — CR 702.169, on-attack
  mints N 1/1 red Warriors tapped + attacking via `Effect::CreateTokenAttacking`
  with `AttackingTokenCleanup::SacrificeAtEndOfCombat`), ✅ Myriad
  (`shortcut::myriad` / `Effect::Myriad` — CR 702.115, on-attack mints a
  tapped+attacking copy of the source for each other opponent, exiled at end
  of combat), ✅ Amass (`shortcut::amass(n)` / `Effect::Amass` — CR 701.43,
  grows or creates a 0/0 black Army with N +1/+1 counters).
- **Value/ETB:** ✅ Investigate (`shortcut::investigate(n)` — CR 701.13,
  mints `clue_token()`s; Thraben Inspector. Sac-Clue payoff rides the
  token's printed `{2}, Sac: Draw`), ✅ Fabricate (`shortcut::fabricate` — CR 702.122, ETB
  `ChooseMode([+1/+1 counters, 1/1 Servo tokens])`), ✅ Riot
  (`shortcut::riot` — CR 702.137, ETB choose Haste-permanent or a +1/+1
  counter; Zhur-Taa Goblin, Frenzied Arynx),
  ✅ Raid (`shortcut::raid_etb` — CR 702.108 ability word, ETB gated on
  `Predicate::PlayerAttackedThisTurn`; Mardu Heart-Piercer),
  ✅ Afterlife (`shortcut::afterlife` — CR 702.135),
  ✅ Explore (`Effect::Explore` + `EventKind::Explored`, CR 701.40 — Merfolk
  Branchwalker, Jadelight Ranger, Wildgrowth Walker, Seekers' Squire,
  Emperor's Vanguard, Path of Discovery), ⏳ Squad, ⏳ Forage,
  ✅ Exploit (`shortcut::exploit(payoff)` — CR 702.105, ETB `MayDo`
  sacrifice-a-creature → run payoff; declining skips it),
  ✅ Extort (`shortcut::extort` — CR 702.99, SpellCast
  `MayPay({W/B}, drain 1)`; Basilica Screecher, Syndic of Tithes, Tithe
  Drinker, Kingpin's Pet), ⏳ Cohort, ✅ Support (`shortcut::support(n)` /
  `Effect::SupportCounters` — CR 701.32, a +1/+1 counter on each of up to N
  target creatures via the slot-based "up to N targets" machinery).
- **Spell-matters:** ✅ Escalate (`Effect::Escalate { modes,
  cost }` — CR 702.119; pick one or more modes, paying the escalate cost once
  per extra mode; Collective Brutality's discard-a-card), ⏳ Splice,
  ⏳ Replicate, ⏳ Overload, ⏳ Cipher, ⏳ Surge, ✅ Spectacle
  (`shortcut::spectacle` / `AlternativeCost.condition` —
  `Predicate::PlayerLostLifeThisTurn` + `Player.lost_life_this_turn`, CR
  702.111: cast for the spectacle cost if an opponent lost life this turn;
  Skewer the Critics, Light Up the Stage), ⏳ Addendum,
  ⏳ Conspire, ⏳ Demonstrate.
- **Resource systems:** ✅ Energy ({E}) — `Player.energy` pool +
  `Effect::AddEnergy` / `Effect::PayEnergy`; surfaced in `PlayerView.energy`
  + HUD chip; bot spends surplus via `pick_energy_payoff`. Wired the
  Kaladesh set (`sets::kld`: Longtusk Cub, Bristling Hydra, Dynavolt Tower,
  Aether Swooper, Glimmer of Genius, …). Energy-gated *mana* abilities now
  ship via `ActivatedAbility.energy_cost` (Aether Hub's `{T}: Add {C}` +
  `{T}, Pay {E}: Add any color`, Servant of the Conduit). ⏳ Experience
  counters,
  ✅ Poison/Toxic (`Keyword::Toxic(N)` adds N poison on combat damage,
  CR 702.180c; 10-poison loss SBA wired),
  ✅ Devotion (CR 700.5 — `Value::DevotionTo`,
  `StaticEffect::NotCreatureWhileDevotionBelow` god gate,
  `ManaPayload::DevotionOfChosenColor`; surfaced in `PlayerView.devotion`
  + HUD chip), ✅ **Ascend / city's blessing** (CR 702.131 / 700.6 —
  `Effect::Ascend` grants `Player.city_blessing` at ten+ permanents,
  `Predicate::HasCityBlessing` gates payoffs; surfaced in
  `PlayerView.has_city_blessing` + `CityBlessingGained` log event),
  🟡 **Day/Night** (CR 731 + 502.2 — `GameState.day_night` +
  `Effect::BecomeDay`/`BecomeNight` + `Predicate::IsDay`/`IsNight`; the
  502.2 turn-based day↔night transition is wired off the previous turn's
  active-player spell count; surfaced in `ClientView.day_night` +
  `DayNightChanged` log. Remaining: Daybound/Nightbound DFC transform),
  🟡 Initiative / **monarch ✅** (CR 724 — `GameState.monarch` +
  `Effect::BecomeMonarch`; the monarch draws at their end step, combat damage
  to the monarch steals the crown, leaves-game transfer per 724.3; surfaced in
  `PlayerView.is_monarch` + `MonarchChanged` log event. The Initiative/
  Undercity dungeon is still ⏳), ⏳ Day/Night,
  ⏳ Ring-bearer (the Ring tempts you).
- **Fading family:** ✅ Fading (`Keyword::Fading(N)`, CR 702.32),
  ✅ Vanishing (`Keyword::Vanishing(N)`, CR 702.62) — enter with N fade/time
  counters; `process_fading_vanishing` ticks them down at the controller's
  upkeep and sacrifices when empty. Parallax Nexus / Parallax Tide ship the
  keyword (Tide uses `ExileReturnZone::BattlefieldTapped` linked exile).
  Remaining: Parallax Dementia's steal-on-leave rider.
- **Older mechanics:** ✅ Soulshift (`shortcut::soulshift(n)` — CR 702.46,
  dies → `MayDo(return target Spirit MV≤n from your graveyard)`), ⏳ Offering, ⏳ Epic, ⏳ Absorb,
  ⏳ Affinity (have artifact count?), ⏳ Entwine,
  ✅ Buyback (`Keyword::Buyback(cost)` + `GameAction::CastSpellBuyback` —
  CR 702.27, optional additional cost; bought-back spell returns to its
  owner's hand instead of the graveyard on resolution; surfaced in
  `PlayerView.buyback_hand`; Corpse Dance), ⏳ Miracle,
  ⏳ Bloodrush, ✅ Unleash (`Keyword::Unleash` + `shortcut::unleash()` — CR
  702.98: ETB "may enter with a +1/+1 counter" + computed `CantBlock` while it
  has one; Rakdos Cackler, Gore-House Chainwalker, Spawn of Rix Maadi),
  ⏳ Scavenge,
  ✅ Bestow (`CardDefinition.bestow` + `GameAction::CastBestow` +
  `CardInstance.bestowed` — CR 702.103; cast an enchantment-creature as an
  Aura for its bestow cost, granting its `equipped_bonus`; not a creature
  while bestowed (`compute_permanent` strips the type); reverts to a
  creature when its host leaves (SBA); surfaced in `PlayerView.
  bestowable_hand`; Baleful Eidolon), ⏳ Tribute.

## Tier 5 — Mana & cost system

- ⏳ **Mana provenance tag** — fixes Fellwar Stone, Locus scaling
  (Cloudpost/Glimmerpost), Cavern type-gated uncounterability in one shot.
- ⏳ **Per-source mana restrictions** ("spend only on X", filter lands).
- ⏳ **Minimum-cost floor** (Trinisphere) and **cost-increase statics**
  beyond the existing first-spell tax (Thalia, Sphere of Resistance).
- ⏳ **Conditional / additional costs** as a general modal layer (sacrifice,
  discard, pay life, exile-from-gy as alt/escape costs, tap creatures).
- ⏳ **{X} in activated abilities** generalized; **delve/convoke colored**
  contribution (currently generic-only).
- ⏳ **Snow-mana-only** and **mana-value-X** cost gates.

## Tier 6 — Combat fidelity

- ⏳ **Damage assignment order** (Tier-1 #3) and **trample math** with
  multiple/deathtouch blockers.
- ⏳ **Banding** combat rules (keyword exists; rules not wired).
- 🟡 **Multiple combat phases / extra attack steps.** `Effect::Additional
  CombatPhase` + `GameState.additional_combat_phases` (CR 505.1b) loop the
  turn back to Begin Combat when the active player leaves End of Combat with
  a phase banked — built for combat-activated extra-combat effects (Hellkite
  Charger), usually paired with `Untap`. Remaining: main-phase-cast "after
  this main phase, an additional combat phase followed by an additional main
  phase" sorceries (Relentless Assault) need post-main insertion.
- 🟡 **"Must attack/block", "can't attack alone", "attacks each combat"**
  restrictions and requirements. `Keyword::CantAttack` / `CantBlock`
  (Pacifism), `Keyword::AttacksAlone` (CR 508.0 — Master of Cruelties),
  `Keyword::MustBeBlocked` (CR 509.1c — "must be blocked", Academic
  Dispute), `Keyword::AllMustBlock` (CR 509.1c true Lure — every able
  creature must block; Lure aura), `Keyword::MustAttack`
  (CR 508.1d — "attacks each combat if able", Juggernaut), and
  `Keyword::CanAttackOnlyIfDefenderControls(filter)` (per-attacker attack
  gate on the defending player's board — Dandân's "can't attack unless
  defending player controls an Island") are wired from
  computed keywords in `declare_attackers`/`declare_blockers`. **Goad**
  (CR 701.38) is wired via `CardInstance.goaded_by` + `Effect::Goad`
  (treated as must-attack, clears at the goader's next untap — Disrupt
  Decorum). Still open: *granted* must-attack with a future-turn-scoped
  duration ("attacks next turn if able" — Big Play mode 0); the
  goaded "attack a player other than the goader" clause in multiplayer;
  cost-to-block (509.1d-f). **Blocker-side `Keyword::MustBlock`** (CR 509.1c
  — "blocks each combat if able") is now wired in `declare_blockers`.
- ⏳ **Planeswalker / Battle as attack targets** UI + redirection.
- ✅ **Goad** (above). ✅ **Lure** (`Keyword::AllMustBlock`).
  ✅ **Provoke** (`shortcut::provoke`, CR 702.39).
  ✅ **Ninjutsu attacking-creature swap** (`GameAction::Ninjutsu`, CR 702.49).

## Tier 7 — UI / UX core (the Arena "feel" gap)

Mostly buildable on existing `ClientView` / `StackItemView` data.

1. ⏳ **Big card-zoom preview on hover** — table-stakes; only a counter
   tooltip exists today.
2. ⏳ **Stops / auto-yield configuration** — per-phase stops + "yield until
   something needs me" (priority plumbing already exists; today only Pass /
   End Turn / Next Turn).
3. ⏳ **Combat math / damage preview** — projected life swing + which
   creatures die on declared attacks/blocks.
4. ⏳ **Undo / mana-tap rollback** — undo un-committed taps before a spell
   locks in (`ManualTapRequired` already signals partial manual-tap model).
5. 🟡 **Targeting arrows on the stack** — `KnownStackItem` now carries
   `additional_targets` (all slots, not just slot 0), so the client can draw
   an arrow to every target of a multi-target spell. Arrow rendering itself
   still ⏳.
6. ⏳ **Hold-priority toggle** ("F" key auto-pass; shift-hold to keep
   priority after your spell resolves).
7. ⏳ **Stack visualization** with response affordances and "respond / let
   resolve" per item.
8. ⏳ **Phase bar / step indicator** with click-to-advance and stop markers.

## Tier 8 — UI / UX quality-of-life

- ⏳ Browsable **graveyard / exile / library-count** zoom per player.
- ⏳ **Search / Scry / Surveil / Mulligan** dedicated picker UIs (drag,
  reorder, bottom).
- ⏳ Confirm **London mulligan** bottoming + scry-on-keep.
- ⏳ **Floating life deltas** + per-turn life-history graph.
- ⏳ **Hand sorting / auto-tap preferences / "play tapped land" prompt**.
- ⏳ **Reminder text & rules tooltips** on keywords; **oracle text panel**.
- ⏳ **Hotkey legend / help overlay**; remappable keys.
- 🟡 **Highlight legal plays** (castable cards, legal attackers/blockers,
  legal targets) — `ClientView` now carries `castable_hand`,
  `pitchable_hand`, `kickable_hand`, **`activatable_permanents`**, and
  **`legal_attackers` / `legal_blockers`** (step-aware; honor
  tapped/sickness/Defender and per-attacker block legality). Remaining:
  per-target hint layers (`legal_target_filter` exists to build on).
- ⏳ **Animations & SFX** polish; **board-state pings / alerts**
  (low life, triggers waiting, your turn).
- ⏳ **Settings menu** (graphics quality exists; add audio, gameplay,
  accessibility tabs).
- ⏳ **Battlefield organization** (auto-tuck lands, group tokens, stack
  identical permanents).

## Tier 9 — Multiplayer & social

- ⏳ **Lobby / matchmaking** (host, join-by-code, quick-match).
- ⏳ **Reconnect / resume** a dropped game (snapshots make this feasible).
- ⏳ **Spectator mode** (read-only `ClientView` stream).
- ⏳ **Chat + emotes** (Arena's canned phrases; XMage free chat).
- ⏳ **Per-turn / per-game timers, chess-clock, "rope," and timeouts.**
- ⏳ **Friends / invites / ratings / leaderboards** (server-side).
- ⏳ **Free-for-all politics** UI (deals, voting, monarch/initiative
  passing) for 3+ player tables.

## Tier 10 — Formats & match structure

- ⏳ **Best-of-3 + sideboarding** flow (core competitive structure).
- ⏳ **Deck legality validation** per format (banlist, size, singleton,
  color identity for Commander).
- ⏳ **More 60-card formats:** Modern, Pioneer, Legacy, Vintage, Pauper
  (mostly banlist/pool config on top of existing rules).
- ⏳ **Limited match rules** (40-card, basic-land access).
- ⏳ **Multiplayer variants:** Planechase (planar deck + dice),
  Archenemy (scheme deck), Commander variants (Oathbreaker, Brawl exists),
  Star, Emperor.
- ⏳ **Casual toggles:** free mulligans, starting-hand rules, vanguard.

## Tier 11 — Limited (draft / sealed)

- ✅/🟡 **Draft + cube** exist. Extend with:
- ⏳ **Sealed** (open packs, build pool).
- ⏳ **Bot drafters** with signal/pick heuristics (beyond random).
- ⏳ **Draft variants:** Winston, Rochester, Grid, Solomon, Glimpse, Team.
- ⏳ **Set-based draft** (pack composition by rarity/collation).
- ⏳ **Draft replay / pick history / pool export.**

## Tier 12 — Deckbuilding & collection

- ⏳ **In-app deck builder** (search by name/type/cost/keyword, curve view,
  legality check, sample-hand tester).
- ⏳ **Import / export** (Arena/MTGO/.dec/.cod text formats).
- ⏳ **Deck stats** (mana curve, color pips, type breakdown).
- ⏳ **Collection / ownership tracking** (if a progression layer is wanted).
- ⏳ **Card search engine** over the catalog (Scryfall-like syntax).

## Tier 13 — AI

- 🟡 **Smarter combat** — `server/bot.rs` blocking is heuristic (value
  trades, first-strike/deathtouch/trample awareness, gang-block-to-survive
  lethal) and attacking has a suicide filter plus evasion awareness
  (first-strike, deathtouch, menace, lifelink, trample, indestructible) and
  planeswalker redirection. Remaining: race math / when-to-hold-back across
  turns, multi-blocker attacker math, and attacking-into-open-mana respect.
- ⏳ **Better sequencing** (land drops, hold-up interaction, when to cast).
- 🟡 **Mulligan decisions** — `RandomBot` ships flooded/screwed opening
  hands via `decide_mulligan`: keep 2–5 lands **and** at least one nonland
  spell castable early (mana value ≤ lands + 1, with **color-screw
  awareness** — the lands must produce the spell's colored pips), stop after
  two mulligans. Remaining: transitive fetch/dual color sources (a lone
  fetchland still reads as colorless).
- ⏳ **Targeting / mode / X-value choices** by evaluation, not first-legal.
- ⏳ **Difficulty levels**; optional **search-based AI** (MCTS over the
  deterministic engine + snapshot cloning).

## Tier 14 — Replays, analysis & observability

- ⏳ **Action-log replay viewer** (step forward/back; snapshots + the
  `GameEvent` stream are the foundation).
- ⏳ **Game history / match results** persistence.
- ⏳ **Export game to shareable file**; import to reproduce bugs (the audit
  workflow already uses snapshots — formalize it).
- ⏳ **In-game "what happened" event filtering** in the log (by player,
  zone, type).

## Tier 15 — Accessibility

- ⏳ **Colorblind-safe** mana/color indicators (not color alone).
- ⏳ **Text scaling / high-contrast / reduced-motion** options.
- ⏳ **Full keyboard play** (cursor exists; complete the coverage).
- ⏳ **Screen-reader / narration** of board state and prompts.
- ⏳ **"Full control" mode** (XMage) — never auto-skip priority/steps.

## Tier 16 — Infra, correctness & content tooling

- ⏳ **Seeded / deterministic RNG** surfaced for reproducible games & tests.
- ⏳ **Snapshot round-trip property tests** + **fuzzing** of action
  sequences against SBA invariants.
- ⏳ **Crash-recovery / autosave** from snapshots.
- ⏳ **Card-scripting DSL or macro layer** to reduce catalog boilerplate
  (the catalog is large and hand-written).
- ⏳ **Set / Scryfall import pipeline** + automated data verification
  (`scripts/verify_cards.py` exists — extend it).
- ⏳ **Card art / image pipeline** for the client.
- ⏳ **Rules-engine conformance suite** mapped to CR section numbers.

---

## Suggested sequencing

1. **Replacement-effect framework** (Tier-1 #1) — the highest-leverage
   primitive still open. (Combat damage-order and multi-pick "choose N"
   decisions, formerly bundled here, are now wired.)
2. **Card-zoom preview + stops/auto-yield + combat-math preview**
   (Tier-7 #1–3) — the trio that most closes the "feels like Arena" gap.
3. **Best-of-3 + sideboard + deck legality** (Tier 10) — makes draft/cube
   and constructed competitive.
4. **Static-ability framework + mana provenance** — broad correctness wins
   that unblock many cards at once. (Inter-player APNAP ordering is already
   wired; only same-controller trigger ordering remains.)
5. **Smarter AI blocking** (Tier 13) — biggest single-player upgrade.
6. Then the **Tier-4 mechanic sweep** and **Tier-3 object-model** features,
   card batch by card batch, promoting entries in the per-card trackers.
7. **Replays, spectator, social, accessibility** as the product matures.
