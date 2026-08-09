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

Re-anchored (`release`, mimalloc — the shipped configuration) at the
2026-08-09 gather commit, on the same box as the previous anchor
(`host_calib_ms` 45-51 against its 45-49, so the hosts are comparable).
**45.72 -> 48.74 games/s mean, +6.6 %**, alongside a measured -10.05 %
instructions; RSS 24.0 -> 22.4 MiB.
Refresh only alongside an intentional, explained change.
Regressions beyond ~5 % get investigated before anything else lands — but
check `host_calib_ms` first (see "How to measure").

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        46 / 51 / 45 / 48            <- compare this first
games                320
games_per_s          50.38 / 45.37 / 49.48 / 49.72   (mean 48.74, spread 10 %)
games_per_s_th       16.79 / 15.13 / 16.49 / 16.57
decisions_per_s      30423 / 27399 / 29878 / 30023   (mean 29431)
turns_per_game       26.98
stalls               0 (0.00 %)
peak_rss_mib         22.3 - 22.5
determinism          ok (all pairs split)
```

The 45.37 run is the one with `host_calib_ms` 51, the slowest probe of the
four — the spread is the box, not the code. Previous anchor for reference:
45.72 mean / 27609 dec/s / 23.9-24.2 MiB at `host_calib_ms` 45-49.

**What this run is worth, measured end to end.** The pre-run tip
(`2eee69ad`) and this one, both built `release` + mimalloc, alternated
A/B/A/B ×5 in one sitting on one box:

```text
release + mimalloc, --bench, 5 alternated pairs   <- the shipped configuration
games_per_s      34.26 -> 46.44      +35.6 %   5/5 pairs, no overlap
                 (32.95-34.93)  (45.65-47.40)
decisions_per_s  20686 -> 28045      +35.6 %
peak_rss_mib      41.0 ->  23.8      -42 %
host_calib_ms    45-78 vs 45-46; turns_per_game 26.98 and stalls 0 on all
                 10 runs; determinism ok, all pairs split
```

That also settles the previous anchor's absolute: the old block read 33.13
on what it called a slower box, and the *same pre-run code* reads 34.26
here — so the box difference is ~3 %, and 33.13 → 45.72 is almost all
code.

**The per-change rows in Log are measured differently and don't add up to
+35.6 %**, deliberately — each was an alternated `release-fast` + system
allocator sitting, which is the only way to resolve a small change here:

```text
release-fast + system allocator, callgrind, fixed six-game workload
14,403,731,176 -> 10,718,206,071 Ir   CardInstance CoW handle       -25.59 %
10,718,206,071 ->  8,886,099,152 Ir   ComputedPermanent Printed<T>  -17.09 %
                                      cumulative this run           -38.31 %
16.7 M -> 5.09 M allocations          cumulative                    -69 %

