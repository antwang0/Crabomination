# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.
See `CUBE_FEATURES.md` (cube-card implementation status) and
`STRIXHAVEN2.md` (Secrets-of-Strixhaven status).

## Recent additions

- ✅ **Push XLVI (2026-05-04)**: 16 new modern cards + AnotherOfYours
  ETB de-dup + CR 605 (Mana Abilities) audit. Tests at 1468 (was
  1445; +23 net), all green.
  - **12 new modern cards** (`catalog::sets::decks::modern`):
    Toxic Deluge ✅ ({X}{2}{B} Sorcery — pay X life, all creatures
    -X/-X EOT); Supreme Verdict 🟡 ({1}{W}{W}{U} Sorcery — destroy
    all creatures; "can't be countered" rider gap); Devour Flesh 🟡
    ({1}{B} Sorcery — opp sacrifices creature; toughness-loss rider
    gap); Brought Back 🟡 ({W}{W} Instant — return up to 2
    permanent cards from gy; "this turn" filter approximated);
    Persist 🟡 ({1}{B}{G} Sorcery — reanimate ≤3-MV creature; -1/-1
    counter rider gap); Selfless Spirit ✅ ({1}{W} 2/1 Spirit Flying
    — sac for indestructible team-wide); Hangarback Walker ✅
    ({X}{X} 0/0 Construct — enters with X +1/+1 counters; on death
    mints X Thopter tokens, validates the AnotherOfYours fix);
    Utter End ✅ ({2}{W}{B} Instant — exile any nonland);
    Vraska's Contempt ✅ ({3}{B} Instant — exile creature/PW + 2
    life); Cut Down ✅ ({B} Instant — destroy ≤3-MV creature);
    Stitcher's Supplier ✅ ({B} 1/1 Zombie — mill 3 on ETB + death,
    validates the AnotherOfYours fix); Soul Warden ✅ ({W} 1/1
    Cleric — gain 1 per "another creature" ETB);
    Pyroblast ✅ + Red Elemental Blast ✅ ({R} Instant — color hate:
    counter blue spell or destroy blue permanent);
    Hydroblast ✅ + Blue Elemental Blast ✅ ({U} Instant — counter
    red spell or destroy red permanent).
  - **Engine fix: AnotherOfYours ETB de-dup** —
    `game/mod.rs::is_event_hardcoded` now skips `PermanentEntered
    + AnotherOfYours` triggers in the generic event-matching
    walker. Previously the trigger was processed in BOTH the
    hardcoded `stack.rs` ETB path (line 389-433) AND the generic
    `event_matches_spec` walker, causing every "another creature
    enters" trigger (Soul Warden, Felisa Fang, Pestbrood Sloth,
    Arnyn Deathbloom Botanist's death-trigger, etc.) to fire
    twice. The fix mirrors the existing SelfSource skip — both ETB
    scopes are owned by the hardcoded path.
  - **New CreatureType: Thopter** — added to support Hangarback
    Walker's death-trigger token mint.
  - **CR 605 audit (Mana Abilities)**: full per-rule status in
    STRIXHAVEN2.md. Highlights: 605.1/605.1a (mana ability
    structural definition) ✅, 605.2 (still mana ability when
    blocked) ✅, 605.3a (can activate mid-cast/mid-resolve) ✅,
    605.3b (off-stack resolve) ✅, 605.5a (targeted ≠ mana ability)
    ✅, 605.5b (spells ≠ mana abilities) ✅. Still 🟡: 605.1b /
    605.4 / 605.4a (triggered mana abilities — no catalog card
    exercises this path).
  - **23 new tests**: 14 + 4 + 5 color-blast mode tests (Pyroblast
    counter, Pyroblast destroy, Hydroblast counter, REB destroy, BEB
    destroy).

