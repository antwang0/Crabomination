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
# Profile the binary *in place*. `profiling-fast` sets
# `split-debuginfo = "unpacked"`, so the DWARF lives beside the binary in
# `target/profiling-fast/deps/`; copying `bot_ladder` elsewhere to keep a
# base around still counts instructions correctly but annotates every frame
# as `???:0x…`. Keep the base by `git stash`ing the source instead.
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

**Re-anchored 2026-08-14 at `76804984`** (`release`, mimalloc — the shipped
configuration), the thirty-first pass's tip. This one *does* chain to the
`5034eb2f` block below: `host_calib_ms` reads **45-46** against its 44-48,
the same mid-40s fingerprint, on the same `host_cpu` string.

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        45 / 45 / 46 / 46
games                320
games_per_s          119.49 / 121.21 / 119.62 / 120.06 (mean 120.10,
                     spread 1.44 %)
games_per_s_th       39.83 - 40.40
decisions_per_s      72,152 - 73,192
turns_per_game       26.98
decisions            193,232 byte-identical on all four
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         29.1 - 29.4
determinism          ok (all 160 pairs split, on all 4 runs)
```

**115.31 -> 120.10 is +4.2 %, and it is the three passes since the old
anchor.** Passes 29, 30 and 31 are -2.456 %, -1.990 % and -3.407 % by
instruction count (-7.6 % compounded), so wall-clock reading +4.2 % is the
expected under-delivery, not a surprise: `Ir` over-weights the allocation
and representation work these passes removed. The invariants are unchanged
— `decisions` **193,232**, `turns_per_game` 26.98, stalls 0, `peak_rss_mib`
29.1-29.4, all identical to the old block. Compare against this block only
when `host_calib_ms` is in the mid-40s.

**The wide check, and why a recorded constant is not one.** `--decks all
--games 300 --threads 3` reads **2,553,880** decisions, `turns_per_game`
21.32, **2 draws (0.04 %)**, determinism ok — and the *base* binary
(`a58447d9`, rebuilt and run in the same sitting) reads exactly the same.
The figures this file and TODO carried, 2,548,986 and 6 draws, match
neither and are from a superseded configuration. **Run the base binary;
do not diff against a number.**

**The superseded `5034eb2f` block.** Re-anchored 2026-08-13 (`release`,
mimalloc), the twenty-eighth pass's tip. The previous anchor
(`ed4c152c`, 2026-08-11) had gone seven passes stale and was on a third
box; it is kept below the new block because its *ratios* still read.

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        44-48 across the sitting   <- within-sitting only
games                320
games_per_s          115.66 / 115.78 / 115.38 / 108.85 / 114.32 / 115.40
                     (mean 114.23, spread 6.07 %; the five without run 4
                     are 114.32-115.78, mean 115.31, spread 1.27 %)
games_per_s_th       36.28 - 38.59
decisions_per_s      mean 68,979
turns_per_game       26.98
decisions            193,232 byte-identical on all six
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         29.0 - 29.4
determinism          ok (all 160 pairs split, on all 6 runs)
```

**This block does not chain to either older one and is not a +19 % claim.**
`host_calib_ms` reads **44-48** against the `ed4c152c` anchor's 47-52 on a
2.10 GHz part and against 51-70 on the other 2.80 GHz block — a third
fingerprint, i.e. a third box. The pass is worth **-7.257 %** by
instruction count and nothing in it can produce 19 %. Use this block only
against a run reporting `host_calib_ms` in the mid-40s.

**Cross-checked 2026-08-14 at the twenty-ninth pass's tip `645b978d`, and
it does *not* re-anchor.** `release`, mimalloc, three runs: `games_per_s`
88.33 / 84.03 / 83.29, `games_per_s_th` 27.76-29.44, `decisions_per_s`
50,296-53,336, `peak_rss_mib` 29.2-31.3. **`host_calib_ms` reads 58 / 90 /
56 against this block's 44-48** — a fourth fingerprint, ~25 % slower on the
probe, which is the whole of the 114.2 -> 85.2 gap. The invariants are what
this run proves: `decisions` **193,232 byte-identical** to the block below
and to every run since, `turns_per_game` 26.98, `stalls` 0, `determinism
ok` on all three. The pass is worth **-1.993 % by instruction count** and
nothing in it can produce -25 %. Do not chain these numbers to the block
above; re-measure on a mid-40s-calib box before treating the baseline as
moved.

**`peak_rss_mib` 21.7-22.3 -> 29.0-29.4, and it is the host, not the
pass.** The one bench number that moved against us, so it was A/B'd rather
than explained away: `profiling-fast` + mimalloc, same box, same sitting,
**31.5 MiB with the twenty-eighth pass's library strip still in place
against 32.1 without it** — 0.6 MiB, where the delta to explain is 7. The
same tip on the *system* allocator reads **19.9 MiB**, below every number
this file has recorded for it. So the move is mimalloc's arena behaviour
on this host, not a change in what the engine holds live. Re-check it if a
future sitting on a mid-40s-calib box reads ~22.

**The superseded `ed4c152c` block, kept for its ratios.** `release`,
mimalloc, 2.10 GHz host, `host_calib_ms` 47-52: `games_per_s` 94.23 /
91.94 / 93.27 / 98.47 / 99.09 / 96.81 (mean **95.64**, spread 7.48 %),
`games_per_s_th` 30.65-33.03, `decisions_per_s` 57,748, `peak_rss_mib`
21.7-22.3, `turns_per_game` 26.98, stalls 0, determinism ok on all six.
Its own lesson, which is why it is kept: **95.64 against the 67.31 below
was +42 % and almost all of it was the host.** The twentieth pass was
worth -4.897 % by instruction count and nothing in it can produce 42 %;
the tell was `host_cpu` reporting a different model and `host_calib_ms`
47-52 against 53-70 — a nominally slower part running the probe ~25 %
faster, i.e. a different machine, not a quieter hour on the same one.
*Read the calib column before any absolute.* The Ir numbers do chain: the
pass's base rebuilt read 3,694,337,730 against the nineteenth pass's
recorded 3,694,708,603 (-0.010 %).

**The five cross-checks that ran against `ed4c152c` without refreshing it
are in git** (`git log -- PERF.md`); each was rejected for one of three
reasons worth keeping: a sitting whose spread was wider than the effect
(`f2fb6722`, 14.8 % against a 1.4 % change), *a tight sample that was the
bottom of the distribution rather than a regression* (`1112e709` — three
tight runs read 90.03, eight idle ones 96.53; **a spread smaller than the
effect is the trap, not the reassurance**), and a run on the wrong profile
or the wrong box (`ac8e3b50` at `release-fast`, `247ee13d` at
`profiling-fast`).

**Re-checked 2026-08-14 at the thirtieth pass's merged tip `a4960740`, and
it still does not re-anchor.** `host_calib_ms` reads **58-69** — the fourth
fingerprint again, not the mid-40s box this block was taken on — so no
absolute here is comparable. What the run proves is the invariants, and all
of them hold at the tip: `--bench` 320 games reads `decisions` **193,232
byte-identical**, `turns_per_game` **26.98**, `stalls` **0** (`cap 0 /
stuck 0 / draw 0`), `determinism ok` on all 160 pairs. The wide pool at the
tip — `--bench --threads 1 --games 300 --seed 11 --decks all`, 5,100 games
over 17 archetypes — reads `decisions` **2,548,986**, identical to
`841dd40b`, `5034eb2f` and the twenty-ninth pass's tip, with the recorded
**6 draws / 5,100 (0.12 %)** and `determinism ok`. **The pass is worth
-1.990 % by instruction count and moved zero decisions on either pool**,
and both pools were re-run after the merge, not before it.
(`peak_rss_mib` 20.4 on this binary is the *system* allocator — the
`--no-default-features` profiling build — not a comparison to the
mimalloc numbers above.)

**Re-checked 2026-08-14 at the thirty-first pass's tip `62e6dd42`, and it
still does not re-anchor.** `host_calib_ms` reads **68** on the fixed pool
and **91** on the wide one against this block's 44-48 — the slow
fingerprint again, and on a 2.10 GHz part this time — so no absolute here
is comparable and none is recorded. Both runs are the
`--no-default-features` `profiling-fast` binary (system allocator), which
is a third reason its 123.43 games/s and 20.3 MiB RSS do not belong beside
the `release` + mimalloc block above. What the run proves is the
invariants, and every one holds after **-6.241 % by instruction count**:
`--bench` reads `decisions` **193,232** byte-identical, `turns_per_game`
**26.98**, `stalls` **0** (`cap 0 / stuck 0 / draw 0`), `determinism ok` on
all 160 pairs; `--bench --threads 1 --games 300 --seed 11 --decks all`
reads `decisions` **2,548,986** over 5,100 games and 17 archetypes —
identical to `841dd40b`, `5034eb2f`, `645b978d` and `a4960740` — with
`turns_per_game` 20.99, the recorded **6 draws / 5,100 (0.12 %)** and
`determinism ok` on all 2,547 pairs.

**Crash-freedom at the merged tip.** The `overflow` profile (release-fast +
`overflow-checks`) over `--a gang --b gang --games 400 --threads 3 --decks
all` on seeds 11/12/13: **20,400 games, 20,392 decided, no panic and no
arithmetic overflow**, 76-89 s a seed. The 8 undecided are all on seed 11
and are rules draws, the same rate the pool's other invocations show.

**The wide-pool checks the fixed-deck anchor cannot make, re-run at
`645b978d`.** The twenty-ninth pass's strongest behaviour proof is a
**base-vs-tip diff, not a recorded constant**: the pre-pass binary
(`797040ba`, kept from the first callgrind of the sitting) and the tip both
run `--a gang --b gang --games 300 --threads 3 --decks all` and print
**byte-identical output over 5,100 games across 17 archetypes** — every
archetype's record, 2,549 paired splits, `rho -1.000`, and the same 2
undecided in the same `cube RG` bucket. Keeping the old binary around costs
one `cp` and answers in three minutes what a recorded decision count
answers only if the invocation matches; do it every pass.

**Correction to the recorded stall figure.** This file recorded `--decks
all` at "6 draws / 5,100 (0.12 %)"; the command above reads **2 undecided /
5,100 (0.039 %)** at *both* tips, so the 6 belongs to a different
invocation (the `--bench --threads 1 --seed 11` form, whose `decisions`
2,548,986 is the number that block is really about). Two figures for one
pool with no invocation attached is how that happens — quote the command.

Crash-freedom: the `overflow` profile (release-fast + `overflow-checks`)
over `--games 400 --threads 3 --decks all` on seeds 11/12/13 —
**20,400 games, 20,392 decided, 8 undecided (0.039 %), no panic and no
arithmetic overflow**, ~85 s per seed against a 16-minute build. Cheap
enough to run every pass; do.

**What every one of them agreed on, which is the part that matters**:
`turns_per_game` **26.98** (now eleven consecutive anchors), `stalls`
**0** with `stalls_by` reading `cap 0 / stuck 0 / draw 0`, `determinism
ok` on every run with all 160 pairs split and `rho -1.000`, `decisions`
**193,232 byte-identical**, `decisions_per_game` 603.9. All five still
hold at `645b978d`.

~~**The `--decks fixed` bench is exactly reproducible and the wider pools
are not**~~ — **fixed 2026-08-11 (`841dd40b`)**. Every pool now reproduces
on a fixed seed: `--bench --threads 1 --games 300 --seed 11` reads decisions
**1,130,728** (cube, 3 runs), **2,548,986** (all, 2 runs), **684,268** (sos,
2 runs), `determinism ok` on all seven, against cube's 1,129,690 /
1,130,785 / 1,130,706 and two FAILs before. `--decks all`'s stall rate is a
stable **0.12 %** (6 draws / 5,100 games, `cap 0 / stuck 0`) where it used
to move run to run — and **2 undecided / 10,200 games (0.02 %)** on
`--games 600 --seed 20250808` (2026-08-12), i.e. it is rules draws at a
seed-dependent rate and there is nothing to fix. No number in this file moved — all of them are
`--decks fixed` or the six-game callgrind workload — but the wider pools are
now usable as measurements.

**The host fingerprint can disagree with itself, and the calibration probe
is the half to believe.** One container reported `host_cpu` *2.80 GHz* — the
`4f3e86c0` box — while `host_calib_ms` read **46-62**, overlapping the
2.10 GHz box's 47-52 and not the 2.80 box's own 53-70. That is why the probe
exists: compare `host_calib_ms` before comparing absolutes, and never chain
across two blocks whose probes don't overlap.

**Previous anchors, compacted to a table.** The full `--bench` blocks are in
git (`git log -- PERF.md`); every one of them was taken on the 2.80 GHz box
at `release` + mimalloc, 320 games, and **`turns_per_game` reads 26.98 and
`stalls` 0 in all of them**, which is the only column that chains.

