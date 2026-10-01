//! "Any color / type that a land (or Gate) you control could produce"
//! (CR 106.7): Gond Gate, Reflecting Pool, Naga Vitalist, Incubation Druid.

use crate::effect::{Effect, ManaPayload};
use crate::game::GameState;
use crate::mana::ColorSet;

/// A payload that itself reads "what your lands could produce" — skipped
/// while computing that set, so two Reflecting Pools add nothing (CR 106.7).
fn reads_your_lands(pool: &ManaPayload) -> bool {
    match pool {
        ManaPayload::AnyColorAGateYouControlCouldProduce | ManaPayload::AnyTypeALandYouControlCouldProduce(_) => true,
        ManaPayload::Restricted(inner, _) => reads_your_lands(inner),
        _ => false,
    }
}

/// The colors `pool` can make for seat `p`, and whether it can make {C}.
fn pool_types(state: &GameState, p: usize, pool: &ManaPayload) -> (ColorSet, bool) {
    match pool {
        ManaPayload::Colorless(_) => (ColorSet::empty(), true),
        // CR 903.4 — Command Tower makes only its controller's identity.
        ManaPayload::AnyColorInCommanderIdentity => (state.players[p].commander_identity, false),
        ManaPayload::Restricted(inner, _) => pool_types(state, p, inner),
        other => (crate::game::actions::payload_produced_colors(other), false),
    }
}

fn effect_types(state: &GameState, p: usize, e: &Effect, out: &mut (ColorSet, bool)) {
    match e {
        Effect::AddMana { pool, .. } if !reads_your_lands(pool) => {
            let (cs, c) = pool_types(state, p, pool);
            out.0 = out.0.union(cs);
            out.1 |= c;
        }
        Effect::Seq(v) => v.iter().for_each(|e| effect_types(state, p, e, out)),
        Effect::If { then, else_, .. } => {
            effect_types(state, p, then, out);
            effect_types(state, p, else_, out);
        }
        _ => {}
    }
}

impl GameState {
    /// The colors (and whether {C}) the mana abilities of `p`'s lands — only
    /// its Gates when `gates_only` — and their chosen colors could produce.
    pub(crate) fn types_lands_could_produce(&self, p: usize, gates_only: bool) -> (ColorSet, bool) {
        use crate::card::LandType;
        let mut out = (ColorSet::empty(), false);
        for c in self.battlefield.iter().filter(|c| {
            c.controller == p
                && self.computed_has_card_type(c, crate::card::CardType::Land)
                && (!gates_only || self.permanent_has_land_type(c, LandType::Gate))
        }) {
            for a in &c.definition.activated_abilities {
                effect_types(self, p, &a.effect, &mut out);
            }
            if let Some(col) = c.chosen_color {
                out.0.insert(col);
            }
        }
        out
    }

    /// Gond Gate — the colors `p`'s Gates' mana abilities (and their chosen
    /// colors) could produce, in WUBRG order.
    pub fn colors_gates_could_produce(&self, p: usize) -> Vec<crate::mana::Color> {
        self.types_lands_could_produce(p, true).0.iter().collect()
    }
}