- ✅ **Push XLV (2026-05-04)**: 8 new modern cards + 2 SOS promotions +
  bot life-cost preflight + CR 120 (Damage) audit. Tests at 1445
  (was 1430; +15 net), all green.
  - **8 new modern cards** (`catalog::sets::decks::modern`):
    Abrade ✅ ({1}{R} Instant — modal: 3 dmg / destroy artifact);
    Izzet Charm ✅ ({U}{R} Instant — 3-mode counter / 2 dmg / loot 2);
    Pillar of Flame 🟡 ({R} Sorcery — 2 dmg, exile-if-dies omitted);
    Smash to Smithereens ✅ ({1}{R} Instant — destroy artifact + 3 to
    its controller); Forked Bolt 🟡 ({R} Sorcery — divided damage
    collapsed to single-target); Knight of Meadowgrain ✅ ({W}{W}
    First Strike + Lifelink); Defiant Strike ✅ ({W} Instant — pump +
    cantrip); Fanatical Firebrand ✅ ({R} Goblin Pirate, Haste, sac-
    to-ping).
  - **2 SOS promotions (🟡 → ✅)**: Brush Off (cost reduction now
    wires via `StaticEffect::CostReductionTargeting` for IS-spell
    targets — same self-static shape as Ajani's Response); Run
    Behind (same cost-reduction primitive shape, target filter is
    `Creature ∧ IsAttacking`).
  - **Bot improvement: `additional_life_cost` pre-flight**.
    `can_afford_in_state` now rejects spells whose life cost would
    crash the controller below 1 life. Builds a temporary
    `EffectContext` with `x_value = max_affordable_x` so X-cost
    life payments (Vicious Rivalry's `XFromCost`) evaluate against
    the upper-bound the bot would actually pump to. Mirrors the
    push XXXIX `additional_sac_cost` and push XLIII
    `additional_discard_cost` pre-flight rejections.
  - **CR 120 (Damage) audit**: full per-rule status in
    STRIXHAVEN2.md push XLV. Highlights: 120.1 (object can deal
    damage) ✅, 120.2 (any object) ✅, 120.3a (player damage →
    life loss) ✅, 120.3c (damage to walker → loyalty) ✅, 120.3e
    (damage marks on creatures) ✅, 120.3f (lifelink) ✅, 120.5
    (damage doesn't destroy — SBA does) ✅, 120.6 (marks persist
    till cleanup) ✅, 120.7 (source) ✅. Still 🟡: 120.3b/d
    (Infect/Wither routing — keyword exists but damage path doesn't
    branch), 120.4a/b (trample-excess + spell prevention), 120.8
    (0-damage skip), 120.9 (per-source damage tally). Still ⏳:
    120.3g (Toxic), 120.3h (Battles), 120.10 (excess-damage
    triggers).
  - **15 new tests**: 11 modern card tests (Abrade modes 0/1, Izzet
    Charm modes 1/2, Pillar of Flame, Smash to Smithereens, Forked
    Bolt, Knight of Meadowgrain definition, Defiant Strike pump+draw,
    Fanatical Firebrand sac-ping + definition), 2 SOS promotion
    tests (Brush Off and Run Behind cost-reduction targeting), 2 bot
    pre-flight tests (Vicious Rivalry rejected at 1 life; accepted
    with comfortable buffer).

- ✅ **Push XLIV (2026-05-04)**: 10 new cards + 2 engine bug fixes +
  view-side `additional_life_cost` rendering. Tests at 1430 (was
  1415; +15 net).
  - **Engine fix #1: AnyPlayer SpellCast trigger** —
    `fire_spell_cast_triggers` previously folded `EventScope::
    AnyPlayer` onto `YourControl` (`c.controller == caster`),
    meaning Eidolon-style "whenever a player casts X, …" triggers
    *never* fired on opponent casts. The fix accepts the match
    unconditionally for AnyPlayer scope. Same family of gap
    blocked Mindbreak Trap, Sanctum Prelate, Ethersworn Canonist
    sibling triggers from firing symmetrically.
  - **Engine fix #2: ControllerOf(EntityRef::Card)** —
    `PlayerRef::ControllerOf(sel)` only handled
    `EntityRef::Permanent`, returning `None` when the inner
    selector resolved to `EntityRef::Card` (stack-resident
    spells via `Selector::TriggerSource`). The fix walks the
    stack for the matching `StackItem::Spell` and returns the
    `caster`; falls back to battlefield + owner. Together with
    the AnyPlayer fix, this lets Eidolon's body
    `DealDamage { to: Player(ControllerOf(TriggerSource)), … }`
    resolve to the spell's caster correctly.
  - **10 new cards**:
    - **STX 2021 (3 ✅)**: Archmage Emeritus ({2}{U}{U} 3/3 magecraft
      → draw 1), Fortifying Draught ({2}{W} Instant — Lesson; gain
      4 + scry 2), Sage of Mysteries ({U} 1/2 magecraft → opp
      mills 2).
    - **Modern (7)**: Serum Visions ✅ ({U} draw 1 + scry 2);
      Burst Lightning 🟡 ({R} 2 damage; kicker omitted); Roiling
      Vortex 🟡 ({R} Enchantment, upkeep ping + sac activation;
      "players can't gain life" lock omitted); Murderous Cut 🟡
      ({4}{B} destroy creature; Delve omitted); Eidolon of the
      Great Revel ✅ ({R}{R} 2/2, MV-≤-3 ping — validates the
      AnyPlayer + ControllerOf fixes end-to-end); Wild Slash ✅
      ({R} 2 damage); Krenko, Mob Boss ✅ ({2}{R}{R} 3/3 Goblin
      doubler).
  - **Server view enrichment**: `KnownCard.additional_cost_label`
    now also surfaces `additional_life_cost` (push XLIII
    primitive). Vicious Rivalry's `XFromCost` life cost renders
    as "Pay X life". Combines with sac / discard labels via
    " and " when multiple are present.
  - **CR 603 audit (Handling Triggered Abilities)**: full per-
    rule status in STRIXHAVEN2.md push XLIV. Highlights: 603.1
    (shape) ✅, 603.2 (auto-trigger) ✅, 603.2c (one trigger
    per event) ✅, 603.4 (intervening 'if') ✅, 603.5 (may) ✅,
    603.6 (zone-change) ✅, 603.7 (delayed) ✅. Still 🟡:
    603.2g (no trigger on prevented events — only combat
    damage prevention is wired), 603.3b (APNAP ordering),
    603.8 (state triggers), 603.10 (look-back-in-time
    triggers — partial).

- ✅ **Push XLIII (2026-05-04)**: cast-time additional-cost primitive
  trio + 10 promotions + bot/view ergonomics + CR 118.8 audit. Tests
  at 1415 (was 1405; +10 net).
  - **Two new engine primitives** (sister fields to push XXXIX's
    `additional_sac_cost`):
    - **`CardDefinition.additional_discard_cost: Option<u32>`** —
      number of cards to discard at cast time. Pre-flight rejects
      with `SelectionRequirementViolated` when hand is too small;
      auto-pick is the first N cards in hand. Each discard emits a
      `CardDiscarded` event so madness / discard-trigger listeners
      fire pre-resolution.
    - **`CardDefinition.additional_life_cost: Option<Value>`** — life
      to pay at cast time. The `Value` is evaluated against the
      cast-time `EffectContext` (X is read from the spell's `{X}`
      pip), then debited; emits a `LifeLost` event so loss-of-life
      listeners fire pre-resolution. Loss-of-game from crossing zero
      life is left to the next SBA pass per CR 119.4.
  - **10 cards promoted** (7 🟡 → ✅, 3 fidelity bumps; full list in
    STRIXHAVEN2.md push XLIII):
    Thrilling Discovery, Cathartic Reunion, Necrotic Fumes,
    Tormenting Voice, Wild Guess, Thrill of Possibility, Big Score,
    Crop Rotation, Mine Collapse, Vicious Rivalry.
  - **Bot ergonomics**: `can_afford_in_state` now pre-flight-rejects
    spells whose `additional_discard_cost` exceeds the controller's
    non-spell hand count. Mirrors the existing
    `additional_sac_cost` rejection path.
  - **Server view ergonomics**: `KnownCard.additional_cost_label`
    now combines sac + discard labels with " and " when both are
    present. Singular ("Discard a card") vs. plural ("Discard 2
    cards") via bare-count formatting. New `additional_discard_cost_
    label(n)` renderer mirrors the `additional_sac_cost_label(filter)`
    shape from push XXXIX.
  - **CR 118.8 audit (additional costs)**: rule-by-rule status; full
    breakdown in STRIXHAVEN2.md. The big wins: 118.8 (additional
    cost listed on a spell) ✅, 118.8d (mana cost unchanged) ✅,
    601.2f (total cost determination) ✅, 601.2h (pay all costs in
    any order) ✅. Stacked additional costs (118.8a) and optional
    additional costs (118.8b — kicker / buyback) stay 🟡.
  - **10 new tests**: 6 STX (Thrilling Discovery / Cathartic Reunion
    / Necrotic Fumes accept + reject paths) + 2 bot pre-flight gate
    tests + 2 view label tests.

- ✅ **Push XLII (2026-05-04)**: 9 new STX cards + Plargg fidelity
  bump + 1 server-view enrichment (`PermanentView.loyalty`) + CR 306
  audit. Tests at 1405 (was 1389, +16 net).
  - **9 new STX 2021 cards** (`catalog::sets::stx::mono`):
    - **Quick Study** ({1}{U} Sorcery — Lesson) — `Effect::Draw(2)`.
      Functional twin of Divination at the printed common rate.
    - **Introduction to Prophecy** ({3}{U} Sorcery — Lesson) — `Scry 4
      + Draw 1`. Mono-blue card-selection rare.
    - **Introduction to Annihilation** ({3}{R} Sorcery — Lesson) —
      Universal exile + the *target's controller* draws 1. Wired via
      `Selector::Player(ControllerOf(Target(0)))` reading the cast-time
      target's `controller` field (preserved post-exile).
    - **Soothsayer Adept** ({1}{U}, 1/2 Merfolk Wizard) — `{U}: Scry 1`
      repeatable card-selection.
    - **Drainpipe Vermin** ({B}, 1/1 Rat) — Death-trigger
      `EachOpponent → Mill 2`. Witherbloom self-sacrifice enabler.
    - **Make Your Move** ({B}{G} Instant) — "Choose one or both"
      destroy-tapped-creature / destroy-enchantment via
      `Effect::ChooseModes { count: 2, up_to: true }`.
    - **Returned Pastcaller** ({4}{B}, 4/3 Zombie Wizard) — ETB returns
      a MV ≤ 3 IS card from gy → hand via `Selector::take(CardsInZone(
      Graveyard, IS ∧ MV ≤ 3), 1)`.
    - **Field Research** ({1}{W} Sorcery — Lesson) — `+1/+1 counter
      on target creature, then gain 2 life`.
    - **Mage Duel** ({R} Instant) 🟡 — 2 damage to opp creature. The
      Magecraft "may pay {R}{R} on this spell, copy it" rider stays
      gap (would need a self-spell magecraft trigger that fires
      *during* the same cast — same family as Devastating Mastery's
      Mastery alt-cost-on-the-spell-itself).
  - **Plargg, Dean of Chaos** ({1}{R}, 1/3 Legendary Human Wizard) —
    second activation `{2}{R}: Look at top 3, exile top 1` now wires
    via `Effect::Seq([LookAtTop(3), Move(TopOfLibrary{1} → Exile)])`.
    Auto-decider always exiles the topmost (closest fidelity without
    an interactive "may exile one of three" picker). The "may play
    that exiled card until EOT" rider stays gap (same family as
    Suspend Aggression / Tablet of Discovery / Outpost Siege /
    Conspiracy Theorist). Stays 🟡 overall.
  - **Server view**: `PermanentView.loyalty: Option<i32>` — top-level
    field showing current loyalty for planeswalkers (CR 306.5c).
    Non-planeswalkers leave the field `None`. Defaulted via
    `#[serde(default)]` for back-compat with older serialized views.
    Client (`counter_tooltip.rs`) prefers the new field, falling back
    to scanning `counters` for older views.
  - **Doc-only fix**: Practiced Scrollsmith's note now reflects the
    wired `{R/W}` hybrid pip (push XL); the prior note still claimed
    the hybrid was approximated as `{R}` (cost: `{R}{R}{W}`) when in
    fact `cost(&[r(), hybrid(Red, White), w()])` has been on the
    factory since push XL.
  - **CR 306 audit (Planeswalkers)**: rule-by-rule:
    - 306.1 (cast as a spell) ✅ — PWs go through `cast_spell`.
    - 306.2 (puts onto bf) ✅ — standard spell-resolution path.
    - 306.3 (subtypes) ✅ — `PlaneswalkerSubtype` enum with per-PW
      tags (Liliana, Tibalt, Dellian, Ral).
    - 306.4 (legend rule applies) ✅ — `Supertype::Legendary` SBA.
    - 306.5a (loyalty off-bf = printed) ✅ — `base_loyalty` field.
    - 306.5b ("enters with N loyalty" replacement effect) 🟡 —
      hardcoded in `CardInstance::new` rather than threaded through
      the `enters_with_counters` replacement-effect primitive (push
      XL). Means counter-multiplier effects (Doubling Season-style)
      won't multiply loyalty. Future work: align loyalty placement
      with the `enters_with_counters` cast-time hook.
    - 306.5c (loyalty = # loyalty counters) ✅ — `counter_count(
      CounterType::Loyalty)`. Push XLII surfaces this in
      `PermanentView.loyalty`.
    - 306.5d (one loyalty ability per turn) ✅ —
      `used_loyalty_ability_this_turn` flag, reset at cleanup.
    - 306.6 (PW can be attacked) ✅ — `AttackTarget::Planeswalker`.
    - 306.7 (no redirect; direct damage) ✅ — `target_filtered`
      filters allow PW targets directly; no implicit redirect.
    - 306.8 (combat damage = loyalty counter removal) ✅ —
      `combat.rs` `AttackTarget::Planeswalker(pw_id)` branch.
    - 306.9 (0 loyalty → graveyard SBA) ✅ — `stack.rs` SBA pass.
  - **16 new tests**: 14 STX card tests (Plargg ×2 + 12 cards) +
    2 view tests (PermanentView.loyalty for PW + None for non-PW).

- ✅ **Push XLI (2026-05-04)**: 1 engine primitive
  (`Effect::PreventCombatDamageThisTurn`) + 1 new STX card
  (Biomathematician) + 6 promotions/new fog cards (Owlin Shieldmage
  🟡 → ✅; Holy Day, Spore Frog NEW; Monastery Swiftspear,
  Stormchaser Mage Prowess promotions; Faerie Mastermind 🟡 → ✅
  via skip-first-opp-draw gate) + new audit script
  (`scripts/audit_stx_base.py`). Tests at 1389 (was 1379, +10 net).
  - **Engine: `Effect::PreventCombatDamageThisTurn`** —
    `GameState.combat_damage_prevented_this_turn: bool` flag, set
    by the new effect, sticky for the rest of the turn (cleared in
    `do_cleanup` per CR 615). `resolve_combat_damage_with_filter`
    short-circuits per attacker when the flag is on, so no damage
    events fire (lifelink, infect, trample-trigger riders all skip
    too). New `GameEvent::CombatDamagePreventedThisTurn` (+ wire
    mirror) for spectator UI rendering. Defaulted via
    `#[serde(default)]` for older snapshots.
  - **NEW STX 2021**: Biomathematician ({1}{G}{U}, 2/2 Vedalken
    Druid) — death-trigger creates a 0/0 Fractal token + ×2 +1/+1
    counter stamp. Closes Quandrix (G/U) at 8 ✅ / 0 🟡 — first STX
    college with no remaining partials.
  - **STX 2021 promotion**: Owlin Shieldmage ({3}{W} 2/3 Bird
    Wizard, Flash + Flying) 🟡 → ✅. ETB triggers the new prevention
    primitive.
  - **NEW Modern**: Holy Day ({W} Instant — Alpha-era fog) and
    Spore Frog ({G} 1/1 Frog — sac-as-cost activation) both wired
    on the same prevention primitive. Validates the primitive works
    through both instant-speed cast and sacrifice activation paths.
  - **Modern promotions**: Monastery Swiftspear + Stormchaser Mage
    Prowess doc/test promotions (push XXXVIII Prowess wiring works
    for these too — they had stale "engine work pending" comments).
    Faerie Mastermind 🟡 → ✅: skip-first-opp-draw gate now wired
    via `Predicate::ValueAtLeast(CardsDrawnThisTurn(Triggerer), 2)`
    — opp's 1st draw skips, 2nd+ fires.
  - **Tooling**: New `scripts/audit_stx_base.py` (sibling to SOS
    audit). 0 false positives, 0 false negatives across 111 STX
    rows. 96 ✅ / 15 🟡 / 0 ⏳ totals.
  - **Cleanup**: 2× clippy `extend(drain)` → `append`, 1× doc
    rewrap to dodge `doc_lazy_continuation`.
  - **CR 615 audit** (Prevention Effects): the new primitive
    implements 615.1/615.1a (continuous prevention), 615.4
    (pre-event check), 615.6 (no event when prevented). Still ⏳:
    615.7 (specific-amount shields like "prevent the next 3
    damage"), 615.8 (next-instance-from-source), 615.9
    (property-recheck shields), 615.13 (triggers on prevention).

- ✅ **Push XL (2026-05-04)**: 1 engine primitive + 2 SOS 🟡 → ✅
  promotions + 4 hybrid pip fidelity bumps + 1 STX 2021 fidelity
  bump. Tests at 1376 (was 1368, +8 net).
  - **Engine: `CardDefinition.enters_with_counters: Option<(CounterType,
    Value)>`** — "this permanent enters with N {kind} counters on
    it" replacement effect. Resolved at the cast-time spell-resolution
    path (`stack.rs`) *between* battlefield entry and the ETB-trigger
    push, so the counters land *before* SBAs run. The `Value` is
    evaluated against the cast-time `EffectContext` — the spell's
    `x_value`, `converged_value`, and `targets[]` are in scope, so
    X-cost permanents like Pterafractyl read the actual paid X.
    Distinct from an ETB trigger that adds counters via
    `Effect::AddCounter`: ETB triggers fire *after* bf entry, so a
    1/0 body would die to the 0-toughness SBA before its trigger
    could resolve. The replacement form wires the counters in
    atomically with bf entry, surviving the post-entry SBA pass.
    Honored only on the spell-resolution path; tokens and
    `Move → Battlefield` paths skip this hook. `#[serde(default)]`
    for back-compat with older serialized snapshots.
  - **2 SOS 🟡 → ✅ promotions**:
    - **Lluwen, Exchange Student // Pest Friend** ({2}{B}{G} //
      {B/G}) — back-face Pest Friend's `{B/G}` hybrid pip now wired
      exactly via `ManaSymbol::Hybrid(Black, Green)`. Closes the
      Witherbloom (B/G) school's last 🟡 row.
    - **Pterafractyl** ({X}{G}{U}, 1/0 Dinosaur Fractal) — printed
      1/0 body via the new `enters_with_counters` replacement.
  - **STX 2021 fidelity bumps**: Star Pupil (printed 0/0 body via
    replacement); Reckless Amplimancer (printed 0/0 body, scaling
    proxy stays 🟡).
  - **4 hybrid pip fidelity bumps** (stay-status, more faithful):
    Stirring Honormancer ({W/B}); Practiced Scrollsmith ({R/W});
    Paradox Surveyor ({G/U}); Essenceknit Scholar ({B/G}).
  - **8 new tests**: Lluwen castable with {G}-only pool; Lluwen
    rejects empty pool; Practiced Scrollsmith castable with extra W;
    Essenceknit Scholar castable with extra G; Pterafractyl X=0
    dies; Pterafractyl printed body 1/0; Star Pupil printed body
    0/0; `enters_with_counters` snapshot serde.
  - **CR 122.6 / 122.6a audit** — "Some spells and abilities refer
    to counters being put on an object. This refers to putting
    counters on that object while it's on the battlefield and also
    to an object that's given counters as it enters the battlefield.
    / If an object enters the battlefield with counters on it, the
    effect causing the object to be given counters may specify which
    player puts those counters on it. If the effect doesn't specify
    a player, the object's controller puts those counters on it."
    The new `enters_with_counters` field implements the bf-entry
    half of CR 122.6 and the controller-puts-counters default of
    CR 122.6a (the field doesn't expose a player parameter today,
    so the cast's controller — `caster` in `stack.rs` — places
    them, matching the unspecified-player default). Validated by
    Pterafractyl tests (X=0 → 0 counters on entry, dies; X=2 →
    2 counters before SBA pass) and Star Pupil tests (printed 0/0
    body + replacement-counters land at entry).

- ✅ **Push XXXIX (2026-05-03)**: 3 engine primitives + 5 card
  promotions + 5 new STX 2021 cards + 1 server view enrichment +
  1 bot affordability fold-in. Tests at 1368 (was 1363, +5 net).
  - **Engine: `Value::IfPredicate { cond, then, else_ }`** — branching
    value evaluated at effect time. Lets a `Value` switch on a
    `Predicate` against the same `EffectContext`. Used by Wilt in the
    Heat's "{2} less if cards left graveyard" cost reduction; future
    "X = N if condition else 0" payoffs (conditional pump magnitudes,
    Spectacle riders) reuse the same shape.
  - **Engine: `CardDefinition.additional_sac_cost: Option<Selection
    Requirement>`** — "as an additional cost: sacrifice a [filter]"
    primitive on the cast pipeline. `cast_spell` does a pre-flight
    check (controller must control ≥1 matching permanent other than
    the spell itself), then auto-picks the lowest-value matching
    creature (tokens first, then by mana value, then by power) and
    sacrifices it after mana payment but before the spell goes on
    the stack. Daemogoth Woe-Eater / Eyeblight Cullers graduated
    from "ETB sacrifice approximation" to printed-faithful cast-time
    sacrifice.
  - **Engine: Aura cast-time pre-attach (`stack.rs`)** — when a
    permanent spell is an Aura with a Permanent target (CR 303.4f),
    the engine pre-binds the target onto `card.attached_to` at the
    moment the Aura enters the battlefield. Without this, the
    orphaned-aura SBA (CR 704.5m) would immediately graveyard the
    Aura between bf entry and the cast-target snapshot. Solid
    Footing is the first catalog Aura that uses the static-attach
    layer pattern via `Selector::AttachedTo(This)`.
  - **Engine: `target_filter_for_slot_in_mode` walks Value args** —
    new `val_find` recursion that pulls a slot 0 filter out of a
    `DealDamage.amount`'s `Value::PowerOf(target_filtered(...))`.
    Closes Decisive Denial mode 1's fidelity gap (slot 0 friendly-
    creature filter is now enforced at cast time) and unblocks Pest
    Wallop's "your creature deals damage" printed slot-0 filter.
  - **5 STX 🟡 → ✅ promotions**: Wilt in the Heat (cost reduction
    wired); Daemogoth Woe-Eater + Eyeblight Cullers (cast-time
    sacrifice); Big Play (up-to-two-creatures fan-out); Decisive
    Denial (slot 0 filter enforced).
  - **5 new STX 2021 cards**: Pest Wallop ({3}{G} Sorcery, friendly-
    creature damage), Solid Footing ({W} Aura, +1/+2 + vigilance),
    Swarm Shambler ({G} 1/1 Beast, ETB counter + activation grow),
    Containment Breach ({1}{W} Instant, destroy enchantment + Learn),
    Unwilling Ingredient ({B} 1/1 Insect Pest, MayPay death cantrip).
  - **UI: `KnownCard.additional_cost_label: Option<String>`** —
    populated from `additional_sac_cost` via a tiny filter-shape
    renderer. Lets the client warn before wasting mana on a spell
    the controller can't currently afford (Daemogoth Woe-Eater /
    Eyeblight Cullers without a creature to sacrifice). Defaulted
    to `None` for back-compat with older serialized views.
  - **Bot: additional-sac-cost affordability fold-in** —
    `can_afford_in_state` now rejects a hand card whose
    `additional_sac_cost` filter has no matching permanent on the
    battlefield. Skips dry-run noise on Woe-Eater / Cullers casts
    that the engine would reject anyway.
  - **CR 704.5m audit**: rule is implemented (orphaned-aura SBA in
    `stack.rs`) and now validated by two tests
    (`solid_footing_pumps_enchanted_creature_with_vigilance` for
    legal-aura survival; `solid_footing_graveyards_when_enchanted_
    creature_dies` for the SBA fire). Code comment in `stack.rs`
    updated from `CR 704.5n` (incorrect) to `CR 704.5m` (correct
    citation).
  - **Tests**: +5 net (1363 → 1368). New tests cover Wilt's
    discount fires + no-discount paths, Woe-Eater + Cullers cast
    rejection, Big Play 2-creature fan-out, Decisive Denial mode 1
    rejection, Pest Wallop pump+damage + reject, Solid Footing pump
    + die-graveyards, Swarm Shambler ETB + activation, Containment
    Breach + Unwilling Ingredient cantrips, KnownCard view label.

- ✅ **Push XXXVIII (2026-05-03)**: 4 engine primitives + 10 card
  promotions across STX 2021 + SOS. Tests at 1363 (was 1336; +27 net).
  - **Engine: `StaticEffect::CostReductionTargeting`** — Killian's
    target-aware discount. `cost_reduction_for_spell` walks both
    battlefield static abilities (controller-scoped to caster) and the
    cast card's own static abilities, summing matching discounts.
    `ManaCost::reduce_generic` drains generic pips left-to-right, capped
    at 0. All three cast paths (regular, alt cost, flashback) consult
    the reduction.
  - **Engine: `StaticEffect::CostReductionScaled`** — Affinity-style
    Value-typed discount (Witherbloom's Affinity for creatures; Dawning
    Archaic's per-IS-card-in-gy discount).
  - **Engine: `AffectedPermanents::All { excluded_supertypes,
    exclude_source }`** — Hofri Ghostforge's "Other *nonlegendary*
    creatures" anthem now decomposes faithfully via the new
    `excluded_supertypes` field at static-layer translation time.
  - **Engine: `AlternativeCost.mode_on_alt: Option<usize>`** —
    Devastating Mastery's Mastery alt cost auto-selects mode 1
    (Wrath + reanimate) at cast time.
  - **Engine: Prowess wired** as a synthetic SpellCast trigger that
    sweeps every battlefield Keyword::Prowess permanent on each
    noncreature spell cast.
  - **5 STX 2021 promotions**: Killian Ink Duelist, Spectacle Mage,
    Tempted by the Oriq, Devastating Mastery (all 🟡 → ✅); Hofri
    Ghostforge (anthem fix only).
  - **5 SOS promotions**: Ajani's Response, Inkshape Demonstrator,
    Ennis Debate Moderator (all 🟡 → ✅); Witherbloom Balancer (first
    Affinity clause wired); Dawning Archaic (⏳ → 🟡, body added).
  - **UI: `PermanentView.static_abilities`** — `Vec<String>` field
    populated from each `StaticAbility.description`. Lets clients
    render printed rules-text without rebuilding from the static-effect
    tree.
  - **Bot: discount-aware affordability prefilter** —
    `extra_cost_for_card_in_hand` now subtracts target-independent cost
    reductions before returning net excess.
  - **27 new tests**: 5 Killian + 4 Spectacle Mage + 3 Hofri +
    2 Tempted + 2 Devastating Mastery + 2 Ajani + 3 Witherbloom +
    3 Dawning Archaic + 2 view + 3 snapshot serde + 1 doc fix.

- ✅ **Push XXXVII (2026-05-03)**: 2 engine primitives + 4 STX 2021
  promotions + 2 SOS doc fixes. Tests at 1336 (was 1325; +11 net).
  - **Engine: `Effect::PickModeAtResolution(Vec<Effect>)`** —
    sibling to `Effect::ChooseMode` that prompts at *resolution*
    time rather than cast time. Used by triggered abilities whose
    printed text reads "your choice of X or Y" (the surrounding
    spell's `ctx.mode` is already pinned to the cast-time pick).
    AutoDecider picks mode 0; ScriptedDecider can flip via `Mode(N)`.
    Decision surfaces via the existing `Decision::ChooseMode` (no
    new wire-format type). Snapshot serde round-trip tested.
  - **Engine: `StaticEffect::TaxActivatedAbilities { filter, amount }`**
    — Augmenter Pugilist-style "activated abilities of [filter] cost
    {N} more to activate." `extra_cost_for_activation(state, source)`
    helper in `game/actions.rs` walks the battlefield's
    `TaxActivatedAbilities` statics and sums the surcharge when the
    activating permanent matches `filter`. Folded into
    `activate_ability` as additional generic mana before the pre-
    flight payment snapshot — failures roll back tap + mana cleanly.
    Mana abilities are NOT exempt per the rules. Snapshot serde
    round-trip tested.
  - **4 STX 2021 promotions to ✅** (`catalog::sets::stx::*`):
    - **Shadrix Silverquill** (legends.rs) — choose-2-of-3 attack
      trigger now wires via `Effect::ChooseModes { count: 2 }` re-
      used at trigger resolution. AutoDecider picks modes 0+1
      (draw + drain).
    - **Prismari Apprentice** (prismari.rs) — Magecraft "Scry 1 or
      +1/+0 EOT" now wires via the new `PickModeAtResolution`.
      AutoDecider picks Scry 1; ScriptedDecider can flip the +1/+0
      combat-trick mode.
    - **Augmenter Pugilist** (quandrix.rs) — static "activated
      abilities of creatures cost {2} more" now wires via the new
      `TaxActivatedAbilities`.
    - **Silverquill Apprentice** (decks/modern.rs) — Magecraft
      "+1/+1 or -1/-1" now wires via `PickModeAtResolution`.
      AutoDecider picks +1/+1 (combat-trick safe default).
  - **2 SOS doc fixes** (cards already wired with their gates;
    status notes were stale):
    - **Potioner's Trove** (sos/artifacts.rs) — `{T}: gain 2 life,
      activate only if you've cast an IS spell this turn` was
      already wired since push XIII via
      `Predicate::InstantsOrSorceriesCastThisTurnAtLeast`.
    - **Ennis, Debate Moderator** (sos/creatures.rs) — end-step
      counter trigger has been using exact-printed
      `Predicate::CardsExiledThisTurnAtLeast` since push IX (not
      the gy-leave proxy the doc claimed).
  - **11 new tests**: 7 STX 2021 promotion tests (Prismari Apprentice
    × 2, Shadrix × 2, Augmenter Pugilist × 3) + 1 mode-1 shrink
    Silverquill Apprentice test + 2 snapshot serde round-trip tests
    + 1 view-label test for the new `PickModeAtResolution` arm.
  - **`prefers_friendly_target` extended to `PickModeAtResolution`**
    so any inner mode preferring friendly bubbles up — covers the
    "+1/+1 or -1/-1" Silverquill Apprentice shape where the pump
    mode's friendly preference drives auto-target while the shrink
    mode is opt-in via ScriptedDecider.

- ✅ **Push XXXVI (2026-05-03)**: `Effect::ChooseModes` primitive +
  `Effect::DelayUntil.capture` field + 10 card promotions across SOS
  and STX 2021. Tests at 1325 (was 1315; +10 net).
  - **Engine: `Effect::ChooseModes { modes, count, up_to,
    allow_duplicates }`** — resolution-time "choose K modes from N"
    primitive, sibling to `Effect::ChooseMode`. Backed by the new
    `Decision::ChooseModes` and `DecisionAnswer::Modes(Vec<usize>)`.
    AutoDecider returns first `count` modes; ScriptedDecider picks any
    combination. Backwards-compat: cast with `mode: Some(N)` runs
    *only* mode N, preserving existing test casts that target a
    specific mode of a Command.
  - **Engine: `Effect::DelayUntil.capture: Option<Selector>`** —
    extends the delayed-trigger primitive with an optional selector
    evaluated at delay-registration time. The first entity it resolves
    to is bound to the delayed body's `Selector::Target(0)`,
    overriding the legacy `ctx.targets[0]` capture path. Used by
    Conciliator's Duelist's Repartee — trigger has no target slot of
    its own, so `Selector::CastSpellTarget(0)` captures the cast
    spell's target into the delayed return.
  - **5 STX Command promotions to ✅** (`catalog::sets::stx::*`):
    Lorehold Command, Witherbloom Command, Prismari Command,
    Silverquill Command, Quandrix Command — all use the new
    `Effect::ChooseModes { count: 2 }` shape. Auto-decider picks
    modes 0+1 (the most-typical play pattern for each Command);
    ScriptedDecider drives other mode pairs for tests.
  - **5 individual promotions**:
    - **Conciliator's Duelist** (SOS Silverquill) — Repartee's exile +
      return-at-next-end-step rider now both wired via the new
      `Effect::DelayUntil { capture: ... }`.
    - **Borrowed Knowledge** (SOS Lorehold) — doc-only promotion.
      Both modes were already wired; status was a stale annotation.
    - **Mentor's Guidance** (STX Quandrix) — doc-only promotion.
      Printed Oracle is single-target so the existing wire matches
      printed exactly.
    - **Multiple Choice** (STX mono-blue) — "Choose one or more"
      now wires faithfully via `Effect::ChooseModes { count: 3,
      up_to: true }`. Auto-decider picks all 3 modes. The "if you
      chose all of the above" mega-mode rider stays gap.
    - **Lorehold, the Historian** (SOS Lorehold) — fidelity bump
      (still 🟡 since miracle grant remains gap). Per-opp-upkeep
      loot trigger now wired via `EventScope::OpponentControl +
      StepBegins(Upkeep)`.
  - **10 new tests** covering ChooseModes auto-decider behavior,
    ScriptedDecider mode-pair overrides, MOR regression check, and
    Conciliator's Duelist + Lorehold the Historian.

- ✅ **Push XXXV (2026-05-03)**: 6 SOS 🟡 → ✅ promotions + 3 fidelity
  bumps using the existing `Selector::one_of` and `Effect::ChooseMode`
  primitives — no new engine work. Tests at 1315 (was 1306; +9 net).
  - **6 SOS promotions to ✅**:
    - **Dina's Guidance** ({1}{B}{G}, Sorcery) — hand-or-graveyard
      destination prompt now wired as `Effect::ChooseMode` with two
      modes (Search → Hand vs Search → Graveyard). Both modes use
      the existing `Effect::Search` primitive. Reanimator decks
      (Goryo's Vengeance / Animate Dead / Reanimate downstream) can
      flip to mode 1 via the cast-time `mode` argument.
    - **Vibrant Outburst** ({U}{R}, Instant) — printed "tap up to one
      target creature" half now wired via `Selector::one_of(
      EachPermanent(opp creature))`. Tap auto-picks an opp creature,
      no-ops cleanly when no opp creature exists. 3-damage primary
      slot still user-targeted (any target).
    - **Dissection Practice** ({B}, Instant) — printed "Up to one
      target creature gets +1/+1 EOT" half now wired via `Selector::
      one_of(EachPermanent(Creature ∧ ControlledByYou))`. All three
      optional halves now fire (drain 1 + +1/+1 friendly + -1/-1 user-
      targeted).
    - **Practiced Offense** ({2}{W}, Sorcery) — printed "your choice
      of double strike or lifelink" mode pick now a top-level `Effect::
      ChooseMode`: mode 0 = +1/+1 fan-out + double strike grant;
      mode 1 = +1/+1 fan-out + lifelink grant. Cast-time `mode: Some(
      0)` / `Some(1)` flips between the two; default is DS.
    - **Cost of Brilliance** ({2}{B}, Sorcery) — +1/+1 half is now
      optional via `Selector::one_of(EachPermanent(Creature ∧
      ControlledByYou))`. Cast is now legal even when you control no
      creatures.
    - **Render Speechless** ({2}{W}{B}, Sorcery) — same treatment as
      Cost of Brilliance: "up to one creature target" half now
      auto-picks a friendly creature, no-ops cleanly when none exist.
  - **3 fidelity bumps (still 🟡)**:
    - **Stress Dream** ({3}{U}{R}, Instant) — 5-damage half now uses
      `Selector::one_of(EachPermanent(opp creature))`. Cast is now
      legal even when no opp creature exists. The look-at-top-2 half
      stays approximated as scry 1 + draw 1.
    - **Burrog Barrage** ({1}{G}, Instant) — damage half now hits an
      opp creature via `Selector::one_of(EachPermanent(opp creature))`
      (was: self-damage to slot 0). One-sided power-as-damage.
    - **Homesickness** ({4}{U}{U}, Instant) — second creature slot
      now wired via `Selector::one_of(EachPermanent(opp creature))`.
      With 2 distinct opp creatures both get tapped + stunned. With
      only 1 opp creature, the auto-pick collides with slot 0
      (multi-target uniqueness gap).
  - **9 new tests**: one per promotion + the no-creature-on-bf path
    for the 5 promotions that gained "castable with no targets"
    semantics (Dina's Guidance mode 1, Vibrant Outburst no-creature,
    Cost of Brilliance no-creature, Stress Dream no-creature,
    Practiced Offense lifelink mode), plus Vibrant Outburst's tap-half
    + Dissection Practice's +1/+1 friendly-creature half + Burrog
    Barrage's friendly-power-vs-opp test + Homesickness's collision
    behavior with one opp creature.
  - **Engine note**: nested `ChooseMode` works correctly when `ctx.mode`
    is set at cast time (Practiced Offense, Dina's Guidance verify).
    Future cards that need *resolution-time* mode picks (e.g. Prismari
    Apprentice's "Scry 1 or +1/+0 EOT" magecraft) still need a separate
    `Effect::PickModeAtResolution` primitive — tracked below.

- ✅ **Push XXXIV (2026-05-03)**: 2 STX 🟡 → ✅ promotions + 9 new cube
  cards + 2 engine primitives + 1 UI label. Tests at 1306 (was 1292;
  +14 net).
  - **Engine: `ActivatedAbility::exile_gy_cost: u32`** — pre-flight
    gate rejects with the new `GameError::InsufficientGraveyard`
    when the controller has fewer cards than the cost requires. After
    tap/mana/life payment, the engine picks the oldest gy cards and
    moves them to exile via `move_card_to`. Each pick fires
    `CardLeftGraveyard` so SOS gy-leave payoffs trigger off the cost.
    `#[serde(default)]` on the new field keeps snapshot wire format
    back-compat. Used by **Lorehold Pledgemage** (🟡 → ✅).
  - **Engine: `Effect::GainControl` is now Duration-aware** —
    refactored from a permanent-flip stub to a Layer-2 continuous
    effect (`Modification::ChangeController`) that honors the
    `Duration` field. EOT/EndOfCombat → `UntilEndOfTurn`,
    UntilNextTurn/UntilYourNextUntap → `UntilNextTurn`, Permanent →
    `Indefinite`. The cleanup step's `expire_end_of_turn_effects`
    drops the EOT bindings, restoring the original controller. Used
    by **Mascot Interception** (🟡 → ✅).
  - **Engine: post-move filter introspection** —
    `evaluate_requirement_static` now walks **hands and libraries**
    as a fallback for card-id lookups (in addition to the existing
    battlefield → graveyards → exile → stack chain). Powers trigger
    filters that fire after a card has already been moved out of
    the graveyard — Murktide Regent's "instant or sorcery card left
    your gy" trigger evaluates the filter after Zealous Lorecaster
    has returned the bolt to hand.
  - **9 new cube / MH2 cards** (`catalog::sets::decks::modern`):
    Subtlety, Monastery Swiftspear, Wild Nacatl, Seasoned Pyromancer,
    Murktide Regent, Faerie Mastermind, Fury, Young Pyromancer, Grief,
    Sage of the Falls.
  - **UI improvement**: `ability_cost_label` (server view) now renders
    the new `exile_gy_cost` field as printed-text "Exile a card from
    your graveyard" / "Exile N cards from your graveyard", mirroring
    the existing `sac_cost` / `life_cost` rendering.
  - **14 new tests**: 5 STX promotion tests (Pledgemage activate +
    reject / Mascot steal + revert), 9 cube-card tests (one per new
    card + body sanity for vanilla bodies), 1 view test
    (`ability_cost_label_renders_exile_gy_cost`).

## Future work — engine/UI suggestions surfaced by push XL

- **`enters_with_counters` for tokens** — the current implementation
  honors the field only on the cast-time spell-resolution path
  (`stack.rs`). Tokens (`Effect::CreateToken`) and reanimate-style
  `Move → Battlefield` paths skip the hook. Future "this token
  enters with N counters" minters (Incubator tokens, Body of
  Research's Fractal token mints with X counters, Snow Day's
  hand-size-counters Fractal) would need a parallel hook on the
  token-creation path that reads the value at token-mint time. Mostly
  blocked by the question of which `EffectContext` to evaluate the
  Value against (the spell's ctx vs. a fresh ctx at mint time);
  `Value::HandSizeOf` etc. read fine in either, but X-cost values
  need the spell's x_value.

- **`enters_with_counters` for the cast-from-graveyard path** —
  Flashback / Aftermath casts re-resolve through `cast_flashback`,
  which uses `continue_spell_resolution` — that path may or may
  not hit the `enters_with_counters` hook in `stack.rs`. Audit
  needed: does flashback-cast Pterafractyl correctly enter with
  X counters? (Pterafractyl has no Flashback, but a future X-cost
  Flashback creature would need to be tested.)

- **Multi-counter `enters_with_counters` (poison + +1/+1)** — current
  field is `Option<(CounterType, Value)>` — exactly one counter
  kind. Future cards may enter with multiple kinds (a planeswalker
  enters with 4 loyalty + a stun counter from a static; battles
  enter with defense counters from a separate static). Would need
  a `Vec<(CounterType, Value)>` shape to express. Not blocking any
  current card.

- **CR 122.6a `player` parameter** — CR 122.6a says "the effect
  causing the object to be given counters may specify which player
  puts those counters on it. If the effect doesn't specify a
  player, the object's controller puts those counters on it." The
  current implementation defaults to the caster (controller) and
  doesn't expose a player parameter; future "your opponent puts X
  counters on it" effects would need a `placer: Option<PlayerRef>`
  field. Today no card needs it.

- **`enters_with_counters` × Pterafractyl etc. with X=0 audit** —
  Pterafractyl printed 1/0 + 0 counters → dies to SBA. Verified by
  the new test. Some cards (Hangarback Walker {X}) have a "this
  enters with X +1/+1 counters and is a 0/0" body that's fine at
  X=0 because the body is also dependent on counters. Add a future
  test for Hangarback-style "enters as a 0/0 with X +1/+1 counters"
  bodies once such a card lands.

- **Hybrid pip migration audit** — push XL wired faithful hybrid
  pips on Lluwen, Stirring Honormancer, Practiced Scrollsmith,
  Paradox Surveyor, Essenceknit Scholar. Other cards with
  "approximated as {single-color}" hybrid notes (audit script
  output) deserve the same treatment when they come up in test
  scenarios. The engine's `pay()` already handles hybrid pips
  correctly since push XXXVIII (Spectacle Mage); this is purely a
  factory-side migration.

## Future work — engine/UI suggestions surfaced by push XXXIX

- **Damage-replacement primitive ("if it would die, exile instead")** —
  Wilt in the Heat's "If that creature would die this turn, exile it
  instead" rider stays gap. Same shape as Anger of the Gods, Path of
  Peril's exile-instead clauses. Engine would need a per-permanent
  `damage_to_exile_until_eot` flag (or a `Effect::ApplyExileInsteadOf
  Damage`) read by the SBA-graveyard handler. Would also unblock
  Sundering Stroke's "exile instead" multi-target lines (still gap on
  divided damage too).

- **`additional_sac_cost` count generalisation** — the current shape
  is `Option<SelectionRequirement>` (sacrifice exactly one matching
  permanent). Future "as additional cost: sacrifice TWO creatures" /
  "sacrifice X creatures" shapes (Mortal Combat, Wing Storm, Bone
  Splinters family) need a count field or a separate `Vec<Sacrifice
  Requirement>` to express multi-sacrifice. Would also unblock
  **escalate** ({1}{B} per extra mode picked → sacrifice cost per
  extra mode) and similar additive costs.

- **Aura targeting prompt UI** — Solid Footing currently relies on
  the cast-time pre-attach to set `attached_to`. The client's
  targeting UI doesn't yet special-case Auras (CR 303.4f says Aura
  targets must match the Aura's own targeting filter — "enchant
  creature" → only creatures legal). Future UI work: read
  `EnchantmentSubtype::Aura` and only show creature-shaped target
  candidates at cast time (today the cast can attach to non-creature
  permanents because there's no enchant-target filter). Same UI gap
  as Equipment's Equip-target prompt.

- **`val_find` recursion on more effect arms** — the new recursion
  walks `DealDamage.amount`. Future cards may store slot 0 filters
  inside other Value-typed effect args (`AddCounter.amount`,
  `PumpPT.power`, `PumpPT.toughness`) — when those arms grow filter-
  bearing Values, extend `val_find` to them. Tracked here as a
  reminder; no current card needs it.

- **Auto-pick smarter sacrifice fodder** — `additional_sac_cost`
  picks the lowest-value matching creature (tokens first, then by
  mana value, then by power). Future improvement: avoid creatures
  with valuable triggers (a Spirit token from Sparring Regimen would
  actually be MORE valuable to sacrifice into Daemogoth Woe-Eater
  than a generic Pest token, since the Spirit isn't part of an
  on-attack engine), or prefer tokens with on-die abilities (Pest
  token → +1 life). Today the heuristic is a flat sort; a payoff-
  weighted sort would be a small cleanup.

## Future work — engine/UI suggestions surfaced by push XXXVIII

- **`StaticEffect::CostReductionScaled` for "spells you cast" (cross-
  spell affinity)** — Witherbloom Balancer's second clause ("IS spells
  you cast have affinity for creatures") needs a static that *grants*
  a CostReductionScaled to each IS spell cast by the controller.
  Distinct from the self-static used today: requires a "modify another
  spell's discount" primitive, registered at cast time per IS spell.
  The shape would mirror `CostReductionTargeting` but with the discount
  attached per-cast rather than per-permanent-in-play.

- **Cross-spell affinity-grant primitive — "Affinity for creatures"
  bestow chain.** Same family as Witherbloom Balancer's second clause.
  Could also unlock Mishra's Bauble's "{T}: Sacrifice this artifact:
  Look at top card of your library" gate (artifact affinity), Storm
  payoffs ("Storm spells you cast have storm")…

- **Cast-from-graveyard pipeline.** Outstanding gap blocking 7 SOS ⏳
  cards: Echocasting Symposium, Archaic's Agony, Flashback (the
  card!), Improvisation Capstone, Fix What's Broken, Nita Forum
  Conciliator, Applied Geometry. The Dawning Archaic's attack trigger
  also waits on this. Same shape as Velomachus Lorehold's reveal-and-
  cast, Practiced Scrollsmith's "may cast that card", Conspiracy
  Theorist's "may cast from exile."

- **Token-copy-of-permanent primitive.** Hofri Ghostforge's dies-as-
  Spirit-copy rider, Phantasmal Image, Mockingbird, Choreographed
  Sparks's creature-copy mode all gate on this. Engine would need a
  `Effect::CreateTokenCopy { source: Selector, modifications: Vec<…> }`
  that snapshots the source's CardDefinition and overlays modifications
  (e.g. "becomes a 1/1 R/W Spirit with flying").

- **Multi-target prompt + multi-mode replay.** Moment of Reckoning's
  "Choose up to four (same mode allowed)" + multi-target combo needs:
  (1) `Effect::ChooseModes { allow_duplicates: true }` (already
  available since push XXXVI) AND (2) per-mode-invocation distinct
  target slots. Currently each ChooseModes iteration shares
  `Target(0)` from cast time, so 4× mode 0 destroys the same target
  4×. Same gap as Together as One, Cost of Brilliance.

- **Self-counter-gated static (Comforting Counsel).** "As long as
  there are five or more growth counters on this enchantment, creatures
  you control get +3/+3" needs a runtime gate on the source's counter
  count. New shape: a `condition: Option<Predicate>` field on
  `ContinuousEffect` (or a new variant) checked at apply time.

- **Replacement-effect primitive (Strict Proctor, Owlin Shieldmage,
  Wilt in the Heat's "would die → exile instead").** Long-tracked gap,
  surfaced again by Strict Proctor's ETB-trigger replacement and
  Wilt's damage-replacement.

- **`AdditionalCost { sacrifice_filter, count }` on cast.** Daemogoth
  Woe-Eater and Eyeblight Cullers approximate "as additional cost,
  sacrifice a creature" with an ETB sacrifice trigger — at cast time
  the user should commit to the sacrifice (and the spell becomes
  illegal if no creature is available). Same family as Force of
  Negation's pitch cost; needs a sacrifice-flavor of `AlternativeCost`
  (or a regular cost with extra fields).

## Future work — engine/UI suggestions surfaced by push XXXVII

- **`StaticEffect::TaxActivatedAbilities` excluding mana abilities** —
  the current implementation taxes ALL activated abilities including
  mana abilities (per Augmenter Pugilist's exact printed text). A
  variant `TaxActivatedAbilities { exclude_mana: bool }` would model
  cards like Damping Engine that explicitly exempt mana abilities.
  Read at `extra_cost_for_activation` time by classifying the
  ability's effect tree (any `Effect::AddMana` direct child = mana
  ability per MTG rules 605.1a).

- **`StaticEffect::TaxSpellCost` / spell-side activation tax** —
  Trinisphere ({3} Artifact: "While ~ is untapped, each spell that
  would cost less than three to cast costs three to cast.") needs
  a different shape — it's a *minimum cost*, not an *additive*
  surcharge. A new `StaticEffect::SpellMinimumCost { amount }` would
  model it: every spell cast checks the static, and any spell whose
  natural cost is less than `amount` gets bumped up to exactly
  `amount`. Damping Sphere's `AdditionalCostAfterFirstSpell` is the
  closest existing shape (additive, not minimum), so this needs a
  fresh primitive. Tracked separately from `TaxActivatedAbilities`
  since the spell-side path lives in `extra_cost_for_spell`.

- **`Effect::PickModeAtResolution` + per-mode target prompts** — the
  current shape shares `Target(0)` across all modes (matching
  ChooseMode's behavior). For "your choice of X creature gets +1/+1,
  or Y creature gets -1/-1" cards where the modes pick *different*
  targets (e.g. Silverquill Apprentice's printed "your creature
  gets +1/+1 OR opponent's creature gets -1/-1"), a per-mode target
  filter prompt would unblock fully-printed semantics. Same engine
  work as the SOS Commands' multi-target prompt gap.

- **Bot tax-aware activation pre-filter** — the bot's `pick_action`
  currently uses `state.would_accept` to dry-run candidate
  activations, which correctly rejects when an opponent's Pugilist
  raises the cost beyond what the bot has floated. A pre-filter
  that *includes* the activation tax in `can_afford_with_extra`
  would skip dry-run noise on Pugilist boards. Low priority — the
  dry-run is the source of truth and correctly catches all cases.

- **`Effect::PickModeAtResolution` UI display** — the auto-decider
  picks mode 0 silently, which masks the "your choice" text from
  the user. A future "modal trigger panel" UI surface (sibling to
  the existing OptionalTrigger panel) would let humans see "Choose
  one — Scry 1 / +1/+0 EOT" and tap to flip. Today only
  ScriptedDecider can drive the alt mode.

## Future work — engine/UI suggestions surfaced by push XXXVI

- **`Effect::ChooseModes` smart auto-decider** — current AutoDecider
  picks the first `count` modes deterministically. Works fine for
  Lorehold/Witherbloom/Prismari Commands (target-less modes 0+1) but
  produces target-collision plays for Silverquill/Quandrix Commands
  whose modes 0+1 want different target filters (counter ability +
  -3/-3 creature). A smarter decider could:
  - Skip modes whose target filter is incompatible with the chosen
    Target(0).
  - Prefer modes that produce immediate value (drain, draw) over
    modes that need a board state (counter ability needs a stack
    item).
  - Fall back to first-K only when no smarter pick is available.
  Tracked separately because it's an AutoDecider heuristic, not a
  primitive change.

- **`Effect::ChooseModes` distinct Target slots per mode** — the
  current shape has all picked modes share `Target(0)` from cast time.
  For Commands with target-collision-prone mode pairs (Silverquill
  modes 0+1 = ability vs creature), this is a fidelity gap. A
  follow-up `Decision::ChooseTargetsForModes` would prompt for K
  distinct Target(slot) values, one per picked mode, at cast time.
  Same engine work as the long-tracked "multi-target prompt for
  sorceries/instants" gap (Together as One, Cost of Brilliance).

- **Modes-picked introspection** — new `Predicate::AllModesPicked`
  / `Value::CountOfModesPicked` would unlock the "if you chose all
  of the above" rider on Multiple Choice (the printed mega-mode
  fires only when all 3 modes are picked). Same shape needed for
  Strixhaven's "if you control creatures, do X" or "for each mode
  chosen" patterns.

- **`Effect::DelayUntil` capture for selectors that target multiple
  entities** — push XXXVI's `capture: Option<Selector>` field
  picks the first entity from the resolved selector. For multi-
  capture (e.g. "exile up to 3 cards, return them all at next end
  step") we'd need `capture: Vec<Selector>` or a selector that
  returns multiple captured slots. Tracked for future "may
  exile any number" cards.

- **`EventScope::OpponentControl` step-trigger fidelity** —
  push XXXVI wired Lorehold the Historian's "at the beginning of
  each opponent's upkeep" via this scope. The same shape could
  unlock several other cards:
  - **Smothering Tithe** ({3}{W}) — "Whenever an opponent draws a
    card …" needs an opp-control event filter, not the step
    variant.
  - **Approach of the Second Sun** ({6}{W}) — fires on "your
    upkeep", uses ActivePlayer scope already.
  - **Ichorid** — graveyard-resident upkeep trigger uses
    `FromYourGraveyard`, already wired.

## Future work — engine/UI suggestions surfaced by push XXXV

- **`Effect::PickModeAtResolution(Vec<Effect>)`** — sibling to
  `Effect::ChooseMode` that prompts at *resolution* time rather than
  cast time. Needed by:
  - **Prismari Apprentice** ({U}{R}, 1/2): Magecraft "Scry 1 or +1/+0
    EOT" — currently collapsed to Scry 1 only.
  - Other "your choice of X or Y" embedded mode picks in Effect::Seq
    that don't fit the spell-level cast-time `ctx.mode` shape (today
    a nested ChooseMode would re-read the spell's `ctx.mode` and pick
    the *same* mode index for both, which is incorrect).
  - The new primitive would push a `Decision::Mode { source, modes,
    description }` at resolution time, suspend on `wants_ui`, and
    answer with `DecisionAnswer::Mode(idx)` — reusing the existing
    decision plumbing. Auto-decider picks mode 0.

- **Multi-target uniqueness in auto-target** — push XXXV's
  Homesickness, Burrog Barrage, Stress Dream all use `Selector::
  one_of(EachPermanent(...))` for the second creature slot. The
  auto-target picker re-evaluates the selector each Effect resolution,
  so when only one eligible creature exists, the second pick collides
  with slot 0 (e.g. 2 stun counters on a single creature instead of
  2 separate ones). A `Selector::one_of_excluding(filter, slot_to_
  exclude)` variant — or a per-resolution "already picked" set fed
  into `resolve_selector` — would close the gap. Cards: Homesickness
  (tap up to two), Pull from the Grave (return up to two), Together
  as One (target player + any-target dual prompt), Cost of Brilliance
  (target player + creature dual).

- **Multi-target prompt for sorceries/instants** — ongoing engine
  gap (tracked across many pushes). Cards collapsed because the
  caster can't pick two distinct entities at cast time:
  - **Together as One**: target player draws X (collapsed to "you"),
    any-target damage (single slot 0) — needs 2 separate target slots.
  - **Cost of Brilliance**: target player draws 2 + loses 2 (collapsed
    to "you"), +1/+1 counter on creature (slot 0).
  - **Practiced Offense**: target player gets +1/+1 fan-out
    (collapsed to "you"), creature target gets DS/Lifelink (slot 0).
  Closing this needs `Decision::ChooseTargets { slot_count, filters }`
  + a multi-slot `Target` value, plus `Selector::Target(n)` already
  supports n > 0 — just the cast-time prompt is missing.

- **`Effect::Search` to graveyard prompt UX** — Dina's Guidance
  (push XXXV) shows the working pattern: `Effect::ChooseMode` between
  two `Effect::Search { to: ZoneDest::Hand }` and `Effect::Search { to:
  ZoneDest::Graveyard }` modes. Could be promoted to a helper
  `effect::shortcut::tutor_to_hand_or_graveyard(filter)` once a 2nd
  card uses the same shape.

## Future work — engine/UI suggestions surfaced by push XXXIV

- **`Effect::GrantKeyword` should honor its `duration` field**
  (currently mutates `card.definition.keywords` directly,
  irrespective of the `duration: Duration::EndOfTurn` argument).
  The fix is to migrate the resolution arm to push a Layer-6
  `ContinuousEffect { modification: AddKeyword(_), duration:
  EffectDuration::UntilEndOfTurn, … }` (mirroring how
  `Effect::GainControl` was just refactored). This would close the
  haste-grant-on-Mascot-Interception gap (Layer-2 control change
  expires correctly at Cleanup, but the haste grant doesn't —
  documented in the
  `mascot_interception_control_reverts_at_end_of_turn` test).
  Touches every `Keyword::DoubleStrike`/`Haste`/`CantBlock`/etc.
  granted-EOT path, so a careful diff is required: the existing
  ETB-pump-and-grant family (Augusta, Storm-Kiln Artist, Eager
  First-Year, etc.) shouldn't change behaviour mid-turn — only the
  cleanup-step revert behaviour gets fixed.

- **`ActivatedAbility::exile_gy_cost` interactive picker** — the
  current implementation auto-picks the oldest cards in the
  controller's graveyard (gy index `0..N`). Real MTG lets the
  controller pick any N cards. A future `Decision::PickFromZone`
  variant + UI prompt would close the gap; for now the auto-pick
  matches the auto-decider's "least-valuable card first" heuristic.

- **`Effect::GainControl` interactive Duration prompt** — the new
  Duration-aware GainControl honors EOT, but the Threaten / Bribery
  / Mind Control family also needs:
  - **Bribery's "from opp's library" target prompt** — needs a
    new `Selector::OpponentLibraryCreature` + `move_card_to`
    walking the *opp's* library, then placing under the caster's
    control.
  - **"For as long as you control X" duration** — `Duration::
    WhileSourceOnBattlefield` would suffice via the existing
    `EffectDuration::WhileSourceOnBattlefield` mapping; just needs
    the card factory to wire it.

- **Hands & libraries as filter-eval lookup fallback** (push XXXIV
  shipped this) — opens up trigger filters like "exile every
  artifact card that left your graveyard this turn" where the
  card may end up in any zone. Future improvement: a single
  `find_card_anywhere(cid)` helper that consolidates the now-five-
  zone fallback chain (battlefield, graveyards, exile, hands,
  libraries, stack). Hot path for `evaluate_requirement_static`,
  so the fallback chain order matters for performance — battlefield
  first (the most common case), exile / graveyards second,
  hands / libraries last.

- ✅ **Push XXXIII (2026-05-03)**: 8 promotions (3 STX 2021 + 5 SOS) +
  `effect::shortcut::any_target()` helper + UI "any target" label arm
  + sign-aware Pump/Shrink label split. Tests at 1292 (was 1279; +13
  net).
  - **3 STX 2021 promotions to ✅**:
    - **Lorehold Apprentice** ({R}{W}, 1/1) — Magecraft now uses the
      new `any_target()` helper for the printed "1 damage to any
      target" half. Lifegain half unchanged.
    - **Storm-Kiln Artist** ({2}{R}{W}, 3/3) — same `any_target()`
      upgrade for the "1 damage to any target" magecraft. Treasure
      follow-up unchanged.
    - **Decisive Denial** mode 1 wired ({G}{U} Instant) — promoted
      from "mode 0 only" to both modes. Mode 1 is one-sided "deal
      damage equal to your creature's power" via `DealDamage { to:
      one_of(EachPermanent(opp creature)), amount: PowerOf(target_
      filtered(...)) }`. Status stays 🟡 because the slot 0 friendly-
      creature filter lives inside the `amount` Value, not in `to`,
      so cast-time legality doesn't reject opp picks — small fidelity
      gap pending multi-slot target filter introspection.
  - **5 SOS promotions to ✅**:
    - **Thunderdrum Soloist** ({1}{R}, 1/3) — Opus rider via `opus(5,
      ...)`. Always: 1 damage to each opp. Big-cast (≥5 mana): an
      additional 2 damage (net 3 to each opp).
    - **Expressive Firedancer** ({1}{R}, 2/2) — Opus +1/+1 EOT always
      + DoubleStrike grant on big cast.
    - **Molten-Core Maestro** ({1}{R}, 2/2) — Opus +1/+1 counter
      always + `AddMana(OfColor(Red, PowerOf(This)))` on big cast.
      Counter resolves first → 5-mana cast adds {R}{R}{R} on a 2/2.
    - **Ambitious Augmenter** ({G}, 1/1) — Increment trigger via
      `increment()`. Stays 🟡 because the dies-as-Fractal-with-
      counters rider needs a counter-transfer-on-death primitive.
    - **Topiary Lecturer** ({2}{G}, 1/2) — Increment trigger via
      `increment()` — counters scale the existing `{T}: Add {G}×power`
      mana ability linearly.
  - **New `effect::shortcut::any_target()` helper** — `target_filtered
    (Creature ∨ Planeswalker ∨ Player)`, the canonical "any target"
    filter for `Effect::DealDamage` magecraft / Repartee triggers and
    burn spells. Auto-target picker prefers the opp face for hostile
    damage but falls through to creatures / planeswalkers when face
    damage isn't legal (hexproof, shroud).
  - **UI improvement**: `entity_matches_label` recognises the 3-way
    Or shape (`Creature ∨ Planeswalker ∨ Player`, both nesting
    orders) and renders it as the canonical "any target".
  - **Engine improvement**: `ability_effect_label` (server/view.rs)
    now splits `Effect::PumpPT` into "Pump" (positive or dynamic P/T)
    and "Shrink" (both halves non-positive with at least one negative
    Const). Powers ~12 catalog cards that use negative PumpPT —
    Burrog Befuddler magecraft -2/-0, Witherbloom Command mode 3
    -3/-3, Dina, Soul Steeper -X/-X. Dynamic values (XFromCost,
    CountOf) default to "Pump".
  - **13 new tests**: 9 in `tests::sos::*` (one per Opus / Increment
    promotion + cheap/big cast variants for Thunderdrum / Firedancer
    / Maestro), 3 in `tests::stx::*` (Lorehold Apprentice + Decisive
    Denial mode 1 ×2), 1 in `server::view::tests::*` (Pump/Shrink
    label split). Plus an extension to the Or-composite label test
    for "any target".

## Future work — engine/UI suggestions surfaced by push XXXIII

- **Multi-slot target filter introspection** — Decisive Denial mode 1
  ships with the slot 0 friendly-creature filter buried inside
  `Value::PowerOf(target_filtered(...))` rather than the `to`
  selector. The cast-time legality check (`cast_spell_with_convoke`)
  only reads `target_filter_for_slot_in_mode(0, mode)`, which walks
  the effect tree but doesn't descend into `Value` arguments. A
  `Value::has_target_filter()` walker (mirror of `value_has_target`
  but returning `Option<&SelectionRequirement>`) would close the gap
  for Decisive Denial and any future "deal damage equal to its
  power"-style spell where the user-picked target is on the source
  side.

- **Counter-transfer-on-death primitive** — Ambitious Augmenter's
  printed "When this dies, if it had counters on it, create a 0/0
  Fractal token, then put this creature's counters on that token" is
  the only blocker keeping it from full ✅. The "if it had counters
  on it" gate is already expressible via `Predicate::ValueAtLeast(
  CountersOn(SelfSource), 1)` (push XVII counter-on-graveyard
  fallback). The "put this creature's counters on that token" half
  needs a new `Effect::TransferCounters { from, to }` primitive that
  reads the counter snapshot from the dying card's graveyard-resident
  copy and applies it to the freshly-minted token (`Selector::
  LastCreatedToken`). Roughly the same pattern as Scolding
  Administrator's "if it had counters on it, put those counters on
  up to one target creature" — that already uses
  `Value::CountersOn(SelfSource)` to read the count, but `AddCounter`
  ships fixed-kind counters today; transfer needs both kind + count
  to flow through.

- ✅ **Push XXXII (2026-05-03)**: 13 new STX 2021 cards + lethal-first
  auto-target + UI label coverage. Tests at 1279 (was 1261; +18 net).
  - **13 new STX 2021 cards** (`catalog::sets::stx::mono`): Vortex
    Runner ({1}{U}, 1/2 Unblockable + Attack/Scry 1), Burrog Befuddler
    ({1}{U}, 1/3 Frog Wizard Flash + magecraft -2/-0), Crackle with
    Power ({X}{R}{R}{R}, 5X damage via `Times(XFromCost, 5)`),
    Sundering Stroke ({3}{R}{R}{R}, 7 damage; divided-damage rider
    omitted), Professor of Symbology ({1}{W}, 1/1 Bird Wizard Flying +
    ETB Learn), Professor of Zoomancy ({1}{G}, 1/1 Squirrel Wizard +
    ETB Squirrel token), Leyline Invocation ({4}{G} Lesson, X/X
    Elemental where X = lands), Verdant Mastery ({3}{G}{G}, two basic-
    land searches), Rise of Extus ({3}{W}{B} Lesson, exile +
    reanimate), Gnarled Professor ({3}{G}, 4/4 Reach + ETB MayDo
    loot), Inkfathom Witch ({2}{B}, 2/2 Flying + Attack/MayPay drain
    2), Blood Researcher ({1}{B}, 1/1 Vampire Wizard with
    LifeGained → +1/+1 counter), First Day of Class ({W}, token-only
    +1/+1 + haste anthem via two ForEach passes).
  - **Engine improvement**: new
    `Effect::hostile_damage_amount(&self) -> Option<i32>` static
    classifier returning the constant damage amount of a damage
    effect. `auto_target_for_effect_avoiding` consults it on hostile
    picks and re-sorts the primary candidate list so creatures whose
    toughness ≤ damage (lethal kills) come first, then by descending
    power. Pre-fix the picker walked battlefield order — could pick a
    2/2 utility creature when a 4/4 next-in-scan was a clean kill.
    Covers `DealDamage(Const)`, `DealDamage(Times(Const, Const))`,
    and `Seq` leading with one. Returns None for X-cost folded values
    (Crackle's `Times(XFromCost, 5)`) since X is only known at
    cast-time.
  - **UI improvement**: `predicate_short_label` (server/view.rs) gained
    arms for `Value::CardsDrawnThisTurn(_)` ("after drawing" / "if
    drew ≥N" / "if drew ≤N") and `Value::PermanentCountControlledBy
    (_)` ("if has permanents" / "if ≥N permanents" / "if ≤N
    permanents"). Pairs with the existing `CountOf` arm for
    permanent-count thresholds, but reads off the per-player tally
    directly.
  - **18 new tests**: 14 in `tests::stx::*` (one per new card +
    `first_day_of_class_pumps_token_creatures_only`), 2 in
    `tests::modern::*` (`heated_debate_auto_target_prefers_lethal_kill`,
    `heated_debate_auto_target_falls_through_when_no_lethal`), 1 in
    `server::view::tests::*`
    (`predicate_short_label_covers_cards_drawn_and_permanent_count`),
    plus 1 misc.

## Future work — engine/UI suggestions surfaced by push XXXII

- **Divided damage primitive** — Sundering Stroke ("7 damage divided
  among 1, 2, or 3 targets"), Magma Opus ("4 damage divided"), Crackling
  Doom ("greatest power" sac), and Volcanic Geyser-class spells all
  collapse to single-target today. A new `Effect::DealDamageDivided
  { amount, max_targets }` with a per-target distribution prompt
  would unlock all of them. Multi-target prompt plumbing through
  `GameAction::CastSpell` is the gating dependency (also blocks
  Render Speechless / Cost of Brilliance / Homesickness — see
  "Multi-Target Prompt" below).
- **Cost-doubling-by-pip-count rider** — Sundering Stroke's "if
  {R}{R}{R}{R} was spent to cast it, deals 14 damage instead" needs
  a `Predicate::ColoredManaSpent(Color, AtLeast)` over
  `Value::ManaSpentToCast`-like primitive. Same family as
  Crackling Geyser-style "X colored mana spent" gates.
- **Lesson sideboard model** — Professor of Symbology's "reveal a
  Lesson card from outside the game" still collapses to Draw 1.
  Five push-XXIX-and-prior cards share this approximation
  (Eyetwitch, Hunt for Specimens, Igneous Inspiration, Field Trip,
  the new Professor of Symbology). Adding a `learn_pool: Vec<
  CardDefinition>` field on `Player` plus a `RevealAndChoose`
  decision shape would unblock the cycle at full fidelity.
- **Effect::Search variant against opponents** — Verdant Mastery's
  "target opponent searches their library for a basic land card,
  puts it onto the battlefield tapped, then shuffles" half is
  omitted. The `Effect::Search { who: PlayerRef, ... }` already
  takes a player ref; the gap is the controller-of-search authority
  (does the caster pick, or does the opp pick?). Could be a new
  flag on `Effect::Search { search_decider: PlayerRef }`.
- **PumpPT label improvement** — `ability_effect_label` returns
  "Pump" for both positive PumpPT (Giant Growth) and negative
  PumpPT (Lash of Malice / Burrog Befuddler / Witherbloom Command's
  -3/-3). A simple sign-aware split ("Pump" vs "Shrink") would
  improve readability in the activated-ability badge UI for the
  ~12 catalog cards that use negative PumpPT.

- ✅ **Push XXXI (2026-05-03)**: Mana-spent-to-cast introspection lands
  + 15 SOS / STX 2021 promotions + new `EventKind::Blocks` event + UI
  label coverage. Tests at 1261 (was 1246, +15 net).
  - **New `Value::ManaSpentToCast`** — reads `cost.cmc() + x_value`
    of the spell on the stack matched by `ctx.trigger_source =
    Card(cid)`. Returns 0 outside a spell context. Implementation
    parallels push XXVII's `Predicate::CastSpellHasX` but exposes the
    actual mana figure rather than a "has X" boolean. Used by the SOS
    Opus + Increment payoff cycle.
  - **New `effect::shortcut::opus(at_least, big, always)`** —
    short-form constructor for the SOS Opus pattern (magecraft
    trigger + ManaSpentToCast gate + always-fires + extra). Used by
    Tackle Artist, Spectacular Skywhale, Muse Seeker, Deluge
    Virtuoso, Exhibition Tidecaller.
  - **New `effect::shortcut::increment()`** — short-form
    constructor for the SOS Increment pattern (any spell cast where
    mana_spent > min(P, T) drops a +1/+1 counter). Used by Berta,
    Cuboid Colony, Fractal Tender, Hungry Graffalon, Pensive
    Professor, Tester of the Tangential, Textbook Tabulator.
  - **New `EventKind::Blocks`** — symmetric to `BecomesBlocked` but
    fires from the *blocker* side of `GameEvent::BlockerDeclared`.
    The dispatcher splits SelfSource scope by event kind: `Blocks`
    reads `blocker == source.id`, `BecomesBlocked` reads `attacker
    == source.id`. Unblocks Daemogoth Titan's "or blocks" rider and
    any future "whenever ~ blocks" trigger.
  - **15 promotions to ✅**: Tackle Artist, Aberrant Manawurm,
    Spectacular Skywhale, Muse Seeker, Deluge Virtuoso, Exhibition
    Tidecaller, Cuboid Colony, Hungry Graffalon, Textbook Tabulator
    (all SOS); Daemogoth Titan, Karok Wrangler (STX 2021). Three
    further promotions to 🟡 with note updates: Pensive Professor,
    Tester of the Tangential, Fractal Tender, Berta (Increment
    half wired, other rider stays gated).
  - **UI improvement**: `predicate_short_label` (server/view.rs)
    gained an arm for `Value::ManaSpentToCast` — formats as "if N+
    mana spent" / "if ≤N mana spent". Same shape as push XXX's
    `AttackersThisCombat` predicate label.
  - **15 new / updated tests**: 9 in `tests::sos::*`
    (`aberrant_manawurm_pumps_by_mana_spent_on_is_cast` +
    `_scales_with_big_spells`, `tackle_artist_opus_small_cast_pumps_eot_only` +
    `_big_cast_adds_counter`, `spectacular_skywhale_opus_big_cast_adds_three_counters`,
    `cuboid_colony_increment_fires_on_two_drop` +
    `_does_not_fire_on_equal_cmc`, `pensive_professor_increment_fires_on_any_cast`,
    `tester_of_tangential_increment_skips_one_mana_cast` + `_fires_on_two_mana_cast`,
    `berta_increment_then_mana_ramp_chains`,
    `mana_spent_to_cast_is_zero_outside_spell_context`); 2 in
    `tests::stx::*` (`daemogoth_titan_block_trigger_sacrifices_another_creature`,
    `karok_wrangler_double_stuns_when_two_wizards`); 1 in
    `server::view::tests::*`
    (`predicate_short_label_covers_mana_spent_to_cast`); 1 signature
    update in `tests::sos::*`
    (`berta_wise_extrapolator_def_is_one_four_legendary_frog_druid` —
    triggered abilities now count 2 rather than 1).

- ✅ **Push XXX (2026-05-02)**: 8 new STX 2021 cards + 2 promotions
  + new `Value::AttackersThisCombat` primitive + filter evaluation
  on broadcast Attack triggers + UI labels for AttackersThisCombat
  and And-composite stack filters. Tests at 1241 (was 1227; +14
  net).
  - **10 new STX 2021 cards**:
    - **Witherbloom**: **Mortality Spear** ✅ ({3}{B}{G} Sorcery —
      Lesson, destroy creature/PW), **Foul Play** ✅ ({2}{B} Instant,
      destroy tapped creature + draw if ≥2 Wizards via
      `Predicate::ValueAtLeast(CountOf(Wizards), 2)`).
    - **Silverquill**: **Dueling Coach** ✅ ({2}{W}, 3/3 Vigilance
      Cleric, magecraft +1/+1 counter), **Hall Monitor** ✅ ({W},
      1/1 Wizard, magecraft CantBlock-EOT grant), **Clever
      Lumimancer** ✅ ({W}, 1/1 Wizard, magecraft self-pump
      +2/+2 EOT), **Karok Wrangler** 🟡 ({2}{W}, 3/3 Wizard, ETB
      tap+stun).
    - **Lorehold**: **Hofri Ghostforge** 🟡 ({2}{R}{W}, 3/4 Legendary
      with anthem), **Mascot Interception** 🟡 ({2}{R}{W} Instant,
      destroy-substitute for "gain control"), **Approach of the
      Lorehold** ✅ ({1}{R}{W} Sorcery, 2 dmg + 1/1 flying Spirit).
    - **Quandrix**: **Augmenter Pugilist** 🟡 ({3}{G}{G}, 6/6 Trample,
      body-only).
  - **2 promotions**:
    - **Dina, Soul Steeper** 🟡 → ✅ — the activated -X/-X EOT now
      scales with `Value::Diff(Const(0), CountOf(EachPermanent(
      Creature ∧ ControlledByYou)))`. Three-creature board → -3/-3.
    - **Augusta, Dean of Order** 🟡 → ✅ — the "two or more creatures
      attack" gate is now real, via the new
      `Value::AttackersThisCombat` primitive +
      `Predicate::ValueAtLeast(AttackersThisCombat, 2)` filter.
      Single-attacker swings no longer false-positive +1/+1 +
      double strike.
  - **Engine improvement**: new `Value::AttackersThisCombat` arm in
    `evaluate_value` reads `state.attacking.len()`. Unblocks
    Augusta (just promoted) and Adriana, Captain of the Guard's
    "for each *other* attacking" pump (just `Diff(
    AttackersThisCombat, 1)`).
  - **Engine improvement**: `combat.rs` declare-attackers broadcast
    now evaluates each `AnotherOfYours` / `YourControl` /
    `AnyPlayer`-scoped Attack trigger's `EventSpec.filter` in a
    second pass (after every attacker is in `self.attacking`), so
    `AttackersThisCombat`-keyed gates read the *final* count
    uniformly across all attackers. Pre-fix the broadcast silently
    ignored every filter on Attack triggers, so any non-trivial
    gate would have been a no-op.
  - **UI improvement**: `predicate_short_label` (server/view.rs)
    gained an arm for `Value::AttackersThisCombat` — formats as
    "if attacking" (≥1) / "if ≥N attackers" / "if ≤N attackers".
  - **UI improvement**: `entity_matches_label` collapses common
    And-composite filters: `IsSpellOnStack ∧ X` strips the "spell"
    qualifier; `ControlledByYou ∧ X` / `ControlledByOpponent ∧ X`
    collapse to "if your X" / "if opp's X". Powers
    Choreographed Sparks's stack-spell filter, Saw It Coming-
    style counter targets, and any "your creature" / "opp's
    artifact" matters.
  - **18 new tests**: 15 in `tests::stx::*` (one per new card +
    one per promotion + Augusta's solo-attacker negative case +
    3 Foul Play tests including untapped-target rejection +
    Wizard-count gate + Clever Lumimancer self-pump), 2 server-side
    view (`entity_matches_label_covers_and_composite_filters`,
    `predicate_short_label_covers_attackers_this_combat`), 1
    replacement test (`augusta_dean_of_order_pumps_when_two_
    attackers` replacing the old `_pumps_attacker`).

- ✅ **Push XXIX (2026-05-02)**: 10 new STX 2021 cards across schools +
  Abrupt Decay MV bug fix + UI Or-composite filter labels. Tests at
  1227 (was 1218; +9 net). Pure card additions + non-blocking UX
  polish — no new engine primitives.
  - **3 new Lorehold (R/W) cards** (`catalog::sets::stx::lorehold`):
    - **Rip Apart** ✅ ({R}{W} Sorcery) — modal removal: 3 dmg to
      creature/PW OR destroy artifact/enchantment via
      `Effect::ChooseMode` (same shape as Boros Charm). Modal pick
      is "choose one" so the implementation matches printed Oracle
      at full fidelity.
    - **Plargg, Dean of Chaos** 🟡 ({1}{R}, 1/3 Legendary Wizard) —
      `{T}: Discard a card, then draw a card` rummage activation.
      The {2}{R} top-3-exile activation is omitted (no exile-from-
      top primitive). The DFC pairing with Augusta is split into
      two separate front-face card definitions (engine MDFC pipeline
      currently lacks an "always-flippable, both faces equally"
      mode).
    - **Augusta, Dean of Order** 🟡 ({1}{W}, 2/2 Vigilance Wizard)
      — per-attacker pump trigger via the `Attacks/AnotherOfYours`
      broadcast. The "two or more creatures attack" gate collapses
      to per-attack — single-attacker case is a minor false
      positive vs. printed text; multi-attacker case matches.
  - **2 new Prismari (U/R) cards** (`catalog::sets::stx::prismari`):
    - **Magma Opus** 🟡 ({7}{U}{R} Sorcery) — finisher: 4 dmg to
      creature/PW + 4/4 Elemental token + draw 2. The "4 dmg
      divided" + "tap two permanents" both collapse to single-
      target picks; the discard-for-Treasure alt cost is omitted
      (alt-cost-by-discard primitive gap).
    - **Expressive Iteration** 🟡 ({U}{R} Sorcery) — collapsed to
      Scry 2 + Draw 1 cantrip approximation. The "exile top 3 +
      may play land + cast spell" rider is omitted (cast-from-exile
      + play-land-from-exile primitive gap).
  - **5 new mono-color staples** (`catalog::sets::stx::mono`):
    - **Environmental Sciences** ✅ ({2} colorless Sorcery —
      Lesson) — basic-land tutor + 2 life. Universal Lesson at
      every color.
    - **Expanded Anatomy** ✅ ({3}{G} Sorcery — Lesson) — three
      +1/+1 counters on a target creature.
    - **Big Play** 🟡 ({3}{G}{U} Instant — Lesson) — untap creature
      + +1/+1 + hexproof + trample EOT. "Up to two" collapses to
      single-target.
    - **Confront the Past** 🟡 ({4}{R} Sorcery — Lesson) —
      `Effect::CounterAbility`. The "steal opponent's PW loyalty
      ability" mode is omitted (dynamic mode-pick from target's
      `loyalty_abilities` list is a brand-new primitive).
    - **Pilgrim of the Ages** ✅ ({3}{W}, 2/3 Spirit Wizard Cleric)
      — death-trigger basic-land recursion to hand. Mirrors
      Pillardrop Rescuer's shape on a mono-white slot.
  - **Engine bug fix**: **Abrupt Decay**'s target filter was
    `ManaValueAtMost(2)` — printed Oracle is "mana value 3 or less".
    Fix: `ManaValueAtMost(3)`. Updated the rejection-cap test to
    swap Phyrexian Arena (CMC 3, now LEGAL) for Sun Titan (CMC 6).
    Added `abrupt_decay_accepts_cmc_three_target` to lock in the
    boundary case.
  - **UI improvement**: `entity_matches_label` Or-composite arm —
    previously, an Or-of-two-types filter (`Creature OR
    Planeswalker`, `Artifact OR Enchantment`) fell through to the
    catch-all "if matches filter". Now binary Or-composites of
    simple type tokens render as "if A/B" — covers Rip Apart's
    targets, Magma Opus, Nature's Claim, Igneous Inspiration, and
    any future binary-Or filter on basic types. Recurses one level
    deep — three-way Or chains keep the generic hint. New helpers
    `or_label` + `simple_type_token`.
  - **13 new tests**: 11 in `tests::stx::*` (one per new card), 1
    in `tests::modern::*` (`abrupt_decay_accepts_cmc_three_target`),
    1 in `server::view::tests::*`
    (`entity_matches_label_covers_or_composite_filters`).

- ✅ **Push XXVIII (2026-05-02)**: Thread trigger subject through
  `StackItem::Trigger` so `PlayerRef::Triggerer` resolves to the actual
  event actor at resolution time. Pre-fix the dispatch path captured
  the subject for the filter check and then discarded it when pushing
  the trigger to the stack — `continue_trigger_resolution` rebuilt
  the context with `trigger_source = Permanent(source)`, overwriting
  the actual triggerer. Now every `StackItem::Trigger` push site
  records the natural subject (ETB → entering permanent, Magecraft /
  Repartee → cast spell, OpponentControl casts → cast spell, Dies →
  dying creature, attack → attacker), threaded through to the
  resolution context. `EntityRef` gains `Serialize` /
  `Deserialize`. `subject` field defaults to `None` via
  `#[serde(default)]` for snapshot back-compat — pre-XXVIII snapshots
  fall back to the source-permanent default.
  - **Sheoldred, the Apocalypse** drain promoted from `EachOpponent`
    collapse (push XXVII) to exact `Triggerer`-keyed targeting. In
    2-player it was already correct; 3+ player now drains *only* the
    drawing opponent.
  - Unblocks future "whenever a player X" payoffs (Tergrid,
    God of Fright; Painful Quandary; Liliana of the Dark Realms;
    Mindblade Render; symmetric drain triggers) that need to attribute
    back to the event actor.

- ✅ **Push XXVII (2026-05-02)**: 6 more cards + UI EntityMatches
  label coverage. Tests at 1214 (was 1207, +7 net).
  - **Cards**: Careful Study ({U}, draw 2 + discard 2), Sheoldred,
    the Apocalypse ({2}{B}{B}, 4/5 deathtouch+lifelink with
    CardDrawn/YourControl → +2 life and CardDrawn/OpponentControl →
    drain 2 to drawing opp), Liliana of the Veil ({1}{B}{B}, +1 each
    player discards / -2 sac creature; -6 omitted), Light Up the Stage
    ({2}{R}, approximated as Draw 2; Spectacle alt cost omitted),
    Liliana of the Last Hope ({1}{B}{B}, +1 -2/-1 EOT / -2 reanimate
    creature card from gy → hand; -7 emblem omitted), Tibalt's
    Trickery ({1}{R}, hard counter; cascade-from-exile rider omitted).
  - **`entity_matches_label` helper** in `server/view.rs` unpacks
    `Predicate::EntityMatches`'s inner filter for common simple cases
    — "if creature" / "if noncreature" / "if artifact" /
    "if multicolored" / "if MV ≤2" — instead of the generic
    "if matches filter" hint. Composite (And/Or) predicates and
    counter-keyed filters keep the generic fallback. Powers Esper
    Sentinel's "if noncreature" gate badge.

- ✅ **Push XXVI (2026-05-02)**: 10 new cube + STX cards +
  OpponentControl SpellCast dispatch. Tests at 1207 (was 1195, +12
  net).
  - **Engine**: extend `fire_spell_cast_triggers` to walk every
    battlefield permanent's SpellCast trigger and route by scope. Pre-
    fix only the caster's permanents were considered (filter on
    `c.controller == caster`), which silently ignored
    `EventScope::OpponentControl` triggers — Esper Sentinel,
    Mindbreak Trap, future "whenever an opponent casts X" payoffs
    would never fire. Now `YourControl` / `AnyPlayer` keep the
    caster-side path; `OpponentControl` walks non-caster permanents
    and fires under the *trigger's* controller.
  - **10 new card factories** in `catalog::sets::decks::modern`:
    Cabal Ritual ({B}, +3{B} → +4{B}+{C} threshold gate via
    `Predicate::ValueAtLeast(GraveyardSizeOf(You), 7)`), Rift Bolt
    ({2}{R}, 3 dmg; Suspend omitted), Ancient Stirrings ({G}, top-5
    reveal colorless via `RevealUntilFind { find: Colorless,
    cap: 5 }`), Stinkweed Imp ({1}{B}, 1/3 Flying +
    DealsCombatDamageToPlayer mill 5; Dredge omitted), Endurance
    ({1}{G}{G}, 3/4 Reach Flash + ETB
    ShuffleGraveyardIntoLibrary; Evoke omitted), Esper Sentinel
    ({W}, 1/1 + Draw on opp's noncreature cast via OpponentControl +
    EntityMatches(Noncreature)), Path of Peril ({2}{B}{B}, ForEach
    Creature ∧ MV≤2 → -3/-3 EOT), Fiery Confluence ({2}{R}{R}, 3-mode
    `ChooseMode`; multi-pick collapse), Brilliant Plan ({3}{U},
    Scry 3 + Draw 3 — STX 2021 mono-blue), Silverquill Apprentice
    ({W}{B}, 2/2 magecraft +1/+1 EOT — STX 2021).

- ✅ **Push XXV (2026-05-02)**: 10 new cards (4 STX 2021 + 6 cube) +
  smarter bot blocking + UI predicate labels + bot/view tests. Tests at
  1195 (was 1179, +16 net). Pure card additions + non-blocking
  bot/UI/UX polish — no new engine primitives.
  - **4 new STX 2021 cards** (`catalog::sets::stx::*`):
    - **Silverquill (W/B)**: Star Pupil ({B}, 0/0 Spirit with ETB +1/+1
      counter rider + dies-counter-on-target rider — printed two-
      counters-on-0/0 collapses to base 1/1 + 1 ETB counter, same
      approximation as Reckless Amplimancer); Codespell Cleric
      ({W}, 1/1 Lifelink Cleric with ETB Scry 1 — fully wired);
      Combat Professor ({3}{W}, 2/3 Flying Cat Cleric with magecraft
      +1/+1 EOT pump — same shape as Eager First-Year on a flier).
    - **Shared / Lessons**: Spirit Summoning ({3}{W}, Sorcery — Lesson:
      1/1 white Spirit token with flying — fills white's slot in the
      STX Lesson cycle alongside Pest Summoning, Inkling Summoning,
      Mascot Exhibition).
  - **6 new cube cards** (`catalog::sets::decks::modern.rs`):
    - Kolaghan's Command ({B}{R}, 4-mode `ChooseMode` — gy-recursion /
      opp-discard / 2-dmg / artifact-destroy; "choose two" collapsed to
      "choose one" same as Boros Charm and the STX Commands), Twincast
      ({U}{U}) and Reverberate ({R}{R}) — both copy target IS via
      `Effect::CopySpell`; Vendetta ({B}, destroy nonblack creature,
      lose 2 life — printed "lose life equal to its toughness" collapses
      to flat 2 since `Value` doesn't read pre-destroy toughness yet),
      Generous Gift ({2}{W}, destroy nonland + opp gets 3/3 Elephant
      via `PlayerRef::ControllerOf(Target(0))`), Crackling Doom
      ({R}{W}{B}, 2 dmg each opp + each opp sacs a creature; "greatest
      power" filter omitted — same gap as Pithing Edict's "creature or
      planeswalker" choice).
  - **Bot improvement**: `pick_blocks` (`server/bot.rs`) now considers
    trades. Pre-fix the bot threw every legal blocker into a random
    legal attacker — suicide blocks (1/1 vs 5/5) chewed through bodies
    for nothing. The new logic carries P/T + relevant keywords
    (flying / reach / deathtouch / indestructible) up-front and
    computes a `trade_score` per (attacker, blocker) pair: killing the
    attacker is the dominant payoff, losing a body is the cost. A
    per-pressure-tier threshold (lethal / critical / normal) gates
    the assignment so the bot stops suicide-blocking at high life and
    chumps under lethal pressure. Greedy assignment by attacker power
    descending.
  - **UI improvement**: `predicate_short_label` (`server/view.rs`)
    gained explicit arms for `ValueAtLeast` / `ValueAtMost` over
    `GraveyardSizeOf` ("≥N in gy"), `LibrarySizeOf` ("≥N in library"),
    `CountOf(_)` ("if ≥N match" / "if board matches"), and a generic
    "if matches filter" for `EntityMatches`. Closes the gap for
    Dragon's Approach's "≥4 in gy" tutor gate (was the catch-all
    "conditional"), Resonating Lute's hand-size gate, and any future
    selector-count predicate.
  - **16 new tests**: 5 STX (`tests::stx::*`), 8 modern
    (`tests::modern::*`), 1 server-side view
    (`server::view::tests::predicate_short_label_covers_value_keyed_predicates`),
    2 server-side bot
    (`server::bot::tests::bot_skips_suicide_block_at_high_life`,
    `bot_chump_blocks_when_lethal_imminent`).

- ✅ **Push XXIV (2026-05-02)**: STX 2021 push — Witherbloom completion
  + 4 cross-school Commands + Saw It Coming + 2 promotions + bot
  life-cost guard + UI plural-tally predicate labels. Tests at 1179
  (was 1159, +20 net). No new engine primitives — pure card additions
  + UX/AI polish.
  - **3 new Witherbloom cards** + **2 promotions** completing the B/G
    school: Daemogoth Titan ({3}{B}{G}, 11/11 Demon Horror with attack
    sac trigger), Pest Infestation ({X}{B}{G}, X Pest tokens with on-die
    +1-life trigger), Witherbloom Command ({B}{G}, 4-mode `ChooseMode`
    instant). Witherbloom Pledgemage promoted via
    `ActivatedAbility.life_cost: 1`. Hunt for Specimens promoted to
    parity with Eyetwitch's Lesson approximation.
  - **4 cross-school Commands** (all 🟡 — printed "choose two" collapses
    to "choose one" via `Effect::ChooseMode`): Lorehold Command (drain 4
    / two flying Spirits / gy → hand MV ≤ 2 / exile gy), Prismari
    Command (2 dmg / discard 2 + draw 2 / Treasure / destroy artifact),
    Quandrix Command (counter ability / +1/+1 ×2 / gy → bottom / draw),
    Silverquill Command (counter ability / -3/-3 / drain 3 / draw).
  - **1 mono-color**: Saw It Coming ({1}{U}{U}) — Cancel-equivalent at
    {1}{U}{U}; Foretell omitted (no Foretell primitive).
  - **Bot improvement**: `is_free_mana_ability` now skips activations
    with `life_cost > 0` or `condition.is_some()`. Witherbloom
    Pledgemage's `{T}, Pay 1 life: Add {B}` no longer auto-fires as a
    "free" mana rock — paying life is a non-trivial cost the random
    bot can't reason about.
  - **UI improvement**: `predicate_short_label` (server/view.rs) now
    formats plural N≥2 thresholds for the per-turn tally predicates
    (`CardsLeftGraveyardThisTurnAtLeast`, `LifeGainedThisTurnAtLeast`,
    `CardsExiledThisTurnAtLeast`, `CreaturesDiedThisTurnAtLeast`). Was
    only n=1 covered; n>1 fell through to "conditional".
  - **20 new tests**: 18 STX (`tests::stx::*`), 1 server-side bot
    (`server::bot::tests::bot_does_not_tap_life_cost_mana_source`),
    1 server-side view (`server::view::tests::
    predicate_short_label_covers_plural_tally_thresholds`).

- ✅ **Push XXIII (2026-05-02)**: 18 new STX 2021 + cube cards + bot
  walker-attack routing + UI predicate label coverage. Tests at 1159
  (was 1132, +27 net). No new engine primitives — pure card
  additions + non-blocking UX/AI improvements.
  - **12 new STX 2021 card factories**:
    - **Witherbloom**: Daemogoth Woe-Eater ({2}{B}{G}, 9/9 Demon
      with sac-on-ETB approximation + `{T}: gain 4 life`),
      Eyeblight Cullers ({1}{B}{B}, 4/4 Elf with sac-on-ETB +
      drain 2), Dina, Soul Steeper ({B}{G}, 1/3 Legendary Deathtouch
      + lifegain → ping opp + -X/-X activation collapsed to flat
      -1/-1).
    - **Lorehold**: Reconstruct History ({1}{R}{W}, return up to 2
      artifacts gy → hand via `Selector::take(_, 2)` + draw 1),
      Igneous Inspiration ({2}{R}, 3 dmg + Learn).
    - **Prismari**: Creative Outburst ({3}{U}{U}{R}{R}, full discard
      via `Value::HandSizeOf(You)` + draw 5).
    - **Quandrix**: Snow Day ({1}{G}{U}, Fractal token + counters
      scaled to hand size), Mentor's Guidance ({2}{G}{U}, draw 2 +
      hand-size +1/+1 counters on a creature).
    - **Mono-color**: Solve the Equation ({2}{U}, IS tutor + scry
      1), Enthusiastic Study ({1}{G}, +2/+2 + trample + Learn),
      Tempted by the Oriq ({1}{W}{B}, destroy-ish + Inkling token —
      printed "gain control" approximated as Destroy ≤3-MV).
  - **6 new cube cards in `modern.rs`**: Boros Charm (3-mode modal
    instant), Dragon's Rage Channeler (1/1 + on-noncreature-cast
    Surveil 1), Unholy Heat (3-dmg removal), Pelt Collector (1/1
    body), Frantic Inventory (graveyard-tally cantrip), Pegasus
    Stampede (2 fliers + flashback).
  - **Bot improvement**: `server/bot.rs` DeclareAttackers branch now
    routes attackers at opp planeswalkers when their power matches
    the walker's loyalty — greedy first-fit accumulator, advances
    walkers in alpha-strike turns. Closes "Bot / AI — Planeswalker
    Targeting" in TODO.md.
  - **UI improvement**: `predicate_short_label` (server/view.rs)
    gained explicit arms for `SelectorExists`,
    `SelectorCountAtLeast { n }`, `IsTurnOf`, `All`/`Any` boolean
    combinators (with empty-list collapses), `Not`, `True`,
    `False` — six previously-unhandled Predicate variants now read
    naturally in tooltips.
  - **27 new tests**: 13 STX (`tests::stx::*`), 9 modern
    (`tests::modern::*`), 5 server-side (`server::view::tests`,
    `server::bot::tests`).

- ✅ **Cube + STX push XXII (2026-05-02)**: `SelectionRequirement::HasName`
  predicate + Dragon's Approach 🟡 → ✅ promotion + 17 new card
  factories + Rofellos rewire + Frantic Search comment cleanup.
  Tests at 1132 (was 1110, +22 net):
  - **New `SelectionRequirement::HasName(Cow<'static, str>)`** — name-
    match predicate. Wired into both `evaluate_requirement` and
    `evaluate_requirement_on_card` so it works for both target prompts
    and library/graveyard counts. Powers Dragon's Approach + Slime
    Against Humanity + future "named X" payoffs (Rat Colony, Persistent
    Petitioners, Shadowborn Apostle, etc.). Stored as `Cow` so card-
    side construction is allocation-free for `&'static str` literals
    while snapshot restore (which builds owned strings from JSON)
    avoids leaking.
  - **Dragon's Approach** 🟡 → ✅: full wire of "if 4+ DA in graveyard,
    may search for a Dragon" via `Predicate::ValueAtLeast(CountOf(
    CardsInZone(Graveyard, HasName)), Const(4))` gating an
    `Effect::Search { Creature ∧ Dragon → Battlefield(untapped) }`
    branch.
  - **17 new card factories** (modern.rs):
    - **Pongify** ({U}) + **Rapid Hybridization** ({U}) — destroy
      creature, controller gets a 3/3 token (Ape / Lizard distinct
      tokens via the new `ape_token()` / `lizard_token()` helpers).
    - **Mulldrifter** ({4}{U}) — 2/2 Flying Elemental, ETB Draw 2.
    - **Wall of Omens** ({1}{W}) — 0/4 Defender, ETB Draw 1.
    - **Sun Titan** ({4}{W}{W}) — 6/6 Vigilance, ETB+attacks recur ≤3-
      MV permanent from your graveyard.
    - **Solemn Simulacrum** ({4}) — 2/2 with ETB land tutor + death
      draw.
    - **Three Visits** ({1}{G}) — Forest tutor untapped (Nature's Lore
      twin).
    - **Fume Spitter** ({B}) — 1/1 with sac → -1/-1 EOT.
    - **Galvanic Blast** ({R}) — 2 dmg, Metalcraft → 4 dmg via the new
      `Predicate::ValueAtLeast(CountOf(Artifact & ControlledByYou), 3)`
      branching.
    - **Pithing Edict** ({1}{B}) — each opponent sacrifices a
      creature/PW.
    - **Lash of Malice** ({B}) — -2/-2 EOT.
    - **Aether Adept** ({1}{U}{U}) — 2/2 with ETB Unsummon.
    - **Wind Drake** ({2}{U}) — 2/2 Flying Drake (vanilla baseline).
    - **Cursecatcher** ({U}) — 1/1 Merfolk Wizard, sac →
      `CounterUnlessPaid({1})`.
    - **Resilient Khenra** ({2}{G}) — 3/2 with death-pump on a
      friendly creature.
    - **Persistent Petitioners** ({1}{U}) — 1/3 Advisor, `{1},{T}`
      mill 1 (the tap-4-Advisors mode is omitted).
    - **Slime Against Humanity** ({1}{G}) — X+1 Ooze tokens, X = SAH in
      your gy (HasName-driven).
  - **Rofellos, Llanowar Emissary** rewire — `{T}: Add {G}` now scales
    via `ManaPayload::OfColor(Green, CountOf(Forest & ControlledByYou))`
    (push VI primitive that previously only powered Topiary Lecturer).
    Flat-`{G}{G}` collapse removed; Forest count snowballs as printed.
  - **22 new tests** in `tests::modern::*` and `tests::stx::*` covering
    Pongify/Hybridization controller-of-target tokens, Sun Titan low-MV
    recursion + high-MV no-op, Galvanic Blast metalcraft branching,
    Cursecatcher sac-counter, Slime Against Humanity X scaling, Dragon's
    Approach gate-skip + tutor, Rofellos forest scaling. All 1132 lib
    tests pass.

- ✅ **SOS push XXI (2026-05-02)**: `Effect::CopySpell` first-class
  implementation + `Selector::CastSpellSource` + 7 SOS card promotions
  to ✅. Tests at 1110 (was 1103, +7):
  - **`Effect::CopySpell { what, count }`** — was a stub, now wires
    end-to-end. Resolves `what` to a `CardId`, finds the matching
    `StackItem::Spell` on the stack, and pushes `count` copies onto
    the stack with `is_copy: true`. Each copy gets a fresh `CardId`,
    inherits the original's target / mode / x_value / converged_value,
    and is controlled by the *source's controller* (the listener that
    fired the trigger), matching MTG's "you may copy that spell"
    semantic. Permanent-spell copies are not yet supported (rule
    707.10b token-version path is a follow-up).
  - **`StackItem::Spell.is_copy: bool`** — new field with
    `#[serde(default)]` for snapshot back-compat. Threaded into
    `continue_spell_resolution_with_face_copy` so a copy resolving
    doesn't go to the graveyard or exile (copies cease to exist per
    rule 707.10). Counter spell paths also recognize the flag — a
    countered copy is dropped silently instead of going to the
    caster's graveyard.
  - **`Selector::CastSpellSource`** — resolves to the topmost
    `StackItem::Spell` on the stack. Since `SpellCast` triggers fire
    *above* the cast spell, the topmost remaining Spell at trigger-
    resolution time is the just-cast spell. Used by `CopySpell`'s
    "copy that spell" semantic.
  - **`SelectionRequirement::ControlledByYou` / `ControlledByOpponent`
    fall through to stack-resident spells** — was battlefield-only;
    now finds the spell on the stack (caster = controller) when the
    target is a stack-resident spell. Powers Choreographed Sparks's
    "target IS spell *you control*" filter.
  - **`push_on_cast_triggers` filter threading** —
    `collect_self_cast_triggers` now returns `(Effect,
    Option<Predicate>)` pairs and `push_on_cast_triggers` evaluates
    the filter against the cast spell as `trigger_source` before
    pushing. Powers Lumaret's Favor's "if you gained life this turn"
    Infusion gate without firing the copy trigger when the gate
    fails.
  - **7 promotions to ✅**: Aziza Mage Tower Captain (magecraft tap-3
    + copy), Mica Reader of Ruins (magecraft sac-artifact + copy),
    Lumaret's Favor (Infusion on-cast self-trigger + copy), Silverquill
    the Disputant (Casualty 1 grant approximated via magecraft + may-sac
    + copy), Social Snub (on-cast may-copy with creature-control gate),
    Harsh Annotation (token now goes to destroyed creature's controller
    via `PlayerRef::ControllerOf(Target(0))` + graveyard fallback), and
    Choreographed Sparks (NEW factory — single-mode "Copy target IS
    spell you control" via `IsSpellOnStack & ControlledByYou` filter).
  - **8 new tests** in `tests::sos::*` and `snapshot::tests`. All 1110
    lib tests pass.

- ✅ **STX 2021 push XX (2026-05-02)**: 19 new STX 2021 card factories +
  1 engine primitive (`SelectionRequirement::Monocolored`) + 2 STX
  promotions (Vanishing Verse + Beledros Witherbloom both 🟡 → ✅).
  Tests at 1102 (was 1079, +23 new):
  - **`SelectionRequirement::Monocolored`** — sibling to push VII's
    `Multicolored` and `Colorless`. Matches when a card's mana cost
    contains exactly one distinct colored pip (`distinct_colors() ==
    1`). Wired into both `evaluate_requirement` (battlefield/permanent)
    and `evaluate_requirement_on_card` (library/non-bf zones), so it
    works for both target prompts and library searches.
  - **Vanishing Verse 🟡 → ✅** — target filter promoted to `Permanent
    ∧ Nonland ∧ Monocolored` via the new predicate. Two-color and
    colorless permanents now reject as invalid targets at cast time.
  - **Beledros Witherbloom 🟡 → ✅** — "Pay 10 life: Untap each land
    you control. Activate only as a sorcery." now wired via push XV's
    `ActivatedAbility.life_cost: u32` gate + `Effect::Untap` over
    `Selector::EachPermanent(Land & ControlledByYou)`. Sorcery-speed
    flag set true; pre-flight life check rejects with
    `InsufficientLife` when life < 10.
  - **19 new STX 2021 cards** in `catalog::sets::stx::mono`:
    Pillardrop Warden, Beaming Defiance, Ageless Guardian, Expel
    (white); Eureka Moment, Curate, Skyswimmer Koi, Stonebinder's
    Familiar (blue); Necrotic Fumes, Specter of the Fens (black);
    Ardent Dustspeaker, Dragon's Approach (red); Bookwurm, Spined
    Karok, Field Trip, Reckless Amplimancer (green); Square Up,
    Thrilling Discovery, Quandrix Cultivator, Quintorius Field
    Historian (multicolor). 10 of the 19 ship as ✅ on existing
    primitives; 9 ship as 🟡 with one-line gaps tracked in the
    table.
  - 22 new tests in `tests::stx::*`. All 1102 lib tests pass.

- ✅ **SOS push XIX (2026-05-02)**: Lorehold school complete + 11 SOS
  cards added/promoted (1 ✅ + 10 🟡) + UI label cleanup. Tests at
  1079 (was 1063, +16 new):
  - **Molten Note** ⏳ → ✅: Lorehold's last ⏳ row closes. Wired the
    full "amount of mana spent" damage formula by branching on
    `Predicate::CastFromGraveyard` (push XVIII) — hand cast deals
    `XFromCost + 2`, flashback cast deals 8 (the fixed {6}{R}{W}
    mana spent). Untap-all-your-creatures + Flashback {6}{R}{W}
    wired faithfully.
  - **10 ⏳→🟡 body-only / partial wires**: Strife Scholar, Campus
    Composer, Elemental Mascot (with magecraft pump), Biblioplex
    Tomekeeper, Strixhaven Skycoach (with ETB land tutor), Skycoach
    Waypoint (mana ability), Silverquill the Disputant, Quandrix the
    Proof, Prismari the Inspiration, Social Snub (full mass-sac +
    drain wire). Together: 4/5 Elder Dragons body-wired (only
    Witherbloom + Lorehold finishers were already done from earlier
    pushes), 3 Ward bodies (Strife, Campus, Prismari), 2 colorless
    artifact bodies, 1 colorless utility land.
  - **Lorehold school = fully implemented** (0 ⏳ rows). Joins
    Witherbloom (push XV) as the second school with no remaining
    ⏳ entries. Remaining ⏳: 9 cards across Blue (1), Red (4),
    Silverquill (2), Quandrix (1), Colorless (1) — all blocked on
    new primitives (copy-spell, Cascade, Prepare, Vehicle/Crew,
    cast-from-exile pipeline).
  - **Server view cleanup**: `Predicate::CastSpellHasX` label
    updated to the more readable "when you cast an X spell" (was
    "cast spell w/ {X}"); `CastSpellTargetsMatch` similarly. Push
    XVIII suggestion item closed.
  - **Lib hygiene**: minor indent fix in `hydro_channeler()` (two
    `life_cost: 0,` lines were misaligned vs sibling fields).
  - **Doc updates**: STRIXHAVEN2.md tables progress 100/135/20 →
    101/145/9 (✅/🟡/⏳).

- ✅ **SOS push XVIII (2026-05-02)**: 3 engine primitives + 5 new SOS
  cards + 4 promotions. Tests at 1063 (was 1050):
  - **Combat-damage gy-broadcast** — `fire_combat_damage_to_player_
    triggers` now walks the attacker's controller's graveyard for
    `EventScope::FromYourGraveyard` triggers, in addition to the
    attacker's own SelfSource/AnyPlayer triggers. Two trigger families
    resolve here. Unblocks Killian's Confidence's "may pay {W/B} to
    return from gy" recursion.
  - **`StackItem::Spell.face: CastFace`** — push XIV's `CastFace` enum
    is now stamped onto the `StackItem::Spell` itself (with serde-
    default for snapshot back-compat) and threaded into
    `EffectContext.cast_face` at resolution time via the new
    `continue_spell_resolution_with_face` entry point. `cast_flashback`
    sets `pending_cast_face = Flashback` before delegating.
  - **`Predicate::CastFromGraveyard`** — reads `EffectContext.
    cast_face` and matches `CastFace::Flashback`. Powers Antiquities
    on the Loose's "Then if this spell was cast from anywhere other
    than your hand, put a +1/+1 counter on each Spirit you control"
    rider — the cast-from-gy branch now adds counters faithfully.
  - **5 new SOS cards**: Grave Researcher // Reanimate (MDFC, ETB
    Surveil 2 + back-face Reanimate), Emeritus of Ideation //
    Ancestral Recall (MDFC, 5/5 Ward 2 + back-face draw 3), Mica
    Reader of Ruins (body-only 4/4 Ward 3), Colorstorm Stallion (3/3
    Ward 1 Haste + magecraft pump), Killian's Confidence's gy-trigger
    fully wired.
  - **4 promotions to ✅**: Antiquities on the Loose (cast-from-gy
    counter rider), Killian's Confidence (gy-trigger), Colossus of
    the Blood Age (death rider was already wired — doc flip),
    plus the 4 doc-flips waiting from XVII (Pursue the Past,
    Witherbloom Charm, Stadium Tidalmage, Heated Argument).
  - **Server**: Snapshot round-trip test for `face` on `StackItem::
    Spell` (closes part of XV server suggestion). View label "if cast
    from gy" added for `Predicate::CastFromGraveyard`.
  - **Doc updates**: STRIXHAVEN2.md tables progress 97/134/24 →
    100/135/20 (✅/🟡/⏳).

