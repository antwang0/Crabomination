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

A terse capability checklist — these are wired; don't re-propose them. The
exhaustive primitive-by-primitive list (and every card that exercises each) was
elided in a doc-compaction pass; recover it from `git log -p -- FEATURE_ROADMAP.md`
and the rules-coverage audit in `TODO.md`.

- **Core loop:** real LIFO stack, multiplayer priority loop, state-based
  actions, delayed triggers, intervening-`if` (603.4), the layer system (613),
  split first-strike / regular combat-damage steps, APNAP ordering.
- **Keywords (~120):** evasion + combat (Flying/Reach/Menace/First+Double
  Strike/Trample/Deathtouch/Lifelink/Vigilance/Defender/Protection/Hexproof/
  Shroud/Ward/Indestructible/Bushido/Flanking/Rampage/Provoke/Melee/Dash/Boast/
  Afflict/Enlist/Mobilize/Myriad/Ninjutsu/Goad/Lure…); ETB/value (Persist/
  Undying/Riot/Fabricate/Afterlife/Explore/Exploit/Extort/Investigate/Support/
  Embalm/Eternalize/Backup/Soulbond/Mentor); counter-matters (Proliferate/
  Bolster/Adapt/Training/Evolve/Modular/Graft/Outlast/Renown/Bloodthirst/
  Monstrosity/Devour/Amass); cast-mode + alt-cost (Kicker/Casualty/Connive/
  Offspring/Plot/Saddle/Blitz/Spectacle/Escalate/Buyback/Bestow/Foretell/
  Suspend/Flashback/Madness/Escape/Adventure/Cascade/Storm/Convoke/Delve);
  plus Phasing-adjacent Fading/Vanishing, Cumulative Upkeep, Echo, Dredge,
  Retrace, Morph/Megamorph, Crew/Reconfigure, Changeling, Soulshift, Unleash,
  Devoid (CDA colorless), Ingest.
- **Costs/mana:** colored/generic/colorless/hybrid/mono-hybrid/Phyrexian/snow/X;
  Convoke/Delve generic reduction; Commander tax; alternative (pitch) costs;
  energy-gated mana abilities; X-cost activated abilities.
- **Resource systems:** Energy {E}, Poison/Toxic, Devotion, Ascend/city's
  blessing, Monarch, Day/Night, coin-flip + die-roll randomization.
- **Objects:** tokens (Treasure/Clue/Blood/Food/Map/Army/Germ), counters
  (incl. keyword/shield/stun/finality/rad), planeswalkers + loyalty + **emblems**,
  MDFC front/back, split // fuse // aftermath, adventure, command zone + Commander,
  manlands (`BecomeCreature`), living weapon, clones/token-copies/spell-copies.
- **Replacement effects:** enters-tapped, enters-with-counters, token/counter/
  damage **doubling**, **mana-doubling** (CR 701.10f, `ManaProductionDoubled`),
  regeneration, EtbTriggerTax, Maze-of-Ith per-source combat-damage prevention,
  prevention shields, finality exile-instead. Counters cease on zone change
  (CR 122.2). Counter types incl. `-0/-1` / `-1/-0`.
- **Statics (misc):** no-maximum-hand-size, play-lands-from-graveyard
  (`PlayLandFromGraveyard` action), artifact/creature non-mana-ability locks,
  spell-tax (`AdditionalCost`, incl. nonartifact). Two-player coin-flip-off
  (`Effect::ManaClash`), reveal-top-land-to-battlefield-else-hand.
- **Misc primitives:** per-card board-bounce to each owner
  (`PlayerRef::OwnerOfMoved`; Aetherize/Evacuation), set-all-life-to-lowest
  (`Value::LowestLifeTotal`; Repay in Kind), step-gated activated abilities
  (`Predicate::CurrentStepIs`; Mirror Universe/Magus of the Mirror upkeep gate),
  sacrifice-unless-pay-mana (`Effect::PayManaOrElse`; Archway Commons),
  single-player graveyard exile (`Effect::ExilePlayerGraveyard`; Go Blank),
  exchange-control of two permanents (`Effect::ExchangeControl`, CR 701.12;
  Switcheroo).
- **Formats/modes:** Standard, Commander, Brawl, Two-Headed Giant (+ teams);
  singleplayer vs. bot, networked TCP multiplayer, draft + cube, Learn/Lessons
  sideboard, full-state serde snapshots (save/restore + replay foundation).
