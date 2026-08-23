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
# A second CARGO_TARGET_DIR (gitignored: /target-probe/) lets the *next*
# candidate build while the current one runs under callgrind — callgrind is
# single-threaded and contention-immune, so the overlap is free. Two cargo
# builds at once is not: on 4 cores they take ~1.5x each.
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

**The forty-second pass is measured as a paired `release` A/B, which is what
this file has been asking for and had never been able to do.** Both sides
built `release` + mimalloc from the same tree, run alternating in one sitting
on one box, so host drift moves both. Base is `a81df3fe` (rounds 48-49, no
perf commits); tip is `b1a95b22` (the pass's seventh commit — the eighth,
`1032979c`, is a further -0.496 % Ir this reading does not include):

```text
                     base (a81df3fe)              tip (b1a95b22)
games_per_s          153.80 / 154.33 / 157.09     160.24 / 164.71 / 166.11
mean                 155.07                       163.69      -> +5.56 %
host_calib_ms        46 / 46 / 44                 46 / 45 / 45
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz (both)
decisions            196,220                      196,220     byte-identical
turns_per_game       27.53                        27.53
stalls               0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism          ok (all pairs split, every run)
peak_rss_mib         27.5 - 29.8 (tip)
```

**+5.56 % wall-clock against -5.48 % Ir, and the agreement is the point.**
The same A/B on `profiling-fast --no-default-features` (system allocator)
reads **132.84 -> 140.64, +5.87 %**. Three independent measurements of one
pass landing within half a point of each other is what a *non*-allocation-
shaped pass looks like: the fortieth's allocation rows read -11 % Ir and
+39 % wall-clock because Ir counts an allocation's instructions and not the
stalls behind them, and this file predicted the divergence. This pass's two
big rows are a **deep copy** and a **`memcpy`**, which Ir counts honestly.

**The bench invariants moved before this pass, not in it.** `decisions`
196,220 and `turns_per_game` 27.53 against the block below's 193,232 / 26.98:
the pass's own base binary reads the new values, so rounds 43-47's adopted
search changes (chump blocks, target arms, hostile targeting) own the drift.
**196,220 / 27.53 / 0 stalls is the invariant set to compare against from
here.**

**Checked at the true tip (`1032979c`), on the `profiling-fast` binary the
Ir rows are taken on** — the eighth commit is a measured -0.496 %, so this is
a confirmation, not a re-anchor:

```text
decisions            196,220        <- byte-identical with the base binary
turns_per_game       27.53
stalls               0 (0.00 %), cap 0 / stuck 0 / draw 0
determinism          ok (all pairs split)
peak_rss_mib         20.5
```

An unpaired `release` reading at the tip, taken after the whole session's
builds had been thrashing the box: **163.48 / 158.72 / 160.92 (mean 161.04)**,
calib 45 / 46 / 52. It is *lower* than the paired A/B's tip mean of 163.69
even though the eighth commit removes another 0.5 % of instructions — which is
the file's standing warning in one line: **an unpaired absolute on this box
cannot resolve half a percent**, and the third run's calib of 52 says why.
Quote the paired A/B.

Plus the wide pool at the tip — `--decks all --games 200 --paired`, **3,400
games over 17 decks, two processes, output byte-identical** (modulo the
wall-clock line), 1,699 pairs all split, **2 undecided (0.06 %)**, no panics.
That is inside the 0.12 % rules-draw band `TODO.md` records for `--decks
all`; `--decks fixed` still reads 0.

**On the absolute: 163.69 is not comparable to the 153.17 below and only
looks comparable to the 163.62 anchor by coincidence.** This box reads
`host_cpu` 2.10 GHz with `host_calib_ms` 44-46, where the 153.17 reading was
2.80 GHz at calib 44-46 and the 163.62 anchor was 2.10 GHz at calib 55-90 —
the probe and the CPU string disagree about which box class this is, and the
workload changed underneath both. **Quote the paired A/B, not the absolute.**

**The fortieth pass was the first `release` reading in five passes, and this
bench did resolve it.** Same `--bench`, `release` + mimalloc, three runs, on
a box with the same `host_cpu` string as the thirty-sixth and thirty-seventh
passes' readings below. Taken at the pass's **fourth** commit (`c185f313`);
the fifth is a further -0.978 % Ir that this reading does not include:

```text
games_per_s          147.22 / 153.84 / 158.44   (mean 153.17)
games_per_s_th       49.07 - 52.81
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        46 / 45 / 44               <- 48-51 at those two readings
decisions            193,232 byte-identical on all three
turns_per_game       26.98
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         27.0 - 29.0
determinism          ok (all pairs split, on all 3 runs)
```

**110.40 (pass 36's tip), 110.34 (pass 37's tip), 153.17 (pass 40's fourth
commit) — +38.8 %.** Those first two readings established a **2.2 %** spread on this
box class, so a 38.8 % gap is signal, not noise; `host_calib_ms` is 44-46
against their 48-51, so a few points of it are a faster box. **What it covers
is passes 38, 39 and 40 together** — cumulatively **-11.1 % Ir** — and it
cannot be split among them: no `release` binary was built at the
thirty-eighth or thirty-ninth tip. **A -11 % Ir reading +39 % wall-clock is
the shape this file predicts for allocation-shaped work** (see the caveat in
**How to measure**): the fortieth pass alone removed **90,382 allocations**
and **35 % of `memcpy`**, and Ir counts an allocation's instructions, not the
stalls behind them.

**The 163.62 anchor is not replaced, because it is a different box** — the
readings below are 110-153 on 2.80 GHz hosts against its 2.10 GHz / calib
55-90, and nothing in this file has ever made those comparable. Compare a
future reading to **153.17 at calib 44-46** and check the probe first.

**NOT re-anchored at the thirty-ninth pass's tip, and the host said why.**
The three commits are behaviour-preserving and sum to **-2.255 % Ir** under
callgrind — inside the bench's noise band, and this sitting's box is not the
one the anchor was taken on: `host_cpu` reads **2.10 GHz** against the
recorded tip's 2.80 GHz and `host_calib_ms` **71** against 45. A `--bench`
absolute here would measure the box. What was checked at the tip is the
**invariants**, and they are byte-identical with the anchor — `decisions`
**193,232**, `turns_per_game` 26.98, stalls 0, determinism ok. The block is
in the thirty-ninth pass's **Log** entry. The anchor stands at 163.62.

**NOT re-anchored at the thirty-eighth pass's tip either, and this time no
`release` reading was taken at all.** The pass is one behaviour-preserving
commit measured at **-2.525 % Ir** under callgrind, which is inside the
bench's noise band, and the three readings below already establish that a
`--bench` absolute on this box cannot resolve it. What was checked at the tip
is the **invariants**, on the `profiling-fast --no-default-features` binary
the callgrind rows were taken on:

```text
decisions            193,232        <- byte-identical with the anchor
turns_per_game       26.98
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
determinism          ok (all pairs split)
games_per_s          124.04         <- NOT comparable: profiling-fast, system
peak_rss_mib         21.0              allocator. Neither goes in this block.
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        45
```

Plus the wide pool at the tip — `--decks all --games 200`, **3,400 games over
17 decks, two processes, output byte-identical** (modulo the wall-clock line),
1,700 pairs all split, **0 undecided**, no panics. The anchor stands at
163.62.

**NOT re-anchored at the thirty-seventh pass's tip (`59c964dc`).** Same
`--bench`, `release` + mimalloc, three runs, on the same container and the
same `host_cpu` string as the thirty-sixth pass's reading below:

```text
games_per_s          109.79 / 109.96 / 111.27   (mean 110.34)
games_per_s_th       36.60 - 37.09
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        51 / 49 / 48
decisions            193,232 byte-identical on all three
turns_per_game       26.98
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         29.3 - 31.2
determinism          ok (all pairs split, on all 3 runs)
```

**Three readings on this box now: 110.40 (pass 36's tip), 112.73
(`7af2b489`, three commits and -1.590 % Ir later) and 110.34 (`59c964dc`,
one more commit and -0.223 % later).** The spread is 2.2 % and the Ir moved
-1.809 % monotonically down across it, so the bench cannot resolve this
pass and does not try to. **What it does check is the invariants, and they
are identical**: `decisions` is 193,232 byte-identical with the anchor and
with every reading below, `turns_per_game` and the stall counts unchanged,
determinism ok on all six runs. Nothing to investigate; the anchor stands.

**The thirty-sixth pass's reading, kept because it is the file's third and
sharpest demonstration of why the anchor does not move.** Same `--bench`,
`release` + mimalloc, three runs at `898a9912`:

```text
games_per_s          111.58 / 108.32 / 111.29   (mean 110.40)
games_per_s_th       36.11 - 37.19
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz    <- 2.10GHz at the anchor
host_calib_ms        48 / 48 / 51                            <- 55-90 at the anchor
decisions            193,232 byte-identical on all three
turns_per_game       26.98
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         31.0 - 31.2
determinism          ok (all pairs split, on all 3 runs)
```

The pass's *first* commit read **112.09 / 109.68 / 112.60 (mean 111.46)**
on the same box an hour earlier. The second commit is **-1.640 % in Ir** and
the two bench means are **0.95 % apart** — a textbook instance of the rule
above it: a sub-5 % change does not clear this bench's noise, whichever
direction it goes. Callgrind is the arbiter; the bench's job here is the
invariants, and they are identical across both readings.

**That is -32 % against the 163.62 anchor, on a change that callgrind
measures at -0.879 %.** It is the box, and this time the box says so out
loud: `host_cpu` reports a *different processor string* and `host_calib_ms`
reads **faster** (48-52 against 55-90) while three-thread throughput is a
third lower — the same single-threaded-probe blind spot recorded at the
anchor, now with the host difference visible in the CPU string too. Core
count is 4 on both.

**Every invariant is identical, and one of them is decisive**: `decisions`
is **193,232 byte-identical**, the anchor's exact figure, so the two
readings performed the same work. `games_per_s_th` fell uniformly by ~30 %,
which is a per-core-speed change, not something a code change that *removes*
20.6 M instructions can do. The callgrind base for this pass was built and
run on **this** container and reproduced the recorded `bdc11c86` figure to
within 3,123 Ir, so the -0.879 % is attributed on one box in one sitting.

Nothing to investigate; the anchor below stands. **Do not replace it with
111.46** — a baseline is only refreshed alongside an intentional, explained
change to what the program does, and this is neither.

**Re-anchored 2026-08-15 at `bdc11c86`** (`release`, mimalloc — the shipped
configuration), the thirty-fifth pass's tip.

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default)
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz
host_calib_ms        76 / 56 / 90 / 55
games                320
games_per_s          165.87 / 168.93 / 164.04 / 155.63
                     (mean 163.62, spread 8.5 %)