- ✅ **SOS push XVII (2026-05-01)**: 4 engine primitives + 5 SOS card
  promotions + 8 new STX 2021 card factories. Tests at 1050 (+13
  net):
  - **`Value::CardsDiscardedThisResolution`** + sibling
    **`Selector::DiscardedThisResolution(SelectionRequirement)`** —
    per-resolution counter (u32) and id list (Vec<CardId>) bumped by
    every `Effect::Discard` invocation in the same `Effect::Seq`
    resolution. Reset on every entry to `resolve_effect`. Both
    player-chosen `DiscardChosen` and random-discard
    (`Effect::Discard{ random: true }`) feed the tally so callers
    don't need to know which discard mode is in play. The selector
    walks each player's graveyard to locate the discarded
    `CardInstance`, runs the card-level filter on it, and yields the
    matching ids as `EntityRef::Card`. Promoted Borrowed Knowledge
    mode 1 (now exact-printed via the value), Colossus of the Blood
    Age's death rider (discard hand → draw discarded+1), and Mind
    Roots's "Put up to one land card discarded this way onto the
    battlefield tapped" (the second half was previously dropped
    entirely).
  - **`resolve_zonedest_player` flatten-You fix** — the helper that
    pre-resolves selector-based `PlayerRef` in `ZoneDest` was only
    flattening `OwnerOf`/`ControllerOf`, leaving `PlayerRef::You`
    unresolved. Caused `place_card_in_dest` to mis-resolve `You` to
    the wrong seat when the source card lived in a different
    player's zone. Mind Roots's "discard from opp → land to *your*
    bf" silently routed the land to the opponent's battlefield.
    Now flattens every non-`Seat` variant via `resolve_player(ctx)`.
  - **Combat-side broadcast for `EventKind::Attacks/AnotherOfYours`**
    — `declare_attackers` now consults all your permanents'
    `Attacks/AnotherOfYours` triggers, pre-binding the just-declared
    attacker as `Target(0)`. Promotes Sparring Regimen's
    "whenever you attack, put a +1/+1 counter on each attacking
    creature" rider to ✅. The self-source attack-trigger walk on
    the attacker's own card unchanged.
  - **`Value::CountersOn` graveyard fallback** — extended the
    counter lookup to walk graveyards when the source is no longer
    on battlefield. Promotes Scolding Administrator's death-
    trigger counter transfer (`If it had counters on it, put those
    counters on up to one target creature`). The counters survive
    the bf-to-gy transition (engine only clears
    `damage`/`tapped`/`attached_to`), so the Value reads the right
    count off the graveyard-resident card.
  - **5 SOS promotions (🟡 → 🟡 with full wiring)**: Borrowed
    Knowledge mode 1, Colossus death rider, Mind Roots,
    Scolding Administrator, Sparring Regimen.
  - **8 new STX 2021 card factories** (`catalog::sets::stx::mono`):
    Charge Through ({G} ✅: pump+trample+draw), Resculpt ({1}{U} ✅:
    exile artifact/creature, owner mints 4/4 Elemental), Letter of
    Acceptance ({3} ✅: Scry+Draw artifact with sac-draw activation),
    Reduce to Memory ({2}{U} 🟡: exile + Inkling token), Defend the
    Campus ({3}{R}{W} 🟡: -3/-0 EOT on attacker), Conspiracy
    Theorist ({R} 🟡: 1/3 body), Honor Troll ({2}{W} 🟡: 0/3 body),
    Manifestation Sage ({2}{G}{U} 🟡: 3/3 Flying with Magecraft
    HandSize-3 pump).
  - 14 new tests in `tests::sos::*` and `tests::stx::*`. All 1050
    lib tests pass (was 1037).

