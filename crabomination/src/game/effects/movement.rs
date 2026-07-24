//! Helpers that move cards between zones (battlefield ↔ graveyard / hand /
//! library / exile) and apply damage to entities. Called from the resolver
//! `match` arms for `Effect::Move`, `Effect::Destroy`, `Effect::DealDamage`,
//! etc.

use super::{EffectContext, EntityRef};
use crate::card::{CardId, CardInstance, CounterType};
use crate::effect::{LibraryPosition, PlayerRef, ZoneDest};
use crate::game::{GameEvent, GameState, TriggerPush};

impl GameState {
    /// CR 614.9 — if damage aimed at `ent` (a player, or a permanent that
    /// player controls) is covered by a `RedirectDamageToSelf` static
    /// (Palisade Giant), return the redirecting permanent. The redirector
    /// never re-redirects its own damage.
    pub(crate) fn damage_redirect_target(&self, ent: EntityRef) -> Option<crate::card::CardId> {
        use crate::effect::StaticEffect;
        let protected = match ent {
            EntityRef::Player(p) => p,
            EntityRef::Permanent(c) => self.battlefield_find(c)?.controller,
            EntityRef::Card(_) => return None,
        };
        let aimed_at = match ent {
            EntityRef::Permanent(c) => Some(c),
            _ => None,
        };
        // Pariah's Shield — a player's damage is dealt to the equipped
        // creature instead. Only player-directed damage redirects.
        if let EntityRef::Player(p) = ent
            && let Some(cid) = self.battlefield.iter().find_map(|c| {
                (c.controller == p
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(
                            sa.effect,
                            StaticEffect::RedirectControllerDamageToEquippedCreature
                        )
                    }))
                .then_some(c.attached_to)
                .flatten()
            })
        {
            return Some(cid);
        }
        if let Some(cid) = self.battlefield.iter().find_map(|c| {
            (c.controller == protected
                && Some(c.id) != aimed_at
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::RedirectDamageToSelf)))
            .then_some(c.id)
        }) {
            return Some(cid);
        }
        // Gideon's Sacrifice — a one-shot "all damage to you and your
        // permanents this turn goes to the chosen permanent instead", so long
        // as it's still on the battlefield and isn't itself the aimed-at card.
        self.damage_redirect_this_turn.iter().find_map(|(p, to)| {
            (*p == protected && Some(*to) != aimed_at && self.battlefield_find(*to).is_some())
                .then_some(*to)
        })
    }

    /// CR 615.1 / 615.7 / 615.12 — apply prevention shields to a pending
    /// damage event aimed at `ent`. "Prevent all" shields zero the event;
    /// "prevent next N" shields soak up to N and then expire. The whole
    /// step is bypassed while `damage_cant_be_prevented_this_turn` is set.
    /// Emits `GameEvent::DamagePrevented` for the prevented portion
    /// (CR 615.13) and returns the unprevented remainder.
    pub fn apply_prevention_shields(
        &mut self,
        ent: EntityRef,
        amount: u32,
        source: Option<crate::card::CardId>,
        events: &mut Vec<GameEvent>,
    ) -> u32 {
        use crate::game::types::PreventionTarget;
        if self.damage_cant_be_prevented_this_turn || self.damage_cant_be_prevented_now() {
            return amount;
        }
        // CR 615.12 (scoped) — Questing Beast: combat damage dealt by creatures
        // the controller controls can't be prevented. Bypass shields when the
        // damage source is a creature whose controller has the static.
        if let Some(src_id) = source
            && let Some(src) = self.battlefield_find(src_id)
            && src.definition.is_creature()
        {
            let ctrl = src.controller;
            let unpreventable = self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::CombatDamageCantBePrevented)
                        || (c.controller == ctrl
                            && matches!(
                                sa.effect,
                                crate::effect::StaticEffect::ControllerCreaturesCombatDamageCantBePrevented
                            ))
                })
            });
            if unpreventable {
                return amount;
            }
        }
        // CR 615.12 (source-scoped) — Excruciator: damage dealt by this
        // permanent can't be prevented. Keyed on the damage source itself.
        if let Some(src_id) = source
            && self.battlefield_find(src_id).is_some_and(|src| {
                src.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::SourceDamageCantBePrevented)
                })
            })
        {
            return amount;
        }
        // Protection from everything (The One Ring) — all damage to the
        // player is prevented until their next turn.
        if let EntityRef::Player(p) = ent
            && self.players[p].protected_from_everything
        {
            if amount > 0 {
                events.push(GameEvent::DamagePrevented { amount, to_player: Some(p), to_card: None });
            }
            return 0;
        }
        // CR 702.64 — Absorb N on the damaged creature prevents N of this
        // event's damage per instance (each instance applies separately).
        let mut amount = amount;
        if let EntityRef::Permanent(cid) = ent {
            let absorbed: u32 = self
                .computed_permanent(cid)
                .map(|cp| {
                    cp.keywords
                        .iter()
                        .filter_map(|k| match k {
                            crate::card::Keyword::Absorb(n) => Some(*n),
                            _ => None,
                        })
                        .sum()
                })
                .unwrap_or(0);
            let soaked = absorbed.min(amount);
            if soaked > 0 {
                events.push(GameEvent::DamagePrevented {
                    amount: soaked,
                    to_player: None,
                    to_card: Some(cid),
                });
                amount -= soaked;
                if amount == 0 {
                    return 0;
                }
            }
        }
        // "If damage would be dealt to this while it has a [kind] counter,
        // prevent that damage and remove that many counters" (Polukranos,
        // Unchained). Prevents the whole event; removes min(amount, counters).
        if let EntityRef::Permanent(cid) = ent
            && amount > 0
            && let Some(kind) = self.battlefield_find(cid).and_then(|c| {
                c.definition.static_abilities.iter().find_map(|sa| match sa.effect {
                    crate::effect::StaticEffect::PreventDamageByRemovingCounters { kind } => {
                        (c.counter_count(kind) > 0).then_some(kind)
                    }
                    _ => None,
                })
            })
        {
            events.push(GameEvent::DamagePrevented {
                amount,
                to_player: None,
                to_card: Some(cid),
            });
            if let Some(c) = self.battlefield_find_mut(cid) {
                c.remove_counters(kind, amount);
            }
            return 0;
        }
        // Phyrexian Vindicator — "If damage would be dealt to this creature,
        // prevent that damage. When damage is prevented this way, this
        // creature deals that much damage to any other target" (auto-picked).
        if let EntityRef::Permanent(cid) = ent
            && amount > 0
            && self.battlefield_find(cid).is_some_and(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        crate::effect::StaticEffect::PreventDamageToThisRedirect
                    )
                })
            })
        {
            events.push(GameEvent::DamagePrevented {
                amount,
                to_player: None,
                to_card: Some(cid),
            });
            let controller = self.battlefield_find(cid).map(|c| c.controller).unwrap_or(0);
            let redirect = crate::effect::Effect::DealDamage {
                to: crate::effect::shortcut::target_any(),
                amount: crate::effect::Value::Const(amount as i32),
            };
            let target = self.auto_target_for_effect_avoiding(&redirect, controller, Some(cid));
            self.stack.push(
                crate::game::types::TriggerPush::new(cid, controller, redirect)
                    .target(target)
                    .build(),
            );
            return 0;
        }
        if self.prevention_shields.is_empty() {
            return amount;
        }
        let (to_player, to_card, key) = match ent {
            EntityRef::Player(p) => (Some(p), None, PreventionTarget::Player(p)),
            EntityRef::Permanent(c) => (None, Some(c), PreventionTarget::Permanent(c)),
            EntityRef::Card(_) => return amount,
        };
        let mut remaining = amount;
        let mut prevented = 0u32;
        // CR 615.1 — life gained by `gain_life` shields that soak damage.
        let mut life_gain = 0u32;
        // Deflecting Palm — damage soaked by `reflect` shields, dealt to
        // the source's controller after the shield pass (stamped controller
        // as the fallback when the source has left every visible zone).
        let mut reflected = 0u32;
        let mut reflect_ctrl: Option<usize> = None;
        // Carom / Razia — damage soaked by `redirect_to` shields, dealt to a
        // chosen permanent after the shield pass.
        let mut redirected: Vec<(crate::card::CardId, u32)> = Vec::new();
        // One-event (Circle of Protection) shields spent in this event.
        let mut spent_one_event: Vec<usize> = Vec::new();
        // Ria Ivor — (seat, prevented) mite mints owed after the loop.
        let mut mite_mints: Vec<(usize, u32)> = Vec::new();
        // Kill-Suit Cultist — the permanent to destroy after the shield pass
        // when a `destroy` shield soaks this event.
        let mut destroy_after: Option<crate::card::CardId> = None;
        for (i, shield) in self
            .prevention_shields
            .iter_mut()
            .enumerate()
            .filter(|(_, s)| {
                s.target == key && (s.source.is_none() || s.source == source)
            })
        {
            if remaining == 0 {
                break;
            }
            let soak = match shield.remaining {
                // Prevent-all: soak everything; the shield stays for the
                // turn unless it's a one-event shield.
                None => std::mem::take(&mut remaining),
                Some(ref mut n) => {
                    let soak = remaining.min(*n);
                    remaining -= soak;
                    *n -= soak;
                    soak
                }
            };
            if shield.one_event && soak > 0 {
                spent_one_event.push(i);
            }
            if soak > 0 && let Some(seat) = shield.mint_mites_for {
                mite_mints.push((seat, soak));
            }
            prevented += soak;
            if shield.gain_life {
                life_gain += soak;
            }
            if shield.reflect {
                reflected += soak;
                reflect_ctrl = reflect_ctrl.or(shield.source_controller);
            }
            if soak > 0 && let Some(dst) = shield.redirect_to {
                redirected.push((dst, soak));
            }
            if shield.destroy && soak > 0 {
                destroy_after = to_card;
            }
        }
        // Drop spent "next N" shields and used one-event shields.
        let mut idx = 0;
        self.prevention_shields.retain(|s| {
            let spent = s.remaining == Some(0) || spent_one_event.contains(&idx);
            idx += 1;
            !spent
        });
        if prevented > 0 {
            events.push(GameEvent::DamagePrevented { amount: prevented, to_player, to_card });
        }
        if life_gain > 0 && let Some(p) = to_player {
            let applied = self.adjust_life_applied(p, life_gain as i32);
            if applied > 0 {
                events.push(GameEvent::LifeGained { player: p, amount: applied as u32 });
            }
        }
        // Ria Ivor — one Phyrexian Mite per point of damage the shield ate.
        for (seat, n) in mite_mints {
            for _ in 0..n {
                let def = crabomination_base::tokens::token_to_card_definition(
                    &crabomination_base::tokens::phyrexian_mite_token(),
                );
                self.mint_token_onto_battlefield(def, seat, false, events);
            }
        }
        // Kill-Suit Cultist — "destroy that creature instead". The shield
        // both prevented the damage (above) and now destroys the target.
        if let Some(cid) = destroy_after
            && self.battlefield_find(cid).is_some()
        {
            self.destroy_permanent(cid, false, events);
        }
        // Carom / Razia — redirect the soaked damage onto the chosen
        // permanent. Its own event, keyed to the original source so
        // protection/prevention on the new target still applies.
        for (dst, amt) in redirected {
            if self.battlefield_find(dst).is_some() {
                self.deal_damage_to_from(EntityRef::Permanent(dst), amt, source, events);
            }
        }
        // Deflecting Palm's "deals that much damage to that source's
        // controller". The reflected damage is its own event (source: the
        // prevention effect, not the original source), so it can itself be
        // prevented/replaced — bounded because each reflect shield is
        // one-event and already spent.
        if reflected > 0 {
            let ctrl = source
                .and_then(|src| {
                    self.battlefield_find(src)
                        .map(|c| c.controller)
                        .or_else(|| {
                            self.stack.iter().find_map(|si| match si {
                                crate::game::StackItem::Spell { card, caster, .. }
                                    if card.id == src => Some(*caster),
                                _ => None,
                            })
                        })
                        .or_else(|| self.died_card_snapshots.get(&src).map(|c| c.controller))
                })
                .or(reflect_ctrl);
            if let Some(ctrl) = ctrl {
                self.deal_damage_to_from(EntityRef::Player(ctrl), reflected, None, events);
            }
        }
        remaining
    }

    /// Damage delivery with the source's identity threaded through, so
    /// CR 702.90b (Infect) can convert player damage into poison
    /// counters when the source has the Infect keyword. `source` is
    /// the `CardId` of the damaging permanent (typically `ctx.source`).
    /// Combat damage uses a separate path in `combat.rs` that already
    /// honors infect for combat damage.
    pub fn deal_damage_to_from(
        &mut self,
        ent: EntityRef,
        amount: u32,
        source: Option<crate::card::CardId>,
        events: &mut Vec<GameEvent>,
    ) {
        // CR 120.8 — "If a source would deal 0 damage, it does not deal
        // damage at all. That means abilities that trigger on damage
        // being dealt won't trigger. It also means that replacement
        // effects that would increase the damage dealt by that source,
        // or would have that source deal that damage to a different
        // object or player, have no event to replace, so they have no
        // effect." We bail out of the entire damage-delivery sequence
        // when `amount == 0`, so no `GameEvent::DamageDealt`,
        // `LifeLost`, `PoisonAdded`, or `LoyaltyChanged` event is
        // emitted. Damage-watching triggered abilities won't fire on
        // 0-damage events.
        if amount == 0 {
            return;
        }
        // Damage-source attribution: the source permanent's controller, or
        // the resolving spell's caster (mirrors `scale_damage_to`).
        let from_controller = source.and_then(|s| {
            self.battlefield_find(s)
                .map(|c| c.controller)
                .or_else(|| match &self.resolving_source {
                    Some((id, caster, _)) if *id == s => Some(*caster),
                    _ => None,
                })
        });
        // CR 615.7 — "prevent all damage [chosen source] would deal this
        // turn" (Burrenton Forge-Tender), unless prevention is off (615.12).
        if let Some(src) = source
            && !self.damage_cant_be_prevented_this_turn
            && self.damage_prevented_sources.contains(&src)
        {
            return;
        }
        // CR 614.9 — redirect the whole event to a Palisade-Giant-style
        // permanent. One redirect per event (CR 614.5; the flag also stops
        // two redirectors ping-ponging).
        if !self.in_damage_redirect
            && let Some(redirect) = self.damage_redirect_target(ent)
        {
            self.in_damage_redirect = true;
            self.deal_damage_to_from(EntityRef::Permanent(redirect), amount, source, events);
            self.in_damage_redirect = false;
            return;
        }
        // CR 702.16e — protection from the source's color prevents the whole
        // damage event to a permanent (noncombat damage path).
        if let (EntityRef::Permanent(tgt), Some(src)) = (ent, source)
            && self.damage_prevented_by_protection(src, tgt)
        {
            return;
        }
        // CR 615 — Light of Sanction: "prevent all damage to creatures you
        // control by sources you control."
        if let (EntityRef::Permanent(tgt), Some(src)) = (ent, source)
            && self.damage_from_your_source_to_your_creature_prevented(src, tgt)
        {
            return;
        }
        // CR 615 — Indentured Oaf: this source prevents its own damage to
        // creatures of a chosen color.
        if let (EntityRef::Permanent(tgt), Some(src)) = (ent, source)
            && self.source_damage_to_color_prevented(src, tgt)
        {
            return;
        }
        // CR 615 — Iroas-style "prevent all damage to attacking creatures
        // you control".
        if let EntityRef::Permanent(tgt) = ent
            && self.damage_to_attacker_prevented(tgt)
        {
            return;
        }
        // CR 615 — Glacial-Chasm-style "prevent all damage that would be dealt
        // to you" (combat and noncombat alike).
        if let EntityRef::Player(p) = ent
            && self.all_damage_to_player_prevented(p)
        {
            return;
        }
        // CR 702.16j — a player with protection from a card type (Serra's
        // Emissary) takes no damage from a source of that type.
        if let (EntityRef::Player(p), Some(src)) = (ent, source) {
            let types = self.player_protection_card_types(p);
            if !types.is_empty() {
                let src_types = self
                    .computed_permanent(src)
                    .map(|c| c.card_types)
                    .or_else(|| {
                        self.find_card_anywhere(src)
                            .map(|c| c.definition.card_types.clone())
                    })
                    .unwrap_or_default();
                if types.iter().any(|t| src_types.contains(t)) {
                    return;
                }
            }
        }
        // CR 615 — self-static "prevent all damage to this permanent"
        // (Gideon Blackblade during your turn); the combat half is gated in
        // the combat resolver.
        if let EntityRef::Permanent(tgt) = ent
            && self.permanent_prevents_all_damage_to_self(tgt)
        {
            return;
        }
        // CR 615 — Mark-of-Asylum-style "prevent all noncombat damage to
        // creatures you control" (this funnel only carries noncombat damage to
        // permanents; combat damage is marked elsewhere).
        if let EntityRef::Permanent(tgt) = ent
            && self.noncombat_damage_to_creature_prevented(tgt)
        {
            return;
        }
        // CR 615 — The Wanderer: "prevent all noncombat damage to you and other
        // permanents you control" (shields the player and any permanent, not
        // just creatures).
        if self.noncombat_damage_to_you_and_permanents_prevented(ent) {
            return;
        }
        // CR 615 — Emmara-Tandris-style "prevent all damage to creature tokens
        // you control" (both damage paths; combat is gated in combat.rs).
        if let EntityRef::Permanent(tgt) = ent
            && self.all_damage_to_creature_token_prevented(tgt)
        {
            return;
        }
        // CR 614.2 / 614.5 — global damage doubling (Furnace of Rath) then
        // halving (Ghosts of the Innocent), applied before prevention so a
        // shield soaks the already-scaled total (CR 616 lets the affected
        // player order the replacements — double-then-halve is the common
        // pick and keeps the event single-pass here).
        let amount = self.scale_damage_to(source, ent, amount);
        // CR 614.5 — Solphim, Mayhem Dominus: a noncombat-only doubler scoped
        // to "a source you control" hitting an opponent / their permanent.
        // Applied here (not in `scale_damage_to`) so combat damage is exempt.
        let amount = {
            let n = self.noncombat_damage_doublers_for(source, ent);
            amount.saturating_mul(1 << n.min(16))
        };
        // CR 614 — Phytohydra: "If damage would be dealt to this creature, put
        // that many +1/+1 counters on it instead." A replacement (not
        // prevention), so it fires even when damage can't be prevented; grows
        // by the full scaled amount. Combat damage is replaced on the combat
        // path (`ironscale_replace`).
        if let EntityRef::Permanent(tgt) = ent
            && self.creature_replaces_damage_with_counters(tgt)
        {
            if let Some(c) = self.battlefield_find_mut(tgt) {
                c.add_counters(crate::card::CounterType::PlusOnePlusOne, amount);
            }
            events.push(GameEvent::CounterAdded {
                card_id: tgt,
                counter_type: crate::card::CounterType::PlusOnePlusOne,
                count: amount,
            });
            return;
        }
        // CR 615 — Stormwild Capridor: "If noncombat damage would be dealt
        // to this creature, prevent that damage. Put a +1/+1 counter on it
        // for each 1 damage prevented this way." Applied after scaling so
        // the counter count matches the damage that WOULD have been dealt;
        // skipped when prevention is off (CR 615.12).
        if let EntityRef::Permanent(tgt) = ent
            && !self.damage_cant_be_prevented_this_turn
            && self
                .battlefield_find(tgt)
                .is_some_and(|c| c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        crate::effect::StaticEffect::PreventNoncombatDamageToSelfAddCounters
                    )
                }))
        {
            if let Some(c) = self.battlefield_find_mut(tgt) {
                c.add_counters(crate::card::CounterType::PlusOnePlusOne, amount);
            }
            events.push(GameEvent::CounterAdded {
                card_id: tgt,
                counter_type: crate::card::CounterType::PlusOnePlusOne,
                count: amount,
            });
            return;
        }
        // "…deals that much damage plus N instead" (Aether Revolt) — additive,
        // opponent-scoped, applied after the doublers. Only when damage is
        // actually being dealt (amount > 0), so a 0 stays 0.
        let amount = if amount > 0 {
            amount.saturating_add(self.noncombat_damage_bonus_for(source, ent))
        } else {
            amount
        };
        // CR 615.1 — prevention shields. Before applying the damage, let
        // any shield around the target soak it (unless a "damage can't be
        // prevented this turn" effect is active, CR 615.12). Returns the
        // unprevented remainder; 0 means the whole event is prevented.
        let amount = self.apply_prevention_shields(ent, amount, source, events);
        if amount == 0 {
            return;
        }
        // CR 702.90b — damage dealt to a player by a source with infect
        // doesn't cause life loss; it gives the player poison counters
        // equal to that damage. We check the source's effective
        // keywords via `computed_permanent` so layered grants (e.g.
        // Triumph of the Hordes-style anthems) are honored.
        let src_kws: Vec<crate::card::Keyword> = source
            .and_then(|s| self.computed_permanent(s))
            .map(|cp| cp.keywords.clone())
            .unwrap_or_default();
        let source_has_infect = src_kws.contains(&crate::card::Keyword::Infect);
        // CR 702.80a / 702.90e — wither/infect damage to a creature lands as
        // -1/-1 counters instead of marked damage; CR 702.2c — nonzero
        // deathtouch damage flags the creature for the destroy SBA.
        let source_has_wither =
            source_has_infect || src_kws.contains(&crate::card::Keyword::Wither);
        let source_has_deathtouch = src_kws.contains(&crate::card::Keyword::Deathtouch);
        // Kumano / Frostwielder: a creature this source damages is exiled
        // instead of dying for the rest of the turn (source-bound CR 614
        // replacement). Registered after the damage lands below.
        let source_exiles_damaged = source
            .and_then(|s| self.battlefield_find(s))
            .map(|c| c.definition.damage_exiles_if_dies)
            .unwrap_or(false);
        match ent {
            EntityRef::Player(p) => {
                // Bloodthirst (CR 702.54) window: any damage to a player
                // (combat or not, incl. infect→poison) marks them damaged
                // this turn.
                self.players[p].was_dealt_damage_this_turn = true;
                // Record the damaging creature so "destroy target creature
                // that dealt damage to you this turn" (Spear of Heliod) can
                // filter targets. Only track battlefield creatures.
                if let Some(src) = source {
                    let is_creature = self
                        .computed_permanent(src)
                        .map(|cp| cp.card_types.contains(&crate::card::CardType::Creature))
                        .unwrap_or(false);
                    if is_creature && !self.players[p].creatures_that_damaged_me_this_turn.contains(&src) {
                        self.players[p].creatures_that_damaged_me_this_turn.push(src);
                    }
                }
                // Phyrexian Unlife — at ≤ 0 life all damage lands as poison.
                let unlife_infect = self.players[p].life <= 0 && self.player_unlife_active(p);
                if source_has_infect || unlife_infect {
                    self.players[p].poison_counters =
                        self.players[p].poison_counters.saturating_add(amount);
                    events.push(GameEvent::PoisonAdded { player: p, amount });
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: Some(p),
                        to_card: None,
                        combat: false,
                        from_controller,
                    });
                } else {
                    // Angel's Grace / Worship — the damage is dealt in full
                    // (the event below carries `amount`), but the life
                    // reduction is clamped to the floor.
                    let life_delta = self.clamp_damage_to_life_floor(p, amount);
                    let applied = self.adjust_life_applied(p, -(life_delta as i32));
                    events.push(GameEvent::DamageDealt { amount, to_player: Some(p), to_card: None, combat: false, from_controller });
                    let lost = (-applied).max(0) as u32;
                    if lost > 0 {
                        events.push(GameEvent::LifeLost { player: p, amount: lost });
                    }
                }
                // Phase M: direct damage from a commander source also
                // counts toward the 21-commander-damage SBA
                // (CR 704.5v doesn't restrict the damage type — combat
                // and non-combat both apply).
                if let Some(src) = source
                    && self.is_commander(src)
                {
                    self.record_commander_damage(p, src, amount);
                }
            }
            EntityRef::Permanent(cid) => {
                // CR 122.1c — Shield counters: if damage would be dealt
                // to this permanent, prevent that damage and remove a
                // shield counter from it.
                let has_shield = self
                    .battlefield_find(cid)
                    .map(|c| c.counter_count(CounterType::Shield) > 0)
                    .unwrap_or(false);
                if has_shield {
                    if let Some(c) = self.battlefield_find_mut(cid) {
                        c.remove_counters(CounterType::Shield, 1);
                        // No 0-count residue (CR 700.9 IsModified).
                        if c.counter_count(CounterType::Shield) == 0 {
                            c.counters.remove(&CounterType::Shield);
                        }
                    }
                    return;
                }
                // CR 120.3c — damage dealt to a planeswalker causes that
                // many loyalty counters to be removed from that
                // planeswalker. Before this branch, non-combat
                // `Effect::DealDamage` was marking the damage on `c.damage`
                // regardless of card type, so a Lightning Bolt at a 3-loyalty
                // PW correctly removed 3 damage to be applied to toughness
                // (toughness = 0 → die!) but skipped the printed
                // loyalty-loss path. Combat damage already routes through
                // `combat.rs::AttackTarget::Planeswalker` which decrements
                // loyalty — this aligns spell damage with the same rule.
                let is_pw = self
                    .battlefield_find(cid)
                    .map(|c| c.definition.is_planeswalker())
                    .unwrap_or(false);
                // CR 310.10 — damage dealt to a battle removes that many
                // defense counters (the noncombat analogue of the combat path
                // in `combat.rs`; a battle isn't a creature, so without this it
                // would mark useless `c.damage`). The defeat trigger fires from
                // the SBA once the last counter is gone.
                let is_battle = self
                    .battlefield_find(cid)
                    .map(|c| c.definition.is_battle())
                    .unwrap_or(false);
                if is_pw {
                    if let Some(c) = self.battlefield_find_mut(cid) {
                        let current = c.counter_count(CounterType::Loyalty);
                        let new_loyalty = current.saturating_sub(amount);
                        c.counters
                            .insert(CounterType::Loyalty, new_loyalty);
                        events.push(GameEvent::DamageDealt {
                            amount,
                            to_player: None,
                            to_card: Some(cid),
                            combat: false,
                            from_controller,
                        });
                        events.push(GameEvent::LoyaltyChanged {
                            card_id: cid,
                            new_loyalty: new_loyalty as i32,
                        });
                        let removed = current.saturating_sub(new_loyalty);
                        if removed > 0 {
                            events.push(GameEvent::CounterRemoved {
                                card_id: cid,
                                counter_type: CounterType::Loyalty,
                                count: removed,
                            });
                        }
                    }
                } else if is_battle {
                    if let Some(c) = self.battlefield_find_mut(cid) {
                        let current = c.counter_count(CounterType::Defense);
                        let new_defense = current.saturating_sub(amount);
                        c.counters.insert(CounterType::Defense, new_defense);
                        events.push(GameEvent::DamageDealt {
                            amount,
                            to_player: None,
                            to_card: Some(cid),
                            combat: false,
                            from_controller,
                        });
                    }
                } else {
                    // CR 120.10 — excess damage: amount beyond what's lethal,
                    // accounting for damage already marked. Deathtouch makes any
                    // damage past 1 excess (CR 702.2c). Computed before the
                    // mutable borrow below applies the new damage.
                    if let Some(cp) = self.computed_permanent(cid)
                        && cp.card_types.contains(&crate::card::CardType::Creature)
                    {
                        let prior = self.battlefield_find(cid).map(|c| c.damage).unwrap_or(0);
                        let lethal_needed = if source_has_deathtouch {
                            1
                        } else {
                            (cp.toughness.max(0) as u32).saturating_sub(prior)
                        };
                        let excess = amount.saturating_sub(lethal_needed);
                        self.excess_damage_this_resolution =
                            self.excess_damage_this_resolution.saturating_add(excess);
                    }
                    if let Some(c) = self.battlefield_find_mut(cid) {
                    if c.definition.is_creature() {
                        c.dealt_damage_this_turn = true;
                        if let Some(src) = source {
                            c.damaged_by_this_turn.push(src);
                        }
                    }
                    if source_has_wither && c.definition.is_creature() {
                        c.add_counters(CounterType::MinusOneMinusOne, amount);
                        events.push(GameEvent::CounterAdded {
                            card_id: cid,
                            counter_type: CounterType::MinusOneMinusOne,
                            count: amount,
                        });
                    } else {
                        c.damage += amount;
                        if source_has_deathtouch && c.definition.is_creature() {
                            c.dealt_deathtouch_damage = true;
                        }
                    }
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: None,
                        to_card: Some(cid),
                        combat: false,
                        from_controller,
                    });
                    let is_creature = c.definition.is_creature();
                    if source_exiles_damaged && is_creature {
                        self.dies_to_exile_eot.insert(cid);
                    }
                    }
                }
            }
            EntityRef::Card(_) => {}
        }
        // Record "damaged this way" for `Selector::DamagedThisResolution`
        // (Aurelia's Fury). Reached only after every prevention / shield /
        // replacement early-return, so it captures creatures and players that
        // truly took damage. Planeswalkers/battles are excluded (only combat-
        // relevant creatures + players are ever queried).
        match ent {
            EntityRef::Player(_) => self.damaged_this_resolution.push(ent),
            EntityRef::Permanent(cid)
                if self
                    .battlefield_find(cid)
                    .is_some_and(|c| c.definition.is_creature()) =>
            {
                self.damaged_this_resolution.push(ent)
            }
            _ => {}
        }
        // CR 702.15 — lifelink on the non-combat damage path: if the source is
        // a lifelink permanent (a ping ability) or an instant/sorcery spell
        // whose controller has "your spells have lifelink" (Radiant
        // Scrollwielder), that controller gains life equal to the damage dealt.
        // (Combat damage handles its own lifelink in `combat.rs`.)
        if let Some(seat) = self.noncombat_lifelink_seat(source) {
            let applied = self.adjust_life_applied(seat, amount as i32);
            if applied > 0 {
                events.push(GameEvent::LifeGained { player: seat, amount: applied as u32 });
            }
        }
        // CR 603.4 — "whenever [this creature] deals damage this turn" delayed
        // triggers watching the noncombat source (Paladin of Prahv's Forecast).
        if let Some(src) = source {
            self.fire_source_dealt_damage_watchers(src, amount);
        }
        // "Whenever an instant or sorcery spell you control deals damage"
        // (Blaze Commando). Fires once per resolution across a multi-hit spell.
        self.fire_your_spell_dealt_damage(source, amount);
    }

    /// Fire `EventKind::YourInstantOrSorceryDealtDamage` for the caster of the
    /// instant/sorcery currently resolving, once per resolution (Blaze Commando).
    /// `source` is the damage source (the spell); we only fire when it is the
    /// resolving spell so combat/ability damage during the same window is
    /// ignored.
    fn fire_your_spell_dealt_damage(&mut self, source: Option<crate::card::CardId>, amount: u32) {
        use crate::effect::{EventKind, EventScope};
        if self.spell_damage_trigger_fired || amount == 0 {
            return;
        }
        let Some(seat) = self.resolving_spell_caster else { return };
        // The damage must be dealt by the resolving spell itself.
        if let (Some(src), Some((res_id, _, _))) = (source, &self.resolving_source)
            && src != *res_id
        {
            return;
        }
        let listeners: Vec<(crate::card::CardId, crate::effect::Effect, usize)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .flat_map(|c| {
                c.definition
                    .triggered_abilities
                    .iter()
                    .filter(|ta| {
                        ta.event.kind == EventKind::YourInstantOrSorceryDealtDamage
                            && ta.event.scope == EventScope::YourControl
                    })
                    .map(move |ta| (c.id, ta.effect.clone(), c.controller))
            })
            .collect();
        if listeners.is_empty() {
            return;
        }
        self.spell_damage_trigger_fired = true;
        for (listener, effect, controller) in listeners {
            let auto_target = self.auto_target_for_effect_avoiding(&effect, controller, Some(listener));
            self.stack.push(
                TriggerPush::new(listener, controller, effect)
                    .target(auto_target)
                    .event_amount(amount)
                    .build(),
            );
        }
    }

    /// Seat that gains life from lifelink on a *non-combat* damage event from
    /// `source`, if any (CR 702.15). Returns the source's controller when the
    /// source is a lifelink permanent, or the caster of an instant/sorcery
    /// spell whose controller grants spells lifelink.
    fn noncombat_lifelink_seat(&self, source: Option<crate::card::CardId>) -> Option<usize> {
        use crate::card::Keyword;
        // A lifelink permanent (e.g. a ping ability from a lifelink creature).
        if let Some(src) = source
            && let Some(cp) = self.computed_permanent(src)
            && cp.keywords.contains(&Keyword::Lifelink)
        {
            return Some(cp.controller);
        }
        // The currently-resolving instant/sorcery whose controller grants
        // spells lifelink (stamped in `resolve_top_of_stack`).
        self.resolving_spell_lifelink_seat
    }

    /// True if `seat` controls a permanent granting
    /// `StaticEffect::YourInstantSorcerySpellsHaveLifelink` (Radiant Scrollwielder).
    pub(crate) fn controller_grants_spell_lifelink(&self, seat: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == seat
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::YourInstantSorcerySpellsHaveLifelink)
                })
        })
    }

    /// CR 701.34 — manifest the card `cid` (in player `p`'s library): flip it
    /// face down in place so it enters as a vanilla 2/2 (no real-card ETB
    /// triggers), then put it onto the battlefield under `p`'s control.
    pub fn manifest_card(
        &mut self,
        cid: CardId,
        p: usize,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        let pl = &mut self.players[p];
        if let Some(c) = pl
            .library
            .iter_mut()
            .chain(pl.hand.iter_mut())
            .find(|c| c.id == cid)
        {
            c.turn_face_down();
        }
        let dest = ZoneDest::Battlefield {
            controller: crate::effect::PlayerRef::Seat(p),
            tapped: false,
        };
        self.move_card_to(cid, &dest, ctx, events);
    }

    pub fn move_card_to(
        &mut self,
        cid: CardId,
        dest: &ZoneDest,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        // Grafdigger's Cage / Soulless Jailer — locked cards in graveyards
        // and libraries can't enter the battlefield.
        if matches!(dest, ZoneDest::Battlefield { .. }) {
            use crate::card::Zone;
            let blocked = self.players.iter().any(|pl| {
                pl.graveyard
                    .iter()
                    .map(|c| (c, Zone::Graveyard))
                    .chain(pl.library.iter().map(|c| (c, Zone::Library)))
                    .any(|(c, zone)| {
                        c.id == cid && self.battlefield_entry_from_zone_blocked(&c.definition, zone)
                    })
            });
            if blocked {
                return;
            }
        }
        // Resolve any selector-based player refs in the destination *now*,
        // while the card is still findable in its source zone — otherwise
        // `PlayerRef::OwnerOf(Target(0))` can't see the card after we remove
        // it. The resolved dest uses concrete `PlayerRef::You`-anchored refs.
        let resolved_dest = self.resolve_zonedest_player(dest, ctx);

        // Try battlefield first.
        if let Some(pos) = self.battlefield.iter().position(|c| c.id == cid) {
            let mut card = self.battlefield.remove(pos);
            self.remove_effects_from_source(cid);
            self.collect_leaver_counters(&card);
            // CR 708.10 — a face-down permanent is turned face up as it leaves
            // the battlefield (no-op unless it carries a stashed real def).
            card.turn_face_up();
            // CR 709.5c — Room unlocked designations are battlefield-only.
            card.reset_room_doors();
            // MKM — a Case's solved designation is battlefield-only.
            card.reset_case();
            // CR 716.2 — a Class's level is battlefield-only.
            card.reset_class_level();
            // CR 707 — a temporary copy reverts as it leaves.
            self.revert_copy_on_leave(&mut card);
            card.damage = 0;
            card.tapped = false;
            card.attached_to = None;
            // CR 506.4 — A permanent leaving the battlefield is removed
            // from combat. The helper prunes `self.attacking` and
            // `self.block_map` so the post-move combat state stays
            // consistent for downstream selectors and trigger dispatchers.
            self.remove_from_combat(cid);
            // CR 603.6 — note a creature leaving for a non-graveyard zone
            // (bounce / exile / library) *without dying* before it's consumed.
            let leaver = (card.definition.is_creature() && !matches!(resolved_dest, ZoneDest::Graveyard))
                .then_some((card.id, card.controller));
            // EOE Void — any nonland permanent leaving the battlefield (bounce,
            // sacrifice, exile, …) latches the turn-wide flag.
            if !card.definition.is_land() {
                self.nonland_permanent_left_bf_this_turn = true;
            }
            // Second Sunrise's restore set — battlefield → graveyard moves.
            if matches!(resolved_dest, ZoneDest::Graveyard) {
                self.graveyard_from_battlefield_this_turn.insert(cid);
            }
            self.place_card_in_dest(card, ctx.controller, &resolved_dest, events);
            self.on_left_battlefield(cid, events);
            if let Some((card_id, controller)) = leaver {
                events.push(GameEvent::CreatureLeftWithoutDying { card_id, controller });
            }
            return;
        }
        // Then graveyards. Emit `CardLeftGraveyard` so Strixhaven
        // "cards leave your graveyard" payoffs (Garrison Excavator,
        // Living History, Spirit Mascot, Hardened Academic) trigger.
        for p in 0..self.players.len() {
            if let Some(pos) = self.players[p].graveyard.iter().position(|c| c.id == cid) {
                let card = self.players[p].graveyard.remove(pos);
                self.players[p].cards_left_graveyard_this_turn =
                    self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
                events.push(GameEvent::CardLeftGraveyard { player: p, card_id: cid });
                // Prized Amalgam's gate — record gy→battlefield entries.
                if matches!(resolved_dest, ZoneDest::Battlefield { .. }) {
                    self.entered_from_graveyard_this_turn.insert(cid);
                }
                // "When this card is put into your hand from your graveyard"
                // (Golgari Brownscale). Emitted for any graveyard→hand return.
                if matches!(resolved_dest, ZoneDest::Hand(_)) {
                    events.push(GameEvent::CardPutIntoHandFromGraveyard { player: p, card_id: cid });
                }
                self.place_card_in_dest(card, p, &resolved_dest, events);
                return;
            }
        }
        // Then exile.
        if let Some(pos) = self.exile.iter().position(|c| c.id == cid) {
            let card = self.exile.remove(pos);
            let owner = card.owner;
            // Fire Lord Zuko's gate — record exile→battlefield entries so a
            // "whenever a permanent you control enters from exile" trigger can
            // distinguish them from cast / reanimate-from-graveyard entries.
            if matches!(resolved_dest, ZoneDest::Battlefield { .. }) {
                self.entered_from_exile_this_turn.insert(cid);
            }
            self.place_card_in_dest(card, owner, &resolved_dest, events);
            return;
        }
        // Hands. Used by start-of-game opening-hand effects
        // (Leyline of Sanctity, Gemstone Caverns) that move a hand card
        // to the battlefield.
        for p in 0..self.players.len() {
            if let Some(pos) = self.players[p].hand.iter().position(|c| c.id == cid) {
                let card = self.players[p].hand.remove(pos);
                self.place_card_in_dest(card, p, &resolved_dest, events);
                return;
            }
        }
        // Libraries. Used by `Selector::TopOfLibrary` → `ZoneDest::Exile`
        // / `Hand` / etc. (Suspend Aggression's exile-top-of-library half,
        // Daydream's exile-then-return flicker pattern in passing).
        for p in 0..self.players.len() {
            if let Some(pos) = self.players[p].library.iter().position(|c| c.id == cid) {
                let card = self.players[p].library.remove(pos);
                self.place_card_in_dest(card, p, &resolved_dest, events);
                return;
            }
        }
    }

    /// Pre-resolve any selector-based player refs in a `ZoneDest` against
    /// the active ctx. `place_card_in_dest` constructs its own bare ctx and
    /// can't see the caster's targets, so any `PlayerRef::OwnerOf(Selector)`
    /// / `ControllerOf(Selector)` need to be flattened to a concrete
    /// `PlayerRef::Seat(n)` while the source card is still in its origin
    /// zone. Other ref kinds (You / ActivePlayer / etc.) pass through.
    pub(super) fn resolve_zonedest_player(&self, dest: &ZoneDest, ctx: &EffectContext) -> ZoneDest {
        let flatten = |who: &PlayerRef| -> PlayerRef {
            match who {
                PlayerRef::OwnerOf(_) | PlayerRef::ControllerOf(_) => {
                    if let Some(p) = self.resolve_player(who, ctx) {
                        PlayerRef::Seat(p)
                    } else {
                        who.clone()
                    }
                }
                // Flatten `PlayerRef::You` to the caster's seat now —
                // `place_card_in_dest` builds its own context anchored to
                // the card's *origin owner* (which is the graveyard owner
                // for gy-to-bf moves like Mind Roots, not the caster). If
                // we don't flatten here, "controller: PlayerRef::You" on a
                // ZoneDest::Battlefield would end up resolving to the
                // graveyard's owner instead of the caster, putting the
                // stolen land back under the opp's control.
                PlayerRef::You => PlayerRef::Seat(ctx.controller),
                _ => who.clone(),
            }
        };
        match dest {
            ZoneDest::Hand(who) => ZoneDest::Hand(flatten(who)),
            ZoneDest::Library { who, pos } => ZoneDest::Library {
                who: flatten(who),
                pos: *pos,
            },
            ZoneDest::Battlefield { controller, tapped } => ZoneDest::Battlefield {
                controller: flatten(controller),
                tapped: *tapped,
            },
            ZoneDest::Graveyard | ZoneDest::Exile => dest.clone(),
        }
    }

    pub(crate) fn place_card_in_dest(
        &mut self,
        mut card: CardInstance,
        default_player: usize,
        dest: &ZoneDest,
        events: &mut Vec<GameEvent>,
    ) {
        // Phase H — consult the replacement-effect registry. The
        // resolver only sees the *destination kind* (a `Zone`); the
        // origin is left unconstrained here (passed as
        // `Zone::Battlefield` for now, which covers the Commander
        // case since its replacement effect uses `from: None`).
        // If the resolver redirects to a different zone, we hand off
        // to `place_card_at_resolved_zone` which handles the
        // terminal-zone placement uniformly. Same-zone return falls
        // through to the existing rich `ZoneDest` logic so player /
        // tapped / library-position information is preserved.
        let intended = match dest {
            ZoneDest::Hand(_) => crate::card::Zone::Hand,
            ZoneDest::Library { .. } => crate::card::Zone::Library,
            ZoneDest::Battlefield { .. } => crate::card::Zone::Battlefield,
            ZoneDest::Graveyard => crate::card::Zone::Graveyard,
            ZoneDest::Exile => crate::card::Zone::Exile,
        };
        // CR 702.47e — a spell loses its splice changes once it leaves the
        // stack for any reason.
        card.spliced_effects.clear();
        // CR 712.16/712.17 — a melded permanent leaving the battlefield
        // leaves as its two component cards; the melded shell ceases to
        // exist.
        if !card.meld_parts.is_empty() && intended != crate::card::Zone::Battlefield {
            for part in std::mem::take(&mut card.meld_parts) {
                self.place_card_in_dest(part, default_player, dest, events);
            }
            return;
        }
        // CR 702.140e — a merged (mutated) permanent leaving the battlefield
        // becomes its individual component cards in that zone.
        if !card.mutate_stack.is_empty() && intended != crate::card::Zone::Battlefield {
            for part in std::mem::take(&mut card.mutate_stack) {
                self.place_card_in_dest(part, default_player, dest, events);
            }
            return;
        }
        let resolved = self.resolve_zone_change(
            card.id,
            crate::card::Zone::Battlefield,
            intended,
        );
        if resolved != intended {
            self.place_card_at_resolved_zone(card, resolved);
            return;
        }
        // CR 122.2 — counters are not retained when an object changes
        // zones; they cease to exist. Cleared for every destination (the
        // battlefield arm additionally re-seeds planeswalker loyalty).
        // Dies-with-counters triggers read the `died_card_snapshots` /
        // `leaves_bf_lki` LKI caches, not the new zone's object.
        card.counters.clear();
        card.keyword_counters.clear();
        match dest {
            ZoneDest::Hand(who) => {
                let ctx = EffectContext::for_spell(default_player, None, 0, 0);
                // `OwnerOfMoved` routes the card to *its own* owner (per-card
                // board-bounce — Aetherize / Evacuation).
                let p = match who {
                    PlayerRef::OwnerOfMoved => card.owner,
                    _ => self.resolve_player(who, &ctx).unwrap_or(default_player),
                };
                card.controller = p;
                self.players[p].hand.push(card);
            }
            ZoneDest::Library { who, pos } => {
                let ctx = EffectContext::for_spell(default_player, None, 0, 0);
                let p = match who {
                    PlayerRef::OwnerOfMoved => card.owner,
                    _ => self.resolve_player(who, &ctx).unwrap_or(default_player),
                };
                match pos {
                    LibraryPosition::Top => self.players[p].library.insert(0, card),
                    LibraryPosition::Bottom => self.players[p].library.push(card),
                    LibraryPosition::OwnerChoice => {
                        // CR 701: "owner's choice" library placement.
                        // Ask the *owner* of the moved card (= the
                        // library we're putting it in — `p` resolved
                        // above) yes/no via `Decision::OptionalTrigger`.
                        // True = top, false = bottom. AutoDecider
                        // defaults to false (bottom). Run Behind is the
                        // only printed user today.
                        let decision = crate::decision::Decision::OptionalTrigger {
                            source: card.id,
                            description: "Put on top of library? (no = bottom)".into(),
                        };
                        let answer = self.decider.decide(&decision);
                        let put_on_top = matches!(
                            answer,
                            crate::decision::DecisionAnswer::Bool(true)
                        );
                        if put_on_top {
                            self.players[p].library.insert(0, card);
                        } else {
                            self.players[p].library.push(card);
                        }
                    }
                    LibraryPosition::SecondFromTopOrBottom => {
                        // Deem Inferior — owner picks second-from-top or
                        // bottom. Yes = second from top; no/default = bottom.
                        let decision = crate::decision::Decision::OptionalTrigger {
                            source: card.id,
                            description: "Put second from the top of library? (no = bottom)".into(),
                        };
                        let second_from_top = matches!(
                            self.decider.decide(&decision),
                            crate::decision::DecisionAnswer::Bool(true)
                        );
                        if second_from_top && !self.players[p].library.is_empty() {
                            self.players[p].library.insert(1, card);
                        } else {
                            self.players[p].library.push(card);
                        }
                    }
                    LibraryPosition::Shuffled => {
                        // Push the card in, then shuffle the entire library
                        // so the card lands at a random position (Chaos Warp,
                        // bottom-of-library reanimate-prevention effects, etc.).
                        // Pre-fix this fell through to `push` (effectively
                        // sending to bottom), which exposed deterministic
                        // ordering across cards that semantically should
                        // randomize.
                        use rand::seq::SliceRandom;
                        let mut rng = rand::rng();
                        self.players[p].library.push(card);
                        self.players[p].library.shuffle(&mut rng);
                    }
                    LibraryPosition::FromTop(n) => {
                        // CR 401.7: "If a player is instructed to put a
                        // card 'Nth from the top' of a library, and there
                        // are fewer than N cards in that library, the
                        // card is put on the bottom of that library."
                        // `FromTop(0)` = top; otherwise insert at index
                        // `n` if the library has at least `n` cards,
                        // else `push` (= bottom).
                        let lib_len = self.players[p].library.len();
                        if *n >= lib_len {
                            self.players[p].library.push(card);
                        } else {
                            self.players[p].library.insert(*n, card);
                        }
                    }
                }
            }
            ZoneDest::Graveyard => {
                // CR 614.6 — graveyard-hate statics redirect to exile.
                self.route_to_graveyard(card, events);
            }
            ZoneDest::Exile => {
                let cid = card.id;
                self.exile.push(card);
                // Record for `Selector::ExiledThisResolution` ("if you exiled
                // a [type] card this way" — Bonehoard Dracosaur).
                self.exiled_card_ids_this_resolution.push(cid);
                // Bump the controller-of-the-exile-effect's per-turn
                // exile tally for Strixhaven "if one or more cards were
                // put into exile this turn" payoffs (Ennis the Debate
                // Moderator). Reset on `do_untap`.
                if default_player < self.players.len() {
                    self.players[default_player].cards_exiled_this_turn =
                        self.players[default_player].cards_exiled_this_turn.saturating_add(1);
                }
                events.push(GameEvent::PermanentExiled { card_id: cid });
            }
            ZoneDest::Battlefield { controller, tapped } => {
                // CR 614.x — Containment Priest. A nontoken creature put onto
                // the battlefield without being cast (reanimate / blink /
                // reveal-and-put — anything routed through this path rather
                // than `resolve_spell`) is exiled instead.
                if !card.is_token
                    && card.definition.is_creature()
                    && self.nontoken_creature_etb_exile_active()
                {
                    let cid = card.id;
                    self.exile.push(card);
                    events.push(GameEvent::PermanentExiled { card_id: cid });
                    return;
                }
                let ctx = EffectContext::for_spell(default_player, None, 0, 0);
                let p = self.resolve_player(controller, &ctx).unwrap_or(default_player);
                card.controller = p;
                card.tapped = *tapped;
                card.summoning_sick = card.definition.is_creature();
                // CR 603.4 — stamp the entry turn immediately (the central
                // dispatch loop also sets it, but doing it here makes it
                // readable mid-resolution — Emergent Sequence counts the land
                // it just put in). Same for the CR 613.7d object timestamp.
                card.entered_turn = Some(self.turn_number);
                card.battlefield_timestamp = self.next_timestamp();
                if card.definition.is_creature() {
                    self.players[p].creatures_entered_this_turn.push(card.id);
                }
                if card.definition.is_artifact() {
                    self.players[p].artifacts_entered_this_turn += 1;
                }
                if !card.definition.is_land() {
                    self.players[p].nonland_permanents_entered_this_turn += 1;
                }
                if card.definition.has_creature_type(crate::card::CreatureType::Mount)
                    || card.definition.is_vehicle()
                {
                    self.players[p].mounts_vehicles_entered_this_turn += 1;
                }
                // A permanent entering the battlefield from another zone is
                // a brand-new object (rule 400.7) — clear residual damage,
                // pump bonuses, and attachment.
                card.damage = 0;
                card.power_bonus = 0;
                card.toughness_bonus = 0;
                card.perm_power_bonus = 0;
                card.perm_toughness_bonus = 0;
                card.attached_to = None;
                // CR 400.7 — a fresh object isn't saddled and remembers no
                // riders (Fortune, Loyal Steed returning after its own blink).
                card.saddled = false;
                card.saddled_by.clear();
                card.crewed_by.clear();
                // Not a cast: reanimation / blink / put-onto-battlefield clears
                // the "if you cast it" flag (CR 400.7 new object).
                card.entered_by_cast = false;
                // CR 702.29 — a fresh battlefield object owes echo again.
                card.echo_paid = false;
                // CR 122.2 cleared the counters above; re-seed a
                // planeswalker's starting loyalty (CR 306.5b) so a reanimated
                // / blinked planeswalker enters with full base loyalty rather
                // than its last-known (possibly 0) value.
                if card.definition.is_planeswalker() && card.definition.base_loyalty > 0 {
                    // CR 702.150c — a Compleated planeswalker cast with life
                    // enters with two fewer loyalty counters per {C/P} paid
                    // with life (2 life each, so the reduction = life paid).
                    let loyalty = card
                        .definition
                        .base_loyalty
                        .saturating_sub(card.compleated_life_paid);
                    card.compleated_life_paid = 0;
                    if loyalty > 0 {
                        card.counters.insert(CounterType::Loyalty, loyalty);
                    } else {
                        card.counters.remove(&CounterType::Loyalty);
                    }
                }
                // CR 310.7 — a Battle enters with defense counters equal to its
                // printed defense, and (CR 310.6) its controller chooses an
                // opponent to protect it. In 2-player there is a single
                // opponent; multiplayer protector choice is a follow-up.
                if card.definition.is_battle() {
                    if card.definition.defense > 0 {
                        card.counters
                            .insert(CounterType::Defense, card.definition.defense);
                    }
                    if card.protected_by.is_none() {
                        let ctrl = card.controller;
                        card.protected_by = (0..self.players.len())
                            .find(|&p| p != ctrl && self.players[p].is_alive());
                    }
                }
                let cid = card.id;
                // CR 614.12 — apply "enters with N counters" replacement
                // BEFORE the new permanent is exposed to state-based-action
                // sweeps and BEFORE ETB triggers fire. This lets a printed
                // 0/0 or 1/0 body (Pterafractyl, Symmathematics) survive
                // without the historic base-toughness bump workaround. The
                // Value is evaluated against a self-ability ctx anchored
                // to the new permanent's `CardId` so `Value::XFromCost`
                // reads via a `for_ability` shim — for spells using
                // `Value::Const(N)` (Symmathematics) this is exact; for
                // X-on-cast bodies (Pterafractyl) the x_value would need
                // additional plumbing through `move_card_to` from the
                // cast-time ctx, tracked separately.
                let enters_spec = card.definition.enters_with_counters.clone();
                // CR 614.12 — "enters with N counters if you cast it from your
                // hand" (Patched Plaything) reads the resolving card's cast
                // zone via `Predicate::CastFromHand`. Capture it before the
                // instance is moved onto the battlefield.
                let entered_from_hand = card.cast_from_hand;
                let mut card = card;
                card.controller = self.apply_etb_control_replacement(&card, card.controller);
                // CR 716.2 — a Class enters the battlefield at level 1.
                if card.definition.is_class() {
                    card.class_level = 1;
                }
                self.battlefield.push(card);
                // CR 122.1 — Solemnity drops the enters-with-counters too.
                let mut counter_specs: Vec<(crate::card::CounterType, crate::effect::Value)> =
                    Vec::new();
                if let Some(spec) = enters_spec {
                    counter_specs.push(spec);
                }
                // Metallic Mimic-style chosen-type ETB counters (any matching
                // creature entry — tokens, reanimation, search-to-battlefield).
                for (kind, n) in self.chosen_type_etb_counter_specs(cid, p) {
                    counter_specs.push((kind, crate::effect::Value::Const(n as i32)));
                }
                if self.counters_locked() { counter_specs.clear(); }
                for (kind, value) in counter_specs {
                    let mut etb_ctx = crate::game::effects::EffectContext::for_ability(cid, p, None);
                    etb_ctx.cast_from_hand = entered_from_hand;
                    let base = self.evaluate_value(&value, &etb_ctx);
                    if base > 0 {
                        // CR 614.16: counter replacement statics also apply
                        // to the "enters with N counters" replacement.
                        let bf = self.battlefield.iter().find(|c| c.id == cid);
                        let n = bf
                            .map(|c| (c.controller, c.definition.is_creature()))
                            .map(|(ctrl, cre)| {
                                self.scaled_counter_count(ctrl, kind, base as u32, cre)
                            })
                            .unwrap_or(base as u32);
                        if let Some(card_mut) =
                            self.battlefield.iter_mut().find(|c| c.id == cid)
                        {
                            card_mut.add_counters(kind, n);
                        }
                        events.push(GameEvent::CounterAdded {
                            card_id: cid,
                            counter_type: kind,
                            count: n,
                        });
                    }
                }
                // CR 702.32 / 702.62 — Fading / Vanishing enter-with-counters.
                self.apply_fading_vanishing_etb(cid, events);
                events.push(GameEvent::PermanentEntered { card_id: cid });
                // Fire self-source ETB triggers so reanimate / flicker /
                // search-to-battlefield paths trigger creature ETBs the same
                // way casting does. CR 603.3d — the trigger's controller is
                // the permanent's controller AFTER any ETB control
                // replacement (Gather Specimens), not the pre-replacement
                // destination seat.
                let etb_ctrl = self
                    .battlefield
                    .iter()
                    .find(|c| c.id == cid)
                    .map(|c| c.controller)
                    .unwrap_or(p);
                self.fire_self_etb_triggers(cid, etb_ctrl);
            }
        }
    }

    /// Shared leaves-battlefield bookkeeping, called from every
    /// battlefield-removal path: CR 603.6e linked-exile returns plus
    /// "when [that permanent] leaves the battlefield" delayed triggers
    /// (`DelayedKind::WhenCardLeavesBattlefield` — Hofri Ghostforge).
    pub(crate) fn on_left_battlefield(&mut self, id: CardId, events: &mut Vec<GameEvent>) {
        self.return_linked_exiles(id, events);
        // CR 702.26 — permanents phased out "until [this] leaves the
        // battlefield" (Out of Time) phase in now.
        let mut i = 0;
        let mut phased_in: Vec<CardId> = Vec::new();
        while i < self.phased_out.len() {
            if self.phased_out[i].phased_out_by == Some(id) {
                let mut c = self.phased_out.remove(i);
                c.phased_out_by = None;
                phased_in.push(c.id);
                self.battlefield.push(c);
            } else {
                i += 1;
            }
        }
        for card_id in phased_in {
            events.push(GameEvent::PermanentPhasedIn { card_id });
        }
        // Source-bound control steals end with their source (Sower of
        // Temptation — CR 800.4 hands the permanent back).
        let mut kept = Vec::new();
        for tc in std::mem::take(&mut self.temporary_control) {
            if tc.source == Some(id) {
                self.change_control(tc.card, tc.original_controller);
            } else {
                kept.push(tc);
            }
        }
        self.temporary_control = kept;
        // CR 400.7 — the card is a new object in its next zone: effects
        // that granted abilities to the permanent don't follow it.
        if let Some(c) = self.find_card_anywhere_mut(id) {
            c.granted_activated_abilities.clear();
        }
        // CR 611.2c — continuous effects aimed at this specific permanent
        // end with it (don't re-attach if the same card re-enters).
        for e in self.continuous_effects.iter_mut() {
            if let crate::game::layers::AffectedPermanents::Specific(ids) = &mut e.affected {
                ids.retain(|cid| *cid != id);
            }
        }
        self.continuous_effects.retain(|e| {
            !matches!(&e.affected,
                crate::game::layers::AffectedPermanents::Specific(ids) if ids.is_empty())
        });
        use crate::game::types::DelayedKind;
        let mut fire: Vec<crate::game::types::DelayedTrigger> = Vec::new();
        self.delayed_triggers.retain(|dt| {
            if dt.kind == DelayedKind::WhenCardLeavesBattlefield(id) {
                fire.push(dt.clone());
                false
            } else {
                true
            }
        });
        for dt in fire {
            self.stack.push(
                TriggerPush::new(dt.source, dt.controller, dt.effect)
                    .target(dt.target)
                    .trigger_source(Some(super::EntityRef::Card(id)))
                    .build(),
            );
        }
    }

    /// CR 603.6e — when a permanent that exiled card(s) via
    /// `Effect::ExileUntilSourceLeaves` leaves the battlefield, return the
    /// linked card(s) to the zone the linking ability specified
    /// (battlefield for Banisher Priest / Oblivion Ring, hand for Brain
    /// Maggot / Tidehollow Sculler). Called from every battlefield-removal
    /// path. The return is resolved directly rather than as a stack
    /// trigger — a deliberate simplification; the observable result (the
    /// card comes back) matches the printed linked ability.
    pub(crate) fn return_linked_exiles(
        &mut self,
        source: CardId,
        events: &mut Vec<GameEvent>,
    ) {
        use crate::card::ExileReturnZone;
        let linked: Vec<CardId> = self
            .exile
            .iter()
            // Monarch-guarded exiles (Palace Jailer) return when the monarchy
            // moves, not when the source leaves — `set_monarch` handles them.
            .filter(|c| {
                c.exiled_by.map(|l| l.source) == Some(source)
                    && c.exiled_by.and_then(|l| l.monarch_guard).is_none()
            })
            .map(|c| c.id)
            .collect();
        for cid in linked {
            let Some(pos) = self.exile.iter().position(|c| c.id == cid) else {
                continue;
            };
            // Skyclave Apparition: the card stays in exile; its owner gets
            // an X/X blue Illusion (X = the card's mana value) instead.
            if self.exile[pos].exiled_by.map(|l| l.return_to)
                == Some(ExileReturnZone::IllusionToken)
            {
                let owner = self.exile[pos].owner;
                let mv = self.exile[pos].definition.cost.cmc() as i32;
                self.exile[pos].exiled_by = None;
                let def = crate::card::CardDefinition {
                    name: "Illusion",
                    cost: crate::mana::ManaCost::default(),
                    card_types: vec![crate::card::CardType::Creature],
                    subtypes: crate::card::Subtypes {
                        creature_types: vec![crate::card::CreatureType::Illusion],
                        ..Default::default()
                    },
                    power: mv,
                    toughness: mv,
                    ..Default::default()
                };
                self.mint_token_onto_battlefield(def, owner, false, events);
                continue;
            }
            let mut card = self.exile.remove(pos);
            let return_to = card.exiled_by.take().map(|l| l.return_to);
            let owner = card.owner;
            let dest = match return_to {
                Some(ExileReturnZone::Hand) => ZoneDest::Hand(PlayerRef::Seat(owner)),
                Some(ExileReturnZone::BattlefieldTapped) => ZoneDest::Battlefield {
                    controller: PlayerRef::Seat(owner),
                    tapped: true,
                },
                _ => ZoneDest::Battlefield {
                    controller: PlayerRef::Seat(owner),
                    tapped: false,
                },
            };
            self.place_card_in_dest(card, owner, &dest, events);
        }
    }

    /// CR 724 — after the monarchy moves to `new_monarch`, return every
    /// monarch-guarded exile (Palace Jailer) whose guard player is no longer
    /// the monarch, to the battlefield under its owner's control.
    pub(crate) fn return_monarch_guarded_exiles(
        &mut self,
        new_monarch: Option<usize>,
        events: &mut Vec<GameEvent>,
    ) {
        let freed: Vec<CardId> = self
            .exile
            .iter()
            .filter(|c| {
                c.exiled_by
                    .and_then(|l| l.monarch_guard)
                    .is_some_and(|guard| new_monarch != Some(guard))
            })
            .map(|c| c.id)
            .collect();
        for cid in freed {
            let Some(pos) = self.exile.iter().position(|c| c.id == cid) else { continue };
            let mut card = self.exile.remove(pos);
            card.exiled_by = None;
            let owner = card.owner;
            let dest = ZoneDest::Battlefield { controller: PlayerRef::Seat(owner), tapped: false };
            self.place_card_in_dest(card, owner, &dest, events);
        }
    }
}
