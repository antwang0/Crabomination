# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.


Split for size (this file is the working handoff; the two archives below are
reference and want their own triage pass):

- `ENGINE_BACKLOG.md` — the long engine/rules/client backlogs and audits
  (follow-ups not yet done, suggested next-up tasks, the CR coverage audit,
  the decision-plumbing audit, client UX/visualization).
- `CARD_BACKLOG.md` — per-set and per-run card residuals: what each closed set
  still approximates, and the remaining gap lists.
- `PERF.md` — the perf record: baseline, log, profile of record, candidates.

## NEXT (handoff — rewrite each run, keep it terse)

**FIRST COMMAND:** `git fetch origin claude/modern_decks && git checkout -B
claude/modern_decks origin/claude/modern_decks` — the container clones `main`,
~2,200 commits behind. **Sessions run this branch concurrently: expect to
rebase mid-run and re-read your numbers afterwards.**

1. **Nothing is in flight. Re-read at `1,314,288,098` Ir** at `03ab571d`
   (pass 50's tip plus pass 49's three; the three commits above it are
   comment-only). Pass 50's own reading of its tip was 1,314,421,002, and the
   133 k gap is not accounted for by anything in between — **re-read your own
   base**.
   Pass 50 reads `1,531,246,782 -> 1,314,421,002`, **-14.160 %** in five
   commits (A -6.958, B -2.022, C -4.703, D -0.875, E -0.316); pass 49 under
   it -5.785 %. Suite **18,712 / 0 / 5** over 22 binaries, traces unchanged,
   `cargo clippy --workspace --all-targets` clean, `--bench` invariants
   byte-identical (196,220 / 27.53 / 0 stalls / determinism ok, peak RSS 21.6).
   **No net needs retraining.**
2. **A determinism bug the wide-pool sweep could not see is FIXED
   (`c6898506`), and the sweep recipe changes because of it.** `restart_game`
   (CR 727) rebuilt the state with `GameState::new`, whose `GameRng` is
   `from_entropy` — so every game that *restarted* stopped being a function of
   its seed, and the antithetic pairing broke on exactly those games. It hid
   for 49 passes because the sweep only ever ran **three seeds at one thread
   count**. **Run it as thirteen seeds across `--threads 1/2/3`**, and read
   the sweep count, not just the panic count: in a `gang`-vs-`gang` mirror
   every pair must split, so one sweep is a bug. `CRAB_PAIR_SWEEPS=1` names
   the offending pair and prints the seed that replays it. Post-fix: 88,400
   games, 88,382 decided, no panic, **all 42,391 pairs split**.
3. **Read PERF's new ranking rule first: *ask what is done twice*.** All five
   rows are one class — `would_accept`'s dry run **is** the action, and the
   caller then performed it again. Invisible as a hot function; the tell is a
   validate-then-do pair.
4. **`cg_lines.py` works now and did not before** — it annotated libc against
   this binary's DWARF and hardcoded the PIE bias. Cross-check any line row
   against `cg_edges.py`'s call counts; a `profiling-lines` build is ~40 min.
5. **Best next moves.** (-32), the class's last row: `main_phase_action_with`'s
   2,036 probes, 7.2 %, blocked on `Bot::next_action` carrying only an action
   and on the driver's live decider — its own pass. Then
   `dispatch_triggers_for_events` (5.60 % self, now readable per line), (-35)
   `evaluate_requirement_static` (2.52 % self / 182,532 calls), (-33), and
   `resolve_combat` at 55,816 Ir a combat. **`cast_candidates` is closed** —
   see (-34).
6. **A twenty-second robustness filter is owed, and filter 21's bug names its
   shape:** *state the harness installs that a rules path can reset*.
   `restart_game`'s `*self = GameState::new(...)` dropped the seeded `rng`,
   the live `decider` and two per-seat pilot flags, all of which are
   simulator configuration rather than rules state. Anywhere the engine
   rebuilds or replaces a whole struct, ask what the caller had put in it.
   (18, 19, 20 all landed in the *measuring devices*; that vein is not
   exhausted either.)
7. **Owed housekeeping.** PERF 4.0k — the forty-sixth pass's profile table is
   the next fold. TODO 1.14k — the filter write-ups 12-20 are the compaction.
   **Actor scaling is measured and closed** — linear to 4 cores (98.4 %
   at `--threads 4`, mimalloc); see PERF. It says nothing about 8+ actors.

## Standing rules for a perf pass

Durable, not per-run. Every refutation named here is written up in **PERF**'s
Log with its numbers; read the entry before re-proposing any of them.

