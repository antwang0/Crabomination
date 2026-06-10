//! MTG continuous-effect layer system (CR 613).
//!
//! Continuous effects from static abilities and resolved spells/abilities are
//! collected here and applied in layer order to produce a `ComputedPermanent`
//! — the "true" state of each permanent after all effects are considered.
//!
//! Layer order:
//!   1. Copy effects
//!   2. Control-changing effects
//!   3. Text-changing effects
//!   4. Type-changing effects
//!   5. Color-changing effects
//!   6. Ability-adding/removing effects
//!      7a. P/T set by characteristic-defining abilities
//!      7b. P/T set to specific value
//!      7c. P/T modifications (+N/+M)
//!      7d. P/T switching
//!      Counters (+1/+1, -1/-1) are applied after 7c per CR 613.7f.

use crate::card::{
    CardId, CardType, CounterType, CreatureType, Keyword, LandType, SelectionRequirement, Subtypes,
    Supertype,
};
use crate::mana::Color;
use serde::{Deserialize, Serialize};

// ── Duration ──────────────────────────────────────────────────────────────────

/// How long a continuous effect persists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectDuration {
    /// Lasts while the source permanent is on the battlefield (static ability).
    WhileSourceOnBattlefield,
    /// Expires at the next end-of-turn Cleanup step.
    UntilEndOfTurn,
    /// Lasts until the beginning of the next turn.
    UntilNextTurn,
    /// Expires when the current combat phase ends (CR 511.2 — "Effects
    /// that last 'until end of combat' expire at the end of the combat
    /// phase"). Cleared as the end-of-combat step ends. If the effect
    /// is registered outside a combat phase it falls through to the
    /// next combat phase's end-of-combat step.
    UntilEndOfCombat,
    /// Indefinite (e.g. counters, "for as long as" effects).
    Indefinite,
}

// ── Layer classification ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub enum Layer {
    L1Copy        = 1,
    L2Control     = 2,
    L3Text        = 3,
    L4Type        = 4,
    L5Color       = 5,
    L6Ability     = 6,
    L7PowerTough  = 7,
}

/// Sub-layer within Layer 7 for power/toughness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub enum PtSublayer {
    /// 7a: characteristic-defining abilities (e.g. Tarmogoyf).
    CharDefining,
    /// 7b: set to a specific value (e.g. "becomes a 1/1").
    SetValue,
    /// 7c: +N/+M modifications (Giant Growth, Glorious Anthem).
    Modify,
    /// 7d: switch power and toughness.
    Switch,
}

// ── Modification ──────────────────────────────────────────────────────────────

/// The actual change made by a continuous effect within its layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Modification {
    // ── Layer 2 ──────────────────────────────────────────────────────────────
    ChangeController(usize),

    // ── Layer 3 ──────────────────────────────────────────────────────────────
    /// CR 612 — replace all instances of one color word with another
    /// (Protection-from-color keywords). Trait Doctoring, Mind Bend.
    ReplaceColorWord(Color, Color),
    /// CR 612 / 305.7 — replace all instances of one basic land type with
    /// another (type line + landwalk; the intrinsic mana ability follows
    /// the computed land type).
    ReplaceBasicLandType(crate::card::LandType, crate::card::LandType),

    // ── Layer 4 ──────────────────────────────────────────────────────────────
    AddCardType(CardType),
    RemoveCardType(CardType),
    SetCardTypes(Vec<CardType>),
    AddCreatureType(CreatureType),
    SetCreatureTypes(Vec<CreatureType>),
    AddLandType(LandType),
    SetLandTypes(Vec<LandType>),

    // ── Layer 5 ──────────────────────────────────────────────────────────────
    AddColor(Color),
    SetColors(Vec<Color>),
    LoseAllColors,

    // ── Layer 6 ──────────────────────────────────────────────────────────────
    AddKeyword(Keyword),
    RemoveKeyword(Keyword),
    RemoveAllAbilities,

    // ── Layer 7 ──────────────────────────────────────────────────────────────
    SetPowerToughness(i32, i32),   // 7b
    ModifyPower(i32),              // 7c
    ModifyToughness(i32),          // 7c
    ModifyPowerToughness(i32, i32),// 7c
    SwitchPowerToughness,          // 7d
}

