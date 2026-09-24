//! Radiant Performer — "copy that spell for each other permanent or player
//! the spell could target. Each copy targets a different one of those."

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
        let Some(idx) = self
            .stack
            .iter()
            .rposition(|s| matches!(s, StackItem::Spell { card, .. } if card.id == spell_id))
        else {
            return Ok(());
        };
        let StackItem::Spell { card, target, additional_targets, mode, x_value, converged_value, .. } =
            &self.stack[idx]
        else {
            return Ok(());
        };
        let Some(current) = target.clone() else { return Ok(()) };
        if !additional_targets.is_empty() {
            return Ok(());
        }
        let def = card.definition.arc();
        let (mode, x_value, converged_value) = (*mode, *x_value, *converged_value);
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
            self.stack.push(StackItem::Spell {
                card: Box::new(copy),
                caster: me,
                target: Some(t),
                additional_targets: Vec::new(),
                mode,
                x_value,
                converged_value,
                mana_spent: 0,
                uncounterable: false,
            });
            events.push(GameEvent::SpellsCopied { original: spell_id, count: 1, controller: me });
        }
        Ok(())
    }
}
