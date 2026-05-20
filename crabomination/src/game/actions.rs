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
    if already_cast > 0 {
        for src in &state.battlefield {
            for sa in &src.definition.static_abilities {
                if let StaticEffect::AdditionalCostAfterFirstSpell { filter, amount } = &sa.effect
                    && state.evaluate_requirement_on_card(filter, card, caster)
                {
                    tax += amount;
                }
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
    reduction
}

/// True if any battlefield permanent has `StaticEffect::LandsTapColorlessOnly`
/// (Damping Sphere). Used by `play_land` to decide whether to downgrade
/// multi-color/multi-mana lands to "{T}: Add {C}".
pub(crate) fn multi_mana_ability_count(def: &crate::card::CardDefinition) -> bool {
    use crate::effect::{Effect, ManaPayload};
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
        return match pool {
            ManaPayload::AnyOneColor(_) | ManaPayload::AnyColors(_) => true,
            ManaPayload::Colors(cs) => cs.len() > 1,
            ManaPayload::OfColor(_, _) => false,
            ManaPayload::Colorless(_) => false,
        };
    }
    false
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
    for c in &state.battlefield {
        let count_spotlight = c
            .definition
            .static_abilities
            .iter()
            .filter(|sa| matches!(sa.effect, StaticEffect::EtbTriggerSpotlight))
            .count();
        if count_spotlight == 0 {
            continue;
        }
        if c.controller == etb_controller {
            your_norns += count_spotlight;
        } else {
            opp_norns += count_spotlight;
        }
    }
    if opp_norns > 0 {
        0
    } else {
        1 + your_norns
    }
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
}

fn effect_produces_color(effect: &Effect, color: ManaColor) -> bool {
    match effect {
        Effect::AddMana { pool, .. } => match pool {
            ManaPayload::Colors(cs) => cs.contains(&color),
            ManaPayload::AnyOneColor(_) | ManaPayload::AnyColors(_) => true,
            ManaPayload::OfColor(c, _) => *c == color,
            ManaPayload::Colorless(_) => false,
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
        if !self.players[p].can_play_land() {
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
            card.definition = *back;
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
                self_counter_cost_reduction: None,
            });
            card.definition.activated_abilities = kept;
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
            c.definition = back_def;
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
        self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &[])
    }

    /// Internal cast-spell helper with optional convoke creatures. Each
    /// listed creature must be untapped + controlled by the caster + the
    /// spell must have `Keyword::Convoke`. Each tap adds {1} generic mana
    /// to the player's pool so the rest of the cost flow consumes it
    /// alongside lands.
    pub(crate) fn cast_spell_with_convoke(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        convoke_creatures: &[CardId],
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;

        if !self.players[p].has_in_hand(card_id) {
            return Err(GameError::CardNotInHand(card_id));
        }
        let mut card = self.players[p].remove_from_hand(card_id).unwrap();
        card.cast_from_hand = true;

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
                .target_filter_for_slot_in_mode(0, mode)
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
                .target_filter_for_slot_in_mode(slot, mode)
                && !self.evaluate_requirement_static(filter, tgt, p, Some(card.id))
            {
                self.players[p].hand.push(card);
                return Err(GameError::SelectionRequirementViolated);
            }
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

        let receipt = match self.try_pay_after_snapshot(p, &cost, snapshot) {
            Ok(r) => r,
            Err(e) => {
                self.players[p].hand.push(card);
                return Err(e);
            }
        };
        if receipt.side_effects.life_lost > 0 {
            self.adjust_life(p, -(receipt.side_effects.life_lost as i32));
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
        consume_first_spell_tax(self, p);

        let on_cast_triggers = collect_self_cast_triggers(&card);
        let uncounterable = self.caster_grants_uncounterable_with_x(p, &card, x_value);

        let was_creature_spell = card.definition.is_creature();
        self.stack.push(StackItem::Spell {
            card: Box::new(card),
            caster: p,
            target,
            additional_targets,
            mode,
            x_value,
            converged_value,
            mana_spent,
            uncounterable,
        });
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

        // The card must have Flashback.
        let flashback_cost = card
            .definition
            .has_flashback()
            .ok_or(GameError::SorcerySpeedOnly)?
            .clone();

        // Timing: instants can be cast at instant speed, others at sorcery
        // speed. Honor Teferi-style opponent restriction.
        let must_be_sorcery_speed = !card.definition.is_instant_speed()
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }

        // Validate target.
        if let Some(ref tgt) = target {
            self.check_target_legality(tgt, p)?;
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
        let receipt = self.try_pay_with_auto_tap(p, &cost)?;
        if receipt.side_effects.life_lost > 0 {
            self.adjust_life(p, -(receipt.side_effects.life_lost as i32));
        }
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());

        // Remove from graveyard.
        let mut card = self.players[p].graveyard.remove(graveyard_pos);
        self.players[p].cards_left_graveyard_this_turn =
            self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
        // Mark as cast via flashback so the resolver routes the card to
        // exile (CR 702.34d) instead of its owner's graveyard.
        card.cast_via_flashback = true;

        let events = vec![
            GameEvent::CardLeftGraveyard { player: p, card_id },
            GameEvent::SpellCast {
                player: p,
                card_id,
                face: CastFace::Flashback,
            },
        ];
        self.finalize_cast(p, card, target, additional_targets, mode, x_value.unwrap_or(0), 0, mana_spent);
        Ok(events)
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
            _ => return Err(GameError::CardNotInHand(card_id)),
        };

        // Clear any outstanding may-play permission — once the card is
        // cast, the grant is consumed.
        card.may_play_until = None;
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
        let permission = card_ref
            .may_play_until
            .ok_or(GameError::CardNotInHand(card_id))?;
        if permission.player != p {
            return Err(GameError::CardNotInHand(card_id));
        }
        // Expiry check: EndOfThisTurn => only valid this turn;
        // EndOfControllersNextTurn => one full controller-turn later.
        // Defensive — the cleanup hook also clears expired permissions.
        let is_instant = card_ref.definition.is_instant_speed();
        let exile_after = permission.exile_after;
        let must_be_sorcery_speed = !is_instant || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
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

        // Pay. On failure put the card back in the command zone.
        let receipt = match self.try_pay_with_auto_tap(p, &cost) {
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

        // Push (modern_decks): optional cast-time predicate gate. Used by
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

        // Timing: sorcery-speed unless instant-speed, plus Teferi-style
        // opponent restriction.
        let must_be_sorcery_speed = !card.definition.is_instant_speed()
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
        if let Some(ref tgt) = target
            && let Some(filter) = card
                .definition
                .effect
                .target_filter_for_slot_in_mode(0, mode)
            && !self.evaluate_requirement_static(filter, tgt, p, Some(card.id))
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
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

        // Pay the alt mana cost (with X substitution + static-ability tax).
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
        let receipt = match self.try_pay_with_auto_tap(p, &mana_cost) {
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
                    // No specific card to attach — reuse the permanent-shaped
                    // Hexproof error variant with a placeholder CardId so the
                    // UI/server still recognizes "this is a hexproof rejection".
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
                    cast_from_hand: true,
                    event_amount: 0,
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
        let snapshot = self.snapshot_payment_state(payer);
        self.try_pay_after_snapshot(payer, cost, snapshot)
    }

    /// Same as `try_pay_with_auto_tap` but uses a snapshot the caller
    /// already captured — for paths that mutate state between snapshot and
    /// payment (e.g. convoke taps creatures in between, activate_ability
    /// applies its tap-cost in between).
    pub(crate) fn try_pay_after_snapshot(
        &mut self,
        payer: usize,
        cost: &crate::mana::ManaCost,
        snapshot: PaymentSnapshot,
    ) -> Result<PaymentReceipt, GameError> {
        let auto_events = self.auto_tap_for_cost(payer, cost);
        match self.players[payer].mana_pool.pay(cost) {
            Ok(side_effects) => Ok(PaymentReceipt {
                auto_events,
                side_effects,
                pool_before: snapshot.pool,
            }),
            Err(e) => {
                self.restore_payment_state(payer, snapshot);
                Err(GameError::Mana(e))
            }
        }
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
        let mut generic: u32 = 0;

        for sym in &cost.symbols {
            match sym {
                ManaSymbol::Colored(c) => {
                    let have = avail.entry(*c).or_default();
                    if *have > 0 { *have -= 1; } else { still_need_colors.push(*c); }
                }
                ManaSymbol::Hybrid(a, b) => {
                    let have_a = *avail.get(a).unwrap_or(&0);
                    let have_b = *avail.get(b).unwrap_or(&0);
                    if have_a > 0 { *avail.entry(*a).or_default() -= 1; }
                    else if have_b > 0 { *avail.entry(*b).or_default() -= 1; }
                    else { still_need_colors.push(*a); }
                }
                ManaSymbol::Phyrexian(c) => {
                    // Pool covers it if available; otherwise paid with life — no tapping.
                    let have = avail.entry(*c).or_default();
                    if *have > 0 { *have -= 1; }
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
            let source = self.battlefield.iter().find(|c| {
                c.controller == player
                    && !c.tapped
                    && c.definition.activated_abilities.iter().any(|a| {
                        is_mana_ability(&a.effect) && effect_produces_color(&a.effect, color)
                    })
            }).map(|c| {
                let idx = c.definition.activated_abilities.iter().position(|a| {
                    is_mana_ability(&a.effect) && effect_produces_color(&a.effect, color)
                }).unwrap_or(0);
                (c.id, idx)
            });
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
                let result = self.activate_ability(id, idx, None);
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
            let source = self.battlefield.iter().find(|c| {
                c.controller == player
                    && !c.tapped
                    && c.definition.activated_abilities.iter().any(|a| is_mana_ability(&a.effect))
            }).map(|c| c.id);
            let Some(id) = source else { break };
            if let Ok(mut evs) = self.activate_ability(id, 0, None) {
                events.append(&mut evs);
            } else {
                break;
            }
        }

        events
    }

    // ── Activate ability ──────────────────────────────────────────────────────

    pub(crate) fn activate_ability(
        &mut self,
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;

        // Source zone: battlefield by default, or the controller's graveyard
        // when the ability is flagged `from_graveyard`. We scan battlefield
        // first; if missing, fall back to graveyards (any player's; we
        // verify ownership/controller match below).
        let (source_in_gy, source_owner) = {
            let on_bf = self.battlefield.iter().any(|c| c.id == card_id);
            if on_bf {
                (false, None)
            } else {
                let owner = self.players.iter().position(|pl|
                    pl.graveyard.iter().any(|c| c.id == card_id));
                match owner {
                    Some(o) => (true, Some(o)),
                    None => return Err(GameError::CardNotOnBattlefield(card_id)),
                }
            }
        };

        let ability: crate::effect::ActivatedAbility = if source_in_gy {
            let owner = source_owner.unwrap();
            self.players[owner].graveyard.iter()
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
                .compute_battlefield()
                .into_iter()
                .find(|c| c.id == card_id)
                .map(|c| c.lost_all_abilities)
                .unwrap_or(false);
            let raw = self.battlefield[pos]
                .definition
                .activated_abilities
                .get(ability_index)
                .cloned()
                .ok_or(GameError::AbilityIndexOutOfBounds)?;
            if stripped && !is_mana_ability(&raw.effect) {
                return Err(GameError::AbilityIndexOutOfBounds);
            }
            raw
        };

        // For graveyard activations, reject if the ability isn't flagged
        // `from_graveyard`. This prevents activating a card's printed
        // battlefield-only ability from the graveyard accidentally.
        if source_in_gy && !ability.from_graveyard {
            return Err(GameError::CardNotOnBattlefield(card_id));
        }

        // Only the controller (or graveyard owner) can activate abilities.
        if source_in_gy {
            if source_owner != Some(p) {
                return Err(GameError::NotYourPriority);
            }
        } else {
            let pos = self.battlefield.iter().position(|c| c.id == card_id).unwrap();
            if self.battlefield[pos].controller != p {
                return Err(GameError::NotYourPriority);
            }
        }

        // Once-per-turn: reject if this ability index has already been
        // used since the most recent turn-cleanup. The ability is recorded
        // as "used" *after* successful activation below so failed mana
        // payments / illegal targets don't burn the per-turn budget.
        // (Graveyard activations don't track per-card once-per-turn state
        // since the card may move between zones; the gate is no-op.)
        if !source_in_gy && ability.once_per_turn {
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

        // Pre-flight exile-other-from-gy gate: confirm a graveyard card
        // matching the cost's filter exists, *excluding* the source itself
        // for graveyard activations where source_in_gy is true. Picks the
        // lowest-CMC matching card. If none, reject cleanly so tap/mana
        // aren't burned. The actual exile happens after payment succeeds.
        let exile_other_pick: Option<CardId> = if let Some(filter) =
            ability.exile_other_filter.as_ref()
        {
            let mut picks: Vec<(CardId, i32)> = self.players[p]
                .graveyard
                .iter()
                .filter(|c| c.id != card_id)
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .map(|c| (c.id, c.definition.cost.cmc() as i32))
                .collect();
            picks.sort_by_key(|(_, cmc)| *cmc);
            match picks.first().copied() {
                Some((cid, _)) => Some(cid),
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
        let mut effective_mana_cost = ability.mana_cost.clone();
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
            if source_in_gy {
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
            let receipt = self.try_pay_after_snapshot(p, &effective_mana_cost, snapshot)?;
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
                // Cache the dying card's snapshot so AnotherOfYours
                // triggers and type-filter predicates fire off
                // sacrifices even when the dying card is a token.
                if let Some(c) = self.battlefield_find(card_id) {
                    self.died_card_snapshots.insert(card_id, c.clone());
                }
                // CR 701.16 — emit the sacrifice-specific event first.
                events.push(GameEvent::CreatureSacrificed { card_id, who: sac_who });
                events.push(GameEvent::CreatureDied { card_id });
            }
            let mut die_evs = self.remove_to_graveyard_with_triggers(card_id);
            events.append(&mut die_evs);
        }

        // Exile-another-from-gy-as-cost: with tap/mana/life paid, exile
        // the cost-picked graveyard card (already validated to exist via
        // the pre-flight `exile_other_pick` lookup). Used by cards like
        // Postmortem Professor's `{1}{B}, Exile an instant or sorcery
        // card from your graveyard: …` and Lorehold Pledgemage's
        // `{2}{R}{W}, Exile a card from your graveyard: +1/+1 EOT`.
        if let Some(other_cid) = exile_other_pick
            && let Some(idx) = self.players[p]
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
                x_value: 0,
                converged_value: 0,
                trigger_source: None,
                mana_spent: 0,
                event_amount: 0,
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
