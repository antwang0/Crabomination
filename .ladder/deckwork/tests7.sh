#!/usr/bin/env bash
cd /home/archuser/repos/Crabomination
while pgrep -f "build7.sh" >/dev/null || pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null; do sleep 10; done
echo "=== tests7 start $(date)"
cargo nextest run -p crabomination -E 'test(/server::bot::/)' 2>&1 | grep -E "FAIL|panicked|Summary|^error" -A 8 | head -40
echo "BOT TESTS EXIT ${PIPESTATUS[0]}"
echo "=== tests7 done $(date)"
