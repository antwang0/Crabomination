# Crabomination

A Magic: The Gathering game engine and client written in Rust.

## Workspace

| Crate | Description |
| --- | --- |
| [`crabomination`](crabomination) | Pure-Rust rules engine — no rendering dependencies. Stack, priority, state-based actions, the layer system, combat, and a large hand-written card catalog. |
| [`crabomination_client`](crabomination_client) | Bevy-based 3D client: board, game-log panel, targeting/decision UI, animations. |
| [`crabomination_server`](crabomination_server) | Networked TCP multiplayer server plus the singleplayer bot. |

## Building & testing

```sh
cargo check                 # fast iteration on the engine
cargo nextest run           # run the test suite
cargo run -p crabomination_client
```

## Status & roadmap

Feature status and planning live in the tracking docs:

- [`FEATURE_ROADMAP.md`](FEATURE_ROADMAP.md) — prioritized engine/UX/infra capabilities.
- [`CUBE_FEATURES.md`](CUBE_FEATURES.md), [`DECK_FEATURES.md`](DECK_FEATURES.md), [`STRIXHAVEN2.md`](STRIXHAVEN2.md) — per-card implementation status.
- [`TODO.md`](TODO.md) — engine approximations log and rules-coverage audit.
