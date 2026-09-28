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

/// The spell a half-card cast puts on the stack, as the cost walks read it:
/// the half's name, cost and card types only (CR 709.3 split halves, 715.3
/// an Adventure, an Omen). The card stays whole on the stack — only the
/// cost filters (Thalia's "noncreature", an instant/sorcery discount) see
/// this.
pub(crate) fn half_spell_probe(
    card: &CardInstance,
    name: Option<crate::static_str_serde::StaticStr>,
    cost: &ManaCost,
    types: &[crate::card::CardType],
) -> CardInstance {
    let mut def = (*card.definition.arc()).clone();
    if let Some(n) = name {
        def.name = n;
    }
    def.cost = cost.clone();
    def.card_types = types.to_vec();
    def.subtypes = Default::default();
    def.supertypes = Vec::new();
    let mut probe = CardInstance::new(card.id, def, card.owner);
    probe.controller = card.controller;
    probe
}

impl GameState {
    /// CR 601.2f for a half-card cast: the taxes, then the generic and
    /// colored reductions, read against [`half_spell_probe`].
    pub(crate) fn apply_half_cast_cost_modifiers(
        &self,
        p: usize,
        probe: &CardInstance,
        target: Option<&Target>,
        from_graveyard: bool,
        cost: &mut ManaCost,
    ) {
        self.add_spell_taxes(p, probe, target, cost);
        let less = super::actions::cost_reduction_for_spell_full(self, p, probe, target, from_graveyard, false);
        if less > 0 {
            cost.reduce_generic(less);
        }
        super::actions::apply_colored_cost_statics(self, p, probe, cost);
    }
}

/// CR 119.4 — a life payment greater than 0 needs at least that much life;
/// paying 0 life is always legal, even at 0 or less life (a seat kept in
/// the game by Platinum Angel still casts its spells).
pub(crate) fn life_payable(life: i32, amount: u32) -> bool {
    amount == 0 || life >= amount as i32
}
