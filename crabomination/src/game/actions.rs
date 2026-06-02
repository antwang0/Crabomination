use super::*;
use crate::card::{CardType, Keyword};
use crate::effect::{Effect, ManaPayload};
use crate::mana::{Color as ManaColor, ManaSymbol};

/// Skip-Ward check. Ward variants whose payment is trivially affordable
/// (free mana, 0 life, 0 discard) would always auto-pay and produce no
/// visible difference from no Ward at all — so we skip the stack-churn
/// of pushing the trigger. `SacrificeCreature` is never trivial since
/// the controller might have no creatures to sacrifice.
fn ward_cost_is_trivial(cost: &crate::card::WardCost) -> bool {
    use crate::card::WardCost;
    match cost {
        WardCost::Mana(c) => c.cmc() == 0,
        WardCost::Life(n) => *n == 0,
        WardCost::Discard(n) => *n == 0,
        WardCost::SacrificeCreature => false,
    }
}

/// Returns true if the given effect is purely a mana ability — only adds
/// mana and uses no targets. Mana abilities resolve immediately without the stack.
fn is_mana_ability(effect: &Effect) -> bool {
    match effect {
        Effect::AddMana { .. } => true,
        Effect::Seq(steps) => !steps.is_empty() && steps.iter().all(is_mana_ability),
        _ => false,
    }
}

/// The set of colours `card`'s untapped mana abilities can produce, in
/// WUBRG order (a Forest → `[Green]`, a dual → two, Birds → all five, a
/// colorless rock → `[]`). Used as a source's "signature" so the
/// manual-tap decision can tell interchangeable sources (same signature)
/// from genuinely different ones.
fn source_color_signature(card: &crate::card::CardInstance) -> Vec<ManaColor> {
    ManaColor::ALL
        .into_iter()
        .filter(|c| {
            card.definition
                .activated_abilities
                .iter()
                .any(|a| is_mana_ability(&a.effect) && effect_produces_color(&a.effect, *c))
        })
        .collect()
}

/// Pull the "when you cast this spell" (`EventKind::SpellCast` +
/// `EventScope::SelfSource`) triggers off a card. Used by the cast paths
/// to push these onto the stack above the cast spell so they resolve
/// before the spell itself. Returns the trigger's optional filter
/// predicate alongside its effect so the caller can gate the trigger
/// fire on the predicate (e.g. Infusion's LifeGainedThisTurnAtLeast).
fn collect_self_cast_triggers(
    card: &crate::card::CardInstance,
) -> Vec<(Option<crate::card::Predicate>, Effect)> {
    use crate::effect::{EventKind, EventScope};
    card.definition
        .triggered_abilities
        .iter()
        .filter(|t| {
            t.event.kind == EventKind::SpellCast
                && matches!(t.event.scope, EventScope::SelfSource)
        })
        .map(|t| (t.event.filter.clone(), t.effect.clone()))
        .collect()
}

/// Count distinct colors of mana that decreased between two pool
/// snapshots — i.e. the spell's converge value.
fn converge_count(before: &crate::mana::ManaPool, after: &crate::mana::ManaPool) -> u32 {
    use crate::mana::Color;
    let mut count = 0u32;
    for color in Color::ALL {
        if before.amount(color) > after.amount(color) {
            count += 1;
        }
    }
    count
}

/// Walk the battlefield's static abilities + per-player tax charges to
/// compute the total extra generic mana the caster owes for casting `card`.
///
/// Honors:
///   * `StaticEffect::AdditionalCostAfterFirstSpell` (Damping Sphere): if
///     the caster has already cast at least one spell this turn and the
///     spell matches the static's `filter`, charge `amount` more.
///   * `Player.first_spell_tax_charges` (Chancellor of the Annex): each
///     pending charge taxes the caster's *next* spell {1} more. Consumed by
///     the caller on a successful cast (we only **read** here so callers
///     can see the tax before payment; the caster path decrements after).
/// Flashback-only additional cost(s), keyed by card name (CR 702.34a). Some
/// cards' Flashback cost is more than mana — "Flashback—Sacrifice a Mountain"
/// (Lava Dart) or "Flashback—Sacrifice three creatures" (Dread Return). These
/// apply *only* on the flashback cast, so they live here rather than on every
/// CardDefinition (the `dynamic_pt_for_name` lookup-table idiom). `cast_flashback`
/// validates + pays them on top of the flashback mana cost.
pub(crate) fn flashback_additional_cost_for_name(
    name: &str,
) -> Vec<crate::card::AdditionalCastCost> {
    use crate::card::{AdditionalCastCost as A, LandType, SelectionRequirement as S};
    match name {
        "Lava Dart" => vec![A::SacrificePermanent {
            filter: S::Land.and(S::HasLandType(LandType::Mountain)),
            count: 1,
        }],
        "Dread Return" => vec![A::SacrificePermanent {
            filter: S::Creature.and(S::ControlledByYou),
            count: 3,
        }],
        _ => vec![],
    }
}

pub(crate) fn extra_cost_for_spell(
    state: &crate::game::GameState,
    caster: usize,
    card: &crate::card::CardInstance,
) -> u32 {
    use crate::effect::StaticEffect;
    let mut tax = 0u32;
    if state.players[caster].first_spell_tax_charges > 0 {
        tax += 1;
    }
    let already_cast = state.players[caster].spells_cast_this_turn;
    for src in &state.battlefield {
        for sa in &src.definition.static_abilities {
            match &sa.effect {
                StaticEffect::AdditionalCostAfterFirstSpell { filter, amount }
                    if already_cast > 0
                        && state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    tax += amount;
                }
                StaticEffect::AdditionalCost { filter, amount }
                    if state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    tax += amount;
                }
                _ => {}
            }
        }
    }

    tax
}

/// Consume one Chancellor-of-the-Annex tax charge from `caster`, if any.
/// Called by every cast path immediately after the spell successfully
/// resolves payment, so each first-spell-tax charge is single-use.
pub(crate) fn consume_first_spell_tax(state: &mut crate::game::GameState, caster: usize) {
    if state.players[caster].first_spell_tax_charges > 0 {
        state.players[caster].first_spell_tax_charges -= 1;
    }
}

/// Sum all generic-mana cost reductions applicable to a spell being cast.
///
/// Supports two flavors:
///   * `StaticEffect::CostReduction { filter, amount }` — flat per-spell
///     reduction whose `filter` matches the cast card.
///   * `StaticEffect::CostReductionTargetingFilter { spell_filter,
///     target_filter, amount }` — Killian-style "if the spell targets a
///     creature, it costs {2} less". Honors the cast's chosen target via
///     `target` (so Lightning Bolt at face counts as targeting a player,
///     Lightning Bolt at a creature counts as targeting a creature).
///
/// CR 601.2f / 117.7c: cost reductions can never reduce a colored or X
/// pip. The caller funnels the returned reduction through
/// `ManaCost::reduce_generic`, which clamps at the generic pip total.
pub(crate) fn cost_reduction_for_spell(
    state: &crate::game::GameState,
    caster: usize,
    card: &crate::card::CardInstance,
    target: Option<&crate::game::Target>,
) -> u32 {
    use crate::effect::StaticEffect;
    let mut reduction = 0u32;
    for src in &state.battlefield {
        for sa in &src.definition.static_abilities {
            match &sa.effect {
                StaticEffect::CostReduction { filter, amount }
                    if src.controller == caster
                        && state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    reduction += amount;
                }
                StaticEffect::CostReductionTargetingFilter {
                    spell_filter,
                    target_filter,
                    amount,
                } => {
                    if src.controller != caster {
                        continue;
                    }
                    if !state.evaluate_requirement_on_card(spell_filter, card, caster) {
                        continue;
                    }
                    let Some(tgt) = target else { continue };
                    if state.evaluate_requirement_static(target_filter, tgt, caster, Some(card.id)) {
                        reduction += amount;
                    }
                }
                StaticEffect::GrantAffinityToISSpells { permanent_filter } => {
                    // "Instant and sorcery spells you cast have Affinity for
                    // [permanent_filter]" — only fires on the controller's
                    // IS spells. Counts every battlefield permanent matching
                    // `permanent_filter` and reduces by 1 per match.
                    if src.controller != caster {
                        continue;
                    }
                    if !card.definition.is_instant() && !card.definition.is_sorcery() {
                        continue;
                    }
                    let count = state
                        .battlefield
                        .iter()
                        .filter(|c| state.evaluate_requirement_on_card(permanent_filter, c, caster))
                        .count();
                    reduction = reduction.saturating_add(count as u32);
                }
                _ => {}
            }
        }
    }
    // Card-intrinsic Affinity-for-[filter] cost reduction: "{1} less for
    // each [filter]" baked onto the spell card itself. Counts every
    // battlefield permanent matching `affinity_filter`. CR 601.2f / 117.7c —
    // generic-only, the colored-pip clamp happens in
    // `ManaCost::reduce_generic` once the caller folds this back into the
    // cost.
    if let Some(filter) = &card.definition.affinity_filter {
        let count = state
            .battlefield
            .iter()
            .filter(|c| state.evaluate_requirement_on_card(filter, c, caster))
            .count();
        reduction = reduction.saturating_add(count as u32);
    }
    // Per-card hardcoded "Affinity-for-cards-in-your-graveyard" hook.
    // The Dawning Archaic ("This spell costs {1} less to cast for each
    // instant and sorcery card in your graveyard") was the only card
    // needing this primitive at promotion time; rather than thread a
    // new `affinity_graveyard_filter` field through every CardDefinition
    // site, the discount is dispatched per-name. Future Affinity-for-gy
    // cards can extend this match.
    if card.definition.name == "The Dawning Archaic" {
        let count = state.players[caster]
            .graveyard
            .iter()
            .filter(|c| c.definition.is_instant() || c.definition.is_sorcery())
            .count();
        reduction = reduction.saturating_add(count as u32);
    }
    reduction
}

/// Trinisphere floor: the largest `StaticEffect::SpellCostFloor` amount
/// among untapped battlefield permanents (affects every player's spells).
/// Returns 0 if none in play.
pub(crate) fn spell_cost_floor(state: &crate::game::GameState) -> u32 {
    use crate::effect::StaticEffect;
    state
        .battlefield
        .iter()
        .filter(|c| !c.tapped)
        .flat_map(|c| c.definition.static_abilities.iter())
        .filter_map(|sa| match &sa.effect {
            StaticEffect::SpellCostFloor { amount } => Some(*amount),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

/// Raise `cost` so its mana value is at least the active Trinisphere floor
/// by padding generic mana. No-op when no floor is in play or the cost is
/// already at/above it.
pub(crate) fn apply_spell_cost_floor(
    state: &crate::game::GameState,
    cost: &mut crate::mana::ManaCost,
) {
    let floor = spell_cost_floor(state);
    let mv = cost.cmc();
    if floor > mv {
        cost.symbols
            .push(crate::mana::ManaSymbol::Generic(floor - mv));
    }
}

/// True if `player` controls a permanent granting Omniscience-style free
/// casting of hand spells (`StaticEffect::CastHandSpellsFree`).
impl crate::game::GameState {
    pub(crate) fn player_casts_hand_spells_free(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::CastHandSpellsFree))
        })
    }
}

/// True if any battlefield permanent has `StaticEffect::LandsTapColorlessOnly`
/// (Damping Sphere). Used by `play_land` to decide whether to downgrade
/// multi-color/multi-mana lands to "{T}: Add {C}".
pub(crate) fn multi_mana_ability_count(def: &crate::card::CardDefinition) -> bool {
    use crate::effect::Effect;
    // The Oracle says "tap to add more than one mana" — for our purposes:
    // any land with two or more separate mana abilities, OR any single
    // ability that produces an `AnyOneColor`/`AnyColors` payload (which
    // could conceptually be more than one), counts. Single-color basics
    // (one ability, `Colors([X])` of length 1) and single-color non-basics
    // (one ability) pass through unchanged.
    let mana_abilities: Vec<_> = def
        .activated_abilities
        .iter()
        .filter(|a| matches!(a.effect, Effect::AddMana { .. }))
        .collect();
    if mana_abilities.len() >= 2 {
        return true;
    }
    if let Some(a) = mana_abilities.first()
        && let Effect::AddMana { pool, .. } = &a.effect
    {
        return payload_yields_multiple(pool);
    }
    false
}

/// True if a single `AddMana` payload could yield more than one mana (or a
/// player-chosen color), used by `multi_mana_ability_count`. A spend
/// restriction is transparent here — it wraps an inner payload whose shape
/// is what matters.
fn payload_yields_multiple(pool: &crate::effect::ManaPayload) -> bool {
    use crate::effect::ManaPayload;
    match pool {
        ManaPayload::AnyOneColor(_)
        | ManaPayload::AnyColors(_)
        | ManaPayload::DevotionOfChosenColor
        | ManaPayload::AnyColorOpponentCouldProduce => true,
        ManaPayload::Colors(cs) => cs.len() > 1,
        ManaPayload::OfColors(cs, _) => cs.len() > 1,
        ManaPayload::OfColor(_, _) | ManaPayload::Colorless(_) => false,
        ManaPayload::Restricted(inner, _) => payload_yields_multiple(inner),
    }
}

/// Elesh Norn, Mother of Machines: count how many times an ETB trigger
/// from a permanent owned by `etb_controller` should fire.
///
/// Rules:
/// - "Permanents entering the battlefield don't cause abilities of permanents
///   your opponents control to trigger" → if any opponent of the
///   permanent's controller has an Elesh Norn, the trigger is suppressed
///   (returns 0).
/// - "If a permanent entering the battlefield causes a triggered ability of
///   a permanent you control to trigger, that ability triggers an additional
///   time" → each Elesh Norn on the trigger-source's side adds one extra fire.
///
/// `etb_controller` is the controller of the ability's source — for self-ETB
/// triggers, that's the entering permanent itself.
pub(crate) fn etb_trigger_multiplier(
    state: &crate::game::GameState,
    etb_controller: usize,
) -> usize {
    use crate::effect::StaticEffect;
    let mut your_norns = 0usize;
    let mut opp_norns = 0usize;
    // Yarok/Panharmonicon-style doublers add fires for the controller's own
    // ETB triggers without suppressing opponents'.
    let mut your_doublers = 0usize;
    for c in &state.battlefield {
        let count_spotlight = c
            .definition
            .static_abilities
            .iter()
            .filter(|sa| matches!(sa.effect, StaticEffect::EtbTriggerSpotlight))
            .count();
        let count_doubler = c
            .definition
            .static_abilities
            .iter()
            .filter(|sa| matches!(sa.effect, StaticEffect::DoubleControllerEtbTriggers))
            .count();
        if c.controller == etb_controller {
            your_norns += count_spotlight;
            your_doublers += count_doubler;
        } else {
            opp_norns += count_spotlight;
        }
    }
    if opp_norns > 0 {
        0
    } else {
        1 + your_norns + your_doublers
    }
}

/// Strict Proctor ETB-trigger tax — CR 614 replacement effect.
///
/// "If a permanent entering the battlefield causes a triggered ability of
/// a permanent to trigger, that ability's controller sacrifices the
/// permanent unless they pay {amount}." Read at ETB-trigger dispatch time
/// for each `StaticEffect::EtbTriggerTax` in play.
///
/// Returns `true` if the trigger should fire (controller paid or no tax in
/// play), `false` if it should be suppressed (controller declined or
/// couldn't pay). On `false`, the trigger source has been sacrificed.
///
/// `trigger_source` is the permanent whose ability is triggering — that's
/// the permanent that gets sacrificed if the controller doesn't pay.
pub(crate) fn apply_etb_trigger_tax(
    state: &mut crate::game::GameState,
    trigger_source: crate::card::CardId,
    trigger_controller: usize,
) -> bool {
    use crate::decision::{Decision, DecisionAnswer};
    use crate::effect::StaticEffect;
    use crate::mana::ManaCost;

    // Sum tax amounts from every Strict Proctor on the battlefield.
    // Each Strict Proctor demands its own payment per the printed
    // "sacrifices the permanent unless they pay {2}" wording — but
    // applied as a single rolled-up amount via additive tax (matching
    // the existing engine's handling of stacking-tax effects).
    let total_tax: u32 = state
        .battlefield
        .iter()
        .flat_map(|c| c.definition.static_abilities.iter())
        .filter_map(|sa| {
            if let StaticEffect::EtbTriggerTax { amount } = &sa.effect {
                Some(*amount)
            } else {
                None
            }
        })
        .sum();
    if total_tax == 0 {
        return true;
    }
    // Build a "Pay {total_tax}" decision aimed at the trigger's controller.
    let answer = state.decider.decide(&Decision::OptionalTrigger {
        source: trigger_source,
        description: format!("Pay {{{}}} to keep this trigger?", total_tax),
    });
    if matches!(answer, DecisionAnswer::Bool(true)) {
        let cost = ManaCost::new(vec![crate::mana::generic(total_tax)]);
        if state.players[trigger_controller].mana_pool.pay(&cost).is_ok() {
            return true;
        }
        // Couldn't actually afford the tax — fall through and sacrifice.
    }
    // Sacrifice the trigger source. Only sacrifice if it's still on
    // the battlefield (it may have already left between push and
    // resolution for fast self-die effects).
    if state.battlefield.iter().any(|c| c.id == trigger_source) {
        state.remove_from_battlefield_to_graveyard(trigger_source);
    }
    false
}

/// Cavern of Souls approximation: when a creature spell is cast, mark it
/// uncounterable if the caster controls a Cavern of Souls.
///
/// The real card requires Cavern to be tapped for mana, that mana to be
/// spent on the cast, and the creature's type to match the named type. We
/// don't track mana provenance or named-types, so this collapses to "any
/// creature you cast is uncounterable while you control a Cavern" — close
/// enough for the demo deck.
impl crate::game::GameState {
    /// Legacy entrypoint kept for symmetry; new call sites should use
    /// `caster_grants_uncounterable_with_x` to thread the cast's X
    /// value. Internally delegates with X = 0.
    #[allow(dead_code)]
    pub(crate) fn caster_grants_uncounterable(
        &self,
        caster: usize,
        card: &crate::card::CardInstance,
    ) -> bool {
        self.caster_grants_uncounterable_with_x(caster, card, 0)
    }

