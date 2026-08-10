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
# measures the interception, not the program. `profiling-fast` is
# `release-fast` + debuginfo: same opt settings as the A/B binaries, so the
# attribution describes the code the Log rows move, and the engine rebuilds
# in ~3.5 min instead of ~24. (`profiling` inherits `release`; use it only
# if you need to attribute LTO'd code.)
cargo build --profile profiling-fast -p crabomination --bin bot_ladder \
  --no-default-features
# `-p crabomination` is load-bearing. Drop it and `--no-default-features`
# does not reach the engine crate: the binary keeps mimalloc, callgrind
# measures mimalloc-under-valgrind instead of the system allocator, and the
# total lands ~11 % low (4.41 G vs 4.96 G on the same tip, 2026-08-10) while
# looking like a win. Check the profile for `libmimalloc-sys` frames before
# trusting a total.
RUST_MIN_STACK=33554432 valgrind --tool=callgrind --callgrind-out-file=cg.out \
  target/profiling-fast/bot_ladder --a gang --b gang --games 6 --threads 1 \
  --seed 1 --decks fixed
callgrind_annotate --auto=yes --threshold=99 cg.out > ann.txt   # per-line
# Read the *file:function* rows, not just function totals: a function whose
# cost sits in slice/iter/macros.rs and ptr/non_null.rs is walking, not
# computing, and the fix is fewer walks. That is what found this run's -9 %.
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

**An allocator-shaped change must be measured with the shipped allocator.**
The default is mimalloc; callgrind forces the *system* allocator (valgrind
replaces malloc). A change that removes allocations therefore reads far
larger under callgrind and under a `--no-default-features` wall-clock A/B
than it is worth in a training run: this run's `Printed<T>` row measured
-17.09 % Ir and +13.5 % system-allocator wall-clock, and **+1.7 %** at
`release` with mimalloc. Ir still tells you *whether* a change helps and by
how much it cut work; only a `release` run tells you what ships. Do both
before quoting a throughput number for an allocation fix.

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

**Anchored 2026-08-10 at `610df3b6`** (`release`, mimalloc — the shipped
configuration), i.e. after the thirteenth pass's first two rows; its third
(-0.49 %) landed later and is an order of magnitude below this bench's
resolution. The `6bbdc38c` anchor below it
(55.88) is superseded as an absolute and kept for the box-drift lesson: it
was taken on a *different container*, and this one runs the same workload
~25 % faster (`host_calib_ms` 45-54 against 49-61). **The two anchors do
not subtract — nothing like +25 % happened.**

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_calib_ms        45-54 across the sitting   <- within-sitting only
games                320
games_per_s          67.96 / 71.00 / 73.99 / 72.98 / 74.03 / 72.56 / 70.79 /
                     68.82   (mean 71.52, spread 8.5 % — take >=6 runs)
decisions_per_s      mean 43185
turns_per_game       26.98
stalls               0 (0.00 %)
peak_rss_mib         22.1 - 22.5
determinism          ok (all 160 pairs split, on all 8 runs)
```

**The thirteenth pass's wall-clock is a null, and that is the expected
result.** Six alternated `release` + mimalloc pairs of `a4947da6` against
`610df3b6`, both built and run in one sitting on this container: **69.22 ->
69.59 games/s, +0.54 % mean, 3/6 pairs positive** (paired deltas -4.73 /
+8.66 / +1.70 / -0.26 / +3.37 / -5.29 %; the +8.66 pair's A run read
`host_calib_ms` 61 against 46 everywhere else). The pass was -4.17 % at
the tip measured; it finished at -4.64 %, and neither can be resolved by
wall-clock on a box whose within-sitting
spread is ±8 % — this is the "sub-5 % changes need callgrind" note above,
demonstrated rather than asserted. **The pass's measurement is the Ir
figure; this block is a health check** (turns/game 26.98, stalls 0,
determinism ok, RSS flat), not a delta.

The pre-merge anchors below are kept only for the box-drift lesson.

**The previous 70.65 anchor does not compare at all**: it predates
`998b2433` making `EvalWeights::default()` carry `determinize: 1`, and
`--bench` runs `gang` = `EvalWeights::default()`, so the bench *workload*
changed underneath it. Treat 70.65 as belonging to a different bench.
**Read `host_calib_ms` before comparing to an older anchor.** Refresh only
alongside an intentional, explained change; regressions beyond ~5 % get
investigated before anything else lands.

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_calib_ms        49-61 across the sitting   <- within-sitting only
games                320
games_per_s          56.42 / 56.31 / 55.72 / 55.45 / 56.52 / 56.63 / 54.19 /
                     55.77   (mean 55.88, spread 4.5 % — take >=6 runs)
decisions_per_s      mean 33741
turns_per_game       26.98
stalls               0 (0.00 %)
peak_rss_mib         22.3 - 22.5
determinism          ok (all 160 pairs split, on all 8 runs)
```

**Absolutes do not transfer between containers, and this pass proved it
twice.** The same engine code read 60.49 and 64.42 in two sittings on one
box (+6.5 % of pure drift), and the second session's container read 55.70 on
its own tip where this container reads 55.88 on a tip with 1.0 % *fewer*
instructions. **Quote a paired A/B measured in one sitting, never a
difference of anchors.** The pass's own deltas, each paired:

- callgrind, merged, `81c88580` -> `6bbdc38c`: **-11.68 %**
- wall-clock, first session, 6/6 alternated pairs: **+9.57 %**
- wall-clock, second session, 8/8 alternated pairs: **+10.85 %**
  (50.24 -> 55.70; the two sessions' work overlaps, so these don't add)

*(Pre-merge anchors, kept for the drift record only: `64.42` mean of 6 on
the first session's pushed tip, `60.49` mean of 8 on the same engine code an
earlier sitting the same night, `55.70` mean of 8 on the second session's
pre-rebase tip on a different container. None is comparable to another or to
the anchor above.)*

**What the eleventh pass is worth, measured end to end.** Eight alternated
pairs in one sitting, both sides `release` + mimalloc on the same idle box:

```text
                pre-pass 87d76144     tip           paired delta
pair 1          62.52                 67.86         +5.34
pair 2          65.73                 70.52         +4.79
pair 3          67.01                 72.10         +5.09
pair 4          68.70                 70.73         +2.03
pair 5          67.97                 71.81         +3.84
pair 6          67.37                 68.92         +1.55
pair 7          69.41                 73.22         +3.81
pair 8          64.81                 70.02         +5.21
mean            66.69                 70.65         +3.96  = +5.93 %
dec/s mean      40272                 42660                  +5.93 %
median paired                                       +4.31
```

