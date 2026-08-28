# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.


Split for size — this file is the working handoff, the archives below are
reference. All three carry an index table at the top; triaged at the
sixty-seventh pass, so don't re-take that.

- `ENGINE_BACKLOG.md` — engine backlog and audits, ordered **bugs /
  mechanics & primitives / rules coverage / tooling**: the correctness audit,
  the robustness filters, the decision-plumbing audit, missing mechanics,
  follow-ups not yet done, suggested next-up tasks, the CR coverage audit.
- `CARD_BACKLOG.md` — per-set card residuals: what each set still
  approximates, and the remaining gap lists. Titled by **subject**, ordered
  open-first, then the closed sets whose only content is residuals.
- `CLIENT_BACKLOG.md` — the Bevy GUI backlog (visualization / UX / refactor).
  The client **does** build here after four apt packages; that file's header
  has the command and the disk caveat.
- `PERF.md` — the perf record: how to measure, **the standing rules**,
  baseline, log, profile of record, candidates.

## NEXT (handoff — an INDEX. Every number lives in PERF; nothing here restates one.)

**FIRST:** `git fetch origin claude/modern_decks && git checkout -B
claude/modern_decks origin/claude/modern_decks` — the container clones `main`,
and sessions run concurrently: push code before tracker prose, rebase not force.

1. **Perf queue** — PERF "Perf candidates". `(-77)`'s memo device is
   **CLOSED**: four rows shipped, three refuted, and one refutation
   *reversed* — `dispatch_board_scan` came back off the refuted list at the
   eighty-eighth pass at `cube` -0.317 %, and its last row
   (`granted_abilities_of`, six exact bits) was built and reverted in the
   same pass at `fixed` +0.024 %. The rule that separates the two is in that
   Baseline block: **count the work items the bit elides, not their kind** —
   four per card wins, one length check does not. `(-51)(a)` is **re-sized in
   the actor block and its named blocker shipped**, so take it next.
   `(-78)`'s `flat_map` half is **swept** — six commits, `FlatMap::next` off
   the profile as a named row; what is left of it is `Chain` at a third the
   traffic and none of it a whole-board walk, so expect a tenth. **New and
   half-swept: the write-once `Box` field.** `resolving_spell_snapshot` is
   `Arc` now (`fixed` -0.287 % / `cube` -0.203 % / `sealed` -0.222 %, 21,896
   allocations); `GameState`'s other `Box` is the decider and it is checked
   and already free (ZST). `PlayerData` / `CardData` / `ColdState` are the
   unswept half — the test is "does anything write it after it is set", and
   the entry is in PERF's newest Baseline block. Then (-70) (quiet window
   only), (-69)'s two unclaimed rows, (-61), (-59). **(-60) is closed** — the
   visitor rewrite plus the eighty-eighth's gate.
2. **Perf method** — PERF's "How to measure", **"Standing rules for a perf
   pass"** (moved there from this file this pass) and "Which pool a change
   moves". Read all three pools; a pool split is a revert.
3. **Instruments before profiles** — `CRAB_SIM_REJECTS`, `CRAB_PAY_FAILS`,
   `server::bot_rejection_count`, `--bench`'s stall split. **All re-read at
   this tip** (PERF, above the eighty-seventh Baseline block): rejects **0 in
   25 cells** over 246,000+ simulated declarations, and `CRAB_PAY_FAILS` on
   `--decks fixed` has gone from ~700 to **zero** while `sos` is
   byte-identical, which is what makes it a reading rather than drift.
4. **Encoding caution** — pool / `Vocab` / `TrainRow` / observation and deck
   encodings invalidate the trained nets. **Two commits this pass are inside
   `encode.rs` and neither moves a byte of output**: the library grouping
   dropped its `BTreeMap` for a sorted inline buffer (unique keys, same total
   order), and `Vocab::index_of` became a memo slot. Identical rows over
   identical play, both audited by
   `the_library_encodes_as_a_deduplicated_multiset` and by the actor's
   `debug_assert!`. **No net needs retraining as of `dc478735`.**
5. **Robustness gate** — RUN and clean at the eighty-seventh pass for all
   seven memo families: `-C debug-assertions=yes` on `[profile.overflow]`,
   **33,120 games over 5 pools x 6 seeds**, plus **360 actor games** for the
   vocab-index memo the ladder cannot reach. **`scripts/robustness_grid.sh`**
   is the committed recipe — build, the `strings … | grep "memo is stale"`
   proof that `RUSTFLAGS` reached the right crate, the grid, non-zero exit on
   any failure; `POOLS` / `SEEDS` / `GAMES` override and `--no-build` reuses
   `target-audit/`.
