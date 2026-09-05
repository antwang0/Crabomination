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

# WALL CLOCK, AND IT CAN BE READ AFTER ALL — but only paired. One run of each
# side cannot resolve anything here (`games_per_s` reads 212-407 across a day
# on the same binary), so alternate the two binaries A/B/A/B... and take the
# median of the per-pair ratios: every drift the host has lands on both sides
# in the same proportion. Measured at the hundred-and-sixteenth pass, 16 pairs
# resolved a -2.011 % Ir change as **+2.62 % median games/s** (paired mean
# +1.47 %, per-pair sd 4.6 points) against a single-run spread of 237-276.
#   python3 scripts/bench_ab.py /tmp/base_bl /tmp/cand_bl 16
# ⚠ It separates ~2 % from zero and does NOT separate 0.3 % from zero. **Ir
# stays the signal**; this is the confirmation that an Ir win is a wall-clock
# win, not a replacement for the Ir reading. Both binaries must be the same
# profile and feature set.

# determinism across thread counts (opt-in; doubles the run, so off the
# throughput reading above). Replays the identical --bench workload at a
# contrasting thread count and asserts the order-independent outcome matches
# — the aggregate is a sum over seed-fixed jobs, so it must. Clean at the
# pass-52 tip: `thread_determinism ok (3 vs 1 threads identical)`. See TODO
# filter 23 (`1c304384`).
CRAB_THREAD_CHECK=1 cargo run --release --bin bot_ladder -- --bench

# WHAT A CAPPED GAME WAS DOING. `undecided_by cap N` names the count and stops
# there, so "which board, which loop" has cost a rebuild every time it has been
# asked. Read once, on the capped game only, so the throughput path pays
# nothing — the `CRAB_SBA_CENSUS` shape. Prints the action/turn count, the
# stack depth, per-seat life/board/untapped/hand/graveyard/library/pool, and
# two tallies by name: **what is on the stack** and **what is on the board**.
# A runaway names itself in one of them — an unbounded stack is a trigger or
# activation loop, an unbounded battlefield is a token loop; the pool and the
# untapped count beside them say whether a repeated activation was *paid for*.
# It found the Pentad Prism / Gravecrawler stall on its first run.
CRAB_CAP_DIAG=1 target/profiling-fast/bot_ladder --a gang --b gang \
  --games 34 --threads 1 --seed 2 --decks cube
# ⚠ `CRAB_CAP_DIAG=<n>` reports every game past `n` actions, capped or not,
# which is the ONLY way to see a *slow* game: one that decides is never
# "undecided", and `--decks all --seed 43 --games 370` hides a nine-minute
# game behind `0 undecided`.

# WHICH GAME DIVERGED, AND AT WHICH ACTION. Every game the in-process paired
# loop plays is written as its golden-trace text (one line per accepted
# action with the board after it) to `<dir>/<job seed>_<pair>_<seat
# order>.txt`. `bot_ladder`'s stdout is outcomes only, `--bench` reaches one
# pool and the golden traces are one pairing, so "two binaries on one seed
# play different `cube` games" was invisible until it moved a callgrind
# total by 2.6 %. Dump both sides and `diff -rq` the directories: the first
# differing file names the pairing, the first differing line the action.
# `CRAB_CAP_DIAG=2` on a binary without the instrument prints every game's
# final board, which is enough to name the pairing from the outside.
#   CRAB_DUMP_TRACES=/tmp/trA target/profiling-fast/bot_ladder --a gang \
#     --b gang --games 6 --threads 1 --seed 1 --decks cube
# ⚠ AND CHECK THE CATALOG BEFORE CALLING A DIVERGENCE A BUG. The first use
# of this instrument chased a "layout-dependent game" for four builds; the
# base binary predated a rebase that had pulled two concurrent catalog
# commits (a Moonshadow rewrite among them). **An A/B's two sides are built
# from one base tree**, and `git log --oneline --stat <base>..HEAD --
# crabomination_catalog crabomination_base` is the check when a rebase
# lands between them.
#
# WHERE DID THAT LIFE COME FROM. Prints every single life adjustment of at
# least `n` with the seat, the turn and the running total. A compounding
# source reads as a doubling series, a one-shot as a single row — which is how
# the `i32::MAX` life totals in the stall sweep were pinned on Beacon of
# Immortality (1,580 / 3,161 / 6,322 / 12,644, one doubling every other turn).
CRAB_LIFE_WATCH=1000 target/profiling-fast/bot_ladder --a gang --b gang \
  --games 400 --threads 2 --seed 53 --decks all

# allocator A/B — mimalloc is the default now, so the *system* allocator is
# the opt-in side. A feature change on the engine crate is a full rebuild, so
# the variants need separate caches; /target-mi/ is gitignored.
cargo build --release -p crabomination --bin bot_ladder
CARGO_TARGET_DIR=target-mi cargo build --release -p crabomination \
  --bin bot_ladder --no-default-features

# profile-guided build — OPT-IN, and worth -23.8 % on `fixed` / -23.4 % on
# `cube` (ninety-first pass; see the Baseline). Nothing turns it on by
# default, so every committed number in this file stays a plain `release-fast`
# number: quote a PGO reading only against another PGO reading. Needs
# `rustup component add llvm-tools` — the system llvm-profdata is 18.1.3
# against rustc's 22.1.2 and fails on the format.
scripts/pgo_build.sh                 # bot_ladder, release-fast
scripts/pgo_build.sh selfplay_train  # any other bin, with PGO_TRAIN set

# instruction-level profile (deterministic; no `perf` in the routine image).
# Profile the system allocator: valgrind replaces malloc, so a mimalloc build
# measures the interception, not the program. `profiling-fast` is
# `release-fast` + debuginfo: same opt settings as the A/B binaries, so the
# attribution describes the code the Log rows move, and the engine rebuilds
# in ~3.5 min instead of ~24. (`profiling` inherits `release`; use it only
# if you need to attribute LTO'd code.)
cargo build --profile profiling-fast -p crabomination --bin bot_ladder \
  --no-default-features
#
# THE CACHE AND BRANCH AXIS — what Ir cannot see. Same binary, same recipe;
# deterministic, so it A/Bs like Ir. Read by function with `cg_cache.py`
# (one column; `--rate` for the badly-predicted small function). The
# (-248) reading is in "Profile of record": I1 misses 4.03 % of Ir, the CoW
# unshare's clone at one per 8.6 instructions, mispredicts 11.4 %. Not part
# of the three-pool gate; run it on a leg that touches a hot `match` or a
# clone path, and let `bench_ab.py` arbitrate.
valgrind --tool=cachegrind --cache-sim=yes --branch-sim=yes \
  --cachegrind-out-file=cache.out target/profiling-fast/bot_ladder \
  --a gang --b gang --games 6 --threads 1 --seed 1 --decks cube
python3 scripts/cg_cache.py cache.out Bcm        # or I1mr, D1mr, ...; N rows; --rate
#
# ⚠ CALLGRIND MERGES MONOMORPHIZATIONS. It demangles the symbol and keys the
# function by the demangled name, so all eleven instances of
# `Arc::clone_from_ref_in` land in ONE row and the question "which T is being
# deep-copied" cannot be asked of a default dump. Add `--demangle=no` and they
# separate by their legacy-mangling hash suffix (`…clone_from_ref_in17h<hash>E`);
# `cg_edges.py --callers/--callees <hash>` then identifies each by the clone
# helpers it calls (`CardMemo::clone` is `CardData`, `String::clone` is
# `PlayerCold`, `RawTable::clone` + `SmallVec::extend` is `PlayerData`).
# That reading is the hundred-and-first pass's entry point; `(-99)` had read
# the same copies by *calling context* four times without it. `readelf -sW`
# ranks the instances by code size first, which is a free pre-sort.
#
# ⚠ **THE `--games 6` DUMP IS NOT SIX GAMES' WORTH OF WORK, AND ~0.5 % OF IT
# NEVER SCALES.** Measured at `43844faf` on `cube`, one thread, seed 1:
#   --games 1   (paired mode plays 0)      1,337,263 Ir   <- true startup
#   --games 6                          2,502,898,733 Ir
#   --games 18                         5,961,254,552 Ir
# Startup proper is 0.053 % and can be ignored. What cannot is the class of
# cost that is **once per distinct card name per process**:
# `CardDefinition::wants_converge` scans the definition's `{:?}` rendering
# for `ConvergedValue` behind a two-level cache, and that is **217 calls /
# 12,426,187 Ir on `--games 6` against 262 / 14,968,576 on `--games 18`** —
# 0.50 % of the six-game baseline, 0.25 % of the eighteen-game one, and
# ~nothing in the 10-30k-game gate runs the engine actually serves. It is not
# a defect (see its doc comment: the string scan is deliberately the ONE
# oracle for converge, and a hand-written variant walker is the rot this
# codebase has been bitten by) — but a change that moved it would read as a
# 0.5 % win that no real run will ever see. Ask whether a row saturates
# before ranking it: run the dump at two game counts and compare the row, not
# the total.
#
# ⚠ **ASK WHO CALLS THE LIBC ROWS.** The self table is read top-down for
# *engine* names, and `(-169)` sat in it for eleven passes as
# `__strncmp_avx2 7,109,249 (0.80 %)` and `getenv 5,475,482 (0.62 %)` with
# nobody asking whose they were. One command answered it:
#   python3 scripts/cg_edges.py cg.out --callers getenv
# and the answer was `GameState::adjust_life` asking `env::var_os` per life
# adjustment — 1.5 % of `fixed`, invisible to the allocation census, the
# growth census and the line profile alike, because none of them looks at
# libc. `memcpy`, `memcmp`, `strncmp`, `getenv`, `qsort`: name the caller
# before ranking the next engine row.
#
# WHO CALLS THE CALLERS — two levels of caller for one row, which is what a
# std generic needs: `--callers` gets you `Vec::from_iter`, and the engine
# function that owns the cost is one hop further up. **Needs `--demangle=no`**,
# because a demangled dump merges every `from_iter` monomorphization into one
# row and the second hop then cannot tell which of them fed the first. Found
# `(-156)`.
#   python3 scripts/cg_chain.py cg.nd 'FnMut' --top 4 --up 2
#
# WHICH HOT FUNCTIONS PAY A FRAME THEY ALMOST NEVER USE — the instrument for
# `(-129)`'s rule. Joins the dump's call counts against the binary's
# disassembly and ranks by `calls x prologue instructions`, with the body's
# `call` count beside each row. **A row is a candidate, not a finding**: 0
# body calls means the frame is the code (`has_keyword`'s four scan loops),
# 1-3 with a hot caller is the shape. Needs the same binary the dump came
# from.
#   python3 scripts/cg_frames.py cg.out target/profiling-fast/bot_ladder
#
# ⚠ AND `cg_contexts.py` UNDER-READ ITS Ir COLUMN BY 28x until `00b17a18` —
# it took only a cost line whose position starts with a digit, and callgrind
# subposition-compresses those to `*`, `+2`, `-96`. Any inclusive-Ir figure
# quoted from that script before that commit is wrong; its *call* counts were
# always right.
# ⚠ AND `cg_symbolize.py`'s PREMISE IS STALE for these builds: valgrind 3.22
# in the current image *does* read `profiling-fast`'s symbol table, so the
# dump already carries names and the script is a no-op on it (it reports
# "2/43 addresses in a FUNC"). Harmless, but do not read that line as a
# failure, and pass the dump straight to `cg_edges.py` when names are present.
# THE WHOLE THREE-POOL A/B AS ONE SCRIPT, and on warm caches it is ~10
# minutes, not the hour the per-step budgets imply. One tree, one target dir,
# and the base's callgrind runs overlap the candidate's build:
#   git stash push -m ab
#   cargo build --profile profiling-fast -p crabomination --bin bot_ladder \
#     --no-default-features && cp target/profiling-fast/bot_ladder /tmp/base
#   git stash pop
#   ( for p in cube fixed sealed; do valgrind --tool=callgrind \
#       --callgrind-out-file=/tmp/cg.$p.b.out /tmp/base <args> ; done; touch /tmp/b.done ) &
#   cargo build --profile profiling-fast … && cp target/profiling-fast/bot_ladder /tmp/cand
#   while [ ! -f /tmp/b.done ]; do sleep 10; done
#   for p in cube fixed sealed; do valgrind … /tmp/cand … ; done
# The stash is safe here where PERF's worktree rule warns it is not: nothing
# rebases mid-run, the tree returns to the same commit both times, and the
# base binary is *copied out* before the pop. Delete both 215-MB binaries as
# soon as the six dumps exist.
# A second CARGO_TARGET_DIR (gitignored: /target-probe/) lets the *next*
# candidate build while the current one runs under callgrind — callgrind is
# single-threaded and contention-immune, so the overlap is free. Two cargo
# builds at once is not: on 4 cores they take ~1.5x each.
#
# **Take the BASELINE in a detached worktree, not by stashing** (ninetieth
# pass). `git worktree add <dir> <base-sha>` gives the A side its own tree and
# its own target dir; apply the patch *in that worktree* for the B side, so
# both halves are built by one warm cache and the main tree stays free for the
# suite and the release build. Stash/unstash cannot do this here: a concurrent
# session lands engine commits every few minutes, so the tree you unstash into
# is no longer the tree you measured against. Cost is one extra cold build of
# the worktree's target dir; `git worktree remove --force <dir>` reclaims it.
#   git worktree add /tmp/basetree <base-sha>
#   (cd /tmp/basetree && cargo build --profile profiling-fast \
#      -p crabomination --bin bot_ladder --no-default-features)
#   … callgrind the three pools …
#   (cd /tmp/basetree && git apply /tmp/candidate.patch && cargo build …)
#   … callgrind the three pools again …
# Ir is contention-immune, so the *release* build for the `--bench` gate can
# run in the main tree at the same time as the candidate's callgrind runs.
# **For the ACTOR the flag has to name `crabomination_ml`**, not the engine:
# that crate has its own `mimalloc` default *and* its own `#[global_allocator]`,
# so `-p crabomination --no-default-features` leaves `selfplay_train` on
# mimalloc and callgrind measures the interception. The recipe is
#   cargo build --profile profiling-fast -p crabomination_ml \
#     --bin selfplay_train --no-default-features
#   CRAB_NO_JITTER=1 RUST_MIN_STACK=33554432 valgrind --tool=callgrind \
#     --callgrind-out-file=cg.actor.out target/profiling-fast/selfplay_train \
#     --actors 1 --games 60 --steps 1 --seed 7 --out /tmp/actorprof
# and the same `grep -c libmimalloc` check on the dump applies.
# ⚠ AND **`grep mimalloc` IS NOT THE CHECK** — mimalloc's symbols are `mi_*` /
# `_mi_*` and it is statically linked, so both `nm -C <bin> | grep mimalloc`
# and `grep libmimalloc <dump>` return zero on a mimalloc binary, i.e. the same
# answer as on a correct one. The hundred-and-ninth pass lost a profile to it
# (read 6.7-7.4 % low across all three pools). Positive tests:
#   nm <bin> | grep -cE " (T|t) (_)?mi_"     # 0 = system allocator
#   grep -c 'fn=.*mi_' <dump>                # 0 = system-allocator run
# and the row table is the backstop: `_int_malloc`/`_int_free`/`malloc`/`free`
# is glibc, `mi_theap_malloc_aligned` is mimalloc.
# `-p crabomination` is load-bearing for `bot_ladder`. Drop it and
# `--no-default-features` does not reach the engine crate: the binary keeps mimalloc, callgrind
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

# WHICH BUFFER ALLOCATED — a source line per allocation, which `cg_growth.py`
# cannot give: it ranks the *callers* of `finish_grow` and stops at
# "declare_blockers, 19,616 growths". Needs `--dump-instr=yes` AND the
# **`profiling-lines`** binary: `profiling-fast` is `split-debuginfo =
# "unpacked"`, so every `DW_TAG_inlined_subroutine` lives in a `.dwo` and
# `addr2line -i` answers with the OUTERMOST frame — all eight sites in
# `resolve_combat` came back as the one line `resolve_combat` calls. Packed
# DWARF resolves the same eight addresses to eight different lines. `(-158)`.
RUST_MIN_STACK=33554432 valgrind --tool=callgrind --dump-instr=yes \
  --callgrind-out-file=cg.lines.out target-lines/profiling-lines/bot_ladder \
  --a gang --b gang --games 6 --threads 1 --seed 1 --decks cube
python3 scripts/cg_alloc_sites.py cg.lines.out \
  target-lines/profiling-lines/bot_ladder declare_blockers
#
# BUT A FUNCTION'S OWN LINES NEED NO LINES BUILD. `profiling-fast` has
# `debug = true`, so the ordinary three-pool dump already annotates the
# outermost frame's source: `callgrind_annotate --auto=yes --context=0
# cg.cube.<tag>.out crabomination/src/game/mod.rs` prints every costed line
# (self on plain lines, inclusive on the `=>` call edges), and a 20-line
# parser ranking them inside a line range (the scratchpad's `annlines.py`,
# `(-244)`) is the whole instrument. What it cannot do is name an *inlined*
# callee's line — that cost sits on the callee's file (`vec/mod.rs:*`), and
# only the packed-DWARF build above resolves it (`(-158)`, `(-243)`).
# ⚠ AND SPLIT THE GROWTHS BY ALLOCATOR ENTRY BEFORE PICKING A TOOL.
# `--separate-callers=2` puts `malloc` vs `realloc` beside each grow context:
# a `realloc` row is a growth *ladder* and a `reserve` flattens it; a `malloc`
# row is a FIRST allocation and a `reserve` only moves it — only an inline
# buffer removes that one. 81 % of this program's growths are first
# allocations, which is why `cg_growth.py`'s growths-per-call ranking points
# at the wrong rows on their own. `(-158)`.

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
# the whole dump ranked by CALL COUNT, with self Ir/call beside it — the
# standing rules' "rank the dump by call count and read the Ir/call column",
# which had no script for eight passes and is the device that found
# `Option::or_else`. Divides a whole-board walk's row into card visits, which
# is what says whether the row is body or iteration (pass 89's refutation).
python3 scripts/cg_calls.py cg.sym.out 45
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
# THE CENSUSES — what a memo or a gate is worth *before* a build is spent on
# it, and the only instrument that separates "the lane misses" from "the lane
# cannot help this workload". `CRAB_SBA_CENSUS` and `CRAB_PAY_FAILS` are
# runtime-gated; `trig-census` is a cargo feature because its tick sits in a
# per-dispatch preamble where even a never-taken runtime gate costs 0.03-0.04 %
# ((-115)). **A census that has to live inside a register-starved loop is a
# compile-time feature, not an env var.**
cargo build --profile release-fast -p crabomination --bin bot_ladder \
  --features trig-census
CRAB_TRIG_CENSUS=1 target/release-fast/bot_ladder --a gang --b gang \
  --games 6 --threads 1 --seed 1 --decks cube
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
# **The location column carries one directory component** (sixty-fourth pass).
# It was a bare basename, so `sync/mod.rs:1917` (the CoW deep copy) and
# `game/mod.rs:17108` (the trigger dispatcher) both read `mod.rs`, and every
# `macros.rs` in the standard library shared a label. That is what left this
# file describing `check_state_based_actions`' largest row as "a dependency's
# `macros.rs:332` at 0.62 %" for six passes: it is
# `core/src/slice/iter/macros.rs`, i.e. `slice::Iter::next` — the sweep's own
# battlefield walks.
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

**Hazards of this container, moved here from TODO's NEXT at the eighty-third
pass because they are measurement rules, not handoff.**

* ⚠ **Wall-clock rows do not cross hosts, and `host_calib_ms` is a
  *fingerprint*, not a correction factor.** Three rows this file carries are
  170-175 games/s at calib 51-57 on a 2.80 GHz box, 277-308 at 64-71 on a
  2.10 GHz one, and 239.9 at 53 on a 2.80 GHz one — a faster rate at a worse
  calib on a slower nominal clock, twice. If a games/s row looks wrong, build
  both sides in one sitting.
* ⚠ **`peak_rss_mib` is an allocator reading and therefore a distribution:
  take three before calling a difference.** A "13 % step" flagged at the
  eighty-second pass did not reproduce.
* ⚠ **A container reset wipes `target/`, removes `cargo-nextest`, and checks
  the repo out on the *system-prompt* branch.** Commit each measured change as
  soon as it measures, re-run the branch fetch after any surprising `git
  status`, and reinstall nextest with
  `curl -sSLf https://get.nexte.st/latest/linux | tar xzf - -C ~/.cargo/bin`.
* ⚠ **Disk.** A cold `release` + `profiling-fast` + `profiling-lines` +
  `overflow`-audit set of target dirs does not fit beside a debug build:
  `target/debug/incremental` is 7-15 GB and is the first thing to delete
  (`rm -rf target/debug/incremental`) — it costs the next debug build its
  incremental cache and nothing else.
  **The symptom names neither the disk nor the linker: a `release` link dies
  with a Bus error.** That directory reached 19 GB of a 30 GB allowance at the
  hundred-and-fifth pass and 14 GB at the hundred-and-sixth; delete it first
  and re-run before believing anything else about the failure.
* ⚠ **Do not rebase while a build is running.** Cargo fixes its unit graph and
  fingerprints when it starts, so a dependency rlib compiled *before* the
  rebase is linked *after* it — and the failure names the wrong thing. The
  hundred-and-ninth pass lost a 25-minute build to
  `assert!(GLOBAL_FEATS == 57)` failing against a source file that said 57:
  `crabomination_nn` had been compiled at the pre-rebase value. Rebase between
  builds, never across one.
* ⚠ **Killing a `cargo` does not kill its `rustc` children.** The orphan
  keeps its CPU and its output path — `ps -o pid,ppid -C rustc`, PPID 1 is the
  giveaway. One held 2.5 of four cores for 25 minutes after its parent was
  gone, which is why the next build "took 40 minutes"; it took 13. Reap them
  before starting the replacement, and note that two rustcs writing one
  artifact path is its own hazard.
* ⚠ **Two cargo builds at once take ~1.5x each on four cores**, and a cold
  `release` of this workspace is ~25 min on its own. Start the one whose
  result gates the next step first.
* ⚠ **…and on this container two cold builds at once do not finish at all:
  the OOM killer takes them** (eighty-ninth pass, `release` + `profiling-fast`
  started together, both dead with `signal: 9, SIGKILL` on
  `crabomination_catalog`). One `rustc` on that crate at `opt-level = 3` peaks
  at ~3.8 GB and the engine crate's at ~2.5 GB, so two of each does not fit in
  15 GB. **Build sequentially**; the "~1.5x each" above is what you get when
  they survive.

**MEASURING A CHANGE TO THE BOT: PIN THE JITTER, OR THE COLUMN IS GAME
LENGTH.** The scored pickers draw one `jitter_below(4)` per *candidate*, so
any change to how many candidates reach a picker re-aligns the tie-break
stream for the rest of the game and the two builds stop playing the same
games — with the policy untouched. `CRAB_NO_JITTER=1` pins every draw to 0
(one `OnceLock` read in `bot::jitter_below`); with it set, the decision count
is byte-comparable and says outright whether the games moved.

```text
CRAB_NO_JITTER=1 RUST_MIN_STACK=33554432 valgrind --tool=callgrind \
  --callgrind-out-file=cg.out target/profiling-fast/bot_ladder \
  --a gang --b gang --games 6 --threads 1 --seed 1 --decks cube
python3 scripts/cg_edges.py cg.out --callers next_action_settled   # the count
```

At the seventy-fifth pass the same commit read `cube` **+0.503 %** live and
**-1.618 %** pinned; the gap was 24,880 -> 25,012 decisions at a flat
-0.03 % apiece. It is a *measurement* switch — no shipped profile sets it,
and a strength number taken under it is not a strength number.

**AND WHEN THE QUESTION IS "IS IT STRONGER", NOT "IS IT THE SAME GAME":
`--vs` PLAYS TWO BINARIES AGAINST EACH OTHER.** `--a/--b` compares two
*profiles* inside one binary, so a code change that moves play could only
ever be argued for. Build the tip and the base, then:

```text
cargo build --profile release-fast -p crabomination --bin bot_ladder
cp target/release-fast/bot_ladder /tmp/base          # at the BASE commit
# ...apply the change, rebuild...
target/release-fast/bot_ladder --vs /tmp/base --a gang --b gang \
  --games 2000 --threads 3 --decks fixed --seed 11
```

Side A is the binary you invoke, side B is the one at `--vs`. Both must
carry the same `--a`/`--b` profile or the run measures the profile and the
code at once. **Run the null first** — `--vs` a byte-identical copy has to
read 50.0 % with every pair split (fixed/cube/sos, 400/160/100 pairs at the
tip); anything else is a bug in `crossplay.rs`, not a result.

**It gates a change to how the bot CHOOSES, and reports a change to how the
engine RESOLVES as a fault.** The two processes mirror one game and exchange
one `Option<GameAction>` per seat poll with a digest of the state it was
chosen in; a mismatch means the engines disagree, which voids the run rather
than one game of it, so it aborts with the seat, the poll and both digests
and exits 3. Verified against a copy with one rules constant moved: fault at
poll 50 of the first game.

Costs **1.9x wall** (800 games, 3 threads: 4.9 s -> 9.1 s) — the peer
replays every action, plus the round trips. **Its absolutes do not compare
to the in-process ladder's**: one process interleaves both seats' tie-break
draws on one jitter stream and two processes each draw their own. The
*estimate* does — `--a gang --b atk-sim`, 800 games, seed 11, in-process
51.1 % [49.9, 52.3] vs cross 51.2 % [50.1, 52.4].

**WHAT THE SIMULATION'S OWN PICKERS PROPOSE AND THE ENGINE REJECTS.**
`sim_step` quietly rolls a rejected declaration back and retries it as a
priority pass, so a picker that proposes an illegal attack or block is
invisible to the suite, the traces and `--bench`. `CRAB_SIM_REJECTS` counts
them; it is an early return on one `OnceLock` read when unset, so the hot
path pays a relaxed load and a branch.

```text
CRAB_SIM_REJECTS=1     target/release-fast/bot_ladder --a gang --b gang \
    --games 12 --threads 3 --seed 11 --decks cube      # prints the counts
CRAB_SIM_REJECTS=names …  --threads 1  2>&1 | grep sim_reject | sort | uniq -c
```

The census that closed (-54) and opened (-55) is in the candidates section.
**(-55) is closed and it stays the guard**: `CRAB_SIM_REJECTS=1` reads 0 in
all 69 configurations swept, so run it either side of anything touching a
picker or a combat check, and **sweep seeds rather than sampling three** —
`cube` deck content is seed-dependent and two passes called a half closed off
three seeds and were wrong both times.

**AND WHAT THE SIMULATION'S OWN PAYMENTS COST.** `CRAB_PAY_FAILS` splits
rolled-back payments by the `ManaError` they failed with, by what was being
paid for, and — the part that matters most — reports the **auto-tap work the
whole population does**, failing or not. Same device and the same off-cost as
above.

```text
CRAB_PAY_FAILS=1     target/release-fast/bot_ladder --a gang --b gang \
    --games 12 --threads 1 --seed 1 --decks cube       # the split + the cost
CRAB_PAY_FAILS=names …  2>&1 | grep '^pay_fail ' | sort | uniq -c   # + cost, site
CRAB_PAY_FAILS=names …  2>&1 | grep '^pay_kind ' | sort | uniq -c   # + what for
```

**generic** is the bot's `total` over-estimating — the (-51)(b) perf bug, and
the same asymmetry the CR 508.1g trim hit. **coloured / colorless / snow** is
the assignment problem, or auto-tap stranding a colour it could have covered,
which is a *correctness* bug: a payable line becomes invisible. **hybrid** is
neither half payable. `pay_kind` says *what was being paid for* — `fixed`'s
failures were 100 % `instant/sorcery` and were three response paths with no
affordability filter; `cube`'s are 1,352 `creature` / 508 `ability` / 478
`instant/sorcery` / 414 `other` in six games, a different question in a
different place. The tables are in (-51)(b).

**⚠ A FAILED PAYMENT IS NOT A UNIT OF COST, AND THIS CENSUS MISLED ITS OWN
AUTHOR BY IMPLYING IT WAS.** `auto_tap_for_cost_inner` returns before building
anything when the pool already covers the cost or the board has nothing to
tap; a probe that takes that exit is a `GameState` clone and little else.
Removing **700** such probes from `fixed` is worth **-0.282 %** (Ir, all four
filters); removing **64** is worth about a tenth of that, which is under the
clock's floor. **Not because the 64 were cheap** — `pay_fails_costly` says
98.7-100 % of failures on every pool had already built a source table — but
because 64 of anything is small. So `pay_taps` reports the
work, not the count:

```text
  pay_taps N auto-tap calls — M returned early (x %), T tables, S sources tapped

--games 12 --threads 1   calls   returned early   tables    taps
fixed  s1               11,298   3,802 (33.7 %)    7,496   19,478
cube   s1               21,142   5,518 (26.1 %)   15,624   41,698
all    s1               35,336  10,284 (29.1 %)   25,052   71,376
--bench (320 games)     64,678  21,276 (32.9 %)   43,402  113,204
```

Priced against (-51)(a) — **6,690 Ir a table, 7,665 Ir a tap**, both at the
seventy-fifth tip, read them off that entry rather than trusting a copy — the
bench workload spends ~290 M Ir on tables and ~868 M on taps. **1.75 sources
tapped per call, and taps outweigh tables three to one.** `--bench` also gives
the ratio that sizes the menu: **0.33 auto-tap calls per decision**, so the
cast sweep is not probing twenty candidates a tick.

**Do not split those taps by probe/committed — it was tried and it lies.**
The reading is 100 % of `fixed`'s and 99.3 % of `cube`'s inside a probe,
because `accept_on` is how the bot performs *every* action, the ones it adopts
as much as the ones it discards. The split that would matter,
evaluated-and-dropped against chosen-and-kept, is invisible from that call
site.

**And a green trace suite is not evidence that a bot change is
behaviour-preserving until you check the trace pool executes the code.** The
`fixed` pool reaches none of `cast_candidates`' specialty blocks —
`cast_candidates -> accept_on` is absent from its profile — so all 7 golden
traces and `--bench`'s `decisions` stayed byte-identical across a commit that
moved `cube` by 132 decisions.

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

**`peak_rss_mib` IS AN ALLOCATOR READING TOO, AND THE FILE HAS BEEN QUOTING
THE WRONG ONE.** Measured at the sixty-second tip on one box within the same
minute — same code, same `--bench` workload, `decisions 196,220` on all
three, so nothing but the build differs:

| build | features | allocator | peak_rss_mib |
|---|---|---|---|
| `profiling-fast` | `--no-default-features` | system | **17.6** |
| `release` | default | mimalloc | **24.0 - 24.3** |
| `overflow` | default | mimalloc | **27.2** |

**The shipped binary uses ~36 % more resident memory than the number this
file records.** The sixtieth pass's headline "peak RSS 21.9 -> 17.7 MiB,
-19 %" sits in a block whose Ir readings are stated as `profiling-fast
--no-default-features`, and the RSS came off the same binaries — so the -19 %
is real and reproducible (17.6 here, two passes later) but it is a fact
about the **system allocator**, and `selfplay_train` actors do not run that
build. **Plan actor counts off ~24 MiB, not 17.7.**

Neither figure is comparable across containers either: the concurrent
session's `release` block reads 30.0-30.1 MiB at `host_cpu` 2.80 GHz where
this box reads 24.0-24.3 at 2.10 GHz. **So an RSS row needs its profile, its
feature flags and its host before it means anything** — the same discipline
this file already applies to `games_per_s`, and for the same reason.

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

**⚠ `ab_wall` CANNOT RESOLVE A SUB-1 % CHANGE, AND ITS CONFIDENCE INTERVAL
WILL NOT TELL YOU SO.** Measured at the eighty-fourth pass, twice, against
callgrind: a scan removal worth **0.045 %** by Ir read **-1.31 % [-2.34,
-0.27]** and **-1.46 % [-2.68, -0.23]** over three sittings with **22 of 26
blocks faster**. The four response-path filters, worth **-0.282 %** by Ir,
read **-2.20 % [-3.63, -0.78]** with 6 of 6 blocks faster.

**The mechanism is code layout and the ABBA schedule does not cancel it.**
Moving one function's body shifts every function after it, and I-cache and
branch-predictor alignment shift with it. That bias is a *fixed property of
the binary pair*: every block sees the same amount of it, so the block-to-block
variance the CI prices says nothing about it. A repeatable, direction-correct,
interval-clearing result is exactly what layout bias looks like.

**And the null control does not catch it** — the script's footer says to run
one, and it is still worth running, but a null puts the *same binary* on both
sides, so its layout difference is zero by construction. **The null validates
the box; nothing in this harness validates the pair.**

**This does not contradict the seventy-third pass's Ir:wall ratio of 2.15x**
(the probe-removal entry below, `-1.29 %` wall against `-2.775 %` Ir with a
same-hour flat null). That change was ~2.8 % of Ir — *above* the layout floor,
where the ratio means something. Both of the numbers corrected here were under
0.3 % of Ir, where it means nothing. That entry's own closing line already
said it: **"anything under ~2 % of Ir will not show on the clock here at
all."** What the eighty-fourth pass adds is that it does not fail to show —
it shows as a significant result of the wrong size.

**So: callgrind Ir for anything under ~2 %.** It is deterministic and
layout-blind, one run per binary settles it, and the recipe is at the top of
this section. `ab_wall` is for changes big enough to clear layout bias, and
for changes Ir cannot see.

**And Ir has the mirror-image blind spot: it cannot price memory.** An
allocator call and a cache miss are one `call` and one `mov` to callgrind. The
scan removal, which allocates nothing, shows a 25x Ir-to-wall gap — pure
layout. The response-path filters, which remove ~700 `GameState` *clones* a
run, show 8x, and some of that is real work Ir undercounts. **Quote both
numbers whenever a change adds or removes allocations, and let neither stand
alone.**

**When you do want a clock number, use `scripts/ab_wall.py` and run its null
control — and read the null's own resolution, because it is the box's, not the
harness's.** The +/-2 % below is the 2.10 GHz container's; the 2.80 GHz one
(`host_calib_ms` 50-57) resolved **+/-0.99 %** at the same eight blocks and
workload at the sixty-fourth pass, which is what made (-48)'s -5.99 %
quotable.
 It is the loop every pass has hand-rolled — alternate two binaries,
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

**A LOCALITY ARGUMENT CANNOT BE MEASURED BY Ir AT ALL, AND THAT IS SHARPER
THAN "Ir undercounts".** `same_team` sits at **143,564 calls x 26.5 Ir** in the
call-count ranking — the sort of row this file's own device says to take. It
reaches `team_of` twice, and each of those indexes `Team::members`, a heap
`Vec` one allocation away from the `Team` it hangs off; the exactly-equivalent
`teams.iter().all(|t| t.members.len() == 1)` reads only the inline `len`.
Built and measured at the ninety-first pass: **`fixed` -0.009 % / `cube`
-0.007 %** — 92,583 Ir of an expected 2.3 M. Reverted.

**The change removed two dependent *loads*, not two instructions, and
callgrind charges 1 Ir for a load whether it hits L1 or misses to DRAM.** So
the whole predicted win was invisible to the instrument by construction. The
rule: **before proposing a change, say whether it removes instructions or
removes stalls.** Only the first is measurable here; the second needs the
clock, and this box's paired clock resolves +/-0.34 % on `--decks fixed`,
which will not see a 0.2 % locality win either. A row that is big in
*call count* and small in *Ir/call* is usually the second kind — `same_team`
at 26.5 Ir/call is already only ~26 instructions.

**`-C target-cpu=native` IS FLAT, AND IT IS THE SAME LESSON FROM THE BUILD
SIDE.** Never tried before the ninety-first pass — `pgo`, `target-cpu` and
`native` appear nowhere in this file, `Cargo.toml` or `.cargo/config.toml` —
and the box is a Xeon with `avx512f`, `avx512vl`, `bmi2` and `fma` against a
default target of baseline x86-64, so the whole ISA gap was unmeasured.

```text
release-fast, RUSTFLAGS="-C target-cpu=native", CARGO_INCREMENTAL=0 (8m 54s).
Both binaries read the committed invariant: 195,528 decisions / 27.44 turns /
0 stalls, so they play the same games.
ab_wall.py, 8 blocks, --games 2000 --decks fixed --threads 4 --seed 11:
  mean B/A  +0.10 %   median +0.19 %   3/8 blocks faster
  95 % CI   -0.55 % .. +0.75 %          verdict FLAT
```

**And it is flat for a reason that predicts the next lever.** The widened ISA
is vector width and a handful of scalar encodings; this engine walks `Arc`
graphs and branches on enums, and has nothing to vectorize. The instrument
this file reaches for cannot see the difference either — a wider register is
*fewer* instructions, so Ir would have flattered it — which is why this was
put on the clock directly. **What is left on the build side is layout, not
width**: branch ordering, block placement and icache, i.e. PGO, and that is
the half a branchy pointer-chaser can actually use.

**AND PGO IS `-23.8 %` ON `fixed` AND `-23.4 %` ON `cube`, WHICH IS LARGER
THAN EVERY CODE CHANGE IN THIS FILE PUT TOGETHER.** `scripts/pgo_build.sh`
at the ninety-first pass: instrument, play 470 games across the three pools,
merge, rebuild. Ninety passes of this file went to micro-architecture in the
source and the build's own layout was never once tried.

```text
release-fast, base 15734b6f. Both sides built from the same tree; both read
the committed invariant (195,528 decisions / 27.44 turns / 0 stalls /
determinism ok), so they play the same games.

  null  base2 vs base2  fixed  -1.42 %  6/8   CI  -2.49 .. -0.35 %
  A/B   base2 vs pgo2   fixed -23.79 %  8/8   CI -24.42 .. -23.16 %
  A/B   base2 vs pgo2   cube  -23.37 %  6/6   CI -24.23 .. -22.52 %
  and an independent build pair one commit earlier, fixed -24.13 %, 8/8

  binary  142,125,472 -> 118,117,656 bytes  (-16.9 %, both pairs)
```

**Read the null against the verdict, because the null is NOT clean here** —
−1.42 % with a CI that excludes zero, i.e. the box drifted during the run and
this workload's resolution today is about ±2.5 %, not the ±0.34 % the
eighty-eighth pass got. That is an order of magnitude under the effect, so it
does not touch this verdict; it would have swallowed any of the last twenty
commits whole. **A null that comes back significant does not invalidate a
verdict — it sets the size of verdict the run can carry.**

**The 16.9 % smaller binary is the mechanism, not a side effect.** LLVM knows
which blocks are cold and stops inlining and unrolling into them; a
142 MB binary walking `Arc` graphs is an icache-miss machine, and the pool
split confirms it — `fixed` and `cube` move *together* despite having almost
disjoint hot sets (`fixed` carries no statics at all). A layout win is
uniform across pools in a way no algorithmic change in this file has been.

**AND THE PROFILE THIS FILE MEASURES ON IS 8.3 % SLOWER THAN THE ONE THE
PROJECT ALREADY HAS, WHICH NOBODY HAD EVER PRICED.** `release-fast` was
chosen for build speed — `Cargo.toml` says 2m04s against 10m02s, a 4.9x
rebuild — and the *throughput* half of that trade was never measured. Four
binaries, one tree, all four reading the committed invariant:

```text
  ab_wall.py 8 blocks, --games 2000 --decks fixed --threads 4 --seed 11

  A                    B                      verdict        blocks  bytes(B)
  release-fast         release (LTO, cgu 1)   -8.28 %         8/8   123,768,408
  release-fast         release-fast + PGO    -23.79 %         8/8   118,112,504
  release (LTO)        release-fast + PGO    -16.15 %         8/8   118,112,504
  release (LTO)        release + PGO          FLAT (-0.33 %)  5/8   119,306,264
                                              CI -0.57..+0.29
  release-fast (plain)                                             142,161,400
```

**AND THE FLAT ROW WAS AN ARTIFACT — THE PROFILE HAD TO BE RAISED UNDER THE
PROFILE IT IS CONSUMED UNDER. THAT IS THE ENTRY'S MOST REUSABLE LESSON.** The
`release + PGO` build above reused a profile generated from a `release-fast`
**instrumented** binary. Redo it with a profile raised under `release`'s own
settings and the flat row becomes the largest win in the table:

```text
  release (LTO)         release + PGO, matched profile   -20.75 %  8/8
                                          CI -21.02 .. -20.48 %
  release-fast + PGO    release + PGO, matched profile    -5.03 %  8/8
                                          CI  -5.26 ..  -4.80 %

  binary, same tree:  release-fast          142,161,400
                      release (LTO)         123,768,408
                      release + PGO (reused profile) 119,306,264
                      release-fast + PGO    118,112,504
                      release + PGO (matched)        107,551,920
```

**The size column is the tell, and it is how to catch this without an A/B.**
A mismatched profile is not rejected — it is *partially* applied: 3.6 % off
the binary and no time, against 13.1 % off and -20.75 % for the matched one.
Nothing warns. **If a PGO build's binary did not shrink by roughly what the
matched case shrinks, the profile did not take**, and `-Cllvm-args=
-pgo-warn-missing-function` will not tell you either, because a matched build
warns about `std` and the dependency graph anyway.

`scripts/pgo_build.sh` is immune by construction — it instruments and rebuilds
under one `PGO_PROFILE` — which is why the hand-run shortcut, not the script,
is what produced the artifact.

**So PGO and LTO DO stack, and the whole ladder is worth 27.6 %.** Against the
plain `release-fast` this file measures on: `release` (LTO) 0.917,
`release-fast + PGO` 0.762, `release + PGO` **0.724**. The arithmetic closes —
-16.15 % and -20.75 % from a common baseline predict -5.5 % between the two
PGO builds, and the direct pair reads -5.03 %.

**Which one to build is a build-time question, not a throughput one.**
`release-fast + PGO` is ~18 min for -23.8 %; `release + PGO` is ~44 min (two
LTO builds) for -27.6 %. The last 5 % costs 26 minutes of wall clock per
build, so it belongs to a long training run, not to an iteration loop.

**Three cautions before anyone leans on it.** (a) It is **opt-in and stays
opt-in**: no profile in `Cargo.toml` or `.cargo/config.toml` turns it on, so
every committed number in this file remains a plain `release-fast` number and
stays comparable. Quote a PGO reading only against another PGO reading. (b)
**Ir cannot see this at all** — same lesson as the locality entry above, from
the other end: PGO removes stalls and moves code, and callgrind charges the
same instruction wherever it sits. Do not try to attribute it with
`cg_edges.py`. (c) The training workload is three pools on **seed 7**,
deliberately not the seed anything is measured on; a profile fitted to the
sequence under test flatters itself. The measurement was on seed 11.

**THE ML ACTOR WAS FILED HERE AS "SHOULD CARRY — AN INFERENCE", AND
MEASURING IT SAID BOTH `-23.1 %` AND `-4.9 %` DEPENDING ON THE WORKLOAD.
THAT SPLIT IS WORTH MORE THAN THE PGO NUMBER.** Same two `selfplay_train`
binaries, same box, `--actors 4`, one hour apart:

```text
  --games 3000 --steps 200   -4.93 %  6/6  CI -8.28 .. -1.59 %  (null +/-1.09)
  --games 6000 --steps 1    -23.14 %  6/6  CI -23.88 .. -22.39 % (null +/-1.29)
  binary 147,327,672 -> 121,662,320 bytes (-17.4 %)
```

**Because the first workload is not measuring the simulator.** Its own
`stats.jsonl` says `t_step_ms` 31,437 of a 33 s run — **the learner thread is
busy ~95 % of the wall clock**, doing batch-256 gradient steps in candle,
which is already-tuned numeric kernels PGO has nothing to give. Drop the
learner to one step and the four actors own the clock: **90.9 games/s ->
261.0 games/s** on the base binary, and the PGO win goes straight back to the
`bot_ladder` figure.

**AND THE NUMBER THAT MISLED HERE WAS `selfplay_train`'s OWN, SO IT IS FIXED
RATHER THAN WRITTEN UP AS A CAUTION.** `games_per_s` was games ÷ *elapsed*,
and a `--steps`-bounded run outlives its actors — so the denominator belongs
to the learner. The proof is two runs of one binary doing provably identical
work:

```text
  --games 3000 --steps 1     3000 games, 287,852 rows, 12.6 s -> 242.6 /s
  --games 3000 --steps 200   3000 games, 287,957 rows, 32.8 s ->  92.6 /s
```

The run now records when the last game finished and prints the actor window
beside the run's, so the two are never confused again:

```text
  done:   3000 games (92.2/s), 288422 rows, 0 stalls, 33s
  actors: 232.0 games/s over 12.9s (27% of the run; the rest is the
          learner outliving them)
```

`actor_s` and `actor_games_per_s` join `stats.jsonl` for the series.
**Quote `actors:`, never `done:`, for anything about the simulator** — a
simulator optimization measured on the old line was divided by up to three
before you saw it, which is also why `--bench` and `bot_ladder` are this
file's instruments and `selfplay_train` was not.

**And it re-prices the ML loop itself.** At a learner-heavy setting the
training run is not simulator-bound at all, so *no* amount of engine work
moves it; the lever there is the learner (batch size, step cadence, device).
Nothing in this file has ever said which side a real training configuration
sits on. That is now a question with a one-line instrument.

Two notes for whoever repeats it. The run's `load average` line warned; it
was a **false positive** — `start 2.49` is the decaying one-minute average
from the `--bench` fingerprint run immediately before, and the per-block
`3.88-4.03` is exactly this run's own four threads. Read the per-block range
against `--threads`, not the start value. And if it is ever taken, it must be
**opt-in** (a documented profile or flag, never the default): a
`target-cpu=native` baseline is not comparable across boxes, and this file's
committed numbers are.

**`ab_wall.py` prints a load line and warns on a contended box** (eighty-ninth
pass). A verdict taken while the other session was linking is not a verdict —
the same null reads `+/-1.91 %` quiet and `+/-9.73 %` under six spinners, and
returns FLAT both times. Read the load line before the verdict.

**But "sub-5 %" is about `--bench`, not about the clock.** A *paired* ABBA
run (`scripts/ab_wall.py`, 8 blocks) resolved **+/-0.34 %** on `--decks fixed
--games 2000` at the eighty-eighth pass, and measured a pass worth -4.33 % in
Ir at **-1.59 %** with 8/8 blocks and a null that came back flat. So an
accumulated pass *can* be put on the clock; a single commit under a couple of
percent still cannot. Run the null on the workload you are about to quote —
the resolution is the workload's, not the box's (`sos` reads +/-2 %).

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

## Standing rules for a perf pass

- **Price a hot row by WHO CALLS IT, not by its body.** Six functions had
  been read by line across three passes and all said "no hot line";
  `(-194)`..`(-196)` came off one `--separate-callers=3` dump and
  `cg_contexts.py`, which ranked `computed_permanent_hinted`'s 366 k
  calls by context and found the top two were a caller asking for a view
  it held. A memo-hit path's cost is the asks that reach it; a walk's
  cost is the entries that enter it. `cg_lines.py` is for the row whose
  callers are all legitimate.
- ⚠ **AN A/B'S TWO SIDES ARE BUILT FROM ONE BASE TREE.** A rebase between
  the base build and the candidate build can pull a concurrent *catalog*
  commit, and a card rewrite moves the six-game `cube` run — `(-185)`'s
  first reading was `cube` +2.62 % / `sealed` +0.52 % / `fixed` -0.59 %
  against a base that predated a Moonshadow fix, and four builds went into
  proving the candidate innocent. Read `git log --oneline --stat
  <base>..HEAD -- crabomination_catalog crabomination_base` after any
  rebase; a non-empty answer means re-take the base. The tell in the dump
  is a *different game*, not a different cost: `pay_taps` under
  `CRAB_PAY_FAILS=1` moves, and `CRAB_DUMP_TRACES` names the pairing.
- **A one-caller wrapper is inlined by luck; adding a caller to it is a
  codegen change to the first caller** (`(-182)`, two builds). Check
  `cg_edges.py --callers <wrapper>` on the base: 0 calls means "do not add
  one" — read the wrapper's body through a closure of your own instead.

**Memo / lane / gate rules, moved verbatim from `TODO.md`'s NEXT when that
section passed its ~15-line budget again. They come off `(-149)` through
`(-155)`.**

- ⚠ **PRICE AN INLINE BUFFER BY WHETHER IT GROWS ITS OWNER, NOT BY THE
  ALLOCATION OR THE SPILL IT REMOVES** (`(-165)`, four builds, all reverted;
  `(-161)` is the same rule seen from the winning side). It is **free** when the
  inline form is the same size as the `Vec` it replaces — `(-161)`'s
  `CopyVec<[CardId; 4]>` at 24 bytes, `fixed` -0.521 % — and free when the owner
  is a frame nothing copies (`(-158)`'s twelve locals, `(-164)`). It **loses**
  on an owner that is **returned** (`DispatchScan` 56 -> 216 bytes: `__memcpy`
  +9.5 M against 5.6 M of allocator recovered, and **+0.99 % on `fixed`**, where
  both its lists are always empty) and on one that is **cloned**
  (`LayerFreezeState::perms` 8 -> 24 slots is +256 bytes of `GameState`:
  `__memcpy` +1.74 M, of which `GameState::clone` is +1.06 M and the bot's
  dry-run probes the rest). **A byte added to `GameState` costs ~6,800 Ir a
  six-game `cube` run** — `PlayerData`'s size-class rule one level out. And a
  spill costs ~208 Ir, about what the width to remove it costs, so **the
  `reserve_one_unchecked` census is an instrument, never a queue**.
- ⚠ **`PlayerData`'s ceiling is 1,016 bytes**: 1,032 is a 1,040-byte chunk
  past glibc's largest smallbin, and crossing it cost 11.3 M Ir — more than
  the 10.0 M device that grew it saved. **Price a field added to the hot
  player group at the size class, not at the copy.** 24 bytes of headroom
  left (992 of 1,016).
- **Price a memo by reads-per-invalidation, not by the size of the walk it
  replaces.** `(-153)` took half its filed ceiling because `Battlefield`'s
  lanes clear on *membership* changes, which happen several times a turn
  against ~20,012 sweeps, so a once-per-sweep question misses ~3 times in 4.
  `(-152)`'s piles won because a library is written a few times a turn and
  read every sweep.
- **The `zone::` lane device covers INSTANCE state** (`(-152)` is the first),
  which widens what a zone memo can answer — but only on a zone whose every
  `&mut` route clears the word. `Battlefield::iter_mut`/`get_mut` deliberately
  do **not**, so a predicate put on a *battlefield* lane must still be
  definition-only, widening its instance legs in the sound direction
  (`(-153)` widened two).
- ⚠ **A line-profile row is a pointer, never a size.** `cg_lines.py` read one
  source line at 84 Ir a walk on `cube` and 3.4 on `fixed` where the A/B says
  both pools pay the same share. Take the A/B; the counts are the truth.
- ⚠ **Read a six-game number for a new `debug_flag` as an understatement**:
  the flag `format!`s a definition **233 times a process**, once per distinct
  card name, so its cost is fixed and the gate's win is not.
- **When a cost field duplicates what an effect does, check it takes the same
  replacement path.** `add_counter_cost` placed its counter raw instead of
  through `scaled_counter_count`, so Vizier of Remedies shaved the counter an
  *effect* placed and not the one a *cost* placed (CR 614.16).
- **Rank a chain of pure guards by cost x rejection rate** (`(-116)`, and
  `(-155)` is its fourth application): put the field test that rejects on the
  common board in front of the pointer chase, not after it.
- ⚠ **PRICE A STD-ADAPTER REWRITE AT ~10 % OF THE ADAPTER'S SELF Ir, NOT AT
  ITS SELF Ir** (`(-156)`, measured). Deleting `Filter`/`FilterMap` from
  `pick_blocks_inner` removed the adapter's entire 4,375,134-Ir row and made
  the program 475,735 Ir cheaper — **89 % of the row moved into the `for`
  loop**, because an adapter's self cost is mostly the iteration it drives
  (bounds, advance, the `find` loop), which a hand loop drives too. Only the
  call frame is removable. `cg_calls.py`'s "a std generic the inliner declined
  is real" means the row is not an artifact; it does **not** mean the row is
  an opportunity.

**The recycle-list rules, from `(-166)`/`(-167)`/`(-168)` — the device is now
three entries deep and each one produced a rule that is not about this pool.**

- ⚠ **A "is this handle still shared" question is answered by WHEN it is
  asked, not by what it is asked about.** `(-27)` asked `Arc::get_mut` at the
  *release* site (scope exit, the caller still in frame) and hit 1 in 8;
  `(-166)` asks the same question at the *reuse* site and hits 68-82 %. Move
  the check, not the data structure.
- ⚠ **When a pool's hit rate is short, ask "choosing badly or running out?"
  BEFORE building a policy change.** A reordering answers only the first, and
  `(-167)`'s FIFO moved the allocation count by *exactly zero* on two pools —
  which is the proof that it was the second. **Size a recycle list by the tail
  of its per-scope demand, never by the mean**: 4.3 and 5.8 permanents a scope
  sized a list at 8, and the misses were all scopes asking for twenty.
- **Pool the OWNER, not the buffer, when the buffer's owner is already an
  `Arc` somebody parks** (`(-168)`). An `Arc<Vec<T>>` is two allocations and a
  parked one comes back sized, so one hit is worth 1.3-1.7 allocations —
  `(-162)`'s free-list-with-RAII-guard design for the inner `Vec` is subsumed
  by recycling the box that holds it, with no guard and no escape hatch.
  `std::mem::take` on the parked slot is what keeps the box in the `Arc`;
  unwrap-and-rewrap retakes the allocation and wins nothing.
- **Thread-local, not per-object, whenever the object is cloned more often
  than it is used.** `Clone for LayerFreeze` is `default()` and a `cube` run
  takes 22,684 `GameState` clones against 28,992 freeze scopes.
- ⚠ `try_borrow_mut`, never `borrow_mut`, and **release the borrow before
  calling back into the engine**: the gather re-enters itself, and a borrow
  held across it turns every inner call into a miss (or, with `borrow_mut`, a
  panic reachable from self-play).

**Census / catalog-audit rules, moved verbatim from `TODO.md`'s NEXT at the
hundred-and-sixth pass when that section passed its ~15-line budget. They come
off the targeting lane (`13435f3e` / `d9e6454d` / `d0799d5c` / `45c55cc3`).**

- **Discover a class by joining the census against
  `scripts/.scryfall_cache.json`, then gate it on a STRUCTURAL predicate** —
  all 38 blink bodies name `ControlledByYou` / `OwnedByYou` /
  `ExiledWithSource`, which is what makes the test an invariant instead of a
  list of 79 names that goes stale on the next card.
- **An implicit filter belongs to the FIELD, not the card.**
  `IMPLICIT_CREATURE_TARGET` (pump), `IMPLICIT_ANY_TARGET` (damage, CR 115.4)
  and `implicit_player_if_bare_player_field` are one line each; the per-card
  filter is only for nouns narrower than the field's own type (twenty of
  those, `45c55cc3`).
- **When a catalog fix declares a filter on a slot, check the slot walker has
  an arm for that effect** — `every_declared_target_slot_is_answerable` caught
  `CoinFlipDestroyLoop` and `MoveChosenKeyword` mid-pass, where the fix would
  have aimed correctly and re-checked against nothing.
- **Group a census by the nearest enclosing enum key**, not by card: 204 card
  rows were ~40 match arms.
- **Two groups in that census are structural false positives — checked, so
  nobody re-checks them:** the counterspells (`CounterSpell { what: Target(0) }`
  targets a *spell*, which `Target` cannot express) and the ~25 reflexive
  triggers ("whenever this deals combat damage to a creature, tap that
  creature"), whose slot `combat.rs:5234` **stamps from the event**.
- **`audit_oracle_verbs.py`'s false positives are the same shape: a verb done
  by a BESPOKE effect has no primitive in the tree and the filter cannot see
  it.** Breath of Fury is the checked example — its "untap all creatures you
  control" is inside `Effect::SacrificeEnchantedForExtraCombat`'s resolver, so
  the audit reports a missing `Untap` that is neither missing nor wrong.
  Read the resolver, not only the effect tree, before filing a row. Of the
  nine rows in the audit's three smallest classes, seven were real (four
  fixed at the hundred-and-sixth pass), one is this, and one — All-Out
  Assault's "when you next attack this turn, untap each creature you control"
  — needs a `DelayedKind` that does not exist yet, so it is a primitive job
  and not a catalog one.
- **`dispatch_triggers_for_events`' self cost is mostly the per-event
  bookkeeping switch** — entry timestamps, soulbond, land equilibrium, the
  per-turn tallies — **not the listener search.** A listener index keyed on
  `EventKind` cannot reach the row's whole share, which is what `(-90)`'s
  "mask ceiling 0.86 %" was already saying; read the function before pricing
  an index for it. `(-115)` carries the line profile.

Durable, not per-run. Every refutation named here is written up in **PERF**'s
Log with its numbers; read the entry before re-proposing any of them.

**Moved here from `TODO.md` at the eighty-seventh pass, verbatim.** It was
555 of that file's 1,030 lines and it is the perf record, not the handoff.
The instruction it carried — collapse each rule to its claim plus the pass
that measured it — is **not** what happened, deliberately: the detail is the
refutation, and a rule refuted on a *mechanism* stays refuted. Compact it
here if it ever needs compacting.

- **A concurrent push invalidates a MEASUREMENT, not a candidate** (pass 99).
  A commit landed under this pass's A/B mid-run; the whole thing was retaken
  against the new base and the numbers moved by a quarter while the conclusion
  did not. Rebase, rebuild both sides, re-read — and **re-read a candidate
  against the tip it will land on**, because two changes to the same walk
  multiply on its count and add on its per-call cost.
- **Check a hand-written walk against another walk of the same tree, not
  against a guess at what the tree means** (pass 99/100, the target-walker
  invariants). The slot-agreement test holds at 7,728 bodies; a blanket
  "holds a `Move`" version of the same idea produced 29 findings and every one
  was correct as it stood. **And make such a test assert its own population**,
  or it goes quietly vacuous — the failure an empty ratchet hides.
- **A pool can only recycle a handle whose lifetime the pool's owner bounds**
  (pass 99, `(-27)`'s single-slot variant, built and reverted; `fixed`
  **+0.137 %**). `computed_permanent` pushes one `Arc` into the scope's memo
  and returns the other to a caller that collects it *out* of the scope, so
  `Arc::get_mut` fails at seven of every eight scope exits and 316,576
  bookkeeping calls recovered 19,086 allocations. Check the lifetime before
  counting the allocations; the arithmetic was right and the premise was not.
- **A branch added to a function that is inlined everywhere is not one
  branch — it is the inlining decision, retaken** (same commit).
  `end_of_scope` was free, a `None` store and a `clear()` folded into
  `Unfreeze::drop`; one `if` made it a named 43.9-Ir row over 150,732 calls.
  Same shape as pass 98's `#[cold]` refutation from the other direction.
- **A second reader of a lazily-filled memo makes the first one cheaper, so do
  not attribute a memo's win to the caller that reads it** (pass 99, the
  combat dispatch's listener bits). `dispatch_board_scan` is not in that diff
  and its call count does not move, yet it fell 5 % per call on every pool
  because the new caller reaches most permanents first and pays the miss path
  it used to inline. The two row deltas together were the whole program delta.
- **Before hoisting a per-item board walk out of a loop, divide the loop's
  total item count by its call count** (pass 93's second concurrent half,
  `(-84)(b)`, built twice and reverted; `fixed` **+0.090 %**). A "once per
  pass" scan is only cheap if a pass has many items — `legal_blockers` and
  `pick_blocks_inner` run over **one to three** candidate blockers while
  being called thousands of times, so an *unconditional* board scan hoisted
  out of them runs about as often as the *conditional* per-blocker walk it
  replaced, at roughly twice the cost. The deletion ceiling for that walk is
  a real -0.333 % of `cube`; a hoist is simply not the way to it.
- **When a hoist fails because the loop is short, ask whether the *scope* is
  long** (pass 94, the same `(-84)(b)`, and it took 54 % of the ceiling the
  rule above declared unreachable). A `PresenceGate` slot memoizes a board
  question for the lifetime of a freeze scope, and a scope spans the whole
  bot tick — thousands of the calls a per-pass hoist could only ever amortise
  one to three at a time. The test for the slot is one question: **can a
  freeze scope change the answer?** `false` for every printed static on the
  battlefield. A slot costs ~113 k Ir a `cube` run in `clear_gates` and
  nothing anywhere else; `(-85)` tried to remove even that and is a
  refutation.
- **A whole-board presence memo pays on two conditions, not one: the caller
  must not already gate the walk, AND the walk's per-card body must be more
  than a length check** (pass 98, the cast-lock mask, built and reverted;
  `fixed` **+0.034 %**). Three lanes shipped in the same pass on 460 / 360 Ir
  walks; this one replaced a 56 Ir walk over `def.static_abilities` and the
  out-of-line miss path alone cost more than the whole walk did. It is the
  rule below seen from the collection level rather than the card level.
- **A per-definition presence bit pays only when the walk it replaces is over
  a list that is usually non-empty** (pass 94, `(-87)`, built and reverted;
  `cube` **+0.138 %**). `def.static_abilities.iter().any(..)` on a board of
  ordinary permanents is a pointer load, a length load and a not-taken
  branch — the `CardMemo` read that replaces it is a word load, a valid test
  and a mask against the *same* `CardData` deref, i.e. the same instructions
  plus a recompute. The memo was hitting **98.3 %** of the time and it still
  lost. `sba_scan_bits` (five inner loops) and `dispatch_scan_bits` (every
  static plus every Station band) won because their walks are real; count the
  elements the walk actually touches before adding a bit for it.
- **A presence gate is read ~3.5x more often than a scope exits** (pass 94,
  `(-85)`, built and reverted; `cube` **+0.011 %**). ~400 k gate reads a
  `cube` run against ~113 k outermost pops, so **any per-read cost added to
  make the per-exit clear cheaper loses**, and it loses on the pool that asks
  the most gates while winning on the two that ask fewest — the pool split
  is the signature of this trade, not an accident of one encoding.
- **Price a `find` site at its expected stopping point, not at the
  collection's length** (pass 92's concurrent third, and it cost a build).
  A `battlefield.iter().find(|c| c.id == id)` for a permanent that *is* there
  stops at the match: half the board at worst, and much less for the lands an
  auto-tap walks in board order. Two such sites were sized at ~120 Ir apiece
  from "23 `Arc`-boxed permanents at ~5 Ir each" and measured ~58.
  `(-38)`'s "read a `cg_sites` number as a **floor**" is about a *sampled
  instrument* and does not transfer to hand arithmetic over a `find`.
- **A gate is worth the population it covers, so check each conjunct's hit
  rate before writing it down — and then check what widening it costs on the
  pool that gains nothing** (same pass, and the second half is why the change
  was reverted). `granted_abilities_of`'s gate tests `me.counters.is_empty()`
  for the Cauldron leg and 35 % of the permanents the mana sweeps visit carry
  a counter; replacing it with a board-level bit recovers that third and reads
  `sealed` **-0.129 %** / `cube` **-0.043 %** / `fixed` **+0.005 %**. The
  `fixed` sign is the extra conjunct and the wider `GrantScan` — built per
  sweep — not the bit's walk, which never runs on a pool whose archetypes
  carry no `static_abilities`. **A conjunct swapped into a gate is a struct
  field somewhere, and the pool that does not gain still pays for it.**
- **A rebase shrinks a patch without shrinking its measurement** (same pass;
  the cauldron entry states the rule from the worktree side and it is the same
  failure). A whole-gate A/B taken at a base that predated a concurrent
  session's equivalent gate became a one-conjunct diff on rebase and kept the
  whole-gate number. Re-read the moved row at the **tip** before filing, and
  treat a changed *row name* — a split, an inline, a rename — as a base
  mismatch rather than a rounding difference.
- **An iterator adapter chain is a *per-element* branch, and on a whole-board
  walk it can be more than the loop body** (pass 87, the concurrent half,
  (-78); `fixed` -0.839 % / `cube` -0.552 % for one site).
  `all_static_sources()`'s `Chain` + `FlatMap` + `Filter` cost ~20 Ir a
  permanent in a loop whose body does nothing on `--decks fixed`. **Price it
  by deleting the rare leg** — one build, and it read -0.675 % before the real
  fix existed. Three tests: is the collection walked per element, is the
  chained leg usually empty, and does the body have a `continue` that has to
  become a `return` when it moves into a closure. **Test 1 is the whole
  rule**: every candidate of this shape that failed, failed on it, and
  `activate_ability_inner`'s nine whole-board static walks *deleted outright*
  are only `fixed` -0.121 % / `cube` -0.167 % in total.
- **`Vec::clone` hands back `capacity == len`, so every `Vec` inside a
  copy-on-write structure reallocates on its first push after the copy**
  (pass 86, the concurrent half, (-76); `cube` -0.44 % over six sites). The
  `CowBox<Vec<T>>` half is closed centrally by an inherent `push` that
  materializes at `len + 1`; the plain-field half is per field. **Ask what
  else copies a `Vec` and then writes to the copy.**
- **The byte test is necessary and not sufficient — run the read-count test
  on the field's *consumer*** (pass 87; `affected_from_requirement`, both
  halves refuted). `AffectedPermanents::Specific` at `[CardId; 4]` is the
  same 24 bytes the `Vec` was and removes 30,534 allocations, and
  `affected_includes_gated` — the whole-board matcher that reads it once per
  (effect x permanent) pair — pays **+9.7 % of its own row** for them,
  because on a grant-heavy board the list spills anyway. `fixed` -0.212 % /
  `cube` +0.158 % / `sealed` +0.174 %: a pool split, and the field's own
  function is not where its reads are.
- **Size an inline buffer before rejecting it: `SmallVec<[T; N]>` is
  `8 + max(N * size_of::<T>(), 16)` bytes, so below 16 bytes of payload it is
  exactly the 24 bytes the `Vec` was** (same entry). The same three-field
  change reads `fixed` **+0.137 %** at `[_; 4]/[_; 8]/[_; 4]` and **-0.463 %**
  at `[_; 1]/[_; 4]/[_; 1]` **on an identical allocation saving** — the tax is
  the struct's bytes, priced at ~0.0009 % of `cube` a byte by (-74)'s padding
  probe. This **narrows (-72)**, which is the *read* count on `.players`
  (35,000 sites) and not a refutation of struct fields as such. Two tests, not
  one: read count first, byte count second.
- **A *returned* buffer is not disqualified from inline storage — the
  disqualifier is bytes moved** (same entry; `cube` -0.211 %).
  `statics_granted_triggers_inner` returns a `Vec` of *references*, so
  `SmallVec<[&T; 2]>` moves the same 24 bytes it always moved and the 0-2 case
  stops allocating. Read (-71)'s warning as arithmetic, not as a rule about
  ownership.
- **A definition-derived answer can be memoized on the object, and
  `CardInstance::DerefMut` is the invalidation** (pass 87; `fixed`
  -0.862 %, `cube` -0.700 %, `sealed` -0.940 %). `sba_board_scan`'s five
  per-card list walks are twenty-two bits in `CardMemo`, the atomic word
  that already carries the printed colours — one `clear()` store however
  many answers ride on it, and the read's `debug_assert!` is the audit.
  **This is also the counter-example the rule above needs: (-11) said such
  a cache "cannot be a lazily-cached field" because ~20 sites rewrite a
  definition through `Arc::make_mut`, and that was true until pass 83 built
  the chokepoint.** A refutation written against an *argument* dates; one
  written against a *measurement* does not. PERF's `(-77)` has the device
  and the three tests a candidate has to pass.
- **The memo pays for the walk it replaces, so price that walk on the card
  that answers "no"** (pass 87, `(-77)`'s fourth row, built and reverted:
  `fixed` **+0.106 %**, `cube` -0.067 %, `sealed` -0.078 %).
  `dispatch_board_scan`'s per-card body is `for sa in
  &def.static_abilities {}`, i.e. **one length check** on a definition with
  no statics — a memo load, a mask and three tests do not beat that. The
  three rows that shipped removed five list walks, five pointer-chased loads
  and three list walks respectively. Bits were correct (the function's own
  `debug_assert!` against the four walks it fuses passed); the trade was not.
- **…and the rule that qualifies it: count the work items the bit elides,
  not their kind** (pass 88; the same function TAKEN at `fixed` -0.095 % /
  `cube` -0.317 % / `sealed` -0.330 %, and `granted_abilities_of` REFUTED at
  `fixed` +0.024 % in the same pass). `dispatch_board_scan`'s per-card body
  is four things — an `equipped_bonus` load, a `static_abilities` walk, a
  `station` walk, an `active_static` loop — and `if bits == 0 { continue }`
  skips all four; `granted_abilities_of`'s prologue is one walk over an empty
  list, and six exact bits cannot beat one length check. "The bit *is* the
  answer rather than a gate" is necessary and **not** sufficient.
- **A gate's own row lies about whether the gate pays** (pass 88's narrow
  probe). `trigger_grant_sources` gets **worse** under the shipped change
  (`cube` 14,365,404 -> 14,898,698), and removing that gate and three
  siblings costs the program `cube` **+0.171 %** / `sealed` +0.143 % — the
  memo a gate warms is read again by the next consumer, so the cost lands on
  the gate's row and the saving lands on someone else's.
- **Price the *walk*, not the function's row: a whole-board presence row can
  be 98 % iteration** (pass 89, `(-77)`, built and reverted: `fixed`
  +0.031 % / `cube` -0.040 % / `sealed` +0.015 %). `card_can_change_creature_
  types` and `card_can_change_land_types` on a `change_bits` family with
  **its own valid flag** — the prescription the rule below writes — moved
  their two rows (1.05 % of `cube`) by 1.0 % and 1.9 %, because the per-card
  body was never where they spend: the closures run 27,794 and 45,394 times
  at 642 and 353 Ir, i.e. ~20 cards at ~22 Ir, and those 22 Ir are the `Arc`
  deref, the iterator and the `any` closure — not the `Option` check and two
  length checks the bit replaced. **The shared slot explained pass 87's loss;
  it was not hiding a win.** The rule above counts the work items a bit
  elides; this one says to count them *per card visit*, which
  `scripts/cg_calls.py` does by division.
- **A memo slot's miss path is the sum of every family on it, and it is paid
  by whichever consumer touches the card first** (pass 87, same entry, built
  and reverted: `fixed` +0.135 %, `cube` +0.145 %, `sealed` +0.097 %). Adding
  `card_can_change_creature_types` and `..._land_types` to the *winning*
  `type_bits` slot took it from two bits to six — three times the miss, no
  fewer misses — and ate row (4)'s own win on the pool where it had won.
  **Give a new family its own valid flag, or re-measure the slot's miss after
  widening it.**

- **A "more exact" reserve is still a reserve, and `ContinuousEffect` is a
  large struct** (pass 86; `fixed` +0.461 %, `cube` +0.401 %, `sealed`
  +0.373 %). The gather sizes `all_effects` by a *card* count while
  `push_static_ability_effects` emits per *ability*, so sizing it by the
  ability count is strictly closer — and it is the fifty-fourth pass's
  `+ battlefield.len()` (+1.54 %) all over again, because most statics emit
  nothing and every extra slot is a kilobyte on 71,930 gathers. "Exact" in
  that entry means exact in *emitted effects*, which nothing cheap knows.
- **Compare a buffer's growth count to its *call* count before inlining it**
  (pass 86, `(-71)`'s sweep). A `grow_one` row says how often the buffer
  allocated; the `SmallVec` is paid on every call. The gather's `sa_cards` is
  69,896 growths over 71,930 calls (97 %) and shipped at `cube` -0.513 %;
  `statics_granted_triggers_inner` is 19,128 over **142,744** (13 %) and
  measured **nothing**, because a returned buffer costs a 40-byte move on the
  87 % of calls that never allocate. Reverted.
- **Inline storage is a *local's* device; on a struct field the read count
  pays for it** (pass 86, `(-72)`, built and refuted: `fixed` +0.600 %,
  `cube` +0.490 %, `sealed` +0.542 %). `players: SmallVec<[Player; 4]>`
  removed 22,684 allocations on `cube` — one per `GameState` clone, to the
  unit — and cost 16.5 M Ir, because `sa_cards` has ~forty read sites in one
  function and `.players` has **~35,000** across the workspace, each paying
  the `spilled()` compare. `dispatch_triggers_for_events` alone took
  +3.6 M Ir without touching an allocation. **Count the read sites before
  moving a buffer inline**, and do not retry it at a smaller inline capacity:
  the cost is per read, not per byte. **Confirmed a second time in the same
  pass**: `PlayerData`'s `spell_ids_cast_this_turn` and `spell_casts_this_turn`
  are `clear()`ed per turn and regrow only because the CoW deep copy's
  `Vec::clone` hands back `capacity == len`; inlining them removed all 22,930
  of `finalize_cast`'s growths — the largest `grow_one` row on `fixed` — and
  read `fixed` +0.366 % / `cube` +0.291 % / `sealed` +0.322 %.
- **A DORMANT GATE'S COST IS ITS CALL COUNT** (pass 97). `(-85)` priced a
  presence gate at "~113 k Ir and nothing else — add gates freely" on a
  ~20 k-call site. In front of `presence_gate`, asked **242,788** times a
  `cube` run, a census hook read `cube` **+0.049 %** as a `OnceLock<bool>` and
  **+0.187 %** as an `#[inline]` `AtomicU8` fast path — *worse*, because
  `#[inline]` expanded the reader's **cold** branch (an `env::var` returning a
  `String`) into all 242,788 call sites. Two rules: multiply ~5 Ir by the site's
  call count before adding a gate, and **never `#[inline]` a reader whose slow
  path allocates**. `CRAB_SBA_CENSUS` (20,152 calls, +0.001-0.004 %) is on the
  other side of that line and stayed.
- **FILL A `SmallVec` WITH A LOOP, NOT A `collect()` — and this is the
  largest number in the file for that rule** (pass 97). The SBA death gate's
  candidate list, one `collect` over the battlefield in a function called at
  every priority pass, read `fixed` -0.065 % / `cube` -0.362 % / `sealed`
  +0.007 % as a collect and **-0.580 / -0.766 / -0.549 %** as a `for` loop —
  0.515 / 0.404 / 0.556 percentage points, i.e. the collect gave the whole win
  back on two pools and reversed the sign on the third (`SmallVec::extend`
  +8.0 M and `call_mut` +10.5 M on `sealed`). `Vec::from_iter` specializes to
  internal iteration and `SmallVec`'s `Extend` does not, so the collect is an
  external `next()` loop with a spill check per element. **The inline storage
  is still worth having — it is the iteration protocol that costs, not the
  buffer.** The ninety-sixth pass's `sorted` entry reached the same rule from
  `extend()`; `blockers_of` reached it from `collect()` at -0.072 %.
- **A `SmallVec` without the `union` feature is an *enum*, and the
  discriminant match is the whole trade** (pass 86, `(-71)`; 0.12 % of
  `fixed`). Inlining the gather's `sa_cards` buffer read **+0.108 % on
  `fixed`** and -0.458 % on `cube` — a pool split — until the feature was
  turned on, after which it is -0.012 % / -0.513 %. Every read of a non-union
  `SmallVec` matches a discriminant on top of the `spilled()` compare, ~40 Ir
  per owner call. Two shapes measured and refuted alongside it: inline
  capacity 4 (removes the same growths, then pays 10,782 spill allocations)
  and shadowing the buffer with a `&[T]` after the fill (holding the borrow
  across 3,600 lines is worse on every pool).
- **The `grow_one` caller table ranks the local accumulators, and a row named
  for a function is not necessarily the buffer you think** (same entry).
  `gather_continuous_effects_inner`'s row on `fixed` is `all_effects`, not
  `sa_cards` — the four bench archetypes carry no permanent with a
  `static_abilities` entry, so that buffer never allocates on that pool, and
  the shipped change left `fixed`'s allocation table *byte-identical*. Read
  the row on the pool the change is aimed at.
- **A gate that rides on an existing *early-exiting* scan is not free**
  (pass 85, the concurrent half, `(-68)`; estimated 0.5-0.8 % of `cube` and
  measured 0.317 %). Three of `fire_combat_damage_triggers`' six battlefield
  walks were gated on facts the dealer lookup could compute on the way past —
  but that lookup was a short-circuiting `find`, so widening it to a full walk
  gave back part of the saving, and the arithmetic that priced the entry had
  not counted what the `find` was skipping.
- **When a pass deletes the work an abstraction existed to amortise, the
  abstraction is the next thing to read — and its doc comment will not tell
  you** (pass 85, the concurrent half, item 1; `fixed` -0.901 %, `cube`
  -0.806 %). `bot::ProbeCell` cached `state.affordance_probe_template()` for
  probes that each cloned it; the function had become plain `self.clone()`
  when a different pass deleted the library strip it existed to amortise, so
  the cached value was equal to the state it was made from. The cell was
  lazy, documented and carried real numbers — about work that no longer
  existed. **Grep for the shape, not the symptom:** `affordances.rs` still
  has ~12 of it.
- **Ask whether a function reads its parameter's *value* or its *support***
  (pass 85, the concurrent half, item 2; -58.5 % of `static_build_score`).
  `score_brief_with_colors` took a `ColorCounts` and touched it only through
  `is_empty()` and `get(c) > 0`, which makes its colour term a function of a
  five-bit set — and then `off = total - on` collapses two accumulators into
  one masked sum and the rest of the function into a memo field.
- **Memoize the pool, not just the cards** (pass 85, the concurrent half,
  item 3; `sealed_pool` -25.9 %). The card definitions had been memoized for
  thirty passes; `sos_draft_pool` and `SosPacks::new` rebuilt the *pool* made
  out of them per pool. **No profile row said so** — the cost was spread over
  three small ones. Ask what else in a prologue has no inputs.
- **Run a sweep on the range you found the bug in, before *and* after**
  (pass 85). A fix's own insurance was a second bug, on a seed the pre-fix
  sweep had passed; only the same 4,000-seed range re-run afterwards told a
  fix from a trade. **A sweep that runs only after the change cannot tell
  those apart**, and "the tests still pass" is not the same statement.
- **A thin candidate list is not an empty engine** (pass 84, and pass 85's
  concurrent half is three of four). Three of that
  pass's eight rows were on no candidate and in no self table —
  `Option::or_else`, the boxed keyword list, `printed_color_set` — and all
  three came from *counting* (call counts, allocations per call, a hot
  function's callee list), which is the three rules below this one.

- **Read the instruments before the profile, and know which kind of "no" you
  are holding.** `CRAB_SIM_REJECTS`, `CRAB_PAY_FAILS` and `--bench`'s stall
  split are counts of the *workload*, and the largest row in fifteen passes
  came from one of them rather than from a dump (PERF's Log, pass 83 item 0).
  **A "do not re-open" written against an *argument* dates the moment the
  evidence moves; one written against a *measurement* does not.**
- **A shrinking instrument is not a dead one** (pass 85 item 2). `--decks
  sealed --games 1` was retired in two sections of PERF for having fallen from
  2.9 G Ir to 21.9 M, and it is 76.5 % deck construction — the fall was five
  passes of work on the thing it measures. Read a candidate instrument's
  callee table before retiring it; an absolute is not a share.
- **Size a clone removal by the copy's whole lifecycle** (pass 85 item 0).
  (-65) was priced at ~0.2 % off `TriggeredAbility::clone`'s own row and
  shipped at 1.195 % of `cube`: the allocation, the `memcpy`, the `grow_one`,
  the drop and the `free` were all outside the row that named it, and the
  `clone` row was the smallest of the five.
- **After deferring work out of a hot path, re-read what the path still
  computes for it** (pass 85 item 1(d)). Moving land assembly off the shape
  lattice left two pool-sized vectors per shape being built, joined and
  dropped with no reader. **Removing a caller does not remove what fed it.**
- **Rank the dump by call count and read the Ir/call column** (pass 83's
  fifth commit, `fixed` -0.444 % / `cube` -0.568 %). `Option::or_else` was the
  most-called function in the program — 2,187,078 calls, all but 54 of them
  `evaluate_requirement_static_hinted`'s fallback chains, ~5 Ir apiece and
  invisible to a self table, a callee table and a line profile alike. A row
  with a million calls and single-digit Ir/call is pure call overhead, and the
  only question is which kind: a non-generic `crabomination_base` callee is a
  **profile artifact** (`release`'s thin LTO inlines it — the
  `CardDefinition::is_creature` trap), while a std generic the local inliner
  declined is **real**, and the fix is restructuring the call site, never an
  `#[inline]`.
- **Read a hot function's callee list and ask which rows are doing *work***
  (pass 83's eighth commit, `fixed` -0.173 % / `cube` -0.184 %).
  `compute_permanent_pass` makes three per-call definition reads;
  `base_power` and `base_toughness` are 13 Ir and `printed_color_set` is
  **56**, because it walks the keyword list, the colour indicator and every
  mana symbol. It is 32nd by call count and never rises above the noise in a
  self table. Three rows at the same call count with one an order of
  magnitude dearer is the tell.
- **A memo's invalidation point is priced by the *write* rate, and the
  tightest-looking key is not always sound** (same commit). Keying the colour
  memo on `Arc::as_ptr(&definition)` misses the MDFC face-swap and Mind
  Bend's override, which `Arc::make_mut` performs **in place** on a uniquely
  owned definition; `CardInstance::DerefMut` is the one point both must pass.
  It is also hot enough that the clear eats about half of what the hits save.
- **A redundancy you cannot remove without a refactor can still be priced by
  *adding another copy of it*** (pass 85, `(-70)`). `ComputedPermanent` holds
  four `Arc<CardDefinition>` handles to one definition; removing three is
  600-800 call sites, but *adding* three is a two-line field, and it reads
  `fixed` +0.532 % / `cube` +0.580 % — the change with the sign flipped. Pair
  it with the padding probe below and a struct's size and its handle count are
  both priced without touching a call site.
  **…and a sign-flipped probe is a FLOOR, not the answer** (pass 91, the same
  entry taken: `fixed` -0.626 % / `cube` -0.706 % / `sealed` -0.741 %, i.e.
  18-28 % over). Adding three handles leaves the struct's drop glue and its
  allocation size class alone; removing three takes a whole
  `drop_in_place<PrintedList<_>>` row (7.9 M of `cube`) and moves the
  `Arc<ComputedPermanent>` into a smaller size class. **Price the *margin*
  with the probe, then expect the removal to beat it.**
- **A freeze scope only gathers if a read inside it asks for a computed view,
  and the read that pays is whichever gets there first** (pass 91; `fixed`
  -0.755 % / `cube` **-2.226 %** / `sealed` -0.856 %, 71,930 gathers ->
  59,010). `(-81)`'s context census reads every gather as "one per scope, one
  scope per distinct game state", which makes the count look irreducible —
  and `resolve_combat`'s pair scope was gathering because its **first two**
  `&self` calls asked for one permanent's computed keyword set with no
  presence gate, while the six behind them were all gated.
  **Read a scope's first few reads in source order before concluding it has
  to gather.** Expect (-14)'s "guarding one promotes the next": the gate that
  was already there gets 27 % dearer because it now runs in an unwarmed scope,
  and the trade is still 44 M against 17.5 M.
  **…and the necessary second half, from the same pass's refutation: check
  what the scope does *after* its first read** (`board_keyword_in_scope` in
  `declare_attackers_banded` / `declare_blockers`, built and reverted: `fixed`
  -0.026 / `cube` -0.044 / `sealed` **+0.005** %). Gating the first read only
  *moves* the gather when the scope goes on to compute permanents anyway, and
  then the gate is paid for nothing. The pair-scope fix paid because every
  other read in that scope was already gated.
- **A private field is a compiler-driven rename** (pass 91). Turning four
  `pub` fields into private overlays behind same-named accessors is 2,991
  lines over 282 files, and none of them were found by grep: E0616 carries a
  machine-applicable "call it with parentheses" span, so
  `cargo check --message-format=json` plus a byte-exact patcher converges in
  four rounds. **A method and a field may share a name**, which is what keeps
  the diff to inserting `()`. Watch for the four shapes the parenthesis pass
  cannot fix: an inherent method the new return type does not have
  (`as_slice`), a `.clone()` that used to clone the wrapper and now reborrows
  (`to_vec()`), `assert_eq!` against the owned type, and
  `let x = &cp.field().sub` — temporary lifetime extension applies to a place
  expression, not to a call.
- **When a change trades a known saving against an unknown cost, build the
  cost alone first** (pass 83's seventh commit). Unboxing `layers::Printed`
  read +1.755 % and the narrower keyword-only version needed the *struct-size*
  half priced without the refactor: **a `u64` of padding on
  `ComputedPermanent`, two lines**, read `fixed` +0.040 % / `cube` +0.058 %,
  under the saving on both pools. That turned "do not take it on this
  arithmetic" into a decision in one build, and a linear extrapolation from
  the +208-byte data point would have over-priced it by 70 %.
- **A ceiling measured by short-circuiting a condition is an upper bound on
  the *code*, not on the walk** (pass 83's (-62), and it is 30 % of one).
  `false &&` and `std::iter::empty()` let the optimizer take the surrounding
  structure with it, so the graveyard pair priced at `fixed` -0.717 % by
  deletion shipped at -0.499 %. Both candidate explanations for the
  difference were built and measured **flat** — the memo's miss path
  (-0.003 %) and the memo read itself (0.027 % of the program; splitting it
  for inlining read +0.016 %). Read such a ceiling as "no implementation of
  this gate beats X" and do not spend a pass chasing the rest.
- **Before pricing a walk, grep for the other consumers of the fact it
  computes** (same entry, and it was 63 % more prize than the entry carried).
  (-62) sized `gather_continuous_effects_inner`'s `GraveyardAnthem` pass;
  `keyword_grant_in_scope` walks the same zone for the same variant and no
  entry had named it. Both match the variant by name, so the second walker
  was one grep away.
- **A number a sweep reports needs its breakdown at the same call site**, or
  the sweep only generates a follow-up question. The undecided count is the
  case: `SimCost::record` split capped / stuck / draw the moment a game
  ended, and only `--bench` printed the split, so a robustness sweep on
  `--decks` reported a bare "20 undecided in 44,400" and answering the one
  question that matters — rules outcome or broken loop? — cost a rebuild.
  They were all draws, i.e. **zero capped and zero stuck**, a much stronger
  result than the bare count looked like.
- **A share is a ratio whose denominator you chose.** Read the absolute Ir
  next to the percentage: one is about your code and the other is about
  everyone else's. Two ways this has bitten — a fixed startup cost that reads
  4.34 % at 20 games and 1.40 % at 60 with *identical* Ir (PERF's actor
  profile), and (-53)'s share falling a third further than its work did
  because the program grew 2.63 G -> 3.57 G around it.
- **Ask what the answer costs when it is "no"** (pass 59, ~1.5 % of `sos`
  across four sites). None of them was a hot function: a SipHash of ~84
  small integers for a digest compared only within one process, an
  iterator stack collected into an always-empty `Vec`, a `flat_map` over two
  always-empty command zones, a battlefield `filter` for a card no bench deck
  contains. A presence question is paid on every sweep or dispatch whether or
  not it can fire. `cg_edges.py --callers SpecFromIterNested` **ranked by
  calls, not Ir** is the table that finds them; PERF's (-44) has the rest.
- **A line profile's row is what that source construct's instructions cost,
  not what removing it would save** (pass 61, +0.12 % and reverted; **pass 106
  re-derived the identical candidate off a fresh line profile and paid four
  builds for +0.555 / +0.438 / +0.634 %** — see the Log). `cg_lines.py` put
  `is_event_hardcoded`'s `match ev` at 0.38 % of `sos` (10,240,198 Ir, 0.46 %
  of `cube` at the later tip) inside the biggest engine row; tabulating it
  per event moved the function's own self cost by **-58 k** and added 7.9 M of
  `SmallVec::Extend`. **Ask what the loop still does when the line is gone**
  before costing the row — a three-way match on a class byte is the same
  load-and-branch the enum match was. **`is_event_hardcoded` /
  `dispatch_triggers_for_events`' per-event gate is CLOSED; do not take it a
  third time.** Its line row will keep looking like the largest in the
  function, because that is where those instructions execute.
- **A presence bit belongs in a shared scan only when the question has no
  early exit of its own** (pass 59, +0.29 % on `fixed` and reverted). Folding
  `card_type_change_unscoped`'s battlefield leg into `sba_board_scan` cost
  more than the walk it removed, because the standalone `any` short-circuits
  per card and a scan bit has to finish. Third loss for the (-6) fusion
  device inside `creature_death_possible` alone.
- **This branch is rebased constantly, so a hash in a doc is a liability**
  (pass 58, three stale ones in one session — `223c77b5` twice, and a
  write-up that named its own commit before the rebase renamed it). Cite a
  hash only for a commit that is *already* on `origin`; for the commit the
  paragraph is describing, name it by title, and re-grep for the short hash
  after every `rebase --continue`.
- **A memoized object is not a memoized *answer*** (pass 54, and it was
  -68.8 % of the deck build). `card_def` had cached the `CardDefinition`
  since pass 53, and the builders still re-derived everything they read off
  it — the pip counts, the mana value, four `Vec<CardType>::contains`
  scans, the keyword walk in `card_quality`, the whole effect-tree walk in
  `is_fixing_card` — per (pick x candidate x colour shape). `CardBrief` is
  where a new derived fact goes; the memo is sound because a leaked
  definition is never mutated.
- **Callgrind runs the system allocator and mimalloc ships** (pass 54). An
  allocation-shaped change reads larger in Ir than it measures: -68.8 % of
  the deck build was +8.5 % of `selfplay_train`'s throughput. Get the
  `selfplay_train` number before sizing the next allocation-shaped
  candidate, and alternate — the same base binary read 129.1 and 156.5
  games/s minutes apart, so quote best-of-N and never a single run.
- **Ask which pool the change lives on** (pass 53, and it is the rule that
  found the two largest costs in the simulator). `--decks fixed` carries no
  `GrantTriggeredAbility` static and builds its decks once, so the per-card
  grant walk and the whole deck builder are dead on the bench. A change to
  statics / grants / layers / the requirement walker gets a `--decks cube`
  reading too; a change under `draft.rs` / `recommend.rs` / `selfplay.rs`
  gets `--decks sealed --games 1`, which plays no games and so isolates deck
  construction exactly. PERF's "Which pool a change moves" has the recipes.
- **The freeze scope that pays is the one around a loop whose borrow already
  proves it** (pass 53). Three per-card grant walks each hold a shared borrow
  of `self.battlefield` for their whole body, so no `&mut self` call can
  happen inside — which is exactly the freeze-scope invariant, checked by the
  compiler rather than by hand. Bare `freeze_layers_push`/`pop`, not
  `with_frozen_layers` (the closure costs ~0.9 % because the loop's locals go
  through its environment), gated on a fact the loop already computes.
- **Rank the tail, not the function** (pass 49). A chain of narrow generators
  is invisible in a self-cost profile and in a callee table sorted by Ir; it
  shows up only in the **call counts** — twenty-two rows at exactly 2,176
  calls each, once per traversal, on a board with nothing for any of them,
  4.9 % together. Wherever the code reads as a fallback chain, count the rows
  before costing them. The device is `spec` / `gated_block!`'s and the debug
  audit is what makes a mask safe to over-approximate.
- **Ask what a tick pays when the answer is "nothing to do."** Four of pass
  49's five rows are that question at a different level: a
  twenty-two-generator fallback chain, three land blocks asking the same
  question, a gate that gathered to prove a negative, a freeze scope opened
  for a closure that returns immediately, two clones handed to a walk that
  skips.
- **Build the answer *after* asking whether anyone wants it** (pass 55,
  eight of its ten commits, -20.7 % of the cube pool between them). A gathered
  layer view a presence gate answers; a requirement tree cloned to build a
  residual the caller discards on 99.3 % of calls; a CoW zone unshared to
  restore flags that never moved; a `battlefield_find` for the card the
  caller is iterating. **In five of the eight the cheap question was already
  a function in the file** — `requirement_mentions_power`,
  `creature_type_change_in_scope`, `evaluate_requirement_static_on`. Grep
  for the cheap form before writing one.
- **Ask a `collect()` how often it is *empty*, not how big it is** (pass 55,
  the two commits that pay on the bench pool). An empty `collect()` still
  calls `Vec::from_iter`; two per gather and two per state-based-action
  sweep were -0.7 % of `--decks fixed` between them. The worklist is
  `cg_edges.py --callers __rust_alloc` ranked by call count, then `grow_one`
  one level up.
- **Rank a lazy cell by what is under it, not by its own row** (pass 55, and
  it was 16.9 % of the cube pool). Candidate (-37) was sized at
  `computed_permanent`'s 4.14 % + `compute_permanent_pass`'s 2.97 % — self
  cost. Read from the top, the requirement walker's `OnceCell::try_init` is
  **413,844 calls / 605,927,621 Ir inclusive, 15.03 %**. `cg_edges.py
  --callers <callee>` is the table; the self table cannot see a cell.
- **Two shapes that look like wins and are not** (pass 55, both measured and
  reverted). A `OnceCell` around a presence gate that runs *once* per call
  is two constructions and a branch that never pay (+1.24 M on `fixed`) —
  `evaluate_requirement_static` evaluates one `req` and its arms are
  exclusive. And asking the cheap question first — `!computed_absent()`
  before the gate — is **+0.066 %**, because inside a freeze scope the gate
  is a memo hit at 69 Ir and the atomic load is not free.
- **The gate rule, both halves.** Swap a gather for a presence gate **only
  where nothing else in the scope reads the gather** (pass 48's (E),
  -0.747 %); where the scope goes on to `compute_battlefield()` the same swap
  is **+0.30 %**. `layers_memoized()` answers "is the gather already built".
  Fusing a cheap per-card question into a walk that already happens has lost
  four times; removing a walk outright still pays.
- **The `Keyword::eq` device is not exhausted, and its trap is now MEASURED
  rather than argued — the rule was right in direction and 5x wrong in size**
  (pass 93; see the Baseline). The argument was: no LTO in the profiles this
  file quotes, so any small non-generic `crabomination_base` function is an
  out-of-line call here and a bare `#[inline]` on it is unmeasurable in the
  shipped thin-LTO build — **"do not take one on an Ir number"**. Eleven
  `#[inline]`s on `CardDefinition`'s card-type predicates, read on both sides:

  ```text
                    release-fast (no LTO)      + thin LTO
    fixed              -0.907 %                  -0.175 %
    cube               -0.741 %                  -0.124 %
    sealed             -0.831 %                  -0.162 %
  ```

  **Thin LTO recovers ~80 % of it and not the rest**, consistently on three
  pools, so the residual is real and the attribute ships. Two durable halves:
  **an Ir number from a no-LTO profile over-states a cross-crate `#[inline]`
  by about five times** — halve-and-halve-again before believing one — and
  **the way to settle it costs one profile**, `[profile.profiling-lto]`
  (`profiling-fast` + `lto = "thin"`), because `profiling` itself is
  unbuildable here: rustc peaks at ~5.9 GB on the engine's single codegen unit
  and the container's memcg kills it, **and the `#[inline]` side reproducibly
  needs more than the base**, so the two halves of that A/B are not both
  buildable. What *also* works is making the callee smaller than any inliner
  threshold, which is what `has_kw` does.
  **…and the third half, which is what stops this becoming a sweep: the next
  tier is built and refuted, and the 10-Ir one-liner is in it.**
  `has_keyword` + `counter_count` + `same_team` read **+0.453 / +0.541 /
  +0.418 %**, and **+0.154 / +0.169 / +0.120 %** with `has_keyword` dropped —
  every combination loses. **Pick an `#[inline]` candidate by what its body
  EXPANDS TO at the call site, not by its self Ir**: `counter_count`'s row is
  10 Ir and its one statement is a map lookup, expanded at 65,606 sites, while
  the card-type predicates expand to a `Vec::contains` against a constant
  discriminant over a one- or two-element list, which the caller folds. A
  call-count ranking puts the worst candidate first.
- **A presence gate in front of a loop is only free if the loop was not
  already empty** (pass 57, and it is the one thing the second session
  measured that the first did not). The gather's thirty-eight `sa_cards`
  passes: gating by swapping the iterated slice read **+1.076 % on `--decks
  fixed`**, a board branch outside the loop +0.551 %, and the same test moved
  *inside* the loop (`if mask & bit == 0 { break; }`) **+0.234 %** — while
  taking cube from -1.85 to -2.26 % and sos from -2.75 to -3.16 % at the same
  time. `fixed`'s `sa_cards` is empty on all 32,002 gathers, so the walks were
  already free there and anything in front of them is pure charge.
- **Read the Ir/call column of a caller table, not just the calls or the
  total** (pass 60, -2.9 % of `sos` in one commit). `__memcpy` is 7.80 % of
  `sos` over forty diffuse rows — except `CardInstance::new`, 3,452 calls at
  **8,242 Ir each**. A memcpy costing eight thousand instructions is moving
  kilobytes, and `size_of::<CardDefinition>()` is **8,232**: every deck-fill
  site handed `CardInstance::new` a fresh `f()` and `Arc::new` copied the whole
  definition per card in a library.
- **A memo whose miss path is expensive is not a free memo** (same pass). The
  first version of `card_arc` rode `card_brief`'s memo, so a miss also paid the
  pip counts, the keyword walk and `is_fixing_card`'s effect-tree walk:
  **+6.591 % on `--decks sealed --games 1`**, which is all misses and no games.
  Its own memo reads +0.330 % there. Quote the *cold* workload for a memo.
- **The Ir does show up on the clock, and it over-reads by ~2x** (measured
  2026-08-25, and it is the first time anyone asked). Passes 57-59 together,
  `28ae2416` -> `49c7220d`, eight ABBA blocks a pool with a flat null control:
  **`sos` -6.57 % Ir / -3.87 % clock (8/8 blocks), `cube` -8.91 % / -3.16 %
  (7/8)**. So rank work by callgrind — deterministic, thirty times cheaper,
  and it found every one of those commits — but **halve an Ir delta before
  quoting it as throughput**, and expect a single commit under ~3 % of Ir to
  be unseparable on this box's clock.
- **Best-of is a biased estimator, and every clock number in this file before
  pass 59 was one.** `scripts/ab_wall.py` runs an ABBA schedule (linear host
  drift cancels inside a block), reports the mean per-block ratio with a 95 %
  t CI, fingerprints `decisions` on both sides and refuses to time two
  binaries that played different games. **Run its null control
  (`--bin-a X --bin-b X`) at the same block count before believing a
  verdict.** Calibrated on the routine box: `--games 2000 --decks sos
  --threads 4`, eight blocks, **+/-2 % and nothing finer** — four blocks
  called a null-equivalent result significant.
- **`Ir` counts a `memcpy`; the machine barely does** (`cae6b605`, and it is
  pass 57's clock rule with its mechanism named). Replacing
  `granted_abilities_of`'s deep-copied `Vec<ActivatedAbility>` with
  `Vec<&ActivatedAbility>` — 11,324 `ActivatedAbility::clone` and 11,324
  `__memcpy` a six-game run, gone — reads **-1.946 % on `sos`** and is **flat
  on the clock over nine alternated 20,000-game pairs**, under *both*
  allocators. A deep copy of a contiguous struct runs at high IPC out of a
  just-written cache line; the borrow that replaces it turns a hot-buffer read
  into a pointer chase into cold definitions. Keep such a change for its Ir and
  its clarity, but do not quote it as throughput.
- **A `Vec` returned to a caller that immediately drains it is two
  allocations, not one** (pass 57's (D) and (E), -1.4 % of cube between them
  on top of the mask). `static_ability_to_effects` collected a `Vec` per
  static-ability card and `static_effect_to_effects` built `vec![one]` per
  emitted effect; both were `extend`ed into `all_effects` one frame up. The
  tell is in the callee table, not the self table: `SpecFromIterNested::
  from_iter` and an `IntoIter::drop` at **exactly the same call count**. Write
  through the caller's buffer (`out: &mut Vec<_>`) and patch
  `out[start..]` where the caller was patching the temporary.
- **A presence gate is sized by its arm's call count, not by what the arm
  costs when it is taken** (pass 56, one gate paid and two lost in the same
  sitting; pass 55 closed the same entry by sizing rather than building and
  agrees). `creature_type_change_in_scope` was worth taking out of the mutex
  because `HasCreatureType` is **410,900 of the requirement walker's 654,950
  calls** on cube; `HasArtifactSubtype` / `HasSupertype` are rare, and gates
  for them measured **+0.123 %**. Count the arm before writing the predicate.
- **A gather is one per freeze *scope*, not one per unscoped read** (pass 56,
  and it closes three rows of the contexts table that look like candidates).
  `computed_permanent` gathers when the scope's memo is empty — its first
  computed read — so N gathers in a context is usually N scopes each paying
  for itself. `pick_blocks` and `eval_material` are both that shape.
- **`cg_sites.py`'s number is a floor, and there are two data points.** The
  auto-tap source table read 0.15 % of `fixed` in that table and measured
  **-0.291 %**; pass 53's two sites read 0.35 % and measured -0.611 %. Do not
  decline a site because its `cg_sites` row looks small.
- **A wall-clock number for the deck build carries the process floor with
  it** (pass 56). `--decks sealed --games 1` is ~6 ms of build on a ~3.3 ms
  startup floor, so quote both columns; the tip's floor was 6.5 % *higher*
  than the base's on a bigger binary, and that came straight off the measured
  win. A training actor never pays it — it builds decks in one process.
- **Ask what varies with the shape** (pass 56, and it was -23.1 % of the deck
  build in five commits). The sealed lattice runs ~57 shapes over one pool;
  the pack buckets, each card's brief, the pool's pip totals and each card's
  score are all properties of the *pool*, and each was being rebuilt per
  shape. `PoolScores` is where a new per-pool derived fact goes.
- **Measurement.** Read PERF's "How to measure" — pass 48 rewrote it.
  `scripts/cg_symbolize.py` + `scripts/cg_edges.py`, never
  `callgrind_annotate --tree` for a caller table. **`cg_edges.py`'s shares
  were ~18x low until `ac85463f`** — a table read before that commit ranked
  work upside down; re-derive anything carried forward from one. Re-read your
  own base: on a shared branch the commit you stand on may not be the one the
  last pass measured, and argv length lands in the Ir total (~500 Ir).
- **Measurement gotcha, hit three times.** The 6-game ladder printout (24
  decided / 12 splits) does **not** move on a change that breaks something;
  only `--bench`'s `decisions` and the Ir total catch it. Run
  `./target/profiling-fast/bot_ladder --bench --threads 3` on any change whose
  Ir moves more than its blast radius allows.
- **Ir over-reads a commit that removes whole action executions by about
  two** (pass 75's clock, `ab_wall.py`, 8 ABBA blocks, `release-fast` +
  mimalloc, `CRAB_NO_JITTER=1` both sides, `--games 2000 --decks sos --seed 11
  --threads 4`): **-1.29 %, CI -2.19 .. -0.39, 6/8 blocks** against a FLAT
  null (+0.20 %, CI -0.70 .. +1.10, resolution ±0.90 %), where Ir read
  -2.775 %. The direction is the opposite of a *clone* removal (wall bigger
  than Ir, pass 68) and of an allocator swap (Ir blind to it, pass 64).
  **Under ~2 % of Ir will not show on this box's clock at all**, so do not
  promise a wall number for one.
- **`selfplay_train --seed N` does not reproduce a run** (pass 77). Same
  binary, same seed, `--actors 1`: 1,788 / 1,770 / 1,776 rows over twenty
  games; with `CRAB_NO_JITTER=1`, 1,788 every time. `bot::jitter_below` falls
  back to the *thread* RNG unless a seeded stream is installed, and
  `set_jitter_seed` is the ladder's device for antithetic pairs — nothing in
  the actor path calls it. **Pin the jitter for any actor-path measurement**;
  an unpinned reading compared a base that had played 1 % fewer rows.
- **Four standing measurement facts**, kept here because each has cost a pass
  once. **Callgrind Ir is portable across these containers** — four
  independent readings of one commit on four boxes agree to 0.0004 %, and the
  whole difference is argv length — so another session's Ir column is a usable
  base; its wall-clock and RSS columns are not. **A change whose soundness
  rests on a `debug_assert!` is audited by the `dev`-profile grid, not the
  `overflow` one**: release profiles compile the assertion out. **Plan actor
  counts off ~24 MiB RSS**, not the `--no-default-features` 17.7 — mimalloc is
  the shipped allocator and costs ~9.7 MiB a process. And **`--decks fixed` is
  the bench pool**: a change to statics / grants / layers gets a `--decks
  cube` reading too, and one under `draft.rs` / `recommend.rs` / `selfplay.rs`
  gets `--decks sealed --games 1`.
- **A refutation carries the workload it was measured on, and this branch's
  workload moves.** The seventy-sixth pass re-opened `can_afford_in_state`,
  closed at +0.066 % on 2026-08-12, and it paid -0.6 % on all three pools: the
  entry's own figures — 0.29 % of the profile in the walks, 1.13 cards per
  sweep — had become 1.14 % and 2.80, because the attack search now runs 1,910
  sims a `cube` run and every one of them sweeps. **Re-read a refutation's
  numbers against the current dump before trusting its verdict**, and record
  numbers rather than verdicts so the next re-check is one `cg_edges.py` call.
  This does *not* license re-taking anything under "Do not rebuild these"
  below: those are refuted on a mechanism, not on a ratio.
- **A wrong bot pre-filter is invisible to every invariant this file checks.**
  It costs Ir, not correctness, so a green suite, identical golden traces and
  a flat ladder all survive it indefinitely. The tell is the *ratio* between
  what the bot offers and what the engine completes — `restore_payment_state`
  against `try_pay_after_snapshot_mode`, `finalize_cast` against
  `cast_spell_with_convoke`. Grep the other pre-filters (`ward_tax_payable`,
  `pick_combat_trick`, `max_affordable_x`) for the presence-vs-count shape the
  seventy-first pass found in `available_mana`.
- **The oracle, and use it again.** A bot-side estimate of a rules question
  usually has an engine function that answers it exactly (`could_pay_cost`,
  `would_accept_on`). Wire it behind an env var at the *divergence* site,
  report only where the old estimate would have said yes, and sweep pools x
  seeds. At the seventy-first pass the count went **6 -> 6 -> 240 -> 0** and
  every non-zero named the card that found the hole; **the first two versions
  of that commit looked correct and were not**, and no reading of the code
  found what the oracle did.
- **Do not rebuild these.** Unboxing `layers::Printed`'s override
  (`Option<Box<T>>` -> `Option<T>`; +1.755 % `fixed` / +1.317 % `cube`, and
  the narrower three-field version prices out worse than the boxes cost — the
  *keyword-only* `Box<[Keyword]>` is the one that shipped, see PERF's
  Baseline (7)),
  the board-presence epoch, the `GameState` husk
  pool, gating `do_untap`, narrowing `GameState`, splitting the big engine
  files for build time, the `LayerFreeze` depth shadow,
  the trigger-carrier bitmask, the APNAP
  rank table, the headroom-reserving `Vec`, `board_keyword_matching`'s
  presence gate, presence gates for `has_atype` / `has_stype` (pass 56,
  +0.123 % cube), and (-31)'s `improves_this_turn` reuse. And **never** skip
  `push_ordered_trigger_candidates` on an empty batch (+7.3 % *and* a
  correctness bug — it owns the per-batch `died_card_snapshots.clear()`).
  **Three entries came OFF this list at the eighty-seventh pass because they
  shipped**: the `sba_board_scan` definition bitmask (`cube` **-0.700 %**),
  the per-definition keyword-grant bit (**-0.283 %**) and
  `card_type_change_unscoped`'s (**-0.405 %**). All three were listed against
  an argument — "a cached bit goes stale because ~20 sites rewrite a
  definition", "a scan bit cannot short-circuit where the standalone `any`
  can" — and both arguments were about mechanisms that changed: pass 83 built
  `CardInstance::DerefMut`'s memo chokepoint, and a *lazy per-printing* memo
  is not the *eager per-sweep* scan bit those entries measured. **What stays
  refuted, with numbers, is the fusion into `sba_board_scan` itself** — see
  the Baseline's eighty-seventh block. The memo on `dispatch_board_scan` was
  on this list for one pass and came off it at the eighty-eighth, taken at
  `cube` -0.317 %: the refuted shape gated a walk, the shipped one replaces
  the loop body. A line on this list is only as good as the mechanism it
  names.
- **Env.** `cargo-nextest` **is** installable in this image and this bullet
  said the opposite for twenty passes:
  `curl -sSLf https://get.nexte.st/latest/linux | tar xzf - -C ~/.cargo/bin`
  takes seconds, and `cargo nextest run -p crabomination -p
  crabomination_tests` runs the gate in **~110 s** after the build against
  `cargo test -j 2`'s ~25 minutes from cold. Cold everything (deps + catalog +
  engine, two profiles in parallel on 4 cores) is **~45 min a profile**; a
  warm engine-only rebuild is ~15 min, and a change in `crabomination_base`
  costs the catalog too, so **batch base-crate edits**. Workspace
  clippy needs `apt-get update && apt-get install -y libwayland-dev
  libasound2-dev libudev-dev libxkbcommon-dev`. Cold `profiling-fast` engine
  build ~14 min, warm rebuild ~4m30s; callgrind ~4 min and contention-immune.
  Wide-pool sweep ~55 s a seed — no excuse to skip it. Quote callgrind under
  5 %; a `profiling-fast` games/s compares to nothing, and a clock A/B needs
  `scripts/ab_wall.py` with eight blocks *and* its null control (~35 min).
- **Trackers.** TODO 1.0k, ROADMAP 0.66k, PERF 6.0k (**passes 45-49's
  Baseline blocks are one table plus the lessons they carried, and passes 45
  and 46's Log entries are folded, at the 58th tip**; the 48th's and 49th's
  Log entries are the next fold). ENGINE_BACKLOG **3.8k**, CARD_BACKLOG
  **4.0k**, CLIENT_BACKLOG 0.4k — all three triaged and indexed at the
  sixty-seventh pass; see NEXT item 10.

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

**That recipe still works and this file wrongly said it did not — corrected
at the eighty-fifth pass.** It runs **21,864,561 Ir** now rather than 2.9 G,
and a note added here and to (-63) read the fall as "it plays no games *and*
builds no decks, so it isolates nothing". It builds the same twelve decks;
`heuristic_sealed_build` inclusive is **16,732,968 of the 21,864,561, 76.5 %**
(`cg_edges.py --callees heuristic_sealed_build`). The 130x fall is five
passes of deck-builder work on the thing it measures. **Use it for anything
under `draft.rs` / `recommend.rs` / `selfplay.rs`:** twenty seconds under
callgrind, no `crabomination_ml` build, and 100 % of the delta lands in rows
you can name.

**And once you have two pools, rank the rows by the RATIO of their shares,
not by either share alone. That is a device, it is one script, and it found
the sixty-second pass's second commit.** A row that is 0.61 % of `cube` is
nowhere near the top of any table and nobody would look at it. The same row
is 0.12 % of `sos` — **5.08x** — and *that* is a pointer: whatever it does,
the grant-heavy pool does five times more of it per instruction, so the work
is pool-specific and structural rather than diffuse. Dump both pools at one
tip, parse `cg_edges.py <dump> --rows 0` into `{row: share}` for each, and
sort by `cube% / sos%` over the rows above ~0.45 % of cube:

**`scripts/cg_ratio.py` is that script** — it was described here as "one
script" for two passes before one existed, and every pass that used the
device re-derived the join by hand:

```text
python3 scripts/cg_ratio.py cg.cube.out cg.sos.out --floor 0.45
```

It reads `cg_edges.py`'s parse directly, so it has `--rows 0` semantics by
construction (a row truncated out of one dump is what makes a ratio read
infinite), and it prints rows with **no** denominator cost in their own
section rather than as a ratio — a row the other pool never executes is a
stronger finding than a large one, but it is not a number.

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
`draft.rs` / `recommend.rs` / `selfplay.rs` gets a **four-game actor run**
(`CRAB_NO_JITTER=1 selfplay_train --actors 1 --games 4 --steps 1 --seed 7`,
~1 minute under callgrind), which isolates deck construction exactly.
**and `--decks sealed --games 1` does the same job for a twentieth of the
cost** — see the correction under point 2 above; the claim that it "builds no
decks" was wrong and stood for two passes.
`fixed` stays the committed bench
because it is reproducible, cheap and *is* representative of the game loop
— it is the pool the Log's absolutes are comparable across — but it is not
the whole simulator.

```text
# the four pools, same config, at the fifty-third tip. The `totals:` line is
# the whole-program Ir every Log row is a ratio of — read it straight out of
# the dump rather than through `callgrind_annotate`.
for d in fixed cube sos sealed; do
  RUST_MIN_STACK=33554432 valgrind --tool=callgrind --callgrind-out-file=cg.$d.out \
    target/profiling-fast/bot_ladder --a gang --b gang --games 6 --threads 1 \
    --seed 1 --decks $d > /dev/null 2>&1
  printf '%-7s %s\n' "$d" "$(grep -a '^totals:' cg.$d.out | awk '{print $2}')"
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

## Test-suite cleanup does not buy build time at this scale — measured 2026-08-27

The suite convention says "test *execution* is nearly free; the cost is
compile + link", which is the reason given for deleting pure-data tests. At
the eighty-second pass nineteen of them went (the last of what
`find_data_tests.sh` finds that is not a false positive), and the number says
the motivation does not transfer to a sweep this size:

```text
cargo test -p crabomination_tests --test classic_sets --no-run,
one file touched, two readings a side

  before  11.85 s / 7.50 s        after  7.50 s / 7.64 s
```

**7.50 s both ways** — the 11.85 is a first-reading artifact and the second
readings are identical. 199 lines of the suite's 377,435 is 0.05 %, and one
integration binary rebuilds in 7.5 s either way. **So a data-test sweep is a
convention and clarity change, not a build-time one**; the levers that do move
this number are the ones already taken (fewer binaries, `test = false` on
bin targets) and they are structural, not per-test. Do not sell the next
sweep on compile time.

**AND "the last of what `find_data_tests.sh` finds that is not a false
positive" was measured against a script that was wrong three ways.** All
three are fixed at the ninety-first pass and all three matter, because the
output is a *delete list*: the `fn foo() {` line's opening brace was never
counted (so every test ended after one line), only the test body was scanned
(so a test delegating to a local helper looked pure-data whatever the helper
does — `sos/hybrid_lands.rs`'s six school lands call one that builds a game
and activates two mana abilities), and sacredness read only the doc comment
above the test (so a CR citation on the assert itself did not protect it).
The header carries the numbers per fix. The list as it now stands:

```text
  284 found, 15 sacred -> 269 candidates, 3,212 lines
      145 read exactly ONE `catalog::` factory   1,169 lines   <- the echoes
      124 read several                           2,043 lines   <- mostly the
                                                   per-set definition tables
                                                   the convention asks for,
                                                   i.e. already the folded form
  by directory (single-factory): modern 59, classic_sets 39, stx 32,
      core_rules 11, mh 3, recent_b 1
```

**The 145 single-factory echoes are the sweep, and it is spread over ~60
files at three lines each**, which is why it is written down rather than
taken here: it is a convention change with no build-time return (above) and
it collides with any concurrent session touching the suite. Take it when the
branch is quiet, as one commit, with the suite green either side.

**First slice taken, `stx/part_23.rs`, ninety-first pass.** Its nineteen
single-factory echoes — `_bNNN` batch bodies checked for mana value, printed
P/T, a keyword or two and a creature type — are one `PrintedShape` table now,
row-per-card, asserting exactly what the nineteen asserted.

```text
tests        19,073 -> 19,055   (-19 + 1)
LOC          -167 / +81, net -86 of the file's 2,489
binaries     8, flat            suite green either side, clippy clean
build time   not re-measured: the section above measured this exact question
             at nineteen tests and read 7.50 s both ways
```

**It is the shape the convention asks for and it is the cheapest slice** —
one file, one batch range, every row mechanical. The other ~126 are the same
job spread over ~60 files; the pattern is now in the tree to copy.

**Second slice taken, `classic_sets/ogw.rs`, ninety-third pass** — eight
echoes into eleven `PrintedShape` rows (two of them checked two cards apiece),
`classic_sets` 6,078 -> 6,071 tests, LOC -77 / +64, binary green either side.

```text
  the rule found doing it, and it is what stops the sweep over-reaching:
  a test that pins a card-specific EFFECT SHAPE does not fold.
    rna.rs   applied_biomancy   ChooseModesCast (2 modes, min 1, max 2)
             swirling_torrent   ChooseModesCast (min 1, max 2)
             growth_chamber_..  Search { filter: HasName("...") }
    ogw.rs   kor_castigator     CantBeBlockedBy(_) matched by VARIANT
  A `PrintedShape` row can only assert what the table has a column for, and
  pinning `CantBeBlockedBy`'s payload asserts MORE than the test it replaces.
  Fold the P/T-and-keyword echoes; leave the shape asserts as per-card tests,
  which is what "per-card tests assert what is unique" already meant.
```

**AND THE SCRIPT WAS WRONG A FOURTH WAY, FOUND THE SAME SITTING AND BY THE
SAME METHOD.** `modern/lands_equipment_vehicles.rs` sat at the top of the
by-file count with **fourteen** rows, and reading three of them showed all
fourteen delegate to `assert_fetchland_fetches` / `assert_deck_dual_land` —
helpers that seed a library, activate an ability, and assert a sacrifice and a
life payment. **The helper-collection pass had bug 1**: a helper closed as
soon as its brace depth hit zero, and a multi-line parameter list has depth
zero on its signature line, so the body spliced into every caller was the
signature. Sixteen live engine tests (those fourteen plus two callers of
`cast_and_resolve_at` in `core_rules/xtra.rs`) were on the delete list.
Population **266 -> 250** (235 + 15 sacred); **no row was added by the fix**,
which is the check to run on the next one.

**The reusable half is about the tool, not the tests.** A script whose output
is a delete list is a *safety* instrument, and every one of its **four** bugs
put live engine tests on that list. It had been run and quoted by three
passes, and *fixed* by one, before the fourth was found. **Re-derive a
filter's output against a handful of its own hits before acting on it** —
reading eight of the 185 found the first three, and reading three of the
fourteen in one file found the fourth. **Read the file with the most hits
first: a filter's false positives cluster, because they share a helper.**

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
GameState` block is not free of risk.

**A SECOND REASON WAS PROPOSED AT `(-109)` AND `(-110)` REFUTED IT — SO THIS
SECTION STANDS UNCHANGED, AND THE ROUND TRIP IS WORTH READING.** `(-109)`
argued that at `codegen-units = 16` with no LTO the module sizes bound what
can be *attributed* in them (a ten-line edit in `actions.rs` moving `cube` by
±0.7 %), which would have made a split a measurement lever as well as a build
one. `(-110)`'s null controls — uncalled `pub fn`s in `actions.rs` with zero
executed instructions, one of them instantiating the very `SmallVec::extend`
monomorphization `(-109)` blamed — read **+0.006 % at worst on three pools**.
There is no floor to subtract: what `(-109)` saw was its own diff's content,
not the instrument. **Splitting the oversized modules is a build-time question
only, and this section has already answered it.** The lever that *does* bear on the
33-41 s is the one already written down: **keep the integration-binary count
flat or lower, and never add a new top-level `tests/*.rs`.** Twenty binaries
is what the relink costs.

**AMENDED at the eighty-ninth pass, and the amendment is the useful half: the
count is not what it costs — the *critical path* is.** Two targets still built
a harness for zero tests (`crabomination_ml`'s `selfplay_train` bin, and
`crabomination_tests`' three-line stub lib; the sweep that added `test = false`
to seven engine bins never reached those two manifests). Turning them off took
the executable count **19 -> 17** and the relink read:

```text
touch crabomination/src/game/effects/mod.rs, then
cargo test --workspace --exclude crabomination_client --no-run
CARGO_INCREMENTAL=0 throughout (so these are not comparable with the 33-41 s
above, which ran with incremental on)

  before   150.7 / 121.2 / 120.4 s      (the first is still warming)
  after    121.4 / 119.6 / 123.7 s
```

**Flat.** `cargo test --no-run` builds targets in parallel, so the wall clock
is the makespan of the longest chain, not the sum of the links — and the two
removed harnesses were never on it. The standing rule survives *for the eight
integration binaries*, because those are the long chain; it does not extend to
"any target removed is time saved". **Before proposing a binary-count change,
ask which target is on the critical path.** The change was kept anyway, as
rule-compliance and dead work removed (CLAUDE.md already requires
`test = false` on a `[[bin]]` with no `#[cfg(test)]` block), not as a win.

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

**`incremental = true` ON `release-fast` IS A TRAP — 3.1x faster to build
and 2.2 % worse code, measured at the eightieth pass.** It is the obvious
answer to the optimized-rebuild cost and it must not be taken, because
`release-fast` is where `--bench` runs.

```text
warm rebuild, engine edit, `-p crabomination --bin bot_ladder`, 4 cores
  release-fast as shipped     108 / 108 s
  + incremental = true         34 /  35 s      (a `touch` reads 5 s and is
                                                not a real edit — rustc
                                                reuses every CGU)
  one-off priming build       357 s, and target/release-fast 1.8G -> 4.0G

same binary, callgrind, --decks fixed --games 6 --threads 1 --seed 1
  no incremental        1,141,851,263
  incremental           1,166,977,048        +2.20 %
  --bench identical both sides (195,616 decisions / 27.44 turns), so the
  delta is codegen and nothing else; the incremental binary is also 12 %
  *smaller* (125 MB vs 141 MB), which is the lost inlining showing up.
```

**A 2.2 % codegen shift on the benchmark profile is worse than a 3x build
win is good.** Every `--bench` row and every release-fast Ir in this file
would move by more than most of the wins it records, all of them at once and
none of them real — and future perf work would be optimizing code the
shipped profile does not generate. **The rule this makes explicit: the
profile you measure on is an instrument, and you do not adjust an instrument
to make it more convenient.** If the iteration cost is worth paying down, it
wants a *separate* profile that nothing measures on — which costs a cold
build (~45 min) and 4 GB on a box that hit 93 % twice in one session, so it
is a real trade and not a free one.

Still open, and untouched by the above: the *cold* `release` /
`profiling-fast` engine build (~13 min) is codegen-bound, where CGU
partitioning rather than query invalidation decides the cost.

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

**THE CRITICAL-PATH QUESTION THIS SECTION LEFT OPEN IS ANSWERED, AND THE
TARGET ON IT WAS THE CATALOG'S OWN TEST HARNESS — 110.7 s OF A 213.5 s
MAKESPAN (ninety-second pass).** The amendment above says *"before proposing a
binary-count change, ask which target is on the critical path"* and nobody had
run the timings that answer it, because the recipe that found the seven bins
(`cargo test -p crabomination -p crabomination_tests --no-run`) **does not
rebuild the catalog at all**, so the unit never appeared in its `--timings`.
A whole-workspace one shows it immediately:

```text
CARGO_INCREMENTAL=0 cargo test --workspace --exclude crabomination_client
  --no-run --timings, cold-ish, four cores. 230 units, makespan 213.5 s,
  658 s of CPU -> 3.08x parallelism.

  start   dur   unit
   29.6  110.7  crabomination_catalog  "lib" (test)   <- the largest unit
  113.0   97.7  crabomination          "lib" (test)
   28.3   84.7  crabomination_catalog  (normal)
   57.9   67.9  crabomination          (normal)
  164.4   49.1  crabomination_tests    classic_sets
  171.2   34.0  crabomination_tests    core_rules
  167.9   32.8  crabomination_tests    recent_b
  130.2   32.4  crabomination_tests    recent_a
  140.3   30.9  crabomination_tests    modern
    0.6   29.0  crabomination_base     (normal)
    0.6   27.6  crabomination_base     "lib" (test)
  138.6   25.8  crabomination_tests    stx
```

**Read the shape before the row.** The build has two regimes: **0-58 s is
dependency-bound** — only `crabomination_base`, then the catalog, so two of
four cores idle and nothing removed there can help — and **58-213 s is
core-saturated**, eight integration binaries plus two lib-test compilations
queueing (`classic_sets` waits from 125.8 to 164.4 s for a core, not for a
dependency). **In the second regime the lever is total work, not the chain**,
which is the other half of the eighty-ninth pass's amendment and the reason
its own change read flat: `crabomination_ml`'s bin and the stub lib are
small *and* early.

**`crabomination_catalog` has no `#[test]` and no `#[cfg(test)]` anywhere**, so
that 110.7 s compiled 190 k lines a second time to run zero tests, straddling
the whole saturated window. `test = false` on its `[lib]`:

```text
ABBA, 3 runs a side, interleaved on one settled box, base `eb42c05f`
touch crabomination_base/src/card.rs; CARGO_INCREMENTAL=0
cargo test --workspace --exclude crabomination_client --no-run
  before  234.0 / 234.4 / 241.3 s   mean 236.6
  after   208.2 / 209.1 / 212.1 s   mean 209.8    -11.3 %
  6 of 6 ordered right, the sides do not overlap

the same A/B under this section's usual recipe (touch game/effects/mod.rs)
  before  126.7 s      after  126.9 s              flat
```

**Both halves are the finding.** The win is real on a base-crate edit, a
manifest edit and every *cold* build — CI, a fresh container, the 15m43s
`cargo nextest run --workspace` this session paid twice — and it is exactly
zero on the engine-file loop, because touching `game/effects/mod.rs` does not
invalidate the catalog. **So the amendment stands and gains a second clause:
ask which target is on the critical path, and ask which edit the loop you care
about actually makes.** A build-time number without its *touched file* is the
same half-a-figure as a stall rate without its invocation.

**And the first attempt at this measurement was wrong in a way worth
recording: it read the two sides across a container restart.** Baseline
(before the restart, warm) 126.7 s; candidate (after, cold page cache, and the
series still descending run over run) 301.4 / 272.5 / 237.7 s. Taken at face
value that says the change *costs* 90 %. `scripts/ab_wall.py` exists for
wall-clock A/B on the simulator for exactly this reason and the build loop had
no equivalent — **interleave the sides (ABBA) and discard a warm-up, or do not
quote a build-time delta at all.** A one-sided series is not a measurement on
a box whose state moves.

## Baseline

Closing states from the `(-185)` tip down are in `PERF_ARCHIVE.md`, verbatim.

### `panic = "abort"` — closing state at the `(-250)` tip, THE NEW IR BASE

One build-profile commit on top of `(-249)`, behaviour-preserving
(three-pool outcomes identical, `--bench` counters identical, golden
traces unmoved, the suite untouched because test harnesses always
unwind). **Every Ir number after this line is on the new base; a
reading against a `(-249)`-or-older number is not an A/B.**

```text
  pool     base (-249)      tip (-250)       delta
  cube     1,876,460,069    1,817,493,748   **-3.142 %**
  fixed      681,439,653      656,319,384   **-3.686 %**
  sealed   1,959,940,755    1,886,392,273   **-3.753 %**
  wall     paired --bench, release-fast, two 16-pair runs: **+4.45 % / +5.51 %** median games/s

rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.10 GHz, 4 cores
suite   19,255 / 0 / 5 at the tip (156 s under nextest beside the grid's build); golden traces 7/7
clippy  --workspace --exclude crabomination_client --all-targets   clean at (-249); the
        change since is Cargo.toml's release profile and prose
release the release-fast typecheck gate: clean at (-249); the profile change is not code
--bench profiling-fast at the tip: **195,806 decisions / 27.49 turns / 611.9 per game /
        0 stalls** — counters identical to 2003d1cf; determinism ok (all pairs split);
        thread_determinism ok (3 vs 1 threads identical); peak_rss 20.7 MiB;
        bin_bytes 183,143,512 (was 219,672,984)
grid    robustness_grid.sh --no-actor on the (-250) tree — the audit binary now aborts on a
        fired debug_assert! (exit 134) and the script's rc check reads that as FAIL:
        ladder 30 cells / 33,120 games, 0 failures, cap 0 / stuck 0 / draw 0;
        assertion strings 9 in the audit binary. Then --no-build --no-actor --pilots on the
        same binary: ladder 30 cells ok again, **pilots 45 cells, 0 failures**.
cache   cachegrind, release-fast, cube: Ir -3.47 %, I1 misses -5.45 %, D1 -2.15 %,
        mispredicts flat; rows in the Log entry.
actor   selfplay_train builds under release-fast (+abort; candle and rayon included) and a
        smoke run (--actors 2 --games 60 --steps 1 --seed 7) is rc 0: 60 games, 5,793 rows,
        0 stalls, `actors:` 106.6 games/s beside a compile (not a throughput number).
```

**The device: the cachegrind axis named the lever.** The I1-miss table
said the program is front-end-bound and that layout, not width, is what
moves it (which is also what PGO's -24 % against `target-cpu`'s flat had
been saying for a hundred passes); the cheapest layout change in the
toolchain is to stop emitting unwind cleanup, and it was worth more than
the last twelve source legs together.

### The untap-static lane — closing state at the `(-249)` tip

One engine commit on top of the `(-248)` tip `b7285f4e`, behaviour-
preserving (three-pool outcomes identical, `--bench` counters identical,
golden traces unmoved). The run's other product is an instrument: the
first cachegrind reading (Profile of record, "THE CACHE AND BRANCH AXIS")
and `scripts/cg_cache.py`.

```text
  pool     base (-248)      tip (-249)       delta
  cube     1,883,973,537    1,876,460,069   **-0.399 %**
  fixed      681,812,730      681,439,653   **-0.055 %**
  sealed   1,967,054,280    1,959,940,755   **-0.362 %**

rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.10 GHz, 4 cores
suite   19,255 / 0 / 5 at the tip (113.9 s under nextest); golden traces 7/7 in it
clippy  --workspace --exclude crabomination_client --all-targets   clean at the tip
release the release-fast typecheck gate (debug-assertions off): clean at the tip
--bench profiling-fast at the tip: **195,806 decisions / 27.49 turns / 611.9 per game /
        0 stalls** — counters identical to 2003d1cf; determinism ok (all pairs split);
        peak_rss 21.4 MiB; games_per_s 325 at host_calib_ms 74 (not a committed number)
grid    robustness_grid.sh --no-actor on the (-249) tree (the new lane and its
        debug_assert audit, string verified in the audit binary): ladder 30 cells /
        33,120 games, 0 failures, cap 0 / stuck 0 / draw 0. --pilots not re-run: the
        change is one presence gate on a step every pilot reaches through the same
        `advance_step`, and the (-248) pilots grid covers the tree it sits on.
cache   cachegrind, cube, same recipe as the Ir baseline: I1 misses 76,094,636
        (4.03 % of Ir), D1 misses 34,108,069 (3.8 %), LL misses 148,725,
        mispredicts 36,771,576 (11.4 %; 31.8 % of indirect). A/B this axis with
        `python3 scripts/cg_cache.py <dump> <col>`; deterministic like Ir.
```

**The device: a step that reads as once a turn is once a turn on every
simulation clone**, and its cost rides the probe count, not the turn
count — `do_untap` was 2,834 calls a six-game `cube` run against ~160
real turns and had never been on a table. When a gate is "any static
at all", ask whether the pool's boards ever read it clear; on `cube`
they did not, and the walks it guarded ran on nearly every call.

### The consumer-read legs — closing state at the `(-248)` tip

Four engine commits on top of the concurrent session's `(-244)` tip
`e44e9d90`, each behaviour-preserving (three-pool outcomes identical
to the `(-242)` base at every leg, `--bench` counters identical, golden
traces unmoved). The first three legs were measured as a chain off the
`(-242)` tip `e6b58ca4` before `(-243)` and `(-244)` landed and were
rebased over them twice (no shared line; `(-244)` and `(-247)` both
touch `game/mod.rs`); `(-248)` was measured on the rebased tree. The
tip row below is the rebased tree against the `(-244)` closing state's
numbers.

```text
  pool     base (-244)      tip (-248)       delta
  cube     1,928,746,090    1,883,972,930   **-2.321 %**
  fixed      690,547,383      681,813,326   **-1.265 %**
  sealed   1,992,138,486    1,967,055,576   **-1.259 %**

  leg      cube      fixed     sealed    what (each against the leg before it, off e6b58ca4)
  (-245)  -0.278 %  -0.205 %  -0.196 %   pick_attacks_inner's computed CantBlock read behind board_keyword_in_scope
  (-246)  -1.470 %  -0.843 %  -0.831 %   declare_blockers' Flanking/Bushido/Rampage scope behind board_keyword_matching
  (-247)  -0.421 %  -0.044 %  -0.025 %   permanent_is_creature's printed line behind card_type_change_in_scope
  (-248)  -0.224 %  -0.167 %  -0.240 %   the two targeting-time keyword reads behind card_keyword_possible (on the rebased tree)
  chain   -2.377 %  -1.255 %  -1.288 %   product of the four

rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.10 GHz, 4 cores
suite   19,255 / 0 / 5 at the (-246) tree (pre-rebase), at the rebased (-247) tip and at (-248); golden traces in it
clippy  --workspace --exclude crabomination_client --all-targets   clean at the tip
release the release-fast typecheck gate (debug-assertions off): clean at the tip
--bench profiling-fast at the (-246) tree, the rebased (-247) tip and (-248): **195,806 decisions / 27.49
        turns / 611.9 per game / 0 stalls** — counters identical to 2003d1cf;
        determinism ok (all pairs split); peak_rss 21.3 / 21.6 MiB
grid    robustness_grid.sh --no-actor --pilots on the (-248) tree (the widened gate-keyword
        lane and the three new gates, audited by board_keyword_matching's whole-board
        debug_assert on every `false`): ladder 30 cells / 33,120 games, 0 failures,
        cap 0 / stuck 0 / draw 0; pilots 45 cells, 0 failures.
sweep   fresh seeds on the debug-assertions overflow build (target-audit, this run):
        20 primes 103..199 x --decks all x 400 games (136,000 games) and the same 20 x
        --decks cube x 400 (64,000): 200,000 games, 0 panics, 0 assertion fires, 0 stuck,
        26 caps / 24 draws. Every cap is seed 149 or 193 in either pool, and every one
        reads twin i32::MAX life totals with a library of one — the Beacon of
        Immortality class in ENGINE_BACKLOG's closed stall lead (the card is in the
        cube pool); draws are rules outcomes. ⚠ A Beacon cap costs ~100 s of a thread
        on the overflow build (seed 193: 892 s for the `all` cell against ~85 s, 488 s
        for the `cube` cell against ~40 s), so a pool that carries the card has a
        wall-clock tail an actor run sees; the cap is the clock, and lowering it is a
        harness decision, not a rules one (ENGINE_BACKLOG has the numbers).
```

**The device: rank a memo's asks by caller, then read what the caller
CONSUMES of the view.** `computed_permanent_hinted`'s 284,812 asks
(11.3 % of `cube`) were ranked by caller on the `(-242)` dump; the
rows whose consumer was one keyword (`(-245)`), three keywords
(`(-246)`) or one card type (`(-247)`) went behind the presence gate
that answers that fact without a view, and the rows whose consumer is
the whole view are the freeze design's floor. Two rules fell out: **a
keyword put behind the lane joins `card_has_gate_keyword`'s union** or
the printed leg answers a wrong `false` (the gate's own debug audit is
what catches it), and **price a scope by its `with_frozen_layers` row,
not by the memo's asks** — a scope whose first question is a miss pays
the gather too, and `(-246)`'s ask row said 21.3 M where the scope said
25.5 M.

### The watch-deferral, member-list, block-tax, event-buffer and presence-lane legs — closing state at the `(-244)` tip

Twenty-one engine commits on top of the `(-219)` tip `52b9a743`, each
behaviour-preserving (three-pool outcomes identical, `--bench` counters
identical, golden traces unmoved), from the second of two concurrent
sessions; the other session's `(-221)` refutation sits between them, and
three of this session's own (`(-227)`, `(-232)`, `(-242)`) were reverted
in the hour they were built. (At the `(-241)` tip, where the wall row
below was taken: `fixed` -6.503 %, `cube` -4.619 %, `sealed` -3.730 %.)

```text
  pool     base (-219)      tip (-244)       delta
  fixed      745,162,383      690,547,383   **-7.329 %**
  cube     2,035,552,660    1,928,746,090   **-5.247 %**
  sealed   2,085,024,159    1,992,138,486   **-4.455 %**

  leg      fixed      cube      sealed    what
  (-220)  -0.019 %  -0.163 %  -0.076 %   the CR 732.3 watch fingerprints only on a key repeat
                                         (-0.919 / -1.130 / -0.974 % on its own before (-219) took the land taps)
  (-222)  -0.126 %  -0.134 %  -0.073 %   declare_attackers_banded's two printed-trigger walks over the member list
  (-223)  -0.375 %  -0.316 %  -0.371 %   declare_blockers stops paying a {0} block tax
  (-224)  -0.096 %  -0.069 %  -0.057 %   the combat-damage-to-player listener walk over the member list
  (-225)  -0.118 %  -0.070 %  -0.136 %   the combat damage step writes into the caller's event buffer
  (-226)  -0.229 %  -0.035 %  -0.042 %   do_untap's two remaining static-driven walks behind any_static
  (-227)  +0.241 %  +0.220 %  +0.298 %   a caller-side reserve ahead of the damage step — REFUTED, reverted
  (-229)  -0.174 %  -0.341 %  -0.004 %   the layer pass's effect list by push loops (pins the inliner's coin)
  (-228)  -0.421 %  -0.127 %  -0.052 %   the step-trigger walk over the member list when no static grant is live
  (-230)  -0.784 %  -0.350 %  -0.428 %   the event dispatcher's graveyard leg behind the (widened) graveyard lane
  (-231)  -1.319 %  -0.341 %  -0.492 %   the cast-trigger walker's two zone walks behind their memos (one was quadratic)
  (-232)  +0.077 %  -0.005 %  +0.049 %   the walkers' closures by value — REFUTED, reverted (the shim was not the cost)
  (-233)  -0.911 %  -0.773 %  -0.478 %   a draw-replacement static lane in front of draw_one's eleven walks
  (-234)  -0.391 %  -0.373 %  -0.116 %   an ETB-static lane in front of the ETB multiplier and enters-tapped walks
  (-235)  -0.192 %  -0.165 %  -0.173 %   a damage-replacement static lane in front of six per-damage-event walks
  (-236)  -0.388 %  -0.172 %  -0.192 %   a land-play static lane in front of can_player_play_land's three walks
  (-237)  -0.485 %  -0.525 %  -0.520 %   an any-colour-spend lane in front of the payment relaxation's walk
  (-238)  -0.257 %  -0.140 %  -0.134 %   a hand-size static lane in front of effective_max_hand_size's four walks
  (-239)  -0.198 %  -0.156 %  -0.183 %   an ETB-counter static lane (plus a command-zone term) in front of the two enters-with-counters walkers
  (-240)  -0.134 %  -0.315 %  -0.179 %   a prevention-static lane in front of prevent_static_scan's per-damage-event mask walk
  (-241)  -0.086 %  -0.154 %  -0.089 %   a block-tax lane in front of block_tax_for's per-blocker walk
  (-242)  +1.449 %  +0.711 %  +1.160 %   the dispatcher's grant list inline (SmallVec) with a borrowed filter — REFUTED, reverted
  (-243)  -0.315 %  -0.323 %  -0.318 %   the auto-tapper's activations write into its event buffer instead of returning a Vec each
  (-244)  -0.571 %  -0.337 %  -0.437 %   the dispatcher's empty batch skips the trigger push, the empty drain and their Vec drops
```

```text
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.10 GHz, 4 cores
suite   19,255 / 0 / 5; golden traces in it and unmoved at every leg
clippy  --workspace --exclude crabomination_client --all-targets   clean
        at the (-229), (-238), (-241) and (-243) tips; -p crabomination at every leg
release the release-fast typecheck gate (debug-assertions off): every
        leg's profiling-fast build is that profile, all clean
--bench profiling-fast at every leg through (-241), release at (-243)
        under CRAB_THREAD_CHECK=1: **195,806 decisions / 27.49 turns /
        611.9 per game / 0 stalls** — counters identical to 2003d1cf at
        every leg; determinism ok (all pairs split); peak_rss 21.5 MiB
grid    robustness_grid.sh --pilots on the (-220) tree (the loop guard's
        own audit — `abilarms` on `cube` is the cell that found the
        guard's two defects): ladder 30 cells / 33,120 games, 0
        failures; actor 3 cells, 0 failures; pilots 45 cells, 0
        failures, 24m39s. Re-run on the (-231) tree (the graveyard
        lane's widened predicate, audited by its debug_assert on every
        read, and the member-list walks): ladder 30 / 33,120 games, 0
        failures; actor 3, 0; pilots 45, 0; 26m22s. Re-run on the (-238)
        tree (six presence lanes, each audited by the lane debug_assert
        on every read): ladder 30 / 33,120, 0; actor 3, 0; pilots 45, 0;
        25m14s. Re-run on the (-243) tree (the activation path's
        into-form, whose Err path truncates to a mark): ladder 30 /
        33,120, 0; actor 3, 0; pilots 45, 0 (cold build, ~65 min with
        two A/B builds beside it). (-244) reorders an empty-batch tail
        and is covered by the suite, the traces and the outcome diff.
        No encoder or pool change, no serialized shape change.
wall    bench_ab.py, 24 pairs, the 52b9a743 binary (rebuilt in a
        worktree, 17 min cold) vs the (-241) tip, both profiling-fast
        with default features (mimalloc; 294 `mi_` symbols in each), load
        2.1 at the start: **+9.29 % median games/s** (mean +9.43 %,
        per-pair sd 9.90; A median 452.1, B 493.6) for -6.50 / -4.62 /
        -3.73 % Ir. The lane-write pattern of the (-215) sitting again:
        a device that removes cache-missing board walks reads above its
        Ir share on wall clock.
```

The rules this closing state adds, each from its leg's Log entry:
**price a memo by what compares against it, not by what computes it**
(`(-220)`: computed on every announcement, read on one in fifty); **read
a "validation body" for the walks that are not over the batch**
(`(-222)`: two board walks priced as batch scans); **when two sides of
one mechanic are written twice, diff the gates, not the bodies**
(`(-223)`: the attack side had `> 0`, the block side never did); **a
buffer recycled per state is not recycled across probe clones**
(`(-225)`: the reserve stayed, the append and free went); **`Vec::
reserve(n)` is `n` beyond `len`, and a 32-slot event buffer costs the
same ~1,400 Ir to obtain by `malloc` or `realloc`** (`(-227)`); **a
generic adapter on a 400 k-call path is a coin the inliner flips per
build — write the inlined shape down, and when a total contradicts the
device's rows, diff the two self tables** (`(-229)`, found through
`(-228)`'s first reading); **a self row's line profile names where the
instructions are; a grep of the read sites names which of them a memo
already answers** (`(-230)`/`(-231)`: three whole-zone walks with the
lane beside them, found by listing every `triggered_abilities` read);
**when a function asks one zone N presence questions, the lane's
predicate is the union** (`(-233)`: twelve `matches!` arms, one lane,
eleven walks; `(-234)`..`(-238)` the same shape five more times, from a
grep of the `static_abilities` reads ranked by the enclosing function's
self cost); **a `call_mut` shim on a closure whose body is a dozen loads
is not `(-98)`'s 18.8 M** (`(-232)`, reverted); **inline storage in a
struct returned by value is a memcpy per call, not per allocation**
(`(-242)`, reverted: 12 M of `memcpy` on every dispatch against 4.9 M of
allocator on a third of them); **an inlined `Vec::push` leaves the
dump's call-site position at `vec/mod.rs:*`, so a `push_mut` edge names
the function and the body has to be read for the two-push `Vec`**
(`(-243)`); **rank a collect row by its `__rust_alloc` count, not its
inclusive Ir — the Ir is the iterator's body** (the selector collect,
priced at 4.1 M and worth 0.2 M).

### The target gate, cold-group, walker-lane and watch legs — closing state at the `(-219)` tip

Four engine commits on top of the `(-215)`+fix tip `999da717`, each
behaviour-preserving (three-pool stdout identical, `--bench` counters
identical, golden traces unmoved), one refutation reverted in the same
hour (`(-217)`'s first cut), plus a client fix (the Corruption counter's
two missing match arms; `cargo clippy -p crabomination_client` clean
again).

```text
  pool     base 999da717     tip (-219)       delta
  fixed      763,717,868      745,162,927   **-2.430 %**
  cube     2,090,168,791    2,035,554,686   **-2.613 %**
  sealed   2,120,435,808    2,085,022,232   **-1.670 %**

  leg      fixed      cube      sealed    what
  (-216)  -0.324 %  -0.245 %  -0.181 %   a presence gate on the target in check_target_legality
  (-217)  -0.832 %  -1.228 %  -0.351 %   the two per-death registries leave the cold group
                                         (the device is -0.30 / -0.25 / -0.20 %; the rest is the build's inlining shift)
  (-218)  -0.344 %  -0.158 %  -0.208 %   step / combat-damage walkers behind zone lanes, the filter in place
  (-219)  -0.951 %  -1.004 %  -0.938 %   the CR 732.3 watch behind the land-tap fast path
```

```text
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.10 GHz, 4 cores
suite   19,255 / 0 / 5 (+1 zone::tests::pile_encoded_lane_follows_the_
        instance_flag); golden traces in it and unmoved at every leg
clippy  --workspace --exclude crabomination_client --all-targets   clean
        at every leg; -p crabomination_client clean after the fix
release the release-fast typecheck gate (debug-assertions off): every
        leg's profiling-fast build is that profile, all clean
--bench profiling-fast at the (-219) tip: **195,806 decisions / 27.49
        turns / 611.9 per game / 0 stalls** — counters identical to
        2003d1cf at every leg; determinism ok (all pairs split);
        peak_rss 20.6-21.4 MiB across sittings (it moves 0.5 MiB run
        to run on one binary; not a byte-identity column)
grid    robustness_grid.sh --pilots on the (-219) tree: ladder 30 cells
        / 33,120 games, 0 failures; actor 3 cells, 0 failures; pilots
        45 cells, 0 failures. The (-216) gate's debug_assert (a view
        recomputed on every gated miss), the (-218) pile lane's audit
        and every earlier lane audit ran under debug-assertions across
        it. No encoder or pool change; the serialized shape is
        unchanged (TurnDeaths is flattened where its two fields were).
wall    bench_ab.py, 24 pairs, the 999da717 binary (rebuilt in a
        worktree, 16m50s cold) vs the tip, profiling-fast, fixed. Two
        sittings, quoted both: **+0.16 % median / -0.30 % mean** (sd
        3.64; taken with the grid's tail still on the box, load 4.5)
        and **+1.65 % median / +2.15 % mean** (sd 4.59; load 2.1) for
        -2.43 % Ir. Inside the instrument's noise band either way —
        the standing rule says quote Ir for these, and the Ir is the
        number.
```

The rules this closing state adds, each from its leg's Log entry:
**an unshare is paid by the first cold write of the action, not by the
field** (`(-217)`, whose first cut only moved the unshare one insert
down the path); **quote a device at its own rows when the build moved
the inliner** (`(-217)`, two to five times its rows); **a zone lane's
predicate is the scope, not the caller** (`(-218)` reused `(-210)`'s
lane for a second walker); and **price every caller-side wrapper of
the function a fast path shortcuts** (`(-219)`: `(-204)` was measured
through a prologue that kept paying 900 Ir a tap).

### The land-tap, lane-word and lane-write legs — closing state at the `(-215)` tip

Twelve engine commits on top of `(-203)`: ten perf legs, each
behaviour-preserving (three-pool stdout identical, `--bench`
byte-identical, golden traces unmoved), one rules fix priced as a cost
(`(-206)`), one refutation reverted in the same hour (`(-209)`).

```text
  pool     base 62a4e20b     tip (-215)+fix    delta
  fixed      801,539,915      763,717,868   **-4.718 %**
  cube     2,200,107,698    2,090,168,791   **-4.997 %**
  sealed   2,211,363,961    2,120,435,808   **-4.112 %**

  leg      fixed      cube      sealed    what
  (-204)  -1.496 %  -1.509 %  -1.623 %   the printed land tap settled by inspection
  (-205)  -0.032 %  -0.156 %  -0.114 %   the AddMana arm's Contamination walk behind the lane
  (-206)  +0.048 %  +0.197 %  +0.054 %   CR 305.7 fix: stripped printed mana abilities refuse
  (-207)  -0.764 %  -0.912 %  -0.622 %   card-type lane; the lane word widened to 64 bits
  (-208)  -0.092 %  -0.058 %  -0.068 %   ContinuousEffects, a fold of modification families
  (-209)  +0.101 %  -0.088 %  +0.073 %   strip lane — REFUTED, reverted
  (-210)  -0.618 %  -0.468 %  -0.430 %   graveyard lane in front of the combat-damage dispatch
  (-211)  -0.026 %  -0.031 %  -0.015 %   two standing-rule reorders in the same dispatch
  (-212)  -0.440 %  -0.469 %  -0.334 %   membership writes demote only the lanes they can change
  (-213)  -1.158 %  -0.847 %  -0.887 %   a membership write answers each lane off the one card it moved
  (-214)  -0.235 %  -0.171 %  -0.148 %   the member lists kept exact through membership writes
  (-215)  +0.003 %  -0.678 %  +0.007 %   the dispatch scan visits its member list
  fix     +0.001 %  +0.001 %  +0.001 %   (-214)'s index-63 removal shift (the closing grid's find)
```

```text
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores
suite   19,253 / 0 / 5 (+11 core_rules::land_tap_fast_path, +2 modern
        CR 305.7 regressions); golden traces in it and unmoved at every leg
clippy  --workspace --exclude crabomination_client --all-targets   clean
        at every leg
release the release-fast typecheck gate (debug-assertions off): every
        leg's profiling-fast build is that profile, all clean
--bench profiling-fast at the (-215) tip: **195,806 decisions / 27.49
        turns / 611.9 per game / 0 stalls** — byte-identical to 2003d1cf
        at every leg; determinism ok (all pairs split); peak_rss 20.6 MiB
grid    robustness_grid.sh --wide --pilots, built between the (-211) and
        (-213) edits: ladder 52 cells / 301,600 games, 0 cell failures,
        undecided cap 4 (the Beacon of Immortality board, seeds 53/73 —
        ENGINE_BACKLOG's closed stall lead) / stuck 0 / draw 12; actor
        2 cells, 0 failures; pilots 45 cells, 0 failures. Every lane
        audit, the fold audit and the fast path's debug tally ran under
        debug-assertions across it. The default-size grid with --pilots
        on the (-215) tree found (-214)'s index-63 shift in two cells
        (cube 23, sos 11); re-run on the fixed tree: ladder 30 cells /
        33,120 games, 0 failures, cap 0 / stuck 0 / draw 0; actor 3
        cells, 0 failures; pilots 45 cells, 0 failures. No encoder or
        pool change; ContinuousEffects and the lane word are not
        serialized.
wall    bench_ab.py, 24 pairs, the 62a4e20b binary (rebuilt in a worktree)
        vs the tip, profiling-fast, fixed. At the (-211) tip: **+1.87 %
        median games/s** (mean +2.02 %, per-pair sd 3.57; A 317.2, B
        325.5) for -2.95 % Ir — the (-198) ratio again. At the (-215)
        tip: **+5.16 % median games/s** (mean +5.23 %, per-pair sd 6.20;
        A median 380.2, B 399.7) for -4.72 % Ir — the lane-write legs
        (-212)..(-215) read above their Ir share, which fits a device
        that removes cache-missing board walks rather than arithmetic.
```

### The cheap-clone, held-views and death-lane legs — closing state at `5b50323f`

Four more engine commits on top of `(-199)`, each behaviour-preserving
(three-pool stdout identical, `--bench` byte-identical, golden traces
unmoved). Two of them found their priced row belonged to someone else and
said so (Log); the third is the `(-194)` shape again; the fourth came off
the re-profile at `966289ae` and is the `(-197)` lane shape on the death
path.

```text
  pool     base 4bd4fc1b     tip (-203)        delta       run (from 2003d1cf)
  fixed      808,660,509      801,539,784   **-0.881 %**   **-1.437 %**
  cube     2,222,094,501    2,200,107,512   **-0.989 %**   **-1.627 %**
  sealed   2,228,991,395    2,211,369,741   **-0.791 %**   **-1.141 %**

  leg      fixed      cube      sealed    what
  (-200)  -0.360 %  -0.294 %  -0.346 %   OftenEmpty on CardData/CounterBag, GameState::clone guards
  (-201)  -0.154 %  -0.113 %  -0.143 %   OftenEmpty on PlayerData's seven lists
  (-202)  -0.070 %  -0.127 %  -0.037 %   resolve_combat's protection asks over held views
  (-203)  -0.299 %  -0.459 %  -0.268 %   death-redirect lane in front of the death path's four walks
```

```text
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores
suite   19,240 / 0 / 5 (+1: oftenempty's unit test); golden traces in
        it and unmoved at every leg
clippy  --workspace --exclude crabomination_client --all-targets   clean
        at (-201); the engine lib re-checked clean at (-202) and (-203)
release the release-fast typecheck gate (debug-assertions off)   clean
        at the (-202) and (-203) tips
--bench profiling-fast at every leg: **195,806 decisions / 27.49 turns /
        611.9 per game / 0 stalls** — byte-identical to 2003d1cf;
        determinism ok; thread_determinism ok; peak_rss 21.3-21.4 MiB
grid    not re-run: no encoder or pool change. `OftenEmpty` is
        `#[serde(transparent)]`, so the wire format is unchanged.
```

### The grants-nothing pass — closing state at `4bd4fc1b`

One engine commit, behaviour-preserving (three-pool stdout identical,
`--bench` byte-identical, golden traces unmoved), from the base dumps'
*caller* tables again: `granted_abilities_of_inner`'s 96,734 calls were
one `cg_edges.py --callers` away from "the gate refuses rows for a grant
aimed at something else".

```text
  pool     base 2003d1cf     tip 4bd4fc1b      delta
  fixed      813,222,102      808,660,509   **-0.561 %**
  cube     2,236,502,758    2,222,094,501   **-0.644 %**
  sealed   2,236,900,247    2,228,991,395   **-0.354 %**

  (base re-taken this run: within 0.0003 % of the 2003d1cf readings)
```

```text
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores
suite   19,239 / 0 / 5; golden traces in it and unmoved
clippy  --workspace --exclude crabomination_client --all-targets   clean
release the release-fast typecheck gate (debug-assertions off)   clean
--bench profiling-fast at 4bd4fc1b: **195,806 decisions / 27.49 turns /
        611.9 per game / 0 stalls** — byte-identical to 2003d1cf;
        determinism ok (all pairs split); thread_determinism ok (3 vs 1
        threads identical); peak_rss 21.3 MiB, bin 219,701,352 B
        (`--no-default-features`)
grid    not re-run: no encoder or pool change; the widened gate is audited
        by `granted_abilities_of`'s `debug_assert!` on every accept, which
        the suite runs with debug-assertions on.
```

### The dispatcher-mask pass — closing state at `2003d1cf`

Five engine commits, each behaviour-preserving (three-pool stdout
identical, `--bench` byte-identical, golden traces unmoved at every
step), all found by ranking a hot row's *callers* on a
`--separate-callers=3` dump rather than reading its body by line.

```text
  pool     base 0e9bdaa4     tip 2003d1cf      cumulative
  fixed      837,759,772      813,220,278   **-2.929 %**
  cube     2,335,851,736    2,236,499,052   **-4.253 %**
  sealed   2,345,940,541    2,236,898,811   **-4.648 %**

  leg      fixed      cube      sealed    what
  (-194)  -0.313 %  -0.787 %  -0.504 %   the block planner reads the views it holds
  (-195)  -0.251 %  -1.096 %  -1.339 %   batch kind mask ahead of the dispatcher's pair loop
  (-196)  -0.977 %  -1.270 %  -1.669 %   per-card printed-trigger kind fold gates its walk
  (-197)  -0.568 %  -0.684 %  -0.562 %   mana-static lane ahead of the land tap's three walks
  (-198)  -0.854 %  -0.488 %  -0.658 %   per-definition mana summary is the auto-tapper's row

  bench_ab.py, 24 pairs, base 0e9bdaa4 vs the (-198) tip (--bench =
  fixed, -2.93 % Ir): **median +1.89 % games/s, paired mean +1.81 %,
  sd 3.55** — the wall clock sees it, at the ratio the instrument's
  calibration predicts (a -2.0 % Ir read +2.6 % at the 116th pass).
  At the (-196) tip (-1.5 % Ir) 16 pairs read +0.67 %, inside noise.
```

```text
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores
suite   19,239 / 0 / 5 (+1: events::tests::variant_and_payload_halves_
        agree_with_the_reference_table); golden traces in it and unmoved
clippy  --workspace --exclude crabomination_client --all-targets   clean
        (one type_complexity on (-194)'s blocker tuple, now `BlockerFacts`)
release the release-fast typecheck gate (debug-assertions off)   clean
--bench profiling-fast at d3d28c18: **195,806 decisions / 27.49 turns /
        611.9 per game / 0 stalls** — byte-identical to df27df7e;
        determinism ok (all pairs split); peak_rss 21.4 MiB,
        bin 219,679,016 B (`--no-default-features`)
grid    `scripts/robustness_grid.sh --no-actor` at 2003d1cf (and at
        5bf4e81c and d3d28c18 before it): ladder **30 cells / 33,120
        games, 0 failures**, cap 0 / stuck 0 / draw 0, 7 assertion
        strings in the binary — the `trigger_kind_fold`, `mana_summary`,
        dispatch-memo and mana-static lane staleness `debug_assert!`s
        live on every cell. Actor leg not run: no encoder or pool change
        this run.
audits  audit_incomplete --structural-only 21,795 / 0 to review (Elite
        Interceptor reviewed); audit_stubs 0 flagged;
        audit_oracle_verbs.py 70 -> 61 rows, every one filed
```

### The attack-search census pass — closing state at `e725e5c2`

No engine change — the default profile is byte-identical to `df27df7e`
(`--bench` 195,806 / 27.49 / 611.9 / 0 stalls, golden traces 7/7 unmoved).
The run read `(-21)`'s never-read half with a new instrument
(`CRAB_ATTACK_CENSUS`) and priced the one device it suggested (`atk-open`,
the open-board skip): -1.3/-1.8 % on cube/sealed but -0.1 pt on a 96 k-game
sealed ladder, so filed as an opt-in pilot, not adopted (Log, `e725e5c2`).

```text
suite   19,238 / 0 / 5 (+1: attack_skip_open_only_shortcuts_a_blockerless_board)
clippy  --workspace --exclude crabomination_client --all-targets   clean
release the release-fast typecheck gate (debug-assertions off)   clean
--bench profiling-fast: 195,806 / 27.49 / 611.9 / 0 stalls, determinism ok,
        byte-identical to df27df7e (default profile unchanged)
audits  audit_incomplete --structural-only 21,795 / 0 to review; audit_stubs
        0; audit_variant_coverage 0 dead capability / 2 dead primitive
        (AddRadCounters, GrantCastBackFromGraveyard — unbuilt, pre-existing);
        audit_panics 0 bare; audit_doc_drift 0; audit_bottom_random 0/0
```

The robustness grid was not re-run: the default engine is byte-identical to
`df27df7e`, whose `--wide` grid (301,600 games, cap 4 = the Beacon board)
still describes it.

### The oracle-verb close-out — closing state at `df27df7e`

No perf leg: three fresh dumps at `a198daf3` read within 0.07 % of
`(-192)` and nothing priced at a build (candidates, top entry). The run
was twenty-one shipped-card defects across six oracle-verb classes, with
three engine bits that ride along (`CounterType::Corruption`,
`MillThenToHand` on `LastMoved`, `SameNameAsExiledWithSource` reading the
until-leaves link). **The catalog change moves `cube` / `sealed` play, so
the next A/B takes a fresh base at this tip or later.**

```text
  pool     a198daf3 (this run's base, pre-catalog)   vs (-192)
  fixed      833,934,847                              +0.070 %
  cube     2,316,705,788                              +0.061 %
  sealed   2,344,388,085                              +0.071 %
```

```text
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores
suite   19,237 / 0 / 5 (19,220 at a198daf3; +12 and +5 card tests, four
        Geyadrone tests that pinned an invented card rewritten as three);
        golden traces in it and unmoved at both commits
clippy  --workspace --exclude crabomination_client --all-targets   clean
release the release-fast typecheck gate (debug-assertions off)   clean
--bench profiling-fast at df27df7e: **195,806 decisions / 27.49 turns /
        611.9 per game / 0 stalls (cap 0 / stuck 0 / draw 0)** —
        byte-identical to the invariant (the fixed pool carries none of
        the 21 cards); determinism ok (3 vs 1 threads identical);
        peak_rss 19.3 MiB, bin 219,404,200 B (`--no-default-features`)
grid    `scripts/robustness_grid.sh` at `df27df7e`: ladder **30 cells /
        33,120 games, 0 failures**, cap 0 / stuck 0 / draw 0; actor leg
        3 seeds x 600 games clean (99-126 games/s under `overflow` +
        `debug-assertions`, 7 assertion strings in the binary); **pilots
        45 / 45 clean** at the same tip (`--no-build --no-actor --pilots`).
        `--wide` (`--no-build --no-actor`) at `df27df7e`: ladder **52 cells /
        301,600 games, 0 panic / 0 assertion / 0 overflow**, cap 4 / stuck 0
        / draw 12 — the four caps are seeds 53 and 73 on `all`, twin
        `i32::MAX` life totals at turns 2,159 and 2,490 (`CRAB_CAP_DIAG=1`),
        the Beacon board ENGINE_BACKLOG closes, unchanged from `094b361a`;
        pilots 45 / 45 clean again. The script's `failures=1` is that cap
        count. Wide actor leg (`--no-build --actor-only --wide`): **2 x
        30,000 games clean at 129.7 / 129.6 games/s** under `overflow` +
        `debug-assertions`, 231 s each.
audits  `audit_incomplete --structural-only` 21,795 cards / 0 to review
        (Elite Interceptor reviewed); `audit_stubs` 0 flagged;
        `audit_oracle_verbs.py` 106 -> **70** rows, every one filed.
```

### The presence-flag pass — closing state at `(-192)`

Five candidates, one base (`fb120400`, the `(-185)` tip plus trackers),
one device: the keyword presence gates asked five questions per call and
a probe with each leg out of line priced them (`(-188)`'s Log entry);
each leg then went behind the cheapest thing that could hold its answer
— a definition-only battlefield lane, a member list on the zone, two
state flags exact at cleanup, and no hoist at all. A sixth candidate,
`(-170)(b)`'s pool batching, built and reverted at +0.9-1.3 %.

```text
  pool     base fb120400     tip (-192)       cumulative
  fixed      847,238,714      833,352,202   **-1.6391 %**
  cube     2,353,123,287    2,315,286,224   **-1.6079 %**
  sealed   2,362,293,751    2,342,732,076   **-0.8281 %**

  leg        fixed      cube     sealed   what
  (-188)   -0.233 %  -0.133 %  -0.176 %  printed leg on LANE_GATE_KEYWORD
  (-189)   -0.449 %  -0.879 %  +0.001 %  grant lane -> grant member list (+has_anthem #[inline])
  6717b648 +0.000 %  -0.002 %  -0.000 %  three eot grants take a CR 613.7 timestamp (a fix)
  (-190)   -0.676 %  -0.395 %  -0.414 %  board_instance_keywords flag for the instance leg
  (-191)   -0.110 %  -0.099 %  -0.103 %  offboard_keyword_grants flag for the command/emblem legs
  (-192)   -0.181 %  -0.108 %  -0.138 %  no per-ask tag list
  (-193)   +1.288 %  +0.920 %  +1.097 %  REVERTED: one pool access per scope
```

```text
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.10 GHz, 4 cores
suite   19,217 / 0 / 5 at every leg (19,216 at (-188); (-189) adds one
        zone test); golden traces in it and unmoved at every leg
clippy  --workspace --exclude crabomination_client --all-targets   clean
release the profiling-fast build of each leg (release-fast opts,
        debug-assertions off) is the typecheck gate here
--bench profiling-fast at (-192): **195,806 decisions / 27.49 turns /
        611.9 per game / 0 stalls (cap 0 / stuck 0 / draw 0)** —
        byte-identical to the invariant; determinism ok (3 vs 1 threads
        identical); peak_rss 19.3 MiB, bin 219,372,936 B
        (`--no-default-features`)
stalls  three-pool six-game stdout identical to the base's at every leg
        but the wall-clock line
grid    `scripts/robustness_grid.sh` at `a9ce7489` (the `(-192)` engine plus
        the CR 400.7 fix): ladder **30 cells / 33,120 games, 0 failures**,
        cap 0 / stuck 0 / draw 0; actor leg 3 seeds x 600 games clean
        (151-172 games/s under `overflow` + `debug-assertions`). The first
        run at `(-192)` itself tripped the `(-190)` audit on six cells —
        the CR 400.7 defect (Log, top) — so the grid is what qualified
        this pass's three new `debug_assert!`s. `--pilots` / `--wide` last
        ran at the `(-185)` engine.
```

### Actor scaling — `(-52)` is right, its RSS-per-row is not

**A third session measured actor scaling this afternoon, having read the same
seed-list line, and `(-118)` above is the process rule.** The shape it found
is confirmed on a third box (`release`, `--games 3000 --steps 1 --seed 7`,
three reps, 4-core 2.10 GHz Xeon): 90.4 / 193.4 / 373.4 / 368.8 games/s at
1 / 2 / 4 / 8 actors — **1.00 / 2.14 / 4.13 / 4.08x**, flat past saturation,
per-actor 90.4 / 96.7 / 93.4 / 46.1. Nothing new; do not re-run it a fourth
time. `scripts/actor_scaling.sh` is the recipe so that the next reader runs it
in one line instead of re-deriving it.

**What is new is a correction, and it is an order of magnitude.** `(-52)`
reads "the replay window costs ~1.3 KiB a row" off a 600-game run — and 600
games push ~58 k rows, so its `--window 250000` column measured a window that
was **never filled**. Filled, on a 3,000-game run (~288 k rows pushed):

```text
                        peak RSS (VmHWM)   rows actually held
  --window 250000          3,170 MiB            250,000
  --window  25000            632 MiB             25,000
  1 / 2 / 4 / 6 / 8 actors, --window 250000:
      3,133 / 3,145 / 3,168 / 3,189 / 3,216 MiB
```

**~10.4 KiB a row over a ~370 MiB floor, not 1.3 KiB** — and flat in the
actor count, which is `(-52)`'s conclusion and stands. A box sized off its
805 MiB figure OOMs the moment the window fills, which on these rates is
about forty seconds in. **A memory figure taken before the bounded thing is
bounded is not a measurement of it**; run the window to its cap or do not
quote a per-row cost.

**And `(-52)`'s caution about `--bench --threads N` is confirmed by giving it
the longer workload it asked for.** `--a gang --b gang --games 400 --decks
fixed` (1,600 games, 11.8 s at one thread) reads 1 / 2 / 3 / 4 threads at
11.8 / 6.1 / 4.1 / 3.1 s — **1.93 / 2.88 / 3.81x, 95-97 % of ideal**, against
`--bench`'s 83 %. The game loop has no contention either; `--bench` is too
short to say so.


## Log

Entries `(-199)` and older are in `PERF_ARCHIVE.md`, verbatim.

### `(-252)` REFUTED — `-C llvm-args=-inline-threshold=500` on `release-fast`: paired wall clock **+0.02 %** (flat), Ir -1.69 %, I1 misses -0.13 %, `.text` -28 %

```text
  wall clock, scripts/bench_ab.py, release-fast (+abort) vs the same with the flag, 16 pairs
    A median 525.55   B median 529.94   paired B/A median +0.02 %  mean -0.11 %  sd 3.14
  cachegrind, cube (mimalloc on both sides)
    Ir            1,692,597,648 -> 1,663,979,447   -1.69 %
    I1 misses        70,175,375 ->    70,082,275   -0.13 %
    D1 misses        35,231,226 ->    35,878,030   +1.84 %
    mispredicts      34,470,843 ->    34,343,504   -0.37 %  (indirect 4.80 M -> 4.54 M)
    .text          112.7 MB -> 81.1 MB (-28 %); bin_bytes 125,179,032 -> 92,734,568
  --bench counters identical; determinism ok
  built with RUSTFLAGS="-C llvm-args=-inline-threshold=500" CARGO_TARGET_DIR=target-inl
```

`(-251)`'s converse: if less inlining loses 8 %, does more win? No.
The threshold doubles the inliner's budget, and what that inlines is
mostly the catalog — thousands of small definition constructors fold
into their one caller and are no longer emitted, which is where the
28 % of `.text` goes — while the engine's hot working set, already
inlined at level 3, does not change: I1 misses move 0.13 %, the Ir
saving is the call frames of the cold constructors, and the wall clock
does not move at all. **Total code size is not the front-end cost; the
hot working set is**, and the flag does not touch it. One build, one
bench, one dump; not landed (it would be a global `RUSTFLAGS` lever
anyway, `PGO`'s awkward shape without its win). Do not rebuild.

### `(-251)` REFUTED — `opt-level = 2` on `release-fast`: paired wall clock **-8.13 %**, Ir +5.0 %, I1 misses +9.0 %

```text
  wall clock, scripts/bench_ab.py, release-fast (+abort) vs the same at opt-level 2, 16 pairs
    A median 532.01   B median 493.05   paired B/A median -8.13 %  mean -7.76 %  sd 2.70
  cachegrind, cube (mimalloc on both sides)
    Ir            1,692,599,315 -> 1,777,484,725   +5.01 %
    I1 misses        70,175,391 ->    76,493,739   +9.00 %
    D1 misses        35,218,791 ->    35,480,323   +0.74 %
    mispredicts      34,470,805 ->    35,260,148   +2.29 %
    bin_bytes       125,179,032 ->   135,886,408   +8.6 %  (larger, not smaller)
  --bench counters identical; determinism ok
```

The obvious follow-up to `(-250)`'s "front-end-bound, so layout is the
lever": if level 3's inlining and unrolling buy width the core does not
use, level 2 should be smaller and miss less. It is neither — the
binary is 8.6 % *larger* and misses L1i 9 % more, because what level 3
removes here is not width but *calls*: the engine's hot paths are short
generic adapters and accessors that level 3 inlines and level 2 leaves
as call-return pairs, each a jump the front end has to fetch through.
One build, one bench, reverted. **Do not rebuild**; the direction that
is still open is the opposite one — more inlining where a hot small
callee is left out of line (`cg_frames.py` names them) — and PGO, which
is the same lever with a profile behind it.

### `(-250)` TAKEN — `panic = "abort"` on every optimized profile: paired wall clock **+4.45 % / +5.51 %**, Ir `sealed` -3.753 % / `fixed` -3.686 % / `cube` -3.142 %

```text
  wall clock, scripts/bench_ab.py, release-fast bot_ladder (mimalloc), two independent 16-pair runs
    run 1   A median 490.37   B median 517.78   paired B/A median +4.45 %  mean +4.78 %  sd 4.36
    run 2   A median 490.59   B median 518.84   paired B/A median +5.51 %  mean +4.55 %  sd 3.61
  cachegrind, release-fast bot_ladder, cube (a relative read — mimalloc on both sides)
    Ir            1,753,456,679 -> 1,692,598,374   -3.47 %
    I1 misses        74,217,599 ->    70,175,409   -5.45 %
    D1 misses        35,977,779 ->    35,205,205   -2.15 %
    mispredicts      34,403,011 ->    34,470,898   +0.20 %  (flat)
    bin_bytes       144,142,368 ->   125,179,032  -13.2 %
  Ir, profiling-fast --no-default-features, three pools against the (-249) tip
    cube    1,876,460,069 -> 1,817,493,748   **-3.142 %**
    fixed     681,439,653 ->   656,319,384   **-3.686 %**
    sealed  1,959,940,755 -> 1,886,392,273   **-3.753 %**
  three-pool outcomes identical; --bench counters identical (195,806 / 27.49 / 611.9 / 0);
  determinism ok; thread_determinism ok (3 vs 1); suite untouched (test harnesses always unwind)
  where the Ir went (cachegrind by function, cube): __memcpy -15.1 M, perform_action_inner -11.2 M,
    dispatch_triggers_for_events -9.2 M, IntoIter::drop -7.1 M, Vec::clone -4.9 M, Vec::push_mut
    -3.6 M (gone), clone_from_ref_in -3.0 M; a few drop-glue and closure rows rise (inlining moved)
```

The one build lever nobody had pulled. TODO's item 0 says "THE BUILD IS
THE LEVER" and lists PGO (-24 %), LTO (0.917) and `target-cpu=native`
(flat); unwinding was never on the list. Nothing in the tree calls
`catch_unwind` and no worker's `join` recovers a panicked thread, so a
panic already ended a run — with `abort` it ends it after printing the
same message, exit 134 instead of 101, which the robustness grid's
`rc` check reads as the same failure. What goes away is real code on
the hot path, not just cold landing pads: every call LLVM could not
mark `nounwind` kept its cleanup edge, drop flags and the copies that
feed them, which is why `__memcpy` and `IntoIter::drop` are the largest
movers and why Ir moves 3.5 % where the landing pads themselves never
executed. **Found off the cachegrind axis**: the I1-miss table said the
program is front-end-bound and layout is the lever, and the cheapest
layout change in the toolchain is deleting the unwind tables.

**The new Ir base.** Every three-pool number from here is against the
`(-250)` binaries; the `(-249)` closing state is the last on the old
base, and a reading across the boundary is not an A/B.

### `(-249)` TAKEN — an untap-static lane in front of `do_untap`'s eight static walks: `cube` -0.399 % / `sealed` -0.362 % / `fixed` -0.055 %

```text
  pool    base (-248)       (-249)          delta
  cube    1,883,973,537   1,876,460,069   **-0.399 %**
  fixed     681,812,730     681,439,653   **-0.055 %**
  sealed  1,967,054,280   1,959,940,755   **-0.362 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  do_untap (cube)   2,834 calls  23,708,407 -> 16,362,427 Ir inclusive  (8,366 -> 5,773 a call)
  do_untap (fixed)  1,896 calls   9,918,205 ->  9,514,657               (5,231 -> 5,018)
```

The tenth presence lane, off this run's fresh `--separate-callers=3`
re-read of the plain `cube` self table at the `(-248)` tip: `do_untap`
had never been on any table (11.7 M self, 0.62 %; 23.7 M inclusive),
because the untap step reads as once a turn — and it *is* once a turn on
every simulation clone too, 2,834 times a six-game `cube` run against
~160 real turns. `(-226)` had put six of its walks behind `any_static`,
"any permanent carries a static ability at all", which is `true` on
nearly every `cube` board (a cube creature usually prints one) and so
walked all of them anyway; the question the walks actually ask is
whether any static is one of the eleven untap `StaticEffect`s, and that
is a definition-only predicate the lane device already covers.
`card_has_untap_static` peels the five `While*` wrappers
unconditionally where the consumers' `active_static` peels them by
condition — the sound direction — and a `debug_assert!` in `do_untap`
recomputes the consumers' arm list through `active_static` on every
clear gate, so the suite and the robustness grid audit the enumeration.
The command-zone half stays a walk of two usually-empty lists, as
`(-239)`'s did. `fixed` was already clear under the old gate on most
boards, so it moves only by the walk that asked it.

**What the step still costs with the gate clear, on `fixed`: ~5,000 Ir
a call** — `do_phasing`, the untap loop's per-card empty-set lookups,
the lock fold, the `MayChooseNotToUntap` walk, the flag-reset walk and
one `vec![p]` a call (1,896 allocations, ~0.03 %); nothing above 1 M.

### `(-248)` TAKEN — the two targeting-time keyword reads behind `card_keyword_possible`: `sealed` -0.240 % / `cube` -0.224 % / `fixed` -0.167 %

```text
  pool    base (-247)       (-248)          delta
  cube    1,888,198,355   1,883,972,930   **-0.224 %**
  fixed     682,953,228     681,813,326   **-0.167 %**
  sealed  1,971,796,448   1,967,055,576   **-0.240 %**
  three-pool outcomes identical; --bench counters identical
  cube:  push_first_targeting_counter's view   884 asks / 2.7 M  ->  4 asks (the gate: can_grant_keyword 1,024 / 75 k, card_has_anthem 484 / 7 k)
         has_hostile_ward's view             6,494 asks / 1.7 M  ->  5,438 (the memo hits stay; the 1,056 out-of-scope gathers go)
```

The `(-245)` re-read's `push_ward_triggers_for_targets` row (884 asks
at ~3,000 Ir — gathers, out of any scope). The Ward read itself was
already behind `card_keyword_possible` (`(-216)`'s shape); the row was
`push_first_targeting_counter` one line above it, asking a whole view
of every targeted permanent on every cast for the Glasskite cycle's
`CounterFirstTargetingEachTurn`, which four printings carry. Same gate.
`has_hostile_ward` — the bot's auto-target ranking, 6,494 asks — is the
other reader that consumes one keyword; it takes the gate behind
`layers_memoized`, as `check_target_legality_inner` does, so a scope
that has already gathered keeps its 124-Ir memo hit and one that has
not skips the gather. Every pool moves the same ~0.2 %: every pool
casts targeted spells, and the probes cast them through the same path.

**When a gate is on one of two sibling reads, the other is the row.**
The Ward push was gated at `(-216)` and its ungated sibling sat one
call above it in the same function for thirty passes; the by-caller
table charged the asks to the enclosing function, which read as "the
gated one", and only the callee table (`cg_edges.py --callees`) said
which of the two lines was still asking.

### `(-247)` TAKEN — `permanent_is_creature` reads the printed type line behind the card-type presence gate: `cube` -0.421 % / `fixed` -0.044 % / `sealed` -0.025 %

```text
  pool    base (-246)       (-247)          delta
  cube    1,907,672,245   1,899,633,344   **-0.421 %**
  fixed     689,420,559     689,113,878   **-0.044 %**
  sealed  1,986,793,457   1,986,301,307   **-0.025 %**
  three-pool outcomes identical
  permanent_is_creature, cube:  computed_permanent_hinted  2,156 asks / 7.96 M  ->  card_type_change_unscoped  2,144 reads / 0.08 M
  its callers:                  the SBA sweep's CR 704.5n collect 1,174 + a second collect 236, destroy_permanent 464,
                                activate_ability_inner 154, sacrifice_one 128
```

The `(-245)` re-read's `permanent_is_creature` row: 2,156 asks at
~4,200 Ir each, i.e. every one a whole gather plus a layer pass,
because the caller is the SBA sweep (`&mut self`, no scope) checking
whether each attached Equipment's host is still a creature (CR
704.5n). `compute_permanent_pass` seeds the type line from three
things — the definition, the CR 702.103d bestowed rewrite, and the
layer-4 `AddCardType` / `RemoveCardType` / `SetCardTypes`
modifications — and `card_type_change_in_scope` is the presence gate
for the third, so `!bestowed && !gate` makes the printed line the
computed one. The same device `activate_ability_inner` already uses
for its `is_creature` read (`(-204)`'s leg), applied at the helper so
its five callers get it at once. `fixed` and `sealed` are flat: the
asks are attached Equipment, which those pools rarely put on the board.

**Three legs, one read.** `(-245)`, `(-246)` and this one all came off
ranking `computed_permanent_hinted`'s asks by caller and asking what
each caller *consumed* of the view: one keyword, three keywords, one
card type. A presence gate answers each without a view. The rows left
in that table consume the whole view (the block planner's per-blocker
and per-attacker facts, the material eval) and are the freeze design's
floor.

### `(-246)` TAKEN — `declare_blockers`' Flanking / Bushido / Rampage keyword views behind the keyword presence gate: `cube` -1.470 % / `fixed` -0.843 % / `sealed` -0.831 %

```text
  pool    base (-245)       (-246)          delta
  cube    1,936,134,211   1,907,672,245   **-1.470 %**
  fixed     695,281,092     689,420,559   **-0.843 %**
  sealed  2,003,446,075   1,986,793,457   **-0.831 %**
  three-pool outcomes identical
  under declare_blockers, cube:  with_frozen_layers        10,072 -> 5,036 calls   45.29 M -> 19.81 M
                                 board_keyword_matching         0 -> 5,036 calls    0     ->  0.98 M
                                 compute_permanents (scope 1)  5,036 unchanged      30.28 M -> 30.25 M
  program-wide:                  gather_continuous_effects_inner  55,700 -> 52,872 gathers
```

The `(-245)` re-read's fourth row: `computed_permanent_hinted` under
`Map::next <- SmallVec::extend <- with_frozen_layers`, 14,554 asks /
21.3 M, unattributed at three callers deep and `declare_blockers <-
perform_action_inner` at five. It is the declaration's CR 702.25 /
702.45 / 702.23 P/T-delta pass: after the block taxes are paid it
opened a **second** freeze scope and asked a view of every declared
blocker and attacker — a fresh gather and a layer-pass miss per
participant, ~5,060 Ir a declaration — to read three keywords off the
computed sets. No bench archetype prints one, and the cube pool prints
Flanking on 5 files, Bushido on 37 and Rampage on 13 of a 22 k-card
catalog. `board_keyword_matching(Flanking | Bushido(_) | Rampage(_))`
in front of the scope; `false` is authoritative for every computed
keyword set, so the empty list reads exactly as the views would have.
The three join `card_has_gate_keyword`'s union.

**Gated, not reused.** The declaration's first scope already computes
those same participants (`(-215)`'s subset pass) and the obvious
change is to read `kws_of` there — but the block-tax payments sit
between the two scopes, and the second reads the *post-payment* state,
which is the state the CR 702 triggers resolve on. A gate keeps that
reading; a reuse would have moved it, on a board nobody has, and the
rule is behaviour-preserving by default.

**The scope was the cost, not the asks.** The ask row said 21.3 M; the
scope's `with_frozen_layers` row said 25.5 M and the gather count fell
by 2,828 — a scope whose first question is a miss pays the gather too,
and a table keyed on the memo's asks does not show it. Price a scope
by its `with_frozen_layers` row.

### `(-245)` TAKEN — `pick_attacks_inner`'s computed `CantBlock` read behind the keyword presence gate: `cube` -0.278 % / `fixed` -0.205 % / `sealed` -0.196 %

```text
  pool    base (-242)       (-245)          delta
  cube    1,941,531,739   1,936,134,211   **-0.278 %**
  fixed     696,705,944     695,281,092   **-0.205 %**
  sealed  2,007,370,975   2,003,446,075   **-0.196 %**
  three-pool outcomes identical
  under pick_attacks_inner, cube:  computed_permanent_hinted  24,918 -> 15,530 asks   26.34 M -> 19.46 M
                                   board_keyword_in_scope      4,688 ->  9,376 calls    0.63 M ->  1.34 M
                                   the legality collect (from_iter)                    10.00 M -> 11.37 M  (asks moved, not removed)
```

Found by ranking `computed_permanent_hinted`'s 284,812 asks by caller
on the `(-242)` base dump: `legal_blockers` 51,056 / 34.2 M and
`permanent_value_with` 44,874 / 43.3 M are the `(-194)` freeze-design
misses, and the third row was `pick_attacks_inner` at 24,918 / 26.3 M
— 5.3 asks a pick, ~1,060 Ir each, i.e. nearly all of them scope-first
misses. The `opp_blockers` walk asked a view of every untapped opposing
creature to test the *computed* set for `CantBlock`, and the pair
legality gate that follows short-circuits on the first blocker that
can block, so most of those views were asked nothing else in the scope.
`board_keyword_in_scope(&[CantBlock])` once per pick, `false`
authoritative, and the loop reads two instance fields per permanent.
`CantBlock` joins `card_has_gate_keyword`'s union, which is the lane's
printed leg; a keyword missing from that list gets a wrong `false`
there, and the gate's own `debug_assert!` (the whole-board recompute)
is the audit — 119 catalog files print `CantBlock`, so the lane is set
on more boards than before and the per-ask printed scan then runs; the
gate's +0.7 M is that price, paid on both pools.

**The transferable half: rank a memo's asks by caller and read the
caller's *consumer*, not the ask.** Three of the top four rows are
asks whose views are consumed whole; this one consumed a single
keyword, which is what a presence gate answers without a view. The
same read found `(-246)` one row further down.

### `(-244)` TAKEN — the dispatcher's empty batch skips the trigger push, the empty drain and their `Vec` drops: `fixed` -0.571 % / `sealed` -0.437 % / `cube` -0.337 %

```text
  pool    base (-243)       (-244)          delta
  fixed     694,512,156     690,547,383   **-0.571 %**
  cube    1,935,264,256   1,928,746,090   **-0.337 %**
  sealed  2,000,872,501   1,992,138,486   **-0.437 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  push_ordered_trigger_candidates 77,126 calls / 12.4 M inclusive (cube) carrying 806 candidates in all;
    per empty call: its own 60 Ir, drain_trigger_queue 34 Ir + an out-of-line Vec::drop 14 Ir, the slice drop 13 Ir
  cube self deltas: push_ordered_trigger_candidates -4.45 M  drain_trigger_queue -1.83 M
                    drop_in_place<[TriggerCandidate]> -0.99 M  dispatch_triggers_for_events +0.76 M
  first reading, #[inline] on the flag-flip helper: cube -0.265 % / fixed -0.450 % / sealed -0.340 % — the
    helper stayed out of line, 77 k x 18 Ir = 1.41 M; #[inline(always)] removed the row whole (+16 k inlined)
```

Found without a build: `profiling-fast` carries line tables, so
`callgrind_annotate --auto=yes <dump> mod.rs` (the scratchpad's
`annlines.py` ranks the result by line) puts 12.4 M on the dispatcher's
one `push_ordered_trigger_candidates` call, and that function's callee
table shows 806 `Effect::clone`s under 77,126 calls — **97.5 % of the
calls carried nothing and paid the function's prologue, an empty drain
and two `Vec` drops, ~120 Ir each.** The dispatcher's own comment said
the call was kept on the empty batch for the two per-batch jobs the
function owns (the life-gain flag flip, the `died_card_snapshots`
clear). Those are now a shared `#[inline(always)]` helper and the
`clear`, run in the dispatcher on the empty path, in the same order as
the full path (flags, drain, clear) minus the drain that had nothing to
drain. `fixed` gains most: more dispatches per Ir. **When a function's
callee table shows N calls and its work rows show N/100, the N are its
prologue — count the candidates against the calls before calling a row
a floor.** The second reading is the `(-229)` rule again: a helper whose
body is one `is_empty` behind a cold group's `Deref` is a coin the
inliner flipped tails; `inline(always)` pins it.

### `(-243)` TAKEN — the auto-tapper's activations write into its event buffer instead of returning a `Vec` each: `cube` -0.323 % / `sealed` -0.318 % / `fixed` -0.315 %

```text
  pool    base (-241)       (-243)          delta
  fixed     696,705,741     694,512,156   **-0.315 %**
  cube    1,941,530,315   1,935,264,256   **-0.323 %**
  sealed  2,007,247,937   2,000,872,501   **-0.318 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  push_mut <- activate_ability_inner <- activate_ability <- auto_tap_for_cost_inner: 48,592 pushes, 24,182 grow_one (cube)
  cube self deltas: _int_free -1.28 M  malloc -1.15 M  finish_grow -0.93 M  free -0.92 M  activate_ability -0.84 M
                    memcpy -0.66 M  grow_one -0.55 M  auto_tap_for_cost_inner -0.54 M  _int_malloc -0.45 M  __rdl_alloc -0.21 M
                    activate_ability_into +1.37 M (the body that was activate_ability's, plus the mark)
```

The `(-241)` re-read's second lead, found without a line profile: the
dump's `push_mut` edge under `activate_ability_inner` sat inside an
inlined `Vec::push` (its call site reads `vec/mod.rs:*`), so the pair of
pushes was found by reading the activation body for a two-event `Vec`
— `PermanentTapped` then `TappedForMana`, on both the `(-204)` plain
land tap and the generic mana branch. Every activation returned that
`Vec` to `auto_tap_for_cost_inner`, which appended it into its own
buffer and freed it: one allocation, one growth, one free, one 32-byte
copy per mana source paid, 24,182 times on a `cube` run because the
probes pay their casts through the auto-tapper.

The `(-225)` shape: `activate_ability_inner` and `activate_plain_land_
tap` take `events: &mut Vec<GameEvent>` and return `()`, the
eight `return Ok(vec![])`s become `Ok(())`, the nested payment's
`auto_mana_events` is appended at the point it used to *become* the
list; `activate_ability_into` is the new entry point (it records a mark
and truncates back to it on `Err`, which is what dropping the owned
`Vec` did), and `activate_ability` is that with a fresh `Vec`, so its
eight other callers do not move. The auto-tapper's two loops call the
`_into` form, with their `reserve(16)` ahead of the activation instead
of ahead of the append. The whole win is the allocator's rows — every
pool the same 0.32 %, since every pool casts through the auto-tapper.

### `(-242)` REFUTED — the dispatcher's grant list inline (`SmallVec<[TriggerGrant; 2]>`) with the filter borrowed (`Cow`): `fixed` +1.449 % / `sealed` +1.160 % / `cube` +0.711 %, reverted

```text
  pool    base (-241)       (-242)          delta
  fixed     696,705,741     706,802,793   **+1.449 %**
  cube    1,941,530,315   1,955,337,247   **+0.711 %**
  sealed  2,007,247,937   2,030,531,687   **+1.160 %**
  three-pool outcomes identical (only the wall-time line differs)
  cube self deltas:  memcpy +12.02 M   dispatch_triggers_for_events +3.73 M   SmallVec::drop +2.86 M
                     SmallVec::retain +1.20 M / Vec::retain closure +0.63 M against Vec::retain -1.53 M
                     allocator side -4.1 M (_int_free -0.98, malloc -0.85, finish_grow -0.74, free -0.72,
                       grow_one -0.44, __rdl_alloc -0.17, drop Vec<TriggerGrant> -0.19)
                     the Cow half: mentions_named_by_source +0.49, resolve_named_by_source_cow +0.49,
                       resolve_named_by_source -0.49, SelectionRequirement::clone -0.44, its drop -0.31 = -0.27 M
                     event_matches_spec -3.60 / event_kind_matches +3.04: an inliner flip, a wash
```

The `(-241)` re-read's first lead: `dispatch_board_scan` builds a fresh
`Vec<TriggerGrant>` per dispatch and a third of `cube`'s dispatches
carry a grant (23,576 heap lists), and each grant's filter was cloned by
`resolve_named_by_source` whether or not it had a `NamedBySource` leaf
to concretize. Both halves as priced: the list inline for two
(`DispatchScan.trigger_grants` and `trigger_grant_sources`'s return),
the filter a `Cow<'a, SelectionRequirement>` borrowed from the
definition unless the leaf is present. The allocator gave back the
4.9 M the table promised and the change cost 14 M more than that.

**Inline storage in a struct returned by value is a memcpy per call,
not per allocation.** `DispatchScan` went from ~56 bytes to ~220 (two
`TriggerGrant`s of a `Cow<SelectionRequirement>` plus three words each),
and every dispatch — the ~70 k that carry no grant included — moved it
out of `dispatch_board_scan`, destructured it, and ran `SmallVec`'s
out-of-line destructor on both lists: 12 M of `memcpy` and 2.9 M of
`drop` against an allocation saved on a third of them. The `Vec` form
is three words, `#[may_dangle]` (no explicit `drop` before the `&mut
self` phase — `SmallVec`'s destructor has none, so the consumers needed
a `drop(trigger_grants)` and a destructuring `let` to compile at all),
and its allocation is the cheaper side of that trade. The Cow half on
its own is -0.27 M, noise; the `mentions_named_by_source` walk costs
what the clone cost. Reverted whole. What would make the list free is
storage that is neither moved nor dropped per dispatch, and the grant
borrows the board, so that storage cannot live on the state.

### `(-241)` TAKEN — a block-tax lane in front of `block_tax_for`'s per-blocker walk: `cube` -0.154 % / `sealed` -0.089 % / `fixed` -0.086 %

```text
  pool    base (-240)       (-241)          delta
  fixed     697,307,501     696,705,741   **-0.086 %**
  cube    1,944,519,629   1,941,530,315   **-0.154 %**
  sealed  2,009,041,879   2,007,247,937   **-0.089 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  block_tax_for 8,018 calls x 437 Ir (cube) at the (-231) re-read; block_tax_present is the lane read now
```

The ninth presence lane, and the last of the walks the `(-231)` grep
ranked above ~3 M: `block_tax_for` walked the board's statics once per
declared blocker for `BlockTaxToController`, and the bot's
`block_tax_present` gate made the same walk. One lane; `block_tax_
present` *is* the lane read now (its walk was the lane's predicate),
and `block_tax_for` returns the turn-scoped tax alone on a clear lane.
Below the bar on `fixed`/`sealed` — it was 0.18 % of `cube` when priced
and the lane read costs a little of that back per blocker.

**The `(-233)`..`(-241)` lanes together: `fixed` -3.35 % / `cube`
-2.90 % / `sealed` -2.05 %, nine lanes over 61 `StaticEffect` variants.
Six lanes free on the word.**

### `(-240)` TAKEN — a prevention-static lane in front of `prevent_static_scan`'s per-damage-event mask walk: `cube` -0.315 % / `sealed` -0.179 % / `fixed` -0.134 %

```text
  pool    base (-239)       (-240)          delta
  fixed     698,244,755     697,307,501   **-0.134 %**
  cube    1,950,673,721   1,944,519,629   **-0.315 %**
  sealed  2,012,653,333   2,009,041,879   **-0.179 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  apply_prevention_shields_with 21,812 calls (cube), each opening with the scan
```

`prevent_static_scan` folds twelve prevention statics into a mask with a
board walk, and `apply_prevention_shields_with` runs it on every damage
event — 21,812 a six-game `cube` run. The `(-235)` entry priced a
*word* lane (the mask kept through membership writes); the presence form
is enough: the lane's predicate is the scan's own arm list, so a clear
lane is a zero mask, and a board carrying any of the twelve walks as
before. `cube` gains most — more damage events per game — and its
boards still carry a prevention static rarely enough for the lane to
read clear on most of them.

### `(-239)` TAKEN — an ETB-counter static lane in front of `chosen_type_etb_counter_specs` and the cast-rider walk: `fixed` -0.198 % / `sealed` -0.183 % / `cube` -0.156 %

```text
  pool    base (-238)       (-239)          delta
  fixed     699,633,465     698,244,755   **-0.198 %**
  cube    1,953,713,803   1,950,673,721   **-0.156 %**
  sealed  2,016,346,658   2,012,653,333   **-0.183 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  chosen_type_etb_counter_specs 4,918 calls (cube), the ExtraEtbCountersForCreatureCasts walk per resolving creature spell
```

The seventh presence lane, over eight `StaticEffect`s (Metallic Mimic,
Oath of Gideon, Arlinn, Giada, Master Biomancer, Muzzio's Preparations,
the two type-keyed enters-with-counter forms, and the cast rider). The
walker reaches `all_static_sources` — the battlefield plus the active
command-zone cards — so its gate is the lane **and** "no active
command-zone card", the second term being a walk of two usually-empty
lists; on a clear board the function returns only the turn-scoped
Combine Guildmage grant and skips its `creature_types` clone. **A lane
gates a walk over the zone it memoizes; a walker over two zones needs
a term per zone.**

### `(-238)` TAKEN — a hand-size static lane in front of `effective_max_hand_size`'s four walks: `fixed` -0.257 % / `cube` -0.140 % / `sealed` -0.134 %

```text
  pool    base (-237)       (-238)          delta
  fixed     701,432,933     699,633,465   **-0.257 %**
  cube    1,956,461,572   1,953,713,803   **-0.140 %**
  sealed  2,019,051,137   2,016,346,658   **-0.134 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  effective_max_hand_size 2,878 calls (cube): four walks a call — no-max, the two reductions, set-to, increase
```

The last of the four lanes filed with `(-233)`: `effective_max_hand_size`
(cleanup and the bot's discard planning) made four board walks per call
for six `StaticEffect`s. One lane; a clear lane returns the seat's
printed maximum, which is what the four walks compute on such a board.

**The `(-233)`..`(-238)` legs together: `fixed` -2.94 % / `cube` -2.29 % /
`sealed` -1.60 %, six lanes over 35 `StaticEffect` variants, each
priced off one grep of the `static_abilities` read sites ranked by the
enclosing function's self cost.** Thirteen lanes were free on the
64-bit word before, seven now. The rest of that list is below the bar
(`empty_mana_pools` already gates on pool emptiness; `chosen_type_etb_
counter_specs` walks `all_static_sources`, which reaches the command
zone, so a battlefield lane alone cannot gate it; `apply_prevention_
shields_with` and `prevent_static_scan` are the fold-word shape noted in
`(-235)`).

### `(-237)` TAKEN — an any-colour-spend static lane in front of the payment relaxation's walk: `cube` -0.525 % / `sealed` -0.520 % / `fixed` -0.485 %

```text
  pool    base (-236)       (-237)          delta
  fixed     704,849,254     701,432,933   **-0.485 %**
  cube    1,966,779,439   1,956,461,572   **-0.525 %**
  sealed  2,029,609,917   2,019,051,137   **-0.520 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  spend_mana_as_any_color_for_spell 8,886 calls (cube) — once per payment, from relax_cost_colors_for_spell
```

Every payment — spell, ability, tax — opens with `relax_cost_colors_for_
spell`, which asks whether any permanent lets mana be spent as any colour
(Mycosynth Lattice, Unexpected Potential, Emissary's Ploy) with a
board walk that runs a three-arm `match` per static. One lane over the
three; the walk stays behind it and the seat-agnostic sibling
`spend_mana_as_any_color_active_for` reads the same lane. The largest of
the four `(-235)`..`(-238)` lanes because a payment is the commonest event
of the four and the walk's per-static body was the dearest — the same
ranking the `static_abilities` grep's self-cost column gave it.

### `(-236)` TAKEN — a land-play static lane in front of `can_player_play_land`'s three walks: `fixed` -0.388 % / `sealed` -0.192 % / `cube` -0.172 %

```text
  pool    base (-235)       (-236)          delta
  fixed     707,597,733     704,849,254   **-0.388 %**
  cube    1,970,174,350   1,966,779,439   **-0.172 %**
  sealed  2,033,517,838   2,029,609,917   **-0.192 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  can_player_play_land 8,198 calls (cube): the Aggressive Mining walk, damping_engine_locks' walk, extra_land_plays_per_turn's walk
```

The bot asks `can_player_play_land` once per land-play candidate — 8,198
times a six-game `cube` run — and each ask was three board walks
(Aggressive Mining / "you can't play lands", Damping Engine, the
extra-land count). One lane over the four statics; the two helpers gate
themselves (they have callers of their own) and the walker reads it once.
`fixed` gains most: its archetypes play a land nearly every turn and the
bot re-asks on every main-phase decision.

### `(-235)` TAKEN — a damage-replacement static lane in front of six per-damage-event walks: `fixed` -0.192 % / `sealed` -0.173 % / `cube` -0.165 %

```text
  pool    base (-234)       (-235)          delta
  fixed     708,958,856     707,597,733   **-0.192 %**
  cube    1,973,436,802   1,970,174,350   **-0.165 %**
  sealed  2,037,047,943   2,033,517,838   **-0.173 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  damage_redirect_target 7,428 calls (two walks); deal_damage_to_from's player arm, four walks per player-damage event
```

`(-233)`'s device on the damage path: every damage event to a player
walked the board for The Mindskinner, Crumbling Sanctuary, Delaying
Shield and Nefarious Lich in turn, and `damage_redirect_target` walked
it twice more (Pariah's Shield, Palisade Giant) for every damage event
at all. One lane, six `StaticEffect`s in its predicate, both functions
read it once. Smaller than the draw and ETB lanes because the walks
were already cheap — a `matches!` per static on a list that is empty on
most permanents — and the prevention machinery beside them
(`prevent_static_scan`'s per-event mask, `apply_prevention_shields_with`)
is untouched: the scan is a definition-only fold of twelve statics
computed by a board walk on every damage event, and it is the same shape
as a lane holding a *word* rather than a bit — the next device on this
path if one is wanted (~280 Ir an event over ~13 k events on `cube`).

### `(-234)` TAKEN — an ETB-static lane in front of `etb_trigger_multiplier` and `apply_enters_tapped_replacement`: `fixed` -0.391 % / `cube` -0.373 % / `sealed` -0.116 %

```text
  pool    base (-233)       (-234)          delta
  fixed     711,743,769     708,958,856   **-0.391 %**
  cube    1,980,826,495   1,973,436,802   **-0.373 %**
  sealed  2,039,413,384   2,037,047,943   **-0.116 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  etb_trigger_multiplier 7,916 calls / apply_enters_tapped_replacement 7,758 (cube), both once per entering permanent
```

`(-233)`'s device on the ETB path: every permanent entering the
battlefield paid `etb_trigger_multiplier` (two `battlefield_find`s and
three board walks — Torpor Orb, Doorkeeper Thrull, Elesh Norn /
Panharmonicon) and `apply_enters_tapped_replacement` (one cross-permanent
`EntersTapped` walk with a selector match per static, and a
`LandsEnterUntapped` walk for lands), ~1,400 Ir between them on a board
that carries none of the five statics. One lane whose predicate is the
union of the five; the multiplier returns `1` on a clear lane (exact:
every other return is behind one of those statics) and the enters-tapped
walks read an empty slice. `sealed` is the smallest because its boards
carry an enters-tapped static (an Aura or a tapped-land lord) more often
— the lane reads `PRESENT` and the walks run as before.

### `(-233)` TAKEN — a draw-replacement static lane in front of `draw_one`'s eleven board walks: `fixed` -0.911 % / `cube` -0.773 % / `sealed` -0.478 %

```text
  pool    base (-231)       (-233)          delta
  fixed     718,286,239     711,743,769   **-0.911 %**
  cube    1,996,255,268   1,980,826,495   **-0.773 %**
  sealed  2,049,218,164   2,039,413,384   **-0.478 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  draw_one inclusive (cube)   <- advance_step 2,822 / 14.16 M -> 5.84 M;  <- run_effect 1,406 / 10.03 M -> 3.00 M
  the 340 recursive draws (redirects) are 340 on both sides; the 486 "draw_one -> draw_one" calls that
  went were the global_static closure's own symbol
```

`draw_one` — 5,072 calls a six-game `cube` run, once per card drawn,
including every draw inside a bot probe — walked the whole battlefield's
`static_abilities` up to **eleven** times per draw: three global
replacement statics (Uba Mask, Shared Fate, "players skip draws"), four
controller-scoped dig replacements (Abundance, Tomorrow, Parallel
Thoughts, Archmage Ascension), Obstinate Familiar's skip, Chains of
Mephistopheles, Notion Thief, Blood Scrivener and Breathstealer's Crypt.
Each walk is ~250-500 Ir (the dig ones run `active_static` per static)
and each matches a different `StaticEffect`. One battlefield lane whose
predicate is the **union** of all twelve (`card_has_draw_static`) answers
every walk at once: a clear lane — nearly every board — is one word load
per draw, and the eleven walks stay exactly as written behind it. The
predicate is definition-only, so the lane holds across taps, damage and
counters, and the per-instance `active_static` gates inside the helpers
only narrow what it over-approximates.

**The rule: when a function asks the same zone N different presence
questions, the lane's predicate is the union, not one question.** Twelve
`matches!` arms in one predicate is what makes one lane serve eleven
walks; a lane per question would have been eleven lanes and eleven
reads. Found by the `static_abilities` grep NEXT filed after `(-231)`,
ranked by the enclosing functions' self cost: `draw_one` was the top row
at 8.9 M self, and the same list has `etb_trigger_multiplier` /
`apply_enters_tapped_replacement` (7.5 M + 3.7 M, the next lane),
`spend_mana_as_any_color_for_spell` (3.3 M, once per payment),
`effective_max_hand_size`, `empty_mana_pools`, `can_player_play_land`.

### `(-232)` REFUTED — the step and cast walkers' closures handed to `for_each_triggerer` by value instead of `&mut`: `cube` -0.005 % / `sealed` +0.049 % / `fixed` +0.077 %, reverted

```text
  pool    base (-231)       (-232)          delta
  fixed     718,286,239     718,837,316   **+0.077 %**
  cube    1,996,255,268   1,996,148,610   **-0.005 %**
  sealed  2,049,218,164   2,050,212,788   **+0.049 %**
  FnMut for &mut F::call_mut <- fire_step_triggers 73,194 / 4.19 M and <- finalize_cast 53,340 / 13.06 M: both rows gone
  fire_step_triggers self (cube) 10.72 M -> 10.74 M
```

`(-228)` and `(-231)` pass their per-card closure as `&mut visit` so one
body serves two branches (whole board under a live grant, the member
list otherwise), and the `(-98)` rule says `&mut F: FnMut` routes each
card through a `call_mut` shim the inliner declines — the dump showed the
shim on 126,534 calls. A `for_each_triggerer_or_all(all, f)` entry point
takes the closure by value, the shim rows vanish, and the totals do not
move: **the shim is a `call` and a frame, ~20 Ir, and the by-value form
pays it back by monomorphizing the closure body into both branches** —
the per-card body is what the walk costs, not how it is reached.
`(-98)`'s 18.8 M was a *predicate* called per card inside `any` over the
whole board; a closure whose body is a dozen loads is not that. Reverted:
the `&mut` form is fewer lines.

### `(-231)` TAKEN — the cast-trigger walker's two zone walks behind their memos: `fixed` -1.319 % / `sealed` -0.492 % / `cube` -0.341 %

```text
  pool    base (-230)       (-231)          delta
  fixed     727,884,518     718,286,239   **-1.319 %**
  cube    2,003,093,203   1,996,255,268   **-0.341 %**
  sealed  2,059,349,123   2,049,218,164   **-0.492 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  finalize_cast self (fire_spell_cast_triggers is inlined into it)   fixed 9.56 M -> 3.74 M   sealed 17.61 M -> below the 0.5 % cut   cube 16.04 M -> below the cut
```

`fire_spell_cast_triggers` runs once per cast and made two zone walks
into printed `triggered_abilities`: the whole battlefield for `SpellCast`
triggers (now the trigger member list when no static or equipment grant
is live — `(-228)`'s shape), and **every player's graveyard collected as
`(id, owner)` pairs, filtered to the caster, then each id re-found by a
linear search of the graveyard it came from** — a quadratic walk per cast
for the Dissension Eidolons' "whenever you cast a multicolored spell,
return this from your graveyard". The caster's graveyard is the only one
the filter ever kept, so it is now walked directly, behind the `(-230)`
lane. `fixed`'s -1.3 % is the quadratic half: its four archetypes cast
often and their graveyards are long by mid-game.

**The grep after `(-228)` found this too** (every `definition.
triggered_abilities` site in the engine, asked "which zone, and is it
memoized"). Three of the twenty sites were whole-zone walks with a memo
already beside them — `(-228)`, `(-230)`, `(-231)` — and the grep cost
nothing. The dispatcher's remaining zone legs (exile behind an event-kind
gate, command zones and hands behind presence gates) were read and are
gated.

### `(-230)` TAKEN — the event dispatcher's graveyard leg behind the graveyard lane, the lane's predicate widened to both graveyard-firing families: `fixed` -0.784 % / `sealed` -0.428 % / `cube` -0.350 %

```text
  pool    base (-228)       (-230)          delta
  fixed     733,633,503     727,884,518   **-0.784 %**
  cube    2,010,133,057   2,003,093,203   **-0.350 %**
  sealed  2,068,205,261   2,059,349,123   **-0.428 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  dispatch_triggers_for_events self   fixed 43.98 M -> 38.39 M   cube 86.71 M -> 79.54 M   sealed 117.26 M -> 108.30 M
```

`dispatch_triggers_for_events` — 143,852 calls a six-game `cube` run —
walked **both players' whole graveyards** into every card's printed
`triggered_abilities` on every dispatch, for the `FromYourGraveyard`
scope and the graveyard-resident `SelfSource` kinds (cycling, milling,
discard, "from anywhere"). The `(-210)` lane already held "does any card
here carry a `FromYourGraveyard` trigger" for the combat-damage and step
walkers; its predicate now covers both families (`is_graveyard_self_
source_kind` is the one list, so the lane and the walk cannot drift),
which is wider — the sound direction — for its two older readers, and
the dispatcher's leg skips a graveyard whose lane reads `ABSENT`. A
graveyard grows all game; the cost was a definition deref per graveyard
card per dispatch on a zone that holds such a card in a few games out of
six.

**Found by grepping for the read, not by the profile**: every
`definition.triggered_abilities` site in the engine was listed after
`(-228)` and each asked "is this a whole-zone walk, and is the zone
memoized". This one had sat inside the dispatcher's 4-6 % self row —
which the line profile at `(-115)` called "the per-event bookkeeping" —
for the whole history of the lane that could gate it. **A self row's
line profile names where the instructions are; the grep names which of
them a memo already answers.**

### `(-228)` TAKEN — the step-trigger walk visits the trigger member list when no static grant is live: `fixed` -0.421 % / `cube` -0.127 % / `sealed` -0.052 %

```text
  pool    base (-229)       (-228)          delta
  fixed     736,733,907     733,633,503   **-0.421 %**
  cube    2,012,692,074   2,010,133,057   **-0.127 %**
  sealed  2,069,285,360   2,068,205,261   **-0.052 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  fire_step_triggers self   fixed 10.89 M -> 6.95 M   cube 16.79 M -> 10.94 M   sealed 19.74 M -> 13.72 M
  compute_permanent_pass self (cube) 74.22 -> 73.67 M — the (-229) shape held across the build
```

`fire_step_triggers` runs on every step of every turn (24,216 calls a
six-game `cube` run) and walked every permanent's printed
`triggered_abilities` for the step's kind. A live static grant can hand a
trigger to any permanent, so that board is still walked whole (under the
freeze scope it already took); without one only a permanent with a
printed trigger or a Station band can contribute — `card_is_triggerer`,
the trigger member list's own predicate — so the walk is
`for_each_triggerer`. The `(-196)` refutation in this function was a
*per-card* fold gate (a memo load against a one-element tag loop, a
wash); this skips the cards, not the compare.

**Measured twice, and the first reading is the reason `(-229)` exists:**
against the `(-226)` tip this read `cube` **+0.86 %** with its own rows
at -8 M, because the build flipped the layer pass's `extend` out of
line. **When a total contradicts the device's rows, diff the two self
tables before believing either** — the confound was two rows the edit
never touched, and pinning them was worth more than the device.

### `(-229)` TAKEN — the layer pass fills its effect list with push loops, not `SmallVec::extend`: `cube` -0.341 % / `fixed` -0.174 % / `sealed` -0.004 %

```text
  pool    base (-226)       (-229)          delta
  fixed     738,014,635     736,733,907   **-0.174 %**
  cube    2,019,586,479   2,012,692,074   **-0.341 %**
  sealed  2,069,367,315   2,069,285,360   **-0.004 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  compute_permanent_pass self (cube)   81.09 M -> 74.22 M;  SmallVec::extend <- compute_permanent_pass: gone (was inlined at (-226), out of line at (-228)'s first build)
```

Found by the confound, not by a profile: `(-228)`'s first build (a
stack.rs-only edit) read `cube` **+0.86 %** with the device's own rows
at -8 M, and the self-table diff put the whole difference in two rows the
edit never touched — `SmallVec::extend` +27.6 M and `compute_permanent_
pass` -10.2 M. `compute_permanent_pass` built its per-permanent effect
list with two `extend`s, one over a `Filter`; whether that generic
inlines is decided per build, and out of line it is 420,672 calls at
~137 Ir on `cube` against ~0 inlined. **A generic adapter on a
400 k-call path is a coin the inliner flips on every build; write the
inlined shape down.** Two `push` loops are that shape. The `(-156)` rule
("a std-adapter rewrite is worth ~10 % of the adapter's self") is about a
row that *is* inlined; this one is about keeping it so, and the loop form
also reads below the previously-inlined build (-6.9 M self on `cube`).
`sealed` is flat because its layer pass is dominated by boards where the
list is longer and the loop body, not the frame, is the cost.

### `(-227)` REFUTED — a `reserve(32)` in `advance_step` ahead of the damage steps' first push: `sealed` +0.298 % / `fixed` +0.241 % / `cube` +0.220 %, reverted

```text
  pool    base (-226)       (-227)          delta
  fixed     738,014,635     739,793,304   **+0.241 %**
  cube    2,019,586,479   2,024,029,917   **+0.220 %**
  sealed  2,069,367,315   2,075,533,702   **+0.298 %**
  do_reserve_and_handle (cube)   <- advance_step         0 -> 4,862 / 7.36 M  (1,514 Ir each)
                                 <- resolve_combat_into  4,524 / 5.79 M -> 4,524 / 3.46 M  (still there)
```

`(-225)`'s priced follow-up, built as filed and wrong on the arithmetic:
**`Vec::reserve(n)` reserves `n` slots *beyond `len`*, not a capacity of
`n`.** `advance_step` reserved 32, pushed `StepChanged`, and the damage
step's own `reserve(32)` then asked for 33 and reallocated the fresh
32-slot buffer to 64 — two large allocations where the tree had a 4-slot
first push plus one realloc. The per-call price says the rest: a 32-slot
`GameEvent` buffer costs ~1,300-1,500 Ir to obtain from the system
allocator whether by `malloc` or by `realloc`, so a caller-side
`reserve(33)` would at best trade the 4-slot `malloc` for nothing and
move the big allocation one frame up. **The cost is the size of the
buffer, not the number of allocations, and the damage step's batch is
genuinely that size** (the `(-80)`-era ladder climbed to 32 and past it).
What would remove it is a scratch that survives probe clones, which is a
thread-local pool — the `(-166)` recycle-list device — and that is a
separate entry to price, not this one.

### `(-226)` TAKEN — `do_untap`'s two remaining static-driven walks behind `any_static`: `fixed` -0.229 % / `sealed` -0.042 % / `cube` -0.035 %

```text
  pool    base (-225)       (-226)          delta
  fixed     739,706,759     738,014,635   **-0.229 %**
  cube    2,020,284,063   2,019,586,479   **-0.035 %**
  sealed  2,070,228,163   2,069,367,315   **-0.042 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  do_untap inclusive   fixed 1,896 / 11.92 M -> 10.22 M   cube 2,834 / 24.76 -> 24.07 M   sealed 3,658 / 26.94 -> 26.09 M
```

`do_untap` answers "does any permanent or command-zone card carry a
static at all" once (`any_static`) and six of its walks already sit
behind it; two did not — the Thousand Moons Infantry "untap this during
each other player's untap step" `&mut` loop and the Urban Burgeoning
aura-host collect, both of which read `static_abilities` only. On
`fixed`, whose four archetypes carry few statics, the gate answers `false`
on most untap steps and the two walks (~900 Ir a step between them) go;
`cube`/`sealed` boards usually have a static somewhere, so the gate
answers `true` and only the gate's own cost shows. Read off the callee
table's `Vec::from_iter` row (4.4 collects an untap step, 367 Ir each —
the walk runs *inside* `from_iter`, so a collect over a filter is charged
to the adapter, not to the function).

### `(-225)` TAKEN — the combat damage step writes into the caller's event buffer: `sealed` -0.136 % / `fixed` -0.118 % / `cube` -0.070 %

```text
  pool    base (-224)       (-225)          delta
  fixed     740,581,398     739,706,759   **-0.118 %**
  cube    2,021,695,434   2,020,284,063   **-0.070 %**
  sealed  2,073,037,291   2,070,228,163   **-0.136 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  do_reserve_and_handle <- resolve_combat(_into)   cube 4,848 / 5.13 M -> 4,524 / 5.79 M
                                                    fixed 2,878 / 3.15 M -> 2,688 / 3.64 M
                                                    sealed 6,632 / 7.21 M -> 6,276 / 8.46 M
```

`resolve_combat_damage_with_filter` built its own `Vec`, `reserve(32)`'d
it (the `(-80)`-era fix for a 0->4->8->16->32 ladder) and handed it back
for `advance_step` to `append` into the recycled scratch buffer and free —
an allocation, a copy and a free per damage step. Both damage-step entry
points now have an `_into` form that writes into the caller's buffer, and
`advance_step` passes its scratch; the `()` forms stay as wrappers for the
139 suite call sites and `submit_decision`.

**What it did not remove, and why the row grew per call:** 4,524 of the
4,848 reserves are still there, at 1,279 Ir instead of 1,057. Most combat
damage steps run inside the bot's probe clones, and a clone's scratch is
`Vec::new()` — so the step's buffer arrives holding only `StepChanged` in
a 4-slot allocation, and `reserve(32)` is a `realloc` of it rather than
a fresh `malloc`. The win is the append and the free; the allocation
moved rather than vanished, exactly as the recycle-list rules say a
reserve does. **A buffer recycled per state is not recycled across
probe clones** — the next device, if any, is a `reserve(32)` in
`advance_step` before its first push on the two damage steps, merging the
4-slot allocation and the realloc into one (~1.4 M on `cube`, priced, not
built).

### `(-224)` TAKEN — the combat-damage-to-player listener walk visits the trigger member list: `fixed` -0.096 % / `cube` -0.069 % / `sealed` -0.057 %

```text
  pool    base (-223)       (-224)          delta
  fixed     741,293,309     740,581,398   **-0.096 %**
  cube    2,023,099,539   2,021,695,434   **-0.069 %**
  sealed  2,074,212,577   2,073,037,291   **-0.057 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  fire_combat_damage_to_player_triggers inclusive   cube 7,128 / 18.35 M -> 16.93 M   fixed 3,266 / 6.65 -> 5.99 M   sealed 9,450 / 21.02 -> 19.79 M
```

`(-222)`'s device on its third walk: the CR 510 "whenever combat damage
is dealt to you" listeners (Risona, Teysa) were a whole-battlefield walk
into printed `triggered_abilities` per combat-damage event, once per
attacker that connects. `for_each_triggerer` again. ~200 Ir an event;
priced at the bar and taken because the change is one line and the list
is already warm. The remaining printed-trigger walks in `combat.rs` are
inside `fire_combat_damage_triggers`, whose listener leg is gated by the
`LISTENER` dispatch bit (and whose fold `(-221)` refuted).

### `(-223)` TAKEN — the block declaration stops paying a `{0}` block tax: `fixed` -0.375 % / `sealed` -0.371 % / `cube` -0.316 %

```text
  pool    base (-222)       (-223)          delta
  fixed     744,080,362     741,293,309   **-0.375 %**
  cube    2,029,520,828   2,023,099,539   **-0.316 %**
  sealed  2,081,933,562   2,074,212,577   **-0.371 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  try_pay_after_snapshot_mode <- declare_blockers   cube 2,828 / 5.50 M   fixed 1,528 / 2.81 M   sealed 3,680 / 6.65 M   -> 0
  declare_blockers inclusive (cube)   136.5 M -> 130.3 M
```

Read off `declare_blockers`' callee table at the `(-220)` tip: the CR
509.1b "can't block unless its controller pays {N}" leg summed the tax
per blocking seat and then paid it through `try_pay_with_auto_tap` **for
every seat in the map, zero included** — a payment snapshot of the seat's
whole board (`Vec` of `(id, tapped)`), a pool clone, the colour
relaxation, an auto-tap pass that finds nothing to tap, and a `{0}`
`pay_for_spell`, once per blocking seat per declaration, on a board that
carries the keyword on no card. The attack side's twin (`total_tax > 0`,
in `declare_attackers_banded`) has had the gate for its whole life; this
side never did. A `{0}` payment moves nothing (the snapshot is restored
only on failure, and it cannot fail), so the gate is outcome-identical,
which the three pools, `--bench` and the traces confirm.

**The rule: when two sides of one mechanic are written twice, diff the
gates, not the bodies.** `attack_block_keyword_tax` is shared; the two
call sites were not, and the one without `> 0` cost a third of a percent
of every pool for as long as the block tax has existed.

### `(-222)` TAKEN — the attack declaration's two printed-trigger walks visit the trigger member list: `cube` -0.134 % / `fixed` -0.126 % / `sealed` -0.073 %

```text
  pool    base (-220)       (-222)          delta
  fixed     745,019,299     744,080,362   **-0.126 %**
  cube    2,032,242,422   2,029,520,828   **-0.134 %**
  sealed  2,083,448,867   2,081,933,562   **-0.073 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  declare_attackers_banded self   cube 27.62 M -> 22.37 M   fixed 10.35 M -> 8.81 M   sealed 26.95 M -> 22.46 M
```

The `(-219)` re-read priced `declare_attackers_banded`'s 4,086-Ir self as
"the validation body — a dozen `attacks.iter().any(..)` scans" and left
it; two of the walks in that body are not scans of the batch but of the
*board*: the CR 508.1g `ControllerAttackedByOpponent` listeners (one
whole-battlefield walk per attacker, into every permanent's printed
`triggered_abilities`) and the CR 508 "whenever you attack" walk (one
per declaration). Both read printed triggers only, which is exactly the
question `Battlefield::trigger_members` already holds the answer to —
kept exact through membership writes since `(-214)`, filled by every
dispatch, so it is warm at every declaration. `for_each_triggerer` walks
the list on a hit and the board on a miss (it never fills; the
dispatcher does). Self came down 19 % on `cube` for ~5 lines.

**The rule: a "validation body" is worth reading for the walks that are
not over the batch.** A dozen `any` over two attackers is nothing; the
two loops over twenty-three permanents' definitions beside them were a
fifth of the function, and the member list that removes them was
already there.

### `(-221)` REFUTED — the combat-damage walker's listener walk fused into its dealer walk (built against the `(-219)` tip, concurrently with `(-220)`): `fixed` +0.131 % / `cube` +0.174 % / `sealed` +0.131 %, reverted

```text
  pool    base (-219)       (-221)          delta
  fixed     745,162,927     746,135,663   **+0.131 %**
  cube    2,035,554,686   2,039,094,144   **+0.174 %**
  sealed  2,085,022,232   2,087,761,970   **+0.131 %**
  cube:   fire_combat_damage_triggers self 28.57 M -> 32.68 M (+4.1 M); nothing else moved
  three-pool stdout identical; --bench counters identical (the fold changed no push)
```

The `profiling-lines` read the `(-219)` re-read asked for (a 10-minute
cold build, `cg_lines.py --in fire_combat_damage_triggers`): 29.0 M
grouped, and the top rows were `CardId::eq` 5.17 M (`card.rs:13`), the
listener walk's memo-word test 2.44 M (`combat.rs:5613`), slice
stepping 1.95 M, `NonNull` / `mut_ptr` 3.0 M, the atomic load 1.34 M —
i.e. the two full battlefield walks (dealer, listener), each an `Arc`
deref and a compare per permanent. The obvious fold — collect the
listeners into a `SmallVec<[&CardInstance; 16]>` on the dealer walk and
process them where the second walk ran, every push in the same bucket
in the same order — **cost 4.1 M more than the walk it removed.** The
second walk over a slice whose `CardData` lines are already hot is a
tight loop of one word load and one branch per card; the fold added a
`scan_listeners` test and a `SmallVec` push (length check, spill
check, store) to the dealer loop's body on every card, and then
re-walked the hits through an indirection. **A line profile's `Arc`
deref and `CardId::eq` rows are instruction counts, not cache misses:
a second walk over a hot slice is priced by its own loop body, and
fusing it buys nothing unless the fused body is smaller than the two
it replaces.** Reverted; `fire_combat_damage_triggers` stays at
~1,400 Ir a call, and its call count (20,480 on `cube`: one per
attacker per damage event, from `resolve_combat`) is the lever, which
is the attack search's.

### `(-220)` TAKEN — the CR 732.3 announcement watch fingerprints only on a key repeat: `cube` -0.163 % / `sealed` -0.076 % / `fixed` -0.019 %

```text
  pool    base (-219)       (-220)          delta
  fixed     745,162,383     745,019,299   **-0.019 %**
  cube    2,035,552,660   2,032,242,422   **-0.163 %**
  sealed  2,085,024,159   2,083,448,867   **-0.076 %**
  three-pool outcomes identical; --bench counters identical; golden traces 7/7 unmoved
  fingerprint <- activate_ability   cube 3,232 -> 62   sealed 2,192 -> 362   fixed 122 -> 8
  the resolve-side watch (3,488 / 808 / 7,384 calls) untouched
  census, (-217) tip, six games, consecutive same-key announcements / all announcements:
              cube 220 / 26,000 (0.85 %)   fixed 198 / 8,000 (2.5 %)   sealed 520 / 28,000 (1.9 %)
```

The other half of the `(-219)` row, built concurrently against the
`(-217)` tip where it read **`cube` -1.130 % / `sealed` -0.974 % /
`fixed` -0.919 %** on its own — `(-219)` landed first and took the
land-tap share, so this is what is left of the announcement side. The
watch compares a fingerprint against the previous announcement's *only
when that one was the same ability*, and a census says the key repeats on
0.9-2.5 % of announcements: a different key now stores the key with
`n == 0` ("pending") and no fingerprint, and the first repeat computes
one and counts the pending announcement as unchanged. A genuine loop is
refused at the same announcement as before (the 50-then-refuse test is
byte-for-byte); the one case that moves is a repeat whose first two
announcements saw different states, refused one announcement earlier.
Same serde shape; the `(-219)` reset is the initial state.

**The rule: price a memo by what compares against it, not by what
computes it.** The fingerprint was computed on every announcement and
read on one in fifty; the candidates entry priced the walk (900 Ir a
call, "no device seen") and refuted a cheaper *policy*, when the device
was to defer the computation to the read. What is left of `fingerprint`
is the CR 104.4b resolution watch — 3.2 M on `cube`, 5.8 M on `sealed`
(0.28 %) — which compares every trigger resolution against the previous
one, so consecutive trigger resolutions are the common case there and
the same deferral does not apply.

### `(-219)` TAKEN — the CR 732.3 announcement watch behind the land-tap fast path: `cube` -1.004 % / `fixed` -0.951 % / `sealed` -0.938 %

```text
  pool    base (-218)       (-219)          delta
  fixed     752,320,968     745,162,927   **-0.951 %**
  cube    2,056,208,537   2,035,554,686   **-1.004 %**
  sealed  2,104,769,657   2,085,022,232   **-0.938 %**
  three-pool stdout identical; --bench counters identical; golden traces 7/7 unmoved
  fingerprint (self)   fixed   9,636 calls / 7.53 M  ->    930 / 0.63 M
                       cube   29,196 calls / 26.28 M -> 6,720 / 6.40 M
                       sealed 35,598 calls / 26.41 M -> 9,576 / 7.46 M
```

Read off the `(-217)` tip's self table: `fingerprint` was 1.0-1.3 % of
every pool, and its caller table said 23,666 of `cube`'s 29,196 calls
came from `activate_ability` — the CR 732.3 announcement watch, taken
*before* `activate_ability_inner` and so before the `(-204)` fast path,
on every printed land tap. A plain land tap cannot trip the watch: it
flips its source's `tapped` bit and grows a mana pool, both in the
digest, so its own key can never repeat on an unmoved state; and since
the watch only compares against the *immediately previous* activation,
the one thing a land tap ever did to it was reset it. The fast path now
resets the watch to its initial `(0, None, 0)` and the check runs after
the fast path for every other activation — still before any cost is
paid. Outcome-identical in every reachable sequence (the count can
differ by one only when the same land is re-tapped on a byte-identical
digest, which needs a step boundary and an untap trigger between the
two, i.e. at most a handful a turn against a cap of 50); the both-ways
test names the field and checks each side's value.

**The rule this adds to the fast-path device: price every *caller-side*
wrapper of the function the fast path shortcuts.** `(-204)` settled
`activate_ability_inner` and was measured through `activate_ability`,
whose own prologue kept paying 900 Ir a tap for three passes.

### `(-218)` TAKEN — the step and combat-damage walkers behind zone lanes, the intervening-if filter in place: `fixed` -0.344 % / `sealed` -0.208 % / `cube` -0.158 %

```text
  pool    base (-217)       (-218)          delta
  fixed     754,914,487     752,320,968   **-0.344 %**
  cube    2,059,454,483   2,056,208,537   **-0.158 %**
  sealed  2,109,167,085   2,104,769,657   **-0.208 %**
  three-pool stdout identical; --bench counters identical; golden traces 7/7 unmoved
  rows (self)                        fixed                cube                 sealed
  fire_step_triggers            12.18 -> 10.89 M    18.06 -> 16.79 M    21.23 -> 19.74 M
  from_iter_in_place             1.27 ->  0.50 M     2.10 ->  0.94 M     2.58 ->  1.09 M
  fire_combat_damage_triggers    7.47 ->  7.31 M    28.84 -> 28.57 M    25.31 -> 25.01 M
```

Three small gates on the two trigger walkers, priced off the `(-217)`
tip's self table (`fire_step_triggers` 0.9-1.6 % of a pool at ~700 Ir
a call over 24,216 `cube` calls; `fire_combat_damage_triggers` 1.4 %):

* **`fire_step_triggers` walked the active player's graveyard on every
  step** — a definition deref per graveyard card for a
  `FromYourGraveyard` step trigger 44 printings carry. The `(-210)`
  lane's predicate is the scope, not the kind, so
  `has_graveyard_trigger()` already held the answer; the walk now sits
  behind it. Most of the leg on every pool.
* **The intervening-if pass was `into_iter().filter().collect()` on a
  list that is empty on most steps**, and the in-place collect
  machinery runs whether or not there is anything to filter. A guarded
  `retain` (the emptiness test is there because `retain` is an
  out-of-line generic that cost `fixed` +0.08 % on an empty list the
  last time it was tried bare).
* **The Cipher walk of exile** in `fire_combat_damage_triggers` read
  `encoded_on` behind every exiled card's `Arc` on every damage event
  to a player. `CardPile` grew a second lane, `has_encoded()` — an
  *instance* predicate, which a pile lane may hold because every `&mut`
  route into a pile (`push`, `DerefMut`) clears the whole word, unlike
  the battlefield's `iter_mut`; `zone::tests::pile_encoded_lane_follows_
  the_instance_flag` pins that. The smallest of the three, as priced:
  exile is short on these pools.

What is left in `fire_step_triggers` (~690 Ir a call on `cube`) is the
battlefield walk's tag test per printed trigger, the equipment walk and
the command-zone / emblem walks; `(-196)` measured the per-card kind
fold as a wash there, and a board-level fold of it would read PRESENT on
most `cube` boards (`StepBegins` is one tag for every step, and upkeep
triggers are common). Not a lead.

### `(-217)` TAKEN — the two per-death registries leave the cold group: `cube` -1.228 % / `fixed` -0.832 % / `sealed` -0.351 %

```text
  pool    base (-216)       (-217)          delta
  fixed     761,244,306     754,914,487   **-0.832 %**
  cube    2,085,050,322   2,059,454,483   **-1.228 %**
  sealed  2,116,592,411   2,109,167,085   **-0.351 %**
  three-pool stdout identical; --bench counters identical; golden traces 7/7 unmoved
  the device, priced by make_mut_slow's inclusive row:
              cube    106.10 M -> 100.94 M   -5.16 M  (-0.25 %)
              fixed    48.17 M ->  45.89 M   -2.28 M  (-0.30 %)
              sealed  123.68 M -> 119.42 M   -4.26 M  (-0.20 %)
              note_creature_death's unshares 1,900 / 7.06 M -> 2,450 / 0.53 M (cube)
  the rest is a codegen shift that came with the build (cube):
              SmallVec::extend        37.76 M -> 10.18 M
              compute_permanent_pass  70.86 M -> 81.09 M
              FilterMap::next          6.28 M ->  0.34 M;  FnMut::call_mut  9.69 M -> 14.37 M
```

`creature_deaths_this_turn` and `graveyard_from_battlefield_this_turn`
were `ColdState` fields and a death writes both, so every death that
was the first cold write after a clone paid the group's ~3,700-Ir
unshare — 1,900 of `cube`'s 9,074 deaths under `note_creature_death`
and the rest under `remove_from_battlefield_to_graveyard_raw`'s insert.
They now share a `CowBox<TurnDeaths>` of their own, flattened into the
same serde shape (`ColdState`'s doc said this move was the remedy for a
field written on the hot path; it was). **Quote the device at its own
rows, not the total:** the three-pool total is two to five times the
unshare saving because the inliner moved the layer pass's `SmallVec::
extend` into `compute_permanent_pass` in the same build (the "LTO
confound" in the anchors section, seen from the winning side); real
for this binary, and not to be re-counted when a later build moves it
back.

**The first cut was refuted (+0.103 % / +0.068 % / +0.078 %):** it
moved `creature_deaths_this_turn` alone, to its own `CowBox<Vec<..>>`,
and `make_mut_slow`'s caller table showed the 1,900 unshares had simply
walked down the path to `remove_from_battlefield_to_graveyard_raw`
(1,768 -> 3,668) — the graveyard-set insert was the next cold write of
the same action — while the extra handle cost +1.4 M. **An unshare is
paid by the first cold write of the action, not by the field that
happens to be first; move a field out of the cold group only with every
other cold write on that path** (the cold-write census that found the
second one is a grep of `self.<cold field>` writes over the path's
functions, and the remaining ones — `temporary_control`,
`auras_at_death`, the trigger-use maps — are all guarded or rare).

### `(-216)` TAKEN — a presence gate on the target in `check_target_legality`: `fixed` -0.324 % / `cube` -0.245 % / `sealed` -0.181 %

```text
  pool    base (-215)+fix   (-216)          delta
  fixed     763,717,868     761,244,306   **-0.324 %**
  cube    2,090,168,791   2,085,050,322   **-0.245 %**
  sealed  2,120,435,808   2,116,592,411   **-0.181 %**
  three-pool stdout identical; --bench counters identical; golden traces 7/7 unmoved
  computed_permanent_hinted <- check_target_legality_with_source:
              cube   15,416 calls / 21.28 M  ->  11,132 / 8.87 M
              fixed   4,286 calls /  5.63 M  ->   2,196 / 1.48 M
  the gate's own rows (cube): Mutex::lock 15,416 / 0.40 M (`layers_memoized`),
              can_grant_keyword 2,166 / 0.17 M, card_has_anthem 496 / 0.01 M
```

The candidates block's top lead, taken as written: the check opened its
own freeze scope and read the target's computed view for Shroud,
Hexproof and the ability ward, so every call that was not nested in an
outer scope gathered the whole board to ask three questions almost no
target answers `yes` to. `card_keyword_possible_on` — the `(-204)` fast
path's device, aimed at the target — answers "none of the three can be
on this permanent" off its printed keywords, EOT grants, keyword
counters and the grant member list without a view; only a `true` takes
the view. Two details carry the win: the gate skips itself when the
scope's gather is already memoized (`layers_memoized`, the same pairing
`damage_from_source_prevented_by_keyword` uses — a memo read is cheaper
than the gate), and the ability-ward family is asked only when a
`source_card_id` is present, since the check does not read it otherwise.
A `debug_assert!` recomputes the view on every gated miss, so the suite
audits the four-seed claim on every targeting decision it makes.

The 11,132 views that remain on `cube` are the memoized ones: the
gate's `can_grant_keyword` row (2,166 calls) says a granter was on the
board and asked in only a seventh of them. **What is left in this
function is the scope itself** — `Unfreeze::drop` 17,458 / 0.32 M and
`Mutex::lock` — about 0.04 % of `cube`; not a lead.

### `(-215)` TAKEN — the dispatch scan visits its member list: `cube` -0.678 % / `fixed` +0.003 % / `sealed` +0.007 %

```text
  pool    base (-214)       (-215)          delta
  fixed     763,686,383     763,711,694   **+0.003 %**  (noise: no contributor on those boards)
  cube    2,104,414,486   2,090,149,281   **-0.678 %**
  sealed  2,120,264,238   2,120,419,985   **+0.007 %**
  three-pool stdout identical; golden traces 7/7 unmoved
  cube rows:  dispatch_board_scan self   16.35 M -> 6.92 M
              trigger_grant_sources self 12.28 M -> 3.66 M
              dispatch_scan_card                 +2.25 M (the per-contributor body, out of line)
              lanes_after_push            4.00 M -> 3.80 M
```

`(-189)`'s device on the dispatch lane, made affordable by `(-214)`:
the lane's presence bit becomes a **member list** of the permanents
whose definition carries a `BOARD_SCAN` bit, kept exact through
membership writes like the other two, and both walkers that filled the
lane — `dispatch_board_scan` (82,718 calls, once per dispatch) and
`trigger_grant_sources` — visit only the members on a hit. A `cube`
board keeps an Equipment, a Blood Moon or a grant-trigger static out
most of the game, which is why the presence lane read `PRESENT` and
every dispatch walked the board; `fixed` and `sealed` never had a
contributor, so their lanes read empty before and after and the leg is
flat there to the noise. `dispatch_lane()` is now `members != 0`, so
`ability_strip_possible` reads the same answer it did.

### `(-214)` TAKEN — the member lists kept exact through membership writes: `fixed` -0.235 % / `cube` -0.171 % / `sealed` -0.148 %

```text
  pool    base (-213)       (-214)          delta
  fixed     765,488,207     763,686,383   **-0.235 %**
  cube    2,108,026,790   2,104,414,486   **-0.171 %**
  sealed  2,123,403,116   2,120,264,238   **-0.148 %**
  three-pool stdout identical; golden traces 7/7 unmoved
  cube rows:  dispatch_triggers_for_events self  89.15 M -> 86.71 M (its trigger-list refills)
              lanes_after_push                    3.29 M ->  4.00 M
              lanes_after_removal                 1.24 M ->  1.59 M
```

`(-213)` one step further: the two member-list lanes (`LANE_GRANT`,
`LANE_TRIGGERER`) cleared on every membership write because their
lists are indices — but a push appends at the end, so the new card's
bit is `1 << len` when it qualifies and nothing else moves, and a
removal at `i` is a shift of the bits above `i` down by one. Both are
exact, so `push` / `remove` / `pop` keep the lists; `retain` still
drops them (it cannot name what it removed), and a 65th card drops
them too (no list past 64). `member_lanes()` names the two with the
predicate each list's audit recomputes with (`card_has_any_grant_bits`,
`card_is_triggerer`), and the audits stay on every read. The refills
this saves were inline in the dispatcher (its self row moved) and in
`board_grants_keyword`; smaller than `(-213)` because the lists were
only ever asked by two callers.

**Fixed after the closing grid found it:** the removal shift was
`(bits >> (index + 1)) << index`, which for the card at index 63 of a
64-card board shifts a `u64` by 64 — a panic under overflow checks and
a silently wrong list in release. Two default-size grid cells (`cube`
seed 23, `sos` seed 11) hit it inside `remove_from_battlefield_to_
graveyard_raw`; the suite never builds a 64-card board, and the
three-pool runs never reached one. `checked_shr(..).unwrap_or(0)` is
the fix and `zone::tests::membership_writes_demote_only_the_lanes_they_
can_change` now removes index 63 of a full list. **A grid cell is the
only audit a 64-card board gets — run the grid before calling a lane
change done.**

### `(-213)` TAKEN — a membership write answers each lane off the one card it moved: `fixed` -1.158 % / `sealed` -0.887 % / `cube` -0.847 %

```text
  pool    base (-212)       (-213)          delta
  fixed     774,454,748     765,488,207   **-1.158 %**
  cube    2,126,041,074   2,108,026,790   **-0.847 %**
  sealed  2,142,409,640   2,123,403,116   **-0.887 %**
  three-pool stdout identical; golden traces 7/7 unmoved
  cube rows:  walk_and_store          14.96 M -> 1.94 M  (50,576 walks -> ~6,600)
              lanes_after_push                 +3.29 M  (new: the per-card answers)
              lanes_after_removal              +1.24 M  (new)
              board_has_mana_static    4.01 M -> 1.51 M  (its inline fill)
              dispatch_board_scan     16.40 M -> 16.35 M
```

`(-212)` kept the lanes a write could not change; this answers the
ones it could, **off the one card that moved**. Every lane is "does
some permanent's definition satisfy P", so after a push an `ABSENT`
lane is `PRESENT` iff P(new card), and after a removal a `PRESENT` lane
is still `PRESENT` if the leaver fails P (its witness is elsewhere) —
one predicate call per lane per write, against a whole-board walk per
lane per ask. The zone now holds every lane's predicate in
`LANE_PREDICATES` (indexed by lane; `None` on the two member-list
lanes, which clear on any change), the eight engine-side predicates
went `pub(crate)`, and `push` / `remove` / `pop` / `take_by_id` update
through it; `retain` cannot name what it dropped, so it demotes
`PRESENT` lanes as `(-212)` did. **The table entry and the predicate a
lane's callers hand to `lane()` must be the same function**, and the
lane audits — recomputing against the handed one on every read under
debug assertions — are what enforce it; the three zone tests that used
a foreign predicate to stand in for a lane's own were rewritten
against the real ones, and one of them found that Blood Moon carries
`STRIP` (CR 305.7) and so sets the dispatch lane, which the old test's
`never` had been hiding.

The `definition_epoch` half is settled by a census, not a build: a
throwaway test counted **0, 2 and 6** rewrites over three whole bot
games (a `fixed`-shaped one and two `cube` pairings), so the "~8 k a
run" that `card.rs`'s epoch note quotes is wrong by two orders and the
epoch is not where the fills were. The 1.9 M of walks left are the
first asks on fresh boards and the `retain` demotions.

### `(-212)` TAKEN — membership writes demote only the lanes they can change: `cube` -0.469 % / `fixed` -0.440 % / `sealed` -0.334 %

```text
  pool    base (-211)       (-212)          delta
  fixed     777,877,362     774,454,748   **-0.440 %**
  cube    2,136,059,209   2,126,041,074   **-0.469 %**
  sealed  2,149,580,101   2,142,409,640   **-0.334 %**
  three-pool stdout identical; golden traces 7/7 unmoved
  cube rows:  walk_and_store          22.44 M -> 14.96 M; 72,464 walks -> 50,576:
                the death-redirect lane's  10,116 -> 2,278
                dying_snapshot's            8,422 -> 2,148
                card_type_change_unscoped's 10,752 -> 8,006
                the SBA's                   9,068 -> 7,410
              dispatch_board_scan     16.88 M -> 16.40 M (its inline fill)
              board_has_mana_static    4.23 M ->  4.01 M (its inline fill)
  fixed:      walk_and_store           8.47 M ->  6.48 M;  sealed: 19.59 M -> 15.19 M
```

`(-209)`'s lesson turned around: the fills are the cost, so make fewer
writes cause them. Every lane is "does *some* permanent's definition
satisfy P", and a membership write moves that answer in one direction
only — **an addition can turn a lane `PRESENT` but never `ABSENT`; a
removal the reverse.** So `Battlefield::push` keeps every `PRESENT`
lane and drops only the `ABSENT` ones to `UNKNOWN`, and the removal
routes — a shadowed `remove`, `retain` and `pop`, plus `take_by_id`,
which the seven death-path `take_card(&mut self.battlefield, ..)` sites
now call — keep every `ABSENT` lane and drop only the `PRESENT` ones.
The two member-list lanes (`LANE_GRANT`, `LANE_TRIGGERER`) hold indices
and clear on either; anything else that reaches `DerefMut` clears whole
as before. Two masks (`LANE_ABSENT_BITS` = bit 0 of every field,
`LANE_PRESENT_BITS` = bit 1) make the demotion one `and`. The lanes'
`debug_assert!` audits recompute against the handed predicate on every
read, so a kept state that was wrong fails the suite; `zone::tests::
membership_writes_demote_only_the_lanes_they_can_change` pins the
contract and the two old tests that asserted the full clear now assert
the direction.

Why it pays: a death is a removal followed by a burst of asks (the
death-redirect lane, the SBA's card-type and dispatch lanes, `dying_
snapshot`'s creature lane), and on most boards those lanes read
`ABSENT` — which the removal now leaves standing. An ETB is a push
followed by the same asks, and the lanes a `cube` board keeps `PRESENT`
(dispatch, listener) stand through it. The definition-epoch bump still
throws every lane on every board away; that half is untouched and is
what the remaining 15 M of fills mostly are.

### `(-211)` TAKEN, below the bar — two standing-rule reorders in `fire_combat_damage_triggers`: `cube` -0.031 % / `fixed` -0.026 % / `sealed` -0.015 %

```text
  pool    base (-210)       (-211)          delta
  fixed     778,079,531     777,877,362   **-0.026 %**
  cube    2,136,721,312   2,136,059,209   **-0.031 %**
  sealed  2,149,902,704   2,149,580,101   **-0.015 %**
  three-pool stdout identical; golden traces 7/7 unmoved
  cube rows:  SmallVec::extend under the dispatch  20,560 calls / 1.77 M -> gone
              fire_combat_damage_triggers self     27.93 M -> 29.04 M (the loop
                                                   it absorbed; net -0.66 M)
```

The dealer walk read `c.definition.soulbond_bonus` — a pointer chase
into the definition — on every permanent before the instance's
`soulbond_partner`, which is `None` on nearly all of them (`(-116)`'s
order); and the `by_kind` buckets were a `collect()` into a `SmallVec`,
whose `Extend` is external iteration (the `(-97)` rule). Both are the
shape the Standing rules prescribe and both measured, so they stay —
and the reading is that the definition deref was not the cost: the
`Arc` is hot. **What is left is diffuse across the function's own
walks** — the dealer pass, the listener pass behind a lane that a
`cube` board with any `YourControl` trigger keeps `PRESENT`, the
cipher walk over exile, `slot()`'s linear search per trigger — 1,400 Ir
a call over 20,560 calls, 98 % of which push nothing. An early-out
would have to answer "nothing attached to the dealer" without the walk
that answers it, and `attached_to` is instance state a battlefield lane
may not hold; a line profile (`profiling-lines`) is the instrument
before anything else here.

### `(-210)` TAKEN — a graveyard lane in front of the combat-damage dispatch's per-kind graveyard walk: `fixed` -0.618 % / `cube` -0.468 % / `sealed` -0.430 %

```text
  pool    base (-208)       (-210)          delta
  fixed     782,919,572     778,079,531   **-0.618 %**
  cube    2,146,764,044   2,136,721,312   **-0.468 %**
  sealed  2,159,178,810   2,149,902,704   **-0.430 %**
  three-pool stdout identical; golden traces 7/7 unmoved
  cube rows:  fire_combat_damage_triggers self  38.75 M -> 27.93 M (1.81 % -> 1.31 %)
              card_has_graveyard_trigger        20,990 calls / 260 k Ir (the lane's fills)
  fixed:      fire_combat_damage_triggers self  12.71 M -> 7.33 M (1.62 % -> 0.94 %)
```

Read off the `(-208)` self table with the `(-207)` lesson in hand: a
1.8 % self row at ~1,900 Ir over 20,560 outer calls is walks, and the
function's phase 2 walked the dealer's controller's **whole graveyard
once per event kind** — a `contains` on the dedupe list and a
definition deref per card per kind — for a `FromYourGraveyard` trigger
that 44 cards in the catalog print and almost no graveyard holds. The
`Graveyard` zone had one lane slot left (`GY_LANE_COMBAT_TRIGGER`,
shift 6); its predicate is the definition-only scope test, and the
whole phase now sits behind `has_graveyard_trigger()`. `fixed` moved
most: its graveyards fill with the vanilla creatures it trades.

**What is left in the function (27.9 M self, 1,360 Ir a call), and
that only 420 of the 20,560 calls push anything:** the dealer walk —
one pass over the battlefield that finds the dealer, any attachment
and any soulbond pair, and reads `c.definition.soulbond_bonus` (a
definition deref) on every permanent *before* the instance's
`soulbond_partner`, which is `None` on nearly all of them (the `(-116)`
order, next); the listener walk behind the listener lane, which a
`cube` board with any `YourControl` trigger keeps `PRESENT`; the cipher
walk over exile for a `DealsCombatDamageToPlayer` kind; the `by_kind`
buckets' build and drop (3.2 M).

### `(-209)` REFUTED, one build — a strip lane under `ability_strip_in_scope`: `cube` -0.088 % but `fixed` +0.101 % / `sealed` +0.073 %

```text
  pool    base (-208)       strip lane      delta
  fixed     782,919,572     783,708,459   **+0.101 %**
  cube    2,146,764,044   2,144,879,802   **-0.088 %**
  sealed  2,159,178,810   2,160,760,531   **+0.073 %**
  three-pool stdout identical
  cube rows:  ability_strip_in_scope self  4.62 M -> 1.64 M   (-2.98 M, the walks)
              walk_and_store              22.44 M -> 24.34 M  (+1.90 M, the fills)
```

The `(-207)` shape a third time (`LANE_STRIP`, predicate `STRIP |
STRIP_ATTACHED`, the exact walk behind `PRESENT`), under
`ability_strip_in_scope` itself so every activation and the fast path's
new read took it. It lost on two pools and the table says why: **a lane
is filled by its own whole-board walk after every membership change and
every definition-epoch bump, and a lane that is asked less often than
the board changes pays more fills than it saves.** `card_type_change_
unscoped` won 0.9 % because the SBA sweep asks it after every action;
the strip question is asked once per activation, the dispatch lane the
fast path was already reading is filled by the SBA's `dispatch_board_
scan` for free, and on `fixed` / `sealed` the walk it replaced was over
boards with almost no statics. Reverted; `(-206)`'s `ability_strip_
possible` (the dispatch-lane pre-gate) stands.

**What would make a new lane free is the device this refutes toward:
one walk that fills every definition-only lane at once.** Eighteen
lanes each walk the board on their own miss, so a membership change is
up to eighteen walks; every lane predicate is a memo-word read per
permanent, so one pass could fill them all for ~18 loads a card. The
predicates live in `mod.rs` / `actions.rs` and are handed in by the
caller (that is what keeps a lane's `debug_assert!` honest), so the
batch fill needs a registered table of them in `zone.rs` — a
structural change, filed in candidates, not a lane.

### `(-208)` TAKEN — `ContinuousEffects`, the stored effect list with a fold of its modification families: `fixed` -0.092 % / `sealed` -0.068 % / `cube` -0.058 %

```text
  pool    base (-207)       (-208)          delta
  fixed     783,637,902     782,919,572   **-0.092 %**
  cube    2,148,010,997   2,146,764,044   **-0.058 %**
  sealed  2,160,644,386   2,159,178,810   **-0.068 %**
  three-pool stdout identical; golden traces 7/7 unmoved
  cube rows:  ContinuousEffects::fill   5,966 calls / 69 k Ir (the refills:
                                        2,878 under process_cumulative_upkeep,
                                        2,866 under dispatch_triggers_for_events,
                                        192 under the SBA sweep)
              card_type_change_unscoped self 2.60 M -> 2.51 M
```

Seven presence gates walked `GameState::continuous_effects` on every
ask for one `Modification` family — `card_type_change_unscoped`'s
other half, `card_color_change_unscoped`, `land_type_change_in_scope`
and `creature_type_change_in_scope` (behind a freeze-scope slot each),
`keyword_grant_in_scope`'s `AddKeyword` leg, `ability_strip_off_
battlefield`, `pt_reduction_in_scope` — and `eval.rs`'s `PrintedGates`
carried a second hand-written copy of the land and creature walks. The
list is a `CowBox<Vec<ContinuousEffect>>` with ten mutation sites, all
already behind an `iter().any` pre-check or once a turn, so
`layers::ContinuousEffects` is the `Battlefield` shape one level down:
`Deref` for every read, every `&mut` route (`DerefMut`, `push`) clears
an `AtomicU32` fold word, and the first ask after a write recomputes
`modification_families` over the list (`mod_families`: seven bits,
`TOUGHNESS_REDUCE` from the same `modification_reduces_toughness` the
gate used, so each gate's answer is exact by construction). The two
`PresenceGate` slots (`Land`, `Creature`) the fold subsumes are gone;
`PrintedGates` calls the engine's one gate per family.

**Why it is small, and why it stays.** The walks were over a list that
is empty on most boards in bot play — a six-game `cube` run refills the
fold under six thousand times, so the gates were paying a length load
and a branch, not a walk. The device's value is the other board: a
client game or a long combat with a dozen until-end-of-turn effects
scaled every one of these gates by the list's length, and now none
does. It also closed a parallel-walker pair (`eval.rs`) — the class
`ENGINE_BACKLOG` keeps closing. Measured, positive on all three pools,
kept as the structural change it is; the `(-206)` cost it was built to
take back turned out to live in `ability_strip_in_scope`'s battlefield
walk instead (that entry is corrected), which is the next lane.

### `(-207)` TAKEN — a card-type lane (the lane word widened to 64 bits) in front of `card_type_change_unscoped`'s battlefield walk: `cube` -0.912 % / `fixed` -0.764 % / `sealed` -0.622 %

```text
  pool    base (-206)       (-207)          delta
  fixed     789,673,260     783,637,902   **-0.764 %**
  cube    2,167,790,574   2,148,010,997   **-0.912 %**
  sealed  2,174,162,818   2,160,644,386   **-0.622 %**
  three-pool stdout identical; golden traces 7/7 unmoved
  cube self rows:  check_state_based_actions_into     -6.63 M  (the death sweep's
                                                               inlined copy)
                   card_type_change_unscoped          -5.28 M  (the out-of-line row:
                                                               7.89 M -> 2.60 M)
                   evaluate_requirement_static_hinted -5.22 M  (the presence-gate
                                                               copy, inlined)
                   fold_printed_grant_filter          -2.89 M
                   tap_ability_summoning_sick         -1.62 M
                   presence_gate                      -0.64 M
                   walk_and_store                     +2.71 M  (the lane's misses)
  fixed:           check_state_based_actions_into -3.04 M, evaluate_requirement_
                   static_hinted -2.36 M, card_type_change_unscoped -1.75 M,
                   walk_and_store +1.24 M
```

`(-204)` priced this at 0.36 % off the one row it could see and it came
in at 0.9 %: **the function is inlined into four of its eight callers**,
so the caller table (22,534 calls, all `activate_ability_inner`) named
a fifth of its cost. The SBA death sweep asks it once per sweep
(21 k), the requirement walker once per type-flavoured predicate, the
summoning-sickness gate once per tap — each a `continuous_effects`
walk plus a memo-bit read per permanent. The Standing rule the first
line of this entry restates: **price a small function by every caller
that inlined it, not by its row.**

The device is the `(-87)` lane, one entry further: `LANE_CARD_TYPE`
(shift 32) holds `type_bits::ALL` over the board — the definition-only
superset of `card_can_change_card_types`, whose attachment gate reads an
instance field a lane may not — and the function runs its exact walk
only when the lane says `PRESENT`, so its answer is unchanged on every
board. The word was full at sixteen lanes; `type_gates` is now an
`AtomicU64`, thirty-two lanes, `LANE_MASK` a `u64` and the three state
constants cast to match — a mechanical widening, +4 bytes on
`Battlefield` where the `AtomicU64` epoch beside it already fixed the
alignment. The `continuous_effects` half of the walk stays, and is the
next entry.

### `(-206)` RULES FIX, priced — a stripped permanent's printed mana ability no longer activates (CR 305.7 / 613.1f): `cube` +0.197 % / `sealed` +0.054 % / `fixed` +0.048 %

```text
  pool    base (-205)       (-206)          delta
  fixed     789,290,614     789,673,260   **+0.048 %**
  cube    2,163,533,978   2,167,790,574   **+0.197 %**
  sealed  2,172,992,444   2,174,162,818   **+0.054 %**
  three-pool stdout identical; golden traces 7/7 unmoved
  cube by row:  ability_strip_possible   +22,476 calls x 187 Ir = 4.20 M (new)
                  of which ability_strip_in_scope  2,872 x 428 = 1.23 M (the
                  dispatch lane PRESENT or unknown — the full walk)
                  and 19,604 x 151 = 2.97 M: the lane read plus
                  ability_strip_off_battlefield's continuous_effects walk
```

Not a perf change — the `(-204)` audit found `activate_ability_inner`'s
printed-index gate was `stripped && !is_mana_ability(..)` ("no catalog
card stripping abilities has a mana ability of interest"), so a
Blood-Mooned Temple of Epiphany tapped for `{U}` by direct activation
while the auto-tapper's source table refused it. The gate now refuses
on `stripped` for the printed leg; the granted and intrinsic legs are
unchanged, which is exactly the tapper's rule (ENGINE_BACKLOG has the
entry; `modern::decks_16_17_misc` the two regression tests). The bot
never took that path, so no trace moved.

**The cost is the fast path's, and it is priced so the next leg can
take it back.** `activate_plain_land_tap`'s board half needs "no strip
in scope" now, and the generic path's `ability_strip_in_scope` is a
~500-Ir walk. `ability_strip_possible` puts the battlefield half behind
the dispatch lane (`BOARD_SCAN` carries both strip bits, so `ABSENT`
settles it). **Corrected at `(-208)`, off its callee table:** the 4.2 M
is 80 % `ability_strip_in_scope` — the dispatch lane is `PRESENT` or
unknown on 29 % of taps (6,596 of 22,476: an Equipment with a trigger
grant, a dies-suppressor, a grant-trigger static all set `BOARD_SCAN`
bits that are not strip bits), and each of those pays the full walk at
516 Ir. The `continuous_effects` walk in `ability_strip_off_battlefield`
that this entry first blamed is nearly free in bot play (the list is
empty most of the time; `(-208)` measured it). **A dedicated strip lane
— predicate `STRIP | STRIP_ATTACHED`, the definition-only superset of
`card_can_strip_abilities` — is the device**, and `ability_strip_in_
scope` itself takes it too: candidates, top.

### `(-205)` TAKEN — the `AddMana` arm's Contamination / Pulse walk behind the mana-static lane: `cube` -0.156 % / `sealed` -0.114 % / `fixed` -0.032 %

```text
  pool    base (-204)       (-205)          delta
  fixed     789,546,300     789,290,614   **-0.032 %**
  cube    2,166,909,379   2,163,533,978   **-0.156 %**
  sealed  2,175,477,317   2,172,992,444   **-0.114 %**
  three-pool stdout identical; golden traces 7/7 unmoved
  cube by row:  run_effect self            12.51 M -> 7.94 M (-4.57 M)
                board_has_mana_static      25,166 calls -> 47,844 (2.68 M -> 4.23 M)
                is_basic                   24,278 calls, unchanged (the source read
                                           stays: the turn-scoped replacements need it)
```

`(-204)` left the resolver alone and priced its `AddMana` arm: a
whole-board `static_abilities` walk for `LandsProduceColorInstead` /
`YourBasicLandsProduceChosenColorInstead` on every land source, which
is every land tap. Engine-only: the two statics fold into the `(-198)`
memo word as `mana_summary::LAND_MANA_REPLACER` (bit 50), the lane's
predicate is now one `pub(crate) fn card_has_mana_static` — the four
`MANA_STATIC` dispatch bits *or* that memo bit — shared by the lane's
fill and its `debug_assert!` audit, and the walk runs only behind
`board_has_mana_static`. The lane is a superset for its other three
consumers, as it already was; a Contamination board now takes the
generic activation path (the fast path's board half reads the same
lane), which `core_rules::land_tap_fast_path`'s Contamination board
now pins as a decline. `fixed` moved least because its boards carry
almost no statics, so the walk it lost was over empty lists.

**What the new lane reads cost, and why it is a candidate:** the extra
22,678 `board_has_mana_static` asks are 68 Ir each, and the lane's
*hit* is a handful of loads — the average is the misses, filled inline
by `board_has_mana_static`'s own walk (two memo-word reads per
permanent under the widened predicate) after every membership change
and every definition rewrite, since `definition_epoch` is global and
one bump throws every lane on every board away. Both halves are
structural to the lane design; the epoch's over-invalidation is the
half worth a census (how many lane misses follow an epoch bump alone).

### `(-204)` TAKEN — the printed land tap settled by inspection, ahead of `activate_ability_inner`'s gate walk: `sealed` -1.623 % / `cube` -1.509 % / `fixed` -1.496 %

```text
  pool    base (-203)       (-204)          delta
  fixed     801,539,915     789,546,300   **-1.496 %**
  cube    2,200,107,698   2,166,909,379   **-1.509 %**
  sealed  2,211,363,961   2,175,477,317   **-1.623 %**
  (base re-taken from the committed tip: within 0.00001 % of the (-203)
  readings)
  three-pool stdout identical; --bench byte-identical
  (195,806 / 27.49 / 611.9 / 0 stalls); golden traces 7/7 unmoved
  cube by row:  activate_ability_inner self  46.02 M -> 12.83 M (2.09 % -> 0.59 %)
                its callee table 751,274 calls -> 384,322 — the gate walk's
                helpers: Keyword::eq 77,766 -> 10,256, ManaCost::has_x
                76,040 -> 8,612, battlefield_find 57,104 -> 6,516,
                ability_spend_kind 25,708 -> 3,232, prefers_graveyard_target
                24,882 -> 2,438, tap_ability_summoning_sick 24,540 -> 0,
                requires_target 24,382 -> 0, its own closures 51,018 -> 6,066
                CardData::mana_summary  +24,718 asks (176 k Ir); the fast
                path took 22,534 of them (91 %)
  fixed by row: activate_ability_inner self  16.61 M -> 2.77 M (2.07 % -> 0.35 %);
                8,706 of 8,770 asks taken
  sealed:       activate_ability_inner self  49.68 M -> 11.26 M (2.25 % -> 0.52 %);
                26,022 of 26,714 asks taken
```

`(-197)`'s "two-thousand-line read" was done, and the device fell out of
it: nearly every one of the ~100 gates in `activate_ability_inner` is a
question about the *ability's cost line* or the *source's printed type
line*, both pure in the definition, and the rest are five board presence
reads the generic path already pays. `activate_plain_land_tap` runs ahead
of the gate walk when the definition word says so and the board does not
object, and performs the generic path's mutations verbatim, in its order.

* **The definition half is two new families on the `(-198)` memo word**
  (bits 43-49, computed by `mana_summary_of` beside the others).
  `PLAIN_TAP << i`: printed ability `i` is a bare `{T}: Add …` for the
  activator — `plain_tap_mana`, which is `is_free`'s probe compare with
  `tap_cost` and the `AddMana` body masked, so every cost field, cap,
  gate, zone flag and reduction sits at its default. `PLAIN_LAND`: a land
  whose printed type line is neither creature, artifact nor enchantment —
  the three types the Karn / Abolisher / Cursed Totem / CR 106.12 gates
  key on. The first six indices pack; a seventh takes the generic path.
* **The board half is what the generic path reads for the same
  activation anyway**, minus the ones it reads and then ignores for a
  mana ability: yours, untapped, not detained, not bestowed;
  `card_type_change_unscoped` (layer 4 could make it a creature — CR
  106.12 and CR 602.5g both hang off that), `land_type_change_in_scope`
  only when `printed_land_mana_basic` is `Some` (CR 305.6),
  `card_keyword_possible_on(CantActivateTapAbilities)` (CR 602.5),
  `board_has_mana_static` (the Skyseer tax, the multiplier and the CR
  605.1b grant all sit behind it — a board with one goes generic rather
  than reproduce three walks), and `limited_range` (CR 801.6). Any
  `true` hands the activation back untouched.
* **The mutations are the generic path's, in its order, including the
  ones it makes for nothing**: the five pending-pick takes, the tap, the
  two events, `tapped_land_for_mana_this_turn`, the six cost-scratch
  resets (`exiled_for_cost_mana_value`, `sacrificed_count`,
  `sacrificed_total_power`, `counters_removed_as_cost`,
  `cost_discarded_mana_value`, `cost_exiled_cards`, plus the gated
  `cost_sacrificed_batch` / `tapped_for_cost` assignments), the
  multiplier around `continue_ability_resolution_x_into` and the extra
  mana splice. The resolver is untouched — Contamination, Pulse, the
  turn-scoped replacements and Bubbling Muck all resolve through the same
  code — so the win is the gate walk alone.
* **`core_rules::land_tap_fast_path` is the audit**: fourteen boards
  built twice, tapped once down each path (`FORCE_GENERIC_ACTIVATION`
  is the switch, one relaxed load an activation), the returned events
  and the whole serialized `GameState` compared; a debug-only tally says
  which path was taken so an "accept" board cannot pass by declining;
  and a 4,000-action bot game traced both ways. It found one wrong
  *expectation*, not one wrong answer: a Blood-Mooned Temple's printed
  `{T}: Add {U}` is accepted by both paths — only a basic's *intrinsic*
  ability is CR 305.6-gated, and the generic path lets a stripped
  permanent's printed mana ability through (`stripped &&
  !is_mana_ability`). That is a rules gap in the generic path (CR
  113.10b: "loses all abilities" loses mana abilities too; the
  auto-tapper's source table already gets it right), filed in TODO —
  a fix there adds `ability_strip_in_scope` to the fast path's board
  half, one presence read.

**What is left of the tap, priced off the candidate's callee table
(`cube`, 22,534 fast taps):** `continue_ability_resolution_x_into`
24,252 x 897 Ir = 21.7 M (1.0 %) — `resolve_effect_into`'s self is 238
a call (the ~30 resolution-scratch resets), and `run_effect`'s `AddMana`
arm walks every permanent's `static_abilities` for Contamination / Pulse
of Llanowar on **every land source** (`is_basic` 24,278 calls, the False
Dawn `find` 22,702) — a lane question, the two statics are not in
`MANA_STATIC`; `card_type_change_unscoped` 22,534 x 350 = 7.9 M (0.36 %,
the `continuous_effects` walk plus a memo bit per permanent — a lane,
and the lane word is full); `find_by_id_mut` 24,388 x 220 = 5.4 M (the
probe clone's CoW unshare, structural); `card_keyword_possible_on`
22,476 x 223 = 5.0 M (`keyword_grant_in_scope`'s board walk);
`board_has_mana_static` 25,166 x 106 = 2.7 M (a lane *hit* should not
cost 106 — read `mana_static_lane`); the two event pushes 4.2 M (the
first push's allocation). Candidates carries them.

### `(-203)` TAKEN — a death-redirect lane in front of the death path's four board walks: `cube` -0.459 % / `fixed` -0.299 % / `sealed` -0.268 %

```text
  pool    base (-202)       (-203)          delta
  fixed     803,947,410     801,539,784   **-0.299 %**
  cube    2,210,248,307   2,200,107,512   **-0.459 %**
  sealed  2,217,304,432   2,211,369,741   **-0.268 %**
  (base re-taken from the committed tip via stash: within 0.0001 % of
  the (-202) readings)
  three-pool stdout identical; --bench byte-identical
  (195,806 / 27.49 / 611.9 / 0 stalls); golden traces 7/7 unmoved
  cube by row:  remove_from_battlefield_to_graveyard_raw  10.8 M -> 4.9 M self
                graveyard_exile_redirects                 5.0 M -> 0.6 M
                Vec::from_iter (the hand-redirect collect) -4.2 M
                walk_and_store                            +3.9 M (the lane's
                                                          misses, see below)
```

The re-read at `966289ae` priced the death path at ~4,800 Ir a death and
said "line profile"; reading the three bodies by eye was enough. Four
whole-board `static_abilities` walks ran on every death — Valentin's
`ExileDyingOpponentCreatures`, `DiesToLibraryTopInstead`,
`DiesToOwnersHandInstead` (which also `collect`ed a `Vec` of filters to
evaluate) in `remove_from_battlefield_to_graveyard_raw`, and
`ExileCardsBoundForGraveyard` in `graveyard_exile_redirects`, which
`route_to_graveyard` asks at every graveyard placement (mills and
discards included). One definition bit answers all four:
`mana_summary::DEATH_REDIRECT` on the engine's definition-fold word
(bit 59, computed by `mana_summary_of` like `(-199)`'s two), and
`Battlefield::has_death_redirect` — `LANE_DEATH_REDIRECT`, lane 30, **the
word's last free lane** — holds the board's answer. The dying card's
*own* statics are still read (it is already off the battlefield when the
walks ask); the walks over everything else run only behind the lane.

**The miss is structural and priced:** `take_card` moves the dying card
out before the ask, so the first ask after every death is a membership
miss and walks the board once at ~390 Ir (a memo-word read per
permanent) — the +3.9 M. Asking before `take_card` does not help: the
placement's own ask comes after it either way, so it is one walk a death
whichever side asks first. What is left of the death path on `cube`:
`place_card_at_resolved_zone` ~1,260 Ir (the revert chain: face, flip,
transform, prototype, rooms, cases, `clear_effects_on_zone_change`),
`on_left_battlefield` ~1,080 Ir (`find_card_anywhere_mut` across zones
for a card that just moved, the `phased_out` / `temporary_control` /
`continuous_effects` / `delayed_triggers` walks) and the raw self ~480
(`remove_effects_from_source`, `remove_from_combat`,
`collect_leaver_counters`) — each a line read, none a lane.

### `(-202)` TAKEN — `resolve_combat`'s protection asks over the views it holds: `cube` -0.127 % / `fixed` -0.070 % / `sealed` -0.037 %

```text
  pool    base (-201)       (-202)          delta
  fixed     804,508,935     803,947,481   **-0.070 %**
  cube    2,213,055,780   2,210,248,706   **-0.127 %**
  sealed  2,218,115,166   2,217,299,276   **-0.037 %**
  three-pool stdout identical; --bench byte-identical
  (195,806 / 27.49 / 611.9 / 0 stalls); golden traces 7/7 unmoved
  cube by row:  damage_prevented_by_protection  4.92 M -> 2.20 M self
                can_grant_keyword               5.49 M -> 4.67 M
                protection_prevents_views       2.68 M -> 3.24 M (+13 k calls)
                resolve_combat self             +1.3 M (the closure inlined
                                                differently; net -2.8 M)
```

The `(-194)` shape, third application: the per-pair damage loop in
`resolve_combat` already holds the batch's `computed` slice
(`computed_of`, a slice find) and a freeze scope, and still asked
`damage_prevented_by_protection` twice per (attacker, blocker) — a nested
scope, a memo-hit `computed_permanent` of each side and, on the misses,
the presence gate's board walk. Both sites now call
`protection_prevents_views` over `computed_of(target)` /
`computed_of(source)`. The remaining 5,896 calls are the SBA's
attachment-legality sweep (CR 704.5m, Auras and Equipment via
`is_protected_from`), a different shape: under `&mut self`, no scope, and
the presence gate's `can_grant_keyword` walk is its cost — a lane
question, not a views one.

### `(-201)` TAKEN — `OftenEmpty` on `PlayerData`'s seven lists: `fixed` -0.154 % / `sealed` -0.143 % / `cube` -0.113 %

```text
  pool    base (-200)       (-201)          delta
  fixed     805,746,838     804,508,935   **-0.154 %**
  cube    2,215,563,492   2,213,055,780   **-0.113 %**
  sealed  2,221,285,766   2,218,115,166   **-0.143 %**
  three-pool stdout identical; --bench byte-identical
  (195,806 / 27.49 / 611.9 / 0 stalls); golden traces 7/7 unmoved
  fixed by row:  Arc::clone_from_ref_in self 25.65 M -> 24.41 M (-1.24 M)
                 Vec::clone under it   118,704 calls, unchanged
```

The seven plain `Vec` fields on `PlayerData` take the `(-200)` newtype;
engine-only, three sites gained an `.into()` (two tests, and
`creatures_entered_last_turn = mem::take(..)` needed nothing). A sixth of
the priced ceiling, for the same reason as `(-200)`: the seat's lists
were **inlined** into `clone_from_ref_in` too, so the guard's whole win
is that row's self cost. **The 118,704 out-of-line `Vec::clone` calls
under `make_mut_slow` (8.75 M, 1.09 % of `fixed`) still do not move** —
they are a third CoW'd owner. Read next with `--demangle=no` (How to
measure): 26,488 of them allocate and 8,900 `memcpy`, so ~80 % copy
nothing at ~45 Ir.

### `(-200)` TAKEN — cheap-on-empty clones on `CardData`, `CounterBag` and `GameState::clone`: `fixed` -0.360 % / `sealed` -0.346 % / `cube` -0.294 %

```text
  pool    base (-199)       (-200)          delta
  fixed     808,660,509     805,746,838   **-0.360 %**
  cube    2,222,094,501   2,215,563,492   **-0.294 %**
  sealed  2,228,991,395   2,221,285,766   **-0.346 %**
  three-pool stdout identical; --bench byte-identical
  (195,806 / 27.49 / 611.9 / 0 stalls); golden traces 7/7 unmoved
  cube by row:  Vec::clone           471,251 calls -> 334,757 (24.1 M -> 19.7 M):
                                     GameState::clone's 139,412 are gone
                GameState::clone     self +0.5 M (the inlined guards + to_vec)
                Arc::clone_from_ref_in self 59.0 M -> 56.4 M (-2.55 M): the
                                     five CardData lists' guards
```

`OftenEmpty<T>` (`crabomination_base::oftenempty`): a `Vec` newtype whose
`Clone` tests `is_empty()` first, `Deref`/`DerefMut`/`From`/`IntoIterator`
/`PartialEq<Vec<T>>` so its 46 call sites did not change, same size as the
`Vec`. On `CardData`'s four damage lists; `CounterBag` (already its own
type) takes the same `Clone` by hand; `GameState::clone` guards its two
lists and five `IdMap`s through `clone_list` / `clone_map`.

**What the row said that the candidate did not.** The 260,894 `Vec::clone`
calls under `Arc::clone_from_ref_in` (20.4 M, 0.92 % of `cube`) are
**unchanged** by this — `CardData`'s five lists were already *inlined*
into `clone_from_ref_in` (their whole cost was the -2.55 M off its self
row), so the out-of-line clones belong to another CoW'd owner:
`PlayerData` (`player.rs`), which carries seven plain `Vec` fields and is
unshared on every probe write to a seat. **Candidates, top: the same
device on `PlayerData`, engine-only.**

## Profile of record

### THE CACHE AND BRANCH AXIS AT THE `(-248)` TIP — the first cachegrind reading; Ir has been the only column for 248 legs

`valgrind --tool=cachegrind --cache-sim=yes --branch-sim=yes` on the
same `cube` six-game recipe and binary as the callgrind baseline
(`profiling-fast`, `--no-default-features`), read by function with the
new `scripts/cg_cache.py`. Deterministic like Ir, so it A/Bs the same
way; **what it adds is the two costs a superscalar core pays that an
instruction count cannot see**.

```text
  I  refs      1,886,902,605
  I1 misses       76,094,636   4.03 %   <- one L1i miss per 25 instructions
  D1 misses       34,108,069   3.8 %    (22.6 M rd / 11.5 M wr)
  LL misses          148,725   0.0 %    the working set fits LL; nothing is DRAM-bound
  Branches       321,176,359   (306.1 M cond / 15.1 M ind)
  Mispredicts     36,771,576   11.4 %   (10.4 % cond / 31.8 % indirect)

  I1mr by function (share of 76.1 M)          Bcm by function (share of 32.0 M cond)
   5.92 M  7.8 %  Arc::clone_from_ref_in        1.37 M  4.3 %  dispatch_triggers_for_events
   4.88 M  6.4 %  gather_continuous_effects_i.  1.31 M  4.1 %  Vec::from_iter
   4.50 M  5.9 %  dispatch_triggers_for_events  1.08 M  3.4 %  _int_malloc
   3.00 M  3.9 %  check_state_based_actions_i.  1.07 M  3.4 %  __memcpy
   2.24 M  2.9 %  perform_action_inner          1.01 M  3.2 %  check_state_based_actions_into
   1.73 M  2.3 %  Vec::from_iter                0.79 M  2.5 %  declare_blockers
   1.58 M  2.1 %  GameState::clone              0.77 M  2.4 %  gather_continuous_effects_inner
   1.41 M  1.9 %  compute_permanent_pass        0.68 M  2.1 %  declare_attackers_banded
   1.33 M  1.8 %  _int_malloc                   0.58 M  1.8 %  computed_permanent_hinted
   1.25 M  1.6 %  cast_spell_with_convoke       0.57 M  1.8 %  bot::cast_candidates
                                                0.45 M  1.4 %  do_untap  (1 per 26 Ir; (-249) took a third of its Ir)
```

**Three things this says, none of which an Ir table could.**

* **`Arc::clone_from_ref_in` — the CoW unshare's element clone — misses
  L1i once every 8.6 instructions** (5.9 M misses on 50.8 M Ir; the
  program averages one per 25). It is the cold, wide code the record
  has priced by Ir alone for a hundred passes: every `CardInstance`
  clone inlines every field's clone, the monomorphizations are many
  (the hundred-and-first pass counted eleven), and each runs briefly
  and leaves the cache. **Its wall-clock share is larger than its
  2.7 % Ir share says**, which is one more reason `(-200)`/`(-201)`'s
  "one unshare a probe" direction is the right one — and why PGO, which
  lays hot paths contiguously, reads -24 % where `-C target-cpu=native`
  reads flat (TODO's item 0): the program is front-end-bound, and
  layout is the lever the Ir ledger cannot see.
* **The mispredict rate is 11.4 % overall and 31.8 % on indirect
  branches** — 4.8 M indirect mispredicts, which is `match` over
  `Effect` / `StaticEffect` / `GameEvent` dispatched through jump
  tables plus the `dyn Decider` calls. At ~15-20 cycles each, 36.8 M
  mispredicts are of the same order as the Ir count in cycles; a
  -0.3 % Ir leg on a well-predicted path may be worth less wall-clock
  than a branch it leaves alone, and `bench_ab.py` (which resolves
  ~2 %) is the only instrument here that arbitrates. **The rate-ranked
  table (`cg_cache.py <dump> Bcm --rate`) names no large row**: the
  worst-predicted functions with ≥ 20 k branches are 30-48 % and all
  under 40 k mispredicts each (`pick_prepare_response`,
  `pick_stack_response`, `cast_spell`); the count is spread over the
  same ten wide functions the Ir table names. Not a lead on its own.
* **LL misses are nil**, so nothing here is memory-bandwidth-bound; the
  `__memcpy` row is L1/L2 traffic, and the "byte added to `GameState`"
  rule (~6,800 Ir a byte) is an instruction-count rule, not a cache
  one. `D1` misses are 3.8 % and spread.

**How to use it:** run it beside callgrind on a candidate whose Ir
reading is small but whose shape touches a hot `match` or a clone path;
a leg that moves `I1mr` or `Bcm` by more than its Ir share is one to
confirm with `bench_ab.py`. It is one more reading a run has to make,
so it is not part of the three-pool gate.

### THE ACTOR RE-READ AT `b13f5ccd` — the printed-filter pass reaches the training path: -5.6 % since `ec1bb132`, and the requirement walker has left the table

Same recipe (`profiling-fast -p crabomination_ml --no-default-features`,
`CRAB_NO_JITTER=1 selfplay_train --actors 1 --games 60 --steps 1 --seed 7`,
callgrind, `nm | grep -cE " (T|t) (_)?mi_"` 0). **2,884,346,924 Ir** against
`ec1bb132`'s 3,056,559,076 — **-5.63 %**, across the `(-176)`..`(-185)` legs
and the concurrent catalog commits, so a direction and not an A/B. 60 games,
5,805 rows, 0 stalls.

```text
            now     ec1bb132   row
  6.04 %   6.23 %   __memcpy_avx_unaligned_erms
  5.92 %   5.52 %   dispatch_triggers_for_events
  2.75 %   3.48 %   _int_free
  2.47 %   2.33 %   compute_permanent_pass
  2.37 %   2.32 %   gather_continuous_effects_inner
  2.31 %   2.35 %   computed_permanent_hinted
  2.27 %   2.55 %   _int_malloc
  2.14 %   2.68 %   malloc
  2.11 %      -     Vec::from_iter
  2.08 %      -     activate_ability_inner
  2.04 %   2.60 %   check_state_based_actions_into
  1.97 %   1.84 %   encode_state_inner       } the encoder, 3.74 %
  1.77 %   1.67 %   encode_card_object_into  }
  1.72 %   2.17 %   free
  1.68 %      -     cow::make_mut_slow
  1.46 %   1.37 %   rand_distr Normal::sample  <- net init, once per process
  1.14 %   1.08 %   recommend::rank_shape      <- the deck builder
```

`evaluate_requirement_static_hinted` is **105,958 calls** on the actor and
neither it nor `printed_requirement` is in the top forty rows; on `cube` at
the same tip the walker is ~145 k calls / 1.4 %. The allocator cluster fell
from 10.88 % to 8.88 % — a smaller total's share, not a device. The shape
otherwise holds: the encoder and the deck builder remain the only actor-only
rows, both priced in the two entries below.

### THE ACTOR RE-READ AT `ec1bb132` — the shape has not moved in fifteen more passes, and that is the finding

Same recipe as the `d0243e89` entry below (`profiling-fast -p crabomination_ml
--no-default-features`, `CRAB_NO_JITTER=1 selfplay_train --actors 1 --games 60
--steps 1 --seed 7`, callgrind). **3,056,559,076 Ir** against that entry's
3,102,072,633 — **-1.5 % over fifteen passes** — and `nm | grep -cE " (T|t)
(_)?mi_"` reads 0, so this is the system allocator.

```text
            now     then   row
  6.23 %   6.14 %   __memcpy_avx_unaligned_erms
  5.52 %   5.44 %   dispatch_triggers_for_events
  2.60 %   2.56 %   check_state_based_actions_into
  2.35 %   2.29 %   computed_permanent_hinted
  2.33 %   2.30 %   compute_permanent_pass
  2.32 %   2.28 %   gather_continuous_effects_inner
  2.15 %   2.12 %   Arc::clone_from_ref_in
  1.84 %   1.81 %   encode_state_inner   } the encoder, 3.51 % (was 3.46 %)
  1.67 %   1.65 %   encode_card_object_into
  1.37 %   1.35 %   rand_distr Normal::sample  <- net init, once per process
  1.08 %   1.06 %   recommend::rank_shape      <- the deck builder
  allocator cluster 10.88 % (was 10.71): _int_free 3.48 malloc 2.68
                                         _int_malloc 2.55 free 2.17
```

**Every row is within 0.1 points of its fifteen-pass-old value.** The actor is
not drifting, and a candidate found here will still be there next pass.

**The two entries the old reading left open are both still open and both still
priced.** `computed_permanent_hinted` is the largest `__rust_alloc` caller at
**297,560 of 1,712,673 (17.4 %)**, and **145,476 of its 491,057 calls are
`encode_state_inner`** — unchanged shares. That is `(-107)`'s third row, and
`(-111)` built and reverted the by-value form of exactly it. Nothing new to
say without a different device.

**Three rows read this pass and ruled out, so nobody re-reads them:**

* **`__memcpy` is still diffuse.** 1,741,124 calls over the whole program; the
  largest single caller is `GameState::clone` at 10.8 M of 190 M — **5.7 %**.
  `(-92)`'s "stop looking for a hot line" holds.
* **The `{:?}` formatting traffic is `wants_converge`, and it does not scale.**
  `core::fmt::write` reads 10,746,402 Ir inclusive (0.35 %) with
  `DebugStruct::field` the dominant caller; `wants_converge` alone is
  **9,643,168 inclusive (0.32 %)**, i.e. essentially all of it. It is once per
  distinct card name per *process* — charged in full to a 60-game dump and
  ~nothing to a 30 k-game run, exactly as the "How to measure" warning says.
  **A change that moved it would read as a third of a percent no real run ever
  sees.**
* **`rank_shape` is 1.08 % and it is one deck build a game**, 6,840 calls of
  which 6,720 come from `lattice` — 112 shapes ranked per game, which is the
  lattice doing its job. Its own callees are `static_build_score` (0.27 %) and
  its allocations; there is no hot line under it and no memo, because every
  shape is a different `(colors, splash, spells)`.

**The ratio device against `cube` at the same tip** (`cg_ratio.py actor
cube --floor 0.45`; the totals do not compare, the shares do): **below
`__memcpy`'s 2.25x nothing exceeds 1.30x**, and the 89x
`small_sort_general_with_scratch` row of the old reading is gone — it was
taken. `effective_mana_abilities_into` 1.30, `finalize_cast` 1.19,
`event_matches_spec` 1.14, then the allocator cluster at 1.09-1.14. **The
actor and `cube` now agree to within 30 % on every row but one**, which is
the strongest form of the old entry's finding: there is no actor-only
candidate left above the floor except the encoder and the deck builder, and
both are priced above.

### THE ACTOR RE-READ AT `d0243e89` — fifteen passes on from `bb67895a`, and the encoder is now the largest caller of the largest allocation row

`(-123)`. `profiling-fast -p crabomination_ml --no-default-features`,
`CRAB_NO_JITTER=1 selfplay_train --actors 1 --games 60 --steps 1 --seed 7`,
callgrind. **3,102,072,633 Ir**, and `grep -c "mi_\|_mi_"` on the dump reads
0, so this is the system allocator and not mimalloc under valgrind.

```text
  6.14 %  __memcpy_avx_unaligned_erms          <- the largest row on the actor
  5.44 %  dispatch_triggers_for_events         (5.18 % of cube, after (-121))
  3.42 %  _int_free  } the allocator cluster: 10.71 %, against cube's 9.68
  2.64 %  malloc     }
  2.51 %  _int_malloc}
  2.14 %  free       }
  2.56 %  check_state_based_actions_into
  2.30 %  compute_permanent_pass
  2.29 %  computed_permanent_hinted
  2.28 %  gather_continuous_effects_inner
  2.12 %  Arc::clone_from_ref_in
  1.81 %  encode_state_inner        } the encoder, 3.46 %, and `bot_ladder`
  1.65 %  encode_card_object_into   } runs it on NO pool
  1.35 %  rand_distr Normal::sample <- candle's net init, see below
  1.06 %  recommend::rank_shape     <- the deck builder, one build per game
```

**`(-107)`'s remaining half is PROMOTED, and the promotion is a number.** The
allocation table (1,712,610 `__rust_alloc` calls over 60 games) is topped by
`computed_permanent_hinted` at **297,560 — 17.4 % of every allocation the
actor makes**, against 12.83 % when `(-107)` filed it. Total allocations are
flat over the same window (1,712,610 against `bb67895a`'s 1,690,418 per 60
games, +1.3 %), so the share moved because the row grew **+37 %** in absolute
terms. It is the largest single allocation caller in the program.

**And its caller table says why, in a way the old entry could not**: of its
491,057 calls, **145,476 (29.6 %) are `encode_state_inner`** — a caller that
exists on no `bot_ladder` pool. The next four are `permanent_value_with`
58,154, `legal_blockers` 46,980, `damage_prevented_by_protection` 45,498,
`pick_blocks_inner` 28,181. **Anyone sizing `(-107)` off a `cube` dump is
sizing 70 % of it.**

**`(-107)`'s encoder-growth half is CONFIRMED CLOSED by the same dump.** The
single backing buffer at `d402e5da` was priced at ~5.25 `do_reserve_and_handle`
growths per encoded state; it now reads **6,282 growths over 6,282 states =
1.00**. `encode_state_inner`'s callee table, per state: 35.6
`encode_card_object_into`, 23.2 `computed_permanent_hinted`, 5.6
`affordable_covered`, 2.0 `can_player_play_land`, 1.0 `with_frozen_layers`,
1.0 growth.

**Two rows that look like leads and are not.**
* **`candle`'s `rand_normal` (1.35 % self, 58.4 M inclusive over 859,200
  samples) is the net's weight *initialisation*** — once per process, charged
  here to 60 games. `(-95)`'s rule exactly: a short workload charges every
  once-per-process cost to it. A real run plays millions of games and this
  vanishes. **Do not optimise it.**
* **`__memcpy` is the largest row and it is diffuse.** 1,743,449 calls over
  **2,319 callers**, of which 43.6 % are below the listing's cut; the top row
  is `GameState::clone` at 10.8 M of 190 M. `(-92)`'s "stop looking for a hot
  line" applies.

**The ratio device against `cube` at the same tip** (`cg_ratio.py actor cube
--floor 0.45`; the two totals do not compare, the shares do):

```text
  actor%  cube%      x   row
   0.46    0.01   88.98  small_sort_general_with_scratch   <- TAKEN, see the Log
   6.14    2.72    2.25  __memcpy_avx_unaligned_erms
   0.62    0.48    1.30  effective_mana_abilities_into
   ...nothing else above 1.2x...
  five rows with NO cube cost at all:
   1.81  encode_state_inner      1.65  encode_card_object_into
   1.35  candle rand_normal      1.06  recommend::rank_shape
   0.49  rand_chacha refill_wide
```

**The table is the finding: below the top two rows the actor and `cube` agree
to within 30 %.** The actor is not a different program — it is `cube` plus an
encoder, a deck builder and a determinizer, and those three are the only
places a candidate can live that the bench cannot price. The 89x row was one
of them and is taken; `rank_shape` and the encoder are what is left.

⚠ **THE ACTOR'S AMBIENT CODEGEN BAND IS MUCH WIDER THAN `bot_ladder`'s, AND
`(-110)`'s NULL CONTROL DOES NOT TRANSFER.** That control read +0.006 % for a
no-op on three `bot_ladder` pools. Here, extracting four identical statements
into a named helper — a change with *no* release-side semantics — moved
`compute_permanent_pass` and a `FilterMap` by +3.6 M and the total by
**+0.18 %**. `crabomination_ml` builds at `codegen-units = 16`, so any edit to
a hot file can repartition it. **Attribute an actor change to its own rows
before believing its total**; a whole-program delta under ~0.2 % on this
workload is not a measurement on its own.


### ALL THREE POOLS RE-READ AT `cab8d5d7`, AFTER `(-120)`/`(-121)`

`release-fast --no-default-features`, callgrind, six games, one thread,
seed 1. Totals `fixed` 847,467,337 / `cube` 2,529,883,427 / `sealed`
2,572,625,951 — an independent rebuild that reproduces the `(-121)` A/B's
candidate column to **77 / 1,179 / 1,072 Ir**, i.e. under 2 parts per million.
That is the cheapest confirmation of an A/B on file and it is worth taking
whenever a profile follows one.

⚠ **THE ALLOCATOR CHECK IN "How to measure" IS NOT `grep mimalloc`.**
mimalloc's symbols are `mi_*` / `_mi_*` and it is statically linked, so
`nm -C <bin> | grep mimalloc` returns **zero on a mimalloc binary** and
`grep libmimalloc <dump>` returns zero too — the object is `bot_ladder`. The
hundred-and-ninth pass profiled a default-features binary behind exactly that
check and read `fixed` / `cube` / `sealed` **6.7 / 7.1 / 7.4 % low**, which is
this file's documented ~11 % and looked like a free win on the control pool.
Use one of these instead, both of which are positive tests:
```text
nm <bin> | grep -cE " (T|t) (_)?mi_"      # 0 = system allocator
grep -c 'fn=.*mi_' <dump>                 # 0 = the dump is a system-alloc run
```
**A check that returns zero for the thing it is looking for AND zero when it
is broken is not a check.** The row tables are the backstop: `_int_malloc` /
`_int_free` / `malloc` / `free` mean glibc, `mi_theap_malloc_aligned` means
mimalloc.

```text
row                                                   fixed%   cube%  sealed%
dispatch_triggers_for_events                            5.43    5.18     7.09
gather_continuous_effects_inner                         3.32    3.27     2.75
_int_free                                               3.46    3.14     3.46
layers::compute_permanent_pass                          2.52    3.12     2.19
check_state_based_actions_into                          3.10    2.86     3.46
__memcpy_avx_unaligned_erms                             2.30    2.72     3.44
Vec::SpecFromIterNested::from_iter                      2.41    2.57     2.28
malloc                                                  2.67    2.40     2.68
computed_permanent_hinted                               1.91    2.25     1.90
_int_malloc                                             1.73    2.19     2.17
Arc::clone_from_ref_in                                  2.65    2.03     2.47
free                                                    2.22    1.95     2.20
activate_ability_inner                                  1.89    1.75     1.93
perform_action_inner                                    2.36    1.48     1.94
fire_combat_damage_triggers                             1.22    1.48     1.30
evaluate_requirement_static_hinted (3 instances)        1.87    3.81     2.54
event_matches_spec                                      0.39    1.26     1.82
GameState::clone                                        1.62    1.14     1.44
```

**Two things this table says that the last one did not.**

(a) **The allocator is now the largest single cluster on every pool** —
`_int_free` + `malloc` + `_int_malloc` + `free` is **9.68 % of `cube`**,
10.08 % of `fixed`, 10.51 % of `sealed`, before counting the rows that feed it
(`Arc::clone_from_ref_in` 2.03, `Vec::from_iter` 2.57, `__memcpy` 2.72). That
is `(-80)`'s finding still standing, and `(-107)`'s `computed_permanent_hinted`
`Arc`s (2.25 % of `cube` in self alone) are the largest named contributor.

(b) **`dispatch_triggers_for_events` is still the top row but `sealed` is now
where it costs most** (7.09 %, against `cube`'s 5.18 and `fixed`'s 5.43) —
inverted by this pass, which took 53.9 % of `cube`'s walk to 28.3 % and left
`sealed` untouched at 32.5 %. `sealed` has zero grants and an 86 % lane hit
rate, so what is left there is the **per-event bookkeeping switch**, exactly
as the standing rule says. The walk is done; the body is not.

### THE ACTOR RE-READ AT `bb67895a` — AND THE DECK BUILDER IS BYTE-IDENTICAL, WHICH MAKES THIS THE CLEANEST ACTOR COMPARISON ON FILE

`selfplay_train --actors 1 --games 120 --steps 1 --seed 7 --out <dir>`,
`profiling-fast -p crabomination_ml --no-default-features`, callgrind, 0
`libmimalloc` frames. **6,020,307,568 Ir** against `c92f3851`'s
6,113,733,616 — **-1.53 %** over a window carrying this session's three perf
commits plus the other session's card work.

**`rank_shape` reads 65,739,164 at both tips, to the instruction.** That row
is the deck builder, which runs twice a game, so an identical count says the
actor built the same decks and played the same length of workload — the
control `(-95)` and `(-97)` both wanted and neither had. Read the totals here,
not only the shares.

```text
                                 c92f3851        bb67895a          share
  dispatch_triggers_for_events  383,520,709   377,500,872   6.27 %  ->  6.27 %
  __memcpy_avx_unaligned_erms   311,528,687   305,745,219   5.10 %  ->  5.08 %
  _int_free                     221,370,934   215,901,283   3.62 %  ->  3.59 %
  _int_malloc                   192,622,796   190,040,391   3.15 %  ->  3.16 %
  malloc                        166,300,976   164,249,979   2.72 %  ->  2.73 %
  check_state_based_actions_into161,538,532   158,992,910   2.64 %  ->  2.64 %
  free                          133,916,685   131,698,755   2.19 %  ->  2.19 %
  gather_continuous_effects_in. 130,935,400   130,319,040   2.14 %  ->  2.16 %
  Arc::clone_from_ref_in        131,634,351   130,143,309   2.15 %  ->  2.16 %
  Vec::from_iter (nested)       128,956,156   127,187,568   2.11 %  ->  2.11 %
  activate_ability_inner        120,350,979   116,887,939   1.97 %  ->  1.94 %
  compute_permanent_pass        108,543,362   109,817,412   1.78 %  ->  1.82 %
  computed_permanent_hinted      99,413,876    98,082,453   1.63 %  ->  1.63 %
  encode_state                   93,537,391    92,383,517   1.53 %  ->  1.53 %
  encode_card_object_into        91,569,647    91,710,690   1.50 %  ->  1.52 %
  rank_shape                     65,739,164    65,739,164   1.08 %  ->  1.09 %
```

**`compute_permanent_pass` LEAVES THE ALLOCATION TABLE ENTIRELY, which is
`(-106)` confirmed on the workload.** It was the largest single allocation
context on `bot_ladder --decks cube` (61,518 of 1,384,794); at this tip it
does not appear among the actor's `__rust_alloc` callers at all, and its own
`do_reserve_and_handle` row is gone. Its *self* Ir rises 1.78 -> 1.82 % —
the inline-or-spilled branch, exactly as the three-pool A/B priced it.

**`encode_state` falls 1.23 % of itself while its sibling
`encode_card_object_into` rises 0.15 %**, which is the only reading available
for `244e849b`'s actor-only site (the 768-byte `IntoIter` in that function).
It is a differential against an unchanged neighbour, not an A/B; treat it as
consistent-with rather than measured.

**THE ACTOR'S ALLOCATION CENSUS, 3,380,837 CALLS OVER 120 GAMES, AND TWO OF
THE TOP ROWS ARE INVISIBLE TO EVERY `--bench` POOL:**

```text
  callers of __rust_alloc                     calls
  656,028  finish_grow                                (the growth path)
  433,775  computed_permanent_hinted                  the Arc<ComputedPermanent>
  346,535  Arc::clone_from_ref_in                     the CoW deep copy
  323,781  Vec::from_iter (nested)
  186,395  Vec::clone
  136,278  GameState::clone
   96,193  gather_continuous_effects_inner
   93,120  CowBox<Vec<T>>::push

  callers of do_reserve_and_handle             calls        Ir (incl)
   71,045  Vec::from_iter (nested)             71,045      39,917,447
   66,485  encode_state                        66,485      25,503,509   <- actor-only
   51,112  auto_tap_for_cost_inner             51,112      31,820,359
   35,956  mana_source_table                   35,956       4,775,973
   10,828  resolve_combat                      10,828      20,934,769
```

`encode_state` is **the second-largest `reserve` grower in the program and
`bot_ladder` cannot see it**: 66,485 growths over 12,660 encoded states, i.e.
~5.25 group `Vec`s allocated per state, and `EncodedState::default` builds
`NUM_GROUPS` empty `Vec`s that the reserves then have to allocate. See
`(-107)`.

### THE ACTOR RE-READ AT `c92f3851` — THE CoW COPY FAMILY FALLS BY THE SAME 41 % THERE AS ON `bot_ladder`

`selfplay_train --actors 1 --games 120 --steps 1 --seed 7 --out <dir>`,
`profiling-fast -p crabomination_ml --no-default-features`, callgrind.
**6,113,733,616 Ir** against `(-97)`'s 6,148,474,954 at `633acc3e`; 120 games,
11,893 rows, 0 stalls. The two runs are **not** the same workload — four card
fixes from the other session land in that window and a card fix changes the
games a cube actor plays — so read the *shares*, not the total.

```text
                                 633acc3e (-97)      c92f3851        share
  dispatch_triggers_for_events   374,722,938 6.09%  383,520,709  6.27 %
  __memcpy_avx_unaligned_erms    308,368,422 5.02%  311,528,687  5.10 %
  _int_free                      223,754,938 3.64%  221,370,934  3.62 %
  Arc::clone_from_ref_in         223,610,691 3.64%  131,634,351  2.15 %  <--
  _int_malloc                    202,102,545 3.29%  192,622,796  3.15 %
  malloc                         169,155,800 2.75%  166,300,976  2.72 %
  check_state_based_actions_into 158,048,663 2.57%  161,538,532  2.64 %
  free                           135,489,984 2.20%  133,916,685  2.19 %
  Vec::from_iter                 127,359,932 2.07%  128,956,156  2.11 %
  gather_continuous_effects_in.  127,171,806 2.07%  130,935,400  2.14 %
  activate_ability_inner         118,601,411 1.93%  120,350,979  1.97 %
  compute_permanent_pass         108,092,295 1.76%  108,543,362  1.78 %
  computed_permanent_hinted      102,372,062 1.66%   99,413,876  1.63 %
  encode_state                    93,895,348 1.53%   93,537,391  1.53 %
  encode_card_object_into         89,840,110 1.46%   91,569,647  1.50 %
  rank_shape                      66,637,955 1.08%   65,739,164  1.08 %
```

**Fifteen of the sixteen rows are flat to within 0.1 points and one moved:
the deep copy, 3.64 % -> 2.15 %, i.e. -41.1 %.** `bot_ladder --decks cube`
read the same family 3.28 % -> 1.93 % over the same window, **-41.2 %**. So
`(-97)`'s "an engine percent is an actor percent" is now confirmed for a
*change* and not only for a *level* — the width and count levers of `(-100)`
transfer to the workload the branch actually exists to run, at the same rate,
and nothing about the actor's extra copying (MCTS state, the encoder) dilutes
them.

**And the flat fifteen are the control.** A shifted workload would move
`encode_state` and `rank_shape` — the two rows that are pure actor and
proportional to games and decks — and they read 1.53 -> 1.53 % and
1.08 -> 1.08 %. The card fixes changed which games are played, not how many.


### THE THREE POOLS RE-READ AT `b6218fad`, AFTER THE `CardData` PASS

The copy family it took apart is no longer in the top ten of any pool:
`Arc::clone_from_ref_in` was 3.28 % of `cube` and is **1.92 %**. What is
left, self costs, with each row's status so nobody re-derives one:

```text
                      fixed     cube    sealed
  dispatch_triggers..  7.33 %   7.19 %   7.84 %   (-59)/(-90): no hot line,
                                                   mask ceiling 0.86 %
  the allocator family 10.44 % 10.25 %  10.78 %   1,392 k allocs; UPPER BOUND
   (malloc/_int_malloc/free/_int_free)             — callgrind runs the system
                                                   allocator, the ship is mimalloc
  gather_..._inner     3.15 %   3.09 %   2.64 %   (-81), closed door
  check_state_based..  2.91 %   2.69 %   3.36 %   (-69)/(-88)
  __memcpy             2.23 %   2.76 %   3.40 %   diffuse, ~355 callers
  compute_permanent_p. 2.32 %   2.80 %   2.13 %   (-92) lead 2
  Vec::from_iter       2.47 %   2.60 %   2.37 %   ~93 callers — DIFFUSE
  clone_from_ref_in    2.51 %   1.92 %   2.40 %   (-100), both halves taken
  computed_permanent_h  —       2.23 %   1.94 %   (-27)'s pool refuted
  activate_ability_in.  —       1.69 %    —       the tap; (-51)(a)'s make_mut
                                                   half is now gone from it
```

**The allocator is the largest family on every pool and it is where the next
census belongs.** 1,392,000 allocations on `cube`, and `finish_grow` — a
`Vec` that outgrew itself, i.e. a *re*allocation a reserve would remove
outright rather than move — is **361,249 of them, 26 %**, 35.7 M Ir
inclusive (1.34 %). Its `grow_one` callers at this tip:

```text
  60,974  Vec::push_mut          (out of line; the growths belong to ITS callers)
  29,474  dispatch_board_scan
  21,334  resolve_combat
  13,604  mana_source_table
  13,104  deal_combat_damage_to_target
  12,996  bot::pick_attacks_inner
  11,410  grant_scan
  10,434  affected_from_requirement
   9,146  effective_mana_abilities_into
   8,286  granted_abilities_of_inner
   8,262  CowBox<Vec<T>>::push
```

**Read `(-80)`'s row 2 before pricing any of these**: a first-push allocation
is *moved* by a `with_capacity`, not removed, and only a *re*growth is
removed. `grow_one` is the regrowth path, so this table is the right one —
but `(-80)`'s other finding stands too (an allocation count is not a cost:
84,558 allocations removed made the program slower). Rank by the Ir on the
edge, size the reserve off the observed length, and expect a third of the
row.


### THE WHOLE-PROGRAM LINE PROFILE, TAKEN FOR THE FIRST TIME (ninety-sixth pass, `cd0842e9`, `--decks cube`)

Every earlier line profile in this file was `--in <function>` scoped, so
nobody had ever asked what the *program's* hottest line is. **It is 0.97 %,
and 11,165 lines hold 86 % of the run.** The profile is flat, and that is
the most useful thing this pass measured.

```text
cargo build --profile profiling-lines -p crabomination --bin bot_ladder \
  --no-default-features                                    (cold, ~9 min)
RUST_MIN_STACK=33554432 valgrind --tool=callgrind --dump-instr=yes \
  --callgrind-out-file=cg.instr.out target/profiling-lines/bot_ladder \
  --a gang --b gang --games 6 --threads 1 --seed 1 --decks cube
python3 scripts/cg_lines.py cg.instr.out target/profiling-lines/bot_ladder
2,865,631,615 Ir against profiling-fast's 2,865,614,181 — the two inline
identically, which is what makes the attribution transferable.

  23,867,576  0.97 %  ptr/mod.rs:1917      Arc::clone_from_ref_in  (the CoW copy)
  16,683,846  0.68 %  game/mod.rs:?        dispatch_triggers_for_events
  12,367,872  0.50 %  effects/eval.rs:3189 the requirement walker's `match req`
  11,216,374  0.46 %  iter/macros.rs:349   computed_permanent's battlefield find
  10,814,250  0.44 %  iter/macros.rs:332   the SBA sweep's own walks
  10,332,090  0.42 %  game/mod.rs:2661     `impl Clone for GameState`'s `Self {`
```

**Three of those six are a `match` arm dispatch or a struct move and have no
cheaper form**: `eval.rs:3189` is 8.7 Ir a call over 1,425,996 calls,
`game/mod.rs:2661` is 455 Ir over 22,684 clones of a ~110-field struct, and
`game/mod.rs:15582` (0.32 %, not shown) is `perform_action_inner`'s
`GameAction` match. Those are floors.

**BY SOURCE FILE, AND THIS IS THE FRAMING NUMBER THE FILE HAS BEEN MISSING:**

```text
  377,262,930  13.17 %  iter/macros.rs     <- core's SLICE ITERATOR
  295,410,894  10.31 %  game/mod.rs
  242,965,358   8.48 %  ptr/non_null.rs    <- the pointer stepping under it
  142,661,390   4.98 %  vec/mod.rs
  105,561,348   3.68 %  raw_vec/mod.rs
   99,079,474   3.46 %  src/card.rs        (base: CardId::eq, the memo reads)
   97,205,165   3.39 %  ptr/mod.rs
   76,271,460   2.66 %  game/actions.rs
   75,219,446   2.62 %  effects/eval.rs
   73,580,970   2.57 %  game/layers.rs
   62,956,784   2.20 %  src/option.rs
   58,806,668   2.05 %  game/stack.rs
   52,688,420   1.84 %  server/bot.rs
   49,661,734   1.73 %  src/sync.rs   +  1.41 % sync/atomic.rs  (Arc refcounts)
   41,674,446   1.45 %  src/alloc.rs  +  0.50 % alloc/unix.rs
```

**Slice iteration, the pointer stepping under it and the `Vec` machinery
around it are ~30 % of the program.** No engine source file is above 10.3 %.
That is the shape of a simulator that walks collections, and it says the
lever is **fewer element visits**, not a faster body — which is exactly what
every win of the last ten passes has been (presence gates, per-definition
memos, fused walks, `_on` forms that skip a `find`).

### FIVE FUNCTIONS READ BY LINE ACROSS THREE PASSES, AND ALL FIVE SAY "NO HOT LINE"

Recorded together so nobody spends a sixth cold build to learn it a sixth
time:

```text
  dispatch_triggers_for_events  6.68 %   largest line 0.23 %   (-59), pass 89
  compute_permanent_pass        2.80 %   largest line 0.21 %   pass 96
  check_state_based_actions_into 2.46 %  largest line 0.44 %   pass 96
  resolve_combat                1.86 %   largest line 0.23 %   pass 96
  fire_combat_damage_triggers   1.49 %   largest line 0.23 %   pass 96
  activate_ability_inner        1.68 %   largest line 0.21 %   pass 96
```

Every one of them is a sequence of gated whole-collection walks, and in every
one the top rows are `iter/macros.rs`, `ptr/non_null.rs` and the struct move
at the end. **`compute_permanent_pass`'s own top rows are the
`ComputedPermanent { .. }` construction at line 1017 and the `Vec`/overlay
moves under it — ~30 M / 1.05 % of `cube` to materialise and move the
struct**, which is the one of these six with a *shape* worth attacking
rather than a call count. (-13) already measured the husk-pool answer at
+2.60 %, so it is not a pool; it would have to be construction in place.

**Do not run another `--in` line profile on a function of this shape.** Read
`cg_calls.py` for its call count and `cg_contexts.py` for whose calls they
are; those are the two questions a flat profile can still answer.


### THE ACTOR'S PROFILE, RE-READ AT `8beed408` (ninety-eighth pass)

The first re-read since the ninety-sixth pass took it, and the shape holds:
**the actor is the bot's attack search wrapped around the engine, and the
engine rows are the same ones `bot_ladder` shows.** 3,415,123,660 Ir.

```text
cargo build --profile profiling-fast -p crabomination_ml --bin selfplay_train \
  --no-default-features
CRAB_NO_JITTER=1 RUST_MIN_STACK=33554432 valgrind --tool=callgrind \
  --callgrind-out-file=cg.actor.out target/profiling-fast/selfplay_train \
  --actors 1 --games 60 --steps 1 --seed 7 --out /tmp/actorprof
(`grep -c libmimalloc cg.actor.out` = 0, so the allocator is the system one.)

INCLUSIVE — the subtree table is the one to read here
  94.12 %  play_recorded_game_mcts
  78.74 %  HeuristicBot::next_action
  63.24 %  perform_action_inner
  46.57 %  pick_attacks_scored          <- the largest single subtree
  46.24 %    simulate_attack_outcome_once   2,329 candidates, 678,088 Ir each
  28.95 %  main_phase_action_with
  21.56 %  sim_step                       74,423 calls
  17.68 %  affordances::accept_on         13,426 probe clones
  15.02 %  Vec::from_iter (nested)
  14.14 %  perform_action                 8,347 checkpoint clones
   9.41 %  try_pay_after_snapshot_mode
   8.80 %  auto_tap_for_cost_inner
   7.32 %  with_frozen_layers

SELF
   6.02 %  dispatch_triggers_for_events   \  the same four rows, the same
   5.29 %  __memcpy                        |  order, as every `bot_ladder`
   3.47 %  Arc::clone_from_ref_in          |  pool profile
  11.3  %  the allocator family           /   1,822,999 allocations
   2.50 %  check_state_based_actions_into
   2.07 %  Vec::from_iter (nested)
   1.98 %  gather_continuous_effects_inner
   1.76 %  compute_permanent_pass
  ---- and the rows `bot_ladder` does not have ----
   1.45 %  encode_card_object             228,515 objects, 217 Ir each
   1.30 %  encode_state
   0.98 %  rank_shape                     deck building; scales with games
   1.03 %  rand_distr normal sample       722,816 calls — NET WEIGHT INIT
```

**`rand_normal` has grown from 22 calls to 722,816 and it still does not
scale.** The ninety-sixth pass's note said "22 calls, i.e. net weight init,
ignore"; the net is bigger now, so the row is bigger, and it is still once per
process. Re-deriving that costs a reader ten minutes — the rule it belongs to
is "size a row by whether it scales with games", and this row is the standing
counter-example.

**`__memcpy` is 5.29 % here against ~2.5 % on `cube`, and the extra is still
the recorder**: `play_recorded_game_mcts` 273,163 calls, `encode_state`
241,273, `encode_card_object` 228,515 — the `EncodedObject`s being built on
the stack and copied into their group `Vec`s. An `EncodedObject` is
`{ u16, [f32; 53] }` = 216 bytes and is built by value and then pushed, so
each one is copied twice. **That is the only actor-only lever this profile
names**, and it is worth ~0.2-0.3 % of the actor: an `_into(&mut EncodedObject)`
form would remove one of the two copies. Everything above it is engine.

**And the framing number: `ManaCost::cmc` is 452,805 calls / 11.8 M / 0.34 %
here against 149,660 / 0.13 % on `cube`, because `encode_card_object` asks it
228,515 times.** A `CardMemo` slot for it is *not* the answer — `(-87)` measured
a new memo family at `fixed` +0.135 % for widening the miss path of every
other consumer of that word — but it is the shape of the difference between
the two workloads.

### THE ACTOR'S PROFILE, READ FOR THE FIRST TIME (ninety-sixth pass, `599825ba`)

`selfplay_train` is the workload the ML phase actually runs, and every
"Profile of record" block below it is `bot_ladder`. They are not the same
program: **three of the actor's top rows have no row at all in any of the
three `bot_ladder` pool profiles**, so nothing in this file had ever priced
them.

```text
CRAB_NO_JITTER=1 RUST_MIN_STACK=33554432 valgrind --tool=callgrind \
  --callgrind-out-file=cg.actor.out target/profiling-fast/selfplay_train \
  --actors 1 --games 60 --steps 1 --seed 7 --out /tmp/actorprof
3,493,965,685 Ir.  (`-p crabomination_ml --no-default-features` — that crate
has its own mimalloc default; check the dump for `libmimalloc` frames, 0 here.)

  5.88 %  dispatch_triggers_for_events        \
  5.06 %  __memcpy                             |  shared with bot_ladder,
  3.45 %  Arc::clone_from_ref_in               |  same order, same shape
 10.7  %  the allocator family                /
  ---- and then the rows bot_ladder does not have ----
  5.72 %  encode_state, INCLUSIVE (6,378 calls, 31,367 Ir each)
            of which encode_card_object 1.94 % over 228,515 objects
            and mana_source_table       0.93 % over 12,756 calls
  1.84 %  build_random_deck -> recommend::lattice (120 calls, one per deck)
            of which rank_shape 0.94 % over 6,840 calls (56 shapes a deck)
  1.41 %  candle rand_normal — 22 calls, i.e. NET WEIGHT INIT. Does not
          scale; it is 1.4 % of a 60-game run and ~0 of a real one. Ignore.
```

**`__memcpy` is 5.06 % here against 2.48 % on `cube`**, and the extra is the
recorder: `play_recorded_game_mcts` 273,163 calls, `encode_state` 241,273,
`encode_card_object` 228,515 — i.e. the `EncodedState` rows being built and
moved. That is the actor's own cost and no `--bench` reading will ever show
it.

**Two rules for reading an actor profile, both learned here:**

* **Size a row by whether it scales with games.** `rand_normal` is 1.41 % of
  this run and zero of a training run; `lattice` is per *deck*, so it scales
  with games; `encode_state` is per recorded position, so it scales with
  rows. A 60-game run over-weights everything that happens once.
* **`--feature-census` is the encoder's regression check, and it was not
  reproducible until this pass.** Two runs of `--feature-census 8 --seed 5`
  on one binary disagreed by 12 positions and 515 objects out of 820 /
  29,143, because `play_recorded_game` left `bot::jitter_below` on the
  unseeded thread-local RNG. It now seeds per game. **With it pinned, a diff
  of two census outputs is a byte-level "did the encoding move" answer over
  ~29 k encoded objects** — which is what the encoding caution has always
  needed and never had. Use it on any change that touches `encode.rs`.


Callgrind on `profiling-fast --no-default-features` (= `release-fast` opt
settings + debuginfo; system allocator, because valgrind replaces malloc and
a mimalloc build would measure the interception), 1 thread, `--a gang --b
gang --games 6 --seed 1 --decks fixed`.

### THE ALLOCATION TABLE at the eighty-sixth base (`c8ebea50`), `--decks cube`

The table that has found the most in this file — callers of `__rust_alloc`
ranked by **call count**, not Ir. Re-read after five passes; two rows the
eighty-first tip's copy carried are gone.

```text
1,832,924 allocations (was 1,988,682 at the eighty-first tip)
  495,777  RawVecInner::finish_grow        27.0 %   a Vec that outgrew itself
  244,032  Arc::clone_from_ref_in          13.3 %   the CoW deep copy
  227,430  GameState::computed_permanent   12.4 %   the Arc<ComputedPermanent>
  176,719  Vec::from_iter (nested)          9.6 %
   82,830  Vec::from_iter (nested)'2        4.5 %
   76,612  gather_continuous_effects_inner  4.2 %   `all_effects`'s with_capacity
   56,414  <GameState as Clone>::clone      3.1 %   2.49 per clone
   52,138  PrintedList::push                2.8 %
   49,096  hashbrown fallible_with_capacity 2.7 %
   48,545  Vec::clone                       2.6 %

callers of `grow_one` (546,800 of the 654,597 `finish_grow` calls)
  69,896  gather_continuous_effects_inner  14,105,371 Ir   0.42 %  <- (-71), TAKEN
  62,632  Vec::push_mut                    11,798,939      0.35 %
  37,670  stack::advance_step               4,522,660
  36,526  combat::declare_blockers          7,966,197      0.24 %
  29,474  GameState::dispatch_board_scan    3,544,522
  26,942  check_state_based_actions         4,718,160
  22,930  actions::finalize_cast            7,335,377      0.22 %
  21,120  GameState::computed_permanent     7,203,581
```

**`layers::compute_permanent_pass`'s 51,706 growths are gone** — a concurrent
session took them at `31eb7333` (`Printed<Vec<_>>`'s materialize sizing at
`len + 1`), which is most of the 155,758-allocation fall.
**`Vec::push_mut` is an out-of-line `Vec::push`, so its growths belong to
*its* callers**: `static_effect_to_effects` 64,740 (i.e. the gather's
`all_effects`), `activate_ability_inner` 44,776, `declare_attackers_banded`
27,098, `run_effect` 24,970.

**⚠ A `grow_one` row named for a function is not necessarily the buffer you
think.** `gather_continuous_effects_inner`'s row on **`fixed`** (9,836
growths) is *not* `sa_cards` — the four bench archetypes carry no permanent
with a `static_abilities` entry, so `sa_cards` never allocates there at all,
and those growths are `all_effects` outgrowing its `base.len() +
sa_cards.len()` reserve. (-71) proved it by shipping and reading a
byte-identical allocation table on that pool. Check the pool before pricing a
row off one dump.

**The whole family is 11.3 % of `cube` and 10.9 % of `fixed`** (`_int_free`
3.45 / 3.64, `_int_malloc` 2.63 / 2.25, `malloc` 2.58 / 2.72, `free` 2.11 /
2.26, `_int_free_merge_chunk` 0.51), plus `__memcpy` at 2.32 / 1.91. Read the
mimalloc entry before sizing anything off it: callgrind runs the system
allocator and the shipped build does not.

### `GameState::clone` at the same tip — 22,684 calls on `cube`, 10,822 on `fixed`

```text
cube  22,684 clones, ~50.7 M Ir inclusive over its callers, 1.50 % of the run
      (accept_on 11,936 / perform_action 4,964 / sim_start_state 2,338 /
       evaluate_action_sequence 1,408 / main_phase_action_with 1,354)
fixed 10,822 clones, ~23.6 M Ir, 2.12 % of the run
callees, cube:  Vec::clone 136,148 (6.0 a clone) / __memcpy 100,912 /
      RawTable::clone 68,052 / __rust_alloc 56,414 (2.49 a clone) /
      Box::clone 21,440
```

**2.49 allocations per clone, and one of them has a name**: `players:
self.players.clone()` is a `Vec<Player>` of two `Arc` handles, i.e. a malloc
and a free for 16 bytes. The eighty-third pass's line profile priced it at
**472 Ir a clone, 35 % of the clone's whole inline group** — 0.32 % of `cube`
and 0.46 % of `fixed` at this tip's clone counts, before the matching `free`.
See candidate (-72).

### THE WHOLE PROGRAM BY SOURCE LINE, at the eighty-third tip (`34d118fe`), `--decks cube`

**Run once so nobody has to run it again: the simulator has no hot line, and
this is the table that says so.** `profiling-lines` + `--dump-instr=yes` +
`cg_lines.py` with no `--in`, so every function's inlined body is attributed.
96.1 % of the run resolves. **The largest single source line in the program
is 0.82 %**, and only six named engine lines clear 0.25 %:

```text
  24,285,248  0.82%  ptr/mod.rs:1917       Arc::clone_from_ref_in   (the CoW deep copy)
  19,461,794  0.66%  game/mod.rs:?         dispatch_triggers_for_events
  17,159,388  0.58%  iter/macros.rs:?      gather_continuous_effects_inner
  16,294,384  0.55%  game/mod.rs:2283      GameState::clone — `players: self.players.clone()`
  15,352,974  0.52%  game/stack.rs:?       sba_board_scan
  14,559,528  0.49%  ptr/non_null.rs:444   gather_continuous_effects_inner
  13,769,886  0.47%  effects/eval.rs:3117  evaluate_requirement_static_hinted (prologue)
  12,593,316  0.43%  vec/mod.rs:464        dispatch_triggers_for_events
  11,441,834  0.39%  iter/macros.rs:332    check_state_based_actions
   9,344,330  0.32%  iter/macros.rs:180    card_type_change_unscoped
   7,653,600  0.26%  game/mod.rs:14805     perform_action_inner (the action match)
```

**The one line worth an entry is `game/mod.rs:2283`.** `GameState::clone`'s
whole inline group is 46,128,094 Ir (1.6 % of `cube`) over **34,522 clones**,
and **35 % of it is one field**: `players: self.players.clone()`, **472 Ir a
clone**, which is a `malloc` and two `Arc` bumps for a two-element `Vec`.
Every other zone in `GameState` is `CowBox`-wrapped and clones for a
refcount; `players` is a bare `Vec<Player>` and clones for an allocation.
Wrapping it moves that allocation from *every* clone to the first `&mut`
reach — a real win only on the clones that never write a player, and
**allocation-shaped, so Ir overstates it** (mimalloc ships; PERF's pass-54
rule). Size it against `selfplay_train` throughput before building it.

The gather's own line profile is in the same run and reads like the
dispatcher's: **no hot line**. Its inline group is 174,780,740 Ir (5.9 %),
its largest named source line is `any_attached |= card.attached_to.is_some()`
at 3,548,308 (0.12 %), the whole per-card prologue's named lines are ~10.5 M
(0.30 %), and `sa_open`'s `bits & bit != 0` is 4,411,586 (0.15 %) — that is
what the thirty-eight-pass gate device costs to *ask*. **Do not re-run this
profile to look for a hot line in either function.** What it does not show is
(-62): the ungated graveyard walk has no row of its own, because it is
inlined into a 3,581-line function and its cost lands in `iter/macros.rs`.
**A line profile finds hot lines; it does not find cheap lines repeated over
a collection nobody gated.** That one was found by reading the function.

### THE ACTOR RE-READ at the eighty-seventh pass — **-8.02 % since the eighty-third tip, with play byte-identical**

Same workload as every block below it, which is what makes the totals a
comparison: `CRAB_NO_JITTER=1 selfplay_train --actors 1 --games 60 --steps 1
--seed 7`, `profiling-fast --no-default-features` (**`-p crabomination_ml
--no-default-features`** — that crate has its own `mimalloc` default and its
own `#[global_allocator]`; the dump was checked for `libmimalloc` frames and
has none). Play is **byte-identical to the eighty-first and eighty-third
readings**: 32,402 `next_action`, 1,102 `pick_attacks_scored`, 6,386
`encode_state`, 6,895 `main_phase_action_with`.

```text
  a828b393 (eighty-first)   4,235,372,210
  651a98f2 (eighty-third)   4,187,375,624     -1.13 %
  cfc55ae4 (eighty-seventh) 3,851,460,377     -8.02 % from the eighty-third
```

**Four passes of two concurrent sessions, so it is a base and not an
attribution.** What is in it: passes 84-86's inline-storage and CoW work, and
this pass's memo device, prevention fusion and static-source visitor.

**One `actor_loop` iteration, top-down — and deck construction has halved
again.**

```text
  3,650,913,601  94.8 %  play_recorded_game_mcts       60 games
     70,883,884   1.84 % heuristic_sealed_build       120  (was 168,509,048)
     27,889,380   0.72 % encode_deck                  120  (was  28,058,541)
      9,319,428   0.24 % sealed_game_template          60  (was  10,279,007)
      7,428,383   0.19 % sealed_pool                  120  (was  17,852,492)
```

**Deck construction is 2.99 % of the actor, from 5.37 %** — `heuristic_sealed_
build` alone is **-57.9 %**, which is where pass 85's five deck-builder
commits and the concurrent half's `rank_shape` work landed. (-63)'s framing
holds: what is left is `rank_shape`'s own body and the number of shapes.

```text
  self, top twelve
   204,159,083  5.30 %  dispatch_triggers_for_events
   183,869,341  4.77 %  __memcpy_avx_unaligned_erms
   132,829,827  3.45 %  _int_free
   132,204,751  3.43 %  gather_continuous_effects_inner
   115,326,415  2.99 %  Arc::clone_from_ref_in
   109,615,890  2.85 %  _int_malloc
   102,231,746  2.65 %  malloc
    92,478,695  2.40 %  check_state_based_actions_into
    87,270,802  2.27 %  Vec::spec_from_iter_nested
    82,795,088  2.15 %  free
    81,800,586  2.12 %  computed_permanent
    64,712,678  1.68 %  activate_ability_inner
```

**The allocator family is 11.1 % over four symbols and `memcpy` is 4.77 % —
and `memcpy`'s caller table is READ and it is diffuse.** 2,727,324 calls at
**67 Ir apiece**; the largest rows are `play_recorded_game_mcts` (273,902),
`encode_state` (235,586), `encode_card_object` (229,198),
`computed_permanent` (205,261) and `GameState::clone` (166,120), and the
dearest per call in the top sixteen is 91 Ir. (-60)'s device — rank a
`memcpy` table by Ir/call to find the kilobyte copies — finds **nothing** at
this tip: there is no `CardInstance::new`-shaped row left. Do not re-run it.

**(-51)(a) RE-SIZED HERE, AND ITS NAMED BLOCKER IS GONE.**
`auto_tap_for_cost_inner -> activate_ability` is **33,431 calls /
227,870,478 Ir / 5.92 % of the actor**, i.e. **6,816 Ir a tap** (7,555 at the
seventy-fifth tip). Inside `activate_ability_inner`, by callee:

```text
   76,051   41,861,836  Arc::make_mut          1.09 % of the actor, 2.2 a call
   34,446   30,182,959  card_keyword_possible  0.78 %, 876 Ir a question
   67,407    5,833,889  Vec::push_mut
   34,634    5,454,719  FlattenCompat::iter_fold
```

**The entry said the fix for the second row was "a cheaper
`keyword_grant_in_scope`, and the per-definition keyword-grant bit that would
do the latter is in TODO's do-not-rebuild list". That bit shipped this pass**
(Baseline row (3)) and `card_can_grant_keyword` is **31 Ir a card** inside it,
from 43.6. What is left is the *number of cards*: 562,156 visits over 34,446
questions, **16.3 a question**, which is the board walk itself and is (-61)'s
"fewer walks, not a cheaper one" unchanged. `make_mut` is now the larger half
and it is (-74)'s "fewer deep copies", on genuine writes (the tapped card and
the seat's mana pool).

### THE ACTOR at the eighty-third tip (`651a98f2`) — and deck construction is on the table again

Same workload as the two blocks below, so the three are comparable:
`CRAB_NO_JITTER=1 selfplay_train --actors 1 --games 60 --steps 1 --seed 7`,
`profiling-fast --no-default-features`, callgrind. **Play is byte-identical
to both earlier readings** — 32,402 `next_action`, 1,102
`pick_attacks_scored`, 6,386 `encode_state`, 6,895 `main_phase_action_with` —
which is what makes the totals a comparison rather than a coincidence.

```text
  a828b393 (eighty-first)   4,235,372,210
  651a98f2 (eighty-third)   4,187,375,624     -1.13 %
```

**That span is a whole pass, not one commit, so it is a base and not an
attribution** — the file's own rule. What is in it: the `sim_step`
checkpoint, the `zone::Graveyard` memo, the three `SecondPass::of` hoists,
`pick_stack_response`'s filter, and the requirement walker's fallback chains.

**One `actor_loop` iteration, top-down** — the level nobody had recorded, and
the one that says what a *training* actor pays outside the games:

```text
  3,877,448,028  92.6 %  play_recorded_game_mcts       60 games
    168,509,048   4.02 % heuristic_sealed_build       120  (two a game)
     28,058,541   0.67 % encode_deck                  120
     17,852,492   0.43 % sealed_pool                  120
     10,279,007   0.25 % sealed_game_template          60
```

**Deck construction is 5.37 % of the actor** — build + pool + template +
`encode_deck` — at ~1.87 M Ir a build. The fifty-third pass found it at
*ten times the simulation* and the deck-builder fix took it 3.28x; this is
where that landed. It is still **invisible on `--bench`**, which builds its
decks once, and it is 87 % of the encoder's share for a fifth of the passes
spent on the encoder. `recommend::build_shape` is 54,029,042 Ir of *self*
(1.29 %, 6,840 calls) and is the largest single row inside it.

**Inside the games, and the shape has not moved:**

```text
  inclusive, top-down
   3,222,789,991  76.97 %  HeuristicBot::next_action     32,402 calls
     1,914,936,337  45.73 %   pick_attacks_scored          1,102
     1,183,552,872  28.26 %   main_phase_action_with       6,895
     260,128,409   6.21 %  encode_state                    6,386

  self, top ten
   209,403,502  5.00 %  __memcpy_avx_unaligned_erms
   207,716,264  4.96 %  dispatch_triggers_for_events
   159,467,263  3.81 %  _int_free
   141,446,867  3.38 %  _int_malloc
   131,229,904  3.13 %  gather_continuous_effects_inner
   121,131,123  2.89 %  malloc
   110,942,742  2.65 %  Arc::clone_from_ref_in
    96,751,249  2.31 %  free
    91,555,500  2.19 %  Vec::spec_from_iter_nested
    82,562,743  1.97 %  check_state_based_actions
```

**`sim_step` is 74,388 calls and every one of them is `perform_action_inner`
now** — the checkpoint row is gone from the actor as well as the ladder.
`simulate_attack_outcome_once` reaches it 72,819 times for 850,191,378 Ir,
and its other half is `sim_spell_action_inner` at 30,748 / 527,154,621.

**The allocator family is 12.4 % between four symbols and `memcpy` is the
largest self row in the program**, which is the same story the eightieth
pass told and the one thing no pass has aimed at directly. Read PERF's
mimalloc entry before sizing anything from those rows: callgrind runs the
system allocator and the shipped build does not.

### THE ACTOR RE-READ at the eighty-first pass — and the base had moved

The eightieth tip's actor profile is the block below. Re-running the same
workload after the eighty-first pass's first half looked like a **+0.159 %
regression** against its recorded total — and it is not, because the recorded
total is not this pass's base. `be4a9987` (the previous pass's last commit,
the CR 509.1d block tax) landed between them and moved the actor on its own.

```text
CRAB_NO_JITTER=1 selfplay_train --actors 1 --games 60 --steps 1 --seed 7
profiling-fast --no-default-features, callgrind. Play byte-identical across
all three: 32,402 `next_action`, 1,102 `pick_attacks_scored`, 6,386
`encode_state`, 6,895 `main_phase_action_with`.

  a4b24308 (recorded, eightieth tip)   4,228,661,490
  be4a9987 (this pass's real base)     4,236,954,968     +0.196 %
  a828b393 (eighty-first, first half)  4,235,372,210     -0.037 % vs base
```

**So the pass is flat on the actor and the queue's rule paid for itself
again**: "re-measure the base if any commit landed since the recorded row"
(eightieth pass) is the difference between reporting a 0.16 % regression and
reporting the truth. **A recorded total is a measurement of a commit, not of
a branch.**

Row-level, `be4a9987` -> `a828b393`, and it splits cleanly by author:

```text
  -7,099,670  declare_attackers_banded      \  the CR 508.1a walker unification:
  +2,466,577  attacker_self_block           /   net -4.6 M
  -5,795,203  bot_can_block                     `legal_blockers`, net -5.8 M
  -7,352,274  computed_permanent                both of those, plus `Printed`

  +12,540,432  blocker_self_block           \  the CR 509.1a/b blocker walker:
  +10,111,869  blocker_pair_block            |  net +6.9 M, and its ladder rows
   -7,620,697  declare_blockers              |  are wins — see "which pool"
   -5,431,781  blocker_can_block_attacker_pair|
   -2,678,499  blocker_side_gates_allow_block/
   +6,233,130  board_keyword_in_scope

  -10,730,478  can_afford_in_state_with     \  (-53)'s close: the two `*_over`
   +7,801,660  cost_reduction_for_spell_full_over|  helpers stopped inlining,
   +3,442,781  extra_cost_for_spell_over    /   net ~+0.5 M, i.e. attribution
```

**The two walker unifications land on opposite sides on this workload**, and
neither is a mistake: the attack one is a net win on the actor *and* on both
ladder pools, the block one is a win on the ladder and a small cost here. The
actor's pools are sealed decks and its blocker boards are wider than
`fixed`'s, which is the fifty-third pass's ranking rule (**ask which pool the
change lives on**) reappearing on the ML workload rather than the bench.

### THE ACTOR, at the eightieth tip — the ML workload, profiled at last

NEXT has said for four passes that this file describes `bot_ladder` only.
Here is the other one.

```text
CRAB_NO_JITTER=1 selfplay_train --actors 1 --games 60 --steps 1 --seed 7
profiling-fast --no-default-features, callgrind.  4,228,661,490 Ir.

  inclusive, top-down
   3,275,831,114  77.5 %  HeuristicBot::next_action     32,402 calls
     1,956,331,551  46.3 %    pick_attacks_scored        1,102   1.78 M each
     1,181,904,885  27.9 %    main_phase_action_with     6,895
     291,221,400   6.9 %  GameState::perform_action     27,701   (the real game)
     259,906,228   6.1 %  encode_state                   6,386
      59,354,329   1.4 %  net init (VarBuilder::get)        20   ONE-TIME

  self, top ten
   212,915,362  5.04 %  __memcpy_avx_unaligned_erms
   207,716,264  4.91 %  dispatch_triggers_for_events
   162,417,603  3.84 %  _int_free
   143,477,492  3.39 %  _int_malloc
   135,897,243  3.21 %  gather_continuous_effects_inner
   123,360,589  2.92 %  malloc
   115,308,969  2.73 %  Arc::clone_from_ref_in
    98,590,762  2.33 %  free
    92,984,331  2.20 %  Vec::spec_from_iter_nested
    83,263,034  1.97 %  computed_permanent
```

**The headline is that the actor is not a different animal.** Its top rows
are the *engine* — allocation and copying 17.7 % between five symbols, the
trigger dispatcher 4.91 %, the layer gather 3.21 % — and its shape is the
ladder's shape: an attack search that is half the program, a main phase that
is a quarter, and the game itself under 7 %. The encoder, four passes of work,
is **6.1 %**. So a lead found on `bot_ladder` mostly transfers, and the
"three of the actor's top rows are 0 calls on the bench" line in NEXT was
about the *encoder's* rows, not about the profile as a whole.

**One attack decision costs 1.78 M Ir.** The ladder's figure is 826 k for one
*sim* and 59.6 % for the whole search on `cube`; this is the same device on
`sos` decks, and it is the largest single number in the ML pipeline.

**⚠ READ THE RUN LENGTH BEFORE READING ANY SHARE HERE.** A `selfplay_train`
process pays a fixed startup — a randomly-initialized net, 722,816 normal
samples — and at a short run that fixed cost looks like a hot path:

```text
                                       20 games        60 games
  rand_distr Normal::sample      35,325,414  2.58 %   35,317,689  0.84 %
  net init, inclusive            59,354,329  4.34 %   59,354,329  1.40 %
```

**The same absolute Ir, three times the share.** `Normal::sample` sits at #8
in a 20-game self-cost table and vanishes from a 60-game one, and nothing
about the program changed. It is not waste — `--steps 1` genuinely starts a
fresh net — it is fixed cost, and **a share is a ratio whose denominator you
chose**. Profile the actor at 60 games, or subtract two run lengths; the
20-game runs the encoder passes measured against carried ~4.5 % of this, so
those deltas are all slightly *understated* against real game work.

**Correction to `6c9746ec`'s message**, which said the Monte Carlo bot is
"the path the training actors take": `selfplay_train`'s `--mcts-actors`
defaults to **0**, so actors run `HeuristicBot` unless asked otherwise —
which is exactly what this profile shows. The MctsBot test that commit added
is still worth having (it covers a supported actor mode and the only consumer
that can play an unfiltered menu entry), but it is not on the default path.

### Inside one attack sim, at the seventy-fifth tip (`5e4ec3bd`), `--decks cube`

NEXT's item N3 read `pick_attacks_scored` inclusively for the first time. This
is the level below: what the 1,910 sims of a `cube` run actually spend, taken
with `cg_edges.py --callers/--callees` on the base dump of the seventy-sixth
pass. **The whole search is 59.63 % of the program and one sim is 826,000 Ir**
— 0.031 % of the run apiece.

| edge | calls | inclusive Ir | % cube |
|---|---|---|---|
| `next_action_inner -> pick_attacks_scored` | 928 | 1,589,376,409 | **59.63** |
| `pick_attacks_scored -> simulate_attack_outcome_once` | 1,910 | 1,577,542,477 | 59.19 |
| `next_action_inner -> pick_blocks_scored` | 338 | 75,481,536 | 2.83 |

Inside `simulate_attack_outcome_once` (per-sim rate in brackets):

| callee | calls | inclusive Ir | % cube |
|---|---|---|---|
| `sim_step` (29.1/sim) | 55,510 | 799,456,685 | **30.0** |
| `sim_spell_action_inner` (10.7/sim) | 20,388 | 413,788,025 | **15.5** |
| `perform_action_inner` (the declaration + decisions) | 3,358 | 191,473,271 | 7.2 |
| `pick_blocks` | 2,550 | 69,606,358 | 2.6 |
| `drop_in_place<GameState>` | 4,670 | 30,572,125 | 1.15 |
| `pick_attacks_inner` (the greedy re-declaration) | 1,766 | 28,594,797 | 1.07 |
| `eval_material_frozen` | 1,910 | 18,447,619 | 0.69 |
| `sim_start_state` (the clone) | 1,910 | 5,199,711 | 0.20 |

And inside `sim_spell_action_inner`, which is the part that is *not* the
engine advancing a turn:

| callee | calls | inclusive Ir | % cube |
|---|---|---|---|
| `accept_on` — the cast the sim adopts | 3,910 | 284,124,532 | **10.66** |
| `cast_candidates` | 6,210 | 96,013,585 | **3.60** |
| `pick_combat_trick` | 5,830 | 8,354,437 | 0.31 |
| `pick_stack_response` | 9,112 | 4,478,080 | 0.17 |

**Three readings worth keeping.** (a) Of the 20,388 entries, **9,112 are the
stack branch and 5,830 the blocker branch, and both are cheap**; the
main-phase branch is 6,210 and carries everything. (b) **63 % of main-phase
entries end in a probe** (3,910 of 6,210), and that probe *is* the cast — the
sim adopts its state — so `accept_on` here is not waste and the fiftieth
pass's bargain is working. (c) A sim probe costs **72,666 Ir against
`main_phase_action_with`'s 45,146**, which is the one number in this block
nobody has explained.

**(c) IS STILL OPEN AND `--separate-callers=3` IS NOT THE INSTRUMENT — recorded
so nobody spends the run again (ninety-second pass).** The gap survives at the
ninety-second tip on `fixed`: **61,878 Ir a probe from `sim_spell_action_inner`
against 41,870 from `main_phase_action_with`**, the same ~1.5x. A depth-3 dump
puts `pay_census::in_probe` and its caller in the context chain, so the two
sites *are* separable — but only for frames within three of the leaf, and
`accept_on`'s body is deeper than that. What it does bound is the **checkpoint**:
the `memcpy` that names a probe site reads **413 Ir a probe at
`main_phase_action_with`, 437 at `sim_spell_action_inner` and 454 at
`pick_land_to_play`** — within 6 % across three sites and two orders below the
gap. **So the extra 20,000 Ir is not the clone and not CoW unsharing at the
clone; it is the action the probe then performs.** That leaves board state (a
sim probes a later, larger board) and action mix, and neither is a profile
question — the next attempt wants a *counter* (permanents on the board and
actions per probe, split by `in_probe`'s origin, which `pay_census` already
tracks) rather than a deeper dump.

**So the attack search's cost is the engine playing a turn, not the bot
choosing.** `sim_step` + the direct `perform_action_inner` is 37.2 % of the
program against `cast_candidates`' 3.6 %. The levers are fewer sims
(ladder-gated: 928 decisions produce 1,910 sims, so most declarations already
take the one-candidate early return) or a cheaper `perform_action_inner`.

**And one level further down, the step machinery, taken at the same tip:**

| edge | calls | inclusive Ir | % cube |
|---|---|---|---|
| `sim_step -> perform_action_inner` (the pass) | 52,406 | 646,926,078 | **24.3** |
| `sim_step -> perform_action` (the checkpointed branch) | 4,322 | 177,669,748 | **6.66** |
| `pass_priority -> advance_step` | 35,800 | 566,139,820 | **21.2** |
| `advance_step -> resolve_combat` | 4,480 | 341,332,087 | **12.8** |
| `advance_step -> fire_step_triggers` | 22,466 | 76,101,195 | 2.86 |

**`sim_step`'s checkpointed branch is a candidate nobody has costed**:
4,322 actions at **41,100 Ir apiece**, and ~5,750 of each is
`perform_action`'s checkpoint — clone 1,194, drop 2,324, plus the CoW
unshares the action pays only because the checkpoint re-shared the zones the
sim had already unshared. That is **~0.93 % of `cube` spent taking a snapshot
of a state the caller owns and throws away.** It is *not* free to remove:
`sim_step`'s documented fallback rolls a rejected declaration back and retries
it as a priority pass, and `declare_blockers` / `declare_attackers_banded`
hold 82 of the engine's `Err` sites between them — which is exactly the half
(-13) calls "where the checkpoint earns its keep". What has never been
measured is **how often those 4,322 actually fail**; if it is zero on every
pool, the shape is (-13)'s fallibility closure applied to three action kinds
rather than to `PassPriority`.

### The three pools at the seventieth tip (`ee376912`) — and the *inclusive* half

`fixed` **1,148,918,411**, `sos` **1,482,238,008**, `cube` **2,631,861,321**,
one binary, one config, one pool each. Against the Baseline's columns for the
same tip the totals agree to **362 Ir on cube and 493 on fixed (0.00001 %)** —
a third box, a third reading, and the fourth independent confirmation that
callgrind Ir is portable across these containers.

| row (self) | sos | fixed | cube |
|---|---|---|---|
| `dispatch_triggers_for_events` | **6.14 %** | **5.86 %** | **5.51 %** |
| allocator (`_int_free`+`malloc`+`_int_malloc`+`free`) | 13.04 % | 11.00 % | 11.46 % |
| `gather_continuous_effects_inner` | 3.88 % | 4.72 % | 4.31 % |
| `__memcpy_avx_unaligned_erms` | 5.14 % | 2.61 % | 3.19 % |
| `Arc::clone_from_ref_in` | 3.13 % | 3.35 % | 3.08 % |
| `Vec::from_iter` (SpecFromIterNested, all monos) | 1.92 % | 2.71 % | 2.64 % |
| `check_state_based_actions` | 2.53 % | 2.09 % | 2.01 % |
| `computed_permanent` | 1.44 % | 1.56 % | 1.75 % |
| `compute_permanent_pass` | 1.20 % | 1.38 % | 1.68 % |
| `GameState::clone` | 1.68 % | 1.89 % | 1.61 % |
| `sba_board_scan` | 1.89 % | 1.82 % | 1.53 % |
| `card_can_grant_keyword` | 1.07 % | 1.48 % | 1.46 % |
| `activate_ability_inner` | 1.49 % | 1.26 % | 1.44 % |
| `dispatch_board_scan` | 1.47 % | 1.79 % | 1.32 % |
| `fire_combat_damage_triggers` | 0.91 % | 1.23 % | 1.30 % |
| `perform_action_inner` | 1.42 % | 1.76 % | 1.29 % |
| `card_type_change_unscoped` | 1.11 % | 1.03 % | 1.10 % |
| `trigger_grant_sources` | 0.78 % | 1.20 % | 0.99 % |
| `resolve_combat` | 0.55 % | 0.92 % | 0.98 % |
| `Arc::make_mut` | 0.83 % | 0.99 % | 0.89 % |

**The self table has stopped saying anything new and this is the fourth pass
to notice it.** Every row above 1 % is either refuted by name in this file
(`dispatch_triggers_for_events` — (-16), the trigger-carrier bitmask, the
line-profile-is-not-a-saving rule), structural (`clone_from_ref_in` + the
allocator + `memcpy` = **19.3 % of cube** and that *is* the checkpoint), or
already carrying its own fused scan (`sba_board_scan`, `dispatch_board_scan`).
**`dispatch_triggers_for_events` line-profiled again on `profiling-lines` at
this tip and came back diffuse exactly as (-16) says**: the largest engine
line in it is 0.23 % (`is_event_hardcoded`'s `match ev`, refuted at pass 61),
the graveyard / exile / command walks at the tail are **1.5 M Ir between them
(0.06 %)**, and the rest is `iter/macros.rs` and `ptr/*` — the loops
themselves. *Do not re-line-profile this function.*

**So read the inclusive column instead, and it says something the self column
cannot.** Every figure below is `cg_edges.py --callers/--callees` on the
`cube` dump at this tip.

| call site | calls | inclusive Ir | % cube |
|---|---|---|---|
| `accept_on` -> `perform_action_inner` (the bot's dry-run probe) | 11,986 | 535,808,060 | **20.36** |
| `auto_tap_for_cost_inner` -> `activate_ability` (tapping for mana) | 21,566 | 162,930,901 | **6.19** |
| `cast_spell_with_convoke` -> `try_pay_after_snapshot_mode` | 7,186 | 233,173,980 | 8.86 |
| `try_pay_after_snapshot_mode` -> `auto_tap_for_cost_inner` | 11,454 | 279,514,144 | 10.62 |
| `cast_candidates` -> the candidate `collect` | 11,314 | 118,051,106 | 4.49 |
| `can_afford_in_state_with` (the bot's affordability pre-filter) | 29,442 | 72,767,888 | 2.77 |
| `auto_tap_for_cost_inner` -> `mana_source_table` | 8,986 | 60,112,994 | 2.28 |
| `OnceCell::try_init` -> `bot::available_mana` | 10,268 | 30,908,232 | 1.17 |
| `do_untap` (whole) | 2,566 | 40,730,961 | 1.55 |
| `card_keyword_possible` <- `activate_ability_inner` | 25,268 | 28,951,558 | 1.10 |

**A land tap costs 7,555 Ir.** That is `activate_ability` run in full for a
`{T}: add mana` ability — the CR 602.5 gates (`card_keyword_possible` alone is
1,146 Ir of it, 726 of *that* being `keyword_grant_in_scope`'s board walk),
the cost machinery, the delayed-trigger plumbing and the event push. It is the
second-largest single call site in the simulator and nothing in this file has
ever costed it.

**And a third of the payments the simulator makes are thrown away.**
`restore_payment_state` runs **3,712 times against 11,634
`try_pay_after_snapshot_mode` calls — 31.9 %**, and a failed payment has
already built its `mana_source_table` (6,690 Ir) and tapped whatever it could
before `pay_for_spell` rejects it. NEXT's cast-failure figure (39 % of `cube`
cast attempts) is the same fact one frame up: 7,790 non-recursive
`cast_spell_with_convoke` calls reach `finalize_cast` 4,720 times. See (-51).

### `clone_from_ref_in` by calling context — NEXT's item 3, run

`--separate-callers=2` on the same tip and pool, `cg_contexts.py`. **157,402
actual deep copies**, against 806,878 `make_mut` *calls*: the caller table
(-43) points at describes who *asks*, this one describes who *pays*, and the
two rank differently. `activate_ability_inner` is 21 % of every deep copy in
the program and does not head the `make_mut` table; `do_untap` heads a
70,838-call `make_mut` row and clones 5,632 times (8 %).

```text
  32,782  make_mut <- activate_ability_inner          (65 % of its make_mut calls clone)
  30,670  make_mut <- cast_spell_with_convoke         (26 %)
  19,052  make_mut <- declare_attackers_banded        (30 %)
   7,382  make_mut <- declare_blockers
   5,632  make_mut <- do_untap                        (8 %)
   5,304  make_mut <- PlayerData::send_to_graveyard
   5,294  make_mut <- resolve_top_of_stack_inner
   5,014  make_mut <- finalize_cast
   4,240  make_mut <- on_left_battlefield             (NEXT 1c's lead)
   4,062  make_mut <- PlayerData::draw_top
   2,790  make_mut <- try_pay_after_snapshot_mode
   2,404  PlayerData::deref_mut <- resolve_top_of_stack_inner
   2,402  GameState::deref_mut <- declare_blockers
   2,124  make_mut <- PlayerData::remove_from_hand
   2,116  make_mut <- remove_from_battlefield_to_graveyard_raw
   2,022  make_mut <- finish_cleanup
   1,830  make_mut <- dispatch_triggers_for_events
   1,544  make_mut <- objects_leave_with_player
     902  make_mut <- auto_tap_for_cost_inner
# 10,498 calls in 63 further contexts
```

**The clone/ask ratio is the column to read, not the clone count.** A site at
65 % is one where the first `&mut` after every checkpoint is genuinely the
first write — there is no no-op write to gate, which is why (-50)'s device
does not apply to `activate_ability_inner` (tapping the source and paying from
the pool are real writes). A site at 8 % is mostly re-writes of an already
unshared handle, i.e. already paid for. **(-50) lives where the ratio is high
*and* the write is a no-op**, and the two rows that still look like that are
`declare_attackers_banded` (30 %) and `cast_spell_with_convoke` (26 %, and
NEXT item 2b already names its mechanism: the card is taken out of hand ahead
of fifty gates).

### The three pools at `21a48317` — the re-based base

Same binary, same config, one pool each: **`fixed` 1,172,084,149, `sos`
1,510,906,673, `cube` 2,710,032,778.** Against the sixty-third tip's columns
below: `fixed` **-0.310 %**, `sos` **-0.850 %**, `cube` **-0.828 %**. That
span carries the sixty-fourth pass (token mint, layer-pass collect), the
sixty-fifth's (-45) row, and three rules commits, so **it is not an
attribution of anything** — it is a base. The rules commits are the reason
the drop is a little smaller than the perf passes claimed end to end: nine
cards that used to no-op now resolve, and the auto-targeter makes one extra
walker call where `primary_target_filter` is silent. Paying Ir to make a card
do its printed thing is the trade working.

**Read the tip before using these columns.** They were measured at
`21a48317`; the concurrent session pushed `780ef86c` (four more of (-45)'s
rows) while the three runs were in flight, so a total taken after that commit
is already a few tenths below these. That is the branch working as intended
and it is why every block in this file carries its tip — the columns are
comparable to each other, which is what a profile of record is for, not to
whatever HEAD happens to be.

| row | sos | fixed | cube |
|---|---|---|---|
| `dispatch_triggers_for_events` | **6.03 %** | **5.75 %** | **5.35 %** |
| allocator (`_int_free`+`malloc`+`free`+`_int_malloc`) | 12.99 % | 10.83 % | 11.47 % |
| `gather_continuous_effects_inner` | 3.81 % | 4.63 % | 4.18 % |
| `__memcpy_avx_unaligned_erms` | 5.08 % | 2.57 % | 3.11 % |
| `Arc::clone_from_ref_in` | 3.22 % | 3.33 % | 3.48 % |
| `Vec::from_iter` (all monos) | 2.01 % | 2.78 % | 2.63 % |
| `check_state_based_actions` | 2.48 % | 2.05 % | 1.95 % |
| `sba_board_scan` | 1.86 % | 1.79 % | 1.48 % |
| `GameState::clone` | 1.65 % | 1.85 % | 1.56 % |
| `activate_ability_inner` | 1.46 % | 1.24 % | 1.40 % |
| `dispatch_board_scan` | 1.44 % | 1.75 % | 1.29 % |
| `perform_action_inner` | 1.39 % | 1.72 % | 1.26 % |
| `computed_permanent` | 1.32 % | 1.38 % | 1.53 % |
| `evaluate_requirement_static_hinted` | 1.30 % | 2.05 % | 1.23 % |
| `compute_permanent_pass` | 1.17 % | 1.35 % | 1.63 % |
| `card_can_grant_keyword` | 1.05 % | 1.45 % | 1.42 % |
| `fire_combat_damage_triggers` | 0.89 % | 1.21 % | 1.26 % |

**`dispatch_triggers_for_events` is still the largest engine self row on all
three pools, and it has been since pass 43.** Its known devices are refuted:
the trigger-carrier bitmask is in TODO's do-not-rebuild list and (-16) reads
it as diffuse by line.

**The pool-ratio device is mined out at this tip and the next run should not
spend a round on it.** `scripts/cg_ratio.py cg.cube.out cg.sos.out --floor
0.45` (its first real use) returns **nothing above 1.83x**, where the
sixty-second tip had a 5.08x that became a commit and a 2.09x that pointed
pass 63 at `pick_blocks_inner`:

```text
num%   den%      x    row                              (cube over sos, 21a48317)
1.05   0.57   1.83   evaluate_requirement_static_hinted
0.87   0.50   1.73   resolve_combat
0.75   0.44   1.69   CardInstance::has_keyword          <- read at 1.84x, "flat, no"
0.84   0.51   1.64   declare_blockers
0.89   0.57   1.56   apply_prevention_shields           <- (-2b) already paid on it
0.53   0.34   1.54   compute_permanent
```

`pick_blocks_inner` is absent from the listing entirely — pass 63 took it
from 0.90 % of cube to 0.31 %, which is what a mined-out device looks like
from the inside. **A flat ratio table is a result, not a failed run**: it
says the remaining cost is diffuse across both pools rather than
pool-specific, so the next pointer has to come from a different device
(Ir/call outliers, or `--callers SpecFromIterNested` by call count).

**And one standing caution in this file no longer reproduces — do not act on
it until it is re-derived.** The "`name_index()` builds 22,568
`CardDefinition`s, 104,687,400 Ir, **6.8 % of a six-game `sos` total**"
figure, and its instruction to *subtract it before quoting an `sos` share*,
is not visible here. Summing every `crabomination_catalog::` self row gives
**sos 18,074,614 (1.20 %), cube 207,187 (0.01 %), fixed 23,424 (0.00 %)**,
and `--callers OnceLock` finds six calls totalling under 4 k Ir with no
`initialize` edge of anything like the recorded magnitude. Self rows
understate an inclusive cost, so this is not proof the build is gone — but
it is proof the 6.8 % figure is not this tip's. **Subtracting 104.7 M from a
1.51 G `sos` total on the file's say-so would now distort every share by
~7 %**, which is the exact failure CLAUDE.md's "a bare present-tense count
goes stale and then misleads" rule exists to prevent. Re-derive before
subtracting; (-46) wants re-sizing on this basis and stays ranked last
either way.

### The three pools at the sixty-third tip (`fa3bf671`)

Same binary, same config, one pool each: **`fixed` 1,175,724,194, `sos`
1,523,856,909, `cube` 2,732,667,632.** Self costs, top 18 on each pool, with
the allocator family summed rather than listed four times.

| row | sos | fixed | cube |
|---|---|---|---|
| `dispatch_triggers_for_events` | **5.98 %** | **5.73 %** | **5.30 %** |
| allocator (`_int_free`+`malloc`+`free`+`_int_malloc`) | 12.9 % | 11.3 % | 12.8 % |
| `__memcpy_avx_unaligned_erms` | 5.43 % | 2.56 % | 3.64 % |
| `gather_continuous_effects_inner` | 3.77 % | 4.61 % | 4.15 % |
| `Arc::clone_from_ref_in` | 3.19 % | 3.32 % | 3.45 % |
| `check_state_based_actions` | 2.45 % | 2.04 % | 1.93 % |
| `Vec::from_iter` (all monos) | 2.06 % | 2.83 % | 2.64 % |
| `sba_board_scan` | 1.84 % | 1.78 % | 1.47 % |
| `GameState::clone` | 1.64 % | 1.84 % | 1.55 % |
| `dispatch_board_scan` | 1.43 % | 1.75 % | 1.27 % |
| `compute_permanent_pass` | 1.31 % | 1.47 % | 1.66 % |
| `computed_permanent` | 1.30 % | 1.36 % | 1.51 % |
| `evaluate_requirement_static_hinted` | 1.28 % | 2.04 % | — |
| `card_can_grant_keyword` | — | 1.45 % | 1.40 % |
| `activate_ability_inner` | 1.45 % | — | 1.39 % |
| `fire_combat_damage_triggers` | — | — | 1.25 % |

**`dispatch_triggers_for_events` is the largest engine self row on all three
pools and has been since pass 43**, and it is still the standing "biggest row
with no taker": 91,088,068 Ir on `sos` over 81,744 calls, of which 44,560 get
past the empty-batch return — ~2,044 Ir of self per working dispatch,
essentially all of it the `for card in &self.battlefield` walk. (-16) reads
it as diffuse by line (largest single line 1.06 % on cube) and it has never
been read by line on `sos`.

**Nothing in the sixty-third pass's subtree is in this table**, which is the
point of the pool-ratio device: `pick_blocks_inner` was 0.90 % of cube before
the pass and 0.31 % after, and no top-18 listing would ever have shown it.

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

\* **corrected at the ninety-second pass: `'2` is not a monomorphization.** It
is callgrind's recursion level (`--separate-recs`, default 2), so this cell is
one frame of a function whose self cost is split across two rows plus a
`::{{closure}}`, and "the two split differently per pool" is recursion depth
following board size. The folded figure is the one to compare — see the
ninety-second pass's Baseline entry, where the same function reads 1.10 % as a
row and **3.36 %** folded on `cube`.

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

**State at the `(-250)` tip — THE IR BASE MOVED (`panic = "abort"` on
every optimized profile, three-pool Ir against the `(-249)` tip):
`cube` 1,876,460,069 -> 1,817,493,748 (-3.142 %), `fixed` 681,439,653
-> 656,319,384 (-3.686 %), `sealed` 1,959,940,755 -> 1,886,392,273
(-3.753 %); paired wall clock +4.45 % / +5.51 %. Every A/B from here is
against these; nothing in the tables below was re-read on the new base
(the shares shift by ~3 %, the ranking does not).**

**Build levers now pulled, so nobody re-pulls them:** mimalloc (+22 %),
PGO (-24 %, opt-in), `panic = "abort"` (`(-250)`, +4.5-5.5 %); refuted:
`target-cpu=native` (flat), **`opt-level = 2` (`(-251)`: -8.1 % wall
clock, Ir +5.0 %, I1 misses +9.0 %, binary +8.6 % — level 3's win here
is the calls it inlines, not width)**, **`-inline-threshold=500`
(`(-252)`: flat wall clock, Ir -1.7 %, I1 misses -0.1 %, `.text` -28 %
— it inlines the cold catalog constructors, not the hot working set)**.
The inlining axis is closed from both sides; what is left on the
build is PGO (a profile, not a threshold) and the unshare direction in
source. Not tried, with the reason: fat LTO on
`release` (the single-codegen-unit engine already peaks at ~5.9 GB in
this container — `profiling` cannot build here; fat LTO is worse);
`opt-level = 3` on the dev deps is already on. **`-C force-frame-
pointers=no` is the default; `debug = false` on `release` already.**
BOLT is blocked (no `llvm-bolt`, no `perf`).

**State at the `(-249)` tip (one leg on top of `(-248)`, three-pool Ir
against the `(-248)` tip `b7285f4e`): `cube` 1,883,973,537 ->
1,876,460,069 (-0.399 %), `fixed` 681,812,730 -> 681,439,653
(-0.055 %), `sealed` 1,967,054,280 -> 1,959,940,755 (-0.362 %). The
Log row has `do_untap`'s before/after.**

**RE-READ AT the `(-248)` tip — the plain `cube` self table, top forty
rows, against the `(-241)` table below, so nobody re-takes it.** Every
row the `(-241)` read priced is where it was, minus the legs since; the
rows that had **never been on a table** were read this run:

```text
  self Ir   share   row                          what
   14.8 M   0.78 %  LocalKey<T>::with            the freeze memo's lock: 146,008 asks from computed_permanent_hinted (18.2 M incl., ~124 Ir — the `perms` linear scan, "a memo index is marginal" at (-219)) + 66,904 Unfreeze::drop (6.0 M, ~90 Ir); floor
   12.2 M   0.65 %  grant_scan                   33,372 builds, ~500 Ir: 12,518 from available_mana, 8,678 mana_source_table, 4,252 main_phase_action_with — the lane is PRESENT on most cube boards; the walk is the cost. Read; a per-scope memo is the (-82) shape (0.98 builds a call); floor
   11.7 M   0.62 %  do_untap                     TAKEN (-249): 2,834 calls, 8,366 -> 5,773 Ir each; the residue is the untap loop + do_phasing + the flag-reset walk
   10.9 M   0.58 %  advance_step                 the step machine; not read
   10.7 M   0.57 %  fire_step_triggers           24,216 calls, ~440 Ir self: trigger_grant_sources + retain, the triggerer walk, the equipped_bonus walk (per-card `attached_to` test), the graveyard lane; three IntoIter drops a call (the candidate/grant Vecs). Read, nothing above 1 M; floor
   10.0 M   0.53 %  grants_nothing_slow          204,606 calls, ~49 Ir: 104 k recursive, 36.7 k available_mana, 35.8 k mana_source_table, 13.6 k effective_mana_abilities_into; the per-card grant test the (-199) device left; floor
    9.6 M   0.51 %  SmallVec::extend             133,272 calls: blockers_of 24.6 k, GameState drop 23.2 k, gather 18.3 k, declare_blockers 15.4 k, SBA 9.0 k / 4.6 M, resolve_combat 7.6 k / 6.0 M — the "fill a SmallVec with a loop" rule's remaining collects; each under 0.3 %
    9.4 M   0.50 %  blocker_self_block           65,204 calls: 51,056 under bot::legal_blockers (5.8 M), 8,018 declare_blockers, 6,130 blocker_can_block_attacker; the planner's pair count, as blocker_pair_block's is; floor
    9.1 M   0.48 %  IntoIter::drop               367,830 calls, ~25 Ir: fire_step_triggers 72.6 k, declare_attackers_banded 57.9 k, SBA 51.5 k, the three combat-damage trigger walks 62.7 k — every `for x in vec` over a short candidate Vec; the shape (-244) priced, spread over forty sites
```

**The cache/branch axis (Profile of record, "THE CACHE AND BRANCH AXIS")
is the new instrument, not a lead:** I1 misses 4.03 % of Ir with
`Arc::clone_from_ref_in` at one per 8.6 instructions, mispredicts
11.4 % (31.8 % indirect); nothing rate-ranked is large. It says the
`(-200)`/`(-201)` unshare direction and PGO are worth more wall-clock
than their Ir share, and that `bench_ab.py` is the arbiter for a leg
that touches a `match` or a clone path.

Before it:
**State at the `(-248)` tip (`(-245)`..`(-248)` on top of the concurrent
`(-243)`/`(-244)`, three-pool Ir against the `(-244)` tip `e44e9d90`):
`cube` 1,928,746,090 -> 1,883,972,930 (-2.321 %), `fixed` 690,547,383 ->
681,813,326 (-1.265 %), `sealed` 1,992,138,486 -> 1,967,055,576
(-1.259 %). Per-leg rows in the Baseline.**

**RE-READ AT the `(-248)` tip — the same three context tables on a
`--separate-callers=3` `cube` dump (1,883,973,787 Ir), so nobody
re-takes it.** `computed_permanent_hinted` 256,508 asks / 179.5 M
(9.53 %): the top rows are the `(-245)` table's floors unchanged
(`legal_blockers` 49,768 / 34.0 M, `permanent_value_with` 29,602 +
8,376 + 2,668 / 38.1 M, the block planner's `attacker_info` collect
13,262 / 20.2 M, the declaration's subset pass 7,882 / 16.3 M,
`pick_attacks_inner`'s own-side asks 6,986 + 6,378 + 2,166 / 19.5 M —
all consumed whole). The one row not in that table,
`intrinsic_land_mana_abilities <- activate_ability_inner` (666 asks /
3.3 M, ~5,000 Ir), is the `want_extra` branch's scope, whose gather
`granted_abilities_for` needs anyway; gating the land-type read alone
is the layer pass, ~0.7 M / 0.04 %. Not taken. `with_frozen_layers`
146,428 scopes / 222.2 M (11.8 %): `simulate_attack_outcome_once`
32,790 / 113.0 M (6.0 %) and `simulate_through_combat` 2,262 + 1,130 /
58.8 M are the probe design; the declaration's two scopes are 5,688 +
3,688 / 19.0 M after `(-246)`. `gather_continuous_effects_inner` 49,444
gathers / 117.8 M (6.25 %; 58 k at `(-241)`): `compute_permanents`
9,732 + 6,316 + 2,622 (the combat-damage and SBA views, one gather per
distinct state), `permanent_value_with` 7,212, the planner's collect
4,506, `pick_attacks_inner` 4,178 — one gather per scope per distinct
state, which is the freeze design. **Nothing in the three tables is a
consumer-read lead; the next device is structural** (a gather version
cannot come from CoW pointer identity — a uniquely-owned `Arc` mutates
in place — and holding the `Arc` to force a copy is the checkpoint's
`(-200)`/`(-201)` cost).

**RE-READ AT the `(-245)` tip — `computed_permanent_hinted`'s 284,812
asks by caller (218.6 M inclusive, 11.26 % of `cube`), which is where
both legs came from. What each row is, so nobody re-reads it:**

```text
  asks     incl Ir    caller <- context                                    what
  49,768   34.1 M     legal_blockers <- pick_blocks_inner                   (-194): scope-first misses, views consumed whole; floor
  29,602   26.4 M     permanent_value_with <- eval_material_inner           same; floor
  14,554   21.3 M     declare_blockers' second scope                        TAKEN (-246): three keywords, gated
  24,918   26.3 M     pick_attacks_inner (own attackers + opp CantBlock)    TAKEN (-245) for the opp half; the own half feeds may_declare_attacker, needed
  12,550   19.8 M     call_mut <- from_iter <- pick_blocks_inner            attacker_info's per-attacker views, consumed whole (pair gate, keywords); floor
   7,882   16.3 M     with_frozen_layers <- declare_blockers                the first scope's subset pass ((-215)); floor
  11,132    7.9 M     check_target_legality_with_source                     ~710 Ir an ask, mostly hits; not read
   1,410    5.9 M     permanent_is_creature <- from_iter <- SBA sweep       ~4,200 Ir an ask: out-of-scope misses; TAKEN (-247), the helper gated
   6,130    5.3 M     blocker_can_block_attacker (by id)                    all under pick_attacks_inner's legality collect; the pair check itself is 1.8 M of it
     666    3.3 M     intrinsic_land_mana_abilities <- activate_ability     ~5,000 Ir an ask: the want_extra branch's scope; rare
     884    2.7 M     push_ward_triggers_for_targets <- finalize_cast       the ungated sibling read; TAKEN (-248)
```

* **TAKEN as `(-247)` — `permanent_is_creature` under the SBA sweep:
  2,156 asks / 8.0 M, ~4,200 Ir each** (the CR 704.5n equipment-link
  check, out of any scope). The helper reads the printed line behind
  `card_type_change_in_scope`; `cube` -0.421 %, the other pools flat.
* **TAKEN as `(-248)` — `push_ward_triggers_for_targets`' 884 out-of-
  scope asks** were `push_first_targeting_counter`'s, the Ward gate's
  ungated sibling; `has_hostile_ward` took the same gate. Every pool
  ~-0.2 %.
* **Read and closed — `check_target_legality_with_source` (11,132 asks
  / 7.9 M, ~710 Ir):** its Shroud / Hexproof read is gated since
  `(-216)`, and the rest are in-scope hits whose consumers read
  keywords *and* colours *and* the controller (the hexproof-from-colour
  and ability-hexproof arms). Not one fact; a floor.
* **What is left in that table consumes the whole view**: the block
  planner's per-blocker and per-attacker facts, the material eval, the
  declaration's subset pass. The freeze design's floor — the next
  device is a gather version, which is structural.
* **Priced and not built — `pick_blocks_inner`'s gang / requirement /
  top-up passes ask the pair gate by id** (`blocker_can_block_attacker`
  re-finds both cards and re-asks both memos): 28 calls a six-game
  `cube` run. The 6,130-call row above is `pick_attacks_inner`'s
  legality collect, where the views are *not* in hand and the `all()`
  short-circuits on the first blocker that can block, so a pre-resolved
  blocker list would ask more views than it saves on the common board.
  Neither is a lead.
* **`blocker_pair_block` read: ~250 Ir a pair is a chain of keyword
  scans over two ~3-entry lists** (sector lock, three `has_kw`s, the
  attacker-keyword loop, `cant_block_pairs`, the pure gate) with an
  early-return per gate. Nothing to hoist; the count is the planner's.
  Floor.

Before it:
**State at the `(-241)` tip (`(-220)`, `(-222)`..`(-241)` less the three
refutations, three-pool Ir against the `(-219)` tip `52b9a743`): `fixed`
745,162,383 -> 696,705,741 (-6.503 %), `cube` 2,035,552,660 ->
1,941,530,315 (-4.619 %), `sealed` 2,085,024,159 -> 2,007,247,937
(-3.730 %).**

**RE-READ AT the `(-241)` tip — the two grep sweeps, so nobody re-runs
them.** Every `definition.static_abilities` read inside a whole-zone walk
(181 sites in 96 functions), ranked by the enclosing function's self Ir
on the `cube` dump: the nine walkers above ~3 M are `(-233)`..`(-241)`;
the residue is `empty_mana_pools` (gated on pool emptiness already),
`advance_step`'s draw-skip walk (once a turn), `scale_damage_to_inner`'s
three walks (behind the damage-scale lane), `cleanup_wear_off`, and a
long tail under 1 M each. Every `definition.keywords` read inside a
whole-zone walk (20 sites in 16 functions): the hot ones are the gather's
Bushido and flyer-count arms (per-card `DynamicPt`, exact), the SBA
sweep's `StartYourEngines` seat check (behind `scan.start_engines`),
Sunburst on resolution (an id find), and the two requirement evaluators'
name-sharing arms (per requirement, exact) — **nothing ungated; the
keyword presence questions went behind lanes at `(-188)`..`(-192)` and
`(-204)`.** The next grep, if any, is `triggered_abilities` reads under
`players.iter()` (hand / library / command walks per event), which the
dispatcher already gates by event kind.

**RE-READ AT the `(-241)` tip — the plain `cube` self table and a
`--separate-callers=3` `cube` dump (1,941,531,669 Ir), read as
allocation, growth, unshare and collect tables *by context*. Leads first
with their ceilings, then the floors, so nobody re-reads them.**

```text
  cube self at (-241), 1,941,530,315 Ir
    79.5 M  4.10 %  dispatch_triggers_for_events      floor ((-21)'s search count; 86.7 M at (-219))
    78.4 M  4.04 %  gather_continuous_effects_inner   58 k gathers; needs a version
    74.2 M  3.82 %  compute_permanent_pass            the layer pass (81.1 M at (-219); (-229)'s push loops)
    59.4 M  3.06 %  _int_free                         the allocator: with malloc 45.3 M, _int_malloc 43.0 M,
                                                      free 37.0 M and arena free 10.4 M, ~195 M / 10.0 %
    59.3 M  3.05 %  Vec::from_iter                    collects; the table below
    56.3 M  2.90 %  memcpy                            the unshares' and probe clones' element copies
    50.8 M  2.62 %  Arc::clone_from_ref_in            the CoW unshares' element clones
    46.1 M  2.38 %  check_state_based_actions_into    the sweep
    42.6 M  2.19 %  computed_permanent_hinted         read at (-219): a memo index is marginal
    38.5 M  1.98 %  perform_action_inner              floor
    28.5 M  1.47 %  fire_combat_damage_triggers       unchanged; still wants profiling-lines
    26.8 M  1.38 %  affected_includes_gated           floor
    25.2 M  1.30 %  declare_blockers                  after (-223)
    25.0 M  1.29 %  sba_board_scan                    instance-gated, floor
    23.6 M  1.22 %  FnMut::call_mut                   closure bodies under collects; (-232) showed the shim itself is nothing
    22.4 M  1.15 %  declare_attackers_banded          after (-222)
    19.2 M  0.99 %  resolve_combat_into               after (-225)
    18.2 M  0.94 %  bot::available_mana               floor ((-199) device)
    18.2 M  0.94 %  Vec::clone                        the unshared owners' Vec fields
    14.1 M  0.73 %  blocker_pair_block                49 k pair checks under pick_blocks_inner; not yet read
```

```text
  __rust_alloc by context (calls / Ir inclusive of the callee)
   213,799  14.4 M  grow_one                       every Vec growing from empty or doubling
   151,960  20.7 M  clone_from_ref_in <- make_mut_slow   the CoW unshares, all owners
    47,780   3.8 M  PrintedList::push <- compute_permanent_pass
    40,252   2.5 M  Vec::clone <- clone_from_ref_in      the unshared owners' Vec fields
    33,625   4.0 M  RawVecInner::reserve
    24,764   1.4 M  GameState::clone <- accept_on <- pay_census::in_probe   the probes
    23,576   1.3 M  dispatch_scan_card <- dispatch_board_scan

  grow_one by context (grow calls / Ir)
    48,364   1.4 M  push_mut <- activate_ability_inner   24,156 of them under auto_tap_for_cost_inner (2.9 M)
    47,152   1.3 M  dispatch_scan_card <- dispatch_board_scan   (+2.1 M under dispatch_triggers_for_events)
    19,760   1.1 M  IdSet::insert <- declare_attackers_banded
    19,396   0.5 M  affected_from_requirement <- selector_to_affected
    16,504   0.6 M  deal_combat_damage_to_target
    14,890   0.4 M  advance_step
    14,672   0.4 M  push_trigger_grants <- trigger_grant_sources
    13,816   0.8 M  resolve_combat_into
    13,316   2.5 M  push_mut <- declare_attackers_banded
    12,736   1.1 M  declare_blockers

  make_mut_slow by context (element clones under it / Ir)
   273,896  13.9 M  cast_spell_with_convoke        34,756 unshares / 29.9 M: ~7.9 owners a probe cast, 860 Ir each
    97,998   6.3 M  resolve_top_of_stack_inner
    45,788   1.8 M  declare_blockers
    41,912   2.1 M  on_left_battlefield <- remove_from_battlefield_to_graveyard_raw
    32,624   4.2 M  Battlefield::find_by_id_mut <- activate_ability_inner   the probe's first land tap unshares the board
    27,648   0.9 M  CardInstance::clear_end_of_turn_effects <- cleanup_wear_off   already behind end_of_turn_effects_are_clear
    27,004   0.6 M  declare_attackers_banded
    19,298   0.4 M  adjust_life <- deal_combat_damage_to_target
    18,540   1.2 M  run_effect;  18,054 / 1.0 M cleanup_wear_off;  17,920 / 0.7 M dispatch_triggers_for_events

  Vec::from_iter by context (collects / Ir inclusive)
   132,434  26.4 M  call_mut <- from_iter <- pick_blocks_inner <- with_frozen_layers   30,092 / 18.2 M of it is computed_permanent_hinted
    52,124  21.4 M  from_iter <- compute_permanents <- combat_damage_computed <- resolve_combat_into
    34,852  23.3 M  from_iter <- pick_blocks_inner <- with_frozen_layers <- simulate_attack_outcome_once
    34,184  11.0 M  from_iter <- check_state_based_actions_into <- resolve_top_of_stack_inner   the sweep's view collect
    32,700  11.9 M  from_iter <- compute_permanents <- declare_blockers
    69,802   3.6 M  printed_requirement_impl <- from_iter <- resolve_selector_inner
    42,300   4.1 M  from_iter <- resolve_selector_inner <- resolve_selector <- evaluate_predicate
    39,690   4.4 M  Map::fold <- from_iter <- declare_attackers_banded
    33,932   1.1 M  Chain::fold <- from_iter <- dispatch_triggers_for_events
    30,652   1.5 M  from_iter <- cast_spell_with_convoke
```

* **REFUTED as `(-242)` — `dispatch_board_scan`'s grant list: a fresh
  `Vec<TriggerGrant>` per dispatch, 23,576 allocations / ~3.5 M
  (0.18 %).** Priced as a `SmallVec<[TriggerGrant; 2]>` with the filter
  a `Cow`: `cube` +0.711 %, `fixed` +1.449 %. The allocator gave back
  4.9 M and the by-value `DispatchScan` cost 12 M of `memcpy` plus
  2.9 M of `SmallVec::drop` on every dispatch, grant or not. **Inline
  storage in a struct returned by value is a memcpy per call, not per
  allocation.** The Cow half alone is -0.27 M, noise. Do not rebuild;
  the Log entry has the self-table diff.
* **TAKEN as `(-243)` — a `Vec::push` from empty inside
  `activate_ability_inner`, 24,182 allocations / 2.9 M, all of them
  under the bot's `auto_tap_for_cost_inner`:** the activation's own
  two-event return `Vec`; now written into the auto-tapper's buffer
  (`cube` -0.323 %, every pool the same). Found by reading the body for
  a two-push `Vec`, no line profile: an inlined `Vec::push` leaves the
  dump's call-site position at `vec/mod.rs:*`, so the edge names the
  function but not the line.
* **Floor, re-read after `(-243)` — the selector collect under
  `evaluate_predicate`: 42,300 collects / 4.1 M plus the 69,802
  requirement evaluations inside them (3.6 M).** The table's "collects"
  are the `from_iter` *contexts* — the walk over the board with the
  requirement evaluated per card is charged to the collect; the
  allocation itself is 1,960 `__rust_alloc` calls (0.2 M) because an
  empty answer allocates nothing, and `resolve_selector` +
  `resolve_selector_inner` self are 0.9 M together. A visitor form
  would save under 0.05 %. **Rank a collect row by its `__rust_alloc`
  count, not its inclusive Ir — the Ir is the iterator's body.**
* **`PrintedList::push` — 47,780 pushes / 10.8 M inclusive (226 Ir each),
  every one under `compute_permanent_pass`, one allocation each already
  (`Box<[T]>`, the eighty-fourth pass).** What is not counted: a *second*
  push on the same list re-materializes the whole slice (`Box<[T]>` has
  no headroom by design). If a large share of the 47,780 are seconds, a
  `SmallVec<[Keyword; 4]>` override with headroom saves them at the
  `ComputedPermanent` byte cost already priced at +0.04 % / +0.058 % for
  eight bytes; count the seconds before pricing — probably half of the
  grants are a lone keyword and the answer is "no".
* **`blocker_pair_block` — 14.1 M self over 64,882 pairs (217 Ir each),
  ~49 k of them under `pick_blocks_inner`, plus `can_block_attacker_
  computed` 62,966 / 5.2 M beside it.** Read after `(-243)`: the self
  is nine short keyword-list scans per pair (`has_kw` on the blocker's
  computed keywords four times, `any` over the attacker's three times,
  `block_barred_by_protection_filter` and `blocker_matching_restriction_
  bars` inlined) plus `cant_block_pairs.contains` on a usually-empty
  `Vec`; `effective_ring_bearer` returns on `ring_bearer?` before its
  board walk. No walk, no gather: the one device is an evasion-family
  bitmask on `ComputedPermanent` folding the nine scans into two `&`
  tests, +2 bytes a view (they may fit in padding) and ~10 Ir per view
  to build over 227 k views, against ≤ 9 M saved — net ≤ 0.2 % `cube`
  and nothing on `fixed`. Not taken; a floor unless the planner's pair
  count grows.
* **TAKEN as `(-244)` — not from these tables but from the line
  annotation of the (-243) dump:** the dispatcher's one
  `push_ordered_trigger_candidates` call, 77,126 times for 806
  candidates, ~120 Ir of prologue, empty drain and `Vec` drops per empty
  batch (`fixed` -0.571 %, `cube` -0.337 %). The same read on the other
  top self rows: `dispatch_triggers_for_events`' 27 M own-line self is
  spread over 200 lines with nothing above 1.3 M (`match ev`), and
  `gather_continuous_effects_inner`'s largest own edge is the attached
  bonus scale count (`equipped_bonus.scale`, a board walk with a
  requirement per card, 9.6 M on `cube`) — per gather by nature, the
  gather-version memo again. `dispatch_board_scan` is 103 Ir a dispatch,
  `push_ordered_trigger_candidates`' non-empty remainder 0.19 M.
* **The same line read on the other top self rows at the `(-244)` tip
  (`cube`), so nobody repeats it:** `sba_board_scan`'s 16.5 M own-line
  self is seven instance-field reads per card per sweep (`flipped`,
  `controller != owner`, `attached_to`, `bestowed`, `sector`,
  `soulbond_partner`, `counters`), 2.2 M a line — instance fields
  written from dozens of sites, so neither a lane nor a per-card memo
  can hold them; floor. `fire_combat_damage_triggers`' 7.2 M own-line
  self is one board walk per call finding the source and OR-ing
  `soulbond_partner` (2.6 M) and the `LISTENER` bits walk (1.8 M); the
  rest is spread. `compute_permanent_pass`' 24.8 M own-line self is 39
  lines of 0.3–3.9 M over 276 k passes (the `ComputedPermanent` build
  3.9 M, prologue 3.6 M); its `PrintedList::push` edge is 47,780 *first*
  materializations (316 deallocs under `push`, so second pushes are
  0.7 %) at 222 Ir — 85 of them the allocation, 11 the `into_boxed_
  slice`, and 32,624 out-of-line `Keyword::clone`s for the payload
  keywords — so neither headroom nor a `SmallVec` override pays.
  `perform_action_inner`'s 16 M own-line self is the `match` and the
  `PassPriority` arm (8.6 M); `check_state_based_actions_into`'s 7.2 M
  own-line self is spread over 227 lines; `declare_blockers`' own lines
  are 1.6 M — everything else in it is the three layer passes. On the
  bot's side, `cast_candidates`' 60.6 M is `can_afford_in_state_with`
  over 33,758 candidates: 38.3 M of it the `available_mana` `OnceCell`
  fill (24,256 fills at 1,577 Ir, the `(-199)` floor) and ~600 Ir a
  candidate in the cost adjusters — `cost_reduction_for_spell_full_over`
  294 Ir (a walk of the precomputed source list's statics plus ~15
  `self_cost_reduction_*` definition-field checks), `can_afford_from`
  126, `extra_cost_for_spell_over` 94, the colour tax 52 — each spread
  over dozens of 2–3 Ir checks; a "has any self cost reduction"
  definition bit would fold ~1 M. `pick_blocks_inner`'s 42.7 M is
  `legal_blockers` (4,688 x 9.1 k Ir: the views and the pair checks read
  above).
* **Floors, so nobody re-prices them:** the allocator's ~195 M (10 %) is
  the sum of the contexts above, most of it the probe design — a
  `GameState::clone` per probe (24,764; 13.8 M self) followed by the
  ~7.9 unshares a probe cast makes (`cast_spell_with_convoke` 29.9 M)
  and the board unshare on its first land tap (`find_by_id_mut` 4.2 M),
  all of them the `(-200)`/`(-201)` per-owner floor. The
  `compute_permanents` collects under `combat_damage_computed`
  (21.4 M), `declare_blockers` (11.9 M) and the SBA sweep (11.0 M) are
  the gather-version memo the gathers entry rejects for want of a
  version; `pick_blocks_inner`'s two collects (26.4 M + 23.3 M) are the
  planner's own passes, the `(-194)` census. `clear_end_of_turn_effects`
  is already behind its emptiness gate; the 27,648 clones under it are
  the survivors' CoW'd groups. `IdSet::insert` and the `push_mut` under
  `declare_attackers_banded` (1.1 M + 2.5 M) are the attack search's
  per-declaration sets, `(-21)`'s count.
Before it:
**State at the `(-219)` tip (`(-216)`..`(-219)`, three-pool Ir against
the `(-215)`+fix tip `999da717`): `fixed` 763,717,868 -> 745,162,927
(-2.430 %), `cube` 2,090,168,791 -> 2,035,554,686 (-2.613 %), `sealed`
2,120,435,808 -> 2,085,022,232 (-1.670 %).** Before it, `(-204)`..
`(-215)` against `62a4e20b`: `fixed` -4.718 %, `cube` -4.997 %, `sealed`
-4.112 %; `(-199)`..
`(-203)` against `2003d1cf`: `fixed` -1.437 %, `cube` -1.627 %, `sealed`
-1.141 %; and `(-194)`..`(-198)` against
`0e9bdaa4`: `fixed` -2.929 %, `cube` -4.253 %, `sealed` -4.648 %. The
`cube` self table at `(-196)`, top rows:
`dispatch_triggers_for_events` 3.95 % (was 5.80 %), `gather_continuous_
effects_inner` 3.65 %, `compute_permanent_pass` 3.17 %, `Vec::from_iter`
2.99 %, `Arc::clone_from_ref_in` 2.60 %, `memcpy` 2.51 %, SBA 2.33 %,
`activate_ability_inner` 2.05 %, `computed_permanent_hinted` 1.96 %.

**RE-READ AT the `(-219)` tip — the plain `cube` self table (no
separate-callers dump this time) and the caller/callee tables of every
row above 1 % that was not already a floor. What was priced and why it
was not taken, so nobody re-prices it:**

```text
  cube self at (-219), 2,035,554,686 Ir
    86.7 M  4.26 %  dispatch_triggers_for_events      floor ((-21)'s search count)
    81.1 M  3.98 %  compute_permanent_pass            the layer pass (+10 M is the (-217) build's inlining of SmallVec::extend)
    78.4 M  3.85 %  gather_continuous_effects_inner   58 k gathers; needs a version
    60.6 M  2.98 %  Vec::from_iter                    collects; the SBA views and pick_by_outcome, read at (-215)
    46.1 M  2.27 %  check_state_based_actions_into    the sweep
    42.6 M  2.09 %  computed_permanent_hinted         see below
    38.5 M  1.89 %  perform_action_inner              306 Ir an action over 125,666: the match, a floor
    28.6 M  1.40 %  fire_combat_damage_triggers       ~1,400 Ir a call, diffuse walks; profiling-lines is the instrument
    27.6 M  1.36 %  declare_attackers_banded          see below
    26.8 M  1.31 %  affected_includes_gated           544,626 calls under the layer pass: 49 Ir each, a floor
    25.4 M  1.25 %  declare_blockers                  its {0} block-tax payment TAKEN as (-223); block_tax_for's per-blocker static walk (8,018 x 437 Ir) is what is left
    25.0 M  1.23 %  sba_board_scan                    21,392 sweeps x 1,170 Ir: instance fields, cannot be laned
    24.3 M  1.20 %  resolve_combat
    18.3 M  0.90 %  bot::available_mana               see below
```

* **`declare_attackers_banded` — 6,758 calls / 102 M inclusive (5.0 %),
  27.6 M self (4,086 Ir a call).** By callee: `compute_permanents`
  22.5 M (one layer pass per declaration, `&mut self`, the shape the
  `(-215)` re-read already closed), a `Vec::from_iter` 12,412 / 8.3 M,
  `auto_target_for_effect_avoiding_set_x` 716 / 4.7 M, `push_mut`
  26,862 / 3.8 M, `IdSet::insert` 26,460 / 2.6 M, `iter_mut` 13,230 /
  2.1 M, `board_keyword_in_scope` 6,758 / 2.0 M. The self is the
  validation body — a dozen `attacks.iter().any(..)` scans and the
  requirement loops, each already a `for` — and the call count is the
  attack search's. **Two of those walks were over the board, not the
  batch, and are TAKEN as `(-222)`** (the trigger member list; self
  27.6 M -> 22.4 M). What is left: the `groups` static walk (~230 Ir,
  no lane holds `AttackTogether`), the per-attacker `attacker_grants`
  build (three `Vec<TriggeredAbility>` an attacker, cheap on empty) and
  the batch scans. The count is `(-21)`.
* **`computed_permanent_hinted` — 42.6 M self over ~343 k asks (124 Ir
  each):** 168,044 memo hits (`LocalKey::with` 21.3 M — the lock and
  the `perms` scan) and 175,304 per-scope misses (`compute_permanent_
  pass` 72 M). The hit path is a linear `perms.iter().find` over a
  scope's ~10 entries of `(CardId, Arc)`; an index would save ~30 Ir an
  ask, ~10 M, against a `LayerFreezeState` byte budget `(-165)` already
  priced at ~6,800 Ir a byte. Marginal; not taken.
* **`bot::available_mana` — 12,518 calls / 35.2 M inclusive (1.7 %),
  18.3 M self, already once per decision behind a `OnceCell`:** the
  self is the per-untapped-source ability walk; `grant_scan` 6.7 M
  (534 Ir a call with the act-grant lane already in front of its
  battlefield leg) and `grants_nothing_slow` 36,752 / 4.2 M are the
  `(-199)` device at its floor.
* **`fingerprint` — 3,550 calls / 3.2 M left** after `(-219)` + `(-220)`
  (the non-land activations went with `(-220)`, deferred to the key
  repeat that reads them): all but 62 under `resolve_top_of_stack`, the
  CR 104.4b watch, one per trigger resolution against the previous one —
  consecutive by nature, so no deferral. Floor.
* **The CoW unshares are closed as a class after `(-217)`:** no
  `make_mut_slow` caller above 2,000 Ir a call has more than 732 calls;
  the ~900-Ir ones are `PlayerData` / zone unshares, the probe design.
* **Not priced, and the next thing to price:** `fire_combat_damage_
  triggers`' remaining 1,400 Ir a call needs a `profiling-lines` build
  (9 min cold) to say which of its five walks carries it — the listener
  walk on a `cube` board (usually `PRESENT`) is the guess; and the
  `Vec::from_iter <- check_state_based_actions_into` row (15 M at the
  `(-215)` re-read) is the death sweep's view collect, which is the
  layer pass's cost in a different coat.

**RE-READ AT the `(-215)` tip plus its fix — a `--separate-callers=3`
`cube` dump, the context tables of the largest remaining rows. Leads
first, then floors, so nobody re-reads them.**

* **`check_target_legality` — 19,380 calls / 26.9 M inclusive
  (1.29 %), and 3,848 of them gather** (`fx_pool::alloc_with <-
  computed_permanent_hinted <- check_target_legality`, 8.7 M): the
  check opens its own freeze scope and reads the target's computed
  view for Shroud / Hexproof / Protection / Ward, so every call that
  is not nested in an outer scope gathers the whole board. Its callers
  are the bot's `cast_candidates` auto-targeting (7,338 calls) and a
  target-enumeration collect (5,406). **The device is the fast path's
  presence gate, aimed at the target:** `card_keyword_possible_on`
  for the four keyword families (printed, `granted_keywords_eot`,
  keyword counters, a grant in scope) answers `false` on most targets
  without a view; only a `true` takes the scope. Ceiling ~1 % of
  `cube`; the audit is the same equality test `(-204)` used.
* **`compute_permanents <- combat_damage_computed` — 5,896 calls /
  32.5 M (1.56 %)**, one gather + id-subset pass per combat-damage
  computation, under `&mut self`. And `declare_blockers`' 4,612 /
  18.2 M, `declare_attackers_banded`'s 4,272 / 10.4 M — the same shape.
  Layer inputs move between the declare and damage steps (damage,
  deaths, counters), so the views cannot be carried across; what could
  be carried is the *gather* when nothing that feeds it moved — the
  cross-scope memo the gathers entry below rejects for want of a
  version. Not a lead until one exists.
* **The SBA sweep by caller — 21,222 sweeps / 199 M (9.5 %):**
  `resolve_combat <- advance_step` 4,286 x 15,846 Ir; **`resolve_combat
  <- submit_decision` 562 x 74,738 Ir** (the block declaration's damage
  step, a sweep with several deaths); `resolve_top_of_stack_inner`
  9,038 x 5,780. The death path is ~3,000 Ir a death after `(-203)`
  and `(-212)`/`(-213)` (its lane asks stopped refilling); the rest of a
  post-combat sweep is the per-sweep view collection (`Vec::from_iter
  <- check_state_based_actions_into` 6,020 / 15 M) and `sba_board_scan`
  (read above, instance-gated).
* **The CoW unshares by context — `make_mut_slow` 123,494 / 105 M
  (5.0 %):** `cast_spell_with_convoke` 34,800 / 30.0 M (a probe cast
  touches ~7 CoW'd owners: hand, stack, the payer's `PlayerData`, the
  battlefield through the first land's `find_by_id_mut`, the scratch
  and cold groups), `resolve_top_of_stack_inner` 10,458 / 10.5 M,
  `note_creature_death` 1,666 / 6.0 M (3,600 Ir each — the largest per
  unshare; read what it copies), `declare_blockers` 8,586 / 5.1 M,
  `find_by_id_mut` 8,156 / 4.5 M. The per-owner sizes are the
  `(-200)`/`(-201)` floor; the count is the probe design.
* `computed_permanent_hinted` 289,096 asks / 242.8 M inclusive
  (11.6 %): the `(-194)` census unchanged — `legal_blockers <-
  pick_blocks_inner` 49,768 / 38.7 M and `permanent_value_with <-
  eval_material_inner` 29,602 / 28.8 M are misses inherent to the
  freeze design, the next two are a `SmallVec::extend` and a
  `Vec::from_iter` under the block planner (22.8 M + 21.5 M, the
  planner's own passes). Floor.
* `dispatch_triggers_for_events` 143,852 / 147.8 M (7.1 %): 61,874 of
  them under the attack search's `sim_step` (60.8 M) — `(-21)`'s
  search-count decision, still not a dispatcher cost.
* `Vec::from_iter <- pick_by_outcome` 588 calls / 117.8 M (5.6 %) is
  the bot's outcome evaluation *inside* a collect, i.e. the search
  itself charged to the adapter; not an allocation lead.

**RE-READ AT `966289ae` (the `(-202)` tip) — a fresh `cube` self table
and a `--separate-callers=3` dump, ranked by caller. Rows and what they
say; the first two are leads, the rest are floors read so nobody
re-reads them.**

* **The death path — 10,116 `remove_from_battlefield_to_graveyard_raw`
  calls x ~4,800 Ir = 48.5 M (2.2 % of `cube`), under the SBA sweep
  (9,096 of them). The four board walks in it are TAKEN as `(-203)`**
  (`cube` -0.459 %; a source read found them, no line profile needed).
  What is left per death: `place_card_at_resolved_zone` ~1,260 Ir (the
  revert chain), `on_left_battlefield` ~1,080 (`find_card_anywhere_mut`
  across zones for a card that just moved, four list walks) and the raw
  self ~480, plus `note_creature_death` 7.3 M and `dying_snapshot` 4.9 M
  beside it in the sweep — each a line read (`profiling-lines`), none a
  lane. The lane's own misses are one walk a death (~390 Ir), structural.
  Read after `(-203)`, no build spent: the revert chain in
  `place_card_at_resolved_zone` (`turn_face_up`, `revert_flip`,
  `revert_transform`, `revert_prototype`, `reset_room_doors`,
  `reset_case`, `clear_effects_on_zone_change`'s `probe!`,
  `revert_copy_on_leave`) is already read-first at every step, so the
  ~1,260 Ir there is `send_to_graveyard`'s CoW unshare of the graveyard
  buffer (a real `Vec<CardInstance>` copy on a probe-cloned state) plus
  the gates themselves — a floor, not a device.
* **The SBA sweep is 10.5 % of the program inclusive (21,222 calls,
  232 M)** and its cost is *which* sweep: `resolve_combat`'s 4,286
  post-damage sweeps cost 17,760 Ir each (76 M) and the 562 under the
  bot's `submit_decision` 89 k each (50 M), against 7,000 for the 9,038
  after a stack resolution. The death path above is most of the
  difference; the rest is the per-sweep view collection (`Vec::from_iter`
  24,286 calls / 38.8 M — `compute_permanents` on the id subset the
  lethal-damage walk needs) and `sba_board_scan` at 1,231 Ir a sweep
  (26.3 M). The sweep's own self is 2,480 Ir a call (52.7 M): the
  per-permanent legality checks, diffuse — a line read if ever.
* **`compute_permanent_pass`'s `SmallVec::extend` — 399,380 calls /
  54.2 M (2.5 %) — is the layer filter loop itself and at its floor.**
  Line ~147: `sorted.extend(effects.iter().filter(affects))`, one pass per
  (scope, permanent) — `computed_permanent_hinted` memoizes the rest —
  and `(-156)`'s rule prices a hand loop at ~10 % of the adapter row. The
  `affected_includes_gated` inside it (26.9 M) was already refuted above.
* **`dispatch_triggers_for_events` 143,852 calls / 157 M inclusive
  (7.1 %): 42 % of it (61,874 calls, 65.8 M) is `perform_action_inner <-
  sim_step <- simulate_attack_outcome_once`** — the attack search's own
  sim steps at 1,063 Ir a dispatch. That is `(-21)`'s search-count
  decision, not a dispatcher cost; the dispatcher's self (89 M, 4.0 %)
  is the per-event bookkeeping the Standing rules already describe.
* `fire_combat_damage_triggers`: 73,534 calls / 45 M inclusive, 52,974
  of them the function calling itself (a per-event recursion at ~25 Ir —
  cheap); the 20,560 outer calls carry the cost at ~2,000 Ir, mostly the
  `by_kind: SmallVec<[Vec<DamageTrigger>; 4]>` build + `Flatten` drop
  per batch. A line read before anything; 0.4 % at most.
* `perform_action_inner` self 38.5 M over 125,666 calls (306 Ir): the
  checkpoint plus the action dispatch. Read by line if ever; diffuse.

Open, priced, largest first:

* **The printed-mana-ability fast path — TAKEN as `(-204)`** (Log:
  `sealed` -1.623 % / `cube` -1.509 % / `fixed` -1.496 %): the gate walk
  is gone for 91-99 % of land taps; the resolver was deliberately left
  alone. **What is left of the tap, per `cube` tap, priced at the
  `(-204)` tip** — the next devices, in order:
  * `run_effect`'s `AddMana` arm's Contamination / Pulse walk — **TAKEN
    as `(-205)`** (`cube` -0.156 %, `sealed` -0.114 %, `fixed`
    -0.032 %; `run_effect` self -4.57 M against +1.55 M of lane reads).
    What is left of the resolver per tap: `resolve_effect_into`'s
    238-Ir self (the ~30 scratch resets, plain stores — a floor), the
    `EffectContext` build and drop, `resolve_player`, the source
    `battlefield_find` (a memo hit), the `is_basic` read the turn-scoped
    replacements need, and the lane read itself — ~650 Ir, diffuse.
  * `card_type_change_unscoped`'s battlefield walk — **TAKEN as
    `(-207)`** (`cube` -0.912 % / `fixed` -0.764 % / `sealed` -0.622 %,
    2.5x the priced ceiling: the function was inlined into the SBA
    death sweep and the requirement walker, whose rows the caller table
    never showed). The lane word is an `AtomicU64` now — fifteen lanes
    free.
  * **A strip lane — REFUTED as `(-209)`** (`cube` -0.088 % but `fixed`
    +0.101 % / `sealed` +0.073 %): a lane asked once per activation
    pays more fills than it saves on the pools whose walk was cheap.
    `ability_strip_possible` (the dispatch-lane pre-gate) stands.
  * **The `continuous_effects` kind fold — TAKEN as `(-208)`** (small:
    the list is empty on most bot boards).
  * **The lanes' own fills — TAKEN as `(-212)` + `(-213)`**
    (`walk_and_store` 22.4 M -> 1.9 M on `cube`; a membership write
    keeps the lanes it cannot change and answers the rest off the one
    card it moved). The definition epoch was not the cost: a census
    counted 0-6 rewrites a game (`(-213)`'s Log). What is left: the
    first asks on a fresh board, `retain`'s demotions (it cannot name
    what it dropped), and `lanes_after_push`'s 3.3 M of per-card
    predicate calls — a floor unless the predicates get cheaper. The
    batch fill by memo word is moot now.
  * **`fire_combat_damage_triggers` — 29 M self after `(-210)`/`(-211)`,
    1,400 Ir a call, 98 % of calls push nothing.** Diffuse across its
    own walks (Log `(-211)`); a `profiling-lines` read is the
    instrument.
  * **`LocalKey::with` 16 M self — READ, not a lead:** 171,042 of its
    220,720 calls are `computed_permanent_hinted`'s memo-hit lookup
    inside the thread-local's closure, i.e. the `(-194)` hit path
    charged to the accessor, not thread-local overhead.
  * **Gathers — 58,426 a `cube` run, ~137 M inclusive (6.4 %):** 39,206
    under freeze scopes (`fx_pool::alloc_with`), 11,200 under
    `compute_permanents`' `&mut` callers, 7,260 under
    `computed_permanent_hinted`. `(-90)` read them as one per distinct
    state; a cross-scope memo would need a version over every layer
    input (counters, attachments, emblems, graveyard counts, life,
    phase …), which is the dirty-flag design ENGINE_BACKLOG rejected.
    Not a lead without that.
  * `card_keyword_possible_on(CantActivateTapAbilities)` 22,476 x 223 =
    5.0 M (0.23 %): the definition and instance legs are cheap, the cost
    is `keyword_grant_in_scope`'s `board_grants_keyword` walk — a
    per-board "can anything grant a keyword" bit is the same lane shape.
  * `board_has_mana_static` 25,166 x 106 = 2.7 M — the 106 were its
    inline fills after every membership change; `(-213)` took them
    (4.0 M -> 1.5 M at 47 k asks).
  * `find_by_id_mut` 24,388 x 220 = 5.4 M — the probe clone's CoW
    unshare of the tapped permanent; structural.
  * The two event pushes 4.2 M — the first push's allocation; a
    `with_capacity` moves it, nothing removes it.
* **The stripped printed mana ability — FIXED as `(-206)`** (CR 305.7
  / 613.1f; the fast path's strip read is `ability_strip_possible`,
  priced there and in the `(-209)` refutation).
* **The dispatch scan's walk — TAKEN as `(-215)`** (`cube` -0.678 %,
  flat elsewhere: the lane is a member list now). **`sba_board_scan`
  25 M (1.17 % of `cube`), READ and left:** one walk per SBA sweep,
  but half of what it folds is *instance* state (`flipped`,
  `controller != owner`, `attached_to`, `bestowed`, `soulbond_partner`,
  `sector`, the counter bag) that no definition-only lane may hold, so
  the walk stays and a member list would only skip the memo-word read
  on the cards that carry no `sba_scan_bits` — a fraction of ~1,200 Ir
  a sweep. Not a lane.
* **`mana_source_table`'s other 58 % of rows — TAKEN as `(-199)`** (Log:
  `cube` -0.644 % / `fixed` -0.561 % / `sealed` -0.354 %). The reason
  they missed was a grant static, an Equipment or a Soulbond pair
  *elsewhere on the board*, not the permanent's own static or counter;
  the gate now asks per permanent. What is left of the family on `cube`:
  `grants_nothing_slow` 100 k calls x 99 Ir (0.45 %), most of it the
  `EachPermanent` filter test per (permanent x grant static) — a
  per-scan "which permanents does this grant reach" mask would be the
  device, priced against `PrintedGrantFilter::test`'s 1.6 M; and the
  counter-grant lane's misses (`walk_and_store` +0.8 M, a memo-word read
  per permanent per membership change — below the bar on its own).

* **Cheap-on-empty clones — CLOSED at `(-201)`, the family is at its
  floor.** `(-200)` (`CardData`, `CounterBag`, `GameState::clone`) and
  `(-201)` (`PlayerData`'s seven lists) took `fixed` -0.51 % / `sealed`
  -0.49 % / `cube` -0.41 % between them, and both found their lists were
  *inlined* into `clone_from_ref_in` — the win was that row's self cost,
  not the `Vec::clone` row the candidate was priced on. The
  `--demangle=no` read of the rest (`fixed` at `(-201)`, 118,704
  out-of-line `Vec::clone` calls / 8.75 M): the largest monomorph is
  the zone buffer (`Vec<CardInstance>`, 14,692 real copies under the
  `CowBox` unshare, 3.4 M — refcount bumps, not empties); the two big
  `clone_from_ref_in` rows are `PlayerData` (11,182 unshares x 633 Ir)
  and `CardData` (13,626 x 431 Ir), i.e. the inlined field copies the
  size-class rules already price; what is left of the empties is ~1 M
  spread over a dozen small owners (the largest 9,492 unshares x three
  `Vec`s at ~32 Ir). Nothing there prices at a build.
* **`resolve_combat`'s protection asks — 0.2 %.** 13,164
  `damage_prevented_by_protection` calls (5.0 M) inside scopes that
  already hold `computed_of`; `protection_prevents_views` is the form.
  The SBA lethal-damage walk's 5,896 (3.1 M) are the same shape.
* **`fingerprint` — TAKEN as `(-219)` + `(-220)`** (the land taps
  behind the fast path, then the rest deferred to the key repeat that
  reads it; together `cube` -1.17 % / `fixed` -0.97 % / `sealed`
  -1.01 %). Read at the `(-219)` tip above: what is left is the CR
  104.4b resolution watch. Closed.

Refuted this run, no build spent or one: the kind-fold gate in
`fire_step_triggers` (`(-196)`, +0.2 M on `cube` — a one-element loop
over an inlined tag compare is cheaper than a memo load); the same gate
on the dispatcher's graveyard leg, by the same arithmetic (its per-trigger
test is a `matches!` on the scope).

**`(-21)`'s search-count half read at `e725e5c2` (Log): the census
`CRAB_ATTACK_CENSUS` says 9-16 % of searched declarations face no blocker
and greedy wins 94-100 % of those, so the open-board skip saves 1.3-1.8 %
on cube/sealed — but it overrides the sim's correct hold-back of an
attack-trigger creature and leans -0.1 pt on a 96 k-game sealed ladder.
Filed as an opt-in pilot (`atk-open`), NOT adopted; the reusable half is
the instrument, and a `greedy.len()>=2` gate is refuted off the dump (18 of
60 bench divergences are 2+ attackers and the biggest swings — a Goblin
Guide held back to not card the opponent).**

**`computed_permanent_hinted`'s memo-hit path — READ BY CONTEXT AT
`0e9bdaa4`, `(-194)` TAKEN (Log).** The `profiling-lines` read NEXT asked
for was the wrong instrument: `--separate-callers=3` ranks the 366,058
calls by who asks, and the top hit-path contexts were the block planner
asking for views it held. What the census leaves, on `cube`:

```text
  calls    incl Ir    context                                     state
  49,768  38.96 M     legal_blockers <- pick_blocks_inner         misses: the pass, inherent
  29,602  28.89 M     permanent_value_with <- eval_material_inner misses: leaf eval, inherent
  14,554  22.85 M     Map::next <- SmallVec::extend <- scope      misses
  13,262  21.54 M     attacker_info build <- pick_blocks_inner    misses + the scope's gather
  13,164   5.0 M      damage_prevented_by_protection <- resolve_combat   hits; `_views` form
   5,896   3.1 M      ... <- check_state_based_actions_into       hits; the lethal-damage walk
```

The miss rows are one layer pass per permanent per scope and the scope's
one gather — the price of the freeze design, not a device. The two hit
rows are the `(-194)` shape again at a fifth of the size (0.35 % between
them); `resolve_combat`'s strike-back loop already holds `computed_of`.

**Engine thread scaling — no global contention (measured `e725e5c2`,
`profiling-fast`).** 1,200 `fixed` games (gang mirror, seed 7): 16.5 s at
1 thread, 4.4 s at 4 = **3.75x on 4 cores (~94 % efficiency)**, repeatable
(16.1/16.8 s across two 1-thread runs). The paired loop is independent
seed-fixed jobs and the engine holds no cross-thread mutable state, so the
"find contention if sublinear" candidate is a negative on the engine loop.
What that does **not** cover is the actor's learner/actor shared replay
buffer and net — the selfplay-specific contention `(-52)` measures, which
`bot_ladder` never exercises. Not re-run this pass.

**RE-READ AT `a198daf3` (the `(-192)` engine + CR 400.7 + the mill/token
card fixes) — three fresh dumps, the map agrees with `(-90)`/`(-92)`, and
no row prices at a build.** `profiling-fast`, `--no-default-features`,
`--games 6 --threads 1 --seed 1`:

```text
  pool     a198daf3          vs (-192)
  fixed      833,934,847     +0.070 %   (catalog commits since; no engine change)
  cube     2,316,705,788     +0.061 %
  sealed   2,344,388,085     +0.071 %
```

Inclusive shape of `cube`, for orientation (the bot owns the game):
`next_action_inner` 93.5 %; `pick_attacks_scored` -> `simulate_attack_
outcome_once` **59.8 %** over 2,070 candidates (`(-21)`: the count is a
search-quality decision); `main_phase_action_with` 30.0 %; `sim_step`
26.5 %; `pass_priority` 25.4 %; `accept_on` 22.4 %; `cast_spell_with_
convoke` 16.0 %; `resolve_combat` 15.3 %; `perform_action` 13.5 %;
`with_frozen_layers` 11.4 %; `computed_permanent_hinted` 10.4 %;
`dispatch_triggers_for_events` 8.3 %; `check_state_based_actions_into`
6.7 %; `compute_permanents` 5.8 % (23,004 calls: `combat_damage_computed`
7,296 / `declare_attackers_banded` 6,758 / `declare_blockers` 5,036 / SBA
3,914 — every one an id subset, and 11,200 of them gather because they
run under `&mut self`).

Rows read this pass and closed by inspection, so nobody re-reads them:

* **`affected_includes_gated` 548,110 x 49 Ir (1.16 %) — the "per-pass
  reach mask" device is refuted by arithmetic.** 279,444 passes at ~2
  effects each: every (effect, permanent) pair is evaluated exactly once
  per pass already, and a pass is one `compute_permanent_pass` call for
  one permanent, so there is no second read for a mask to serve. The
  common anthem decomposes to `AffectedPermanents::All` (a seat compare
  and a type `contains`), not `CardMatch`; the 49 Ir is the call and the
  arm, and inlining it into the filter closure is a ~10 Ir/call bet
  (~0.2 %) that also grows the 293 Ir pass body. Not a candidate.
* **`make_mut_slow` 123,490 x 300 = 37.1 M (1.6 %), by caller:**
  `cast_spell_with_convoke` 38,084 / 34.7 M, `resolve_top_of_stack_inner`
  10,458, `declare_blockers` 8,586, `Battlefield::find_by_id_mut` 8,326
  (22,354 of its 24,886 calls are `activate_ability_inner`'s),
  `on_left_battlefield` 5,430, `resolve_combat` 5,296,
  `declare_attackers_banded` 3,376 + its `iter_mut` 3,376 (the one
  `Battlefield::iter_mut` caller). All real writes on a probe-cloned
  state — hand/stack/battlefield under a cast, block flags under a
  declaration. CoW as designed; no read-only `&mut` route found.
* **`Vec::clone` 469,736 x 51 = 24 M**: `GameState::clone` 139,412 (the
  non-`CowBox` fields, `(-13)`), `make_mut_slow` 137,594 (the unshared
  zone's buffer), `Arc::clone_from_ref_in` 123,300 (`CardInstance` under
  `Arc::make_mut`). Diffuse across the three.
* **`__rust_alloc` 981,466 by caller**: `finish_grow` 261,534, `from_iter`
  112,469, `make_mut_slow` 91,418, `Vec::clone` 64,219,
  `clone_from_ref_in` 60,446, `GameState::clone` 49,702,
  `PrintedList::push` 47,928, `CowBox::push` 38,312. The `(-163)`/`(-165)`
  census, unchanged in shape.
* **`IntoIter::drop` 401,462 x 25 = 9.9 M**: `effective_mana_abilities_
  into` 84,220, `declare_attackers_banded` 57,902, `fire_step_triggers`
  48,432, `check_state_based_actions_into` 42,444, three combat-damage
  trigger firers ~63 k. `for x in vec` over collected lists; `(-156)`'s
  rule prices each at ~10 % of its row.
* `DebugStruct::field` 114 k `write_str` calls = `wants_converge`, once
  per card name per process (How to measure). `Unfreeze::drop` 213 k x 34
  = 7.2 M: the lock on the three-in-four empty scopes is ~20 Ir, and a
  lock-free "memo present" flag is `(-47th)`'s depth-shadow refutation
  again (0.14 % ceiling). `card_can_reduce_toughness` 179 k x 42 = 7.5 M
  (0.32 %) under the toughness-reducer lane's re-walks — below the bar.

**What is left is the per-step engine cost under a search whose size is
deliberate, and the actor agrees with `cube` to within 30 % on every row
(Profile of record). The next perf lead is not in a self table: it is
either a search-count decision (`EVAL_TOP`, `attack_search`) measured on
strength as well as Ir, or a `profiling-lines` read of
`computed_permanent_hinted`'s 150 Ir/call self (366 k calls, 2.4 %) —
the one memo-hit path nobody has read by line.**

**Every entry from `(-188)..(-192)` down — ~10.5 k lines of taken, refuted
and closed candidates — moved verbatim to `PERF_ARCHIVE.md` at the `(-241)`
tip.** Read there before re-proposing anything numbered below `(-192)`:
`(-174)`'s open half, the `(-90)`/`(-92)` whole-profile maps and every
census the Standing rules cite by number are in it, unchanged.
