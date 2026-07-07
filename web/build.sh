#!/usr/bin/env bash
# Build the browser bundle into web/dist/.
#
# Usage:
#   web/build.sh            # wasm-release profile (small + optimized)
#   web/build.sh --debug    # dev profile (fast rebuilds, huge wasm)
#
# Output layout (web/dist/):
#   index.html
#   crabomination_client.js         wasm-bindgen glue
#   crabomination_client_bg.wasm    the game
#   assets/                         symlink to crabomination_client/assets
#
# Serve web/dist over HTTP (see web/serve.py) — browsers won't run wasm
# modules from file:// URLs. The game connects to the lobby server's
# WebSocket port (CRAB_WS_BIND, default 7778).

set -euo pipefail
cd "$(dirname "$0")/.."

PROFILE=wasm-release
TARGET_DIR=wasm-release
if [[ "${1:-}" == "--debug" ]]; then
    PROFILE=dev
    TARGET_DIR=debug
fi

echo "==> cargo build (--profile $PROFILE)"
cargo build --target wasm32-unknown-unknown --profile "$PROFILE" -p crabomination_client

echo "==> wasm-bindgen"
mkdir -p web/dist
wasm-bindgen --target web --no-typescript \
    --out-dir web/dist \
    "target/wasm32-unknown-unknown/$TARGET_DIR/crabomination_client.wasm"

cp web/index.html web/dist/

# Assets are served alongside the bundle; symlink instead of copying the
# multi-GB card cache. (Deployments should copy/rsync instead.)
if [[ ! -e web/dist/assets ]]; then
    ln -s ../../crabomination_client/assets web/dist/assets
fi

SIZE=$(du -h web/dist/crabomination_client_bg.wasm | cut -f1)
echo "==> done: web/dist ($SIZE wasm). Serve it with: python3 web/serve.py"
