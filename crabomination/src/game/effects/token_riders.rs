//! "…create a token that's a copy of that creature, except it's a Food …
//! and it has '[ability]'" (Brenard, Ginger Sculptor). The exceptions are
//! part of the copy's copiable values (CR 707.9b), so they are written into
//! the new tokens' own definitions rather than granted by a layer effect.

use super::EffectContext;
use crate::card::ArtifactSubtype;
use crate::effect::{ActivatedAbility, Selector};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// `Effect::StampTokenCopyExceptions { artifact_subtypes, activated }` —
    /// add the subtypes and abilities to the tokens this resolution created.
    pub(super) fn stamp_token_copy_exceptions(
        &mut self,
        artifact_subtypes: &[ArtifactSubtype],
        activated: &[ActivatedAbility],
        ctx: &EffectContext,
        _events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let ids: Vec<_> = self
            .resolve_selector(&Selector::LastCreatedTokens, ctx)
            .into_iter()
            .filter_map(|e| e.as_permanent_id())
            .collect();
        for id in ids {
            let Some(c) = self.battlefield.find_by_id_mut(id) else { continue };
            let def = c.definition_make_mut();
            for st in artifact_subtypes {
                if !def.subtypes.artifact_subtypes.contains(st) {
                    def.subtypes.artifact_subtypes.push(*st);
                }
            }
            def.activated_abilities.extend(activated.iter().cloned());
        }
        Ok(())
    }
}
