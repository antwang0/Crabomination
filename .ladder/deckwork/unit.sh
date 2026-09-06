#!/usr/bin/env bash
cd /home/archuser/repos/Crabomination
while pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null; do sleep 10; done
echo "=== unit start $(date)"
cargo nextest run -p crabomination -E 'test(stun_counters_at_x_reads_trudge) | test(bot_choose_cards)' 2>&1 | tail -15
echo "UNIT EXIT ${PIPESTATUS[0]}"
