use super::*;
use crate::card::{
    ActivatedAbility, CardDefinition, CardId, CardInstance, EffectCondition, Keyword,
    SelectionRequirement, SpellEffect, TokenDefinition, Zone,
};
use crate::decision::{Decision, DecisionAnswer};
use crate::mana::{Color, ManaSymbol, ManaCost};

impl GameState {
    // ── Effect resolution ─────────────────────────────────────────────────────

    /// Resolve a single spell or ability effect.
    ///
    /// `mode` is the chosen option index for `ChooseOne` effects (0 if not applicable).
    pub(crate) fn resolve_effect(
        &mut self,
        effect: &SpellEffect,
        controller: usize,
        target: Option<&Target>,
        mode: usize,
    ) -> Result<Vec<GameEvent>, GameError> {
        let mut events = vec![];
        match effect {
            SpellEffect::DealDamage { amount, target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                match tgt {
                    Target::Player(pidx) => {
                        let amount = *amount;
                        self.players[*pidx].life -= amount as i32;
                        events.push(GameEvent::DamageDealt {
                            amount,
                            to_player: Some(*pidx),
                            to_card: None,
                        });
                        events.push(GameEvent::LifeLost {
                            player: *pidx,
                            amount,
                        });
                        let mut sba = self.check_state_based_actions();
                        events.append(&mut sba);
                    }
                    Target::Permanent(cid) => {
                        let amount = *amount;
                        if let Some(c) = self.battlefield_find_mut(*cid) {
                            c.damage += amount;
                        } else {
                            return Err(GameError::CardNotOnBattlefield(*cid));
                        }
                        events.push(GameEvent::DamageDealt {
                            amount,
                            to_player: None,
                            to_card: Some(*cid),
                        });
                        let mut sba = self.check_state_based_actions();
                        events.append(&mut sba);
                    }
                }
            }

            // ── Destroy ──────────────────────────────────────────────────────

            SpellEffect::DestroyCreature { target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                let Target::Permanent(cid) = tgt else {
                    return Err(GameError::InvalidTarget);
                };
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let card_id = *cid;
                // Indestructible permanents are unaffected by destroy effects.
                if self.battlefield_find(card_id).map(|c| c.has_keyword(&Keyword::Indestructible)).unwrap_or(false) {
                    // No event; spell fizzles on indestructible target.
                } else {
                    events.push(GameEvent::CreatureDied { card_id });
                    let mut die_evs = self.remove_to_graveyard_with_triggers(card_id);
                    events.append(&mut die_evs);
                }
            }

            SpellEffect::DestroyPermanent { target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                let Target::Permanent(cid) = tgt else {
                    return Err(GameError::InvalidTarget);
                };
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let card_id = *cid;
                if !self.battlefield_find(card_id).map(|c| c.has_keyword(&Keyword::Indestructible)).unwrap_or(false) {
                    let is_creature = self.battlefield_find(card_id).map(|c| c.definition.is_creature()).unwrap_or(false);
                    if is_creature { events.push(GameEvent::CreatureDied { card_id }); }
                    let mut die_evs = self.remove_to_graveyard_with_triggers(card_id);
                    events.append(&mut die_evs);
                }
            }

            SpellEffect::DestroyAll { target: req } => {
                let ids: Vec<CardId> = self.battlefield.iter()
                    .filter(|c| {
                        let tgt = Target::Permanent(c.id);
                        self.evaluate_requirement_static(req, &tgt, controller)
                            && !c.has_keyword(&Keyword::Indestructible)
                    })
                    .map(|c| c.id)
                    .collect();
                for id in ids {
                    let is_creature = self.battlefield_find(id).map(|c| c.definition.is_creature()).unwrap_or(false);
                    if is_creature { events.push(GameEvent::CreatureDied { card_id: id }); }
                    let mut die_evs = self.remove_to_graveyard_with_triggers(id);
                    events.append(&mut die_evs);
                }
            }

            SpellEffect::DestroyAllCreatures => {
                let ids: Vec<CardId> = self.battlefield.iter()
                    .filter(|c| c.definition.is_creature() && !c.has_keyword(&Keyword::Indestructible))
                    .map(|c| c.id)
                    .collect();
                for id in ids {
                    events.push(GameEvent::CreatureDied { card_id: id });
                    let mut die_evs = self.remove_to_graveyard_with_triggers(id);
                    events.append(&mut die_evs);
                }
            }

            // ── Exile ─────────────────────────────────────────────────────────

            SpellEffect::ExilePermanent { target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                let Target::Permanent(cid) = tgt else {
                    return Err(GameError::InvalidTarget);
                };
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let card_id = *cid;
                events.push(GameEvent::PermanentExiled { card_id });
                self.remove_from_battlefield_to_exile(card_id);
            }

            SpellEffect::ExileAll { target: req } => {
                let ids: Vec<CardId> = self.battlefield.iter()
                    .filter(|c| {
                        let tgt = Target::Permanent(c.id);
                        self.evaluate_requirement_static(req, &tgt, controller)
                    })
                    .map(|c| c.id)
                    .collect();
                for id in ids {
                    events.push(GameEvent::PermanentExiled { card_id: id });
                    self.remove_from_battlefield_to_exile(id);
                }
            }

            // ── Draw / discard / mill ──────────────────────────────────────────

            SpellEffect::DrawCards { amount } => {
                for _ in 0..*amount {
                    let drawn = self.players[controller].draw_top();
                    match drawn {
                        Some(id) => {
                            events.push(GameEvent::CardDrawn { player: controller, card_id: id });
                        }
                        None => {
                            let opp = (controller + 1) % self.players.len();
                            self.game_over = Some(Some(opp));
                            events.push(GameEvent::GameOver { winner: Some(opp) });
                            return Ok(events);
                        }
                    }
                }
            }

            SpellEffect::EachPlayerDraws { amount } => {
                for p in 0..self.players.len() {
                    for _ in 0..*amount {
                        if let Some(id) = self.players[p].draw_top() {
                            events.push(GameEvent::CardDrawn { player: p, card_id: id });
                        }
                    }
                }
            }

            SpellEffect::Discard { amount } => {
                let opp = (controller + 1) % self.players.len();
                for _ in 0..*amount {
                    if !self.players[opp].hand.is_empty() {
                        let card = self.players[opp].hand.remove(0);
                        let card_id = card.id;
                        self.players[opp].send_to_graveyard(card);
                        events.push(GameEvent::CardDiscarded { player: opp, card_id });
                    }
                }
            }

            SpellEffect::Mill { amount } => {
                // Default: mill controller's opponent.
                let opp = (controller + 1) % self.players.len();
                for _ in 0..*amount {
                    if self.players[opp].library.is_empty() {
                        let opp_opp = (opp + 1) % self.players.len();
                        self.game_over = Some(Some(opp_opp));
                        events.push(GameEvent::GameOver { winner: Some(opp_opp) });
                        return Ok(events);
                    }
                    let card = self.players[opp].library.remove(0);
                    let card_id = card.id;
                    self.players[opp].send_to_graveyard(card);
                    events.push(GameEvent::CardMilled { player: opp, card_id });
                }
            }

            SpellEffect::Scry { amount } => {
                let n = (*amount as usize).min(self.players[controller].library.len());
                if n > 0 {
                    let peeked: Vec<(CardId, &'static str)> = self.players[controller]
                        .library
                        .iter()
                        .take(n)
                        .map(|c| (c.id, c.definition.name))
                        .collect();
                    // Suspend: the enclosing resolver will install a
                    // `pending_decision` with the resume context. The library
                    // is left untouched; the top N cards stay in place until
                    // `submit_decision` is called with the chosen order.
                    self.suspend_signal = Some((
                        Decision::Scry { player: controller, cards: peeked },
                        PendingEffectState::ScryPeeked { count: n, player: controller },
                    ));
                }
            }

            // ── Mana ──────────────────────────────────────────────────────────

            SpellEffect::AddMana { colors } => {
                for &c in colors {
                    self.players[controller].mana_pool.add(c, 1);
                    events.push(GameEvent::ManaAdded { player: controller, color: c });
                }
            }

            SpellEffect::AddManaAnyColor { amount } => {
                for _ in 0..*amount {
                    let ans = self.decider.decide(&Decision::ChooseColor {
                        source: CardId(0),
                        legal: vec![
                            Color::White, Color::Blue, Color::Black, Color::Red, Color::Green,
                        ],
                    });
                    let DecisionAnswer::Color(c) = ans else {
                        return Err(GameError::InvalidTarget);
                    };
                    self.players[controller].mana_pool.add(c, 1);
                    events.push(GameEvent::ManaAdded { player: controller, color: c });
                }
            }

            SpellEffect::AddColorlessMana { amount } => {
                self.players[controller].mana_pool.add_colorless(*amount);
                // No ManaAdded event color to report; emit one per pip with a placeholder.
                for _ in 0..*amount {
                    events.push(GameEvent::ColorlessManaAdded { player: controller });
                }
            }

            // ── Life ──────────────────────────────────────────────────────────

            SpellEffect::GainLife { amount } => {
                let amount = *amount;
                self.players[controller].life += amount as i32;
                events.push(GameEvent::LifeGained { player: controller, amount });
            }

            SpellEffect::LoseLife { amount } => {
                let amount = *amount;
                // Default: controller loses life (e.g. "you lose 3 life").
                self.players[controller].life -= amount as i32;
                events.push(GameEvent::LifeLost { player: controller, amount });
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
            }

            // ── Pump ──────────────────────────────────────────────────────────

            SpellEffect::PumpCreature { power_bonus, toughness_bonus } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                let Target::Permanent(cid) = tgt else {
                    return Err(GameError::InvalidTarget);
                };
                let c = self
                    .battlefield_find_mut(*cid)
                    .ok_or(GameError::CardNotOnBattlefield(*cid))?;
                c.power_bonus += power_bonus;
                c.toughness_bonus += toughness_bonus;
                events.push(GameEvent::PumpApplied {
                    card_id: *cid,
                    power: *power_bonus,
                    toughness: *toughness_bonus,
                });
            }

            SpellEffect::PumpAllCreatures { power_bonus, toughness_bonus, filter } => {
                let ids: Vec<CardId> = self.battlefield.iter()
                    .filter(|c| {
                        c.definition.is_creature() && {
                            let tgt = Target::Permanent(c.id);
                            self.evaluate_requirement_static(filter, &tgt, controller)
                        }
                    })
                    .map(|c| c.id)
                    .collect();
                for id in ids {
                    if let Some(c) = self.battlefield_find_mut(id) {
                        c.power_bonus += power_bonus;
                        c.toughness_bonus += toughness_bonus;
                        events.push(GameEvent::PumpApplied {
                            card_id: id,
                            power: *power_bonus,
                            toughness: *toughness_bonus,
                        });
                    }
                }
            }

            // ── Bounce ────────────────────────────────────────────────────────

            SpellEffect::ReturnToHand { target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                let Target::Permanent(cid) = tgt else {
                    return Err(GameError::InvalidTarget);
                };
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let card_id = *cid;
                if let Some(pos) = self.battlefield.iter().position(|c| c.id == card_id) {
                    let card = self.battlefield.remove(pos);
                    let owner = card.owner;
                    self.players[owner].hand.push(card);
                }
            }

            SpellEffect::ReturnAllToHand { target: req } => {
                let ids: Vec<CardId> = self.battlefield.iter()
                    .filter(|c| {
                        let tgt = Target::Permanent(c.id);
                        self.evaluate_requirement_static(req, &tgt, controller)
                    })
                    .map(|c| c.id)
                    .collect();
                for id in ids {
                    if let Some(pos) = self.battlefield.iter().position(|c| c.id == id) {
                        let card = self.battlefield.remove(pos);
                        let owner = card.owner;
                        self.players[owner].hand.push(card);
                    }
                }
            }

            // ── Counters ──────────────────────────────────────────────────────

            SpellEffect::AddCounters { count, counter_type, target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                let Target::Permanent(cid) = tgt else {
                    return Err(GameError::InvalidTarget);
                };
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let card_id = *cid;
                let ct = *counter_type;
                let n = *count;
                if let Some(c) = self.battlefield_find_mut(card_id) {
                    c.add_counters(ct, n);
                    events.push(GameEvent::CounterAdded { card_id, counter_type: ct, count: n });
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
            }

            SpellEffect::RemoveCounters { count, counter_type, target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                let Target::Permanent(cid) = tgt else {
                    return Err(GameError::InvalidTarget);
                };
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let card_id = *cid;
                let ct = *counter_type;
                let n = *count;
                if let Some(c) = self.battlefield_find_mut(card_id) {
                    let removed = c.remove_counters(ct, n);
                    if removed > 0 {
                        events.push(GameEvent::CounterRemoved { card_id, counter_type: ct, count: removed });
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
            }

            // ── Tap / untap ────────────────────────────────────────────────────

            SpellEffect::TapPermanent { target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                let Target::Permanent(cid) = tgt else {
                    return Err(GameError::InvalidTarget);
                };
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let card_id = *cid;
                if let Some(c) = self.battlefield_find_mut(card_id) {
                    c.tapped = true;
                    events.push(GameEvent::PermanentTapped { card_id });
                }
            }

            SpellEffect::TapAll { target: req } => {
                let ids: Vec<CardId> = self.battlefield.iter()
                    .filter(|c| {
                        let tgt = Target::Permanent(c.id);
                        self.evaluate_requirement_static(req, &tgt, controller)
                    })
                    .map(|c| c.id)
                    .collect();
                for id in ids {
                    if let Some(c) = self.battlefield_find_mut(id) {
                        c.tapped = true;
                        events.push(GameEvent::PermanentTapped { card_id: id });
                    }
                }
            }

            SpellEffect::UntapPermanent { target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                let Target::Permanent(cid) = tgt else {
                    return Err(GameError::InvalidTarget);
                };
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let card_id = *cid;
                if let Some(c) = self.battlefield_find_mut(card_id) {
                    c.tapped = false;
                    events.push(GameEvent::PermanentUntapped { card_id });
                }
            }

            SpellEffect::UntapAll { target: req } => {
                let ids: Vec<CardId> = self.battlefield.iter()
                    .filter(|c| {
                        let tgt = Target::Permanent(c.id);
                        self.evaluate_requirement_static(req, &tgt, controller)
                    })
                    .map(|c| c.id)
                    .collect();
                for id in ids {
                    if let Some(c) = self.battlefield_find_mut(id) {
                        c.tapped = false;
                        events.push(GameEvent::PermanentUntapped { card_id: id });
                    }
                }
            }

            // ── Stack interaction ──────────────────────────────────────────────

            SpellEffect::CounterSpell { target: req } => {
                let _tgt = target.ok_or(GameError::TargetRequired)?;
                // CounterSpell targets a StackItem — simplified: remove top matching spell.
                // The `req` is checked against the top-of-stack card.
                let stack_pos = self.stack.iter().rposition(|item| {
                    if let StackItem::Spell { card, .. } = item {
                        let fake_target = Target::Permanent(card.id);
                        self.evaluate_requirement_static(req, &fake_target, controller)
                    } else {
                        false
                    }
                });
                if let Some(pos) = stack_pos {
                    if let StackItem::Spell { card, caster, .. } = self.stack.remove(pos) {
                        self.players[caster].send_to_graveyard(*card);
                    }
                } else if !self.stack.is_empty() {
                    return Err(GameError::StackEmpty);
                }
            }

            // ── Token creation ─────────────────────────────────────────────────

            SpellEffect::CreateTokens { count, definition } => {
                for _ in 0..*count {
                    let id = self.next_id();
                    let def = token_to_card_definition(definition);
                    let token = CardInstance::new(id, def, controller);
                    self.battlefield.push(token);
                    events.push(GameEvent::TokenCreated { card_id: id });
                    events.push(GameEvent::PermanentEntered { card_id: id });
                }
            }

            // ── Library manipulation ────────────────────────────────────────────

            SpellEffect::SearchLibrary { filter, put_into } => {
                let target_zone = *put_into;
                let candidates: Vec<(CardId, &'static str)> = self.players[controller]
                    .library
                    .iter()
                    .filter(|c| {
                        let tgt = Target::Permanent(c.id);
                        self.evaluate_requirement_static(filter, &tgt, controller)
                    })
                    .map(|c| (c.id, c.definition.name))
                    .collect();
                let ans = self.decider.decide(&Decision::SearchLibrary {
                    player: controller,
                    candidates,
                });
                let DecisionAnswer::Search(picked) = ans else {
                    return Err(GameError::InvalidTarget);
                };
                if let Some(card_id) = picked
                    && let Some(pos) = self.players[controller]
                        .library
                        .iter()
                        .position(|c| c.id == card_id)
                {
                    let card = self.players[controller].library.remove(pos);
                    match target_zone {
                        Zone::Hand => self.players[controller].hand.push(card),
                        Zone::Battlefield => {
                            let cid = card.id;
                            self.battlefield.push(card);
                            events.push(GameEvent::PermanentEntered { card_id: cid });
                        }
                        Zone::Graveyard => self.players[controller].graveyard.push(card),
                        _ => self.players[controller].library.insert(pos, card),
                    }
                }
                // Tutors shuffle after; we don't shuffle here to keep resolution
                // deterministic for tests. The caller can shuffle explicitly.
            }

            SpellEffect::ReturnFromGraveyard { filter: req, put_into } => {
                let target_zone = *put_into;
                let pos = self.players[controller].graveyard.iter().position(|c| {
                    let tgt = Target::Permanent(c.id);
                    self.evaluate_requirement_static(req, &tgt, controller)
                });
                if let Some(p) = pos {
                    let card = self.players[controller].graveyard.remove(p);
                    match target_zone {
                        Zone::Hand => self.players[controller].hand.push(card),
                        Zone::Battlefield => {
                            let card_id = card.id;
                            self.battlefield.push(card);
                            events.push(GameEvent::PermanentEntered { card_id });
                        }
                        Zone::Library => self.players[controller].library.push(card),
                        _ => {}
                    }
                }
            }

            // ── Iterated effects ────────────────────────────────────────────────

            SpellEffect::ForEachCreature { effects } => {
                let creature_ids: Vec<CardId> = self.battlefield.iter()
                    .filter(|c| c.definition.is_creature())
                    .map(|c| c.id)
                    .collect();
                for cid in creature_ids {
                    let tgt = Some(Target::Permanent(cid));
                    for effect in effects {
                        let mut eff_evs = self.resolve_effect(effect, controller, tgt.as_ref(), mode)?;
                        events.append(&mut eff_evs);
                    }
                }
            }

            SpellEffect::ForEachOpponent { effects } => {
                let opponents: Vec<usize> = (0..self.players.len()).filter(|&i| i != controller).collect();
                for opp in opponents {
                    let tgt = Some(Target::Player(opp));
                    for effect in effects {
                        let mut eff_evs = self.resolve_effect(effect, controller, tgt.as_ref(), mode)?;
                        events.append(&mut eff_evs);
                    }
                }
            }

            // ── Modal ───────────────────────────────────────────────────────────

            SpellEffect::ChooseOne { options } => {
                let chosen = mode;
                let option_effects = options.get(chosen)
                    .ok_or(GameError::ModeOutOfBounds(chosen))?
                    .clone();
                for effect in &option_effects {
                    let mut eff_evs = self.resolve_effect(effect, controller, target, mode)?;
                    events.append(&mut eff_evs);
                }
            }

            // ── Conditional ─────────────────────────────────────────────────────

            SpellEffect::Conditional { condition, then_effects, else_effects } => {
                let applies = self.evaluate_condition(condition, controller, target);
                let branch = if applies { then_effects } else { else_effects };
                let branch = branch.clone();
                for effect in &branch {
                    let mut eff_evs = self.resolve_effect(effect, controller, target, mode)?;
                    events.append(&mut eff_evs);
                }
            }

            // ── Deal damage to all ──────────────────────────────────────────────

            SpellEffect::DealDamageToAll { amount, target_filter } => {
                let amount = *amount;
                // Damage creatures
                let creature_ids: Vec<CardId> = self.battlefield.iter()
                    .filter(|c| {
                        let tgt = Target::Permanent(c.id);
                        self.evaluate_requirement_static(target_filter, &tgt, controller)
                    })
                    .map(|c| c.id)
                    .collect();
                for id in creature_ids {
                    if let Some(c) = self.battlefield_find_mut(id) {
                        c.damage += amount;
                        events.push(GameEvent::DamageDealt { amount, to_player: None, to_card: Some(id) });
                    }
                }
                // Damage players if filter includes players
                let player_filter = Target::Player(0); // proxy check
                if self.evaluate_requirement_static(target_filter, &player_filter, controller) {
                    for pidx in 0..self.players.len() {
                        self.players[pidx].life -= amount as i32;
                        events.push(GameEvent::DamageDealt { amount, to_player: Some(pidx), to_card: None });
                        events.push(GameEvent::LifeLost { player: pidx, amount });
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
            }

            // ── Life gain/loss (targeted) ─────────────────────────────────────

            SpellEffect::TargetGainsLife { amount, target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let amount = *amount;
                match tgt {
                    Target::Player(pidx) => {
                        self.players[*pidx].life += amount as i32;
                        events.push(GameEvent::LifeGained { player: *pidx, amount });
                    }
                    _ => return Err(GameError::InvalidTarget),
                }
            }

            SpellEffect::TargetLosesLife { amount, target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let amount = *amount;
                match tgt {
                    Target::Player(pidx) => {
                        self.players[*pidx].life -= amount as i32;
                        events.push(GameEvent::LifeLost { player: *pidx, amount });
                        let mut sba = self.check_state_based_actions();
                        events.append(&mut sba);
                    }
                    _ => return Err(GameError::InvalidTarget),
                }
            }

            SpellEffect::EachPlayerLosesLife { amount } => {
                let amount = *amount;
                for pidx in 0..self.players.len() {
                    self.players[pidx].life -= amount as i32;
                    events.push(GameEvent::LifeLost { player: pidx, amount });
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
            }

            SpellEffect::Drain { amount, target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let amount = *amount;
                match tgt {
                    Target::Player(pidx) => {
                        self.players[*pidx].life -= amount as i32;
                        events.push(GameEvent::LifeLost { player: *pidx, amount });
                    }
                    _ => return Err(GameError::InvalidTarget),
                }
                self.players[controller].life += amount as i32;
                events.push(GameEvent::LifeGained { player: controller, amount });
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
            }

            // ── Card selection ─────────────────────────────────────────────────

            SpellEffect::Surveil { amount } => {
                let n = (*amount as usize).min(self.players[controller].library.len());
                if n > 0 {
                    let peeked: Vec<(CardId, &'static str)> = self.players[controller]
                        .library
                        .iter()
                        .take(n)
                        .map(|c| (c.id, c.definition.name))
                        .collect();
                    self.suspend_signal = Some((
                        crate::decision::Decision::Scry { player: controller, cards: peeked },
                        PendingEffectState::SurveilPeeked { count: n, player: controller },
                    ));
                }
            }

            SpellEffect::LookAtTopCards { amount } => {
                // Simplified: just look, no choice made. Actual UI would show cards.
                let n = (*amount as usize).min(self.players[controller].library.len());
                for i in 0..n {
                    if let Some(card) = self.players[controller].library.get(i) {
                        let _name = card.definition.name; // revealed to player
                        let _ = _name;
                    }
                }
            }

            // ── Poison counters ────────────────────────────────────────────────

            SpellEffect::AddPoisonCounters { count, target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                let amount = *count;
                match tgt {
                    Target::Player(pidx) => {
                        self.players[*pidx].poison_counters += amount;
                        events.push(GameEvent::PoisonAdded { player: *pidx, amount });
                        let mut sba = self.check_state_based_actions();
                        events.append(&mut sba);
                    }
                    _ => return Err(GameError::InvalidTarget),
                }
            }

            // ── Proliferate ────────────────────────────────────────────────────

            SpellEffect::Proliferate => {
                // Add one of each counter type already on permanents and players.
                let ids: Vec<CardId> = self.battlefield.iter()
                    .filter(|c| !c.counters.is_empty())
                    .map(|c| c.id)
                    .collect();
                for id in ids {
                    if let Some(c) = self.battlefield_find_mut(id) {
                        let counter_types: Vec<_> = c.counters.keys().copied()
                            .filter(|&ct| c.counters[&ct] > 0)
                            .collect();
                        for ct in counter_types {
                            c.add_counters(ct, 1);
                            events.push(GameEvent::CounterAdded { card_id: id, counter_type: ct, count: 1 });
                        }
                    }
                }
                // Proliferate poison counters on players.
                for pidx in 0..self.players.len() {
                    if self.players[pidx].poison_counters > 0 {
                        self.players[pidx].poison_counters += 1;
                        events.push(GameEvent::PoisonAdded { player: pidx, amount: 1 });
                    }
                }
                let mut sba = self.check_state_based_actions();
                events.append(&mut sba);
            }

            // ── Control changing ───────────────────────────────────────────────

            SpellEffect::GainControlUntilEndOfTurn { target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                if let Target::Permanent(cid) = tgt {
                    let card_id = *cid;
                    let ts = self.next_timestamp();
                    self.add_continuous_effect(crate::game::layers::ContinuousEffect {
                        timestamp: ts,
                        source: card_id,
                        affected: crate::game::layers::AffectedPermanents::Specific(vec![card_id]),
                        layer: crate::game::layers::Layer::L2Control,
                        sublayer: None,
                        duration: crate::game::layers::EffectDuration::UntilEndOfTurn,
                        modification: crate::game::layers::Modification::ChangeController(controller),
                    });
                    // Give haste (standard for "gain control until end of turn" effects).
                    self.add_continuous_effect(crate::game::layers::ContinuousEffect {
                        timestamp: ts + 1,
                        source: card_id,
                        affected: crate::game::layers::AffectedPermanents::Specific(vec![card_id]),
                        layer: crate::game::layers::Layer::L6Ability,
                        sublayer: None,
                        duration: crate::game::layers::EffectDuration::UntilEndOfTurn,
                        modification: crate::game::layers::Modification::AddKeyword(crate::card::Keyword::Haste),
                    });
                }
            }

            SpellEffect::GainControl { target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                if let Target::Permanent(cid) = tgt {
                    let card_id = *cid;
                    let ts = self.next_timestamp();
                    self.add_continuous_effect(crate::game::layers::ContinuousEffect {
                        timestamp: ts,
                        source: card_id,
                        affected: crate::game::layers::AffectedPermanents::Specific(vec![card_id]),
                        layer: crate::game::layers::Layer::L2Control,
                        sublayer: None,
                        duration: crate::game::layers::EffectDuration::Indefinite,
                        modification: crate::game::layers::Modification::ChangeController(controller),
                    });
                }
            }

            // ── Keyword granting ──────────────────────────────────────────────

            SpellEffect::GrantKeywordUntilEndOfTurn { keyword, target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                if let Target::Permanent(cid) = tgt {
                    let card_id = *cid;
                    let ts = self.next_timestamp();
                    self.add_continuous_effect(crate::game::layers::ContinuousEffect {
                        timestamp: ts,
                        source: card_id,
                        affected: crate::game::layers::AffectedPermanents::Specific(vec![card_id]),
                        layer: crate::game::layers::Layer::L6Ability,
                        sublayer: None,
                        duration: crate::game::layers::EffectDuration::UntilEndOfTurn,
                        modification: crate::game::layers::Modification::AddKeyword(keyword.clone()),
                    });
                }
            }

            SpellEffect::GrantKeywordsUntilEndOfTurn { keywords, target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                if let Target::Permanent(cid) = tgt {
                    let card_id = *cid;
                    for keyword in keywords {
                        let ts = self.next_timestamp();
                        self.add_continuous_effect(crate::game::layers::ContinuousEffect {
                            timestamp: ts,
                            source: card_id,
                            affected: crate::game::layers::AffectedPermanents::Specific(vec![card_id]),
                            layer: crate::game::layers::Layer::L6Ability,
                            sublayer: None,
                            duration: crate::game::layers::EffectDuration::UntilEndOfTurn,
                            modification: crate::game::layers::Modification::AddKeyword(keyword.clone()),
                        });
                    }
                }
            }

            SpellEffect::GrantKeywordToYourCreaturesUntilEndOfTurn { keyword } => {
                let ts = self.next_timestamp();
                self.add_continuous_effect(crate::game::layers::ContinuousEffect {
                    timestamp: ts,
                    source: CardId(controller as u32),
                    affected: crate::game::layers::AffectedPermanents::All {
                        controller: Some(controller),
                        card_types: vec![crate::card::CardType::Creature],
                    },
                    layer: crate::game::layers::Layer::L6Ability,
                    sublayer: None,
                    duration: crate::game::layers::EffectDuration::UntilEndOfTurn,
                    modification: crate::game::layers::Modification::AddKeyword(keyword.clone()),
                });
            }

            // ── Token creation (specific) ─────────────────────────────────────

            SpellEffect::CreateFood { count } => {
                for _ in 0..*count {
                    let id = self.next_id();
                    let def = food_token_definition();
                    let mut token = CardInstance::new_token(id, def, controller);
                    token.is_token = true;
                    self.battlefield.push(token);
                    events.push(GameEvent::TokenCreated { card_id: id });
                    events.push(GameEvent::PermanentEntered { card_id: id });
                }
            }

            SpellEffect::CreateTreasure { count } => {
                for _ in 0..*count {
                    let id = self.next_id();
                    let def = treasure_token_definition();
                    let mut token = CardInstance::new_token(id, def, controller);
                    token.is_token = true;
                    self.battlefield.push(token);
                    events.push(GameEvent::TokenCreated { card_id: id });
                    events.push(GameEvent::PermanentEntered { card_id: id });
                }
            }

            SpellEffect::CreateBlood { count } => {
                for _ in 0..*count {
                    let id = self.next_id();
                    let def = blood_token_definition();
                    let mut token = CardInstance::new_token(id, def, controller);
                    token.is_token = true;
                    self.battlefield.push(token);
                    events.push(GameEvent::TokenCreated { card_id: id });
                    events.push(GameEvent::PermanentEntered { card_id: id });
                }
            }

            SpellEffect::Investigate { count } => {
                for _ in 0..*count {
                    let id = self.next_id();
                    let def = clue_token_definition();
                    let mut token = CardInstance::new_token(id, def, controller);
                    token.is_token = true;
                    self.battlefield.push(token);
                    events.push(GameEvent::TokenCreated { card_id: id });
                    events.push(GameEvent::PermanentEntered { card_id: id });
                }
            }

            // ── Sacrifice ─────────────────────────────────────────────────────

            SpellEffect::Sacrifice { count, filter } => {
                let n = *count as usize;
                let candidates: Vec<CardId> = self.battlefield.iter()
                    .filter(|c| {
                        c.controller == controller && {
                            let tgt = Target::Permanent(c.id);
                            self.evaluate_requirement_static(filter, &tgt, controller)
                        }
                    })
                    .map(|c| c.id)
                    .take(n)
                    .collect();
                for id in candidates {
                    let is_creature = self.battlefield_find(id).map(|c| c.definition.is_creature()).unwrap_or(false);
                    if is_creature { events.push(GameEvent::CreatureDied { card_id: id }); }
                    let mut die_evs = self.remove_to_graveyard_with_triggers(id);
                    events.append(&mut die_evs);
                }
            }

            SpellEffect::OpponentSacrifices { count, filter } => {
                let opp = (controller + 1) % self.players.len();
                let n = *count as usize;
                let candidates: Vec<CardId> = self.battlefield.iter()
                    .filter(|c| {
                        c.controller == opp && {
                            let tgt = Target::Permanent(c.id);
                            self.evaluate_requirement_static(filter, &tgt, opp)
                        }
                    })
                    .map(|c| c.id)
                    .take(n)
                    .collect();
                for id in candidates {
                    let is_creature = self.battlefield_find(id).map(|c| c.definition.is_creature()).unwrap_or(false);
                    if is_creature { events.push(GameEvent::CreatureDied { card_id: id }); }
                    let mut die_evs = self.remove_to_graveyard_with_triggers(id);
                    events.append(&mut die_evs);
                }
            }

            SpellEffect::EachPlayerSacrifices { count, filter } => {
                let n = *count as usize;
                for pidx in 0..self.players.len() {
                    let candidates: Vec<CardId> = self.battlefield.iter()
                        .filter(|c| {
                            c.controller == pidx && {
                                let tgt = Target::Permanent(c.id);
                                self.evaluate_requirement_static(filter, &tgt, pidx)
                            }
                        })
                        .map(|c| c.id)
                        .take(n)
                        .collect();
                    for id in candidates {
                        let is_creature = self.battlefield_find(id).map(|c| c.definition.is_creature()).unwrap_or(false);
                        if is_creature { events.push(GameEvent::CreatureDied { card_id: id }); }
                        let mut die_evs = self.remove_to_graveyard_with_triggers(id);
                        events.append(&mut die_evs);
                    }
                }
            }

            // ── Discard (controller choice) ────────────────────────────────────

            SpellEffect::DiscardCards { amount } => {
                let n = (*amount as usize).min(self.players[controller].hand.len());
                for _ in 0..n {
                    if !self.players[controller].hand.is_empty() {
                        let card = self.players[controller].hand.remove(0);
                        let card_id = card.id;
                        self.players[controller].send_to_graveyard(card);
                        events.push(GameEvent::CardDiscarded { player: controller, card_id });
                    }
                }
            }

            SpellEffect::DiscardToHandSize { hand_size } => {
                let opp = (controller + 1) % self.players.len();
                let max = *hand_size as usize;
                while self.players[opp].hand.len() > max {
                    let card = self.players[opp].hand.remove(0);
                    let card_id = card.id;
                    self.players[opp].send_to_graveyard(card);
                    events.push(GameEvent::CardDiscarded { player: opp, card_id });
                }
            }

            // ── Copy effects ──────────────────────────────────────────────────

            SpellEffect::CopyTopSpell | SpellEffect::CreateCopies { .. } => {
                // Copying is complex (needs full stack copy). Placeholder: no-op.
            }

            // ── Graveyard/exile manipulation ──────────────────────────────────

            SpellEffect::ShuffleGraveyardIntoLibrary => {
                let graveyard = std::mem::take(&mut self.players[controller].graveyard);
                for card in graveyard {
                    self.players[controller].library.push(card);
                }
                // Shuffle not implemented deterministically; order remains.
            }

            SpellEffect::ReturnFromExile { filter: req } => {
                let pos = self.exile.iter().position(|c| {
                    c.owner == controller && {
                        let tgt = Target::Permanent(c.id);
                        self.evaluate_requirement_static(req, &tgt, controller)
                    }
                });
                if let Some(p) = pos {
                    let card = self.exile.remove(p);
                    self.players[controller].hand.push(card);
                }
            }

            SpellEffect::ReanimateFromGraveyard { filter: req, controller: reanimate_ctrl } => {
                use crate::card::ReanimateController;
                let target_player = match reanimate_ctrl {
                    ReanimateController::Caster => controller,
                    ReanimateController::OriginalOwner => controller, // simplified
                };
                // Find any card in any player's graveyard matching the filter.
                let found = (0..self.players.len()).find_map(|pidx| {
                    self.players[pidx].graveyard.iter().position(|c| {
                        let tgt = Target::Permanent(c.id);
                        self.evaluate_requirement_static(req, &tgt, controller)
                    }).map(|pos| (pidx, pos))
                });
                if let Some((from_player, pos)) = found {
                    let mut card = self.players[from_player].graveyard.remove(pos);
                    card.controller = target_player;
                    card.damage = 0;
                    card.summoning_sick = true;
                    let cid = card.id;
                    self.battlefield.push(card);
                    events.push(GameEvent::PermanentEntered { card_id: cid });
                }
            }

            SpellEffect::ResetCreature { target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                if let Target::Permanent(cid) = tgt {
                    let card_id = *cid;
                    let ts = self.next_timestamp();
                    // Remove all abilities and set P/T to 1/1.
                    self.add_continuous_effect(crate::game::layers::ContinuousEffect {
                        timestamp: ts,
                        source: card_id,
                        affected: crate::game::layers::AffectedPermanents::Specific(vec![card_id]),
                        layer: crate::game::layers::Layer::L6Ability,
                        sublayer: None,
                        duration: crate::game::layers::EffectDuration::UntilEndOfTurn,
                        modification: crate::game::layers::Modification::RemoveAllAbilities,
                    });
                    self.add_continuous_effect(crate::game::layers::ContinuousEffect {
                        timestamp: ts + 1,
                        source: card_id,
                        affected: crate::game::layers::AffectedPermanents::Specific(vec![card_id]),
                        layer: crate::game::layers::Layer::L7PowerTough,
                        sublayer: Some(crate::game::layers::PtSublayer::SetValue),
                        duration: crate::game::layers::EffectDuration::UntilEndOfTurn,
                        modification: crate::game::layers::Modification::SetPowerToughness(1, 1),
                    });
                }
            }

            SpellEffect::BecomeBasicLand { land_type, target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                if let Target::Permanent(cid) = tgt {
                    let card_id = *cid;
                    let ts = self.next_timestamp();
                    self.add_continuous_effect(crate::game::layers::ContinuousEffect {
                        timestamp: ts,
                        source: card_id,
                        affected: crate::game::layers::AffectedPermanents::Specific(vec![card_id]),
                        layer: crate::game::layers::Layer::L4Type,
                        sublayer: None,
                        duration: crate::game::layers::EffectDuration::Indefinite,
                        modification: crate::game::layers::Modification::AddLandType(*land_type),
                    });
                }
            }

            SpellEffect::DrawFromTop { amount, target: req } => {
                let tgt = target.ok_or(GameError::TargetRequired)?;
                if !self.evaluate_requirement(req, tgt, controller) {
                    return Err(GameError::SelectionRequirementViolated);
                }
                if let Target::Player(pidx) = tgt {
                    for _ in 0..*amount {
                        if let Some(id) = self.players[*pidx].draw_top() {
                            events.push(GameEvent::CardDrawn { player: *pidx, card_id: id });
                        }
                    }
                }
            }

            // ── Legacy effects ─────────────────────────────────────────────────

            SpellEffect::RevealOpponentTopCard => {
                let opp = (controller + 1) % self.players.len();
                if let Some(top) = self.players[opp].library.first() {
                    let name = top.definition.name;
                    let is_land = top.definition.is_land();
                    events.push(GameEvent::TopCardRevealed { player: opp, card_name: name, is_land });
                    if is_land
                        && let Some(id) = self.players[opp].draw_top() {
                            events.push(GameEvent::CardDrawn { player: opp, card_id: id });
                        }
                }
            }

            SpellEffect::OpponentDiscardRandom => {
                let opp = (controller + 1) % self.players.len();
                if !self.players[opp].hand.is_empty() {
                    let card = self.players[opp].hand.remove(0);
                    let card_id = card.id;
                    self.players[opp].send_to_graveyard(card);
                    events.push(GameEvent::CardDiscarded { player: opp, card_id });
                }
            }

        }
        Ok(events)
    }

    /// Evaluate a boolean `EffectCondition` at resolution time.
    fn evaluate_condition(
        &self,
        condition: &EffectCondition,
        controller: usize,
        _target: Option<&Target>,
    ) -> bool {
        match condition {
            EffectCondition::ControllerControls(req) => {
                self.battlefield.iter().any(|c| {
                    c.controller == controller && {
                        let tgt = Target::Permanent(c.id);
                        self.evaluate_requirement_static(req, &tgt, controller)
                    }
                })
            }
            EffectCondition::TargetHasCounter(ct, n) => {
                if let Some(Target::Permanent(cid)) = _target {
                    self.battlefield_find(*cid)
                        .map(|c| c.counter_count(*ct) >= *n)
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            EffectCondition::ControllerLifeAtMost(threshold) => {
                self.players[controller].life <= *threshold
            }
            EffectCondition::ControllerGraveyardAtLeast(n) => {
                self.players[controller].graveyard.len() >= *n
            }
            EffectCondition::OpponentGraveyardAtLeast(n) => {
                let opp = (controller + 1) % self.players.len();
                self.players[opp].graveyard.len() >= *n
            }
            EffectCondition::ControllerHandAtLeast(n) => {
                self.players[controller].hand.len() >= *n
            }
            EffectCondition::ControllerHandEmpty => {
                self.players[controller].hand.is_empty()
            }
            EffectCondition::ControllerControlsCreatureType(ct) => {
                self.battlefield.iter().any(|c| {
                    c.controller == controller
                        && c.definition.is_creature()
                        && (c.definition.subtypes.creature_types.contains(ct)
                            || c.definition.keywords.contains(&crate::card::Keyword::Changeling))
                })
            }
            EffectCondition::ControllerGraveyardCreaturesAtLeast(n) => {
                let count = self.players[controller].graveyard.iter()
                    .filter(|c| c.definition.is_creature())
                    .count();
                count >= *n
            }
            EffectCondition::ControllerGraveyardHasLand => {
                self.players[controller].graveyard.iter().any(|c| c.definition.is_land())
            }
            EffectCondition::ControllerLandCountAtLeast(n) => {
                let count = self.battlefield.iter()
                    .filter(|c| c.controller == controller && c.definition.is_land())
                    .count();
                count >= *n
            }
            EffectCondition::IsControllersTurn => {
                self.active_player_idx == controller
            }
            EffectCondition::SpellsCastThisTurnAtLeast(n) => {
                self.spells_cast_this_turn as usize >= *n
            }
            EffectCondition::IsAttacking => {
                if let Some(Target::Permanent(cid)) = _target {
                    self.attacking.contains(cid)
                } else { false }
            }
            EffectCondition::IsBlocking => {
                if let Some(Target::Permanent(cid)) = _target {
                    self.block_map.contains_key(cid)
                } else { false }
            }
            EffectCondition::TargetIsToken => {
                if let Some(Target::Permanent(cid)) = _target {
                    self.battlefield_find(*cid).map(|c| c.is_token).unwrap_or(false)
                } else { false }
            }
        }
    }

    /// Same as `evaluate_requirement` but callable from methods that hold `&self`
    /// (avoids borrow checker issues when iterating `battlefield`).
    pub(crate) fn evaluate_requirement_static(
        &self,
        req: &SelectionRequirement,
        target: &Target,
        controller: usize,
    ) -> bool {
        match req {
            SelectionRequirement::Any => true,
            SelectionRequirement::Player => matches!(target, Target::Player(_)),

            SelectionRequirement::Permanent => matches!(target, Target::Permanent(_)),

            SelectionRequirement::Creature => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_creature())
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::Artifact => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_artifact())
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::Enchantment => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_enchantment())
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::Planeswalker => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_planeswalker())
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::Land => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_land())
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::Nonland => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| !c.definition.is_land())
                    .unwrap_or(false),
                _ => true, // players are not lands
            },
            SelectionRequirement::Noncreature => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| !c.definition.is_creature())
                    .unwrap_or(false),
                _ => true,
            },
            SelectionRequirement::Tapped => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.tapped)
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::Untapped => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| !c.tapped)
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::HasColor(color) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| card_has_color(c, *color))
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::HasKeyword(kw) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.has_keyword(kw))
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::PowerAtMost(n) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_creature() && c.power() <= *n)
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::ToughnessAtMost(n) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_creature() && c.toughness() <= *n)
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::WithCounter(ct) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.counter_count(*ct) > 0)
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::ControlledByYou => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.controller == controller)
                    .unwrap_or(false),
                Target::Player(pidx) => *pidx == controller,
            },
            SelectionRequirement::ControlledByOpponent => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.controller != controller)
                    .unwrap_or(false),
                Target::Player(pidx) => *pidx != controller,
            },
            SelectionRequirement::HasSupertype(st) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.supertypes.contains(st))
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::HasCreatureType(ct) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.subtypes.creature_types.contains(ct))
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::HasLandType(lt) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.subtypes.land_types.contains(lt))
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::HasArtifactSubtype(at) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.subtypes.artifact_subtypes.contains(at))
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::HasEnchantmentSubtype(et) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.subtypes.enchantment_subtypes.contains(et))
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::PowerAtLeast(n) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_creature() && c.power() >= *n)
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::ToughnessAtLeast(n) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_creature() && c.toughness() >= *n)
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::IsToken => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.is_token)
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::NotToken => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| !c.is_token)
                    .unwrap_or(true),
                _ => true,
            },
            SelectionRequirement::IsBasicLand => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.is_land() && c.definition.is_basic())
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::IsAttacking => match target {
                Target::Permanent(cid) => self.attacking.contains(cid),
                _ => false,
            },
            SelectionRequirement::IsBlocking => match target {
                Target::Permanent(cid) => self.block_map.contains_key(cid),
                _ => false,
            },
            SelectionRequirement::IsSpellOnStack => {
                false // stack targeting requires separate logic
            }
            SelectionRequirement::ManaValueAtMost(n) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.cost.cmc() <= *n)
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::ManaValueAtLeast(n) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.cost.cmc() >= *n)
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::HasCardType(card_type) => match target {
                Target::Permanent(cid) => self
                    .battlefield_find(*cid)
                    .map(|c| c.definition.card_types.contains(card_type))
                    .unwrap_or(false),
                _ => false,
            },
            SelectionRequirement::And(a, b) => {
                self.evaluate_requirement_static(a, target, controller)
                    && self.evaluate_requirement_static(b, target, controller)
            }
            SelectionRequirement::Or(a, b) => {
                self.evaluate_requirement_static(a, target, controller)
                    || self.evaluate_requirement_static(b, target, controller)
            }
            SelectionRequirement::Not(a) => !self.evaluate_requirement_static(a, target, controller),
        }
    }

    /// Pick a sensible auto-target for a single effect triggered by `controller`.
    pub(crate) fn auto_target_for_effect(&self, effect: &SpellEffect, controller: usize) -> Option<Target> {
        let opp = (controller + 1) % self.players.len();
        match effect {
            SpellEffect::DealDamage { .. } => {
                Some(Target::Player(opp))
            }
            SpellEffect::DestroyCreature { .. } => {
                self.battlefield.iter()
                    .find(|c| c.owner == opp && c.definition.is_creature())
                    .map(|c| Target::Permanent(c.id))
            }
            SpellEffect::PumpCreature { .. } => {
                self.battlefield.iter()
                    .find(|c| c.owner == controller && c.definition.is_creature())
                    .map(|c| Target::Permanent(c.id))
            }
            // These effects are controller-relative; no Target needed
            SpellEffect::RevealOpponentTopCard | SpellEffect::OpponentDiscardRandom => None,
            _ => None,
        }
    }

    /// Pick a sensible auto-target for a slice of effects triggered by `controller`.
    /// Returns the first non-None target found across all effects.
    pub(crate) fn auto_target_for_effects(&self, effects: &[SpellEffect], controller: usize) -> Option<Target> {
        effects.iter().find_map(|e| self.auto_target_for_effect(e, controller))
    }
}