    /// X-aware variant. Threaded by `finalize_cast` so cards whose
    /// "can't be countered" rider is gated on the X value (Banefire's
    /// "if X is 5 or more, this spell can't be countered") see the
    /// correct flag at cast time.
    pub(crate) fn caster_grants_uncounterable_with_x(
        &self,
        caster: usize,
        card: &crate::card::CardInstance,
        x_value: u32,
    ) -> bool {
        // The card itself is uncounterable (Dovin's Veto, Stubborn Denial,
        // etc. — `Keyword::CantBeCountered`).
        if card.definition.keywords.contains(&Keyword::CantBeCountered) {
            return true;
        }
        // Turn-scoped grant — Veil of Summer's "spells your opponents
        // control can't counter spells you control this turn."
        if self.players[caster].spells_uncounterable_this_turn {
            return true;
        }
        // Conditional "if X is N or more, this spell can't be countered"
        // rider (Banefire-style). Threshold lives on the card's printed
        // keywords as `CantBeCounteredIfXAtLeast(threshold)`; checked
        // against the actual paid X. Any future card with the same
        // shape plugs in by carrying the keyword — no engine change.
        let xcc_threshold = card.definition.keywords.iter().find_map(|kw| {
            if let Keyword::CantBeCounteredIfXAtLeast(n) = kw {
                Some(*n)
            } else {
                None
            }
        });
        if let Some(n) = xcc_threshold
            && x_value >= n
        {
            return true;
        }
        // Cavern of Souls–style: any permanent the caster controls
        // whose static abilities include
        // `UncounterableCreaturesOfChosenType` makes a matching-type
        // creature spell uncounterable. Permanents whose ETB hasn't yet
        // resolved (`chosen_creature_type == None`) fall back to
        // "unrestricted" so legacy test fixtures that bypass the ETB
        // still work. The mana-provenance gate (must spend the
        // permanent's mana on the cast) is collapsed — see Cavern's
        // catalog doc-comment.
        use crate::effect::StaticEffect;
        if !card.definition.is_creature() {
            return false;
        }
        self.battlefield.iter().any(|c| {
            if c.controller != caster {
                return false;
            }
            let has_static = c.definition.static_abilities.iter().any(|sa| {
                matches!(sa.effect, StaticEffect::UncounterableCreaturesOfChosenType)
            });
            if !has_static {
                return false;
            }
            match c.chosen_creature_type {
                None => true, // ETB not resolved → unrestricted (legacy)
                Some(t) => card.definition.has_creature_type(t),
            }
        })
    }

    /// True if any battlefield permanent's static abilities include
    /// `StaticEffect::LandsTapColorlessOnly` (Damping Sphere). Used by
    /// `play_land` to downgrade multi-mana lands to colorless on entry.
    pub(crate) fn lands_tap_colorless_only_active(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::LandsTapColorlessOnly))
        })
    }

    /// CR 305.2 — Number of additional lands `player` may play this turn
    /// over the default "one." Counts every battlefield permanent that
    /// the given player controls whose static abilities include
    /// `StaticEffect::ExtraLandPerTurn` (Exploration, Azusa Lost But
    /// Seeking, Wayward Swordtooth). Each granting permanent adds one,
    /// so two Explorations stack to "three lands per turn."
    pub fn extra_land_plays_per_turn(&self, player: usize) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == player)
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::ExtraLandPerTurn))
                    .count() as u32
            })
            .sum()
    }

    /// CR 305.2 — Total lands `player` may play this turn. Defaults to
    /// 1, plus any `ExtraLandPerTurn` static grants, plus the player's
    /// manually-set `extra_land_plays` field (set by resolved effects
    /// like Explore).
    pub fn max_lands_per_turn(&self, player: usize) -> u32 {
        1 + self.extra_land_plays_per_turn(player) + self.players[player].extra_land_plays
    }

    /// CR 305.2a — Whether `player` may legally play another land this
    /// turn. Compares lands already played to the active per-turn cap
    /// (which honors `ExtraLandPerTurn` static effects).
    pub fn can_player_play_land(&self, player: usize) -> bool {
        self.players[player].lands_played_this_turn < self.max_lands_per_turn(player)
    }
}

fn effect_produces_color(effect: &Effect, color: ManaColor) -> bool {
    match effect {
        Effect::AddMana { pool, .. } => match pool {
            ManaPayload::Colors(cs) => cs.contains(&color),
            ManaPayload::AnyOneColor(_)
            | ManaPayload::AnyColors(_)
            | ManaPayload::AnyColorOpponentCouldProduce => true,
            // Devotion-scaled: it can make `color`, but only the controller
            // should choose to tap it (devotion may be 0). Not auto-tapped.
            ManaPayload::DevotionOfChosenColor => false,
            ManaPayload::OfColor(c, _) => *c == color,
            ManaPayload::OfColors(cs, _) => cs.contains(&color),
            ManaPayload::Colorless(_) => false,
            // Spend-restricted sources are not auto-tapped: their mana can
            // only fund some spells, so tapping one to "cover" a colored
            // pip could strand an otherwise-payable cast. The controller
            // activates them deliberately (or they float via a trigger),
            // and `pay_for_spell` consumes the floated mana.
            ManaPayload::Restricted(_, _) => false,
        },
        Effect::Seq(steps) => steps.iter().any(|s| effect_produces_color(s, color)),
        _ => false,
    }
}

impl GameState {
    // ── Play land ─────────────────────────────────────────────────────────────

    pub(crate) fn play_land(&mut self, card_id: CardId) -> Result<Vec<GameEvent>, GameError> {
        self.play_land_with_face(card_id, /* back_face */ false)
    }

    /// Shared implementation for `PlayLand` and `PlayLandBack`. When
    /// `back_face` is true and the card has a `back_face`, the card's
    /// definition is swapped to the back face's definition before placing on
    /// the battlefield — so the resulting permanent has the back face's
    /// types, mana abilities, and ETB triggers.
    pub(crate) fn play_land_with_face(
        &mut self,
        card_id: CardId,
        back_face: bool,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // CR 305.2 — honor ExtraLandPerTurn static grants (Exploration,
        // Azusa, Wayward Swordtooth) when checking the per-turn cap.
        if !self.can_player_play_land(p) {
            return Err(GameError::AlreadyPlayedLand);
        }
        if !self.players[p].has_in_hand(card_id) {
            return Err(GameError::CardNotInHand(card_id));
        }
        let mut card = self.players[p].remove_from_hand(card_id).unwrap(); // we just checked has_in_hand
        if back_face {
            // Swap to the back face's definition. Reject if there isn't one.
            let Some(back) = card.definition.back_face.clone() else {
                self.players[p].hand.push(card);
                return Err(GameError::NotALand(card_id));
            };
            card.definition = std::sync::Arc::new(*back);
        }
        if !card.definition.is_land() {
            // Put it back then error
            self.players[p].hand.push(card);
            return Err(GameError::NotALand(card_id));
        }
        // Damping Sphere: if any battlefield permanent grants
        // `LandsTapColorlessOnly`, downgrade this land's mana abilities
        // to `{T}: Add {C}` if the original would have produced more than
        // one mana per tap. Applied in-place on the new instance's
        // definition before the card lands on the battlefield, so all
        // downstream activations see the replaced ability set.
        if self.lands_tap_colorless_only_active()
            && multi_mana_ability_count(&card.definition)
        {
            // Drop the printed mana abilities and replace them with a
            // single `{T}: Add {C}`. Non-mana activated abilities
            // (fetchland sac, manland animate, channel, cycling) survive
            // — Damping Sphere's Oracle only affects mana production.
            let mut kept: Vec<crate::card::ActivatedAbility> = card
                .definition
                .activated_abilities
                .iter()
                .filter(|a| !is_mana_ability(&a.effect))
                .cloned()
                .collect();
            kept.push(crate::card::ActivatedAbility {
                tap_cost: true,
                mana_cost: crate::mana::ManaCost::default(),
                effect: crate::effect::Effect::AddMana {
                    who: crate::effect::PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(
                        crate::effect::Value::Const(1),
                    ),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
                life_cost: 0,
                from_graveyard: false,
                exile_self_cost: false,
                exile_other_filter: None,
                self_counter_cost_reduction: None, sac_other_filter: None,
                tap_other_filter: None, from_hand: false,
            });
            std::sync::Arc::make_mut(&mut card.definition).activated_abilities = kept;
        }
        self.players[p].lands_played_this_turn += 1;
        self.battlefield.push(card);
        // Fire self-source ETB triggers for the land (shockland pay-or-tap,
        // surveil-land tap-and-surveil, etc.). The cast path inlines the same
        // logic in `resolve_top_of_stack`; play_land needs an analogous push
        // so triggered abilities on lands actually fire.
        self.fire_self_etb_triggers(card_id, p);
        Ok(vec![
            GameEvent::LandPlayed { player: p, card_id },
            GameEvent::PermanentEntered { card_id },
        ])
    }

    /// Push the source-itself ETB triggered abilities for a permanent that
    /// has just entered the battlefield. Used by `play_land` and by Move →
    /// Battlefield zone changes so triggered abilities fire consistently
    /// regardless of how the permanent arrived.
    pub(crate) fn fire_self_etb_triggers(&mut self, card_id: CardId, controller: usize) {
        use crate::effect::{EventKind, EventScope};
        let etb_triggers: Vec<Effect> = self
            .battlefield
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| {
                c.definition
                    .triggered_abilities
                    .iter()
                    .filter(|t| t.event.kind == EventKind::EntersBattlefield
                        && matches!(t.event.scope, EventScope::SelfSource))
                    .map(|t| t.effect.clone())
                    .collect()
            })
            .unwrap_or_default();
        // Elesh Norn replacement: zero or more copies depending on which
        // side controls a Mother of Machines.
        let multiplier = etb_trigger_multiplier(self, controller);
        for effect in etb_triggers {
            // Strict Proctor's CR 614 replacement: pay {2} or sacrifice
            // the source. Applied once per fire of the trigger.
            if !apply_etb_trigger_tax(self, card_id, controller) {
                // Source was sacrificed; remaining fires are moot.
                return;
            }
            let auto_target =
                self.auto_target_for_effect_avoiding(&effect, controller, Some(card_id));
            // CR 700.2b — modal ETB trigger mode pick at push-time.
            let mode = self.pick_trigger_mode(&effect, card_id);
            for _ in 0..multiplier {
                self.stack.push(StackItem::Trigger {
                    source: card_id,
                    controller,
                    effect: Box::new(effect.clone()),
                    target: auto_target.clone(),
                    mode,
                    x_value: 0,
                    converged_value: 0,
                trigger_source: None,
                    mana_spent: 0,
                    event_amount: 0,
                    intervening_if: None,
                });
            }
        }
    }

    // ── Cast spell ────────────────────────────────────────────────────────────

    /// Cast a modal-double-faced card via its back face. Mirrors
    /// `play_land_with_face` but for non-land back faces (creature /
    /// instant / sorcery). The card's `definition` is swapped to the back
    /// face's definition before payment + cast, so cost / type / effect
    /// all resolve against the back face. Errors with `NotALand` (reused
    /// for "no back face") if the front has no `back_face`.
    pub(crate) fn cast_spell_back_face(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.players[p].has_in_hand(card_id) {
            return Err(GameError::CardNotInHand(card_id));
        }
        // Snapshot the front-face definition AND look up the back face.
        // On a rejected cast the inner path returns the card to the hand
        // with its (then-swapped) back-face definition; we restore the
        // front face here so the player can still cast either face on
        // retry. Without the restore, one failed back-face cast burns the
        // front face for the rest of the game.
        let (front_def, back_def) = {
            let card = self
                .players[p]
                .hand
                .iter()
                .find(|c| c.id == card_id)
                .expect("has_in_hand verified");
            let back = match card.definition.back_face.clone() {
                Some(b) => *b,
                None => return Err(GameError::NotALand(card_id)),
            };
            (card.definition.clone(), back)
        };
        // Swap the in-hand definition to the back face in place. The
        // hand card's back_face slot is kept (it points at the back
        // we just installed), so a later restore can flip back without
        // a second catalog lookup.
        if let Some(c) = self.players[p].hand.iter_mut().find(|c| c.id == card_id) {
            c.definition = std::sync::Arc::new(back_def);
        }
        // Delegate to the regular cast path. The back face's cost,
        // type, target filters, and effect now drive validation.
        // Tag the cast face so the SpellCast event surfaces it.
        self.pending_cast_face = CastFace::Back;
        let result = self.cast_spell(card_id, target, additional_targets, mode, x_value);
        self.pending_cast_face = CastFace::Front;
        // On rejection, the inner cast pushed the card (with the
        // back-face definition) back into the hand. Restore the front
        // face so the player can retry either face.
        if result.is_err()
            && let Some(c) = self.players[p].hand.iter_mut().find(|c| c.id == card_id)
        {
            c.definition = front_def;
        }
        result
    }

