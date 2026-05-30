# Feature roadmap

Prioritized *engine functionality* backlog (the broader card/cleanup backlog
lives in `TODO.md`). Keep this lean: list what unblocks the most cards next.

## Already shipped (don't re-propose)

- `GrantKeywordToAttackers` static — "attacking creatures you control have
  <kw>" (Blade Historian, double strike).
- `Keyword::MustBeBlocked` (CR 509.1c) — "must be blocked if able"
  (Academic Dispute), enforced in `declare_blockers` + bot block post-pass.
- `Selector::SharingNameWith` — "all permanents with the same name"
  (Maelstrom Pulse, Echoing Truth).
- `SelectionRequirement::PowerLessThanSource` (CR 702.114 Mentor) — source-
  relative "lesser power" (Combat Professor, Lorehold Mentor).
- `Value::PermanentsDestroyedThisResolution` + `ManaPayload::OfColors`
  (Culling Ritual's "add {B} or {G} per permanent destroyed").
- Flicker-and-return (`DelayUntil(NextEndStep, Move→battlefield)`) reused for
  Elemental Expressionist's magecraft.
- Ward enforcement (CR 702.21), Cascade, Casualty sac-copy, free-cast grants,
  Madness, Dredge, Enrage, Exalted — all live.
- `StaticEffect::AdditionalCost` — unconditional "spells of [filter] cost N
  more" (Thalia, Guardian of Thraben), no first-spell gate.
- `Effect::WhenTargetDiesThisTurn` + `DelayedKind::WhenCardDies` — event-keyed
  delayed trigger watching a card's death this turn (Searing Blood faithful,
  incl. deferred deaths). Register the watch *before* the damage so it's live.
- Storm (CR 702.40) — `finalize_cast` mints one token copy per prior spell
  this turn for `Keyword::Storm` spells (Grapeshot).
- Inspired (CR 702.108) — `EventKind::BecomesUntapped` fires off the untap
  step's tapped→untapped flips (Servant of Tymaret).
- Exert (CR 702.83) — `CardInstance.skip_next_untap`, set on attacking
  `Keyword::Exert` creatures (auto-exert), consumed by `do_untap`
  (Ahn-Crop Crasher).

## Tier 1 — unblocks several partial cards each

1. **Additional cast costs (sacrifice / discard as a cost)** — a pre-flight
   cost paid at cast time, not folded into resolution. Unblocks Tend the
   Pests, Necrotic Fumes, and every "As an additional cost, sacrifice …"
   card. Today these `Seq(Sacrifice, …)` at resolution.
2. **Choose-N modes ("choose two")** — a modal-spell primitive that picks N
   distinct modes with a real decision. Unblocks Cryptic Command, Prismari
   Charm (multi-target), the Lorehold "choose two" instant. Today collapsed
   to a single `ChooseMode` of bundled pairs.
3. ~~**"When target dies this turn" delayed trigger**~~ — DONE
   (`Effect::WhenTargetDiesThisTurn`). Searing Blood faithful; reusable for
   Rushed Rebirth's reanimate-on-death.
4. **`GrantActivatedAbility(applies_to)` static** — grant a `{T}: …` ability
   to a selector's permanents. Unblocks Galazeth Prismari ("artifacts tap for
   any color"), Cryptolith Rite-style mana grants.

## Tier 2 — single-card or niche

- Energy as a player resource (gain/pay energy) — ~1–2 modern cards.
- Multi-target instants/sorceries ("any number of target …") prompt.
- Copy-spell choosing new targets (CopySpell inherits originals today).
- Emblem zone (planeswalker ult emblems).
- Overload / alt-cost that swaps target filters at cast time.
