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

    /// CR 205.3 — does the land card `card`, off the battlefield, gain land
    /// type `lt` from a static its owner controls (Dune Chanter)?
    pub(crate) fn card_off_battlefield_gains_land_type(&self, card: &CardInstance, lt: crate::card::LandType) -> bool {
        use crate::effect::StaticEffect as SE;
        card.definition.is_land()
            && self.battlefield.find_by_id(card.id).is_none()
            && self.battlefield.iter().filter(|c| c.controller == card.owner).any(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, SE::OwnedLandCardsOffBattlefieldHaveLandType(t) if t == lt))
            })
    }

    /// CR 205.1a / 611.3a — does `card`, off the battlefield, gain card type
    /// `t` from a static its controller (a spell's caster) or owner controls
    /// (Biotransference: "creature spells you control and creature cards you
    /// own that aren't on the battlefield" are artifacts)? The type it must
    /// already have is read off the printed line, so the question never
    /// recurses. `false` behind the card-type lane on every other board.
    pub(crate) fn card_off_battlefield_gains_card_type(&self, card: &CardInstance, t: crate::card::CardType) -> bool {
        self.off_battlefield_card_type_adds(card).contains(&t)
    }

    /// The card types [`Self::card_off_battlefield_gains_card_type`] adds to
    /// `card`; empty (no walk) behind the card-type lane.
    pub(crate) fn off_battlefield_card_type_adds(
        &self,
        card: &CardInstance,
    ) -> smallvec::SmallVec<[crate::card::CardType; 1]> {
        use crate::effect::StaticEffect as SE;
        let mut out = smallvec::SmallVec::new();
        if !self.battlefield.has_card_type_changer(super::card_can_change_card_types_def)
            || self.battlefield.find_by_id(card.id).is_some()
        {
            return out;
        }
        let who = self.stack_caster_for_card(card.id).unwrap_or(card.owner);
        let has = |having: &crate::card::CardType| {
            card.definition.card_types.contains(having)
                || (*having == crate::card::CardType::Creature && card.definition.creature_off_battlefield)
        };
        for c in self.battlefield.iter().filter(|c| c.controller == who) {
            for sa in &c.definition.static_abilities {
                if let SE::OwnedCardsOffBattlefieldHaveCardType { having, add } = &sa.effect
                    && has(having)
                    && !card.definition.card_types.contains(add)
                    && !out.contains(add)
                {
                    out.push(add.clone());
                }
            }
        }
        out
    }

    /// [`super::layers::requirement_matches_card`] for a card off the
    /// battlefield, with the card types a static grants it there
    /// (Biotransference's creature cards are artifacts for Szarekh's mill).
    pub(crate) fn off_battlefield_card_matches(
        &self,
        req: &crate::card::SelectionRequirement,
        card: &CardInstance,
        controller: usize,
    ) -> bool {
        let adds = self.off_battlefield_card_type_adds(card);
        if adds.is_empty() {
            return super::layers::requirement_matches_card(req, card, controller);
        }
        let mut types = card.definition.card_types.to_vec();
        types.extend(adds);
        super::layers::requirement_matches_card_typed(req, card, controller, &types, None, None, None)
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
            return card.definition.card_types.contains(&t) || self.card_off_battlefield_gains_card_type(card, t);
        }
        self.computed_permanent_on(card)
            .map_or_else(|| card.definition.card_types.contains(&t), |cp| cp.card_types().contains(&t))
    }
}

#[cfg(test)]
mod tests {
    use crate::card::{CreatureType, SelectionRequirement as R};
    use crate::catalog;

    /// The graveyard-card requirement read and the walker agree on a type an
    /// owner's static grants off the battlefield (Maskwood Nexus) and on the
    /// last-known controller of a card whose ability is on the stack — the
    /// two disagreements strict debug pods asserted on (seeds 63029, 65019).
    #[test]
    fn graveyard_requirement_reads_agree_with_the_walker() {
        let mut g = crate::game::multi_player_game(3);
        g.add_card_to_battlefield(1, catalog::maskwood_nexus());
        let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let not_zombie = R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Zombie))));
        let card = g.players[1].graveyard.iter().find(|c| c.id == bear).unwrap().clone();
        // The debug assertion inside is the agreement check.
        assert!(!g.requirement_on_graveyard_card(&not_zombie, &card, 0, None), "every type: a Zombie");
        let mine = R::Creature.and(R::ControlledByYou);
        g.push_stack(
            crate::game::TriggerPush::new(bear, 1, crate::effect::Effect::Noop).build(),
        );
        assert!(g.requirement_on_graveyard_card(&mine, &card, 1, None));
        assert!(!g.requirement_on_graveyard_card(&mine, &card, 0, None));
    }
}