games_per_s_th       51.88 - 56.31
turns_per_game       26.98
decisions            193,232 byte-identical on all four
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         29.2 - 31.5
determinism          ok (all 160 pairs split, on all 4 runs)
```

**The same box read `169.85` (calib 53-66) at `8ff6daab` ninety minutes
earlier**, so this block is 3.7 % *under* the parent commit while the tip is
0.855 % *better* in Ir. Nothing to investigate: the drop is inside the
bench's noise band, calib moved the same way (55-90 against 53-66, and the
8.5 % spread against 5.1 % says the host was busier), and every invariant is
identical. Kept as two readings rather than one because the pair is the
cleanest demonstration in this file of why a `--bench` absolute is not an
attribution — **two sittings, one binary-identical workload, same
container, 3.7 % apart.**

**136.75 -> 163.62 is +20 %, and *none of it is the change*. Read this
before quoting either number.** The pass's change is **-2.177 % Ir**, and an
alternated `profiling-fast` A/B/A/B/A/B in one sitting read base 140.56 /
134.46 / 140.07 against new 139.55 / 138.93 / 136.06 — **-0.13 %,
distributions fully overlapping**, exactly what a 1-2 % change looks like
against this bench's ~5 % noise floor. The gap is the *box*.

**And that is a finding about the probe, not just about the box:
`host_calib_ms` did not detect it.** It read 53-66 against 47-57 at
the previous anchor — if anything *slower* — while three-thread throughput
is a quarter higher. The probe is a **single-threaded** ALU + 4 MiB
random-access loop, so it measures one core's speed and nothing about how
the host schedules three workers; two containers reporting the same
`host_cpu` string and the same calib can still differ by ~20 % on the
actual workload. The cross-check that *did* work is a configuration comparison:
`release-fast` + **system** allocator read ~138 games/s on this box, i.e.
*above* the committed `release` + **mimalloc** baseline of 136.75, which is
impossible on one host — release+mimalloc is strictly the faster
configuration. **Treat a `--bench` absolute as comparable only within one
sitting on one container**, calib agreement included, and use the
Log-row A/B or callgrind for anything else. A cheap improvement if this
comes up again: have the calib probe run one pass on `--threads N` as well
as single-threaded.

**The superseded `6cc0bdc3` block**, kept because the Log rows chain to it
and for its invariants:

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default)
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz
host_calib_ms        49 / 57 / 48 / 47 / 52 / 49
games                320
games_per_s          138.65 / 137.80 / 133.02 / 139.40 / 133.43 / 138.18
                     (mean 136.75, spread 4.8 %)
games_per_s_th       44.34 - 46.47
decisions_per_s      80,325 - 84,175
turns_per_game       26.98
decisions            193,232 byte-identical on all six
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         29.1 - 30.8
determinism          ok (all 160 pairs split, on all 6 runs)
```

**Do not read 130.71 -> 136.75 as +4.6 %: the box is a different one.**
`host_cpu` is **2.10 GHz** here against **2.80 GHz** at every earlier
anchor in this file, so the two blocks are not the same measurement and no
delta between them is sound. Every invariant is unchanged across the
collision merge: `decisions` **193,232**, `turns_per_game` 26.98, stalls 0,
`peak_rss_mib` 29.1-30.8, determinism ok on all six runs.

**The superseded `35fdfce3` block**, the thirty-third pass's measurement,
kept because the Log rows chain to it:

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default)
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        71 / 45 / 44 / 53 / 48 / 51
games                320
games_per_s          134.50 / 136.10 / 127.81 / 132.63 / 121.53 / 131.71
                     (mean 130.71, spread 12.0 %)
games_per_s_th       40.51 - 45.37
decisions_per_s      77,176 - 82,186
turns_per_game       26.98
decisions            193,232 byte-identical on all six
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         29.2 - 29.4
determinism          ok (all 160 pairs split, on all 6 runs)
```

**120.10 -> 125.16 -> 130.71 is +8.8 % across passes 32 and 33, of which
+4.4 % is pass 33.** The two are -1.969 % and -5.235 % by instruction count
(-7.10 % compounded), so wall-clock roughly matching Ir here — rather than
under-delivering as passes 29-31 did — is the expected shape: pass 33
removes whole gathers and 91-field deep copies, which cost cache lines as
well as instructions. Every invariant is unchanged — `decisions`
**193,232**, `turns_per_game` 26.98, stalls 0, `peak_rss_mib` 29.2-29.4.

**This block was measured at `35fdfce3`, which is *not* the merged tip** —
the fourth collision this file records. **Settled 2026-08-14, and by the
instruction count rather than by the hash.** `35fdfce3` is not reachable on
this branch at all (`git rev-parse` fails on it), so the wall-clock block
above it cannot be chained to and the box changed CPU model underneath in
any case. But the *code* it describes is this tip's: the merged tip
measures **2,394,813,677 Ir** against the pass's recorded
**2,394,812,950**, a difference of **727 Ir in 2.4 G (0.00003 %)**. Pass
33's `-5.235 %` therefore does describe the branch, and the collision cost
this file a dangling hash, not a wrong number. **When a hash is in doubt,
callgrind identifies a build; a `--bench` mean does not.**

**The host is noisier than it was at the previous anchor** — spread 12.0 %
against 1.44 %, and `host_calib_ms` ranges 44-71 against 45-46. The three
runs whose calib matches the old block's mid-40s fingerprint read 136.10 /
127.81 / 121.53, mean 128.48, i.e. +7.0 %; the +8.8 % above is the honest
six-run mean and the +7.0 % is the conservative floor. **Neither is a
substitute for an alternated A/B in one sitting** — see the warning above.

The wide check at this tip: `--bench --threads 1 --games 300 --seed 11
--decks all` reads `decisions` **2,548,986** over 5,100 games and 17
archetypes, `turns_per_game` 20.99, **6 draws (0.12 %)**, determinism ok on
all 2,547 pairs — byte-identical to the recorded constant, which is the
behaviour check the golden traces can't make at that scale.

**The superseded `5174acd3` block**, the thirty-second pass's tip
(`release`, mimalloc), kept because it chains to `76804984` and through it
to `5034eb2f`: `host_calib_ms` reads **45-46** against 45-46 and 44-48, the
same mid-40s fingerprint, same `host_cpu`.

```text
bot_ladder --bench   release, rustc 1.95.0, 4-core VM, 3 worker threads
                     mimalloc (the default); measured on an idle box
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        45 / 45 / 46 / 45
games                320
games_per_s          123.19 / 127.26 / 125.56 / 124.61 (mean 125.16,
                     spread 3.25 %)
games_per_s_th       41.06 - 42.42
decisions_per_s      74,388 - 76,846
turns_per_game       26.98
decisions            193,232 byte-identical on all four
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
peak_rss_mib         29.0 - 29.3
determinism          ok (all 160 pairs split, on all 4 runs)
```

**120.10 -> 125.16 is +4.2 %, against -1.969 % by instruction count.** The
sign is the usual one for a representation change and the size is the
unusual one: the thirty-second pass removed twenty-two `Vec` clones per
`CardData` copy, most of them empty (cheap in Ir, a branch and a store) and
some of them allocating (expensive in wall-clock, invisible in Ir under the
system allocator that callgrind forces). Ir under-counts this pass for the
same reason it over-counted passes 29-31. Every invariant is unchanged —
`decisions` **193,232**, `turns_per_game` 26.98, stalls 0, `peak_rss_mib`
29.0-29.3.

**The wide-pool check at this tip, and it closes the stall item.**
`--bench --decks all --games 300 --seed 11 --threads 3` reads `decisions`
**2,548,986** over 5,100 games and 17 archetypes — byte-identical to every
tip since `841dd40b` — `turns_per_game` 20.99, `peak_rss_mib` 137.8,
determinism ok on all 2,547 pairs, and **`stalls_by cap 0 / stuck 0 /
draw 6`**. TODO's open question was which of `cap` (budget too small) or
`stuck` (a genuine no-legal-move fixed point) the ~0.1 % rate is; the
answer is **neither — all six are rules draws**, so there is nothing to fix
and the entry closes.

**The superseded `76804984` block.** Re-anchored 2026-08-14 (`release`,
mimalloc), the thirty-first pass's tip.

```text
host_calib_ms        45 / 45 / 46 / 46
games_per_s          119.49 / 121.21 / 119.62 / 120.06 (mean 120.10,
                     spread 1.44 %)
games_per_s_th       39.83 - 40.40
decisions_per_s      72,152 - 73,192
decisions            193,232        turns_per_game 26.98
stalls               0              peak_rss_mib   29.1 - 29.4
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

**Crash-freedom at the thirty-second pass's tip `5174acd3`.** The
`overflow` profile over `--a gang --b gang --games 400 --threads 3 --decks
all` on seeds **3 / 11 / 29 / 41**: **27,200 games, 27,192 decided, no
panic and no arithmetic overflow**, 52-66 s a seed. The 8 undecided are
all on seed 11 — the same rules draws the last two tips reported, same
count, same seed. Two new seeds were added to the set because the pass
rewrote `CardData`'s field layout, which is the one thing an
overflow-checked run can catch that the suite cannot.

**Crash-freedom at the thirty-first pass's merged tip `54f5981b`.** The
`overflow` profile (release-fast + `overflow-checks`) over `--a gang --b
gang --games 400 --threads 3 --decks all` on seeds 11/12/13: **20,400
games, 20,392 decided, no panic and no arithmetic overflow**, 50-57 s a
seed. The 8 undecided are all on seed 11 and are the same rules draws the
thirtieth pass's tip reported — the pass moved neither the count nor which
seed carries them.

**Crash-freedom at the thirtieth pass's merged tip.** The `overflow` profile (release-fast +
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

### Forty-second pass — the cold group was being deep-copied for nothing

Cumulative: **2,040,144,900 -> 1,918,782,724 Ir, -121,362,176 / -5.949 %**,
in eight commits, behaviour-preserving (suite **18,728** green over 11
binaries, all golden traces identical, clippy clean workspace-wide including
the client). **Wall-clock, paired `release` + mimalloc A/B on one box:
155.07 -> 163.69 games/s, +5.56 %** — see **Baseline**; that reading is taken
at the seventh commit and does not include the eighth. All Ir readings
`profiling-fast --no-default-features`, callgrind, `--a gang --b gang --games
6 --threads 1 --seed 1 --decks fixed`, each candidate and its base built and
run in one sitting. A parallel session landed rounds 48-49 mid-pass; the tip
was re-measured after the rebase at **1,950,802,499** against the
1,951,006,287 this pass had just read, so **203,788 Ir** of the total is
theirs, not this pass's.

| step | before -> after | what |
|---|---|---|
| A | 2,040,144,900 -> 2,031,526,907 (**-0.423 %**) | activation asks `is_mana_ability` once, not thirteen times |
| B | 2,031,526,907 -> 2,026,806,678 (**-0.232 %**) | activation locates the source permanent once, not ten times |
| C | 2,026,806,678 -> 1,988,442,143 (**-1.893 %**) | **no-op writes to the cold group are guarded** |
| D | 1,988,442,143 -> 1,951,006,287 (**-1.883 %**) | `last_cast_spell_colors` leaves the cold group and becomes a `ColorSet` |
| — | *rebase onto rounds 48-49*; re-read 1,950,802,499 | |
| E | 1,950,802,499 -> 1,940,837,886 (**-0.511 %**) | the mana-source walks stop building a `Vec` per permanent |
| F | 1,940,837,886 -> 1,935,547,942 (**-0.273 %**) | a cost's colours are a `ColorSet`, not a `Vec` |
| G | 1,935,547,942 -> 1,928,339,700 (**-0.372 %**) | the untap step stops paying for locks nobody has |
| H | 1,928,339,700 -> 1,918,782,724 (**-0.496 %**) | one walk of the effect tree answers all five colours |

**(C) and (D) are the pass, and neither was on the candidates list.**
`ColdState` is ~90 collections behind one `CowBox`, and `perform_action`
holds a checkpoint of the state, so the group is **always shared**: the first
cold write of *any* action runs `Arc::make_mut` and deep-copies the lot,
**~1,700 Ir**. Reads go through `Deref` and cost nothing. So a `clear()` on
an already-empty collection, a `retain` that keeps everything, an `iter_mut`
over an empty list, or a `mem::take` of nothing pays the full copy for no
effect — and three of those were on the hottest paths in the program:

```text
32,707,116 / 1.60 %  activate_ability_inner's `self.tapped_for_cost =
                     tap_n_picks.clone()`, 18,774x — nearly all of them a
                     land tapping for mana with no tap-N cost, i.e. an
                     empty-to-empty write
 6,018,061 / 0.29 %  revert_temporary_control's `mem::take`, 3,534x
 1,918,909 / 0.09 %  the cleanup step's last_cast_spell_colors.clear(), 1,216x
```

