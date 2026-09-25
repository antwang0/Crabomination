//! CR 614 — draw replacements that only apply to a draw from an empty
//! library (Out of the Tombs). Laboratory Maniac's win is read at the loss
//! site instead (`lose_to_empty_draw`).

use super::GameState;
use super::types::GameEvent;
use crate::card::{SelectionRequirement as R, Zone};
use crate::effect::{Effect, PlayerRef, Selector, StaticEffect, ZoneDest};

impl GameState {
    /// Out of the Tombs — "If you would draw a card while your library has no
    /// cards in it, instead return a creature card from your graveyard to the
    /// battlefield." `true` when a static applied and a creature came back;
    /// `false` leaves the empty draw to CR 104.3c ("if you can't, you lose").
    pub(crate) fn reanimate_instead_of_empty_draw(&mut self, p: usize, events: &mut Vec<GameEvent>) -> bool {
        let Some(source) = self.battlefield.iter().find_map(|c| {
            (c.controller == p
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(self.active_static(&sa.effect, c), Some(StaticEffect::ReanimateInsteadOfDrawFromEmpty))
                }))
            .then_some(c.id)
        }) else {
            return false;
        };
        if !self.players[p].graveyard.iter().any(|c| c.definition.is_creature()) {
            return false;
        }
        let ctx = super::effects::EffectContext::for_ability(source, p, None);
        let pick = Effect::MoveChosen {
            from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature },
            filter: None,
            count: crate::effect::Value::ONE,
            up_to: false,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        };
        let before = self.players[p].graveyard.len();
        let _ = self.run_effect(&pick, &ctx, events);
        self.players[p].graveyard.len() < before
    }
}
