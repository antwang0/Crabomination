use super::*;
use crate::card::Keyword;
use crate::effect::Effect;

/// Returns true if the given effect is purely a mana ability — only adds
/// mana and uses no targets. Mana abilities resolve immediately without the stack.
fn is_mana_ability(effect: &Effect) -> bool {
    match effect {
        Effect::AddMana { .. } => true,
        Effect::Seq(steps) => !steps.is_empty() && steps.iter().all(is_mana_ability),
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
        Ok(vec![
            GameEvent::LandPlayed { player: p, card_id },
            GameEvent::PermanentEntered { card_id },
        ])
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
        match self.players[p].mana_pool.pay(&cost) {
            Err(e) => {
                self.players[p].hand.push(card);
                return Err(GameError::Mana(e));
            }
            Ok(side_effects) => {
                if side_effects.life_lost > 0 {
                    self.players[p].life -= side_effects.life_lost as i32;
                }
            }
        }

        let events = vec![GameEvent::SpellCast { player: p, card_id }];

        // Track spells cast this turn (for Storm, etc.).
        self.spells_cast_this_turn += 1;

        // Push onto the stack — spell waits there until all players pass priority.
        self.stack.push(StackItem::Spell {
            card: Box::new(card),
            caster: p,
            target,
            mode,
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
        match self.players[p].mana_pool.pay(&cost) {
            Err(e) => return Err(GameError::Mana(e)),
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

        // Pay mana cost
        if !ability.mana_cost.symbols.is_empty() {
            let side_effects = self.players[p]
                .mana_pool
                .pay(&ability.mana_cost)
                .map_err(GameError::Mana)?;
            if side_effects.life_lost > 0 {
                self.players[p].life -= side_effects.life_lost as i32;
            }
        }

        let mut events = vec![GameEvent::AbilityActivated { source: card_id }];

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
