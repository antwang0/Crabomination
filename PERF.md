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

**Re-anchored 2026-08-13 at `5034eb2f`** (`release`, mimalloc — the shipped
configuration), the twenty-eighth pass's tip. The previous anchor
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
| 26 | 3,162,657,064 -> 3,132,870,988 Ir (**-0.942 %**) | **A determinism defect and a perf candidate can be the same item.** The engine's hash containers get a fixed hasher (`841dd40b`): `crate::fxhash` (rustc's seedless FxHasher) behind `HashMap` / `HashSet` aliases, replacing `std`'s per-map-seeded SipHash-1-3 on keys that are one or two words — `CardId`s, seat indices, `&'static str` names — where SipHash's setup dominates. Longer-lived candidate (6) sat unpulled for a dozen passes at ~1 %; what moved it was the cube pool's fixed-seed nondeterminism making the same change mandatory. **Check the standing candidate list before designing a robustness fix.** Also carries `125108c1` (coin flips off the game RNG), which cannot move this workload. Base rebuild was **0.115 %** off the recorded tip, the widest this file has seen — a different container, and the row is nine times it. |
| 27 | 3,132,892,846 -> 3,115,598,474 Ir (**-0.552 %**) | **A helper that re-finds a card its callers hold.** `granted_abilities_of(&CardInstance)` beside `granted_abilities_with(CardId)`: the `CardId` form opened with a `battlefield_find` and its three hot callers were already iterating the battlefield — `available_mana`'s untapped-producer loop (63,656 of 201,834 calls), `effective_mana_abilities_with` (which had *just* done the same find), the bot's activatable sweep. `available_mana` **1.14 -> 0.97 %**, the `granted_abilities_*` family 1.39 -> 0.81 %. This is candidate (11)'s shape and it recurs; the base rebuild here was **0.0007 %** off, the tightest in the file. |
| 28 | 3,116,508,803 -> 2,890,337,225 Ir (**-7.257 %**) | **The CoW unshare, read off the profile by name** — four rows, all the same defect. Trigger dispatch stops building a `Vec` per permanent (`e02767aa`, -1.250 %: three `extend`s per permanent per dispatch, 945,812 times, most producing an empty vector; 155 M self, 69 M of it `alloc::vec` + `alloc::raw_vec`). The step-bounded may-play sweep gets a presence gate (`078b8cef`, **-2.189 %**: `clear_step_bounded_may_play` took `iter_mut` over every hand, graveyard, library and exile at *every step transition* for a CR 702.94 miracle window — **61.3 M / 1.97 % on one source line**, since `iter_mut` on a `CowBox` unshares and reaching a zone through `Player` unshares the whole `PlayerData` first). The affordance probe template stops stripping libraries (`2f70affb`, **-2.024 %**: the strip's comment was true when zones were plain `Vec`s, and cost a `PlayerData` unshare per player per template once they became `CowBox`es). And three periodic writes are guarded by a read (`5034eb2f`, **-1.998 %**: `cards_drawn_this_step = 0` per player per step boundary, 57.9 M / 1.96 %; `opponent_cast_spell_since_your_turn`; `tapped_land_for_mana_this_turn`). |

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

**Twenty-ninth pass — the same defect one level up: stop paying for the
clone, then make the clone cheaper.** Base `797040ba` read
**2,890,336,504 Ir** against the twenty-eighth pass's recorded
2,890,337,225 (**721 Ir apart, 0.00002 %** — the tracker-only commit
between them changed nothing, and this is the tightest build-to-build
agreement this file has recorded).

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-14 | "An opponent cast a spell" becomes a seat mask (`88c178f5`) | 2,890,336,504 Ir | 2,869,737,485 Ir (**-0.713 %**) | The twenty-eighth pass guarded this write with a read; the residual was still **14,498,464 Ir / 5,800 unshares** in `finalize_cast` plus 2,330,929 / 1,718 at cleanup, because after a *genuine* false→true flip the guard cannot help. One bool per seat is now one `u64` on the hot `GameState` — `\|=` per cast, `&= !bit` at cleanup, `& bit` on the read. `Player::deref_mut` inclusive 94,562,722 → 84,058,045; the other half of the delta is the allocator and memcpy work the clones dragged behind them |
| 2026-08-14 | A cold group for the rarely-written tail of `PlayerData` (`645b978d`) | 2,869,737,485 Ir | 2,832,745,260 Ir (**-1.289 %**) | `PlayerData` is 158 fields and **24,852 deep clones per six games at ~3,300 Ir each**. Fifteen own heap and are written by a handful of cards or never: `name` (2,811,379 Ir of `String::clone` alone), `commanders`, `emblems`, `epic_spells`, `draft_notes`, ten per-turn registries — 12.7 M in explicit field-clone calls plus their share of the struct memcpy and malloc. Now `PlayerCold` behind one `CowBox`, the `GameState`/`ColdState` device one level down; `#[serde(flatten)]` keeps the wire format identical. `Arc::clone_from_ref_in` **380,386,491 → 249,073,124 inclusive, -34.5 %**; `_int_malloc` 175,068,200 → 163,751,005 |
| 2026-08-14 | `find_card_anywhere` walks the libraries last (`dbd3efeb`) | 2,832,745,260 Ir | 2,819,346,784 Ir (**-0.473 %**) | The walk was battlefield, then per seat graveyard / hand / **library** / ante, then exile, then the stack. A library is ~55 cards against a hand's ~7, so every lookup that ended on the stack, in exile, or in the second seat's hand walked both libraries first. `find_card_anywhere` self was **36,110,078 / 1.27 %** across six file attributions. Order is not part of the contract — a `CardId` names one instance, so at most one zone holds it — and the `_mut` sibling was reordered with it so the pair stays in step |
| | **cumulative, twenty-ninth pass** | **2,890,336,504 Ir** | **2,819,346,784 Ir (-2.456 %)** | callgrind on the fixed six-game workload; bench output byte-identical on every row, and the pass as a whole diffs byte-identical against `797040ba` over 5,100 games |