release + mimalloc, --bench, alternated pairs   <- the shipped configuration
45.73 -> 46.49 games/s                Printed<T>, 4/4 pairs          +1.7 %
34.26 -> 46.44 games/s                both rows, 5/5 pairs          +35.6 %
```

**The two allocators disagree on purpose, and the gap is the lesson.**
`Printed<T>` is an allocation-count fix: -17.09 % Ir and +13.5 %
wall-clock against the *system* allocator, **+1.7 %** against mimalloc,
which is what ships. glibc's `malloc` is expensive enough that removing
3.5 M of them looks enormous; mimalloc's is cheap enough that the same
3.5 M were most of a rounding error. The `CardInstance` CoW row is the
other shape — it removes deep struct copies, not just allocations — and
carries essentially the whole +35.6 %. Measure allocation-shaped changes
at `release` before quoting a throughput number for them.

Older steps, kept because the percentages carry even though the absolutes
don't: 13,307,099,945 -> 13,052,911,075 Ir for the `computed_permanent`
memo (-1.91 %) and -> 12,235,211,102 for the mana-walker freeze (-6.26 %,
+4.5 % wall). Those two ran on the 2.10GHz box and do not subtract against
this run's Ir figures — a different build of the same tree read 14.40 G
here, so only same-sitting pairs are comparable.

The instruction counts are the same fixed six-game workload throughout, so
the -38.31 % subtracts honestly; the wall-clock figures do not, which is the
whole reason this run measured in Ir. Nothing regressed on the correctness
side: `turns_per_game` held at 26.98 and `stalls` at 0 across all 22 bench
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
| 2026-08-09 | `CardInstance` becomes a CoW handle: the ~110 fields move to `CardData` behind an `Arc`, `Deref`/`DerefMut` keep every `card.field` read and write working, and `DerefMut` is the single unshare point (candidate 7) | 14,403,731,176 Ir; 25.44 games/s, 15351 dec/s | 10,718,206,071 Ir (**-25.59 %**); 32.50 games/s (**+27.8 %**) | The zones were already `CowBox<Vec<CardInstance>>`, so a `GameState` clone was cheap but the *first write* deep-copied every card in the zone: `Arc::make_mut` was **24.12 %** inclusive and `CardInstance::clone` **14.69 %** self, 15.85 % of the program under `advance_step` alone. Now unsharing a zone copies N pointers and only the written card clones. Callgrind A/B on identical `release-fast --no-default-features` binaries over the fixed six-game workload; `--bench` ×4 alternated pairs, 4/4 positive and non-overlapping (base 25.12-25.85, cand 31.88-32.91). Peak RSS **fell** 23.3 → 21.3 MiB (shared cards aren't duplicated). Suite 18810 passed / 0 failed; all four golden traces byte-identical; `turns_per_game` 26.98 and `stalls` 0 on all 10 bench runs; determinism ok, all pairs split. Cost: two borrow-checker fixes where a `&mut` field write and a `&self` read of the same card overlapped. **Under `release` + mimalloc this row carries essentially the run's whole +35.6 %** (see Baseline): unlike the `Printed<T>` row it removes deep struct copies, not just allocations, so the allocator can't absorb it. |
| 2026-08-09 | Gate each of the gather's 38 per-variant passes on a `u64` presence mask built in one walk of the board's static abilities (candidate 3(ii)) | 8,886,099,152 Ir | 9,013,111,944 Ir (**+1.43 %**) | **No win — reverted.** The gather's own self cost went *up*, 1,573 M -> 1,696 M (+7.8 % of itself): the 38-arm classifier costs ~342 Ir per gather and the passes it skips were already near-free. The negative result is the useful part — it says the 39 passes are not where the gather's 4,385 Ir/call lives, so the next attempt should profile the body line by line (`--profile profiling`, which keeps debuginfo) rather than guess again. Written up in candidate 3. |
| 2026-08-09 | `ComputedPermanent`'s four printed-derived collections (`card_types`, `supertypes`, `subtypes`, `keywords`) become `Printed<T>` — the `Arc<CardDefinition>` plus a projection, cloned only on the first layer write (candidate 1(a)) | 10,718,206,071 Ir; 32.61 games/s (system alloc); 45.73 games/s (`release`, mimalloc) | 8,886,099,152 Ir (**-17.09 %**); 37.02 games/s (**+13.5 %**, system alloc); **46.49 games/s (+1.7 %, `release` + mimalloc — the number that ships)** | `compute_permanent_pass` was **3,482,320 of the program's 8,723,045 allocations (40 %)**, nearly all of them cloning a collection that nothing then modified. `Printed<T>` `Deref`/`DerefMut`s to `T`, so the ~4.5 k `cp.keywords.contains(…)` / `cp.subtypes.creature_types` sites are untouched; the whole change is 11 `.clone()` → `.to_vec()` fixups plus gating the two unconditional `keywords.retain` calls (they take `&mut`, so they materialized the list even when they removed nothing). Result: **compute_permanent_pass allocations 3,482,320 → 30,086**, program-wide 8,723,045 → **5,093,895 (-41.6 %)**, allocator self cost 21.4 → 10.9 %. Callgrind A/B on identical `release-fast --no-default-features` binaries, fixed six-game workload; `--bench` ×4 alternated, 4/4 positive and non-overlapping (before 31.94-33.31, after 35.44-37.66). Peak RSS 21.3 → 23.7 MiB — the one cost, four `Arc<CardDefinition>` clones per computed permanent keeping definitions alive. **The two wall-clock figures disagree and the mimalloc one is the real one**: the +13.5 % was measured against the *system* allocator (`--no-default-features`, which callgrind forces), and a 4/4-pair alternated A/B of the two `release` + mimalloc binaries reads **45.73 → 46.49 games/s, +1.7 %** (paired deltas +0.76 / +0.03 / +0.79 / +1.46, `host_calib_ms` 45-58). Kept: 4/4 positive under the shipped configuration, an exact -17.09 % of the work, and no cost but 2 MiB. See the new allocator note under "How to measure". Suite 18810 passed / 0 failed; all four golden traces byte-identical; turns_per_game 26.98, stalls 0, determinism ok on all 16 runs. |
| 2026-08-09 | Fold the gather's eleven whole-battlefield walks into one: the walk that builds `sa_cards` also sets a presence flag per pass, and each pass iterates an empty slice when its flag is clear. Bludgeon Brawl's `ArtifactsAreEquipment` scan is hoisted out of its per-card loop (candidate 3) | 8,887,218,012 Ir; gather self 1,573,494,744 | 8,085,908,260 Ir (**-9.02 %**); gather self 995,310,108 (**-36.7 %**) | **The line-level attribution the previous attempt asked for, taken on the new `profiling-fast` profile.** It says the gather's 4,385 Ir/call is not in the 39 per-variant arms at all: **527,365,400 Ir (5.93 % of the program) is `slice::iter` and 494,753,454 (5.57 %) is `ptr::non_null`** — ~65 % of the gather is raw slice iteration and pointer advance, against 179,561,212 in `mod.rs` itself. That is the *eleven separate `for card in &self.battlefield` loops*, four of which scanned a `Vec<Keyword>` per card unconditionally, plus `brawl_equip_mv` — which re-scanned every card's static abilities *per battlefield card*, 7,140,444 `is_artifact` calls for six games. Now one walk with one keyword scan per card, and each pass is skipped wholesale. Gather calls unchanged (358,792), so Ir/gather is 4,385 → 2,774; `is_artifact` + `is_creature` self 222,027,450 → 114,920,790. **Why this worked where the presence mask didn't**: that one gated already-empty walks over a short `sa_cards` and paid a 38-arm classifier; this one gates eleven walks of the *whole* board and pays one `u32`-ish flag set per card. Callgrind A/B on identical `profiling-fast --no-default-features` binaries (= `release-fast` + debuginfo, same opt settings — the baseline's gather self reproduces PERF's recorded 1,573,494,744 exactly), fixed six-game workload. Suite 18627 passed / 0 failed; all four golden traces byte-identical. **One trap paid for**: the Unleash loop also carries the CR 611.2 predicate gate, its sibling, and the suspect / living-metal statics — gating that loop on `any_unleash` silently killed every `WhileCondition` static (15 failures across `classic_sets`). It stays ungated; only its two keyword scans are gated. Check a loop's whole body before gating its head. |
| 2026-08-09 | `effective_mana_abilities` returns `Cow<'_, ActivatedAbility>` so printed mana abilities are borrowed from `card.definition` instead of deep-cloned, and `granted_abilities_for` does its linear `battlefield_find` once instead of twice (candidate 1.5) | 8,085,908,260 Ir | 7,993,961,114 Ir (**-1.14 %**) | `granted_abilities_for` 404,837,864 → 388,335,872 (**-4.1 %** of a function that had never been touched and is 272,334 calls per six games). Only three callers, all of which read `.effect` and the cost fields, so `Cow` costs nothing at the call sites — `Deref` covers `mana_source_cost_rank(first)`. Callgrind A/B on identical `profiling-fast --no-default-features` binaries, fixed six-game workload; under the 5 % claim bar, which is exactly why it is quoted in instructions rather than `--bench`. Suite 18627 passed / 0 failed; all four golden traces byte-identical. Cumulative for the two rows this run: **8,887,218,012 → 7,993,961,114, -10.05 %**, and at `release` + mimalloc — the shipped configuration, ×4 runs on the same box as the previous anchor (`host_calib_ms` 45-51 vs 45-49) — **45.72 → 48.74 games/s mean, +6.6 %**, decisions/s 27609 → 29431, peak RSS 24.0 → 22.4 MiB, `turns_per_game` 26.98 and `stalls` 0 on every run, determinism ok. **Baseline** is re-anchored there. |
| 2026-08-09 | `Player` becomes a CoW handle: the ~165 fields move to `PlayerData` behind an `Arc`, `Deref`/`DerefMut` keep every `player.field` read and write working, and `DerefMut` is the single unshare point (candidate 1.7) | 7,994,965,799 Ir | 7,818,537,433 Ir (**-2.21 %**) | Candidate 7 one level up, and the ceiling is much lower for the reason the shape implies: there are **two seats, not twenty cards**, and one of them is written on essentially every checkpoint. `GameState::clone` inclusive **967,524,311 → 473,827,424 (-51.0 %)** and `drop_in_place<GameState>` 441,814,641 → 396,985,432, against a new `Player::deref_mut` line at **326,104,839 (4.17 % of the program)** — i.e. the change removes the *other* seat's deep copy and nothing else. Six borrow-checker fixes, all one kind: a `&mut` field borrow now borrows the whole seat, so three loops re-borrow per zone, `manifest_card` scans library-then-hand instead of chaining, and `find_card_anywhere_mut` locates with shared borrows before taking the one `&mut` (which also stops it unsharing every seat on a miss). Snapshot wire format unchanged — `Player`'s serde impls are transparent over `PlayerData`. Rider measured at **-0.04 %**: `empty_mana_pools` gets a read-only fast path (plus `ManaPool::is_empty()`, stricter than `total() == 0`) so it stops taking `&mut` on 51 k step changes with nothing to do; kept for the sharp edge it documents, not the instructions — **the deep copy it skips is mostly paid by the next writer in the same checkpoint**, which is the general lesson for CoW gating. Callgrind A/B on identical `profiling-fast --no-default-features` binaries, fixed six-game workload. `core_rules` 1664 passed / 0 failed; golden traces byte-identical. |
| 2026-08-09 | `granted_abilities_for` looks its own card up once instead of eight times, and classifies the five "has the activated abilities of …" statics in one pass | 7,818,537,433 Ir | 7,696,302,458 Ir (**-1.56 %**) | The function ran `battlefield_find(card_id)` — a linear scan of the whole battlefield — once at the top and again inside each of seven blocks, all resolving to the same card (the top one early-returns when the card is off the battlefield, so the later seven could never differ). It is called ~233 k times per six games from three per-card loops, so the redundant scans multiply. `granted_abilities_for` **411,928,730 → 291,885,476 inclusive (-29.1 %)**, its `ptr::non_null` line 120,941,056 → 79,449,414 (-34.3 %) — pointer advance, the same signature the gather fold read. Push order into `out`, which indexes `GameAction::ActivateAbility`, is unchanged. `core_rules` 1664 passed / 0 failed; golden traces byte-identical. |
| 2026-08-09 | `GrantScan`: the board-level half of `granted_abilities_for` is hoisted out of the per-card loops (candidate 2's shape, applied to activated rather than triggered grants) | 7,696,302,458 Ir | 7,497,680,035 Ir (**-2.58 %**) | What was left in `granted_abilities_for` after the row above was **four whole-board walks per call**: `all_static_sources()` (battlefield + command zones) for `GrantActivatedAbility` statics, both graveyards for `GrantActivatedAbilityFromGraveyard`, and two more battlefield passes for soulbond pairs and attached `equipped_bonus`. None of those depend on *which* permanent is asking — including the CR 611.2 `active_static` unwrap and the Hellbent-style `condition` predicate, both of which read only the source. `grant_scan()` collects them in one pass and `granted_abilities_with(card_id, &scan)` does the per-card half. Threaded into the five per-card loops that were paying it: `mana_source_table_inner`, `untapped_relevant_source_exists_inner`, `untapped_producers_of_inner`, `bot::available_mana`, and `bot::usable_abilities` (six call sites). `granted_abilities_for` keeps its old signature by building a scan of its own, so the ~18 k non-loop callers are unaffected. Emission order into `out` is preserved exactly — each scan `Vec` is filled in the source-iteration order the old inline loop used. Callgrind A/B on identical `profiling-fast --no-default-features` binaries. Full suite green; golden traces byte-identical. |
| | **cumulative this run** | **7,994,965,799 Ir** | **7,497,680,035 Ir (-6.22 %)** | three alternated `profiling-fast --no-default-features` callgrind A/Bs on the one fixed six-game workload, so the three rows subtract honestly |

## Profile of record

Callgrind on `profiling-fast --no-default-features` (= `release-fast` opt
settings + debuginfo; system allocator, because valgrind replaces malloc and
a mimalloc build would measure the interception), 1 thread, `--a gang --b
gang --games 6 --seed 1 --decks fixed`. Retaken 2026-08-09 at the
whole-board-walk-folding commit, i.e. *after* every row in the Log.

**8.09 G instructions for six games**, from 8.89 G at the `Printed<T>`
commit and 14.40 G at this run's start on the same workload. The
`profiling-fast` baseline reproduced the `release-fast` gather figure to
the instruction (1,573,494,744 both), so the two profiles' numbers
subtract; `--profile profiling` (LTO) does not — it read 13.84 G where
`release-fast` read 14.40.

Self cost, grouped (share at the `Printed<T>` commit in brackets):

| share | site | note |
|---|---|---|
| 12.3 % | `gather_continuous_effects_inner` | [17.7 %] 995,310,108 Ir over the same 358,792 calls = **2,774 Ir/gather**, from 4,385. Still #1, but the walking is gone: `slice::iter` 527 M → 203 M, `non_null` 495 M → 242 M, while `mod.rs`'s own lines went *up* 180 M → 219 M (the fused walk). What is left is pass bodies. |
| 12.1 % | `_int_malloc` 4.30 / `_int_free` 4.05 / `malloc` 2.94 / `free` 1.78 | [10.9 %] unchanged in absolute terms — the share rose because the total fell |
| 5.01 % | `granted_abilities_for` | [4.77 %] 404,837,864 Ir, **byte-identical before and after** — now the #2 engine function and the obvious next pull |
| 3.69 % | `__memcpy_avx_unaligned_erms` | |
| 2.75 % | `compute_permanent_pass` | its *allocations* are gone, not its arithmetic |
| 1.48 % | `CardInstance::counter_count` | |
| 1.30 % | `hashbrown RawTable::clone` | candidate 6 |
| 1.02 % | `trigger_grant_sources` | candidate 2 |

**The costs the per-line attribution named, in caller terms** (these are
inclusive of callees and so overlap each other):

| Ir | share | site |
|---|---|---|
| 768,854,629 | 8.65 % | `GameState::clone`, 64,248 calls = **11,967 Ir per checkpoint** |
| 488,350,327 | 5.49 % | └ of which `players: self.players.clone()` alone — candidate 1.7 |
| 319,829,660 | 3.60 % | `drop_in_place<GameState>` |
| 160,860 | — | `battlefield`, for contrast: a `CowBox` `Arc` bump |

(Shares from the pre-fix profile, where the total was 8.89 G; the absolutes
are unmoved by this run's fix, which touched neither.)

**The read after this run:** state checkpointing is now the largest single
cost in the program — `GameState::clone` plus its drop is ~12.3 % and
`players` is nearly half of it, for exactly the reason candidate 7 named one
level down. That is candidate 1.7. The gather is still #1 as a single
function but is no longer the whole story, and `granted_abilities_for` at
5 % has never been touched.

## Perf candidates

Ordered by expected value. Each run pulls the top one, attaches numbers,
and feeds what it finds back in. Re-profile and replenish when the list
goes thin or stale.

**The profile of record is current** (2026-08-09, at the `Printed<T>`
commit) — see that section. Every share below is from it. Three
methodological notes, each learned the hard way:

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

1. **Make `ComputedPermanent` cheap to build.**
   (a) ~~Hold the `Arc<CardDefinition>` and clone a collection only when a
   layer writes it~~ — **done, -17.09 % Ir / +13.5 % wall.** See the Log
   row. `compute_permanent_pass` went from 3,482,320 allocations to 30,086.
   What is left of the item: **`colors` is still cloned per call.**
   `CardDefinition::printed_colors()` *computes* a fresh `Vec<Color>`
   (from `color_override` / Devoid / `color_indicator` / cost symbols), so
   there is nothing to borrow — it can't be a `Printed<T>` without caching
   the result on the definition, and a cache there is **unsound as written**:
   `Arc::make_mut(&mut card.definition)` mutates a *uniquely owned*
   definition in place (MDFC face-swap, "loses all abilities", keyword
   grants), which would leave a filled cache stale. A `ColorSet` bitmask
   return would dodge both, at the cost of touching every `cp.colors`
   reader. Worth ~0.5 M of the remaining 5.09 M allocations, which after this run's
   allocator lesson means **~0.2 % at `release`** — do it only if it falls
   out of other work, not on its own.
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

1.7 **`Player` is the last non-CoW checkpoint cost — the top item after the
   gather.** The 2026-08-09 line-level attribution reads
   `GameState::clone` at **768,854,629 Ir (8.65 %) over 64,248 calls
   (11,967 Ir per clone)**, plus `drop_in_place<GameState>` at 319,829,660
   (3.60 %) — ~12.3 % of the program in state checkpointing. Of the clone,
   **`players: self.players.clone()` alone is 488,350,327 Ir (5.49 %)**,
   against 160,860 for `battlefield` (a `CowBox` `Arc` bump) and 160,860 for
   `exile`. `Player` is a plain struct with **165 fields** — its zones are
   already `CowBox`, but the ~30 per-turn `Vec`/`HashSet` tallies, the
   `String` name and the rest are deep-copied per player per checkpoint.
   This is candidate 7 one level up and takes the same fix: `Player` becomes
   `Arc<PlayerData>` with `Deref`/`DerefMut`, `DerefMut` the single unshare
   point, so `players.clone()` is N `Arc` bumps and only a written player
   unshares. Candidate 7 was worth -25.6 % Ir on the same shape.

2. **The dispatcher's per-card trigger gathering.** With the
   `compute_battlefield` call gone, the remaining per-card work in
   `dispatch_triggers_for_events` is `statics_granted_triggers_for` (2.78 %
   self before the fix) plus `granted_triggers` and
   `equip_granted_triggers_for`, each called once per battlefield card per
   dispatch — O(cards²) against `all_static_sources`. Hoist the
   "which sources carry a `GrantTriggeredAbility`" scan out of the per-card
   loop, the same shape as the layer-gather filter that won +27.8 %.
3. ~~**The gather.**~~ **Largely done, -9.02 % program Ir / -36.7 % of the
   gather.** See the Log row. What the line-level attribution settled, and
   what is left:

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
   invalidation, not caching. `compute_battlefield` alone runs 46,090 times
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
