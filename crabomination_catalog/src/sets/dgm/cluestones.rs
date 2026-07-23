//! Dragon's Maze (DGM) Cluestone cycle — {3} guild mana rocks that tap for one
//! of two colors and sacrifice for a card. Tests in `classic_sets/dgm`.

use crate::card::{ActivatedAbility, CardDefinition, CardType, Effect, Value};
use crate::effect::{ManaPayload, PlayerRef, Selector};
use crate::mana::{colored, cost, generic, Color};

fn cluestone(name: &'static str, c1: Color, c2: Color) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![c1, c2], Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[colored(c1), colored(c2)]),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

pub fn azorius_cluestone() -> CardDefinition { cluestone("Azorius Cluestone", Color::White, Color::Blue) }
pub fn dimir_cluestone() -> CardDefinition { cluestone("Dimir Cluestone", Color::Blue, Color::Black) }
pub fn rakdos_cluestone() -> CardDefinition { cluestone("Rakdos Cluestone", Color::Black, Color::Red) }
pub fn gruul_cluestone() -> CardDefinition { cluestone("Gruul Cluestone", Color::Red, Color::Green) }
pub fn selesnya_cluestone() -> CardDefinition { cluestone("Selesnya Cluestone", Color::Green, Color::White) }
pub fn orzhov_cluestone() -> CardDefinition { cluestone("Orzhov Cluestone", Color::White, Color::Black) }
pub fn izzet_cluestone() -> CardDefinition { cluestone("Izzet Cluestone", Color::Blue, Color::Red) }
pub fn golgari_cluestone() -> CardDefinition { cluestone("Golgari Cluestone", Color::Black, Color::Green) }
pub fn boros_cluestone() -> CardDefinition { cluestone("Boros Cluestone", Color::Red, Color::White) }
pub fn simic_cluestone() -> CardDefinition { cluestone("Simic Cluestone", Color::Green, Color::Blue) }