- ✅ **SOS push XVI (2026-05-01)**: 5 engine primitives + 10 SOS/STX
  card promotions. Tests at 1025 (+13 net):
  - **`Predicate::CastSpellHasX`** — cast-time introspection on the
    just-cast spell's `{X}` symbols. Used by Quandrix's "whenever
    you cast a spell with `{X}` in its mana cost" payoffs.
  - **`Effect::MayPay { description, mana_cost, body }`** — sibling
    to push XV's `Effect::MayDo`, but with a mana-cost payment.
    Decline / can't-afford skip the body silently. Powers Bayou
    Groff's "may pay {1} to return on death" + future "may pay X
    to do Y" patterns.
  - **`SelectionRequirement::HasXInCost`** — card-level filter
    matching cards whose printed cost has at least one `{X}` pip.
    Wires Paradox Surveyor's "land OR card with {X} in cost"
    reveal filter to its exact-printed shape.
  - **`Value::LibrarySizeOf(PlayerRef)`** — `players[p].library
    .len()`. Promotes Body of Research from `GraveyardSizeOf`
    proxy to the printed library-size predicate.
  - **`shortcut::cast_has_x_trigger(effect)`** — Magecraft/Repartee-
    style helper for "whenever you cast a spell with {X}" payoffs.
  - **`Selector::CardsInZone(Hand)` filter-evaluation fix** —
    routing through `evaluate_requirement_on_card` (the card-level
    evaluator) instead of `evaluate_requirement_static` (which
    walks battlefield → graveyard → exile → stack only). Fixes
    silent zero-results for hand-source predicates.
  - **10 card promotions**: Geometer's Arthropod (⏳→✅),
    Matterbending Mage (🟡→✅), Paradox Surveyor (🟡→✅), Embrace
    the Paradox (🟡→✅), Sundering Archaic (🟡 — `{2}` activated
    ability wired), Aziza Mage Tower Captain (⏳→🟡 body-only),
    Zaffai and the Tempests (⏳→🟡 body-only); STX: Bayou Groff
    (🟡→✅), Felisa Fang of Silverquill (🟡→✅), Body of Research
    (🟡→✅).
  - 13 new tests in `tests::sos::*` and `tests::stx::*`. All 1025
    lib tests pass (was 1012).

- ✅ **SOS push XV (2026-05-01)**: Witherbloom (B/G) school complete +
  `Effect::MayDo` primitive + `ActivatedAbility.life_cost` field + 9
  card touches (3 new + 6 promotions/expansions):
  - **`Effect::MayDo { description: String, body: Box<Effect> }`** —
    first-class "you may [body]" primitive. Emits a yes/no decision via
    `Decision::OptionalTrigger`; only runs `body` when the decider
    answers `Bool(true)`. `AutoDecider` defaults to `false` (skip),
    matching MTG's "you may" defaults. Walkers
    (`requires_target`, `primary_target_filter`,
    `target_filter_for_slot_in_mode`) recurse into the inner body so
    target prompts/filters carry through correctly. The `description`
    is `String` (not `&'static str`) because `Effect` derives
    `Deserialize` via `GameState`.
  - **`ActivatedAbility.life_cost: u32`** — pre-flight life-payment
    gate on activations. Rejects activation cleanly with new
    `GameError::InsufficientLife` when controller's life is below the
    cost; pays up front after tap/mana succeed. Backed by
    `#[serde(default)]` for snapshot back-compat. The `cost_label`
    rendering in `server::view` shows "Pay N life" tokens.
    Powers Great Hall of the Biblioplex's `{T}, Pay 1 life: Add one
    mana of any color` faithfully — the effect is a pure `AddMana`,
    so the ability still resolves immediately as a true mana ability.
  - **Lluwen, Exchange Student // Pest Friend** 🟡 — Witherbloom MDFC
    (3/4 Legendary Elf Druid front + Pest-token sorcery back). Closes
    out the Witherbloom (B/G) school (zero ⏳ rows remaining for the
    school).
  - **Great Hall of the Biblioplex** 🟡 — Legendary colorless utility
    land. `{T}: Add {C}` + `{T}, Pay 1 life: Add one mana of any
    color` (via `life_cost: 1`). The `{5}: becomes 2/4 Wizard
    creature` clause is omitted (no land-becomes-creature primitive).
  - **Follow the Lumarets** 🟡 — `{1}{G}` Sorcery with the Infusion
    rider. `If(LifeGainedThisTurn) → 2× pull : 1× pull` over the top 4
    library cards (find creature-or-land → hand). Misses go to
    graveyard (engine default for `RevealUntilFind`).
  - **Erode** ✅ (was 🟡) — basic-land tutor for the target's
    controller now wired via
    `Search { who: ControllerOf(Target(0)), filter: IsBasicLand,
    to: Battlefield(ControllerOf(Target(0)), tapped) }`. The "may"
    optionality is collapsed to always-search (decline path covered
    by `Effect::Search`'s decider returning `Search(None)`).
  - **5 promotions via `Effect::MayDo`**: Stadium Tidalmage (ETB +
    Attacks loot), Pursue the Past (discard+draw chain), Witherbloom
    Charm mode 0 (sacrifice→draw 2), Heated Argument (gy-exile +
    2-to-controller rider), Rubble Rouser (ETB rummage). All five had
    been collapsed to always-on; now correctly opt-in.
  - 13 new tests in `tests::sos::*` (Lluwen P/T + back-face Pest
    minting; Great Hall mana abilities including the life-cost
    prepay; Follow the Lumarets mainline + Infusion paths;
    `MayDo`-skip tests for each promoted card to ensure the
    AutoDecider's `false` answer keeps the body unfired). All 1012
    lib tests pass.

- ✅ **SOS pushes XI / XII / XIII / XIV (2026-05-01)**: 29 new MDFC
  factories + 3 engine improvements + 44 new tests:
  - **Push XI**: 17 MDFC factories (Elite Interceptor // Rejoinder,
    Emeritus of Truce // Swords to Plowshares, Honorbound Page // Forum's
    Favor, Joined Researchers // Secret Rendezvous, Quill-Blade Laureate
    // Twofold Intent, Spiritcall Enthusiast // Scrollboost, Encouraging
    Aviator // Jump, Harmonized Trio // Brainstorm, Cheerful Osteomancer
    // Raise Dead, Emeritus of Woe // Demonic Tutor, Scheming Silvertongue
    // Sign in Blood, Adventurous Eater // Have a Bite, Emeritus of
    Conflict // Lightning Bolt, Goblin Glasswright // Craft with Pride,
    Emeritus of Abundance // Regrowth, Vastlands Scavenger // Bind to
    Life, Leech Collector // Bloodletting, Pigment Wrangler // Striking
    Palette). All 🟡 (front-face vanilla + back-face spell wired). New
    `catalog::sets::sos::mdfcs` module with `vanilla_front` /
    `spell_back` helpers keeping per-card boilerplate under 20 lines.
    24 new tests.
  - **Push XII**: 12 more MDFC factories — 7 mono-color (Spellbook
    Seeker, Skycoach Conductor, Landscape Painter, Blazing Firesinger,
    Maelstrom Artisan, Scathing Shadelock, Infirmary Healer) + 5 legendary
    multicolor (Jadzi, Sanar, Tam, Kirol, Abigale). All 🟡. 16 new
    tests.
  - **Push XIII** (engine): `Player.instants_or_sorceries_cast_this_turn`
    + `Player.creatures_cast_this_turn` tallies bumped in `finalize_cast`
    (when the resolving spell carries `CardType::Instant`/`Sorcery`/
    `Creature`). Reset on `do_untap`. New predicates
    `Predicate::InstantsOrSorceriesCastThisTurnAtLeast` and
    `Predicate::CreaturesCastThisTurnAtLeast`. Surfaced through
    `PlayerView` (with `#[serde(default)]`). Promotes Potioner's Trove's
    lifegain ability gate from the proxy `SpellsCastThisTurnAtLeast` →
    exact `InstantsOrSorceriesCastThisTurnAtLeast`. New gate label
    strings ("after instant/sorcery cast", "after creature cast") in
    `predicate_short_label`. 2 new tests.
  - **Push XIV** (engine + server): `enum CastFace { Front, Back,
    Flashback }` threaded through `GameEvent::SpellCast.face` +
    `GameEventWire::SpellCast.face`. Replays / spectator UIs can now
    distinguish back-face MDFC casts from normal hand casts and from
    flashback graveyard replays. New transient
    `GameState.pending_cast_face`; `cast_spell_back_face` sets `Back`
    before delegating, `cast_flashback` emits `Flashback` directly,
    default cast paths emit `Front`. 2 new tests.
  - All 997 lib tests pass (was 953; +44 net).
  - Cube color pool wiring: 6 white, 6 blue, 6 black, 5 red, 3 green
    MDFCs added; legendary multicolor MDFCs (Sanar UR, Tam GU, Kirol
    RW, Abigale WB) added to the matching cross-pools.