// ── Continuous effect ─────────────────────────────────────────────────────────

/// A single continuous effect active in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousEffect {
    /// Unique ID (monotonically increasing timestamp for ordering within a layer).
    pub timestamp: u64,
    /// The permanent that generated this effect (used for `WhileSourceOnBattlefield`).
    pub source: CardId,
    /// Which permanents this effect applies to.
    /// `None` means it applies to the source itself.
    pub affected: AffectedPermanents,
    pub layer: Layer,
    pub sublayer: Option<PtSublayer>,
    pub duration: EffectDuration,
    pub modification: Modification,
}

/// Describes which permanents a continuous effect affects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AffectedPermanents {
    /// Only the source permanent itself.
    Source,
    /// All permanents matching the given predicate.
    /// `controller: Some(p)` — only that player's permanents; `None` — all players.
    /// `exclude_source: true` skips the effect's source permanent — matches the
    /// printed "**other** [type]" wording (Hofri Ghostforge's "Other creatures
    /// you control get +1/+0", various "Other X you control" anthems). Defaults
    /// to `false` (include source) via `#[serde(default)]` for snapshot back-
    /// compat with pre-push-XXXV serialized states.
    /// `card_types` is matched as a conjunction: a permanent must carry
    /// *every* listed type (so `[Artifact, Creature]` means "artifact
    /// creatures", not "artifacts or creatures"). Empty = any type.
    All {
        controller: Option<usize>,
        card_types: Vec<CardType>,
        #[serde(default)]
        exclude_source: bool,
        /// Optional color filter (Honor of the Pure's "White creatures you
        /// control"). `None` = any color. Matched against the card's cost
        /// colors. `#[serde(default)]` keeps old snapshots valid.
        #[serde(default)]
        color: Option<crate::mana::Color>,
        /// Optional token filter (Intangible Virtue's "creature tokens you
        /// control", Always Watching's "nontoken creatures"). `None` = either;
        /// `Some(true)` = tokens only; `Some(false)` = nontokens only.
        /// `#[serde(default)]` keeps old snapshots valid.
        #[serde(default)]
        token: Option<bool>,
        /// "Colorless only" filter (Ruination Guide's "other colorless
        /// creatures you control"). Devoid-aware (CR 702.114). `#[serde(default)]`
        /// keeps old snapshots valid.
        #[serde(default)]
        colorless: bool,
    },
    /// All permanents controlled by any player on a team other than the
    /// source's. In 1v1 / free-for-all this is "everyone but the source's
    /// controller" — but in team formats (2HG) teammates of the source
    /// must not be hit. `friendly_seats` lists the source's team; empty
    /// means "use the legacy single-seat `source_controller` check"
    /// (snapshots predating push-XLIII).
    AllOpponents {
        source_controller: usize,
        card_types: Vec<CardType>,
        #[serde(default)]
        friendly_seats: Vec<usize>,
    },
    /// A specific set of permanents.
    Specific(Vec<CardId>),
    /// All creatures with the given creature type (lord effect).
    /// `controller: Some(p)` restricts to that player's creatures; `None` = all.
    /// `exclude_source: true` skips the effect's source permanent itself —
    /// matches printed "**other** [creature type]s" wording (Goblin King,
    /// Quintorius Field Historian, etc.). `#[serde(default)]` so existing
    /// literal initializers stay valid (defaults to false = include source).
    AllWithCreatureType {
        controller: Option<usize>,
        creature_type: crate::card::CreatureType,
        #[serde(default)]
        exclude_source: bool,
    },
    /// All permanents bearing at least `at_least` counters of the given
    /// kind. Used for SOS Emil's "creatures you control with +1/+1
    /// counters have trample" lord-with-counter pattern, and the
    /// broader "Monstrous / Levelup-creatures get [keyword]" buff
    /// shape. `controller: Some(p)` restricts to that player; `None` =
    /// all. `card_types` empty means any permanent type matches.
    AllWithCounter {
        controller: Option<usize>,
        card_types: Vec<CardType>,
        counter: crate::card::CounterType,
        at_least: u32,
    },
    /// A requirement the simpler variants can't express (disjunctions,
    /// `IsNonbasicLand`, type unions). Evaluated card-locally via
    /// [`requirement_matches_card`], relative to `source_controller` for
    /// `ControlledByYou`/`ControlledByOpponent`. Only emitted for filters
    /// whose every leaf is computable from a card's printed characteristics
    /// (no power/combat/zone state), so the match is correct without a
    /// `GameState` handle. Powers Thalia, Heretic Cathar ("creatures **and
    /// nonbasic lands** your opponents control enter tapped").
    CardMatch {
        source_controller: usize,
        requirement: Box<SelectionRequirement>,
    },
}

