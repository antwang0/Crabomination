# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.
See `CUBE_FEATURES.md` (cube-card implementation status) and
`STRIXHAVEN2.md` (Secrets-of-Strixhaven status).

## MagicCompRules coverage audit

Periodic spot-check of the rules document
(`crabomination/MagicCompRules 20260116.txt`). Each rule below has a
status tag (✅ wired, 🟡 partial, ⏳ todo) plus a short note.

- ✅ **CR 605.1a — Mana abilities (activated)**: An activated ability is
  a mana ability if it (a) doesn't require a target, (b) could add mana
  to a player's pool when it resolves, and (c) is not a loyalty
  ability. The engine's `is_mana_ability` recogniser in
  `game/actions.rs` matches against the rule's criteria conservatively:
  pure `Effect::AddMana` (no target field, always can add mana) or
  `Effect::Seq` of mana abilities. The `tap_for_mana` mana-source
  driver only accepts an ability that passes this check. Pushed
  XVIII: Witherbloom Pledgemage refactored to use `life_cost: 1` +
  pure `AddMana` so it qualifies — proving the round-trip via the new
  `witherbloom_pledgemage_is_a_mana_ability_per_cr_605` test.
- ✅ **CR 605.4a — Mana abilities don't go on the stack**: The mana-
  ability path in `activate_ability` resolves immediately via
  `continue_ability_resolution` (no `StackItem::Trigger` push, no
  priority window). Test:
  `witherbloom_pledgemage_is_a_mana_ability_per_cr_605` asserts the
  stack length is unchanged after activation.
- ✅ **CR 707.2 — Copy characteristics**: Copies acquire copiable values
  of the source (name, cost, color, types, text, P/T, loyalty) plus
  on-stack choices (mode, targets, X, kicker). Wired in push XVII via
  `Effect::CopySpell` cloning the source's `CardDefinition` (which
  holds all copiable values) and the StackItem's `target`/`mode`/
  `x_value`/`converged_value`. Counters, status, and stickers are NOT
  copied (the copy uses a fresh `CardInstance::new` which starts
  zero-state).
