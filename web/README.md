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

## Card art

The browser cannot download or cache art: it asks the web host for
`assets/cards/<name>.jpg` and draws a name placeholder for any it does not
get. Filling that directory is the web host's job:

```sh
cargo run -p crabomination_art --bin fill_art -- crabomination_client/assets
```

`web/dist/assets` is a symlink to that directory. `fill_art` writes the
fonts and cardback, then every catalog card's art. It resolves image URLs
from Scryfall's daily bulk card file (one 25 MB download) and fetches the
images from their unlimited image host. Per-card API lookups are left for
tokens and the few names the bulk file misses, at Scryfall's 2/second limit.
A full fill on 2026-10-07 took 8m20s: 22,184 images, 3.7 GB. A rerun skips
what is on disk and what Scryfall has no art for (`cards/.unavailable.txt`),
so run it again after a deploy that adds cards. The native client runs the
same prefetch at launch into its own asset dir.

## Deploying to a VM

The game server and the static files are separate processes; neither needs
the other's machine, but one VM can run both:

```sh
cargo build --profile release-fast -p crabomination_server -p crabomination_art
target/release-fast/fill_art crabomination_client/assets      # ~10 min the first time
web/build.sh                                                   # or copy web/dist from a build machine
CRAB_BIND=0.0.0.0:7777 target/release-fast/crabomination_server &
python3 web/serve.py 8000                                      # or nginx/caddy over web/dist
```

- Open **8000** (the page) and **7778** (the WebSocket browsers play over).
  7777 is only for native clients.
- `release-fast` rather than `release`: `release` compiles the engine as one
  codegen unit with LTO, and rustc has been OOM-killed at ~6 GB building that
  in a container (CLAUDE.md). `release-fast` splits it into 16 units with
  no LTO, so it needs much less memory and builds several times faster.
- `CRAB_STATUS_BIND` (the server's health/metrics page) must not take 7778.

## Notes

- Config lives in localStorage (same TOML the native build writes to disk).
- WebGL2 drops what it cannot run: contrast-adaptive sharpening is off in the
  browser at every quality (its pipeline fails WebGL2 validation and the
  render error quits the app), and Bevy logs the plugins it disables
  (SSAO, OIT, GPU clustering, a second directional light).
- `serve.py` does not compress; a deployment should serve the wasm gzipped
  or brotli'd. `wasm-opt -Os` and trimming Bevy default features are the
  known size wins.
