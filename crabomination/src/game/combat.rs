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

        // CR 508.0 — "attacks only alone" (Master of Cruelties). If any
        // declared attacker carries AttacksAlone, the batch must be a
        // single attacker. Read from the computed keyword set so granted
        // variants count.
        if attacks.len() > 1 {
            let computed_pre = self.compute_battlefield();
            if attacks.iter().any(|atk| {
                computed_pre
                    .iter()
                    .find(|c| c.id == atk.attacker)
                    .is_some_and(|c| c.keywords.contains(&Keyword::AttacksAlone))
            }) {
                return Err(GameError::CannotAttack(attacks[0].attacker));
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

        // CR 508.1d — "attacks each combat if able" (Juggernaut, goaded
        // creatures). Any creature the active player controls that carries
        // MustAttack and *can* legally attack (untapped, not sick / has
        // Haste, not Defender / CantAttack) must be in the declared batch
        // while at least one opponent is in range. Reject an incomplete
        // declaration so the requirement is honored.
        let has_legal_target = self
            .players
            .iter()
            .enumerate()
            .any(|(i, pl)| !self.same_team(p, i) && pl.is_alive());
        if has_legal_target {
            for c in &self.battlefield {
                // A creature must be declared if it carries MustAttack
                // (Juggernaut) or is goaded (CR 701.38 — "attacks each
                // combat if able").
                let must = computed_kw(c.id).contains(&Keyword::MustAttack)
                    || !c.goaded_by.is_empty();
                if c.controller != p || !must {
                    continue;
                }
                let kws = computed_kw(c.id);
                let able = c.definition.is_creature()
                    && !c.tapped
                    && !kws.contains(&Keyword::Defender)
                    && !kws.contains(&Keyword::CantAttack)
                    && (!c.summoning_sick || kws.contains(&Keyword::Haste));
                if able && !attacks.iter().any(|atk| atk.attacker == c.id) {
                    return Err(GameError::CannotAttack(c.id));
                }
            }
        }

        for atk in attacks {
            let id = atk.attacker;
            // "Can't attack unless defending player controls a [filter]"
            // (Dandân). Resolved before the mutable attacker binding to keep
            // the borrows disjoint: require ≥1 matching permanent under the
            // attack's defending player's control.
            if let Some(req) = computed_kw(id).iter().find_map(|kw| match kw {
                Keyword::CanAttackOnlyIfDefenderControls(r) => Some(r.clone()),
                _ => None,
            }) {
                let defender = self.defender_for(atk.target);
                let satisfied = defender.is_some_and(|d| {
                    self.battlefield
                        .iter()
                        .any(|c| c.controller == d && self.evaluate_requirement_on_card(&req, c, d))
                });
                if !satisfied {
                    return Err(GameError::CannotAttack(id));
                }
            }
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
                && !kws.contains(&Keyword::CantAttack)
                && (!card.summoning_sick || kws.contains(&Keyword::Haste));
            if !can_attack {
                if card.tapped {
                    return Err(GameError::CardIsTapped(id));
                }
                if kws.contains(&Keyword::Defender) || kws.contains(&Keyword::CantAttack) {
                    return Err(GameError::CannotAttack(id));
                }
                return Err(GameError::SummoningSickness(id));
            }
            if !computed_kw(id).contains(&Keyword::Vigilance) {
                card.tapped = true;
                // CR 508.1f — attacking taps the creature; surface a
                // "becomes tapped" event so Tapped triggers fire (Magda).
                events.push(GameEvent::PermanentTapped { card_id: id });
            }
            // CR 702.83 — Exert. We auto-exert any attacking creature with
            // the keyword (the "you may" choice is collapsed; the AutoDecider
            // would have no policy and a real exert is almost always taken for
            // its bonus). The creature won't untap next untap step. Its exert
            // bonus rides its normal SelfSource Attacks trigger.
            if computed_kw(id).contains(&Keyword::Exert) {
                card.skip_next_untap = true;
            }
            // CR 702.142 — record that this creature attacked (gates Boast).
            card.attacked_this_turn = true;
            self.attacking.push(atk);
            // Raid (CR 702.108 ability word): the controller attacked this turn.
            self.players[p].attacked_this_turn = true;
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
            // ControllerAttackedByOpponent (CR 508.1g listeners): permanents
            // the defending player controls that fire "when a creature an
            // opponent attacks me" — Coveted Jewel's control-flip. The
            // attacking creature's controller is bound to the trigger's
            // target slot ("that creature's controller").
            if let Some(defender) = self.defender_for(atk.target)
                && defender != p
            {
                let listeners: Vec<(CardId, Effect)> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == defender)
                    .flat_map(|c| {
                        c.definition.triggered_abilities.iter().filter_map(move |t| {
                            (t.event.kind == EventKind::Attacks
                                && t.event.scope
                                    == crate::effect::EventScope::ControllerAttackedByOpponent)
                                .then(|| (c.id, t.effect.clone()))
                        })
                    })
                    .collect();
                for (src, effect) in listeners {
                    self.stack.push(StackItem::Trigger {
                        source: src,
                        controller: defender,
                        effect: Box::new(effect),
                        target: Some(Target::Player(p)),
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
                    kicked: false,
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

            // Landwalk (CR 702.15): the attacker can't be blocked while the
            // defending player controls a land of the named type. Needs
            // game state (the defender's lands), so it lives here rather
            // than in the pure two-creature `can_block_attacker_computed`.
            for kw in kws_of(attacker_id) {
                if let Keyword::Landwalk(lt) = kw
                    && self.defender_controls_land_type(defender_idx, lt)
                {
                    return Err(GameError::CannotBlock(blocker_id));
                }
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

        // CR 509.1c — true Lure ("all creatures able to block this do so").
        // Every idle defender creature that *can* legally block such an
        // attacker must be assigned to it in the merged block set.
        for atk in &self.attacking {
            if !kws_of(atk.attacker).contains(&Keyword::AllMustBlock) {
                continue;
            }
            let Some(defender_idx) = self.defender_for(atk.target) else { continue };
            let attacker = match self.battlefield_find(atk.attacker) {
                Some(a) => a,
                None => continue,
            };
            let atk_colors = cp_of(atk.attacker).map(|c| c.colors.as_slice()).unwrap_or(&[]);
            let unmet = self.battlefield.iter().any(|b| {
                b.definition.is_creature()
                    && self.same_team(b.controller, defender_idx)
                    && b.can_block()
                    && !kws_of(b.id).contains(&Keyword::CantBlock)
                    && cp_of(b.id).is_some_and(|bcp| {
                        super::can_block_attacker_computed(
                            b, attacker, bcp, kws_of(atk.attacker), atk_colors,
                        )
                    })
                    // Able to block it but not assigned to it (here or earlier).
                    && self.block_map.get(&b.id) != Some(&atk.attacker)
                    && !assignments.iter().any(|(bid, aid)| *bid == b.id && *aid == atk.attacker)
            });
            if unmet {
                return Err(GameError::MustBeBlockedIfAble(atk.attacker));
            }
        }

        // CR 702.39 — Provoke: a creature provoked this combat (`must_block`
        // set to an attacker still in this combat) must be assigned to block
        // that attacker if it's able to. It was untapped by the provoke
        // resolution, so "able" reduces to the normal can-block checks.
        for b in &self.battlefield {
            let Some(required) = b.must_block else { continue };
            // The provoker must still be attacking for the requirement to bind.
            if !self.attacking.iter().any(|a| a.attacker == required) { continue; }
            if !b.definition.is_creature()
                || !b.can_block()
                || kws_of(b.id).contains(&Keyword::CantBlock)
            {
                continue;
            }
            let Some(attacker) = self.battlefield_find(required) else { continue };
            let atk_colors = cp_of(required).map(|c| c.colors.as_slice()).unwrap_or(&[]);
            let able = cp_of(b.id).is_some_and(|bcp| {
                super::can_block_attacker_computed(b, attacker, bcp, kws_of(required), atk_colors)
            });
            if !able {
                continue;
            }
            let assigned = self.block_map.get(&b.id) == Some(&required)
                || assignments.iter().any(|(bid, aid)| *bid == b.id && *aid == required);
            if !assigned {
                return Err(GameError::MustBeBlockedIfAble(required));
            }
        }

        // Combat-keyword P/T adjustments applied on block declaration:
        // Flanking (CR 702.25), Bushido (CR 702.45), Rampage (CR 702.23).
        // Snapshot the +/-N deltas (same value on power and toughness)
        // before mutating so the borrow of `assignments` stays clean.
        let computed = self.compute_battlefield();
        let kws_for = |id: CardId| -> Vec<Keyword> {
            computed.iter().find(|c| c.id == id).map(|c| c.keywords.clone()).unwrap_or_default()
        };
        let sum_n = |kws: &[Keyword], pick: fn(&Keyword) -> Option<i32>| -> i32 {
            kws.iter().filter_map(pick).sum()
        };
        let mut pt_deltas: Vec<(CardId, i32)> = vec![];
        let mut blocked: std::collections::HashMap<CardId, usize> = std::collections::HashMap::new();
        for &(b, a) in &assignments {
            *blocked.entry(a).or_insert(0) += 1;
            let bk = kws_for(b);
            let ak = kws_for(a);
            // Flanking: nonflanking blocker shrinks once per flanking instance.
            let flank = ak.iter().filter(|k| **k == Keyword::Flanking).count() as i32;
            if flank > 0 && !bk.contains(&Keyword::Flanking) {
                pt_deltas.push((b, -flank));
            }
            // Bushido on the blocker (it blocks).
            let bn = sum_n(&bk, |k| if let Keyword::Bushido(x) = k { Some(*x as i32) } else { None });
            if bn > 0 { pt_deltas.push((b, bn)); }
        }
        for (a, count) in blocked {
            let ak = kws_for(a);
            // Bushido on the attacker (it becomes blocked — once).
            let bn = sum_n(&ak, |k| if let Keyword::Bushido(x) = k { Some(*x as i32) } else { None });
            if bn > 0 { pt_deltas.push((a, bn)); }
            // Rampage: +N for each blocker beyond the first.
            let rn = sum_n(&ak, |k| if let Keyword::Rampage(x) = k { Some(*x as i32) } else { None });
            let extra = count.saturating_sub(1) as i32;
            if rn > 0 && extra > 0 { pt_deltas.push((a, rn * extra)); }
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
        for (id, d) in pt_deltas {
            if let Some(c) = self.battlefield_find_mut(id) {
                c.power_bonus += d;
                c.toughness_bonus += d;
            }
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
        // CR 702.39 — provoke's "block this combat" requirement ends here.
        for c in &mut self.battlefield {
            c.must_block = None;
        }

        events.push(GameEvent::CombatResolved);
        Ok(events)
    }

    /// CR 702.15 — does `defender` control a land with the given land type?
    /// Reads printed land subtypes (Forest/Island/…), so dual lands and
    /// nonbasics with the type count.
    fn defender_controls_land_type(
        &self,
        defender: usize,
        lt: &crate::card::LandType,
    ) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller == defender && c.definition.has_land_type(*lt)
        })
    }

    /// CR 510.1c — ask the attacking player's decider to order the blockers
    /// for combat-damage assignment. `default_order` is the engine's
    /// deterministic fallback (CardId order). The decider's answer reorders
    /// the listed ids; any id it omits is appended in its default position,
    /// and unknown ids are ignored — so a partial or empty answer is always
    /// legal and `AutoDecider` keeps `default_order`.
    fn order_combat_blockers(
        &mut self,
        attacker: CardId,
        default_order: Vec<CardId>,
    ) -> Vec<CardId> {
        use crate::decision::{Decision, DecisionAnswer};
        let blockers: Vec<(CardId, String)> = default_order
            .iter()
            .map(|id| {
                let name = self
                    .battlefield_find(*id)
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_default();
                (*id, name)
            })
            .collect();
        let answer = self
            .decider
            .decide(&Decision::CombatDamageOrder { attacker, blockers });
        let DecisionAnswer::DamageOrder(chosen) = answer else {
            return default_order;
        };
        let mut ordered: Vec<CardId> = Vec::with_capacity(default_order.len());
        for id in &chosen {
            if default_order.contains(id) && !ordered.contains(id) {
                ordered.push(*id);
            }
        }
        for id in &default_order {
            if !ordered.contains(id) {
                ordered.push(*id);
            }
        }
        ordered
    }

    /// CR 510.1c-d — divide an attacker's `total_power` combat damage among
    /// its blockers (given in assignment order with their `lethal` amounts).
    /// Returns `(blocker_id, amount)` pairs in the same order; any power not
    /// assigned to a blocker is the trample-over leftover.
    ///
    /// The default (and `AutoDecider`) split assigns lethal to each blocker
    /// in order until the power runs out. A `wants_ui` / scripted decider may
    /// answer `CombatDamageAssignment` to over-assign (e.g. deny trample),
    /// subject to CR 510.1c: a blocker may receive damage only after every
    /// earlier blocker has been assigned at least its lethal, and the total
    /// can't exceed `total_power`. A malformed answer falls back to default.
    fn assign_combat_damage(
        &mut self,
        attacker: CardId,
        total_power: u32,
        lethals: &[(CardId, u32)],
    ) -> Vec<(CardId, u32)> {
        use crate::decision::{Decision, DecisionAnswer};
        let default_split = || {
            let mut remaining = total_power;
            lethals
                .iter()
                .map(|&(id, lethal)| {
                    let a = lethal.min(remaining);
                    remaining -= a;
                    (id, a)
                })
                .collect::<Vec<_>>()
        };
        // No meaningful choice for a single blocker or zero power.
        if lethals.len() <= 1 || total_power == 0 {
            return default_split();
        }
        let blockers: Vec<(CardId, String, u32)> = lethals
            .iter()
            .map(|&(id, lethal)| {
                let name = self
                    .battlefield_find(id)
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_default();
                (id, name, lethal)
            })
            .collect();
        let answer = self.decider.decide(&Decision::AssignCombatDamage {
            attacker,
            attacker_power: total_power,
            blockers,
        });
        let DecisionAnswer::CombatDamageAssignment(pairs) = answer else {
            return default_split();
        };
        if pairs.is_empty() {
            return default_split();
        }
        // Re-key the answer into blocker order (missing entries = 0).
        let amounts: Vec<u32> = lethals
            .iter()
            .map(|&(id, _)| {
                pairs
                    .iter()
                    .find(|(pid, _)| *pid == id)
                    .map(|(_, a)| *a)
                    .unwrap_or(0)
            })
            .collect();
        let assigned: u32 = amounts.iter().sum();
        if assigned > total_power {
            return default_split();
        }
        // Ordering rule: once a blocker is under-assigned, no later blocker
        // (nor trample-over) may receive damage.
        let mut earlier_all_lethal = true;
        for (i, &(_, lethal)) in lethals.iter().enumerate() {
            if !earlier_all_lethal && amounts[i] > 0 {
                return default_split();
            }
            if amounts[i] < lethal {
                earlier_all_lethal = false;
            }
        }
        if total_power > assigned && !earlier_all_lethal {
            // A trample-over leftover requires every blocker to be at lethal.
            return default_split();
        }
        lethals.iter().map(|&(id, _)| id).zip(amounts).collect()
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
                    toxic: kws.iter().filter_map(|k| match k {
                        Keyword::Toxic(n) => Some(*n),
                        _ => None,
                    }).sum(),
                    // CR 510.1 — a creature with "deals no combat damage this
                    // turn" (Master of Cruelties) is skipped in both damage
                    // steps even though it's a legal attacker/blocker. CR 614.9
                    // — a Maze-of-Ith'd attacker deals no combat damage either.
                    should_deal: attacker_filter(kws)
                        && !kws.contains(&Keyword::DealsNoCombatDamage)
                        && !self.combat_damage_prevented_creatures.contains(&cp.id),
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

        // CR 614.2 — global combat-damage doubling (Furnace of Rath /
        // Gratuitous Violence). Each `DoubleDamageDealt` permanent doubles
        // every damage *event*; the doubling applies to the amount dealt
        // (after assignment, before prevention), so a creature is still
        // assigned base lethal but takes double, and trample-over / player
        // damage double too. `1` (no doublers) leaves combat untouched.
        let dmg_mult = {
            let d = self.damage_doublers();
            if d > 0 { 1u32 << d.min(16) } else { 1 }
        };

        for atk in &attacker_infos {
            if !atk.should_deal {
                continue;
            }

            // CR 510.1c: the attacking player chooses the order in which an
            // attacker assigns combat damage to its multiple blockers. Start
            // from a deterministic default (CardId = declaration-order proxy,
            // so iteration isn't gated on HashMap order), then let the
            // attacker's decider reorder via `Decision::CombatDamageOrder`.
            let mut blocker_ids: Vec<CardId> = self
                .block_map
                .iter()
                .filter(|(_, aid)| **aid == atk.id)
                .map(|(&bid, _)| bid)
                .collect();
            blocker_ids.sort_by_key(|id| id.0);
            if blocker_ids.len() > 1 {
                blocker_ids = self.order_combat_blockers(atk.id, blocker_ids);
            }

            if blocker_ids.is_empty() {
                let raw = if prevent_combat_damage {
                    0
                } else {
                    atk.power.max(0) as u32
                };
                // CR 615 — per-target prevention shields on the defending
                // player/planeswalker also reduce unblocked combat damage.
                // Lifelink scales off the post-prevention amount (702.15a).
                let amount =
                    self.prevent_combat_to_target(atk.target, raw.saturating_mul(dmg_mult), &mut events);
                if amount > 0 {
                    self.deal_combat_damage_to_target(atk, amount, &mut events);
                    if atk.has_lifelink {
                        let a = self.active_player_idx;
                        self.adjust_life(a, amount as i32);
                        events.push(GameEvent::LifeGained { player: a, amount });
                    }
                }
            } else {
                let total_power = if prevent_combat_damage {
                    0
                } else {
                    atk.power.max(0) as u32
                };
                // CR 510.1c-d — the attacking player divides combat damage
                // among the blockers in the chosen order. Build the default
                // lethal-to-each split, then let the controller override it
                // (e.g. over-assign to a blocker to deny trample). The lethal
                // for each is 1 under deathtouch (CR 702.2e).
                let lethals: Vec<(CardId, u32)> = blocker_ids
                    .iter()
                    .map(|&bid| {
                        let tough = computed_of(bid)
                            .map(|c| c.toughness.max(0) as u32)
                            .unwrap_or(0);
                        (bid, if atk.has_deathtouch { 1 } else { tough })
                    })
                    .collect();
                let assignment = self.assign_combat_damage(atk.id, total_power, &lethals);
                let mut lifelink_dealt = 0i32;
                let mut assigned_to_blockers = 0u32;

                for &(blocker_id, assign) in &assignment {
                    assigned_to_blockers += assign;
                    if assign == 0 {
                        continue;
                    }
                    // CR 614.9 — a Maze-of-Ith'd blocker takes no combat damage.
                    if self.combat_damage_prevented_creatures.contains(&blocker_id) {
                        continue;
                    }
                    // CR 615 — route attacker→blocker combat damage through
                    // the blocker's prevention shields. Lifelink and the
                    // wither/infect -1/-1 counters scale off the actual
                    // (post-prevention) amount dealt (CR 702.15a).
                    let dealt = self.apply_prevention_shields(
                        crate::game::effects::EntityRef::Permanent(blocker_id),
                        assign.saturating_mul(dmg_mult),
                        &mut events,
                    ) as i32;
                    lifelink_dealt += dealt;

                    if atk.has_infect || atk.has_wither {
                        if dealt > 0
                            && let Some(blocker) = self.battlefield_find_mut(blocker_id)
                        {
                            blocker.add_counters(
                                crate::card::CounterType::MinusOneMinusOne,
                                dealt as u32,
                            );
                            events.push(GameEvent::CounterAdded {
                                card_id: blocker_id,
                                counter_type: crate::card::CounterType::MinusOneMinusOne,
                                count: dealt as u32,
                            });
                        }
                    } else if dealt > 0
                        && let Some(blocker) = self.battlefield_find_mut(blocker_id)
                    {
                        blocker.damage += dealt as u32;
                        if atk.has_deathtouch {
                            blocker.dealt_deathtouch_damage = true;
                        }
                        events.push(GameEvent::DamageDealt {
                            amount: dealt as u32,
                            to_player: None,
                            to_card: Some(blocker_id),
                        });
                    }
                }

                let trample_leftover = total_power.saturating_sub(assigned_to_blockers);
                if atk.has_trample && trample_leftover > 0 {
                    // Trample-over damage to the defending player/PW is also
                    // subject to prevention shields; lifelink follows the
                    // post-prevention amount.
                    let amount = self.prevent_combat_to_target(
                        atk.target,
                        trample_leftover.saturating_mul(dmg_mult),
                        &mut events,
                    );
                    lifelink_dealt += amount as i32;
                    if amount > 0 {
                        self.deal_combat_damage_to_target(atk, amount, &mut events);
                    }
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
                    // CR 614.9 — a Maze-of-Ith'd blocker deals no combat damage.
                    .filter(|bid| !self.combat_damage_prevented_creatures.contains(bid))
                    .collect();

                let blocker_damage_to_attacker: i32 = if prevent_combat_damage
                    // CR 614.9 — a Maze-of-Ith'd attacker takes no combat damage.
                    || self.combat_damage_prevented_creatures.contains(&atk.id)
                {
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
                    // CR 615 — route blocker→attacker combat damage through
                    // the attacker's prevention shields before marking it.
                    // CR 614.2 doubling also applies to the blocker's strike.
                    let dmg = self.apply_prevention_shields(
                        crate::game::effects::EntityRef::Permanent(atk.id),
                        (blocker_damage_to_attacker.max(0) as u32).saturating_mul(dmg_mult),
                        &mut events,
                    );
                    if dmg > 0 && let Some(attacker) = self.battlefield_find_mut(atk.id) {
                        if any_infect_blocker {
                            attacker.add_counters(crate::card::CounterType::MinusOneMinusOne, dmg);
                            events.push(GameEvent::CounterAdded {
                                card_id: atk.id,
                                counter_type: crate::card::CounterType::MinusOneMinusOne,
                                count: dmg,
                            });
                        } else {
                            attacker.damage += dmg;
                            if any_deathtouch_blocker {
                                attacker.dealt_deathtouch_damage = true;
                            }
                            events.push(GameEvent::DamageDealt {
                                amount: dmg,
                                to_player: None,
                                to_card: Some(atk.id),
                            });
                        }
                    }

                    // Blocker lifelink — gained by each blocker's controller
                    // (different blockers can have different controllers in
                    // multiplayer). Only blockers actually striking back in
                    // this step gain life from it. CR 702.15a: lifelink
                    // scales off damage actually dealt, so a fully-prevented
                    // attacker (shield) yields no blocker lifelink.
                    let mut lifelink_by_controller: std::collections::HashMap<usize, i32> =
                        std::collections::HashMap::new();
                    for &bid in dealing_blocker_ids.iter().filter(|_| dmg > 0) {
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
                        *lifelink_by_controller.entry(controller).or_insert(0) +=
                            bc.power.saturating_mul(dmg_mult as i32);
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
    /// CR 615 — apply prevention shields to combat damage headed for an
    /// attack target (player or planeswalker). Returns the unprevented
    /// remainder. Creature-vs-creature combat damage is not yet routed
    /// through shields (tracked in TODO.md).
    fn prevent_combat_to_target(
        &mut self,
        target: AttackTarget,
        amount: u32,
        events: &mut Vec<GameEvent>,
    ) -> u32 {
        use crate::game::effects::EntityRef;
        match target {
            AttackTarget::Player(p) => {
                self.apply_prevention_shields(EntityRef::Player(p), amount, events)
            }
            AttackTarget::Planeswalker(pw) => {
                self.apply_prevention_shields(EntityRef::Permanent(pw), amount, events)
            }
        }
    }

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
                // Mark the player damaged this turn (Bloodthirst window, CR
                // 702.54) and record the attacker so "destroy target creature
                // that dealt damage to you this turn" (Spear of Heliod) can
                // filter targets.
                if amount > 0 {
                    self.players[p].was_dealt_damage_this_turn = true;
                    if !self.players[p].creatures_that_damaged_me_this_turn.contains(&atk.id) {
                        self.players[p].creatures_that_damaged_me_this_turn.push(atk.id);
                    }
                }
                // CR 702.180c — Toxic N adds N poison on combat damage to a
                // player, on top of any life loss (and stacks with Infect's
                // poison). Only when damage was actually dealt.
                if atk.toxic > 0 && amount > 0 {
                    self.players[p].poison_counters += atk.toxic;
                    events.push(GameEvent::PoisonAdded {
                        player: p,
                        amount: atk.toxic,
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
    toxic: u32,
    should_deal: bool,
}
