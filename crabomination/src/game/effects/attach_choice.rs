//! "You may attach to it any number of [Auras / Equipment]" — Bruna, Light of
//! Alabaster and Heavenly Blademaster. The controller picks the subset. And
//! its mirror, "for each one, you may attach it to a creature you control"
//! (Inventory Management): a host per attachment. And "for each of those
//! tokens, you may attach an Equipment you control to it" (Goldwardens'
//! Gambit): an attachment per host.

use super::{EffectContext, PickValue};
use crate::card::CardId;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, Target};

impl GameState {
    /// CR 701.3b — can `att` become attached to `host`? An Equipment only to
    /// a creature (CR 301.5c), a Fortification only to a land (CR 301.6), an
    /// Aura only to what its enchant ability allows (CR 303.4a, a player for
    /// "enchant player"); nothing to itself.
    pub(crate) fn can_become_attached(&self, att: CardId, host: &Target) -> bool {
        let Some(a) = self.battlefield_find(att) else { return false };
        let def = &a.definition;
        let enchant_ok = |host: &Target| {
            def.aura_enchant_filter()
                .is_some_and(|f| self.evaluate_requirement_static(f, host, a.controller, Some(att)))
        };
        match host {
            Target::Permanent(h) if *h != att => {
                let Some(hc) = self.battlefield_find(*h) else { return false };
                if def.is_aura() {
                    if def.aura_enchant_filter().is_some() { enchant_ok(host) } else { self.computed_is_creature(hc) }
                } else if def.is_equipment() {
                    self.computed_is_creature(hc)
                } else if def.subtypes.artifact_subtypes.contains(&crate::card::ArtifactSubtype::Fortification) {
                    self.evaluate_requirement_static(&crate::card::SelectionRequirement::Land, host, a.controller, None)
                } else {
                    false
                }
            }
            Target::Player(_) => def.is_aura() && enchant_ok(host),
            _ => false,
        }
    }

    /// `Effect::AttachAnyNumberTo` — offer every permanent `what` resolves to
    /// that could legally attach to the host `to` (a permanent or, for an
    /// "enchant player" Aura, a player) and attach the chosen ones.
    pub(super) fn attach_any_number_to(
        &mut self,
        what: &Selector,
        to: &Selector,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        if !self.scratch.last_moved_cards.is_empty() {
            self.scratch.last_moved_cards.clear();
        }
        let Some(host) = self.resolve_selector(to, ctx).iter().find_map(|e| match e {
            super::EntityRef::Player(p) => Some(Target::Player(*p)),
            e => e.as_permanent_id().map(Target::Permanent),
        }) else {
            return Ok(());
        };
        let candidates: Vec<(CardId, String)> = self
            .resolve_selector(what, ctx)
            .into_iter()
            .filter_map(|e| e.as_permanent_id())
            .filter(|&id| self.can_become_attached(id, &host))
            .filter_map(|id| self.battlefield_find(id).map(|c| (id, c.definition.name.to_string())))
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        let hostile_host = |g: &GameState, id: CardId| {
            g.battlefield_find(id).is_some_and(|c| {
                c.attached_to.and_then(|h| g.battlefield_find(h)).is_some_and(|h| h.controller != me)
                    || c.attached_to_player.is_some_and(|p| p != me)
            })
        };
        // Headless, an attachment moves only between hosts on the same side:
        // Equipment from one of yours to another, a Curse between opponents.
        let host_hostile = match host {
            Target::Player(p) => p != me,
            Target::Permanent(h) => self.battlefield_find(h).is_some_and(|c| c.controller != me),
        };
        let auto: Vec<CardId> = candidates
            .iter()
            .map(|(id, _)| *id)
            .filter(|&id| {
                self.battlefield_find(id).is_some_and(|c| c.controller == me) && hostile_host(self, id) == host_hostile
            })
            .collect();
        let max = candidates.len() as u32;
        let Some(picked) = self.choose_up_to_cards(
            me,
            "Choose any number to attach".into(),
            ctx.source.unwrap_or(CardId(0)),
            candidates,
            max,
            PickValue::Gain,
            effect,
            auto,
        ) else {
            return Ok(());
        };
        let (on_card, on_player) = match host {
            Target::Permanent(h) => (Some(h), None),
            Target::Player(p) => (None, Some(p)),
        };
        // The attached ones are `LastMoved` — "when one or more … become
        // attached this way" (Thorin, Mountain-king). Guarded: a CoW group.
        if !picked.is_empty() {
            self.scratch.last_moved_cards = picked.clone();
        }
        for id in picked {
            if let Some(c) = self.battlefield_find_mut(id) {
                c.attached_to = on_card;
                c.attached_to_player = on_player;
                events.push(GameEvent::AttachmentMoved { attachment: id, attached_to: on_card });
            }
        }
        Ok(())
    }

