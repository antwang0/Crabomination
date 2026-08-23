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

**A number that is too good is a bug report.** (I) first read **-30.0 %**
(1,918,782,724 -> 1,343,642,428) from a gate that was supposed to be worth a
fraction of a percent. The cause was Rust precedence: `if !any_static || a &&
b && c` parses as `(!any_static) || (a && b && c)`, so on a board with no
statics the Seedborn-untapper loop pushed *every* controller and every player
untapped everything each untap step — fewer decisions, shorter games, a third
of the instructions. Correct, it reads -0.167 %. **Sanity-check the magnitude
against what the change can physically remove before running the suite**, and
check the `--bench` invariants (`decisions`, `turns_per_game`) on any change
whose Ir moves more than its blast radius allows: they are one 2-second run
and they fail loudly.

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

## Build time — the file-size lever is dead, measured 2026-08-23

**"Oversized engine files dominate incremental rebuilds" is false on this
codebase.** Measured directly: touch one file, rebuild, time it. `dev`,
`CARGO_TARGET_DIR=target-probe`, warm.

```text
cargo build -p crabomination --lib        (2 runs each, after a warm-up run)
  8.7 / 8.5 s   36,684 lines   game/effects/mod.rs      <- the biggest file
  8.7 / 8.5 s      266 lines   decklist.rs              <- one of the smallest

cargo test -p crabomination -p crabomination_tests --no-run
 41.1 / 33.6 s  36,684 lines   game/effects/mod.rs
 33.2 / 39.1 s     266 lines   decklist.rs
        32.5 s  23,672 lines   game/mod.rs
```

**A 138x difference in file size buys nothing.** The `--lib` rebuild is a flat
~8.6 s of dependency-graph load, metadata and codegen that the touched file's
size does not move; the test-binary rebuild is 33-41 s of **relinking twenty
integration binaries**, and its spread does not order by file size either (the
266-line file was the slowest of the second round).

**So: do not split `effects/mod.rs` (36.7 k lines), `game/mod.rs` (23.7 k) or
`actions.rs` (16.5 k) for build time.** There may be other reasons to split
them — reviewability, merge conflicts with a concurrent session — but the
iteration loop is not one, and a mechanical move of a 34.7 k-line `impl
GameState` block is not free of risk. The lever that *does* bear on the
33-41 s is the one already written down: **keep the integration-binary count
flat or lower, and never add a new top-level `tests/*.rs`.** Twenty binaries
is what the relink costs.

**Test-suite cleanup delta, 2026-08-23**, recorded because the rule asks for
it and because the answer is "nothing", which is the useful part:

```text
tests            18,729 -> 18,708   (-22 per-set registration echoes, +1 audit)
LOC              -607 / +70         (net -537 across 14 files)
test binaries    20 -> 20           (flat, as the standing rule requires)
rebuild after touching game/effects/mod.rs
  before         41.1 / 33.6 / 33.2 / 39.1 / 32.5 s   (mean 35.9)
  after          34.0 / 34.7 / 32.9 s                 (mean 33.9)
```

**Inside the noise band, and the section above says why**: 537 lines against
`classic_sets`' own 116,940 is 0.5 %, and the rebuild is link-dominated
anyway. **So do not justify a test-suite sweep on build time.** The
justification for this one is the maintenance shape — twenty-two hand-kept
per-set factory lists, each of which goes stale the moment someone adds a
card and forgets it, replaced by one tree walk that cannot.

Not measured here, and still open: the `release` / `profiling-fast` rebuild
(~13 min cold for the engine) is a codegen-bound build where CGU partitioning,
not query invalidation, decides the cost. Nothing above says anything about
it.

## Baseline