- **Rank the tail, not the function** (pass 49). A chain of narrow generators
  is invisible in a self-cost profile and in a callee table sorted by Ir; it
  shows up only in the **call counts** — twenty-two rows at exactly 2,176
  calls each, once per traversal, on a board with nothing for any of them,
  4.9 % together. Wherever the code reads as a fallback chain, count the rows
  before costing them. The device is `spec` / `gated_block!`'s and the debug
  audit is what makes a mask safe to over-approximate.
- **Ask what a tick pays when the answer is "nothing to do."** Four of pass
  49's five rows are that question at a different level: a
  twenty-two-generator fallback chain, three land blocks asking the same
  question, a gate that gathered to prove a negative, a freeze scope opened
  for a closure that returns immediately, two clones handed to a walk that
  skips.
- **The gate rule, both halves.** Swap a gather for a presence gate **only
  where nothing else in the scope reads the gather** (pass 48's (E),
  -0.747 %); where the scope goes on to `compute_battlefield()` the same swap
  is **+0.30 %**. `layers_memoized()` answers "is the gather already built".
  Fusing a cheap per-card question into a walk that already happens has lost
  four times; removing a walk outright still pays.
- **The `Keyword::eq` device is not exhausted, and it has a trap.** No LTO
  here, so any small non-generic `crabomination_base` function is an
  out-of-line call in every profile this file quotes — but a bare `#[inline]`
  would be unmeasurable in the shipped `release` (thin LTO) build, so **do not
  take one on an Ir number**. What works is making the callee smaller than any
  inliner threshold, which is what `has_kw` does.
  `CardDefinition::is_creature` is the same family and the same trap.
- **Measurement.** Read PERF's "How to measure" — pass 48 rewrote it.
  `scripts/cg_symbolize.py` + `scripts/cg_edges.py`, never
  `callgrind_annotate --tree` for a caller table. **`cg_edges.py`'s shares
  were ~18x low until `ac85463f`** — a table read before that commit ranked
  work upside down; re-derive anything carried forward from one. Re-read your
  own base: on a shared branch the commit you stand on may not be the one the
  last pass measured, and argv length lands in the Ir total (~500 Ir).
- **Measurement gotcha, hit three times.** The 6-game ladder printout (24
  decided / 12 splits) does **not** move on a change that breaks something;
  only `--bench`'s `decisions` and the Ir total catch it. Run
  `./target/profiling-fast/bot_ladder --bench --threads 3` on any change whose
  Ir moves more than its blast radius allows.
- **Do not rebuild these.** The board-presence epoch, the `GameState` husk
  pool, gating `do_untap`, narrowing `GameState`, splitting the big engine
  files for build time, the per-definition keyword-grant bit, fusing
  `card_type_change_in_scope`, the `LayerFreeze` depth shadow, the
  `sba_board_scan` definition bitmask, the trigger-carrier bitmask, the APNAP
  rank table, the headroom-reserving `Vec`, `board_keyword_matching`'s
  presence gate, and (-31)'s `improves_this_turn` reuse. And **never** skip
  `push_ordered_trigger_candidates` on an empty batch (+7.3 % *and* a
  correctness bug — it owns the per-batch `died_card_snapshots.clear()`).
- **Env.** No `cargo-nextest`; `cargo test -j 2 -p crabomination -p
  crabomination_tests` is the gate (~20 min from cold on this box). Workspace
  clippy needs `apt-get update && apt-get install -y libwayland-dev
  libasound2-dev libudev-dev libxkbcommon-dev`. Cold `profiling-fast` engine
  build ~14 min, warm rebuild ~4m30s; callgrind ~4 min and contention-immune.
  Wide-pool sweep ~55 s a seed — no excuse to skip it. Quote callgrind under
  5 %; a `profiling-fast` games/s compares to nothing.
- **Trackers.** TODO ~1.0k, ROADMAP 0.66k, PERF ~3.6k (the 45th pass's profile
  table was folded at the 48th tip; the 46th's is the next fold when its Log
  rows stop chaining). ENGINE_BACKLOG 4.6k and CARD_BACKLOG 4.2k are the
  archives and still want their own triage pass.

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

## Engine — Robustness / defects (open)

### Robustness filters (the determinism entry itself closed 2026-08-11, `841dd40b`)

