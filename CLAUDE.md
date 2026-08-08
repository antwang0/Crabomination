# Crabomination

MTG engine (Rust) targeting full-card coverage plus ML training; Bevy client.

## Builds

- Always use debug builds. Never pass `--release` unless explicitly asked.
- **Carve-out: benchmarks and profiles are optimized builds.** A debug engine
  runs at opt-level 0, so any number measured there describes the compiler,
  not the code. Throughput runs use `--release`
  (`cargo run --release --bin bot_ladder -- --bench`); profiling uses
  `--profile profiling` (release + full debuginfo + frame pointers). Numbers
  from any other profile don't go in `PERF.md`.

## Performance

`PERF.md` is the record: a committed bench baseline, a before/after row per
optimization, and a "Perf candidates" queue. Every perf change lands with its
numbers and how they were measured; no measured win means revert or re-justify
as a correctness/clarity change. Golden traces
(`crabomination_tests/tests/core_rules/golden_trace.rs`) must stay identical
across a behaviour-preserving change — a commit that moves one says why.

## Test suite conventions (`crabomination_tests`)

The functional suite lives in `crabomination_tests/tests/` as a small number of
grouped integration-test binaries (`core_rules`, `modern`, `sos`, `stx`,
`classic_sets`, `mh`, `recent_a`, `recent_b`). Keep it that way:

- **Do not add new top-level `tests/*.rs` files.** Each one is a separate
  binary to link; add a module inside an existing binary instead.
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