| tip | pass | games/s (mean, spread) | calib | RSS |
|---|---|---|---|---|
| `4f3e86c0` | 19 | 67.31 (1.95 %) | 53-70 | 22.1-22.5 |
| `56986d65` | 18 | 71.29 (7.3 %) | 50-62 | 22.0-22.2 |
| `6ed3dbfc` | 17 | 69.13 (6.6 %) | 49-85 | 21.8-22.4 |
| `abb2b502` | 16 | 81.93 (9.2 %) | 45-55 | 21.7-22.2 |
| `28629ba9` | 15 | 64.19 (7.1 %) | 52-64 | 23.3-23.8 |
| `3e2ee6cb` | 14 | 62.48 (5.9 %) | 50-60 | 21.7-22.1 |

The lessons those six anchors were written to carry, which are the reason
the table above is safe to compress:

- **Consecutive anchors do not subtract.** 81.93 -> 69.13 -> 71.29 -> 67.31
  is four containers, not four regressions; the `calib` column moves with
  them and the instruction counts fall monotonically across the same span.
- **A -5.6 % anchor gap was checked, not asserted, and cost fifteen
  minutes.** At `4f3e86c0` the tip's two perf commits were reverted, both
  sides rebuilt `profiling-fast --no-default-features`, and the binaries
  alternated `--bench` in one sitting: base **56.55** against tip **57.58**,
  **+1.82 % paired, 4/6 pairs positive**. (A seventh pair was discarded —
  `host_calib_ms` read **525**, i.e. something else had the box.)
- **Keep the base binary and re-run it rather than reasoning about
  `host_calib_ms` alone.** At `3e2ee6cb` the base read 52.38 and, an hour
  later, 52.18 on the same box — ~0.4 % within-sitting drift, which is what
  made that pass's 6/6 paired **+11.10 %** trustworthy against an anchor gap
  pointing the other way. That pass agrees with its -11.56 % instruction
  count to half a point: the shape to expect when a change removes work *and*
  allocations.
- **An LTO confound is real and cheap to rule out.** With `codegen-units = 1`
  + thin LTO a 335-line addition elsewhere in the engine crate can move
  inlining crate-wide. Re-running callgrind on the *merged* tip at
  `6ed3dbfc` read +0.014 % against the pass's own tip, i.e. inside build
  noise.
- **70.65 and everything older belong to a different bench.** They predate
  `998b2433` making `EvalWeights::default()` carry `determinize: 1`, and
  `--bench` runs `gang` = `EvalWeights::default()`, so the *workload* changed
  underneath them.
- **Absolutes do not transfer between containers, and one pass proved it
  twice.** The same engine code read 60.49 and 64.42 in two sittings on one
  box (+6.5 % of pure drift), and a second container read 55.70 on its own
  tip where the first read 55.88 on a tip with 1.0 % *fewer* instructions.
  **Quote a paired A/B measured in one sitting, never a difference of
  anchors.**


## Log

**Passes one to nine, compacted to an index** — one line per pass. The rows'
prose is in git; what is worth keeping is which lever each pulled, so a
later run recognizes a lever already spent. Absolutes across these are
wall-clock `--bench` on three different boxes and do not chain.

