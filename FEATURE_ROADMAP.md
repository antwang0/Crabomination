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
  animations, keyboard cursor.

---

## Tier 1 — High-leverage engine primitives

Each unblocks a large swath of cards and removes the most visible "that's
not how Magic works" moments.

1. 🟡 **Replacement-effect framework.** A `replacement.rs` framework exists
   but only models zone-change replacements (Commander "→ command zone
   instead", CR 903.9b); the rest is stubbed per-card. Still to generalize:
   ETB replacement (enters tapped / with counters / as a copy / under your
   control), damage prevention & redirection (Fog family, Maze of Ith),
   draw/skip replacement, counter-doubling (Doubling Season, Hardened
   Scales), and "if it would die, exile instead."
2. ✅ **Multi-pick / "choose N" decisions.** `Decision::ChooseModes` is
   wired (`game/effects/mod.rs`, `DecisionAnswer::Modes`). Remaining nicety:
   a dedicated "pick from revealed cards" decision (Dig Through Time-style
   reveal-and-sort still routes through flat Draw-N).
3. ✅ **Player-chosen combat damage assignment order.**
   `Decision::CombatDamageOrder { attacker, blockers }` prompts the attacker
   (`combat.rs`, CR 510.1c) instead of sorting by CardId. (Trample-over-
   lethal / deathtouch spread math rides on top — see Tier 6.)
4. ⏳ **Linked "until this leaves play" exile.** Tidehollow Sculler, Brain
   Maggot, Banisher Priest, Oblivion Ring, Fiend Hunter — exile-and-return.
5. ⏳ **Token-copy of a permanent (clone).** Phantasmal Image, Helm of the
   Host, Mockingbird, Saheeli/Esika copy clauses, Spark Double, populate.
6. 🟡 **Copy-a-spell-on-the-stack.** `Effect::CopySpell` /
   `CopySpellUnlessPaid` exist and ship Storm / sac-to-copy cards, but the
   copy keeps the original's targets. Remaining: **new-target choice** on
   the copy — Twinning, Fork.

## Tier 2 — Engine rules fidelity (beyond Tier 1)

- 🟡 **APNAP trigger ordering** — inter-player APNAP is wired and tested
  (`game/mod.rs` apnap_rank sort, CR 603.3b; test
  `apnap_orders_simultaneous_triggers_active_pushed_first`). Remaining: let
  a controller order their *own* simultaneous triggers via a
  `Decision::OrderTriggers` prompt (today same-controller order is fixed).
- ⏳ **"Choose targets as it resolves" / divided damage** across N targets
  (Fireball, Forked Bolt, Cryptic-style "tap up to N").
- ⏳ **Targeting refinements:** "up to N targets", "target each", "another
  target", same-target-twice rules, protection re-check on resolution.
- ⏳ **Continuous-effect breadth:** characteristic-defining abilities,
  type/color/text-changing effects (CR 613 layers 1–6 corner cases),
  "becomes a copy of" layer interaction, set-P/T vs +N/+N ordering.
- ⏳ **Static ability framework:** cost-reduction statics, "you may play"
  permissions from permanents, "creatures you control have X", anthem
  stacking, devotion-gated states (Heliod, Nyx gods).
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
- ⏳ **Adventure** (cast-the-spell-then-exile-to-cast-creature duality).
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

- **High frequency / modern staples:** ⏳ Madness, ⏳ Escape, ⏳ Adventure,
  ⏳ Soulbond, ⏳ Mutate, ⏳ Companion, ⏳ Foretell, ⏳ Disturb,
  ⏳ Daybound/Nightbound, ⏳ Blitz, 🟡 Casualty, ⏳ Connive, ⏳ Backup,
  ⏳ Bargain, ⏳ Craft, ⏳ Disguise/Cloak, ⏳ Plot, ⏳ Saddle, ⏳ Gift,
  ⏳ Offspring, ⏳ Impending, ⏳ Ninjutsu, ⏳ Embalm/Eternalize.
- **Counter / +1+1 matters:** ⏳ Proliferate (effect), ⏳ Bolster,
  ⏳ Adapt, ⏳ Evolve, ⏳ Mentor, ⏳ Training, ⏳ Modular, ⏳ Graft,
  ⏳ Outlast, ⏳ Monstrosity, ⏳ Devour, ⏳ Bloodthirst, ⏳ Amass.
- **Cast-from-elsewhere:** ⏳ cast-from-top (Mind's Desire / Amped Raptor /
  Robber of the Rich), ⏳ Suspend (+ time counters), ⏳ Forecast,
  ⏳ Hideaway, ⏳ Aftermath.
- **Combat-flavor:** ⏳ Bushido, ⏳ Flanking, ⏳ Rampage, ⏳ Provoke,
  ⏳ Battle Cry, ⏳ Melee, ⏳ Dash, ⏳ Boast, ⏳ Afflict, ⏳ Enlist,
  ⏳ Mobilize, ⏳ Myriad.
- **Value/ETB:** ⏳ Investigate (verb) + sac-Clue payoff (🟡 Clue tokens
  exist), ⏳ Fabricate, ⏳ Riot, ⏳ Afterlife, ⏳ Squad, ⏳ Forage,
  ⏳ Exploit, ⏳ Extort, ⏳ Cohort, ⏳ Support.
- **Spell-matters:** ⏳ Splice, ⏳ Replicate, ⏳ Overload, ⏳ Cipher,
  ⏳ Surge, ⏳ Spectacle, ⏳ Addendum, ⏳ Conspire, ⏳ Demonstrate.
- **Resource systems:** ⏳ Energy ({E}), ⏳ Experience counters,
  ⏳ Poison/Toxic + corrosion (poison counters exist; needs Toxic + the
  10-poison SBA path verified), ⏳ Ascend / city's blessing,
  ⏳ Initiative / monarch, ⏳ Day/Night, ⏳ Ring-bearer (the Ring tempts you).
- **Fading family:** ⏳ Fading, ⏳ Vanishing (Parallax cards in cube).
- **Older mechanics:** ⏳ Soulshift, ⏳ Offering, ⏳ Epic, ⏳ Absorb,
  ⏳ Affinity (have artifact count?), ⏳ Entwine, ⏳ Buyback, ⏳ Miracle,
  ⏳ Bloodrush, ⏳ Unleash, ⏳ Scavenge, ⏳ Bestow, ⏳ Tribute.

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
- ⏳ **Multiple combat phases / extra attack steps** (Aggravated Assault).
- ⏳ **"Must attack/block", "can't attack alone", "attacks each combat"**
  restrictions and requirements (Master of Cruelties currently drops these).
- ⏳ **Planeswalker / Battle as attack targets** UI + redirection.
- ⏳ **Ninjutsu attacking-creature swap**, **Goad**, **Lure**, **provoke**.

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
5. ⏳ **Targeting arrows on the stack** — source→target lines.
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
- ⏳ **Highlight legal plays** (castable cards, legal attackers/blockers,
  legal targets) — `legal_target_filter` exists; extend to a full hint
  layer.
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

- 🟡 **Smarter blocking** — the bot blocks "at random" (`server/bot.rs`);
  add survive-don't-chump, gang-block lethal, deathtouch/trample awareness.
  Highest-value single-player improvement.
- ⏳ **Better sequencing** (land drops, hold-up interaction, when to cast).
- ⏳ **Mulligan decisions** (keep/ship heuristic).
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
