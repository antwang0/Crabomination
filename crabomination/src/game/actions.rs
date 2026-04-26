use super::*;
use crate::card::Keyword;
use crate::effect::{Effect, EventKind, EventScope, ManaPayload};
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

fn effect_produces_color(effect: &Effect, color: ManaColor) -> bool {
    match effect {
        Effect::AddMana { pool, .. } => match pool {
            ManaPayload::Colors(cs) => cs.contains(&color),
            ManaPayload::AnyOneColor(_) | ManaPayload::AnyColors(_) => true,
            ManaPayload::Colorless(_) => false,
        },
        Effect::Seq(steps) => steps.iter().any(|s| effect_produces_color(s, color)),
        _ => false,
    }
}

impl GameState {
    // ── Play land ─────────────────────────────────────────────────────────────

    pub(crate) fn play_land(&mut self, card_id: CardId) -> Result<Vec<GameEvent>, GameError> {
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
        let card = self.players[p].remove_from_hand(card_id).unwrap(); // we just checked has_in_hand
        if !card.definition.is_land() {
            // Put it back then error
            self.players[p].hand.push(card);
            return Err(GameError::NotALand(card_id));
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
        for effect in etb_triggers {
            let auto_target = self.auto_target_for_effect(&effect, controller);
            self.stack.push(StackItem::Trigger {
                source: card_id,
                controller,
                effect: Box::new(effect),
                target: auto_target,
                mode: None,
            });
        }
    }

    // ── Cast spell ────────────────────────────────────────────────────────────

    pub(crate) fn cast_spell(
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
        let card = self.players[p].remove_from_hand(card_id).unwrap();

        // Timing: sorcery-speed requires empty stack + main phase + active player priority.
        // Instant-speed (Instant type or Flash) may be cast whenever you have priority.
        if !card.definition.is_instant_speed() && !self.can_cast_sorcery_speed(p) {
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
        if let Some(ref tgt) = target
            && let Some(filter) = card.definition.effect.target_filter_for_slot(0)
            && !self.evaluate_requirement_static(filter, tgt, p)
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // Pay the cost (substitute X if present)
        let base_cost = card.definition.cost.clone();
        let cost = if base_cost.has_x() {
            base_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            base_cost
        };

        // Snapshot pool and tapped states before auto-tap so we can roll back
        // cleanly if the cost still can't be paid after tapping (e.g. not enough
        // of the right colours).
        let pool_before = self.players[p].mana_pool.clone();
        let tapped_before: Vec<(crate::card::CardId, bool)> = self.battlefield
            .iter()
            .filter(|c| c.owner == p)
            .map(|c| (c.id, c.tapped))
            .collect();

        let mut auto_events = self.auto_tap_for_cost(p, &cost);

        match self.players[p].mana_pool.pay(&cost) {
            Err(e) => {
                // Rollback: restore pool and untap any lands that auto-tap tapped.
                self.players[p].mana_pool = pool_before;
                for (id, was_tapped) in tapped_before {
                    if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == id) {
                        c.tapped = was_tapped;
                    }
                }
                self.players[p].hand.push(card);
                return Err(GameError::Mana(e));
            }
            Ok(side_effects) => {
                if side_effects.life_lost > 0 {
                    self.players[p].life -= side_effects.life_lost as i32;
                }
            }
        }

        auto_events.push(GameEvent::SpellCast { player: p, card_id });
        let events = auto_events;

        // Track spells cast this turn (for Storm, etc.).
        self.spells_cast_this_turn += 1;

        // Push onto the stack — spell waits there until all players pass priority.
        self.stack.push(StackItem::Spell {
            card: Box::new(card),
            caster: p,
            target,
            mode,
            uncounterable: false,
        });

        // Reset priority to active player so all players get a chance to respond.
        self.give_priority_to_active();

        Ok(events)
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

        // Timing: instants can be cast at instant speed, others at sorcery speed.
        if !card.definition.is_instant_speed() && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }

        // Validate target.
        if let Some(ref tgt) = target {
            self.check_target_legality(tgt, p)?;
        }

        // Pay the flashback cost.
        let cost = if flashback_cost.has_x() {
            flashback_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            flashback_cost
        };
        let pool_before = self.players[p].mana_pool.clone();
        let tapped_before: Vec<(crate::card::CardId, bool)> = self.battlefield
            .iter().filter(|c| c.owner == p).map(|c| (c.id, c.tapped)).collect();
        self.auto_tap_for_cost(p, &cost);
        match self.players[p].mana_pool.pay(&cost) {
            Err(e) => {
                self.players[p].mana_pool = pool_before;
                for (id, was_tapped) in tapped_before {
                    if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == id) {
                        c.tapped = was_tapped;
                    }
                }
                return Err(GameError::Mana(e));
            }
            Ok(side_effects) => {
                if side_effects.life_lost > 0 {
                    self.players[p].life -= side_effects.life_lost as i32;
                }
            }
        }

        // Remove from graveyard.
        let mut card = self.players[p].graveyard.remove(graveyard_pos);
        // Mark as cast via flashback so it goes to exile on resolution.
        card.kicked = true; // reuse kicked flag to signal flashback exile

        let events = vec![GameEvent::SpellCast { player: p, card_id }];
        self.spells_cast_this_turn += 1;

