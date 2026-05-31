# Strixhaven implementation tracker

Two adjacent catalogs: **Secrets of Strixhaven (SOS)** (`catalog::sets::sos`)
and **Strixhaven: School of Mages (STX)** (`catalog::sets::stx`).

All ✅-done cards have been elided — the tables below list only the
remaining **🟡 partial** and **⏳ todo** work. For full per-card history see
`git log -- crabomination/src/catalog/sets/{stx,sos}/`.

## Legend

- 🟡 partial — body wired with simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

## Known engine gaps surfaced by these catalogs

- **Lessons sideboard / Learn** — Eyetwitch, Pest Summoning, Hunt for
  Specimens, Field Trip, Igneous Inspiration. Approximated as `Draw 1`.
- **Cast-from-graveyard / cast-from-exile pipelines** — block several
  Paradigm cards and the lone SOS ⏳ (Improvisation Capstone).
- **Multi-target prompts on instants/sorceries** — recurring 🟡 reason
  across SOS/STX (Vibrant Outburst, Snow Day, Devious Cover-Up, Crackle
  with Power, Magma Opus). Divided-damage / per-mode multi-target slots
  remain a gap distinct from the bag-of-targets primitives.

## Red

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Steal the Show | {2}{R} | Sorcery |  | Choose one or both — / • Target player discards any number of cards, then draws that many cards. / • Steal the Show deals damage equal to the number of instant and sorcery cards in your graveyard to target creature or planeswalker. | 🟡 | Per-mode behavior is faithful (mode 0 = `DiscardAnyNumber` then draw-that-many via `Value::CardsDiscardedThisEffect`; mode 1 = damage = IS-cards in your gy). Only the "choose one **or both**" rider collapses to a single mode pick (no multi-mode-pick that fills two target slots). |
| Tablet of Discovery | {2}{R} | Artifact |  | When this artifact enters, mill a card. You may play that card this turn. (To mill a card, put the top card of your library into your graveyard.) / {T}: Add {R}. / {T}: Add {R}{R}. Spend this mana only to cast instant and sorcery spells. | 🟡 | Wired in `catalog::sets::sos::artifacts` — ETB Mill 1 + two `{T}: Add {R}` mana abilities. The "may play that card this turn" mill-rider is omitted (no per-card may-play primitive yet). The spend-restriction on the {T}: Add {R}{R} ability is omitted (no spend-restricted mana primitive). |

## Prismari (Blue-Red)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Abstract Paintmage | {U}{U/R}{R} | Creature — Djinn Sorcerer | 2/2 | At the beginning of your first main phase, add {U}{R}. Spend this mana only to cast instant and sorcery spells. | 🟡 | Wired with a `StepBegins(PreCombatMain)/ActivePlayer` trigger adding {U}{R} via `ManaPayload::Colors`; the hybrid `{U/R}` pip is modeled. Only the spend restriction is omitted (no per-pip mana metadata). |

## Lorehold (Red-White)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Lorehold, the Historian | {3}{R}{W} | Legendary Creature — Elder Dragon | 5/5 | Flying, haste / Each instant and sorcery card in your hand has miracle {2}. (You may cast a card for its miracle cost when you draw it if it's the first card you drew this turn.) / At the beginning of each opponent's upkeep, you may discard a card. If you do, draw a card. | 🟡 | Body-only wire (5/5 Flying+Haste Legendary Elder Dragon, R/W). Miracle grant on instants/sorceries in hand is omitted (no miracle/alt-cost-on-draw primitive); per-opp-upkeep loot trigger omitted (no opp-upkeep step trigger that fires for non-active player). The vanilla finisher is the most impactful printed clause — both omitted clauses are tracked in TODO.md. |
| Molten Note | {X}{R}{W} | Sorcery |  | Molten Note deals damage to target creature equal to the amount of mana spent to cast this spell. Untap all creatures you control. / Flashback {6}{R}{W} | 🟡 | X+2 damage to creature + untap all your creatures + Flashback. Only edge left: a flashback cast reads X=0, so its damage is undercounted (the "mana spent" model only tracks the {X} pip). |

### Engine pieces driven by STX

- ⏳ **Lesson sideboard model** — Eyetwitch, Hunt for Specimens, Pest
  Summoning all use Learn at some point. Currently approximated as
  `Draw 1`.
