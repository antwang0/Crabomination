# Browser build

The Bevy client compiled to WebAssembly, playable in a browser. Online
(lobby) play only — vs-Bot / Draft / Audit / Host are native-only, since
the browser has no threads to run an in-process match on. A lobby's
"Start (fill w/ bots)" still gives a one-player game: the server runs the bot.

## Build & run

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.126   # must equal the wasm-bindgen crate in Cargo.lock

web/build.sh                          # optimized bundle -> web/dist/  (--debug for fast iteration)
python3 web/serve.py                  # serve web/dist on :8000

CRAB_BIND=0.0.0.0:7777 cargo run -p crabomination_server   # WS listener on :7778 by default
```

Open http://localhost:8000 and press **Join LAN Game**. The server field
defaults to the page's host on port 7778 — the **WebSocket** port, not the
TCP one. The server opens that listener only in lobby mode (the default;
`CRAB_BOT=1` turns it off). `ws://` is assumed; pages served over HTTPS
need `wss://host` via a TLS-terminating reverse proxy.

| | profile | cold build | client-only rebuild | wasm |
|---|---|---|---|---|
| `web/build.sh` | `wasm-release` | ~18 min | ~15 min (cgu 1 + LTO) | 183 MB, 37 MB gzipped |
| `web/build.sh --debug` | `wasm-dev` | ~8.5 min | ~1.5 min | 317 MB |

Measured 2026-10-05 on 24 cores. `--debug` is `dev` at opt-level 1, not plain
`dev`: at opt-level 0 serde's derived visitor for `Effect` has more than
50,000 wasm locals, which wasm-bindgen and every browser refuse
(`[profile.wasm-dev]` in the workspace `Cargo.toml` has the detail).

`cargo check --target wasm32-unknown-unknown -p crabomination_client` is
the quick gate for wasm-only code (`#[cfg(target_arch = "wasm32")]`
branches never compile in a native check).

## Notes

- Card art is served from `assets/cards/` next to the bundle (build.sh
  symlinks the client's asset dir). There is no in-browser prefetch, and the
  client asks for `<name>.jpg`: run the native client once to populate the
  cache (its prefetch also converts art cached as `.png` before the JPEG
  switch), or rsync it to the host. Missing art renders as drawn proxy faces
  with the card's name.
- Config lives in localStorage (same TOML the native build writes to disk).
- WebGL2 drops what it cannot run: contrast-adaptive sharpening is off in the
  browser at every quality (its pipeline fails WebGL2 validation and the
  render error quits the app), and Bevy logs the plugins it disables
  (SSAO, OIT, GPU clustering, a second directional light).
- `serve.py` does not compress; a deployment should serve the wasm gzipped
  or brotli'd. `wasm-opt -Os` and trimming Bevy default features are the
  known size wins.
