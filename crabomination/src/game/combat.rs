use super::*;
use crate::card::Keyword;
use crate::effect::{Effect, EventKind, EventScope};
use crate::game::layers::ComputedPermanent;

impl GameState {
    // ── Declare attackers ─────────────────────────────────────────────────────

    pub(crate) fn declare_attackers(
        &mut self,
        attacks: Vec<Attack>,
    ) -> Result<Vec<GameEvent>, GameError> {
        if self.step != TurnStep::DeclareAttackers {
            return Err(GameError::WrongStep { actual: self.step });
        }
        let p = self.priority.player_with_priority;
        if p != self.active_player_idx {
            return Err(GameError::NotYourPriority);
        }

        // Validate every attack target up-front.
        for atk in &attacks {
            match atk.target {
                AttackTarget::Player(target_player) => {
                    if target_player == self.active_player_idx
                        || target_player >= self.players.len()
                        || !self.players[target_player].is_alive()
                    {
                        return Err(GameError::InvalidAttackTarget(target_player));
                    }
                }
                AttackTarget::Planeswalker(pw_id) => {
                    let pw = self
                        .battlefield_find(pw_id)
                        .ok_or(GameError::InvalidPlaneswalkerAttackTarget(pw_id))?;
                    if !pw.definition.is_planeswalker()
                        || pw.controller == self.active_player_idx
                        || !self.players[pw.controller].is_alive()
                    {
                        return Err(GameError::InvalidPlaneswalkerAttackTarget(pw_id));
                    }
                }
            }
        }

        let mut events = vec![];
        let mut triggers: Vec<(CardId, Effect, usize)> = vec![];
        // Pending broadcast triggers — collected during attack
        // processing (so each attacker contributes its broadcast
        // tuples), then evaluated in a second pass *after* every
        // attacker is registered in `self.attacking`. This way
        // `AttackersThisCombat`-keyed gates (Augusta, Dean of Order's
        // "two or more creatures attack" rider) read the final
        // post-declaration count and gate uniformly across all
        // attackers, rather than off-by-one against the declaration
        // order.
        let mut broadcast_triggers: Vec<(CardId, Effect, Option<crate::card::Predicate>, Target, CardId)> =
            Vec::new();
        let computed = self.compute_battlefield();
        let computed_kw = |id: CardId| -> &[Keyword] {
            computed
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.keywords.as_slice())
                .unwrap_or(&[])
        };

