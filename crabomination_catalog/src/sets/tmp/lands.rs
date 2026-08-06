//! Tempest (TMP) lands — the "slow" dual cycle that stays tapped a turn, the
//! two damage-taplands, and Ghost Town. Tests in `classic_sets/tmp`.

use crate::card::{ActivatedAbility, CardDefinition, CardType, StaticAbility};
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate, Selector, StaticEffect, ZoneDest};
use crate::mana::{Color, ManaCost};

use super::super::{painland, tap_add_colorless};

/// Damage tapland (Caldera Lake / Pine Barrens): enters tapped, `{T}: Add {C}`,
/// and two painful colored taps.
fn tapped_painland(name: &'static str, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        ..painland(name, a, b)
    }
}

/// "Slow" dual (Cinder Marsh cycle): `{T}: Add {C}`, plus a colored tap that
/// costs the land its next untap.
fn slow_dual(name: &'static str, a: Color, b: Color) -> CardDefinition {
    let colored = |color: Color| ActivatedAbility {
        tap_cost: true,
        effect: Effect::Seq(vec![
            Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![color]) },
            Effect::SkipNextUntap { what: Selector::This },
        ]),
        ..Default::default()
    };
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_colorless(), colored(a), colored(b)],
        ..Default::default()
    }
}

/// Caldera Lake — {U}/{R} damage tapland.
pub fn caldera_lake() -> CardDefinition {
    tapped_painland("Caldera Lake", Color::Blue, Color::Red)
}

/// Pine Barrens — {B}/{G} damage tapland.
pub fn pine_barrens() -> CardDefinition {
    tapped_painland("Pine Barrens", Color::Black, Color::Green)
}

/// Cinder Marsh — {B}/{R} slow dual.
pub fn cinder_marsh() -> CardDefinition {
    slow_dual("Cinder Marsh", Color::Black, Color::Red)
}

/// Mogg Hollows — {R}/{G} slow dual.
pub fn mogg_hollows() -> CardDefinition {
    slow_dual("Mogg Hollows", Color::Red, Color::Green)
}

/// Rootwater Depths — {U}/{B} slow dual.
pub fn rootwater_depths() -> CardDefinition {
    slow_dual("Rootwater Depths", Color::Blue, Color::Black)
}

/// Salt Flats — {W}/{B} damage tapland.
pub fn salt_flats() -> CardDefinition {
    tapped_painland("Salt Flats", Color::White, Color::Black)
}

/// Skyshroud Forest — {G}/{U} slow dual.
pub fn skyshroud_forest() -> CardDefinition {
    slow_dual("Skyshroud Forest", Color::Green, Color::Blue)
}

/// Scabland — {R}/{W} damage tapland.
pub fn scabland() -> CardDefinition {
    tapped_painland("Scabland", Color::Red, Color::White)
}

/// Ghost Town — {T}: Add {C}. {0}: bounce it, but only on someone else's turn
/// (an untap-step reset that dodges a "doesn't untap" lock).
pub fn ghost_town() -> CardDefinition {
    CardDefinition {
        name: "Ghost Town",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: ManaCost::new(vec![]),
                condition: Some(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Thalakos Lowlands — {W}/{U} slow dual.
pub fn thalakos_lowlands() -> CardDefinition {
    slow_dual("Thalakos Lowlands", Color::White, Color::Blue)
}

/// Vec Townships — {G}/{W} slow dual.
pub fn vec_townships() -> CardDefinition {
    slow_dual("Vec Townships", Color::Green, Color::White)
}
