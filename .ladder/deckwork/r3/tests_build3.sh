#!/usr/bin/env bash
# 2026-09-06 targeting work: unit tests, pinned engine tests, golden traces, then the optimized build.
set -u
cd /home/archuser/repos/Crabomination
echo "=== tests start $(date)"
cargo nextest run -p crabomination -E 'test(/server::bot::/)' 2>&1 | tail -5
echo "--- engine tests that pin targeting"
cargo nextest run -p crabomination_tests --test core_rules -E 'test(/target|golden/)' 2>&1 | tail -4
cargo nextest run -p crabomination_tests --test sos -E 'test(/target|omens|together|critique|homesick|outburst|mind_roots|mathemagics|brilliance/)' 2>&1 | tail -4
echo "=== build start $(date)"
rm -rf target/release-fast/incremental
cargo build --profile release-fast -p crabomination --bin deck_gauntlet --bin bot_ladder --bin deck_duel 2>&1 | tail -3
ls -la target/release-fast/deck_gauntlet target/release-fast/bot_ladder target/release-fast/deck_duel
echo "=== build done $(date)"