**8/8 pairs positive**, against a callgrind **-8.63 %** over the same span
(6,151,455,670 -> 5,620,794,622). The wall-clock lands at about two thirds
of the instruction win, which is the expected shape: four of the five rows
remove *gathers* and translate, one (`usable_abilities`) removes
*allocations* and mimalloc absorbs most of it — see the `Printed<T>` row,
where -17.09 % Ir was worth +1.7 % at `release`.

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
| 2026-08-09 | `CardInstance` becomes a CoW handle: the ~110 fields move to `CardData` behind an `Arc`, `Deref`/`DerefMut` keep every `card.field` read and write working, and `DerefMut` is the single unshare point (candidate 7) | 14,403,731,176 Ir; 25.44 games/s, 15351 dec/s | 10,718,206,071 Ir (**-25.59 %**); 32.50 games/s (**+27.8 %**) | The zones were already `CowBox<Vec<CardInstance>>`, so a `GameState` clone was cheap but the *first write* deep-copied every card in the zone: `Arc::make_mut` was **24.12 %** inclusive and `CardInstance::clone` **14.69 %** self, 15.85 % of the program under `advance_step` alone. Now unsharing a zone copies N pointers and only the written card clones. Callgrind A/B on identical `release-fast --no-default-features` binaries over the fixed six-game workload; `--bench` ×4 alternated pairs, 4/4 positive and non-overlapping (base 25.12-25.85, cand 31.88-32.91). Peak RSS **fell** 23.3 → 21.3 MiB (shared cards aren't duplicated). Suite 18810 passed / 0 failed; all four golden traces byte-identical; `turns_per_game` 26.98 and `stalls` 0 on all 10 bench runs; determinism ok, all pairs split. Cost: two borrow-checker fixes where a `&mut` field write and a `&self` read of the same card overlapped. **Under `release` + mimalloc this row carries essentially the run's whole +35.6 %** (see Baseline): unlike the `Printed<T>` row it removes deep struct copies, not just allocations, so the allocator can't absorb it. |
| 2026-08-09 | Gate each of the gather's 38 per-variant passes on a `u64` presence mask built in one walk of the board's static abilities (candidate 3(ii)) | 8,886,099,152 Ir | 9,013,111,944 Ir (**+1.43 %**) | **No win — reverted.** The gather's own self cost went *up*, 1,573 M -> 1,696 M (+7.8 % of itself): the 38-arm classifier costs ~342 Ir per gather and the passes it skips were already near-free. The negative result is the useful part — it says the 39 passes are not where the gather's 4,385 Ir/call lives, so the next attempt should profile the body line by line (`--profile profiling`, which keeps debuginfo) rather than guess again. Written up in candidate 3. |
| 2026-08-09 | `ComputedPermanent`'s four printed-derived collections (`card_types`, `supertypes`, `subtypes`, `keywords`) become `Printed<T>` — the `Arc<CardDefinition>` plus a projection, cloned only on the first layer write (candidate 1(a)) | 10,718,206,071 Ir; 32.61 games/s (system alloc); 45.73 games/s (`release`, mimalloc) | 8,886,099,152 Ir (**-17.09 %**); 37.02 games/s (**+13.5 %**, system alloc); **46.49 games/s (+1.7 %, `release` + mimalloc — the number that ships)** | `compute_permanent_pass` was **3,482,320 of the program's 8,723,045 allocations (40 %)**, nearly all of them cloning a collection that nothing then modified. `Printed<T>` `Deref`/`DerefMut`s to `T`, so the ~4.5 k `cp.keywords.contains(…)` / `cp.subtypes.creature_types` sites are untouched; the whole change is 11 `.clone()` → `.to_vec()` fixups plus gating the two unconditional `keywords.retain` calls (they take `&mut`, so they materialized the list even when they removed nothing). Result: **compute_permanent_pass allocations 3,482,320 → 30,086**, program-wide 8,723,045 → **5,093,895 (-41.6 %)**, allocator self cost 21.4 → 10.9 %. Callgrind A/B on identical `release-fast --no-default-features` binaries, fixed six-game workload; `--bench` ×4 alternated, 4/4 positive and non-overlapping (before 31.94-33.31, after 35.44-37.66). Peak RSS 21.3 → 23.7 MiB — the one cost, four `Arc<CardDefinition>` clones per computed permanent keeping definitions alive. **The two wall-clock figures disagree and the mimalloc one is the real one**: the +13.5 % was measured against the *system* allocator (`--no-default-features`, which callgrind forces), and a 4/4-pair alternated A/B of the two `release` + mimalloc binaries reads **45.73 → 46.49 games/s, +1.7 %** (paired deltas +0.76 / +0.03 / +0.79 / +1.46, `host_calib_ms` 45-58). Kept: 4/4 positive under the shipped configuration, an exact -17.09 % of the work, and no cost but 2 MiB. See the new allocator note under "How to measure". Suite 18810 passed / 0 failed; all four golden traces byte-identical; turns_per_game 26.98, stalls 0, determinism ok on all 16 runs. |
| 2026-08-09 | Fold the gather's eleven whole-battlefield walks into one: the walk that builds `sa_cards` also sets a presence flag per pass, and each pass iterates an empty slice when its flag is clear. Bludgeon Brawl's `ArtifactsAreEquipment` scan is hoisted out of its per-card loop (candidate 3) | 8,887,218,012 Ir; gather self 1,573,494,744 | 8,085,908,260 Ir (**-9.02 %**); gather self 995,310,108 (**-36.7 %**) | **The line-level attribution the previous attempt asked for, taken on the new `profiling-fast` profile.** It says the gather's 4,385 Ir/call is not in the 39 per-variant arms at all: **527,365,400 Ir (5.93 % of the program) is `slice::iter` and 494,753,454 (5.57 %) is `ptr::non_null`** — ~65 % of the gather is raw slice iteration and pointer advance, against 179,561,212 in `mod.rs` itself. That is the *eleven separate `for card in &self.battlefield` loops*, four of which scanned a `Vec<Keyword>` per card unconditionally, plus `brawl_equip_mv` — which re-scanned every card's static abilities *per battlefield card*, 7,140,444 `is_artifact` calls for six games. Now one walk with one keyword scan per card, and each pass is skipped wholesale. Gather calls unchanged (358,792), so Ir/gather is 4,385 → 2,774; `is_artifact` + `is_creature` self 222,027,450 → 114,920,790. **Why this worked where the presence mask didn't**: that one gated already-empty walks over a short `sa_cards` and paid a 38-arm classifier; this one gates eleven walks of the *whole* board and pays one `u32`-ish flag set per card. Callgrind A/B on identical `profiling-fast --no-default-features` binaries (= `release-fast` + debuginfo, same opt settings — the baseline's gather self reproduces PERF's recorded 1,573,494,744 exactly), fixed six-game workload. Suite 18627 passed / 0 failed; all four golden traces byte-identical. **One trap paid for**: the Unleash loop also carries the CR 611.2 predicate gate, its sibling, and the suspect / living-metal statics — gating that loop on `any_unleash` silently killed every `WhileCondition` static (15 failures across `classic_sets`). It stays ungated; only its two keyword scans are gated. Check a loop's whole body before gating its head. |
| 2026-08-09 | `effective_mana_abilities` returns `Cow<'_, ActivatedAbility>` so printed mana abilities are borrowed from `card.definition` instead of deep-cloned, and `granted_abilities_for` does its linear `battlefield_find` once instead of twice (candidate 1.5) | 8,085,908,260 Ir | 7,993,961,114 Ir (**-1.14 %**) | `granted_abilities_for` 404,837,864 → 388,335,872 (**-4.1 %** of a function that had never been touched and is 272,334 calls per six games). Only three callers, all of which read `.effect` and the cost fields, so `Cow` costs nothing at the call sites — `Deref` covers `mana_source_cost_rank(first)`. Callgrind A/B on identical `profiling-fast --no-default-features` binaries, fixed six-game workload; under the 5 % claim bar, which is exactly why it is quoted in instructions rather than `--bench`. Suite 18627 passed / 0 failed; all four golden traces byte-identical. Cumulative for the two rows this run: **8,887,218,012 → 7,993,961,114, -10.05 %**, and at `release` + mimalloc — the shipped configuration, ×4 runs on the same box as the previous anchor (`host_calib_ms` 45-51 vs 45-49) — **45.72 → 48.74 games/s mean, +6.6 %**, decisions/s 27609 → 29431, peak RSS 24.0 → 22.4 MiB, `turns_per_game` 26.98 and `stalls` 0 on every run, determinism ok. **Baseline** is re-anchored there. |
| 2026-08-09 | `Player` becomes a CoW handle: the ~165 fields move to `PlayerData` behind an `Arc`, `Deref`/`DerefMut` keep every `player.field` read and write working, and `DerefMut` is the single unshare point (candidate 1.7) | 7,994,965,799 Ir | 7,818,537,433 Ir (**-2.21 %**) | Candidate 7 one level up, and the ceiling is much lower for the reason the shape implies: there are **two seats, not twenty cards**, and one of them is written on essentially every checkpoint. `GameState::clone` inclusive **967,524,311 → 473,827,424 (-51.0 %)** and `drop_in_place<GameState>` 441,814,641 → 396,985,432, against a new `Player::deref_mut` line at **326,104,839 (4.17 % of the program)** — i.e. the change removes the *other* seat's deep copy and nothing else. Six borrow-checker fixes, all one kind: a `&mut` field borrow now borrows the whole seat, so three loops re-borrow per zone, `manifest_card` scans library-then-hand instead of chaining, and `find_card_anywhere_mut` locates with shared borrows before taking the one `&mut` (which also stops it unsharing every seat on a miss). Snapshot wire format unchanged — `Player`'s serde impls are transparent over `PlayerData`. Rider measured at **-0.04 %**: `empty_mana_pools` gets a read-only fast path (plus `ManaPool::is_empty()`, stricter than `total() == 0`) so it stops taking `&mut` on 51 k step changes with nothing to do; kept for the sharp edge it documents, not the instructions — **the deep copy it skips is mostly paid by the next writer in the same checkpoint**, which is the general lesson for CoW gating. Callgrind A/B on identical `profiling-fast --no-default-features` binaries, fixed six-game workload. `core_rules` 1664 passed / 0 failed; golden traces byte-identical. |
| 2026-08-09 | `granted_abilities_for` looks its own card up once instead of eight times, and classifies the five "has the activated abilities of …" statics in one pass | 7,818,537,433 Ir | 7,696,302,458 Ir (**-1.56 %**) | The function ran `battlefield_find(card_id)` — a linear scan of the whole battlefield — once at the top and again inside each of seven blocks, all resolving to the same card (the top one early-returns when the card is off the battlefield, so the later seven could never differ). It is called ~233 k times per six games from three per-card loops, so the redundant scans multiply. `granted_abilities_for` **411,928,730 → 291,885,476 inclusive (-29.1 %)**, its `ptr::non_null` line 120,941,056 → 79,449,414 (-34.3 %) — pointer advance, the same signature the gather fold read. Push order into `out`, which indexes `GameAction::ActivateAbility`, is unchanged. `core_rules` 1664 passed / 0 failed; golden traces byte-identical. |
| 2026-08-09 | `GrantScan`: the board-level half of `granted_abilities_for` is hoisted out of the per-card loops (candidate 2's shape, applied to activated rather than triggered grants) | 7,696,302,458 Ir | 7,497,680,035 Ir (**-2.58 %**) | What was left in `granted_abilities_for` after the row above was **four whole-board walks per call**: `all_static_sources()` (battlefield + command zones) for `GrantActivatedAbility` statics, both graveyards for `GrantActivatedAbilityFromGraveyard`, and two more battlefield passes for soulbond pairs and attached `equipped_bonus`. None of those depend on *which* permanent is asking — including the CR 611.2 `active_static` unwrap and the Hellbent-style `condition` predicate, both of which read only the source. `grant_scan()` collects them in one pass and `granted_abilities_with(card_id, &scan)` does the per-card half. Threaded into the five per-card loops that were paying it: `mana_source_table_inner`, `untapped_relevant_source_exists_inner`, `untapped_producers_of_inner`, `bot::available_mana`, and `bot::usable_abilities` (six call sites). `granted_abilities_for` keeps its old signature by building a scan of its own, so the ~18 k non-loop callers are unaffected. Emission order into `out` is preserved exactly — each scan `Vec` is filled in the source-iteration order the old inline loop used. Callgrind A/B on identical `profiling-fast --no-default-features` binaries. Full suite green; golden traces byte-identical. |
| 2026-08-09 | `compute_permanent_pass` reads the permanent's counters in one pass of the map instead of ten keyed `counter_count` lookups | 7,497,680,035 Ir | 7,282,343,054 Ir (**-2.87 %**) | The CR 613.7f P/T block asked for ten counter kinds by name (`PlusOnePlusOne`, `MinusOneMinusOne` and the eight one-sided ones), and it runs **per card per layer pass — 12.0 M per-card computations for six games**. The caller tree named it exactly: `compute_permanent_pass` → `counter_count` **12,775,080 calls, 166,076,040 Ir**, on permanents that carry one or two counter kinds when they carry any. Now one `for (kind, n) in &card.counters` with a ten-arm match. Summing is order-independent, so the `HashMap` iteration order can't leak into game state — the golden traces, which are the cross-process determinism check, are byte-identical. **Measured first and rejected on the way here**: an `is_empty()` fast path *inside* `counter_count` read **-0.02 %**, because hashbrown's `get_inner` already early-returns on an empty table before hashing. The cost was never the lookup, it was making ten calls; reverted. Full suite 18090 passed / 0 failed; clippy clean. |
| 2026-08-09 | The two whole-battlefield trigger walks stop rebuilding the board-level grant lists per card (candidate 2, the triggered half) | 7,282,343,054 Ir | 7,035,606,377 Ir (**-3.39 %**) | `trigger_grant_sources` and `equip_granted_trigger_sources` were hoisted a run ago, but the **per-card shims that rebuild them were still being called inside per-card loops** — `stack.rs`'s step-trigger walk over the whole battlefield and `actions.rs`'s SpellCast walk over every live permanent, both O(cards²) against `all_static_sources`. Each now builds the pair once and calls `statics_granted_triggers_with` / `equip_granted_triggers_with`. `dispatch_triggers_for_events` was 551,982,244 inclusive (7.58 %) with `trigger_grant_sources` 262,168,860 (3.60 %) and `statics_granted_triggers_for` 268,230,216 (3.68 %) — all three of them walking, 198 M of `trigger_grant_sources` sitting in `option.rs` / `mut_ptr.rs` / `ptr::non_null.rs`. **The lesson to carry: hoisting a scan into a `_with` variant is only half the fix — grep the `_for` shim's callers for a surrounding loop.** The third such loop, `combat.rs`'s per-attacker gather, is *not* converted: it mutates `self` inside the loop, so a `&self`-borrowed grant list can't live across it, and attacker counts are small. Full suite 18090 passed / 0 failed; golden traces byte-identical; clippy clean. |
| 2026-08-09 | Coat of Arms' gather pass stops building a whole-board creature list before checking whether the static that consumes it is anywhere on the board | 7,035,606,377 Ir | 6,538,441,281 Ir (**-7.07 %**) | **The largest single row of the run, from gating one block.** The pass built `Vec<(CardId, &Vec<CreatureType>, bool)>` over every creature — an allocation, an `is_creature` per card and a `Vec<Keyword>` scan per card for `Changeling` — and *then* walked the whole battlefield looking for `PumpPerSharedType`, which is on the board approximately never. `is_creature` was **5,078,630 calls / 78,145,532 Ir inside the gather**, the single hottest line in it, and this pass was most of them. Now the presence question is asked once off the short `sa_cards` list (the same shape as `any_artifacts_are_equipment` beside it), and the inner walk iterates `sa_cards` rather than the battlefield. Safe to gate the *whole* block, unlike last run's Unleash trap: `PumpPerSharedType` is the loop's only purpose, and the pass matches `sa.effect` directly rather than through `active_static`, so a bare `matches!` finds exactly what the loop would have. Its sibling one block up (Sliver Legion / `PumpPTPerOtherOfType`) was already in this shape, which is what the fix copies. `recent_a`'s `coat_of_arms_scales_with_shared_types` exercises the gate's true side. Full suite 18093 passed / 0 failed; golden traces byte-identical; clippy clean. |
| | **cumulative this run** | **7,994,965,799 Ir** | **6,538,441,281 Ir (-18.22 %)** | six alternated `profiling-fast --no-default-features` callgrind A/Bs on the one fixed six-game workload, so the six rows subtract honestly |

Tenth run. Baseline re-taken on a fresh box before anything landed:
**6,497,854,664 Ir** on the same fixed six-game workload (the 6,538,441,281
above was a different host image; the two absolutes are within 0.6 % and the
rows below subtract against the 6,497,854,664 re-take, not against it).

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-09 | Hoist `compute_permanent`'s three CR 613.8 gate scans out of `apply_layers`' per-card loop and fuse them into one pass | 6,497,854,664 Ir | 6,497,409,724 Ir (**-0.007 %**) | **No win — reverted, and the reason is worth keeping.** `compute_permanent` re-derives `has_power_gated` / `has_type_changer` / `has_type_lord` per permanent, three `any()` walks of the effect set inside a whole-board loop — textbook per-card-loop-rebuilds-a-board-scan. LLVM had already done it: the scans are loop-invariant and `compute_permanent` inlines into the `map`, so the hoist moved 444,940 Ir out of 6.5 G. **Check whether the optimizer already hoists a scan before writing the hoist** — the tell is that the function's *self* cost is tiny (`compute_permanent` self was 31,937,700, 0.49 %) even though the source reads O(cards x effects). |
| 2026-08-09 | `GameState`'s cold tail — 90 per-turn / end-of-turn registries, `teams`, the range matrix, cost and vote scratch — moves into a `ColdState` behind one `CowBox`, with `Deref`/`DerefMut` on `GameState` so no call site changes (`69f3a94b`) | 6,497,854,664 Ir | 6,437,616,446 Ir (**-0.93 %**) | `perform_action` snapshots the whole state before every action so a rejected one rolls back. The clone was **379,630,296 Ir (5.84 %)** and dropping it another **272,555,229 (4.19 %)** — 10.0 % of the program over 64,248 actions, of which **20** actually rolled back. Almost none of that is allocation: it is the field-by-field walk over ~140 separate `Vec`/`HashMap`/`HashSet` fields that are empty on a real board (~30 Ir each, both directions). After: clone **161,485,660 (-3.36 % of the program)**, drop 254,059,834. The drop barely moves because it is dominated by the ~45 collections still in `GameState` plus the few that are genuinely non-empty (`players`, `controlled_by`, `acted_on_own_turn`). Suite 18,817 passed / 0 failed; golden traces byte-identical; clippy clean; snapshot round-trip green (`#[serde(flatten)]` keeps the wire shape flat). |
| 2026-08-09 | The same grouping, widened to 126 fields (`*_this_resolution` scratch, `pending_*`, `block_map`, `died_card_snapshots`, `delayed_triggers`) | 6,497,854,664 Ir | 6,577,530,529 Ir (**+1.23 %**) | **Regression — not landed.** The boundary, measured rather than guessed. Those fields are written on most actions, so the group unshares almost every time and one 126-field unshare costs more than the individual empty clones it replaced. **The rule: group size x unshare probability has to stay under the sum of the individual clone costs.** `resolve_effect`'s prologue alone clears twelve of them, and it runs ~20 k times per six games. Membership is a free parameter — `Deref` means moving a field in or out changes no call site — so the next run can retune it with one build. |
| 2026-08-09 | The gather's `AnthemForFilter` walk iterates `sa_cards` instead of the whole battlefield; Leyline of Singularity's presence check asks `sa_cards` too (`f1908d4f`) | 6,437,616,446 Ir | 6,156,192,934 Ir (**-4.37 %**) | **Two passes the eleven-walk fold missed, found by listing every top-level statement in the function rather than every `for card in &self.battlefield`.** The anthem walk chained `self.battlefield.iter()` with the emblem and command-zone legs, and its body does nothing for a card with no printed statics — `sa_cards` is exactly that filter in the same order, so the emitted effect sequence is unchanged. The Leyline check is the Coat of Arms shape verbatim: scan every permanent's statics before asking whether the card is even out. Gather self **885,832,058 -> 610,672,182 (-31.1 %)**. Full suite green; golden traces byte-identical. |
| 2026-08-09 | `card.keyword_counters` becomes an insertion-ordered `Vec` newtype instead of a `HashMap` (`86670250`) | 6,156,192,934 Ir | 6,150,469,969 Ir (**-0.09 %**) | Landed as a determinism fix, not a perf row — see TODO's robustness section. The instructions are a rounding error but they are the right sign: the linear scans replace hashing a `Keyword`, several of whose variants own a boxed `SelectionRequirement`. |
| | **cumulative this run** | **6,497,854,664 Ir** | **6,150,469,969 Ir (-5.35 %)** | four alternated `profiling-fast --no-default-features` callgrind A/Bs on the one fixed six-game workload |

Eleventh pass, landed on top of the tenth. Base re-taken at `87d76144`:
**6,151,471,423 Ir**, 0.016 % off the 6,150,469,969 above — same source, a
different link, which is the size of build-to-build noise on this profile and
worth remembering before claiming anything under ~0.1 %. The four rows below
were each measured individually against a *pre-rebase* base of 6,539,623,988
(the branch point, before the tenth pass landed), so their per-row percentages
are indicative and their absolutes do not chain; the **combined** figure at the
bottom is the authoritative one, measured base-vs-tip on the current base.

**One shape produced all four: a `&mut self` path taking a `computed_permanent`
outside a freeze scope.** Each such call re-gathers *every continuous effect in
the game* — ~2,900 Ir — to answer one question about one card.

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-09 | `do_untap`'s two CR 502.3 untap-lock scans share one whole-board layer pass instead of a `computed_permanent` each, per permanent | — | (**-4.08 %** pre-rebase) | The biggest single item the caller tree named, and it was two `filter` closures. The untap step called `computed_permanent` twice per battlefield permanent (`DoesntUntapWhileCounter`, `DoesntUntapIfAttackedLastTurn`) — **64,956 calls at 4,356 Ir each, 282,935,856 Ir / 4.33 % of the program**. One `compute_battlefield()` and a `zip` answers both, at one gather per untap step instead of ~39. Both locks still read *computed* keywords, so Temporal Distortion's hourglass counters and Tangle Kelp's granted lock behave as before. |
| 2026-08-09 | `scale_damage_to` reads its source permanent once; `activate_ability_inner`'s three-read prelude runs in one `with_frozen_layers` scope | — | (**-1.47 %** pre-rebase) | `scale_damage_to` asked `computed_permanent(source)` for the controller+colours and again three lines later as an "is the source a permanent?" test: 21,936 calls → 14,624. `activate_ability_inner` is `&mut self`, so its `lost_all_abilities` / `granted_abilities_for` / `intrinsic_land_mana_abilities` prelude gathered three times per activation; one scope makes the last two hit the memo. **Measured with a third site that was an exact null and is not in the commit**: a scope around `creature_count`'s battlefield walk removed 0 gathers — its callers already hold one. |
| 2026-08-09 | `effective_mana_abilities_with` reads its card's computed land types once and threads them into the two CR 305.6 checks | — | (**-0.26 %** pre-rebase) | **Right shape, small number, and the reason is the useful part.** Costed at ~2 % off the inclusive shares (`printed_land_mana_ability_lost` 72,436 calls / 144 M, `intrinsic_land_mana_abilities` 85,854 / 27 M); returned an eighth of that. `computed_permanent` calls fell 195,062 → 122,626 but **gathers were flat**: both helpers already run inside `mana_source_table`'s freeze scope, so every read but the first per card was a memo *hit*. **Deduplicating a hit is worth ~200 Ir, not the ~2,900 a gather costs — check whether the caller is frozen before costing a dedup.** |
| 2026-08-09 | `activate_ability`'s "did a creature make this mana" flag comes off the `ComputedPermanent` `_inner` already reads | — | (**-1.15 %** pre-rebase) | The CR 106.12 mana-source flag was `battlefield.iter().any(id) && permanent_is_creature(card_id)` in the `&mut self` wrapper — one whole-game gather per activation, immediately before `_inner` opened a scope and gathered again: **18,386 calls, 66,604,359 Ir (1.09 %)**. `_inner` now reads `card_types` off the same `Arc<ComputedPermanent>` it reads `lost_all_abilities` from and hands the caller the pre-activation mana pool through an out-parameter; nothing has moved mana at that point and the two error paths before it cannot have produced any, so the marking is unchanged. |
| 2026-08-09 | `usable_abilities` returns `Vec<(usize, Cow<'_, ActivatedAbility>)>` so a permanent's *printed* activated abilities are borrowed from `card.definition` instead of deep-cloned (`bab861cf`) | 5,797,631,371 Ir | 5,620,794,622 Ir (**-3.05 %**) | **The one row of this pass that is not the freeze shape — it is the allocation shape**, and it landed after the four above (a second session's work, rebased on). 83,608 calls from six per-permanent ability generators, each cloning an `ActivatedAbility` list whose `Effect` trees allocate; all six callers only read `.effect` and the cost fields, so `Deref` covers every call site and only the synthesized `granted_abilities_with` half stays owned. `usable_abilities` 3.68 % of the program -> inlined away at ~44 M; **program allocations 4,316,601 -> 4,073,937 (-5.6 %)**. Same shape as the `effective_mana_abilities` `Cow` row two passes ago, one layer up. Measured twice — -2.95 % against the tenth pass's tip pre-rebase, -3.05 % against `87507a88` after — which is also a useful cross-check that the two sessions' bases agree. **Expect mimalloc to absorb part of it at `release`**: see the `Printed<T>` row. Suite 18,975 passed / 0 failed; golden traces byte-identical. |
| | *(four-row subtotal)* | **6,151,471,423 Ir** | **5,798,284,923 Ir (-5.74 %)** | base `87d76144` vs tip, both `profiling-fast --no-default-features`, built and run in one sitting on the one fixed six-game workload. This is the number that counts; the four rows above were measured pre-rebase. |
| | *not landed* | 6,539,623,988 Ir | 6,498,593,052 Ir (-0.63 %) | **`LayerGates` — dropped in the rebase, and the disagreement is the point.** Hoisting `compute_permanent`'s three CR 613.8 gate scans out of the per-card loop measured **-0.63 %** on the pre-rebase base (the function's self cost 74,510,172 → 34,492,716) and **-0.007 %** on the tenth pass's base, where LLVM had already hoisted them (self cost 31,937,700). Same source, same profile, two different codegen outcomes — so "the optimizer already does it" is a property of the build, not of the code. The tenth pass's revert stands; this note exists so the third attempt doesn't cost another two builds. |
| | **cumulative this pass** | **6,151,455,670 Ir** | **5,620,794,622 Ir (-8.63 %)** | base `87d76144` vs the final tip, both `profiling-fast --no-default-features`, on the one fixed six-game workload. **8/8 alternated `--bench` pairs positive at `release` + mimalloc, +5.93 %** — see Baseline. |

