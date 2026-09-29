//! CR 205.3 / 702.73a — a battlefield permanent's creature types are its
//! current ones: the layer-4 view when a type-changing effect is in scope,
//! and a changeling is every creature type. Ad-hoc battlefield walks read the
//! printed line, so a changeling missed "Elves you control" and an Island
//! turned into a Zombie stayed out of the count.

use super::GameState;
use crate::card::{CardId, CreatureType, Keyword, KeywordSlice};

impl GameState {
    /// Whether the permanent `cid` currently has creature type `ct`. Not for
    /// use inside the layer gather (it may compute the layer view).
    pub(crate) fn permanent_has_creature_type(&self, cid: CardId, ct: CreatureType) -> bool {
        let Some(c) = self.battlefield_find(cid) else { return false };
        if !self.creature_type_change_in_scope() {
            return c.has_keyword(&Keyword::Changeling) || c.definition.subtypes.creature_types.contains(&ct);
        }
        match self.computed_permanent_on(c) {
            Some(cp) => cp.subtypes().creature_types.contains(&ct) || cp.keywords().has_kw(&Keyword::Changeling),
            None => c.has_keyword(&Keyword::Changeling) || c.definition.subtypes.creature_types.contains(&ct),
        }
    }

    /// "Creature of type `ct`" — a creature now, with that type now.
    pub(crate) fn permanent_is_creature_of_type(&self, cid: CardId, ct: CreatureType) -> bool {
        self.permanent_is_creature(cid) && self.permanent_has_creature_type(cid, ct)
    }
}
