#!/usr/bin/env bash
# Round 72 throughput A/B/C, interleaved, run only on a quiet box: waits for
# cs2 to exit and the GPU to sit under 15 % for 60 s, then A B C A B C at
# 4 000 PG steps each on seed 43 (games capped so generation never binds).
#   A = batch 256 inline packing (--pg-no-prefetch, the historical step)
#   B = batch 256 + host-buffer prefetch worker
#   C = batch 1024 + prefetch, lr 6e-4 (the candidate recipe)
set -u
cd /home/archuser/repos/Crabomination
echo "=== rebuild start $(date)"
cargo build --release -p crabomination_ml --features cuda --bin selfplay_train 2>&1 | grep -E "^error|Finished" | tail -2
quiet=0
while [ $quiet -lt 6 ]; do
  if pgrep -x cs2 > /dev/null; then quiet=0; sleep 30; continue; fi
  u=$(nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader,nounits | head -1)
  if [ "${u:-100}" -lt 15 ]; then quiet=$((quiet+1)); else quiet=0; fi
  sleep 10
done
echo "=== box quiet, ABC start $(date)"
run_arm() { # name batch lr extra
  local out=".ladder/r72/abc_$1"; rm -rf "$out"; mkdir -p "$out"
  ./target/release/selfplay_train --pg-only --use-best nets_r57_ctrl_s43/best.safetensors --pilot net67 \
    --sample-temp 300 --sample-turns 99 --seed 43 --games 60000 --steps 4000 --lr "$3" --lr-cosine 4000 \
    --pg-batch "$2" --pg-entropy 0.01 --pg-rho-max 2 --pg-lambda 1 --actors 3 --holdout 0.05 \
    --checkpoint-every 1000 --min-window 20000 $4 --out "$out" > "$out/log.txt" 2>&1
  # the step-3000 checkpoint: past warm-up, before the tail
  sed -n '3p' "$out/stats.jsonl" | python3 -c '
import json,sys; d=json.loads(sys.stdin.read())
print("  '"$1"' batch '"$2"': steps/s %5.1f  decisions/s %6.0f  t_step %5.1f ms  wait %4.1f ms  sleep %4.1f ms"%(d["steps_per_s"],d["consumed_per_s"],d["t_step_ms"]/1000,d["t_sample_ms"]/1000,d["t_sleep_ms"]/1000))'
}
for rep in 1 2; do
  run_arm A${rep} 256 3e-4 --pg-no-prefetch
  run_arm B${rep} 256 3e-4 ""
  run_arm C${rep} 1024 6e-4 ""
done
echo "=== ABC done $(date)"
