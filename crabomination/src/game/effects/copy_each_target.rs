//! Radiant Performer — "copy that spell or ability for each other permanent
//! or player the spell or ability could target. Each copy targets a different
//! one of those." And a copy handed to another player (Parnesse).

use super::EffectContext;
use crate::card::CardId;
use crate::effect::Selector;
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, StackItem, Target};

impl GameState {
    /// `Effect::CopySpellForEachOtherLegalTarget` — CR 707.10: the copies are
    /// the resolving controller's, one per permanent or player the spell could
    /// target other than its current target, each aimed at its own. Only
    /// graveyard / exile targets are left out. A spell with more than one
    /// target is not copied (the printed filter says "only a single").
    pub(super) fn copy_spell_for_each_other_legal_target(
        &mut self,
        what: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(spell_id) = self.resolve_selector(what, ctx).into_iter().find_map(|e| e.as_card_id())
        else {
            return Ok(());
        };
        if crate::game::types::is_stack_ability_id(spell_id) {
            self.copy_ability_for_each_other_legal_target(spell_id, ctx);
            return Ok(());
        }
        let Some(idx) = self
            .stack
            .iter()
            .rposition(|s| matches!(s, StackItem::Spell { card, .. } if card.id == spell_id))
        else {
            return Ok(());
        };
        let StackItem::Spell { card, target, additional_targets, mode, x_value, .. } =
            &self.stack[idx]
        else {
            return Ok(());
        };
        let Some(current) = target.clone() else { return Ok(()) };
        if !additional_targets.is_empty() {
            return Ok(());
        }
        let def = card.definition.arc();
        // CR 707.10 — decisions copied; no mana spent (converge 0).
        let (mode, x_value, choices) = (*mode, *x_value, crate::game::spell_copy::CastChoices::of(card));
        let me = ctx.controller;
        let others: Vec<Target> = self
            .enumerate_legal_targets(&def.effect, me)
            .into_iter()
            .filter(|t| *t != current && matches!(t, Target::Permanent(_) | Target::Player(_)))
            .collect();
        for t in others {
            let new_id: CardId = self.next_id();
            let mut copy = crate::card::CardInstance::new(new_id, def.clone(), me);
            copy.is_token = true;
            choices.apply(&mut copy);
            self.push_stack(StackItem::Spell {
                card: Box::new(copy),
                caster: me,
                target: Some(t),
                additional_targets: Vec::new(),
                mode,
                x_value,
                converged_value: 0,
                mana_spent: 0,
                uncounterable: false,
            });
            events.push(GameEvent::SpellsCopied { original: spell_id, count: 1, controller: me });
        }
        Ok(())
    }

    /// The ability half (CR 115.1 — the stack ability `aid` names): each copy
    /// is a clone of the ability, controlled by the resolving controller and
    /// aimed at one other permanent or player its effect could target.
    fn copy_ability_for_each_other_legal_target(&mut self, aid: CardId, ctx: &EffectContext) {
        let Some(pos) = self.stack_ability_pos(aid, None) else { return };
        let item = self.stack[pos].clone();
        let StackItem::Trigger { source, effect, target: Some(current), additional_targets, .. } = &item else {
            return;
        };
        if !additional_targets.is_empty() {
            return;
        }
        let me = ctx.controller;
        let others: Vec<Target> = self
            .enumerate_legal_targets_with_source(effect, me, Some(*source))
            .into_iter()
            .filter(|t| t != current && matches!(t, Target::Permanent(_) | Target::Player(_)))
            .collect();
        for t in others {
            let mut copy = item.clone();
            if let StackItem::Trigger { controller, target, .. } = &mut copy {
                *controller = me;
                *target = Some(t);
            }
            self.push_stack(copy);
        }
    }

    /// `Effect::CopySpellForPlayer` — one copy of each spell `what` names,
    /// controlled by `who`, who may choose new targets (CR 707.10c, 115.7).
    pub(super) fn copy_spell_for_player(
        &mut self,
        what: &Selector,
        who: &crate::effect::PlayerRef,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(seat) = self.resolve_player(who, ctx) else { return Ok(()) };
        let ids: Vec<CardId> = match what {
            Selector::TriggerSource => ctx.trigger_source.and_then(|e| e.as_card_id()).into_iter().collect(),
            _ => self.resolve_selector(what, ctx).into_iter().filter_map(|e| e.as_card_id()).collect(),
        };
        for cid in ids {
            self.copy_stack_spell_controlled(cid, 1, true, Some(seat), None, events);
        }
        Ok(())
    }
}