`GameState::deref_mut` inclusive: **56,568,188 / 2.77 % over 75 call sites ->
4,608,931 / 0.24 % over 6**. The two that are left
(`creature_deaths_this_turn.push`, `life_gain_flag_pending.push`) are real
writes.

**The device, and it generalises past this file.** `clear_cold!` /
`retain_cold!` (defined in `game/mod.rs` above the module list, so the whole
engine sees them) put the `is_empty()` read in front. The macro form is
load-bearing: an extension trait cannot work, because `self.field.method()`
fires `DerefMut` *before* the method body runs. Anything that takes `&mut`
through a CoW handle has to be guarded at the call site by a `&`-read.
`PlayerData` has the same shape and the twenty-ninth pass found the same
rule there; this is the `GameState`-cold analogue, and the sweep is 84
mechanical rewrites plus five hand-guarded `mem::take`/`iter_mut`/assignment
sites the profile named.

**(D) is the corollary the field's own doc predicted.** With the no-ops
guarded, the largest survivor was whichever *genuine* write now went first —
`finalize_cast`'s `last_cast_spell_colors`, which jumped from 30 Ir a call to
**4,118** and read 25,880,547 / 1.30 % over 6,284 writes. `ColdState`'s doc
already names the remedy ("move a field out if it turns out to be written on
most actions"), and `opponent_cast_since_your_turn` sits beside it for the
same measured reason. Retyping `Vec<Color>` -> `ColorSet` on the way out
makes the checkpoint clone the move adds a byte instead of an allocation, and
lets `finalize_cast` call `printed_color_set()` instead of `printed_colors()`,
which was building a `Vec` per cast only to store it. **Guarding a cold write
promotes the next one: re-read the `deref_mut` call-site table after every
such change.**

**(A), (B) and (G) are the same shape one level down: a question asked more
times than it has answers.** `activate_ability_inner` binds `ability` once
and then asks `is_mana_ability` — a two-pass recursive walk of the effect
tree — thirteen times about it, and asks "where is this card on the
battlefield" from fourteen sites, each a linear walk over ~18 `Arc`-boxed
permanents. `do_untap` builds two whole-battlefield `HashSet`s per untap step
for lock arms gated behind instance flags nothing in the bench pool sets.

**(G) also carries a trap worth naming: a presence gate asked from inside a
freeze scope pays the gather it exists to avoid.** `board_keyword_matching`
reads `frozen_effects()`, which gathers on the scope's *first* computed read;
outside a scope it answers off the printed/instance legs and
`keyword_grant_in_scope` without gathering at all. `do_untap` asked it from
inside `with_frozen_layers`. Ask the gate outside, enter the scope only on
`true`.

**(E) and (F) are (-10)'s rule again — ask what the `Vec` costs before asking
how often it is built.** `effective_mana_abilities_of` returns an owned
`Vec` and all three hot callers ask it once per untapped permanent from
inside a `battlefield.iter()`: 47,798 allocations for a list one element wide
on a basic land, and the engine's largest `RawVec::grow_one` caller.
`ManaCost::colors` was the fifth largest, and `finalize_cast` put its result
straight into `CastProfile` — one allocation per cast, held for the turn.
Allocations **1,293,553 -> 1,232,517**; the five allocator rows are ~13.4 %
of the tip, from ~13.9 %.

**(H) is the same question as (A) and (B), asked of a colour instead of a
card.** `mana_source_table_inner` walked each ability's whole `Effect` tree
**five times**, once per colour, to build one `ColorSet`.
`effect_produced_colors` does it once and `effect_produces_color` delegates to
it, so the pair cannot drift. **When a predicate is asked once per member of a
small fixed set, the set-valued form is the primitive and the predicate is the
derived one** — the same move as `ManaCost::color_set` in (F), and
`CardDefinition::printed_color_set` had already made it.

**Where the pass did *not* go, recorded so the next run doesn't re-derive
it.** `perform_action`'s checkpoint (`drop_in_place<GameState>` 4.19 % + clone
2.16 %) is now the largest structural cost and wants a design, not a patch —
it is candidate (-13). `card_can_grant_keyword` was priced (1,148 Ir per
activation, ~830 of it a pointer-chasing board walk) and left alone: the fix
needs a per-definition bit that `CardDefinition`'s construction cannot carry
today. Both are written up under **Perf candidates**.

### Forty-first pass — the net's forward pass was scalar (candidate 11, part 1)

**mcts-net-deep 33.0 → 18.4 s/game (−44 %), mcts-net-256 121.9 → 73.0
(−40 %)**, one change: `crabomination_nn`'s `Tensor2::matvec` (and the
attention score dots) rewritten from a single-accumulator loop to an
eight-accumulator body with runtime-dispatched AVX2+FMA. Strict f32
semantics forbid LLVM from reassociating a chained dot product into SIMD
lanes, so the old loop ran scalar on every machine; the workspace also
builds baseline x86-64 (SSE2, no FMA), so even the vectorizable parts
never saw the wide units. Splitting the accumulators licenses the
reassociation, and the `#[target_feature]` wrapper (portable body
inlined into it, wasm path untouched) gets the 8-wide FMA units.

Methodology differs from every row above, deliberately: the callgrind
bench (`--a gang --b gang`) never executes the net, so this change is
invisible to the Ir baseline — the committed bench is untouched by
construction. Numbers are serial wall clock in the r42 Part C harness
(`--a <prof> --b net --decks sealed --games 4 --seed 43 --threads 1`,
release, wall/4), against r42's same-box baselines. The split behind it,
from the new `CRAB_MCTS_TIMING=1` instrumentation (profiling-fast, 24
games): leaf eval was **39 %** of search wall at **635 µs/rollout**, of
which encode was 10 µs and the forward pass the rest; after the change
the forward reads **164 µs** (**4.3×**) and the leaf **12 %**. FP
reassociation moves the sums by ~1 ulp-scale amounts; the candle parity
tests (tolerance 1e-4) and the full `crabomination_ml` suite pass, and
golden traces are identical (heuristic bots never call the net).

What this buys beyond the ladder: the same forward runs in the training
actors' self-play, the 1-ply `net` pilot, and the client's `local_bot`
think time — every net consumer, not just MCTS. Remaining in candidate
11: the rollout sim is now **88 %** of search wall (~1 ms/rollout, ~63
engine actions at ~16 µs), which is the engine's own action loop —
further leaf work (buffer reuse, blocked matvec) is bounded at ~12 %
until the rollout side moves.

### Fortieth pass — five allocation rows, and the one that was a Vec per permanent per generator

Cumulative: **2,136,851,050 -> 1,974,770,479 Ir, -162,080,571 / -7.585 %**,
in five commits, behaviour-preserving (suite **18,645** green over 11
binaries, all five golden traces identical, clippy clean). All readings
`profiling-fast --no-default-features`, callgrind, `--a gang --b gang --games
6 --threads 1 --seed 1 --decks fixed`, built and run in one sitting. The base
read **2,136,851,050** against the **2,136,847,762** this file recorded for
`797e0b65` — **3,288 Ir apart on a 2.14 G workload**.

| step | before -> after | what |
|---|---|---|
| A | 2,136,851,050 -> 2,144,334,188 (**+0.350 %**) | **reverted** — the hand sweep's mana read hoisted *eagerly* |
| A' | 2,136,851,050 -> 2,121,883,499 (**-0.700 %**) | the same hoist behind a `OnceCell` |
| B | 2,121,883,499 -> 2,103,492,419 (**-0.867 %**) | the mana-ability list stops inlining `ActivatedAbility` per element |
| C | 2,103,492,419 -> 2,019,566,094 (**-3.990 %**) | the bot's six ability generators stop building a `Vec` per permanent |
| D | 2,019,566,094 -> 1,994,280,399 (**-1.252 %**) | the free-activation watchdog vetoes before it clones |
| E | 1,994,280,399 -> 1,974,770,479 (**-0.978 %**) | activation holds the definition's `Arc` instead of cloning the ability out of it |

**Allocations 1,416,250 -> 1,325,868, frees 1,468,589 -> 1,378,206** (at D;
E removes more);
`__memcpy_avx_unaligned_erms` **103,787,318 -> 67,253,217 (-35.2 %)** and the
five allocator rows **~19 % -> ~12.5 %**. (-10) said allocation was the
program and nothing on the list was about it; this pass is four rows of it
and none of them came off the list either.

**The device that found all four: ask what a `Vec<T>`'s `T` costs before
asking how often the `Vec` is built.** `Cow<ActivatedAbility>` is as large as
`ActivatedAbility` — an `Effect` is 448 bytes on its own — so a *borrowed*
element still costs a ~600-byte allocation and memcpy. Two lists in the
program were built that way, both per permanent, both inside a loop over the
battlefield, and neither shows up as an expensive function: the cost lands in
`memcpy`, `_int_malloc` and `_int_free`, which no `--auto=yes` sort points
back at the type.

**(C) is the row of the pass and it was not on the list.** `usable_abilities`
collected `Vec<(usize, Cow<ActivatedAbility>)>` per permanent, and **six**
bot generators walk the same battlefield every tick. An earlier pass had
already borrowed the printed half — the comment says deep-cloning it was
1.71 % — and stopped there, leaving both the inline `Cow` and the `.collect()`
in place. Yielding an iterator of `AbilityRef` instead removed **-3.990 %**:

```text
pick_removal_ping       52,257,319 -> 17,689,142
pick_removal_sacrifice  38,006,768 -> 21,531,560
pick_sacrifice_value    27,603,534 -> 10,882,678
pick_removal_destroy    24,404,080 ->  7,751,007
```

**A caller that abandons the list on its first match wants an iterator, and
`Cow` is the wrong borrow type for a big struct.** `AbilityRef::Synth` boxes
the rare synthesized side; the common one is a reference. Both readers of the
old shape are now on it.