6. **Bugs** — ENGINE_BACKLOG's live-match section, and the one open entry is
   now **scoped and ready to take**: a layer-7 `PumpSelfIf` condition cannot
   see a layer-4 type change (CR 613.8). It is a *class* — **~60 of the 194
   catalog `PumpSelfIf` conditions read a characteristic a layer can
   change**, the other ~134 read player/zone facts and are correct as they
   stand. Two designs are written out there with which cards each covers; the
   `CardMatchPowerGated` sibling covers only the per-card ones, so the
   two-phase gather is the one that pays. It moves golden traces
   (legitimately) and lives in the hottest function, so it needs a `--decks
   cube` reading and the debug-assertions ladder gate, not just the suite.
   Both card audits are **re-run and clean at this tip**: `audit_incomplete
   --structural-only` 21,795 scanned / 0 need review, `audit_stubs` 21,795 /
   0 flagged, `audit_variant_coverage` 0 dead capabilities over 1,695
   variants. The seeded 4,000-pairing sweep is clean at the eighty-seventh
   tip.
7. **ML** — deck judge 60.3 % pooled (ML_NOTES). Open, not unilateral: should
   `selfplay` seed `jitter_below` from `--seed`?
8. **Tip state** — PERF "Baseline"'s **CLOSING STATE** block: the
   eighty-seventh pass is **-4.906 % `fixed` / -4.986 % `cube` / -4.571 %
   `sealed`** end to end across both halves, with the rustc version and the
   box fingerprint beside the numbers. The actor is **-8.02 %** since pass 83
   with play byte-identical ("Profile of record"). `--bench` byte-identical
   throughout; suite 19,063 / 0 / 5; clippy clean.
   **The clock is a ladder-pool instrument, not an actor one**: the pass
   reads -1.59 % paired on `ab_wall.py --decks fixed` with a flat null, and
   `selfplay_train --actors 3`'s own null spreads ±26 %.
9. **Filters** — all five syntax filters read zero; `audit_variant_coverage`
   needed a **bit bridge** this pass (a definition bitmask crossing the crate
   boundary made a live capability look dead) and reads zero again. **A sixth:
   `scripts/audit_decision_plumbing.py`** — 195 `decider.decide` sites, 97
   plumbed / 98 bare, and it retired ENGINE_BACKLOG's "~45 live bugs" claim
   (every effect that audit named is plumbed now). Bare is a triage
   population, not a gate; the calibration is in that section.
   `bot_rejection_count()` over the seeded cube sweep is the one that watches
   the live-server path, and it is clean at the tip.

## Standing rules for a perf pass — moved to `PERF.md`

The section lived here for thirty passes and grew to **555 lines**, which is
what put this file over its own 1k limit. It is the *perf record*, not the
handoff, so it moved **verbatim** to `PERF.md`'s "Standing rules for a perf
pass" (directly under "How to measure") rather than being compacted away —
every rule is a refutation with numbers and this file's own instruction was
not to delete one. Read it there before proposing anything on the perf queue.

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

## Engine — Robustness / defects

**No open entries.** The determinism class, the panic/unwrap census and the
twenty-three robustness filters are all closed; each filter is a *shape* that
fails the way a training run notices (a silent wrap at game 400 k, a loud
panic, a hang), and nine of them found nothing, which is the result worth
keeping. **The whole record — what each filter hunted, what it found, and the
rule it yielded — moved verbatim to `ENGINE_BACKLOG.md` at the fifty-fourth
pass.** Read it before proposing a robustness sweep; re-deriving a closed
filter is the failure mode it exists to prevent.

Standing goals that outlive it: no panic/unwrap reachable from bot self-play;
cross-process determinism at a fixed seed (golden traces assert it, and
`CRAB_THREAD_CHECK=1 --bench` asserts it across thread counts); stall rate
tracked on `--bench` (`stalls_by cap / stuck / draw` — re-open only if `cap`
or `stuck` goes non-zero).


## ML — defects (index)

### Every committed deck net fails to load — FIXED for the future, not for those nets

**The code half is the fifty-fourth pass's freeze; the artifact half
closed 2026-08-24 (next paragraph).** `VOCAB_SNAPSHOT` froze
the embedding index at the fifty-fourth pass, `pad_vocab` zero-extends a
shorter table and `--use-deck-best` runs end to end at 91.7 % of the unjudged
rate. The seven committed `*/deck-latest.safetensors` predate the freeze, so
nothing can say which card each of their rows meant; `vocab_fit` refuses them
by name and they need retraining. `nets/champion.safetensors` is unaffected.
**The full write-up — what the bug was, what the fix does, what is
deliberately not fixed and why, and the out-of-range clamp found alongside it
— moved verbatim to `ML_NOTES.md` at the fifty-seventh pass.**

