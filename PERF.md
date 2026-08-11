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

**Cross-check at `15ec11c1` (the twenty-fifth pass's tip), 2026-08-11 —
the anchor is NOT refreshed, and the reason is the host.** Six `--bench`
runs: 69.49 / 57.38 / 82.70 / 91.48 / 92.25 / 91.22. `host_cpu` reads
**2.80 GHz** and `host_calib_ms` 54 / 252 / 51 / 62 / 54 / 54 — i.e. this
is the *older* box, not the 2.10 GHz one the committed anchor was taken
on, so **these absolutes belong to the 66.65-67.95 block below, not to the
95.64 above.** Runs 1-3 are container warm-up and their calibration says
so (252 ms on run 2 is four times the rest); the settled three read 91.48 /
92.25 / 91.22, **mean 91.65 against that host's 67.31**, over a span worth
about -14.5 % by instruction count. Wall-clock running ahead of Ir there is
the expected direction for gather-removal rows but the ratio is not
claimable from four passes of drift on one box, so nothing is claimed from
it.

**What the workload facts say, which is the part that matters:**
`turns_per_game` **26.98** on all six (a seventh consecutive anchor),
`decisions` **193,232 byte-identical run to run**, `decisions_per_game`
603.9, `stalls` 0 with the new `stalls_by` reading `cap 0 / stuck 0 /
draw 0`, `determinism ok` on all six, `peak_rss_mib` 21.5-21.7 against the
anchor's 21.7-22.3. ~~**The `--decks fixed` bench is exactly reproducible
and the wider pools are not**~~ — **fixed 2026-08-11 (`841dd40b`)**. Every
pool now reproduces on a fixed seed: `--bench --threads 1 --games 300
--seed 11` reads decisions **1,130,728** (cube, 3 runs), **2,548,986**
(all, 2 runs), **684,268** (sos, 2 runs), `determinism ok` on all seven,
against cube's 1,129,690 / 1,130,785 / 1,130,706 and two FAILs before.
`--decks all`'s stall rate is a stable **0.12 %** (6 draws / 5,100 games,
`cap 0 / stuck 0`) where it used to move run to run. No number in this file
moved — all of them are `--decks fixed` or the six-game callgrind workload
— but the wider pools are now usable as measurements.

