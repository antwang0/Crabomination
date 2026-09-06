#!/usr/bin/env bash
cd /home/archuser/repos/Crabomination
while pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null; do sleep 10; done
echo "=== build6 start $(date)"
cargo nextest run -p crabomination -E 'test(/server::bot::/)' 2>&1 | grep -E "FAIL|panicked|error\[|Summary|^error" -A 12 | head -120
echo "BOT TESTS EXIT ${PIPESTATUS[0]}"
cargo nextest run -p crabomination_tests --test core_rules -E 'test(/golden/)' 2>&1 | tail -3
echo "GOLDEN EXIT ${PIPESTATUS[0]}"
cargo build --profile release-fast -p crabomination --bin deck_gauntlet --bin bot_ladder 2>&1 | grep -E "^error|Finished" -A 6
echo "BUILD EXIT ${PIPESTATUS[0]}"
cargo check --profile release-fast -p crabomination --bin bot_ladder 2>&1 | grep -E "^(error|warning)" -A 6 | head -20
echo "RELEASE-FAST CHECK EXIT ${PIPESTATUS[0]}"
echo "=== build6 done $(date)"