- ✅ **CR 707.10 — Spell copies**: Copies of spells aren't cast,
  copies of activated abilities aren't activated. Our `CopySpell`
  pushes a `StackItem::Spell` directly without emitting `SpellCast`
  (the cast triggers don't fire for copies). Copies inherit modes /
  targets / X / converged_value.
- ✅ **CR 707.10a — State-based action**: A copy of a spell ceases to
  exist in any zone other than the stack. Copies are marked
  `CardInstance.is_token = true`; the existing token-cleanup SBA path
  (`stack.rs::check_state_based_actions` at line 730) drops them from
  graveyard / hand / library / exile after resolution. Test:
  `tests::sos::copied_spell_does_not_linger_in_graveyard_after_resolution`.
- 🟡 **CR 706 — Casting spells**: `cast_spell` covers the main path.
  Gaps: choose-additional-cost ("kicker"/"buyback" alternatives are
  via `alternative_cost`, but only one alt-cost can be active at
  cast time; multi-alt cycles aren't generalized).
- ⏳ **CR 702.21 — Cycling**: Not implemented. `keyword::Cycling`
  doesn't exist; cards with Cycling are either stubbed or omitted.
- ⏳ **CR 704.5d (token cleanup)**: Already covered by SBA tokens.retain. ✅
- 🟡 **CR 117.1 — Order of priority**: `pass_priority` walks the
  alive players in seat order. Multi-player APNAP ordering for
  triggers / simultaneous effects is approximated.
- ✅ **CR 119.4 — Pay-life-only-if-life-≥-cost**: Per the rule, a
  player may pay X life only if their life total is greater than or
  equal to X. The activated-ability path
  (`actions.rs::activate_ability`) was already wired to reject
  cleanly with `GameError::InsufficientLife` when life < cost. Push
  XIX (2026-05-12) brings the alt-cost cast path
  (`cast_spell_alternative`) up to parity — the alt-cost life-cost
  gate was missing, so a Force-of-Negation-style spell with
  `AlternativeCost.life_cost: 1` could be cast at 0 life, driving
  life negative. Now the pre-flight gate matches the activated
  ability path. Test scaffolding for both paths in
  `tests::stx::witherbloom_pledgemage_rejects_activation_with_zero_life`
  + the activated-ability path; a future test will exercise the
  alt-cost path once we have an alt-cost-with-life-cost card wired.
- ✅ **CR 121.2 — Drawing cards one at a time**: `Effect::Draw` in
  `game/effects.rs` evaluates the count, then loops one-card-at-a-time
  (`for _ in 0..n`) — matching CR 121.2 "Cards may only be drawn one
  at a time." Each draw fires a `GameEvent::CardDrawn` so trigger
  payoffs (Wheel of Fortune-style draw-N-trigger-N effects) see the
  expected stream of CardDrawn events. The deck-out trigger
  (`121.4 — drawing from empty library`) flips `Player.eliminated`
  immediately and the SBA picks it up the next loop. No further
  wiring required.
- ✅ **CR 121.4 — Decking out loses the game**: Drawing from an
  empty library marks the player `eliminated`; the next SBA pass
  drops them out of the game. Wired in `Effect::Draw` and in the
  per-turn draw step path.
- ✅ **CR 122.3 — +1/+1 and -1/-1 counters cancel**: Per the rule,
  if a permanent has both +1/+1 and -1/-1 counters, `N` of each are
  removed as a state-based action, where `N` is the smaller of the
  two counts. Wired in `game/stack.rs::check_state_based_actions`
  at line 512 inside the main SBA loop, processing every
  battlefield permanent each pass. The implementation pre-dates
  the 2024 rules renumbering and still labels the code comment as
  CR 704.5q/r, which is now the deprecated number — code path is
  correct, comment is stale; fixed in push XX.
- ✅ **CR 122.1d — Stun counter prevents next untap**: A permanent
  with one or more stun counters has a replacement effect
  "instead of being untapped, remove one stun counter." Wired in
  `do_untap` which checks for stun counters on each permanent
  before untapping. Frost Trickster / Snow Day exercise this
  path.
- ✅ **CR 122.6a — Counters on enter-with-counters**: "If an
  object enters the battlefield with counters on it, the effect
  causing the object to be given counters may specify which
  player puts those counters on it. If the effect doesn't
  specify, the object's controller puts them on it." Wired
  implicitly through the ETB-triggered `Effect::AddCounter`
  path — every "enters with N counters" body resolves under the
  controller's resolution context, so `ctx.controller`
  determines who places the counters (no observable
  multi-player effect today since the bot harness always has the
  controller place; but the implementation matches the rule).

## Recent additions

- ✅ **Push XXI (2026-05-12)**: 6 new STX cards in
  `catalog::sets::stx::extras` + 6 functionality tests. Tests at
  1098 (+6 net):
  - **Confront the Past** 🟡 — `{3}{R}` Sorcery, 3-mode reanimate-
    or-bounce-or-burn target planeswalker. Mode 2 (X-dmg where X =
    loyalty counters) collapsed to flat 3 dmg.
  - **Specter of the Fens** ✅ — `{4}{B}` 3/4 Flying Specter, ETB
    return creature/PW from your gy → hand.
  - **Mascot Interception** ✅ — `{4}{R}{W}` Instant, threaten +
    untap + haste EOT against any permanent. Instant-speed cousin
    of Tempted by the Oriq.
  - **Twinscroll Shaman** ✅ — `{2}{U}{R}` 3/3, Magecraft copies
    the cast spell via `Effect::CopySpell` (the first cube card
    using the push XVII/XVIII CopySpell wiring through Magecraft's
    trigger_source binding).
  - **Practical Research** ✅ — `{1}{G}{U}` Sorcery, doubles +1/+1
    counters on the chosen friendly creature via `AddCounter(amount
    = CountersOn(target, +1/+1))`.
  - **Hall of Oracles** ✅ — Land. `{T}: Add {C}` + `{2}, {T}:
    +1/+1 counter on Wizard or Fractal you control`.
  - CR 707 alignment notes: `Effect::CopySpell` faithfully copies
    modes/targets/X/converged per 707.10. 707.10d (copy-for-each-
    target) and 707.10f (permanent-spell copies → tokens) remain
    as engine follow-ups.

- ✅ **Push XX (2026-05-12)**: 10 new STX cards + 1 engine improvement
  + CR 122.3 audit. Tests at 1092 (+13 net). All cards land in
  `catalog::sets::stx::extras`:
  - **Curate** (U) 🟡 — Scry 3 + Draw 1 approximation of "look at top
    4, take 1, bottom rest randomly."
  - **Solve the Equation** (U) ✅ — Tutor an instant/sorcery into hand
    via `Effect::Search`.
  - **Resculpt** (U) ✅ — Exile creature/artifact; its original
    controller creates a 4/4 blue Elemental.
  - **Mortality Spear** (B/G) ✅ — Destroy creature/PW (Battle subtype
    omitted).
  - **Daemogoth Titan** (B) 🟡 — 11/11 with attack-trigger sac (block
    half ⏳).
  - **Daemogoth Woe-Eater** (B/G) 🟡 — ETB sac + attack-trigger
    sac-into-counter.
  - **Honor Troll** (B/G) 🟡 — 1/4 trampler; life-gained-this-turn
    rider ⏳.
  - **Quandrix Cultivator** (G/U) ✅ — ETB basic Forest/Island ramp.
  - **Hofri Ghostforge** (R/W Legendary) 🟡 — body only; anthem +
    exile-and-return cycle ⏳.
  - **Tempted by the Oriq** (B) ✅ — Threaten template (steal + untap
    + Haste EOT).
  - **CR 122.3 audit** — Verified the +1/+1 / -1/-1 counter
    auto-cancellation lives in `game/stack.rs` SBA loop at line 512;
    code comment still references CR 704.5q/r (the prior rule
    numbering). Marked CR 122.3 as ✅ wired in this TODO; the SBA
    test path is exercised indirectly by Felisa's counter-bearing
    death trigger and Spike/Persist mechanics. Comment in
    `game/stack.rs` updated from CR 704.5q/r to CR 122.3 in this
    push.
  - **Engine: `Effect::Sacrifice` source-deprioritization** — the
    AutoDecider sort for forced sacrifice was previously `(token
    first, lowest CMC, lowest power)`. Added a new leading sort key
    that puts the trigger's `ctx.source` LAST in the candidate
    order, so "When this attacks, sacrifice another creature"
    triggers (Daemogoth Titan, Daemogoth Woe-Eater) prefer
    non-source candidates as printed. When the source is the only
    matching candidate, it's still picked as a graceful fallback —
    a strict CR 605 "another" gate would require new
    plumbing. New test
    `daemogoth_titan_attacks_sacrifices_non_source_creature_first`
    verifies the priority.

- ✅ **Push XIX (2026-05-12)**: 2 engine primitives + CR 119.4 alt-cost
  fix + 12 STX cards (2 promotions + 10 new). Tests at 1079 (+15 net):
  - **`SelectionRequirement::Monocolored`** — `distinct_colors() == 1`
    filter, sibling to `Multicolored` / `Colorless`. Promotes
    Vanishing Verse to ✅ (target filter is now the printed exact
    "nonland, monocolored permanent" shape).
  - **CR 119.4 — alt-cost pre-flight life check** — the alt-cost
    cast path was missing the life-cost gate that the activated
    ability path already honored. Fixed to mirror; rejects cleanly
    with `GameError::InsufficientLife`.
  - **Tanazir Quandrix ETB** — promoted from 🟡 (attack-only) to 🟡
    (both halves wired): "double +1/+1 counters on each creature
    you control" wired via `ForEach(Creature & ControlledByYou)` +
    `AddCounter(CountersOn(TriggerSource))`. No new primitive
    needed.
  - **10 new STX cards** added in `catalog::sets::stx::extras`:
    Bookwurm (G ETB life+draw), Field Trip (G fetch+Learn), Reduce
    to Memory (U exile+Inkling), Baleful Mastery (B exile+opp
    draws), Igneous Inspiration (R 3-damage+Learn), Combat
    Professor (W 2/4 fly+vig + Mentor approx), Conspiracy Theorist
    (R 2/1 body), Beaming Defiance (W pump+hexproof), Excavated
    Wall (C 0/4 defender + ETB lifegain), Snow Day (U/R tap+stun),
    Spell Satchel (C two activated abilities).
  - 15 new tests in `tests::stx::*`. All 1079 lib tests pass (was
    1064).

## Suggested next-up tasks

- ⏳ **`SelectionRequirement::OtherThanSource`** — first-class "another
  creature" filter for Sacrifice / Destroy / Exile costs and effects.
  Push XX added a `ctx.source`-aware *sort priority* to
  `Effect::Sacrifice` so Daemogoth-style triggers pick non-source
  candidates first, but a strict filter that excludes the source from
  the candidate set entirely (so when the source is the only
  candidate, the effect no-ops cleanly per CR 605) would close the
  remaining edge case. Wiring needs threading `ctx.source` into
  `evaluate_requirement_static` — a single `&Option<CardId>`
  parameter, mostly mechanical.

- ⏳ **`EventKind::Blocks` / `BlockerDeclared`** — block-half triggers
  (Daemogoth Titan, Wall of Junk, …) need a per-blocker event that
  fires when `DeclareBlockers` resolves. The engine has
  `EventKind::Attacks` for the attack half via the combat module's
  `do_attack`-style hook, but no symmetric path for blockers.
  Suggested shape: emit `BlockerDeclared { blocker, attacker }` in
  `combat::declare_blockers` so triggered abilities can subscribe
  via `EventScope::SelfSource` or `AnotherOfYours`.

- ⏳ **Lesson sideboard model** — Learn currently collapses to
  Draw 1. A true Lesson sideboard would let Eyetwitch / Hunt for
  Specimens / Field Trip / Igneous Inspiration etc. search a
  sideboard of Lesson cards. Needs a per-player Lesson sideboard
  slot plus a search-by-spell-subtype primitive on top of
  `Effect::Search`.
- ⏳ **Multi-target prompt for sorceries/instants** — Vibrant
  Outburst, Snow Day, Stress Dream, etc. all collapse "up to two
  targets" / "two targets" / "any number of targets" into a single
  required target. The engine's `CastSpell` action shape carries
  one `target: Option<Target>`; a `targets: Vec<Target>` field
  would unblock a wide swath of two-target effects.
- ⏳ **Counter-multiplier primitive** — Already used by Tanazir
  (via the ForEach idiom). Future cards (Vorinclex, Doubling
  Season) want a true multiplier on counter accrual; tracked
  separately.
- ⏳ **Mana-spent-on-cast introspection** — Opus / Increment
  riders read "amount of mana spent to cast that spell" on the
  just-cast spell event. The engine doesn't yet preserve the
  numeric mana-paid total per stack item; this would unblock
  Aberrant Manawurm, Tackle Artist, Expressive Firedancer, etc.
  Suggested shape: `Value::ManaSpentOnCast(Box<Selector>)` that
  reads from `StackItem::Spell.mana_paid_total`.

## Past additions

- ✅ **Push XVIII (2026-05-12)**: 3 engine primitives + 1 dispatcher
  bugfix + 9 STX/SOS card promotions/wirings + CR 605 audit. Tests at
  1064+ (+15 net):
  - **`ActivatedAbility.exile_other_filter: Option<SelectionRequirement>`** —
    new cost variant: exile a *different* card from the activator's
    graveyard matching the filter. Pre-flight gate rejects cleanly
    with `GameError::SelectionRequirementViolated` when no match (so
    tap/mana aren't burned). Picks the lowest-CMC matching card so
    the activator keeps higher-value cards. Wired:
    Postmortem Professor `{1}{B}, Exile IS from gy: return self`,
    Lorehold Pledgemage `{2}{R}{W}, Exile any from gy: +1/+1 EOT`.
  - **Trigger dispatcher bugfix** —
    `dispatch_triggers_for_events` had been preparing an
    `event_subject` reference but hardcoding it to `None` when
    pushing the `StackItem::Trigger`. Effects referencing
    `Selector::TriggerSource` from unified-dispatcher triggers
    silently zero'd. Now correctly threaded — unblocks Sparring
    Regimen's per-attacker counter bump and every future trigger
    that reads its source via `TriggerSource`.
  - **`EventScope::AnotherOfYours` controller check** — the scope was
    only checking that the subject isn't the trigger source itself.
    Now also enforces controller match (subject's controller ==
    trigger source's controller). Without this, Sparring Regimen
    would pump opp attackers and Felisa would mint Inklings on opp
    creature deaths — both now correctly gated.
  - **`effect::shortcut::prowess()`** — one-liner trigger constructor
    for printed Prowess reminder text ("Whenever you cast a
    noncreature spell, this creature gets +1/+1 EOT").
  - **9 cards** in STX Lorehold (Apprentice, Pledgemage, Storm-Kiln,
    Sparring Regimen), STX legends (Beledros, Tanazir, Spectacle
    Mage), STX Silverquill (Killian subtype fix), STX Witherbloom
    (Pledgemage CR 605 refactor), SOS (Postmortem Professor, Molten
    Note new factory). Lorehold (R/W) STX school: all 6 cards now
    ✅ or fully-wired 🟡.
  - **CR 605.1a / 605.4a — Mana abilities audit** — confirmed engine's
    `is_mana_ability` recogniser matches the printed criteria
    (conservatively), and confirmed mana abilities resolve without
    the stack via `continue_ability_resolution`. Witherbloom
    Pledgemage's refactor exercises the round-trip with a new test
    (`witherbloom_pledgemage_is_a_mana_ability_per_cr_605`).

- ✅ **SOS push XVII (2026-05-12)**: 5 engine primitives + 7 SOS card
  promotions. Tests at 1048 (+12 net):
  - **`ActivatedAbility.from_graveyard: bool`** + **`exile_self_cost: bool`**
    — first-class graveyard-source activations. The `activate_ability`
    engine path now walks the graveyard for `from_graveyard` abilities;
    `exile_self_cost` exiles the source as part of cost (mirror to
    `sac_cost` for battlefield permanents). Closes the
    "Activated-Ability `From Your Graveyard` Path" gap. Wired:
    Summoned Dromedary, Teacher's Pest, Stone Docent, Eternal Student.
  - **`Effect::CopySpell { what, count }`** — proper implementation
    (was a stub). Locates the matching `StackItem::Spell` for the
    selected target, pushes `count` copies above it on the stack
    (each with fresh CardId; copies are uncounterable). Emits
    `GameEvent::SpellsCopied`. Closes a major TODO ("Copy Primitive").
    Wired: Aziza Mage Tower Captain, Lumaret's Favor (Infusion),
    Social Snub.
  - **`StackItem::Trigger.trigger_source: Option<EntityRef>`** —
    preserves the trigger's source entity (the just-cast spell, the
    ETBing permanent) through stack resolution. Previously lost when
    triggers landed on the stack, breaking lookups for
    `Effect::CopySpell` and `Selector::CastSpellTarget` post-resolve.
    Now propagated via the new `continue_trigger_resolution_with_source`
    helper. `EntityRef` now `Serialize`/`Deserialize` for snapshots.
  - **`push_on_cast_triggers` filter evaluation** — the SpellCast/
    SelfSource trigger path now evaluates the trigger's `filter`
    (Infusion's `LifeGainedThisTurnAtLeast`, Social Snub's
    `SelectorExists`) at trigger-creation time. Previously the filter
    was silently dropped — every SelfSource cast trigger fired
    unconditionally. The fix collects `(Option<Predicate>, Effect)`
    instead of bare effects.
  - **Sorcery-speed activation gate** — `ActivatedAbility.sorcery_speed`
    was declared but never enforced; activations leaked through during
    non-main steps. Now checked at the top of `activate_ability`,
    returning `GameError::SorcerySpeedOnly` cleanly. Used by Cauldron
    of Essence, Stone Docent, Summoned Dromedary, etc.
  - **12 new tests** in `tests::sos::*` (from_graveyard activations,
    Copy primitive both yes/no paths, Infusion gating, instant-speed
    rejection). All 1048 lib tests pass (was 1036).

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

### Copy Primitive ✅ DONE
~~No general "create a copy of target spell/permanent" effect exists.  Needed for:
Reverberate, Fork, Strionic Resonator, Quasiduplicate, Saheeli Rai −3, etc.
The `CopySpell` effect stub exists in `effect.rs` but is not wired through
`apply_effect`.~~ Done in push XVII: `Effect::CopySpell { what, count }`
locates the matching `StackItem::Spell` and pushes `count` copies above it
on the stack with fresh CardIds. Copies are flagged `uncounterable: true`.
Wired: Aziza Mage Tower Captain (Magecraft copy with tap-3 cost),
Lumaret's Favor (Infusion copy gated on life-gain), Social Snub (cast-time
copy gated on creature-control). Still TODO for "permanent" copies
(Quasiduplicate, Saheeli Rai −3): the variant exists but the
target → battlefield-token-copy path is not yet wired.

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

### Activated-Ability "From Your Graveyard" Path ✅ DONE
~~The `activate_ability` walker only iterates the battlefield, so cards
with mana-cost-priced graveyard-recursion abilities currently drop the
activation entirely.~~ Done in push XVII:
`ActivatedAbility.from_graveyard: bool` + `exile_self_cost: bool` are
now first-class fields. The `activate_ability` engine path walks the
graveyard for `from_graveyard` abilities; `exile_self_cost` exiles
the source as part of cost (mirror to `sac_cost` for battlefield
permanents). Wired: Summoned Dromedary, Teacher's Pest, Stone Docent,
Eternal Student. Remaining gap (3rd-party cost shapes):
- **Postmortem Professor**: `{1}{B}, Exile an IS card from your
  graveyard: Return this card from your graveyard to the battlefield.`
  needs an additional cost variant: exile a *different* card from gy
  matching a filter. A new `cost: ActivationCost` enum (or sibling
  `exile_other_filter: Option<SelectionRequirement>`) would cover this
  case.
- **Page, Loose Leaf (Grandeur)**: `Discard another card named [self]:
  …`. Needs `discard_named_self_cost: bool` (or named-cost variant).

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

### "Track Cards Discarded by This Effect" Counter
Borrowed Knowledge ("draw cards equal to the number of cards
discarded this way") needs a per-resolution counter that
`Effect::Discard` increments. The mode 1 path is currently
approximated as "draw 7" — a flat-7 reload that misses the printed
"draw exactly as many as you discarded" precision but preserves the
card-advantage tally for typical hand sizes.

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
Secrets of Strixhaven introduces a per-permanent "prepared" flag toggled
by `becomes prepared` / `becomes unprepared` effects. Cards like
Biblioplex Tomekeeper and Skycoach Waypoint flip the flag; payoff cards
have a `Prepare {cost}` activated/triggered ability and reminder text
"(Only creatures with prepare spells can become prepared.)" Engine
needs:
- `PermanentFlag::Prepared` (or `CounterType::Prepared` count-1) on
  `Permanent`, surfaced through `PermanentView`.
- `Effect::SetPrepared { what, value: bool }`.
- `Predicate::IsPrepared` for prepare-payoff conditional clauses.
- A short oracle-text helper that wires "Prepare {cost}: …" into a
  standard activated ability with `gate: IsPrepared`.

Until (1) and (2) land, all prepare-touching SOS cards are ⏳.

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
| Frantic Search | untaps all tapped lands | untap up to three |
| Dark Confidant | fixed 2 life loss | lose life = CMC of revealed card |
| Biorhythm | drain opponents to 0 | set each player's life to creature count |
| Coalition Relic | tap for 1 of any color | tap + charge counter → burst WUBRG |
| Fellwar Stone | tap for 1 of any color | tap for a color an opponent's land produces |
| Static Prison | ETB taps target | also suppresses untap while stun counters exist |
| Rofellos | flat {G}{G} | {G} per Forest you control |
| Spectral Procession | {3}{W}{W}{W} | {2/W}{2/W}{2/W} hybrid (CMC 6) |
| Grim Lavamancer | {R}{T}: 2 damage | must exile 2 cards as additional cost |
| Ichorid | no graveyard gate | requires opponent to have a black creature in GY |
| Pursue the Past | always discards then draws 2 | "you may discard … if you do, draw 2" |
| Witherbloom Charm (mode 0) | always sacrifices | "you may sacrifice … if you do, draw 2" |
| Render Speechless | required creature target | optional second creature target |
| Dina's Guidance | always to hand | choice of hand or graveyard |

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
The bot never attacks planeswalkers.  Adding a heuristic that attacks a
planeswalker when its loyalty is low enough to kill it this turn would make the
bot more competitive.

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

## New suggestions (added 2026-05-12 push XVII)

These items came up while implementing the Increment/Opus payoff
batch (`Value::CastSpellManaSpent`, `Predicate::CastSpellManaSpent
AtLeast`, `Predicate::IncrementSatisfied`, `RevealMissDest`) and are
listed here so the next pass can pick them up without re-deriving.

### Engine

- **`TriggeredAbility.intervening_if: Option<Predicate>`** —
  generalize push XVII's body-wrapping `Effect::If` pattern used by
  `increment_trigger` into a first-class field. Today every card that
  needs rule-603.4 "if X" semantics has to manually wrap its body in
  `Effect::If(predicate, body, Noop)`. A first-class
  `intervening_if` would let the engine handle the wrap automatically
  and surface the gate to the UI (showing the condition string in
  the stack-item tooltip). Felidar Sovereign, Avenger of Zendikar's
  "if you control X creatures" riders, and other classic 603.4 cards
  benefit too. Implementation: read `intervening_if` on the
  `StackItem::Trigger` at resolution-pop time and skip the body if
  it's false.

- **Stack-item `mana_spent` round-trip through the snapshot wire
  format**. `StackItem::Spell.mana_spent` /
  `StackItem::Trigger.mana_spent` are serialized inline today (push
  XVII added the fields with `#[serde(default)]`). But
  `StackItemSnapshot` doesn't yet surface the value to spectator
  clients — the new client wire field is gated on the snapshot's
  `mana_spent: u32` which defaults to 0 on older replays. A new
  `mana_spent` field on `KnownStackItem` (in `crabomination/src/net.rs`)
  with the spectator-side label "Opus 5+" would help debug Increment
  / Opus triggers in playback.

- **Random-bottom shuffle for `RevealMissDest::BottomRandom`**. The
  current implementation deterministically appends each miss to the
  back of the library "in-revealed order". Per the printed text the
  order should be random — but the engine has no RNG hook exposed in
  `resolve_effect`. Adding a `state.rng.shuffle_range(start, end)`
  primitive (with a deterministic seed for tests) would let the
  effect actually randomize. Gameplay impact is essentially zero
  before the next shuffle event, so this is low-priority cosmetic.

- **`Predicate::SourceHasBaseStatAtMost(u32)`** — sibling to
  `IncrementSatisfied` but for fixed thresholds. Several Strixhaven
  cards print "Whenever you cast a spell, if its mana value is
  greater than this creature's power, [effect]" (Hungry Graffalon
  variants); having a Const-keyed comparator lets cards pin the
  threshold to printed number rather than reading P/T live. Both
  shapes are useful; `IncrementSatisfied` is just the "live P/T"
  specialization.

### UI

- **Stack tooltip: show `mana_spent` for top spell**. When a spell is
  on the stack with an Opus listener (Deluge Virtuoso) or an
  Increment listener (Cuboid Colony) on the battlefield, the
  spectator UI doesn't show the cast-time mana_spent value. A small
  badge ("5 mana paid") on the stack item would help players
  visualize which payoffs will fire. Wire this via the new
  `KnownStackItem.mana_spent` field suggested above.

### Server

- **Per-trigger filter-pass logging for Increment / Opus**. Push
  XVII's filter evaluation runs at both push-time and resolution-
  time (rule 603.4). The server doesn't emit a dedicated event for
  either path today. A pair of `TriggerEvaluated { source, kind,
  passed: bool, point: PushOrResolve }` events would help diagnose
  silent-no-fire reports.
