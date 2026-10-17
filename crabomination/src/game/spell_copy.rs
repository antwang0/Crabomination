//! CR 707.10 — "a copy of a spell … copies … all decisions made for it,
//! including modes, targets, the value of X, and additional or alternative
//! costs", and an effect of the copy that refers to objects used to pay its
//! costs uses the original's (Fling's discarded card). Every copy is a fresh
//! `CardInstance`, so these per-cast fields are carried over explicitly.
//! Not copied: anything about mana actually spent (a copy has none — converge
//! is 0, the 2015-08-25 rulings) and whether the spell was CAST (it wasn't).

use crate::card::CardInstance;

/// The per-cast decisions a copy inherits.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CastChoices {
    kicked: bool,
    kick_count: u32,
    kicked_options: Vec<u8>,
    spree_modes: Vec<u8>,
    entwined: bool,
    bargained: bool,
    gift_promised: bool,
    squad_count: u32,
    split_cast: Option<u8>,
    cast_discarded_mana_value: Option<u32>,
    cast_collected_evidence: bool,
    /// A type chosen as a cost was paid (Celestial Reunion's behold).
    chosen_creature_type: Option<crate::card::CreatureType>,
}

impl CastChoices {
    pub(crate) fn of(c: &CardInstance) -> Self {
        Self {
            kicked: c.kicked,
            kick_count: c.kick_count,
            kicked_options: c.kicked_options.clone(),
            spree_modes: c.spree_modes.clone(),
            entwined: c.entwined,
            bargained: c.bargained,
            gift_promised: c.gift_promised,
            squad_count: c.squad_count,
            split_cast: c.split_cast,
            cast_discarded_mana_value: c.cast_discarded_mana_value,
            cast_collected_evidence: c.cast_collected_evidence,
            chosen_creature_type: c.chosen_creature_type,
        }
    }

    /// Stamp the decisions onto a fresh copy. A spell cast with none of them
    /// (nearly all) writes nothing.
    pub(crate) fn apply(&self, c: &mut CardInstance) {
        if *self == Self::default() {
            return;
        }
        c.kicked = self.kicked;
        c.kick_count = self.kick_count;
        c.kicked_options = self.kicked_options.clone();
        c.spree_modes = self.spree_modes.clone();
        c.entwined = self.entwined;
        c.bargained = self.bargained;
        c.gift_promised = self.gift_promised;
        c.squad_count = self.squad_count;
        c.split_cast = self.split_cast;
        c.cast_discarded_mana_value = self.cast_discarded_mana_value;
        c.cast_collected_evidence = self.cast_collected_evidence;
        c.chosen_creature_type = self.chosen_creature_type;
    }
}

impl CastChoices {
    pub(crate) fn chosen_creature_type(&self) -> Option<crate::card::CreatureType> {
        self.chosen_creature_type
    }
}

impl crate::game::GameState {
    /// What one shot of `effect` spends on the permanent it aims at, for the
    /// copy-spread default: a removal shot takes the whole object, a fixed or
    /// X damage shot its amount. `None` for every other effect (keep aim).
    fn copy_shot_weight(effect: &crate::effect::Effect, x: u32) -> Option<u32> {
        use crate::effect::{Effect, Value};
        match effect {
            Effect::Destroy { .. } | Effect::Exile { .. } => Some(u32::MAX),
            Effect::Move { to: crate::effect::ZoneDest::Hand(_), .. } => Some(u32::MAX),
            Effect::DealDamage { amount: Value::Const(n), .. } => Some((*n).max(0) as u32),
            Effect::DealDamage { amount: Value::XFromCost, .. } => Some(x),
            _ => None,
        }
    }

    /// Bot default for the next storm-style copy (CR 702.40a "you may choose
    /// new targets for the copies"): once the shots already `aimed` at the
    /// original's opposing permanent cover it — any removal, or damage summing
    /// to lethal (CR 120.6) — default to another opposing permanent so the
    /// rest don't fizzle on a dead target (CR 608.2b).
    pub(crate) fn storm_copy_default(
        &self,
        def: &crate::card::CardDefinition,
        caster: usize,
        original: &Option<crate::game::types::Target>,
        aimed: &[(crate::card::CardId, u32)],
        x: u32,
    ) -> Option<crate::game::types::Target> {
        use crate::game::types::Target;
        let Some(weight) = Self::copy_shot_weight(&def.effect, x) else {
            return original.clone();
        };
        let Some(Target::Permanent(first)) = original else {
            return original.clone();
        };
        let covered = |s: &Self, id: crate::card::CardId, aimed: &[(crate::card::CardId, u32)]| {
            let spent = aimed.iter().filter(|(a, _)| *a == id).fold(0u32, |n, (_, w)| n.saturating_add(*w));
            spent > 0 && (weight == u32::MAX || s.lethal_damage_needed(id, false).is_none_or(|need| spent >= need))
        };
        if covered(self, *first, aimed) {
            let mut avoid = vec![crate::card::CardId(u32::MAX)];
            avoid.extend(aimed.iter().map(|(a, _)| *a).filter(|a| covered(self, *a, aimed)));
            match self.auto_target_for_effect_avoiding_set_x(&def.effect, caster, &avoid, x) {
                Some(Target::Permanent(other))
                    if self.battlefield_find(other).is_some_and(|c| !self.same_team(c.controller, caster)) =>
                {
                    Some(Target::Permanent(other))
                }
                _ => original.clone(),
            }
        } else {
            original.clone()
        }
    }

    /// Record one shot (the original or a copy) at `target` for
    /// [`Self::storm_copy_default`].
    pub(crate) fn note_copy_aim(
        def: &crate::card::CardDefinition,
        x: u32,
        target: &Option<crate::game::types::Target>,
        aimed: &mut Vec<(crate::card::CardId, u32)>,
    ) {
        if let (Some(w), Some(crate::game::types::Target::Permanent(id))) = (Self::copy_shot_weight(&def.effect, x), target) {
            aimed.push((*id, w));
        }
    }
}
