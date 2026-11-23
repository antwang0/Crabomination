# Crabomination

MTG engine (Rust) targeting full-card coverage plus ML training; Bevy client.

## Builds

- Always use debug builds. Never pass `--release` unless explicitly asked.
- **Two build accelerators, on by default, never used for measurement.**
  `.cargo/config.toml` links through `.cargo/mold-cc` (mold if installed, else
  lld; 2.3x on the debug link). `scripts/fast.sh` reruns any cargo command on
  nightly with the parallel frontend (`-Zthreads=8`):

      scripts/fast.sh check --workspace --exclude crabomination_client --all-targets
      scripts/fast.sh nextest run --workspace --exclude crabomination_client

  Cold `check` 77 -> 36.5 s, cold `test --no-run` 111 -> 72 s, warm rebuild
  55.6 -> ~27 s (2026-09-08). Stable and nightly artifacts coexist in
  `target/` without thrashing. ⚠ Nothing from `fast.sh` is comparable to
  PERF.md, and the `release-fast` gate below stays on stable (a different
  frontend accepts different code). The script refuses optimized profiles.
  Install mold (no root):
  `curl -sSLf https://github.com/rui314/mold/releases/download/v2.40.4/mold-2.40.4-x86_64-linux.tar.gz | tar xz -C ~/.local --strip-components=1`
- **Carve-out: benchmarks and profiles use optimized builds** (opt-level 0
  numbers measure the compiler). Throughput: `cargo run --release --bin
  bot_ladder -- --bench`. Profiling: `--profile profiling-fast` (`release-fast`
  + debuginfo; engine rebuild ~3 min vs ~24). Only these go in `PERF.md`.
  `--profile profiling` OOMs in this container (~5.9 GB rustc peak, `signal:
  9`). `profiling-lto` answers "does LTO already inline this?";
  `profiling-lines` (cold) gives per-line attribution. See PERF's "How to
  measure".
- **Optimized profiles abort on panic** (PERF `(-250)`: +4.5-5.5 % wall
  clock). The message still prints; exit code is 134, not 101. Tests always
  unwind, so `should_panic`/`catch_unwind` work there, but no optimized binary
  can catch a panic.
- **`scripts/pgo_build.sh` must stay opt-in.** PGO is 24-28 % faster under the
  same profile name, so it is easy to file as a baseline by mistake. Compare PGO
  only to PGO, and collect the profile under the profile that consumes it (a
  mismatch is silently partially applied).
- ⚠ **`cargo check` and the suite run with `debug-assertions` on.**
  `debug_assert!` bodies are dead but still type-checked in release, so code
  can break every optimized profile while check and tests stay green. **Before
  pushing, run `cargo check --profile release-fast -p crabomination --bin
  bot_ladder`** (typecheck-only, minutes).

## Container notes (the routine image)

- Install `cargo-nextest`: `curl -sSLf https://get.nexte.st/latest/linux | tar
  zx -C ~/.cargo/bin`.
- The suite is `cargo nextest run --workspace --exclude crabomination_client`.
  The client needs four apt packages to build (listed in `CLIENT_BACKLOG.md`'s
  header) and pulls in all of Bevy.
- **Build `profiling-fast`/`overflow` with nothing else compiling** — two rustc
  on the engine is a memcg OOM (`signal: 9`) that cargo reports as a compile
  failure. Warm: 9m41s; cold or contended: ~40 min. `rm -rf
  target/*/incremental` between profile switches.
- Callgrind is cheap and contention-immune (`--games 6`: ~25 s `cube`, ~10 s
  `fixed`), so run pools in parallel. The build is the cost of an A/B, not the
  measurement.
- **Disk is limited; a full disk makes background output vanish (`ENOSPC`)
  instead of failing loudly.**
  - `target/debug/incremental` grows to ~18 GB; `rm -rf` it (costs a ~4 min
    rebuild). Delete superseded A/B binaries (220 MB each) and dumps as you go.
  - ⚠ `target/debug/deps` is the bigger hoard and free to clear: every
    edit-and-test round leaves a ~280 MB stale test binary. `find
    target/debug/deps -maxdepth 1 -type f -size +50M -mmin +180 -delete`
    rebuilds nothing. Do it before a long optimized build.
  - `scripts/prune_target.py` (`--dry-run` first) prunes by rule via
    mark-and-sweep over cargo's `.fingerprint` records; it refuses while cargo
    runs. 2026-09-30: `target/` 91 -> 46 GB.
  - ⚠ Never delete an output a live unit was built against, however old — it
    rebuilds under the same hash and the new mtime cascades (pruning by atime
    once cost a Bevy + engine rebuild per profile).
