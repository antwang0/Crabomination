//! The Doctor Who villains' primitives (Masters of Evil):
//!
//! - "Put it onto the battlefield face down under your control. It's a 2/2
//!   Cyberman artifact creature" (CR 708.2: the effect that puts it face down
//!   names its characteristics). A permanent is turned face down in place
//!   (Cyber Conversion); a card anywhere else enters the battlefield face down
//!   under the effect's controller.
//! - The Valeyard's doubled villainous choices (CR 701.55).
//! - Ashad's "first nonlegendary artifact spell each turn has casualty N".

use super::{EffectContext, EntityRef};
use crate::effect::{PlayerRef, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::types::GameEvent;

impl GameState {
    /// `Effect::PutFaceDownAsCyberman`.
    pub(super) fn put_face_down_as_cyberman(
        &mut self,
        what: &Selector,
        tapped: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        let dest = ZoneDest::Battlefield { controller: PlayerRef::Seat(ctx.controller), tapped };
        for ent in self.resolve_selector(what, ctx) {
            match ent {
                EntityRef::Permanent(id) => {
                    if let Some(c) = self.battlefield_find_mut(id) {
                        c.turn_face_down_as_cyberman();
                    } else if let Some(c) = self.find_card_anywhere_mut(id) {
                        // A selector can name an off-battlefield card as a
                        // permanent id (`Target(0)` of a graveyard slot).
                        c.turn_face_down_as_cyberman();
                        self.move_card_to(id, &dest, ctx, events);
                    }
                }
                EntityRef::Card(id) => {
                    if let Some(c) = self.find_card_anywhere_mut(id) {
                        c.turn_face_down_as_cyberman();
                    }
                    self.move_card_to(id, &dest, ctx, events);
                }
                EntityRef::Player(_) => {}
            }
        }
    }
}

impl GameState {
    /// The Valeyard — how many extra times `p` faces each villainous choice:
    /// one per `OpponentsFaceVillainousChoicesTwice` among the permanents
    /// `p`'s opponents control (CR 701.55).
    pub(crate) fn villainous_choice_repeats(&self, p: usize) -> usize {
        self.battlefield
            .iter()
            .filter(|c| c.controller != p && !self.same_team(p, c.controller))
            .flat_map(|c| &c.definition.static_abilities)
            .filter(|sa| matches!(sa.effect, crate::effect::StaticEffect::OpponentsFaceVillainousChoicesTwice))
            .count()
    }
}

impl GameState {
    /// Ashad, the Lone Cyberman — "the first nonlegendary artifact spell you
    /// cast each turn has casualty N" (CR 702.153): `None` once `seat` has cast
    /// one this turn, or with no such static on their side.
    pub(crate) fn first_nonlegendary_artifact_casualty(
        &self,
        seat: usize,
        def: &crate::card::CardDefinition,
    ) -> Option<u32> {
        let nonlegendary_artifact = |d: &crate::card::CardDefinition| {
            d.is_artifact() && !d.supertypes.contains(&crate::card::Supertype::Legendary)
        };
        if !nonlegendary_artifact(def) {
            return None;
        }
        let n = self
            .battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .flat_map(|c| &c.definition.static_abilities)
            .find_map(|sa| match sa.effect {
                crate::effect::StaticEffect::FirstNonlegendaryArtifactSpellHasCasualty(n) => Some(n),
                _ => None,
            })?;
        let already = self.players[seat]
            .spell_ids_cast_this_turn
            .iter()
            .any(|id| self.find_card_anywhere(*id).is_some_and(|c| nonlegendary_artifact(&c.definition)));
        (!already).then_some(n)
    }
}
