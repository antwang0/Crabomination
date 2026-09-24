//! Gond Gate's "any color that a Gate you control could produce" (CR 106.7).

use crate::game::GameState;

impl GameState {
    /// Gond Gate — the colors `p`'s Gates' mana abilities (and their chosen
    /// colors) could produce, in WUBRG order.
    pub fn colors_gates_could_produce(&self, p: usize) -> Vec<crate::mana::Color> {
        use crate::card::LandType;
        let mut set = crate::mana::ColorSet::empty();
        for c in self.battlefield.iter().filter(|c| {
            c.controller == p && c.definition.subtypes.land_types.contains(&LandType::Gate)
        }) {
            for a in &c.definition.activated_abilities {
                if matches!(
                    a.effect,
                    crate::effect::Effect::AddMana {
                        pool: crate::effect::ManaPayload::AnyColorAGateYouControlCouldProduce,
                        ..
                    }
                ) {
                    continue;
                }
                set = set.union(crate::game::actions::effect_produced_colors(&a.effect));
            }
            if let Some(col) = c.chosen_color {
                set.insert(col);
            }
        }
        set.iter().collect()
    }
}