**What the pass corrects, and it matters for the next sweep.** The
twenty-eighth pass's handoff listed two candidates at 0.95 % / 7,456 calls
— `&mut self.cold` and `remove_from_hand`. **They were one site
double-counted.** At `797040ba` the sweep reads `remove_from_hand`
28,073,333 / x7,456 and `GameState`'s `&mut self.cold` **232,920 / x7,764
(30 Ir a call — it almost never clones)**. A `=> …deref_mut` line in
`--auto=yes` output names the *call*; the accessor's own definition line
carries self cost separately, and reading both as sites counts one unshare
twice. Check the Ir-per-call before ranking a hit: ~30 means the handle was
already unique.

**And `remove_from_hand` is not a candidate at all**, which is worth
recording because it looks like the biggest one. Removing a card from hand
*must* unshare — but so must the eight `PlayerData` writes `finalize_cast`
does immediately after (`spells_cast_this_turn`, `spell_ids_cast_this_turn`,
…), each of which reads 30 Ir on the profile precisely because
`remove_from_hand` already paid. **Move the first write out and the second
inherits the unshare**; the count is irreducible. The same trap kills
`attacked_this_turn` (`creatures_attacked_this_turn` is written on the next
line) and any scheme for hoisting the zones out of `PlayerData`. *A CoW
unshare site is only removable when it is the seat's **only** write in that
action* — which is exactly what made the seat mask pay. What is left on
`PlayerData` is therefore **cost per clone, not clone count**, and that is
the second row.

**The sweep recipe needs updating too**: with `PlayerData` carrying its own
`DerefMut`, LLVM inlines `Player::deref_mut` and the `=> …deref_mut` lines
stop naming the expensive sites (40 sites, 3,277,866 Ir / 0.12 % between
them at the tip). Rank unshares by `--tree=caller` on
`alloc::sync::Arc<T,A>::make_mut` instead — 212,323,729 / 7.50 % inclusive
at the tip, and the one leaf that still describes the whole class is
`clone_from_ref_in` at 8.79 %.

**Thirtieth pass — two linear scans that were ordered wrong, not too
long.** Base `b82f3113` read **2,832,747,493 Ir** against the twenty-ninth
pass's recorded 2,832,745,260 (**2,233 Ir apart, 0.00008 %**).

| date | change | before | after | how measured |
|---|---|---|---|---|
| 2026-08-14 | The zone scan looks at the stack before both libraries (`8241d092`) | 2,832,747,493 Ir | 2,816,245,576 Ir (**-0.583 %**) | `find_card_anywhere` visited battlefield, then per seat graveyard / hand / library / ante, then exile, then the stack — and the cast path asks it for spells that are *on the stack* (`finalize_cast` 21,362 calls, `perform_action_inner` 37,280, `activate_ability` 18,386, `cast_spell` 7,456), so those calls walked ~35 cards a seat of library first. **Self cost 36.1 M / 1.27 % over 104,240 calls, ~65 % of it in calls that fell through everything.** A `CardId` names one object and lives in one zone (copies and tokens are minted fresh from `next_id`), so the visit order picks how long the answer takes, never which answer comes back; `duplicate_zone_id()` is a debug-only whole-state check the golden-trace games now run after every action. Self **36.1 M -> 19.6 M (-46 %)** at unchanged calls |
| 2026-08-14 | The gather's stateful board pass walks `sa_cards` when it can (`6aee2973`) | 2,816,245,576 Ir | 2,781,048,142 Ir (**-1.250 %**) | `gather_continuous_effects_inner` ends with an *ungated* walk of the whole battlefield carrying five unrelated passes. Two read `static_abilities` and so can only emit for a card already in `sa_cards`; the other three are keyword/flag reads (unleash, living metal, suspect) that are false on almost every board. Every card paid three flag tests and two empty slice iterations for nothing — **4,985,332 + 7,477,998 + 7,477,998 Ir on three source lines** over 127,878 gathers. `any_suspected` joins the pre-scan fold; with all three bits clear the walk *is* `sa_cards`, in battlefield order, so the emitted sequence is unchanged. `&mut dyn Iterator`, not a `Box`, so choosing costs no allocation. Gather self **219.4 M / 7.75 % -> 185.9 M / 6.68 %** |
| 2026-08-14 | An `_of(&CardInstance)` twin for `effective_mana_abilities` (`005c1d33`) | 2,781,048,142 Ir | 2,776,663,732 Ir (**-0.158 %**) | Candidate (11)'s shape at the site the twenty-seventh pass's row named and did not finish. `effective_mana_abilities_with(card_id, scan)` opened with a `battlefield_find` and all three callers ask from inside a `battlefield.iter()` — `mana_source_table_inner`, `untapped_producers_of_inner`, `untapped_relevant_source_exists_inner` — 54,570 calls each re-finding the card the loop was standing on. The `CardId` form stays as a find plus delegate. Self **13.9 M / 0.49 % -> 9.5 M / 0.34 %** |
| 2026-08-14 | The merge with the concurrent session (`a4960740`) | 2,776,663,732 Ir | 2,776,361,994 Ir (**-0.011 %**) | Not a row so much as the honest tip: the merge keeps this session's zone order and adds the other's `find_card_anywhere_mut` reorder, worth **301,738 Ir**. Measured because neither session's cumulative describes the merged code |
| | **cumulative, thirtieth pass** | **2,832,747,493 Ir** | **2,776,361,994 Ir (-1.990 %)** | callgrind on the fixed six-game workload; bench output byte-identical on every row, including the merge |

