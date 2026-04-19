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
    CardId, CardType, CounterType, CreatureType, Keyword, LandType, Subtypes, Supertype,
};
use crate::mana::Color;

// ── Duration ──────────────────────────────────────────────────────────────────

/// How long a continuous effect persists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectDuration {
    /// Lasts while the source permanent is on the battlefield (static ability).
    WhileSourceOnBattlefield,
    /// Expires at the next end-of-turn Cleanup step.
    UntilEndOfTurn,
    /// Lasts until the beginning of the next turn.
    UntilNextTurn,
    /// Indefinite (e.g. counters, "for as long as" effects).
    Indefinite,
}

// ── Layer classification ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone)]
pub enum Modification {
    // ── Layer 2 ──────────────────────────────────────────────────────────────
    ChangeController(usize),

    // ── Layer 4 ──────────────────────────────────────────────────────────────
    AddCardType(CardType),
    RemoveCardType(CardType),
    SetCardTypes(Vec<CardType>),
    AddCreatureType(CreatureType),
    SetCreatureTypes(Vec<CreatureType>),
    AddLandType(LandType),

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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum AffectedPermanents {
    /// Only the source permanent itself.
    Source,
    /// All permanents matching the given predicate.
    /// `controller: Some(p)` — only that player's permanents; `None` — all players.
    All { controller: Option<usize>, card_types: Vec<CardType> },
    /// All permanents controlled by any player *other* than `source_controller`.
    AllOpponents { source_controller: usize, card_types: Vec<CardType> },
    /// A specific set of permanents.
    Specific(Vec<CardId>),
    /// All creatures with the given creature type (lord effect).
    /// `controller: Some(p)` restricts to that player's creatures; `None` = all.
    AllWithCreatureType { controller: Option<usize>, creature_type: crate::card::CreatureType },
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

fn compute_permanent(
    card: &crate::card::CardInstance,
    effects: &[ContinuousEffect],
) -> ComputedPermanent {
    // Start from the base card definition.
    let mut controller = card.controller;
    let mut card_types = card.definition.card_types.clone();
    let supertypes = card.definition.supertypes.clone();
    let mut subtypes = card.definition.subtypes.clone();
    let mut colors = colors_from_card(card);
    let mut keywords = card.definition.keywords.clone();

    // Base P/T from definition + counters (applied after layer 7c).
    let base_power = card.definition.base_power() + card.power_bonus;
    let base_toughness = card.definition.base_toughness() + card.toughness_bonus;
    let counter_delta = {
        let plus = card.counter_count(CounterType::PlusOnePlusOne) as i32;
        let minus = card.counter_count(CounterType::MinusOneMinusOne) as i32;
        plus - minus
    };

    let mut set_pt: Option<(i32, i32)> = None;
    let mut mod_power: i32 = 0;
    let mut mod_toughness: i32 = 0;
    let mut switched = false;

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
            Modification::RemoveAllAbilities => keywords.clear(),

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
    power += counter_delta;
    toughness += counter_delta;
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
    }
}

/// Determine which colors a card has from its mana cost symbols.
fn colors_from_card(card: &crate::card::CardInstance) -> Vec<Color> {
    use crate::mana::ManaSymbol;
    let mut colors = Vec::new();
    for sym in &card.definition.cost.symbols {
        match sym {
            ManaSymbol::Colored(c) => {
                if !colors.contains(c) { colors.push(*c); }
            }
            ManaSymbol::Hybrid(a, b) => {
                if !colors.contains(a) { colors.push(*a); }
                if !colors.contains(b) { colors.push(*b); }
            }
            ManaSymbol::Phyrexian(c) => {
                if !colors.contains(c) { colors.push(*c); }
            }
            _ => {}
        }
    }
    colors
}

/// Returns true if `effect` affects `card`.
fn affects(effect: &ContinuousEffect, card: &crate::card::CardInstance) -> bool {
    match &effect.affected {
        AffectedPermanents::Source => effect.source == card.id,
        AffectedPermanents::Specific(ids) => ids.contains(&card.id),
        AffectedPermanents::All { controller, card_types } => {
            let ctrl_ok = controller.is_none_or(|c| c == card.controller);
            let type_ok = card_types.is_empty()
                || card_types.iter().any(|t| card.definition.card_types.contains(t));
            ctrl_ok && type_ok
        }
        AffectedPermanents::AllOpponents { source_controller, card_types } => {
            let ctrl_ok = card.controller != *source_controller;
            let type_ok = card_types.is_empty()
                || card_types.iter().any(|t| card.definition.card_types.contains(t));
            ctrl_ok && type_ok
        }
        AffectedPermanents::AllWithCreatureType { controller, creature_type } => {
            let ctrl_ok = controller.is_none_or(|c| c == card.controller);
            let is_creature = card.definition.card_types.contains(&CardType::Creature);
            let has_type = card.definition.subtypes.creature_types.contains(creature_type)
                || card.definition.keywords.contains(&Keyword::Changeling);
            ctrl_ok && is_creature && has_type
        }
    }
}