- ✅ **SOS push X (2026-05-01)**: 5 new SOS card factories (1 ✅, 4 🟡)
  + 4 promotions from 🟡 to ✅ (Flashback wirings) + 3 engine
  primitives:
  - **`Selector::Take { inner, count }`** — wraps another selector to
    clamp how many entities flow through (in resolution order). Sugar:
    `Selector::one_of(inner)`, `Selector::take(inner, n)`. Promoted
    Practiced Scrollsmith's gy-exile from "every matching" to "exactly
    one"; lifted Pull from the Grave from one creature to two. The
    target-filter/`requires_target` walkers recurse into the `inner`
    arm so wrapping a `TargetFiltered`/`CardsInZone` selector is
    transparent. Closes the long-standing "Move at most one matching
    card" / `Selector::OneOf` gap.
  - **`GameAction::CastSpellBack`** + **`cast_spell_back_face`** —
    generalises `PlayLandBack` to non-land MDFC back faces. Mirrors
    the `PlayLandBack` flow: swaps the in-hand card's `definition` to
    the back face's, then routes through `cast_spell` so cost / type
    / target filters / effect all resolve against the back face.
    First non-land MDFC wired: **Studious First-Year // Rampant
    Growth**. The 3D client picks this up automatically — the
    right-click flip on hand cards now routes flipped non-land
    MDFCs through `CastSpellBack` (in addition to `PlayLandBack` for
    land MDFCs). New `TargetingState.back_face_pending` flag carries
    the routing through the targeting prompt.
  - **`Keyword::Flashback` wirings on 7 SOS cards** — Daydream, Dig
    Site Inventory, Practiced Offense, Antiquities on the Loose,
    Pursue the Past, Tome Blast, Duel Tactics. Promotes Daydream,
    Dig Site Inventory, Tome Blast, Duel Tactics to ✅ (the only
    omission was Flashback, which is now wired via the engine's
    existing `cast_flashback` path). Antiquities, Pursue the Past,
    and Practiced Offense stay 🟡 because of separate non-Flashback
    omissions (cast-from-elsewhere rider, may-discard collapse,
    lifelink-or-DS mode pick).
  - 14 new tests in `tests::sos::*`. Cards: Inkshape Demonstrator 🟡,
    Studious First-Year // Rampant Growth ✅, Fractal Tender 🟡,
    Thornfist Striker 🟡, Lumaret's Favor 🟡; Daydream ✅, Dig Site
    Inventory ✅, Tome Blast ✅, Duel Tactics ✅, Practiced Offense 🟡,
    Pursue the Past 🟡, Antiquities on the Loose 🟡; Practiced
    Scrollsmith 🟡 (now exact one-card exile), Pull from the Grave 🟡
    (now up-to-2). All 953 lib tests pass.

- ✅ **SOS push IX (2026-05-01)**: 12 new SOS card factories
  (5 ✅, 7 🟡) plus one new engine primitive, finishing the
  Witherbloom (B/G) school (only the Lluwen MDFC remains, blocked
  on cast-from-secondary-face plumbing):
  - **`Player.creatures_died_this_turn`** + **`Predicate::CreaturesDiedThisTurnAtLeast`**
    — per-turn tally bumped from both the SBA dies handler in
    `stack.rs::apply_state_based_actions` (lethal-damage path) and
    `remove_to_graveyard_with_triggers` (destroy-effect path). Reset
    on `do_untap`. Surfaced through `PlayerView.creatures_died_this_turn`.
    Powers Essenceknit Scholar's end-step gated draw.
  - **`CreatureType::Dryad`** + **`PlaneswalkerSubtype::Dellian`** —
    new subtypes for Witherbloom-flavoured cards.
  - 17 new tests in `tests::sos::*` (ETB triggers, end-step gated
    draws, planeswalker loyalty activations, Surveil-anchored
    instants/sorceries, plus a tally-bumps-on-lethal-damage SBA test).
    All 932 lib tests pass.
  - Cards: Essenceknit Scholar ✅, Unsubtle Mockery ✅, Muse's
    Encouragement ✅, Prismari Charm ✅; Professor Dellian Fel 🟡,
    Textbook Tabulator 🟡, Deluge Virtuoso 🟡, Moseo Vein's New
    Dean 🟡, Stone Docent 🟡, Page Loose Leaf 🟡, Ral Zarek Guest
    Lecturer 🟡, Flow State 🟡.
  - Several 🔍-needs-review cards previously flagged as
    "Needs: Surveil keyword primitive" in the auto-generated table
    were already unblocked — Surveil is a first-class
    `Effect::Surveil` primitive. The script's
    `COMPLEX_KWS`/keyword-heuristic was stale. Fixed in-doc; future
    `gen_strixhaven2.py` runs should drop "Surveil" from
    `COMPLEX_KWS` so newly-fetched cards don't get flagged.

- ✅ **SOS push VIII (2026-05-01)**: 14 new SOS card factories
  (2 ✅, 12 🟡) plus two engine primitives that unblock conditional
  activations and counter-add self triggers:
  - **`ActivatedAbility.condition: Option<Predicate>`** — first-class
    "activate only if …" gate. Evaluated against the controller/source
    context **before** any cost is paid, so a failed gate doesn't burn
    the tap-cost or once-per-turn budget. New
    `GameError::AbilityConditionNotMet` for failed gates. Powers
    Resonating Lute's `{T}: Draw a card. Activate only if you have
    seven or more cards in your hand.` and promotes Potioner's Trove's
    lifegain ability to its printed gate. The struct field is
    `#[serde(default)]`; all 100+ existing literal initializations
    pick up `condition: None` via a one-shot patch.
  - **`EventScope::SelfSource` + `EventKind::CounterAdded` recognition**
    — `event_card`/`SelfSource` now match CounterAdded events to the
    source card. Berta, Wise Extrapolator's "whenever one or more +1/+1
    counters are put on Berta, add one mana of any color" trigger now
    fires only when counters land on Berta. Same hook unblocks
    Heliod-style "whenever a counter is put on this …" payoffs.
  - 19 new tests in `tests::sos::*`. Cards: Primary Research ✅,
    Artistic Process ✅, Decorum Dissertation 🟡, Restoration Seminar 🟡,
    Germination Practicum 🟡, Ennis the Debate Moderator 🟡, Tragedy
    Feaster 🟡, Forum Necroscribe 🟡, Berta the Wise Extrapolator 🟡,
    Paradox Surveyor 🟡, Magmablood Archaic 🟡, Wildgrowth Archaic 🟡,
    Ambitious Augmenter 🟡, Resonating Lute 🟡. Potioner's Trove was
    previously 🟡 (no gate); the gate is now wired so its lifegain
    ability rejects activation without an IS-cast that turn.
  - All 910 lib tests pass.

- ✅ **SOS push VII (2026-05-01)**: 11 new SOS card factories
  (3 ✅, 8 🟡) + 2 promotions (Owlin Historian 🟡 → ✅; Postmortem
  Professor's printed `Keyword::CantBlock` now wired). Engine adds:
  - **`SelectionRequirement::Multicolored`** + **`Colorless`** —
    counts the distinct colored pips in a card's mana cost (hybrid
    pips count both halves; Phyrexian counts the colored side;
    generic / colorless / Snow / X don't count). Backed by the new
    `ManaCost::distinct_colors()` helper. Wired into both the
    battlefield-resolve and library-search requirement evaluators
    so it works for cast-time triggers and selector-based
    cardpool filters. Promotes Mage Tower Referee
    (multicolored-cast → +1/+1 counter); ready for any future
    "multicolored matters" / "colorless matters" payoff.
  - **`tap_add_colorless()` shared helper** under
    `catalog::sets::mod` — `{T}: Add {C}` mana ability shorthand
    used by Petrified Hamlet and ready for Wastes / Eldrazi-flavoured
    colorless lands.
  - 11 new functionality tests in `tests::sos::*` + 3 in
    `tests::mana::*`. All 885 lib tests pass.
  - Cards: Mage Tower Referee ✅, Additive Evolution ✅, Owlin
    Historian ✅ (was 🟡), Spectacular Skywhale 🟡, Lorehold the
    Historian 🟡, Homesickness 🟡, Fractalize 🟡, Divergent Equation 🟡,
    Rubble Rouser 🟡, Zimone's Experiment 🟡, Petrified Hamlet 🟡.
    Postmortem Professor stays 🟡 but the printed "this creature
    can't block" static is now wired via `Keyword::CantBlock`.

- ✅ **SOS push VI (2026-05-01)**: 12 new SOS cards (4 ✅, 8 🟡) plus
  Topiary Lecturer rewrite + 5 false-negative cleanups, with three
  new engine primitives:
  - **`TokenDefinition.triggered_abilities`** + plumbing through
    `token_to_card_definition`. Promotes Send in the Pest, Pestbrood
    Sloth, Pest Summoning, Tend the Pests, Hunt for Specimens — the
    Pest tokens those spells mint now correctly carry their printed
    "die / attack → gain 1 life" rider. Added `stx_pest_token()`
    helper in `catalog::sets::stx::shared` for the death-trigger
    Witherbloom Pests.
  - **`ManaPayload::OfColor(Color, Value)`** — fixed-color, value-
    scaled mana adder. Single AddMana call, no player choice. Powers
    Topiary Lecturer's "{T}: Add G equal to power" cleanly (was a
    `Repeat × Colors([Green])` approximation).
  - **`Keyword::CantBlock`** — first-class "this creature can't block"
    keyword. Enforced inside `declare_blockers`, `can_block_any_attacker`,
    and `blocker_can_block_attacker`. Used by Duel Tactics's transient
    grant; Postmortem Professor's static restriction can be promoted
    to use it.
  - **`move_card_to` library traversal** — `Effect::Move` from a
    `Selector::TopOfLibrary` source now actually moves the top library
    card (previously the library branch was missing in `move_card_to`,
    so Suspend Aggression's exile-top-of-library half no-op'd). The
    library-source move is last in the search order to avoid
    accidentally consuming a hand card with the same id.
  - **Auto-target picker improvement**: friendly pumps (Magecraft /
    Repartee +1/+1 fan-out, transient PumpPT spells) now prefer the
    highest-power friendly creature, not the first-in-Vec match. This
    correctly aims Hardened Academic's CardLeftGraveyard counter at
    the biggest threat instead of the first 1-drop. Hostile picks
    still use first-match.
  - 12 new tests in `tests::sos::*`. All 870 lib tests pass.
  - Cards: Snarl Song ✅, Wild Hypothesis ✅, Send in the Pest ✅,
    Pestbrood Sloth ✅, Daydream 🟡, Soaring Stoneglider 🟡, Tome
    Blast 🟡, Duel Tactics 🟡, Ark of Hunger 🟡, Suspend Aggression
    🟡, Wilt in the Heat 🟡, Practiced Scrollsmith 🟡, Topiary
    Lecturer (rewrite, kept 🟡 — Increment rider still missing).
  - 5 false-negative status cleanups (the cards were already wired
    but the doc still said ⏳): Hydro-Channeler, Geometer's
    Arthropod, Sundering Archaic, Transcendent Archaic, Ulna Alley
    Shopkeep — all 🟡.

- ✅ **SOS push V (2026-04-30)**: 12 new SOS cards (3 ✅, 9 🟡) plus
  three new engine primitives that unblock Lorehold "cards leave your
  graveyard" payoffs and proper fight resolution:
  - **`EventKind::CardLeftGraveyard`** + `GameEvent::CardLeftGraveyard`
    — fires per card removed from a graveyard (return-to-hand,
    flashback cast, persist/undying battlefield-return, exile-from-gy).
    Plumbed in `move_card_to`'s graveyard branch, `cast_spell_flashback`
    in actions.rs, and persist/undying returns in stack.rs. Each
    emission also bumps the new
    `Player.cards_left_graveyard_this_turn` tally (reset on
    `do_untap`), surfaced through `PlayerView` for client UIs.
  - **`Predicate::CardsLeftGraveyardThisTurnAtLeast`** — gates Lorehold
    "if a card left your graveyard this turn" payoffs (Living
    History's combat trigger; Primary Research's end-step draw and
    Wilt in the Heat's cost reduction will use the same predicate).
  - **`Predicate::SpellsCastThisTurnAtLeast`** — gates Burrog
    Barrage's "if you've cast another instant or sorcery this turn"
    pump.
  - **`Effect::Fight { attacker, defender }`** — proper bidirectional
    fight primitive. Snapshots both creatures' powers up-front; no-ops
    cleanly when either selector resolves to no permanent. Unblocks
    Chelonian Tackle's "fight up to one opp creature" (single-target
    collapse on the defender pick), and is ready for Decisive Denial
    mode 1 + future fight-style cards.
  - **`Effect::Untap.up_to: Option<Value>`** — untap-with-cap. Frantic
    Search's "untap up to three lands" now honors the printed cap
    precisely (was "untap all"). Other Untap callers opt-out via
    `up_to: None`.
  - 13 new tests in `tests::sos::*` + 1 in `tests::modern::*`. All 857
    lib tests pass.
  - Cards: Hardened Academic ✅, Spirit Mascot ✅, Garrison Excavator ✅,
    Living History 🟡, Witherbloom the Balancer 🟡, Burrog Barrage 🟡,
    Chelonian Tackle 🟡, Rabid Attack 🟡, Practiced Offense 🟡, Mana
    Sculpt 🟡, Tablet of Discovery 🟡, Steal the Show 🟡.

- ✅ **modern_decks post-push III batch (2026-04-30)**: 10 SOS cards
  (5 ✅, 5 🟡) plus 5 new engine primitives:
  - **`Value::Pow2(Box<Value>)`** — 2ˣ with the exponent capped at
    30. Powers Mathemagics's "draw 2ˣ cards".
  - **`Value::HalfDown(Box<Value>)`** — half of a value, rounded
    down. Powers Pox Plague's "loses half / discards half / sacs
    half" three-stage effect.
  - **`Value::PermanentCountControlledBy(PlayerRef)`** — counts
    permanents controlled by the resolved player. Lets per-player
    iteration in `ForEach Selector::Player(EachPlayer)` correctly
    compute the iterated player's permanent count instead of always
    reading `ctx.controller`'s board.
  - **`Selector::CastSpellTarget(u8)`** — resolves the chosen target
    slot of the spell whose `SpellCast` event produced the current
    trigger. Walks the stack for the matching spell. Used by
    Conciliator's Duelist's Repartee body to exile the cast spell's
    chosen creature target.
  - **`AffectedPermanents::AllWithCounter { controller, card_types,
    counter, at_least }`** — counter-filtered lord-style statics.
    `affected_from_requirement` recognises `SelectionRequirement::
    WithCounter(...)` in the static's selector and routes through the
    new variant. Powers Emil's "creatures with +1/+1 counters have
    trample" + future "monstrous / leveled creatures gain
    [keyword]" buffs.
  - 12 new tests in `tests::sos::*`. Cards: Mathemagics ✅, Visionary's
    Dance ✅, Pox Plague ✅, Emil Vastlands Roamer ✅, Orysa ✅
    (post-push III), Conciliator's Duelist 🟡 (Repartee exile half
    promoted), Abstract Paintmage 🟡, Matterbending Mage 🟡,
    Exhibition Tidecaller 🟡, Colossus of the Blood Age 🟡. All 851
    lib tests pass.

