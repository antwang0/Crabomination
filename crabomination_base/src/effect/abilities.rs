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
    /// Grant a keyword to everything the selector picks.
    GrantKeyword { applies_to: Selector, keyword: Keyword },
    /// Strip a keyword from matching permanents (CR 613 layer 6) — "creatures
    /// your opponents control lose hexproof and shroud" (Nowhere to Run). A
    /// layer-6 `Modification::RemoveKeyword`, the mirror of `GrantKeyword`.
    LoseKeyword { applies_to: Selector, keyword: Keyword },
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
    /// Card-intrinsic "This spell costs {X} less to cast, where X is the
    /// greatest power among creatures you control" (The Great Henge). Read by
    /// `cost_reduction_for_spell` off the *spell being cast* (not battlefield
    /// permanents), so it only discounts its own cast. Generic-only; clamped by
    /// `ManaCost::reduce_generic`. No continuous-layer effect.
    SelfCostReducedByGreatestPower,
    /// Card-intrinsic "This spell costs {X} less to cast, where X is your
    /// Domain" (CR 702.43 — Leyline Binding). Read by `cost_reduction_for_spell`
    /// off the *spell being cast*; the count is the distinct basic land types
    /// among the caster's lands (0–5). Generic-only; clamped by
    /// `ManaCost::reduce_generic`. No continuous-layer effect.
    SelfCostReducedByDomain { per: u32 },
    /// "This spell costs {N} less to cast for each card you've discarded
    /// this turn" (Hollow One). Card-intrinsic; read by
    /// `cost_reduction_for_spell` off `Player.cards_discarded_this_turn`.
    SelfCostReducedPerDiscardThisTurn { per: u32 },
    /// "Each player can't cast more than one spell each turn" (Rule of Law,
    /// Eidolon of Rhetoric, Archon of Emeria). Enforced at the central
    /// `perform_action` cast gate against `Player.spells_cast_this_turn`.
    OneSpellPerTurn,
    /// CR 104.3c override — "If you would draw a card while your library has
    /// no cards in it, you win the game instead" (Laboratory Maniac, Jace,
    /// Wielder of Mysteries, Thassa's Oracle's gate). Consulted by
    /// `lose_to_empty_draw`.
    WinInsteadOfDrawFromEmpty,
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
    /// CR 121.2a / 614 — draw replacement: while active, when the source's
    /// controller would draw a card, they draw two instead (Thought
    /// Reflection, Alhammarret's Archive). Consulted per draw event in
    /// `draw_one`; the extra draw is not itself re-doubled by the same
    /// pass (CR 614.5), though stacked doublers each apply once.
    ControllerDrawsDoubled,
    /// CR 614.9 — damage redirection: all damage that would be dealt to the
    /// source's controller or another permanent they control is dealt to the
    /// source instead (Palisade Giant). Applied once per damage event
    /// (CR 614.5). Combat damage aimed at the controller's *other creatures*
    /// isn't redirected (blocker damage keeps its normal path).
    RedirectDamageToSelf,
    /// Codie's lock: the source's controller can't cast permanent spells
    /// (creature/artifact/enchantment/planeswalker). Checked at the main
    /// cast gate in `cast_spell`.
    ControllerCantCastPermanentSpells,
    /// CR 615.12 — while active, damage can't be prevented (global). A
    /// permanent-static sibling of `Effect::DamageCantBePreventedThisTurn`;
    /// `apply_prevention_shields` bypasses all shields while any source on the
    /// battlefield has this. Sulfuric Vortex, Sunspine Lynx, Everlasting Torment.
    DamageCantBePrevented,
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
    /// CR 614.5 — "If a source would deal damage to a permanent or player,
    /// it deals half that damage, rounded down, instead." (Ghosts of the
    /// Innocent.) Read by `GameState::damage_halvers`; applied after any
    /// doublers at both damage funnels.
    HalveDamageDealt,
    /// CR 614.5 — "If a source would deal damage to an opponent or a
    /// permanent an opponent controls, it deals double that damage instead."
    /// (Gisela, Blade of Goldnight.) Scoped to the static's controller's
    /// opponents; consulted by `GameState::scale_damage_to`.
    DoubleDamageToOpponents,
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
    /// "Each other planeswalker you control has the loyalty abilities of
    /// [this]." (Kasmina, Enigma Sage.) Read by `activate_loyalty_ability`,
    /// which appends the source's loyalty abilities (indices ≥ printed
    /// count) to every other friendly planeswalker.
    OtherPlaneswalkersHaveSourceLoyaltyAbilities,
    /// CR 401.6-adjacent: the controller may play/cast cards matching
    /// `filter` from the top of their library (Courser of Kruphix /
    /// Oracle of Mul Daya lands, Mystic Forge artifact+colorless spells).
    /// Checked in `play_land_with_face` and `cast_spell`.
    PlayFromLibraryTop { filter: crate::card::SelectionRequirement },
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
    /// "Abilities you activate that aren't mana abilities cost {N} less to
    /// activate. This effect can't reduce the mana in that cost to less
    /// than one mana." Zirda, the Dawnwaker (generic-only reduction).
    ActivationCostReduction { amount: u32 },
    /// CR 602.5 / 614 — "Activated abilities cost {N} more to activate
    /// unless they're mana abilities." Applies to every player's
    /// activations (Suppression Field).
    ActivationTax { amount: u32 },
    /// "During each of your turns, you may cast a permanent spell of each
    /// permanent type from your graveyard." Muldrotha, the Gravetide
    /// (checked in `cast_spell`; per-type-per-turn tally on the player).
    MayCastPermanentsFromGraveyard,
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
    AnthemForChosenType { power: i32, toughness: i32, #[serde(default)] exclude_source: bool },
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
    /// "Spells with the chosen name cost {N} more to cast" — reads the
    /// source's `named_card` (stamped at ETB via `Effect::NameCard`).
    /// Disruptor Flute. Folded into `extra_cost_for_spell`.
    NamedSpellTax { amount: u32 },
    /// Meddling Mage — spells with the source's `named_card` can't be cast.
    NamedSpellCantBeCast,
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
        /// Stamp a void counter on each card this redirect exiles
        /// (Dauthi Voidwalker — its sac ability frees one for a free play).
        #[serde(default)]
        void_counter: bool,
    },
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoyaltyAbility {
    pub loyalty_cost: i32,
    pub effect: Effect,
    /// Variable `-X` loyalty ability (CR 606.5): the player picks X (0..=current
    /// loyalty) on activation, loyalty drops by X, and the body reads X via
    /// `Value::XFromCost`. `loyalty_cost` is ignored when set. — Kasmina.
    #[serde(default)]
    pub x_cost: bool,
}