**(A/A') Laziness is the whole of the hoist, and the eager version is a
measured loss.** `available_mana` is a whole-battlefield walk asked per card
by three hand sweeps — 51,354 calls / 30,448,111 Ir / 1.42 %. Hoisting it to
one read per sweep reads **+0.350 %** if the read is eager, because
`pick_combat_trick` runs on every tick and usually filters its hand to
nothing first: that site alone went **30,969,070 -> 44,867,733** while
`cast_candidates` fell 120,279,723 -> 115,088,463. Behind a `OnceCell` the
same code reads **-0.700 %** and the call count is **5,294**. **Before
hoisting a per-element cost to the loop head, count the loops that reach zero
elements** — this file's (-6) note predicted exactly this ("laziness is worth
real money on this path") for a different site.

**(E) The same question as (B)/(C), asked by the engine.**
`activate_ability_inner` needs `&mut self` once it knows which ability is
being activated, so the ability could not stay borrowed out of `self` and was
deep-cloned to end the borrow — **18,386 activations**, 13,508,019 Ir in
`ActivatedAbility::clone`, inside the 16.99 % `auto_tap_for_cost_inner`
subtree. `CardInstance::definition` is an `Arc<CardDefinition>`, so
`HeldAbility::Printed(Arc, index)` answers the borrow question for a
refcount. **When a clone exists to end a borrow, look for an `Arc` already
wrapping the thing being cloned** — the whole hot path here (every zone's
printed abilities) is behind one.

**(D) A predicate that deep-clones.** `ActivatedAbility::is_free` asked "is
every field but the body at its default?" by cloning `self`, overwriting the
body and comparing against a fresh default — **18,394 clones**, 13,511,227 Ir
in `ActivatedAbility::clone` alone, **25,516,897 / 1.26 %** inclusive, on a
path both CR 104.4b/732.3 watchdogs take per activation. Nine cost fields now
veto first; the exhaustive comparison still backs them, so a field added to
the struct is still covered. **A drift-proof check written as
clone-mutate-compare is worth a cheap veto in front, not a rewrite.**

**Invariants at the tip**, on the `profiling-fast --no-default-features`
binary the rows were measured on:

```text
decisions            193,232        <- byte-identical with the anchor
turns_per_game       26.98
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
determinism          ok (all pairs split)
host_cpu             Intel(R) Xeon(R) Processor @ 2.80GHz
host_calib_ms        47-48
```

### Thirty-ninth pass — two loops stop asking a board-level question per element

Cumulative: **2,186,153,036 -> 2,136,847,762 Ir, -49,305,274 / -2.255 %**, in
three commits, behaviour-preserving (suite **18,645** green over 11 binaries,
all five golden traces identical, clippy clean). All readings
`profiling-fast --no-default-features`, callgrind, `--a gang --b gang --games
6 --threads 1 --seed 1 --decks fixed`, built and run in one sitting; bench
output byte-identical at every step. The base read **2,186,153,036** against
the **2,186,150,874** this file recorded for `0aa1342e` — **2,162 Ir apart on
a 2.19 G workload**.

| step | before -> after | what |
|---|---|---|
| A | 2,186,153,036 -> 2,145,028,678 (**-1.881 %**) | the trigger dispatcher's per-card loop early-outs on a definition with no trigger |
| B | 2,145,028,678 -> 2,149,437,144 (**+0.206 %**) | **reverted** — the same gate as B', hoisted into `grant_scan` instead |
| B' | 2,145,028,678 -> 2,141,951,957 (**-0.143 %**) | the CR 305.6 land-type question moves to the three loops that reuse a scan |
| C | 2,141,951,957 -> 2,136,847,762 (**-0.238 %**) | the empty dispatch stops synthesizing |

**The profile picked all of these and the candidates list picked none of
them.** That list had been worked down to sites of 0.2-0.7 %; the two rows
taken here read **4.94 %** and **4.66 %** and had never been looked at,
because neither is an expensive *function* — both are cheap questions asked
in a loop. **A gate that is cheap per call and asked per element is a row,
and it does not show up as an expensive function.** The sort that finds them
is `--auto=yes` over the hot function and reading the **call counts** on its
callee lines rather than the Ir: `54,570x` next to a per-table helper is the
finding, not the 15.9 M beside it.

**(A) `dispatch_triggers_for_events` was the largest engine function on the
tip** — **~108 M self / 4.94 % over 52,332 dispatches**, inclusive
169,290,522 / 7.74 %. Only 33.7 M of the self cost is attributed to
`game/mod.rs`; the rest is inlined `vec/mod.rs`, `ptr/non_null.rs`,
`slice/iter/macros.rs` and `raw_vec/mod.rs`. **Read the file-attribution
split before ranking a function**: one whose self cost is mostly those four
is walking and allocating, its engine-source lines will each look tiny, and
the auto-annotation under-reports it ~3x. The per-card body ran **945,812
times** (52,332 x ~18 permanents) and reached its four-way `is_empty()`
`continue` having read `stripped_ids`, taken `triggered_abilities.len()`,
built and dropped two `Vec`s and evaluated three gate branches — **~114 Ir
per permanent** to decide that a Mountain has no triggered ability. The three
board-level gates the loop already computes are asked once as `no_grants`;
with two definition loads that is the whole question. Self falls to **~71 M /
3.31 %**.

**(B/B') The finding, the wrong home for it, and the rule that comes out.**
`mana_source_table`'s `.collect()` was **101,842,310 / 4.66 % over 7,370
calls** — the second-largest `spec_from_iter` row in the program — and the
auto-annotation puts **15,899,355 Ir over 54,570 calls** of `frozen_effects`
on one line of `effective_mana_abilities_of`: the `match self.frozen_effects()
{ Some(fx) if !fx.iter().any(rewrites_land_types) => … }` that decides whether
the card needs a layer view. A mutex, an `Arc` clone and a walk of the whole
gathered set, once per untapped permanent per table, for an answer that is a
property of the board.

Putting it on `GrantScan` at scan-build time measured **+0.206 %** and was
reverted. **`grant_scan()` is not a per-loop object**: `granted_abilities_for`
is `granted_abilities_with(id, &self.grant_scan())`, a fresh scan *per card*,
and that path runs 201,834 times per six games — so the field taxed ~27
constructions for every one it helped. **Count a shared object's
constructions, not its uses, before hoisting a cost into it** (this file's
older twin of the rule is (-2b)'s "the gate's own cost is the thing to
watch"). Filled in by the three loops that build one scan and ask it about
many cards, the same code reads **-0.143 %**.

**And the row is smaller than its line, for a reason worth keeping.**
`frozen_effects` *gathers* on the first read in a scope, and that gather is
inside the 15.9 M and does not go away — it moves to whichever read now
gathers first. What a hoist off this function removes is the mutex, the clone
and the walk on every *repeat*. **A `frozen_effects` line's Ir is not all
removable; subtract one gather per scope before pricing it.**

**(C) The empty dispatch.** `dispatch_triggers_for_events` already returned
early on an empty batch — after synthesizing the CR 700.4 death and CR 800.4
control-change events, folding them in, and walking the batch to decide
whether the graveyard events needed collapsing. The synthesis `.collect()`
alone was **13,650,937 Ir over 94,608 calls**: the chain is not free on an
empty pair, because the filter closes over `&self` and the collect still
builds and drops a `Vec`. The return moves up to just after the two
`mem::take`s — all the state the call has touched by then — and the synthesis
is skipped outright when both drained lists are empty.

**Invariants at the tip**, on the `profiling-fast --no-default-features`
binary the rows were measured on:

```text
decisions            193,232        <- byte-identical with the anchor
turns_per_game       26.98
stalls               0 (0.00 %), stalls_by cap 0 / stuck 0 / draw 0
determinism          ok (all pairs split)
games_per_s          139.35         <- NOT comparable: profiling-fast, system
peak_rss_mib         20.8              allocator, and a different host. Neither
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz    goes in Baseline.
host_calib_ms        71             <- 45 at the recorded tip: a slower box.
```

**Passes thirty-seven and thirty-eight, compacted to an index** — same
treatment as one to thirty-six. The rows' prose is in git (`git log -S` on
each hash); the lessons both passes established are carried in **Perf
candidates** under (-8), (-8b) and (-9), and the thirty-seventh's ranking
correction is quoted at the head of that section.

| pass | base -> tip | levers, and what each taught |
|---|---|---|
| 37 | 2,284,098,792 -> 2,242,782,905 Ir (**-1.809 %**) | Three sites stop gathering — and **two null results that repriced the table**. Ir-per-call at a `computed_permanent` site does *not* separate "gathers" from "one `apply_layers_one`": only 29,780 of ~90,000 calls gathered at all. **The reliable test is static — a `computed_permanent` reachable only from `&mut self` code always gathers**, because a freeze scope holds `&GameState`. Rank by that predicate first, Ir/call second. |
| 38 | 2,242,782,284 -> 2,186,150,874 Ir (**-2.525 %**) | The SBA death sweep behind a **signed** layer-7 gate (**-2.277 %**), taking SBA's gathers 10,670 -> 1,442. Two halves worth keeping: **(i)** a `compute_*` site costs gather + layer pass + collect, so read the `spec_from_iter` caller table beside the gather table before ranking one; **(ii)** a layer-7 presence gate has to be *signed* to be worth anything — "can computed toughness come out *below* instance toughness" is the question every positive anthem answers `false` to for free. The fuse device (folding the gate's walks into one pass) lost here for the second time. |

**Passes thirty-five and thirty-six, compacted to an index** — same
treatment as one to thirty-four. The rows' prose is in git; the lessons both
passes established are already carried forward in **Perf candidates** under
(-5), (-6) and (-7).

| pass | base -> tip | levers, and what each taught |
|---|---|---|
| 35 | 2,394,920,900 -> 2,342,776,898 Ir (**-2.177 %**) | **A layer read behind a board-level presence gate.** The requirement walker's layer-4 card-type answer stops gathering when nothing on the board can change a card type (**-1.333 %**), and the keyword-grant twin lands on its two cheap consumers (-0.855 %). The device is candidate (-5)'s: name every route to the modification in the gate's doc, `debug_assert!` the implication in the emitting block, and let the suite audit the enumeration without any caller paying a gather. |
| 36 | 2,342,773,775 -> 2,284,099,256 Ir (**-2.505 %**) | **Gate the site, not the read.** `scale_damage_to`'s whole body behind one presence gate read **-1.640 %** — a third as many gathers as the previous pass's row for nearly twice the Ir, because the cost was ~8 battlefield walks and an allocation per call, not the gather. Plus the land-type twin of the thirty-fifth's gate (-0.881 %). A gate with no shared choke point can audit against its own outcome (`debug_assert_eq!` that the guarded body is a no-op) instead of against an enumeration. |


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
| 31 | 2,776,363,573 -> 2,577,862,811 Ir (**-7.152 %**, two concurrent sessions merged at `54f5981b`) | **A periodic sweep writing the value the field already holds.** Nine rows, one defect: `cleanup_wear_off` (`d33552e5` / `da5b1f1c`, **-2.379 % / -2.368 %** — the same site found twice, 0.011 % apart, the closest agreement two independent implementations have produced here; `finish_cleanup` was **36,768 `clone_from_ref_in` calls at ~1,976 Ir, 2.62 %**, after: 0); the per-turn and per-combat card sweeps (`6b72c0e7`, **-3.753 %** — `resolve_combat`'s provoke reset was exactly `4,474 combats x battlefield` = 68,212 `CardData` `make_mut`s, `do_untap`'s nine sickness clears 207,272; both now guarded, 23,168 / 7,532); the combat-damage pair taking one gather, not three (`3d15878b`, -0.967 %); `do_untap`'s Winter Orb branch (`76804984` / `9ee83f5d`, -0.098 %); the zone-walk family visiting its zones in one order (`1f68d1b0` / `76d31eb8`, candidate 11a, no Ir claimed); five `#[serde(skip)]` `GameState` hash fields becoming `IdMap`/`IdSet` (`62e6dd42`, -0.211 %, candidate (3)'s non-serde half). **The recipe that found six of them in one read is in candidate (-1)** — `--tree=caller` on `Arc::clone_from_ref_in` / `make_mut`, sorted by the *inlining* function. **Two devices worth more than the rows.** *A per-caller row is an attribution, not a measurement*: measured together with the cleanup row, `do_untap`'s tree read 13.3 M -> 39.3 M **at a lower call count** and looked like a regression, because a new frame changed which file:function the inliner attributed the copies to — rebuild two changes apart before believing a `--tree`. And **the third collision, much the largest**: both sessions pulled three of the same fixes *and* both wrote the `CardCold` candidate, so adding the two cumulatives (-9.4 %) overstates the merge by the overlap, which is unknowable without building it. Three merge rules kept: a **named guarded helper beats a hoisted local** (`clear_summoning_sickness()` over a `clear_sick` binding — the guard travels with the field); a **zone gate and a per-card guard compose**, they are not alternatives; and a walker's visit order takes the **union** of what each session learned about its callers. |
| 32 | 2,577,862,290 -> 2,527,098,526 Ir (**-1.969 %**); wall-clock `release` 120.10 -> 125.16 games/s (**+4.2 %**), four runs a side on one host fingerprint | **The rare heap tail of a card.** `CardCold`, a `CowBox` group for the twenty-two rarely-written heap fields of a 148-field `CardData` (`5174acd3`) — candidate (-2), above its ~1.2 % estimate; the `CardData` deep-copy table fell ~105 M / 4.0 % -> ~44.5 M / 1.76 % (the surviving rows and the field list are in candidate (-2)). **The blocker the candidate named did not exist**: `CardInstance`'s serde is *manual*, through a `CardInstanceWire` that names each field, so the 148-field split compiled with zero call-site changes and no serde attribute moved — *read the type's actual serde impl before pricing a field move by it*. **Membership held at 22** by the tenth pass's rule (group size x unshare probability < sum of the individual clone costs), from a field-by-field read rather than "is it a `Vec`"; the one judgement call, `cast_mana_spent_by_color`, is written once per cast and still wins cold. Five `clear()`/take sites needed `is_empty()` gates or the group hands the win back — one of them, `pending_etb_counters`, was a full `CardData` deep copy per permanent entering and is candidate (-1a)'s shape. Also no-Ir: `39381511` moved `CowBox` into `crabomination_base`; `9b0e5799`/`3e795d49` fixed `audit_stubs`' carrier list (59 false positives -> **0 flagged over 21,795 cards**, two tests pinning the list). |
| 33 | 2,527,094,401 -> 2,394,812,950 Ir (**-5.235 %**) | **A view computed above the `match` that decides whether anyone wants it** — a shape, not a site. `evaluate_requirement_static` resolved the CR 613.2 layer-4 view before its `match req`, and was **40 % of every `computed_permanent` call in the program** (93,612 of 230,974); a `OnceCell` behind a `computed()` accessor took it to 15,574 (`54ab4247`, **-2.128 %**). *The search*: a `let` bound above a wide `match` whose right-hand side is a gather, a whole-board walk or a clone — see candidate (-4) for the siblings and the two that measured null. Plus `ally_trigger_extra_fires`' presence gate (`db7ef79f`, -0.528 %) and **eight unguarded `ColdState` writes on per-action paths** (`4f582d8f`, **-2.507 %**; `35fdfce3`, -0.156 % tail): `GameState::deref_mut` was 114,566,801 / 4.53 %, its three largest callers each writing the value the field already held. **The syntactic sweep that finds these**: list `ColdState`'s fields, grep `mem::take(&mut self.<f>`, `self.<f>.clear()`, `.retain(`, `.iter_mut()`, `self.<f> = ` — 257 sites, the hot ones are the ones the profile names. **Why the rows do not sum**: a guard pays in full only where the site is the call's *sole* cold write (the twenty-ninth pass's arithmetic, on the group instead of the seat). |
| 34 | 2,394,813,677 -> 2,394,920,914 Ir (**+0.0045 %**) | **A crash fix at zero Ir, and a re-profile.** The layer gather's reentrancy guard moved into `computed_permanent` (`73db9c64`) — ~0.5 Ir per call, one branch on an atomic load that was already the function's first instruction; the bug it closes is a stack overflow (TODO.md's thirteenth filter), not a slow path. `d922f8d9` re-anchored the baseline at the merged tip and found **the box had changed CPU model** (2.10 GHz against 2.80 GHz), so that block does not chain to the one before it. The profile of record was retaken here: the `computed_permanent` call-site table is new, `bot.rs` closed as a hunting ground, `eval.rs:3311` entered at 1.58 % as candidate (-5). No Ir claimed. |

**Passes 29 and 30 were compacted to the two rows above on the thirty-first
pass, and passes 32-34 on the thirty-ninth** (their measurement tables are
in `git log -- PERF.md`). Their live numbers did not go with them: the
`PlayerCold` field list and the `remove_from_hand` non-candidate are in
candidate (-1), the `Keyword` bitset arithmetic in candidate (13), the
zone-walk family in (11a), the `CardData` deep-copy table in (-2), and the
"eager view above a wide `match`" siblings in (-4).

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

## Profile of record

Callgrind on `profiling-fast --no-default-features` (= `release-fast` opt
settings + debuginfo; system allocator, because valgrind replaces malloc and
a mimalloc build would measure the interception), 1 thread, `--a gang --b
gang --games 6 --seed 1 --decks fixed`.

**The forty-second pass reads 1,918,782,724 Ir at its tip.** The table below
was taken one commit earlier at `b1a95b22` (1,928,339,700); the eighth commit
only removes `effect_produces_color`'s four redundant tree walks, so every
share here reads ~0.5 % high and the `effect_produces_color` row is gone. What the next run wants from this reading:

| row | at the 42nd tip | note |
|---|---|---|
| `advance_step` inclusive | **308,428,973 / 15.99 % over 22,892** | the largest engine subtree now. `do_untap` 37,120,019 / 1.92 % over 1,764 (**25,207 -> 21,043 Ir a call** this pass) and `do_cleanup` ~27 M are once-per-turn work with more fat in them; the rest is `resolve_combat` and `resolve_top_of_stack` |
| `auto_tap_for_cost_inner` inclusive | **277,149,670 / 14.37 % over 18,340** | was 338,843,874 / 16.99 %. `activate_ability` is 163,106,449 / 8.46 % of it (was 242,104,933 / 11.33 %), `mana_source_table` 75,281,100 / 3.90 % |
| `spec_from_iter_nested` inclusive | 391,184,484 / **20.29 %** | still long and flat. Biggest callers `cast_candidates` 103,328,571 / 5.36 %, `mana_source_table` 51,148,075 / 2.65 %, `check_state_based_actions` 35,310,072 / 1.83 % |
| allocator | `free` 136.8 M incl / `_int_free` 91.4 / `malloc` 82.7 / `_int_malloc` 51.5 M | **~13.4 %**, over **1,232,517 allocs and 1,286,325 frees** (was 1,293,553 / 1,347,361). By direct caller: `finish_grow` 215,649, `from_iter` 187,120, `clone_from_ref_in` 177,500, `computed_permanent` 103,318 |
| **the checkpoint** | `drop_in_place<GameState>` 80,831,019 / **4.19 %** + `GameState::clone` 41,569,486 / 2.16 % | `perform_action`'s rollback snapshot, ~6.4 % together and the largest single *structural* cost left. See candidate (-13) |
| `dispatch_triggers_for_events` inclusive | 142,284,414 / **7.38 %** over 70,418 | `dispatch_board_scan` 24,561,076 over 53,838 and `permanents_with_abilities_removed` 7,160,454 over 53,838 are one board walk each per dispatch |
| `gather_continuous_effects_inner` | 105,696,679 incl / 5.48 %, 35,764,348 self | gathers unchanged by this pass |
| `GameState::deref_mut` | **4,608,931 / 0.24 % over 6 sites** | was 56,568,188 / 2.77 % over 75. The cold-group tax is paid off — re-read this row after any change that guards or moves a cold write |
| `check_state_based_actions` inclusive | 98,755,493 / 5.12 % | |
| `card_can_grant_keyword` | 28,547,744 incl / 1.48 %, 15,247,954 self | candidate (-11), untouched; see the correction there *and* the new one below |
| `__memcpy_avx_unaligned_erms` | 62,379,841 / 3.23 % | top caller `finalize_cast` at 12.8 M over 126,492 — the per-turn cast logs |
| `RawVec::grow_one` | 4,123,278 self | top callers `finalize_cast` 28,878, `push_mut` 42,368, `gather_continuous_effects_inner` 16,622, `advance_step` 22,892. `effective_mana_abilities_of` and `ManaCost::colors` left the list this pass |

**The fortieth pass read 1,994,280,399 Ir at its tip.** Kept because the Log
rows chain to it:

| row | at the 40th tip | note |
|---|---|---|
| `auto_tap_for_cost_inner` inclusive | **338,843,874 / 16.99 % over 18,340** | the largest engine subtree in the program, read at D. **242,104,933 / 11.33 % of it is `activate_ability`**, 18,340 calls at ~13,200 Ir each — the taps themselves. E takes ~20 M off it; candidate (-12) has the rest |
| `spec_from_iter_nested` inclusive | 382,312,820 / **19.17 %** | was 21.94 %. No single caller is over 5 % any more; the list is long and flat |
| allocator | `_int_free` 87.4 / `malloc` 60.0 / `_int_malloc` 48.8 / `free` 38.6 / arena 15.2 M | **~12.5 %** (was ~19 %), over **1,325,868 allocs and 1,378,206 frees** (was 1,416,250 / 1,468,589) |
| `__memcpy_avx_unaligned_erms` | 67,253,217 / **3.37 %** | was 103,787,318 / 4.84 %. Callers are diffuse; the biggest is `finalize_cast` at 12.2 M over 121,206 |
| `gather_continuous_effects_inner` | 99,177,104 incl / 4.97 %, 32,767,782 self | **53,732 gathers, unchanged by this pass**: `computed_permanent` 29,780, `frozen_effects` 18,926, `compute_permanents` 3,584, SBA 1,442 |
| `mana_source_table` inclusive | 81,987,227 / 4.11 % over 7,370 | was 107,317,418 / 5.02 % |
| `cast_candidates` inclusive | 105,485,620 / 5.29 % over 7,024 | was 120,279,723 / 5.63 % |
| `card_can_grant_keyword` self | 17,750,852 / 0.89 % + 5.4 M `slice/iter` | candidate (-11), untouched — but see the correction there before taking it |
| `Arc::clone_from_ref_in` / `Arc::make_mut` | 23.1 M / 19.7 M | the CoW tax. `make_mut`'s callers are **diffuse** — the largest single site is 565 k over 18,832, and there are hundreds |
| `effect_produces_color` | 7,018,200 self | `mana_source_table_inner`'s five-colour loop, still `5 x abilities` per source |
| `GameState::clone` self | 21,236,464 / 1.06 % over 17,420 | `perform_action`'s surviving checkpoints, nearly all `sim_step`'s |

**The thirty-ninth pass read 2,136,847,762 Ir at its tip**, and its three
commits moved rows the tables below do not show, because both of the big ones
are loops rather than call sites. Kept because the Log rows chain to it:

| row | at the 39th tip | note |
|---|---|---|
| `dispatch_triggers_for_events` self | **~71 M / 3.31 %** | was ~108 M / 4.94 %; the residue is the battlefield walk itself (`slice/iter` 13.4 M) plus a ~690 Ir per-dispatch preamble |
| `__memcpy_avx_unaligned_erms` | 103,789,248 / **4.84 %** | the largest single row in the program |
| `_int_free` / `_int_malloc` / `malloc` / `free` | 92.2 / 77.0 / 65.4 / 41.1 M | **~19 % in the allocator**, over **1,416,231 allocs and 1,468,562 frees** in six games. System allocator — mimalloc ships, so read it as a work count, not a wall-clock claim |
| `spec_from_iter_nested` inclusive | **21.94 %** | the `.collect()` total. Biggest callers: `bot::cast_candidates` 108,744,110 / 4.97 % over 7,024; `actions::mana_source_table` 101,842,310 / 4.66 % over 7,370; `check_state_based_actions` 33,530,891 / 1.53 % |
| `card_can_grant_keyword` | **~31.8 M / 1.45 %** | fully inlined; the presence gates' own walk. `card_keyword_possible` reaches it 333,914 times |
| `effect_produces_color` | 9,965,844 over **233,940** | `mana_source_table_inner`'s five-colour loop, `5 x abilities` per source |
| `GameState::clone` self | 21,236,464 / 0.99 % over 17,420 | `perform_action`'s surviving checkpoints, nearly all of them `sim_step`'s |
| gathers | **53,722** | `computed_permanent` 29,780, `frozen_effects` 18,916, `compute_permanents` 3,584, SBA 1,442 |

**The thirty-eighth pass reads 2,186,150,874 Ir at its tip.** Its one commit
gates `check_state_based_actions`' death sweep, taking SBA's gathers from
10,670 to **1,442** and all gathers from 62,950 to **53,722**;
`gather_continuous_effects_inner` self is **32,765,200 / 1.49 %**. The
`computed_permanent` caller table below is unchanged by it — the sweep is not
a `computed_permanent` caller — but every *share* there reads ~2.3 % low
against this tip. One new row is worth carrying: `card_type_change_in_scope`
is now **21,776,000 / ~1.0 % over 10,670** — candidate (-8b), which this pass
tried and closed as a loss.

**The thirty-seventh pass read 2,242,782,905 Ir at its tip (`59c964dc`).**
The tables below were taken one commit earlier at `7af2b489`
(2,247,783,661); the fourth commit only shrinks the per-gather layer pass at
`combat.rs:3284`, so the gather counts are the tip's and the
`compute_battlefield` row is now 310 `compute_permanents` calls instead.

| Ir | share | calls | caller |
|---|---|---|---|
| 55,179,840 | 2.45 % | 29,780 | `computed_permanent` |
| 33,727,702 | 1.50 % | 18,916 | `frozen_effects` |
| 18,704,346 | 0.83 % | 10,670 | `check_state_based_actions` |
| 6,590,525 | 0.29 % | 3,274 | `compute_permanents` |
| 622,422 | 0.03 % | 310 | `compute_battlefield` |

**62,950 gathers**, from 73,434 at `898a9912`, 97,212 at `bdc11c86` and
107,084 at `8ff6daab`. `gather_continuous_effects_inner` self is
**37,904,168 / 1.69 %**. `activate_ability_inner`, `scale_damage_to`,
`dying_snapshot` and `has_first_strikers` do not appear as
`computed_permanent` callers at any threshold any more.

**And `computed_permanent` by call site at the tip. Read this table with
the thirty-seventh pass's correction in hand — Ir/call does not say whether
a site gathers, and two of these rows have been *measured* not to.** The
static predicate does: a site reachable only from `&mut self` code gathers,
one inside `next_action`'s scope does not.

| Ir | share | calls | Ir/call | site |
|---|---|---|---|---|
| 19,712,777 | 0.88 % | 19,742 | 998 | `bot.rs:2525` `permanent_value` — scoped |
| 15,918,359 | 0.71 % | 3,274 | 4,862 | `combat.rs:2573` `combat_damage_computed` — **gathers**, candidate (-9) |
| 13,511,799 | 0.60 % | 16,688 | 809 | `bot.rs:5606` `pick_removal_sacrifice` — scoped |
| 11,644,145 | 0.52 % | 8,328 | 1,398 | `mod.rs:20823` `permanent_has_keyword` — scoped, **measured null** |
| 9,170,362 | 0.41 % | 12,010 | 763 | `bot.rs:1839` `attacker_damage_value` — scoped |
| 8,904,436 | 0.40 % | 4,084 | 2,180 | `bot.rs:6246` attacker filter — scoped, **measured null** |
| 7,956,706 | 0.35 % | 4,462 | 1,783 | `combat.rs:2334` `declare_blockers`' computed closure |
| 7,241,323 | 0.32 % | 2,226 | 3,253 | `combat.rs:4036` `creature_redirects_damage_to_controller` |
| 6,404,107 | 0.28 % | 2,598 | 2,465 | `combat.rs:1528` `declare_blockers`' `compute_permanents` |
| 5,251,116 | 0.23 % | 310 | 16,939 | `combat.rs:3284` `apply_combat_decision_answer` — **PAID `59c964dc`** |
| 4,724,245 | 0.21 % | 1,614 | 2,927 | `actions.rs:8410` a keyword read on a `&mut self` path |
| 4,606,395 | 0.20 % | 3,680 | 1,251 | `bot.rs:6356` — scoped |
| 4,491,342 | 0.20 % | 1,916 | 2,344 | `targeting.rs:141` the Ward scan |

**The older reading, kept because the Log rows chain to it.** Re-taken
2026-08-15 at `8ff6daab`: **2,362,985,109 Ir**, the thirty-fifth
pass's *first* commit; that pass's tip (`bdc11c86`) is **2,342,776,898**,
and the tables below are the `8ff6daab` read with the second commit's two
rows patched in. Supersedes the `d922f8d9` figure (2,394,920,914) and, for
totals, the `645b978d` table further below; read a share there as ~16 %
high. The `d922f8d9` tip re-measured on this box at **2,394,920,900**, i.e.
14 Ir of run-to-run drift on a 2.4 G workload — the profile is that
reproducible, and a delta of even 0.01 % is signal.

**The gather, denominated at `8ff6daab`.** `gather_continuous_effects_inner`
is **204,357,700 / 8.65 % over 107,084 gathers** (1,908 Ir each) there; the
tip's second commit removes ~22,600 of those gathers, by
caller:

| Ir | share | calls | caller |
|---|---|---|---|
| 143,448,843 | 6.07 % | 73,914 | `computed_permanent` |
| 33,723,701 | 1.43 % | 18,916 | `frozen_effects` |
| 18,702,077 | 0.79 % | 10,670 | `check_state_based_actions` |
| 6,531,360 | 0.28 % | 3,274 | `compute_permanents` |
| 622,422 | 0.03 % | 310 | `compute_battlefield` |

**And `computed_permanent` by *call site*, which is the list to work from**
(`--tree=caller`; a site over ~1,900 Ir a call is gathering, one near
800-1,300 is one `apply_layers_one` inside somebody's scope, i.e. the
floor):

| Ir | share | calls | Ir/call | site |
|---|---|---|---|---|
| 48,957,891 | 2.07 % | 18,386 | 2,663 | `activate_ability_inner`'s `let cp` — candidate (-3) |
| 25,966,901 | 1.10 % | 14,624 | 1,776 | `scale_damage_to`'s `source_cp` |
| ~20,700,000 | 0.88 % | 6,682 | 3,104 | `resolve_combat` |
| ~19,600,000 | 0.83 % | 19,742 | 996 | `permanent_value` — floor |
| ~13,500,000 | 0.57 % | 16,688 | 810 | `pick_removal_sacrifice` — floor |
| ~12,200,000 | 0.52 % | 7,060 | 1,729 | `has_first_strikers` — already scoped |
| 11,635,606 | 0.49 % | 8,328 | 1,397 | `permanent_has_keyword` |
| ~10,700,000 | 0.45 % | 3,420 | 3,138 | `dying_snapshot` |
| ~9,170,000 | 0.39 % | 12,010 | 764 | `attacker_damage_value` — floor |
| 1,007,867 | 0.04 % | 790 | 1,276 | `damage_prevented_by_protection` — **was 28,921,126 over 18,986** |

**Two rows are gone and one is a tenth of what it was.** `eval.rs`'s lazy
layer view (37,727,865 / 1.58 % over 15,574 gathers at `d922f8d9`) and
`apply_prevention_shields`' Absorb read (14,230,429 / 4,456) do not appear
at all; `damage_prevented_by_protection` fell 96 %. All three are the
thirty-fifth pass's two presence gates. Rows marked `~` were not re-read
after the second commit — it only touched the gated paths, so they are the
first commit's figures and are within a few thousand Ir.

**What the gates leave**: `activate_ability_inner` is now the whole top of
the table and is short one printed-static predicate (land types, six
variants) plus a lazy-`bf_cp` restructure — see (-3). `scale_damage_to`
(1.10 %) and `resolve_combat` (0.88 %) are the next unread sites; neither
has been costed for a gate.

**`bot.rs` is finished as a candidate-(10) hunting ground, and here is why
so nobody re-enumerates it.** `HeuristicBot::next_action` wraps the whole
decision in `with_frozen_layers` — it reads **88.27 % inclusive** — so every
`pick_*` helper is already inside a scope and its `computed_permanent` calls
cost one `apply_layers_one` per *card*, not a gather. That is what the
764-1,270 Ir rows above are: `pick_crew_vehicle` reads **107** Ir a call
over 11,364 calls, `pick_removal_destroy` 431 over 9,360. The gathers that
remain in bot frames are the ones inside **probe clones** —
`LayerFreeze::clone` resets to unfrozen by design, so `would_accept` and
`simulate_attack_outcome_once` re-gather, and they must.

The older tables below were taken at `645b978d` and before. The pass's third row (`dbd3efeb`) took another 0.473 %
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

**Ranking rule, corrected by the thirty-seventh pass's two null results:** a
`computed_permanent` site is worth a gate iff it is reachable only from
`&mut self` code, because a freeze scope holds `&GameState` and no `&mut self`
call can happen inside one. Ir/call does not answer this — `apply_layers_one`
alone spans ~760 to ~2,200 Ir — and two candidates picked on Ir/call alone
measured -0.019 % and -0.015 %. Check the caller's `self` binding first.

**Ranking rule added by the forty-second pass, and it is the one that paid:**
before ranking a *function*, ask what an ordinary action pays that it cannot
possibly need. Three of that pass's seven rows were a question asked more
times than it has answers (thirteen `is_mana_ability` walks, fourteen
battlefield walks, two whole-board `HashSet`s per untap step) and two were a
write that changed nothing. None of the five was on this list, and none shows
up as an expensive function — they land in `make_mut`, `memcpy` and
`_int_malloc`.

**(-14) A CoW handle's `DerefMut` is a deep copy, and a no-op write pays it in
full — the rule, now that the sweep is done.** `ColdState` (~90 collections
behind one `CowBox`) and `PlayerData` are both written through a handle that
is *always* shared, because `perform_action` holds a checkpoint. `clear_cold!`
/ `retain_cold!` in `game/mod.rs` are the guarded forms; `mem::take`,
`iter_mut` and whole-field assignment have to be guarded by hand. **An
extension trait cannot do this** — `self.field.method()` fires `DerefMut`
before the method body runs, so the read must be at the call site.
`GameState::deref_mut` is down to **4,608,931 / 0.24 % over 6 sites** and the
survivors are real writes; the standing work is (i) re-read that table after
*any* change that guards or moves a cold write, because guarding one promotes
the next, and (ii) the two survivors —
`creature_deaths_this_turn.push` (2.3 M over 3,104) and
`life_gain_flag_pending.push` (1.4 M over 1,294) — are move-out candidates
like (D)'s: `life_gain_flag_pending` is `#[serde(skip)]` and empty between
batches, so moving it out costs an empty-`Vec` clone (free) and buys ~1.4 M.
`creature_deaths_this_turn` holds `CardInstance`s through the turn, so its
move-out would trade ~2.3 M for a `Vec` clone per checkpoint — probably a
wash, measure before taking it.

**(-13) `perform_action`'s checkpoint — `drop_in_place<GameState>` 80,831,019
/ 4.19 % plus `GameState::clone` 41,569,486 / 2.16 %, ~6.4 % together, and the
largest *structural* cost left.** Every non-trivial action clones the state so
a rejection can be rolled back; `pass_priority_is_trivial` already skips 41 %
of them. What is actually paid: `GameState::clone` is ~20 allocations (one per
non-empty `Vec`/`HashMap` field) and the drop gives them all back — call it
**~360 k of the tip's 1.23 M allocations**. Three shapes worth costing, in
order of how contained they are:
* **Reuse the checkpoint's buffers.** A per-thread scratch `GameState`
  `clone_from`-ed instead of `clone`-d would recycle every Vec's capacity.
  Rust's derived `Clone` does *not* override `clone_from`, so this needs a
  hand-written one — big, and it has to stay in step with the field list.
* **Widen the no-checkpoint path.** The skip is currently one action shape.
  Every `return Err` before `perform_action_inner`'s dispatch is a pure
  validation guard that has touched nothing; the question is which *dispatch
  arms* can also be proved mutation-free-on-error.
* **Count the checkpoints first.** `perform_action` runs 18,208x on this
  workload against `perform_action_inner`'s 44,000+; the bot's probes
  (`would_accept`, `sim_step`) already take the un-checkpointed path, so the
  18,208 are real actions. Establish that number before designing anything.

**(-12) `auto_tap_for_cost_inner` — 277,149,670 Ir / 14.37 % inclusive over
18,340 calls, still the second-largest engine subtree.** The forty-second pass
took **-61,693,204 / -18.2 %** off it (A, B, C, E, F all land here) and the
shape has changed:

* **`activate_ability` is 163,106,449 / 8.46 % of it**, 18,340 calls at ~8,900
  Ir each, from ~13,200. The cold-group unshare, the thirteen
  `is_mana_ability` walks and ten of the fourteen battlefield walks are gone.
* `mana_source_table` is **75,281,100 / 3.90 % over 7,370**, from 81,987,227.
* **18,832 of the activations are still a land tapping for mana.** What that
  land still pays, in order: `card_keyword_possible` for CR 602.5's
  `CantActivateTapAbilities` (**21.6 M / ~1,148 Ir a call, one per
  activation** — see the new note under (-11)), `continue_ability_resolution_x`
  + `resolve_effect` (~58 M over 18,750), the trigger dispatch and the SBA
  sweep each tap drags behind it.
* Where to start next: `--tree=calling` on `activate_ability_inner` again — the
  table is different now.

**(-11) `card_can_grant_keyword` — 28,547,744 incl / 1.48 %, 15,247,954 self.
Second correction, and it points somewhere new.** The fortieth pass's note
(the union device saves nothing, because a land tap reaches the *first* gate
and stops) still stands. What the forty-second pass measured is the size of
that first gate: **`card_keyword_possible` costs 21,620,266 Ir over 18,830
calls from `activate_ability_inner` alone — 1,148 Ir per activation**, and
~830 of that is `keyword_grant_in_scope` walking ~18 permanents at ~46 Ir
each. The 46 is not a decision: `card_can_grant_keyword` loads five fields
(`equipped_bonus`, `soulbond_bonus`, `level_bands`, `station`,
`static_abilities`) off a large `CardDefinition`, i.e. it is pointer-chasing,
not computing. **Two shapes, neither tried:** (i) a per-*definition* "can this
printing grant any keyword at all" bit, which collapses the five loads to one
— but `CardDefinition` is built by ~thousands of catalog factories with
`..Default::default()` and derives `PartialEq`, so it needs a normalisation
pass or a side table, not a field; (ii) a board-level memo with an epoch,
which nothing on `&self` can hold today. Do not take the fusion device; it has
lost three times.

**(-10) Allocation is still the program: 1,232,517 allocations and 1,286,325
frees in six games, ~13.4 % of the tip.** The forty-second pass took 61 k
allocations off it with (E) and (F) — the same rule as the fortieth's: **ask
what a `Vec<T>`'s `T` costs, and whether the enclosing loop runs per
permanent, before asking how often the `Vec` is built.** By direct caller at
the tip:

* `finish_grow` **215,649** — Vec growth. Top source-level caller is
  `finalize_cast` (28,878 growths, and the top `memcpy` caller in the program
  at 126,492 copies): the per-turn cast logs
  (`spell_names_cast_this_turn`, `spell_ids_cast_this_turn`,
  `spell_casts_this_turn`) each take a `push` per cast through `PlayerData`'s
  CoW handle. `CastProfile.card_types` is still a `Vec<CardType>` clone per
  cast — the `ColorSet` treatment (F) applied to card types would remove it,
  but `CardType` has no bitset yet.
* `from_iter` **187,120** — the `.collect()`s. `cast_candidates` 5.36 % and
  `mana_source_table` 2.65 % are the two named callers.
* `clone_from_ref_in` **177,500** — CoW unshares of `CardInstance` /
  `PlayerData`. Inherent to the checkpoint; see (-13).
* `computed_permanent` **103,318** — one `Arc::new(ComputedPermanent)` per
  memo miss, plus the `Vec`s inside it. 7,960,150 Ir in allocation alone.
* `effect_produces_color` — **PAID, -0.496 %, the pass's eighth commit.**
  `effect_produced_colors` is the set-valued primitive and the predicate
  delegates to it. The remaining `effect_produces_color` callers
  (`untapped_producers_of_inner`, the two affordance probes,
  `source_color_signature`) still ask one colour at a time inside a loop over
  colours — the same rewrite applies and has not been measured there.
* `granted_abilities_of` (9.96 M self over 57,484) walks
  `me.definition.static_abilities` **twice** — once for the five
  `HasActivatedAbilitiesOf*` flags, once for Conspicuous Snoop's
  `HasActivatedAbilitiesOfLibraryTop`. Fold the second into the first.

**(-15) `advance_step` — 308,428,973 / 15.99 % over 22,892, the largest engine
subtree, and the forty-second pass only scratched it.** `do_untap` came down
25,207 -> 21,043 Ir a call and still runs **six separate
`battlefield x static_abilities` walks** (untappers, the filtered/Endbringer
set, Storage Matrix, the prevention set, the untap caps, the may-decline ask)
for boards where every one of them is empty; `do_cleanup` is ~15,556 Ir a call
and was not looked at. Both are once-per-turn, so the arithmetic is 1,764
calls — but that is 2 % of the program between them. The device is (-7)'s:
gate the site, not the read, and `debug_assert!` the guarded body is a no-op.

**(-8b) `card_type_change_in_scope`, the gate's own residue — CLOSED, and
it is the third measured loss for the same device.** It reads **21,776,000 Ir
/ ~1.0 % over 10,670 calls**, and **19.3 M of that is `slice/iter/macros.rs`
+ `ptr/non_null.rs`** — walking, not deciding. So: hoist the bit into
`sba_board_scan`, which already walks the same battlefield and every card's
`static_abilities` one block earlier in the same function, and read
`scan.type_change` for free. Written, suite-green (the staleness
`debug_assert!` held over all 18,645 tests), and **+16,934,080 Ir /
+0.77 %**. Reverted.

**Three attempts, three losses, and the rule they establish.** Folding the
gate's walks into one pass: **+0.55 %**. Folding them and the death legs:
**+1.24 %**. Hoisting one of them into an existing walk: **+0.77 %**. The
third is the interesting one, because it is not "one more walk" — the walk
was already happening — so the cost has to be the *body*:
`sba_board_scan`'s per-card loop is already large (a dozen `|=`, four nested
`for`s over subtype vectors), and inlining `card_can_change_card_types` into
it slows the whole scan by more than the separate specialised `any` saved.
**A tight, specialised, short-circuiting `any` over a `Vec` is very cheap on
this codebase — cheaper than the marginal cost of adding its body to a loop
that is already big.** (-6) is the standing counter-example and its
arithmetic still holds *there*, where the four walks were over
`definition.static_abilities.is_empty()` inside a function called 18,386
times, not 10,670. Cost the body against the iteration before assuming
fusion pays; on the evidence here it usually does not.

