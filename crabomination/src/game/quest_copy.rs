//! Firebender Ascension (`Keyword::AttackTriggerQuestCopy`): "whenever a
//! creature you control attacking causes a triggered ability of that creature
//! to trigger, put a quest counter on this enchantment. Then if it has four or
//! more quest counters on it, you may copy that ability." Its trigger goes on
//! the stack right above the one that caused it (ruling), so the copy finds
//! that ability as the attacker's topmost (`Effect::CopyAbility`).

use super::GameState;
use crate::card::{CardId, CounterType, Predicate};
use crate::effect::{Effect, Selector, Value};
use crate::game::effects::EntityRef;
use crate::game::types::{PendingTriggerPush, TriggerPush};

fn quest_copy_effect() -> Effect {
    Effect::Seq(vec![
        Effect::AddCounter { what: Selector::This, kind: CounterType::Quest, amount: Value::ONE },
        Effect::If {
            cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Quest, n: 4 },
            then: Box::new(Effect::MayDo {
                description: "Copy that ability?".into(),
                body: Box::new(Effect::CopyAbility { what: Selector::TriggerSource, times: Value::ONE }),
            }),
            else_: Box::new(Effect::Noop),
        },
    ])
}

impl GameState {
    /// Queue each listener of `controller`'s that `attacker`'s just-queued
    /// attack trigger fires; `ask` follows the cause onto the asking queue.
    pub(crate) fn push_quest_copy_triggers(
        &mut self,
        listeners: &[(CardId, usize)],
        attacker: CardId,
        controller: usize,
        ask: bool,
    ) {
        for &(id, owner) in listeners {
            if owner != controller || self.battlefield_find(id).is_none() {
                continue;
            }
            let subject = Some(EntityRef::Permanent(attacker));
            if ask {
                self.queue_trigger_asking(PendingTriggerPush {
                    source: id,
                    controller,
                    effect: quest_copy_effect(),
                    subject,
                    event_amount: 0,
                    mode: None,
                    intervening_if: None,
                    actor: None,
                    from_mana_ability: false,
                    x_value: 0,
                    converged_value: 0,
                    mana_spent: 0,
                });
            } else {
                self.push_stack(TriggerPush::new(id, controller, quest_copy_effect()).trigger_source(subject).build());
            }
        }
    }
}
