//! `Effect::AmendCopy` — CR 707.9b: the "except …" of a copy effect becomes
//! part of the copiable values it installed (Volrath, the Shapestealer's
//! "except it's 7/5 and it has this ability").

use super::EffectContext;
use crate::card::{CardId, CardType, Keyword, Supertype};
use crate::effect::Selector;
use crate::game::GameState;
use crate::game::types::GameError;

impl GameState {
    /// Rewrite each `what`'s current (copied) definition. "This ability"
    /// indexes the copier's *printed* abilities: the pre-copy definition its
    /// oldest live `temporary_copies` entry remembers, else its own.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn amend_copy(
        &mut self,
        what: &Selector,
        keep_activated: &[u8],
        keep_triggered: &[u8],
        pt: Option<(i32, i32)>,
        keep_name: bool,
        legendary: bool,
        keywords: &[Keyword],
        card_types: &[CardType],
        ctx: &EffectContext,
    ) -> Result<(), GameError> {
        let ids: Vec<CardId> =
            self.resolve_selector(what, ctx).into_iter().filter_map(|e| e.as_permanent_id()).collect();
        for cid in ids {
            let printed =
                self.temporary_copies.iter().find(|t| t.card == cid).and_then(|t| t.original.clone());
            let Some(c) = self.battlefield.find_by_id_mut(cid) else { continue };
            if c.face_down {
                continue;
            }
            let printed = printed.unwrap_or_else(|| c.definition.arc());
            let mut def = (*c.copiable_definition()).clone();
            for &i in keep_activated {
                if let Some(a) = printed.activated_abilities.get(i as usize)
                    && !def.activated_abilities.contains(a)
                {
                    def.activated_abilities.push(a.clone());
                }
            }
            for &i in keep_triggered {
                if let Some(t) = printed.triggered_abilities.get(i as usize)
                    && !def.triggered_abilities.contains(t)
                {
                    def.triggered_abilities.push(t.clone());
                }
            }
            if let Some((p, t)) = pt {
                def.power = p;
                def.toughness = t;
            }
            if keep_name {
                def.name = printed.name;
            }
            if legendary && !def.supertypes.contains(&Supertype::Legendary) {
                def.supertypes.push(Supertype::Legendary);
            }
            for k in keywords {
                if !def.keywords.contains(k) {
                    def.keywords.push(k.clone());
                }
            }
            for t in card_types {
                if !def.card_types.contains(t) {
                    def.card_types.push(t.clone());
                }
            }
            c.set_copiable_definition(std::sync::Arc::new(def));
        }
        Ok(())
    }
}