**What is actually left at this site**, for whoever wants the 1.0 %: not
fusion. Either make the *answer* cheaper to reach (the predicate itself is
only 2.5 M of the 21.8 M — the rest is the walk, so a board-level
memo/epoch is the shape, not a re-arrangement), or drop the question:
the gate only needs it to decide whether non-creatures join the death legs,
and a permanent that is not a printed creature contributes nothing unless
something animates it *and* the animation leaves it at ≤ 0 toughness.

**(-8) `check_state_based_actions`' death sweep — PAID, `-2.277 %`,
thirty-eighth pass.** All the numbers and the two devices are in the Log
block. What the entry taught, kept because both halves recur:
**(i) a `compute_*` site costs gather + layer pass + collect** — this one was
costed off its 0.83 % gather row and paid 2.28 %, so read the
`spec_from_iter` caller table beside the gather table before ranking the next
one. **(ii) A layer-7 presence gate has to be *signed* to be worth
anything.** The entry's own prediction — that a `pt_change_in_scope`
predicate would be a wash because every anthem is layer 7 — was correct, and
the way past it is to ask whether computed toughness can come out *below*
instance toughness, which every positive anthem answers `false` to for free.
The prediction that the sixth leg needed instrumenting before the first five
was also correct and cost one probe build: **9,228 / 10,670 sweeps (86.5 %)
are quiet on instance reads alone, and all 9,228 are quiet on the exact
gathered test too.**

