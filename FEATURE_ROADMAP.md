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

- **Core loop:** LIFO stack, multiplayer priority, state-based actions, delayed
  triggers, intervening-`if` (603.4), the layer system (613), split
  first-strike / regular combat-damage steps, APNAP ordering.
- **Keywords (~120):** evasion + combat (Flying/Reach/Menace/First+Double
  Strike/Trample/Deathtouch/Lifelink/Vigilance/Protection/Hexproof/Shroud/Ward/
  Bushido/Flanking/Rampage/Provoke/Melee/Dash/Boast/Afflict/Enlist/Mobilize/
  Myriad/Ninjutsu/Goad/Lure…); ETB/value (Persist/Undying/Riot/Fabricate/
  Afterlife/Explore/Exploit/Extort/Investigate/Embalm/Eternalize/Backup/
  Soulbond/Mentor); counter-matters (Proliferate/Bolster/Adapt/Training/Evolve/
  Modular/Graft/Outlast/Renown/Bloodthirst/Monstrosity/Devour/Amass); cast-mode
  + alt-cost (Kicker/Casualty/Connive/Offspring/Plot/Saddle/Blitz/Spectacle/
  Escalate/Buyback/Bestow/Foretell/Suspend/Flashback/Madness/Escape/Adventure/
  Cascade/Storm/Convoke/Delve); plus Fading/Vanishing, Cumulative Upkeep, Echo,
  Dredge, Retrace, Morph/Megamorph, Crew/Reconfigure, Changeling, Soulshift,
  Unleash, Devoid, Ingest, Absorb.
- **Costs/mana:** colored/generic/colorless/hybrid/mono-hybrid/Phyrexian/snow/X;
  Convoke/Delve reduction; Commander tax; alternative (pitch) costs;
  energy-gated mana abilities; X-cost activated abilities.
- **Resource systems:** Energy {E}, Poison/Toxic, Devotion, Ascend/city's
  blessing, Monarch, Day/Night, coin-flip + die-roll randomization.
- **Objects:** tokens (Treasure/Clue/Blood/Food/Map/Army/Germ), counters
  (incl. keyword/shield/stun/finality/rad), planeswalkers + loyalty + emblems,
  MDFC, split // fuse // aftermath, adventure, command zone + Commander,
  manlands, living weapon, clones/token-copies/spell-copies.
- **Replacement effects:** enters-tapped, enters-with-counters,
  token/counter/damage/mana doubling, regeneration, EtbTriggerTax, Maze-of-Ith
  per-source prevention, prevention shields, finality exile-instead. Counters
  cease on zone change (122.2).
- **Statics (misc):** no-max-hand-size, play-lands-from-graveyard,
  artifact/creature non-mana-ability locks, spell-tax, two-player coin-flip-off
  (Mana Clash), reveal-top-land-else-hand.
- **Ability/trigger riders:** statics-granted triggered abilities (Kataki),
  conditional aura riders, rhystic taxes (Esper Sentinel), once-per-turn
  triggers (603.3d), opponents-only activations, discard-self cost,
  counter-to-exile, blink-return-EOT.
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

## Tier 1 — High-leverage engine primitives

Each unblocks a large swath of cards.

1. 🟡 **Replacement-effect framework.** `replacement.rs` models zone-change
   replacements (Commander → command zone); the rest is per-card. Shipped:
   enters-tapped (`StaticEffect::EntersTapped`, incl. self-source), exile-instead
   for non-cast creatures (Containment Priest), opponent-creature-dies → exile
   (Valentin), graveyard → exile hate (`ExileCardsBoundForGraveyard` via
   `route_to_graveyard` — Rest in Peace, Leyline of the Void), counter-lock
   (Solemnity), counter/damage doubling (Doubling Season, Furnace of Rath),
   damage prevention as shields (`prevention_shields`), per-source combat shields
   (Maze of Ith), damage redirection (Palisade Giant), draw doubling (Thought
   Reflection), damage halving (Ghosts of the Innocent), creature-ETB control
   steal (Gather Specimens), skip-step and skip-turn. Still to generalize: as-a-copy
   ETB, draw replacement breadth.
2. ✅ **Multi-pick / "choose N" decisions.** `Decision::ChooseModes`;
   pick-from-revealed via `Effect::LookPickToHand` (Impulse, Strategic Planning).
3. ✅ **Player-chosen combat damage assignment order.**
   `Decision::CombatDamageOrder` prompts the attacker (510.1c).
4. ✅ **Linked "until this leaves play" exile** (603.6e).
   `Effect::ExileUntilSourceLeaves` + `return_linked_exiles` (Banisher Priest,
   Fiend Hunter, Oblivion Ring, Brain Maggot, Tidehollow Sculler).
