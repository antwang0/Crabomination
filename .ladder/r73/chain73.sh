#!/usr/bin/env bash
set -u
cd /home/archuser/repos/Crabomination
echo "=== suite start $(date)"
scripts/fast.sh nextest run --workspace --exclude crabomination_client 2>&1 | grep -E '^\s+(FAIL|SIGABRT|TIMEOUT)|Summary|error(\[|:)'
if ! scripts/fast.sh nextest run --workspace --exclude crabomination_client 2>&1 | grep -qE 'Summary.* 0 failed|passed, 0 failed'; then
  scripts/fast.sh nextest run --workspace --exclude crabomination_client 2>&1 | grep -E 'Summary' | grep -q 'failed' && { echo "=== chain73 STOP: suite failed"; exit 1; }
fi
echo "=== release-fast gate"
cargo check --profile release-fast -p crabomination --bin bot_ladder 2>&1 | grep -E '^error|Finished' || { echo "=== chain73 STOP: gate"; exit 1; }
echo "=== rebuild start $(date)"
cargo build --release -p crabomination_ml --features cuda --bin selfplay_train 2>&1 | grep -E "^error|Finished" | tail -2
cargo build --profile release-fast -p crabomination --bin bot_ladder 2>&1 | grep -E "^error|Finished" | tail -2
ls -la --time-style=+%F_%T target/release/selfplay_train target/release-fast/bot_ladder
echo "=== probe start $(date)"
.ladder/run_r73_settled.sh probe || { echo "=== chain73 STOP: probe"; exit 1; }
echo "=== train start $(date)"
.ladder/run_r73_settled.sh train || { echo "=== chain73 STOP: train"; exit 1; }
echo "=== gate start $(date)"
.ladder/run_r73_settled.sh gate
echo "=== chain73 done $(date)"