**(-9) `combat_damage_computed`'s `compute_permanents` — 15,918,359 Ir /
0.71 % over 3,274 calls (4,862 each).** One gather plus a per-participant
layer pass per combat-damage step, on a `&mut self` path, so it gathers every
time. It is genuinely needed — the whole resolver reads its `computed` — but
it is taken **twice per combat** whenever the first-strike step runs, and
`has_first_strikers` (now gated, thirty-seventh pass) has just decided the same
question off the same board immediately before. Two shapes worth costing: hand
the first-strike step's computed set forward to the regular step (unsound as
written — damage is dealt in between and CR 510.2 is a fresh assignment), or
have `advance_step`'s gate and the resolver share one scope. The
`compute_battlefield` half of this entry is **paid** (`59c964dc`, -0.223 %):
the 310 calls at 16,939 Ir each were `apply_combat_decision_answer` taking a
whole-board view for a participant-scoped question, not the free-divider
fallback. **That is the second finding this pass that Ir/call did not
produce** — the device that did was asking which sites take a whole-board
view to answer a question about two to six named permanents. `view.rs` (two
sites), `bot.rs:1711`/`1752` and `eval.rs:1309` are the unread siblings.

**(-7) `scale_damage_to` — PAID, `-1.640 %`, thirty-sixth pass.** One gate
in front of the whole function; the site is off the caller table and
`scale_damage_to_inner` never runs on the `fixed` workload. What the entry
taught, kept because the shape recurs: **gate the site, not the read** — it
removed a third as many gathers as (-3) for nearly twice the Ir, because
the cost was ~8 battlefield walks and an allocation per call, not the
gather. And a gate with no shared choke point can audit against its own
outcome (`debug_assert_eq!` that the guarded body is a no-op) instead of
against an enumeration.