**Resolved 2026-08-24 for the artifact, not the class.** No retrain was
needed: the deck stream rides along in every `selfplay_train` run, so the
recent run directories already hold vocab-164 deck nets. Two were re-gated
at the round-11 shape (`.ladder/run_deck_regate.sh`, 800 games × 12 pools
per cell, seeds 43/97): `nets_r45_ctrl_s43` 59.6/61.1, `nets_r41_v7_s43`
59.7/61.5 — pooled 60.3/60.6, a statistical tie on the historical 60–62
band. The r45 artifact (newest champion-class run) is committed as
`nets/deck-champion.safetensors`; `--use-deck-best`, `--gate-builder-hc`
and `--distill-gen` are live again. No retraining "for those nets" was
ever needed — the run directories held current-vocab spares all along.
The structural class is closed by the freeze above; this artifact is at
the frozen size (164), so `vocab_fit` accepts it and future card
additions only pad it.

## Engine — Missing Mechanics

Moved verbatim to `ENGINE_BACKLOG.md` at the sixty-second pass, when this
file passed the ~1k-line trigger again — it is a backlog of unimplemented
mechanics, which is what that file is for. Nothing is closed by the move.

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

### Phases 0 and 1 — ✅ shipped
Seeded serialized `GameState.rng` behind every "at random" (so undo cannot
re-roll a shuffle and replay is bit-exact); persisted-history serde fidelity
is still gated on the `CardInstanceWire` dropped-fields fix and the round-trip
property test (Infrastructure → Snapshot Round-Trip Test), which in-memory
undo does not need. Then a checkpoint at the top of `perform_action`, restored
on `Err`, for **every** action, pinned by
`cow::tests::rejected_action_restores_state_exactly`. Three semantics worth
remembering, because each was a bug first:
- Suspension is not failure. `GameError::ManualTapRequired` is exempted —
  it deliberately leaves forced pips tapped and mana floating for the
  client's pending-cast driver (pinned by the `sos::mana_shapes` tests).
- The restore keeps the **live** decider; the checkpoint clone holds a
  blank one, and swapping that in wipes a `ScriptedDecider` mid-script.
- A failed *resume* restores to the suspended state, not to before the
  original action. Multi-step atomicity across a suspend/resume chain is
  future work if it is ever needed.