Twelfth pass — the whole-board layer pass, four sites. Base `48ac252c`.

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-09 | The SBA sweep's CR 603.8 steal-penalty disarm walks the latch set instead of the battlefield, and its CR 704.8 ±1/±1 snapshot is gated on the same printed-keyword predicate its reader uses (`97c025a6`, candidate 0.25(b)) | 5,622,084,243 Ir | 5,519,931,366 Ir (**-1.82 %**) | `check_state_based_actions` 11.68 % -> 9.93 % inclusive. Both were whole-board walks kept alive for cards no ordinary board carries: a Bronze Bombshell that has changed controllers, and Persist/Undying. `spec_from_iter` under the sweep was flat (178.2 M both sides) — the win is in `battlefield_find` and the `HashMap` build, not the collects. |
| 2026-08-09 | `declare_attackers_banded` takes one `compute_battlefield` for all four of its legality reads (`a21da084`, candidate (A)) | 5,519,931,366 Ir | 5,411,392,050 Ir (**-1.97 %**) | 9,928 whole-board passes over ~2,600 declarations: band legality, CR 508.0 "attacks only alone", "can't attack alone", and the trigger collection each took their own. Everything between them returns `Err` without touching a layer input. Never more work than before — the band pass was already unconditional and the two alone-checks are mutually exclusive. |
| 2026-08-09 | `finalize_cast` routes its CR 113.10b stripped-set read through `permanents_with_abilities_removed`; the CR 602.5c lock check uses `computed_permanent` (`622b43ae`) | 5,411,392,050 Ir | 5,309,173,117 Ir (**-1.89 %**) | The guarded helper had existed since `c365ede8` and this caller never adopted it — it answers "nothing is stripped" off the gathered effect set on any board without a Turn to Frog. **Grep for the naive shape when a guarded helper lands**; a second copy of the pattern was still there a pass later. |
| 2026-08-09 | `has_first_strikers` and `bands_with_other_qualities` read their combat participants under one freeze instead of computing the board (`a37863a0`, candidate 0(d)) | 5,309,173,117 Ir | 5,260,848,923 Ir (**-0.91 %**) | 2-6 creatures out of ~20 permanents. `has_first_strikers` also short-circuits on an empty combat. `bands_with_other_qualities` keeps the battlefield walk for ordering — its qualities are consumed positionally — and computes only the members. |
| | **cumulative this pass** | **5,622,084,243 Ir** | **5,260,848,923 Ir (-6.43 %)** | base `48ac252c` vs tip **measured pre-rebase** (the four rows then rebased onto `81c88580`, whose changes are in the ML/client/lobby paths); four alternated `profiling-fast --no-default-features` callgrind A/Bs on the one fixed six-game workload. **Wall-clock, 6/6 alternated `--bench` pairs positive, mean +9.57 %** (base 14.49-16.45, cand 16.29-17.88 games_per_s_th) on the same two binaries — larger than the Ir delta, which is the expected direction for removing whole layer passes: each one is a gather plus ~20 per-card passes, and its cache traffic is what Ir under-weights. |