5. 🟡 **Copy of a permanent (clone).** `Effect::BecomeCopyOf` +
   `enters_as_copy` ship Clone, Phantasmal Image, Mirror Image, Stunt Double;
   token copies via `CreateTokenCopyOf`. Remaining: continuous layer-1
   "becomes a copy" effects (Helm of the Host loop, Mirrorform aura).
6. ✅ **Copy-a-spell-on-the-stack.** `Effect::CopySpell` /
   `CopySpellMayChooseTargets` (new-target choice) — Storm cards, Reverberate, Fork.

## Tier 2 — Engine rules fidelity (beyond Tier 1)

- ✅ **APNAP trigger ordering** — inter-player (`apnap_rank`) plus
  same-controller ordering with a real server suspend (`ResumeContext::
  TriggerOrder`), so networked seats are prompted.
- 🟡 **Divided damage / counters** — `Effect::DealDamageDivided` +
  `Effect::DistributeCounters` (Jugan) share `Decision::DivideDamage` (the modal
  is noun-aware). Forked Bolt, Pyrokinesis, Crackle with Power. Remaining:
  "choose targets as it resolves".
- 🟡 **Targeting refinements:** resolution-time legality re-check (608.2b) ships
  for single/multi-target spells and Auras, and now resolves `{X}`-from-cost
  target filters (Hearth Kami's "artifact with mana value X" via
  `ManaValueExactlyXFromCost`). "Up to N targets" ships via
  `Effect::ApplyToTargets` (Sea God's Scorn bounce-3, Wrap in Flames
  1-to-each-of-3). Remaining: "target each", protection-from-color re-check.
- 🟡 **Continuous-effect breadth:** layer-3 text-changing ✅ (Trait Doctoring);
  land-type statics ✅ (Blood Moon, Urborg). Remaining: CDA corners, full
  text-box swaps, "becomes a copy of" layer interaction.
- 🟡 **Static ability framework:** cost-reduction statics, "you may play"
  permissions, anthem stacking incl. disjunctive multi-type lords (Blex);
  devotion-gated god states (`NotCreatureWhileDevotionBelow`); keyword loss
  (`LoseKeyword` — Nowhere to Run). Remaining: broader "you may play",
  devotion-gated non-type states.