        self.stack.push(StackItem::Spell {
            card: Box::new(card),
            caster: p,
            target,
            mode,
            uncounterable: false,
        });
        self.give_priority_to_active();

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
        if alt.evoke_sacrifice {
            card.evoked = true;
        }

        // Timing: sorcery-speed unless instant-speed.
        if !card.definition.is_instant_speed() && !self.can_cast_sorcery_speed(p) {
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
            && let Some(filter) = card.definition.effect.target_filter_for_slot(0)
            && !self.evaluate_requirement_static(filter, tgt, p)
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // Pay the alt mana cost (with X substitution).
        let mana_cost = if alt.mana_cost.has_x() {
            alt.mana_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            alt.mana_cost.clone()
        };
        let pool_before = self.players[p].mana_pool.clone();
        let tapped_before: Vec<(CardId, bool)> = self
            .battlefield
            .iter()
            .filter(|c| c.owner == p)
            .map(|c| (c.id, c.tapped))
            .collect();
        let mut auto_events = self.auto_tap_for_cost(p, &mana_cost);
        match self.players[p].mana_pool.pay(&mana_cost) {
            Err(e) => {
                self.players[p].mana_pool = pool_before;
                for (id, was_tapped) in tapped_before {
                    if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == id) {
                        c.tapped = was_tapped;
                    }
                }
                self.players[p].hand.push(card);
                return Err(GameError::Mana(e));
            }
            Ok(side_effects) => {
                if side_effects.life_lost > 0 {
                    self.players[p].life -= side_effects.life_lost as i32;
                }
            }
        }

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

        auto_events.push(GameEvent::SpellCast { player: p, card_id });
        let events = auto_events;
        self.spells_cast_this_turn += 1;

        self.stack.push(StackItem::Spell {
            card: Box::new(card),
            caster: p,
            target,
            mode,
            uncounterable: false,
        });
        self.give_priority_to_active();

        Ok(events)
    }

    /// Validate that a target is legally targetable by the given controller.
    ///
    /// Returns an error if the target has Hexproof (opponent) or Shroud (anyone),
    /// or has Protection from the caster's color identity.
    fn check_target_legality(&self, target: &Target, caster: usize) -> Result<(), GameError> {
        let Target::Permanent(cid) = target else {
            return Ok(()); // Player targets have no hexproof/shroud
        };
        let Some(card) = self.battlefield_find(*cid) else {
            return Ok(());
        };
        if card.has_keyword(&Keyword::Shroud) {
            return Err(GameError::TargetHasShroud(*cid));
        }
        // Hexproof only prevents targeting by opponents.
        if card.has_keyword(&Keyword::Hexproof) && card.controller != caster {
            return Err(GameError::TargetHasHexproof(*cid));
        }
        Ok(())
    }

    /// Push `SpellCast` triggered abilities (e.g. Prowess) onto the stack.
    /// They will resolve when priority is passed through.
    pub(crate) fn fire_spell_cast_triggers(&mut self, controller: usize, _is_noncreature: bool) {
        use crate::effect::{EventKind, EventScope};
        let triggers: Vec<(CardId, Effect)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == controller)
            .flat_map(|c| {
                c.definition
                    .triggered_abilities
                    .iter()
                    .filter(|t| {
                        t.event.kind == EventKind::SpellCast
                            && matches!(
                                t.event.scope,
                                EventScope::YourControl | EventScope::AnyPlayer
                            )
                    })
                    .map(|t| (c.id, t.effect.clone()))
            })
            .collect();

        for (source, effect) in triggers {
            let auto_target = self.auto_target_for_effect(&effect, controller);
            self.stack.push(StackItem::Trigger {
                source,
                controller,
                effect: Box::new(effect),
                target: auto_target,
                mode: None,
            });
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
            let source = self.battlefield.iter().find(|c| {
                c.owner == player
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
            let source = self.battlefield.iter().find(|c| {
                c.owner == player
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

        // Pay tap cost
        if ability.tap_cost {
            if self.battlefield[pos].tapped {
                return Err(GameError::CardIsTapped(card_id));
            }
            self.battlefield[pos].tapped = true;
        }

        // Pay mana cost (auto-tap if needed)
        let mut auto_mana_events = Vec::new();
        if !ability.mana_cost.symbols.is_empty() {
            let pool_before = self.players[p].mana_pool.clone();
            let tapped_before: Vec<(CardId, bool)> = self.battlefield
                .iter().filter(|c| c.owner == p).map(|c| (c.id, c.tapped)).collect();
            auto_mana_events = self.auto_tap_for_cost(p, &ability.mana_cost);
            match self.players[p].mana_pool.pay(&ability.mana_cost) {
                Ok(side_effects) => {
                    if side_effects.life_lost > 0 {
                        self.players[p].life -= side_effects.life_lost as i32;
                    }
                }
                Err(e) => {
                    self.players[p].mana_pool = pool_before;
                    for (id, was_tapped) in tapped_before {
                        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == id) {
                            c.tapped = was_tapped;
                        }
                    }
                    // Also undo the tap cost on the ability source itself.
                    if ability.tap_cost
                        && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id)
                    {
                        c.tapped = false;
                    }
                    return Err(GameError::Mana(e));
                }
            }
        }

        let mut events = auto_mana_events;
        events.push(GameEvent::AbilityActivated { source: card_id });

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
            });
            self.give_priority_to_active();
        }

        Ok(events)
    }
}
