use super::*;
use crate::card::Keyword;
use crate::effect::{Effect, EventKind, Selector, Value};
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

        // Validate every attack target up-front. The defender must be an
        // *opponent* — not self, not a teammate. `same_team` returns true
        // for `a == b`, so this single check rules out both cases. In
        // 1v1 / FFA it behaves identically to the old `target != active`
        // check; in 2HG / team formats it correctly rejects targeting a
        // teammate's life total or planeswalker.
        for atk in &attacks {
            match atk.target {
                AttackTarget::Player(target_player) => {
                    if target_player >= self.players.len()
                        || self.same_team(self.active_player_idx, target_player)
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
                        || self.same_team(self.active_player_idx, pw.controller)
                        || !self.players[pw.controller].is_alive()
                    {
                        return Err(GameError::InvalidPlaneswalkerAttackTarget(pw_id));
                    }
                }
            }
        }

        let mut events = vec![];
        // Per CR 506.5, the Attacks trigger filter must be evaluated
        // post-batch, so we carry the optional filter alongside each
        // queued trigger.
        let mut triggers: Vec<(
            CardId,
            Effect,
            usize,
            Option<crate::effect::Predicate>,
        )> = vec![];
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

            // Creature-ness is read from the computed view so a crewed
            // Vehicle (CR 702.122 — animated to an artifact creature via a
            // layer-4 AddCardType) can attack, while an uncrewed one can't.
            // Defender / Haste are likewise read post-layer so granted
            // variants are honored.
            let is_creature_now = computed
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.card_types.contains(&crate::card::CardType::Creature))
                .unwrap_or_else(|| card.definition.is_creature());
            let kws = computed_kw(id);
            let can_attack = is_creature_now
                && !card.tapped
                && !kws.contains(&Keyword::Defender)
                && (!card.summoning_sick || kws.contains(&Keyword::Haste));
            if !can_attack {
                if card.tapped {
                    return Err(GameError::CardIsTapped(id));
                }
                return Err(GameError::SummoningSickness(id));
            }
            if !computed_kw(id).contains(&Keyword::Vigilance) {
                card.tapped = true;
            }
            // CR 702.83 — Exert. We auto-exert any attacking creature with
            // the keyword (the "you may" choice is collapsed; the AutoDecider
            // would have no policy and a real exert is almost always taken for
            // its bonus). The creature won't untap next untap step. Its exert
            // bonus rides its normal SelfSource Attacks trigger.
            if computed_kw(id).contains(&Keyword::Exert) {
                card.skip_next_untap = true;
            }
            self.attacking.push(atk);
            events.push(GameEvent::AttackerDeclared(id));
            // Walk printed Attacks triggers + any transient granted
            // Attacks triggers (Root Manipulation's "gain 1 life when
            // this attacks" grant lands in `granted_triggers_eot`).
            let granted: &[crate::card::TriggeredAbility] = self
                .granted_triggers_eot
                .get(&id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            for t in card.definition.triggered_abilities.iter().chain(granted) {
                // Only SelfSource Attacks triggers are hardcoded here.
                // YourControl-scoped Attacks triggers (Exalted via
                // `Predicate::AttackingAlone`, Battle Banner, …) are
                // routed through the unified `dispatch_triggers_for_events`
                // path off the `AttackerDeclared` event — pushing them
                // here too would double-fire the ability.
                if t.event.kind == EventKind::Attacks
                    && t.event.scope == crate::effect::EventScope::SelfSource
                {
                    // Capture the trigger's optional filter so we can
                    // re-evaluate it AFTER the entire attacker batch is
                    // declared (CR 506.5 "attacking alone" semantics
                    // require the post-batch view).
                    triggers.push((id, t.effect.clone(), p, t.event.filter.clone()));
                }
            }
            // Annihilator N — CR 702.85a: "Whenever this creature attacks,
            // defending player sacrifices N permanents." Translate the
            // keyword to an Attacks-trigger that fires
            // `Effect::Sacrifice { who: defender, count: N, filter: Any }`.
            // The defender comes from `atk.target`; for a planeswalker
            // attack, that's the planeswalker's controller (CR 506.4a).
            let annihilator_n = computed_kw(id).iter().find_map(|kw| {
                if let Keyword::Annihilator(n) = kw {
                    Some(*n)
                } else {
                    None
                }
            });
            if let Some(n) = annihilator_n
                && let Some(defender) = self.defender_for(atk.target)
            {
                let sac_effect = Effect::Sacrifice {
                    who: Selector::Player(crate::effect::PlayerRef::Seat(defender)),
                    count: Value::Const(n as i32),
                    filter: crate::card::SelectionRequirement::Permanent,
                };
                triggers.push((id, sac_effect, p, None));
            }
        }
        // YourControl-scoped Attacks triggers (e.g. Battle Banner,
        // Sparring Regimen) are NOT walked here — the unified
        // `dispatch_triggers_for_events` path in `mod.rs` picks them up
        // off the `AttackerDeclared` event(s) and routes them through the
        // same trigger pipeline. Walking them here additionally would
        // double-fire the trigger (one push from combat.rs + one from
        // the dispatcher). The hardcoded `is_event_hardcoded` check only
        // marks SelfSource Attacks as already handled.

        for (source, effect, controller, filter) in triggers {
            // CR 603.2 + CR 506.5: evaluate the trigger's optional filter
            // predicate at fire-time, which for Attacks is "after the
            // entire declare attackers step batch is resolved".
            if let Some(predicate) = filter {
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
                if !self.evaluate_predicate(&predicate, &ctx) {
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
                trigger_source: None,
                mana_spent: 0,
                event_amount: 0,
                intervening_if: None,
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

            // CR 509.1a: any creature controlled by the defending player
            // (or, in team formats, a teammate of the defending player)
            // may block. In 1v1 / FFA `same_team(a, b)` collapses to
            // `a == b`, so this preserves the historical behavior.
            if !self.same_team(blocker.controller, defender_idx) {
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
            let atk_colors = cp_of(attacker_id).map(|c| c.colors.as_slice()).unwrap_or(&[]);
            if !super::can_block_attacker_computed(
                blocker,
                attacker,
                blocker_cp,
                kws_of(attacker_id),
                atk_colors,
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

        // CR 509.1c — "must be blocked if able" (Lure / Academic Dispute).
        // If such an attacker is left unblocked while the defender controls
        // an idle creature that could legally block it, reject the
        // declaration. Considers the merged block set (already-declared
        // blocks plus this batch) so independent multiplayer submissions
        // compose. Single-requirement model; full CR maximization across
        // multiple simultaneous requirements is approximated.
        for atk in &self.attacking {
            if !kws_of(atk.attacker).contains(&Keyword::MustBeBlocked) {
                continue;
            }
            let already = self.block_map.values().any(|&aid| aid == atk.attacker);
            let in_batch = assignments.iter().any(|(_, aid)| *aid == atk.attacker);
            if already || in_batch {
                continue;
            }
            let Some(defender_idx) = self.defender_for(atk.target) else { continue };
            let attacker = match self.battlefield_find(atk.attacker) {
                Some(a) => a,
                None => continue,
            };
            let atk_colors = cp_of(atk.attacker).map(|c| c.colors.as_slice()).unwrap_or(&[]);
            let idle_able_blocker = self.battlefield.iter().any(|b| {
                b.definition.is_creature()
                    && self.same_team(b.controller, defender_idx)
                    && b.can_block()
                    && !kws_of(b.id).contains(&Keyword::CantBlock)
                    && !self.block_map.contains_key(&b.id)
                    && !assignments.iter().any(|(bid, _)| *bid == b.id)
                    && cp_of(b.id).is_some_and(|bcp| {
                        super::can_block_attacker_computed(
                            b, attacker, bcp, kws_of(atk.attacker), atk_colors,
                        )
                    })
            });
            if idle_able_blocker {
                return Err(GameError::MustBeBlockedIfAble(atk.attacker));
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
        // CR 509.3g — emit `AttackerWentUnblocked` for each attacker
        // with no blockers assigned. Trigger source is the unblocked
        // attacker; consumers can read it via `Selector::TriggerSource`.
        // The block_map maps blocker → attacker, so an attacker is
        // unblocked iff no entry has it as a value.
        for atk in &self.attacking {
            let blocked = self.block_map.values().any(|&aid| aid == atk.attacker);
            if !blocked {
                events.push(GameEvent::AttackerWentUnblocked { attacker: atk.attacker });
            }
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
        // CR 510.4: in the first-strike combat damage step, only creatures
        // with first strike or double strike deal combat damage. The same
        // gate applies to attackers (who deals?) and blockers (who strikes
        // back at the attacker?).
        let fs_or_ds = |kws: &[Keyword]| {
            kws.contains(&Keyword::FirstStrike) || kws.contains(&Keyword::DoubleStrike)
        };
        let mut events = self.resolve_combat_damage_with_filter(&computed, fs_or_ds, fs_or_ds)?;
        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);
        events.push(GameEvent::FirstStrikeDamageResolved);
        Ok(events)
    }

    pub(crate) fn resolve_combat(&mut self) -> Result<Vec<GameEvent>, GameError> {
        let computed = self.compute_battlefield();
        // CR 510.5: in the regular combat damage step, every attacking and
        // blocking creature that didn't deal damage in the first-strike step
        // deals damage now — i.e. anyone without first strike, plus double
        // strikers (who strike in both steps).
        let regular_or_ds = |kws: &[Keyword]| {
            !kws.contains(&Keyword::FirstStrike) || kws.contains(&Keyword::DoubleStrike)
        };
        let mut events =
            self.resolve_combat_damage_with_filter(&computed, regular_or_ds, regular_or_ds)?;

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
        blocker_filter: impl Fn(&[Keyword]) -> bool,
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
                    has_infect: kws.contains(&Keyword::Infect),
                    has_wither: kws.contains(&Keyword::Wither),
                    should_deal: attacker_filter(kws),
                })
            })
            .collect();

        // CR 615.1 — "Prevent all combat damage this turn" (Owlin
        // Shieldmage, Holy Day, Constant Mists). When the global flag is
        // set, every combat damage assignment yields 0; lifelink scales
        // off actual damage dealt (CR 702.15a), so prevention zeros
        // lifelink life-gain as well. Triggers that would fire off
        // "deals combat damage to a player" never see a damage event.
        let prevent_combat_damage = self.prevent_combat_damage_this_turn;

        for atk in &attacker_infos {
            if !atk.should_deal {
                continue;
            }

            // CR 510.1c: attacker's controller chooses damage assignment
            // order against multiple blockers. The engine doesn't surface
            // that choice through a Decision yet, but we MUST iterate in
            // a deterministic order so replay/snapshot diverge isn't
            // gated on randomized HashMap iteration order. Sort by
            // CardId (declaration order proxy — ids are handed out
            // monotonically by `next_id`).
            let mut blocker_ids: Vec<CardId> = self
                .block_map
                .iter()
                .filter(|(_, aid)| **aid == atk.id)
                .map(|(&bid, _)| bid)
                .collect();
            blocker_ids.sort_by_key(|id| id.0);

            if blocker_ids.is_empty() {
                let amount = if prevent_combat_damage {
                    0
                } else {
                    atk.power.max(0) as u32
                };
                if amount > 0 {
                    self.deal_combat_damage_to_target(atk, amount, &mut events);
                    if atk.has_lifelink {
                        let a = self.active_player_idx;
                        self.adjust_life(a, atk.power);
                        events.push(GameEvent::LifeGained { player: a, amount });
                    }
                }
            } else {
                let mut remaining_atk_damage =
                    if prevent_combat_damage { 0 } else { atk.power };
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
                        if atk.has_deathtouch && assign > 0 {
                            blocker.dealt_deathtouch_damage = true;
                        }
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
                    self.adjust_life(a, lifelink_dealt);
                    events.push(GameEvent::LifeGained { player: a, amount: amt });
                }

                // Only blockers whose own keywords say they deal damage in
                // this step strike back at the attacker. Per CR 510.4/510.5
                // the attacker's keywords don't gate the blocker's strike
                // step — a regular blocker must wait for the regular step
                // even if the attacker has first strike.
                let dealing_blocker_ids: Vec<CardId> = blocker_ids
                    .iter()
                    .copied()
                    .filter(|&bid| computed_of(bid)
                        .is_some_and(|bc| blocker_filter(&bc.keywords)))
                    .collect();

                let blocker_damage_to_attacker: i32 = if prevent_combat_damage {
                    0
                } else {
                    dealing_blocker_ids
                        .iter()
                        .filter_map(|&bid| computed_of(bid))
                        .map(|c| c.power)
                        .sum()
                };

                if blocker_damage_to_attacker > 0 {
                    let any_deathtouch_blocker = dealing_blocker_ids
                        .iter()
                        .filter_map(|&bid| computed_of(bid))
                        .any(|c| c.keywords.contains(&Keyword::Deathtouch));
                    let any_infect_blocker = dealing_blocker_ids
                        .iter()
                        .filter_map(|&bid| computed_of(bid))
                        .any(|c| {
                            c.keywords.contains(&Keyword::Infect)
                                || c.keywords.contains(&Keyword::Wither)
                        });
                    let _attacker_toughness = computed_of(atk.id).map(|c| c.toughness).unwrap_or(0);
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
                            attacker.damage += blocker_damage_to_attacker.max(0) as u32;
                            attacker.dealt_deathtouch_damage = true;
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
                    // multiplayer). Only blockers actually striking back in
                    // this step gain life from it.
                    let mut lifelink_by_controller: std::collections::HashMap<usize, i32> =
                        std::collections::HashMap::new();
                    for &bid in &dealing_blocker_ids {
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
                    // Sort by seat for deterministic event ordering;
                    // life-gain math is commutative but the event log
                    // shouldn't shuffle across replays.
                    let mut lifelink_entries: Vec<(usize, i32)> =
                        lifelink_by_controller.into_iter().collect();
                    lifelink_entries.sort_by_key(|(p, _)| *p);
                    for (player, gained) in lifelink_entries {
                        if gained > 0 {
                            let amt = gained as u32;
                            self.adjust_life(player, gained);
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
                    self.adjust_life(p, -(amount as i32));
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
                // Phase M: bump the 21-commander-damage tally when the
                // attacker is a Commander. Both Infect and regular
                // damage paths credit here — CR 704.5v doesn't restrict
                // by damage type. The SBA in `check_state_based_actions`
                // reads this table and eliminates the player when any
                // single (victim, commander) entry crosses 21.
                if self.is_commander(atk.id) {
                    self.record_commander_damage(p, atk.id, amount);
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
    /// Phase 1 walks the battlefield for the attacker's own
    /// `SelfSource` / `AnyPlayer` triggers (the printed
    /// "whenever this creature deals combat damage" pattern).
    ///
    /// Phase 2 walks every player's graveyard for `FromYourGraveyard`
    /// triggers whose controller (the gy owner) matches the source's
    /// controller — the "whenever your creatures deal combat damage,
    /// return this card from your graveyard" pattern used by Killian's
    /// Confidence and friends. The trigger source is bound to the
    /// graveyard card itself so a `Move(SelfSource → Hand)` body
    /// returns the right card.
    fn fire_combat_damage_to_player_triggers(&mut self, source: CardId, damaged_player: usize) {
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

        // Phase 1.5: walk all battlefield permanents for `YourControl`-scope
        // combat-damage triggers. This handles "whenever a creature you
        // control deals combat damage to a player" listeners (e.g.,
        // Quandrix Echocrasher b171). The listener's controller must
        // match the attacker's controller.
        if let Some(atk_ctrl) = attacker_controller {
            for c in &self.battlefield {
                if c.id == source || c.controller != atk_ctrl {
                    continue;
                }
                for t in &c.definition.triggered_abilities {
                    if t.event.kind == EventKind::DealsCombatDamageToPlayer
                        && matches!(t.event.scope, crate::effect::EventScope::YourControl)
                    {
                        triggers.push((c.id, t.effect.clone(), c.controller));
                    }
                }
            }
        }

        // Phase 2: walk every player's graveyard for `FromYourGraveyard`
        // triggers. Only fire if the attacker is controlled by the gy
        // owner (the printed "creatures you control" filter on the
        // attacker side).
        if let Some(atk_controller) = attacker_controller {
            for player in &self.players {
                if player.id.0 != atk_controller {
                    continue;
                }
                for gy_card in &player.graveyard {
                    for t in &gy_card.definition.triggered_abilities {
                        if t.event.kind == EventKind::DealsCombatDamageToPlayer
                            && matches!(
                                t.event.scope,
                                crate::effect::EventScope::FromYourGraveyard
                            )
                        {
                            triggers.push((gy_card.id, t.effect.clone(), gy_card.owner));
                        }
                    }
                }
            }
        }

        for (trig_source, effect, controller) in triggers {
            self.stack.push(StackItem::Trigger {
                source: trig_source,
                controller,
                effect: Box::new(effect),
                target: Some(Target::Player(damaged_player)),
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
    has_infect: bool,
    has_wither: bool,
    should_deal: bool,
}
