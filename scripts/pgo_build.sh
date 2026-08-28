#!/usr/bin/env bash
# Build a binary with profile-guided optimization: instrument, play, merge,
# rebuild. **Opt-in** — nothing in `Cargo.toml` or `.cargo/config.toml` turns
# this on, because a PGO binary's speed depends on the workload it was trained
# on and PERF's committed numbers have to stay comparable across boxes.
#
#   scripts/pgo_build.sh                      # bot_ladder, release-fast
#   scripts/pgo_build.sh selfplay_train       # the ML actor
#   PGO_PROFILE=release scripts/pgo_build.sh  # the LTO profile instead
#
# Leaves the optimized binary in `target/$PGO_PROFILE/$BIN` and the merged
# profile in `$PGO_DIR.profdata`, so a later rebuild can skip straight to the
# last step with `RUSTFLAGS="-Cprofile-use=..."` — **under the same profile**.
# Both steps here run under one `PGO_PROFILE` on purpose: a profile raised
# under `release-fast` and consumed by a `release` build is not rejected, it
# is *partially* applied, and it silently costs most of the win (-20.75 %
# became flat; the binary shrank 3.6 % instead of 13.1 %). Nothing warns, so
# the check is the binary size.
#
# **A plain `cargo build` afterwards is a full rebuild**, not an incremental
# one: changing RUSTFLAGS invalidates the whole cache. Budget for it, or keep
# PGO in its own CARGO_TARGET_DIR.
#
# The `llvm-profdata` that merges the raw profiles must be the one matching
# rustc's LLVM. The system `/usr/bin/llvm-profdata` here is 18.1.3 against
# rustc's 22.1.2 and fails on the format; `rustup component add llvm-tools`
# installs the right one, which is what this script looks for.
set -euo pipefail

BIN="${1:-bot_ladder}"
PGO_PROFILE="${PGO_PROFILE:-release-fast}"
PGO_DIR="${PGO_DIR:-/tmp/pgo-$BIN}"

PROFDATA="$(find "$(rustc --print sysroot)" -name llvm-profdata -type f | head -1)"
if [ -z "$PROFDATA" ]; then
    echo "no llvm-profdata in the toolchain: rustup component add llvm-tools" >&2
    exit 1
fi

# The training workload. Trained on a *different* seed from any workload the
# result will be measured on — a profile fitted to the exact game sequence
# under test would flatter itself. Three pools, because the engine's hot set
# differs by pool: `fixed` carries no statics at all, `cube` is grant-heavy,
# `sealed` is mostly the deck builder (see CLAUDE.md).
train() {
    local b="target/$PGO_PROFILE/$BIN"
    case "$BIN" in
    bot_ladder)
        "$b" --a gang --b gang --games 300 --decks fixed  --threads 4 --seed 7 >/dev/null
        "$b" --a gang --b gang --games 150 --decks cube   --threads 4 --seed 7 >/dev/null
        "$b" --a gang --b gang --games 20  --decks sealed --threads 4 --seed 7 >/dev/null
        ;;
    *)
        if [ -n "${PGO_TRAIN:-}" ]; then
            PGO_BIN="$b" bash -c "$PGO_TRAIN"
        else
            echo "no default training workload for '$BIN'; set PGO_TRAIN" >&2
            echo "  (it runs with PGO_BIN set to the instrumented binary)" >&2
            exit 1
        fi
        ;;
    esac
}

rm -rf "$PGO_DIR"
echo "== instrumenting $BIN ($PGO_PROFILE)"
RUSTFLAGS="-Cprofile-generate=$PGO_DIR" CARGO_INCREMENTAL=0 \
    cargo build --profile "$PGO_PROFILE" --bin "$BIN"

echo "== training"
train
echo "   $(find "$PGO_DIR" -name '*.profraw' | wc -l) raw profiles"

echo "== merging"
"$PROFDATA" merge -o "$PGO_DIR.profdata" "$PGO_DIR"

echo "== rebuilding with the profile"
RUSTFLAGS="-Cprofile-use=$PGO_DIR.profdata" CARGO_INCREMENTAL=0 \
    cargo build --profile "$PGO_PROFILE" --bin "$BIN"

echo "== target/$PGO_PROFILE/$BIN is PGO-built; profile at $PGO_DIR.profdata"
