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
  target/profiling/bot_ladder --a gang --b gang --games 6 --threads 1 --seed 1
callgrind_annotate --auto=no --threshold=95 cg.out            # self cost
callgrind_annotate --auto=no --inclusive=yes cg.out           # inclusive
callgrind_annotate --auto=no --tree=caller cg.out             # who calls whom

# behaviour preservation
cargo test -p crabomination_tests --test core_rules golden_trace

# build time
cargo build --timings -p crabomination
```

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

Committed 2026-08-08 at the tip of this run's work (`bbf5ddcc`, layer-gather
filter + mimalloc default). Refresh only alongside an intentional, explained
change. Regressions beyond ~5 % get investigated before anything else lands —
but check `host_calib_ms` first (see "How to measure").

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        54 / 56 / 55 / 60          <- compare this first
games                320
games_per_s          14.07 / 14.66 / 14.41 / 14.81   (mean 14.49, spread 5.1 %)
games_per_s_th       4.69 / 4.89 / 4.80 / 4.94
decisions_per_s      8497 / 8854 / 8699 / 8944       (mean 8749)
turns_per_game       26.98
decisions_per_game   603.9
stalls               0 (0.00 %)
peak_rss_mib         38.8 - 39.8
determinism          ok (160 pairs, 0 sweeps, rho -1.000)
```

The previous baseline read 11.85-12.39 games/s and **is not comparable**: the
same unchanged engine code read 9.64 on this box at the start of this run. The
box moved, not the engine — the three commits between the two measurements are
7 lines of RNG routing in a path the bench decks never take, a post-run assert,
an unused import and an opt-in Cargo feature. That episode is why
`host_calib_ms` exists. Within this run, on this box and this day:

```text
9.64  games/s   HEAD at the start of the run   (system alloc)
12.32           + layer-gather filter          (system alloc, +27.8 %)
14.49           + mimalloc default             (+50.3 % cumulative)
```

Actor scaling, `--bench --threads N --games 120` (a saturated queue — see the
`--threads` note above), system allocator vs mimalloc:

```text
threads   1      4      8      16     24
sys       4.54   15.33  16.71  15.76  13.83   games/s
mi        4.96   18.93  19.72  19.32  18.12
sys       16.1   31.2   49.2   83.3   115.5   peak RSS MiB
mi        24.2   49.5   82.5   134.2  178.7
```

Four cores, so past ~4 actors this measures oversubscription, not scaling. The
readable part is that mimalloc's edge *grows* with actor count (+9 % at 1,
+31 % at 24 — system malloc contends) while its RSS ratio stays flat at
1.5-1.7x. Re-measure the scaling half on a box with real cores.

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

## Profile of record

Callgrind, release, 1 thread, `--a gang --b gang --games 6 --seed 1
--decks fixed`, 38.5 G instructions total. Taken 2026-08-08 *before* the
log rows above, so the shares describe the code they were used to fix.

| share | site | note |
|---|---|---|
| 92.05 % incl | `RandomBot::next_action` | the bot tick is the simulator |
| 70.75 % incl | `perform_action_inner` | mostly inside combat sims |
| 57.94 % incl | `pick_attacks_scored` | the attack search dominates the tick |
| 57.75 % incl | `simulate_attack_outcome_once` | one turn cycle replayed per candidate attack set |
| 52.52 % incl / 38.15 % self | `gather_continuous_effects_inner` | ~1.02 M calls, ~14.4 k instructions each |
| 50.14 % incl | `computed_permanent` | 885 340 of those gathers, one permanent at a time |
| ~16 % | `malloc` / `free` / `memcpy` | allocator churn |
| 5.01 % self | `distinct_card_types_in_all_graveyards` | fixed in the log row above |
| 4.84 % self | `CardInstance::clone` | state clones for probes and checkpoints |
| 2.48 % self | `static_ability_to_effects` | per battlefield card, per gather |

The one number that explains the shape of everything else: the existing
`with_frozen_layers` memo served only **10 420** of those 1.02 M gathers
(~1 %). Every other `computed_permanent` call rebuilds the entire
continuous-effect set to answer a question about one permanent.

## Perf candidates

