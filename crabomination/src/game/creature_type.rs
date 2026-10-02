//! CR 205.3 / 702.73a — a battlefield permanent's creature and land types are
//! its current ones: the layer-4 view when a type-changing effect is in
//! scope, and a changeling is every creature type. Ad-hoc battlefield walks
//! read the printed line, so a changeling missed "Elves you control" and
//! swampwalk ignored Urborg, Tomb of Yawgmoth.

use super::GameState;
use crate::card::{CardId, CardInstance, CreatureType, Keyword, KeywordSlice, LandType};

/// What [`GameState::off_battlefield_type_grants`] adds to a card's types.
#[derive(Default)]
pub(crate) struct OffBattlefieldTypes {
    /// Every creature type (Maskwood Nexus).
    pub every: bool,
    /// Chosen types added (Ashes of the Fallen, Leyline of Transformation).
    pub chosen: smallvec::SmallVec<[CreatureType; 2]>,
}

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

    /// CR 205.3 — does `card`, off the battlefield, have creature type `ct`?
    /// Printed line or changeling, plus the grants layers can't reach
    /// ([`Self::off_battlefield_type_grants`]).
    pub(crate) fn card_off_battlefield_has_creature_type(&self, card: &CardInstance, ct: CreatureType) -> bool {
        card.definition.subtypes.creature_types.contains(&ct)
            || card.has_keyword(&Keyword::Changeling)
            || self.off_battlefield_type_grants(card).is_some_and(|g| g.every || g.chosen.contains(&ct))
    }

    /// CR 702.73a / 205.3 — is `card`, off the battlefield, every creature
    /// type (changeling, or Maskwood Nexus's "cards you own")?
    pub(crate) fn card_off_battlefield_is_every_creature_type(&self, card: &CardInstance) -> bool {
        card.has_keyword(&Keyword::Changeling) || self.off_battlefield_type_grants(card).is_some_and(|g| g.every)
    }

    /// The creature types an off-battlefield `card` gains from its owner's
    /// statics (its caster's, on the stack): Ashes of the Fallen in the
    /// graveyard, Leyline of Transformation's chosen type and Maskwood
    /// Nexus's every type elsewhere. `None` (no walk) unless the lane says a
    /// creature-type changer is on the battlefield, or for a permanent.
    pub(crate) fn off_battlefield_type_grants(&self, card: &CardInstance) -> Option<OffBattlefieldTypes> {
        use crate::effect::StaticEffect as SE;
        if !self.battlefield.has_creature_type_changer(super::card_can_change_creature_types)
            || self.battlefield.find_by_id(card.id).is_some()
        {
            return None;
        }
        let who = self.stack_caster_for_card(card.id).unwrap_or(card.owner);
        let in_graveyard = self.players.get(card.owner).is_some_and(|p| p.graveyard.iter().any(|c| c.id == card.id));
        let mut out = OffBattlefieldTypes::default();
        for c in self.battlefield.iter().filter(|c| c.controller == who) {
            for sa in &c.definition.static_abilities {
                match &sa.effect {
                    SE::YourGraveyardCreaturesHaveChosenType if in_graveyard => {
                        out.chosen.extend(c.chosen_creature_type);
                    }
                    SE::OwnedCardsOffBattlefieldAreChosenTypeToo { filter }
                        if self.evaluate_requirement_on_card(filter, card, who) =>
                    {
                        out.chosen.extend(c.chosen_creature_type);
                    }
                    SE::OwnedCardsOffBattlefieldAreEveryCreatureType { filter }
                        if self.evaluate_requirement_on_card(filter, card, who) =>
                    {
                        out.every = true;
                    }
                    _ => {}
                }
            }
        }
        (out.every || !out.chosen.is_empty()).then_some(out)
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