**What is left here, unmeasured and probably small:** `source_cp` is still
taken eagerly *inside* the body for two uses — the source's computed colours
and controller, and `source_cp.is_none()` as an "is the source a battlefield
permanent?" test. The second needs no gather (`battlefield_find(s).is_none()`
answers it); the first is a genuinely new family (layer 2 control-change,
layer 5 colour) and would need its own enumeration. Only worth doing if a
profile puts the body back on the table, which on this workload it is not.

**(-6) Four presence gates, four walks of one list — the thirty-sixth
pass's residue, and the arithmetic is measured, not guessed.** Closing (-3)
removed 18,386 gathers worth ~49 M Ir and netted 20.6 M, because the gates
that replaced them cost **~28 M Ir / ~1,540 per activation**. They are four
separate `battlefield.iter()` passes — `ability_strip_in_scope`,
`card_type_change_in_scope`, `land_type_change_in_scope` and
`card_keyword_possible`'s `keyword_grant_in_scope` — over the same list,
each mostly spent on `definition.static_abilities.is_empty()` for a board
of vanilla creatures and basic lands. **The fix is the shape
`gather_continuous_effects_inner` already uses on itself**: it folds eleven
whole-board passes into one `sa_cards` walk that sets a flag per family,
"so the emitted effect sequence is unchanged". A `PresenceBits` built in one
walk and asked four times is the same trick one level up.

Two things to settle before writing it, because they decide whether it
wins. **(a) Laziness is worth real money on this path** — a land tapping
for mana never asks the land-type gate (`printed_land_mana_basic` returns
`None` for a non-basic, and `is_mana_ability` skips the third CR 602.5
gate), so a fused walk that computes every bit eagerly gives some of the
win back; measure eager-fused against the four lazy walks rather than
assuming. **(b) `keyword_grant_in_scope` is not battlefield-only** — it also
walks command, emblems and graveyards, and it is `pred`-parameterized, so
it fuses with the other three only for the battlefield leg. Start with the
three type-family gates, which are the same walk over the same list with no
parameter.

