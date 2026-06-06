//! Ability "shell" types: static / triggered / activated / loyalty ability
//! descriptions plus the `StaticEffect` continuous-effect enum. Split out of
//! `effect.rs` (no behavior change); re-exported from `effect` so existing
//! `crate::effect::TriggeredAbility` paths keep resolving.

use super::*;
use serde::{Deserialize, Serialize};
use crate::card::{CounterType, Keyword, SelectionRequirement};

// ── Static abilities ─────────────────────────────────────────────────────────

/// A static ability description — what continuous effect(s) it emits while
/// its source is on the battlefield. Translated at layer-computation time
/// into concrete [`ContinuousEffect`] values by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticAbility {
    // Widen the `&'static str` description to an owned `String` on the wire
    // (and re-intern on load) so `StaticAbility: Deserialize<'de>` holds for
    // any `'de` — required now that `TokenDefinition` (a non-`'static`-bound
    // serde type embedded in `Effect`) carries a `Vec<StaticAbility>`.
    #[serde(with = "crate::static_str_serde")]
    pub description: &'static str,
    pub effect: StaticEffect,
}

/// A continuous effect produced by a static ability. Subsumes the old
/// `StaticAbilityTemplate` enum; maps 1-to-1 to one or more
/// `layers::Modification`s.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaticEffect {
    /// Grant +p/+t to everything the selector picks.
    PumpPT { applies_to: Selector, power: i32, toughness: i32 },
    /// Self-buff scaled by the number of the controller's battlefield
    /// permanents matching `filter`: "this creature gets `+per_power`/
    /// `+per_toughness` for each [filter] you control." Resolved live in
    /// `gather_continuous_effects` (the count needs the GameState). Powers
    /// Karn's Construct token ("+1/+1 for each artifact you control") and
    /// similar self-scaling bodies (Master of Etherium, Ornithopter of
    /// Paradise-style counts).
    PumpSelfByControlledPermanents {
        filter: SelectionRequirement,
        per_power: i32,
        per_toughness: i32,
    },
    /// "As long as [condition], this creature gets +P/+T and has [keyword]."
    /// A conditional self-anthem gated by a `Predicate` evaluated live (with
    /// the source/controller as context) on every layer recompute. Powers
    /// threshold creatures — Carnage Interpreter (≤1 card in hand → +2/+2,
    /// menace), Keen-Eyed Curator (4+ card types exiled with it → +4/+4,
    /// trample), etc. `keyword` is granted only while the condition holds.
    PumpSelfIf {
        condition: Predicate,
        power: i32,
        toughness: i32,
        /// Keywords granted only while the condition holds. Dragon's Rage
        /// Channeler's delirium grants both Flying and "attacks each combat"
        /// (`MustAttack`), so this is a list rather than a single keyword.
        #[serde(default)]
        keywords: Vec<Keyword>,
    },
    /// Grant a keyword to everything the selector picks.
    GrantKeyword { applies_to: Selector, keyword: Keyword },
    /// Replace ETB for matching permanents ("enters tapped").
    EntersTapped { applies_to: Selector },
    /// Controller may play one additional land per turn.
    ExtraLandPerTurn,
    /// Generic cost reduction for spells matching filter.
    CostReduction { filter: SelectionRequirement, amount: u32 },
    /// Target-aware generic cost reduction for spells whose chosen target
    /// matches `target_filter`. Powers Killian, Ink Duelist's "spells you
    /// cast that target a creature cost {2} less to cast."
    ///
    /// Applied during `cast_spell_with_convoke` (and the back-face / alt-
    /// cost siblings) *after* the cast's target is validated. The reduction
    /// is clamped at the spell's current generic-pip total (it cannot
    /// reduce a colored pip), matching CR 601.2f / CR 117.7c.
    CostReductionTargetingFilter {
        spell_filter: SelectionRequirement,
        target_filter: SelectionRequirement,
        amount: u32,
    },
    /// Damping-Sphere-style "spells cost {amount} more after the first
    /// spell that player casts each turn." `filter` narrows which spells
    /// are taxed; the cost increase is applied at cast time when the
    /// caster's `Player.spells_cast_this_turn >= 1`.
    AdditionalCostAfterFirstSpell { filter: SelectionRequirement, amount: u32 },
    /// Thalia-style unconditional tax: spells matching `filter` cost
    /// `amount` more to cast, every time (no first-spell gate). Applied at
    /// cast time alongside `AdditionalCostAfterFirstSpell` in
    /// `extra_cost_for_spell`.
    AdditionalCost { filter: SelectionRequirement, amount: u32 },
    /// Leyline-of-Sanctity-style "you have hexproof": opponents can't
    /// target the source's controller with spells or abilities they
    /// control. Checked by `check_target_legality` for `Target::Player(_)`.
    ControllerHasHexproof,
    /// CR 119.7 — Targeted players can't gain life while this static is
    /// active. The `applies_to` selector resolves to one or more
    /// `PlayerView`-style entries; each matching player has their
    /// `Player.cannot_gain_life` flag set in the per-recompute pass
    /// in `compute_battlefield`. Adjust_life drops positive deltas
    /// targeting those players. Powers Erebos, God of the Dead's
    /// "Each opponent can't gain life" and similar lifegain-prevention
    /// statics. `target: PlayerStaticTarget` carries the affected player
    /// set so the same primitive can express "you can't gain life"
    /// (rare) and the more common "each opponent can't gain life".
    PlayerCannotGainLife { target: PlayerStaticTarget },
    /// CR 119.8 — Targeted players can't lose life while this static is
    /// active. Sibling of `PlayerCannotGainLife`. The check consults
    /// the active battlefield via `player_cannot_lose_life_now` from
    /// the lose-life paths (`Effect::LoseLife`, `Effect::Drain`,
    /// damage-to-player). Cost-side life payments are also gated —
    /// per CR 119.8 "a cost that involves having that player pay
    /// life can't be paid." Used by Platinum Emperion-class statics
    /// ("your life total can't change") and by future "your opponent
    /// can't lose life" payoffs.
    PlayerCannotLoseLife { target: PlayerStaticTarget },
    /// CR 614 — life-gain replacement: while active, when a targeted player
    /// *would* gain life, they lose that much life instead (Tainted Remedy:
    /// "If an opponent would gain life, that player loses that much life
    /// instead."). Consulted in `adjust_life` for positive deltas before the
    /// gain applies; the redirected loss is itself final (not re-replaced).
    LifeGainBecomesLoss { target: PlayerStaticTarget },
    /// CR 121.2b — Targeted players can't draw more than `max` cards each
    /// turn. While active, an `Effect::Draw` that would push a player past
    /// `max` (counting `Player.cards_drawn_this_turn`) is truncated. Models
    /// "Each player can't draw more than one card each turn" effects (Aven
    /// Mindcensor-style draw locks, Spirit of the Labyrinth's `max: 1`
    /// applied to each opponent).
    CapDrawsPerTurn { target: PlayerStaticTarget, max: u32 },
    /// CR 705.3 — Krark's-Thumb-style coin-flip advantage: while active, each
    /// coin flip the targeted player makes is replayed an extra time and
    /// treated as heads if any replay came up heads. Counted (and summed, so
    /// multiple sources stack) by `coin_flip_advantage_now`, which feeds the
    /// `Effect::FlipCoin` resolver on top of `Player.coin_flip_advantage`.
    CoinFlipAdvantage { target: PlayerStaticTarget },
    /// Damping-Sphere-style "lands that tap for more than one mana enter
    /// producing only {C}". Detected at `play_land` time: if any active
    /// `LandsTapColorlessOnly` static is in play, the entering land's
    /// mana abilities are replaced with a single `{T}: Add {C}` ability
    /// when the original would produce > 1 mana per tap. Skipped on the
    /// front-face of MDFCs (which have only one ability) and on basic
    /// lands (single-color, single-mana already).
    LandsTapColorlessOnly,
    /// "Lands `applies_to` are every basic land type" (CR 305.7 — Leyline of
    /// the Guildpact). Emits a layer-4 `SetLandTypes([Plains, Island, Swamp,
    /// Mountain, Forest])`; the engine's intrinsic-basic-land mana abilities
    /// then let each affected land tap for any color.
    GrantAllBasicLandTypes { applies_to: Selector },
    /// "Permanents `applies_to` are all colors" (CR 105.2c — Leyline of the
    /// Guildpact's color half). Emits a layer-5 `SetColors([W,U,B,R,G])`, so
    /// devotion / protection-from-color / color matters reads see all five.
    GrantAllColors { applies_to: Selector },
    /// Collector Ouphe / Karn-style lock: "Activated abilities of artifacts
    /// can't be activated unless they're mana abilities." Checked globally
    /// in `activate_ability` (affects every player). Mana abilities pass.
    ArtifactActivatedAbilitiesLocked,
    /// Teferi, Time Raveler-style: each opponent can cast spells only any
    /// time they could cast a sorcery. Checked at cast time on the
    /// opponent's side.
    OpponentsSorceryTimingOnly,
    /// Teferi, Time Raveler +1: until your next turn, you may cast sorcery
    /// spells as though they had flash. Tracked via `Player.sorceries_as_flash`
    /// (set/cleared by the loyalty ability + `do_untap`).
    ControllerSorceriesAsFlash,
    /// "If one or more tokens would be created under your control, twice
    /// that many tokens are created instead." Used by Adrix and Nev,
    /// Twincasters (Quandrix uncommon legendary). Doubling Season uses a
    /// stronger variant that also doubles counter accrual; this variant
    /// covers the token half only. The static is read at
    /// `Effect::CreateToken` resolution time: each active `DoubleTokens`
    /// permanent the controller has on the battlefield doubles the
    /// token count (2 doublers → 4×, 3 → 8×, …). CR 614.13 framing —
    /// the effect is a replacement that scales the create-token event.
    DoubleTokens,
    /// "If one or more counters would be put on a permanent you control,
    /// twice that many of those counters are put on that permanent instead."
    /// The counter-half of CR 614.16, matching Doubling Season / Hardened
    /// Scales / Branching Evolution-class permanents. Read at
    /// `Effect::AddCounter` resolution time: each active `DoubleCounters`
    /// permanent the controller has on the battlefield doubles the counter
    /// count (2 doublers → 4×, …). Composes multiplicatively with
    /// `DoubleTokens` for cards that print both halves (Doubling Season
    /// itself ships both static abilities).
    DoubleCounters,
    /// CR 614.2 — "If a source would deal damage … it deals double that
    /// damage instead." A *global* damage-replacement (Furnace of Rath,
    /// Gratuitous Violence-class, Fiery Emancipation as ×2 stacking): read
    /// off the battlefield by `GameState::damage_doublers`, every active
    /// instance doubles the dealt amount (2 → 4×, …). Currently wired for
    /// the non-combat `deal_damage_to_from` path; combat-damage doubling is
    /// tracked in TODO.md under CR 614.2.
    DoubleDamageDealt,
    /// CR 614.x — "Permanents entering the battlefield don't cause
    /// abilities of permanents your opponents control to trigger. If a
    /// permanent entering the battlefield causes a triggered ability of
    /// a permanent you control to trigger, that ability triggers an
    /// additional time." Elesh Norn, Mother of Machines. Read at ETB
    /// trigger dispatch via `etb_trigger_multiplier`: any opponent's
    /// permanent with this static suppresses your ETB triggers
    /// (multiplier = 0); each of your own adds one extra fire.
    EtbTriggerSpotlight,
    /// CR 603.x — "If a permanent entering the battlefield causes a
    /// triggered ability of a permanent you control to trigger, that
    /// ability triggers an additional time." Yarok / Panharmonicon. Unlike
    /// `EtbTriggerSpotlight` this only *adds* fires for the controller's own
    /// ETB triggers — it never suppresses opponents'. Read at ETB-trigger
    /// dispatch via `etb_trigger_multiplier`.
    DoubleControllerEtbTriggers,
    /// CR 614.x — "Creatures entering the battlefield don't cause triggered
    /// abilities to trigger." Torpor Orb, Tocatli Honor Guard. When any
    /// permanent with this static is in play, an entering **creature**
    /// fires no ETB triggers at all (its own or other permanents'
    /// "whenever a creature enters" reactions). `etb_trigger_multiplier`
    /// returns 0 for creature entrants while this is active. `also_dies`
    /// extends suppression to creature-death triggers (Hushbringer).
    SuppressCreatureEtbTriggers {
        #[serde(default)]
        also_dies: bool,
    },
    /// CR 702.x — "Creature spells you cast of the chosen type can't be
    /// countered." Cavern of Souls. The chosen creature type lives on
    /// the permanent's `chosen_creature_type` field (set at ETB) — the
    /// engine reads it at cast time via
    /// `caster_grants_uncounterable_with_x`. A creature spell whose
    /// caster controls any permanent carrying this static AND whose
    /// types match the chosen type is flagged uncounterable.
    /// `chosen_creature_type == None` falls back to "unrestricted" so
    /// legacy test fixtures that bypass the ETB still work.
    UncounterableCreaturesOfChosenType,
    /// CR 702.66 — "Spells you cast have delve." Teval, Arbiter of Virtue.
    /// Read at cast time by `controller_grants_spells_delve`: a delve-cards
    /// list is accepted on any spell whose controller has this static, not
    /// just spells printed with `Keyword::Delve`.
    SpellsYouCastHaveDelve,
    /// "Instant and sorcery spells you cast have Affinity for [filter]"
    /// (CR 702.40). The static grants every IS spell the controller casts
    /// an Affinity-style discount of {1} per battlefield permanent matching
    /// `permanent_filter`. Applied during `cost_reduction_for_spell` —
    /// stacks additively with the spell's own card-intrinsic
    /// `CardDefinition.affinity_filter` (so Witherbloom, the Balancer's
    /// own Affinity-for-creatures self-cast doesn't double-dip; non-Balancer
    /// IS spells the controller casts only get the static grant).
    ///
    /// CR 601.2f / 117.7c: generic-only via the existing
    /// `ManaCost::reduce_generic` clamp. Powers Witherbloom, the Balancer's
    /// "Instant and sorcery spells you cast have affinity for creatures"
    /// printed second clause. Future "your IS spells have affinity for
    /// [Artifacts / Lands / Pests]" cards plug in unchanged.
    GrantAffinityToISSpells {
        permanent_filter: SelectionRequirement,
    },
    /// "Whenever you cast a creature spell, that creature enters with
    /// N additional counters of `kind` on it." Read at creature-spell
    /// resolution time (`stack.rs::resolve_spell`'s ETB-counter path)
    /// — after the card's printed `enters_with_counters` are applied
    /// and before SBA. `value` can be `Const(N)`, `XFromCost`, or
    /// `ConvergedValue`, so the static covers fixed-count riders
    /// (Hardened Scales-style) AND mana-spent-scaled riders
    /// (Wildgrowth Archaic — "X is the number of colors of mana spent
    /// to cast it" → `Value::ConvergedValue`). Only the controlled
    /// card's creature spells trigger the rider (the static is gated
    /// on `src.controller == caster`).
    ExtraEtbCountersForCreatureCasts {
        kind: CounterType,
        value: Value,
    },
    /// Strict Proctor — "If a permanent entering the battlefield causes
    /// a triggered ability of a permanent to trigger, that ability's
    /// controller sacrifices the permanent unless they pay {amount}."
    /// Read at ETB-trigger dispatch time (both the self-source path in
    /// `fire_self_etb_triggers` and the unified dispatcher in
    /// `dispatch_triggers_for_events`). For each ETB trigger pushed
    /// onto the stack, the trigger's controller is asked yes/no whether
    /// to pay `amount` generic mana from their pool. On yes + affordable:
    /// pay, fire the trigger normally. On no/unaffordable: sacrifice the
    /// trigger's source (the permanent whose ability is triggering) and
    /// the trigger does not fire. The AutoDecider opts in to paying when
    /// the controller has enough mana floated; otherwise it declines.
    /// Stacks across multiple Strict Proctors (one tax per source).
    EtbTriggerTax {
        amount: u32,
    },
    /// CR 502.3 — "Permanents matching `applies_to` don't untap during
    /// their controllers' untap steps." The classic Stasis / Winter Orb /
    /// Frozen Aether pattern. Read by `do_untap` in `game/stack.rs`:
    /// for every battlefield permanent the engine would normally untap,
    /// it first walks the static effects in play and skips the untap if
    /// any active `PreventUntap` selector matches the permanent.
    ///
    /// The selector is evaluated against the candidate permanent (not
    /// the source of the static), so a permanent-targeted prevention
    /// ("nonbasic lands don't untap during their controllers' untap
    /// steps" via `applies_to = EachPermanent(Nonland & Land)`) and a
    /// global prevention ("creatures don't untap" via
    /// `EachPermanent(Creature)`) both compose cleanly.
    PreventUntap {
        applies_to: Selector,
    },
    /// Trinisphere: while the source is untapped, every spell that would
    /// cost less than `amount` mana to cast costs that much instead
    /// (generic is added to bring the total up). Applies to all players.
    /// Read by the cast paths in `game/actions.rs` after cost reductions.
    SpellCostFloor {
        amount: u32,
    },
    /// Omniscience: the controller may cast spells from their hand without
    /// paying their mana costs. Consulted by
    /// `GameState::player_casts_hand_spells_free`, which lets
    /// `CastFromZoneWithoutPaying` resolve a hand spell free of charge.
    CastHandSpellsFree,
    /// "Attacking creatures you control have <keyword>." Blade Historian
    /// (double strike), and any future combat anthem keyed on the
    /// declare-attackers set. Resolved at `compute_battlefield` time (which
    /// has the live `GameState.attacking` list) into a layer-6 keyword grant
    /// scoped to the controller's attackers — `affects()` can't see combat
    /// state on its own, so this can't route through `selector_to_affected`.
    GrantKeywordToAttackers { keyword: Keyword },
    /// "[Permanents matching `applies_to`] have '<ability>'." Grants a single
    /// activated ability to every permanent the selector picks — Galazeth
    /// Prismari ("Artifacts you control have '{T}: Add one mana of any
    /// color'"), Cryptolith Rite ("Creatures you control have '{T}: Add one
    /// mana of any color'"). `applies_to` is an `EachPermanent(filter)`
    /// evaluated from the static source's controller, so "you control"
    /// clauses scope correctly. Surfaced by `activate_ability` as a virtual
    /// ability at index ≥ the permanent's printed-ability count, so the
    /// standard cost-pay / mana-emit path works unchanged.
    GrantActivatedAbility {
        applies_to: Selector,
        ability: ActivatedAbility,
    },
    /// CR 700.5 / Theros gods — "As long as your devotion to [colors] is
    /// less than `threshold`, this isn't a creature." Resolved at
    /// `gather_continuous_effects` time (which can read devotion via the
    /// live `GameState`) into a layer-4 `RemoveCardType(Creature)` self-
    /// effect, but only while the gate is unmet. Heliod, Erebos, Thassa,
    /// Nylea, Purphoros, and the rest of the Nyx pantheon.
    NotCreatureWhileDevotionBelow {
        colors: Vec<crate::mana::Color>,
        threshold: u32,
    },
    /// CR 614.x — "If a nontoken creature would enter the battlefield and it
    /// wasn't cast, exile it instead." Containment Priest. A global ETB
    /// replacement read off the battlefield in `place_card_in_dest`'s
    /// Battlefield arm: any non-cast nontoken creature being put onto the
    /// battlefield (reanimation, blink-return, reveal-and-put) is rerouted
    /// to exile. Cast creature spells bypass this path entirely (they enter
    /// via `resolve_spell` in `stack.rs`), so they are unaffected.
    ExileNontokenCreaturesNotCast,
    /// CR 402.2 — "You have no maximum hand size." While the controller has
    /// a permanent carrying this static, their cleanup-step discard is
    /// skipped entirely. Read by `effective_max_hand_size`; Reliquary Tower,
    /// Thought Vessel, Spellbook, Library of Leng-adjacent statics.
    NoMaximumHandSize,
    /// CR 305 / 718 — "You may play lands from your graveyard." Crucible of
    /// Worlds, Ramunap Excavator. Read by the land-play legality + the
    /// `PlayLandFromGraveyard` action: a land in the controller's graveyard
    /// becomes a legal land play (still bound by the one-land-per-turn cap).
    MayPlayLandsFromGraveyard,
    /// CR 701.10f — "If you tap a permanent for mana, it produces twice as
    /// much of that mana instead." Mana Reflection. Each instance the
    /// controller of the resolving mana ability has on the battlefield
    /// doubles the produced pip count (2 instances → 4×, …). Read by
    /// `mana_production_doublers_for` just before a mana ability resolves.
    ManaProductionDoubled,
    /// Cursed Totem / Damping Matrix — "Activated abilities of creatures
    /// can't be activated unless they're mana abilities." Global lock
    /// checked in `activate_ability` (sibling of
    /// `ArtifactActivatedAbilitiesLocked`).
    CreatureActivatedAbilitiesLocked,
    /// CR 122.1 — Solemnity-style lock: "Counters can't be put on
    /// permanents or players." A global replacement read at every
    /// counter-placement site (`Effect::AddCounter`, `Effect::Proliferate`,
    /// enters-with-counters). While any instance is on the battlefield the
    /// placement is dropped. Powers Solemnity (the persist / Phyrexian
    /// Unlife combo enabler).
    CountersCantBePlaced,
    /// CR 614.6 — graveyard-hate replacement: "If a card would be put into
    /// a graveyard from anywhere, exile it instead." When `opponents_only`
    /// the redirect applies only to cards bound for a graveyard belonging to
    /// an *opponent* of the static's controller (Leyline of the Void);
    /// otherwise it applies to every player's graveyard (Rest in Peace).
    /// Consulted at every graveyard-placement site via `graveyard_exiled_for`.
    ExileCardsBoundForGraveyard { opponents_only: bool },
}

