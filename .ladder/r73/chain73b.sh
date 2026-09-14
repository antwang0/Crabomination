#!/usr/bin/env bash
set -u
cd /home/archuser/repos/Crabomination
echo "=== train start $(date)"
.ladder/run_r73_settled.sh train || { echo "=== chain73b STOP: train"; exit 1; }
echo "=== gate start $(date)"
.ladder/run_r73_settled.sh gate
echo "=== chain73b done $(date)"