    pub(crate) fn cast_spell(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &[], &[], false)
    }

    /// CR 702.32 — cast a spell paying its optional Kicker cost. The kicker
    /// mana is added to the spell's cost and the resolving spell is stamped
    /// `kicked`, which `Predicate::SpellWasKicked` reads.
    pub(crate) fn cast_spell_kicked(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &[], &[], true)
    }

    /// Cast a spell with `Keyword::Delve` (CR 702.66), exiling each card in
    /// `delve_cards` from the caster's graveyard to pay {1} of the spell's
    /// generic cost. Each listed card must be in the caster's graveyard and
    /// the spell must have `Keyword::Delve`. The graveyard cards are not
    /// physically exiled until the (reduced) mana cost is successfully paid,
    /// so a failed payment leaves the graveyard untouched.
    pub(crate) fn cast_spell_with_delve(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        delve_cards: &[CardId],
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &[], delve_cards, false)
    }

    /// Internal cast-spell helper with optional convoke creatures and delve
    /// cards. Each convoke creature must be untapped + controlled by the
    /// caster + the spell must have `Keyword::Convoke`; each tap adds {1}
    /// generic mana to the player's pool. Each delve card must be in the
    /// caster's graveyard + the spell must have `Keyword::Delve`; each one
    /// exiled reduces the generic cost by {1} (CR 702.66).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn cast_spell_with_convoke(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        convoke_creatures: &[CardId],
        delve_cards: &[CardId],
        kicked: bool,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;

        if !self.players[p].has_in_hand(card_id) {
            return Err(GameError::CardNotInHand(card_id));
        }
        let mut card = self.players[p].remove_from_hand(card_id).unwrap();
        card.cast_from_hand = true;
        // CR 702.32 — opt-in Kicker. Only stamp `kicked` if the card
        // actually has a kicker cost; the cost itself is folded into the
        // spell's mana cost below.
        let kicked = kicked && card.definition.has_kicker().is_some();
        card.kicked = kicked;

        // Validate convoke creatures up-front (before any state mutation).
        if !convoke_creatures.is_empty()
            && !card.definition.keywords.contains(&crate::card::Keyword::Convoke)
        {
            self.players[p].hand.push(card);
            return Err(GameError::SorcerySpeedOnly); // reuse: spell doesn't have convoke
        }
        for cid in convoke_creatures {
            let bad = !self.battlefield.iter().any(|c| {
                c.id == *cid
                    && c.controller == p
                    && c.definition.is_creature()
                    && !c.tapped
            });
            if bad {
                self.players[p].hand.push(card);
                return Err(GameError::CardNotOnBattlefield(*cid));
            }
        }

        // Validate delve cards up-front (CR 702.66): the spell must have
        // Keyword::Delve and every listed card must currently sit in the
        // caster's graveyard. The cards aren't exiled here — only after the
        // reduced cost is paid — so a rejected cast leaves them in place.
        if !delve_cards.is_empty()
            && !card.definition.keywords.contains(&crate::card::Keyword::Delve)
        {
            self.players[p].hand.push(card);
            return Err(GameError::SorcerySpeedOnly); // reuse: spell doesn't have delve
        }
        for cid in delve_cards {
            if !self.players[p].graveyard.iter().any(|c| c.id == *cid) {
                self.players[p].hand.push(card);
                return Err(GameError::CardNotInGraveyard(*cid));
            }
        }

        // Timing: sorcery-speed requires empty stack + main phase + active player priority.
        // Instant-speed (Instant type or Flash) may be cast whenever you have priority.
        // Teferi, Time Raveler's +1 sets `sorceries_as_flash` on its
        // controller — those casters can ignore the sorcery-timing gate
        // until their next turn (when do_untap clears the flag).
        // Teferi's static (`OpponentsSorceryTimingOnly`) flips the rule for
        // opponents: even instants must wait until their main phase.
        let must_be_sorcery_speed = !card.definition.is_instant_speed()
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed
            && !self.can_cast_sorcery_speed(p)
            && !self.players[p].sorceries_as_flash
        {
            self.players[p].hand.push(card);
            return Err(GameError::SorcerySpeedOnly);
        }

        // Validate that the chosen target is legally targetable. Also
        // enforces CR 115.5 — the spell being cast cannot target itself —
        // by threading the cast card's id as `source`.
        if let Some(ref tgt) = target
            && let Err(e) = self.check_target_legality_with_source(tgt, p, Some(card_id))
        {
            self.players[p].hand.push(card);
            return Err(e);
        }
        // Same legality check for each additional target slot (hexproof,
        // shroud, protection, Leyline-of-Sanctity, CR 115.5 self-target).
        for tgt in &additional_targets {
            if let Err(e) = self.check_target_legality_with_source(tgt, p, Some(card_id)) {
                self.players[p].hand.push(card);
                return Err(e);
            }
        }

        // CR 702.16: Protection from [color] prevents targeting by spells
        // of that color. Check the spell's colors against the target's
        // protection keywords.
        if let Some(Target::Permanent(cid)) = target
            && let Some(target_card) = self.battlefield_find(cid)
            && target_card.controller != p
        {
            let spell_colors = card.definition.cost.colors();
            for kw in &target_card.definition.keywords {
                if let Keyword::Protection(prot_color) = kw
                    && spell_colors.contains(prot_color)
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
            }
        }

        // Enforce the spell's target selection requirement (e.g. Terror's
        // "non-black, non-artifact creature"): if the effect binds a filter to
        // slot N and the chosen target doesn't match, reject the cast.
        // For modal cards (`ChooseMode`), only look at the chosen mode's
        // filter — Drown in the Loch's mode 0 (counter spell) and mode 1
        // (destroy creature) have incompatible filters, and the legacy
        // "first match across all modes" path picked mode 0's `IsSpellOnStack`
        // even when the caster picked mode 1.
        // Multi-target spells (Snow Day, Render Speechless, Crackle with
        // Power) thread additional slots through `additional_targets`.
        if let Some(ref tgt) = target
            && let Some(filter) = card
                .definition
                .effect
                .target_filter_for_slot_in_mode_kicked(0, mode, kicked)
            && !self.evaluate_requirement_static(filter, tgt, p, Some(card.id))
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }
        for (idx, tgt) in additional_targets.iter().enumerate() {
            let slot = (idx + 1) as u8;
            if let Some(filter) = card
                .definition
                .effect
                .target_filter_for_slot_in_mode_kicked(slot, mode, kicked)
                && !self.evaluate_requirement_static(filter, tgt, p, Some(card.id))
            {
                self.players[p].hand.push(card);
                return Err(GameError::SelectionRequirementViolated);
            }
        }

        // CR 601.2b — additional cast costs ("As an additional cost to cast
        // this spell, sacrifice / discard …"). Validate payability up front
        // so an unpayable spell reverts to hand before any mana is spent;
        // the costs themselves are paid after the mana cost succeeds.
        let additional_costs = card.definition.additional_cast_cost.clone();
        if !additional_costs.is_empty() && !self.additional_costs_payable(p, &additional_costs) {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // Pay the cost (substitute X if present, then add any
        // static-ability tax such as Damping Sphere's "{1} more after the
        // first spell each turn").
        let base_cost = card.definition.cost.clone();
        let mut cost = if base_cost.has_x() {
            base_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            base_cost
        };
        // CR 702.32b — fold the optional kicker cost into the total cost.
        if kicked && let Some(kick) = card.definition.has_kicker() {
            cost.symbols.extend(kick.symbols.iter().cloned());
        }
        let tax = extra_cost_for_spell(self, p, &card);
        if tax > 0 {
            cost.symbols.push(crate::mana::ManaSymbol::Generic(tax));
        }
        // Apply static cost-reduction effects (Killian's "spells that target
        // a creature cost {2} less"). Tax is applied first so reductions
        // never make the spell free of its tax.
        let reduction = cost_reduction_for_spell(self, p, &card, target.as_ref());
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        // Delve (CR 702.66): each graveyard card to be exiled pays {1} of the
        // generic cost. The reduction is clamped to the generic portion by
        // `reduce_generic`; the cards themselves are exiled only after a
        // successful payment (below).
        if !delve_cards.is_empty() {
            cost.reduce_generic(delve_cards.len() as u32);
        }
        // Trinisphere floor (CR 117.7 / replacement-style): applied after
        // every reduction so a discounted spell still owes the minimum.
        apply_spell_cost_floor(self, &mut cost);

        // Snapshot pristine state before convoke + auto-tap mutate it, so a
        // failed payment can revert both convoke taps and any lands that
        // auto-tap tapped.
        let snapshot = self.snapshot_payment_state(p);

        // Convoke: tap each chosen creature and credit the player's pool
        // with {1} generic per creature. (The full Oracle also lets the
        // creature pay one mana of its own color identity; for now every
        // tap pays {1}.) Convoke discounts can't reduce the cost below
        // colored requirements — those still come from real mana sources.
        for cid in convoke_creatures {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == *cid) {
                c.tapped = true;
            }
            self.players[p].mana_pool.add_colorless(1);
        }

        let forced_only = self.players[p].wants_ui;
        // Spell kind gates spend-restricted mana ("spend only to cast
        // instant and sorcery spells"). Only an I/S spell may draw on it.
        let spell_kind = if card
            .definition
            .card_types
            .iter()
            .any(|t| matches!(t, CardType::Instant | CardType::Sorcery))
        {
            crate::mana::SpellKind::InstantOrSorcery
        } else {
            crate::mana::SpellKind::Other
        };
        let receipt = match self.try_pay_after_snapshot_mode(p, &cost, snapshot, forced_only, spell_kind) {
            Ok(r) => r,
            Err(e) => {
                self.players[p].hand.push(card);
                return Err(e);
            }
        };
        if receipt.side_effects.life_lost > 0 {
            self.adjust_life(p, -(receipt.side_effects.life_lost as i32));
        }

        // Delve payment succeeded — exile the chosen graveyard cards now
        // (CR 702.66: they're exiled as part of paying the cost). Bumps the
        // per-turn exile tally so "if cards were exiled this turn" payoffs see
        // them.
        for cid in delve_cards {
            if let Some(pos) = self.players[p].graveyard.iter().position(|c| c.id == *cid) {
                let exiled = self.players[p].graveyard.remove(pos);
                self.exile.push(exiled);
                self.players[p].cards_exiled_this_turn += 1;
            }
        }

        // Compute converge: count distinct colors of mana drained from the
        // pool by paying the cost. Convoke pips contribute generic only,
        // so they don't raise this count.
        let converged_value = converge_count(&receipt.pool_before, &self.players[p].mana_pool);
        // Total mana spent — `pool_before.total() - pool_after.total()`.
        // Read by `Value::CastSpellManaSpent` for Increment / Opus payoffs.
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());

        let mut auto_events = receipt.auto_events;
        auto_events.push(GameEvent::SpellCast {
            player: p,
            card_id,
            face: self.pending_cast_face,
        });

        // CR 601.2h — pay the additional cast costs now (during casting), so
        // sacrifice/discard triggers fire before the spell resolves. A
        // sacrifice reports the fodder's power, which becomes the spell's X
        // for "X = the sacrificed creature's power" riders (Tend the Pests).
        let mut sac_x = None;
        if !additional_costs.is_empty() {
            let (mut cost_events, power) = self.pay_additional_costs(p, &additional_costs);
            auto_events.append(&mut cost_events);
            sac_x = power;
        }
        let events = auto_events;
        let final_x = x_value.unwrap_or(0).max(sac_x.unwrap_or(0));

        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            final_x,
            converged_value,
            mana_spent,
        );

        Ok(events)
    }

    /// CR 601.2b — can every additional cast cost be paid right now? Checked
    /// before mana so an unpayable spell reverts cleanly.
    pub(crate) fn additional_costs_payable(
        &self,
        p: usize,
        costs: &[crate::card::AdditionalCastCost],
    ) -> bool {
        use crate::card::AdditionalCastCost as A;
        costs.iter().all(|c| match c {
            A::SacrificePermanent { filter, count } => {
                let matching = self.battlefield.iter().filter(|c| {
                    c.controller == p
                        && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p, None)
                }).count();
                matching >= *count as usize
            }
            A::Discard { count } => self.players[p].hand.len() >= *count as usize,
        })
    }

    /// CR 601.2h — pay each additional cast cost immediately. Returns the
    /// emitted events plus, for a sacrifice, the sacrificed permanent's
    /// power (threaded into the spell's X).
    pub(crate) fn pay_additional_costs(
        &mut self,
        p: usize,
        costs: &[crate::card::AdditionalCastCost],
    ) -> (Vec<GameEvent>, Option<u32>) {
        use crate::card::AdditionalCastCost as A;
        let mut events = Vec::new();
        let mut sac_power = None;
        for cost in costs {
            match cost {
                A::SacrificePermanent { filter, count } => {
                    // Auto-pick the `count` cheapest matching permanents (tokens
                    // first, then lowest mana value, then lowest power). The
                    // first sacrifice's power becomes the spell's X.
                    let chosen: Vec<(CardId, u32, bool)> = {
                        let mut cands: Vec<&crate::card::CardInstance> = self
                            .battlefield
                            .iter()
                            .filter(|c| {
                                c.controller == p
                                    && self.evaluate_requirement_static(
                                        filter,
                                        &Target::Permanent(c.id),
                                        p,
                                        None,
                                    )
                            })
                            .collect();
                        cands.sort_by_key(|c| {
                            (!c.is_token, c.definition.cost.cmc(), c.power())
                        });
                        cands
                            .iter()
                            .take(*count as usize)
                            .map(|c| (c.id, c.power().max(0) as u32, c.definition.is_creature()))
                            .collect()
                    };
                    for (idx, (id, power, is_creature)) in chosen.into_iter().enumerate() {
                        if idx == 0 {
                            sac_power = Some(power);
                        }
                        if is_creature {
                            if let Some(c) = self.battlefield_find(id) {
                                self.died_card_snapshots.insert(id, c.clone());
                            }
                            events.push(GameEvent::CreatureSacrificed { card_id: id, who: p });
                            events.push(GameEvent::CreatureDied { card_id: id });
                        }
                        events.push(GameEvent::PermanentSacrificed { card_id: id, who: p });
                        let mut die = self.remove_to_graveyard_with_triggers(id);
                        events.append(&mut die);
                    }
                }
                A::Discard { count } => {
                    for _ in 0..*count {
                        let pick = self.players[p].hand.first().map(|c| c.id);
                        if let Some(id) = pick {
                            self.discard_card(p, id, &mut events);
                        }
                    }
                }
            }
        }
        (events, sac_power)
    }

    /// Common post-cost-payment bookkeeping for the three cast paths
    /// (`cast_spell_with_convoke`, `cast_flashback`, `cast_spell_alternative`).
    ///
    /// 1. Bumps game-wide and per-player `spells_cast_this_turn` (Storm).
    /// 2. Consumes one Chancellor-of-the-Annex first-spell tax charge.
    /// 3. Stamps `StackItem::Spell.uncounterable` if the caster controls a
    ///    Cavern of Souls of the matching type or the card itself is
    ///    uncounterable.
    /// 4. Pushes the spell onto the stack, then pushes its
    ///    `EventKind::SpellCast` + `EventScope::SelfSource` triggers ABOVE
    ///    it so they resolve first (and still fire if the spell is
    ///    countered in response).
    /// 5. Resets priority to the active player so the cast can be responded
    ///    to.
    // The argument list is wide because each cast path's resolution-time
    // context (`x_value`, `converged_value`, `mana_spent`) must reach
    // `finalize_cast` without round-tripping through a struct. The three
    // callers (regular cast, flashback, alternative cast) all hand off
    // the scalars directly from local variables computed during payment.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn finalize_cast(
        &mut self,
        p: usize,
        card: crate::card::CardInstance,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: u32,
        converged_value: u32,
        mana_spent: u32,
    ) {
        let card_id = card.id;
        self.spells_cast_this_turn += 1;
        self.players[p].spells_cast_this_turn += 1;
        // Refine the spell-type tallies. Both gates default to 0 on
        // snapshot back-compat (player.rs `#[serde(default)]`).
        if card.definition.card_types.contains(&CardType::Instant)
            || card.definition.card_types.contains(&CardType::Sorcery)
        {
            self.players[p].instants_or_sorceries_cast_this_turn += 1;
        }
        if card.definition.is_creature() {
            self.players[p].creatures_cast_this_turn += 1;
        }
        // Veil of Summer gate: note when a player casts a blue or black
        // spell (color read off the printed mana cost).
        {
            let colors = card.definition.cost.colors();
            if colors.contains(&crate::mana::Color::Blue)
                || colors.contains(&crate::mana::Color::Black)
            {
                self.players[p].cast_blue_or_black_this_turn = true;
            }
        }
        consume_first_spell_tax(self, p);

        let on_cast_triggers = collect_self_cast_triggers(&card);
        let uncounterable = self.caster_grants_uncounterable_with_x(p, &card, x_value);

        let was_creature_spell = card.definition.is_creature();
        // CR 702.40 — Storm: when this spell is cast, copy it for each spell
        // cast before it this turn. `spells_cast_this_turn` already includes
        // this spell (bumped above), so prior spells = count - 1. Capture the
        // bits needed to mint copies before `card` is moved onto the stack.
        let storm_copies = card
            .definition
            .keywords
            .contains(&Keyword::Storm)
            .then(|| {
                (
                    card.definition.clone(),
                    self.spells_cast_this_turn.saturating_sub(1),
                )
            });

        self.stack.push(StackItem::Spell {
            card: Box::new(card),
            caster: p,
            target: target.clone(),
            additional_targets: additional_targets.clone(),
            mode,
            x_value,
            converged_value,
            mana_spent,
            uncounterable,
        });
        // Push Storm copies above the original (they resolve first, CR 702.40).
        // Each is a token copy that can't be countered, inheriting target/mode.
        if let Some((def, n)) = storm_copies {
            for _ in 0..n {
                let new_id = self.next_id();
                let mut copy_inst = crate::card::CardInstance::new(new_id, def.clone(), p);
                copy_inst.is_token = true;
                self.stack.push(StackItem::Spell {
                    card: Box::new(copy_inst),
                    caster: p,
                    target: target.clone(),
                    additional_targets: additional_targets.clone(),
                    mode,
                    x_value,
                    converged_value,
                    mana_spent: 0,
                    uncounterable: true,
                });
            }
        }
        self.push_on_cast_triggers(card_id, p, on_cast_triggers);
        // SpellCast / YourControl triggers (Prowess, Magecraft, Repartee, …)
        // fire *at cast time*, before the spell resolves. The trigger goes
        // on the stack above the spell so it resolves first (and still
        // fires if the spell itself is countered in response). Filters
        // (e.g. CastSpellTargetsMatch) read the just-cast spell's target
        // from the stack while the spell still sits there.
        //
        // Threads `mana_spent` (and X / Converge) into the trigger context
        // so Increment / Opus payoffs reading `Value::CastSpellManaSpent`
        // observe the actual amount paid for *this* spell.
        self.fire_spell_cast_triggers(p, card_id, !was_creature_spell, mana_spent, converged_value);
        // CR 702.21: Ward triggers on each chosen target permanent the caster
        // doesn't control. Pushed last so Ward sits on top of the caster's
        // own SpellCast triggers (Magecraft, Prowess) — correct APNAP order
        // since the caster is the active player and Ward belongs to a
        // nonactive player. Ward resolves first and may counter the spell
        // unless the caster pays the Ward cost.
        self.push_ward_triggers_for_cast(p, card_id);
        // BecameTarget triggers — fired through the unified dispatcher so
        // APNAP order is correct and the trigger's `EventSpec.filter` is
        // honored. One event per permanent target on the just-pushed
        // spell. Used by SOS Tenured Concocter's "may draw" trigger.
        self.dispatch_became_target_events_for_cast(p, card_id);
        self.give_priority_to_active();
    }

    /// Walk the just-pushed `StackItem::Spell` and emit one
    /// `GameEvent::BecameTarget` for every permanent target slot, then
    /// dispatch the events through the unified trigger pipeline.
    pub(crate) fn dispatch_became_target_events_for_cast(
        &mut self,
        caster: usize,
        cast_card_id: CardId,
    ) {
        let (target, additional_targets): (Option<Target>, Vec<Target>) = match self
            .stack
            .iter()
            .rev()
            .find_map(|si| match si {
                StackItem::Spell { card, target, additional_targets, .. }
                    if card.id == cast_card_id =>
                {
                    Some((target.clone(), additional_targets.clone()))
                }
                _ => None,
            }) {
            Some(t) => t,
            None => return,
        };
        let events: Vec<GameEvent> = target
            .into_iter()
            .chain(additional_targets)
            .filter_map(|t| match t {
                Target::Permanent(id) => Some(GameEvent::BecameTarget {
                    target: id,
                    caster,
                }),
                _ => None,
            })
            .collect();
        if !events.is_empty() {
            self.dispatch_triggers_for_events(&events);
        }
    }

    /// CR 702.21 — push a Ward triggered ability onto the stack for each
    /// target permanent (controlled by another player) that has
    /// `Keyword::Ward(WardCost)`. Each trigger is `Effect::CounterUnless`
    /// aimed at the just-cast spell. At resolution the engine auto-pays
    /// on the spell controller's behalf if affordable; otherwise the
    /// spell is countered.
    ///
    /// Reads slot 0 + every `additional_targets` slot off the just-pushed
    /// `StackItem::Spell` (so this must run after `finalize_cast`'s push).
    /// Trivial Ward variants (e.g. `WardCost::Mana` with an empty/zero
    /// cost) are skipped — a $0 pay is always affordable and the visible
    /// outcome is identical to no Ward at all, so we save the stack churn.
    pub(crate) fn push_ward_triggers_for_cast(&mut self, caster: usize, cast_card_id: CardId) {
        // Locate the just-pushed spell and pull its targets out as owned
        // values — we can't hold an immutable borrow while we push new
        // stack items below.
        let (target, additional_targets): (Option<Target>, Vec<Target>) = match self
            .stack
            .iter()
            .rev()
            .find_map(|si| match si {
                StackItem::Spell { card, target, additional_targets, .. }
                    if card.id == cast_card_id =>
                {
                    Some((target.clone(), additional_targets.clone()))
                }
                _ => None,
            }) {
            Some(t) => t,
            // Spell isn't on the stack (e.g. countered before this hook).
            None => return,
        };

        let all_targets: Vec<Target> = target
            .into_iter()
            .chain(additional_targets)
            .collect();
        self.push_ward_triggers_for_targets(caster, cast_card_id, &all_targets);
    }

    /// Shared core for Ward enforcement: walk `targets`, and for each
    /// permanent target controlled by a player other than `actor` whose
    /// `Keyword::Ward(WardCost)` is non-trivial, push a Ward trigger
    /// above whatever is currently on top of the stack. The trigger's
    /// `target` carries `target_for_trigger` — the spell card-id (for
    /// casts) or the source permanent's id (for activated abilities) —
    /// so `Effect::CounterUnless` can walk the stack for the topmost
    /// matching `Spell` or `Trigger`.
    pub(crate) fn push_ward_triggers_for_targets(
        &mut self,
        actor: usize,
        target_for_trigger: CardId,
        targets: &[Target],
    ) {
        use crate::card::{Keyword, WardCost};
        use crate::effect::Selector;

        for tgt in targets {
            let perm_id = match tgt {
                Target::Permanent(id) => *id,
                _ => continue,
            };
            let (ward_cost, ward_controller) = match self
                .battlefield
                .iter()
                .find(|c| c.id == perm_id)
            {
                Some(c) if c.controller != actor => {
                    let computed = self.computed_permanent(perm_id);
                    let cost: Option<WardCost> = computed
                        .as_ref()
                        .map(|cp| cp.keywords.as_slice())
                        .unwrap_or(&c.definition.keywords)
                        .iter()
                        .find_map(|k| match k {
                            Keyword::Ward(cost) => Some(cost.clone()),
                            _ => None,
                        });
                    match cost {
                        Some(cc) if !ward_cost_is_trivial(&cc) => (cc, c.controller),
                        _ => continue,
                    }
                }
                _ => continue,
            };

            let effect = Effect::CounterUnless {
                what: Selector::Target(0),
                cost: ward_cost,
            };
            self.stack.push(StackItem::Trigger {
                source: perm_id,
                controller: ward_controller,
                effect: Box::new(effect),
                target: Some(Target::Permanent(target_for_trigger)),
                mode: None,
                x_value: 0,
                converged_value: 0,
                trigger_source: None,
                mana_spent: 0,
                event_amount: 0,
                intervening_if: None,
            });
        }
    }

    /// CR 702.21 — Ward enforcement on activated-ability targeting. Hooked
    /// into `activate_ability` immediately after the ability is pushed
    /// onto the stack as a `StackItem::Trigger`. The Ward trigger's
    /// `Effect::CounterUnless` walks the stack for the topmost matching
    /// `Trigger` whose `source` is the activating permanent (identifying
    /// the ability to counter).
    pub(crate) fn push_ward_triggers_for_activated_ability(
        &mut self,
        activator: usize,
        ability_source: CardId,
        target: Option<Target>,
    ) {
        let targets: Vec<Target> = target.into_iter().collect();
        self.push_ward_triggers_for_targets(activator, ability_source, &targets);
    }

    /// Push pre-collected `SpellCast`/`SelfSource` triggers from the
    /// just-cast card onto the stack as `Trigger` items, so they resolve
    /// before the spell itself. Caller is responsible for collecting the
    /// effect list before the card moves into the stack item.
    pub(crate) fn push_on_cast_triggers(
        &mut self,
        source: CardId,
        controller: usize,
        triggers: Vec<(Option<crate::card::Predicate>, Effect)>,
    ) {
        for (filter, effect) in triggers {
            // Evaluate the trigger's filter (Infusion's
            // LifeGainedThisTurnAtLeast, etc.) before pushing. The ctx
            // is the on-cast context where `trigger_source` points to
            // the cast card. If the filter rejects, drop the trigger
            // silently — matches the "won't trigger unless the
            // condition is met" wording.
            if let Some(pred) = &filter {
                let ctx = crate::game::effects::EffectContext {
                    controller,
                    source: Some(source),
                    targets: vec![],
                    trigger_source: Some(crate::game::effects::EntityRef::Card(source)),
                    mode: 0,
                    x_value: 0,
                    converged_value: 0,
                    mana_spent: 0,
                    source_name: None,
                    cast_from_hand: true,
                    event_amount: 0,
                    kicked: false,
                };
                if !self.evaluate_predicate(pred, &ctx) {
                    continue;
                }
            }
            let auto_target =
                self.auto_target_for_effect_avoiding(&effect, controller, Some(source));
            self.stack.push(StackItem::Trigger {
                source,
                controller,
                effect: Box::new(effect),
                target: auto_target,
                mode: None,
                x_value: 0,
                converged_value: 0,
                // Self-cast trigger fires from the cast card itself —
                // carry its CardId as the trigger source so
                // Effect::CopySpell can find it on the stack.
                trigger_source: Some(crate::game::effects::EntityRef::Card(source)),
                mana_spent: 0,
                event_amount: 0,
                intervening_if: None,
            });
        }
    }

    /// Cast a spell from the graveyard using its Flashback cost.
    pub(crate) fn cast_flashback(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;

        // Find the card in the controller's graveyard.
        let graveyard_pos = self.players[p]
            .graveyard
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;

        let card = self.players[p].graveyard[graveyard_pos].clone();

        // The card must have Flashback — printed, or granted until end of
        // turn (the SOS "Flashback" instant).
        let flashback_cost = card
            .effective_flashback()
            .ok_or(GameError::SorcerySpeedOnly)?
            .clone();

        // Timing: instants can be cast at instant speed, others at sorcery
        // speed. Honor Teferi-style opponent restriction.
        let must_be_sorcery_speed = !card.definition.is_instant_speed()
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }

        // CR 702.34a — flashback-only additional costs ("Flashback—Sacrifice
        // a Mountain"; Dread Return's "sacrifice three creatures"). Keyed by
        // card name (the idiom for rare riders that would otherwise bloat
        // every CardDefinition literal). Reject up front if unpayable so no
        // mana is spent on an uncastable flashback.
        let flashback_additional = flashback_additional_cost_for_name(card.definition.name);
        if !flashback_additional.is_empty()
            && !self.additional_costs_payable(p, &flashback_additional)
        {
            return Err(GameError::SelectionRequirementViolated);
        }

        // Validate target.
        if let Some(ref tgt) = target {
            self.check_target_legality(tgt, p)?;
            // Ward enforcement happens via push_ward_triggers_for_cast
            // after finalize_cast, not as a synchronous cost payment.
            let _ = tgt; let _ = p;
        }

        // Pay the flashback cost.
        let mut cost = if flashback_cost.has_x() {
            flashback_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            flashback_cost
        };
        // Flashback IS a cast (CR 702.34a), so Killian-style target-aware
        // cost reductions apply the same as for hand casts. Drain
        // generic-only pips after substituting X.
        let reduction = cost_reduction_for_spell(self, p, &card, target.as_ref());
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        apply_spell_cost_floor(self, &mut cost);
        let forced_only = self.players[p].wants_ui;
        let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
        if receipt.side_effects.life_lost > 0 {
            self.adjust_life(p, -(receipt.side_effects.life_lost as i32));
        }
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());

        // Pay the flashback-only additional cost(s) now (CR 601.2h). A
        // sacrifice can drop cards into this player's graveyard, shifting
        // indices, so recompute the flashback card's position afterward.
        let mut cost_events = Vec::new();
        if !flashback_additional.is_empty() {
            let (mut e, _sac_power) = self.pay_additional_costs(p, &flashback_additional);
            cost_events.append(&mut e);
        }
        let graveyard_pos = self.players[p]
            .graveyard
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;

        let mut events = self.finalize_flashback_cast(
            p,
            card_id,
            graveyard_pos,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            mana_spent,
        )?;
        // Sacrifice/discard events precede the cast events in the log.
        cost_events.append(&mut events);
        Ok(cost_events)
    }

    /// Shared tail for `cast_flashback` and `cast_flashback_tap`:
    /// remove the card from its owner's graveyard, mark it
    /// `cast_via_flashback` so the resolver exiles it (CR 702.34d),
    /// emit `CardLeftGraveyard` + `SpellCast{Flashback}`, and thread the
    /// rest through `finalize_cast`.
    #[allow(clippy::too_many_arguments)]
    fn finalize_flashback_cast(
        &mut self,
        p: usize,
        card_id: CardId,
        graveyard_pos: usize,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: u32,
        mana_spent: u32,
    ) -> Result<Vec<GameEvent>, GameError> {
        let mut card = self.players[p].graveyard.remove(graveyard_pos);
        self.players[p].cards_left_graveyard_this_turn =
            self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
        card.cast_via_flashback = true;
        let events = vec![
            GameEvent::CardLeftGraveyard { player: p, card_id },
            GameEvent::SpellCast {
                player: p,
                card_id,
                face: CastFace::Flashback,
            },
        ];
        self.finalize_cast(p, card, target, additional_targets, mode, x_value, 0, mana_spent);
        Ok(events)
    }

    /// Cast a graveyard card with `Keyword::Retrace` (CR 702.81): pay its
    /// normal mana cost plus discard a land card from hand. The spell
    /// returns to the graveyard after resolving (no exile), so a single
    /// land + the card can be recast every turn.
    pub(crate) fn cast_retrace(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let graveyard_pos = self.players[p]
            .graveyard
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        let card = self.players[p].graveyard[graveyard_pos].clone();
        if !card.definition.has_retrace() {
            return Err(GameError::SorcerySpeedOnly);
        }
        let must_be_sorcery_speed = !card.definition.is_instant_speed()
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // Additional cost: a land card in hand to discard. Reject before
        // paying mana if none is available.
        let land_in_hand = self.players[p]
            .hand
            .iter()
            .find(|c| c.definition.is_land())
            .map(|c| c.id)
            .ok_or(GameError::SelectionRequirementViolated)?;
        if let Some(ref tgt) = target {
            self.check_target_legality(tgt, p)?;
        }
        // Pay the printed mana cost (Retrace doesn't change it).
        let mut cost = if card.definition.cost.has_x() {
            card.definition.cost.with_x_value(x_value.unwrap_or(0))
        } else {
            card.definition.cost.clone()
        };
        let reduction = cost_reduction_for_spell(self, p, &card, target.as_ref());
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        apply_spell_cost_floor(self, &mut cost);
        let receipt = self.try_pay_with_auto_tap(p, &cost)?;
        if receipt.side_effects.life_lost > 0 {
            self.adjust_life(p, -(receipt.side_effects.life_lost as i32));
        }
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        // Discard the land as the additional cost (routes through the
        // central discard path so discard-matters triggers fire).
        let mut events = Vec::new();
        self.discard_card(p, land_in_hand, &mut events);

        // Lift the card from the graveyard and cast it normally — no
        // `cast_via_flashback`, so it returns to the graveyard on
        // resolution and can be retraced again.
        let card = self.players[p].graveyard.remove(graveyard_pos);
        self.players[p].cards_left_graveyard_this_turn =
            self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
        events.push(GameEvent::CardLeftGraveyard { player: p, card_id });
        events.push(GameEvent::SpellCast { player: p, card_id, face: CastFace::Front });
        self.finalize_cast(
            p, card, target, additional_targets, mode, x_value.unwrap_or(0), 0, mana_spent,
        );
        Ok(events)
    }

    /// Cast a graveyard card with `Keyword::Escape` (CR 702.139): pay its
    /// escape mana cost plus exiling `exile_cards` (exactly N other cards
    /// from the caster's graveyard). Instants/sorceries resolve back to
    /// the graveyard (so they can be escaped again); permanents enter the
    /// battlefield normally.
    pub(crate) fn cast_escape(
        &mut self,
        card_id: CardId,
        exile_cards: &[CardId],
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let graveyard_pos = self.players[p]
            .graveyard
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        let card = self.players[p].graveyard[graveyard_pos].clone();
        let (escape_cost, exile_count) = card
            .definition
            .has_escape()
            .map(|(c, n)| (c.clone(), n))
            .ok_or(GameError::SorcerySpeedOnly)?;
        let must_be_sorcery_speed = !card.definition.is_instant_speed()
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // The exile set must be exactly N distinct *other* graveyard cards.
        if exile_cards.len() as u32 != exile_count {
            return Err(GameError::SelectionRequirementViolated);
        }
        for cid in exile_cards {
            if *cid == card_id
                || !self.players[p].graveyard.iter().any(|c| c.id == *cid)
            {
                return Err(GameError::SelectionRequirementViolated);
            }
        }
        let mut seen = exile_cards.to_vec();
        seen.sort_unstable();
        seen.dedup();
        if seen.len() != exile_cards.len() {
            return Err(GameError::SelectionRequirementViolated);
        }
        if let Some(ref tgt) = target {
            self.check_target_legality(tgt, p)?;
        }
        // Pay the escape mana cost (with X substitution + reductions).
        let mut cost = if escape_cost.has_x() {
            escape_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            escape_cost
        };
        let reduction = cost_reduction_for_spell(self, p, &card, target.as_ref());
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        apply_spell_cost_floor(self, &mut cost);
        let forced_only = self.players[p].wants_ui;
        let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
        if receipt.side_effects.life_lost > 0 {
            self.adjust_life(p, -(receipt.side_effects.life_lost as i32));
        }
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        // Pay the additional cost: exile the chosen graveyard cards.
        let mut events = Vec::new();
        for cid in exile_cards {
            if let Some(pos) = self.players[p].graveyard.iter().position(|c| c.id == *cid) {
                let exiled = self.players[p].graveyard.remove(pos);
                events.push(GameEvent::CardLeftGraveyard { player: p, card_id: *cid });
                self.exile.push(exiled);
            }
        }
        // Lift the escaping card from the graveyard and cast it normally —
        // no `cast_via_flashback`, so an instant/sorcery returns to the
        // graveyard on resolution and can be escaped again.
        let pos = self.players[p]
            .graveyard
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        let card = self.players[p].graveyard.remove(pos);
        self.players[p].cards_left_graveyard_this_turn =
            self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
        events.push(GameEvent::CardLeftGraveyard { player: p, card_id });
        events.push(GameEvent::SpellCast { player: p, card_id, face: CastFace::Front });
        self.finalize_cast(
            p, card, target, additional_targets, mode, x_value.unwrap_or(0), 0, mana_spent,
        );
        Ok(events)
    }

    /// Cast a graveyard card with `Keyword::FlashbackTap(N)` by tapping
    /// `tap_creatures` (must be exactly N untapped creatures the caster
    /// controls). The spell costs no mana — the tap is the entire
    /// flashback cost. Routes the resolved card to exile via
    /// `cast_via_flashback` (CR 702.34d). Used by Group Project.
    pub(crate) fn cast_flashback_tap(
        &mut self,
        card_id: CardId,
        tap_creatures: &[CardId],
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let graveyard_pos = self.players[p]
            .graveyard
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        let card = self.players[p].graveyard[graveyard_pos].clone();
        let required_taps = card
            .definition
            .has_flashback_tap()
            .ok_or(GameError::FlashbackTapInvalid)?;
        let must_be_sorcery_speed = !card.definition.is_instant_speed()
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        if tap_creatures.len() as u32 != required_taps {
            return Err(GameError::FlashbackTapInvalid);
        }
        // Validate every creature in `tap_creatures` is currently untapped,
        // controlled by the caster, and a creature.
        for cid in tap_creatures {
            let c = self
                .battlefield
                .iter()
                .find(|c| c.id == *cid)
                .ok_or(GameError::FlashbackTapInvalid)?;
            if c.tapped || c.controller != p || !c.definition.is_creature() {
                return Err(GameError::FlashbackTapInvalid);
            }
        }
        if let Some(ref tgt) = target {
            self.check_target_legality(tgt, p)?;
        }
        // Pay the tap cost: tap every nominated creature.
        for cid in tap_creatures {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == *cid) {
                c.tapped = true;
            }
        }
        self.finalize_flashback_cast(
            p,
            card_id,
            graveyard_pos,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
        )
    }

    /// Cast a spell from an arbitrary zone (graveyard or exile) without
    /// paying any mana cost. The card is lifted out of `source_zone`,
    /// stamped with `cast_via_flashback = true` when `exile_after` is
    /// set (so the resolver routes it to exile), and threaded through
    /// `finalize_cast` with `mana_spent = 0` / `converged_value = 0`.
    /// No timing / target / cost / alt-cost checks run — the caller is
    /// responsible for any timing window enforcement at the call site.
    ///
    /// Used by:
    /// - `GameAction::CastFromZoneWithoutPaying` (player invokes a
    ///   `may_play_until` permission, e.g. Practiced Scrollsmith's
    ///   exiled card)
    /// - `Effect::CastWithoutPayingImmediate` (immediate resolve-time
    ///   cast — Improvisation Capstone, The Dawning Archaic, Nita)
    /// - `Effect::CastFreeParadigmCopy` (per-main-phase paradigm copy)
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn cast_card_for_free(
        &mut self,
        p: usize,
        card_id: CardId,
        source_zone: crate::card::Zone,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        exile_after: bool,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::Zone;
        // Lift the card out of the named zone. Owner-based zones
        // (graveyard, hand, library) walk all players to locate it.
        let mut card = match source_zone {
            Zone::Exile => {
                let pos = self
                    .exile
                    .iter()
                    .position(|c| c.id == card_id)
                    .ok_or(GameError::CardNotInHand(card_id))?;
                self.exile.remove(pos)
            }
            Zone::Graveyard => {
                let mut found: Option<crate::card::CardInstance> = None;
                for player in self.players.iter_mut() {
                    if let Some(pos) = player.graveyard.iter().position(|c| c.id == card_id) {
                        found = Some(player.graveyard.remove(pos));
                        break;
                    }
                }
                found.ok_or(GameError::CardNotInHand(card_id))?
            }
            Zone::Hand => {
                // Omniscience-style free cast straight from hand.
                let mut found: Option<crate::card::CardInstance> = None;
                for player in self.players.iter_mut() {
                    if let Some(pos) = player.hand.iter().position(|c| c.id == card_id) {
                        found = Some(player.hand.remove(pos));
                        break;
                    }
                }
                found.ok_or(GameError::CardNotInHand(card_id))?
            }
            _ => return Err(GameError::CardNotInHand(card_id)),
        };

        // Clear any outstanding may-play permission — once the card is
        // cast, the grant (and its miracle alt-cost) is consumed.
        card.may_play_until = None;
        card.granted_alt_cast_cost_eot = None;
        // Route to exile on resolve when the granting effect demands it
        // (Nita's "if would go to graveyard, exile instead").
        if exile_after {
            card.cast_via_flashback = true;
        }
        // Bump the "card left graveyard this turn" counter for gy casts.
        if matches!(source_zone, Zone::Graveyard) {
            let owner = card.owner;
            self.players[owner].cards_left_graveyard_this_turn =
                self.players[owner].cards_left_graveyard_this_turn.saturating_add(1);
        }

        let mut events = Vec::new();
        if matches!(source_zone, Zone::Graveyard) {
            events.push(GameEvent::CardLeftGraveyard {
                player: card.owner,
                card_id,
            });
        }
        events.push(GameEvent::SpellCast {
            player: p,
            card_id,
            face: CastFace::Front,
        });
        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            0,
        );
        Ok(events)
    }

    /// `GameAction::CastFromZoneWithoutPaying` entry point. Validates
    /// that the priority-holding player has an outstanding
    /// `may_play_until` permission on `card_id`, that the permission
    /// hasn't expired, and that the printed-Oracle timing (sorcery vs
    /// instant) allows casting at this window — then hands off to
    /// `cast_card_for_free`.
    pub(crate) fn cast_from_zone_without_paying(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let zone = self
            .find_card_zone(card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        // Locate the permission + card definition without holding a long
        // borrow across the cast helper.
        let card_ref = self
            .find_card_anywhere(card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        let is_instant = card_ref.definition.is_instant_speed();
        // A "miracle {N}"-style grant attaches an alternative cast cost to
        // the permission (Lorehold, the Historian). When present, the cast
        // isn't free — the controller pays this cost instead of the card's
        // full mana cost.
        let alt_cast_cost = card_ref.granted_alt_cast_cost_eot.clone();
        // Two ways to invoke a free cast: a per-card `may_play_until`
        // permission (Discovery / Paradigm / etc.), or an Omniscience-style
        // standing static letting the controller free-cast their own hand
        // spells. The latter doesn't exile the spell afterwards (it goes
        // wherever it normally would).
        let exile_after = match card_ref.may_play_until {
            Some(permission) => {
                if permission.player != p {
                    return Err(GameError::CardNotInHand(card_id));
                }
                permission.exile_after
            }
            None => {
                let from_own_hand = zone == crate::card::Zone::Hand
                    && self.players[p].hand.iter().any(|c| c.id == card_id);
                if !(from_own_hand && self.player_casts_hand_spells_free(p)) {
                    return Err(GameError::CardNotInHand(card_id));
                }
                false
            }
        };
        // Expiry check: EndOfThisTurn => only valid this turn;
        // EndOfControllersNextTurn => one full controller-turn later.
        // Defensive — the cleanup hook also clears expired permissions.
        let must_be_sorcery_speed = !is_instant || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // Pay the miracle alt-cost up front, if any. Failure leaves the
        // permission intact so the controller can retry once they have the
        // mana.
        if let Some(cost) = alt_cast_cost {
            let forced_only = self.players[p].wants_ui;
            let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
            if receipt.side_effects.life_lost > 0 {
                self.adjust_life(p, -(receipt.side_effects.life_lost as i32));
            }
        }
        self.cast_card_for_free(
            p,
            card_id,
            zone,
            target,
            additional_targets,
            mode,
            x_value,
            exile_after,
        )
    }

    /// Cast a spell using its `alternative_cost` (a "pitch" cost) instead of
    /// its regular mana cost. Pays the alt cost's mana, deducts life, and
    /// exiles the chosen `pitch_card` from hand if the alt cost requires
    /// Cast a commander from your command zone (Phase L).
    ///
    /// Differences vs. `cast_spell`:
    /// * The card is sourced from `players[p].command` instead of
    ///   `players[p].hand`.
    /// * Cost = printed cost + `{2}` × `commander_cast_count[card_id]`
    ///   (the commander tax, CR 903.8).
    /// * On a successful payment the cast count is bumped so the next
    ///   cast pays `{4}` extra, then `{6}`, etc.
    /// * The Phase J zone-change replacement is already registered for
    ///   each seated commander, so when the resulting permanent
    ///   eventually leaves play it gets snagged back into the command
    ///   zone automatically.
    ///
    /// Sorcery-speed / target legality / etc. piggyback off the same
    /// helpers as `cast_spell_with_convoke` to keep behavior aligned.
    /// X / mode / target slots are threaded through verbatim.
    pub(crate) fn cast_from_command_zone(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;

        // Locate + remove the commander from the caster's command zone.
        let pos = self.players[p]
            .command
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        let mut card = self.players[p].command.remove(pos);
        card.cast_from_hand = false;

        // Sorcery-speed gate (commanders are creatures by definition,
        // which are sorcery-speed unless flash). We rebuild the same
        // gate `cast_spell` uses so timing matches.
        let must_be_sorcery_speed = !card.definition.is_instant_speed()
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed
            && !self.can_cast_sorcery_speed(p)
            && !self.players[p].sorceries_as_flash
        {
            self.players[p].command.push(card);
            return Err(GameError::SorcerySpeedOnly);
        }

        // Target legality (the rare commander with a targeted ETB
        // wants the same hexproof / shroud / Leyline checks).
        if let Some(ref tgt) = target
            && let Err(e) = self.check_target_legality_with_source(tgt, p, Some(card_id))
        {
            self.players[p].command.push(card);
            return Err(e);
        }
        for tgt in &additional_targets {
            if let Err(e) = self.check_target_legality_with_source(tgt, p, Some(card_id)) {
                self.players[p].command.push(card);
                return Err(e);
            }
        }

        // Build the cost: printed + commander tax. The tax is
        // `{2}` × prior casts; it stacks on top of any X / generic
        // tax / cost reduction the spell would normally see.
        let base_cost = card.definition.cost.clone();
        let mut cost = if base_cost.has_x() {
            base_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            base_cost
        };
        let prior = self.commander_cast_count.get(&card_id).copied().unwrap_or(0);
        let commander_tax = prior.saturating_mul(2);
        if commander_tax > 0 {
            cost.symbols
                .push(crate::mana::ManaSymbol::Generic(commander_tax));
        }
        let tax = extra_cost_for_spell(self, p, &card);
        if tax > 0 {
            cost.symbols
                .push(crate::mana::ManaSymbol::Generic(tax));
        }
        let reduction = cost_reduction_for_spell(self, p, &card, target.as_ref());
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        apply_spell_cost_floor(self, &mut cost);

        // Pay. On failure put the card back in the command zone.
        let forced_only = self.players[p].wants_ui;
        let receipt = match self.try_pay_with_auto_tap_mode(p, &cost, forced_only) {
            Ok(r) => r,
            Err(e) => {
                self.players[p].command.push(card);
                return Err(e);
            }
        };
        if receipt.side_effects.life_lost > 0 {
            self.adjust_life(p, -(receipt.side_effects.life_lost as i32));
        }
        let converged_value = converge_count(&receipt.pool_before, &self.players[p].mana_pool);
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());

        // Bump the cast counter on success.
        *self.commander_cast_count.entry(card_id).or_insert(0) += 1;

        let mut auto_events = receipt.auto_events;
        auto_events.push(GameEvent::SpellCast {
            player: p,
            card_id,
            face: self.pending_cast_face,
        });
        let events = auto_events;

        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            converged_value,
            mana_spent,
        );

        Ok(events)
    }

    /// one. The spell otherwise behaves identically to a normal cast (goes
    /// onto the stack, resolves later, etc.).
    pub(crate) fn cast_spell_alternative(
        &mut self,
        card_id: CardId,
        pitch_card: Option<CardId>,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;

        if !self.players[p].has_in_hand(card_id) {
            return Err(GameError::CardNotInHand(card_id));
        }
        // Validate the spell actually has an alternative cost; clone it before
        // any mutation so we don't borrow the card twice.
        let alt = self.players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .ok_or(GameError::CardNotInHand(card_id))?
            .definition
            .alternative_cost
            .clone()
            .ok_or(GameError::NoAlternativeCost)?;

        // Force of Negation–style "you may pay this alt cost only if it's
        // not your turn." Reject the alt cast on the caster's own turn —
        // they can still pay the regular mana cost via `cast_spell`.
        if alt.not_your_turn_only && self.active_player_idx == p {
            return Err(GameError::NoAlternativeCost);
        }

        // Optional cast-time predicate gate. Used by
        // SOS Wilt in the Heat's "{2} less if cards left your graveyard
        // this turn" rider, where the alt cost is only legal under a
        // specific game-state condition. Rejected before any state
        // mutation (no card removal, no mana payment) so callers can
        // retry the cast via the regular cost path.
        if let Some(cond) = &alt.condition {
            let ctx = crate::game::effects::EffectContext {
                controller: p,
                source: Some(card_id),
                targets: vec![],
                trigger_source: None,
                mode: 0,
                x_value: 0,
                converged_value: 0,
                mana_spent: 0,
                source_name: None,
                cast_from_hand: true,
                event_amount: 0,
                kicked: false,
            };
            if !self.evaluate_predicate(cond, &ctx) {
                return Err(GameError::NoAlternativeCost);
            }
        }

        // CR 119.4: A player can only pay an amount of life if their
        // life total is greater than or equal to the payment. Pre-flight
        // gate so we reject cleanly rather than driving life negative
        // mid-cast.
        if alt.life_cost > 0 && self.players[p].life < alt.life_cost as i32 {
            return Err(GameError::InsufficientLife);
        }

        // Pre-flight: confirm the caster has enough cards in their
        // graveyard for the `exile_from_graveyard_count` additional
        // cost. Picks are committed AFTER the mana payment succeeds
        // (mirroring `exile_other_filter` on activated abilities) so a
        // failed mana pay rolls back cleanly. The auto-picker takes the
        // lowest-CMC matching cards so higher-value graveyard cards
        // stay put.
        let exile_gy_picks: Vec<CardId> = if alt.exile_from_graveyard_count > 0 {
            let n = alt.exile_from_graveyard_count as usize;
            let mut picks: Vec<(CardId, i32)> = self.players[p]
                .graveyard
                .iter()
                .map(|c| (c.id, c.definition.cost.cmc() as i32))
                .collect();
            picks.sort_by_key(|(_, cmc)| *cmc);
            if picks.len() < n {
                return Err(GameError::SelectionRequirementViolated);
            }
            picks.into_iter().take(n).map(|(cid, _)| cid).collect()
        } else {
            Vec::new()
        };

        // Return-N-permanents-to-hand additional cost (Gush / Daze). Pick
        // the matches up front so a shortfall rejects before any mana is
        // paid; commit the moves after payment succeeds. Auto-picker
        // prefers untapped permanents (lower tempo loss).
        let return_picks: Vec<CardId> = if let Some((filter, count)) = &alt.return_to_hand {
            let n = *count as usize;
            let mut matches: Vec<(CardId, bool)> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == p
                        && self.evaluate_requirement_static(
                            filter,
                            &Target::Permanent(c.id),
                            p,
                            None,
                        )
                })
                .map(|c| (c.id, c.tapped))
                .collect();
            matches.sort_by_key(|(_, tapped)| *tapped);
            if matches.len() < n {
                return Err(GameError::SelectionRequirementViolated);
            }
            matches.into_iter().take(n).map(|(cid, _)| cid).collect()
        } else {
            Vec::new()
        };

        // Sacrifice-N-permanents additional cost (Fireblast). Same up-front
        // pick / late commit discipline as `return_picks`.
        let sacrifice_picks: Vec<CardId> = if let Some((filter, count)) = &alt.sacrifice_permanents {
            let n = *count as usize;
            let matches: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == p
                        && self.evaluate_requirement_static(
                            filter,
                            &Target::Permanent(c.id),
                            p,
                            None,
                        )
                })
                .map(|c| c.id)
                .collect();
            if matches.len() < n {
                return Err(GameError::SelectionRequirementViolated);
            }
            matches.into_iter().take(n).collect()
        } else {
            Vec::new()
        };

        // Validate that the pitch card matches the filter (if any).
        if let Some(filter) = &alt.exile_filter {
            let pitch_id = pitch_card.ok_or(GameError::NoAlternativeCost)?;
            // The pitch card must be in hand AND match the filter. The filter
            // typically refers to spell colors (e.g. HasColor(Blue)) so we
            // evaluate it against the card's definition rather than against
            // a battlefield CardInstance.
            let pitch_card_inst = self.players[p]
                .hand
                .iter()
                .find(|c| c.id == pitch_id)
                .ok_or(GameError::InvalidPitchCard(pitch_id))?;
            // The pitch card must not be the spell itself.
            if pitch_id == card_id {
                return Err(GameError::InvalidPitchCard(pitch_id));
            }
            if !self.evaluate_requirement_on_card(filter, pitch_card_inst, p) {
                return Err(GameError::InvalidPitchCard(pitch_id));
            }
        }

        // Remove the spell card from hand now (so the pitch card doesn't
        // accidentally collide with it during validation).
        let mut card = self.players[p].remove_from_hand(card_id).unwrap();
        card.cast_from_hand = true;
        if alt.evoke_sacrifice {
            card.evoked = true;
        }
        if alt.dash {
            card.dashed = true;
        }

        // Timing: sorcery-speed unless instant-speed (or the alt cost grants
        // flash — Rout), plus Teferi-style opponent restriction.
        let must_be_sorcery_speed = (!card.definition.is_instant_speed() && !alt.flash)
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            self.players[p].hand.push(card);
            return Err(GameError::SorcerySpeedOnly);
        }

        // Validate target legality.
        if let Some(ref tgt) = target
            && let Err(e) = self.check_target_legality(tgt, p)
        {
            self.players[p].hand.push(card);
            return Err(e);
        }
        // When the alt cost carries an effect_override, use its target
        // filter instead of the base spell's (kicker-style alt modes
        // change the legal target set). Otherwise, validate against the
        // base spell's filter.
        {
            let effect_for_filter = alt.effect_override.as_ref()
                .unwrap_or(&card.definition.effect);
            if let Some(ref tgt) = target
                && let Some(filter) = effect_for_filter
                    .target_filter_for_slot_in_mode(0, mode)
                && !self.evaluate_requirement_static(filter, tgt, p, Some(card.id))
            {
                self.players[p].hand.push(card);
                return Err(GameError::SelectionRequirementViolated);
            }
        }
        // Alt-cost-specific target filter (e.g. Mystical Dispute's "target
        // must be a blue spell"). Applied on top of the spell's regular
        // target filter, only on the alternative-cast path.
        if let Some(ref tgt) = target
            && let Some(ref alt_filter) = alt.target_filter
            && !self.evaluate_requirement_static(alt_filter, tgt, p, Some(card.id))
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // Pay the alt mana cost (with X substitution + static-ability tax + Ward).
        let mut mana_cost = if alt.mana_cost.has_x() {
            alt.mana_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            alt.mana_cost.clone()
        };
        let tax = extra_cost_for_spell(self, p, &card);
        if tax > 0 {
            mana_cost.symbols.push(crate::mana::ManaSymbol::Generic(tax));
        }
        // CR 601.2f: cost reductions apply uniformly across cast paths
        // (hand cast / flashback / alt-cost), and `cost_reduction_for_
        // spell` returns the same delta in each. The alt cost is often
        // {0} for pitch spells (Force of Negation, Mystical Dispute), in
        // which case the reduction simply no-ops (clamps at zero).
        let reduction = cost_reduction_for_spell(self, p, &card, target.as_ref());
        if reduction > 0 {
            mana_cost.reduce_generic(reduction);
        }
        apply_spell_cost_floor(self, &mut mana_cost);
        let forced_only = self.players[p].wants_ui;
        let receipt = match self.try_pay_with_auto_tap_mode(p, &mana_cost, forced_only) {
            Ok(r) => r,
            Err(e) => {
                self.players[p].hand.push(card);
                return Err(e);
            }
        };
        if receipt.side_effects.life_lost > 0 {
            self.adjust_life(p, -(receipt.side_effects.life_lost as i32));
        }
        let alt_mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        let mut auto_events = receipt.auto_events;

        // Pay the life portion of the alt cost.
        if alt.life_cost > 0 {
            self.adjust_life(p, -(alt.life_cost as i32));
            auto_events.push(GameEvent::LifeLost {
                player: p,
                amount: alt.life_cost,
            });
        }

        // Exile the pitch card from hand if required.
        if alt.exile_filter.is_some()
            && let Some(pitch_id) = pitch_card
            && let Some(pitch) = self.players[p].remove_from_hand(pitch_id)
        {
            let cid = pitch.id;
            self.exile.push(pitch);
            auto_events.push(GameEvent::PermanentExiled { card_id: cid });
        }

        // Exile-N-from-graveyard additional cost. Validated up front;
        // commit the moves now that mana and life are paid. Emits
        // `CardLeftGraveyard` per exile so payoffs that count cards
        // leaving the graveyard (Ark of Hunger, Wilt in the Heat) see
        // the event stream.
        for gy_cid in &exile_gy_picks {
            if let Some(idx) = self.players[p].graveyard.iter().position(|c| c.id == *gy_cid) {
                let exiled = self.players[p].graveyard.remove(idx);
                self.exile.push(exiled);
                self.players[p].cards_exiled_this_turn =
                    self.players[p].cards_exiled_this_turn.saturating_add(1);
                auto_events.push(GameEvent::CardLeftGraveyard {
                    player: p,
                    card_id: *gy_cid,
                });
                self.players[p].cards_left_graveyard_this_turn =
                    self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
            }
        }

        // Sacrifice additional cost: sacrifice the picked permanents now
        // that mana/life are paid (CR 701.16). Fires dies/sacrifice triggers.
        for sac_cid in &sacrifice_picks {
            if self.battlefield_find(*sac_cid).is_some() {
                auto_events.push(GameEvent::PermanentSacrificed { card_id: *sac_cid, who: p });
                let mut die_evs = self.remove_to_graveyard_with_triggers(*sac_cid);
                auto_events.append(&mut die_evs);
            }
        }

        // Return-to-hand additional cost: bounce the picked permanents to
        // their owners' hands now that mana/life are paid. Reuses the full
        // `move_card_to` battlefield-exit path (combat removal, continuous-
        // effect cleanup, linked-exile return).
        for ret_cid in &return_picks {
            let owner = self.battlefield_find(*ret_cid).map(|c| c.owner);
            if let Some(owner) = owner {
                let ret_ctx = EffectContext::for_spell(owner, None, 0, 0);
                self.move_card_to(
                    *ret_cid,
                    &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::You),
                    &ret_ctx,
                    &mut auto_events,
                );
            }
        }

        // Overload / alt-effect override: swap the spell's resolution
        // effect to the alternative version so it resolves with "each"
        // instead of "target" semantics (or whatever the override says).
        if let Some(override_effect) = alt.effect_override {
            std::sync::Arc::make_mut(&mut card.definition).effect = override_effect;
        }

        auto_events.push(GameEvent::SpellCast {
            player: p,
            card_id,
            face: CastFace::Front,
        });
        let events = auto_events;
        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            alt_mana_spent,
        );
        Ok(events)
    }


    /// True if `player` is restricted to sorcery-only spell timing by an
    /// opponent's `StaticEffect::OpponentsSorceryTimingOnly` (Teferi, Time
    /// Raveler's static "Each opponent can cast spells only any time they
    /// could cast a sorcery"). Walked by `cast_spell` ahead of the
    /// is-instant-speed timing check.
    pub(crate) fn player_locked_to_sorcery_timing(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller != player
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::OpponentsSorceryTimingOnly)
                })
        })
    }

    /// Validate that a target is legally targetable by the given controller.
    ///
    /// Returns an error if the target has Hexproof (opponent) or Shroud (anyone),
    /// or has Protection from the caster's color identity. For player targets,
    /// also checks the `ControllerHasHexproof` static (Leyline of Sanctity).
    ///
    /// **Ward** — if the target permanent has `Keyword::Ward(n)` and the caster
    /// is an opponent, the caster must have `{n}` generic mana available.
    /// This check is read-only; use `pay_ward_cost` after a successful check
    /// to actually deduct the mana.
    pub(crate) fn check_target_legality(&self, target: &Target, caster: usize) -> Result<(), GameError> {
        self.check_target_legality_with_source(target, caster, None)
    }

    /// Same as [`check_target_legality`] but also enforces CR 115.5 — "a
    /// spell or ability on the stack is an illegal target for itself" —
    /// when `source_card_id` is provided. Used by the cast pipeline for
    /// spells like Stifle/Squelch that target stack spells/abilities;
    /// passing the casting spell's own `CardId` rejects a self-target
    /// at cast time.
    pub(crate) fn check_target_legality_with_source(
        &self,
        target: &Target,
        caster: usize,
        source_card_id: Option<CardId>,
    ) -> Result<(), GameError> {
        let cid = match target {
            Target::Player(p) => {
                if *p != caster && self.player_has_static_hexproof(*p) {
                    return Err(GameError::TargetHasHexproof(crate::card::CardId(0)));
                }
                return Ok(());
            }
            Target::Permanent(c) => c,
        };
        // CR 115.5: A spell or ability on the stack is an illegal target
        // for itself. When the cast pipeline passes its own source id,
        // reject a target matching that id. Catches Spell Burst /
        // hypothetical "counter target spell" trying to point at itself
        // mid-cast.
        if let Some(src) = source_card_id
            && *cid == src
        {
            return Err(GameError::InvalidTarget);
        }
        let Some(card) = self.battlefield_find(*cid) else {
            return Ok(());
        };
        if card.has_keyword(&Keyword::Shroud) {
            return Err(GameError::TargetHasShroud(*cid));
        }
        if card.has_keyword(&Keyword::Hexproof) && card.controller != caster {
            return Err(GameError::TargetHasHexproof(*cid));
        }
        // Ward is enforced via triggered abilities on the stack (CR 702.21a),
        // not as a pre-flight targeting restriction. The caster CAN target a
        // Ward creature — the Ward trigger fires and counters the spell unless
        // the caster pays the Ward cost at resolution time.
        Ok(())
    }

    /// True if `player` controls any permanent granting "you have hexproof"
    /// via `StaticEffect::ControllerHasHexproof` (Leyline of Sanctity).
    pub(crate) fn player_has_static_hexproof(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::ControllerHasHexproof))
        })
    }

    // Note: `fire_ward_triggers` (the old Ward(u32) version) was removed
    // during the merge — Ward is now enforced via
    // `push_ward_triggers_for_cast` (CR 702.21) which handles the full
    // `WardCost` enum (Mana / Life / Discard / SacrificeCreature).

    /// Push `SpellCast` triggered abilities (e.g. Prowess, Up the Beanstalk)
    /// onto the stack. They will resolve when priority is passed through.
    /// `cast_card` is the id of the spell that just resolved (or just got
    /// cast); the trigger's optional `EventSpec::filter` predicate is
    /// evaluated with `Selector::TriggerSource` bound to this card so
    /// "whenever you cast a spell with property X" filters can read the
    /// cast spell's mana value, color, type, etc.
    pub(crate) fn fire_spell_cast_triggers(
        &mut self,
        controller: usize,
        cast_card: CardId,
        _is_noncreature: bool,
        mana_spent: u32,
        converged_value: u32,
    ) {
        use crate::effect::{EventKind, EventScope};
        // CR 113.10b — permanents under a "loses all abilities" continuous
        // effect (Mercurial Transformation / Turn to Frog) don't fire
        // printed Magecraft / spell-cast triggers. Pre-compute the stripped
        // set so the filter below can drop those listeners.
        let stripped: std::collections::HashSet<CardId> = self
            .compute_battlefield()
            .into_iter()
            .filter(|c| c.lost_all_abilities)
            .map(|c| c.id)
            .collect();
        // Whether the cast spell came from its caster's hand, read off the
        // stack item. Stamped into the trigger context so
        // `Predicate::CastFromHand` (Quandrix, the Proof) reflects the
        // actual cast: cascade / flashback / exile casts read `false`,
        // which stops "spells you cast from your hand have cascade" from
        // re-triggering on the spells it cascades into.
        let cast_from_hand = self
            .stack
            .iter()
            .find_map(|item| match item {
                crate::game::types::StackItem::Spell { card, .. } if card.id == cast_card => {
                    Some(card.cast_from_hand)
                }
                _ => None,
            })
            .unwrap_or(true);
        // Walk every permanent on the battlefield — `YourControl` triggers
        // fire from the caster's permanents, while `OpponentControl` triggers
        // fire from non-caster permanents (Wandering Archaic etc.). The
        // ability's effective controller is its own permanent's controller,
        // *not* the spell-caster's index.
        let candidates: Vec<(CardId, usize, Effect, Option<crate::effect::Predicate>)> = self
            .battlefield
            .iter()
            .filter(|c| !stripped.contains(&c.id))
            .flat_map(|c| {
                let c_controller = c.controller;
                c.definition
                    .triggered_abilities
                    .iter()
                    .filter(move |t| {
                        if t.event.kind != EventKind::SpellCast {
                            return false;
                        }
                        match t.event.scope {
                            // "Whenever you cast …" — listener's controller
                            // must equal the caster.
                            EventScope::YourControl => c_controller == controller,
                            // "Whenever an opponent casts …" — listener's
                            // controller must differ from the caster.
                            EventScope::OpponentControl => c_controller != controller,
                            // "Whenever any player casts …" — always fires
                            // regardless of caster.
                            EventScope::AnyPlayer => true,
                            // Other scopes don't apply to a SpellCast event.
                            _ => false,
                        }
                    })
                    .map(move |t| (c.id, c_controller, t.effect.clone(), t.event.filter.clone()))
            })
            .collect();

        for (source, listener_controller, effect, filter) in candidates {
            if let Some(filter) = filter {
                let ctx = crate::game::effects::EffectContext {
                    controller: listener_controller,
                    source: Some(source),
                    targets: vec![],
                    trigger_source: Some(crate::game::effects::EntityRef::Card(cast_card)),
                    mode: 0,
                    x_value: 0,
                    converged_value,
                    mana_spent,
                    source_name: None,
                    cast_from_hand,
                    event_amount: 0,
                    kicked: false,
                };
                if !self.evaluate_predicate(&filter, &ctx) {
                    continue;
                }
            }
            let auto_target = self.auto_target_for_effect_avoiding(
                &effect,
                listener_controller,
                Some(source),
            );
            // CR 700.2b — pick the mode at push time if the trigger is modal.
            // Powers Prismari Apprentice's modal Magecraft (Scry 1 / +1/+0 EOT):
            // AutoDecider picks mode 0 (Scry); ScriptedDecider::new([Mode(1)])
            // exercises the pump branch.
            let mode = self.pick_trigger_mode(&effect, source);
            self.stack.push(StackItem::Trigger {
                source,
                controller: listener_controller,
                effect: Box::new(effect),
                target: auto_target,
                mode,
                x_value: 0,
                // Thread the just-cast spell's converged_value onto the trigger
                // so per-cast `Value::ConvergedValue` reads the iterated spell's
                // color count (Magmablood Archaic's "+1/+0 for each color spent
                // to cast that spell" pump, Wildgrowth Archaic's "X additional
                // counters where X = colors spent on the iterated creature
                // spell"). Previously hard-coded to 0 → triggers reading
                // ConvergedValue would silently no-op for converge fan-out.
                converged_value,
                // The trigger fires on a spell cast — preserve the cast
                // spell's CardId so resolving effects (Effect::CopySpell,
                // Selector::CastSpellTarget) can find it on the stack.
                trigger_source: Some(crate::game::effects::EntityRef::Card(cast_card)),
                mana_spent,
                event_amount: 0,
                intervening_if: None,
            });
        }
    }

    // ── Payment snapshot / restore ───────────────────────────────────────────

    /// Capture mana pool + tapped state of every permanent owned by `payer`.
    /// Used by the cast/activate/counter paths so a payment that fails
    /// mid-way (after auto-tap has already tapped lands) can be reverted to
    /// pristine state.
    pub(crate) fn snapshot_payment_state(&self, payer: usize) -> PaymentSnapshot {
        PaymentSnapshot {
            pool: self.players[payer].mana_pool.clone(),
            tapped: self
                .battlefield
                .iter()
                .filter(|c| c.owner == payer)
                .map(|c| (c.id, c.tapped))
                .collect(),
        }
    }

    /// Restore the mana pool and tapped state captured by a prior
    /// `snapshot_payment_state`. Skips cards that have since left the
    /// battlefield (the caller is responsible for any zone-change rollback).
    pub(crate) fn restore_payment_state(&mut self, payer: usize, snapshot: PaymentSnapshot) {
        self.players[payer].mana_pool = snapshot.pool;
        for (id, was_tapped) in snapshot.tapped {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == id) {
                c.tapped = was_tapped;
            }
        }
    }

    /// Snapshot, auto-tap, and pay `cost` atomically. Returns the auto-tap
    /// events plus the pre-payment pool snapshot (callers like `cast_spell`
    /// use it to compute converge). On payment failure the snapshot is
    /// restored — pool and tapped flags revert to the pre-call state.
    pub(crate) fn try_pay_with_auto_tap(
        &mut self,
        payer: usize,
        cost: &crate::mana::ManaCost,
    ) -> Result<PaymentReceipt, GameError> {
        self.try_pay_with_auto_tap_mode(payer, cost, false)
    }

    /// `try_pay_with_auto_tap`, but `forced_only` gates manual tapping.
    /// When `forced_only` is true (human-initiated casts/activations), the
    /// engine auto-taps *only* when the payment is forced — see
    /// `try_pay_after_snapshot_mode`.
    pub(crate) fn try_pay_with_auto_tap_mode(
        &mut self,
        payer: usize,
        cost: &crate::mana::ManaCost,
        forced_only: bool,
    ) -> Result<PaymentReceipt, GameError> {
        let snapshot = self.snapshot_payment_state(payer);
        // The auto-tap wrappers fund non-spell costs (cycling, equip,
        // ability activations, engine-driven pays). Restricted "cast only"
        // mana never applies to these, so pay as `Other`.
        self.try_pay_after_snapshot_mode(payer, cost, snapshot, forced_only, crate::mana::SpellKind::Other)
    }

    /// Pay `cost` for `payer`, auto-tapping mana sources as needed.
    /// Used with a snapshot the caller already captured — for paths that
    /// mutate state between snapshot and payment (convoke taps creatures
    /// in between, `activate_ability` applies its tap-cost in between).
    ///
    /// `forced_only` implements "proper tapping" for human players (CR
    /// 601.2g — the active player chooses which mana sources to tap):
    /// - If the pool already covers the cost, pay from it directly (the
    ///   player has arranged their mana).
    /// - Otherwise the engine auto-taps **only if the payment is forced** —
    ///   i.e. after a full auto-tap, no untapped source the player controls
    ///   *could have contributed to this cost* remains. If a relevant
    ///   untapped source is left over, the player had a real choice, so the
    ///   cast is rejected with `ManualTapRequired` (rolled back) and they
    ///   tap manually before re-submitting.
    ///
    /// `forced_only` is false for bots, scripted tests, and engine-driven
    /// auto-pays (Counter-unless-paid, "pay X or sacrifice"), which keep
    /// the original full auto-tap behavior.
    pub(crate) fn try_pay_after_snapshot_mode(
        &mut self,
        payer: usize,
        cost: &crate::mana::ManaCost,
        snapshot: PaymentSnapshot,
        forced_only: bool,
        kind: crate::mana::SpellKind,
    ) -> Result<PaymentReceipt, GameError> {
        if forced_only {
            // Fast path: the player already has the mana floating — pay it.
            if self.players[payer].mana_pool.clone().pay_for_spell(cost, kind).is_ok() {
                let pool_before = self.players[payer].mana_pool.clone();
                let side_effects = self.players[payer]
                    .mana_pool
                    .pay_for_spell(cost, kind)
                    .expect("pool covered the cost a line ago");
                return Ok(PaymentReceipt { auto_events: vec![], side_effects, pool_before });
            }
            // Eagerly tap for the *forced colored* pips — their colour can
            // only come from those sources, so there's no choice to leave
            // the player (which Forest pays {G} doesn't matter). This mana
            // stays floating in the pool; if the rest of the cost needs a
            // manual tap, the player only has to tap the ambiguous part.
            let colored_only = crate::mana::ManaCost {
                symbols: cost
                    .symbols
                    .iter()
                    .filter(|s| matches!(s, crate::mana::ManaSymbol::Colored(_)))
                    .copied()
                    .collect(),
            };
            let mut events = if colored_only.symbols.is_empty() {
                Vec::new()
            } else {
                self.auto_tap_for_cost(payer, &colored_only)
            };
            // With the colored pips covered, does the remaining (generic)
            // part still involve a genuine choice of which source to tap?
            if self.payment_requires_manual_choice(payer, cost) {
                // Leave the forced colored mana floating; the player taps
                // the ambiguous remainder, then the cast completes.
                return Err(GameError::ManualTapRequired { cost: cost.summary() });
            }
            // No remaining choice — finish auto-tapping and pay.
            let mut more = self.auto_tap_for_cost(payer, cost);
            events.append(&mut more);
            let pool_after_auto_tap = self.players[payer].mana_pool.clone();
            return match self.players[payer].mana_pool.pay_for_spell(cost, kind) {
                Ok(side_effects) => {
                    Ok(PaymentReceipt { auto_events: events, side_effects, pool_before: pool_after_auto_tap })
                }
                Err(e) => {
                    self.restore_payment_state(payer, snapshot);
                    Err(GameError::Mana(e))
                }
            };
        }

        let auto_events = self.auto_tap_for_cost(payer, cost);
        // Snapshot the pool *after* auto-tap so `pool_before` reflects the
        // mana actually available to `pay()`. Without this, a player who
        // starts with an empty pool and auto-taps lands to cover the cost
        // shows mana_spent = 0 (pre-auto-tap 0 → post-pay 0), which silently
        // breaks Increment / Opus / converge payoffs that read the
        // difference. The original snapshot is still used for rollback.
        let pool_after_auto_tap = self.players[payer].mana_pool.clone();
        match self.players[payer].mana_pool.pay_for_spell(cost, kind) {
            Ok(side_effects) => Ok(PaymentReceipt {
                auto_events,
                side_effects,
                pool_before: pool_after_auto_tap,
            }),
            Err(e) => {
                self.restore_payment_state(payer, snapshot);
                Err(GameError::Mana(e))
            }
        }
    }

    /// True if `player` controls an untapped mana source that *could have
    /// contributed* to `cost` — used by the forced-only payment path to
    /// detect that a manual tapping choice existed. A source is relevant
    /// when the cost has a generic (or monocolored-hybrid) pip that any
    /// mana satisfies, or when the source can produce a color the cost
    /// requires.
    fn untapped_relevant_source_exists(&self, player: usize, cost: &crate::mana::ManaCost) -> bool {
        use crate::mana::ManaSymbol;
        let flexible = cost.symbols.iter().any(|s| {
            matches!(s, ManaSymbol::Generic(n) if *n > 0) || matches!(s, ManaSymbol::MonoHybrid(_, _))
        });
        let cost_colors = cost.colors();
        self.battlefield.iter().any(|c| {
            if c.controller != player || c.tapped {
                return false;
            }
            let mana_abilities = self.effective_mana_abilities(c.id);
            if mana_abilities.is_empty() {
                return false;
            }
            // Any mana pays a generic / mono-hybrid-generic pip.
            if flexible {
                return true;
            }
            // Otherwise the source must make a color the cost needs.
            cost_colors
                .iter()
                .any(|col| mana_abilities.iter().any(|(_, a)| effect_produces_color(&a.effect, *col)))
        })
    }

    /// Does paying `cost` for `player` involve a *genuine* choice of which
    /// mana sources to tap — one the engine shouldn't make for them?
    ///
    /// Returns true only when the cost is affordable from untapped sources
    /// AND the player could tap different sources leaving different mana
    /// behind. Forced colored pips (a colour only one kind of source can
    /// make) and interchangeable sources (two of the same basic) are *not*
    /// choices, so they auto-tap. Hybrid / mono-hybrid costs fall back to
    /// the conservative "any relevant untapped source remains" check.
    ///
    /// Only consulted on the forced-only (human) path when the pool doesn't
    /// already cover the cost; `pay()` is still the source of truth for
    /// whether the resulting tap actually pays.
    fn payment_requires_manual_choice(&self, player: usize, cost: &crate::mana::ManaCost) -> bool {
        use crate::mana::{Color, ManaSymbol};
        use std::collections::{HashMap, HashSet};
        // Hybrids keep the simpler conservative behaviour.
        if cost
            .symbols
            .iter()
            .any(|s| matches!(s, ManaSymbol::Hybrid(_, _) | ManaSymbol::MonoHybrid(_, _)))
        {
            return self.untapped_relevant_source_exists(player, cost);
        }

        // Colored requirement per colour + generic (folding {C} in as
        // generic — payable from any source for the choice analysis).
        let mut need: HashMap<Color, u32> = HashMap::new();
        let mut generic = 0u32;
        for s in &cost.symbols {
            match s {
                ManaSymbol::Colored(c) => *need.entry(*c).or_default() += 1,
                ManaSymbol::Generic(n) | ManaSymbol::Colorless(n) => generic += *n,
                _ => {}
            }
        }

        // Pay colored pips from the pool first; what's left needs sources.
        let pool = &self.players[player].mana_pool;
        let mut pool_used = 0u32;
        let mut colored_from_sources: Vec<(Color, u32)> = Vec::new();
        for (c, k) in &need {
            let from_pool = (*k).min(pool.amount(*c));
            pool_used += from_pool;
            if *k - from_pool > 0 {
                colored_from_sources.push((*c, *k - from_pool));
            }
        }

        // Untapped mana sources, each as its colour-production signature.
        let sigs: Vec<Vec<Color>> = self
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == player
                    && !c.tapped
                    && c.definition
                        .activated_abilities
                        .iter()
                        .any(|a| is_mana_ability(&a.effect))
            })
            .map(source_color_signature)
            .collect();

        // Reserve sources for each still-needed colored pip (most dedicated
        // first). A colour with two *different* source types able to make it
        // is a real choice.
        let mut reserved = vec![false; sigs.len()];
        let mut color_choice = false;
        for (c, rc) in &colored_from_sources {
            let mut cands: Vec<usize> = (0..sigs.len())
                .filter(|i| !reserved[*i] && sigs[*i].contains(c))
                .collect();
            if (cands.len() as u32) < *rc {
                return false; // unaffordable for this colour
            }
            let distinct: HashSet<&Vec<Color>> = cands.iter().map(|i| &sigs[*i]).collect();
            if distinct.len() > 1 {
                color_choice = true;
            }
            cands.sort_by_key(|i| sigs[*i].len()); // dedicated (shortest sig) first
            for &i in cands.iter().take(*rc as usize) {
                reserved[i] = true;
            }
        }

        // Generic: pool leftover first, then the remaining untapped sources.
        let remaining: Vec<&Vec<Color>> =
            (0..sigs.len()).filter(|i| !reserved[*i]).map(|i| &sigs[i]).collect();
        let pool_left = pool.total().saturating_sub(pool_used);
        let gen_from_sources = generic.saturating_sub(pool_left);
        if (remaining.len() as u32) < gen_from_sources {
            return false; // unaffordable
        }
        if color_choice {
            return true;
        }
        if gen_from_sources == 0 {
            return false; // only forced colored taps remain — auto-tap them
        }
        // More candidate sources than the generic needs → some are held
        // back; that's a real choice only if they aren't all interchangeable.
        if (remaining.len() as u32) > gen_from_sources {
            let distinct: HashSet<&&Vec<Color>> = remaining.iter().collect();
            return distinct.len() >= 2;
        }
        false
    }

    // ── Auto-tap mana sources ─────────────────────────────────────────────────

    /// Tap untapped mana sources to cover `cost` for `player`, returning the
    /// events produced. Called before spell/ability payment so the client
    /// doesn't need to manually tap lands before casting.
    ///
    /// `activate_ability` uses `priority.player_with_priority` for permission
    /// checks, but auto-tap may run in contexts where priority is held by
    /// another player (e.g. resolving a Pact upkeep trigger during the
    /// caster's upkeep). We temporarily override priority to `player` so
    /// our `activate_ability` calls don't reject the tap.
    /// Count untapped permanents `player` controls that can tap for
    /// `color` — used by `auto_tap_for_cost_inner` to decide which half
    /// of a hybrid pip is actually producible. Counts an "add any one
    /// color" source (Birds of Paradise, etc.) toward every color, since
    /// the tap loop can script it to the needed color.
    fn untapped_producers_of(&self, player: usize, color: ManaColor) -> u32 {
        self.battlefield
            .iter()
            .filter(|c| {
                c.controller == player
                    && !c.tapped
                    && self.effective_mana_abilities(c.id).iter().any(|(_, a)| {
                        effect_produces_color(&a.effect, color)
                    })
            })
            .count() as u32
    }

    pub(crate) fn auto_tap_for_cost(&mut self, player: usize, cost: &crate::mana::ManaCost) -> Vec<GameEvent> {
        let prev_priority = self.priority.player_with_priority;
        self.priority.player_with_priority = player;
        let events = self.auto_tap_for_cost_inner(player, cost);
        self.priority.player_with_priority = prev_priority;
        events
    }

    fn auto_tap_for_cost_inner(&mut self, player: usize, cost: &crate::mana::ManaCost) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Deduct what the pool already covers before deciding what to tap.
        // We track a "virtual" pool snapshot so we don't mutate the real pool here.
        let pool = &self.players[player].mana_pool;
        let mut avail: std::collections::HashMap<ManaColor, u32> = [
            (ManaColor::White, pool.amount(ManaColor::White)),
            (ManaColor::Blue,  pool.amount(ManaColor::Blue)),
            (ManaColor::Black, pool.amount(ManaColor::Black)),
            (ManaColor::Red,   pool.amount(ManaColor::Red)),
            (ManaColor::Green, pool.amount(ManaColor::Green)),
        ].into_iter().collect();
        let mut avail_colorless = pool.colorless_amount();

        let mut still_need_colors: Vec<ManaColor> = Vec::new();
        // Hybrid pips are resolved after the fixed-color pass so the
        // pool is drained by the more-constrained colored pips first.
        let mut hybrids: Vec<(ManaColor, ManaColor)> = Vec::new();
        let mut generic: u32 = 0;

        for sym in &cost.symbols {
            match sym {
                ManaSymbol::Colored(c) => {
                    let have = avail.entry(*c).or_default();
                    if *have > 0 { *have -= 1; } else { still_need_colors.push(*c); }
                }
                ManaSymbol::Hybrid(a, b) => hybrids.push((*a, *b)),
                ManaSymbol::Phyrexian(c) => {
                    // Pool covers it if available; otherwise paid with life — no tapping.
                    let have = avail.entry(*c).or_default();
                    if *have > 0 { *have -= 1; }
                }
                ManaSymbol::MonoHybrid(n, c) => {
                    // {n/C}: spend a matching colored mana if on hand;
                    // otherwise treat the pip as {n} generic to tap for.
                    let have = avail.entry(*c).or_default();
                    if *have > 0 { *have -= 1; } else { generic += n; }
                }
                ManaSymbol::Generic(n) => generic += n,
                ManaSymbol::Colorless(n) => {
                    // {C} must be paid from the colorless bucket.
                    avail_colorless = avail_colorless.saturating_sub(*n);
                    // If colorless bucket can't cover it fully, we'd need to tap a colorless
                    // source — skip that complexity for now (generic fallback handles it).
                }
                ManaSymbol::Snow | ManaSymbol::X => {}
            }
        }

        // Resolve hybrid pips ({a/b}). Each can be paid by one mana of
        // either color — from the pool, or by tapping a source that makes
        // it. Resolve "forced" pips (only one color reachable) first so a
        // limited board isn't spent on the wrong half: {W/B}{W/B} with a
        // Plains + a Swamp must split W and B, and {W/B} with only a Swamp
        // must tap the Swamp rather than hunting for a white source (the
        // previous code always tried color A and stranded the cast).
        // `reach` = pool mana + untapped sources that can make the color.
        fn reach(
            c: &ManaColor,
            avail: &std::collections::HashMap<ManaColor, u32>,
            prod: &std::collections::HashMap<ManaColor, u32>,
        ) -> u32 {
            avail.get(c).copied().unwrap_or(0) + prod.get(c).copied().unwrap_or(0)
        }
        if !hybrids.is_empty() {
            let mut prod: std::collections::HashMap<ManaColor, u32> = ManaColor::ALL
                .iter()
                .map(|c| (*c, self.untapped_producers_of(player, *c)))
                .collect();
            while !hybrids.is_empty() {
                let idx = hybrids
                    .iter()
                    .position(|(a, b)| (reach(a, &avail, &prod) > 0) ^ (reach(b, &avail, &prod) > 0))
                    .or_else(|| {
                        hybrids.iter().position(|(a, b)| {
                            reach(a, &avail, &prod) > 0 || reach(b, &avail, &prod) > 0
                        })
                    })
                    .unwrap_or(0);
                let (a, b) = hybrids.remove(idx);
                let pick = if reach(&a, &avail, &prod) > 0 { a } else { b };
                if avail.get(&pick).copied().unwrap_or(0) > 0 {
                    // Already in the pool — consume it, no tapping needed.
                    *avail.entry(pick).or_default() -= 1;
                } else if prod.get(&pick).copied().unwrap_or(0) > 0 {
                    // Reserve an untapped source of this color to tap below.
                    *prod.entry(pick).or_default() -= 1;
                    still_need_colors.push(pick);
                } else {
                    // Neither color reachable — push anyway; the tap loop
                    // no-ops and the cast fails downstream as before.
                    still_need_colors.push(pick);
                }
            }
        }

        // Remaining pool total after colored deductions covers generic pips.
        let pool_total_left: u32 = avail.values().sum::<u32>() + avail_colorless;
        let generic_to_tap = generic.saturating_sub(pool_total_left);

        // Tap a color-matched source for each still-needed colored pip.
        // For abilities that produce `AnyOneColor` (Black Lotus, Birds of
        // Paradise, Mox Diamond, etc.) the source's own resolver asks the
        // installed `Decider` which color to add. We temporarily swap in a
        // `ScriptedDecider` that answers with `color`, so the chosen color
        // matches the pip we're trying to satisfy. (Without this, the
        // default `AutoDecider` always picks White and leaves the requested
        // color unfilled.)
        for color in still_need_colors {
            // `controller` not `owner`: a permanent you've stolen
            // (Threaten / Mind Control) is a tap-for-mana source for
            // you, regardless of its original ownership.
            let source = self.battlefield.iter().filter_map(|c| {
                if c.controller != player || c.tapped {
                    return None;
                }
                self.effective_mana_abilities(c.id).into_iter()
                    .find(|(_, a)| effect_produces_color(&a.effect, color))
                    .map(|(idx, _)| (c.id, idx))
            }).next();
            if let Some((id, idx)) = source {
                let scripted = crate::decision::ScriptedDecider::new([
                    crate::decision::DecisionAnswer::Color(color),
                ]);
                let prev_decider = std::mem::replace(
                    &mut self.decider,
                    Box::new(scripted),
                );
                // Force synchronous resolution: if the player normally wants
                // a UI prompt for `AnyOneColor`, auto-tap must still finish
                // inline (otherwise the cast aborts mid-payment with a
                // pending decision). The scripted decider already supplies
                // the right answer.
                let prev_wants_ui = self.players[player].wants_ui;
                self.players[player].wants_ui = false;
                let result = self.activate_ability(id, idx, None, None);
                self.decider = prev_decider;
                self.players[player].wants_ui = prev_wants_ui;
                if let Ok(mut evs) = result {
                    events.append(&mut evs);
                }
            }
        }

        // Tap any mana source for remaining generic pips.
        for _ in 0..generic_to_tap {
            // Same controller-vs-owner fix as the colored-pip loop.
            let source = self.battlefield.iter().filter_map(|c| {
                if c.controller != player || c.tapped {
                    return None;
                }
                self.effective_mana_abilities(c.id).into_iter().next()
                    .map(|(idx, _)| (c.id, idx))
            }).next();
            let Some((id, idx)) = source else { break };
            if let Ok(mut evs) = self.activate_ability(id, idx, None, None) {
                events.append(&mut evs);
            } else {
                break;
            }
        }

        events
    }

    // ── Activate ability ──────────────────────────────────────────────────────

    /// Collect the activated abilities granted to the permanent `card_id`
    /// by `StaticEffect::GrantActivatedAbility` statics in play (Galazeth
    /// Prismari, Cryptolith Rite). Each grant's `applies_to` filter is
    /// evaluated from the static source's controller so "you control"
    /// clauses scope to that player's permanents. Returned in battlefield
    /// order so a permanent's granted-ability indices are stable within a
    /// recompute. Surfaced by `activate_ability` at indices ≥ the
    /// permanent's printed-ability count.
    /// Intrinsic basic-land-type mana abilities for a battlefield
    /// permanent, derived from its *computed* land types (CR 305.6): any
    /// land with a basic land type has the intrinsic `{T}: Add <color>`.
    /// We derive only for basic land types the permanent gained via a
    /// continuous effect (computed but not printed), so printed basics —
    /// which hard-code `tap_add` — aren't double-counted. Lets Blood Moon
    /// / Spreading Seas / Urborg-style type changers tap for the right
    /// colour. Surfaced at ability indices after printed + granted.
    pub(crate) fn intrinsic_land_mana_abilities(
        &self,
        card_id: CardId,
    ) -> Vec<crate::effect::ActivatedAbility> {
        use crate::card::LandType;
        let Some(card) = self.battlefield_find(card_id) else {
            return vec![];
        };
        let printed: &[LandType] = &card.definition.subtypes.land_types;
        let Some(computed) = self.computed_permanent(card_id) else {
            return vec![];
        };
        let mut out = Vec::new();
        for lt in &computed.subtypes.land_types {
            if printed.contains(lt) {
                continue;
            }
            let color = match lt {
                LandType::Plains => ManaColor::White,
                LandType::Island => ManaColor::Blue,
                LandType::Swamp => ManaColor::Black,
                LandType::Mountain => ManaColor::Red,
                LandType::Forest => ManaColor::Green,
                _ => continue,
            };
            out.push(crate::effect::ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: crate::effect::PlayerRef::You,
                    pool: ManaPayload::Colors(vec![color]),
                },
                ..Default::default()
            });
        }
        out
    }

    /// `(index, ability)` for every mana-producing activated ability a
    /// battlefield permanent can currently use — printed, granted, and
    /// intrinsic basic-land — in `activate_ability`'s index order. The
    /// single source of truth for the auto-tap source finders so a land
    /// whose type changed (Spreading Seas / Blood Moon / Urborg) taps for
    /// its computed colours.
    pub(crate) fn effective_mana_abilities(
        &self,
        card_id: CardId,
    ) -> Vec<(usize, crate::effect::ActivatedAbility)> {
        let Some(card) = self.battlefield_find(card_id) else {
            return vec![];
        };
        let printed_count = card.definition.activated_abilities.len();
        let mut out: Vec<(usize, crate::effect::ActivatedAbility)> = Vec::new();
        for (i, a) in card.definition.activated_abilities.iter().enumerate() {
            if is_mana_ability(&a.effect) {
                out.push((i, a.clone()));
            }
        }
        let granted = self.granted_abilities_for(card_id);
        for (j, a) in granted.iter().enumerate() {
            if is_mana_ability(&a.effect) {
                out.push((printed_count + j, a.clone()));
            }
        }
        let gc = granted.len();
        for (k, a) in self.intrinsic_land_mana_abilities(card_id).into_iter().enumerate() {
            out.push((printed_count + gc + k, a));
        }
        out
    }

    pub(crate) fn granted_abilities_for(
        &self,
        card_id: CardId,
    ) -> Vec<crate::effect::ActivatedAbility> {
        use crate::effect::{Selector, StaticEffect};
        if self.battlefield_find(card_id).is_none() {
            return vec![];
        }
        let tgt = Target::Permanent(card_id);
        let mut out = Vec::new();
        for src in &self.battlefield {
            for sa in &src.definition.static_abilities {
                let StaticEffect::GrantActivatedAbility { applies_to, ability } = &sa.effect
                else {
                    continue;
                };
                let Selector::EachPermanent(req) = applies_to else { continue };
                // Evaluate the filter from the granting source's controller
                // so "ControlledByYou" picks that player's permanents.
                if self.evaluate_requirement_static(req, &tgt, src.controller, None) {
                    out.push(ability.clone());
                }
            }
        }
        out
    }

    pub(crate) fn activate_ability(
        &mut self,
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;

        // Source zone: battlefield by default, the controller's graveyard
        // when the ability is flagged `from_graveyard`, or the controller's
        // hand when flagged `from_hand` (Spirit Guides' exile-to-pitch mana
        // abilities). We scan battlefield first; if missing, fall back to
        // graveyards then hands (any player's; ownership is verified below).
        let (source_in_gy, source_in_hand, source_owner) = {
            let on_bf = self.battlefield.iter().any(|c| c.id == card_id);
            if on_bf {
                (false, false, None)
            } else if let Some(o) = self
                .players
                .iter()
                .position(|pl| pl.graveyard.iter().any(|c| c.id == card_id))
            {
                (true, false, Some(o))
            } else if let Some(o) = self
                .players
                .iter()
                .position(|pl| pl.hand.iter().any(|c| c.id == card_id))
            {
                (false, true, Some(o))
            } else {
                return Err(GameError::CardNotOnBattlefield(card_id));
            }
        };

        let ability: crate::effect::ActivatedAbility = if source_in_gy {
            let owner = source_owner.unwrap();
            self.players[owner].graveyard.iter()
                .find(|c| c.id == card_id)
                .and_then(|c| c.definition.activated_abilities.get(ability_index).cloned())
                .ok_or(GameError::AbilityIndexOutOfBounds)?
        } else if source_in_hand {
            let owner = source_owner.unwrap();
            self.players[owner].hand.iter()
                .find(|c| c.id == card_id)
                .and_then(|c| c.definition.activated_abilities.get(ability_index).cloned())
                .ok_or(GameError::AbilityIndexOutOfBounds)?
        } else {
            let pos = self
                .battlefield
                .iter()
                .position(|c| c.id == card_id)
                .ok_or(GameError::CardNotOnBattlefield(card_id))?;
            // CR 113.10b — a permanent with all abilities stripped (Turn to
            // Frog / Mercurial Transformation) can't have its printed
            // activated abilities used. We allow mana abilities through (no
            // catalog card stripping abilities has a mana ability of
            // interest right now) by detecting them via `is_mana_ability`.
            let stripped = self
                .computed_permanent(card_id)
                .map(|c| c.lost_all_abilities)
                .unwrap_or(false);
            // `StaticEffect::GrantActivatedAbility` (Galazeth Prismari,
            // Cryptolith Rite, …): surface granted abilities as virtual
            // activated abilities at indices ≥ printed_count, so standard
            // activate_ability validation and mana payment work without
            // modifying every ability lookup path. Stripped permanents
            // (CR 113.10b) keep their granted mana abilities but lose
            // non-mana grants.
            let printed_count = self.battlefield[pos].definition.activated_abilities.len();
            let granted = self.granted_abilities_for(card_id);
            let intrinsic = self.intrinsic_land_mana_abilities(card_id);
            if ability_index < printed_count {
                let raw = self.battlefield[pos]
                    .definition
                    .activated_abilities[ability_index]
                    .clone();
                if stripped && !is_mana_ability(&raw.effect) {
                    return Err(GameError::AbilityIndexOutOfBounds);
                }
                raw
            } else if ability_index < printed_count + granted.len() {
                let g = granted[ability_index - printed_count].clone();
                // Stripped permanents keep granted mana abilities only.
                if stripped && !is_mana_ability(&g.effect) {
                    return Err(GameError::AbilityIndexOutOfBounds);
                }
                g
            } else if ability_index < printed_count + granted.len() + intrinsic.len() {
                // Intrinsic basic-land mana abilities survive stripping.
                intrinsic[ability_index - printed_count - granted.len()].clone()
            } else {
                return Err(GameError::AbilityIndexOutOfBounds);
            }
        };

        // For graveyard/hand activations, reject if the ability isn't flagged
        // for that zone. This prevents activating a card's printed
        // battlefield-only ability from another zone accidentally.
        if source_in_gy && !ability.from_graveyard {
            return Err(GameError::CardNotOnBattlefield(card_id));
        }
        if source_in_hand && !ability.from_hand {
            return Err(GameError::CardNotOnBattlefield(card_id));
        }

        // Only the controller (or graveyard/hand owner) can activate abilities.
        if source_in_gy || source_in_hand {
            if source_owner != Some(p) {
                return Err(GameError::NotYourPriority);
            }
        } else {
            let pos = self.battlefield.iter().position(|c| c.id == card_id).unwrap();
            if self.battlefield[pos].controller != p {
                return Err(GameError::NotYourPriority);
            }
        }

        // CR 201.3 — Pithing Needle / Phyrexian Revoker: a permanent that
        // named this source's card name shuts off its activated abilities
        // unless they're mana abilities. The suppression is global (affects
        // every player's matching sources), so we scan the whole battlefield
        // for a `named_card` matching this source's printed name.
        if !is_mana_ability(&ability.effect) {
            let source_name = if source_in_gy {
                self.players[source_owner.unwrap()].graveyard.iter()
                    .find(|c| c.id == card_id)
                    .map(|c| c.definition.name)
            } else {
                self.battlefield.iter().find(|c| c.id == card_id).map(|c| c.definition.name)
            };
            if let Some(name) = source_name
                && self.battlefield.iter().any(|c| c.named_card.as_deref() == Some(name))
            {
                return Err(GameError::AbilitySuppressedByNamedCard);
            }
        }

        // Collector Ouphe / Karn lock: non-mana activated abilities of
        // artifacts can't be activated while a `ArtifactActivatedAbilitiesLocked`
        // static is in play (global — affects every player). A source on the
        // battlefield is checked for its artifact type; gy/hand sources of an
        // artifact (rare) are caught the same way.
        if !is_mana_ability(&ability.effect) {
            let src_is_artifact = if source_in_gy {
                self.players[source_owner.unwrap()].graveyard.iter()
                    .find(|c| c.id == card_id)
                    .is_some_and(|c| c.definition.is_artifact())
            } else if source_in_hand {
                self.players[source_owner.unwrap()].hand.iter()
                    .find(|c| c.id == card_id)
                    .is_some_and(|c| c.definition.is_artifact())
            } else {
                self.battlefield_find(card_id).is_some_and(|c| c.definition.is_artifact())
            };
            if src_is_artifact
                && self.battlefield.iter().flat_map(|c| &c.definition.static_abilities).any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::ArtifactActivatedAbilitiesLocked)
                })
            {
                return Err(GameError::AbilitySuppressedByNamedCard);
            }
        }

        // Once-per-turn: reject if this ability index has already been
        // used since the most recent turn-cleanup. The ability is recorded
        // as "used" *after* successful activation below so failed mana
        // payments / illegal targets don't burn the per-turn budget.
        // (Graveyard activations don't track per-card once-per-turn state
        // since the card may move between zones; the gate is no-op.)
        if !source_in_gy && !source_in_hand && ability.once_per_turn {
            let pos = self.battlefield.iter().position(|c| c.id == card_id).unwrap();
            if self.battlefield[pos].once_per_turn_used.contains(&ability_index) {
                return Err(GameError::AbilityAlreadyUsedThisTurn);
            }
        }

        // Sorcery-speed gate: reject the activation if the ability is
        // flagged sorcery-speed and the controller can't currently
        // cast sorceries (not their main phase, or stack non-empty).
        // Used by cards with printed "Activate only as a sorcery" — e.g.
        // SOS Summoned Dromedary, Stone Docent, Cauldron of Essence's
        // reanimation. The pre-fix flow let upkeep activations leak
        // through silently.
        if ability.sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }

        // Per-card "activate only if …" gate. Evaluated against the
        // controller/source context before any cost is paid. Used by
        // cards like Resonating Lute (`{T}: Draw a card. Activate only
        // if you have seven or more cards in your hand.`), Potioner's
        // Trove (`{T}: gain 2 life. Activate only if you've cast an IS
        // spell this turn.`), and similar conditional activations.
        if let Some(cond) = &ability.condition {
            let ctx = crate::game::effects::EffectContext {
                controller: p,
                source: Some(card_id),
                targets: vec![],
                trigger_source: None,
                mode: 0,
                x_value: 0,
                converged_value: 0,
                mana_spent: 0,
                source_name: None,
                cast_from_hand: true,
                event_amount: 0,
                kicked: false,
            };
            if !self.evaluate_predicate(cond, &ctx) {
                return Err(GameError::AbilityConditionNotMet);
            }
        }

        // Reject the activation if the chosen target has hexproof / shroud /
        // protection / Leyline-of-Sanctity-style player hexproof. Mana-only
        // and self-targeting abilities don't pass a target so they bypass.
        if let Some(tgt) = &target {
            self.check_target_legality(tgt, p)?;
            // Ward enforcement happens via push_ward_triggers_for_cast
            // after finalize_cast, not as a synchronous cost payment.
            let _ = tgt; let _ = p;
        }

        // Enforce the ability's own target selection requirement (e.g.
        // Wasteland's "destroy target nonbasic land", Goblin Bombardment's
        // "any target"). Spell casts already validate this in `cast_spell`;
        // activated abilities went unchecked, which let bots/UIs aim a
        // Wasteland at a Plains. Mirror the cast-side gate for parity.
        if let Some(tgt) = &target
            && let Some(filter) = ability.effect.target_filter_for_slot(0)
            && !self.evaluate_requirement_static(filter, tgt, p, Some(card_id))
        {
            return Err(GameError::SelectionRequirementViolated);
        }

        // Pre-flight life-cost gate: reject activation cleanly when the
        // controller doesn't have enough life. Mirror the mana-cost
        // pre-pay check (we want a clean error, not a "you can't pay
        // and just lost a tap" surprise). Activation that gets past
        // this point will deduct the life after tap/mana succeed.
        if ability.life_cost > 0 && self.players[p].life < ability.life_cost as i32 {
            return Err(GameError::InsufficientLife);
        }

        // Pre-flight exile-other-from-gy gate: confirm `count` graveyard
        // cards matching the cost's filter exist, *excluding* the source
        // itself for graveyard activations where source_in_gy is true.
        // Picks the lowest-CMC matching cards (so the activator keeps
        // higher-value cards in their graveyard). If fewer than `count`
        // match, reject cleanly so tap/mana aren't burned. The actual
        // exile happens after payment succeeds.
        let exile_other_picks: Vec<CardId> = if let Some((filter, count)) =
            ability.exile_other_filter.as_ref()
        {
            let count = *count as usize;
            let mut picks: Vec<(CardId, i32)> = self.players[p]
                .graveyard
                .iter()
                .filter(|c| c.id != card_id)
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .map(|c| (c.id, c.definition.cost.cmc() as i32))
                .collect();
            if picks.len() < count {
                return Err(GameError::SelectionRequirementViolated);
            }
            picks.sort_by_key(|(_, cmc)| *cmc);
            picks.into_iter().take(count).map(|(cid, _)| cid).collect()
        } else {
            Vec::new()
        };

        // Pre-flight sacrifice-other gate: confirm `count` battlefield
        // permanents the activator controls match the cost's filter
        // (excluding the source itself, since activating from the
        // battlefield can pair this with `sac_cost: true` for the source).
        // Picks the lowest-power matching permanents (so the activator
        // keeps higher-value creatures alive). If fewer than `count`
        // match, reject cleanly so tap/mana aren't burned.
        let sac_other_picks: Vec<CardId> = if let Some((filter, count)) =
            ability.sac_other_filter.as_ref()
        {
            let count = *count as usize;
            let mut picks: Vec<(CardId, i32)> = self.battlefield
                .iter()
                .filter(|c| c.id != card_id && c.controller == p)
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .map(|c| (c.id, c.power()))
                .collect();
            if picks.len() < count {
                return Err(GameError::SelectionRequirementViolated);
            }
            picks.sort_by_key(|(_, pow)| *pow);
            picks.into_iter().take(count).map(|(cid, _)| cid).collect()
        } else {
            Vec::new()
        };

        // Pre-flight tap-another gate (CR 602.5b): confirm an untapped
        // permanent (other than the source) the activator controls matches
        // the cost's filter. Picks the lowest-power match so higher-value
        // creatures stay open. Tapped after payment succeeds.
        let tap_other_pick: Option<CardId> = if let Some(filter) =
            ability.tap_other_filter.as_ref()
        {
            let pick = self
                .battlefield
                .iter()
                .filter(|c| c.id != card_id && c.controller == p && !c.tapped)
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .min_by_key(|c| c.power())
                .map(|c| c.id);
            match pick {
                Some(id) => Some(id),
                None => return Err(GameError::SelectionRequirementViolated),
            }
        } else {
            None
        };

        // Apply self-counter cost reduction (Strixhaven Book artifacts).
        // Subtracts one generic pip per counter of the specified kind on
        // the source permanent. Clamped at the printed generic total via
        // `ManaCost::reduce_generic`. Only applies when the source is on
        // the battlefield (graveyard-activations skip; the source carries
        // no live counter pool there).
        let mut effective_mana_cost = if ability.mana_cost.has_x() {
            // Bind the X value from the action into the printed cost so
            // the player has to actually pay X generic mana. Used by
            // Pernicious Deed's `{X}, Sacrifice this: …` activation,
            // future Walking Ballista-style `{X}` activations.
            ability.mana_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            ability.mana_cost.clone()
        };
        let activated_x = if ability.mana_cost.has_x() { x_value.unwrap_or(0) } else { 0 };
        if let Some(kind) = ability.self_counter_cost_reduction
            && !source_in_gy
            && let Some(src) = self.battlefield_find(card_id)
        {
            let count = src.counter_count(kind);
            if count > 0 {
                effective_mana_cost.reduce_generic(count);
            }
        }

        // Snapshot pristine state before applying tap-cost so a failed mana
        // payment rolls back both the auto-tap of mana sources AND the
        // tap-cost on the source itself.
        let needs_payment = !effective_mana_cost.symbols.is_empty();
        let pre_snapshot = needs_payment.then(|| self.snapshot_payment_state(p));

        // Pay tap cost. Graveyard activations can't tap (the source is not
        // a permanent), so we reject any `tap_cost: true` ability from a
        // graveyard source as a guard against malformed card definitions.
        if ability.tap_cost {
            if source_in_gy || source_in_hand {
                return Err(GameError::CardIsTapped(card_id));
            }
            let pos = self.battlefield.iter().position(|c| c.id == card_id).unwrap();
            if self.battlefield[pos].tapped {
                return Err(GameError::CardIsTapped(card_id));
            }
            self.battlefield[pos].tapped = true;
        }

        let mut auto_mana_events = Vec::new();
        if let Some(snapshot) = pre_snapshot {
            let forced_only = self.players[p].wants_ui;
            // Activated-ability mana costs are not spell casts; restricted
            // "cast only" mana can never pay them.
            let receipt = self.try_pay_after_snapshot_mode(
                p,
                &effective_mana_cost,
                snapshot,
                forced_only,
                crate::mana::SpellKind::Other,
            )?;
            if receipt.side_effects.life_lost > 0 {
                self.adjust_life(p, -(receipt.side_effects.life_lost as i32));
            }
            auto_mana_events = receipt.auto_events;
        }

        // Pay the life cost. Tap and mana are committed; the life
        // payment is now safe (the pre-flight gate above guaranteed
        // sufficient life). Emits a LifeLost event so trigger / replay
        // observers see the cost.
        if ability.life_cost > 0 {
            self.adjust_life(p, -(ability.life_cost as i32));
            auto_mana_events.push(GameEvent::LifeLost {
                player: p,
                amount: ability.life_cost,
            });
        }

        let mut events = auto_mana_events;
        events.push(GameEvent::AbilityActivated { source: card_id });

        // Mark the ability as used for the once-per-turn budget. (After
        // tap/mana cost validation succeeds, before sacrifice or stack
        // queueing — all of which are guaranteed to commit if we get here.)
        if ability.once_per_turn
            && !source_in_gy
            && let Some(card) = self.battlefield.iter_mut().find(|c| c.id == card_id)
        {
            card.once_per_turn_used.push(ability_index);
        }

        // Sacrifice-as-cost: with tap and mana costs paid, sacrifice the
        // source. The effect runs/queues after, and any selectors that
        // reference the source by id will miss it on the battlefield —
        // which matches the Oracle (sac is part of the activation cost,
        // so the source is in the graveyard by the time the ability
        // resolves). Cards whose effect references self after sacrifice
        // (Greater Good's "draw cards equal to its power") need to
        // capture that data via `Effect::SacrificeAndRemember` instead.
        if ability.sac_cost {
            let is_creature = self
                .battlefield_find(card_id)
                .map(|c| c.definition.is_creature())
                .unwrap_or(false);
            // The activator is the player paying the cost; the
            // sacrifice attribution should match the controller of the
            // sacrificed permanent, which is `p` (priority player) for
            // activated abilities since you only activate abilities of
            // permanents you control.
            let sac_who = self
                .battlefield_find(card_id)
                .map(|c| c.controller)
                .unwrap_or(p);
            if is_creature {
                // Stamp the sacrificed creature's P/T on the resolution
                // scratch so downstream `Value::SacrificedPower` /
                // `Value::SacrificedToughness` reads (Witch's Cauldron's
                // "gain life equal to the sacrificed creature's
                // toughness", future Thud-style `sac_cost` activations)
                // see the right values. Same plumbing as
                // `Effect::SacrificeAndRemember`.
                let snap_pt = self
                    .battlefield_find(card_id)
                    .map(|c| (c.power(), c.toughness(), c.clone()));
                if let Some((p_val, t_val, snap)) = snap_pt {
                    self.sacrificed_power = Some(p_val);
                    self.sacrificed_toughness = Some(t_val);
                    // Cache the dying card's snapshot so AnotherOfYours
                    // triggers and type-filter predicates fire off
                    // sacrifices even when the dying card is a token.
                    self.died_card_snapshots.insert(card_id, snap);
                }
                // CR 701.16 — emit the sacrifice-specific event first.
                events.push(GameEvent::CreatureSacrificed { card_id, who: sac_who });
                events.push(GameEvent::CreatureDied { card_id });
            }
            // Generic permanent-sacrifice event (CR 701.16) — fires for
            // every sacrificed permanent regardless of type so
            // "whenever you sacrifice a permanent" payoffs catch
            // artifact / enchantment / land sacrifices alongside
            // creatures.
            events.push(GameEvent::PermanentSacrificed { card_id, who: sac_who });
            let mut die_evs = self.remove_to_graveyard_with_triggers(card_id);
            events.append(&mut die_evs);
        }

        // Sacrifice-another-from-bf-as-cost: with tap/mana/life paid,
        // sacrifice each cost-picked battlefield permanent (already
        // validated to exist via the pre-flight `sac_other_picks`
        // lookup). Used by cards like Greater Good's `{0}, Sacrifice a
        // creature: …`, Korlash, Heir to Blackblade's `{B}, Sacrifice a
        // Swamp: …`, Witherbloom Harvester-style "sac another creature
        // for an effect" activations.
        for other_cid in sac_other_picks {
            let is_creature = self
                .battlefield_find(other_cid)
                .map(|c| c.definition.is_creature())
                .unwrap_or(false);
            let sac_who = self
                .battlefield_find(other_cid)
                .map(|c| c.controller)
                .unwrap_or(p);
            if is_creature {
                let snap_pt = self
                    .battlefield_find(other_cid)
                    .map(|c| (c.power(), c.toughness(), c.clone()));
                if let Some((p_val, t_val, snap)) = snap_pt {
                    self.sacrificed_power = Some(p_val);
                    self.sacrificed_toughness = Some(t_val);
                    self.died_card_snapshots.insert(other_cid, snap);
                }
                events.push(GameEvent::CreatureSacrificed { card_id: other_cid, who: sac_who });
                events.push(GameEvent::CreatureDied { card_id: other_cid });
            }
            events.push(GameEvent::PermanentSacrificed { card_id: other_cid, who: sac_who });
            let mut die_evs = self.remove_to_graveyard_with_triggers(other_cid);
            events.append(&mut die_evs);
        }

        // Tap-another-as-cost (CR 602.5b): with tap/mana/life paid, tap the
        // pre-selected untapped permanent. Opposition's "Tap an untapped
        // creature you control" cost runs here.
        if let Some(other_cid) = tap_other_pick
            && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == other_cid)
        {
            c.tapped = true;
        }

        // Exile-another-from-gy-as-cost: with tap/mana/life paid, exile
        // each cost-picked graveyard card (already validated to exist via
        // the pre-flight `exile_other_picks` lookup). Used by cards like
        // Postmortem Professor's `{1}{B}, Exile an instant or sorcery
        // card from your graveyard: …` (count 1), Lorehold Pledgemage's
        // `{2}{R}{W}, Exile a card from your graveyard: +1/+1 EOT`
        // (count 1), and Grim Lavamancer's `{R}, {T}, Exile two cards
        // from your graveyard` (count 2).
        for other_cid in exile_other_picks {
            if let Some(idx) = self.players[p]
                .graveyard
                .iter()
                .position(|c| c.id == other_cid)
            {
                let card = self.players[p].graveyard.remove(idx);
                self.exile.push(card);
                self.players[p].cards_exiled_this_turn = self.players[p]
                    .cards_exiled_this_turn
                    .saturating_add(1);
                events.push(GameEvent::CardLeftGraveyard {
                    player: p,
                    card_id: other_cid,
                });
                self.players[p].cards_left_graveyard_this_turn = self.players[p]
                    .cards_left_graveyard_this_turn
                    .saturating_add(1);
            }
        }

        // Exile-self-as-cost (graveyard activations): with tap/mana/life
        // paid, exile the source from the graveyard. This is the cost
        // line for cards like Stone Docent and Eternal Student that read
        // "Exile this card from your graveyard:". The effect then
        // resolves *after* the source is in exile, mirroring `sac_cost`
        // for battlefield sources.
        if ability.exile_self_cost && source_in_gy {
            let owner = source_owner.unwrap();
            if let Some(idx) = self.players[owner].graveyard.iter().position(|c| c.id == card_id) {
                let mut card = self.players[owner].graveyard.remove(idx);
                card.controller = owner;
                self.exile.push(card);
                self.players[owner].cards_exiled_this_turn = self.players[owner]
                    .cards_exiled_this_turn
                    .saturating_add(1);
                // Emit CardLeftGraveyard so Lorehold "cards leave your gy" payoffs fire.
                events.push(GameEvent::CardLeftGraveyard { player: owner, card_id });
                self.players[owner].cards_left_graveyard_this_turn = self.players[owner]
                    .cards_left_graveyard_this_turn
                    .saturating_add(1);
            }
        }

        // Exile-self-as-cost (hand activations): the "Exile this card from
        // your hand:" cost line of the Spirit Guides. Exile happens after
        // the (typically empty) mana cost is paid, before the mana ability
        // resolves.
        if ability.exile_self_cost && source_in_hand {
            let owner = source_owner.unwrap();
            if let Some(idx) = self.players[owner].hand.iter().position(|c| c.id == card_id) {
                let mut card = self.players[owner].hand.remove(idx);
                card.controller = owner;
                self.exile.push(card);
                self.players[owner].cards_exiled_this_turn = self.players[owner]
                    .cards_exiled_this_turn
                    .saturating_add(1);
            }
        }

        // Mana abilities resolve immediately (no stack, no priority reset).
        let is_mana_ab = is_mana_ability(&ability.effect);

        if is_mana_ab {
            let effect = ability.effect.clone();
            let mut ability_events =
                self.continue_ability_resolution(card_id, p, effect, target.clone())?;
            events.append(&mut ability_events);
        } else {
            // Non-mana activated ability goes on the stack.
            let ability_target = target.clone();
            self.stack.push(StackItem::Trigger {
                source: card_id,
                controller: p,
                effect: Box::new(ability.effect),
                target,
                mode: None,
                x_value: activated_x,
                converged_value: 0,
                trigger_source: None,
                mana_spent: 0,
                event_amount: 0,
                intervening_if: None,
            });
            // CR 702.21: Ward also fires on activated abilities targeting
            // an opp's Ward permanent (the "or ability" half of 702.21a).
            // Push Ward triggers above the just-queued ability so they
            // resolve first.
            self.push_ward_triggers_for_activated_ability(p, card_id, ability_target.clone());
            // BecameTarget — fire per permanent target the activation
            // chose (CR 603.x). The unified dispatcher handles APNAP and
            // the trigger filter.
            if let Some(Target::Permanent(target_id)) = ability_target {
                let evs = vec![GameEvent::BecameTarget {
                    target: target_id,
                    caster: p,
                }];
                self.dispatch_triggers_for_events(&evs);
            }
            self.give_priority_to_active();
        }

        Ok(events)
    }
}

/// Pre-payment state captured by `snapshot_payment_state` so a failed
/// payment can revert mana pool and tap-state mutations.
pub(crate) struct PaymentSnapshot {
    pub pool: crate::mana::ManaPool,
    pub tapped: Vec<(CardId, bool)>,
}

/// What a successful payment yields: events from auto-tapping mana sources,
/// any side-effects (Phyrexian life loss), and the pool state from before
/// the payment (for convergence / similar metrics).
pub(crate) struct PaymentReceipt {
    pub auto_events: Vec<GameEvent>,
    pub side_effects: crate::mana::PaymentSideEffects,
    pub pool_before: crate::mana::ManaPool,
}
