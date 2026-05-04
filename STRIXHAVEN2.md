# Strixhaven implementation tracker

This file tracks two adjacent Strixhaven catalogs:

1. **Secrets of Strixhaven (SOS)** — the 255-card supplemental set, parsed
   from Scryfall set `sos`. Cards live under `catalog::sets::sos` and are
   listed in the per-color tables below.
2. **Strixhaven: School of Mages (STX)** — the 2021 base set. Cards live
   under `catalog::sets::stx` and are listed in their own table at the end
   of this file (see "Strixhaven base set (STX)" below).

## Legend

- ✅ done — wired in `crate::catalog` with full functionality
- 🟡 partial — exists with simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

## Set Overview

| Color | Cards |
|---|---|
| White | 32 |
| Blue | 32 |
| Black | 33 |
| Red | 30 |
| Green | 32 |
| Prismari (Blue-Red) | 17 |
| Witherbloom (Black-Green) | 17 |
| Silverquill (White-Black) | 16 |
| Quandrix (Green-Blue) | 16 |
| Lorehold (Red-White) | 16 |
| Colorless | 14 |

## Implementation Progress

Counts reflect the regenerated tables below (audited via
`scripts/audit_strixhaven2.py` against `catalog::sets::sos`).

- ✅ done: **149** (push XLV: SOS Brush Off + Run Behind 🟡 → ✅ via
  the new self-static `CostReductionTargeting` wire — same primitive
  shape as Ajani's Response).
- 🟡 partial: **111** (-2 from the Brush Off + Run Behind promotions).
- ⏳ todo: **7** (unchanged — all blocked by cast-from-exile / copy-
  permanent pipelines).

Push XLVII (2026-05-04) targets the `decks::modern` package rather
than the SOS catalog: 4 modern promotions (Unholy Heat, Dragon's Rage
Channeler, Vendetta, Kolaghan's Command, all 🟡 → ✅) + 1 new engine
primitive (`Value::DistinctCardTypesInGraveyard`) + 1 server view
field (`PlayerView.distinct_card_types_in_graveyard`). SOS totals
unchanged.

Push XLV (2026-05-04) STX 2021 totals (per `scripts/audit_stx_base.py`):
**106 ✅ / 17 🟡 / 0 ⏳ across 123 rows.** Push XLV's contribution
to STX 2021 is zero new factories — the push focuses on the
`catalog::sets::decks::modern` package + 2 SOS promotions. Quandrix
(G/U) still the only fully closed college (8 ✅ / 0 🟡).

Push XXXVIII (2026-05-03) introduces 4 engine primitives and promotes
10 cards across STX 2021 + SOS:

**Engine primitives:**
- `StaticEffect::CostReductionTargeting` — target-aware cost reduction
  (Killian, Ajani's Response).
- `StaticEffect::CostReductionScaled` — Affinity-style cost reduction
  whose discount evaluates a `Value` at cast time (Witherbloom Balancer,
  Dawning Archaic).
- `AffectedPermanents::All { excluded_supertypes, exclude_source }` —
  layer-layer support for `Not(HasSupertype(_))` filters (Hofri's
  "Other nonlegendary creatures").
- `AlternativeCost.mode_on_alt: Option<usize>` — alt-cost-implies-mode
  (Devastating Mastery's Mastery alt cost).
- Prowess wired as a synthetic SpellCast trigger (Spectacle Mage).

**STX 2021 promotions (5):** Killian, Ink Duelist; Spectacle Mage;
Tempted by the Oriq; Devastating Mastery; Hofri Ghostforge (anthem
fix only — dies-as-Spirit still gap, stays 🟡).

**SOS promotions (5):** Ajani's Response; Witherbloom, the Balancer
(first Affinity clause only — second still gap, stays 🟡); The
Dawning Archaic (⏳ → 🟡, body wired); Inkshape Demonstrator (doc-
only); Ennis Debate Moderator (doc-only).

All 248 cards marked ✅ or 🟡 have a corresponding factory in
`crabomination/src/catalog/sets/sos/`; the audit script reports 0 false
positives and 0 stale ⏳ rows. STX 2021 progress is tracked in the
"Strixhaven base set (STX)" section near the bottom of this file.

## 2026-05-04 push XLVIII: 14 modern cards + ExtraLandPerTurn (CR 305.2) + delirium_active view + CR 305 audit

Adds 14 new cards to the `decks::modern` catalog (Talisman cycle
completion, Pristine Talisman, Wayfarer's Bauble, Burnished Hart, 5
Painlands, Exploration), wires up the long-dormant
`StaticEffect::ExtraLandPerTurn` to a new `GameState::player_can_play_
land` predicate (CR 305.2), surfaces `PlayerView.delirium_active` as a
derived flag for UI rendering, and audits CR 305 (Lands). Tests at
1501 (was 1480, +21 net), all green.

### New Modern cards (14, all in `catalog::sets::decks::modern`)

**Talisman cycle completion (5 — closes the 10-painland-pair cycle alongside the existing Progress / Dominance / Conviction / Creativity / Curiosity factories):**

| Card | Cost | Status | Notes |
|---|---|---|---|
| Talisman of Hierarchy | {2} | ✅ | WB Talisman. {T}: Add {C}. {T}: Add {W} or {B}, lose 1 life. Reuses the `talisman_cycle` helper. |
| Talisman of Impulse | {2} | ✅ | RG Talisman. {T}: Add {R} or {G}, lose 1 life. |
| Talisman of Indulgence | {2} | ✅ | BR Talisman. |
| Talisman of Resilience | {2} | ✅ | BG Talisman. |
| Talisman of Unity | {2} | ✅ | GW Talisman. |

**Painland cycle (5 — covers the 5 ally-color combinations from Apocalypse / Ice Age):**

| Card | Cost | Status | Notes |
|---|---|---|---|
| Adarkar Wastes | — | ✅ NEW | Land. WU painland. New `painland(name, type_a, type_b, c1, c2)` helper bundles `{T}: Add {C}` (no life cost) plus two colored taps that each lose 1 life. The damage is modeled as `LoseLife 1` since the engine treats both uniformly for the colored-tap cost. Index 0 = colorless, 1 = c1, 2 = c2. |
| Underground River | — | ✅ NEW | Land. UB painland twin of Adarkar Wastes. |
| Sulfurous Springs | — | ✅ NEW | Land. BR painland. |
| Karplusan Forest | — | ✅ NEW | Land. RG painland. |
| Brushland | — | ✅ NEW | Land. GW painland. Closes the ally cycle. |

**Other (4 colorless artifacts + 1 enchantment):**

| Card | Cost | Status | Notes |
|---|---|---|---|
| Pristine Talisman | {3} | ✅ NEW | Artifact. `{T}: Add {C}` + `{T}: You gain 1 life`. Lifegain ability goes on the stack (not a mana ability) and is drained. |
| Wayfarer's Bauble | {1} | ✅ NEW | Artifact. `{2}, {T}, Sacrifice this artifact: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.` Sac-as-cost folded into resolution; `Effect::Search` to BF tapped. |
| Burnished Hart | {3} | ✅ NEW | 2/2 Artifact Creature — Construct. `{3}, {T}, Sac: Search for up to two basic lands → BF tapped`. Uses `Effect::Repeat(2, Search)` (same shape as Buried Alive's `Repeat(3, Search)`). |
| Exploration | {G} | ✅ NEW | Enchantment. **First catalog card** to exercise the engine's `StaticEffect::ExtraLandPerTurn`, which has been in the StaticEffect enum for several pushes but until this push wasn't hooked into `play_land`. The new `GameState::player_can_play_land` reads the static off the controller's battlefield and bumps the per-turn land cap by 1 per Exploration in play. Multiple copies stack — two Explorations let you play three lands per turn. |

### Engine improvement: wire `StaticEffect::ExtraLandPerTurn` (CR 305.2)

`StaticEffect::ExtraLandPerTurn` was defined in the `StaticEffect` enum
since several pushes ago but was a dead branch — `Player::can_play_land()`
just checked `lands_played_this_turn == 0` flat, with no awareness of
controller-scoped statics. This push hooks the static into a new
`GameState::max_lands_per_turn(player)` helper that walks the
controller's battlefield for `ExtraLandPerTurn` instances and returns
`1 + count(extras)`. A new `GameState::player_can_play_land(player)`
predicate compares the player's `lands_played_this_turn` against the
new cap and is the canonical "may another land be played?" check.

`actions.rs::play_land_with_face` now uses
`self.player_can_play_land(p)` instead of the ad-hoc
`self.players[p].can_play_land()` check, so Exploration / Azusa /
Burgeoning-style cards correctly grant additional land plays.
`bot.rs` updated alongside so the bot greedily plays a second land
when Exploration is in play.

`Player::can_play_land()` is preserved (still checks `==0` directly)
for callers that don't have GameState in scope, but new code should
prefer the GameState method.

### Server view: `PlayerView.delirium_active`

A derived `bool` field that pre-computes the Delirium threshold
(`distinct_card_types_in_graveyard >= 4`) on the server side so
clients can render a "Delirium active" badge on Dragon's Rage
Channeler / Unholy Heat / future MH2 Delirium payoffs without needing
to recompute the threshold themselves. Same shape as
`infusion_ready` (which clients derive from `life_gained_this_turn`,
though we surface it explicitly in the SOS infusion path). Defaulted
via `#[serde(default)]` for back-compat with older serialized views.

The `view.rs` projection now extracts the distinct-types walk into a
shared `distinct_card_types_in_graveyard(player)` helper that drives
both `distinct_card_types_in_graveyard: u32` and
`delirium_active: bool` — eliminates a duplicate `HashSet` walk and
keeps the projection cleaner.

### CR 305 audit (Lands)

Per CR 305 (Lands):

- **305.1** (play a land as a special action during main phase, stack
  empty, doesn't use stack) ✅ — implemented in
  `actions.rs::play_land_with_face`. Sorcery-speed gate via
  `can_cast_sorcery_speed`; non-spell semantics enforced (no stack
  push, no SpellCast triggers fire on a land play).
- **305.2** (one land per turn baseline; continuous effects can
  increase) ✅ — newly wired this push. The new
  `GameState::max_lands_per_turn(player)` returns `1 + count(
  ExtraLandPerTurn statics on controller's battlefield)`, and
  `player_can_play_land` compares the cap against
  `lands_played_this_turn`. Exploration is the first catalog card to
  exercise the path; Azusa, Lost But Seeking / Burgeoning / Oracle of
  Mul Daya can be added the same way (each contributing one or more
  `ExtraLandPerTurn` statics).
- **305.2a** (compare cap to plays-already-this-turn) ✅ — the
  comparison runs every land play; transient grants take effect
  immediately.
- **305.2b** (can't play a land if cap reached; ignore effect
  instructions to do so) ✅ — `GameError::AlreadyPlayedLand` is
  returned at the cap; the turn-based action just stops.
- **305.3** (can't play a land on opp's turn) ✅ — `can_cast_sorcery_
  speed` returns false on opp turns, so `play_land_with_face` rejects
  with `SorcerySpeedOnly`.
- **305.4** (effects that "put" lands onto the battlefield don't
  count as a land played) ✅ — `Effect::Search` to `Battlefield`
  destination uses `Move`-style mechanics, not `play_land`, so the
  `lands_played_this_turn` counter isn't bumped. Verified by Cultivate /
  Kodama's Reach / Wood Elves' tutor paths.
- **305.5** (subtypes are single words after a long dash) ✅ — the
  `LandType` enum encodes the basic land types; nonbasic subtypes
  (e.g. Locus, Urza's, Lair) are tracked similarly. Multi-subtype
  lands (Tundra has Plains + Island) are wired via `Subtypes.land_
  types: Vec<LandType>`.
- **305.6** (basic land types confer intrinsic `{T}: Add {color}`) 🟡 —
  partial. The basic-land cards in the catalog all have an explicit
  `{T}: Add {color}` activated ability, so the *behavior* is
  correct. The engine doesn't yet treat the mana ability as
  *intrinsic* (i.e. derived from the subtype rather than the rules
  text). Cards that *change* a land's subtype (e.g. Spreading Seas's
  "becomes an Island") would need a new layer-4 subtype-rewrite +
  intrinsic-ability-lookup pass; today no such cards are in the
  catalog. Same-shape gap for the Dryad of the Ilysian Grove /
  Prismatic Omen "every land is also a Plains, Island, ..." path.
- **305.7** (subtype-set rewrite loses old land types + abilities,
  gains the new ones) ⏳ — same gap as 305.6: no subtype-rewrite
  primitive yet.
- **305.8** (`basic` supertype distinguishes basic from nonbasic
  lands) ✅ — `Supertype::Basic` is wired; basic lands carry it,
  nonbasics don't. `IsBasicLand` predicate reads it for tutor
  filters (Rampant Growth, Sakura-Tribe Elder, Burnished Hart, etc.).
- **305.9** (a card that's both a land and another card type can only
  be played as a land) 🟡 — partial. No catalog card today is both a
  land and another type (DFC / MDFC cards have separate front/back
  faces, each a single card type). `MDFC` cards explicitly support a
  `PlayLandBack` action when the back face is a land, separate from
  `CastSpell`, so the "can only play as a land" semantics are
  enforced at the action-level. A non-DFC land+other-type (e.g.
  Dryad Arbor — a creature *and* a land) would need a unified
  `PlayLand` action that recognizes the dual nature; today no such
  card is in the catalog.

### Tests (+21 net, 1480 → 1501)

- 5 Talisman tests (Hierarchy black tap, Impulse green tap, Indulgence
  colorless-tap-no-life, Resilience black tap, Unity white tap).
- 2 Pristine Talisman tests (lifegain ability adds 1 life; mana
  ability doesn't change life).
- 1 Wayfarer's Bauble test (search-and-sac).
- 2 Burnished Hart tests (search 2 basics; 2/2 Construct body).
- 6 Painland tests (Adarkar colorless-no-life, Adarkar blue cost-1,
  Underground River black, Sulfurous Springs red, Karplusan green,
  Brushland white).
- 1 `delirium_active` derived-flag test (off below threshold).
- 4 ExtraLandPerTurn (CR 305.2) tests:
  - `baseline_player_caps_at_one_land_per_turn`
  - `exploration_allows_a_second_land_play_in_one_turn`
  - `two_explorations_grant_three_land_plays`
  - `exploration_does_not_help_the_opponent`

## 2026-05-04 push XLVII: 10 modern promotions + 2 engine primitives + CR 121 audit

Adds two engine primitives (`Value::DistinctCardTypesInGraveyard`
for Modern Horizons 2 Delirium, `Effect::PreventLifegainThisTurn`
for Skullcrack-style locks) and uses them plus the existing
`Effect::If` / `Value::ToughnessOf` / `Effect::ChooseModes`
machinery to promote 10 Modern-supplement cards from 🟡 to ✅.
New `PlayerView.distinct_card_types_in_graveyard` and
`PlayerView.lifegain_prevented_this_turn` server-view fields
surface the two new flags for UI hint rendering. Tests at 1480
(was 1468; +12 net).

### New engine primitive: `Value::DistinctCardTypesInGraveyard(PlayerRef)`

A new arm of the `Value` enum that returns the count of distinct
card types across cards in the resolved player's graveyard. Same
shape as `Value::DistinctTypesInTopOfLibrary`, but sourced from the
graveyard rather than the library. Evaluated in `evaluate_value`
via a fresh `HashSet<CardType>` walk over the resolved player's
graveyard. Backs the printed "if there are four or more card types
among cards in your graveyard" gate that anchors Modern Horizons 2's
Delirium cycle.

The primitive composes with the existing `Predicate::ValueAtLeast`
and `Value::IfPredicate` arms — Unholy Heat's "deals 6 damage
instead if Delirium is on" body becomes:

```
Value::IfPredicate {
  cond: Predicate::ValueAtLeast(
    Value::DistinctCardTypesInGraveyard(PlayerRef::You),
    Value::Const(4),
  ),
  then: Box::new(Value::Const(6)),
  else_: Box::new(Value::Const(3)),
}
```

For "static while Delirium is on" body buffs (Dragon's Rage
Channeler's "+2/+2 and flying"), the primitive isn't directly usable
— Layer 7 / Layer 6 modifications need pre-computed integer values,
not Value-typed expressions. So the DRC hookup uses the same compute-
time injection pattern as Tarmogoyf / Cruel Somnophage in
`compute_battlefield`: read the controller's distinct-card-types
count, and if ≥ 4, push a `ModifyPowerToughness(2, 2)` continuous
effect plus an `AddKeyword(Flying)` continuous effect.

### Promotions (10: 🟡 → ✅)

| Card | Status | Notes |
|---|---|---|
| Unholy Heat | ✅ ← 🟡 | Delirium upgrade now wires faithfully via `Value::IfPredicate { cond: Predicate::ValueAtLeast(DistinctCardTypesInGraveyard(You), 4), then: 6, else_: 3 }`. |
| Dragon's Rage Channeler | ✅ ← 🟡 | "+2/+2 and flying as long as ≥4 distinct card types in your graveyard" now wires via compute-time injection in `compute_battlefield` (same path as Tarmogoyf / Cruel Somnophage). |
| Vendetta | ✅ ← 🟡 | "Lose life equal to its toughness" now reads the actual toughness via `Value::ToughnessOf(Target(0))` — `LoseLife` resolves *first* in the Seq so the target is still on bf when its toughness is read; then `Destroy` fires. Same pattern Swords to Plowshares uses for `PowerOf(Target)`. |
| Kolaghan's Command | ✅ ← 🟡 | "Choose two" now wires via `Effect::ChooseModes { count: 2, up_to: false, allow_duplicates: false }` — same primitive the STX Commands use (push XXXVI). AutoDecider picks modes 0+1 (gy-recursion + opp-discard). |
| Visions of Beyond | ✅ ← 🟡 | "If a graveyard has 20+ cards" gate now wires via `Predicate::Any([ValueAtLeast(GraveyardSizeOf(You), 20), ValueAtLeast(GraveyardSizeOf(EachOpponent), 20)])` inside `Value::IfPredicate` — flips the draw amount from 1 → 3 when any graveyard crosses the threshold. |
| Skullcrack | ✅ ← 🟡 | "Players can't gain life this turn" rider now wires via the new `Effect::PreventLifegainThisTurn` primitive — sets a per-player sticky flag that `Effect::GainLife` checks before applying any life delta. Cleared at the player's next untap (CR 615 sticky-shield). |
| Lava Coil | ✅ ← 🟡 | "Exile if it would die this turn" rider now wires via `Effect::If` over `Predicate::ValueAtMost(ToughnessOf(Target), 4)` — toughness ≤ 4 → exile path; otherwise damage path. Combat-correct in the typical "Lava Coil hits a fresh creature" case. |
| Magma Spray | ✅ ← 🟡 | Same `Effect::If`-on-toughness pattern as Lava Coil but at the 2-damage threshold. |
| Fiery Confluence | ✅ ← 🟡 | "Choose three. You may choose the same mode more than once" now wires via `Effect::ChooseModes { count: 3, up_to: false, allow_duplicates: true }` — same primitive Moment of Reckoning uses. |
| Searing Blood | ✅ ← 🟡 | "If it dies this turn, deal 3 to its controller" rider now wires via `Effect::If` over `Predicate::ValueAtMost(ToughnessOf(Target), 2)` — toughness ≤ 2 → controller takes 3 damage *first*, then the 2 damage to the creature; otherwise just the 2 damage. Same controller-while-still-on-bf pattern as Vendetta. |

### Server view: `PlayerView.distinct_card_types_in_graveyard`

`u32` field that mirrors the new
`distinct_card_types_in_graveyard(seat)` engine helper. Surfaced so
clients can render "Delirium active" hints (for DRC, Unholy Heat,
future MH2 Delirium payoffs) and display the current count out of 4.
Defaulted via `#[serde(default)]` for back-compat with older
serialized views.

### CR 121 audit (Drawing a Card)

Rule-by-rule status, per `MagicCompRules_20260417.txt` lines 1132-1166:

- **121.1** (draw = top of library to hand) ✅ — `Effect::Draw` and the
  turn-based draw step in `stack.rs::TurnStep::Draw` both call
  `Player::draw_top` which removes from `library[0]` (top) and pushes
  onto `hand`.
- **121.2** (cards drawn one at a time) ✅ — `Effect::Draw` evaluates
  `amount` then loops `for _ in 0..n` issuing `draw_top` per
  iteration. Each draw emits a separate `GameEvent::CardDrawn`.
- **121.2a** (replacement effects modify the count before draw) 🟡 —
  the engine has no draw-replacement primitive yet (no Dredge, no
  Sylvan Library, no "draw N → mill instead"). The `amount` is fixed
  at evaluation time.
- **121.2b** ("can't draw more than one card each turn" applies per
  individual draw) ⏳ — no per-turn-draw-cap static yet (Dauthi
  Voidwalker's downside, Underworld Dreams sibling).
- **121.2c** (multi-player draws: AP first, then NAP) 🟡 — partial.
  Single-target draws (`Selector::You`) trivially APNAP-correct; multi-
  player draws over `EachPlayer` resolve in seat-index order which
  matches APNAP when the active player is seat 0 but breaks down on
  later turns. No per-effect APNAP re-ordering primitive.
- **121.2d** (Two-Headed Giant team-draws) — N/A: engine is
  duel-only; no team-turn option.
- **121.3** (empty-library draw still happens if "may") 🟡 — the
  "may" optionality is wired via `Effect::MayDo`, but the empty-
  library check in `Effect::Draw` causes elimination on the first
  draw (per CR 121.4) regardless of "may". The "you can choose to
  draw 0 from an empty library" subtlety isn't tracked.
- **121.4** (deck-out → loss) ✅ — `Effect::Draw` sets
  `players[p].eliminated = true` on empty-library draw. The SBA pass
  in `check_state_based_actions` handles the actual game-over (CR
  704 cross-reference is faithful).
- **121.5** (zone moves without "draw" don't trigger draw triggers) ✅
  — `Effect::Move { to: Hand(_) }` emits `GameEvent::CardEntersHand`
  (or similar), distinct from `GameEvent::CardDrawn`. Sheoldred's
  CardDrawn trigger only fires from `Effect::Draw` paths. Stitcher's
  Supplier and similar mill effects use `Effect::Mill` or
  `Effect::Move`, also distinct. `Effect::Search { to: Hand }` and
  reveal-then-take-to-hand effects (RevealUntilFind, Telling Time,
  Sleight of Hand approximations) likewise emit CardDrawn-distinct
  events.
- **121.6** (replacement effects on draws) ⏳ — see 121.2a; no
  draw-replacement primitive yet.
- **121.6a-c** (replacement details, replacements interrupting
  sequences, replacement-then-additional-action) ⏳ — same gap.
- **121.7** (replacement/prevention causing further draws) ⏳ — same
  gap as 121.6.
- **121.8** (draw mid-cast → face-down until cast) ⏳ — no face-down
  in-hand state. Spell casts that draw cards (Manamorphose-style
  cantrips) all draw at resolution, not mid-cast — so this rule's
  edge case isn't observable in the catalog today.
- **121.9** (look-then-reveal optionality) ⏳ — no per-card "may
  reveal as drawn" prompt; no cards in the catalog use this.

### 6 new tests

- 4 Delirium tests:
  - `unholy_heat_delirium_off_deals_three` — gy with 1 type, deal 3.
  - `unholy_heat_delirium_on_deals_six` — gy with 4 types, deal 6.
  - `dragons_rage_channeler_no_delirium_stays_one_one` — empty gy,
    body stays 1/1 no flying.
  - `dragons_rage_channeler_delirium_three_three_flying` — gy with 4
    types, body becomes 3/3 with flying.
- 1 Vendetta scaling test:
  - `vendetta_loses_six_life_on_six_toughness_target` — 6/6 target
    drains 6 life (vs the prior flat 2).
- 1 server view test:
  - `player_view_surfaces_distinct_card_types_in_graveyard` —
    `PlayerView.distinct_card_types_in_graveyard` reflects gy
    distinct-type count.

The Kolaghan's Command promotion was an in-place rename of the
existing `kolaghans_command_has_four_modes_chooses_one` test to
`..._chooses_two`, with the body upgraded to assert the
`ChooseModes { count: 2, up_to: false, allow_duplicates: false }`
shape rather than the old `ChooseMode(modes)` shape.

## 2026-05-04 push XLVI: 16 new modern cards + AnotherOfYours ETB de-dup + CR 605 audit

Adds 16 new Modern-supplement card factories (`catalog::sets::decks::
modern`) including Toxic Deluge (X-life sweeper), Supreme Verdict,
Hangarback Walker, Soul Warden, and the four classic color-blasts
(Pyroblast / Red Elemental Blast / Hydroblast / Blue Elemental Blast).
Fixes a longstanding double-fire bug in the `AnotherOfYours`
ETB-trigger dispatch (the generic event-matching walker and the
hardcoded `stack.rs` ETB path were both pushing the trigger, doubling
lifegain on Soul Warden / Felisa, Fang of Silverquill style cards).
Tests at 1468 (was 1445; +23 net).

### Engine fix: `AnotherOfYours` ETB de-dup

`game/mod.rs::is_event_hardcoded` now skips
`PermanentEntered + AnotherOfYours` triggers in the generic
event-matching walker. Previously the trigger was processed in both:

1. The hardcoded `stack.rs` ETB path (line 389-433), which walks the
   battlefield for AnotherOfYours triggers when a creature enters,
   filtering by `c.controller == caster` and pushing per-listener
   triggers with the entering creature as the subject.
2. The generic `event_matches_spec` walker in `game/mod.rs::fire_
   triggers_for_events`, which would re-fire the same trigger from
   the broadcast event stream.

Net effect: Soul Warden gained 2 life per creature ETB instead of 1,
Felisa Fang dropped two Inkling tokens per died-with-counter creature,
Pestbrood Sloth's death trigger fired twice. The fix mirrors the
existing `SelfSource` skip — both ETB scopes (SelfSource for "this
creature enters" + AnotherOfYours for "another creature you control
enters") are owned by the hardcoded path.

### New Modern cards (12, all in `catalog::sets::decks::modern`)

- **Toxic Deluge** ({X}{2}{B} Sorcery) ✅ — Pay X life as additional
  cost; all creatures get -X/-X EOT. Wired via `additional_life_cost
  = Some(XFromCost)` (push XLIII primitive) + `ForEach + PumpPT(-X,
  -X, EOT)` body. At X=2 wipes 2/2s; at X=4 wipes most modern threats.
  Same family as Vicious Rivalry but on a sweeper rather than mass-
  destroy.
- **Supreme Verdict** ({1}{W}{W}{U} Sorcery) 🟡 — Destroy all
  creatures. The "can't be countered" rider stays gap (no per-spell
  counter-prevention primitive). Body identical to Wrath of God.
- **Devour Flesh** ({1}{B} Sorcery) 🟡 — Target player sacrifices a
  creature. The "loses life equal to its toughness" rider is omitted
  (no `OfSacrificed` value primitive); sacrifice half wired faithfully
  via `Effect::Sacrifice` against `Target(0)`.
- **Brought Back** ({W}{W} Instant) 🟡 — Return up to two permanent
  cards from your graveyard to the battlefield. Approximated as
  generic graveyard pick (engine has no per-turn "left bf this turn"
  predicate yet); body uses `Selector::take(_, 2) → Battlefield`.
- **Persist** ({1}{B}{G} Sorcery) 🟡 — Cheap reanimate at the {1}{B}
  {G} rate. The "enters with -1/-1 counter" rider is omitted (engine
  has no Move-to-bf-with-counters replacement primitive — same family
  gap as `enters_with_counters` for tokens, see TODO.md push XL).
- **Selfless Spirit** ({1}{W} 2/1 Spirit Flying) ✅ — Sacrifice
  activation grants Indestructible EOT to every creature you control.
  Body is a `sac_cost: true` activated ability with `ForEach +
  GrantKeyword` over `EachPermanent(Creature ∧ ControlledByYou)`.
- **Hangarback Walker** ({X}{X} 0/0 Construct) ✅ — Enters with X
  +1/+1 counters via `enters_with_counters = Some((PlusOnePlusOne,
  XFromCost))`. On death, mints 1 Thopter token for each +1/+1
  counter via `Value::CountersOn(SelfSource, +1/+1)` (counters
  persist on zone-out per push XVI). Adds new `Thopter` creature
  subtype.
- **Utter End** ({2}{W}{B} Instant) ✅ — Universal exile of any
  nonland permanent at instant speed. `Effect::Move(target Nonland
  → Exile)`.
- **Vraska's Contempt** ({3}{B} Instant) ✅ — Exile target
  creature/PW + caster gains 2 life.
- **Cut Down** ({B} Instant) ✅ — Destroy creature with mana value
  ≤ 3. Cheap modern removal.
- **Stitcher's Supplier** ({B} 1/1 Zombie) ✅ — Mills 3 on both ETB
  and on death. Validates the AnotherOfYours fix end-to-end (without
  the de-dup, the death-mill would have fired twice).
- **Soul Warden** ({W} 1/1 Cleric) ✅ — Whenever another creature
  enters, gain 1 life. Wired via `EntersBattlefield/AnotherOfYours`
  + `EntityMatches { what: TriggerSource, filter: Creature }` filter.
  Validates the AnotherOfYours fix — without the de-dup, Soul Warden
  gained 2 life per ETB instead of 1.

### CR 605 audit (Mana Abilities)

Per-rule status (`rules 605.1–605.5`):

- **605.1** (mana ability definition) ✅ — `actions.rs::is_mana_
  ability` walks the effect tree, returning true iff every step is
  `AddMana` (with optional `Tap` cost). Matches "doesn't require a
  target, could add mana to a player's pool, isn't a loyalty
  ability."
- **605.1a** (activated mana ability criteria) ✅ — same helper.
  Loyalty abilities are stored separately on `CardDefinition.
  loyalty_abilities`, never mistaken for mana abilities.
- **605.1b** (triggered mana ability criteria) 🟡 — engine has no
  triggered mana abilities yet. Cards like Mana Reflection's
  resolution trigger ("whenever a player taps a land for mana, that
  player adds one mana of any type that land produced") would need
  a new event kind. No such cards in the catalog.
- **605.2** (mana ability stays a mana ability even when game state
  blocks it) ✅ — `is_mana_ability` is purely structural; doesn't
  check whether the ability could currently produce mana.
- **605.3a** (can be activated mid-cast/mid-resolve) ✅ — the
  payment loop in `actions.rs::pay_mana_cost` calls back into
  `auto_tap_mana_for_color` which can fire mana abilities of any
  un-tapped lands the controller has, even mid-cast.
- **605.3b** (mana abilities don't go on the stack) ✅ — `actions.
  rs::activate_ability` short-circuits to
  `continue_ability_resolution` for mana abilities, bypassing the
  stack push and priority reset.
- **605.3c** (single-shot per activation) ✅ — the immediate-resolve
  path means there's no window for double-activation; no special
  flag needed.
- **605.4 / 605.4a** (triggered mana abilities don't go on stack) 🟡
  — see 605.1b. No catalog card exercises this path; the dispatch
  would be a one-liner in `fire_triggers_for_events` if needed.
- **605.5a** (targeted abilities aren't mana abilities) ✅ —
  `is_mana_ability` returns false for any effect with a target slot;
  helper walks `Selector::Target` references via `effect_has_target`
  reachability.
- **605.5b** (spells aren't mana abilities) ✅ — spells route through
  `cast_spell`, never `activate_ability`; the mana ability check is
  structurally unreachable from the spell path.

### Color-blast quad (Pyroblast / REB / Hydroblast / BEB)

Four classic Old-School / cube color-hate one-mana modal counters /
removal — Pyroblast and Red Elemental Blast (functional reprints) at
{R} hate blue; Hydroblast and Blue Elemental Blast at {U} hate red.
Each is an `Effect::ChooseMode` over `[CounterSpell, Destroy]`,
both modes filtered by `HasColor(Blue)` / `HasColor(Red)`. AutoDecider
picks mode 0 (counter) by default. Counter mode filter is
`IsSpellOnStack ∧ HasColor(Blue/Red)`; destroy mode targets
`Permanent ∧ HasColor(Blue/Red)`.

### Tests (+23 net, 1445 → 1468)

- 19 new tests for the 16 new cards (Toxic Deluge ×2 — kills small +
  X=0 noop; Supreme Verdict; Devour Flesh; Brought Back; Persist ×2
  — reanimate + no-eligible noop; Selfless Spirit ×2 — activation +
  body shape; Hangarback Walker — enters-with-counters + on-death
  Thopter mint; Utter End; Vraska's Contempt; Cut Down ×2 — accept
  + reject high-MV; Stitcher's Supplier ×2 — ETB + on-death; Soul
  Warden ×2 — fires on opp creature + does NOT fire on self ETB;
  Pyroblast ×2 — counter + destroy modes; Hydroblast — counter mode;
  Red Elemental Blast — destroy mode; Blue Elemental Blast — destroy
  mode).
- All 1468 tests pass — including the 17 existing AnotherOfYours-
  scoped triggered abilities (Felisa Fang, Pestbrood Sloth, Arnyn,
  Augusta, Sparring Regimen, etc.) which previously were silently
  double-firing.

## 2026-05-04 push XLV: 8 new modern cards + 2 SOS promotions + bot life-cost preflight + CR 120 audit

Adds 8 new Modern-supplement card factories (`catalog::sets::decks::
modern`), promotes 2 SOS instants from 🟡 to ✅ via existing
primitives (no new engine code on the promote path), and wires the
bot's `can_afford_in_state` to pre-flight reject `additional_life_
cost` casts that would crash the controller below 0 life. Tests at
1445 (was 1430; +15 net), all green.

### Engine improvement: bot pre-flight for `additional_life_cost`

`server/bot.rs::can_afford_in_state` now rejects spells whose
cast-time life cost would crash the controller below 1 life.
Mirrors the existing `additional_sac_cost` (push XXXIX) and
`additional_discard_cost` (push XLIII) pre-flight rejections. The
check builds a temporary `EffectContext` with `x_value` set to the
bot's `max_affordable_x` so X-cost life payments (Vicious Rivalry's
`Value::XFromCost`) evaluate against the upper-bound the bot would
actually pump to. Conservative — CR 119.4 says the cast itself is
legal at exactly your life total (the loss-of-game SBA fires later);
the bot still skips that suicide line because the SBA pass would
end the game in a loss. Unblocks Vicious Rivalry from the bot's
deadlock-prone "always cast at high X" line.

### New Modern cards (8, all in `catalog::sets::decks::modern`)

- **Abrade** ({1}{R} Instant) ✅ — Modal: 3 dmg to creature, OR
  destroy artifact. Same `Effect::ChooseMode` shape as Mage Hunters'
  Onslaught (✅ STX). AutoDecider picks the first mode with a legal
  target (creature first); ScriptedDecider exercises both.
- **Izzet Charm** ({U}{R} Instant) ✅ — Three-mode `ChooseMode`:
  counter unless {2} (`CounterUnlessPaid`); 2 dmg to creature; loot 2
  (Draw 2 + Discard 2). Mode 0 filter is `IsSpellOnStack ∧
  ¬HasCardType(Creature)` for the printed "noncreature spell" scope.
- **Pillar of Flame** ({R} Sorcery) 🟡 — 2 dmg to any target via
  `any_target()`. The "exile if it would die this turn" rider is
  omitted (no damage-replacement → exile primitive). The 2-damage
  half kills any X/2.
- **Smash to Smithereens** ({1}{R} Instant) ✅ — Destroy artifact +
  3 dmg to its controller. Threads through `PlayerRef::ControllerOf
  (Target(0))` for the printed "to that artifact's controller" half.
- **Forked Bolt** ({R} Sorcery) 🟡 — 2 dmg to any target. Printed
  "divided as you choose among one or two targets" collapses to
  single-shot (no divided-damage primitive — same gap as Magma Opus).
- **Knight of Meadowgrain** ({W}{W} 2/2 Kithkin Knight) ✅ — Vanilla
  `Keyword::FirstStrike + Keyword::Lifelink` body.
- **Defiant Strike** ({W} Instant) ✅ — Combat trick + cantrip
  (`Seq([PumpPT(+1/+0, EOT), Draw 1])`).
- **Fanatical Firebrand** ({R} 1/1 Goblin Pirate) ✅ — Haste body +
  `{T}, Sac: 1 dmg to any target` activation. Uses `sac_cost: true`
  (same plumbing as Shattered Acolyte's destroy-on-sac).

### SOS promotions (2 — both 🟡 → ✅ via existing primitives)

- **Brush Off** ({2}{U}{U} Instant) — "Costs {1}{U} less if it
  targets an instant or sorcery spell." Wires the cost reduction
  via `StaticEffect::CostReductionTargeting` on the cast card itself
  (self-static, same shape as Ajani's Response). Target filter is
  `IsSpellOnStack ∧ (HasCardType(Instant) ∨ HasCardType(Sorcery))`.
  At {U}{U} the spell is effectively a Mana-Leak-rate hard counter
  for IS spells. The {1}{U} → {2} generic discount approximation
  loses the colored-pip refund (the engine's discount primitive
  is generic-only); the {U}{U} colored-pip floor stays put.
- **Run Behind** ({3}{U} Instant) — "Costs {1} less if it targets an
  attacking creature." Same `CostReductionTargeting` shape as Brush
  Off; target filter is `Creature ∧ IsAttacking`. At {2}{U} when
  pointed at an attacker, the spell becomes a 3-mana
  bounce-to-bottom — strictly Vraska's Contempt-rate permanent
  removal.

### CR 120 audit (Damage)

Today's audit references CR 120 (Damage), the section directly
governing `Effect::DealDamage`, the `LifeLost` event chain, and the
prevention shield primitives. Per-rule status:

- 120.1 (objects can deal damage to battles, creatures, walkers,
  players) ✅ — `Effect::DealDamage { to: Selector }` resolves
  against any of those targets via `evaluate_requirement_static`.
- 120.2 (any object can deal damage; 120.2a combat, 120.2b spell/
  ability) ✅ — `resolve_combat_damage_with_filter` for combat,
  `Effect::DealDamage` for spell/ability.
- 120.3a (damage to player → that player loses life) ✅ —
  `LifeLost` event fires before the SBA pass; lifelink riders ride
  on the same path.
- 120.3b (damage to player by source with infect → poison counters)
  🟡 — `Keyword::Infect` exists; the engine routes infect damage
  through `AddPoison` but the predicate `HasKeyword(Infect)`
  enforcement on damage events is partial (combat-side only; spell-
  source infect is wired but less heavily tested).
- 120.3c (damage to planeswalker → loyalty counters removed) ✅ —
  `EntityRef::Planeswalker` damage decrements loyalty; SBA destroys
  at 0.
- 120.3d (wither/infect → -1/-1 counters on creature) 🟡 — `Keyword::
  Wither` exists but is not currently routed through `Effect::
  DealDamage` to creatures (creatures take regular damage marks
  instead of -1/-1 counters from wither sources).
- 120.3e (damage marks on creatures) ✅ — `card.damage_marked` set
  by `deal_damage`; cleared at cleanup per 120.6.
- 120.3f (lifelink → controller gains that much life) ✅ —
  `Keyword::Lifelink` triggers a `GainLife` event in the same
  effect resolution.
- 120.3g (toxic combat damage → poison counters) ⏳ — Toxic keyword
  is not yet a first-class engine concept (no `Keyword::Toxic` yet).
- 120.3h (damage to battle → defense counters) ⏳ — Battle card type
  is not yet modeled (engine has `CardType::Battle` placeholder
  only; combat damage to battles is not wired).
- 120.4 (4-part processing: trample-excess, prevent/replace, results,
  event) 🟡 — partial:
  - 120.4a (trample-excess routing) 🟡 — Push XLI's
    `combat_damage_prevented_this_turn` flag short-circuits combat
    damage uniformly; trample's "excess to defending player" is
    handled in `resolve_combat_damage_with_filter` for normal combat
    but doesn't run through the replacement-effect layer.
  - 120.4b (prevention/replacement effects) 🟡 — Push XLI's
    `Effect::PreventCombatDamageThisTurn` is the only prevention
    primitive; spell-prevention layers (Awe Strike, Ethereal Haze)
    are still ⏳.
  - 120.4c (damage → results processing) ✅ — life loss / counter
    add / damage marks all resolve in a deterministic order.
  - 120.4d (final event) ✅ — emitted via `GameEvent::DamageDealt`
    (and combat-specific `CombatDamage`), feeding triggered ability
    listeners.
- 120.5 (damage doesn't destroy — SBAs do) ✅ — destruction is a
  separate SBA pass that walks `damage_marked >= toughness`.
- 120.6 (damage marked persists till cleanup) ✅ — cleared in
  `cleanup_step`.
- 120.7 (source of damage = object that dealt it) ✅ —
  `EffectContext.source` carries the source through the effect tree.
- 120.8 (0-damage doesn't deal damage; no triggers) 🟡 — partial;
  damage events with amount 0 are emitted in some paths and skipped
  in others (no unified gate). Most damage triggers gate on
  `ValueAtLeast(amount, 1)` to avoid spurious fires.
- 120.9 (specific-source "damage dealt" wording) 🟡 — partial; the
  engine doesn't track per-source damage tally separately, so
  triggers that read "the damage dealt" pull from the just-emitted
  event's amount field.
- 120.10 (excess-damage triggers) ⏳ — no excess-damage tracking
  primitive yet.

### Tests (+15 net, 1430 → 1445)

- 11 new modern card tests covering Abrade modes 0/1, Izzet Charm
  modes 1/2, Pillar of Flame, Smash to Smithereens, Forked Bolt,
  Knight of Meadowgrain definition, Defiant Strike pump+draw,
  Fanatical Firebrand sac-ping + definition.
- 2 new SOS promotion tests (Brush Off cost reduction when
  targeting an IS spell; Run Behind cost reduction when targeting
  an attacker).
- 2 new bot pre-flight tests (Vicious Rivalry rejected at low life;
  accepted with comfortable buffer).

## 2026-05-04 push XLIV: 10 new cards + AnyPlayer-spell-cast-trigger fix + ControllerOf(Card) fix

Adds 10 new cards (3 STX 2021 + 7 modern), fixes two engine bugs in
the trigger dispatch, and surfaces `additional_life_cost` in the
`KnownCard` view label. Tests at 1430 (was 1415; +15 net), all green.

### Engine fixes

- **`fire_spell_cast_triggers` AnyPlayer arm** — the `EventScope::
  AnyPlayer` arm of the SpellCast trigger filter previously
  collapsed onto the `YourControl` arm (`c.controller == caster`),
  meaning Eidolon-style "whenever a player casts X, …" triggers
  *never* fired on opponent casts. The fix unconditionally accepts
  matches when the trigger source's scope is `AnyPlayer` — the
  controller can be anyone. Pre-fix Eidolon of the Great Revel
  would only have pinged its own controller's cheap casts; post-
  fix the symmetric Modern-Burn punisher fires correctly on both
  players' casts.
- **`PlayerRef::ControllerOf(Selector)` for `EntityRef::Card`** —
  the player-resolution path for `ControllerOf(sel)` only handled
  `EntityRef::Permanent`, returning `None` when the inner selector
  resolved to `EntityRef::Card` (stack-resident spells, the typical
  shape for `Selector::TriggerSource` in a SpellCast trigger). The
  fix walks the stack to find the `StackItem::Spell` matching the
  card id and returns the `caster` field; falls back to battlefield
  + owner. Together with the AnyPlayer fix, this lets Eidolon's
  body `DealDamage { to: Player(ControllerOf(TriggerSource)), ... }`
  resolve to the spell's caster.

### New cards (10)

**STX 2021 (3 new ✅, all in `catalog::sets::stx::mono`):**

- **Archmage Emeritus** ({2}{U}{U}, 3/3 Human Wizard) — Magecraft:
  draw a card. Pure draw-engine body wired via `magecraft(Effect::
  Draw 1)`. Universally good in any spellslinger / Strixhaven
  shell.
- **Fortifying Draught** ({2}{W} Instant — Lesson) — Gain 4 life,
  scry 2. Mono-white Lesson cantrip-style life buffer wired as
  `Seq([GainLife 4, Scry 2])`.
- **Sage of Mysteries** ({U}, 1/2 Spirit Wizard) — Magecraft: target
  opponent mills 2. Wired via `magecraft(Effect::Mill { who: Each
  Opponent, amount: 2 })`. With one opponent the EachOpponent
  collapse matches printed; in 2-player it's exact.

**Modern (7 new, in `catalog::sets::decks::modern`):**

- **Serum Visions** ({U} Sorcery) ✅ — Draw a card, then scry 2.
  Classic Modern blue cantrip; the printed *draw-then-scry* order
  matters (lets Scry 2 process the freshly drawn card).
- **Burst Lightning** ({R} Instant) 🟡 — Base mode 2 damage to any
  target via `Effect::DealDamage` on `any_target()`. Kicker {4}
  4-damage upgrade omitted (alt-cost-implies-mode primitive gap —
  same family as Devastating Mastery's Mastery cost).
- **Roiling Vortex** ({R} Enchantment) 🟡 — Upkeep ping wired via
  `EventKind::StepBegins(Upkeep) + EventScope::AnyPlayer`. Sac
  activation `{1}{R}, Sacrifice this: 4 damage to any target`
  uses `sac_cost: true`. The "players can't gain life" continuous
  lock is omitted (no global lifegain-replacement static).
- **Murderous Cut** ({4}{B} Instant) 🟡 — Destroy target creature
  at full {4}{B}. Delve cost reduction omitted (same gap as
  Treasure Cruise / Dig Through Time).
- **Eidolon of the Great Revel** ({R}{R}, 2/2 Spirit) ✅ — Symmetric
  "Burn punisher" via `EventKind::SpellCast + EventScope::AnyPlayer`
  + `Predicate::EntityMatches { what: TriggerSource, filter:
  ManaValueAtMost(3) }`. Damage routes through `Selector::Player(
  PlayerRef::ControllerOf(TriggerSource))` to find the cast-spell's
  caster on the stack. Validates the two engine fixes end-to-end —
  pre-fix Eidolon was completely non-functional (the trigger never
  fired and its body had no resolution path).
- **Wild Slash** ({R} Instant) ✅ — 2 damage to any target. The
  Spell Mastery damage-can't-be-prevented rider is a no-op
  gameplay-wise (engine has no prevention layer that could fight
  the rider; the 2-damage half is unconditional anyway).
- **Krenko, Mob Boss** ({2}{R}{R}, 3/3 Legendary Goblin Warrior) ✅
  — `{T}: CreateToken(Goblin, X)` with X = your Goblin count.
  Exponential Goblin tribal blowout (1 → 2 → 4 → 8 → ...).

### Server view: additional_life_cost label

`KnownCard.additional_cost_label` now also surfaces
`additional_life_cost` (push XLIII primitive). Combines with sac /
discard labels via " and " when multiple are present. Recognised
shapes: `Const(N)` → "Pay N life", `XFromCost` → "Pay X life",
`ConvergedValue` → "Pay life equal to converge". Vicious Rivalry's
{X}-life cost now renders as "Pay X life" pre-cast — surfacing the
hidden cost so clients can warn before letting the player into a
spell that would crash them below 0 life.

### CR 603 audit (Handling Triggered Abilities)

Today's audit references CR 603 (Handling Triggered Abilities), the
section directly governing the AnyPlayer + ControllerOf fixes:

- 603.1 (trigger condition + effect shape) ✅ — `TriggeredAbility {
  event: EventSpec, effect: Effect }`.
- 603.2 (auto-trigger on event) ✅ — `fire_*_triggers` walks
  battlefield permanents on each event family (SpellCast, ETB,
  Dies, LifeGained, etc.).
- 603.2b (step-begin triggers) ✅ — `fire_step_triggers(step)`
  fires upkeep/draw/end-step trigger families.
- 603.2c (one trigger per event) ✅.
- 603.2g (no trigger on prevented events) 🟡 — partial;
  prevention-shielded combat damage events skip per the new
  `Effect::PreventCombatDamageThisTurn` primitive (push XLI),
  but spell prevention layers and replacement effects more
  generally don't have a unified pre-event check yet.
- 603.3a (controller is who controlled source at trigger time) ✅
  — Push XLIV's `c.controller` capture in `fire_spell_cast_
  triggers` keeps the source-controller binding correct even
  through control-change effects.
- 603.3b (APNAP trigger ordering) 🟡 — partial; multi-trigger
  ordering follows insertion order rather than a true APNAP +
  re-stage protocol.
- 603.4 (intervening 'if' clause) ✅ — `EventSpec.filter` is
  re-evaluated at trigger time; `Predicate::EntityMatches`,
  `ValueAtLeast`, `CastFromGraveyard` all participate.
- 603.5 ("may" optionals) ✅ — `Effect::MayDo` / `MayPay`.
- 603.6 (zone-change triggers) ✅ — `EventKind::CreatureDied`,
  `EntersBattlefield`, `LeavesBattlefield`, `CardDrawn`, etc.
- 603.6d ("[This] enters with N counters" as static) ✅ —
  `enters_with_counters` field (push XL).
- 603.7 (delayed triggered abilities) ✅ — `Effect::DelayUntil`.
- 603.8 (state triggers) 🟡 — partial; LifeOf-based "if you have
  ≤ 0 life" continuous checks fire via SBAs, but the engine
  doesn't model state triggers as a separate dispatch class.
- 603.10 (look-back-in-time triggers) 🟡 — partial; LeavesBattle
  field triggers preserve the source's pre-zone-change state via
  `EventKind::CreatureDied/SelfSource`, but the more exotic
  look-back cases (counter-spell triggers, lose-game triggers)
  aren't all wired.

### Tests (+15 net, 1415 → 1430)

- 9 STX/Modern card tests: Archmage Emeritus draw + body, Fortifying
  Draught life+scry, Sage of Mysteries mill + body, Serum Visions
  draw+scry, Burst Lightning damage, Roiling Vortex upkeep ping +
  sac activation, Murderous Cut destroy, Eidolon symmetric ping +
  body, Wild Slash damage, Krenko Goblin doubling.
- 1 view test: Vicious Rivalry's `additional_life_cost` renders as
  "Pay X life".
- 5 engine-bug-fix safety nets fall out implicitly via the Eidolon
  flow (the trigger now fires + the ControllerOf resolution lands
  on the caster).

## 2026-05-04 push XLIII: cast-time additional-cost primitives + 10 promotions

Adds two new sister cast-time additional-cost primitives —
`additional_discard_cost: Option<u32>` and `additional_life_cost:
Option<Value>` — to round out the cast-time additional-cost trio
started by push XXXIX's `additional_sac_cost`. Net: ten 🟡 cards
across the catalog promote to ✅, with cast-time deductions paid
*before* the spell goes on the stack (matching CR 118.8 / 601.2f
"additional costs are paid at cast time, before the spell hits the
stack"). Tests at 1415 (was 1405; +10 net), all green.

### Engine primitives

- **`CardDefinition.additional_discard_cost: Option<u32>`** — number
  of cards the controller must discard as an additional cast-time
  cost. Sister to `additional_sac_cost` (push XXXIX). The pre-flight
  check rejects with `SelectionRequirementViolated` when the
  controller has fewer than N cards in hand at cast time. The
  discard happens after mana is paid but before the spell goes on
  the stack — so madness / discard-trigger listeners (Putrid Imp,
  Faithless Looting derivatives) fire pre-resolution. Auto-pick is
  the first N cards in hand (matches the random-discard branch in
  `Effect::Discard`).
- **`CardDefinition.additional_life_cost: Option<Value>`** — life
  cost the controller pays as an additional cast-time cost. The
  `Value` is evaluated against the cast-time `EffectContext` (X is
  read from the spell's `{X}` pip), then the controller's life is
  debited. The deduction emits a `LifeLost` event so "whenever you
  lose life" listeners (Lifelink-payoff, pain-land derivatives)
  fire pre-resolution. Loss-of-game from crossing zero life is left
  to the next SBA pass per CR 119.4.
- Both fields default to `None` via `#[serde(default)]` for snapshot
  back-compat with older serialized states.
- Cast-pipeline integration: the bot's `can_afford_in_state`
  `would_accept` walker pre-flight-rejects unaffordable casts (no
  card to discard / no creature to sac) so the same gate that fires
  in `cast_spell` is mirrored on the heuristic side.
- View-side rendering: `KnownCard.additional_cost_label` now
  combines sac + discard labels with " and " when both are present.
  "Discard a card" / "Discard 2 cards" pluralisation uses the bare
  count.

### Promotions (10 cards: 7 🟡 → ✅, 3 fidelity bumps)

- **Thrilling Discovery** ({1}{U}{R}, Strixhaven instant) 🟡 → ✅ —
  printed "additional cost: discard a card" now paid at cast time
  via `additional_discard_cost: Some(1)`. Body is just the gain-2 +
  draw-2 halves.
- **Cathartic Reunion** ({1}{R}, modern sorcery) → ✅ — printed
  "additional cost: discard two cards" via
  `additional_discard_cost: Some(2)`. Body is the draw-3.
- **Necrotic Fumes** ({1}{B}{B}, Strixhaven sorcery) 🟡 → ✅ — moves
  the printed sacrifice from in-resolution to cast-time via the
  existing `additional_sac_cost` (push XXXIX). Body is the targeted
  exile.
- **Tormenting Voice / Wild Guess / Thrill of Possibility** (modern
  red rummagers) → ✅ — printed "additional cost: discard a card"
  → `additional_discard_cost: Some(1)`. Bodies are the draw-2.
- **Big Score** ({3}{R}, modern instant) → ✅ — printed "additional
  cost: discard a card" → `additional_discard_cost: Some(1)`. Body
  is the 2-Treasure mint + draw-2.
- **Crop Rotation** ({G}, modern instant) → ✅ — printed "additional
  cost: sacrifice a land" → `additional_sac_cost: Some(Land &
  ControlledByYou)`. Body is the land tutor.
- **Mine Collapse** ({2}{R}, modern sorcery) → ✅ — printed
  "additional cost: sacrifice a Mountain" → `additional_sac_cost:
  Some(Land & HasLandType(Mountain))`. Body is the targeted 4-damage.
- **Vicious Rivalry** ({X}{2}{B}{G}, SOS sorcery) 🟡 → ✅ — printed
  "additional cost: pay X life" → `additional_life_cost:
  Some(Value::XFromCost)`. Body is the destroy-X-or-less mass
  removal.

### CR 118.8 / 601.2f audit (Additional Costs)

Push XLIII closes the two largest gaps in the additional-cost
primitive family:

- 118.8 (additional cost listed on a spell) ✅ — three sister fields
  `additional_sac_cost` (push XXXIX) / `additional_discard_cost`
  (XLIII) / `additional_life_cost` (XLIII) cover the most common
  shapes (sacrifice / discard / pay life).
- 118.8a (any number of additional costs) 🟡 — the engine fields are
  three independent `Option<...>`s so multiple shapes can stack
  on the same spell, but the cost-payment path doesn't yet thread
  a "pay all of them in any order" decision picker. Today every
  card with a stacked additional cost (e.g. some hypothetical
  "sacrifice a creature and discard a card") would just have all
  three fields populated; the engine pays them in the listed order
  (sac, discard, life).
- 118.8b (optional additional costs) 🟡 — kicker / buyback /
  optional escalation aren't modeled today (no `Option<Cost>` on
  optional vs. mandatory). All catalog `additional_*_cost` are
  mandatory.
- 118.8c ("if able" + hidden zone) — N/A (no opponent-cast-spell
  effects yet).
- 118.8d (mana cost unchanged) ✅ — `additional_*_cost` payments
  don't mutate `card.definition.cost`. `cost_label`-style readers
  still see the printed mana cost.
- 601.2f (total cost determination) ✅ — additional costs are
  applied after mana cost discounts (Killian, hybrid pip choice)
  and surcharges (Damping Sphere). The order matches printed
  semantics.
- 601.2h (pay all costs in any order) ✅ — `cast_spell` pays mana
  first, then sacrifice, then discard, then life. Each step is
  atomic and emits its own event before the spell goes on the
  stack.

### Tests (+10 net, 1405 → 1415)

- 6 new STX tests covering Thrilling Discovery / Cathartic Reunion /
  Necrotic Fumes (cast accept + cast reject paths for each).
- 2 new bot tests: `bot_skips_thrilling_discovery_with_only_one_card`,
  `bot_accepts_thrilling_discovery_with_extra_card`.
- 2 new view tests: discard-1 singular / discard-2 plural label
  rendering.
- Existing `vicious_rivalry_destroys_creatures_at_or_below_x` already
  asserted the post-cast life delta — naturally updated by the new
  primitive without a test edit.

## 2026-05-04 push XLII: 9 STX cards + Plargg fidelity bump

Adds 9 new STX 2021 mono-color cards and brings Plargg's second
activation online — closes one of the long-standing "look-at-top +
exile" gaps in the Lorehold catalog. Tests at 1403 (was 1389; +14
net), all green.

**New STX 2021 cards (8 ✅ + 1 🟡):**

- **Quick Study** ({1}{U}, Sorcery — Lesson) — `Effect::Draw(2)`.
  Mono-blue Lesson cantrip (functional twin of Divination) at the
  printed common rate.
- **Introduction to Prophecy** ({3}{U}, Sorcery — Lesson) — `Scry 4 +
  Draw 1`. Blue Lesson card-selection rare.
- **Introduction to Annihilation** ({3}{R}, Sorcery — Lesson) —
  Universal exile (any permanent) + the *target's controller* draws 1.
  Wired via `Selector::Player(ControllerOf(Target(0)))` reading the
  cast-time target's `controller` field, which is preserved post-exile
  by `move_card_to`.
- **Soothsayer Adept** ({1}{U}, 1/2 Merfolk Wizard) — `{U}: Scry 1`
  repeatable card-selection. Same shape as Hedron Crab's repeatable
  mill body.
- **Drainpipe Vermin** ({B}, 1/1 Rat) — Death-trigger
  `EachOpponent → Mill 2`. Witherbloom self-sacrifice / "leaves
  graveyard" enabler at the printed common rate.
- **Make Your Move** ({B}{G}, Instant) — "Choose one or both"
  destroy-tapped-creature / destroy-enchantment, wired faithfully via
  `Effect::ChooseModes { count: 2, up_to: true }`. AutoDecider picks
  both modes when both targets exist; ScriptedDecider can flip to
  either single-mode pick.
- **Returned Pastcaller** ({4}{B}, 4/3 Zombie Wizard) — ETB returns a
  MV ≤ 3 IS card from your graveyard to hand via
  `Selector::take(CardsInZone(Graveyard, IS ∧ MV ≤ 3), 1)`.
- **Field Research** ({1}{W}, Sorcery — Lesson) — `+1/+1 counter on
  target creature, then gain 2 life`. White Lesson card.
- **Mage Duel** ({R}, Instant) 🟡 — 2 damage to an opp creature. The
  Magecraft "may pay {R}{R} on the spell itself, copy it" rider stays
  gap (would need a self-spell magecraft trigger that fires *during*
  the same cast — same family gap as Devastating Mastery's Mastery
  alt-cost-on-the-spell-itself).

**Plargg's second activation** ({2}{R}: Look at top 3, exile top 1) —
now wired as `Effect::Seq([LookAtTop(3), Move(TopOfLibrary{1} →
Exile)])`. The auto-decider always exiles the topmost (closest
fidelity without an interactive "may exile one of three" picker — the
value of the activation is the exile, not the abstain). The "may play
that exiled card until end of turn" rider stays gap (same family as
Suspend Aggression / Tablet of Discovery / Outpost Siege /
Conspiracy Theorist). Plargg stays 🟡 overall.

**Doc-only fix**: Practiced Scrollsmith's note now reflects the wired
`{R/W}` hybrid pip (push XL); the prior note still claimed the
hybrid was approximated as `{R}`.

**14 new tests:** Plargg (×2 — exile success, exile no-op on empty
library), Quick Study, Intro to Prophecy, Intro to Annihilation,
Soothsayer Adept, Drainpipe Vermin, Make Your Move (×2 — mode 0
success + reject untapped), Returned Pastcaller (×2 — returns IS
card + no-op on empty gy), Field Research, Mage Duel (×2 — kills
2-toughness opp + rejects friendly target).

## 2026-05-04 push XLI: Effect::PreventCombatDamageThisTurn + 6 cards/promotions

Adds 1 engine primitive (combat-damage prevention shield), 1 new STX
2021 card (Quandrix Biomathematician), 5 promotions/new fog-style
cards (Owlin Shieldmage, Holy Day, Spore Frog, Monastery Swiftspear,
Stormchaser Mage, Faerie Mastermind), and the new STX-base audit
script.

### Engine primitive

- **`Effect::PreventCombatDamageThisTurn`** — "Prevent all combat
  damage that would be dealt this turn" (Holy Day / fog-style
  shield). Sets a `GameState.combat_damage_prevented_this_turn`
  flag, sticky for the rest of the turn. Cleared in `do_cleanup`
  (CR 615 — prevention only applies to *this* turn). Implementation
  is faithful to printed "prevent" semantics: `resolve_combat_damage_
  with_filter` short-circuits the per-attacker damage step when the
  flag is set, so no damage events fire (lifelink, infect, and
  trample-trigger riders all skip too — same shape as a real
  prevention shield, not "deal then zero out"). New
  `GameEvent::CombatDamagePreventedThisTurn` (+ wire mirror) so
  spectator UIs can render the shield activation. Defaulted via
  `#[serde(default)]` for back-compat with older snapshots.

### STX 2021 promotions (1) + new card (1)

- **Owlin Shieldmage** ({3}{W} 2/3 Bird Wizard with Flash + Flying)
  🟡 → ✅. ETB now triggers `Effect::PreventCombatDamageThisTurn`,
  closing the printed "When this creature enters, prevent all combat
  damage that would be dealt this turn" rider end-to-end.
- **Biomathematician** ({1}{G}{U}, 2/2 Vedalken Druid, NEW Quandrix
  uncommon). Death-trigger creates a 0/0 green-and-blue Fractal
  token + ×2 +1/+1 counter stamp via the existing `Selector::Last
  CreatedToken` pattern. Same shape as Pestbrood Sloth's death-
  trigger token-mint, with the counter stamp on top. Closes Quandrix
  (G/U) at 8 ✅ / 0 🟡 — first STX 2021 college with no remaining
  partials.

### Modern catalog cards (4 new + promotions)

- **Holy Day** ({W} Instant) — NEW. Classic Alpha-era fog wired via
  `Effect::PreventCombatDamageThisTurn`.
- **Spore Frog** ({G} 1/1 Frog) — NEW. Sacrifice-as-cost activation
  on a body that puts up the same shield. Validates the primitive
  works through the existing `sac_cost: true` activation path.
- **Monastery Swiftspear** ({R} 1/2 Human Monk, Haste + Prowess) —
  doc-only promotion. Push XXXVIII wired Prowess as a synthetic
  SpellCast trigger; Swiftspear's per-noncreature-cast +1/+1 EOT
  pump now fires correctly. Same wiring shape as Spectacle Mage
  (✅ since push XXXVIII).
- **Stormchaser Mage** ({1}{U}{R} 1/3 Flying + Haste + Prowess) —
  same Prowess promotion.
- **Faerie Mastermind** ({1}{U} 2/1 Faerie Rogue, Flash + Flying)
  🟡 → ✅. The "except the first card they draw each turn" gate
  now wires faithfully via `Predicate::ValueAtLeast(CardsDrawnThis
  Turn(Triggerer), 2)`. Trigger source is bound to the opp who drew;
  the gate resolves on that opp's per-turn draw tally, so a 2nd, 3rd,
  … draw fires the trigger and the 1st draw skips.

### Tooling

- **`scripts/audit_stx_base.py`** — NEW. Sibling to the SOS audit
  script. Walks the "Strixhaven base set (STX)" table at the bottom
  of STRIXHAVEN2.md, cross-references against `catalog::sets::stx/`
  + `decks/modern.rs`, reports false positives (doc says ✅/🟡, no
  catalog string) and false negatives (catalog string but doc says
  ⏳), with a per-section breakdown. Closes the audit-script
  suggestion from TODO.md push XXIX.

### CR 615 audit (Prevention Effects)

The new primitive implements:
- **CR 615.1 / 615.1a** — continuous prevention effects watching for
  damage events with the word "prevent". The flag is checked at
  damage-deal time, not pre-locked.
- **CR 615.4 / 615.6** — prevention exists before damage event;
  prevented damage never happens (no damage event fires, no
  triggers).

Still ⏳:
- **CR 615.7** — specific-amount shields ("Prevent the next 3
  damage that would be dealt to any target this turn"). Needs a
  per-source / per-amount replacement primitive.
- **CR 615.8** — next-instance-from-source shields.
- **CR 615.9** — prevention with property-recheck on the source.
- **CR 615.13** — triggered abilities that fire when damage is
  prevented.

### Tests (+9 net, 1379 → 1389)

- biomathematician_death_creates_fractal_with_two_counters
- biomathematician_is_two_two_vedalken_druid
- owlin_shieldmage_prevents_combat_damage_on_etb
- prevent_combat_damage_clears_on_cleanup
- owlin_shieldmage_full_cast_to_prevention_flow
- monastery_swiftspear_prowess_pumps_on_noncreature_cast
- stormchaser_mage_prowess_pumps_on_noncreature_cast
- faerie_mastermind_skips_first_opp_draw_each_turn (+ updated old
  test to bump cards_drawn_this_turn so Ponder is the 2nd draw)
- holy_day_prevents_combat_damage_this_turn
- spore_frog_sacrifice_prevents_combat_damage

### Cleanup

- 2× `extend(drain(..))` → `append()` in `game/mod.rs` scry/surveil
  zone-redistribution (clippy::extend_with_drain).
- Swarm Shambler doc-string rewrap (clippy::doc_lazy_continuation —
  the wrap on "+ counters" tripped the markdown-list lint).

## 2026-05-04 push XL: Hybrid pip fidelity + `enters_with_counters` primitive

Promotes 2 SOS partials to ✅ + adds 1 new engine primitive that
replaces the prior "base body bumped" approximation across 3 cards.

### Engine primitive

- **`CardDefinition.enters_with_counters: Option<(CounterType, Value)>`**
  — "this permanent enters with N {kind} counters on it" replacement
  effect. Resolved at the cast-time spell-resolution path (`stack.rs`)
  *between* battlefield entry and the ETB-trigger push, so the
  counters land *before* SBAs run. The `Value` is evaluated against
  the cast-time `EffectContext` (the spell's `x_value`,
  `converged_value`, and `targets[]` are in scope), so X-cost
  permanents like Pterafractyl ({X}{G}{U}) read the actual paid X.

  Distinct from an ETB trigger that adds counters via
  `Effect::AddCounter`: ETB triggers fire *after* the permanent is
  on the battlefield, so a 1/0 body would die to the 0-toughness
  SBA before its trigger could resolve. The replacement form wires
  the counters in atomically with bf entry, surviving the post-entry
  SBA pass.

  Honored only on the spell-resolution path; tokens and `Move →
  Battlefield` paths skip this hook (tokens have no X or paid mana
  to reference, and reanimate-style returns shouldn't re-add the
  original counter count). Backed by `#[serde(default)]` for
  back-compat with older serialized snapshots.

### SOS 🟡 → ✅ promotions (2)

- **Lluwen, Exchange Student // Pest Friend** ({2}{B}{G} // {B/G})
  — back-face Pest Friend's `{B/G}` hybrid pip now wired exactly
  via `ManaSymbol::Hybrid(Black, Green)`. The previous `{B}`-only
  approximation forced black mana for what should be either color.
  Pest Friend is now castable from a {G}-only pool (or {B}-only,
  or any mix). Closes the Witherbloom (B/G) school's last 🟡 row.

- **Pterafractyl** ({X}{G}{U}, 1/0 Dinosaur Fractal) — printed
  1/0 base body via the new `enters_with_counters` replacement
  (was 1/1 over-statement to keep the body alive). At X=2 it lands
  as a 3/2 (matching printed 1/0 + 2 +1/+1 counters); at X=0 the
  1/0 body has 0 toughness with no counters and immediately
  graveyards (matching printed). The "you gain 2 life" half stays
  on the ETB trigger.

### STX 2021 fidelity bump (1 — stays ✅, base body now exact)

- **Star Pupil** (STX Silverquill {B}, 0/0 Spirit) — printed 0/0
  body now exact via the new `enters_with_counters` replacement
  (was 1/1 over-statement). Two +1/+1 counters land at bf entry
  before the 0-toughness SBA fires. Felisa, Fang of Silverquill's
  "creature you control with a counter on it dies" trigger still
  fires correctly (counters are real `CounterType::PlusOnePlusOne`).

### STX 2021 fidelity bump (1 — stays 🟡)

- **Reckless Amplimancer** (STX mono-green {2}{G}, 0/0 Elf Druid
  Mutant) — printed 0/0 base body + replacement-effect counters.
  Per-permanent-you-control proxy stays (the printed
  per-mana-symbol scaling needs a per-pip introspection primitive
  the engine doesn't have yet).

### Hybrid pip fidelity bumps (4 — all stay-status, more faithful)

The engine's mana-payment path supports hybrid pips since push
XXXVIII (Spectacle Mage). This push wires through 4 SOS factories
that were approximating their hybrid pips as a single color:

- **Stirring Honormancer** ({2}{W}{W/B}{B}, 4/5 Bard) — `{W/B}` pip
  now exact.
- **Practiced Scrollsmith** ({R}{R/W}{W}, 3/2 Dwarf Cleric) —
  `{R/W}` pip now exact.
- **Paradox Surveyor** ({G}{G/U}{U}, 3/3 Elf Druid) — `{G/U}` pip
  now exact.
- **Essenceknit Scholar** ({B}{B/G}{G}, 3/1 Dryad Warlock) —
  `{B/G}` pip now exact.

### Tests (+8 net, 1368 → 1376)

- Lluwen back-face castable with {G}-only pool (hybrid pip exact).
- Lluwen back-face rejects empty mana pool.
- Practiced Scrollsmith castable with {R}{W}{W} (hybrid as W).
- Essenceknit Scholar castable with {B}{G}{G} (hybrid as G).
- Pterafractyl X=0 dies to 0-toughness SBA (printed body).
- Pterafractyl printed body is 1/0 + replacement field wired.
- Star Pupil printed body is 0/0 + replacement field wired.
- `enters_with_counters` snapshot serde round-trip.

## 2026-05-03 push XXXIX: Cast-time additional cost + Aura pre-attach + 10 cards

Three engine primitives + 5 STX 🟡 → ✅ promotions + 5 new STX 2021
cards + 1 server view enrichment + 1 bot affordability check.
Tests at 1368 (was 1363, +5 net).

### Engine primitives

- **`Value::IfPredicate { cond, then, else_ }`** — branching value.
  Lets a `Value` switch on a `Predicate` evaluated against the same
  `EffectContext`. Used by Wilt in the Heat's printed "{2} less if
  one or more cards left your graveyard this turn" wired through
  `StaticEffect::CostReductionScaled` with
  `amount: IfPredicate { cond: CardsLeftGraveyardThisTurnAtLeast(1),
  then: 2, else_: 0 }`. Future "X = N if condition else 0" payoffs
  (Spectacle riders, conditional pump magnitudes) reuse the same
  shape.

- **`CardDefinition.additional_sac_cost: Option<SelectionRequirement>`**
  — "as an additional cost to cast this spell, sacrifice a [filter]"
  primitive. The cast pipeline's `cast_spell` does a pre-flight check
  (the controller must have ≥1 matching permanent, otherwise the cast
  rejects with `SelectionRequirementViolated`), then auto-picks the
  lowest-value matching creature (tokens first, then by mana value,
  then by power) and sacrifices it after mana payment but before the
  spell goes on the stack. Daemogoth Woe-Eater + Eyeblight Cullers
  graduated from "ETB sacrifice approximation" to printed-faithful
  cast-time sacrifice.

- **`stack.rs` Aura cast-time pre-attach** — when a permanent spell is
  an Aura with a Permanent target (CR 303.4f), the engine pre-binds
  the target onto `card.attached_to` at the moment the Aura enters the
  battlefield. Without this, the orphaned-aura SBA (CR 704.5m) would
  immediately graveyard the Aura between bf entry and the cast-target
  snapshot. Lets simple-shape Auras (Solid Footing) stay on bf and
  apply their static buff via `Selector::AttachedTo(This)`.

- **`target_filter_for_slot_in_mode` walks Value args** — adds a
  `val_find` recursion that lets the cast-time filter check pull a
  slot 0 filter out of a `DealDamage.amount`'s `Value::PowerOf(target_
  filtered(...))`. Closes the Decisive Denial mode 1 fidelity gap
  (slot 0 friendly-creature filter is now enforced) and unblocks
  Pest Wallop's "your creature deals damage" printed slot-0 filter.

### STX 🟡 → ✅ promotions (5)

- **Wilt in the Heat** (SOS Lorehold {2}{R}{W}) — the printed cost-
  reduction-when-cards-left-gy clause now wires faithfully via the
  new `Value::IfPredicate` + `StaticEffect::CostReductionScaled`. The
  spell card is its own static source (read by `cost_reduction_for_
  spell` walking `card.definition.static_abilities` at cast time),
  matching Ajani's Response's self-static pattern. The "if it would
  die, exile instead" damage-replacement rider stays gap (no damage-
  replacement primitive).

- **Daemogoth Woe-Eater** (STX Witherbloom {2}{B}{G}, 9/9 Demon) —
  ETB-sacrifice approximation removed; the additional cost is now
  paid at cast time via `additional_sac_cost`. Without a
  sacrificable creature in play the cast is rejected with
  `SelectionRequirementViolated` (matches the printed "as an
  additional cost: sacrifice a creature").

- **Eyeblight Cullers** (STX Witherbloom {1}{B}{B}, 4/4 Elf
  Warrior) — same shape as Woe-Eater. The double-counted ETB sac
  has been dropped; the drain rider stays unchanged on the ETB
  trigger.

- **Big Play** (STX Quandrix {3}{G}{U}) — fidelity bump. The "up to
  two creatures" rider now applies to two friendly creatures via a
  `ForEach + Selector::take(EachPermanent(Creature ∧ ControlledByYou),
  2)` fan-out body. Each picked creature untaps + +1/+1 + hexproof +
  trample EOT.

- **Decisive Denial** (STX Quandrix {G}{U}) — mode 1's slot 0 filter
  (friendly creature) is now enforced at cast time via the new
  `val_find` recursion. Picking an opp creature in mode 1 now rejects
  with `SelectionRequirementViolated`. Mode 0 (counter-noncreature-
  unless-{2}) unchanged.

### New STX 2021 cards (5)

- **Pest Wallop** (STX Witherbloom-adjacent {3}{G}) — your creature
  pumps +1/+1 EOT then deals damage = its power to an opp creature.
  Slot 0 must be friendly (cast-time filter). Same one-sided shape
  as Decisive Denial mode 1.

- **Solid Footing** (STX Lorehold-adjacent {W}) — Aura. Enchanted
  creature gets +1/+2 and gains vigilance. Wires via the new Aura
  cast-time pre-attach + `StaticEffect::PumpPT` + `StaticEffect::
  GrantKeyword` over `Selector::AttachedTo(This)`. First catalog
  Aura that uses the static-attach layer pattern.

- **Swarm Shambler** (STX Quandrix-adjacent {G}, 1/1 Beast) — ETB
  +1/+1 counter; `{2}{G}: untap + add a +1/+1 counter`. Mono-green
  growth body that scales with available mana.

- **Containment Breach** (STX shared {1}{W}) — Instant: destroy
  target enchantment + Learn. Standard enchantment removal + cantrip
  (Learn collapses to Draw 1, same approximation as Eyetwitch /
  Hunt for Specimens / Igneous Inspiration / Professor of Symbology).

- **Unwilling Ingredient** (STX Witherbloom {B}, 1/1 Insect Pest) —
  When this creature dies, may pay {B}: draw a card. Death-trigger
  uses `Effect::MayPay`. AutoDecider declines by default;
  ScriptedDecider yes path drives the cantrip.

### UI improvement: KnownCard.additional_cost_label

- New `KnownCard.additional_cost_label: Option<String>` field
  populated from `CardDefinition.additional_sac_cost` via a tiny
  filter-shape renderer (`Sacrifice a creature` /
  `Sacrifice an artifact` / etc.). Lets the client warn before
  wasting mana on a spell the controller can't currently afford
  (Daemogoth Woe-Eater / Eyeblight Cullers without a creature to
  sacrifice). Defaulted to `None` for back-compat with older
  serialized views.

### Bot improvement: additional-sac-cost affordability

- `can_afford_in_state` now rejects a hand card whose
  `additional_sac_cost` filter has no matching permanent on the
  battlefield (other than the spell card itself). Skips dry-run
  noise for Daemogoth Woe-Eater / Eyeblight Cullers casts that
  the engine would reject anyway.

### Tests (+5 net, 1363 → 1368)

- Wilt in the Heat: 2 new tests (discount fires when card-left-gy
  tally ≥1; no discount when quiet graveyard).
- Daemogoth Woe-Eater: 1 new test (cast rejected without
  sacrificable creature).
- Eyeblight Cullers: 1 new test (cast rejected without
  sacrificable creature; ETB drain doesn't fire).
- Big Play: 1 new test (two friendly creatures pumped).
- Decisive Denial: 1 new test (mode 1 rejects opp creature target).
- Pest Wallop: 2 new tests (friendly pumps + damages opp; rejects
  opp creature target).
- Solid Footing: 2 new tests (pumps enchanted creature + vigilance;
  graveyards when enchanted creature dies — covers CR 704.5m).
- Swarm Shambler: 2 new tests (ETB counter; activation untap +
  counter).
- Containment Breach: 1 new test (destroys enchantment + cantrip).
- Unwilling Ingredient: 1 new test (death + MayPay yes path draws).
- KnownCard view: 2 new tests (additional_cost_label populated for
  Woe-Eater; absent for vanilla Bears).

Net: -3 (a few existing tests subsumed by the ETB-sac removal on
Woe-Eater/Cullers — those tests still pass since the cast-time path
puts the bear in graveyard before the ETB resolves).

### CR 704.5m audit

The cast-time pre-attach work surfaced a pre-existing engine path
for CR 704.5m ("If an Aura is attached to an illegal object or
player, or is not attached to an object or player, that Aura is put
into its owner's graveyard"). The implementation is now exercised by
two tests: `solid_footing_pumps_enchanted_creature_with_vigilance`
(legal aura survives) + `solid_footing_graveyards_when_enchanted_
creature_dies` (orphaned-aura SBA fires when enchanted creature
dies). The rule is implemented and validated. Code comment in
`stack.rs` updated from `CR 704.5n` (incorrect) to `CR 704.5m`
(correct citation).

## 2026-05-03 push XXXVIII: Cost reduction primitives + 10 promotions

Four engine primitives + 10 card promotions across STX 2021 and SOS.
Tests at 1363 (was 1336, +27 net).

### Engine primitives

- **`StaticEffect::CostReductionTargeting { spell_filter, target_filter,
  amount }`** — Killian, Ink Duelist's "spells you cast that target a
  creature cost {2} less to cast." `cost_reduction_for_spell` walks
  every battlefield permanent's static abilities (controller-scoped to
  the caster) plus the cast card's own static abilities, summing
  matching discounts. The discount is applied to generic mana only via
  the new `ManaCost::reduce_generic` method, which drains `Generic(N)`
  pips left-to-right and caps at 0 (colored requirements always
  remain). All three cast paths (regular, alt cost, flashback)
  consult the reduction.

- **`StaticEffect::CostReductionScaled { filter, amount: Value }`** —
  Affinity-style cost reduction whose discount evaluates a `Value` at
  cast time. Used by Witherbloom, the Balancer (Affinity for creatures
  → `CountOf(EachPermanent(Creature ∧ ControlledByYou))`) and The
  Dawning Archaic (`{1} less per IS card in your graveyard`).

- **`AffectedPermanents::All.excluded_supertypes`** + `.exclude_source`
  — adds two serde-default fields to the `All` layer-affected variant.
  `excluded_supertypes` filters out permanents bearing any of the
  listed supertypes (Hofri's "Other *nonlegendary* creatures"); the
  decomposer in `affected_from_requirement` detects
  `Not(HasSupertype(_))` and emits the new field. `exclude_source`
  is a future-proofing flag for "Other …" qualifiers on lord cards
  whose source isn't otherwise filtered (defaulted to false; not
  currently used since Hofri herself is Legendary).

- **`AlternativeCost.mode_on_alt: Option<usize>`** — when `Some(idx)`,
  casting via the alt cost path auto-selects mode `idx` of a modal
  spell, overriding any caller-supplied mode. Used by Devastating
  Mastery's Mastery alt cost ({7}{W}{W}) which auto-selects mode 1
  (Wrath + reanimate); regular cast at {4}{W}{W} resolves mode 0
  (Wrath only).

- **Prowess wired as a synthetic SpellCast trigger.** New code in
  `fire_spell_cast_triggers` sweeps every battlefield permanent with
  `Keyword::Prowess` controlled by the caster on each *noncreature*
  spell cast and pushes a synthetic +1/+1 EOT pump trigger sourced at
  the Prowess permanent. Spectacle Mage, Monastery Swiftspear, and
  any future Prowess body all share the same trigger path.

### Card promotions (10)

**STX 2021 (5):**
- **Killian, Ink Duelist** ({W}{B}, 2/3 Lifelink) — 🟡 → ✅. Static
  cost reduction via `CostReductionTargeting`.
- **Spectacle Mage** ({U/R}{U/R}, 1/2 Prowess) — 🟡 → ✅. Hybrid mana
  via `ManaSymbol::Hybrid(Blue, Red)` + Prowess wired.
- **Tempted by the Oriq** ({1}{W}{B} Sorcery) — 🟡 → ✅. Permanent-
  duration `Effect::GainControl` on a ≤3-MV creature + Inkling token.
- **Devastating Mastery** ({4}{W}{W} Sorcery) — 🟡 → ✅. 2-mode
  `ChooseMode` + alt cost via `mode_on_alt: Some(1)`.
- **Hofri Ghostforge** ({2}{R}{W}, 3/4 Legendary) — anthem fix
  via `excluded_supertypes`. Stays 🟡 because the dies-as-Spirit-copy
  rider still needs a token-copy-of-permanent primitive.

**SOS (5):**
- **Ajani's Response** ({4}{W} Instant) — 🟡 → ✅. Self-static
  cost reduction (no permanent in play needed).
- **Witherbloom, the Balancer** ({6}{B}{G} Elder Dragon) — first
  Affinity clause now wires via `CostReductionScaled`. Stays 🟡 because
  the second clause ("IS spells you cast have affinity") needs a
  cross-spell discount-grant primitive.
- **The Dawning Archaic** ({10} Avatar, 7/7 Reach) — ⏳ → 🟡. New
  body factory + cost discount per IS card in graveyard.
- **Inkshape Demonstrator** ({3}{W}, 3/4 Ward(2)) — 🟡 → ✅. Doc-only
  promotion (Ward keyword + Repartee body wired since first add).
- **Ennis, Debate Moderator** ({1}{W}, 1/1 Legendary) — 🟡 → ✅.
  Doc-only promotion (exact-tally gate has been wired since push IX).

### UI improvement: PermanentView.static_abilities

`net::PermanentView` gains a `static_abilities: Vec<String>` field
populated from each `StaticAbility.description`. Lets clients render
the printed rules-text without rebuilding it from the static-effect
tree. Defaulted to empty for back-compat.

### Bot improvement: discount-aware affordability prefilter

`extra_cost_for_card_in_hand` now subtracts target-independent cost
reductions (Witherbloom's Affinity, Dawning Archaic's gy-scaled
discount) before returning the net excess. Target-dependent reductions
(Killian's targeting filter) still resolve at would_accept time.

### Tests (+27 net, 1336 → 1363)

- Killian: 5 (discount lands on tapped creature target; no-discount
  without Killian; targetless spells skip discount; two Killians cap
  at zero generic; opponent's Killian doesn't discount your spells).
- Spectacle Mage: 4 (Prowess pump on noncreature cast; Prowess skips
  creature-spell cast; hybrid pays from blue or red).
- Hofri: 3 (anthem pumps nonleg creatures; skips Legendary; skips opp).
- Tempted by the Oriq: 2 (gain control + Inkling; rejects ≥4 MV).
- Devastating Mastery: 2 (regular cast Wraths only; alt cost Wraths +
  reanimates).
- Ajani's Response: 2 (discount on tapped creature; no discount on
  untapped).
- Witherbloom Balancer: 3 (Affinity discount; no discount with 0
  creatures; caps at 0 generic).
- Dawning Archaic: 3 (body; discount per IS card; no discount with
  empty gy).
- View: 2 (static_abilities populates; empty for vanilla creatures).
- Snapshot serde: 3 (CostReductionTargeting; CostReductionScaled;
  AlternativeCost.mode_on_alt incl. legacy back-compat).

## 2026-05-03 push XXXVII: Effect::PickModeAtResolution + StaticEffect::TaxActivatedAbilities + 4 STX 2021 promotions

Two new engine primitives + 4 STX 2021 🟡 → ✅ promotions + 2 doc fixes
on already-wired SOS cards. Tests at 1336 (was 1325, +11 net).

### Engine primitives

- ✅ **`Effect::PickModeAtResolution(Vec<Effect>)`** — sibling to
  `Effect::ChooseMode` that prompts the controller for a mode pick at
  *resolution* time rather than cast time. Used by triggered abilities
  whose printed text reads "your choice of X or Y" — the surrounding
  spell's `ctx.mode` is already pinned to the cast-time pick (so re-
  using `ChooseMode` would incorrectly read the spell-level mode index).
  AutoDecider picks mode 0 (the universal safe default — usually
  Scry/+1/+1/draw); ScriptedDecider can flip via `Mode(N)`. The
  decision surfaces as `Decision::ChooseMode { source, num_modes }`,
  reusing the existing decision plumbing (no new wire-format type).

- ✅ **`StaticEffect::TaxActivatedAbilities { filter, amount }`** —
  Augmenter Pugilist-style "activated abilities of [filter] cost {N}
  more to activate." Walks all battlefield permanents at activation
  time via the new `extra_cost_for_activation` helper and surcharges
  the activator's mana cost when the activating permanent matches the
  filter. Multiple distinct sources (two Pugilists, Pugilist + tax
  artifact) sum. Mana abilities aren't exempt at the rules level —
  Llanowar Elves's `{T}: Add {G}` becomes `{2}, {T}: Add {G}` while
  Pugilist is on the battlefield. The static is *not* a layer-applied
  continuous effect (`game/mod.rs` skips it in the layers builder); it
  reads at activation time.

### STX 2021 promotions (4)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Shadrix Silverquill | {2}{W}{B} | ✅ ← 🟡 | Choose-2-of-3 attack trigger now wires via `Effect::ChooseModes { count: 2 }` re-used at trigger resolution. AutoDecider picks modes 0+1 (draw + drain — the canonical value pair). ScriptedDecider drives mode pairs that involve targeting (mode 1 needs an opp; mode 2 needs a creature). |
| Prismari Apprentice | {U}{R} | ✅ ← 🟡 | Magecraft "Scry 1 or +1/+0 EOT" now wires faithfully via the new `Effect::PickModeAtResolution`. AutoDecider picks mode 0 (Scry — universal safe pick). ScriptedDecider can flip mode 1 for the +1/+0 self-pump combat trick. |
| Augmenter Pugilist | {3}{G}{G} | ✅ ← 🟡 | Static "Activated abilities of creatures cost {2} more to activate" now wires via the new `StaticEffect::TaxActivatedAbilities { filter: Creature, amount: 2 }`. Augmenter Pugilist taxes every creature on the battlefield (friend + foe). Mana abilities are NOT exempt per the rules — Llanowar Elves `{T}: {G}` becomes `{2}, {T}: {G}` while Pugilist is in play. |
| Silverquill Apprentice | {W}{B} | ✅ ← 🟡 | Magecraft "+1/+1 or -1/-1" now wires faithfully via `Effect::PickModeAtResolution([+1/+1 EOT, -1/-1 EOT])`. AutoDecider picks mode 0 (pump — safe combat-trick default). ScriptedDecider flips mode 1 for the printed "shrink an opp creature" line. |

### SOS doc-only fixes (2)

| Card | Status | Notes |
|---|---|---|
| Potioner's Trove | ✅ (was 🟡 stale) | The "{T}: gain 2 life. Activate only if you've cast an IS spell this turn" gate has been wired since push XIII via `Predicate::InstantsOrSorceriesCastThisTurnAtLeast` + `Player.instants_or_sorceries_cast_this_turn`. Doc string updated to reflect the wired state (was claiming "engine has no per-turn-cast-tracking gate yet"). |
| Ennis, Debate Moderator | 🟡 (doc fix) | The end-step counter trigger uses the exact-printed `Predicate::CardsExiledThisTurnAtLeast` (push IX), backed by `Player.cards_exiled_this_turn`. Doc string previously claimed it was approximated via gy-leave proxy — now reflects the exact tally. |

### Tests (+11 net, 1325 → 1336)

- 7 STX 2021 promotion tests:
  `prismari_apprentice_auto_picks_scry_mode_on_magecraft`,
  `prismari_apprentice_scripted_picks_pump_mode`,
  `shadrix_silverquill_attack_trigger_draws_and_drains_via_auto_decider`,
  `shadrix_silverquill_attack_trigger_pumps_via_scripted`,
  `augmenter_pugilist_taxes_creature_activated_abilities`,
  `augmenter_pugilist_tax_satisfied_by_extra_generic`,
  `augmenter_pugilist_does_not_tax_noncreature_activations`,
  `silverquill_apprentice_magecraft_can_shrink_via_scripted_mode_one`.
- 2 snapshot serde round-trip tests:
  `pick_mode_at_resolution_effect_serde_round_trip`,
  `tax_activated_abilities_static_effect_serde_round_trip`.
- 1 view-label test:
  `ability_effect_label_handles_pick_mode_at_resolution`.

### Engine + UI integration

- `extra_cost_for_activation(state, source) -> u32` helper in
  `game/actions.rs` walks the battlefield's `TaxActivatedAbilities`
  statics and sums the surcharge for the activating permanent. Folded
  into `activate_ability` as additional generic mana before the
  pre-flight payment snapshot — failures roll back tap + mana cleanly.
- `ability_effect_label` (`server/view.rs`) gains an arm for
  `Effect::PickModeAtResolution`, surfacing the first non-fallback
  inner-mode label (Prismari Apprentice's "Scry/Surveil" instead of
  the catch-all "Activate").
- `effect.rs` introspection methods (`requires_target`,
  `primary_target_filter`, `accepts_player_target`,
  `prefers_friendly_target`, `target_filter_for_slot`) all gained
  `PickModeAtResolution` arms that walk the inner mode list — same
  shape as the existing `ChooseMode` / `ChooseModes` arms.
- `server/bot.rs::effect_uses_x` recurses into PickModeAtResolution
  so X-cost inner modes still feed the bot's affordability check.
- Snapshot wire format unchanged — both new primitives serialize via
  serde derives without dedicated wire types.

## 2026-05-03 push XXXVI: Effect::ChooseModes + 5 STX Commands + 3 SOS promotions

Engine adds the long-tracked **multi-modal "choose K of N"** primitive,
unlocking the printed "choose two" semantics on every Strixhaven Command
and the "choose one or more" semantics on Multiple Choice. Plus a new
`Effect::DelayUntil { capture: ... }` field that closes the Repartee
"capture-as-target from selector" gap for Conciliator's Duelist.
10 cards promoted to ✅ across SOS and STX 2021. Tests at 1325 (was
1315; +10 net).

### Engine primitives

- ✅ **`Effect::ChooseModes { modes, count, up_to, allow_duplicates }`** —
  resolution-time multi-mode picker, sibling to `Effect::ChooseMode`. The
  controller picks `count` modes (or up to `count` if `up_to`) from the
  list. Backed by the new `Decision::ChooseModes` and
  `DecisionAnswer::Modes(Vec<usize>)`. AutoDecider returns the first
  `count` modes (deterministic baseline); `ScriptedDecider::new([
  DecisionAnswer::Modes(vec![…])])` lets tests pick any combination.
  - **Backwards-compat single-mode override**: if `ctx.mode != 0` (set
    via cast-time `mode: Some(N)` arg), the resolver runs *only* mode
    N — preserves existing test casts that target a specific mode of a
    Command. New ChooseModes cards cast with `mode: None` route through
    the decider.
  - Wire format: `DecisionWire::ChooseModes { source, modes_count,
    pick_count, up_to, allow_duplicates }` round-trips for snapshot /
    spectator UI parity.

- ✅ **`Effect::DelayUntil.capture: Option<Selector>`** — extends the
  existing delayed-trigger primitive with an optional selector that's
  evaluated at delay-registration time. The first entity it resolves
  to is bound to the delayed body's `Selector::Target(0)` slot,
  overriding the legacy `ctx.targets[0]` capture path. Used by
  Conciliator's Duelist's Repartee — the trigger has no target slot of
  its own (Repartee fires on a cast event), but the cast spell's
  target is captured via `Selector::CastSpellTarget(0)` and threaded
  into the delayed return. Existing usages (Goryo's Vengeance, Ennis
  Debate Moderator, Restoration Angel, etc.) get `capture: None` and
  retain the legacy `ctx.targets[0]` capture path.

### STX 2021 Command promotions (5)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Lorehold Command | {R}{W} | ✅ ← 🟡 | "Choose two" now wires faithfully via `Effect::ChooseModes { count: 2 }`. Auto-decider picks modes 0+1 (drain 4 + 2 Spirit tokens). |
| Witherbloom Command | {B}{G} | ✅ ← 🟡 | Same shape — auto-decider picks drain 3 + gy → hand. |
| Prismari Command | {1}{U}{R} | ✅ ← 🟡 | Auto-decider picks 2 dmg + discard/draw. |
| Silverquill Command | {2}{W}{B} | ✅ ← 🟡 | Auto-decider picks counter ability + -3/-3. ScriptedDecider drives target-less drain+draw. |
| Quandrix Command | {1}{G}{U} | ✅ ← 🟡 | Auto-decider picks counter ability + +1/+1 ×2. |

### SOS / STX 2021 individual promotions (5)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Conciliator's Duelist | {W}{W}{B}{B} | ✅ ← 🟡 | Repartee's "exile + return at next end step" rider now wires via the new `Effect::DelayUntil { capture: Some(Selector::CastSpellTarget(0)) }` field. Trigger fires at next end step and the captured target moves back to bf under owner. |
| Borrowed Knowledge | {2}{R}{W} | ✅ ← 🟡 | Doc-only promotion. Both modes were already wired (mode 0 = discard hand + draw=opp.hand_size; mode 1 = discard hand + draw = `Value::CardsDiscardedThisResolution`). Status was a stale annotation. |
| Mentor's Guidance | {2}{G}{U} | ✅ ← 🟡 | Doc-only promotion. The printed Oracle is single-target so the existing wire matches printed exactly — the prior 🟡 was a stale annotation that misread "for each card in your hand" as multi-target fan-out. |
| Multiple Choice | {1}{U}{U} | ✅ ← 🟡 | "Choose one or more" now wires faithfully via `Effect::ChooseModes { count: 3, up_to: true }`. Auto-decider picks all 3 modes. The "if you chose all of the above" mega-mode rider stays gap (would need modes-picked introspection). |
| Lorehold, the Historian | {3}{R}{W} | 🟡 → 🟡 | Per-opp-upkeep loot trigger now wired via `EventScope::OpponentControl + StepBegins(Upkeep)`. Body uses `Effect::MayDo` so AutoDecider's "no" default skips on bot turns; ScriptedDecider yes path drives the discard+draw chain. Miracle grant still omitted. |

### Tests (+10 net, 1315 → 1325)

- 6 ChooseModes integration tests:
  `lorehold_command_choose_modes_drains_and_creates_spirits`,
  `witherbloom_command_choose_modes_runs_two_halves`,
  `witherbloom_command_choose_modes_destroy_and_pump_via_scripted_decider`,
  `prismari_command_choose_modes_damage_and_loot`,
  `silverquill_command_choose_modes_drain_and_draw_via_scripted`,
  `quandrix_command_choose_modes_gy_to_library_and_draw_via_scripted`.
- `multiple_choice_choose_modes_runs_all_three` + reordered
  `multiple_choice_mode_two_creates_pest_token` (mode 1 → mode 2 to
  match printed Oracle order).
- `moment_of_reckoning_still_uses_single_mode_pick` regression check
  (MOR keeps `ChooseMode` since "up to four / same mode more than
  once" needs multi-target slots, not just multi-mode).
- `conciliators_duelist_repartee_returns_target_at_next_end_step` —
  end-to-end exile + return cycle.
- `lorehold_the_historian_loots_on_each_opp_upkeep` — opp's upkeep
  triggers the loot trigger.

## 2026-05-03 push XXXV: 6 SOS multi-target promotions + 3 fidelity bumps

Six SOS 🟡 → ✅ promotions across multiple colors — each closes a
different multi-target or destination-prompt gap that had kept the
printed card at 🟡 even after the rest of its primitives were wired.
Plus three additional cards (Stress Dream, Burrog Barrage,
Homesickness) gain meaningful fidelity bumps (still 🟡 due to deeper
gaps but their auto-target shape now matches the printed flavor).

### SOS promotions (6)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Dina's Guidance | {1}{B}{G} | ✅ ← 🟡 | The hand-or-graveyard destination prompt is now wired as `Effect::ChooseMode` with two modes (Search → Hand vs Search → Graveyard). Both modes use the existing `Effect::Search` primitive (already routes to `ZoneDest::Graveyard`). Reanimator decks (Goryo's Vengeance / Animate Dead / Reanimate downstream) can flip to mode 1 via the cast-time `mode` argument; `mode: None` defaults to mode 0 (hand, the strictly stronger pick). |
| Vibrant Outburst | {U}{R} | ✅ ← 🟡 | The printed "tap up to one target creature" half is now wired via `Selector::one_of(EachPermanent(opp creature))` — same approximation as Decisive Denial mode 1 / Chelonian Tackle. Tap auto-picks an opp creature; no-ops cleanly when no opp creature is on the battlefield. The 3-damage primary slot is still user-targeted (any target). |
| Dissection Practice | {B} | ✅ ← 🟡 | The printed "Up to one target creature gets +1/+1 EOT" half is now wired via `Selector::one_of(EachPermanent(Creature ∧ ControlledByYou))`. Three optional halves all fire: drain 1 + +1/+1 friendly + -1/-1 user-targeted (same multi-target collapse as Vibrant Outburst). |
| Practiced Offense | {2}{W} | ✅ ← 🟡 | The printed "your choice of double strike or lifelink" mode pick is now a top-level `Effect::ChooseMode`: mode 0 = +1/+1 fan-out + double strike grant; mode 1 = +1/+1 fan-out + lifelink grant. Cast-time `mode: Some(0)` / `Some(1)` flips between the two; `None` defaults to DS (the strictly more aggressive pick). Flashback {1}{W} unchanged. |
| Cost of Brilliance | {2}{B} | ✅ ← 🟡 | The +1/+1 half is now optional via `Selector::one_of(EachPermanent(Creature ∧ ControlledByYou))` — auto-picks a friendly creature, no-ops cleanly when none exist. Cast is now legal even when you control no creatures (just self-loot fires). |
| Render Speechless | {2}{W}{B} | ✅ ← 🟡 | The "up to one creature target" half now uses `Selector::one_of(EachPermanent(Creature ∧ ControlledByYou))` — same shape as Cost of Brilliance. Cast legal even with no friendly creature; just the discard half resolves. |

### Fidelity bumps (3, still 🟡)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Stress Dream | {3}{U}{R} | 🟡 → 🟡 | The 5-damage half now uses `Selector::one_of(EachPermanent(opp creature))` — auto-picks an opp creature (lethal-first). Cast is now legal even when no opp creature exists (just the scry+draw resolves). The "look at top 2, choose 1 to hand and other to bottom" half is still approximated as scry 1 + draw 1. |
| Burrog Barrage | {1}{G} | 🟡 → 🟡 | Damage half now hits an opp creature via `Selector::one_of(EachPermanent(opp creature))` (was: self-damage to slot 0 = friendly creature). One-sided power-as-damage to an opp creature, scaled by slot 0's power. No-ops when no opp creature exists. |
| Homesickness | {4}{U}{U} | 🟡 → 🟡 | Second creature slot now wired via `Selector::one_of(EachPermanent(opp creature))` — when 2 distinct opp creatures exist both get tapped + stunned. With only 1 opp creature, the auto-pick collides with slot 0 (2 stun counters on same target). Multi-target uniqueness is still an open engine gap. |

### Tests (+10 net)

- **`dinas_guidance_searches_creature_to_graveyard`** — verifies mode 1
  drops a Griselbrand directly into the controller's graveyard
  (reanimator-fuel mode), with the spell itself also resolving to
  graveyard. Existing `dinas_guidance_searches_creature_to_hand` test
  re-verified for mode 0 (made the `mode: Some(0)` explicit).
- **`vibrant_outburst_taps_opp_creature_alongside_damage`** — verifies
  3 damage to a friendly target plus the auto-picked opp creature
  getting tapped by the second half. Plus `vibrant_outburst_no_op_tap_
  when_no_opp_creature` for the no-creature-on-battlefield path.
- **`dissection_practice_pumps_friendly_creature`** — verifies the
  +1/+1 half lands on a friendly creature when one exists, alongside
  the existing -1/-1 + drain 1 halves.
- **`practiced_offense_mode_one_pumps_and_grants_lifelink`** — mode 1
  grants Lifelink (not Double Strike) on the chosen creature. Existing
  test made explicit on `mode: Some(0)`.

### Engine note: nested `ChooseMode`

The promotions confirm `Effect::ChooseMode` works correctly at any
nesting level — top-level (Practiced Offense, Dina's Guidance) and
inside `Effect::Seq` are both valid. The `ctx.mode` index is read once
per resolution (set at cast time) and the spell-level mode_pick is what
gets picked. Future cards that need *resolution-time* mode picks (e.g.
Prismari Apprentice's "Scry 1 or +1/+0 EOT" magecraft) still need a
`Effect::PickModeAtResolution(Vec<Effect>)` primitive — separate from
the cast-time `ctx.mode` plumbing.

## 2026-05-03 push XXXIV: exile_gy_cost + EOT-aware GainControl + 9 new cube cards

Two engine primitives + 2 STX 🟡 → ✅ promotions + 9 new cube/MH2 cards.
Tests at 1306 (was 1292; +14 net).

### Engine primitives

- **`ActivatedAbility::exile_gy_cost: u32`** (field on the existing
  `ActivatedAbility` struct). Pre-flight gate rejects with the new
  `GameError::InsufficientGraveyard` when the controller's graveyard
  has fewer cards than `exile_gy_cost`. After tap/mana/life payment
  succeeds, the engine picks the controller's `exile_gy_cost` oldest
  graveyard cards (gy index 0..N) and moves them to exile via
  `move_card_to(_, &ZoneDest::Exile)`. Each pick fires the standard
  `CardLeftGraveyard` event so SOS gy-leave payoffs (Spirit Mascot,
  Hardened Academic, Garrison Excavator, Living History) trigger
  off the cost. `#[serde(default)]` on the new field keeps the
  snapshot wire format back-compat (existing initialisers don't
  need to set it). Used by Lorehold Pledgemage's `{2}{R}{W}, exile
  a card from your graveyard: +1/+1 EOT`.
- **`Effect::GainControl` is now Duration-aware** (refactor of the
  previously-stub arm — used to permanently flip `card.controller`
  irrespective of its `duration` field). Now creates a Layer-2
  continuous effect (`Modification::ChangeController`) with the
  `Duration` mapped to `EffectDuration` — `EndOfTurn`/`EndOfCombat`
  → `UntilEndOfTurn` (reverted by `expire_end_of_turn_effects` at
  Cleanup), `UntilNextTurn`/`UntilYourNextUntap` → `UntilNextTurn`,
  `Permanent` → `Indefinite`. The computed-permanent `controller`
  field reflects the temporary new owner; combat / activation /
  decision pipelines already read computed_permanent.controller for
  ownership checks, so no other plumbing was needed. Used by
  Mascot Interception's "gain control until EOT + untap + haste"
  Threaten template.

### Engine improvement: post-move filter introspection

`evaluate_requirement_static` (server-side filter resolver) now walks
**hands and libraries** as a fallback for card-id lookups, in addition
to the existing battlefield → graveyards → exile → stack chain. Powers
trigger filters that fire *after* a card has already been moved out of
the graveyard — e.g. Murktide Regent's "instant or sorcery card left
your gy" trigger evaluates the filter after Zealous Lorecaster has
returned the bolt to hand. The card data is the same regardless of
zone — the lookup just needs to find it.

### STX 2021 promotions

| Card | Cost | Status | Notes |
|---|---|---|---|
| Lorehold Pledgemage | {1}{R}{W} | ✅ ← 🟡 | Activation now wires via the new `exile_gy_cost: 1` field. Pre-flight gate rejects with `InsufficientGraveyard` when gy is empty; otherwise auto-picks oldest gy card and moves to exile after tap/mana payment. Pump is +1/+1 EOT on `Selector::This`. |
| Mascot Interception | {2}{R}{W} | ✅ ← 🟡 | "Threaten / Act of Treason + untap + haste" now wires faithfully via `Effect::GainControl` (Layer-2 continuous effect) + `Untap` + `GrantKeyword(Haste, EOT)`. Control reverts to opp at Cleanup; haste grant is durable due to a separate engine gap (tracked in TODO.md). |

### 9 new cube / MH2 cards (`catalog::sets::decks::modern`)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Subtlety | {3}{U}{U} | ✅ | 3/3 Elemental Incarnation (Flying + Flash). ETB tucks target creature/PW to top of owner's library via `Move → Library{Top}`. Evoke pitch-cost omitted (alt-cost-by-pitch gap, same as Solitude / Endurance). |
| Monastery Swiftspear | {R} | 🟡 | 1/2 Human Monk with Haste + Prowess. Prowess keyword tag is correct; per-IS-cast +1/+0 trigger is engine work pending. |
| Wild Nacatl | {G} | 🟡 | Vanilla 1/1 Cat Warrior. Mountain/Plains lord effects pending (`StaticEffect::SelfPumpIfLandcontrolled` primitive gap). |
| Seasoned Pyromancer | {1}{R}{R} | 🟡 | 2/2 Human Shaman. ETB discard 2 + draw 2 + create 2 Elemental tokens (printed "for each nonland discarded" rider approximated as always-2). Gy-exile cast ability omitted. |
| Murktide Regent | {3}{U}{U} | 🟡 | 3/3 Flying Dragon. Gy-leave-by-IS trigger wired (`+1/+1 counter`); ETB-with-counters and Delve alt-cost both omitted. |
| Faerie Mastermind | {1}{U} | 🟡 | 2/1 Faerie Rogue (Flash + Flying). Opp-draw trigger fires on every CardDrawn (the "except first turn draw" gate is pending). {2}{U}, sac: draw 1 activation wired faithfully. |
| Fury | {2}{R}{R} | 🟡 | 3/3 Elemental Incarnation (Double Strike). ETB 4-damage to a single creature/PW; "divided" rider omitted (same gap as Magma Opus). Evoke pitch-cost omitted. |
| Young Pyromancer | {1}{R} | ✅ | 2/1 Human Shaman. Magecraft (cast/copy IS) → 1/1 red Elemental token. |
| Grief | {2}{B} | 🟡 | 3/2 Elemental Incarnation (Menace). ETB targeted hand-strip via `DiscardChosen`. Evoke pitch-cost omitted. |
| Sage of the Falls | {3}{U} | 🟡 | 2/4 Bird Fish (Flying). Draw trigger gated on `HandSize ≥ 5` → may-do `Seq([Draw, Discard])`. Auto-decider declines the may-do by default. |

### UI improvement: exile-gy cost label

`ability_cost_label` (`server::view`) now renders the new
`exile_gy_cost` field as printed-text "Exile a card from your
graveyard" (or "Exile N cards from your graveyard" for `≥2`), mirroring
the existing `sac_cost` / `life_cost` rendering. Powers Lorehold
Pledgemage's UI tooltip; future `exile_gy_cost`-flavoured activations
inherit the label automatically. Tests:
`ability_cost_label_renders_exile_gy_cost`.

### Tests (+14 net)

- **3 STX promotions**: `lorehold_pledgemage_pumps_via_exile_from_graveyard_cost`,
  `lorehold_pledgemage_rejects_when_graveyard_is_empty`, augmented
  `lorehold_pledgemage_has_reach`. `mascot_interception_steals_opp_creature_until_eot`,
  `mascot_interception_control_reverts_at_end_of_turn`.
- **9 new card tests**: subtlety / monastery_swiftspear / wild_nacatl /
  seasoned_pyromancer / murktide_regent / faerie_mastermind / young_pyromancer /
  grief / sage_of_the_falls / fury (one per card; play-pattern coverage).
- **1 view test**: `ability_cost_label_renders_exile_gy_cost`.

## 2026-05-03 push XXXIII: 8 promotions + any_target() shortcut + Pump/Shrink label split

Eight 🟡 → ✅ promotions across Strixhaven 2021 (Lorehold Apprentice,
Storm-Kiln Artist, Decisive Denial) and the SOS catalog (Thunderdrum
Soloist, Expressive Firedancer, Molten-Core Maestro, Ambitious
Augmenter, Topiary Lecturer). Plus a new `effect::shortcut::any_target()`
helper, a UI label arm for the same shape, and a sign-aware Pump-vs-
Shrink split in `ability_effect_label`. Tests at 1292 (was 1279, +13).

### Promotions

| Card | Cost | Status | Notes |
|---|---|---|---|
| Lorehold Apprentice | {R}{W} | ✅ ← 🟡 | Magecraft body now uses the new `effect::shortcut::any_target()` (`Creature ∨ Planeswalker ∨ Player`) for the "1 damage to any target" half — auto-target picks the opp face for hostile damage but falls through to creatures / planeswalkers when face damage isn't legal (hexproof, shroud). The +1 lifegain half is unchanged (resolves first via `Effect::Seq`). |
| Storm-Kiln Artist | {2}{R}{W} | ✅ ← 🟡 | Same `any_target()` upgrade for the "1 damage to any target" magecraft. Treasure follow-up unchanged. |
| Decisive Denial | {G}{U} | ✅ ← 🟡 | Mode 1 now wired: `DealDamage { to: one_of(EachPermanent(opp creature)), amount: PowerOf(target_filtered(Creature ∧ ControlledByYou)) }`. Slot 0 is the user-picked friendly; opp creature is auto-picked (one-sided damage, not Fight). Mode 0 (counter-noncreature-unless-{2}) unchanged. |
| Thunderdrum Soloist | {1}{R} | ✅ ← 🟡 | Opus rider wired via `opus(5, ...)`. Always: 1 damage to each opp. Big-cast (≥5 mana): an additional 2 damage (net 3 to each opp) — additive approximation matching Spectacular Skywhale / Tackle Artist's pattern. |
| Expressive Firedancer | {1}{R} | ✅ ← 🟡 | Opus rider wired. Always: +1/+1 EOT. Big-cast: `Keyword::DoubleStrike` granted EOT (additive — both halves run, the +1/+1 stacks under the Double Strike). |
| Molten-Core Maestro | {1}{R} | ✅ ← 🟡 | Opus rider wired. Always: +1/+1 counter on This. Big-cast: `AddMana(OfColor(Red, PowerOf(This)))` reads the post-counter power, so a 2/2 Maestro on a 5-mana cast adds {R}{R}{R} (+1/+1 counter resolves first → power 3). |
| Ambitious Augmenter | {G} | 🟡 ← 🟡 | Increment trigger now wired via `increment()`. The dies-as-Fractal-with-counters rider stays omitted pending a counter-transfer-on-death primitive — body still 🟡 until both clauses ship. |
| Topiary Lecturer | {2}{G} | ✅ ← 🟡 | Increment trigger wired via `increment()`. Each cast at ≥2 mana drops a +1/+1 counter, and the existing `{T}: Add {G}×power` ability scales linearly with each Increment-grown counter. |

### New shortcut: `effect::shortcut::any_target()`

`target_filtered(Creature ∨ Planeswalker ∨ Player)` — the canonical
"any target" filter for `Effect::DealDamage` magecraft / Repartee
triggers and burn spells whose printed wording is "deals N damage to
any target". The auto-target picker first tries the opponent face
(`Effect::DealDamage::accepts_player_target() = true`) and only falls
through to creatures / planeswalkers when the player face is not a
legal pick (hexproof, shroud). Used by Lorehold Apprentice, Storm-Kiln
Artist; future "any target" burn spells inherit the helper.

### UI improvement: any_target label

`entity_matches_label` (server/view.rs) recognises the 3-way Or shape
(`Creature ∨ Planeswalker ∨ Player`, left-associative as produced by
`any_target()`) and renders it as the canonical "any target" — same
wording the printed cards use. Both nesting orders are matched
(`(Creature ∨ Planeswalker) ∨ Player` and `Player ∨ (Creature ∨
Planeswalker)`) so future helpers that build the filter in either
direction surface the same label.

### Engine improvement: PumpPT label sign split

`ability_effect_label` (server/view.rs) now splits `Effect::PumpPT`
into "Pump" (positive or dynamic P/T) and "Shrink" (both halves
non-positive with at least one negative — Const-only). Powers the
activated-ability badge UI for ~12 catalog cards that use negative
PumpPT: Burrog Befuddler's magecraft -2/-0, Witherbloom Command's
mode 3 -3/-3, Dina, Soul Steeper's activated -X/-X, Lash of Malice-
style EOT shrinks. X-cost / dynamic values (XFromCost, CountOf, Diff)
default to "Pump" since static sign classification isn't possible.

### Tests (+13 net, 1279 → 1292)

13 new tests cover the promotions and the engine/UI improvements:

- `tests::stx::lorehold_apprentice_pings_creature_when_opp_face_is_hexproof`
  (smoke for the `any_target()` shape after the Apprentice promotion)
- `tests::stx::decisive_denial_mode_one_damages_opp_creature_by_friendly_power`
  (Tyrant 7 power → opp bear dies, friendly takes no return damage)
- `tests::stx::decisive_denial_mode_one_uses_target_creature_power`
  (Tyrant kill confirmation + one-sided damage check)
- `tests::sos::thunderdrum_soloist_pings_each_opp_on_cheap_cast`
- `tests::sos::thunderdrum_soloist_pings_three_each_opp_on_big_cast`
  (Mind Twist X=4 → ManaSpentToCast=5 → 3 to each opp)
- `tests::sos::expressive_firedancer_pumps_on_cheap_cast`
- `tests::sos::expressive_firedancer_grants_double_strike_on_big_cast`
- `tests::sos::molten_core_maestro_drops_counter_on_cheap_cast`
- `tests::sos::molten_core_maestro_adds_red_mana_on_big_cast`
  (5-mana cast → +1/+1 counter then {R}×3)
- `tests::sos::ambitious_augmenter_grows_on_two_mana_cast`
- `tests::sos::ambitious_augmenter_does_not_grow_on_one_mana_cast`
- `tests::sos::topiary_lecturer_grows_on_three_mana_cast`
- `server::view::tests::ability_effect_label_splits_pump_vs_shrink_by_sign`

Plus an extension to `entity_matches_label_covers_or_composite_filters`
covering the new "any target" arm.

## 2026-05-03 push XXXII: 13 new STX 2021 cards + lethal-first auto-target + UI label coverage

13 new STX 2021 cards added across mono and cross-college slots, plus
an engine improvement to the auto-target picker for hostile damage
spells, and three UI label arms.

### Card additions (`catalog::sets::stx::mono`)

All 13 cards use existing engine primitives — no new primitives gated.

| Card | Cost | Status | Notes |
|---|---|---|---|
| Vortex Runner | {1}{U} | ✅ NEW | 1/2 Salamander Warrior. `Keyword::Unblockable` (matches printed "can't be blocked") + `Attacks/SelfSource → Scry 1`. |
| Burrog Befuddler | {1}{U} | ✅ NEW | 1/3 Frog Wizard with Flash + magecraft `PumpPT(-2, 0, EOT)` on a target creature — combat-trick magecraft body. |
| Crackle with Power | {X}{R}{R}{R} | ✅ NEW | Sorcery. `DealDamage` to any target, amount = `Times(XFromCost, 5)`. At X=3 deals 15 damage; at X=2 deals 10. Routes through the new lethal-first auto-target picker for "any target" creature kills. |
| Sundering Stroke | {3}{R}{R}{R} | 🟡 NEW | Sorcery. 7 damage to one target. The "divided as you choose among 1, 2, or 3 targets" multi-target variant + the {R}{R}{R}{R}-spent doubling rider are both omitted (no divided-damage primitive — same gap as Magma Opus). |
| Professor of Symbology | {1}{W} | 🟡 NEW | 1/1 Bird Wizard with Flying. ETB Learn collapses to `Draw 1` (Lesson sideboard model not yet present — same approximation as Eyetwitch / Hunt for Specimens / Igneous Inspiration). |
| Professor of Zoomancy | {1}{G} | ✅ NEW | 1/1 Squirrel Wizard. ETB mints a 1/1 green Squirrel creature token. |
| Leyline Invocation | {4}{G} | ✅ NEW | Sorcery — Lesson. Mint a 0/0 green Elemental token + stamp it with N +1/+1 counters where N = lands you control. Uses `Selector::LastCreatedToken` + `AddCounter` with `Value::CountOf(EachPermanent(Land ∧ ControlledByYou))` — same shape as Body of Research / Snow Day. Token P/T are locked at creation time. |
| Verdant Mastery | {3}{G}{G} | 🟡 NEW | Sorcery. Two `Effect::Search` calls (basic land tapped to bf, then basic land to hand). The opp-half search is omitted (no `Effect::Search` variant targeting an opponent). The {7}{G}{G} alt cost is omitted (alt-cost-implies-mode primitive gap). |
| Rise of Extus | {3}{W}{B} | ✅ NEW | Sorcery — Lesson. `Seq([Exile(Creature OR Planeswalker), Move(graveyard pick → battlefield)])` — exile + reanimate combo, single-target on each half. |
| Gnarled Professor | {3}{G} | ✅ NEW | 4/4 Treefolk Druid with Reach. ETB `MayDo(Discard 1 → Draw 1)`. AutoDecider defaults "no"; ScriptedDecider can flip "yes" for tests. |
| Inkfathom Witch | {2}{B} | ✅ NEW | 2/2 Faerie Warlock with Flying. `Attacks/SelfSource → MayPay({1}{B}, Drain 2)` attack-trigger optional drain. |
| Blood Researcher | {1}{B} | ✅ NEW | 1/1 Vampire Wizard. `LifeGained/YourControl → AddCounter(This, +1/+1, ×1)` — lifegain payoff that scales linearly with Witherbloom drains. |
| First Day of Class | {W} | ✅ NEW | Sorcery. Two `ForEach` passes over `EachPermanent(IsToken ∧ Creature ∧ ControlledByYou)`: PumpPT(+1/+1, EOT) + GrantKeyword(Haste, EOT). Targets *only* token creatures (a non-token bear stays at base P/T). |

### Engine improvement: lethal-first auto-target

New `Effect::hostile_damage_amount(&self) -> Option<i32>` static
classifier that returns the constant damage amount of a damage effect.
Covers `DealDamage(Const)`, `DealDamage(Times(Const, Const))`, and
`Seq` leading with one. Returns None for X-cost folded values
(Crackle's `Times(XFromCost, 5)`) since X is only known at cast-time.

`auto_target_for_effect_avoiding` now consults that classifier on
hostile-target picks and re-sorts the primary-candidate list so
creatures whose toughness ≤ damage (lethal kills) come first, then by
descending power for tiebreaks. Pre-fix the picker walked battlefield
order and could pick a 2/2 utility creature when a 4/4 next-in-scan
was a clean kill.

### UI improvement: predicate_short_label coverage

Three new arms in `predicate_short_label` (server/view.rs) covering:
- `Value::CardsDrawnThisTurn(_)` — "after drawing" (≥1) / "if drew ≥N"
  / "if drew ≤N" — surfaces lifegain-on-draw style gates and
  Niv-Mizzet-flavored counts.
- `Value::PermanentCountControlledBy(_)` — "if has permanents" / "if
  ≥N permanents" / "if ≤N permanents" — pairs with the existing
  CountOf arm, but reads off the per-player tally directly.

### Tests (+18 net, 1261 → 1279)

- `tests::stx::*` — 13 new tests (one per new card); plus
  `first_day_of_class_pumps_token_creatures_only` covering the
  IsToken-only filter.
- `tests::modern::*` — 2 new tests for the lethal-first auto-target
  picker (`heated_debate_auto_target_prefers_lethal_kill`,
  `heated_debate_auto_target_falls_through_when_no_lethal`).
- `server::view::tests::*` — 1 new test
  (`predicate_short_label_covers_cards_drawn_and_permanent_count`).

## 2026-05-03 push XXXI: ManaSpentToCast + Opus/Increment shortcuts + EventKind::Blocks + 15 promotions

Mana-spent introspection lands as a first-class engine primitive,
unlocking the entire SOS Opus + Increment payoff cycle. Tests at
1261 (was 1246, +15 net).

### Engine improvements

- **`Value::ManaSpentToCast`** — new variant under `Value`, returns
  `cost.cmc() + x_value` of the just-cast spell located via
  `ctx.trigger_source = Card(cid)`. Parallel to push XXVII's
  `Predicate::CastSpellHasX` but exposes the actual mana figure rather
  than just a "has X" boolean. Returns 0 when the trigger source isn't
  a spell on the stack (e.g., the spell already resolved out from
  under the trigger, or the context is from a non-cast source).
- **`effect::shortcut::opus(at_least, big, always)`** — shorthand for
  the SOS Opus pattern: a magecraft trigger that always fires `always`
  and additionally fires `big` when `Value::ManaSpentToCast ≥
  at_least`. Used by Tackle Artist, Spectacular Skywhale, Muse Seeker,
  Deluge Virtuoso, Exhibition Tidecaller. The "instead" wording in
  printed Opus oracle is approximated as additive (both halves run on
  big casts) — combat-correct since the bigger payoff dominates.
- **`effect::shortcut::increment()`** — shorthand for the SOS
  Increment pattern: a SpellCast/YourControl trigger gated on
  `ValueAtLeast(ManaSpentToCast, Min(Power, Toughness) + 1)`. On
  match, adds a +1/+1 counter to the source. Used by Berta, Cuboid
  Colony, Fractal Tender, Hungry Graffalon, Pensive Professor,
  Tester of the Tangential, Textbook Tabulator.
- **`EventKind::Blocks`** — new event variant, fires from the
  blocker side of `GameEvent::BlockerDeclared`. Symmetric counterpart
  to `BecomesBlocked` (which fires from the attacker side). Backed
  by a small dispatcher branch that splits the SelfSource scope
  matching by event kind: `Blocks/SelfSource` reads `blocker ==
  source.id`, `BecomesBlocked/SelfSource` reads `attacker ==
  source.id`. Unblocks Daemogoth Titan's "or blocks" rider and any
  future "whenever ~ blocks" trigger.

### Card promotions to ✅ (15 total)

- **Tackle Artist** 🟡 → ✅ — Opus rider wired via `opus(5, ...)`.
  Cheap-cast: +1/+1 EOT pump. Big-cast (5+ mana spent): +1/+1
  permanent counter (in addition to the EOT pump).
- **Aberrant Manawurm** 🟡 → ✅ — printed "+X/+0 EOT where X is the
  mana spent to cast that spell" now wires via `Value::ManaSpentToCast`
  on a magecraft trigger. Bolt → +1/+0 EOT; Wisdom of Ages (CMC 7) →
  +7/+0 EOT.
- **Spectacular Skywhale** 🟡 → ✅ — Opus rider via `opus(5, ...)`.
  Always: +3/+0 EOT. Big-cast: 3 +1/+1 counters.
- **Muse Seeker** 🟡 → ✅ — Opus loot rider. Always: draw 1.
  Cheap-cast: discard 1 (gated on `ValueAtMost(ManaSpentToCast, 4)`).
- **Deluge Virtuoso** 🟡 → ✅ — Opus +1/+1 EOT pump on every IS cast,
  +1/+1 additional EOT pump on big casts (net +2/+2 EOT).
- **Exhibition Tidecaller** 🟡 → ✅ — Opus mill rider. Cheap-cast: opp
  mills 3. Big-cast: +10 mill (net 13).
- **Cuboid Colony** 🟡 → ✅ — Increment trigger now fires.
- **Pensive Professor** 🟡 → ✅ — Increment trigger now fires.
- **Tester of the Tangential** 🟡 → ✅ — Increment trigger now fires
  (combat pay-X-move-counters rider remains omitted).
- **Textbook Tabulator** 🟡 → ✅ — Increment + ETB Surveil 2.
- **Hungry Graffalon** 🟡 → ✅ — Increment now fires.
- **Fractal Tender** 🟡 → ✅ — Increment trigger wired (end-step
  Fractal-with-counters rider still omitted).
- **Berta, Wise Extrapolator** 🟡 → ✅ — Increment trigger now wired
  alongside the existing CounterAdded → AnyOneColor mana ramp.
  Increment + counter-driven mana ramp creates a self-feeding
  engine: any 2+ mana spell drops a counter on Berta, the counter
  triggers an AnyOneColor mana add.
- **Daemogoth Titan** 🟡 → ✅ — "or blocks" rider now wires via the
  new `EventKind::Blocks` event. Both attack-side and block-side
  triggers run the same body — sacrifice another non-titan creature.
- **Karok Wrangler** 🟡 → ✅ — Wizard-count rider now wires via
  `Effect::If` gated on `ValueAtLeast(CountOf(Wizard ∧
  ControlledByYou), 2)`. Single Karok → 1 stun; Karok next to any
  other Wizard → 2 stuns.

### UI improvements

- **`predicate_short_label`** in `server/view.rs` gained an arm for
  `Value::ManaSpentToCast`. Renders `ValueAtLeast(ManaSpentToCast,
  Const(5))` as "if 5+ mana spent" (matching the Augusta-style "if
  ≥N attackers" pattern from push XXX).

### Tests (+15)

- 9 new tests in `tests::sos::*` covering Aberrant Manawurm scaling,
  Tackle Artist Opus cheap/big branches, Spectacular Skywhale Opus,
  Cuboid Colony Increment positive/negative, Pensive Professor
  always-fires (P=0), Tester of the Tangential threshold (≤1=skip,
  ≥2=fire), Berta Increment + ramp chain, ManaSpentToCast outside
  spell context.
- 1 new test in `tests::stx::*` for Daemogoth Titan's block-side
  sacrifice (parallel to the existing attack-side test).
- 1 new test in `tests::stx::*` for Karok Wrangler's two-Wizard
  double-stun branch.
- 1 new test in `server::view::tests::*` for the Predicate label
  (`predicate_short_label_covers_mana_spent_to_cast`).
- 1 test signature update in `tests::sos::*`
  (`berta_wise_extrapolator_def_is_one_four_legendary_frog_druid` —
  triggered abilities now count 2 rather than 1).
- Test count net +15 (1246 → 1261).

## 2026-05-02 push XXX: STX 2021 expansion + AttackersThisCombat + 2 promotions

8 new STX 2021 cards + 2 promotions + new `Value::AttackersThisCombat`
primitive + `Effect::Attacks/AnotherOfYours` filter evaluation in
`combat.rs` + UI labels for the new primitive and And-composite stack
filters. Tests at 1241 (was 1227; +14 tests).

### Card additions (`catalog::sets::stx::*`)

#### Witherbloom (B/G) — `witherbloom.rs`

- **Mortality Spear** ✅ ({3}{B}{G} Sorcery — Lesson) — destroy target
  creature or planeswalker. Lesson sub-type recorded so future
  Lesson-aware code can filter on it. Same shape as Hero's Downfall on
  a Witherbloom-flavoured curve.
- **Dina, Soul Steeper** 🟡 → ✅ — the activated -X/-X EOT now scales
  with `Value::Diff(Const(0), CountOf(EachPermanent(Creature ∧
  ControlledByYou)))`. Three creatures-you-control yields -3/-3, five
  creatures yields -5/-5. Lifegain trigger unchanged.

#### Silverquill (W/B) — `silverquill.rs`

- **Dueling Coach** ✅ ({2}{W}, 3/3 Vigilance Human Cleric) — magecraft
  +1/+1 counter on target creature. Same shape as the existing
  Lecturing Scornmage / Stonebinder's Familiar magecraft-counter
  family on a meatier {3} body with Vigilance.
- **Hall Monitor** ✅ ({W}, 1/1 Human Wizard) — magecraft "target
  creature can't block this turn". Wired via `Effect::GrantKeyword`
  with `Keyword::CantBlock` (EOT) — same primitive Duel Tactics uses
  for its CantBlock rider. Auto-target picks the largest opposing
  blocker.
- **Karok Wrangler** 🟡 ({2}{W}, 3/3 Human Wizard) — ETB tap target
  opp's creature + stun counter. Same shape as Frost Trickster (the
  blue mono mainline) on a {2}{W} 3/3 frame without flash. The "if
  you control two or more Wizards, additional stun counter" rider is
  omitted (no SelectorCount-keyed branching inside triggered abilities
  yet).

#### Lorehold (R/W) — `lorehold.rs`

- **Hofri Ghostforge** 🟡 ({2}{R}{W}, 3/4 Legendary Human Cleric) —
  static anthem on other creatures you control (printed: "Other
  *nonlegendary* creatures"). Static-layer filter decomposition
  doesn't yet support `Not(HasSupertype(Legendary))` so the wider
  "Other creatures" anthem ships — minor false-positive on legendary
  friendly creatures. The dies-as-Spirit-copy rider is omitted
  (token-copy-of-permanent primitive gap, same as Phantasmal Image /
  Mockingbird).
- **Mascot Interception** 🟡 ({2}{R}{W} Instant) — printed "gain
  control + untap + haste" approximated as Destroy on a single opp
  creature. Engine has no `Effect::GainControl` primitive yet.
- **Approach of the Lorehold** ✅ ({1}{R}{W} Sorcery) — 2 damage to
  each opponent (auto-target collapse) + creates a 1/1 white Spirit
  with flying. Lorehold's flexible utility sorcery.
- **Augusta, Dean of Order** 🟡 → ✅ — the per-attacker pump trigger is
  now gated by `Predicate::ValueAtLeast(Value::AttackersThisCombat,
  Const(2))`. Single-attacker swings no longer false-positive
  (matches printed text); two-or-more attacker swings each get +1/+1
  + double strike EOT.

#### Quandrix (G/U) — `quandrix.rs`

- **Augmenter Pugilist** 🟡 ({3}{G}{G}, 6/6 Trample Human Warrior) —
  body + Trample only. The "activated abilities of creatures cost
  {2} more" static is omitted (no `StaticEffect::TaxActivatedAbilities`
  primitive yet — same gap as Trinisphere's "minimum cost" flavor in
  CUBE_FEATURES.md).

### Engine improvements

- **`Value::AttackersThisCombat`** — new primitive. Reads
  `state.attacking.len()`. Used by Augusta's "two or more attackers"
  gate via `Predicate::ValueAtLeast(AttackersThisCombat, 2)`.
  Unblocks Adriana, Captain of the Guard's "+1/+1 for each *other*
  attacking creature" pump (just `Diff(AttackersThisCombat, 1)`).
- **Filter evaluation on broadcast Attack triggers** (`combat.rs`).
  Pre-fix the `AnotherOfYours` / `YourControl` / `AnyPlayer`-scoped
  Attack broadcast collected `(source, effect, target)` tuples and
  pushed every trigger unconditionally, silently ignoring
  `EventSpec.filter`. Now a second pass evaluates each trigger's
  filter against an `EffectContext::for_trigger` after every
  attacker is in `self.attacking`, so `AttackersThisCombat`-keyed
  gates read the *final* count uniformly across all attackers
  (rather than off-by-one against declaration order). Augusta's
  symmetric pumps both fire when 2+ attack; neither fires when 1
  attacks.

### UI improvements

- **`predicate_short_label`** gained an arm for
  `Value::AttackersThisCombat` — formats Augusta-style gates as
  "if attacking" (≥1) / "if ≥N attackers" / "if ≤N attackers".
- **`entity_matches_label`** now collapses common And-composite
  filters: `IsSpellOnStack ∧ X` strips the "spell" qualifier;
  `ControlledByYou ∧ X` / `ControlledByOpponent ∧ X` collapse to
  "if your X" / "if opp's X". Powers Choreographed Sparks's "you
  control" stack-spell filter, Saw It Coming-style counter targets,
  and any "your creature" / "opp's artifact" matters.

### Tests

- 9 new card-functionality tests: Mortality Spear (destroy + Lesson
  flag), Dueling Coach (magecraft +1/+1 counter + Vigilance body),
  Hall Monitor (magecraft CantBlock grant), Karok Wrangler (ETB tap
  + stun), Approach of the Lorehold (2 dmg + flying Spirit token),
  Mascot Interception (destroy on opp creature), Hofri Ghostforge
  (anthem pumps via `computed_permanent`), Augmenter Pugilist (body
  sanity check), Dina, Soul Steeper (-X/-X scaling).
- 2 new Augusta tests: `_pumps_when_two_attackers` (≥2 gate passes,
  both attackers pump) + `_skips_pump_when_solo_attacker` (gate
  fails, lone attacker stays at base P/T).
- 2 new view tests:
  `entity_matches_label_covers_and_composite_filters` and
  `predicate_short_label_covers_attackers_this_combat`.

## 2026-05-02 push XXIX: Lorehold expansion + STX 2021 + UI + bugfix

10 new STX 2021 cards across schools + Abrupt Decay MV bug fix + UI
or-composite filter labels. Tests at 1227 (was 1218; +9 tests for new
cards + 1 abrupt-decay rejection-cap test + 1 view-or label test).

### Card additions (`catalog::sets::stx::*`)

#### Lorehold (R/W) — `lorehold.rs`

- **Rip Apart** ({R}{W} Sorcery) — modal removal: 3 dmg to creature/PW
  OR destroy artifact/enchantment. Wired with `Effect::ChooseMode`
  same shape as Boros Charm.
- **Plargg, Dean of Chaos** ({1}{R}, 1/3 Legendary Human Wizard) —
  rummage activation: `{T}: Discard a card, then draw a card`.
  🟡 The {2}{R} top-3-exile activation is omitted (no exile-from-top
  primitive yet).
- **Augusta, Dean of Order** ({1}{W}, 2/2 Legendary Vigilance Wizard)
  — per-attacker pump trigger: each attacker gets +1/+1 + double
  strike EOT via the `Attacks/AnotherOfYours` broadcast.
  🟡 The "two or more creatures attack" gate collapses to per-attack
  (no count-of-attackers Value primitive yet — same gap as Adriana).

#### Prismari (U/R) — `prismari.rs`

- **Magma Opus** ({7}{U}{R} Sorcery) — finisher: 4 dmg to creature/PW,
  create 4/4 Elemental token, draw 2.
  🟡 The "4 damage divided" + "tap two permanents" both collapse to
  single-target picks; the discard-for-Treasure alt cost is omitted.
- **Expressive Iteration** ({U}{R} Sorcery) — collapsed to Scry 2 +
  Draw 1 cantrip approximation. The "exile top 3 + play / cast from
  exile" rider is omitted (cast-from-exile primitive gap).

#### Mono-color staples — `mono.rs`

- **Environmental Sciences** ({2} Sorcery — Lesson) — colorless
  basic-land tutor + 2 life. Universal Lesson at every color.
- **Expanded Anatomy** ({3}{G} Sorcery — Lesson) — three +1/+1
  counters on a target creature.
- **Big Play** ({3}{G}{U} Instant — Lesson) — untap a creature, +1/+1
  + hexproof + trample EOT. 🟡 "up to two" collapses to single-target.
- **Confront the Past** ({4}{R} Sorcery — Lesson) — counter target
  ability. 🟡 "steal a planeswalker loyalty ability" mode is
  omitted.
- **Pilgrim of the Ages** ({3}{W}, 2/3 Spirit Wizard Cleric) — death-
  trigger basic-land recursion to hand.

### Engine improvement

- **Abrupt Decay bug fix** (`catalog::sets::decks::modern.rs`) — the
  target filter was `ManaValueAtMost(2)` but the printed Oracle text
  is "mana value 3 or less". Fix: `ManaValueAtMost(3)`. Reduced the
  rejection-cap test to swap Phyrexian Arena (CMC 3, now LEGAL) for
  Sun Titan (CMC 6) and added a new test
  `abrupt_decay_accepts_cmc_three_target`.

### UI improvement

- **`entity_matches_label` Or-composite arm**
  (`server::view::entity_matches_label`) — Or-composite predicates
  of two simple type tokens now render as "if A/B" rather than the
  catch-all "if matches filter". Covers Rip Apart's
  "creature/planeswalker" + "artifact/enchantment" filters, Magma
  Opus's "creature/planeswalker", Nature's Claim's "artifact/
  enchantment", any future binary-Or filter on basic type tokens.
  Recurses one level deep — three-way Or chains keep the generic
  hint. New helper `or_label` + `simple_type_token`. Test:
  `entity_matches_label_covers_or_composite_filters`.

### Tests (+9 net cards + 1 view + 1 modern)

- 11 new tests in `tests::stx::*`:
  - `rip_apart_mode_zero_deals_three_to_creature`
  - `rip_apart_mode_one_destroys_artifact`
  - `plargg_dean_of_chaos_rummages`
  - `augusta_dean_of_order_pumps_attacker`
  - `magma_opus_deals_damage_creates_token_and_draws`
  - `expressive_iteration_scrys_and_draws`
  - `environmental_sciences_searches_for_basic_and_gains_two_life`
  - `expanded_anatomy_puts_three_counters_on_creature`
  - `big_play_untaps_and_pumps_creature`
  - `confront_the_past_counters_an_ability_on_stack`
  - `pilgrim_of_the_ages_returns_basic_land_on_death`
- 1 new test in `tests::modern::*`:
  - `abrupt_decay_accepts_cmc_three_target`
- 1 new test in `server::view::tests::*`:
  - `entity_matches_label_covers_or_composite_filters`

## 2026-05-02 push XXVIII: trigger-subject threading

Engine improvement (no card additions). `PlayerRef::Triggerer` (and
`Selector::Player(Triggerer)`) now resolves to the actual event actor
at trigger resolution time, not the catch-all `Permanent(source)`
fallback. Pre-fix the dispatch path captured the subject for filter
evaluation and discarded it; the resolution path rebuilt context with
`trigger_source = Permanent(source)`. Now every `StackItem::Trigger`
push records the natural subject (ETB → entering permanent, Magecraft
→ cast spell, Dies → dying creature, attack → attacker, opponent's
draw → drawing player). New `subject: Option<EntityRef>` field on
`StackItem::Trigger` and `ResumeContext::Trigger` with
`#[serde(default)]` for snapshot back-compat. `EntityRef` gains
`Serialize` / `Deserialize`. **Sheoldred's drain** (push XXVII) now
uses exact Triggerer targeting instead of the EachOpponent collapse —
correct in 3+ player as well as 2-player.

## 2026-05-02 push XXVII: 6 more cards + UI EntityMatches label coverage

Card additions and a server view improvement. Tests at 1214 (was
1207).

### Card additions (`catalog::sets::decks::modern`)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Careful Study | {U} | ✅ | Sorcery. Draw 2, then discard 2. Net-zero hand size, filters two cards out. |
| Sheoldred, the Apocalypse | {2}{B}{B} | ✅ | 4/5 Legendary Phyrexian Praetor with Deathtouch + Lifelink. CardDrawn/YourControl → +2 life; CardDrawn/OpponentControl → drain 2 to drawing opponent (uses `PlayerRef::Triggerer` from push XXVIII). |
| Liliana of the Veil | {1}{B}{B} | 🟡 | Legendary Planeswalker — Liliana, base loyalty 3. +1 (each player discards 1) + -2 (target player sacs a creature) wired faithfully. -6 ult omitted (no two-pile-split primitive). |
| Light Up the Stage | {2}{R} | 🟡 | Sorcery. Approximated as Draw 2. Exile-and-may-play-this-turn rider omitted (cast-from-exile pipeline gap). Spectacle {R} alt cost also omitted. |
| Liliana of the Last Hope | {1}{B}{B} | 🟡 | Legendary Planeswalker — Liliana, base loyalty 3. +1 (-2/-1 EOT to creature) + -2 (return creature card from gy → hand) wired. -7 emblem omitted (no emblem zone). |
| Tibalt's Trickery | {1}{R} | 🟡 | Instant. Hard counter at {1}{R}. The chaotic exile-3 + cast-random-nonland cascade rider is omitted (cast-from-exile-without-paying primitive gap). |

### Server view improvements

`predicate_short_label` now unpacks `Predicate::EntityMatches`'s inner
filter for common simple cases — "if creature" / "if noncreature" /
"if artifact" / "if multicolored" / "if MV ≤2" / "if power ≥N" /
"if has counter" — instead of the generic "if matches filter" hint.
Composite (And / Or) predicates and counter-keyed filters keep the
generic fallback. Powers Esper Sentinel's "if noncreature" gate
badge and any future cast-trigger that filters by a single shape
predicate.

### Tests

- 7 new card-functionality tests in `tests::modern::*` (careful study,
  Sheoldred drain + life, Liliana +1/-2 each, Light Up the Stage,
  Liliana of the Last Hope -2, Tibalt's Trickery counter).
- 4 new EntityMatches label tests in `server::view::tests`.

## 2026-05-02 push XXVI: 10 new cube + STX cards + OpponentControl SpellCast dispatch

Engine improvement + 10 new card factories. Tests at 1207 (was
1195).

### Engine improvement

`fire_spell_cast_triggers` now walks every battlefield permanent's
SpellCast trigger and routes by scope. Pre-fix only the caster's
permanents were considered (`c.controller == caster`), which silently
ignored `EventScope::OpponentControl` triggers — Esper Sentinel,
Mindbreak Trap, future "whenever an opponent casts X" payoffs would
never fire. Now `YourControl` / `AnyPlayer` keep the caster-side path;
`OpponentControl` walks non-caster permanents and fires under the
*trigger's* controller (so the body resolves on the Sentinel's
controller, not the spell-caster).

### Card additions (`catalog::sets::decks::modern`)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Cabal Ritual | {B} | ✅ | Sorcery. +3{B} default; threshold ({GraveyardSizeOf(You) ≥ 7}) → +4{B}+{C}. |
| Rift Bolt | {2}{R} | 🟡 | Sorcery. 3 dmg to any target. Suspend 1—{R} omitted (no time-counter primitive yet). Ships at the printed full cost. |
| Ancient Stirrings | {G} | ✅ | Sorcery. Look at top 5; reveal colorless card → hand via `RevealUntilFind { find: Colorless, cap: 5 }`. Misses go to graveyard (engine default for `RevealUntilFind`). |
| Stinkweed Imp | {1}{B} | 🟡 | 1/3 Flying Imp. DealsCombatDamageToPlayer/SelfSource → mill 5. Dredge 5 omitted (no Dredge primitive). |
| Endurance | {1}{G}{G} | 🟡 | 3/4 Reach Flash Elemental Incarnation. ETB shuffle target player's gy into library. Evoke {2}{G} pitch omitted. |
| Esper Sentinel | {W} | 🟡 | 1/1 Human Advisor. Whenever opp casts a noncreature spell → you draw 1. Approximated as unconditional draw on every opp noncreature cast (was supposed to be once-per-turn + opp-may-pay-X). |
| Path of Peril | {2}{B}{B} | ✅ | Sorcery. ForEach Creature ∧ MV≤2 → -3/-3 EOT. Boltable creatures die outright. Boast omitted (no Boast). |
| Fiery Confluence | {2}{R}{R} | 🟡 | Sorcery. 3-mode `ChooseMode` (1 to each creature / 2 to each opp / destroy artifact). Printed "choose three with repetition" collapses to single-mode pick. |
| Brilliant Plan | {3}{U} | ✅ | STX 2021 mono-blue. Sorcery. Scry 3 + Draw 3. |
| Silverquill Apprentice | {W}{B} | 🟡 | STX 2021. 2/2 Human Cleric. Magecraft +1/+1 EOT to a creature. The W/B-mode pip choice (your creature +1/+1 vs opp creature -1/-1) collapses to +1/+1 only. |

### Tests

- 12 new tests in `tests::modern::*`.

## 2026-05-02 push XXV: STX Silverquill expansion + cube cards + bot/UI improvements

Card additions + non-blocking improvements. Tests at 1195 (was 1179, +16
net). Pure card additions + a smarter bot blocking heuristic + extended
UI predicate labels.

### Card additions

#### STX 2021 (`catalog::sets::stx::*`)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Star Pupil | {B} | 🟡 | 0/0 Spirit. ETB with two +1/+1 counters; dies → +1/+1 counter on target creature. Same approximation as Reckless Amplimancer / Body of Research: base 1/1 + ETB AddCounter +1/+1 ×1 (engine has no "enters with N counters" replacement primitive; a 0/0 base would die before the ETB trigger lands). Net effective body is 2/2 with one counter, matching the printed two-counters-on-a-0/0. The dies trigger is faithful — `EventKind::CreatureDied/SelfSource` → `Effect::AddCounter` on a targeted creature. |
| Codespell Cleric | {W} | ✅ | 1/1 Human Cleric, Lifelink. ETB Scry 1. All three pieces are first-class engine primitives. |
| Combat Professor | {3}{W} | ✅ | 2/3 Cat Cleric with Flying. Magecraft +1/+1 EOT on target creature (same shape as Eager First-Year, just on a 2/3 flier). |
| Spirit Summoning | {3}{W} | ✅ | Sorcery — Lesson. Creates a 1/1 white Spirit creature token with flying. White's slot in the STX Lesson cycle (siblings: Pest Summoning B/G, Inkling Summoning W/B, Mascot Exhibition W). |

#### Cube (`catalog::sets::decks::modern.rs`)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Kolaghan's Command | {B}{R} | 🟡 | Modal instant; printed "choose two" collapsed to "choose one" via `Effect::ChooseMode` (same approximation as Boros Charm / STX Commands). Each individual mode wired faithfully — gy-recursion (creature card → hand), opp discard (random), 2 dmg to creature/PW, destroy artifact. BR midrange staple. |
| Twincast | {U}{U} | 🟡 | Copy target instant or sorcery. `Effect::CopySpell` against a target filtered to `IsSpellOnStack ∧ (Instant ∨ Sorcery)`. The "may choose new targets" clause inherits the original's targets (no interactive re-target prompt). |
| Reverberate | {R}{R} | 🟡 | Functionally identical to Twincast at red. Same `Effect::CopySpell` wiring. Ships for cube color-pool diversity. |
| Vendetta | {B} | 🟡 | Destroy target nonblack creature. The "lose life equal to its toughness" rider collapses to a flat 2-life payment — `Value` doesn't yet have a "toughness of pre-destroy target" reader (the target is in the graveyard by the time the life-loss step would resolve). Same approximation gap as Bone Splinters' generic-cost. |
| Generous Gift | {2}{W} | ✅ | Destroy target nonland permanent + the destroyed card's controller creates a 3/3 green Elephant token. The token's controller resolves via `PlayerRef::ControllerOf(Target(0))` (graveyard-fallback path matches Harsh Annotation's Inkling rider — see SOS push XXI). |
| Crackling Doom | {R}{W}{B} | 🟡 | Each opp loses 2 life + sacrifices a creature. The "creature with greatest power" constraint isn't enforced — the engine's `Effect::Sacrifice` with a `Creature` filter delegates the pick to the targeted player (auto-decider picks the lowest power). Same gap as Pithing Edict's "creature or planeswalker" choice. The 2-damage half is faithful via `Selector::Player(EachOpponent)`. |

### Engine / server improvements

- **`pick_blocks` smarter heuristic** (`server/bot.rs`): pre-fix the bot
  threw every legal blocker into a random legal attacker — suicide
  blocks (1/1 vs 5/5) chewed through bodies for nothing. The new logic
  carries P/T + relevant keywords (flying/reach/deathtouch/
  indestructible) up-front and computes a `trade_score` per
  (attacker, blocker) pair: killing the attacker is the dominant
  payoff (+3 + power), losing a body is the cost (-1 - power).
  Damage-prevention is only counted when life is at risk
  (`add_blunting` flag toggled when life ≤ 5 after summed-attack).
  Greedy assignment: highest-power attackers first, best-scoring
  blocker per attacker, gated by a per-pressure-tier threshold
  (lethal=any, critical≥0, normal≥1). Net result: the bot stops
  suicide-blocking at high life and properly chumps under lethal
  pressure.

- **`predicate_short_label` Value-keyed coverage** (`server/view.rs`):
  added human-readable labels for `ValueAtLeast` / `ValueAtMost` over
  `GraveyardSizeOf` ("≥N in gy") / `LibrarySizeOf` ("≥N in library") /
  `CountOf(_)` ("if ≥N match" or "if board matches" at n=1) and a
  generic "if matches filter" for `EntityMatches`. Powers Dragon's
  Approach's "≥4 in gy" tutor gate (was the catch-all "conditional"),
  Resonating Lute's hand-size gate, and any future selector-count
  predicate that doesn't unpack the selector to the UI.

### Tests (+16 net, 1179 → 1195)

- 5 STX (`tests::stx::*`): Star Pupil ETB-counters + dies-counter rider,
  Codespell Cleric ETB-Scry + lifelink body, Combat Professor magecraft
  pump, Spirit Summoning Lesson token shape.
- 8 modern (`tests::modern::*`): Kolaghan's Command 4-mode shape +
  damage-mode + gy-recursion, Twincast copies a Bolt for 6 total to
  opponent, Vendetta destroy + 2 life loss, Reverberate body shape,
  Generous Gift destroy + Elephant token under opp control, Crackling
  Doom each-opp damage + sac.
- 2 server-side (`server::view::tests`,
  `server::bot::tests`): predicate_short_label Value-keyed coverage,
  pick_blocks suicide-block skip + lethal-pressure chump.
- 1 server bot (`bot_chump_blocks_when_lethal_imminent`).

## 2026-05-02 push XXIV: Witherbloom completion + cross-school Commands

Pure-card-additions + UI/bot polish. Extends the STX 2021 catalog with
the four "choose two" Commands (Lorehold / Prismari / Quandrix /
Silverquill) plus a Witherbloom completion pass (Daemogoth Titan, Pest
Infestation, Witherbloom Command), Saw It Coming, and two promotions
(Witherbloom Pledgemage 🟡 → ✅ via `life_cost: 1`, Hunt for Specimens
🟡 → ✅ for Lesson-approximation parity with Eyetwitch). Tests pass at
1179 (was 1159, +20 net): 18 new STX tests + 1 bot life-cost guard +
1 predicate-label plural.

### Card additions

- **Witherbloom completion** (3 cards new + 2 promotions):
  - **Daemogoth Titan** ({3}{B}{G}, 11/11 Demon Horror) — attack
    trigger sacrifices another creature via
    `EventKind::Attacks/SelfSource → Effect::Sacrifice` filtered to
    creatures-you-control. The "or blocks" rider is omitted (no
    `EventKind::Blocks` event yet).
  - **Pest Infestation** ({X}{B}{G} sorcery) — `Effect::CreateToken`
    with `count: Value::XFromCost` over the shared `stx_pest_token()`.
    Each minted Pest carries the on-die +1-life trigger via
    `TokenDefinition.triggered_abilities` (SOS push VI).
  - **Witherbloom Command** ({B}{G} instant) — 4-mode `ChooseMode`
    (drain 3 / gy → hand on permanent MV ≤ 3 / destroy enchantment /
    -3/-3 EOT). Printed "choose two" collapses to "choose one" — same
    approximation as Moment of Reckoning.
  - **Witherbloom Pledgemage** 🟡 → ✅ — `{T}, Pay 1 life: Add {B}` now
    uses `ActivatedAbility.life_cost: 1` (push XV primitive). Activation
    rejects pre-pay with `InsufficientLife` when life < 1, mirroring the
    mana-cost pre-pay check.
  - **Hunt for Specimens** 🟡 → ✅ (parity with Eyetwitch / Igneous
    Inspiration's Learn approximation). Token + Learn → Draw 1.

- **Cross-school Commands** (4 cards):
  - **Lorehold Command** ({R}{W}) — drain 4 / two 1/1 white Spirit
    tokens with flying / gy → hand on permanent MV ≤ 2 / exile target
    gy card.
  - **Prismari Command** ({1}{U}{R}) — 2 dmg / discard 2 + draw 2 /
    Treasure / destroy artifact.
  - **Quandrix Command** ({1}{G}{U}) — counter target activated
    ability / +1/+1 ×2 on creature / gy → bottom of owner's library /
    draw a card.
  - **Silverquill Command** ({2}{W}{B}) — counter activated/triggered
    ability / -3/-3 EOT / drain 3 / draw a card.
  - All four use `ChooseMode` for "choose one of N modes". Printed
    "choose two" collapses to "choose one" (same gap as Moment of
    Reckoning, Witherbloom Command).

- **Mono-color additions** (1 card):
  - **Saw It Coming** ({1}{U}{U}) — `counter_target_spell()` shortcut.
    Foretell {1}{U} alt-cost omitted (Foretell needs alt-cost-on-exile
    + cast-from-exile-with-time-limit primitives).

### Server / bot improvements

- **`is_free_mana_ability` tightened** — `server/bot.rs`. Push XXIV
  added `life_cost > 0` and `condition.is_some()` to the skip list (was
  only `tap_cost / sac_cost / mana_cost`). The bot now correctly
  refuses to fire Witherbloom Pledgemage's `{T}, Pay 1 life: Add {B}`
  as a "free" mana rock — paying life is a non-trivial cost the random
  bot can't reason about. Existing `condition` check covers Resonating
  Lute's 7-cards-in-hand gate, Potioner's Trove's instant/sorcery gate,
  etc.

- **`predicate_short_label` plural tally arms** — `server/view.rs`.
  Push XXIV added explicit `Value::Const(n)` formatters for n ≥ 2 on
  `CardsLeftGraveyardThisTurnAtLeast`, `LifeGainedThisTurnAtLeast`,
  `CardsExiledThisTurnAtLeast`, `CreaturesDiedThisTurnAtLeast` — was
  only n=1 covered, n>1 fell through to "conditional".

### Tests (+20 net, 1159 → 1179)

- `tests::stx::witherbloom_pledgemage_pays_one_life_for_mana`
- `tests::stx::witherbloom_pledgemage_rejects_when_life_too_low`
- `tests::stx::daemogoth_titan_is_an_eleven_eleven_demon_horror`
- `tests::stx::daemogoth_titan_attack_trigger_sacrifices_another_creature`
- `tests::stx::pest_infestation_at_x_three_creates_three_pest_tokens`
- `tests::stx::pest_infestation_pest_die_triggers_lifegain`
- `tests::stx::witherbloom_command_mode_zero_drains_three`
- `tests::stx::witherbloom_command_mode_two_destroys_enchantment`
- `tests::stx::lorehold_command_mode_zero_drains_four_life`
- `tests::stx::lorehold_command_mode_one_creates_two_flying_spirits`
- `tests::stx::prismari_command_mode_zero_deals_two_damage`
- `tests::stx::prismari_command_mode_two_creates_treasure`
- `tests::stx::quandrix_command_mode_one_adds_two_counters`
- `tests::stx::quandrix_command_mode_three_draws_a_card`
- `tests::stx::silverquill_command_mode_one_pumps_minus_three`
- `tests::stx::silverquill_command_mode_three_draws_a_card`
- `tests::stx::saw_it_coming_counters_target_spell`
- `tests::stx::hunt_for_specimens_promoted_pest_dies_trigger`
- `server::bot::tests::bot_does_not_tap_life_cost_mana_source`
- `server::view::tests::predicate_short_label_covers_plural_tally_thresholds`

## 2026-05-02 push XXIII: 18 new STX 2021 + cube cards + bot/UI improvements

Pure-card-additions push (no new engine primitives) — extends the STX
2021 catalog across Witherbloom, Lorehold, Prismari, Quandrix, and
mono-color staples with 12 new cards, plus 6 new modern cube cards in
`catalog::sets::decks::modern.rs`. Server bot now picks off opp
planeswalkers when killable, and `predicate_short_label` covers six
more `Predicate` variants (SelectorExists, SelectorCountAtLeast,
IsTurnOf, All/Any/Not/True/False). Tests pass at 1159 (was 1132,
+27 net).

### Card additions

#### STX 2021 (`catalog::sets::stx::*`)

- **Witherbloom**: Daemogoth Woe-Eater (9/9 Demon, sac-on-ETB +
  `{T}`: gain 4 life), Eyeblight Cullers (4/4 Elf with sac-on-ETB +
  drain 2), Dina, Soul Steeper (1/3 Legendary Deathtouch + lifegain-
  pings-opp). Same additional-cost approximation pattern as Vicious
  Rivalry (sac fires at ETB rather than at cast time).
- **Lorehold**: Reconstruct History (return up to 2 artifacts gy →
  hand via `Selector::take(_, 2)` + draw 1), Igneous Inspiration
  (3 dmg to creature/PW + Learn).
- **Prismari**: Creative Outburst (full discard via
  `Value::HandSizeOf(You)` + draw 5).
- **Quandrix**: Snow Day (Fractal token + counters scaled to hand
  size), Mentor's Guidance (draw 2 then put hand-size +1/+1 counters
  on a creature you control).
- **Mono-color** (`stx::mono`): Solve the Equation ({2}{U} tutor for
  instant/sorcery + scry 1), Enthusiastic Study ({1}{G} pump +
  trample + Learn), Tempted by the Oriq ({1}{W}{B} destroy ≤3-MV
  creature + Inkling token; printed "gain control" approximated as
  Destroy since `Effect::GainControl` doesn't have a static prompt
  primitive yet).

#### Modern cube (`catalog::sets::decks::modern.rs`)

- Boros Charm (3-mode `ChooseMode`: 4 dmg to PW / your permanents
  gain Indestructible EOT / target creature gains Double Strike EOT).
- Dragon's Rage Channeler (1/1 Human Shaman with Surveil 1 on
  noncreature cast — filter via `Predicate::EntityMatches +
  HasCardType(Creature).negate()`). Delirium body buff omitted.
- Unholy Heat ({R} Instant: 3 dmg to creature/PW; Delirium upgrade
  to 6 dmg omitted — same gap as DRC's static).
- Pelt Collector ({G} 1/1 Elf Warrior body — power-comparison
  ETB/death triggers + counter-gated trample static omitted).
- Frantic Inventory ({1}{U}: draw 1 + 1 per other named copy in gy
  via `Value::Sum + CountOf(CardsInZone(Graveyard, HasName))`; same
  shape as Slime Against Humanity from push XXII).
- Pegasus Stampede ({3}{W}: two 1/1 white Pegasus tokens with flying
  + Flashback `{6}{W}{W}`).

### Engine / server improvements

- **Bot planeswalker targeting** (`server/bot.rs`): the
  DeclareAttackers branch now scans opp planeswalkers, sorts them by
  loyalty ascending, and assigns its strongest attackers to walker
  targets first via a greedy first-fit accumulator. Once an attacker
  pool meets a walker's loyalty, advance to the next walker so an
  alpha-strike can pick off multiple walkers in one turn. Falls
  through to the previous Player(opponent) target when no walkers
  are on the board.
- **Predicate labels** (`server/view.rs`): `predicate_short_label`
  gained explicit arms for `SelectorExists`,
  `SelectorCountAtLeast { n }` (with separate `n=1` collapse and
  `n=k` "if ≥k match" form), `IsTurnOf`, `All`/`Any` boolean
  combinators (with empty-list collapses to "always"/"never"),
  `Not`, `True`, `False`. The catch-all "conditional" arm now only
  fires for genuinely unhandled cases.

### Tests

- 13 new STX 2021 functionality tests (`tests::stx::*`): body P/T
  sanity, ETB sac/drain/triggers, lifegain pings, gy-recursion target
  filtering, hand-size-scaled counter / discard, tutor flow, Learn
  draw rider, token mint counts.
- 9 new cube tests (`tests::modern::*`): modal mode-pick (Boros
  Charm), on-cast trigger filter (DRC), hard-removal damage, scaling
  card draw via gy-tally, Pegasus token mint.
- 5 new server-side tests (`server::view::tests`,
  `server::bot::tests`): predicate label coverage, bot walker-attack
  routing, walker-absent fallback.

All 1159 lib tests pass (was 1132, +27 net).

## 2026-05-02 push XXII: HasName predicate + Dragon's Approach + 17 cube cards

Engine improvement (`SelectionRequirement::HasName`) +
1 STX 2021 promotion + 17 brand-new cube cards (modern.rs) +
Rofellos rewire to scale with Forest count. Tests pass at 1132 (was
1110, +22 net).

### Engine improvement

- **`SelectionRequirement::HasName(Cow<'static, str>)`** — name-match
  predicate. Wired into both `evaluate_requirement` (battlefield path)
  and `evaluate_requirement_on_card` (library/graveyard path) so it
  works for both target prompts and CardsInZone counts. The `Cow`
  storage avoids allocating for `&'static str` literals at card
  construction while letting snapshot restore (which builds owned
  strings from JSON) avoid leaking. Powers Dragon's Approach +
  Slime Against Humanity + future "named X" payoffs.

### Card promotion

| Card | Status before → after | Notes |
|---|---|---|
| Dragon's Approach | 🟡 → ✅ | "if 4+ DA in graveyard, search for a Dragon" rider now wired via `Predicate::ValueAtLeast(CountOf(CardsInZone(Graveyard, HasName)), Const(4))` gating an `Effect::Search { Creature ∧ Dragon → Battlefield(untapped) }` branch. Auto-decider takes the value (collapses the "may"). |

### New cube cards (`catalog::sets::decks::modern`)

17 new card factories spanning W/U/B/R/G + colorless. Each card uses
existing engine primitives — no engine changes required. See the push
XXII section in `TODO.md` for the full per-card list.

### Tests

22 new functionality tests covering:
- ETB triggers on bouncers / draw-on-ETB / death triggers (Aether
  Adept, Wall of Omens, Mulldrifter, Resilient Khenra, Solemn
  Simulacrum)
- Token-mint-to-target-controller (Pongify, Rapid Hybridization)
- Recursion target validation (Sun Titan low-MV recur + high-MV no-op)
- Conditional damage scaling (Galvanic Blast metalcraft branching)
- Sac-counter activation (Cursecatcher)
- Forest-count mana scaling (Rofellos rewire)
- Card-name graveyard payoffs (Dragon's Approach gate-skip + tutor,
  Slime Against Humanity X scaling)

All 1132 lib tests pass.

## 2026-05-02 push XXI: Effect::CopySpell + Selector::CastSpellSource + 7 promotions

7 SOS card promotions to ✅ riding on the new copy-spell pipeline + 4
engine primitives. Tests pass at 1110 (was 1103, +7 net).

### Engine improvements

- **`Effect::CopySpell { what, count }`** — first-class implementation
  (was a stub). Resolves the `what` selector to a `CardId`, finds the
  matching `StackItem::Spell` on the stack, and pushes `count` copies
  back onto the stack with `is_copy: true`. Each copy shares the
  original's target, mode, x_value, and converged_value but gets a
  fresh `CardId`. The copy's controller is the source's controller
  (the listener that fired the trigger), matching MTG's "you may copy
  that spell" semantic. Permanent-spell copies are not supported in
  this first cut (would need a token-version path per rule 707.10b).

- **`StackItem::Spell.is_copy: bool`** — new field with
  `#[serde(default)]` for snapshot back-compat. Threaded into
  `continue_spell_resolution_with_face_copy` so a copy resolving
  doesn't go to the graveyard or exile (copies cease to exist per
  rule 707.10). The snapshot round-trip is verified by the new
  `stack_spell_is_copy_round_trips_through_snapshot` test.

- **`Selector::CastSpellSource`** — resolves to the topmost
  `StackItem::Spell` on the stack at trigger-resolution time. Since
  SpellCast triggers fire *above* the cast spell, the topmost
  remaining Spell at trigger-resolution time IS the just-cast spell
  whose event fired the trigger. Used by `Effect::CopySpell` to copy
  "that spell" without needing trigger_source plumbing.

- **`SelectionRequirement::ControlledByYou` / `ControlledByOpponent`
  fall through to stack-resident spells** — was battlefield-only.
  Now finds the spell on the stack (caster = controller) when the
  target is a stack-resident spell. Powers Choreographed Sparks's
  "target instant or sorcery spell *you control*" filter against a
  Lightning Bolt mid-resolution.

- **`push_on_cast_triggers` filter threading** —
  `collect_self_cast_triggers` now returns `(Effect, Option<Predicate>)`
  pairs and `push_on_cast_triggers` evaluates the filter against the
  cast spell as `trigger_source` before pushing. Powers Lumaret's
  Favor's "if you gained life this turn" Infusion gate without
  firing the copy trigger when the gate fails.

### Card promotions to ✅

| Card | School / Color | Status before → after | Notes |
|---|---|---|---|
| Aziza, Mage Tower Captain | Lorehold (R/W) | 🟡 → ✅ | Magecraft → MayDo body taps up to 3 untapped friendly creatures + copies the just-cast spell via `Selector::CastSpellSource`. |
| Mica, Reader of Ruins | Red | 🟡 → ✅ | Magecraft → MayDo body sacrifices a friendly artifact + copies the just-cast spell. |
| Lumaret's Favor | Green | 🟡 → ✅ | Mainline +2/+4 EOT pump + on-cast self-trigger gated on `LifeGainedThisTurnAtLeast(1)` that copies via `CopySpell`. |
| Silverquill, the Disputant | Silverquill (W/B) | 🟡 → ✅ | Casualty 1 grant approximated as a magecraft trigger that may-sacs a power-≥-1 creature to copy. |
| Social Snub | Silverquill (W/B) | 🟡 → ✅ | On-cast may-copy rider wired (filtered on `SelectorExists(Creature & ControlledByYou)`); copy resolves first then original. |
| Harsh Annotation | White | 🟡 → ✅ | Inkling token now created under the destroyed creature's controller via `PlayerRef::ControllerOf(Target(0))` (graveyard-fallback resolves the target's controller post-destroy). |
| Choreographed Sparks | Red | ⏳ → ✅ | NEW factory. Single-mode wire of "Copy target IS spell you control" via `CopySpell { what: target_filtered(IsSpellOnStack & ControlledByYou) }`. The "creature spell — gains haste, end-step sac" rider is omitted (no permanent-copy primitive). |

### Tests

8 new functionality tests in `tests::sos::*`:
- `aziza_magecraft_taps_three_creatures_and_copies_lightning_bolt`
- `aziza_magecraft_skipped_when_decider_says_no`
- `mica_magecraft_sacs_artifact_and_copies_lightning_bolt`
- `lumarets_favor_infusion_copies_pump_when_life_gained`
- `social_snub_copy_doubles_drain_when_decider_says_yes`
- `choreographed_sparks_copies_target_lightning_bolt`
- `stack_spell_is_copy_round_trips_through_snapshot` (snapshot)
- `harsh_annotation_destroys_and_creates_token` (strengthened to verify
  the Inkling lands on the destroyed creature's controller, not the
  caster)

All 1110 lib tests pass (was 1103, +7).

## 2026-05-02 push XX: STX 2021 expansion + Monocolored predicate + Beledros wire

19 new STX 2021 card factories + 1 engine primitive + 2 SOS/STX
🟡→✅ promotions. Tests pass at 1102 (was 1079, +23 new).

### Engine improvements

- **`SelectionRequirement::Monocolored`** — sibling to push VII's
  `Multicolored` and `Colorless`. Matches when a card's mana cost
  contains exactly one distinct colored pip (`distinct_colors() == 1`).
  Wired into both `evaluate_requirement` (battlefield/permanent) and
  `evaluate_requirement_on_card` (library/non-bf zones), so it works
  for both target prompts and library searches. Powers Vanishing
  Verse's "exile target nonland, monocolored permanent" exact-printed
  filter.

### Card promotions to ✅

| Card | School / Color | Status before → after | Notes |
|---|---|---|---|
| Vanishing Verse | Silverquill (STX) | 🟡 → ✅ | Target filter promoted to `Permanent ∧ Nonland ∧ Monocolored` via the new predicate. Two-color and colorless permanents now reject as invalid targets at cast time. |
| Beledros Witherbloom | Witherbloom (STX) | 🟡 → ✅ | "Pay 10 life: Untap each land you control. Activate only as a sorcery." now wired via push XV's `ActivatedAbility.life_cost: u32` gate (rejects with `InsufficientLife` < 10) + `Effect::Untap` over `Selector::EachPermanent(Land & ControlledByYou)`. Sorcery-speed flag matches printed restriction. |

### New STX 2021 cards (`catalog::sets::stx::mono`)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Pillardrop Warden | {2}{W} | 🟡 | 2/3 Spirit Cleric. ETB Scry 1. |
| Beaming Defiance | {1}{W} | ✅ | Instant: +1/+1 EOT and Hexproof EOT on target friendly creature. |
| Ageless Guardian | {1}{W} | 🟡 | 0/4 Spirit Wall with Defender + Vigilance. Becomes-attacker rider omitted. |
| Expel | {2}{W} | ✅ | Instant: exile target attacking or blocking creature. |
| Eureka Moment | {2}{U} | ✅ | Instant: untap target land + draw 2. |
| Curate | {1}{U} | ✅ | Instant: Surveil 2 + draw 1. |
| Skyswimmer Koi | {2}{U} | ✅ | 2/3 Fish; {4}{U}: +1/+1 EOT activated mana sink. |
| Stonebinder's Familiar | {U} | 🟡 | 1/2 Spirit. Approximates "permanent → graveyard" trigger as `EventKind::CreatureDied/AnyPlayer` (engine has no PermanentToGraveyard event yet). |
| Necrotic Fumes | {1}{B}{B} | 🟡 | Sorcery: sac a creature + exile target creature. Additional-cost-on-cast collapsed to in-resolution sac. |
| Specter of the Fens | {2}{B}{B} | ✅ | 3/3 Flying Specter; ETB mints a 1/1 black Pest with the standard die-→-gain-1 rider. |
| Ardent Dustspeaker | {3}{R} | ✅ | 3/3 Minotaur Shaman. Begin-combat trigger exiles up to one card from a graveyard. |
| Dragon's Approach | {1}{R} | ✅ | Sorcery: 3 damage to any target + "if 4+ Dragon's Approach in gy, search for a Dragon" rider now wired via the new `SelectionRequirement::HasName` predicate (push XXII). The "may" optionality collapses to always-tutor (auto-decider takes the value). |
| Bookwurm | {3}{G}{G} | ✅ | 4/5 Wurm; ETB gain 4 life + draw a card. |
| Spined Karok | {3}{G} | ✅ | 4/5 vanilla Beast (printed Wurm flavor; we use Beast since Wurm is reserved for Strixhaven-specific tribal hooks). |
| Field Trip | {2}{G} | ✅ | Sorcery — Lesson. Search library for a basic Forest, put it onto the battlefield tapped, then Scry 1. |
| Reckless Amplimancer | {2}{G} | 🟡 | Push XL: printed 0/0 body now uses the new `enters_with_counters` replacement (push XL) to land a +1/+1 counter per permanent you control at bf entry, before SBAs run. The "for each mana symbol on permanents you control" rider remains approximated as "per permanent" (no per-pip introspection primitive yet — that gap keeps the card 🟡 overall). |
| Square Up | {U}{R} | 🟡 | Instant: +0/+1 EOT pump on target friendly + fight an opp creature. Multi-target prompt collapsed to one chosen friendly + auto-picked enemy. |
| Thrilling Discovery | {1}{U}{R} | 🟡 | Instant: discard 1 + 2 life + draw 2. Additional-cost-on-cast collapsed to in-resolution discard. |
| Quandrix Cultivator | {3}{G}{U} | ✅ | 3/4 Elf Druid; ETB tutors two basic lands tapped (the printed "up to two" is approximated as exactly-two; second search no-ops if library is empty). |
| Quintorius, Field Historian | {2}{R}{W} | 🟡 | 3/4 Elephant Spirit. ETB exiles a graveyard card + creates a 3/2 R/W Spirit token. The "may pay {3}{R}{W} on gy-leave to make another Spirit" rider omitted. |

### Tests

22 new functionality tests in `tests::stx::*` exercising P/T,
ETB triggers, activated abilities, target filtering, and the
Beledros + Vanishing Verse promotions. All 1102 lib tests pass
(was 1079, +23 net).

## 2026-05-02 push XIX: Molten Note + Lorehold-closer + body-only batch

One engine-faithful Lorehold finisher (Molten Note — closes the last
⏳ row in the Lorehold school) + 10 ⏳→🟡 body-only / partial wires
across 5 schools + Colorless. Tests pass at 1079 (was 1063, +16 new).

**Three schools are now fully implemented (0 ⏳ rows): Witherbloom
(closed in push XV), Prismari (closed in push XIX via Elemental
Mascot), and Lorehold (closed in push XIX via Molten Note).**

- **Molten Note** ⏳ → ✅: full wire of Lorehold's last unresolved
  card. The "amount of mana spent" damage formula uses
  `Predicate::CastFromGraveyard` (push XVIII) to branch — hand cast
  deals `XFromCost + 2` (the X plus the {R}{W} portion); flashback
  cast deals 8 (the fixed {6}{R}{W} mana spent). Untap-all-your-
  creatures via `Effect::Untap` on `EachPermanent(Creature &
  ControlledByYou)`. Flashback {6}{R}{W} wired via
  `Keyword::Flashback`. Three new tests cover hand cast (X=3 → 5
  damage / X=0 → 2 damage) and flashback (8 damage + exile-on-
  resolution).

- **10 body-only / partial wires** (⏳ → 🟡):
  - **Strife Scholar** ({2}{R}, 3/2 Orc Sorcerer with `Keyword::
    Ward(2)`) — MDFC back face Awaken the Ages omitted.
  - **Campus Composer** ({3}{U}, 3/4 Merfolk Bard with `Keyword::
    Ward(2)`) — MDFC back face Aqueous Aria omitted.
  - **Elemental Mascot** ({1}{U}{R}, 1/4 Flying+Vigilance Elemental
    Bird with magecraft +1/+0 EOT pump on every IS cast). Opus
    exile-top-of-library half omitted (cast-from-exile gap).
  - **Biblioplex Tomekeeper** ({4}, 3/4 Construct artifact creature)
    — Prepare-state ETB toggle omitted.
  - **Strixhaven Skycoach** ({3}, 3/2 Flying artifact creature with
    ETB MayDo basic-land tutor) — Vehicle subtype + Crew keyword
    omitted.
  - **Skycoach Waypoint** (Land with `{T}: Add {C}`) — Prepare
    activation omitted.
  - **Silverquill, the Disputant** ({2}{W}{B}, 4/4 Flying+Vigilance
    Legendary Elder Dragon W/B) — Casualty 1 grant omitted (no
    copy-spell primitive).
  - **Quandrix, the Proof** ({4}{G}{U}, 6/6 Flying+Trample Legendary
    Elder Dragon G/U) — Cascade keyword omitted.
  - **Prismari, the Inspiration** ({5}{U}{R}, 7/7 Flying Legendary
    Elder Dragon U/R with `Keyword::Ward(5)` — printed Ward—Pay 5
    life approximated as flat mana Ward) — Storm grant omitted.
  - **Social Snub** ({1}{W}{B} Sorcery) — full wire on the mass-
    sacrifice + drain-1 halves; the on-cast may-copy rider is
    omitted (no copy-spell primitive).

- 16 new tests across the 11 cards (3 Molten Note + 13 body / mechanic
  checks). All 1079 lib tests pass (was 1063).

- Audit: 101/145/9 (✅/🟡/⏳) vs prior 100/135/20.

## 2026-05-02 push XVIII: combat-damage gy-broadcast + Predicate::CastFromGraveyard + body-with-Ward batch

Three engine wins + 5 new SOS cards + 4 promotions to ✅. Tests pass at
1063 (was 1050).

- **Combat-damage graveyard broadcast** — `fire_combat_damage_to_player_
  triggers` was extended to walk the attacker's controller's graveyard
  for `EventScope::FromYourGraveyard` triggers, in addition to the
  attacker's own SelfSource/AnyPlayer triggers. Two trigger families
  resolve here. Unblocks Killian's Confidence's "may pay {W/B} to
  return from graveyard to hand" gy-recursion clause.

- **`StackItem::Spell.face: CastFace`** + **`EffectContext.cast_face`**
  — push XIV's `CastFace` enum is now stamped onto the
  `StackItem::Spell` itself (with `#[serde(default)]` on the
  StackItemSnapshot field for snapshot back-compat) and threaded into
  `EffectContext.cast_face` at resolution time via the new
  `continue_spell_resolution_with_face` entry point. `cast_flashback`
  sets `pending_cast_face = Flashback` before delegating to
  `finalize_cast` (and restores after). All other paths default to
  `Front`.

- **`Predicate::CastFromGraveyard`** — reads `EffectContext.cast_face`
  and matches `CastFace::Flashback`. Powers Antiquities on the Loose's
  "Then if this spell was cast from anywhere other than your hand, put
  a +1/+1 counter on each Spirit you control" rider — the cast-from-gy
  branch now adds counters faithfully (verified by the new flashback
  cast test). Returns `false` in trigger / activated-ability contexts
  (those reset `cast_face` to `Front`).

- **5 new SOS cards** (3 fully wired, 2 body-only):
  - **Grave Researcher // Reanimate** ({2}{B} // {B}, MDFC). 3/3 Troll
    Warlock front with ETB Surveil 2; back-face Reanimate (target gy
    creature → BF + lose life equal to MV via `Value::ManaValueOf`).
  - **Emeritus of Ideation // Ancestral Recall** ({3}{U}{U} // {U},
    MDFC). 5/5 Human Wizard with `Keyword::Ward(2)` front; back-face
    Ancestral Recall (target player draws 3).
  - **Mica, Reader of Ruins** ({3}{R}, body-only 4/4 Legendary Human
    Artificer with `Keyword::Ward(3)`). IS-cast → may-sac → copy-spell
    rider omitted (no copy-spell primitive).
  - **Colorstorm Stallion** ({1}{U}{R}, 3/3 Elemental Horse with
    `Keyword::Ward(1)` + `Haste`). Magecraft +1/+1 EOT pump on every
    IS cast wired faithfully via `effect::shortcut::cast_is_instant_
    or_sorcery()`. Token-copy upper half omitted (no copy-permanent
    primitive).
  - **Killian's Confidence's gy-trigger** wired (the body's pump+draw
    was already there).

- **4 promotions to ✅**:
  - **Antiquities on the Loose** 🟡 → ✅: cast-from-gy +1/+1 rider
    wired via the new `Predicate::CastFromGraveyard`.
  - **Killian's Confidence** 🟡 → ✅: gy-recursion trigger wired via
    the combat-damage gy-broadcast + `Effect::MayPay({W/B})`.
  - **Colossus of the Blood Age** 🟡 → ✅: death rider was already
    correctly wired in push XVII; doc flip + source comment refresh.
  - Plus 4 doc-flips deferred from push XVII (Pursue the Past,
    Witherbloom Charm, Stadium Tidalmage, Heated Argument) — all
    correctly wired with `Effect::MayDo` from XV but still showing
    🟡. Now visible as ✅ in tables.

- **Server**: snapshot round-trip test added for the new
  `StackItem::Spell.face` field — verifies a `CastFace::Flashback`
  spell on the stack survives a serde_json round trip via
  `GameSnapshot`. View label "if cast from gy" added for
  `Predicate::CastFromGraveyard` so ability gates render the new
  condition in mouseover/tooltip text.

- 9 new tests: 3 Grave Researcher (front body / back reanimate /
  static fields), 2 Emeritus of Ideation (body / back Ancestral
  Recall), 1 Mica (Ward 3 body), 2 Colorstorm Stallion (body / magecraft
  pump), 1 Antiquities cast-from-hand vs cast-from-gy +1/+1 path, 1
  Killian's Confidence gy-trigger paid path, 1 Killian's Confidence
  declined path, 1 snapshot face round-trip. All 1063 lib tests pass
  (was 1050).

- Audit: 100/135/20 (✅/🟡/⏳) vs prior 97/134/24.

## 2026-05-01 push XVII: CardsDiscardedThisResolution + multi-card-promotions + STX 2021 expansion

Three engine wins + 4 SOS card promotions (🟡→✅) + 8 new STX 2021 card
factories. Tests pass at 1050 (was 1037).

- **`Value::CardsDiscardedThisResolution`** + sibling
  `Selector::DiscardedThisResolution(SelectionRequirement)` — per-
  resolution counters/id-list bumped by every `Effect::Discard` in
  the same `Effect::Seq` resolution. Reset on every entry to
  `resolve_effect`. Unblocks Borrowed Knowledge mode 1's "draw cards
  equal to the number of cards discarded this way" (previously
  collapsed to a flat-7 reload), Colossus of the Blood Age's
  "discard any number, draw that many plus one" death rider
  (previously a stubbed discard-1 / draw-2), and Mind Roots's "Put
  up to one land card discarded this way onto the battlefield
  tapped" (previously dropped entirely). Cards discarded by
  player-chosen `DiscardChosen` and random-discard
  (`Effect::Discard{ random: true }`) both feed the tally.

- **`resolve_zonedest_player` flatten-You fix** — the helper that
  pre-resolves selector-based `PlayerRef` in `ZoneDest` was only
  flattening `OwnerOf` / `ControllerOf`, leaving `PlayerRef::You`
  unresolved. This caused `place_card_in_dest` to mis-resolve `You`
  to the wrong seat when the source card was found in a different
  player's zone (Mind Roots's "discard from opp → land to *your*
  bf" silently routed the land to the opponent's battlefield). Now
  flattens every non-`Seat` variant via `resolve_player(ctx)`.

- **Combat-side broadcast for `EventKind::Attacks/AnotherOfYours`**
  — `declare_attackers` now consults all your permanents'
  `Attacks/AnotherOfYours` triggers, pre-binding the just-declared
  attacker as `Target(0)`. Unblocks Sparring Regimen's per-attacker
  +1/+1 fan-out. Self-source attack triggers (the attacker's own
  "Whenever this creature attacks") still fire via the existing
  per-attacker walk.

- **`Value::CountersOn` graveyard fallback** — extended the counter
  lookup to walk graveyards when the source card is no longer on
  battlefield. Promotes Scolding Administrator's death-trigger
  counter transfer (`If it had counters on it, put those counters
  on up to one target creature`) — the counters survive the
  battlefield-to-graveyard zone transition (engine only clears
  `damage`/`tapped`/`attached_to`), so the Value reads the right
  count off the graveyard-resident card.

### Card promotions / additions

| Card | School / Color | Status | Notes |
|---|---|---|---|
| Borrowed Knowledge mode 1 | Lorehold | 🟡 → 🟡 | Mode 1 now exact-printed via `Value::CardsDiscardedThisResolution`. Mode 0 unchanged. |
| Colossus of the Blood Age | Lorehold | 🟡 → 🟡 | Death rider now discards hand and draws cards-discarded+1 (was: discard-1 + draw-2). |
| Mind Roots | Witherbloom | 🟡 → 🟡 | "Put up to one land card discarded this way onto the battlefield tapped" now wired via the new selector. |
| Scolding Administrator | Silverquill | 🟡 → 🟡 | Death trigger transfers counters from dying card to target creature (oracle gap closed; gated on counter-count ≥ 1). |
| Sparring Regimen | Lorehold (STX) | 🟡 → 🟡 | "Whenever you attack, +1/+1 counter on each attacker" now fires per-attacker via the new combat-side broadcast. |

### New STX 2021 cards (`catalog::sets::stx::mono`)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Charge Through | {G} | ✅ | Sorcery: pump +1/+1 + grant trample EOT + draw a card. Cantripping combat enabler. |
| Resculpt | {1}{U} | ✅ | Instant: exile target artifact/creature; its controller mints a 4/4 blue Elemental token. |
| Letter of Acceptance | {3} | ✅ | Artifact: ETB Scry 1 + Draw 1; `{3}, Sacrifice this artifact: Draw a card` for late-game value. |
| Reduce to Memory | {2}{U} | 🟡 | Instant: exile target creature/PW; its controller mints an Inkling token (1/1 W/B Flying via the SOS helper). |
| Defend the Campus | {3}{R}{W} | 🟡 | Instant: -3/-0 EOT on a single attacking creature (multi-target prompt collapsed). |
| Conspiracy Theorist | {R} | 🟡 | 1/3 Human Shaman body; discard-recursion trigger omitted (cast-from-exile-with-time-limit). |
| Honor Troll | {2}{W} | 🟡 | 0/3 Troll body; conditional-lifelink rider omitted. |
| Manifestation Sage | {2}{G}{U} | 🟡 | 3/3 Flying Elf Wizard with Magecraft +(HandSize-3)/+(HandSize-3) EOT pump. |

## 2026-05-01 push XVI: CastSpellHasX + MayPay + HasXInCost + LibrarySizeOf

Six engine primitives + 10 SOS/STX card promotions. Five new
predicates/effects/values + a `CardsInZone(Hand)` filter-evaluation
fix that was silently breaking hand-source MayDo riders. Tests pass at
1025 (+13).

Engine improvements:

- **`Predicate::CastSpellHasX`** — cast-time introspection on the
  just-cast spell's `{X}` symbols. Used by Quandrix's "whenever you
  cast a spell with `{X}` in its mana cost" payoffs (Geometer's
  Arthropod, Matterbending Mage). Reads the cast spell's
  `card.definition.cost.has_x()` from `StackItem::Spell` via
  `ctx.trigger_source`.
- **`Effect::MayPay { description, mana_cost, body }`** — sibling to
  push XV's `Effect::MayDo`, but with a mana-cost payment. The
  controller is asked yes/no; on yes + sufficient pool, the engine
  deducts the mana and runs `body`. Decline / can't-afford skip the
  body silently. Powers Bayou Groff's printed "may pay {1} to return
  on death" rider, Killian's Confidence-style optional reanimation,
  and any future "may pay X to do Y" pattern with a pure-mana cost.
- **`SelectionRequirement::HasXInCost`** — card-level filter that
  matches if the card's printed mana cost contains an `{X}` pip.
  Wires Paradox Surveyor's "land OR card with {X} in cost" reveal
  filter to its exact-printed shape.
- **`Value::LibrarySizeOf(PlayerRef)`** — reads `players[p].library.
  len()` at evaluation time. Promotes Body of Research's "for each
  card in your library" Fractal scaling from the prior
  `GraveyardSizeOf` approximation to the printed exact predicate.
- **`shortcut::cast_has_x_trigger(effect)`** — Magecraft/Repartee-
  style helper for "whenever you cast a spell with {X}" payoffs.
  Bundles `EventKind::SpellCast / YourControl` with a `CastSpellHasX`
  filter.
- **`Selector::CardsInZone(Hand)` filter-evaluation fix** — was
  routing through `evaluate_requirement_static` (which only walks
  battlefield → graveyard → exile → stack), so any predicate against
  a hand-resident card silently returned false (e.g. Embrace the
  Paradox's `MayDo(Move(one_of(CardsInZone(Hand, Land)) → bf
  tapped))` couldn't see the lands in hand). Now routes through
  `evaluate_requirement_on_card`, the card-level evaluator that
  works for non-battlefield zones. Also benefits any future
  hand/library/exile-source `Move` chain.

Cards (10 promotions):

| Card | Status before | Status after |
|---|---|---|
| Geometer's Arthropod | ⏳ | ✅ (CastSpellHasX trigger fully wired) |
| Matterbending Mage | 🟡 | ✅ (X-cast trigger grants Unblockable EOT) |
| Paradox Surveyor | 🟡 | ✅ (Land OR HasXInCost reveal filter) |
| Embrace the Paradox | 🟡 | ✅ (MayDo land-from-hand → bf tapped) |
| Sundering Archaic | 🟡 | 🟡 ({2}: graveyard → bottom-of-library wired) |
| Aziza, Mage Tower Captain | ⏳ | 🟡 (body-only; copy-spell still gating ult) |
| Zaffai and the Tempests | ⏳ | 🟡 (body-only; once-per-turn free cast omitted) |
| Bayou Groff (STX) | 🟡 | ✅ (MayPay {1} → return-to-hand on death) |
| Felisa, Fang of Silverquill (STX) | 🟡 | ✅ (counter-bearing-creature dies → Inkling) |
| Body of Research (STX) | 🟡 | ✅ (LibrarySizeOf promotes from gy proxy) |

13 new tests in `tests::sos::*` and `tests::stx::*`. All 1025 lib
tests pass.

## 2026-05-01 push XV: MayDo primitive + Witherbloom MDFC closer + life-cost activations

Closes out the **Witherbloom (B/G) school** (Lluwen MDFC was the last ⏳)
and lands two long-tracked engine primitives — `Effect::MayDo` and
`ActivatedAbility.life_cost`. New cards: **Lluwen, Exchange Student //
Pest Friend** (Witherbloom MDFC), **Great Hall of the Biblioplex**
(legendary colorless utility land), **Follow the Lumarets** (G Infusion
sorcery). Promotions of "you may"-bearing 🟡 cards: Stadium Tidalmage,
Pursue the Past, Witherbloom Charm mode 0, Heated Argument, Rubble
Rouser; Erode now wires the basic-land tutor for the target's
controller.

Engine improvements:

- **`Effect::MayDo { description, body }`** — first-class "you may [body]"
  primitive. Emits a yes/no via `Decision::OptionalTrigger`; `AutoDecider`
  defaults to `Bool(false)` (skip), matching MTG's "you may defaults to no"
  rule. The `description` is a `String` (rather than `&'static str`)
  because `Effect` is bound to `Deserialize` via `GameState`'s serde
  derive. Walkers (`requires_target`, `primary_target_filter`,
  `target_filter_for_slot_in_mode`) recurse into the inner body.

- **`ActivatedAbility.life_cost: u32`** — additional life-payment cost on
  activations. Pre-flight gate rejects activation cleanly when controller
  has insufficient life (new `GameError::InsufficientLife`); life is
  paid up front after tap/mana succeed. Powers Great Hall of the
  Biblioplex's `{T}, Pay 1 life: Add one mana of any color` printed
  ability faithfully — the effect itself is a pure `AddMana`, so the
  ability still qualifies as a true mana ability and resolves
  immediately without going on the stack. New
  `ActivatedAbilityView.cost_label` rendering shows "Pay N life" tokens.

- **Erode's basic-land tutor for target's controller** — uses
  `PlayerRef::ControllerOf(Box::new(Selector::Target(0)))` for both the
  `Search.who` field and the `ZoneDest::Battlefield.controller` field, so
  the tutor and battlefield-place both target the correct (gy-resident)
  player. Unblocks the printed "its controller may search their library
  for a basic land" half.

Cards (3 new + 6 promotions = 9 SOS rows touched):

| Card | Status before | Status after |
|---|---|---|
| Lluwen, Exchange Student // Pest Friend | ⏳ | 🟡 |
| Great Hall of the Biblioplex | ⏳ | 🟡 |
| Follow the Lumarets | ⏳ | 🟡 |
| Stadium Tidalmage | 🟡 | 🟡 (may-loot now opt-in) |
| Pursue the Past | 🟡 | 🟡 (may-discard now opt-in) |
| Witherbloom Charm mode 0 | 🟡 | 🟡 (may-sac now opt-in) |
| Heated Argument | 🟡 | 🟡 (may-exile rider now opt-in) |
| Rubble Rouser | 🟡 | 🟡 (may-rummage now opt-in) |
| Erode | 🟡 | ✅ (basic-land tutor now wired) |

All 1012 lib tests pass.

## 2026-05-01 pushes XI/XII: 29 MDFCs + per-spell-type tallies + CastFace audit log

Two batches of MDFCs (modal-double-faced cards) under a new
`catalog::sets::sos::mdfcs` module, plus three engine improvements:

- **`enum CastFace { Front, Back, Flashback }`** — threaded through
  `GameEvent::SpellCast.face` + `GameEventWire::SpellCast.face`. Replays
  / spectator UIs can distinguish a back-face MDFC cast from a normal
  hand cast and from a graveyard-replay flashback cast. New transient
  `GameState.pending_cast_face` field; `cast_spell_back_face` sets it to
  `Back`, `cast_flashback` emits `Flashback` directly, default cast
  paths emit `Front`.
- **`Player.instants_or_sorceries_cast_this_turn`** +
  **`Player.creatures_cast_this_turn`** — refines
  `spells_cast_this_turn` (which counts every spell type) so cards that
  explicitly gate on the IS or creature subset can target the
  exact-printed predicate. Backed by two new predicates:
  `Predicate::InstantsOrSorceriesCastThisTurnAtLeast` and
  `Predicate::CreaturesCastThisTurnAtLeast`. Surfaced through
  `PlayerView` for client UIs (with `#[serde(default)]` for snapshot
  back-compat).
- **Potioner's Trove** — gate promoted from the proxy
  `SpellsCastThisTurnAtLeast(You, 1)` → exact
  `InstantsOrSorceriesCastThisTurnAtLeast(You, 1)` matching the printed
  Oracle text.

Push XI (17 MDFCs, all 🟡):

| Card | Color | Front | Back |
|---|---|---|---|
| Elite Interceptor // Rejoinder | W | 1/2 Human Wizard | Counter target creature spell |
| Emeritus of Truce // Swords to Plowshares | W | 3/3 Cat Cleric | Exile + lifegain via `PlayerRef::ControllerOf` |
| Honorbound Page // Forum's Favor | W | 3/3 Cat Cleric | +1/+1 EOT + 1 life |
| Joined Researchers // Secret Rendezvous | W | 2/2 Human Cleric Wizard | Draw 3 (collapsed to caster) |
| Quill-Blade Laureate // Twofold Intent | W | 1/1 Human Cleric | +1/+1 EOT + Inkling token |
| Spiritcall Enthusiast // Scrollboost | W | 3/3 Cat Cleric | Fan-out +1/+1 counter on each creature you control |
| Encouraging Aviator // Jump | U | 2/3 Bird Wizard w/ Flying | Grant Flying EOT |
| Harmonized Trio // Brainstorm | U | 1/1 Merfolk Bard Wizard | Draw 3 + put 2 on top |
| Cheerful Osteomancer // Raise Dead | B | 4/2 Orc Warlock | Return creature card from gy → hand |
| Emeritus of Woe // Demonic Tutor | B | 5/4 Vampire Warlock | Search library for any card → hand |
| Scheming Silvertongue // Sign in Blood | B | 1/3 Vampire Warlock | Target player draws 2 + loses 2 |
| Adventurous Eater // Have a Bite | B | 3/2 Human Warlock | -3/-3 EOT |
| Emeritus of Conflict // Lightning Bolt | R | 2/2 Human Wizard | 3 dmg to any target |
| Goblin Glasswright // Craft with Pride | R | 2/2 Goblin Sorcerer | +2/+0 + Haste EOT |
| Emeritus of Abundance // Regrowth | G | 3/4 Elf Druid | Return any card from gy → hand |
| Vastlands Scavenger // Bind to Life | G | 4/4 Trample Bear Druid | Return up to 2 creatures from gy |
| Leech Collector // Bloodletting | B | 2/2 Human Warlock | Drain 2 |
| Pigment Wrangler // Striking Palette | R | 4/4 Orc Sorcerer | 2 dmg to any target |

Push XII (12 more MDFCs, all 🟡):

| Card | Color | Front | Back |
|---|---|---|---|
| Spellbook Seeker // Careful Study | U | 3/3 Bird Wizard w/ Flying | Draw 2 + discard 2 |
| Skycoach Conductor // All Aboard | U | 2/3 Bird Pilot w/ Flying | Bounce target creature to owner's hand |
| Landscape Painter // Vibrant Idea | U | 2/1 Merfolk Wizard | Draw 3 |
| Blazing Firesinger // Seething Song | R | 2/3 Dwarf Bard | Add `{R}{R}{R}{R}{R}` |
| Maelstrom Artisan // Rocket Volley | R | 3/2 Minotaur Sorcerer | 2 dmg each opp + 2 dmg to opp creature |
| Scathing Shadelock // Venomous Words | B | 4/6 Snake Warlock w/ Deathtouch | -2/-2 EOT |
| Infirmary Healer // Stream of Life | G | 2/3 Cat Cleric w/ Lifelink | Gain X life |
| Jadzi, Steward of Fate // Oracle's Gift | U | Legendary 2/4 Human Wizard w/ Flying | Draw 2X cards |
| Sanar, Unfinished Genius // Wild Idea | U/R | Legendary 0/4 Goblin Sorcerer | Draw 3 |
| Tam, Observant Sequencer // Deep Sight | G/U | Legendary 4/3 Snake Wizard w/ Deathtouch | Scry 4 + Draw 1 |
| Kirol, History Buff // Pack a Punch | R/W | Legendary 2/3 Vampire Cleric w/ Lifelink | 3 dmg to creature |
| Abigale, Poet Laureate // Heroic Stanza | W/B | Legendary 2/3 Bird Bard w/ Flying | +2/+2 EOT + Lifelink |

Cube color pool wiring:
- White: + 6 white MDFCs.
- Blue: + 6 blue MDFCs (incl. Jadzi).
- Black: + 6 black MDFCs.
- Red: + 5 red MDFCs.
- Green: + 3 green MDFCs.
- Cross-pools: Sanar (UR), Tam (GU), Kirol (RW), Abigale (WB).

Test additions: 42 new tests in `tests::sos::*`. All 997 lib tests
pass.

## 2026-05-01 push X: Flashback wirings, Selector::Take, MDFC back-face cast

5 new SOS card factories (4 🟡 + 1 ✅), 4 promotions from 🟡 to ✅, plus
three engine primitives:

- **`Selector::Take { inner, count }`** — wraps another selector to
  clamp how many entities flow through (in resolution order). Primary
  payoff is "select up to N from a graveyard / library / hand": it
  promotes Practiced Scrollsmith's gy-exile from "every matching
  noncreature/nonland card" to "exactly one"; lifts Pull from the
  Grave's gy-recursion from one card to two; available for future SOS
  Heated Argument-style "may exile a card" wraps. Sugar:
  `Selector::one_of(inner)` and `Selector::take(inner, n)`.

- **`GameAction::CastSpellBack`** + **`cast_spell_back_face`** —
  generalises `PlayLandBack` to non-land MDFC back faces. Mirrors the
  PlayLandBack flow: swaps the in-hand card's `definition` to the
  back face's, then routes through the regular `cast_spell` path so
  cost / type / target filters / effect all resolve against the back
  face. Unblocks the SOS MDFC cycles whose backs are
  creatures/instants/sorceries (Studious First-Year // Rampant
  Growth wired as the first one; the rest of the cycle is wireable
  one-by-one as oracle text becomes available). The 3D client picks
  this up automatically: the right-click flip on hand cards now
  routes flipped non-land MDFCs through `CastSpellBack` (in addition
  to the existing `PlayLandBack` for land MDFCs). New
  `TargetingState.back_face_pending` flag carries the routing through
  the targeting prompt for back-face spells that need a target.

- **`Keyword::Flashback` wirings on 7 SOS cards** — Daydream, Dig
  Site Inventory, Practiced Offense, Antiquities on the Loose,
  Pursue the Past, Tome Blast, Duel Tactics. The engine's existing
  `cast_flashback` path replays each card's body identically when
  cast from graveyard for the flashback cost; the result is exiled
  on resolution.

| Card | School / Color | Status | Note |
|---|---|---|---|
| Daydream | White | ✅ (was 🟡) | Flashback {2}{W} now wired via `Keyword::Flashback`. The flicker pattern + counter rider already worked. |
| Dig Site Inventory | White | ✅ (was 🟡) | Flashback {W} now wired. |
| Tome Blast | Red | ✅ (was 🟡) | Flashback {4}{R} now wired. |
| Duel Tactics | Red | ✅ (was 🟡) | Flashback {1}{R} now wired (mainline ping + transient `CantBlock` from push VI). |
| Studious First-Year // Rampant Growth | Green (MDFC) | ✅ (was ⏳) | First non-land MDFC. Front: 1/1 vanilla Bear Wizard at {G}; back: Rampant Growth ({1}{G} basic-land tutor). The new `GameAction::CastSpellBack` casts the back face. |
| Inkshape Demonstrator | White | 🟡 (was ⏳) | Body + `Keyword::Ward(2)` wired (Ward keyword tagged for future enforcement). Repartee body wired faithfully via `repartee()` shortcut: source +1/+0 + Lifelink (EOT). |
| Fractal Tender | Quandrix | 🟡 (was ⏳) | Body + `Keyword::Ward(2)` wired. Increment trigger and end-step Fractal-with-counters payoff omitted. |
| Thornfist Striker | Green | 🟡 (was ⏳) | Body + `Keyword::Ward(1)` wired. Infusion continuous static omitted. |
| Lumaret's Favor | Green | 🟡 (was ⏳) | Mainline +2/+4 EOT pump wired faithfully. Infusion copy half omitted (no copy-spell primitive). |
| Practiced Scrollsmith | Lorehold | 🟡 | ETB now exiles **exactly one** matching noncreature/nonland card via the new `Selector::Take(_, 1)`. May-cast-until-next-turn rider still omitted. |
| Pull from the Grave | Black | 🟡 | Returns up to **two** creature cards (was: one) via `Selector::Take(_, 2)`. Lifegain unchanged. |
| Antiquities on the Loose | White | 🟡 | Flashback {4}{W}{W} now wired; cast-from-elsewhere counter rider still omitted. |
| Pursue the Past | Lorehold | 🟡 | Flashback {2}{R}{W} now wired; "may discard" optionality still collapsed. |
| Practiced Offense | White | 🟡 | Flashback {1}{W} now wired; lifelink-or-DS mode pick still collapsed. |

Cube color pool updates:
- White: + Inkshape Demonstrator
- Green: + Studious First-Year, Thornfist Striker, Lumaret's Favor
- G/U (Quandrix): + Fractal Tender

Test additions: 14 new tests in `tests::sos::*` covering the new
primitives (Selector::Take, CastSpellBack), Flashback keyword presence,
Flashback graveyard-replay (Daydream + Pursue the Past), and per-card
body shape (Inkshape Demonstrator, Fractal Tender, Thornfist Striker,
Lumaret's Favor, Studious First-Year MDFC). All 953 lib tests pass.

## 2026-05-01 push IX: Witherbloom finisher + Surveil-anchored cards + creatures-died tally

12 new SOS card factories (5 ✅, 7 🟡) plus one new engine primitive,
finishing the Witherbloom (B/G) school except for the Lluwen MDFC
(blocked on cast-from-secondary-face plumbing):

- **`Player.creatures_died_this_turn` + `Predicate::CreaturesDiedThisTurnAtLeast`**
  — per-turn tally bumped from the SBA dies handler in `stack.rs`
  (lethal-damage path) and from `remove_to_graveyard_with_triggers`
  (destroy-effect path). Reset to 0 in `do_untap`. Surfaced on
  `PlayerView.creatures_died_this_turn` so a UI can hint
  "Witherbloom end-step ready". Powers Essenceknit Scholar's
  end-step gated draw.

| Card | School / Color | Status | Note |
|---|---|---|---|
| Essenceknit Scholar | Witherbloom | ✅ (was ⏳) | ETB Pest token (with on-attack lifegain rider) + end-step gated draw via the new `Predicate::CreaturesDiedThisTurnAtLeast`. Hybrid `{B/G}` pip approximated as `{B}`. |
| Professor Dellian Fel | Witherbloom | 🟡 (was ⏳) | New `PlaneswalkerSubtype::Dellian` + 5 base loyalty. +2 (gain 3 life), 0 (draw 1 / lose 1 life), -3 (destroy target creature) all wired faithfully. The -7 emblem ult is omitted (no emblem zone yet). |
| Unsubtle Mockery | Red | ✅ (was ⏳) | 4-to-creature + Surveil 1. Surveil is a first-class engine primitive (the script's `COMPLEX_KWS` heuristic was stale). |
| Muse's Encouragement | Blue | ✅ (was ⏳) | Mints a 3/3 U/R Flying Elemental token + Surveil 2. Same Surveil-already-shipped fix. |
| Prismari Charm | Prismari | ✅ (was ⏳) | 3-mode: Surveil 2 + draw / 1 dmg to creature-or-PW / bounce nonland to owner. Single-target collapse on mode 1 (printed "one or two targets" — multi-target gap). |
| Textbook Tabulator | Blue | 🟡 (was ⏳) | 0/3 Frog Wizard + ETB Surveil 2. Increment rider omitted (mana-spent introspection). |
| Deluge Virtuoso | Blue | 🟡 (was ⏳) | 2/2 Human Wizard + ETB tap+stun against target opp creature. Opus +1/+1-or-+2/+2 rider omitted. |
| Moseo, Vein's New Dean | Black | 🟡 (was ⏳) | 2/1 Flying Bird Skeleton Warlock + ETB Pest token. Infusion end-step rider omitted (oracle truncated; no MayDo per-turn-lifegain primitive). |
| Stone Docent | White | 🟡 (was ⏳) | 3/1 Spirit body. Graveyard-exile activated ability omitted (engine activated-ability walker only iterates the battlefield — same gap as Eternal Student, Summoned Dromedary). |
| Page, Loose Leaf | Colorless | 🟡 (was ⏳) | 0/2 Legendary Construct artifact creature + `{T}: Add {C}` mana ability. Grandeur (discard-named-this-card) ability omitted (no card-name-as-cost activation). |
| Ral Zarek, Guest Lecturer | Black | 🟡 (was ⏳) | 3 base loyalty + +1 Surveil 2 / -1 each opp discards 1 (single-target collapse) / -2 return ≤3 MV creature card from your gy → bf. -7 coin-flip emblem omitted. |
| Flow State | Blue | 🟡 (was ⏳) | Approximated as `Scry 3 + Draw 1`. The conditional "instead pick 2 to hand" upgrade rider when both an instant and sorcery sit in your gy is omitted (no "look-and-distribute-by-count" primitive). |

Cube color pool updates:
- White: + Stone Docent
- Blue: + Deluge Virtuoso, Flow State, Muse's Encouragement, Textbook Tabulator
- Black: + Moseo Vein's New Dean, Ral Zarek Guest Lecturer
- Red: + Unsubtle Mockery
- Witherbloom (B/G): + Essenceknit Scholar, Professor Dellian Fel
- Prismari (U/R): + Prismari Charm

Test additions: 17 new tests in `tests::sos::*` (ETB triggers, end-step
gated draws, planeswalker loyalty activations, Surveil card resolution,
plus a tally-bumps-on-lethal-damage SBA test). All 932 lib tests pass.

## 2026-05-01 push VIII: Lesson cycle + Berta's CounterAdded(SelfSource) + ActivatedAbility.condition

14 new card factories (2 ✅, 12 🟡) bridging the Lesson cycle
(Decorum Dissertation, Restoration Seminar, Germination Practicum)
and a handful of body-only converge / Increment / Repartee bodies.
Engine ships two new primitives unblocking these cards (and several
others):

- **`ActivatedAbility.condition: Option<Predicate>`** — first-class
  "activate only if …" gating. Evaluated against the controller/
  source context **before** any cost is paid, so a failed gate doesn't
  burn the tap-cost or once-per-turn budget. Powers Resonating Lute's
  `{T}: Draw a card. Activate only if you have seven or more cards in
  your hand.` (gate: `ValueAtLeast(HandSizeOf(You), 7)`). Promotes
  Potioner's Trove's lifegain ability — `{T}: You gain 2 life.
  Activate only if you've cast an instant or sorcery spell this turn.`
  — to its printed gate (`SpellsCastThisTurnAtLeast(You, 1)`, an
  approximation of the printed "instant or sorcery"-only filter; the
  engine's per-turn spell tally tracks all spells today). New
  `GameError::AbilityConditionNotMet` for failed gates.
- **`EventScope::SelfSource` for `EventKind::CounterAdded`** —
  `event_card`/`SelfSource` extended to recognise CounterAdded events.
  Berta, the Wise Extrapolator's "whenever one or more +1/+1 counters
  are put on Berta, add one mana of any color" trigger now fires only
  when counters land on Berta (not on every +1/+1 counter on every
  permanent). Same hook will unblock Heliod-style "whenever a counter
  is put on this …" payoffs in future sets.

| Card | School / Color | Status | Note |
|---|---|---|---|
| Primary Research | White | ✅ (was ⏳) | ETB return Nonland & ManaValueAtMost(3) gy → bf + end-step gated draw via `Predicate::CardsLeftGraveyardThisTurnAtLeast`. |
| Artistic Process | Red | ✅ (was ⏳) | Three modes wired: 6-to-creature, 2-to-each-opp-creature (via `Selector::EachPermanent(Creature & ControlledByOpponent)`), Elemental + transient haste via `LastCreatedToken`. |
| Decorum Dissertation | Black (Lesson) | 🟡 (was ⏳) | Mode 0 wired (you draw 2, lose 2 life — collapses target-player to caster). Paradigm rider omitted. |
| Restoration Seminar | White (Lesson) | 🟡 (was ⏳) | Mode 0 wired (return Nonland gy → bf). Paradigm rider omitted. |
| Germination Practicum | Green (Lesson) | 🟡 (was ⏳) | `ForEach Creature & ControlledByYou → AddCounter +1/+1 ×2` fan-out. Paradigm rider omitted. |
| Ennis, Debate Moderator | White | 🟡 (was ⏳) | 1/1 body + ETB flicker (`Exile + DelayUntil(NextEndStep, Move(Target → Battlefield(OwnerOf)))`) + end-step gated counter via `CardsLeftGraveyardThisTurnAtLeast` proxy. |
| Tragedy Feaster | Black | 🟡 (was ⏳) | 7/6 Trample Demon body. Ward—Discard rider + Infusion sac-unless-lifegain rider both omitted. |
| Forum Necroscribe | Black | 🟡 (was ⏳) | 5/4 Troll Warlock body + Repartee gy-creature-recursion (`repartee()` chained with `Move(target Creature → bf)`). Ward—Discard rider omitted. |
| Berta, Wise Extrapolator | Quandrix | 🟡 (was ⏳) | 1/4 Legendary Frog Druid + CounterAdded(+1/+1, SelfSource) → `AddMana(AnyOneColor)` trigger + X-cost Fractal-token activation (X resolves to 0 today). Increment rider omitted. |
| Paradox Surveyor | Quandrix | 🟡 (was ⏳) | 3/3 Reach Elf Druid + ETB `RevealUntilFind(IsBasicLand, cap 5)`. Hybrid {G/U} approximated as {G}. |
| Magmablood Archaic | Red | 🟡 (was ⏳) | 2/2 Trample+Reach Avatar + Converge ETB `AddCounter(Value::ConvergedValue)`. IS-cast pump rider omitted (per-cast converge introspection). Hybrid `{2/R}` approximated as `{2}+{R}` per pip. |
| Wildgrowth Archaic | Green | 🟡 (was ⏳) | 0/0 Trample+Reach Avatar + Converge ETB AddCounter. Hybrid `{2/G}` approximated as `{2}+{G}` per pip. The "creature spells you cast enter with X extra counters" rider is omitted. |
| Ambitious Augmenter | Green | 🟡 (was ⏳) | 1/1 Turtle Wizard at {G}. Increment + dies-with-counters → Fractal-with-counters omitted. |
| Resonating Lute | Prismari | 🟡 (was ⏳) | {T}: Draw a card with new `ActivatedAbility.condition: ValueAtLeast(HandSizeOf(You), 7)` gate. Lands-grant tap-for-2 omitted. |

Cube color pool updates:
- White: + Primary Research, Restoration Seminar, Ennis the Debate Moderator
- Black: + Decorum Dissertation, Tragedy Feaster, Forum Necroscribe
- Red: + Artistic Process, Magmablood Archaic
- Green: + Germination Practicum, Wildgrowth Archaic, Ambitious Augmenter
- G/U (Quandrix): + Berta the Wise Extrapolator, Paradox Surveyor
- U/R (Prismari): + Resonating Lute

Test additions: 22 new tests in `tests::sos::*` covering the new
factories' primary play patterns + the new Resonating Lute and
Potioner's Trove condition gates (positive + negative cases). All
910 lib tests pass.

## 2026-05-01 push VII: Multicolored predicate + MDFC bodies + Lorehold capstone

10 new card factories (3 ✅, 7 🟡) plus 2 promotions (Owlin Historian
🟡 → ✅, Postmortem Professor — kept 🟡 but now carries the printed
`Keyword::CantBlock`). Engine ships two new primitives:

- **`SelectionRequirement::Multicolored`** + **`Colorless`** — count
  the *distinct* colored pips in a card's mana cost. Hybrid pips
  (`{W/B}`) contribute both halves; Phyrexian (`{B/P}`) contributes
  the colored half; generic / colorless / Snow / X don't count. Cost
  ≥ 2 distinct ⇒ multicolored; cost = 0 distinct ⇒ colorless.
  Powered by the new `ManaCost::distinct_colors()` helper.

- **`tap_add_colorless()` shared land helper** — `{T}: Add {C}` mana
  ability shorthand under `catalog::sets::mod`. Used by Petrified
  Hamlet (and ready for Wastes / future Eldrazi-flavoured colorless
  lands).

| Card | School / Color | Status | Note |
|---|---|---|---|
| Mage Tower Referee | Colorless | ✅ (was ⏳) | Trigger filtered on `EntityMatches(TriggerSource, Multicolored)`. |
| Additive Evolution | Green | ✅ (was ⏳) | ETB Fractal token + 3 +1/+1 counters via `Selector::LastCreatedToken`; begin-combat counter+vigilance pump. |
| Owlin Historian | White | ✅ (was 🟡) | Now wires the cards-leave-graveyard +1/+1 EOT pump via `EventKind::CardLeftGraveyard`. |
| Spectacular Skywhale | Prismari | 🟡 (was ⏳) | 1/4 Flying body. Opus rider omitted. |
| Lorehold, the Historian | Lorehold | 🟡 (was ⏳) | 5/5 Flying+Haste Legendary Elder Dragon body. Miracle grant + opp-upkeep loot omitted. |
| Homesickness | Blue | 🟡 (was ⏳) | Draw 2 (you) + Tap target creature + Stun 1. Multi-target prompts collapsed. |
| Fractalize | Blue | 🟡 (was ⏳) | `PumpPT(+(X+1), +(X+1)) EOT` — base-(X+1)/(X+1) rewrite collapsed. |
| Divergent Equation | Blue | 🟡 (was ⏳) | Single-target return one IS card from graveyard to hand. |
| Rubble Rouser | Red | 🟡 (was ⏳) | ETB rummage (collapsed `you may` to always-do). Activated `{T}, Exile from graveyard` omitted. |
| Zimone's Experiment | Green | 🟡 (was ⏳) | RevealUntilFind(creature → hand) + Search(IsBasicLand → bf tapped) approximation. |
| Petrified Hamlet | Colorless | 🟡 (was ⏳) | `{T}: Add {C}` only. Choose-name prompt + name-keyed lock-out omitted. |
| Postmortem Professor | Black | 🟡 (was 🟡) | Now carries `Keyword::CantBlock`; gy recursion still omitted. |

Cube color pool updates:
- Blue: + Divergent Equation, Fractalize, Homesickness
- Red: + Rubble Rouser
- Green: + Additive Evolution, Zimone's Experiment
- U/R (Prismari): + Spectacular Skywhale
- R/W (Lorehold): + Lorehold, the Historian

Test additions: 11 new tests in `tests::sos::*` covering Homesickness,
Fractalize, Divergent Equation, Spectacular Skywhale (def shape),
Lorehold the Historian (def shape), Mage Tower Referee (multicolored
+ mono-color cast), Rubble Rouser, Additive Evolution (ETB +
combat), Zimone's Experiment, Petrified Hamlet, Owlin Historian's
new pump trigger, Postmortem Professor's CantBlock keyword. Plus
3 mana-cost tests covering `ManaCost::distinct_colors`. All 885
lib tests pass.

## 2026-05-01 push VI: Lorehold completion + token-side triggers + ManaPayload::OfColor

12 new card factories + ~12 new functionality tests under
`tests::sos::*`. Engine ships three new primitives unblocking these
cards (and several more):

- **`TokenDefinition.triggered_abilities`** — token definitions now
  carry triggered abilities. `token_to_card_definition` copies them
  through, so the SOS Pest token's "Whenever this token attacks, you
  gain 1 life" rider and the STX Pest token's "When this creature
  dies, you gain 1 life" rider both fire. Witherbloom payoffs (Pest
  Mascot's lifegain → +1/+1 counter, Blech's per-creature-type counter
  fan-out, Bogwater Lumaret, Cauldron of Essence's death drain triple-
  loop) get the printed lifegain trickle for free. Promoted Send in
  the Pest, Pestbrood Sloth, Pest Summoning, Tend the Pests, Hunt for
  Specimens (their tokens now print correctly).
- **`ManaPayload::OfColor(Color, Value)`** — fixed-color, value-scaled
  mana adder. Single AddMana call produces N pips of a specified
  color. Powers Topiary Lecturer's "{T}: Add G equal to power"
  (replaces the prior `Repeat × Colors([Green])` approximation), and
  is ready for any future power-scaled mana ability (Llanowar Mentor,
  Wirewood Channeler, Cryptolith Rite-style scaling).
- **`Keyword::CantBlock`** — "this creature can't block" as a first-
  class keyword. Enforced inside `declare_blockers` and the
  `can_block_*` helpers. Used by Duel Tactics's "1 damage + can't
  block this turn" pump and the static restriction on Postmortem
  Professor (which can now be promoted).
- **`move_card_to` library traversal** — `Effect::Move` from a
  `Selector::TopOfLibrary` source now actually moves the top library
  card to the destination. Previously the library branch in
  `move_card_to` was missing, so Suspend Aggression's "exile the top
  card of your library" half silently no-op'd. The library-source
  move is now last in the search order (battlefield → graveyard →
  exile → hand → library) to avoid accidentally consuming a hand card
  with the same id.

| Card | School / Color | Status | Note |
|---|---|---|---|
| Daydream | White | 🟡 (was ⏳) | Restoration-Angel-style flicker pattern (`Exile + Move(target → bf) + AddCounter +1/+1`). Flashback half omitted. |
| Soaring Stoneglider | White | 🟡 (was ⏳) | 4/3 Flying-Vigilance Elephant Cleric at the **paid** cost path: full {3}{W}. Alt cost (exile two from gy) omitted. |
| Tome Blast | Red | 🟡 (was ⏳) | 2-to-any-target burn. Flashback half omitted. |
| Duel Tactics | Red | 🟡 (was ⏳) | 1-to-creature ping + new `Keyword::CantBlock` (EOT). Flashback half omitted. |
| Snarl Song | Green | ✅ (was ⏳) | Two Fractal tokens, each stamped with X +1/+1 counters via `Selector::LastCreatedToken`, plus X life. X = `Value::ConvergedValue`. |
| Wild Hypothesis | Green | ✅ (was ⏳) | Fractal token + X +1/+1 counters + Surveil 2 (all first-class primitives). |
| Topiary Lecturer | Green | 🟡 (was ⏳) | Now uses `ManaPayload::OfColor(Green, PowerOf(This))` — single AddMana, value-scaled count. Increment rider still omitted. |
| Ark of Hunger | Lorehold | 🟡 (was ⏳) | `EventKind::CardLeftGraveyard` drain trigger + {T}: Mill 1. May-play-from-mill rider omitted. |
| Suspend Aggression | Lorehold | 🟡 (was ⏳) | Exile target nonland permanent + exile top of library (library traversal added to `move_card_to`). May-play rider omitted. |
| Wilt in the Heat | Lorehold | 🟡 (was ⏳) | 5-to-creature. Cost reduction + die-replace-with-exile rider omitted. |
| Practiced Scrollsmith | Lorehold | 🟡 (was ⏳) | First strike body + ETB exiles every matching noncreature/nonland in your gy (no `Selector::OneOf` primitive yet). May-cast-until-next-turn rider omitted. Hybrid `{R/W}` approximated as `{R}`. |
| Send in the Pest | Black | ✅ (was 🟡) | Token-side attack-trigger lifegain wired. |
| Pestbrood Sloth | Green | ✅ (was 🟡) | Token-side attack-trigger lifegain wired. |

Cube color pool updates:
- White: + Daydream, Soaring Stoneglider
- Red: + Tome Blast, Duel Tactics
- Green: + Snarl Song, Wild Hypothesis, Topiary Lecturer
- R/W (Lorehold): + Ark of Hunger, Wilt in the Heat

## 2026-04-30 push V: CardLeftGraveyard event + Lorehold/mono-color batch

12 new card factories + 13 new functionality tests under
`tests::sos::*` (+1 in `tests::modern::*` for the new Untap cap).
Engine ships three new primitives unblocking these cards (and others
in the future):

- **`EventKind::CardLeftGraveyard`** + `GameEvent::CardLeftGraveyard`
  — fires per card removed from a graveyard. Plumbed in:
  `move_card_to`'s graveyard branch, `cast_spell_flashback` in
  actions.rs, and persist/undying battlefield-returns in stack.rs.
  Each emission also bumps the new
  `Player.cards_left_graveyard_this_turn` tally (reset on `do_untap`),
  which `Predicate::CardsLeftGraveyardThisTurnAtLeast` reads. Surfaced
  through `PlayerView` so client UIs can render "Lorehold ready"
  hints.
- **`Predicate::SpellsCastThisTurnAtLeast`** — gates Burrog Barrage's
  conditional pump on "have you already cast another spell this turn".
- **`Effect::Fight { attacker, defender }`** — proper bidirectional
  fight primitive. Snapshots both creatures' powers up-front so the
  back-swing isn't affected by the first hit. No-ops cleanly when
  either selector resolves to no permanent (matches MTG's "if either
  is no longer a creature, no damage is dealt"). Used by Chelonian
  Tackle; future cards (Decisive Denial mode 1, fight-style green
  removal) can drop in trivially.
- **`Effect::Untap.up_to: Option<Value>`** — untap-with-cap. Frantic
  Search's "untap up to three lands" now honors the printed cap
  precisely (previously the engine collapsed it to "untap all"). Other
  Untap callers (Cryptolith Rite, mass-untap effects) opt-out via
  `up_to: None`.

| Card | School / Color | Status | Note |
|---|---|---|---|
| Hardened Academic | Lorehold | ✅ (was 🟡) | Cards-leave-gy → +1/+1 counter on a friendly creature. Triggers per-card (the printed "one or more" wording is approximated by per-card emission). |
| Spirit Mascot | Lorehold | ✅ (was 🟡) | Self +1/+1 counter on every gy-leave event. |
| Garrison Excavator | Red mono / Lorehold | ✅ (was 🟡) | 2/2 R/W Spirit token on every gy-leave event. |
| Living History | Red mono / Lorehold | 🟡 (was ⏳) | ETB Spirit token + on-attack +2/+0 to attacker if a card left your graveyard this turn (gated on the new per-turn tally). |
| Witherbloom, the Balancer | Witherbloom | 🟡 (was ⏳) | 5/5 Legendary Elder Dragon (new `CreatureType::Elder`); Flying + Deathtouch. Affinity-for-creatures cost reduction stub (engine work tracked in TODO.md). |
| Burrog Barrage | Green mono | 🟡 (was ⏳) | Conditional pump (gated on `Predicate::SpellsCastThisTurnAtLeast`) + power-as-damage to target. |
| Chelonian Tackle | Green mono | 🟡 (was ⏳) | +0/+10 EOT pump + `Effect::Fight` against auto-selected opp creature (single-target collapse on the defender pick). |
| Rabid Attack | Black mono | 🟡 (was ⏳) | +1/+0 friendly pump (multi-target prompt + die-to-draw rider omitted). |
| Practiced Offense | White mono | 🟡 (was ⏳) | Fan-out +1/+1 to friendly creatures + double-strike grant on a target. Mode-pick (DS vs lifelink) collapsed to DS. |
| Mana Sculpt | Blue mono | 🟡 (was ⏳) | Counter target spell + wizard-mana-refund rider (fixed +{C}{C} as mana-spent introspection unavailable). |
| Tablet of Discovery | Red mono | 🟡 (was ⏳) | ETB-mill + `{T}: Add {R}` mana abilities (may-play-from-mill rider + spend-restriction omitted). |
| Steal the Show | Red mono | 🟡 (was ⏳) | Modal sorcery: discard-then-draw (collapsed to "discard 2, draw 2") OR damage = #IS-cards-in-your-gy to target creature/PW. |
| Frantic Search | Modern (cube) | ✅ upgrade | Untap-up-to-three now precise (was "untap all") via the new `Effect::Untap.up_to` cap. |

Cube color pool updates:
- White: + Practiced Offense
- Blue: + Mana Sculpt
- Black: + Rabid Attack
- Red: + Tablet of Discovery, Garrison Excavator, Living History, Steal the Show
- Green: + Burrog Barrage, Chelonian Tackle
- R/W (Lorehold): + Hardened Academic, Spirit Mascot
- B/G (Witherbloom): + Witherbloom, the Balancer

## 2026-04-30 push IV: post-push-III modern_decks batch

10 new card factories + 11 new functionality tests under
`tests::sos::*`. Engine ships five new primitives unblocking these
cards (and others in the future):

- **`Value::Pow2(Box<Value>)`** — two raised to a value, capped at
  exponent 30. Powers Mathemagics's "draw 2ˣ cards".
- **`Value::HalfDown(Box<Value>)`** — half of a value, rounded down.
  Powers Pox Plague's "loses half their life / discards half / sacs
  half" clauses.
- **`Value::PermanentCountControlledBy(PlayerRef)`** — counts the
  permanents controlled by the resolved player. Lets Pox Plague's
  per-player iteration compute the right "half their permanents"
  count under a `ForEach` over each player.
- **`Selector::CastSpellTarget(u8)`** — resolves the chosen target
  slot of the just-cast spell whose `SpellCast` event produced the
  current trigger. Walks the stack for the topmost matching spell.
  Used by Conciliator's Duelist's Repartee exile half (the body
  pulls the target off the cast spell rather than choosing a fresh
  target). Future Repartee-exile-bounce-counter spells get this for
  free.
- **`AffectedPermanents::AllWithCounter`** — counter-filtered
  lord-style statics. `affected_from_requirement` recognises
  `SelectionRequirement::WithCounter(...)` and routes through the
  new variant. Powers Emil's "creatures with +1/+1 counters have
  trample" + future "monstrous / leveled creatures gain
  [keyword]" buffs.

| Card | School / Color | Status | Note |
|---|---|---|---|
| Mathemagics | Blue | ✅ | Draw 2ˣ cards via `Value::Pow2(XFromCost)`; multi-target collapsed to "you draw". |
| Visionary's Dance | Prismari | ✅ | Two 3/3 flying Elemental tokens (new `elemental_token()` helper); discard activation from hand omitted. |
| Pox Plague | Black | ✅ | `ForEach Player` body using `HalfDown(Life/Hand/PermanentCount)` for each clause. |
| Emil, Vastlands Roamer | Green | ✅ | Static GrantKeyword(Trample) filtered to creatures with +1/+1 counters via `AllWithCounter`; tap+{4}{G} fractal-token activation scaling on lands. |
| Orysa, Tide Choreographer | Blue | ✅ | ETB draw 2; conditional cost reduction omitted. |
| Conciliator's Duelist | Silverquill | 🟡 | Repartee exile of cast spell's target via `Selector::CastSpellTarget(0)`; "return at next end step" rider still omitted. |
| Abstract Paintmage | Prismari | 🟡 | Trigger fires {U}{R} on PreCombatMain step; spend restriction omitted. |
| Matterbending Mage | Blue | 🟡 | ETB bounce target creature; "spell with X" trigger omitted. |
| Exhibition Tidecaller | Blue | 🟡 | 0/2 body wired; Opus mill rider omitted. |
| Colossus of the Blood Age | Lorehold | 🟡 | ETB drain wired; death "discard any number" collapsed to discard 1 / draw 2. |

Cube color pool updates:
- Blue: + Mathemagics, Matterbending Mage, Orysa Tide Choreographer,
  Exhibition Tidecaller
- Black: + Pox Plague
- Green: + Emil, Vastlands Roamer
- U/R (Prismari): + Visionary's Dance, Abstract Paintmage
- R/W (Lorehold): + Colossus of the Blood Age

Earlier counts in this section quoted 86 ✅ / 38 🟡 / 138 ⏳; that
header drift was reconciled when `scripts/gen_strixhaven2.py` was
re-run with full Scryfall oracle text. Some rows that had been claimed
✅ on a stub (no implementation in the SOS module) were corrected back
to 🟡 or ⏳, and several ⏳ cards from the body-only batch were
correctly reclassified to 🟡.

## 2026-04-30 push II: SOS Increment / Opus body-only batch

13 cards bumped from ⏳ → 🟡 by shipping the printed cost / type / P/T /
keywords without their Increment / Opus / mana-spent-pump rider. Each
rider depends on a "mana-paid-to-cast introspection on cast" engine
primitive (tracked in TODO.md). The vanilla bodies are still useful: they
fill out cube color pools, take combat correctly, and can be promoted to
✅ once the engine grows the right hooks. 11 functionality tests in
`tests::sos::*` exercise the bodies (P/T, keywords, ETB/attack triggers
where relevant).

Plus one new ✅-functionality card with omitted-rider note:
- **Ajani's Response** {4}{W} — destroy creature; cost-reduction-when-
  target-tapped omitted (logged as a TODO under "target-aware cost
  reduction").

New CreatureTypes: `Dwarf` (Thunderdrum Soloist, Scolding
Administrator), `Badger` (Shopkeeper's Bane), `Salamander` (Noxious
Newt), `Giraffe` (Hungry Graffalon).

Plus a follow-up Silverquill card: **Scolding Administrator** {W}{B} 2/2
Dwarf Cleric — Menace + Repartee +1/+1 counter on self (the truncated
"When this creature dies, …" trigger is unimplemented pending an
oracle-fetch refresh).

Cube color pool updates:
- White: + Ajani's Response
- Blue: + Pensive Professor, Tester of the Tangential, Muse Seeker
- U/G: + Cuboid Colony
- Red: + Tackle Artist, Thunderdrum Soloist, Molten-Core Maestro,
  Expressive Firedancer
- Green: + Aberrant Manawurm, Hungry Graffalon
- Black: + Eternal Student, Postmortem Professor

## 2026-04-30 push: Silverquill / Lorehold / mono-color expansion

11 new cards bridging the Silverquill (W/B) school's gap and a
handful of cross-school removal/utility staples, all wired entirely on
existing primitives (no engine work needed). Plus 11 functionality
tests in `tests::sos::*`.

| Card | School | Status | Note |
|---|---|---|---|
| Moment of Reckoning | Silverquill | ✅ | Modal destroy / graveyard return — "choose up to four" collapsed to single mode pick. |
| Stirring Honormancer | Silverquill | ✅ | ETB look-at-X-find-creature via `RevealUntilFind`. |
| Conciliator's Duelist | Silverquill | 🟡 | ETB draw + each-player-loses-1; Repartee exile-with-return rider omitted. |
| Dissection Practice | Black mono | 🟡 | Drain 1 + creature -1/-1; +1/+1 mode collapsed (multi-target gap). |
| Heated Argument | Red mono | 🟡 | 6 to creature + 2 to controller; "may exile" optionality dropped. |
| End of the Hunt | Black mono | 🟡 | Exile opponent's creature/PW; "greatest mana value" picker not enforced. |
| Vicious Rivalry | Witherbloom | ✅ | Push XLIII: now uses cast-time `additional_life_cost: Some(Value::XFromCost)` primitive — life is debited before the spell goes on the stack. Body is the `ForEach.If(ManaValueAtMost X)` mass destroy. |
| Proctor's Gaze | Quandrix | ✅ | Bounce nonland + Search basic to bf tapped. |
| Lorehold Charm | Lorehold | ✅ | All three modes (sac-artifact / return ≤2-mv / +2/+1). |
| Borrowed Knowledge | Lorehold | 🟡 | Mode 0 wired faithfully; mode 1 collapsed to "draw 7" (no track-discarded primitive). |
| Planar Engineering | Green mono | ✅ | Sacrifice 2 lands + Repeat×4 Search basic to bf tapped. |

Cross-pool wiring updated in `cube.rs`:
- W/B (Silverquill): added Moment of Reckoning, Stirring Honormancer,
  Conciliator's Duelist
- B/G (Witherbloom): added Vicious Rivalry
- G/U (Quandrix): added Proctor's Gaze
- R/W (Lorehold): added Borrowed Knowledge, Lorehold Charm

Mono-color pools picked up Dissection Practice, End of the Hunt
(Black), Heated Argument (Red), Planar Engineering (Green).

## Oracle re-fetch (2026-04-30)

`scripts/gen_strixhaven2.py` no longer truncates oracle text (was 600
chars, now unlimited). 52 SOS rows whose oracle column was previously
clipped have been tagged **🔍 needs review (oracle previously
truncated)** in the Notes column. Re-running the script against fresh
Scryfall pages (`sos_p1.json`, `sos_p2.json`) will replace the
truncated bodies with the full oracle, but until that happens the
🔍 marker flags every row whose status / engine-gap analysis was
based on incomplete text. When implementing one of these cards the
first step is to fetch the full oracle from Scryfall and verify the
existing Notes are still accurate.

First-pass coverage focuses on Silverquill (W/B) and Witherbloom (B/G)
plus the easier mono-color removal/utility cards. School lands
(Forum of Amity, Titan's Grave, Fields of Strife, Paradox Gardens) are
all wired since the surveil primitive is already in the engine. Quandrix
(G/U) and Prismari (U/R) now have first-pass coverage too — Pterafractyl,
Fractal Mascot, Mind into Matter, Growth Curve, Quandrix Charm,
Stadium Tidalmage, Vibrant Outburst, Stress Dream, and Traumatic Critique
are wired. Remaining SOS gaps are mostly **Repartee** (spell-targets-
creature predicate), **Increment** / **Opus** (mana-paid introspection on
cast), **Casualty** (copy-spell primitive), and **Flashback / Paradigm**
(cast-from-graveyard pipeline).

**X-cost ETB triggers** now correctly read the spell's paid X via
`StackItem::Trigger.x_value`. Pterafractyl's "enters with X +1/+1
counters" and Static Prison's "enters with X stun counters" both
honour the cast-time X (previously the trigger context defaulted to
`x_value: 0`).

**Infusion** ("if you gained life this turn, …") is now wired engine-side
via `Predicate::LifeGainedThisTurnAtLeast`, with **Foolish Fate**,
**Old-Growth Educator**, and **Efflorescence** as the first three
beneficiaries. Remaining Infusion cards (Tenured Concocter,
Ulna Alley Shopkeep, Tragedy Feaster, Withering Curse, …) need either
Ward enforcement or static "as long as you've gained life this turn"
gating which is a separate engine primitive.

### Prepare mechanic (SOS)

The Prepare mechanic has two halves that interact. The first ships
today; the second is still ⏳.

#### Half 1 — Prepared cards (the spell side)

A "prepared card" is a creature whose front face carries a vanilla
(or near-vanilla) body and whose back face is a fully-castable
**prepare spell** — usually a real reprinted spell (Careful Study,
Lightning Bolt, Raise Dead, …). The owner picks which face to cast
when the card leaves their hand, identical to the engine's existing
MDFC plumbing (`back_face: Some(Box<CardDefinition>)` + the
`GameAction::CastSpell` / `CastSpellBack` action pair). Mechanically
this is just an MDFC; the **distinguishing feature** is that the pair
is engine-invented — Scryfall has the back card on its own and the
front creature standalone, but not the two glued together as a
double-faced printing.

The catalog lives in `crabomination/src/catalog/sets/sos/mdfcs.rs`
(plus a few stragglers in `sos/creatures.rs`). Every entry uses the
`vanilla_front(...) / spell_back(...)` helpers in that module.

**Client image prefetch:** `crabomination_client::scryfall` looks up
each MDFC back by querying the **front** name with `face=back` first
(real-MDFC path — pathways and the like). On HTTP 422 (Scryfall:
"unprocessable" — the front isn't a double-faced card) or 404 (the
front is engine-invented and not on Scryfall), it falls back to a
direct `cards/named` lookup of the **back** name as a regular card.
That second path is what saves prepared cards: the back is always a
real Scryfall printing on its own, so the fallback always succeeds
and the runtime renders the actual spell art instead of a cardback
placeholder. See `download_card_image` in `scryfall.rs`.

#### Half 2 — The prepared flag (still ⏳)

Independent of who has a prepare spell, certain SOS cards toggle a
per-permanent boolean **prepared** flag on a creature. The flag works
like a stun / phased / monstrosity counter: a per-permanent boolean
(or counter of count 1) set by "becomes prepared" and cleared by
"becomes unprepared". Cards that *care* about the flag have a
**Prepare {cost}** activated/triggered ability with the reminder text
"(Only creatures with prepare spells can become prepared.)" — i.e.
the gating predicate is "this creature's `back_face` is a prepare
spell, **and** the prepared flag is set".

Cards in the SOS table that toggle the flag:

- **Biblioplex Tomekeeper** ({4} 3/4) — ETB toggle (prepare or unprepare).
- **Skycoach Waypoint** (land) — `{3},{T}: prepare target`.
- Cards whose oracle text was previously truncated by the gen script's
  220-char cap (now 600 chars) may also expose a `Prepare {cost}` ability;
  when re-running `scripts/gen_strixhaven2.py` look for "prepare " or
  "prepared" in the oracle column to spot them.

Engine-side, Half 2 still needs:

1. A new `CounterType::Prepared` (or a `PermanentFlag::Prepared` boolean)
   on `Permanent`, surfaced through `PermanentView` for the client UI.
2. `Effect::SetPrepared { what, value: bool }` to flip the flag.
3. A `Predicate::IsPrepared` so prepare-payoff cards (the cards
   *granting* a Prepare ability) can gate their riders on the flag.
4. The activated ability *itself* on payoff cards — those need an
   ability authored from the truncated body of the card.

The flag side is ⏳ until at least (1) and (2) land. Track in
`TODO.md` under Engine — Missing Mechanics. The spell-side prepared
cards (Half 1) are wired today and tagged 🟡 in the SOS table when
the back-face spell body is in place.

---

## White

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Ajani's Response | {4}{W} | Instant |  | This spell costs {3} less to cast if it targets a tapped creature. / Destroy target creature. | ✅ | Push XXXVIII: 🟡 → ✅. Self-static cost reduction via the new `StaticEffect::CostReductionTargeting` primitive: `target_filter: Creature ∧ Tapped, amount: 3`. `cost_reduction_for_spell` walks the cast card's own static abilities (in addition to the battlefield), so self-discount spells close the cost-reduction gap without needing a permanent in play. |
| Antiquities on the Loose | {1}{W}{W} | Sorcery |  | Create two 2/2 red and white Spirit creature tokens. Then if this spell was cast from anywhere other than your hand, put a +1/+1 counter on each Spirit you control. / Flashback {4}{W}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | All three clauses wired. Creates 2× 2/2 R/W Spirit tokens + `Keyword::Flashback({4}{W}{W})`. The cast-from-elsewhere rider uses the new `Predicate::CastFromGraveyard` (reads `EffectContext.cast_face` — `CastFace::Flashback` triggers the rider). On flashback cast, each Spirit you control gets a +1/+1 counter (per `ForEach Spirit & ControlledByYou → AddCounter(+1/+1)`). |
| Ascendant Dustspeaker | {4}{W} | Creature — Orc Cleric | 3/4 | Flying / When this creature enters, put a +1/+1 counter on another target creature you control. / At the beginning of combat on your turn, exile up to one target card from a graveyard. | ✅ | Wired in `catalog::sets::sos::creatures` with both ETB pump + combat-step exile triggers. |
| Daydream | {W} | Sorcery |  | Exile target creature you control, then return that card to the battlefield under its owner's control with a +1/+1 counter on it. / Flashback {2}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Wired in `catalog::sets::sos::sorceries` as the standard Restoration-Angel-style flicker pattern (`Exile + Move(target → battlefield) + AddCounter`). Flashback {2}{W} now wired via `Keyword::Flashback` (push X) — graveyard replay reuses the engine's existing `cast_flashback` path. The library traversal in `move_card_to` was extended to handle library-source moves so the flicker round-trip resolves end-to-end. |
| Dig Site Inventory | {W} | Sorcery |  | Put a +1/+1 counter on target creature you control. It gains vigilance until end of turn. / Flashback {W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Mainline pump+vigilance wired in `catalog::sets::sos::sorceries`; Flashback {W} clause now wired via `Keyword::Flashback` (push X). |
| Eager Glyphmage | {3}{W} | Creature — Cat Cleric | 3/3 | When this creature enters, create a 1/1 white and black Inkling creature token with flying. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Elite Interceptor // Rejoinder | {W} // {1}{W} | Creature — Human Wizard // Sorcery | 1/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Emeritus of Truce // Swords to Plowshares | {1}{W}{W} // {W} | Creature — Cat Cleric // Instant | 3/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Ennis, Debate Moderator | {1}{W} | Legendary Creature — Human Cleric | 1/1 | When Ennis enters, exile up to one other target creature you control. Return that card to the battlefield under its owner's control at the beginning of the next end step. / At the beginning of your end step, if one or more cards were put into exile this turn, put a +1/+1 counter on Ennis. | ✅ | Push XXXVIII: doc-only promotion. Both abilities fully wired since push IX. ETB flicker (`Exile + DelayUntil(NextEndStep, Move(Target → Battlefield(OwnerOf)))`) + end-step gated counter via the exact `Predicate::CardsExiledThisTurnAtLeast` (push IX, backed by `Player.cards_exiled_this_turn`). Prior 🟡 was a stale annotation that misread the proxy doc string. |
| Erode | {W} | Instant |  | Destroy target creature or planeswalker. Its controller may search their library for a basic land card, put it onto the battlefield tapped, then shuffle. | ✅ | Push XV: now fully wired. Destroy + `Search { who: ControllerOf(Target), filter: IsBasicLand, to: Battlefield(ControllerOf(Target), tapped) }`. The "may" optionality is collapsed to always-search (decline path covered by `Effect::Search`'s decider returning `Search(None)`). |
| Graduation Day | {W} | Enchantment |  | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, put a +1/+1 counter on target creature you control. | ✅ | Wired in `catalog::sets::sos::enchantments` via `repartee()` shortcut + `target_filtered(Creature & ControlledByYou)` AddCounter. |
| Group Project | {1}{W} | Sorcery |  | Create a 2/2 red and white Spirit creature token. / Flashback—Tap three untapped creatures you control. (You may cast this card from your graveyard for its flashback cost. Then exile it.) | 🟡 | Mainline 2/2 R/W Spirit token wired (new `spirit_token()` helper); Flashback "tap three" cost omitted. |
| Harsh Annotation | {1}{W} | Instant |  | Destroy target creature. Its controller creates a 1/1 white and black Inkling creature token with flying. | ✅ | Push XXI: Inkling token now created under the destroyed creature's controller via `PlayerRef::ControllerOf(Target(0))` — the engine's `find_card_owner` graveyard fallback resolves the target's controller post-destroy. |
| Honorbound Page // Forum's Favor | {3}{W} // {W} | Creature — Cat Cleric // Sorcery | 3/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Informed Inkwright | {1}{W} | Creature — Human Wizard | 2/2 | Vigilance / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, create a 1/1 white and black Inkling creature token with flying. | ✅ | Vigilance body + Repartee Inkling token wired via `repartee()` + `inkling_token()`. |
| Inkshape Demonstrator | {3}{W} | Creature — Elephant Cleric | 3/4 | Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.) / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, this creature gets +1/+0 and gains lifelink until end of turn. | ✅ | Push XXXVIII: doc-only promotion. Body + `Keyword::Ward(2)` + Repartee body all wired identically to Mica Reader of Ruins (✅) — same Ward-keyword-tagged-for-future-enforcement pattern. The Repartee +1/+0 + lifelink combat trick is the dominant piece of value; the Ward keyword tag is in place for future enforcement. |
| Interjection | {W} | Instant |  | Target creature gets +2/+2 and gains first strike until end of turn. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Joined Researchers // Secret Rendezvous | {1}{W} // {1}{W}{W} | Creature — Human Cleric Wizard // Sorcery | 2/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Owlin Historian | {2}{W} | Creature — Bird Cleric | 2/3 | Flying / When this creature enters, surveil 1. (Look at the top card of your library. You may put it into your graveyard.) / Whenever one or more cards leave your graveyard, this creature gets +1/+1 until end of turn. | ✅ | All three abilities wired. The cards-leave-graveyard pump uses the SOS-V `EventKind::CardLeftGraveyard` event (per-card emission; the printed "one or more" wording approximates as per-card). |
| Practiced Offense | {2}{W} | Sorcery |  | Put a +1/+1 counter on each creature target player controls. Target creature gains your choice of double strike or lifelink until end of turn. / Flashback {1}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Push XXXV: DS-or-Lifelink mode pick now wired as a top-level `Effect::ChooseMode`: mode 0 = +1/+1 fan-out + DS grant EOT; mode 1 = +1/+1 fan-out + Lifelink grant EOT. Cast-time `mode: Some(0)` / `Some(1)` flips between the two; default is DS. Flashback {1}{W} via `Keyword::Flashback`. Player-target slot collapses to "you" (fan-out lands on every creature you control). |
| Primary Research | {4}{W} | Enchantment |  | When this enchantment enters, return target nonland permanent card with mana value 3 or less from your graveyard to the battlefield. / At the beginning of your end step, if a card left your graveyard this turn, draw a card. | ✅ | Wired in `catalog::sets::sos::enchantments`. ETB returns target Nonland & ManaValueAtMost(3) gy → bf. End-step gated draw uses `Predicate::CardsLeftGraveyardThisTurnAtLeast`. |
| Quill-Blade Laureate // Twofold Intent | {1}{W} // {1}{W} | Creature — Human Cleric // Sorcery | 1/1 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Rapier Wit | {1}{W} | Instant |  | Tap target creature. If it's your turn, put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) / Draw a card. | ✅ | Wired in `catalog::sets::sos::instants` with `IsTurnOf` gating on the stun counter. |
| Rehearsed Debater | {2}{W} | Creature — Djinn Bard | 3/3 | Vigilance / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, this creature gets +1/+1 until end of turn. | ✅ | Vigilance + Repartee +1/+1 EOT, via `effect::shortcut::repartee()` + `Predicate::CastSpellTargetsMatch`. |
| Restoration Seminar | {5}{W}{W} | Sorcery — Lesson |  | Return target nonland permanent card from your graveyard to the battlefield. / Paradigm (...) | 🟡 | Wired in `catalog::sets::sos::sorceries`. Mode 0 (`Move target Nonland gy → bf untapped`) wired faithfully. Paradigm rider omitted (no copy-spell-from-exile-at-upkeep primitive — same gap as Decorum Dissertation, Improvisation Capstone, Echocasting Symposium). |
| Shattered Acolyte | {1}{W} | Creature — Dwarf Warlock | 2/2 | Lifelink / {1}, Sacrifice this creature: Destroy target artifact or enchantment. | ✅ | Wired in `catalog::sets::sos::creatures` with `sac_cost` activation. |
| Soaring Stoneglider | {2}{W} | Creature — Elephant Cleric | 4/3 | As an additional cost to cast this spell, exile two cards from your graveyard or pay {1}{W}. / Flying, vigilance | 🟡 | Wired in `catalog::sets::sos::creatures` as a 4/3 Flying+Vigilance Elephant Cleric at the **paid** cost path: full {3}{W} (base {2}{W} + the {1}{W} payment fork). The alternative additional cost (exile two from gy) is omitted (no alt-cost-with-exile-from-gy primitive). |
| Spiritcall Enthusiast // Scrollboost | {2}{W} // {1}{W} | Creature — Cat Cleric // Sorcery | 3/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Stand Up for Yourself | {2}{W} | Instant |  | Destroy target creature with power 3 or greater. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Stirring Hopesinger | {2}{W} | Creature — Bird Bard | 1/3 | Flying, lifelink / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, put a +1/+1 counter on each creature you control. | ✅ | Flying/lifelink body + Repartee fan-out via `ForEach(Creature & ControlledByYou) → AddCounter`. |
| Stone Docent | {1}{W} | Creature — Spirit Chimera | 3/1 | {W}, Exile this card from your graveyard: You gain 2 life. Surveil 1. Activate only as a sorcery. (Look at the top card of your library. You may put it into your graveyard.) | 🟡 | Body-only wire (3/1 Spirit). Graveyard-exile activated ability omitted (engine activated-ability walker only iterates the battlefield — same gap as Eternal Student, Summoned Dromedary). |
| Summoned Dromedary | {3}{W} | Creature — Spirit Camel | 4/3 | Vigilance / {1}{W}: Return this card from your graveyard to your hand. Activate only as a sorcery. | 🟡 | Vigilant 4/3 body wired; the graveyard-recursion activated ability is omitted (engine activated-ability path only walks the battlefield). |

## Blue

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Banishing Betrayal | {1}{U} | Instant |  | Return target nonland permanent to its owner's hand. Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::instants`. |
| Brush Off | {2}{U}{U} | Instant |  | This spell costs {1}{U} less to cast if it targets an instant or sorcery spell. / Counter target spell. | ✅ | Push XLV: 🟡 → ✅. Cost reduction now wires faithfully via `StaticEffect::CostReductionTargeting` on the cast card itself (self-static, same shape as Ajani's Response). Target filter is `IsSpellOnStack ∧ (HasCardType(Instant) ∨ HasCardType(Sorcery))`; at IS-targets the spell drops from {2}{U}{U} to {U}{U}. The {1}{U} → {2}-generic discount approximation loses the colored-pip refund (the engine's `CostReductionTargeting.amount` is generic-only); the {U}{U} colored-pip floor stays put. Counter half wired via `Effect::CounterSpell` against any stack-resident spell. |
| Campus Composer // Aqueous Aria | {3}{U} // {4}{U} | Creature — Merfolk Bard // Sorcery | 3/4 |  | 🟡 | Push XIX: front body wired (3/4 Merfolk Bard with `Keyword::Ward(2)`). MDFC back face Aqueous Aria omitted — oracle text unverified (Scryfall unavailable in this environment). Same body-only shape as Mica Reader of Ruins (push XVIII) / Strife Scholar / Colorstorm Stallion. |
| Chase Inspiration | {U} | Instant |  | Target creature you control gets +0/+3 and gains hexproof until end of turn. (It can't be the target of spells or abilities your opponents control.) | ✅ | Wired in `catalog::sets::sos::instants`. |
| Deluge Virtuoso | {2}{U} | Creature — Human Wizard | 2/2 | When this creature enters, tap target creature an opponent controls and put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) / Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If five or more mana was spent to cast that spell, this creature gets +2/+2 until end of turn instead. | ✅ | Push XXXI: Opus rider now wired via `effect::shortcut::opus(5, ...)`. Always: +1/+1 EOT pump. Big-cast: +1/+1 additional EOT pump (net +2/+2 EOT). ETB tap+stun unchanged. |
| Divergent Equation | {X}{X}{U} | Instant |  | Return up to X target instant and/or sorcery cards from your graveyard to your hand. / Exile Divergent Equation. | 🟡 | Wired in `catalog::sets::sos::instants` as a single-target return. The "up to X" multi-target prompt is collapsed to one target (no `Selector::OneOf` / count-bounded pick primitive yet — TODO.md). The "exile this" rider is omitted (no replay-prevention payoff). |
| Echocasting Symposium | {4}{U}{U} | Sorcery — Lesson |  | Target player creates a token that's a copy of target creature you control. / Paradigm (Then exile this spell. After you first resolve a spell with this name, you may cast a copy of it from exile without paying its mana cost at the beginning of each of your first main phases.) | ⏳ | 🔍 needs review (oracle previously truncated). Needs: copy-spell/permanent primitive; cast-from-exile pipeline. |
| Emeritus of Ideation // Ancestral Recall | {3}{U}{U} // {U} | Creature — Human Wizard // Instant | 5/5 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs`: 5/5 Human Wizard front + back-face Ancestral Recall (target player draws 3) via `GameAction::CastSpellBack`. Front-face `Keyword::Ward(2)` is tagged for future enforcement (same as Inkshape Demonstrator); the cost-to-counter rider isn't yet a ward-the-spell trigger. |
| Encouraging Aviator // Jump | {2}{U} // {U} | Creature — Bird Wizard // Instant | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Exhibition Tidecaller | {U} | Creature — Djinn Wizard | 0/2 | Opus — Whenever you cast an instant or sorcery spell, target player mills three cards. If five or more mana was spent to cast that spell, that player mills ten cards instead. | ✅ | Push XXXI: Opus mill rider now wired via `effect::shortcut::opus(5, ...)`. Cheap-cast: opp mills 3. Big-cast: opp mills 13 (additive 3+10). The "target player" prompt collapses to "each opponent" (auto-target). |
| Flow State | {1}{U} | Sorcery |  | Look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order. If there is an instant card and a sorcery card in your graveyard, instead put two of… | 🟡 | Approximated as `Scry 3 + Draw 1`. Conditional "instead pick 2 to hand" gy-IS-pair upgrade rider is omitted (no "look-and-distribute-by-count" primitive). |
| Fractal Anomaly | {U} | Instant |  | Create a 0/0 green and blue Fractal creature token and put X +1/+1 counters on it, where X is the number of cards you've drawn this turn. | ✅ | Wired in `catalog::sets::sos::instants` using the engine's new `Selector::LastCreatedToken` + `Value::CardsDrawnThisTurn` primitives. X=0 → 0/0 token dies to SBA (matches printed). |
| Fractalize | {X}{U} | Instant |  | Until end of turn, target creature becomes a green and blue Fractal with base power and toughness each equal to X plus 1. (It loses all other colors and creature types.) | 🟡 | Collapsed to `PumpPT(+(X+1), +(X+1)) EOT` in `catalog::sets::sos::instants`. The "becomes a base-(X+1)/(X+1) Fractal" rewrite is omitted (no `Effect::ResetCreature` primitive); the printed creature-type loss + color rewrite would change tribal interactions but at typical X≥2 the buffed P/T plays correctly in combat. |
| Harmonized Trio // Brainstorm | {U} // {U} | Creature — Merfolk Bard Wizard // Instant | 1/1 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Homesickness | {4}{U}{U} | Instant |  | Target player draws two cards. Tap up to two target creatures. Put a stun counter on each of them. (If a permanent with a stun counter would become untapped, remove one from it instead.) | 🟡 | Push XXXV: second creature slot now wired via `Selector::one_of(EachPermanent(opp creature))` — when 2 distinct opp creatures exist both get tapped + stunned (lethal-first). When only 1 exists the auto-pick collides with slot 0, landing 2 stun counters on the same creature. Multi-target uniqueness is still an open engine gap (the auto-target picker doesn't exclude already-targeted entities). The "target player" prompt for draws is collapsed to "you" (caster self-loots). |
| Hydro-Channeler | {1}{U} | Creature — Merfolk Wizard | 1/3 | {T}: Add {U}. Spend this mana only to cast an instant or sorcery spell. / {1}, {T}: Add one mana of any color. Spend this mana only to cast an instant or sorcery spell. | 🟡 | Wired in `catalog::sets::sos::creatures` with both mana abilities (`{T}: Add {U}` and `{1},{T}: Add one mana of any color`). The "spend this mana only to cast an instant or sorcery" restriction is omitted (no spend-restricted mana primitive — TODO.md). |
| Jadzi, Steward of Fate // Oracle's Gift | {2}{U} // {X}{X}{U} | Legendary Creature — Human Wizard // Sorcery | 2/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Landscape Painter // Vibrant Idea | {1}{U} // {4}{U} | Creature — Merfolk Wizard // Sorcery | 2/1 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Mana Sculpt | {1}{U}{U} | Instant |  | Counter target spell. If you control a Wizard, add an amount of {C} equal to the amount of mana spent to cast that spell at the beginning of your next main phase. | 🟡 | Wired in `catalog::sets::sos::instants` — counterspell + conditional `If(SelectorExists Wizard).then(AddMana(2 colorless))`. The "amount of mana spent on the countered spell" introspection is unavailable, so we approximate the rider as a flat +{C}{C}; the "delay-until-next-main" rider collapses to immediate add. |
| Mathemagics | {X}{X}{U}{U} | Sorcery |  | Target player draws 2ˣ cards. (2º = 1, 2¹ = 2, 2² = 4, 2³ = 8, 2⁴ = 16, 2⁵ = 32, and so on.) | ✅ | Wired in `catalog::sets::sos::sorceries` via the new `Value::Pow2(XFromCost)` primitive. Multi-target slot collapsed to "you" (caster draws); exponent capped at 30 to avoid deck-out. |
| Matterbending Mage | {2}{U} | Creature — Human Wizard | 2/2 | When this creature enters, return up to one other target creature to its owner's hand. / Whenever you cast a spell with {X} in its mana cost, this creature can't be blocked this turn. | ✅ | Push XVI: both abilities wired. ETB bounce stays as before; the X-cast trigger uses the new `Predicate::CastSpellHasX` + `Effect::GrantKeyword(Unblockable, EOT)` on `Selector::This`. |
| Muse Seeker | {1}{U} | Creature — Elf Wizard | 1/2 | Opus — Whenever you cast an instant or sorcery spell, draw a card. Then discard a card unless five or more mana was spent to cast that spell. | ✅ | Push XXXI: Opus loot rider now wired. Draw is unconditional; discard fires only on cheap casts (gated on `ValueAtMost(ManaSpentToCast, 4)`). Big-cast (≥5 mana): flat draw 1 with no discard. |
| Muse's Encouragement | {4}{U} | Instant |  | Create a 3/3 blue and red Elemental creature token with flying. / Surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | ✅ | Mints a 3/3 U/R Flying Elemental via the shared `elemental_token()` helper + `Effect::Surveil 2`. |
| Orysa, Tide Choreographer | {4}{U} | Legendary Creature — Merfolk Bard | 2/2 | This spell costs {3} less to cast if creatures you control have total toughness 10 or greater. / When Orysa enters, draw two cards. | 🟡 | ETB draw 2 wired faithfully. The conditional "{3} less if total toughness ≥ 10" alt-cost rider is omitted (alt-cost-with-board-state-predicate primitive). |
| Pensive Professor | {1}{U}{U} | Creature — Human Wizard | 0/2 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / Whenever one or more +1/+1 counters are put on this cr… | 🟡 | Push XXXI: Increment now wired via `effect::shortcut::increment()`. At min(P, T)=0, every 1+ mana spell pushes a +1/+1 counter. The counter-trigger half stays omitted (oracle truncated). 🔍 needs review (oracle previously truncated). |
| Procrastinate | {X}{U} | Sorcery |  | Tap target creature. Put twice X stun counters on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) | ✅ | Wired in `catalog::sets::sos::sorceries` with `Value::Times(2, XFromCost)`. |
| Run Behind | {3}{U} | Instant |  | This spell costs {1} less to cast if it targets an attacking creature. / Target creature's owner puts it on their choice of the top or bottom of their library. | ✅ | Push XLV: 🟡 → ✅. Cost reduction now wires via `StaticEffect::CostReductionTargeting` (self-static, same shape as Brush Off / Ajani's Response). Target filter is `Creature ∧ IsAttacking`; at attacker-targets the spell drops from {3}{U} to {2}{U} — Vraska's-Contempt-rate permanent removal at instant speed. Body unchanged: target creature → bottom of owner's library. The "owner's choice top/bottom" rider stays collapsed to bottom-only (the engine has no top-or-bottom owner-prompt primitive). |
| Skycoach Conductor // All Aboard | {2}{U} // {U} | Creature — Bird Pilot // Instant | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Spellbook Seeker // Careful Study | {3}{U} // {U} | Creature — Bird Wizard // Sorcery | 3/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Tester of the Tangential | {1}{U} | Creature — Djinn Wizard | 1/1 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / At the beginning of combat on your turn, you may pay {X}. When you do, move X +1/+1 counters from this creature onto another target creature. | 🟡 | Push XXXI: Increment now wired via `effect::shortcut::increment()`. Cast where mana_spent ≥ 2 (one above min(P, T)=1) drops a +1/+1 counter. The combat-step pay-X-move-counters rider stays omitted (no `MayPay`-X-cost combat-step trigger primitive). |
| Textbook Tabulator | {2}{U} | Creature — Frog Wizard | 0/3 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / When this creature enters, surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | ✅ | Push XXXI: Increment now wired via `effect::shortcut::increment()`. ETB Surveil 2 unchanged. min(P, T)=0 so every 1+ mana spell drops a counter, ramping the Frog into a real attacker. |
| Wisdom of Ages | {4}{U}{U}{U} | Sorcery |  | Return all instant and sorcery cards from your graveyard to your hand. You have no maximum hand size for the rest of the game. / Exile Wisdom of Ages. | 🟡 | Mass instant/sorcery recursion wired in `catalog::sets::sos::sorceries` via `Selector::CardsInZone` filter. The "no maximum hand size" rider and the "exile this" replacement are omitted. |

## Black

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Adventurous Eater // Have a Bite | {2}{B} // {B} | Creature — Human Warlock // Sorcery | 3/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Arcane Omens | {4}{B} | Sorcery |  | Converge — Target player discards X cards, where X is the number of colors of mana spent to cast this spell. | ✅ | Wired in `catalog::sets::sos::sorceries` via `Effect::Discard { amount: Value::ConvergedValue }` against `EachOpponent`. |
| Arnyn, Deathbloom Botanist | {2}{B} | Legendary Creature — Vampire Druid | 2/2 | Deathtouch / Whenever a creature you control with power or toughness 1 or less dies, target opponent loses 2 life and you gain 2 life. | ✅ | Wired in `catalog::sets::sos::creatures` (deathtouch + `CreatureDied/AnotherOfYours` trigger gated by `Predicate::EntityMatches { what: TriggerSource, filter: PowerAtMost(1).or(ToughnessAtMost(1)) }`). |
| Burrog Banemaker | {B} | Creature — Frog Warlock | 1/1 | Deathtouch / {1}{B}: This creature gets +1/+1 until end of turn. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Cheerful Osteomancer // Raise Dead | {3}{B} // {B} | Creature — Orc Warlock // Sorcery | 4/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Cost of Brilliance | {2}{B} | Sorcery |  | Target player draws two cards and loses 2 life. Put a +1/+1 counter on up to one target creature. | ✅ | Push XXXV: the +1/+1 half is now optional via `Selector::one_of(EachPermanent(Creature ∧ ControlledByYou))` — auto-picks a friendly creature, no-ops cleanly when none exist. Cast is now legal even when you control no creatures. The "target player" prompt for draws/loses-life half is collapsed to "you" (caster self-loots) — engine has no multi-target prompt for sorceries. |
| Decorum Dissertation | {3}{B}{B} | Sorcery — Lesson |  | Target player draws two cards and loses 2 life. / Paradigm (...) | 🟡 | Wired in `catalog::sets::sos::sorceries`. Mode 0 (you draw 2, lose 2 life) wired — collapses the "target player" prompt to the caster (engine has no multi-target prompt for sorceries). Paradigm rider omitted (no copy-spell-from-exile-at-upkeep primitive). |
| Dissection Practice | {B} | Instant |  | Target opponent loses 1 life and you gain 1 life. / Up to one target creature gets +1/+1 until end of turn. / Up to one target creature gets -1/-1 until end of turn. | ✅ | Push XXXV: all three optional halves now fire — drain 1 + +1/+1 EOT on a `Selector::one_of`-picked friendly creature (auto-falls-through when no friendly creature exists) + user-targeted -1/-1 EOT on slot 0. Same multi-target collapse pattern as Vibrant Outburst's tap half. |
| Emeritus of Woe // Demonic Tutor | {3}{B} // {1}{B} | Creature — Vampire Warlock // Sorcery | 5/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| End of the Hunt | {1}{B} | Sorcery |  | Target opponent exiles a creature or planeswalker they control with the greatest mana value among creatures and planeswalkers they control. | 🟡 | Wired in `catalog::sets::sos::sorceries` as a single-target Exile against `Creature ∨ Planeswalker & ControlledByOpponent`. The "greatest mana value" picker isn't enforced (auto-target picks first eligible). |
| Eternal Student | {3}{B} | Creature — Zombie Warlock | 4/2 | {1}{B}, Exile this card from your graveyard: Create two 1/1 white and black Inkling creature tokens with flying. | 🟡 | Vanilla 4/2 body wired in `catalog::sets::sos::creatures`. Graveyard-exile activated ability omitted (engine activated-ability path only walks the battlefield). |
| Foolish Fate | {2}{B} | Instant |  | Destroy target creature. / Infusion — If you gained life this turn, that creature's controller loses 3 life. | ✅ | Wired with the new `Predicate::LifeGainedThisTurnAtLeast` Infusion gate. |
| Forum Necroscribe | {5}{B} | Creature — Troll Warlock | 5/4 | Ward—Discard a card. / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, return target creature card from your graveyard to the battlefield. | 🟡 | Wired in `catalog::sets::sos::creatures` (5/4 Troll Warlock body + Repartee gy-creature-recursion via the `repartee()` shortcut chained with `Effect::Move(target Creature → Battlefield(You))`). Ward—Discard a card omitted (no Ward keyword primitive yet — tracked in TODO.md). |
| Grave Researcher // Reanimate | {2}{B} // {B} | Creature — Troll Warlock // Sorcery | 3/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs`. Front: 3/3 Troll Warlock with ETB Surveil 2 (Surveil is a first-class engine primitive). Back: Reanimate (target gy creature → BF under your control + lose life equal to its MV via `Value::ManaValueOf`). |
| Lecturing Scornmage | {B} | Creature — Human Warlock | 1/1 | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, put a +1/+1 counter on this creature. | ✅ | Repartee +1/+1 counter via `effect::shortcut::repartee()`. |
| Leech Collector // Bloodletting | {1}{B} // {B} | Creature — Human Warlock // Sorcery | 2/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Masterful Flourish | {B} | Instant |  | Target creature you control gets +1/+0 and gains indestructible until end of turn. (Damage and effects that say "destroy" don't destroy it.) | ✅ | Wired in `catalog::sets::sos::instants`. |
| Melancholic Poet | {1}{B} | Creature — Elf Bard | 2/2 | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, each opponent loses 1 life and you gain 1 life. | ✅ | Repartee drain 1 via `effect::shortcut::repartee()`. |
| Moseo, Vein's New Dean | {2}{B} | Legendary Creature — Bird Skeleton Warlock | 2/1 | Flying / When Moseo enters, create a 1/1 black and green Pest creature token with "Whenever this token attacks, you gain 1 life." / Infusion — At the beginning of your end step, if you gained life this turn, return up… | 🟡 | Body + Flying + ETB Pest token wired faithfully (Pest's on-attack lifegain rider rides on the shared `pest_token()` helper). Infusion end-step rider omitted (oracle truncated; no MayDo per-turn-lifegain primitive). |
| Poisoner's Apprentice | {2}{B} | Creature — Orc Warlock | 2/2 | Infusion — When this creature enters, target creature an opponent controls gets -4/-4 until end of turn if you gained life this turn. | ✅ | Wired in `catalog::sets::sos::creatures` with the `LifeGainedThisTurnAtLeast` Infusion gate on the ETB trigger. |
| Postmortem Professor | {1}{B} | Creature — Zombie Warlock | 2/2 | This creature can't block. / Whenever this creature attacks, each opponent loses 1 life and you gain 1 life. / {1}{B}, Exile an instant or sorcery card from your graveyard: Return this card from your graveyard to the battlefield. | 🟡 | On-attack drain + the printed `Keyword::CantBlock` static restriction (now first-class via SOS push VI) wired in `catalog::sets::sos::creatures`. The graveyard-exile recursion activated ability is still omitted (engine activated-ability walker only iterates the battlefield — TODO.md "Activated-Ability `From Your Graveyard` Path"). |
| Pox Plague | {B}{B}{B}{B}{B} | Sorcery |  | Each player loses half their life, then discards half the cards in their hand, then sacrifices half the permanents they control of their choice. Round down each time. | ✅ | Wired in `catalog::sets::sos::sorceries` via `ForEach Player(EachPlayer)` body using the new `Value::HalfDown` + `Value::PermanentCountControlledBy(Triggerer)` primitives. Half-life / half-hand / half-permanents per player. |
| Pull from the Grave | {2}{B} | Sorcery |  | Return up to two target creature cards from your graveyard to your hand. You gain 2 life. | 🟡 | Returns up to **two** creature cards from your graveyard via the new `Selector::Take(_, 2)` primitive (push X) — strictly closer to the printed "up to two" cap than the prior single-card return. Multi-target prompt for the printed "target creature card" slots is still collapsed (no UI multi-target picker yet); the auto-decider picks the top two matching creature cards in graveyard order. Lifegain half resolves regardless. |
| Rabid Attack | {1}{B} | Instant |  | Until end of turn, any number of target creatures you control each get +1/+0 and gain "When this creature dies, draw a card." | 🟡 | Wired in `catalog::sets::sos::instants` as a +1/+0 EOT pump on a single target. The "any number of target creatures" multi-target prompt and the transient die-to-draw triggered ability grant are both omitted (engine TODOs). |
| Ral Zarek, Guest Lecturer | {1}{B}{B} | Legendary Planeswalker — Ral | [3] | +1: Surveil 2. / −1: Any number of target players each discard a card. / −2: Return target creature card with mana value 3 or less from your graveyard to the battlefield. / −7: Flip five coins. Target opponent skips their next X turns, where X is the number of coins that came up heads. | 🟡 | +1 Surveil 2 / -1 each opp discards 1 (single-target collapse) / -2 return ≤3-MV creature card from your gy → bf. -7 coin-flip / skip-turns ult omitted (no coin-flip + no skip-turns primitive). |
| Scathing Shadelock // Venomous Words | {4}{B} // {B} | Creature — Snake Warlock // Sorcery | 4/6 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Scheming Silvertongue // Sign in Blood | {1}{B} // {B}{B} | Creature — Vampire Warlock // Sorcery | 1/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Send in the Pest | {1}{B} | Sorcery |  | Each opponent discards a card. You create a 1/1 black and green Pest creature token with "Whenever this token attacks, you gain 1 life." | ✅ | Discard + Pest token wired; the token's "gain 1 on attack" rider now fires (via the new `TokenDefinition.triggered_abilities` field). |
| Sneering Shadewriter | {4}{B} | Creature — Vampire Warlock | 3/3 | Flying / When this creature enters, each opponent loses 2 life and you gain 2 life. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Tragedy Feaster | {2}{B}{B} | Creature — Demon | 7/6 | Trample / Ward—Discard a card. / Infusion — At the beginning of your end step, sacrifice a permanent unless you gained life this turn. | 🟡 | Body-only wire in `catalog::sets::sos::creatures` (7/6 Trample Demon). Ward—Discard rider and Infusion upkeep-sac-unless-lifegain rider both omitted (no Ward keyword primitive; no `MayDo` / per-turn-lifegain-sac primitive). The vanilla shell is still useful as a 4-mana finisher. |
| Ulna Alley Shopkeep | {2}{B} | Creature — Goblin Warlock | 2/3 | Menace (This creature can't be blocked except by two or more creatures.) / Infusion — This creature gets +2/+0 as long as you gained life this turn. | 🟡 | Body-only wire (2/3 Menace Goblin Warlock). The Infusion static "+2/+0 while you've gained life this turn" rider is omitted (no continuous-static-on-predicate primitive). |
| Wander Off | {3}{B} | Instant |  | Exile target creature. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Withering Curse | {1}{B}{B} | Sorcery |  | All creatures get -2/-2 until end of turn. / Infusion — If you gained life this turn, destroy all creatures instead. | ✅ | `If LifeGainedThisTurnAtLeast(1)` branch: Infusion-path = ForEach(Creature) Destroy; mainline = ForEach(Creature) PumpPT(-2/-2 EOT). |

## Red

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Archaic's Agony | {4}{R} | Sorcery |  | Converge — Archaic's Agony deals X damage to target creature, where X is the number of colors of mana spent to cast this spell. Exile cards from the top of your library equal to the excess damage dealt to that creature this way. You may play those cards until the end of your next turn. | ⏳ | 🔍 needs review (oracle previously truncated). Needs: cast-from-exile pipeline. |
| Artistic Process | {3}{R}{R} | Sorcery |  | Choose one — / • Artistic Process deals 6 damage to target creature. / • Artistic Process deals 2 damage to each creature you don't control. / • Create a 3/3 blue and red Elemental creature token with flying. It gains haste until end of turn. | ✅ | Wired in `catalog::sets::sos::sorceries`. All three modes wired: 6-to-creature, 2-to-each-opp-creature (via `Selector::EachPermanent(Creature & ControlledByOpponent)`), Elemental token + transient haste via `Selector::LastCreatedToken`. |
| Blazing Firesinger // Seething Song | {2}{R} // {2}{R} | Creature — Dwarf Bard // Instant | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Charging Strifeknight | {2}{R} | Creature — Spirit Knight | 3/3 | Haste / {T}, Discard a card: Draw a card. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Choreographed Sparks | {R}{R} | Instant |  | This spell can't be copied. / Choose one or both — / • Copy target instant or sorcery spell you control. You may choose new targets for the copy. / • Copy target creature spell you control. The copy gains haste and "At the beginning of the end step, sacrifice this token." | ✅ | Push XXI: NEW factory wired via the new `Effect::CopySpell` primitive. Single-mode wire of "Copy target IS spell you control" — target filter is `IsSpellOnStack & ControlledByYou` (the engine's `ControlledByYou` evaluator now falls through to stack-resident spells). The "creature spell — gains haste, end-step sac" rider is omitted (no permanent-copy primitive yet); "this spell can't be copied" is a no-op (no copy-immune flag). |
| Duel Tactics | {R} | Sorcery |  | Duel Tactics deals 1 damage to target creature. It can't block this turn. / Flashback {1}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Wired as `DealDamage(1) + GrantKeyword(CantBlock, EOT)` — pulls in the new `Keyword::CantBlock` (enforced inside `declare_blockers` and the `can_block_*` helpers). Flashback {1}{R} now wired via `Keyword::Flashback` (push X). |
| Emeritus of Conflict // Lightning Bolt | {1}{R} // {R} | Creature — Human Wizard // Instant | 2/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Expressive Firedancer | {1}{R} | Creature — Human Sorcerer | 2/2 | Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If five or more mana was spent to cast that spell, this creature also gains double strike until end of turn. | ✅ | Push XXXIII: Opus rider now wired via `opus(5, ...)`. Always: +1/+1 EOT. Big-cast: `Keyword::DoubleStrike` granted EOT (additive — both halves stack on big casts). |
| Flashback | {R} | Instant |  | Target instant or sorcery card in your graveyard gains flashback until end of turn. The flashback cost is equal to its mana cost. (You may cast that card from your graveyard for its flashback cost. Then exile it.) | ⏳ | Needs: cast-from-exile pipeline; cast-from-graveyard. |
| Garrison Excavator | {3}{R} | Creature — Orc Sorcerer | 3/4 | Menace (This creature can't be blocked except by two or more creatures.) / Whenever one or more cards leave your graveyard, create a 2/2 red and white Spirit creature token. | ✅ | Wired against the new `EventKind::CardLeftGraveyard` event — every gy-leave mints a 2/2 R/W Spirit token via the shared `spirit_token()` helper. |
| Goblin Glasswright // Craft with Pride | {1}{R} // {R} | Creature — Goblin Sorcerer // Sorcery | 2/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Heated Argument | {4}{R} | Instant |  | Heated Argument deals 6 damage to target creature. You may exile a card from your graveyard. If you do, Heated Argument also deals 2 damage to that creature's controller. | ✅ | 6-to-creature is unconditional; the gy-exile + 2-to-controller chain is wrapped in `Effect::MayDo` (push XV) and either both fire or both skip. Uses `Selector::take(CardsInZone(GY), 1)` to pick exactly one gy card to exile (matching "a card", not "every card"). |
| Impractical Joke | {R} | Sorcery |  | Damage can't be prevented this turn. Impractical Joke deals 3 damage to up to one target creature or planeswalker. | 🟡 | 3-to-creature/PW wired; "damage can't be prevented" rider is a no-op (engine has no damage-prevention layer). |
| Improvisation Capstone | {5}{R}{R} | Sorcery — Lesson |  | Exile cards from the top of your library until you exile cards with total mana value 4 or greater. You may cast any number of spells from among them without paying their mana costs. / Paradigm (Then exile this spell. After you first resolve a spell with this name, you may cast a copy of it from exile without paying its mana cost at the beginning of each of your first main phases.) | ⏳ | 🔍 needs review (oracle previously truncated). Needs: copy-spell/permanent primitive; cast-from-exile pipeline. |
| Living History | {1}{R} | Enchantment |  | When this enchantment enters, create a 2/2 red and white Spirit creature token. / Whenever you attack, if a card left your graveyard this turn, target attacking creature gets +2/+0 until end of turn. | 🟡 | ETB Spirit token + on-attack +2/+0 EOT (gated on the new `Predicate::CardsLeftGraveyardThisTurnAtLeast`). The "target attacking creature" picks the trigger source (the just-declared attacker) rather than a fresh target — collapsed for the per-attacker auto-target framework. |
| Maelstrom Artisan // Rocket Volley | {1}{R}{R} // {1}{R} | Creature — Minotaur Sorcerer // Sorcery | 3/2 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Magmablood Archaic | {2/R}{2/R}{2/R} | Creature — Avatar | 2/2 | Trample, reach / Converge — This creature enters with a +1/+1 counter on it for each color of mana spent to cast it. / Whenever you cast an instant or sorcery spell, creatures you control get +1/+0 until end of turn for each color of mana spent to cast that spell. | 🟡 | Body wired in `catalog::sets::sos::creatures` (2/2 Avatar with Trample+Reach + Converge ETB AddCounter using `Value::ConvergedValue`). The IS-cast pump rider is omitted pending per-cast converge introspection on the *just-cast* spell (the trigger fires but reads the Archaic's own ETB converge value, not the iterated cast's). Hybrid `{2/R}` pips approximated as `{2}+{R}` per pip. |
| Mica, Reader of Ruins | {3}{R} | Legendary Creature — Human Artificer | 4/4 | Ward—Pay 3 life. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays 3 life.) / Whenever you cast an instant or sorcery spell, you may sacrifice an artifact. If you do, copy that spell and you may choose new targets for the copy. | ✅ | Push XXI: magecraft → MayDo body sacrifices an artifact + copies the just-cast spell via `Effect::CopySpell { what: Selector::CastSpellSource }`. The "If you do" rider is approximated — if no artifact is available the body still fires the copy (small over-payoff vs printed semantics). `Keyword::Ward(3)` body unchanged. |
| Molten-Core Maestro | {1}{R} | Creature — Goblin Bard | 2/2 | Menace / Opus — Whenever you cast an instant or sorcery spell, put a +1/+1 counter on this creature. If five or more mana was spent to cast that spell, add an amount of {R} equal to this creature's power. | ✅ | Push XXXIII: Opus rider now wired via `opus(5, ...)`. Always: +1/+1 counter on This. Big-cast: `AddMana(OfColor(Red, PowerOf(This)))`. The +1/+1 counter resolves first (always before big), so a 2/2 → 3/3 → adds {R}{R}{R} on a 5-mana cast. |
| Pigment Wrangler // Striking Palette | {4}{R} // {R} | Creature — Orc Sorcerer // Sorcery | 4/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Rearing Embermare | {4}{R} | Creature — Horse Beast | 4/5 | Reach, haste | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Rubble Rouser | {2}{R} | Creature — Dwarf Sorcerer | 1/4 | When this creature enters, you may discard a card. If you do, draw a card. / {T}, Exile a card from your graveyard: Add {R}. When you do, this creature deals 1 damage to each opponent. | 🟡 | Push XV: ETB rummage now wrapped in `Effect::MayDo` so the "you may discard" optionality is honored. The `{T}, Exile a card from your graveyard:` activated ability is still omitted (engine activated-ability path has no `from-your-graveyard` cost variant — separate from `sac_cost`). |
| Steal the Show | {2}{R} | Sorcery |  | Choose one or both — / • Target player discards any number of cards, then draws that many cards. / • Steal the Show deals damage equal to the number of instant and sorcery cards in your graveyard to target creature or planeswalker. | 🟡 | Modal sorcery: mode 0 (target player discards N then draws N — collapsed to "discard 2, draw 2" since the engine has no "any number" prompt for the targeted player); mode 1 deals damage = `Value::CountOf(CardsInZone(your graveyard, IS-cards))` to a creature/PW. The "choose one or both" rider collapses to "pick one mode" (no multi-mode-pick primitive yet). |
| Strife Scholar // Awaken the Ages | {2}{R} // {5}{R} | Creature — Orc Sorcerer // Sorcery | 3/2 |  | 🟡 | Push XIX: front body wired (3/2 Orc Sorcerer with `Keyword::Ward(2)`). MDFC back face Awaken the Ages omitted — oracle text unverified. Same body-only shape as Mica Reader of Ruins / Colorstorm Stallion. |
| Tablet of Discovery | {2}{R} | Artifact |  | When this artifact enters, mill a card. You may play that card this turn. (To mill a card, put the top card of your library into your graveyard.) / {T}: Add {R}. / {T}: Add {R}{R}. Spend this mana only to cast instant and sorcery spells. | 🟡 | Wired in `catalog::sets::sos::artifacts` — ETB Mill 1 + two `{T}: Add {R}` mana abilities. The "may play that card this turn" mill-rider is omitted (no per-card may-play primitive yet). The spend-restriction on the {T}: Add {R}{R} ability is omitted (no spend-restricted mana primitive). |
| Tackle Artist | {3}{R} | Creature — Orc Sorcerer | 4/3 | Trample / Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If five or more mana was spent to cast that spell, put a +1/+1 counter on this creature instead. | ✅ | Push XXXI: Opus rider now wired via `effect::shortcut::opus(5, ...)` powered by the new `Value::ManaSpentToCast` primitive. Cheap-cast: +1/+1 EOT pump. Big-cast (≥5 mana): +1/+1 permanent counter. The "instead" wording is approximated as additive (both halves run on big-cast — minor over-payoff, combat-correct). |
| Thunderdrum Soloist | {1}{R} | Creature — Dwarf Bard | 1/3 | Reach / Opus — Whenever you cast an instant or sorcery spell, this creature deals 1 damage to each opponent. If five or more mana was spent to cast that spell, this creature deals 3 damage to each opponent instead. | ✅ | Push XXXIII: Opus rider now wired via `effect::shortcut::opus(5, ...)`. Always: 1 damage to each opp. Big-cast (≥5 mana): an additional 2 damage (net 3 to each opp) — additive "instead" approximation matching Spectacular Skywhale. |
| Tome Blast | {1}{R} | Sorcery |  | Tome Blast deals 2 damage to any target. / Flashback {4}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Wired as a 2-to-any-target burn spell. Flashback {4}{R} now wired via `Keyword::Flashback` (push X). |
| Unsubtle Mockery | {2}{R} | Instant |  | Unsubtle Mockery deals 4 damage to target creature. Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | `DealDamage(4) + Surveil 1` via `Effect::Surveil`. |
| Zealous Lorecaster | {5}{R} | Creature — Giant Sorcerer | 4/4 | When this creature enters, return target instant or sorcery card from your graveyard to your hand. | ✅ | Wired in `catalog::sets::sos::creatures` with a Move-target-from-graveyard ETB trigger. |

## Green

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Aberrant Manawurm | {3}{G} | Creature — Wurm | 2/5 | Trample / Whenever you cast an instant or sorcery spell, this creature gets +X/+0 until end of turn, where X is the amount of mana spent to cast that spell. | ✅ | Push XXXI: now wired via the new `Value::ManaSpentToCast` primitive on a magecraft trigger. Bolt cast → +1/+0 EOT; Wisdom of Ages (CMC 7) → +7/+0 EOT. Trample turns the pump straight into face damage. |
| Additive Evolution | {3}{G}{G} | Enchantment |  | When this enchantment enters, create a 0/0 green and blue Fractal creature token. Put three +1/+1 counters on it. / At the beginning of combat on your turn, put a +1/+1 counter on target creature you control. It gains vigilance until end of turn. | ✅ | Wired in `catalog::sets::sos::enchantments`. ETB Fractal-with-3-counters via the existing `fractal_token()` helper + `Selector::LastCreatedToken` AddCounter. Begin-combat +1/+1 counter + Vigilance (EOT) on a friendly creature, gated through the active-player StepBegins(BeginCombat) trigger. |
| Ambitious Augmenter | {G} | Creature — Turtle Wizard | 1/1 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / When this creature dies, if it had one or more counters on it, create a 0/0 green and blue Fractal creature token, then put this creature's counters on that token. | 🟡 | Push XXXIII: Increment trigger now wired via `effect::shortcut::increment()` (push XXXI primitive). Each cast at ≥2 mana drops a +1/+1 counter. The dies-as-Fractal-with-counters trigger stays omitted pending a counter-transfer-on-death primitive (Selector::Self.counters_at_death snapshot + transfer-to-LastCreatedToken effect). |
| Burrog Barrage | {1}{G} | Instant |  | Target creature you control gets +1/+0 until end of turn if you've cast another instant or sorcery spell this turn. Then it deals damage equal to its power to up to one target creature an opponent controls. | ✅ | Push XXXV: damage half now hits an opp creature via `Selector::one_of(EachPermanent(opp creature))` (was: dealt damage to slot 0 = the friendly creature, which was self-damage). One-sided power-as-damage to an opp creature, scaled by slot 0's power (the friendly with the +1/+0 pump applied first). No-ops cleanly when no opp creature is on the battlefield. |
| Chelonian Tackle | {2}{G} | Sorcery |  | Target creature you control gets +0/+10 until end of turn. Then it fights up to one target creature an opponent controls. (Each deals damage equal to its power to the other.) | 🟡 | +0/+10 EOT pump + the new `Effect::Fight` against an auto-selected opp creature (no multi-target prompt for the defender slot). Fight no-ops cleanly when no opp creature is on the battlefield, preserving the printed "up to one" semantics. |
| Comforting Counsel | {1}{G} | Enchantment |  | Whenever you gain life, put a growth counter on this enchantment. / As long as there are five or more growth counters on this enchantment, creatures you control get +3/+3. | 🟡 | Lifegain → Growth counter trigger wired in `catalog::sets::sos::enchantments`. The "≥5 counters → anthem" static is omitted (no self-counter-gated `StaticEffect` primitive). |
| Efflorescence | {2}{G} | Instant |  | Put two +1/+1 counters on target creature. / Infusion — If you gained life this turn, that creature also gains trample and indestructible until end of turn. | ✅ | Wired with the new `Predicate::LifeGainedThisTurnAtLeast` Infusion gate. |
| Emeritus of Abundance // Regrowth | {2}{G} // {1}{G} | Creature — Elf Druid // Sorcery | 3/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Emil, Vastlands Roamer | {2}{G} | Legendary Creature — Elf Druid | 3/3 | Creatures you control with +1/+1 counters on them have trample. / {4}{G}, {T}: Create a 0/0 green and blue Fractal creature token. Put X +1/+1 counters on it, where X is the number of differently named lands you control. | ✅ | Wired in `catalog::sets::sos::creatures` — `StaticEffect::GrantKeyword(Trample)` filtered to creatures with +1/+1 counters via the new `AffectedPermanents::AllWithCounter` layer variant; activated `{4}{G},{T}` creates a Fractal + counters scaled to land count. "Differently named" filter on X is collapsed to total land count (typical cube games have unique land slots). |
| Environmental Scientist | {1}{G} | Creature — Human Druid | 2/2 | When this creature enters, you may search your library for a basic land card, reveal it, put it into your hand, then shuffle. | ✅ | Wired with `Effect::Search` over `IsBasicLand`. |
| Follow the Lumarets | {1}{G} | Sorcery |  | Infusion — Look at the top four cards of your library. You may reveal a creature or land card from among them and put it into your hand. If you gained life this turn, you may instead reveal two creature and/or land cards from among them and put them into your hand. Put the rest on the bottom of your library in a random order. | 🟡 | Push XV: wired as `If(LifeGainedThisTurnAtLeast(1)) → 2× RevealUntilFind(cap=4) → Hand : 1× RevealUntilFind(cap=4) → Hand`. Find filter = Creature OR Land. Approximations: misses go to graveyard (not bottom of library) — `RevealUntilFind`'s engine default; "you may reveal" optionality collapsed to always-do (declining would mill 4, strictly worse). |
| Germination Practicum | {3}{G}{G} | Sorcery — Lesson |  | Put two +1/+1 counters on each creature you control. / Paradigm (...) | 🟡 | Wired in `catalog::sets::sos::sorceries` as a `ForEach Creature & ControlledByYou → AddCounter +1/+1 ×2` fan-out. Paradigm rider omitted (no copy-spell-from-exile-at-upkeep primitive). |
| Glorious Decay | {1}{G} | Instant |  | Choose one — / • Destroy target artifact. / • Glorious Decay deals 4 damage to target creature with flying. / • Exile target card from a graveyard. Draw a card. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Hungry Graffalon | {3}{G} | Creature — Giraffe | 3/4 | Reach / Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) | ✅ | Push XXXI: Increment now wired via `effect::shortcut::increment()` powered by `Value::ManaSpentToCast`. Each cast where mana_spent ≥ 4 (one above min(P, T)=3) drops a +1/+1 counter. |
| Infirmary Healer // Stream of Life | {1}{G} // {X}{G} | Creature — Cat Cleric // Sorcery | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Lumaret's Favor | {1}{G} | Instant |  | Infusion — When you cast this spell, copy it if you gained life this turn. You may choose new targets for the copy. / Target creature gets +2/+4 until end of turn. | ✅ | Push XXI: mainline +2/+4 EOT pump + Infusion on-cast self-trigger gated on `Predicate::LifeGainedThisTurnAtLeast(1)` that copies via `CopySpell { what: CastSpellSource }`. The "you may choose new targets for the copy" rider is collapsed (copy inherits original target). |
| Mindful Biomancer | {1}{G} | Creature — Dryad Druid | 2/2 | When this creature enters, you gain 1 life. / {2}{G}: This creature gets +2/+2 until end of turn. Activate only once each turn. | ✅ | Wired in `catalog::sets::sos::creatures`; once-per-turn enforced engine-side. |
| Noxious Newt | {1}{G} | Creature — Salamander | 1/2 | Deathtouch / {T}: Add {G}. | ✅ | Wired in `catalog::sets::sos::creatures`. Now uses the new `Salamander` creature subtype. |
| Oracle's Restoration | {G} | Sorcery |  | Target creature you control gets +1/+1 until end of turn. You draw a card and gain 1 life. | ✅ | Wired in `catalog::sets::sos::sorceries`. |
| Pestbrood Sloth | {3}{G} | Creature — Plant Sloth | 4/4 | Reach / When this creature dies, create two 1/1 black and green Pest creature tokens with "Whenever this token attacks, you gain 1 life." | ✅ | Death trigger creates two Pests; each token now ships with the "gain 1 on attack" rider (via the new `TokenDefinition.triggered_abilities` field). |
| Planar Engineering | {3}{G} | Sorcery |  | Sacrifice two lands. Search your library for four basic land cards, put them onto the battlefield tapped, then shuffle. | ✅ | Wired in `catalog::sets::sos::sorceries` — `Sacrifice 2 lands` + `Repeat × 4 Search { IsBasicLand → Battlefield(tapped) }`. |
| Shopkeeper's Bane | {2}{G} | Creature — Badger Pest | 4/2 | Trample / Whenever this creature attacks, you gain 2 life. | ✅ | Wired in `catalog::sets::sos::creatures` with the new `Badger` creature subtype. |
| Slumbering Trudge | {X}{G} | Creature — Plant Beast | 6/6 | This creature enters with a number of stun counters on it equal to three minus X. If X is 2 or less, it enters tapped. (If a permanent with a stun counter would become untapped, remove one from it instead.) | ✅ | Wired in `catalog::sets::sos::creatures` using `Value::NonNeg(3-X)` stun counters. The "enters tapped if X≤2" half is implemented as always-tap-on-ETB; for X≥3 the no-stun-counter trudge naturally untaps next turn. |
| Snarl Song | {5}{G} | Sorcery |  | Converge — Create two 0/0 green and blue Fractal creature tokens. Put X +1/+1 counters on each of them and you gain X life, where X is the number of colors of mana spent to cast this spell. | ✅ | Wired in `catalog::sets::sos::sorceries`: two `CreateToken` calls each followed by `AddCounter(LastCreatedToken, +1/+1, ConvergedValue)`, plus `GainLife(ConvergedValue)`. Powered by `Value::ConvergedValue` (Rancorous Archaic) and `Selector::LastCreatedToken` (Fractal Anomaly). |
| Studious First-Year // Rampant Growth | {G} // {1}{G} | Creature — Bear Wizard // Sorcery | 1/1 | Front: vanilla 1/1 Bear Wizard. Back: search your library for a basic land card, put it onto the battlefield tapped, then shuffle. | ✅ | First non-land MDFC. Front face is wired as a vanilla 1/1 Bear Wizard at `{G}`; back face is `Rampant Growth`. Cast either face via `GameAction::CastSpell` (front) or the new `GameAction::CastSpellBack` (back, added in push X — mirror to `PlayLandBack` but for non-land back faces). The engine's `cast_spell_back_face` helper swaps the in-hand `definition` to the back face's before validating cost / type / effect, so the printed back-face Sorcery resolves end-to-end. |
| Tenured Concocter | {4}{G} | Creature — Troll Druid | 4/5 | Vigilance / Whenever this creature becomes the target of a spell or ability an opponent controls, you may draw a card. / Infusion — This creature gets +2/+0 as long as you gained life this turn. | 🟡 | Vanilla 4/5 vigilance body wired in `catalog::sets::sos::creatures`. The "becomes the target" trigger is omitted (no `BecameTarget` event). The Infusion static pump is omitted (no continuous-static-on-predicate primitive). |
| Thornfist Striker | {2}{G} | Creature — Elf Druid | 3/3 | Ward {1} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.) / Infusion — Creatures you control get +1/+0 and have trample as long as you gained life this turn. | 🟡 | Body + `Keyword::Ward(1)` wired in `catalog::sets::sos::creatures`. Infusion continuous static (creatures you control get +1/+0 + trample while you gained life this turn) is omitted (no continuous-static-on-predicate primitive). |
| Topiary Lecturer | {2}{G} | Creature — Elf Druid | 1/2 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / {T}: Add an amount of {G} equal to this creature's power. | ✅ | Push XXXIII: Increment trigger now wired via `effect::shortcut::increment()`. Each cast at ≥2 mana drops a +1/+1 counter on Topiary, scaling the {T}: Add {G}×power mana ability linearly. The mana ability uses `ManaPayload::OfColor(Green, PowerOf(This))` — fixed color, value-scaled count. |
| Vastlands Scavenger // Bind to Life | {1}{G}{G} // {4}{G} | Creature — Bear Druid // Instant | 4/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Wild Hypothesis | {X}{G} | Sorcery |  | Create a 0/0 green and blue Fractal creature token. Put X +1/+1 counters on it. / Surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | ✅ | Wired in `catalog::sets::sos::sorceries`: `CreateToken(fractal) + AddCounter(LastCreatedToken, +1/+1, XFromCost) + Surveil 2`. `Effect::Surveil` is a first-class primitive so this resolves end-to-end with no approximation. |
| Wildgrowth Archaic | {2/G}{2/G} | Creature — Avatar | 0/0 | Trample, reach / Converge — This creature enters with a +1/+1 counter on it for each color of mana spent to cast it. / Whenever you cast a creature spell, that creature enters with X additional +1/+1 counters on it, where X is the number of colors of mana spent to cast it. | 🟡 | Body wired in `catalog::sets::sos::creatures` (0/0 Avatar with Trample+Reach + Converge ETB AddCounter via `Value::ConvergedValue`). Hybrid `{2/G}` pips approximated as `{2}+{G}` per pip. The "creature spells you cast enter with X extra counters" rider is omitted pending per-cast converge introspection on the *just-cast* creature spell. Mono-G casts will die immediately to SBA (printed 0/0); 2-color casts land as 2/2. |
| Zimone's Experiment | {3}{G} | Sorcery |  | Look at the top five cards of your library. You may reveal up to two creature and/or land cards from among them, then put the rest on the bottom of your library in a random order. Put all land cards revealed this way onto the battlefield tapped and put all creature cards revealed this way into your hand. | 🟡 | Approximated as `RevealUntilFind { find: Creature, cap: 5, → Hand }` followed by a `Search { filter: IsBasicLand, → Battlefield(tapped) }`. The "look at top 5, choose ≤2 matching from among them" two-destination split isn't expressible (no "look + sort by category" primitive yet); the approximation pulls one creature into hand and one basic into play, which is the typical play pattern. |

## Prismari (Blue-Red)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Abstract Paintmage | {U}{U/R}{R} | Creature — Djinn Sorcerer | 2/2 | At the beginning of your first main phase, add {U}{R}. Spend this mana only to cast instant and sorcery spells. | 🟡 | Wired in `catalog::sets::sos::creatures` with a `StepBegins(PreCombatMain)/ActivePlayer` trigger that adds {U}{R} via `ManaPayload::Colors`. The spend restriction is omitted (no per-pip mana metadata). The hybrid `{U/R}` pip is approximated as `{U}`. |
| Colorstorm Stallion | {1}{U}{R} | Creature — Elemental Horse | 3/3 | Ward {1}, haste / Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If five or more mana was spent to cast that spell, create a token that's a copy of this creature. | 🟡 | Wired in `catalog::sets::sos::creatures`. 3/3 Elemental Horse body + `Keyword::Ward(1)` + `Keyword::Haste`. Magecraft +1/+1 EOT pump on every IS cast (via `effect::shortcut::cast_is_instant_or_sorcery()` + `Effect::PumpPT { what: This }`). The "5+ mana → token copy of this creature" half is omitted (no copy-permanent primitive yet — same gap as Mica, Aziza, Silverquill the Disputant). |
| Elemental Mascot | {1}{U}{R} | Creature — Elemental Bird | 1/4 | Flying, vigilance / Opus — Whenever you cast an instant or sorcery spell, this creature gets +1/+0 until end of turn. If five or more mana was spent to cast that spell, exile the top card of your library. You may play that card until the end of your next turn. | 🟡 | Push XIX: 1/4 Flying+Vigilance Elemental Bird body wired with the +1/+0 EOT pump on every IS cast (via `effect::shortcut::cast_is_instant_or_sorcery()`). The 5+-mana exile-top-of-library "may play that card" rider is omitted (cast-from-exile pipeline gap, same as Practiced Scrollsmith / The Dawning Archaic / Conspiracy Theorist). |
| Prismari Charm | {U}{R} | Instant |  | Choose one — / • Surveil 2, then draw a card. / • Prismari Charm deals 1 damage to each of one or two targets. / • Return target nonland permanent to its owner's hand. | ✅ | 3-mode `ChooseMode`: Surveil 2 + draw / 1 dmg to creature-or-PW / bounce nonland to owner. Single-target collapse on mode 1 (printed "one or two targets" — multi-target gap). |
| Prismari, the Inspiration | {5}{U}{R} | Legendary Creature — Elder Dragon | 7/7 | Flying / Ward—Pay 5 life. / Instant and sorcery spells you cast have storm. (Whenever you cast an instant or sorcery spell, copy it for each spell cast before it this turn. You may choose new targets for the copies.) | 🟡 | Push XIX: body wired (7/7 Flying Legendary Elder Dragon U/R with `Keyword::Ward(5)` — printed Ward—Pay 5 life approximated as flat mana Ward, same primitive applied to Mica's Ward—Pay 3 life). The Storm grant on every IS cast is omitted (no copy-spell primitive yet — same gap as Aziza, Mica, Silverquill the Disputant, Choreographed Sparks). |
| Rapturous Moment | {4}{U}{R} | Sorcery |  | Draw three cards, then discard two cards. Add {U}{U}{R}{R}{R}. | ✅ | Wired in `catalog::sets::sos::sorceries`: Draw 3 + Discard 2 + AddMana with the printed UU/RRR pool. |
| Resonating Lute | {2}{U}{R} | Artifact |  | Lands you control have "{T}: Add two mana of any one color. Spend this mana only to cast instant and sorcery spells." / {T}: Draw a card. Activate only if you have seven or more cards in your hand. | 🟡 | Wired in `catalog::sets::sos::artifacts`. The {T}: Draw activation uses the new `ActivatedAbility.condition: Predicate::ValueAtLeast(HandSizeOf(You), 7)` gate — the engine rejects the activation cleanly when hand size < 7. Lands-grant tap-for-2 static is omitted (no spend-restricted-mana primitive yet — tracked in TODO.md). |
| Sanar, Unfinished Genius // Wild Idea | {U}{R} // {3}{U}{R} | Legendary Creature — Goblin Sorcerer // Sorcery | 0/4 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Spectacle Summit |  | Land |  | This land enters tapped. / {T}: Add {U} or {R}. / {2}{U}{R}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands` via the shared `school_land` builder. |
| Spectacular Skywhale | {2}{U}{R} | Creature — Elemental Whale | 1/4 | Flying / Opus — Whenever you cast an instant or sorcery spell, this creature gets +3/+0 until end of turn. If five or more mana was spent to cast that spell, put three +1/+1 counters on this creature instead. | ✅ | Push XXXI: Opus rider now wired via `effect::shortcut::opus(5, ...)`. Always: +3/+0 EOT pump. Big-cast (≥5 mana): three +1/+1 counters. |
| Splatter Technique | {1}{U}{U}{R}{R} | Sorcery |  | Choose one — / • Draw four cards. / • Splatter Technique deals 4 damage to each creature and planeswalker. | ✅ | Wired in `catalog::sets::sos::sorceries` as a `ChooseMode` with Draw 4 (mode 0) and DealDamage to `EachPermanent(Creature ∪ Planeswalker)` (mode 1). |
| Stadium Tidalmage | {2}{U}{R} | Creature — Djinn Sorcerer | 4/4 | Whenever this creature enters or attacks, you may draw a card. If you do, discard a card. | ✅ | Push XV: ETB + Attacks loot triggers wired faithfully via `Effect::MayDo`. The "you may" prompt asks the controller via `Decision::OptionalTrigger` — `AutoDecider` says no, `ScriptedDecider::new([Bool(true)])` for tests. Both halves opt-in; both fire on yes. |
| Stress Dream | {3}{U}{R} | Instant |  | Stress Dream deals 5 damage to up to one target creature. Look at the top two cards of your library. Put one of those cards into your hand and the other on the bottom of your library. | 🟡 | Push XXXV: the 5-damage half now uses `Selector::one_of(EachPermanent(opp creature))` — auto-picks an opp creature (lethal-first), no-ops when none exist (cast is legal even when no opp creatures, just the scry+draw resolves). The "look at top 2, choose 1 to hand and other to bottom" half is still approximated as `scry 1 + draw 1` (no "look at N, choose K to hand, rest to bottom" primitive yet). |
| Traumatic Critique | {X}{U}{R} | Instant |  | Traumatic Critique deals X damage to any target. Draw two cards, then discard a card. | ✅ | Wired in `catalog::sets::sos::instants` (X damage via `Value::XFromCost` + draw 2 + discard 1 loot tail). |
| Vibrant Outburst | {U}{R} | Instant |  | Vibrant Outburst deals 3 damage to any target. Tap up to one target creature. | ✅ | Push XXXV: tap half now wired via `Selector::one_of(EachPermanent(opp creature))` — auto-picks an opp creature, no-ops cleanly when none. 3-damage primary slot is still user-targeted (any target). Same multi-target-collapse pattern as Decisive Denial mode 1 / Chelonian Tackle. |
| Visionary's Dance | {5}{U}{R} | Sorcery |  | Create two 3/3 blue and red Elemental creature tokens with flying. / {2}, Discard this card: Look at the top two cards of your library. Put one of them into your hand and the other into your graveyard. | ✅ | Wired in `catalog::sets::sos::sorceries` (uses the new `elemental_token()` helper). The `{2}, Discard this card:` activation from hand is omitted (engine activation walker is battlefield-only). |
| Zaffai and the Tempests | {5}{U}{R} | Legendary Creature — Human Bard Sorcerer | 5/7 | Once during each of your turns, you may cast an instant or sorcery spell from your hand without paying its mana cost. | 🟡 | Push XVI: body-only wire (5/7 Legendary Human Bard Sorcerer). The "once per turn cast IS for free" rider is omitted (no per-turn alt-cost-grant primitive — would need `Player.zaffai_free_cast_used: bool` consumed by an alternative-cost path keyed off a cast-time static). |

## Witherbloom (Black-Green)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Blech, Loafing Pest | {1}{B}{G} | Legendary Creature — Pest | 3/4 | Whenever you gain life, put a +1/+1 counter on each Pest, Bat, Insect, Snake, and Spider you control. | ✅ | `LifeGained` (YourControl) trigger + `ForEach` fan-out filtered to Pest ∪ Bat ∪ Insect ∪ Snake ∪ Spider. |
| Bogwater Lumaret | {B}{G} | Creature — Spirit Frog | 2/2 | Whenever this creature or another creature you control enters, you gain 1 life. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Cauldron of Essence | {1}{B}{G} | Artifact |  | Whenever a creature you control dies, each opponent loses 1 life and you gain 1 life. / {1}{B}{G}, {T}, Sacrifice a creature: Return target creature card from your graveyard to the battlefield. Activate only as a sorcery. | ✅ | Death drain trigger (`CreatureDied/AnotherOfYours`) + `{1}{B}{G},{T},Sac:` reanimation activation, sorcery-speed gated. 🔍 needs review (oracle previously truncated). |
| Dina's Guidance | {1}{B}{G} | Sorcery |  | Search your library for a creature card, reveal it, put it into your hand or graveyard, then shuffle. | ✅ | Push XXXV: hand-or-graveyard destination now wired as `Effect::ChooseMode` with two modes (Search → Hand vs Search → Graveyard). Reanimator decks can flip to mode 1; default is mode 0 (hand). |
| Essenceknit Scholar | {B}{B/G}{G} | Creature — Dryad Warlock | 3/1 | When this creature enters, create a 1/1 black and green Pest creature token with "Whenever this token attacks, you gain 1 life." / At the beginning of your end step, if a creature died under your control this turn, draw a card. | ✅ | ETB Pest token (with on-attack lifegain rider) + end-step gated draw via the new `Predicate::CreaturesDiedThisTurnAtLeast` (backed by `Player.creatures_died_this_turn`). Hybrid `{B/G}` pip approximated as `{B}` (cost: `{B}{B}{G}`). New `CreatureType::Dryad`. |
| Grapple with Death | {1}{B}{G} | Sorcery |  | Destroy target artifact or creature. You gain 1 life. | ✅ | Wired in `catalog::sets::sos::sorceries`. |
| Lluwen, Exchange Student // Pest Friend | {2}{B}{G} // {B/G} | Legendary Creature — Elf Druid // Sorcery | 3/4 |  | ✅ | Push XL: 🟡 → ✅. Front 3/4 Legendary Elf Druid vanilla + back-face Sorcery `Pest Friend` (now exact `{B/G}` hybrid via `ManaSymbol::Hybrid(Black, Green)`, payable from any pool with at least one of {B} or {G}). The Pest token rides on the shared `pest_token()` helper with the on-attack lifegain rider intact. Closes out the Witherbloom (B/G) school's last 🟡 row. |
| Mind Roots | {1}{B}{G} | Sorcery |  | Target player discards two cards. Put up to one land card discarded this way onto the battlefield tapped under your control. | 🟡 | Push XVII: both halves now wired. Discard half: each opponent discards 2 (player target collapsed to EachOpponent). Land-rider half: `Selector::DiscardedThisResolution(Land)` filtered to one entity via `Selector::one_of(...)`, then moved to your battlefield tapped via `ZoneDest::Battlefield { controller: You, tapped: true }`. The discard tally is bumped by `DiscardChosen` so the per-resolution id list captures the actually-discarded cards. |
| Old-Growth Educator | {2}{B}{G} | Creature — Treefolk Druid | 4/4 | Vigilance, reach / Infusion — When this creature enters, put two +1/+1 counters on it if you gained life this turn. | ✅ | Wired with the new `Predicate::LifeGainedThisTurnAtLeast` Infusion gate on the ETB trigger. |
| Pest Mascot | {1}{B}{G} | Creature — Pest Ape | 2/3 | Trample / Whenever you gain life, put a +1/+1 counter on this creature. | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Professor Dellian Fel | {2}{B}{G} | Legendary Planeswalker — Dellian | [5] | +2: You gain 3 life. / 0: You draw a card and lose 1 life. / −3: Destroy target creature. / −6: You get an emblem with "Whenever you gain life, target opponent loses that much life." | 🟡 | New `PlaneswalkerSubtype::Dellian` + 5 base loyalty. +2 (gain 3 life), 0 (draw 1 / lose 1 life), -3 (destroy target creature) all wired faithfully. The -6 emblem ult is omitted (no emblem zone yet). |
| Root Manipulation | {3}{B}{G} | Sorcery |  | Until end of turn, creatures you control get +2/+2 and gain menace and "Whenever this creature attacks, you gain 1 life." (A creature with menace can't be blocked except by two or more creatures.) | 🟡 | `ForEach(Creature & ControlledByYou) → PumpPT(+2/+2 EOT) + GrantKeyword(Menace EOT)`. The "whenever this creature attacks → gain 1 life" rider is omitted (no transient-trigger-grant primitive yet). |
| Teacher's Pest | {B}{G} | Creature — Skeleton Pest | 1/1 | Menace (This creature can't be blocked except by two or more creatures.) / Whenever this creature attacks, you gain 1 life. / {B}{G}: Return this card from your graveyard to the battlefield tapped. | 🟡 | Menace + attacks-gain-1 trigger wired; graveyard-recursion activated ability omitted. |
| Titan's Grave |  | Land |  | This land enters tapped. / {T}: Add {B} or {G}. / {2}{B}{G}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands`. |
| Vicious Rivalry | {2}{B}{G} | Sorcery |  | As an additional cost to cast this spell, pay X life. / Destroy all artifacts and creatures with mana value X or less. | ✅ | Push XLIII: now uses the new cast-time `additional_life_cost: Some(Value::XFromCost)` primitive (life is debited before the spell goes on the stack). Body is `ForEach(Creature ∨ Artifact).If(ManaValueOf ≤ X) → Destroy`. |
| Witherbloom Charm | {B}{G} | Instant |  | Choose one — / • You may sacrifice a permanent. If you do, draw two cards. / • You gain 5 life. / • Destroy target nonland permanent with mana value 2 or less. | ✅ | All three modes wired faithfully. Mode 0: `Effect::MayDo` sacrifice → draw 2 (push XV). Mode 1: gain 5 life. Mode 2: destroy nonland with mana value ≤ 2. |
| Witherbloom, the Balancer | {6}{B}{G} | Legendary Creature — Elder Dragon | 5/5 | Affinity for creatures (This spell costs {1} less to cast for each creature you control.) / Flying, deathtouch / Instant and sorcery spells you cast have affinity for creatures. | 🟡 | Push XXXVIII: first clause now wires faithfully via the new `StaticEffect::CostReductionScaled { amount: CountOf(EachPermanent(Creature ∧ ControlledByYou)) }` primitive — at 4 friendly creatures, the printed {6}{B}{G} drops to {2}{B}{G}. The second clause ("IS spells you cast have affinity for creatures") still 🟡 — would need a "modify another spell's cost reduction" primitive (a static that adds CostReductionScaled to other casts). |

## Silverquill (White-Black)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Abigale, Poet Laureate // Heroic Stanza | {1}{W}{B} // {1}{W/B} | Legendary Creature — Bird Bard // Sorcery | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Conciliator's Duelist | {W}{W}{B}{B} | Creature — Kor Warlock | 4/3 | When this creature enters, draw a card. Each player loses 1 life. / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, exile up to one target creature. Return that card to the battlefield under its owner's control at the beginning of the next end step. | ✅ | Push XXXVI: Repartee exile + delayed return now both wired via the new `Effect::DelayUntil { capture: Some(Selector::CastSpellTarget(0)) }` field. Trigger fires at next end step and the captured target moves back to bf under owner. |
| Fix What's Broken | {2}{W}{B} | Sorcery |  | As an additional cost to cast this spell, pay X life. / Return each artifact and creature card with mana value X from your graveyard to the battlefield. | ⏳ | Needs: cast-from-graveyard. |
| Forum of Amity |  | Land |  | This land enters tapped. / {T}: Add {W} or {B}. / {2}{W}{B}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands`. |
| Imperious Inkmage | {1}{W}{B} | Creature — Orc Warlock | 3/3 | Vigilance / When this creature enters, surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.) | ✅ | Wired in `catalog::sets::sos::creatures`. |
| Inkling Mascot | {W}{B} | Creature — Inkling Cat | 2/2 | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, this creature gains flying until end of turn. Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Repartee trigger grants Flying (EOT) on `Selector::This` + Surveil 1. |
| Killian's Confidence | {W}{B} | Sorcery |  | Target creature gets +1/+1 until end of turn. Draw a card. / Whenever one or more creatures you control deal combat damage to a player, you may pay {W/B}. If you do, return this card from your graveyard to your hand. | ✅ | Mainline pump+draw + the gy-recursion trigger now both wired. The trigger uses `EventScope::FromYourGraveyard` + `EventKind::DealsCombatDamageToPlayer`; the engine's `fire_combat_damage_to_player_triggers` was extended to walk the attacker's controller's graveyard for `FromYourGraveyard`-scoped triggers. The pay-{W/B}-to-return body uses `Effect::MayPay` with a hybrid `{W/B}` cost; declining or insufficient mana skips silently. Per-card emission (one trigger per attacker that connects). |
| Moment of Reckoning | {3}{W}{W}{B}{B} | Sorcery |  | Choose up to four. You may choose the same mode more than once. / • Destroy target nonland permanent. / • Return target nonland permanent card from your graveyard to the battlefield. | 🟡 | Wired in `catalog::sets::sos::sorceries` as a 2-mode `ChooseMode`. The "choose up to four / same mode more than once" rider is collapsed to "pick one mode and target one permanent" — same-resolution multi-mode replay needs an engine primitive. |
| Nita, Forum Conciliator | {1}{W}{B} | Legendary Creature — Human Advisor | 2/3 | Whenever you cast a spell you don't own, put a +1/+1 counter on each creature you control. / {2}, Sacrifice another creature: Exile target instant or sorcery card from an opponent's graveyard. You may cast it this turn, and mana of any type can be spent to cast that spell. If that spell would be put into a graveyard, exile it instead. Activate only as a sorcery. | ⏳ | 🔍 needs review (oracle previously truncated). Needs: cast-from-exile pipeline. |
| Render Speechless | {2}{W}{B} | Sorcery |  | Target opponent reveals their hand. You choose a nonland card from it. That player discards that card. / Put two +1/+1 counters on up to one target creature. | ✅ | Push XXXV: the "up to one creature target" half now uses `Selector::one_of(EachPermanent(Creature ∧ ControlledByYou))` — auto-picks a friendly creature, no-ops cleanly when none exist. Cast is now legal even when you control no creatures (just discard half fires). |
| Scolding Administrator | {W}{B} | Creature — Dwarf Cleric | 2/2 | Menace (This creature can't be blocked except by two or more creatures.) / Repartee — Whenever you cast an instant or sorcery spell that targets a creature, put a +1/+1 counter on this creature. / When this creature dies, if it had counters on it, put those counters on up to one target creature. | 🟡 | Push XVII: death-trigger counter transfer wired via the new `Value::CountersOn` graveyard fallback. The dying card's `+1/+1` counter count is read off its graveyard-resident copy (counters survive the bf-to-gy transition); the `AddCounter` body adds that many to a target creature, gated on `ValueAtLeast(CountersOn(SelfSource), 1)` so the trigger no-ops when there are no counters. Menace + Repartee +1/+1 unchanged. |
| Silverquill Charm | {W}{B} | Instant |  | Choose one — / • Put two +1/+1 counters on target creature. / • Exile target creature with power 2 or less. / • Each opponent loses 3 life and you gain 3 life. | ✅ | Wired in `catalog::sets::sos::instants`. |
| Silverquill, the Disputant | {2}{W}{B} | Legendary Creature — Elder Dragon | 4/4 | Flying, vigilance / Each instant and sorcery spell you cast has casualty 1. (As you cast that spell, you may sacrifice a creature with power 1 or greater. When you do, copy the spell and you may choose new targets for the copy.) | ✅ | Push XXI: Casualty 1 grant approximated as a magecraft trigger that asks the controller to may-sac a power-≥-1 creature, copying the just-cast spell on yes. Difference vs printed: Casualty's "as you cast" timing is moved to post-cast resolution, which is functionally equivalent for combat math but doesn't double-fire other "when you cast" payoffs. |
| Snooping Page | {1}{W}{B} | Creature — Human Cleric | 2/3 | Repartee — Whenever you cast an instant or sorcery spell that targets a creature, this creature can't be blocked this turn. / Whenever this creature deals combat damage to a player, you draw a card and lose 1 life. | ✅ | Repartee grants `Keyword::Unblockable` (EOT) on the source; combat-damage trigger wired (draw + lose 1). |
| Social Snub | {1}{W}{B} | Sorcery |  | When you cast this spell while you control a creature, you may copy this spell. / Each player sacrifices a creature of their choice. Each opponent loses 1 life and you gain 1 life. | ✅ | Push XXI: mass-sacrifice + drain-1 halves unchanged. The on-cast may-copy rider is now wired via a `SelfSource + SpellCast` triggered ability filtered on `SelectorExists(Creature & ControlledByYou)` whose body is `MayDo { CopySpell { what: CastSpellSource } }`. Copy resolves first then original — each pass independently sacrifices + drains. |
| Stirring Honormancer | {2}{W}{W/B}{B} | Creature — Rhino Bard | 4/5 | When this creature enters, look at the top X cards of your library, where X is the number of creatures you control. Put one of those cards into your hand and the rest into your graveyard. | ✅ | Wired in `catalog::sets::sos::creatures` via `Effect::RevealUntilFind` (find: Creature, cap: count of creatures you control, misses go to graveyard). The hybrid `{W/B}` pip is approximated as `{W}` so cost is `{2}{W}{W}{B}`. |

## Quandrix (Green-Blue)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Applied Geometry | {2}{G}{U} | Sorcery |  | Create a token that's a copy of target non-Aura permanent you control, except it's a 0/0 Fractal creature in addition to its other types. Put six +1/+1 counters on it. | ⏳ | Needs: copy-spell/permanent primitive. |
| Berta, Wise Extrapolator | {2}{G}{U} | Legendary Creature — Frog Druid | 1/4 | Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / Whenever one or more +1/+1 counters are put on Berta, add one mana of any color. / {X}, {T}: Create a 0/0 green and blue Fractal creature token and put X +1/+1 counters on it. | 🟡 | Push XXXI: Increment now wired via `effect::shortcut::increment()`. Counter-add → AnyOneColor mana trigger unchanged. Increment + counter-driven mana ramp creates a self-feeding engine: any 2+ mana spell drops a +1/+1 counter on Berta, the counter triggers an AnyOneColor mana add. X-cost activation wired but X resolves to 0 today (engine has no X-cost activated ability path). |
| Cuboid Colony | {G}{U} | Creature — Insect | 1/1 | Flash / Flying, trample / Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) | ✅ | Push XXXI: Increment now wired via `effect::shortcut::increment()`. Each cast where mana_spent ≥ 2 (one above min(P, T)=1) drops a +1/+1 counter. |
| Embrace the Paradox | {3}{G}{U} | Instant |  | Draw three cards. You may put a land card from your hand onto the battlefield tapped. | ✅ | Push XVI: draw 3 + `MayDo` rider that picks (at most) one land from hand via `Selector::one_of(CardsInZone(Hand, Land))` and moves it to bf tapped. AutoDecider answers "no" by default; `ScriptedDecider::new([Bool(true)])` exercises the paid path in tests. |
| Fractal Mascot | {4}{G}{U} | Creature — Fractal Elk | 6/6 | Trample / When this creature enters, tap target creature an opponent controls. Put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.) | ✅ | Wired in `catalog::sets::sos::creatures` (trample + ETB tap+stun). |
| Fractal Tender | {3}{G}{U} | Creature — Elf Wizard | 3/3 | Ward {2} / Increment (Whenever you cast a spell, if the amount of mana you spent is greater than this creature's power or toughness, put a +1/+1 counter on this creature.) / At the beginning of each end step, if you put a counter on this creature this turn, create a 0/0 green and blue Fractal creature token and put three +1/+1 counters on it. | 🟡 | Push XXXI: Increment now wired via `effect::shortcut::increment()` — a cast at ≥4 mana drops a +1/+1 counter. The end-step Fractal-with-counters rider stays omitted (no per-permanent "did this creature gain a counter this turn" flag yet). Ward(2) keyword tag stays for future Ward enforcement. |
| Geometer's Arthropod | {G}{U} | Creature — Fractal Crab | 1/4 | Whenever you cast a spell with {X} in its mana cost, look at the top X cards of your library. Put one of them into your hand and the rest on the bottom of your library in a random order. | ✅ | Push XVI: trigger fully wired via the new `Predicate::CastSpellHasX` + `RevealUntilFind { cap: XFromCost, to: Hand }`. Misses go to graveyard (engine default for `RevealUntilFind`); the printed "rest to bottom random order" rider stays approximated since the engine has no random-bottom primitive. |
| Growth Curve | {G}{U} | Sorcery |  | Put a +1/+1 counter on target creature you control, then double the number of +1/+1 counters on that creature. | ✅ | Wired in `catalog::sets::sos::sorceries`: AddCounter(+1) then AddCounter(`Value::CountersOn`) faithfully doubles. |
| Mind into Matter | {X}{G}{U} | Sorcery |  | Draw X cards. Then you may put a permanent card with mana value X or less from your hand onto the battlefield tapped. | 🟡 | Draw X wired in `catalog::sets::sos::sorceries` via `Value::XFromCost`. The "may put a permanent ≤ X tapped" half is omitted (no hand-to-battlefield mana-value-gated primitive yet). |
| Paradox Gardens |  | Land |  | This land enters tapped. / {T}: Add {G} or {U}. / {2}{G}{U}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands`. |
| Paradox Surveyor | {G}{G/U}{U} | Creature — Elf Druid | 3/3 | Reach / When this creature enters, look at the top five cards of your library. You may reveal a land card or a card with {X} in its mana cost from among them and put it into your hand. Put the rest on the bottom of your library in a random order. | ✅ | Push XVI: filter promoted to `Land OR HasXInCost` via the new `SelectionRequirement::HasXInCost` primitive — exact-printed reveal filter. Hybrid `{G/U}` pip stays approximated as `{G}` (cost: `{G}{G}{U}`). Misses go to graveyard. |
| Proctor's Gaze | {2}{G}{U} | Instant |  | Return up to one target nonland permanent to its owner's hand. Search your library for a basic land card, put it onto the battlefield tapped, then shuffle. | ✅ | Wired in `catalog::sets::sos::instants`: bounce target nonland to owner's hand, then `Search { filter: IsBasicLand, to: Battlefield(tapped) }`. |
| Pterafractyl | {X}{G}{U} | Creature — Dinosaur Fractal | 1/0 | Flying / This creature enters with X +1/+1 counters on it. / When this creature enters, you gain 2 life. | ✅ | Push XL: 🟡 → ✅. Printed body is now exactly 1/0 (was 1/1 over-statement). The new `CardDefinition.enters_with_counters: Option<(CounterType, Value)>` field lands the X +1/+1 counters at battlefield-entry time *before* SBAs run, so the printed 1/0 body is safe. At X=0 the 1/0 body has 0 toughness with no counters and immediately graveyards (matching printed). The "you gain 2 life" half stays on the ETB trigger. |
| Quandrix Charm | {G}{U} | Instant |  | Choose one — / • Counter target spell unless its controller pays {2}. / • Destroy target enchantment. / • Target creature has base power and toughness 5/5 until end of turn. | 🟡 | Modes 0 (counter unless {2}) and 1 (destroy enchantment) wired in `catalog::sets::sos::instants`. Mode 2 is approximated as a flat +3/+3 EOT (the engine's `Effect::ResetCreature` is a stub, so a true "set base 5/5" rewrite isn't possible yet). |
| Quandrix, the Proof | {4}{G}{U} | Legendary Creature — Elder Dragon | 6/6 | Flying, trample / Cascade (When you cast this spell, exile cards from the top of your library until you exile a nonland card that costs less. You may cast it without paying its mana cost. Put the exiled cards on the bottom in a random order.) / Instant and sorcery spells you cast from your hand have cascade. | 🟡 | Push XIX: body wired (6/6 Flying+Trample Legendary Elder Dragon G/U). Cascade is not yet a first-class engine keyword (no reveal-until-MV-less-than primitive, no cast-from-exile pipeline; tracked in TODO.md push XVIII). The 6/6 Flying+Trample finisher still hits combat correctly at the 6 CMC slot. |
| Tam, Observant Sequencer // Deep Sight | {2}{G}{U} // {G}{U} | Legendary Creature — Gorgon Wizard // Sorcery | 4/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|

## Lorehold (Red-White)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Ark of Hunger | {2}{R}{W} | Artifact |  | Whenever one or more cards leave your graveyard, this artifact deals 1 damage to each opponent and you gain 1 life. / {T}: Mill a card. You may play that card this turn. | 🟡 | Wired against the `EventKind::CardLeftGraveyard` event — drain 1 (1 to each opp + you gain 1) per gy-leave emission. The {T}: Mill activation is wired faithfully; the "may play that card this turn" rider is omitted (same gap as Tablet of Discovery, Suspend Aggression). |
| Aziza, Mage Tower Captain | {R}{W} | Legendary Creature — Djinn Sorcerer | 2/2 | Whenever you cast an instant or sorcery spell, you may tap three untapped creatures you control. If you do, copy that spell. You may choose new targets for the copy. | ✅ | Push XXI: magecraft → MayDo body taps up to 3 untapped friendly creatures + copies the just-cast spell via `Effect::CopySpell { what: Selector::CastSpellSource }`. The "If you do" rider is approximated — if fewer than 3 creatures are available the body still copies (small over-payoff vs printed semantics). |
| Borrowed Knowledge | {2}{R}{W} | Sorcery |  | Choose one — / • Discard your hand, then draw cards equal to the number of cards in target opponent's hand. / • Discard your hand, then draw cards equal to the number of cards discarded this way. | ✅ | Push XXXVI: doc-only promotion — both modes were already fully wired (mode 0 = discard hand + draw = target opp hand size, mode 1 = discard hand + draw = `Value::CardsDiscardedThisResolution`). Status was previously 🟡 from a stale annotation; now ✅. |
| Colossus of the Blood Age | {4}{R}{W} | Artifact Creature — Construct | 6/6 | When this creature enters, it deals 3 damage to each opponent and you gain 3 life. / When this creature dies, discard any number of cards, then draw that many cards plus one. | ✅ | Both ETB drain (3 to each opp + gain 3) and death rider wired faithfully. Death rider uses `Value::CardsDiscardedThisResolution` and `Value::HandSizeOf` to "discard any number" (greedy = entire hand) then draw cards-discarded+1. The "+1" floor matches the printed wording (≥ 1 draw even from an empty hand). |
| Fields of Strife |  | Land |  | This land enters tapped. / {T}: Add {R} or {W}. / {2}{R}{W}, {T}: Surveil 1. (Look at the top card of your library. You may put it into your graveyard.) | ✅ | Wired in `catalog::sets::sos::lands`. |
| Hardened Academic | {R}{W} | Creature — Bird Cleric | 2/1 | Flying, haste / Discard a card: This creature gains lifelink until end of turn. / Whenever one or more cards leave your graveyard, put a +1/+1 counter on target creature you control. | ✅ | All three abilities wired. The cards-leave-graveyard trigger uses the new `EventKind::CardLeftGraveyard` event (per-card emission; "one or more" rider is naturally per-card). |
| Kirol, History Buff // Pack a Punch | {R}{W} // {1}{R}{W} | Legendary Creature — Vampire Cleric // Sorcery | 2/3 |  | 🟡 | Wired in `catalog::sets::sos::mdfcs` (push XI/XII): vanilla front + back-face spell via the new `GameAction::CastSpellBack` path. Original ⏳ note: Standard primitives — should be straightforward to wire.|
| Lorehold Charm | {R}{W} | Instant |  | Choose one — / • Each opponent sacrifices a nontoken artifact of their choice. / • Return target artifact or creature card with mana value 2 or less from your graveyard to the battlefield. / • Creatures you control get +1/+1 and gain trample until end of turn. | ✅ | Wired in `catalog::sets::sos::instants` — all three modes wired with existing primitives (`Sacrifice`, `Move from gy`, `ForEach(Creature) → PumpPT`). |
| Lorehold, the Historian | {3}{R}{W} | Legendary Creature — Elder Dragon | 5/5 | Flying, haste / Each instant and sorcery card in your hand has miracle {2}. (You may cast a card for its miracle cost when you draw it if it's the first card you drew this turn.) / At the beginning of each opponent's upkeep, you may discard a card. If you do, draw a card. | 🟡 | Push XXXVI: per-opp-upkeep loot trigger now wired via `EventScope::OpponentControl + StepBegins(Upkeep)`. Body uses `Effect::MayDo` so the auto-decider's "no" default skips on bot turns; ScriptedDecider yes path drives the discard+draw chain. Miracle grant still omitted (no alt-cost-on-draw primitive). |
| Molten Note | {X}{R}{W} | Sorcery |  | Molten Note deals damage to target creature equal to the amount of mana spent to cast this spell. Untap all creatures you control. / Flashback {6}{R}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | Push XIX: full wire — closes Lorehold's last ⏳ row. Damage half branches on `Predicate::CastFromGraveyard` (push XVIII): hand cast deals `XFromCost + 2` (the X plus the {R}{W} portion); flashback cast deals 8 (the fixed {6}{R}{W} mana spent). Untap-all-your-creatures via `Effect::Untap` on `EachPermanent(Creature & ControlledByYou)`. Flashback {6}{R}{W} wired via `Keyword::Flashback`. |
| Practiced Scrollsmith | {R}{R/W}{W} | Creature — Dwarf Cleric | 3/2 | First strike / When this creature enters, exile target noncreature, nonland card from your graveyard. Until the end of your next turn, you may cast that card. | 🟡 | Wired in `catalog::sets::sos::creatures`. ETB now exiles **exactly one** matching noncreature/nonland card in the controller's graveyard via `Selector::Take(_, 1)` (push X) — closer to the printed "target one card" semantics. Push XL: hybrid `{R/W}` pip now wired faithfully via `ManaSymbol::Hybrid(Red, White)` — payable from R+W, R+R, or W+W pools (matching printed legality). The "may cast until next turn" rider is omitted (no cast-from-exile-with-time-limit primitive). |
| Pursue the Past | {R}{W} | Sorcery |  | You gain 2 life. You may discard a card. If you do, draw two cards. / Flashback {2}{R}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.) | ✅ | All three clauses wired. Gain 2 life + `Effect::MayDo` discard+draw chain (push XV) + `Keyword::Flashback({2}{R}{W})`. The lifegain half always resolves; the loot half is opt-in. |
| Spirit Mascot | {R}{W} | Creature — Spirit Ox | 2/2 | Whenever one or more cards leave your graveyard, put a +1/+1 counter on this creature. | ✅ | Wired against the new `EventKind::CardLeftGraveyard` event. Trigger fires per-card emission (the printed "one or more" wording is approximated per-card). |
| Startled Relic Sloth | {2}{R}{W} | Creature — Sloth Beast | 4/4 | Trample, lifelink / At the beginning of combat on your turn, exile up to one target card from a graveyard. | ✅ | Wired in `catalog::sets::sos::creatures` (trample + lifelink + begin-combat exile-from-GY trigger; same shape as Ascendant Dustspeaker's combat trigger). Sloth subtype bridged through Beast (no Sloth creature type yet). |
| Suspend Aggression | {1}{R}{W} | Instant |  | Exile target nonland permanent and the top card of your library. For each of those cards, its owner may play it until the end of their next turn. | 🟡 | Wired in `catalog::sets::sos::instants` as a `Seq` of two `Move → Exile` calls (target nonland permanent + caster's top of library). `move_card_to` was extended to walk libraries when locating the source card so the top-of-library exile resolves end-to-end. The "may play those cards until next end step" rider is omitted (no per-card "may-play-from-exile-until-EOT" primitive). |
| Wilt in the Heat | {2}{R}{W} | Instant |  | This spell costs {2} less to cast if one or more cards left your graveyard this turn. / Wilt in the Heat deals 5 damage to target creature. If that creature would die this turn, exile it instead. | ✅ | Push XXXIX: cost-reduction now wires faithfully via the new `Value::IfPredicate` + `StaticEffect::CostReductionScaled` (self-static — same shape as Ajani's Response). With one or more cards in the gy this turn the cost drops to {R}{W}. The "if it would die, exile instead" damage-replacement rider stays gap (no damage-replacement primitive). |

## Colorless

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Biblioplex Tomekeeper | {4} | Artifact Creature — Construct | 3/4 | When this creature enters, choose up to one — / • Target creature becomes prepared. (Only creatures with prepare spells can become prepared.) / • Target creature becomes unprepared. | 🟡 | Push XIX: body wired (3/4 Construct artifact creature at {4}). The ETB Prepare/Unprepare toggle is omitted (Prepare keyword pending — see "Prepare mechanic" below). Body alone slots into colorless midrange shells. |
| Diary of Dreams | {2} | Artifact — Book |  | Whenever you cast an instant or sorcery spell, put a page counter on this artifact. / {5}, {T}: Draw a card. This ability costs {1} less to activate for each page counter on this artifact. | 🟡 | Page-counter accrual on instant/sorcery cast (counter type approximated as Charge — engine has no Page counter) + flat {5},{T} draw. The page-counter-scaled cost reduction is omitted (no self-counter cost-reduction primitive). |
| Great Hall of the Biblioplex |  | Land |  | {T}: Add {C}. / {T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast an instant or sorcery spell. / {5}: If this land isn't a creature, it becomes a 2/4 Wizard creature with "Whenever you cast an instant or sorcery spell, this creature gets +1/+0 until end of turn." It's still a land. | 🟡 | Push XV: legendary colorless utility land. `{T}: Add {C}` faithful + `{T}, Pay 1 life: Add one mana of any color` via the new `ActivatedAbility.life_cost: u32` field — the effect is a pure mana ability (`AddMana(AnyOneColor 1)`) so it resolves immediately without going on the stack. The `{5}: becomes 2/4 Wizard creature` clause is omitted (no land-becomes-creature primitive — same gap as Mishra's Factory). The spend-restriction rider on the rainbow ability is omitted (no per-pip mana metadata yet). |
| Mage Tower Referee | {2} | Artifact Creature — Construct | 2/1 | Whenever you cast a multicolored spell, put a +1/+1 counter on this creature. | ✅ | Wired in `catalog::sets::sos::creatures` with a `SpellCast/YourControl` trigger filtered on `EntityMatches(TriggerSource, Multicolored)` — uses the new `SelectionRequirement::Multicolored` predicate (≥ 2 distinct colored pips, hybrid both halves, Phyrexian colored side). Mono-color and colorless casts don't bump the Referee. |
| Page, Loose Leaf | {2} | Legendary Artifact Creature — Construct | 0/2 | {T}: Add {C}. / Grandeur — Discard another card named Page, Loose Leaf: Reveal cards from the top of your library until you reveal an instant or sorcery card. Put that card into your hand and the rest on the bottom of your library in a random order. | 🟡 | Body wired (0/2 Legendary Construct artifact creature) + `{T}: Add {C}` mana ability via the shared `tap_add_colorless()` helper. Grandeur (discard-named-this-card-as-cost activation) omitted. |
| Petrified Hamlet |  | Land |  | When this land enters, choose a land card name. / Activated abilities of sources with the chosen name can't be activated unless they're mana abilities. / Lands with the chosen name have "{T}: Add {C}." / {T}: Add {C}. | 🟡 | Mana ability `{T}: Add {C}` wired via the new shared `tap_add_colorless()` helper in `catalog::sets`. The "choose a land card name" prompt + name-keyed lock-out static + name-keyed grant of `{T}: Add {C}` on opp lands are all omitted (no name-prompt decision, no name-match selector). The card still slots into colorless utility roles. |
| Potioner's Trove | {3} | Artifact |  | {T}: Add one mana of any color. / {T}: You gain 2 life. Activate only if you've cast an instant or sorcery spell this turn. | 🟡 | Mana ability + lifegain ability wired; the "if you've cast an instant or sorcery this turn" gate on the lifegain activation is omitted (no per-turn-cast gate on activated abilities yet). |
| Rancorous Archaic | {5} | Creature — Avatar | 2/2 | Trample, reach / Converge — This creature enters with a +1/+1 counter on it for each color of mana spent to cast it. | ✅ | Wired in `catalog::sets::sos::creatures` (trample/reach + ETB AddCounter using `Value::ConvergedValue`). Powered by the engine's new `StackItem::Trigger.converged_value` plumbing. |
| Skycoach Waypoint |  | Land |  | {T}: Add {C}. / {3}, {T}: Target creature becomes prepared. (Only creatures with prepare spells can become prepared.) | 🟡 | Push XIX: `{T}: Add {C}` mana ability wired via the shared `tap_add_colorless()` helper. The `{3},{T}: prepare a creature` activation is omitted (Prepare keyword pending — same gate as Biblioplex Tomekeeper). |
| Strixhaven Skycoach | {3} | Artifact — Vehicle | 3/2 | Flying / When this Vehicle enters, you may search your library for a basic land card, reveal it, put it into your hand, then shuffle. / Crew 2 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.) | 🟡 | Push XIX: 3/2 Flying artifact-creature body wired + ETB tutor (`Effect::MayDo` + `Effect::Search { filter: IsBasicLand, to: Hand }`). The Vehicle subtype + Crew keyword are not yet engine concepts (no Vehicle/Crew primitive); the card enters as a plain artifact creature directly — a small over-statement in vacuum, but the ETB tutor + 3/2 Flying body still slot into colorless ramp. |
| Sundering Archaic | {6} | Creature — Avatar | 3/3 | Converge — When this creature enters, exile target nonland permanent an opponent controls with mana value less than or equal to the number of colors of mana spent to cast this creature. / {2}: Put target card from a graveyard on the bottom of its owner's library. | 🟡 | Push XVI: `{2}: gy → bottom of owner's library` activated ability now wired via `Effect::Move { what: Target(0), to: ZoneDest::Library { who: OwnerOf(Target(0)), pos: Bottom } }`. ETB Converge exile is wired against `Nonland & ControlledByOpponent`; the mana-value cap against `ConvergedValue` is still approximated to "any nonland opp permanent" (no `Value`-keyed `ManaValueAtMost` predicate yet — tracked in TODO.md). |
| The Dawning Archaic | {10} | Legendary Creature — Avatar | 7/7 | This spell costs {1} less to cast for each instant and sorcery card in your graveyard. / Reach / Whenever The Dawning Archaic attacks, you may cast target instant or sorcery card from your graveyard without paying its mana cost. If that spell would be put into your graveyard, exile it instead. | 🟡 | Push XXXVIII: ⏳ → 🟡. New `catalog::sets::sos::creatures::the_dawning_archaic` factory wires the 7/7 Reach Avatar at {10} with the printed self-discount via the new `StaticEffect::CostReductionScaled { amount: CountOf(CardsInZone(Graveyard, IS-cards)) }`. With 5 IS cards in your graveyard, the printed {10} drops to {5}. The attack-trigger cast-from-graveyard rider stays gap pending the cast-from-exile/graveyard pipeline (same family as Velomachus Lorehold, Conspiracy Theorist). |
| Together as One | {6} | Sorcery |  | Converge — Target player draws X cards, Together as One deals X damage to any target, and you gain X life, where X is the number of colors of mana spent to cast this spell. | 🟡 | Damage and life-gain halves wired in `catalog::sets::sos::sorceries`; the "target player draws X" half collapses to "you draw X" (multi-target prompt gap). |
| Transcendent Archaic | {7} | Creature — Avatar | 6/6 | Vigilance / Converge — When this creature enters, you may draw X cards, where X is the number of colors of mana spent to cast this spell. If you draw one or more cards this way, discard two cards. | 🟡 | Body wired (6/6 Vigilance Avatar). ETB Converge draw is wired via `Value::ConvergedValue`; the conditional discard 2 is gated on `ConvergedValue ≥ 1`. The "you may" optionality is collapsed to always-draw-when-X-≥-1 (no may-do primitive yet). |

## Strixhaven base set (STX)

Strixhaven 2021 cards (separate from the supplemental SOS catalog above).
Cards live under `catalog::sets::stx` and use the same engine primitives
as the SOS module. The two catalogs are independent — bringing them up to
parity is a matter of porting card factories one at a time.

### Silverquill (W/B)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Silverquill Apprentice | {W}{B} | ✅ | Push XXXVII: 🟡 → ✅. Magecraft "+1/+1 or -1/-1" now wires faithfully via the new `Effect::PickModeAtResolution([+1/+1 EOT, -1/-1 EOT])` primitive. AutoDecider picks mode 0 (pump — combat-trick safe default); ScriptedDecider flips mode 1 for the printed shrink line. |
| Spirited Companion | {1}{W} | ✅ | 1/2 Dog Spirit. ETB: draw a card. |
| Eyetwitch | {B} | ✅ | 1/1 Pest. When dies: "learn" approximated as `Draw 1` (no Lesson sideboard yet). |
| Closing Statement | {X}{W}{W} | ✅ | Sorcery. Exile target nonland permanent. You gain X life (`Value::XFromCost`). |
| Vanishing Verse | {W}{B} | ✅ | Push XX: target filter is now `Permanent ∧ Nonland ∧ Monocolored` via the new `SelectionRequirement::Monocolored` predicate. Two-color and colorless permanents reject as invalid targets at cast time. |
| Killian, Ink Duelist | {W}{B} | ✅ | Push XXXVIII: 🟡 → ✅. 2/3 Legendary Human Warrior with Lifelink. "Spells you cast that target a creature cost {2} less" now wires faithfully via the new `StaticEffect::CostReductionTargeting { spell_filter: Any, target_filter: Creature, amount: 2 }` primitive. `cost_reduction_for_spell` walks each battlefield permanent's static abilities (controller-scoped) plus the cast card's own static abilities at cast time. Multiple Killians stack ({4} less); colored requirements always remain. |
| Devastating Mastery | {4}{W}{W} | ✅ | Push XXXVIII: 🟡 → ✅. Sorcery. Wired as a 2-mode `Effect::ChooseMode` where mode 0 is the printed Wrath and mode 1 is Wrath + reanimate. The Mastery alt cost ({7}{W}{W}) now wires the printed reanimate via the new `AlternativeCost.mode_on_alt: Some(1)` field — paying the alt cost auto-selects mode 1. |
| Felisa, Fang of Silverquill | {2}{W}{B} | ✅ | 4/3 Legendary Cat Cleric, Flying + Lifelink. Push XVI: counter-bearing-creature-dies → Inkling trigger now wired via `EventKind::CreatureDied/AnotherOfYours` filtered by `EntityMatches { what: TriggerSource, filter: WithCounter(+1/+1) }`. Counters persist on a card after move-to-graveyard (only `damage`/`tapped`/`attached_to` are cleared on zone-out), so the post-die graveyard-resident card still reports its `+1/+1` counters via `evaluate_requirement_static`. |
| Mavinda, Students' Advocate | {1}{W}{W} | 🟡 | 1/3 Legendary Human Cleric, Flying + Vigilance. Cast-from-graveyard activated ability is ⏳. |
| Eager First-Year | {W} | ✅ | 2/1 Human Student. Magecraft: target creature gets +1/+1 EOT. Uses the new `effect::shortcut::magecraft()` helper. |
| Hunt for Specimens | {3}{B} | ✅ | Push XXIV: promoted from 🟡 to ✅. Creates a 1/1 black Pest token whose on-die +1-life trigger rides on `TokenDefinition.triggered_abilities` (SOS push VI), then Learn → Draw 1 (same Lesson approximation as Eyetwitch / Igneous Inspiration). |
| Silverquill Command | {2}{W}{B} | ✅ | Push XXXVI: "choose two" now wires faithfully via the new `Effect::ChooseModes { count: 2 }` primitive. Auto-decider picks modes 0+1 (counter ability + -3/-3 EOT). ScriptedDecider drives modes [2, 3] (drain + draw) for tests. |
| Star Pupil | {B} | ✅ | Push XL: 🟡 → ✅ via the new `enters_with_counters` replacement (push XL). Printed body is now exactly 0/0; two +1/+1 counters land at bf entry before the 0-toughness SBA fires. Dies trigger unchanged — `EventKind::CreatureDied/SelfSource` → +1/+1 counter on a targeted creature. |
| Codespell Cleric | {W} | ✅ | Push XXV: 1/1 Human Cleric, Lifelink. ETB Scry 1. All three pieces are first-class engine primitives. |
| Combat Professor | {3}{W} | ✅ | Push XXV: 2/3 Cat Cleric with Flying. Magecraft +1/+1 EOT on target creature (same shape as Eager First-Year, just on a 2/3 flier body). |
| Clever Lumimancer | {W} | ✅ | Push XXX: 1/1 Human Wizard. Magecraft +2/+2 EOT on self via `magecraft_self_pump(2, 2)`. Aggressive one-mana self-pump magecraft body (1 → 3 → 5 → ... per IS spell cast that turn). |
| Dueling Coach | {2}{W} | ✅ | Push XXX: 3/3 Human Cleric with Vigilance. Magecraft +1/+1 counter on target creature (any side) — same shape as Lecturing Scornmage / Stonebinder's Familiar's magecraft-counter family on a meatier {3} body with Vigilance. |
| Hall Monitor | {W} | ✅ | Push XXX: 1/1 Human Wizard. Magecraft "target creature can't block this turn" — wired via `Effect::GrantKeyword(Keyword::CantBlock, EOT)` (same primitive Duel Tactics uses). Auto-target picks the largest opposing blocker. |
| Karok Wrangler | {2}{W} | ✅ | Push XXXI: promoted from 🟡 to ✅. The "if you control two or more Wizards, additional stun counter" rider now wires via `Effect::If` gated on `ValueAtLeast(CountOf(EachPermanent(Creature ∧ Wizard ∧ ControlledByYou)), 2)`. Karok itself counts toward the threshold — solo Karok lands 1 stun, Karok next to any other Wizard lands 2. The "instead" wording is approximated as additive (1 base + 1 conditional rather than a strict swap), but stun counters stack 1-for-1 against future untap steps so combat-correct. |

### Witherbloom (B/G)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Witherbloom Apprentice | {B}{G} | ✅ | 2/2 Human Warlock. Magecraft: drain 1 (each opponent loses 1; you gain 1). |
| Pest Summoning | {B}{G} | ✅ | Sorcery (Lesson). Creates two 1/1 Pest tokens; the death-trigger lifegain rider rides on the token via SOS-VI's `TokenDefinition.triggered_abilities`. |
| Witherbloom Pledgemage | {1}{B}{G} | ✅ | Push XXIV: promoted from 🟡 to ✅. `{T}, Pay 1 life: Add {B}` now uses `ActivatedAbility.life_cost: 1` (push XV primitive) — the activation rejects pre-pay with `InsufficientLife` when life < 1, mirroring the mana-cost pre-pay check. The "{B} or {G}" mode pick collapses to {B} (modal-mana primitive ⏳). |
| Daemogoth Titan | {3}{B}{G} | ✅ | Push XXXI: "or blocks" rider now wired via the new `EventKind::Blocks` event. Both attack-side and block-side triggers run the same body — sacrifice a non-titan creature you control. Combat-correct in every defender scenario, not just attack-only swings. |
| Pest Infestation | {X}{B}{G} | ✅ | Push XXIV: Sorcery. Create X 1/1 black-green Pest tokens with on-die +1-life trigger (X = `Value::XFromCost`). Each Pest carries its death trigger via `TokenDefinition.triggered_abilities`. |
| Witherbloom Command | {B}{G} | ✅ | Push XXXVI: "choose two" now wires faithfully via the new `Effect::ChooseModes { count: 2 }` primitive. Auto-decider picks modes 0+1 (drain 3 + gy → hand). ScriptedDecider drives modes [2, 3] for tests. |
| Bayou Groff | {2}{B}{G} | ✅ | 5/4 Beast. Push XVI: "may pay {1} on death to return to hand" rider now wired via the new `Effect::MayPay` primitive (sibling to push XV's `Effect::MayDo`). On the death trigger, the controller is asked yes/no; on yes + sufficient mana, the engine pays {1} and `Move(SelfSource → Hand(OwnerOf(Self)))`. |
| Daemogoth Woe-Eater | {2}{B}{G} | ✅ ← 🟡 | Push XXXIX: additional sacrifice cost now paid at *cast* time via the new `CardDefinition.additional_sac_cost` field — `cast_spell` rejects with `SelectionRequirementViolated` when the controller has no creature to sacrifice. The auto-pick lands the lowest-value matching creature (tokens first, then by mana value, then by power). 9/9 Demon body + `{T}: gain 4 life` unchanged. |
| Eyeblight Cullers | {1}{B}{B} | ✅ ← 🟡 | Push XXXIX: same shape as Woe-Eater — additional sacrifice at cast time. The double-counted ETB sac has been dropped; the drain rider stays unchanged. |
| Dina, Soul Steeper | {B}{G} | ✅ | Push XXX: promoted from 🟡 to ✅. The activated -X/-X EOT now scales with `Value::Diff(Const(0), CountOf(EachPermanent(Creature ∧ ControlledByYou)))` (was flat -1/-1). At three creatures-you-control the activation shrinks the target by -3/-3 EOT (hard kill on most early-game blockers); at five creatures it's -5/-5. Lifegain → opp-loses-1 trigger unchanged. |
| Mortality Spear | {3}{B}{G} | ✅ | Push XXX: Sorcery — Lesson. "Destroy target creature or planeswalker." Wired with `Effect::Destroy` on a `Creature OR Planeswalker` filter (same shape as Hero's Downfall / Mage Hunters' Onslaught). Lesson sub-type recorded so future Lesson-aware code (Mascot Exhibition's Lesson filter, Learn payoffs) can filter on it. |
| Foul Play | {2}{B} | ✅ | Push XXX: Instant. "Destroy target tapped creature. If you control two or more Wizards, draw a card." Wired with `Effect::Seq([Destroy(Creature ∧ Tapped), If(≥2 Wizards, Draw 1)])` — the gate uses the existing `Predicate::ValueAtLeast(CountOf(EachPermanent(Creature ∧ HasCreatureType(Wizard) ∧ ControlledByYou)), Const(2))` shape (same family as Galvanic Blast's metalcraft gate). All Strixhaven Wizards (Codespell Cleric, Hall Monitor, Karok Wrangler, Symmetry Sage, Spectacle Mage, etc.) feed the gate via tribal type-line matching. |

### Lorehold (R/W)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Lorehold Apprentice | {R}{W} | ✅ | Push XXXIII: 1/1 Human Cleric. Magecraft now wires both clauses faithfully: gain 1 life + `DealDamage(1, any_target())`. The `any_target()` helper (`Creature ∨ Planeswalker ∨ Player`) routes the damage through the auto-target picker — opp face by default; falls through to creatures / planeswalkers when face damage isn't legal. |
| Lorehold Pledgemage | {1}{R}{W} | ✅ | Push XXXIV: 2/2 Spirit Cleric with Reach. Activated `{2}{R}{W}, exile a card from your graveyard: +1/+1 EOT` now wires via the new `ActivatedAbility::exile_gy_cost: u32` field — pre-flight gate rejects with `GameError::InsufficientGraveyard` when the controller has 0 cards in their graveyard, otherwise picks the oldest gy card (index 0) and moves it to exile after tap/mana/life payment. |
| Pillardrop Rescuer | {3}{R}{W} | ✅ | 3/3 Spirit Cleric with Flying. ETB: return target instant or sorcery card from your graveyard to your hand. |
| Heated Debate | {2}{R} | ✅ | Instant. 4 damage to target creature. Damage-can't-be-prevented rider is a no-op (engine has no prevention layer). |
| Storm-Kiln Artist | {2}{R}{W} | ✅ | Push XXXIII: 3/3 Human Wizard. Magecraft now uses `any_target()` (`Creature ∨ Planeswalker ∨ Player`) for the printed "1 damage to any target" — auto-target prefers opp face, falls through to creatures / planeswalkers when needed. Treasure follow-up unchanged. |
| Sparring Regimen | {2}{R}{W} | ✅ | Push XVII: both abilities wired. ETB creates a 2/2 R/W Spirit token; "whenever you attack, +1/+1 on each attacker" now fires per-attacker via the new combat-side broadcast in `declare_attackers` — the trigger source is Sparring Regimen, the target is pre-bound to the just-declared attacker as `Target(0)`. Net result: each declared attacker ends up with one new counter, matching the printed mass pump. |
| Reconstruct History | {1}{R}{W} | ✅ | Push XXIII: return up to 2 artifact cards from your gy → hand via `Selector::take(_, 2)` over `CardsInZone(Graveyard, Artifact)` + draw 1. |
| Igneous Inspiration | {2}{R} | ✅ | Push XXIII: 3 dmg to creature/PW + Learn (collapsed to draw 1). |
| Lorehold Command | {R}{W} | ✅ | Push XXXVI: "choose two" now wires faithfully via the new `Effect::ChooseModes { count: 2 }` primitive. Auto-decider picks modes 0+1 (drain 4 + 2 Spirit tokens). ScriptedDecider drives modes [2, 3] for tests. |
| Rip Apart | {R}{W} | ✅ | Push XXIX: Sorcery. Choose one — 3 damage to target creature/planeswalker, or destroy target artifact/enchantment. Wired with `Effect::ChooseMode` (same shape as Boros Charm) and Or-composite filters on each mode's target. Modal pick is "choose one" (printed) so it ships at full fidelity. |
| Plargg, Dean of Chaos | {1}{R} | 🟡 | Push XLII: 1/3 Legendary Human Wizard. {T}-rummage activation unchanged. The {2}{R} second activation now wires faithfully via `Effect::Seq([LookAtTop(3), Move(TopOfLibrary{1} → Exile)])` — looks at top 3, exiles top 1 (auto-decider always exiles; the printed "may exile one of three" interactive picker collapses to greedy-exile since the value of the activation is the exile, not the abstain). The "may play that exiled card until end of turn" rider stays gap (no per-card may-play-from-exile-with-time-limit primitive — same family as Suspend Aggression / Tablet of Discovery / Outpost Siege / Conspiracy Theorist). The DFC pairing with Augusta, Dean of Order is still split into two separate front-face card definitions. |
| Augusta, Dean of Order | {1}{W} | ✅ | Push XXX: promoted from 🟡 to ✅ via the new `Value::AttackersThisCombat` primitive. The per-attacker pump trigger is now gated by `Predicate::ValueAtLeast(AttackersThisCombat, 2)` — single-attacker swings no longer false-positive. Two-or-more attacker swings: each attacker passes the gate and ends up with +1/+1 + double strike EOT (matches printed). `combat.rs` was extended to evaluate broadcast Attack-trigger filters in a second pass, so the `attacking.len()` reading is uniform across all attackers. |
| Hofri Ghostforge | {2}{R}{W} | 🟡 | Push XXXVIII: anthem now exact via the new `excluded_supertypes: Vec<Supertype>` field on `AffectedPermanents::All` — `Not(HasSupertype(Legendary))` decomposes at static-layer translation time so legendary friendly creatures are correctly skipped. The dies-as-Spirit-copy rider stays omitted (token-copy-of-permanent primitive gap, same as Phantasmal Image / Mockingbird in CUBE_FEATURES.md), keeping the card 🟡 overall. |
| Mascot Interception | {2}{R}{W} | ✅ | Push XXXIV: Instant. Printed "gain control of opp's creature + untap + haste" now wires faithfully — `Effect::GainControl` graduated from a permanent-control-flip stub to a Layer-2 continuous effect with `EffectDuration::UntilEndOfTurn`, so the steal reverts at Cleanup. Body is `Seq([GainControl, Untap, GrantKeyword(Haste)])` — control change first so the untap and haste land on the freshly-stolen creature. (Haste-grant-expiration is tracked separately — `Effect::GrantKeyword` still mutates `card.definition.keywords` directly without honoring its `duration` field; see TODO.md push XXXIV.) |
| Approach of the Lorehold | {1}{R}{W} | ✅ | Push XXX: Sorcery. 2 damage to each opponent (auto-target collapse — printed "any target") + creates a 1/1 white Spirit creature token with flying. Lorehold's flexible utility sorcery; same Spirit token shape as Lorehold Command's mode 1. |

### Quandrix (G/U)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Quandrix Apprentice | {G}{U} | ✅ | 1/1 Elf Druid. Magecraft: target creature you control gets +1/+1 EOT. |
| Quandrix Pledgemage | {1}{G}{U} | ✅ | 2/2 Fractal Wizard. Activated `{1}{G}{U}: +1/+1 counter on this creature`. |
| Decisive Denial | {G}{U} | ✅ ← 🟡 | Push XXXIX: cast-time legality now enforces the slot 0 friendly-creature filter via the new `val_find` arm of `target_filter_for_slot_in_mode` — the engine recurses into `DealDamage.amount`'s `Value::PowerOf(target_filtered(...))` to pull the slot 0 filter at cast time, so opp-creature targets in mode 1 reject with `SelectionRequirementViolated`. Mode 0 unchanged (counter-noncreature-unless-{2}). Mode 1's damage half stays one-sided (friendly takes no return damage). |
| Snow Day | {1}{G}{U} | ✅ | Push XXIII: Instant. Create a 0/0 Fractal token + put X +1/+1 counters on it where X = `Value::HandSizeOf(You)`. With a 7-card hand the Fractal lands as a 7/7. |
| Mentor's Guidance | {2}{G}{U} | ✅ | Push XXXVI: doc-only promotion. Sorcery. Draw 2 + put hand-size +1/+1 counters on a target creature you control. The printed Oracle is single-target, so the existing wire matches printed exactly — the prior 🟡 was a stale annotation that misread "for each card in your hand" as multi-target fan-out. |
| Quandrix Command | {1}{G}{U} | ✅ | Push XXXVI: "choose two" now wires faithfully via the new `Effect::ChooseModes { count: 2 }` primitive. Auto-decider picks modes 0+1 (counter ability + +1/+1 ×2). ScriptedDecider drives modes [2, 3] for tests. |
| Augmenter Pugilist | {3}{G}{G} | ✅ | Push XXXVII: 🟡 → ✅. Static "Activated abilities of creatures cost {2} more to activate" now wires via the new `StaticEffect::TaxActivatedAbilities { filter: Creature, amount: 2 }`. `extra_cost_for_activation` walks every battlefield permanent's static abilities at activation time and surcharges the activator's mana cost when the activating permanent matches the filter. Mana abilities are NOT exempt per the rules — Llanowar Elves's `{T}: Add {G}` becomes `{2}, {T}: Add {G}` while Pugilist is in play. |
| Biomathematician | {1}{G}{U} | ✅ NEW | 2/2 Vedalken Druid. "When this creature dies, create a 0/0 green and blue Fractal creature token. Put two +1/+1 counters on it." Wired via `EventKind::CreatureDied/SelfSource` → `Effect::Seq([CreateToken(Fractal), AddCounter(LastCreatedToken, +1/+1, ×2)])`. The Fractal lands as a 2/2 because the two counters resolve in the same effect Seq. Same shape as Pestbrood Sloth's death-trigger token-mint, with the `LastCreatedToken` counter stamp on top. Closes Quandrix (G/U) at 8 ✅ / 0 🟡. |

### Prismari (U/R)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Prismari Pledgemage | {1}{U}{R} | ✅ | 2/3 Elemental with Trample + Haste. |
| Prismari Apprentice | {U}{R} | ✅ | Push XXXVII: 🟡 → ✅. Magecraft "Scry 1 or +1/+0 EOT" now wires faithfully via the new `Effect::PickModeAtResolution`. AutoDecider picks mode 0 (Scry 1 — universal safe default); ScriptedDecider flips mode 1 for the +1/+0 self-pump combat trick. |
| Symmetry Sage | {U} | ✅ | 1/2 Human Wizard. Magecraft: this creature gets +1/+0 and gains flying until end of turn. |
| Creative Outburst | {3}{U}{U}{R}{R} | ✅ | Push XXIII: Sorcery. Discard your hand (`Discard { amount: HandSizeOf(You) }`), draw 5. Prismari spellslinger refill that fuels later magecraft / flashback payoffs. |
| Prismari Command | {1}{U}{R} | ✅ | Push XXXVI: "choose two" now wires faithfully via the new `Effect::ChooseModes { count: 2 }` primitive. Auto-decider picks modes 0+1 (2 dmg + discard 2/draw 2). ScriptedDecider drives modes [2, 3] for tests. |
| Magma Opus | {7}{U}{R} | 🟡 | Push XXIX: Sorcery finisher. Wired body: 4 damage to creature/PW, mint a 4/4 Elemental token, draw 2. The "4 damage divided" + "tap two target permanents" both collapse to single-target picks. The discard-for-Treasure alt cost ({U}{R}, Discard) is omitted (no alt-cost-by-discard primitive yet — same gap as Bonecrusher Giant's Adventure). |
| Expressive Iteration | {U}{R} | 🟡 | Push XXIX: Sorcery — collapsed to Scry 2 + Draw 1 cantrip approximation. The "exile top 3 + you may play a land + cast a spell from among them" rider is omitted (cast-from-exile + play-land-from-exile primitive gap, same family as Augur of Bolas / Outpost Siege). |

### Mono-color staples (`stx::mono`)

| Card | Cost | Status | Notes |
|---|---|---|---|
| Pop Quiz | {1}{W} | ✅ | Sorcery (Lesson). Draw 2, then put a card from your hand on top of your library. |
| Mascot Exhibition | {5}{W}{W} | ✅ | Sorcery. Creates a 3/3 Elephant, 2/2 lifelink Cat, and 1/1 flying Bird. |
| Plumb the Forbidden | {X}{B}{B} | ✅ | Instant. Sacrifice X creatures, draw X cards, lose X life. |
| Owlin Shieldmage | {3}{W} | ✅ ← 🟡 | Push XLI: 2/3 Bird Wizard, Flash + Flying. ETB now triggers the new `Effect::PreventCombatDamageThisTurn` primitive — sets the `GameState.combat_damage_prevented_this_turn` flag (CR 615 sticky shield), which `resolve_combat_damage_with_filter` short-circuits per attacker so no damage events fire. Cleared on cleanup. Same primitive powers Holy Day + Spore Frog in the modern catalog. |
| Frost Trickster | {1}{U} | ✅ | 2/2 Spirit Wizard, Flash + Flying. ETB taps + stuns target opponent's creature. |
| Body of Research | {4}{G}{U} | ✅ | Push XVI: now uses the new `Value::LibrarySizeOf(You)` primitive — Fractal token enters with one +1/+1 counter per library card, matching the printed Oracle exactly (was approximating via `GraveyardSizeOf` since `LibrarySize` wasn't a primitive). |
| Show of Confidence | {1}{W} | ✅ | Instant. Adds `StormCount + 1` +1/+1 counters on target creature you control. |
| Bury in Books | {3}{U} | ✅ | Sorcery. Put target creature on top of its owner's library. |
| Test of Talents | {1}{U}{U} | 🟡 | Counter target instant or sorcery; the search-and-exile-by-name follow-up is ⏳. |
| Multiple Choice | {1}{U}{U} | ✅ | Push XXXVI: "choose one or more" now wires faithfully via the new `Effect::ChooseModes { count: 3, up_to: true, allow_duplicates: false }` primitive. Auto-decider picks all 3 modes (Scry 2 + pump+hexproof + Pest token). The "if you chose all of the above" mega-mode rider stays gap (would need modes-picked introspection). |
| Solve the Equation | {2}{U} | ✅ | Push XXIII: Sorcery. Search library for an instant or sorcery, put it into your hand, then scry 1. |
| Enthusiastic Study | {1}{G} | ✅ | Push XXIII: Instant. Target creature gets +2/+2 and gains trample EOT, then learn (collapses to draw 1). |
| Tempted by the Oriq | {1}{W}{B} | ✅ | Push XXXVIII: 🟡 → ✅. Sorcery. The printed "gain control of target creature with mana value 3 or less" now wires faithfully via `Effect::GainControl { duration: Duration::Permanent }` (Push XXXIV pioneered EOT-bounded GainControl on Mascot Interception; this is the indefinite flavor for Bribery / Mind Control / Tempted). The Inkling token rider is wired via the SOS catalog's `inkling_token()` helper. |
| Brilliant Plan | {3}{U} | ✅ | Push XXVI: Sorcery. Scry 3 + Draw 3 — pure card-selection sorcery (STX 2021 mono-blue). Wired via `Effect::Seq([Scry(3), Draw(3)])`. |
| Saw It Coming | {1}{U}{U} | 🟡 | Push XXIV: Instant. Counter target spell (Cancel-equivalent at the {1}{U}{U} rate). Foretell {1}{U} alt-cost is omitted (no Foretell primitive: would need alt-cost-on-exile + cast-from-exile-with-time-limit, same gap as Velomachus Lorehold's reveal-and-cast). |
| Environmental Sciences | {2} | ✅ | Push XXIX: colorless Sorcery (Lesson). `Effect::Search(IsBasicLand → Hand) + GainLife 2`. Universal Lesson at every color — every Strixhaven Mystical Archive deck plays this regardless of pip requirements. |
| Expanded Anatomy | {3}{G} | ✅ | Push XXIX: Sorcery (Lesson). `Effect::AddCounter(Creature, +1/+1, ×3)` — three +1/+1 counters on a target creature. |
| Big Play | {3}{G}{U} | ✅ ← 🟡 | Push XXXIX: fidelity bump. The "up to two creatures" rider now applies to two friendly creatures via `ForEach + Selector::take(EachPermanent(Creature ∧ ControlledByYou), 2)` fan-out body. Each picked creature untaps + +1/+1 + hexproof + trample EOT. Lesson sub-type unchanged. |
| Confront the Past | {4}{R} | 🟡 | Push XXIX: Sorcery (Lesson). Mode 0 only — counter target activated/triggered ability via `Effect::CounterAbility`. The "steal a planeswalker loyalty ability" mode is omitted (dynamic mode-pick from a target's `loyalty_abilities` list is a brand-new primitive, same family as Sarkhan, the Masterless's static loyalty stamp). |
| Pilgrim of the Ages | {3}{W} | ✅ | Push XXIX: 2/3 Spirit Wizard Cleric. Death-trigger basic-land recursion (`CreatureDied/SelfSource → Move(Selector::take(CardsInZone(Graveyard, IsBasicLand), 1) → Hand)`). Mirrors Pillardrop Rescuer's Lorehold-themed graveyard-recursion shape on a mono-white slot. |
| Vortex Runner | {1}{U} | ✅ | Push XXXII: 1/2 Salamander Warrior. `Keyword::Unblockable` (printed "can't be blocked") + `Attacks/SelfSource → Scry 1` attack-trigger card selection. |
| Burrog Befuddler | {1}{U} | ✅ | Push XXXII: 1/3 Frog Wizard with Flash + magecraft `PumpPT(-2, 0, EOT)` on a target creature. Combat-trick magecraft body. |
| Crackle with Power | {X}{R}{R}{R} | ✅ | Push XXXII: Sorcery. `DealDamage` to any target with amount = `Times(XFromCost, 5)`. At X=3 → 15 damage. Routes through the new lethal-first auto-target picker for "any target" creature kills. |
| Sundering Stroke | {3}{R}{R}{R} | 🟡 | Push XXXII: 7 damage to one target. Multi-target divided-damage variant + the {R}{R}{R}{R}-spent doubling rider both omitted (no divided-damage primitive — same gap as Magma Opus). |
| Professor of Symbology | {1}{W} | 🟡 | Push XXXII: 1/1 Bird Wizard with Flying. ETB Learn collapsed to Draw 1 (Lesson sideboard model not yet present). |
| Professor of Zoomancy | {1}{G} | ✅ | Push XXXII: 1/1 Squirrel Wizard. ETB mints a 1/1 green Squirrel creature token. |
| Leyline Invocation | {4}{G} | ✅ | Push XXXII: Sorcery — Lesson. Mints a 0/0 green Elemental token + stamps it with N +1/+1 counters where N = `Value::CountOf(EachPermanent(Land ∧ ControlledByYou))` via `Selector::LastCreatedToken`. Net: an X/X Elemental at curve. Token P/T are locked at creation time (same shape as Body of Research / Snow Day). |
| Verdant Mastery | {3}{G}{G} | 🟡 | Push XXXII: Sorcery. Search basic land tapped to bf + search basic land to hand (two `Effect::Search` calls). Opponent half + the {7}{G}{G} alt cost both omitted. |
| Rise of Extus | {3}{W}{B} | ✅ | Push XXXII: Sorcery — Lesson. `Seq([Exile(Creature OR Planeswalker), Move(graveyard pick → battlefield)])` — exile + reanimate combo, single-target on each half. |
| Gnarled Professor | {3}{G} | ✅ | Push XXXII: 4/4 Treefolk Druid with Reach. ETB `MayDo(Discard 1 → Draw 1)`. AutoDecider defaults "no"; ScriptedDecider can flip "yes" for tests. |
| Inkfathom Witch | {2}{B} | ✅ | Push XXXII: 2/2 Faerie Warlock with Flying. `Attacks/SelfSource → MayPay({1}{B}, Drain 2)` — attack-trigger optional drain. |
| Blood Researcher | {1}{B} | ✅ | Push XXXII: 1/1 Vampire Wizard. `LifeGained/YourControl → AddCounter(This, +1/+1, ×1)` — lifegain-payoff body. |
| First Day of Class | {W} | ✅ | Push XXXII: Sorcery. Two `ForEach` passes over `EachPermanent(IsToken ∧ Creature ∧ ControlledByYou)`: PumpPT(+1/+1, EOT) + GrantKeyword(Haste, EOT). Targets *only* token creatures. |
| Pest Wallop | {3}{G} | ✅ NEW | Push XXXIX: Sorcery. Friendly creature gets +1/+1 EOT (slot 0), then deals damage = its power to an opp creature (auto-picked via `Selector::one_of(EachPermanent(opp creature))`). Slot 0 friendly-creature filter is enforced at cast time via the new `val_find` recursion (push XXXIX). One-sided damage (not Fight). |
| Solid Footing | {W} | ✅ NEW | Push XXXIX: Aura. Enchanted creature gets +1/+2 + vigilance via `StaticEffect::PumpPT` + `StaticEffect::GrantKeyword` over `Selector::AttachedTo(This)`. ETB attachment is now pre-bound at cast time by the engine (`stack.rs`) so the orphaned-aura SBA (CR 704.5m) doesn't immediately graveyard the card. First catalog Aura that uses the static-attach layer pattern. |
| Swarm Shambler | {G} | ✅ NEW | Push XXXIX: 1/1 Beast (Squirrel approximated through Beast). ETB +1/+1 counter; `{2}{G}: untap + add a +1/+1 counter`. Mono-green growth body that scales with available mana. |
| Containment Breach | {1}{W} | ✅ NEW | Push XXXIX: Instant. Destroy target enchantment + Learn (collapses to Draw 1, same approximation as Eyetwitch / Hunt for Specimens). |
| Unwilling Ingredient | {B} | ✅ NEW | Push XXXIX: 1/1 Insect Pest. When this creature dies, may pay {B}: draw a card. Death-trigger uses `Effect::MayPay`. AutoDecider declines by default; ScriptedDecider yes path drives the cantrip. |
| Quick Study | {1}{U} | ✅ NEW | Push XLII: Sorcery — Lesson. Trivial mono-blue cantrip wired as `Effect::Draw(Const(2))`. Functional twin of Divination at the same printed rate; the Lesson sub-type tag opens future Lesson-aware effects. |
| Introduction to Prophecy | {3}{U} | ✅ NEW | Push XLII: Sorcery — Lesson. `Effect::Seq([Scry 4, Draw 1])` — mono-blue card-selection rare. Slots into Strixhaven mystic-arcane shells alongside Brilliant Plan (Scry 3 + Draw 3). |
| Introduction to Annihilation | {3}{R} | ✅ NEW | Push XLII: Sorcery — Lesson. Universal exile (any permanent) + the *target's controller* draws 1. Wired via `Selector::Player(ControllerOf(Target(0)))` reading the cast-time target's `controller` field (preserved post-exile). |
| Soothsayer Adept | {1}{U} | ✅ NEW | Push XLII: 1/2 Merfolk Wizard. `{U}: Scry 1` activated ability — repeatable card-selection on any free blue pip. Same shape as Hedron Crab's repeatable mill body. |
| Drainpipe Vermin | {B} | ✅ NEW | Push XLII: 1/1 Rat. Death-trigger `EachOpponent → Mill 2` — Witherbloom self-sacrifice / "leaves graveyard" enabler at the printed common rate. |
| Make Your Move | {B}{G} | ✅ NEW | Push XLII: Instant. "Choose one or both" wired faithfully via `Effect::ChooseModes { count: 2, up_to: true }` — mode 0 destroys a tapped creature (`Creature ∧ Tapped` filter), mode 1 destroys an enchantment. AutoDecider picks both modes when both targets exist. |
| Returned Pastcaller | {4}{B} | ✅ NEW | Push XLII: 4/3 Zombie Wizard. ETB returns a MV ≤ 3 IS card from your graveyard to hand via `Selector::take(CardsInZone(Graveyard, IS ∧ MV ≤ 3), 1)`. Pure recursion body — no double-counted ETB drain. |
| Field Research | {1}{W} | ✅ NEW | Push XLII: Sorcery — Lesson. `Effect::Seq([AddCounter(target Creature, +1/+1, ×1), GainLife(2)])` — printed Oracle exact. White Lesson card-selection at the {1}{W} rate. |
| Mage Duel | {R} | 🟡 NEW | Push XLII: Instant. 2 damage to an opp creature (via the existing `Creature ∧ ControlledByOpponent` filter). The Magecraft "may pay {R}{R} on the spell itself, copy it" rider stays gap (would need a self-spell magecraft trigger that fires *during* the same cast — same family gap as Devastating Mastery's Mastery alt-cost-on-the-spell-itself). |
| Archmage Emeritus | {2}{U}{U} | ✅ NEW | Push XLIV: 3/3 Human Wizard. Magecraft — draw a card. Pure draw-engine body wired via `magecraft(Effect::Draw 1)`; every IS spell the controller casts (or copies) replaces itself. |
| Fortifying Draught | {2}{W} | ✅ NEW | Push XLIV: Instant — Lesson. `Effect::Seq([GainLife 4, Scry 2])`. Mono-white life-buffer cantrip. The Lesson sub-type tag opens future Lesson-aware effects. |
| Sage of Mysteries | {U} | ✅ NEW | Push XLIV: 1/2 Spirit Wizard. Magecraft — target opponent mills 2. Wired via `magecraft(Effect::Mill { who: EachOpponent, amount: 2 })`; with one opponent the EachOpponent collapse matches printed exactly. |

### Shared / multi-college

| Card | Cost | Status | Notes |
|---|---|---|---|
| Inkling Summoning | {3}{W}{B} | ✅ | Sorcery (Lesson). Creates a 2/1 white-and-black Inkling token with flying. |
| Spirit Summoning | {3}{W} | ✅ | Push XXV: Sorcery — Lesson. Creates a 1/1 white Spirit creature token with flying. White's slot in the STX Lesson cycle (siblings: Pest Summoning B/G, Inkling Summoning W/B, Mascot Exhibition W). |
| Tend the Pests | {1}{B}{G} | ✅ | Sacrifice a creature; create X 1/1 Pest tokens (X = sacrificed power); "When this dies, gain 1 life" trigger now rides on the token via SOS-VI's `TokenDefinition.triggered_abilities`. |

### Iconic / legendary (`stx::iconic` + `stx::legends`)

Cards added in the latest push that didn't fit a single college file —
each college's flagship Dragon, plus a few cross-college staples.

| Card | Cost | Status | Notes |
|---|---|---|---|
| Strict Proctor | {1}{W} | 🟡 | 1/3 Spirit Cleric, Flying. ETB-tax replacement is omitted (no replacement-effect primitive). |
| Sedgemoor Witch | {2}{B}{B} | ✅ | 3/2 Human Warlock, Menace + Ward(1) keyword. Magecraft creates a Pest token. Ward enforcement still pending — keyword tag is correct. Test: `sedgemoor_witch_magecraft_creates_pest_token`. |
| Spectacle Mage | {U/R}{U/R} | ✅ | Push XXXVIII: 🟡 → ✅. 1/2 Human Wizard. Hybrid {U/R}{U/R} cost now wired exactly via two `ManaSymbol::Hybrid(Blue, Red)` pips (the engine's mana payment path already supports hybrid pips). Prowess is now first-class — `fire_spell_cast_triggers` sweeps every Keyword::Prowess permanent controlled by the caster on each noncreature spell cast and pushes a synthetic +1/+1 EOT pump. |
| Mage Hunters' Onslaught | {2}{B}{B} | ✅ | Sorcery. Destroy target creature; draw a card. Test: `mage_hunters_onslaught_destroys_creature_and_draws_card`. |
| Galazeth Prismari | {2}{U}{R} | 🟡 | 3/4 Legendary Dragon Wizard, Flying. ETB creates a Treasure token (full real-card behaviour). The "artifacts you control are mana sources" static is still ⏳ (no `GrantActivatedAbility(applies_to)` primitive). Test: `galazeth_prismari_is_three_four_flying_dragon_with_etb_treasure`. |
| Beledros Witherbloom | {3}{B}{B}{G}{G} | ✅ | Push XX: 6/6 Legendary Demon, Flying + Trample + Lifelink. "Pay 10 life: Untap each land you control. Activate only as a sorcery." now wired via push XV's `ActivatedAbility.life_cost: u32` gate (rejects with `InsufficientLife` < 10) + `Effect::Untap` over `Selector::EachPermanent(Land & ControlledByYou)`. Sorcery-speed flag set true to match printed restriction. |
| Velomachus Lorehold | {3}{R}{R}{W} | 🟡 | 5/5 Legendary Dragon, Flying + Vigilance + Haste. Attack-trigger reveal-and-cast is ⏳ (cast-from-exile-without-paying primitive). |
| Tanazir Quandrix | {2}{G}{G}{U}{U} | 🟡 | 5/5 Legendary Dragon, Flying + Trample. ETB +1/+1-counter doubling is ⏳ (no counter-multiplier primitive). |
| Shadrix Silverquill | {2}{W}{B} | ✅ | Push XXXVII: 🟡 → ✅. 4/4 Legendary Dragon Flying + Double Strike. Choose-2-of-3 attack trigger now wires faithfully via `Effect::ChooseModes { count: 2 }` re-used at trigger resolution. AutoDecider picks modes 0+1 (draw + drain — canonical value pair). ScriptedDecider drives mode pairs that involve targeting (mode 1 needs an opp; mode 2 needs a creature). |

### Engine pieces driven by STX

- ✅ **`effect::shortcut::magecraft(effect)` helper** — bundles the
  spell-cast trigger with `cast_is_instant_or_sorcery()`, so card
  factories use a one-liner. Used by Eager First-Year and Witherbloom
  Apprentice; future Apprentices (Lorehold, Prismari, Quandrix) will
  reuse it.
- ✅ **Token death-trigger lifegain** — `TokenDefinition` now carries
  `triggered_abilities` (added in SOS push VI). The STX Pest token's
  "die → gain 1" trigger fires consistently for Pest Summoning, Tend
  the Pests, Hunt for Specimens. SOS Pest token's "attack → gain 1"
  rider also rides on every minted copy.
- ⏳ **Lesson sideboard model** — Eyetwitch, Hunt for Specimens, Pest
  Summoning all use Learn at some point. Currently approximated as
  `Draw 1`.
