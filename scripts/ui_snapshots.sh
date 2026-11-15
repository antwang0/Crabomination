#!/usr/bin/env bash
# The client's standard screens at three window sizes, for a before/after
# look at a HUD or layout change: run it on both sides and compare the dirs.
# Not a golden-image test — a GPU, driver or font change moves pixels — but
# the same boards at the same sizes, every time.
#
#   scripts/ui_snapshots.sh OUT_DIR      (builds nothing: build the client first)
set -eu
dir=$1; mkdir -p "$dir"
shot=$(dirname "$0")/client_shot.sh
screens=(
  "duel:--layout-fixture 2"
  "duel-stack:--layout-fixture 2 --stack-depth 3"
  "duel-tokens:--layout-fixture 2 --tokens"
  "duel-help:--layout-fixture 2 --shortcut-help"
  "pod4:--layout-fixture 4"
  "pod4-stack:--layout-fixture 4 --stack-depth 3"
)
for size in 1280x720 1920x1080 2560x1440; do
  for s in "${screens[@]}"; do
    name=${s%%:*}; args=${s#*:}
    # shellcheck disable=SC2086
    W=${size%x*} H=${size#*x} "$shot" "$dir/$name-$size.png" $args --screenshot-delay 12 >/dev/null \
      && echo "$dir/$name-$size.png" || echo "FAILED $name $size" >&2
  done
done