// ── Computed permanent ────────────────────────────────────────────────────────

/// The fully resolved state of a permanent after all layers are applied.
/// Use this instead of reading `CardInstance` fields directly when layers matter.
#[derive(Debug, Clone)]
pub struct ComputedPermanent {
    pub id: CardId,
    pub controller: usize,
    pub card_types: Vec<CardType>,
    pub supertypes: Vec<Supertype>,
    pub subtypes: Subtypes,
    pub colors: Vec<Color>,
    pub keywords: Vec<Keyword>,
    pub power: i32,
    pub toughness: i32,
    /// True when at least one `Modification::RemoveAllAbilities` continuous
    /// effect is in scope for this permanent. Lets the trigger dispatcher
    /// and activated-ability resolver skip the card's printed
    /// `triggered_abilities` / `activated_abilities` / `static_abilities` so
    /// "loses all abilities" cards (Turn to Frog, Mercurial Transformation,
    /// Lignify) honor CR 113.10 — the layer-6 strip applies to printed
    /// abilities, not just keywords. Defaults to false so pre-push snapshots
    /// keep their existing behavior.
    pub lost_all_abilities: bool,
}

// ── Layer application ─────────────────────────────────────────────────────────

/// Apply all active continuous effects to every permanent in `battlefield`,
/// returning one `ComputedPermanent` per card instance.
///
/// `effects` must already be filtered to only active (non-expired) effects.
pub fn apply_layers(
    battlefield: &[crate::card::CardInstance],
    effects: &[ContinuousEffect],
) -> Vec<ComputedPermanent> {
    battlefield
        .iter()
        .map(|card| compute_permanent(card, effects))
        .collect()
}

/// Apply all active continuous effects to a single permanent, returning
/// its `ComputedPermanent`. Same per-card work as one iteration of
/// [`apply_layers`] — for callers that only need one card's computed
/// state and shouldn't pay to build (and discard) every other
/// permanent's view.
pub fn apply_layers_one(
    card: &crate::card::CardInstance,
    effects: &[ContinuousEffect],
) -> ComputedPermanent {
    compute_permanent(card, effects)
}