Twelfth pass, **second concurrent session** — the state-based-action sweep.
Base `81c88580`. **These four rows were measured on a tree that did not
contain the four above, and vice versa**: the two sessions ran at the same
time on the same branch and were joined by a rebase. Where they overlap (the
CR 603.8 disarm, the CR 704.8 snapshot, `declare_attackers_banded`,
`finalize_cast`) the merged tree keeps one implementation, so **the two
cumulative figures describe overlapping work and must not be added.** The
merged tip has not been re-measured; that is the first job next run.

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-10 | The SBA's ~20 rare whole-board sweeps ride one presence pass (`2038bb59`) | 5,620,660,987 Ir | 5,444,517,546 Ir (**-3.13 %**) | `check_state_based_actions` opened each rare state-based action with its own `battlefield.iter().filter(…).collect()` and every one came back empty: **178,224,638 Ir (3.17 %) in `spec_from_iter.rs:check_state_based_actions`**, plus **63,537,327 (1.13 %)** in hashbrown for the CR 704.8 ±1/±1 map and the soulbond id set. `sba_board_scan` answers "can this SBA fire at all" for all of them in one battlefield pass; each block keeps its original code behind an over-approximating flag, and the scan is retaken wherever the sweep can change the answer (flip, `BecomeCopyOf` revert, Persist/Undying return, defeated battle). Rode along, same shape: the CR 122.3 / 122.4 / soulbond / token-cleanup loops read before taking `&mut` (which unshares the zone, and for the token sweep the seat's whole `PlayerData`). `check_state_based_actions` inclusive **656,879,426 (11.69 %) → 485,902,312 (8.92 %), -26.0 %**. **This is the answer to the other session's "next up (1)"** — the ~21 collects per sweep were the cost, and a presence pass, not line-level costing, is what removes them. |
| 2026-08-10 | The death sweep's layer pass skips permanents that can't be creatures (`f3c8670c`) | 5,444,517,546 Ir | 5,339,874,694 Ir (**-1.92 %**) | The CR 704.5g scan's `compute_battlefield()` was **155,605,066 Ir, 2.86 %**, the largest line in the sweep, and half a bench board is lands. `apply_layers` computes each permanent independently, and only `AddCardType` / `RemoveCardType` / `SetCardTypes` write `card_types`, so with none live the noncreatures' views are never built and the scan's existing `unwrap_or_else(is_creature)` fallback gives the same answer. Bestow's layer-4 Creature strip is card-intrinsic, so a bestowed permanent is still computed and still reads noncreature. **The other session's note that the sweep's layer pass is "genuinely whole-board" is half right** — it is whole-board over *creatures*, and `compute_battlefield_creatures` now exists for any caller in that shape. |
| 2026-08-10 | The SBA presence scan runs once per sweep and walks each vector once (`129a1b0e`) | 5,339,874,694 Ir | 5,271,399,214 Ir (**-1.28 %**) | The scan itself was **46,166,408 + 45,227,168 Ir (1.68 %)** across its two sites. The post-death retake is unnecessary — the flags over-approximate and everything before it only *removes* permanents, which can only turn a flag off — so it is taken only when a Persist/Undying return actually grew the board. The per-card body walked `keywords` ×3, `supertypes` ×2, `card_types` ×2 and `enchantment_subtypes` ×2 through `contains` / `is_planeswalker` / `is_battle` / `is_aura`; it now walks each once with a `match`, and the two `counter_count` lookups sit behind `counters.is_empty()`. |
| 2026-08-10 | Three call sites stop rebuilding a whole-board layer view they already have (`0045cbc0`) | 5,271,399,214 Ir | 5,013,096,289 Ir (**-4.90 %**) | Found independently of the other session and overlapping it: `declare_attackers_banded` (three passes here, four there — same fix, theirs landed), `finalize_cast`'s CR 113.10b strip set (same fix, theirs landed), and **`declare_blockers`, which is this row's unique half** — its own pass plus CR 509.1b's Okk power read, 132,691,778 Ir over 7,754 calls, with nothing between them mutating the board. `compute_battlefield` inclusive **602,521,248 (11.43 %) → 362,605,632 (7.23 %)** on this session's tree. |
| | **cumulative, second session** | **5,620,660,987 Ir** | **5,013,096,289 Ir (-10.81 %)** | base `81c88580` vs this session's pre-rebase tip, both `profiling-fast --no-default-features`, every side built and run in one sitting on one container. **Wall-clock: 8/8 alternated `release` + mimalloc pairs positive, 50.24 -> 55.70 games/s, +10.85 %** (per-pair deltas +2.89 / +7.57 / +4.75 / +5.75 / +4.67 / +7.65 / +4.83 / +5.50; dec/s 30,341 -> 33,631, +10.84 %; turns/game 26.98, stalls 0, all 160 pairs split on all 16 runs, `host_calib_ms` 48-60). **Ir and wall-clock agree to 0.05 points**, which is the shape to expect when every row removes work rather than allocations — contrast the eleventh pass's allocation row, where -17.09 % Ir was worth +1.7 % at `release`. |