        for atk in attacks {
            let id = atk.attacker;
            // Filter by *controller*, not *owner* — a creature you've
            // stolen (Threaten / Mind Control) attacks for you, even
            // though its `owner` field still points at the original
            // player. Captured at
            // `debug/deadlock-t9-1777413906-987970800.json` where
            // bot 0 controlled a Cosmogoyf with `owner=1, controller=0`
            // and `declare_attackers` rejected the cast with
            // `CardNotOnBattlefield(93)`.
            let card = self
                .battlefield
                .iter_mut()
                .find(|c| c.id == id && c.controller == p)
                .ok_or(GameError::CardNotOnBattlefield(id))?;

            if !card.can_attack() {
                if card.tapped {
                    return Err(GameError::CardIsTapped(id));
                }
                return Err(GameError::SummoningSickness(id));
            }
            if !computed_kw(id).contains(&Keyword::Vigilance) {
                card.tapped = true;
            }
            self.attacking.push(atk);
            events.push(GameEvent::AttackerDeclared(id));
            // Self-source attack triggers (the attacker's own
            // "Whenever this creature attacks, ..." abilities).
            for t in &card.definition.triggered_abilities {
                if t.event.kind == EventKind::Attacks {
                    triggers.push((id, t.effect.clone(), p));
                }
            }
            // Broadcast the AttackerDeclared event to other permanents
            // controlled by the attacker's controller — this lets
            // enchantments / artifacts with "Whenever you attack /
            // whenever a creature you control attacks" triggers fire
            // (Sparring Regimen's per-attacker pump). The trigger's
            // body picks the attacker via `Selector::Target(0)`, so
            // we pre-bind the just-declared attacker as the target.
            // The `AnotherOfYours` scope check filters out the
            // attacker's own self-source triggers, which we already
            // added above.
            let attacker_id = id;
            for c in &self.battlefield {
                if c.controller != p || c.id == attacker_id {
                    continue;
                }
                let cid = c.id;
                for t in &c.definition.triggered_abilities {
                    if t.event.kind == EventKind::Attacks
                        && matches!(
                            t.event.scope,
                            EventScope::AnotherOfYours
                                | EventScope::YourControl
                                | EventScope::AnyPlayer
                        )
                    {
                        broadcast_triggers.push((
                            cid,
                            t.effect.clone(),
                            t.event.filter.clone(),
                            Target::Permanent(attacker_id),
                            attacker_id,
                        ));
                    }
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
        // Second pass: evaluate broadcast trigger filters now that
        // every attacker is in `self.attacking`. A passing predicate
        // (or no predicate) lets the trigger push; a failing one
        // drops it silently. Push order is per-attacker-then-source,
        // matching the original iteration order so visible behavior
        // is unchanged for unfiltered triggers.
        for (src, eff, filter, tgt, attacker_id) in broadcast_triggers {
            if let Some(ref pred) = filter {
                let ctx = crate::game::effects::EffectContext::for_trigger(
                    src,
                    p,
                    Some(tgt.clone()),
                    0,
                );
                if !self.evaluate_predicate(pred, &ctx) {
                    continue;
                }
            }
            self.stack.push(StackItem::Trigger {
                source: src,
                controller: p,
                effect: Box::new(eff),
                target: Some(tgt),
                mode: None,
                x_value: 0,
                converged_value: 0,
                // Sparring Regimen-style "whenever you attack, +1/+1
                // on each attacking creature" — subject is the
                // attacker the trigger fires off.
                subject: Some(crate::game::effects::EntityRef::Permanent(attacker_id)),
            });
        }
        for (source, effect, controller) in triggers {
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
                // SelfSource Attacks trigger — subject is the
                // attacking creature itself.
                subject: Some(crate::game::effects::EntityRef::Permanent(source)),
            });
        }
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

        let computed = self.compute_battlefield();
        let cp_of = |id: CardId| computed.iter().find(|c| c.id == id);
        let kws_of = |id: CardId| -> &[Keyword] {
            computed
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.keywords.as_slice())
                .unwrap_or(&[])
        };

        // Validate ALL assignments before mutating any state. Each blocker's
        // controller must equal the defender of the attacker it's blocking.
        for &(blocker_id, attacker_id) in &assignments {
            let atk = self
                .attack_for(attacker_id)
                .ok_or(GameError::CardNotOnBattlefield(attacker_id))?;
            let defender_idx = self
                .defender_for(atk.target)
                .ok_or(GameError::CardNotOnBattlefield(attacker_id))?;

            let blocker = self
                .battlefield
                .iter()
                .find(|c| c.id == blocker_id)
                .ok_or(GameError::CardNotOnBattlefield(blocker_id))?;

            if blocker.controller != defender_idx {
                return Err(GameError::BlockerWrongDefender { blocker: blocker_id });
            }

            if !blocker.can_block() {
                return Err(GameError::CannotBlock(blocker_id));
            }

            // `Keyword::CantBlock` is enforced from the *computed* keyword
            // set so transient grants (e.g. SOS Duel Tactics's "this
            // creature can't block this turn", Postmortem Professor's
            // static restriction) take effect immediately.
            if kws_of(blocker_id).contains(&Keyword::CantBlock) {
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
        for atk in &self.attacking {
            let has_menace = kws_of(atk.attacker).contains(&Keyword::Menace);
            if has_menace {
                let blocker_count = assignments
                    .iter()
                    .filter(|(_, aid)| *aid == atk.attacker)
                    .count();
                if blocker_count == 1 {
                    return Err(GameError::MenaceRequiresTwoBlockers(atk.attacker));
                }
            }
        }

        // All valid — apply (merge into existing block_map so multiple
        // defenders can submit independently in multiplayer).
        self.blockers_declared = true;
        let mut events = vec![];
        for (blocker_id, attacker_id) in assignments {
            self.block_map.insert(blocker_id, attacker_id);
            events.push(GameEvent::BlockerDeclared {
                blocker: blocker_id,
                attacker: attacker_id,
            });
        }
        self.give_priority_to_active();
        Ok(events)
    }

    // ── Combat resolution ─────────────────────────────────────────────────────

    pub(crate) fn has_first_strikers(&self) -> bool {
        let computed = self.compute_battlefield();
        let kws_of = |id: CardId| -> &[Keyword] {
            computed
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.keywords.as_slice())
                .unwrap_or(&[])
        };
        self.attacking.iter().any(|atk| {
            kws_of(atk.attacker).contains(&Keyword::FirstStrike)
                || kws_of(atk.attacker).contains(&Keyword::DoubleStrike)
        }) || self.block_map.keys().any(|&id| {
            kws_of(id).contains(&Keyword::FirstStrike)
                || kws_of(id).contains(&Keyword::DoubleStrike)
        })
    }