| pass | levers | result |
|---|---|---|
| 1 | Gate the layer gather's graveyard tallies on a `dynamic_pt` (`b17a76b`); run the whole `RandomBot` tick in one `with_frozen_layers` scope (`e919496`) | 10.36 -> 12.22 games/s (**+18.0 %**) |
| 2 | mimalloc as an opt-in `#[global_allocator]` (**+12.0 %**, RSS 25.3 -> 39.0 MiB); filter the gather's 39 static-ability battlefield passes through one precomputed slice (`80086059`, **+27.8 %**); mimalloc becomes the default (`bbf5ddcc`, **+21.9 %** — larger than the opt-in row because the gather fix removed the work the allocator cost hid behind) | 9.64 -> 14.49 games/s (**+50.3 %**) |
| 3 | `dispatch_triggers_for_events` stops running `compute_battlefield()` for one bool per card, and `permanents_with_abilities_removed` answers from the gathered set (`c365ede8`, **+8.2 %**); hoist the dispatcher's two grant scans out of its per-permanent loop (`f87974c3`, **+3.9 %**) | 19.76 games/s, **+12.4 %** |
| 3 | **No win — reverted.** CoW-wrapping the per-turn / per-game tally collections (cast-name / id / profile logs, ETB + death lists, delayed triggers, graveyard + discard sets) read 20.12 -> 20.10 over 8 alternated pairs. The negative result is the point: per-clone allocation traffic is *not* in these collections, which is what sent that run to the caller tree | — |
| 4 (2nd box) | `cast_candidates` asks `spliceable_hand_cards_on` against the probe template instead of calling `compute_hand_affordances` for one field (`489bb1d3`) | 16.08 -> 22.83 games/s (**+42.0 %**) |
| 4 | **No win — reverted.** Hoisting `available_mana` out of `cast_candidates`' per-hand-card `can_afford_in_state` filter read 22.83 -> 23.03 (+0.9 %, distributions overlap). The asymptotics were real, the constant is not: the bench hand is ~7 cards over a 3-6 permanent board. Left here so it is not re-derived | — |
| 5-9 | The representation pass, in one sitting: `CardInstance` becomes a CoW `Arc<CardData>` handle (**-25.6 %** on its own — the row that taught "look for the type that makes the class impossible"); `Player` the same; a `u64` presence mask gating the gather's 38 per-variant passes; the gather's eleven whole-battlefield walks folded into one; `ComputedPermanent`'s four printed-derived collections; `effective_mana_abilities` returning `Cow`; `GrantScan` hoisting the board-level half of `granted_abilities_for`; a per-freeze-scope `computed_permanent` memo; freeze scopes around the attack/block sim helpers (`836059e2`, **+5.0 %**) and the three read-only mana walkers; card-name tallies keyed on `&'static str` (`eb5f661c`); `compute_permanent_pass` reading counters in one pass; Coat of Arms' gather pass | **7,994,965,799 -> 6,538,441,281 Ir (-18.22 %)** |

Passes ten to thirteen, compacted to an index — same treatment as one to
nine. The rows' prose is in git (`git log -S` on the commit hashes); what is
kept below is the lever and the lesson.

| pass | base -> tip | levers, and what each taught |
|---|---|---|
| 10 | 6,497,854,664 -> 6,150,469,969 Ir (**-5.35 %**) | `GameState`'s cold tail (90 fields) behind one `CowBox` + `Deref` (`69f3a94b`, -0.93 %); the gather's `AnthemForFilter` walk and Leyline presence check iterate `sa_cards` instead of the battlefield (`f1908d4f`, **-4.37 %**); `keyword_counters` becomes an insertion-ordered `Vec` (`86670250`, a determinism fix). **Two no-wins worth not re-deriving**: hoisting `compute_permanent`'s three CR 613.8 gate scans read -0.007 % because *LLVM had already hoisted them* — the tell is a tiny self cost under an O(n x m) source; and widening the `ColdState` group to 126 fields read **+1.23 %**, which fixes the rule *group size x unshare probability < sum of the individual clone costs*. |
| 11 | 6,151,455,670 -> 5,620,794,622 Ir (**-8.63 %**) | One shape, four rows: **a `&mut self` path taking a `computed_permanent` outside a freeze scope**, each re-gathering every continuous effect in the game to answer one question about one card (`do_untap`, `scale_damage_to`, `activate_ability_inner`'s prelude, the CR 106.12 mana-source flag). Plus `usable_abilities` returning `Cow` so printed abilities are borrowed (`bab861cf`). Build-to-build noise on this profile was pinned here at **0.016 %**. |
| 12 | 5,620,660,987 -> 4,964,563,445 Ir (**-11.68 %**, two concurrent sessions merged at `6e4fa142`) | **A `&mut self` entry point taking a whole-board `compute_battlefield` to read one bit, or several where one would do**: `declare_attackers_banded` (`a21da084`), `finalize_cast` (`622b43ae`), `has_first_strikers` / `bands_with_other_qualities` (`a37863a0`); and the SBA sweep's ~20 rare whole-board passes riding one presence pass (`2038bb59`, `129a1b0e`), its death sweep skipping non-creatures (`f3c8670c`), three sites reusing a layer view they already held (`0045cbc0`). The two sessions overlapped, so their separate cumulative figures must not be added — only the merged tip is authoritative. |
| 13 | 4,963,254,419 -> 4,733,001,860 Ir (**-4.64 %**) | **A hot struct field that is computed and allocated when it could be a bitmask**: `ComputedPermanent.colors` -> `ColorSet` (`59cd783e`, -2.55 %) and `ManaSourceInfo.colors` -> `ColorSet` + `[usize; 5]` (-0.49 %); plus auto-tap skipping the per-card layer pass when nothing rewrites a land type (`d95cd5ba`, -1.66 %). **The trick that made the first two cheap to land, third time now** (`Printed<T>`, `KeywordCounters`, `ColorSet`): give the new representation the old one's API — `contains` / `iter` / `to_vec` / `FromIterator` / `IntoIterator` — and ~70 call sites compile unchanged. **And the presence flag was *checked*, not assumed**: `cr_305_6_auto_tap_sees_a_rewritten_land_type` fails if any one of the three `land_types` writers is dropped from the predicate. |

Passes fourteen to sixteen, compacted to an index — same treatment as one
to thirteen. The rows' prose is in git (`git log -S` on the commit hashes);
what is kept below is the lever and the lesson.

| pass | base -> tip | levers, and what each taught |
|---|---|---|
| 14 | 4,733,001,860 -> 4,186,040,742 Ir (**-11.56 %**); wall-clock 52.18 -> 57.97 games/s (**+11.10 %**, 6/6 alternated `release-fast` pairs) | **The transaction checkpoint**: `perform_action` cloned the whole state before every action so a rejected one could be restored, and almost nothing reads the restore. The mid-round priority pass takes none (`42c5db08`, -1.85 % — **41.6 % of a bot game's actions cannot be rejected**); the bot's dry runs take none (`831054fb`, **-11.38 %** — every probe throws its clone away on `Err`). **The correction is the audit rule** (`3e2ee6cb`, +1.77 %): `simulate_through_combat`'s torn state *is* read, by `combat_aware`'s `before` probe — so *a `dry_run` site is only sound when the caller cannot read the state after an `Err`*. And the clone is never the whole cost: it **shares every CoW zone**, so the next write deep-copies one that was uniquely owned a line earlier, which was 40 % of `Arc::clone_from_ref_in`. **Ir and wall-clock agreed to half a point** — the shape to expect when a change removes work *and* allocations. |
| 15 | 4,185,775,886 -> 4,023,920,637 Ir (**-3.87 %**) | **A presence gate instead of a gather.** `permanents_with_abilities_removed` ran a full gather per trigger dispatch for one bit that is `false` on every bench board (`a7eaa930`, **-2.54 %**); `ability_strip_in_scope` names all six routes to `RemoveAllAbilities` in its doc, the emitting blocks `debug_assert!` against the same predicates, and a debug-only cross-check re-runs the gather whenever the gate says `false` — the device every later gate in this file copies. Plus `activate_ability_inner`'s land-mana check reading the view its own scope already took (`88a0b787`, -1.36 %). |
| 16 | 4,021,875,017 -> 3,948,056,772 Ir (**-1.836 %**) | **Ask the cheap, selective question first.** `cast_candidates`' fourteen specialty blocks gate on one warm walk per zone (`52f4311a`, -0.226 % — and the negative result that the block walks are only ~13 M of self cost, so the next attempt on that function belongs in the plain-cast `flat_map` at 5.44 %); `team_of` reads the singleton-team layout instead of scanning every team's member list (`4772369a`, **-1.208 %**); nine opponent-static battlefield scans test the printed ability before the team (`abb2b502`, -0.410 %). Across the last two, `same_team` **58,406,112 -> 5,693,328, -90.3 %**. |

Passes seventeen to nineteen, compacted to an index — same treatment as one
to sixteen. The rows' prose is in git (`git log -S` on the commit hashes);
what is kept below is the lever and the lesson. The chain: `8ca7df9f`
rebuilt read 3,948,115,609 against the sixteenth pass's recorded
3,948,056,772 (0.0015 % apart); `26b5d2c7` read 3,812,623,112 against
3,817,208,224 (-0.12 %); `10cdbe63` read 3,768,870,942 against
3,768,577,483 (+0.0078 %). All rows are callgrind on `profiling-fast
--no-default-features`, the fixed six-game workload, bench output
byte-identical.

| pass | base -> tip | levers, and what each taught |
|---|---|---|
| 17 | 3,948,115,609 -> 3,817,208,224 Ir (**-3.316 %**) | **Dead work behind a gate the function already holds.** Activation skips the grant/intrinsic scans for a printed ability index (`b4a016b5`, -0.581 % — 18,386 grant scans, and a land tapping for mana is index 0); the cleanup sweep stops unsharing permanents with nothing to clear (`15a9cce6`, -0.698 % — 844,428 `DerefMut`s on a CoW `CardData`, almost all of them already clear); protection asks the *target* for a protection keyword before taking five `computed_permanent`s on the source (`96129f68`, -0.731 % — 45 % of the program's `computed_permanent` calls); activation takes one whole-game gather instead of two (`6ed3dbfc`, **-1.346 %** — the second read sat on a `&mut self` path outside any freeze scope). |
| 18 | 3,812,623,112 -> 3,768,577,483 Ir (**-1.155 %**) | **A per-card question only the board can answer `yes` to is a board-level flag with a per-card call attached.** The trigger dispatcher's three per-card grant lookups ride board-level presence gates — `statics_granted_triggers_with` was 34,049,232 Ir over 945,812 calls at **36 Ir each**, walking two empty slices. *How to find the next one*: `--tree=calling` on a hot loop body, list the callees whose per-call cost is tiny and whose call count is `permanents x batches`, and ask what each answer depends on. **Plus a no-win worth not re-deriving** (`SimBases`, +0.083 %): caching `sim_start_state` across a candidate loop. *Before caching a pure function across a candidate loop, ask what fraction of its cost is work the candidates would each have done regardless* — here the two `PlayerData` unshares are paid either way, eagerly or lazily. |
| 19 | 3,768,870,942 -> 3,694,708,603 Ir (**-1.968 %**) | **A whole-board `compute_battlefield` whose consumers are all `find(id)` lookups is a participant computation with a presence gate on whichever consumer is not.** `declare_blockers` stops re-deriving computed views it already holds (`42f59829`, -1.021 %); combat damage computes the participants, not the board (`4f3e86c0`, -1.008 % — ~23 layer passes for ~4 participants, with `butcher_orgg_divides_damage_among_defenders` as the checked gate on the one whole-board consumer). New helper `compute_permanents(&[CardId])`. **Plus `CounterBag`** (`df87c2d1`, +0.051 %) — no win, kept as a determinism fix; its empty-table conclusion is corrected by the twenty-first pass below. |

Passes twenty to thirty, compacted to an index — same treatment as one
to nineteen. The rows' prose is in git (`git log -S` on the commit hashes);
what is kept below is the lever and the lesson. All rows are callgrind on
`profiling-fast --no-default-features`, the fixed six-game workload, with
bench output byte-identical unless the row says otherwise. The chain, base
rebuilt against the previous pass's recorded tip each time: -0.010 %,
+0.009 %, +0.003 %, -0.009 %, +0.035 %, -0.016 % — i.e. these absolutes do
chain, and the spread across containers is the 0.02 % build noise.

| pass | base -> tip | levers, and what each taught |
|---|---|---|
| 20 | 3,694,337,730 -> 3,501,374,248 Ir (**-5.223 %**) | **"One pass per turn" is not a reason to keep a whole-board layer pass — it is a reason nobody looked.** `declare_attackers_banded` (`31116d43`, -1.733 %) and `declare_blockers` (`911cf298`, -1.028 %) gate their whole-board passes on a subset plus a keyword presence check; the three per-turn passes the nineteenth pass wrote off as legitimate — `process_cumulative_upkeep`, `do_phasing`, `do_untap` — were each ~23 layer passes building an empty set (`ed4c152c`, **-2.216 %**, and the win exceeds the three site costs because the freeze scope folds each gather in with the pass it gates); the dispatcher's four delayed-trigger scans ride one `is_empty` (`c7bdd850`, -0.343 %). New `board_keyword_in_scope` / `board_keyword_matching`. `compute_battlefield` calls **5,488 -> 310**. *The filter*: a `compute_battlefield` (or per-card `computed_permanent` loop) whose `filter` names one `Keyword` variant. |
| 21 | 3,501,692,629 -> 3,424,021,668 Ir (**-2.218 %**) | **Count the clones and ask whether the collection is ever non-empty at clone time.** `ColdState`'s 15 id sets become `Vec`-backed `IdSet` (`271c7d14`, -0.418 %, ~22 Ir per empty-table clone — *not* ~0, which corrects the nineteenth pass's `CounterBag` conclusion); `died_card_snapshots` becomes the insertion-ordered `IdMap` (`ea8cc1fd`, -0.278 % **on one field**, four times the sets' per-field rate because it is populated on every death — and a determinism fix, since a candidate's position decides stack order). Plus `auto_tap`'s inner loops hoisting constants and dropping five-key `HashMap`s for `[u32; 5]` (`f2fb6722`, -0.622 % — two of the three rebuilds carried a comment asserting the opposite), `auto_tap` building its source table only when it will tap (`1ec589d1`, -0.807 %), and payment's cost relaxation borrowing instead of cloning (`7c75fb94`, -0.110 %). |
| 22 | 3,423,919,639 -> 3,362,421,936 Ir (**-1.796 %**) | **A hot path that answers a structural question about an `Effect` by *serializing* it.** `activate_ability_inner` picked its tap-another target by `serde_json::to_string`-ing the effect tree and substring-searching it, on all 36,698 activations (`df35df04`, -0.727 %); `serde_json::ser::to_vec` was 0.61 % of a run that writes no JSON. *How to find the next one: `--tree=caller` on `malloc`, and read the caller names for anything that has no business allocating.* Plus `empty_mana_pools` gating its three board scans on the seats (`ef731ecc`, -0.709 %) and damage triggers building their grant set once per event rather than per kind (`1112e709`, -0.372 %). **A no-win kept as dead-work removal** (`084e4126`, -0.030 %, excluded from the total): *count the calls at the line you are changing, not at the function* — 28,552 allocations under the function, 1 M Ir at the line. |
| 23 | 3,361,108,555 -> 3,317,550,360 Ir (**-1.296 %**) | The kind/batch form of the file's oldest lever. `fire_combat_damage_triggers` takes `kinds: &[EventKind]` so its five kind-independent board walks run once and bucket (`08cbc9c3`, -1.051 %; the graveyard walk stays per-kind because `gy_combat_trigger_fired_this_step` is written between kinds); the dispatcher's whole delayed-trigger block moves behind one `is_empty` (`b925063c`, -0.247 %). **The ratio is the lesson**: not *scanning* bought -0.343 % and not *collecting* -0.247 %, so an empty `collect` is ~150 Ir, not ~0. Candidate (2) was costed here and **not taken** — see the candidates section. |
| 24 | 3,318,705,480 -> 3,177,885,139 Ir (**-4.243 %**) | **`cloned()` sitting *before* the `filter` that throws the clones away**, in a function every step of every turn calls (`006d5966`, **-1.885 %** — `fire_step_triggers` cloned every printed trigger on every permanent, then filtered on kind). Plus two fused-scan rows: the dispatcher's four opening board walks become one `DispatchScan` (`f28faaa0`, -1.118 %) and `fire_spell_cast_triggers`' three become the same, dropping an O(cards²) collect-then-re-`find` (`125557eb`, -0.496 %); the damage step's three read-only prefixes each take one freeze scope (`56f6623f`, -0.740 %); the SBA sweep's Hushbringer scan gates on there being a death (`41551bcc`, -0.070 %). **A fused board scan is worth writing even when each walk it replaces is individually invisible** — `creature_dies_triggers_suppressed` and `equip_granted_trigger_sources` never appear in the profile at all, and the four walks together were 1.118 %. The count that predicts the win is `walks x battlefield x call count`. |
| 25 | 3,177,885,139 -> 3,159,019,265 Ir (**-0.594 %**) | **Shape (a) is a *family*, not a damage-path item.** `check_target_legality_with_source` is `&self` end to end and read the layer system three times (Shroud, Hexproof, the Artifact Ward keyword scan); one `with_frozen_layers` around an `_inner` body was **-0.516 %** measured in isolation, with `permanent_has_keyword` 21,988,411 -> 11,808,343 Ir at an unchanged 8,328 calls (`5d4b5402`). The -0.594 % span also carries `419d2ea6` (stall instrumentation) and `15ec11c1` (`STALE_ROUNDS`), one branch per game. **The first collision this file has recorded**: a concurrent session pulled the same candidate from the same base and wrote functionally identical code, measuring -0.698 % against this container's -0.740 % — a 0.042 % cross-container spread on identical source, twice the build-to-build figure and inside the claim floor. |
| 26 | 3,162,657,064 -> 3,132,870,988 Ir (**-0.942 %**) | **A determinism defect and a perf candidate can be the same item.** The engine's hash containers get a fixed hasher (`841dd40b`): `crate::fxhash` (rustc's seedless FxHasher) behind `HashMap` / `HashSet` aliases, replacing `std`'s per-map-seeded SipHash-1-3 on keys that are one or two words — `CardId`s, seat indices, `&'static str` names — where SipHash's setup dominates. Longer-lived candidate (6) sat unpulled for a dozen passes at ~1 %; what moved it was the cube pool's fixed-seed nondeterminism making the same change mandatory. **Check the standing candidate list before designing a robustness fix.** Also carries `125108c1` (coin flips off the game RNG), which cannot move this workload. Base rebuild was **0.115 %** off the recorded tip, the widest this file has seen — a different container, and the row is nine times it. |
| 27 | 3,132,892,846 -> 3,115,598,474 Ir (**-0.552 %**) | **A helper that re-finds a card its callers hold.** `granted_abilities_of(&CardInstance)` beside `granted_abilities_with(CardId)`: the `CardId` form opened with a `battlefield_find` and its three hot callers were already iterating the battlefield — `available_mana`'s untapped-producer loop (63,656 of 201,834 calls), `effective_mana_abilities_with` (which had *just* done the same find), the bot's activatable sweep. `available_mana` **1.14 -> 0.97 %**, the `granted_abilities_*` family 1.39 -> 0.81 %. This is candidate (11)'s shape and it recurs; the base rebuild here was **0.0007 %** off, the tightest in the file. |
| 28 | 3,116,508,803 -> 2,890,337,225 Ir (**-7.257 %**) | **The CoW unshare, read off the profile by name** — four rows, all the same defect. Trigger dispatch stops building a `Vec` per permanent (`e02767aa`, -1.250 %: three `extend`s per permanent per dispatch, 945,812 times, most producing an empty vector; 155 M self, 69 M of it `alloc::vec` + `alloc::raw_vec`). The step-bounded may-play sweep gets a presence gate (`078b8cef`, **-2.189 %**: `clear_step_bounded_may_play` took `iter_mut` over every hand, graveyard, library and exile at *every step transition* for a CR 702.94 miracle window — **61.3 M / 1.97 % on one source line**, since `iter_mut` on a `CowBox` unshares and reaching a zone through `Player` unshares the whole `PlayerData` first). The affordance probe template stops stripping libraries (`2f70affb`, **-2.024 %**: the strip's comment was true when zones were plain `Vec`s, and cost a `PlayerData` unshare per player per template once they became `CowBox`es). And three periodic writes are guarded by a read (`5034eb2f`, **-1.998 %**: `cards_drawn_this_step = 0` per player per step boundary, 57.9 M / 1.96 %; `opponent_cast_spell_since_your_turn`; `tapped_land_for_mana_this_turn`). |
| 29 | 2,890,336,504 -> 2,819,346,784 Ir (**-2.456 %**) | **Stop paying for the clone, then make the clone cheaper.** "An opponent cast a spell" becomes a `u64` seat mask (`88c178f5`, -0.713 % — the twenty-eighth pass's read-guard cannot help a genuine false->true flip, and 14.5 M / 5,800 unshares survived it); `PlayerCold`, a `CowBox` group for the fifteen rarely-written heap fields of a 158-field `PlayerData` cloned 24,852 times per six games at ~3,300 Ir each (`645b978d`, **-1.289 %**, `Arc::clone_from_ref_in` inclusive **-34.5 %**, `#[serde(flatten)]` keeps the wire identical); `find_card_anywhere` walks the libraries last (`dbd3efeb`, -0.473 %). **Two corrections worth more than the rows.** The twenty-eighth pass's two handoff candidates were *one site double-counted* — a `=> …deref_mut` line names the *call* and the accessor's own definition line carries self cost separately; check Ir-per-call before ranking a hit, ~30 means the handle was already unique. And **a CoW unshare site is only removable when it is the seat's *only* write in that action**: `remove_from_hand` must unshare, but so must the eight `PlayerData` writes `finalize_cast` does on the next lines, each reading 30 Ir *because* it already paid. What is left on `PlayerData` is cost per clone, not clone count. |
| 30 | 2,832,747,493 -> 2,776,361,994 Ir (**-1.990 %**) | **The filter is the *order* of a linear scan, not its length.** The zone scan looks at the stack before both libraries (`8241d092`, -0.583 % — the cast path asks `find_card_anywhere` for spells that are *on the stack*, and those calls walked ~35 library cards a seat first); the gather's last board pass walks `sa_cards` when its three flag bits are clear (`6aee2973`, **-1.250 %**); an `_of(&CardInstance)` twin for `effective_mana_abilities` (`005c1d33`, -0.158 %, candidate (11)'s shape). **Neither large row is visible as an expensive function** — `find_card_anywhere` was 1.27 % spread over six file attributions, the gather's three lines 0.18/0.26/0.26 %. *The tell is a `for`/`find` whose first branch is the rarest case, and it is cheaper to read than to profile.* **The second collision this file has recorded, and on the top candidate**: a concurrent session landed `dbd3efeb` from the same base within the hour; the two measurements sit 0.110 % apart, which is the size of the ordering difference and not noise (the twenty-fifth pass's collision, on functionally identical source, was 0.042 %). Neither session's cumulative describes the merged tip `a4960740`, which is why it was re-measured. Candidate (13)/(5), the `Keyword` bitset, was costed here and **ranked down** — the arithmetic is in the candidates section. |