**The second collision this file has recorded, and this one was on the top
candidate.** A concurrent session pulled `find_card_anywhere` from the same
base within the hour and landed `dbd3efeb` — libraries after every small
zone and after the stack, **-0.473 %**. This session's `8241d092` moved the
*stack* to second as well, because the cast path is what asks
(`finalize_cast` 21,362 calls, `perform_action_inner` 37,280,
`activate_ability` 18,386, `cast_spell` 7,456 all look up spells that are on
the stack), and read **-0.583 %** from the same base. The merge keeps the
stack-second ordering and takes their `_mut` sibling reorder, which this
session had left alone. **Two independent measurements of the same
candidate, 0.110 % apart, are the size of the ordering difference and not
noise** — the first collision (twenty-fifth pass, functionally identical
source) was 0.042 %. *Neither session's cumulative describes the merged
tip*, which is why the row below exists.

**What the pass leaves behind: the filter is the *order* of a scan, not its
length.** Both large rows are a linear walk that was correct and visited its
cases in the wrong order — the zone scan looking at 70 library cards before
the stack, the gather's board pass testing three always-false flags before
the two passes that could only fire for a card already on a short list.
Neither shows up as an expensive function: `find_card_anywhere` was 1.27 %
spread over six file attributions, and the gather's three lines were 0.18 %,
0.26 % and 0.26 %. **The tell is a `for`/`find` whose *first* branch is the
rarest case**, and it is cheaper to check than to profile: read the loop and
ask which arm the workload actually takes.

**And a candidate this pass costed and did *not* take.** Candidate (13)/(5),
the `Keyword` bitset, was measured before being written: `Keyword::eq` is
**14,393,938 Ir / 0.51 % self**, but it is **~1.3 M calls at 11 Ir spread
over thirty call sites** (`printed_color_set` 209,154, `has_keyword`
170,112, `board_keyword_in_scope` 135,886, `can_block_attacker_computed`
115,502, then a long tail), and the gather pre-scan's keyword loop is
0.23 %. So the whole addressable total is ~0.6 %, against a representation
change that has to reach **7,716 `keywords: vec![…]` literals in the
catalog** and be maintained at the ~10 runtime sites that push a keyword.
Ranked down accordingly; the arithmetic is in the candidates section so the
next run doesn't re-derive it.

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

**(-1) The CoW-unshare sweep — re-run it, but with the new recipe.** The
twenty-eighth pass took four of these for -7.257 % and the twenty-ninth two
more for -1.993 %. The `=> …deref_mut` sort that found both no longer
resolves (see the twenty-ninth pass's Log block): `Player::deref_mut` is
inlined now, and the 40 surviving `deref_mut` call lines are 0.12 % between
them. **The sort that still works is `--tree=caller` on
`alloc::sync::Arc<T,A>::make_mut`** (7.50 % inclusive at the tip). What is
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

11a. **The zone-walk family, and it is the same shape as `dbd3efeb`.**
    Four hand-written walkers ask "where is this card": `find_card_anywhere`
    (paid, -0.473 %), `find_card_anywhere_mut` (reordered with it),
    `find_card_zone` and `find_card_owner`. The last two still scan each
    seat's **library third**, before the other seat's hand and before exile;
    `death_was_replaced` scans exile then both libraries on every dispatch.
    **None of the three appears above the 99 % threshold on this profile**,
    so reordering them is a consistency change, not a measured win — do it
    as part of whatever *does* touch them, and do not claim Ir for it. Noted
    here because a family of walkers that must agree is worth keeping in
    step, and three of five are now out of step by cost.

    **What is left of the paid one, measured at the merged tip.**
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
