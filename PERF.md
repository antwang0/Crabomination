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
# ⚠ THE SECOND-WORKTREE FORM OF THE SAME RECIPE HAS A TRAP. A worktree that
# shares `target/` is keyed like the main tree (cargo hashes a workspace
# member by its path relative to the workspace root), so its build is "fresh"
# whenever its sources are older than the tip's outputs — `Finished in 0.16s`,
# and the base binary IS the tip's (2026-09-08: identical md5s, -501 Ir).
# `touch` the worktree's engine sources first and `md5sum` both sides before
# reading a number. CLAUDE.md's container notes carry the same warning.
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

**Sweep at the `(-253)` tip (2026-09-05):** 35 single-card printed-line
tests deleted across `modern` (14), `classic_sets` (15), `mh` (2), `stx`
(2), `core_rules` (1 — `c21::zetalpa_has_all_keywords`) — each asserted
only P/T, keywords, types, subtypes or cmc of one `catalog::` definition,
which `catalog_registration`'s oracle-backed audits already check for
every card. Kept: every multi-card table (`*_stat_lines`, `*_bodies_*`,
`*_printed_shapes`), every test asserting ability *structure* (a trigger
count, an effect variant, an alt cost), the serde and `effect_short_text`
tests, and the 18 `[CR]` rows. `find_data_tests.sh` still lists ~185
rows after this, nearly all of them the tables that are the sweep's
destination, so the list is not a delete count. Suite 19,255 -> 19,220,
-302 test LOC, no link-time claim (the section above measured that lever
dead).

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

### Serde derives — priced on the base crate 2026-09-06, the engine half pending

The routine's build-time list names "serde derives on the giant effect/state
types" as a plausible monomorphization cost that nobody had measured. The
base-crate half is now measured, by stripping every `Serialize` /
`Deserialize` derive, `#[serde(..)]` attribute and manual impl out of
`crabomination_base` in the working tree (93 derive sites, 716 attributes,
6 manual impls; a scratch script, reverted with `git checkout`) and timing
the warm check, `CARGO_TARGET_DIR=target-probe`, `touch card.rs` between
runs, the wide robustness grid running on three of the four cores
throughout:

```text
cargo check -p crabomination_base, warm
  serde on        7.01 / 7.28 / 7.22 s     then after the restore  7.23 s
  serde stripped  1.14 / 1.06 / 1.03 s
```

**~6.1 s of the base crate's 7.2 s check is the serde derive expansion and
the typecheck of what it expands to — 85 %.** That is a real number and a
small lever: a base-crate edit is followed by the catalog (619 k lines) and
the engine rebuilding against it, which is minutes, and the 6 s sits in
front of that chain. The engine crate's own derive cost cannot be measured
by stripping (its `GameState` / `GameEvent` derives need the base impls);
what *is* measurable there is the monomorphization share — the
`serde_json` instantiations that every engine build codegens — and
`cargo llvm-lines -p crabomination --lib` (dev profile, the tool installed
with `cargo install cargo-llvm-lines`) answers it:

```text
engine crate, LLVM-IR lines by family, at 0a0ee368 (the (-275) tip)
  TOTAL                          4,407,454 lines   112,687 copies
  serde (any)                    1,930,400   43.8 %
    serde_json::value::de          482,509   11.0 %   <- ONE call site: replace_creature_type_text's from_value
    serde_json::de:: (from_str)    238,229    5.4 %   the wire protocol (ClientMsg, crossplay Msg)
    derive visitors (_::)          999,368   22.7 %   Effect::deserialize alone 201,333 lines / 811 copies; Effect::serialize 143,654
  crabomination:: own              862,389   19.6 %
  Vec machinery                    402,157    9.1 %
```

**Nearly half of the engine crate's IR was serde, and a quarter of the
crate was one function.** `replace_creature_type_text` (CR 612.1,
Artificial Evolution) round-trips a `CardDefinition` through
`serde_json::Value` to rewrite a creature-type word without a per-variant
visitor — and that `from_value` was the crate's only `CardDefinition:
Deserialize` instantiation, which drags the whole `Effect` /
`StaticEffect` / `SelectionRequirement` tree in behind it.

Two readings, one refuted: routing the same call through `to_string` +
`from_str` instead **grew** the crate to 4,776,794 lines (+8.4 %: the text
deserializer's per-type machinery is larger than `Value`'s), reverted.
Moving the round trip into `crabomination_base::textrewrite` (a
non-generic function, so the instantiation is codegen'd where it is
written, not where it is called) took the engine crate to
**3,214,231 lines (-27.1 %)**, copies 112,687 -> 86,646; the base crate
went 250,838 -> 1,444,541 — the instantiation moved, and now it is paid on
a *base* edit (which rebuilds everything anyway) instead of on every
engine edit. `serde_json` becomes a real dependency of the base crate
(it was a dev-dependency); the graph gains nothing, the engine already
built it. What is left of serde in the engine (795 k lines) is the wire
protocol's `Deserialize` (`ClientMsg` and friends, 238 k + visitors) and
the `Serialize` side (`to_value` in `audit`, the deadlock dump, the
decision log) — the next lever of the same shape is the server's
deserialization, which `bot_ladder` and `selfplay_train` codegen and never
run.

Wall clock, `release-fast bot_ladder` rebuilt after `touch
crabomination/src/game/mod.rs` (the engine-edit loop) and after `touch
crabomination_base/src/card.rs` (the base-edit chain), the before side a
detached worktree at `0a0ee368` with its own target dir, ABBA:

```text
release-fast bot_ladder, touch game/mod.rs (engine + bin + link), 2 rounds A B B A
  before   171.4 / 169.8 / 173.4 / 171.2 s   mean 171.5
  after    161.4 / 167.9 / 164.4 / 168.1 s   mean 165.5      -3.5 %   (every B row below every A row)
dev --lib, touch game/mod.rs, CARGO_INCREMENTAL=0 (the engine crate alone at opt 0), 2 rounds
  before    61.7 / 61.6 / 67.8 s              mean 63.7       (one warm-up row discarded)
  after     56.9 / 55.8 / 54.0 / 58.7 s       mean 56.4      -11.5 %
dev --lib, touch game/mod.rs, incremental (the edit loop the suite pays), 2 rounds
  before     9.5 / 10.7 / 9.2 s               mean 9.8        (one warm-up row discarded)
  after      8.9 / 8.9 / 8.5 / 9.4 s          mean 8.9        -9 %
dev --lib, touch crabomination_base/src/card.rs, CARGO_INCREMENTAL=0 (base + catalog + engine)
  before   132.0 / 130.3 s                    mean 131.2
  after    127.1 / 130.4 s                    mean 128.8      -1.8 %   (the instantiation moved to base and the chain did not lose)
release-fast bot_ladder, touch crabomination_base/src/card.rs (the whole optimized chain)
  before   622.5 / 631.7 s                    mean 627.1
  after    598.8 / 603.1 s                    mean 601.0      -4.2 %