- **Ability/trigger riders:** once-per-turn triggers (`EventSpec::once_per_turn`,
  CR 603.3d), opponents-only activations (`ActivatedAbility.opponents_only`, CR
  602.5), discard-self activation cost (`discard_self_cost`), counter-to-exile
  (`CounterUnlessPaid.exile`), `Predicate::PlayerSacrificedThisResolution`,
  slot-parameterized `WhenTargetDiesThisTurn`, `Effect::DoublePower`,
  `Effect::ExileReturnNextEndStep` (blink-return-EOT), `Value::CountMatching`.
- **Client:** 3D board, game-log panel, targeting + decision UI, attack-all +
  per-attacker picking, priority-aware Pass/Respond, counter tooltips, card-zoom
  hover preview, animations, keyboard cursor (incl. WUBRG hotkeys), commander-
  damage HUD, legal-play highlighting, monarch/day-night/blessing chips.

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
   Authority of the Consuls, Blind Obedience, Kismet; **also self-source**
   `EntersTapped { This }` so a permanent can enter tapped on its own —
   Overlord of the Hauntwoods' "Everywhere" omniland); "exile non-cast
   nontoken creatures instead" ships via
   `StaticEffect::ExileNontokenCreaturesNotCast` — Containment Priest;
   **opponent's nontoken creature would-die → exile instead** ships via
   `StaticEffect::ExileDyingOpponentCreatures { when_you_do }` checked at the
   shared `remove_from_battlefield_to_graveyard` funnel, with an optional
   reflexive "when you do" — Valentin, Dean of the Vein),
   damage *redirection* (Maze of Ith) and draw replacement. **Skip-step**
   (CR 614.10) ships via `StaticEffect::SkipStep` (Eon Hub, Stasis);
   skip-*turn* remains.
   **Creature-ETB control replacement** ships via
   `Effect::StealCreatureEtbThisTurn` + `apply_etb_control_replacement`
   (Gather Specimens; token mints don't consult it yet).
   **Graveyard→exile hate** ships via
   `StaticEffect::ExileCardsBoundForGraveyard { opponents_only }` +
   `GameState::route_to_graveyard` (CR 614.6 — Rest in Peace, Leyline of
   the Void), centralizing every graveyard-placement site. Counter
   placement can be globally locked via `StaticEffect::CountersCantBePlaced`
   (CR 122.1 — Solemnity). Counter-doubling (Doubling Season,
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
   all combat damage to and by a creature this turn, CR 614.9). **Damage
   redirection** (CR 614.9) ships via `StaticEffect::RedirectDamageToSelf`
   at both damage funnels — Palisade Giant. **Draw replacement** (CR 121.2a)
   ships via `StaticEffect::ControllerDrawsDoubled` in `draw_one` (Thought
   Reflection; stacks per 614.5). Damage *halving* ✅ (614.5 —
   `StaticEffect::HalveDamageDealt`, Ghosts of the Innocent). Skip-step ✅;
   skip-*turn* ✅ (`Player.skip_turns`).
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

- ✅ **APNAP trigger ordering** — inter-player APNAP (`apnap_rank` sort,
  CR 603.3b) plus same-controller ordering with a real server suspend:
  `continue_trigger_ordering` parks the dispatch in
  `ResumeContext::TriggerOrder` and sets `pending_decision =
  OrderTriggers`, so a networked `wants_ui` seat is prompted; the client
  modal applies the order on resume.
- 🟡 **Divided damage** across N targets — `Effect::DealDamageDivided` +
  `Decision::DivideDamage` ship Forked Bolt, Pyrokinesis, Crackle with Power,
  Magma Opus (AutoDecider spreads evenly; UI/scripted deciders choose the
  split). Remaining: divided *non-damage* riders ("tap up to N", split-mill)
  and true "choose targets as it resolves".
- 🟡 **Targeting refinements:** resolution-time legality re-check (CR 608.2b)
  ships for single-target spells aimed at battlefield permanents (zone-gone /
  filter-mismatch / granted Hexproof-Shroud → fizzle to graveyard). Remaining:
  "up to N targets", "target each", "another target", multi-target partial
  fizzle, protection-from-color re-check.
- 🟡 **Continuous-effect breadth:** layer-3 text-changing ✅ (CR 612 —
  `ReplaceColorWord`/`ReplaceBasicLandType`; Trait Doctoring, Mind Bend);
  land-type statics ✅ (`StaticEffect::LandTypeChanger` — Blood Moon, Magus
  of the Moon, Urborg, Yavimaya). Remaining: CDA corner cases, full
  text-box swaps, "becomes a copy of" layer interaction.
- 🟡 **Static ability framework:** cost-reduction statics, "you may play"
  permissions from permanents, "creatures you control have X", anthem
  stacking — wired, including **disjunctive multi-type "other … you control"
  anthems** (the `affected_from_requirement` / `CardMatch` path now matches
  `Or`-of-creature-type filters and applies `OtherThanSource` against the live
  source id — Blex, Vexing Pest's Pest/Bat/Insect/Snake/Spider lord).
  Devotion-gated creature states (Nyx gods) ship via
  `StaticEffect::NotCreatureWhileDevotionBelow` (CR 700.5). Keyword **loss**
  ships via `StaticEffect::LoseKeyword` (layer-6 `RemoveKeyword`; Nowhere to
  Run strips opponents' Hexproof/Shroud), and `check_target_legality` reads
  layer-computed Hexproof/Shroud so granted *and* stripped evasion both
  affect targeting (fixes Lightning Greaves' granted Shroud). Remaining:
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
- 🟡 **Sagas** (CR 714). `CardDefinition.saga_chapters: Vec<(u32, Effect)>` +
  `GameState::saga_advance`: enters with one lore counter (chapter I fires),
  gains one each precombat main (turn-based action), and is sacrificed by SBA
  once lore counters reach the final chapter and no chapter ability is still on
  the stack. Ships History of Benalia, The Eldest Reborn. Remaining ⏳: DFC
  sagas (transforming back face) and read-ahead/chapter-choice variants.
- ✅ **Split cards** (CR 709) + **Fuse** (CR 702.102). The left half lives on
  the main `CardDefinition` (cast via the normal path); `CardDefinition.split:
  Some(SplitCard{ right, fuse })` carries the right half + Fuse flag. Cast the
  right half via `GameAction::CastSplitRight`, or both fused via
  `CastSplitFused` (combined cost; left target rides `target`, right target
  rides `additional_targets` slot 0, resolved in a second pass). `CardInstance.
  split_cast` marks which half(s) are on the stack. Affordance surfaced via
  `HandAffordances.splittable_right` → `PlayerView.splittable_right_hand`.
  Ships Wear // Tear.
- ✅ **Adventure** (CR 715) — `CardDefinition.adventure` + `CardInstance.
  {adventuring,on_adventure}` + `GameAction::CastAdventure` /
  `CastAdventureCreature`. The adventure half resolves down the spell path
  (its own effect, instant/sorcery type for Prowess/Magecraft), then exiles
  the card with permission to cast the creature half from exile. Ships
  Bonecrusher Giant // Stomp, Brazen Borrower // Petty Theft, Murderous
  Rider // Swift End, Foulmire Knight // Profane Insight, Order of Midnight
  // Alter Fate, Rimrock Knight // Boulder Rush, Garenbrig Carver // Shield's
  Might.
- 🟡 **Classes / Cases / Backgrounds** (enchantment subtypes with level
  mechanics). **Rooms ship** (CR 709.5 — `CardDefinition.room` +
  `CardInstance.unlocked_doors`; `GameAction::CastRoomDoor` /
  `UnlockRoomDoor`, door-scoped abilities via live definition rebuild,
  `EventKind::DoorUnlocked`; Unholy Annex // Ritual Chamber).
- ⏳ **Leveler cards** (level-up counters).
- ✅ **Transforming DFCs** (CR 712) — `Effect::Transform` toggles a
  permanent's active face in place (`CardInstance.{transformed,front_face}`,
  same object: counters/tapped/attachments persist), fires
  `EventKind::Transformed` for "when this transforms" triggers, and round-trips
  through both the serde and `GameSnapshot` paths (front name + flag rebuild
  the back face). Ships Concealing Curtains, Delver of Secrets, The Everflowing
  Well. Remaining: Daybound/Nightbound auto-flip, DFC sagas.
- ✅ **Meld** (CR 701.37/712.16) — `Effect::Meld` exiles both own+controlled
  components and mints the melded card with the parts stashed in
  `CardInstance.meld_parts`; every leave-battlefield funnel unmelds back into
  both cards. Urza, Lord Protector + The Mightstone and Weakstone → Urza,
  Planeswalker (loyalty-twice-each-turn override, CR 606.3).
- ⏳ **Flip cards** (Kamigawa), **Prototype**, **Omen**.
- 🟡 **Face-down permanents** — the 2/2-face-down object ships (CR 708):
  `CardInstance.face_up_def` stashes the real card while `definition` is the
  vanilla 2/2 (`facedown_creature_definition`), restored on leaving the
  battlefield (708.10) and round-tripped through serde. `Effect::Manifest` /
  `ManifestDread` (701.34 / 702.166) + `GameAction::TurnFaceUp` (708.5,
  Morph/manifest cost) + `EventKind::TurnedFaceUp` (708.8). Hauntwoods
  Shrieker, Ainok Survivalist (Morph/Megamorph cast-face-down ✅ via
  `GameAction::CastFaceDown`). Remaining ⏳: Disguise/Cloak.
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
  ✅ Soulbond (see CUBE_FEATURES — `SoulbondBonus`, pairs auto-resolve), ⏳ Mutate, 🟡 Companion (CR 702.139 — `Keyword::Companion` + `GameAction::CompanionToHand`, {3} sideboard→hand; deck validation ⏳), ✅ Foretell
  (`CardDefinition.foretell_cost` + `GameAction::Foretell` /
  `CastForetold` — CR 702.143: pay {2} to exile face-down, cast from exile
  for the foretell cost on a later turn; Saw It Coming, Doomskar, Behold the
  Multiverse), ✅ Disturb (CR 702.146 — `Keyword::Disturb` +
  `GameAction::CastDisturb`, cast transformed from the graveyard; 702.146e
  exile rider at the graveyard funnels),
  ⏳ Daybound/Nightbound, ✅ Decayed (`Keyword::Decayed` — CR 702.147: can't
  block + sacrifice at end of combat via the attacking-token cleanup queue),
  ✅ Blitz (`shortcut::blitz` /
  `AlternativeCost.blitz` — CR 702.152: alt-cost haste + "when this dies,
  draw a card" + sacrifice at next end step via `Effect::SacrificeSource`;
  Tenacious Underdog, Ardent Elementalist, Goldhound), ✅ Casualty
  (CR 702.153 — `Keyword::Casualty(n)` + `GameAction::CastSpellCasualty`:
  optional sacrifice-a-creature-of-power-≥-n additional cost that copies the
  spell on cast via `copy_stack_spell`; Cut of the Profits),
  ✅ Connive (`shortcut::connive` — CR 702.158, draw/discard +
  +1/+1-per-nonland via `Selector::DiscardedThisResolution`; Quandrix
  Cryptomancer), ✅ Backup (CR 702.164 — `shortcut::backup` /
  `backup_with`; ETB +N/+N + granted keywords *and* triggered abilities to a
  backed-up other creature; Conclave Sledge-Captain, Bola Slinger),
  ✅ Bargain (CR 702.176 — `Keyword::Bargain` + `GameAction::CastSpellBargain`:
  optional sacrifice-an-artifact/enchantment/token additional cost;
  `Predicate::SpellWasBargained` gates the payoff; Torch the Tower, Candy
  Grapple, Archon's Glory), ⏳ Craft, ⏳ Disguise/Cloak, ✅ Plot (CR 702.170 —
  `CardDefinition.plot_cost` + `GameState.{plotted_cards,plotted_this_turn}`
  + `GameAction::Plot` / `CastPlotted`: exile face-up for the plot cost, cast
  free on a later turn; Spinewoods Paladin, Vault Plunderer),
  ✅ Saddle (CR 702.171 — `Keyword::Saddle(n)` + `CardInstance.saddled` +
  `GameAction::Saddle` + `shortcut::attacks_while_saddled`; Stingerback
  Terror), ⏳ Gift,
  ✅ Offspring (CR 702.166 — `Keyword::Offspring(cost)` reuses the Kicker
  pipeline; `SpellWasKicked` gates an ETB 1/1 token-copy; Thundertrap Trainer),
  ✅ Impending (CR 702.183 — `Keyword::Impending(n)` + `AlternativeCost.impending`:
  enters with N time counters, not a creature until they tick off at end step;
  Duskmourn Overlord cycle), ✅ Ninjutsu (`Keyword::Ninjutsu(cost)` +
  `GameAction::Ninjutsu` — declare-blockers special action that returns an
  unblocked attacker and swaps the ninja in tapped + attacking; Fallen
  Shinobi), ✅ Embalm (CR 702.88) / Eternalize (CR 702.91) —
  `shortcut::embalm` / `eternalize` ride the from-graveyard exile-self
  activation + `CreateTokenCopyOf` (Zombie type, 4/4 for Eternalize); `sets::akh`.
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
- **Cast-from-elsewhere:** ✅ cast/play-from-library-top statics (CR
  401.5/401.6 — `StaticEffect::PlayFromLibraryTop` + `TopOfLibraryRevealed`,
  `LibraryView.known_top`; Courser of Kruphix, Oracle of Mul Daya, Mystic
  Forge; impulse-style exile-top already shipped), ✅ Suspend (`Keyword::Suspend(n, cost)` +
  `GameAction::Suspend` + `process_suspend` — CR 702.62: pay the suspend cost
  to exile from hand with N time counters, tick one off per owner's upkeep,
  free-cast when the last is removed; Rift Bolt, Ancestral Vision, Lotus
  Bloom. Creature-suspend haste + a UI prompt for the free cast's targets are
  TODO.md follow-ups), ✅ Forecast (CR 702.56 — rides the `from_hand`
  activated-ability path gated to the controller's upkeep + once-per-turn;
  Steeling Stance), ✅ Hideaway (CR 702.76 — `Effect::Hideaway { count }` +
  `Selector::CardExiledWithSource`; Shelldock Isle), ⏳ Aftermath.
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
  Emperor's Vanguard, Path of Discovery), ✅ Squad (CR 702.157 —
  `Keyword::Squad(cost)` + `GameAction::CastSpellSquad { times }` +
  `shortcut::squad_etb` + `Value::SquadCount`: pay the squad cost any number
  of times, mint that many token copies on ETB; Galadhrim Brigade, Vanguard
  Suppressor, Wasteland Raider, Zephyrim),
  ✅ Forage (CR 701.61 — `Effect::Forage { then }`: exile three graveyard
  cards or sacrifice a Food, then run the payoff; Treetop Sentries, Bushy
  Bodyguard), ✅ Endure (CR 701.63 — `Effect::Endure { target, n }`: counters
  or an N/N Spirit; Fortress Kin-Guard, Dusyut Earthcarver),
  ✅ Exploit (`shortcut::exploit(payoff)` — CR 702.105, ETB `MayDo`
  sacrifice-a-creature → run payoff; declining skips it),
  ✅ Extort (`shortcut::extort` — CR 702.99, SpellCast
  `MayPay({W/B}, drain 1)`; Basilica Screecher, Syndic of Tithes, Tithe
  Drinker, Kingpin's Pet), ⏳ Cohort, ✅ Support (`shortcut::support(n)` /
  `Effect::SupportCounters` — CR 701.32, a +1/+1 counter on each of up to N
  target creatures via the slot-based "up to N targets" machinery),
  ✅ Suspect (`Effect::Suspect` + `CardInstance.suspected` → computed Menace +
  CantBlock, CR 701.60; Barbed Servitor, Repeat Offender, Reasonable Doubt,
  Person of Interest), ✅ Discover (`Effect::Discover { n }` — CR 701.57,
  cascade-style exile-until-MV≤N then cast-free-or-to-hand; Geological
  Appraiser, Trumpeting Carnosaur), ✅ Collect Evidence
  (`Effect::CollectEvidence { amount, then }` — CR 701.59; `wants_ui` controller
  picks the exiled cards via a sum-validated `ChooseCards`, bots auto-pick the
  cheapest set; Sample Collector, Izoni),
  ✅ Expend (`EventKind::Expend` + `Predicate::ExpendReached(n)` over the
  per-turn `mana_spent_on_spells_this_turn`, CR 700.14; Roughshod Duo),
  ✅ Valiant (`BecameTarget + YourControl` + `once_per_turn`, CR 603.3d;
  Heartfire Hero).
- **Leaves-battlefield LKI:** ✅ CR 603.10 — `Value::PowerOf`/`ToughnessOf`
  read a dying object's last-known counter/pump-boosted P/T (`leaves_bf_lki`
  + `resolving_lki_source`); Goldvein Hydra, Cacophony Scamp, Heartfire Hero.
- **Spell-matters:** ✅ Escalate (`Effect::Escalate { modes,
  cost }` — CR 702.119; pick one or more modes, paying the escalate cost once
  per extra mode; Collective Brutality's discard-a-card), ⏳ Splice,
  ✅ Replicate (CR 702.107 — `Keyword::Replicate(cost)` +
  `GameAction::CastSpellReplicate { times }`: pay the replicate cost any number
  of times, copy the spell that many times via `copy_stack_spell`; Pyromatics,
  Train of Thought, Shattering Spree), ⏳ Overload (note: already shipped as
  an alt-cost `effect_override`; see Tier 1), ✅ Cipher (CR 702.46 — `Effect::Cipher` + `CardInstance.encoded_on`; Shadow Slice), ✅ Surge (`shortcut::surge`, CR 702.108 — OGW batch), ✅ Spectacle
  (`shortcut::spectacle` / `AlternativeCost.condition` —
  `Predicate::PlayerLostLifeThisTurn` + `Player.lost_life_this_turn`, CR
  702.111: cast for the spectacle cost if an opponent lost life this turn;
  Skewer the Critics, Light Up the Stage), ✅ Addendum
  (`shortcut::addendum` / `cast_during_your_main` — CR 702.124; resolution-time
  main-phase gate; Sphinx's Insight, Precognitive Perception),
  ⏳ Conspire, ✅ Demonstrate (CR 702.150 — `Effect::Demonstrate` +
  `shortcut::demonstrate`: a SpellCast/SelfSource trigger copies the spell for
  its caster and an opponent, each copy may choose new targets; the STX
  Technique cycle + Transforming Flourish).
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
  ✅ Affinity for artifacts (`CardDefinition.affinity_filter` generic reduction;
  Frogmite, Myr Enforcer, Thoughtcast, Somber Hoverguard, Qumulox, Sojourner's
  Companion, Carapace Forger), ✅ Entwine (CR 702.41 —
  `Keyword::Entwine(cost)` + `CastSpellEntwine`; entwined `ChooseMode` runs
  every mode; Tooth and Nail + the Mirrodin entwine batch),
  ✅ Buyback (`Keyword::Buyback(cost)` + `GameAction::CastSpellBuyback` —
  CR 702.27, optional additional cost; bought-back spell returns to its
  owner's hand instead of the graveyard on resolution; surfaced in
  `PlayerView.buyback_hand`; Corpse Dance), ✅ Miracle
  (CR 702.94 — `CardDefinition.miracle = Some(cost)`; the turn's first draw
  stamps the miracle alt-cost + may-play window via `maybe_grant_miracle`,
  cast for that cost through `CastFromZoneWithoutPaying`; Bonfire of the
  Damned, Temporal Mastery, Reforge the Soul),
  ✅ Bloodrush (CR 702.78 — a from-hand activated ability with
  `from_hand` + `discard_self_cost`; `{R}{G}, Discard this: target attacking
  creature gets +4/+4 + trample` — Ghor-Clan Rampager),
  ✅ Unleash (`Keyword::Unleash` + `shortcut::unleash()` — CR
  702.98: ETB "may enter with a +1/+1 counter" + computed `CantBlock` while it
  has one; Rakdos Cackler, Gore-House Chainwalker, Spawn of Rix Maadi),
  ✅ Scavenge (`shortcut::scavenge` — CR 702.97, gy-activated exile-self for
  +1/+1 counters = the exiled card's power; Dreg Mangler),
  ✅ Transmute (`shortcut::transmute` — CR 702.53, from-hand discard-self MV
  tutor; Drift of Phantasms),
  ✅ Bestow (`CardDefinition.bestow` + `GameAction::CastBestow` +
  `CardInstance.bestowed` — CR 702.103; cast an enchantment-creature as an
  Aura for its bestow cost, granting its `equipped_bonus`; not a creature
  while bestowed (`compute_permanent` strips the type); reverts to a
  creature when its host leaves (SBA); surfaced in `PlayerView.
  bestowable_hand`; Baleful Eidolon), ✅ Tribute (CR 702.104 — `Effect::Tribute` + `shortcut::tribute`; opponent answers via the synchronous decider; Fanatic of Xenagos, Oracle of Bones).

## Tier 5 — Mana & cost system

- ✅ **Typed spend restrictions / provenance riders** — `SpellKind` is a
  spend-context struct (instant/sorcery, artifact, creature types) and
  `SpendRestriction` gained `ArtifactOnly` + `CreatureOfTypeUncounterable`;
  `pay_for_spell` reports which restricted buckets paid, so Cavern of Souls'
  uncounterable rider rides actual mana provenance and Power Depot's
  artifact-only mana works for artifact spells *and* abilities
  (`CardDefinition::ability_spend_kind`). Fellwar Stone / Locus scaling were
  already separately wired. Remaining ⏳: per-source restrictions beyond
  these (filter lands' "spend only on activated abilities" etc.).
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
- ✅ **Multiple combat phases / extra attack steps.** `Effect::Additional
  CombatPhase` + `GameState.additional_combat_phases` (CR 505.1b) loop the
  turn back to Begin Combat when the active player leaves End of Combat with
  a phase banked (Hellkite Charger). Post-main insertion ships too:
  `Effect::AdditionalCombatPhaseAfterMain` + `additional_post_main_combats`
  re-enter Begin Combat when the postcombat main ends, with the follow-up
  main from the normal EndCombat → PostMain flow (Relentless Assault, plus
  `SelectionRequirement::AttackedThisTurn` for its untap clause).
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

1. ✅ **Big card-zoom preview on hover** — `hover_card_preview`
   (`systems::ui`) shows an enlarged copy of the hovered card's face beside
   the cursor (flipping to whichever side has more room so it never covers
   the card), with no board-dimming. Alt-hold still drives the centered
   detailed peek + counter/P-T tooltip.
2. ⏳ **Stops / auto-yield configuration** — per-phase stops + "yield until
   something needs me" (priority plumbing already exists; today only Pass /
   End Turn / Next Turn).
3. 🟡 **Combat math / damage preview** — `combat_preview` (`server::view`)
   projects each player's life swing (damage + lifelink) and the dying
   creatures off the declared attackers/blocks, honoring first/double strike,
   deathtouch lethal-spread, trample overflow, indestructible, and
   protection. Now **layer-aware**: P/T and keywords read the computed
   battlefield, so anthems and granted/stripped evasion are reflected. The
   client surfaces the life rows + a "Dying: N theirs / N yours" summary
   (`update_combat_preview_panel`), plus **planeswalker-target rows**
   (`CombatPreview.damage_to_planeswalkers` → loyalty projections).
   Remaining: multi-blocker damage-order nuance.
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
- ✅ **Commander-damage HUD readout** (CR 903.10a) — `PlayerView.
  commander_damage_taken` (projected in `server::view`) drives a per-source
  `⚔ <commander> N/21` chip next to each player's life in the stat strip,
  graded amber→red as it nears the 21-from-a-single-commander loss. Only
  present in Commander games.
- ⏳ **Hand sorting / auto-tap preferences / "play tapped land" prompt**.
- ✅ **Squad / Replicate pay-N-times stepper** — right-click modal feeding
  `CastSpellSquad`/`CastSpellReplicate`; impending countdown badge; NameCard
  picker modal with engine-ranked suggestions.
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
   primitive still open. (Combat damage-order, multi-pick "choose N",
   damage redirection, and draw doubling are now wired; layer-1 continuous
   copies ship via `Effect::BecomeCopyOfFor`.)
2. **Card-zoom preview + stops/auto-yield + combat-math preview**
   (Tier-7 #1–3) — the trio that most closes the "feels like Arena" gap.
3. **Best-of-3 + sideboard + deck legality** (Tier 10) — makes draft/cube
   and constructed competitive.
4. **Static-ability framework + mana provenance** — broad correctness wins
   that unblock many cards at once. (APNAP + same-controller trigger
   ordering are fully wired, including the server suspend.)
5. **Smarter AI blocking** (Tier 13) — biggest single-player upgrade.
6. Then the **Tier-4 mechanic sweep** and **Tier-3 object-model** features,
   card batch by card batch, promoting entries in the per-card trackers.
7. **Replays, spectator, social, accessibility** as the product matures.