// ── Specific token definitions ─────────────────────────────────────────────────

fn food_token_definition() -> CardDefinition {
    use crate::card::{ArtifactSubtype, Subtypes};
    CardDefinition {
        name: "Food",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![crate::card::CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Food],
            ..Default::default()
        },
        power: 0, toughness: 0, base_loyalty: 0,
        keywords: vec![],
        static_abilities: vec![],
        spell_effects: vec![],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::new(vec![crate::mana::ManaSymbol::Generic(2)]),
            effects: vec![SpellEffect::GainLife { amount: 3 }],
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        loyalty_abilities: vec![],
    }
}

fn treasure_token_definition() -> CardDefinition {
    use crate::card::{ArtifactSubtype, Subtypes};
    CardDefinition {
        name: "Treasure",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![crate::card::CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Treasure],
            ..Default::default()
        },
        power: 0, toughness: 0, base_loyalty: 0,
        keywords: vec![],
        static_abilities: vec![],
        spell_effects: vec![],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effects: vec![SpellEffect::AddManaAnyColor { amount: 1 }],
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        loyalty_abilities: vec![],
    }
}

fn blood_token_definition() -> CardDefinition {
    use crate::card::{ArtifactSubtype, Subtypes};
    CardDefinition {
        name: "Blood",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![crate::card::CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Blood],
            ..Default::default()
        },
        power: 0, toughness: 0, base_loyalty: 0,
        keywords: vec![],
        static_abilities: vec![],
        spell_effects: vec![],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::new(vec![crate::mana::ManaSymbol::Generic(1)]),
            effects: vec![SpellEffect::DiscardCards { amount: 1 }, SpellEffect::DrawCards { amount: 1 }],
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        loyalty_abilities: vec![],
    }
}

