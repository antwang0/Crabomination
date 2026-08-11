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

**Anchored 2026-08-11 at `ed4c152c`** (`release`, mimalloc — the shipped
configuration), i.e. on the twentieth pass's third row. The fourth
(`c7bdd850`) landed after and is worth **-0.343 %** by instruction count —
far under what `--bench` resolves here, so the anchor was not re-run for
it; a `release` rebuild is 22 minutes.

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz   <- a DIFFERENT box
host_calib_ms        47-52 across the sitting   <- within-sitting only
games                320
games_per_s          94.23 / 91.94 / 93.27 / 98.47 / 99.09 / 96.81
                     (mean 95.64, spread 7.48 %)
games_per_s_th       30.65 - 33.03
decisions_per_s      mean 57,748
turns_per_game       26.98
stalls               0 (0.00 %)
peak_rss_mib         21.7 - 22.3
determinism          ok (all 160 pairs split, on all 6 runs)
```

**95.64 against the 67.31 below is +42 %, and almost all of it is the
host — this anchor is NOT a throughput claim.** The twentieth pass is worth
**-4.897 % by instruction count**, and nothing in it can produce 42 %. The
tell is in the fingerprint, and it is unusually clean this time: `host_cpu`
reports a *different CPU model* (2.10 GHz against the previous anchor's
2.80 GHz) and `host_calib_ms` **47-52 against 53-70** — a nominally slower
part that runs this probe ~25 % faster, i.e. a different machine, not a
quieter hour on the same one. **Absolutes across these two blocks do not
chain**; use them only against a run on the same `host_cpu`. No paired A/B
was taken: at -4.897 % the change sits at the edge of what this file says
`--bench` can resolve at all, so the measurement of record is the
callgrind number, and the Ir numbers *do* chain (the pass's base rebuilt
here read 3,694,337,730 against the nineteenth pass's recorded
3,694,708,603, -0.010 %). `turns_per_game` 26.98 across five consecutive
anchors, `stalls` 0, determinism ok on all six runs, peak RSS 21.7-22.3
against 22.1-22.5.

**Cross-check at `f2fb6722` (the twenty-first pass's tip), 2026-08-11 — the
anchor is NOT refreshed, and the reason is the spread.** Eight `--bench`
runs in one sitting on this container: 91.49 / 94.20 / 93.00 / 93.61 /
96.02 / 96.14 / 101.55 / 87.57, **mean 94.20, spread 14.8 %** — twice the
committed anchor's 7.48 %, so this sitting resolves nothing at the pass's
1.423 % and re-anchoring on it would replace a tighter measurement with a
looser one. 94.20 against 95.64 is **-1.5 %, inside that spread**, while
the instruction count says the tip does 1.423 % *less* work; no
investigation is owed. The workload facts all match the anchor exactly:
`turns_per_game` **26.98** (a sixth consecutive anchor), `stalls` **0**,
`determinism` ok on all eight, `peak_rss_mib` 21.8-22.3 against 21.7-22.3,
`decisions_per_game` 603.9.

**Cross-check at `1112e709` (the twenty-second pass's tip), 2026-08-11 —
the anchor is NOT refreshed, and no regression is owed.** Eleven `--bench`
runs on this container. The first three, taken immediately after the
`release` link finished, read 90.14 / 90.59 / 89.36 (mean 90.03, spread
1.4 %) — which against the anchor's 95.64 is **-5.9 %**, past the noise
band, so eight more were taken on an idle box: 95.92 / 92.99 / 90.60 /
98.72 / 95.18 / 102.65 / 99.94 / 96.26, **mean 96.53, spread 12.5 %**.
**96.53 against 95.64 is +0.9 %**, and across all eleven the mean is 94.76
with a **14.0 % spread** — the same figure the twenty-first pass measured
here. *The tight three-run sample was the bottom of the distribution, not a
regression*: three runs is not a spread estimate on this box, and a sample
whose spread is smaller than the effect you are testing for is the trap.
The instruction count is the measurement of record either way and it says
the tip does **4.3 % less work than the anchor** (3,362,421,936 against
3,513,438,110). Re-anchoring on a 12.5 % sitting would replace a 7.48 %
measurement with a looser one, and `host_cpu` still reads *2.80 GHz*
against the anchor's 2.10 GHz box. Workload facts all match the anchor
exactly: `turns_per_game` **26.98** (a seventh consecutive anchor),
`stalls` **0**, `determinism` ok on all eleven, `decisions_per_game`
**603.9**, `decisions` **193,232** identical run to run, `peak_rss_mib`
21.4-22.0 against 21.7-22.3, `host_calib_ms` 45-55 against 47-52.

**The host fingerprint disagrees with itself here, and the calibration
probe is the half to believe.** `host_cpu` reads *2.80 GHz* — the
`4f3e86c0` anchor's box, not the `ed4c152c` anchor's 2.10 GHz one — while
`host_calib_ms` reads **46-62**, overlapping the 2.10 GHz box's 47-52 and
not the 2.80 box's own 53-70. A container that reports the older model
string and calibrates like the newer one is why the probe exists: compare
`host_calib_ms` before comparing absolutes, and never chain across two
blocks whose probes don't overlap.

The previous anchor, `4f3e86c0` (the nineteenth pass's tip), for the
record:

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        53-70 across the sitting   <- within-sitting only
games                320
games_per_s          66.65 / 67.67 / 66.82 / 66.92 / 67.82 / 67.95
                     (mean 67.31, spread 1.95 %)
games_per_s_th       22.22 - 22.65
decisions_per_s      mean 40,641
turns_per_game       26.98
stalls               0 (0.00 %)
peak_rss_mib         22.1 - 22.5
determinism          ok (all 160 pairs split, on all 6 runs)
```

**67.31 against the 71.29 below was -5.6 %, and it was the host — checked,
not asserted**, by the method this file recommends for exactly this
situation and which costs about fifteen minutes. The tip's two perf commits
were reverted out of `combat.rs` + `game/mod.rs`, both sides rebuilt
`profiling-fast --no-default-features`, and the binaries alternated
`--bench` in one sitting: base mean **56.55** against tip **57.58**,
**+1.82 % paired, 4/6 pairs positive**. (A seventh pair was discarded —
`host_calib_ms` read **525**, i.e. something else had the box. That is what
the probe is for.) Supporting: `host_calib_ms` 53-70 against 50-62 on the
previous anchor; -1.968 % Ir with bench output byte-identical; both changes
strictly *remove* allocations, so there is no cache-shaped story in which
Ir falls and wall-clock rises. No wall-clock delta was claimed for that
pass either.

