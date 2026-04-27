//! One submodule per Magic set, named by the set's three-letter code.
//! Helpers shared across all set modules live here.

use crate::card::ActivatedAbility;
use crate::effect::{Effect, ManaPayload, PlayerRef};
use crate::mana::{Color, ManaCost};

pub fn tap_add(color: Color) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        mana_cost: ManaCost::default(),
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![color]),
        },
        once_per_turn: false,
        sorcery_speed: false,
    }
}

pub fn no_abilities() -> Vec<ActivatedAbility> {
    vec![]
}

pub mod all;
pub mod ap;
pub mod arn;
pub mod dis;
pub mod fem;
pub mod gpt;
pub mod ice;
pub mod inv;
pub mod lea;
pub mod m11;
pub mod ogw;
pub mod pc2;
pub mod por;
pub mod rav;
pub mod rtr;
pub mod ths;
pub mod tmp;
pub mod zen;
pub mod decks;
pub mod mod_set;
