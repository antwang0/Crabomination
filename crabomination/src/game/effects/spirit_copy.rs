//! CR 707.9b — a spell copy "except it's a 1/1 Spirit in addition to its
//! other types" (Donal, Herald of Wings). The exception is a copiable value,
//! so it is written into the copy's own definition and the token the copy
//! resolves into (CR 111.1) keeps it.

use crate::card::{CardId, CreatureType};
use crate::game::types::{GameEvent, StackItem};
use crate::game::GameState;

impl GameState {
    /// Copy the spell `cid` once and make the copy a 1/1 Spirit.
    pub(crate) fn copy_spell_as_one_one_spirit(&mut self, cid: CardId, events: &mut Vec<GameEvent>) {
        let before = self.stack.len();
        self.copy_stack_spell_controlled(cid, 1, true, None, None, events);
        for item in self.stack.iter_mut().skip(before) {
            if let StackItem::Spell { card, .. } = item {
                let def = card.definition_make_mut();
                def.power = 1;
                def.toughness = 1;
                if !def.subtypes.creature_types.contains(&CreatureType::Spirit) {
                    def.subtypes.creature_types.push(CreatureType::Spirit);
                }
            }
        }
    }
}
