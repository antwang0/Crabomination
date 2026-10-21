//! "…if another opponent controls one or more nonland permanents that spell
//! could target, choose one of those permanents. Copy that spell. The copy
//! targets the chosen permanent." — Exterminator Magmarch; Feather, Radiant
//! Arbiter's paid copies onto chosen creatures; and Zevlor, Elturel Exile's
//! copy for each other opponent.

use super::{EffectContext, PickValue};
use crate::card::CardId;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, StackItem, Target};

impl GameState {
    /// The spell `what` resolves to must be on the stack aimed at one
    /// permanent; the candidates are the nonland permanents an opponent of
    /// the copier OTHER than that permanent's controller controls and the
    /// spell could legally target for the copier. None — the intervening
    /// "if" fails and nothing happens (CR 603.4).
    pub(super) fn copy_spell_onto_another_opponents_permanent(
        &mut self,
        what: &Selector,
        ctx: &EffectContext,
        effect: &Effect,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let Some(spell_id) = self.resolve_selector(what, ctx).into_iter().find_map(|e| e.as_card_id()) else {
            return Ok(());
        };
        let Some((def, aimed)) = self.stack.iter().rev().find_map(|s| match s {
            StackItem::Spell { card, target: Some(Target::Permanent(t)), .. } if card.id == spell_id => {
                Some((card.definition.arc(), *t))
            }
            _ => None,
        }) else {
            return Ok(());
        };
        let Some(first_opp) = self.battlefield_find(aimed).map(|c| c.controller) else { return Ok(()) };
        let opponents = self.opponents_of(me);
        let candidates: Vec<(CardId, String)> = self
            .enumerate_legal_targets(&def.effect, me)
            .into_iter()
            .filter_map(|t| match t {
                Target::Permanent(id) => self.battlefield_find(id),
                _ => None,
            })
            .filter(|c| {
                !c.definition.is_land() && c.controller != first_opp && opponents.contains(&c.controller)
            })
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        let auto = candidates
            .iter()
            .filter_map(|(id, _)| self.battlefield_find(*id))
            .max_by_key(|c| c.definition.cost.cmc())
            .map(|c| vec![c.id])
            .unwrap_or_default();
        let Some(picked) = self.choose_up_to_cards(
            me,
            "Choose another opponent's permanent for the copy".into(),
            ctx.source.unwrap_or(CardId(0)),
            candidates,
            1,
            PickValue::Cost,
            effect,
            auto,
        ) else {
            return Ok(());
        };
        let Some(aim) = picked.first().copied() else { return Ok(()) };
        let before = self.stack.len();
        self.copy_stack_spell_controlled(spell_id, 1, false, Some(me), None, events);
        if self.stack.len() > before
            && let Some(StackItem::Spell { target: t, .. }) = self.stack.last_mut()
        {
            *t = Some(Target::Permanent(aim));
        }
        self.remark_stack_targets(self.stack.len().saturating_sub(1));
        Ok(())
    }