**Perf:** the checkpoint is the engine's largest single structural cost —
`-5.47 %` of the whole program on the bench workload, and it is *never*
read there (see PERF's forty-third pass and candidate (-13)). Any narrowing
of it is a rules-correctness argument first and a perf change second.


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

## Bot / AI

### Instant-Speed Responses
~~The bot never responds to spells on the stack.~~ `pick_stack_response`
now counters an opponent's spell when it targets the bot's permanents /
the bot, or costs 3+ — cheapest affordable counter first, `would_accept`
dry-run as the final gate (so Spell Snare's MV filter etc. are honored).
Future: respond with removal/protection instants, not just counters;
race-aware "is this worth a card" valuation. Round 43: the buff 2-for-1
(`buff_2for1`, kill the creature under the opponent's own pump) is
built and zero-incidence in bot mirrors — human-facing, default off.

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
walker fills up. Round 43: the chip candidate exists (`walker_chip`, one declaration at
the lowest-loyalty unfinishable walker, sims judge it) — zero-incidence
in the walker-free sealed gate pools, so it stays default off on the
strength of the recorded ten-turn-ultimate loss, not a ladder number.
Still open: the inverse case (a low-loyalty walker not worth committing
trample beaters to).

### Smarter Mana Rock Usage
The bot taps mana rocks eagerly before knowing what it wants to cast.  A
"plan this turn's spending first" pass before mana-ability activation would
avoid situations where it taps a Sol Ring with nothing to cast.

### Encoder / net follow-ups — moved to `ML_NOTES.md`
The belief-head recall diagnostic, the encoder-v7 gap list, the
static-anthem P/T hole, the feature-occupancy precondition and the
prioritized next-round candidate list live in **`ML_NOTES.md` → "Encoder and
net follow-ups (moved from TODO, 2026-08-23)"**, verbatim. They are long
experiment narratives and they exist so nobody re-derives the dead ends.
One of them is load-bearing for *this* branch: **static-anthem P/T is
invisible to the encoder**, and a modern-decks pool is exactly the pool
change that makes it the top encoder item.

### Multiple Difficulty Levels
- Easy: current random bot
- Medium: rule-based heuristics (responsive countering, threat assessment)
- Hard: Monte-Carlo tree search or minimax over the simplified game state

---

## Infrastructure / Dev

### Engine test coverage — the old gap list is stale, and the real finding is a ratio

The three "priority gaps" this section listed (combat, the layer system,
stack ordering) all closed without anyone striking them out: `core_rules`
alone carries 75 combat tests in `combat_keywords.rs`, 57 that cite CR 613 /
layer ordering / timestamps, and `golden_trace.rs` compares whole games
action-for-action, which is the only thing that catches a reordered
iteration. Checked and removed at the fifty-fourth pass rather than left to
be re-derived.

**What is open is the pure-data sweep, and it is smaller than it looks.**
`scripts/find_data_tests.sh` finds **174 non-sacred pure-data tests** (plus
25 marked `[CR]`, which are sacred) across 62 files — asserts that only echo
`CardDefinition` shape, the largest cluster being `stx/part_23.rs` (19),
`modern/lands_equipment_vehicles.rs` (11), `core_rules/format.rs` (11) and
`classic_sets/rna.rs` (11). Folding them into one table-driven audit per set
is the standing rule.

**Do not justify it on build time.** 174 tests at ~8 lines is ~1,400 of the
suite's **375,047** lines — 0.37 % — and PERF's "Build time" section already
measured a 537-line cleanup as inside the noise band, because the rebuild is
dominated by relinking the integration binaries. The justification, if a run
takes it, is maintenance shape only.

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
and deterministic bug reproduction. Partially covered: `CRAB_REPLAY_DIR`
(`server/replay.rs`) logs the event stream per match, and
`CRAB_DECISION_LOG` (`server/decision_log.rs`, 2026-08-18) logs every
*human* action beside what the heuristic bot would have done from the
same position — one JSONL line per decision with an `agree` flag and a
disagreement tally in the footer. That second log is the bot-debugging
instrument: sort by `agree:false` and read the disagreements (it's how
converge-blind payment would have been caught from game data). The
viewer's first tier exists (2026-08-18): replay files are v2 — each
line carries first-appearance card names, since wire events hold only
ids and a file has no live state to resolve them — and
`cargo run -p crabomination --bin replay_view [file] [--all]` narrates
one as readable prose (newest file under `$CRAB_REPLAY_DIR` by
default; `--all` includes the mana/tap noise). Still open: an in-client
replay mode driving the real renderer with step/seek, and state-hash
checkpoints for deterministic reproduction.

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

**Status legend:** ✅ done, 🟡 partial, ⏳ todo. Phases **A, D, E, G, H** are
shipped — N-player construction, multiplayer combat, APNAP priority,
team-aware loss/game-end, and the replacement-effect framework. Git history
carries the per-phase detail; only what is still open is listed here.

- **Phase F — shared turns (2HG)** 🟡. Shared life pool and its polish are
  done. ⏳ CR 810.5 shared-turn priority ("active team's primary player
  first, may yield to teammate"): rotation is per-seat today and both
  teammates already get priority inside the 4-passes-to-advance loop, so
  this is cosmetic.
- **Phase H — known limitation, accepted for the phase's scope.** Inline
  `graveyard.push` / `hand.push` / `exile.push` sites outside the three
  wired entry points bypass the replacement resolver. `Effect::Destroy`,
  `Effect::Exile`-from-battlefield and `move_card_to` all hit the wired
  paths; ETB-triggered direct pushes are the gap, and likely do not need
  replacement coverage for Commander.

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

## Simulation throughput

**`PERF.md` is the record — baseline, log, profile of record, candidates.
Nothing here duplicates it.** What this section keeps is the one structural
fact the rest of the file leans on: the heavy zones (battlefield / stack /
exile / per-player library, hand, graveyard, command, sideboard /
continuous_effects) are `CowBox`-wrapped (`crate::cow`), so a
`GameState::clone` — probe, probe template, `evaluate_action_outcome`, the
`perform_action` checkpoint — is reference bumps plus only the zones the
action mutates. The sharp edge that follows from it, and the one that keeps
producing perf rows: **any `&mut` access unshares, including an `iter_mut`
used read-only or a write that changes nothing.**

**The second structural fact, added 2026-08-24: an actor's per-game work is
not all simulation.** `selfplay_train`'s `actor_loop` calls `sealed_pool`
twice and `heuristic_sealed_build` twice **per game** — 32 candidate builds
a side under `--deck-judge` — and until pass 53 that was ~485 M Ir a game
against ~48 M for the game itself. `cube::card_def` memoizes
`CardFactory -> CardDefinition` and it is now ~20 M. Measure deck work with
`bot_ladder --decks sealed --games 1`, which plays no games at all.

Remaining scaling levers that are *not* engine instructions: the racing
schedule (`racing_rounds` + small `games_per_pairing`); the deck builder's
own residual ((-39) in PERF); and, if ever needed, early adjudication of
stalled games via `eval_material`.