fn compute_permanent(
    card: &crate::card::CardInstance,
    effects: &[ContinuousEffect],
) -> ComputedPermanent {
    // Start from the base card definition.
    let mut controller = card.controller;
    let mut card_types = card.definition.card_types.clone();
    let supertypes = card.definition.supertypes.clone();
    let mut subtypes = card.definition.subtypes.clone();
    // CR 702.103d — while bestowed, the permanent is an Aura enchantment,
    // not a creature. Strip the Creature type and add the Aura subtype so
    // it isn't a valid blocker/attacker/removal target and the orphan-Aura
    // checks treat it correctly.
    if card.bestowed {
        card_types.retain(|t| !matches!(t, CardType::Creature));
        if !card_types.contains(&CardType::Enchantment) {
            card_types.push(CardType::Enchantment);
        }
        if !subtypes
            .enchantment_subtypes
            .contains(&crate::card::EnchantmentSubtype::Aura)
        {
            subtypes
                .enchantment_subtypes
                .push(crate::card::EnchantmentSubtype::Aura);
        }
    }
    let mut colors = colors_from_card(card);
    let mut keywords = card.definition.keywords.clone();
    // Merge in EOT-granted keywords so a computed view sees them just like
    // the layered-effect ones. Cleared at Cleanup via
    // `clear_end_of_turn_effects`.
    for kw in &card.granted_keywords_eot {
        if !keywords.contains(kw) {
            keywords.push(kw.clone());
        }
    }
    // CR 122.1b — keyword counters: each keyword counter type on the
    // permanent grants the named keyword while at least one counter of
    // that type is present. Applied as a layer-6 keyword addition.
    for (kw, count) in &card.keyword_counters {
        if *count > 0 && !keywords.contains(kw) {
            keywords.push(kw.clone());
        }
    }

    // Base P/T from definition + counters (applied after layer 7c).
    let base_power = card.definition.base_power() + card.power_bonus;
    let base_toughness = card.definition.base_toughness() + card.toughness_bonus;
    // CR 613.7f — +1/+1, -1/-1, and the rarer -0/-1 / -1/-0 counters. The
    // last two affect only one of power/toughness, so track per-stat deltas.
    let (counter_power_delta, counter_toughness_delta) = {
        let plus = card.counter_count(CounterType::PlusOnePlusOne) as i32;
        let minus = card.counter_count(CounterType::MinusOneMinusOne) as i32;
        let minus_zero_one = card.counter_count(CounterType::MinusZeroMinusOne) as i32;
        let minus_one_zero = card.counter_count(CounterType::MinusOneMinusZero) as i32;
        let base = plus - minus;
        (base - minus_one_zero, base - minus_zero_one)
    };

    let mut set_pt: Option<(i32, i32)> = None;
    let mut mod_power: i32 = 0;
    let mut mod_toughness: i32 = 0;
    let mut switched = false;
    let mut lost_all_abilities = false;

    // Sort effects by layer, then sublayer, then timestamp.
    let mut sorted: Vec<&ContinuousEffect> = effects
        .iter()
        .filter(|e| affects(e, card))
        .collect();
    sorted.sort_by(|a, b| {
        a.layer.cmp(&b.layer)
            .then(a.sublayer.cmp(&b.sublayer))
            .then(a.timestamp.cmp(&b.timestamp))
    });

    for effect in sorted {
        match &effect.modification {
            // Layer 2
            Modification::ChangeController(c) => controller = *c,

            // Layer 3 — text-changing (CR 612). Runs before type/color/
            // ability layers, so the swapped words feed everything below.
            Modification::ReplaceColorWord(from, to) => {
                for kw in keywords.iter_mut() {
                    if matches!(kw, crate::card::Keyword::Protection(c) if c == from) {
                        *kw = crate::card::Keyword::Protection(*to);
                    }
                }
            }
            Modification::ReplaceBasicLandType(from, to) => {
                for lt in subtypes.land_types.iter_mut() {
                    if lt == from {
                        *lt = *to;
                    }
                }
                for kw in keywords.iter_mut() {
                    if matches!(kw, crate::card::Keyword::Landwalk(l) if l == from) {
                        *kw = crate::card::Keyword::Landwalk(*to);
                    }
                }
            }

            // Layer 4
            Modification::AddCardType(t) => {
                if !card_types.contains(t) { card_types.push(t.clone()); }
            }
            Modification::RemoveCardType(t) => card_types.retain(|x| x != t),
            Modification::SetCardTypes(ts) => card_types = ts.clone(),
            Modification::AddCreatureType(ct) => {
                if !subtypes.creature_types.contains(ct) {
                    subtypes.creature_types.push(*ct);
                }
            }
            Modification::SetCreatureTypes(cts) => subtypes.creature_types = cts.clone(),
            Modification::AddLandType(lt) => {
                if !subtypes.land_types.contains(lt) {
                    subtypes.land_types.push(*lt);
                }
            }
            Modification::SetLandTypes(lts) => subtypes.land_types = lts.clone(),

            // Layer 5
            Modification::AddColor(c) => {
                if !colors.contains(c) { colors.push(*c); }
            }
            Modification::SetColors(cs) => colors = cs.clone(),
            Modification::LoseAllColors => colors.clear(),

            // Layer 6
            Modification::AddKeyword(kw) => {
                if !keywords.contains(kw) { keywords.push(kw.clone()); }
            }
            Modification::RemoveKeyword(kw) => keywords.retain(|k| k != kw),
            Modification::RemoveAllAbilities => {
                keywords.clear();
                lost_all_abilities = true;
            }

            // Layer 7
            Modification::SetPowerToughness(p, t) => set_pt = Some((*p, *t)),
            Modification::ModifyPower(n) => mod_power += n,
            Modification::ModifyToughness(n) => mod_toughness += n,
            Modification::ModifyPowerToughness(p, t) => {
                mod_power += p;
                mod_toughness += t;
            }
            Modification::SwitchPowerToughness => switched = !switched,
        }
    }

    // Compute final P/T.
    let (mut power, mut toughness) = if let Some((p, t)) = set_pt {
        (p, t)
    } else {
        (base_power, base_toughness)
    };
    power += mod_power;
    toughness += mod_toughness;
    // Counters applied after 7c (CR 613.7f).
    power += counter_power_delta;
    toughness += counter_toughness_delta;
    if switched {
        std::mem::swap(&mut power, &mut toughness);
    }

    ComputedPermanent {
        id: card.id,
        controller,
        card_types,
        supertypes,
        subtypes,
        colors,
        keywords,
        power,
        toughness,
        lost_all_abilities,
    }
}

