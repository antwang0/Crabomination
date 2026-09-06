#!/usr/bin/env bash
cd /home/archuser/repos/Crabomination
while pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null; do sleep 10; done
echo "=== gate start $(date)"
cargo check --profile release-fast -p crabomination --bin bot_ladder 2>&1 | grep -E "^(error|warning)" -A 6 | head -40
echo "RELEASE-FAST CHECK EXIT ${PIPESTATUS[0]}"
cargo nextest run -p crabomination -E 'test(/server::bot::/)' 2>&1 | tail -4
echo "BOT TESTS EXIT ${PIPESTATUS[0]}"
cargo nextest run -p crabomination_tests --test core_rules -E 'test(/golden/)' 2>&1 | tail -4
echo "GOLDEN EXIT ${PIPESTATUS[0]}"
echo "=== gate done $(date)"
