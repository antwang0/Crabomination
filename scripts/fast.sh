#!/usr/bin/env bash
#
# Fast typecheck / test loop: rustc's *parallel frontend*, on nightly.
#
# WHY THIS EXISTS
#
# The workspace build is one long serial chain — `crabomination_base` ->
# `crabomination_catalog` -> `crabomination` -> the eight integration binaries
# — and rustc's frontend is single-threaded on stable. Measured 2026-09-08 on
# the 24-core box, cold `cargo test --workspace --exclude crabomination_client
# --no-run`: **104.5 s makespan for 401 s of CPU, i.e. 3.84x parallelism on 24
# cores.** Twenty cores idle on average. Adding cores does not help; the chain
# is the wall. `-Zthreads` parallelises the frontend and is the single largest
# lever available without restructuring crates.
#
#     cold `cargo check --workspace --all-targets`
#       stable 1.95.0             70.6 s
#       nightly                   59.0 s
#       nightly -Zthreads=8       34.7 s          -51 %
#
#     cold `cargo test --no-run`         104.5 s -> 66.3 s   (with mold)
#       per unit: engine 37.2 -> 16.4 s, engine lib(test) 48.3 -> 31.1 s,
#                 catalog 25.1 -> 16.1 s, classic_sets 20.9 -> 16.6 s
#     warm rebuild, touch game/effects/mod.rs
#       stable+lld   55.5 / 58.6 / 55.6 s
#       this script  27.4 / 23.2 / 28.2 / 25.1 / 28.9 s      ~ -51 %
#
#     suite under it: 19,287 / 19,287 pass, 5 skipped.
#
# ⚠ THIS IS AN ITERATION TOOL, NOT AN INSTRUMENT.
#
# A nightly compiler generates different code from the pinned 1.95.0, so:
#   * nothing built here goes anywhere near PERF.md — a `--bench` or callgrind
#     number from a different compiler is not comparable to any row in it;
#   * CLAUDE.md's pre-push gate
#       cargo check --profile release-fast -p crabomination --bin bot_ladder
#     stays on **stable**. It exists to reproduce what the shipped compiler
#     does with `debug-assertions = false`, and a different frontend can accept
#     or reject different code.
# The script refuses optimized profiles outright rather than trusting the
# reader to remember that.
#
# USAGE
#
#   scripts/fast.sh check --workspace --exclude crabomination_client --all-targets
#   scripts/fast.sh nextest run --workspace --exclude crabomination_client
#   scripts/fast.sh test -p crabomination_tests --test core_rules
#
#   CRAB_THREADS=16 scripts/fast.sh check ...     # default 8
#
# The linker is NOT set here: it lives in `.cargo/config.toml`'s `linker` key
# (see `.cargo/mold-cc`) precisely so that setting RUSTFLAGS below cannot
# clobber it.

set -euo pipefail

if [ "$#" -eq 0 ]; then
    sed -n '3,50p' "$0" >&2
    exit 2
fi

threads="${CRAB_THREADS:-8}"

# ── Guard: optimized profiles belong to stable ───────────────────────────────
# `check` is typecheck-only and cheap, but the release-fast check is a *gate*
# whose whole job is to reproduce the shipped compiler, so it is refused too.
for arg in "$@"; do
    case "$arg" in
        --release|release|release-fast|profiling|profiling-fast|profiling-lto|profiling-lines|overflow|wasm-release)
            cat >&2 <<MSG
scripts/fast.sh: refusing '$arg'.

Optimized profiles are measurement instruments and are pinned to the stable
toolchain in rust-toolchain.toml. A nightly build of one is not comparable to
anything in PERF.md, and the pre-push release-fast check must see the same
compiler the shipped binary does. Run it directly instead:

    cargo ${*}
MSG
            exit 2
            ;;
    esac
done

if ! rustup run nightly rustc --version >/dev/null 2>&1; then
    cat >&2 <<'MSG'
scripts/fast.sh: no nightly toolchain.

    rustup toolchain install nightly --profile minimal

rust-toolchain.toml's 1.95.0 pin stays the default; `cargo +nightly` overrides
it per-invocation and nothing else in the tree changes.
MSG
    exit 1
fi

# Append rather than replace, so an explicit RUSTFLAGS still composes.
export RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }-Zthreads=${threads}"

exec cargo +nightly "$@"
