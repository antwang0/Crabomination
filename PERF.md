# PERF

Numbers, not prose. Every perf change gets a row in **Log** with what
changed, the before/after, and how it was measured. No measured win means
revert — or keep it and say plainly that it's a correctness/clarity change.

Benchmarks and profiles run on optimized builds (see CLAUDE.md's carve-out).
A number from a debug build describes `opt-level = 0`, not the code.

## How to measure

```text
# throughput — the committed configuration
cargo run --release --bin bot_ladder -- --bench

# allocator A/B — mimalloc is the default now, so the *system* allocator is
# the opt-in side. A feature change on the engine crate is a full rebuild, so
# the variants need separate caches; /target-mi/ is gitignored.
cargo build --release -p crabomination --bin bot_ladder
CARGO_TARGET_DIR=target-mi cargo build --release -p crabomination \
  --bin bot_ladder --no-default-features

# instruction-level profile (deterministic; no `perf` in the routine image).
# Profile the system allocator: valgrind replaces malloc, so a mimalloc build
# measures the interception, not the program.
cargo build --profile profiling -p crabomination --bin bot_ladder \
  --no-default-features
RUST_MIN_STACK=33554432 valgrind --tool=callgrind --callgrind-out-file=cg.out \
  target/profiling/bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1 \
  --decks fixed
callgrind_annotate --auto=no --threshold=95 cg.out            # self cost
callgrind_annotate --auto=no --inclusive=yes cg.out           # inclusive
callgrind_annotate --auto=no --tree=caller cg.out             # who calls whom
# the one that found this run's fix: callers of `malloc`, and the call
# counts next to each. Self cost lies about allocation — a function with
# 1.9 % self can be 35 % of every malloc in the program.
callgrind_annotate --auto=no --tree=caller --threshold=99 cg.out > tree.txt

# behaviour preservation
cargo test -p crabomination_tests --test core_rules golden_trace

# build time
cargo build --timings -p crabomination
```

**Iterating with `release-fast`.** A `release` rebuild of the engine is ~25 min
on a 4-core box (`codegen-units = 1` + thin LTO); `release-fast` (cgu 16, no
LTO) rebuilds in a few minutes and is what A/B iteration should use. It is a
*different profile*, so its absolutes never go in **Baseline** and never
compare to a `release` number — but a candidate and its baseline both built
`release-fast` and alternated in one sitting is a sound relative measurement,
and that is what a Log row needs. Say which profile a row used.

`--bench` pins the workload — `gang` (= `EvalWeights::default()`, the
profile the bot actually plays) mirrored against itself, 4 hand-built
archetypes, 80 games each, seed 20250808, paired — so two runs on different
days measure the same games. Per-thread games/sec is reported next to the
aggregate: a change that moves only the aggregate is a scaling change
(contention, allocator), one that moves the per-thread rate is a change to
the game loop.

**Absolute games/sec is not comparable across routine boxes, and barely
across an hour on one.** Three back-to-back runs of one binary read 11.73 /
12.34 / 10.01 games/s here — a 23 % swing — and `host_calib_ms` moved 63 / 55
/ 69, in exact inverse rank order. `--bench` prints that probe (a fixed
deterministic ALU + 4 MiB random-access loop, timed after the games so it
can't perturb them) plus `host_cpu` precisely so the next run can tell a
slower host from a regression. Check it *before* investigating a moved
baseline. The only sound way to attribute a delta to code is to measure both
sides in one sitting, alternating A/B/A/B — host drift then moves both.

**Sub-5 % changes need callgrind, not `--bench`.** Two runs of one binary
here differ by more than a 2 % code change is worth, so a small win reads as
noise however many pairs you run. `callgrind` on a fixed workload counts
instructions deterministically: build both sides `release-fast`, run each
under `--tool=callgrind` on `--a gang --b gang --games 6 --threads 1 --seed
1 --decks fixed` (~3 min each, and both can run at once — instruction counts
don't care about contention), and diff `I refs`. Keep the allocator the same
on both sides; the absolute number then describes mimalloc's interception,
but the *ratio* is sound. Wall-clock is still the arbiter for anything
allocator- or cache-shaped, where Ir undercounts.

**`--threads N` needs enough work to fill N workers.** The queue holds
`decks x ceil(games / 20)` chunks under `--paired`; a worker that finds it
empty exits. `--threads 24 --games 8` therefore runs four workers and looks
exactly like "scaling flattens at 4". The run now prints a note when this
happens — heed it. Use `--games 120` or more for an actor-scaling sweep.

A self-mirror on a shared seed must report **every pair as a split**
(`rho -1.000`): the two games of a pair are the same game with the seats
relabelled. A sweep in a `--bench` run is a determinism bug, not variance.

Wall-clock notes for whoever iterates next: a release rebuild of the engine
took 24 min solo on this run's box (`codegen-units = 1` + thin LTO), and 32
min each when two ran concurrently — budget two or three measured iterations
per run, not ten. Callgrind on six games takes about three minutes and is
contention-immune, which makes it the better first look.

## Baseline

Re-anchored at the tip of this run's perf work (`release`, built from the
commit that adds the mana-walker freeze). Refresh only alongside an
intentional, explained change. Regressions beyond ~5 % get investigated
before anything else lands — but check `host_calib_ms` first (see "How to
measure").

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz
host_calib_ms        48 / 65 / 58 / 48            <- compare this first
games                320
games_per_s          33.26 / 33.46 / 31.98 / 33.80   (mean 33.13, spread 5.7 %)
games_per_s_th       11.09 / 11.16 / 10.66 / 11.27
decisions_per_s      20085 / 20208 / 19314 / 20408   (mean 20004)
turns_per_game       26.98
decisions_per_game   603.9
stalls               0 (0.00 %)
peak_rss_mib         38.1 - 40.8
determinism          ok (160 pairs, 0 sweeps, all pairs split)
```

**Read this block before concluding anything from the absolute.** The
previous anchor (`eb5f661c`, same box model, one day earlier) read mean
**33.64** with `host_calib_ms` 69/70/70/69. This block reads **33.13** with
the probe at 48-65 — a *faster* probe going with slightly slower games,
which is the same inverse relationship the "How to measure" note describes,
and the two blocks were taken in different containers. The -1.5 % is inside
this block's own 5.7 % spread and inside the 5 % investigate band, and the
pre-run binary no longer exists on disk, so the two cannot be alternated.
**The per-change rows in Log are what carry this run's claims**, and both
were measured in a single alternated sitting on this box:

```text
release-fast, alternated A/B, one sitting each
13,307,099,945 -> 13,052,911,075 Ir   computed_permanent memo        -1.91 %
13,052,911,075 -> 12,235,211,102 Ir   mana walkers frozen            -6.26 %
                                      cumulative this run            -8.06 %
27.43 -> 28.67 games/s                mana walkers, 6/6 pairs        +4.5 %
```

The instruction counts are the same fixed six-game workload throughout, so
the -8.06 % subtracts honestly; the wall-clock figures do not, which is the
whole reason this run measured in Ir. Nothing regressed on the correctness
side: `turns_per_game` held at 26.98 and `stalls` at 0 across all 28 bench
runs of both A/Bs and this block, determinism ok everywhere, and all golden
traces are byte-identical.

The actor-scaling sweep from an earlier run (system allocator vs mimalloc
at 1/4/8/16/24 threads) is untouched; its readable conclusion was that
mimalloc's edge grows with actor count (+9 % at 1, +31 % at 24) while its
RSS ratio stays flat at 1.5-1.7x. Re-measure on a box with real cores.

Historical steps, kept because the percentages carry even though the
absolutes don't: 19.76 at one morning's branch tip, 21.38 after the trigger
dispatcher stopped computing the whole board (+8.2 %), 23.00 after the grant
scans were hoisted (+3.9 %), 16.08 -> 22.83 for the affordance-sweep fix
(+42.0 % fixed decks, +52.7 % sealed), 21.94 -> 23.03 for the sim-loop
freeze (+5.0 %). Each was its own alternated sitting; none of them subtract
across the gaps.

## Log

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-08 | Gate the layer gather's three graveyard tallies on a `dynamic_pt` being present; drop the O(n²) battlefield-membership test in the `AnthemForFilter` walk (`b17a76b`) | 10.36 games/s, 6253 dec/s | 11.25 games/s, 6791 dec/s | `--bench` ×3 each side; golden traces byte-identical, turns/game unchanged |
| 2026-08-08 | Run the whole `RandomBot` tick inside one `with_frozen_layers` scope (`e919496`) | 11.25 games/s, 6791 dec/s | 12.22 games/s, 7381 dec/s | `--bench` ×3 each side; golden traces byte-identical, turns/game unchanged |
| | **cumulative this run** | **10.36 games/s** | **12.22 games/s (+18.0 %)** | both ends measured post-build; on the settled box the same code reads 11.85 |
| 2026-08-08 | Opt-in mimalloc `#[global_allocator]` on `bot_ladder` / `selfplay_train` (`--features mimalloc`) | 12.39 games/s, 7478 dec/s | 13.88 games/s, 8378 dec/s | `--bench` ×3 each side on an idle box (12.42/12.38/12.36 → 13.82/14.07/13.74); **+12.0 %**. Two separate release builds into `target/` and `target-mi/`, allocator the only difference. Peak RSS 25.3 → 39.0 MiB. turns/game 26.98 unchanged, stalls 0, all pairs still split. |
| 2026-08-08 | Filter the layer gather's 39 static-ability battlefield passes through one precomputed slice (`80086059`) | 9.64 games/s, 5825 dec/s | 12.32 games/s, 7437 dec/s | **+27.8 %**. `--bench` ×3 per side, *alternated* A/B/A/B in one sitting on an idle box (9.75/9.69/9.49 → 12.15/12.42/12.38). Sealed pool, the shape self-play trains on: 10.64 → 13.51, **+26.9 %** — so it isn't a vanilla-deck artifact. turns/game unchanged (26.98 / 19.01), stalls 0, RSS unchanged, all pairs split. Suite 18617 passed / 0 failed; all four golden traces byte-identical. |
| 2026-08-08 | mimalloc from opt-in to default on `crabomination` + `crabomination_ml` (`bbf5ddcc`) | 12.12 games/s, 7317 dec/s | 14.77 games/s, 8918 dec/s | **+21.9 %**. `--bench` ×3 per side alternated, `host_calib_ms` steady 54–57. Larger than the +12.0 % row above because the gather fix removed the work the allocator cost was hiding behind. RSS 25.4 → 41.5 MiB at 3 threads; the actor sweep in **Baseline** is what unblocked making it default. |
| | **cumulative this run** | **9.64 games/s** | **14.49 games/s (+50.3 %)** | both ends on the same box, same day; see the Baseline note on why the previous run's absolutes don't compare |

| 2026-08-08 | CoW-wrap the per-turn / per-game tally collections (cast-name / id / profile logs, ETB + death lists, delayed triggers, graveyard + discard sets) so a state clone stops deep-copying them | 20.12 games/s | 20.10 games/s | **no win — reverted.** `--bench` x8 per side alternated in one sitting (base 19.28-20.65, cow 19.02-20.68). Golden traces identical. The negative result is the useful part: per-clone allocation traffic is *not* in these collections, which is what sent the run to the caller tree and found the layer-compute path below. |
| 2026-08-08 | `dispatch_triggers_for_events` stops running `compute_battlefield()` for one bool per card; `permanents_with_abilities_removed` answers from the gathered effect set and only pays the layer pass when a `RemoveAllAbilities` effect is actually in scope (`c365ede8`) | 19.76 games/s, 11950 dec/s | 21.38 games/s, 12909 dec/s | **+8.2 %.** `--bench` x6 per side alternated A/B in one sitting; every B run beat every A run. Sealed pool (720 games) 32.15 s -> 31.30 s, +2.7 %. turns/game 26.98 unchanged, stalls 0, all pairs split. Suite 18788 passed / 0 failed; all four golden traces byte-identical. |
| 2026-08-08 | Hoist the trigger dispatcher's two grant scans (`statics_granted_triggers_for`, `equip_granted_triggers_for`) out of its per-permanent loop into board-level source lists (`f87974c3`) | 22.14 games/s | 23.00 games/s | **+3.9 %.** `--bench` x5 per side alternated. Under the 5 % claim bar, but the distributions don't overlap — slowest after (22.54) > fastest before (22.41), 5/5 pairs. The by-card methods stay as shims over the same pair, so there is still one walker. Suite 18788 passed / 0 failed; golden traces byte-identical. |
| | **cumulative this run** | **19.76 games/s** | **+12.4 %** | measured as two alternated A/B sittings (+8.2 %, +3.9 %); the box drifted upward between them, so the absolutes on either side of the gap don't subtract |

Measured on a **second, concurrent box** (Xeon @ 2.80GHz, `host_calib_ms`
47-62) against base `3f1ddaac`, i.e. *without* the two dispatcher rows
above. Absolutes therefore don't line up with the block above; the
percentages are what carry over.

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-08 | `cast_candidates` stops calling `compute_hand_affordances` for its one `.spliceable` field; it asks `spliceable_hand_cards_on` against the probe template it already built (`489bb1d3`) | 16.08 games/s, 9709 dec/s | 22.83 games/s, 13786 dec/s | **+42.0 %.** `--bench` ×3 per side alternated A/B/A/B in one sitting on an idle box (15.98/16.07/16.18 → 23.00/22.38/23.10); every B run beat every A run by >6 games/s, so the gap is 20× the spread. Sealed pool (720 games), the shape self-play trains on: 40.2/40.0 s → 26.3/26.2 s, **+52.7 %**, 0 undecided both sides. turns/game 26.98 unchanged, stalls 0, RSS unchanged, all pairs split. Suite 18623 passed / 0 failed; all four golden traces byte-identical. |
| 2026-08-08 | Hoist `available_mana` out of `cast_candidates`' per-hand-card `can_afford_in_state` filter (one `AvailableMana` per call instead of one per card, each walking the board and allocating a `Vec` per untapped permanent via `granted_abilities_for`) | 22.83 games/s | 23.03 games/s | **No win — reverted.** `--bench` ×3 per side alternated on an idle box (22.82/22.53/23.13 → 23.04/23.18/22.88); +0.9 %, and the distributions overlap (the best A run beats the worst B run). Measured against the same base as the row above by rebuilding that exact commit with the patch applied in a worktree. The asymptotics were real but the constant is not: the bench hand is ~7 cards over a 3-6 permanent board, and the sweep that *was* multiplying this is gone as of the row above. Left in **Perf candidates** as a closed sub-item so it doesn't get re-derived. |
| 2026-08-08 | Card-name tallies (`spells_cast_by_name_this_game`, `spell_names_cast_this_turn`, `cycled_count_by_name`) key on `&'static str` instead of `String` (`eb5f661c`) | — | — | **No separate win claimed.** It rode in the sitting above, and it is exactly the class the CoW row measured at 0.1 %: `GameState::clone` is 0.9 M of ~16.7 M allocations. Kept as a typing change — the keys are `CardDefinition::name`, already `&'static str`, and the owned copies were a widening that allocated per entry per clone. Golden traces byte-identical. |
| 2026-08-08 | Freeze the layer scope around the attack/block sims' read-only helpers — `pick_attacks`, `sim_spell_action`, `decide_pending_policy`, `eval_material` (`836059e2`) | 21.94 games/s, 13248 dec/s | 23.03 games/s, 13904 dec/s (**+5.0 %**) | `release-fast` A/B, 12 alternated pairs in one sitting; median paired +5.7 %, 11/12 rounds positive (the one loss read `host_calib_ms` 66 vs 52 on its pair). `turns_per_game` 26.98 and `stalls` 0 on every run, both sides; determinism ok |
| 2026-08-08 | Per-freeze-scope memo of `computed_permanent`: `LayerFreezeState` grows a `perms: Vec<(CardId, Arc<ComputedPermanent>)>` cleared where `memo` is, and the method hands back the `Arc` (candidate 1(b)) | 13,307,099,945 Ir | 13,052,911,075 Ir (**-1.91 %**) | **Small win, kept, and wall-clock cannot see it.** Measured by callgrind instruction count, A/B on identical `release-fast` binaries over `--a gang --b gang --games 6 --threads 1 --seed 1 --decks fixed` — deterministic, so the 1.91 % is exact rather than a mean. `--bench` wall-clock over 8 alternated pairs read +0.7 % mean / +1.1 % median with 5/8 pairs positive, i.e. inside this box's noise; that is *why* the row quotes Ir. An instrumented build put the memo's hit rate at **3.30 M hits / 9.57 M calls (34.5 %)** — the ceiling is 50 %, because half of all `computed_permanent` calls come from `depth == 0` mutating paths (combat damage, SBAs, effect resolution) where freezing is unsound. Reducing the per-call lock traffic from 3 acquisitions to 1-2 was worth only 0.12 pts of the 1.91, so the lock was not the cost. Suite 18806 passed / 0 failed; all golden traces byte-identical; `turns_per_game` 26.98 and `stalls` 0 on all 16 bench runs; determinism ok. |
| 2026-08-08 | Freeze the layer scope around the three read-only mana walkers — `mana_source_table`, `untapped_relevant_source_exists`, `untapped_producers_of` | 13,052,911,075 Ir; 27.43 games/s | 12,235,211,102 Ir (**-6.26 %**); 28.67 games/s (**+4.5 %**) | Same lesson as the sim-loop freeze, one level down. `auto_tap_for_cost_inner` (17.47 % inclusive) is `&mut self`, so the table it builds ran at `depth == 0`: every untapped permanent's `effective_mana_abilities` called `printed_land_mana_ability_lost` *per printed mana ability* and `intrinsic_land_mana_abilities` once, each of which is a full `gather_continuous_effects` + layer pass. A five-land board paid ~10 gathers per auto-tap. The three walkers are all `&self` and pure reads, so one scope covers each. Callgrind A/B as above, plus `--bench` ×6 alternated pairs: **6/6 pairs positive**, median paired +1.17 games/s. Compounds with the memo row above — the freeze is what lets it hit. Suite 18806 passed / 0 failed; golden traces byte-identical; turns_per_game 26.98, stalls 0, determinism ok on all 12 runs. |

## Profile of record

Callgrind, `--profile profiling --no-default-features` (system allocator —
valgrind replaces malloc, so a mimalloc build would measure the
interception), 1 thread, `--a gang --b gang --games 6 --seed 1 --decks
fixed`. Retaken 2026-08-08 at `fae58aa7`, i.e. *after* the dispatcher pair,
the affordance sweep and the sim-loop freeze, all of which the previous
profile predates.

**13.84 G instructions for six games, down from 23.50 G** — the same
workload, so the three landed fixes are worth **-41 %** of the simulator's
instructions between the two profiles.

Self cost, grouped (previous profile's share in brackets):

| share | site | note |
|---|---|---|
| 20.3 % | `_int_malloc` 6.56 / `_int_free` 4.94 / `malloc` 3.49 / `free` 2.01 / `malloc_consolidate` 1.58 / `unlink_chunk` 0.90 / arena `free` 0.79 | [24.0 %] still the single biggest cost |
| 15.9 % | `gather_continuous_effects_inner` (6.27 slice-iter + 4.14 non_null + 1.56 mod.rs + 1.52 vec + 1.14 option + 0.79 raw_vec + 0.53 mut_ptr) | [16.35 %] |
| 11.0 % | `CardInstance::clone` (7.96 + 2.00 option + 1.03 raw_vec) | [5.32 %] **doubled its share — now the #1 engine function** |
| 8.71 % | `__memcpy_avx_unaligned_erms` | [8.78 %] |
| 2.40 % | `compute_permanent_pass` | [1.91 %] tiny self cost, big allocator cost |
| 2.20 % | `drop_in_place<CardInstance>` | |
| 1.66 % | `granted_abilities_for` | |
| 0.78 % | `Keyword::eq` | [0.74 %] |

Inclusive:

| share | site |
|---|---|
| 86.32 % | `RandomBot::next_action` |
| 66.27 % | `perform_action_inner` |
| 63.53 % | `perform_action` (the checkpoint-cloning wrapper) |
| 58.16 % | `pick_attacks_scored` → 58.08 % `simulate_attack_outcome_once` |
| 38.79 % | `pass_priority` → 31.79 % `advance_step` |
| 21.34 % | `cast_spell` |
| 21.31 % | `Arc::make_mut` — the `CowBox` unshare, 20.91 % of it in `clone/uninit.rs` |
| 21.12 % | `gather_continuous_effects_inner` |
| 18.45 % | `computed_permanent` (**memoized this run — see the Log**) |
| 17.47 % | `auto_tap_for_cost_inner` |
| 11.58 % | `compute_battlefield` |
| 9.67 % | `would_accept_on` |
| 9.53 % | `mana_source_table`, of which **8.23 % is `Vec::clone`** |
| 9.10 % | `effective_mana_abilities` |
| 6.58 % | `drop_in_place<GameState>` / 5.28 % `GameState::clone` |
| 5.07 % | `intrinsic_land_mana_abilities` |

**Who actually allocates** (`--tree=caller` on `malloc`):

| calls | caller |
|---|---|
| 2,254,014 | `Vec::clone` |
| 2,588,924 | `compute_permanent_pass` (1,537,206 + 1,051,718 across two sites) |
| 1,537,206 | `Subtypes::clone` (all of it inside the above) |
| 1,036,628 | `iter::flatten::and_then_or_clear` |
| 888,340 | `apply_layers` |
| 868,914 | `HashMap::clone` |
| 796,844 | `CardInstance::clone` |
| 648,866 | `computed_permanent` |
| 367,900 | `gather_continuous_effects_inner` |
| 331,628 | `ActivatedAbility::clone` |
| 241,290 + 160,860 | `GameState::clone` |

`compute_permanent` (the layer pass for one card) is entered **888,340×
from `apply_layers`** and **648,866× from `computed_permanent`**, at ~970
instructions each. The layer system is still where the allocator time is,
and the split says a per-scope `compute_battlefield` memo (candidate 1(c))
is the larger of the two remaining halves.

**What the re-profile changed about the read:** the allocator share fell
24.0 → 20.3 % but `CardInstance::clone` doubled to 11 %, so the *checkpoint
clone* — not the layer system — is now the fastest-growing cost, and
`Arc::make_mut` at 21.31 % inclusive says most of it is `CowBox` zones
unsharing. That is candidate 7 and it has moved up.

## Perf candidates

Ordered by expected value. Each run pulls the top one, attaches numbers,
and feeds what it finds back in. Re-profile and replenish when the list
goes thin or stale.

**The re-profile is done** (2026-08-08, `fae58aa7`) — see "Profile of
record". Every share below is from it. Two methodological notes for whoever
comes next, both learned the hard way this run:

- **This box cannot resolve a sub-5 % change by wall-clock.** Eight
  alternated `--bench` pairs of a change worth exactly -1.91 % instructions
  read +0.7 % mean with 5/8 pairs positive. Use `callgrind` instruction
  counts for anything you expect to land under ~5 %: same binaries, same
  fixed workload, deterministic to the instruction. Wall-clock stays the
  arbiter for anything allocator- or cache-shaped, where Ir lies.
- **An inclusive share is an upper bound on a memo, not an estimate.**
  `computed_permanent` was 18.45 % inclusive; memoizing it per freeze scope
  hit 34.5 % of calls and returned 1.91 %. Half of its calls come from
  `depth == 0` mutating paths that cannot be frozen at all — check the
  frozen fraction before costing the next memo.

0. **Audit the bot's other whole-sweep calls the same way.** The
   `.spliceable` win was not an algorithmic insight — it was one caller
   asking for 40 answers and reading 1. `cast_candidates` is clean now, but
   nothing structural stops the next one:
   (a) ~~`available_mana` per hand card~~ — **tried, no win, reverted.** See
   the Log row; the shape was right but the constant is tiny once the
   affordance sweep is gone. Don't redo it without a board-size argument.
   (b) Grep discipline: a bot-path call to any `compute_*` / `*_hand_cards`
   aggregate that reads one field is the smell. `view.rs` is the only
   legitimate caller of `compute_hand_affordances` — it genuinely needs all
   40 categories for the client.
   (c) ~~The sims' read-only helpers ran unfrozen~~ — **done, +5.0 %.** See
   the Log row for `836059e2`. The generalisable rule it leaves behind:
   **a freeze scope stops at a clone.** `LayerFreeze` clones as unfrozen on
   purpose, so any helper called on a *cloned* state re-gathers per
   `computed_permanent` even when its caller was frozen. Grep for
   `&GameState` helpers reached from `simulate_*` / `sim_*` before assuming
   the tick-level scope covers them.
   (d) **Combat's whole-board reads** — `combat.rs` has 12
   `compute_battlefield()` calls. `has_first_strikers` (2315) computes every
   permanent to read the keywords of the 2-4 combat participants; the
   banding pair (42, 304) is the same shape on a rarer path. The
   first-strike one is called once per combat from `stack.rs:243`, so cost
   it before spending on it. 404/418 and 2333/2354 genuinely want the whole
   board — leave them.
   (e) **The mana walkers ran unfrozen** — **done, +4.5 %.** See the Log row.
   The shape it leaves: a `&mut self` entry point (`auto_tap_for_cost_inner`)
   means everything under it runs at `depth == 0`, however read-only. Look
   for the `&self` sub-walkers *inside* a mutating path, not for mutating
   paths to freeze.
   (f) **Where the remaining unfrozen layer traffic is.** At the tip the
   gather still runs 260,370× from `computed_permanent`, 52,332× from
   `dispatch_triggers_for_events` and 46,090× from `compute_battlefield`.
   The `computed_permanent` callers, by inclusive share:
   `printed_land_mana_ability_lost` 2.04 % (72,436×), `activate_ability_inner`
   2.01 % (36,772×), `scale_damage_to` 1.46 % (21,936×),
   `intrinsic_land_mana_abilities` 1.27 % (85,854×), `activate_ability`
   1.00 % (18,386×), `evaluate_requirement_static` 0.90 % (93,612×),
   `damage_prevented_by_protection` 0.78 % (91,762×). The last two are the
   interesting pair: both are `&self`, both are called in tight loops from
   combat-damage resolution, and `damage_prevented_by_protection` already
   opens its *own* one-call scope — which memoizes nothing, because the
   scope dies with the call.
   **Do not freeze across `resolve_combat_damage_with_filter`'s apply loop
   without deciding the rules question first.** Read the code before
   trying: it already hoists a whole-board `computed: &[ComputedPermanent]`
   taken *before* the batch and reads attacker/blocker P/T from it, which is
   CR 510.2 (combat damage is simultaneous). But `scale_damage_to` and
   `damage_prevented_by_protection` are called *inside* the apply loop and
   re-derive live, and the loop mutates a layer input: Wither/Infect damage
   adds -1/-1 counters, which `compute_permanent_pass` reads. A freeze
   scope would silently switch those two helpers from live to pre-batch
   values. Pre-batch is probably the *more* correct reading, but that makes
   it a rules change wearing an optimization's clothes — land it as a rules
   fix with its own tests and a blessed trace, or not at all. The safe
   subset is phase 1 (`gather_combat_damage_decisions`), which runs before
   any damage is dealt; cost it first, it may be too small to matter.

1. **Make `ComputedPermanent` cheap to build.** ~2.59 M of the profile's
   mallocs are `compute_permanent_pass` cloning five collections per
   permanent, most byte-identical to the (immutable, `Arc`-shared)
   `CardDefinition` they came from.
   (a) Hold `Arc<CardDefinition>` plus `Option<Vec<…>>` overrides and read
   through accessors — `None` means "the printed value", so the common
   unmodified permanent allocates nothing. **This is now the top item**:
   it is the only shape that helps the ~50 % of `computed_permanent` calls
   and the 888,340 `apply_layers` entries that no memo can reach, because
   they run on mutating paths. Big: ~4.5 k call sites read `ComputedPermanent`,
   though nearly all read `.power` / `.toughness`; the five collection
   fields are the ones that need accessors.
   (b) ~~Memoize per (card, freeze scope)~~ — **done, -1.91 % Ir.** See the
   Log row. Hit rate 34.5 %, ceiling 50 %.
   (c) ~~The same memo for `compute_battlefield`~~ — **dead, don't build
   it.** An instrumented `--bench` run counted **617,032
   `compute_battlefield` calls of which 0 are inside a freeze scope**, at
   19.51 permanents each. A per-scope memo would never hit. (This is the
   frozen-fraction check from the method note, run *before* the work
   instead of after — total cost one 2m18s build and one bench run.)
   What the same run does say is that `compute_battlefield` is the
   **bigger** consumer of the per-card layer pass: ~12.0 M per-card
   computations against `computed_permanent`'s 9.57 M calls, and all
   12.0 M of them unfrozen. Its callers, by inclusive share:
   `declare_attackers_banded` 2.45 % (9,928×),
   `check_state_based_actions` 2.41 % (10,670×), `declare_blockers` 2.00 %
   (7,754×), `finalize_cast` 1.51 % (7,046×), `resolve_combat` 0.86 %,
   `advance_step` 0.67 %, `process_cumulative_upkeep` 0.40 %, `do_phasing`
   0.40 %, `bands_with_other_qualities` 0.33 %. Three routes, in
   increasing size: put those callers inside a scope where they're
   read-only (`check_state_based_actions` and `declare_blockers` look
   like item 0's shape); ask item 0's question of each — does it really
   need all 19.5 permanents, or two of them; or make the per-card pass
   itself cheap, which is 1(a) and helps all 12.0 M unconditionally.
   **1(a) is the item this measurement promotes.**

1.5 **`effective_mana_abilities` clones every ability it returns** — still
   open after the freeze row above, which fixed the *layer* half of this
   cluster but not the *allocation* half. **Do not re-try freezing it**: a
   `with_frozen_layers` scope *inside* `effective_mana_abilities` measured
   12,235,211,102 -> 12,239,293,155 Ir (**+0.03 %, a null**) and was
   reverted. Its callers already hold a scope, so the nested freeze buys
   nothing; within one `mana_source_table` scope each land's
   `printed_land_mana_ability_lost` misses the per-card memo once and
   `intrinsic_land_mana_abilities` then hits it. What is left is the
   allocation (`mana_source_table` 9.53 % inclusive, of which **8.23 % is
   `Vec::clone`**;
   `effective_mana_abilities` 9.10 %; `ActivatedAbility::clone` 331,628
   mallocs). Every untapped permanent's printed mana abilities are deep-cloned
   on every `auto_tap_for_cost_inner` (17.47 % inclusive) call, and the two
   consumers — `mana_source_table` and `untapped_mana_colors` — only ever
   *read* `.effect`. The printed ones can borrow from `card.definition`;
   only `granted_abilities_for` and `intrinsic_land_mana_abilities` (5.07 %)
   synthesize, so `Vec<(usize, Cow<'_, ActivatedAbility>)>` covers it. Small,
   local, and the numbers are already attached — a good next pull.

2. **The dispatcher's per-card trigger gathering.** With the
   `compute_battlefield` call gone, the remaining per-card work in
   `dispatch_triggers_for_events` is `statics_granted_triggers_for` (2.78 %
   self before the fix) plus `granted_triggers` and
   `equip_granted_triggers_for`, each called once per battlefield card per
   dispatch — O(cards²) against `all_static_sources`. Hoist the
   "which sources carry a `GrantTriggeredAbility`" scan out of the per-card
   loop, the same shape as the layer-gather filter that won +27.8 %.
3. **The gather is still the #1 engine function by self cost** (15.9 %,
   21.12 % inclusive — barely moved by any of the three landed fixes).
   Shape confirmed by reading: each of the 39 passes is
   `for &card in &sa_cards { let StaticEffect::X { .. } = sa.effect else { continue }; … }`,
   so a presence summary needs a `StaticEffect` discriminant tag in
   `crabomination_base` — that tag is the whole cost of this item.
   The filter landed last run cut the *number of cards* each of the 39
   static-ability passes walks; it did not cut the 39 walks. A pairs list
   (`Vec<(&CardInstance, &StaticAbility)>`) keeps push order and so keeps
   the traces, but it does not change the asymptotics either — the real win
   is skipping a pass whose `StaticEffect` variant is absent from the board
   entirely. That needs a per-variant presence summary built in one pass;
   assign the tags in one place next to the passes so the two can't drift.
4. **Memoize the gather outside freeze scopes.** Unchanged: the blocker is
   invalidation, not caching. `compute_battlefield` alone runs 46,090 times
   per six games and `gather_continuous_effects_inner` allocates 367,900
   times. Two designs: a mutation epoch
   bumped at every `&mut GameState` entry point with the field set made
   private behind accessors; or route zone mutation through `CowBox`'s
   `DerefMut` and derive validity from `Arc` identity. Multi-run project.
5. **`Keyword::eq` (0.78 % self)** — linear scans of `Vec<Keyword>`. A
   bitset for the ~64 common keywords makes `has_keyword` O(1) and shrinks
   `CardInstance`; rides along with item 1.
6. **`HashMap` hash choice** — `block_map`, `combat_damage_order` /
   `_assignment` use SipHash and show up at 868,914 `malloc` calls via
   `HashMap::clone`.
7. **CowBox sharp edge audit — promoted by the re-profile.**
   `Arc::make_mut` is **21.31 % inclusive** and `CardInstance::clone` has
   doubled its self share to 11 %, so the checkpoint clone is now the
   fastest-growing cost in the simulator. Any `&mut` access — including a
   read-only `iter_mut` — deep-copies the zone while a snapshot shares it.
   79 `battlefield.iter_mut()` sites; start by finding which of them are
   read-only. This is also the prerequisite for
   per-card CoW (`Vec<CowBox<CardInstance>>`), which would make a zone
   unshare one pointer memcpy plus one card clone instead of N card clones.
8. **`legal_block_targets` per-pair requirement evaluation.** Still does not
   appear in the profile — it is a view-layer path, not a bot path.
9. **Actor scaling — re-measure on real cores.** Four-core boxes make
   everything past 4 actors oversubscription.
10. **Effect-resolution recursion depth.** The 32 MB worker stacks
    (`RUST_MIN_STACK` in `.cargo/config.toml`, plus explicit `stack_size` on
    every worker) are still required. A robustness constraint rather than a
    throughput cost; it lives here so nobody "cleans up" the stack sizes.

**Closed / ruled out:**

- *State-clone allocation traffic* (was candidate 1). CoW-wrapping the
  per-turn tally collections moved the bench 0.1 % over 8 alternated pairs.
  The caller tree explains why: `GameState::clone` is 0.9 M of ~16.7 M
  allocations. Shrinking `CardInstance` is still theoretically worth
  something for the memcpy share, but it is not where the allocator time is.
- *The bot's affordance sweep* — `cast_candidates` calling
  `compute_hand_affordances` for one field. Fixed (+42 % fixed decks,
  +52.7 % sealed). The lesson generalises and is item 0 above.
