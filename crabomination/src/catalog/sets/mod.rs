//! One submodule per Magic set, named by the set's three-letter code.
//! Helpers shared across all set modules live here.

use crate::card::{ActivatedAbility, SpellEffect};
use crate::mana::{Color, ManaCost};

pub(super) fn tap_add(color: Color) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        mana_cost: ManaCost::default(),
        effects: vec![SpellEffect::AddMana { colors: vec![color] }],
    }
}

pub(super) fn no_abilities() -> Vec<ActivatedAbility> {
    vec![]
}

pub mod ap;
pub mod dis;
pub mod gpt;
pub mod inv;
pub mod lea;
pub mod ogw;
pub mod pc2;
pub mod por;
pub mod rav;
pub mod rtr;
pub mod zen;
pub mod ths;
