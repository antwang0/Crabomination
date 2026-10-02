//! CR 205.3 / 702.73a — a battlefield permanent's creature and land types are
//! its current ones: the layer-4 view when a type-changing effect is in
//! scope, and a changeling is every creature type. Ad-hoc battlefield walks
//! read the printed line, so a changeling missed "Elves you control" and
//! swampwalk ignored Urborg, Tomb of Yawgmoth.

use super::GameState;
use crate::card::{CardId, CardInstance, CreatureType, Keyword, KeywordSlice, LandType};

impl GameState {
    /// Whether the permanent `cid` currently has creature type `ct`. Not for
    /// use inside the layer gather (it may compute the layer view).
    pub(crate) fn permanent_has_creature_type(&self, cid: CardId, ct: CreatureType) -> bool {
        let Some(c) = self.battlefield_find(cid) else { return false };
        if !self.creature_type_change_in_scope() {
            return self.permanent_is_changeling(c) || c.definition.subtypes.creature_types.contains(&ct);
        }
        // CR 613.1d — a later layer-4 "set" (Curse of Conformity) overrides
        // changeling's every-type CDA.
        match self.computed_permanent_on(c) {
            Some(cp) if cp.creature_types_set => cp.subtypes().creature_types.contains(&ct),
            Some(cp) => self.permanent_is_changeling(c) || cp.subtypes().creature_types.contains(&ct),
            None => self.permanent_is_changeling(c) || c.definition.subtypes.creature_types.contains(&ct),
        }
    }

    /// CR 702.73a — changeling, printed or granted by a layer-6 effect (an
    /// animated Mutavault). The instance's own keywords miss a layer grant,
    /// so the computed view is asked when a changeling grant is in scope.
    pub(crate) fn permanent_is_changeling(&self, c: &CardInstance) -> bool {
        c.has_keyword(&Keyword::Changeling)
            || (self.changeling_grant_in_scope()
                && self.battlefield.find_by_id(c.id).is_some()
                && self.computed_permanent_on(c).is_some_and(|cp| cp.keywords().has_kw(&Keyword::Changeling)))
    }

    /// "Creature of type `ct`" — a creature now, with that type now.
    pub(crate) fn permanent_is_creature_of_type(&self, cid: CardId, ct: CreatureType) -> bool {
        self.permanent_is_creature(cid) && self.permanent_has_creature_type(cid, ct)
    }

    /// Whether the permanent `c` currently has land type `lt` (Urborg makes
    /// every land a Swamp; Blood Moon makes nonbasics Mountains). Inside the
    /// layer pass, or for a card off the battlefield, the printed line.
    pub(crate) fn permanent_has_land_type(&self, c: &CardInstance, lt: LandType) -> bool {
        if self.layer_reads_are_printed()
            || !self.land_type_change_in_scope()
            || self.battlefield.find_by_id(c.id).is_none()
        {
            return c.definition.has_land_type(lt);
        }
        self.computed_permanent_on(c)
            .map_or_else(|| c.definition.has_land_type(lt), |cp| cp.subtypes().land_types.contains(&lt))
    }

    /// Could a layer-6 effect be granting changeling (Maskwood Nexus, an
    /// animated Mutavault)? `false` is authoritative: the printed keyword
    /// list is the whole answer.
    pub(crate) fn changeling_grant_in_scope(&self) -> bool {
        !self.layer_reads_are_printed() && self.keyword_grant_in_scope(|k| *k == Keyword::Changeling)
    }

    /// CR 613.1d — does battlefield permanent `card` currently have card type
    /// `t` (Liquimetal Coating's artifact, an animation's creature)? Gated the
    /// way [`Self::computed_is_creature`] is; off the battlefield, printed.
    #[inline]
    pub(crate) fn computed_has_card_type(&self, card: &CardInstance, t: crate::card::CardType) -> bool {
        if self.layer_reads_are_printed() || (!card.bestowed && !self.card_type_change_in_scope()) {
            return card.definition.card_types.contains(&t);
        }
        self.computed_has_card_type_slow(card, t)
    }

    #[inline(never)]
    fn computed_has_card_type_slow(&self, card: &CardInstance, t: crate::card::CardType) -> bool {
        if self.battlefield.find_by_id(card.id).is_none() {
            return card.definition.card_types.contains(&t);
        }
        self.computed_permanent_on(card)
            .map_or_else(|| card.definition.card_types.contains(&t), |cp| cp.card_types().contains(&t))
    }
}