**Passes 29 and 30 were compacted to the two rows above on the thirty-first
pass** (their measurement tables are in `git log -- PERF.md`). Their live
numbers did not go with them: the `PlayerCold` field list and the
`remove_from_hand` non-candidate are in candidate (-1), the `Keyword`
bitset arithmetic in candidate (13), the zone-walk family in (11a).

**The twenty-seventh pass's negative result, and it is worth more than the
row.**
Candidate (8)'s largest named item — `can_afford_in_state`'s five
whole-board walks per call — was implemented as this file prescribed (a
`CostScan` presence gate over the four `sources x static_abilities` walks,
plus `AvailableMana` hoisted, both built once per hand sweep) and measured
**+0.066 %** (3,132,892,846 -> 3,134,973,100). Reverted. **The denominator
was wrong**: the candidate divided 12,114 checks by `cast_candidates`' 7,024
calls to get 1.72 cards per sweep, but the filter lives in
`main_phase_action_with` and `pick_combat_trick`, not in `cast_candidates`.
The scan built once per *sweep* ran **11,188 times against 12,702 checks —
1.13 cards per sweep**, so there is no duplication to hoist, and the added
`cost_scan` walk (~12 M) costs more than the gated walks saved (the four
walks are only **0.29 %** of the program; `available_mana` is 60 % of
`can_afford_in_state` on its own). **This is the second time this file has
recorded a no-win on hoisting `available_mana` out of that filter** — pass 4
measured +0.9 % wall-clock on it. *Before hoisting anything out of a loop,
count the loop's iterations at the site you are changing, not at a function
that merely appears nearby.* The measurement that did pay came from reading
what was left: `available_mana`'s cost is not the walk, it is the
`battlefield_find` inside `granted_abilities_with`.

**The pass's second measurement, which closes a direction rather than
opening one.** Candidate (0) — half the program, untouched for six passes —
has always carried the same first probe: *how often does the attack search
depart from greedy?* It was run (a temporary counter on `choose_scored`'s
index, `release-fast`, `--decks all`, 10,200 games, 110,000 searches) and
the answer is **46 %**: greedy 54.0 %, the empty declaration 35.0 %, a
greedy-minus-one 11.0 %. **Every candidate class pays for itself**, so there
is no pruning row here and the entry is now explicitly a
make-each-simulation-cheaper item. *A candidate that has sat at the top of a
list for six passes is worth an hour of measurement before it is worth an
hour of code* — this one cost one instrumented build and a two-minute run,
and it redirects the largest item in the file.

**Filters and devices these six passes leave behind**, the part that
outlives the numbers:

- **`--tree=caller` on `computed_permanent`, per-call inclusive Ir.** >2,000
  means the caller is gathering. Applied to a *caller* whose per-call cost
  is ~2,000+, read upward to the nearest `&mut self` call and ask how many
  gathering reads sit between the two — the eleventh pass's rule in its
  plural form, and what candidate (10) enumerates.
- **The presence-gate device, audited both ways.** `ability_strip_in_scope`
  is the pattern every later gate copies: name all the routes to the thing
  in the doc, `debug_assert!` the emitting blocks against the same
  predicates, and add a debug-only whole-board re-run inside the gate that
  fails if the gate ever says `false` wrongly. The subset half carries a
  `debug_assert!` at its read sites that no battlefield permanent was read
  outside it — which is what makes a subset safe to widen later.
- **An iterator that clones and then narrows.** The syntactic sweep
  (`\.cloned\(\)` … `\.filter`) is clean workspace-wide; the semantic
  question — how much of what a loop builds does it keep — is not swept.
- **A `find`/`collect` over a hash container in game logic** is no longer a
  determinism bug (`841dd40b` gave the engine a fixed hasher) but is still
  an arbitrary rules choice; see TODO.md.

**What the twenty-eighth pass leaves behind, and it is a filter, not a
fact.** All four of its rows are the same defect: **`&mut` on a CoW handle
costs a deep copy, so a write that changes nothing is not free.** `cow.rs`'s doc comment has warned
about this since the type was written ("any `&mut` access, including
`iter_mut` used read-only, copies the whole inner value") and the profile
had four of them at 1.4-2.0 % each. The reason they survived twenty-seven
passes is that **none of them is visible in a function list**: the cost is
attributed to `Player::deref_mut` and `Arc::clone_from_ref_in`, generic
leaves that look like unavoidable infrastructure. What finds them is
`callgrind_annotate --auto=yes` and a sort of *the `=> …deref_mut` call
lines by cost* — each one names a source line that took an unshare, and the
line above it says whether the write was needed.

```text
# the sweep that found this pass, run it again next time
callgrind_annotate --auto=yes --threshold=98 cg.out > ann.txt
# then: every "=> …DerefMut>::deref_mut (Nx)" line in ann.txt, sorted by
# Ir, with the five source lines above it for context.
```

Three shapes to check at each hit, in order of how often they paid here:
a **periodic reset** that writes a value the field already holds
(`cards_drawn_this_step`, `opponent_cast_spell_since_your_turn`); a
**sweep with a rare subject** that should be behind a presence gate
(`clear_step_bounded_may_play`); and a **cost paid for a benefit the
representation already provides** (the library strip — its comment named a
cost that stopped existing when zones became `CowBox`es). *A perf comment
that explains why something is worth it is a claim with a date on it;
re-check it against the current representation before trusting it.*

**Thirty-first pass, session A — a write that changes nothing, and a scope
that already existed.** Base `a58447d9` read **2,776,361,361 Ir** against the
thirtieth pass's recorded 2,776,361,994 (**633 Ir apart, 0.00002 %** — ties
the tightest agreement in this file).

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-14 | The combat-damage pair takes one gather, not three (`3d15878b`) | 2,776,361,361 Ir | 2,749,500,214 Ir (**-0.967 %**) | Each (attacker, blocker) iteration already froze the layer system for its read-only prefix, dropped the scope, and then took two more whole-game gathers: `ironscale_replace`'s CR 614.9 `creature_redirects_damage_to_controller` (**15,650,898 Ir / 0.56 % over 4,450 calls, 3,517 each** — 91 % of `ironscale_replace`'s entire 17.2 M) and the CR 615 self-prevention pair, whose `damage_from_source_prevented_by_keyword` topped the `computed_permanent` caller table at **3,186 Ir over 4,450 calls**. Both are `&self`; only `apply_prevention_shields` sits between them and the scope, and it consumes shield charges and emits events without moving a layer input. Both loops now answer them inside the scope and hand `ironscale_replace` its redirect |
| 2026-08-14 | The cleanup wear-off stops writing what is already clear (`da5b1f1c`) | 2,749,500,214 Ir | 2,684,396,951 Ir (**-2.368 %**) | The pass's largest row and the cheapest to write. `clear_end_of_turn_effects` has carried an `end_of_turn_effects_are_clear` guard since `15a9cce6`; its two siblings in the same function did not. The graveyard sweep walked every card in every graveyard once per player per turn writing two `= None`s over `None`, and the CR 514.2 damage sweep wrote `damage = 0` over every battlefield and phased-out permanent. **`--tree=caller` on `Arc::clone_from_ref_in` read `finish_cleanup` 72,631,289 Ir / 2.62 % over 36,768 calls (1,976 each) — after, 0 calls.** Graveyards only grow, so this got worse every turn of every game |
| 2026-08-14 | The untap step stops clearing sickness that is already clear (`76804984`) | 2,684,396,951 Ir | 2,681,771,690 Ir (**-0.098 %**) | Same shape one step earlier: eleven sites write `summoning_sick = false` for every permanent their controller untaps, once per early-exit branch, and the flag is already `false` for all but last turn's arrivals. `CardInstance::clear_summoning_sickness` holds the guard in one place, like `clear_end_of_turn_effects`. Measured as its own container, not by arithmetic |
| | **cumulative, session A** | **2,776,361,361 Ir** | **2,681,771,690 Ir (-3.407 %)** | callgrind on the fixed six-game workload; golden traces byte-identical on every row, full suite green |

**What the pass leaves behind, and it is the twenty-eighth pass's filter
with a new place to point it.** That pass found four CoW unshares by
sorting `=> …deref_mut` call lines; this one found the largest row a
different way, and the way generalizes: **`--tree=caller` on
`Arc::clone_from_ref_in`, and read the caller names for a function that has
no business deep-copying anything.** `finish_cleanup` at 2.62 % over 36,768
calls is not a *sweep* problem — the sweep is required by CR 514.2 — it is
that the sweep wrote values the fields already held. The three shapes the
twenty-eighth pass listed still rank in the same order, and *periodic reset
that writes a value the field already holds* has now paid twice, for
-1.998 % and -2.368 %. **Where to look next: any `for … in &mut` over a
zone, in a function that runs on a turn or step boundary.** The guard costs
a `Deref` read; the write costs a `CardData` or `PlayerData` deep copy.

**And a methodological note worth more than the row.** The untap row was
nearly mis-scored. Measured *together* with the cleanup row, the
`--tree=caller` attribution for `do_untap` read **13,304,910 -> 39,345,229
Ir at a lower call count** — i.e. it looked like a regression — because the
new `clear_summoning_sickness` frame changed which file:function the
inliner attributed the deep copies to. Splitting the two changes into two
containers and measuring each showed the untap row is **-0.098 %**, a small
real win. *A per-caller row is an attribution, not a measurement; when two
changes land together and one looks wrong, rebuild them apart rather than
reasoning about the tree.*

**Thirty-first pass, session B — the twenty-eighth pass's defect, at the
five sites it did not reach.** Base `a58447d9` read **2,776,363,573 Ir** against the
thirtieth pass's recorded 2,776,361,994 (**1,579 Ir apart, 0.00006 %**).

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-14 | The cleanup sweep stops unsharing to write what is already there (`d33552e5`) | 2,776,363,573 Ir | 2,710,312,848 Ir (**-2.379 %**) | `cleanup_wear_off` ran three unconditional writes over CoW handles once a turn. `card.damage = 0` is a `DerefMut` on a `CardInstance`, i.e. a deep copy of the whole `CardData`, and almost every permanent is undamaged at cleanup: **36,768 `clone_from_ref_in` calls reached from `finish_cleanup` at ~1,976 Ir each, 72.6 M / 2.62 %**. The graveyard loop had the shape twice over — `&mut player.graveyard` unshares the seat's `PlayerData` before the loop starts, then each `granted_flashback_eot = None` deep-copies a card to write the `None` it held — and the three per-seat scalar resets below it kept the seat unshared once the loop stopped doing so. `finish_cleanup` inclusive **122.5 M / 4.41 % plus a separate 73.1 M / 2.63 %** of inlined `Arc` code, over 1,718 calls |
| 2026-08-14 | The per-turn and per-combat card sweeps stop unsharing (`6b72c0e7`) | 2,710,312,848 Ir | 2,608,585,721 Ir (**-3.753 %**) | Five more sites of the same defect. `resolve_combat`'s provoke reset (`for c in &mut self.battlefield { c.must_block = None }`) was **68,212 `CardData` `make_mut`s, 55.5 M / 2.05 %**, and the count is exactly `4,474 combats x battlefield`; `do_untap`'s nine `summoning_sick = false` clear sites read the flag once into `clear_sick` instead (**207,272 `make_mut`s, 33.8 M / 1.25 %**) and `tapped = false` moved inside the `if card.tapped` already there; the turn-begin goad/detain/`attacked_this_turn` reset had six unconditional writes and a `retain` per permanent per turn; `end_turn`'s `may_play_until` sweep got one gate per zone (same shape as `078b8cef` — the libraries are ~35 cards a seat and reaching one through `Player` unshares the whole `PlayerData`); plus `may_spend_any_color_this_turn` / `kept_mana_this_turn` on the seats the turn-begin reset does not otherwise touch. `resolve_combat`'s card `make_mut`s **68,212 -> 23,168**, `do_untap`'s **207,272 -> 7,532** |
| 2026-08-14 | Five transient `GameState` hash fields become `IdMap`/`IdSet` (`62e6dd42`) | 2,608,585,721 Ir | 2,603,084,794 Ir (**-0.211 %**) | Candidate (3)'s `GameState::clone` half, taking only the fields the serde gate does not apply to: `combat_damage_order`, `combat_damage_assignment`, `names_this_resolution`, `leaves_bf_lki`, `players_sacrificed_this_resolution` are all `#[serde(skip)]`. Each is empty or one-entry for almost the whole game and is cloned on every `GameState::clone` (`RawTable::clone` reads **268,816 table clones under `GameState::clone`**). `IdMap`/`IdSet` already carry the `HashMap` API the sites use, so only six initializers and one index expression needed touching. **The span also carries `1f68d1b0`** (the zone-walk reorder, candidate 11a, which claimed no Ir) **and `9ee83f5d`** (a guard in a `PermanentsDontUntap` branch no bench deck reaches, i.e. exactly zero here), so the number is for the two together |
| | **cumulative, session B** | **2,776,363,573 Ir** | **2,603,084,794 Ir (-6.241 %)** | callgrind on the fixed six-game workload. Behaviour: `--bench` reads `decisions` **193,232** byte-identical to Baseline, `turns_per_game` 26.98, stalls 0, determinism ok; the wide pool `--bench --threads 1 --games 300 --seed 11 --decks all` reads `decisions` **2,548,986** over 5,100 games and 17 archetypes, identical to every tip since `841dd40b`, with the recorded 6 draws and determinism ok. Full suite green |

