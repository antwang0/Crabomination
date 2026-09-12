# Crabomination

MTG engine (Rust) targeting full-card coverage plus ML training; Bevy client.

## Builds

- Always use debug builds. Never pass `--release` unless explicitly asked.

- **The unoptimized loop has two accelerators, both opt-in-by-default and
  neither of them an instrument.** `.cargo/config.toml` links Linux through
  `.cargo/mold-cc`, a clang wrapper that uses **mold** when it is installed and
  falls back to lld when it is not (2.3x on the link of a 225 MB debug binary).
  `scripts/fast.sh` re-runs any cargo command on **nightly with rustc's
  parallel frontend** (`-Zthreads=8`):

      scripts/fast.sh check --workspace --exclude crabomination_client --all-targets
      scripts/fast.sh nextest run --workspace --exclude crabomination_client

  Measured 2026-09-08, 24 cores, ABBA on a settled box: cold workspace `check`
  **77.0 -> 36.5 s** (-52.6 %), cold `cargo test --no-run` **111.0 -> 72.0 s**
  (-35.1 %), both 4 of 4 ordered right with no overlap between the sides.
  Warm rebuild after touching `game/effects/mod.rs` **55.6 -> ~27 s**; suite
  19,288/19,288 green. The two
  toolchains' artifacts coexist in one `target/` — cargo keys fingerprints by
  compiler version, so alternating stable and `fast.sh` does *not* thrash
  (measured: 0.82 s to return to stable).
  ⚠ **Nothing from `scripts/fast.sh` is comparable to anything in PERF.md**,
  and the `release-fast` gate below stays on stable — a different frontend
  accepts and rejects different code, which is the one thing that gate exists
  to check. The script refuses optimized profiles rather than rely on memory.
  Install mold (no root):
  `curl -sSLf https://github.com/rui314/mold/releases/download/v2.40.4/mold-2.40.4-x86_64-linux.tar.gz | tar xz -C ~/.local --strip-components=1`
- **Carve-out: benchmarks and profiles are optimized builds.** A debug engine
  runs at opt-level 0, so any number measured there describes the compiler,
  not the code. Throughput runs use `--release`
  (`cargo run --release --bin bot_ladder -- --bench`); profiling uses
  **`--profile profiling-fast`** (`release-fast` + debuginfo — the same opt
  settings as the A/B binaries, and the engine rebuilds in ~3 min instead of
  ~24). Numbers from any other profile don't go in `PERF.md`.
  **`--profile profiling` does not build in this container** — rustc peaks at
  ~5.9 GB on the engine's single codegen unit and the memcg kills it
  (`signal: 9`). Use `profiling-lto` when the question is specifically
  whether LTO already inlines a callee, and `profiling-lines` (cold) when it
  is per-source-line attribution; PERF's "How to measure" has all four and
  when each is the right instrument.
- **Every optimized profile aborts on panic** (`panic = "abort"` on
  `release`, PERF `(-250)`: +4.5-5.5 % wall clock, -3.5 % Ir). A panic
  still prints its message; the process then exits 134, not 101. Tests
  always unwind (cargo forces it for test harnesses), so `should_panic`
  and `catch_unwind` work there as before — but no optimized binary can
  catch a panic, and none tries to.
- **`scripts/pgo_build.sh` is opt-in and must stay opt-in.** A PGO build
  carries its profile's name but is 24-28 % faster, so it is the one binary
  that can be filed as a baseline reading by mistake. Quote a PGO number only
  against another PGO number, and raise the profile under the profile it will
  be consumed under — a mismatched one is partially applied with no warning.

- ⚠ **`cargo check` and the suite both run with `debug-assertions` ON, so
  neither catches code that only fails with them off.** `debug_assert!`
  expands to `if cfg!(debug_assertions) { assert!(..) }` — the body is *dead*
  in release, not *absent*, so its format arguments are still type-checked;
  a release-only `()`-returning stub handed to one of them broke every
  optimized profile on this branch while `cargo check` and 19,197 tests
  stayed green. **Before pushing, run
  `cargo check --profile release-fast -p crabomination --bin bot_ladder`** —
  it is typecheck-only, so it is minutes, and it is the only gate in the loop
  that sees `debug-assertions = false`.

## Container notes (the routine image)

- `cargo-nextest` is not installed: `curl -sSLf https://get.nexte.st/latest/linux
  | tar zx -C ~/.cargo/bin`.
- The suite is `cargo nextest run --workspace --exclude crabomination_client`.
  Without the exclusion cargo builds the whole Bevy stack, and
  `crabomination_client` does not build here without four apt packages
  (`CLIENT_BACKLOG.md`'s header has them).
- **Run a `profiling-fast`/`overflow` build with nothing else compiling** —
  two rustc on the engine at once is a memcg OOM (`signal: 9`) that cargo
  reports as a compile failure. `profiling-fast` on the engine is **9m41s warm**
  (deps cached, nothing else building) and ~40 min cold or contended — the
  dependency graph, not the engine. `rm -rf target/*/incremental` between
  profile switches.
- Callgrind is cheap here and contention-immune: a `--games 6` dump is ~25 s
  on `cube`, ~10 s on `fixed`, so two pools in parallel are free. **The build
  is the whole cost of an A/B, not the measurement.**
- **Disk is a per-session allowance and a long run fills it.**
  `target/debug/incremental` reached 18 GB after one suite run plus a few
  targeted `nextest` runs, and each saved `profiling-fast` binary is 220 MB;
  a full disk makes a background command's output vanish (`ENOSPC` on the
  task log) rather than fail loudly. `rm -rf target/debug/incremental`
  costs one non-incremental engine rebuild (~4 min) and returns the space;
  delete superseded A/B binaries and dumps from the scratchpad as you go.
  ⚠ **`target/debug/deps` IS THE BIGGER HOARD AND IT COSTS NOTHING TO CLEAR.**
  Cargo keys each test binary by a content hash and never reclaims the old
  one, so every edit-and-test round leaves a ~280 MB `core_rules-<hash>` /
  `classic_sets-<hash>` / `recent_b-<hash>` behind: 21 GB of them after one
  day, of which 4 GB was live.
  `find target/debug/deps -maxdepth 1 -type f -size +50M -mmin +180 -delete`
  freed 16 GB and rebuilt nothing — a stale hash is one cargo will never look
  up again. Do it before a long optimized build, not after the build dies.