- ✅ **SOS push III + Multicolored predicate (2026-04-30)**: 13 new SOS
  card factories (4 fully ✅, 9 body-only 🟡) plus engine wins:
  - **`SelectionRequirement::Multicolored`** + **`Colorless`** —
    counts distinct colored pips in a card's cost (hybrid counts both
    sides; Phyrexian counts the colored side). Unblocks Mage Tower
    Referee's "whenever you cast a multicolored spell" trigger.
  - **`Effect::Move` from library** — `move_card_to` now walks each
    player's library when locating the source card, so a `Selector::
    TopOfLibrary { count } → ZoneDest::Exile` move actually exiles the
    top card. Suspend Aggression uses this; Daydream / Practiced
    Scrollsmith and other "exile top of library, then …" cards get
    library-source moves for free.
  - 14 new tests in `tests::sos::*`. All 838 lib tests pass.
  - Cards: Mage Tower Referee ✅, Transcendent Archaic ✅, Snarl Song ✅,
    Poisoner's Apprentice ✅, Sundering Archaic 🟡, Hydro-Channeler 🟡,
    Ulna Alley Shopkeep 🟡, Topiary Lecturer 🟡, Garrison Excavator 🟡,
    Spirit Mascot 🟡, Geometer's Arthropod 🟡, Suspend Aggression 🟡,
    Living History 🟡.

- ✅ **SOS body-only batch (2026-04-30)**: 13 SOS creatures previously
  marked ⏳ are now 🟡 with their printed cost / type / P/T / keywords
  correct. Cards are usable in cube color pools and combat; their
  Increment / Opus / mana-spent-pump riders are omitted pending the
  "mana-paid-on-cast introspection" engine primitive (see Engine —
  Missing Mechanics below). Plus Ajani's Response shipped with destroy
  but no cost-reduction. New `CreatureType::Dwarf` added for
  Thunderdrum Soloist. 11 functionality tests in `tests::sos::*`. All
  822 lib tests pass.

- ✅ **Auto-target source-avoidance (2026-04-30)**: triggered abilities
  now skip the trigger source as a target candidate when another legal
  target is available. New `auto_target_for_effect_avoiding(eff,
  controller, avoid_source)` API; all trigger-creation paths updated
  (ETB, combat, dies/leaves, delayed). Quandrix Apprentice's Magecraft
  pump now deterministically prefers a non-source creature; falls back
  to the source when it's the only legal pick. 2 new tests in
  `tests::stx::*`.

- ✅ **SOS expansion II (2026-04-30)**: 11 more cards bridging the
  Silverquill (W/B) and Lorehold (R/W) schools, plus a handful of
  cross-school staples and mono-color removal/utility.
  - Silverquill: Moment of Reckoning (modal destroy/return), Stirring
    Honormancer (look-at-X-find-creature via `RevealUntilFind`),
    Conciliator's Duelist (ETB body wired; Repartee exile-with-return
    is omitted).
  - Lorehold: Lorehold Charm (all 3 modes), Borrowed Knowledge (mode 0
    faithful, mode 1 collapsed to "draw 7").
  - Witherbloom: Vicious Rivalry (X-life cost approximation +
    `ForEach.If(ManaValueOf ≤ X) → Destroy`).
  - Quandrix: Proctor's Gaze (bounce + Search basic to bf tapped).
  - Mono-color staples: Dissection Practice ({B} drain+shrink), End of
    the Hunt ({1}{B} exile opp creature/PW), Heated Argument ({4}{R} 6
    + 2-to-controller), Planar Engineering ({3}{G} sac 2 lands +
    Repeat×4 fetch basics).
  - 11 functionality tests in `tests::sos::*`. All 807 lib tests pass.
  - Cube cross-pool pools updated for W/B, B/G, G/U, R/W; mono-color
    pools (Black, Red, Green) picked up the new mono-color cards.

- ✅ **SOS expansion (2026-04-30)**: 10 new / improved cards.
  - Graduation Day ({W} Repartee enchantment) — new.
  - Stirring Hopesinger / Informed Inkwright / Inkling Mascot /
    Snooping Page — Repartee riders fully wired (was 🟡, now ✅).
  - Withering Curse ({1}{B}{B}) — Infusion-gated mass debuff/wrath.
  - Root Manipulation ({3}{B}{G}) — pump + menace fan-out (🟡:
    on-attack rider stubbed pending transient-trigger-grant primitive).
  - Blech, Loafing Pest ({1}{B}{G}) — lifegain-multi-tribe pump.
  - Cauldron of Essence ({1}{B}{G}) — death drain + sac-reanimation.
  - Diary of Dreams + Potioner's Trove (colorless artifacts, 🟡 with
    minor caveats noted in STRIXHAVEN2.md).
  - Spectacle Summit (Prismari U/R school land).
  - 13 new tests in `tests::sos::*`.
  - Cube color pools refreshed: Witherbloom (B/G), Silverquill (W/B),
    Prismari (U/R) cross-pools each picked up the relevant cards.
- ✅ **`scripts/gen_strixhaven2.py`** — oracle text is no longer
  truncated. Earlier revisions cut to 220 chars (then 600); both
  silently dropped late keywords (Flashback, Crew, Prepare reminder
  text). The script now passes the full oracle through unmodified.
  All STRIXHAVEN2.md rows whose oracle was previously clipped were
  marked **🔍 needs review (oracle previously truncated)** so future
  card-implementation passes know to cross-check the body before
  authoring against the row's existing notes (52 rows tagged).
- ✅ **STX schools expanded**: new modules under `catalog::sets::stx` for
  Lorehold, Quandrix, and Prismari. 11 new STX cards across the four
  colleges (Lorehold Apprentice/Pledgemage, Pillardrop Rescuer, Heated
  Debate, Storm-Kiln Artist, Quandrix Apprentice/Pledgemage, Decisive
  Denial, Prismari Pledgemage/Apprentice, Symmetry Sage) plus
  Witherbloom Pledgemage. Pest Summoning bumped from 1 → 2 tokens to
  match the printed Oracle. 13 new functionality tests.
- ✅ **`scripts/gen_strixhaven2.py` parsing fixes**:
  - Oracle truncation cap raised 220 → 600 chars (was clipping the
    bodies of cards with reminder-text-laden modes — including the
    Prepare keyword's definition on its grantor cards).
  - Recognises new SOS-only mechanics (Repartee, Magecraft, Increment,
    Opus, Infusion, Paradigm, Converge, Casualty, Prepare) as needing
    engine primitives, so the per-card hint column now points at the
    right plumbing.
  - Added a "Prepare mechanic" explainer to STRIXHAVEN2.md and a TODO
    item for the per-permanent prepared flag + setter primitive.
- ✅ `once_per_turn` flag on activated abilities is now enforced engine-side
  (was a struct field with no validation). Cards: Mindful Biomancer, etc.
- ✅ Strixhaven creature/spell subtypes added: Inkling, Pest, Fractal, Orc,
  Warlock, Bard, Sorcerer, Pilot, Elk.
- ✅ SOS catalog scaffolded under `catalog::sets::sos` with 51+ card
  factories wired into the cube color pools (white, blue, black, red,
  green, plus W/B Silverquill, B/G Witherbloom, G/U Quandrix, U/R
  Prismari, R/W Lorehold cross-pools).
- ✅ `Player.life_gained_this_turn` tally added (with `Effect::GainLife`,
  `Effect::Drain`-recipient, and combat-lifelink integration). Cleared on
  `do_untap`. Surfaced through `PlayerView` for client UIs.
- ✅ `Predicate::LifeGainedThisTurnAtLeast { who, at_least }` for "if you
  gained life this turn" Infusion riders (Foolish Fate, Old-Growth
  Educator, Efflorescence wired so far).
- ✅ `PlayerRef::OwnerOf(Selector)` / `ControllerOf(Selector)` now fall
  back through graveyards / hands / library / exile when the target has
  already changed zones (typical case: destroy-then-drain-controller),
  via the new `GameState::find_card_owner` helper.
- ✅ **`StackItem::Trigger.x_value`** — ETB triggers fired off a
  resolving spell now inherit that spell's paid X. `Effect::AddCounter
  { amount: Value::XFromCost }` and similar X-driven effects on
  creature/permanent ETBs read the correct X (Pterafractyl, Static
  Prison). `ResumeContext::Trigger` carries the same `x_value` so a
  suspended trigger resumes with the right X.
- ✅ **`Selector::LastCreatedToken`** + **`Value::CardsDrawnThisTurn`**
  + **`Player.cards_drawn_this_turn`**. `Effect::CreateToken` stashes
  the freshly-minted token id on the game state so a follow-up
  `AddCounter` / `PumpPT` in the same `Effect::Seq` can target it via
  `Selector::LastCreatedToken`. Combined with `Player.draw_top()`
  incrementing `cards_drawn_this_turn` (reset on the controller's
  untap), the new primitives unblock Quandrix scaling (Fractal Anomaly
  is now ✅).
- ✅ **`ClientView.exile`** + **`ExileCardView`**. The shared exile
  zone now projects through the per-seat view so a client UI can
  render an exile browser. Each entry carries the card's owner so the
  UI can distinguish "exiled by you" from "exiled from your library".
- ✅ **`PlayerView.cards_drawn_this_turn`**. Surfaced for client UIs
  to preview Quandrix scaling on cards in hand.
- ✅ **STX (Strixhaven base set) module** under `catalog::sets::stx`,
  parallel to the existing SOS module. 14 cards across Silverquill,
  Witherbloom, and shared (Inkling Summoning / Tend the Pests). 15
  functionality tests, all passing. See `STRIXHAVEN2.md` ("Strixhaven
  base set (STX)" section).
- ✅ **`effect::shortcut::magecraft(effect)` helper** + supporting
  `cast_is_instant_or_sorcery()` predicate. Lets a Magecraft trigger
  drop into a card factory in one line instead of seven. Used by
  Eager First-Year and Witherbloom Apprentice.
- ✅ **12 stale-test fixes** — Devourer of Destiny re-cost (5→7), plus
  Biorhythm/Holy Light/Loran/Path of Peace/Read the Tides cost drift,
  Lumra keyword (Reach→Trample), and a cube-prefetch test that lost
  several no-longer-pooled card names. All 736 → 751 tests now pass.

---

## Engine — Missing Mechanics

### Replacement Effects
The engine has no replacement-effect primitive.  Many real cards need one:
- ETB replacements (Containment Priest, Torpor Orb, Rest in Peace)
- Damage replacements (protection, preventing damage)
- Draw replacements (Leyline of the Void)
- Death replacements (Kalitas, Oubliette)
Until this lands, cards with "instead" clauses are either stubbed or collapsed
into a close approximation.

### Cast-From-Exile Pipeline
Many cards exile a spell/card temporarily and later cast it (Foretell,
Suspend, Rebound, Flashback-from-exile, Escape, Adventure second cast,
Cascade resolution).  Currently each is handled ad-hoc or omitted.  A shared
"cast from alternate zone" code path would unlock dozens of cards.

### Copy Primitive
**Spell copy**: ✅ DONE in push XXI. `Effect::CopySpell { what, count }`
is now first-class, finding the matching `StackItem::Spell` on the
stack and pushing `count` copies with `is_copy: true`. Powers
Choreographed Sparks, Aziza, Mica, Silverquill the Disputant
(Casualty 1), Lumaret's Favor (Infusion), Social Snub. Plus
`Selector::CastSpellSource` for "copy that spell" semantics inside
SpellCast triggers. See `STRIXHAVEN2.md` push XXI.

**Permanent copy** (rule 707.10b — copy of a permanent spell becomes
a token): ⏳ still todo. Needed for Echocasting Symposium, Applied
Geometry, Saheeli Rai −3. The current `CopySpell` no-ops on permanent
spells. A follow-up `Effect::CopyPermanent { what, count }` plus a
"copy → token" minting path would close this.

**Permanent activation copy** (Strionic Resonator's "copy that
ability"): ⏳ still todo. Needs an analogous `Effect::CopyTrigger`
that copies a `StackItem::Trigger` instead of a Spell.

**New-targets prompt**: copies inherit their original's target slot
today. The printed "you may choose new targets for the copy" prompt
is collapsed — closing this needs a target-prompt step on the new
copy before resolution.

### Triggered-Ability Event Gaps
`EventKind` is missing several commonly-needed triggers:
- `PermanentLeftBattlefield(CardId)` — needed for "LTB" abilities and
  exile-until-LTB patterns (Tidehollow Sculler, Fiend Hunter)
- `DamageDealtToCreature` — needed for enrage, lifelink gain on creature damage
- `TokenCreated` — needed for populate, alliance triggers
- `CounterAdded / CounterRemoved` — needed for proliferate payoffs, Heliod combo
- `SpellCopied` — storm payoffs, Bonus Round
- `PlayerAttackedWith` — needed for Battalion and similar attack-count effects
- ~~`SpellCastTargetingCreature` (or a `Predicate::SpellTargetsCreature`
  knob) — needed for Strixhaven Repartee.~~ **Done**: see
  `Predicate::CastSpellTargetsMatch` + `effect::shortcut::repartee()`.
  Stirring Hopesinger, Rehearsed Debater, Informed Inkwright, Inkling
  Mascot, Snooping Page, Lecturing Scornmage, Melancholic Poet, and
  Graduation Day all use it. Remaining Repartee cards are blocked on
  separate primitives (Ward, exile-until-X, copy-spell).
- ~~`CardLeftGraveyard` — needed for Lorehold "cards leave your
  graveyard" payoffs.~~ **Done** in push V: see
  `EventKind::CardLeftGraveyard` + `Predicate::CardsLeftGraveyardThisTurnAtLeast`.
  Hardened Academic, Spirit Mascot, Garrison Excavator, Living
  History all wired. Remaining gy-leave-aware cards (Ark of Hunger,
  Owlin Historian, Primary Research, Wilt in the Heat) need only
  catalog wiring against the event.

### Multi-Card Batch Triggers
The engine emits `CardLeftGraveyard` per card removed; printed cards
say "Whenever **one or more** cards leave your graveyard". We
approximate by firing the trigger per-card (a strict power upgrade
on multi-card-removal turns, but harmless in 2-player play where
single-card returns dominate). A future refinement: collapse a
batch of `CardLeftGraveyard` events emitted in the same resolution
window into one trigger fire (similar to MTG's "looks back in time"
rule for batch triggers). Same shape applies to `CardDiscarded`,
`CreatureDied`, and any future per-zone-move event.

### Spell-Side Predicate: Mana-Spent-On-Cast
SOS introduces **Increment** ("if mana spent > this creature's P or T,
+1/+1 counter") and **Opus** ("Whenever you cast an instant or sorcery,
do X. If five or more mana was spent, do bigger X"). Both need a
per-cast "mana value paid" snapshot exposed as a `Value` (or a
`Predicate::ManaSpentAtLeast(n)`). The engine already retains the cost
on the `StackItem`; lifting that into the `EffectContext` for trigger
filters should unlock a few dozen Strixhaven cards.

### X-Cost and Converge
`Value::XFromCost` exists but converge (number of *distinct colors* of mana
spent) is not tracked per cast.  `Value::ConvergedValue` is a stub that always
returns 0 for non-Prismatic-Ending uses.  Fix: record color set paid at cast
time and expose it as a `Value` primitive.

### Cost-Reduction Stacking
Delve, Improvise, Convoke, and generic cost-reducers each have separate
branches.  There is no unified "reduce mana cost by X before payment" hook,
making cards like Hogaak (Convoke + Delve) or Affinity impossible to express
cleanly.

### Target-Aware Cost Reduction
"This spell costs {X} less to cast if it targets [some condition]" is a
Strixhaven design pattern (Ajani's Response, Brush Off, Run Behind,
Mavinda, Killian, Orysa). Today we either drop the discount and ship the
spell at its printed full cost, or omit the spell entirely. Engine fix:
let `CostReduction` static / per-card alt-cost evaluate against the
candidate-cast's chosen target before payment. Probably a new
`SelectionRequirement`-keyed cost discount that the cast path consults.

### Mana Ability from Non-Battlefield Zone
`activate_ability` only walks the battlefield.  Cards like Elvish Spirit Guide
and Simian Spirit Guide (exile from hand: add mana) are completely omitted
because hand-activated mana abilities need a separate activation path.

### Activated-Ability "From Your Graveyard" Path
The `activate_ability` walker only iterates the battlefield, so cards
with mana-cost-priced graveyard-recursion abilities (e.g. SOS Summoned
Dromedary's `{1}{W}: return this from graveyard, sorcery speed`,
Teacher's Pest's `{B}{G}: return tapped`, Postmortem Professor's exile-
an-IS-card-from-gy:return) currently drop the activation entirely. The
`FromYourGraveyard` event-scope path supports *triggered* recursion
(Bloodghast, Silversmote Ghoul) but not activated. Adding a parallel
graveyard walk in `activate_ability` would unlock five+ SOS cards.

### "Look At Top X, Pick One, Put Rest in Graveyard" Primitive
Stirring Honormancer ("look at top X cards where X is creatures you
control, put one in hand, rest into graveyard") and similar look-and-
sort effects need a "look at top N, choose K, mill the rest" primitive
to express faithfully. `Effect::Surveil` covers the "look + may put in
graveyard" shape but with a fixed number; the SOS variant is dynamic
and forces the rest-to-graveyard branch unconditionally.

### Choice of "Which Zone" for a Tutor Result
Dina's Guidance ("search a creature, put into hand or graveyard")
exposes a 2-option destination prompt that no other primitive currently
needs. Adding a `Effect::Search` flavor with `to: Either(ZoneDest,
ZoneDest)` (or a separate decision shape) would honor the toggle for
this and a handful of black/green search effects.

### "May" Optionality Inside Sequences ✅ DONE
~~Several SOS cards bake a "you may" into the middle of a `Seq` (Pursue
the Past's "you may discard a card", Witherbloom Charm's mode 0 "you
may sacrifice a permanent", Practiced Offense's "may double-strike or
lifelink"). The engine has no "ask the controller yes/no" primitive,
so all of these collapse the optional branch into either always-do or
always-skip. A `Effect::MayDo(inner)` that emits a yes/no decision
(answered immediately by `AutoDecider`'s heuristic) would unblock a
chunk of cards without surfacing a new UI affordance.~~ Done in push
XV: `Effect::MayDo { description: String, body: Box<Effect> }` is now
first-class. Emits `Decision::OptionalTrigger`, AutoDecider answers
`false` by default, ScriptedDecider can flip to `true` for tests.
Promoted: Stadium Tidalmage, Pursue the Past, Witherbloom Charm mode
0, Heated Argument, Rubble Rouser. Practiced Offense's choice-mode
("double-strike or lifelink") still ⏳ since that's a 2-option pick,
not a yes/no.

### Multi-Target Prompt for Sorceries / Instants
A handful of SOS cards specify two target slots with different filters
(Render Speechless: opponent + creature; Cost of Brilliance: player +
creature; Homesickness: player + up to two creatures). The engine
today only exposes a single-target slot per spell at cast time, so
these collapse one of the two halves. A multi-target cast prompt
(`Vec<Target>` in `GameAction::CastSpell`) would unlock all of them.

### Auto-Target Picker: Source-Avoidance + Best-Pick Heuristics
~~The current `auto_target_for_effect` walks the battlefield in `Vec`
order and returns the first legal match.~~ **Source-avoidance done**:
the new `auto_target_for_effect_avoiding(eff, controller, avoid_source)`
takes the trigger source and prefers any *other* legal target,
falling back to the source only when nothing else is legal. All
trigger-creation paths (`stack.rs`'s `flush_pending_triggers`,
`actions.rs`'s ETB triggers, `combat.rs`'s combat triggers, the
delayed-trigger fire path, Dies/PermanentLeavesBattlefield triggers)
now pass the source ID. Quandrix Apprentice's Magecraft pump now
deterministically targets the bear over the Apprentice, and the test
suite asserts the source-fallback when no other target is legal.

~~Prefer the highest-power creature for friendly pumps.~~ **Done** in
push VI: `auto_target_for_effect_avoiding` now sorts the primary-player
candidate set by descending current power when the effect prefers a
friendly target (Magecraft / Repartee fan-outs, transient PumpPT
spells). Hostile picks still use first-match.

Remaining best-pick heuristics still ⏳:
- Prefer creatures whose current power matches what the pump would
  unlock (lethal swing, post-pump unblockable, etc.).

### Mana-Cost Reduction with Target Predicate
Killian, Ink Duelist's "spells you cast that target a creature cost
{2} less" needs a `StaticEffect::CostReduction` variant whose filter
inspects the cast spell's targets. Today's `CostReduction` filters
on the spell card's own attributes only. Plumbing the cast-time
target list into the cost-reduction site would unlock this card and
similar Lorehold/Witherbloom cost-cutters.

### "May Pay" Optionality on Death/ETB Triggers
Bayou Groff ("may pay {1} to return to hand on death") and several
Strixhaven cards bake an optional cost into a triggered effect
("may pay X: do Y"). The current engine has no `Effect::MayPay {
cost, then }` primitive — neither for life nor mana costs — so all
these collapse to either "always do" or "always skip". A decision-
generating `Effect::MayPay` would unblock a chunk of cards across
SOS Witherbloom and STX Lorehold without surfacing new UI affordances
beyond a yes/no prompt.

### Transient Triggered-Ability Grants on Pump Spells
SOS Root Manipulation ("Until end of turn, creatures you control get
+2/+2 and gain menace and 'Whenever this creature attacks, you gain
1 life.'") needs a way to attach a *triggered* ability to a creature
for a duration, on top of the keyword-grant primitive. Today the engine
has `Effect::GrantKeyword { what, keyword, duration }` but no
`Effect::GrantTriggeredAbility { what, ability, duration }`. Adding
this would unlock the third clause of Root Manipulation, similar
"creatures gain combat-damage trigger until EOT" pump spells, and
the on-attack rider on tokens (Pest token's "gain 1 on attack",
Spirit token combat triggers).

### Per-Turn-Cast Gate on Activated Abilities ✅ DONE
~~SOS Potioner's Trove ("{T}: You gain 2 life. Activate only if you've
cast an instant or sorcery spell this turn.") needs an
`ActivatedAbility::condition: Predicate` field (or a sibling
`gated_when: Option<Predicate>`) to express "activate only if you
played a spell of type X this turn".~~ Done in push VIII:
`ActivatedAbility.condition: Option<Predicate>` is now first-class.
Evaluated against the controller/source context before any cost is
paid (failed gate doesn't burn tap-cost or once-per-turn budget).
Promoted Potioner's Trove (gate: `SpellsCastThisTurnAtLeast(You, 1)`,
an approximation of the printed "instant or sorcery"-only filter) and
Resonating Lute (gate: `ValueAtLeast(HandSizeOf(You), 7)`). New
`GameError::AbilityConditionNotMet`. The remaining gap is a
per-spell-type tally that distinguishes IS casts from creature casts —
once that lands, Potioner's Trove can swap from
`SpellsCastThisTurnAtLeast` to the exact predicate.

### Self-Counter-Scaled Cost Reduction
SOS Diary of Dreams's `{5},{T}: Draw a card` activation costs `{1}`
less per page counter on the source. There's no
`StaticEffect::CostReduction` variant whose discount scales off the
source's own counter count. Adding a `CostReduction { delta:
Value::CountersOn { what: Selector::This, kind: Charge } }` shape
would unlock Diary of Dreams cleanly, plus other counter-scaled cost
reducers (M21 Mazemind Tome).

### Page Counter Type
SOS Diary of Dreams (and the rest of the SOS book/grandeur subtheme)
references "page counter" but the engine `CounterType` enum has no
`Page` variant. Diary is currently approximated with `CounterType::
Charge`, which is fine in 2-player play (no other card uses Charge as
a payoff source) but obscures the printed identity. Adding `Page`,
`Knowledge`, and the small handful of other novelty counters from
recent sets would close the gap.

### `Move`-with-count for Selecting One Card from a Zone
Today `Effect::Move { what: Selector::CardsInZone { zone: Graveyard, ... } }`
moves *every* matching card. Cards like Heated Argument's "you may
exile a card from your graveyard" need a "move at most one matching
card" primitive. A `Selector::OneOf(inner)` wrapper, or a `count` knob
on `CardsInZone`, would fix this. The current workaround for Heated
Argument collapses the optionality into "always do the rider".

### "Choose Up To N Modes (with Repetition)" for `ChooseMode`
Strixhaven's "Choose up to four. You may choose the same mode more
than once." pattern (Moment of Reckoning, Witherbloom Charm-style
spells with N copies) needs an extension on `Effect::ChooseMode` that
takes a list of (index, target) tuples per cast. Today the engine's
modal flow picks exactly one mode and one target per cast — the
"choose up to N" wrappers collapse to single-mode resolution.

### "X Life as Additional Cost" Primitive
Vicious Rivalry, Fix What's Broken, and a handful of SOS sorceries
have "As an additional cost to cast this spell, pay X life." The
engine has no per-cast life-payment cost — we approximate by reading
X from the spell's `{X}` slot and running `LoseLife X` at resolution
time, but that double-counts X (paying X mana via XFromCost AND X
life). A `cost.life: Value` field on `CardDefinition` (or an
`alternative_cost` variant whose payment also requires the life)
would make this faithful.

### "Track Cards Discarded by This Effect" Counter ✅ DONE
~~Borrowed Knowledge ("draw cards equal to the number of cards
discarded this way") needs a per-resolution counter that
`Effect::Discard` increments. The mode 1 path is currently
approximated as "draw 7" — a flat-7 reload that misses the printed
"draw exactly as many as you discarded" precision but preserves the
card-advantage tally for typical hand sizes.~~ Done in push XVII:
`Value::CardsDiscardedThisResolution` + sibling
`Selector::DiscardedThisResolution(SelectionRequirement)` are now
first-class. Backed by `GameState.cards_discarded_this_resolution`
(u32) + `cards_discarded_this_resolution_ids` (Vec<CardId>); both
reset on every `resolve_effect` entry. Promoted: Borrowed Knowledge
mode 1, Colossus of the Blood Age death rider, Mind Roots's "land
discarded → bf tapped" half.

### Capture-As-Target From Selector (Repartee Exile-Until-End-Step)
Conciliator's Duelist's Repartee body wants to:
1. Exile the cast spell's chosen creature target
   (`Selector::CastSpellTarget(0)` — wired).
2. Schedule a delayed trigger that returns *the exiled card* to
   battlefield at next end step.

Step (2) collides with `Effect::DelayUntil`'s capture model — it
captures `ctx.targets.first()`, but a Repartee trigger has no
target slot of its own (the selector is what tracks the spell's
target). Need either:
- An `Effect::CaptureTargetFromSelector { slot, selector }` that
  mutates ctx.targets so the subsequent DelayUntil reads it back, OR
- An `Effect::ExileWithDelayedReturn { what, kind, controller }`
  combinator that pre-resolves the selector at registration time.

The latter is more general (also unblocks Tidehollow Sculler,
Banisher Priest, Fiend Hunter). The former is smaller surface but
introduces effect-side mutation of ctx.

### "Untap Up To N" Cap ✅ DONE
~~`Effect::Untap` with a selector untaps *all* matching permanents.~~
Done in push V: `Effect::Untap` now carries an `up_to: Option<Value>`
field. Frantic Search caps at 3 lands; other Untap callers opt-out
with `up_to: None`. The picker takes the first N matching in
resolution order — a future enhancement could add a "highest-CMC
first" heuristic for max mana refund.

### Spend-Restricted Mana
Strixhaven's "Spend this mana only to cast an instant or sorcery
spell" (Hydro-Channeler, Tablet of Discovery's {T}: Add {R}{R}
ability, Abstract Paintmage's PreCombatMain trigger, Resonating
Lute's land-grant) needs per-pip metadata on the mana pool. Today
mana is fungible — once it's in the pool, anything can spend it.
Adding a `restriction: Option<SpellTypeFilter>` knob on each
ManaPool entry (and consuming it during cost-pay) would honor the
printed restriction. Wide-ranging change touching `ManaPool`,
`pay()`, and the cost-pay-validation path.

### "Move at most one matching card" — `Selector::OneOf`
Several SOS effects exile/move "a card" from a graveyard, hand, or
top of library where the count is at most 1 (Heated Argument's "may
exile a card from your graveyard", Practiced Scrollsmith's "exile
target noncreature/nonland card from your graveyard"). Today
`Selector::CardsInZone { ... }` returns ALL matching cards. Adding
`Selector::OneOf(Box<Selector>)` (or a `count` knob on `CardsInZone`)
would let these spells correctly pick exactly one. Without it, the
catalog approximates by "exile every matching card" which over-
shoots when the graveyard has multiple matches.

### Snow Mana Validation
`ManaPool` tracks a `snow` counter but `pay()` never validates that a `Snow`
mana symbol must be paid from a snow source.  Any mana from any land currently
satisfies a `{S}` pip.

### Multiplayer / Commander Format
- Command zone: `Zone::Command` exists but `ClientView` has no field for it;
  the server never moves cards there.
- Commander damage tracking (21 from the same commander = loss).
- "Your opponents" vs. "each other player" distinctions (multiplayer targeting
  semantics differ from 2-player).
- Four-player free-for-all match setup in `run_match` / `build_cube_state`.
- Commander-specific rules: color identity deck building, commander tax.

### Planeswalker Interactions
- Planeswalkers can be attacked directly — `AttackTarget::Planeswalker` is in
  `types.rs` but the bot never chooses it and the client has no UI for it.
- "Planeswalker redirect" rule (damage that would be dealt to a player can be
  redirected) is unimplemented.
- Emblems are not modelled.

### Saga Lore Counters
Sagas need: ETB with 1 lore counter, trigger each chapter, advance at upkeep,
sacrifice when the last chapter triggers.  No `SagaLore` counter type or
upkeep-advance primitive exists.

### Prepare Mechanic (SOS)
Secrets of Strixhaven splits Prepare into two halves; only the flag
side is missing.

**Half 1 — Prepared cards (the spell side).** Already wired. A
"prepared card" is a creature with a back-face *prepare spell* (e.g.
Spellbook Seeker // Careful Study, Pigment Wrangler // Striking
Palette). The pair rides the engine's existing MDFC plumbing
(`back_face: Some(...)` + `GameAction::CastSpell` /
`CastSpellBack`). Catalog: `crabomination/src/catalog/sets/sos/mdfcs.rs`.
The client image prefetcher (`crabomination_client::scryfall`) handles
these via a 422/404 fallback in `download_card_image` — the front is
engine-invented so `face=back` returns 422; the back name is always a
real Scryfall printing on its own. See STRIXHAVEN2.md → "Prepare
mechanic" for the lookup contract.

**Half 2 — The prepared flag (⏳).** A per-permanent boolean toggled
by `becomes prepared` / `becomes unprepared` effects. Cards like
Biblioplex Tomekeeper and Skycoach Waypoint flip the flag; payoff
cards have a `Prepare {cost}` activated/triggered ability and
reminder text "(Only creatures with prepare spells can become
prepared.)" — note the gate references the spell-side from Half 1.
Engine needs:
- `PermanentFlag::Prepared` (or `CounterType::Prepared` count-1) on
  `Permanent`, surfaced through `PermanentView`.
- `Effect::SetPrepared { what, value: bool }`.
- `Predicate::IsPrepared` for prepare-payoff conditional clauses
  (combined with `back_face.is_some_and(is_prepare_spell)` for the
  full reminder-text gate).
- A short oracle-text helper that wires "Prepare {cost}: …" into a
  standard activated ability with `gate: IsPrepared`.

Until the flag-side primitives land, flag-toggling cards (Tomekeeper,
Waypoint, prepare-payoff activations) stay ⏳; spell-side prepared
cards are 🟡 today (back-face castable, no flag interaction yet).

### Vehicle / Crew
`CardType::Artifact` exists but there is no `CrewN` keyword or "becomes a
creature until end of turn" mechanism.  Vehicle subtype is in `ArtifactSubtype`
but nothing uses it.

### Proper Split-Damage Distribution
Effects like Pyrokinesis ("deals 4 damage divided as you choose among any
number of targets") are collapsed to a single-target 4-damage hit.  A
`DealDamageDivided { total, targets: Vec<Selector> }` effect would express
the real card.

### Affinity / Self-Permanent-Scaled Cost Reduction
Witherbloom, the Balancer's "Affinity for creatures (this spell costs
{1} less to cast for each creature you control)" needs a per-cast cost
reduction whose discount scales off the caster's permanent count.
`StaticEffect::CostReduction { filter, amount }` is a fixed amount
today. Generalising to `amount: Value::CountOf(Selector)` (or a sister
variant `AffinityCostReduction { filter, scaler: Selector }`) would
unlock Affinity for Artifacts (Modern Affinity / Cranial Plating-era
shells), Affinity for X (Strixhaven Witherbloom + future), and Awaken
the Woods-style "X = forests" payoff costs.

### Token-Side Triggered Abilities ✅ DONE
~~`TokenDefinition` has `activated_abilities` but not
`triggered_abilities`.~~ **Done** in push VI: `TokenDefinition` now
carries `triggered_abilities: Vec<TriggeredAbility>` and
`token_to_card_definition` copies them through.

Wired tokens:
- **SOS Pest token** (`catalog::sets::sos::sorceries::pest_token`):
  "Whenever this token attacks, you gain 1 life." Promotes Send in
  the Pest, Pestbrood Sloth, Cauldron of Essence (its reanimation
  output), and any future SOS Pest minter.
- **STX Pest token** (`catalog::sets::stx::shared::stx_pest_token`):
  "When this creature dies, you gain 1 life." Promotes Pest
  Summoning, Tend the Pests, Hunt for Specimens (and Eyetwitch's
  Pest body would use it if Eyetwitch were a Pest token rather than
  a creature).

The Pest token chain now correctly trickles 1 life per qualifying
event into Witherbloom payoffs (Pest Mascot's lifegain → +1/+1
counter on self, Blech's per-creature-type counter fan-out, Bogwater
Lumaret's per-creature-ETB drain).

### Exile Zone as Viewable State
Exile is a zone in the engine (`Zone::Exile`) and cards move there.
`ClientView.exile` now projects the shared exile zone with each card's
owner so the UI can render an exile browser (added with the
Strixhaven coverage push). Remaining gaps:
- The 3D client has no exile browser UI yet.
- Graveyard-order information is lost (cards are a flat Vec).

---

## Engine — Approximation Cleanups

| Card / Feature | Current Approximation | Correct Behaviour |
|---|---|---|
| Windfall | draws flat 7 | draw equal to most cards discarded |
| Dark Confidant | fixed 2 life loss | lose life = CMC of revealed card |
| Biorhythm | drain opponents to 0 | set each player's life to creature count |
| Coalition Relic | tap for 1 of any color | tap + charge counter → burst WUBRG |
| Fellwar Stone | tap for 1 of any color | tap for a color an opponent's land produces |
| Static Prison | ETB taps target | also suppresses untap while stun counters exist |
| Spectral Procession | {3}{W}{W}{W} | {2/W}{2/W}{2/W} hybrid (CMC 6) |
| Grim Lavamancer | {R}{T}: 2 damage | must exile 2 cards as additional cost |
| Ichorid | no graveyard gate | requires opponent to have a black creature in GY |
| Render Speechless | required creature target | optional second creature target |
| Dina's Guidance | always to hand | choice of hand or graveyard |
| Slime Against Humanity | counts your gy only | counts all gy + your exile per printed Oracle |

### Resolved approximations (push XXII, 2026-05-02)

- **Frantic Search** — `up_to: Some(Const(3))` cap precise (push V).
- **Rofellos, Llanowar Emissary** — `{T}: Add {G} per Forest you
  control` now scales via `ManaPayload::OfColor(Green,
  CountOf(Forest & ControlledByYou))`; flat-`{G}{G}` collapse removed.
- **Pursue the Past** — `Effect::MayDo` discard-then-draw branch wired
  faithfully (push XV).
- **Witherbloom Charm (mode 0)** — `Effect::MayDo` may-sac branch wired
  (push XV).
- **Dragon's Approach** — "if 4+ named copies in graveyard, search for
  a Dragon" tutor now wired via the new `SelectionRequirement::HasName`
  predicate (push XXII).

---

## Client — Visualization

### Counter Display
`PermanentView.counters` carries all counter types and counts, but there is no
in-world or HUD display.  Suggested: floating text labels above affected cards
showing `+1/+1 ×3`, `Lore: 2`, `Charge: 1`, `Poison: 3`, etc., using Bevy
`Text3d` or billboard sprites.

### Modified Power/Toughness Display
When a creature's P/T differs from its printed values (pump spells, counters,
static effects), the UI shows the base stats.  `PermanentView` exposes both
`power`/`toughness` (current) and `base_power`/`base_toughness` (printed).
Show current P/T on the card and dim or strike through the base if modified.

### Modified Loyalty Display
Planeswalkers show a static loyalty badge but it doesn't update as
`CounterType::Loyalty` changes in-game.  Wire the loyalty counter from
`PermanentView` to the badge text.

### Exile Zone Browser
Similar to the graveyard browser, an exile browser would let players inspect
exiled cards (Foretell staging area, Leyline victims, Imprint sources, etc.).

### Stun Counter Visualization
Static Prison and Rapier Wit add stun counters.  No indicator currently shows
that a permanent has a stun counter (i.e., won't untap next turn).  A small
badge or coloured ring on the card would communicate this clearly.

### Mana Pool HUD
During the player's main phase, their current mana pool is shown in the player
status text but as a compact string.  A pip-style display (coloured circles for
each mana symbol available) would be faster to read at a glance.

### Damage Overlays
When combat damage is assigned, show floating damage numbers rising off
affected creatures before SBA removes the dead ones.

### Card Tooltip with Full Oracle Text
Hovering over a card shows its Scryfall art via the peek popup, but not the
full rules text.  A tooltip panel (shown on hover or via a dedicated key)
displaying the oracle text would reduce the need to look cards up externally.

### Graveyard Order and Timestamps
The graveyard browser shows cards as a flat unordered list.  Preserving
insertion order (most recently added = top) matches player intuition and helps
with "top of graveyard" effects.

### Attacking / Blocking Arrow Polish
Gizmo arrows are drawn in `draw_blocking_gizmos.rs` and `draw_attacker_overlays.rs`.
Improvements:
- Colour-code arrows by blocked/unblocked status.
- Show combat damage assignment numbers on arrows.
- Animate arrows fading in/out on declare-attackers/blockers transitions.

### Token Labeling
Token cards in the 3D view use the Scryfall-fetched art path, which often
resolves to a generic back image.  A text overlay (name + P/T) on token cards
would disambiguate multiple different tokens on the battlefield.

---

## Client — UX

### Undo / Take-Back
A "request take-back" action the opponent can approve would reduce frustration
from misclicks, especially during the targeting flow.

### Keyboard Shortcut Reference
Add a `?` or `H` key that opens an in-game overlay listing all keyboard
shortcuts (A = attack all, Space/P = pass, E = end turn, N = next turn, etc.).

### Responsive Stack Display
The stack panel (bottom-center) is a fixed-width overlay.  On narrow windows
it can overlap the player panel.  Clamp its width to `min(420px, 40vw)` or
reposition it to the right sidebar.

### Per-Phase Auto-Stop Flags
Arena-style "stop at" checkboxes per phase (e.g., "always stop at opponent's
end step").  Currently the only fast-forward controls are End Turn (E) and
Next Turn (N).

### Deck Browser
A pre-game or in-game panel listing the full deck composition (name + count
for each unique card) would help players understand the randomly-assembled cube
deck they are playing.

---

## Bot / AI

### Instant-Speed Responses
The bot currently never responds to spells on the stack — it auto-passes
priority whenever it gets it during an opponent's turn.  A rule-based layer
that recognises "this creature is being targeted by removal, I have a
counterspell" would make the bot feel more like a real opponent.

### Sacrifice Prioritisation
~~When forced to sacrifice, the bot always picks the first eligible
permanent.~~ Now sorts candidates: **tokens first, then by lowest CMC,
then by lowest power**. This is enforced inside `Effect::Sacrifice` so
both Innocent-Blood-style edict flow and forced sacrifices from
activated abilities see the same ordering. Future improvements:
respect "you may sacrifice" optionality (skip when the cheapest
candidate is more valuable than the payoff).

### Planeswalker Targeting
~~The bot never attacks planeswalkers.~~ Push XXIII: bot now routes
attackers at opp planeswalkers when their power matches the walker's
loyalty (greedy first-fit accumulator).

### Smarter Blocking
~~Bot blocks randomly with all eligible blockers.~~ Push XXV: bot now
considers trades. Each (attacker, blocker) pair gets a `trade_score`
(killing the attacker is the dominant payoff, losing a body is the
cost), and a per-pressure-tier threshold gates assignment (lethal /
critical / normal). Net result: the bot stops suicide-blocking at
high life and properly chumps under lethal pressure. Future
improvements: deathtouch attacker handling (any block kills the
blocker, but every blocker contributes to killing the attacker — the
current logic counts deathtouch on the *blocker* but not on the
*attacker*'s "every blocker dies" implication when multiple blockers
team up); combat-trick anticipation (don't trade into Giant Growth);
chumping with a token over a real card.

### Smarter Mana Rock Usage
The bot taps mana rocks eagerly before knowing what it wants to cast.  A
"plan this turn's spending first" pass before mana-ability activation would
avoid situations where it taps a Sol Ring with nothing to cast.

### Multiple Difficulty Levels
- Easy: current random bot
- Medium: rule-based heuristics (responsive countering, threat assessment)
- Hard: Monte-Carlo tree search or minimax over the simplified game state

---

## Infrastructure / Dev

### Engine Test Coverage
Current test density is low outside `effects.rs` and card-specific unit tests.
Priority gaps:
- **Combat module** (`game/combat.rs`) has zero standalone tests.
- **Layer system** (`game/layers.rs`) — continuous effects, P/T ordering,
  timestamp tracking — has no dedicated tests.
- **Stack resolution ordering** — no tests for multi-item LIFO resolution,
  replacement effects, or trigger ordering.

### Snapshot Round-Trip Test
`GameSnapshot` and `GameState` serialisation exist.  Add a property-based test
that plays N random actions, serialises/deserialises the state, and asserts
game continuity — catching any `Serialize`/`Deserialize` drift.

### Card Correctness CI
`scripts/verify_cards.py` (with its Scryfall cache) verifies CMC, P/T, types,
and keywords.  Wire it as a CI step that runs against `scripts/.scryfall_cache.json`
(no network) to catch regressions when catalog entries change.

### Bot vs. Bot Simulation
Automate a "run 1 000 cube games bot vs. bot, report win rates by colour pair"
script.  Useful for catching degenerate card interactions and unbalanced pools
without manual play.

### Replay / Game Log Export
The server already collects `GameEventWire` events.  A replay file format
(sequence of `(action, resulting_state_hash)`) would enable post-game review
and deterministic bug reproduction.

### Scryfall Art Pre-fetch CLI
`all_cube_cards()` drives the in-game prefetch, but there is no standalone CLI
tool to warm the asset cache before a session.  A `cargo run --bin prefetch_art`
that downloads missing Scryfall images to the local cache would speed up first-
session load times.

### WASM / Web Build
`Cargo.toml` already has a `wasm-release` profile.  Completing the web build
(removing native-only dependencies, adding a WASM server bridge) would make
the game playable in a browser without installation.

---

## Formats

### Commander (1v1 or 4-player)
- 100-card singleton decks built around a legendary creature commander
- Command zone with commander-tax mechanic
- 40 starting life
- Commander damage loss condition
- Color-identity deck-construction enforcement
- Multiplayer turn order and attack direction

### Draft
- 8-player booster draft simulation
- Bot drafters with a basic pick-order heuristic
- Deck construction phase before play begins

### Sealed
- Generate 6 booster packs per player
- Deck construction phase
- Best-of-3 match support

### Brawl / Historic Brawl
- Lighter-weight commander variant (60-card, Standard-legal)
- Good stepping stone before full Commander

---

## Card Implementations (high-priority unblocked cards)

These cards are in the cube or demo decks and need only existing primitives —
no new engine features required:

| Card | Missing Piece | Effort |
|---|---|---|
| Grim Lavamancer | Exile-2-from-GY additional cost | Low |
| Bloodtithe Harvester | Sac-Blood ping (sac_cost activation) | Low |
| Dread Return | Flashback sac-3-creatures cost | Medium |
| Swan Song | Correct Bird token controller | Low |
| Frantic Search | Untap cap (up to 3) | Low |
| Windfall | Dynamic draw-equal-to-max-discarded | Medium |
| Balefire Dragon | Dynamic "that much damage" (use creature's power) | Medium |
| Dark Confidant | CMC-dependent life loss | High (needs card-CMC Value) |
| Rofellos | Forest-count mana scaling | Medium |
| Tidehollow Sculler | Exile-until-LTB primitive | High |
| Ichorid | Graveyard-color trigger filter | Medium |
| Coalition Relic | Charge-counter burst | Medium |
| Tezzeret, Cruel Captain | Artifact-creature static pump | Low |
| Karn, Scion of Urza | Artifact-count scaling Construct | Medium |

## New suggestions (added 2026-05-01 push VIII)

These items came up while implementing the push VIII batch and are
listed here so the next pass can pick them up without re-deriving them.

### Engine

- **X-cost activated abilities**. `ActivatedAbility.mana_cost` accepts
  `ManaSymbol::X` symbols today, but the activation entry point doesn't
  surface an X-value prompt (unlike `cast_spell`, which has
  `x_value: Option<u32>`). Berta, the Wise Extrapolator's `{X}, {T}:
  Create a Fractal token + X +1/+1 counters` is currently stubbed
  because X resolves to 0 at activation time. Adding an `x_value` arg
  to `GameAction::ActivateAbility` (and threading it through
  `Effect::AddCounter { amount: Value::XFromCost }`) would unblock
  Berta plus several X-cost utility activations across MTG history
  (Forerunner of the Empire-style scaling).

- **Per-spell-type per-turn tallies**. `Player.spells_cast_this_turn`
  counts every cast — Potioner's Trove's printed "Activate only if
  you've cast an instant or sorcery spell this turn" approximates by
  reading any spell. A sibling `instant_or_sorcery_cast_this_turn`
  (and `creature_cast_this_turn` for creature-spell triggers) would
  promote Potioner's Trove + a handful of Magecraft-adjacent payoffs
  to their exact-printed gates.

- **Per-turn exile count tally**. Ennis, Debate Moderator's end-step
  counter is gated on `CardsLeftGraveyardThisTurnAtLeast` as a proxy
  for the printed "if one or more cards were put into exile this
  turn". A first-class `Player.cards_exiled_this_turn` (incremented in
  `move_card_to`'s exile branch) + `Predicate::CardsExiledThisTurnAtLeast`
  would land Ennis on the printed predicate and unblock other
  exile-matters Strixhaven cards (Decadence's Lament, Devoted Caretaker
  variants).

- **CounterAdded scope filter**. `EventScope::SelfSource` for
  CounterAdded fires only for counters on the source card. The
  remaining Berta/Heliod-style payoffs need scope variants for
  "any creature you control" (Heliod, Sun-Crowned) and "any permanent"
  (Vorinclex, Monstrous Raider). Add `EventScope::AnotherOfYours` and
  `AnyPlayer` matching for CounterAdded events.

- **Counter-transfer-on-death primitive**. Ambitious Augmenter and
  several SOS Increment-payoff cards trigger "when this dies, if it
  had counters, create a token with those counters." Today there's no
  way to snapshot the dying creature's counter set in a death
  trigger's body. Adding `Selector::DyingPermanent` (or a
  `Effect::TransferCountersToToken { kind, count }`) would unblock
  this whole subtheme.

- **Per-cast converge introspection on the just-cast spell**.
  Magmablood Archaic and Wildgrowth Archaic have spell-cast triggers
  whose body reads the *cast spell's* converge value (number of colors
  spent on the iterated cast), not the source card's own converge
  value. Today the trigger fires but `Value::ConvergedValue` resolves
  to the source's own ETB-recorded value. A
  `Value::CastSpellConvergedValue` (mirror to the existing
  `Selector::CastSpellTarget`) would unblock both Archaic spell-cast
  riders + similar future cards.

### UI

- **Activate-ability gate hint**. When the new
  `ActivatedAbility.condition` rejects an activation,
  `GameError::AbilityConditionNotMet` bubbles up. The 3D client's
  ability-tray UI doesn't yet show "needs 7+ in hand" or "needs IS
  this turn" hint text — add a small tooltip or grayed-out treatment
  that surfaces the predicate in human-readable form (`Predicate ⇒
  "you need ≥7 cards in hand"` etc.) so players don't get cryptic
  rejection feedback.

### Server

- **Per-trigger gate evaluation logging**. Push VIII's
  `EventScope::SelfSource` extension landed silently; the server has
  no instrumentation for which triggers fired vs. were filtered out
  by scope. A debug flag on `dispatch_triggers_for_events` that emits
  `TriggerFiltered { source, kind, scope, reason }` events would help
  diagnose silent-no-fire reports during cube playtesting.

## New suggestions (added 2026-05-01 push IX)

These items came up while implementing the push IX batch and are
listed here so the next pass can pick them up without re-deriving them.

### Engine

- **Look-and-distribute-by-count primitive**. Flow State's printed
  shape ("look at top 3, put 1 in hand and 2 on bottom") and a
  handful of similar SOS cards (Stress Dream, Zimone's Experiment)
  need a `Effect::LookSplit { count, to_hand: Value, to_bottom: Value }`
  primitive that deals out the looked-at cards by category. Today we
  approximate with `Scry N + Draw 1` (correct first-card-to-hand,
  but the controller can't reorder mid-resolution). A first-class
  primitive would also unblock the conditional "instead pick 2"
  upgrade rider on Flow State (gated on a graveyard-IS-pair predicate).

- **Multi-target prompt for instants/sorceries**. Several SOS cards
  specify two target slots (Prismari Charm's "1 damage to one or two
  targets", Pull from the Grave's "up to two creature cards", Cost of
  Brilliance's "draw + LoseLife on player + counter on creature").
  Engine fix tracked in TODO.md "Multi-Target Prompt for Sorceries /
  Instants". Push IX collapses Prismari Charm mode 1 to single-target.

- **Emblem zone**. Professor Dellian Fel's -7 ult and Ral Zarek
  Guest Lecturer's -7 ult both produce emblems that grant ongoing
  abilities. The engine has no emblem zone or `Zone::Emblem` model
  yet. Adding one would unblock dozens of planeswalker ults
  (Elspeth's "creatures get +1/+1, vigilance, lifelink", Liliana's
  "your creatures get +2/+2 menace", etc.). A flat-list `Vec<Emblem>`
  per-player with the same trigger/static plumbing as battlefield
  permanents would suffice.

- **Coin-flip primitive**. Ral Zarek Guest Lecturer's -7 ult
  ("flip five coins"), Krark's Thumb-style replay, and Fiery Gambit
  use coin-flip mechanics. Add `Effect::FlipCoins { count, then }`
  with a `Value::HeadsCount` reading the most recent flip-coin batch.

- **Skip-turn primitive**. Ral Zarek Guest Lecturer's -7 ult also
  needs "target opponent skips their next X turns". Add
  `Effect::SkipTurns { who, count }` + a per-player
  `extra_turn_skip: u32` counter consumed at turn-roll time
  (mirror to the existing `extra_turns_to_take` pattern).

- **Card-name-as-cost activation (Grandeur)**. Page, Loose Leaf
  has Grandeur — "Discard another card named Page, Loose Leaf:
  do thing." Adding `ActivatedAbility.discard_named_self: bool` (or
  a sibling `ActivatedAbility.cost: ActivationCost` enum) would
  unblock Grandeur-style mechanics across MTG history (the original
  Future Sight cycle).

### UI

- **Witherbloom end-step hint**. The new
  `PlayerView.creatures_died_this_turn` field surfaces the
  "Essenceknit Scholar will draw at end step" predicate. The 3D
  client doesn't yet render this hint — adding a small icon or
  badge over Witherbloom-flavoured payoffs (Essenceknit Scholar,
  Cauldron of Essence's death drain) would improve readability.

### Server

- **Death-trigger event ordering audit**. Push IX's tally bumps in
  both `apply_state_based_actions` (SBA path) and
  `remove_to_graveyard_with_triggers` (destroy path) are correct
  for the common case but assume mutual exclusivity. Audit the
  call graph to ensure no creature-death path bumps the tally
  twice (e.g. if a destroy effect both calls
  `remove_to_graveyard_with_triggers` *and* triggers SBA in the
  same resolution window). Today they're disjoint, but this is a
  silent invariant worth a comment + a regression test.

## New suggestions (added 2026-05-01 push X)

These items came up while implementing the push X batch and are
listed here so the next pass can pick them up without re-deriving
them.

### Engine

- **Ward enforcement primitive**. `Keyword::Ward(u32)` exists as a
  variant and is now carried on Inkshape Demonstrator (Ward {2}),
  Fractal Tender (Ward {2}), and Thornfist Striker (Ward {1}). Real
  enforcement still needs:
  - A `BecameTarget(CardId)` event emitted by the cast/activation
    paths when a permanent first becomes the target of an opponent's
    spell or ability.
  - A "counter the spell unless that player pays N" decision shape
    consumable by an `EventScope::Opponent` Ward trigger reading
    `Keyword::Ward(N)` off the source. The decision answer is yes/no
    (pay) — paid means proceed with cost+resolve; refused means
    counter the spell.
  - Hard-mode variant: Ward—Pay X life / Ward—Discard a card / Ward—
    Sacrifice a creature (Mica, Tragedy Feaster, Forum Necroscribe,
    Strife Scholar, Inkshape Demonstrator's printed mode is just mana).

- **Multi-target prompt for spells/abilities**. Push X works around
  this in Pull from the Grave by auto-picking the top 2 creature
  cards from the controller's graveyard via `Selector::Take(_, 2)`,
  but the printed cards specify *target* slots — the current
  implementation can't accept opponent-side targets. A real fix
  needs `GameAction::CastSpell`'s `target` field to become
  `Vec<Target>` (or a sibling `targets: Vec<Target>` channel) and
  the cost/effect path to address `Selector::Target(0)`,
  `Selector::Target(1)`, etc., to the corresponding entries.
  Unblocks Cost of Brilliance, Render Speechless, Homesickness,
  Prismari Charm mode 1, Stress Dream, Vibrant Outburst, and
  several SOS instants/sorceries that bake two target slots.

- **Cast-from-zone snapshot on `StackItem`**. Antiquities on the
  Loose's "if this spell was cast from anywhere other than your hand,
  +1/+1 counter on each Spirit you control" rider reads the cast's
  source zone. The engine already differentiates flashback casts via
  the `CardInstance.kicked` flag, but the rider needs a clean
  `cast_zone: Zone` snapshot stashed on the resolving spell so a
  `Predicate::CastFromGraveyard` (or `CastFromExile`) can gate the
  bonus branch. Same plumbing unblocks Lurrus-style "cast
  permanent-from-graveyard" payoffs.

- **Per-permanent "gained-counter-this-turn" flag**. Fractal Tender's
  end-step "if you put a counter on this creature this turn, mint a
  Fractal" + Tester of the Tangential's pay-X-move-counters need a
  per-`Permanent.counters_added_this_turn: bool` toggle, set on any
  AddCounter event scoped to the permanent and reset on
  begin-of-untap.

### UI

- **Non-land MDFC flip indicator**. The 3D client's right-click flip
  now routes flipped non-land MDFCs through `CastSpellBack`
  (push X). The art swap is already wired (the existing
  `back_face_name` hand-card visual flow handles this), but the cast
  button's tooltip should change from "Cast for {front cost}" to
  "Cast back face for {back cost}" when flipped, so players
  understand which cost will be charged. Today the tooltip still
  reflects the front face's cost.

### Server

- **Action telemetry: `CastSpellBack` audit log**. The new MDFC
  back-face cast path emits the same `SpellCast` event as the front-
  face path. The server's wire log doesn't distinguish "cast as front"
  vs "cast as back" — both look identical from the spectator's view.
  Add a `cast_face: CastFace::{Front,Back,Flashback}` payload on
  `GameEventWire::SpellCast` so replays / spectator UIs can render
  the right face name without round-tripping through the engine.
  **DONE** in push XIV: `GameEvent::SpellCast.face` +
  `GameEventWire::SpellCast.face` now carry the tag.

## New suggestions (added 2026-05-01 pushes XI–XIV)

These items came up while implementing the MDFC + per-spell-type
batches; listed here so the next pass can pick them up without
re-deriving them.

### Engine

- **`Predicate::CastFace`** for triggers that gate on cast face. Push
  XIV added the audit log; future cards like Lurrus / Yorion-style
  "if cast from a non-hand zone" payoffs need a predicate that reads
  the resolving spell's `face` to gate triggers / static effects.

- **MDFC back-face mana-cost label in client**. Push X / XI's right-
  click flip routes through `CastSpellBack`, but the cast button's
  tooltip still shows the front face's cost. Tracked in TODO.md "UI
  — Non-land MDFC flip indicator". Once a `CastingState.flipped: bool`
  flag flows from the targeting prompt to the tooltip layer, the
  tooltip can swap to "Cast back face for {N}".

- **`CastFace::Back` payload on `GameAction::CastSpellBack`** (UI hint).
  The action input has no face indicator today — `CastSpellBack` is
  the only signal. Adding a `face: CastFace` field to other cast
  actions (front cast, alt cast) would make the input log fully
  symmetric with the output event log.

- **Multi-face MDFC support beyond two faces**. Currently
  `CardDefinition.back_face: Option<Box<CardDefinition>>` supports a
  single back face. Modal triple-faced cards (MDF triples like Esika
  // Esika's Chariot, or future cycles) would need
  `back_faces: Vec<Box<CardDefinition>>` + a face-index in
  `CastSpellBack`. Not pressing today but worth tracking.

### UI

- **Per-MDFC card-front recognition**. The 3D client's hand renders
  the card name + cost based on `definition.name`. A right-click flip
  swaps `definition` to the back face's definition; the UI can render
  the new face's name, but the original front face's name is lost
  during the swap. Adding a `back_face_visible: bool` field on the
  client-side hand-card state (instead of mutating `definition`) would
  let the UI flip the rendering without touching the engine state.

### Server

- **MDFC cast face metric**. Push XIV's `CastFace` event payload
  unblocks per-face replay counting. A `metrics::cast_face_counts`
  Prometheus-style histogram (or simple Vec<(CastFace, u32)>
  tally) on the server would surface "how many MDFC back-face casts
  per game" stats useful for cube tuning.

## New suggestions (added 2026-05-01 push XV)

These items came up while implementing the `Effect::MayDo` +
`ActivatedAbility.life_cost` batch and are listed here so the next
pass can pick them up without re-deriving them.

### Engine

- **`Effect::MayPay { mana_cost, body }`** — sibling to push XV's
  `Effect::MayDo`. Adds an optional mana payment (rather than just
  yes/no). Bayou Groff's "may pay {1} to return on death", Killian's
  Confidence's "may pay {W/B} on combat damage to reanimate from gy",
  Tenured Concocter's may-draw-on-target. Today these are collapsed
  to always-do or always-skip. Cleanest path: a new `Decision::
  OptionalCost` variant carrying both the prompt + the mana cost so
  the bot/UI can evaluate affordability before answering yes/no.

- **`Effect::MayChoose { description: String, options: Vec<(String,
  Effect)> }`** — multi-option pick (rather than yes/no). Practiced
  Offense's "lifelink-or-DS" mode pick, Dina's Guidance's "hand or
  graveyard" destination pick, future "name a card" prompts. Today
  these collapse to one always-on branch.

- **`MayDo` for `wants_ui` players**. Today the synchronous decider
  path means UI players land on AutoDecider's default `false`
  answer when their `wants_ui` is true. A future refinement: surface
  `MayDo` through the `suspend_signal` flow so a human-in-the-loop
  player sees the prompt directly. (Current bot/test play is
  unaffected.)

- **`Predicate::CastFace`** — cast-face introspection on the
  resolving spell. Push XIV's `CastFace` event payload added the
  audit log; future cards like Lurrus / Yorion-style "if cast from
  a non-hand zone" payoffs need a predicate that reads the
  resolving spell's `face` (Front / Back / Flashback) to gate
  triggers / static effects.

- **Land-becomes-creature primitive**. Great Hall of the Biblioplex's
  `{5}: becomes 2/4 Wizard creature with 'whenever you cast IS, +1/+0
  EOT'` clause is omitted (push XV) because the engine has no
  Mishra's Factory-style transient creature-grant. Adding `Effect::
  BecomeCreature { p, t, types: Vec<CreatureType>, abilities: …,
  duration }` would unblock this card, Mishra's Factory, Mutavault,
  and the rest of the manland cycle.

- **Bottom-of-library miss path on `RevealUntilFind`**. Today the
  effect mills misses; many SOS cards (Follow the Lumarets, Zimone's
  Experiment, Stirring Honormancer) want misses to go to the bottom
  of the library instead. Add a `to_misses: ZoneDest` field on
  `RevealUntilFind` (defaulting to `ZoneDest::Graveyard` for
  back-compat) and update existing callers to opt into bottom-of-lib.

### UI

- **MayDo prompt rendering**. The 3D client doesn't yet route
  `OptionalTrigger` decisions through a UI affordance — `wants_ui`
  players land on the AutoDecider's `false` answer by default. A
  small "Yes / No" prompt panel anchored to the source card would
  surface the prompt without breaking the existing bot/test paths.

- **"Pay N life" cost label**. The new `cost_label` rendering shows
  "Pay 1 life" for activations carrying `life_cost > 0`. The 3D
  client's ability-tray could use a different color (red?) for the
  life portion of a hybrid mana+life cost so players spot the life
  payment at a glance.

### Server

- **Snapshot test for `life_cost` round-trip**. The new field has
  `#[serde(default)]` so older snapshots load with `life_cost: 0`.
  Add a snapshot round-trip test that exercises a `life_cost: 1`
  ability across a serialize/deserialize cycle to lock in the
  back-compat invariant.