    /// `Effect::AttachEachToCreatureYouControl` — for each permanent `what`
    /// resolves to, its controller may move it onto a creature they control
    /// that it can legally be attached to (CR 301.5c / 303.4). Every host is
    /// asked first, then they move; headless, each goes to the greatest-power
    /// one (where it already is, nothing moves).
    pub(super) fn attach_each_to_creature_you_control(
        &mut self,
        what: &Selector,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let attachments: Vec<CardId> = self
            .resolve_selector(what, ctx)
            .into_iter()
            .filter_map(|e| e.as_permanent_id())
            .filter(|id| self.battlefield_find(*id).is_some_and(|c| c.controller == me))
            .collect();
        let mut creatures: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == me && self.computed_is_creature(c))
            .map(|c| c.id)
            .collect();
        creatures.sort_by_key(|&id| std::cmp::Reverse(self.battlefield_find(id).map_or(0, |c| self.effective_power_on(c))));
        let asks = self.seat_prompts(me) || !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto);
        let mut cursor = 0;
        let mut moves: Vec<(CardId, CardId)> = Vec::new();
        for att in attachments {
            let Some(def) = self.battlefield_find(att).map(|c| c.definition.clone()) else { continue };
            let legal: Vec<CardId> = creatures
                .iter()
                .copied()
                .filter(|&h| h != att)
                .filter(|&h| match def.aura_enchant_filter() {
                    Some(f) if def.is_aura() => {
                        self.evaluate_requirement_static(f, &crate::game::types::Target::Permanent(h), me, Some(att))
                    }
                    _ => true,
                })
                .collect();
            let Some(&best) = legal.first() else { continue };
            let pick = if asks {
                let name = def.name.to_string();
                let targets = legal.iter().map(|&h| crate::game::types::Target::Permanent(h)).collect();
                match self.ask_seat_target_maybe_logged(&mut cursor, me, format!("Attach {name} to"), att, targets, effect, true) {
                    None => return Ok(()),
                    Some(Some(crate::game::types::Target::Permanent(h))) => Some(h),
                    Some(_) => None,
                }
            } else {
                Some(best)
            };
            if let Some(h) = pick {
                moves.push((att, h));
            }
        }
        self.clear_answer_log();
        for (att, h) in moves {
            if let Some(c) = self.battlefield_find_mut(att)
                && c.attached_to != Some(h)
            {
                c.attached_to = Some(h);
                c.attached_to_player = None;
                events.push(GameEvent::AttachmentMoved { attachment: att, attached_to: Some(h) });
            }
        }
        Ok(())
    }

    /// `Effect::AttachDistinctToEach` — for each host `to` resolves to, its
    /// controller may pick one not-yet-picked permanent of `what`; the picks
    /// are all made first, then attached at once (2023-02-04: no Equipment
    /// moves twice). Headless, the unattached ones go out greatest mana value
    /// first, so nothing is stripped off a creature already wearing it.
    pub(super) fn attach_distinct_to_each(
        &mut self,
        what: &Selector,
        to: &Selector,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let hosts: Vec<CardId> = self
            .resolve_selector(to, ctx)
            .into_iter()
            .filter_map(|e| e.as_permanent_id())
            .filter(|&id| self.battlefield_find(id).is_some())
            .collect();
        let mut pool: Vec<(CardId, String)> = self
            .resolve_selector(what, ctx)
            .into_iter()
            .filter_map(|e| e.as_permanent_id())
            .filter_map(|id| self.battlefield_find(id).map(|c| (id, c.definition.name.to_string())))
            .collect();
        let mut free: Vec<CardId> = pool
            .iter()
            .map(|(id, _)| *id)
            .filter(|&id| self.battlefield_find(id).is_some_and(|c| c.attached_to.is_none()))
            .collect();
        free.sort_by_key(|&id| std::cmp::Reverse(self.battlefield_find(id).map_or(0, |c| c.definition.cost.cmc())));
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mut moves: Vec<(CardId, CardId)> = Vec::new();
        for host in hosts {
            let candidates: Vec<(CardId, String)> = pool.iter().filter(|(id, _)| *id != host).cloned().collect();
            if candidates.is_empty() {
                break;
            }
            let auto: Vec<CardId> = free.first().copied().filter(|&id| id != host).into_iter().collect();
            let Some(picked) = self.ask_seat_cards_logged(
                &mut cursor,
                me,
                "Choose an attachment for this creature".into(),
                source,
                candidates,
                0,
                1,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            if let Some(&att) = picked.first() {
                pool.retain(|(id, _)| *id != att);
                free.retain(|&id| id != att);
                moves.push((att, host));
            }
        }
        self.clear_answer_log();
        for (att, host) in moves {
            if let Some(c) = self.battlefield_find_mut(att)
                && c.attached_to != Some(host)
            {
                c.attached_to = Some(host);
                c.attached_to_player = None;
                events.push(GameEvent::AttachmentMoved { attachment: att, attached_to: Some(host) });
            }
        }
        Ok(())
    }
}