**The forty-fourth pass is a paired `release` A/B over six alternated pairs.**
Both sides built `release` + mimalloc from one tree, run alternating in one
sitting on one box. Base is `c0f4e3b6` (the pass's own base), tip is
`36592fd8`:

```text
                     base (c0f4e3b6)                 tip (36592fd8)
games_per_s          151.02 160.50 162.14             160.26 168.56 164.72
                     149.06 154.04 153.28             158.98 160.33 155.73
mean                 155.01 (spread 8.4 %)            161.43 (7.9 %)  -> +4.14 %
per pair             +6.12 +5.02 +1.59 +6.66 +4.08 +1.60   -> 6/6 positive
host_calib_ms        47-59 (both sides interleaved)
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz (both)
decisions            196,220                          196,220   byte-identical
turns_per_game       27.53                            27.53
stalls               0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism          ok (all 160 pairs split, every run)
peak_rss_mib         29.9 - 31.0                      29.1 - 31.1
```

**+4.14 % wall-clock against -5.310 % Ir, and the gap is the one this file
predicts.** Two of the four rows are allocation removals measured under
callgrind's *system* allocator, where an allocation's instructions are counted
and mimalloc's cheaper path is not; -5.310 % Ir would be +5.61 % if every
instruction were worth the same, and it reads +4.14 % with the allocator that
ships. Quote the paired A/B.

**Behaviour preservation, and it is stronger than the golden traces here.**
The base and tip binaries both ran `--a gang --b gang --games 200 --threads 3
--decks all --paired` and printed **byte-identical output over 3,400 games
across 17 archetypes** — every archetype's record, 1,700 paired splits,
`rho -1.000`, **0 undecided**. Two tip processes agree with each other too.
That check also covers the concurrent session's `ProtectionKind` refactor,
which is in the same range.

**Crash-freedom at the tip.** `overflow` profile (`release-fast` +
`overflow-checks`), `--a gang --b gang --games 400 --threads 3 --decks all`,
seeds 11 / 12 / 13: **20,400 games, 20,396 decided, no panic and no arithmetic
overflow**, 41-44 s a seed against an 8m30s build. The 4 undecided are all on
seed 11 — rules draws, and fewer than the 8 the thirty-second pass recorded
because rounds 43-47's search changes moved them, not this pass.

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

**And a third box, at the forty-third pass's tip, that neither the probe nor
the CPU string predicts.** Same `--bench`, `release` + mimalloc, three runs:
**121.31 / 122.83 / 120.71 (mean 121.62)**, `host_calib_ms` **48 / 51 / 53**,
`host_cpu` `Intel(R) Xeon(R) Processor @ 2.80GHz`, `peak_rss_mib` 29.6-30.0.
That is **-25 %** against the reading directly above, on a calib band that
overlaps it (48-53 vs 45-52) and a *higher* clock string than the paired
A/B's 2.10 GHz. **So `host_calib_ms` does not discriminate box class as
finely as this file has been assuming** — it caught the half-percent question
above, and it says nothing useful about a 25 % one. Do not read this reading
as a regression: the same tip measures **1,911,862,094 Ir** under callgrind
against the base's 1,918,781,907, base and tip in one sitting on one box.

**What is portable is the invariant set, and it is byte-identical on all
three boxes**: `decisions` **196,220**, `turns_per_game` **27.53**, `stalls`
0 (cap 0 / stuck 0 / draw 0), `determinism ok` (all 160 pairs split, every
run), `peak_rss_mib` 29.6-30.0 against the A/B's 27.5-29.8. **Check those
first; quote the paired A/B for anything else.**

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

**The superseded anchors of passes 28-34, compacted.** Full `--bench` blocks
are in git (`git log -- PERF.md`). Every one is `release` + mimalloc, 320
games, and every one reads `decisions` **193,232**, `turns_per_game` **26.98**,
`stalls` **0** (`cap 0 / stuck 0 / draw 0`) and `determinism ok` — that
invariant set is the only column that chains across them.

| tip | pass | games/s (mean) | calib | RSS |
|---|---|---|---|---|
| `6cc0bdc3` | 34 | 136.75 (6 runs) | 47-57 | 29.1-30.8 |
| `35fdfce3` | 33 | 130.71 (6 runs) | 44-71 | 29.2-29.4 |
| `5174acd3` | 32 | 125.16 (4 runs) | 45-46 | 29.0-29.3 |
| `76804984` | 31 | 120.10 (4 runs) | 45-46 | 29.1-29.4 |
| `5034eb2f` | 30 | 115.40 (6 runs) | 44-48 | 29.0-29.4 |
| `ed4c152c` | 28 | 94.23 (3 runs) | 47-52 | 29.0-29.4 |

The three lessons those blocks were carrying, which are why the numbers can go:

- **Do not chain two blocks whose `host_calib_ms` bands don't overlap.**
  `130.71 -> 136.75` looks like +4.6 % and is a different container; the
  `bdc11c86` re-anchor to 163.62 is +20 % and *none of it is the change*.
- **A stall figure without its invocation is two figures.** `--decks all`
  reads **6 draws / 5,100 (0.12 %)** under `--bench --threads 1 --games 300
  --seed 11` and **2 undecided / 5,100 (0.039 %)** under `--a gang --b gang
  --games 300 --threads 3`. Quote the command.
- **Keep the pre-pass binary and diff its output, don't compare recorded
  constants.** At `645b978d` the base and tip both ran `--games 300 --threads
  3 --decks all` and printed byte-identical output over 5,100 games and 17
  archetypes. One `cp`, three minutes, and it answers what a recorded
  decision count only answers if the invocation matches.

**Crash-freedom, the standing recipe and its record.** The `overflow` profile
(release-fast + `overflow-checks`) over `--a gang --b gang --games 400
--threads 3 --decks all`, seeds 11/12/13 (+ 3/29/41 at `5174acd3`): **no
panic and no arithmetic overflow** at every tip from the thirtieth pass on —
20,400 games each, 27,200 at `5174acd3` — with the 8 undecided always on seed
11 and always the same rules draws. ~50-90 s a seed against a 16-minute build.
Cheap enough to run every pass; do.

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

### Forty-fourth pass — the round-closing pass stops buying a restore nobody reads

Base `c0f4e3b6` re-read at **1,911,861,368** (the forty-third pass recorded
1,911,862,094 on another box; the 726 Ir is argv). All readings
`profiling-fast --no-default-features`, callgrind, `--a gang --b gang --games
6 --threads 1 --seed 1 --decks fixed`.

A second session pushed `d3f797e5` + `6939bc93` between (A) and (B), so (B)'s
base is (A)'s tip rebased onto them: **1,857,534,552** against (A)'s
1,857,530,653, **3,899 Ir**. The whole-catalog tree walk is Ir-neutral on this
workload, which is worth knowing — it is the second concurrent session on this
branch and the first one whose commits could have moved a reading.

| step | before -> after | what |
|---|---|---|
| A | 1,911,861,368 -> 1,857,530,653 (**-2.842 %**) | the round-closing `PassPriority` skips the transaction checkpoint |
| B | 1,857,534,552 -> 1,838,517,483 (**-1.024 %**) | the target scans take one freeze instead of one per candidate |
| C | 1,838,517,483 -> 1,814,461,559 (**-1.308 %**) | the trigger dispatcher stops allocating for a Ring nobody wears |
| D | 1,814,461,559 -> 1,810,341,507 (**-0.227 %**) | two per-action walks stop allocating for statics nobody controls |

**(A) is half of (-13)'s `-5.47 %` ceiling, and it is the half that was
provable.** `GameState::clone` from `perform_action` drops **18,208 -> 8,266
calls**, so the round-closing pass was **9,942 of the 18,208 checkpointed
actions (55 %)** at **5,465 Ir each** — inside the 5,750 the forty-third pass
measured for the whole population.

**The forty-third pass's audit said this was not provable from the
signatures, and it was reading the wrong signatures.** Its objection is
sound about `pass_priority` itself — the non-trivial branch writes
`consecutive_passes = 0` and then propagates — but the question is not
whether the branch mutates before it can fail; it is whether anything
downstream of it *ever* fails. A transitive closure over the engine's 149
`Result`-returning functions says **46** are reachable from `pass_priority`
and exactly **five** of those raise at all:

* `run_effect` — `ModeOutOfBounds`, two `DecisionAnswerMismatch`
* `apply_pending_effect_answer` — 43 `DecisionAnswerMismatch`
* `check_target_legality_inner` — `InvalidTarget` / hexproof / shroud
* `cast_card_for_free` — `CardNotInHand`
* `try_pay_after_snapshot_mode` — `ManualTapRequired`, `Mana`

None of the step machinery raises: `advance_step`, `do_untap`, `do_cleanup`,
`resolve_combat`, `resolve_first_strike_damage` and
`resolve_combat_damage_with_filter` have **zero** `Err` sites between them
(combat.rs's 82 all live in `declare_blockers` / `declare_attackers_banded`,
which are actions, not steps), and the whole `effects/` tree has **one**
(`effects/mod.rs:5011`). Every survivor raises only where an engine
invariant is already broken, and a restore repairs none of them — the bot
passes again, fails again, and the game stalls instead of continuing. The
checkpoint was buying a different bug, not a fix.

**What replaces it is an audit, not an argument.** Debug builds keep the
clone and assert that an `Err` left the serialized state byte-identical, so
the 18.7 k-test suite checks the claim on every failing pass it exercises.
Release pays nothing. The audit is blind to the ~78 `#[serde(skip)]`
per-resolution scratch fields; those are reset by the next resolution either
way.

**What is left of (-13): 8,266 checkpointed actions, ~45 M Ir, ~2.43 % — and
it is the half where the checkpoint earns its keep.** Those are casts,
activations and combat declarations: mana paid then a target rejected,
`declare_attackers` failing mid-loop. That is the partial-mutation family
Phase 1 was built for. Do not take it without a per-arm proof that reads as
well as this one.

**(B) `check_target_legality` opens its own freeze scope, so a scan that asks
it per candidate re-gathers every continuous effect in the game per
candidate.** `legal_targets_for_filter` (the CR 115.4 enumeration core) and
`auto_target_for_effect_avoiding_set_xc` (the picker core) both walk the
battlefield plus every player asking
`evaluate_requirement_static && check_target_legality` per entity, both are
`&self`, and neither held a freeze. One `with_frozen_layers` around each body
— nested freezes reuse the outer memo, so the inner scope becomes free — is
**-19,017,069 / -1.024 %**, measured base-and-tip in one sitting.

Two more sites with the same shape were written and **reverted for want of a
measurement**: `bot.rs`'s alternate-target scan and `actions.rs`'s
`ChooseTarget` candidate walk together read **-11,778 Ir**, i.e. nothing, on
`--decks fixed`. They are the same idea and would pay on a pool that casts
more targeted spells; nobody should re-derive them blind.

**(C) The only `alloc_zeroed` in the program was a two-byte `vec![false; 2]`
for The Ring.** `dispatch_triggers_for_events` built it on *every* non-empty
dispatch — **53,838 over six games, 259 Ir each** — to dedupe the level-3
"Ring-bearer becomes blocked" trigger per seat, on boards where no player has
ever been tempted. Both levels the block handles need `ring_temptations >= 2`,
so one `any` in front of the whole block is exactly equivalent, and it gates
the event walk with it: **-24,055,924 / -1.308 %**, three times the
13,951,476 the `alloc_zeroed` row alone predicted, because the block's event
match and the compiler's view of the surrounding function went with it.

**(D) Two more of the same shape, and they price the tail of it.**
`resolve_extra_mana_on_land_tap` runs on every land tap (18,570) and built a
`Vec` off a whole-board `static_abilities` walk plus a clone of the
turn-scoped grant list before checking whether anything grants extra mana;
`declare_attackers_banded` collected `(source, cloned filter)` pairs into a
`Vec` and then ran `any` over it, one allocation and one
`SelectionRequirement` clone per lock per attacker for a question that
short-circuits. Together **-4,120,052 / -0.227 %** — an order of magnitude
under (C), which is the useful part: **the prize in this class is the
allocation that fires on every action, not the one that fires on every cast.**

**The pass sums to `1,911,861,368 -> 1,810,341,507`, -101,519,861 /
-5.310 %** — and lands **0.003 %** from the forty-third pass's
`1,810,396,553`, which was the ceiling it measured for a probe that skipped
*every* checkpoint and gave up the transactional guarantee to do it. Same
number, guarantee intact, by four unrelated rows.

**Wall-clock: `155.01 -> 161.43 games/s, +4.14 %`, six alternated `release` +
mimalloc pairs in one sitting, 6/6 positive.** The block is in **Baseline**,
with the wide-pool base-vs-tip byte-identity check (3,400 games, 17
archetypes) and the `overflow`-profile crash-freedom run (20,400 games, no
panic, no wrap).

**Where the gathers still are, and it is the largest structural cost left.**
53,806 continuous-effect gathers on this workload at ~1,900 Ir each, ~102 M /
**5.5 %** — see candidates (-18) and (-22).

### Forty-third pass — a cleared collection is not an empty one

Base `1032979c` re-read at **1,918,781,907** (the forty-second pass's
1,918,782,724 on a different box; the 817 Ir is argv). All readings
`profiling-fast --no-default-features`, callgrind, `--a gang --b gang --games
6 --threads 1 --seed 1 --decks fixed`.

| step | before -> after | what |
|---|---|---|
| A | 1,918,781,907 -> 1,915,090,409 (**-0.192 %**) | the three `GameState` `HashMap`s are dropped, not cleared |
| B | 1,915,082,326 -> 1,911,862,094 (**-0.168 %**) | the untap step asks once whether any static can reach it |

**(A) `HashMap::clear` keeps the table, and every later `GameState::clone`
re-allocates it.** hashbrown clones a table with the *source's* bucket count,
not its length, so a map that held entries once and was cleared allocates and
memcpys a full table on every checkpoint and every probe clone for the rest
of the game. `GameState` has exactly three: `block_map` (cleared at combat
end, twice) and the two `*_discarded_per_player_this_resolution` maps
(cleared per resolution). Assigning `Default::default()` instead costs two
stores when the map is already empty — hashbrown's `new` does not allocate —
and the next use rebuilds the table. **-16,276 allocations**, 1,232,517 ->
1,216,241, and `RawTable::clone` 5,117,712 -> 4,688,212 Ir. The *call* count
is unchanged at 104,310 (three per `GameState::clone`, 34,770 clones); only
their cost moved. **The rule generalises: any `clear()` on a collection that
a `GameState` clone reaches is a standing per-clone allocation.** `Vec` is
exempt — `Vec::clone` allocates `len`, not `capacity`.

**(B) the same idea as this pass's own refuted `do_untap` gate, and the
difference between them is the whole lesson.** The refutation above reads
"the six `battlefield x static_abilities` walks are not the cost; a
short-circuiting `any` over an empty `static_abilities` is nearly free" — and
that is exactly right about the *walks*. What (B) removes is not the walks but
the **blocks around them**: `filtered_untap` builds a `Vec` of filters and a
`HashSet`, `matrix_choice` builds a `HashMap`, `untap_caps` builds a
`Vec<(HashSet, u32)>` through a `flat_map`, and the Mist-of-Stagnation and
Seedborn passes each set up an iterator chain. One `any_static` bit computed
once, hoisting `if !any_static { Default::default() }` in front of each block,
reads **-3,220,232 / -0.168 %** — measured base-and-tip in one sitting on this
pass's own base (which re-reads 1,915,082,326 against the 1,915,090,409 above,
8,083 Ir of argv/sitting drift), and again at **-3,197,754 / -0.167 %** on the
forty-second pass's tip before the rebase. Two bases, the same answer.