Ordered by expected value. Each run pulls the top one, attaches numbers,
and feeds what it finds back in. Re-profile and replenish when the list
goes thin or stale.

1. **Memoize the layer gather outside freeze scopes.** 52 % inclusive; the
   existing memo covers 1 % of calls. The blocker is invalidation, not
   caching: the gather reads `continuous_effects`, `battlefield`, players'
   emblems/command/graveyards, `attacking`, `active_player_idx` and turn
   state, and all of those are `pub` fields mutated from hundreds of sites,
   so a "sound by construction" argument like `with_frozen_layers`' isn't
   available. Two candidate designs: (a) a mutation epoch bumped at every
   `&mut GameState` entry point, with the field set made private behind
   accessors so the bump can't be forgotten; (b) route zone mutation
   through `CowBox`'s `DerefMut` and derive validity from Arc identity plus
   a counter for the non-CoW inputs. Either is a multi-run project. Biggest
   item on this list by a wide margin.
2. **Merge the ~50 separate `for card in &self.battlefield` passes inside
   `gather_continuous_effects_inner`.** ~14.4 k instructions per gather is
   almost entirely fifty linear scans, each doing one cheap field test and
   finding nothing — a typical bot board is vanilla creatures and basic
   lands. One pass with all the tests inlined, or a precomputed
   "participates in a stateful pass" flag per `CardDefinition`. Must
   preserve push order within a `(layer, sublayer, timestamp)` group;
   golden traces are the check.
3. **The checkpoint clone budget.** `perform_action` clones the whole
   `GameState` for its rollback checkpoint on every accepted action, and
   the combat sims and `evaluate_action_sequence` run inside a clone that
   is discarded anyway — the rollback there is dead weight.
   `perform_action_inner` skips it. Not behaviour-preserving on the `Err`
   path (partial mutation survives into the rest of the simulated line), so
   it needs a golden-trace update with a justification, or a variant that
   restores only on error paths a sim can actually hit.
4. **Allocator swap — done, opt-in, not yet default.** mimalloc behind
   `--features mimalloc` on `bot_ladder` and `selfplay_train` bought
   +12.0 % (see the log row) for +14 MiB peak RSS at 3 worker threads.
   Left opt-in rather than default because the RSS cost is per-thread-heap
   and nobody has measured it at 16+ actors — do that, and if it holds,
   make it the default for the ML binaries. jemalloc is unmeasured. Note
   the two variants need separate target dirs (a feature change on the
   engine crate forces a full 15–20 min rebuild); `/target-mi/` is
   gitignored for this.
5. **CowBox sharp edge audit.** Any `&mut` access — including a read-only
   `iter_mut` — deep-copies the zone while a snapshot shares it. 79
   `battlefield.iter_mut()` sites; the ones that matter are those running
   while a probe clone is alive. Unmeasured.
6. **String keys in cloned state.** `Player.spells_cast_by_name_this_game:
   HashMap<String, u32>` and `GameState.cycled_count_by_name` are cloned
   with every state clone, and card names are already `&'static str`.
   Small, but it is on the clone path that item 3 also touches.
7. **Hash choice for internal maps.** `block_map`,
   `combat_damage_order` / `_assignment` and friends use SipHash;
   `hash_one` + `reserve_rehash` measured 0.42 % combined, so this is a
   ride-along with item 4 rather than its own run.
8. **`legal_block_targets` per-pair requirement evaluation.** O(blockers ×
   attackers) with a full requirement walk inside. Did not appear in the
   profile — it is a view-layer path, not a bot path. Only worth doing if a
   wide board shows up in a measurement.
9. **Actor scaling — measured, healthy, no action.** 1/2/4 threads gave
   4.23 / 8.05 / 14.56 games/s (86 % of linear at 4 on a 4-core box that is
   also running the OS). Re-measure at 16+ actors before assuming it holds.
10. **Effect-resolution recursion depth.** The 32 MB worker stacks
    (`RUST_MIN_STACK` in `.cargo/config.toml`, plus explicit
    `stack_size` on every worker) are still required. This is a robustness
    constraint rather than a throughput cost; it belongs here so nobody
    "cleans up" the stack sizes.