- **A second worktree sharing `target/` builds NOTHING unless its sources are
  newer than the main tree's outputs.** Cargo keys a workspace member by its
  path *relative to the workspace root*, so `/home/user/crab_base/crabomination`
  and the main tree's engine share one fingerprint and the file mtime is the
  only tiebreak: a worktree checked out before the tip's build finished reads
  "Finished in 0.16s" and hands back the tip's binary as the base (this run's
  A/B read `-501 Ir` and identical md5s before anyone looked). `find
  crabomination/src crabomination_base/src -name '*.rs' -exec touch {} +` in
  the worktree first, and `md5sum` both sides before a callgrind.
- **`pkill -f <pattern>` kills your own shell when the pattern appears in
  the command line that runs it** (a background `cargo nextest` chain
  named in the same `bash -c`). Kill by pid.

## Performance

`PERF.md` is the record: a committed bench baseline, a before/after row per
optimization, and a "Perf candidates" queue. Every perf change lands with its
numbers and how they were measured; no measured win means revert or re-justify
as a correctness/clarity change.

**A profile number in a code comment carries the tip it was measured at**, or
it is written in the past tense as the justification for the shape the code
already has. A bare present-tense count goes stale the moment a later pass
moves it and then misleads: the seventeenth robustness filter found "the
127,878 gathers a six-game run takes" (39,692 at the forty-eighth tip) and
"`RawTable::clone` ran 984,988 times under the CoW unshare" (34,220, and none
of them the field the comment is on).

**A number is about a pool, not about the engine.** `--bench` measures
`--decks fixed`, which carries no `GrantTriggeredAbility` static and builds
its decks once — so the per-card grant walk (49 % of a cube game) and the
whole deck builder (a `selfplay_train` actor builds two decks per game) are
invisible there. A change to statics / grants / layers / the requirement
walker gets a `--decks cube` reading too; a change under `draft.rs` /
`recommend.rs` / `selfplay.rs` gets `--decks sealed --games 1`, which plays
no games and so isolates deck construction. See PERF's "Which pool a change
moves".

Golden traces (`crabomination_tests/tests/core_rules/golden_trace.rs`) must
stay identical across a behaviour-preserving change — a commit that moves one
says why.

**A CoW group is priced by its first write on a clone, not by the size of
the field written.** `ResolutionScratch`, `ColdState`, `TurnRegistries`,
`PlayerCold` and `CardCold` exist so a bot probe's clone bumps a refcount
instead of copying; one unguarded store into any of them — an empty list
over an empty list, a `mem::take` of an empty `Vec`, a stamp cleared a few
lines later — deep-copies the whole group on every probe that reaches it
(PERF `(-280)`..`(-287)`, eight legs, sealed -2.2 % / actor -1.9 %). Guard
a clear / take / store with the matching read, and put a field an
ordinary action writes on the plain-copied state (or in a small group of
its kind), never in a cold one. The census is one dump:
`valgrind --tool=callgrind --demangle=no`, then
`cg_edges.py --callers make_mut_slow17h<hash>` per instance and a grep of
the top caller's body for the group's field names.

## Test suite conventions (`crabomination_tests`)

The functional suite lives in `crabomination_tests/tests/` as a small number of
grouped integration-test binaries (`core_rules`, `modern`, `sos`, `stx`,
`classic_sets`, `mh`, `recent_a`, `recent_b`). Keep it that way:

- **Do not add new top-level `tests/*.rs` files**, in this crate or in
  `crabomination`. Each one is a separate binary to link; add a module inside
  an existing binary instead. `crabomination/tests/` no longer exists.
- **A new `[[bin]]` with no `#[cfg(test)]` block gets `test = false`.**
  `cargo test` otherwise builds and links a whole extra harness — engine plus
  the 619 k-line catalog — to run zero tests. Seven of nine bins were doing
  that; turning them off took the incremental test rebuild from 25.2 s to
  21.7 s (-14 %). See `crabomination/Cargo.toml`.
- **No micro-files.** Group card tests into files of roughly 500–2000 lines by
  set/mechanic/batch range, not one tiny file per card batch.
- **One test per card, table-driven where possible.** A card gets one test for
  its primary play pattern. When several cards share a shape (vanilla cast,
  targeted removal, enters-tapped land, ...), write one table-driven test
  looping over `catalog::` defs rather than copy-pasted bodies.
- **Generic mechanics belong in `core_rules`.** Don't re-verify a shared
  mechanic (e.g. flying blocks, spectacle cost) per card if `core_rules`
  already covers it; per-card tests should assert what's unique to the card.
- **Regression tests are sacred.** A test whose comment cites a CR rule,
  bug fix, or ruling stays, even if it looks redundant.
- Test *execution* is nearly free (thousands of tests per second); the cost is
  compile + link. Optimize for fewer binaries and less code, not fewer runs.