**The merged tip, measured.** Both sessions' work joined, callgrind on the
same fixed six-game workload, `profiling-fast --no-default-features`:

| | | before | after | |
|---|---|---|---|---|
| 2026-08-10 | **twelfth pass, both sessions, merged** | **5,620,660,987 Ir** (`81c88580`) | **4,964,563,445 Ir** (`6e4fa142`) | **-11.68 %.** Below either session's own tip (5,013,096,289 / 5,260,848,923), so the merge kept the union of the non-overlapping work and none of it cancelled. Base check: the other session's base `48ac252c` reads 5,622,084,243 against `81c88580`'s 5,620,660,987 — 0.03 % apart, confirming `81c88580` doesn't touch the engine hot path. Inclusive at the merged tip: `auto_tap_for_cost_inner` 744,864,725 (15.00 %), `computed_permanent` 706,279,518 (14.23 %), `dispatch_triggers_for_events` 534,111,146 (10.76 %), `gather_continuous_effects_inner` 516,321,501 (10.40 %), `check_state_based_actions` 313,205,944 (6.31 %, from 11.69 %), `compute_battlefield` 297,944,249 (6.00 %, from 13.51 % two passes ago). **The `release` anchor on this tip is now taken: 55.88 games/s mean of 8** — see Baseline. |

