# Prepare Mechanic — quick reference

Real SOS mechanic (WotC "Secrets of Strixhaven Mechanics" article + release
notes). Reminder text: *"While it's prepared, you may cast a copy of its
spell. Doing so unprepares it."*

> History: the first implementation modeled preparation cards as engine-invented
> MDFCs cast from hand. That was wrong — the real cards are creatures in every
> zone (Adventure/Omen-style inset frame), and the spell is cast as a **copy,
> from exile, off the prepared battlefield creature**. Replaced June 2026.

## Card model

A preparation card is a creature whose `CardDefinition` carries
`prepare_spell: Some(Box<CardDefinition>)` (the inset spell's own name, cost,
type, effect). It is **not** a `back_face` — the card is a creature for casting,
searching, and deckbuilding, and the spell is never castable from hand. Cards
printed "enters prepared" set
`enters_with_counters: Some((CounterType::Prepared, Const(1)))`.

Catalog: `crabomination_catalog/src/sets/sos/mdfcs.rs` (module name kept for
history; contents are preparation cards) plus Studious First-Year in
`sos/creatures.rs`.

## The prepared designation

`CounterType::Prepared` (count-1 flag) on the permanent, surfaced via
`PermanentView.counters`. Toggle cards (Biblioplex Tomekeeper, Skycoach
Waypoint) add/remove it via `Effect::AddCounter`/`RemoveCounter`, gated by
`SelectionRequirement::HasPrepareSpell` (only creatures with prepare spells can
become prepared). Payoffs read it via
`SelectionRequirement::WithCounter(Prepared)` (Top of the Class anthem). Many
preparation cards also prepare themselves via printed triggers/activations.

## Casting the prepare spell

`GameAction::CastPrepareSpell { creature_id, .. }` →
`GameState::cast_prepare_spell` (`game/actions.rs`):

- Legal only while `creature_id` is on the battlefield (else
  `GameError::CardNotOnBattlefield`), controlled by the actor (a stolen
  creature brings its spell along), and carries a Prepared counter; else
  `GameError::NotPrepared`.
- A fresh `CardInstance` of the spell is routed through the caster's hand into
  the normal `cast_spell` pipeline — payment, timing, targeting, and cast
  triggers all apply. The copy is registered in
  `GameState.pending_prepare_copies`, and `settle_prepare_after_cast` flags
  the stack item `is_token` (CR 707.10a — the copy never reaches a graveyard)
  and removes the Prepared counter **only once the copy is actually on the
  stack** — a mid-cast suspension (float-spend confirm, additional-cost pick)
  parks the copy in hand, keeps the creature prepared, and settles on the
  resume; a failed cast unmaterializes the copy.
- Approximation: the engine doesn't keep a persistent copy in exile while
  prepared, so "cast from exile" zone-watch triggers don't see it; the copy
  materializes at cast time.

**Affordance:** `prepare_castable` (ClientView) — prepared creatures whose spell
`would_accept` now. Feeds the ability menu and `auto_advance_p0`'s instant-play
hold.

**Client UX:** right-click (or `M`) on your prepared creature → ability menu with
a "Cast <spell> {cost}" entry (greyed when not payable/timeable). Targeted spells
arm the targeting cursor via `TargetingState.pending_prepare_source`.

**Image prefetch:** spell names are prefetched as plain fronts (`main.rs::visit`)
so the cast copy has art.

## Official rulings (release notes)

- Only the current controller may cast the copy; it counts as casting a spell.
- A creature can't become prepared while already prepared.
- The copy ceases to exist if the creature leaves play or becomes unprepared.
- Cost reductions / alternative costs apply to the copy.
- Prepared isn't a copiable value.

See TODO.md → "Prepare Mechanic (SOS)" for per-card status.
