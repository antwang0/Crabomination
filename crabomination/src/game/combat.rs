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

        // CR 508.1g — attack tax (Ghostly Prison / Propaganda). Sum {amount}
        // for each attacker hitting a player who controls an
        // `AttackTaxToController` static (copies stack), then auto-pay it from
        // the active player's mana pool. Reject the whole declaration if the
        // pool can't cover it (a wants_ui interactive pay prompt is a TODO).
        let mut total_tax = 0u32;
        for atk in &attacks {
            // The defending player whose statics apply, and whether the attack
            // is aimed at a planeswalker (so `protect_planeswalkers` gates it).
            let (defender, at_planeswalker) = match atk.target {
                crate::game::types::AttackTarget::Player(d) => (Some(d), false),
                crate::game::types::AttackTarget::Planeswalker(pw) => {
                    (self.battlefield_find(pw).map(|c| c.controller), true)
                }
            };
            let Some(d) = defender else { continue };
            // Evaluate each tax `amount` with the defender as "you" (and the
            // tax permanent as source) so dynamic taxes — Sphere of Safety's
            // "number of enchantments you control" — count the defender's
            // board. Fixed taxes are `Value::Const(n)`.
            for c in &self.battlefield {
                if c.controller != d {
                    continue;
                }
                for sa in &c.definition.static_abilities {
                    if let crate::effect::StaticEffect::AttackTaxToController {
                        amount,
                        protect_planeswalkers,
                    } = &sa.effect
                        && (!at_planeswalker || *protect_planeswalkers)
                    {
                        let mut ctx = crate::game::effects::EffectContext::for_spell(d, None, 0, 0);
                        ctx.source = Some(c.id);
                        total_tax += self.evaluate_value(amount, &ctx).max(0) as u32;
                    }
                }
            }
        }
        if total_tax > 0 {
            if self.players[p].mana_pool.total() < total_tax {
                return Err(GameError::CannotAttack(attacks[0].attacker));
            }
            self.players[p].mana_pool.spend_generic(total_tax);
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
            // "Can't attack unless you control a [filter]" (Lovestruck Beast):
            // require ≥1 matching permanent under the attacker's controller.
            if let Some(req) = computed_kw(id).iter().find_map(|kw| match kw {
                Keyword::CanAttackOnlyIfYouControl(r) => Some(r.clone()),
                _ => None,
            }) {
                let satisfied = self
                    .battlefield
                    .iter()
                    .any(|c| c.controller == p && self.evaluate_requirement_on_card(&req, c, p));
                if !satisfied {
                    return Err(GameError::CannotAttack(id));
                }
            }
            // "Can't attack unless it has an even number of counters on it"
            // (Sab-Sunen). Zero is even. Sum every counter kind on the card.
            if computed_kw(id).contains(&Keyword::CantAttackOrBlockUnlessEvenCounters)
                && let Some(c) = self.battlefield.iter().find(|c| c.id == id)
                && c.counters.values().sum::<u32>() % 2 != 0
            {
                return Err(GameError::CannotAttack(id));
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
                && card.detained_by.is_none()
                && !kws.contains(&Keyword::Defender)
                && !kws.contains(&Keyword::CantAttack)
                && (!card.summoning_sick || kws.contains(&Keyword::Haste));
            if !can_attack {
                if card.tapped {
                    return Err(GameError::CardIsTapped(id));
                }
                // CR 701.35 — a detained permanent can't attack.
                if card.detained_by.is_some()
                    || kws.contains(&Keyword::Defender)
                    || kws.contains(&Keyword::CantAttack)
                {
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
            // CR 702.147 — Decayed. "When it attacks, sacrifice it at end of
            // combat." Reuse the attacking-token cleanup queue (CR 511.3).
            if computed_kw(id).contains(&Keyword::Decayed) {
                self.attacking_token_cleanup.push((
                    id,
                    crate::effect::AttackingTokenCleanup::SacrificeAtEndOfCombat,
                ));
            }
            // CR 702.142 — record that this creature attacked (gates Boast).
            card.attacked_this_turn = true;
            self.attacking.push(atk);
            // Raid (CR 702.108 ability word): the controller attacked this turn.
            self.players[p].attacked_this_turn = true;
            self.players[p].creatures_attacked_this_turn += 1;
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
                        c.definition
                            .triggered_abilities
                            .iter()
                            .filter(|t| {
                                t.event.kind == EventKind::Attacks
                                    && t.event.scope
                                        == crate::effect::EventScope::ControllerAttackedByOpponent
                            })
                            .map(move |t| (c.id, t.effect.clone()))
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
                    bargained: false,
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

            // CR 701.35 — a detained permanent can't block.
            if blocker.detained_by.is_some() {
                return Err(GameError::CannotBlock(blocker_id));
            }

            // `Keyword::CantBlock` is enforced from the *computed* keyword
            // set so transient grants (e.g. SOS Duel Tactics's "this
            // creature can't block this turn", Postmortem Professor's
            // static restriction) take effect immediately.
            if kws_of(blocker_id).contains(&Keyword::CantBlock)
                || kws_of(blocker_id).contains(&Keyword::Decayed)
            {
                return Err(GameError::CannotBlock(blocker_id));
            }

            // Per-pair "can't block this creature this turn" (Kozilek's
            // Pathfinder): the blocker is barred only from this attacker.
            if self.cant_block_pairs.contains(&(blocker_id, attacker_id)) {
                return Err(GameError::CannotBlock(blocker_id));
            }


            // "Can't block unless it has an even number of counters on it"
            // (Sab-Sunen). Zero is even; reject an odd total counter count.
            if kws_of(blocker_id).contains(&Keyword::CantAttackOrBlockUnlessEvenCounters)
                && blocker.counters.values().sum::<u32>() % 2 != 0
            {
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

        // CR 509.1d — block tax (Archangel of Tithes). Sum every active
        // `BlockTaxToController` amount (an `only_while_attacking` source counts
        // only while it's attacking this combat) and auto-pay {tax} per declared
        // blocker out of that blocker's controller's mana pool. Reject the whole
        // declaration if a blocking player can't cover it. (A wants_ui pay prompt
        // is a TODO, shared with the attack-tax path.)
        let block_tax_per: u32 = {
            let mut sum = 0u32;
            for c in &self.battlefield {
                for sa in &c.definition.static_abilities {
                    if let crate::effect::StaticEffect::BlockTaxToController {
                        amount,
                        only_while_attacking,
                    } = &sa.effect
                    {
                        if *only_while_attacking
                            && !self.attacking.iter().any(|a| a.attacker == c.id)
                        {
                            continue;
                        }
                        let mut ctx =
                            crate::game::effects::EffectContext::for_spell(c.controller, None, 0, 0);
                        ctx.source = Some(c.id);
                        sum += self.evaluate_value(amount, &ctx).max(0) as u32;
                    }
                }
            }
            sum
        };
        if block_tax_per > 0 {
            let mut tax_by_controller: std::collections::HashMap<usize, u32> =
                std::collections::HashMap::new();
            for &(blocker_id, _) in &assignments {
                if let Some(b) = self.battlefield_find(blocker_id) {
                    *tax_by_controller.entry(b.controller).or_insert(0) += block_tax_per;
                }
            }
            for (&player, &owed) in &tax_by_controller {
                if self.players[player].mana_pool.total() < owed {
                    return Err(GameError::CannotBlock(assignments[0].0));
                }
            }
            for (player, owed) in tax_by_controller {
                self.players[player].mana_pool.spend_generic(owed);
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

        // "Can't be blocked except by N or more creatures" (Pathrazer of
        // Ulamog). Generalized Menace: 0 or >= N blockers, never 1..N-1.
        for atk in &self.attacking {
            for kw in kws_of(atk.attacker) {
                if let Keyword::CantBeBlockedExceptByN(n) = kw {
                    let blocker_count = assignments
                        .iter()
                        .filter(|(_, aid)| *aid == atk.attacker)
                        .count()
                        + self.block_map.values().filter(|&&aid| aid == atk.attacker).count();
                    if blocker_count > 0 && (blocker_count as u32) < *n {
                        return Err(GameError::MenaceRequiresTwoBlockers(atk.attacker));
                    }
                }
            }
        }

        // CR 509.1g — "can't be blocked by more than one creature" (Charging
        // Rhino). At most one blocker may be assigned (the inverse of Menace).
        for atk in &self.attacking {
            if kws_of(atk.attacker).contains(&Keyword::CantBeBlockedByMoreThanOne) {
                let blocker_count = assignments
                    .iter()
                    .filter(|(_, aid)| *aid == atk.attacker)
                    .count()
                    + self.block_map.values().filter(|&&aid| aid == atk.attacker).count();
                if blocker_count > 1 {
                    return Err(GameError::CannotBeBlockedByMoreThanOne(atk.attacker));
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

        // CR 509.1c — "blocks each combat if able" (`MustBlock`). A creature
        // carrying the keyword that can legally block at least one declared
        // attacker must be assigned to block one of them.
        for b in &self.battlefield {
            if !kws_of(b.id).contains(&Keyword::MustBlock)
                || !b.definition.is_creature()
                || !b.can_block()
                || kws_of(b.id).contains(&Keyword::CantBlock)
            {
                continue;
            }
            let already = self.block_map.contains_key(&b.id)
                || assignments.iter().any(|(bid, _)| *bid == b.id);
            if already { continue; }
            // Could it have blocked any declared attacker?
            let could_block = self.attacking.iter().any(|atk| {
                let same_team = self
                    .defender_for(atk.target)
                    .is_some_and(|d| self.same_team(b.controller, d));
                same_team
                    && self
                        .battlefield_find(atk.attacker)
                        .zip(cp_of(b.id))
                        .is_some_and(|(attacker, bcp)| {
                            let atk_colors =
                                cp_of(atk.attacker).map(|c| c.colors.clone()).unwrap_or_default();
                            super::can_block_attacker_computed(
                                b, attacker, bcp, kws_of(atk.attacker), &atk_colors,
                            )
                        })
            });
            if could_block {
                return Err(GameError::MustBeBlockedIfAble(b.id));
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
        // Suspended on a `wants_ui` player's combat-damage choice — no damage
        // has been dealt yet; `submit_decision` re-enters this step.
        if self.pending_decision.is_some() {
            return Ok(events);
        }
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

        // Suspended on a `wants_ui` player's combat-damage choice — no damage
        // dealt yet; combat is not torn down. `submit_decision` re-enters.
        if self.pending_decision.is_some() {
            return Ok(events);
        }

        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);

        self.attacking.clear();
        self.block_map.clear();
        self.clear_combat_damage_plan();
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

    /// CR 510.1c — build the `Decision::CombatDamageOrder` asking the
    /// attacking player to order `default_order` (deterministic CardId order)
    /// for combat-damage assignment.
    fn combat_damage_order_decision(
        &self,
        attacker: CardId,
        default_order: &[CardId],
    ) -> crate::decision::Decision {
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
        crate::decision::Decision::CombatDamageOrder { attacker, blockers }
    }

    /// Validate a `DamageOrder` answer into a concrete blocker order. Ids the
    /// answer omits are appended in their default position, and unknown ids
    /// are ignored — so a partial or empty answer is always legal and keeps
    /// `default_order`.
    fn resolve_damage_order(
        &self,
        default_order: &[CardId],
        answer: &crate::decision::DecisionAnswer,
    ) -> Vec<CardId> {
        use crate::decision::DecisionAnswer;
        let DecisionAnswer::DamageOrder(chosen) = answer else {
            return default_order.to_vec();
        };
        let mut ordered: Vec<CardId> = Vec::with_capacity(default_order.len());
        for id in chosen {
            if default_order.contains(id) && !ordered.contains(id) {
                ordered.push(*id);
            }
        }
        for id in default_order {
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
    fn default_damage_split(
        &self,
        total_power: u32,
        lethals: &[(CardId, u32)],
    ) -> Vec<(CardId, u32)> {
        let mut remaining = total_power;
        lethals
            .iter()
            .map(|&(id, lethal)| {
                let a = lethal.min(remaining);
                remaining -= a;
                (id, a)
            })
            .collect()
    }

    /// Build the `Decision::AssignCombatDamage` for dividing `total_power`
    /// among `lethals` (in assignment order).
    fn assign_combat_damage_decision(
        &self,
        attacker: CardId,
        total_power: u32,
        lethals: &[(CardId, u32)],
    ) -> crate::decision::Decision {
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
        crate::decision::Decision::AssignCombatDamage {
            attacker,
            attacker_power: total_power,
            blockers,
        }
    }

    /// Validate a `CombatDamageAssignment` answer into a concrete split. An
    /// empty or rule-violating answer falls back to `default_damage_split`.
    fn resolve_damage_assignment(
        &self,
        total_power: u32,
        lethals: &[(CardId, u32)],
        answer: &crate::decision::DecisionAnswer,
    ) -> Vec<(CardId, u32)> {
        use crate::decision::DecisionAnswer;
        let DecisionAnswer::CombatDamageAssignment(pairs) = answer else {
            return self.default_damage_split(total_power, lethals);
        };
        if pairs.is_empty() {
            return self.default_damage_split(total_power, lethals);
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
            return self.default_damage_split(total_power, lethals);
        }
        // Ordering rule: once a blocker is under-assigned, no later blocker
        // (nor trample-over) may receive damage.
        let mut earlier_all_lethal = true;
        for (i, &(_, lethal)) in lethals.iter().enumerate() {
            if !earlier_all_lethal && amounts[i] > 0 {
                return self.default_damage_split(total_power, lethals);
            }
            if amounts[i] < lethal {
                earlier_all_lethal = false;
            }
        }
        if total_power > assigned && !earlier_all_lethal {
            // A trample-over leftover requires every blocker to be at lethal.
            return self.default_damage_split(total_power, lethals);
        }
        lethals.iter().map(|&(id, _)| id).zip(amounts).collect()
    }

    /// CR 510.1c-d — gather (and cache) the active player's combat-damage
    /// ordering and assignment choices for every multi-blocker attacker,
    /// before any damage is applied. Returns `true` if it suspended on a
    /// `wants_ui` player's pending decision (the caller must return early and
    /// re-enter the damage step after the answer); `false` once every choice
    /// is settled. Pure w.r.t. the battlefield — only the decision caches and
    /// `pending_decision` are written.
    fn gather_combat_damage_decisions(
        &mut self,
        attacker_infos: &[AttackerInfo],
        computed: &[ComputedPermanent],
    ) -> bool {
        use crate::game::types::{CombatDecisionKind, PendingDecision, ResumeContext};
        // Reset the caches once when entering a new damage step (first-strike
        // vs regular), but never on a mid-step decision resume.
        if self.combat_damage_plan_step != Some(self.step) {
            self.combat_damage_order.clear();
            self.combat_damage_assignment.clear();
            self.combat_damage_plan_step = Some(self.step);
        }
        let active = self.active_player_idx;
        let wants_ui = self.players[active].wants_ui;
        for atk in attacker_infos.iter().filter(|a| a.should_deal) {
            let mut blocker_ids: Vec<CardId> = self
                .block_map
                .iter()
                .filter(|(_, aid)| **aid == atk.id)
                .map(|(&bid, _)| bid)
                .collect();
            blocker_ids.sort_by_key(|id| id.0);
            if blocker_ids.len() <= 1 {
                continue;
            }

            // 1) Blocker order (CR 510.1c).
            if !self.combat_damage_order.contains_key(&atk.id) {
                let decision = self.combat_damage_order_decision(atk.id, &blocker_ids);
                if wants_ui {
                    self.pending_decision = Some(PendingDecision {
                        decision,
                        resume: ResumeContext::CombatDamage {
                            player: active,
                            attacker: atk.id,
                            kind: CombatDecisionKind::Order,
                        },
                    });
                    return true;
                }
                let answer = self.decider.decide(&decision);
                let order = self.resolve_damage_order(&blocker_ids, &answer);
                self.combat_damage_order.insert(atk.id, order);
            }
            let order = self.combat_damage_order[&atk.id].clone();

            // 2) Damage assignment across the ordered blockers (CR 510.1d).
            if !self.combat_damage_assignment.contains_key(&atk.id) {
                let total_power = if self.prevent_combat_damage_this_turn {
                    0
                } else {
                    atk.power.max(0) as u32
                };
                let lethals = self.combat_lethals(atk.has_deathtouch, &order, computed);
                // No meaningful choice with zero power — store the default.
                if total_power == 0 {
                    let split = self.default_damage_split(total_power, &lethals);
                    self.combat_damage_assignment.insert(atk.id, split);
                    continue;
                }
                let decision =
                    self.assign_combat_damage_decision(atk.id, total_power, &lethals);
                if wants_ui {
                    self.pending_decision = Some(PendingDecision {
                        decision,
                        resume: ResumeContext::CombatDamage {
                            player: active,
                            attacker: atk.id,
                            kind: CombatDecisionKind::Assign,
                        },
                    });
                    return true;
                }
                let answer = self.decider.decide(&decision);
                let split = self.resolve_damage_assignment(total_power, &lethals, &answer);
                self.combat_damage_assignment.insert(atk.id, split);
            }
        }
        false
    }

    /// Lethal damage required for each blocker in `order` (its toughness, or 1
    /// under deathtouch per CR 702.2e). Blockers no longer on the battlefield
    /// resolve to 0.
    fn combat_lethals(
        &self,
        attacker_deathtouch: bool,
        order: &[CardId],
        computed: &[ComputedPermanent],
    ) -> Vec<(CardId, u32)> {
        order
            .iter()
            .map(|&bid| {
                let tough = computed
                    .iter()
                    .find(|c| c.id == bid)
                    .map(|c| c.toughness.max(0) as u32)
                    .unwrap_or(0);
                (bid, if attacker_deathtouch { 1 } else { tough })
            })
            .collect()
    }

    /// Validate and cache one combat-damage decision answered via
    /// `submit_decision`, so the re-entered damage step finds it settled.
    pub(crate) fn apply_combat_decision_answer(
        &mut self,
        attacker: CardId,
        kind: crate::game::types::CombatDecisionKind,
        answer: &crate::decision::DecisionAnswer,
    ) {
        use crate::game::types::CombatDecisionKind;
        match kind {
            CombatDecisionKind::Order => {
                let mut default_order: Vec<CardId> = self
                    .block_map
                    .iter()
                    .filter(|(_, aid)| **aid == attacker)
                    .map(|(&bid, _)| bid)
                    .collect();
                default_order.sort_by_key(|id| id.0);
                let order = self.resolve_damage_order(&default_order, answer);
                self.combat_damage_order.insert(attacker, order);
            }
            CombatDecisionKind::Assign => {
                let order = self
                    .combat_damage_order
                    .get(&attacker)
                    .cloned()
                    .unwrap_or_default();
                let computed = self.compute_battlefield();
                let atk_cp = computed.iter().find(|c| c.id == attacker);
                let deathtouch = atk_cp
                    .is_some_and(|c| c.keywords.contains(&Keyword::Deathtouch));
                let power = atk_cp.map(|c| c.power).unwrap_or(0);
                let total_power = if self.prevent_combat_damage_this_turn {
                    0
                } else {
                    power.max(0) as u32
                };
                let lethals = self.combat_lethals(deathtouch, &order, &computed);
                let split = self.resolve_damage_assignment(total_power, &lethals, answer);
                self.combat_damage_assignment.insert(attacker, split);
            }
        }
    }

    /// Clear the cached combat-damage choices at the end of a combat phase.
    pub(crate) fn clear_combat_damage_plan(&mut self) {
        self.combat_damage_order.clear();
        self.combat_damage_assignment.clear();
        self.combat_damage_plan_step = None;
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

        // PHASE 1 — gather the active player's combat-damage ordering and
        // assignment choices for every multi-blocker attacker, before any
        // damage is dealt. For a `wants_ui` player each choice surfaces as a
        // `pending_decision` and this returns early; `submit_decision` then
        // re-enters this damage step (which re-runs the now-cached gather and
        // proceeds once every choice is settled). The choices are cached in
        // `combat_damage_order` / `combat_damage_assignment` and read in the
        // apply phase below.
        if self.gather_combat_damage_decisions(&attacker_infos, computed) {
            return Ok(vec![]);
        }

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

        // Creature-vs-creature combat damage recorded here and dispatched after
        // all damage in this step is dealt, so `DealsCombatDamageToCreature`
        // triggers (CR 510.2) go on the stack simultaneously (CR 603.3b).
        let mut creature_damage: Vec<(CardId, CardId, u32)> = vec![];

        for atk in &attacker_infos {
            if !atk.should_deal {
                continue;
            }

            // CR 510.1c: the attacking player chose the order in which an
            // attacker assigns combat damage to its multiple blockers; that
            // choice was gathered in PHASE 1 and cached. Start from the
            // deterministic default (CardId = declaration-order proxy) and use
            // the cached order when present.
            let mut blocker_ids: Vec<CardId> = self
                .block_map
                .iter()
                .filter(|(_, aid)| **aid == atk.id)
                .map(|(&bid, _)| bid)
                .collect();
            blocker_ids.sort_by_key(|id| id.0);
            if blocker_ids.len() > 1
                && let Some(order) = self.combat_damage_order.get(&atk.id)
            {
                blocker_ids = order.clone();
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
                // among the blockers in the chosen order; that choice was
                // gathered in PHASE 1 and cached. Fall back to the default
                // lethal-to-each split when there's no cached choice (single
                // blocker, prevented, or a non-UI path that stored the
                // default). The lethal for each is 1 under deathtouch (CR
                // 702.2e).
                let lethals: Vec<(CardId, u32)> = blocker_ids
                    .iter()
                    .map(|&bid| {
                        let tough = computed_of(bid)
                            .map(|c| c.toughness.max(0) as u32)
                            .unwrap_or(0);
                        (bid, if atk.has_deathtouch { 1 } else { tough })
                    })
                    .collect();
                let assignment = self
                    .combat_damage_assignment
                    .get(&atk.id)
                    .cloned()
                    .unwrap_or_else(|| self.default_damage_split(total_power, &lethals));
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
                    // CR 702.16e — protection from the attacker's color prevents
                    // its combat damage to the blocker.
                    if self.damage_prevented_by_protection(atk.id, blocker_id) {
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
                        creature_damage.push((atk.id, blocker_id, dealt as u32));
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
                    // CR 702.16e — a blocker whose color the attacker has
                    // protection from deals no combat damage to it.
                    .filter(|&bid| !self.damage_prevented_by_protection(bid, atk.id))
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
                    // Each blocker that struck the attacker dealt combat damage
                    // to a creature (CR 510.2). Record per-blocker (its own
                    // power as the amount) so each blocker's to-creature
                    // triggers fire; only when some damage got through.
                    if dmg > 0 {
                        for &bid in &dealing_blocker_ids {
                            let p = computed_of(bid).map(|c| c.power.max(0) as u32).unwrap_or(0);
                            if p > 0 {
                                creature_damage.push((bid, atk.id, p));
                            }
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

        // CR 510.2 — now that all combat damage in this step has been dealt,
        // put `DealsCombatDamageToCreature` triggers on the stack.
        for (source, damaged, amount) in creature_damage {
            self.fire_combat_damage_to_creature_triggers(source, damaged, amount);
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
                // CR 614.9 — Palisade-Giant-style redirect: combat damage
                // aimed at the player lands on the redirector instead.
                if let Some(redirect) =
                    self.damage_redirect_target(crate::game::effects::EntityRef::Player(p))
                {
                    if let Some(c) = self.battlefield_find_mut(redirect) {
                        c.damage += amount;
                    }
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: None,
                        to_card: Some(redirect),
                    });
                    return;
                }
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
                // CR 724 — a creature dealing combat damage to the monarch
                // makes its controller the new monarch.
                if amount > 0 && self.monarch == Some(p) {
                    let ctrl = self.battlefield.iter()
                        .find(|c| c.id == atk.id).map(|c| c.controller);
                    if let Some(ctrl) = ctrl
                        && ctrl != p {
                            self.set_monarch(ctrl, events);
                        }
                }
                self.fire_combat_damage_to_player_triggers(atk.id, p, amount);
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
    pub(crate) fn fire_combat_damage_to_player_triggers(
        &mut self,
        source: CardId,
        damaged_player: usize,
        damage_amount: u32,
    ) {
        self.fire_combat_damage_triggers(
            source,
            EventKind::DealsCombatDamageToPlayer,
            Target::Player(damaged_player),
            damage_amount,
        );
    }

    /// Push triggered abilities of `source` whose event spec is
    /// `DealsCombatDamageToCreature` onto the stack, binding the damaged
    /// creature to the trigger's target so "destroy / exile / -1/-1 that
    /// creature" payoffs and equipment charge-triggers (Umezawa's Jitte)
    /// resolve correctly. CR 510.2. Fires once per (source, damaged-creature)
    /// pair; an equipped creature blocked by several creatures therefore
    /// charges Jitte once per blocker (a minor over-count for the rare
    /// multi-block case).
    pub(crate) fn fire_combat_damage_to_creature_triggers(
        &mut self,
        source: CardId,
        damaged_creature: CardId,
        damage_amount: u32,
    ) {
        self.fire_combat_damage_triggers(
            source,
            EventKind::DealsCombatDamageToCreature,
            Target::Permanent(damaged_creature),
            damage_amount,
        );
    }

    /// Shared body for the combat-damage trigger dispatch (to a player or to a
    /// creature). Walks the attacker's printed `SelfSource`/`AnyPlayer`
    /// triggers, equipment- and soulbond-granted triggers (CR 702.6e / 702.95),
    /// `YourControl`-scope listeners, and `FromYourGraveyard` triggers, pushing
    /// each onto the stack with `default_target` bound to slot 0.
    fn fire_combat_damage_triggers(
        &mut self,
        source: CardId,
        kind: EventKind,
        default_target: Target,
        damage_amount: u32,
    ) {
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
                        t.event.kind == kind
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

        // Phase 1b: equipment-granted combat-damage triggers (CR 702.6e). Each
        // Equipment attached to the attacker grants its `equipped_bonus.
        // triggered_abilities` to the creature; a `DealsCombatDamageToPlayer`
        // one fires here (the Sword cycle's "create a token / mill / draw"
        // riders), bound to the attacker's controller.
        if let Some(atk_ctrl) = attacker_controller {
            for eq in &self.battlefield {
                if eq.attached_to != Some(source) {
                    continue;
                }
                let Some(bonus) = &eq.definition.equipped_bonus else { continue };
                // CR 702.6e — the granted ability fires off the creature, unless
                // the Equipment opts to fire off itself (Umezawa's Jitte puts the
                // counters on the Equipment, so `Selector::This` must read it).
                let trig_source = if bonus.triggers_on_equipment { eq.id } else { source };
                for t in &bonus.triggered_abilities {
                    if t.event.kind == kind
                        && matches!(
                            t.event.scope,
                            crate::effect::EventScope::SelfSource
                                | crate::effect::EventScope::AnyPlayer
                        )
                    {
                        triggers.push((trig_source, t.effect.clone(), atk_ctrl));
                    }
                }
            }
            // CR 702.95 — Soulbond-granted combat-damage triggers. A paired
            // creature carrying `soulbond_bonus.triggered_abilities` grants
            // them to BOTH members; a `DealsCombatDamageToPlayer` one fires off
            // the attacker (Tandem Lookout's "deals combat damage → draw").
            for src in &self.battlefield {
                let Some(bonus) = &src.definition.soulbond_bonus else { continue };
                let Some(partner) = src.soulbond_partner else { continue };
                if src.id != source && partner != source {
                    continue;
                }
                if !self.battlefield.iter().any(|c| c.id == partner) {
                    continue;
                }
                for t in &bonus.triggered_abilities {
                    if t.event.kind == kind {
                        triggers.push((source, t.effect.clone(), atk_ctrl));
                    }
                }
            }
        }

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
                    if t.event.kind == kind
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
                        if t.event.kind == kind
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

        // CR 702.46 — Cipher. A card exiled encoded on this creature offers its
        // controller a free copy whenever the creature deals combat damage to a
        // player. Reuses the Paradigm free-copy effect (mint a token copy of the
        // exiled card and free-cast it; the encoded original stays in exile).
        if kind == EventKind::DealsCombatDamageToPlayer
            && let Some(atk_ctrl) = attacker_controller
        {
            for enc in &self.exile {
                if enc.encoded_on == Some(source) {
                    triggers.push((enc.id, Effect::CastFreeParadigmCopy, atk_ctrl));
                }
            }
        }

        for (trig_source, effect, controller) in triggers {
            // Most combat-damage triggers implicitly target the damaged player
            // (drain riders, "that player discards / loses life"). But some
            // target a *graveyard* card instead — Efreet Flamepainter re-casts
            // an instant, Venerable Warsinger reanimates a creature. For those,
            // auto-pick the graveyard target rather than mis-binding slot 0 to
            // the damaged player.
            let target = if effect.prefers_graveyard_target() {
                self.auto_target_for_effect_avoiding(&effect, controller, Some(trig_source))
                    .or(Some(default_target.clone()))
            } else {
                Some(default_target.clone())
            };
            self.stack.push(StackItem::Trigger {
                source: trig_source,
                controller,
                effect: Box::new(effect),
                target,
                mode: None,
                x_value: 0,
                converged_value: 0,
                trigger_source: None,
                mana_spent: 0,
                // CR 119.3 — the damage dealt, so `Value::TriggerEventAmount`
                // riders (Visions of Brutality's "controller loses that much
                // life") scale by the hit.
                event_amount: damage_amount,
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
