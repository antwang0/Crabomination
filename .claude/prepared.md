# Prepare Mechanic — quick reference

Real SOS mechanic (per WotC's "Secrets of Strixhaven Mechanics" article
and the set's release notes). Reminder text: *"While it's prepared, you
may cast a copy of its spell. Doing so unprepares it."*

> Historical note: the first implementation modeled preparation cards as
> engine-invented MDFCs whose spell half was cast from hand via
> `CastSpellBack`. That was wrong — the real cards use an Adventure/Omen
> style inset frame, are creature cards in **every** zone, and the spell
> is cast as a **copy, from exile, off the prepared battlefield creature**.
> The MDFC model was replaced in June 2026.

## Card model

A preparation card is a creature whose `CardDefinition` carries
`prepare_spell: Some(Box<CardDefinition>)` — the inset spell (own name,
cost, type, effect). It is **not** a `back_face`: the card is a creature
for casting, searching, and deck building, and the spell side is never
castable from hand. Cards printed "This creature enters prepared." set
`enters_with_counters: Some((CounterType::Prepared, Value::Const(1)))`.

**Catalog:** `crabomination_catalog/src/sets/sos/mdfcs.rs` (module name
kept for history; contents are preparation cards) plus Studious
First-Year in `sos/creatures.rs`.

## The prepared designation

`CounterType::Prepared` (count-1 flag) on the permanent, surfaced through
`PermanentView.counters`. Toggle cards (Biblioplex Tomekeeper, Skycoach
Waypoint) add/remove it via `Effect::AddCounter` / `RemoveCounter`, with
the printed "(Only creatures with prepare spells can become prepared.)"
reminder enforced by `SelectionRequirement::HasPrepareSpell` target
filters. Payoffs read it via
`SelectionRequirement::WithCounter(CounterType::Prepared)` (Top of the
Class anthem). Many preparation cards also prepare themselves via
printed triggers/activations (attack triggers, landfall, etc.).

## Casting the prepare spell

`GameAction::CastPrepareSpell { creature_id, .. }` →
`GameState::cast_prepare_spell` (`game/actions.rs`):

- Legal only while `creature_id` is on the battlefield, controlled by
  the acting player (a stolen creature brings its spell along), and
  carries a Prepared counter; else `GameError::NotPrepared`.
- A fresh `CardInstance` of the prepare spell is hopped through the
  caster's hand into the regular `cast_spell` pipeline — payment,
  timing (instant vs sorcery speed), targeting, and cast triggers all
  apply normally. On success the stack item is flagged `is_token`
  (CR 707.10a — the copy ceases to exist off the stack and never hits
  a graveyard) and the Prepared counter is removed ("unprepares it").
- Known approximation: the engine doesn't model a persistent copy
  object sitting in exile while prepared, so "cast from exile"
  zone-watch triggers don't see it; the copy materializes at cast time.

**Affordance:** `prepare_castable` (ClientView root) — prepared
creatures whose spell `would_accept` right now. Feeds the client's
ability-menu entry and `auto_advance_p0`'s `has_instant_play` hold.

**Client UX:** right-click (or `M`) on your prepared creature opens the
ability menu with a "Cast <spell> {cost}" entry (greyed when not
currently payable/timeable). Targeted spells arm the standard targeting
cursor via `TargetingState.pending_prepare_source`.

**Image prefetch:** prepare-spell names are prefetched as plain fronts
(`main.rs::visit`) — Scryfall's `cards/named` resolves inset face names
— so the cast copy has art on the stack.

## Official rulings worth remembering (release notes)

- Only the **current controller** of the prepared creature may cast the
  copy; casting it counts as casting a spell.
- A creature can't become prepared while already prepared.
- The copy ceases to exist if the creature leaves the battlefield or
  becomes unprepared.
- Cost reductions / alternative-cost effects apply to the copy.
- Prepared isn't a copiable value.

See TODO.md → "Prepare Mechanic (SOS)" for per-card status.
