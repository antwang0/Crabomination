# PERF

Numbers, not prose. Every perf change gets a row in **Log** with what
changed, the before/after, and how it was measured. No measured win means
revert — or keep it and say plainly that it's a correctness/clarity change.

Benchmarks and profiles run on optimized builds (see CLAUDE.md's carve-out).
A number from a debug build describes `opt-level = 0`, not the code.

## How to measure

```text
# throughput — the committed configuration. The header names the profile the
# binary was built under (read off `target/<profile>/`), so a `release-fast`
# or `profiling-fast` reading cannot be filed as a `release` one by mistake.
cargo run --release --bin bot_ladder -- --bench

# determinism across thread counts (opt-in; doubles the run, so off the
# throughput reading above). Replays the identical --bench workload at a
# contrasting thread count and asserts the order-independent outcome matches
# — the aggregate is a sum over seed-fixed jobs, so it must. Clean at the
# pass-52 tip: `thread_determinism ok (3 vs 1 threads identical)`. See TODO
# filter 23 (`1c304384`).
CRAB_THREAD_CHECK=1 cargo run --release --bin bot_ladder -- --bench

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
# SYMBOLS: **check before you symbolize.** At the fifty-eighth pass the dump
# came back fully symbolized on its own — `ob=` names the binary, `fn=` lines
# carry demangled Rust names and `fi=` lines carry source paths — so
# `cg_symbolize.py` was a no-op and its bias auto-detect (43 unresolved libc
# addresses, 2 of them inside a FUNC) printed a scary-looking line for
# nothing. Run `cg_edges.py` on the raw dump first; symbolize only if the rows
# come out as addresses. The note this replaces, kept because the failure is
# real when it happens:
# valgrind 3.22 in this image did **not** read bot_ladder's symbol
# table — `valgrind -v` never prints "Reading syms from …/bot_ladder", so
# every engine frame comes out `???:0x…`. It is not the copy-the-binary
# hazard the older note here blamed, and not `split-debuginfo`, size, or lld
# (all three were tested at the forty-eighth pass and all three symbolize
# fine). Put the names back before annotating; the addresses are ELF vaddrs
# plus valgrind's PIE base 0x108000, so the symbol table resolves them.
python3 scripts/cg_symbolize.py cg.out target/profiling-fast/bot_ladder \
  > cg.sym.out
callgrind_annotate --auto=no --threshold=95 cg.sym.out        # self cost
callgrind_annotate --auto=no --inclusive=yes cg.sym.out       # inclusive
# `--auto=yes` per-line annotation stays dead (DWARF lives in `.dwo` files
# valgrind can't read); read function totals and call counts instead.
#
# what an *inlined* function costs, and where. `cg_edges.py` ranks functions
# and `cg_lines.py` ranks lines; neither sees a small always-inlined function,
# which has no row and no line of its own — `battlefield_find` is 556 call
# sites and was 4.03 % of the simulator, unread for fifty-two passes.
python3 scripts/cg_sites.py cg.instr.out target-probe/profiling-lines/bot_ladder \
    battlefield_find
# Its number is a FLOOR: each address is charged only its own instructions,
# so a scan's per-element loads land in `slice::iter`'s frames instead. The
# two sites it found at 0.35 % between them measured -0.611 % when removed.

# whose calls those are, three frames up. A one-level caller table ranks by
# the immediate caller, which for `gather_continuous_effects_inner` is
# `computed_permanent` and says nothing. `--separate-callers=N` gives one
# entry per calling context; `cg_contexts.py` sums them. It costs no run time
# and roughly doubles the dump. The fifty-fourth pass spent a build and a
# callgrind run on a scope that removed zero gathers before adding this.
RUST_MIN_STACK=33554432 valgrind --tool=callgrind --separate-callers=3 \
  --callgrind-out-file=cg.sc.out target-probe/profiling-fast/bot_ladder \
  --a gang --b gang --games 6 --threads 1 --seed 1 --decks fixed
python3 scripts/cg_symbolize.py cg.sc.out \
  target-probe/profiling-fast/bot_ladder > cg.sc.sym.out
python3 scripts/cg_contexts.py cg.sc.sym.out gather_continuous_effects_inner

# For any caller/callee table, use `cg_edges.py` rather than
# `--tree`: `callgrind_annotate --tree` truncates a caller list at its
# threshold and silently drops rows. Its `__rust_alloc` block printed 23 k of
# the program's 967 k allocations and omitted `finish_grow` (200,972) and
# `finalize_cast` (24,108) entirely.
python3 scripts/cg_edges.py cg.sym.out                        # self costs
python3 scripts/cg_edges.py cg.sym.out --callers __rust_alloc # the alloc table
python3 scripts/cg_edges.py cg.sym.out --callees finalize_cast
python3 scripts/cg_edges.py cg.sym.out --callers __rust_alloc --rows 0  # all
# **A listing that caps its rows says what it dropped, and `--rows N` (`0` =
# no cap) lifts the cap.** Until the fiftieth pass neither was true, under a
# docstring that promised a *complete* table — the `--tree` truncation above
# wearing the fix's clothes. Nineteenth robustness filter; see TODO.
# Per-source-line attribution needs the DWARF *packed into the binary*, which
# is what `[profile.profiling-lines]` is for (cold build; it reads the same
# total, 1,659,704,679 vs profiling-fast's 1,659,704,666, so the two inline
# identically). Costs are self cost per line.
cargo build --profile profiling-lines -p crabomination --bin bot_ladder \
  --no-default-features
RUST_MIN_STACK=33554432 valgrind --tool=callgrind --dump-instr=yes \
  --callgrind-out-file=cg.instr.out target/profiling-lines/bot_ladder \
  --a gang --b gang --games 6 --threads 1 --seed 1 --decks fixed
python3 scripts/cg_lines.py cg.instr.out target/profiling-lines/bot_ladder
python3 scripts/cg_lines.py cg.instr.out target/profiling-lines/bot_ladder \
  --in dispatch_triggers_for_events
# **The line profile was wrong until the fiftieth pass, and it was wrong in a
# way that looked right.** `cg_lines.py` summed the instruction addresses of
# *every* object the process mapped — libc, ld.so, libm, libgcc, valgrind's
# preloads, 16.5 % of the run — and resolved them against this binary's DWARF,
# and it hardcoded a `0x108000` load bias where the `profiling-lines` binary
# needs `0`. 36 % of the run came out `??` and the rest was attributed to
# whichever Rust symbol sat at the wrong offset: `Effect::clone` read
# **35,279,138 / 2.65 %** where its own call edges are 2,890 calls and
# **0.5 M**. The forty-eighth pass's `drift::sort` row, blamed here on lld's
# identical-code folding, is more likely the same bug. It now keeps only the
# annotated binary's object, auto-detects the bias, prints the hit rate
# (400/400 at bias 0) and refuses below half — including when you hand it a
# dump taken from a *different* binary, which used to annotate happily.
# **Cross-check any line-profile row against `cg_edges.py`'s call counts
# before ranking work by it.** The counts are the truth; the lines are a
# pointer to where inside a function the cost sits, never to a function total.
# Every listing `cg_edges.py` prints says what it truncated (the nineteenth
# robustness filter) — and the first thing that reports is that **the top 45
# self-cost rows are 68.5 % of the program and 1,150 rows hold the rest**. A
# profile that diffuse is why the forty-ninth pass's wins came from counting
# call rows, not from ranking by self cost.
# Callers of `__rust_alloc`, ranked by *call count*, is the table that has
# found the most: self cost lies about allocation — a function with 1.9 %
# self can be 35 % of every malloc in the program. `cg_edges.py`'s "total Ir"
# line used to double-count (it summed the inclusive edge costs too) and the
# forty-ninth pass fixed it: the total is now the self lines only, it
# cross-checks the dump's own `totals:` line, and it prints a WARNING when the
# two disagree. Before that fix the *shares* in every table it printed came
# off the inflated total and read ~18x low.

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

**A six-game callgrind run charges one whole-catalog build to the game loop,
and on `sos` that is 6.8 % of the total.** `card_registry::name_index()`
builds its `OnceLock` by *calling all 22,568 catalog factories* — a full
`CardDefinition` built and dropped apiece — to read their names. It is one
`OnceLock::initialize` edge, **104,687,400 Ir**, hanging off whichever
`lookup_by_name` happens to run first.

Which pool pays it, read off the dumps at the sixty-first tip
(`cg_edges.py --callees lookup_by_name`):

| pool | `lookup_by_name` calls | index build |
|---|---|---|
| sos | 85 | 104,687,400 (**6.8 %** of the run) |
| cube | 0 | — |
| fixed | 0 | — |

So the three pools' totals are **not comparable to each other** at the
hundred-million level, and an `sos` total sits ~104 M above what its game
loop costs. It is not a deck-building path — `sos` reaches it from
`apply_pending_effect_answer`, validating the name a decider returned to a
`NameCard` decision — so *whether* a pool pays it is "did a card that names
a card resolve", not anything about the pool's construction.

**Never rank a candidate against a share of an `sos` six-game total without
subtracting it.** A change worth 50 M Ir reads 3.2 % of the run and is 3.5 %
of the simulator. The sixty-first pass found this by asking why
`lookup_by_name` had a hundred-million-Ir inclusive row on one call.

**And then it does not matter.** It is one-time per process, so a training
actor playing thousands of games amortizes it to ~0.001 %, and a test binary
that resolves any name pays it once — tens of milliseconds against a 27 s
suite. Recorded
as candidate (-46) at that honest size — a measurement artefact first and a
throughput item barely at all — so that nobody reads the 6.8 % as a
simulator cost and spends a pass on it.

**When you do want a clock number, use `scripts/ab_wall.py` and run its null
control.** It is the loop every pass has hand-rolled — alternate two binaries,
quote best-of — with the two things that loop was missing. It runs an **ABBA**
schedule, so a linear host drift cancels inside a block instead of landing on
whichever binary went first; and it reports the **mean of the per-block ratios
with a 95 % t confidence interval**, so the answer arrives with the effect size
the sample can distinguish. It also fingerprints both runs (`decisions`, the
decided/undecided split) and refuses to report a timing when they differ.

```text
python3 scripts/ab_wall.py --bin-a /tmp/base/bot_ladder \
    --bin-b target/release-fast/bot_ladder --blocks 8 \
    -- --a gang --b gang --games 2000 --decks sos --seed 11 --threads 4
python3 scripts/ab_wall.py --bin-a X --bin-b X --blocks 8 -- <same workload>
```

**Best-of is a biased estimator and this file has been quoting it.** It
compares two extreme order statistics of an unknown distribution, so with a
7-9 % within-binary spread it mostly reports which side caught the quiet
minute. `cae6b605` read **+2.5 % slower** by best-of over nine hand-rolled
pairs, **+1.26 % slower with a `+/-0.67 %` half-range** at four ABBA blocks —
and **flat** at eight, where the null control is equally flat:

```text
--games 2000 --decks sos --threads 4, ~30 s a run, Xeon @ 2.10GHz, 4 cores
8 blocks, base vs tip    mean +0.18 %   CI -1.64 .. +2.00 %   FLAT
8 blocks, null control   mean -0.40 %   CI -2.45 .. +1.66 %   FLAT
```

**The box resolves +/-2 % and nothing finer, for thirty-two runs and sixteen
minutes a side.** Four blocks is not enough — it called a null-equivalent
result significant. That number is the reason the rule below holds, and it is
now measured rather than asserted.

**And with the harness in hand, the obvious question got asked for the first
time: does the Ir show up on the clock?** Three passes' worth — base
`28ae2416` (pass 56's tip) against `49c7220d`, i.e. passes 57, 58 and 59
together — measured both ways on the same box, eight ABBA blocks a pool, with
the null control run at the same block count and workload and coming back
flat:

```text
pool     Ir base          Ir tip           Ir        clock (8 blocks)     blocks
sos      1,715,661,899    1,603,018,915    -6.57 %   -3.87 %  [-5.2,-2.5]   8/8
cube     3,162,425,896    2,880,726,915    -8.91 %   -3.16 %  [-5.2,-1.1]   7/8
fixed    1,226,171,101    1,219,893,702    -0.51 %   not run
```

**The method works, and Ir over-reads the clock by about 1.7x on `sos` and
2.8x on `cube`.** Both readings are outside the +/-2 % floor and the same
direction as the Ir, so ranking work by callgrind is sound — it is
deterministic, thirty times cheaper to collect, and it found every one of
these commits. What it is *not* is a throughput number. **Halve an Ir delta
before quoting it as games/sec**, and treat a pass under ~3 % of Ir as
unseparable on this box's clock at eight blocks — which is most single
commits, and is why the per-commit rows in the Log stay in Ir.

The `cube` ratio being the worse of the two is consistent with what the two
pools carry: cube's share of the arc is the trigger dispatcher's grant walk
and thirty-eight `sa_cards` walks, i.e. cheap predictable instructions at high
IPC, while `sos` also carries `cae6b605`'s allocations and the CoW unshares.

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

## Which pool a change moves — read this before ranking anything

**Added at the fifty-third pass, which found a 32 % row and a 46 % row that
`--bench` cannot see at all.** Every Log row before that pass measures
`--decks fixed`: four hand-built vanilla archetypes. That pool is a fine
*game-loop* proxy — the sealed-game profile below is the same shape to
within a point — but it is blind in two directions, and both of them are
where the simulator's largest costs turned out to be.

**1. `fixed` carries no `GrantTriggeredAbility` static.** So `no_grants` is
true on every board, `statics_granted_triggers_with` is never called, and
the whole per-card grant path — which evaluates a `SelectionRequirement`
against every battlefield permanent and reads the *computed* type line to
do it — is dead code on the bench. On `--decks cube` that path reached
**59.6 % of the program** and `gather_continuous_effects_inner` was
**32.51 %** against `fixed`'s 4.12 %. Three freeze scopes took the cube
pool from 7.95 G to 4.05 G (**-49.1 %**) and moved `fixed` by +0.11 %.

**2. `--bench` builds its decks once; a training actor builds two per
game.** `bot_ladder --decks sealed --games 1` plays *no games* (paired needs
two per archetype) and still ran **2,910,408,580 Ir** — twelve sealed pools
and twelve heuristic builds at 242.5 M apiece, against **48.4 M for an
actual sealed game**. `selfplay_train`'s `actor_loop` calls `sealed_pool`
twice and `build` twice *per game*, so that was ~485 M Ir of deck building
per 48 M Ir of simulation. The bench amortises it over 80 games an
archetype and never sees it.

**And once you have two pools, rank the rows by the RATIO of their shares,
not by either share alone. That is a device, it is one script, and it found
the sixty-second pass's second commit.** A row that is 0.61 % of `cube` is
nowhere near the top of any table and nobody would look at it. The same row
is 0.12 % of `sos` — **5.08x** — and *that* is a pointer: whatever it does,
the grant-heavy pool does five times more of it per instruction, so the work
is pool-specific and structural rather than diffuse. Dump both pools at one
tip, parse `cg_edges.py <dump> --rows 0` into `{row: share}` for each, and
sort by `cube% / sos%` over the rows above ~0.45 % of cube:

```text
cube%   sos%     x    row                                    (sixty-second tip)
 0.61   0.12  5.08    layers::affected_includes_gated        <- taken, -28 % of itself
 0.90   0.43  2.09    bot::pick_blocks_inner
 0.81   0.44  1.84    CardInstance::has_keyword              <- 494,394 calls, flat, no
 1.03   0.56  1.84    evaluate_requirement_static_hinted
 1.38   1.04  1.33    card_can_grant_keyword                 <- (-11), demoted
```

**Use `--rows 0` on both sides or the ratio lies.** A default listing
truncates, so a row present in one dump and merely *below the cutoff* in the
other reads as an infinite ratio. Five rows came out `inf` on the first
attempt here and all five were truncation, not pool-specific work.

**The ratio is a pointer, not a size.** Confirm with the Ir/call read before
writing anything: `affected_includes_gated` was 236,026 calls at 71 Ir of
*self* each with only 0.6 M in its callees, which says the cost is inlined
predicate work inside the function — takeable. `has_keyword` right below it
is 494,394 calls at a flat ~46 Ir across every caller, which says diffuse —
not takeable, and the ratio alone could not tell them apart.

**The rule.** A change to statics, grants, layers or the requirement walker
gets a `--decks cube` reading as well as a `fixed` one. A change anywhere in
`draft.rs` / `recommend.rs` / `selfplay.rs` gets `--decks sealed --games 1`,
which isolates deck construction exactly. `fixed` stays the committed bench
because it is reproducible, cheap and *is* representative of the game loop
— it is the pool the Log's absolutes are comparable across — but it is not
the whole simulator.

```text
# the four pools, same config, at the fifty-third tip
for d in fixed cube sos sealed; do
  RUST_MIN_STACK=33554432 valgrind --tool=callgrind --callgrind-out-file=cg.$d.out \
    target/profiling-fast/bot_ladder --a gang --b gang --games 6 --threads 1 \
    --seed 1 --decks $d
done
# deck construction alone (0 games played, all setup):
target/profiling-fast/bot_ladder --a gang --b gang --games 1 --threads 1 \
  --seed 1 --decks sealed
# the training loop itself, shipped allocator, for a wall-clock number:
cargo build --profile release-fast -p crabomination_ml --bin selfplay_train
target/release-fast/selfplay_train --actors 3 --games 120 --steps 1 --seed 7 --out /tmp/x
```

**And the one that has to be a wall-clock number.** The deck-builder fix is
allocation-shaped, and callgrind runs the system allocator, so its Ir
overstates what ships. Measured on the real loop with mimalloc,
`--actors 3 --games 120 --steps 1 --seed 7`, alternated A/B/A/B:
**26.1 / 25.0 games/s before, 85.6 / 85.6 after — 3.28x.**

**How to actually run it, characterised at the fifty-eighth pass, because
two of the three obvious instincts are wrong.**

* **Always discard the first run.** It reads ~45 % low, every time, on an
  otherwise idle box: 66.6 then 119.8, 119.8. Four separate batches this
  pass each opened with a low outlier (80.0, 59.9, 74.9, 66.6) and it is
  what a single-run reading will hand you.
* **Warm, the committed 120-game recipe is stable to 0.1 %** — six
  consecutive runs read 119.9 / 119.9 / 119.9 / 119.8 / 119.9 / 119.8. It is
  a *good* bench, which is not obvious from the fact that it spans one
  second.
* **A longer run is noisier, not quieter.** 1,200 games reads 104-115
  (~10 % spread) and 3,000 games 109.3-114.5 (~5 %), against 120 games'
  0.1 %. Raising the game count to fight noise makes it worse here, so
  don't; the learner thread and the 250 k row window start participating.
  The absolutes are also not comparable across game counts, so a baseline
  number only means something at a fixed one.
* **Nothing else may be running.** The 1,200-game batch read
  68.1/89.2/90.5/83.2 with a leftover build finishing, and 104-115 with the
  box idle. Check `/proc/loadavg` before quoting.

**The 85.6 above is not comparable to a reading taken now**: the same recipe
on this container reads **119.8 games/s** at the fifty-eighth tip. That is
container and accumulated-pass drift, not one pass's win — quote it as a
same-session A/B or not at all.

**And the 0.1 % above is repeatability, not resolution — do not read it as a
contradiction of `scripts/ab_wall.py`'s 6.5 % spread, and do not use it to
justify quoting best-of.** Six consecutive runs of one binary inside one
quiet batch agreeing to 0.1 % says the recipe is not self-noisy; it says
nothing about whether *two* binaries measured minutes apart can be told
apart, which is the question an A/B asks and which only `ab_wall.py`'s ABBA
blocks and null control answer. The two figures also measure different
things: 0.1 % is `selfplay_train`, 3 actors, one-second runs; the 6.5 % is
`bot_ladder`, 4 threads, ~30-60 s runs. **Use these bullets to take a single
honest absolute; use `ab_wall.py` for any comparison between two binaries.**

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

**And the binary count had seven executables in it that run nothing, found
2026-08-25.** The section above is right that the relink is the lever and that
the count is the thing to keep down — and nobody had counted. `cargo test`
builds a *test harness* per `[[bin]]` on top of the normal binary, so the nine
bins under `crabomination/src/bin/` are nine of the twenty executables it
relinks; **seven of them carry no `#[cfg(test)]` block at all.** `test = false`
on those seven, `crabomination/Cargo.toml`:

```text
touch game/effects/mod.rs; cargo test -p crabomination -p crabomination_tests --no-run
  before   24.99 / 26.18 / 24.47 s              (mean 25.21)   20 executables
  -7 bin   21.87 / 21.95 / 21.23                (mean 21.68)   13   -14.0 %
  -1 int   18.37 / 21.56 / 19.05 / 20.40 /
           18.65 / 17.65                        (mean 19.28)   12   -23.5 %
suite      18,728 passed / 0 failed / 5 ignored — unchanged throughout: the
           seven harnesses held no tests, and the integration binary's
           seventeen moved into `core_rules`
```

The twelfth executable came off with `crabomination/tests/card_instance.rs`,
a top-level integration test *in the engine crate* holding seventeen tests
that use only public API (`CardInstance::new`, `catalog::`). It is a module of
`crabomination_tests`' `core_rules` binary now, which is where CLAUDE.md says
it belongs; `crabomination/tests/` is gone.

**It is the link and only the link.** One of the two harnesses that stays
(`replay_view`, two tests) starts and finishes in **3.5 ms**, so the catalog's
`ctor` registration is not eager and there was no run-time cost to remove.
`audit_stubs` and `replay_view` keep their harnesses: four real tests over
bin-local helpers (`classify` / `def_has_any_ability`, `narrate`), and moving
them would mean moving the helpers into the library to be tested, which is a
worse trade than one link.

**Where the rest of it goes, `cargo build --timings` at the twelve-executable
tip** (four cores, 69 s of CPU over ~19 s of wall): `crabomination` lib
**10.00 s in test mode** on top of **7.54 s** normal — the engine compiled
twice, and the 553 unit tests that second compile exists for are
`#[cfg(test)] mod tests` blocks in thirty files, mostly `server/` internals
(`available_mana`, `mcts`, `encode`) that are private and cannot move out.
Then the eight integration binaries, 37.5 s of CPU between them
(`classic_sets` 9.29, `modern` 6.23, `core_rules` 5.31), and the nine bins'
*normal* builds at ~11.5 s, which `cargo test` does whether or not their
harnesses are on — there is no cargo flag to skip them short of
`--lib --tests`, which would also drop `audit_stubs`' and `replay_view`'s four
tests. **Nothing left here is worth a risk.**

**The standing rule gains a clause**, now in CLAUDE.md: a new `[[bin]]` with
no `#[cfg(test)]` block gets `test = false`.

Not measured here, and still open: the `release` / `profiling-fast` rebuild
(~13 min cold for the engine) is a codegen-bound build where CGU partitioning,
not query invalidation, decides the cost. Nothing above says anything about
it.

**The probe loop, measured at the forty-sixth pass, because it changes
iteration by 4x.** A cold whole-workspace `profiling-fast` build of
`bot_ladder` is **11m00s**, and the catalog crate (619 k lines over 708 files)
is most of it; an **engine-only** rebuild is **3m15s**. A
`rm target-probe/profiling-fast/{bot_ladder,deps/*crabomination*}` glob
matches `crabomination_base` and `crabomination_catalog` too and so forces the
11-minute path every time — delete
`deps/{crabomination-*,libcrabomination-*,bot_ladder-*}` instead. `cargo check
-p crabomination` is 2m01s cold and seconds warm; run it before every probe
build.

## Baseline

**Sixty-first pass (the other session's line, run concurrently with 59 and
60). Base `ba15f249` vs this block's tip.** Four commits, one class and one
defect: **four questions the simulator asked with a walk or an iterator chain
it did not need to build.** All three pools move, which is the difference
from pass 58 — nothing here is on the deck builder. Ir readings
`profiling-fast --no-default-features`, callgrind, one thread, `--a gang --b
gang --games 6 --threads 1 --seed 1`.

**`ba15f249` is the base both columns were measured at, and pass 60's
`6344adf6` is in neither.** That commit (the deck-fill `CardDefinition`
memcpy, -2.9 % on `sos` by itself) landed underneath during the rebase that
brought this block onto the branch; it is on `cube::card_arc` and the deck
fill, which none of the four commits here touch, so the deltas stand — but
**do not diff the current tip against the current base and expect these
numbers.** Name the base, as this file has had to say four times.

```text
                          base (ba15f249)   tip (this block's)
I refs, --decks fixed     1,218,195,816    1,206,204,087   -0.984 %
I refs, --decks sos       1,593,831,453    1,580,084,804   -0.862 %
I refs, --decks cube      2,866,729,876    2,841,539,263   -0.879 %
```

**One caveat added after the fact, and it makes these numbers conservative:**
the sixty-second pass found that `name_index()` builds 22,568
`CardDefinition`s at startup — **104.7 M Ir, 6.8 % of a six-game `sos`
total** (see (-46)). Both columns above carry it, so the deltas are sound,
but the *shares* are diluted: net of startup, `sos` reads -0.923 % rather
than -0.862 %.

**The base column is a cross-check as well as a base.** `ba15f249`'s own
commit message reads `fixed` 1,218,193,228 and `sos` 1,593,828,683 for the
same commit, measured by the other session; this reading is 2,588 and 2,770
Ir above them, which is argv length and nothing else (see pass 49's note).
Two sessions, two containers, two build directories, the same binary
behaviour to five digits.

| step | commit | fixed | sos | what |
|---|---|---|---|---|
| A | `e0e64d12` | **-0.262 %** | -0.205 % | the combat-damage dispatch's `AnyPlayer` leg `collect()`ed an always-empty `Vec` |
| B | `7f8f94d2` | +0.012 % | +0.014 % | **defect**: converge had two oracles, disagreeing in both directions |
| C | `2336817d` | **-0.641 %** | **-0.564 %** | three whole-battlefield / command-zone walks at the top of every SBA sweep |
| D | `c676a229` | -0.095 % | -0.100 % | C's `sculptor` bit folded into the keyword walk the scan already does |

**The per-step column was measured on this session's own line, before the
rebase**, i.e. on `ff929e7f` plus a fifth commit of this session's that the
rebase dropped (below). The three-pool pair above was re-read at the rebased
base and tip, and it is the number to quote — but the two agree: the steps
sum to **-0.986 % on `fixed`** against the pair's -0.984 %, and **-0.855 %
on `sos`** against -0.862 %. The concurrent session's commits underneath are
on different code, and this is the check that says so.

**And the fifth commit is the reason to read this block before starting
anything: two sessions found `loop_fingerprint` in the same hour, and the
other one was right.** Both saw that the CR 104.4b watchdog's digest ran
`DefaultHasher` (SipHash-1-3) over ~84 small integers per call, 0.78 % of a
six-game `sos` run. This session replaced it with the engine's vendored
`FxHasher` and measured `sos` -0.706 %; the other replaced it with
**SplitMix64's finalizer** (`ba15f249`) for -0.578 % on its own base — and
its argument is the correct one. `fxhash`'s own doc says it is not
collision-resistant, and it exists for *map iteration determinism*; this
digest decides a **draw**, where the function's comment says a false positive
ends a live game. A cheaper hash with a weaker avalanche is the wrong trade
there even though the Ir was better. **This session's commit was dropped in
the rebase.** The rule, and it is new: when two sessions land the same
finding, keep the one whose *argument* survives, not the one whose number is
larger.

**B is a bug fix that costs 0.01 % and the number is quoted so nobody has to
re-derive it.** `CardDefinition::wants_converge` scanned the definition's
Debug rendering for `ConvergedValue` and missed converge's other spelling,
`SelectionRequirement::ManaValueAtMostConverged` — so **Bring to Light** and
**Sundering Archaic**, whose converge is entirely in a target filter, were
paid for with the mana-conserving order that the function's own doc says
"routinely counted one color on a five-color board". Measured directly rather
than by row attribution: on the pre-rebase line a base binary carrying only
this commit read `fixed` 1,220,954,533 against 1,220,812,183, so it is
**+142,350 Ir, +0.0117 %** — the second substring scan, once per card *name*
per process, amortized to nothing over a training run.

**The rows that moved, tip against base (pre-rebase line, `fixed` / `sos`):**

```text
                                 fixed                     sos
check_state_based_actions  28,960,956 -> 24,033,162   43,928,234 -> 37,493,708
Vec::from_iter (all monos) 37,336,782 -> 33,571,904   36,400,225 -> 31,767,323
Map::try_fold               6,167,108 ->  2,495,168    (same shape)
sba_board_scan             20,935,578 -> 20,927,838   28,013,796 -> 28,032,252
```

`sba_board_scan` is flat: the four bits C added (`shapeshifter`,
`sector_set`, `sculptor`, and D's fold of the last one) cost what the walks
they replaced cost per card, and the win is the three walks that stopped
happening at all. `Vec::from_iter` attributed to
`check_state_based_actions` went **53,362 calls / 21,358,086 Ir to 13,576 /
15,880,576** on `sos` — 4.02 collects a sweep down to 1.02.

**One thing was built, measured and reverted, and it is the pass's second
finding.** Serving `card_type_change_unscoped`'s battlefield leg off
`sba_board_scan` reads **+0.295 % on `fixed` / +0.255 % on `sos`**:
`card_type_change_unscoped` comes off in full (-5,948,694 on `sos`) and
`sba_board_scan` goes up **9,249,352** to pay for it. The standalone
`any(card_can_change_card_types)` short-circuits per card; a scan bit has to
run `static_effect_changes_card_types` over every static ability of every
permanent whether or not the answer is already known. **That is the third
refutation of the (-6) fusion device inside `creature_death_possible`
alone** (+0.55 % and +1.24 % are the other two, already in the code
comment). The rule it yields: *a presence bit belongs in a shared scan only
when the question has no early exit of its own.*

**And the shape all the wins share, which is the one to carry forward: ask
what the answer costs when it is "no".** None of these was a hot function.
A `filter`/`flat_map`/`filter`/`map` stack over an empty result, a
`flat_map` over two empty command zones, a battlefield `filter` for a card
nobody's deck contains — and, both sessions independently, a SipHash of ~84
small integers for a digest that is only compared with itself. Every one is
the cost of *asking*, paid on every sweep or every dispatch, on a board
where the answer is always no. `cg_edges.py --callers SpecFromIterNested`
found two of them in one table: rank the collect sites by *calls*, then ask
which of them can return non-empty on the bench pools.

```text
decisions        196,220 -> 196,220        byte-identical
turns_per_game   27.53   -> 27.53
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0
determinism      ok (all pairs split); thread_determinism ok (3 vs 1 identical)
peak_rss_mib     30.0 / 30.1 / 30.1 over three `--bench` runs
suite            18,736 passed / 0 failed / 5 ignored over 14 test binaries
                 (11 of them carry tests), at every commit
golden traces    7 passed, all unchanged (every commit)
clippy           `--workspace --all-targets` clean, all eight crates
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.80GHz, 4 cores, host_calib_ms 52-60
```

**No wall-clock pair is quoted, and the concurrent session's Ir-vs-clock
measurement is why.** That reading (see "How to measure") puts Ir at ~1.7x
the clock on `sos` and ~2.8x on `cube`, so these four commits are worth
roughly **-0.5 % of wall time on `sos` and -0.3 % on `cube`** — a quarter of
the +/-2 % this box resolves at eight ABBA blocks, and exactly the "under
~3 % of Ir is unseparable" case that note describes. The Ir column is the
attribution. None of the four is allocation-shaped —
`malloc`/`free`/`_int_malloc` are flat to five digits on both pools — so
there is no "Ir counts a memcpy, the machine barely does" discount on top:
what came off is iterator-adapter frames and battlefield loads.

**Crash-freedom and determinism at the tip, widest pool.** `release`, `--a
gang --b gang --games 200 --threads 3 --decks all`, seeds 11 / 12 / 13:
**10,200 games, 10,200 decided, no panic**, and all 5,100 mirrored pairs
split (`rho -1.000` on every seed). `--decks sealed --games 200 --seed 11`:
**2,400 decided, 0 undecided**. `CRAB_THREAD_CHECK=1 --bench` reads
`thread_determinism ok (3 vs 1 threads identical)`.

**The `release` throughput reading, for the record and not as a claim.**
Three `--bench` runs at the rebased tip: **171.72 / 167.88 / 167.94
games/s**, `decisions_per_s` 105,299 / ~102,900, at `host_calib_ms` 58 / 54 /
64. (The pre-rebase tip read 162.69 / 162.72 / 167.98 at calib 52 / 60 / 56,
two hours earlier on the same container — which is the spread this file keeps
warning about, not a result.) No base binary was built at `release` in this
sitting, so there is no pair, and the standing rule is that a cross-sitting
`games_per_s` difference is not evidence. What the run attests is the
invariants above it.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState`
or `Vocab` change is in this pass.

**Sixtieth pass, base `58346b57` vs tip `6344adf6`, two commits, both about
a copy nobody needed.** `--decks sos` **-3.46 %**, `cube` **-2.82 %**,
`fixed` **-2.16 %**, `sealed` **-2.33 %**, and **peak RSS on `--bench`
21.9 -> 17.7 MiB, -19 %**. Ir readings `profiling-fast
--no-default-features`, callgrind, one thread, `--a gang --b gang --games 6
--seed 1`.

```text
                       base (58346b57)   (A) ba15f249    tip (B) 6344adf6
I refs, --decks sos      1,603,088,243   1,593,828,683   1,547,629,927  -3.460 %
I refs, --decks cube     2,878,150,309   2,866,725,942   2,797,004,247  -2.820 %
I refs, --decks fixed    1,219,893,985   1,218,193,228   1,193,591,244  -2.156 %
I refs, --decks sealed     3,276,783,848 (read at (A))   3,200,503,032  -2.328 %
deck build alone              21,800,071 (read at (A))      21,871,968  +0.330 %
peak_rss_mib, --bench               21.9                          17.7    -19 %
```

| step | commit | sos | cube | fixed | what |
|---|---|---|---|---|---|
| A | `ba15f249` | -0.578 % | -0.397 % | -0.139 % | the CR 104.4b loop watchdog ran SipHash over fifty small integers |
| B | `6344adf6` | **-2.899 %** | **-2.432 %** | **-2.019 %** | a deck-fill memcpy'd an 8,232-byte `CardDefinition` per card |

**(B) is the pass, and the number that found it is a struct size.**
`CardDefinition` is **8,232 bytes**; `CardInstance::new` takes
`impl Into<Arc<CardDefinition>>`, and every deck-fill site handed it a fresh
`f()`, so `Arc::new` copied all of it once per card in a library. In the sos
profile `CardInstance::new` is the second-largest `__memcpy` caller — 3,452
memcpys / 28,451,728 Ir, **8,242 Ir apiece** — and that per-call figure is
what named the cause: a memcpy that expensive is copying kilobytes, and only
one thing in this engine is that big. `cube::card_arc(f)` memoizes one
`Arc<CardDefinition>` per factory per thread; a deck-fill is now a refcount
bump. It is sound because `Arc<CardDefinition>` is already the CoW handle for
a definition — the ~twenty sites that rewrite one all go through
`Arc::make_mut`.

**The RSS half is the same change seen from the other side** and matters more
to `selfplay_train` than the Ir does: a forty-card deck holds ~twenty distinct
definitions instead of forty, and an actor count multiplies it.

**The one column that goes the wrong way is the cold deck-build
microbenchmark, +0.330 %, and the first version of (B) read +6.591 % there.**
Routing `card_arc` through `card_brief`'s memo made a *miss* also pay
`CardBrief::of` — the pip counts, the keyword walk, `is_fixing_card`'s whole
effect-tree walk — and `--decks sealed --games 1` is eighty template cards
that are all misses and no games. Its own memo costs a map lookup and an
`Arc` allocation, which is the +0.330 %. **A long-lived actor pays that once
per factory ever**; the microbenchmark is a cold process by construction.

```text
decisions        196,220 -> 196,220        byte-identical
turns_per_game   27.53   -> 27.53
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0
determinism      ok; CRAB_THREAD_CHECK=1 ok (3 vs 1 threads identical)
ladder output    the full printout diffs identically on fixed, cube, sos
                 and sealed
suite            18,735 passed / 0 failed / 5 ignored over 14 result blocks
golden traces    all unchanged
clippy           `--workspace --all-targets` clean
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.10GHz, 4 cores
```

**Fifty-eighth pass, base `c18552fd` (pass 57's tip) vs its own tip
`13f3521c`.** Four commits, one class: **the sealed builder's last three
per-shape re-derivations, plus the land walk that rediscovered what the pile
builder already knew.** The workload that moves is `--decks sealed --games 1`,
which plays no games and so is deck construction and nothing else:
**26,478,634 -> 23,574,309, -10.968 %.** Ir readings `profiling-fast
--no-default-features`, callgrind, one thread, one argv throughout.

```text
                          base (c18552fd)   tip (13f3521c)
deck build (sealed, 1 g)     26,478,634       23,574,309   -10.968 %
I refs, --decks fixed     1,234,918,094    1,233,675,802    -0.101 %
I refs, --decks sos       1,658,496,337    1,657,624,397    -0.053 %
I refs, --decks cube      2,952,041,099      not re-read — no commit is on that path
```

| step | commit | deck build | what |
|---|---|---|---|
| A | `811cddec` | **-3.738 %** | the splash ranker reads the pool's memoized colours and scores |
| B | `432976a0` | -0.599 % | one `PoolScores` per pool, not one per random build |
| C | `88b97f25` | **-2.828 %** | fifty-seven shapes share three sorted orders |
| D | `13f3521c` | **-4.246 %** | the land walk reads the index the pile builder kept |
| — | *rebase onto pass 57's (D)+(E)*; the same four re-read `23,611,357` | | |
| E | `15cad53b` | **-7.782 %** | the copy cap counts by dense card id, not by `HashMap` |

**And a sixth commit that is not on the deck builder at all** — `8a0f11fb`,
candidate (-42)'s answer: `do_untap`'s per-turn seat reset writes ~55 fields
through `Player`, which is a CoW handle, so each was its own
`Arc::make_mut`. `make_mut` calls in that function **212,012 -> 80,148**;
`--decks sos` **1,644,049,924 -> 1,639,754,965, -0.261 %**, `--decks cube`
**2,910,850,945 -> 2,903,490,499, -0.253 %**. Same recipe, base re-read at
the tip that carries (E); a concurrent session's `cae6b605` (the grant list's
deep copy) then landed underneath it, and is not in either column. Two
cheaper explanations were built and refuted first; the line profile named
`Player::deref_mut`, not a hot line. See the Log and (-42).

**And a seventh, `cae6b605`, from the third concurrent session: the grant
list was deep-copied for callers that only ever read it.**
`granted_abilities_of` collected a permanent's granted activated abilities
into a `Vec<ActivatedAbility>` — a whole `Effect` tree, a cost and a dozen
filters per element — and its three callers either folded the list or wrapped
each element in `AbilityRef::Synth(Box::new(a))`, a second allocation, and
read it through `Deref`. Every source it draws from is reachable from
`&'a self`, `me` or the scan, so it returns `Vec<&'a ActivatedAbility>`; the
one ability with nothing to borrow from (CR 804.2's deploy-creatures grant,
all-constant fields) is a `LazyLock`. Base `7c7f2e5e` re-read at the tip that
carries (E) and `8a0f11fb`:

```text
                     base (7c7f2e5e)   tip (cae6b605)
I refs, --decks fixed  1,233,122,037   1,230,058,286   -0.248 %
I refs, --decks cube   2,910,851,269   2,896,273,896   -0.501 %
I refs, --decks sos    1,644,050,093   1,612,056,291   -1.946 %
```

The function's own callee table on `sos`: `ActivatedAbility::clone` **11,324
calls / 7,868,320 Ir** and `__memcpy` **11,324 / 2,501,090** both go to zero,
`RawVec::grow_one` **11,324 / 8,485,733 -> 9,782 / 1,252,111** (the `Vec` now
grows by pointers), and `evaluate_requirement_static_hinted` is unchanged at
39,430 / 19,241,574.

**And the clock cannot see any of it.** `scripts/ab_wall.py`, `release-fast` +
mimalloc, `--games 2000 --decks sos --threads 4`, **eight ABBA blocks**: mean
B/A **+0.18 %**, 95 % CI **-1.64 .. +2.00 %**, 3/8 blocks faster — and the
null control on the same workload reads -0.40 % with a comparable CI, so the
box's floor is +/-2 % and the effect is under it. The same workload on the
*system*-allocator binaries (`profiling-fast`) is flat too, which rules out
"mimalloc already made the allocation cheap" as the explanation. **Two earlier
readings of this same pair said otherwise and both were noise**: best-of over
nine hand-rolled pairs said +2.5 % slower, four ABBA blocks said +1.26 %
slower. That is what the tool exists for.

**This is pass 57's clock rule with its mechanism named, and it is the rule
this file should carry forward: `Ir` counts a `memcpy`; the machine barely
does.** A deep copy of a contiguous struct runs at high IPC out of a
just-written cache line, and the borrow that replaces it turns a hot-buffer
read into a pointer chase into the (cold, scattered) card definitions. The
commit is **kept** — the Ir is real, the ladder printout and the 18,728-test
suite are identical, and not deep-copying an immutable shared structure in
order to read it is a clarity win on its own — but **-1.95 % is not a
throughput claim**, and the next allocation-shaped candidate on this branch
should be sized with that in front of it.

**And an eighth, `4382bd43`, the rest of (-42)'s class** — `do_untap`'s tail, where the
runs of seat writes are split by `retain_cold!` calls on `self`. `make_mut`
in that function **80,148 -> 41,200**; `--decks sos` **1,607,757,957 ->
1,605,824,543, -0.120 %**, `--decks cube` **2,888,913,466 -> 2,885,591,189,
-0.115 %**. **Both base columns here are re-read at `d2a8320b`, i.e. with
`cae6b605` in** — quoting the sixth commit's `1,639,754,965` instead would
have read -2.069 %, seventeen times the real win, and the seventh commit
directly above is where that missing -1.946 % went. See (-42).

**And a ninth, the same device off the untap step entirely**, at the three
sites a call-count sweep of `make_mut`'s callers found: `advance_step`'s
cleanup-step seat reset **51,142 -> 9,198 (-82.0 %)**,
`deal_combat_damage_to_target` **21,012 -> 10,008 (-52.4 %)**, and
`clear_step_bounded_may_play`'s per-seat zone sweep. `--decks sos`
**1,605,824,543 -> 1,604,031,880, -0.112 %**, `--decks cube`
**2,885,591,189 -> 2,882,468,355, -0.108 %**. **Program-wide the three
CoW-handle commits take `make_mut` on `sos` from 582,552 calls to 475,676,
-18.3 %.**

**And a tenth, a second `--callers make_mut` sweep for the cheap-per-call
rows.** `finalize_cast`'s ten per-seat cast tallies take three bindings (a
`&mut self` call and two `GameState` field writes split the run)
**46,992 -> 26,474**, and `on_left_battlefield`'s six unconditional
`cast_from_*` clears take the gate-then-bind shape already used by the four
`CardCold` clears above them, **24,352 -> 8,494**. `--decks sos`
**1,604,031,880 -> 1,603,018,685, -0.063 %**, `--decks cube`
**2,882,468,355 -> 2,880,726,428, -0.060 %**; program-wide `make_mut`
**475,676 -> 439,300, -7.6 %**.

**So across the four CoW-handle commits `make_mut` on `sos` goes
582,552 -> 439,300, -24.6 %**; what is left, and the reason half of it is
*not* this device, is the new candidate (-43).

**Both game pools read slightly *down* rather than flat, and the reason is
the binary.** No commit here is on the game loop; `_dl_relocate_object` is
2.31 % of the deck-build workload and `build_shape` shrank, so the process
startup floor moved with it. Read -0.101 % and -0.053 % as "unchanged, and
the code got smaller", not as a game-loop win.

```text
decisions        196,220 -> 196,220        byte-identical
turns_per_game   27.53   -> 27.53
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism      ok (all pairs split); thread_determinism ok (3 vs 1 identical)
peak_rss_mib     22.0
suite            18,728 passed / 0 failed / 5 ignored (A-D, at (E), and at each
                 of the four CoW commits). **11 binaries actually run tests**;
                 the per-binary sum is 553 + 2 + 2 + 6,087 + 1,724 + 370 +
                 3,464 + 1,858 + 1,813 + 649 + 2,206. The "22" this file has
                 carried since the 18,712 era is cargo's result-line count,
                 which includes the zero-test lines a concurrent session is
                 removing — so count the nonzero lines, not the lines.
golden traces    7 passed, all unchanged
clippy           `--workspace --all-targets` clean, all eight crates
throughput       119.8 games/s warm (`selfplay_train --actors 3 --games 120
                 --steps 1 --seed 7`), six consecutive runs within 0.1 %, 0
                 stalls. An absolute at this tip, not an A/B claim — no
                 commit in this pass is sized off it. See "How to measure".
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.80GHz, 4 cores, host_calib_ms 58-73
```

**Crash-freedom and determinism at the tip, widest pool.** `--a gang --b gang
--games 400 --threads 3 --decks all`, seeds 11 / 12 / 13: **20,400 games,
20,396 decided, no panic**, and all 10,198 mirrored pairs split
(`rho -1.000` on every seed). The 4 undecided are seed 11's standing rules
draws, the same four every pass since the forty-fourth has recorded. **Re-run
at the tip that carries the three CoW-handle commits: identical — 3,398 /
3,400 / 3,400 pairs, 0 A-sweeps, 0 B-sweeps, every pair split.**

Behaviour beyond the bench: `--decks sealed --games 6` and `--decks all
--games 20 --seed 11` are **byte-identical to the base at every one of the
four steps** apart from the wall-clock line. That pair is the check a
deck-builder change needs and the golden traces cannot give — they play
hand-built decks, so the builder never runs in them.

**No wall-clock pair is quoted, and the arithmetic is why.** Deck
construction is ~2.2 M Ir per pool-plus-build against ~50 M for a sealed
game, and a `selfplay_train` actor builds two decks per game — so the whole
builder is ~8 % of an actor's per-game work and this pass is **~0.9 %** of
it. That is an order of magnitude under what `--bench` resolves on this box
(an 11 % spread between runs of one binary), which is the case this file says
callgrind exists for.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState` or
`Vocab` change is in this pass; the decks the builder produces are
byte-identical.

**(A)-(D) were read at `13f3521c`; two of pass 57's commits then landed on
top** (`7b0477d4` and `2394206c`, the gather's two `Vec`s) and (E) is measured
above them. Nothing in this pass is on the gather and nothing in theirs is on
the deck builder, so the two compose — but their two commits move the
deck-build workload from **23,574,309 to 23,611,357** (+0.157 %, code size on
a workload whose `_dl_relocate_object` is 2.5 % of the total), which is why
(E)'s base is re-read rather than chained. **The pass end to end is
`26,478,634 -> 21,774,018`, -17.77 %**, and the 37,048 Ir their commits add is
inside that.

**A `--decks sealed --games 1` absolute moves with the size of the binary, not
just with the deck builder.** Two commits that touch neither `recommend.rs`
nor `selfplay.rs` moved it 37 k. Re-read the base after any rebase before
quoting a delta on this workload.

**Anchors at the branch tip** (this pass's (E) on top of pass 57's (D)+(E)),
same recipe, for whoever measures next: `--decks sos` **1,644,049,924**,
`--decks cube` **2,910,850,945**, deck build **21,774,018**.

**Fifty-seventh pass, base `28ae2416` (pass 56's tip) vs its own tip
`6c5dd0ab`.** Five commits in two classes. **The simulator's two largest
engine functions each ended in a fan of narrow walks that ask a question the
board has already answered** — (A) `cg_lines.py --rows`, because the shape is
a run of identically-costed rows below the default print; (B) the gather's
thirty-eight per-static passes get a variant bitmask; (C) the trigger
dispatcher stops evaluating grant filters for grants no event in the batch
could fire. Then **the gather allocated twice for every effect it emitted** —
(D) and (E), landed by a second session and rebased on top of (C). **`--decks
cube` -7.97 %, `sos` -4.21 %, `fixed` +0.51 %.** Ir readings `profiling-fast
--no-default-features`, callgrind, one thread, `--a gang --b gang --games 6
--seed 1`.

```text
                       base (28ae2416)   (B) da30d1c2    (C) 353105fe     tip (E)
I refs, --decks cube     3,162,426,135   3,082,752,911   2,952,044,117   2,910,120,990  -7.973 %
I refs, --decks sos      1,715,663,129   1,656,877,045   1,658,498,742   1,643,320,227  -4.216 %
I refs, --decks fixed    1,226,171,600   1,233,007,810   1,234,920,109   1,232,447,924  +0.512 %
I refs, --decks sealed     3,430,701,306   not re-read — neither the gather nor
deck build alone              26,570,012   the dispatcher is on that path
```

| step | commit | cube | sos | fixed | what |
|---|---|---|---|---|---|
| B | `da30d1c2` | -2.519 % | **-3.427 %** | +0.558 % | the gather's thirty-eight per-static passes ask a variant bitmask |
| C | `c6ef9af8` | **-4.241 %** | +0.098 % | +0.155 % | the dispatcher drops a grant no event in the batch could fire |
| D | `603d354b` | **-1.001 %** | -0.634 % | -0.011 % | `static_ability_to_effects` collected a `Vec` its only caller drains |
| E | `6c5dd0ab` | -0.424 % | -0.283 % | -0.189 % | thirty-three arms returned `vec![one]` per emitted effect |

**(D) and (E) were written concurrently with (B) and (C) on `28ae2416` and
read -0.937 / -0.301 % on cube there; on top of (B)+(C) they read -1.001 /
-0.424 %.** They are worth *more* after the mask, on every pool, which is the
composition to expect: the mask took the walking out, so what is left of the
gather is a larger share allocation. They are the only rows in this pass that
move `--decks fixed` **down**.

**The columns above were read on `353105fe`; the second rebase then put pass
58's four deck-builder commits underneath (D) and (E).** Re-read at the true
branch tip `7918d1e6`: fixed **1,233,103,048**, cube **2,910,805,300**, sos
**1,643,976,414**. Against `353105fe` that is -0.147 / -1.397 / -0.876 % for
pass 58 + (D) + (E) together, against -0.200 / -1.420 / -0.915 % for (D)+(E)
alone — i.e. pass 58 is within 0.05 % of flat on all three *game* pools, which
is what a deck-builder change should be, and the two do not overlap.

**The two rows are on different pools and that is the pass's rule.** `sos`
carries the static abilities the gather walks; `cube` carries the
`GrantTriggeredAbility` statics the dispatcher walks (`--decks fixed` carries
neither, which is why it only ever pays). Neither commit is visible on the
other's pool, and both are invisible on the committed bench.

**(B) was read once on pass 56's base and again on pass 57's, and the two
passes compose to the third decimal.** It was measured first against
`91f3ede3` (sos -3.422 %, cube -2.476 %, fixed +0.551 %) and re-measured
after the rebase onto pass 56's eight commits: **-3.427 / -2.519 / +0.558**.
Nothing pass 56 removed was already removed here, and nothing here makes
pass 56's rows smaller.

**`fixed` pays and does not collect, and that is the pool, not the gate.**
No permanent in the vanilla archetype decks has a printed static ability *or*
a `GrantTriggeredAbility` static, so `sa_cards` is empty on all 32,002
gathers and `trigger_grants` is empty on every dispatch: both fans of walks
were already free there, and a gate can only add the cost of asking. Three
variants of (B)'s gate were built and measured rather than argued, and two
placements of (C)'s — see the Log. **`sos` is the pool
`crabomination_ml::selfplay_train`'s actors play** (`Vocab::sos_sealed`).

```text
decisions        196,220 -> 196,220        byte-identical
turns_per_game   27.53   -> 27.53
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism      ok (all pairs split, both)
suite            18,728 passed / 0 failed / 5 ignored over 22 binaries
golden traces    all unchanged
clippy           `--workspace --all-targets` clean
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.10GHz, 4 cores
```

**Wall clock over the whole pass**, `release-fast` + mimalloc, 600 games /
1 thread / seed 1, interleaved `tip base base tip` so linear drift cancels,
six readings a side, base `28ae2416` vs tip `c6ef9af8`:

```text
--decks cube   base 62.98 63.37 63.96 64.67 65.48 66.20   best 62.98  med 64.3
               tip  61.52 62.59 63.00 63.59 65.12 65.55   best 61.52  med 63.3
                                                          -2.3 % best, -1.6 % median
--decks sos    base 38.33 38.62 39.55 39.64 41.12 41.22   best 38.33  med 39.6
               tip  37.20 37.95 38.06 38.66 39.06 39.12   best 37.20  med 38.4
                                                          -3.0 % best, -3.1 % median
```

**`sos` tracks its Ir (-3.0 % against -3.33 %); `cube` does not (-2.3 %
against -6.65 %), and the gap is what (C) removes.** A requirement evaluation
the batch filter skips is a short, perfectly predicted walk over an
L1-resident grant list — Ir counts every instruction of it and the machine
retires several per cycle. The rule this file has for allocation-shaped
change (Ir over-reads it) has a mirror: **a change that removes cheap,
predictable instructions under-delivers on the clock.** Quote both.

**The pass's first pair was measured separately** (base `91f3ede3` against
(B)) and read cube -4.4 % / sos -4.0 % on best-of-interleaved, on a sitting
where the same workload ran 56-59 s rather than 62-66 s. Neither absolute is
comparable to the other; both deltas are.

**`--bench`, paired** (`release-fast` + mimalloc, 3 threads, interleaved
`tip base base tip`, six a side):

```text
base  189.65 184.15 196.87 202.21 204.62 204.47    mean 197.00  best 204.62
tip   190.49 194.80 198.97 199.28 199.86 204.33    mean 197.96  best 204.33
                                                   +0.5 % mean, -0.1 % best
```

**Read it as flat**, which is what +0.71 % of Ir on that pool has to be
against an 11 % spread. Earlier in the same session a **`release` pair** was
built for (B) — base `91f3ede3` at `release` reads mean 210.48 / best 216.47
against the tip's 212.36 / 225.44 over nine readings a side, also flat, and
**the base binary reads 210 there against the 281 this file records at the
pass-55 tip for the same commit and profile**. A `--bench` absolute is never
a cross-sitting comparison; the anchor is not refreshed.

**Crash-freedom and determinism at the tip.** `release-fast`, `--a gang
--b gang --games 200 --threads 3`, seeds 11/12/13 x `--decks all` plus
`--decks sealed` at seed 11: **12,600 games, every cell decided, 0
undecided, no panic**. `CRAB_THREAD_CHECK=1 --bench` reads
**`thread_determinism ok (3 vs 1 threads identical)`**, `decisions` 196,220,
`turns_per_game` 27.53, `stalls_by` 0/0/0. The same grid ran clean at (B) on
a `release` build.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState`
or `Vocab` change is in this pass.

**Fifty-sixth pass, base `00348ada` vs its own tip `3801f01f`,
re-read on the branch after a second rebase onto pass 55's (I)-(K).** Eight
commits in two classes: **five things the deck builder derived once per
shape from a pool that never changed** (the deck-build workload is
**-23.1 %**), and **three re-reads the game loop paid on every call**. Ir
readings `profiling-fast --no-default-features`, callgrind, one thread,
`--a gang --b gang --games 6 --seed 1`; both columns were read directly on
this box with one argv, so the deltas are exact.

```text
                          base (00348ada)   tip (3801f01f)
I refs, --decks fixed       1,243,883,799    1,236,397,437   -0.602 %
I refs, --decks cube        3,199,250,814    3,182,035,509   -0.538 %
I refs, --decks sos         1,735,661,867    1,728,941,302   -0.387 %
I refs, --decks sealed      3,483,347,170    3,459,192,362   -0.693 %
deck build alone               34,622,104       26,612,999  -23.13 %
  (--decks sealed --games 1: 0 games played, all setup)
```

**Re-read after the second rebase, and the two passes compose to the third
decimal.** Pass 55 landed its (I)-(K) under this chain mid-flight; measured
again on top of them (branch tip `9582d1ea`, and pass 55's (K) `c676cf48` as
the base column):

```text
                       pass 55 (K)      branch tip (9582d1ea)
I refs, --decks fixed   1,234,031,722    1,226,172,210   -0.637 %
I refs, --decks cube    3,179,782,586    3,162,426,697   -0.546 %
I refs, --decks sos     1,722,423,954    1,715,663,088   -0.392 %
I refs, --decks sealed         n/a       3,430,701,306
deck build alone               n/a          26,570,012
```

**-0.637 / -0.546 / -0.392 against -0.602 / -0.538 / -0.387 on the pass's
own base** — the same rows, so nothing this pass removes was already removed
by (I)-(K), and nothing it removes makes theirs smaller. The base column
there is pass 55's own reading on its own box; the offset between the two
containers is the argv string (this pass measured `00348ada` at +483 /
+1,319 / +707 Ir over pass 55's numbers for the same commit), which is three
orders below the deltas.

Per commit. **Measured on the pass's own chain against `00348ada`**, before
the second rebase.

| step | commit | pool it moves | delta |
|---|---|---|---|
| B | `1c223827` | cube / fixed / sos | -0.090 % / -0.018 % / -0.051 % — the requirement walker's stack leg was the one eager leg in a lazy chain |
| C | `02caa399` | fixed / cube / sos | **-0.291 %** / -0.188 % / -0.153 % — auto-tap's source table carries its battlefield index, candidate (-38)'s `actions.rs:12626` |
| D | `9c9afc74` | deck build | **-8.03 %** — `SosPacks`: six packs from one pool re-derived the pool's sheet and buckets six times |
| E | `1ca90507` | deck build | -3.04 % — `candidate_label` allocated a `String` per colour, a `Vec` and a join buffer, per candidate |
| F | `a8ced063` | deck build | -3.49 % — the builder looked the same `CardBrief` up three times per card per shape |
| G | `3b7d2c0b` | deck build | -3.94 % — every shape re-summed `colors_of_picks(pool)` |
| H | `708171c3` | deck build | **-7.35 %** — `PoolScores`: a card's base score is the same for all ~57 shapes, which follows from G |
| J | `4871ffb7` | fixed / cube / sos | **-0.280 %** / -0.214 % / -0.150 % — eleven sites cloned the computed keyword list to read it |
| — | dropped | — | **this pass's own (A) was the same optimization as pass 55's (B)** and was dropped on the first rebase. See the Log |
| — | reverted | cube +0.123 % | presence gates for `has_atype` / `has_stype`, candidate (-37)'s residue. See the Log |

```text
decisions          196,220 -> 196,220      byte-identical
turns_per_game     27.53   -> 27.53
stalls             0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism        ok (all pairs split, both)
thread determinism ok (3 vs 1 threads identical)
ladder output      --decks sealed --games 6's full printout diffs identically
                   base vs tip — the check that covers the decks a seed
                   *builds*, which five of the eight commits touch
suite              18,728 passed / 0 failed / 5 ignored over 22 binaries
golden traces      all unchanged
clippy             `--workspace --all-targets` clean
rustc              1.95.0 (59807616e 2026-04-14)
host_cpu           Intel(R) Xeon(R) Processor @ 2.80GHz, 4 cores
```

**Wall clock, `release-fast` + mimalloc, both binaries alternated in one
sitting on this box, and the deck build is the number this pass claims.**
100 invocations of `--decks sealed --games 1` (which plays no games), minus
the process-startup floor measured the same way with `--decks fixed`, four
alternated pairs:

```text
                base      tip
pair 1        0.6862    0.5292   -22.9 %
pair 2        0.5978    0.5911    -1.1 %
pair 3        0.6468    0.5578   -13.8 %
pair 4        0.6186    0.5128   -17.1 %       4/4 pairs positive
best-of       0.5978    0.5128   -14.2 %
floor (100 procs)  0.3093    0.3294
```

**-14 % against -23 % Ir, and two things account for the gap.** The
pass-54 caveat first: this work is allocation-shaped, callgrind runs the
*system* allocator and mimalloc ships. And the tip's own startup floor is
**6.5 % higher** (0.3294 s against 0.3093 s per 100 processes) — a bigger
binary to relocate, which is subtracted from both columns but is real time
the change added. A training actor never pays it: it builds its decks inside
one long-lived process.

The game loop, same binaries and sitting, `--decks cube` 600 games / 1
thread / seed 1, alternated: base **62.73 / 62.43** against tip **62.06 /
63.03**, best-of **-0.6 %**, which is the -0.538 % Ir to within the drift.

**`--bench` cannot see this pass's game-loop rows, and that is the expected
result rather than a null one.** Paired `release-fast`, 3 threads,
A/B/A/B/A/B: base 201.33 / 200.61 / 200.41 against tip 203.59 / 197.39 /
194.09 — **1/3 pairs positive, mean -1.2 %** against -0.602 % Ir on that
exact pool. A 0.6 % change is a quarter of this bench's spread; the Ir
column is the attribution and the cube pair above is the wall-clock claim.

**The committed `release` anchor is not refreshed, and the reading this
container gives is why.** `release` + mimalloc, 3 threads, three readings at
the pass's tip: **208.03 / 219.77 / 212.18**, `decisions` 196,220 on all
three, `turns_per_game` 27.53, `stalls_by` 0/0/0, `peak_rss_mib` 28.1-30.1,
`host_calib_ms` **45 / 47 / 46** — against pass 55's 270.23 / 276.73 /
269.17 at calib 55/53/55 on its box. This box's single-core probe is
*faster* and its three-thread throughput is a fifth lower, which is the
disagreement this file has now written up four times (the `bdc11c86`
re-anchor, the "two sittings, one binary-identical workload, 3.7 % apart"
pair, the note that the calib probe is single-threaded, and this). Nothing
intentional changed the workload, so the anchor stands and a
cross-container absolute is not evidence of anything.

**Crash-freedom and determinism at the tip.** `release-fast`, `--a gang
--b gang --games 200 --threads 3`, seeds 11/12/13 x `--decks all` plus
`--decks sealed` at seed 11: every cell **decided, 0 undecided, no panic,
all pairs split** — 12,600 games and 6,300 pairs. The `overflow` profile
(`release-fast` + `overflow-checks`) over seeds 11/12/13 x `--decks all`
reads the same counts with **no panic and no arithmetic overflow** — 10,200
games. `CRAB_THREAD_CHECK=1 --bench` reads **`thread_determinism ok (3 vs 1
threads identical)`**.

**No net needs retraining, and five of the eight commits are in the deck
builder, so that claim is checked rather than asserted.** No encoding, pool,
`TrainRow`, `EncodedState` or `Vocab` change is in this pass, and
`--decks sealed --games 6`'s full ladder printout — which covers the decks
each seed *builds*, not just the games they play — is byte-identical
between the base and tip binaries.

**Fifty-fifth pass, base `bf4917a5` (pass 54's tip) vs its own tip
`c676cf48`.** Eleven commits, one class after the first: **the simulator kept
building answers before asking whether anyone wanted them.** (A) the
requirement walker's subtype arms stop gathering where the printed line
answers; (B) the freeze scope's depth and gate slots come out of the mutex;
(C)-(K) nine helpers that allocated, cloned or re-found something their
caller discards or already holds. **`--decks cube` -20.75 %,
`--decks sos` -2.16 %, `--decks fixed` -1.15 %.** Ir readings `profiling-fast
--no-default-features`, callgrind, one thread, `--a gang --b gang --games 6
--seed 1` unless the row says otherwise.

```text
                          base (bf4917a5)   (A) 8779aa9f     tip (K)
I refs, --decks cube        4,012,095,058   3,332,029,985   3,179,782,586  -20.75 %
I refs, --decks fixed       1,248,407,927   1,249,622,086   1,234,031,722   -1.151 %
I refs, --decks sos         1,760,442,504   1,761,529,321   1,722,423,954   -2.160 %
I refs, --decks sealed      3,497,162,303   3,500,013,528     (B), below
deck build alone               34,506,869      34,859,382     (B), below
  (--decks sealed --games 1: 0 games played, all setup)
```

Per commit, the three pools each was measured on:

| step | cube | fixed | sos | what |
|---|---|---|---|---|
| A `8779aa9f` | **-16.95 %** | +0.097 % | +0.062 % | the requirement walker's subtype arms ask a presence gate before forcing the layer view |
| B `4c58c9c7` | -0.709 % | -0.276 % | -0.365 % | the freeze scope's depth and gate slots come out of the mutex |
| C `24860169` | -0.665 % | +0.001 % | -0.338 % | `affected_from_requirement`'s And-tree stack is an inline array, not a heap `Vec` |
| D `67fc39ab` | -0.114 % | -0.121 % | -0.064 % | `restore_payment_state` asks with a shared borrow before unsharing the battlefield |
| E `5f988142` | -1.021 % | flat | -0.442 % | `extract_power_gate` asks `requirement_mentions_power` before cloning the tree |
| F `9d9555e9` | -1.208 % | -0.034 % | -0.018 % | the per-card grant walk hands the permanent to the filters instead of re-finding it |
| G `863d882d` | -0.205 % | -0.028 % | -0.254 % | `granted_abilities_of` does the same for the mana sweep's grant scan |
| H `353273ef` | -0.126 % | flat | flat | thirteen more battlefield walks hand their card to the requirement walker |
| I `7ec4836c` | -0.256 % | **-0.356 %** | -0.291 % | the gather's two always-empty `collect()`s become `Vec::new()` |
| J `ca16b33a` | -0.291 % | -0.369 % | **-0.422 %** | the SBA sweep's game-over check is a walk, not two `Vec`s and a sort |
| K `c676cf48` | -0.062 % | -0.069 % | -0.051 % | combat's three per-attacker collects are gated on a presence scan |
| — | +0.40 % | +0.66 % | — | **REVERTED** — the presence gate on `board_keyword_matching`'s *frozen* leg. See the Log |
| — | +0.43 % | +0.12 % | — | **REVERTED** — a two-phase exactly-sized build in `statics_granted_triggers_with`. See the Log |

`sealed` and the deck build were read at (B) and not re-read; (C) through (K)
are engine paths the deck builder does not reach.

**The deck-build row was layout, and the profile says so rather than the
usual hand-wave.** At (A) it read +1.02 %, and
`creature_type_change_in_scope` / `land_type_change_in_scope` do not appear
in that dump at all — the deck builder never reaches the requirement walker.
The whole +352,513 sat in `LocalKey::with` (207 attributed calls before,
4,850 after), i.e. the `card_def` front cache inlining differently under a
bigger binary; (B) moved it back to +0.293 % without touching that code,
which is the confirmation. Check the callee before blaming a row like this;
here it cost one `cg_edges.py` run to rule out.

```text
decisions        196,220 -> 196,220        byte-identical
turns_per_game   27.53   -> 27.53
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism      ok (all pairs split, both)
suite            18,827 passed / 0 failed / 5 ignored over 31 binaries
                 (the whole workspace, at the final tip)
golden traces    all 7 unchanged
clippy           `--workspace --all-targets` clean
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.10GHz, 4 cores
```

**Wall clock over the whole pass, and it is a quarter of the Ir.**
`release-fast` + mimalloc, 600 games / 1 thread / seed 1, alternated
base/tip in one sitting. The base here is **the pass's own engine diff
reverted at the current catalog** — the concurrent session's card commits
land between `bf4917a5` and this tip, and this isolates the engine work
from them.

```text
--decks cube    base 54.06 / 55.32 / 55.10   best 54.06
   600 games    tip  50.70 / 52.20 / 51.75   best 50.70   -6.2 %
--decks sos     base 31.90 / 31.80 / 31.73   best 31.73
   600 games    tip  30.85 / 30.95 / 30.67   best 30.67   -3.3 %
--decks fixed   base 59.44 / 59.79           tip 59.31 / 59.47   -0.2 %
   1200 games
```

The tip is faster in every one of the eight pairs. **`sos` -3.3 % is the
number the training loop gets** — that is the pool a `selfplay_train` actor
plays.

**The training loop itself was measured and is INCONCLUSIVE, which is worth
recording rather than rounding into a win.** `selfplay_train --actors 3
--games 6000 --steps 1 --seed 7`, both binaries `release-fast` + mimalloc,
alternated in one sitting:

```text
base   144.4 / 176.1 / 178.2 games/s      best 178.2
tip    185.1 / 177.7 / 179.5              best 185.1     (+3.9 %)
wall   base 34.2 / 34.0 / 33.0 s          tip 33.5 / 33.1 / 33.2 s   (flat)
```

The reported rate and the wall clock disagree in sign, and the base's spread
on the identical binary is **23 %** (144.4 to 178.2) against the tip's 4 %.
Three actors on a four-core box that is also running the harness is not an
instrument at this resolution. **Use `--decks sos` on the ladder as the
proxy** — same pool, one thread, 3/3 pairs, -3.3 % — and re-run the training
loop on the box that matters.

**-6.2 % wall clock against -20.7 % Ir, and the gap is the point.** A sixth
of the Ir this pass removed is the allocator family, callgrind runs the
*system* allocator, and mimalloc ships — so the Ir is the attribution and
the wall clock is what the training host gets. **An earlier sitting read
(A) alone at -6.8 %** on a different catalog; the two are not comparable and
neither is wrong, which is this file's standing warning about wall-clock
absolutes restated. Quote both numbers or neither.

The gap is the pass-54 caveat again: a sixth of the Ir saved is the
allocator family, callgrind runs the *system* allocator, and mimalloc ships.
Quote both numbers or neither.

**`--bench`, the committed throughput configuration** (`release` + mimalloc,
3 threads), three readings at the tip:

```text
games_per_s      277.25 / 281.06 / 274.34   best 281.06  (pass 54 tip: 269.41)
decisions        196,220 on all three
turns_per_game   27.53
stalls_by        cap 0 / stuck 0 / draw 0 on all three
peak_rss_mib     28.2 / 28.4 / 30.1         (pass 54 tip: 30.3)
host_calib_ms    52 / 65 / 55
```

**Read it as up, but not by 4 %.** `--bench` is `--decks fixed`, which moved
-1.08 % in Ir over the pass and -0.2 % on the release-fast wall clock above;
the 2.5 % spread across three back-to-back runs of *one* binary is this
file's standing warning about `--bench` absolutes, and the rest of the gap
to pass 54's 269.41 is a different sitting on a different host. No base
binary was built at `release` here — the Ir column is the attribution and
the release-fast pairs above are the wall-clock claim.

**Crash-freedom and determinism at the tip.** `release`, `--a gang --b gang
--games 200 --threads 3`, seeds 11/12/13 x `--decks all` plus `--decks
sealed` at seed 11: every cell **decided, 0 undecided, no panic, all pairs
split** — 12,600 games and 6,300 pairs, re-run unchanged at the final tip. `CRAB_THREAD_CHECK=1 --bench` reads
**`thread_determinism ok (3 vs 1 threads identical)`**.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState`
or `Vocab` change is in this pass.

### The fifty-fourth pass's baseline

**Base `4369a0d6` (pass 53's tip) vs its own tip
`e1cbc390`.** Nine commits in two classes: **deck construction stops
re-deriving what a memoized definition already answers** (seven), and **two
gathers that nobody read** (two, found with a measuring device this pass
added — see `scripts/cg_contexts.py`). Ir readings `profiling-fast
--no-default-features`, callgrind, one thread, `--a gang --b gang --games 6
--seed 1` unless the row says otherwise.

```text
                          base (4369a0d6)   tip (e1cbc390)
I refs, --decks fixed       1,250,405,745   1,248,408,061   -0.160 %
I refs, --decks cube        4,026,141,796*  4,012,096,941   -0.349 %
I refs, --decks sos         1,760,202,906*  1,760,445,728   +0.014 %
I refs, --decks sealed      3,572,196,844*  3,497,168,270   -2.101 %
deck build alone              111,755,559      34,509,612  -69.12 %
  (--decks sealed --games 1: 0 games played, all setup)
```

**Re-read after a rebase onto a concurrent session's three vocabulary-loader
commits** (`796427ab`, `e8f5dbad`, `b36ba8f2`, which sit between this pass's
`457b3864` and its two engine commits). `fixed` moved 1,248,410,451 ->
1,248,408,061 and the deck build 34,511,759 -> 34,509,612 — 2,390 and 2,147
Ir, i.e. nothing, which is what a change confined to the net loaders should
do. The `cube` / `sos` / `sealed` rows above were read on the pre-rebase tip
and are not re-read; the `fixed` pair is what says they did not move.

\* the base `cube` / `sos` / `sealed` figures are the fifty-third pass's own
tip readings carried forward. This pass's base *is* that tip, and its `fixed`
re-read 1,250,405,745 against the recorded 1,250,409,741 — 3,996 Ir of argv,
the offset every pass sees.

**The deck-builder commits move the three non-sealed pools by code layout
alone** — `fixed`, `cube` and `sos` build their decks from hand lists or the
cube recipe and reach none of that code — and at the seventh commit that
drift had accumulated to +0.13 % on `fixed`. The last two commits are engine
work and take it back past zero. Read the per-commit table for which is
which; no single deck-builder commit's own `fixed` delta was over 0.08 %.

**The wall-clock number, and it is the one that matters for training.** The
deck-builder work is allocation-shaped, so callgrind's system allocator
overstates it; measured on the shipped `release` build with mimalloc, 30
invocations of `--decks sealed --games 1` (which plays no games), minus the
0.113 s process-startup floor measured the same way with `--decks fixed`:

```text
                base binary   tip binary
30 x deck build   0.458 s       0.163 s     2.81x
```

**And the number that decides whether any of it matters: the training loop.**
`selfplay_train --actors 3 --games 3000 --steps 1 --seed 7`, both binaries
`release-fast` + mimalloc, alternated A/B/A/B/A/B in one sitting, plus an
earlier `--games 900` pair the same way:

```text
base   117.3 / 156.5 / 136.7 / 129.1 / 156.5 games/s      best 156.5
tip    169.8 / 169.6 / 165.6 / 152.5 / 167.2 games/s      best 169.8   +8.5 %
rows/s 15,163 -> 16,498 at the two best readings
```

**And the judged builder, which is where the deck work compounds.**
`--use-deck-best` runs `best_build_by(pool, 32, ..)` — thirty-two
`build_random_deck`s per side per game — so every commit above pays
thirty-two times over on that path. It was untestable at the pass's start
(no committed deck net loads; see TODO's ML section), so a throwaway one was
trained at the current vocabulary to run it, which also **verifies the
vocabulary freeze end to end**: a net trained after the freeze loads, pads
and drives the actors.

```text
--actors 3 --steps 1 --seed 7, release-fast + mimalloc, alternated,
best of four (two pairs at --games 600, two at --games 1200)
  judged (--use-deck-best)   132.9 / 146.2 / 148.9 / 135.3   best 148.9
  unjudged, same sitting     152.6 / 155.9 / 158.3 / 162.3   best 162.3
  judged / unjudged                                          91.7 %
```

**The judged path is within 8 % of the unjudged one**, where the fifty-third
pass left it at 83.4 % (83.2 against 99.8 on a different box — the *ratio* is
what carries across hosts, not the absolutes). 0 stalls in ~7,600 judged
actor games.

**+8.5 % on best-of-five, and it is well under the ~19 % the Ir predicted.**
That gap is this file's own caveat and it is worth stating plainly: the
builder's cost is allocation-shaped, callgrind runs the *system* allocator,
and mimalloc — which is what ships — had already absorbed a good part of what
the Ir attributes to the change. Note also the spread: base reads 129.1 to
156.5 on the identical binary minutes apart while the tip reads 152.5 to
169.8, so take the best of each pair rather than the mean, and do not quote a
single run of this workload.

**`--bench`, the committed throughput configuration** (`release` + mimalloc,
3 threads), both binaries alternated A/B/A/B in one sitting, best of two:

```text
                 base            deck-builder tip   final tip
games_per_s      264.71          264.79             269.41
decisions        196,220         196,220            196,220   byte-identical
turns_per_game   27.53           27.53              27.53
stalls           0 (0.00 %), cap 0 / stuck 0 / draw 0 (all three)
determinism      ok (all pairs split); CRAB_THREAD_CHECK ok (3 vs 1)
peak_rss_mib     31.3            30.2               30.3
ladder output    all four pools' full printout (20 games, seed 1) diffs
                 identically base vs tip — the strongest behaviour check
                 here, because it covers the decks a seed builds and not
                 just the games they play
suite            18,815 passed / 0 failed / 5 ignored over 31 binaries
golden traces    all unchanged
clippy           `--workspace --all-targets` clean (client included)
rustc            1.95.0 (59807616e 2026-04-14)
host_cpu         Intel(R) Xeon(R) Processor @ 2.10GHz, 4 cores
host_calib_ms    53-70 across the readings
```

**Read the `--bench` column as flat and nothing else.** `--bench` is
`--decks fixed`, which builds no deck, so the seven deck-builder commits
cannot move it and did not (264.71 -> 264.79). The final +1.8 % is the two
engine commits plus host drift, and this file's own note two sections down
says a `--bench` absolute has read 3.7 % apart on one binary in one
container ninety minutes apart. The Ir column is the attribution.

**Crash-freedom and determinism at the tip.** `release`, `--a gang --b gang
--games 200 --threads 3`, seeds 11/12/13 x `--decks all` and `--decks sealed`
(the deck builder's own pool, which `--decks all` does not include): every
cell **decided, 0 undecided, no panic, all pairs split** — 17,400 games and
8,700 pairs. Re-run unchanged after the run's later card-defect commits **and after the
rebase onto the fifty-fifth pass**, at `1badee12`: same grid clean (11,600
games), `thread_determinism ok (3 vs 1)`, `--bench` decisions 196,220 / turns
27.53 / stalls 0 / **269.72 games/s best of four**, suite 18,827 passed /
0 failed / 5 ignored, clippy `--workspace --all-targets` clean. All four
pools' 20-game printout diffs identically against the pass base at the
pre-rebase tip.

**A caution the same sitting supplied.** The first two `--bench` runs after
clippy read 225.24 and 231.00 with the host still settling, against 252-270
across four runs minutes later on the same binary — a **16 % spread**, four
times the 3.7 % this file already records. Take the best of several, never
the first after a build.
**Read that last one narrowly**: eleven cards changed behaviour in this run
(a dropped "you may", a collapsed mode, three absent abilities), and a
20-game sample not separating them means the sample did not reach them, not
that nothing moved. Their per-card tests are the proof of those changes; the
ladder diff is only evidence that the *engine* did not move. `CRAB_THREAD_CHECK=1 --bench` reads **`thread_determinism ok
(3 vs 1 threads identical)`**. And the `[profile.overflow]` run that turns a
silent wrap into a panic, re-run here because the deck builder is the code
that moved: `--decks sealed` 2,400 games and `--decks all` 3,400 games at 3
threads, **0 panics**. The `selfplay_train` A/B above is a further ~19,500
actor games at the tip with no stall and no panic.

**The front cache's memory cost, since it is a new per-thread allocation.**
4,096 slots x (8-byte key + 8-byte pointer) = 64 KiB per thread, in `.tbss`
— zero image cost, and `peak_rss_mib` moved 31.3 -> 30.2 at three threads,
i.e. inside the noise.

**No net needs retraining.** No encoding, pool, `TrainRow`, `EncodedState`
or `Vocab` change is in this pass, and every pool's ladder output is
byte-identical — only the cost of building the decks moved.

Per commit, `--decks sealed --games 1` (the deck build in isolation), with
each commit's `--decks fixed` reading beside it:

| step | deck build, before -> after | fixed | what |
|---|---|---|---|
| A `3c154e8d` | 111,755,559 -> 104,842,406 (**-6.186 %**) | +2,194 | `colors_of_picks` returns `ColorCounts` (`[u32; 5]`) instead of a `HashMap<Color, u32>` |
| B `5489b9fa` | 104,842,406 -> 94,866,008 (**-9.516 %**) | +0.006 % | both shape rankers read the pip totals they already hold instead of five more walks of the spell list |
| C `b10fdebd` | 94,866,008 -> 70,513,288 (**-25.67 %**) | -0.005 % | a 4,096-slot direct-mapped front cache in front of `card_def`'s map probe |
| D `5ca71f05` | 70,513,288 -> 65,341,053 (**-7.335 %**) | +0.076 % | `pip_counts` walks a cost once instead of once per colour in it |
| E `9cc1175c` | 65,341,053 -> 43,588,088 (**-33.29 %**) | -0.014 % | `CardBrief`: the per-definition derived facts (pips, cmc, type flags, quality, the fixing walk) memoized with the definition |
| F `735e365d` | 43,588,088 -> 37,942,385 (**-12.95 %**) | +0.066 % | the builder's four piles are `with_capacity`, not grown a doubling at a time |
| G `ec138369` | 37,942,385 -> 34,861,499 (**-8.12 %**) | -0.0003 % | `land_produced_colors` is a `ColorSet` on the brief, not a `Vec` per land per shape |
| — | 34,861,499 -> 35,864,023 (**+2.88 %**) | — | **REVERTED** — iterating `ColorCounts` by zipping rather than indexing. See the Log |

And the two engine commits, measured on `--decks fixed` (the deck build is
not what they touch). Their base is `457b3864`, the vocab-freeze commit,
re-read at **1,252,225,395** — +0.016 % on `ec138369`, layout again:

| step | fixed, before -> after | what |
|---|---|---|
| — | 1,252,225,395 -> 1,252,445,508 (**+0.018 %**) | **REVERTED** — a freeze scope around `eval_material_inner`'s board walk. Zero gathers removed; see the Log |
| I `25438a8b` | 1,252,225,395 -> 1,250,520,577 (**-0.136 %**) | `do_phasing`'s presence gate asked from inside its own freeze scope, so it gathered the effect set it exists to avoid |
| J `e1cbc390` | 1,250,520,577 -> 1,248,410,451 (**-0.169 %**) | the gather's own buffer was a `Vec::clone` (`capacity == len`) and reallocated on its first static ability |

**Fifty-third and fifty-second passes, compacted.** The full blocks are in
git (`PERF.md` before the sixty-first pass) and the Log entries carry the
substance. The numbers worth keeping:

```text
pass 53  base d37f31d8 -> tip ae938ac3
  fixed  1,265,405,219 -> 1,250,409,741   -1.185 %
  cube   7,962,354,254 -> 4,026,141,796  -49.436 %
  sos    1,771,650,597 -> 1,760,202,906   -0.646 %
  — and the two largest wins were invisible on `--decks fixed`, which is
    why they survived fifty-two passes. That is where "which pool a change
    moves" comes from.
pass 52  base b906be3b -> tip 1,265,410,851
  fixed  1,314,290,577 -> 1,265,410,851   -3.716 %
  decisions 198,810 byte-identical, turns_per_game 27.94, stalls 0
  — the pickers that dry-run their picks hand the state out, so the driver
    adopts it instead of running the same action a second time.
```


**Fiftieth pass, base `e7b3b3d4` (pass 49's tip) vs its own tip**, both
`profiling-fast --no-default-features`, built and run in one sitting on one
box. Rebased onto the concurrent run of the same pass (`4107e017`,
`49fce1ff`) after measuring; those two commits touch only `scripts/*.py` and
the trackers, so both columns stand unrederived.

```text
                     base (e7b3b3d4)          tip
I refs (callgrind)   1,531,246,782            1,314,288,098   -14.168 %
decisions            196,220                  196,220         byte-identical
turns_per_game       27.53                    27.53
stalls               0 (0.00 %), cap 0 / stuck 0 / draw 0 (both)
determinism          ok (all pairs split, both)
allocations          not re-read              755,521
peak_rss_mib         21.6                     22.5
suite                18,712 passed / 0 failed / 5 ignored over 22 binaries
golden traces        all 5 unchanged
clippy               `--workspace --all-targets` clean
host_cpu             Intel(R) Xeon(R) Processor @ 2.10GHz
host_calib_ms        47-49 across every reading
```

**The base column is 11 Ir under pass 49's recorded 1,531,246,793 for the same
commit** — argv length again (this run's `--callgrind-out-file` name is a
character shorter; pass 49 saw 492 Ir of the same effect and pass 47 686). The
base was read *directly* here, so the delta is exact.

**This box is not pass 49's.** `host_cpu` reads 2.10 GHz against that pass's
2.80 GHz and `host_calib_ms` 47-49 against 52-57, so **no `games_per_s` from
this pass compares to one from that pass** — see the note under **How to
measure**. Ir transfers between containers to within the argv string, which is
why the Log rows are quoted in Ir and nothing else.

**Crash-freedom and determinism at the tip, widest pool — and the recipe
changed after the fiftieth pass, because the old one could not see a
determinism bug that had been on the branch the whole time.** It was three
seeds at one thread count; it is now **thirteen seeds across `--threads
1/2/3`**, and the number to read is the **sweep count**, not just the panic
count: in a `gang`-vs-`gang` mirror the two games of a pair are one game with
the seats relabelled, so a single sweep is a bug. `CRAB_PAIR_SWEEPS=1` names
the offending pair and prints the seed that replays it.

`--a gang --b gang --games 400 --decks all`, seeds 11-23: **88,400 games,
88,382 decided, no panic**, and **all 42,391 mirrored pairs split**. The
undecided are rules draws, the same ones passes 44-50 recorded. What the old
recipe missed: `restart_game` (CR 727) rebuilt the state with
`GameState::new`, whose `GameRng` is `from_entropy`, so a seeded game that
*restarted* stopped replaying — fixed in `c6898506`, written up as TODO's
twenty-first robustness filter.

**Passes 45-49's Ir blocks, compacted.** The full blocks are in git
(`git log -- PERF.md`); every one is `profiling-fast --no-default-features`,
callgrind, `--a gang --b gang --games 6 --seed 1`, and every one reads
`decisions` **196,220**, `turns_per_game` **27.53**, `stalls` 0
(`cap 0 / stuck 0 / draw 0`) and `determinism ok` — that invariant set is the
only column that chains across them.

| pass | base -> tip | I refs | delta |
|---|---|---|---|
| 45 | `8a384e5c` -> `fec179f0` | 1,810,336,693 -> 1,765,005,375 | -2.504 % |
| 46, own chain | `11792f4c` -> `61fb3007` | 1,771,223,960 -> 1,747,982,407 | -1.312 % |
| 46, rebased and with its (E) | `fec179f0` -> tip | 1,765,005,375 -> 1,735,997,491 | -1.643 % |
| 47, own chain | `c9606062` -> `3706f96f` | 1,727,336,594 -> 1,674,581,042 | -3.054 % |
| 47, rebased and with the `Keyword::eq` pair | `636902ca` -> `a98d39b0` | 1,715,304,981 -> 1,645,831,969 | -4.050 % |
| 48, own chain | `89f55a5c` -> `1b32e4fb` | 1,662,145,003 -> 1,643,104,718 | -1.146 % |
| 48, rebased (A-E, then with F) | `40fb5e31` -> branch tip | 1,645,831,968 -> 1,628,221,407 / 1,625,262,542 | -1.070 % / -1.250 % |
| 49, own chain | `40fb5e31` -> own tip | 1,645,831,476 -> 1,560,268,509 | -5.198 % |
| 49, rebased onto 48 | `04282f2e` -> final tip | 1,625,264,320 (derived) -> 1,531,246,793 | -5.785 % |

**The branch across passes 46 and 47: 1,765,005,375 -> 1,645,831,969,
-6.752 %**, and every adjacent pair **composes**. Pass 47's seven commits take
*more* off the branch after pass 46's `cast_cost_scan` landed underneath them
(-53,159,867) than they did before it (-52,755,552); pass 48's rows read
slightly *smaller* rebased (-1.070 % against -1.146 %) because pass 47's
`Keyword::eq` pair had already removed some of what its (B) and (E) reach; and
pass 49's read slightly *larger* (-5.359 % against -5.198 %) because pass 48's
(E) took the `mana_source_table` gathers out from under the same ticks.
Pass 49's intermediate rebased readings, for the Log rows that chain to them:
`bf658313` (derived 1,628,220,915) -> A+B 1,540,962,924 -> C 1,538,787,495,
with **908,931** allocations at the A+B tip. Allocation counts across the two:
967,377 -> 949,413 (48), 967,377 -> 926,895 (49).

**Two containers, one Ir apart — so an absolute *does* transfer, and the
forty-eighth pass first concluded the opposite and was wrong.** Pass 47's tip
`40fb5e31` read **1,645,831,968** on pass 48's box against the
**1,645,831,969** pass 47 recorded on its own. The thing that misled: pass
48's base `89f55a5c` reads 1,662,145,003, not the 1,674,581,042 pass 47's Log
records — because that Log number is pass 47's **pre-rebase** tip `3706f96f`,
and `89f55a5c` is the same seven commits *after* a concurrent session landed
pass 46's `cast_cost_scan` (-0.697 %) underneath them. The gap is that commit,
not the container. **The rule that survives is narrower and still worth
having: re-read your own base, because on a shared branch the commit you think
you are standing on may not be the one the last pass measured.**

**And argv length lands in the Ir total.** Pass 49's base columns are 492 Ir
below what passes 47 and 48 recorded for the same commits, because its
`--callgrind-out-file` name is a character shorter; pass 47 saw the same
effect at 686 Ir. `40fb5e31` was re-read directly there (1,645,831,476,
exactly 492 under pass 48's reading); `bf658313` and `04282f2e` were not, so
those bases are that 492 subtracted from pass 48's numbers. One argv
throughout a pass makes its deltas exact — the absolute transfers between
containers to within the argv string.

The three things those blocks were written to carry, which are why the
numbers can go:

- **None of passes 45-49 ran a `release` A/B.** Each row is callgrind plus
  the `--bench` invariant check, which is what this file asks for a sub-5 %
  change; the committed `release` block below is still the forty-fourth
  pass's.
- **A `profiling-fast` `games_per_s` settles nothing.** Pass 46's four
  unalternated readings ran 138.16 / 139.14 / 135.69 / 133.08 in commit
  order and then **143.36 at the rebased tip** — drift that tracks the box,
  against an Ir column falling monotonically. Pass 47 declined to quote a
  pair at all: `host_calib_ms` read 49 then 60 across the two runs it had,
  and pass 48's only `--bench` runs were taken between builds on a shared box.
- **Wide pool clean at every tip.** `--a gang --b gang --games 400
  --threads 3 --decks all`, seeds 11/12/13: 20,400 games, 20,396 decided, no
  panic, all 10,198 mirrored pairs split (`rho -1.000` every seed), and
  byte-identical across pass 48's two runs. The 4 undecided are seed 11's
  standing rules draws, the same four passes 44-48 recorded.
  `peak_rss_mib` 21.0-21.9 across both passes; `host_cpu` Intel Xeon @
  2.80GHz, `host_calib_ms` 52-57; suite 18,709 passed / 0 failed / 5 ignored
  over 22 binaries, clippy `--workspace --all-targets` clean.


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

**Crash-freedom at the sixty-second tip (`52e0b801`): clean, and identical to
the record.** `overflow` profile (`release-fast` + `overflow-checks`), `--a
gang --b gang --games 400 --threads 3 --decks all`, seeds 11 / 12 / 13:
**20,400 games, 20,396 decided, no panic and no arithmetic overflow**,
28.5-30.4 s a seed against a 10m11s build. The 4 undecided are all on seed
11, which is what every tip since the forty-second pass records.

**Run here because this tip is the first to carry both sessions' lines**,
and the other one's four commits reordered state-based actions on purpose —
`assign_sectors` and `sync_graveyard_shapeshifters` moved under
`sba_board_scan`, and the converge oracle changed which payment order two
cards take. Reordering SBAs is exactly the shape that turns into a stall or
a panic several thousand games out.

It did not, and the integration check is sharper than the grid:
`CRAB_THREAD_CHECK=1 --bench` on the same binary reads **decisions 196,220,
turns_per_game 27.53, 0 stalls (cap 0 / stuck 0 / draw 0), determinism ok
(all pairs split, rho -1.000), thread_determinism ok (3 vs 1 threads
identical)**, `host_calib_ms` 48 (in the 47-52 band). **196,220 is
byte-identical to the count at `b370d69e`, before any of the five commits.**
Five commits from two sessions, four of them behaviour-changing in the
rules, and the bench workload makes the same decisions in the same order.

Its `peak_rss_mib 27.3` is **not** comparable to the 17.7-18.3 MiB in the
Baseline block — that is an `overflow` build, and RSS is profile-dependent.

**Crash-freedom at the sixtieth tip: clean, and identical to the record.**
`overflow` profile (`release-fast` + `overflow-checks`), `--a gang --b gang
--games 400 --threads 3 --decks all`, seeds 11 / 12 / 13: **20,400 games,
20,396 decided, no panic and no arithmetic overflow**, 28-30 s a seed against
an 8m19s build. The 4 undecided are all on seed 11, which is what every tip
since the forty-second pass records. Worth running here because the pass
touched arithmetic on purpose (`ba15f249`'s `wrapping_*` mixer) and changed
three cube-pool cards' behaviour.

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

### Sixty-second pass — two questions that were asked before they were needed

This session's line, run concurrently with the sixty-first pass below; its
four commits are under neither of (A)'s columns. Two code commits and three
measurement ones. Both code commits are the same shape from two directions:
**work done to answer a question whose answer was already to hand, or whose
asking could have stopped at the first term.**

**(B) `655e1e47` — six predicates computed before the AND that could reject
on the first. `cube` -0.170 %, `sos` -0.069 %, `fixed` -0.047 %; the
function itself -28.0 % / -45.1 % / -15.7 %.** `affected_includes_gated` is
the filter body of a collect over the battlefield (236,026 calls on cube),
and four of its arms bound every predicate to a `let` and ANDed the lot at
the end. The first term is one integer compare that rejects about half the
board; below it sat two `cost.symbols` walks and a `card_types` walk that
ran regardless. The predicates are pure, so cheapest-first `&&` is the same
function.

**LLVM does not fix this for you, and that is the transferable half.** A
loop is not something it will sink past a branch it cannot prove, so the
usual "the optimizer will short-circuit pure code anyway" intuition is
wrong for exactly the terms that cost the most. Identical call counts on
every pool and the whole-program delta agreeing with the function's delta to
within 0.1 % on cube and fixed:

| pool | calls | fn before | fn after | Ir/call | fn | program |
|---|---|---|---|---|---|---|
| cube | 236,026 | 16,799,876 | 12,095,286 | 71.2 -> 51.2 | **-28.0 %** | -0.170 % |
| sos | 29,436 | 1,839,932 | 1,010,872 | 62.5 -> 34.3 | **-45.1 %** | -0.069 % |
| fixed | 79,728 | 3,561,560 | 3,003,464 | 44.7 -> 37.7 | **-15.7 %** | -0.047 % |

**The program column is what ships and it is under the clock's resolution**,
so nothing here is a claimed throughput win. The pool spread is the arm mix:
cube runs the most `AffectedPermanents::All` and gets the most absolute; sos
gets the largest fraction because its calls skew to arms where the seat
compare fails.

**How it was found is written up in "Which pool a change moves" and is the
more reusable output of this pass**: rank rows by `cube% / sos%`, not by
either share. At 0.61 % of cube this row is invisible in any top-N table;
at **5.08x** sos it is the most pool-specific thing in the profile.

**(A) `718a66f8` — the suggestion asked a global index about a card it was
holding.** One commit, base `b370d69e`. `profiling-fast
--no-default-features`, callgrind, 1 thread, `--a gang --b gang --games 6
--seed 1`.

| pool | before | after | delta |
|---|---|---|---|
| sos | 1,547,629,927 | 1,543,297,150 | **-0.280 %** |
| cube | 2,797,004,247 | 2,797,723,828 | +0.026 % |
| fixed | 1,193,591,244 | 1,193,886,624 | +0.025 % |

**Under the ~5 % rule that is flat, and the commit says so.** It lands as a
correctness/clarity change; the row is here because the *reading* that
produced it is worth more than the delta.

`NameCard`'s CR 201.4a namespace filter ranks suggestions out of a
battlefield slice or the controller's library, then — to answer "is this a
*land* card name?" — took each name it had just produced and looked it back
up in `card_registry::lookup_by_name`, and deep-cloned what came back into a
scratch `CardInstance`. Every one of those names belongs to a card the
function is already holding. It now keeps the `Vec<&CardInstance>` the
ranking was built from and tests the definition the name came from.
`definition_matches_requirement` takes `impl Into<Arc<_>>` instead of
`&CardDefinition`, so the caller hands over a refcount rather than a full
copy of an 8,232-byte struct and every `Vec` in it.

**The find was in the caller table, and it is a shape worth repeating: an
inclusive row that is enormous on a tiny call count.** `lookup_by_name` was
156 calls and ~105 M Ir on `sos`, which is not a hot function — it is a
function standing in front of a one-time `OnceLock` that builds a
`CardDefinition` for all 22,568 catalog factories to read their names.

```text
callers of lookup_by_name, --decks sos
before   114  105,337,826  <suggestion filter's try_fold>   <- built the index
          42      155,380  apply_pending_effect_answer
after     42  104,880,417  apply_pending_effect_answer      <- builds it now
```

**Which is the honest half of the result: the index build did not go away,
it moved.** The answer path validates a name a *decider* returned, which
need not be a card in any zone, so it still needs the registry. The
suggestion path no longer reaches for it. Ir moved by the deep clones only,
which is what -0.280 % is.

That 104.7 M is now written up twice — as a measurement caution in **How to
measure** (it is **6.8 % of a six-game `sos` total** and **0 % of `cube` and
`fixed`**, so those totals are not comparable to each other at the
hundred-million level) and as candidate **(-46)**, ranked deliberately low:
it is one-time per process, so a training actor amortizes it to ~0.001 %.
**A cost that is 6.8 % of the measurement and 0.001 % of the workload is a
measurement bug, not a perf candidate**, and the entry says so in its own
title so the next pass does not spend itself on it.

Slightly more correct at the edges, too: a battlefield permanent that is a
copy of something else, or a card with no registry entry, used to fail the
lookup and have its suggestion silently dropped. Ladder printouts identical
on all three pools; suite 18,735 passed / 0 failed / 5 ignored; golden
traces unchanged; `--bench` decisions 196,220, turns 27.53, 0 stalls,
determinism ok, peak RSS 18.3 MiB.

**Read the sixty-first pass below before taking anything from this one's
profile block.** It landed four commits against the same
`SpecFromIterNested` rows this pass's TODO block was about to send the next
reader at, and its device — rank `--callers SpecFromIterNested` by *calls*,
then ask whether each collect can be non-empty on the bench pools — is the
better first look. The two lines were measured against different bases and
neither total includes the other.

### Sixty-first pass — what does the answer cost when it is "no"?

The other session's line, run concurrently with passes 59 and 60; pass 60's
`6344adf6` is under neither column. Four commits, base `ba15f249`. **`--decks fixed` 1,218,195,816 ->
1,206,204,087, -0.984 %; `--decks sos` 1,593,831,453 -> 1,580,084,804,
-0.862 %; `--decks cube` 2,866,729,876 -> 2,841,539,263, -0.879 %.** The Baseline block above has
the step table, the reverted experiment, and the fifth commit this session
wrote and the first rebase dropped. What is worth keeping here:

**(A) `e0e64d12` — a `collect()` into a `Vec` that is empty on every bench
board. `fixed` -0.262 %.** `fire_combat_damage_triggers`' Phase 1.6 (the
`AnyPlayer` dealer listeners — Cabal Slaver) ran
`filter`/`flat_map`/`filter`/`map` over the whole battlefield, collected it,
and drained the collection into `by_kind` two lines later. Nothing in the
chain borrows `self` mutably, so there was never anything to buffer.
**`Map::try_fold` 6,167,108 -> 2,495,168** and the function's own self cost
went *up* 1.09 M, which is the adapter stack being inlined into it instead.
The dealer lookup that ran twice (`battlefield.iter().find(|c| c.id ==
source)` one line apart, once for the controller and once for Phase 1) runs
once.

**And the thing (A) tried first, measured, and did not take:** folding the
five kind-independent battlefield walks in that function (equipment, Auras,
soulbond, `YourControl`, `AnyPlayer`) into one pass with five
order-preserving buckets reads **-0.245 % / -0.174 %** — worse than the
one-line fix on both pools. The buckets and their five-way `chain` cost
1.14 M (`IntoIter::drop` +627 k, `drop_in_place<Chain<..>>` +274 k, self
+242 k) against walks that were never the cost. **The walks are not the
cost; the iterator built over them is.**

**(B) `7f8f94d2` — converge had two oracles and they disagreed in both
directions.** `CardDefinition::wants_converge` (the payment path's) scans
the definition's Debug rendering; `bot::card_reads_converge` (the
pre-float's) was a hand-written walk of the effect tree. The walker
enumerated fifteen `Effect` arms — so converge in any other arm, or in an
activated or triggered ability, was invisible to it, which is exactly the
rot `wants_converge`'s doc comment predicted a hand-written match would
have. And the walker was the only side that knew converge's *other*
spelling, `SelectionRequirement::ManaValueAtMostConverged`: **Bring to
Light** and **Sundering Archaic** put their converge entirely in a target
filter, so `wants_converge` said false for both and their casts took the
mana-conserving payment order. One oracle now; the walker is 45 lines
deleted and a table-driven catalog test in its place. **+142,350 Ir on
`fixed` (+0.0117 %, measured on a base binary carrying only this commit) and
+0.014 % on `sos`, all of it the second substring scan, once per card name
per process.**

**(C) `2336817d` — three whole-zone walks ran at the top of every SBA sweep.
`fixed` -0.641 %, `sos` -0.564 %, the pass's biggest commit.**
`check_state_based_actions` runs at every priority pass (13,262 times over
six games) and opened with four helpers, three of which walked
unconditionally:

* `sweep_finished_schemes` and `sweep_finished_phenomena` each `flat_map` +
  `collect` over every seat's **command zone**, which is empty for the whole
  game outside Planechase/Archenemy. One `is_empty` per seat now; the scheme
  sweep's gate goes ahead of its stack walk, whose predicate reads the
  command zones too.
* `sync_graveyard_shapeshifters` (CR 613 layer 1, Volrath's Shapeshifter)
  `filter`ed the battlefield for a definition flag `sba_board_scan` reads off
  the same definitions. It moved *under* the scan, gated on a new bit, and
  returns whether it rewrote a definition so the scan can be retaken when it
  did — the shape the two flip legs below it already use.
* `assign_sectors` (CR 704.5u) walked the battlefield with a keyword scan per
  permanent to compute its own "no sculptor and nothing designated" bail.
  Two more scan bits *are* that condition. It moves under the shapeshifter
  sync too, so a copied Space Sculptor is read from the layer-1 definition —
  the only case where the order is observable, and the new one is correct.

`check_state_based_actions` self 28,960,956 -> 24,033,162 on `fixed`;
`Vec::from_iter` attributed to it went 53,362 calls / 21,358,086 Ir to
13,576 / 15,880,576 on `sos` — **4.02 collects a sweep down to 1.02.**

**(D) `c676a229` — and (C)'s own new bit got its own keyword walk.**
`sba_board_scan` already iterates each permanent's printed keywords; the
`sculptor` bit ran a second `any` over the same list. One more match arm:
-0.095 % / -0.100 %.

**The reverted experiment, and it is the second finding.** See the Baseline
block: serving `card_type_change_unscoped`'s battlefield leg off the scan
reads +0.295 % / +0.255 %, because the standalone `any` short-circuits per
card and a scan bit cannot. Third refutation of the fusion device inside
`creature_death_possible` alone.

**How two of the four commits were found, so the next pass can repeat it:**
`python3 scripts/cg_edges.py cg.out --callers SpecFromIterNested`, ranked by
*calls*, then ask of each row "can this collect be non-empty on the bench
pools?". `check_state_based_actions` at 53,362 calls / 4.02 per sweep and
`fire_combat_damage_triggers`' leg were both in the top ten. The general
form is the pass's title: **a presence question costs the same whether the
answer is yes or no, and the bench pools answer no every time.**
### Sixtieth pass — a struct size found it, not a function row

Two commits, base `58346b57`. `sos` **-3.46 %**, `cube` -2.82 %, `fixed`
-2.16 %, `sealed` -2.33 %, **peak RSS -19 %**. Numbers in the Baseline block.

**(B) `6344adf6` — a deck-fill memcpy'd an 8,232-byte `CardDefinition` per
card. sos -2.899 %.** The device is worth writing down because no function
row says it: `__memcpy_avx_unaligned_erms` is **7.80 % of `sos`** and has
never had an entry, and its caller table is forty rows of small copies —
except one. `CardInstance::new` is **3,452 calls at 8,242 Ir each**, and
*that per-call number is the finding*: a memcpy costing eight thousand
instructions is moving kilobytes, and `size_of::<CardDefinition>()` is 8,232.
`CardInstance::new` takes `impl Into<Arc<CardDefinition>>` and every deck-fill
site handed it a fresh `f()`, so `Arc::new` copied the whole definition, once
per card in a library, on top of building it. `cube::card_arc(f)` memoizes one
`Arc` per factory per thread and the fill becomes a refcount bump.

**Read the Ir/call column of a caller table, not just the calls or the
total.** Every other row in that table is a hundred-odd Ir and genuinely
diffuse; the one row worth a commit is visible only as an outlier in the
ratio.

**Sound because the definition was already a CoW handle.** `Arc<Card
Definition>` is what `CardData` holds, and the ~twenty sites that rewrite a
definition — transform, become-a-copy, a permanent keyword grant, a colour
override — all go through `Arc::make_mut`. Four copies of a card in a deck
share one definition until one of them is rewritten, and the first writer
unshares. The RSS drop (21.9 -> 17.7 MiB on `--bench`) is the same fact from
the other side, and it is the half that matters to an actor count.

**Its own memo, and the alternative was measured.** Riding `card_brief`'s memo
made a *miss* also pay `CardBrief::of` — pip counts, the keyword walk,
`is_fixing_card`'s effect-tree walk — which read **+6.591 % on `--decks
sealed --games 1`**, a workload that is eighty template cards, all misses, and
no games. A separate memo reads +0.330 % there. **A memo whose miss path is
expensive is not a free memo**, and the workload that shows it is the cold
one.

**(A) `ba15f249` — the CR 104.4b loop watchdog ran SipHash over fifty small
integers. sos -0.578 %.** `loop_fingerprint` built its digest into a
`DefaultHasher` at ~52 Ir a `write`: **2,424 calls / 12,546,606 Ir / 0.78 % of
`sos`**, and 203,496 of the program's 204,774 `sip::Hasher::write` calls. It
runs after every triggered-ability resolution, so a trigger-heavy pool pays it
constantly and almost always to learn that the state moved. SplitMix64's
finalizer chained over the same field stream avalanches every input bit across
all 64 output bits in ten instructions, which is what the function's own doc
asks for ("a false positive would end a live game"). **The engine's vendored
`fxhash` would have been the wrong tool** — its own doc says it is not
collision-resistant, and it exists for iteration determinism in *maps*, not
for a 64-bit digest that decides a draw.

### Fifty-ninth pass — the clock gets a harness, and the arc gets measured on it

Four commits, base `7112d857` (pass 58's tip after its doc reconciliation).
Three of them are measurement and build-time work whose write-ups live in the
sections above — `ff929e7f` (`scripts/ab_wall.py`, its ABBA schedule and its
+/-2 % calibration: **How to measure**), `5c590e9a` (passes 57-59 on the
clock, Ir over-reads ~1.7x on `sos` and ~2.8x on `cube`: **How to measure**),
`e23286d9` + `49c7220d` (twenty test executables to twelve, relink **-23.5 %**:
**Build time**). Read them there; this entry exists so the fourth is on the
record with its numbers.

**`ba15f249` — the CR 104.4b loop watchdog's digest ran SipHash over fifty
small integers. `sos` -0.578 %, `cube` -0.397 %, `fixed` -0.139 %.**
`loop_fingerprint` builds the state digest — turn number, stack depth, five
fields a seat, four per battlefield permanent — into a `DefaultHasher`, i.e.
SipHash-1-3 at ~52 Ir per `write`. On `--decks sos` that was **2,424 calls for
12,546,606 Ir, 0.78 % of a six-game run**, and 203,496 of the program's
204,774 `sip::Hasher::write` calls. It runs after every triggered-ability
resolution, so a trigger-heavy pool pays it constantly and almost always to
learn that the state moved. SplitMix64's finalizer chained over the same field
stream avalanches every input bit across all 64 output bits in ten
instructions, which is exactly the property the function's own doc asks for —
"a false *negative* just means the draw isn't detected, while a false positive
would end a live game".

```text
                     base (58346b57)   tip (ba15f249)
I refs, --decks fixed  1,219,893,985   1,218,193,228   -0.139 %
I refs, --decks cube   2,878,150,309   2,866,725,942   -0.397 %
I refs, --decks sos    1,603,088,243   1,593,828,683   -0.578 %
```

**The engine's vendored `fxhash` would have been a weakening and this is
not.** `fxhash`'s own doc says it is not collision-resistant, and it is a
*map* hasher — chosen for iteration determinism, not for a 64-bit digest that
decides a draw. The digest's *values* change, so the two watch counters compare
different numbers, but only ever against numbers this same function produced
inside one process. `--bench` decisions **196,220 byte-identical**, turns/game
27.53, 0 stalls (cap 0 / stuck 0 / draw 0), determinism ok; the full ladder
printout diffs identically on `fixed`, `cube` and `sos`; suite 18,735 passed /
0 failed / 5 ignored with **golden traces unchanged**.

**Anchors at this pass's tip** (`ba15f249`), same recipe as the block in
**Baseline**: `--decks fixed` **1,218,193,228**, `--decks cube`
**2,866,725,942**, `--decks sos` **1,593,828,683**.

### Fifty-eighth pass — the shape lattice's last per-shape re-derivations

Four commits, base `c18552fd` (pass 57's tip). One workload moves: `--decks
sealed --games 1`, which plays no games and so is deck construction and
nothing else, **26,478,634 -> 23,574,309 Ir, -10.968 %**. `--decks fixed` and `--decks
sos` are flat to five decimal places at every step, which is what a
deck-builder change has to read on a pool that builds its decks once.

The pass is the fifty-sixth's question — **what actually varies with the
shape?** — asked of the three things that pass left: the splash ranker, the
per-shape sort, and the land assignment. The answer each time was "nothing",
and the fifty-sixth pass's own device (`PoolScores`, the pool's
shape-invariant per-card facts) is where all three answers now live.

**(A) `811cddec` — the splash ranker re-derived the pool's colours and every
candidate's score, thirty times per pool. -3.738 %.** `splash_cards` opened
with `colors_of_picks(pool)` and scored its survivors with
`score_card_quality(f, &pool_colors)`. Both are what `PoolScores` holds: the
scorer's colour argument *is* the pool's pip totals and the scorer is
`cfg.builder_v2`'s, so the score it computes is the base score already in
hand. `PoolScores` gains the `quality` flag it was scored under and
`build_shape` `debug_assert_eq!`s it against `cfg.builder_v2`, so mixing the
two scorers fails the suite rather than silently ranking splashes under the
wrong one.

**(B) `432976a0` — a random build re-derived the pool scores its own lattice
was built from. -0.599 %.** `build_random_deck` enumerates the lattice (which
builds a `PoolScores`) and then calls `build_random_deck_from`, which built a
second one. `selfplay::build_candidates_cfg` is the sharper case: it hoists
the lattice out of its `n`-candidate loop and then paid `PoolScores::new`
*inside* it, so `best_build_by`'s n = 32 walked the pool thirty-two extra
times.

**(C) `88b97f25` — fifty-seven shapes sorted the same pool fifty-seven times.
-2.828 %, the pass's biggest row.** With no pick jitter a pick scores `base +
fixing_bonus * is_fixing`, and `fixing_bonus` is a function of the shape's
colour *count* — 0, 1 or 3. **So the pool's descending order is one of three
permutations, not one per shape.** A stable sort commutes with a filter, so
walking the pool-wide order and skipping what a shape disallows is
byte-identical to sorting the allowed list: the shape's `allowed` answers go
into a bitmask in the pass that already evaluates them, and the copy-cap
funnel reads the order behind it. The jittered path (gauntlet and variant
builds) still sorts, and a pool wider than the bitmask falls back to it.

**(D) `13f3521c` — the land assignment looked a `CardBrief` up per leftover to
rediscover what the pile-builder already knew. D-10.968 %.** `assemble_lands`
`retain`ed over ~70 leftovers asking `card_brief(f).is_land` for each, to find
the two or three pool duals. But a land never occupies a spell slot, so
`allowed` rejects every one of them and they all land in the `off` pile —
which keeps pool order, which *is* the order the `retain` was rediscovering.
The pile-builder records `(index, produces)` per land as it goes, the
dual test becomes two `ColorSet` mask tests instead of an iterator over the
build's colours, and the removal is one gap-copy over the tail.

**(E) — the copy cap counted copies of a card in a `HashMap` keyed by the
card, per shape. -7.782 %, the pass's biggest row and it was not in the plan.**
The tip's own self-cost table put `suggest_main_deck_shape::take` at 7.85 %
(24,147 calls at **76.7 Ir**, nearly all of it the `entry()` probe) — a row
that only became visible because (C) gave the funnel a name. The counter's key
is the card, and **the pool's distinct-card partition is invariant across
every shape**, exactly like the briefs, the scores and the sort order this
pass and pass 56 hoisted. `PoolScores` gains a dense id per distinct factory,
the counter becomes a `Vec<u8>` indexed by it, and both walk paths hand `take`
a pick index instead of a factory. `take` no longer has a row at all: it
inlines back into `build_shape`, which grows 248 k against the 1.85 M that
came off.

**The generalisation, and it is the pass's whole shape in one line: every
per-shape data structure in this builder was keyed by something the *pool*
determines.** Briefs (pass 54), scores (pass 56), the sort order (C), the land
index (D) and now the copy-cap counter. When the next per-shape allocation
shows up, ask what its key varies with before costing it.

**What the pass leaves behind, by self cost of `--decks sealed --games 1` at
the tip** (`build_shape` is still the residual and still diffuse):

| row | % of 21,774,018 | note |
|---|---|---|
| `build_shape` | **24.77** | diffuse; `suggest_main_deck_shape`, `take` and `assemble_lands` all inline into it now |
| `__memcpy` | 9.49 | still mostly the twelve pools' definitions being built once |
| `score_brief_with_colors` | 6.81 | `static_build_score`'s, 684 x ~23 main cards, and `main_colors` really does vary per shape |
| allocator | ~14.1 | `_int_malloc` 5.95, `_int_free` 3.48, `malloc` 2.64, `free` ~2 |
| `Map::fold'2` | 5.88 | |
| `static_build_score` | 2.94 | 684 calls |
| `small_sort_network` | 2.74 | down from ~8 % across three sort rows: what is left is (C)'s two or three pool-wide orders, 24 `OnceCell::try_init` calls at 26,036 Ir |
| `_dl_relocate_object` | 2.52 | process startup; a training actor never pays it |
| `HashMap::insert` | 2.49 | **not the copy cap any more** — `basic_split`'s `HashMap<Color, u32>` return and `PoolScores`' one-per-pool id map |

```text
                          base (c18552fd)   (A)          (B)          (C)          tip (D)
deck build (sealed, 1)       26,478,634    25,488,835   25,336,128   24,619,696   23,574,309
```

Behaviour: `--decks sealed --games 6` and `--decks all --games 20 --seed 11`
byte-identical to the base at every step apart from the wall-clock line.
No encoding, pool, `TrainRow`, `EncodedState` or `Vocab` change: **no net
needs retraining.**

**No wall-clock pair is quoted, and the reason is arithmetic rather than
laziness.** Deck construction is ~2.2 M Ir per pool-plus-build against ~50 M
for a sealed game, and a `selfplay_train` actor builds two decks per game — so
the whole deck builder is ~8 % of an actor's per-game work and this pass is
~0.9 % of it. That is an order of magnitude under what `--bench` can
resolve on this box (an 11 % spread between runs of one binary), which is
exactly the case this file says callgrind is for.

### Fifty-seventh pass — a fan of narrow walks at the end of two big functions

Five commits, base `28ae2416`. `--decks cube` **-7.973 %**, `sos`
**-4.216 %**, `fixed` **+0.512 %**. (B) and (C) are the same shape in the
simulator's two largest engine functions — a chain of narrow per-card walks
asking a question the *board* or the *batch* has already answered — and they
land on different pools: (B) on `sos`, which has the static abilities;
(C) on `cube`, which has the `GrantTriggeredAbility` statics. (D) and (E) are
a second session's, written concurrently against the same base and rebased on
top: the gather allocated twice for every effect it emitted.

**Two sessions opened this pass on the same function, and this time both
halves survived.** The other three occasions (passes 55, 56 twice) ended with
one session's commit dropped on the rebase because the two had written the
*same* optimization. The difference here is that (B) removes walks and (D)/(E)
remove allocations, so the rebase was a conflict in `mod.rs` and not a
duplicate. **What did get dropped is the second session's own version of (B)**
— see "the gate placement (B) did not try", below, which is the one thing it
measured that (B) did not.

**(D) `603d354b` — `static_ability_to_effects` collected a `Vec` its only
caller drains. cube -1.001 %, sos -0.634 %, fixed -0.011 %.** The function
flat-mapped a permanent's static abilities into a fresh
`Vec<ContinuousEffect>` and handed it back; all three call sites are the
gather and all three `extend` it into `all_effects`. So a static-ability
permanent paid an allocation, a growth chain, a memcpy out and a free, per
gather, for a buffer that lived for the length of one `extend`. It writes
through `&mut all_effects` now and the three per-effect patches (the
`AllOpponents` team fill, the emblem duration remap, the command-zone
named-card resolve) run over `all_effects[start..]`.

The gather's callee table sized it on `28ae2416`, `--decks cube`, six games:
`SpecFromIterNested::from_iter` **65,288 calls / 39,931,402 Ir** plus 19,910 /
12,113,011 at the second arity, `IntoIter::drop` **85,198 / 6,108,635** —
exactly one per collect — and `grow_one` 44,384 / 6,455,215.

**(E) `6c5dd0ab` — thirty-three arms returned `vec![one]` per emitted effect.
cube -0.424 %, sos -0.283 %, fixed -0.189 %.** The same shape one level down:
`static_effect_to_effects` returns a `Vec` per static ability, and the arms
that emit build it with `vec![ContinuousEffect { .. }]`. The arms that emit
nothing returned `vec![]`, which allocates nothing — so the cost fell exactly
on the boards that have statics, and `--decks fixed` read -0.002 % for this
commit on the pre-(B) base.

**The pair reads larger *after* (B) than before it, on every pool** (cube
-0.937 -> -1.001, sos -0.634 -> -0.634, fixed -0.069 -> -0.200 for the two
together). That is what composition looks like when one change removes work
and the other removes allocation for the work that remains: the mask made the
gather smaller, so the allocation is a larger share of what is left. They are
also the only rows in this pass that move `--decks fixed` down.

**The gate placement (B) did not try, and it is worth ~0.25 % of `fixed`.**
(B)'s own variant table measured three placements — per-card bit alone
(fixed +0.298 %), board branch plus per-card bit (+0.551 %, taken), and a
board-wide slice swap (+1.03 %) — and attributes the board branch's 0.253 %
to "38 tests x 32,002 gathers, ~2.6 Ir each". The second session's version
measured a **fourth**: the board test *inside* the loop,
`for &card in &sa_cards { if mask & bit == 0 { break; } .. }`, which read
**+0.234 % on fixed against +1.076 % for the slice swap on the same base**,
and took cube and sos further at the same time (-1.85 -> -2.26 %, -2.75 ->
-3.16 %). On an empty `sa_cards` a test inside the loop costs nothing, which
is exactly `fixed`'s board. **The rule: a presence gate in front of a loop is
only free if the loop was not already empty.** Applied to (B)'s shape it would
be `for &(card, bits) in &sa_cards { if !sa_open(sa_mask, BIT) { break; } if
!sa_open(bits, BIT) { continue; } .. }`, which costs one extra test per card
on the passes whose board bit *is* set. **Not taken here**: it is a ~2,000-line
re-indent of thirty-eight blocks on a branch two sessions are writing to, for
0.25 % of the pool that matters least to `selfplay_train`. Take it when the
branch is quiet.

**(C) The trigger dispatcher asked every permanent about grants no event
could fire. `cube` -4.241 %, `sos` +0.098 %, `fixed` +0.155 %.**
`dispatch_triggers_for_events` walks the battlefield and evaluates every
grant filter on the board against every permanent —
`statics_granted_triggers_inner` reaching
`evaluate_requirement_static_hinted` **406,346 times for 106.6 M Ir, 3.46 %
of a cube run**. A granted trigger is only ever used inside the per-event
`event_matches_spec` check below it, so a grant whose ability no event in the
batch could match contributes nothing however many permanents match its
filter. `trigger_grants.retain(event_kind_matches(.., None))` asks the batch
first: one test per (grant, event) pair instead of one requirement evaluation
per (grant, permanent) pair. **On the bench boards no grant survives it** —
the dispatcher's 235,062 grant-walk calls go to zero, and with them the
freeze scope those calls were the reason for.

`event_kind_matches` is `event_matches_spec`'s own `(spec.kind, event)` match
with the source made optional; three of its ~170 arms read the source and
answer `true` under `None`, so the cheap question is a sound
over-approximation of the exact one **and is the same code**, which is the
difference between this and the "fuse the cheap question in" device that has
lost four times.

Two placements were measured. An `#[inline(never)]` helper, so the 170-arm
match is not inlined into the dispatcher a second time, reads **worse on all
three pools** (cube +6.98 M, sos +4.53 M, fixed +0.69 M) — the code-size
theory for the `fixed` residual is refuted. An `is_empty()` guard in front of
the `retain` is worth 0.96 M on `fixed`, where `trigger_grants` is always
empty.

**(B) Thirty-eight gather passes that re-read every static ability.**
`--decks sos` **-3.427 %**, `cube` **-2.519 %**, `fixed` **+0.558 %** — and
the same rows read -3.422 / -2.476 / +0.551 against `91f3ede3` before the
rebase onto pass 56, so the two passes compose.

**The finding is the forty-ninth pass's device at a bigger scale, and the
tooling could not see it.** `gather_continuous_effects_inner` ends in
thirty-eight passes shaped

```rust
for &card in &sa_cards {
    for sa in &card.definition.static_abilities {
        let StaticEffect::X { .. } = &sa.effect else { continue };
        ...
    }
}
```

— one per `StaticEffect` variant that needs live board state. Each walks the
same list and re-reads the same static abilities, and on a typical board none
of the thirty-eight matches. In the line profile they are **ten rows at
exactly 511,188 Ir, eight at 425,990, seven at 340,792, three at 361,704** —
the identically-costed run that says "fallback chain", and **none of them is
inside `cg_lines.py`'s default forty-five rows**. `--rows` was added first
(`fe41ae32`); `cg_edges.py` has had it since the fiftieth pass.

**This is the answer to (-40)'s open question — "the gather's `Vec::from_iter`
traffic has never been read by line" — and the line profile says the collects
were the smaller half.** At the pass-55 tip the gather is **195,923,396 Ir
self, 6.12 % of the cube run, the largest self row in the program**, and by
line the bulk of it is the thirty-eight passes' *iteration machinery*:
`macros.rs:?` 29.8 M, `non_null.rs:444` 18.7 M, `non_null.rs:1720` 15.7 M,
`mod.rs:?` 12.9 M, `macros.rs:180` 9.8 M. The two always-empty `collect()`s
pass 55's (I) took were 2 of 3.43 `from_iter`s per gather and worth -0.36 %
on `fixed`; the walks around them are worth five times that on `cube`.

**The fix.** The pre-scan that already builds `sa_cards` folds each card's
statics into a `gather_spec` bitmask, stored per card and OR'd board-wide.
A pass runs only if the board bit is set, and then skips a card whose own bit
is clear. Bits over-approximate — the five `While*` wrappers recurse, because
`active_static` peels them, and every pass still runs its own `match` — so a
set bit costs only the walk it always paid. `any_artifacts_are_equipment`,
`any_pump_per_shared_type` and `anthem_open` are three more walks the same
mask answers.

**The audit is the suite, not a re-derived list.** In a debug build `sa_open`
is unconditionally true, so every pass runs on the full list and `sa_audit`
asserts that a pass the mask closed emitted nothing. A variant the mask
forgets is a `_ => 0` in `static_effect_gather_bits`, and 18,728 tests on real
boards are what catches it — the `gated_block!` device from `bot.rs`, which
is where this shape was first made safe.

**Three variants, all built and measured on this base:**

| gate | fixed | cube | sos |
|---|---|---|---|
| per-card bit only, no board branch | +0.298 % | -1.446 % | -1.957 % |
| **board branch + per-card bit (shipped)** | **+0.551 %** | **-2.476 %** | **-3.422 %** |
| board-wide slice swap, no per-card bit | +1.03 % | -2.03 % | -3.01 % |

(All three rows are against `91f3ede3`, the pass's first base, so they are
comparable to each other; the shipped row re-reads +0.558 / -2.519 / -3.427
on `28ae2416`.)

(The third row was measured against the *previous* base, `00348ada`, before
the rebase; its `fixed` cost was localised to one line — the slice select at
**8,793,228 Ir, 39 gates x 32,002 gathers, ~7 Ir each** — and an ablation
with the select alone removed read 1,245,732,866 against 1,243,882,876, so
the select was 11.5 M of that variant's 13.4 M.)

**What the `fixed` column costs and why it is taken anyway** (numbers on
`91f3ede3`, where the variants were compared)**.** The board
branch is 3.13 M Ir of the 6.81 M (38 tests x 32,002 gathers, ~2.6 Ir each).
The other 3.68 M is diffuse codegen in a 3,300-line function and did not
localise: `#[inline(never)]` on `static_effect_gather_bits` (so the 471-arm
match is not inlined into the gather twice) reads **+1.17 M worse**, and
making the audit's `before` debug-only reads **flat**, so it is neither the
inlined match nor a live `Vec::len`. **No permanent in `--decks fixed` has a
printed static ability at all** — `sa_cards` is empty on every gather, which
is why the passes were free there and why a mask can only add the cost of
asking. `selfplay_train`'s actors play the SOS pool.

**A pool rule this pass adds, and pass 53's is its other half.** "Which pool
does the change live on" tells you where a win will show. Its converse is
that a *gate* lives on the pool that has nothing to gate: the cost of asking
lands exactly where the answer is always no. Quote all three pools for
anything that adds a per-call question, and say which pool the shipped
workload is.

### Fifty-sixth pass — ask what varies with the shape

Eight commits, base `00348ada`. The pass's finding is a question rather than
a code shape: **the sealed builder runs ~57 shapes over one pool, and almost
nothing it derives per shape actually varies with the shape.** Five commits
are that at a different level, for **-23.1 % of a twelve-pool, twelve-build
workload**; three more are the game loop paying a re-read on every call.
Per-commit numbers are in the Baseline block.

**(D) `9c9afc74` — six packs from one pool re-derived the pool's buckets six
times. Deck build -8.03 %.** `generate_sos_pack` walks the pool building the
Special Guests index list and the colour buckets before it rolls anything,
and both are pure functions of the pool; every caller that wants a sealed
pool rolls six packs from the *same* pool. `SosPacks::new(pool)` does it once
and `roll(rng)` is the old body verbatim, so the rolls draw from `rng` in the
same order and the built decks are byte-identical.

**(E) `1ca90507` — `candidate_label` allocated once per colour in the label.
-3.04 %.** `map(|c| c.to_string()).collect::<Vec<_>>().join("/")`, twice, for
a label two to seven characters long, on every one of the ~57 candidates.
`alloc::str::join_generic_copy` was 1,048 of the build's 16,997 allocations.

**(F) `a8ced063` — the builder looked the same `CardBrief` up three times per
card per shape. -3.49 %.** Once in `allowed`, once for the fixing bonus and
once inside `score_card_with_colors` — four with `builder_v2`. The scorers
gain `score_brief_*` forms that take the brief.

**(G) `3b7d2c0b` — every shape re-summed the pool's pip totals. -3.94 %.**
`suggest_main_deck_in_colors` hoists `colors_of_picks(picks)` out of its own
loop, but `picks` is the whole pool and is the *same* pool for every shape.

**(H) `708171c3` — and (G) implies the whole score. -7.35 %.** If the
scorer's colour argument is a property of the pool, then a card's score is
the same for all ~57 shapes and only the pick jitter and the fixing bonus (a
function of `colors.len()`) vary. `PoolScores` is `(brief, base score)` per
pick, built once; `score_brief_with_colors` fell from 44,849 calls to the
pool size.

**(B) `1c223827` — the requirement walker scanned the stack before it looked
at the battlefield. cube -0.090 %.** Its zone walk is a chain of lazy
`or_else` legs except the stack one, which was bound to a local above the
chain and ran on all 654,950 calls.

**(C) `02caa399` — auto-tap re-found every mana source on the battlefield,
once per pip. `fixed` -0.291 %.** Candidate (-38)'s `actions.rs:12626`,
sized off `cg_sites.py` at 0.15 % and **worth twice that** — the second data
point for that script's "the number is a floor". `ManaSourceInfo` carries the
battlefield index it was built at; the scan stays as the fallback for the one
thing that invalidates it (a mana ability that sacrifices its own source).

**(J) `4871ffb7` — the computed keyword list was cloned to be read. `fixed`
-0.280 %, cube -0.214 %, sos -0.150 %.** Eleven sites wrote
`computed_permanent(id).map(|cp| cp.keywords.to_vec())` and iterated the
result; the `to_vec` was there only because `computed_permanent` hands back
an `Arc` temporary. Bind the `Arc` first and the slice borrows out of it.
`Keyword` is payload-carrying — `PreventDamageFromMatching` holds a whole
`SelectionRequirement` tree, `Ward` a cost — so the clone was not just an
allocation. `server::view`'s is left: that one builds an owned list for the
wire.

**REFUTED by construction, not by sizing: presence gates for `has_atype` /
`has_stype`, candidate (-37)'s residue. cube +0.123 %, fixed +0.075 %, sos
+0.052 %.** Both gates were built exactly like their three siblings —
`artifact_subtype_change_in_scope` over `AddArtifactSubtype` /
`SetArtifactSubtypes` with a `card_can_change_artifact_subtypes` printed
scan, `supertype_change_in_scope` over `AddSupertype`, `ring_temptations >=
1` and Leyline of Singularity's static, with the two `debug_assert!` audits
in `gather_continuous_effects` that let the suite prove them. Clean and
worthless. **Pass 55 closed the same entry from the other side in the same
hours** (`OnceCell::try_init` down to 117,334 calls at 101 Ir), and the two
agree: **a presence gate is worth what its arm's *call count* is worth, not
what the arm costs when it is taken.** `has_ctype`'s gate paid because
`HasCreatureType` is 410,900 of the requirement walker's 654,950 calls on
cube; these two arms are rare, so what is left is layout.

**DROPPED ON THE REBASE, and it is worth recording because two sessions
found it independently in the same hours.** This pass's own (A) took
`type_gate`'s memo out from behind `layer_freeze`'s mutex — the same
observation as pass 55's (B) (`4c58c9c7`), reached from the same profile row
(`creature_type_change_in_scope`, 0.93 % self on cube, ~410 k calls at
~69-75 Ir, almost all memo hits). Pass 55's landed first and is the more
thorough of the two: it moves `depth` out as well, so `computed_permanent`,
`frozen_effects` and `layers_memoized` stop locking too, where this pass's
only helped the gates. Measured -0.589 % cube against their -0.709 %.
**The concurrency lesson, and this pass hit it three times (the mutex, the
(-37) residue, and a rebase in between): read the log before starting the
top candidate.**

### Fifty-fifth pass — the requirement walker's subtype arms stop gathering

Eleven commits, base `bf4917a5`. (A) is the pass's finding; (B) through (K)
are each a win on every pool they move — **cube -20.8 % over the pass**:

```text
                  base (bf4917a5)   (A) 8779aa9f
--decks cube        4,012,095,058    3,332,029,985   -16.95 %
--decks fixed       1,248,407,927    1,249,622,086    +0.097 %
--decks sos         1,760,442,504    1,761,529,321    +0.062 %
computed_permanent        680,960          267,116     calls
```

Wall clock, `release-fast` + mimalloc, 600 games / 1 thread / seed 1,
best-of-three alternated: cube **55.49 s -> 51.72 s, -6.8 %**; sos 31.99 ->
31.30 (-2.2 %, inside the drift); `--bench` 248.90 -> 255.08 games/s.
`decisions` 196,220 and `turns_per_game` 27.53 both sides, every ladder
outcome identical, suite 18,716 passed / 0 failed / 5 ignored.

**The Ir readings were taken twice**, once at `66304712` and again after a
rebase onto `bf4917a5` — the concurrent session moved the base under this
pass mid-flight. The deltas are the same to two decimal places (-16.92 vs
-16.95), which is the useful part: the win does not depend on that base.

**Candidate (-37), and the number it was ranked on was low.** The entry
sized the four ungated `computed()` arms at `computed_permanent`'s 4.14 % +
`compute_permanent_pass`'s 2.97 %. Read from the top instead —
`cg_edges.py --callers computed_permanent` — the requirement walker's
`OnceCell::try_init` is **413,844 calls / 605,927,621 Ir inclusive, 15.03 %
of the cube run**, against the card-type gate's 13,052 walks. Self cost
undercounts a lazy cell by everything under it.

`has_ctype` and `has_ltype` now ask
`creature_type_change_in_scope()` / `land_type_change_in_scope()` before
forcing the cell, the way `has_type` has asked `card_type_change_in_scope()`
since pass 50. Both predicates already existed and are already audited by
`gather_continuous_effects`' `debug_assert!`s in the sound direction, so the
gates cost no new enumeration to trust: `AddCreatureType` / `SetCreatureTypes`
are the only two modifications that write `subtypes.creature_types`,
`AddLandType` / `SetLandTypes` / `ReplaceBasicLandType` the only three that
write `subtypes.land_types`, and `shallow_creature_types` reads the first
pair off the *stored* set — so with none in scope, printed is computed.
`computed_permanent` drops 680,960 -> 267,116 calls; the savings land as
`gather_continuous_effects_inner` -118.5 M, `compute_permanent_pass` -74.8 M,
`affected_includes_gated` -34.5 M, `compute_permanent` -25.6 M and
-160 M across the allocator family (the `66304712` reading; the split is the
same after the rebase). **A sixth of that is allocator**, so read the wall
clock — -16.9 % Ir is -6.8 % with mimalloc, the pass-54 caveat again.

**Two things the pass got wrong first, and both are general.**

**A gate the caller memoizes per scope is not free when the caller is a
`OnceCell`.** The first shape wrapped each gate in its own
`std::cell::OnceCell<bool>`, mirroring `ct_gate`. That cost **+1.24 M Ir on
`fixed`** for nothing: `evaluate_requirement_static` evaluates *one* `req`,
its match arms are exclusive, and a composite requirement recurses into a
fresh frame — so each gate runs at most once per call and the cell is two
constructions and a branch that never pay. `ct_gate` is in the same position
and has been since pass 50. Removing the cells took the whole change from
-16.62 % to **-16.92 %** on cube and halved the `fixed` cost.

**Asking the cheap question first lost.** `computed()` answers `None` for
free when the card is not a live battlefield permanent, so gating on
`!computed_absent()` before the walk looks strictly better. Measured
**+0.066 % cube / +0.024 % fixed / +0.018 % sos** against the ungated form —
the extra atomic load costs more than the gate it skips, because inside a
freeze scope the gate is a memo hit at **69 Ir** (410,900 calls /
28,299,420). Reverted. The gate is only expensive *outside* a scope, where
it reads 356 Ir, and that is where the residual `fixed` cost is.

**Why `fixed` and `sos` pay at all.** On both pools the gate answers *true*
— those boards do carry a creature-type source — so the walk runs and
`computed()` still follows. +1.05 M creature gate + 0.49 M land gate,
against -0.84 M of `dying_snapshot`'s avoided gathers. The candidates list
called (-37) "cube pool only" and that reading is confirmed, in both
directions.

**REFUTED in the same sitting, and it is the gate rule's second half with a
new face: `board_keyword_matching`'s presence gate does *not* belong on its
frozen leg.** The unfrozen leg has asked `keyword_grant_in_scope` since pass
48; the frozen leg calls `frozen_effects()`, which returns `Some` inside a
scope but **gathers to fill the memo on the scope's first computed read** —
so a caller inside a fresh scope pays exactly the gather the gate exists to
avoid. On cube that is 10,374 of the run's 59,470 gathers, all of them
`declare_attackers_banded` / `declare_blockers`. Putting the gate there
(guarded by `!layers_memoized()`, the established idiom) measured **cube
+0.40 %, fixed +0.66 %** and was reverted.

**Why, and it is the rule to carry:** filling that memo is not waste, it is
*prepaid work for the rest of the scope*. The gate only pays where the
gather would otherwise be built and thrown away. `layers_memoized()` answers
"is it already built"; nothing answers "will anyone else in this scope read
it", and that is the question. Ask it by hand before proposing this shape
again.

**(B) The freeze scope's depth and its gate slots come out of the mutex.
`fixed` -0.276 %, `cube` -0.709 %, `sos` -0.365 % — a win on every pool,
and the only one this pass has that is not pool-shaped.** (A) left
`creature_type_change_in_scope` at 0.93 % self on cube: 410,900 calls at
**75 Ir**, almost all of them a memo hit, and a memo hit was an uncontended
`Mutex` lock/unlock around two loads. `computed_permanent` (267,116 calls),
`frozen_effects` and `layers_memoized` all opened with the same
lock-to-read-`depth`.

`LayerFreeze` is now `{ depth: AtomicU32, gates: [AtomicU8; 3], state:
Mutex<{memo, perms}> }`. `type_gate` takes no lock at all; every other site
answers "am I frozen" from the atomic and locks only when it is. Sound by
the same argument the mutex was there for in the first place — it exists to
keep `GameState: Sync` for the server's `Arc<GameState>` snapshot sink, and
both moved fields describe things a `&GameState` holder cannot change: the
depth is written only by the thread that pushed the scope, and a gate slot
caches an answer about `continuous_effects` + `battlefield`. `memo` and
`perms` stay under the lock, where a torn read would matter.

**(C) `affected_from_requirement`'s And-tree stack was a heap `Vec`.
`cube` -0.665 %, `sos` -0.338 %, `fixed` flat (+0.001 %).** `let mut walk =
vec![req]` is one allocation and — after the first `And` — one regrowth on
every call, 44,396 + 44,438 of a cube run's **1,963,140 allocations**, and
the function is one of the gather's inner helpers. A fixed eight-slot inline
stack with a `Vec` spill (which no printed card's tree reaches) walks the
same leaves. Sound because the accumulators are order-independent, which the
function's own `opponent` comment already states as a requirement.

**The table that found it is `cg_edges.py --callers __rust_alloc` ranked by
*call count*, and this file has said so since the forty-ninth pass.**
`finish_grow` is 443,589 of those 1.96 M — a fifth of every allocation in
the simulator is a `Vec` that was pushed into without reserving — and one
level up, `grow_one`'s caller table is a ranked worklist of exactly this
shape. `affected_from_requirement` was its largest engine row at 44,438;
`statics_granted_triggers_with` is next at 33,424 and is **not** the same
fix (its `out` is empty on most calls, so a `with_capacity` would allocate
where nothing does today).

**REFUTED on that second row, and the shape is worth naming: an
exactly-sized two-phase build loses when the common case is one element.**
`statics_granted_triggers_with` pushes 736-byte `TriggeredAbility` clones
into a growing `Vec`, so collecting `&TriggeredAbility` matches first and
cloning once into an exact `Vec` looked like the obvious fix — it turns a
~3 KiB first allocation and every 736-byte regrowth into 8-byte growth plus
one exact allocation. Measured **cube +0.43 %, fixed +0.12 %** and reverted.
The regrowth it removes is rare; the *second allocation* it adds is not, and
`MIN_NON_ZERO_CAP` already gives the naive `Vec` four slots on its first
push. Count the pushes per call before splitting a build in two.

**(D) `restore_payment_state` unshared the battlefield once per snapshot
entry, on a path whose common case restores nothing. `fixed` -0.121 %,
`cube` -0.114 %, `sos` -0.064 %.** `battlefield` is a `CowBox`, so **any**
`iter_mut` deep-copies the zone whenever a probe clone still shares it — and
`would_accept`'s dry run means one usually does. The loop asked for that
mutable borrow once per entry (one per permanent the payer owns), 67,750
`Arc::make_mut` calls / 24.0 M Ir on a cube run. It now asks with a shared
borrow first and takes one mutable borrow only if a flag actually moved.

Small, and on every pool, which is the point: `Arc::make_mut` is
**1,364,822 calls / 174,142 real unshares** on a cube run, and its caller
table is the CoW sharp edge's worklist. Most rows are genuine mutations;
this one was not.

**(E) `extract_power_gate` deep-cloned the requirement tree to build a
residual the caller throws away. `cube` -1.021 %, `sos` -0.442 %, `fixed`
flat.** Its inner `walk` rebuilds the tree as an owned `SelectionRequirement`
— a `Box` and a clone per node — and the last line discards all of it unless
a `PowerAtLeast` leaf set the gate. It did on **312 of 44,084 calls**. The
waste is three rows of `affected_from_requirement`'s callee table:
`walk` 19.5 M, `drop_in_place<SelectionRequirement>` 15.7 M, `__rust_dealloc`
4.8 M — **40 M Ir / 1.2 % of a cube run**, and 120,400 of the run's 168,512
`Box` allocations. `requirement_mentions_power` already existed, is
non-allocating, and answers `true` for exactly the leaf that writes the
gate, so the early return is the same answer.

**Two entries in this pass are the same shape and it is worth naming**:
(-40)'s "ask what a tick pays when the answer is nothing to do", one level
down. `walk` and `restore_payment_state` both *built the answer* before
asking whether anyone wanted it, and in both the cheap version of the
question was already written in the file.

**(F) The per-card grant walk re-found the permanent its caller was
iterating. `cube` -1.208 %, `sos` -0.018 %, `fixed` -0.034 %.**
`statics_granted_triggers_with` is **351,982 calls / ~207 M Ir inclusive**
on a cube run — 143.6 M from `dispatch_triggers_for_events`, 45.2 M from
`fire_step_triggers` — and every call ran one `battlefield_find` per grant
through `evaluate_requirement_static`'s `Target::Permanent` branch. Three of
its four call sites are `for c in self.battlefield`, i.e. they are holding
the permanent. `statics_granted_triggers_on` hands it over.

**This is pass 53's (A) at a site it did not reach**, and the device is the
same: `evaluate_requirement_static_on` has existed since that pass, with a
`debug_assert!` that the hint is the battlefield permanent it names, so the
only work here was finding the callers that could promise it. The fourth
(`statics_granted_triggers_for`, and the death-snapshot path under it) takes
a card that may be off the battlefield and keeps the plain form.

**(G) The same fix at the second site the pass-53 device did not reach.
`sos` -0.254 %, `cube` -0.205 %, `fixed` -0.028 %.** `granted_abilities_of`
takes a battlefield permanent by contract — its doc says so and all four
callers are walking the battlefield — and then asked its grant filters about
`Target::Permanent(me.id)`, one `battlefield_find` per (permanent x grant).
28,496 calls / 19.8 M Ir on a cube run, and it is the mana sweep, which is
why `sos` moves more than `cube` here.

**`cg_sites.py battlefield_find` at the tip says what is left**: 61.5 M /
2.35 % over the whole program, of which `eval.rs:3113` — `bf_hint_or_find`'s
fallback, i.e. every *remaining* unhinted requirement evaluation — is
14.2 M / 0.54 %, and `find_card_anywhere`'s first leg is 7.4 M / 0.28 %. The
number is a floor (see the tool's docstring).

**(H) The rest of the class inside the gather and the combat gates.
`cube` -0.126 %, `fixed` and `sos` flat.** Thirteen sites of the shape
`self.battlefield.iter().filter(|c| self.evaluate_requirement_static(req,
&Target::Permanent(c.id), ..))` — the caller is holding `c`. Small on its
own; taken because it is the same duplication and because
`evaluate_requirement_static_on` makes the site say what it means. The
suite is the check: its `debug_assert!` fires on any hint that is not the
live battlefield permanent it names, and all 18,728 tests pass.

**About eighty more sites of that literal shape exist**, mostly in
`effects/mod.rs`'s resolution paths. **Do not convert them by pattern
match**: several iterate graveyards or hands, where the hint would change
the *answer* (the walker's battlefield branch reads the layer view) and the
`debug_assert` only catches a site a test actually plays. Convert one only
after reading what its loop iterates, and only where the profile says it is
worth it — `eval.rs:3113`, the fallback that every unhinted evaluation
reaches, is **14.2 M / 0.54 %** for all of them together.

**(I) Two `collect()`s that build an empty `Vec` on every gather.
`fixed` -0.356 %, `sos` -0.291 %, `cube` -0.256 % — the pass's only commit
whose largest win is on the bench pool.** The gather's anthem walk chains
`sa_cards` with `emblem_anthems` (synthesized `CardInstance`s for emblems
carrying an anthem static) and `face_up_schemes` (command-zone objects whose
statics function). Both are built unconditionally, both are empty on any
board with no emblem and no face-up command object, and **an empty
`collect()` still calls `Vec::from_iter`** — 2 of the gather's 3.43
`from_iter`s per call, 59,470 gathers. A two-player `is_empty` walk in front
of each is cheaper than the call it skips.

**That is how to read (-40)'s `from_iter` row**, and it took one guess and
one measurement rather than a line profile: the row is 204,138 calls /
117.3 M inclusive, and the question to ask of a collect inside a hot
function is not "how big is it" but "how often is it empty".

**(J) The state-based-action sweep asked "has anyone won" with two `Vec`s
and a sort. `sos` -0.422 %, `fixed` -0.369 %, `cube` -0.291 %.** The
game-over block collected the alive seats, mapped them to teams, collected
that, sorted and deduped it — on **every** sweep, and the sweep runs on
every priority pass (9,206 of them over six bench games, six `from_iter`s
apiece). The question is only "does more than one team still have an
uneliminated seat", which one walk answers and which is `true` for every
sweep but a game's last.

The reported `winner` is unchanged and the reason is worth writing down:
when one team survives, every alive seat is on it, so "lowest alive seat on
the winning team" *is* "first alive seat in seat order", which the walk has
already found.

**(K) The same question of combat's per-attacker collects. -0.069 % /
-0.062 % / -0.051 %, and it is where the class stops paying.** Three
`collect()`s run once per declared attacker (`AttackCostBounce`'s filters,
`AttackCostSacrifice`'s costs) or once per declaration
(`tap_another_filters`, which is also asked per declared *blocker*), and all
three are empty on every board these pools play. They exist only so the
borrow of the computed keyword slice ends before the `&mut self` below, so
the fix is a presence scan over that same slice.

**Recorded at 0.06 %, which is the useful part**: the `from_iter` rows left
after (I)-(K) are `compute_permanent_pass`'s `sorted` (75,260, and it is
never empty — it is the layer walk's own input) and a tail of sub-0.1 %
sites. The always-empty-collect class is worked out on these three pools.

**Not left for the taker: the other two arms are worth nothing.** `has_atype`
and `has_stype` stayed ungated, and the residual was sized at the pass's tip
rather than guessed: the requirement walker's `OnceCell::try_init` is
**117,334 calls at 101 Ir**, against 581,256 at 1,084 Ir at the pass's base,
and `computed_permanent` no longer appears in its caller table. The two arms
(A) gated were the whole of it. See (-37), now closed.

### Fifty-fourth pass — deck construction, read from the top for the first time

Nine commits. Seven on one workload: `--decks sealed --games 1`, which plays
no games and so is deck construction and nothing else, **111,755,559 ->
34,511,759 Ir, -69.12 %**; then two on the engine, from a measuring device
the first seven made necessary (see **THE DEVICE** below). A `selfplay_train` actor builds two pools and two decks per
game, so this is ~18.6 M of per-game work becoming ~5.8 M against ~48 M for
the game itself: **~19 % off an actor's per-game total**, and none of it is
visible on `--bench`.

The pass's device is the fifty-third's ranking rule applied one level down:
**a definition is memoized, but everything read off it is not.** Four of the
seven commits are that — pip counts, mana value, card types, card quality,
the fixing walk and the land's produced colours were all re-derived at every
read, per (pick x candidate x colour shape).

**(A) `3c154e8d` — `colors_of_picks` returned a `HashMap<Color, u32>`.
-6.186 %.** An allocation per scored pile and a hash probe per pip read in
the scorer's inner loop, 12.1 % of the build on its own. `ColorCounts` is
`[u32; 5]`, WUBRG-indexed, `Copy`. It also takes a `HashMap` iteration order
out of `top_two_colors`' tie-break (bucket order -> WUBRG); the one- and
zero-colour fallbacks were already "first WUBRG colour that is not the
primary", so only exact ties move, and no pool's ladder output does.

**(B) `5489b9fa` — both shape rankers rebuild the pip totals they hold.
-9.516 %.** `static_build_score` opens with `colors_of_picks(main)` and then
walks `main` five more times, once per colour with a `card_def` and a
`colored_pip_count` per card, to rebuild the same five numbers for the
consistency penalty. A colour absent from a cost contributes zero pips to
it, so the two sums are equal by construction. ~115 lookups and five list
walks off every ranked shape, and `enumerate_candidates` ranks 26 a build.

**(C) `b10fdebd` — `card_def`'s map probe was 20.8 % of the build.
-25.67 %, the pass's largest.** 487,071 lookups at ~40 Ir: TLS access,
`RefCell` borrow, hashbrown SIMD scan. Two earlier passes took everything
*around* the probe (a `const` TLS at `16f03d27`, a leaked `&'static` at
`867de7bb`) and neither moved it, which is what said the probe itself was
the cost. A **4,096-slot direct-mapped front cache** of `Cell<usize>` keys
and `Cell<Option<&'static _>>` values: a hit is a multiply-shift, an array
load and a compare. The map stays the authority, so a slot collision costs a
probe and not a rebuilt definition; a pool asks for ~90 (sealed) to ~309
(cube) distinct factories against 4,096 slots. Key 0 is the empty slot — a
`fn` pointer is never null — so the hit path has no `unwrap`.

**(D) `5ca71f05` — a two-colour card's cost was walked three times to count
its pips. -7.335 %.** `colored_pip_count` answers for one colour, so a
caller wanting the distribution asks `colors_of_cost` for the set (one walk)
and then walks the cost again per colour in it. `pip_counts` walks the
symbols once.

**(E) `9cc1175c` — the derived-facts memo. -33.29 %, and it is the shape to
copy.** `CardBrief` holds, per factory: the pip distribution, the mana
value, three card-type flags, `card_quality`, `is_fixing_card`. The front
cache holds it instead of the bare definition and `card_def` is
`card_brief(f).def`. It is sound for the same reason the definition memo is
— a leaked definition is never mutated and every field is a pure function of
it. What it removes is not one row but a class: `card_types` is a
`Vec<CardType>` the scorer scans four times a card, `cmc` and the pips each
walk the cost, `card_quality` walks the keyword list, and `is_fixing_card`
walks the whole effect tree.

**(F) `735e365d` — the builder's four piles grew a doubling at a time.
-12.95 %.** `build_shape` was the largest `RawVec::grow_one` caller in the
build: 11,281 growths over 312 calls, 4.73 M inclusive, 10.9 %. They are all
in `suggest_main_deck_in_colors`, whose piles partition sets whose sizes are
known before the loop. Not the `GrowVec` shape refuted at the forty-eighth
pass (+0.050 %): nothing here is cloned per checkpoint.

**(G) `ec138369` — `land_produced_colors` allocated a `Vec<Color>` per land
per shape. -8.12 %.** The caller does a `filter().count()` and a `len()` on
it. It is a `ColorSet` now and a `CardBrief` field.

**REFUTED, and do not rebuild it: iterating `ColorCounts` by zipping the
array instead of indexing it, +2.88 %** (34,861,499 -> 35,864,023). The
reasoning was that `Color::ALL.into_iter().map(|c| (c, self.get(c)))`
re-derives an index from the discriminant per element and leaves a bounds
check the scorer's inner loop pays five times a card, and `index_range.rs`
was 1.49 % under `score_card_with_colors`. `zip(self.0)` copies the
twenty-byte array into the iterator at every `iter()` call, and `is_empty`
as an array compare loses the short-circuit. Reverted.

**(H) `25438a8b` — the phasing gate gathered the effect set it exists to
avoid. `fixed` -0.136 %.** `do_phasing` opens a freeze scope and asks
`board_keyword_in_scope` whether any permanent can carry Phasing — a gate
whose whole point is to skip the whole-board layer pass on the ~every board
that has none. Inside a scope that gate reads `frozen_effects()`, which
**gathers** on the scope's first computed read, so the step paid a full
gather every turn to prove a negative and then read nothing else from the
memo it had just built. Hoisted outside the scope, where `frozen_effects()`
is `None` and `keyword_grant_in_scope` answers off printed shapes — the
shape `do_untap` and `process_cumulative_upkeep` already use. **The clause
that makes it a win here and a loss at `declare_attackers_banded` /
`declare_blockers` (+0.30 %, forty-eighth pass) is "nothing else in the
scope reads the memo"**: those two run a `compute_permanents` afterwards
either way. Gathers under `frozen_effects` 8,364 -> 6,600.

**(I) `e1cbc390` — the gather's own buffer reallocated on its first static
ability. `fixed` -0.169 %.** `gather_continuous_effects_inner` opened with
`(*self.continuous_effects).clone()`, and `Vec::clone` hands back
`capacity == len`: 10,040 `grow_one` calls over 32,002 gathers, 3.53 M Ir.
The `sa_cards` walk moves above the buffer so the buffer can be sized off
it — `sa_cards` is where every further push comes from, and it is empty on a
vanilla board. **A blanket `+ battlefield.len()` headroom instead measured
+1.54 %**, which is the forty-eighth pass's `GrowVec` refutation again: the
reserve has to be where the pushes are, not where the clone is.

**THE DEVICE, and it is the reusable part of this pass:
`scripts/cg_contexts.py` over `valgrind --separate-callers=N`.** A one-level
caller table says `computed_permanent` called
`gather_continuous_effects_inner` 20,374 times and stops at the level where
every caller looks alike. `--separate-callers=3` gives one entry per calling
context, so the 33,766 gathers rank by *whose they are*: `do_phasing` was
1,764 of them and nothing else in this file could have said so. It costs no
run time and roughly doubles the dump. The same table over `__memcpy` puts
16 % of the program's 417,679 copies in one context (`finalize_cast` under
the cast path), which is where (-28) should be re-read from.

**REFUTED after the pass's own commits, and it corrects a ranking rule this
file has been quoting since the forty-eighth: a freeze scope around
`eval_material_inner`'s board walk, +0.018 % and zero gathers removed.**
`computed_permanent`'s caller table said `bot::permanent_value_with` was
13,792 calls at **1,476 Ir each**, and the rule reads "Ir/call is the tell —
~2,000 is a gather, ~300 a memo hit", so that looked like a walk paying a
whole-game gather per permanent. It is not: `eval_material` already runs
inside an outer scope, and `gather_continuous_effects_inner`'s call count is
**33,766 before and 33,766 after** — byte-identical. What the 1,476 buys is
`apply_layers_one` per permanent, which a memo hit does not avoid (this file
already records that it "spans ~760 to ~2,200 Ir").

**So the tell is unreliable and the count is not.** Before costing a freeze
scope, read `cg_edges.py --callers gather_continuous_effects_inner` and
check the total actually moves; a high Ir/call can be a memo hit plus a
layer pass just as easily as a gather. At the fifty-fourth tip 20,374 of
`computed_permanent`'s 93,918 calls gather, for 40,374,824 Ir / 3.22 %, and
which callers those 20,374 belong to is **not answerable from a one-level
caller table** — that is the open question this entry leaves.

**What is left of the build, at 34.9 M.** `score_card_with_colors` 12.3 %
(44,849 calls at ~74 Ir, and the refutation above is what a first attempt on
it costs), the allocator family ~11 %, `build_shape`'s residual ~12 %,
`generate_sos_pack` ~4 % (pool generation, not the builder — its
`guests.contains(&i)` is a linear scan per pool card over a list it built a
`HashSet` for and dropped).

**The `--decks fixed` drift across the pass is +0.13 %** (1,250,405,745 ->
1,252,028,493). `fixed` builds its four hand-written decks once and reaches
none of this code; the movement is code layout, and it is spread over the
commits rather than concentrated in one.

### Fifty-third pass — the bench's own pool cannot see the two largest costs in the simulator

Ten commits, three classes, and the pass's real finding is the measuring
device: **`--decks fixed` is blind to the grant/layer path and to deck
construction, and those were 49 % of a cube game and 96 % of a deck build.**
Read "Which pool a change moves" above before ranking anything from a
`fixed` profile again.

**(A) `9bf2ae2e` — the requirement walker re-finds a permanent its caller is
holding. `fixed` -0.642 %.** `evaluate_requirement_static`'s
`Target::Permanent` branch opens by locating the object; the battlefield leg
of that is a linear `battlefield_find`, and **that one source line
(`eval.rs:3271`) is 18,188,014 Ir / 1.72 % of the program** — the second
largest line in the whole profile. `battlefield_find` altogether is
**4.03 %** and has never appeared as a function row, because it always
inlines. The callers that dominate it are *walking the battlefield* when
they ask: `auto_targets_for_effect_all_slots` is 113,726 of the 182,016
calls and its candidate loop is
`battlefield.iter().filter(|c| is_legal(&Target::Permanent(c.id)))`.
`evaluate_requirement_static_on(req, card, ..)` takes the permanent; the
shared body threads it as a hint that stands in for the walk, a mismatched
id falls back to the walk, and a `debug_assert` in the wrapper checks the
hint really is the battlefield permanent it names.

**(B) `3d29f9c4` — the same conversion at eighteen more walks, and it is
bench-dead.** `fixed` +2,181 Ir (nothing), `cube` **-0.078 %**. The vanilla
archetypes reach almost none of those selectors. Kept on the cube number
and on the fact that a hint compare cannot cost more than the walk it
replaces — and recorded here so nobody re-derives it.

**(C) `36e998aa` + `fdac88df` + `1ba3e76b` — three per-card grant walks, one
freeze scope each. `cube` -49.10 %, `fixed` +0.11 %.** The class:
`statics_granted_triggers_with(card, grants)` evaluates each grant's
`SelectionRequirement` against `card`, the requirement reads the *computed*
type line (CR 613), and outside a freeze scope every such read re-gathers
the whole game's continuous effects. Three loops do it per battlefield
permanent — `dispatch_triggers_for_events`' phase 1 (59.6 % of the cube
program inclusive), `fire_step_triggers` (21.7 %), `fire_spell_cast_
triggers` (3.05 %). Each already holds a shared borrow of
`self.battlefield` for its whole body, **so the borrow checker has already
proved no `&mut self` call happens inside it**, which is exactly the
invariant a freeze scope needs.

Three forms were measured on the first one and the choice matters:

| form | fixed | cube |
|---|---|---|
| `with_frozen_layers` closure | +0.945 % | -35.64 % |
| bare `freeze_layers_push`/`pop`, ungated | +0.303 % | -35.84 % |
| **bare push/pop, gated on the loop's own `no_grants`** | **+0.081 %** | **-35.884 %** |

The closure costs ~0.9 % because every captured local in the loop then goes
through the closure environment; the gate takes the two lock pairs per
dispatch off the board that cannot use them. **This is the first win the
scope-widening route ((-22)) has produced at this size, and it is only
visible on a pool with grants in it.** Gathers on cube fell 469,116 ->
187,220 at the first commit alone.

**Left alone on purpose**: `combat.rs`'s `static_granted_triggers_of` and
`stack.rs`'s ETB gather each ask about *one* card, so a scope there buys one
gather and pays for one — the forty-seventh pass's rule in its other
direction.

**(D) `67809f9f` — a deck build costs five games, and 43 % of it is memcpy.
Deck construction -93.95 %, `--decks sealed` -42.78 %, the real training
loop 26.1 -> 85.6 games/s.** `CardFactory` is `fn() -> CardDefinition` and
every call materialises the whole thing — several `Vec`s, an effect tree, a
mana cost — so a caller that wants the card's *name* pays a full
construction and a full drop for it. The deck builders call one per
(pick x candidate x colour shape).

The measurement that names it: **`bot_ladder --decks sealed --games 1` plays
no games at all and still runs 2,910,408,580 Ir.** That is twelve sealed
pools and twelve heuristic builds, 242.5 M apiece, against 48.4 M for a
sealed game; its own profile is 43.31 % `__memcpy`, 4.92 %
`drop_in_place<CardDefinition>` and ~22 % allocator. The ladder amortises it
over six games an archetype; `selfplay_train`'s `actor_loop` does not — it
calls `sealed_pool` twice and `build` twice **per game** (and 32 candidate
builds a side under `--deck-judge`).

`cube::card_def(f)` memoizes the construction per thread, keyed by the
function pointer: two factories the linker folds to one address have
identical machine code and so build identical definitions, which makes a
fold unobservable. It returns `Arc<CardDefinition>` and the cache holds a
reference forever, so every `Arc::make_mut` on one clones first and a game
can never write through to the cached copy.

**(E) `16f03d27` — the residual: a colour list that allocates and a lazy TLS
check. Deck construction -32.80 % on top of (D).** `LocalKey::with` was
18.23 % of what was left (487,071 `card_def` calls in one twelve-deck build,
each doing a lazy-init check) and `Vec::from_iter` 12.84 % —
`colors_of_cost` returned a `Vec<Color>`, i.e. an allocation per pool card
per colour shape, 253,333 of them. `const`-initialized `thread_local!` and
`ColorSet`. Across (D) and (E): **242.5 M -> 9.9 M Ir per pool+build, 24.5x.**

**(F2) `867de7bb` — the memo hands back a leaked `&'static`, not an `Arc`.
Deck construction -5.51 % again.** The cached definition is already immortal,
so the `Arc` was buying nothing and costing an atomic pair per lookup over
487,071 lookups a build. The bound on leaking is one definition per factory
*actually asked for* — the pool in play, not the 22.5 k catalog, because the
deck builders are the only callers.

**(H) `d1b4081f` — the judged builder enumerated the same shape lattice
thirty-two times. The judged training loop 25.8 -> 83.2 games/s, and 1.2 ->
83.2 from the pass base — 69x.** `build_random_deck` opens with
`enumerate_candidates(pulls, cfg)` (~26 `build_shape` calls) and uses it only
to pick one shape by softmax; `build_candidates_cfg` calls it `n` times on the
identical pool, and it is **deterministic in `(pulls, cfg)`** — its rng is
seeded from `cfg.seed` and `noise = 0` means the lattice never varies. At
`--use-deck-best`'s n = 32 per side per game (and `recommend_pool`'s 512),
~26n `build_shape` calls become ~26 + n. The soundness condition is a
`debug_assert` that re-derives the lattice after the loop and compares, so a
later change giving `enumerate_candidates` hidden state fails the suite rather
than changing every judged build silently.

**And the reason this took fifty-three passes to find: it is invisible on
every workload this file measures.** `--decks fixed` never builds a deck per
game; `--decks sealed` builds one (n = 1, where the hoist is a no-op); the
`--use-deck-best` path needs a deck net, and every committed one fails to
load. It cost one throwaway 20-step training run to make it measurable.

**(G) `4a951123` — two helpers that re-find a permanent the caller already
walked past. `fixed` -0.611 %, and every pool moves** (sealed -0.568 %, cube
-0.524 %, sos -0.424 %). `all_damage_to_player_prevented` collected the
controlled ids into a `Vec` and then `battlefield_find`'d each one back,
inside a method that is `&self` throughout; `bot::permanent_value` opens with
`battlefield_find` and `eval_material_inner` — 31,666 of its 34,892 calls —
asks it from inside `for c in &state.battlefield`. **The delta is bigger than
the two line rows predicted (0.35 %), because the line profile charges each
site only its *own* instructions: the scan's loads and the `Arc` deref per
element land in `slice::iter`'s rows.** Read a `battlefield_find` row as a
floor, not an estimate.

**What is left of the deck builder, for whoever takes it next**: 112 M for
twelve builds, of which the memo's own lookup is still ~25 M — a hash probe
per read over 487,071 reads a build. The structural answer is for
the builder to resolve a pool's definitions **once** into a
`Vec<Arc<CardDefinition>>` and index it, rather than looking each up by
function pointer; that is a signature change across `draft.rs` /
`recommend.rs` and was not attempted here. It is worth ~4 % of an actor's
per-game work, not more.

**Behaviour, all ten commits.** `--bench` byte-identical at every step
(196,220 decisions, 27.53 turns/game, 0 stalls, determinism ok); the full
`--decks cube` / `--decks sos` / `--decks sealed` ladder output at the tip
diffs **identically** against the pass base, so the decks a seed builds and
the games they play are unchanged; suite 18,712 passed / 0 failed / 5
ignored over 22 binaries at each commit. **No net needs retraining** — no
encoding, pool, `TrainRow`, `EncodedState` or `Vocab` change is in the pass.

### Fifty-second pass — the picker adopts on its own side, and the driver was where the second run was

Base `b906be3b` (pass 51's tip), read directly at **1,314,289,790** — 787
Ir under pass 51's recorded 1,314,290,577 for the same commit (argv
length again; my `--callgrind-out-file` name is shorter). All readings
`profiling-fast --no-default-features`, callgrind, `--a gang --b gang
--games 6 --threads 1 --seed 1 --decks fixed`. Same recipe as pass 50.

| step | before -> after | what |
|---|---|---|
| A | 1,314,289,790 -> 1,279,629,727 (**-2.637 %**) | `Bot::next_action_settled` — `main_phase_action_with`'s finalist path hands out the `settled` state its `accept_on` produced; the self-play driver adopts it via `g = *settled` instead of running the same action a second time |
| B | 1,279,629,727 -> 1,274,999,328 (**-0.362 %**) | dispatch dead-work: fuse three `for ev in events` tail loops + gate the delayed-trigger halves on `!self.delayed_triggers.is_empty()`, presence-gate the exile walk on `PermanentExiled` and the hand walk on `CardPutIntoHandFromGraveyard`, hoist `declare_attackers_banded`'s `trigger_grant_sources` + `equip_granted_trigger_sources` out of the attacker loop |
| C | 1,274,999,328 -> 1,265,410,851 (**-0.752 %**) | picker adopts: `pick_stack_response`, `pick_combat_trick`, `pick_land_to_play` (from hand, graveyard, and the impulse-exile land) and `legacy_pretap` thread `Probed`'s state through to `BotStep` |

**The class this pass sits in is pass 50's, and this is the second half of
it: `Bot::next_action` was where the state was thrown away.** Pass 50
adopted at the finalist level *inside* `main_phase_action_with` and at
the sim boundary. It said the last row was blocked on the driver's live
decider — that adopting across the driver "means handing it across
`Bot::next_action` and the blocker is the decider". It is not.

`Clone for GameState` reconstructs `decider` through
`self.decider.kind().into_boxed()`, and `DeciderKind` is Serde — for
`AutoDecider` (stateless) and `ScriptedDecider` (Vec<DecisionAnswer> +
Vec<Decision>) both, the clone preserves the answer queue exactly. So
adopting the probe's state, whose decider was cloned via `kind()`,
replaces the driver's live decider with a *different box holding the same
queue state at the same position*. The doc comment saying otherwise was
older than the `ScriptedDecider` `kind()` override. The self-play driver
is `AutoDecider` and never sees any of this either way; the interactive
server keeps calling `next_action` (plain form) and never adopts, so its
`perform_action` still runs and its event list still broadcasts. The
adoption is scoped to the driver in `play_one_game_traced` that owns its
state and discards events.

**(A) is where 516 of the 2,036 finalist probes came off the driver.**
Pass 50's Log recorded 2,036 `accept_on` calls from `main_phase_action_with`
and said the driver then re-ran them; my measurement here:
`GameState::perform_action` from `sim_step` unchanged (this is `sim_step`
itself, not the finalist), while the driver's edge (`perform_action` from
its own name) drops from 26,502 / 291.6 M to 25,986 / 259.1 M — 516
skipped calls at ~63 K Ir each = 32.5 M off the program. The other 1,520
probes are losing finalists whose `settled` is dropped in
`pick_by_outcome` and pre-validated (`ok=true`) winners that reached the
return with `settled: None` — a post-hoc `accept_on` for the second class
was tried and measured a dead lift (0 additional `accept_on` calls; every
reachable winner had `settled: Some`). Both are for a future pass that
either probes lazily or threads `Option<Box<GameState>>` through
`cast_candidates`.

**(B) is four gates and one hoist.** All read as one thing — the caller
of `dispatch_triggers_for_events` and `declare_attackers_banded` on a
board that has nothing to say wants the walk to notice. The three tail
loops in dispatch each iterated `events` once looking for one narrow
kind; the exile/hand walks each walked their zone's cards to gate on
`triggered_abilities` that never matched their one event kind; the
attacker loop did the whole battlefield's `trigger_grant_sources` scan
per attacker where its answer is the same for all. The delayed-trigger
side of the fusion is a presence gate: `fire_life_gained_watchers` and
`fire_opponent_graveyard_watchers` both `collect()` an empty
`delayed_triggers.filter()` on every matching event, which is one
allocation-and-drop for nothing.

**(C) is what falls out once `Bot::next_action_settled` exists.** The
`Picked` enum already carried `Probed(GameAction, Box<GameState>)` — the
sim adopts it — but every pass through `Bot::next_action`'s consumers
called `.map(Picked::action)` on it, dropping the state. Now they call
`.map(Picked::into_step)`. `pick_stack_response`, `pick_combat_trick`
and the ones with an inline `would_accept_on` (`pick_land_to_play`, the
graveyard replay, Impulse-exile PlayLand and `legacy_pretap`) switch to
`accept_on` — same probe, different return — and the driver skips
another 524 `perform_action` calls.

**The pass on the branch: `1,314,289,790 -> 1,265,410,851`, -48,878,939 /
-3.719 %.**

**What is left of the class, and why the obvious next step is bench-dead.**
The pre-validated `cast_candidates` finalists (from the ~30 `castable.push`
blocks with a `would_accept_on` gate) discard state at build time and carry
`ok=true` back with no state; in the finalist loop `ok=true` finalists get
`settled: None`. Capturing them means changing `cast_candidates`' return
from `Vec<(GameAction, bool)>` to `Vec<(GameAction, Option<Box<GameState>>)>`
(swap each block's `would_accept_on` — which is already `accept_on(...)
.is_some()`, so the state is *already built and dropped* — to
`accept_on(...).map(Box::new)`) across four call sites. **But on `--decks
fixed` this gains nothing, checked by construction:** the archetype decks
are vanilla (bolt/shock/bears/drakes), so every `castable.push` block —
delve, kicker, prototype, split, alt-cost, splice, gy-recast, gift, spree,
impulse — has no card to fire on, `castable` is empty, and every candidate
goes through `unvalidated`/lazy, so the scored winner already carries
`settled: Some`. The pre-validated-winner count on the fixed bench is **0**.
The refactor's ceiling (~1,520 adoptions × ~63 K Ir ≈ 7 %) is real only on
`all`/`cube`/`sos`, which are not the throughput bench. Do not run it for
the fixed-Ir number.

### Fiftieth pass — the dry run *is* the action, and the simulator was paying for it twice

Base `e7b3b3d4` (pass 49's tip), read directly at **1,531,246,782** — eleven Ir
under pass 49's recorded 1,531,246,793, which is argv length again (this run's
`--callgrind-out-file` name is a character shorter; see **Baseline**). All
readings `profiling-fast --no-default-features`, callgrind, `--a gang --b gang
--games 6 --threads 1 --seed 1 --decks fixed`. Rebased onto pass 50's
concurrent run (`4107e017`, `49fce1ff`) after measuring; those two commits
touch only `scripts/*.py` and the trackers, so both numbers stand unrederived.

| step | before -> after | what |
|---|---|---|
| A | 1,531,246,782 -> 1,424,690,649 (**-6.958 %**) | the attack/block sims adopt the state their own dry run produced instead of re-running the cast |
| B | 1,424,690,649 -> 1,395,881,928 (**-2.022 %**) | the two validating pickers the sim shares with the real path hand their probe's state out too |
| C | 1,395,881,928 -> 1,330,233,580 (**-4.703 %**) | the finalist carries its probe's state to the outcome eval and the summon-sick gate |
| — | 1,330,233,580 -> 1,330,231,550 | a clippy `collapsible_if` on (C)'s loyalty-finalist filter; 2,030 Ir, not a fourth optimization |
| D | 1,330,231,550 -> 1,318,590,025 (**-0.875 %**) | the probe template is built on first use, and the last consumer that forced it every tick goes behind a mask bit |
| E | 1,318,590,025 -> **1,314,421,002** (**-0.316 %**) | the layer-4 card-type presence gate joins the freeze scope's memo |

**The class, and it is the largest one this file has named.**
`would_accept_on` clones the state and runs the action **to completion** —
5,260 calls, `perform_action_inner` 242,970,273 Ir under them, **15.87 % of
the program** — and then drops the result. Every caller then performs the
identical action on a state equal to the one the probe started from. The
simulator was paying for two casts per simulated cast.

Where the 5,260 probes were, at the base tip:

| caller | probes | Ir | owns its state? |
|---|---|---|---|
| `main_phase_action_with` | 2,036 | 101,010,033 | no — the action goes to the driver through `Bot::next_action` |
| `sim_spell_action_inner` | 1,552 | 116,543,666 | **yes** — the attack sim's own throwaway clone |
| `pick_land_to_play` | 934 | 12,335,382 | no |
| `pick_stack_response` | 440 | 26,844,876 | mixed (sim + real) |
| `pick_combat_trick` | 290 | 16,640,512 | mixed |

(A) takes the second row. `GameState::accept_on` is `would_accept_on`'s body
returning `Some(probe)` instead of `true` — plus the trailing
`clear_stale_target_suppression` that `perform_action` does and
`perform_action_inner` does not, so the state handed back is what a
checkpointed action would have left. `would_accept_on` delegates to it, so
there is one body, not two. `sim_spell_action` returns `SimSpell::Advanced`
with that state and the two sim loops assign it to `g` and `continue` instead
of calling `sim_step`.

**Why the states are equal, which is the whole argument.** The probe is
`g.clone()` then `perform_action_inner(a)`; `sim_step` was `g.clone()` (the
checkpoint) then `perform_action_inner(a)` on `g`. `Clone` reconstructs three
fields rather than copying them — `decider` fresh-by-kind, `in_layer_gather`
false, `layer_freeze` unfrozen — and all three already hold for the sim's `g`,
because `sim_start_state` built it by cloning too and the freeze scope
`sim_spell_action` opens is popped before the loop resumes. The fourth
difference was real and is paid for explicitly: `perform_action` ends with
`clear_stale_target_suppression`, so `accept_on` does.

Measured, and it is exactly the predicted removal:

```text
sim_step -> perform_action    4,568 calls / 209,220,325  ->  3,100 / 102,091,809
sim_spell_action_inner probes 1,552 / 116,543,666        ->  1,552 / 111,211,472
main_phase_action_with probes 2,036 / 101,010,033        ->  2,036 /  95,706,307
```

1,468 fewer checkpointed casts, 107,128,516 Ir off that one edge against
106,556,133 off the program. The two probe rows each came down ~5 M as well:
the accepted candidate no longer needs `a.clone()` to survive the probe.
`--bench --threads 3` invariants byte-identical: decisions **196,220**,
turns_per_game 27.53, stalls 0 (cap 0 / stuck 0 / draw 0), determinism ok.

**(B) is the same trade in the two pickers the sim shares with the real
path.** `pick_stack_response` and `pick_combat_trick` probe with
`state.would_accept(a)` and return only the action; ~80 % and ~94 % of those
probes are the sim's. Both now return `Picked` — `Probed(action, state)` when
a dry run validated it, `Plain(action)` when nothing ran — and `Picked::action`
is how the four real-game call sites drop the state they must not adopt.
`GameState::accept` is `accept_on` against `self`, so there is still one body.
`sim_step -> perform_action` falls again, 3,100 calls / 102,091,809 Ir to
**2,636 / 72,020,298**: 464 more casts run once instead of twice, at ~64,800
Ir each. The `a.clone()` that (A) had removed comes back — `Probed` carries
the action for the real path — and costs ~75 Ir on 1,552 candidates, i.e.
nothing against the 30 M removed. Invariants byte-identical again; traces
unchanged.

**(C) is the same trade a level up, and it is where the class stops being
about the simulator.** `main_phase_action_with` probes up to `EVAL_TOP = 3`
finalists with `accept_on`, and then the winner is run **twice more** on a
clone of the same state: once by `evaluate_action_sequence` (`state.clone()`
then `dry_run(action)`) and once by `improves_this_turn` (the same two lines).
A `Finalist` now carries `settled` — its probe's result — and both consumers
clone *that* instead. Both edges came off exactly:

```text
evaluate_action_sequence -> perform_action_inner  2,598 / 62,218,246 -> 1,756 / 22,156,413
main_phase_action_with   -> perform_action_inner  1,514 / 42,673,732 -> 1,040 / 14,712,770
```

-842 and -474 calls, 68.0 M off the two edges against 65.6 M off the program:
the 1,316 `settled` clones are the ~2.9 M difference, at ~2,200 Ir against a
~46,000 Ir cast. `evaluate_action_outcome` ignores `settled` when
`w.determinize > 0` — the redeal makes it the wrong state, and every finalist
has to be judged against the same one — so a determinized profile keeps the
old path.

**One field differs, and it is the reused state that is right.** `accept_on`
ends with `clear_stale_target_suppression`, which `dry_run` does not, so the
reused state has `suppress_extra_target_prompts` cleared where the old path
left it set. That record is scoped to one cast attempt and the attempt ends
when the cast lands — its own doc says so — so clearing it after a completed
cast is the correct half of the pair. Golden traces unchanged either way.

**(D) is what the class leaves behind: a tick paying for a probe it does not
use.** `affordance_probe_template` is a whole `GameState` clone, and every use
of it sits inside a gated block or a conditional — but it was built eagerly at
the top of `cast_candidates`' two hot callers, 7,238 times over six bench
games, and `sim_spell_action_inner` probed on at most 1,552 of its 3,732.
One consumer kept it honest: the Splice sweep ran on **every** tick, so the
template could never be lazy. It is a `gated_block!` on a new `spec::SPLICE`
bit now — 7,238 calls to `spliceable_hand_cards_on` become **0** — and the
template is a `OnceCell` filled by `probe_of` on first use. `GameState::clone`
falls **22,184 -> 17,808**: 4,376 clones and their drops, ~12.8 M, against
11.6 M off the program (the cell's own check is the difference). The mask
rides `gated_block!`'s debug audit, so the 18,709-test suite proves it against
real boards rather than a re-derived list.

**(E) puts a third field in `LayerFreezeState`, on the argument the other two
already stand on.** `card_type_change_in_scope` is two whole-collection walks
(`continuous_effects`, then `battlefield`) at ~559 Ir, called **34,906** times,
15,096 of them from `evaluate_requirement_static`'s card-type gate — ~5 asks
per target enumeration, all with the same answer, because nothing it reads can
change while a scope is frozen. Memoized beside `memo` and `perms` it costs
254 Ir a call there. The other two callers (`activate_ability_inner`,
`check_state_based_actions`) are `&mut self` and therefore *provably* outside
every scope — a freeze scope borrows `&self` for its closure — so they take a
new `card_type_change_unscoped` and pay neither the lock nor the memo slot they
could never read. Without that split the pass reads -0.277 %; with it,
-0.316 %.

**The suite caught a real bug in it, and the bug is filter 11's shape.**
There are *two* scope exits — `with_frozen_layers`' `Unfreeze` guard and
`freeze_layers_pop` — and the first version added the new field's clear to
only one. A `freeze_layers_push`/`pop` scope therefore leaked a stale
card-type gate into the next scope, and
`war::sarkhan_masterless_animates_and_pings` failed: Sarkhan animates
planeswalkers, so the gate's answer is the whole card. Both exits call one
`LayerFreezeState::end_of_scope` now, so a fourth field cannot survive a scope
by being forgotten at one of them. **The correct version is 468 K Ir *more*
expensive than the broken one** — the stale memo was skipping walks it owed.

**The pass on the branch: `1,531,246,782 -> 1,314,288,098`, -216,958,684 /
-14.168 %.** The (A)-(E) chain above was measured on this pass's own commits
and ends at **1,314,421,002**; the branch was then rebased onto the concurrent
run's three (`2e48c7a8`, `11116ea2`, `c6898506` — a threaded replay test, the
`CRAB_PAIR_SWEEPS` reporter, and the `restart_game` RNG fix) and re-read at
**1,314,288,098** at `03ab571d` (a reading at the head gives 1,314,290,577;
the 2,479 between them is argv length). **The 133 k is not work.** `--bench`'s
invariants are byte-identical across the rebase — same 196,220 decisions, same
27.53 turns — so the program did the same things; and none of the three
commits is on the bench path, `restart_game` needing a Karn ultimate to run at
all. What is left is code layout, which is what a 33-line addition to
`game/mod.rs` moves. **The rule this exercises is the standing one: re-read
your own base.** A pass that had carried 1,314,421,002 forward as its base
would have booked 133 k of layout as its first win.

**What is left of the class, and why each row was not taken.**
`main_phase_action_with`'s 2,036 probes and `pick_land_to_play`'s 934 hand
their action to the game driver across the `Bot::next_action` boundary;
adopting there means the *driver's* state, whose decider is live, and
`perform_action`'s own doc says why swapping a fresh-by-kind decider in
would wipe a `ScriptedDecider` mid-script. **That is the last row of the class
and the biggest single one left in the profile — 2,036 probes, ~95.7 M Ir,
7.2 % of the tip** — and it wants `Bot::next_action` to be able to hand the
driver a state, not just an action. It is not another commit in this shape:
the driver's decider is *live*, `GameState::clone` rebuilds one fresh-by-kind,
and `perform_action` swaps the live one back on every restore precisely so a
`ScriptedDecider` survives. Budget it as a `Decider`-trait change with the
server and the scripted-decider tests in scope.

### Forty-ninth pass — a chain of twenty-four narrow generators is invisible in a profile until you read the counts

Ran concurrently with pass 48 and is **rebased on top of it**. Its own chain
against pass 47's tip `40fb5e31`; then the branch. All readings
`profiling-fast --no-default-features`, callgrind, `--a gang --b gang --games
6 --threads 1 --seed 1 --decks fixed`. (The base reads 492 Ir below passes 47
and 48's number for the same commit — argv length; see **Baseline**.)

| step | before -> after | what |
|---|---|---|
| A | 1,645,831,476 -> 1,565,722,561 (**-4.867 %**) | `main_phase_action_with`'s twenty-four fallback generators go behind one board-facts mask |
| B | 1,565,722,561 -> 1,560,268,509 (**-0.348 %**) | the three land blocks ask `can_player_play_land` once; a landless hand stops `pick_land_to_play`'s mana-base walks |
| C | 1,540,962,924 -> 1,538,787,495 (**-0.141 %**) | the upkeep's keyword gate stops gathering to prove a negative — measured on the rebased branch, after (A) and (B) |
| D | 1,535,903,173 -> 1,533,436,329 (**-0.161 %**) | the attack sim's spell layer asks whether there is a window before opening a freeze scope — measured after the second rebase |
| E | 1,533,436,329 -> 1,531,246,793 (**-0.143 %**) | two clones taken only to hand a combat walk that skips the state it was given |

**(A) and (B) sum to `1,645,831,476 -> 1,560,268,509`, -85,562,967 /
-5.198 %** on their own chain, and rebased onto pass 48 they read
`1,628,220,915 -> 1,540,962,924`, -5.359 %. (C) was written after that rebase
(-> 1,538,787,495). Three more of pass 48's commits then landed underneath,
so the chain was rebased a second time and re-read end to end:
`1,625,264,320 -> 1,535,903,173`, -5.498 %. (D) and (E) then took it to
**1,531,246,793**, so the pass on the branch is
**`1,625,264,320 -> 1,531,246,793`, -94,017,527 / -5.785 %.** `--bench --threads 3` invariants byte-identical at
every step: decisions **196,220**, turns_per_game 27.53, stalls 0
(cap 0 / stuck 0 / draw 0), determinism ok. Suite 18,709 / 0 failed /
5 ignored, golden traces included. **No encoding change; no net needs
retraining as of this tip.**

**(A) is candidate (-26) read from the top, and the money was nowhere the
entry pointed.** (-26) said to read `pick_by_outcome` first — 7.08 % over 920
calls, 130,069 Ir each — and to check whether the count was a search-quality
decision. It is one, and it is beside the point. `main_phase_action_with`'s
33.00 % decomposes as `pick_by_outcome` 7.05 %, `would_accept` 6.35 %,
**`simulate_through_combat` 5.71 % over 948 calls** (every one of them
`improves_this_turn`'s two probes — see (-31)), `cast_candidates` 2.87 %,
`perform_action_inner` 2.67 %, `computed_permanent` 1.19 % over 22,542 —
and then **the tail below the cast block, which no profile had ever named,
because every generator in it is under a tenth of a percent on its own.**

The tail is two hand loops and twenty-two `pick_*` generators, reached on
**2,176 of the 3,506 ticks** — every tick with no cast and no land. Each took
its own walk of the seat's battlefield to ask "is there anything here for
me", and three took much more than a walk:

* `pick_sacrifice_value` opened with `eval_material` **and** `grant_scan`
  before it knew whether a sacrifice ability existed — 11,951,510 / 0.73 %.
* `pick_removal_ping`, `pick_removal_destroy` and `pick_removal_sacrifice`
  each built a `foes` vector of **every opposing creature's computed power**
  before checking for a matching ability. Three of the 22,542
  `computed_permanent` calls per opposing creature per tick.
* `pick_crack_lander` walked the whole **library** running
  `IsBasicLand` per card before checking for a Lander on the board.
* `pick_graveyard_recursion` deep-cloned `activated_abilities` for **every
  graveyard card** every tick (`Vec::clone` 11,482 calls) and asked
  `graveyard_granted_abilities` per card (11,482).
* `pick_equip` ranked its own creatures by *computed* power — twice — before
  checking for an Equipment.

One walk of the hand, battlefield and graveyard now answers all of them
(`sink_facts`), and `gated_pick!` skips a generator whose bit is clear. The
device is `spec` / `gated_block!`'s, already in this file for
`cast_candidates`, and so is the audit: a debug build runs the generator
anyway and asserts it returned nothing, so the 18,709-test suite checks the
mask against real boards instead of a re-derived list. Four nested shape
predicates (`reach_amount`, `makes_token`, `grants_play`, `prepares_target`)
were hoisted to module scope so the mask and the generator call one walker
and cannot drift.

`main_phase_action_with`'s direct callees, before -> after:

```text
computed_permanent            22,542 calls / 19,554,572 -> 238 / 15,044
granted_abilities_of          74,530 /  8,102,598       -> 25,044 / 2,686,034
grant_scan                     6,540 /  5,209,320       ->  2,442 / 2,005,684
graveyard_granted_abilities   11,482 /  2,054,518       -> gone
Vec::clone                    11,482 /  3,030,994       -> gone
pick_sacrifice_value           2,176 / 11,951,510       -> gone
pick_crew_vehicle              2,176 /  3,514,983       -> gone
self                                    4,346,948       -> 2,876,216
can_player_play_land (B)       9,590 /  5,600,262       ->  3,506 / 2,021,946
```

and every other tail generator with them. The mask's own cost is the residue
in that table: 25,044 `granted_abilities_of`, one `grant_scan` per traversal
instead of up to six, and ~19 k each of `is_planeswalker` / `crew_cost` /
`saddle_cost` / `is_equipment` — about 4.4 M against 85 M removed.

**The reusable finding: rank the tail, not the function.** A chain of narrow
generators is invisible in a self-cost profile (none of these reaches 0.8 %)
and invisible in a callee table sorted by Ir. It shows up only when the
**call counts** are read: twenty-two rows, every one of them at exactly 2,176
calls — once per traversal, on a board that had nothing for any of them. The
forty-second pass's rule ("ask what an ordinary action pays that it cannot
possibly need") finds these; sorting by Ir never will. **Anywhere the code
reads as a fallback chain, count the rows before costing them.**

Gating the generator is strictly better than reordering inside it, so none of
the five expensive prologues above was reordered — the gate skips the whole
call, prologue included.

**(C) is pass 48's (E) at a third site, and the site pattern was already
written down.** `board_keyword_matching` inside a freeze scope reads
`frozen_effects()`, which gathers on the scope's first computed read, so the
gate paid the gather it exists to avoid — and when it answers no, nothing else
in that scope reads the memo. `do_untap` had been converted with a comment
saying exactly this; `process_cumulative_upkeep` was the one site that was
not. 1,788 gathers (3,447,071 Ir) become 1,788 `None` reads (78,672), and the
gate's own `keyword_grant_in_scope` costs ~1.3 M of it back. **The remaining
8,364 `board_keyword_in_scope` gathers are *not* this shape** — pass 48
measured moving those and it is **+0.30 %**, because their callers go on to
`compute_battlefield()` in the same scope. Read what else the scope does
after the question; that is the whole rule.

**(D) is the same shape as (A), one level down: the question outside, the
work inside.** `sim_spell_action` opened a freeze scope on every one of the
attack search's **35,430** sim-loop iterations, and on ~23,200 of them
`sim_spell_action_inner`'s three entry tests all missed and the closure
returned `None` having read nothing layer-aware. The tests are plain field
reads (`stack`, `step`, `blockers_declared`, `active_player_idx`,
`player_with_priority`), so they move outside. **A scope is not free even when
nothing reads the memo**: the `Unfreeze` drop alone is 6,127,240 Ir of self
across the program's ~50,000 scopes, ~122 Ir a scope, and the push/pop is
another ~60. The debug audit is `gated_pick!`'s — run the closure anyway and
assert it returned nothing.

**(E) is the same question a third time — what does this cost when the answer
is "nothing to do"?** `simulate_through_combat` returns `Skipped`, leaving its
state byte-identical, on a board that is over, already past combat damage, or
has no untapped unsick creature for the active seat. Both callers that clone
*only* in order to simulate-then-score were paying for the clone anyway.
`combat_sim_skips` is now the walk's own early-out, hoisted so the guard and
the walk cannot drift, and **only the skip case takes the shortcut** — an
`Incomplete` walk really has mutated the state and the `before` probe
deliberately scores that torn board.

**(-31) was read from the top this pass and is REFUTED on cost — do not build
it.** The idea was that `improves_this_turn`'s "after" half repeats the clone,
dry-run and combat walk `pick_by_outcome` had just done for the same winner.
The call counts say how little of it is actually reusable:

```text
pick_by_outcome                920 calls
evaluate_action_outcome        842      \
evaluate_action_sequence       842       > all equal, so every finalist that
score_settled_state            842      /  was evaluated ran the full path
action_outcome_is_temporary    842      -- and none was pinned to baseline
follow_up_candidates             0      -- `gang` has lookahead 0; the
                                           sequence recursion is dead here
improves_this_turn             474
```

**842 evaluated finalists across 920 calls, with `EVAL_TOP = 3`, means at
least 499 of those calls returned at `finalists.len() <= 1` and evaluated
nothing at all.** So on more than half the ticks that reach it there is no
prior evaluation of the winner to reuse — `improves_this_turn` *is* the only
clone-and-resolve the bot does there. Against that, the lift needs a second
score threaded out of a recursive evaluator's depth-0 rung and a full
fallback for `w.determinize > 0` (where the outcome eval runs on a redealt
state and the gate's answer would change). Ceiling ~1.5 % for high call-site
churn.

**What the reading does say, and it is a strength question, not a perf one:**
the `hold_sick` / `hold_instants` gate costs about 6 % of simulator throughput
and on most of its firings is more expensive than the pick it gates. Whether
it earns that belongs in a `bot_ladder` A/B, not in this file.

### Forty-eighth pass — the profile came back, and the gate that pays is the one whose gather nobody else reads

Base `89f55a5c` (pass 47's tip as the branch stood) re-read at
**1,662,145,003**, not the 1,674,581,042 that pass's Log records. That gap is
**not** the container — pass 47's own final tip `40fb5e31` reads
1,645,831,968 here against its recorded 1,645,831,969, one Ir apart. It is
that 1,674,581,042 is pass 47's *pre-rebase* tip `3706f96f`, and `89f55a5c`
is the same seven commits sitting on top of pass 46's `cast_cost_scan`
(-0.697 %), landed by a concurrent session. **Re-read your own base**: on a
shared branch the commit you are standing on may not be the one the last pass
measured, even when the code is identical.

**The first hour went on measurement, because there was none.** Valgrind
3.22 in this image never reads `bot_ladder`'s symbol table, so every engine
frame annotated as `???:0x…` and PERF.md's whole method — read the caller
table for `__rust_alloc`, read the `file:function` rows — was unavailable.
Ruled out one at a time (all three are in the commit message of `61ea75ca`):
not the copy-the-binary hazard this file blamed, not `split-debuginfo`, not
lld/clang, not size. The dump's addresses are ELF vaddrs plus valgrind's PIE
base **0x108000**, so the symbol table resolves them; `scripts/cg_symbolize.py`
does that and `scripts/cg_edges.py` reads caller/callee tables straight out of
the dump. **Use `cg_edges.py`, not `callgrind_annotate --tree`**: the tree's
`__rust_alloc` caller block printed 23,451 of the program's 967,377
allocations and omitted `finish_grow` (200,972) and `finalize_cast` (24,108)
outright. The recovered table reproduces pass 47's recorded rows exactly
(`computed_permanent` 93,570, `GameState::clone` 79,204), which is how it was
checked.

| step | before -> after | what |
|---|---|---|
| A | 1,662,145,003 -> 1,659,704,666 (**-0.147 %**) | the cast's three tail hooks read one target list instead of three |
| C | 1,659,704,666 -> 1,658,300,737 (**-0.085 %**) | eleven expensive sort keys computed once per element, not once per comparison |
| — | 1,658,300,737 -> 1,663,268,474 (**+0.30 %**) | **REVERTED** — `board_keyword_matching` asks the presence gate before gathering. See below |
| B | 1,658,300,737 -> 1,655,478,937 (**-0.170 %**) | the cast asks one board walk what it used to ask three |
| E | 1,655,478,937 -> 1,643,104,718 (**-0.747 %**) | the mana-source table's land-type gate stops gathering to prove a negative |
| — | 1,643,104,718 -> 1,652,660,375 (**+0.58 %**) | **REVERTED** — a trigger-carrier bitmask out of `dispatch_board_scan`. See below |
| — | 1,643,104,718 -> 1,643,733,422 (**+0.038 %**) | **REVERTED** — a precomputed APNAP rank table for the trigger sort |
| — | 1,643,104,718 -> 1,643,924,923 (**+0.050 %**) | **REVERTED** — a `Vec` whose clone reserves headroom, on `stack` and two per-turn cast logs. See below |
| F | 1,628,221,407 -> 1,625,262,542 (**-0.182 %**) | two per-turn `PlayerData` sets stop being cloned by capacity — measured *after* the rebase |
| G | 1,625,262,542 -> 1,625,264,812 (**+0.00014 %**) | **KEPT as a structural fix, not a perf row** — the extra-cast target walk takes one freeze scope. See below |

**Rows A-E sum to `1,662,145,003 -> 1,643,104,718`, -19,040,285 / -1.146 %**
on pass 48's own chain; **rebased onto pass 47's last five commits they read
`1,645,831,968 -> 1,628,221,407`, -17,610,561 / -1.070 %** — the two passes
compose, with pass 48's rows slightly smaller on the branch because pass 47's
`Keyword::eq` pair had already removed some of what (B) and (E) reach. **(F)
was measured after the rebase and takes the branch to 1,625,262,542, so the
pass is `1,645,831,968 -> 1,625,262,542`, -20,569,426 / -1.250 %.**
`--bench --threads 3` invariants byte-identical at every step:
decisions **196,220**, turns_per_game 27.53, stalls 0 (cap 0 / stuck 0 /
draw 0), determinism ok. **No encoding change; no net needs retraining as of
this tip.**

**(E) is the reusable finding, and it is the forty-seventh pass's rule with
the missing clause supplied.** That pass said a presence gate is a loss where
the gather it stands in for has already happened. The clause it did not say:
**a gate is a win where the gather it avoids is read by nobody else.**
`mana_source_table` opens a freeze scope and `scan_land_type_rewrites` is its
first computed read, so `frozen_effects()` there was a whole-game gather —
7,550 of them, ~1,875 Ir each — and *nothing else in that scope reads the memo
it fills*. Swapping it for `land_type_change_in_scope` (the printed-static
twin, `false` authoritative, audited by the gather's own `debug_assert!`) took
**-0.747 %**, the pass's largest row.

The same shape applied to `board_keyword_matching` **lost +0.30 %**, and the
difference is exactly the clause: its three callers (the attack-requirement
pass, the block-requirement pass, the phasing sweep) all go on to
`compute_battlefield()` or a run of `computed_permanent`s **in the same
scope**, so the gather is not wasted — and `keyword_grant_in_scope` is ~93
`card_can_grant_keyword` calls on a late-game board, which is more than the
gather it was trying to save. **Before swapping a gather for a gate, read what
else the scope does after the question.**

**The trigger-carrier bitmask, and it is the fourth loss for fusion.**
`dispatch_triggers_for_events`' per-permanent loop runs **945,812 times over
six bench games** and its fast-path `continue` asks two `is_empty()`s that are
two `Arc` hops away (`CardInstance` -> `CardData` -> `CardDefinition`).
`dispatch_board_scan` walks the same battlefield in the same order immediately
above with the definition already in hand, so it can answer them into a `u64`
mask and the loop can read one bit. Built, correct (mask audited by a
`debug_assert!`, invariants identical), **+0.58 %** — the two loads added to
the scan cost more than the two removed from the loop, on cards the scan was
not otherwise touching those fields of. (-8b), the `do_untap` null, the
`creature_death_possible` fusion and now this: **fusing a cheap per-card
question into a walk that is already happening has lost every time it has been
tried.** Stop proposing it.

**The APNAP rank table, and why the line profile said to build it.**
`dispatch_triggers_for_events` sorted its candidates with
`sort_by_key(|c| apnap_rank(c.controller))`, and `apnap_rank` walked
`next_alive_seat` from the active player per key — O(n log n) walks a batch.
The line profile put 2.8 M Ir on `next_alive_seat`'s two lines. Ranking every
seat once per sort read **+0.038 %**, i.e. nothing: the sort is reached with
two or more candidates on a small fraction of dispatches, and 2.8 M is what
the whole thing costs. Reverted rather than kept, because it is more code for
a null.

**(C) is smaller than it looks and the reason is worth keeping.** Eleven
`sort_by_key` sites take a key that is a `computed_permanent`, a
`permanent_value` on top of one, or a linear `battlefield_find`, and
`sort_by_cached_key` is the right idiom for all of them — but it read only
**-0.085 %**, because every sort in this program is a `smallsort` (18,888 of
them; `--callers smallsort` names them all) and `sort_by_cached_key`
allocates. The two three-thousand-Ir-a-sort sites
(`beneficial_aura_host`, `pick_defensive_removal`) are most of the win.

**The line profile is back too, and it needs its caveat read.**
`[profile.profiling-lines]` (`release-fast` + packed DWARF) plus
`scripts/cg_lines.py` restores per-source-line attribution — the two profiles
read 1,659,704,679 and 1,659,704,666 on the same tip, so they inline
identically. But **lld folds identical code**, so a *function* total from the
line profile can be another function's cost wearing its name: it put 2.4 %
under `core::slice::sort::stable::drift::sort`, and the edge table says the
program's sorts cost a fraction of that. Read it for *where inside a function*
the cost is; read `cg_edges.py` for everything else.

**(G) is kept on the rule's other arm, and the honest label matters.** The
extra-cast target picker (`actions.rs`'s `CastExtraTargetPick` slot walk)
filters every battlefield permanent *plus* every player through
`check_target_legality`, which opens a freeze scope of its own per call — so
unfrozen it re-gathers every continuous effect in the game once per
candidate. That is the exact shape `legal_targets_for_filter`'s doc warns
about and was fixed for. It reads **+2,270 Ir**, i.e. nothing, because the
path is **cold on `--decks fixed`**: the bench decks essentially never cast a
multi-target spell that needs an extra slot picked. Kept as a
correctness/clarity change under this file's own escape clause — it is
strictly fewer gathers on any board that reaches it, and it is one call to a
device the sibling function already documents. **It is not a perf row and the
pass total does not include it.**

**(F) is (-29)'s cheap half, and the device was already in the file.**
`IdSet` was written for `ColdState` because "an empty `hashbrown` table clone
still walks its control bytes where an empty `Vec` clone allocates nothing" —
and hashbrown clones by **capacity**, not length, so a per-turn set used once
keeps costing a sized allocation on every clone for the rest of the game.
`PlayerData`'s two per-turn id sets never got it. `RawTable::clone` was
34,220 calls at ~137 Ir under the CoW unshare and two-thirds of them were
these; `discarded_this_turn` and `graveyard_ids_this_turn` are `IdSet<CardId>`
now, **-0.182 %**. Every consumer already used only `insert` / `contains` /
`is_empty` / `clear`. `spells_cast_by_name_this_game` stays a map — it is
game-long and grows, so it is data, not capacity. **The generalisation: a
`HashSet` field that is `clear()`ed per turn is a capacity leak into every
clone of its owner.**

**The headroom clone — built, measured, +0.050 %, and it closes (-28)'s main
body.** The program takes **224,481 `grow_one`s**, and the reasoning that
looked airtight is: `Vec::clone` hands back `capacity == len`, the checkpoint
clones `GameState` once per action, a `CowBox` unshares once per write, and a
collection taking one push per action therefore reallocates on *every* one.
A `GrowVec<T>` newtype whose `Clone` does `with_capacity(len + 4)` +
`extend_from_slice` — `#[serde(transparent)]`, `Deref`/`DerefMut` to `Vec`,
applied to `GameState::stack` and two of `PlayerData`'s per-turn cast logs —
should have removed one allocation, one `memcpy` and one `free` per
clone-then-push pair. It read **+820,205 Ir**. The clone-side cost of
`with_capacity` + `extend_from_slice` (no `to_vec` specialisation, a larger
size class) is the same order as the growth it removes, so the pair nets to
nothing. **Do not rebuild this**; and treat the 224,481 growths as a
*description* of the checkpoint, not as a cost with a lever on it.

**What the restored profile settles about `dispatch_triggers_for_events`.**
It is the largest engine self row at **86,702,366 / 5.22 %** and it is
**diffuse, measured, not assumed**: grouping every instruction address in the
program by source line, *no line in `game/mod.rs` reaches the top 400* except
`next_alive_seat`'s two at 2.8 M. There is no hot line to take. Its callees
are (-18)'s and are refuted. Stop reading it from the top.

### Forty-seventh pass — a gate that stands in for a gather stops paying once the gather has run — *folded*

Base `c9606062` at **1,727,336,594**; the pass sums to `1,715,304,981 ->
1,645,831,969` on the rebased branch, **-4.050 %** over ten commits.
`git log -- PERF.md` at the fifty-fourth pass's parent has the full entry,
its step table and its profile table. What still matters:

* **(B) -0.570 % is the reusable finding and it inverts a rule this file had
  been applying since the thirty-eighth pass.** A presence gate is a *loss*
  where the gather it stands in for has already happened; `layers_memoized()`
  answers "has it" without gathering. The forty-eighth pass supplied the
  clause that makes it usable — the question is not "has the gather
  happened" but "does anyone else in this scope read it". Both halves are
  written up in the candidates section's ranking rules.
* **(I) and (J) -0.984 % together — the derived `PartialEq` of a
  200-variant enum is an out-of-line call**, 11,532,358 Ir / 0.68 % over
  ~1.09 M calls across 224 sites. `[Keyword]::has_kw` is a three-instruction
  discriminant test in front of the `match`. **The trap that goes with it:**
  `release-fast` / `profiling-fast` have no LTO, so a bare `#[inline]` on a
  small `crabomination_base` function reads as a win here and is nothing in
  the shipped `release` build — what works is making the callee smaller than
  any inliner threshold.
* **(A) -0.283 %, (C) -0.610 %, (D) -0.083 %, (E) -0.570 %, (G) -0.426 %,
  (H) -0.552 %** — the forty-second pass's ranking rule (what does an
  ordinary action pay that it cannot possibly need) applied eight more
  times: a band question with its own freeze scope, three rare-mechanic
  questions asked before the thing that disqualifies them, a target check
  asking the layer system three times per card, two things every trigger
  dispatch paid for with nothing to dispatch, three lists walked twice, and
  two `Vec`s built where nothing wanted a `Vec`.
* **Two REFUTATIONS from this pass are load-bearing and are repeated in the
  candidates section: never skip `push_ordered_trigger_candidates` on an
  empty batch (+7.3 % *and* a correctness bug — it owns the per-batch
  `died_card_snapshots.clear()`), and the lock-free depth shadow on
  `LayerFreeze` (+0.027 %; Ir undercounts a lock prefix, so only a
  wall-clock A/B could overturn it, and that was never run).
* `--bench --threads 3` invariants byte-identical at every step: decisions
  196,220, turns 27.53, stalls 0, determinism ok. Suite 18,709 / 0 failed.
  No encoding change.


### Forty-sixth pass — the cast pays for its spell kind three times, and a land tap deep-copies its own effect tree — *folded*

Base `11792f4c` at **1,771,223,960**; own chain `-> 1,747,982,407`
(**-1.312 %**), and on the branch after the rebase onto pass 45's (E),
`1,765,005,375 -> 1,715,304,981` (**-2.816 %**) with (E), (F) and (G).
`git log -- PERF.md` at the fifty-eighth pass's parent has the full entry and
its step table. What still matters:

* **(A)+(C) -0.823 % — `CardDefinition::spell_kind`, 1,409 Ir a call, asked
  1.93 times per cast.** Two independent costs, and both are shapes: the five
  payment paths each built one for `try_pay_after_snapshot_mode` and an
  identical one for `note_cast_payment_riders` a line later; and building it
  called `printed_colors()` (a `ColorSet::to_vec` heap allocation) to ask what
  a mask test answers, while `wants_converge` hashed the card *name* with
  SipHash **under a global `RwLock`** — 394 Ir to look up a bool. A
  thread-local direct-mapped L1 keyed on the name pointer sits in front of the
  process-wide map. **That last part does not show up in a one-thread
  callgrind at all**: the row was two `RwLock` round-trips per cast on every
  actor thread at once, which is the shape the actor-scaling question is about.
* **(G) -0.697 %, the pass's biggest row, and the device is reusable.** The
  cost pipeline asked the battlefield six separate times per cast for statics
  a normal board does not have. One walk, six bits, and the replacement walk
  is cheaper than any one of the six it removes. Two things make it safe
  rather than clever: **a bit is a pure over-approximation** (the gated block
  still runs its own controller / tapped / filter tests, so a set bit costs a
  walk and a clear bit skips a no-op) with a `debug_assert!` at every gated
  site, and all six families are exercised by the suite, so the audit is not
  vacuous. **What is left is measured and priced:** `cost_reduction_for_spell_full`
  3,532,696 / 0.20 % and `extra_cost_for_spell` 1,923,032 / 0.11 %. A seventh
  bit does **not** drop in for the first — it reads 16 `StaticEffect` variants
  over `all_static_sources`, not just the battlefield, and its walk is
  followed by card-intrinsic contributions no presence bit can gate. Worth
  ~0.19 %; do it deliberately or not at all.
* **(F) -0.499 %, and it paid twice what the arithmetic said.**
  `Effect::AddMana`'s three land-tap replacement blocks each opened with their
  own `battlefield_find` to ask "is the source a land"; one source lookup and
  one fused walk beat the `find_map` row alone, because three redundant
  `battlefield_find`s went with it. Three rules tests cover the three arms —
  the coverage a rules-touching optimization needs beyond the golden traces,
  because the bench decks contain none of those cards.
* **(B) -0.457 % — `activate_ability_inner` cloned the whole effect tree of
  every land tap** (18,774 clones plus their drops) so the resolution could
  own it across `&mut self`. The `Arc<CardDefinition>` + index the fix wanted
  already existed as `HeldAbility::Printed`; three call sites took `&Effect`.
* **(E) -0.277 % — `can_afford_in_state_with` allocated a `ManaCost` twice per
  hand card per bot tick**, both for mechanics the board does not have.
* **(D) is a small row with a useful negative result. Do not chase the ward
  gathers again on this workload.** `push_ward_triggers_for_targets` takes a
  whole-game gather per opposing permanent a spell targets; gating on
  `card_keyword_possible` removes only a fifth of them (1,914 -> 1,536),
  because on these boards four out of five targeted opposing permanents really
  can carry Ward.
* **`declare_blockers`' 7,088,925 / 0.41 % on one `push` is `ColdState`'s deep
  copy, and it must not be guarded.** The very next cold write in the same
  loop costs 52,020 over 1,734 *because the group is already unshared*, so
  guarding the first write only promotes the second. The lever would be a
  cheaper `ColdState`, and (-13) already measured `clone_from` losing there.
* `--bench --threads 3` invariants byte-identical at every step. No encoding
  change.

### Forty-fifth pass — one walk answers what eleven blocks kept asking, and the board epoch is refuted — *folded*

Base `8a384e5c` at **1,810,336,693**, tip **1,765,005,375**, **-2.504 %** over
five commits and two refutations. `git log -- PERF.md` at the fifty-eighth
pass's parent has the full entry and its step table. What still matters:

* **(A) -1.608 % — `perform_action_inner`'s CR 601 cast gate was eleven
  blocks, and every one walked the whole battlefield.** Between them they
  looked the cast spell up six times through `find_card_anywhere` (38,200
  calls from `perform_action_inner`, exactly 5 x 7,640 cast attempts), and
  `damping_engine_locks` counted "who is ahead on permanents" *before* asking
  whether a Damping Engine exists. `cast_lock_scan` is one battlefield walk
  plus one command-zone walk returning a `u32` of presence bits; pass 46's (G)
  is the same device one stage later, and the over-approximation +
  `debug_assert_eq!` audit that makes it safe is written up there.
* **(B) THE BOARD EPOCH IS BUILT, ITS KEY IS SOUND, AND IT LOSES: +0.727 %
  behind a `Mutex`, +0.490 % lock-free. Do not build it again.** (-18) asked
  for it for three passes and every part of the design was right except the
  one that matters. `CowBox` got a `writes: u64` bumped in `deref_mut` and in
  the `&mut` `IntoIterator` — the only two mutation entry points, so the count
  is complete — and the five `*_in_scope` predicates read nothing but the
  battlefield and the stored `continuous_effects`, so
  `(battlefield.writes(), continuous_effects.writes())` is a *complete* key,
  established by reading four free functions rather than by hoping. A
  `debug_assert` recomputed on every hit ran the whole suite green. It still
  lost, for two reasons that generalise: **the counter is not free where the
  writes are** (`Arc::make_mut` runs 945,272+ times over six games, almost all
  of it `CardInstance`'s own handle, and every one pays the increment for a
  memo that will never read it), and **a ~700 Ir predicate is too cheap to
  memoize behind a call** — these five were inlined into their callers and
  stop being once there is a closure and a slot lookup in the way.
  **The lesson: an epoch pays only where the memoized answer costs much more
  than a call, and where the writes it counts are the writes it cares about.**
  Both would hold for the *gather* — and the gather is exactly the thing whose
  key is **not** enumerable, because it reads life totals, hand sizes,
  graveyard contents, `statics_ignored_this_turn` and `evaluate_predicate`
  through `active_static`'s `WhileCondition`. (-18) is closed both ways.
* **(D) -0.526 % and (E) -0.351 % are the forty-fourth pass's class again:
  work on a path every action takes, for a case the board or the action does
  not have.** `flagbearer_violation` ran a whole-battlefield `static_abilities`
  walk before noticing that CR 601.2c's "if able" clause is an `any` over the
  declared slot filters, so an activation with no targets cannot violate it
  (18,796 calls, almost all a land tapping for mana); `can_afford_from` cloned
  the printed `ManaCost` on every call for a mutation most costs never need;
  `action_lock_rejection` asked `GameAction::is_cast()` eight times per action.
  (E) is the two sites that shape left behind. **The rule: ask the board
  question before the argument question, and ask whether anyone wants the
  answer before either.**
* **(C) -632,248 Ir, and it is reported as small because of what it teaches.**
  The modal-mode enumeration's `vec![None]` per non-modal hand card came to
  ~25 k allocations rather than the ~50 k the shape suggests, because the
  affordability filter drops most of the hand above it. **Count what survives
  the filters above an allocation before sizing it** — the clause (-23) needed.

### Forty-fourth pass — the round-closing pass stops buying a restore nobody reads — *folded*

Base `c0f4e3b6` at **1,911,861,368**, tip **1,810,341,507**, **-5.312 %** in
four commits. `git log -- PERF.md` at `1b32e4fb`'s parent has the full entry
and its profile table. What still matters, in one line each:

* **(A) -2.842 %** — the round-closing `PassPriority` skips the transaction
  checkpoint. `GameState::clone` from `perform_action` drops **18,208 ->
  8,266** calls: the round-closing pass was 55 % of all checkpointed actions
  at ~5,465 Ir each. The remaining 8,266 are (-13)'s, and they are the half
  where the checkpoint earns its keep.
* **(B) -1.024 %** — the target scans take one freeze scope instead of one per
  candidate. The device that reaches the `frozen_effects` gathers is
  **lexical scope widening, site by site, each measured**: two more written
  the same day read -11,778 Ir together, i.e. nothing.
* **(C) -1.308 %** — the trigger dispatcher stopped building `vec![false; 2]`
  53,838 times for a Ring nobody wears. **(D) -0.227 %** — two per-action
  walks stopped allocating for statics nobody controls. Together they are
  candidate (-23)'s class and it is not exhausted.
* `scripts/fallibility_closure.py` is that pass's, and it is the device that
  made (A) provable: it enumerates the `Result` functions an action's arm can
  reach and how many of them raise. `play_land` reaches 6 and 2 raise;
  `submit_decision` reaches 137 and 70 raise, which is why the rest of (-13)
  is not proven arm by arm.


### Forty-third pass — a cleared collection is not an empty one — *folded*

Base `1032979c` at **1,918,781,907**, tip **1,911,862,094**, -0.361 % in two
commits. `git log -- PERF.md` at `36592fd8` has the full entry and its
profile table. What still matters, in one line each:

* **(A) -0.192 %, and the rule generalises: any `clear()` on a collection a
  `GameState` clone reaches is a standing per-clone allocation.** hashbrown
  clones a table with the *source's* bucket count, not its length, so a map
  that held entries once and was cleared re-allocates and memcpys a full
  table on every checkpoint and probe clone for the rest of the game.
  `Default::default()` instead of `clear()` on the three `GameState`
  `HashMap`s: **-16,276 allocations**. `Vec` is exempt — `Vec::clone`
  allocates `len`, not `capacity`, which is also why `finalize_cast`'s cast
  logs regrow instead (see (-23)).
* **(B) -0.168 %** — the untap step asks once whether any static can reach
  it, instead of six specialised walks per step.
* **The null result that is the reusable half.** Gating `do_untap`'s six
  walks behind one pass over the same list `active_static` walks read
  **+0.0001 %**: each of the six short-circuits on
  `definition.static_abilities.is_empty()` for a board of lands and vanilla
  creatures, so six specialised `any`s beat one general pass. **(-8b)'s
  lesson from the other side.** `do_untap`'s 37 M is not in those walks —
  read its callee table, not its walk count.

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

**(C) and (D) are the pass, and the durable part is the rule, not the rows.**
`ColdState` is ~90 collections behind one `CowBox` and `perform_action` holds
a checkpoint, so the group is **always shared**: the first cold write of *any*
action runs `Arc::make_mut` and deep-copies the lot, ~1,700 Ir then and ~3,425
now. Reads through `Deref` cost nothing. So a `clear()` on an already-empty
collection, a `retain` that keeps everything, an `iter_mut` over an empty list
or a `mem::take` of nothing pays the full copy for no effect — and three of
those were on the hottest paths in the program (`tapped_for_cost`'s
empty-to-empty write, 32.7 M / 1.60 % over 18,774, was the largest). The
`clear_cold!` / `retain_cold!` macros in `game/mod.rs` are the guarded forms;
`mem::take`, `iter_mut` and whole-field assignment still have to be guarded by
hand, and an extension trait cannot do it because `self.field.method()` fires
`DerefMut` before the method body runs. See (-14) for where the survivors are
and for the measured reason not to guard `declare_blockers`' first cold write.

**(A), (B), (E)-(H) are the other rule this pass established, and it is the one
that keeps paying**: before ranking a *function*, ask what an ordinary action
pays that it cannot possibly need. Thirteen `is_mana_ability` walks, ten source
lookups, a `Vec` per permanent in the mana-source walk, a `Vec` for a cost's
colours, two whole-board `HashSet`s per untap step, four tree walks for five
colours. None was on the candidates list and none shows up as an expensive
function — they land in `make_mut`, `memcpy` and `_int_malloc`.

**How to find the cold-group ones again:**

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

### The three pools at the sixty-first tip

Same binary, same config, one pool each: **`fixed` 1,206,204,087, `sos`
1,580,084,804, `cube` 2,841,539,263.** Top self costs, `sos` (the pool the
actors play) with `fixed` alongside (shares to two places; the two sessions'
tips are within 0.06 % of each other on every pool, so the shares hold):

| row | sos | fixed |
|---|---|---|
| `__memcpy_avx_unaligned_erms` | **7.92 %** | 4.30 % |
| `dispatch_triggers_for_events` | **5.77 %** | **5.58 %** |
| allocator (`_int_free`+`malloc`+`free`+`_int_malloc`) | ~12.7 % | ~12.7 % |
| `gather_continuous_effects_inner` | 3.64 % | 4.49 % |
| `Arc::clone_from_ref_in` | 3.08 % | 3.24 % |
| `check_state_based_actions` | 2.37 % | 1.99 % |
| `Vec::from_iter` (all monos) | 2.01 % | 2.78 % |
| `sba_board_scan` | 1.78 % | 1.73 % |
| `GameState::clone` | 1.58 % | 1.79 % |
| `activate_ability_inner` | 1.40 % | 1.19 % |
| `dispatch_board_scan` | 1.38 % | 1.70 % |
| `perform_action_inner` | 1.33 % | 1.67 % |
| `computed_permanent` | 1.28 % | 1.34 % |
| `compute_permanent_pass` | 1.26 % | 1.44 % |
| `evaluate_requirement_static_hinted` | 1.23 % | 1.99 % |
| `card_type_change_unscoped` | 1.04 % | 0.97 % |
| `card_can_grant_keyword` | 1.00 % | 1.40 % |

**`dispatch_triggers_for_events` is the largest engine self row on both
pools and has been since pass 43** — 91,088,068 Ir on `sos` over 81,744
calls, of which **44,560 get past the empty-batch return**, i.e. **2,044 Ir
of self per working dispatch**, essentially all of it the
`for card in &self.battlefield` walk. (-16) reads it as diffuse by line
(largest single line 1.06 % on cube) and it has never been read by line on
`sos`. That is the standing "biggest row with no taker".

**Two things did not move this pass and are worth the line:** `__memcpy` is
7.92 % of `sos` against 4.30 % of `fixed` — the same 3.6-point gap (-40)
recorded, still diffuse across 21 k caller rows — and the allocator family
is 12.7 % on both.

### The four pools at the fifty-third tip (`1ba3e76b`)

Same binary, same config (`--a gang --b gang --games 6 --threads 1 --seed 1`),
one pool each. **A game costs about the same on every pool** — 52.4 M
(`fixed`), 84.3 M (`cube`), 58.9 M (`sos`), 49.2 M (`sealed`, measured over
240 games so the deck build amortises out) — and the *shapes* differ only
where the pool's cards do. Read the row that matches what your change
touches.

| row | fixed 1,258,304,569 | cube 4,048,597,048 | sealed (240 games) 11,810,935,584 |
|---|---|---|---|
| `dispatch_triggers_for_events` | **5.29 %** | 3.91 % | 5.56 % |
| `gather_continuous_effects_inner` | 4.14 % | **7.99 %** | 5.88 % |
| `__memcpy` | 4.13 % | 4.75 % | 3.97 % |
| `Vec::from_iter` | 3.45 % | 3.09 % | 3.21 % |
| `Arc::clone_from_ref_in` | 3.17 % | 2.41 % | 3.01 % |
| `check_state_based_actions` | 2.30 % | — | 2.61 % |
| `evaluate_requirement_static` | 1.94 % | 2.70 % | 1.19 %* |
| `computed_permanent` | 1.32 % | **4.14 %** | 1.46 % |
| `compute_permanent_pass` | 1.38 % | **2.97 %** | 1.43 % |
| `sba_board_scan` | 1.66 % | — | 1.77 % |
| allocator (`malloc`+`free`+`_int_*`) | ~10.7 % | ~13.1 % | ~12.1 % |

\* the `'2` monomorphization only; the two split differently per pool.

**The conclusion worth keeping: `fixed` is a sound proxy for the game loop
and a useless one for the layer path.** The sealed pool — what
`selfplay_train` actually plays — is within a point of `fixed` on every row.
The cube pool is the outlier, and it is the outlier in exactly one place:
`computed_permanent` + `compute_permanent_pass` + the gather are **15.1 %**
there against 6.8 % on `fixed`, because cube boards carry the layer-4 and
grant statics the hand-built archetypes do not. That is what is left of
the cube gap after the three freeze scopes took it from 32.5 % to 8.0 %.

**The branch ends at 1,258,304,569 Ir on `--decks fixed`** at the
fifty-third tip; the pass-52 table below was read at 1,265,410,851 and every
row holds to within 0.6 % (the pass's `fixed` delta). Top self-cost rows at
that reading:

| row | Ir | % | note |
|---|---|---|---|
| `dispatch_triggers_for_events` | 66,550,642 | **5.26** | down from 5.60 % — pass 52's (B) took the three tail-loop walks + gates off it, but its cost is diffuse across the phase-1 walk. **(-16), still the largest self-cost row** |
| `gather_continuous_effects_inner` | 52,105,052 | 4.12 | |
| `__memcpy` | 51,953,975 | 4.11 | |
| `_int_free` | 46,160,262 | 3.65 | |
| `Vec::from_iter` | 43,392,008 | 3.43 | |
| `Arc::clone_from_ref_in` | 39,906,990 | 3.15 | -3.9 M vs pass 50 tip (fewer CoW unshares on the adopted paths). (-29) |
| `malloc` | 33,983,716 | 2.69 | |
| `evaluate_requirement_static` | 33,494,274 | 2.65 | unchanged. (-35), the largest non-allocator self row after dispatch |
| `check_state_based_actions` | 28,964,498 | 2.29 | (-17) |
| `_int_malloc` | 26,560,235 | 2.10 | |
| `GameState::clone` | 21,641,360 | 1.71 | -1.3 M vs pass 50 tip |
| `Arc::make_mut` | 21,151,274 | 1.67 | |
| `sba_board_scan` | 20,935,578 | 1.65 | |
| `dispatch_board_scan` | 20,561,286 | 1.62 | |

Perform-action edges at the pass 52 tip, by caller:

| caller | calls | Ir |
|---|---|---|
| `sim_step` | 31,874 | 276,884,388 |
| `perform_action` (driver) | 25,462 | 248,720,738 (was 26,502 / 291.6 M at pass 51's tip — **1,040 skipped** by adoption) |
| `accept_on` | 5,260 | 242,457,976 (unchanged count — same probes, different return type) |
| `evaluate_action_sequence` | 1,756 | 22,004,817 |
| `simulate_attack_outcome_once` | 1,622 | 63,230,581 |
| `main_phase_action_with` (via finalist) | 1,040 | 14,635,882 (was 1,514 / 42.7 M at pass 49 tip) |
| `simulate_block_outcome_once` | 302 | 14,120,666 |

The pass 50 table below is kept for its Log rows; the numbers hold to within
~50 M against this pass's tip, which is enough for candidate ranking. The
pass-50 tip row and the fifty-second Log's step tables carry the exact
totals for their commits.

The table below was taken at 1,330,233,580, before (D) and a clippy
`collapsible_if` on (C)'s diff — (D) moved 4,376 `GameState` clones and the
Splice sweep, so every row here holds to within ~12 M.

| row at the 50th tip | Ir | % | note |
|---|---|---|---|
| `pick_attacks_scored` | 706,842,699 | **53.14** | still the largest subtree; `simulate_attack_outcome_once` 699,394,707 / 52.58 % over 1,170 candidates. Candidate (-21) |
| `perform_action_inner` | 927,821,672 | 69.75 | 68,356 calls. By caller: `sim_step` 31,874 / 278.8 M, `perform_action` 26,502 / 291.4 M, **`accept_on` 5,260 / 243.3 M**, `simulate_attack_outcome_once` 1,622 / 63.3 M, `evaluate_action_sequence` 1,756 / 22.2 M, `main_phase_action_with` 1,040 / 14.7 M |
| `main_phase_action_with` | 386,602,637 | 29.06 | `pick_by_outcome` 920, `accept_on` 2,036 / 95.7 M (the class's last row — see the Log), `simulate_through_combat` 804 |
| `pass_priority` | 364,163,549 | 27.38 | -> `advance_step` 22,892 / 267.6 M, `resolve_top_of_stack` 4,250 / 89.6 M |
| `sim_step` | 356,610,293 | 26.81 | 31,874 `PassPriority` / 278.8 M **+ 2,636 checkpointed / 72.0 M** (was 4,568 / 209.2 M before this pass) |
| `advance_step` | 267,564,235 | 20.11 | **11,688 Ir a step advance.** `resolve_combat` 2,694 / 150.4 M, its own recursion 1,764 / 42.1 M, `do_untap` 1,764 / 31.8 M, `do_cleanup` 1,764 / 25.7 M, `fire_step_triggers` 14,898 / 21.7 M |
| `accept_on` | 260,534,785 | 19.59 | the dry-run probes. 5,260 calls; only `main_phase_action_with`'s 2,036 and `pick_land_to_play`'s 934 are still followed by a second execution |
| `cast_spell` | 251,144,642 | 18.88 | `try_pay_after_snapshot_mode` 137.8 M, `auto_tap_for_cost_inner` 127.2 M — (-12) |
| `sim_spell_action_inner` | 226,453,098 | 17.02 | `accept_on` 1,552, `cast_candidates` 3,732, `pick_stack_response` 4,656, `pick_combat_trick` 3,842 |
| `resolve_combat` | 195,218,265 | 14.68 | 2,694 calls at **55,816 Ir each** — the largest engine row, and (-25) reads it as diffuse |
| `check_state_based_actions` | 125,389,731 | 9.43 | |
| `dispatch_triggers_for_events` | 116,921,949 | 8.79 | **the largest self-cost row in the program at 74,482,294 / 5.60 %.** 90,750 calls, 53,838 past the empty-batch return, so ~1,383 Ir of *self* per working dispatch. (-16) read it at the 43rd tip and called it diffuse; nobody has read it per source line |
| `cast_candidates` | 105,425,302 | 7.93 | 7,238 calls; never read from the top |

Self cost, same tip: `dispatch_triggers_for_events` 74,482,294 / 5.60 %,
`__memcpy` 52,606,496 / 3.95 %, `gather_continuous_effects_inner` 52,514,806 /
3.95 %, `_int_free` 48,958,844 / 3.68 %, `from_iter` 45,014,132 / 3.38 %,
`Arc::clone_from_ref_in` 42,918,920 / 3.23 %, `malloc` 36,078,832 / 2.71 %,
`evaluate_requirement_static` 33,530,088 / 2.52 %. **773,131 allocations**
(926,895 at pass 49's own tip). The top 24 self rows are 53.6 % of the
program and 1,170 rows hold the rest — see the note above the `__rust_alloc`
recipe.

**The forty-ninth tip's table is kept below** because its Log rows chain to
it; it was taken five commits before that pass ended (`cg.rb.out`,
1,540,962,924) and every row holds to within 10 M of 1,531,246,793. The
forty-eighth's, forty-seventh's and forty-sixth's are kept under that for the
same reason — read those as shares, not absolutes. The forty-fifth's was
folded away at the 48th tip, as the forty-second's and forty-fourth's were at
the 2.8 k fold; **the forty-sixth's is the next fold.**

| row | at the 49th tip | note |
|---|---|---|
| `pick_attacks_scored` inclusive | 845,192,380 / **54.85 %** | **the largest subtree by a distance now, and the share is up 3 points because this pass took 5 % out of everything else.** `simulate_attack_outcome_once` 837,784,800 / 54.37 % over 1,170 candidates; under it `sim_step` 30.5 % over 35,316 and `sim_spell_action`'s freeze scope 16.7 % over 35,430. Candidate (-21) |
| `perform_action_inner` inclusive | 706,797,014 / 45.87 % over 70,418 | |
| **`main_phase_action_with` inclusive** | **455,418,707 / 29.55 %** | was 32.98 %. `pick_by_outcome` 7.42 %, `would_accept` 6.6 %, `simulate_through_combat` 6.1 % (all `improves_this_turn` — (-31)), `cast_candidates` 3.0 %. **The tail is gone; (-26) is closed** |
| `cast_spell` inclusive | 439,809,476 / 28.54 % | `auto_tap_for_cost_inner` 224,238,052 / **14.55 %** — (-12) |
| `pass_priority` inclusive | 359,845,190 / 23.35 % | `advance_step` 270,052,007 / 17.52 % |
| `would_accept` inclusive | 274,479,135 / **17.81 %** | the probe *is* a cast — do not go after the clone |
| `resolve_combat` inclusive | 166,862,790 / 10.83 % | candidate (-25) |
| `activate_ability` inclusive | 134,094,874 / 8.70 % | the land tap |
| **`simulate_through_combat` inclusive** | **127,301,040 / 8.26 % over ~1,790** | 948 of those calls are `improves_this_turn`'s two probes and 842 are `score_settled_state`'s. New candidate (-31) |
| `pick_by_outcome` inclusive | 114,355,626 / 7.42 % over 920 | search, not engine — (-26)'s closing note |
| `finalize_cast` inclusive | 110,664,718 / 7.18 % over 7,172 | diffuse — (-28) |
| `dispatch_triggers_for_events` incl | 110,747,151 / 7.19 % | self 5.5 %, the largest engine self row, and **measured diffuse** |
| `computed_permanent` inclusive | 89,128,971 / **5.78 %** | **96,206 calls, down from 156,624**, of which 22,494 gather. 69,202 `Arc::new` allocations — (-27) |
| `gather_continuous_effects_inner` incl | 73,308,172 / 4.76 % | **37,674 gathers**, down from 39,692 |
| `check_state_based_actions` incl | 73,209,534 / 4.75 % | (-17) |
| `compute_permanent_pass` | 39,335,473 / 2.55 % | |
| `card_can_grant_keyword` | 21,777,378 / 1.41 % | (-11), still demoted |
| allocator | `__rust_dealloc` 98.4 M / 6.38 %, `free` 96.5 / 6.26, `__rust_alloc` 71.3 / 4.63 | **908,931 allocations**, down from 949,413 |

**The allocator caller table at the 49th tip, by call count**
(`cg_edges.py --callers __rust_alloc`):

| direct caller of `__rust_alloc` | allocs | note |
|---|---|---|
| `RawVecInner::finish_grow` | 208,813 | Vec growth, all callers. (-28) closed the headroom idea |
| `Arc::clone_from_ref_in` | 152,062 | the CoW unshares — (-29) |
| `Vec::from_iter` (nested) | 120,044 | was 126,686 |
| `GameState::clone` | 79,204 | (-13) costed narrowing and said no |
| `computed_permanent` | 69,202 | **was 93,570** — this pass's (A) took 24 k of them. (-27) |
| `gather_continuous_effects_inner` | 33,976 | |
| `RawTable::clone` | 29,428 | under the CoW unshare — (-29)'s cheap half |
| `Vec::clone` | 29,172 | |
| `Box::clone` | 26,386 | |
| `finalize_cast` | 24,108 | (-28) |
| `RawTable::reserve_rehash` | 19,280 | |
| `Vec::from_iter` (in-place) | 17,146 | |
| `frozen_effects` | 10,152 | **was 17,702** — pass 48's (E) |
| `auto_tap_for_cost_inner` | 9,544 | |

**The forty-eighth pass's table, kept because its Log rows chain to it.** It
was taken at 1,643,104,718 on that pass's *own* chain (`1b32e4fb`,
`cg.E.out`), before the rebase, so its absolutes read ~15 M high against the
branch. Call counts and ratios transfer.

| row | at the 48th tip | note |
|---|---|---|
| `pick_attacks_scored` inclusive | 854,074,781 / **51.98 %** | still the largest subtree. Candidate (-21) |
| `perform_action_inner` inclusive | 710,991,030 / 43.27 % over 70,418 | |
| `main_phase_action_with` inclusive | 541,932,039 / 32.98 % | `pick_by_outcome` 115,656,568 / 7.04 %, `cast_candidates` 105,480,446 / 6.42 %. Candidate (-26) |
| `cast_spell` inclusive | 442,486,580 / 26.93 % | `try_pay_after_snapshot_mode` 238,333,418 / 14.51 %, `auto_tap_for_cost_inner` 223,976,275 / 13.63 % — (-12). **-14.6 M off this pass's (E)** |
| `pass_priority` inclusive | 364,367,083 / 22.18 % | `advance_step` 271,848,026 / 16.54 % |
| `would_accept` inclusive | 275,410,930 / 16.76 % | the probe *is* a cast — do not go after the clone |
| `resolve_combat` inclusive | 197,778,990 / 12.04 % | candidate (-25) |
| `activate_ability` inclusive | 134,102,108 / 8.16 % | the land tap |
| `computed_permanent` inclusive | 117,834,527 / **7.17 %** | 156,624 calls, of which **24,512 gather** (47.7 M / 2.90 %). 93,570 `Arc::new` allocations — (-27) |
| **`finalize_cast` inclusive** | **111,559,898 / 6.79 % over 7,172** | **read from the top for the first time this pass, and two rows came off it.** What is left is diffuse — see (-28) |
| `dispatch_triggers_for_events` incl | 111,303,671 / 6.77 % | self **86,702,366 / 5.28 %**, the largest engine self row, and **measured diffuse**: no line of `game/mod.rs` reaches the top 400 lines of the program |
| `gather_continuous_effects_inner` incl | 77,140,506 / 4.69 % | **39,692 gathers, down from 48,466** — this pass's (E) took 7,550 off `frozen_effects` |
| `check_state_based_actions` incl | 74,164,270 / 4.51 % | (-17) |
| allocator | `free` 101.5 M / 6.18 %, `__rust_dealloc` 103.4 / 6.29, `__rust_alloc` 76.8 / 4.67 | **949,413 allocations** (was 967,377) |

**The allocator caller table at the 48th tip, by call count** — the complete
one, from `cg_edges.py --callers __rust_alloc` (`callgrind_annotate --tree`
truncates it):

| direct caller of `__rust_alloc` | allocs | Ir | note |
|---|---|---|---|
| `RawVecInner::finish_grow` | 210,649 | 17,795,566 | Vec growth, all callers |
| `Arc::clone_from_ref_in` | 152,062 | 15,118,806 | the CoW unshares; **52.9 M self on top**, and the largest unclaimed structural row |
| `Vec::from_iter` (nested) | 129,596 | 8,959,741 | the `.collect()`s |
| `computed_permanent` | 93,570 | 7,539,892 | one `Arc::new` per memo miss — (-27) |
| `GameState::clone` | 79,204 | 4,610,566 | (-13) costed narrowing and said no |
| `gather_continuous_effects_inner` | 35,340 | 2,861,721 | |
| `Vec::clone` | 31,068 | 3,666,933 | |
| `RawTable::clone` | 29,428 | 1,805,423 | **read this pass, and two-thirds of it is PAID by (F)**: `CardData` has no `HashMap`, so these were `PlayerData`'s three; the two per-turn ones are `IdSet` now. `spells_cast_by_name_this_game` is what is left and it is real data |
| `Box::clone` | 26,386 | 1,457,618 | 23,822 of them are one `Box` field on `GameState`, cloned per checkpoint. **Unread** |
| `finalize_cast` | 24,108 | 2,766,997 | 3.4 per cast — (-28) |

**`grow_one`'s callers, 224,481 growths:** `Vec::push_mut` 41,842,
`finalize_cast` 28,878, `advance_step` 22,892, `gather_continuous_effects_inner`
13,406, `declare_blockers` 13,122, `dispatch_board_scan` 11,654,
`auto_tap_for_cost_inner` 7,550, `effective_mana_abilities_into` 7,490,
`resolve_combat` 7,250, `compute_permanent_pass` 6,216. **Every one of these
is the same shape**: `Vec::clone` hands back `capacity == len`, so the first
push after a checkpoint or a CoW unshare reallocates. See (-28).

**`computed_permanent`'s callers, 156,624 calls** — the Ir/call column is the
one that matters, because ~2,000 means a gather and ~300 means a memo hit:

| caller | calls | Ir | Ir/call |
|---|---|---|---|
| `bot::permanent_value` | 23,020 | 21,765,364 | 946 |
| `main_phase_action_with` | 22,542 | 19,757,899 | 877 |
| sort comparators (`FnMut::call_mut`) | 21,922 | 15,958,694 | 728 |
| `blocker_can_block_attacker` | 13,688 | 3,734,496 | 273 |
| `damage_prevented_by_protection` | 12,644 | 5,648,595 | 447 |
| `bot::attacker_damage_value` | 11,482 | 8,673,609 | 755 |
| **`resolve_combat`** | 5,682 | 11,821,508 | **2,080 — a gather each** |
| **`check_target_legality_with_source`** | 4,692 | 10,350,943 | **2,206 — a gather each** |
| **`push_ward_triggers_for_targets`** | 1,536 | 4,602,784 | **2,997 — a gather each** |

**The forty-seventh pass's table, kept because live Log rows chain to it.
Its absolutes are from a different container — see Baseline.**

**The forty-seventh pass ends at 1,645,831,969 Ir (`a98d39b0`).** The table
below is the pass's *seventh* tip (`3706f96f`, `cg.H.out`, 1,674,581,042), so
its absolutes read ~29 M high against the branch; the shares moved by under
a tenth of a point except where the `Keyword::eq` pair reached them. Read at
the final tip for comparison: `pick_attacks_scored` 855,253,773 / **51.96 %**,
`main_phase_action_with` 543,137,204 / **33.00 %**, `cast_spell`
457,553,907 / 27.80 %, `pass_priority` 361,907,500 / 21.99 %, `would_accept`
283,190,480 / 17.21 %, `advance_step` 269,863,905 / 16.40 %,
`auto_tap_for_cost_inner` 236,526,902 / 14.37 %, `resolve_combat`
167,309,456 / 10.17 %, `activate_ability` 133,325,490 / 8.10 %,
`dispatch_triggers_for_events` 110,716,632 / 6.73 %,
`gather_continuous_effects_inner` 89,317,427 / 5.43 %,
`check_state_based_actions` 73,845,459 / 4.49 %, `compute_permanent_pass`
48,772,538 / 2.96 %, `card_can_grant_keyword` 21,777,378 / 1.32 %, and
**`Keyword::eq` 3,219,922 / 0.20 %** (was 11,532,358 / 0.68 %).

| row | at the 47th tip | note |
|---|---|---|
| `pick_attacks_scored` inclusive | 872,027,524 / **52.07 %** | still the largest subtree, and its share barely moved: this pass took Ir out of the engine *under* it as much as anywhere else. Candidate (-21) |
| **`main_phase_action_with` inclusive** | **552,113,968 / 32.97 %** | **the second-largest bot subtree and never read from the top.** `pick_by_outcome` 119,663,481 / 7.08 % over **920 calls** (130,069 Ir a call), `would_accept*` 108,264,447 / 6.40 %, `cast_candidates` 47,284,337 / 2.80 % over 3,506, `computed_permanent` 20,737,266 / 1.23 %, `pick_land_to_play` 15,420,727 over 1,488. New candidate (-26) |
| `cast_spell` inclusive | 472,510,200 / 28.22 % | `auto_tap_for_cost_inner` 236,674,814 / **14.13 %** is what is left, and it is (-12) |
| `pass_priority` inclusive | 364,878,954 / 21.79 % | `advance_step` 272,354,213 / 16.26 % |
| `would_accept` (affordances) incl | 290,234,464 / 17.33 % | the probe *is* a cast — do not go after the clone |
| `resolve_combat` inclusive | 168,952,362 / **10.09 %** | **was 11.86 % at the 46th tip**; (A) and (C) came off it. Candidate (-25) |
| `activate_ability` inclusive | 133,390,741 / 7.97 % | the land tap |
| `computed_permanent` inclusive | 118,156,936 / **7.06 %** | 93,570 `Arc::new(ComputedPermanent)` allocations, one per memo miss — the fourth-largest allocator caller and **unclaimed** |
| `dispatch_triggers_for_events` incl | 111,316,638 / **6.65 %** | was 7.01 %. `dispatch_board_scan` 24,561,076 / 1.47 % over 53,838 is the largest thing left in it and is (-18)'s |
| `gather_continuous_effects_inner` incl | 89,702,428 / 5.36 % | **48,466 gathers**, unchanged: the count is the lever, not the gather |
| `check_state_based_actions` incl | 85,517,165 / 5.11 % | 55,720 `from_iter` calls / 35 M is (-17); most of the collects inside are already behind an `sba_board_scan` flag |
| `declare_blockers` inclusive | 68,832,813 / 4.11 % | 7.1 M of it is one `ColdState` unshare per block declaration — (-14), and guarding it promotes the next write |
| `compute_permanent_pass` | 50,560,044 / **3.02 %** | was 3.38 %; (H) took the empty `granted_keywords_eot` collect off it. `printed_color_set` is 8,176,716 / 0.48 % of what is left, 81 Ir over 99,840 passes |
| allocator | `free` 104.2 M / 6.22 %, `_int_free` 70.9 / 4.24, `malloc` 68.6 / 4.09, `memcpy` 56.2 / 3.35, `_int_malloc` 40.4 / 2.41 | over **974,927 allocations** (was 1,021,777). See (-23)'s refreshed table |
| `card_can_grant_keyword` | 21,777,378 / 1.30 % | was 28.6 M / 1.66 % — (B) took the protection caller off it. Candidate (-11), still demoted |
| `card_keyword_possible` inclusive | 21,733,120 / 1.30 % | **unchanged**: this is the land tap's CR 602.5 gate, which runs from `&mut self` with no scope open, so (B) does not reach it |
| `sba_board_scan` | 20,966,376 / 1.25 % over 9,206 | 2,277 Ir a sweep, ~65 Ir a card — five inner `Vec` loops plus ten field reads. A per-`CardDefinition` cached bitmask would collapse it and is **unsound**; see (-11)'s note |

**The allocator caller table at the 47th tip, `<`-block only, by call count
(974,927 allocations):**

| direct caller of `__rust_alloc` | allocs | note |
|---|---|---|
| `RawVecInner::finish_grow` | 211,913 | Vec growth, all callers. `finalize_cast` 28,878 growths / 8.8 M is the largest single site and is **unread** |
| `Arc::clone_from_ref_in` | 152,062 | the CoW unshares |
| `Vec::from_iter` (nested) | 134,604 | was 149,696 — (H) |
| `computed_permanent` | 93,570 | one `Arc::new(ComputedPermanent)` per memo miss. **Unclaimed, and the largest named row** |
| `GameState::clone` | 79,204 | (-13) costed narrowing and said no |
| `gather_continuous_effects_inner` | 39,800 | |
| `Vec::clone` | 31,068 | |
| `RawTable::clone` | 29,428 | **unread** |
| `Box::clone` | 26,386 | **unread** |
| `finalize_cast` | 24,108 | 3.4 per cast; the logs regrow after every `PlayerData` clone because `Vec::clone` gives capacity == len |
| `RawTable::reserve_rehash` | 19,280 | |
| `frozen_effects` | 17,702 | one per freeze scope |
| `auto_tap_for_cost_inner` | 9,544 | |
| `ManaCost::reduce_generic` | 7,550 | |

**The forty-sixth pass's profile table is folded** (fifty-third pass): its
Log rows have stopped chaining and every number a live candidate needs is
carried by that candidate. It was taken at 1,747,982,407, so its absolutes
read ~20 M high against that pass's own tip; `git log -- PERF.md` at
`fdac88df^` has it in full. What it established and where that lives now:
`pass_priority` 21.32 % and `resolve_combat` 11.86 % read from the top for
the first time (**(-25)**), the 2,646 combat SBA sweeps at 27,065 Ir each
(**(-17)**), `declare_blockers`' one `ColdState` unshare per declaration
(**(-14)**), and the land tap's callee table — `card_keyword_possible`
1,149 Ir a call over 18,910, `continue_ability_resolution_x` 1,058,
`card_type_change_in_scope` 483 — which **(-12)** carries.

**The one argument from it that is not a number, kept because it is a
refutation.** ~830 of `card_keyword_possible`'s 1,149 Ir is
`keyword_grant_in_scope`'s board walk, and that is the same answer for every
tap in one `auto_tap_for_cost_inner` batch (2.1 of them). **Stamping it per
batch is unsound**: a mana ability may put a counter on its source or
sacrifice it, so the board can move between taps and a stale `false` would
skip a real restriction. (-11) has the cache shapes and why they lose.

**The forty-seventh pass's table is the next fold.**

**The forty-fifth's, the forty-fourth's and the forty-second's profile tables
were folded away** (the last two at the 2.8 k mark, the forty-fifth's at the
forty-eighth pass). Their Log entries keep every row that a live candidate
chains to; the full tables are in `git log -- PERF.md` at `36592fd8`,
`b1a95b22` and `89f55a5c`.

## Perf candidates

Ordered by expected value. Each run pulls the top one, attaches numbers,
and feeds what it finds back in. Re-profile and replenish when the list
goes thin or stale.

**Ranking rule added by the fifty-third pass, and it is about the workload,
not the code: ask which pool the change lives on before you cost it.** Two
of that pass's finds were larger than anything in this list and neither was
visible on the bench — the per-card grant walk (49 % of a cube game) and
deck construction (96 % of a deck build, and a training actor builds two
decks a game). "Which pool a change moves" at the top of this file is the
device; the short version is that `--decks fixed` carries no
`GrantTriggeredAbility` static and builds its decks once.

**(-47) `pick_blocks` IS 120,203,116 Ir / 4.34 % OF `cube` INCLUSIVE, AND
`blocker_can_block_attacker` RECOMPUTES THE ATTACKER'S COMPUTED VIEW ONCE
PER CANDIDATE BLOCKER.** Found at the sixty-second tip by the pool-ratio
device — `pick_blocks_inner` is 0.90 % of cube against 0.43 % of sos
(**2.09x**) and 0.53 % of fixed. Sized, not taken; the reason it was not
taken is at the bottom and is about risk, not size.

```text
pick_blocks                 4,070 calls   120,203,116 incl   29,533 Ir/call
-> pick_blocks_inner       17,502          24,906,488 self
   -> blocker_can_block_attacker  28,374   34,005,223 incl    1,449 Ir/call
      -> computed_permanent       56,748   13,238,631 incl      233 Ir/call  <- 2x
      -> can_block_attacker_computed 28,374  6,399,180
      -> blocker_side_gates_allow_block 28,374 5,472,506
```

**Two per check is one too many.** `blocker_can_block_attacker(blocker_id,
attacker_id)` resolves *both* ids to computed views internally, and its
callers loop blockers inside a fixed attacker: `pick_blocks_inner`'s forced
and menace passes are `battlefield.iter().filter(|c| … state
.blocker_can_block_attacker(c.id, *a_id))`. So for one attacker and N
candidate blockers the attacker's view is built N times and the blockers'
once each — N+1 distinct permanents, 2N computations. It also costs two
`battlefield_find`s per check, which is **(-38)**'s.

**And the 233 Ir is itself worth reading before costing a hoist.** Outside a
`layer_freeze` scope `computed_permanent` recomputes; inside one it hits a
memo, and 233 Ir says these calls are hitting it. But the memo is
`st.perms.iter().find(|(k, _)| *k == id)` **under a mutex** — a lock
acquisition plus a linear scan plus an `Arc` clone. So there are two
independent shapes here and they are worth different amounts:

1. **Hoist the attacker's view out of the blocker loop.** Removes ~28,374
   of the 56,748 calls: **~6.6 M on cube, ~0.24 %**. Needs a
   `blocker_can_block_attacker_with(blocker, blocker_cp, attacker, atk_cp)`
   and seven call sites in `bot.rs` threaded through it.
2. **Make the freeze memo a map rather than a linear scan.** Helps every
   `computed_permanent` caller inside a scope, not just this one — and
   `computed_permanent` is 1.61 % of cube by itself. Unsized; measure how
   long `perms` actually gets before assuming a scan is the cost, because on
   ~20 permanents the *lock* is the likelier half.

**Why it is here and not in the Log: the risk is behavioural, not
numerical.** All seven call sites are inside the bot's block selection, and
this is the function that decides which blocks are *legal* — a hoist that
passes the wrong side's computed view does not crash, it quietly makes the
bot block illegally or decline legal blocks, and the ladder printout is the
only thing that would catch it. Take it with the three-pool printout diff
and the `--bench` `decisions` count, not on the suite alone. Shape 2 has no
such exposure and is the better first take.

**(-45) THE COST OF ASKING — the sixty-first pass's class, and the table it
came out of still has rows.** Every one of that
pass's wins was a presence question whose *asking* cost the same whether the
answer was yes or no, on a board where it is always no: a SipHash of ~84
small integers for a fingerprint (both sessions found that one
independently), a `filter`/`flat_map`/`filter`/`map` stack collected into an
always-empty `Vec`, a `flat_map` over two always-empty command zones, and a
battlefield `filter` for a card no bench deck contains. **None of them is a
hot function**; together they are ~1.5 % of `sos`.

**The device is one command**, and it is why this entry exists rather than a
list of sites: `python3 scripts/cg_edges.py cg.out --callers
SpecFromIterNested`, **ranked by calls, not by Ir**, then ask of each row
"can this collect be non-empty on the pools the actors play?". The
sixty-first tip's table on `--decks sos` (1,580,084,804), with the two rows
that pass already took removed:

```text
  calls        Ir        caller
  90,170    6,537,071    layers::compute_permanent_pass       <- 72 Ir each, 9.6 per
                                                                 compute_permanents
  37,420    1,336,242    resolve_effect                       <- 36 Ir each
  21,912    7,323,096    declare_attackers_banded
  18,750      944,725    fire_delayed_event_watchers          <- 50 Ir each
  13,576   15,884,891    check_state_based_actions            <- 1.02 per sweep now,
                                                                 was 4.02
  11,334    2,445,441    finalize_cast
  11,004    1,543,826    fire_combat_damage_to_player_triggers
  10,836      769,032    blockers_of
  10,328   20,436,507    bot::pick_attacks_inner
   9,374   13,233,290    compute_permanents
 149,490         —      103 more rows (38.07 % of the calls)
```

**Read the two columns against each other.** A row with many calls and few
Ir apiece (`compute_permanent_pass`, `resolve_effect`,
`fire_delayed_event_watchers`) is a `Vec` being built to be thrown away —
the sixty-first pass's shape, and the cheapest kind to fix, because the
replacement is a loop. A row with few calls and a lot of Ir apiece
(`pick_attacks_inner`, `compute_permanents`) is the *iterator body* being
re-reported through `from_iter`, not collect overhead — **(-17)'s standing
caveat**, and those rows are not this entry's.

**And the third kind, which is the one to check before writing any code: a
collect that exists because a `&mut self` follows it.** Phase 1.6 of
`fire_combat_damage_triggers` was removable precisely because its loop body
touched nothing but a local. `declare_attackers_banded`'s two big ones
(`listeners`, `you_attack` — both whole-battlefield `flat_map`s over
`triggered_abilities`) push onto `self.stack` in the drain, so the buffer is
load-bearing and only a presence gate could help there — and the gate is the
walk. **The tell is one line: does the drain touch `self`?** Check it first;
it takes seconds and it is the difference between a one-line fix and a
refactor that cannot pay.

**The one refutation this entry ships with, because it is the boundary:**
folding a per-sweep presence question into `sba_board_scan` — the walk that
already visits every permanent — read **+0.295 % / +0.255 %** for
`card_type_change_unscoped`'s battlefield leg. The standalone
`any(card_can_change_card_types)` short-circuits per card; a scan bit cannot.
**A presence bit belongs in a shared scan only when the question has no early
exit of its own.** Third refutation of the (-6) fusion device inside
`creature_death_possible` alone (+0.55 %, +1.24 %, +0.29 %).

**And the sibling table nobody has run:** `--callers` on the *`Vec::clone`*
and `RawVec::grow_one` rows is the same question asked of a different
allocation shape. `finish_grow` is 11,680,360 / 0.73 % on `sos` and
`Vec::clone` 12,902,527 / 0.81 %, and neither has ever had a caller table in
this file.

**(-46) `name_index()` BUILDS 22,568 `CardDefinition`s TO READ 22,568
STRINGS — 104,687,400 Ir. RANKED LOW ON PURPOSE; READ THE SIZING BEFORE
TAKING IT.** `card_registry::name_index()`'s `OnceLock` calls every catalog
factory, keeps `def.name` (and the MDFC back face's), and drops the
definition. That is the whole cost: one edge, 104.7 M Ir, **6.8 % of a
six-game `--decks sos` run** and 0 % of `cube` and `fixed` (call tables in
**How to measure**).

**It is one-time per process, and that is the entry.** A `selfplay_train`
actor plays thousands of games on one process, so 104.7 M is ~0.001 % of it;
a test binary that resolves any name pays it once, tens of milliseconds
against a 27 s suite. The 6.8 % is a *measurement* artefact — it inflates every short `sos`
callgrind total — not a throughput one. **Do not take this ahead of anything
in the game loop.**

The fix, if a pass ever wants it cheaply: the names are known at build time,
so `crabomination_catalog` can emit a `&[(&'static str, CardFactory)]`
alongside `all_known_factories` and the index becomes a map build over a
static slice. That is codegen work in the catalog crate for a startup
number, which is why it sits here rather than getting done. A narrower
half-measure — have `apply_pending_effect_answer` resolve a `NameCard`
answer against the same pool the suggestions were ranked from before falling
back to the registry (the sixty-first pass did exactly this for the
*suggestion* side) — would keep bot self-play off the index entirely, but it
only moves *when* the build happens for anything that names an off-board
card, so measure that it removes the edge rather than assuming it.

**(-44) `__memcpy` IS 5.55 % OF `sos` AND THE ALLOCATOR FAMILY 12.7 %, AND
NEITHER HAS EVER HAD A CALLER TABLE READ BY Ir/CALL.** The sixtieth pass took
`__memcpy` from 7.80 % to 5.55 % with one commit, and the way it found the row
is the entry: the table is forty rows of a hundred-odd Ir each, and one row —
`CardInstance::new`, 3,452 calls at **8,242 Ir apiece** — is an outlier in the
*ratio*, not in the calls or the total. 8,242 Ir is `size_of::<CardDefinition>
()` = 8,232 bytes moving. Do the same read on `_int_free` (4.25 %) and
`malloc` + `_int_malloc` (5.78 %): an allocation whose Ir/call is far above
the family's mean is an allocation of something big, and that is a shape, not
a diffuse cost.

**The allocator half of this entry was read the same way and there is no
outlier — record kept so nobody redoes it.** 1,072,958 `__rust_alloc` calls a
six-game `sos` run; by Ir/call the table is flat: `finish_grow` 241,644 at 86,
`Arc::clone_from_ref_in` 140,010 at 123, `Vec::clone` 39,750 at **174** (the
highest, and it is a moderate `Vec`, not a kilobyte), `Box::clone` 38,129 at
79, `finalize_cast` 13,868 at 119, `mana::cost` 28,024 at 58. The engine
functions that allocate *directly* and look wrong — `mana::cost` 28,024,
`SelectionRequirement::and` 16,992, `::or` 8,464 — are **catalog
constructors**, i.e. definition-build time, which `card_arc` and `card_brief`
now amortize to once per factory per thread; the only game-loop caller of
`mana::cost` is `declare_blockers` at 1,258 calls / 191,426 Ir (0.012 %).
`finish_grow`'s 22.5 % share is (-28)'s `Vec::clone` capacity story and the
`GrowVec` newtype that measured **+0.050 %**. **So: the allocator is a real
12.7 % and it is genuinely diffuse.**

**The *dealloc* side is now read too, at the sixty-second tip, and it is the
same answer — so this half of the entry is CLOSED, both sides.** The alloc
table above is `__rust_alloc`'s; `_int_free` (4.26 %) and `free` (2.71 %)
are reached through `__rust_dealloc`, which is a different table and had
never been read. It is 1,071,319 calls and flat to within a factor of 1.3
by Ir/call — there is no `CardInstance::new`-shaped outlier in it:

```text
callers of __rust_dealloc, --decks sos, sixty-second tip
  calls        Ir        Ir/call  caller
 243,857   27,078,427      111    Arc::drop_slow
  89,930   11,310,723      126    Arc::drop_slow'2
  87,118    9,574,823      110    drop_in_place<GameState>
  70,374    7,663,336      109    drop_in_place<CardDefinition>
  42,176    4,169,983       99    check_state_based_actions
  39,191    3,859,235       98    drop_in_place<Box<SelectionRequirement>>
  30,302    2,919,322       96    gather_continuous_effects_inner
  27,830    3,294,958      118    drop_in_place<Result<Vec<GameEvent>,_>>
  26,748    2,755,679      103    auto_tap_for_cost_inner
  25,405    3,168,277      125    IntoIter::drop
 322,448         —          —     440 more rows (30.10 %)
```

**95 to 126 Ir a call, top to bottom.** The sixtieth pass's device works by
finding a ratio outlier; there isn't one here, on either side. **Do not
spend a build re-reading `_int_free` or `malloc` by Ir/call** — a
handoff-note in TODO was still pointing at it, and this is the record that
retires it. The allocator's 12.75 % is a million small frees, and the two
rows with any shape to them (`Arc::drop_slow` at 333,787 calls between them,
`drop_in_place<CardDefinition>` at 70,374) are the *paying* side of the CoW
unshares that **(-43)** already prices at 80.9 M — counted there, not here.

The sos table at the sixtieth tip, top of `__memcpy`'s callers:

| caller | calls | Ir | Ir/call |
|---|---|---|---|
| `GameState::clone` | 103,330 | 6,222,884 | 60 |
| `finalize_cast` | 92,590 | 7,832,712 | 85 |
| `String::write_str` | 65,285 | 1,029,267 | 16 |
| `computed_permanent` | 60,556 | 1,816,680 | 30 |
| `Vec::clone` | 34,547 | 1,875,090 | 54 |

**Nothing left in that table is an outlier**, which is what "diffuse" should
mean and usually does not. `finalize_cast`'s 24.7 memcpys a call are
(-28)'s and that entry is closed to everything but a `CardTypeSet` bitset.

**What is left under `CardInstance::new`, both of them the same shape one size
down:** `mint_token_onto_battlefield` is **370 calls / 6,644,250 Ir (0.43 %)**
— `token_to_card_definition` builds a whole 8 KB `CardDefinition` per token
minted and `Arc::new` copies it — and
`definition_matches_requirement` **deep-clones** a `CardDefinition` (156 /
2,736,861, 0.18 %) to build a scratch `CardInstance` for a predicate.
`card_arc`'s trick does not transfer to either: a token's definition is a
*value*, so a memo needs `TokenDefinition: Hash` (it derives only `Eq`) and an
eviction rule for `CreateTokenCopyOf`, and the predicate site holds a
`&CardDefinition` with no `Arc` to share. **Size them together at ~0.6 % and
do not start one alone.**

**(-43) THE CoW-HANDLE FAMILY, READ FROM THE TOP AT THE FIFTY-EIGHTH TIP.
`make_mut` on `sos` is 439,300 calls after four commits took it down from
582,552 (-24.6 %), and the table below is where the rest of it is.** This
is (-42) generalised: `Player`, `CardInstance` and `CowBox` are all CoW
handles whose `DerefMut` is `Arc::make_mut`, so a run of writes through one
pays an unshare per write.

```text
callers of make_mut, --decks sos, at the fifty-eighth tip
  calls        Ir        Ir/call  caller
  52,110   12,005,430      230    cast_spell_with_convoke
  41,200    4,572,433      111    do_untap            (paid; residual is singletons)
  37,532    3,959,023      105    resolve_top_of_stack_inner
  30,208   13,848,574      458    activate_ability_inner
  28,722    9,731,410      339    declare_attackers_banded
  26,474    1,700,225       64    finalize_cast       (paid, 46,992 -> 26,474)
  26,538      875,532       33    resolve_combat
   8,494      224,380       26    on_left_battlefield (paid, 24,352 -> 8,494)
 222,990         —          —     62 more rows (50.8 %)
```

**Read the Ir/call column before picking a row, because it splits the family
in two and only one half is the (-42) device.** A caller at ~30 Ir/call is
paying the refcount check and nothing else — that is a run of writes on an
already-unshared handle, and `let x = &mut *…` collapses it, which is what
the three commits did (`advance_step` 51,142 -> 9,198, `do_untap` 80,148 ->
41,200, `deal_combat_damage_to_target` 21,012 -> 10,008). A caller at
230-458 Ir/call is *actually cloning*: the `Arc` is genuinely shared at that
point, and binding once saves nothing because the second write through the
same binding was already cheap. `activate_ability_inner` (13.85 M Ir, the
largest in the table) and `cast_spell_with_convoke` (12.01 M) are that
second kind, and **the question for them is not "bind once", it is "who else
holds this `Arc`, and does the write have to happen while they do"** — a
snapshot, a `clone()` kept across the call, or a handle parked in a local.
Size the prize off the Ir column, not the calls column: the two clone-shaped
rows are 25.9 M Ir together, 1.6 % of `sos`, against 8.5 M for the whole
bind-once half.

**STOP GRINDING THE BIND-ONCE HALF — it has a measured ceiling and four
commits already took most of it.** A `profiling-lines` build plus
`cg_sites.py … deref_mut` prices the *entire* inlined `deref_mut` family at
**6,551,438 Ir, 0.53 % of the run**, across 111 sites, and the largest single
one is 1,193,304 Ir (0.10 %). That is the whole remaining prize for binding
handles, against the 80.9 M (5.04 %) of actual clones below. The four
commits of this pass took `make_mut` 582,552 -> 439,300 and there is no
site left that is worth a pass of its own:

```text
inlined deref_mut call sites, --decks sos, after the four commits
  1,193,304 (0.10%)  stack.rs:5777    check_state_based_actions (token sweep)
    623,910 (0.05%)  layers.rs:487    compute_permanent_pass
    548,064 (0.04%)  mod.rs:2153      GameState::deref_mut
    527,502 (0.04%)  card.rs:6513     CardInstance::deref_mut
    389,528 (0.03%)  mod.rs:3812      Vec::index_mut
    356,520 (0.03%)  layers.rs:581    compute_permanent_pass
    267,360 (0.02%)  mod.rs:17738     dispatch_triggers_for_events
    267,360 (0.02%)  mod.rs:17872     dispatch_triggers_for_events
    800,194 (0.06%)  86 more sites
```

**`resolve_top_of_stack_inner` and `resolve_combat` looked like the two rows
left and are not takeable**: both are short functions whose `make_mut` count
belongs to *inlined callees*, so there is no run of writes to bind — that is
why they need `cg_sites.py` rather than a read of the function. Same for
`check_state_based_actions`: its eight seat writes are all singletons and all
already gated, and its `Player::deref_mut` line (`player.rs:1053`,
1,361,936 Ir) has nothing to collapse.

**The by-line read of `check_state_based_actions` is a separate lead and a
bigger one — it is 47,133,588 Ir, 3.8 % of the run**, and diffuse: its top
row is a dependency's `macros.rs:332` at 7,724,320 (0.62 %), then `cmp.rs:412`
at 2,770,838. Nothing in it is a CoW handle. Sized here, not chased.

**Then the family was read one level further down, and the real number is
much larger than the caller table suggests. `make_mut` genuinely *clones*
85,322 times out of 475,676 (17.9 %), at 748 Ir each.** Counting the
specialised `deref_mut`s alongside the generic row, the CoW bodies are
deep-copied **91,478 times for 80,868,330 Ir — 5.04 % of `sos`**:

```text
callers of Arc::clone_from_ref_in, --decks sos
  85,322   63,860,853   Arc::make_mut (the generic, inlined row)
   3,102   14,285,495   GameState::deref_mut          <- 4,605 Ir a clone
   1,692    1,532,689   PlayerData::deref_mut
     842      767,056   CardInstance::deref_mut
     460      367,273   CowBox::deref_mut
      60       54,964   Player::deref_mut
```

Inside one clone: **1.64 allocations** (140,010 `__rust_alloc` / 17,044,619
Ir, the single largest line item), ~2 `Vec` clones (168,822 / 10,123,493),
half a `RawTable` (43,362 / 2,923,028).

**And the cause is not in any of those callers — it is the bot's probe
machinery, one frame up.** `GameState::clone` runs **19,086 times, 24.8 M Ir
self (1.55 %)**, essentially all of it speculative: `accept_on` 6,660,
`perform_action`'s failure checkpoint 5,606, `ProbeCell::try_init` 3,376,
`sim_start_state` 1,226,
`evaluate_action_sequence` 896, `main_phase_action_with` 894. Each such
clone is cheap by design — it bumps refcounts — and then **every subsequent
write on either side pays the deferred deep copy.** So the probe machinery's
true price is the 24.8 M of clones *plus* the 80.9 M they force:
**105.7 M Ir, 6.6 % of `sos`**.

**The value of this entry is the 80.9 M deferred half, not the sub-candidate
list — because every item on that list turns out to be an existing entry
that was already answered.** That is the finding: the CoW clone cost has
been costed three times from the *causing* side ((-13) on the checkpoint,
(-41) on `available_mana`, `ProbeCell` on the probe templates) and never
once from the *paying* side, which is 3.2x larger than the clones that cause
it and does not appear in any caller table of `GameState::clone`. Read it
that way before proposing anything:

1. **`perform_action`'s checkpoint (5,606 clones) — this is (-13), it is
   already costed, and the answer was no. Do not re-open it without a new
   argument.** Pass 43's (A) took the provable half (-2.842 %): the
   round-closing `PassPriority` skips the checkpoint, and
   `GameState::clone` from `perform_action` went 18,208 -> 8,266. **The
   remainder is explicitly the half where the checkpoint earns its keep**,
   and `scripts/fallibility_closure.py` is the tool that decides it —
   `play_land` reaches 6 `Result` functions of which 2 raise, but
   `submit_decision` reaches **137 of which 70 raise**, which is exactly why
   the rest is not proven arm by arm. TODO's "Engine — Rollback / Undo
   system" has the other half of the argument: the checkpoint is what
   structurally kills the audit-P0 partial-mutation family (Squad/Casualty
   under-pay, `declare_attackers` mid-loop corruption, back-face land
   corruption, madness mana loss), it is pinned by
   `cow::tests::rejected_action_restores_state_exactly`, and **any narrowing
   of it is a rules-correctness argument first and a perf change second**.
2. **The `ProbeCell` inits (3,376 clones) — already largely paid, and the
   `try_init` caller table will mislead you into re-taking it.** Three
   different `OnceCell`s share that table and only one holds a `GameState`.
   The 3,376 are exactly `sim_spell_action_inner` 1,280 +
   `main_phase_action_with` 1,152 + `cast_candidates` 944, and `ProbeCell`'s
   whole point is that a previous pass *already* made them lazy (its doc
   comment: `sim_spell_action_inner` alone took 3,732 templates per six
   bench games and probed on at most 1,552). What is left is inits that were
   genuinely needed. **The table's two big rows are not `GameState` at
   all** — `evaluate_requirement_static_hinted` 106,242 / 8,520,486 is a
   static-eval memo at 80 Ir a call, and `can_afford_in_state_with` 5,712 /
   19,593,552 is `SweepMana`'s `AvailableMana` cell, i.e. `available_mana`,
   which is **(-41)**, not this entry. Check which cell a `try_init` row
   belongs to before costing it; this write-up got it wrong on the first
   pass for exactly that reason.
3. **`accept_on` (6,660 clones), and it is the one to leave alone.** The
   divergence is the point: probing N candidate actions against one template
   means N of them must diverge, and CoW already makes that cost
   proportional to what each action touches. The only waste is the probes
   that return `None`, and that is not knowable in advance.

**Size any of it against the clock before believing the Ir**, and this
family is the exact shape the standing rules warn about twice over: 17.0 M
of the 80.9 M is `__rust_alloc` under callgrind's *system* allocator when
mimalloc ships, and the rest is `memcpy`, which pass 57's `cae6b605` showed
reads -1.95 % in Ir and flat on nine alternated `selfplay_train` pairs.
Get the `selfplay_train` number first.

**(-42) PAID at the fifty-eighth pass, in two commits: `sos` -0.261 % then
-0.120 %, `cube` -0.253 % then -0.115 %, and `do_untap`'s `make_mut` calls
went 212,012 -> 41,200.** The
answer was **`Player` is itself a CoW handle** — `Player::deref_mut` is
`Arc::make_mut` — so the `for pl in &mut self.players` per-turn reset, about
fifty-five field writes per seat, took one unshare *per field*. One
`let pl = &mut **pl;` collapses them.

**The generalisable shape, and it is the CowBox sharp edge from the other
side:** a run of writes through a CoW *handle* pays a refcount check each,
and the handles in this engine are `Player`, `CardInstance`, `CowBox` and the
`cold` groups behind them. Wherever a sweep writes several fields of one
handle in a row — the cleanup step's per-turn resets are the obvious next
place — bind the target once. The reads are already free: rustc picks `Deref`
for a place used immutably even through a `&mut` binding (checked with a
standalone test; see below).

**The tail is paid too, in a second commit: 80,148 -> 41,200 calls
(-48.6 %), `sos` -0.120 %, `cube` -0.115 %.** Three runs of
`self.players[p].X = …` (7, 5 and 5 writes) took a `let me = &mut
*self.players[p];`, and three `for pl in &mut self.players` loops took the
`let pl = &mut **pl;`. The interleaved `retain_cold!`/`clear_cold!` calls on
`self` are what had kept them out of the first commit; they only split the
runs, they do not force a re-unshare, so each run collapses on its own.
Ir saved 1,933,414 on `sos` against 1,168,440 of `make_mut` self — the other
765 k is the per-write preamble (index, `Arc` load, refcount load, branch)
that went with it.

**What is left is 41,200 calls, 27.5 per untap step**, in the main untap
loop's per-card writes. `clear_end_of_turn_effects` is already one
`deref_mut` for its whole macro-expanded reset *and* gated on
`end_of_turn_effects_are_clear()`, so the residual is the gated singletons
(`granted_flashback_eot`, `granted_harmonize_eot`, `damage`) — one write
each, nothing left to bind once. This entry is closed as a site; the
*device* is open everywhere else (see the generalisable shape above, and the
cleanup step).

**The base was re-read before the delta was quoted, and it is the whole
reason the number above is 0.120 % and not 2.069 %.** The first reading
compared against `1,639,754,965`, the figure in this pass's Baseline block —
but that was measured before a concurrent session's `cae6b605` rebased in
underneath, and the true base at `d2a8320b` is **1,607,757,957**. The stale
column would have credited this commit with 17x its actual win — and that
session's own write-up, landing in the same rebase, prices `cae6b605` on
`sos` at **-1.946 %**, which is exactly the gap. The rule
already in TODO's NEXT ("re-read the base after a rebase before quoting a
delta") was written for `--decks sealed --games 1` and binary size; it
applies just as hard to a game pool when the branch is shared. **A predicted
size that comes in 17x high is the signal — model the win first, then let a
miss that large indict the base rather than the change.**

The original reading, for the sizing: 1,498 untap steps, **212,012 `make_mut`
calls** at 45.7 Ir each — 9,694,094 Ir, **0.58 % of the run** — against 9,832
`clear_summoning_sickness` calls and 6,960 Ir of `remove_counters`. On cube
it was **363,462 calls / 15,771,096 Ir**, the largest `make_mut` caller in the
program by count (27 % of all of them) and 12 % of the program's 1.33 M
`make_mut` calls for one turn-based action.

**TWO explanations are refuted, and between them they narrow the entry to
"not the obvious places". Read both before proposing anything.**

* **Reads through `&mut` do not pay a `DerefMut`.** rustc picks `Deref` for a
  place expression used immutably even when the base binding is `&mut`, and a
  standalone test of exactly this shape (an `Arc::make_mut` in `DerefMut` with
  a counter) reads **zero** `deref_mut` calls after a loop body of pure reads
  and one after a single write. So the 141.5 are real writes.
* **They are not the per-turn flag roll-over loop**, which is the obvious
  candidate: `goaded_by` / `detained_by` / `attacked_this_turn` /
  `blocked_this_turn` / `blocked_attackers_this_turn` / `attacked_last_turn` /
  `attacked_own_turn` / `attack_ban`, up to seven writes per permanent. Built
  and measured at the fifty-eighth tip: the reads moved to a shared reborrow,
  the whole block gated on "does anything want a write at all", and the writes
  taking **one** `&mut **card` between them. `do_untap`'s `make_mut` count
  went **212,012 -> 211,298** — 714 calls, 0.34 % of them — and the run read
  `sos` **+0.0008 %**, `cube` **+0.0004 %**. Reverted. Pass 43's per-write
  gates were already doing their job; the loop writes almost nothing.

**The line profile is what named it, and the entry is worth reading for how
cheap the answer was once the right tool ran.** `cg_lines.py --in do_untap`
on a `profiling-lines` build (11m32s cold, 552 MB binary) reports `do_untap`
as *diffuse by line* — its largest row is 365,638 Ir, 0.03 % of the run —
which is exactly the wrong-looking answer. The row that mattered was two
lines down the list: **`player.rs:1053` at 155,792 Ir, which is
`Player::deref_mut`**, sitting inside `do_untap`. The line profile does not
rank the fix; it names the *type* being unshared, and that was the whole
find. Both cheap refutations above assumed the card was the handle.

**(-41) `available_mana` walks the grants of every untapped permanent, and it
is the one caller of `granted_abilities_of` that can be pre-filtered.**
`--decks sos` at the fifty-eighth tip: `granted_abilities_of` is **68,538
calls / ~48 M Ir, 2.9 % of the run**, over three callers —
`effective_mana_abilities_into` 27,846 / 18.88 M, **`bot::available_mana`
25,680 / 18,159,501 (1.09 %)**, `main_phase_action_with` (via
`usable_abilities`) 15,012 / 10.91 M. Inside it,
`evaluate_requirement_static_hinted` is 39,430 calls / 19,242,702 and the
pushes cost `ActivatedAbility::clone` 11,324 / 7.87 M plus `grow_one` 11,324
/ 8.49 M.

The lever is pass 57's (C) at a different site: **ask the cheap question
first.** `available_mana` keeps only abilities that pass
`is_countable_mana_ability` (a handful of field tests), so testing the
*ability* before evaluating the grant's `SelectionRequirement` against this
permanent skips both the evaluation and the clone for every grant that is not
a mana ability.

**It is sound for `available_mana` only.** The other two callers index into
the returned list — `effective_mana_abilities_into` pushes
`(printed_count + j, …)` and `usable_abilities` `(n + i, …)`, and
`activate_ability` resolves granted abilities by that index — so a filtered
list would renumber them. Skipping a grant's requirement evaluation also
means not knowing whether it *would* have been included, so the indices
cannot be preserved by carrying the original position either.

**AND IT IS WORTH NOTHING ON `sos`, WHICH IS THE POOL THAT MATTERS. Do not
build it for the actors.** The filter only pays for grants whose ability is
*not* a countable mana ability, and **SOS has exactly one
`GrantActivatedAbility` static in the whole set** — Petrified Hamlet's
"lands with the chosen name have `{T}: Add {C}`" — which is a mana ability
and survives the filter. `--decks fixed` carries no grant at all. So the
ceiling on the shipped workload is zero, and `available_mana`'s 18.16 M is
the walk itself, not the filter evaluations inside it.

Catalog-wide there are **68 `GrantActivatedAbility` sites** and roughly a
third of them grant mana, so a *cube* board carrying one of the other
two-thirds would pay. If someone wants this, measure it on `--decks cube`
and quote that pool; it is the pass-53 rule ("which pool does the change
live on") pointing the other way for once — the change lives on a pool the
training loop does not play.

**It was also built and measured, by a second session, which is the same
answer from the other end.** `granted_abilities_of_where(.., keep: fn(&Ac
tivatedAbility) -> bool)` with `available_mana` passing
`is_countable_mana_ability`: `granted_abilities_of`'s edge to
`evaluate_requirement_static_hinted` came back at **exactly 39,430 calls /
19,241,574 Ir — unchanged to the instruction**, and `ActivatedAbility::clone`
at exactly 11,324, i.e. the filter rejected nothing at all. What it added was
24,926 `FnOnce::call_once` (the `|_| true` the two other callers now pass) and
14,504 `is_countable_mana_ability` calls, for **+0.049 % on `sos`,
+0.019 % on cube**. Reverted. **The entry stays closed for the shipped pools,
and the useful part of it is elsewhere: the same table is what named
`cae6b605`'s 11,324 clones.**

**`wants_converge` is 12,507,301 Ir / 0.42 % of a six-game cube run over 217
calls, and it is startup, not steady state.** Pass 46 gave it a thread-local
L1 in front of the process-wide map so the `format!("{self:?}")` probe runs at
most once per card name per process; those 217 are the cube pool's distinct
names, and the whole `core::fmt` family is 0.3 % of the run behind them
(158,350 `String::write_str` calls, 78,120 of them `DebugStruct::field`). A
training actor amortises it to nothing over thousands of games. **What it
does mean is that a six-game benchmark carries a fixed ~12 M of per-name
setup** — do not read a 0.4 % move on this workload as a steady-state result
without checking whether it landed there.

**(-40) THE CUBE POOL, READ FROM THE TOP AT THE FIFTY-SEVENTH TIP
(2,952,041,750).** Self cost, whole program:

| row | % | note |
|---|---|---|
| `__memcpy_avx_unaligned_erms` | **5.73** | the CoW/clone family's memory traffic |
| `dispatch_triggers_for_events` | **4.96** | was 5.14 % before this pass's (C); its self cost is what is left after the grant walk came off, and it is **diffuse by line** — the largest row inside it is 1.06 % and unresolved |
| `gather_continuous_effects_inner` | 4.00 | 118.0 M absolute, down from 195.9 M at the pass-55 tip |
| allocator family | ~11.5 | `_int_free` 3.65, `malloc` 2.75, `_int_malloc` 2.57, `free` ~2.2 |
| `Arc::clone_from_ref_in` | 3.20 | |
| `Vec::from_iter` | 2.87 | |
| `evaluate_requirement_static_hinted` | ~1.9 | 106.6 M of it came off with (C) |
| `check_state_based_actions` | ~2.1 | |
| `compute_permanent_pass` / `computed_permanent` | ~1.5 each | |
| `GameState::clone` / `Arc::make_mut` | ~1.4 / ~1.3 | |
| `sba_board_scan` | ~1.4 | |

**`dispatch_triggers_for_events` has now been read from the top, and the
answer was one callee, not the function.** Its caller table is
`perform_action_inner` 103,598 of 125,820 calls; its callee table put
`statics_granted_triggers_inner` at **235,062 calls / 113.1 M inclusive**,
and (C) took all of it. What is left is genuinely diffuse: by line the
largest row inside it is 1.06 % and unresolved, then 0.36 %, 0.36 %,
0.29 % — the forty-ninth pass's "measured diffuse" verdict, now with a
measurement behind it. **The remaining named callees are
`dispatch_board_scan` (67,360 calls / 35.5 M), `event_matches_spec` (662,098
/ 19.2 M at 29 Ir each) and `push_ordered_trigger_candidates` (67,352 /
13.2 M)**; none is a fan of narrow walks and none has an obvious question to
ask first.

**The one thing left to rank off this table is the clone/allocator family:
`__memcpy` + the allocator + `Arc::clone_from_ref_in` + `make_mut` +
`GameState::clone` is ~23 % between them**, which is (-10)/(-13)'s
checkpoint-sharing cost seen from the profile side.

**READ FROM THE TOP AT THE FIFTY-EIGHTH TIP, ON `sos` — the pool the shipped
workload plays — AND THE ANSWER IS: DIFFUSE. It is bigger there than on
cube and there is still no caller to take.** `--decks sos` is 1,658,496,337
and the family is **26.5 %** of it: `__memcpy` **8.21** (against cube's
5.73), the allocator 12.66 (`_int_free` 4.06, `malloc` 3.16, `_int_malloc`
2.85, `free` 2.59), `clone_from_ref_in` 2.92, `GameState::clone` 1.50,
`make_mut` 1.26.

`__memcpy`'s caller table on `sos` is 1,293,413 calls, and **the top twenty
callers hold barely half of them — 630,711 are spread over 21,130 rows**
(48.76 %). The named half: `GameState::clone` 103,330, `finalize_cast`
92,590, `String::write_str` 65,285 (the `wants_converge` startup below),
`computed_permanent` 60,556, `Vec::clone` 34,547, `mana::cost` 28,704 (a card
factory building the pool's definitions — startup), `clone_from_ref_in`
28,010, `RawTable::clone` 24,672, `ActivatedAbility::clone` 23,118,
`Iterator::partition` 20,424, `effective_mana_abilities_into` 18,880.
`GameState::clone` itself is **32,600 calls on cube**: `accept_on` 12,012,
the `perform_action` checkpoint 8,964, the bot's probe cells 6,300 (see
`ProbeCell`), `sim_start_state` 2,170, `evaluate_action_sequence` 1,290,
`main_phase_action_with` 1,252, `score_settled_state` 496 — i.e. the dry-run
probes and the transaction, which (-10)/(-13) costed and refused.

`Arc::make_mut` is **1,331,820 calls on cube** of which only 169,420 actually
unshare (12.7 %), so the *fast* path is the cost and it is a call count
problem, not a copying one. Its biggest caller by count is `do_untap` —
see (-42), which is the one row of this family that is not diffuse.

**So: do not open the clone family as a whole again.** What is left in it are
the two named rows with their own entries, (-41) and (-42), and the
allocation table by call count, which is (-23)'s recipe.

**The gather is the target, and the question is scope count, not gating.**
59,470 gathers for six games; a freeze scope's *first* computed read gathers
to fill its memo, so the count is roughly "how many scopes, plus every
unscoped read". Gating that first read is refuted (see the fifty-fifth
pass's Log).

**The gather's internals are now read by line, and the `from_iter` row was
the smaller half.** The fifty-seventh pass ran `profiling-lines` +
`cg_lines.py --in gather_continuous_effects_inner` on cube, which this entry
had been asking for. The gather's 195.9 M self / 6.12 % breaks down as
iteration machinery, not allocation: `macros.rs:?` 29.8 M,
`non_null.rs:444` 18.7 M, `non_null.rs:1720` 15.7 M, `mod.rs:?` 12.9 M,
`macros.rs:180` 9.8 M — the thirty-eight per-static passes walking
`sa_cards` and re-reading every card's `static_abilities`. **The tell is a
run of identically-costed rows** (ten at 511,188 Ir, eight at 425,990, seven
at 340,792), and none of them was inside `cg_lines.py`'s default forty-five;
`--rows` was added for it. Pass 57 took those with a variant bitmask
(`sos` -3.43 %, `cube` -2.52 %). The two `collect()`s pass 55's (I) removed
were the other half and worth a fifth as much on `cube`.

**Where the gathers come from** (`cg_contexts.py`, `--separate-callers=3`,
cube, the fifty-fifth tip — the top of 74 contexts):

```text
6,098  computed_permanent <- resolve_combat <- advance_step
5,976  frozen_effects <- board_keyword_in_scope <- declare_attackers_banded
5,166  computed_permanent <- permanent_value_with <- eval_material_inner
5,114  compute_permanents <- combat_damage_computed <- resolve_combat
4,398  frozen_effects <- board_keyword_in_scope <- declare_blockers
3,828  computed_permanent <- with_frozen_layers <- declare_blockers
2,588  computed_permanent <- resolve_combat <- submit_decision
2,000  check_state_based_actions <- resolve_combat <- advance_step
```

**Combat is over a third of it** — `resolve_combat` accounts for 13,212
between three of those rows, and it is `&mut self`, so a freeze scope is not
the tool. The `declare_attackers` / `declare_blockers` rows are scope-firsts
and their gather is prepaid work for the rest of the scope, not waste.

**(-39) THE DECK BUILDER — 111.8 M -> 34.9 M at the fifty-fourth pass
(-68.80 %), 34.6 M -> 26.6 M at the fifty-sixth (-23.13 % Ir, -14 % wall
clock) and 26.5 M -> 23.6 M at the fifty-eighth (-10.97 %), for twelve pools
+ twelve builds.** It was ~28 % of what a `selfplay_train` actor does per
game and is now ~7 %. Read all three passes' Log entries before proposing
anything here. Pass 54's device was "a definition is memoized, but everything
read off it is not", and `CardBrief` is where a new derived fact belongs;
pass 56's is one level up — **ask what varies with the shape**; pass 58 is
that question asked of the three things pass 56 left (the splash ranker, the
score sort, the land walk) and the answer was "nothing" all three times.

**The copy-cap counter is PAID at the fifty-eighth pass's (E), -7.782 %**, and
it is worth recording that this entry sized it at "~8 % of the build" off one
self-cost row and the measurement agreed to the first decimal. What is left
here is `build_shape`'s 24.77 % residual, which is the `allowed` filter chain
and the two output piles and has been diffuse for three passes; `__memcpy`
9.49 %, which is the twelve pools' definitions being built once; and
`score_brief_with_colors` 6.81 %, whose colour argument genuinely varies per
shape (one attempt on the scorer is refuted at +2.88 %).

**The structural answer this entry used to propose — resolving a pool's
definitions once into a `Vec<Arc<CardDefinition>>` and indexing it, a
signature change through `draft.rs` / `recommend.rs` — is superseded and
should not be started.** The 4,096-slot direct-mapped front cache
(`b10fdebd`) took the same 20.8 % without touching a signature, and the
derived-facts memo (`9cc1175c`) took the re-derivation the index change
would not have.

The leftover table is in the fifty-eighth pass's Log entry, read at its tip
(21,774,018). `generate_sos_pack`, `candidate_label`, `Vec::retain` and the
copy-cap `HashMap` are all off it now, and the sorts fell from ~8 % across
three rows to 2.74 % in one.

**A second-order note the fifty-sixth pass's wall-clock pair exposed: the
tip's process startup floor is 6.5 % higher than the base's** (0.3294 s
against 0.3093 s per 100 processes). `PoolScores` and `SosPacks` add code and
the binary got bigger; on a workload this short that is a real fraction of
what the change saved, and it is why -23.1 % in Ir read -14 % on the clock.
A training actor never pays it — it builds its decks in one long-lived
process.

**The `--use-deck-best` configuration stays CLOSED** — see the fifty-third
pass. `best_build_by(pool, 32, ..)` runs thirty-two `build_random_deck`s per
side per game; that pass's lattice hoist took the judged path from 1.2 to
83.2 games/s against 99.8 for the unjudged one, and every build the fifty-
fourth pass made cheaper is inside it.

**(-38) `battlefield_find` is 4.03 % of the program and has never had an
entry, because it never appears as a function row — it always inlines.**
`self.battlefield.iter().find(|c| c.id == id)`, 556 call sites.
**`scripts/cg_sites.py` is the tool that finds this shape** (added with the
entry): it groups hot addresses by the frame just outside the needle, i.e.
by call site, and it is the only one of the three scripts that can see a
function with no row and no line of its own. At the fifty-third tip the same
table on `--decks sealed` reads 2.40 %, still the largest unnamed structural
cost. The
fifty-third pass took the largest one (`eval.rs:3271`, 1.72 % alone) by
handing the permanent in; the rest of the table, from `cg_lines.py` at that
pass's base:

| call site | Ir | % |
|---|---|---|
| `evaluate_requirement_static` (`eval.rs:3271`) | 18,008,634 | 1.71 — **PAID** (`9bf2ae2e`) |
| `find_card_anywhere`'s first leg (`mod.rs:21269`) | 3,614,532 | 0.34 |
| `all_damage_to_player_prevented` (`mod.rs:12212`) | 2,230,452 | 0.21 — **PAID** (`4a951123`) |
| `auto_tap_for_cost_inner`'s source table (`actions.rs:12626`) | 1,621,360 | 0.15 — **PAID** (`02caa399`, and it measured -0.291 %) |
| `bot::permanent_value` (`bot.rs:3003`) | 1,527,960 | 0.14 — **PAID** (`4a951123`, `permanent_value_with`) |
| `pick_blocks_inner` (`bot.rs:8937` / `8702`) | 1,050,364 | 0.10 |

**Three of those six rows are paid.** `all_damage_to_player_prevented` and
`bot::permanent_value` came off at `4a951123` (the fifty-third pass's (H),
-0.611 % on `fixed`); the auto-tap source table came off at the fifty-sixth
pass's `02caa399` — a `bf_idx` hint on `ManaSourceInfo`, and it measured
**-0.291 % on `fixed`, twice what this table sized it at.** That is
`cg_sites.py`'s own "read the number as a floor" with a second data point
(pass 53's two sites read 0.35 % and measured -0.611 %): **do not decline a
site because its `cg_sites` row looks small.** What is left unclaimed is
`find_card_anywhere`'s first leg (0.34 %) and `pick_blocks_inner` (0.10 %),
and on the floor rule both are worth more than they read.

The rest are candidate (11)'s
shape — a helper that opens with `battlefield_find` and is called from a
battlefield loop. **The structural fix that would take all of them at once
was costed and refused**: putting the `CardId` in the `CardInstance` handle
(so the scan reads a dense array instead of chasing an `Arc` per element)
buys ~20 % of the scan's Ir but doubles the handle to 16 bytes, and
`Vec<CardInstance>` is cloned on every `GameState` clone and every CoW
unshare. Do not take it on the Ir alone.

**(-37) MOSTLY PAID at the fifty-fifth pass: `--decks cube` -16.92 %, and
the entry had sized it at half that.** `has_ctype` and `has_ltype` now ask
the two presence gates that already existed;
`computed_permanent` went 680,960 -> 267,116 calls. Read that pass's Log
entry before touching the rest — it also records the two shapes that lost
(a `OnceCell` around a gate that runs once, and gating on
`!computed_absent()` first).

**The residual was sized at the pass's tip and it is nothing — this entry is
CLOSED.** `has_atype` and `has_stype` are still ungated, and gating them
would need two new predicates (`SetArtifactSubtypes` / `AddArtifactSubtype`
fold into a battlefield-shape scan — Bludgeon Brawl's `brawl_equip_mv`,
`equipped_bonus.set_artifact_types`, the `AddCardType`-with-subtype static;
`AddSupertype` has two emitters, neither a printed card shape). **Do not
write them.** The requirement walker's `OnceCell::try_init` at the
fifty-fifth tip is **117,334 calls at 101 Ir**, against 581,256 at 1,084 Ir
at the pass's base — `computed_permanent` no longer appears in its caller
table at all. The two arms the pair above gated were the whole of it.

**And the fifty-sixth pass closed it from the other side, by building both
gates and measuring them: cube +0.123 %, fixed +0.075 %, sos +0.052 %,
reverted.** Two independent closes agreeing is worth the line, and together
they give the sizing rule this file did not have: **a presence gate is worth
what its arm's *call count* is worth, not what the arm costs when it is
taken.** `has_ctype`'s gate paid because `HasCreatureType` is 410,900 of the
requirement walker's 654,950 calls on cube. Count the arm.

**Ranking rule added by the fiftieth pass, and it found 13 % in one sitting:
ask what is done *twice*.** Not "what is expensive" and not "what is called
often" — what does the program compute, throw away, and then compute again?
`would_accept`'s dry run *is* the action: it clones the state and runs the
action to completion, and every caller then ran the identical action on a
state equal to the one the probe started from. Three commits took that
duplication out of the simulator, the shared pickers and the finalist
evaluators for **-13.128 %**, and none of it shows up as a hot function —
`accept_on` is a thin wrapper, and the work it repeats is spread over
`cast_spell`, `auto_tap_for_cost_inner` and `finalize_cast`, where it looks
like ordinary casting. **The tell is a validate-then-do pair**: a
`would_accept` / `is_ok()` probe followed by the same action being performed.
Grep for those pairs before ranking functions.

**(-32) LARGELY CLOSED by pass 52 (`Bot::next_action_settled` +
`.map(Picked::into_step)` + land-play adopts), for -3.72 % across
pass-52's chain. The 1,040 driver `perform_action` calls skipped there
came off `main_phase_action_with`'s finalist, `pick_stack_response`,
`pick_combat_trick`, the three `pick_land_to_play` blocks and
`legacy_pretap`.** What is left is the pre-validated finalists whose
state `cast_candidates` discards at build time — and on `--decks fixed`
they **never win**: the vanilla archetype decks reach none of the
specialty-cast `castable` blocks, so `castable` is empty and every scored
winner is already lazily probed with `settled: Some` (see the Log's "what
is left" note). The `cast_candidates` refactor that would capture them is
therefore bench-dead for the fixed-Ir number; its ~7 % ceiling is real
only on `all`/`cube`/`sos`. Budget it only if the goal is those pools.
The doc's earlier concern about `ScriptedDecider` survival is
moot: `DeciderKind` derives `Clone` and `ScriptedDecider::kind()`
carries `answers` + `asked` verbatim, so `state.decider.kind().into_boxed()`
reconstructs the queue at the same position. Server / interactive
callers keep calling `next_action` (plain form) precisely so their
`perform_action` result — the event list they broadcast — is not lost.

**ACTOR SCALING — measured 2026-08-24, and it is CLOSED at the scale this box
can test.** The seed list has asked for "games/sec at 1, half, full actor
counts — find contention if sublinear" since the candidates section was
written, and nobody had run it. `release-fast` (mimalloc — the shipped
allocator, which is the point: allocator contention is what a scaling sweep
exists to find, and callgrind's system-allocator build cannot see it),
`--a gang --b gang --games 400 --decks all --seed 11` = 6,800 games a run,
thread counts alternated 1/2/4/4/2/1 in one sitting so host drift moves both
ends. Best of two per count:

```text
threads   wall     games/s   per-thread   speedup   efficiency
  1      100.4 s     67.7       67.7        1.00x      100 %
  2       50.8 s    133.9       66.9        1.976x    98.8 %
  4       25.5 s    266.7       66.7        3.937x    98.4 %
host: Intel(R) Xeon(R) @ 2.10GHz, 4 cores, host_calib_ms 46
```

**Linear to the core count — there is no contention to find here.** What the
sweep also shows is why it wants the alternating order: the two one-thread
runs read 100.4 s and 112.6 s, a **12 % spread** on the same binary minutes
apart, while the two four-thread runs differed by 0.8 %. The long runs drift
more, so take the best of each pair, not the mean.

**What this does not answer**, and the next run should not read it as if it
did: four cores is the whole range this box has. A training host running 8,
16 or 32 `selfplay_train` actors is asking a different question, and this
sweep says nothing about it beyond "the game loop itself is not the shared
resource". Re-run it on the box that matters before sizing an actor count.

**(-34) `cast_candidates` READ FROM THE TOP at the fiftieth tip, and the
answer is one function.** The entry that said "3.0 %, never read from the top"
is closed: `cast_candidates` is **105,425,302 / 7.93 % over 7,238 calls**, and
**93,499,518 of it (7.09 %) is the single `.collect()` that builds the plain-cast
candidate list** — 12,917 Ir a call. Inside that collect, by caller edge:

| row | calls | Ir | % | Ir/call |
|---|---|---|---|---|
| `auto_targets_for_effect_all_slots` | 2,942 | 54,523,024 | **4.13** | **18,533** |
| `can_afford_in_state_with` | 12,986 | 31,754,663 | 2.41 | 2,445 |

Neither is redundant work: the target walk runs 0.41 times per
`cast_candidates` call (only targeted candidates reach it) and the
affordability check 1.8 times (the five hand filters drop the rest first).
**The cost is inside the target walk**, and it is one row —
`evaluate_requirement_static` **113,726 calls / 30,595,070 / 2.32 % at 269 Ir
each, i.e. 38.7 requirement evaluations per targeted candidate**: every
possible target for every slot, against the slot's `SelectionRequirement`. A
card offering three modes pays three whole enumerations, and the requirement
differs per mode, so there is nothing to share between them.

**(-35) HALF PAID (`9bf2ae2e`). The 269 Ir this entry told its taker to look
at was mostly one line: `eval.rs:3271`'s `battlefield_find`, 18,188,014 Ir /
1.72 % of the program, 46 % of the function's own cost.** Handing the
permanent in took `fixed` -0.642 %. What is left is the enumeration, which
is the bot's job, and the four ungated `computed()` arms — see (-37).

**(-35, historical) `evaluate_requirement_static` — 182,532 calls, 33,530,088 self /
2.52 %, the largest non-allocator self row after
`dispatch_triggers_for_events`.** Callers: the target walk 113,726, a
`Map::try_fold` 30,374, its own recursion 28,000 (two arities), the cast's
own closure 1,426. Its callees say the lazy shape is already in place — two
`OnceCell`s per call, `try_init` firing on only **15,562 of the 182,532**
(8.5 %) for 8,834,020 + 9,536,344 of closure. Both cells are keyed to the
*target* being tested, so they cannot be hoisted out of the per-target loop
above them. **Whoever takes this should look at the 269 Ir, not the count:**
the count is the enumeration and the enumeration is the bot's job.

**(-33) `trigger_grant_sources` — 29,448 whole-board grant walks, 14,134,170
Ir / 1.06 %, all of it self, and this is the first pass that could see it.**
The fixed `cg_lines.py` (see **How to measure**) puts the cost inside it at
`option.rs` 4.32 M (the `active_static` peel per static ability), the slice
iteration 7.2 M across `mod.rs`/`mut_ptr.rs`/`non_null.rs`, and `out.push`
0.24 M — i.e. **530,064 battlefield-source visits looking for a
`GrantTriggeredAbility` static that the bench boards do not have.** Callers:
`fire_step_triggers` 14,898 (already hoisted — one walk per call, gated per
card, and it is called once per step) and `statics_granted_triggers_for`
14,550, which is the *per-card shim* and rebuilds the whole list every time.

**The shim's five callers never took the `_with` form its own doc was written
for** ("so a caller asking about every battlefield permanent peels and
resolves the grants once instead of once per card"):
`declare_attackers_banded` 3,960, `fire_combat_damage_to_creature_triggers`
3,826, `fire_combat_damage_to_player_triggers` 2,694,
`resolve_top_of_stack_inner` 2,276, `fire_self_etb_triggers` 1,784. Only the
first three are in a loop with anything to hoist to — an attacker loop and a
damage event — so **hoisting is worth ~6,000 of the 29,448 walks, ~2.9 M,
~0.22 %**, and it means threading a grant list through the combat damage
path. Size it before starting.

**What will *not* work here, and the file has the measurements:** there is no
cheap presence gate, because the gate is the walk — most sources have no
static abilities at all, so the full walk already skips them and a pre-pass
costs the same. A board-level memo with an epoch is (-18), refuted at
+0.727 %. Fusing the question into a walk that already happens is the device
that has now lost four times.

**Ranking rule added by the forty-ninth pass, and it is about how you read
the profile rather than about the code:** **rank the tail, not the function.**
A chain of narrow generators — `main_phase_action_with`'s twenty-two `pick_*`
fallbacks, and anything else shaped like one — is invisible in a self-cost
profile (none of those reached 0.8 %) and invisible in a callee table sorted
by Ir. It shows up only when the **call counts** are read: twenty-two rows at
exactly 2,176 calls each, once per traversal, on a board that had nothing for
any of them. Together they were **4.9 %**. Sorting by Ir will never find one.
Wherever the code reads as a fallback chain, count the rows before costing
them, and gate the whole call rather than reordering its prologue.

**And the corollary that made it safe:** `spec` / `gated_block!`'s debug audit
(run the block anyway in a debug build, assert it produced nothing) is what
lets a mask over-approximate freely. The 18,709-test suite becomes the
mask's proof on real boards, so a bit can be added without re-deriving the
generator's filter by hand. `gated_pick!` is the same macro for a generator
that returns rather than appends.

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

**Ranking rule the forty-seventh pass's last two commits added, and it is
about the *build*, not the code:** `release-fast` / `profiling-fast` have
**no LTO**, so a small non-generic function in `crabomination_base` is an
out-of-line call in every profile this file quotes — while the shipped
`release` profile (`lto = "thin"`) would inline it. **That makes a pure
`#[inline]` change unmeasurable here and possibly worthless there: do not
take one on an Ir number.** What *is* sound is making the callee smaller
than any inliner threshold, which is what `[Keyword]::has_kw` does — a
three-instruction discriminant test in front of a 200-arm `match` no
inliner would ever take. `CardDefinition::is_creature` (~11.5 M / 0.67 %
over ~950 k calls, a `Vec<CardType>::contains`) is the same family and the
same trap: an `#[inline]` on it would read as a win here and be nothing in
`release`. A `CardTypeSet` bitset is the only shape that beats it and it is
(-11)'s staleness hazard again.

**Ranking rule added by the forty-seventh pass, and it is the inverse of the
one this file has applied since the thirty-eighth:** a **presence gate is a
loss where the gather it avoids has already happened.**
`keyword_grant_in_scope` costs one `card_can_grant_keyword` per battlefield
permanent *plus* per command-zone card *plus* per graveyard card — ~93 calls
on a late-game board, more than a `computed_permanent` memo lookup — so
before adding or keeping one, ask whether the caller runs inside a live
freeze scope. `layers_memoized()` answers that without gathering and is the
device to reach for; the forty-seventh pass's (B) is -0.570 % of exactly
this. It cuts the other way too: the land tap's CR 602.5 gate runs from
`&mut self` with no scope open, and there the gate is still the cheap side.

**The clause the forty-eighth pass had to supply, having measured both
sides of it in one sitting:** the question is not "has the gather happened"
but **"does anyone else in this scope read it".** `scan_land_type_rewrites`
is the first computed read in `mana_source_table`'s scope and nothing after
it touches the memo, so trading its gather for
`land_type_change_in_scope` took **-0.747 %**. `board_keyword_matching` is
also a scope's first computed read, but all three of its callers go on to
`compute_battlefield()` or a run of `computed_permanent`s in the same scope
— the gather is *used* — and the same trade there read **+0.30 %** and was
reverted. Read what the scope does after the question before swapping a
gather for a gate.

**And the rule that has now lost four times: do not fuse a cheap per-card
question into a walk that is already happening.** (-8b) hoisting
`card_type_change_in_scope` into `sba_board_scan` (+0.77 %), the `do_untap`
gate (+0.0001 %), `creature_death_possible`'s three-into-one (+0.55 %, and
+1.24 % for the wider version), and the forty-eighth pass's trigger-carrier
bitmask out of `dispatch_board_scan` (+0.58 %). The loads you add to the
first walk cost what you save in the second, every time. What *does* pay is
removing a walk outright — the forty-eighth pass's (B), three whole-board
`static_abilities` scans in the cast's uncounterable check collapsed to one,
**-0.170 %**.

**(-28) `finalize_cast` is 111,559,898 / 6.79 % over 7,172 casts and was read
from the top for the first time at the forty-eighth pass. Two rows came off
it; what is left is one class, and it is not local to this function.** Its
callee table at the forty-eighth tip: `dispatch_triggers_for_events`
21,488,540 over 10,898, `__memcpy` **13,451,064 over 142,796 calls — 19.9 per
cast**, `grow_one` 8,998,811 over **28,878 — 4.03 per cast**,
`Vec::from_iter` 5,313,642 over 21,516, `__rust_alloc` 2,766,997 over
**24,108 — 3.36 per cast**, `find_card_anywhere` 3,272,638 over 21,736.

**The 4.03 growths looked like the class and are not a lever — that half of
this entry is CLOSED.** `Vec::clone` hands back `capacity == len`, so the
first push into any `Vec` after a checkpoint clone or a CoW unshare
reallocates; `finalize_cast`'s four are two of `PlayerData`'s per-turn cast
logs, `spell_names_cast_this_turn`, and `self.stack`, and the same shape is
every large `grow_one` caller in the program (224,481 growths). **A `GrowVec`
newtype whose clone reserves headroom was built and measured at +0.050 %** —
the clone-side cost of `with_capacity` + `extend_from_slice` is the same order
as the growth it removes. See the forty-eighth pass's Log; do not rebuild it.

**What is left of this entry, and it is small.** `CastProfile.card_types` is
a `Vec<CardType>` cloned per cast *and* per `PlayerData` unshare — a
`CardTypeSet` bitset over 14 variants removes both allocations and custom
serde keeps the wire; that is one of `finalize_cast`'s 3.36 allocations per
cast, so size it at ~0.1 %. Everything else here is **diffuse**: the line
profile's largest row inside `finalize_cast` is 331,402 Ir.

**(-29) `Arc::clone_from_ref_in` — 152,062 allocations, 15,118,806 Ir of
allocation and 52,912,414 / 3.22 % of self, the largest unclaimed structural
row.** The CoW unshare of `CardInstance` / `PlayerData`, ~6 per checkpointed
action. Under it: `Vec::clone` 194,386 calls (mostly empty, 42 Ir each) and
`RawTable::clone` 34,220 at ~137 Ir. **The `RawTable` half is PAID** — the
forty-eighth pass's (F), the two per-turn `PlayerData` sets to `IdSet`,
-0.182 %; what is left there is `spells_cast_by_name_this_game`, which is
game-long and is data. **The rest of the entry is (-13)'s and is measured
dead**: the unshare itself is the checkpoint sharing every zone, `clone_from`
lost at +2.60 % and narrowing `GameState` was costed and refused. What has
never been read is the *`CardData` side*: which of its ~110 fields actually
cost something in a `clone_from_ref_in`, given the `Vec::clone`s under it are
42 Ir each and therefore mostly empty.

**(-30) Three callers of `computed_permanent` pay a whole-game gather per
call, and they are named now.** From the forty-eighth tip's caller table
(Ir/call is the tell — ~2,000 is a gather, ~300 a memo hit):
`resolve_combat` **5,682 at 2,080** (diffuse, see (-25)),
`check_target_legality_with_source` **4,692 at 2,206**, and
`push_ward_triggers_for_targets` **1,536 at 2,997** — 26.8 M / 1.63 %
together. `check_target_legality_with_source` opens its *own*
`with_frozen_layers` per call, and nested freezes reuse the outer memo, so
the lever is a scope around the caller's whole target announcement: the cast
path calls it 2,718 times over ~7,640 casts (~1.36 slots a targeted cast, so
~700 gathers), and a `.collect()` caller reaches it **1,616** times — find
that loop first, it is the one with real fan-out. The Ward one cannot be
collapsed: it is ~one call per cast that has a warded target, so there is
nothing to share it with.

**(-27) `computed_permanent` allocates 93,570 `Arc<ComputedPermanent>` over
six games — the largest *named* allocator row, unclaimed, and worth
**~0.2 %, not the 0.75 % this entry first claimed**.** One `Arc::new` per
memo miss. The correction: a *frozen* miss has to hand its `Arc` to the
memo, so only the **~25,358 unscoped** calls can avoid one — and each of
those also pays a full gather (1,953 Ir) that dwarfs the ~130 Ir of
alloc+free. Outside a freeze scope the `Arc` is created, read once and
dropped with nothing to share it with. The shape to cost first: a return type that can be
owned-or-shared (`Cow`-like, `Deref` to `ComputedPermanent`) so the unscoped
path stays on the stack. It touches every caller that writes
`.map(|c| …)` / `.is_some_and(…)` on the current `Option<Arc<…>>`, so size
the call-site churn before starting. **Do not** reach for a pool: (-13)
measured the husk-pool shape at +2.60 %.

**(-31) `improves_this_turn` — READ FROM THE TOP AND REFUTED ON COST by the
forty-ninth pass. Do not build the reuse; see that pass's Log entry for the
call counts.** 842 evaluated finalists across 920 `pick_by_outcome` calls
means at least 499 of them evaluate nothing, so on more than half the ticks
that reach it there is no prior evaluation of the winner to lift. What is
left of the entry is the strength question — the gate costs ~6 % of simulator
throughput — and the analysis below, kept because it is the map of the path.

`main_phase_action_with`'s summon-sick / hold-instants gate calls it once per
winning line (474 calls, 948 combat sims). The `before` half fast-forwards an
idle clone through combat; the `after` half resolves `best`, runs it to
quiescence and fast-forwards *that* through combat. But `pick_by_outcome` ->
`evaluate_action_outcome` -> `evaluate_action_sequence` -> `score_settled_state`
already did exactly that for `best`, on a clone, at depth 0, before recursing
into follow-ups. Only the final scoring function differs
(`eval_material` vs `eval_material_summon_sick_blind`), and both read the same
settled state.

**Three things stop it being a straight lift, and all three are behaviour, not
plumbing.** (i) `evaluate_action_sequence` returns the max over follow-up
sequences, so the value it hands back is not the depth-0 state's — the depth-0
`score_settled_state` call is the one to tee off, not the return. (ii) With
`w.determinize > 0` the outcome eval runs on a *redealt* `sim_start_state`
while `improves_this_turn` uses the true state, so a shared score changes what
the gate answers. (iii) `pick_by_outcome` skips the outcome eval entirely for
`action_outcome_is_temporary` candidates, so a temporary winner has no state
to reuse. Ceiling is about half of the 6.1 %; budget it as a search-plumbing
pass with a determinize-on A/B and a golden-trace diff, not as a drive-by.

**(-26) `main_phase_action_with` — CLOSED by the forty-ninth pass, -4.867 %.**
The tail below the cast block was twenty-four narrow generators, all reached
2,176 times, and gating them behind one board-facts mask was the whole entry.
What is left of the function is search and the engine: `pick_by_outcome`
7.42 % over 920 (a search-quality decision, as this entry suspected — the
finalist count is `EVAL_TOP = 3` and the cost per call is the dry-run, not
enumeration), `would_accept` 6.6 % over 2,036 lazy candidate probes,
`improves_this_turn` 6.1 % ((-31)), `cast_candidates` 3.0 % over 3,506. **Do
not reopen this entry as a whole**; (-31) is refuted and `cast_candidates` was
read from the top at the fiftieth tip — see (-34). The historical entry:

**(-26, historical) `main_phase_action_with` is 552,113,968 / 32.97 % and has never been
read from the top.** Second-largest bot subtree after `pick_attacks_scored`.
Its callee table at the forty-seventh tip:

| callee | Ir | % | calls |
|---|---|---|---|
| `pick_by_outcome` | 119,663,481 | 7.08 | **920** (130,069 Ir a call) |
| `would_accept*` (affordances) | 108,264,447 | 6.40 | — |
| `cast_candidates` | 47,284,337 | 2.80 | 3,506 |
| `perform_action` | 45,339,965 | 2.68 | — |
| `computed_permanent` | 20,737,266 | 1.23 | — |
| `pick_land_to_play` | 15,420,727 | 0.91 | 1,488 |
| `pick_sacrifice_value` | 11,990,848 | 0.71 | 2,176 |

`pick_by_outcome` at 130,069 Ir a call over 920 calls is the thing to read
first, and it is search, not engine — so **check whether the count is a
search-quality decision before treating it as a perf one**, the same answer
(-21) reached for `attack_candidates_for_mcts`. `next_action_inner` runs
inside a `with_frozen_layers` scope (89 % of the program is under one), so
the `computed_permanent` row is memo lookups and layer passes, not gathers;
the probes' clones reset `LayerFreeze` to unfrozen, which is why
`would_accept` gathers.

**REFUTED, forty-seventh pass — do not rebuild either of these.**

* **A lock-free depth shadow on `LayerFreeze`** (`AtomicU32` mirror of
  `state.depth`, written under the mutex, read `Relaxed`, so every unfrozen
  `frozen_effects` / `computed_permanent` / `layers_memoized` answers without
  an acquisition). Correct, suite green, invariants identical, **+0.027 %**.
  An uncontended `std::sync::Mutex` acquire is a handful of instructions.
  The one thing that could overturn it is a *wall-clock* A/B — Ir undercounts
  a lock-prefixed instruction badly and this removes ~100 k acquires over six
  games — and that measurement was not run. Do not reopen it on Ir alone.
* **A per-`CardDefinition` cached bitmask for `sba_board_scan`'s ten
  definition-only flags** (2,277 Ir a sweep, ~65 Ir a card, five inner `Vec`
  loops). Not built, because it is **unsound**: the engine rewrites
  definitions in place through `Arc::make_mut` (MDFC face swap, "loses all
  abilities", keyword grants), and a mutation that turns a bit from `false`
  to `true` leaves a stale `false` — the same hazard that demoted (-11).
  A `Clone` impl that resets the cache covers the `make_mut`-clones-first
  case but not the uniquely-owned one.

**(-25) `resolve_combat` — **168,952,362 / 10.09 % at the forty-seventh tip,
down from 11.86 %**, and two more of its rows are now closed.** The
forty-seventh pass's (A) took `quality_band_assigner` (5,017,771 / 0.29 % over
846 — a full gather per band question) off it outright, and (C)'s
`assign_sectors` fix took the SBA sweeps it drags behind it. The step
machinery above it at the forty-seventh tip: `advance_step` 272,354,213 /
16.26 % over 22,892, of which `resolve_combat` 168,952,362 / 10.09 %,
`do_cleanup` ~26 M over 1,764 (14,914 Ir a cleanup, `finish_cleanup`'s SBA
sweep 16.5 M of it), `fire_step_triggers` ~19 M over 13,134. **The table
below is the forty-fifth tip's** and its absolutes are stale by two passes;
the ratios and the call counts still hold.

| callee of `resolve_combat` | Ir | % | calls |
|---|---|---|---|
| `check_state_based_actions` | 71,644,252 | 4.06 | 2,646 (**27,076 Ir a sweep** — combat sweeps kill, so they are ~10x an ordinary one) |
| `deal_combat_damage_to_target` | 20,368,973 | 1.15 | 2,612 |
| `combat_damage_computed` | 16,121,577 | 0.91 | 3,226 |
| `fire_combat_damage_to_creature_triggers` | 14,152,432 | 0.80 | 3,806 |
| **`computed_permanent`** | **11,968,280** | **0.68** | **3,806 at 3,144 Ir — i.e. a full gather each, outside every scope** |
| `apply_prevention_shields` | 11,216,490 | 0.64 | 3,806 |
| `prevent_combat_to_target` | 10,391,336 | 0.59 | 2,682 |
| `quality_band_assigner` | 5,015,730 | 0.28 | 846 |
| `combat_damage_prevented_to_self` | 3,322,842 | 0.19 | 3,806 |

**The `computed_permanent` row is the one to take**, and it is (-22)'s
lexical route rather than (-18)'s dead one: 3,806 gathers is one per damage
instance, and `resolve_combat` already opens three `with_frozen_layers`
scopes around neighbouring reads (the pair prefix at combat.rs:3594, the
strike-back gate, the player-damage pair). The call is not a literal
`computed_permanent` in `resolve_combat`'s own body — it is inlined from a
helper — so **find it with `--tree=calling` on the tip binary and read the
3,806-call edges, then decide whether it can join a scope that already
exists**. Do not widen a scope across an `&mut self` call to get there; that
is unsound, and `freeze_layers_push`/`pop` make it *look* possible.

**The forty-sixth pass read the same table and narrowed it — two of the
three leads above are closed** (and the forty-seventh closed a fourth —
`quality_band_assigner` is **PAID**). The SBA row is *deaths*: 2,646 sweeps at
27,065 Ir with `remove_from_battlefield_to_graveyard_raw` 16.3 M of it over
2,938, i.e. real work, not a gate. And **the `computed_permanent` row is
diffuse, not one inlined helper**: `--tree=calling` at the 46th tip spreads
those 3,806 calls over many source lines, the largest of which is **748
calls**, and the damage loop already opens three `with_frozen_layers` scopes.
There is no single scope to widen. What is still unread and unclaimed:
`combat_damage_computed` at 4,997 Ir over 3,226 (one gather plus a layer pass
per attacker and blocker — mostly essential) and `prevent_combat_to_target`
at 3,875 over 2,682. **`quality_band_assigner` is paid** — the forty-seventh
pass's (A). What the forty-seventh pass read and did *not* take:
`apply_prevention_shields` is 3,806 calls of a funnel of ~a dozen rare-shield
tests, and its per-line annotation has **no hot line at all** — the cost is
spread over the funnel, so there is no single gate to add.

**(-24) `cast_spell` — the forty-sixth pass took `spell_kind` and the land
tap's effect clone off it, and what is left is (-12).** `try_pay_after_snapshot_mode`
-> `auto_tap_for_cost_inner` is 258,552,458 / 14.60 % over 9,034 at the 46th
tip; `activate_ability` inside it is 153,425,087 / 8.66 % over 18,830, i.e.
**the cost of a cast is 2.5 land taps**. The table below is the forty-fifth
pass's reading, from when the entry was opened; the ratios still hold.

| row | Ir | % | calls |
|---|---|---|---|
| `cast_spell` inclusive | 526,228,783 | 29.07 | 7,640 |
| -> `cast_spell_with_convoke` | 523,041,911 | 28.89 | 7,640 |
| -> `try_pay_after_snapshot_mode` | 281,184,997 | 15.53 | 8,656 |
| -> `auto_tap_for_cost_inner` | 262,553,222 | 14.50 | 18,340 |
| -> `finalize_cast` | 121,353,406 | 6.70 | 7,172 |
| `would_accept` inclusive | 337,688,158 | 18.65 | 5,260 |

**Half of a cast is paying for it**, and (-12) already owns that half. What is
new here is the denominator: 7,640 cast *attempts* for 7,172 finished casts,
so the bot's filtering is already tight and the cost is the real payment path,
not rejected probes. `would_accept`'s 64,200 Ir a probe is the same cast:
the `GameState` clone and drop are ~3,500 of it, so **the probe is not the
clone, it is the cast** — do not go after the clone.

**(-23) The class the forty-fourth pass's (C) and (D) came from, and it is
not exhausted: an allocation on a path every action takes, for a mechanic the
board does not have.** (C) was `vec![false; 2]` 53,838 times for The Ring
(-1.308 %); (D) was a `Vec` per land tap and a `Vec` per attacker-lock
(-0.227 %). **How to find the next one** — `--tree=caller` on the allocator
entries and read the *call counts*, not the Ir:

**The whole allocator caller table at the forty-fifth tip, by call count.**
1,112,945 allocations; `__rust_alloc` is 84,648,612 / 4.68 % inclusive and the
free side 117,236,275 / 6.48 %.

| direct caller of `__rust_alloc` | allocs | Ir | % |
|---|---|---|---|
| `RawVecInner::finish_grow` | 212,245 | 17,256,045 | 0.95 |
| `Vec::from_iter` (nested) | 186,820 | 13,304,418 | 0.73 |
| `Arc::clone_from_ref_in` | 152,062 | 15,001,917 | 0.83 |
| `computed_permanent` | **96,544** | 7,638,352 | 0.42 |
| `GameState::clone` | 79,204 | 4,538,793 | 0.25 |
| `gather_continuous_effects_inner` | 40,704 | 3,311,572 | 0.18 |
| `Vec::clone` | 38,748 | 4,229,994 | 0.23 |
| `RawTable::clone` | 29,428 | 1,812,606 | 0.10 |
| `Box::clone` | 26,386 | 1,458,050 | 0.08 |
| `finalize_cast` | 24,108 | 1,711,585 | 0.09 |
| `ManaPayload::clone` | 19,306 | 1,059,282 | 0.06 |
| `RawTable::reserve_rehash` | 19,280 | 1,084,448 | 0.06 |
| `frozen_effects` | 17,702 | 1,272,260 | 0.07 |
| `Vec::from_iter` (in-place) | 16,230 | 942,347 | 0.05 |
| `can_afford_in_state_with` / `can_afford_from` / `relax_cost_colors` | 12,986 **each** | ~2,300,000 | 0.13 |
| `ManaCost::reduce_generic` | 7,550 | 843,761 | 0.05 |

`can_afford_from`'s three allocations per affordability question are **PAID**
(the forty-fifth pass's (D)): the clone only existed so `reduce_generic` could
mutate, and a `Cow` borrows on the path most costs take.

**Refreshed at the forty-sixth tip: 1,021,777 allocations, down from
1,112,945.** How to get this table right — and the forty-sixth pass got it
wrong first, so read this: `callgrind_annotate --tree=caller` prints a
function's callers as `<` lines *immediately above* its `*` line, and its
callees as `>` lines below. **Take only the contiguous `<` block directly
above the `__rust_alloc` node.** A regex over a window instead picks up `>`
edges belonging to neighbouring nodes and invents rows — that produced two
plausible, entirely fictional leads ("`perform_action_inner` allocates once
per action", "`push_ordered_trigger_candidates` allocates once per
dispatch"). The second was built and measured before the mistake surfaced:
short-circuiting `continue_trigger_ordering`'s copy read **-107,468 Ir /
-0.006 %**, i.e. nothing, and was reverted. `candidates` is empty on nearly
every dispatch, so there was never a copy to skip.

The real table, `<`-block only, by call count:

| direct caller of `__rust_alloc` | allocs | note |
|---|---|---|
| `RawVecInner::finish_grow` | 212,177 | Vec growth, all callers |
| `Arc::clone_from_ref_in` | 152,062 | the CoW unshares |
| `Vec::from_iter` (nested) | 149,696 | the `.collect()`s |
| `computed_permanent` | 96,166 | one `Arc::new(ComputedPermanent)` per memo miss |
| `GameState::clone` | 79,204 | ~4 per clone — the non-`CowBox` fields. (-13) costed narrowing and said no |
| `gather_continuous_effects_inner` | 40,408 | |
| `Vec::clone` | 31,068 | |
| `RawTable::clone` | 29,428 | hash maps cloned inside the state clones. **Unread** |
| `Box::clone` | 26,386 | **Unread** |
| `finalize_cast` | 24,108 | **3.4 per cast, and the cheapest unclaimed named row**: the three per-turn cast logs plus `CastProfile.card_types`' `Vec<CardType>` clone. The logs regrow after every `PlayerData` clone because `Vec::clone` gives capacity == len, so `clear()`ing them per turn does not help. `CastProfile` is serialized in snapshots, so a bitset there is a wire change |
| `RawTable::reserve_rehash` | 19,280 | |
| `frozen_effects` | 17,702 | one per freeze scope |
| `Vec::from_iter` (in-place) | 16,230 | |
| `relax_cost_colors` / `can_afford_in_state_with` | 12,986 **each** | **PAID** — the forty-sixth pass's (E) |
| `auto_tap_for_cost_inner` | 9,544 | |
| `ManaCost::reduce_generic` | 7,550 | |
| `fire_combat_damage_triggers` | 6,530 | |
| `compute_permanent_pass` | 6,298 | |
| `bot::ward_gate_ok` | 4,082 | |
| `spell_kind` | 4,248 | `creature_types.clone()`, all that is left of it after (A)/(C) |

A count in the tens of thousands on a six-game workload is one per action or
one per permanent per action; that is the tell. **Rank by call count and then
read the source.**

**And `grow_one`'s own callers, 224,927 growths / 39,576,235 / 2.19 %:**

| site | growths | Ir |
|---|---|---|
| `Vec::push_mut` | 42,354 | 5,684,749 |
| `finalize_cast` | 28,878 | 8,899,235 |
| `advance_step` | 22,892 | 2,747,040 |
| `gather_continuous_effects_inner` | 13,730 | 4,498,998 |
| `declare_blockers` | 13,122 | 2,142,128 |
| `dispatch_board_scan` | 11,654 | 1,398,480 |
| `auto_tap_for_cost_inner` | 7,550 | 886,420 |
| `effective_mana_abilities_into` | 7,490 | 899,697 |
| `compute_permanent_pass` | 6,338 | 2,492,443 |
| `computed_permanent` | 5,450 | 1,270,798 |

A count in the tens of thousands on a six-game workload is one per action or
one per permanent per action; that is the tell. The Ir column lies here — a
`from_iter` row carries the iterator's own body — so **rank by call count and
then read the source**. `alloc_zeroed` is now zero calls; it was the cheapest
possible find and there is only ever one of those.

**(-22) SCOPE WIDENING PAID, AND LARGE — the fifty-third pass took
`--decks cube` from 7.95 G to 4.05 G (-49.1 %) with three scopes, and the
gather there fell from 32.51 % to 7.99 %.** This entry has said since the
forty-fourth pass that "widening scopes is what reaches those" and it was
right; what it could not say was *where*, because the three loops that
mattered (`dispatch_triggers_for_events`' phase 1, `fire_step_triggers`,
`fire_spell_cast_triggers`) are **dead on `--decks fixed`**. The device that
made them safe is not a new one: each already holds a shared borrow of
`self.battlefield` for its whole body, so the borrow checker has proved no
`&mut self` call happens inside. **Look for that shape before proposing
anything else here.** Use bare `freeze_layers_push`/`pop`, not
`with_frozen_layers` (the closure costs ~0.9 % because the loop's locals go
through its environment), and gate it on a fact the loop already computes.

**The caller table below is the forty-fourth tip's `fixed` reading** and is
kept for its shape; the live numbers are the four-pool table in "Profile of
record".

**(-22, historical) The gather's caller table — the *only* live entry on the gathers,
because (-18) is refuted and scope widening is the one route left.** 53,806 continuous-effect gathers on
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


**(-21) The bot's attack search is 53 % of the program, and it is the largest
single subtree in the profile.** **Refreshed at the fiftieth tip
(1,330,233,580):** `pick_attacks_scored` **706,842,699 / 53.14 %** over 438
calls; `simulate_attack_outcome_once` 699,394,707 / 52.58 % over 1,170
candidates = **597,773 Ir per candidate**, down from 825,854 — the fiftieth
pass took a whole cast out of every simulated cast, and this is where most of
it came from. `sim_step` is now **31,874 passes / 278,835,203 (21.0 %) plus
2,636 checkpointed actions / 72,020,298**, where it was 4,568 / 209,220,325.
**What is left under it is the engine's per-step cost**: `advance_step` at
11,688 Ir a step, of which `resolve_combat` is 55,816 Ir a combat. The
numbers below are the forty-ninth tip's and are kept for the shape.

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
**The forty-sixth pass measured "guarding one promotes the next" directly, and
it is the reason not to take the biggest remaining row.**
`declare_blockers`' `self.blocks_declared_this_turn.push(...)` is
**7,088,925 Ir / 0.41 % over 2,070 pushes — 3,425 Ir each, all of it the
`ColdState` deep copy**. The very next cold write in the same loop
(`blocked_attackers.push`) reads 52,020 over 1,734 *because the group is
already unshared*. Guarding or moving out the first write therefore buys about
1.2 M of the 7.1 M and hands the rest to the second. The honest shape is one
`ColdState` clone per simulated block declaration — CoW working as designed on
a state the bot cloned — and the only lever is a cheaper `ColdState`, where
(-13) already measured `clone_from` losing.

`GameState::deref_mut` was **4,608,931 / 0.24 % over 6 sites** at the 42nd tip
and the survivors are real writes; the standing work is (i) re-read that table
after *any* change that guards or moves a cold write, because guarding one
promotes the next, and (ii) the two survivors —
`creature_deaths_this_turn.push` (2.3 M over 3,104) and
`life_gain_flag_pending.push` (1.4 M over 1,294) — are move-out candidates
like (D)'s: `life_gain_flag_pending` is `#[serde(skip)]` and empty between
batches, so moving it out costs an empty-`Vec` clone (free) and buys ~1.4 M.
`creature_deaths_this_turn` holds `CardInstance`s through the turn, so its
move-out would trade ~2.3 M for a `Vec` clone per checkpoint — probably a
wash, measure before taking it.

**Re-asked at the fifty-third pass and still refused.** The epoch's payoff
side is eight times bigger on the cube pool than on the `fixed` one it was
refuted against — but both of the refutation's reasons are workload-
independent (the counter is not free where the writes are; a ~700 Ir
predicate inlined into its caller is too cheap to memoize behind a call),
and the scope-widening route took 49 % of that pool without any new state.
Take the remaining scopes first.

**(-18) THE BOARD EPOCH — BUILT, MEASURED, REFUTED (forty-fifth pass). Do
not build it again; read the Log entry before proposing anything shaped like
it.** The write counter on `CowBox::deref_mut`, the memo reachable from
`&self`, the recompute-and-compare `debug_assert` — all of it was written,
all of it was sound (the 18,708-test suite ran green with the audit armed),
and it measured **+0.727 %** behind a `Mutex` and **+0.490 %** lock-free.

The two reasons generalise and are the useful part:

1. **The counter is not free where the writes are.** `Arc::make_mut` runs
   945,272+ times over six games, almost all of it `CardInstance`'s own
   handle. Every one pays the increment for a memo that never reads it.
2. **A ~700 Ir predicate inlined into its caller is too cheap to memoize
   behind a call.** The hit rate does not pay back the lost inlining.

**And the thing that *would* pay — the gather, ~1,900 Ir x 48,466 — is
exactly the one whose key is not enumerable.** `gather_continuous_effects_inner`
reads life totals (Aettir and Priwen's base P/T), hand sizes (Kagemaro's
Clutch), graveyard contents, `statics_ignored_this_turn`, and
`evaluate_predicate` through `active_static`'s `WhileCondition` at ~30 sites.
This file said three times that "a board-level memo with an epoch is the
shape". It is not. **The remaining route to those gathers is lexical: widen
freeze scopes, site by site, each one measured** — the forty-fourth pass's (B)
took 1.02 % from two of them and two more written the same day read
-11,778 Ir, i.e. nothing.

What is left of the entry's *numbers*, still unpaid and now without a plan:
`dispatch_board_scan` 24,561,076 / 1.36 % over 53,838 and
`permanents_with_abilities_removed` over the same 53,838 are whole-board walks
with no scope to widen, and `card_type_change_in_scope` is 22.1 M / 1.22 %
over ~30 k calls at ~736 Ir. **Do not reach for fusion there either** —
(-8b) is the same site, and hoisting it into `sba_board_scan` (a walk that
was already happening) read **+0.77 %**. Between (-8b) and this entry the
site has now lost to fusion three times and to a memo twice; and the "ask it less often" shape is **gone**:
the claim that `activate_ability_inner` asks it twice per activation is stale
— callgrind at the forty-sixth tip reads 18,864 calls over 18,830
activations, i.e. once, and one call site is left in that function.

**(-16) READ BY LINE ON `sos` FOR THE FIRST TIME (sixty-first pass), AND THE
VERDICT IS "DIFFUSE" WITH A MEASUREMENT BEHIND IT — including one commit
built against the table and reverted.** `dispatch_triggers_for_events` has
been the largest engine self row since pass 43 and every previous read of it
was on `fixed` or `cube`. `profiling-lines` + `cg_lines.py --in
dispatch_triggers_for_events` on `--decks sos` (1,535,739,641), everything
whose inline chain mentions the function — **91,537,964 Ir, 7.5 % of the
run**:

```text
 14,467,414 (1.18%)  mod.rs:?          <- no line info; the largest single row
  5,708,410 (0.47%)  macros.rs:?
  3,785,518 (0.31%)  mod.rs:22000      is_event_hardcoded's `match ev`
  3,196,160 (0.26%)  mod.rs:464        a ColdState field read (doc-comment line)
  2,948,808 (0.24%)  mod.rs:1815       "
  2,919,532 (0.24%)  mut_ptr.rs:961    slice iteration
  2,868,740 (0.23%)  non_null.rs:444   "
  2,745,712 (0.22%)  non_null.rs:1720  "
  2,718,104 (0.22%)  mod.rs:17108      `if any_static_grant || !station.is_empty()`
  2,359,736 (0.19%)  mod.rs:612        a ColdState field read
  2,222,190 (0.18%)  macros.rs:180
  2,151,812 (0.18%)  mod.rs:17241      `event_subject(ev, &ta.event.kind)`
  2,019,616 (0.17%)  mod.rs:17223      `if is_event_hardcoded(ev, &ta.event)`
  1,587,184 (0.13%)  mod.rs:17154      the FromYourGraveyard scope test
  1,431,576 (0.12%)  macros.rs:332
  1,279,136 (0.10%)  mod.rs:17240      `event_matches_spec(...)`
```

**The largest *named* engine line is 0.31 % and the top two rows have no line
info at all.** That is what diffuse looks like when you finally see it. (The
percentages are of the run total, which carries `name_index()`'s 104.7 M of
startup — see (-46); net of it every share here is ~7 % larger, and the
ranking is unchanged.)

**And the table's most promising row was taken, measured and reverted, which
is the entry's real contribution.** `is_event_hardcoded` reads
5,805,134 Ir / 0.38 % across its two rows, it is asked once per
(permanent x trigger x event), and its answer is a function of the *event*
alone plus one bit of the trigger's scope — the textbook (-45) shape. Three
`u64` masks over the batch (hardcoded-always, hardcoded-if-`SelfSource`, and
the `CreatureDied` skip that folds in Hushbringer suppression and CR 700.4's
replaced deaths), built once per dispatch, allocation-free, with a
`> 64 events` fallback. It reads **+0.123 % on `sos` and +0.187 % on
`fixed`**, and the whole regression is inside the function: `+1,917,670` on
`dispatch_triggers_for_events` and *nothing else in the program moves*.

**The rule that comes out of it, and it is about `cg_lines.py`, not about
this function: a line's Ir is what the instructions attributed to that
source construct cost, not what removing the construct would save.** The
`match ev` was already compiled to a jump the loop had to make anyway; the
mask replaced it with a branch, an `enumerate`, a shift and a test, and paid
more. **Before costing a line-profile row, ask what the loop still has to do
when the line is gone.** Two rows here answer "the same thing".

**(-16) PARTLY PAID (`36e998aa`): the phase-1 board walk runs under a freeze
scope now, which is worth nothing on `fixed` (5.29 % self, unchanged) and
took the function from 59.6 % to 3.91 % *inclusive* on `--decks cube`. The
diffuse `fixed` cost this entry describes is what is left, and it is still
diffuse — the fifty-third pass's line profile puts its largest single engine
line at 3,288,498 Ir / 0.31 % (`mod.rs:16621`, the per-card static-grant
branch test).**

**(-16, historical) `dispatch_triggers_for_events` — 141,288,450 / 7.61 % at the
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

**What is actually left at this site — and it is now less than it was.** Not
fusion, and (forty-fifth pass) **not a memo either**: the board-level
memo/epoch this paragraph asked for was built against exactly this predicate
and measured **+0.490 %** lock-free; see (-18). Four losses at one site.
The remaining shape is to drop the question (asking it less often is out —
`activate_ability_inner` asks it *once*, 18,864 calls over 18,830 activations
at the forty-sixth tip; the older "twice" note was wrong):
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
