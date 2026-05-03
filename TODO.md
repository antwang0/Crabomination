# Crabomination — TODO

Improvement opportunities for the engine, client, and tooling.
Items are grouped by area and roughly ordered by impact within each group.
See `CUBE_FEATURES.md` (cube-card implementation status) and
`STRIXHAVEN2.md` (Secrets-of-Strixhaven status).

## Recent additions

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

- **Prepare keyword + prepared-state flag**. Biblioplex Tomekeeper
  and Skycoach Waypoint both gate on a "prepared" creature state.
  This is an SOS-only flag flipped on/off by toggle effects (the
  Tomekeeper's ETB choice, the Waypoint's `{3},{T}` activation),
  and only consultable by creatures whose own oracle text grants
  them a "Prepare {cost}" ability. Add `CardInstance.prepared: bool`
  + `Keyword::Prepare(ManaCost)` + an `Effect::SetPrepared { what,
  state: bool }` primitive. Then surface "prepare a creature" /
  "unprepare a creature" effects on the target side.

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

- **Audit script for STX 2021 cards**. The existing
  `scripts/audit_strixhaven2.py` audits SOS only. A sibling script
  that walks `catalog::sets::stx::*` and cross-references against
  the STX 2021 table at the bottom of STRIXHAVEN2.md would catch
  status-row drift (a card added to the catalog without a row in
  the table, or vice versa). Today the STX 2021 status table is
  hand-maintained.

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