**Thirteenth pass.** Base `a4947da6` (the merged twelfth-pass tip plus the
round-27 ML commits, which don't touch the engine hot path — the re-taken
profile reproduces the merged figure to 0.03 %).

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-10 | `ComputedPermanent.colors` becomes a `ColorSet` bitmask (`59cd783e`, candidate 1(a)'s remainder) | 4,963,254,419 Ir | 4,836,659,318 Ir (**-2.55 %**) | `colors` was the one field the eleventh pass's `Printed<T>` row could not borrow: `printed_colors()` *computes* a `Vec<Color>` from `color_override` / Devoid / `color_indicator` / cost symbols, so there is nothing to project. It was **106,024,788 Ir inclusive (2.14 %) over 683,248 calls from `compute_permanent_pass`** — 155 Ir each, 302,944 of them growing a `Vec` — and the result was then cloned again on every `ComputedPermanent::clone`. `ColorSet` already existed for Commander colour identity; giving it `contains<Borrow<Color>>` / `iter` / `to_vec` / `intersects` / `FromIterator` / `Extend` / `IntoIterator` let the ~70 `cp.colors` readers compile unchanged — the `Printed<T>` / `KeywordCounters` trick, third time. `printed_colors()` survives as `printed_color_set().to_vec()`, so there is still one walker. `can_block_attacker_computed` takes a `ColorSet` instead of a `&[Color]`, which also drops two `colors.clone()` per must-block check in `declare_blockers`' CR 509.1c loops. **Colour order is now WUBRG rather than indicator-then-pip order** — strictly more deterministic, a bitmask cannot carry an insertion order into a trace — and all four golden traces are byte-identical, so nothing on the bench path read it. Callgrind A/B on identical `profiling-fast --no-default-features` binaries. Suite 18825 passed / 0 failed; clippy clean. |
| 2026-08-10 | Auto-tap skips the per-card layer pass when nothing rewrites a land type (`d95cd5ba`) | 4,836,659,318 Ir | 4,756,306,488 Ir (**-1.66 %**) | `effective_mana_abilities_with` took a `computed_permanent` per untapped permanent per auto-tap and read **one field** out of it — `subtypes.land_types`, for the two CR 305.6 consumers. Inside `mana_source_table`'s freeze scope the gather is memoized but each call is still a full per-card layer pass: **95,644,776 Ir (1.93 %) over 67,468 calls, 1,418 Ir each**, to answer "is this Forest still a Forest". Exactly three modifications write `subtypes.land_types` — `AddLandType` (Urborg), `SetLandTypes` (Blood Moon), `ReplaceBasicLandType` (Mind Bend) — so with none in scope the computed type line *is* the printed one. The test is a walk of the already-gathered effect list; outside a freeze scope `frozen_effects()` returns `None` and the old path stands rather than paying a gather to save a gather. `effective_mana_abilities_with` **161,920,619 → 89,565,572 inclusive (-44.7 %)**, `mana_source_table` 244,258,164 → 156,843,652, `auto_tap_for_cost_inner` 15.00 % → 13.78 %. **The presence flag was checked, not assumed**: `cr_305_6_auto_tap_sees_a_rewritten_land_type` pins the third writer through the frozen auto-tap path and *fails* when `ReplaceBasicLandType` is deleted from the predicate. Suite 18827 passed / 0 failed; golden traces byte-identical; clippy clean. |
| 2026-08-10 | `ManaSourceInfo.colors` becomes a `ColorSet` + a fixed `[usize; 5]` (candidate 1.5's allocation leftover) | 4,756,306,488 Ir | 4,733,001,860 Ir (**-0.49 %**) | **The smallest row on this list, and quoted as such.** `Vec<(ManaColor, usize)>` held at most five entries and was one heap allocation per untapped source per `auto_tap_for_cost` — 67,468 of them per six games, 1.8 % of the program's allocations — and `redundancy` did a linear scan of it per (source, colour, other source), i.e. quadratic in the untapped board. The bitmask makes the membership test one `and`, and `color_idx[color_index(c)]` replaces the `find`. Iteration order is unchanged (`ColorSet::iter` walks WUBRG, which is what `ManaColor::ALL` gave), and `redundancy` mins over the colours, so ordering could not leak anyway. Callgrind A/B on identical `profiling-fast --no-default-features` binaries; well under the 5 % claim bar, which is why it is quoted in instructions on a deterministic workload rather than in `--bench`. Suite 18827 passed / 0 failed; golden traces byte-identical; clippy clean. |
| | **cumulative this pass** | **4,963,254,419 Ir** | **4,733,001,860 Ir (-4.64 %)** | three alternated `profiling-fast --no-default-features` callgrind A/Bs on the one fixed six-game workload, so the three rows subtract honestly. **Wall-clock at `release` + mimalloc is a null** — see Baseline; six alternated pairs read +0.54 %, which is what a sub-5 % change looks like against this box's ±8 % spread. |

## Profile of record

Callgrind on `profiling-fast --no-default-features` (= `release-fast` opt
settings + debuginfo; system allocator, because valgrind replaces malloc and
a mimalloc build would measure the interception), 1 thread, `--a gang --b
gang --games 6 --seed 1 --decks fixed`. **Retaken 2026-08-10 on the merged
twelfth-pass tip `a4947da6`** — the re-take NEXT had been owing since the
two concurrent sessions joined. The number reproduces the recorded merged
figure to 0.03 % (4,963,254,419 against 4,964,563,445), which is this
profile's build-to-build noise, so the merge measurement stands.

**4.96 G instructions for six games**, from 5.62 G at the twelfth pass's
start, 6.15 G at the eleventh's, and 14.40 G six passes ago on the same
workload. **3,748,803 allocations**, from 4,073,937.

Self cost, grouped:

| share | site | note |
|---|---|---|
| 19.3 % | `_int_malloc` 5.49 / `_int_free` 4.80 / `malloc` 3.56 / `free` 2.14 / `malloc_consolidate` 0.92 / arena+merge+unlink 2.34 | **the largest single block, and it has grown as a share every pass because no row has ever attacked it head-on.** Ir over-weights it — see the allocator note above — but it is the only theme left at this size. |
| 6.6 % | `gather_continuous_effects_inner` | `mod.rs` 150 M / `ptr::non_null` 97 M / `slice::iter` 68 M / `vec` 51 M. 516 M inclusive over 243,190 gathers = **2,123 Ir/gather** — the per-call cost the tenth pass moved and the eleventh/twelfth did not. |
| 3.29 % | `__memcpy_avx_unaligned_erms` | 163 M |
| 2.90 % | `Arc::clone_from_ref_in` | 109 M `ptr` + 52 M `raw_vec` + 40 M `uint_macros`. **802,482 of the program's 3.75 M allocations** sit under it — the CoW unshares deep-copying the collections inside `CardData` / `PlayerData`. |
| 1.72 % | `compute_permanent_pass` | 86 M |
| 1.30 % | `hashbrown RawTable::clone` | 353,862 allocations |
| 1.21 % | `Vec::clone` | 239,240 allocations |
| 1.07 % | `GameState::clone` | own fields only; the zone unshare is the `Arc` row above |
| 0.47 % | `CardDefinition::printed_colors` | **removed by the ColorSet row** — kept here as the before |

Inclusive, in caller terms (these overlap each other):

| Ir | share | calls | site |
|---|---|---|---|
| 3,501,254,529 | 70.54 % | 64 k | `perform_action` |
| 744,365,168 | 15.00 % | 8,892 | `auto_tap_for_cost_inner` — **now the largest named consumer**, and only ~1/3 of it is the tapping itself (`activate_ability` 4.55 % + 4.29 %) |
| 706,741,290 | 14.24 % | | `computed_permanent` |
| 534,140,634 | 10.76 % | 52 k | `dispatch_triggers_for_events` |
| 516,339,194 | 10.40 % | 243,190 | `gather_continuous_effects_inner` |
| 412,685,108 |  8.31 % | | `compute_permanent_pass` |
| 359,977,808 |  7.25 % | | `drop_in_place<GameState>` — the transaction checkpoint's other half (candidate 0.5) |
| 313,182,377 |  6.31 % | 10,670 | `check_state_based_actions` — was 11.69 % two passes ago (candidate 0.25) |
| 297,647,165 |  6.00 % | 17,718 | `compute_battlefield` — was 13.51 %, and its call count fell 47,808 → 17,718 |
| 244,258,164 |  4.92 % | 8,892 | `mana_source_table` |
| 200,930,662 |  4.05 % | | `GameState::clone` |
| 161,920,619 |  3.26 % | 67,468 | `effective_mana_abilities_with` — **removed 44.7 % of by the land-type gate row** |
| 130,398,805 |  2.63 % | 59,378 | `permanents_with_abilities_removed` — gathers once per call to answer one bit |
| 106,024,788 |  2.14 % | 683,248 | `CardDefinition::printed_colors` — **removed by the ColorSet row** |
| 87,836,252 |  1.77 % | | `CardInstance::clear_end_of_turn_effects` |
| 84,286,316 |  1.70 % | | `CardInstance::deref_mut` — the CoW unshare point |
| 73,944,552 |  1.49 % | | `same_team` |

**The allocation tree**, 3,748,803 allocations for six games by allocating
call site: `Arc::clone_from_ref_in` **802,482 (4.60 % of the program)**;
`RawVecInner::finish_grow` 657,705 (1.53 %) — `Vec::push` growth;
`SpecFromIterNested::from_iter` 258,284 (1.24 %); `RawTable::clone` 353,862;
`computed_permanent` 340,310; `GameState::clone` 264,202; `Vec::clone`
239,240; `gather_continuous_effects_inner` 199,072.

**Where the `collect()`s are**, by inlining site — this is the map for the
next three candidates, and each row is a `Vec` materialized and thrown away:
`compute_battlefield` 224 M / 4.71 % over 17,718 calls (12,641 Ir each);
`bot::cast_candidates` 169 M / 3.55 % over 7,024 calls (**24,040 Ir each**);
`mana_source_table` 146 M / 3.07 % over 8,892; `check_state_based_actions`
140 M / 2.95 % over **82,634 collects, i.e. 7.7 per sweep**;
`fire_step_triggers` 63 M / 1.32 %.

## Perf candidates

Ordered by expected value. Each run pulls the top one, attaches numbers,
and feeds what it finds back in. Re-profile and replenish when the list
goes thin or stale.

**The profile of record is current** — retaken 2026-08-10 at `a4947da6`,
the thirteenth pass's base. Shares quoted below without a date are from it.
Two of its rows have since been paid (`printed_colors`,
`effective_mana_abilities_with`); everything else is live.

Methodological notes, each learned the hard way:

- **The shape that paid the eleventh pass: a `&mut self` path taking a
  `computed_permanent` outside a freeze scope.** Each one re-gathers *every
  continuous effect in the game* (~2,900 Ir then, ~2,100 now) to answer one
  question about one card, and they cluster: `do_untap` asked twice per
  permanent, `activate_ability` + `_inner` four times per activation,
  `scale_damage_to` twice per call. Four rows, **-5.74 % between them**,
  gathers from `computed_permanent` 260,370 → 151,776. **How to find the
  next one: `--tree=caller` on `computed_permanent`, divide each caller's
  inclusive Ir by its call count.** >2,000 per call means unfrozen — it is
  gathering, and deduplicating it is worth a gather each. A few hundred
  means the caller is already inside a scope and only memo *hits* are left;
  deduplicating those is worth ~200 Ir each, which is how the land-mana row
  got costed at 2 % and returned 0.26 %.
- **Check whether LLVM already hoists the scan — but check it on *your*
  build.** Three per-card `any()` walks of the effect set inside
  `apply_layers`' whole-board loop read **-0.007 %** when hoisted by hand on
  the tenth pass's base (self cost 31,937,700 — already hoisted) and
  **-0.63 %** on the branch point's base (self cost 74,510,172 — not
  hoisted). Same source, same profile, two codegen outcomes. The tell is a
  function whose *self* cost is tiny even though the source reads O(n x m);
  read the tell, don't assume the answer. The hoist is not in the tree.
- **Build-to-build noise on this profile is ~0.02 %.** Two links of
  `87d76144` read 6,150,469,969 and 6,151,471,423 (and a third, from a
  different session on a different container, 6,151,455,670). Nothing under
  ~0.1 % should be claimed from a single A/B pair.
- **Read the caller rows under `malloc`, not the function totals.** The
  eleventh pass's `do_untap` row (4.33 % of the program) never appears in a
  self-cost list at all — its cost is entirely in callees, and what named it
  in the second session was a `spec_from_iter.rs:do_untap` row sitting third
  in `--tree=caller` under `__rust_alloc`. The self-cost list is a list of
  *leaves*; the work is in the callers.
- **A CoW group pays only while it stays unwritten.** Grouping 90 rarely
  written collections behind one `CowBox` was -0.93 %; widening the same
  group to 126 by adding the per-resolution scratch was **+1.23 %**, because
  one big unshare on nearly every action costs more than the empty clones it
  replaced. Group size x unshare probability < sum of individual clone costs.

- **The dominant shape in this engine is "a per-card loop rebuilds a
  board-level scan".** Four of this run's five rows were that one shape and
  they were -10.0 % between them. The tell in the profile is a function
  whose cost sits in `slice/iter/macros.rs` + `ptr/non_null.rs` rather than
  its own file, and whose caller tree shows it called tens of thousands of
  times. **When a scan is hoisted into a `_with` variant, grep the `_for`
  shim's callers for a surrounding loop** — the trigger half of candidate 2
  had had its `_with` variant for a run and was still O(cards²), because
  three loops still called the shim.
- **A CoW handle pays off in proportion to how many siblings share the
  unshare.** Twenty cards in a zone with one written: -25.6 %. Two seats
  with one written: -2.2 %. Count the siblings before costing the next one.

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
- **A representation change can beat a dozen call-site fixes.** Candidate 7
  read as "audit 79 `iter_mut()` sites"; the fix that landed was one type
  (`CardInstance` = `Arc<CardData>` + `Deref`/`DerefMut`) touching three
  files and two borrow-checker conflicts, for **-25.6 %** instructions. When
  a cost is *structural* — here, "every write to any card deep-copies the
  whole zone" — look for the type that makes the whole class impossible
  before enumerating its instances.

**After the twelfth (whole-board-pass) pass.** Four rows, -6.43 % Ir /
+9.57 % wall. The shape they shared: **a `&mut self` entry point taking a
whole-board `compute_battlefield` to read one bit, or taking several where
one would do.** How to find the next one: `--tree=caller` on
`compute_battlefield` and read the call counts against how many times the
caller can actually run. `declare_attackers_banded` showed 9,928 calls over
~2,600 attack declarations — the ratio *is* the bug. What is left in that
list, in order: `check_state_based_actions` (10,670 / ~7,500 sweeps — still
>1 per sweep, find the second one), `declare_blockers` (7,754),
`resolve_combat` (3,196), `process_cumulative_upkeep` (1,742), `do_phasing`
(1,718). The last three look like one pass per turn each, i.e. legitimate.

**After the thirteenth (representation + presence-gate) pass.** Three rows,
-4.64 % Ir. The shapes they shared with the last three passes: **a hot
struct field that is computed and allocated when it could be a bitmask**,
and **a caller that materializes a whole computed view to read one field of
it**. The re-taken profile promotes the next three by size, and all three
are the *same* shape one level up — a `collect()` whose result is mostly
thrown away:

- **(0) `bot::cast_candidates` allocates 169 M Ir / 3.55 % in
  `collect()` over 7,024 calls — 24,040 Ir per call, the most expensive
  single collect site in the program.** Never profiled at line level. Read
  `--auto=yes` on it before guessing: the affordance sweep it used to carry
  is gone, so this is the candidate *list* itself.
- **(0.1) `compute_battlefield` materializes 224 M Ir / 4.71 % of `Vec`
  over 17,718 calls** (12,641 Ir each) — candidate 4(a), unchanged in
  substance but now costed: does each caller need all ~19.5
  `ComputedPermanent`s, or an iterator, or two of them?
- **(0.2) `permanents_with_abilities_removed` runs a full gather 59,378
  times** (115 M / 2.42 % from `dispatch_triggers_for_events` alone) to
  answer one bit that is `false` on every bench board. `c365ede8` already
  made it bail off the gathered set; what is left is *the gather*. The
  land-type gate above is the pattern — find a cheap over-approximation of
  "could a `RemoveAllAbilities` be in scope" that doesn't need the gather,
  and enumerate every way one can enter (a resolved `continuous_effect`, a
  static ability converted during the gather) or it will silently keep
  abilities a Turn to Frog took away.

**Top of the list**, in order:

- **(A) `compute_battlefield` is still the biggest layer consumer** —
  759 M inclusive (13.50 %) over 47,808 calls at 15,870 Ir each at the
  twelfth pass's base; the pass took ~7,300 of those calls out via
  `declare_attackers_banded`, `finalize_cast` and the two combat helpers.
  Re-profile before costing the rest. That is candidate 1(c)'s successor and
  candidate 4(a): does each caller need all ~19.5 `ComputedPermanent`s
  materialized, or two of them? Caller list is under candidate 1(c) —
  `declare_attackers_banded`, `check_state_based_actions`, `declare_blockers`,
  `finalize_cast` are the top four, and the SBA one is a genuine whole-board
  read (lethal damage over every creature). The 1,718 calls the `do_untap`
  row *added* are the good kind: one board pass replacing 39 single-card ones.
- **(B) The allocator, 18.4 % and growing as a share.** Named contributors,
  each measurable before any work: `GameState::clone` + `drop_in_place`
  (candidate 0.5, ~10 % between them); the CoW unshare's ~785 k `CardData`
  deep copies (`Arc::clone_from_ref_in` 2.81 % self); and `printed_colors`,
  1,277,508 `Vec<Color>` allocations, one per computed permanent — candidate
  1(a)'s leftover, wanting a `ColorSet` bitmask. **Ir over-weights all of
  these**: the `Printed<T>` row read -17.09 % Ir and +1.7 % at `release` +
  mimalloc. Cost them with a `release` A/B, not callgrind alone.
- **(C) The remaining unfrozen `computed_permanent` callers**, by
  Ir-per-call from `--tree=caller` (>2,000 means it is gathering):
  `permanent_is_creature` (its `activate_ability` caller is fixed; what is
  left comes from `effects/mod.rs`'s `DamageEachCreaturePerAura` and
  `actions.rs`'s two type checks), then the combat-damage cluster —
  `apply_prevention_shields`, `creature_redirects_damage_to_controller`,
  `damage_from_source_prevented_by_keyword`, `resolve_combat`, all ~4,400
  Ir/call at ~4,450 calls each. **Read the CR 510.2 warning in item 0 before
  touching the cluster**: a scope around the *apply loop* is a rules change,
  a scope inside one helper call is not.

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
   (d) ~~**Combat's whole-board reads**~~ — **done, -0.91 % between
   `has_first_strikers`, `bands_with_other_qualities` and the
   `declare_attackers_banded` hoist (-1.97 %, its own row).** What is left:
   `declare_blockers` (7,754 calls) takes exactly one pass and reads only
   the attackers and blockers through two closures — the same shape, but
   cost the participant count against the board size first; and
   `resolve_first_strike_damage` / `resolve_combat` genuinely want the whole
   board (CR 510.2, damage is simultaneous) — leave them.
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

0.25 **`check_state_based_actions` — 6.31 % at the thirteenth pass's base,
   down from 11.69 %.** The number that names what is left:
   **140,479,219 Ir (2.95 %) in `spec_from_iter.rs:check_state_based_actions`
   over 82,634 collects — 7.7 per sweep at ~1,700 Ir each**, after the
   presence scan already removed ~21 empty ones. These are the walks that
   *find* something. Cost them with `--auto=yes` before gating; an empty
   `collect()` does not allocate, so what is left is real work or a
   whole-board walk that could be a participant walk.
   *(historical, from when this was 9.93 %)*
   ~7,500 sweeps at **~87,600 Ir each**, and it is the shape the gather fold
   (-9.02 %) and `do_untap` (-4.08 %) both paid out on: ~20 unconditional
   whole-board walks for state triggers no bench board carries
   (`flip_when_predicate`, `sacrifice_when`, `state_trigger`,
   `sacrifice_and_burn_when_stolen`, `sacrifice_when_you_control_no_other`,
   the `WhileSourceTapped` / `WhileSourceAttached` effect sweeps, …).
   **(a)** ~~"more than one whole-board pass per sweep — find the second
   one"~~ — **there is no second one; the sweep count was wrong.** At the
   twelfth pass's tip `compute_battlefield` runs 10,670× from the sweep and
   `effective_life` runs 21,502× = 2 per sweep, so there are **~10,750
   sweeps, not ~7,500** — i.e. exactly one pass each, and it is the CR 704.5g
   lethal-damage/toughness read, which genuinely wants the whole board. The
   other site (the `flip_when_has_keyword` guard) never fires on the bench.
   **What is actually left in the sweep**: its own code is only 6.2 M
   (0.12 % — a leaf-free function), `compute_battlefield` is 156 M (2.96 %),
   and **178 M (3.23 %) is 227,678 `Vec` collects inlined into it — ~21 per
   sweep at ~780 Ir each**, the remaining whole-board walks. Note an *empty*
   `collect()` does not allocate, so these are the walks that find
   something; cost them with a line-level `--auto=yes` run before gating.
   **(b)** ~~the steal-penalty disarm and the CR 704.8 ±1/±1 snapshot~~ —
   **done, -1.82 %.** What is left of the item: the other ~18 whole-board
   walks the sweep does unconditionally (`flip_when_predicate`,
   `sacrifice_when`, `state_trigger`, `sacrifice_when_you_control_no_other`,
   the `WhileSourceTapped` / `WhileSourceAttached` sweeps). Each wants the
   same treatment — find the latch or the printed predicate its reader
   already uses and gate on that, not on the board. Read the gather-fold
   trap in candidate 3 before gating any loop's *head*.

0.5 **Stop taking the transaction checkpoint where nothing recovers from
   it.** `perform_action` clones the state before every action and drops the
   clone after: **2.9 % + 6.6 % = ~9.5 % of the program** at the eleventh
   pass's tip (373 M drop + 161 M clone — flat in absolute terms since
   `ColdState`, so the share climbs as everything else falls), over 64,248
   actions of which **20** rolled back. `ColdState` took the half
   that came from walking empty collections; the rest is irreducible while
   the snapshot is unconditional. Two routes, neither taken yet:
   **(a)** the bot's `simulate_attack_outcome_once` loop is where nearly all
   the actions are — it calls `perform_action` and, on `Err`, retries
   `PassPriority` and bails. It could call `perform_action_inner` and take
   one explicit checkpoint only around the calls that are allowed to fail,
   which is a bot change, not an engine one.
   **(b)** make the residual `GameState` narrow enough that the snapshot is a
   memcpy plus a handful of `Arc` bumps — that means CoW-ing `players`
   (written on most actions, so count the siblings first: two seats) and
   folding the ~45 remaining collections into a *second*, hotter group whose
   unshare is still cheaper than 45 empty clones. The +1.23 % row says the
   naive version of (b) loses; a two-group split is untested.

1. **Make `ComputedPermanent` cheap to build.**
   (a) ~~Hold the `Arc<CardDefinition>` and clone a collection only when a
   layer writes it~~ — **done, -17.09 % Ir / +13.5 % wall.** ~~What is left
   of the item: `colors` is still cloned per call~~ — **also done, -2.55 %
   Ir, 2026-08-10: `ColorSet` bitmask, see the Log.** The reasoning the item
   carried (a cache on `CardDefinition` is unsound because
   `Arc::make_mut(&mut card.definition)` mutates a uniquely owned definition
   in place — MDFC face-swap, "loses all abilities", keyword grants) is why
   the fix had to be a representation change rather than a memo; keep it in
   mind before caching anything else on a definition.
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

1.5 **`effective_mana_abilities` clones every ability it returns** — the
   `Cow` row fixed the printed half and the **2026-08-10 land-type gate
   removed 44.7 % of what was left** (161,920,619 -> 89,565,572 inclusive).
   What survives at 1.88 %, over 67,468 calls (1,327 Ir each):
   `battlefield_find`'s linear scan, `granted_abilities_with` (15.1 M),
   `intrinsic_land_mana_abilities_with` (3.0 M), the `frozen_effects` lock
   and the `out` `Vec`. ~~`ManaSourceInfo.colors: Vec<(ManaColor, usize)>`~~
   — **done, -0.49 %, see the Log.**
   The original text, still true of the allocation half: **Do not re-try freezing it**: a
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

1.7 ~~**`Player` is the last non-CoW checkpoint cost.**~~ **Done, -2.21 % Ir.**
   See the Log row. It was worth an eighth of what candidate 7 was worth on
   the same shape, and the reason generalises: **a CoW handle pays off in
   proportion to how many siblings share the unshare.** Twenty cards in a
   zone and one written is a 20x saving; two seats and one written is 2x, and
   half of that 2x is all this returned. `GameState::clone` still halved
   (967 M -> 474 M inclusive) — the new `Player::deref_mut` line is 326 M
   against the 494 M saved. **Before costing the next CoW candidate, count
   the siblings.**
   What the row also settled, and it cost a build to learn: **gating a `&mut`
   to skip an unshare is usually worthless.** `empty_mana_pools` was the
   single biggest unshare source (153 M, 1.96 %) doing nothing on 51 k step
   changes; a read-only fast path in front of it returned **-0.04 %**,
   because the seat it declines to unshare is unshared by the next writer in
   the same checkpoint anyway. Gate a `&mut` for clarity, not for
   instructions — unless the checkpoint really has no other writer.

2. **The dispatcher's per-card trigger gathering.** With the
   `compute_battlefield` call gone, the remaining per-card work in
   `dispatch_triggers_for_events` is `statics_granted_triggers_for` (2.78 %
   self before the fix) plus `granted_triggers` and
   `equip_granted_triggers_for`, each called once per battlefield card per
   dispatch — O(cards²) against `all_static_sources`. Hoist the
   "which sources carry a `GrantTriggeredAbility`" scan out of the per-card
   loop, the same shape as the layer-gather filter that won +27.8 %.
   **The activated-ability half of this is done, -2.58 %** — see the
   `GrantScan` Log row, which is the exact pattern this item asks for. The
   triggered half already has its board-level list (`trigger_grant_sources` +
   `statics_granted_triggers_with`), so what is left here is the *cost of
   building that list*: `trigger_grant_sources` is **199 M (2.6 %) spread
   over `option.rs` / `mut_ptr.rs` / `ptr::non_null.rs`**, i.e. all walking,
   and it is rebuilt per dispatch. Two reads: does every dispatch need it
   (gate on the event set), and can the board walk be shared with
   `grant_scan`, which now walks the same sources for the activated half.
3. **The gather — still #1 among engine functions, now 9.93 % self.**
   **This run took another -4.37 % out of it** by listing every *top-level
   statement* in the function instead of grepping for
   `for card in &self.battlefield`: the `AnthemForFilter` walk was still
   chaining the whole battlefield (now `sa_cards`), and Leyline of
   Singularity's presence check was the Coat of Arms shape verbatim. Gather
   self 885,832,058 -> 610,672,182.
   **What is left in it, in order:**
   - **The `GraveyardAnthem` pass walks both graveyards unconditionally**
     (`for player in &self.players { for card in &player.graveyard { … } }`),
     ~30 cards late game x 358 k gathers. There is no cheap presence gate —
     the sources are *in* the graveyard — so this one wants either a cached
     per-definition flag or candidate 4.
   - The ungated `for card in &self.battlefield` at the Unleash / CR 611.2
     block stays ungated on purpose (read its comment and last run's trap
     before touching it); splitting it would also reorder emission.
   - `all_effects` starts as a clone of `continuous_effects` and grows by
     repeated `push`/`extend` — reserve once.
   - ~40 separate `for &card in &sa_cards` loops, i.e. ~40 loop set-ups per
     gather. **Ruled out already**: the `u64` presence mask over them read
     +1.43 % and was reverted.

   *(historical, kept for the numbers it settled)* **back to #1, and now by a wide margin.** 918 M self
   (13.1 %), 1,550,667,623 inclusive (**22.0 %**), 358,792 calls =
   **2,559 Ir/gather**. Untouched this run, so its absolute is unchanged and
   only its share moved. **This is the top item for the next run.** What the
   line-level attribution settled, and what is left:

   - **The 39 per-variant passes are NOT where the cost is — measured
     twice, don't redo it.** The `u64` presence mask over them read
     +1.43 % and was reverted (Log). The attribution says why: on a bench
     board `sa_cards` is near-empty, so all 39 are loop set-ups.
   - **It was the eleven whole-battlefield walks**, at 5.93 % of the
     program in `slice::iter` plus 5.57 % in `ptr::non_null`. Folding them
     into the `sa_cards` walk took the gather from 4,385 to 2,774 Ir/call.
   - **Still open, in order:** the gather is *still* the #1 engine
     function at 995,310,108 Ir (12.3 % of the new 8.09 G) over the same
     358,792 calls. `mod.rs`'s own lines are now 184,286,054 and
     `option.rs` 146,755,592 — i.e. the remaining cost is the passes'
     bodies, not the walking. The next reads are the *first* loop, which
     calls `static_ability_to_effects(card, …)` per static-ability card
     and gets a freshly allocated `Vec<ContinuousEffect>` back to `extend`
     from (make it push into `&mut all_effects`); `all_effects` itself,
     which starts as a clone of `continuous_effects` and grows by repeated
     `push`/`extend` (reserve once); and the emblem / graveyard /
     `all_static_sources` loops, which run whether or not anything is
     there — same shape as the eleven just folded, one zone out.
   - **The trap this cost, worth reading before the next gate.** The
     Unleash loop is not a single pass: it also carries the CR 611.2
     `WhileCondition` predicate gate, its sibling, and the suspect /
     living-metal statics. Gating its head killed every threshold /
     retype / conditional-keyword static in the catalog — 15 `classic_sets`
     failures, all green again once only the two keyword scans were gated.
     Read a loop's whole body before gating its head; nine of the eleven
     were single-purpose, that one was not.
   - After this, candidate 4 (memoizing the gather) is the only structural
     move left on it, and candidate 1.7 is worth more.

4. **Memoize the gather outside freeze scopes.** Unchanged: the blocker is
   invalidation, not caching. **Promoted by this run's profile** — with every
   per-card-loop-rebuilds-a-scan site fixed, the layer system is the program
   (`computed_permanent` 23.0 % inclusive, the gather 22.0 %,
   `compute_battlefield` 11.8 %, heavily overlapping), and 627 M of
   `compute_battlefield`'s 833 M is `Vec::from_iter` materializing a fresh
   `ComputedPermanent` for the whole board on each of its 46 k calls. Two
   sub-items now worth costing separately: **(a)** does
   `compute_battlefield` need to *materialize*, or can its callers take an
   iterator / ask for the two permanents they read (item 0's question, one
   level up); **(b)** the memo itself. `compute_battlefield` alone runs 46,090 times
   per six games and `gather_continuous_effects_inner` allocates 367,900
   times. Two designs: a mutation epoch
   bumped at every `&mut GameState` entry point with the field set made
   private behind accessors; or route zone mutation through `CowBox`'s
   `DerefMut` and derive validity from `Arc` identity. Multi-run project.
5. **`Keyword::eq` (0.77 % self, 1,277,508 calls from
   `compute_permanent_pass`)** — linear scans of `Vec<Keyword>`. A bitset
   for the ~64 common keywords makes `has_keyword` O(1) and shrinks
   `CardData`; rides along with item 1.
6. **`HashMap` hash choice** — `block_map`, `combat_damage_order` /
   `_assignment` use SipHash; `hashbrown RawTable::clone` is still 531,520
   allocations after the CoW row.
7. ~~**CowBox sharp edge / per-card CoW.**~~ **Done, -25.6 % Ir /
   +27.8 % wall-clock.** See the Log row. The fix was not the
   `iter_mut()` audit this item described: `CardInstance` became
   `Arc<CardData>` with `Deref`/`DerefMut`, so `DerefMut` is the one
   unshare point and a read-only `iter_mut()` no longer copies anything but
   pointers. The 79 `iter_mut()` sites are now harmless and need no audit.
   What is left of the sharp edge: `CowBox<Vec<…>>` on the *non-card* zones
   (`stack`, `continuous_effects`) still deep-copies its elements, but
   neither shows in the profile.
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
  allocations. **The reading was right about the tallies and wrong about
  the conclusion** — the checkpoint cost was never in `GameState`'s own
  fields, it was in the zone *unshare* that followed, which candidate 7
  finally removed. `GameState::clone` is now 329,100 of 8.72 M.
- *The bot's affordance sweep* — `cast_candidates` calling
  `compute_hand_affordances` for one field. Fixed (+42 % fixed decks,
  +52.7 % sealed). The lesson generalises and is item 0 above.
