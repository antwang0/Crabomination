use super::*;
use crate::card::Keyword;
use crate::effect::{Effect, EventKind};
use crate::game::layers::ComputedPermanent;

impl GameState {
    // ── Declare attackers ─────────────────────────────────────────────────────

    pub(crate) fn declare_attackers(
        &mut self,
        ids: Vec<CardId>,
    ) -> Result<Vec<GameEvent>, GameError> {
        if self.step != TurnStep::DeclareAttackers {
            return Err(GameError::WrongStep { actual: self.step });
        }
        let p = self.priority.player_with_priority;
        if p != self.active_player_idx {
            return Err(GameError::NotYourPriority);
        }
        let mut events = vec![];
        let mut triggers: Vec<(CardId, Effect, usize)> = vec![];
        let computed = self.compute_battlefield();
        let computed_kw = |id: CardId| -> &[Keyword] {
            computed
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.keywords.as_slice())
                .unwrap_or(&[])
        };

        for id in ids {
            let card = self
                .battlefield
                .iter_mut()
                .find(|c| c.id == id && c.owner == p)
                .ok_or(GameError::CardNotOnBattlefield(id))?;

            if !card.can_attack() {
                if card.tapped {
                    return Err(GameError::CardIsTapped(id));
                }
                return Err(GameError::SummoningSickness(id));
            }
            // Vigilant creatures don't tap when attacking (use computed keyword)
            if !computed_kw(id).contains(&Keyword::Vigilance) {
                card.tapped = true;
            }
            self.attacking.push(id);
            events.push(GameEvent::AttackerDeclared(id));
            // Collect attack triggers (pushed after the loop to avoid borrow conflict)
            for t in &card.definition.triggered_abilities {
                if t.event.kind == EventKind::Attacks {
                    triggers.push((id, t.effect.clone(), p));
                }
            }
            // Annihilator: TODO — translate to Effect tree (no-op for now).
            let _annihilator_n = computed_kw(id).iter().find_map(|kw| {
                if let Keyword::Annihilator(n) = kw {
                    Some(*n)
                } else {
                    None
                }
            });
        }
        // Push collected attack triggers onto the stack.
        for (source, effect, controller) in triggers {
            let auto_target = self.auto_target_for_effect(&effect, controller);
            self.stack.push(StackItem::Trigger {
                source,
                controller,
                effect: Box::new(effect),
                target: auto_target,
                mode: None,
            });
        }
        // Active player gets priority after attackers are declared.
        self.give_priority_to_active();
        Ok(events)
    }

    // ── Declare blockers ──────────────────────────────────────────────────────

    pub(crate) fn declare_blockers(
        &mut self,
        assignments: Vec<(CardId, CardId)>,
    ) -> Result<Vec<GameEvent>, GameError> {
        if self.step != TurnStep::DeclareBlockers {
            return Err(GameError::WrongStep { actual: self.step });
        }
        let defender = (self.active_player_idx + 1) % self.players.len();

        // Compute layer-resolved keywords for all permanents.
        let computed = self.compute_battlefield();
        let cp_of = |id: CardId| computed.iter().find(|c| c.id == id);
        let kws_of = |id: CardId| -> &[Keyword] {
            computed
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.keywords.as_slice())
                .unwrap_or(&[])
        };

        // Validate ALL assignments before mutating any state.
        for &(blocker_id, attacker_id) in &assignments {
            let blocker = self
                .battlefield
                .iter()
                .find(|c| c.id == blocker_id && c.owner == defender)
                .ok_or(GameError::CardNotOnBattlefield(blocker_id))?;

            if !blocker.can_block() {
                return Err(GameError::CannotBlock(blocker_id));
            }

            let attacker = self
                .battlefield_find(attacker_id)
                .ok_or(GameError::CardNotOnBattlefield(attacker_id))?;

            let blocker_cp = cp_of(blocker_id).ok_or(GameError::CannotBlock(blocker_id))?;
            if !super::can_block_attacker_computed(
                blocker,
                attacker,
                blocker_cp,
                kws_of(attacker_id),
            ) {
                return Err(GameError::CannotBlock(blocker_id));
            }
        }

        // Menace: attackers with Menace must be blocked by 2+ creatures or not at all.
        for &atk_id in &self.attacking {
            let has_menace = kws_of(atk_id).contains(&Keyword::Menace);
            if has_menace {
                let blocker_count = assignments.iter().filter(|(_, aid)| *aid == atk_id).count();
                if blocker_count == 1 {
                    return Err(GameError::MenaceRequiresTwoBlockers(atk_id));
                }
            }
        }

        // All valid — apply.
        self.blockers_declared = true;
        let mut events = vec![];
        for (blocker_id, attacker_id) in assignments {
            self.block_map.insert(blocker_id, attacker_id);
            events.push(GameEvent::BlockerDeclared {
                blocker: blocker_id,
                attacker: attacker_id,
            });
        }
        // Active player gets priority after blockers are declared.
        self.give_priority_to_active();
        Ok(events)
    }

    // ── Combat resolution ─────────────────────────────────────────────────────

    /// Returns true if there are any first-strike or double-strike creatures in combat.
    pub(crate) fn has_first_strikers(&self) -> bool {
        let computed = self.compute_battlefield();
        let kws_of = |id: CardId| -> &[Keyword] {
            computed
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.keywords.as_slice())
                .unwrap_or(&[])
        };
        self.attacking.iter().any(|&id| {
            kws_of(id).contains(&Keyword::FirstStrike)
                || kws_of(id).contains(&Keyword::DoubleStrike)
        }) || self.block_map.keys().any(|&id| {
            kws_of(id).contains(&Keyword::FirstStrike)
                || kws_of(id).contains(&Keyword::DoubleStrike)
        })
    }

    /// Resolve the first-strike damage step.
    /// Only first-strike and double-strike creatures deal damage here.
    pub(crate) fn resolve_first_strike_damage(&mut self) -> Result<Vec<GameEvent>, GameError> {
        let computed = self.compute_battlefield();
        let mut events = self.resolve_combat_damage_with_filter(
            &computed,
            |kws: &[Keyword]| {
                kws.contains(&Keyword::FirstStrike) || kws.contains(&Keyword::DoubleStrike)
            },
            |_kws: &[Keyword]| false, // blockers with first strike
        )?;
        // State-based actions between damage steps.
        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);
        events.push(GameEvent::FirstStrikeDamageResolved);
        Ok(events)
    }

    /// Resolve the normal (non-first-strike) damage step.
    /// Only non-first-strike (and double-strike) creatures deal damage here.
    pub(crate) fn resolve_combat(&mut self) -> Result<Vec<GameEvent>, GameError> {
        let computed = self.compute_battlefield();
        let mut events = self.resolve_combat_damage_with_filter(
            &computed,
            |kws: &[Keyword]| {
                !kws.contains(&Keyword::FirstStrike) || kws.contains(&Keyword::DoubleStrike)
            },
            |_kws: &[Keyword]| true, // all remaining blockers deal damage
        )?;

        // State-based actions after all damage
        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);

        // Reset combat state
        self.attacking.clear();
        self.block_map.clear();
        self.blockers_declared = false;

        events.push(GameEvent::CombatResolved);
        Ok(events)
    }

    /// Core combat damage resolver. `attacker_filter` returns true if a given
    /// attacker should deal damage in this step (based on its keyword set).
    /// `blocker_filter` returns true if a given blocker deals damage back.
    fn resolve_combat_damage_with_filter(
        &mut self,
        computed: &[ComputedPermanent],
        attacker_filter: impl Fn(&[Keyword]) -> bool,
        _blocker_filter: impl Fn(&[Keyword]) -> bool,
    ) -> Result<Vec<GameEvent>, GameError> {
        let mut events = vec![];
        let defender_idx = (self.active_player_idx + 1) % self.players.len();

        let computed_of =
            |id: CardId| -> Option<&ComputedPermanent> { computed.iter().find(|c| c.id == id) };

        // Snapshot attacker data (using layer-computed P/T and keywords).
        struct AttackerInfo {
            id: CardId,
            power: i32,
            has_trample: bool,
            has_lifelink: bool,
            has_deathtouch: bool,
            has_first_strike: bool,
            has_double_strike: bool,
            has_infect: bool,
            has_wither: bool,
            should_deal: bool,
        }

        let attacker_infos: Vec<AttackerInfo> = self
            .attacking
            .iter()
            .filter_map(|&aid| {
                computed_of(aid).map(|c| {
                    let kws = &c.keywords;
                    AttackerInfo {
                        id: c.id,
                        power: c.power,
                        has_trample: kws.contains(&Keyword::Trample),
                        has_lifelink: kws.contains(&Keyword::Lifelink),
                        has_deathtouch: kws.contains(&Keyword::Deathtouch),
                        has_first_strike: kws.contains(&Keyword::FirstStrike),
                        has_double_strike: kws.contains(&Keyword::DoubleStrike),
                        has_infect: kws.contains(&Keyword::Infect),
                        has_wither: kws.contains(&Keyword::Wither),
                        should_deal: attacker_filter(kws),
                    }
                })
            })
            .collect();

        for atk in &attacker_infos {
            if !atk.should_deal {
                continue;
            }

            let blocker_ids: Vec<CardId> = self
                .block_map
                .iter()
                .filter(|(_, aid)| **aid == atk.id)
                .map(|(&bid, _)| bid)
                .collect();

            if blocker_ids.is_empty() {
                // Unblocked — deal damage to defending player.
                let amount = atk.power.max(0) as u32;
                if amount > 0 {
                    if atk.has_infect {
                        // Infect: deal damage as poison counters to players.
                        self.players[defender_idx].poison_counters += amount;
                        events.push(GameEvent::PoisonAdded {
                            player: defender_idx,
                            amount,
                        });
                    } else {
                        self.players[defender_idx].life -= atk.power;
                        events.push(GameEvent::DamageDealt {
                            amount,
                            to_player: Some(defender_idx),
                            to_card: None,
                        });
                        events.push(GameEvent::LifeLost {
                            player: defender_idx,
                            amount,
                        });
                    }
                    if atk.has_lifelink {
                        let a = self.active_player_idx;
                        self.players[a].life += atk.power;
                        events.push(GameEvent::LifeGained { player: a, amount });
                    }
                }
            } else {
                // Blocked — distribute attacker damage among blockers (sequential).
                let mut remaining_atk_damage = atk.power;
                let mut lifelink_dealt = 0i32;

                for &blocker_id in &blocker_ids {
                    if remaining_atk_damage <= 0 {
                        break;
                    }
                    let blocker_toughness =
                        computed_of(blocker_id).map(|c| c.toughness).unwrap_or(0);
                    let lethal = if atk.has_deathtouch {
                        1
                    } else {
                        blocker_toughness
                    };
                    let assign = remaining_atk_damage.min(lethal);
                    remaining_atk_damage -= assign;
                    lifelink_dealt += assign;

                    if atk.has_infect || atk.has_wither {
                        // Infect/Wither: deal as -1/-1 counters to creatures.
                        if assign > 0
                            && let Some(blocker) = self.battlefield_find_mut(blocker_id)
                        {
                            blocker.add_counters(
                                crate::card::CounterType::MinusOneMinusOne,
                                assign as u32,
                            );
                            events.push(GameEvent::CounterAdded {
                                card_id: blocker_id,
                                counter_type: crate::card::CounterType::MinusOneMinusOne,
                                count: assign as u32,
                            });
                        }
                    } else if let Some(blocker) = self.battlefield_find_mut(blocker_id) {
                        blocker.damage += assign as u32;
                        events.push(GameEvent::DamageDealt {
                            amount: assign as u32,
                            to_player: None,
                            to_card: Some(blocker_id),
                        });
                    }
                }

                // Trample: excess damage to player.
                if atk.has_trample && remaining_atk_damage > 0 {
                    let amount = remaining_atk_damage as u32;
                    lifelink_dealt += remaining_atk_damage;
                    if atk.has_infect {
                        self.players[defender_idx].poison_counters += amount;
                        events.push(GameEvent::PoisonAdded {
                            player: defender_idx,
                            amount,
                        });
                    } else {
                        self.players[defender_idx].life -= remaining_atk_damage;
                        events.push(GameEvent::DamageDealt {
                            amount,
                            to_player: Some(defender_idx),
                            to_card: None,
                        });
                        events.push(GameEvent::LifeLost {
                            player: defender_idx,
                            amount,
                        });
                    }
                }

                if atk.has_lifelink && lifelink_dealt > 0 {
                    let a = self.active_player_idx;
                    self.players[a].life += lifelink_dealt;
                    events.push(GameEvent::LifeGained {
                        player: a,
                        amount: lifelink_dealt as u32,
                    });
                }

                // Blockers deal damage back to the attacker.
                // In the first-strike step, only first/double-strike blockers deal damage.
                // In the normal step, all non-first-strike and double-strike blockers deal damage.
                let blocker_damage_to_attacker: i32 = blocker_ids
                    .iter()
                    .filter_map(|&bid| computed_of(bid))
                    .filter(|bc| {
                        // In this simplified version, all blockers deal damage in the normal step.
                        !bc.keywords.contains(&Keyword::FirstStrike)
                            || bc.keywords.contains(&Keyword::DoubleStrike)
                            || atk.has_first_strike
                            || atk.has_double_strike
                    })
                    .map(|c| c.power)
                    .sum();

                if blocker_damage_to_attacker > 0 {
                    let any_deathtouch_blocker = blocker_ids
                        .iter()
                        .filter_map(|&bid| computed_of(bid))
                        .any(|c| c.keywords.contains(&Keyword::Deathtouch));
                    let any_infect_blocker = blocker_ids
                        .iter()
                        .filter_map(|&bid| computed_of(bid))
                        .any(|c| {
                            c.keywords.contains(&Keyword::Infect)
                                || c.keywords.contains(&Keyword::Wither)
                        });
                    let attacker_toughness = computed_of(atk.id).map(|c| c.toughness).unwrap_or(0);
                    if let Some(attacker) = self.battlefield_find_mut(atk.id) {
                        if any_infect_blocker {
                            let dmg = blocker_damage_to_attacker.max(0) as u32;
                            attacker.add_counters(crate::card::CounterType::MinusOneMinusOne, dmg);
                            events.push(GameEvent::CounterAdded {
                                card_id: atk.id,
                                counter_type: crate::card::CounterType::MinusOneMinusOne,
                                count: dmg,
                            });
                        } else if any_deathtouch_blocker {
                            attacker.damage = attacker.damage.max(attacker_toughness as u32);
                            events.push(GameEvent::DamageDealt {
                                amount: blocker_damage_to_attacker.max(0) as u32,
                                to_player: None,
                                to_card: Some(atk.id),
                            });
                        } else {
                            attacker.damage += blocker_damage_to_attacker.max(0) as u32;
                            events.push(GameEvent::DamageDealt {
                                amount: blocker_damage_to_attacker.max(0) as u32,
                                to_player: None,
                                to_card: Some(atk.id),
                            });
                        }
                        // Blocker lifelink
                        let blocker_lifelink_total: i32 = blocker_ids
                            .iter()
                            .filter_map(|&bid| computed_of(bid))
                            .filter(|bc| bc.keywords.contains(&Keyword::Lifelink))
                            .map(|bc| bc.power)
                            .sum();
                        if blocker_lifelink_total > 0 {
                            self.players[defender_idx].life += blocker_lifelink_total;
                            events.push(GameEvent::LifeGained {
                                player: defender_idx,
                                amount: blocker_lifelink_total as u32,
                            });
                        }
                    }
                }
            }
        }

        Ok(events)
    }
}