**So (-7)'s rule survives one more test and gets sharper: gate the *site*, not
the read — and "the site" is the block that allocates, not the loop that
scans.** A gate that only shortens a scan of an empty `Vec` is worth nothing on
this codebase (three losses under (-8b) and one here); a gate that skips a
block which builds a collection is worth something every time.

**The pass's largest finding is a ceiling, not a commit: `perform_action`'s
checkpoint is worth `-5.47 %` and it is never read on the bench workload.**
A probe binary that skips the checkpoint entirely and `panic!`s if a restore
would have happened read **1,915,090,409 -> 1,810,396,553, -104,693,856 /
-5.47 %** — and **exited 0**, so across six games / 24 decided matches /
18,208 checkpointed actions, **not one `perform_action` returned an `Err`
that the checkpoint had to roll back** (`ManualTapRequired`, which is a
suspension rather than a failure, was exempted as it is in the real path).
That is **5,750 Ir per checkpointed action**: clone 1,194, drop 2,324, and
~2,230 of CoW unshares the action pays only because the checkpoint made its
zones shared.

**It is not capturable by skipping the pass path, and here is the audit so
nobody repeats it.** `pass_priority`'s non-trivial branch mutates
(`consecutive_passes = 0`) and then propagates from `resolve_top_of_stack`;
`advance_step` propagates from `pass_priority` (recursively, line 449),
`resolve_first_strike_damage` and `resolve_combat`. None of those is total,
so "PassPriority cannot fail after mutating" is not provable from the
signatures, and *measuring* zero failures on one workload is not a licence to
drop a transactional guarantee that exists for the partial-mutation bug
family. The work (-13) needs is the per-arm proof, arm by arm, with a
`debug_assert` in each skipped arm that an `Err` left the serialized state
untouched — the whole suite then audits the claim. **Budget it as a pass of
its own; the prize is up to 5.47 %.**

