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

# allocator A/B (a feature change on the engine crate is a full rebuild, so
# the variants need separate caches; /target-mi/ is gitignored)
cargo build --release -p crabomination --bin bot_ladder
CARGO_TARGET_DIR=target-mi cargo build --release -p crabomination \
  --bin bot_ladder --features mimalloc

# instruction-level profile (deterministic; no `perf` in the routine image)
cargo build --profile profiling --bin bot_ladder
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

A self-mirror on a shared seed must report **every pair as a split**
(`rho -1.000`): the two games of a pair are the same game with the seats
relabelled. A sweep in a `--bench` run is a determinism bug, not variance.

Wall-clock notes for whoever iterates next: a release rebuild of the engine
is ~13 min on the 4-core routine box (`codegen-units = 1` + thin LTO), so
budget two or three measured iterations per run, not ten. Callgrind on six
games takes about three minutes and is contention-immune, which makes it
the better first look.

## Baseline

Committed 2026-08-08 at the tip of this run's work (post "Freeze the layer
memo for the whole bot tick", `e919496`, rebased onto the concurrent card
commits). Refresh only alongside an intentional, explained change.
Regressions beyond ~5 % get investigated before anything else lands.

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     measured on an idle box (load average < 0.5)
games                320
games_per_s          11.80 / 11.82 / 11.79 / 11.97   (mean 11.85, spread 1.5 %)
                     12.42 / 12.38 / 12.36 (mean 12.39) re-measured a run
                     later on the same idle box. The +4.6 % over 11.85 is
                     NOT a claimed win: the only engine change between them
                     moved three `rand::random` sites onto `GameRng`, and
                     those effects never fire in the bench decks. Read it as
                     day-to-day drift on a shared VM, and as the reason a
                     perf claim needs both sides measured in one sitting.
                     Compare against 12.39 (system alloc) or 13.88
                     (`--features mimalloc`), both measured that sitting.
games_per_s_th       3.93 / 3.94 / 3.93 / 3.99
decisions_per_s      7123 / 7139 / 7117 / 7231       (mean 7153)
turns_per_game       26.98
decisions_per_game   603.9
stalls               0 (0.00 %)
peak_rss_mib         25.1 – 25.5
determinism          ok (all pairs split)

thread scaling (--bench --threads N, games/s; measured pre-optimization,
so read the ratios, not the absolutes)
  1 → 4.23     2 → 8.05 (95 % of linear)     4 → 14.56 (86 % of linear)
```

**Measure on an idle box.** The optimization rows below were each taken as
three back-to-back runs right after a 13–17 min release build, which reads
2–3 % high and with a much wider spread (one such triple ran 10.91 / 11.52 /
11.98 — a 9.8 % spread) than the same binary gives once the machine settles.
The before/after *deltas* in the log are still sound because both sides were
measured the same way; the absolute baseline above deliberately is not, and
is the number to compare against.

## Log

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-08 | Gate the layer gather's three graveyard tallies on a `dynamic_pt` being present; drop the O(n²) battlefield-membership test in the `AnthemForFilter` walk (`b17a76b`) | 10.36 games/s, 6253 dec/s | 11.25 games/s, 6791 dec/s | `--bench` ×3 each side; golden traces byte-identical, turns/game unchanged |
| 2026-08-08 | Run the whole `RandomBot` tick inside one `with_frozen_layers` scope (`e919496`) | 11.25 games/s, 6791 dec/s | 12.22 games/s, 7381 dec/s | `--bench` ×3 each side; golden traces byte-identical, turns/game unchanged |
| | **cumulative this run** | **10.36 games/s** | **12.22 games/s (+18.0 %)** | both ends measured post-build; on the settled box the same code reads 11.85 |
| 2026-08-08 | Opt-in mimalloc `#[global_allocator]` on `bot_ladder` / `selfplay_train` (`--features mimalloc`) | 12.39 games/s, 7478 dec/s | 13.88 games/s, 8378 dec/s | `--bench` ×3 each side on an idle box (12.42/12.38/12.36 → 13.82/14.07/13.74); **+12.0 %**. Two separate release builds into `target/` and `target-mi/`, allocator the only difference. Peak RSS 25.3 → 39.0 MiB. turns/game 26.98 unchanged, stalls 0, all pairs still split. |

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