fn clue_token_definition() -> CardDefinition {
    use crate::card::{ArtifactSubtype, Subtypes};
    CardDefinition {
        name: "Clue",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![crate::card::CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Clue],
            ..Default::default()
        },
        power: 0, toughness: 0, base_loyalty: 0,
        keywords: vec![],
        static_abilities: vec![],
        spell_effects: vec![],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::new(vec![crate::mana::ManaSymbol::Generic(2)]),
            effects: vec![SpellEffect::DrawCards { amount: 1 }],
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        loyalty_abilities: vec![],
    }
}

fn card_has_color(card: &CardInstance, color: Color) -> bool {
    card.definition.cost.symbols.iter().any(|s| match s {
        ManaSymbol::Colored(c) => *c == color,
        ManaSymbol::Hybrid(a, b) => *a == color || *b == color,
        ManaSymbol::Phyrexian(c) => *c == color,
        _ => false,
    })
}

/// Convert a `TokenDefinition` into a minimal `CardDefinition` that can be
/// placed on the battlefield as a `CardInstance`.
fn token_to_card_definition(token: &TokenDefinition) -> CardDefinition {
    CardDefinition {
        name: token.name,
        cost: ManaCost::default(),
        supertypes: token.supertypes.clone(),
        card_types: token.card_types.clone(),
        subtypes: token.subtypes.clone(),
        power: token.power,
        toughness: token.toughness,
        base_loyalty: 0,
        keywords: token.keywords.clone(),
        static_abilities: vec![],
        spell_effects: vec![],
        activated_abilities: vec![],
        triggered_abilities: vec![],
        loyalty_abilities: vec![],
    }
}