**(-5) `evaluate_requirement_static`'s lazy layer view — CLOSED. Card-type
half paid `-1.333 %` (thirty-fifth pass) and the land-type twin shipped in
the thirty-sixth.** The `OnceCell` at `eval.rs` serves five layer-4
families: `has_type` (card types, gated), `has_ctype` (creature types),
`has_atype` (artifact subtypes), `has_ltype` (land types — the predicate now
exists as `land_type_change_in_scope`, built for (-3)) and `has_stype`
(supertypes). The three ungated ones no longer appear on the
`computed_permanent` caller table at all — the whole `eval.rs` row went to
zero — so **their traffic is not measurable on the `fixed` bench and none of
them is worth a predicate until a profile puts one back on the table.** The
one fact kept so it is not re-derived: creature types, artifact subtypes and
supertypes have **no cheap predicate** — `AddCreatureType` /
`SetCreatureTypes` alone have ~20 emitters, and `shallow_creature_types`
already spares `has_ctype` the gather on the mid-gather path.

**The device the paid half used, because the next family will want it.**
`permanents_with_abilities_removed`'s cross-check re-gathers whenever its
gate says "no", and a gate asked 15,574 times per six games cannot afford
that in a debug suite. Check the implication in the *sound* direction
instead — a `debug_assert!` in `gather_continuous_effects` that a gathered
set carrying the modification implies the gate said `true`. It runs only
where a gather already happened, and the whole suite audits it.

**(-3) `activate_ability_inner`'s gather — PAID, `-0.879 %`, thirty-sixth
pass.** All 18,386 gathers removed; the row is off the `computed_permanent`
caller table. Four legs in the end, and each is a reusable predicate:
`ability_strip_in_scope`, `card_type_change_in_scope`,
`land_type_change_in_scope` (built here) and `card_keyword_possible`. What
the entry taught, kept because (-6) is its direct descendant: **removing
100 % of a 2.09 % site netted 0.88 %**, because four battlefield walks per
activation is not free. See the Log block and (-6).

**The auto-tap scope is *not* the way in, and here is the arithmetic so it
is not re-derived.** `auto_tap_for_cost_inner -> activate_ability` is 18,832
calls over 8,892 payments (2.1 per payment), so a scope spanning the tapping
loop would fold ~52 % of these gathers — but tapping is a layer input in at
least four places inside `gather_continuous_effects_inner` (relative lines
726, 2571, 2589-2593, and every `WhileCondition`), so the scope is unsound
without a guard over all of them. Moot now that (-3) is paid with an exact
gate, and kept only so the idea is not re-proposed.

**(-2b) `apply_prevention_shields`' Absorb read — PAID, `-0.855 %` with
`damage_prevented_by_protection`, thirty-fifth pass.** Both now ask
`card_keyword_possible` before gathering. What the entry taught, kept
because the next gate will want it: the keyword-grant predicate is the
device, and **its own cost is the thing to watch** — the first version was
a wash at 1,643 Ir a call. See the Log block.

**(-4) The rest of the "eager view above a wide `match`" sweep — cheap,
mechanical, and one hit already paid -2.128 %.** *Two of the four named
below were read on the 2026-08-14 retake and are **not** hits*:
`evaluate_predicate` (eval.rs:1499) and `evaluate_value` (eval.rs:135) bind
nothing above their `match` — every layer read is inside an arm — which is
what the paragraph's own warning predicted. `evaluate_requirement_on_card`
and `resolve_selector_inner` are still unread. The thirty-third pass took
`evaluate_requirement_static`'s. The siblings are the other wide dispatchers
that bind something expensive before the arm is known: `evaluate_predicate`
(eval.rs:1499, 10 `computed_permanent` sites across arms),
`evaluate_value` (eval.rs:135, 9, incl. a `compute_battlefield`),
`evaluate_requirement_on_card` (4) and `resolve_selector_inner`
(effects/mod.rs, 4). **Read each for a `let` *above* the `match`, not for
the count** — a call inside one arm costs nothing on the other arms, and
all four of those counts are per-arm, which is why none of them is
automatically a hit. The enumeration script that produced the list is three
lines of `awk` over `fn` signatures: `&self`, no `with_frozen_layers`, two
or more layer reads. Rank by profile before writing: `evaluate_value` and
`evaluate_predicate` do not appear on the tip's caller table at all.

**(-2) `CardCold` — PAID, `5174acd3`, -1.969 %.** What is left of the
entry, so the next run does not re-derive it. The `CardData` deep-copy
table (`--tree=caller` on `Arc::make_mut`, rows whose *file* is
`crabomination_base/src/card.rs`) reads **~44.5 M / 1.76 %** at the tip,
and its largest row is `activate_ability_inner` **15,346,614 over 18,312 =
838 Ir each**. 838 is the ~126-field struct memcpy plus the collections
that stayed hot on purpose, i.e. **there is no second `CardCold`** — the
tenth pass's rule (*group size x unshare probability < sum of the
individual clone costs*) is what stopped the group at 22, and the fields
deliberately left hot are `damaged_by_this_turn`,
`damage_by_source_this_turn`, `damage_by_source_name_this_turn`,
`blocked_attackers_this_turn`, `damaged_players_this_game`,
`damaged_permanents_this_game`, `granted_keywords_eot`(`_ts`), `counters`,
`keyword_counters` and the five `Option<Arc<CardDefinition>>` faces. The
next thing on this shape is **cost per clone, not clone count**: a
`CowBox` on one individual hot-but-large field, when its clone count far
exceeds its write count. Cost the ratio before writing it.

**(-1a) The `iter_mut().find()` half of the unshare sweep — the syntactic
filter misses it.** The thirty-first pass's sweep matched
`for \w+ in &mut self\.` and `\.iter_mut\(\)` over a *zone*; a
`self.battlefield.iter_mut().find(|c| c.id == id)` unshares the zone and
then deep-copies the found card, and reads as an ordinary lookup. One was
taken at `5174acd3` (the `pending_etb_counters` drain, once per permanent
entering). **54 more sites match
`\.(battlefield|hand|graveyard|library|exile|command)\.iter_mut\(\)` under
`game/effects/` and `server/`** and were not audited; nearly all sit in a
`run_effect` arm where the write is genuine, so this wants the profile's
call counts rather than a syntactic pass — an arm that runs on a bench deck
*and* writes conditionally is the shape. Read the `make_mut` caller table
first; anything not on it is not worth touching.


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

   **The SBA row, denominated at the thirty-second pass's tip — and the
   denominator is the whole finding.** `check_state_based_actions` is
   **10,670 calls / 151,182,228 Ir inclusive / 5.98 %**, i.e. ~14,170 Ir a
   pass. Three separate `--tree=caller` rows read exactly **10,670**: the
   gather (18,599,381), its own `stack.rs` self (23,915,320) and a
   `game/mod.rs` row (469,480). **So a pass takes exactly one gather and
   there is nothing to fold inside one** — the two layer sites in the
   function body (the CR 603.8 flip `compute_battlefield`, the CR 704.5j
   legend-rule `computed_permanent`) are both behind `sba_board_scan` flags
   that are false on every bench board. The `spec_from_iter` collect row is
   **82,634 over the same 10,670 passes = 7.74 collects a pass**,
   129,943,199 / 5.14 %, i.e. **1,572 Ir a collect and 12,178 Ir a pass**.
   The lever is therefore *fewer passes* or *a cheaper per-pass body*, and
   **not** deduplicating gathers. Do not re-derive this.
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

   *Re-read at the thirty-second pass's tip `5174acd3`, and the entry has
   shrunk under the passes that took the combat scopes.* The gather is
   **118,978 calls / ~228 M inclusive / 9.04 %** by caller:
   `computed_permanent` **85,808** (169,154,302 / 6.69 %),
   `frozen_effects` 18,916, `check_state_based_actions` 10,670,
   `compute_permanents` 3,274, `compute_battlefield` 310. And
   `computed_permanent`'s own callers now read *under* the 2,000 threshold:
   `damage_prevented_by_protection` 28,907,705 / 18,986 (**1,523**),
   `scale_damage_to` 25,846,331 / 14,624 (**1,767**),
   `permanent_has_keyword` 10,889,788 / 8,328 (**1,308**),
   `dying_snapshot` 10,739,232 / 3,420 (**3,140**),
   `blocker_can_block_attacker` 4,415,123 / 15,368 (**287** — in a scope).
   **The threshold moved because the gather got cheaper, not because the
   callers got scoped**; 2,000 was calibrated when a gather was ~2,000 Ir
   and it is now ~1,900 with a memo hit near zero, so rank by *total*, not
   by the ratio.

   *And the obvious remaining site was costed and is not one.*
   `deal_damage_to_from` — the noncombat damage funnel in
   `effects/movement.rs` — runs ~14 `&self` layer reads before its first
   write (`computed_permanent`, `damage_prevented_by_protection`,
   `damage_sealed_by_aura`, the three per-source prevention checks, the
   Iroas/Glacial-Chasm/`player_protection_card_types` trio, the four
   `permanent_prevents_*` shields, `scale_damage_to`) and is the one
   damage path the combat passes never scoped. It **does not appear at all
   in `--inclusive=yes --threshold=99`**, i.e. it is under 0.5 % on the
   bench decks, which are creature-combat archetypes. The refactor is real
   but must not be priced off the noncombat funnel: the prefix is broken
   into two `&self`-only runs by three `&mut self` redirect branches
   (Martyrs of Korlis, Reverberation, Harsh Judgment), so it is two
   `with_frozen_layers` closures, not one. **Do it only if a wider deck
   pool moves it onto the profile.**

   *The memo itself is healthy — checked, not assumed.* Inside a scope
   `computed_permanent` hits `st.perms` by id and returns a refcount bump,
   and a miss reuses `st.memo`'s gather; the first computed read of a scope
   fills both under one lock. So a caller inside a scope pays
   `apply_layers_one` once per *card*, not per call. Do not go looking for
   a broken memo.

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
11. **MCTS leaf-evaluation throughput — the strength-conversion lever.**
    r42 Part C measured 33.0 / 56.8 / 121.9 / 249.4 s per game serial at
    64 / 128 / 256 / 512 iterations — linear, so every leaf pays a full
    net eval and nothing is amortized across a search. Rounds 27/29/42
    say raw iterations are the only search lever that pays (256 vs 64
    head-to-head: +2.1 ±0.4 on r42's first ladder seed), and adoption of
    higher rungs into the client is latency-gated — so a 2× here is not
    a wall-clock nicety, it converts directly into playing strength at
    fixed latency. Shapes to cost, cheapest first: batching leaf evals
    (the training-side GPU collator already batches; the ladder/client
    search path evaluates one state at a time — cost the CPU-batching
    win before assuming it), a per-search transposition cache for
    repeated leaf states, early adjudication of settled rollouts.
    Anything that changes *which* states get evaluated needs a
    `bot_ladder` win-rate gate per the round-29 house rule; pure
    batching needs only the perf numbers and a golden-trace check.
    **Part 1 landed (forty-first pass): the forward pass was scalar** —
    vectorized matvec took 64-iter games −44 % and 256-iter −40 %, and
    the leaf from 39 % of search wall to 12 %. The measured split
    (`CRAB_MCTS_TIMING=1`) says the remaining cost is the **rollout
    sim: 88 %**, ~63 engine actions per rollout at ~16 µs — so the next
    rungs here are rollout-side (early adjudication of settled
    rollouts, a cheaper rollout policy tick), which change what gets
    evaluated and therefore gate on the ladder, or the engine action
    loop itself (the (-12) auto-tap subtree above).

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
