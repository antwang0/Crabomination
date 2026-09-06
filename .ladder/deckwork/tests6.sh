#!/usr/bin/env bash
cd /home/archuser/repos/Crabomination
while pgrep -f "build6.sh" >/dev/null || pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null; do sleep 10; done
echo "=== tests6 start $(date)"
grep -E "GOLDEN EXIT|BUILD EXIT|RELEASE-FAST CHECK EXIT" .ladder/deckwork/build6.log
cargo nextest run -p crabomination -E 'test(/server::bot::/)' 2>&1 | grep -E "FAIL|panicked|error\[|Summary|^error|assertion" -A 8 | head -80
echo "BOT TESTS EXIT ${PIPESTATUS[0]}"
if grep -q "BUILD EXIT 0" .ladder/deckwork/build6.log; then
  # The test patch touched only #[cfg(test)] code; rebuild is a no-op for the binaries but keeps them in step.
  cargo build --profile release-fast -p crabomination --bin deck_gauntlet --bin bot_ladder 2>&1 | grep -E "^error|Finished"
  echo "REBUILD EXIT ${PIPESTATUS[0]}"
  .ladder/deckwork/phase6.sh
fi
echo "=== tests6 done $(date)"