The cube pool's fixed-seed nondeterminism is fixed and the whole class is
shut: `crate::fxhash::HashMap` / `HashSet` (rustc's seedless FxHasher)
replace `std`'s across the engine, so no map's walk order can differ
between two runs of one seed. Same-seed decision counts are identical over
repeated runs on every pool (`cube` 1,130,728, `all` 2,548,986, `sos`
684,268, `fixed` 193,232), `determinism ok` on all of them, and `all`'s
stall rate is a stable 6 rules draws / 5,100 games (0.12 %). A separate leak
fixed in the same sitting (`125108c1`): CR 705.1 coin flips read
`rand::random()` inside `AutoDecider`, and Mana Crypt is in the cube pool.
**Re-checked at the forty-seventh tip**: `--decks all --games 400 --threads
3`, seeds 11/12/13 — 20,400 games, 20,396 decided, no panic, all 10,198
mirrored pairs split.

**What is left of it, as a rules question, not a determinism one.** A map
whose walk order picks a *game outcome* is still arbitrary, just
reproducibly so. Known site: `actions.rs`'s discard-cost gate does
`by_name.values().find(|ids| ids.len() >= count)` for "discard N cards with
the same name" (Kozilek-style), which picks an arbitrary qualifying name
rather than the cheapest. Sweep the ~110 map/set locals for siblings when
someone wants a rules pass; none of them can desynchronize a run any more.

*(No open entries. The audits that closed here are an index; `git log -S` on
each hash has the prose.)* `df87c2d1` — `CardData.counters` becomes the
insertion-ordered `CounterBag`; `86670250` — the same for `KeywordCounters`;
`ea8cc1fd` — `died_card_snapshots` becomes `IdMap`, because a
`TriggerCandidate`'s position decides stack order and two LKI deaths stacked
differently per process. **That was the survey's one leak in 31 fields** —
every `HashMap`/`HashSet` on `GameState` / `ColdState` / `Player`, asked of
each consumer whether it sums, tests membership or looks up by key (all safe)
or `find`s / `collect`s / iterates into an ordered structure (not). The three
that *look* risky and are not, so nobody re-checks them: `encode.rs` sums
`block_map` into a map read by key, `bot.rs`'s two `block_map.keys().collect()`
are `contains` + `len`, and `combat.rs`'s `block_map.keys().for_each(want)`
decides which permanents get computed, never what a reader sees. Also
`a67c5b9a` (actor-sampler panic) and `9db8557c` (Mirror Gallery aborting the
whole SBA sweep — a `return` inside a `let … = { … };` initializer, so one
board skipped every later state-based action and the game could not be won or
lost; regression test in `classic_sets/bok`, and the filter that found it was
swept workspace-wide).

**The panic/unwrap sweep of the self-play path — CLOSED 2026-08-23 by the
census under filter 16, and it wanted triage, not the blanket rewrite this
entry used to ask for.** 84 `unwrap()` + 46 `expect(` in non-test engine
code; drop the `lock()/read()/write().unwrap()` poison calls and **118
sites** remain, every one reachable from self-play guarded by a preceding
test that is usually named in a comment or the message. One dead hazard,
not reachable from anywhere: `impl Index<&CounterType> for CounterBag`
panics on a missing kind and has no `counters[..]` call site. **Re-run the
census after a batch of new cards, not per run.** The seventeen narrower
filters below are what got run instead of the rewrite, and nine of the
seventeen found nothing — which is the result worth keeping. The section
has **no open entries**.

**The seventeen filters, compacted to an index.** Each is a *shape* that fails
the way a training run notices — a silent wrap at game 400 k, a loud panic,
or a hang — swept over `game/` + `bot.rs` (some wider). The prose is in
`git log -- TODO.md`; what is kept is what each hunted and why it is
closed, so none of them is re-derived.

| # | date | the shape it hunts | result |
|---|---|---|---|
| 1 | 08-10 | A `debug_assert!` standing in for a runtime guard, or a `len() - 1` / bare index on a slice whose emptiness the *caller* tolerates | **Found `a67c5b9a`** (`sample_scored_index`, on the one path only a training actor takes). Both halves then swept clean: 13 `len() - 1` sites all guarded; the two surviving `debug_assert!`s (`mod.rs:3900`, `stack.rs:5911`) fall through to defined release behaviour |
| 2 | 08-10 | A `return` inside a `let … = { … };` initializer | **Found `9db8557c`** — CR 704.5j's Mirror Gallery check aborted the *whole* SBA sweep, so one board skipped every later state-based action and the game could not be won or lost. Regression test in `classic_sets/bok`. The other nine workspace hits are `Err` / let-else guards that legitimately abort |
| 3 | 08-10 | Unsigned `len() - k` where the caller tolerates empty; a stale index across a mutation (`position()` then `battlefield[pos]`) | Clean. 16 + 53 sites; the one path that mutates in between (the equip sacrifice) re-finds by id and says why |
| 4 | 08-10 | `evaluate_value(…) as usize` with no `.max(0)`; `power()`/`toughness()`/`life` cast to `usize` | Clean. One hit each, both already clamped (`mod.rs:20879` is `.max(1)`; `bot.rs`'s `LIFE_TENTHS[life as usize]` sits under its own two branches) |
| 5 | 08-10 | A precondition *some* sites enforce and a sibling might not — documented `///` preconditions, and the `i.min(xs.len() - 1)` clamp family | Clean. Eight doc'd preconditions all validated or structural; all ten clamps guarded, by four different idioms |
| 6 | 08-11 | Not syntax — **run the arithmetic**. `[profile.overflow]` (`release-fast` + `overflow-checks`) turns every silent wrap into a panic with a backtrace | Clean. `bot_ladder` 4 seeds x 4 pools = **17,693 games, 0 panics**; `selfplay_train --actors 3 --games 600` = 600 games / 56,353 rows / 0 panics. **Rerun after any change to counters, damage, mana or the encoder** — one ~9-minute build, ~1 minute a seed |
| 7 | 08-11 | The opposite of 1-6: a `/` or `%` whose denominator is a runtime count the caller can zero (panics loudly, or goes `NaN`) | Clean. Every non-constant divisor under `game/`, `bot.rs`, `crabomination_ml/` read; seat rotation always has a seat, the rest are `.max(1)` or guarded by an `is_empty()` in the same condition |
| 8 | 08-11 | A std collection/slice op whose runtime argument is a *length*, not an index — `split_off`/`split_at`/`copy_from_slice`, `chunks`/`step_by` with runtime `n`, `&xs[a..b]`, `Vec::remove`/`insert` | Clean. Five + two + two + ~30 sites; the ML `copy_from_slice`s copy fixed-width **arrays**, so a mismatch is a compile error, not a panic |
| 9 | 08-11 | A comparator that is not a total order (`sort_by` panics on one; a `NaN` produces one for free) | Clean. **No `partial_cmp(…).unwrap()` in the workspace**; every float comparator is `total_cmp` or `unwrap_or(Equal)`, and the three that could see a `NaN` are in `recommend.rs`, off the self-play path |
| 10 | 08-11 | The failure a training run sees as a *hang*: an unbounded `loop`/`while` whose exit condition is game state | Clean. All eight `loop {` and ~40 `while`s bounded by one of three shapes — a strictly shrinking collection, a finite effect-tree peel, or an explicit counter. The one bounded by none of the three is the top-level game loop, and its two counters are exactly what a *stall* is |
| 11 | 08-11 | **One invariant written out by hand in more than one place** | **Found `15ec11c1`**: `stale < 8` appeared six times across five files, so the ladder's stall rate and the training actor's were never the same measurement. All six read `recommend::STALE_ROUNDS` now; no value changed. The per-context *action* budgets beside them are deliberately different and were left alone |
| 12 | 08-14 | **A predicate two callers each re-derive** | **Found two**, `caa44eb2` — see below |
| 13 | 08-14 | **A reentrancy guard some sites spell out by hand and a sibling does not** | **Found a stack overflow** — `in_layer_gather`, unguarded at ~a dozen computed-P/T arms the gather evaluates. See below |
| 14 | 08-15 | **A comment that states a cost or a shape** | **Found one**, in the measuring device: `host_calib_ms`' doc claimed you could scale a throughput comparison by it. The syntactic half is clean; see below |
| 15 | 08-23 | **A claim made by a tool's output rather than by its source** | **Found one**, in the measuring device again (`95453974`): `--bench` printed "release build" for `release-fast` / `profiling-fast` / `overflow`. See below |
| 16 | 08-23 | A default no caller ever overrides | Clean in four readings; don't re-run — see below |
| 17 | 08-23 | **A comment that names a call count or a share** | **Found five** across two concurrent runs. The share survives, the count rots. See below |
| 18 | 08-23 | **A tool's own extraction step** | **Found two** (`ac85463f`), both in the profiling scripts: `cg_edges.py`'s total was ~18x high, `cg_lines.py` returned a silent zero. See below |
| 18b | 08-24 | filter 18 re-run on `cg_lines.py` (**a tool's own extraction step**) | **Found two more.** It folded every mapped object's addresses (libc, ld.so, libm — 16.5 % of the run) in with the binary's and hardcoded the PIE bias. 36 % of the run resolved to `??` and the rest to the wrong symbols; `Effect::clone` read 2.65 % against the 0.5 % its call edges account for. See below |
| 20 | 08-24 | **A default that only one caller ever exercises** (the inverse of the sixteenth) | **Nearly clean — one hit.** 14 of 36 `EvalWeights` knobs have exactly one overriding profile; ten of those profiles carry an on/off test, four do not, and three of the four are correctly untested (a scoring weight, a historical control, a net-dependent blend). The fourth, `smart_tap`, was real engine behaviour with no test; it has one now. See below |
| 21 | 08-24 | **An invariant checked at one point in its parameter space** | **Found a determinism bug** (`c6898506`). The wide-pool sweep had only ever run three seeds at one thread count; ten more seeds across `--threads 1/2/3` produced a self-mirror pair that did not split. `restart_game` (CR 727) rebuilt the state with `GameState::new`, whose `GameRng` is `from_entropy`. See below |
| 19 | 08-24 | **A threshold or cap that silently truncates a listing** | **Found seven** across two concurrent runs — `cg_edges.py`'s three tables and `cg_lines.py`'s two caps (`4107e017`), plus `cg_symbolize.py`'s recommended `--threshold` and three ranked report tables in `bot_probe` / `selfplay_train` / `recommend_pool`. See below |

**A note the table would lose**: filters 3-5 and 7-10 are syntactic and
found nothing between them. Filter 6 is not syntactic — it *runs* the
program with the checks on — and filters 2, 11, 12 and 13 look at structure
rather than syntax. Four of the five filters that found something are in
that second group. Prefer a filter that runs the code or reads its
structure over one that greps it.

**Filters 12-14, compacted; `git log -- TODO.md` has the prose.** All three
hunted structure rather than syntax and all three found something.

* **12 (`caa44eb2`) — a predicate two callers each re-derive.** The search
  that works is not syntactic: it is `grep -niE "must (also )?(appear|be)
  (listed|added|here)|kept in sync|must agree|drift from"`, i.e. the doc
  comments that admit to the pairing. Nine hits, three real.
  `CardData::clear_end_of_turn_effects` wrote 26 fields and
  `end_of_turn_effects_are_clear` guarded that write by listing the same 26 —
  both expand from one `eot_wear_off!` list now, so a field cannot reach one
  without the other. `rewrites_land_types` asked in prose to be kept in step
  with `layers::compute_permanent_pass` — now the `ability_strip_in_scope`
  device, a `debug_assert!` at the gate that runs the layer pass it skipped
  and fails if the computed land-type line differs from the printed one.
* **13 — a reentrancy guard some sites spell out by hand and a sibling does
  not.** Found a **stack overflow**: `in_layer_gather` was unguarded at ~a
  dozen computed-P/T arms the gather itself evaluates, so a card pairing a
  gather-evaluated filter with a P/T requirement overflowed rather than
  answering wrong. The guard lives in `computed_permanent` now, once, where
  it cannot be forgotten.
* **14 — a comment that states a cost or a shape.** Found one, in the
  measuring device: `host_calib_ms`' doc claimed you could scale a throughput
  comparison by it, and two containers with the same `host_cpu` and
  overlapping calib differed by **24 %** on `--bench`. **The syntactic half is
  clean and should not be re-run** — ~60 hits for present-tense cost claims
  over `game/`, `server/`, `crabomination_base/`, all game-semantics uses of
  "free"/"cheap" or past-tense justifications. Both filters' yield was in
  claims a *measurement* relies on, not in the engine's prose.
**The sixteenth filter was run 2026-08-23 and is CLEAN in every reading it
has** — *a default that no caller ever overrides*. Mechanically, over
`crabomination`, `crabomination_base` and `crabomination_ml`: (i) a `bool`
parameter every call site pins to one literal — **zero**; (ii) an
`Option<T>` parameter every call site passes `None` — **zero**; (iii) an
`EvalWeights` field no profile constructor ever sets away from the baseline —
**zero of 36**; (iv) a `GameState` / `ColdState` field with no read outside
its own definition — **zero of 196 / 91**. A looser first pass (a function
whose call sites show only one bool *literal*, variables ignored) returns 14
hits and every one is noise — `default_damage_split(has_trample)` reads
"always false" only because the single literal site is the zero-power case.
Don't re-run any of it.

**And the standing panic goal was audited the same day: also clean.** 84
`unwrap()` and 46 `expect(` in non-test engine code; drop the
`lock()/read()/write().unwrap()` poison calls and 118 sites remain. Every one
reachable from self-play is guarded, and almost all name the guard in a
comment or a message: the five `back_face…unwrap()` sites each sit behind an
`is_none()`/`has_back` check, the six `max_by_key(..).unwrap()` "look at top
N" sites each behind an `is_empty()` return, `mayhem_cost().unwrap()` behind
`mayhem_cost().is_some()`, `hand.first().unwrap()` behind
`hand.is_empty() { continue }`. One dead hazard, not reachable:
`impl Index<&CounterType> for CounterBag` panics on a missing kind and has no
`counters[..]` call site anywhere. Re-run the census after a batch of new
cards, not per run.

**The seventeenth filter was run 2026-08-23 and is NOT clean — four hits.**
*A comment that names a call count or a share.* Mechanically: grep `//` and
`///` lines over `crabomination/src` and `crabomination_base/src` for a
comma-grouped integer, a `N.N %`, or the token `Ir`; that returns 163, of
which **44 are perf claims a reader would rank work by** (the rest are the
bot's win-rate gate history, which is dated and self-describing). Checking
those 44 against the forty-seventh tip's profile:

* `CardDefinition::printed_color_set` — "the layer pass calls this ~1.2 M
  times per six bench games". It is **114,562**, 99,840 of them the layer
  pass. An order out, and the function's own 4.6 M self cost rules the claim
  out on its face.
* `GameState::team_of` — "`same_team` is called ~1.2 M times per six games".
  `same_team` is **319,624** and reaches `team_of` twice each, so the fast
  path answers ~658 k times. Roughly 2x out.
* `dispatch_triggers_for_events`' death-synthesis chain — "94,608 of them
  over six bench games". **53,838** dispatches reach that line.
* `auto_tap_for_cost_inner` — "`mana_source_table`, 3.83 % of the profile
  over 8,892 calls". The *share* is right to two decimals (3.85 %); the
  **count** is 7,550.

**The pattern in all four: the share survives and the count rots.** A share
is re-derived every time someone reads a profile; a call count is copied
forward from the run that first found it and nobody re-reads it, so it
drifts every time the caller's own count moves. Corrected in place, each
with the tip it was read at. **Caveat for whoever re-runs this:** a
callgrind edge count *undercounts* a caller that got inlined, so only
correct a number down when the callee has its own node and its self cost
makes the old claim arithmetically impossible — which is why `team_of`'s
correction is stated through `same_team`'s node rather than its own.

**Pass 48 ran the same filter concurrently and found two more, both of the
same shape and both now corrected:** `mod.rs`'s gather-iterator note claimed
"the 127,878 gathers a six-game run takes" (**39,692** at the forty-eighth
tip — four passes of scope work have been taking it down), and `types.rs`'s
`IdSet` doc claimed "`RawTable::clone` ran 984,988 times under the CoW
unshare for 1.22 %" in the present tense when the number was the
*justification* for `IdSet` existing (**34,220 / 0.28 %** now, and none of
them on a `ColdState` field). **Five distinct sites between the two runs** —
both caught the mana-source one independently — and the direction is the same
in every one.

**The twenty-first filter was run 2026-08-24 and is NOT clean — it found a
determinism bug the suite, the golden traces and forty-nine passes of
wide-pool sweeps had all missed.** *An invariant checked at one point in its
parameter space.* The wide-pool sweep is `--decks all --games 400 --threads 3`
on seeds 11/12/13, and it had never been run on another seed or another thread
count. Ten more seeds found a self-mirror pair that did **not** split — which
the pairing's own contract forbids, since a pair's two games are one game with
the seats relabelled.

The cause: `GameState::restart_game` (CR 727) rebuilds the state with
`*self = GameState::new(players)` and copies back `next_id`, `attack_option`
and `teams` — but not **`rng`**, and `GameState::new` installs
`GameRng::default()`, which is `from_entropy`. Every game that *restarted*
therefore stopped being a function of its seed from that point on.
`decider`, `smart_tap` and `wants_ui` were dropped the same way and are
carried back too. Fixed in `c6898506`; the regression test replays the exact
pair on eight threads.

**Two things worth keeping from the hunt.** The thread count was a red
herring — it only changed how often a pair's two halves happened to draw the
same entropy — so *the parameter that exposed a bug is not always the
parameter that causes it*. And the tool that made it tractable was five
minutes of work: `CRAB_PAIR_SWEEPS=1` on `bot_ladder` names the offending pair
and prints the seed that replays it, which turned "somewhere in 68,000 games"
into a 1.2-second test.

**The structural fix is a rule, not six edits, and it is in CLAUDE.md's
Performance section:** a profile number in a code comment carries the tip it
was measured at, or it is written in the past tense as the justification for
the shape the code already has.

**The eighteenth filter was run 2026-08-23 and is NOT clean — two hits, both
in the profiling scripts pass 48 added.** *A tool's own extraction step* —
the place the fifteenth filter, pass 46's retracted allocator rows and pass
48's `callgrind_annotate --tree` truncation all came from.

* **`cg_edges.py`'s program total was ~18x high**, so every share it printed
  was an order out: `total += cost` ran on every cost line, and a call-edge
  line carries the callee's whole *inclusive* subtree. PERF's "How to
  measure" already *said* the total double-counts and to take the program
  total from valgrind's `I refs:` line — but a note beside the tool does not
  fix the tool, and nothing said that the **percentage column of every table
  it prints** is computed from that same total. A reader ranking by the
  column the tool actually shows them ranks upside down. **A known-wrong
  number that a comment warns about is still a wrong number.** It printed
  28,094,793,086 for a run whose `I refs` is 1,540,962,924, and reported
  `dispatch_triggers_for_events` at **0.30 %** where it is **5.63 %** — a
  reader ranking work by that output ranks it upside down. Fixed in
  `ac85463f`; the total now sums self lines only and cross-checks the dump's
  own `totals:` line, printing a WARNING on a mismatch instead of a plausible
  number.
* **`cg_lines.py` printed "0 instruction addresses, 0 Ir" and exited 0** on a
  dump taken without `--dump-instr=yes` — which reads as "nothing is hot",
  not "wrong input". It refuses at the `positions:` header now.

`cg_lines.py`'s cost parse is otherwise correct (it already excludes the
post-`calls=` line, which is exactly the mistake `cg_edges.py` made) and
`cg_symbolize.py` already reports its own hit rate to stderr. **The rule the
filter yields: every extraction step either agrees with a number its source
computed itself, or refuses.** An extraction that yields nothing looks
exactly like a measurement that found nothing.

**The nineteenth filter was run 2026-08-24 and is NOT clean — seven hits
across two concurrent runs, three of them in the same two scripts.** *A
threshold or cap that silently truncates a listing.* `cg_edges.py` printed its caller/callee tables at `most_common(40)`
and its self-cost table at `most_common(45)`; `cg_lines.py` resolved only
`most_common(60000)` addresses and printed `most_common(45)` lines. Every one
stopped at its limit and read as finished — the shape behind pass 48's two
fictional allocator leads. Each now reports the rows it dropped and the Ir or
calls that went with them (`4107e017`).

**The first thing that fell out of the fix is worth having on its own: the
self-cost table's top 45 rows are 68.5 % of the program, and 1,150 rows hold
the other 31.5 %.** A profile that diffuse is why pass 49's wins came from
counting call rows rather than ranking by self cost.

**Filter 18 was re-run on `cg_lines.py` the same day and is NOT clean — two
more, and they are why the line profile had never been trusted.** It summed
the instruction addresses of *every* object the process mapped and resolved
them against the annotated binary's DWARF, and it hardcoded a `0x108000` load
bias where the `profiling-lines` binary needs `0`. The output looked like a
profile: 36 % under `??` and `Effect::clone` at 35,279,138 Ir / 2.65 %, which
`cg_edges.py`'s call edges put at 2,890 calls and 0.5 M. **The rule that
catches this is the eighteenth's own** — agree with a number the source
computed, or refuse — so it now keeps one object, auto-detects the bias,
prints the hit rate, and exits when fewer than half the hot addresses land in
a symbol or when the dump names a different binary. PERF's note blaming lld
identical-code folding for a bogus `drift::sort` row is probably this bug.

**A second run swept filter 19 concurrently and found four more outside those
two files** (`17d0a5e1`): `cg_edges.py`'s **docstring contradicted its own
printer** — "reads the dump directly, so a table is complete" sat above
`most_common(40)`, and the parse was complete where the print never was
(`--rows N`, `0` = all, now lifts the cap); `cg_symbolize.py`'s docstring
**recommended `callgrind_annotate --threshold=95/99`**, the exact truncation
`cg_edges.py` exists to escape, from the file a reader hits *first*; and
three ranked report tables named no denominator (`bot_probe`'s wedged-action
and combat-shape tops, `selfplay_train`'s leaf/train ratios,
`recommend_pool`'s static scores).

**What it did *not* find, and the distinction is worth keeping.** The
engine's own `take`/`truncate` caps — `attack_search`, `block_search`,
`MAX_CANDIDATES`, `MAX_FOLLOW_UPS`, convoke's `cap` — are named
search-quality decisions with doc comments, not silent truncations of a
listing someone reads, and filter 8 already swept that syntax for panic
safety. **Across both runs the whole yield was in output a human ranks work
by** — the third filter running to land in the measuring device.

**The twentieth filter was run 2026-08-24 and is NEARLY clean — one hit.**
*A default that only one caller ever exercises*, the inverse of the sixteenth.
Mechanically over `EvalWeights`: parse `baseline`'s 36 field literals, parse
every one of the 59 named profile constructors, and keep the fields exactly
one of them sets to a different value. (`v2` and `scaled_control` are
alternative *baselines* — they set all 36 — so they are excluded, which is the
step that makes the question answerable at all.)

**14 fields have exactly one overriding profile.** Ten of those profiles are
exercised by a unit test as well as by a `bot_ladder` pilot name —
`impulse_draw_on`, `converge_lands_on`, `buff_2for1_on`, `ability_arms_on`,
`walker_chip_on`, `mull_sim_on`, `creature_base_only`, `legacy_mana` and the
two search flags — and the tests read "flag off: the class is invisible /
flag on: the activation is a candidate". That convention is what makes the
four exceptions legible.

**Three of the four are correctly untested**, and saying so is most of this
filter's value: `power_emphasis_only` is a scoring *weight* (`power: 15`), so
the question it asks is a ladder question; `legacy_cashout_on` is the
*historical* planeswalker rule kept as a control, and the shipped behaviour is
what a test should pin; `net_eval_blend_ply` needs a loaded net. **The fourth
was a real gap.** `smart_tap` routes through `PlayerData::smart_tap` into
`auto_tap_for_cost_inner`'s source choice, where it makes a coloured pip spend
the *least flexible* source — the engine's own comment says "a Swamp pays {B}
before a Dimir dual does" — and nothing tested it.
`core_rules::game::smart_tap_spends_the_narrowest_colour_source_first` does
now: same board both ways, and the two arms assert opposite outcomes, so it
cannot pass vacuously.

**The rule it yields:** an opt-in flag in this codebase is expected to carry
both a ladder pilot name *and* an on/off test, and the ten that do are what
make the four that don't findable. A flag whose question is a *measurement*
(a weight, a control, a net) is the documented exception — say so at the
flag, so the next sweep does not re-derive it.

**Stall rate — CLOSED 2026-08-14, and the answer is "nothing to fix".**
`419d2ea6` put `recommend::StopReason` on the outcome and a `stalls_by cap /
stuck / draw` line on `--bench` (and `stalls_capped` / `stalls_stuck` in
`selfplay_train`'s `stats.jsonl`), and reading it settled the entry: `--decks
all --games 300 --seed 11` reads 6 stalls in 5,100 games (0.12 %), **cap 0 /
stuck 0 / draw 6** — all rules draws, so neither held-open fix applies.
`--decks fixed` reads 0 and always has. Keep the instrumentation; re-open
only if `cap` or `stuck` goes non-zero.

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

### Client build in the routine sandbox — CLOSED 2026-08-23
The four dev packages install via apt and `cargo clippy --workspace
--all-targets` then builds `crabomination_client` outright (**Environment
note**, top of file). Runtime/GPU verification still needs the local
`verifier-client` skill; only compile-checking works headless.

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

### Multiplayer / Commander / Planeswalkers — mostly SHIPPED, index only
This entry claimed the engine had no command zone, no commander damage, no
emblems and no planeswalker attacks. All four exist: `Player::command` with
`command_zone_abilities_active()` and a `ClientView` field,
`commander_damage` with a per-source tally surfaced in the view (CR 903.10a)
and a regression test, `Player::emblems` joining the layer gather's anthem
walk, and `AttackTarget::Planeswalker` chosen by the bot's attack search
(`walker_chip`) and resolved by combat. **Still open, and that is all that is
left here:** four-player free-for-all match setup in `run_match` /
`build_cube_state`, colour-identity deck building and commander tax, the
"your opponents" vs "each other player" multiplayer targeting split, and
CR 118.3c planeswalker damage redirection.

### Saga Lore Counters
✅ Non-DFC Sagas ship via `CardDefinition.saga_chapters` + `saga_advance`
(ETB chapter I, +1 lore each precombat main, final-chapter sacrifice SBA).
History of Benalia, The Eldest Reborn. Remaining ⏳: DFC/transforming sagas
(The Everflowing Well saga-land) and read-ahead chapter-choice variants.

### Vehicle / Crew and divided damage — SHIPPED, index only
`GameAction::Crew` / `Saddle` are real actions with a bot picker
(`pick_crew_vehicle`), and divided combat damage is resolved by
`free_division_targets` + the CR 510.1c assignment path (Butcher Orgg's
`DividesCombatDamageAmongDefenders` included). **Still open:** a
`DealDamageDivided { total, targets }` *spell* effect — Pyrokinesis-style
"4 damage divided as you choose among any number of targets" is still
collapsed to a single-target hit.

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

Remaining scaling levers that are *not* engine instructions: the racing
schedule (`racing_rounds` + small `games_per_pairing`), and, if ever needed,
early adjudication of stalled games via `eval_material`.
