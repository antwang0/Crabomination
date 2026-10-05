//! CR 608.2h — what a resolution remembers about a sacrifice. "Each creature
//! that shares a creature type with the sacrificed creature" (Endemic
//! Plague's cost) read the card through its death snapshot, which the trigger
//! dispatcher clears after each batch, then the card itself: a sacrificed
//! token (CR 111.7) named no tribe, so the Plague swept nothing.

use super::GameState;
use crate::card::CardId;

impl GameState {
    /// `Selector::SacrificedCard`, and a token's creature types beside it
    /// (see `ResolutionScratch::sacrificed_token_types`), stamped while `id`
    /// is on the battlefield or has a death snapshot.
    pub(crate) fn stamp_sacrificed_card(&mut self, id: CardId) {
        self.sacrificed_card = Some(id);
        let stamp = if let Some(c) = self.battlefield_find(id) {
            c.is_token.then(|| {
                let printed = &c.definition.subtypes.creature_types;
                let types = if self.creature_type_change_in_scope() {
                    self.computed_permanent_on(c).map_or_else(|| printed.clone(), |cp| cp.subtypes().creature_types.clone())
                } else {
                    printed.clone()
                };
                (types, self.permanent_is_changeling(c))
            })
        } else {
            self.died_card_snapshots.get(&id).filter(|c| c.is_token).map(|c| {
                (c.definition.subtypes.creature_types.clone(), c.has_keyword(&crate::card::Keyword::Changeling))
            })
        };
        if let Some((types, changeling)) = stamp {
            self.scratch.sacrificed_token_types = Some((id, types, changeling));
            let lki = match self.battlefield_find(id) {
                Some(c) => Some(self.lki_clone(c)),
                None => self.died_card_snapshots.get(&id).cloned(),
            };
            self.scratch.sacrificed_token_lki = lki.map(Box::new);
        }
    }

    /// The type / colour flags of the last cost sacrifice, for
    /// `Effect::WithSacrificedPt` to carry to resolution (CR 602.2b).
    pub(crate) fn sacrificed_traits(&self) -> Option<Box<crate::effect::SacrificedTraits>> {
        let t = crate::effect::SacrificedTraits {
            artifact: self.sacrificed_was_artifact,
            outlaw: self.sacrificed_was_outlaw,
            vehicle: self.sacrificed_was_vehicle,
            colors: self.sacrificed_colors.clone(),
        };
        (t != crate::effect::SacrificedTraits::default()).then(|| Box::new(t))
    }
}
