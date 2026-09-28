//! CR 601.2f — the cost INCREASES every cast pays, whatever base cost its
//! path chose (mana cost, flashback, escape, foretell, a may-play grant).
//! The hand path applies them inline; the zone paths share this.

use super::GameState;
use super::types::Target;
use crate::card::CardInstance;
use crate::mana::ManaCost;

impl GameState {
    /// Add the generic and colored taxes (Thalia, Sphere of Resistance, the
    /// Invasion Leeches) a cast of `card` by `p` pays, before its reductions.
    pub(crate) fn add_spell_taxes(&self, p: usize, card: &CardInstance, target: Option<&Target>, cost: &mut ManaCost) {
        cost.add_generic(super::actions::extra_cost_for_spell(self, p, card, target));
        cost.symbols.extend(super::actions::colored_spell_tax_for_spell(self, p, card).symbols);
    }
}
