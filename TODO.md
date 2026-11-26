# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling, grouped by area
and roughly ordered by impact within each group.

This file is the working handoff; the archives are reference. Each carries an
index table at the top (triaged at the sixty-seventh pass — don't re-take that).

- `ENGINE_BACKLOG.md` — engine backlog and audits, ordered bugs / mechanics &
  primitives / rules coverage / tooling. Its last block, "The 2026-09-12/13
  handoff detail, moved verbatim from TODO's NEXT", holds the printed-body skip
  taxonomy, the `cap` bucket's four shapes, the sweep-table discipline, the
  closed-with-a-reason leads, the `EventKind` fan-out census, the modal census,
  the collision log and the seed discipline. Findings go there, not into NEXT.
- `CARD_BACKLOG.md` — per-set card residuals, titled by subject, open first.
- `CLIENT_BACKLOG.md` — the Bevy GUI backlog. The client builds here after four
  apt packages; that file's header has the command and the disk caveat.
- `PERF.md` — how to measure, the standing rules, baseline, log, profile of
  record, candidates. `PERF_ARCHIVE.md` holds, verbatim, the Log from `(-249)`
  down, the Baseline's closing states from `(-185)` down, and the candidates
  from `(-192)` down (incl. the `(-90)`/`(-92)` maps and `(-174)`'s open half).

## NEXT — the handoff. Rewritten each run; <= 15 lines. Every number lives in PERF.

1. **FIRST:** `git fetch origin claude/modern_decks && git checkout -B claude/modern_decks origin/claude/modern_decks`. Rebase, never force; code before tracker prose; fetch before you start and before every push (concurrent ML / Commander sessions land engine commits mid-run). Gotchas in CLAUDE.md, measurement in PERF's "How to measure" + "Standing rules", the 2026-09-12/13 detail in ENGINE_BACKLOG's "moved verbatim" block.
2. **Gates at the closing tip `cf9a3ad1`:** suite 20,327 / 0 / 6 (`CRAB_ANSWER_LOG=strict`); clippy 0; `--bench` 195,806 / 27.49 / 611.9 / 0 stalls, byte-identical, `determinism ok`, `thread_determinism ok (3 vs 1)`; `audit_panics` 0 bare; every mirror ratchet at its floor (`seat_walks` 0, `retarget` 0, `each_player` 1-allowlisted, `target_opponent` 1 known, invented-* 0, `loop_splice` 13/13/0/0); both 614 gates clean; CR_COVERAGE 146/146. Pod smoke seeds 9103-9108 × 2..8 seats × 2,000 games: all decided, 0 panics; mean turns/game is linear in seats at ~12.2 turns a seat above two — a run that bends that curve is changing the format, not a card. **FRONTIER 9109.** ⚠ `--bench` plays `--decks fixed` only, so a byte-identical `--bench` says nothing about cards in other pools — when a change touches cards, run the pools that hold them (cube / sos / sealed). ⚠ The suite count moves ~200 with the box (the client's tests build only when its apt deps are present). Later session `011z4qQX` gates (suite 22,875 / 0 / 7, `--bench` 200,190 byte-identical, pods decided): ENGINE_BACKLOG under that id.
3. 🔎 **The method: rank a clone by whether its owner needs to own it**, not by the line the allocation census names — walk up from the census row to the field type that asked for the copy (`(-364)`). PERF candidate (R) is this census; start there.
4. 📐 **The three shapes, in the order they paid:** ① a clone in a loop whose trip count is "1 + something nobody has" → `std::iter::repeat_n` (`(-361)`); ② a table materialized before anyone asked → ask first, and an empty table is the cleared table (`(-362)`); ③ a `Box<T>` in a cloned type with immutable T → `Arc<T>` (`(-364)`; `cargo check --workspace --all-targets` catches the misses).
5. 💡 **Top lead: PERF candidate (S).** Cheap half taken as `(-370)`. Expensive half censused, no build spent: `CowBox<Vec<T>>::push`'s unshare path costs two allocations (`cow.rs:107` buffer, `cow.rs:110` `Arc` box); `fx_pool::alloc_with` (`mod.rs:1029`) is the existing device. An empty `impl Drop for CowBox<T>` is free (and corrects `(-369)`'s rule: drop flags on a value moved through returns cost, a field's drop glue does not); the open unknown is the pool's hit rate. ⚠ Park only at a release site where the handle is already unique — parking the handle `push` replaces manufactures unshares. ⚠ `Drop` can't be specialized to `CowBox<Vec<T>>`, and a `thread_local!` in a generic body is one static for every instantiation — use a `CowPooled` trait with a macro-generated static per impl (`CardInstance`, `StackItem`, `ContinuousEffect`). The `Drop` forces `into_inner` to `ManuallyDrop` + `ptr::read`. Numbers in (S).
6. 💡 **Shape ③ has stock left:** `Box<Effect>`, `Box<SelectionRequirement>`, `Box<Selector>` / `Box<Value>`. ⚠ `Box<Effect>` is not `(-364)`'s safety case — nested effects are rewritten (X substitution, modes), so it needs `Arc::make_mut` and the write sites found before the read win is priced.
7. ⚠ **`(-363)`: an exact allocation prediction is not a prediction of the row.** `players` as `SmallVec<[Player; 2]>` removed exactly the promised allocations and lost, because a `SmallVec` read is a branch where a `Vec` read is a load. `Box`→`Arc` is one pointer hop and fine; inlining a table the engine indexes is not.
8. 📐 **The pool trilogy (`(-367)` won, `(-368)` / `(-369)` lost):** what matters is allocations recovered per call and frames the guard crosses — a `Drop` guard costs drop-flag bookkeeping at every frame it is returned through. An allocation's price is a range (PERF has it); size a lead by the bytes it copies, not only the count. A fresh container is a fresh box: re-take the base and read what it says.
9. ⚠ **An `Arc` swap needs the atomic check, and it passed** (`bench_ab.py`, noise): `simulate_match_pairs_piloted` takes `&[CardFactory]`, so each thread builds its own `CardDefinition`s and no `Arc<TokenDefinition>` is shared across threads. Check the sharing before worrying about the refcount.
10. **Perf queue:** taken `(-362)`, `(-364)`..`(-367)`, `(-370)`; refuted `(-363)`, `(-369)`. Declined, no build spent: the auto-tap `events` buffer (15 of 21 consumers move it into their own accumulator) and `PrintedList::push` as (T). Open on the shortlist: `affected_from_requirement` `types`, `computed_permanent_hinted`; also (H) (price the clone side first), (N), (P).
11. ⚠ **`fixed` is the control for a three-pool split** (`(-361)`): a row whose pools split (trigger-heavy pools move, `fixed` doesn't) is a workload row; one that moves all three together is a codegen row.
12. **The Ir profile is flat — do not re-derive it** (largest line 0.97 %). ⚠ The allocation and clone tables are not flat. ⚠ `Battlefield`'s lane word has one two-bit field left (shift 62), with a `const` assertion in `zone.rs`.
13. ⚠ **Traps, all live.** `cargo build --profile profiling-fast -p crabomination --bin bot_ladder --no-default-features` — both flags load-bearing; `nm <bin> | grep -cE " (T|t) (_)?mi_"` = 0 before filing a number. Don't edit the tree while a measurement binary builds. Never overlap two sweep blocks on four cores. Disk fills mid-run and fails a build as a compile error — clear `target/debug/incremental` before the release build. Rebuild an A/B's base after every rebase. Box timings: PERF.
14. **Build times are closed as a lever** (PERF's three build-time sections). While iterating: `cargo check -p crabomination --no-default-features`, the affected test binary only, full suite at the end.
15. **Cards & rules, leftover only:** `audit_dropped_may` 199; `audit_keyword_drift`'s last rows (bestow / flashback / kicker) each need a primitive; card skips `nocolors` 950 / `noback` 60 / `nosubtypes` 43 / `nokeywords` 29; `audit_incomplete` 858 documented approximations; `audit_panics` 0 bare. ⚠ 605 `definition.is_*()` reads remain and that is not a grep to run to the end.

## NEXT — Commander. The other 3-hour handoff; <= 15 lines. Rewritten each Commander run.

1. ⚠ **Several sessions share this branch:** fetch before you start and before every push; rebase, never force; `cargo check -p crabomination_catalog` after each rebase. Seat numbers race: after a conflicted rebase, checkout `--ours` of `pod/decks.rs` + `pod/mod.rs`, re-run `scripts/seat_decks.py` on top of the other session's seats, run `cr_903_5a_target_decks_are_legal_commander_decks`. New seats go at the end of `target_decks()`. ⚠ `scripts/gen_factory_lists.py` rewrites all of `all_factories.rs` in a new order — append your factories by hand instead.
2. ⚠ **One optimized build at a time; `CARGO_INCREMENTAL=0`; never edit the tree while one compiles** (an edit mid-build broke `target-audit` once this run). The audit binary: `RUSTFLAGS="-C debug-assertions=yes" CARGO_TARGET_DIR=target-audit cargo build --profile overflow -p crabomination --bin bot_ladder` (~10 min warm) — it checks the pod invariants. `cr_903_5a_the_official_precon_seats_play_pod_games` plays one game a seat: skip it in the suite, run it alone (`CRAB_SEAT=<prefix>`, `CRAB_SEAT_SHARD=i/n`).
3. **State (2026-11-26, `01WRuXc3U3`): 6,251 seats** — 6189-6205 singles closed by `trek_singles8`, 6206 Massimo (`01WRuXc5Gy`), 6207-6251 the 45 complete legal pair decks no one had seated (Clara Oswald / Prismatic Piper / Faceless One pairs left out: their averages mix chosen colors). DECK_FEATURES `01WRuXc3U3`. Commander rules: every row of FEATURE_ROADMAP's Commander status is ✅ save the bot's waterbend cast; Rulebreaker now has five allowances.
4. **The smoke test is `--pod-random`** (`bot_ladder --commander --pod-random --seats N --games G --seed S`): each game draws its pod from every deck. `--seats N` alone replays the first N decks — keep it only for the golden-outcome seeds. A failure prints `--pod-decks .. --first I --games 1`; replay under `CRAB_POD_TRACE=0` and, for state, a throwaway `#[ignore]` test in `pod/mod.rs` driving `play_pod_game_observed` with an observer (how both bugs this run were found).
5. **Sweeps this run (audit build):** seeds 29443-29445 × 3/4/5/6/8 seats, ~11,400 random games: 2 invariant panics (both fixed, ENGINE_BACKLOG `01WRuXc3U3`), undecided 0.1-0.2 % (CR 104.4a draws and board caps). **Next seed 29446.**
6. **Next decks** (`edhrec_scan_top.py /tmp/crab_commander` after `commander_backlog.py`'s `load_commanders` with `COMMANDER_PAGES = 68`; check `legalities.commander` — Unfinity / Secret Lair / Dune commanders are not legal): four singles short only their commander — Itazura, Yume, Miss Highwater, Sahir — each a primitive (Massimo seated as 6206 by `01WRuXc5Gy`); four two-card gaps (Sash and Waistcoat, Chira, Glava, Matoc). Chief Montgomery Scott is five short now.
7. **The CR is pinned at 2026-11-06** (`6117678c2`, `01WRuXc5Gy`): 713 Meld Cards inserted, so Monarch is 726, the initiative 727, restarting 728; Rulebreaker is 903.5e, sideboards 903.5f. Cite those.
8. **Gotchas:** inside the layer pass land types read printed except through `lands_of_computed_type`; `ShuffleSelfIntoLibrary` sets the resolving-spell flag only for the resolving spell; a random pod's draw depends on the deck count, so a seat append moves every `--pod-random` game (replay by `--pod-decks`).
9. **Earlier runs' NEXT lines** are in ENGINE_BACKLOG ("The Commander NEXT's lines as `01WRuXc3G1` / `01WRuXc4Bu` / `01WRuXc3U3` found them").

## Standing index (every number lives in PERF, ENGINE_BACKLOG or INCOMPLETE_CARDS; a line here that restates one is a line to delete)

0. **The build is the lever, not the source.** PGO is a ~24 % win and
   `-C target-cpu=native` is flat; `panic = "abort"` is landed on every
   optimized profile (`(-250)`; CLAUDE.md carries the consequence). The
   profile this file measures on is slower than `release`; LTO and PGO stack
   (matrix in PERF's Baseline). A profile must be raised under the profile it
   is consumed under, or it is partially applied with no warning — the binary
   size is the check. `scripts/pgo_build.sh` is opt-in and stays opt-in.
   Before quoting any `selfplay_train` throughput, print `t_step_ms` against
   `elapsed_s` (the same binaries read very differently by learner/actor
   balance; PERF says why). BOLT is blocked here (no `llvm-bolt`, no `perf`).
1. **Perf queue** — PERF "Perf candidates" and its Baseline carry every number;
   this is the state and the rules only.
   **Read the maps first:** `(-90)` (the whole-profile re-read, with leads and
   four things refuted by inspection — the gather count is at its floor; its
   event-kind mask was taken as `(-195)`) and `(-92)` (the profile is flat;
   six functions read by line say "no hot line" — stop looking for one;
   `cg_calls.py` and `cg_contexts.py` are what a flat profile still answers).
   **Open, in order:** `(-89)` (the combat-damage prevention cascade — eight
   rows left; both taken mechanisms lost on a neighbouring row, read the
   refutations first), `(-83)`, `(-9)`'s open half, `(-80)` rows 3/4,
   `(-51)(a)`, `(-69)`, `(-61)`, `(-59)`.
   **Taken/closed:** `(-87)` (the `zone::Battlefield` lane; then generalised to
   `(-89)`'s shield walks), `(-91)` (put the cache inside the handle it is
   keyed on), `(-88)` (half taken, half refuted by `CRAB_SBA_CENSUS`), `(-82)`
   (refuted off the dumps), `(-84)`(a)(b), `(-70)`, `(-79)`, `(-77)`, `(-60)`,
   `(-39)`, the `Box` class, `(-80)` rows 1-2, the `call_mut` census, the
   `#[inline]` family past the card-type predicates (every combination loses —
   don't rebuild it; `same_team` then won by removing the repeated question).
   **Refuted, do not rebuild:** `(-85)`, `(-86)`, `(-84)`(c), the Cauldron bit,
   `(-80)` row 1.
   **Rules this queue runs on:**
   * Run `cg_edges.py --callees` on a 3 % row before theorising about it, and
     `cg_edges.py --callers` to ask "is this already shared?" — `asks / builds`
     is a ratio in the dump.
   * A census that prices a memo's hit rate before anyone builds it is cheap
     and turns "nobody has priced it" into a shipped win (`(-87)`).
   * A ceiling is a percentage of the base it was read at (`(-91)`).
   * Price a witness against the work; a witness over the same collection as
     the work is never cheaper than the hit rate (`(-88)`).
   * Rank a `call_mut` census by the closure's captures, not its call count —
     and check the consumer: the rule is about `collect()` into a `Vec`;
     `sum`/`fold`/`count`/`for_each`/`any` already iterate internally.
   * Halve a no-LTO `#[inline]` reading and halve it again
     (`[profile.profiling-lto]` checks it); pick the candidate by what its body
     expands to at the call site, not by its self Ir. Read a hot small
     function's body for a repeated question before reaching for the attribute.
   * Divide a loop's item count by its call count before hoisting a per-item
     board walk; when the loop is short, ask whether the scope is long.
   * A per-definition presence bit pays only when the walk it replaces is over
     a list that is usually non-empty.
   * `SmallVec` cannot replace a `Vec` of borrowed data that outlives its last
     use (no `#[may_dangle]`); `DispatchScan`'s two lists are that shape and
     need a borrow refactor.
   * Price a `Vec -> SmallVec` swap as `alloc + free` minus `n x (spill check +
     external next)`; fill a `SmallVec` with a loop, never a `collect()`.
   * A `same_team`-shaped call inside a collection walk is a per-seat question
     asked per element: hoist it, put a cheaper term in front, or build a
     per-seat mask (pays only on long, otherwise ungated loops).
   * A scope only gathers if a read inside it asks for a computed view;
     `(-81)`'s named first-reads are the cheapest leads.
   * Fold a `'N` row into its parent before ranking a self table; Ir/call on a
     function a gate is about to split averages two populations.
   * A dormant gate's cost is its call count (`(-85)`'s "add gates freely" was
     calibrated at ~20 k calls). Never `#[inline]` a reader whose slow path
     allocates.
   * A rebase shrinks a patch without shrinking its measurement; cite an anchor
     by a hash already on `origin`.
   * Split a patch whose rows you want to attribute, or attribute nothing.
   **Open question, not re-landed unilaterally:** the Cauldron revert is a 20x
   asymmetric pool split (bench pool vs `sealed`, which training plays).
   Decide it deliberately or leave it.
2. **Perf method** — PERF's "How to measure", "Standing rules for a perf
   pass", "Which pool a change moves". Read all three pools; a pool split is
   a revert.
3. **Instruments** — `--feature-census` is deterministic and is the encoder's
   regression check: `selfplay_train --feature-census 8 --seed 5` twice and
   diff; byte-identical means the encoding did not move. Use it on anything
   touching `encode.rs`. PERF's "Profile of record" carries the actor's
   profile (rows no `bot_ladder` profile has). Also `CRAB_SIM_REJECTS`,
   `CRAB_PAY_FAILS`, `CRAB_SBA_CENSUS` (the SBA re-sweep rate),
   `server::bot_rejection_count`, `--bench`'s stall split and `undecided_by`,
   and `selfplay_train`'s `actors:` line (plus `actor_s` / `actor_games_per_s`
   in `stats.jsonl`). Quote `actors:`, never `done:`, for anything about the
   simulator — `done:` divides by a clock the learner owns.
4. **Encoding caution** — pool / `Vocab` / `TrainRow` / observation and deck
   encodings invalidate the trained nets. Nothing since `dc478735` touches
   `encode.rs`, `crabomination_ml` or `crabomination_nn`. ⚠ A card fix is not
   an encoding change but is a seeded-game change (a pool card that plays
   differently changes the same seed's game) — say which a card commit is.
5. **Robustness gate** — `scripts/robustness_grid.sh` + the actor leg (the
   `-C debug-assertions=yes` + `overflow` build of `-p crabomination_ml --bin
   selfplay_train`, 4 seeds × `--actors 3 --games 3000 --steps 20` — the only
   leg that reaches the encoder-side memos), the seeded 4,000-pairing cube
   sweep (arm it in `bot_vs_bot_random_cube_decks_terminate`; recipe in its
   doc comment), the two census env vars. Last green: grid at `599825ba`,
   actor leg at `6e44ce7c`, cube sweep at `527f872f`; earlier runs are in git
   — don't add a line per run. Run the grid after any pass that lands a
   `debug_assert!` or a presence gate (the suite is not their audit: an
   assertion needs a board to fire on); re-arm the cube sweep after a pass
   that changes what the bot proposes or the engine accepts. The script's
   header check (`strings | grep -c "memo is stale"` > 0, else `RUSTFLAGS`
   didn't reach the crate) counts lines, not messages — don't read it as a
   census.
6. **Bugs** — ENGINE_BACKLOG's live-match section: no open entry. Card audits
   clean — see INCOMPLETE_CARDS.
7. **ML** — ML_NOTES. `selfplay` seeds `jitter_below` per game
   (`play_recorded_game_mcts`, 2026-09-11), so a recorded game replays from its
   seed; runs before and after it draw different games (no retrain). A whole
   net-in-the-loop run is still not reproducible (`n` comes off a shared
   atomic).
7b. **Test suite** — `find_data_tests.sh` (fixed five ways; its output is a
   DELETE list, so every bug put live engine tests on it). A pure-data test is
   one with no logic under it, not one with no `GameState`. False positives
   cluster (shared helper or API) — read three tests at the top of the by-file
   count. The sacred list must be byte-identical across a fold. Doesn't fold: a
   test pinning a card-specific effect shape (modal `min`/`max`, a `Search`
   filter, a `CantBeBlockedBy(_)` / `Madness(_)` / `Suspend(4, _)` /
   `Typecycling(_)` variant), and a per-cycle table that already names its card
   in the failure message (`ths.rs` / `rtr.rs`'s `*_stat_lines`,
   `cube_rounds`'s `*_cycle_definitions`). Seven slices taken; the remainder
   (124 foldable echoes over 77 files at one to five each) buys no build time —
   take a file when already editing it, one commit per file, binary green
   either side; don't open a pass for it.
8. **Tip state / build time / filters** — PERF's newest Baseline blocks.
   Anchor measured at `79019a42` (on `origin`): `fixed` 935,959,897 / `cube`
   2,777,133,820 / `sealed` 2,773,877,997; earlier anchors are in git and
   PERF. Anchor rules: cite a hash already on `origin`; re-read the anchor,
   don't sum the rows; two sessions reading the same tip agree to ~100 Ir.
   ⚠ `--bench` is not a throughput instrument — it is the invariant
   (decisions / turns / stalls / determinism). Quote Ir, or `ab_wall.py`'s five
   ABBA blocks of `--games 2000 --decks fixed` (they resolve ±2.40 %, so a
   sub-1 % Ir change reads flat). `games_per_s` is not portable across
   containers: `--bench` runs `available_parallelism - 1` threads — quote
   `games_per_s_th` or pin `--threads`; check `host_calib_ms` and `bin_bytes`
   (a PGO binary prints `release-fast build` like a plain one) before
   believing it. PERF's "Build time" has the critical-path answer
   (`crabomination_catalog`'s harness is now `test = false`; quote a build
   number with the file it touched) and the ABBA rule. ENGINE_BACKLOG has the
   seven filters. Run the base callgrinds while the candidate builds and
   nothing else; take `release` once at the tip as the closing gate, not per
   candidate. Install nextest per CLAUDE.md; don't fall back to `cargo test`.

## Standing rules for a perf pass — in `PERF.md`

Under "How to measure". Read it before proposing anything on the perf queue.

## Environment note

The client needs system libs the base image lacks: `apt-get update && apt-get
install -y libwayland-dev libasound2-dev libudev-dev libxkbcommon-dev`, then
`cargo build/test -p crabomination_client` compiles (first build ~6 min).
`target/` disk hygiene is in CLAUDE.md ("Container notes"). Beyond that:

- `apt-get install` without a preceding `apt-get update` 404s on a stale index;
  `.claude/hooks/install-client-deps.sh` does the update but silences its
  failures — check `pkg-config --exists wayland-client` before blaming the
  crate. The GUI can't be run headless (see the `verifier-client` skill), but
  its code and unit tests compile and test here.
- A full disk arrives as `rustc` exit 101 with no diagnostic — check `df`
  before debugging the crate. Cargo never garbage-collects `release-fast`
  rebuilds under different `RUSTFLAGS`; `rm -rf target/debug/incremental
  target/debug/examples target/release-fast` frees space without a workspace
  rebuild. `scripts/robustness_grid.sh` fills `target-audit/` (~3 GB with the
  actor leg) — `rm -rf target-audit` once the grid is green; delete copied
  A/B binaries (215 MB each) once their callgrind dumps are taken.
- Two concurrent cold builds fit in 15 GB but each takes ~twice its solo wall
  time: run builds in series for throughput.
- Killing `cargo` does not kill its `rustc` children — after cancelling a
  build check `pgrep rustc` (read `/proc/<pid>/cwd` and `--crate-name` to tell
  whose it is).
- A build started before a `git rebase` compiles the pre-rebase source.
  Rebase first, then build; if a rebase lands mid-build and touched the graph,
  restart it.

## Client — defects from the 2026-11-23 code audit

Ten read-only auditors over `crabomination_client`; the detail (file:line,
scenario, evidence, fix) is `CLIENT_BACKLOG.md`'s "Code audit, ten areas
(2026-11-23)" section, **(V)** = confirmed by a second read. The five that
changed game actions are fixed (trample split, keyboard selection eating
clicks, choose-N modes, stuck pass-until, sticky audit target).

**Wrong action / blocked action**
- [ ] Alt-cost modal: Enter / Space also cast the hovered card / pass.
- [ ] Attack All guesses attackers instead of `cv.legal_attackers` (one "can't attack" creature rejects the batch; stolen creatures left out).
- [ ] Pods: a seat nobody attacked is stopped to declare blocks (`declares_blocks` is the duel rule).
- [ ] Next Turn / End Turn presses dropped unless holding priority.
- [ ] Rematch after a lobby / LAN / puzzle / draft match starts the wrong game.
- [ ] Cast pickers aren't `InGameRoot`: they outlive the rope, game over and the match.
- [ ] Take-back request stuck after a reconnect (auto-pass dead for the game).
- [ ] Rejected resume token: "reconnecting" banner forever; dead Rejoin kept.

**Board / panels show the wrong state**
- [ ] (V) Rows, clicks and the abilities menu key on `owner`, not `controller` (stolen permanents).
- [ ] (V) Pod seats 2+ have no graveyard pile or player target zone.
- [ ] Animated Vehicles / lands keep their spawn row and never show tapped.
- [ ] Transform / turn face up never re-skins; back-face MDFC art path wrong.
- [ ] Exiled cards fly to the library / graveyard pile.
- [ ] Spectating a duel stacks both boards on the far side.
- [ ] Opponents' face-down hand cards leak into the next match.
- [ ] (V) Combat preview re-projects dealt damage (false LETHAL) — server `combat_preview`.
- [ ] (V) Attackers all read "unblocked" after blocks are declared.
- [ ] (V) "Zzz" through the opponent's turn on creatures that can block.
- [ ] Pod threat alarm sums every opponent (on at full life).
- [ ] (V) Spectators see "Defeat.".
- [ ] Alt peek and zone browsers don't refresh while open; MDFC back face shown twice.
- [ ] Rematch carries life history / life ticker; `MatchStats` leaks into puzzles.
- [ ] Chess clock not reset, ticks after game over, bills humans for bot think; rope toast lost after a manual-mana suspend.
- [ ] "→ target" for spells targeting spells; combat hints stale after declarations.
- [ ] First-strike commander damage double-counted; turn-order strip ignores reversal; Hint dead after leaving mid-think.
- [ ] Match history: pod losses lost on early leave; puzzles / lobby filed as Modern; spectated lobby game filed as a loss.
- [ ] Draft: a card move re-splits basics (splash lost); no exit before the deck is built.

**Keyboard / pad**
- [ ] Hand-card picks (discard, Brainstorm) not keyboard-reachable; `ModalRoots` lacks the mode picker, ability / hand menus, cast pickers.
- [ ] Modal focus starts on the invisible root and returns there after each move.
- [ ] Pad keys injected unordered against `PreUpdate` readers; remaps onto fixed keys unwarned; keymap Done dead while capturing.
- [ ] Auto-rematch field not a text field; X opens the bug prompt with "x"; empty hand menu eats Esc.
- [ ] (V) Menu text fields: no space bar, no ':' in paths, no `~`; focused field types under overlays.

**Text**
- [ ] (V) Raw engine errors in the log ("CardId(42)"); (V) Debug enums ("Step → PreCombatMain", "PlusOnePlusOne", "TimeLord", "Now a Elf").
- [ ] Brainstorm prompt says bottom (it's top); search wording for non-library picks; OrderBottom captioned "Top".
- [ ] Seat placeholders ("May play (P1)", "seat 1", "→ P2", "player 2", "#7").
- [ ] (V) Draft / sealed seat named "You" ("You loses", "You's", "You wins").
- [ ] Plurals ((V) "1 land drops", (V) "Picked 1 cards", "1 picks", "(1 cards)", "1 spells", "2 bot"); land-drop chip on opponents' turns.
- [ ] Lobby: bots counted twice, menu format ignored, fake Sealed.
- [ ] Labels that ignore rebinds (Space, Keep (K), Mulligan (M), V, X, I).
- [ ] Danger-coloured "on the play" / "your priority" chips.
- [ ] Alt tooltip duplicates abilities / equip / ward ("Ward {2}"); recast badges as mana value ("Flashback {3}").
- [ ] Legend-rule buttons indistinguishable; draft opponent colours mismatch; "Pick 17/15".
- [ ] Host LAN busy port → silent bot game, port 0 accepted; in-game render quality not saved; spectator chat silently dropped.

## Engine — Robustness / defects

**No open entries.** The determinism class, the panic/unwrap census and the
twenty-three robustness filters are closed; the whole record — what each
filter hunted, what it found, the rule it yielded (nine found nothing, which is
the result worth keeping) — is in `ENGINE_BACKLOG.md`. Read it before
proposing a robustness sweep.

Standing goals: no panic/unwrap reachable from bot self-play; cross-process
determinism at a fixed seed (golden traces, and `CRAB_THREAD_CHECK=1 --bench`
across thread counts); stall rate tracked on `--bench` (`stalls_by cap / stuck
/ draw` — re-open only if `cap` or `stuck` goes non-zero).

## ML — defects (index)

### Every committed deck net fails to load — FIXED for the future, not for those nets

`VOCAB_SNAPSHOT` froze the embedding index at the fifty-fourth pass;
`pad_vocab` zero-extends a shorter table. The seven committed
`*/deck-latest.safetensors` predate the freeze, and `vocab_fit` refuses them by
name; `nets/champion.safetensors` is unaffected. Resolved 2026-08-24 for the
artifact: no retrain was needed — run directories already held vocab-164 deck
nets, and the re-gated r45 one (a statistical tie on the historical band) is
committed as `nets/deck-champion.safetensors`, so `--use-deck-best`,
`--gate-builder-hc` and `--distill-gen` are live again and future card
additions only pad it. Full write-up (and the out-of-range clamp found
alongside): `ML_NOTES.md`.

## Engine — Missing Mechanics

In `ENGINE_BACKLOG.md` (moved at the sixty-second pass; nothing closed by the
move).

## Engine — Approximation Cleanups

Earlier approximations are resolved (`git log -p -- TODO.md` has the per-card
primitives + tests). Still open:

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

One mechanism, two deliverables: (a) transactional action application — every
rejected `GameAction` restores the exact pre-action state, structurally killing
the audit-P0 partial-mutation family (Squad/Casualty under-pay,
`declare_attackers` mid-loop corruption, back-face land corruption, madness
mana loss); (b) player-facing undo/take-back — instant vs the bot,
consent-gated in multiplayer. The same checkpoint recorder later feeds the
replay scrubber and crash recovery.

**Approach: whole-state snapshots, not inverse commands.** `GameState` has a
hand-written `Clone` (`game/mod.rs:859`) and full serde, and bot dry-runs
already clone per candidate; inverse ops for the effect resolver would be
unmaintainable and inherit every funnel-bypass bug.

### Phases 0 and 1 — ✅ shipped
Seeded serialized `GameState.rng` behind every "at random" (undo cannot
re-roll a shuffle; replay is bit-exact); persisted-history serde fidelity is
still gated on the `CardInstanceWire` dropped-fields fix and the round-trip
property test (Infrastructure → Snapshot Round-Trip Test), which in-memory undo
does not need. A checkpoint at the top of `perform_action`, restored on `Err`,
for every action (`cow::tests::rejected_action_restores_state_exactly`).
Semantics, each a bug first:
- Suspension is not failure: `GameError::ManualTapRequired` is exempted — it
  leaves forced pips tapped and mana floating for the client's pending-cast
  driver (`sos::mana_shapes` tests).
- The restore keeps the live decider; the checkpoint clone holds a blank one,
  and swapping that in wipes a `ScriptedDecider` mid-script.
- A failed resume restores to the suspended state, not to before the original
  action. Multi-step atomicity across a suspend/resume chain is future work.

**Perf:** the checkpoint is the engine's largest single structural cost and is
never read on the bench workload (PERF's forty-third pass, candidate (-13)).
Any narrowing is a rules-correctness argument first and a perf change second.

### Phases 2–5 — player take-back (plan, 2026-10-01)

The plan (snapshot ring, `SubmitAuto`, consent, view rewind) is in
`ENGINE_BACKLOG.md`'s last section, "Rollback / Undo — Phases 2–5".

---

## Bot / AI

### Playing stack — next steps after the combat fix (2026-11-11)
**Status (ML_NOTES "Five search flags, gated").** Items 2a, 2b, 4 (`-wd2`, a
missing sooner-win preference) and 5 (`-hr`) are adopted as a bundle,
`MctsConfig::with_search_fixes`; item 1 (`-ts`) was flat, so item 3 does not
run. The lobby's Modern / SOS duels play the client's pilot. The blunder set
has 65 items; the client fails the three "save the answer for the bigger
threat" items. The leaf reads real boards correctly (the shells were the
problem — ML_NOTES "What the leaf misreads, and why", "The counter item on
real positions"): on real boards (`bot_puzzles --real-counter`) the client
still spends the counter where holding wins, because the menu search's hold
arm is a priority pass whose rollouts spend the mana later in the turn. The
Murder item is the heuristic's opponent-turn removal timing plus a policy
prior that decides a tie. The hold arm (`-ho`) was re-gated on real-board
items and **adopted 2026-11-21** (`MctsConfig::with_hold_arm`, b7b90e4b7: NO
LOSS on SOS / FRA / cube / Modern, real-board counter items 10 / 10). The
2026-11-23 audit found its rollouts refused only plain `CastSpell` (fixed
behind `-hc`, round 88) and that the real-board truths deal the real hand
(`--honest-truth`, round 88; C5: the hold survives an honest truth). Round 88: `-rp` costs a real-board counter item, so the other four were adopted without it (`with_audit_fixes`). Its remaining gap is rollout fidelity inside a
3-turn horizon.

The 2026-11-11 item list, by outcome (reasoning in ML_NOTES "Why the
champion reads 0.442 on a 0.500 base rate", "The combat window of the menu
search", "The blunder set"):
1. **Two-seat leaf** (`-ts`, averaging `win_prob(seat)` with
   `1 − win_prob(opp)`) — flat, not adopted; so 3 does not run.
2. **Two search fixes** — (a) no Gumbel noise at play time; (b)
   `decision_menu::key` stops merging same-named permanents in different
   states. Adopted in the bundle.
3. **Value training, decided by 1** (an antisymmetric output or a p + p' = 1
   penalty, value only, separate policy trunk) — not run.
4. **An ability arm in the menu search's main window** — adopted as `-wd2`
   (the miss was a sooner-win preference, not a missing arm).
5. **Held removal** — adopted as `-hr`.
6. **Harder blunder items** — the set is at 65; the crack-back item (an attack
   that loses to the crack-back two turns out) is not built — no position
   isolates it.
7. **The FRA and lobby pilots on the set** — read (ML_NOTES).
No training run without the user's go-ahead.

### The net chooses every move (2026-11-25, the user's direction)
No heuristic picks a move, rollouts included, batched to pay for it. Plan
and Phase 0 are in ML_NOTES "The net chooses every move". Phase 0 found
that a net-played move is 51 % move-list building (~100 trial applications
a decision in `decision_menu::accepts`) and 43 % forward. Batching alone is
<= 1.75x, so Phase 1 is a cheap legal-move list plus one-forward scoring
(the action head, which `nets/policy-champion` lacks). Policy-played
rollouts are 19x uniform ones today. Phase 1 part 1 (0d70bdd45): the list
is −27 %, and there is a root-only action head (one forward a decision). The
2026-11-25 root-only gate is NOT adopted: SOS 49.6 is level, FRA ~47.0, and
blunders 66/75 vs 69/75 (it loses aim items). Next: FRA actors, a
target-owner role, or a hybrid with successor re-scoring of the top few.
See ML_NOTES "Phase 1 gate".

Round 88 follow-ups:
- `-ia` re-gate on the fixed binary (8f5ceceda);
- `-hc-sv-rs-kx` ADOPTED 2026-11-25 (`with_audit_fixes`); `-rp` alone loses
  `real_counter_112`, so it is unexplained and off;
- an SOS-only value slot for `nets_r88_R2_s97` (fails plain-card blunder
  items as the all-format champion);
- the engine accepts a repeated empty `DeclareAttackers` in one step (the
  `-ia` loop's enabler).

### One net for every mode, Commander included (2026-09-28)
The plan, its gates and per-step code locations are ML_NOTES "One net for every
mode". Step 1 (the table net, `server::encode_table`, `server::card_tokens`,
parity in both crates) is built and untrained; next is step 2, the any-seat
recorder and mode mixer. No training run without the user's go-ahead.

### Instant-Speed Responses
`pick_stack_response` counters an opponent's spell that targets the bot or its
permanents, or costs 3+ — cheapest affordable counter first, `would_accept`
dry-run as the final gate (so Spell Snare's MV filter etc. are honored).
Future: respond with removal/protection instants, not just counters;
race-aware "is this worth a card" valuation. `buff_2for1` (kill the creature
under the opponent's own pump, round 43) is built and zero-incidence in bot
mirrors — human-facing, default off.

### Sacrifice Prioritisation
Forced sacrifices sort candidates tokens first, then lowest CMC, then lowest
power, inside `Effect::Sacrifice` (edicts and activated-ability costs alike).
Future: respect "you may sacrifice" optionality (skip when the cheapest
candidate is worth more than the payoff).

### Planeswalker Targeting
The bot redirects attackers at an opponent's planeswalker when total
attacking power can finish it in one swing, smallest power first (`b34a23a`).
`walker_chip` (round 43: one declaration at the lowest-loyalty unfinishable
walker, sims judge it) is zero-incidence in the walker-free sealed pools and
stays default off on the strength of the recorded ten-turn-ultimate loss.
Still open: the inverse case (a low-loyalty walker not worth committing
trample beaters to).

### Smarter Mana Rock Usage
The bot taps mana rocks eagerly before knowing what it wants to cast. A "plan
this turn's spending first" pass before mana-ability activation would avoid
tapping a Sol Ring with nothing to cast.

### Encoder / net follow-ups — moved to `ML_NOTES.md`
In `ML_NOTES.md` → "Encoder and net follow-ups (moved from TODO, 2026-08-23)":
the belief-head recall diagnostic, the encoder-v7 gap list, the static-anthem
P/T hole, the feature-occupancy precondition and the next-round candidates.
Load-bearing for this branch: **static-anthem P/T is invisible to the
encoder**, and a modern-decks pool makes it the top encoder item.

### Multiple Difficulty Levels
- Easy: current random bot
- Medium: rule-based heuristics (responsive countering, threat assessment)
- Hard: Monte-Carlo tree search or minimax over the simplified game state

---

## Infrastructure / Dev

### Engine test coverage — the old gap list is stale, and the real finding is a ratio

The old priority gaps (combat, the layer system, stack ordering) are closed:
`core_rules` carries combat tests in `combat_keywords.rs`, CR 613 / layer /
timestamp tests, and `golden_trace.rs` compares whole games action for action
(the only thing that catches a reordered iteration).

Open is the pure-data sweep (`scripts/find_data_tests.sh`; see Standing index
7b): fold card-shape echo tests into one table-driven audit per set. Don't
justify it on build time — the suite's rebuild is dominated by relinking the
integration binaries (PERF "Build time"); it is maintenance shape only.

### Snapshot Round-Trip Test
`GameSnapshot` and `GameState` serialisation exist. Add a property-based test
that plays N random actions, serialises/deserialises the state, and asserts
game continuity — catching any `Serialize`/`Deserialize` drift.

### Card Correctness CI
`scripts/verify_cards.py` (with its Scryfall cache) verifies CMC, P/T, types,
and keywords. Wire it as a CI step against `scripts/.scryfall_cache.json` (no
network) to catch regressions when catalog entries change.

### Bot vs. Bot Simulation
Automate a "run 1 000 cube games bot vs. bot, report win rates by colour pair"
script — catches degenerate card interactions and unbalanced pools without
manual play.

### Replay / Game Log Export
A replay file format (sequence of `(action, resulting_state_hash)`) would
enable post-game review and deterministic bug reproduction. Partly there:
`CRAB_REPLAY_DIR` (`server/replay.rs`) logs the event stream per match (v2
files carry first-appearance card names, since wire events hold only ids), and
`scripts/replay_scan.py DIR` reports over a directory of them.
`CRAB_DECISION_LOG` (`server/decision_log.rs`) logs every human action beside
what the heuristic bot would have done — one JSONL line per decision with an
`agree` flag and a disagreement tally; sort by `agree:false` to debug the bot.
Still open: an in-client replay mode driving the real renderer with step/seek,
and state-hash checkpoints for deterministic reproduction.

### Scryfall Art Pre-fetch CLI
`all_cube_cards()` drives the in-game prefetch, but there is no standalone CLI
to warm the asset cache. A `cargo run --bin prefetch_art` that downloads
missing Scryfall images to the local cache would speed up first-session loads.

### WASM / Web Build
`Cargo.toml` already has a `wasm-release` profile. Completing the web build
(removing native-only dependencies, adding a WASM server bridge) would make the
game playable in a browser without installation.

---

## Formats

### Commander + Two-Headed Giant — what the phased rollout left open

Phases A–N shipped (FEATURE_ROADMAP's "Commander status" is the map). Open:
- **2HG, CR 810.5 shared-turn priority** ("active team's primary player first,
  may yield to a teammate") — rotation is per-seat; both teammates already get
  priority inside the passes-to-advance loop, so this is cosmetic.
- **Replacement coverage of inline zone pushes** — `graveyard.push` /
  `hand.push` / `exile.push` sites outside the wired entry points bypass the
  replacement resolver; ETB-triggered direct pushes are the known gap.
- CLI / deck-loader entry points should accept a format.

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

Every row of the old table (cube / demo-deck cards needing only existing
primitives) has shipped; git history has the per-card detail.

## Simulation throughput

`PERF.md` is the record. Two structural facts the rest of this file leans on:

- The heavy zones (battlefield / stack / exile / per-player library, hand,
  graveyard, command, sideboard / continuous_effects) are `CowBox`-wrapped
  (`crate::cow`), so a `GameState::clone` — probe, probe template,
  `evaluate_action_outcome`, the `perform_action` checkpoint — is reference
  bumps plus the zones the action mutates. **Any `&mut` access unshares,
  including an `iter_mut` used read-only or a write that changes nothing.**
- An actor's per-game work is not all simulation: `selfplay_train`'s
  `actor_loop` builds sealed pools and decks per game (32 candidate builds a
  side under `--deck-judge`); `cube::card_def` memoizes `CardFactory ->
  CardDefinition`. Measure deck work with `bot_ladder --decks sealed --games
  1`, which plays no games.

Remaining scaling levers that are not engine instructions: the racing schedule
(`racing_rounds` + small `games_per_pairing`); the deck builder's residual
((-39) in PERF); and, if ever needed, early adjudication of stalled games via
`eval_material`.