**Refuted, same pass, and it closes (-13)'s first shape: a per-thread pool of
`GameState` husks with a hand-written `clone_from`.** `1,915,090,409 ->
1,964,903,711`, **+49,813,302 / +2.60 %**. Reverted. What was built, because
the next run should not build it again: an exhaustively-destructured
`Clone::clone_from` over all 195 `GameState` fields (a new field fails to
compile until handled — the pattern carries no `..`), a `Scratch` guard that
draws a husk from a `thread_local` pool and returns it on drop, and a
`release_shared` that drops the six `CowBox` handles and the `players` `Vec`
into per-thread empty prototypes so an idle husk cannot make a live zone look
shared. All three invariants held under test — `clone_from` from a dirty
destination serializes identically to `clone`, a released husk leaves every
zone at `handle_count() == 1`, and a rejected action still restores exactly.

**The allocation saving is real and still loses.** Allocations **1,216,241 ->
1,115,853, -100,388 (-8.3 %)** — 2.9 per `GameState::clone`, as predicted.
The cost:

```text
                        clone/drop            pool
Scratch::of / clone     41,569,486 (2.16 %)   70,445,322 (3.59 %)
drop side               80,831,019 (4.21 %)   80,638,233 (4.10 %)
  of which release_shared          —          30,725,495 (1.56 %)
  of which drop_in_place  80,831,019          31,835,358 (1.62 %)
```

**`clone_from` costs ~834 Ir a call more than `clone`, against ~535 Ir of
allocator work saved.** The reason is visible in the base profile and is the
entry's one durable lesson: the hand-written `clone`'s `Self { … }` is a
*bulk* copy — 15,924,660 Ir on that one line plus one `memcpy` per clone —
because 175 of the 195 fields are `Copy` and contiguous. `clone_from` cannot
be that: it must interleave a drop and a construct per field, and nothing
merges 195 separate assignments. **On a struct this wide, `clone_from` is not
a cheaper `clone`; it is a more expensive one that happens to allocate less.**

**Where that leaves (-13), and it is a different entry now.** Split the two
sides and the allocator is a minority shareholder in both:

* **clone, 1,194 Ir:** `Self { … }` 458, its `memcpy` 289, three
  `Vec::clone`s 106, `RawTable::clone` 135, `__rust_alloc` 178.
* **drop, 2,324 Ir:** `sync.rs:drop_in_place<GameState>` is **50,619,793 /
  2.64 % — 63 % of the whole drop** — and that row is `Arc` drop glue.

**So the checkpoint's largest cost is not building or freeing the `GameState`.
It is that everything the action deep-copied through a CoW handle — because
the checkpoint made the zone shared — has to be freed when the checkpoint
dies.** That is the same 177,500 `clone_from_ref_in` unshares (-10) lists,
counted on the drop side. A pool cannot touch it: the checkpoint still
shares, the action still unshares, the copy still has to go back.