```

**Kept: every loop reads faster, none slower.** The IR fell 27 % and the
wall clock 3.5-11.5 % because what left was cheap-per-line serde visitor
code and what stayed is the engine's own giant functions (`run_effect`
alone is 151 k lines in one body), which is where an O3 build spends its
time; the dev profile (O0, cost ~linear in lines) is where the cut shows
most. The base-edit chain is flat to slightly better in both profiles —
the moved instantiation is codegen'd in `crabomination_base` while the
catalog builds beside it. **The rule that fell out: a `serde_json::
from_value::<T>` / `from_str::<T>` in a crate instantiates `T`'s entire
`Deserialize` tree *in that crate*; put the call in the crate that owns
`T`, as a non-generic function, and the codegen moves with it.** The next
candidate of the shape is the wire protocol's `Deserialize` (`ClientMsg`
and crossplay `Msg`, `serde_json::de::` 238 k lines plus their visitors),
which `bot_ladder` and `selfplay_train` codegen and never run. Sized at
the same tip: `serde_json::de::` 238 k + `GameAction` 72 k + `GameEventWire`
29 k + the views / `DecisionWire` / `CreatureType` / `Keyword` /
`SelectionRequirement` deserializers ~85 k = **~420 k lines, 13 % of the
crate** — but the device that moves it (a decoder closure handed to
`tcp_seat` / `ws_seat` / `tcp_client`, so the bin instantiates
`from_slice`) changes three signatures that `crabomination_client` calls
from four files, and that crate does not build here. Pull it from a box
where the client builds, or not at all. The `Serialize` side (~330 k:
`Effect` 75 k, `StaticEffect` 21 k, the text serializer 37 k) stays
regardless: the server's deadlock dump (`server/mod.rs`,
`to_value(state)`) needs `GameState: Serialize` in the lib, and
`GameState` reaches `Effect`.

**The wire half, taken 2026-09-07 without the signature change.** The
device is `#[inline]` on `tcp_seat` / `tcp_client` / `ws_seat` and
`CrossLink::recv`: rustc's mono-item collector roots a non-generic
function only when it is *not* cross-crate-inlinable, so an `#[inline]`
function with no in-crate caller is codegen'd in no crate at all, and the
`from_slice::<ClientMsg>` / `from_slice::<ServerMsg>` / `from_str::<Msg>`
trees it owns move to whichever binary calls it (`crabomination_server`,
the client, `bot_ladder`'s peer mode). Nothing in the lib calls the four;
`crabomination_client` type-checks against them unchanged (5 min cold,
after CLIENT_BACKLOG's four apt packages). What left with them was more
than the decoders: the coalescing writer's `to_vec::<ServerMsg>` (the
whole view tree's `Serialize`) was reachable only from the same bodies.

```text
cargo llvm-lines -p crabomination --lib, dev, at 03b53eab either side
  TOTAL                    3,217,700 lines / 86,669 copies  ->  2,778,508 / 79,833   (-13.65 % / -7.9 %)
  serde_core::de::           469,461  (14.6 %)             ->    105,060  (3.8 %)
  serde_json::de::           238,767  ( 7.4 %)             ->     52,928  (1.9 %)
  <crabomination::net::      107,527  ( 3.3 %)             ->     10,787  (0.4 %)
  any *erialize              735,086  (22.8 %)             ->    378,662 (13.6 %)
  bot_ladder release-fast    127,765,784 bytes             ->  126,628,472         (-0.9 %; the same engine tip)
```

Wall clock, ABAB with `git stash` as the A side, `touch game/mod.rs`
between every build, the box otherwise idle:

```text
release-fast bot_ladder (engine + bin + link)   A 139.8 s     B 142.8 / 150.6 s    (A1 discarded: 495 s, a cold engine after the llvm-lines fingerprint)
dev --lib, incremental                          A 7.57 / 7.42  B 7.03 / 7.61
dev --lib, CARGO_INCREMENTAL=0                  A 46.6 / 47.0  B 45.6 / 46.5        (-1.6 %, inside the spread)
```

**FLAT on every loop.** The 440 k lines that left were the cheapest kind
(visitor shells, one match arm per field) and none of them sat in a CGU
an engine edit dirties, so neither the incremental loop nor the O3 build
had been paying for them in time — only in IR and in the 1.1 MB of dead
decoder every simulator binary carried. Kept as a codegen-placement
change: the wire protocol is now instantiated by the process that opens
a socket and by nothing else, at four attributes. **The rule that fell
out narrows the one above: moving an instantiation out of a crate buys
wall clock only when the instantiation is *expensive per line* or sits
in a CGU the edit loop rebuilds; `serde_json::value::de` (the `Value`
walker, -27 % IR for -3.5..-11.5 % time) was, the text decoder's visitors
were not.** What is left of serde in the lib (`serde_core::de::` 105 k,
`serde_json::de::` 53 k) is `replay.rs` / the puzzle corpus / the decision
log reading `Value`, plus the `Serialize` side the deadlock dump pins.

### `(-276)` `run_effect`'s frame — the 32 MB stack requirement, priced and a third of it taken 2026-09-07

The routine's candidate list carries "effect-resolution recursion depth (the
32 MB-stack requirement)" and nobody had asked *why* every worker thread is
spawned with `stack_size(32 * 1024 * 1024)`. The prologue answers it in one
`objdump`: **`run_effect` had a 97,256-byte frame, and it is the only
function in the whole binary past 4 KB** (`scripts`-less census: one
streamed `objdump -d` over `bot_ladder`, every probed or >= 16 KB prologue;
two hits, the other is `gimli` at 1 byte). Every call runs the inline probe
loop — 23 `movq $0,(%rsp)` page touches — and every level of effect
nesting (a `Seq` in a trigger in a `Reflexive` ...) keeps ~95 KB: 32 MB is
~340 levels, a default 2 MB thread ~20. The next-largest frames are
`main_phase_action_with` 4,056, `gather_continuous_effects_inner` 3,976,
`submit_decision` 3,576, `finalize_cast` 3,432; `pass_priority` is 72.

What is in it (the `profiling-fast` `.dwo`, `llvm-dwarfdump
--debug-info=<concrete DIE> -c`, every `DW_OP_fbreg` slot, the gap to the
next slot as its size — `scripts/frames.py` and `scripts/dwarf_frame.py`,
whose docstrings carry the three-step recipe): 1,112
distinct slots, and the frame is **a few whole `CardDefinition`s by value
(8,232 bytes each) plus a long tail LLVM's stack colouring did not merge**
— 28 arms each own a 208-byte `PendingEffectState` slot, 7 a `StackItem`,
one an `ActivatedAbility` (2,016). The 8 KB slots were: two transform arms
cloning `back_face` (`(**b).clone()`), `CreateTokenCopyOf` editing a
by-value clone, the basic-land factory's by-value return, Grist's
`Box::new(insect.clone())` and the `TokenDefinition` literal beside it, an
`Option<CardDefinition>` from `lookup_by_name`, and two *unnamed* 8 KB
temporaries — `Arc::make_mut`'s inlined clone path, at the 12 sites that
rewrite a card's definition in place.

**The device is the same one every time: the 8 KB value must be born in a
frame that dies.** `CardDefinition::clone_arc` / `boxed_clone`,
`TokenDefinition::boxed_clone`, `CardData::definition_make_mut`,
`draft::basic_land_arc`, `catalog::lookup_arc_by_name`, and
`effects::grist_insect_token` are all `#[inline(never)]` one-liners whose
only job is to own that temporary; `Box::new(self.clone())` in such a
helper compiles to a 48-byte frame (the clone is built in the allocation).
Two things that do NOT work, both tried: `Arc::make_mut` on a fresh `Arc`
(its shared-path clone inlines anyway), and any form that hands the value
back by value.

```text
release-fast bot_ladder, run_effect's frame
  before                            97,256 bytes  (probe loop 23 pages)
  first pass (five arms boxed)      72,568
  second pass (+ make_mut, lookup)  64,312          (probe loop 16 pages)   -33.9 %
callgrind, dflt mirror --games 6 --threads 1 --seed 1, profiling-fast, system allocator, the 41ca3089 tip either side
  first pass   sealed 3,283,015,188 -> 3,279,823,194  (-0.097 %)   cube 3,639,696,059 -> 3,635,489,731  (-0.116 %)
  second pass  sealed 3,283,015,188 -> 3,274,597,848  (-0.256 %)   cube 3,639,696,059 -> 3,629,217,631  (-0.288 %)
  outcomes identical on every dump (72 / 48 decided, 0 undecided); suite 19,246 / 0 / 5
```

A robustness lever first and a small throughput one second: the probe
loop is ~5 Ir a page, and most of the -0.26 / -0.29 % is the twelve
`make_mut` bodies that no longer inline into `run_effect` (the clone
path's `memcpy` and allocator calls were laid out in the hot function's
own code; `definition_make_mut`'s frame reads 0 bytes — it tail-calls). The tail — 28
`pending` slots that are one variable in 28 arms — is the 970-arm `match`
itself, and the only device for that is the split into per-family
functions, which the file-size section above rightly says is not a
build-time lever and this section says is a frame one. Sized: ~8.5 KB of
`PendingEffectState` slots, ~2.3 KB of `StackItem`, so after the 8 KB
values are gone the floor of the current shape is ~60 KB.

## Baseline

Closing states from the `(-185)` tip down are in `PERF_ARCHIVE.md`, verbatim.

### `(-279)` — the pooled event scratch: closing state at the `(-279)` tip, THE NEW A/B BASE ON ALL THREE POOLS

One engine leg after the `(-278)` addendum below (Log `(-279)`; the
block-sim context read in the candidates). Behaviour-preserving — every
trace identical, `--bench` counters identical — so the three totals below
are the `(-278)` games, cheaper, and the base for every later A/B.

```text
  sealed dflt, callgrind --games 6 --threads 1 --seed 1, UNTRACED, one tree:  2,572,706,282 -> 2,562,977,750 Ir  (-0.378 %, 72 / 72)
  cube   dflt, same recipe:                                                   2,515,278,324 -> 2,508,428,873 Ir  (-0.272 %, 48 / 48)
  fixed  gang, same recipe:                                                     673,786,345 ->   671,549,557 Ir  (-0.332 %)
  traces CRAB_DUMP_TRACES both sides, sealed + cube: 72 + 48 files, 0 differ
--bench release-fast (mimalloc) at 3d8f3072: 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok; thread_determinism ok (3 vs 1);
        bin_bytes 126,743,528 (+2,784 B against 6acd58c3); 285.1 / 297.8 / 291.7 games/s, three single runs (host_calib_ms 50 / 53 / 53) — THE BOX, NOT THE
        CHANGE: the 6acd58c3 profiling-fast binary reads 272.9 median on the same afternoon against its recorded ~520; paired bench_ab.py tip vs (-279),
        12 pairs, B/A median +0.99 % / mean +1.10 % (sd 3.97) — inside the instrument's noise, the right sign, Ir is the number
suite   19,286 / 0 / 5 (one lib test added: `game::event_scratch_tests`); golden 7/7 unmoved
clippy  --workspace --exclude crabomination_client --all-targets   clean
gate    profiling-fast (release-fast opt settings, debug-assertions off) and release-fast bot_ladder both built from the tree
sweep   fresh seeds on the (-278) tip binary (profiling-fast): {all, sealed} x seeds 701..706 x {dflt, gang} mirror x --games 400 --threads 3, CRAB_CAP_DIAG=4000:
        24 cells / 139,200 games, 0 panics, 0 cap, 0 stuck, 16 draws (all rc 0, the diag silent)
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.10 GHz, 4 cores; cold profiling-fast bot_ladder 13m07s (cold registry), warm engine rebuild 4m05s,
        cold debug suite build ~9 min, release-fast 11m42s
```

### `(-278)` and round 71 — the chains read the menu's scores, the block chain gate refuted: closing state at the `(-278)` tip, THE NEW UNTRACED DEFAULT-PILOT BASE

One bot-side leg and one refuted round after the round-70 addendum below
(Log: `(-278)`, round 71; ML_NOTES round 71; the candidates' "THE BLOCK
CHAIN SPLIT"). `(-278)` is behaviour-preserving — every trace identical —
so the six-game dflt totals below are the round-70 games, cheaper; the
untraced pair is the base for every later A/B (the round-70 Log's totals
were traced, +5.5 %). `--bench` (`gang`, `fixed`) carries no chain and is
byte-identical in its counters.

```text
  sealed dflt, callgrind --games 6 --threads 1 --seed 1, UNTRACED, one tree:  2,725,852,967 (the round-71 tip) -> 2,572,709,132 Ir  (-5.62 %, same games, 72 / 72)
  cube   dflt, same recipe:                                                   2,645,404,719 -> 2,515,279,341 Ir  (-4.92 %, same games, 48 / 48)
  traces CRAB_DUMP_TRACES both sides, both pools: 72 + 48 files, 0 differ
  round 71   bchain-skipg / bchain-empty / bchain-seed: sealed wall 0.886 / 0.900 / 0.900, ladder 48.50 / 47.67 / 48.30 pooled — every cell wholly below 50, PARKED
--bench release-fast (mimalloc) at the (-278) tip: 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok; thread_determinism ok (3 vs 1);
        bin_bytes 126,740,744 (-4,120 B against 1d98b202); 541.2 / 494.7 / 522.7 games/s, three single runs on the quiet box (host_calib_ms 53 / 66 / 57 — this host's spread)
suite   19,286 / 0 / 5 (one test added: the three round-71 arms on the round-56 board and the greedy block); golden 7/7 unmoved
clippy  --workspace --exclude crabomination_client --all-targets   clean
gate    cargo check --profile release-fast -p crabomination --bin bot_ladder   clean
audit   audit_stubs 0 flagged; audit_incomplete --structural-only 0 to review (21,795 cards)
actor   callgrind selfplay_train --actors 1 --games 60 --steps 1 --seed 7 at the (-278) tip (profiling-fast, system allocator): 2,490,213,350 Ir, 6,063 rows —
        -20.2 % against 2c63ce52's 3,121,212,158 (different games: round 70 + (-278)); the recorded shape (candidates, "THE ACTOR RE-READ AT THE (-278) TIP")
grid    scripts/robustness_grid.sh --wide, PILOTS with dflt prepended, at the round-71 tip (the round-70 default, the first --wide run under it):
        52 ladder cells / 301,600 games on `all` + `sealed` x 26 seeds x 400 — 0 panics, 0 assertion fires, 0 stuck, 4 cap / 12 draw (the caps: seeds 53
        and 73 on `all`, the recorded Beacon of Immortality fingerprint, unchanged); 2 actor cells x 30,000 games (seeds 7 / 20260901, 71.2 / 67.8 games/s
        on the audit build — 62.7 / 60.1 before round 70) 0 failures; 46 pilot cells (dflt + the script's 45) 0 failures. Exit 1 is the four caps, by the script's rule
census  CRAB_ATTACK_CENSUS=1 sealed 14,400 games seed 43 at the round-71 tip: block chain 4.77 sims a search, split greedy / nobody / other 46.6 / 27.6 / 25.8 % of the
        sims, won 24.9 / 48.7 / 40.1 %; at the (-278) tip (1,200 games): attack chain sims 21,096 + served 10,288, block chain 30,386 + 6,026
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.10 GHz, 4 cores; cold debug suite build 5m08s (a warm registry), cold profiling-fast bot_ladder 8m03s,
        warm engine rebuild 2m51s; release-fast 14m30s under the grid's three threads
```

### Round 70 — the attack chain gated on the menu's outcome: closing state at `1d98b202`, THE NEW DEFAULT-PILOT BASE

One bot-side throughput leg after the `(-277)` addendum below (Log, ML_NOTES
round 70): the attack chain skipped where the menu alone picked a non-empty
greedy, gated for no loss and adopted. **The dflt six-game totals below are
a different pilot's games** — quote them as the base from here, never as an
engine change against the `(-277)` block. `--bench` (`gang`, `fixed`) is
byte-identical in its counters.

```text
  sealed dflt, callgrind --games 6 --threads 1 --seed 1:  3,315,951,121 (the (-277)+2 tip) -> 2,882,893,998 Ir  (-13.06 %, different games)
  cube   dflt, same recipe:                               3,144,754,672 -> 2,820,089,306 Ir  (-10.32 %, different games)
  wall   200 x 12 mirrors, 5 paired reps, median arm/dflt:  sealed 0.841 / cube 0.870
  gate   sealed 49.9 / 50.0 / 50.1 / 50.3 (seeds 43/97/151/199, 12,000 games a cell, pooled 50.08); cube 49.8 / 50.0; fixed 49.9 / 50.1 — no loss
--bench release-fast (mimalloc) at 1d98b202: 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok; thread_determinism ok (3 vs 1);
        bin_bytes 126,744,864 (+3,280 B: the gate and the census slots); 502.0 / 506.9 / 454.1 games/s, three single runs on the quiet box (this host's spread)
suite   19,285 / 0 / 5; golden 7/7 unmoved (no seeded game reaches a search the gate changes)
clippy  --workspace --exclude crabomination_client --all-targets   clean
sweep   fresh seeds on the ADOPTED DEFAULT under the debug-assertions overflow build (target-audit, 8 assertion strings): 601..603 x {sealed, cube, fixed}
        x --games 400 --threads 3 = 9 cells / 28,800 games, 0 undecided, 0 panics / assertions / overflows, every rc 0, CRAB_CAP_DIAG=4000 silent
census  CRAB_ATTACK_CENSUS=1 sealed 1,200 games seed 43: attack 14,572 searched / 49,464 candidates / chain sims 52,224 before the gate;
        chain won when the menu alone picked greedy/nobody/holdback 642/992/238 of 1,218/992/406 chained searches (the split behind the round)
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.10 GHz, 4 cores
```

### `(-277)` — the last printed-only hooks read instance grants, and the walk that inlined: addendum at the `(-277)` tip

One engine commit after the addendum below (the Log's `(-277)` entry has the
five-row build table and the delta table): the three combat listener walks
read `granted_triggers_timed`, every hook outside the dispatcher goes through
`any_granted_trigger_of_kind` + `for_each_triggerer_or_all`, and the walk's
single call site inlined the step and cast hooks' closures. The fixed pool
is byte-identical; sealed and cube are the same games faster:

```text
--bench release-fast (mimalloc) at the (-277) tip: 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok; thread_determinism ok (3 vs 1);
        bin_bytes 126,693,632 (+6,560 B against 72297152's 126,687,072; 126,688,776 at 4db20c51 in this container); 513.0 / 503.0 / 428.1 games/s, three single
        runs (453.3 / 505.8 / 506.5 at 4db20c51, the same container) — THIS CONTAINER IS A DIFFERENT HOST (Xeon @ 2.10 GHz, host_calib_ms 53) from the
        addendum below's 2.80 GHz box, so its games/s do not compare with any earlier addendum; the counters and Ir do
Ir      against 4db20c51, traced, base and tip from one tree, --a dflt --b dflt --games 6 --threads 1 --seed 1, system allocator:
        sealed 3,320,021,324 -> 3,315,651,668 (-4,369,656, -0.132 %) / cube 3,148,739,249 -> 3,144,748,521 (-3,990,728, -0.127 %); 72 + 48 trace files, 0 differ
golden  7/7 unmoved
suite   19,282 / 0 / 5 at (-277) (one test added: the three walks on a grant with no printed trigger); 19,285 / 0 / 5 at the run's tip (the grant-bucket
        ratchet, Calix, All-Out Assault); the two rules changes after (-277) read sealed +0.009 % / cube +0.000 % against it, traces identical (Log)
clippy  --workspace --exclude crabomination_client --all-targets   clean (two needless_borrows fixed after the Ir reading — `&listens` -> `listens`, a ZST closure by copy)
gate    cargo build --profile release-fast -p crabomination --bin bot_ladder   clean (the debug-assertions=off build behind --bench)
audit   audit_stubs 0 flagged; audit_incomplete --structural-only 0 to review (21,795 cards); audit_panics.py 78 sites / 0 bare; audit_variant_coverage.py 0 dead capabilities
--bench at the run's tip (2c63ce52, the two rules changes in): counters identical (195,806 / 27.49 / 611.9 / 0), determinism + thread_determinism ok,
        bin_bytes 126,741,584 (+47,952 B: the new delayed kind's arms), 501.6 / 522.5 / 521.3 games/s under the grid's pilot leg — the host's spread
actor   callgrind selfplay_train --actors 1 --games 60 --steps 1 --seed 7 at 2c63ce52 (profiling-fast, system allocator): 3,121,212,158 Ir, 6,240 rows — the
        recorded shape (candidates, "THE ACTOR RE-READ AT THE RUN'S TIP"); the two closure rows are std's FilterMap and the Arc pool, floor
grid    scripts/robustness_grid.sh --wide, PILOTS with dflt prepended, at 2c63ce52: 52 ladder cells / 301,600 games on `all` + `sealed` x 26 seeds x 400 —
        0 panics, 0 assertion fires, 0 stuck, 4 cap / 12 draw (the caps: seeds 53 and 73 on `all`, twin i32::MAX life totals at turn 2,159 / 2,490 with
        a library of 0-1, the closed Beacon of Immortality fingerprint, read with CRAB_CAP_DIAG=1); 2 actor cells x 30,000 games (seeds 7 / 20260901,
        62.7 / 60.1 games/s on the audit build under a concurrent callgrind) 0 failures; 46 pilot cells (dflt + the script's 45) 0 failures.
        The script's own exit is 1 for the four caps — a capped game is a defect cell by its rule; the boards say it is the recorded one
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.10 GHz, 4 cores; cold debug suite build ~25 min, cold profiling-fast bot_ladder 8m42s, warm engine
        rebuild 2m23s; cold release-fast ~12 min
```

### 2026-09-08 — the trigger-grant duration carrier, the planeswalker damage record, the two hooks and the wrapper peel: addendum at the tip after `72297152`

Three engine commits and one catalog commit after the Life Matrix addendum
below (ENGINE_BACKLOG, first section; the Log's two "READ" entries have the
Ir A/Bs and the delta tables). Rules changes on the grant map, two damage
branches, two hooks and one gather leg, priced and flat; the fixed pool is
byte-identical:

```text
--bench release-fast (mimalloc) at 72297152: 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok; thread_determinism ok (3 vs 1);
        bin_bytes 126,687,072 (-71,016 B against the Life Matrix tip's 126,758,088; 126,725,176 at 005a9b8c); 311.0 / 295.4 / 287.7 games/s, three single
        runs (299.6 / 300.8 / 317.7 at 005a9b8c) — the host's spread, not a change
Ir      against 901c66f4, untraced, at 005a9b8c: sealed 3,023,237,395 -> 3,025,048,377 (+0.060 %) / cube 2,966,378,057 -> 2,968,192,944 (+0.061 %);
        traced, the base rebuilt beside each tip: 005a9b8c sealed +0.064 % / cube +0.056 %, 72297152 a further +0.027 % / +0.001 % — every trace file
        identical to the base's on both pools (72 + 48 games, action for action); the residue is read by function in the Log (empty-map gates, then glibc)
golden  7/7 unmoved
suite   19,278 / 0 / 5 at 005a9b8c (three tests added, all failing on the previous engine), 19,281 / 0 / 5 at 72297152 (the catalog commit's two and the peel's one)
clippy  --workspace --exclude crabomination_client --all-targets   clean (one type_complexity in a new test, aliased)
gate    cargo build --profile release-fast -p crabomination --bin bot_ladder   clean at both tips (the debug-assertions=off build behind --bench)
audit   audit_catalog_stats.py: tim 1 -> 0 with the sorcery-for-your-turn leniency retired (17,229 cards; the other columns unmoved at 20 / 1 / 3 / 6 / 3);
        audit_stubs 0 flagged; audit_incomplete --structural-only 0 to review (21,795 cards)
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores; a cold profiling-fast bot_ladder is 14m30s here, the warm engine rebuild 10m under a
        concurrent build; a second git worktree sharing target/ builds a base side warm (deps by package id, the workspace crates re-keyed by path)
```

### Round 68 — closing state at the round-68 tip, THE NEW DEFAULT-PILOT BASE

One bot-side throughput leg (Log, ML_NOTES round 68): the attack sim's
main-phase casts capped at one per sim, adopted at no loss. **The dflt
six-game totals below are a different pilot's games** — quote them as
the base from here, never as an engine change against the `(-275)`
block. `--bench` (`gang`, `fixed`) is byte-identical. The round-68
commit was rebased onto the concurrent session's serde relocation
(`41ca3089`, a build-time leg, "Serde derives" above) and then onto
`(-276)` (the addendum below, an engine leg landed concurrently); the
six-game dumps below predate both, the suite / clippy / `--bench` lines
were re-taken on the tip rebased onto `41ca3089`.

```text
  sealed dflt, callgrind --games 6 --threads 1 --seed 1:  3,277,686,484 (round-67 tip, this container) -> 3,017,209,691 Ir  (-7.95 %, different games)
  cube   dflt, same recipe:                               3,635,747,593 -> 2,953,897,394 Ir  (-18.75 %, different games)
  actor  selfplay_train --actors 1 --games 60 --steps 1 --seed 7 (profiling-fast -p crabomination_ml --no-default-features):
         3,120,856,905 -> 3,094,007,397 Ir  (-0.86 %, different games: 6,078 -> 6,198 rows);  393 k -> 327 k Ir a sim (-17 %)
  wall   200 x 12 mirrors, 5 paired reps, median cap1/uncapped:  sealed 0.864 / cube 0.837
suite   19,247 / 0 / 5 (120 s) at the round-68 tip; golden traces 7/7 (seeds 3, 4 and c0ffee re-blessed for the round, seeds 1, 2, 5 unmoved)
clippy  --workspace --exclude crabomination_client --all-targets   clean
release release-fast build of bot_ladder: clean
--bench release-fast (mimalloc) at the round-68 tip: 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf;
        determinism ok; 290 games/s at --threads 3 (games_per_s_th 96.7, host_calib_ms 49; bin_bytes 125,892,976)
sweep   fresh seeds on the ADOPTED DEFAULT (release-fast, the round-68 tip): 601..603 x {sealed, cube, fixed} x --games 400 --threads 3 =
        9 cells / 28,800 games, 0 undecided (0 cap / 0 stuck / 0 draw), 0 panics, every rc 0, CRAB_CAP_DIAG=4000 silent
        AND the training binary: release-fast selfplay_train --actors 4 --steps 1, seeds 911 / 912 x --games 3000 / 6000 =
        9,000 games / 902,097 rows, 0 stalls, both rc 0 — 165.6 / 162.5 games/s (`actors:`) on an otherwise idle 4-core box, the capped default's first uncontended reading
grid    scripts/robustness_grid.sh (debug-assertions, overflow profile) at the round-68 tip, PILOTS="dflt sim-cast-off sim-cast0 sim-cast2" PILOT_GAMES=40 --pilots:
        green — 30 ladder cells (33,120 games, 0 undecided) + 3 actor cells (seeds 1 / 7 / 23 x 600 games, the actors on the capped default) + 4 pilot cells (680 games each),
        0 failures, no panic / assertion / overflow; 10 assertion-string lines in the audit binary. Run because the round changes what the bot proposes; the grid's own
        pilot list does not carry `dflt`, so the pilots leg is how the default pilot gets audited — pass PILOTS explicitly.
scaling release-fast selfplay_train --steps 1 --seed 31, --actors 1 / 2 / 4 x 600 games an actor, quiet box: 54.7 / 103.9 / 235.2 games/s (`actors:`) — linear to 4;
        the (-275)-era 96 % reading holds under the capped default
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores
```

### 2026-09-07 — `GrantActivatedAbilityToMatching`'s permanent carrier and the five upkeep gates: addendum at the tip after the Life Matrix commits

Two commits after the duration addendum below (ENGINE_BACKLOG, first
section): one engine arm (the Log's "READ" entry has the Ir A/B) and one
catalog shape on five cards. The fixed pool is byte-identical:

```text
--bench release-fast (mimalloc), the engine commit: 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok;
        thread_determinism ok (3 vs 1); bin_bytes 126,758,088; 275.0 games/s single run
Ir      against 09b9f056: sealed 3,023,234,030 -> 3,023,237,395 (+0.000 %) / cube 2,966,367,971 -> 2,966,378,057 (+0.000 %), outcome lines identical (Log)
golden  7/7 unmoved
suite   19,275 / 0 / 5 at the engine commit (one test added), the same count at the catalog commit (that test extended); clippy clean; release-fast typecheck clean
```

### 2026-09-07 — the keyword-grant carriers and `UntilNextTurn`: addendum at the tip after the duration commit

One engine commit after the `1f2cabcb` addendum below (ENGINE_BACKLOG,
first section; the Log's "READ" entry has the Ir A/B). A rules change on
four effect arms and the duration map, priced and flat; the fixed pool is
byte-identical:

```text
--bench release-fast (mimalloc): 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok; thread_determinism ok (3 vs 1);
        bin_bytes 126,762,112 (+16,784 B: the four arms' collects and the two helpers); 253.0 games/s single run under a suite build
Ir      against 1f2cabcb: sealed 3,022,028,711 -> 3,023,234,030 (+0.040 %) / cube 2,966,685,176 -> 2,966,367,971 (-0.011 %), outcome lines identical (Log)
golden  7/7 unmoved
suite   19,274 / 0 / 5 (one test added, one extended across the turn cycle); clippy clean; release-fast typecheck clean
sweep   fresh seeds on the profiling-fast binary of the duration tip: 610..612 x {sealed, cube, fixed} x --a dflt --b dflt --games 400 --threads 2 =
        9 cells / 28,800 games, 0 undecided, 0 panics, every rc 0, CRAB_CAP_DIAG=4000 silent
```

### 2026-09-07 — the LKI-walk EOT-grant read and the conditional rider gate: addendum at the tip after `1f2cabcb`

One engine commit after the addendum below (ENGINE_BACKLOG, first
section; the Log's "READ" entry has the Ir A/B). A rules change on the
dispatcher's died-snapshot walk, priced and flat; the fixed pool is
byte-identical:

```text
--bench release-fast (mimalloc): 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok; thread_determinism ok (3 vs 1);
        bin_bytes 126,745,328 (+576 B); 276.9 / 290.1 games/s single runs (3 threads, the host's spread)
Ir      against e52c6191 (the run's base, re-read: sealed 3,021,668,348 / cube 2,966,365,259): sealed 3,022,028,711 (+0.012 %) / cube 2,966,685,176 (+0.011 %),
        outcome lines identical on both dumps (Log)
golden  7/7 unmoved
suite   19,273 / 0 / 5 at 1f2cabcb (two tests added, both failing on the previous engine); audit_stubs 0 flagged; audit_incomplete --structural-only 0 to review
clippy  --workspace --exclude crabomination_client --all-targets   clean
gate    cargo check --profile release-fast -p crabomination --bin bot_ladder   clean (the debug-assertions=off typecheck)
sweep   fresh seeds on the profiling-fast binary of 1f2cabcb (release-fast's opt settings + debuginfo): 607..609 x {sealed, cube, fixed} x --a dflt --b dflt
        --games 400 --threads 2 = 9 cells / 28,800 games (4,800 / 3,200 / 1,600 a cell), 0 undecided, 0 panics, every rc 0, CRAB_CAP_DIAG=4000 silent
profile the sealed self table re-read at the tip: the (-271) shape, no new row (Perf candidates, first entry)
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores; a cold profiling-fast bot_ladder is 12m30s here, the warm engine rebuild 3m56s
```

### 2026-09-07 — the trigger-hook fixes (`triggers_on_equipment`, `once_per_turn`, `dealer_filter`): addendum at the tip after `b3f6067b`

Four commits after the addendum below (ENGINE_BACKLOG, first two
sections; the Log's two "READ" entries have the Ir A/Bs). Rules changes
on the grant list and the two hardcoded hooks, priced and flat:

```text
--bench release-fast (mimalloc): 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok; thread_determinism ok (3 vs 1); bin_bytes 126,744,752; 379.3 / 264.3 games/s single runs (3 threads, the second under a suite build)
Ir      against d04a225d: sealed dflt 3,020,686,102 -> 3,021,669,628 (+0.033 %) / cube 2,963,669,195 -> 2,966,364,550 (+0.091 %), outcomes identical (Log, both entries)
golden  7/7 unmoved
suite   19,271 / 0 / 5 at the run's tip (19,267 at b3f6067b, 19,270 at the once/dealer_filter tip); audit_stubs 0 flagged; audit_incomplete --structural-only 0 to review
clippy  --workspace --exclude crabomination_client --all-targets   clean
gate    cargo check --profile release-fast -p crabomination --bin bot_ladder   clean (the debug-assertions=off typecheck)
sweep   fresh seeds, release-fast: 601..603 at b3f6067b and 604..606 at the once/dealer_filter tip, x {sealed, cube, fixed} x --a dflt --b dflt --games 400
        --threads 3 = 18 cells / 57,600 games, 0 undecided, 0 panics, every rc 0, CRAB_CAP_DIAG=4000 silent
grid    scripts/robustness_grid.sh --pilots, PILOTS="dflt" PILOT_GAMES=40, at 78e57bd2 (the once/dealer_filter tip; the debug-assertions build took
        the tree with every hook change in): green — 30 ladder cells (33,120 games, 0 undecided: cap 0 / stuck 0 / draw 0) + 3 actor cells (seeds
        1 / 7 / 23 x 600 games, 60.6-64.3 games/s) + 1 pilot cell (dflt, 680 games, 0 undecided), 0 failures, no panic / assertion / overflow;
        9 assertion-string lines in both audit binaries. Run because the grant list and both hooks changed, and the `equip_grants ==
        equip_granted_trigger_sources()` cross-check is a `debug_assert!`.
```

### 2026-09-07 — the `trig` column, the leaves-the-battlefield fix and the wire `#[inline]`: addendum at the tip after `03b53eab`

Three commits after the `(-276)` addendum: the sixth audit column and 40
catalog cards on their printed trigger event (INCOMPLETE_CARDS "Trigger
events"), the engine's non-death leaves event + the `BecomesUntapped`
fan-out (ENGINE_BACKLOG, first section), and the wire decoders out of the
lib ("Serde derives" above). The engine leg is a rules change and could
have moved games; it did not move the fixed pool:

```text
--bench release-fast (mimalloc): 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok
        games/s 340.8 (before) / 394.9 (after) single runs, i.e. the host's spread, not a reading; bin_bytes 127,765,784 -> 126,628,472
golden  7/7 unmoved (the seeds' games bounce and exile nothing that carried a leaves trigger)
suite   19,253 / 0 / 5 at 03b53eab (two Looter il-Kor fixtures gained a library — the Looter loots on any damage now); 19,265 / 0 / 5 at a83993a9, clippy clean
build   engine LLVM IR 3,217,700 -> 2,778,508 lines (-13.65 %); wall clock flat on all three loops (the table above)
clippy  --workspace --exclude crabomination_client --all-targets   clean;  cargo check -p crabomination_client   clean
```

```text
grid    scripts/robustness_grid.sh --pilots, PILOTS="dflt" PILOT_GAMES=40, at 03b53eab (the audit build took the tree before the scope commits):
        green — 30 ladder cells (33,120 games, 0 undecided: cap 0 / stuck 0 / draw 0) + 3 actor cells (seeds 1 / 7 / 23 x 600 games, 61.5-68.6 games/s on a
        busy box) + 1 pilot cell (dflt, 680 games, 0 undecided), 0 failures, no panic / assertion / overflow; 9 assertion-string lines in both audit binaries.
        Run because the leaves fix changes what a bounce does.
sweep   fresh seeds on release-fast at a83993a9's catalog (the Tapped fan-out in, the last seven card fixes half in): 601..603 x {sealed, cube, fixed} x
        --games 400 --threads 3 = 9 cells / 28,800 games, 0 undecided, 0 panics, every rc 0, CRAB_CAP_DIAG=4000 silent;
        --bench on the same binary 195,806 / 27.49 / 611.9 / 0 — counters identical to 2003d1cf, 367.6 games/s, bin_bytes 126,638,608
```

### `(-276)` — addendum to the closing state, at the `(-276)` tip

One more behaviour-preserving leg after the closing state below (outcomes
identical on every dump, `--bench` counters unmoved, golden 7/7): the
base for it is the tree at `41ca3089` (round 67's picker + the serde
relocation), which reads sealed 3,283,015,188 / cube 3,639,696,059 —
+1.5 % / +1.9 % on the `(-275)` totals below, the picker's different
games. Quote these as the base from here.

```text
  sealed dflt, callgrind --games 6 --threads 1 --seed 1:  3,283,015,188 -> 3,274,597,848 Ir  (-0.256 %)
  cube   dflt, same recipe:                               3,639,696,059 -> 3,629,217,631 Ir  (-0.288 %)
  frame   run_effect 97,256 -> 64,312 bytes (release-fast); still the only frame in the binary past 4 KB
suite   19,246 / 0 / 5 at the (-276) tip; golden traces 7/7 unmoved; clippy clean
--bench release-fast (mimalloc) at the (-276) tip: 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok; thread_determinism ok
build   engine LLVM IR 4,407,454 -> 3,214,231 lines under the serde relocation ("Serde derives" in the build-time section), engine-edit rebuild -3.5 % release-fast / -11.5 % dev
```

### `(-275)` — closing state at the `(-275)` tip

One behaviour-preserving engine leg (outcomes identical on every dump,
`--bench` counters and golden traces unmoved), the last priced lead off
the `(-274)` actor re-read. **The base was re-taken at `0876f5f3`**:
two deck-work commits (`4b09dcb0`, `0876f5f3` — `trick_modes_combat_only`
into the default, two flags off) landed after the `(-274)` closing state
and moved the *default pilot's* six-game runs — sealed
3,368,479,285 -> 3,238,972,578 (-3.8 %), cube 2,582,609,088 ->
3,577,188,860 (**+38.5 %**), actor 3,154,050,563 -> 3,147,098,694
(-0.2 %, 6,080 -> 6,074 rows). Those are different games, not a costlier
engine: `--bench` (`gang`, `fixed`) is byte-identical, and
`pick_combat_trick` is 7.3 M of the cube run. Quote the new totals as
the base from here; the `(-274)` block's numbers describe a pilot that
no longer ships.

```text
  sealed dflt, callgrind --games 6 --threads 1 --seed 1:  3,238,972,578 -> 3,235,138,032 Ir  (-0.118 %)
  cube   dflt, same recipe:                               3,577,188,860 -> 3,571,024,045 Ir  (-0.172 %)
  actor  selfplay_train --actors 1 --games 60 --steps 1 --seed 7 (profiling-fast -p crabomination_ml --no-default-features):
         3,147,098,694 -> 3,142,639,712 Ir  (-0.142 %);  60 games / 6,074 rows / 0 stalls both sides
suite   19,242 / 0 / 5 (125 s) at the (-275) tip; golden traces 7/7 unmoved
clippy  --workspace --exclude crabomination_client --all-targets   clean
release release-fast build of bot_ladder and selfplay_train: clean
--bench release-fast (mimalloc) at the (-275) tip: 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf;
        determinism ok; thread_determinism ok (3 vs 1 threads identical); 189 games/s at --threads 3 with a build running beside it (not a throughput reading)
sweep   fresh seeds on the ADOPTED DEFAULT (release-fast, the (-275) tip): 601..603 x {sealed, cube, fixed} x --games 400 --threads 3 =
        9 cells / 28,800 games, 0 cap / 0 stuck, 2 `draw` (cube 601 — a legal simultaneous loss, not a stall), 0 panics, every rc 0
        AND the training binary: release-fast selfplay_train --actors 4 --steps 1, seeds 911 / 912 x --games 3000 / 6000 =
        9,000 games / 897,316 rows, 0 stalls, both rc 0 (98-101 games/s with the ladder sweep and the grid build sharing the box — not a throughput reading)
grid    scripts/robustness_grid.sh (debug-assertions, overflow profile) at the (-275) tip: green — 30 ladder cells (5 pools x 6 seeds x 120 games = 33,120 games, 0 undecided) + 3 actor cells (seeds 1 / 7 / 23 x 600 games),
        0 failures, no panic / assertion / overflow; both audit binaries carry the assertion strings (8 lines). Run because (-275) lands a debug_assert! in gather_continuous_effects.
        AND --wide at the same tip (first since df27df7e, sixty passes and the round 55-67 bot changes ago): ladder 52 cells / 301,600 games,
        cap 4 / stuck 0 / draw 12 — the four caps are seeds 53 and 73 on `all`, the documented Beacon of Immortality board (ENGINE_BACKLOG "CLOSED —
        the two stall-sweep leads"), the same two seeds and the same count df27df7e read; actor leg 2 x 30,000 games (seeds 7 / 20260901,
        --actors 3 --steps 2, ~50 games/s on the audit build; that binary was built from the tree carrying the serde relocation below) 0 failures;
        pilots leg 45 decision policies x 12 games on `all`, 0 failures. ~2 h 20 min of box time end to end.
audits  audit_panics.py: 78 sites off the bin/test paths, 67 guarded, 11 lock-poison, 0 bare;  audit_variant_coverage.py: 0 dead capabilities, the same 2 dead primitives
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores
```

### `(-272)`..`(-274)` — closing state at the `(-274)` tip

Two behaviour-preserving engine legs off the growth census and the
call-count table of a fresh sealed `dflt` base at the `(-271)` tip
(outcomes identical on every dump, `--bench` counters and golden traces
unmoved), plus `(-272)` built, read flat (-0.018 %) and reverted. The
tip binary re-dumped after the revert: its totals are cand4 + the
revert's 0.6 M / 0.4 M exactly.

```text
  sealed dflt, callgrind --games 6 --threads 1 --seed 1:  3,378,571,112 -> 3,368,479,285 Ir  (-0.299 %)
        (-272) -0.018 % [reverted]  (-273) -0.097 %  (-274) -0.202 %
  cube   dflt, same recipe:                               2,583,686,536 -> 2,582,609,088 Ir  (-0.042 %)
        (-272) -0.016 % [reverted]  (-273) -0.068 %  (-274) +0.027 %
        (this base against the (-271) record's 3,378,573,605 / 2,583,687,288: -2.5 k / -0.8 k Ir, a different host)
  actor  selfplay_train --actors 1 --games 60 --steps 1 --seed 7 (profiling-fast -p crabomination_ml --no-default-features), the (-274) tip
         against the (-271) record:  3,160,759,000 -> 3,154,050,563 Ir  (-0.212 %);  60 games / 6,080 rows / 0 stalls both sides
         (one binary, no (-271) actor rebuilt here: the sealed base matched its record to 2.5 k Ir, so the record is the base)
suite   19,239 / 0 / 5 (123 s) at the (-274) tip; golden traces 7/7 unmoved
clippy  --workspace --exclude crabomination_client --all-targets   clean
release release-fast build of bot_ladder (the typecheck gate and more): clean
--bench release-fast (mimalloc) at the (-274) tip: 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf;
        determinism ok; thread_determinism ok (3 vs 1 threads identical); 304 games/s on this host (a slower box than the (-271) record's 506)
sweep   fresh seeds on the ADOPTED DEFAULT (release-fast, the (-274) tip): 501..503 x {sealed, cube, fixed} x --games 400 --threads 3 =
        9 cells / 28,800 games, 0 undecided, 0 panics, every rc 0 (sealed 4,800 games in 31-43 s a cell)
        AND the training binary itself, first time in a closing state: release-fast selfplay_train --actors 4 --steps 1, seeds 909 / 910 x
        --games 3000 / 6000 = 9,000 games / 900,512 rows, 0 stalls, both rc 0 (194.7 / 210.2 games/s on this 4-core host)
audits  audit_panics.py: 78 sites off the bin/test paths, 67 guarded, 11 lock-poison, 0 bare;  audit_variant_coverage.py: 0 dead capabilities, the same 2 dead primitives
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.10 GHz, 4 cores
```

### `(-266)`..`(-271)` — closing state at the `(-271)` tip

Five actor-only legs on the recorder and the encoder (outcomes identical
on every dump; `--bench` counters, golden traces and the ladder pools
unmoved by construction — nothing a `bot_ladder` game runs was touched),
all read off the actor recipe, then one engine leg off the actor's
growth census that moved every pool (`(-271)`), plus the `spell_kind`
memo leg built, read -0.105 % and reverted (Log). The concurrent
deck-work commit `b3fd2c43` landed between `(-266)` and `(-267)`
(bot.rs: two gated flags, both off), so the base was re-taken there;
its own delta on the actor is +0.011 %.

```text
  actor  selfplay_train --actors 1 --games 60 --steps 1 --seed 7 (profiling-fast -p crabomination_ml --no-default-features):
         3,246,053,464 -> 3,160,759,000 Ir  (-2.628 %)  across (-266) -0.317 %, b3fd2c43 +0.011 %, (-267) -0.229 %, (-268) -1.007 %, (-269) -0.465 %,
         (-270) -0.265 %, (-271) -0.382 %;  60 games / 6,080 rows / 0 stalls / 6,566 encoded states on every side
  sealed dflt, callgrind --games 6 --threads 1 --seed 1:  3,393,159,334 -> 3,378,573,605 Ir  (-0.430 %)   [(-271) alone; (-266)..(-270) touch nothing a ladder game runs]
  cube   dflt, same recipe:                               2,591,645,196 -> 2,583,687,288 Ir  (-0.307 %)
         (the (-265) record's 3,385,114,447 / 2,584,225,619 -> this base is b3fd2c43's +0.24 % / +0.29 %, its two gated bot flags' checks)
suite   19,239 / 0 / 5 (75 s) at the (-269), (-270) and (-271) tips; golden traces 7/7 unmoved; 32 encoder tests (two new: the totals fold, the packed word vs every catalog card)
clippy  --workspace --exclude crabomination_client --all-targets   clean (one manual_is_multiple_of in the concurrent deck_gauntlet bin, fixed here)
release release-fast typecheck of bot_ladder: clean
--bench release-fast (mimalloc) at the (-269) and (-271) tips: 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok (506 games/s on this host)
sweep   fresh seeds on the ADOPTED DEFAULT (release-fast, the (-268) tip + b3fd2c43): 401..403 x {sealed, cube, fixed} x --games 400 --threads 3 =
        9 cells / 28,800 games, 0 undecided, 0 panics, every rc 0
audits  audit_panics.py: 78 sites off the bin/test paths, 67 guarded, 11 lock-poison, 0 bare;  audit_variant_coverage.py: 0 dead capabilities, the same 2 dead primitives
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.10 GHz, 4 cores
```

### `(-261)`..`(-265)` — closing state at the `(-265)` tip

Four behaviour-preserving engine commits on top of the round-63 read
(outcomes identical on every dump, `--bench` counters and golden traces
unmoved): three read off the sealed `dflt` profile re-taken at the
`(-260)` tip (the "SEALED `dflt` RE-READ" block in candidates — the sim's
spell layer by callee), the fourth off the actor re-read at the `(-264)`
tip (Profile of record). `(-263)`, the enumerator's graveyard hint, was
built, read flat and reverted; the census it asked for found `(-264)`.
Dumps are `profiling-fast` with the system allocator
(`--no-default-features`, one `-p` per build — two `-p` flags left
`bot_ladder` on mimalloc once this run, caught by `nm`).

```text
  sealed dflt, callgrind --games 6 --threads 1 --seed 1:  3,437,314,145 -> 3,385,114,447 Ir  (-1.519 %)
        (-261) -0.485 %  (-262) -0.303 %  (-264) -0.201 %  (-265) -0.538 %   [(-263) -0.020 %, reverted]
  cube   dflt, same recipe:                               2,603,519,085 -> 2,584,225,619 Ir  (-0.741 %)
        (-261) +0.037 %  (-262) -0.243 %  (-264) -0.040 %  (-265) -0.496 %   [(-263) -0.014 %, reverted]
  actor  selfplay_train --actors 1 --games 60 --steps 1 --seed 7, (-264) -> (-265) tips:  3,297,447,030 -> 3,238,000,614 Ir  (-1.803 %)
  Ir base for the three-pool gate: unchanged from (-250) — not remeasured
suite   19,234 / 0 / 5 (123 s) at each of the four tips; golden traces 7/7 unmoved
clippy  --workspace --exclude crabomination_client --all-targets   clean
release the release-fast build of bot_ladder (the typecheck gate and more): clean
--bench release-fast (mimalloc): 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf;
        determinism ok; thread_determinism ok (3 vs 1 threads identical)
sweep   fresh seeds on the ADOPTED DEFAULT (release-fast): the (-264) tip, 211, 223 x {sealed, cube} x --games 120 --threads 3 =
        4 cells / 4,800 games, 0 undecided, 0 panics, every rc 0;  the (-265) tip, 301..305 x {sealed, cube, fixed} x --games 400
        --threads 3 = 15 cells / 48,000 games, 0 undecided, 0 panics, every rc 0 (sealed 4,800 games in ~23 s a cell)
audits  audit_panics.py: 78 sites off the bin/test paths, 67 guarded, 11 lock-poison, 0 bare;
        audit_variant_coverage.py: 0 dead capabilities, the same 2 dead primitives
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores
```

### `(-257)`..`(-260)` — closing state at the `(-260)` tip

Four behaviour-preserving engine commits on top of `(-256)` (outcomes
identical on both dumps, `--bench` counters and golden traces
unmoved), each read off the actor-path map's libc/std rows by caller.
The dumps here are `profiling-fast` with the **system allocator**
(`--no-default-features`), so the totals sit above the `(-256)` row's
mimalloc figures; the base was retaken at this tip and matches the map's
total to 0.004 %.

```text
  sealed dflt, callgrind --games 6 --threads 1 --seed 1:  3,547,131,260 -> 3,483,259,284 Ir  (-1.801 %)
        (-257) -0.903 %  (-258) -0.114 %  (-259) -0.548 %  (-260) -0.246 %
  cube   dflt, same recipe:                               2,585,999,842 -> 2,572,001,381 Ir  (-0.541 %)
        (-257) -0.085 %  (-258) -0.010 %  (-259) -0.337 %  (-260) -0.111 %
  Ir base for the three-pool gate: unchanged from (-250) — the `gang` bench path pays the same legs, not remeasured
suite   19,230 / 0 / 5 (97 s); golden traces 7/7 unmoved
clippy  --workspace --exclude crabomination_client --all-targets   clean
release the release-fast build of bot_ladder (the typecheck gate and more): clean
--bench release-fast (mimalloc): 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf;
        determinism ok; thread_determinism ok (3 vs 1 threads identical)
sweep   fresh seeds on the ADOPTED DEFAULT (release-fast): 211, 223 x {sealed, cube} x --games 120 --threads 2 =
        4 cells / 4,800 games, 0 undecided, 0 panics, every rc 0
audits  audit_panics.py: 78 sites off the bin/test paths, 67 guarded, 11 lock-poison, 0 bare
rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores
```

### `(-256)` — closing state at the `(-256)` tip

One behaviour-preserving engine commit on top of round 60 (outcomes,
census, `--bench` counters and golden traces identical).

```text
  sealed dflt, callgrind --games 6 --threads 1 --seed 1:  3,313,679,418 -> 3,262,741,089 Ir  (-1.537 %)
  cube   dflt, same recipe:                               2,470,130,419 -> 2,406,910,404 Ir  (-2.559 %)
  paired wall clock, sealed 200 x 12, 7 reps:             0.991 median / 0.988 mean
  Ir base for the three-pool gate: unchanged from (-250) — `gang` runs no sims
suite   19,230 / 0 / 5; golden traces 7/7 unmoved
clippy  --workspace --exclude crabomination_client --all-targets   clean
release the release-fast typecheck gate: clean
--bench release-fast (mimalloc): 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf
```

### Round 60 — closing state at the round-60 tip

One adoption commit on top of `(-255)`, a search-budget change gated
for no loss (`.ladder/run_r60_open.sh`): `attack_skip_open` on the
default. `round58_default()` frozen as `dflt58`, the base rounds 59 and
60 read on.

```text
  sealed dflt58 mirror, 200 x 12, 5 paired reps:  dflt-open 0.959 median / 0.958 mean
  cube  dflt58 mirror, 200 x 12, 5 paired reps:   dflt-open 0.939 median
  cumulative vs the r56 default:  sealed 0.851 x 0.984 x 0.959 = ~0.80 (~1.7x gang); cube 0.750 x 0.977 x 0.939 = ~0.69
  scaling, sealed dflt 200 x 12 on this 4-core host: 70.4 / 67.8 / 67.4 games/s per thread at 1 / 2 / 4 threads
        (96 % linear at 4 — no contention on the chain path; the (-52) reading stands)
sweep   fresh seeds on the ADOPTED DEFAULT (the earlier sweeps ran `gang`): 8 primes 211..251 x {all, cube} x
        --games 120 --threads 2 = 16 cells / 24,000 games, 0 undecided, 0 panics, every rc 0
audits  audit_incomplete --structural-only 21,795 / 0 to review; audit_stubs 0
  Ir base for the three-pool gate: unchanged from (-250) — the `gang` path is untouched
suite   19,230 / 0 / 5; golden traces 7/7 (see the commit for any re-bless)
clippy  --workspace --exclude crabomination_client --all-targets   clean
release the release-fast typecheck gate: clean
--bench release-fast (mimalloc): 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf
ladder  round 60, dflt-open vs dflt58, four seeds: 50.0 / 50.0 / 50.0 / 49.9 (pooled 49.98) — adopted;
        cube 50.2 / 50.0, fixed 50.2 / 50.1 (seeds 43/97) as the attack-trigger cross-check
```

### `(-255)` and round 59 — closing state at the `(-255)` tip

One engine commit on top of the round-58 tip, behaviour-preserving
(outcomes identical, golden traces unmoved): the attack chain's pool
resolved ahead of the sims, an empty-greedy menu with nothing eligible
returned without one. Round 59's blocker gate measured and not adopted
(its flag stays as the `empty-gate` control).

```text
  sealed default (dflt) mirror, callgrind --games 6 --threads 1 --seed 1:  3,453,977,308 -> 3,379,132,442 Ir  (-2.167 %)
  paired wall clock, sealed 200 x 12, 7 reps:                                0.984 median / 0.982 mean
  Ir base for the three-pool gate: unchanged from (-250) — the `gang` path is untouched
suite   19,230 / 0 / 5; golden traces 7/7 unmoved
clippy  --workspace --exclude crabomination_client --all-targets   clean
release the release-fast typecheck gate: clean
--bench release-fast (mimalloc): 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok
census  CRAB_ATTACK_CENSUS on the default, sealed 100 x 12, seed 43: 16,060 searched (25,384 before),
        3.32 candidates/search, chain sims 3.41/search, from an empty greedy 1,876 (proposed 452, won 452)
ladder  round 59, empty-gate vs dflt, four seeds: 49.9 / 50.0 / 50.2 / 50.4 (pooled 50.12), wall 1.000 — not adopted
```

### Round 58 and `(-254)` — closing state at the round-58 tip

Two engine commits on the bot's chain path and one adoption: the wide
attack chain's pair move restricted (round 58, a search-budget change
gated for no loss), the bare block menu's sim skipped when the chain
cannot run (`(-254)`). The `--bench` profile (`gang`) carries neither
chain, so its counters are unmoved; the number that moved is the
default's own wall clock, which every actor pays.

```text
  sealed default mirror, this container (4 cores), 200 x 12 games, one binary, 5 paired reps:
    dflt56 (r56 default)  1.000      pairs-both (adopted)  0.851 median  [0.833 .. 0.888]
  Ir, sealed dflt56 --games 6 --threads 1 --seed 1 (mimalloc totals):  4,460,465,907 -> 4,451,753,506  (-0.195 %, (-254))
  Ir base for the three-pool gate: unchanged from (-250) — neither leg touches the `gang` path
  (cube 1,817,493,748 / fixed 656,319,384 / sealed 1,886,392,273 under `--a gang --b gang`)

rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores
suite   19,230 / 0 / 5 (97.6 s under nextest); golden traces 7/7 (seed 3's digest re-blessed:
        same winner / 32 turns / 737 actions, one attack declaration moved; the full trace unmoved)
clippy  --workspace --exclude crabomination_client --all-targets   clean
release the release-fast typecheck gate (debug-assertions off): clean, 1m24s
--bench release-fast (mimalloc): 195,806 decisions / 27.49 turns / 611.9 per game / 0 stalls —
        counters identical to 2003d1cf; determinism ok (all pairs split); peak_rss 30.5 MiB
        (mimalloc; the 18.9 MiB readings are system-allocator profiling-fast builds)
ladder  round 58, sealed, 12,000 games a cell, four seeds each: pairs-empty 50.08, pairs-lazy 50.00,
        pairs-both 50.10 pooled — every cell's interval touches 50, none below it
census  CRAB_ATTACK_CENSUS on the adopted default, sealed 100 x 12, seed 43: 25,384 searched,
        2.47 candidates/search, chain sims 2.16/search (3.20 before), 11,200 from an empty greedy;
        block chain runs on 98.8 % of 7,484 searches (bare menus with nothing to add no longer
        counted), start reused 100 % of runs
```

### The layout-flag refutation and the tracker compaction — closing state at the `(-253)` tip

No engine commit: `(-253)` was measured and not landed, and the run's
other product is `PERF.md` 21,520 -> 7,600 lines (the Log from `(-199)`
down and the Baseline closing states from the `(-185)` tip down moved
verbatim to `PERF_ARCHIVE.md`). The Ir base is unchanged from `(-250)`
and was re-taken on this container to the same three numbers.

```text
  pool     (-250) base      re-taken here    delta
  cube     1,817,493,748    1,817,493,538   -0.00001 %   (the once-per-process cost's jitter)
  fixed      656,319,384      656,319,786   +0.00006 %
  sealed   1,886,392,273    1,886,392,623   +0.00002 %

rustc   1.95.0 (59807616e 2026-04-14); Intel Xeon @ 2.80 GHz, 4 cores
suite   19,255 / 0 / 5 (97.8 s under nextest) before the data-test sweep, 19,220 after it
        (the five touched binaries re-run green); golden traces 7/7 in it
clippy  --workspace --exclude crabomination_client --all-targets   clean
release the release-fast typecheck gate (debug-assertions off): clean, 1m34s
--bench profiling-fast (system allocator): 195,806 decisions / 27.49 turns / 611.9 per game /
        0 stalls — counters identical to 2003d1cf; determinism ok (all pairs split);
        peak_rss 18.9 MiB; games_per_s 374-378 median over 32 paired runs (not a committed number)
sweep   fresh seeds on the profiling-fast binary: 8 primes 211..251 x {all, cube} x
        --games 120 --threads 2 = 16 cells / 24,000 games, 0 undecided, 0 panics, every rc 0
        (release-only: no overflow checks, no debug_assert — the (-248) audit-build sweep stands)
census  CRAB_ATTACK_CENSUS on cube --games 6 --seed 1: 538 searched, 3.85 candidates/search,
        greedy 55.0 % / none 32.0 % / holdback 13.0 %, 208 tied the winner — identical to the
        e725e5c2 reading, so the search menu has not moved in 60 legs
```

**The device: a profile-free layout flag is refuted in one build and
one bench.** `bench_ab.py` at 16 pairs costs a minute against a
ten-minute build, and cachegrind's I1 column on `cube` says the same
thing deterministically; a build flag that claims layout can be priced
in under fifteen minutes end to end, which is how `(-251)`..`(-253)`
closed the axis from three sides.

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

Entries `(-249)` and older are in `PERF_ARCHIVE.md`, verbatim.

### `(-279)` TAKEN — the event scratch outlives the state through a thread-local pool, and the nested upkeep pass hands its buffer back: sealed default Ir **-0.378 %** / cube **-0.272 %** / fixed **-0.332 %**, 120 / 120 traces identical

Off the block-sim context read (candidates, "THE BLOCK SIM READ BY
CONTEXT"): the largest allocation-shaped row inside `resolve_combat_into`
was its own `events.reserve(32)` — 10,462 slow-path `do_reserve_and_handle`
calls / 12.1 M Ir, **1,155 Ir each** on the system allocator (a 4 -> 32
event regrow is a large-bin realloc: the first push in `pass_priority`
sized the buffer at four, and the combat step's reserve moved it). The
buffer is `GameState::event_scratch`, per state, and **a clone starts
without one** (`Clone` writes `Vec::new()`): every bot simulation's first
combat re-grew from scratch, and `(-80)`'s recycling only ever helped the
state that lived long enough to see a second combat. Two changes, both
allocation-only:

* `EventScratch`, a newtype over the slot with a bounded thread-local pool
  (`EVENT_POOL`, 8 buffers, only those at 32+ capacity): a dropped state
  parks its grown buffer, and a fresh state's first `take` — the one place
  the pool is asked, so the `LocalKey::with` is once per state, not per
  pass — picks it up already sized. The pool is consulted only when the
  slot has no capacity; a state that recycled once never touches it again.
* The nested untap -> upkeep pass in `advance_step` (`stack.rs`, the
  `pass_priority()?` whose events are appended to the outer batch) took a
  second buffer from the slot's replacement and dropped it: one pooled
  buffer leaked to the allocator every turn start, which is why the pool
  read half-empty on the first A/B (5,081 of the 10,462 slow paths left,
  `advance_step'2` 4,970 first pushes). `recycle_events` on that inner
  buffer closed it.

```text
callgrind --a dflt --b dflt --games 6 --threads 1 --seed 1, profiling-fast, system allocator, UNTRACED, one tree at 6acd58c3:
  sealed  2,572,706,282 -> 2,562,977,750 Ir   (-9,728,532, -0.378 %)   72 / 72 decided both sides
  cube    2,515,278,324 -> 2,508,428,873 Ir   (-6,849,451, -0.272 %)   48 / 48
  fixed     673,786,345 ->   671,549,557 Ir   (-2,236,788, -0.332 %)   gang, the --bench pool
  CRAB_DUMP_TRACES both sides, sealed + cube: 72 + 48 trace files, 0 differ
rows (sealed): do_reserve_and_handle <- resolve_combat_into 10,462 / 12.08 M -> 2,923 / 2.68 M; grow_one <- advance_step 9,432 / 1.14 M -> 242 / 0.04 M,
               <- advance_step'2 4,970 / 0.60 M -> 752 / 0.09 M; LocalKey::with 22.07 M -> 22.07 M (+1.4 k: the pool's asks are ~1 per state);
               _int_free 88.99 -> 87.86 M, malloc 67.49 -> 66.81 M, _int_malloc 62.55 -> 59.52 M, free 55.48 -> 54.95 M
the pool alone (the first A/B, before the upkeep leak was found): sealed -0.272 % / cube -0.179 % / fixed -0.148 %
REFUTED arm: an empty pool handing out `Vec::with_capacity(32)` instead of `Vec::new()`: +0.035 % / +0.033 % / +0.053 % against the pool
             alone — most first pushes never reach a combat, so the wide start bought a large-bin allocation for nothing
```

What is left of the row: 2,923 slow paths (1,104 under the attack sim,
944 under `resolve_combat <- submit_decision`, which builds its own
`Vec`, 174 in the real game) — 0.1 %, thin, not a shape. The rule that
fell out: **a per-state scratch is a per-*clone* scratch; when the hot
path is a clone-per-probe, the capacity has to live somewhere the clone
does not, and the leak to look for first is the nested call that takes
the slot's replacement and drops it.** Under mimalloc (the shipped
allocator) a large-bin realloc is cheaper than glibc's, so the wall-clock
share is smaller than the Ir share; `--bench` counters unchanged (below).

### `(-278)` TAKEN — both chains read a candidate the menu already simulated instead of re-simulating it: sealed default Ir **-5.62 %** / cube **-4.92 %**, 120 / 120 traces identical

The chains grow from "nobody" and so re-derive the menu from below: on
a two-attacker greedy the attack chain's first-step singles *are* the
menu's two holdbacks and its second step is greedy; the block chain's
first step offers the menu's chumps and its gangs again. Only the
*start* plan's score was being reused (`(-255)`'s device, and 100 % of
runs hit it). A sim is deterministic per (start state, declaration), so
`attack_chain_candidate` / `block_chain_candidate` now key the menu's
scored sets once (the whole declaration — attacker and target — for the
attack side, since the menu's planeswalker retarget shares greedy's
attacker set) and every chain candidate asks that table before it is
simulated. Not a gate and not a search change: the argmax sees the same
scores in the same order, the chain's `served` candidates are counted
beside its `sims` in both censuses, and no flag is involved.

```text
callgrind --a dflt --b dflt --games 6 --threads 1 --seed 1, profiling-fast, system allocator, UNTRACED, base and candidate from one tree at the round-71 tip:
  sealed  2,725,852,967 -> 2,572,709,132 Ir   (-153,143,835, -5.62 %)   72 / 72 decided both sides
  cube    2,645,404,719 -> 2,515,279,341 Ir   (-130,125,378, -4.92 %)   48 / 48 decided both sides
  CRAB_DUMP_TRACES both sides, both pools: 72 + 48 trace files, 0 differ
census  CRAB_ATTACK_CENSUS=1 sealed dflt mirror 1,200 games seed 43 (the candidate binary):
  attack chain  sims 21,096 (1.44 a search) + served from the menu 10,288  = 32.8 % of its candidates never simulated
  block chain   sims 30,386 (4.15 a search) + served from the menu  6,026  = 16.5 %
golden 7/7 unmoved; --bench (gang, fixed) carries no chain and is untouched
```

**The untraced base above is the number to A/B against from here.** The
round-70 Log's 2,882,893,998 / 2,820,089,306 were traced runs
(`CRAB_DUMP_TRACES` on both sides): the trace writer is ~5.5 % of a
six-game run, and the a1794130 binary rebuilt untraced from this tree
reads 2,725,174,355 / 2,645,811,737 — the round-71 census commit is
+0.025 % / -0.015 % against it, the `menu_class` fold. **What is left of
the same shape:** a chain candidate that repeats across *steps* (none: a
step's sets are strict supersets of the last), and the attack chain's
lazy pair move (2-sets, never in the menu). The menu itself is 3.4
candidates a search and all distinct.

### Round 71 REFUTED — the block chain gated on the menu's outcome, three arms: paired wall clock **0.886 / 0.900 / 0.900** sealed, ladder **48.50 / 47.67 / 48.30** pooled, every cell wholly below 50, all parked

The round-70 device on the block side (`.ladder/run_r71_blockchain.sh`,
ML_NOTES round 71; the census split is in the candidates, "THE BLOCK
CHAIN SPLIT"). Three one-flag arms on the default, one `profiling-fast`
binary, the 200 x 12 x 5-rep paired clock and four sealed seeds x 12,000:

```text
  arm           flag                        sealed wall/dflt   cube wall/dflt   ladder cells (43 / 97 / 151 / 199)    pooled
  bchain-skipg  block_chain_skip_greedy     0.886 (mean 0.936)  0.921           48.5 / 48.5 / 48.4 / 48.6              48.50   LOSS
  bchain-empty  block_chain_empty_only      0.900 (0.891)       0.895           47.6 / 47.9 / 47.6 / 47.6              47.67   LOSS
  bchain-seed   block_chain_from_menu       0.900 (0.919)       1.000           48.3 / 48.4 / 48.3 / 48.2              48.30   LOSS
  every interval's high is below 50 (48.8 at best); 12,000 decided / 0 undecided a cell; 24-35 s a cell on 3 threads
```

What it says, so nobody re-gates it: **the block chain's wins are ladder
wins on every board class**, where the attack chain's greedy-board wins
were not (round 70). Skipping it on the greedy board alone costs 1.5
points, on the greedy and "other" boards 2.3, and even *seeding* it from
the menu's own winner — the chain still runs, it only stops reassigning a
blocker greedy already used — costs 1.7. The chain's value is the
reassignment, not the extension. The three flags stay as measured arms
(off, the round-59 / round-61 precedent); the ~10 % of the default's
sealed wall clock they would buy is the block chain's price for ~2
points, and the next cut at the block search is a cheaper *sim*, not a
cheaper *search*. Golden traces and `--bench` untouched (no flag on the
default). The census's `menu_class` fold and the seeded start are the
only code on the default path: ~10 Ir a block search, not read.

### Round 70 ADOPTED — the attack chain skipped when the menu alone picked a non-empty greedy (`attack_chain_skip_greedy`): sealed default wall clock **0.841** / cube **0.870** at no loss; six-game Ir sealed **-13.06 %** / cube **-10.32 %** (different games)

The census question the previous candidates entry filed, answered the
same run (ML_NOTES round 70; `.ladder/run_r70_chainskip.sh`): of the
2,616 sealed searches where the chain proposed a novel set, the menu alone
had picked greedy on 1,218 (the chain won 642 — 4.4 % of all 14,572
searches), nobody on 992 (won all 992), a holdback on 406 (won 238). The
gate keeps the chain on the nobody / holdback / empty-greedy boards and
drops it where the menu already settled on a non-empty greedy — roughly
half the chain's 3.58 sims a search. Pre-registered, gated for no loss on
four sealed seeds (pooled 50.08, cells 49.9 / 50.0 / 50.1 / 50.3, none
wholly below 50), cube 49.8 / 50.0, fixed 49.9 / 50.1; adopted in
`EvalWeights::default_const()`, control `chain-skipg-off`. The first cut
skipped the empty-greedy chain too (0.866 / 0.907, pooled 50.35) and broke
the round-56 board's unit test; the corrected arm is what was measured and
adopted.

```text
paired wall clock, 200 x 12 mirrors, 5 reps, median arm/dflt:   sealed 0.841 (0.841 / 0.860 / 0.828 / 0.865 / 0.835)   cube 0.870 (0.833 / 0.870 / 0.929 / 0.909 / 0.833)
callgrind --a dflt --b dflt --games 6 --threads 1 --seed 1, profiling-fast, system allocator (cg.r68.* / cg.cand.* in a scratchpad):
  sealed  3,315,951,121 -> 2,882,893,998 Ir   (-13.06 %, different games; 72 / 72 decided both sides)
  cube    3,144,754,672 -> 2,820,089,306 Ir   (-10.32 %, different games; 48 / 48 decided)
golden 7/7 unmoved (no seeded game reaches a search the gate changes); suite 19,285 / 0 / 5; clippy clean; --bench (gang, fixed) untouched
```

**The dflt six-game totals above are a different pilot's games** — quote
them as the base from here, never as an engine change against the
`(-277)` block. The device that found it is the census extension, not a
profile: a per-context split of *who wins* the sims the search buys, which
callgrind cannot see.

### The `dealer_filter` source hint and the next-attack delayed kind READ — traced sealed Ir **+0.009 %** / cube **+0.000 %**, traces identical

Two rules changes after `(-277)` (ENGINE_BACKLOG, first section): the
combat-damage hook hands a trigger's `dealer_filter` its listener as the
source (`Some(c.id)` at phases 1 and 1.6, so `IsSource` can be true —
Calix), and the dispatcher's attack-declared leg consumes
`DelayedKind::YourNextAttackThisTurn` for the attacking player (All-Out
Assault) — one `delayed_triggers.iter().any()` per attack batch on a list
that is empty on almost every board. Priced against the `(-277)` binary
under the same traced recipe (`cg.base277.*` / `cg.cand.*` in a scratchpad):

```text
profiling-fast, system allocator, --a dflt --b dflt --games 6 --threads 1 --seed 1, CRAB_DUMP_TRACES both sides
  sealed  3,315,651,668 -> 3,315,951,121 Ir   (+299,453, +0.009 %)   72 trace files, 0 differ
  cube    3,144,748,521 -> 3,144,754,672 Ir   (+6,151, +0.000 %)     48 trace files, 0 differ
```

Kept as rules changes; nothing to read by function at this size.

### `(-277)` TAKEN — every trigger hook reads instance grants through one walk shape: traced sealed Ir **-0.132 %** / cube **-0.127 %**, traces identical

The three combat listener walks (`ControllerAttackedByOpponent`, `YouAttack`,
`ControllerDealtCombatDamage`) were the last hooks outside the dispatcher
that read printed triggers only (ENGINE_BACKLOG, first section); they read
`granted_triggers_timed` now behind the same kind-filtered presence gate the
step and cast hooks use, `any_granted_trigger_of_kind`, and
`expire_granted_triggers` took the empty-map fast path the candidates list
filed. A rules change, priced under the traced recipe of the entries below
(`--a dflt --b dflt --games 6 --threads 1 --seed 1`, `profiling-fast`, system
allocator, `CRAB_DUMP_TRACES` both sides, the base at `4db20c51` rebuilt in
a second worktree — see the trap below), and it took four builds to land as
a win rather than a loss:

```text
                                                            sealed Ir            cube Ir              traces
base 4db20c51                                              3,320,021,324        3,148,739,249
1  closures as `&mut visit` into two walks (`if own {iter} else {triggerer}`)   +0.217 %             +0.159 %        72 / 48 identical
2  closures by value into the same two walks               +0.239 %             +0.187 %             identical
3  one walk, `for_each_triggerer_or_all(all, f)` (match arms, `#[inline]`)      +0.138 %             +0.100 %        identical
4  + the grant loop out of line (`for_each_granted_trigger_matching`)           +0.072 %             +0.074 %        identical
5  + one `f(c)` call site (the chunked bit loop)             3,315,651,668 (-0.132 %)  3,144,748,521 (-0.127 %)   identical
```

**What the five rows say is about the inliner, not the grants.** Every
row's residue was the per-permanent `visit` closure being *called* instead
of inlined — `FnMut for &mut F::call_mut` 626,234 -> 892,152 calls in row 1
(+11.8 M), the same closures as named out-of-line functions in rows 2-4
(`declare_attackers_banded::{closure}` 183,408 calls / 6.2 M,
`fire_step_triggers::{closure}` 206,750 / 11.5 M,
`fire_spell_cast_triggers::{closure}` 35,974 / 4.5 M). Two things kept them
out: a closure handed to two generic walks (row 1-2) is codegen'd once and
called from both; and a walk with two `f(c)` sites (the member-list loop
and the whole-board tail, which the old `for_each_triggerer` also had) is
two inlining decisions, and once the closure body carried the grant loop
LLVM took neither. Row 5 is one bit loop over 64-card chunks — "all" is a
full mask — so `f` has one call site, and every closure inlined:

```text
sealed self-Ir delta, row 5 against the base
   -16,401,806  FnMut for &mut F::call_mut      626,234 -> 383,576 calls   the step and cast hooks' `visit`, inlined for the first time (they were `&mut` on both sides before)
   +12,011,360  for_each_triggerer_or_all             0 -> 46,048          the three combat walks + the cast hook, bodies inlined
    +7,263,108  fire_step_triggers                    53,848 both sides    its walk inlined into the hook itself
    -5,718,368  for_each_triggerer                 35,280 -> 0             gone (a one-line wrapper now)
      +568,400  any_granted_trigger_of_kind            0 -> 28,420         20 Ir a read, `#[inline]`, still out of line at release-fast — the price of the gate
```

**The rule that fell out: a `visit` closure the hot walks take must have
ONE call site and a body under the inline budget.** The grant branch lives
in `GameState::for_each_granted_trigger_matching`, `#[inline(never)]`, so a
hook's closure stays the printed loop plus one call; and
`Battlefield::for_each_triggerer_or_all` is the only walk, taking the
"whole board" answer as a flag. The step and cast hooks moved onto it too,
which is where most of the -0.13 % comes from — their closures had been
`&mut visit` since `(-228)`/`(-231)` and out of line the whole time.

⚠ **AND THE FIRST BASE WAS THE TIP.** The worktree recipe the previous
entry describes gives a "base" build that finishes in 0.16 s and hands back
the tip's binary when the worktree's sources are older than the tip's
outputs (cargo keys a member by its path relative to the workspace root):
the first A/B read -501 Ir with identical md5s. "How to measure" carries the
`touch` + `md5sum` step now.

`core_rules::cr_recent49::cr_603_2_instance_granted_combat_listeners_fire`
pins the three walks (a grant on a permanent with no printed trigger fires
from each). Suite 19,282 / 0 / 5, clippy clean, golden 7/7 unmoved.

### The stateful keyword-grant gather's wrapper peel READ — traced sealed Ir **+0.027 %** / cube **+0.001 %**, traces identical

`gather_continuous_effects_inner`'s live-filter `SetBasePtForFilter` /
`GrantKeyword` leg peels the `WhileYourTurn`-family wrappers through
`active_static` like the eager leg beside it (`72297152`, ENGINE_BACKLOG
first section). Priced against the hook tip under the same traced recipe
as the entry below (`cg.tip3.*` in a scratchpad):

```text
profiling-fast, system allocator, --a dflt --b dflt --games 6 --threads 1 --seed 1, CRAB_DUMP_TRACES on both sides
  sealed  3,310,152,922 -> 3,311,060,214 Ir   (+907,292, +0.027 %)   72 trace files, 0 differ against the 901c66f4 base
  cube    3,169,363,945 -> 3,169,383,029 Ir   (+19,084, +0.001 %)    48 trace files, 0 differ against the 901c66f4 base
```

The residue is the allocator, not the peel: the sealed delta table is
`_int_malloc` +529,604 (265,403 -> 269,777 calls), `malloc_consolidate`
+323,614, `_int_free` +214,391, and `gather_continuous_effects_inner`
itself **-111,372** at an identical 536,963 calls — glibc's arena state
under a different layout, the same games. Kept as a correctness change.

### The trigger-grant duration carrier and the two hooks READ — sealed default Ir **+0.060 %** / cube **+0.061 %**, traces identical

The last arm of the duration axis (ENGINE_BACKLOG, first section):
`granted_triggers_eot` became `granted_triggers_timed` with an
`EffectDuration` per entry, `expire_granted_triggers` joined the five
sweeps that retire continuous effects of the same durations, the two
planeswalker damage branches record their damager, and the step and cast
hooks read the map behind a kind-filtered gate (`189bcaee` + `005a9b8c`).
Priced against `901c66f4`, first as the untraced tip alone and then with
the base rebuilt beside it in a second worktree (same `target/`, so the
deps were warm) and both sides dumped under `CRAB_DUMP_TRACES`
(`cg.base.*` / `cg.tip.*` in a scratchpad):

```text
profiling-fast, system allocator, --a dflt --b dflt --games 6 --threads 1 --seed 1
  sealed  3,023,237,395 -> 3,025,048,377 Ir   (+1,810,982, +0.060 %)   72 / 72 decided
  cube    2,966,378,057 -> 2,968,192,944 Ir   (+1,814,887, +0.061 %)   48 / 48 decided
traced, base and tip from one tree (the trace writer is ~+9.4 % Ir on both sides)
  sealed  3,308,028,286 -> 3,310,152,922 Ir   (+2,124,636, +0.064 %)   72 trace files, 0 differ
  cube    3,167,588,826 -> 3,169,363,945 Ir   (+1,775,119, +0.056 %)   48 trace files, 0 differ
```

**Identical games, action for action, and the residue is the price of
asking "is the map empty?" on paths that run every step.** The self-Ir
delta table (sealed; cube is the same shape):

```text
   +830,688  Vec::from_iter                 1,427,126 calls both sides — +0.58 Ir a call, the same collects at a different layout
   +550,018  fire_step_triggers               406,758 -> 406,822 calls — the `any_own_grant` gate, ~1.3 Ir a call on an empty map
   -533,450  dispatch_triggers_for_events   1,172,882 calls both sides — the `own_granted` slice read (a `GrantedTrigger` slice, not the old `&[TriggeredAbility]`)
   +500,554  FnMut::call_mut                  527,650 calls — the two hooks' per-permanent `visit`, one branch each
   +327,762  check_state_based_actions_into   706,035 -> 706,251 calls — the SBA sweep's `is_empty` read, ~0.5 Ir a call
   +244,320  GameState::granted_triggers      the iterator adaptor the four chain sites now go through, not inlined at release-fast
   +212,976  expire_end_of_combat_effects       5,118 calls, +41.6 Ir each — `expire_granted_triggers` builds `values().flatten().any()` before it reads the length
   +141,484  advance_step'2                   the turn-start and upkeep sweeps, the same 40-Ir shape
   -157,298  deal_combat_damage_to_target     the planeswalker branch's record (a push and a tally on an empty vec)
```

Kept as a rules change: the four sweeps that grew are ~40 Ir each on an
empty map because `expire_granted_triggers` asks `values().flatten().any()`
first and `is_empty()` never — a one-line fast path worth ~1 M Ir a
six-game run (0.03 %), filed in the candidates rather than built in this
pass (the third engine commit was already pricing under it).

### `GrantActivatedAbilityToMatching`'s permanent carrier READ — sealed default Ir **+0.000 %** / cube **+0.000 %**, outcomes identical

The arm that returned early on every duration but end of turn (Life
Matrix, ENGINE_BACKLOG first section) grants through the two
`GainActivatedAbility` lists now. Priced against the duration tip
(`09b9f056`, the `cg.cand2.*` dumps):

```text
profiling-fast, system allocator, --a dflt --b dflt --games 6 --threads 1 --seed 1
  sealed  3,023,234,030 -> 3,023,237,395 Ir   (+3,365, +0.0001 %)   72 / 72 decided, outcome lines identical
  cube    2,966,367,971 -> 2,966,378,057 Ir   (+10,086, +0.0003 %)  48 / 48 decided, outcome lines identical
```

**FLAT** — the arm runs only when a card resolves the effect, and none did
in either dump; the few thousand Ir are layout.

### The keyword-grant carriers and `UntilNextTurn` READ — sealed default Ir **+0.040 %** / cube **-0.011 %**, outcomes identical

The fourth rules fix off the consumer-read method (ENGINE_BACKLOG, first
section): `GrantKeyword` / `GrantKeywords` / `GrantProtectionFromChosenColor`
/ `LoseKeyword` route any duration past EOT / Permanent through a layer-6
continuous effect (`grant_keyword_for`, `keyword_layer_effect`), and
`effect_duration_for` maps `Duration::UntilNextTurn` to the controller's
next turn. Priced against `1f2cabcb` (the `cg.cand.*` dumps of the entry
below, this run's tip):

```text
profiling-fast, system allocator, --a dflt --b dflt --games 6 --threads 1 --seed 1
  sealed  3,022,028,711 -> 3,023,234,030 Ir   (+0.040 %)   72 / 72 decided, outcome lines identical
  cube    2,966,685,176 -> 2,966,367,971 Ir   (-0.011 %)   48 / 48 decided, outcome lines identical
```

**FLAT** — the arms now collect their selector's ids before granting
(one `Vec` per resolution, a handful a game), and no game in either dump
resolved a grant with one of the new durations. Kept as correctness:
thirteen shipped cards had a permanent grant, six more a debuff that wore
off before the opponent's combat.

### The LKI walk's EOT-grant read and the conditional rider gate READ — sealed default Ir **+0.012 %** / cube **+0.011 %**, outcomes identical

The third rules fix off the consumer-read method (ENGINE_BACKLOG, first
section): `dispatch_triggers_for_events`' died-snapshot walk chains
`granted_triggers(snap.id)` beside the printed list (Requiem Monolith's
lethal ping drew nothing), and `granted_abilities_of` evaluates a
`ConditionalEquipBonus`'s `condition` before granting its abilities
(latent, no shipped card). Priced against the tip the run started on
(`e52c6191`, whose base dumps re-read within 1.3 k / 0.7 k Ir of the
`78e57bd2` readings below):

```text
profiling-fast, system allocator, --a dflt --b dflt --games 6 --threads 1 --seed 1
  sealed  3,021,668,348 -> 3,022,028,711 Ir   (+0.012 %)   72 / 72 decided, outcome lines identical
  cube    2,966,365,259 -> 2,966,685,176 Ir   (+0.011 %)   48 / 48 decided, outcome lines identical
```

**FLAT**: one map probe per died snapshot per dispatch (the EOT map is
empty on nearly every board) and a `condition` read that no rider on
either pool carries. Kept as correctness.

### The hook `once_per_turn` / `dealer_filter` gates READ — an `Option<usize>` on the attack and combat-damage tuples: sealed default Ir **+0.033 %** / cube **+0.091 %** against the run's base, outcomes identical

The second rules fix of the day on the trigger hooks (ENGINE_BACKLOG,
first section): `declare_attackers_banded`'s trigger tuple and
`DamageTrigger` carry the CR 603.3d once-key, the two consumers insert
into `triggered_once_per_turn_used` after their filter, and Phase 1 of the
combat-damage hook reads `dealer_filter`. Priced against the same
`d04a225d` base dumps as the entry below (so this reading includes that
entry's flat -0.006 / +0.018 %):

```text
profiling-fast, system allocator, --a dflt --b dflt --games 6 --threads 1 --seed 1
  sealed  3,020,686,102 -> 3,021,669,628 Ir   (+0.033 %)   72 / 72 decided both sides
  cube    2,963,669,195 -> 2,966,364,550 Ir   (+0.091 %)   48 / 48 decided
--bench release-fast (mimalloc): 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok; bin_bytes 126,744,752
```

**FLAT** (the tuple grew 8 bytes on a per-damage-event push, plus an
`enumerate` and an `is_none_or` per Phase-1 trigger); the games are the
same on both pools because neither six-game run fields a once-per-turn
attacker or a `dealt_by` card. Kept as correctness — an Aurelia in
self-play was an unbounded combat loop before it.

### The equipment-grant dispatcher fix READ — flagged attachments join `equip_grants`: sealed default Ir **-0.006 %** / cube **+0.018 %**, outcomes identical

A rules fix, not a perf leg, filed here because it touches the trigger
dispatcher's grant list (ENGINE_BACKLOG, first section: `triggers_on_
equipment` was honoured by two hooks and dropped by the dispatcher). The
change widens `DispatchScan::equip_grants` to every attachment with a
granted trigger — the ten `sword()` Swords, Jitte, Godsend, Kusari-Gama
and Mask of Griselbrand were excluded before — and carries a per-grant
source through the three consumers. The cost to price was the wider
`any_equip_grant` gate (a flagged Sword on the board used to leave the
dispatcher on its member lane; now it walks the whole board on the
batches whose kinds its grants can match, which `(-121)`'s retain keeps
to the combat-damage batches the hook already owns).

```text
profiling-fast, system allocator, --a dflt --b dflt --games 6 --threads 1 --seed 1, base d04a225d in a detached worktree, candidate = the two commits' patch applied there
  sealed  3,020,686,102 -> 3,020,489,826 Ir   (-0.006 %)   72 / 72 decided, 0 undecided both sides
  cube    2,963,669,195 -> 2,964,205,035 Ir   (+0.018 %)   48 / 48 decided
  cube rows: dispatch_triggers_for_events 136,115,116 -> 136,129,146 (+0.01 %); dispatch_board_scan 12,047,280 -> 12,081,568; dispatch_scan_card 3,961,968 -> 4,087,828 (+3.2 % of a 0.13 % row: the flag test moved into the bit fold)
--bench release-fast (mimalloc): 195,806 / 27.49 / 611.9 / 0 stalls — counters identical to 2003d1cf; determinism ok; bin_bytes 126,742,096
```

**FLAT on both pools and the same games** — the fixed pool carries no
flagged attachment and the six-game cube/sealed runs field them rarely.
Kept as the correctness change it is. The base here (3,020.7 M / 2,963.7 M)
is within 0.2 % of the round-68 context map's totals (`62e38777`:
3,015.6 M / 2,957.2 M) and is the base to quote from this tip.

### `(-276)` TAKEN — the 8 KB `CardDefinition` temporaries leave `run_effect`'s frame: 97,256 -> 64,312 bytes, sealed default Ir **-0.256 %** / cube **-0.288 %**

```text
  binary pair   dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast -p crabomination --no-default-features), the 41ca3089 tip either side
  sealed        3,283,015,188 -> 3,274,597,848 Ir   **-0.256 %**   (first pass, the five by-value arms alone: -0.097 %)
  cube          3,639,696,059 -> 3,629,217,631 Ir   **-0.288 %**   (first pass -0.116 %)
  frame         run_effect 97,256 -> 72,568 -> 64,312 bytes (release-fast); the inline probe loop 23 -> 16 pages a call, 61,084 calls a cube run
  outcomes identical on every dump (72 / 48 decided, 0 undecided); suite 19,246 / 0 / 5 either pass
```

The build-time section's "`run_effect`'s frame" entry has the whole
read: the only frame in the binary past 4 KB, attributed slot by slot
from the `.dwo`. Eight 8 KB `CardDefinition` values were born in the
frame — five named (`back_face` clones, `CreateTokenCopyOf`, the
basic-land factory, Grist's insect and its literal, `lookup_by_name`)
and two unnamed (`Arc::make_mut`'s inlined clone path at twelve
definition-rewrite sites). Each now lives in an `#[inline(never)]`
one-liner (`clone_arc` / `boxed_clone` / `definition_make_mut` /
`basic_land_arc` / `lookup_arc_by_name` / `grist_insect_token`), whose
frame is 48 bytes because `Box::new(self.clone())` builds the clone in
the allocation. What is left is the 970-arm `match`'s own tail — 28
`PendingEffectState` slots LLVM did not colour together — and that is
the split, not a helper.

### Round 68 ADOPTED — the attack sim's main-phase casts capped at one per sim (`sim_main_cast_cap`): sealed default wall clock **0.864** / cube **0.837** at no loss; six-game Ir sealed **-7.95 %** / cube **-18.75 %** (different games)

```text
  binary pair    dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast -p crabomination --no-default-features), one tree either side (the 301f5edb tip)
  sealed         3,277,686,484 (uncapped) -> 3,017,209,691 Ir (cap 1, the new default)   -7.95 %   (72 decided either side, different games)
  cube           3,635,747,593            -> 2,953,897,394 Ir                            -18.75 %  (48 decided either side, different games)
  actor          3,120,856,905 -> 3,094,007,397 Ir (-0.86 %, different games: 6,078 -> 6,198 rows, 0 stalls both)   (selfplay_train --actors 1 --games 60 --steps 1 --seed 7, -p crabomination_ml --no-default-features)
                 per sim: 4,042 sims / 1,590.3 M -> 4,518 / 1,478.7 M under simulate_attack_outcome_from = 393 k -> 327 k Ir a sim (-17 %); the sim's main-phase casts 8,860 -> 4,635
  flag off       the same binary with the cap off reads sealed 3,279,647,929 / cube 3,637,573,887 against the base's 3,277,686,484 / 3,635,747,593: +0.060 % / +0.050 %, the counter and the window test per sim iteration
  arms (cand_bl) sealed cast0 2,411,806,036 / cast1 3,017,223,838 / cast2 3,202,641,219;  cube cast0 3,185,572,866 / cast1 2,953,897,330 / cast2 3,205,334,601
  the rows       accept_on <- sim_spell_action_inner (the sim's main-phase casts):  cube 11,030 calls / 419.7 M -> 4,928 / 175.6 M;  sealed 8,998 / 300.9 M -> 4,986 / 171.5 M
                 cast_candidates <- sim_spell_action_inner:  cube 17,816 / 179.7 M -> 6,284 / 67.7 M;  sealed 17,672 / 134.2 M -> 7,282 / 66.9 M
                 sims (simulate_attack_outcome_once):  cube 4,686 -> 4,588;  sealed 5,168 -> 5,456
  wall clock     200 x 12 mirrors, 5 paired reps, median arm/dflt:  sealed cast0 0.730 / cast1 0.864 / cast2 0.926;  cube 0.753 / 0.837 / 0.940
  ladder         sealed --a ARM --b dflt, 12,000 games a cell, seeds 43/97/151/199:
                 cast0  49.0 / 48.9 / 49.3 / 49.1  (pooled 49.08, every cell wholly below 50)  LOSS, parked
                 cast1  50.0 / 50.1 / 50.2 / 50.1  (pooled 50.10, every interval touching 50)  adopted;  cube 50.6 / 50.2 (3,200 each), fixed 50.4 / 50.1 (1,600 each)
                 cast2  50.0 / 50.0 / 50.3 / 50.1  (pooled 50.10)  no loss, not adopted (0.926 / 0.940 against cast1's 0.864 / 0.837)
  golden traces  seeds 3 (32 -> 19 turns, winner flips back to seat 0) and 4 (22 -> 20) and the committed c0ffee trace (372 -> 368 actions, same winner) re-blessed; seeds 1, 2, 5 unmoved
  --bench        gang / fixed, untouched by construction
```

A search-budget change, not a pure optimization, so it went through the
strength gate (`.ladder/run_r68_simcast.sh`, ML_NOTES round 68) and the
traces move with it. The `(-275)` context read filed the lever and
called it bot-side: the attack sim's spell layer casts from a main
phase, resolves, re-enumerates and casts again — 2.35 casts a sim on
cube, 1.74 on sealed — and each cast is a `cast_candidates` enumeration
(~10 k Ir) plus an `accept_on` resolution (~38 k) plus the passes it
induces. The cap keeps the first: the sim still sees the opponent's
best card and our post-combat play, and the ladder cannot tell the
boards the second and third produce from the ones they don't. Zero
casts loses a point (the crack-back creature and the main-phase removal
vanish from every hold-back's price). Tricks, removal and stack
responses are never capped. The flag-off residue (+0.05-0.06 %) is the
`main_window` test and the counter per sim iteration, inside the win.
Cumulative against the round-56 default: ~0.69 of its sealed wall clock.

### `(-275)` TAKEN — the two CR 602.5 ability-lock gates read an exact-keyword fold: sealed default Ir **-0.118 %** / cube **-0.172 %** / actor **-0.142 %**

```text
  binary pair   dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast -p crabomination --no-default-features), one tree either side (the 0876f5f3 tip)
  sealed        3,238,972,578 -> 3,235,138,032 Ir   **-0.118 %**
  cube          3,577,188,860 -> 3,571,024,045 Ir   **-0.172 %**
  actor         3,147,098,694 -> 3,142,639,712 Ir   **-0.142 %**   (selfplay_train --actors 1 --games 60 --steps 1 --seed 7, -p crabomination_ml --no-default-features)
  the row       card_keyword_possible_on <- activate_ability_inner  46,106 / 44,070 / 47,529 calls, 7.34 M / 9.22 M / 7.70 M incl (sealed / cube / actor)
                -> ability_lock_possible_on  48,654 / 48,964 / ~48 k calls, 4.89 M / 5.45 M incl;  callees now card_has_anthem ~10 k / 0.15 M and gather_scan_bits a few hundred
                base callees: Keyword::eq 138,318 / 1.52 M (the synth triple, three calls an ask, not inlined), can_grant_keyword 1,280 / 0.14 M sealed but 21,470 / 1.88 M cube (a static walk per grant member)
  by line       flat on the candidate (--dump-instr): 4.65 M over ~25 lines, the top line 10 Ir an ask, three atomic loads ~9.4 Ir an ask, the graveyard anthem lane ~8
  outcomes identical on every dump (72 / 48 decided, 0 undecided; 60 games / 6,074 rows / 0 stalls on the actor)
```

The land-tap keyword gates, priced at the `(-270)` tip and left as the
one lead with a device. Three folds already held the exact answer and
none was asked: the definition memo (a `gather_spec::ABILITY_LOCK_GRANT`
bit, `can_grant_keyword` at the two-keyword predicate, so the grant
member list is tested by word load instead of a static walk), the
continuous-effects family fold (a `mod_families::ABILITY_LOCK` subset
of `KEYWORD`, one load instead of the list walk), and the instance
legs by `has_kw` (a discriminant compare, no `Keyword::eq` call). The
off-board legs are `keyword_grant_in_scope`'s own, extracted rather
than copied, so the generic gate lost nothing and there is one walker
of the command zone / emblems / graveyard anthems, not two. Sound by
monotonicity (`can_grant_keyword(lock)` implies `ANY_GRANT`) and audited
in `gather_continuous_effects` beside the generic lane's assertion. The
residue is ~100 Ir an ask of branching over eight legs with no hot
line; not a lead.

### `(-274)` TAKEN — the dispatcher's pair loop reuses the per-event kind mask it folded for the batch gate: sealed default Ir **-0.202 %** / cube **+0.027 %**

```text
  binary pair   dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast -p crabomination --no-default-features), the (-273) build either side
  sealed        3,374,695,837 -> 3,367,871,911 Ir   **-0.202 %**
  cube          2,581,521,333 -> 2,582,210,012 Ir   **+0.027 %**
  sealed rows   event_matches_spec <- dispatcher  496,062 calls / 28.53 M incl (36.8 self + 12.1 event_kind_bits + the _rest calls)  ->  0;
                event_matches_spec_rest 60,148 / 4.48 M stays;  dispatcher self 190.46 M -> 207.68 M (+17.2 M: the payload match, now inlined, at ~35 Ir a pair-event against ~49 through the call)
  cube rows     the same shape at 169,968 pair-events: 11.26 M removed, self +8.38 M, rest 3.05 M, +0.53 M of event_matches_spec from the LKI/exile walks  ->  +0.70 M net
  first build   the masks collected into a SmallVec<[u128; 4]>: sealed +0.354 % / cube +0.580 % — SmallVec::extend 185,754 / 28.4 M (153 Ir a dispatch, try_grow on 32,858 of them: 18 % of batches run past four events)
  outcomes identical on every dump (72 / 48 decided, 0 undecided)
```

The base's second-largest call row by count was `event_kind_bits` at
1.09 M calls / 13.1 M: 594 k from the dispatcher's own batch fold
(`(-195)`), and 496 k from `event_matches_spec` re-deriving the same
mask per (pair, event) one frame down. The leg keeps the first eight
masks in a frame array beside the fold and hands each back through
`event_matches_spec_with_bits`, which also lets the payload match
inline into the pair loop — that inlining is most of sealed's win (the
call chain was ~14 Ir a pair-event) and none of cube's, where the
per-dispatch setup (~5 Ir over 245 k dispatches) is the residue. The
`SmallVec` form of the same idea was +0.35 / +0.58 % and is the
`(-229)`/`(-242)` rule again: a per-call `SmallVec` built by `collect`
costs more than a 12-Ir callee it replaces, and a push loop would have
spilled on 18 % of batches. Fixed array, events past it re-derive.
Taken on the sealed default (the actors' pool), as `(-261)` was at
sealed -0.485 / cube +0.037.

### `(-273)` TAKEN — `declare_attackers_banded`'s event buffer sized at two events an attacker: sealed default Ir **-0.097 %** / cube **-0.068 %**

```text
  binary pair   dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast -p crabomination --no-default-features), one tree either side (on top of the (-272) build, which is +0.6 M on both)
  sealed        3,377,968,053 -> 3,374,695,837 Ir   **-0.097 %**
  cube          2,583,275,496 -> 2,581,521,333 Ir   **-0.068 %**
  grow_one <- declare_attackers_banded (sealed)   28,378 calls / 7.06 M -> 12,830 / 2.87 M;  + try_allocate_in 14,266 / 1.20 M (the exact allocation)
  the --separate-callers=2 split of the base row: 23,212 first allocations (__rust_alloc, 2.23 M) + 5,166 re-growths (__rust_realloc, 3.00 M)
  outcomes identical on both dumps (72 / 48 decided, 0 undecided)
```

`cg_growth.py`'s top volume row at the `(-271)` tip (14,266 calls, 1.99
growths a call, 7.06 M) split by allocator entry — the instrument PERF's
"How to measure" prescribes before touching a growth row — as 23,212
mallocs and 5,166 reallocs: two buffers take a first push on every
declaration (`events` and the cloned state's capacity-0 `attacking`),
and `events` climbs 4 -> 8 on the 36 % of declarations with three or
more attackers (two events each: the tap and `AttackerDeclared`).
`Vec::with_capacity(2 * attacks.len())` is the exact size, so the
ladder is gone and the 1-2 attacker case pays the same one allocation
it did. `attacking`'s first push stays: an inline buffer there is bytes
on `GameState` (the `(-165)` rule), and a reserve only moves a first
allocation. Not the `(-227)` shape — that reserved 32 slots a frame up
for a buffer the callee then re-reserved past.

### `(-272)` REFUTED — `drain_trigger_queue`'s clickable/off-board split in place (`retain`) when the filter names no off-board zone: sealed **-0.018 %** / cube **-0.016 %**, reverted

```text
  binary pair   dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast -p crabomination --no-default-features), the (-271) tip either side
  sealed        3,378,571,112 -> 3,377,968,053 Ir   -0.018 %
  cube          2,583,686,536 -> 2,583,275,496 Ir   -0.016 %
  Iterator::partition <- drain_trigger_queue   2,880 calls / 3.01 M -> 1,906 / 2.43 M   (the other 974 prompts took `retain`)
  outcomes identical on both dumps
```

The growth census's `Iterator::partition` row (4,068 calls, 1.34 a
call, 1.16 M inclusive under `grow_one`) is `drain_trigger_queue`'s
split of a `wants_ui` trigger's ~18 legal targets into clickable and
off-board halves, the off-board half then dropped unless the filter
names a graveyard or exile. Two things the row did not say: **66 % of
those prompts DO name an off-board zone** (1,906 of 2,880 still take the
partition), and the row's 3.0 M inclusive is the per-candidate
`battlefield.iter().any(..)` predicate, which `retain` runs too — the two
grow ladders and the dealloc it removes are ~0.6 M on the third of
prompts it reaches. Below the branch's floor for a shipped leg
(`(-263)`'s -0.020 % was reverted on the same reading); reverted. The
row is a floor, not a lead: what would move it is the predicate (a
battlefield presence test per candidate is the `find_by_id` shape), and
at 3.0 M / 0.09 % that is not worth a lane.

### `(-271)` TAKEN — `damaged_by_this_turn` as an inline `CopyVec<[CardId; 4]>`: sealed default Ir **-0.430 %** / cube **-0.307 %**, actor **-0.382 %**

```text
  actor         selfplay_train --actors 1 --games 60 --steps 1 --seed 7 (profiling-fast -p crabomination_ml --no-default-features), against the (-270) tip
                3,172,892,750 -> 3,160,759,000 Ir  **-0.382 %**;  60 games / 6,080 rows / 0 stalls both sides
                allocator growths 438,603 -> 420,002 (-18,601; resolve_combat_into's 28,339 / 2.04 a call row is gone from cg_growth's table);
                _int_malloc 82.32 M -> 78.06 M, _int_free 108.66 M -> 106.36 M, malloc 84.11 M -> 82.32 M;  clone_from_ref_in +0.47 M, __memcpy +0.53 M (the inline copies)
  binary pair   dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast -p crabomination --no-default-features), one tree either side of the field change
  sealed        3,393,159,334 -> 3,378,573,605 Ir   **-0.430 %**
  cube          2,591,645,196 -> 2,583,687,288 Ir   **-0.307 %**
  outcomes identical on every dump (72 / 48 decided, 0 undecided)
```

`cg_growth.py` on the actor dump ranked `resolve_combat_into` at 2.04
re-growths a call over 13,924 calls (19.2 M inclusive) — the row the
candidates list had asked to census (`(-227)` had refuted a `reserve`
on the events buffer one function up). The source says which `Vec`:
every creature a combat damages takes `damaged_by_this_turn.push(..)`,
an `OftenEmpty<CardId>` whose first push is a heap growth, ~2 a damage
step. `CopyVec<[CardId; 4]>` is the same 24 bytes as the `Vec` (the
`(-161)` rule: an inline buffer is free when it does not grow its owner),
spills past four like any `SmallVec`, and clears through the same
`empty` macro arms `damage_by_source_this_turn` already uses. The three
`OftenEmpty` siblings stay: they are per-game lists that are rarely
pushed at all.

### `(-270)` TAKEN — `encode_state_pair`: a recorder snapshot's two encodes share the two untapped-source tables: actor **-0.265 %**

```text
  actor  selfplay_train --actors 1 --games 60 --steps 1 --seed 7 (profiling-fast -p crabomination_ml --no-default-features), against the (-269) tip
         3,181,309,243 -> 3,172,892,750 Ir  **-0.265 %**;  60 games / 6,080 rows / 0 stalls / 6,566 encoded states both sides
  mana_source_table self 16.65 M -> 13.93 M (the encoder's four builds a snapshot are two);  the castability scope under encode_state_inner (15.68 M) is the pair's now
  bot_ladder pools: unmoved by construction (encode.rs + the recorder)
```

Each seat's castability block reads *both* seats' untapped sources
(its own for the hand's live/dead split, the opponent's for globals
30..=35), so `encode_state(g, 0)` then `encode_state(g, 1)` built the
same two `mana_source_table`s twice. `untapped_sources` builds them
once per call of the new `encode_state_pair`, which the recorder now
uses; `encode_state` keeps its one-seat contract for the tests and any
other single-seat reader. Same tables, same features.

### `(-269)` TAKEN — the encoder's printed half off a per-object memo word (mana value, seven type bits, five pip counts): actor **-0.465 %**

```text
  actor  selfplay_train --actors 1 --games 60 --steps 1 --seed 7 (profiling-fast -p crabomination_ml --no-default-features), against the (-268) tip
         3,196,166,585 -> 3,181,309,243 Ir  **-0.465 %**;  60 games / 6,080 rows / 0 stalls / 6,566 encoded states both sides
  encode_printed_into  236,471 objects:  self 30.04 M -> 21.95 M;  callees (cmc + is_equipment + index_of) 7.46 M -> 0.18 M;  pack_printed ran 4,030 times (the misses: 1.7 %)
  the fifth CardMemo word (+8 bytes on CardData):  Arc::clone_from_ref_in 90.32 M -> 90.54 M (+0.22 M, the unshare copies);  __memcpy unmoved
  bot_ladder pools: not dumped — the word is read by the encoder only; the CardData growth is the +0.22 M above, ~0.007 %
```

The printed half of an object — `cmc()`, five `card_types.contains`
walks, the pip walk and two subtype walks — is a pure function of the
definition, and the object memo already showed the shape hits for
encoded objects (`vocab_index`: 941 `index_of` calls for 236 k
objects). A fifth `CardMemo` word packs it (`penc`: six bits of mana
value, seven type/subtype bits, five three-bit pip counts; bit 63 the
memo's) through `CardData::printed_encoding`, the `mana_summary`
accessor's shape with the same stale-memo `debug_assert!`.
`encode::tests::the_printed_encoding_word_matches_the_walked_features_
for_every_card` packs every catalog card against the walked reads, which
is also what pins the two saturating counts. Not serde-visible.

### `(-268)` TAKEN — one frozen scope per recorder snapshot, so the two encodes and the two material evals share one set of layer views: actor **-1.007 %**

```text
  actor  selfplay_train --actors 1 --games 60 --steps 1 --seed 7 (profiling-fast -p crabomination_ml --no-default-features), against the (-267) tip
         3,228,688,149 -> 3,196,166,585 Ir  **-1.007 %**;  60 games / 6,080 rows / 0 stalls / 6,566 encoded states both sides
  with_frozen_layers <- play_recorded_game_mcts  13,132 scopes / 3.3 M  ->  3,283 / 194.1 M (the snapshot block, inclusive);  encode_state_inner's own scope unchanged
  computed_permanent_hinted <- encode_state_inner  75,948 asks / 43.84 M -> 26.11 M (seat 1's encode now hits seat 0's views)
  program-wide:  compute_permanent_pass 93.44 M -> 80.97 M;  gather_continuous_effects_inner 98.38 M -> 91.34 M;  computed_permanent_hinted self 60.13 M -> 57.72 M
  bot_ladder pools: unmoved by construction (selfplay.rs' recorder is the actor path)
```

The recorder snapshots a position as `[encode_state(g, 0), encode_state(g, 1)]`
followed by `eval_material_public` per seat — four readers of one
position, each opening its own `with_frozen_layers` scope and so each
gathering the continuous effects and rebuilding every permanent's
`ComputedPermanent` from scratch. Nested scopes reuse the outer
gather and view memo (the design `encode_state` itself relies on for
its castability block), so one scope around the snapshot block turns
the second encode and both evals into memo hits. Read-only inside the
scope, the same features and the same material numbers; the bot's
`next_action` stays outside it. Found by asking who calls
`encode_state` (one caller, 6,566 calls, 3,283 pairs).

### `(-267)` TAKEN — a battlefield object skips the printed type flags and the instance keyword pass the computed view overwrites: actor **-0.229 %**

```text
  actor  selfplay_train --actors 1 --games 60 --steps 1 --seed 7 (profiling-fast -p crabomination_ml --no-default-features), against the (-266) tip + b3fd2c43
         3,236,108,381 -> 3,228,688,149 Ir  **-0.229 %**;  60 games / 6,080 rows / 0 stalls / 6,566 encoded states both sides
  encode_card_object_into 236,471 calls / 53.96 M self  ->  encode_printed_into 236,471 / 30.04 M self + encode_instance_keywords_into 160,523 / 15.94 M self
         (the 75,948 battlefield objects no longer run the keyword pass: ward 236,471 -> 160,523 calls, 3.65 M -> 2.57 M);  encode_state_inner self 56.5 M -> 58.2 M (inlining moved)
  bot_ladder pools: unmoved by construction (encode.rs only)
```

`encode_battlefield_object_into` ran the whole printed pass — five
type walks, `object_keyword_bits` over the instance lists, `ward()` —
and then, for every real battlefield object, overwrote all of it from
`ComputedPermanent` (types, keywords, the ward flag). The pass is now
two halves: `encode_printed_into` (cost, P/T, pips, attachment,
multiplicity, the index, and the type flags only when asked) and
`encode_instance_keywords_into` (the off-battlefield keyword answer);
a battlefield object asks for the computed view first and runs the
type flags and the keyword half only on the raw fallback that a real
walk never takes. Same final features, one write instead of two. The
encoder's residual after `(-266)`/`(-267)`: the printed half 30 M
(0.93 %, ~127 Ir an object: `cmc` 22, the pip walk, two subtype walks,
`is_creature`/`is_land`/... for the 160 k off-board objects), the
off-board keyword half 16 M (0.49 %), the layer views 43.8 M (1.36 %),
self 58 M (1.8 %), the castability scope 15.7 M.

### `(-266)` TAKEN — the encoder's board totals summed inside the object pass instead of a second `computed_permanent_on` walk: actor **-0.317 %**

```text
  actor  selfplay_train --actors 1 --games 60 --steps 1 --seed 7 (profiling-fast -p crabomination_ml --no-default-features), (-265) tip
         3,246,053,464 -> 3,235,755,573 Ir  **-0.317 %**;  60 games / 6,080 rows / 0 stalls / 6,566 encoded states both sides
  under encode_state_inner:  computed_permanent_hinted 151,896 asks / 51.98 M -> 75,948 / 43.81 M  (the second ask per permanent was the memo hit, ~110 Ir;
                             the first, ~577, is the layer view the scope has to build);  encode_state_inner self 58.73 M -> 56.52 M
  bot_ladder pools: not dumped — no pool of the ladder encodes a state (encode.rs is the only file touched), so sealed/cube are unmoved by construction
  encoder after this leg (the residual, per the same dump):  encode_card_object_into 236,471 objects / 65.1 M (2.0 %, the printed pass — cmc 5.3 M,
         is_equipment 2.1 M, ward 3.7 M, the rest is inlined type walks and keyword bits; for the 75,948 battlefield objects its keyword and type
         features are OVERWRITTEN by the computed view);  the layer views 43.8 M (1.35 %);  self 56.5 M (1.75 %, flat by line — top line 4.5 M is the
         eight-bit keyword loop);  the castability scope with_frozen_layers 15.7 M (0.5 %, two mana_source_table builds);  the library sort 5.5 M;
         affordable_covered 36,564 / 5.1 M
```

`encode_state_inner` walked the battlefield twice: once per controller
group to encode each permanent off `computed_permanent_on`, and once
more for the globals' land / untapped / creature / power totals, asking
`computed_permanent_on` again per permanent. `encode_battlefield_object_into`
now returns `(is_land, is_creature, power)` off the view it already
holds and the group loop sums them per side — the same computed facts
(an animated manland counts as the creature it is, an anthem reaches
the power total), the same integers, summed in group order instead of
battlefield order. Regression test
`encode::tests::board_totals_follow_the_object_pass`. Actor-only, like
every encoder row.

### `spell_kind` READ and REFUTED — the converge flag off the object memo plus an inline `creature_types`: actor **-0.105 %**, reverted

```text
  actor  selfplay_train --actors 1 --games 60 --steps 1 --seed 7 (profiling-fast -p crabomination_ml --no-default-features), (-265) tip
         3,246,053,464 -> 3,242,646,797 Ir  (-0.105 %);  60 games / 6,080 rows / 0 stalls both sides
  base   debug_flags 43,406 calls, 12.9 M inclusive: format_inner 148 calls / 8.56 M + is_contained_in 736 / 2.43 M (the once-per-name scan)
         + hash_one 384 (the L1 misses: 0.9 %); spell_kind's edge carries 11.3 M of it because a card is cast before it damages anything
  cand   Definition::debug_flags 14,881 asks -> CardDefinition::debug_flags 14,770 (the memo missed 99 %); SmallVec::extend 8,226 / 0.59 M for Vec::clone's 8,226 allocs
```

The candidates entry priced `spell_kind`'s `debug_flags` edge (8.6 M
on the sealed default) as a cache lookup worth 0.25 %. It is the
first-touch `format!` of 148 definitions — the saturating cost "How to
measure" warns about — and the L1 hits 99.1 % of asks. The memo device
cannot help either: `cast_spell_with_convoke` moves the card (a
`DerefMut`, which clears every memo word) and asks `spell_kind` on the
freshly written object, so the per-object memo is cold on exactly this
ask. What is left is the `Vec<CreatureType>` clone per creature cast,
~0.05 %. Reverted; the read closes candidates (3).

### `(-265)` TAKEN — `prepare_spell` shared as an `Arc<CardDefinition>` instead of deep-cloned per Prepare cast: actor **-1.803 %**, sealed default **-0.538 %** / cube **-0.496 %**

```text
  actor             selfplay_train --actors 1 --games 60 --steps 1 --seed 7 (profiling-fast -p crabomination_ml --no-default-features), against the (-264) tip
                    3,297,447,030 -> 3,238,000,614 Ir   **-1.803 %**;  60 games / 6,080 rows / 0 stalls on both sides
  binary pair       dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast -p crabomination --no-default-features), against the (-264) tip
  sealed            3,403,417,590 -> 3,385,114,447 Ir   **-0.538 %**
  cube              2,597,119,771 -> 2,584,225,619 Ir   **-0.496 %**
  outcomes identical on every dump
  under cast_prepare_spell (actor, 3,597 casts):  CardDefinition::clone 13.98 M -> 0;  CardInstance::new 32.56 M -> 1.62 M inclusive;
                                                  settle_prepare_after_cast 9.47 M -> 5.70 M (the unmaterialized copy's drop);  __memcpy program-wide 141.2 M -> 107.2 M
```

Found on the actor re-read at the `(-264)` tip (Profile of record): the
SOS Prepare cast — 3,597 a 60-game run on the actor's sealed-cube pool,
764 a six-game ladder run — materialized its copy by `clone()`ing the
creature's inset `prepare_spell: Box<CardDefinition>` and handing the
owned definition to `CardInstance::new`, which `Arc::new`s it: a deep
clone of a definition's two dozen `Vec`s plus two moves of the
several-KB struct per cast, 46.5 M Ir of the actor's 3,297 M before the
cast itself ran. The field is an `Arc` now (serde's `rc` feature; the
wire form is the `Box`'s), the cast is one refcount, and the copy's
`CardInstance` shares the creature's inset definition — which is also
what every other `CardInstance` already does with its definition. Two
catalog factories build the field. **An actor-only row: no `bot_ladder`
pool prepares often enough to rank it, which is the reason the actor
re-read is on the recipe list.**

### `(-264)` TAKEN — printed arms for the zone and player requirements a battlefield permanent answers by construction: sealed default Ir **-0.201 %** / cube **-0.040 %**

```text
  binary pair       dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast, system allocator), against the (-262) tip
  sealed            3,410,280,060 -> 3,403,417,590 Ir   **-0.201 %**
  cube              2,598,163,044 -> 2,597,119,771 Ir   **-0.040 %**
  outcomes identical on both pools
  under legal_targets_for_filter_scope (sealed):  evaluate_requirement_static_hinted 60,478 / 16.4 M -> 22,886 / 11.0 M;  printed_requirement_impl 61,690 calls both sides
  the census that found it (CRAB_REQ_CENSUS, a throwaway eprintln in the worktree, sealed --games 6): 3,276 enumerations, 61,690 battlefield candidates, 37,616 printed declines —
        1,754 x InGraveyard (36,556 declines, every permanent), 90 x OpponentPlayer (1,036), 6 x And(Creature, PowerAtLeast(3)) (24); the And/Or creature-and-controller shapes declined 0
```

`(-263)`'s follow-up, by the census it asked for. The printed evaluator
had arms for `InYourGraveyard` / `InOpponentGraveyard` (each a graveyard
scan by id, on the battlefield too) and none for bare `InGraveyard`,
`InExile` or the three player-only requirements — and bare `InGraveyard`
is the slot-0 filter on 54 % of the targeted triggers a `wants_ui` seat
is offered on the sealed pool, so the enumerator fell through to the
walker for every permanent on the board. An id names one object: the
battlefield permanent the `OFF = false` path holds is in no graveyard
and no exile, and no card is a player, so those arms are constants on
the battlefield and the walker's own scans off it. The suite's
`debug_assert_eq` in `requirement_on_permanent` audits every printed
answer against the walker. `cube` carries few such triggers on the
six-game board.

### `(-263)` REFUTED — the trigger-prompt enumerator's graveyard candidates through `requirement_on_graveyard_card`: sealed **-0.020 %** / cube **-0.014 %**, reverted

```text
  binary pair       dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast, system allocator), against the (-262) tip
  sealed            3,410,280,060 -> 3,409,599,592 Ir   -0.020 %
  cube              2,598,163,044 -> 2,597,792,272 Ir   -0.014 %
  under legal_targets_for_filter_scope (sealed):  printed_requirement_impl 61,690 -> 72,970 calls;  evaluate_requirement_static_hinted 60,478 / 16.4 M -> 57,728 / 14.5 M
```

The re-read's row (4) guessed the enumerator's ~1:1 printed-to-walker
ratio was its graveyard loop asking the walker by id. It was not: only
11,280 of the 60,478 walker calls were graveyard cards, and the printed
evaluator answered 2,750 of those. The rest are **battlefield**
permanents whose filter `printed_requirement` declines, so
`requirement_on_permanent` falls through to the walker on nearly every
permanent at this one site while the auto-target twin is answered 88 %
printed. Which requirement shapes the `wants_ui` trigger prompts carry
that the printed arms do not was NOT read — a `--separate-callers`
context on `printed_requirement_impl`'s `None` returns, or a census by
`SelectionRequirement` discriminant at this call site, is the next
step; the lead, if it is one, is `printed_requirement`'s coverage
(a `(-183)` follow-up with the suite's `debug_assert_eq` as the
ratchet), not the zone the candidate sits in. Candidates row (4)
corrected.

### `(-262)` TAKEN — a life-static lane in front of `adjust_life`'s seven board walks: sealed default Ir **-0.303 %** / cube **-0.243 %**

```text
  binary pair       dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast, system allocator), against the (-261) tip
  sealed            3,420,656,488 -> 3,410,280,060 Ir   **-0.303 %**
  cube              2,604,494,712 -> 2,598,163,044 Ir   **-0.243 %**
  outcomes identical on both pools
  self by row (sealed):  adjust_life 14,630,125 -> 3,596,035 (28,992 calls, ~505 -> ~124 Ir);  player_cannot_gain_life_now 868,610 -> 132,060
                         the lane's own cost: lanes_after_push +358 k, lanes_after_removal +185 k, walk_and_store +214 k
  self by row (cube):    adjust_life 9,175,883 -> 2,064,413;  lanes_after_push +300 k, walk_and_store +363 k, lanes_after_removal +132 k
```

`adjust_life`'s self was seven inlined battlefield walks — the
gain-to-loss and gain-to-draw replacements, the gain bonus and
multiplier, the cannot-gain lock (on a gain), the cannot-lose lock and
the loss doubler (on a loss) — each matching one `StaticEffect` across
every permanent's `static_abilities`, on every life change of every
simulation clone, for statics that thirteen cards in the catalog carry
between them. `LANE_LIFE_STATIC` is the union of the seven (wrappers
peeled, as `(-249)`'s untap lane does), read by each helper in front of
its walk; the other callers of those helpers (`Effect::LoseLife`, the
drain gates) get the same read for free. A `debug_assert!` in
`adjust_life` recomputes the seven through `active_static` on every
call, so the whole suite audits the predicate. `(-233)`'s rule again:
one lane whose predicate is the union, not seven lanes.

### `(-261)` TAKEN — `fire_spell_cast_triggers`' two delayed-trigger partitions behind a read-only match: sealed default Ir **-0.485 %** / cube **+0.037 %**

```text
  binary pair       dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast, system allocator), against the (-260) tip
  sealed            3,437,314,145 -> 3,420,656,488 Ir   **-0.485 %**
  cube              2,603,519,085 -> 2,604,494,712 Ir   **+0.037 %**
  outcomes identical on both pools
  under finalize_cast (sealed):  Iterator::partition 8,324 calls / 12,783,516 Ir -> 0 calls;  __rust_alloc 25,578 -> 21,416 (the per-cast `cast_name` String)
  the `visit` closure's FnMut shim:  sealed 18.23 M -> 18.44 M, cube 11.98 M -> 12.96 M — the inlining of the changed body retaken, and the whole of cube's delta
```

`(-257)`'s shape on the per-cast walk. `fire_spell_cast_triggers` guarded
its two CR 603.7e watcher blocks on `delayed_triggers` being non-empty,
then `mem::take` + `partition`ed the list into two fresh `Vec`s and
reassigned it — twice a cast, plus a `find_card_anywhere` and a
`name.to_string()` for the name-gated block — and on the sealed pool the
list is non-empty on 4,162 of 14,486 casts while holding a watcher *for
the cast* on almost none (Codie, Medomai's Prophecy III, Rediscover the
Way III are the whole population). Each block now asks its match over a
shared borrow first and enters the rebuild only when something fires; a
partition that keeps nothing left the list exactly as it was, so the
gate moves no order and no outcome. The name compare is `as_deref()`
against a `&'static str` instead of a cloned `Option<String>` per entry.
`cube` carries few delayed triggers on the six-game board (300
partitions), and its +0.037 % is one closure shim inside `finalize_cast`
reading 1 M more at the same call count — the standing rule about a
branch retaking an inlining decision, not a cost of the gate.

### Round 63's cost READ — `removal_sim` on the default is **~+0.4 %** of sealed `dflt` Ir (the six-game totals move ±1.4 % as different games)

```text
  tip e7ffebe9 (rounds 62-64 rebased onto (-260)), profiling-fast, system allocator, --games 6 --threads 1 --seed 1
  sealed  dflt-open (default minus removal_sim) 3,486,761,065   dflt 3,437,316,166   -1.418 %  <- different games, not a cost
  cube    dflt-open                              2,573,695,278   dflt 2,603,518,781   +1.159 %  <- same
  the picker itself, sealed dflt dump:  pick_defensive_removal_any 19,302 windows;
      combat_instant_candidates 1,218 calls / 7.8 M (6.4 k each: a cast_candidates enumeration plus a `format!("{a:?}")` dedup key per candidate),
      sim_start_state 252 / 3.3 M, the candidates' dry runs 208 / 2.9 M, run_combat_window 56 / 3.1 M  = ~18 M inclusive (0.52 %)
      against the rule picker's 4.1 M on the dflt-open dump  ->  net ~+0.4 %
```

The ML session adopted the flag with its wall clock unread. A flag that
changes decisions changes the six games, so the whole-run Ir compare
cannot price it (the two pools disagree in sign); the picker's own
inclusive rows can, and they say the sims are cheap because the
enumerator rarely yields (26 casts in 600 games). What the read leaves
as a small lead: `combat_instant_candidates` runs a full
`cast_candidates` sweep and formats a `Debug` string per candidate at
every pre-block window where the seat holds an instant — 0.22 % for a
picker that acts once in 23 games. Not taken; a paired wall clock at
200 x 12 would resolve nothing at this size.

### `(-260)` TAKEN — the SBA legend-rule leg behind a printed-legendary count of two: sealed default Ir **-0.246 %** / cube **-0.111 %**

```text
  binary pair       dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast, system allocator), against the (-259) tip
  sealed            3,491,854,975 -> 3,483,259,284 Ir   **-0.246 %**
  cube              2,574,857,158 -> 2,572,001,381 Ir   **-0.111 %**
  outcomes identical
  under the sweep (sealed):  SmallVec::extend 40,440 calls / 18.2 M -> 15,990 / 8.4 M (the legend leg's by_id collect)
                             sba_board_scan 71.1 M -> 76.6 M (+5.5 M: one AND + add per card per sweep for the count)
```

The leg ran whenever one printed legendary was on the board — two
thirds of all sweeps on the sealed pool — and walked the whole
battlefield through the `is_legendary` filter to build groups of one.
A group needs two members, so `sba_board_scan` now counts printed
legendaries and the leg runs at two or more (a live supertype grant
keeps the full walk, since it can make any permanent legendary). The
`legendary: bool` flag it replaces had no other reader. Net 8.6 M for
14 M of collect removed: the count is paid on every sweep, the walk only
on the ones that had a legendary.

### `(-259)` TAKEN — `CardInstance::toughness` / `power` as one pass over the counter bag: sealed default Ir **-0.548 %** / cube **-0.337 %**

```text
  binary pair       dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast, system allocator), against the (-258) tip
  sealed            3,511,084,325 -> 3,491,854,975 Ir   **-0.548 %**
  cube              2,583,556,668 -> 2,574,857,158 Ir   **-0.337 %**
  outcomes identical
  self by row (sealed):  toughness 23,189,098 -> 7,822,568 (238,188 calls, ~97 -> ~33 Ir)
                         power      6,375,268 -> 2,511,916 (88,816 calls)
```

`toughness()` was seven `counter_count` scans of the bag and `power()`
six, ~100 Ir a call on a bag that is empty or one entry long; the SBA
death filter alone asks toughness 159 k times a run (the damaged or
countered permanents `card_death_possible` cannot answer from fields).
`CounterBag` holds one entry per kind (`add` / `insert` find-or-push),
so one pass summing each entry's P/T contribution reads the same
number. Found off the self table: a `crabomination_base` row at 0.66 %
that no map had named because every caller is a legitimate ask.

### `(-258)` TAKEN — `fire_delayed_event_watchers` returns on an empty watcher list, its two ungated collects ask the batch first: sealed default Ir **-0.114 %** / cube **-0.010 %**

```text
  binary pair       dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast, system allocator), against the (-257) tip
  sealed            3,515,088,115 -> 3,511,084,325 Ir   **-0.114 %**
  cube              2,583,811,909 -> 2,583,556,668 Ir   **-0.010 %**
  outcomes identical
  fire_delayed_event_watchers, 56,270 calls: 20.3 M -> 16.3 M inclusive; Vec::from_iter under it 113,768 -> 32,306 calls
```

Every leg of the function fires a watcher off `delayed_triggers`, so
an empty list is an early return; and the `entered_creatures` /
`died_creatures` collects ran on every dispatch where the death and
attack legs beside them already asked the batch first. Small, and the
remaining 32 k collects are the batches that do carry an ETB or a
death — the floor for this shape.

### `(-257)` TAKEN — `fire_step_triggers`' delayed-trigger rebuild behind a read-only match: sealed default Ir **-0.903 %** / cube **-0.085 %**

```text
  binary pair       dflt mirror, --games 6 --threads 1 --seed 1 (profiling-fast, system allocator)
  sealed            3,547,131,260 -> 3,515,088,115 Ir   **-0.903 %**
  cube              2,585,999,842 -> 2,583,811,909 Ir   **-0.085 %**
  outcomes identical (stdout differs only in the wall-clock line); golden traces 7/7 unmoved
  before, under fire_step_triggers (53,996 calls, 132.8 M inclusive, 3.74 %):
          __memcpy 228,198 calls / 19.6 M, grow_one 24,962 / 14.4 M, __rust_dealloc 19,602 / 3.9 M
  after:  __memcpy  42,198 calls /  3.1 M, grow_one  6,484 /  3.4 M, __rust_dealloc  5,506 / 1.1 M
```

Every step of every turn `std::mem::take`s `delayed_triggers` and
rebuilds it — a `DelayedTrigger` carries an `Effect` (448 bytes), so
each live entry is two ~500-byte memcpys plus the fresh `keep` Vec's
allocation, and the list is non-empty on about a third of the steps
(the `keep` grows above) with nothing on it matching the step. The
match closure now runs once over a shared borrow first, and the
take-and-rebuild is entered only when something fires. `cube` carries
few delayed triggers on the six-game board, hence the small reading
there; `fixed` (the `--bench` pool) runs `gang`, which takes the same
path. Found off `--callers __memcpy_avx_unaligned_erms` on the first
context map taken under `dflt` — the second-largest memcpy caller in the
program was a function that clones 2,966 effects a run.

### Round 61 REFUTED — `attack_search` capped at 3 (`dflt-as3`): paired wall clock **0.991 median / 1.000 mean**, ladder 49.98 pooled, not adopted

Holdback wins by menu index (sealed, 2,400 games): #1 2,060 / 14,470
offered, #2 1,262 / 14,470, #3 282 / 7,384, #4 48 / 3,402, #5 18 /
1,540, #6+ 10 / 698. The cap drops 5,640 of ~196,000 attack sims and 76
argmax wins in 28,940 searches; the wall clock cannot see it, the
cells read 49.9 / 50.0 / 50.0 / 50.0. The menu's cost is its first two
holdbacks and so are 90 % of its wins — no cheap cut here. The census
column stays (ML_NOTES round 61).

### `(-256)` TAKEN — one redeal per decision, not per candidate: the sims start from a shared `SimStarts` base: sealed default Ir **-1.537 %** / cube **-2.559 %**, paired wall clock **0.991**

```text
  binary pair       dflt mirror, --games 6 --threads 1 --seed 1 (mimalloc totals)
  sealed            3,313,679,418 -> 3,262,741,089 Ir   **-1.537 %**
  cube              2,470,130,419 -> 2,406,910,404 Ir   **-2.559 %**
  wall clock        0.991 median / 0.988 mean over 7 paired reps (sealed 200 x 12)
  outcomes + census identical; --bench (gang) counters identical; golden traces 7/7 unmoved
  before: determinize_hidden 10,334 calls / 106.2 M Ir inclusive (3.14 %) under sim_start_state 119.4 M (3.53 %),
          its body: ipnsort 37.3 M (the redeal's keyed library sort), partial_shuffle 24.7 M, make_mut_slow 30.9 M
```

`sim_start_state` cloned the real state and redealt the opponent's
hidden zones inside every `simulate_*_outcome_once` — with a fixed
seed, so the ~8 candidates of one argmax (a six-game sealed run: 10,334
redeals against ~1,300 decisions) all redealt the same board. The
pickers now build `SimStarts` once per decision (one base per redeal
index `k`) and every sim clones its base; the clone was paid before
too, and the sim's first draw still unshares the library, so what
moved is the sort, the shuffle and the seeding. Read off the
`--callees determinize_hidden` table — the "price a hot row by who
calls it" rule, applied to a row that had never been on a table
because `sim_start_state` inlines into two callers.

### Round 60 ADOPTED — the open-board attack shortcut (`attack_skip_open`) on the round-58 default: sealed wall clock **-4.1 %** at no loss

```text
  arm         wall/dflt58 (median of 5 paired reps, sealed 200 x 12, one binary)   ladder pooled (4 seeds x 12,000)
  dflt-open   0.959  [0.928 .. 0.992]   <- adopted, default only                    49.98  (50.0 / 50.0 / 50.0 / 49.9, every cell within +-0.08)
  chain sims 110,382 -> 99,478 at 200 x 12; searched declarations -11.4 % (the creatureless boards)
  cross-check   cube 50.2 / 50.0 (3,200 games each), fixed 50.2 / 50.1 (1,600 each), seeds 43/97
  cube wall     dflt-open / dflt58, 200 x 12, 5 paired reps:  **0.939** median / 0.941 mean  [0.903 .. 0.970]
  --bench (gang) untouched. Golden traces: seeds 1, 4, 5 re-blessed (same winners; 11->19, 24->22, 9->7 turns).
```

The `e725e5c2` reading of the same flag on the `gang` base (-1.3/-1.8 %,
-0.1 pt on 96 k games) is superseded on the default: the search it
skips tripled in cost since, the loss did not reproduce (ML_NOTES round
60). The client pilot keeps the sim.

### `(-255)` TAKEN — the attack chain's pool resolved ahead of the sims, and an empty-greedy menu with nothing eligible returned without one: sealed default Ir **-2.167 %**, paired wall clock **0.984**

```text
  binary pair          sealed --games 6 --threads 1 --seed 1, dflt mirror (mimalloc totals)
  base (e4babdbd+r59)  3,453,977,308 Ir
  (-255)               3,379,132,442 Ir   **-2.167 %**
  cube, same recipe    2,648,931,446 -> 2,589,149,241 Ir   **-2.257 %**
  wall clock           0.984 median / 0.982 mean over 7 paired reps (sealed 200 x 12, one host, arms alternated)
  searched declarations at 100 x 12   25,384 -> 16,060  (-9,324 full turn-cycle sims of "nobody", 7.9 % of all attack sims)
  outcomes + census otherwise identical; --bench (gang) counters identical; golden traces 7/7 unmoved
```

Round 59's census (ML_NOTES) found it: under the wide flag an empty
greedy is a one-candidate menu that `pick_attacks_scored` simulated
before the chain could say its pool was empty, and on 18,754 of 22,192
such searches (2,400 games) it was — every creature summoning-sick,
tapped or barred. `attack_chain_pool` is the chain's own pool walk
(`may_declare_attacker` per own untapped creature, one freeze scope),
now run once in the picker and handed in; the `(-254)` shape on the
attack side. The chain that remains from an empty greedy runs on 1,876
searches per 1,200 games and wins 452 — leave it alone.

### Round 59 REFUTED — the empty-greedy blocker gate (`empty-gate`): paired wall clock **1.000**, ladder 50.12 pooled, not adopted

Skips the empty-greedy chain when the defender's untapped blockers are
at least as many as our untapped creatures. Census on the r58 default:
covers 3,316 of 22,192 empty-greedy searches and 752 of the chain's 886
wins there; wall clock flat because the searches it prunes are a sliver
once `(-255)` is seen. Strength-neutral on four seeds (49.9 / 50.0 /
50.2 / 50.4), so the chain's argmax wins on those boards are not ladder
wins either. Kept off as the ladder control; its walk runs only under
the flag or the census.

### Round 58 ADOPTED — the wide chain's pair move only from an empty greedy and only after the singles tie: sealed default wall clock **-14.9 %** at no strength loss

```text
  arm           wall/dflt56 (median of 5 paired reps, sealed 200 x 12, one binary)   chain sims/search   ladder pooled (4 seeds x 12,000)
  pairs-empty   0.872  [0.840 .. 0.888]                                                2.19                50.08  (cells 50.0 / 50.1 / 50.0 / 50.2)
  pairs-lazy    0.893  [0.872 .. 0.965]                                                2.48                50.00  (50.0 / 50.0 / 50.0 / 50.0)
  pairs-both    0.851  [0.833 .. 0.888]   <- adopted                                   2.18                50.10  (50.1 / 50.1 / 50.0 / 50.2)
  dflt56        1.000                                                                  3.20
  cube, pairs-both / dflt56, 100 x 12, 5 paired reps:  **0.750** median / 0.749 mean  [0.727 .. 0.773]
  --bench (gang) untouched: no chain in that profile. Golden traces re-blessed (the default's declarations move).
```

A search-budget change, not a pure optimization, so it went through the
strength gate (`.ladder/run_r58_pairs.sh`, ML_NOTES round 58) and the
traces move with it. The pair move was priced at every chain's first
step; restricted to the empty-greedy board it was built for *and* to
the step where the singles have already tied, it keeps the r56 test's
overload and drops a third of the chain's sims. Every ladder cell's
interval touches 50, none sits below it. The `fixed` bench profile
carries no chain, so the committed Baseline is unmoved; the number
that moved is the one every actor pays.

### `(-254)` TAKEN — the block chain's setup built once, and a bare block menu returned without a sim when the chain cannot run: sealed `dflt56` Ir -0.195 %

```text
  binary pair          sealed --games 6 --threads 1 --seed 1, dflt56 mirror (mimalloc, totals only)
  base (3344ba01)      4,460,465,907 Ir
  (-254)               4,451,753,506 Ir   **-0.195 %**
  block searches       11,376 -> 7,552 at 100 x 12 (-3,824 bare-menu sims, 7.2 % of all block sims)
  outcomes + attack census identical; paired wall clock 1.000 median (inside the +-3 % resolution)
```

Round 56's second candidate closed by a census denominator first: the
block chain reused the menu's start score on 65 % of *searches* because
it ran on 65 % of them — reuse is 100 % of runs. The other 34 % had no
free blocker or no legal pair, and `pick_blocks_scored` was simulating
their one-candidate menu to feed an argmax of one before the chain said
so. `BlockChainSetup::new` now answers that before the sim. Small
because the sim it removes is the cheapest one there is — a combat with
nothing blocking — and the callgrind total is the isolating instrument
at this size, not the wall clock.

### `(-253)` REFUTED — `-C llvm-args=-hot-cold-split=true` on `profiling-fast`: paired wall clock **+0.89 % median / -0.03 % mean** (noise), Ir +0.16 %, I1 misses +0.42 %

```text
  wall clock, scripts/bench_ab.py, profiling-fast --no-default-features vs the same with the flag, 16 pairs
    A median 374.18   B median 378.20   paired B/A median +0.89 %  mean -0.03 %  sd 3.78
  cachegrind, cube (system allocator on both sides)
    Ir            1,820,495,293 -> 1,823,480,183   +0.16 %
    I1 misses        71,932,638 ->    72,233,323   +0.42 %
    D1 misses        46,934,487 ->    46,843,819   -0.19 %
    mispredicts      36,146,087 ->    36,342,715   +0.54 %  (cond +1.4 %, indirect -5.0 %)
    .cold symbols  0 -> 4,840; bin_bytes 183,143,512 -> 183,533,352 (+0.2 %)
  --bench counters identical (195,806 / 27.49 / 611.9 / 0 stalls); determinism ok;
  three-pool stdout identical to the base but for the elapsed line
  built with RUSTFLAGS="-C link-arg=-fuse-ld=lld -C llvm-args=-hot-cold-split=true" CARGO_TARGET_DIR=target-hcs
```

The third side of the layout axis after `(-251)`/`(-252)`: `(-250)` said
the program is front-end-bound and PGO's -24 % says layout is the lever,
so the question was whether LLVM's *profile-free* hot/cold splitting
(static heuristics: `noreturn`, `cold` calls, unlikely branches) buys any
of that. It does not. With `panic = "abort"` the `noreturn` tails are
already the small part of every function, so what the pass splits is
guesswork — 4,840 cold sections carved out of live code, each one an
extra jump the front end fetches through — and the I1-miss count goes
*up* by 0.4 %, the conditional mispredicts by 1.4 %, the Ir by 0.16 %;
the wall clock reads flat inside the instrument's noise. One build, one
bench, two cachegrinds; not landed. **Do not rebuild.** The layout lever
needs a profile, which is PGO (opt-in, `scripts/pgo_build.sh`); no
static flag has moved it.

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

### THE ACTOR RE-READ AT THE `(-264)` TIP (`f3fb0fa9`) — the shape holds, the encoder is 6.4 % inclusive, and the SOS Prepare cast builds a definition per copy

Same recipe (`profiling-fast -p crabomination_ml --no-default-features`,
`CRAB_NO_JITTER=1 selfplay_train --actors 1 --games 60 --steps 1 --seed 7`,
callgrind, `nm | grep -cE " (T|t) (_)?mi_"` 0). **3,297,447,030 Ir**; not
comparable to `b13f5ccd`'s 2,884 M — rounds 55–64 changed the default's
pilot (the chained search runs ~1.7x `gang`), so this is a shape read.
60 games, 6,080 rows, 0 stalls, 6,566 encoded states.

```text
   now      b13f5ccd  row
  4.52 %    5.92 %    dispatch_triggers_for_events
  4.28 %    6.04 %    __memcpy_avx_unaligned_erms      make_mut_slow 231 k calls / GameState::clone 201 k / clone_from_ref_in 142 k / fmt write_str 138 k
  3.36 %    2.75 %    _int_free                        allocator cluster (_int_free, malloc, _int_malloc, free, __rdl_alloc, consolidate) 11.6 %; 1.79 M allocations
  2.98 %    2.37 %    gather_continuous_effects_inner
  2.83 %    2.47 %    compute_permanent_pass
  2.43 %    2.11 %    Vec::from_iter
  2.33 %    2.04 %    check_state_based_actions_into
  2.07 %    2.31 %    computed_permanent_hinted
  1.87 %    1.68 %    cow::make_mut_slow               self; cast_spell_with_convoke 64,427 of its 238,866 calls / 50.6 M
  1.78 %    1.97 %    encode_state_inner       } the encoder's self, 3.42 %; INCLUSIVE 209.8 M = 6.36 %, ~32 k Ir a row:
  1.64 %    1.77 %    encode_card_object_into  }   encode_card_object_into 236 k / 65.1 M, computed_permanent_hinted 152 k / 52.0 M, its scope's gather 15.7 M, sort 5.0 M
  1.27 %    1.46 %    rand_distr Normal::sample        net init, once per process (+ rand_chacha 0.46 %)
  0.93 %    1.14 %    recommend::rank_shape            the deck builder
```

New here, and actor-only: **`cast_prepare_spell` — 3,597 SOS Prepare
casts a 60-game run, each `clone()`ing the creature's inset
`prepare_spell: Box<CardDefinition>` (13.98 M, ~3.9 k a clone) and
building a fresh `CardInstance::new` around it (32.56 M, ~9 k each), then
`cast_spell` 54.1 M and `settle_prepare_after_cast` 9.5 M.** The ladder's
sealed pool prepares 764 times a six-game run (2.6 M of clone), so no
`bot_ladder` dump ranked it; the actor's sealed-cube pool is SOS-heavy.
The deep clone exists only to hand `CardInstance::new` an owned
definition — an `Arc<CardDefinition>` in the field would make it a
refcount and skip the Arc allocation inside `new`, ~1.4 % of the actor
between them. **TAKEN as `(-265)`: actor -1.803 %, sealed -0.538 %,
cube -0.496 % (Log).** Otherwise the shape holds: the engine rows in the same
order, the encoder and the deck builder the only actor-only rows,
`evaluate_requirement_static_hinted` off the top forty as at `b13f5ccd`.

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

**~~`expire_granted_triggers`' empty-map fast path~~ TAKEN with `(-277)`
(Log), which also found the larger thing beside it: the step and cast
hooks' per-permanent `visit` closures had been called out of line since
`(-228)`/`(-231)` (`FnMut for &mut F`, 626 k calls a sealed six-game run);
one walk with one call site inlined them, -0.13 % on both pools. The
census for the rest of the class (`rg '\(&mut [a-z_]+\)' crabomination/src/game`
over the `for_each`/walk sites) finds none left in the engine. What the
`FnMut for &mut F` row still holds at the `(-277)` tip (383,576 calls /
28.3 M self, sealed) is std's own `FilterMap::next` — `find_map(&mut
self.f)` — under the `filter_map(|atk| …?…)` collects in
`resolve_combat_damage_with_filter` (combat.rs, the `AttackerInfo` build)
and the bot's `AttackerFacts` build: the bodies are the payload, the call
is ~10 Ir of the 74, ~0.1 % if every one inlined; not a site anyone can
change. The `LocalKey::with` row (405 k calls / 26.6 M self) is the
`ComputedPermanent` Arc pool (`computed_permanent_hinted` 255 k takes,
`Unfreeze::drop` 146 k puts, const-initialised already): ~65 Ir a take
against malloc+free's ~150+, so the pool is the cheaper side; floor.**

**THE BLOCK SIM READ BY CONTEXT AT THE `(-278)` TIP (`--separate-callers=3`
and `=6`, sealed dflt six games, `cg.sealed.sc.out` / `cg.sealed.sc6.out`
in a scratchpad, 2,572,706,800 Ir), the read NEXT (a) asked for, so nobody
re-takes it.** 2,280 `simulate_block_outcome_once` / 196.1 M inclusive
(7.62 %; 1,494 from the menu, 786 from the chain via `_from`). Inside one
sim (~86 k Ir): `sim_step` 3.5 passes / 43.6 k (7,988 / 99.4 M), of which
`pass_priority` 84.2 M, `advance_step` 3,926 / 82.4 M, and
**`resolve_combat_into` under it 2,248 / 74.5 M — the combat-damage
resolution is 38 % of the sim**; the declaration's `declare_blockers`
2,280 / 28.4 M (12.5 k each); `submit_decision` 994 / 37.8 M (damage
order + the resumed `resolve_combat`, 30.6 M of it); `eval_material_frozen`
8.5 M; the clone's drop 8.8 M; `clone` 3.5 M; `sim_spell_action_inner`
4,632 / 2.2 M (the trick window is free here); `decide_pending_policy`
2.5 M. **So the block sim is `resolve_combat_into` (105 M of 196 M) plus
the declaration, and nothing bot-side.** Program-wide `resolve_combat_into`
is 14,624 calls / 437.5 M = **17.0 % of sealed dflt** (the largest engine
subtree; 11,220 via `advance_step`, 3,404 via `submit_decision`), ~30 k a
combat: `check_state_based_actions_into` 11,018 / 155.4 M (14.1 k a
sweep — `remove_from_battlefield_to_graveyard_raw` 15,402 / 59.1 M
program-wide at 3.8 k a death, `sba_board_scan` 1.2 k a scan,
`compute_permanents` 7,320 / 16.1 M), `deal_combat_damage_to_target`
16,086 / 64.5 M (4 k a target, 35.4 M of it
`fire_combat_damage_to_player_triggers`), `combat_damage_computed` 14,624
/ 53.8 M (3.7 k), `fire_combat_damage_triggers` 24,096 / 31.8 M (1.3 k an
event, self 1.1 k: the dealer walk + the printed loops), self 34 M (2.3 k:
the pair loop). **TAKEN `(-279)` (Log)**: the `events.reserve(32)` at the
top of the damage loop was 10,462 slow-path reallocs / 12.1 M — a fresh
clone's first combat re-growing a four-event scratch — and the nested
untap→upkeep pass leaked one buffer a turn start. What is left in this
subtree is the recorded floor: deaths (`(-217)`), the SBA scan, the two
trigger walks (`(-277)`), the views (`(-194)`..`(-196)`); the per-combat
cost moves only with the sim count or the horizon, both strength
questions.

**The combat chains (rounds 55–56) doubled the default's wall clock;
round 58 took a third of it back and this is still the top of the
list.** Sealed mirror, 12 000 games on 23 threads, one `release-fast`
binary: `gang` (no chains) 4.6 s, the r55 default (attack chain) 6.8 s,
+ block chain 7.9 s, + wide attack chain 8.7 s, the r56 default (both)
9.8 s. **Round 58 (Log): the wide chain's pair move only from an empty
greedy and only after the singles tie, `pairs-both` 0.851 of the r56
default's wall clock at no loss (four cells, every interval touching
50), adopted — the default is now ~1.8× `gang`.** `(-254)` (Log): the
bare block menu's sim skipped when the chain cannot run, Ir -0.195 %.
**`(-255)` (Log): the empty-greedy menu with nothing eligible returned
without its full-turn sim, Ir -2.17 % sealed / -2.26 % cube, wall
0.984** — 84 % of the "chains from an empty greedy" the r56 census
counted never had a pool. **Round 60 (Log): `attack_skip_open` adopted
on the default, wall 0.959 at no loss.** **`(-256)` (Log): one redeal
per decision instead of per candidate, Ir -1.54 % sealed / -2.56 %
cube, wall 0.991.** Cumulative: the adopted default runs at ~0.79 of
the r56 default's sealed wall clock, ~1.7× `gang`. The attack sim is
65.5 % of the sealed default's Ir (`simulate_attack_outcome_once`
inclusive), the block sim 6.2 %: the per-sim body is where the rest is.
The `--bench` profile is `gang`, so the committed Baseline is untouched
— every actor and every `dflt`-piloted tool pays the ratio above.
Census on the adopted default (`CRAB_ATTACK_CENSUS=1`, sealed, 1 200
games, seed 43): the attack chain runs 2.16 sims per searched declaration
(25 384 searched, 44 % from an empty greedy under the wide flag), the
block chain 4.50 per block search that reaches it (7 484, 98.8 % of
searches). In order:
(1) ~~the empty-greedy chain's pre-filter~~ CLOSED by round 59: the
chain that actually runs from an empty greedy is 1 876 searches per
1 200 games and wins 452 of them; the blocker gate was flat on wall
clock and strength-neutral. The holdback menu is CLOSED as a lead by round
61 (the deep holdbacks are 3 % of the sims and a cap is flat).
What is left of the attack search's cost is
the sim body itself — `simulate_attack_outcome_once` is 65.5 % of the
sealed default's Ir inclusive, ~50 engine priority passes a sim on a
clone, the block sim 6.2 % — and then the menu (3.3 candidates a
search) and the chain's singles (3.4 sims a search). The sim body is
read by context in the actor-path map below (the `(-256)` tip) and by
callee in the `(-260)` re-read under it; gate nothing by board class
without reading both first; (2)
~~`attack_skip_open` re-read~~ ADOPTED in round 60 (wall 0.959, no
loss); the sim-side twin — a block search whose defender faces no
attacker it could profitably block — has no census yet; (3) the block
chain's gang move builds one
`Vec` per attacker per step — read it by Ir before touching it. Measure
on `--decks sealed` (the chains' pool) and on `cube`; the `fixed` bench
does not carry them. Round 56's second candidate (the 65 % start-score
reuse) is CLOSED: it was the share of searches the chain runs on, reuse
is 100 % of runs (`block_census` now prints both).

**THE ATTACK / BLOCK SEARCH CENSUS AT THE RUN'S TIP (`CRAB_ATTACK_CENSUS=1`,
sealed dflt mirror, 1,200 games, seed 43, `2c63ce52`), so the next bot-side
round starts from numbers, not from round 68's:** 14,572 attack searches,
49,464 menu candidates (3.39 a search), **chain sims 52,224 (3.58 a
search — the chain now out-sims the menu)**, every search reuses its
start; won by greedy 48.8 % / nobody 25.7 % / a holdback 12.6 % / the
chain 12.8 % (it proposed a new set 18.0 %); from an empty greedy 1,882
chains, 386 proposed and 386 won. Block: 7,290 searches, 15,192 candidates
(2.08), chain sims 35,536 (4.87 a search), the chain ran on 98.9 % and
its plan won 37.5 %. The response layer is free (removal asks 347,418 /
sims 1,156; trick and counter sims 0). **TAKEN as round 70 (Log, ML_NOTES): the census extension (chain wins by
menu winner: 642 of 1,218 greedy-won menus, all 992 nobody menus, 238 of
406 holdbacks) priced the gate, and the gated round adopted it at 0.841 /
0.870 of the wall clock, no loss.** What is left of the chain is the
nobody / holdback / empty-greedy half (~1.8 sims a search) whose proposals
win 60-100 % of the time — not a gate candidate. The next census question
of the same shape is the block chain (4.87 sims a block search, its plan
wins 37.5 %): split *its* wins by what the block menu alone would have
chosen. 1,200 dflt sealed games ran in 6.5 s on 3 threads (4.6 s under
round 70).

**THE BLOCK CHAIN SPLIT (`CRAB_ATTACK_CENSUS=1`, sealed dflt mirror
`--games 1200` = 14,400 games, seed 43, at the round-70 tip; the
`block_census` slots 7..15), the question the entry above filed, answered
the next run:** 88,328 block searches, 183,484 candidates (2.08), the chain
ran on 98.8 % at **4.77 sims a search (420,978)** and its plan won 38.2 %.
By what the menu alone would have chosen (its argmax, greedy winning ties):

```text
  menu-alone winner   chained   chain won        sims (share)      sims/search
  non-empty greedy     33,366    8,294 (24.9 %)  196,182 (46.6 %)   5.88
  no blocks            44,272   21,566 (48.7 %)  116,024 (27.6 %)   2.62
  other (chump/gang/-1) 9,670    3,876 (40.1 %)  108,772 (25.8 %)  11.25
  cube (3,200 games):   5,298 / 1,150 (21.7 %) / 42,120 (52 %);  8,258 / 3,186 (38.6 %) / 23,894;  1,112 / 398 (35.8 %) / 14,702
```

The greedy board is the round-70 shape again — the chain grows from
"no blocks", re-derives greedy one pair at a time and beats it a quarter
of the time on the sim's own metric — and the "other" board is the
expensive one (11 sims a search: the menu's chump or gang plan means a
board with several free blockers, so every step has many pairs). The
no-blocks board is the chain's whole reason (round 56's gang from
nothing) and is left alone by every arm. **CLOSED as a gate lead by round
71 (Log, ML_NOTES, `.ladder/run_r71_blockchain.sh`): three one-flag arms
— `bchain-skipg` (no chain on the greedy board), `bchain-empty` (the
chain only from no blocks), `bchain-seed` (the chain grows from the
menu's winner) — read 48.50 / 47.67 / 48.30 pooled, every cell wholly
below 50, for ~0.89-0.90 of the sealed wall clock.** The block chain's
wins are ladder wins on every board class, and the reassignment (not the
extension) is what wins; the block side has no round-70. What is left of
the block search's cost is the sim body (`simulate_block_outcome_once`,
the combat window's passes) and the "other" board's 11 sims a search —
a cheaper sim, never a cheaper search. **And the same census, read
once more, found `(-278)` (Log): the chains re-derive the menu from
below and were re-simulating sets the menu had already priced — 32.8 %
of the attack chain's candidates and 16.5 % of the block chain's are
now read off the menu's scores, sealed -5.62 % / cube -4.92 %, traces
identical.** The chain's remaining sims are all novel sets; the next
thing of this shape would be a sim memo *across* decisions (the same
declaration re-asked after a no-op priority pass), which has no census.

**THE ACTOR RE-READ AT THE `(-278)` TIP (`cg.actor.out` in a scratchpad,
the same `--actors 1 --games 60 --steps 1 --seed 7` recipe, profiling-fast
`-p crabomination_ml --no-default-features`, system allocator confirmed:
2,490,213,350 Ir, 60 games / 6,063 rows — -20.2 % against the `2c63ce52`
record below, DIFFERENT GAMES: round 70 moved the pilot's declarations
and `(-278)` is the rest), so nobody re-takes it.** The shape: the game
90.0 % inclusive, `perform_action_inner` 57.3 %, `pick_attacks_scored`
39.1 % (`simulate_attack_outcome_once` 38.1 %, the chain's closure 12.7 %),
`main_phase_action_with` 26.6 %, `pick_blocks_scored` 7.8 %, the encoder
5.7 %, the deck builder 2.8 % (`rank_shape` 0.75 % self). Self: `__memcpy`
3.57 %, the allocator 9.8 % (`_int_free` 3.32, `_int_malloc` 2.52,
`malloc` 2.47, `free` 1.49), `dispatch_triggers_for_events` 1.74 % + 0.63
slice iteration, `compute_permanent_pass` 1.34 %, `gather_continuous_
effects_inner` 1.19 %, `sba_board_scan` 1.15 %, `perform_action_inner`
1.06 %, the CoW unshare 0.99 %, `encode_state_inner` 0.83 % + 0.49,
`encode_printed_into` 0.70 %, `Normal::sample` 0.73 % + `rand_chacha` 0.49
(the net init, once a process). The recorded shape at a lower total;
nothing above 0.2 % self with a device.

**THE ACTOR RE-READ AT THE RUN'S TIP (`2c63ce52`, `cg.actor.out` in a
scratchpad, `--actors 1 --games 60 --steps 1 --seed 7`, profiling-fast
`-p crabomination_ml --no-default-features`, system allocator confirmed:
3,121,212,158 Ir, 60 games / 6,240 rows): FLAT, the recorded shape, so
nobody re-takes it.** Self: `dispatch_triggers_for_events` 5.16 %, the
allocator 11.2 % (`_int_free` 3.29, `malloc` 2.53, `_int_malloc` 2.43,
`free` 2.05), `__memcpy` 3.23 %, `gather_continuous_effects_inner` 3.05 %,
`compute_permanent_pass` 2.82 %, `from_iter` 2.80 %, the CoW unshare's
`Arc::clone_from_ref_in` 2.79 %, `check_state_based_actions_into` 2.50 %
+ `sba_board_scan` 1.59 %, `computed_permanent_hinted` 2.01 %, the encoder
`encode_state_inner` 1.92 % + `encode_printed_into` 0.72 % +
`encode_instance_keywords_into` 0.52 % (the `(-266)`..`(-270)` floors),
`Normal::sample` 1.34 % (the net init, once a process). The closure-shaped
rows are the two the sealed read names (std's `FilterMap` `&mut F`
333,830 calls, the Arc pool's `LocalKey::with` 414,011); nothing above
0.2 % with a device.

**THE SEALED SELF TABLE RE-READ AT `1f2cabcb` (the LKI-walk fix tip,
`cg.cand.sealed.out` in a scratchpad, 3,022,028,711 Ir), so nobody
re-takes it: the shape of the `(-271)` read below, unmoved.** glibc's
`_int_free` / `malloc` / `_int_malloc` / `free` 3.40 / 2.51 / 2.31 /
1.53 % (the system-allocator profile; mimalloc is the shipped one),
`dispatch_triggers_for_events` self 2.23 % + 0.78 % slice iteration,
`__memcpy` 2.11 %, `compute_permanent_pass` 1.54 %, `sba_board_scan`
1.51 %, `gather_continuous_effects_inner` 1.46 %, `perform_action_inner`
1.36 %, `Arc::clone_from_ref_in` 1.08 % (the CoW unshare), `from_iter`
0.77 %, `GameState::clone` 0.48 %. Nothing above 0.5 % self that the
`(-200)`..`(-276)` legs have not already priced to its floor; no new row.
The floor stands, and the next perf lever remains bot-side (sim count /
horizon, a strength question) or the build (PGO, opt-in).

**THE CAPPED DEFAULT, READ BY CONTEXT AT THE ROUND-68 TIP (`62e38777`,
`--separate-callers=3`, `cg.cube.sc2.out` / `cg.sealed.sc2.out` in a
scratchpad, cube 2,957,247,714 Ir / sealed 3,015,637,612 — within 0.1 %
of the round-68 Baseline dumps after the ability-cost catalog fixes), so
nobody re-takes it.** `perform_action_inner` is 70.2 % (223,232 calls).
The attack sim is **58.9 %** (4,592 `simulate_attack_outcome_once`, 379 k
Ir each; was 63.9 % / ~500 k at the `(-275)` tip): **2,482 sims / 33.5 %
under `attack_chain_candidate` and 2,110 / 25.4 % under the greedy menu**
— the chain is now the larger half (1.18 chain sims per menu sim). Inside
it: `sim_step` 138,692 passes / 900.8 M (30.5 %) = **30.2 passes a sim,
6.5 k Ir a pass** (the `(-260)` shape; the horizon to the opponent's end
of combat sets the count); the sim's main-phase casts (`accept_on` under
`sim_spell_action_inner`) **4,932 / 166.7 M = 5.6 %** (was 17.2 % — the
round-68 cap); `sim_spell_action_inner` itself 45,772 calls / 269.6 M
(9.1 %, ~8 asks a sim, 5.9 k each — the 103 M outside the casts is the
trick / response / `cast_candidates` enumeration); the chain's own
declaration dry runs + decision submits 7,040 / 191 M (6.5 %). The block
sims 7,808 passes / 153 M (5.2 %); the real game 99 M (3.4 %);
`main_phase_action_with`'s probes 107 M (3.6 %) + `simulate_through_combat`
79 M (2.7 %). Engine-side nothing moved: `gather_continuous_effects_inner`
93,762 / 213 M (7.2 %, `frozen_effects <- compute_permanents` 21.5 k /
47 M the top context), the allocator ~5.2 % self, `dispatch_triggers_for_events`
0.86 %, `sba_board_scan` 0.9 %, `compute_permanent_pass` 0.85 %. **What
is left bot-side is the sim count** (the chain's singles: 2,482 sims for
~1,150 searched declarations) **and the horizon**, both strength
questions; the per-pass body is the engine floor.

**THE NEW CUBE DEFAULT, READ BY CONTEXT AT THE `(-275)` TIP
(`--separate-callers=3`, `cg.cube.sc.out` in a scratchpad, 3,571,023,344
Ir), so nobody re-takes it to explain the +38.5 % against the `(-274)`
block.** `perform_action_inner` is 70.4 % (246,132 calls); the attack
sim is 63.9 % (4,562 `simulate_attack_outcome_once`, ~500 k Ir each —
**33 `sim_step` passes a sim against the sealed map's 13.2**), split
2,420 sims / 36.3 % under `attack_chain_candidate` and 2,142 / 27.6 %
under `pick_attacks_scored`'s greedy. **The sim's own casts are 17.2 %**
(`sim_spell_action_inner` 61,534 calls / 611 M inside the sim, ~10 k Ir
each; `accept_on` under it 10,684 / 392 M) — the trick-mode instants
that `trick_modes_combat_only` moved out of the main-phase menu are
now cast inside the sim's combat window, which is what the cube pool
(instant-heavy) pays and sealed does not. The real game is 2.97 %; the
block sims 6.7 %; `main_phase_action_with`'s probes 3.1 %. Nothing
engine-side is new: the per-pass cost is the `(-260)` shape. The lever,
if one is wanted, is bot-side (how many spell responses a sim plays
out per pass, which is a strength question for an ML session with a
gate, not a perf leg) — filed here, not pulled. **TAKEN as round 68
(Log, ML_NOTES): `sim_main_cast_cap: Some(1)` in the default — the
sim's main-phase casts halved (cube 11,030 -> 4,928 a six-game run),
wall clock 0.837 cube / 0.864 sealed at no loss on four ladder seeds,
cube and fixed. What is left of the sim's own casts is the one cast a
main phase the gate says is the information (cap 0 loses a point);
tricks, removal and stack responses were never the cost. The next
context read is the sealed / cube tip dumps of that round
(`cg.sealed.tip.out` / `cg.cube.tip.out` in a scratchpad) — nobody has
re-read the sim's per-pass body under the capped default yet.**

**READ AT THE `(-271)` TIP (the sealed `dflt` and cube base dumps of
the `(-272)`..`(-274)` run, `cg.sealed.b.out` / `cg.cube.b.out` in a
scratchpad; the sealed self table, rows 1-90, against the `(-260)`
re-read), so nobody re-reads them:**

* **TAKEN `(-274)`** — `event_kind_bits` was the second-largest call
  row (1.09 M calls / 13.1 M): half the dispatcher's own batch fold,
  half `event_matches_spec` re-deriving it per (pair, event). Sealed
  -0.202 %, cube +0.027 %; the entry has the `SmallVec` refutation.
* **`fingerprint` 15,334 calls / 13.5 M (0.40 %)** — the CR 104.4b
  resolution watchdog, once per `resolve_top_of_stack`, ~880 Ir a call
  over the board; `(-176)`/`(-179)` already halved it and the residue
  is the per-permanent mix. Floor.
* **`ManaPool::is_empty` 216,316 calls / 6.5 M (0.19 %)** — 203,664
  from `empty_mana_pools` (both seats, every step), ~30 Ir: `total()`
  over six buckets plus the creature array and two restricted lists.
  A `u32` bucket sum on the pool would halve it; ~0.1 %, not built.
* **`sba_board_scan` 55.4 M self (1.64 %)**, `dispatch_triggers_for_events`
  self 68 M + 27 M slice iteration (2.8 %; 82 M / 2.4 % after `(-274)`
  folded the payload match in), `compute_permanent_pass` 48 M + the
  `SmallVec` and `Vec` rows under it, `gather_continuous_effects_inner`
  47 M: all where the `(-260)` table left them. The allocator is 12.4 %
  (1.87 M mallocs); `from_iter` 803 k calls / 98.5 M self is the map's
  "consumed whole" collects.

**THE ACTOR RE-READ AT THE `(-274)` TIP (`cg.actor.tip.out` in a
scratchpad, 3,154,050,563 Ir, -0.212 % against the `(-271)` record):
FLAT, so nobody re-takes it.** The growth census's top volume row is
`mint_token_with_counters` at 2.71 a call over 1,523 calls (0.72 M);
nothing above 1.5 a call has volume. The self table is the sealed
table's shape plus the encoder (`encode_state_inner` 21.1 M + 12.2 M
slice iteration, `encode_printed_into` 17.4 M, the keyword pass 9.3 M —
`(-266)`..`(-270)`'s floors) and the once-a-process rows (`Normal::sample`
42 M, `rand_chacha` 12 M, `debug_flags` 148 calls / 8.6 M: the net init
and the once-per-name `{:?}` cache, 0 in a 10 k-game run). `__memcpy`
is the top row (107 M, 3.4 %) and its callers are the CoW unshare
(417,679 calls / 9.0 M), `GameState::clone` (201,470 / 7.0 M) and glibc's
own realloc copy (82,676 / 5.9 M) — the `(-200)`/`(-201)` unshare
direction, nothing new. `format_inner` 7,687 calls / 14.6 M is
`debug_flags` (8.6 M) plus the prompt-text family (`effect_short_text`
2.1 M, `run_effect`'s prompts 1.5 M, `drain_trigger_queue` 0.8 M,
`target_phrase`/`target_noun` 1.3 M: the "not taken, ~0.35-0.5 %" entry
below, unchanged). Nothing on either pool is a 0.2 %+ lead with a known
device; the land-tap keyword gates (0.24 % actor) were the last one and
are **TAKEN as `(-275)`** (actor -0.142 %). The floor stands.

**THE ACTOR-ONLY ROWS AT THE `(-268)` TIP (`45e162d2`, the same
`selfplay_train --actors 1 --games 60 --steps 1 --seed 7` recipe,
3,196,166,585 Ir; the dump is `cg.actor.c268.out` in a scratchpad).
Three legs came off the encoder this run — `(-266)` the totals fold,
`(-267)` the battlefield object's skipped printed pass, `(-268)` one
scope per recorder snapshot — for -1.54 % of the actor between them.
What is left of the two actor-only rows, so nobody re-reads them:**

* **The encoder, 6.4 % -> ~4.9 % inclusive.** `encode_state_inner` self
  58.2 M (1.8 %, flat by line: the top line is the eight-bit keyword
  loop at 4.5 M); `encode_printed_into` 236,471 objects / 30.0 M
  (~127 Ir an object: `cmc` 22, the pip walk, `is_aura`/`is_equipment`,
  and the five type walks for the 160 k off-board objects);
  `encode_instance_keywords_into` 160,523 / 15.9 M; the layer views
  26.1 M (seat 0's encode is the first reader of the snapshot's scope
  and builds them — seat 1 and both material evals hit); the
  castability scope 15.7 M (two `mana_source_table` builds a state);
  the library sort 5.5 M; `affordable_covered` 5.1 M. **The one device
  left with a size is a per-definition memo of the printed half** (cmc,
  type bits, pips, aura/equipment: ~40 bits) on a fifth `CardMemo` word
  — ~20 M if it hits for the off-board objects, which it should (a hand
  or library card is rarely written), but every `&mut` through the
  handle clears it and 8 bytes on `CardData` is 231 k `make_mut_slow`
  copies a run. Priced, not built. **Extending the snapshot scope over
  the first bot's `next_action` is NOT a lead**: the search runs on
  clones, and a scope held open across the bot is the `(-200)`/`(-201)`
  shape with no census.
* **The deck builder, 64.0 M inclusive (2.0 %; `lattice` 120 pools x
  56 shapes = 6,720 `rank_shape` at 8.8 k each).** Read by line at this
  tip: FLAT — the top line is the `allow` bitmask test at 3.6 M, then
  `splash_cards` 7.9 M and `static_build_score` 8.5 M inclusive. It is
  `(-63)`'s shape at its floor; nothing here is a lead.
* **`Normal::sample` 41.9 M + `rand_chacha` 15.2 M (1.8 %) is the net's
  weight init, once a process** — 0 in a 10 k-game run. Not a row.
* **`(-270)` took the castability tables** (the pair shares them);
  the encoder's castability scope is now one `mana_source_table` pair
  per snapshot, and `mana_source_table` self is 13.9 M program-wide,
  most of it the bot's.
* **TAKEN `(-275)` — the sealed list's (2), priced on the actor at the
  `(-270)` tip:** `card_keyword_possible_on` 47,363 asks / 7.65 M
  (0.24 %), all from `activate_ability_inner`'s two CR 602.5 gates on a
  land tap, ~161 Ir each — the `synth` triple (three `Keyword`
  compares), `has_family(KEYWORD)`, then the grant-member walk's
  `can_grant_keyword(pred)` per member. Built as one exact-keyword
  entry over three existing folds rather than the second walker this
  entry feared: the bit is `can_grant_keyword` *at the lock predicate*
  on the gather word (bits 52-62 of the second memo word were free),
  the family fold gained a `KEYWORD` subset, and the member list is
  read by word. Actor -0.142 %, sealed -0.118 %, cube -0.172 % (Log).

**THE ACTOR-PATH MAP — `--separate-callers=3` on `--decks sealed --a dflt
--b dflt --games 6 --threads 1 --seed 1` at the `(-256)` tip
(`profiling-fast`, system allocator, 3,547,287,352 Ir; the dump is
`cg.sc.sym.out` in a scratchpad, re-take it with the recipe in "How to
measure"). The first context map ever taken under the default the
actors run; every earlier map is a `gang` dump.** Read this before
pulling anything below off the queue.

```text
  calls     incl Ir        share   context (innermost first)
  179,340   1,077,894,292  30.39 %  perform_action_inner <- sim_step <- simulate_attack_outcome_once       the sim's priority passes, ~6.0 k Ir each, 13.2 a sim
    9,460     330,158,120   9.31 %  perform_action_inner <- accept_on <- sim_spell_action_inner (attack sim)  the sim's own casts, ~35 k Ir each, 0.7 a sim
   13,622     279,652,692   7.89 %  perform_action_inner <- simulate_attack_outcome_once = 8,230 decision submissions (submit_decision 217 M, 26 k each: the
                                     resumed resolution work, resolve_combat <- submit_decision 54 M of it) + 5,392 DeclareAttackers dry runs (~79 M, ~14.6 k each:
                                     SBA 19.4 M, compute_permanents 9.5 M, the banded declaration's collects and one battlefield unshare)
   34,406     146,623,189   4.13 %  perform_action_inner <- perform_action <- play_one_game_traced           THE REAL GAME: 4.1 % of the run
    6,558     136,812,770   3.86 %  perform_action_inner <- accept_on <- main_phase_action_with              the real main-phase probes
    9,396     123,220,627   3.47 %  perform_action_inner <- evaluate_action_sequence <- pick_by_outcome
   18,974     120,853,683   3.41 %  perform_action_inner <- perform_action <- simulate_through_combat <- main_phase_action_with
    8,304     132,013,000   3.72 %  perform_action_inner <- sim_step / _once <- simulate_block_outcome_once  the block sims, all contexts
  inside the sim's pass:  pass_priority 743,984,247 (20.97 %) = advance_step 461 M (62,376 calls, 7.4 k each) + resolve_top_of_stack 261 M (20,442, 12.8 k each);
                          dispatch_triggers_for_events 154 M (4.35 %, 860 Ir a pass, 11 event_matches_spec + 1.2 event_kind_bits each)
  SBA by context:         resolve_combat_into <- advance_step 12,268 calls / 135.8 M (3.83 %, 11 k a sweep — compute_permanents for the lethal-damage walk);
                          resolve_top_of_stack_inner 24,918 / 87.1 M (2.46 %, 3.5 k); resolve_combat <- submit_decision 1,558 / 54.2 M (1.53 %, the damage-order decision's own sweep)
  gathers by context:     118,802 calls / 127 M (3.58 %): frozen_effects <- compute_permanents 25,624 / 26.0 M; combat_damage_computed <- resolve_combat_into 17,574 / 20.3 M;
                          permanent_value_with 13,750 / 13.8 M; pick_attacks_inner 8,332 / 8.9 M (the sim's greedy declarations)
  collects (from_iter):   945,942 calls / 305 M (8.61 %): pick_blocks_inner <- with_frozen_layers <- simulate_attack_outcome_once 12,904 / 35.9 M (1.01 %);
                          compute_permanents <- combat_damage_computed 17,574 / 33.4 M; compute_permanents <- declare_blockers 14,032 / 20.8 M;
                          pick_attacks_inner <- simulate_attack_outcome_once 10,008 / 15.9 M; fire_delayed_event_watchers 113,768 / 8.8 M
  CoW unshares:           make_mut_slow 267,552 calls / 202 M (5.70 %): cast_spell_with_convoke 72,782 / 58.6 M (1.65 %, ~2.6 zones a cast);
                          resolve_top_of_stack_inner 13,050 / 12.5 M; find_by_id_mut <- activate_ability_inner 15,814 / 8.5 M; determinize_hidden 10,630 / 6.9 M (after (-256))
  allocator (glibc):      _int_free 116.5 M (3.28 %) + _int_malloc 69.7 M + realloc 26.8 M + Arc::drop_slow free 26.7 M + finish_grow malloc 20.2 M ~ 7.3 %;
                          6,181,581 __rust_alloc calls: finish_grow 345,657 / 38.7 M, Arc::clone_from_ref_in <- make_mut_slow 257,396 / 33.1 M (the unshare copies)
```

What it says. (1) **The real game is 4 % of the run; 96 % is the bot's
probes, and 71 % of everything is `perform_action_inner`** — the engine
under the sims, not the bot's own logic; the bot's non-engine overhead
(`cast_candidates` 23.7 M self, `available_mana` 28.0 M, the block
planner's collects 35.9 M, `permanent_value_with`'s gathers 13.8 M) is
~4-5 % all told. (2) The per-sim fixed costs are the declaration's dry run
(~14.6 k Ir, 2.2 %) and the redeal (now once a decision, `(-256)`); a
full-turn sim is ~13 passes at 6 k, 0.6 decision submissions at 26 k
(the resumed resolution, mostly combat damage after a damage-order
answer) and 0.7 casts at 35 k.
(3) The sim's casts (`attack_sim_spells`, 9.3 %) are the fidelity that
adopted the flag; pricing them down means a cheaper cast path in the
engine, not a bot change. (4) The SBA sweep after combat damage (11 k Ir a sweep,
3.8 %) is **deaths, not views**: its callees are
`remove_from_battlefield_to_graveyard_raw` 10,832 calls / 43.7 M,
`place_card_at_resolved_zone` 11.7 M, `on_left_battlefield` 10.0 M,
`take_by_id` 6.2 M — ~6.6 k Ir per creature death end to end — against
`sba_board_scan` 13.6 M and `compute_permanents` 13.2 M (0.37 %; the
lethal-damage walk's views are not the cost). A death is a `take_by_id`
(a battlefield unshare + remove), a graveyard push, the left-battlefield
walk and the death registries; the sims kill ~0.8 creatures a sim. Price
the move path by `--callees remove_from_battlefield_to_graveyard_raw`
before touching it; the last leg there was `(-217)`.
(5) The `gang`-era rows (the dispatcher mask, the presence lanes, the
gathers' floors) hold their shape here; nothing in this table is a new
1 %+ self row that the earlier maps did not already price.

**Read off the same map at the `(-256)` tip (base retaken here at
3,547,131,260 Ir, 0.004 % off the map's total), the libc and std rows
by caller — `--callers __memcpy`, `--callers __rust_alloc`,
`cg_contexts.py` on `from_iter` / `SmallVec::extend` / `make_mut_slow` —
which is the read the map itself had not done:**

* **TAKEN `(-257)`** — `fire_step_triggers` was the program's
  second-largest memcpy caller (228 k calls) for a function that clones
  2,966 effects a run: the per-step `mem::take` + rebuild of
  `delayed_triggers`. Sealed -0.903 %.
* **TAKEN `(-258)`** — `fire_delayed_event_watchers`' two ungated
  per-dispatch collects and its empty-list early return. Sealed -0.114 %.
* **TAKEN `(-259)`** — `CardInstance::toughness` / `power` as one pass
  over the counter bag instead of six or seven `counter_count` scans
  (toughness 238 k calls / 23.2 M self, 159 k of them the SBA death
  filter's). Numbers in the Log.
* **TAKEN `(-260)`** — the SBA legend-rule leg (`SmallVec::extend`
  40,440 calls / 18.2 M under the sweep) behind a printed-legendary
  count of two from `sba_board_scan`. Numbers in the Log.
* **Priced and CLOSED — the death path's two `make_mut_slow` rows**
  (`on_left_battlefield` 9,644 / 7.5 M, `remove_from_battlefield_to_
  graveyard_raw` 5,234 / 7.3 M), read with `--demangle=no` +
  `cg_chain.py`: the first is the `CardData` unshare that clears
  `cast_from_hand` on the ~53 % of dying permanents that were cast — a
  real write, and moving the six `cast_from_*` flags onto the
  `CardInstance` handle (beside `attacked_last_turn`) is the only device,
  ~0.2 % for a serde-visible layout change; the second is the `PlayerData`
  unshare of the Revolt flag write, which is merely the *first* seat write
  on the death — gating it moves the unshare twenty lines down into
  `send_to_graveyard`. Neither is a lead. `find_card_anywhere_mut`
  (18,788 calls) has no unshare inside it: 0 callees on the dump.
* **Not taken, ~0.35-0.5 % — prompt text for headless seats — AND THE
  PREMISE IS HALF WRONG.** `drain_trigger_queue` builds
  `effect_short_text` + `format!` + the source name for every targeted
  trigger's `Decision::ChooseTarget` / `ChooseCards` (2,880 a run, ~10 M
  with `run_effect`'s 2,382 prompt formats; 17 M with the collects and
  the partition beside them at the `(-260)` re-read). But the bot DOES
  read prompt text: `decide_choose_cards` keys "sacrifice" / "discard"
  off the `ChooseCards` prompt, `ChooseAmount` matches its prompt, and
  every `OptionalTrigger` policy branches on `description` — only
  `ChooseTarget`'s `description` goes unread. A "no prompt text" flag
  must leave those three families intact, which is ~0.2 % of the
  ~0.5 %. Filed, not built.
* **Floors re-read here, so nobody re-reads them:** `cleanup_wear_off`'s
  battlefield `iter_mut` unshares the zone on 6,214 of 6,358 cleanups at
  ~476 Ir (≤ 0.08 %); `sba_board_scan` 1,178 Ir a sweep over 60 k sweeps
  (2.0 %) is the seven instance reads per card; `check_state_based_
  actions_into` self 1,785 a sweep is its gate chain; `fire_combat_
  damage_triggers` ~2 k self per damage event (1.55 %) is its phased
  board walks; the dispatcher's 352 k calls take the empty-batch return
  on most; `Vec::from_iter` 8.6 % inclusive is the block planner's
  per-sim `attacker_info` collect (35.9 M, consumed whole) and the layer
  pass's `compute_permanents` views (real work).

**THE SEALED `dflt` RE-READ AT THE `(-260)` TIP (`1e48b56a`,
`profiling-fast`, system allocator, sealed 3,437,314,145 Ir / cube
2,603,519,085; the dumps are `cg.sealed.base.out` and the
`--separate-callers=3` `cg.sc.out` in a scratchpad). The sim's spell
layer read by callee for the first time; the engine rows sit where the
map left them.**

```text
  share    row                                          what it is
  15.04 %  accept_on::{{closure}} 18,982 probes / 517 M  9,364 / 340.9 M under sim_spell_action_inner (36 k a probe: GameState::clone ~1.2 k + the cast itself, adopted, so nothing is wasted);
                                                        6,518 / 145.3 M under main_phase_action_with (the real game's own probes)
   6.30 %  cast_candidates 28,496 / 216.6 M              18,360 / 146.3 M in the sim, 8 k a call: can_afford_in_state_with 63,678 / 90.0 M (the SweepMana OnceCell 46,120 / 48.9 M of it, ~650 a card after it),
                                                        auto_targets_for_effect_all_slots 11,914 / 75.5 M (6.3 k each: ~20 printed_requirement_impl + 1.9 check_target_legality a call)
   3.91 %  auto_tap_for_cost_inner 15,614 / 134.5 M      activate_ability_into 47,634 / 79.0 M (1.66 k a land tap), mana_source_table 15,246 / 20.3 M (1.33 k a build), Map::fold 48,138 / 10.9 M
   2.85 %  activate_ability_inner 50,184 / 97.9 M        ~1.95 k a tap: continue_ability_resolution_x_into 48,304 / 33.6 M (696 for AddMana), find_by_id_mut 9.6 M, card_keyword_possible_on 48,028 / 7.8 M (162 each, twice a tap), ~680 self
   2.93 %  fire_step_triggers 53,690 / 100.6 M           drain_trigger_queue 53,690 / 44.7 M (enumerate_legal_targets_xc 1,562 / 19.1 M of it — the wants_ui trigger targets), the visit closure 196,768 / 18.1 M, 24.3 M self
   1.05 %  enumerate_legal_targets_xc 3,276 / 35.9 M     11 k a call, all under drain_trigger_queue: legal_targets_for_filter_scope 31.6 M = ~18 candidates x (printed_requirement_impl 87 + evaluate_requirement_static_hinted 271) + 3.4 check_target_legality (554)
   0.58 %  check_target_legality_with_source 53,828      372 self a call + a with_frozen_layers scope + the per-state freeze-memo Mutex (36,422 locks / 0.9 M); 22,626 from auto_targets, 11,084 from legal_targets
   0.47 %  Iterator::partition 12,392 / 16.2 M           8,324 / 12.8 M under finalize_cast — TAKEN (-261); 2,880 / 3.0 M is drain_trigger_queue's clickable/offboard split
   0.42 %  spell_kind 14,570 / 14.5 M                    once a cast: debug_flags 8.6 M (the once-per-name format cache's lookups), creature_types.clone() 9,766 allocs
   0.42 %  adjust_life 28,992 calls, 14.6 M self         ~420 Ir of inlined static walks a call — the seven life helpers (cannot-lose + loss-doubled on a loss, five on a gain) — TAKEN (-262)
  12.4 %   glibc allocator, 1.92 M allocations           finish_grow 393,752 / from_iter 264,716 / clone_from_ref_in 254,940 / Vec::clone 128,154 / GameState::clone 107,190 (44,152 clones) / CowBox::push 86,104 / make_mut_slow 81,924
   4.66 %  Arc::clone_from_ref_in, the CoW unshares      183,928 copies / 160.2 M: cast_spell_with_convoke 72,696 make_mut_slow / 58.5 M, determinize_hidden 17,870 / 11.2 M, declare_blockers 16,422 / 9.5 M, resolve_top_of_stack_inner 12,920 / 12.3 M
  15.0 %   resolve_combat_into 17,422 / 515 M            29.6 k a damage step: deal_combat_damage_to_target 18,604 / 78.9 M (fire_combat_damage_to_player_triggers 38.6 M, adjust_life 15.4 M), combat_damage_computed 17,422 / 62.3 M,
                                                        the SBA sweep 13,678 / 180.8 M (13.2 k: deaths — remove_from_battlefield_to_graveyard_raw 17,326 / 69.1 M, 4.0 k a death), fire_combat_damage_triggers 26,582 / 34.0 M; 2.11 grows a call (22.4 M incl)
   5.54 %  dispatch_triggers_for_events self 190.5 M     347,816 calls, ~185 k past the empty return at ~1 k self each; by line it is FLAT (top line 4.3 M): the member walk and the bookkeeping match — no lead
   3.25 %  gather_continuous_effects_inner 117,386       952 a gather: fx_pool::alloc_with 76,536 (the freeze scopes), compute_permanents 28,612, computed_permanent_hinted 12,070 — one gather per scope per distinct state
   2.97 %  check_state_based_actions_into self 102 M     59,196 sweeps, 1.7 k self each (the gate chain, toughness 159 k, effective_poison 117 k); sba_board_scan 75.1 M beside it, 1.27 k a sweep
```

What it says, cheapest lead first. (1) `adjust_life` and `finalize_cast`
were the two rows with a known device — a lane and the `(-257)` gate —
and are `(-262)` / `(-261)`. (2) `activate_ability_inner`'s land tap asks
`card_keyword_possible_on` twice at 162 Ir (the two CR 602.5 gates, each
ending in `keyword_grant_in_scope`) — 0.22 %, a `dispatch_bits`-shaped
per-definition bit would settle both. (3) ~~`spell_kind` builds a
`Vec<CreatureType>` clone per cast for a field two spend restrictions
read; a borrowed slice is a `SpellKind<'a>` lifetime across 18
construction sites, ~0.05 %, and `wants_converge`'s cache lookup is the
other 0.25 % — a bit on `CardData` beside `trigger_kind_fold` would be
the shape.~~ **CLOSED, built and refuted on the actor (the `(-266)`
run's Log READ): the "cache lookup" is 148 once-per-name `format!`s
(11.1 M of `debug_flags`' 12.9 M inclusive), the L1 misses 384 of
43,406 asks, and a `CardMemo` bit misses anyway — the cast writes the
card through `DerefMut` (the zone move) right before asking, which
clears the memo (14,770 of 14,881 asks fell through). The `SmallVec`
half plus the memo read -0.105 % on the actor; reverted.** (4) `legal_targets_for_filter_scope`'s walker calls (60 k / 16.4 M) were
battlefield permanents whose trigger-prompt filter the printed evaluator
DECLINED — `(-263)` proved it is not the graveyard loop (flat, reverted),
the census found bare `InGraveyard` on 54 % of the prompts, and `(-264)`
gave it (and `InExile`, the player-only arms) a printed answer. The
22,886 walker calls left there are the graveyard-card candidates
(`is_legal` by id) and `PowerAtLeast`; its auto-target twin
`auto_targets_for_effect_all_slots` (2.2 %) is answered 88 % by the
printed path already.
(5) ~~`resolve_combat_into` is the one `cg_growth.py` row above 2 grows a
call with volume (36,822 / 17,422); `declare_attackers_banded` 1.99 —
the `(-108)` reserve shape, but `(-227)` refuted a reserve one function
up, so census which `Vec` first.~~ Both TAKEN: `resolve_combat_into`'s
row was `damaged_by_this_turn` (`(-271)`), `declare_attackers_banded`'s
the events buffer (`(-273)`, split by allocator entry first: 23,212
first allocations / 5,166 re-growths). The growth table at the `(-273)`
tip has no volume row left above 1.35 a call except `do_untap` (8,514
growths / 6,326 calls, 2.2 M — `(-249)`'s residue) and
`Iterator::partition` (`(-272)`, REFUTED: the row is its predicate). (6) **The sim's spell layer is ~18 % of
the sealed default** (enumerate 4.3 + probe/cast 9.9 + the tapper under
it 3.9) and none of it is waste in the engine's sense: a probe is
adopted, a candidate is scored on its target (so lazy targeting is a bot
change, not a pure one), and the tapper's 1.66 k a land is the settled
`(-204)` path. It is the ML session's flag (`attack_sim_spells`); price
it there. (7) The dispatcher, the gather, the SBA sweep and the death
path are at the floors the earlier reads set; nothing new.

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
**`-C llvm-args=-hot-cold-split=true` (`(-253)`: flat wall clock,
Ir +0.16 %, I1 misses +0.42 % — profile-free splitting carves live
code, not cold code, once `panic = "abort"` has removed the unwind
tails)**. The inlining axis is closed from both sides and the
profile-free layout axis is closed too; what is left on the build is
PGO (a profile, not a heuristic) and the unshare direction in source. Not tried, with the reason: fat LTO on
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