/// Determine which colors a card has from its mana cost symbols.
fn colors_from_card(card: &crate::card::CardInstance) -> Vec<Color> {
    use crate::mana::ManaSymbol;
    // CR 702.114 — Devoid: the object is colorless (CDA), ignore cost pips.
    if card.definition.keywords.contains(&crate::card::Keyword::Devoid) {
        return Vec::new();
    }
    let mut colors = Vec::new();
    for sym in &card.definition.cost.symbols {
        match sym {
            ManaSymbol::Colored(c) if !colors.contains(c) => { colors.push(*c); }
            ManaSymbol::Colored(_) => {}
            ManaSymbol::Hybrid(a, b) => {
                if !colors.contains(a) { colors.push(*a); }
                if !colors.contains(b) { colors.push(*b); }
            }
            ManaSymbol::Phyrexian(c) if !colors.contains(c) => { colors.push(*c); }
            ManaSymbol::Phyrexian(_) => {}
            ManaSymbol::MonoHybrid(_, c) if !colors.contains(c) => { colors.push(*c); }
            ManaSymbol::MonoHybrid(_, _) => {}
            _ => {}
        }
    }
    colors
}

/// Returns true if `effect` affects `card`.
fn affects(effect: &ContinuousEffect, card: &crate::card::CardInstance) -> bool {
    affected_includes(&effect.affected, effect.source, card)
}

