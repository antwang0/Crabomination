# Crabomination

MTG engine (Rust) targeting full-card coverage plus ML training; Bevy client.

## Builds

- Always use debug builds. Never pass `--release` unless explicitly asked.
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
