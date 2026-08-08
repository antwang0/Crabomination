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
- [`TODO.md`](TODO.md) — the working handoff: `NEXT`, plus the open engine /
  bot / infra backlog.
- [`PERF.md`](PERF.md) — simulator perf record: bench baseline, before/after
  log, profile of record, candidate queue.
- [`ML_NOTES.md`](ML_NOTES.md) — bot/net experiment history and documented
  dead ends.
- [`ML_PIPELINE.md`](ML_PIPELINE.md) — how self-play data, the encoder, the
  nets, and the training loop fit together.
- [`ENGINE_BACKLOG.md`](ENGINE_BACKLOG.md), [`CARD_BACKLOG.md`](CARD_BACKLOG.md),
  [`SHIPPED.md`](SHIPPED.md) — archives split out of the trackers for size.