/// Whether `card` is one of the permanents described by `affected`, given the
/// describing effect's `source` permanent. Split out of [`affects`] so the
/// enters-tapped ETB replacement (CR 614.13) can reuse the same selector
/// matching without constructing a throwaway `ContinuousEffect`.
pub(crate) fn affected_includes(
    affected: &AffectedPermanents,
    source: CardId,
    card: &crate::card::CardInstance,
) -> bool {
    match affected {
        AffectedPermanents::Source => source == card.id,
        AffectedPermanents::Specific(ids) => ids.contains(&card.id),
        AffectedPermanents::All { controller, card_types, exclude_source, color, token, colorless } => {
            if *exclude_source && source == card.id {
                return false;
            }
            let ctrl_ok = controller.is_none_or(|c| c == card.controller);
            let type_ok = card_types.is_empty()
                || card_types.iter().all(|t| card.definition.card_types.contains(t));
            let color_ok = color.is_none_or(|want| {
                card.definition.cost.symbols.iter().any(|s| {
                    matches!(s, crate::mana::ManaSymbol::Colored(c) if *c == want)
                })
            });
            // CR 702.114 — Devoid CDA: colorless despite colored pips.
            let colorless_ok = !*colorless
                || card.definition.keywords.contains(&crate::card::Keyword::Devoid)
                || !card.definition.cost.symbols.iter()
                    .any(|s| matches!(s, crate::mana::ManaSymbol::Colored(_)));
            let token_ok = token.is_none_or(|want| card.is_token == want);
            ctrl_ok && type_ok && color_ok && colorless_ok && token_ok
        }
        AffectedPermanents::AllOpponents { source_controller, card_types, friendly_seats } => {
            // Empty `friendly_seats` → legacy 1v1 check (snapshots from
            // before push-XLIII didn't populate it).
            let ctrl_ok = if friendly_seats.is_empty() {
                card.controller != *source_controller
            } else {
                !friendly_seats.contains(&card.controller)
            };
            let type_ok = card_types.is_empty()
                || card_types.iter().all(|t| card.definition.card_types.contains(t));
            ctrl_ok && type_ok
        }
        AffectedPermanents::AllWithCreatureType { controller, creature_type, exclude_source } => {
            if *exclude_source && source == card.id {
                return false;
            }
            let ctrl_ok = controller.is_none_or(|c| c == card.controller);
            let is_creature = card.definition.card_types.contains(&CardType::Creature);
            let has_type = card.definition.subtypes.creature_types.contains(creature_type)
                || card.definition.keywords.contains(&Keyword::Changeling);
            ctrl_ok && is_creature && has_type
        }
        AffectedPermanents::AllWithCounter { controller, card_types, counter, at_least } => {
            let ctrl_ok = controller.is_none_or(|c| c == card.controller);
            let type_ok = card_types.is_empty()
                || card_types.iter().all(|t| card.definition.card_types.contains(t));
            let counter_ok = card.counter_count(*counter) >= *at_least;
            ctrl_ok && type_ok && counter_ok
        }
        AffectedPermanents::CardMatch { source_controller, requirement } => {
            // CR "other ... you control": `OtherThanSource` is matched here
            // (where the source id is known) rather than in the source-blind
            // `requirement_matches_card`, which treats it as always-true.
            if requirement_mentions_other_than_source(requirement) && source == card.id {
                return false;
            }
            requirement_matches_card(requirement, card, *source_controller)
        }
    }
}

/// True when `req`'s every leaf is computable from a card's *printed*
/// characteristics alone (type/supertype/subtype/color/token/controller and
/// boolean combinations) — i.e. no power/toughness, combat, counter, or zone
/// state. Such requirements can be matched by [`requirement_matches_card`]
/// without a `GameState`, so `affected_from_requirement` may safely route them
/// through [`AffectedPermanents::CardMatch`].
pub(crate) fn requirement_is_card_only(req: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match req {
        R::Any | R::Permanent | R::Creature | R::Artifact | R::Enchantment
        | R::Planeswalker | R::Land | R::Nonland | R::Noncreature | R::IsBasicLand
        | R::IsNonbasicLand | R::IsToken | R::NotToken | R::ControlledByYou
        | R::ControlledByOpponent | R::Colorless => true,
        // Tap state is a live `CardInstance` field, re-read on every layer
        // recompute — safe to route through the dynamic CardMatch path
        // (Augusta, Dean of Order's tapped/untapped anthems).
        R::Tapped | R::Untapped => true,
        R::HasColor(_) | R::HasCreatureType(_) | R::HasLandType(_) | R::HasSupertype(_)
        | R::HasArtifactSubtype(_) | R::HasEnchantmentSubtype(_) | R::HasCardType(_)
        | R::HasKeyword(_) => true,
        // OtherThanSource is matched in `affects()` (which knows the source id),
        // so it's safe to route a filter containing it through CardMatch.
        R::OtherThanSource => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_is_card_only(a) && requirement_is_card_only(b)
        }
        R::Not(inner) => requirement_is_card_only(inner),
        _ => false,
    }
}