**Final checks at `ac8e3b50` (this run's tip), 2026-08-11 — `release-fast`,
so the anchor is NOT refreshed and these absolutes do not chain to it.**
Three `--bench` runs: 72.97 / 77.12 / 75.68 games/s, `host_calib_ms`
51 / 66 / 52, i.e. the 2.80 GHz box. The three `release-fast` runs taken on
the same box earlier in the sitting, before the hasher row, read 71.37 /
68.37 (calib 58 / 61) plus two contended ones — directionally up and
consistent with -0.942 % Ir, but a 4 % wall-clock delta on this box is not
claimable and nothing is claimed from it. **The workload facts are the
check that matters**: `decisions` **193,232** byte-identical on all three
(an eighth consecutive anchor's worth), `turns_per_game` **26.98**,
`stalls` 0, `determinism ok`, `peak_rss_mib` 23.9-24.6 against the
pre-change `release-fast` runs' 24.0-24.1. Cube twice more at 1,130,728.

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

Passes twenty to twenty-five, compacted to an index — same treatment as one
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

**Twenty-sixth pass — a determinism fix that pays.** Base `8095ff60` (the
twenty-fifth pass's tip) rebuilt here read **3,162,657,064 Ir** against the
recorded 3,159,019,265 — **0.115 % apart**, the widest base-rebuild gap this
file has recorded and still under the 0.1-0.2 % a single pair resolves; it
is a different container, and the row below is nine times it.

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-11 | The engine's hash containers get a fixed hasher (`841dd40b`) | 3,162,657,064 Ir | 3,132,870,988 Ir (**-0.942 %**) | Longer-lived candidate (6), "`HashMap` hash choice", pulled for a correctness reason and paying anyway. `std`'s `RandomState` is SipHash-1-3 with a per-map seed; `crate::fxhash` is rustc's seedless FxHasher — one rotate-xor-multiply per word — behind `HashMap` / `HashSet` type aliases, so the swap is a path rewrite plus `::new()` -> `::default()`. The keys are `CardId`s, seat indices and `&'static str` names, i.e. one or two words each, which is where SipHash's setup dominates. Also carries `125108c1` (coin flips off the game RNG), which cannot move this workload: `--decks fixed` has no coin-flip card, so `Decision::CoinFlip` is never constructed. Bench output identical (24 decided, 12 splits, rho -1.000), 18,618 tests green, golden traces unchanged. **The reason it was pulled is in TODO.md**: it makes `--decks cube` / `sos` / `all` reproducible on a fixed seed for the first time. |
| | **cumulative, twenty-sixth pass** | **3,162,657,064 Ir** | **3,132,870,988 Ir (-0.942 %)** | one row, callgrind on the fixed six-game workload |

**What the pass leaves behind.** *A determinism defect and a perf candidate
can be the same item.* Candidate (6) sat on the longer-lived list for a
dozen passes as a hash-cost question and was never worth pulling on its own
at ~1 %; what moved it was the cube pool's fixed-seed nondeterminism, which
made the same change mandatory. **Check the standing candidate list before
designing a robustness fix** — the cheapest correct fix may already be
costed there.

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

## Profile of record

Callgrind on `profiling-fast --no-default-features` (= `release-fast` opt
settings + debuginfo; system allocator, because valgrind replaces malloc and
a mimalloc build would measure the interception), 1 thread, `--a gang --b
gang --games 6 --seed 1 --decks fixed`.

**Re-taken 2026-08-11 on the twenty-fourth pass's tip `125557eb`:
3,201,568,157 Ir.** Supersedes the twenty-second-pass table beneath it,
which is kept only for the rows it costed that are still live.

**Now three rows stale — retake before pulling anything under ~0.5 %.**
`5d4b5402` (-0.594 %) and `841dd40b` (-0.942 %) landed after it, and the
tip measures **3,132,870,988 Ir**; every share below reads ~2 % high, and
the hasher row moves them unevenly — the map-heavy sites (`GameState::clone`,
the `ColdState` unshare, `dispatch_triggers_for_events`) fell further than
the search does. What the tip's caller tree *does* say, taken 2026-08-11:
`computed_permanent`'s gathering callers are `damage_prevented_by_protection`
2,036 Ir/call over 18,986, `scale_damage_to` 1,920 over 14,624,
`damage_from_source_prevented_by_keyword` 3,507 over 4,450, `dying_snapshot`
3,458 over 3,420, `permanent_has_keyword` **1,417** over 8,328 (down from
2,638 — that is `5d4b5402` showing up), `blocker_can_block_attacker` 286 over
15,368. The four candidate-(10) sites enumerated off it —
`ability_target_has_protection` (24 calls), `auto_target_for_effect_avoiding_set_xc`
(220), `noncombat_damage_doublers_for` (174), `can_block_any_computed_attacker`
— are all **cold on this workload** and were not taken; the family's warm
members are the damage leaves, and those are at their one-gather floor.

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
6. ~~**`HashMap` hash choice**~~ — **done, -0.942 % Ir** (`841dd40b`). The
   engine's `HashMap` / `HashSet` are `crate::fxhash`'s, on rustc's seedless
   FxHasher; see the twenty-sixth pass's Log row. Pulled as a determinism
   fix, not a perf one. What is *not* done: `hashbrown RawTable::clone` is
   still in the clone path for the seven `ColdState` maps and the nine
   `GameState` hash fields — that is candidate (3), a container question,
   and a cheaper hasher does not touch it.
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
