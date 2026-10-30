//! Zada, Hedron Grinder / Beamsplitter Mage — an instant or sorcery that
//! targets only this creature is copied for each other creature its caster
//! controls that the spell could target (Zada), or once, for one of those the
//! caster chooses (Beamsplitter Mage's `CopyForChosenOtherTargetableCreature`).
//! CR 707.10: the copies keep the spell's decisions and no mana is spent.

use super::EffectContext;
use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::Effect;
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, Target};

impl GameState {
    pub(super) fn copy_spell_for_other_creatures(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
        choose_one: bool,
    ) -> Result<(), GameError> {
        use crate::game::types::StackItem;
        // The spell that triggered this (Zada's cast trigger binds it
        // as the trigger source).
        let Some(spell_id) = ctx
            .trigger_source
            .and_then(|e| e.as_card_id())
            .or(ctx.source)
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
        let StackItem::Spell {
            card, caster, target, additional_targets, mode, x_value, ..
        } = &self.stack[idx]
        else {
            return Ok(());
        };
        // "Targets only [this creature]" — one target slot, and it is
        // the source permanent.
        let me = ctx.source;
        let targets_only_me = additional_targets.is_empty()
            && matches!((target, me), (Some(Target::Permanent(t)), Some(m)) if *t == m);
        if !targets_only_me {
            return Ok(());
        }
        let (def, caster) = (card.definition.arc(), *caster);
        // CR 707.10 — decisions copied; no mana spent (converge 0).
        let (mode, x_value, choices) = (*mode, *x_value, crate::game::spell_copy::CastChoices::of(card));
        let others: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == caster && Some(c.id) != me)
            .filter(|c| self.computed_is_creature(c))
            .map(|c| c.id)
            .collect();
        let legal = self.enumerate_legal_targets(&def.effect, caster);
        let mut others: Vec<CardId> =
            others.into_iter().filter(|id| legal.contains(&Target::Permanent(*id))).collect();
        // Beamsplitter Mage: one copy, aimed at a creature its caster chooses.
        if choose_one && others.len() > 1 {
            let mut cands: Vec<(CardId, String, i32)> = others
                .iter()
                .filter_map(|&id| {
                    let c = self.battlefield_find(id)?;
                    Some((id, c.definition.name.to_string(), self.computed_permanent(id).map_or(0, |cp| cp.power)))
                })
                .collect();
            cands.sort_by_key(|&(id, _, power)| (std::cmp::Reverse(power), id));
            let mut cursor = 0;
            let Some(pick) = self.ask_seat_cards_logged(
                &mut cursor,
                caster,
                "Choose a creature for the copy to target".into(),
                ctx.source.unwrap_or(CardId(0)),
                cands.iter().map(|c| (c.0, c.1.clone())).collect(),
                1,
                1,
                PickValue::Gain,
                effect,
                vec![cands[0].0],
            ) else {
                return Ok(());
            };
            self.clear_answer_log();
            others.retain(|id| pick.first() == Some(id));
        }
        for id in others {
            let t = Target::Permanent(id);
            let new_id = self.next_id();
            let mut copy_inst = crate::card::CardInstance::new(new_id, def.clone(), caster);
            copy_inst.is_token = true;
            choices.apply(&mut copy_inst);
            self.push_stack(StackItem::Spell {
                card: Box::new(copy_inst),
                caster,
                target: Some(t),
                additional_targets: Vec::new(),
                mode,
                x_value,
                converged_value: 0,
                mana_spent: 0,
                uncounterable: false,
            });
            events.push(GameEvent::SpellsCopied {
                copy: new_id,
                count: 1,
                controller: caster,
            });
        }
        Ok(())
    }
}
