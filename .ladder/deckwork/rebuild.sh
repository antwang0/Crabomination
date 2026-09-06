#!/usr/bin/env bash
cd /home/archuser/repos/Crabomination
while pgrep -f "screen_dir.sh" >/dev/null; do sleep 10; done
while pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null; do sleep 10; done
echo "=== rebuild start $(date)"
cargo build --profile release-fast -p crabomination --bin deck_gauntlet --bin bot_ladder --bin deck_duel 2>&1 | grep -E "^(error|warning: unused|Finished|Compiling crabomination)" -A 6
echo "BUILD EXIT ${PIPESTATUS[0]}"
echo "=== rebuild done $(date)"
