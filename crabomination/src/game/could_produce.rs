//! CR 106.7 — the mana a land "could produce" (Exotic Orchard, Fellwar Stone:
//! "one mana of any color that a land an opponent controls could produce").

use super::GameState;
use crate::effect::{Effect, ManaPayload};
use crate::mana::ColorSet;

impl GameState {
    /// Every color a land one of `seat`'s opponents controls could produce
    /// (CR 106.7): its mana abilities' colors plus its basic land types'. An
    /// opponent's own "could produce" land reads *its* opponents' lands one
    /// level down and no further, so two Orchards facing each other with no
    /// other land produce nothing (the rule's own example).
    pub(crate) fn colors_opponent_lands_could_produce(&self, seat: usize) -> ColorSet {
        self.lands_could_produce(seat, true)
    }

    fn lands_could_produce(&self, seat: usize, recurse: bool) -> ColorSet {
        let mut set = ColorSet::empty();
        // Opponents only (a 2HG teammate's lands are not "an opponent's"),
        // through `types_lands_could_produce_where`: computed land types
        // (Urborg, Blood Moon), chosen colors (Thriving lands), and each
        // payload read for ITS controller (Command Tower makes that player's
        // identity, not all five).
        for q in self.opponents_of(seat) {
            set = set.union(self.types_lands_could_produce_where(q, |_, _| true).0);
            if recurse && self.controls_orchard_land(q) {
                set = set.union(self.lands_could_produce(q, false));
            }
        }
        set
    }

    /// Does `q` control a land whose mana ability reads its opponents' lands
    /// (Exotic Orchard)?
    fn controls_orchard_land(&self, q: usize) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller == q
                && self.computed_has_card_type(c, crate::card::CardType::Land)
                && c.definition.activated_abilities.iter().any(|ab| {
                    matches!(&ab.effect, Effect::AddMana { pool: ManaPayload::AnyColorOpponentCouldProduce, .. })
                })
        })
    }
}
