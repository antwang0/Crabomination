//! "Whenever this Equipment becomes unattached from a permanent" (CR 301.5c,
//! the Grafted Equipment). Every site that moves an Equipment off a host
//! calls [`GameState::note_unattached`]; the rulings name them — equipping
//! it elsewhere, an unattach effect or cost, the Equipment leaving the
//! battlefield, and the host ceasing to be a creature (CR 704.5n).

use crate::card::{CardDefinition, CardId, SelectionRequirement as R};
use crate::effect::{Effect, Predicate, Selector};
use crate::game::GameState;
use crate::game::effects::EntityRef;
use crate::game::types::PendingTriggerPush;

impl GameState {
    /// Queue the Grafted trigger if `def` carries it: sacrifice `host`, which
    /// its controller does only while they still control it (ruling), or —
    /// Captain's Hook's `destroy_host_when_unattached` — destroy it. A host
    /// that has left the battlefield gets nothing — the ability "won't do
    /// anything in that case".
    pub(crate) fn note_unattached(
        &mut self,
        attachment: CardId,
        def: &CardDefinition,
        controller: usize,
        host: CardId,
    ) {
        let Some(bonus) = def.equipped_bonus.as_ref() else { return };
        if !(bonus.sacrifice_host_when_unattached || bonus.destroy_host_when_unattached)
            || self.battlefield_find(host).is_none()
        {
            return;
        }
        let effect = if bonus.destroy_host_when_unattached {
            Effect::Destroy { what: Selector::TriggerSource }
        } else {
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::ControlledByYou },
                then: Box::new(Effect::SacrificePermanent { what: Selector::TriggerSource }),
                else_: Box::new(Effect::Noop),
            }
        };
        self.push_pending_trigger(
            PendingTriggerPush {
                source: attachment,
                controller,
                effect,
                subject: Some(EntityRef::Permanent(host)),
                event_amount: 0,
                mode: None,
                intervening_if: None,
                actor: None,
                from_mana_ability: false,
                x_value: 0,
                converged_value: 0,
                mana_spent: 0,
            },
            None,
        );
    }
}
