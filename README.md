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

## Playing your own deck against a bot

The client runs single-player matches in-process — no server needed.
Decklists are Arena/MTGO text (`4 Lightning Bolt`, `4x Name`, bare names
= 1 copy, `Deck` / `Sideboard` / `SB:` sections, `#` comments); unknown
card names are reported rather than silently dropped.

**From the menu:** pick a format (Modern, Cube, SoS, **Sealed**,
Commander), type a path into the *Deck file* field, and press **Play
Deck vs Bot**. Under Sealed the opponent is a freshly generated random
sealed build (six SoS packs, built by the same generator the
recommender's gauntlet uses) — a new one each match; under the other
formats it is the stock Modern deck. Lists are validated against the
selected format, so a Sealed/Cube/SoS list plays limited-legal 40-card
rules and a Commander list is checked as 100-card singleton.

**From the command line**, skipping the menu:

```sh
CRAB_SEALED_DECK=/path/to/mydeck.txt cargo run -p crabomination_client -- --play sealed
```

`decks/sealed.txt` is the default deck when `CRAB_SEALED_DECK` is unset.
A missing, unparseable, or under-40-card list is not fatal — the seat
falls back to a generated build, with the reason on stderr.

**The opponent** is net-evaluated MCTS (64 iterations, 3-turn horizon,
honest determinized rollouts) whenever a value net is present —
`nets/champion.safetensors` by default, `CRAB_NET` overrides — and the
heuristic bot otherwise. This is the same profile a hosted lobby seat
gets. Expect a few seconds of thinking per turn.

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