**The recipe, because the twenty-ninth pass's one stopped resolving.** That
pass retired the `=> …deref_mut` sort (`Player::deref_mut` is inlined now)
and pointed at `--tree=caller` on `Arc::make_mut`. The refinement that
found all six of this pass's sites in one read is to sort by the
**inlining function**, not the leaf:

```text
callgrind_annotate --auto=no --tree=caller --threshold=99 cg.out > tree.txt
# then: every `<` row whose *file* is alloc/src/sync.rs and whose
# *function* is an engine function. Each names a function that deep-copies
# a CoW handle, with the call count beside it. Divide: ~1,900 Ir a call is
# `CardData`, ~900 is `PlayerData`, ~30 means the handle was already unique
# and there is nothing to take.
```

`finish_cleanup` sat there at **36,768 calls / 2.62 %** and is invisible in
every other view — its own function row reads 0.03 % self, and
`--auto=yes` attributes the deep copy to `sync.rs`, a file the annotator
cannot show. **Then read the source and ask, of each write in the loop,
whether the field already holds the value being written.**

**What the pass leaves behind: `x = v` is not free, and neither is
`for x in &mut zone`.** Every row is the same two-line audit — a periodic
sweep writing a default over a CoW handle, and an `iter_mut` taken to
decide whether anything needs writing. The twenty-eighth pass called this
class closed after four rows; six more were sitting in `stack.rs` and
`combat.rs` worth **6.13 %** between them, all in code that reads as
obviously correct. *A sweep that resets state is the highest-yield place to
look, because the value it writes is by definition the value most objects
already hold.* The syntactic filter is `\.iter_mut\(\)` and
`for \w+ in &mut self\.` over a zone; the semantic one is a write with no
`if` above it.

**And two candidates costed here and not taken.** Candidate (10)'s
freeze-scope enumeration was applied to the combat-damage tail:
`combat_damage_prevented_to_self` reads **4,051,310 Ir over 4,430 calls =
915 Ir**, i.e. below the 2,000 threshold, so it is *not* gathering and
folding it with `damage_from_source_prevented_by_keyword` (3,185 Ir/call,
one gather) saves nothing — and the two are separated by `ironscale_replace`,
a `&mut self` call, so a scope over the pair would not be sound anyway.
Candidate (11a)'s miss counter on `find_card_anywhere` was also not run:
the function is 0.70 % at the tip and a perfect id→zone index cannot beat
that, so it is ranked below the `CardCold` item the sweep turned up.

**The thirty-first pass's merged tip — `54f5981b`, 2,577,862,811 Ir,
-7.152 % from `a58447d9`. Not -9.4 %, which is what adding the two
cumulatives gives.** The third collision this file has recorded and much
the largest: the sessions independently pulled `cleanup_wear_off`
(-2.368 % / -2.379 % — two measurements of the same defect, 0.011 % apart,
which is the closest agreement two independent implementations have
produced here), `do_untap`'s summoning-sick clears, the zone-walk family,
*and* both wrote the `CardCold` candidate. Only the non-overlapping rows
add: session A's combat-damage scope fold and session B's provoke reset,
turn-begin sweep, `end_turn` zone sweep and `IdMap` conversion. **The
difference between -9.4 % and -7.152 % is the overlap, and there is no way
to know it without building the merge.**

Three resolutions where both sides had a version, kept because each is a
rule rather than a preference: a **named guarded helper beats a hoisted
local** (`CardInstance::clear_summoning_sickness()` over a `clear_sick`
binding — the guard travels with the field); a **zone-level gate and a
per-card guard are not alternatives, they compose** (the cleanup graveyard
loop takes A's per-seat stale check *and* B's per-card test, so one card
carrying a grant no longer deep-copies the fifteen beside it); and a
walker's visit order takes the **union of what each session learned about
its callers**, not one session's list (`find_card_owner` looks at the stack
first — two or three items — then hand/graveyard back to front, exile back
to front, libraries last).

Behaviour at the merged tip: `--bench` `decisions` **193,232**
byte-identical, `turns_per_game` 26.98, stalls 0, determinism ok on all 160
pairs; `--bench --threads 1 --games 300 --seed 11 --decks all` `decisions`
**2,548,986** over 5,100 games and 17 archetypes, `turns_per_game` 20.99,
6 draws (0.12 %), determinism ok on all 2,547 pairs. Full suite green.
`clippy --workspace --all-targets` clean. **`overflow` was not re-run and
is owed at this tip.**

**And the two pool constants are both current — quote the command.**
`--bench --threads 1 --games 300 --seed 11 --decks all` reads 2,548,986
with 6 draws; `--decks all --games 300 --threads 3` reads 2,553,880 with 2.
Session A read the second and concluded the first was stale; it is not, it
is a different invocation. This file already carries that correction once
(see Baseline). *The constant is not the check — a base-vs-tip byte-diff of
a kept binary is.*

## Profile of record

Callgrind on `profiling-fast --no-default-features` (= `release-fast` opt
settings + debuginfo; system allocator, because valgrind replaces malloc and
a mimalloc build would measure the interception), 1 thread, `--a gang --b
gang --games 6 --seed 1 --decks fixed`.

**Re-taken 2026-08-14 at `645b978d`: 2,832,745,260 Ir.** Supersedes the
`5034eb2f` table. The pass's third row (`dbd3efeb`) took another 0.473 %
out of `find_card_anywhere` after this table was derived, so every share
below reads ~0.5 % low against the tip; the ordering did not move. Shares are of the
smaller total, so a row whose absolute Ir *fell* can still show a larger
share — that is what a pass which only touches the rest of the program
looks like.

| Ir | share | site |
|---|---|---|
| 1,451,636,594 | 51.24 % | `pick_attacks_scored` (630 calls) — the search, still untouched |
| 1,444,331,397 | 50.99 % | `simulate_attack_outcome_once` under it, 1,166 calls |
| 455,733,159 | 16.09 % | `would_accept` |
| 433,154,076 | 15.29 % | `resolve_combat` |
| 422,742,124 | 14.92 % | `try_pay_after_snapshot_mode` |
| 418,226,206 | 14.76 % | `computed_permanent` |
| 403,936,099 | 14.26 % | `auto_tap_for_cost_inner` — 8.9 % of it is the two `activate_ability` loops, i.e. real taps |
| 366,160,542 | 12.93 % | `sim_spell_action_inner` |
| 282,706,149 |  9.98 % | `gather_continuous_effects_inner` |
| 273,830,767 |  9.67 % | `check_state_based_actions` |
| 258,911,968 |  9.14 % | `activate_ability` |
| 250,988,227 |  8.86 % | `dispatch_triggers_for_events` |
| 249,073,124 |  8.79 % | `Arc::clone_from_ref_in` — the CoW unshare leaf, 13.66 % before this pass |
| 185,406,363 |  6.55 % | `pick_by_outcome` |

The allocator block on this tip reads **~18.5 %** (`_int_malloc` 5.78 /
`_int_free` 5.71 / the rest merge/arena/consolidate/unlink),
`__memcpy_avx_unaligned_erms` 3.93 %, `Arc::make_mut` 7.50 % inclusive.
**The allocator share went *up* while the program got smaller** — the pass
removed clone work, not allocation sites, so the fixed cost of what is left
is a larger fraction. Read it that way before treating it as a regression.

The counts and per-caller breakdowns quoted below this table are from the
`f814a13b` retake unless a line says otherwise; they were not re-derived
this pass. Absolutes there are ~7 % high against this tip.

