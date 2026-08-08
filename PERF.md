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

Committed 2026-08-08 at the branch tip: this run's dispatcher fix + grant
hoist rebased onto another session's `Effect` boxing (`TokenDefinition` out
of line, 1464 -> 448 bytes) and its catalog wave. Refresh only alongside an
intentional, explained change. Regressions beyond ~5 % get investigated
before anything else lands — but check `host_calib_ms` first (see "How to
measure").

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz
host_calib_ms        57 / 70 / 65 / 70           <- compare this first
games                320
games_per_s          23.94 / 23.81 / 24.17 / 24.44   (mean 24.09, spread 2.6 %)
games_per_s_th       7.98 / 7.94 / 8.06 / 8.15
decisions_per_s      14454 / 14375 / 14592 / 14758   (mean 14545)
turns_per_game       26.98
decisions_per_game   603.9
stalls               0 (0.00 %)
peak_rss_mib         38.3 - 41.5
determinism          ok (160 pairs, 0 sweeps, rho -1.000)
```

This run's own binary (before the rebase) read 22.71 on the same box; the
tip reads 24.09 and peak RSS fell 40.7-44.4 -> 38.3-41.5 MiB, both
consistent with the `Effect` boxing that arrived with the rebase. That
delta was **not** measured A/B by this run — it is a baseline observation,
not a claim.

**This is a different box from the previous baseline and the absolutes do
not compare.** The 2026-08-08 baseline read 14.49 games/s on an Intel Xeon
@ 2.80GHz with `host_calib_ms` 54-60; this box is a Xeon @ 2.10GHz with
`host_calib_ms` 55-107 and the *unchanged* HEAD read **19.76** on it. The
probe is not a linear correction — a slower probe here goes with faster
games — so treat it as a box fingerprint, not a scaling factor, and only
ever compare two binaries measured in one alternating sitting.

Within this run, each step measured as its own alternated A/B sitting:

```text
19.76 -> 21.38   trigger dispatcher stops computing the whole board  +8.2 %
22.14 -> 23.00   grant scans hoisted out of the dispatcher's loop    +3.9 %
```

The two sittings are hours apart and the box drifted upward between them
(the same binary read 21.38 then 22.14), so **the absolutes across the gap
do not subtract** — the compounded figure is +12.4 %, not 23.00/19.76.

The actor-scaling sweep from the previous run (system allocator vs mimalloc
at 1/4/8/16/24 threads) is untouched by this run's work; its readable
conclusion was that mimalloc's edge grows with actor count (+9 % at 1,
+31 % at 24) while its RSS ratio stays flat at 1.5-1.7x. Re-measure on a
box with real cores.

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
| 2026-08-08 | Card-name tallies (`spells_cast_by_name_this_game`, `spell_names_cast_this_turn`, `cycled_count_by_name`) key on `&'static str` instead of `String` (`eb5f661c`) | — | — | **No separate win claimed.** It rode in the sitting above, and it is exactly the class the CoW row measured at 0.1 %: `GameState::clone` is 0.9 M of ~16.7 M allocations. Kept as a typing change — the keys are `CardDefinition::name`, already `&'static str`, and the owned copies were a widening that allocated per entry per clone. Golden traces byte-identical. |

## Profile of record

Callgrind, `--profile profiling --no-default-features` (system allocator —
valgrind replaces malloc, so a mimalloc build would measure the
interception), 1 thread, `--a gang --b gang --games 6 --seed 1 --decks
fixed`. Taken 2026-08-08 at `1d824fe5`, i.e. *before* this run's fix.

**23.50 G instructions for six games** (reproduced the previous run's 23.49 G
exactly, so the workload is stable across boxes).

Self cost, grouped:

| share | site | note |
|---|---|---|
| 24.0 % | `_int_malloc` 8.26 / `_int_free` 5.58 / `malloc` 3.91 / `free` 2.20 / `malloc_consolidate` 2.09 / `unlink_chunk` 1.15 / arena `free` 0.86 | the allocator is the single biggest cost |
| 8.78 % | `__memcpy_avx_unaligned_erms` | |
| 16.35 % | `gather_continuous_effects_inner` (6.64 slice-iter + 4.40 non_null + 1.65 mod.rs + 1.60 vec + 1.22 option + 0.84 raw_vec) | |
| 5.32 % | `CardInstance::clone` | |
| 1.91 % | `compute_permanent_pass` | tiny self cost, huge allocator cost — see below |

Inclusive:

| share | site |
|---|---|
| 89.96 % | `RandomBot::next_action` |
| 75.97 % | `perform_action_inner` |
| 59.59 % | `pick_attacks_scored` |
| 46.64 % | `perform_action` (the checkpoint-cloning wrapper) |
| 24.60 % | `computed_permanent` |
| 22.17 % | `gather_continuous_effects_inner` |
| 15.38 % | `apply_layers`, all of it under `compute_battlefield` |
| 10.39 % | `dispatch_triggers_for_events -> compute_battlefield` (**fixed this run**) |

**Who actually allocates** (`--tree=caller` on `malloc`, ~16.7 M calls for
six games; 9.43 M of them reach `_int_malloc`, the rest are tcache hits):

| calls | caller |
|---|---|
| 5,802,510 | `layers::compute_permanent_pass` |
| 3,279,608 | `Subtypes::clone` (almost all of it inside the above) |
| 1,631,596 | `HashMap::clone` |
| 1,540,297 | `RawVec::finish_grow` |
| 1,387,916 | `gather_continuous_effects_inner` |
| 1,202,386 | `Vec::clone` |
| 901,798 | `GameState::clone` |
| 860,938 | `CardInstance::clone` |
| 123,616 | `apply_layers` |

**The headline correction to the previous run's read: the 24 % allocator
share is the layer system, not state cloning.** `compute_permanent_pass`
clones five collections per permanent (`card_types`, `supertypes`,
`subtypes`, `colors`, `keywords`) and is called 2.28 M times from
`apply_layers` plus 1.00 M times from `computed_permanent` — together ~55 %
of every allocation in the program. `GameState::clone` is 5 % of them. The
CoW experiment in the **Log** above is the direct evidence: removing the
per-clone collection copies moved the benchmark 0.1 %.

## Perf candidates

Ordered by expected value. Each run pulls the top one, attaches numbers,
and feeds what it finds back in. Re-profile and replenish when the list
goes thin or stale. **Re-profile first** — the list below is derived from a
profile taken *before* this run's fix removed 10.4 % of instructions, so the
shares have all moved. The `.spliceable` fix removed a further ~30 % of
wall-clock on top of that, and it sat *above* the layer system in the call
tree, so the profile owes a re-take before item 1 is costed.

0. **Audit the bot's other whole-sweep calls the same way.** The
   `.spliceable` win was not an algorithmic insight — it was one caller
   asking for 40 answers and reading 1. `cast_candidates` is clean now, but
   nothing structural stops the next one:
   (a) `available_mana(state, seat)` is recomputed **per hand card** inside
   `cast_candidates`' `can_afford_in_state` filter, and it calls
   `granted_abilities_for` (which allocates a `Vec` and walks
   `all_static_sources`) once per untapped permanent — O(hand × board²) with
   allocations, all of it invariant across the filter. Hoist it to one
   `AvailableMana` per call and pass it down; `can_afford_in_state` stays as
   a shim that computes its own. Provably identical (`state` is `&`-borrowed
   for the whole filter). Unmeasured but cheap to do.
   (b) Grep discipline: a bot-path call to any `compute_*` / `*_hand_cards`
   aggregate that reads one field is the smell. `view.rs` is the only
   legitimate caller of `compute_hand_affordances` — it genuinely needs all
   40 categories for the client.

1. **Make `ComputedPermanent` cheap to build.** ~55 % of all allocations
   come from `compute_permanent_pass` cloning five collections per
   permanent, most of which are byte-identical to the (immutable,
   `Arc`-shared) `CardDefinition` they came from. Two shapes:
   (a) hold `Arc<CardDefinition>` plus `Option<Vec<…>>` overrides and read
   through accessors — `None` means "the printed value", so the common
   unmodified permanent allocates nothing. ~4.5 k `computed_permanent` call
   sites exist but nearly all read `.power` / `.toughness`; the five
   collection fields are the ones that need accessors.
   (b) memoize per (card, freeze scope): `LayerFreezeState` already caches
   the gathered effect set for the whole bot tick, and a parallel
   `Vec<(CardId, Arc<ComputedPermanent>)>` is sound by the same argument
   (`with_frozen_layers` only hands out `&GameState`). Needs a
   `computed_permanent_shared` returning `Arc<…>` so the memo isn't cloned
   back out; the existing by-value method stays as the compat shim.
   Do (b) first — it is smaller and its win is measurable on its own.
2. **The dispatcher's per-card trigger gathering.** With the
   `compute_battlefield` call gone, the remaining per-card work in
   `dispatch_triggers_for_events` is `statics_granted_triggers_for` (2.78 %
   self before the fix) plus `granted_triggers` and
   `equip_granted_triggers_for`, each called once per battlefield card per
   dispatch — O(cards²) against `all_static_sources`. Hoist the
   "which sources carry a `GrantTriggeredAbility`" scan out of the per-card
   loop, the same shape as the layer-gather filter that won +27.8 %.
3. **The gather is still #1 by self cost** (16.35 %, 22.17 % inclusive).
   The filter landed last run cut the *number of cards* each of the 39
   static-ability passes walks; it did not cut the 39 walks. A pairs list
   (`Vec<(&CardInstance, &StaticAbility)>`) keeps push order and so keeps
   the traces, but it does not change the asymptotics either — the real win
   is skipping a pass whose `StaticEffect` variant is absent from the board
   entirely. That needs a per-variant presence summary built in one pass;
   assign the tags in one place next to the passes so the two can't drift.
4. **Memoize the gather outside freeze scopes.** Unchanged from last run:
   the blocker is invalidation, not caching. `compute_battlefield` alone
   re-gathers 123,712 times per six games. Two designs: a mutation epoch
   bumped at every `&mut GameState` entry point with the field set made
   private behind accessors; or route zone mutation through `CowBox`'s
   `DerefMut` and derive validity from `Arc` identity. Multi-run project.
5. **`Keyword::eq` (0.74 % self)** — linear scans of `Vec<Keyword>`. A
   bitset for the ~64 common keywords makes `has_keyword` O(1) and shrinks
   `CardInstance`; rides along with item 1.
6. **`HashMap` hash choice** — `block_map`, `combat_damage_order` /
   `_assignment` use SipHash and show up at 1.63 M `malloc` calls via
   `HashMap::clone`.
7. **CowBox sharp edge audit.** Any `&mut` access — including a read-only
   `iter_mut` — deep-copies the zone while a snapshot shares it. 79
   `battlefield.iter_mut()` sites. This is also the prerequisite for
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