## New suggestions (added 2026-05-01 push XVI)

These items came up while implementing the `Predicate::CastSpellHasX`
+ `Effect::MayPay` + `SelectionRequirement::HasXInCost` +
`Value::LibrarySizeOf` + `CardsInZone(Hand)` filter-fix batch and are
listed here so the next pass can pick them up without re-deriving.

### Engine

- **`SelectionRequirement::ManaValueAtMostV(Value)`** — `ManaValue
  AtMost` takes a `u32` constant today. Several SOS cards need a
  Value-keyed comparator to gate their target filter against a
  cast-time `Value` (most notably Sundering Archaic's Converge ETB
  exile, which clamps the target's mana value to `ConvergedValue`).
  Mind into Matter's "may put a permanent ≤ X from your hand"
  approximation also rolls in here. Plumbing notes: the predicate
  evaluator (`evaluate_requirement_static` / `_on_card`) currently
  takes `(target, controller)` not `ctx`; adding a Value-typed arm
  means threading `ctx` through every call site.

- **`Value::CastSpellManaSpent`** — total mana paid on the just-cast
  spell, threaded through `StackItem::Spell.mana_spent` (mirror to
  `converged_value`). Compute it in `cast_spell` from `pool_before
  .total() - pool_after.total()` and stash it on the spell stack
  item; `dispatch_triggers_for_events` propagates it onto
  `StackItem::Trigger.mana_spent`. Unblocks ~10 SOS cards: Aberrant
  Manawurm's `+X/+0 EOT`, Tackle Artist's `+1/+1 counter` (plus
  bonus at ≥5 mana), Spectacular Skywhale's Opus rider, all
  Increment-bearing creatures (Pensive Professor, Tester of the
  Tangential, Topiary Lecturer's Increment counter, Cuboid Colony,
  Hungry Graffalon, Ambitious Augmenter, Wildgrowth Archaic creature-
  cast extra-counters rider), plus the Opus +1/+1 cycle (Expressive
  Firedancer, Molten-Core Maestro, Thunderdrum Soloist, Muse Seeker,
  Deluge Virtuoso, Exhibition Tidecaller, Magmablood Archaic IS-cast
  fan-out).

- **`Predicate::ManaSpentAtLeast(u32)`** — sibling to
  `CastSpellManaSpent`. Gates Opus's "If five or more mana was spent
  to cast that spell, instead [bigger effect]" branches that today
  are folded into one always-on collapse.

- **`StaticEffect::PumpPTConditional { applies_to, power, toughness,
  condition: Predicate }`** — continuous `+P/+T` pump gated on a
  predicate (re-evaluated each layer pass). Unblocks Comforting
  Counsel's "≥5 growth counters → creatures get +3/+3" anthem,
  Tenured Concocter's Infusion `+2/+0 while life-gained-this-turn`,
  Thornfist Striker's Infusion `+1/+0 + trample for creatures while
  life-gained`. Plumbing: extend `static_ability_to_effects` with a
  per-layer-pass predicate evaluator.

- **`SelectionRequirement::ManaValueAtMostV(Value)` (alias)** —
  same as the first item; double-listed under a different name so
  catalog factories can use either form.

- **Random-bottom-of-library destination for `RevealUntilFind`**.
  Today misses go to graveyard (engine default). Many SOS cards
  printed-want misses to go to the bottom of the library in random
  order (Geometer's Arthropod's "rest on bottom random", Stirring
  Honormancer's "rest into graveyard" already correct, Follow the
  Lumarets's "bottom in random order"). Add a `to_misses: ZoneDest`
  field on `Effect::RevealUntilFind` that defaults to
  `ZoneDest::Graveyard` for back-compat.

- **`StackItem::Spell.cast_face: CastFace`** — push XIV added
  `CastFace` to the event log; lifting it onto the StackItem lets
  spells gate their own resolution effects on cast face. Antiquities
  on the Loose's "if this spell was cast from anywhere other than
  your hand" rider needs this. Pair with a `Predicate::CastFace`
  primitive that walks the stack to read the resolving spell's face.

- **`Selector::CardsInZone` filter-evaluation correctness**. Push
  XVI fixed a silent bug where hand-source `CardsInZone` predicates
  always returned false (the predicate was routed through
  `evaluate_requirement_static`, which only walks battlefield →
  graveyard → exile → stack). The fix routes hand/library/exile/
  graveyard sources through `evaluate_requirement_on_card` (the
  card-level evaluator). Battlefield sources still use
  `evaluate_requirement_static` so permanent-state predicates
  (Tapped, IsAttacking, etc.) resolve correctly. Audit the rest of
  the selector pipeline (e.g. tutor candidate filters) for similar
  battlefield-vs-card-zone routing mistakes.

### UI

- **Right-click MayPay prompt**. The 3D client's existing decision
  panel handles `Decision::OptionalTrigger` for `MayDo` (push XV).
  `MayPay` reuses the same decision shape but the prompt text should
  also surface the affordability gate (gray-out the "Yes" button
  when the mana pool can't afford the cost, instead of letting the
  click silently no-op via the engine's "decline = false" fallback).
  Today wants_ui players land on AutoDecider's Bool(false) anyway.

- **HasXInCost label tooltip**. The new `SelectionRequirement::Has
  XInCost` filter renders as part of a card's reveal/move target
  prompt. The 3D client's target-prompt UI doesn't yet have a
  dedicated tooltip explaining "card must have {X} in its mana
  cost" — useful for Paradox Surveyor's "Land OR HasXInCost"
  reveal filter.

### Server

- **MayPay payment audit log**. The server's `GameEventWire` doesn't
  emit a dedicated "mana cost paid via MayPay" event today; the
  pool-decrease is silent. A `LifePaid`-style `ManaPaidForOptional`
  event (with source CardId + amount) would help replays diagnose
  surprising pool drops.

## New suggestions (added 2026-05-01 push XVII)

### Engine

- **`AnotherOfYours` / `YourControl` event broadcast for non-Attacks
  events**. Push XVII added a combat-side broadcast in
  `declare_attackers` so other-permanent attack-triggers fire (Sparring
  Regimen). The same pattern would unblock `EventKind::CreatureDied
  /AnotherOfYours` on enchantments / artifacts, `EventKind::CardDrawn
  /YourControl` on cards-drawn-payoffs, etc. Some of these already
  fire via `flush_pending_triggers`; an audit of the event-dispatch
  matrix would show which kinds still rely on the per-source walk
  vs. the global trigger queue.

- **`Selector::DiscardedThisResolution` semantic uniformity**. The
  new selector walks each player's graveyard for the discarded id,
  but `Effect::Move` from this selector currently routes through
  `move_card_to`'s graveyard branch. That emits a `CardLeftGraveyard`
  event for the *opponent's* graveyard (since that's where the
  discarded card lives). Mind Roots's "discard from opp →
  battlefield to *your*" therefore bumps the opp's
  `cards_left_graveyard_this_turn` tally — semantically correct for
  Lorehold "cards leave your graveyard" payoffs (it left the
  opponent's, not yours), but a future "graveyards you control"
  filter on these payoffs would surface this asymmetry.

- **Choose-N for Effect::Discard**. Colossus of the Blood Age's
  "discard any number" is currently approximated as "discard your
  entire hand" (the optimal greedy answer). Real "any number"
  semantics need a player prompt with a range (0..hand) instead of
  a fixed count. Adding `Effect::DiscardChoose { who, max,
  filter }` (vs. the existing `DiscardChosen { count }` which
  forces an exact count) would close this gap and unblock other
  "discard up to N" payoffs (Liliana of the Veil's −2, Library of
  Alexandria's discard mode, etc.).

### Card promotions ready (no new primitive)

- **Pursue the Past** 🟡 → ✅ — fully wired via push XV's `Effect::MayDo`
  for the optional discard half + Flashback keyword. Ready to flip
  the doc status.

- **Witherbloom Charm** 🟡 → ✅ — mode 0 wired via push XV's
  `Effect::MayDo`; modes 1 and 2 always resolved correctly. Ready to
  flip.

- **Stadium Tidalmage** 🟡 → ✅ — ETB + attack loots wired via push
  XV's `Effect::MayDo`. Ready to flip.

- **Heated Argument** 🟡 → ✅ — gy-exile + 2-to-controller now a
  paired MayDo. Ready to flip.

### UI

- **Discard tally HUD hint**. Push XVII's
  `Value::CardsDiscardedThisResolution` is invisible at the UI
  layer. Adding a "draws = N" preview on Borrowed Knowledge / Mind
  Roots / Colossus death-trigger card panels would help the
  player understand the scaling. Same shape as the existing
  Quandrix `cards_drawn_this_turn` preview.

### Server

- **`Selector::DiscardedThisResolution` view rendering**. The
  server's `SelectorView` rendering doesn't yet know about the new
  selector variant — falls through to the generic catch-all. A
  short-form label ("cards discarded this way") would surface it
  properly in mouse-over tooltips and replay logs.

## New suggestions (added 2026-05-02 push XVIII)

These items came up while implementing the combat-damage gy-broadcast
+ `Predicate::CastFromGraveyard` + the body-with-Ward batch.

### Engine

- **Copy-spell / copy-permanent primitive**. `Effect::CopySpell` exists
  but only for "copy target spell on the stack" — it doesn't yet
  handle "create a token that's a copy of [permanent]" (Applied
  Geometry, Colorstorm Stallion's Opus rider, Echocasting Symposium).
  A sibling `Effect::CopyPermanent { source: Selector, with: Vec<...> }`
  primitive would unblock the entire copy-permanent payoff family. The
  back-pattern: pick a permanent, deep-clone the `CardInstance`
  (resetting `id`, `damage`, `tapped`), apply per-card overrides
  (Applied Geometry forces 0/0 Fractal type), then place onto bf
  under the controller. Unblocks: Aziza, Mica, Silverquill the
  Disputant, Choreographed Sparks, Applied Geometry, Echocasting
  Symposium, Colorstorm Stallion (token-copy rider), Prismari the
  Inspiration (storm via copy).

- **Cast-from-exile-with-time-limit primitive**. Practiced
  Scrollsmith's "may cast that card until end of next turn",
  Conspiracy Theorist's discard-recursion, The Dawning Archaic's
  attack-trigger gy-cast, Nita's exile-from-opp-gy-then-cast — all
  share the shape "exile a card; the controller may cast it for free
  until time T". A new `Effect::ExileAndMayCast { what: Selector, who:
  PlayerRef, until: Duration, free: bool }` would unblock 6+ cards.

- **Cascade keyword primitive**. Quandrix, the Proof has Cascade
  baked in. Cascade is "exile until you exile a nonland card with
  lower MV; you may cast it for free". Sibling to ExileAndMayCast
  but with the reveal-until loop and the MV constraint. Add
  `Keyword::Cascade` (already a tagged enum?) + an `Effect::
  CascadeFor { caster_mv: u32 }` primitive.

- **Hybrid Ward (mana-or-life)**. Today `Keyword::Ward(u32)` is a
  single mana-cost integer. Mica's Ward—Pay 3 life is a different
  cost shape (alt-payment). Would benefit from a `Keyword::WardCost
  { mana: ManaCost, life: u32 }` or a more general
  `Keyword::WardEffect(Effect)` (for "Ward—Sac a creature", "Ward—
  Discard a card") that runs a generic effect on Ward triggers.

- **Token-copy of a permanent**. SOS Lluwen, Pest Friend back-face
  + Felisa Inkling triggers all create token copies of the trigger
  source. Today they all hard-code a fresh `TokenDefinition`. A
  generic `Effect::CreateTokenCopy { source: Selector, count: Value }`
  would let cards reference a self-source token shape without
  hard-coding the body each time.

### UI

- **Cast-face badge in replay log**. Push XVIII threads
  `CastFace` into both events and `StackItem`. The replay log /
  spectator UI could surface a per-spell badge ("F" for Front, "B"
  for Back-face, "FB" for Flashback) so viewers see at a glance
  which face was cast. Useful for MDFC tracking + flashback replay
  audits.

- **Ward-tag tooltip**. Cards carrying `Keyword::Ward(N)` have no
  enforcement yet, but the static keyword shows in the keyword bar.
  Adding a hover tooltip ("Ward N: targeting costs N more mana")
  would set player expectations correctly even before the engine
  enforces it.

### Server

- **Selector view for `CardsInZone(Hand, filter)`**. Push XVI fixed
  the runtime evaluation, but the `SelectorView` rendering still
  falls through to the generic "cards in hand" label. A filter-
  aware label ("lands in hand", "instants/sorceries in hand") would
  improve the UI hover.

- **Predicate label for `Predicate::CastSpellHasX`**. Today shows
  "cast spell w/ {X}" — accurate but jargon-heavy. A clearer
  human-readable form ("when you cast an X spell") would read
  better in tooltips.

### Card promotions ready (no new primitive)

- **Strife Scholar // Awaken the Ages** — front face is a 3/2
  Orc Sorcerer with Ward. Body wire is straightforward (same
  pattern as Mica / Colorstorm Stallion). Back-face Awaken the
  Ages oracle still needs verifying — Scryfall lookup pending.

- **Inkling Mascot promotion**: existing 🟡 cards labeled "Ward
  keyword primitive" pending — most are body-wired with the Ward
  tag already; the doc could be flipped from 🟡 to ✅ once Ward
  enforcement lands (or stay 🟡 with a clearer note).

## New suggestions (added 2026-05-02 push XIX)

These items came up while implementing Molten Note + the 10 body-only
⏳→🟡 batch, including the back-face MDFCs that are still pending
oracle verification.

### Engine

- **`Value::ManaSpentToCast` primitive**. Push XIX needed an "amount
  of mana spent to cast this spell" formula for Molten Note's damage
  half. Worked around it via `Predicate::CastFromGraveyard` branching
  (hand → `XFromCost + 2`, flashback → `Const(8)`). A first-class
  `Value::ManaSpentToCast` (read from `EffectContext.mana_spent`,
  stamped onto `StackItem::Spell` at cast time = `cost.cmc()` after X
  substitution) would unify the formula into a single Value and
  unblock other "amount of mana spent" cards: Aberrant Manawurm
  (+X/+0 EOT pump), Together as One ("X is mana spent" — currently
  approximated as ConvergedValue on the wrong axis), Tackle Artist's
  5+-mana Opus rider, Spectacular Skywhale's same Opus split,
  Magmablood Archaic's per-cast pump rider.

- **`SelectionRequirement::ManaValueEqualX` (X-keyed predicate)**.
  Fix What's Broken's "Return each artifact and creature card with
  mana value X from your graveyard" needs an X-aware MV filter
  (current `ManaValueAtMost(N)` takes a literal `u32`). A new
  `ManaValueEqualX` variant that reads `EffectContext.x_value` would
  unblock Fix What's Broken, plus other "find/return X-MV cards"
  cards (Wrath of the Skies' "destroy each nonland X-MV permanent",
  Reckoner Bankbuster's grant restriction). Implementation requires
  threading ctx into `evaluate_requirement_static` /
  `evaluate_requirement_on_card` (or a single new selector wrapper
  that filters by X-MV without changing the predicate evaluators).

- **Vehicle / Crew keyword**. Strixhaven Skycoach is wired body-only
  (3/2 Flying with the ETB land tutor). The Vehicle subtype + Crew
  primitive is gating it from a faithful wire. Vehicles are
  artifact-creature hybrids that *only* attack/block when crewed;
  the engine has no "becomes a creature when activated" primitive
  yet. Add `CardType::Vehicle` + `Keyword::Crew(u32)` + a
  `Player.crewed_this_turn: HashSet<CardId>` flag set by activation,
  consulted by combat-eligibility. Unblocks Strixhaven Skycoach,
  future Strixhaven vehicles, and Kaldheim/MID vehicles.

- **Prepare keyword + prepared-state flag** (Half 2 of Prepare —
  Half 1, the spell-side "prepared cards", already ships via the
  MDFC plumbing; see "Prepare Mechanic (SOS)" above and STRIXHAVEN2.md).
  Biblioplex Tomekeeper and Skycoach Waypoint both gate on a
  "prepared" creature state. This is an SOS-only flag flipped on/off
  by toggle effects (the Tomekeeper's ETB choice, the Waypoint's
  `{3},{T}` activation), and only consultable by creatures whose own
  oracle text grants them a "Prepare {cost}" ability. Add
  `CardInstance.prepared: bool` + `Keyword::Prepare(ManaCost)` + an
  `Effect::SetPrepared { what, state: bool }` primitive. Then surface
  "prepare a creature" / "unprepare a creature" effects on the target
  side. The reminder-text gate "Only creatures with prepare spells can
  become prepared" maps to "the target's `back_face` is a prepare
  spell" — i.e. the flag-toggle effects must reject targets with
  `back_face: None`.

### UI

- **Ward N tooltip / cost gate**. 11 cards in this push carry
  `Keyword::Ward(N)` (Strife Scholar, Campus Composer, Mica from
  XVIII, Colorstorm Stallion from XVIII, Prismari the Inspiration's
  Ward(5) approximation, etc). The keyword is a static-only tag
  today — there's no engine cost gate on opponents' targeting. Add
  a hover tooltip ("Ward N: targeting costs N more mana") so the
  player sees the printed text even before the gate lands. The
  gate itself (intercept opp targeting at cast/activation, demand
  N additional mana to keep target legal) is the larger engine
  work.

- **MDFC back-face oracle preview for body-only wires**. Push XIX
  added Strife Scholar and Campus Composer as front-face-only
  bodies (back-face oracle unverified). The 3D client's hover panel
  could surface a "back face oracle pending verification" badge so
  players know the printed back face exists but isn't wired.

### Server

- **`Selector::DiscardedThisResolution` view rendering**. Push XVII
  added the selector. The `ability_effect_label` in `server/view.rs`
  doesn't know the variant exists today (falls into the generic
  `Effect::Move` "Move permanent" arm). A short-form rider on the
  selector ("discarded this way") in card hover-text would improve
  replay log readability for Borrowed Knowledge, Mind Roots, etc.

- **`Selector::CardsInZone(Hand, filter)` view label**. Push XVII
  suggestion still open — add a filter-aware label ("lands in hand",
  "instants/sorceries in hand") so the UI hover for Embrace the
  Paradox / Paradox Surveyor's reveal filter renders the printed
  filter string instead of a generic "cards in hand".

### Card promotions ready (no new primitive)

- **Mind Roots** 🟡 → ✅ — Push XVII says "both halves now wired".
  The doc could be flipped to ✅ today. Pending verification.

- **Witherbloom, the Balancer** 🟡 — body fully wired with Affinity
  for creatures cost-reduction omitted. Adding a static-effect cost
  reduction primitive that scales off `CountOf(Selector::EachPermanent
  (Creature & ControlledByYou))` would unblock the affinity tax,
  promoting it to ✅. Same mechanism unblocks the affinity static
  on the IS spells caster, which is a separate pass.

## New suggestions (added 2026-05-02 push XXIII)

These items came up while implementing the STX 2021 + cube card batch
+ bot planeswalker-attack routing.

### Engine

- **`Effect::AdditionalCostBeforeCast` primitive**. Daemogoth Woe-
  Eater, Eyeblight Cullers, and Big Score all share the shape "as an
  additional cost to cast this spell, X". The current approximation
  is to fire X at ETB (creatures) or at resolution head (spells).
  This works for the net board state, but loses the timing nuance —
  a counterspell can't cancel the additional cost from being paid,
  for instance, and "if you can't pay the additional cost, you can't
  cast the spell" doesn't trip when the engine's pre-flight cast-
  validity check happens. A dedicated primitive (or a flag on
  `CardDefinition`) that runs the additional cost in
  `pay_spell_cost` would fix the timing.

- **`Effect::GainControl` static prompt for sorceries**. Tempted by
  the Oriq currently approximates "gain control of target ≤3-MV
  creature" with Destroy. Same gap as Mind Control and the Bribery
  family. A prompt-driven `Effect::GainControl { what, duration }`
  with `Duration::Permanent` for the static cases + `Duration::EOT`
  for Threaten would unblock all of them.

- **`Value::DistinctCardTypesInGraveyard(PlayerRef)` primitive**.
  Dragon's Rage Channeler's Delirium body buff (+2/+2 + flying when
  ≥4 distinct card types in your gy) and Unholy Heat's Delirium
  upgrade to 6 dmg both need this. The `Value` resolves to a count
  by walking the controller's graveyard and taking `iter().map(|c|
  c.definition.card_types).flatten().unique().count()`. Backed by a
  fresh helper on `Player`. Also unblocks Tarmogoyf's printed P/T
  scaling (currently a flat 4/5 approximation).

- **`Effect::CounterSwap { from, to, kind }` primitive**. Pelt
  Collector's death-rider (move counters from a dying creature to
  another), Ambitious Augmenter's "fractal-with-counters on death"
  rider, and Pestbrood Sloth-style transfer abilities all share the
  shape "move counters off X onto Y on Z event". A first-class
  primitive (or even `Effect::AddCounter` with `amount: CountersOn(
  prev_source)`) would close 3+ ⏳ rows.

### UI

- **Auto-target hint for ETB-then-sac creatures**. Daemogoth Woe-
  Eater, Eyeblight Cullers fire an ETB sacrifice that picks the
  cheapest creature you control. The UI could surface a tooltip
  ("This creature requires sacrificing another creature on ETB —
  click to confirm or cancel") so players don't accidentally
  sacrifice their best blocker. Currently the prompt fires silently.

- **Walker-attack target indicator**. The bot now picks off
  planeswalkers; the UI could surface this with a colored arrow
  from each attacker to its assigned target (red for player attack,
  yellow for walker attack), so the human player can tell at a
  glance which walkers are about to die.

### Bot / AI

- **Mana-rock prioritisation for X spells**. The bot's
  `max_affordable_x` correctly maxes X based on remaining floating
  mana, but it doesn't tap mana rocks to unlock more X. A 2-step
  pass — "tap all mana, recompute max X, choose highest-value X
  spell" — would let the bot sink Sol Ring + Mind Stone into a big
  Plumb the Forbidden / Snow Day instead of letting the rocks
  idle.

- **Block prioritisation**. The bot blocks haphazardly today (first
  available creature blocks the first attacker). A better rule:
  block to kill the highest-power attacker first, then double-
  block when no chump-block is favorable. This complements the
  walker-attack routing — the same "what damage are we accepting"
  bookkeeping.

## New suggestions (added 2026-05-02 push XXIV)

These came up while implementing the Witherbloom completion + the
four cross-school Commands.

### Engine

- **`Effect::ChooseModes(n)` for "choose two/three"**. The five STX
  Commands (Witherbloom / Lorehold / Prismari / Quandrix /
  Silverquill) and Moment of Reckoning all print "choose two — same
  mode may be chosen more than once" or "choose two — modes are
  distinct". Today they collapse to `ChooseMode` (choose one). A
  `ChooseModes { modes: Vec<Effect>, count: u8, distinct: bool }`
  primitive would fix the printed semantics. Resolution would push N
  copies onto the stack (or run them as a `Seq`), with the
  controller picking N indices via a new `Decision::ChooseModes`
  payload. Same plumbing unblocks 6+ ⏳ rows.

- **`EventKind::Blocks` event**. Daemogoth Titan's "or blocks" rider
  is omitted because the engine has `Attacks` and `BecomesBlocked`
  but no symmetric blocker-side event. A `Blocks/SelfSource` event
  would let the trigger fire when the titan is declared as a
  blocker. Same primitive helps Daemogoth Inquisitor (also "attacks
  or blocks") and any "whenever ~ deals combat damage as a blocker"
  rider. Plumbing: emit the event from `declare_blockers` once
  validation passes, then re-use the existing `fire_attack_triggers`
  shape.

- **Modal mana abilities**. Witherbloom Pledgemage's `{T}, Pay 1
  life: Add {B} or {G}` is collapsed to "Add {B}" because the
  effect path picks one `ManaPayload::Colors` at construction time.
  A `Decision::PickColor` step (mid-resolution, after the cost is
  paid) would let the controller pick {B} or {G} per activation.
  Same path unblocks the SOS school-land "{T}: Add {C} or {color}"
  modes (currently always {C}) and any future "Add one mana of any
  color" choice (Birds of Paradise, City of Brass).

- **Foretell alt-cost primitive**. Saw It Coming's Foretell {1}{U}
  (and Behold the Multiverse, Behold the Beyond, etc.) all share
  the shape "{2} face-down: exile this from your hand. You may cast
  it for its foretell cost on a later turn". This is structurally a
  new `AlternativeCost { foretell: bool }` plus a delayed-cast-from-
  exile zone-tag. Same pipeline (cast-from-exile-with-time-limit)
  also unblocks Velomachus Lorehold's reveal-and-cast and Practiced
  Scrollsmith's "may cast that card until next turn".

### UI

- **Mode-pick prompt for `ChooseMode` spells**. The auto-decider
  picks mode 0 by default for tests; the human-driven path needs a
  modal-prompt UI (radio buttons or a button row) wired to
  `Decision::PickMode`. Currently the client renders the spell
  effect label but no mode-picker, so casting a Command always
  collapses to mode 0.

- **Life-cost ability indicator**. Witherbloom Pledgemage's `Pay 1
  life` activation cost now ships in `ability_cost_label` ("Pay 1
  life"), but there's no visual cue when the activation is *not*
  affordable due to insufficient life. A red-greying of the button
  (similar to the existing mana-affordability greying) would
  surface the rejection reason before the player clicks.

### Bot / AI

- **Modal-spell mode picker tied to board state**. The bot today
  picks mode 0 for `ChooseMode` spells via the auto-decider. A
  smarter rule: walk each mode's effect, score it against the
  current board, pick the highest-scoring legal mode. Concrete:
  Lorehold Command should pick mode 0 (drain 4) when opp life is
  low, mode 1 (Spirit tokens) when board is empty, mode 3 (exile
  gy) when opp has a graveyard recursion threat. Reuses the
  existing `auto_target_for_effect` heuristic.


## New suggestions (added 2026-05-02 push XXIX)

These items came up while implementing the 10-card Lorehold + STX
2021 batch + Abrupt Decay bug fix.

### Engine

- **`Value::AttackersThisCombat` primitive**. ✅ Done in push XXX.
  Reads `state.attacking.len()`. Augusta, Dean of Order's gate is
  now real; Adriana, Captain of the Guard's "for each *other*
  attacking" pump is unblocked (just `Diff(AttackersThisCombat,
  1)`).
- **Filter evaluation on broadcast Attack triggers**. ✅ Done in
  push XXX. `combat.rs` was extended to evaluate
  `AnotherOfYours` / `YourControl` / `AnyPlayer`-scoped Attack
  trigger filters in a second pass after every attacker is in
  `self.attacking`. Pre-fix the broadcast silently ignored every
  filter on Attack triggers.

- **Always-flippable DFC primitive (split-card-style)**. Plargg,
  Dean of Chaos // Augusta, Dean of Order is the front of a paired
  legend where the controller picks which face to cast each time.
  The current MDFC pipeline (`back_face: Some(_)`) is one-directional
  — once you pick the front face at cast time, you can't flip back.
  An "either-face-castable" mode (sibling to `back_face`) would
  unblock the full DFC cycle: Plargg/Augusta (R/W), Will/Rowan
  (U/R), Lisette/Lukka (G/B?), etc. Each face has independent cost,
  effect, body — so the engine just needs to materialise the
  correct face's `CardDefinition` at cast time.

- **`Effect::DivideDamage` primitive for "divided as you choose"**.
  Magma Opus's "4 damage divided" + Bonecrusher Giant's "2 dmg to
  opp + 4 dmg to creature" + Twin Bolt's "2 damage divided as you
  choose" all share the same shape: an integer N to distribute
  across multiple targets. `Effect::DivideDamage { total, targets:
  Vec<Selector> }` + a new `Decision::DivideDamage` answer would
  let the controller spread N damage across M targets.

- **Cast-from-exile + play-land-from-exile primitives**. Expressive
  Iteration's "exile top 3, you may play a land and cast a spell
  from among them this turn" is omitted because (a) the engine has
  no "may cast from exile (without paying)" pipeline, and (b)
  there's no "may play a land from exile" hook. Both close several
  ⏳ rows: Augur of Bolas, Outpost Siege, Sin Prodder, future
  cascade-style cards.

### UI

- **Or-composite filter labels for stack-side filters**.
  `entity_matches_label`'s Or arm now covers binary type-token
  composites (push XXIX). The next gap is Or-composites that mix a
  type token with a stack predicate — e.g., Counterspell-style
  filters of `IsSpellOnStack ∧ HasCardType(Instant) OR
  HasCardType(Sorcery)`. A small recursion that rebuilds the label
  from the inner Or would close that gap.

- ~~**Audit script for STX 2021 cards**~~. ✅ Done in push XLI.
  `scripts/audit_stx_base.py` walks the "Strixhaven base set (STX)"
  table at the bottom of STRIXHAVEN2.md and cross-references against
  `catalog::sets::stx/` plus `decks/modern.rs` (where some STX cards
  spilled over). Reports false positives (doc says ✅/🟡, no catalog
  string), false negatives (catalog string but doc says ⏳), and a
  per-section breakdown. Mixed-status statuses like "✅ ← 🟡"
  count as ✅ — same convention as the SOS audit script.

### Bot / AI

- **Mode picker scored against board state**. The bot enumerates
  each ChooseMode mode as a separate `CastSpell` candidate, then
  picks one randomly. A smarter version would score each mode
  against the current board (mode 0 = "drain 4" → score by opp's
  life total; mode 1 = "create tokens" → score by board emptiness;
  mode 2 = "destroy artifact" → score by opp's artifact count) and
  pick the highest-scoring legal mode.

- **Lesson sideboard model**. Hunt for Specimens, Eyetwitch,
  Igneous Inspiration, Enthusiastic Study all use Learn at some
  point — currently approximated as `Draw 1`. A real Lesson
  sideboard model (a 5-card sideboard the player sees during
  deckbuilding, accessible only via Learn) would close all of
  those at once. Plumbing: a new `PlayerData.lesson_sideboard:
  Vec<CardId>` slot + a Learn-aware `Decision::ChooseLesson` answer.

## New suggestions (added 2026-05-03 push XXXI)

These came up while implementing `Value::ManaSpentToCast` + the Opus +
Increment payoff cycle.

### Engine

- **`Value::ManaSpentToCast` for activated abilities**. The new
  primitive reads off `StackItem::Spell`, so an *activated* ability
  fired from a stack item that isn't a spell (e.g., "Whenever you
  activate an ability with mana value X or greater") returns 0. A
  parallel `Value::ManaSpentToActivate` (reading
  `StackItem::Trigger.activation_cost` on a fresh field) would unblock
  ability-cost-aware payoffs. Same shape as the current ManaSpentToCast
  primitive, just on a different stack-item variant.

- **Strict "instead" semantics for Opus / Increment**. Today the
  `opus()` shortcut runs both halves on big casts (cheap + extra);
  printed Oracle says "this creature gets +X/+0 instead". A new
  `Effect::Branch { gate: Predicate, then: Effect, else_: Effect }`
  would let cards substitute one half for the other — same primitive
  could replace the bespoke `Effect::If` arms scattered around.
  Combat-correct today (the bigger payoff dominates), but a strict
  "instead" fixes some corner-case interactions (e.g., a +1/+1 EOT
  pump that would also trigger a "whenever this gets a counter"
  rider — currently both fire on big casts).

- **`EventKind::Blocks` symmetry pass**. Push XXXI added the event
  but only Daemogoth Titan uses it today. Sweep the catalog for
  cards with a "blocks" rider that might benefit:
  Mardu Heart-Piercer (block trigger ping), Daemogoth Inquisitor
  (printed: "or blocks"), and any future block-side payoff. Right
  now those still get the omitted-rider fallback.

### UI

- **Opus / Increment hover hint**. Cards with the new shortcuts
  should surface a "magecraft" / "increment" badge in the keyword
  bar so players see the trigger family at a glance. Same pattern
  as push XXIV's life-cost ability indicator. Today the trigger is
  only visible by hovering the trigger source.

### Bot / AI

- **Big-spell prioritisation around Opus payoffs**. The bot's
  spell-priority scoring doesn't yet know that casting a 5+-mana
  spell when an Opus body is on the field unlocks the bigger
  payoff. A small heuristic — "+5 score if you control a body with
  an Opus trigger and the candidate spell costs ≥5" — would cluster
  Wisdom of Ages / Pox Plague / X-cost spells around the Opus
  finishers. Same shape as the existing magecraft-aware scoring
  for cheap IS spells.

## New suggestions (added 2026-05-04 push XLI)

These came up while implementing the combat-damage prevention
shield + Owlin Shieldmage promotion + Quandrix Biomathematician.

### Engine

- **`Effect::PreventDamageFromSource { source, amount }`** — CR
  615.7-style "prevent the next N damage from `source`" shield
  primitive. Closes Healing Salve mode 2 ("prevent the next 3
  damage to any target this turn") + various per-source prevention
  riders. Plumbing: a per-game-state `damage_prevention_shields:
  Vec<DamageShield>` list, each shield tracking source / target /
  remaining amount; checked at damage-deal time. Decrements until
  empty, then expires. Distinct from the new
  `combat_damage_prevented_this_turn` blanket flag — that one
  short-circuits all combat damage events; this one would prevent
  per-event damage amounts.

- **`Predicate::CardsDrawnThisTurnAtLeast { who, at_least }`** —
  syntactic sugar for `ValueAtLeast(CardsDrawnThisTurn(who), at_least)`.
  The Faerie Mastermind promotion just used `ValueAtLeast` directly
  but a dedicated predicate would mirror the existing
  `LifeGainedThisTurnAtLeast` / `CardsLeftGraveyardThisTurnAtLeast`
  family for grep-ability and consistent shape. Same evaluation
  logic, just a cleaner card-side spelling.

- **Counter-multiplier static** (Tanazir Quandrix). The "+1/+1
  counters put on permanents you control are doubled" static is
  blocked by the absence of a multiplier hook in `add_counters`. A
  `StaticEffect::CounterPlacementMultiplier { filter, kind, factor }`
  would let `Effect::AddCounter` query the source's controller's
  battlefield for matching statics and multiply the counter count
  before placement. Same shape as Doubling Season, Hardened
  Scales, Pir, Imaginative Rascal.

### UI

- **Prevention shield indicator in combat HUD**. When
  `combat_damage_prevented_this_turn` is true, the combat banner
  should render a shield icon ("FOG" badge) so players see the
  prevention is active before declaring blockers. Today the flag is
  invisible to the UI — only the resolved-damage step reveals "no
  damage was dealt".

- **Hover hint for prevention-source permanents**. Owlin Shieldmage
  / Spore Frog in play should hover-hint "shield up this turn" when
  the prevention flag is set, so players can identify which source
  triggered it.

### Bot / AI

- **Reactive prevention shields**. Bot doesn't yet know to
  activate Spore Frog / cast Holy Day in response to lethal combat
  damage. A small heuristic — "if incoming combat damage ≥ life
  total, activate any prevention source available" — would prevent
  game-loss in mirror matches with Spore Frog or Holy Day in
  hand/play.

## New suggestions (added 2026-05-04 push XLII)

### Engine

- **CR 306.5b alignment**: thread planeswalker loyalty placement
  through the `enters_with_counters` replacement-effect primitive
  (push XL) rather than hardcoding it in `CardInstance::new`. Today
  loyalty counters are inserted at construction time, before the
  cast-time spell-resolution path reaches `stack.rs`. The
  consequence: counter-multiplier effects (Doubling Season-style
  "if a permanent would enter with counters, it enters with twice
  that many"; Pir / Hardened Scales for +1/+1 counters) don't
  multiply loyalty. The fix is to migrate the per-PW loyalty
  placement to a synthetic `enters_with_counters: Some((Loyalty,
  Const(base_loyalty)))` field that the cast-time replacement-effect
  walker honors. Adds zero behavior for cards without multipliers
  but unlocks the multiplier path for the future.

- **"Look at top N, may exile one of them, bottom rest in random
  order" primitive**. Plargg's second activation (push XLII), Outpost
  Siege, Conspiracy Theorist, Tablet of Discovery all have variants
  of this shape. Today push XLII approximates as `LookAtTop(N) +
  Move(TopOfLibrary{1} → Exile)` — auto-decider always exiles the
  top card. A real `Effect::LookAtTopAndExile { reveal_filter:
  SelectionRequirement, count: Value, may_exile: bool, exiled_to:
  ZoneDest }` shape would let the player pick which of the revealed
  cards to exile (interactive prompt) and put the rest in random
  order on bottom of library. Same family as Brainstorm's "look at
  top 3, take 1, bottom 2" (instant-speed scry-equivalent).

- **"May play exiled card until end of turn" primitive**. Plargg /
  Outpost Siege / Tablet of Discovery / Suspend Aggression /
  Conspiracy Theorist all have an "exile card with right to cast it
  this turn (or until next turn)" rider. Engine has no per-card
  may-play-from-exile-until-EOT flag yet. A `CardInstance.
  may_play_until: Option<u32>` field (turn number when the right
  expires) plus a `cast_spell` pre-check that allows casting from
  exile when the flag matches the current turn would unblock all of
  these.

### UI / Server

- **`PermanentView.loyalty` rendering**. Push XLII surfaces current
  loyalty as a top-level field; the client tooltip already prefers
  the field over scanning counters. Future UI work: render a
  loyalty pip (golden coin) on the planeswalker card art when
  `loyalty.is_some()`, showing the number prominently. Today the
  loyalty number is buried in the tooltip's stat line.

- **Loyalty event log enrichment**. `LoyaltyAbilityActivated`
  emits the loyalty change but not the resulting loyalty value;
  `LoyaltyChanged` emits the new value but not the change. A
  client-side combiner that pairs `Activated` + `Changed` events
  per PW (or a single richer event with both fields) would make
  the spectator log easier to read.

### Bot / AI

- **Activate Plargg's exile activation when shields are up**. Bot
  doesn't yet activate Plargg's `{2}{R}: exile top 1` — the
  expected value is small (low-probability hit on a useful card)
  but in graveyard-aware shells (Wilt in the Heat cost reduction,
  Ark of Hunger drain) the activation is direct value. A heuristic
  "in late game, activate the exile if mana is open" would push
  Plargg from a vanilla 1/3 rummager to a card-advantage engine.

### Format / Card coverage

- **Lesson sideboard model**. Push XLII added 4 more Lesson cards
  (Quick Study, Intro to Prophecy, Intro to Annihilation, Field
  Research) bringing the Lesson cycle to 17 cards. The "Learn"
  payoff (Eyetwitch, Hunt for Specimens, Igneous Inspiration,
  Containment Breach, Pillardrop Warden's etb scry, Mascot
  Exhibition's "search your sideboard for a Lesson") still
  collapses to `Draw 1`. A real Lesson sideboard model would
  expose `Player.sideboard: Vec<CardInstance>` plus an `Effect::
  LearnLesson` primitive that prompts the controller to (a) reveal
  a Lesson from sideboard + put it in hand, (b) discard then draw,
  or (c) decline.

- **STX 2021 mono-color section growth**. Push XLII brought the
  doc tally to 120 rows (was 111). The STX 2021 set has 275 total
  cards on Scryfall; adding the remaining 155 would close the
  catalog. Most are commons / uncommons with simple Magecraft /
  Lesson / fight / pump primitives — just hand-coding work.

## New suggestions (added 2026-05-04 push XLIII)

### Engine

- **Stacked-additional-cost decision picker**. Push XLIII added the
  `additional_sac_cost` / `additional_discard_cost` /
  `additional_life_cost` triplet. Today they pay in a fixed order
  (sac, then discard, then life) — but CR 601.2h says costs that
  don't involve random elements may be paid in *any order* the
  controller chooses. A real picker would surface a
  `Decision::CostOrdering` so the human player can pick which
  additional cost to pay first (relevant when one cost triggers
  another — e.g. a sac → mass-draw effect chain).

- **Optional additional costs (kicker / buyback / overload)**. CR
  118.8b: "Some additional costs are optional." The new
  `additional_*_cost` fields are mandatory only — kicker, buyback,
  and overload all fail today. A real model is `Vec<OptionalCost>`
  with a per-cast announcement (CR 601.2b's "the player announces
  their intentions to pay any or all of those costs"). Same family
  as the engine's existing `AlternativeCost` (Force of Will, Force
  of Negation) but for *additional* costs rather than *replacement*
  costs.

- **Player-chosen discard for `additional_discard_cost`**. Today the
  cast-time discard auto-picks the first N cards in hand. The
  in-resolution `Effect::Discard` flow surfaces a
  `Decision::Discard` when `wants_ui` is set; the cast-time path
  could thread the same decision so human players pick which cards
  to feed to Cathartic Reunion / Thrilling Discovery. Same effort
  as suspending mid-cast — the cast pipeline currently doesn't
  suspend, so threading a suspension-resume path is the lift.

- **Stacked discount + cost-tax + additional-cost interaction**. CR
  601.2f says additional costs apply *after* mana discounts and
  surcharges, then the total cost is "locked in." Today the order
  in `cast_spell` is: pay mana (with discounts/taxes), pay sac, pay
  discard, pay life. That matches the CR — but if a future card has
  a static "spells with additional costs cost {1} less" reduction
  (CR 601.2f mentions cost reductions can apply to alternative
  costs), the engine would need a pre-pay hook to apply that
  reduction before mana is paid.

### UI / Server

- **Cost-shape badge for cast-time additional costs**. The combined
  `additional_cost_label` is now displayed on the client tooltip,
  but a tighter visual cue (red icon for sac, blue for discard,
  green for life) on the card frame would let players see at a
  glance which additional cost a spell carries without hovering.

- **Pre-cast hand-size warning**. When the controller's hand is too
  small to pay an `additional_discard_cost`, the client could
  refuse to start the cast prompt entirely (rather than letting
  the engine reject it post-mana-pay). Mirrors the existing pre-
  cast filter on `additional_sac_cost`.

### Bot / AI

- **Discard-cost heuristic improvement**. The bot today rejects a
  spell with `additional_discard_cost` only when the hand is too
  small to pay it. It doesn't *value* the trade-off — discarding 2
  random cards to draw 3 is +1 net hand size, but the discarded
  cards might be high-value (planeswalkers, finishers) while the
  drawn ones are random. A heuristic that values the discard
  against the draw (ratio + late-game / early-game weighting)
  would make Cathartic Reunion / Thrilling Discovery picks
  smarter than the current "always cast if mana + hand permit."

## New suggestions (added 2026-05-04 push XLV)

### Engine

- **Colored-pip cost reductions**. `StaticEffect::
  CostReductionTargeting.amount: u32` is generic-only — it can't
  refund colored pips. Brush Off ({2}{U}{U}) prints "{1}{U} less"
  (1 generic + 1 blue), but the engine approximates it as a flat
  -2 generic discount, losing the {U}-pip refund. To match printed
  Oracle exactly, the field would need to become `discount: ManaCost`
  so the engine could reduce specific colored pips. Same family
  applies to Ajani's Response ("{3} less") which is already exact
  because the printed discount is purely generic. Affects: Brush Off
  (push XLV) and any future "{1}{X} less" / "{X}{X} less" cost
  reduction.

- **Damage-replacement → exile primitive**. CR 614 / 615 cover
  damage replacement effects. The engine's only replacement-
  effect primitive is `Effect::PreventCombatDamageThisTurn` (push
  XLI, sticky shield). Cards like Pillar of Flame, Lava Coil,
  Wilt in the Heat all want a "if this damage would result in
  the creature dying this turn, exile it instead" rider —
  effectively an EOT-tracked damage-replacement scoped to a
  specific source's damage event. A real model would be
  `Effect::SetExileOnDeath { what: Selector, duration: Duration }`
  that flips a per-card flag the SBA pass consults when moving
  damaged creatures to the graveyard. Affects: Pillar of Flame,
  Lava Coil, Skyswimmer Koi (already done via different primitive),
  Wilt in the Heat (still gap on second clause).

- **Divided damage primitive**. Forked Bolt, Magma Opus mode 0,
  Sundering Stroke, Splatter Technique mode 0 all print "X damage
  divided as you choose among one or two targets" — the engine has
  no divided-damage primitive (each spell collapses to single-shot).
  The fix needs `Effect::DealDamageDivided { total: Value,
  target_count: usize, target_filter: SelectionRequirement }` plus
  a per-cast `Decision::DamageDistribution(Vec<u32>)` that the
  controller answers (CR 601.2d says divided damage is announced
  at cast time). Affects: Forked Bolt (push XLV), Magma Opus,
  Sundering Stroke, Splatter Technique mode 0, Boros Charm mode 0
  (the printed "4 to player" half is single-target so unaffected).

- **CR 120.3b/d — Infect/Wither routing**. Damage from a source
  with Infect to a creature should result in -1/-1 counters
  (CR 120.3d), not damage marks. The engine currently emits
  damage marks regardless of source keywords. Same for Wither.
  Fix: `deal_damage_to_creature` should branch on the source's
  keyword set and route through `AddCounter(-1/-1)` instead of
  the damage-marks path when Infect or Wither is present.
  Affects: Phyrexian Crusader, Glissa Sunslayer, Plague Stinger
  (none in catalog yet, but unblocks future Infect cycles).

- **CR 120.10 excess-damage tracking**. Excess-damage triggers
  (Soul-Scar Mage's "if it would deal noncombat damage to a
  creature, prevent that and put -1/-1 counters") want per-event
  excess-damage tally. The engine can compute excess damage at
  resolution time but doesn't expose it as a `Value` for trigger
  conditions. New primitive: `Value::ExcessDamageDealt` reading
  from the most recent `DamageDealt` event. Affects: Soul-Scar
  Mage (not in catalog), Heated Argument's printed "exile cards
  equal to the excess damage" half (already 🟡 by simplifying to
  flat damage in the catalog).

### UI / Server

- **Effective-cost hint in `KnownCard`**. When a spell has a
  `CostReductionTargeting` static, the player sees the printed
  cost in their hand but can't tell what the discounted cost
  would be when targeting matches. A new `KnownCard.
  effective_cost_hints: Vec<String>` (e.g. "{U}{U} when targeting
  an instant/sorcery") would let the client show the discounted
  cost on hover. Same shape as the existing `additional_cost_label`
  but for *reductions* rather than *additions*.

- **Static description rendering for spells**. `PermanentView`
  surfaces `static_abilities: Vec<String>` (push XXXVIII), but
  `KnownCard` (cards in hand) doesn't yet — so Brush Off /
  Run Behind / Ajani's Response's self-static cost reduction
  isn't visible until the card hits the battlefield (which never
  happens for instants). The fix is to also project
  `static_abilities` onto `KnownCard`. Trivial in the view; the
  client just needs to thread it through.

### Bot / AI

- **Cost-reduction-aware target picker**. When the bot considers
  casting a spell with `CostReductionTargeting`, it currently
  picks the auto-target without considering which targets unlock
  the discount. For Brush Off, the bot pays {2}{U}{U} on a
  creature target and {U}{U} on an IS-spell target — but the
  picker doesn't bias toward IS-targets when both are legal.
  Fix: extend `pick_target_for_cast` to score targets by their
  cost reduction contribution (greater discount = higher score),
  with ties broken by the existing lethal-first heuristic.
  Affects: Killian-buffed creature spells, Brush Off, Run Behind,
  Ajani's Response, Wilt in the Heat (cost reduction half).

- **Suicide-cast detection for `additional_life_cost`**. Push XLV's
  bot pre-flight rejects casts at exactly your life total, but
  doesn't *value* the trade-off. A bot at 5 life shouldn't pump
  Vicious Rivalry to X=4 if the resulting board state isn't lethal
  to the opponent — the 4-life loss isn't recouped. A real
  heuristic would compare `expected_value(life_pay, board_clear)`
  against `current_life - life_pay >= safe_threshold`. Same family
  as the existing X-cost-X-spell affordability heuristic in
  `max_affordable_x` but for life rather than mana.