- **A second worktree sharing `target/` builds nothing unless its sources are
  newer than the main tree's outputs** (cargo keys members by workspace-relative
  path; mtime is the tiebreak), so it silently hands back the tip's binary. In
  the worktree run `find crabomination/src crabomination_base/src -name '*.rs'
  -exec touch {} +` first, and `md5sum` both binaries before a callgrind.
- **`pkill -f <pattern>` kills your own shell** if the pattern is in its
  command line. Kill by pid.

## Performance

`PERF.md` is the record: a bench baseline, a before/after row per
optimization, and a "Perf candidates" queue. Every perf change lands with its
numbers and method; no measured win means revert or re-justify as a
correctness/clarity change.

**A profile number in a code comment carries the tip it was measured at**, or
is in the past tense as justification for the code's shape. Bare
present-tense counts go stale and mislead.

**A number is about a pool, not the engine.** `--bench` uses `--decks fixed`,
which has no `GrantTriggeredAbility` statics and builds decks once. Changes to
statics / grants / layers / the requirement walker also need `--decks cube`;
changes under `draft.rs` / `recommend.rs` / `selfplay.rs` need `--decks sealed
--games 1` (deck construction only). See PERF's "Which pool a change moves".

Golden traces (`crabomination_tests/tests/core_rules/golden_trace.rs`) must
stay identical across a behaviour-preserving change; a commit that moves one
says why.

**A CoW group is priced by its first write on a clone, not by the field
size.** `ResolutionScratch`, `ColdState`, `TurnRegistries`, `PlayerCold` and
`CardCold` make a bot probe's clone a refcount bump; one unguarded store (an
empty list over an empty list, a `mem::take` of an empty `Vec`) deep-copies
the group on every probe (PERF `(-280)`..`(-287)`). Guard clears / takes /
stores with the matching read, and put fields ordinary actions write on the
plain-copied state, never in a cold group. Census: `valgrind --tool=callgrind
--demangle=no`, then `cg_edges.py --callers make_mut_slow17h<hash>` per
instance and grep the top caller for the group's fields.

## Card catalog layout

Card files live under `crabomination_catalog/src/sets/`, compiled by nine part
crates under `crabomination_catalog/parts/` (via `#[path]`) and re-exported
by `crabomination_catalog`. A new `decks/` file is two lines (`#[path] mod`,
`pub use`) in `parts/decks4/src/decks.rs`; a new set module goes in
`parts/sets3/src/sets.rs`; a shared helper goes in `sets/helpers.rs`. A part
may use the core part's helpers and hub sets, never another part's files.
Rules are in `sets/mod.rs`, enforced by
`every_card_file_is_compiled_by_exactly_one_catalog_part`.

## Test suite conventions (`crabomination_tests`)

The suite is a few grouped integration-test binaries in
`crabomination_tests/tests/` (`core_rules`, `modern`, `sos`, `stx`,
`classic_sets`, `mh`, `recent_a`, `recent_b`). Keep it that way:

- **No new top-level `tests/*.rs` files** (here or in `crabomination`); add a
  module to an existing binary.
- **A new `[[bin]]` with no `#[cfg(test)]` gets `test = false`**, or `cargo
  test` links a whole harness (engine + catalog) to run zero tests. See
  `crabomination/Cargo.toml`.
- **No micro-files.** Group card tests into ~500–2000-line files by
  set/mechanic/batch range.
- **One test per card, table-driven where possible.** Cards sharing a shape
  (vanilla cast, targeted removal, enters-tapped land, ...) get one test
  looping over `catalog::` defs.
- **Generic mechanics belong in `core_rules`.** Per-card tests assert only
  what's unique to the card.
- **Regression tests are sacred.** A test whose comment cites a CR rule, bug
  fix, or ruling stays, even if it looks redundant.
- The cost is compile + link, not execution: optimize for less code.
  ⚠ **Don't merge the existing eight binaries** — four link in parallel, so
  merging halves link count but doubles each compile (PERF build-time section).
