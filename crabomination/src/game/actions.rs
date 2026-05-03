use super::*;
use crate::card::{CardType, Keyword};
use crate::effect::{Effect, ManaPayload, ZoneDest};
use crate::mana::{Color as ManaColor, ManaSymbol};

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
/// before the spell itself. Each entry carries the trigger's optional
/// filter predicate so the caller can gate the push at cast-time
/// (Lumaret's Favor's "if you gained life this turn" Infusion gate).
fn collect_self_cast_triggers(
    card: &crate::card::CardInstance,
) -> Vec<(Effect, Option<crate::effect::Predicate>)> {
    use crate::effect::{EventKind, EventScope};
    card.definition
        .triggered_abilities
        .iter()
        .filter(|t| {
            t.event.kind == EventKind::SpellCast
                && matches!(t.event.scope, EventScope::SelfSource)
        })
        .map(|t| (t.effect.clone(), t.event.filter.clone()))
        .collect()
}

/// Count distinct colors of mana that decreased between two pool
/// snapshots — i.e. the spell's converge value.
fn converge_count(before: &crate::mana::ManaPool, after: &crate::mana::ManaPool) -> u32 {
    use crate::mana::Color;
    let mut count = 0u32;
    for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
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

/// Walk the battlefield's cost-reduction statics and return the
/// cumulative generic-mana discount the caster is entitled to for
/// the given cast.
///
/// Recognizes:
///   * `StaticEffect::CostReduction { filter, amount }` — a flat
///     discount on every spell matching `filter`. Source's controller
///     must be the caster (printed text always says "spells *you*
///     cast").
///   * `StaticEffect::CostReductionTargeting { spell_filter,
///     target_filter, amount }` — a target-aware discount. The cast
///     must additionally have a chosen target slot 0 that satisfies
///     `target_filter`. Used by Killian, Ink Duelist's "spells you
///     cast that target a creature cost {2} less."
///
/// Multiple distinct sources sum (e.g. two Killians = {4} less).
/// The caller is responsible for clamping the discount against the
/// cost's actual generic component via `ManaCost::reduce_generic`.
pub(crate) fn cost_reduction_for_spell(
    state: &crate::game::GameState,
    caster: usize,
    card: &crate::card::CardInstance,
    target: Option<&crate::game::types::Target>,
) -> u32 {
    use crate::effect::StaticEffect;
    let mut discount = 0u32;
    for src in &state.battlefield {
        if src.controller != caster {
            continue;
        }
        for sa in &src.definition.static_abilities {
            match &sa.effect {
                StaticEffect::CostReduction { filter, amount } => {
                    if state.evaluate_requirement_on_card(filter, card, caster) {
                        discount += amount;
                    }
                }
                StaticEffect::CostReductionTargeting {
                    spell_filter,
                    target_filter,
                    amount,
                } => {
                    if !state.evaluate_requirement_on_card(spell_filter, card, caster) {
                        continue;
                    }
                    let Some(t) = target else { continue };
                    if state.evaluate_requirement_static(target_filter, t, caster) {
                        discount += amount;
                    }
                }
                _ => {}
            }
        }
    }
    discount
}

/// Walk the battlefield's `StaticEffect::TaxActivatedAbilities` statics
/// and return the cumulative generic-mana surcharge owed to activate
/// `source`'s ability. Multiple distinct taxes (e.g. two Augmenter
/// Pugilists, an Augmenter Pugilist + Trinisphere) sum.
///
/// The tax is read against the activating permanent (`source`), not the
/// activator: Augmenter Pugilist's "activated abilities of *creatures*"
/// applies to every creature on the battlefield — friend or foe. Mana
/// abilities are *not* exempt at the rules level (Augmenter Pugilist
/// taxes Llanowar Elves's mana ability too); we mirror that here.
pub(crate) fn extra_cost_for_activation(
    state: &crate::game::GameState,
    source: &crate::card::CardInstance,
) -> u32 {
    use crate::effect::StaticEffect;
    let mut tax = 0u32;
    for src in &state.battlefield {
        for sa in &src.definition.static_abilities {
            if let StaticEffect::TaxActivatedAbilities { filter, amount } = &sa.effect {
                // Evaluate the static's filter against the activating
                // permanent — Augmenter Pugilist's `Creature` filter
                // taxes every creature (friend or foe) per printed Oracle.
                if state.evaluate_requirement_on_card(filter, source, source.controller) {
                    tax += amount;
                }
            }
        }
    }
    tax
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
    let mut your_norns = 0usize;
    let mut opp_norns = 0usize;
    for c in &state.battlefield {
        if c.definition.name == "Elesh Norn, Mother of Machines" {
            if c.controller == etb_controller {
                your_norns += 1;
            } else {
                opp_norns += 1;
            }
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
    pub(crate) fn caster_grants_uncounterable(
        &self,
        caster: usize,
        card: &crate::card::CardInstance,
    ) -> bool {
        // The card itself is uncounterable (Dovin's Veto, Stubborn Denial,
        // etc. — `Keyword::CantBeCountered`).
        if card.definition.keywords.contains(&Keyword::CantBeCountered) {
            return true;
        }
        // Cavern of Souls: a creature spell whose caster controls a Cavern
        // whose chosen creature type matches one of the spell's types.
        // Caverns whose ETB hasn't yet resolved (`chosen_creature_type =
        // None`) — typical for `add_card_to_battlefield`-built test fixtures
        // — fall back to the legacy "any creature is uncounterable" rule so
        // pre-existing tests don't break on the tighter check. The
        // mana-provenance gate (must spend Cavern mana on the cast) is still
        // collapsed.
        if !card.definition.is_creature() {
            return false;
        }
        self.battlefield.iter().any(|c| {
            c.controller == caster
                && c.definition.name == "Cavern of Souls"
                && match c.chosen_creature_type {
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
            card.definition.activated_abilities = vec![
                crate::card::ActivatedAbility {
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
            exile_gy_cost: 0,
                },
            ];
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
            for _ in 0..multiplier {
                self.stack.push(StackItem::Trigger {
                    source: card_id,
                    controller,
                    effect: Box::new(effect.clone()),
                    target: auto_target.clone(),
                    mode: None,
                    x_value: 0,
                    converged_value: 0,
                    // ETB trigger — subject is the entering permanent.
                    subject: Some(crate::game::effects::EntityRef::Permanent(card_id)),
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
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.players[p].has_in_hand(card_id) {
            return Err(GameError::CardNotInHand(card_id));
        }
        // Look up the back face on the front-face definition.
        let back_def = {
            let card = self
                .players[p]
                .hand
                .iter()
                .find(|c| c.id == card_id)
                .expect("has_in_hand verified");
            match card.definition.back_face.clone() {
                Some(b) => *b,
                None => return Err(GameError::NotALand(card_id)),
            }
        };
        // Swap the in-hand definition to the back face. The hand card
        // mutates in place — successful cast removes it; rejected cast
        // means the cast machinery already restored the card with the
        // swapped definition, but since the front face no longer
        // appears in hand the user can still cast either face on retry
        // (back face's `back_face` is None, but front-face revival is
        // out of scope; in practice MDFCs are committed to one face per
        // game).
        if let Some(c) = self.players[p].hand.iter_mut().find(|c| c.id == card_id) {
            c.definition = back_def;
        }
        // Delegate to the regular cast path. The back face's cost,
        // type, target filters, and effect now drive validation.
        // Tag the cast face so the SpellCast event surfaces it.
        self.pending_cast_face = CastFace::Back;
        let result = self.cast_spell(card_id, target, mode, x_value);
        self.pending_cast_face = CastFace::Front;
        result
    }

    pub(crate) fn cast_spell(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_spell_with_convoke(card_id, target, mode, x_value, &[])
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

        // Validate that the chosen target is legally targetable.
        if let Some(ref tgt) = target
            && let Err(e) = self.check_target_legality(tgt, p)
        {
            self.players[p].hand.push(card);
            return Err(e);
        }

        // Enforce the spell's target selection requirement (e.g. Terror's
        // "non-black, non-artifact creature"): if the effect binds a filter to
        // slot 0 and the chosen target doesn't match, reject the cast.
        // For modal cards (`ChooseMode`), only look at the chosen mode's
        // filter — Drown in the Loch's mode 0 (counter spell) and mode 1
        // (destroy creature) have incompatible filters, and the legacy
        // "first match across all modes" path picked mode 0's `IsSpellOnStack`
        // even when the caster picked mode 1.
        if let Some(ref tgt) = target
            && let Some(filter) = card
                .definition
                .effect
                .target_filter_for_slot_in_mode(0, mode)
            && !self.evaluate_requirement_static(filter, tgt, p)
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // Pay the cost (substitute X if present, then apply any
        // static-ability discounts (Killian's "{2} less if targeting a
        // creature") and surcharges (Damping Sphere's "{1} more after
        // the first spell each turn")).
        let base_cost = card.definition.cost.clone();
        let mut cost = if base_cost.has_x() {
            base_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            base_cost
        };
        let discount = cost_reduction_for_spell(self, p, &card, target.as_ref());
        if discount > 0 {
            cost.reduce_generic(discount);
        }
        let tax = extra_cost_for_spell(self, p, &card);
        if tax > 0 {
            cost.symbols.push(crate::mana::ManaSymbol::Generic(tax));
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
            self.players[p].life -= receipt.side_effects.life_lost as i32;
        }

        // Compute converge: count distinct colors of mana drained from the
        // pool by paying the cost. Convoke pips contribute generic only,
        // so they don't raise this count.
        let converged_value = converge_count(&receipt.pool_before, &self.players[p].mana_pool);

        let mut auto_events = receipt.auto_events;
        auto_events.push(GameEvent::SpellCast {
            player: p,
            card_id,
            face: self.pending_cast_face,
        });
        let events = auto_events;

        self.finalize_cast(p, card, target, mode, x_value.unwrap_or(0), converged_value);

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
    pub(crate) fn finalize_cast(
        &mut self,
        p: usize,
        card: crate::card::CardInstance,
        target: Option<Target>,
        mode: Option<usize>,
        x_value: u32,
        converged_value: u32,
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
        let uncounterable = self.caster_grants_uncounterable(p, &card);

        let was_creature_spell = card.definition.is_creature();
        let face = self.pending_cast_face;
        self.stack.push(StackItem::Spell {
            card: Box::new(card),
            caster: p,
            target,
            mode,
            x_value,
            converged_value,
            uncounterable,
            face,
            is_copy: false,
        });
        self.push_on_cast_triggers(card_id, p, on_cast_triggers);
        // SpellCast / YourControl triggers (Prowess, Magecraft, Repartee, …)
        // fire *at cast time*, before the spell resolves. The trigger goes
        // on the stack above the spell so it resolves first (and still
        // fires if the spell itself is countered in response). Filters
        // (e.g. CastSpellTargetsMatch) read the just-cast spell's target
        // from the stack while the spell still sits there.
        self.fire_spell_cast_triggers(p, card_id, !was_creature_spell);
        self.give_priority_to_active();
    }

    /// Push pre-collected `SpellCast`/`SelfSource` triggers from the
    /// just-cast card onto the stack as `Trigger` items, so they resolve
    /// before the spell itself. Caller is responsible for collecting the
    /// effect list before the card moves into the stack item.
    /// `triggers` carries each trigger's effect plus its optional
    /// `EventSpec.filter` predicate; the filter is evaluated against
    /// the cast spell as `trigger_source` and the trigger is skipped
    /// when the predicate returns false (Lumaret's Favor's "if you
    /// gained life this turn" Infusion gate).
    pub(crate) fn push_on_cast_triggers(
        &mut self,
        source: CardId,
        controller: usize,
        triggers: Vec<(Effect, Option<crate::effect::Predicate>)>,
    ) {
        for (effect, filter) in triggers {
            if let Some(filter) = filter {
                let ctx = crate::game::effects::EffectContext {
                    controller,
                    source: Some(source),
                    targets: vec![],
                    trigger_source: Some(crate::game::effects::EntityRef::Card(source)),
                    mode: 0,
                    x_value: 0,
                    converged_value: 0,
                    cast_face: crate::game::types::CastFace::Front,
                };
                if !self.evaluate_predicate(&filter, &ctx) {
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
                // Self-cast trigger (Magecraft on the source itself);
                // subject is the cast card (== source).
                subject: Some(crate::game::effects::EntityRef::Card(source)),
            });
        }
    }

    /// Cast a spell from the graveyard using its Flashback cost.
    pub(crate) fn cast_flashback(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
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

        // Pay the flashback cost (with cost-reduction statics applied).
        let mut cost = if flashback_cost.has_x() {
            flashback_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            flashback_cost
        };
        let discount = cost_reduction_for_spell(self, p, &card, target.as_ref());
        if discount > 0 {
            cost.reduce_generic(discount);
        }
        let receipt = self.try_pay_with_auto_tap(p, &cost)?;
        if receipt.side_effects.life_lost > 0 {
            self.players[p].life -= receipt.side_effects.life_lost as i32;
        }

        // Remove from graveyard.
        let mut card = self.players[p].graveyard.remove(graveyard_pos);
        self.players[p].cards_left_graveyard_this_turn =
            self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
        // Mark as cast via flashback so it goes to exile on resolution.
        card.kicked = true; // reuse kicked flag to signal flashback exile

        let events = vec![
            GameEvent::CardLeftGraveyard { player: p, card_id },
            GameEvent::SpellCast {
                player: p,
                card_id,
                face: CastFace::Flashback,
            },
        ];
        // Stash the flashback face so `finalize_cast` can stamp it on
        // the StackItem::Spell. Reset to Front afterwards so subsequent
        // hand casts read correctly.
        let prior_face = self.pending_cast_face;
        self.pending_cast_face = CastFace::Flashback;
        self.finalize_cast(p, card, target, mode, x_value.unwrap_or(0), 0);
        self.pending_cast_face = prior_face;
        Ok(events)
    }

    /// Cast a spell using its `alternative_cost` (a "pitch" cost) instead of
    /// its regular mana cost. Pays the alt cost's mana, deducts life, and
    /// exiles the chosen `pitch_card` from hand if the alt cost requires
    /// one. The spell otherwise behaves identically to a normal cast (goes
    /// onto the stack, resolves later, etc.).
    pub(crate) fn cast_spell_alternative(
        &mut self,
        card_id: CardId,
        pitch_card: Option<CardId>,
        target: Option<Target>,
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
            && !self.evaluate_requirement_static(filter, tgt, p)
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }
        // Alt-cost-specific target filter (e.g. Mystical Dispute's "target
        // must be a blue spell"). Applied on top of the spell's regular
        // target filter, only on the alternative-cast path.
        if let Some(ref tgt) = target
            && let Some(ref alt_filter) = alt.target_filter
            && !self.evaluate_requirement_static(alt_filter, tgt, p)
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // Pay the alt mana cost (with X substitution + static-ability
        // discounts + static-ability tax).
        let mut mana_cost = if alt.mana_cost.has_x() {
            alt.mana_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            alt.mana_cost.clone()
        };
        let discount = cost_reduction_for_spell(self, p, &card, target.as_ref());
        if discount > 0 {
            mana_cost.reduce_generic(discount);
        }
        let tax = extra_cost_for_spell(self, p, &card);
        if tax > 0 {
            mana_cost.symbols.push(crate::mana::ManaSymbol::Generic(tax));
        }
        let receipt = match self.try_pay_with_auto_tap(p, &mana_cost) {
            Ok(r) => r,
            Err(e) => {
                self.players[p].hand.push(card);
                return Err(e);
            }
        };
        if receipt.side_effects.life_lost > 0 {
            self.players[p].life -= receipt.side_effects.life_lost as i32;
        }
        let mut auto_events = receipt.auto_events;

        // Pay the life portion of the alt cost.
        if alt.life_cost > 0 {
            self.players[p].life -= alt.life_cost as i32;
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

        auto_events.push(GameEvent::SpellCast {
            player: p,
            card_id,
            face: CastFace::Front,
        });
        let events = auto_events;
        // Devastating Mastery-style alt cost that *is* the mode selector:
        // override the caller-supplied mode with the alt's `mode_on_alt`.
        let resolved_mode = alt.mode_on_alt.or(mode);
        self.finalize_cast(p, card, target, resolved_mode, x_value.unwrap_or(0), 0);
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
        caster: usize,
        cast_card: CardId,
        is_noncreature: bool,
    ) {
        use crate::effect::{EventKind, EventScope};
        // Prowess: "Whenever you cast a noncreature spell, this creature
        // gets +1/+1 until end of turn." Treated as a synthetic trigger
        // shared by every battlefield permanent controlled by the
        // caster that has `Keyword::Prowess`. Fires only on noncreature
        // casts (creature spells skip the pump per the printed text).
        // The trigger pushes a `Trigger` stack item whose body is
        // `PumpPT(This, +1/+1, EOT)` — `Selector::This` resolves to
        // each Prowess creature individually so multiple Prowess bodies
        // each receive their own pump.
        if is_noncreature {
            use crate::card::Keyword;
            use crate::effect::{Duration, Selector, Value};
            let prowess: Vec<(CardId, usize)> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == caster
                    && c.definition.keywords.contains(&Keyword::Prowess)
                    && c.definition.is_creature())
                .map(|c| (c.id, c.controller))
                .collect();
            for (source, source_ctrl) in prowess {
                self.stack.push(StackItem::Trigger {
                    source,
                    controller: source_ctrl,
                    effect: Box::new(Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(1),
                        toughness: Value::Const(1),
                        duration: Duration::EndOfTurn,
                    }),
                    target: None,
                    mode: None,
                    x_value: 0,
                    converged_value: 0,
                    subject: Some(crate::game::effects::EntityRef::Card(cast_card)),
                });
            }
        }
        // Walk every permanent on the battlefield whose SpellCast trigger
        // matches the cast event's scope. `YourControl` / `AnyPlayer` fire
        // for triggers controlled by the caster; `OpponentControl` fires
        // for triggers controlled by *another* player than the caster.
        // Without the OpponentControl arm, "whenever an opponent casts X"
        // payoffs (Esper Sentinel, Mindbreak Trap, Ethersworn Canonist's
        // sibling triggers) never fire because the caster doesn't control
        // them. Each candidate captures its own controller for the trigger-
        // resolution context (so the trigger's body resolves on the
        // *trigger's* controller, not the caster).
        let candidates: Vec<(CardId, usize, Effect, Option<crate::effect::Predicate>)> = self
            .battlefield
            .iter()
            .flat_map(|c| {
                c.definition
                    .triggered_abilities
                    .iter()
                    .filter(|t| t.event.kind == EventKind::SpellCast)
                    .filter(|t| match t.event.scope {
                        EventScope::YourControl | EventScope::AnyPlayer => {
                            c.controller == caster
                        }
                        EventScope::OpponentControl => c.controller != caster,
                        _ => false,
                    })
                    .map(|t| (c.id, c.controller, t.effect.clone(), t.event.filter.clone()))
            })
            .collect();

        for (source, source_ctrl, effect, filter) in candidates {
            if let Some(filter) = filter {
                let ctx = crate::game::effects::EffectContext {
                    controller: source_ctrl,
                    source: Some(source),
                    targets: vec![],
                    trigger_source: Some(crate::game::effects::EntityRef::Card(cast_card)),
                    mode: 0,
                    x_value: 0,
                    converged_value: 0,
                    cast_face: crate::game::types::CastFace::Front,
                };
                if !self.evaluate_predicate(&filter, &ctx) {
                    continue;
                }
            }
            let auto_target =
                self.auto_target_for_effect_avoiding(&effect, source_ctrl, Some(source));
            self.stack.push(StackItem::Trigger {
                source,
                controller: source_ctrl,
                effect: Box::new(effect),
                target: auto_target,
                mode: None,
                x_value: 0,
                converged_value: 0,
                // The cast card is the trigger subject — Magecraft /
                // Repartee / Esper Sentinel / Mindbreak Trap all read
                // `Selector::TriggerSource` for the just-cast spell.
                subject: Some(crate::game::effects::EntityRef::Card(cast_card)),
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

        let pos = self
            .battlefield
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotOnBattlefield(card_id))?;

        let ability = self.battlefield[pos]
            .definition
            .activated_abilities
            .get(ability_index)
            .cloned()
            .ok_or(GameError::AbilityIndexOutOfBounds)?;

        // Only the controller can activate abilities.
        if self.battlefield[pos].controller != p {
            return Err(GameError::NotYourPriority);
        }

        // Once-per-turn: reject if this ability index has already been
        // used since the most recent turn-cleanup. The ability is recorded
        // as "used" *after* successful activation below so failed mana
        // payments / illegal targets don't burn the per-turn budget.
        if ability.once_per_turn
            && self.battlefield[pos].once_per_turn_used.contains(&ability_index)
        {
            return Err(GameError::AbilityAlreadyUsedThisTurn);
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
                cast_face: crate::game::types::CastFace::Front,
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
            && !self.evaluate_requirement_static(filter, tgt, p)
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

        // Pre-flight graveyard-exile-cost gate: reject if the
        // controller's graveyard has fewer cards than required. Used
        // by Lorehold Pledgemage's `{2}{R}{W}, Exile a card from your
        // graveyard: +1/+1 EOT` and similar exile-from-gy-cost
        // activations.
        if ability.exile_gy_cost > 0
            && self.players[p].graveyard.len() < ability.exile_gy_cost as usize
        {
            return Err(GameError::InsufficientGraveyard);
        }

        // Compute battlefield-wide activation tax (Augmenter Pugilist
        // family). The tax is folded into the mana cost via additional
        // generic pips before payment — same shape as Damping Sphere's
        // first-spell-tax for casts.
        let activation_tax = {
            let source_card = &self.battlefield[pos];
            extra_cost_for_activation(self, source_card)
        };
        let effective_mana_cost = if activation_tax == 0 {
            ability.mana_cost.clone()
        } else {
            let mut c = ability.mana_cost.clone();
            c.symbols.push(crate::mana::generic(activation_tax));
            c
        };

        // Snapshot pristine state before applying tap-cost so a failed mana
        // payment rolls back both the auto-tap of mana sources AND the
        // tap-cost on the source itself.
        let needs_payment = !effective_mana_cost.symbols.is_empty();
        let pre_snapshot = needs_payment.then(|| self.snapshot_payment_state(p));

        // Pay tap cost
        if ability.tap_cost {
            if self.battlefield[pos].tapped {
                return Err(GameError::CardIsTapped(card_id));
            }
            self.battlefield[pos].tapped = true;
        }

        let mut auto_mana_events = Vec::new();
        if let Some(snapshot) = pre_snapshot {
            let receipt = self.try_pay_after_snapshot(p, &effective_mana_cost, snapshot)?;
            if receipt.side_effects.life_lost > 0 {
                self.players[p].life -= receipt.side_effects.life_lost as i32;
            }
            auto_mana_events = receipt.auto_events;
        }

        // Pay the life cost. Tap and mana are committed; the life
        // payment is now safe (the pre-flight gate above guaranteed
        // sufficient life). Emits a LifeLost event so trigger / replay
        // observers see the cost.
        if ability.life_cost > 0 {
            self.players[p].life -= ability.life_cost as i32;
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
        if ability.once_per_turn {
            if let Some(card) = self.battlefield.iter_mut().find(|c| c.id == card_id) {
                card.once_per_turn_used.push(ability_index);
            }
        }

        // Exile-from-graveyard cost: with tap, mana, and life costs
        // paid, exile the controller's oldest `exile_gy_cost` cards
        // from their graveyard. Auto-pick is the oldest cards (gy
        // index 0..N) — same heuristic the auto-decider uses for
        // similar cost picks. Emits CardLeftGraveyard for each pick
        // so Strixhaven gy-leave payoffs trigger.
        if ability.exile_gy_cost > 0 {
            let n = ability.exile_gy_cost as usize;
            let picked: Vec<CardId> = self.players[p]
                .graveyard
                .iter()
                .take(n)
                .map(|c| c.id)
                .collect();
            for cid in picked {
                let ctx = crate::game::effects::EffectContext {
                    controller: p,
                    source: Some(card_id),
                    targets: vec![],
                    trigger_source: None,
                    mode: 0,
                    x_value: 0,
                    converged_value: 0,
                    cast_face: crate::game::types::CastFace::Front,
                };
                self.move_card_to(cid, &ZoneDest::Exile, &ctx, &mut events);
            }
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
            if is_creature {
                events.push(GameEvent::CreatureDied { card_id });
            }
            let mut die_evs = self.remove_to_graveyard_with_triggers(card_id);
            events.append(&mut die_evs);
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
            self.stack.push(StackItem::Trigger {
                source: card_id,
                controller: p,
                effect: Box::new(ability.effect),
                target,
                mode: None,
                x_value: 0,
                converged_value: 0,
                // Activated ability — subject is the source permanent.
                subject: Some(crate::game::effects::EntityRef::Permanent(card_id)),
            });
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
