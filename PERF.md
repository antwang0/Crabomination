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

Callgrind, `--profile profiling --no-default-features` (system allocator —
valgrind replaces malloc, so a mimalloc build would measure the interception),
1 thread, `--a gang --b gang --games 6 --seed 1 --decks fixed`. Taken
2026-08-08 at `67d6c549`, i.e. *after* this run's two wins.

**23.49 G instructions, down from 38.5 G for the same six games (-39 %).**

Self cost, grouped:

| share | site | note |
|---|---|---|
| 24.0 % | `_int_malloc` 8.25 / `_int_free` 5.58 / `malloc` 3.91 / `free` 2.20 / `malloc_consolidate` 2.09 / `unlink_chunk` 1.14 / arena `free` 0.86 | the allocator is now the single biggest cost |
| 8.80 % | `__memcpy_avx_unaligned_erms` | almost all of it is state/zone cloning |
| 16.35 % | `gather_continuous_effects_inner` (6.64 slice-iter + 4.40 non_null + 1.65 mod.rs + 1.60 vec + 1.22 option + 0.84 raw_vec) | was 38.15 % before the filter |
| 6.66 % | `CardInstance::clone` | 5.32 + 1.34 option |
| 2.78 % | `statics_granted_triggers_for` | |
| 1.95 % | `Vec<T>::clone` | |
| 1.91 % | `compute_permanent_pass` | |
| 1.12 % | `drop_in_place<CardInstance>` | the other half of the clone traffic |
| 0.82 % | `Subtypes::clone` | inside `CardInstance::clone` |
| 0.74 % | `Keyword::eq` | linear `Vec<Keyword>` scans |
| 0.73 % | `HashMap::clone` | |

Inclusive:

| share | site |
|---|---|
| 89.96 % | `RandomBot::next_action` |
| 75.97 % | `perform_action_inner` |
| 59.59 % | `pick_attacks_scored` |
| 59.54 % | `simulate_attack_outcome_once` |
| 46.64 % | `perform_action` (the checkpoint-cloning wrapper) |
| 30.30 % | `cast_spell` |
| 27.75 % | `would_accept_on` |
| 24.83 % | `compute_hand_affordances` |
| 24.60 % | `computed_permanent` (was 50.14 %) |
| 22.17 % | `gather_continuous_effects_inner` (was 52.52 %) |

The headline: **the layer system is no longer the bottleneck — cloning is.**
Allocator plus memcpy is a third of all instructions, and the traffic is state
and `CardInstance` copies for probes and rollback checkpoints.

## Perf candidates

Ordered by expected value. Each run pulls the top one, attaches numbers,
and feeds what it finds back in. Re-profile and replenish when the list
goes thin or stale.

1. **The clone / checkpoint path.** Now the top item by a wide margin: 24 %
   allocator + 8.8 % memcpy + 6.7 % `CardInstance::clone` + 1.1 %
   `drop_in_place` + 2.8 % of `Vec`/`Subtypes`/`HashMap` clones. Three angles,
   probably in this order:
   (a) `perform_action` clones the whole `GameState` for its rollback
   checkpoint on every accepted action, and the combat sims and
   `evaluate_action_sequence` already run inside a clone that is discarded —
   the rollback there is dead weight (`perform_action_inner` skips it, and is
   75.97 % inclusive against `perform_action`'s 46.64 %). Not
   behaviour-preserving on the `Err` path (partial mutation would survive into
   the rest of the simulated line), so it needs either a golden-trace update
   with a justification or a variant that restores only on error paths a sim
   can actually reach.
   (b) Shrink `CardInstance`. `Subtypes::clone` (0.82 %) and `HashMap::clone`
   (0.73 %) are per-card allocations on a struct that is copied by the
   million; the cold fields want to be behind one `Arc` or an `Option<Box<…>>`
   so the common instance clones as a memcpy with no heap traffic.
   (c) String keys: `Player.spells_cast_by_name_this_game: HashMap<String,
   u32>` and `GameState.cycled_count_by_name` are deep-cloned with every state
   clone, and card names are already `&'static str`. `Arc<str>` keeps serde
   working; interning to a `u32` id is better and bigger.
2. **Flatten the layer gather's static-ability passes.** Still 16.35 % self /
   22.17 % inclusive after this run's filter. The filter cut the *number of
   cards* each of the 39 passes walks; it did not cut the 39 walks. Build one
   `Vec<(&CardInstance, &StaticAbility)>` at the top of the gather and let each
   pass be a single linear scan over the pairs. Nesting order is the same, so
   push order is preserved — golden traces are the check. The 10 passes gated
   on other definition fields (`equipped_bonus`, `soulbond_bonus`,
   `dynamic_pt`, `level_bands`, `station`, `keywords`) want the same treatment
   with their own filtered slices.
3. **Memoize the layer gather outside freeze scopes.** Was the biggest item on
   this list at 52 % inclusive; it is 22.17 % now, so the prize shrank while
   the difficulty did not. The blocker is invalidation, not caching: the gather
   reads `continuous_effects`, `battlefield`, players' emblems/command/
   graveyards, `attacking`, `active_player_idx` and turn state, all `pub` and
   mutated from hundreds of sites. Two designs: (a) a mutation epoch bumped at
   every `&mut GameState` entry point, with the field set made private behind
   accessors so the bump can't be forgotten; (b) route zone mutation through
   `CowBox`'s `DerefMut` and derive validity from Arc identity plus a counter
   for the non-CoW inputs. Multi-run project — do item 1 first.
4. **`Keyword::eq` at 0.74 % self** — linear scans of `Vec<Keyword>`. A small
   bitset for the ~64 common keywords would make `has_keyword` O(1) and shrink
   `CardInstance` at the same time, so it rides along with item 1(b).
5. **CowBox sharp edge audit.** Any `&mut` access — including a read-only
   `iter_mut` — deep-copies the zone while a snapshot shares it. 79
   `battlefield.iter_mut()` sites; the ones that matter are those running while
   a probe clone is alive. Unmeasured, but item 1 will walk this code anyway.
6. **Hash choice for internal maps.** `HashMap::clone` is 0.73 % self now that
   the bigger costs are gone; `block_map`, `combat_damage_order` /
   `_assignment` and friends use SipHash. Ride-along with item 1.
7. **`legal_block_targets` per-pair requirement evaluation.** O(blockers x
   attackers) with a full requirement walk inside. Still does not appear in the
   profile — it is a view-layer path, not a bot path. Only worth doing if a
   wide board shows up in a measurement.
8. **Actor scaling — re-measure on real cores.** The sweep in **Baseline** ran
   on a 4-core box, so everything past 4 actors is oversubscription. What it
   does establish: mimalloc's edge grows with actor count and its RSS ratio
   stays flat at 1.5-1.7x.
9. **Effect-resolution recursion depth.** The 32 MB worker stacks
   (`RUST_MIN_STACK` in `.cargo/config.toml`, plus explicit `stack_size` on
   every worker) are still required. A robustness constraint rather than a
   throughput cost; it lives here so nobody "cleans up" the stack sizes.
