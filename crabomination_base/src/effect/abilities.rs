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
    // (and re-intern on load). The `StaticStr` alias keeps serde's derive
    // from pinning `StaticAbility: Deserialize<'static>` — required now that
    // `TokenDefinition` (a non-`'static`-bound serde type embedded in
    // `Effect`) carries a `Vec<StaticAbility>`.
    #[serde(with = "crate::static_str_serde")]
    pub description: crate::static_str_serde::StaticStr,
    pub effect: StaticEffect,
}

/// A continuous effect produced by a static ability. Subsumes the old
/// `StaticAbilityTemplate` enum; maps 1-to-1 to one or more
/// `layers::Modification`s.
/// What a triggered mana ability adds (CR 605.1b — `ExtraManaOnLandTap`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtraManaKind {
    /// One mana of a type the land just produced (Mana Flare).
    Mirror,
    /// A fixed color (Wild Growth's {G}).
    Fixed(crate::mana::Color),
    /// The source's ETB-chosen color (Utopia Sprawl).
    ChosenColor,
    /// One {C}, only when the tap produced colorless mana (Ultima's
    /// "whenever you tap a land for {C}, add an additional {C}").
    MirrorColorless,
    /// One mana of any color, chosen by the controller at tap time (Buried in
    /// the Garden — "adds an additional one mana of any color").
    AnyColor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaticEffect {
    /// Grant +p/+t to everything the selector picks.
    PumpPT { applies_to: Selector, power: i32, toughness: i32 },
    /// Anthem scaled by the counters on the *source*: everything the
    /// selector picks gets +(n×per_power)/+(n×per_toughness) where n is the
    /// source's `kind`-counter count (Joraga Warcaller's "Other Elf
    /// creatures you control get +1/+1 for each +1/+1 counter on this").
    PumpPTPerCounterOnSource {
        applies_to: Selector,
        kind: crate::card::CounterType,
        per_power: i32,
        per_toughness: i32,
    },
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
    /// "As long as [condition], this creature has base power and toughness
    /// P/T." The base-P/T-setting sibling of `PumpSelfIf` — installs a live
    /// layer-7b `SetPowerToughness` while the predicate holds (counters and
    /// +N/+M still stack on top per CR 613.7c/f). Snowmelt Stag
    /// ("During your turn, this creature has base power and toughness 5/2").
    SetBasePtIf { condition: Predicate, power: i32, toughness: i32 },
    /// "This creature can attack as though it didn't have defender as long as
    /// [condition]." A self-static gating defender-bypass on a live predicate
    /// (controller as context). Drowsing Tyrannodon ("…as long as you control
    /// a creature with power 4 or greater").
    CanAttackIgnoringDefenderWhile { condition: Predicate },
    /// "As long as [condition], [creatures the selector picks] get +P/+T."
    /// The conditional-team sibling of `PumpSelfIf` (self) and `PumpPT`
    /// (unconditional team). Resolved live in `gather_continuous_effects`:
    /// the predicate is evaluated with the source as context, and while it
    /// holds a layer-7 pump is emitted for `selector_to_affected(applies_to)`.
    /// Powers quest/threshold anthems — Beastmaster Ascension ("as long as
    /// this has seven or more quest counters, creatures you control get
    /// +5/+5").
    PumpTeamIf {
        condition: Predicate,
        applies_to: Selector,
        power: i32,
        toughness: i32,
        /// Keywords granted while the condition holds (Thornfist Striker's
        /// Infusion trample).
        #[serde(default)]
        keywords: Vec<Keyword>,
    },
    /// "All [filter] have 'This gets +P/+T as long as [condition]'" — the
    /// conditional self-pump granted to a class of permanents (Sedge
    /// Sliver). Unlike `PumpTeamIf`, the condition is evaluated per affected
    /// permanent with *that permanent's controller* as "you", so each
    /// player's Slivers check their own board.
    GrantPumpSelfIf {
        filter: SelectionRequirement,
        condition: Predicate,
        power: i32,
        toughness: i32,
        #[serde(default)]
        keywords: Vec<Keyword>,
    },
    /// "[Creatures the selector picks] have base toughness N" — a layer-7b
    /// characteristic-setting anthem (Maha, Its Feathers Night's "creatures your
    /// opponents control have base toughness 1"). Emits a
    /// `Modification::SetToughness` for `selector_to_affected(applies_to)`; base
    /// power is untouched and +1/+1 counters / 7c pumps stack on top per CR 613.
    SetBaseToughnessForMatching { applies_to: Selector, toughness: i32 },
    /// Grant a keyword to everything the selector picks.
    GrantKeyword { applies_to: Selector, keyword: Keyword },
    /// CR 716.2 — a static ability that only applies while the source Class
    /// enchantment is at level `n` or higher. Wraps `inner`; its continuous
    /// effects are emitted only while `CardInstance.class_level >= n`. Powers a
    /// Class's higher-level static abilities (a level-gated anthem or granted
    /// keyword).
    WhileClassLevelAtLeast { n: u8, inner: Box<StaticEffect> },
    /// CR 611.2 — a static ability whose continuous effect only applies during
    /// the source controller's turn. Wraps `inner`; emitted only while
    /// `active_player == source controller` (Blacksmith's Talent's level-3
    /// "during your turn, equipped creatures you control have double strike and
    /// haste"). Nests inside `WhileClassLevelAtLeast`.
    WhileYourTurn { inner: Box<StaticEffect> },
    /// CR 611.2 — the mirror of `WhileYourTurn`: the wrapped continuous effect
    /// applies only during turns *other than* the source controller's (Oak
    /// Street Innkeeper's "during turns other than yours, tapped creatures you
    /// control have hexproof"). Emitted only while `active_player != controller`.
    WhileNotYourTurn { inner: Box<StaticEffect> },
    /// CR 702.122e / 702.171 — "crews Vehicles and saddles Mounts as though
    /// its power were N greater." Adds `amount` to each affected creature's
    /// power *only* when summing crew / saddle totals (it is not a real P/T
    /// modification). `applies_to` is usually `Selector::This` (Cloudspire
    /// Captain, Deathless Pilot). Read in `GameState::crew` / `saddle`.
    CrewSaddlePowerBonus { applies_to: Selector, amount: i32 },
    /// "This creature crews Vehicles and saddles Mounts using its toughness
    /// rather than its power" (Interface Ace). Read in `GameState::crew` /
    /// `saddle` — a self-only marker; the crew/saddle sum substitutes the
    /// creature's computed toughness for its power.
    SelfCrewsSaddlesWithToughness,
    /// CR 613 — the source has `keyword` as long as it itself matches
    /// `condition` ("As long as this creature is equipped, it has double
    /// strike" — Kor Duelist). Recomputed live against the source via
    /// `evaluate_requirement_static`, so board-state conditions (IsEquipped,
    /// IsEnchanted, IsModified) track correctly.
    SelfHasKeywordWhile { keyword: Keyword, condition: SelectionRequirement },
    /// "This creature has <keyword> as long as <condition>", where the
    /// condition is a live board [`Predicate`] evaluated with the source as
    /// context (not a source-matching filter like `SelfHasKeywordWhile`). Powers
    /// "has lifelink as long as you control another Faerie" (Barrow Naughty) and
    /// similar board-state-gated self keywords.
    SelfHasKeywordWhilePredicate { keyword: Keyword, condition: Predicate },
    /// "All [filter] have protection from the chosen color" — the color is
    /// read from the source's `chosen_color` ETB stamp (Ward Sliver). No-op
    /// until the choice is made.
    GrantProtectionFromChosenColor { applies_to: Selector },
    /// "Your opponents can't cast spells of the chosen color" — reads the
    /// source's `chosen_color` ETB stamp (Iona, Shield of Emeria). Gated at
    /// the cast dispatch.
    OpponentsCantCastChosenColor,
    /// Void Winnower — "Your opponents can't cast spells with even mana
    /// values" (zero is even). Gated at the cast dispatch off the spell's
    /// printed mana value.
    OpponentsCantCastEvenMv,
    /// Void Winnower — "Your opponents can't block with creatures with even
    /// mana values" (zero is even). Consulted in the block-legality check.
    OpponentsCantBlockWithEvenMv,
    /// "Each [creature_type] creature gets +P/+T for each *other*
    /// [creature_type] on the battlefield" (Sliver Legion). State-aware:
    /// gathered with the live battlefield count, one effect per matching
    /// permanent.
    PumpPTPerOtherOfType {
        creature_type: crate::card::CreatureType,
        power: i32,
        toughness: i32,
    },
    /// "Each creature gets +P/+T for each other creature on the battlefield that
    /// shares a creature type with it" (Coat of Arms, CR 702-adjacent tribal
    /// anthem). Resolved state-aware in `gather_continuous_effects`: one
    /// per-creature layer-7 effect scaled by the count of other creatures
    /// sharing ≥1 creature type (Changeling shares every type). Affects every
    /// creature on the battlefield, all controllers.
    PumpPerSharedType { power: i32, toughness: i32 },
    /// Each creature `applies_to` matches gets +`per_power`/+`per_toughness`
    /// for each of *its own* creature types, capped at `max` types (CR 613.7c
    /// layer-7 per-target dynamic pump). Diligent Zookeeper's "+1/+1 for each
    /// of its creature types, to a maximum of 10."
    PumpPTPerOwnCreatureType {
        applies_to: Selector,
        per_power: i32,
        per_toughness: i32,
        max: u32,
    },
    /// "[applies_to] you control get +per/+per for each [count_filter] you
    /// control." A team anthem whose bonus scales with a controlled-permanent
    /// count. `applies_to` and `count_filter` are controller-relative card
    /// filters (the "you control" is implied). Warrior of Light — legendary
    /// creatures you control get +X/+X where X is the number of legendary
    /// creatures you control (`applies_to == count_filter`). With
    /// `count_graveyard`, matching cards in the controller's graveyard are
    /// added to the count (Cid, Timeless Artificer — Artificers on the
    /// battlefield *and* in the graveyard). Resolved live in
    /// `gather_continuous_effects_inner`.
    PumpTeamByControlledPermanents {
        applies_to: SelectionRequirement,
        count_filter: SelectionRequirement,
        per_power: i32,
        per_toughness: i32,
        #[serde(default)]
        count_graveyard: bool,
    },
    /// "As long as this has `n` or more `kind` counters on it, it's an
    /// (artifact) creature." War Balloon (3+ fire counters). Emits a layer-4
    /// `AddCardType(Creature)` self-effect while the count holds; the printed
    /// P/T already carry the creature stats.
    SelfIsCreatureWhileCountersAtLeast { kind: crate::card::CounterType, n: u32 },
    /// "As long as this has `n` or more `kind` counters on it, it has
    /// `keyword`." The keyword-granting sibling of
    /// `SelfIsCreatureWhileCountersAtLeast` (Idol of False Gods — annihilator 2
    /// once it has eight +1/+1 counters). Emits a layer-6 keyword-grant
    /// self-effect while the count holds.
    SelfHasKeywordWhileCountersAtLeast {
        kind: crate::card::CounterType,
        n: u32,
        keyword: crate::card::Keyword,
    },
    /// "[permanents] are [card type] in addition to their other types" — a
    /// layer-4 additive `AddCardType` over everything `applies_to` resolves to
    /// (Toph, the First Metalbender: "nontoken artifacts you control are lands").
    /// Graaz — "Other creatures you control have base power and toughness
    /// 5/3" (layer 7b over a CardMatch filter).
    SetBasePtForFilter {
        applies_to: Selector,
        power: i32,
        toughness: i32,
    },
    /// Graaz — "… and are Juggernauts in addition to their other creature
    /// types" (layer 4 additive creature type over a CardMatch filter).
    AddCreatureTypeToMatching {
        applies_to: Selector,
        creature_type: crate::card::CreatureType,
    },
    AddCardTypeToMatching {
        applies_to: Selector,
        card_type: crate::card::CardType,
    },
    /// CR 613 — each other non-Aura enchantment becomes a creature (layer 4)
    /// with base power and toughness each equal to its mana value (layer 7b).
    /// Opalescence (`yours_only: false`, always on) and Starfield of Nyx
    /// (`yours_only: true`, `requires_five: true` — active only while its
    /// controller has five or more enchantments). Materialized state-aware in
    /// `gather_continuous_effects_inner` since the gate reads the board.
    NonAuraEnchantmentsAreCreatures {
        #[serde(default)]
        yours_only: bool,
        #[serde(default)]
        requires_five: bool,
    },
    /// CR 613 layer 4 — "All nonland permanents are legendary" (Leyline of
    /// Singularity). Adds the Legendary supertype to every nonland permanent
    /// on the battlefield, so the legend rule (CR 704.5j) collapses duplicates
    /// by name across all players. Materialized in
    /// `gather_continuous_effects_inner` (scans the live battlefield).
    AllNonlandPermanentsAreLegendary,
    /// Strip a keyword from matching permanents (CR 613 layer 6) — "creatures
    /// your opponents control lose hexproof and shroud" (Nowhere to Run). A
    /// layer-6 `Modification::RemoveKeyword`, the mirror of `GrantKeyword`.
    LoseKeyword { applies_to: Selector, keyword: Keyword },
    /// CR 113.11 — "lose [keyword] and can't have or gain [keyword]" (the
    /// Theros Archetypes). Beats any grant regardless of timestamp; a
    /// layer-6 `Modification::CantHaveKeyword`.
    CantHaveKeyword { applies_to: Selector, keyword: Keyword },
    /// Replace ETB for matching permanents ("enters tapped").
    EntersTapped { applies_to: Selector },
    /// "This permanent enters tapped unless [condition]" (Horned Loch-Whale —
    /// "enters tapped unless it's your turn"). The conditional sibling of
    /// `EntersTapped`: affected permanents enter tapped only when `condition`
    /// (evaluated with the source as context) is *false*. Applied in
    /// `apply_enters_tapped_replacement`.
    EntersTappedUnless { applies_to: Selector, condition: Predicate },
    /// "Lands you control enter the battlefield untapped" (Spelunking, Amulet
    /// of Vigor-adjacent). An enters-untapped replacement that overrides any
    /// enters-tapped static for lands the source's controller controls.
    LandsEnterUntapped,
    /// "Lethal damage dealt to matching creatures is determined by their power
    /// rather than their toughness" (Zilortha, Strength Incarnate / Mountain
    /// Goat). The SBA reads `power` as the lethal threshold for any creature
    /// `applies_to` matches.
    LethalDamageByPower { applies_to: Selector },
    /// Controller may play one additional land per turn.
    ExtraLandPerTurn,
    /// Generic cost reduction for spells matching filter.
    CostReduction { filter: SelectionRequirement, amount: u32 },
    /// Cost reduction for spells whose name matches the source's `named_card`
    /// (chosen via `Effect::NameCard`). Council of the Absolute — "spells with
    /// the chosen name you cast cost {2} less".
    NamedSpellCostReduction { amount: u32 },
    /// Generic cost reduction equal to the controller's experience-counter
    /// count, for spells matching `filter` (Mizzix of the Izmagnus — "Instant
    /// and sorcery spells you cast cost {X} less, where X is the number of
    /// experience counters you have"). Generic-only; clamped at the generic pip.
    CostReductionPerControllerExperience { filter: SelectionRequirement },
    /// Generic cost reduction equal to the source permanent's computed power,
    /// for spells matching `filter` (Golden-Tail Trainer — "Aura and Equipment
    /// spells you cast cost {X} less, where X is this creature's power").
    /// Generic-only; clamped at the generic pip.
    CostReductionBySourcePower { filter: SelectionRequirement },
    /// Like `CostReduction`, but applies only while `condition` holds for the
    /// controller (Gran-Gran — "Noncreature spells you cast cost {1} less as
    /// long as there are three or more Lesson cards in your graveyard"). The
    /// predicate is evaluated from the controller's perspective. Generic-only.
    CostReductionWhile {
        filter: SelectionRequirement,
        amount: u32,
        condition: crate::effect::Predicate,
    },
    /// Generic cost reduction for spells the controller casts *from their
    /// graveyard* (Gravebreaker Lamia — "Spells you cast from your graveyard
    /// cost {1} less"). Applied only on the graveyard-cast paths (flashback /
    /// retrace / escape / disturb / aftermath); clamped at the generic pip.
    GraveyardCastCostReduction { amount: u32 },
    /// Generic cost reduction for spells the controller casts *from exile*
    /// (Doc Aurlock — "Spells you cast … from exile cost {2} less"). Applied on
    /// the exile-cast paths (foretell, adventure-creature, plotted, impulse
    /// pay-own-cost); clamped at the generic pip.
    ExileCastCostReduction { amount: u32 },
    /// Generic cost reduction for the controller's Plot activations from hand
    /// (Doc Aurlock — "Plotting cards from your hand costs {2} less"). Applied
    /// in `plot_card`; clamped at the generic pip.
    PlotCostReduction { amount: u32 },
    /// Like `CostReduction`, but only on turns other than the controller's
    /// (Naiad of Hidden Coves — "During turns other than yours, spells you
    /// cast cost {1} less"). Applied in `cost_reduction_for_spell` when the
    /// caster is not the active player.
    CostReductionDuringOpponentsTurn { filter: SelectionRequirement, amount: u32 },
    /// "Your Nth spell each turn costs `amount` less" (Highspire Bell-Ringer —
    /// second spell). Applied in `cost_reduction_for_spell` when the caster
    /// controls the source and is about to cast their `nth` spell this turn
    /// (i.e. `Player.spells_cast_this_turn == nth - 1`). Generic-only.
    CostReductionNthSpell { filter: SelectionRequirement, nth: u32, amount: u32 },
    /// "The first creature spell you cast each turn costs `amount` less"
    /// (Conduit of Ruin). Unlike `CostReductionNthSpell` (which keys off the
    /// total spell count), this gates on the controller's *creature*-spell
    /// count for the turn (`Player.creatures_cast_this_turn == 0`).
    /// Generic-only.
    CostReductionFirstCreatureSpell { amount: u32 },
    /// "The first instant or sorcery spell you cast each turn costs `amount`
    /// less" (Melek, Reforged Researcher). Gates on the controller's
    /// instant/sorcery-spell count for the turn
    /// (`Player.instants_or_sorceries_cast_this_turn == 0`). Generic-only.
    CostReductionFirstInstantOrSorcery { amount: u32 },
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
    /// Grand-Arbiter-style tax: spells matching `filter` cast by an opponent of
    /// the source's controller cost `amount` more (Sphinx's Decree, Thalia of
    /// Traben-for-opponents). Unlike `AdditionalCost`, the source's controller
    /// is exempt. Applied in `extra_cost_for_spell`.
    OpponentSpellsCostMore { filter: SelectionRequirement, amount: u32 },
    /// Jubilant-Skybonder-style "spells your opponents cast that target a
    /// [`target_filter`] permanent you control cost `amount` more" — a
    /// continuous target-tax read off the source's controller. Evaluated in
    /// `extra_cost_for_spell` against the spell's chosen target; the tax only
    /// applies to spells cast by an opponent of the source's controller.
    TaxOpponentSpellsTargeting { target_filter: SelectionRequirement, amount: u32 },
    /// Card-intrinsic "This spell costs {X} less to cast, where X is the
    /// greatest power among creatures you control" (The Great Henge). Read by
    /// `cost_reduction_for_spell` off the *spell being cast* (not battlefield
    /// permanents), so it only discounts its own cast. Generic-only; clamped by
    /// `ManaCost::reduce_generic`. No continuous-layer effect.
    SelfCostReducedByGreatestPower,
    /// Card-intrinsic "This spell costs {X} less to cast, where X is the total
    /// power of creatures you control" (Ghalta, Primal Hunger). Read by
    /// `cost_reduction_for_spell` off the *spell being cast*. Generic-only;
    /// clamped by `ManaCost::reduce_generic`. No continuous-layer effect.
    SelfCostReducedByTotalPower,
    /// Card-intrinsic "This spell costs {1} less to cast for each creature card
    /// in your graveyard" (Ghoultree). Read by `cost_reduction_for_spell` off
    /// the *spell being cast*. Generic-only; clamped by `ManaCost::reduce_generic`.
    SelfCostReducedPerCreatureInGraveyard,
    /// Card-intrinsic "This spell costs {`per`} less to cast for each card in
    /// your graveyard matching `filter`" — the generalized sibling of
    /// `SelfCostReducedPerCreatureInGraveyard`. Serpent of the Pass ({1} less
    /// per noncreature, nonland card). Generic-only; clamped by the caller.
    SelfCostReducedPerGraveyardCardMatching {
        filter: SelectionRequirement,
        per: u32,
    },
    /// Card-intrinsic "This spell costs {`per`} less to cast for each permanent
    /// you control matching `filter`" — Affinity for [type] (Allies at Last —
    /// Affinity for Allies). Generic-only; clamped by the caller.
    SelfCostReducedPerPermanentMatching {
        filter: SelectionRequirement,
        per: u32,
    },
    /// Card-intrinsic "This spell costs {1} less to cast for each card type
    /// among cards in your graveyard" (Emrakul, the Promised End). Distinct
    /// card types, not card count. Generic-only; clamped by the caller.
    SelfCostReducedPerCardTypeInGraveyard,
    /// Card-intrinsic "This spell costs {X} less to cast, where X is the
    /// total mana value of noncreature artifacts you control" (Metalwork
    /// Colossus). Generic-only; clamped by the caller.
    SelfCostReducedByNoncreatureArtifactMv,
    /// "You don't lose the game for having 0 or less life. As long as you
    /// have 0 or less life, all damage is dealt to you as though its source
    /// had infect." (Phyrexian Unlife.) Gates the CR 704.5a loss SBA and
    /// flips both player-damage funnels to poison at ≤ 0 life.
    ControllerDoesntLoseFromLife,
    /// Card-intrinsic "This spell costs {amount} less to cast if a creature died
    /// this turn" (Bone Picker). Generic-only; clamped by `ManaCost::reduce_generic`.
    SelfCostReducedIfCreatureDiedThisTurn { amount: u32 },
    /// Card-intrinsic "This spell costs {X} less to cast, where X is your
    /// Domain" (CR 702.43 — Leyline Binding). Read by `cost_reduction_for_spell`
    /// off the *spell being cast*; the count is the distinct basic land types
    /// among the caster's lands (0–5). Generic-only; clamped by
    /// `ManaCost::reduce_generic`. No continuous-layer effect.
    SelfCostReducedByDomain { per: u32 },
    /// Card-intrinsic "This spell costs {X} less to cast, where X is the number
    /// of differently named lands you control" (Fungal Colossus). Read by
    /// `cost_reduction_for_spell` off the *spell being cast*. Generic-only;
    /// clamped by `ManaCost::reduce_generic`. No continuous-layer effect.
    SelfCostReducedByDistinctLandNames,
    /// Card-intrinsic "This spell costs {amount} less to cast during your turn"
    /// (Mental Modulation). Read by `cost_reduction_for_spell` off the spell
    /// being cast when the caster is the active player. Generic-only; clamped by
    /// `ManaCost::reduce_generic`. No continuous-layer effect.
    SelfCostReducedDuringYourTurn { amount: u32 },
    /// Card-intrinsic "This spell costs {X} less to cast, where X is your
    /// devotion to `colors`" (Theros — Daybreak Chimera, etc.). Read by
    /// `cost_reduction_for_spell` off the *spell being cast*; the count is the
    /// caster's devotion (each colored pip in those colors among permanents
    /// they control, CR 700.5). Generic-only; clamped by
    /// `ManaCost::reduce_generic`. No continuous-layer effect.
    SelfCostReducedByDevotion { colors: Vec<crate::mana::Color> },
    /// "This spell costs {N} less to cast for each card you've discarded
    /// this turn" (Hollow One). Card-intrinsic; read by
    /// `cost_reduction_for_spell` off `Player.cards_discarded_this_turn`.
    SelfCostReducedPerDiscardThisTurn { per: u32 },
    /// "This spell costs {N} less to cast for each creature you attacked with
    /// this turn" (Search Party Captain). Card-intrinsic; read by
    /// `cost_reduction_for_spell` off `Player.creatures_attacked_this_turn`.
    /// Generic-only; clamped by `ManaCost::reduce_generic`. No layer effect.
    /// When `all_players` is true the count spans every player's attackers
    /// ("for each creature that attacked this turn" — Witchstalker Frenzy),
    /// otherwise just the caster's.
    SelfCostReducedPerCreatureAttackedThisTurn {
        per: u32,
        #[serde(default)]
        all_players: bool,
    },
    /// CR 702.125 — Undaunted: "This spell costs `per` less to cast for each
    /// opponent." Read off the spell being cast in `cost_reduction_for_spell`.
    /// Generic-only.
    SelfCostReducedPerOpponent { per: u32 },
    /// "This spell costs {N} less to cast for each other spell cast this
    /// turn" (Thrasta). Counts every player's casts; this spell isn't cast
    /// yet at cost time, so no self-exclusion is needed. Generic-only.
    SelfCostReducedPerSpellCastThisTurn { per: u32 },
    /// Card-intrinsic "This spell costs `amount` less to cast if you control a
    /// permanent matching each of `filters`" (Of One Mind — a Human creature
    /// *and* a non-Human creature). Read by `cost_reduction_for_spell` off the
    /// *spell being cast*; the discount applies only when every filter has at
    /// least one match among the caster's permanents. Generic-only.
    SelfCostReducedIfControlEach { filters: Vec<SelectionRequirement>, amount: u32 },
    /// Card-intrinsic "This spell costs `amount` less to cast if `condition`"
    /// (Gigastorm Titan — you've cast another spell this turn; Lashwhip Predator
    /// — your opponents control 3+ creatures). The predicate is evaluated at
    /// cost time with the caster as controller. Generic-only, clamped.
    SelfCostReducedIf { condition: Predicate, amount: u32 },
    /// "Each player can't cast more than one spell each turn" (Rule of Law,
    /// Eidolon of Rhetoric, Archon of Emeria). Enforced at the central
    /// `perform_action` cast gate against `Player.spells_cast_this_turn`.
    OneSpellPerTurn,
    /// "Each player can't cast more than one noncreature spell each turn"
    /// (Deafening Silence). Enforced at the central `perform_action` cast gate
    /// against `Player.noncreature_spells_cast_this_turn`.
    OneNoncreatureSpellPerTurn,
    /// "Each player who has cast a nonartifact spell this turn can't cast
    /// additional nonartifact spells" (Ethersworn Canonist). Enforced at the
    /// central `perform_action` cast gate against
    /// `Player.nonartifact_spells_cast_this_turn`.
    OneNonartifactSpellPerTurn,
    /// "Each spell costs {N} more to cast except during its controller's turn"
    /// (Defense Grid). A generic-mana tax folded into `extra_cost_for_spell`,
    /// skipped when the caster is the active player.
    SpellsCostMoreExceptOnControllerTurn { amount: u32 },
    /// CR 104.3c override — "If you would draw a card while your library has
    /// no cards in it, you win the game instead" (Laboratory Maniac, Jace,
    /// Wielder of Mysteries, Thassa's Oracle's gate). Consulted by
    /// `lose_to_empty_draw`.
    WinInsteadOfDrawFromEmpty,
    /// CR 104.3d — "You can't lose the game and your opponents can't win the
    /// game" (Platinum Angel). Consulted by the SBA loss checks,
    /// `lose_to_empty_draw`, and the win/lose one-shot effects.
    ControllerCantLoseGame,
    /// CR 104.3d flip side — "You can't win the game and your opponents
    /// can't lose the game" (Abyssal Persecutor).
    ControllerCantWinGame,
    /// CR 614 — "Damage that would reduce your life total to less than 1
    /// reduces it to 1 instead" (Worship, gated on controlling a creature
    /// via `requires_creature`). Applied at the damage-to-player life sites;
    /// the damage is still dealt (triggers fire), only the life change is
    /// clamped.
    DamageWontReduceControllerLifeBelowOne { requires_creature: bool },
    /// CR 601.2b — card-intrinsic optional additional cost: "you may sacrifice
    /// any number of creatures; this spell costs {N} less to cast for each."
    /// `per` is the per-creature generic reduction. Carried on the spell's own
    /// `static_abilities`; cast via `GameAction::CastSpellSacrificeReduce`
    /// (Awaken the Blood Avatar). No continuous-layer effect.
    SacrificeCostReduction { per: u32 },
    /// "This spell costs {amount} less to cast if it's bargained" (CR 702.176 —
    /// Ice Out, Johann's Stopgap). Read by `cast_spell_bargain` when the
    /// Bargain cost is actually paid. No continuous-layer effect.
    BargainCostReduction { amount: u32 },
    /// Leyline-of-Sanctity-style "you have hexproof": opponents can't
    /// target the source's controller with spells or abilities they
    /// control. Checked by `check_target_legality` for `Target::Player(_)`.
    ControllerHasHexproof,
    /// Glaring Spotlight — "creatures your opponents control with hexproof can
    /// be the targets of spells and abilities you control as though they didn't
    /// have hexproof." The source's controller ignores plain `Hexproof` on
    /// opponents' creatures when checking target legality.
    IgnoreOpponentsCreatureHexproof,
    /// Kaya, Bane of the Dead — "Your opponents and permanents your opponents
    /// control with hexproof can be the targets of spells and abilities you
    /// control as though they didn't have hexproof." The broad sibling of
    /// `IgnoreOpponentsCreatureHexproof`: the source's controller ignores plain
    /// `Hexproof` on opponents' *permanents* (any type) *and* on opponent
    /// players when checking target legality.
    IgnoreOpponentsHexproof,
    /// Tomik, Distinguished Advokist — "Lands you control can't be the targets
    /// of spells or abilities your opponents control." Read at the targeting
    /// gate. (The printed rider covering land cards in graveyards and the
    /// play-from-graveyard lock are omitted — battlefield lands only.)
    LandsUntargetableByOpponents,
    /// Ashiok, Dream Render — "Spells and abilities your opponents control can't
    /// cause their controller to search their library." While any player
    /// controls this static, that player's opponents' searches of their own
    /// library find nothing (CR 701.19 "can't search"). Checked in the
    /// `Effect::Search` resolver alongside Shadow of Doubt's turn-wide lock.
    OpponentsCantSearchLibraries,
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
    /// CR 614 — life-gain bonus replacement: while active, when a targeted
    /// player *would* gain life, they gain that much plus `amount` instead
    /// (Honor Troll: "If you would gain life, you gain that much life plus 1
    /// instead."). Consulted in `adjust_life` for positive deltas. Per
    /// CR 119.10 a gain of 0 isn't a gain, so the bonus only applies on a
    /// genuine positive delta.
    LifeGainBonus { target: PlayerStaticTarget, amount: i32 },
    /// CR 614 — life-gain multiplier replacement: while active, when a targeted
    /// player *would* gain life, they gain `factor` times that much instead
    /// (Rhox Faithmender / Boon Reflection: "you gain twice that much life
    /// instead"). Consulted in `adjust_life` for positive deltas, applied
    /// before any additive `LifeGainBonus`. Multiple multipliers compound.
    LifeGainMultiplier { target: PlayerStaticTarget, factor: i32 },
    /// CR 121.2a / 614 — draw replacement: while active, when the source's
    /// controller would draw a card, they draw two instead (Thought
    /// Reflection, Alhammarret's Archive). Consulted per draw event in
    /// `draw_one`; the extra draw is not itself re-doubled by the same
    /// pass (CR 614.5), though stacked doublers each apply once.
    ControllerDrawsDoubled,
    /// Like `ControllerDrawsDoubled` but only while `condition` holds for the
    /// source's controller ("Max speed — if you would draw a card, draw two
    /// cards instead" — Vnwxt, Verbose Host).
    ControllerDrawsDoubledIf { condition: Predicate },
    /// CR 614 — Notion Thief: if an *opponent* of the source's controller would
    /// draw a card except the first one they draw in each of their draw steps,
    /// that player skips the draw and the source's controller draws instead.
    /// Consulted in `draw_one`, exempting the turn-based draw-step draw.
    OpponentExtraDrawsRedirected,
    /// CR 121.2a / 614 — "If you would draw a card while you have no cards in
    /// hand, instead draw `extra` additional card(s) and lose `life_loss` life"
    /// (Blood Scrivener). Consulted in `draw_one` only when the source's
    /// controller's hand is empty at draw time; the replacement draws are not
    /// re-replaced (guarded by the draw-replacement reentrancy flag).
    EmptyHandDrawBonus { extra: u32, life_loss: u32 },
    /// CR 701.34 / 614 — "If you would proliferate, proliferate twice
    /// instead" (Tekuthal, Inquiry Dominus). Consulted per `Effect::Proliferate`
    /// resolution for the source's controller; n copies → 2^n proliferations.
    ProliferateTwice,
    /// CR 614 — Melira, the Living Cure: if the source's controller would get
    /// one or more poison counters, they get one instead and can't get more
    /// this turn. Consulted in the `add_poison` funnel.
    PoisonCappedAtOnePerTurn,
    /// "You can't get poison counters" (Melira, Sylvok Outcast). Consulted in
    /// the `add_poison` funnel.
    PlayerCannotGetPoison,
    /// "Creatures you control can't have -1/-1 counters put on them"
    /// (Melira, Sylvok Outcast) — the full-lock sibling of
    /// `MinusCounterReduction`.
    NoMinusCountersOnYourCreatures,
    /// CR 614.9 — damage redirection: all damage that would be dealt to the
    /// source's controller or another permanent they control is dealt to the
    /// source instead (Palisade Giant). Applied once per damage event
    /// (CR 614.5). Combat damage aimed at the controller's *other creatures*
    /// isn't redirected (blocker damage keeps its normal path).
    RedirectDamageToSelf,
    /// CR 614.9 — "All damage that would be dealt to you is dealt to equipped
    /// creature instead" (Pariah's Shield). Only player-directed damage is
    /// redirected, and only to the creature this Equipment is attached to.
    RedirectControllerDamageToEquippedCreature,
    /// Codie's lock: the source's controller can't cast permanent spells
    /// (creature/artifact/enchantment/planeswalker). Checked at the main
    /// cast gate in `cast_spell`.
    ControllerCantCastPermanentSpells,
    /// "Noncreature spells with mana value `min_mana_value` or greater can't be
    /// cast" and (when `or_has_x`) "noncreature spells with {X} in their mana
    /// costs can't be cast." Global — locks every player while any permanent
    /// has it (Gaddock Teeg). Checked at the main cast gate in `cast_spell`.
    NoncreatureSpellsCantBeCastIf { min_mana_value: u32, or_has_x: bool },
    /// "Noncreature spells with mana value equal to the source's chosen number
    /// can't be cast" (Sanctum Prelate). Reads the source permanent's
    /// `chosen_number`; inactive until the ETB choice resolves.
    NoncreatureSpellsWithChosenManaValueCantBeCast,
    /// CR 615.12 — while active, damage can't be prevented (global). A
    /// permanent-static sibling of `Effect::DamageCantBePreventedThisTurn`;
    /// `apply_prevention_shields` bypasses all shields while any source on the
    /// battlefield has this. Sulfuric Vortex, Sunspine Lynx, Everlasting Torment.
    DamageCantBePrevented,
    /// CR 615.12 (source-scoped) — "Damage that would be dealt by this
    /// [permanent] can't be prevented." Only bypasses shields when the damage
    /// source is the permanent carrying the static (Excruciator), unlike the
    /// global `DamageCantBePrevented`.
    SourceDamageCantBePrevented,
    /// CR 614 — "If an opponent would lose life during your turn, they lose
    /// twice that much life instead." (Bloodletter of Aclazotz.) A life-loss
    /// doubling replacement scoped to the source controller's turn and their
    /// opponents; consulted by `adjust_life`.
    OpponentLifeLossDoubledDuringYourTurn,
    /// CR 615.12 (scoped) — combat damage dealt by creatures the source's
    /// controller controls can't be prevented (Questing Beast). Bypasses
    /// prevention shields only for damage whose source is a creature that
    /// controller controls.
    ControllerCreaturesCombatDamageCantBePrevented,
    /// CR 615.12 — *all* combat damage can't be prevented, regardless of
    /// controller (Frenzied Baloth). Bypasses prevention shields for any
    /// damage whose source is a creature (the combat approximation shared
    /// with `ControllerCreaturesCombatDamageCantBePrevented`).
    CombatDamageCantBePrevented,
    /// CR 508.1g — creatures can't attack the source's controller (and, when
    /// `protect_planeswalkers`, a planeswalker they control) unless the
    /// attacking player pays `amount` generic mana for each such attacker.
    /// Checked in `declare_attackers`, which sums the tax across every
    /// attacker hitting a protected player/walker and auto-pays it from the
    /// active player's mana pool (rejecting the declaration if it can't be
    /// covered). `amount` is a `Value` evaluated with the static's controller
    /// as "you", so fixed taxes use `Value::Const(n)` (Ghostly Prison /
    /// Propaganda / Windborn Muse = 2, Baird = 1, all `protect_planeswalkers`
    /// per card) while dynamic ones scale off the controller's board — Sphere
    /// of Safety = number of enchantments you control. Copies stack. Paid from
    /// the pool, auto-tapping mana sources for any shortfall.
    AttackTaxToController { amount: Value, protect_planeswalkers: bool },
    /// CR 508.1 — absolute attack prohibition. "Creatures can't attack you"
    /// (and, when `protect_planeswalkers`, a planeswalker you control) — a hard
    /// no, not a tax. Checked in `declare_attackers`. Blazing Archon,
    /// Peacekeeper-style locks. (`AttackTaxToController` is the payable sibling.)
    CreaturesCantAttackController { protect_planeswalkers: bool },
    /// CR 509.1d — block tax. "Creatures can't block unless their controllers
    /// pay `amount` for each of those creatures." Checked in `declare_blockers`,
    /// which sums the tax across every active source and auto-pays it from each
    /// blocking player's mana pool (rejecting the declaration if it can't be
    /// covered). `only_while_attacking` gates the static on the source itself
    /// being an attacking creature this combat (Archangel of Tithes — the
    /// block-tax half is live only while it attacks); `false` makes it an
    /// always-on enchantment-style tax. Paid from the pool, auto-tapping mana
    /// sources for any shortfall.
    BlockTaxToController {
        amount: Value,
        #[serde(default)]
        only_while_attacking: bool,
    },
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
    /// "You may cast [filter] spells as though they had flash." Sigarda's
    /// Aid (Auras + Equipment). Consulted at the cast-timing gate.
    ControllerSpellsHaveFlash { filter: SelectionRequirement },
    /// Card-intrinsic "You may cast this spell as though it had flash if
    /// `condition` holds" — consulted at the cast-timing gate against the
    /// card being cast (not a battlefield permanent). Serpent of the Pass's
    /// "if 3+ Lessons in your graveyard". `condition` is evaluated from the
    /// caster's perspective.
    SelfFlashIf { condition: crate::effect::Predicate },
    /// "Each instant and sorcery card in your graveyard has flashback. The
    /// flashback cost is equal to that card's mana cost." Lier, Disciple of
    /// the Drowned. Consulted by the flashback-cast path and surfaced in the
    /// graveyard view so the UI offers the recast.
    GraveyardInstantsSorceriesHaveFlashback,
    /// CR 702.97 — "Each creature card in your graveyard has scavenge. The
    /// scavenge cost is equal to its mana cost." Varolz, the Scar-Striped. The
    /// granted scavenge is surfaced as a virtual `from_graveyard` activated
    /// ability at index ≥ printed_count on each of the controller's graveyard
    /// creature cards.
    GraveyardCreaturesHaveScavenge,
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
    /// The counter-half of CR 614.16, matching Doubling Season / Branching
    /// Evolution-class permanents. Read at `Effect::AddCounter` resolution
    /// time: each active `DoubleCounters` permanent the controller has on the
    /// battlefield doubles the counter count (2 doublers → 4×, …). Composes
    /// multiplicatively with `DoubleTokens` for cards that print both halves
    /// (Doubling Season itself ships both static abilities).
    DoubleCounters,
    /// CR 614.16 additive variant — "If one or more +1/+1 counters would be
    /// put on a creature you control, that many *plus one* are put on it
    /// instead." Hardened Scales / Conclave Mentor / Kalonian Hydra-class.
    /// Each active copy adds one to a +1/+1 placement onto the controller's
    /// creatures; applied before any `DoubleCounters` multiplier.
    ExtraPlusOneCounters,
    /// CR 614.16 self-scoped additive variant — "If one or more +1/+1 counters
    /// would be put on THIS, that many plus one are put on it instead." Mowu,
    /// Loyal Companion. Like `ExtraPlusOneCounters` but only for placements onto
    /// the static's own source permanent; applied before any doubler.
    ExtraPlusOneCounterOnSelf,
    /// CR 614.16 multiplicative variant scoped to +1/+1 counters — "If one or
    /// more +1/+1 counters would be put on a creature you control, twice that
    /// many are put on it instead." Branching Evolution / Kami of Whispered
    /// Hopes / The Earth Crystal-class. Unlike `DoubleCounters` (which doubles
    /// *any* counter kind), this only doubles +1/+1 placements onto the
    /// controller's creatures. Composes multiplicatively with `DoubleCounters`.
    DoublePlusOneCounters,
    /// CR 614.16 additive variant for *every* counter kind — "If one or more
    /// counters would be put on an artifact or creature you control, that many
    /// plus one of each of those kinds are put on it instead."
    /// Winding Constrictor-class. Each active copy adds one to a placement of
    /// any kind onto the controller's creatures (the "counters you'd get"
    /// player-counter clause is approximated away); applied alongside
    /// `ExtraPlusOneCounters` before the `DoubleCounters` multiplier.
    ExtraCounterAllKinds,
    /// CR 614 — "If you would get one or more {E}, you get that many plus
    /// `amount` instead." An energy-only gain bonus (Izzet Generatorium), unlike
    /// `ExtraCounterAllKinds` which boosts every counter kind.
    EnergyGainBonus { amount: u32 },
    /// CR 614.2 — "If a source would deal damage … it deals double that
    /// damage instead." A *global* damage-replacement (Furnace of Rath,
    /// Gratuitous Violence-class, Fiery Emancipation as ×2 stacking): read
    /// off the battlefield by `GameState::damage_doublers`, every active
    /// instance doubles the dealt amount (2 → 4×, …). Currently wired for
    /// the non-combat `deal_damage_to_from` path; combat-damage doubling is
    /// tracked in TODO.md under CR 614.2.
    DoubleDamageDealt,
    /// CR 614.5 — "If a source would deal damage to a permanent or player,
    /// it deals half that damage, rounded down, instead." (Ghosts of the
    /// Innocent.) Read by `GameState::damage_halvers`; applied after any
    /// doublers at both damage funnels.
    HalveDamageDealt,
    /// CR 615 — "Prevent all combat damage that would be dealt to this." A
    /// self-static the combat-damage resolver honors (zeroing damage marked on
    /// the permanent) unless combat damage can't be prevented this turn (615.12).
    /// Fog Bank, Guard Gomazoa.
    PreventAllCombatDamageToThis,
    /// "Prevent all damage that would be dealt to this permanent" — the
    /// combat+noncombat superset of `PreventAllCombatDamageToThis`, consulted
    /// on both damage funnels. Wrap in `WhileYourTurn` for turn-gated
    /// protection (Gideon Blackblade during your turn).
    PreventAllDamageToThis,
    /// "Prevent all combat damage that would be dealt to this creature by
    /// creatures blocking it." The narrower sibling of
    /// `PreventAllCombatDamageToThis` — only strikes-back from this creature's
    /// blockers are blanked (damage taken while *it* blocks still applies).
    /// Read in the combat-damage resolver's attacker-takes-from-blocker branch.
    /// Armored Transport.
    PreventAllCombatDamageToThisFromBlockers,
    /// CR 614.5 — "If a source would deal damage to an opponent or a
    /// permanent an opponent controls, it deals double that damage instead."
    /// (Gisela, Blade of Goldnight.) Scoped to the static's controller's
    /// opponents; consulted by `GameState::scale_damage_to`.
    DoubleDamageToOpponents,
    /// CR 614.5 — "If a creature you control that entered this turn would deal
    /// damage, it deals twice that much damage instead." (Neriv, Heart of the
    /// Storm.) Combat and noncombat alike; consulted by `scale_damage_to`.
    DoubleDamageFromCreaturesEnteredThisTurn,
    /// CR 614.2 — "If a creature you control would deal damage to a permanent or
    /// player, it deals double that damage instead." (Gratuitous Violence.)
    /// Source-controller-restricted (unlike the global `DoubleDamageDealt`);
    /// combat and noncombat alike; consulted by `scale_damage_to`.
    DoubleDamageFromControlledCreatures,
    /// CR 614.5 Hellbent — "As long as you have no cards in hand, if a source you
    /// control would deal damage to a permanent or player, it deals double that
    /// damage instead." (Anthem of Rakdos.) Any source (not just creatures),
    /// gated on the static's controller having an empty hand; `scale_damage_to`.
    DoubleYourSourcesDamageWhileHellbent,
    /// CR 614.5 — "If a source you control would deal *noncombat* damage to an
    /// opponent or a permanent an opponent controls, it deals double that
    /// damage instead." (Solphim, Mayhem Dominus.) Noncombat-only and also
    /// requires the *source* to be controlled by the static's controller, so
    /// it's applied in the `deal_damage_to_from` funnel rather than
    /// `scale_damage_to`.
    DoubleNoncombatDamageToOpponents,
    /// "If a source you control would deal noncombat damage to an opponent or a
    /// permanent an opponent controls, it deals that much damage plus `amount`
    /// instead." The additive sibling of `DoubleNoncombatDamageToOpponents`,
    /// applied in the same `deal_damage_to_from` funnel. When `while_revolt`,
    /// gated on CR 702.139 revolt (a permanent left the battlefield under the
    /// static controller's control this turn). Aether Revolt.
    NoncombatDamageToOpponentsBonus { amount: u32, while_revolt: bool },
    /// CR 614.5/615 — "If a source would deal damage to you or a permanent
    /// you control, prevent half that damage, rounded up." (Gisela.) The
    /// remainder is floor(amount/2) — same arithmetic as a halver, scoped
    /// to the static's controller's side.
    HalveDamageToYou,
    /// CR 614.5 — "If a [color] source you control would deal damage to an
    /// opponent or a permanent an opponent controls, it deals that much
    /// damage plus `amount` instead." (Torbran, Thane of Red Fell.)
    /// `source_color: None` matches any source you control. Consulted by
    /// `GameState::scale_damage_to` (additive bonus applied before the
    /// doublers/halvers).
    AddDamageToOpponents { source_color: Option<crate::mana::Color>, amount: u32 },
    /// CR 614.5 — like `AddDamageToOpponents` but the bonus equals the number of
    /// `kind` counters on this static's own source permanent, read live at
    /// damage time (Fated Firepower — "+ the number of fire counters on this").
    AddDamageToOpponentsPerCounter { kind: crate::card::CounterType },
    /// CR 614.x — "If a [color] source would deal damage to a player, it deals
    /// that much damage plus `amount` to that player instead." Unlike
    /// `AddDamageToOpponents`, this matches *any* controller's source of the
    /// color and *every* player (Tok-Tok, Volcano Born). Applied in
    /// `scale_damage_to` before the doublers/halvers.
    AddDamageFromColorToPlayers { color: crate::mana::Color, amount: u32 },
    /// CR 614.5 — "If a creature you control of one of these types would deal
    /// damage to a permanent or player, it deals that much damage plus `amount`
    /// instead." (Valley Flamecaller — Lizard/Mouse/Otter/Raccoon.) Keyed on the
    /// damage *source*'s computed creature types + controller; applied in
    /// `scale_damage_to` before the doublers/halvers.
    ControlledCreatureTypesDealExtraDamage {
        types: Vec<crate::card::CreatureType>,
        amount: u32,
    },
    /// CR 614.x — "If another [color] source you control would deal damage to a
    /// permanent or player, it deals that much damage plus `amount` instead."
    /// (Jaya, Venerated Firemage.) Unlike `AddDamageToOpponents`, it hits *any*
    /// permanent or player (not only opponents') and excludes the static's own
    /// source permanent. Applied in `scale_damage_to` before the doublers.
    YourColorSourcesDealExtraDamage { color: crate::mana::Color, amount: u32 },
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
    /// "If a triggered ability of an Ally you control triggers, that ability
    /// triggers an additional time." Katara, the Fearless. Read at trigger
    /// dispatch via `ally_trigger_extra_fires`; adds one fire per copy for
    /// any non-ETB or ETB trigger whose source is an Ally the controller
    /// controls.
    DoubleControllerAllyTriggers,
    /// CR 603.x — "If a triggered ability of a [type] you control triggers,
    /// that ability triggers an additional time." The generalization of
    /// `DoubleControllerAllyTriggers` to an arbitrary set of creature types.
    /// Read at trigger dispatch via `subtype_trigger_extra_fires`; adds one
    /// fire per copy for any trigger whose source is a creature the
    /// controller controls of one of `types`. When `exclude_source` is set,
    /// the doubler never doubles its own triggers (Harmonic Prodigy's
    /// "Shaman or *another* Wizard" — the source Wizard is itself, so its
    /// own prowess isn't doubled by the Wizard clause).
    /// Drivnod, Carnage Dominus — "If a creature dying causes a triggered
    /// ability of a permanent you control to trigger, that ability triggers
    /// an additional time." Read at trigger dispatch off the
    /// `triggered_by_death` candidate flag.
    DoubleControllerDeathTriggers,
    /// Isshin, Two Heavens as One / Windcrag Siege (Mardu) — "If a creature
    /// attacking causes a triggered ability of a permanent you control to
    /// trigger, that ability triggers an additional time." Read at trigger
    /// dispatch off the `triggered_by_attack` candidate flag (and the
    /// self-source attack path in `combat.rs`).
    DoubleControllerAttackTriggers,
    DoubleControllerTriggersOfType {
        types: Vec<crate::card::CreatureType>,
        #[serde(default)]
        exclude_source: bool,
    },
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
        /// Extends suppression to entering **artifacts** as well (Doorkeeper
        /// Thrull — "Artifacts and creatures entering don't cause abilities to
        /// trigger"). `#[serde(default)]`.
        #[serde(default)]
        also_artifacts: bool,
    },
    /// "Each other planeswalker you control has the loyalty abilities of
    /// [this]." (Kasmina, Enigma Sage.) Read by `activate_loyalty_ability`,
    /// which appends the source's loyalty abilities (indices ≥ printed
    /// count) to every other friendly planeswalker.
    OtherPlaneswalkersHaveSourceLoyaltyAbilities,
    /// "[This] has all loyalty abilities of all other planeswalkers on the
    /// battlefield" (Nicol Bolas, Dragon-God). Self-scoped: read by
    /// `effective_loyalty_abilities`, which appends every *other* battlefield
    /// planeswalker's loyalty abilities (any controller) past the printed count.
    HasAllOtherPlaneswalkerLoyaltyAbilities,
    /// Ichormoon Gauntlet — "Planeswalkers you control have '[0]: Proliferate'
    /// and '[−12]: Take an extra turn after this one.'" Appended past the
    /// printed abilities at loyalty-activation time.
    PlaneswalkersHaveLoyaltyAbilities { abilities: Vec<LoyaltyAbility> },
    /// CR 401.6-adjacent: the controller may play/cast cards matching
    /// `filter` from the top of their library (Courser of Kruphix /
    /// Oracle of Mul Daya lands, Mystic Forge artifact+colorless spells).
    /// Checked in `play_land_with_face` and `cast_spell`.
    PlayFromLibraryTop { filter: crate::card::SelectionRequirement },
    /// Like `PlayFromLibraryTop`, but capped at one cast/play from the library
    /// top per turn (Johann, Apprentice Sorcerer — "Once each turn, you may cast
    /// an instant or sorcery spell from the top of your library"). Tracked via
    /// `Player.cast_from_library_top_this_turn`.
    PlayFromLibraryTopOncePerTurn { filter: crate::card::SelectionRequirement },
    /// Like `PlayFromLibraryTop`, but a *spell* cast this way pays life equal to
    /// its mana value instead of its mana cost (Bolas's Citadel). Lands are
    /// still played for free. Read by the cast-from-top path.
    PlayFromLibraryTopPayLife { filter: crate::card::SelectionRequirement },
    /// "Creatures you control with +1/+1 counters on them have all
    /// activated abilities of all creature cards exiled with [the
    /// source]." Agatha's Soul Cauldron — the exile-zone sibling of
    /// `HasActivatedAbilitiesOfGraveyardCreatures`.
    CounteredCreaturesHaveAbilitiesOfExiledWithSource,
    /// "As long as the top card of your library is a [filter] card, this
    /// creature has all activated abilities of that card" (Conspicuous
    /// Snoop). A self-only grant surfaced by `granted_abilities_for`,
    /// reading the controller's live library top.
    HasActivatedAbilitiesOfLibraryTop { filter: SelectionRequirement },
    /// Grafdigger's Cage: creature cards in graveyards and libraries can't
    /// enter the battlefield, and players can't cast spells from graveyards
    /// or libraries.
    GraveyardLibraryLockdown,
    /// Kunoros, Hound of Athreos: the graveyard-only sibling — creature
    /// cards in graveyards can't enter the battlefield and players can't
    /// cast spells from graveyards (library plays unaffected).
    GraveyardLockdown,
    /// Soulless Jailer: permanent cards in graveyards can't enter the
    /// battlefield, and players can't cast noncreature spells from
    /// graveyards or exile.
    GraveyardExileLockdown,
    /// Underworld Breach: each nonland card in the controller's graveyard
    /// has escape — its own mana cost plus exile `exile_count` other cards.
    GraveyardCardsHaveEscape { exile_count: u32 },
    /// Six: during the controller's turn, nonland permanent cards in their
    /// graveyard have retrace (CR 702.55).
    GraveyardPermanentsHaveRetraceDuringYourTurn,
    /// The Ozolith: when a creature its controller controls leaves the
    /// battlefield with counters on it, those counters move onto this
    /// permanent (applied at the leave funnels).
    CollectsLeaverCounters,
    /// Karn, the Great Creator: activated abilities of artifacts the
    /// controller's opponents control can't be activated.
    OpponentsCantActivateArtifactAbilities,
    /// Ulamog, the Defiler: the source has annihilator X, where X is the
    /// number of +1/+1 counters on it (computed-keyword injection).
    AnnihilatorPerPlusOneCounter,
    /// CR 614.10 — skip-step replacement: "[players/you] skip [their/your]
    /// <step> step(s)." The skipped step never occurs — no turn-based
    /// actions, step triggers, or priority (a skipped untap also skips
    /// phasing). Eon Hub (upkeep, all players), Stasis (untap, all players).
    SkipStep {
        step: crate::TurnStep,
        /// `true` = every player's step; `false` = only the controller's.
        all_players: bool,
    },
    /// Ensnaring Bridge — creatures with power greater than the number of
    /// cards in this permanent's controller's hand can't attack. Enforced
    /// in `declare_attackers` against layer-computed power.
    AttackPowerCapByControllerHand,
    /// CR 305.7 — "[lands] are <type>" statics. `replace: true` strips the
    /// other land types and all abilities (Blood Moon / Magus of the Moon —
    /// the intrinsic mana ability follows the computed type); `false` adds
    /// the type alongside (Urborg, Tomb of Yawgmoth).
    LandTypeChanger {
        applies_to: Selector,
        land_type: crate::card::LandType,
        replace: bool,
    },
    /// CR 305.7 — "Lands you control are the [chosen] type in addition to
    /// their other types." Reads the source permanent's `chosen_land_type`
    /// (stamped by `Effect::ChooseBasicLandTypeForSource` as it entered) and
    /// adds that basic land type to every land the source's controller
    /// controls (layer-4, additive). No-op until a type is chosen. Realmwright.
    LandsYouControlAreChosenType,
    /// CR 305.7 — a `LandTypeChanger` gated on the source carrying at least `n`
    /// counters of `kind`. The layer effect only materializes while the
    /// threshold holds (Zhao, the Moon Slayer — "As long as Zhao has a
    /// conqueror counter on him, nonbasic lands are Mountains").
    LandTypeChangerWhileCounters {
        applies_to: Selector,
        land_type: crate::card::LandType,
        replace: bool,
        kind: crate::card::CounterType,
        n: u32,
    },
    /// "Abilities you activate that aren't mana abilities cost {N} less to
    /// activate. This effect can't reduce the mana in that cost to less
    /// than one mana." Zirda, the Dawnwaker (generic-only reduction).
    ActivationCostReduction { amount: u32 },
    /// DFT — "Exhaust abilities of other permanents you control cost {N} less
    /// to activate" (Boom Scholar). Generic-only, never below one mana; applies
    /// only to CR 702.177 exhaust abilities of the controller's *other*
    /// permanents.
    OtherExhaustActivationCostReduction { amount: u32 },
    /// CR 702.6 — "You may activate equip abilities any time you could cast an
    /// instant" (Leonin Shikari). Lifts the sorcery-speed gate on the
    /// controller's `GameAction::Equip`.
    ControllerEquipAtInstantSpeed,
    /// CR 702.6 — "Equip costs you pay cost {N} less" (Auriok Steelshaper,
    /// Brass Squire-style discounts). Reduces the controller's equip-cost
    /// generic by `amount`, never below the colored portion.
    EquipCostReduction { amount: u32 },
    /// CR 602.5 / 614 — "Activated abilities cost {N} more to activate
    /// unless they're mana abilities." Applies to every player's
    /// activations (Suppression Field).
    ActivationTax { amount: u32 },
    /// CR 606 — "Loyalty abilities of planeswalkers your opponents control
    /// cost {N} more to activate" (Eidolon of Obstruction). Summed across the
    /// taxers an activating player's opponents control and paid as extra
    /// generic mana at `activate_loyalty_ability`.
    OpponentLoyaltyActivationTax { amount: u32 },
    /// Tithe Taker — "During your turn, spells your opponents cast cost {amount}
    /// more and non-mana abilities your opponents activate cost {amount} more."
    /// Only bites on the source controller's turn and never taxes the
    /// controller's own spells/abilities. Spell half read in
    /// `extra_cost_for_spell`; ability half in `effective_ability_mana_cost`.
    OpponentActivityCostsMoreOnYourTurn { amount: u32 },
    /// "During each of your turns, you may cast a permanent spell of each
    /// permanent type from your graveyard." Muldrotha, the Gravetide
    /// (checked in `cast_spell`; per-type-per-turn tally on the player).
    MayCastPermanentsFromGraveyard,
    /// "You may cast [filter] spells from your graveyard by paying `life`
    /// life in addition to paying their other costs. If you cast a spell this
    /// way, it enters with a finality counter." Noctis, Prince of Lucis
    /// (checked in `cast_spell`; the life is paid on a successful cast and
    /// the finality counter stamped via `CardInstance.pending_etb_counters`).
    GraveyardCastWithLifeSurcharge { filter: SelectionRequirement, life: u32 },
    /// CR 401.5: the controller plays with the top card of their library
    /// revealed (surfaced to every seat via `PlayerView.library_top`).
    TopOfLibraryRevealed,
    /// "Creatures you control of the chosen type get +P/+T" — a tribal anthem
    /// keyed to the source permanent's `chosen_creature_type` (set at ETB via
    /// `Effect::NameCreatureType`). Resolved live in `gather_continuous_effects`
    /// (reads the source's chosen type), emitting a layer-7 pump over the
    /// controller's matching creatures. `exclude_source: true` skips the source
    /// itself ("**other** creatures …" — Adaptive Automaton); `false` includes
    /// it (Patchwork Banner). No effect while no type has been chosen.
    /// `opponents: true` instead applies the (typically negative) modifier to
    /// each *opponent's* creatures of the chosen type (Plague Engineer).
    /// When `per_counter: Some(kind)`, `power`/`toughness` are each multiplied
    /// by the number of `kind` counters on the source — "+1/+1 for each charge
    /// counter on this" (Door of Destinies).
    AnthemForChosenType {
        power: i32,
        toughness: i32,
        #[serde(default)]
        exclude_source: bool,
        #[serde(default)]
        opponents: bool,
        #[serde(default)]
        per_counter: Option<crate::card::CounterType>,
    },
    /// "Creatures you control of the chosen color get +P/+T" — the color
    /// sibling of `AnthemForChosenType`, keyed to `CardInstance.chosen_color`
    /// (stamped by `Effect::ChooseColorForSelf`). Heraldic Banner. Resolved
    /// live in `gather_continuous_effects`; no effect while no color is chosen.
    AnthemForChosenColor { power: i32, toughness: i32 },
    /// "[filter] you control get +P/+T [and have keywords]" — a fixed-filter
    /// team anthem (Balthier and Fran → Vehicles; Ardyn, the Usurper → Demons).
    /// Unlike `AnthemForChosenType` (keyed to a chosen creature type stamped at
    /// ETB) the filter is printed on the card. Resolved live in
    /// `gather_continuous_effects`: a layer-7 pump plus one layer-6 keyword
    /// grant per `keywords` entry over the controller's permanents matching
    /// `filter` (via `AffectedPermanents::CardMatch`). `opponents: true` targets
    /// each opponent's matching permanents instead.
    AnthemForFilter {
        filter: SelectionRequirement,
        #[serde(default)]
        power: i32,
        #[serde(default)]
        toughness: i32,
        #[serde(default)]
        keywords: Vec<Keyword>,
        #[serde(default)]
        opponents: bool,
        /// "During your turn, [filter] you control have …" — the anthem only
        /// applies while its controller is the active player (Yuna, Hope of
        /// Spira). Defaults false (always on) for snapshot back-compat.
        #[serde(default)]
        only_your_turn: bool,
        /// "[filter] get +P/+T *for each [kind] counter on this*"
        /// (Chitterspitter's acorn-scaled Squirrel anthem). Multiplies
        /// `power`/`toughness` by the source's live counter count.
        #[serde(default)]
        scale_by_counters_on_self: Option<crate::card::CounterType>,
    },
    /// "As long as [condition], this has [keyword]" — the self keyword-grant
    /// sibling of `SetBasePtIf` / `PumpSelfIf`, gated on a live `Predicate`
    /// (source as ability context). Freya Crescent's "During your turn, Freya
    /// has flying" (`Predicate::IsTurnOf(You)`). Emits a layer-6 self
    /// `AddKeyword` while the predicate holds. Unlike `SelfHasKeywordWhile`
    /// (a `SelectionRequirement` over the source's own characteristics) this
    /// reads game state via the `Predicate` machinery.
    SelfHasKeywordIf { keyword: Keyword, condition: Predicate },
    /// "As long as [condition], this is an artifact creature" — the type-line
    /// analogue of `SelfHasKeywordIf`. Emits a layer-4 `AddCardType(Creature)`
    /// self-effect while the predicate holds (Midnight Mangler — a Vehicle that
    /// is a creature during turns other than its controller's; the printed P/T
    /// already carry the stats). Read via live game state.
    SelfIsCreatureIf { condition: Predicate },
    /// "You and creatures you control have protection from the chosen card
    /// type" (Serra's Emissary). The type is the source's
    /// `chosen_card_type`; grants `Keyword::ProtectionFromCardType` to the
    /// controller's creatures at layer 6, and the player-side half is read
    /// directly at the spell/ability targeting and damage gates.
    YouAndCreaturesProtectionFromChosenCardType,
    /// "Planeswalkers' loyalty abilities you activate cost an additional
    /// [+N] to activate" (Carth the Lion). Shifts every loyalty ability's
    /// cost by +N for the controller.
    LoyaltyAbilitiesCostExtra(i32),
    /// CR 614 — "If a modular triggered ability would put one or more +1/+1
    /// counters on a creature you control, that many plus N are put instead"
    /// (Zabaz, the Glimmerwasp).
    ModularBonusCounters(u32),
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
    /// "`spell_filter` spells you cast have Affinity for `permanent_filter`"
    /// (CR 702.40) — the general sibling of `GrantAffinityToISSpells` whose
    /// spell scope isn't fixed to instants/sorceries. Tezzeret, Master of the
    /// Bridge grants creature and planeswalker spells affinity for artifacts.
    /// Generic-only via the `ManaCost::reduce_generic` clamp.
    GrantAffinityToSpells {
        spell_filter: SelectionRequirement,
        permanent_filter: SelectionRequirement,
    },
    /// "Instant and sorcery spells you cast have storm." (CR 702.40.)
    /// Read at CAST time in `cast_spell`'s intrinsic-storm branch, so the
    /// copy count is the true storm count — spells cast before this one
    /// this turn — and responses cast after it can't inflate the number
    /// (unlike a resolution-time `Value::StormCount` trigger). Powers
    /// Prismari, the Inspiration.
    GrantStormToISSpells,
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
    /// "Each other creature you control of the chosen type enters with an
    /// additional counter of `kind` on it" (Metallic Mimic). Keyed to the
    /// source permanent's `chosen_creature_type` (set at ETB via
    /// `Effect::NameCreatureType`). Unlike `ExtraEtbCountersForCreatureCasts`
    /// this fires for *any* matching creature entry the controller makes
    /// (casts, tokens, reanimation), gated on the entering creature being a
    /// different object whose creature types include the chosen type. Read at
    /// both ETB-counter sites (`stack.rs` spell-resolve and `movement.rs`
    /// move-to-battlefield) via `chosen_type_etb_counter_specs`.
    ChosenTypeEntersWithCounter { kind: CounterType },
    /// "Creatures you control of the chosen type have [keyword]" — a keyword
    /// grant keyed to the source permanent's `chosen_creature_type` (set at ETB
    /// via `Effect::NameCreatureType`). Sibling of `AnthemForChosenType`,
    /// emitting a layer-6 `AddKeyword` over the controller's matching creatures.
    /// Steely Resolve (shroud), Kindred Boon (indestructible). `opponents: true`
    /// applies to each opponent's matching creatures instead.
    GrantKeywordToChosenType {
        keyword: crate::card::Keyword,
        #[serde(default)]
        opponents: bool,
    },
    /// "Creature spells you cast of the chosen type cost {amount} less" — a
    /// generic-only cost reduction keyed to the source permanent's
    /// `chosen_creature_type` (set at ETB via `Effect::NameCreatureType`).
    /// Applied in `cost_reduction_for_spell_zoned`; Changeling spells qualify.
    /// Urza's Incubator ({2}), Herald's Horn ({1}).
    ChosenTypeSpellCostReduction { amount: u32 },
    /// "Each other [creature_type] creature you control enters with an
    /// additional `kind` counter" (Oona's Blackguard). Fixed-type sibling of
    /// `ChosenTypeEntersWithCounter`; Changeling entrants count.
    TypeEntersWithCounter { creature_type: crate::card::CreatureType, kind: CounterType },
    /// "Each other [creature_type] creature you control enters with an
    /// additional `kind` counter for each permanent matching `per` you
    /// control" (Giada, Font of Hope — `per` = "Angel you control", so a new
    /// Angel enters with +1/+1 counters equal to your Angel count). Read at the
    /// same ETB-counter sites as `TypeEntersWithCounter`; the entrant is
    /// excluded from the count.
    TypeEntersWithCountersPerControlled {
        creature_type: crate::card::CreatureType,
        kind: CounterType,
        per: crate::card::SelectionRequirement,
    },
    /// "Each creature you control that's one of `types` enters with `amount`
    /// additional `kind` counters on it" (Arlinn, Voice of the Pack — Wolves
    /// and Werewolves enter with an extra +1/+1). A flat per-entry bonus, read
    /// at the same ETB-counter sites.
    TypedCreaturesEnterWithExtraCounter {
        types: Vec<crate::card::CreatureType>,
        kind: CounterType,
        amount: u32,
    },
    /// "Each other creature you control enters with a number of additional
    /// `kind` counters equal to this creature's power" (Master Biomancer). Read
    /// at the same ETB-counter sites as `TypeEntersWithCounter` via
    /// `chosen_type_etb_counter_specs`; the source's live power (layers applied)
    /// sets the count, so a pumped Biomancer grants more.
    OtherCreaturesEnterWithCountersEqualToSourcePower { kind: CounterType },
    /// Strict Proctor — "Whenever a permanent entering causes a
    /// triggered ability to trigger, counter that ability unless its
    /// controller pays {amount}." Read at ETB-trigger dispatch time
    /// (both the self-source path in `fire_self_etb_triggers` and the
    /// unified dispatcher in `dispatch_triggers_for_events`). For each
    /// ETB trigger pushed onto the stack, the trigger's controller is
    /// asked yes/no whether to pay `amount` generic mana from their
    /// pool. On yes + affordable: pay, fire the trigger normally. On
    /// no/unaffordable: the ability is countered (CR 701.5a) — it never
    /// fires and the trigger's source permanent is untouched. The
    /// AutoDecider opts in to paying when the controller has enough mana
    /// floated; otherwise it declines. Stacks across multiple Strict
    /// Proctors (one tax per source).
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
    /// Like `CastHandSpellsFree` but restricted to hand spells matching
    /// `filter` — "You may cast Dragon spells without paying their mana costs"
    /// (Dracogenesis). Consulted by `player_casts_hand_spells_free`.
    CastFilteredSpellsFree { filter: crate::card::SelectionRequirement },
    /// Aluren (CR 601 alt-timing) — "Any player may cast creature spells
    /// with mana value `max_mv` or less without paying their mana cost and
    /// as though they had flash." Read by
    /// `GameState::player_casts_cheap_creature_free` from the free-cast
    /// action; grants instant-speed timing for the qualifying creature.
    AnyoneCastsCheapCreaturesFree { max_mv: u32 },
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
        /// Optional gate on the granting source's controller (Hellbent —
        /// "has '{B}: Regenerate this creature'" only while hand-empty).
        /// Evaluated from the source's controller; `None` = always granted.
        #[serde(default)]
        condition: Option<crate::effect::Predicate>,
    },
    /// CR 605.1b — triggered mana ability: "Whenever [a matching land] is
    /// tapped for mana, its controller adds [extra]." Doesn't use the stack;
    /// resolved immediately at the mana-ability fast path. `enchanted_only`
    /// restricts to the land this source enchants (Wild Growth);
    /// `filter` matches the tapped land (Vernal Bloom's Forests).
    ExtraManaOnLandTap {
        #[serde(default)]
        enchanted_only: bool,
        filter: SelectionRequirement,
        extra: ExtraManaKind,
        /// Only fires while the source's controller is the monarch (Regal
        /// Behemoth). Defaults to false via `#[serde(default)]`.
        #[serde(default)]
        while_monarch: bool,
    },
    /// "Each [filter] card in each player's hand has typecycling [cost]"
    /// (Homing Sliver's slivercycling grant). Consulted by `landcycle_card`
    /// and the hand-affordance view for cards whose printed keywords lack
    /// cycling; `search` is the fetch filter (CR 702.29e).
    GrantTypecyclingToHandCards {
        filter: SelectionRequirement,
        cost: crate::mana::ManaCost,
        search: SelectionRequirement,
    },
    /// "All [filter] permanents have '[triggered ability]'" (CR 613 layer 6
    /// grant — Kataki, War's Wage's "All artifacts have 'At the beginning of
    /// your upkeep, sacrifice this artifact unless you pay {1}'"). The
    /// ability fires as though printed on each matching permanent, so
    /// `YourControl`/`SelfSource` scopes read that permanent's controller.
    GrantTriggeredAbility {
        filter: SelectionRequirement,
        ability: Box<TriggeredAbility>,
    },
    /// Alpine Moon — lands matching the source's chosen name
    /// (`CardInstance.named_card`) that opponents control lose all land
    /// types and abilities. Pair with a `GrantActivatedAbility` over
    /// `NamedBySource` lands for the "{T}: Add one mana of any color" half.
    NamedLandsNeutralized,
    /// Ultima, Origin of Oblivion — every land carrying a blight counter
    /// loses all land types and abilities while this source remains. Pair
    /// with a `GrantActivatedAbility` over `WithCounter(Blight)` lands for
    /// the "{T}: Add {C}" half.
    BlightedLandsNeutralized,
    /// Quina, Qu Gourmet — "If one or more tokens would be created under your
    /// control, those tokens plus a [definition] token are created instead."
    /// Applied once per resolution that minted 1+ tokens for the controller
    /// (CR 614.13-style single application).
    TokenCreationAddsToken { definition: crate::card::TokenDefinition },
    /// Chatterfang — "If one or more tokens would be created under your
    /// control, those tokens plus that many [definition] tokens are created
    /// instead." Like `TokenCreationAddsToken` but scaled to the number of
    /// tokens the resolution minted for the controller.
    TokenCreationAddsTokenPerToken { definition: crate::card::TokenDefinition },
    /// Academy Manufactor — "If you would create a Clue, Food, or Treasure
    /// token, instead create one of each." Applied at the mint funnel with a
    /// CR 614.5 reentrancy guard (the extra mints aren't re-replaced).
    ClueFoodTreasureMintsOneOfEach,
    /// Necrotic Ooze — "As long as this is on the battlefield, it has all
    /// activated abilities of all creature cards in all graveyards." Surfaced
    /// by `granted_abilities_for` (which walks every graveyard for creature
    /// cards and clones their battlefield-usable activated abilities onto the
    /// source). A self-only grant: only the permanent carrying this static
    /// gains the abilities.
    HasActivatedAbilitiesOfGraveyardCreatures,
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
    /// CR 700.5 — "Your devotion to each color and each combination of colors
    /// is increased by one." Altar of the Pantheon. Each permanent the player
    /// controls carrying this static adds 1 to every non-empty devotion query.
    DevotionBonus,
    /// CR 615 — "If a creature would deal combat damage to this creature,
    /// prevent that damage and put a +1/+1 counter on this creature."
    /// Ironscale Hydra. A self-only combat-damage replacement consulted at the
    /// creature-vs-creature damage sites.
    PreventCombatDamageToSelfAndGrow,
    /// CR 614 — "If damage would be dealt to this creature, put that many
    /// +1/+1 counters on it instead." Phytohydra. A true replacement (not
    /// prevention, so it fires even when damage can't be prevented), consulted
    /// at both the combat and noncombat self-damage sites; grows by the full
    /// amount rather than a single counter.
    ReplaceDamageToSelfWithCounters,
    /// CR 614 — "If this creature would deal combat damage to a player,
    /// instead put that many +1/+1 counters on it and that player mills that
    /// many cards." Szadek, Lord of Secrets. A dealer-side combat-damage
    /// replacement consulted in the attack-a-player branch.
    CombatDamageToPlayerBecomesCountersAndMill,
    /// CR 615 — "Prevent all damage that would be dealt to attacking
    /// creatures you control." Iroas, God of Victory. Consulted at both the
    /// combat strike-back and the shared non-combat damage funnel.
    PreventDamageToYourAttackers,
    /// CR 615 — "Prevent all damage that would be dealt to you." Glacial Chasm.
    /// Consulted at the player-directed branch of the shared damage funnel
    /// (combat and noncombat alike), unless prevention is shut off this turn.
    PreventAllDamageToController,
    /// CR 615 — "Prevent all noncombat damage that would be dealt to creatures
    /// you control." Mark of Asylum. Consulted at the shared (noncombat) damage
    /// funnel for creature targets; combat damage is marked on a separate path
    /// and is unaffected.
    PreventNoncombatDamageToYourCreatures,
    /// CR 615 — "Prevent all noncombat damage that would be dealt to you and
    /// [other] permanents you control." Broader than
    /// `PreventNoncombatDamageToYourCreatures`: it also shields the controller
    /// (player) and their noncreature permanents. The Wanderer. Consulted at
    /// the noncombat funnel for both player and permanent targets.
    PreventNoncombatDamageToYouAndYourPermanents,
    /// CR 615 — "Prevent all damage that would be dealt to creature tokens you
    /// control" (Emmara Tandris). Consulted on both the combat and noncombat
    /// damage paths for token creatures controlled by this static's controller.
    PreventAllDamageToYourCreatureTokens,
    /// CR 615 — "Prevent all damage that the source would deal to creatures of
    /// the given color" (Indentured Oaf — prevents its own damage to red
    /// creatures). Keyed on the damage source having this static.
    PreventThisDamageToColor(crate::mana::Color),
    /// CR 615 — "Prevent all damage that would be dealt to creatures you
    /// control by sources you control." Light of Sanction. Consulted at both
    /// the combat strike-back and the shared non-combat damage funnel; the
    /// source and target must share a controller who has this static.
    PreventDamageToYourCreaturesFromYourSources,
    /// CR 615 self-replacement: "If noncombat damage would be dealt to
    /// this creature, prevent that damage. Put a +1/+1 counter on this
    /// creature for each 1 damage prevented this way." Checked in the
    /// noncombat damage funnel (`deal_damage_to_from`), after scaling,
    /// unless prevention is off (CR 615.12). Stormwild Capridor.
    PreventNoncombatDamageToSelfAddCounters,
    /// CR 106.4 override — "If you would lose unspent mana, that mana
    /// becomes colorless instead." Kruphix, God of Horizons. Consulted at
    /// the step/phase pool-empty sites.
    UnspentManaBecomesColorless,
    /// CR 500.4 exception — "Players don't lose unspent mana as steps and
    /// phases end" (Upwelling). Every player's pool survives step/phase ends
    /// with its colors intact (it still empties at end of turn via cleanup's
    /// separate path only if no keeper remains — Upwelling has no such carve-out,
    /// so pools persist across the whole game while it's in play).
    ManaPoolsNeverEmpty,
    /// CR 106.4 exception — "You don't lose unspent [color] mana as steps and
    /// phases end" (Omnath, Locus of Mana keeps green). The controller keeps
    /// that color's mana; all other mana empties normally.
    UnspentColorManaPersists(crate::mana::Color),
    /// "As long as this card is in your graveyard and you control a
    /// [land subtype], creatures you control have [keyword]" — the Judgment
    /// Incarnation cycle (Anger, Wonder, Brawn, Valor, Filth). Zone-special:
    /// gathered from graveyards, not the battlefield.
    GraveyardAnthem { land_type: crate::card::LandType, keyword: Keyword },
    /// "[Filter] spells you control can't be countered" — Destiny Spinner
    /// (creature and enchantment spells). Read at cast time by
    /// `caster_grants_uncounterable_with_x` off the caster's battlefield.
    SpellsUncounterable { filter: SelectionRequirement },
    /// "Creature spells can't be countered" — a *symmetric* uncounterable
    /// static (any player's copy protects every player's creature spells,
    /// unlike `SpellsUncounterable` which is scoped to the caster's own
    /// permanents). Leyline of Lifeforce.
    CreatureSpellsCantBeCountered,
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
    /// CR 509.1a — "Tapped creatures you control can block as though they were
    /// untapped." While the controller has a permanent carrying this static, a
    /// tapped creature they control is a legal blocker (Masako the Humorless).
    TappedCreaturesCanBlock,
    /// "Spells with the chosen name cost {N} more to cast" — reads the
    /// source's `named_card` (stamped at ETB via `Effect::NameCard`).
    /// Disruptor Flute. Folded into `extra_cost_for_spell`.
    NamedSpellTax { amount: u32 },
    /// Meddling Mage — spells with the source's `named_card` can't be cast.
    NamedSpellCantBeCast,
    /// Ashiok's Erasure — the controller's *opponents* can't cast spells with
    /// the source's `named_card` (the exiled card's name). Unlike
    /// `NamedSpellCantBeCast` (which locks everyone), this is controller-scoped.
    OpponentsCantCastNamed,
    /// Dress Down / Humility-lite — all creatures lose all abilities
    /// (layer 6 `RemoveAllAbilities`).
    CreaturesLoseAllAbilities,
    /// Lantern of Insight — every player plays with their library top
    /// revealed (the all-players sibling of `TopOfLibraryRevealed`).
    AllLibraryTopsRevealed,
    /// "Each opponent's maximum hand size is reduced by N" (Jin-Gitaxias,
    /// Core Augur). Folded into `effective_max_hand_size` for every seat
    /// not on the source controller's team.
    OpponentsMaxHandSizeReduced(u32),
    /// "Your maximum hand size is N" (Necrodominance, Cursed Rack-likes
    /// scoped to the controller). Overrides the base seven; the smallest
    /// active override wins.
    ControllerMaxHandSize(u32),
    /// CR 305 / 718 — "You may play lands from your graveyard." Crucible of
    /// Worlds, Ramunap Excavator. Read by the land-play legality + the
    /// `PlayLandFromGraveyard` action: a land in the controller's graveyard
    /// becomes a legal land play (still bound by the one-land-per-turn cap).
    MayPlayLandsFromGraveyard,
    /// "As long as this card is in your graveyard, if you would learn, you may
    /// instead return this card to the battlefield." Consulted at the top of
    /// `Effect::Learn`; no layer effect. — Retriever Phoenix.
    MayReturnFromGraveyardInsteadOfLearn,
    /// CR 701.10f — "If you tap a permanent for mana, it produces twice as
    /// much of that mana instead." Mana Reflection. Each instance the
    /// controller of the resolving mana ability has on the battlefield
    /// doubles the produced pip count (2 instances → 4×, …). Read by
    /// `mana_production_multiplier_for` just before a mana ability resolves.
    ManaProductionDoubled,
    /// CR 614.5 — "If you tap a permanent for mana, it produces three times
    /// as much of that mana instead." Nyxbloom Ancient. Composes with
    /// `ManaProductionDoubled` multiplicatively (2 + 3 → 6×).
    ManaProductionTripled,
    /// "If damage would be dealt to this permanent while it has a [kind]
    /// counter on it, prevent that damage and remove that many [kind]
    /// counters from it." Polukranos, Unchained. Consulted at both damage
    /// funnels for the source permanent itself.
    PreventDamageByRemovingCounters { kind: crate::card::CounterType },
    /// Mindsplice Apparatus — "[filter] spells you cast cost {1} less to
    /// cast for each [kind] counter on this artifact."
    CostReductionPerCounterOnSource {
        filter: SelectionRequirement,
        kind: crate::card::CounterType,
    },
    /// Mirran Safehouse — "has all activated abilities of all land cards in
    /// all graveyards." Sibling of
    /// `HasActivatedAbilitiesOfGraveyardCreatures`.
    HasActivatedAbilitiesOfGraveyardLands,
    /// Phyrexian Vindicator — "If damage would be dealt to this creature,
    /// prevent it. When damage is prevented this way, this creature deals
    /// that much damage to any other target" (auto-picked, preferring an
    /// opposing creature, then the opponent).
    PreventDamageToThisRedirect,
    /// Cursed Totem / Damping Matrix — "Activated abilities of creatures
    /// can't be activated unless they're mana abilities." Global lock
    /// checked in `activate_ability` (sibling of
    /// `ArtifactActivatedAbilitiesLocked`).
    CreatureActivatedAbilitiesLocked,
    /// "You may activate abilities of creatures you control as though those
    /// creatures had haste." Exempts the controller's creatures from the
    /// CR 602.5g summoning-sickness gate on {T}/{Q} costs (Tyvar, Jubilant
    /// Brawler; Thousand-Year Elixir kin).
    ControllerCreatureAbilitiesAsThoughHaste,
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
    /// `colors: Some(..)` restricts the redirect to cards of those printed
    /// colors (Sanctifier en-Vec's black/red filter). Consulted at every
    /// graveyard-placement site via `graveyard_exiled_for`.
    ExileCardsBoundForGraveyard {
        opponents_only: bool,
        /// Restrict the redirect to the static's controller's own cards
        /// (Necrodominance's "if a card or token would be put into YOUR
        /// graveyard").
        #[serde(default)]
        own_only: bool,
        #[serde(default)]
        colors: Option<Vec<crate::mana::Color>>,
        /// Restrict the redirect to cards whose printed types intersect this
        /// list (Dryad Militant / Scavenging Ooze's instant-and-sorcery-only
        /// graveyard hate). `None` = every card type.
        #[serde(default)]
        card_types: Option<Vec<crate::card::CardType>>,
        /// Stamp a void counter on each card this redirect exiles
        /// (Dauthi Voidwalker — its sac ability frees one for a free play).
        #[serde(default)]
        void_counter: bool,
    },
    /// CR 614 — "If [a matching permanent] would be put into a graveyard,
    /// put it on top of its owner's library instead." Consulted in
    /// `remove_from_battlefield_to_graveyard_raw`; the printed "may" is
    /// auto-taken. Pulmonic Sliver ("All Slivers have …").
    DiesToLibraryTopInstead { filter: crate::card::SelectionRequirement },
    /// CR 614.5 — "If an opponent would mill one or more cards, they mill
    /// twice that many cards instead." (Bruvac the Grandiloquent.) Consulted
    /// by `GameState::mill_count_for` at every mill site.
    OpponentMillDoubled,
    /// CR 701.19c — "If an opponent would search a library, that player
    /// searches the top `count` cards of that library instead." Consulted by
    /// `Effect::Search`: an opponent of this static's controller only sees
    /// candidates among the top N. Aven Mindcensor.
    OpponentsSearchTopN { count: u32 },
    /// "Players can't search libraries. Any player may pay {amount} for that
    /// player to ignore this effect until end of turn." Leonin Arbiter. The
    /// searcher auto-pays from floating mana (once per turn per player); an
    /// unpayable tax makes the search find nothing.
    SearchTax { amount: u32 },
    /// CR 502.3 — "Untap all permanents you control during each other player's
    /// untap step." Seedborn Muse / Prophet of Kruphix. Consulted by
    /// `do_untap`: while the active player is *not* this static's controller,
    /// the controller's permanents untap alongside the active player's (subject
    /// to the same Stun / `PreventUntap` / exert gates). No layer effect.
    UntapAllYoursEachUntapStep,
    /// CR 502.3 — "Untap this permanent during each other player's untap step."
    /// The source untaps itself on every untap step it doesn't already untap on
    /// (i.e. whenever the active player is someone else). Thousand Moons
    /// Infantry. Consulted by `do_untap` in a follow-up pass.
    UntapSelfEachUntapStep,
    /// CR 502.3 — an Aura's "Enchanted [permanent] untaps during each other
    /// player's untap step" (Urban Burgeoning). The source's attached host
    /// untaps on every untap step its controller doesn't already untap on.
    /// Consulted by `do_untap` alongside `UntapSelfEachUntapStep`.
    UntapAttachedEachUntapStep,
    /// CR 502.3 — "Players can't untap more than one nonbasic land during their
    /// untap steps." Winter Moon / Mana Web-style lock. Consulted by `do_untap`:
    /// each untapping player untaps at most one nonbasic land (the rest stay
    /// tapped). Global — applies to every player, not just the controller.
    MaxOneNonbasicLandUntap,
    /// "Permanents you control have: whenever one or more +1/+1 counters are put
    /// on this permanent, put an additional +1/+1 counter on it. This ability
    /// triggers only once each turn." Cursed Wombat. Consulted in the
    /// `Effect::AddCounter` +1/+1 path: after counters land on a permanent whose
    /// controller has this static, one extra is added the first time each turn
    /// (guarded by `permanents_amplified_counter_this_turn`).
    CounterAmplifierOncePerTurn,
    /// CR 614 — "If a nontoken creature an opponent controls would die, exile
    /// it instead." Consulted in `remove_from_battlefield_to_graveyard`: an
    /// opponent's nontoken creature bound for a graveyard from the battlefield
    /// is routed to exile. When `when_you_do` is `Some`, that reflexive effect
    /// is pushed onto the stack for the static's controller each time the
    /// redirect fires ("When you do, …"). Valentin, Dean of the Vein.
    ExileDyingOpponentCreatures {
        #[serde(default)]
        when_you_do: Option<Box<Effect>>,
    },
    /// CR 702.15 — "Instant and sorcery spells you control have lifelink."
    /// Consulted in the non-combat damage path (`deal_damage_to_from`): when
    /// an instant/sorcery spell whose controller has this static deals damage,
    /// that controller gains that much life. Radiant Scrollwielder.
    YourInstantSorcerySpellsHaveLifelink,
    /// "Spells and abilities your opponents control can't cause you to
    /// sacrifice permanents." Consulted in the `Effect::Sacrifice` resolver:
    /// when an opponent-controlled effect would force this static's controller
    /// to sacrifice, that player is skipped. Sigarda, Host of Herons; Tamiyo,
    /// Collector of Tales (the discard half is a separate gap).
    OpponentsCantMakeYouSacrifice,
    /// "Spells and abilities your opponents control can't cause you to
    /// discard cards." (Tamiyo, Collector of Tales.) Consulted by the
    /// `Effect::Discard` resolver; the sacrifice half is the sibling above.
    OpponentsCantMakeYouDiscard,
    /// CR 614.5 — "If one or more -1/-1 counters would be put on a creature
    /// you control, that many minus one are put on it instead." Vizier of
    /// Remedies. Stacks: each copy shaves one more counter.
    MinusCounterReduction,
    /// "Your opponents can't cast spells during your turn." Voice of
    /// Victory. Gated at the cast-action dispatch.
    OpponentsCantCastDuringYourTurn,
    /// "During your turn, your opponents can't cast spells or activate
    /// abilities of artifacts, creatures, or enchantments." Grand Abolisher.
    /// Blocks both the cast dispatch (like `OpponentsCantCastDuringYourTurn`)
    /// and the activated-ability dispatch for A/C/E sources.
    OpponentsCantActDuringYourTurn,
    /// CR 601 — "Your opponents can't cast spells from anywhere other than
    /// their hands." Drannith Magistrate. Checked in `cast_from_zone_blocked`
    /// for every non-hand cast path (flashback / escape / retrace / free-cast).
    OpponentsCantCastFromAnywhereButHand,
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
    /// CR 107.17 — the untap symbol `{Q}`: the source must be tapped and
    /// untaps as a cost (Pili-Pala). Shares `{T}`'s CR 602.5g/h
    /// summoning-sickness gate.
    #[serde(default)]
    pub untap_self_cost: bool,
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
    /// "Pay X life" as a variable additional cost, where X is the activation's
    /// chosen `x_value` (CR 107.16). The body reads the same X via
    /// `Value::XFromCost`. Mirrors `energy_x_cost` but drains life. Powers
    /// Krumar Initiate's `{X}{B}, {T}, Pay X life: This creature endures X.`
    /// Mutually independent from the fixed `life_cost`. Defaults to false.
    #[serde(default)]
    pub x_life_cost: bool,
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
    /// True if this ability is activated from the **exile zone** (the card
    /// must be owned by the activator). Squee, the Immortal's "cast from
    /// exile" half rides the same Move-to-battlefield approximation as
    /// `from_graveyard`. Defaults to false via `#[serde(default)]`.
    #[serde(default)]
    pub from_exile: bool,
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
    /// When true, `exile_other_filter`'s count is the activation's X value
    /// rather than the fixed `u32` (which is then ignored). Mirrors
    /// `sac_other_x`. Used by "{X}, {T}, Exile X cards from your graveyard:"
    /// costs — Necropolis Fiend's "-X/-X".
    #[serde(default)]
    pub exile_other_x: bool,
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
    /// "This ability costs {1} less to activate for each [filter] you
    /// control" — generic-only reduction counted off the activator's
    /// battlefield at payment time (the Kamigawa channel lands' legendary
    /// discount). Defaults to None via `#[serde(default)]`.
    #[serde(default)]
    pub cost_reduction_per: Option<SelectionRequirement>,
    /// "This ability costs {1} less to activate for each [filter] card in your
    /// graveyard" — generic-only reduction counted off the activator's
    /// graveyard at payment time (Battlefield Butcher). Defaults to None.
    #[serde(default)]
    pub cost_reduction_per_graveyard: Option<SelectionRequirement>,
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
    /// When true, `sac_other_filter`'s count is the activation's X value
    /// ("Sacrifice X [filter]:" costs — Lonis, Genetics Expert). The X is
    /// threaded to the effect as `Value::XFromCost`.
    #[serde(default)]
    pub sac_other_x: bool,
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
    /// Optional cost: tap *N* untapped, different permanents the activator
    /// controls matching this filter (CR 602.5b "Tap N untapped … you
    /// control:" costs — Heritage Druid's "Tap three untapped Elves you
    /// control: Add {G}{G}{G}."). The count-bearing sibling of
    /// `tap_other_filter`; rejected when fewer than `u32` untapped matches
    /// exist. Auto-picks the lowest-power matches. Defaults to None.
    #[serde(default)]
    pub tap_n_filter: Option<(SelectionRequirement, u32)>,
    /// Optional cost: return a *different* permanent the activator controls
    /// matching this filter to its owner's hand (CR 602.5b "Return a [filter]
    /// you control to its owner's hand:" costs). Mirrors `sac_other_filter`
    /// but bounces rather than sacrifices. Powers Quirion Ranger ("Return a
    /// Forest you control …"), Wirewood Symbiote ("Return an Elf you control
    /// …"), Scryb Ranger, etc. The auto-picker takes the lowest-power match
    /// (or first matching noncreature) so higher-value permanents stay put.
    /// Rejected with `GameError::SelectionRequirementViolated` when nothing
    /// matches. The bounce is applied after tap / mana / life payments
    /// succeed but before the effect resolves. Defaults to None.
    #[serde(default)]
    pub bounce_other_filter: Option<(SelectionRequirement, u32)>,
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
    /// Optional cost: pay `X` energy, where `X` is the activation's chosen
    /// `x_value` (CR 107.16 + a variable cost). The target filter reads the
    /// same `X` via `ManaValueExactlyXFromCost`, so "Pay X {E}: return a
    /// creature card with mana value X" (Chthonian Nightmare) is one field
    /// plus that filter. Mutually exclusive with a fixed `energy_cost`.
    #[serde(default)]
    pub energy_x_cost: bool,
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
    /// When set with `discard_cost`, the discarded cards must all share a
    /// name (Sphinx of the Chimes — "Discard two nonland cards with the same
    /// name:"). The pre-flight picks a name with enough matching copies.
    #[serde(default)]
    pub discard_cost_same_name: bool,
    /// "Discard your hand" as an activation cost (Diamond Lion / Lion's Eye
    /// Diamond). The whole hand is discarded, firing discard triggers.
    #[serde(default)]
    pub discard_hand_cost: bool,
    /// "{N} less to activate for each `counter_type` counter on permanents
    /// matching the filter" (Deepwood Denizen — {1} less per +1/+1 counter
    /// on creatures you control). Generic-only reduction.
    #[serde(default)]
    pub cost_reduction_per_counter: Option<(crate::card::CounterType, SelectionRequirement)>,
    /// Optional cost: remove `count` counters of the given type from the
    /// source permanent (CR 602.5b "Remove a [kind] counter from this:"
    /// cost lines). Modeled as a real cost — not an effect — so the ability
    /// can't be over-activated off the stack (each activation must pay from
    /// the counters present when it's announced). Powers Walking Ballista,
    /// Triskelion, Hangarback Walker (`Remove a +1/+1 counter from this:`).
    /// Applied after tap/mana/life payments but before the effect resolves.
    /// Rejected with `GameError::SelectionRequirementViolated` when the
    /// source lacks enough counters. Defaults to None via `#[serde(default)]`.
    #[serde(default)]
    pub remove_counter_cost: Option<(crate::card::CounterType, u32)>,
    /// "Remove X [kind] counters from this creature:" (Arcbound Javelineer).
    /// X comes from the activation's `x_value`; the pre-flight gate requires
    /// that many counters, and the body reads `Value::XFromCost`.
    #[serde(default)]
    pub remove_counter_x: Option<crate::card::CounterType>,
    /// Optional cost: remove `u32` counters of the named kind (`None` = any
    /// mix of kinds — Tekuthal's "remove three counters") from among
    /// permanents matching the filter the activator controls (CR 602.5b —
    /// "Remove N [kind] counters from among creatures you control:"). Unlike
    /// `remove_counter_cost` the counters may come from any mix of matching
    /// permanents, not just the source. Rejected when the total available is
    /// below the count; the auto-picker drains lowest-value permanents first.
    /// Hopeful Initiate. Defaults to None via `#[serde(default)]`.
    #[serde(default)]
    pub remove_counter_among_filter:
        Option<(Option<crate::card::CounterType>, u32, SelectionRequirement)>,
    /// Variable sibling of `remove_counter_among_filter`: remove `x_value`
    /// counters of the named kind from among permanents matching the filter the
    /// activator controls ("Remove one or more +1/+1 counters from among
    /// creatures you control:" — Ooze Flux). The body reads the count via
    /// `Value::XFromCost`. Rejected when fewer than X (or fewer than one) are
    /// available; the auto-picker drains lowest-value permanents first.
    #[serde(default)]
    pub remove_counter_among_x: Option<(crate::card::CounterType, SelectionRequirement)>,
    /// True if activating this ability returns the source permanent to its
    /// owner's hand as part of the cost (CR 602.5b "Return this … to its
    /// owner's hand:" cost lines). The bounce happens after tap/mana/life
    /// payments succeed but before the effect resolves, mirroring
    /// `sac_cost`. Powers Grinning Ignus (`{R}, Return this to its owner's
    /// hand: Add {C}{C}{R}.`) and Rootha, Mercurial Artist (`{2}, Return
    /// Rootha to its owner's hand: Copy target instant or sorcery spell`).
    ///
    /// Defaults to false via `#[serde(default)]`.
    #[serde(default)]
    pub return_self_cost: bool,
    /// CR 602.5 — "Only your opponents may activate this ability." When true,
    /// the source permanent's controller is barred from activating it; only an
    /// opponent (a player not on the controller's team) may. Powers Detention
    /// Vortex's `{3}: Destroy this Aura` escape clause. Defaults to false.
    #[serde(default)]
    pub opponents_only: bool,
    /// True if activating this ability discards the source from the activator's
    /// hand as part of its cost (CR 602.5b "Discard this card:" cost lines).
    /// Pairs with `from_hand: true`. The discard (hand → graveyard, firing a
    /// `CardDiscarded` event) happens after mana/life payments succeed but
    /// before the effect resolves, mirroring `exile_self_cost`. Powers
    /// Elemental Masterpiece's `{U/R}{U/R}, Discard this card: Create a
    /// Treasure`. Defaults to false.
    #[serde(default)]
    pub discard_self_cost: bool,
    /// CR 702.177 — Exhaust: this activated ability can be activated only
    /// once (per game, not per turn). Tracked per-permanent-instance in
    /// `CardInstance.exhausted_abilities`, which — unlike `once_per_turn_used`
    /// — is never cleared at turn start. Defaults to false.
    #[serde(default)]
    pub exhaust: bool,
    /// "Activate only once." A plain once-per-game gate (Possessed Goat) that
    /// reuses `exhausted_abilities` bookkeeping like `exhaust` but is *not* the
    /// Exhaust keyword — it fires no `ExhaustAbilityActivated` event. Defaults
    /// to false.
    #[serde(default)]
    pub activate_once: bool,
    /// Craft (CR 702.169) — exile `count` *other* objects matching this
    /// filter from among permanents you control and/or cards in your
    /// graveyard, as an additional cost. Pairs with
    /// `Effect::ExileSelfReturnTransformed` (which exiles the source and
    /// returns it transformed). Activate only as a sorcery
    /// (`sorcery_speed: true`). The auto-picker exiles graveyard cards
    /// first, then the lowest-power battlefield permanents, so higher-value
    /// board pieces stay put. Defaults to None.
    #[serde(default)]
    pub craft_exile_cost: Option<(SelectionRequirement, u32)>,
    /// CR 701.67 — Waterbend N as part of this ability's cost ("Waterbend {N}:
    /// …"). The N generic lives in `mana_cost`; this flag marks the ability so
    /// activation accepts waterbend helpers (tap an untapped artifact/creature
    /// you control to pay {1} of the generic, clamped to the generic total).
    /// Defaults to false.
    #[serde(default)]
    pub waterbend: bool,
    /// CR 701.59 — "Collect evidence N" as part of this ability's cost
    /// ("{T}, Collect evidence N: …"). Exiles the cheapest set of graveyard
    /// cards whose total mana value is ≥ N and emits `EvidenceCollected`.
    /// Pre-flight-gated on `graveyard_can_collect_evidence`; paid after
    /// tap/mana/life succeed but before the effect resolves. Powers Forensic
    /// Researcher's tap-untap sibling. Defaults to None.
    #[serde(default)]
    pub collect_evidence_cost: Option<u32>,
    /// Optional cost: exile a spell the activator controls from the stack
    /// matching this filter (CR 602.5b "Exile [a spell] you control:"). The
    /// exiled spell leaves the stack and won't resolve. Powers Nivmagus
    /// Elemental (`Exile an instant or sorcery spell you control: …`). The
    /// auto-picker exiles the top-most (most recently cast) matching spell.
    /// Rejected with `GameError::SelectionRequirementViolated` when no
    /// controlled spell matches. Defaults to None.
    #[serde(default)]
    pub exile_spell_cost: Option<SelectionRequirement>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoyaltyAbility {
    pub loyalty_cost: i32,
    pub effect: Effect,
    /// Variable `-X` loyalty ability (CR 606.5): the player picks X (0..=current
    /// loyalty) on activation, loyalty drops by X, and the body reads X via
    /// `Value::XFromCost`. `loyalty_cost` is ignored when set. — Kasmina.
    #[serde(default)]
    pub x_cost: bool,
}