/// True if `req` contains an `OtherThanSource` leaf anywhere — the source
/// exclusion is then applied in `affects()` against the live source id.
fn requirement_mentions_other_than_source(req: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match req {
        R::OtherThanSource => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_mentions_other_than_source(a) || requirement_mentions_other_than_source(b)
        }
        R::Not(inner) => requirement_mentions_other_than_source(inner),
        _ => false,
    }
}

/// Evaluate a card-only requirement (see [`requirement_is_card_only`]) against
/// a single permanent. `source_controller` resolves `ControlledByYou` /
/// `ControlledByOpponent`. Unsupported leaves resolve to `false`.
pub(crate) fn requirement_matches_card(
    req: &SelectionRequirement,
    card: &crate::card::CardInstance,
    source_controller: usize,
) -> bool {
    use SelectionRequirement as R;
    let def = &card.definition;
    match req {
        R::Any | R::Permanent => true,
        R::Creature => def.card_types.contains(&CardType::Creature),
        R::Artifact => def.card_types.contains(&CardType::Artifact),
        R::Enchantment => def.card_types.contains(&CardType::Enchantment),
        R::Planeswalker => def.card_types.contains(&CardType::Planeswalker),
        R::Land => def.card_types.contains(&CardType::Land),
        R::Nonland => !def.card_types.contains(&CardType::Land),
        R::Noncreature => !def.card_types.contains(&CardType::Creature),
        R::IsBasicLand => def.is_land() && def.supertypes.contains(&Supertype::Basic),
        R::IsNonbasicLand => def.is_land() && !def.supertypes.contains(&Supertype::Basic),
        R::IsToken => card.is_token,
        R::NotToken => !card.is_token,
        R::Tapped => card.tapped,
        R::Untapped => !card.tapped,
        R::ControlledByYou => card.controller == source_controller,
        R::ControlledByOpponent => card.controller != source_controller,
        R::HasCardType(t) => def.card_types.contains(t),
        R::HasSupertype(s) => def.supertypes.contains(s),
        R::HasCreatureType(ct) => def.subtypes.creature_types.contains(ct)
            || def.keywords.contains(&Keyword::Changeling),
        R::HasLandType(lt) => def.subtypes.land_types.contains(lt),
        R::HasArtifactSubtype(a) => def.subtypes.artifact_subtypes.contains(a),
        R::HasEnchantmentSubtype(e) => def.subtypes.enchantment_subtypes.contains(e),
        R::HasKeyword(k) => def.keywords.contains(k),
        R::HasColor(c) => def
            .cost
            .symbols
            .iter()
            .any(|s| matches!(s, crate::mana::ManaSymbol::Colored(col) if col == c)),
        // CR 702.114 — Devoid is a CDA: the object is colorless regardless of
        // its (possibly colored) cost pips.
        R::Colorless => def.keywords.contains(&crate::card::Keyword::Devoid)
            || !def
                .cost
                .symbols
                .iter()
                .any(|s| matches!(s, crate::mana::ManaSymbol::Colored(_))),
        R::And(a, b) => {
            requirement_matches_card(a, card, source_controller)
                && requirement_matches_card(b, card, source_controller)
        }
        R::Or(a, b) => {
            requirement_matches_card(a, card, source_controller)
                || requirement_matches_card(b, card, source_controller)
        }
        R::Not(inner) => !requirement_matches_card(inner, card, source_controller),
        // Source exclusion is enforced in `affects()` (source id known there);
        // treat as always-matching for the printed-characteristics walk.
        R::OtherThanSource => true,
        _ => false,
    }
}