    /// Feather, Radiant Arbiter — the cast spell (the trigger's source) must
    /// target only the trigger's source permanent. Its caster chooses any
    /// number of other creatures the spell could target and pays `per_copy`
    /// for each; each paid copy targets its creature. A headless caster
    /// chooses its own creatures: the spell was aimed at its own Feather.
    pub(super) fn may_pay_to_copy_onto_other_creatures(
        &mut self,
        per_copy: &crate::mana::ManaCost,
        ctx: &EffectContext,
        effect: &Effect,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(me) = ctx.source else { return Ok(()) };
        let Some(spell_id) = ctx.trigger_source.and_then(|e| e.as_card_id()) else { return Ok(()) };
        let Some((def, caster)) = self.stack.iter().rev().find_map(|s| match s {
            StackItem::Spell { card, caster, target: Some(Target::Permanent(t)), additional_targets, .. }
                if card.id == spell_id && *t == me && additional_targets.is_empty() =>
            {
                Some((card.definition.arc(), *caster))
            }
            _ => None,
        }) else {
            return Ok(());
        };
        let candidates: Vec<(CardId, String)> = self
            .enumerate_legal_targets(&def.effect, caster)
            .into_iter()
            .filter_map(|t| match t {
                Target::Permanent(id) if id != me => self.battlefield_find(id),
                _ => None,
            })
            .filter(|c| self.computed_is_creature(c))
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        // A person may aim a copy anywhere; a headless policy aims it by the
        // spell's polarity — a pump at its own creatures, a hostile spell at
        // its opponents' — the same classifier the auto-targeter reads.
        let friendly = def.effect.prefers_friendly_target();
        let auto: Vec<CardId> = candidates
            .iter()
            .filter(|(id, _)| {
                self.battlefield_find(*id).is_some_and(|c| self.same_team(c.controller, caster) == friendly)
            })
            .map(|(id, _)| *id)
            .collect();
        let max = if self.seat_prompts(caster) { candidates.len() } else { auto.len() } as u32;
        let Some(picked) = self.choose_up_to_cards(
            caster,
            format!("Choose creatures to copy the spell onto (pay {} each)", per_copy.cmc()),
            me,
            candidates,
            max,
            PickValue::Gain,
            effect,
            auto,
        ) else {
            return Ok(());
        };
        for aim in picked {
            if !self.pay_mana_cost_with_picks(caster, per_copy, None, events) {
                break;
            }
            let before = self.stack.len();
            self.copy_stack_spell_controlled(spell_id, 1, false, Some(caster), None, events);
            if self.stack.len() > before
                && let Some(StackItem::Spell { target: t, .. }) = self.stack.last_mut()
            {
                *t = Some(Target::Permanent(aim));
            }
            self.remark_stack_targets(self.stack.len().saturating_sub(1));
        }
        Ok(())
    }

    /// Zevlor, Elturel Exile — the spell `what` resolves to targets one
    /// opponent or one permanent an opponent controls; for each *other*
    /// opponent (turn order, CR 101.4) the copier chooses that player or a
    /// permanent they control the spell could target, and a copy is aimed
    /// there (CR 707.10c). An opponent offering no legal target gets no copy.
    /// Headless: the same kind as the original — the player, or their
    /// priciest permanent.
    pub(super) fn copy_spell_for_each_other_opponent(
        &mut self,
        what: &Selector,
        ctx: &EffectContext,
        effect: &Effect,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let Some(spell_id) = self.resolve_selector(what, ctx).into_iter().find_map(|e| e.as_card_id()) else {
            return Ok(());
        };
        let Some((def, aimed)) = self.stack.iter().rev().find_map(|s| match s {
            StackItem::Spell { card, target: Some(t), .. } if card.id == spell_id => Some((card.definition.arc(), t.clone())),
            _ => None,
        }) else {
            return Ok(());
        };
        let first = match &aimed {
            Target::Player(p) => Some(*p),
            Target::Permanent(id) => self.battlefield_find(*id).map(|c| c.controller),
        };
        let at_player = matches!(aimed, Target::Player(_));
        let legal = self.enumerate_legal_targets(&def.effect, me);
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mut aims: Vec<Target> = Vec::new();
        for opp in self.opponents_of(me) {
            if Some(opp) == first {
                continue;
            }
            let mut mine: Vec<Target> = legal
                .iter()
                .filter(|t| match t {
                    Target::Player(p) => *p == opp,
                    Target::Permanent(id) => self.battlefield_find(*id).is_some_and(|c| c.controller == opp),
                })
                .cloned()
                .collect();
            if mine.is_empty() {
                continue;
            }
            // The headless pick first: the asker's fallback.
            let value = |t: &Target| match t {
                Target::Player(_) => if at_player { u32::MAX } else { 0 },
                Target::Permanent(id) => self.battlefield_find(*id).map_or(0, |c| c.definition.cost.cmc() + 1),
            };
            mine.sort_by_key(|t| std::cmp::Reverse(value(t)));
            let Some(aim) = self.ask_seat_target_logged(
                &mut cursor,
                me,
                format!("Choose seat {opp} or a permanent they control for a copy"),
                source,
                mine,
                effect,
            ) else {
                return Ok(());
            };
            aims.push(aim);
        }
        self.clear_answer_log();
        for aim in aims {
            let before = self.stack.len();
            self.copy_stack_spell_controlled(spell_id, 1, false, Some(me), None, events);
            if self.stack.len() > before
                && let Some(StackItem::Spell { target: t, .. }) = self.stack.last_mut()
            {
                *t = Some(aim);
            }
            self.remark_stack_targets(self.stack.len().saturating_sub(1));
        }
        Ok(())
    }
}