**Which leaves exactly one of (-13)'s three shapes standing: widen the
no-checkpoint path.** It is the only one that removes the *sharing* rather
than the clone. `pass_priority_is_trivial` already proves the shape works;
the open question is unchanged — which `perform_action_inner` dispatch arms
can be shown mutation-free on their error paths. Do not build the pool
again, and do not cost "make `GameState` narrower" off the field count: 175
of the 195 fields are `Copy` and have no drop glue at all, so narrowing the
struct buys a slice of the 458-Ir construction and nothing of the 2,324-Ir
drop.

**One behaviour-preserving change with no perf claim, recorded because the
rule says to say so plainly.** `ProtectionKind` replaces the hand-kept
`matches!` gate in `damage_prevented_by_protection_inner` with a list the
gate and the decision both read (see TODO's defects section — it was that
section's only open entry). **1,911,867,157 -> 1,911,871,594, +4,437 Ir /
+0.0002 %**: the compiler folds `ProtectionKind::of(kw).is_some()` straight
back into the original `matches!`, which is the result the change was
allowed to have. It is a correctness change, not a perf one.

**Null result, same pass: gating the untap step's six
`battlefield x static_abilities` walks behind one.** (-15) named them; a
`UntapStatics::gather` pass with a bit per static, plus a
`StaticEffect::core()` that peels the CR 611.2 conditional wrappers through
the same list `active_static` walks, read **+2,377 Ir / +0.0001 %**. Reverted.
The gate walk costs what the six walks cost, because each of them
short-circuits on `definition.static_abilities.is_empty()` for a board of
lands and vanilla creatures — **this is (-8b)'s lesson again, on the other
side: a specialised short-circuiting `any` is so cheap here that six of them
are not worth one pass to replace.** `do_untap`'s 37,097,627 Ir is not in
those walks: its self cost across every `file:function` entry is ~9 M, and
`do_phasing` is 5.1 M of the rest. **Whoever takes (-15) next should read
`do_untap`'s callee table first, not its walk count.**

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

**Passes thirty-nine to forty-one, compacted to an index** — same treatment.

| pass | base -> tip | levers, and what each taught |
|---|---|---|
| 39 | 2,186,153,036 -> 2,136,847,762 Ir (**-2.255 %**) | Three commits, all found by the profile and none by the candidates list. The trigger dispatcher's per-card loop early-outs on a definition with no trigger (**-1.881 %**); the CR 305.6 land-type question moves to the three loops that already reuse a scan (-0.143 %) — **and the same gate hoisted into `grant_scan` instead read +0.206 % and was reverted**, which is where "gate the site, not the read" got its second data point; the empty dispatch stops synthesizing (-0.238 %). |
| 40 | 2,136,851,050 -> 1,974,770,479 Ir (**-7.585 %**) | **Five allocation rows.** The bot's six ability generators stop building a `Vec` per permanent (**-3.990 %**); the free-activation watchdog vetoes before it clones (-1.252 %); activation holds the definition's `Arc` instead of cloning the ability out (-0.978 %); the mana-ability list stops inlining `ActivatedAbility` per element (-0.867 %); the hand sweep's mana read behind a `OnceCell` (-0.700 %) — **eagerly hoisted it read +0.350 % and was reverted**. Allocations 1,416,250 -> 1,325,868, `memcpy` **-35.2 %**. **The device: ask what a `Vec<T>`'s `T` costs before asking how often the `Vec` is built** — `Cow<ActivatedAbility>` is as large as `ActivatedAbility` (an `Effect` is 448 bytes), so a *borrowed* element still costs a ~600-byte allocation; the cost lands in `memcpy` / `_int_malloc` / `_int_free`, which no `--auto=yes` sort points back at the type. |
| 41 | not on the Ir bench by construction | **`crabomination_nn`'s `Tensor2::matvec` from one accumulator to eight, with runtime-dispatched AVX2+FMA.** Strict f32 semantics forbid LLVM reassociating a chained dot into SIMD lanes, so the old loop ran scalar everywhere; splitting the accumulators licenses it and `#[target_feature]` gets the 8-wide FMA units. **mcts-net-deep 33.0 -> 18.4 s/game (-44 %)**, forward pass 635 -> 164 µs/rollout (**4.3x**), leaf eval 39 % -> 12 % of search wall. The callgrind bench never executes the net, so this is invisible to every Ir row here — numbers are serial wall clock in the r42 Part C harness. Remaining: the rollout sim is now **88 %** of search wall (~1 ms, ~63 engine actions at ~16 µs), i.e. the engine's own action loop. |


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

**The forty-fourth pass reads 1,810,341,507 Ir at its tip.** Fresh table;
the forty-second's below is kept because its Log rows chain to it, and the
thirty-ninth's and fortieth's were dropped when their Log entries became
index rows (`git log -- PERF.md` has them).

| row | at the 44th tip | note |
|---|---|---|
| `pick_attacks_scored` inclusive | **974,200,078 / 52.45 % over 438** | **the largest subtree in the program, and it had never been read.** `simulate_attack_outcome_once` 966,249,395 / 52.02 % over **1,170 candidates** = 825,854 Ir each; `sim_step` 536,967,173 / 28.91 % over 35,316 (30 steps a candidate, 31,874 of them `PassPriority`). Candidate (-21) |
| `perform_action_inner` inclusive | 804,167,059 / **44.42 %** over 70,418 | `perform_action` reaches it 28,434x: 10,226 trivial passes, 9,942 round-closing passes, **8,266 checkpointed actions** |
| `spec_from_iter_nested` inclusive | 361,876,821 / **19.99 %** | the `.collect()` total. `cast_candidates` 103,493,914 / **5.72 % over 7,238**, `mana_source_table` 41,105,069 / 2.27 % over 7,550, `check_state_based_actions` 35,077,285 / 1.94 % over 55,720 |
| `advance_step` inclusive | 278,527,348 / **15.39 %** over 22,892 | was 15.99 % |
| `auto_tap_for_cost_inner` inclusive | 262,553,222 / **14.50 %** over 18,340 | was 14.37 %; candidate (-12) |
| `dispatch_triggers_for_events` inclusive | 124,670,581 / **6.89 %** over 70,418 | was 7.38 %. Self is now ~67 M / 3.7 % across its five `file:function` rows — **the largest engine function left**. 53,838 dispatches get past the empty gate, i.e. **76 %**, which closes (-16)'s open question: most dispatches are *not* empty |
| `gather_continuous_effects_inner` incl | 92,287,090 / **5.10 %** | **53,806 gathers**: `computed_permanent` 32,510, `frozen_effects` 17,702, `compute_permanents` 3,594. Candidate (-18) / (-22) |
| allocator | `_int_free` 69.1 M / 3.82 %, `memcpy` 58.2 / 3.21, `malloc` 50.4 / 2.78, `_int_malloc` 33.3 / 1.84, `free` 31.2 / 1.72 | ~13.4 %. `grow_one` 224,927 calls; **`alloc_zeroed` is now zero** (it was the Ring's `vec![false; 2]`, 53,838 calls) |
| **the checkpoint** | `GameState::clone` self 16,535,448 / 0.91 % | down from 41,569,486 / 2.16 %: 18,208 -> 8,266 checkpoints. The rest of (-13) is those 8,266 |
| `check_state_based_actions` incl | 76,614,637 / 4.23 % | was 5.12 % |
| `card_can_grant_keyword` | 28.3 M / **1.52 %** over 648,698 (43.6 Ir each) | plus `card_keyword_possible`'s own 7.75 M / 0.42 %. Candidate (-11), demoted there |
| `Arc::make_mut` / `clone_from_ref_in` | 19,675,150 / 1.09 % and 15,746,680 / 0.87 % | the CoW tax; callers diffuse |
| `Keyword::eq` | 12,310,142 / 0.68 % | a payload-carrying enum compared by `Vec::contains` all over. No entry yet |

**The forty-second pass reads 1,918,782,724 Ir at its tip, and the
forty-third's second commit takes it to 1,911,862,094.** The table below
was taken at `b1a95b22` (1,928,339,700), two commits earlier; those two remove
`effect_produces_color`'s four redundant tree walks and five of `do_untap`'s
static walks, so every share here reads ~0.7 % high and the
`effect_produces_color` row is gone. What the next run wants from this reading:

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

**(-23) The class the forty-fourth pass's (C) and (D) came from, and it is
not exhausted: an allocation on a path every action takes, for a mechanic the
board does not have.** (C) was `vec![false; 2]` 53,838 times for The Ring
(-1.308 %); (D) was a `Vec` per land tap and a `Vec` per attacker-lock
(-0.227 %). **How to find the next one** — `--tree=caller` on the allocator
entries and read the *call counts*, not the Ir:

| site | calls | Ir | % |
|---|---|---|---|
| `grow_one` (all) | 224,927 | — | 2.19 |
| `finalize_cast` growths | 28,878 | — | 0.49 |
| `Vec::push_mut` growths | 42,354 | — | 0.31 |
| `cast_candidates` collect | 7,238 | 103,486,676 | 5.70 |
| `check_state_based_actions` collects | 55,720 | 34,969,729 | 1.93 |
| `mana_source_table` collect | 7,550 | 41,082,419 | 2.26 |
| `compute_permanent_pass` collects | 95,596 | 8,192,487 | 0.45 |
| `declare_attackers_banded` collects | 33,950 | 8,023,922 | 0.44 |
| `gather_continuous_effects_inner` collects | 79,228 | ~6,500,000 | 0.36 |

A count in the tens of thousands on a six-game workload is one per action or
one per permanent per action; that is the tell. The Ir column lies here — a
`from_iter` row carries the iterator's own body — so **rank by call count and
then read the source**. `alloc_zeroed` is now zero calls; it was the cheapest
possible find and there is only ever one of those.

**(-22) The gather's caller table, kept because (-18) needs it and the
`frozen_effects` half is not (-18)'s.** 53,806 continuous-effect gathers on
the forty-fourth pass's tip at ~1,900 Ir each, ~102 M / 5.5 %:

| caller | Ir | % | gathers |
|---|---|---|---|
| `computed_permanent` | 63,472,587 | 3.42 | 32,510 |
| `frozen_effects` | 31,783,907 | 1.71 | 17,702 |
| `compute_permanents` | 7,255,816 | 0.39 | 3,594 |
| `check_state_based_actions` | 2,992,879 | 0.16 | — |

`gather_continuous_effects_inner` is already lean — one `sa_cards` walk
folding eleven whole-board passes, 645 Ir self — so the win is in the count,
not the gather. **(-18)'s epoch reaches the `computed_permanent` row.** The
**`frozen_effects` row is a different problem and the epoch does not touch
it**: those 17,702 are one gather per freeze *scope*, i.e. every scope that
opens and closes around a handful of reads pays a full one. Widening scopes is
what reaches those — the forty-fourth pass's (B) took 1.02 % from two of them
— but **measure each site**: two more written in the same sitting read
-11,778 Ir together, i.e. nothing, on `--decks fixed`.


**(-21) The bot's attack search is 52 % of the program, and it is the largest
single subtree in the profile.** Never profiled from the top before; these are
the tip's numbers.

* `pick_attacks_scored` **974,200,078 / 52.45 %** inclusive over 438 calls
* -> `simulate_attack_outcome_once` **966,249,395 / 52.02 % over 1,170
  candidates** = **825,854 Ir per candidate**
* -> `sim_step` **536,967,173 / 28.91 % over 35,316** = 15,204 Ir a step, ~30
  steps per candidate. **31,874 of the 36,442 `sim_step`s are `PassPriority`**
  and already take the checkpoint-free path; ~4,568 take `perform_action`.
* -> `pick_blocks` 34,213,150 over 1,508; the per-candidate state clone
  (`sim_start_state` 3,087,194 + `drop_in_place<GameState>` 12,931,434 over
  1,170).
* `attack_candidates_for_mcts` yields `2 + min(attack_search, |greedy|)`
  candidates (+1 for `walker_chip`), all distinct — there is no duplicate to
  memoize, so **the count is a search-quality decision, not a perf one**. What
  is on the table is the cost *per* step, which is the engine, and the ~4,568
  checkpoints, which are (-13)'s other half.

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

**(-18) THE BOARD EPOCH — three candidates, one missing primitive, `4.44 %`
between them, and two of the three are immune to the technique currently
eating the first. Measured at `643330b2` (1,838,526,014) — the absolute Ir
is what matters here; the percentages drift up as later commits shrink the
denominator without touching these rows.**

```text
                                       Ir           %      calls   at 1,857 M
computed_permanent's unscoped gather   50,062,699   2.72 %  25,736    3.42 %
dispatch_board_scan                    24,561,076   1.34 %  53,838    1.32 %
permanents_with_abilities_removed       7,030,646   0.38 %  53,838    0.39 %
                                                    4.44 %
```

**The right-hand column is the same three rows one commit earlier, and it is
the argument for this entry.** Widening a freeze scope (`643330b2`,
-1.024 %) took the gather row from 3.42 % to 2.72 % — 32,510 gathers down to
25,736 — and left `dispatch_board_scan` **byte-identical at 24,561,076** and
`permanents_with_abilities_removed` flat. **Scope widening only reaches the
gather.** Rows two and three are not layer-gather sites at all; they are
whole-board walks with no scope to widen, so they will still be sitting there
at 1.72 % when the scope work runs out.

Every one of them answers a question about the **board**, not about its
caller's arguments, and every one recomputes it from scratch on each call
because nothing readable from `&self` can say "the board has not changed
since last time". `frozen_effects` is the existing partial answer and it
works — inside a `with_frozen_layers` scope the gather is memoized — but a
scope is a *lexical* device and none of these three sites is inside one.
This file has said three times that "a board-level memo with an epoch is the
shape, and nothing on `&self` can hold it today". **That is no longer true,
and here is why.**

**Every board mutation already goes through one chokepoint: `CowBox`'s
`DerefMut`.** The battlefield, `continuous_effects`, `cold` and each
`PlayerData` are `CowBox`es; `iter_mut`, `battlefield_find_mut`, indexing —
all of them reach `Arc::make_mut` through `deref_mut`. So a write counter on
the handle is a complete record of writes:

```rust
pub struct CowBox<T: Clone>(Arc<T>, u64);
fn deref_mut(&mut self) -> &mut T { self.1 += 1; Arc::make_mut(&mut self.0) }
```

The board key is then those counters plus the handful of non-`CowBox` scalars
the layer gather reads (`active_player_idx`, `turn_number`, `step`, and
whatever `active_static`'s wrappers touch — enumerate them, do not guess).
A memo keyed on it, held behind `layer_freeze`'s existing `Mutex` so it is
reachable from `&self`, is sound: an unchanged key proves no zone was
written.

**Cost of the key: one increment inside a function that is already doing
`Arc::make_mut`.** Cost of a miss: nothing, it recomputes.

**The failure mode is the dangerous one — a stale answer is a silently wrong
rules result, not a crash** — so build it with the device this file already
trusts: a `debug_assert` that recomputes and compares on every hit. The
18.7 k-test suite and the golden traces then audit the key on every board
shape they exercise, the same way the forty-third pass's checkpoint skip is
audited by a `debug_assert` on the serialized state.

**Take this after (-13) is finished, and expect it to be the pass after
that.** It subsumes (-11)'s shape (ii), (-16)'s `dispatch_board_scan` half
and (-9)'s "have the gate and the resolver share one scope".

**(-16) `dispatch_triggers_for_events` — 141,288,450 / 7.61 % at the
forty-third pass's tip, and the entry's own open question is now answered.**
`perform_action_inner` drains every action's event list through it; **53,838
dispatches get past the empty-batch early return**. The question this entry
asked was how many of those produce a candidate trigger. **Almost none:
phase 2 (`push_ordered_trigger_candidates`) costs 1,726,302 Ir self and
~2.5 M inclusive over the whole run**, and its per-candidate work runs on the
order of **1,300 times** — so **~97.5 % of full dispatches walk the whole
board and produce nothing.**

Where the 7.61 % sits, measured:
* `dispatch_board_scan` **24,561,076 / 1.32 %** over 53,838 (456 Ir a call) —
  board-only, event-independent, and therefore **(-18)'s**, not this entry's.
* `permanents_with_abilities_removed` **7,160,454 / 0.39 %** over 53,838 —
  same, also **(-18)'s**.
* `event_matches_spec` 2,915,040 over 97,168; `apply_soulbond_pairing`
  1,440,654 over 4,060.
* The rest is the function's own walking, spread across its `slice/iter`,
  `ptr/non_null` and `vec/mod` entries. Its in-body lines are all under 1 M
  Ir — no single line is the cost.

**So the tractable 1.71 % of this entry moved to (-18), and what is left here
is a diffuse walk with no hot line.** Do not gate the battlefield loop on an
`EventKind` presence mask: building the mask is the same
`battlefield x triggered_abilities` walk the loop's fast path already does,
which is the forty-third pass's `do_untap` null result exactly.

**(-17) `check_state_based_actions`' `.collect()`s — 55,720 calls to
`SpecFromIterNested::from_iter` for 35,174,999 Ir / 1.84 %, over 10,670
sweeps.** 5.2 collects per sweep. Most of the collects inside the function are
already behind an `sba_board_scan` flag, so the 5.2 are the *unguarded* ones —
find them before designing. Note the caveat that applies to every row in this
family: a `from_iter` inclusive number contains the whole nested-iterator
body, not just the allocation, so it is a *ceiling* on what a guard could
save. The same caveat applies to the two biggest `from_iter` callers in the
program, `bot::cast_candidates` (**103,318,517 / 5.39 %** over 7,238) and
`auto_tap_for_cost_inner`'s `mana_source_table` (**41,282,703 / 2.16 %** over
7,550) — those are the bot's enumeration cost re-reported, not collect
overhead.

**(-13) `perform_action`'s checkpoint — `drop_in_place<GameState>` 80,831,019
/ 4.21 % plus `GameState::clone` 41,569,486 / 2.16 %, ~6.4 % together, and the
largest *structural* cost left. Two of its three shapes are now measured
dead; read the forty-third pass's Log entry before touching this.**

* **Reuse the checkpoint's buffers — REFUTED, +2.60 %.** Built in full
  (exhaustive `clone_from`, thread-local husk pool, CoW-release on return),
  all invariants green, **-100,388 allocations**, and it still lost by
  49,813,302 Ir. `clone_from` on a 195-field struct costs ~834 Ir more per
  call than `clone` does, because `clone`'s `Self { … }` is one bulk copy of
  175 contiguous `Copy` fields and `clone_from` must interleave a drop and a
  construct per field.
* **Make `GameState` narrower — costed, not worth it.** 175 of the 195
  fields are `Copy`, so they carry no drop glue; a resolution-scratch
  `CowBox` group would take a slice of the 458-Ir `Self { … }` line and
  nothing of the 2,324-Ir drop.
* **Widen the no-checkpoint path — HALF PAID, `-2.842 %`, forty-fourth
  pass.** The whole shape was sized at `-5.47 %` by a probe that skipped
  every checkpoint (**1,810,396,553**, never panicked): the restore fires
  **zero** times in 18,208 checkpointed actions, at **5,750 Ir each** —
  clone 1,194, drop 2,324, and ~2,230 of CoW unshares the action pays only
  because the checkpoint shares its zones (63 % of the drop, **50,619,793 /
  2.64 %**, is `Arc` drop glue freeing exactly those copies). The
  round-closing `PassPriority` — **9,942 actions, 55 % of them** — is now
  skipped alongside the trivial one, for `-54,330,715`. The proof and the
  debug audit that stands in for the missing type-level one are in the
  forty-fourth pass's Log entry.
* **The closure has now been run at the other shapes, and it says no.**
  `scripts/fallibility_closure.py` (committed this pass, and the device that
  made the round-closing pass provable): `play_land` reaches **6**
  `Result` functions and **2** raise — but land drops are ~660 of the 8,266,
  so proving it buys nothing. `submit_decision` reaches **137** and **70**
  raise. A shape with seventy raisers is not proven arm by arm.
* **What is left of it: 8,266 checkpointed actions, ~45 M Ir, ~2.43 %, and
  it is the half where the checkpoint earns its keep.** Casts, activations
  and combat declarations — mana paid then a target rejected,
  `declare_attackers` failing mid-loop — i.e. the partial-mutation family
  Phase 1 exists for. `declare_blockers` and `declare_attackers_banded`
  hold 82 of the engine's `Err` sites between them. A per-arm proof here
  has to read as well as the pass one did; measuring zero failures on one
  workload does not.
* **The count, for whoever sizes it.** `perform_action` ran 18,208x on this
  workload before the pass and 8,266x after, against
  `perform_action_inner`'s 70,418; the bot's probes (`would_accept`,
  `sim_step`) already take the un-checkpointed path, so those are real
  actions.

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

**(-11) `card_can_grant_keyword` — re-costed at the forty-fourth pass's tip
and DEMOTED: 28.3 M / 1.52 % over 648,698 calls (43.6 Ir each), plus
`card_keyword_possible`'s own 7.75 M / 0.42 %.** The per-definition presence
bit (shape (i) below) is worth ~10-15 Ir on the *vanilla* cards only — the
ones whose five containers are all empty — so ~0.3 %, and it cannot be a
lazily-cached field: ~20 sites in `effects/mod.rs` mutate a definition in
place through `Arc::make_mut(&mut c.definition)` (keywords pushed, statics
added), so a cached bit goes stale in the unsound direction. Making it safe
needs a `CowBox`-style handle on `CardData::definition` so `Arc::make_mut`
stops compiling — real churn in `crabomination_base` for 0.3 %.
`CardDefinition` does **not** derive `PartialEq`, so the older note's reason
was wrong; the reason it is still a bad trade is the mutation sites.

**(-11, original note) Second correction, and it points somewhere new.** The fortieth pass's note
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

**(-10) Allocation is still the program: 1,216,241 allocations and 1,269,976
frees in six games, ~13 % of the tip — and the forty-third pass measured what
drives a large slice of it.** `Arc::make_mut` unshares **120,004 times for
83,959,478 Ir / 4.38 %** (~700 Ir each), and 63 % of the checkpoint's drop is
the `Arc` glue freeing those same copies. **Most of that exists because the
checkpoint shares every zone**, so (-13) and this entry are the same cost seen
from two sides — take (-13) first. The per-caller breakdown below is from the
forty-second pass's tip and the counts have moved slightly; the shapes have
not. The forty-second pass took 61 k
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

**(-15) `advance_step` — 308,637,209 / 16.09 % over 22,892, the largest engine
subtree. Both of the forty-second pass's leads into it are now closed and it
needs re-profiling from the top.**
* **The `do_untap` gate — two implementations, two answers, and the second one
  PAID `-0.168 %`** (forty-third pass, rows in the Log). Gating the *walks*
  read +0.0001 % and was reverted: a short-circuiting `any` over an empty
  `static_abilities` is nearly free. Gating the *blocks* that build
  collections around them — `filtered_untap`'s `Vec` + `HashSet`,
  `matrix_choice`'s `HashMap`, `untap_caps`'s `flat_map` — landed.
  `do_untap`'s remaining ~34 M is ~9 M of self across every `file:function`
  entry, 5.1 M of `do_phasing`, and ~20 M in callees nobody has attributed.
  **Read the callee table first.**
* **`do_cleanup` — 27,454,488 / 1.43 %, read, and there is no easy win.**
  `finish_cleanup` is 24,937,836 of it and `check_state_based_actions` is
  **16,643,338 of that** — the CR 514.3a sweep, which is real work.
  `cleanup_wear_off` is the remaining ~5 M over 1,764 calls and its ~60
  `clear_cold!`s are already guarded.

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