    pub(crate) fn resolve_first_strike_damage(&mut self) -> Result<Vec<GameEvent>, GameError> {
        let computed = self.compute_battlefield();
        let mut events = self.resolve_combat_damage_with_filter(
            &computed,
            |kws: &[Keyword]| {
                kws.contains(&Keyword::FirstStrike) || kws.contains(&Keyword::DoubleStrike)
            },
            |_kws: &[Keyword]| false,
        )?;
        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);
        events.push(GameEvent::FirstStrikeDamageResolved);
        Ok(events)
    }

    pub(crate) fn resolve_combat(&mut self) -> Result<Vec<GameEvent>, GameError> {
        let computed = self.compute_battlefield();
        let mut events = self.resolve_combat_damage_with_filter(
            &computed,
            |kws: &[Keyword]| {
                !kws.contains(&Keyword::FirstStrike) || kws.contains(&Keyword::DoubleStrike)
            },
            |_kws: &[Keyword]| true,
        )?;

        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);

        self.attacking.clear();
        self.block_map.clear();
        self.blockers_declared = false;

        events.push(GameEvent::CombatResolved);
        Ok(events)
    }

    /// Core combat damage resolver. Each attacker has its own defending
    /// player or planeswalker (`Attack::target`); damage routing is
    /// per-attacker.
    fn resolve_combat_damage_with_filter(
        &mut self,
        computed: &[ComputedPermanent],
        attacker_filter: impl Fn(&[Keyword]) -> bool,
        _blocker_filter: impl Fn(&[Keyword]) -> bool,
    ) -> Result<Vec<GameEvent>, GameError> {
        let mut events = vec![];

        let computed_of =
            |id: CardId| -> Option<&ComputedPermanent> { computed.iter().find(|c| c.id == id) };

        let attacker_infos: Vec<AttackerInfo> = self
            .attacking
            .iter()
            .filter_map(|atk| {
                let cp = computed_of(atk.attacker)?;
                let defender_player = self.defender_for(atk.target)?;
                let kws = &cp.keywords;
                Some(AttackerInfo {
                    id: cp.id,
                    target: atk.target,
                    defender_player,
                    power: cp.power,
                    has_trample: kws.contains(&Keyword::Trample),
                    has_lifelink: kws.contains(&Keyword::Lifelink),
                    has_deathtouch: kws.contains(&Keyword::Deathtouch),
                    has_first_strike: kws.contains(&Keyword::FirstStrike),
                    has_double_strike: kws.contains(&Keyword::DoubleStrike),
                    has_infect: kws.contains(&Keyword::Infect),
                    has_wither: kws.contains(&Keyword::Wither),
                    should_deal: attacker_filter(kws),
                })
            })
            .collect();

        for atk in &attacker_infos {
            if !atk.should_deal {
                continue;
            }
            // CR 615 prevention shield (Owlin Shieldmage, Holy Day,
            // Ethereal Haze): if any active "prevent all combat damage
            // this turn" effect is in play, every attacker's damage is
            // replaced with 0. We short-circuit here rather than zeroing
            // `atk.power` so that lifelink, infect, and trample-trigger
            // riders that key off "damage dealt" never fire — matching
            // the printed "prevent" semantics (no damage event at all).
            if self.combat_damage_prevented_this_turn {
                continue;
            }

            let blocker_ids: Vec<CardId> = self
                .block_map
                .iter()
                .filter(|(_, aid)| **aid == atk.id)
                .map(|(&bid, _)| bid)
                .collect();

            if blocker_ids.is_empty() {
                let amount = atk.power.max(0) as u32;
                if amount > 0 {
                    self.deal_combat_damage_to_target(atk, amount, &mut events);
                    if atk.has_lifelink {
                        let a = self.active_player_idx;
                        self.players[a].life += atk.power;
                        self.players[a].life_gained_this_turn =
                            self.players[a].life_gained_this_turn.saturating_add(amount);
                        events.push(GameEvent::LifeGained { player: a, amount });
                    }
                }
            } else {
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

                if atk.has_trample && remaining_atk_damage > 0 {
                    let amount = remaining_atk_damage as u32;
                    lifelink_dealt += remaining_atk_damage;
                    self.deal_combat_damage_to_target(atk, amount, &mut events);
                }

                if atk.has_lifelink && lifelink_dealt > 0 {
                    let a = self.active_player_idx;
                    let amt = lifelink_dealt as u32;
                    self.players[a].life += lifelink_dealt;
                    self.players[a].life_gained_this_turn =
                        self.players[a].life_gained_this_turn.saturating_add(amt);
                    events.push(GameEvent::LifeGained { player: a, amount: amt });
                }

                let blocker_damage_to_attacker: i32 = blocker_ids
                    .iter()
                    .filter_map(|&bid| computed_of(bid))
                    .filter(|bc| {
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
                    }

                    // Blocker lifelink — gained by each blocker's controller
                    // (different blockers can have different controllers in
                    // multiplayer).
                    let mut lifelink_by_controller: std::collections::HashMap<usize, i32> =
                        std::collections::HashMap::new();
                    for &bid in &blocker_ids {
                        let Some(bc) = computed_of(bid) else { continue };
                        if !bc.keywords.contains(&Keyword::Lifelink) {
                            continue;
                        }
                        let controller = self
                            .battlefield
                            .iter()
                            .find(|c| c.id == bid)
                            .map(|c| c.controller)
                            .unwrap_or(atk.defender_player);
                        *lifelink_by_controller.entry(controller).or_insert(0) += bc.power;
                    }
                    for (player, gained) in lifelink_by_controller {
                        if gained > 0 {
                            let amt = gained as u32;
                            self.players[player].life += gained;
                            self.players[player].life_gained_this_turn =
                                self.players[player].life_gained_this_turn.saturating_add(amt);
                            events.push(GameEvent::LifeGained { player, amount: amt });
                        }
                    }
                }
            }
        }

        Ok(events)
    }

    /// Apply `amount` damage from `atk` to its declared attack target. For
    /// player targets this is life loss (or poison if Infect); for
    /// planeswalker targets this is loyalty loss. Also fires
    /// `DealsCombatDamageToPlayer` triggers when a player is hit.
    fn deal_combat_damage_to_target(
        &mut self,
        atk: &AttackerInfo,
        amount: u32,
        events: &mut Vec<GameEvent>,
    ) {
        match atk.target {
            AttackTarget::Player(p) => {
                if atk.has_infect {
                    self.players[p].poison_counters += amount;
                    events.push(GameEvent::PoisonAdded {
                        player: p,
                        amount,
                    });
                } else {
                    self.players[p].life -= amount as i32;
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: Some(p),
                        to_card: None,
                    });
                    events.push(GameEvent::LifeLost {
                        player: p,
                        amount,
                    });
                }
                self.fire_combat_damage_to_player_triggers(atk.id, p);
            }
            AttackTarget::Planeswalker(pw_id) => {
                if let Some(pw) = self.battlefield_find_mut(pw_id) {
                    let current = pw.counter_count(crate::card::CounterType::Loyalty);
                    let new_loyalty = current.saturating_sub(amount);
                    pw.counters
                        .insert(crate::card::CounterType::Loyalty, new_loyalty);
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: None,
                        to_card: Some(pw_id),
                    });
                    events.push(GameEvent::LoyaltyChanged {
                        card_id: pw_id,
                        new_loyalty: new_loyalty as i32,
                    });
                }
            }
        }
    }

    /// Push triggered abilities of `source` whose event spec is
    /// `DealsCombatDamageToPlayer` onto the stack, with `damaged_player`
    /// stored as the trigger's target so the effect can refer to "that
    /// player" via `PlayerRef::Target(0)`.
    ///
    /// Two trigger families fire here:
    /// 1. `SelfSource` / `AnyPlayer` triggers on the **attacker itself**
    ///    (e.g. Snooping Page's "deals combat damage → draw + lose 1").
    /// 2. `FromYourGraveyard` triggers on cards in the **attacker's
    ///    controller's graveyard**, with the implicit "any creature you
    ///    control" filter (e.g. Killian's Confidence's may-pay-{W/B}-to-
    ///    return-self trigger). The trigger source is the graveyard
    ///    card itself; the effect typically references it via
    ///    `Selector::This`.
    fn fire_combat_damage_to_player_triggers(&mut self, source: CardId, damaged_player: usize) {
        // Family 1: triggers ON the attacker itself.
        let attacker_controller = self
            .battlefield
            .iter()
            .find(|c| c.id == source)
            .map(|c| c.controller);
        let mut triggers: Vec<(CardId, Effect, usize)> = self
            .battlefield
            .iter()
            .find(|c| c.id == source)
            .map(|c| {
                c.definition
                    .triggered_abilities
                    .iter()
                    .filter(|t| {
                        t.event.kind == EventKind::DealsCombatDamageToPlayer
                            && matches!(
                                t.event.scope,
                                crate::effect::EventScope::SelfSource
                                    | crate::effect::EventScope::AnyPlayer
                            )
                    })
                    .map(|t| (c.id, t.effect.clone(), c.controller))
                    .collect()
            })
            .unwrap_or_default();

        // Family 2: `FromYourGraveyard`-scoped triggers in the attacker's
        // controller's graveyard. These cards self-reference ("return this
        // card") and fire on any creature their owner controls dealing
        // combat damage to a player.
        if let Some(controller_idx) = attacker_controller {
            let gy_triggers: Vec<(CardId, Effect, usize)> = self.players[controller_idx]
                .graveyard
                .iter()
                .flat_map(|c| {
                    c.definition
                        .triggered_abilities
                        .iter()
                        .filter(|t| {
                            t.event.kind == EventKind::DealsCombatDamageToPlayer
                                && matches!(
                                    t.event.scope,
                                    crate::effect::EventScope::FromYourGraveyard
                                )
                        })
                        .map(|t| (c.id, t.effect.clone(), controller_idx))
                        .collect::<Vec<_>>()
                })
                .collect();
            triggers.extend(gy_triggers);
        }

        for (src_id, effect, controller) in triggers {
            self.stack.push(StackItem::Trigger {
                source: src_id,
                controller,
                effect: Box::new(effect),
                target: Some(Target::Player(damaged_player)),
                mode: None,
                x_value: 0,
                converged_value: 0,
                // Combat-damage-to-player trigger — subject is the
                // attacker that dealt the damage.
                subject: Some(crate::game::effects::EntityRef::Permanent(src_id)),
            });
        }
    }
}

/// Resolution-time snapshot of one attacker's combat-relevant data. Captures
/// the attacker's target so damage routes correctly even if the target moves
/// during the loop.
struct AttackerInfo {
    id: CardId,
    target: AttackTarget,
    defender_player: usize,
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
