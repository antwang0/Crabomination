# Browser build

The Bevy client compiled to WebAssembly, playable in a browser. Online
(lobby) play only — vs-Bot / Draft / Audit / Host are native-only, since
the browser has no threads to run an in-process match on.

## Build & run

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli        # keep in sync with the wasm-bindgen crate version

web/build.sh                          # optimized bundle -> web/dist/  (--debug for fast iteration)
python3 web/serve.py                  # serve web/dist on :8000

CRAB_BIND=0.0.0.0:7777 cargo run -p crabomination_server   # WS listener on :7778 by default
```

Open http://localhost:8000, set the join address to `host:7778` (the
**WebSocket** port, not the TCP one), and join a lobby. `ws://` is
assumed; pages served over HTTPS need `wss://host` via a TLS-terminating
reverse proxy.

## Notes

- Card art is served from `assets/cards/` next to the bundle (build.sh
  symlinks the client's asset dir). There is no in-browser prefetch: run
  the native client once to populate the cache, or rsync it to the host.
  Missing art renders as name placeholders.
- Config lives in localStorage (same TOML the native build writes to disk).
- The wasm is ~90 MB; `wasm-opt -Os` and trimming Bevy default features
  are the known next wins.