// ── Triggered / activated / loyalty ability shells ───────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggeredAbility {
    pub event: EventSpec,
    pub effect: Effect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ActivatedAbility {
    pub tap_cost: bool,
    pub mana_cost: crate::mana::ManaCost,
    pub effect: Effect,
    pub once_per_turn: bool,
    pub sorcery_speed: bool,
    /// True if activating this ability requires sacrificing the source
    /// permanent as part of its cost. The sacrifice is applied **after**
    /// tap and mana payment succeed but **before** the effect is queued
    /// for resolution — so by the time the effect runs (or is pushed onto
    /// the stack), the source is already in the graveyard. Used by cards
    /// like Mind Stone (`{1}, {T}, Sacrifice this: Draw a card`),
    /// Cathar Commando, Greater Good, Zuran Orb, etc.
    pub sac_cost: bool,
    /// Optional gating predicate. When set, the activation is rejected
    /// before any cost is paid unless the predicate evaluates to true
    /// against the source/controller context. Used by activated abilities
    /// that include a printed "activate only if …" clause:
    /// - Resonating Lute's `{T}: Draw a card. Activate only if you have
    ///   seven or more cards in your hand.`
    /// - Potioner's Trove's `{T}: You gain 2 life. Activate only if
    ///   you've cast an instant or sorcery spell this turn.`
    /// - Stone Docent's `{W}, Exile this card from your graveyard:
    ///   You gain 2 life. Surveil 1. Activate only as a sorcery.` (the
    ///   sorcery-speed half is already covered by `sorcery_speed`; the
    ///   gate here is for arbitrary predicates).
    #[serde(default)]
    pub condition: Option<Predicate>,
    /// Additional life-payment cost (in addition to mana, tap, and sac).
    /// Paid up front during activation. Activation is rejected with
    /// `GameError::InsufficientLife` when the controller's current life
    /// is below `life_cost` (mirrors the mana-cost pre-pay check). Used
    /// by activated abilities that bake "Pay N life:" into the cost
    /// line — Great Hall of the Biblioplex's `{T}, Pay 1 life: Add one
    /// mana of any color`, future Phyrexian-mana flavoured activations,
    /// City of Brass-style "tap for damage" hybrids, etc.
    ///
    /// Defaults to 0 via `#[serde(default)]` so existing literal
    /// initialisations pick up the new field automatically.
    #[serde(default)]
    pub life_cost: u32,
    /// True if this ability is activated from the controller's graveyard
    /// rather than the battlefield. The activation walker searches the
    /// graveyard for the source instead of the battlefield. Used by
    /// SOS cards with `{cost}: do X` activated abilities that read like
    /// "Activate only from your graveyard." — Summoned Dromedary's
    /// `{1}{W}: return this from gy to hand. sorcery.`, Teacher's Pest's
    /// `{B}{G}: return this from gy to bf tapped.`, Stone Docent (with
    /// `exile_self_cost`), Eternal Student (with `exile_self_cost`),
    /// and Postmortem Professor (with `exile_self_cost` toggled
    /// separately for the "exile an IS from gy" portion not handled
    /// here — the source itself is in gy).
    ///
    /// Defaults to false via `#[serde(default)]` so all existing
    /// literal initializations pick up the new field automatically.
    #[serde(default)]
    pub from_graveyard: bool,
    /// True if activating this ability exiles the source as part of
    /// its cost. Used together with `from_graveyard: true` for cards
    /// whose printed cost line reads "Exile this card from your
    /// graveyard: …" (Stone Docent, Eternal Student). The exile
    /// happens after tap (n/a from gy) + mana + life payments succeed
    /// but **before** the effect resolves, mirroring `sac_cost`'s
    /// timing.
    ///
    /// Defaults to false via `#[serde(default)]`.
    #[serde(default)]
    pub exile_self_cost: bool,
    /// Optional cost: exile a *different* card from the controller's
    /// graveyard matching this filter. Used by activated abilities
    /// whose printed cost line reads "Exile a [filter] card from your
    /// graveyard:" where the exiled card is **not** the source — for
    /// example Postmortem Professor's `{1}{B}, Exile an instant or
    /// sorcery card from your graveyard: Return this card from your
    /// graveyard to the battlefield.` and Lorehold Pledgemage's
    /// `{2}{R}{W}, Exile a card from your graveyard: This creature
    /// gets +1/+1 until end of turn.`
    ///
    /// The exile is applied after tap / mana / life payments succeed
    /// but before the effect resolves, mirroring `sac_cost` /
    /// `exile_self_cost`. If no graveyard card matches, activation is
    /// rejected with `GameError::SelectionRequirementViolated`. The
    /// auto-picker takes the lowest-CMC matching card so the activator
    /// keeps higher-value cards in their graveyard.
    ///
    /// Defaults to None via `#[serde(default)]`. The `u32` count
    /// (defaults to 1 when constructing via the bare-filter helpers) is
    /// the number of graveyard cards that must be exiled to activate.
    /// Used at count 2 for Grim Lavamancer's "exile two cards from
    /// your graveyard as an additional cost".
    #[serde(default)]
    pub exile_other_filter: Option<(SelectionRequirement, u32)>,
    /// Optional self-counter cost-reduction kind. When `Some(kind)`, the
    /// activation's generic mana cost is reduced by one for each counter
    /// of `kind` on the source permanent (clamped at the printed generic
    /// total). Mirrors `affinity_filter` on spells, but reads the
    /// source's own counter pool instead of a battlefield filter — the
    /// shape needed by Strixhaven's Book artifacts whose printed Oracle
    /// is "This ability costs {1} less to activate for each [counter]
    /// counter on this artifact." Currently powers:
    /// - Diary of Dreams's `{5}, {T}: Draw a card.` (Page counters)
    ///
    /// Defaults to None via `#[serde(default)]`.
    #[serde(default)]
    pub self_counter_cost_reduction: Option<crate::card::CounterType>,
    /// Optional cost: sacrifice a *different* permanent the activator
    /// controls matching this filter. Mirrors `exile_other_filter` but
    /// for sacrifice rather than exile. Used by activated abilities
    /// whose printed cost line reads "Sacrifice a [filter]:" where the
    /// sacrifice is **not** the source — for example Greater Good's
    /// `{0}, Sacrifice a creature: Draw cards equal to the sacrificed
    /// creature's power.` and Korlash, Heir to Blackblade's `{B},
    /// Sacrifice a Swamp: Regenerate this creature.` The `u32` count
    /// (defaults to 1 when constructing via bare-filter helpers) is the
    /// number of permanents that must be sacrificed.
    ///
    /// The sacrifice is applied after tap / mana / life payments succeed
    /// but **before** the effect resolves, mirroring `sac_cost` /
    /// `exile_other_filter`. If no controlled permanent matches,
    /// activation is rejected with
    /// `GameError::SelectionRequirementViolated`. The auto-picker
    /// takes the lowest-power matching creature (or the first matching
    /// non-creature) so the activator keeps higher-value creatures
    /// alive.
    ///
    /// Defaults to None via `#[serde(default)]`. When set together with
    /// `sac_cost: true`, both the source AND the filter-matched
    /// permanents are sacrificed (rare but allowed for cost-stacking
    /// shapes).
    #[serde(default)]
    pub sac_other_filter: Option<(SelectionRequirement, u32)>,
    /// Optional cost: tap an *untapped, different* permanent the activator
    /// controls matching this filter (CR 602.5b "tap an untapped … you
    /// control" costs). Mirrors `sac_other_filter` but taps rather than
    /// sacrifices. Used by Opposition (`Tap an untapped creature you
    /// control: Tap target …`) and similar. The auto-picker takes the
    /// lowest-power matching untapped permanent so higher-value creatures
    /// stay open. Rejected with `GameError::SelectionRequirementViolated`
    /// when nothing matches. Defaults to None via `#[serde(default)]`.
    #[serde(default)]
    pub tap_other_filter: Option<SelectionRequirement>,
    /// True if this ability is activated from the controller's hand
    /// rather than the battlefield. The activation walker searches the
    /// hand for the source instead of the battlefield. Pairs with
    /// `exile_self_cost: true` for the "Exile this card from your hand:"
    /// cost line — the pitch mana abilities of Elvish Spirit Guide
    /// (`Exile this from your hand: Add {G}.`) and Simian Spirit Guide
    /// (`… Add {R}.`). Tap costs are illegal from hand and rejected.
    ///
    /// Defaults to false via `#[serde(default)]`.
    #[serde(default)]
    pub from_hand: bool,
    /// Optional {E} (energy) cost (CR 107.16). When > 0, the activator must
    /// have at least this many energy counters; they're spent up front during
    /// activation, mirroring the mana/life pre-pay gate. Powers the
    /// energy-gated mana abilities of Aether Hub and Servant of the Conduit
    /// (`{T}, Pay {E}: Add one mana of any color`).
    ///
    /// Defaults to 0 via `#[serde(default)]` so existing literal
    /// initialisations pick up the new field automatically.
    #[serde(default)]
    pub energy_cost: u32,
    /// Optional cost: discard `count` cards from the activator's hand
    /// matching this filter (CR 602.5b "Discard a [filter] card:" cost
    /// lines). Mirrors `sac_other_filter`/`exile_other_filter` but moves
    /// from hand → graveyard. Used by Fauna Shaman (`{G}, {T}, Discard a
    /// creature card: …`), Survival of the Fittest, etc. Applied after
    /// tap/mana/life payments succeed but before the effect resolves. The
    /// auto-picker takes the lowest-CMC matching hand card. Rejected with
    /// `GameError::SelectionRequirementViolated` when nothing matches.
    ///
    /// Defaults to None via `#[serde(default)]`.
    #[serde(default)]
    pub discard_cost: Option<(SelectionRequirement, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyAbility {
    pub loyalty_cost: i32,
    pub effect: Effect,
}