**The number this retake adds, and it reframes candidate (0).**
`pick_attacks_scored` runs **1,166 simulations over 630 declarations** —
**1.85 per search**, and **1.33 M Ir per simulation, i.e. 0.0425 % of the
whole program each**. With `gang`'s `attack_search: 6` a greedy set of two
attackers would produce four candidates, so nearly every scored declaration
on this workload is the binary *swing with the one creature or don't*:
1,166 = 2 x 583, with the other 47 of the 630 returning early on an empty
greedy set. **Half of every simulation the engine runs is therefore the
"attack with nobody" candidate — about 25 % of the program.** Whether that
candidate ever wins is exactly the probe candidate (0) asks for, and it is
now one counter (`choose_scored`'s returned index) rather than a study.

**Who gathers**: `computed_permanent` 94,708 (213,876,885 / 6.83 %),
`frozen_effects` 18,916 (1.24 %), `check_state_based_actions` 10,670
(0.68 %), `compute_permanents` 3,274 (0.24 %), `compute_battlefield` 310
(0.02 %) — **127,878 total**, from 141,106 at `125557eb`. The
`computed_permanent` row is candidate (9).

**`would_accept` by caller** (candidate (1) is the probe count, so this is
the list to shorten): `main_phase_action_with` 1,980 (179,766,093 / 5.74 %),
`pick_stack_response` 342 (1.37 %), `pick_combat_trick` 292 (1.03 %),
`pick_land_to_play` 916 (0.75 %), `pick_removal_ping` 38,
`pick_defensive_removal` 12.

**`can_afford_in_state` costed at line level, 2026-08-12** — 59,480,310 /
1.90 % over 12,702 calls (4,682 Ir each), by callee:
`available_mana` **35,840,109 / 1.14 %** (60 % of it, and all of
`available_mana`'s cost in the program); the four
`sources x static_abilities` walks **9.0 M / 0.29 %** between `actions.rs`
(5,242,114) and `mod.rs` (3,766,423, the CR 609.4b permission);
the hand `find` in `extra_cost_for_card_in_hand` 2,650,624; `can_afford_from`
and its two `ManaCost` clones ~7.5 M. Inside `available_mana`:
`grant_scan` 8,155,753 over 12,702 and `granted_abilities_with` 13,603,586
over **63,656** (five untapped permanents per call).

**The older tables — passes fifteen, seventeen, twenty and twenty-four — are
in git** (`git log -- PERF.md`, ~150 lines). Every row in them is either
superseded by the table above or already quoted at a fresher share in a
candidate. Two facts from them that do not restate and are kept here:
`compute_battlefield` went 47,808 -> 17,718 -> 310 calls across the
nineteenth and twentieth passes and its table is closed; and the fifteenth
pass's allocation tree (3,748,803 allocations for six games, `Arc::clone_
from_ref_in` 802,482 of them) is the origin of the CoW-unshare theme that
`CardInstance = Arc<CardData>` later paid at -25.6 %.

The allocator block on this tip re-reads **~17.0 %** (`_int_malloc` 5.34 /
`_int_free` 4.38 / `malloc` 3.23 / `free` 1.96 / merge 0.88 / arena 0.77 /
consolidate 0.74 / unlink 0.73), `__memcpy_avx_unaligned_erms` 3.80 %,
`Arc::clone_from_ref_in` ~16 % inclusive over ~1.6 M CoW unshares.


## Perf candidates

Ordered by expected value. Each run pulls the top one, attaches numbers,
and feeds what it finds back in. Re-profile and replenish when the list
goes thin or stale.

**(-2) A `CardCold` group on `CardData` — the largest single item costed on
this profile, and the one that needs a run to itself.** `--tree=caller` on
`Arc::make_mut` at `a58447d9`, summing the rows whose file attribution is
`crabomination_base/src/card.rs` (`CardInstance::deref_mut` inlined into the
caller): `resolve_combat` **55,962,640 / 2.02 % over 68,212**,
`activate_ability_inner` **31,141,085 / 1.12 % over 18,312 (1,700 Ir
each)**, `cast_spell_with_convoke` 21,434,296 / 0.77 % over 74,150,
`declare_attackers_banded` 11,040,699 / 0.40 %,
`place_card_at_resolved_zone` 9,119,601 / 0.33 %, `do_untap` 6,218,160 /
0.22 %, `declare_blockers` 5,294,135 / 0.19 %, then a tail — **~5.4 % of the
program is deep-copying a `CardData`**, and `Arc::clone_from_ref_in` is
8.97 % inclusive. **Re-read at the pass's tip `62e6dd42`, after the two
sweeps took the avoidable copies: ~105 M / 4.0 %**, `resolve_combat` down to
2.8 M and `do_untap` to 9.2 M, `activate_ability_inner` now the largest row
at 31.6 M / 1.21 % — and every one of those is a genuine state change, so
4.0 % is the whole of what a cold group can address. At ~600 of ~1,900 Ir a
clone being empty-`Vec` copies, expect **~1.2 %**. `CardData` is **148 fields, 32 of them heap-owning**;
`PlayerCold` took the fifteen rare ones on `PlayerData` and paid. The rare
heap fields here, from a field-by-field read: `spliced_effects` /
`spliced_names`, `spree_modes` / `modes_chosen` / `kicked_options`,
`stored_die_results`, `named_card` / `named_card_2`, `chosen_colors`,
`goaded_by`, `saddled_by`, `crewed_by`, `meld_parts`, `mutate_stack`,
`granted_activated_abilities` / `granted_activated_eot`, `removed_keywords`
/ `removed_keywords_eot`, `once_per_turn_used`, `exhausted_abilities`,
`pending_etb_counters`, `cast_mana_spent_by_color` — ~20 fields that are
empty on essentially every card. **Do not widen past that**: the tenth
pass's rule is *group size x unshare probability < sum of the individual
clone costs*, and widening `ColdState` to 126 fields read **+1.23 %**. The
blockers are why this wants a whole run: 148 fields means a `Deref` /
`DerefMut` split reaching the entire engine, and every cold field's serde
attribute has to survive (candidate (3)'s note). Do not start it late in a
run.

**(-1) The CoW-unshare sweep — re-run it, but with the new recipe.** The
twenty-eighth pass took four of these for -7.257 %, the twenty-ninth two
more for -1.993 %, and the thirty-first two more for -2.466 %. The
`=> …deref_mut` sort that found the first four no longer resolves (see the
twenty-ninth pass's Log block): `Player::deref_mut` is inlined now, and the
40 surviving `deref_mut` call lines are 0.12 % between them. **Two sorts
still work: `--tree=caller` on `alloc::sync::Arc<T,A>::make_mut`** (7.50 %
inclusive at the tip), **and — the one that found the thirty-first pass's
largest row — `--tree=caller` on `alloc::sync::Arc<T,A>::clone_from_ref_in`,
reading the caller names for a function that has no business deep-copying
anything.** The second is the better first look: `make_mut`'s table is
dominated by writes that are genuinely needed, while a `clone_from_ref_in`
caller at four figures of calls is usually a sweep writing values the fields
already hold (`finish_cleanup`, 2.62 %, gone). **Read either table by the
*inlining* function, not the leaf**: every `<` row whose file is
`alloc/src/sync.rs` and whose function is an engine function names a
function that deep-copies a CoW handle, with the call count beside it —
~1,900 Ir a call is `CardData`, ~900 is `PlayerData`, ~30 means the handle
was already unique and there is nothing to take. **The standing filter is a
`for … in &mut` over a zone in a function that runs on a turn or step
boundary**, and the syntactic sweep of those is clean under `game/` as of
`62e6dd42`; `server/` and `effects/` were never swept the same way. What is
already known and does not need re-deriving:

* **`remove_from_hand` is not a candidate**, nor is `attacked_this_turn`,
  nor is hoisting the zones out of `PlayerData`. The unshare is inherited by
  whatever the action writes next. *Only a seat's **sole** write in an
  action is removable* — the twenty-ninth pass's Log block has the
  arithmetic.
* What is left on `PlayerData` is **cost per clone**. `PlayerCold` took the
  fifteen heap-owning rare fields; the remaining per-clone cost is the
  struct memcpy (~110 scalars) plus the collections that are genuinely hot
  — `spell_casts_this_turn` (3,330,696 Ir over 24,852 clones, i.e. often
  non-empty), `spells_cast_by_name_this_game`, `graveyard_ids_this_turn`,
  `creatures_entered_this_turn`. Each is written on the same action that
  already unshared the seat, so a *second* CoW group would not pay — a
  `CowBox` on the individual field might, when clone count (24,852) far
  exceeds write count (~7,000 for the cast-path ones). Cost the ratio first.
* The new `PlayerCold` group unshares **1,944 times at 891 Ir**; the
  per-turn reset loops guard their `clear()`s on `is_empty()` to keep it
  there. **Any new `clear()` on a cold collection in those loops needs the
  same guard**, or it hands the row back.
* Genuine writes already costed, listed so the next sweep skips them:
  `pay_for_spell` 4,768,455 / x8,892, `pending_creature_etb` 930,435.

**The `compute_battlefield` table is closed.** Every site the nineteenth
and twentieth passes ranked is paid: `declare_attackers_banded`
(`31116d43`), `declare_blockers` x2 (`42f59829`, `911cf298`),
`resolve_combat` + `resolve_first_strike_damage` (`4f3e86c0`),
`do_phasing` / `do_untap` / `process_cumulative_upkeep` (`ed4c152c`).
**Calls 17,718 -> 5,488 -> 310**, and the 310 are all `submit_decision`
(741,960 Ir / 0.02 %). Do not re-open this table; the layer system's
remaining cost is per-card `computed_permanent` and the gather itself.

**Four rows costed on the thirty-first pass's base and not taken**, listed
here rather than as numbered entries because each is one measurement short
of being actionable. All from `a58447d9`, `--tree=caller` /
`--tree=calling` / `--auto=yes` on the fixed six-game workload:

* **Auto-tap's activations, `auto_tap_for_cost_inner` -> `activate_ability`
  253,177,209 / 9.12 % over 18,832 calls (13,444 Ir each)** — the single
  largest callee edge outside the search. `try_pay_after_snapshot_mode` is
  14.84 % inclusive and `auto_tap_for_cost_inner` 14.16 %, i.e. **paying for
  a spell is a seventh of the program** and ~2.1 land taps per payment.
  Inside each activation, `activate_ability_inner`'s single
  `with_frozen_layers` costs **49,055,654 / 1.77 % over 18,386 calls (2,668
  Ir each — a gather)**, and it exists to answer three questions:
  `lost_all_abilities`, is-a-creature, and `land_mana_lost`. **All three
  already have presence gates** (`ability_strip_in_scope`, the layer-4
  card-type gate `compute_battlefield_creatures` documents,
  `rewrites_land_types`) — but every gate tests the *gathered* set, so
  inside a scope it creates they cost exactly what they save. The lever, if
  there is one, is a scope spanning `auto_tap_for_cost_inner`'s tapping
  loop, and it is **unsound as written**: tapping a permanent is a layer
  input (`UntapOnlyChosenTypeWhileUntapped` and any `WhileCondition` keyed
  on an untapped permanent read `!c.tapped`). Cost how many of the 18,832
  taps happen on a board carrying either before designing anything.
* **`dying_snapshot`, 10,815,151 Ir over 3,420 calls (3,162 each — a
  gather)**, one per death from the SBA sweep and four sacrifice paths. Its
  `computed_permanent` exists only to notice that the computed creature
  types differ from the printed ones, which needs a *live type-changer*;
  `compute_permanent` already computes `has_type_changer` from
  `Modification::SetCreatureTypes | AddCreatureType`. Same gate-needs-the-
  gather circularity as the row above, and the callers are `&mut self`, so
  a scope cannot wrap them.
* **`printed_color_set`, 17,572,024 / 0.63 % over 237,652 calls (74 Ir
  each)** — one per `compute_permanent_pass`, and a pure function of the
  `CardDefinition`. Not memoizable *on* the definition (candidate (13)'s
  reasoning: `Arc::make_mut(&mut card.definition)` mutates in place), so it
  is a construction-time field or nothing — i.e. it belongs to the same
  `CardDefinition` rework as the `Keyword` bitset, not to its own row.
* **`compute_permanent_pass` is 134,165,234 / 4.83 % over 237,652 cards
  (565 Ir each)** and its own annotated lines are all under 0.25 %; the cost
  is spread across four `Printed::new` constructions, `base_power` /
  `base_toughness` (3,341,698 each, attributed to `raw_vec`), and
  `affected_includes_gated` (4,086,736 over 91,256). **There is no single
  line to take here** — record it so the next run doesn't re-annotate
  layers.rs hoping for one.

0. **`pick_attacks_scored`, 1,475,649,076 / 51.05 % over 630 calls — half
   the program, and *not* a pruning item.** The search itself, not its
   inner loop, which every pass since the seventeenth has made cheaper.
   **This is a bot-quality question as much as a perf one** — a narrower
   search is a different player, so it needs a `bot_ladder` win-rate gate,
   not an Ir number.

   *Costed and probed 2026-08-12; do not re-run either.* The 630
   declarations run **1,166 simulations — 1.85 per search, 1.33 M Ir each
   (0.0425 % of the program per simulation)**, so on this workload nearly
   every scored declaration is the binary *swing with the one creature or
   don't*. The pruning probe (a counter on `choose_scored`'s returned
   index, `--decks all`, 10,200 games, 110,000 searches) reads **greedy
   54.0 % / the empty declaration 35.0 % / a greedy-minus-one 11.0 %**:
   the search departs from greedy **46 %** of the time and every candidate
   class pays for itself. **So the only lever on this entry is making one
   simulation cheaper, which is what the rest of this list does.** Note
   `--decks fixed` under-represents the search by ~1.8x (3.37 simulations
   per search on `all` against 1.85 on `fixed`) — the fixed archetypes
   rarely present a multi-attacker board. Re-run the probe only if the
   bot's declaration policy changes.
1. **`would_accept`, 491,192,545 / 15.34 %** — the affordance probe, one
   `GameState::clone` + one `perform_action_inner` per candidate action.
   What is unexamined is the *probe count*, i.e. how many candidates the
   bot dry-runs that no scoring pass could have chosen.
2. **The `trigger_grant_sources` scan itself — costed 2026-08-11 and
   deliberately not taken.** The peel is not the cost: `active_static`
   returns on its *first* `match` arm for any non-wrapper effect, and a
   card with no `static_abilities` runs an empty inner loop, so the ~480 Ir
   per call is ~24 Ir per card of iterator overhead over ~20 cards. A
   syntactic peel saves the `EffectContext` only on the rare
   `WhileCondition` static; `resolve_named_by_source` only runs on a hit.
   **The only lever with real headroom is a cached board-level flag**
   ("does any static source carry a `GrantTriggeredAbility`"), and that
   needs invalidation at every battlefield mutation — `sa_cards` is a
   *local* built inside the gather, not a state field, so it cannot be
   borrowed here. Cost the maintenance before writing it. Original
   measurement, kept: 53 M / 1.57 %
   over 110,540 calls at the twenty-second pass's base; `1112e709` took the
   28,564 that were per-*kind*, leaving ~89 k that are genuinely one per
   batch (the dispatcher 52,332, `fire_step_triggers` 14,462,
   `fire_combat_damage_triggers` 7,118, `resolve_top_of_stack` 7,046,
   `cast_spell` 14,092 …). ~480 Ir each for a walk of `all_static_sources()`
   x `static_abilities` calling `active_static` per ability. **The lever
   left is making the scan cheaper, not calling it less**: the same
   presence-gate device on `permanents_with_abilities_removed` — a
   syntactic peel of the duration/predicate wrappers with no
   `EffectContext` and no `resolve_named_by_source` — or a board-level
   flag. Cost it against the walk's ~24 Ir per card before writing it.
   **Since the twenty-fourth pass three of the scan's callers — the
   dispatcher's 52,332, `cast_spell`'s 14,092 and `fire_step_triggers`'
   14,462 — run inside `dispatch_board_scan` or behind its presence gate,
   so a cached board flag now has one place to be read and three to be
   maintained.**
3. **The `RawTable::clone` block — the `#[serde(skip)]` half is now paid;
   what is left is behind serde.** `--tree=caller` on `RawTable::clone` has
   exactly two callers, and `271c7d14` took the larger one's set half.
   `62e6dd42` took the five `GameState` fields that carry `#[serde(skip)]`
   — `combat_damage_order`, `combat_damage_assignment`,
   `names_this_resolution`, `leaves_bf_lki`,
   `players_sacrificed_this_resolution` — for **-0.211 %** across a span
   that also carried two no-Ir commits, against 268,816 table clones under
   `GameState::clone` at the time. Left: the **seven `ColdState`
   `HashMap` fields** (same CoW unshare, ~44.8 k times for six games) and
   `block_map` plus the two per-player-discard maps. **The gate on all of
   them is serde**: a `HashMap` serializes as a JSON object and a `Vec`
   newtype as an array of pairs, so any field that reaches a snapshot needs
   a custom impl or a format bump. Check the field's serde attribute before
   costing it — that check is the whole of what made the five cheap.
4. **PAID, both legs** — `fire_combat_damage_triggers` bucketing its five
   kind-independent walks (`08cbc9c3`, -1.051 %, calls 28,564 -> 7,118) and
   `fire_step_triggers` no longer cloning every printed `TriggeredAbility`
   before filtering on kind (`006d5966`, -1.885 %). Nothing left at either
   site. **What survives is the shape**: *an iterator that clones and then
   narrows*. The syntactic sweep is clean workspace-wide; the semantic
   question — how much of what a loop builds does it keep — is still
   unswept over the hot trigger/candidate builders
   (`push_ordered_trigger_candidates`, `statics_granted_triggers_with`,
   `cast_candidates`' `flat_map`).
5. **`pick_by_outcome`, 210,019,714 / 6.56 %, essentially all of it one
   `collect()` in `bot.rs`.** Never profiled at line level. Read
   `--auto=yes` on it before guessing; the eighteenth pass's candidate (1)
   is the warning that the cost is the *iterator body*, not the container.
   Here it is `evaluate_action_outcome` per finalist — a clone and a
   resolution each — so the container is certainly not the cost, and the
   real question is the *finalist count*, which is a bot-quality question
   like (0) and (1).
6. **`dispatch_triggers_for_events`, now 208,322,657 / 7.21 % over 52,332
   calls** (237,986,906 / 7.64 % before `e02767aa` took the per-permanent
   `Vec`). **The remaining self cost is iteration, not allocation** — the
   `alloc::vec` + `alloc::raw_vec` block that was 69 M is spent, so the
   next attempt here is the (b) below or nothing. Note also that
   `event_matches_spec` runs only **63,846 times for 52,332 dispatches**
   (1.22 per dispatch, 30 Ir each): the battlefield x trigger x event loop
   is *not* the cost and an event-kind presence mask over it would gate
   almost nothing. Historical numbers, kept: `c7bdd850` took the four cheapest blocks, `b925063c` the
   delayed-trigger collects, and `f28faaa0` (lever a', generalized) fused
   the four board walks at the top of the function into one
   `dispatch_board_scan`. Its collect row is **94,608 collects /
   14,001,655 / 0.44 %**, down from 146,940 / 22,068,557 / 0.66 %. What is
   left, in order: (b) a `u32` presence mask over the batch's event kinds
   filled in one pass, with each block gated on its bit — the
   `gated_block!` device from `52f4311a`, `debug_assertions` audit
   included; and `push_ordered_trigger_candidates` (~1.1 %, exactly one
   per dispatch). The `synthesized` chain is **not** worth a gate on its
   own any more — `Vec::from_iter` on an empty iterator is `Vec::new()`,
   so the empty case is one `next()` over two empty vectors. **The
   per-card `all_triggers` Vec is *not* a target** — 33,980 Ir over the
   whole run, checked 2026-08-11.
7. **`mana_source_table`, 116,895,124 / 3.65 % over 7,370 calls** — down
   from 8,892 since the gate (`1ec589d1`). What is left is probe (b),
   untried: the table is a pure function of (battlefield tap state, layer
   inputs, `creature_only`), so a per-freeze-scope memo may be the
   `computed_permanent` trick again. **The surviving calls are exactly the
   ones that go on to tap**, which is what invalidates the memo, so cost
   the hit rate before writing it. `untapped_producers_of` — five frozen
   board scans where one would do — **does not appear on this profile at
   all**; the hybrid path is cold on the bench decks, so leave it.
8. **The collect table, re-measured on the twenty-fourth pass's tip** —
   `--tree=caller` on `Vec::from_iter`, inclusive, so these are *iterator
   body* costs (see the eighteenth pass's correction below, which is still
   the warning that matters):
   `cast_candidates` **168,130,962 / 5.25 %** over 7,024 calls — **broken
   down 2026-08-11**, see below;
   `check_state_based_actions` **132,514,326 / 4.14 %** over 82,634;
   `mana_source_table` **108,286,211 / 3.38 %** over 7,370;
   `pick_attacks_inner` 0.67 % over 7,842;
   `pick_removal_ping` 0.67 % over 37,710;
   `dispatch_triggers_for_events` 0.44 % over 94,608.
   **`fire_step_triggers` is off this table** (`006d5966`); `empty_mana_pools`
   was paid at `ef731ecc`. Only three rows are over 3 %, and the top two are
   the ones whose iterator body is real work rather than container overhead
   — `cast_candidates` is 23,900 Ir per call of auto-targeting per hand
   candidate, `check_state_based_actions` is `compute_battlefield_creatures`
   plus the `dead` filter. Read `--auto=yes` on either before costing it.
   **`cast_candidates`, broken down** (`--auto=yes` + `--tree=calling`,
   twenty-fourth pass's tip). Its *own* self cost is under 0.2 % across six
   file rows — the 5.62 % inclusive is callees inlined into the
   `Vec::from_iter` frame, so read the annotated source, not the function
   list. Two callers, 3,382 and 3,642. By callee:
   * `can_afford_in_state` — **CLOSED 2026-08-12, negative result.** The
     fused-scan fix this entry prescribed measured **+0.066 %** and was
     reverted; see the twenty-seventh pass's Log block for why (1.13 cards
     per sweep, not 1.72 — the filter is not in `cast_candidates`). What is
     left of the item after measurement: the four static walks are **0.29 %**
     between them, `available_mana` is **1.14 %** and was 60 % of the
     function, and the part of `available_mana` that was real cost —
     `granted_abilities_with`'s redundant `battlefield_find` — is **paid**
     (`granted_abilities_of`, -0.552 %). Do not re-open. The original
     measurement, kept because the arithmetic below is still the warning:
     **56,500,015 / 1.76 % over 12,114 calls (4,664 Ir each)**.
     Its body per hand card: `extra_cost_for_card_in_hand`,
     `cost_reduction_for_spell`, **a `ManaCost` clone** to append
     `colored_spell_tax_for_spell` (empty on every bench card),
     `relax_cost_colors`, and `available_mana(state, seat)`. **The lever is
     not the clone and not a hoist — it is that these are five separate
     whole-battlefield walks per call**, four of them `battlefield x
     static_abilities` (`extra_cost_for_spell`, `cost_reduction_for_spell`,
     `colored_spell_tax_for_spell`, `relax_cost_colors`) plus
     `available_mana`'s producer scan. Same shape, and the same fix, as the
     `DispatchScan` row that paid -1.118 %: one pass filling a small struct.
     **Two levers that do *not* pay, by arithmetic, so they need no A/B:**
     the `ManaCost` clone is ~12 k allocations, i.e. under 0.1 %; and
     hoisting `available_mana` out of the filter cannot pay either —
     12,114 calls over 7,024 `cast_candidates` calls is **1.72 per call**,
     so at most 42 % of one walk is duplicated. That is also the correct
     reading of the fourth pass's no-win on exactly that hoist: the
     asymptotics were real, the *filter* just never reaches seven cards.
   * `affordance_probe_template` **31,110,605 / 0.97 % over 3,382**
     (9,199 each) — one library-stripped `GameState` clone per tick.
   * `pick_land_to_play` 26,629,385 / 0.83 % over 1,410;
     `beneficial_aura_host` 3,392,725 over 606; `Vec::retain` 0.09 %.
   The sibling rows at the same call site are `would_accept`
   **168,728,436 / 5.27 % over 1,534** (the lazy final gate, 110 k Ir per
   probe) and `pick_by_outcome` 210,019,714 / 6.56 % over 908 — both
   candidate (1)/(5), i.e. probe-count questions.

   The allocator block re-reads **~17.0 %** on this base (`_int_malloc`
   5.34 / `_int_free` 4.38 / `malloc` 3.23 / `free` 1.96 / merge 0.88 /
   arena 0.77 / consolidate 0.74 / unlink 0.73),
   `__memcpy_avx_unaligned_erms` 3.34 %, `Arc::clone_from_ref_in` **16.09 %
   inclusive** / ~5.4 % self over ~1.6 M CoW unshares,
   `gather_continuous_effects_inner` 2.74 % self / 9.74 % inclusive.
9. **The gathers `computed_permanent` still takes, by caller — the top
   candidate now.** PERF's own rule — divide each caller's inclusive Ir by
   its call count, >2,000 means it is outside a freeze scope — names five,
   all in the damage path, re-read on the twenty-fourth pass's tip:
   `scale_damage_to` 49,405,906 / 14,624 (**3,378**),
   `damage_prevented_by_protection` 39,667,284 / 18,986 (**2,089**),
   `permanent_has_keyword` 21,973,585 / 8,328 (**2,638**),
   `damage_from_source_prevented_by_keyword` 15,610,446 / 4,450 (**3,508**),
   `dying_snapshot` 11,819,145 / 3,420 (**3,456**) — **138.5 M / 4.33 %
   between them**, and `computed_permanent`'s gathers are 7.60 % of the
   program. **What was checked on the twenty-fourth pass and is now known:
   every one of the five already takes its own `with_frozen_layers` scope
   internally** (`damage_prevented_by_protection` since `96129f68`,
   `scale_damage_to` since the eleventh pass), so each per-call cost is
   *one* gather plus its work — there is no intra-call duplicate left to
   remove. **The only lever left is a scope spanning several calls**, and
   the blocker is real: they sit under `resolve_combat`'s damage loop,
   which writes `dealt_damage_this_turn` / `damaged_by_this_turn` and
   **changes life totals via lifelink between the reads**, and a
   `WhileCondition` static can read life (Serra Ascendant), so a blanket
   freeze over the loop is unsound. Two tractable shapes, in order:
   (a) the strictly-`&self` prefix of one (attacker, blocker) iteration —
   `damage_prevented_by_protection`, the three per-source prevention
   checks, and `scale_damage_to` all run before `apply_prevention_shields`
   takes `&mut self`, so one scope over that prefix folds ~2 gathers into
   1 and is sound by construction; (b) note that
   `resolve_combat_damage_with_filter` **already holds `computed:
   &[ComputedPermanent]` for the whole step** and the five helpers
   re-derive from scratch what it contains — a `_with(computed)` variant
   would be nearly free, but it pins the layer view to the step's start,
   which is arguably *more* CR 510.2-correct and is still a behaviour
   change. Golden traces decide (b); (a) needed nothing and **is PAID**
   (`56f6623f`, -0.740 %) — three scopes, one per prefix. (b) is all that
   is left *of the damage path*, and it is a behaviour question, not a perf
   one. **But shape (a) turned out not to be a damage-path item at all** —
   see (10): the same wrapping paid again immediately, on a function with
   nothing to do with combat.

10. **The rest of the "reads the layer system twice" family — the cheapest
    item on this list, and mechanical.** Candidate (9) framed shape (a) as
    a damage-loop fix; `5d4b5402` shows it is a *shape*, not a site. Any
    `&self` function that reads the layer system more than once is one
    `with_frozen_layers` away from paying, the wrapping is mechanical, and
    **it cannot change behaviour because the closure cannot mutate** — so a
    wrong guess costs a rebuild, not a bug, which makes this the one
    candidate worth attacking by enumeration rather than by profile.
    `check_target_legality_with_source` was worth -0.516 % and took ten
    minutes.

    *The search*, and the part candidate (9) got wrong: PERF's rule is
    `--tree=caller` on `computed_permanent`, inclusive Ir over call count,
    >2,000 means it is gathering — but applied to the **leaf helpers** it
    finds only sites where one gather is already the floor. Apply it to
    **their callers**, which is where a second read can exist. On the
    twenty-fifth pass's tip the two damage leaves that are still gathering,
    `damage_from_source_prevented_by_keyword` (3,507 Ir over 4,450 calls)
    and `dying_snapshot` (3,458 over 3,420), each take exactly *one*
    `computed_permanent` and so have nothing to fold — the win is in
    whoever calls them, or in the `&self` functions that call several
    different leaves. `blocker_can_block_attacker` at 286 Ir/call over
    15,368 calls is what a caller already inside someone's scope looks
    like; imitate its caller.

11a. **The zone-walk family — the ordering half is CLOSED (`76d31eb8`,
    `1f68d1b0`, merged).** Five hand-written walkers ask "where is this
    card": `find_card_anywhere` (paid, -0.583 %), `find_card_anywhere_mut`,
    `find_card_zone`, `find_card_owner` and `death_was_replaced`. All are
    libraries-last now; the walkers that scan a *push-only* zone (graveyard,
    exile) do it back to front, so a card that just moved there hits on the
    first comparison, and `death_was_replaced` — a disjunction, not a search
    — answers the unreplaced case off the graveyards without touching a
    library at all. `find_card_owner` also stopped walking exile twice for
    one id and looks at the stack before the seats. **No Ir claimed** on any
    of it: none of the three reordered walkers is above the 99 % threshold.
    `find_card_zone`'s doc claimed the stack among its zones and its body
    never looked there; corrected to match the body. Do not re-open the
    ordering.

    **What is left, and it is the only part of this entry still open.**
    `find_card_anywhere` is still **19.6 M / 0.70 % over 104,240 calls**
    after the reorder, and the residual is calls that return `None` — they
    walk every zone by construction, whatever the order. Costing that needs
    a miss counter (one instrumented `profiling-fast` build, a two-minute
    run) *before* anyone designs an id→zone index, which is the only fix and
    is state to maintain on every zone move. **And the general filter this
    family taught is worth more than the family**: a `for`/`find` whose
    *first* branch is the rarest case is cheaper to check than to profile —
    neither of the thirtieth pass's two large rows was visible as an
    expensive function.
11. **A helper that opens with `battlefield_find` and is called from a
    battlefield loop — the generalization of the twenty-seventh pass's
    row.** `granted_abilities_with(card_id, scan)` was -0.552 % on its own
    because three of its callers already held the `&CardInstance` and it
    re-scanned the battlefield to recover it; the thirtieth pass took
    `effective_mana_abilities` the same way for **-0.158 %** (54,570 calls,
    three callers, all inside `battlefield.iter()`). **Two hits paid, the
    enumeration is not finished** — the remaining named sites are
    `colors_produced_by`, `creature_prevents_combat_damage_grows`,
    `creature_redirects_damage_to_controller`, `source_power_lki`,
    `combat_damage_sealed_for_your_creatures`; none of them appears on the
    current profile, so read the caller before costing one. *The search*: grep the engine
    for `fn .*card_id: CardId` bodies whose first statement is
    `battlefield_find` / `find_card_anywhere`, then read each caller and ask
    whether it is inside a `battlefield.iter()`. The fix is mechanical (an
    `_of(&CardInstance)` twin, the `CardId` form kept as a find plus
    delegate for the off-battlefield fallback) and **cannot change
    behaviour** — the twin is the same body with the lookup removed — so a
    wrong guess costs a rebuild, not a bug. Same property that made
    candidate (10) worth attacking by enumeration.
13. **The gather's battlefield pre-scan, and the keyword loop inside it.**
    Newly costed 2026-08-14, `--auto=yes` on the twenty-ninth pass's base.
    `gather_continuous_effects_inner` opens with one walk of the battlefield
    filling `sa_cards` and eleven `any_*` flags — the fold that replaced
    eleven walks and is still the right shape. What it costs on the mod.rs
    attribution alone: **`for kw in &def.keywords { match kw { … } }`
    6,420,524 / 0.222 %**, `static_abilities.is_empty()` 5,113,186 /
    0.177 %, `attached_to.is_some()` 2,492,666 / 0.086 %, over 127,878
    gathers x ~20 cards. Five of the eleven flags are a **pure function of
    `CardDefinition`**, so the loop re-derives per gather what is fixed for
    the life of the card. The lever is candidate 5's keyword bitset, not a
    memo on the flags: item 1's reasoning applies —
    `Arc::make_mut(&mut card.definition)` mutates a uniquely owned
    definition in place, so a cache *on* a definition is unsound, and it has
    to be a representation the definition carries from construction.

    **Costed 2026-08-14 and deliberately ranked down; do not re-derive.**
    The bitset's whole addressable total is **~0.6 %** and its cost is a
    catalog-wide representation change. `Keyword::eq` is 14,393,938 Ir /
    0.51 % self, but that is **~1.3 M calls at 11 Ir each spread over thirty
    call sites** — `printed_color_set` 209,154, `CardInstance::has_keyword`
    170,112, `board_keyword_in_scope` 135,886, `can_block_attacker_computed`
    115,502, `declare_attackers_banded` 80,056, then a long tail — so there
    is no single site to fix and the win is the 11 Ir, not the call. The
    pre-scan loop is 0.23 % and *its* half was taken a cheaper way (the
    thirtieth pass's `sa_cards` row, -1.250 %, left the pre-scan alone).
    `permanent_has_keyword`'s cost is `computed_permanent`, not the
    `contains`. Against that: **7,716 `keywords: vec![…]` literals in the
    catalog**, plus ~10 runtime sites that push a keyword onto a definition
    and would have to maintain the mask or silently desync it. Worth doing
    only as part of a `CardDefinition` construction rework, not on its own.
12. **`grant_scan`, 20,282,413 / 0.65 % over ~26,000 calls (~780 Ir each)**
    — `available_mana` 12,702, `mana_source_table` 7,370, the three
    `pick_removal_*` ~2,080 each. It walks `battlefield x static_abilities`
    through `active_static` and builds four `Vec`s that are empty on every
    bench board. Same shape as candidate (2) and the same blocker: a cached
    board flag needs invalidation at every battlefield mutation. The cheaper
    half is the caller side — `available_mana` and `mana_source_table` are
    both called from sweeps that could share one scan — but read the
    twenty-seventh pass's negative result first and **count the sweeps, not
    the calls**, before costing it.

**The profile of record was retaken at `5034eb2f`** (2026-08-13, the
twenty-eighth pass's tip). Only the top table was re-derived; every count,
per-caller list and share inside the candidates above is from the
`f814a13b` retake and reads ~7 % high in absolute Ir. Re-read a candidate
on the current tip before costing it, not before ranking it — the ordering
did not move.

Methodological notes, each learned the hard way:

- **Ir over-weights allocation and representation changes.** Callgrind runs
  the *system* allocator (valgrind replaces malloc), so a row that removes
  allocations reads far larger there than it ships at `release` + mimalloc:
  the `Printed<T>` row measured **-17.09 % Ir**, **+13.5 %** wall-clock on
  the system allocator, and **+1.7 %** at `release` with mimalloc. Ir tells
  you whether a change cut work and by how much; only a `release` run tells
  you what ships. For anything allocator- or cache-shaped, do both before
  quoting a throughput number. Hoisted here from the frozen candidate
  snapshots so it is not re-derived.
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

**The frozen candidate snapshots for passes twelve to eighteen are in git,
not here** (~290 lines, `git log -- PERF.md`). Every entry in them was
either paid by a later pass or restated above at a fresher share, so
keeping both meant reading two lists to learn one thing. The two items in
them that were *not* restated are hoisted: the "Ir over-weights allocation
and representation changes" warning is in the methodological notes above,
and the `ability_strip_in_scope` soundness device is in the Log's
filters-and-devices list. What follows is the longer-lived list — items
that outlive any one profile.


0. **Audit the bot's other whole-sweep calls the same way** — one caller
   asking for 40 answers and reading 1. `cast_candidates` is clean; the
   grep discipline stands: a bot-path call to any `compute_*` /
   `*_hand_cards` aggregate that reads one field is the smell. `view.rs` is
   the only legitimate caller of `compute_hand_affordances`. **(a)**
   `available_mana` per hand card — tried, no win, reverted twice; do not
   redo it without a board-size argument.
0.25 **`check_state_based_actions`** — 6.31 % at the thirteenth pass's base,
   9.67 % inclusive now. What is left is **the collects that *find*
   something**: 82,634 of them, ~1,700 Ir each, after the presence scan
   removed ~21 empty walks per sweep. An empty `collect()` does not
   allocate, so cost it with `--auto=yes` before gating.
0.5 **The transaction checkpoint.** The unconditional `perform_action`
   clone was ~9.5 % of the program; the fourteenth pass took the bot's dry
   runs and the mid-round priority pass out of it (-11.56 % net), and
   candidate 7 removed the zone unshare that followed. What remains is
   irreducible while the snapshot is unconditional. **The audit rule the
   correction taught: a `dry_run` site is only sound when the caller cannot
   read the state after an `Err`.**
1. **Make `ComputedPermanent` cheap to build** — both legs done
   (`Printed<T>` -17.09 % Ir, `ColorSet` -2.55 %, per-scope memo -1.91 %,
   hit rate 34.5 % against a 50 % ceiling). **The reasoning to keep**: a
   cache on `CardDefinition` is unsound because
   `Arc::make_mut(&mut card.definition)` mutates a uniquely owned
   definition in place (MDFC face-swap, "loses all abilities", keyword
   grants) — which is why the fix had to be a representation change.
1.5 **`effective_mana_abilities`** — 1.88 % over 67,468 calls (1,327 Ir
   each) after the `Cow` row and the land-type gate (-44.7 %). What is left:
   `battlefield_find`'s linear scan (candidate 11's shape),
   `granted_abilities_with`, `intrinsic_land_mana_abilities_with`, the
   `frozen_effects` lock, the `out` `Vec`. **Do not re-try freezing it**: a
   `with_frozen_layers` scope inside it measured **+0.03 %, a null.**
3. **The gather.** Still the largest engine function (9.98 % inclusive,
   ~7.6 % self across five file attributions). Every named sub-item the
   nineteenth-to-twenty-fifth passes listed is paid; what is left is the
   walk itself — the pre-scan loop over the battlefield (0.22 % in the
   per-card `keywords` match alone) plus the `GraveyardAnthem` pass, which
   walks both graveyards unconditionally and has no cheap presence gate.
4. **Memoize the gather outside freeze scopes.** Unchanged, and the blocker
   is invalidation, not caching. The layer system is the program:
   `computed_permanent` 14.76 % inclusive, the gather 9.98 %, heavily
   overlapping. Sub-item **(a)** — does `compute_battlefield` need to
   *materialize*, or can its callers take an iterator? — is closed by the
   nineteenth/twentieth passes (310 calls left).
5. **`Keyword::eq` — 0.50 % self, mostly from `compute_permanent_pass`** —
   linear scans of `Vec<Keyword>`. A bitset for the common keywords makes
   `has_keyword` O(1), shrinks `CardData`, and would also kill the gather
   pre-scan's per-card keyword loop. Rides along with item 1's reasoning:
   it must be a representation change, not a memo.
6. ~~`HashMap` hash choice~~ — **done, -0.942 %** (`841dd40b`,
   `crate::fxhash`), pulled as a determinism fix.
7. ~~CowBox sharp edge / per-card CoW~~ — **done, -25.6 %**
   (`CardInstance = Arc<CardData>`). *When a cost is structural — "every
   write to any card deep-copies the whole zone" — look for the type that
   makes the class impossible before enumerating its instances.*
8. **`legal_block_targets` per-pair requirement evaluation** — still does
   not appear on this profile; a view-layer path, not a bot path.
9. **Actor scaling — re-measure on real cores.** Four-core boxes make
   everything past 4 actors oversubscription.
10. **Effect-resolution recursion depth.** The 32 MB worker stacks
    (`RUST_MIN_STACK` in `.cargo/config.toml`, plus explicit `stack_size` on
    every worker) are still required — a robustness constraint, listed here
    so nobody "cleans up" the stack sizes.

**Closed / ruled out**, one line each; the reasoning is in git.

- *State-clone allocation traffic.* CoW-wrapping the per-turn tally
  collections moved the bench 0.1 %. Right about the tallies, wrong about
  the conclusion — the checkpoint cost was the zone *unshare* that followed,
  which candidate 7 removed.
- *The bot's affordance sweep* — `cast_candidates` calling
  `compute_hand_affordances` for one field. Fixed (+42 % fixed decks,
  +52.7 % sealed); the lesson generalises and is item 0.
- *`can_afford_in_state`'s fused scan* (+0.066 %), *`SimBases`* (+0.083 %),
  *hoisting `compute_permanent`'s CR 613.8 gate scans* (-0.007 %, LLVM had
  already hoisted them), *`CounterBag`* (+0.051 %, kept as a determinism
  fix), *widening `ColdState` to 126 fields* (+1.23 %), *a freeze scope
  inside `effective_mana_abilities`* (+0.03 %), *an event-kind mask over the
  trigger dispatcher* (`event_matches_spec` runs 1.22 times per dispatch —
  nothing to gate).