The previous anchor, `56986d65` (the eighteenth pass's tip), for the
record:

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        50-62 across the sitting   <- within-sitting only
games                320
games_per_s          72.34 / 73.14 / 68.18 / 71.04 / 71.21 / 71.80
                     (mean 71.29, spread 7.3 %)
games_per_s_th       22.73 - 24.38
decisions_per_s      mean 43,046
turns_per_game       26.98
stalls               0 (0.00 %)
peak_rss_mib         22.0 - 22.2
determinism          ok (all 160 pairs split, on all 6 runs)
```

**71.29 against the 69.13 below is +3.1 %, and the pass is worth -1.155 %
by instruction count — so most of that gap is the host, and no wall-clock
delta is claimed.** The tell is in the probe: `host_calib_ms` reads **50-62
here against 49-85** on the previous anchor, i.e. the same box on a
markedly quieter sitting, and this run's spread (7.3 %) sits inside the
documented ±8 %. The anchor is refreshed because the tip moved, not because
the wall-clock did. What the run does establish, and what it is here for:
**stalls 0 and determinism ok on all six runs**, `turns_per_game` 26.98
unchanged across three consecutive anchors, and peak RSS 22.0-22.2 MiB
against 21.8-22.4.

The previous anchor, `6ed3dbfc` (the seventeenth pass's tip), for the
record:

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        49-85 across the sitting   <- within-sitting only
games                320
games_per_s          70.53 / 70.76 / 66.21 / 69.68 / 68.05 / 69.57
                     (mean 69.13, spread 6.6 %)
games_per_s_th       22.07 - 23.59
decisions_per_s      mean 41,746
turns_per_game       26.98
stalls               0 (0.00 %)
peak_rss_mib         21.8 - 22.4
determinism          ok (all 160 pairs split, on all 6 runs)
```

**Another container again, and 69.13 against the 81.93 recorded at
`abb2b502` is host, not a regression** — the evidence, in order.
(a) `host_calib_ms` reads 49-85 here against 45-55 there. (b) Every one of the pass's four changes strictly
*removes* work, and callgrind — deterministic on a fixed workload —
measures **-3.316 %** instructions across them with the six games' output
byte-identical on every row. (c) The one confound worth ruling out was the
`encode.rs` commit that landed on the branch between the pass's base and
its tip: with `codegen-units = 1` + thin LTO a 335-line addition to the
engine crate can move inlining crate-wide. Re-running callgrind on the
*merged* tip read **3,817,731,167 Ir against the pass's 3,817,208,224 —
+0.014 %**, i.e. inside the documented ~0.02 % build-to-build noise, so the
encoder is inert for the non-net bench path and the pass's number holds on
the merged tree. **No wall-clock delta is claimed for this pass**; the
anchor is here for the stall / determinism / RSS record and for the next
run's host comparison. `turns_per_game` 26.98 unchanged, `stalls` 0 and
determinism ok on all six runs, peak RSS 21.8-22.4 MiB against 21.7-22.2.

The previous anchor, `abb2b502` (the sixteenth pass's tip), for the record:

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        45-55 across the sitting   <- within-sitting only
games                320
games_per_s          84.74 / 85.93 / 83.64 / 81.24 / 80.56 / 81.35 / 78.71 /
                     79.29   (mean 81.93, spread 9.2 % — take >=6 runs)
games_per_s_th       26.24 - 28.64
decisions_per_s      mean 49,475
turns_per_game       26.98
stalls               0 (0.00 %)
peak_rss_mib         21.7 - 22.2
determinism          ok (all 160 pairs split, on all 8 runs)
```

The previous anchor, `28629ba9` (the fifteenth pass's tip), for the record:

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        52-64 across the sitting   <- within-sitting only
games                320
games_per_s          65.77 / 66.18 / 66.65 / 63.82 / 63.13 / 63.28 / 62.43 /
                     62.22   (mean 64.19, spread 7.1 % — take >=6 runs)
games_per_s_th       20.74 - 22.22
decisions_per_s      mean 38,760
turns_per_game       26.98
stalls               0 (0.00 %)
peak_rss_mib         23.3 - 23.8
determinism          ok (all 160 pairs split, on all 8 runs)
```

**This anchor is not a measurement of the pass and must not be subtracted
from the one below it.** It is a different container: `host_calib_ms` reads
52-64 here against 50-60 there, i.e. a slightly *slower* host, and the run
reads 64.19 against 62.48 — the direction agrees with the -3.87 %
instruction count but the magnitude means nothing. **The pass's measurement
is the callgrind number, and deliberately only that**: -3.87 % is inside
this box's wall-clock noise (the record below has eight alternated pairs of
a -1.91 % change reading +0.7 %), so no wall-clock delta is claimed. What
this run *is* good for is the rest of the row — `turns_per_game` 26.98
unchanged, `stalls` 0, determinism ok on all eight runs, peak RSS 23.3-23.8
MiB (up ~1.5 MiB on the previous container; not investigated, the code
allocates strictly less).

The previous anchor, `3e2ee6cb` (the fourteenth pass's tip), for the record:

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_calib_ms        50-60 across the sitting   <- within-sitting only
games                320
games_per_s          62.77 / 62.82 / 62.29 / 62.59 / 59.80 / 62.62 / 63.47 /
                     63.50   (mean 62.48, spread 5.9 % — take >=6 runs)
decisions_per_s      mean 37732
turns_per_game       26.98
stalls               0 (0.00 %)
peak_rss_mib         21.7 - 22.1
determinism          ok (all 160 pairs split, on all 8 runs)
```

**62.48 is *lower* than the 71.52 anchor it replaces and the code is 11 %
faster. Read this before comparing any two anchors again.** The 71.52 was
taken on a different container (`host_calib_ms` 45-54 against 50-60 here);
this sitting's *base* binary measures the difference directly and says the
box, not the code, moved. The pass's own delta is a paired A/B taken in one
sitting on this container: **52.18 -> 57.97 games/s, +11.10 % mean, 6/6
alternated `release-fast` pairs** of `95406ebe` against `3e2ee6cb`
(per-pair +9.81 / +9.03 / +13.70 / +14.05 / +7.69 / +12.46 %;
`decisions_per_s` 31,511 -> 35,338, +12.1 %; `turns_per_game` 26.98 and
`stalls` 0 on all twelve runs). **It agrees with the -11.56 % instruction
count to half a point**, which is what a change that removes work *and*
allocations looks like — contrast the thirteenth pass's null.

**The base binary is a usable control and cost nothing.** `bl_base` read
52.38 mean and, an hour later, 52.18 on the same box: within-sitting
drift here was ~0.4 %, so this sitting's pairs are trustworthy and the
71.52/62.48 gap is not drift but a different machine. **Keep the base
binary and re-run it, rather than reasoning about `host_calib_ms` alone.**

The pre-merge anchors below are compacted to their lessons; the numbers
themselves are in git, and none of them compares to the anchor above.

- **70.65 belongs to a different bench.** It predates `998b2433` making
  `EvalWeights::default()` carry `determinize: 1`, and `--bench` runs
  `gang` = `EvalWeights::default()`, so the *workload* changed underneath
  it. **Read `host_calib_ms` before comparing to any older anchor.**
  Refresh only alongside an intentional, explained change; regressions
  beyond ~5 % get investigated before anything else lands.
- **Absolutes do not transfer between containers, and one pass proved it
  twice.** The same engine code read 60.49 and 64.42 in two sittings on one
  box (+6.5 % of pure drift), and a second container read 55.70 on its own
  tip where the first read 55.88 on a tip with 1.0 % *fewer* instructions.
  **Quote a paired A/B measured in one sitting, never a difference of
  anchors.** That pass's own paired deltas: callgrind -11.68 %, wall-clock
  +9.57 % (6/6 pairs) and +10.85 % (8/8 pairs) in two sittings whose work
  overlaps, so they don't add.
- **The eleventh pass, end to end**: 8/8 alternated `release` + mimalloc
  pairs in one sitting, 66.69 -> 70.65 games/s (**+5.93 %**, median paired
  +4.31), against callgrind **-8.63 %** over the same span. The wall-clock
  lands at about two thirds of the instruction win, which is the expected
  shape when four of five rows remove *gathers* and one removes
  *allocations* that mimalloc absorbs — see the `Printed<T>` row, where
  -17.09 % Ir was worth +1.7 % at `release`.

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

**Twentieth pass.** Base `3655c37c` (the nineteenth pass's tip),
re-measured on a fresh container: **3,694,337,730 Ir** against the recorded
3,694,708,603 — **-0.010 %**, so the two passes' numbers chain. Every row
is callgrind on `profiling-fast --no-default-features`, `--a gang --b gang
--games 6 --threads 1 --seed 1 --decks fixed`, and every one left the six
games' bench output byte-identical apart from the wall-time line.

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-11 | `declare_attackers_banded` gates its whole-board pass (`31116d43`) | 3,694,337,730 Ir | 3,630,334,304 Ir (**-1.733 %**) | The site the nineteenth pass named as the top candidate: 61,859,555 Ir / 1.64 % over 3,780 calls. Exactly two whole-board consumers — the CR 508.1d "attacks each combat if able" loop and the Magnetic Web `AttackTogether` loop — against a dozen `find(id)` lookups. `groups` and the Oracle en-Vec mandate hoist above the pass (both read only the battlefield and the active statics), and the gate, the subset and the whole-board fallback share one freeze scope. New `GameState::board_keyword_in_scope`. |
| 2026-08-11 | `declare_blockers` gates its validation pass the same way (`911cf298`) | 3,630,334,304 Ir | 3,593,030,829 Ir (**-1.028 %**) | 41,289,574 Ir / 1.10 % over 2,598 calls, and the candidate's prediction to the tenth of a point. Five whole-board consumers, all CR 509.1 requirement loops; four are already skipped by an attacker keyword or by `must_block`, which the subset answers, and only "blocks each combat if able" reads the computed view before the keyword that decides it. Subset = declared blockers + their attackers + `self.attacking` + `block_map`'s existing blockers. |
| 2026-08-11 | The three per-turn whole-board passes ride a presence gate (`ed4c152c`) | 3,593,030,829 Ir | 3,513,438,110 Ir (**-2.216 %**) | The three sites the nineteenth pass's table called "legitimate — one pass per turn" are not: each was ~23 layer passes to build an empty set. `process_cumulative_upkeep` 23,239,540 (0.65 %), `do_phasing` 22,747,396 (0.63 %), `do_untap` 22,686,075 (0.63 %) — Phasing, `CumulativeUpkeep(_)`, `DoesntUntapWhileCounter(_)` / `DoesntUntapIfAttackedLastTurn`, none of them on a bench board. `board_keyword_matching` is the predicate form of the gate, for the payload-carrying keywords. **The win is larger than the three site costs** (79.6 M against 68.7 M) because the freeze scope also folds each site's gather in with the pass it gates. `compute_battlefield` calls **5,488 -> 310**: `submit_decision` is the only whole-board caller left and the per-turn path has none. |
| 2026-08-11 | The dispatcher's four delayed-trigger scans ride one `is_empty` (`c7bdd850`) | 3,513,438,110 Ir | 3,501,374,248 Ir (**-0.343 %**) | `dispatch_triggers_for_events` runs 52,332 times over six games and four of its blocks — the `WhenCardDies` watch, the Tamiyo attack watch, the two turn-scoped First Day of Class / Waltz of Rage watches — each scan the event batch into a `Vec` *before* asking whether any `delayed_triggers` entry wants it. Nearly always none is registered, so all four come back empty; they now read an empty slice. Exact by construction: every consumer of the four `Vec`s sits inside an `if !xs.is_empty()` whose body only ever fires a `delayed_triggers` entry. **The rest of that function is still 4.3 % of self cost, ~2.8 % of it `Vec` machinery over 366,316 collects (7 per dispatch)** — this row took the four cheapest; the candidate list keeps the rest. |
| | **cumulative, twentieth pass** | **3,694,337,730 Ir** | **3,501,374,248 Ir (-5.223 %)** | four rows, all callgrind on the fixed six-game workload; no wall-clock delta claimed (see **Baseline** — this box cannot resolve 5 % by `--bench`). |

**What the pass leaves behind, as a rule.** *"One pass per turn" is not a
reason to keep a whole-board layer pass — it is a reason nobody looked.*
The nineteenth pass's table wrote off `do_phasing` / `do_untap` /
`process_cumulative_upkeep` as legitimate on the strength of their call
counts, and all three were the cheapest gate in the file: a single keyword
that no board carries. **The filter that finds the next one:** a
`compute_battlefield` (or a per-card `computed_permanent` loop) whose
`filter` names one `Keyword` variant. The gate is
`board_keyword_in_scope` / `board_keyword_matching`, it is `false`-
authoritative by construction, and both halves are audited across the whole
suite — a debug-only whole-board re-run inside the gate, plus a
`debug_assert!` at the subset's read sites that a battlefield permanent was
never read outside it. That second half is what makes the subset safe to
widen later.

The twenty-first pass. Base `e2d030c6` rebuilt here read
**3,501,692,629 Ir** against the twentieth pass's recorded 3,501,374,248 —
**0.009 % apart**, so the passes' numbers chain. Same workload and build
as every row above.

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-11 | Payment's cost relaxation borrows and walks the board once (`7c75fb94`) | 3,501,692,629 Ir | 3,497,826,864 Ir (**-0.110 %**) | `try_pay_after_snapshot_mode` is 14.12 % of the profile and its first two statements are pure preamble: `relax_cost_colors_for_spell` cloned the `ManaCost` on the common path, and `spend_mana_as_any_color_for_spell` took *two* battlefield passes to decide there was nothing to relax (one for the seat-agnostic `PlayersMaySpendManaAsAnyColor`, one for the two named-spell permissions). Now `Cow::Borrowed` plus one fused walk. At the edge of what a single A/B pair resolves, but the change strictly removes a clone and a board pass, so the sign is not in question. |
| 2026-08-11 | `ColdState`'s 15 id sets are `Vec`-backed (`271c7d14`) | 3,497,826,864 Ir | 3,483,193,405 Ir (**-0.418 %**) | Found in `--tree=caller` under `RawTable::clone`, which has exactly two callers: **`Arc::clone_from_ref_in` 984,988x / 42,825,844 Ir / 1.22 %** (the `ColdState` CoW unshare, 22 tables per unshare, ~44.8 k unshares for six games) and `GameState::clone` 302,418x / 0.41 %. An empty `hashbrown` table clone still walks its control bytes. `IdSet<T>` is a `Vec` newtype carrying `HashSet`'s API for the six methods these fields use — **nothing iterates one**, so the swap is 15 type annotations and no call-site changes — and it serializes as the same sequence. The seven `HashMap` fields in the group are left: their JSON shape is an object, and changing it moves the snapshot format. |
| 2026-08-11 | `died_card_snapshots` is insertion-ordered, not a `HashMap` (`ea8cc1fd`) | 3,483,193,405 Ir | 3,473,495,633 Ir (**-0.278 %**) | **A determinism fix that also pays.** `dispatch_triggers_for_events` walks `.values()` pushing `TriggerCandidate`s, and a candidate's position decides where its ability lands on the stack — nothing between there and `drain_trigger_queue` sorts, so two creatures dying to the same damage stacked their LKI triggers in `RandomState` order. `IdMap<K, V>` is the map sibling of `IdSet`. **Four times the 15 sets' per-field rate on one field**, because this one is *populated* on every death: a non-empty table clone allocates a table plus control bytes where a `Vec` allocates once. The bench never hits two-simultaneous-deaths, which is why golden traces never caught the defect. |
| 2026-08-11 | `auto_tap`'s inner loops stop rebuilding constants (`f2fb6722`) | 3,473,495,633 Ir | 3,451,879,102 Ir (**-0.622 %**) | Three rebuilds in one function (13.46 % over 8,892 calls), two of them with a comment asserting the opposite. **`redundancy` is constant across the generic-pip loop** — the comment said it is recomputed "because tapping one source changes how redundant the rest are", but it reads `sources`, the local table the loop never mutates, so it was O(pips × live × sources × colours) to rebuild one ranking; hoisted, and gated on `generic_to_tap > 0` because most costs have no generic pip and the build is O(sources² × colours). The same loop collected a `live: Vec<&ManaSourceInfo>` per pip only to `min_by_key` it — fused, and `min_by_key` keeps the first of equal keys either way. And `avail` / `prod` were `HashMap<ManaColor, u32>` over five fixed keys, i.e. an allocation plus a SipHash per cost symbol, now `[u32; 5]` indexed by `color_index`. |
| 2026-08-11 | `auto_tap` builds its source table only when it will tap (`1ec589d1`) | 3,451,879,102 Ir | 3,424,021,668 Ir (**-0.807 %**) | Candidate (0)'s probe (a) and its fix in one. `mana_source_table` is 134,127,895 / 3.83 % over 8,892 calls — one `effective_mana_abilities_with` plus a five-colour scan per untapped permanent, in a freeze scope — and `auto_tap_for_cost_inner` built it as its *first* statement, before reading the pool. Every caller whose floating mana already covered the cost paid a layer pass per permanent to then tap nothing. The build moves below the hybrid resolution (which needs `untapped_producers_of`, not the table) and behind `still_need_colors.is_empty() && generic_to_tap == 0`. Exact by construction: the table is `&self`, nothing between the two positions mutates, and `events` is still empty at the gate. |
| | **cumulative, twenty-first pass** | **3,501,692,629 Ir** | **3,424,021,668 Ir (-2.218 %)** | five rows, callgrind on the fixed six-game workload; bench output identical on every row |

**The nineteenth pass's empty-table finding, corrected.** That pass
concluded from `CounterBag` (+0.051 %) that "`hashbrown` short-circuits the
empty-table clone, so those clones pay ~42 Ir for a branch and a `Vec`
clone costs about the same." The per-call half is right; the conclusion
drawn from it was too strong. **Both halves of this pass's row say so**:
672 k empty-set clones removed read -0.418 %, i.e. ~22 Ir each rather than
~0, because an empty `Vec` clone is a two-word copy where an empty table
clone is a call with a branch and a drop; and `died_card_snapshots`
(`ea8cc1fd`) is worth **-0.278 % on one field**, four times the 15 sets'
per-field rate, because it is *populated* on every death — a non-empty
table clone allocates a table plus control bytes where a `Vec` allocates
once. **The rule: count the clones and ask whether the collection is ever
non-empty at clone time.** `CounterBag`'s was empty on nearly every card;
these are not.

**What the pass leaves behind.** The `RawTable::clone` row is *half* paid:
15 of the 22 `ColdState` tables are gone, and the two remaining blocks —
seven `ColdState` maps, nine `GameState` hash fields (0.41 % on its own) —
are the same shape with a serde question attached. See candidate (1).

The twenty-second pass. Base `18b83e6a` (the twenty-first pass's tip)
rebuilt here read **3,423,919,639 Ir** against the recorded 3,424,021,668 —
**0.003 % apart**, so the passes' numbers chain. Same workload and build as
every row above.

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-11 | `empty_mana_pools` gates its three board scans on the seats (`ef731ecc`) | 3,423,919,639 Ir | 3,399,657,945 Ir (**-0.709 %**) | The function's per-seat fast path already skipped a seat with no floating mana, no kept mana and no firebending mana *whatever the board said* — so when every seat is in that state, its preamble (a `keepers` collect, an `all_persist` `any()`, a `color_keepers` `flat_map` collect, two of them allocating) computed inputs nobody read. 51 k calls per six games, 22,108 of them from `pass_priority` alone, and the overwhelming majority have nothing to empty. Gate is three field reads per seat. |
| 2026-08-11 | The `TappedForCostPower` probe runs only for tap-another costs (`df35df04`) | 3,399,657,945 Ir | 3,374,960,020 Ir (**-0.727 %**) | **A `serde_json::to_string` of the whole effect tree, in the activation path, hoisted above its only reader.** `activate_ability_inner` picked which creature a "tap another permanent" cost taps by serializing `ability.effect` and substring-searching for `TappedForCostPower` — on *every* activation, 36,698 per six games, 18,340 of them auto-tap mana abilities with no tap-another cost. `serde_json::ser::to_vec` was 0.61 % of a run that writes no JSON and `format_escaped_str` ran 109,966 times. Moved inside the `tap_other_filter.is_some()` arm with its `auto_tap_pick` closure; nothing else reads either. **How to find the next one: `--tree=caller` on `malloc`, and read the caller names for anything that has no business allocating** — `serde_json` in a bot game is the same tell as `format!` in a comparator. |
| 2026-08-11 | Damage triggers build their grant set once per event, not per kind (`1112e709`) | 3,374,960,020 Ir | 3,362,421,936 Ir (**-0.372 %**) | The `_for`-shim-inside-a-loop shape again, and this time the loop is a *kind list*: `fire_combat_damage_to_creature_triggers` calls `fire_combat_damage_triggers` four times for one damage event (`DealsCombatDamageToCreature`, then three combat-agnostic wordings) and the player half does the same, each call rebuilding `statics_granted_triggers_for(source)` — i.e. `trigger_grant_sources`, a walk of every static ability in play. The set is board-level and kind-independent; the callers build it once. Shim calls **28,564 -> 7,118**. |
| 2026-08-11 | `pick_removal_ping` reuses its grant scan (`084e4126`) | 3,362,421,936 Ir | 3,361,424,559 Ir (-0.030 %) | **No win — kept as dead-work removal, not claimed as a row.** The function built `state.grant_scan()` twice, the second shadowing the first, with only `&GameState` reads between the two loops. -0.030 % is well under the ~0.1 % a single A/B pair resolves here, so it does not go in the cumulative. Kept because it is a pure deletion of a redundant pure call — the sign is not in question — and the walk it drops is O(permanents x static abilities), invisible on a 2-player bench board and not on a 4-player Commander one. **The lesson is the estimate**: the `--tree=caller` malloc rows put 28,552 allocations under this function, which read as tens of thousands of calls; the second scan was worth 1 M Ir, so most calls return from the lethal-ping loop above it. *Count the calls at the line you are changing, not at the function.* |
| | **cumulative, twenty-second pass** | **3,423,919,639 Ir** | **3,362,421,936 Ir (-1.796 %)** | three rows, callgrind on the fixed six-game workload; bench output byte-identical on every row. The `084e4126` line below the total is deliberately excluded — it is under the claim floor |

**What the pass leaves behind.** Two of its three rows are the same lever
the file has pulled a dozen times — *ask the cheap question before building
the expensive answer* — but the middle one is a new shape and worth its own
filter: **a hot path that answers a structural question about an `Effect`
by serializing it.** The remaining `serde_json::to_string`/`to_vec` sites
under `game/` are none, checked; the sibling shape (`format!("{:?}", …)`
inside a comparator) survives at `mod.rs:19045`, in a sort whose call count
is not on this profile. The `trigger_grant_sources` block is *half* paid:
53 M / 1.57 % over 110,540 calls at the pass's base, of which the dispatcher's
own 52,332 (0.71 %) and `fire_step_triggers`' 14,462 (0.20 %) are already
one-per-batch and cannot be hoisted further — what is left is making the
scan itself cheaper, not calling it less. See candidate (2).

The twenty-third pass. Base `a9d62f11` (the twenty-second pass's tip)
rebuilt here read **3,361,108,555 Ir** against the recorded 3,361,424,559 —
**0.009 % apart**, so the passes' numbers chain. Same workload and build as
every row above.

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-11 | `fire_combat_damage_triggers` takes the whole kind list (`08cbc9c3`) | 3,361,108,555 Ir | 3,325,777,827 Ir (**-1.051 %**) | Candidate (4), and the other half of `1112e709`: that row stopped the *callers* rebuilding the grant set per kind, this one stops the *callee* re-walking the board per kind. One damage event fires four `EventKind`s and each call re-walked the battlefield five times — printed/granted on the dealer, attached Equipment, attached Auras, soulbond partners, `YourControl` listeners, `AnyPlayer` dealer-filter listeners — to filter on `t.event.kind`. The walks are kind-independent; the signature becomes `kinds: &[EventKind]` and each walk buckets into `by_kind[i]`, drained in `kinds` order at the bottom so the stack sees exactly the per-kind batches it did. **The graveyard walk stays per-kind**: `gy_combat_trigger_fired_this_step` is read and written between kinds, so a merged walk would let one graveyard card fire for two kinds of one damage event. Calls **28,564 -> 7,118**; bench output byte-identical. |
| 2026-08-11 | The dispatcher's delayed-trigger block rides one `is_empty` (`b925063c`) | 3,325,777,827 Ir | 3,317,550,360 Ir (**-0.247 %**) | Candidate (6a), and the rest of what `c7bdd850` started: that row made the four watch blocks scan an *empty slice*, and the five `collect`s still ran on all 52,332 dispatches. The whole block moves to `fire_delayed_event_watchers`, called only when `delayed_triggers` is non-empty. Exact by construction — every consumer inside sits in an `if !xs.is_empty()` whose body only fires a `delayed_triggers` entry — and the gate is read once at the top, as `watch_events` was, so a `retain` inside cannot change which later blocks run. **The ratio is the lesson**: `c7bdd850` bought -0.343 % by not *scanning*, this one -0.247 % by not *collecting*, so an empty `collect` is ~150 Ir, not ~0 — but the scan half was still the larger one. Also -232 lines from the file's largest function. |
| | **cumulative, twenty-third pass** | **3,361,108,555 Ir** | **3,317,550,360 Ir (-1.296 %)** | two rows, callgrind on the fixed six-game workload; bench output byte-identical on both, 18,612 tests green, golden traces unchanged |

**What the pass leaves behind.** Both rows are the *kind/batch* form of the
file's oldest lever — *ask the cheap question before building the expensive
answer* — and between them they close the two halves of the damage-trigger
and delayed-trigger blocks. Candidate (4) is now paid; candidate (6a) is
paid for the delayed-trigger collects and **not** for the three that
remain per dispatch (`synthesized`, `equip_granted_trigger_sources`, the
per-card `all_triggers`). Candidate (2) was costed this run and **not
taken**: `trigger_grant_sources`' remaining ~89 k calls are ~480 Ir each
over ~20 cards, i.e. ~24 Ir per card for the iterator plus an empty
`static_abilities` loop, and `active_static` returns on its first `match`
arm for a non-wrapper effect — so a syntactic peel saves almost nothing and
the only real lever left is a *cached* board-level flag, which needs
maintenance at every battlefield mutation. Cost that maintenance before
writing it.

The twenty-fourth pass. Base `4d5fbc69` (the twenty-third pass's tip)
rebuilt here read **3,318,705,480 Ir** against the recorded 3,317,550,360 —
**0.035 % apart**, so the passes' numbers chain. Same workload and build as
every row above.

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-11 | Step triggers filter before cloning (`006d5966`) | 3,318,705,480 Ir | 3,256,158,471 Ir (**-1.885 %**) | `fire_step_triggers` built its candidate list as `triggered_abilities.iter().cloned().chain(granted).filter(kind).filter(scope)` — **every printed trigger on every battlefield permanent cloned, then discarded unless it matched this step's kind**, and a `TriggeredAbility` clone is an `Effect` tree plus the event's filter predicate. Filter on the borrowed ability, clone the survivors; the per-card `collect::<Vec<_>>()` inside the `flat_map` goes with it, and the per-card grant walk takes the dispatcher's presence gate. The function's collect row was 67,972,378 / 2.05 % over 14,462 calls and is now **absent from the collect table entirely**. Order unchanged: printed then granted, per card, in battlefield order. |
| 2026-08-11 | The SBA sweep's Hushbringer scan is gated on there being a death (`41551bcc`) | 3,256,158,471 Ir | 3,253,878,954 Ir (**-0.070 %**) | `creature_dies_triggers_suppressed` walks the battlefield x `static_abilities`, and `check_state_based_actions` ran it on all 82,634 sweeps although both reads sit inside `for id in dead`. `!dead.is_empty() && …` is exact — the value is only ever observed when the loop runs. Small because the scan inlines into the sweep and never surfaced as its own profile row; kept for that plus the invariant it states. |
| 2026-08-11 | The trigger dispatcher takes one board scan, not four (`f28faaa0`) | 3,253,878,954 Ir | 3,217,512,298 Ir (**-1.118 %**) | Candidate (6a'), generalized. The dispatcher opened with **four** separate walks of `self.battlefield`, three of them also walking each card's `static_abilities`: `creature_dies_triggers_suppressed`, the ability-strip presence gate, `trigger_grant_sources`, `equip_granted_trigger_sources`. `dispatch_board_scan` -> `DispatchScan` fuses them, the device `stack.rs` already used for `SbaBoardScan`. `ability_strip_in_scope` splits into `ability_strip_off_battlefield` plus a caller-supplied battlefield leg. A `debug_assert!` in the fused scan cross-checks all four fields against the functions it replaces, so the whole suite audits it against drift — the same guarantee the strip gate already carried. |
| 2026-08-11 | Spell-cast triggers walk the battlefield once (`125557eb`) | 3,217,512,298 Ir | 3,201,568,157 Ir (**-0.496 %**) | The same shape one function over, plus one of its own: `fire_spell_cast_triggers` collected `live: Vec<(CardId, usize)>` and then re-`find`-ed each id in the battlefield — **O(cards²) for nothing**, since nothing between the collect and the loop mutates. Three board walks become the one `DispatchScan`, the per-card grant lookups take the presence gates, and `stripped` is the `Vec` the gate returns instead of a `HashSet` built from it (empty on almost every board, which a slice answers with a length check). |
| 2026-08-11 | The combat damage step's read-only prefixes each hold one freeze scope (`56f6623f`) | 3,201,568,157 Ir | 3,177,885,139 Ir (**-0.740 %**) | Candidate (9), shape (a). The five damage-path `computed_permanent` callers are 138.5 M / 4.33 % and **each already freezes internally**, so there is no intra-call duplicate left; a blanket freeze over the damage loop is unsound because lifelink moves life totals between the reads and a `WhileCondition` static can read life. What *is* sound is each iteration's read-only prefix — every call from the first prevention probe through `scale_damage_to` is `&self` and `apply_prevention_shields` is the first `&mut self`. Three scopes: the per-(attacker, blocker) prefix (six probes + the scaling, `continue` becoming a `None` return); the strike-back gate once per attacker (`dealing_blocker_ids`' seven filters, one `damage_prevented_by_protection` per blocker, plus `attacker_takes_strike_back`); and the per-blocker strike-back prefix. Short-circuit order and every predicate unchanged. |
| | **cumulative, twenty-fourth pass** | **3,318,705,480 Ir** | **3,177,885,139 Ir (-4.243 %)** | five rows, callgrind on the fixed six-game workload; bench output identical on every row (24 decided, 12 splits, rho -1.000), 18,612 tests green, golden traces unchanged |

**Twenty-fifth pass — a concurrent session, and the first collision this
file has had to record.** Two sessions pulled candidate (9) shape (a) at
the same time from the same base `10cb8fbf`, wrote **functionally
identical** code for the damage step's three prefixes (same three scopes,
same predicates, same short-circuit order; the diff is variable naming and
comment wrapping), and measured it independently at **-0.740 %**
(`56f6623f`, kept) and **-0.698 %** (dropped in the rebase). *That 0.042 %
spread between two containers on the same source is the best cross-session
reproducibility datapoint the file has* — it is twice the 0.02 %
build-to-build figure and still well inside the 0.1 % claim floor, so the
floor is doing its job. The base measurements agreed to 0.016 %
(3,201,053,304 against the recorded 3,201,568,157).

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-11 | Target legality takes one freeze scope for the whole check (`5d4b5402`), plus the stall instrumentation (`419d2ea6`) and `STALE_ROUNDS` (`15ec11c1`) | 3,177,885,139 Ir | 3,159,019,265 Ir (**-0.594 %**) | **Shape (a) is not a damage-path item — it is a *family*, and this is the second member.** `check_target_legality_with_source` is `&self` end to end and reads the layer system three times: `permanent_has_keyword` for Shroud, again for Hexproof, and a `computed_permanent` keyword scan for Artifact Ward. Body moves to `check_target_legality_inner`; the entry point is one `with_frozen_layers`. Measured in isolation on the dropped twin's tip it read **-0.516 %** (3,178,703,430 -> 3,162,316,313) with `permanent_has_keyword` **21,988,411 -> 11,808,343 Ir at an unchanged 8,328 calls** and gathers 134,116 -> 127,878 — that is the row's own mechanism, off the caller table. The -0.594 % above is the rebased span and so also carries the two instrumentation commits, which add one `StopReason` branch per *game*; the difference between the two figures is that plus noise, not a second win. Bench output identical (24 decided, 12 splits, rho -1.000). |


**What the pass leaves behind.** The largest row in five passes was not a
gate, a memo or a representation — it was **`cloned()` sitting *before* the
`filter` that throws the clones away**, in a function every step of every
turn calls. Nothing about the call counts said so: `fire_step_triggers` was
2.55 % inclusive and its 14,462 calls looked one-per-step and therefore
fine. **The filter that finds the next one: an iterator that clones and
then narrows.** `.cloned()` / `.to_vec()` / `.clone()` followed (however
many adapters later) by a `.filter` or an `if` on a field the *borrowed*
item could have answered. The whole-workspace sweep for the exact shape
(`\.cloned\(\)\s*\n\s*(\.chain\(…\)\s*\n\s*)?\.filter`) is clean after this
row, but the sweep only catches the syntactic form — the semantic one is
"how much of what this loop builds does it keep?".

Second lesson, from the two `DispatchScan` rows: **a fused board scan is
worth writing even when each walk it replaces is individually invisible.**
`creature_dies_triggers_suppressed` and `equip_granted_trigger_sources` do
not appear anywhere in the profile — they inline — and the four walks
together were still 1.118 % of the program. The count that predicts the win
is `walks x battlefield x call count`, not any one walk's profile row.

Third, from the damage-prefix row: **when every call on a hot path already
freezes internally, the remaining duplicate gathers are *between* the
calls, and the scope that removes them is bounded by the first `&mut self`
call, not by the loop.** The eleventh pass's rule ("a `&mut self` path
taking a `computed_permanent` outside a freeze scope") finds single sites;
this is its plural form. **How to find the next one:** take a caller from
`--tree=caller` on `computed_permanent` whose per-call cost is ~2,000+,
read *upward* to the nearest `&mut self` call, and ask how many other
gathering reads sit between the two.

## Profile of record

Callgrind on `profiling-fast --no-default-features` (= `release-fast` opt
settings + debuginfo; system allocator, because valgrind replaces malloc and
a mimalloc build would measure the interception), 1 thread, `--a gang --b
gang --games 6 --seed 1 --decks fixed`.

**Re-taken 2026-08-11 on the twenty-fourth pass's tip `125557eb`:
3,201,568,157 Ir.** Supersedes the twenty-second-pass table beneath it,
which is kept only for the rows it costed that are still live.

| Ir | share | site |
|---|---|---|
| 1,602,691,451 | 50.06 % | `pick_attacks_scored` (630 calls) — the search, still untouched |
| 1,595,447,621 | 49.83 % | `simulate_attack_outcome_once`, under it |
| 491,192,545 | 15.34 % | `would_accept` |
| 431,881,315 | 13.49 % | `try_pay_after_snapshot_mode` |
| 410,025,202 | 12.81 % | `auto_tap_for_cost_inner` |
| 359,094,195 | 11.22 % | `resolve_combat` |
| 311,764,753 |  9.74 % | `gather_continuous_effects_inner` (141,106 gathers) |
| 240,615,249 |  7.52 % | `dispatch_triggers_for_events` (52,332 calls) |
| 210,019,714 |  6.56 % | `pick_by_outcome` |
| 179,992,085 |  5.62 % | `cast_candidates` (7,024 calls) |
| 157,750,775 |  4.93 % | `compute_permanent` (248,654 calls) |
| 116,895,124 |  3.65 % | `mana_source_table` (7,370 calls) |
| 113,472,401 |  3.54 % | `check_state_based_actions` (82,634 calls) |

`fire_step_triggers` has left this table (2.55 % -> below threshold) and
the collect table with it; `fire_combat_damage_triggers` left at the
twenty-third pass. Shares rise where the denominator fell — `would_accept`
and `try_pay_after_snapshot_mode` are unchanged in absolute Ir.

**Who gathers, at this tip**: `computed_permanent` 107,936 (243,439,898 /
7.60 %), `frozen_effects` 18,916 (1.21 %), `check_state_based_actions`
10,670 (0.67 %), `compute_permanents` 3,274, `compute_battlefield` 310 —
**141,106 total**. The `computed_permanent` row is candidate (9).

The twentieth pass's tip `ed4c152c` read **3,513,438,110 Ir**. The layer
system has come down hard across the nineteenth and twentieth passes and
the top of the list has reshuffled; these supersede the seventeenth-pass
table below, which is kept for the rows it costed that are still live.

| Ir | share | site |
|---|---|---|
| 1,845,020,927 | 51.35 % | `pick_attacks_scored` (630 calls) — the search, still untouched |
| 534,428,881 | 14.87 % | `would_accept` |
| 507,575,030 | 14.13 % | `resolve_combat` |
| 507,252,955 | 14.12 % | `try_pay_after_snapshot_mode` — **new to this file** |
| 483,645,579 | 13.46 % | `auto_tap_for_cost_inner` |
| 352,489,276 |  9.81 % | `dispatch_triggers_for_events` |
| 317,061,930 |  8.83 % | `simulate_through_combat` |
| 315,917,638 |  8.79 % | `gather_continuous_effects_inner` |
| 226,841,402 |  6.31 % | `pick_by_outcome` (225,396,844 of it one `in_place_collect`) |
| 209,692,860 |  5.84 % | `compute_permanent` |
| 181,032,011 |  5.04 % | `cast_candidates` |
| 114,869,442 |  3.20 % | `computed_permanent` |

**Who still takes a whole-board pass**: `compute_battlefield` **310 calls**
(`submit_decision`, 0.02 %), from 17,718 three passes ago. The layer cost
that is left is per-card `computed_permanent` and the gather.

The seventeenth pass's table, for the rows it costed. **Retaken 2026-08-10
on its tip `6ed3dbfc`: 3,817,208,224 Ir.** The pass's base (`8ca7df9f`)
rebuilt on a fresh container read 3,948,115,609 against the recorded
sixteenth-pass figure of 3,948,056,772 — **0.0015 % apart**, so the two
passes' numbers chain.

The tip's shape, re-taken (inclusive, overlapping). The top of the list is
still the `auto_tap -> activate_ability -> activate_ability_inner` chain,
but it has come down 14.21 -> 12.67 / 9.19 -> 7.48 / 8.51 -> 6.77 and
`would_accept` is now the largest named consumer on its own:

| Ir | share | site |
|---|---|---|
| 542,961,849 | 14.22 % | `would_accept` — the affordance probe, 5,102 calls |
| 483,549,248 | 12.67 % | `auto_tap_for_cost_inner` (8,892 calls) |
| 454,701,876 | 11.91 % | `sim_spell_action_inner` (34,620 calls; **only 3,642 reach the main-phase branch, 4,424 the stack branch, 3,718 the trick branch — the other ~22.8 k bail at 35 Ir**, so the freeze scope it enters is not the cost) |
| 325,599,331 |  8.53 % | `gather_continuous_effects_inner` |
| 322,138,354 |  8.44 % | `dispatch_triggers_for_events` (52,332 calls, ~2,867 Ir of *self* each) |
| 294,451,348 |  7.71 % | `compute_permanent_pass` |
| 285,583,611 |  7.48 % | `activate_ability` (18,340 of its 18,386 calls come from auto-tap) |
| 258,521,084 |  6.77 % | `activate_ability_inner` |
| 180,998,747 |  4.74 % | `bot::cast_candidates` |
| 144,304,650 |  3.78 % | `mana_source_table` (8,892) |
| 143,779,527 |  3.77 % | `computed_permanent` |
| 114,223,290 |  2.99 % | `check_state_based_actions` |

**The one number that reframes the file**: `pick_attacks_scored` is
**1,989,089,442 Ir / 52.11 % over 630 calls** — 3.2 M Ir per attack
decision — and `simulate_attack_outcome_once` under it is 51.92 %. *Half
the simulator is the bot re-playing a turn cycle per candidate attack.* It
calls `sim_step` 34,384 times and `sim_spell_action` 34,620 times. Nothing
in this file has ever attacked the search itself (candidate widths,
horizon, or per-iteration cost); every pass so far has made its inner loop
cheaper.

**Who still gathers**: `computed_permanent` 109,680, `compute_battlefield`
17,718, `check_state_based_actions` 10,670, `frozen_effects` 8,882 —
**146,950 total**, from 165,336 at the pass's start.

**The allocator is still the largest single theme and still untouched**:
`_int_malloc` 4.97 / `_int_free` 3.98 / `malloc` 2.93 / `free` 1.77 /
`malloc_consolidate` 0.69 / arena+merge+unlink 2.20 = **~16.5 %**, with
`Arc::clone_from_ref_in` (the CoW unshare) another **~3.5 % self**. It has
risen as a *share* only because the pass removed non-allocating work.

The fifteenth pass's table, for the record:

**4.02 G instructions for six games**, from 4.19 G at this pass's start,
4.96 G at the fourteenth's, 5.62 G at the twelfth's, and 14.40 G eight
passes ago on the same workload. **2,445,057 allocations**, from 3,748,803.

The tip's shape, in one table (inclusive, overlapping):

| Ir | share | calls | site |
|---|---|---|---|
| 3,094,150,259 | 76.89 % | | `perform_action_inner` |
| 1,391,324,294 | 34.58 % | | `perform_action` |
| 606,964,219 | 15.08 % | 5,102 | `would_accept` — the affordance probe, now the largest single named consumer that isn't the action dispatcher |
| 581,810,215 | 14.46 % | 8,892 | `auto_tap_for_cost_inner` |
| 526,847,459 | 13.09 % | 203,770 | `computed_permanent` — 128,066 of those gather, i.e. 37 % memo hits |
| 394,720,976 |  9.81 % | | `dispatch_triggers_for_events` — was 11.72 % before the strip gate |
| 361,673,091 |  8.99 % | **165,336** | `gather_continuous_effects_inner` — 243,190 gathers two passes ago |
| 307,176,076 |  7.63 % | | `compute_permanent_pass` |
| 286,537,425 |  7.12 % | 10,670 | `check_state_based_actions` (candidate 0.25) |
| 267,242,117 |  6.64 % | 17,718 | `compute_battlefield` |
| 191,166,118 |  4.75 % | 7,024 | `bot::cast_candidates` — climbing as a share; still never read at line level |
| 144,255,177 |  3.58 % | 8,892 | `mana_source_table` |
| 82,717,638 |  2.06 % | | `GameState::clone` |

**Who still gathers**, the number that has driven four passes:
`computed_permanent` 128,066, `compute_battlefield` 17,718,
`check_state_based_actions` 10,670, `frozen_effects` 8,882 — **165,336
total**, down from 243,190.

Self cost, grouped (allocator block re-measured on the tip: `_int_malloc`
4.74 / `_int_free` 3.89 / `malloc` 2.87 / `free` 1.74 / `malloc_consolidate`
0.66 / arena+merge+unlink 2.22 = **~16.1 %**, from 19.3 % — the first pass
where it fell as a share, because the strip gate removed 78 k gathers and
each gather allocated):

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

**Where the `collect()`s are**, by inlining site. **These are inclusive
rows and the "a `Vec` materialized and thrown away" gloss they were written
with is wrong** — see candidate (1) above, where the `Vec` machinery at the
four largest of them is measured at 0.10 % between them. The cost is the
iterator body, so read each row as "this much work is driven by one
`collect`", not "this much allocation":
`compute_battlefield` 224 M / 4.71 % over 17,718 calls (12,641 Ir each);
`bot::cast_candidates` 169 M / 3.55 % over 7,024 calls (**24,040 Ir each**);
`mana_source_table` 146 M / 3.07 % over 8,892; `check_state_based_actions`
140 M / 2.95 % over **82,634 collects, i.e. 7.7 per sweep**;
`fire_step_triggers` 63 M / 1.32 %.

## Perf candidates

Ordered by expected value. Each run pulls the top one, attaches numbers,
and feeds what it finds back in. Re-profile and replenish when the list
goes thin or stale.

**The `compute_battlefield` table is closed.** Every site the nineteenth
and twentieth passes ranked is paid: `declare_attackers_banded`
(`31116d43`), `declare_blockers` x2 (`42f59829`, `911cf298`),
`resolve_combat` + `resolve_first_strike_damage` (`4f3e86c0`),
`do_phasing` / `do_untap` / `process_cumulative_upkeep` (`ed4c152c`).
**Calls 17,718 -> 5,488 -> 310**, and the 310 are all `submit_decision`
(741,960 Ir / 0.02 %). Do not re-open this table; the layer system's
remaining cost is per-card `computed_permanent` and the gather itself.

0. **`pick_attacks_scored`, 1,602,691,451 / 50.06 % over 630 calls.** The
   largest single item in the file for five passes and still untouched; the
   search itself, not its inner loop, which every pass since the
   seventeenth has made cheaper. **This is a bot-quality question as much
   as a perf one** — a narrower search is a different player, so it needs a
   `bot_ladder` win-rate gate, not an Ir number. Cheapest first probe: how
   often does the search depart from greedy? If rarely, the candidates that
   never win are pure cost.
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
3. **The `RawTable::clone` block, half paid.** `--tree=caller` on
   `RawTable::clone` has exactly two callers, and `271c7d14` took the
   larger one's set half. Left: the **seven `ColdState` `HashMap` fields**
   (same CoW unshare, ~44.8 k times for six games) and
   **`GameState::clone` 302,418 table clones / 14,182,866 Ir / 0.41 %**
   over its nine non-cold hash fields (`block_map`,
   `combat_damage_order`, `combat_damage_assignment`, `died_card_snapshots`,
   `leaves_bf_lki`, `names_this_resolution`, the two
   per-player-discard maps, `players_sacrificed_this_resolution`).
   `IdMap<K, V>` already exists for the `#[serde(skip)]` ones. **The gate
   on the rest is serde**: a `HashMap` serializes as a JSON object and a
   `Vec` newtype as an array of pairs, so any field that reaches a
   snapshot needs a custom impl or a format bump. Check the field's serde
   attribute before costing it.
4. **PAID (`08cbc9c3`, -1.051 %).** `fire_combat_damage_triggers` takes
   `kinds: &[EventKind]`; the five kind-independent walks run once and
   bucket by kind, the graveyard walk stays per-kind for its dedupe set.
   Calls 28,564 -> 7,118. Nothing left here on this profile.
4b. **PAID (`006d5966`, -1.885 %) — and the shape it opened is the one to
   sweep next.** `fire_step_triggers` cloned every printed
   `TriggeredAbility` on every permanent and then filtered on
   `t.event.kind`. **The general filter: an iterator that clones and then
   narrows.** The syntactic sweep is clean workspace-wide; the semantic
   question — "how much of what this loop builds does it keep?" — is not
   swept and wants a pass of its own over the hot trigger/candidate
   builders (`push_ordered_trigger_candidates`,
   `statics_granted_triggers_with`, `cast_candidates`' `flat_map`).
5. **`pick_by_outcome`, 210,019,714 / 6.56 %, essentially all of it one
   `collect()` in `bot.rs`.** Never profiled at line level. Read
   `--auto=yes` on it before guessing; the eighteenth pass's candidate (1)
   is the warning that the cost is the *iterator body*, not the container.
   Here it is `evaluate_action_outcome` per finalist — a clone and a
   resolution each — so the container is certainly not the cost, and the
   real question is the *finalist count*, which is a bot-quality question
   like (0) and (1).
6. **`dispatch_triggers_for_events`, 240,615,249 / 7.52 % over 52,332
   calls.** `c7bdd850` took the four cheapest blocks, `b925063c` the
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
   * `can_afford_in_state` **56,500,015 / 1.76 % over 12,114 calls
     (4,664 Ir each)** — the hand-walk filter, and the largest named item.
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

**The profile of record was retaken at `125557eb`** (the twenty-fourth
pass's tip) and every share quoted in the candidates above is from that
retake, except where a candidate says otherwise. **It is now two rows
stale**: `56f6623f` and `5d4b5402` moved the damage and targeting paths by
-1.33 % between them, so shares in (0), (1), (5) and (8) read very slightly
low. Retake before pulling anything whose margin is under ~0.3 %.

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
- ~~**(0.2) `permanents_with_abilities_removed` runs a full gather**~~ —
  **done.** `ability_strip_in_scope` is the over-approximation this item
  asked for, with the six routes to `RemoveAllAbilities` enumerated in its
  doc and a `debug_assert!` cross-check against the gather. On the
  eighteenth pass's profile the call is **18,905,488 Ir / 0.50 % over
  52,332 dispatches, 361 Ir each** — that is the presence walk itself, not
  a gather.

**Re-taken in full on the eighteenth pass's base** (`26b5d2c7`,
3,812,623,112 Ir). The shares below are from that profile and supersede
every share quoted lower down. Two things it says that the seventeenth
pass's table did not:

- **(0) `resolve_combat` is 14.27 % and has never been on this list.** It
  was invisible before because its cost is spread over callees the earlier
  tables named individually (`scale_damage_to` 1.84 %,
  `apply_prevention_shields`, `damage_prevented_by_protection`,
  `fire_combat_damage_triggers`). **Read the CR 510.2 warning in item 0(f)
  before touching the apply loop** — a freeze scope across it silently
  switches `scale_damage_to` and `damage_prevented_by_protection` from live
  to pre-batch values, which is a rules change wearing an optimization's
  clothes. The safe subset is still a scope around *one* assignment's
  pre-application checks, and phase 1 (`gather_combat_damage_decisions`),
  which runs before any damage is dealt.
- **(1) Four `collect()` sites are 17.25 % between them** — `compute_
  battlefield` **5.87 %** (17,718 calls), `cast_candidates` **4.43 %**,
  `mana_source_table` **3.49 %**, `check_state_based_actions` **3.46 %**
  (7.7 collects per sweep) — and no pass has taken any of them. **Read the
  next sentence before costing one.** Those are *inclusive* rows, and the
  earlier framing of this item ("a `Vec` materialized and thrown away")
  is wrong: the `Vec` machinery at these four sites is **0.10 % between
  them** (`raw_vec` + `vec/mod` self cost: 3.14 M / 0.23 M / 0.12 M /
  0.09 M Ir). The 17.25 % is the *iterator body* — for
  `compute_battlefield` that is `compute_permanent` per card. So the lever
  is **not** a cheaper container and not an arena; it is **how many entries
  the consumer actually reads**. A caller that wants one permanent, or
  wants to stop at the first match, should take a lazy iterator or a
  targeted lookup; swapping the collect for `SmallVec` buys 0.1 % at most.
  Check each caller's consumption pattern first — that is the measurement
  this item still needs.

The rest of the tip, inclusive and overlapping, for the record:
`pick_attacks_scored` 52.11 % (630 calls) / `simulate_attack_outcome_*`
51.84 %; `would_accept` 14.21 % (5,102); `auto_tap_for_cost_inner` 12.66 %
(8,892); `computed_permanent` 12.06 %; `sim_spell_action_inner` 11.91 %;
`dispatch_triggers_for_events` 10.35 % (52,332);
`gather_continuous_effects_inner` 8.53 %; `activate_ability` 7.51 %;
`check_state_based_actions` 7.30 %; `compute_battlefield` 6.99 %;
`activate_ability_inner` 6.77 %; `cast_candidates` 4.74 %;
`mana_source_table` 3.78 %; `pick_blocks_scored` 2.95 %. Self cost: the
allocator block **~16.4 %**, `__memcpy_avx_unaligned_erms` 2.97 %,
`Arc::clone_from_ref_in` ~3.2 %, `gather_continuous_effects_inner` ~5.2 %
across its files, `dispatch_triggers_for_events` **3.93 %** (of which
1.49 % is `raw_vec` + `vec` — allocation — and 1.53 % `slice::iter` +
`non_null` — walking).

**After the seventeenth (dead-work-gate) pass.** Four rows, **-3.316 % Ir**
(3,948,115,609 -> 3,817,208,224), the pass's whole yield from one shape:
*a hot `&self`/`&mut self` helper that computes the expensive half of its
answer before asking the cheap question that decides it.* Four instances,
found by reading the caller tree under `computed_permanent` and
`CardInstance::deref_mut` and asking "how many of these calls can possibly
matter". The filter that finds the next one: **a function whose callee list
shows N calls of an expensive helper per invocation, where N is the number
of things it *might* need rather than the number it usually does.**

What the re-profile on the tip promotes, in order:

- **(0) The attack search itself, 52.11 % of the program over 630 calls.**
  `pick_attacks_scored` plays a full turn cycle per candidate through
  `simulate_attack_outcome_once` (51.92 %, 34,384 `sim_step`s). Every pass
  in this file has made that loop's *inside* cheaper and none has touched
  the loop: candidate count is `2 + w.attack_search`, the horizon runs to
  the opponent's end of combat (one more cycle under
  `attack_race_horizon`), and `w.determinize` multiplies the whole thing.
  **This is a bot-quality question as much as a perf one** — a narrower
  search is a different player, so it needs a `bot_ladder` win-rate gate,
  not just an Ir number. Cheapest first probe: how often does the search
  actually depart from greedy? If the answer is "rarely", the candidates
  that never win are pure cost.
- **(1) `would_accept`, 14.22 % over 5,102 calls, ~106 k Ir each.** Now the
  largest named consumer. Each is a probe clone plus a full
  `perform_action_inner`, and for a `CastSpell` that means the auto-tap
  chain: this candidate and (2) are the same item from two ends. 1,522 of
  the calls come from `sim_spell_action_inner`, i.e. from inside candidate
  (0)'s search.
- **(2) The auto-tap chain, 12.67 %.** `auto_tap_for_cost_inner` (8,892) ->
  `activate_ability` (18,340) -> `_inner`. **Its own body is only
  1,112,188 Ir / 0.03 %** — costed this pass, so do not spend a run on the
  `HashMap<ManaColor, u32>` scratch or the per-pip `live` collect; they are
  under 0.05 % between them. What is left inside `_inner`, by size:
  `computed_permanent` 54,317,768 (one gather per activation, and it is the
  *first* read so it cannot be deduplicated further without a caller-side
  scope), `continue_ability_resolution_x` 30 M, `ActivatedAbility::clone`
  13.6 M (740 Ir per activation, cloned only to release the borrow — an
  `Arc<ActivatedAbility>` would take it), `resolve_extra_mana_on_land_tap`
  9.1 M over 18,296 (presence-gate shape: it walks the whole battlefield x
  static_abilities and clones `extra_mana_on_land_tap_this_turn` per land
  tap), `is_mana_ability::mana_compatible` **219,772 calls, 12 per
  activation** — the function is called from a dozen places in one pass.
- **(3) `dispatch_triggers_for_events`, 8.44 % over 52,332 calls, ~2,867 Ir
  of *self* each.** The `events.is_empty()` early-out already fires; what
  remains is the per-event `match` over the whole batch plus
  `push_ordered_trigger_candidates` (48 M), `trigger_grant_sources` (24 M)
  and `statics_granted_triggers_with` (**945,812 calls, 18 per dispatch**).
  `event_matches_spec` runs only 63,846 times — 1.2 per dispatch — so the
  cost is *setup*, not matching. Read it for a gate on "does this batch
  contain an event any permanent listens for" taken before the setup.
- **(4) `scale_damage_to` takes 14,624 whole-game gathers.** `&self`, no
  freeze scope, one `computed_permanent` per call at ~3,364 Ir. The eleventh
  pass's shape exactly, and the fix is a caller-side scope — check first
  whether the damage loop that calls it can hold one across its writes.
- **(5) The allocator, ~16.5 %, plus `Arc::clone_from_ref_in` ~3.5 % self.**
  Unchanged and still never attacked head-on. Cost with a `release` +
  mimalloc A/B, not callgrind alone.

**After the sixteenth (leaf-cost) pass.** Three rows, -1.836 % Ir
(4,021,875,017 → 3,948,056,772). Two of them were one shape — *a cheap
leaf function called a million times, and a scan whose selective conjunct
was second* — and the third was a null worth recording. What the re-profile
on the tip promotes, in order:

- **(0) The auto-tap chain, 14.21 %, is now the whole top of the list.**
  `auto_tap_for_cost_inner` (8,892 calls) → `activate_ability` (18,340) →
  `activate_ability_inner` (336 M / 8.51 %, ~18 k Ir per land tap). Inside
  `_inner`, by size: **`computed_permanent` 36,772 calls / 106 M / 2.65 %,
  i.e. exactly two whole-game gathers per activation** (candidate 4 below —
  read its warning); `continue_ability_resolution_x` 30 M;
  `flagbearer_violation` 25 M *before* this pass's reorder took its
  `same_team` walk; `grant_scan` 15.6 M (a board scan per activation);
  `ActivatedAbility::clone` 13.6 M (740 Ir per activation, cloned only to
  release the borrow — an `Arc<ActivatedAbility>` would take it);
  `resolve_extra_mana_on_land_tap` 9.1 M over 18,296 calls, which is a
  presence-gate shape. **8,892 auto-taps for six games is the other half of
  the lever**: most of them are inside `would_accept_on` probes, so
  candidate (1) and this one are the same item from two ends.
- **(1) `bot::cast_candidates` — read at line level twice now; the answer
  has moved.** Its cost is *not* the fourteen specialty blocks (this pass
  gated them all for -0.226 %; their walks and predicates are ~13 M of self
  cost between them). It is the **plain-cast `flat_map`: 219 M / 5.44 %
  over 32,124 iterations**, of which `can_afford_in_state` is 56 M / 1.41 %
  and the rest is `auto_targets_for_effect_all_slots` + `requires_target`
  walking the effect tree per hand card per mode. `requires_target` is a
  deep recursive walk of an immutable `Arc<CardDefinition>` field and is
  called from every block — **a per-definition memo is the obvious shape,
  and nothing has tried it**.
- **(2) The allocator, ~16.2 %, plus `Arc::clone_from_ref_in` ~3.4 % self.**
  Still the largest theme and still never attacked head-on. Named
  sub-targets with counts on the tip: `hashbrown RawTable::clone` 0.91 % /
  353,862 allocations (**which `HashMap` is being deep-copied per card
  clone, and would a small insertion-ordered `Vec` — the shape
  `KeywordCounters` already uses — make it free?**), `Vec::clone` 1.01 %.
  Cost these with a `release` + mimalloc A/B, not callgrind alone.
- **(3) `compute_battlefield` materializes 224 M / 5.66 % of `Vec` over
  17,718 calls** — unchanged, now the single largest `collect()` site.
- **(4) `check_state_based_actions`, 2.99 % inclusive and 82,634 collects
  (7.7 per sweep)** — `sba_board_scan` already gates the rare SBAs; what is
  left is the ~21 whole-board walks per sweep.

**After the fifteenth (presence-gate) pass.** Two rows, -3.87 % Ir
(4,185,775,886 → 4,023,920,637), both the same shape: *a `&mut self` path
paying a whole-game gather to read one bit*. What the re-profile on the tip
promotes, in order:

- **(0) `bot::cast_candidates` — 191,166,118 Ir / 4.75 % over 7,024 calls
  (27,215 each), and now the largest single named removable item.**
  **Read at line level this pass, so don't spend that again — and the answer
  is that there is no hot line.** The largest single one is
  `can_afford_in_state` in the plain-cast filter, **56,546,672 Ir / 1.41 %
  over 12,114 calls (4,668 each)**; after that nothing inside the function
  clears 0.1 % (`beneficial_aura_host` 3.4 M, `spliceable_hand_cards_on`
  0.6 M, `ward_gate_ok` 2.7 M, the three `has_*` printed predicates ~0.2 M
  each). The other ~134 M is *breadth*: fourteen candidate blocks each
  walking the hand or the battlefield and pushing `GameAction`s. So the
  lever is **not** a hoist, it is **how many candidates get built at all** —
  the blocks run unconditionally even when the seat has no mana for any of
  them. Cost a "can this block produce anything" gate per block, cheapest
  first, before touching `can_afford_in_state` (which candidate 0(a) already
  tried and reverted).
- **(1) `would_accept` — 606,964,219 Ir / 15.08 % over 5,102 calls, i.e.
  119,000 Ir per probe.** Each one is a full `perform_action_inner`. This is
  real simulation, not waste, so the lever is *how many probes*, not how
  fast one is — the same question as (0), one level up. Cost the two
  together before touching either.
- **(2) `check_state_based_actions`, 7.12 %** — unchanged in substance;
  candidate 0.25 below still describes it, and the ~21 whole-board walks per
  sweep are still the item.
- **(3) The combat-damage cluster, ~62 M / 1.5 % between four helpers**:
  `apply_prevention_shields` (4,456×), `creature_redirects_damage_to_
  controller` (4,454×), `damage_from_source_prevented_by_keyword` (4,450×)
  and `resolve_combat` (4,474×), each ~3,510 Ir/call — i.e. **all four
  gather, all four are called once per damage application, and they are
  called in sequence for the same assignment.** The safe subset is a scope
  around *one* assignment's pre-application checks, which shares three
  gathers into one; a scope around the batch is the CR 510.2 rules change
  candidate 0(f) warns about. Read that warning before starting.
- **(4) `activate_ability_inner` still takes two computed views per
  activation** (36,772 calls over 18,386 activations, 106,612,588 Ir /
  2.65 %): the `with_frozen_layers` scope at the ability lookup, and the
  CR 602.5 gate ~270 lines later. Folding the second onto the first is
  worth ~1.3 % but is **not** obviously behaviour-preserving — the {X}
  modal, the cost-pick consumption and the free-activation guard all sit
  between them, and the later gates also run for graveyard / hand / command
  sources where the earlier scope never ran. Prove the region touches no
  layer input before moving it, or leave it.

**After the fourteenth (transaction-checkpoint) pass.** Two rows and a
correction, -11.56 % Ir — the largest single pass since the `CardInstance` representation change.
Shares below that predate it are upper bounds; **the allocator fell 19.1 % →
14.5 % and `Arc::clone_from_ref_in` 17.22 % → 11.59 % inclusive without
either being touched directly**, because the checkpoint was what made the
unshares necessary. What the pass leaves at the top of the list, measured on
its tip (4,113,269,670 Ir):

- ~~**(0) `permanents_with_abilities_removed` — one full gather per trigger
  dispatch to answer one bit**~~ — **done, -2.54 %.** See the Log row. The
  soundness device that made it landable, and the one to reuse for the next
  presence gate: the gate over-approximates and is only authoritative on
  `false`; the six emitting blocks `debug_assert!` against the same two named
  predicates (`card_can_strip_abilities` / `static_effect_strips_abilities`),
  which recurse through the `While*` gate wrappers because
  `static_effect_to_effects` does; and a `debug_assert!` in the gated function
  re-runs the gather whenever the gate says `false`, so **every one of the
  18 k suite tests audits the gate against the thing it approximates on a
  real board** rather than against a re-derived list. Residual gate cost is
  ~400 Ir/call over 59,378 calls (~0.57 %); a cached flag on `GameState`
  would take that but has to be invalidated on every battlefield write —
  not worth it at that size.
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
   the snapshot is unconditional.
   **(a)** ~~the bot's simulation loops call `perform_action` and, on `Err`,
   retry `PassPriority` and bail~~ — **done, -11.56 % net;
   see the fourteenth pass in the Log.** The item under-costed itself by a
   factor of three: it counted the clone and the drop and missed that the
   clone *shares every CoW zone*, so the next write deep-copies one. That
   second half was 40 % of `Arc::clone_from_ref_in`. It also asked for "one
   explicit checkpoint around the calls that are allowed to fail" and the
   answer turned out to be simpler — **no checkpoint at all**, because a
   dry run throws the state away on `Err` and a `PassPriority` retry after a
   rollback fails identically on a deterministic engine.
   **Residual: 13,980 checkpoints, 111,734,105 Ir (2.72 %)** — the real
   game's own actions plus the fallible declarations inside the sims. Route
   (b) below is what is left of the item, and at 2.72 % it is no longer
   worth its risk.
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