- 🟡 **Replacement of life/draw/damage events** (ties to Tier-1 #1).
- ✅ **Regeneration shields & "next time" prevention** as proper shields.
- ⏳ **Damage marking vs. wither/−1−1, lethal/indestructible** audited against
  CR 120/704. (Wither/Infect damage-as-counters already ships; this is the
  marking-interplay audit.)
- ⏳ **Loyalty fidelity:** loyalty-set effects, "any time" riders, proliferate on
  loyalty, attacking-planeswalker redirect.
- 🟡 **State-based action coverage:** ±1/±1 annihilation ✅, counter caps ✅,
  legend rule ✅, saga sacrifice ✅, world rule ✅. Remaining: attached-Aura
  orphan corners.

## Tier 3 — Object model & zones

- ⏳ **Battle card type** (110.4) + defense counters + `AttackTarget::Battle`.
- 🟡 **Sagas** (714). `saga_chapters` + `saga_advance` (History of Benalia, The
  Eldest Reborn). Remaining: DFC sagas, read-ahead/chapter-choice.
- ✅ **Split cards** (709) + **Fuse** — `CardDefinition.split`,
  `CastSplitRight`/`CastSplitFused` (Wear // Tear).
- ✅ **Adventure** (715) — `CardDefinition.adventure` + `CastAdventure` (Bonecrusher
  Giant, Brazen Borrower, Murderous Rider, …).
- 🟡 **Classes / Cases / Backgrounds.** **Rooms ship** (709.5 — `room` +
  `CastRoomDoor`/`UnlockRoomDoor`; Unholy Annex // Ritual Chamber).
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
  two-target aura-move, Nezumi Graverobber, Student of Elements). **Prototype**,
  **Omen** (other in-place-modify mechanics) still ⏳.
- 🟡 **Face-down permanents** (708) — `face_up_def` stashes the real card; Manifest
  / ManifestDread + `TurnFaceUp`; Morph/Megamorph cast-face-down ✅. Remaining:
  Disguise/Cloak edge cases (both core paths ship — see Tier 4).
- ⏳ **Ante / conspiracy / dungeon / sticker / attraction** zones (novelty only).
- ✅ **Emblems** as command-zone objects — `Player.emblems` + `CreateEmblem`.
- ⏳ **Sideboard zone** + "from outside the game" (wishes, companions).

## Tier 4 — Keyword & ability mechanics (the long tail)

Each a small targeted feature; sweep batch by batch.

- **High frequency / modern staples:** ✅ Madness, ✅ Escape, ✅ Adventure,
  ✅ Soulbond, ⏳ Mutate, 🟡 Companion ({3} sideboard→hand; deck validation ⏳),
  ✅ Foretell, ✅ Disturb, ✅ Daybound/Nightbound (keywords + day/night +
  502.2 transition + DFC auto-flip), ✅ Decayed, ✅ Blitz, ✅ Casualty, ✅ Connive,
  ✅ Backup, ✅ Bargain, ⏳ Craft, ✅ Disguise/Cloak, ✅ Plot, ✅ Saddle,
  ⏳ Gift, ✅ Offspring, ✅ Impending, ✅ Ninjutsu, ✅ Embalm / Eternalize,
  ✅ Exhaust (activate-only-once activated abilities — Camera Launcher).
- **Counter / +1+1 matters:** ✅ Proliferate, Bolster, Adapt, Training, Evolve,
  Mentor, Modular, Graft, Outlast, Renown, Bloodthirst, Monstrosity, Devour,
  Amass — all via `shortcut::*` builders.
- **Cast-from-elsewhere:** ✅ play-from-library-top statics (Courser, Oracle of
  Mul Daya, Mystic Forge), ✅ Suspend (creature-suspend haste + free-cast target
  UI are follow-ups), ✅ Forecast, ✅ Hideaway, ⏳ Aftermath.
- **Combat-flavor:** ✅ Bushido, Flanking, Rampage, Provoke, Battle Cry, Exalted,
  Frenzy, Melee, Dash, Boast, Afflict, Enlist, Mobilize, Myriad, Amass.
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
  ✅ Day/Night (502.2 turn-based transition + Daybound/Nightbound DFC auto-flip);
  ⏳ Ring-bearer.
- **Fading family:** ✅ Fading, Vanishing (`process_fading_vanishing`). Remaining:
  Parallax Dementia's steal-on-leave rider.
- **Older mechanics:** ✅ Soulshift, Epic, Umbra armor, Affinity, Entwine, Buyback,
  Miracle, Bloodrush, Unleash, Scavenge, Transmute, Bestow, Tribute, Offering
  (CR 702.48 — `AlternativeCost.offering` + `ManaCost::reduce_by_cost`; the
  Kamigawa Patron cycle). Spiritcraft "cast a Spirit or Arcane spell" triggers
  ride `SelectionRequirement::HasSpellSubtype` + `shortcut::spiritcraft`.

## Tier 5 — Mana & cost system

- ✅ **Typed spend restrictions / provenance riders** — `SpellKind` +
  `SpendRestriction` (Cavern of Souls, Power Depot). Remaining ⏳: per-source
  restrictions beyond these (filter lands).
- ⏳ **Minimum-cost floor** (Trinisphere) and **cost-increase statics** beyond the
  first-spell tax. (Note: Trinisphere floor actually ships — see CUBE_FEATURES.)
- ⏳ **Conditional / additional costs** as a general modal layer.
- ⏳ **{X} in activated abilities** generalized; **delve/convoke colored**
  contribution.
- ⏳ **Snow-mana-only** and **mana-value-X** cost gates.

## Tier 6 — Combat fidelity

- ⏳ **Damage assignment order** (Tier-1 #3) and **trample math** with
  multiple/deathtouch blockers.
- 🟡 **Banding** (CR 509.2 / 510.1c) — a banding blocker routes the attacker's
  combat-damage order + assignment to the *defending* player (Benalish Hero).
  Remaining: attacking-band formation + "bands with other".
- ✅ **Multiple combat phases** — `AdditionalCombatPhase` (Hellkite Charger) +
  post-main insertion (Relentless Assault).
- 🟡 **"Must/can't attack/block" restrictions** — `Keyword::{CantAttack,CantBlock,
  AttacksAlone,MustBeBlocked,AllMustBlock,MustAttack,MustBlock}`, Goad. Open:
  granted must-attack with future-turn duration, multiplayer goad-target clause,
  cost-to-block (509.1d-f).
- ⏳ **Planeswalker / Battle as attack targets** UI + redirection.
- ✅ **Goad**, **Lure**, **Provoke**, **Ninjutsu swap**.

## Tier 7 — UI / UX core (the Arena "feel" gap)

1. ✅ **Card-zoom hover preview** — `hover_card_preview` (flips side to avoid
   covering the card); Alt-hold drives the centered detailed peek.
2. ✅ **Stops / auto-yield config** — `auto_advance_p0` smart default + per-step
   Stop/Skip overrides on the phase chart (`StopConfig`), separate for your turns
   vs. opponents'.
3. 🟡 **Combat math / damage preview** — `combat_preview` projects life swing +
   dying creatures (first/double strike, deathtouch spread, trample, protection),
   layer-aware, with planeswalker-target rows. Remaining: multi-blocker
   damage-order nuance.
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
   choices, seat-routed yes/no asks (rhystic, Tribute, Browbeat, MayPay).
   Remaining ⏳: CommanderRedirect, ChooseLegendToKeep (raised inside SBA/damage),
   modal triggers with targeting modes, non-Bool opponent-owned picks.

## Tier 8 — UI / UX quality-of-life

- ✅ Browsable **graveyard / exile** zones (`V` toggles exile, with source
  annotations); library shows a count chip only.
- ✅ **Search / Scry / Surveil / Mulligan** picker UIs (top/bottom toggles, reorder
  buttons). Drag-and-drop reorder ⏳.
- ✅ **London mulligan** bottoming; Serum Powder gets its own button.
- 🟡 **Floating life deltas** ✅; per-turn life-history graph ⏳.
- ✅ **Commander-damage HUD** (903.10a) — per-source `⚔ <cmdr> N/21` chip,
  amber→red near loss.
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
  restore. Remaining ⏳: surface a "reconnecting (N/10)…" banner.
- ✅ **Spectator mode** (read-only `ClientView` stream).
- ✅ **Player identity** — editable display name reaches every seat + log lines,
  persisted across launches.
- 🟡 **Chat** — free in-match chat ships (`T`). Remaining ⏳: emotes, mute,
  lobby-phase chat.
- 🟡 **Timers** — per-action rope ships server-side + client countdown banner.
  Remaining: per-game chess clock.
- ⏳ **Friends / invites / ratings / leaderboards**.
- ⏳ **Free-for-all politics** UI for 3+ player tables.

## Tier 10 — Formats & match structure

- ⏳ **Best-of-3 + sideboarding** flow.
- 🟡 **Deck legality validation** — size/copy/singleton/Commander-identity ✅,
  ban + restricted lists ✅ (`format::validate_deck`). Remaining: per-set legality
  pools (Standard rotation), Pauper rarity.
- ⏳ **More 60-card formats** (Modern/Pioneer/Legacy/Vintage/Pauper — mostly
  banlist/pool config).
- ⏳ **Limited match rules** (40-card, basic-land access).
- ⏳ **Multiplayer variants** (Planechase, Archenemy, Oathbreaker, Star, Emperor).
- ⏳ **Casual toggles** (free mulligans, vanguard).

## Tier 11 — Limited (draft / sealed)

- ✅/🟡 **Draft + cube** exist. Extend with:
- ⏳ **Sealed**, ⏳ **bot drafters** (signal/pick heuristics), ⏳ **draft variants**
  (Winston/Rochester/Grid/…), ⏳ **set-based draft**, ⏳ **draft replay / pick
  history / pool export**.

## Tier 12 — Deckbuilding & collection

- ⏳ **In-app deck builder** (search, curve view, legality, sample-hand).
- 🟡 **Import / export** — import ships (`decklist::parse_decklist`, Arena/MTGO
  text; menu "Play Deck vs Bot" loads and validates). Remaining ⏳: export,
  .dec/.cod, paste-from-clipboard, choosing opponent's deck.
- ⏳ **Deck stats** (curve, pips, type breakdown).
- ⏳ **Collection tracking**; ⏳ **Scryfall-like card search** over the catalog.

## Tier 13 — AI

- 🟡 **Smarter combat** — `server/bot.rs` blocking is heuristic (value trades,
  first-strike/deathtouch/trample awareness, gang-block-to-survive); attacking has
  a suicide filter + evasion awareness + planeswalker redirection. Remaining: race
  math, multi-blocker math, attacking-into-open-mana respect.
- ⏳ **Better sequencing** (land drops, hold-up, when to cast).
- 🟡 **Mulligan decisions** — `RandomBot` ships flood/screw mulligans with
  color-screw awareness. Remaining: transitive fetch/dual sources.
- ⏳ **Targeting / mode / X-value choices** by evaluation.
- ⏳ **Difficulty levels**; optional **search-based AI** (MCTS over snapshots).

## Tier 14 — Replays, analysis & observability

- ⏳ **Action-log replay viewer** (snapshots + `GameEvent` stream are the
  foundation).
- ⏳ **Game history / match results** persistence.
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
