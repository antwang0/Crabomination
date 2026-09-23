//! CR 106.7 — the mana a land "could produce" (Exotic Orchard, Fellwar Stone:
//! "one mana of any color that a land an opponent controls could produce").

use super::GameState;
use crate::card::LandType;
use crate::effect::{Effect, ManaPayload};
use crate::mana::{Color, ColorSet};

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
        for land in self.battlefield.iter().filter(|c| c.controller != seat && c.definition.is_land()) {
            for lt in &land.definition.subtypes.land_types {
                let c = match lt {
                    LandType::Plains => Color::White,
                    LandType::Island => Color::Blue,
                    LandType::Swamp => Color::Black,
                    LandType::Mountain => Color::Red,
                    LandType::Forest => Color::Green,
                    _ => continue,
                };
                set = set.union(ColorSet::single(c));
            }
            for ab in &land.definition.activated_abilities {
                let colors = match &ab.effect {
                    Effect::AddMana { pool: ManaPayload::AnyColorOpponentCouldProduce, .. } => {
                        if recurse {
                            self.lands_could_produce(land.controller, false)
                        } else {
                            ColorSet::empty()
                        }
                    }
                    e => super::actions::effect_produced_colors(e),
                };
                set = set.union(colors);
            }
        }
        set
    }
}
