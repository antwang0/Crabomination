//! Foundations (FDN) gap batch 9 — the Guildgate land cycle, a choose-color
//! anthem artifact (Heraldic Banner, on the new `AnthemForChosenColor` static),
//! two Equipment (ETB-attach and landfall), a targeted-draw beater, and a
//! color-cast growth creature. Tests in `tests/recent210.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EquipBonus, Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate,
};
use crate::mana::{Color, cost, generic};

/// A Guildgate: Land — Gate, enters tapped, taps for one of two colors.
fn guildgate(name: &'static str, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Gate],
            ..Default::default()
        },
        activated_abilities: vec![super::super::tap_add(a), super::super::tap_add(b)],
        triggered_abilities: vec![super::super::etb_tap()],
        ..Default::default()
    }
}

pub fn azorius_guildgate() -> CardDefinition {
    guildgate("Azorius Guildgate", Color::White, Color::Blue)
}
pub fn dimir_guildgate() -> CardDefinition {
    guildgate("Dimir Guildgate", Color::Blue, Color::Black)
}
pub fn rakdos_guildgate() -> CardDefinition {
    guildgate("Rakdos Guildgate", Color::Black, Color::Red)
}
pub fn gruul_guildgate() -> CardDefinition {
    guildgate("Gruul Guildgate", Color::Red, Color::Green)
}
pub fn selesnya_guildgate() -> CardDefinition {
    guildgate("Selesnya Guildgate", Color::Green, Color::White)
}
pub fn orzhov_guildgate() -> CardDefinition {
    guildgate("Orzhov Guildgate", Color::White, Color::Black)
}
pub fn izzet_guildgate() -> CardDefinition {
    guildgate("Izzet Guildgate", Color::Blue, Color::Red)
}
pub fn golgari_guildgate() -> CardDefinition {
    guildgate("Golgari Guildgate", Color::Black, Color::Green)
}
pub fn boros_guildgate() -> CardDefinition {
    guildgate("Boros Guildgate", Color::Red, Color::White)
}
pub fn simic_guildgate() -> CardDefinition {
    guildgate("Simic Guildgate", Color::Green, Color::Blue)
}

/// Heraldic Banner — {3} Artifact. As it enters, choose a color. Creatures you
/// control of the chosen color get +1/+0. {T}: Add one mana of the chosen color.
pub fn heraldic_banner() -> CardDefinition {
    CardDefinition {
        name: "Heraldic Banner",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::ChooseColorForSelf)],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control of the chosen color get +1/+0.",
            effect: StaticEffect::AnthemForChosenColor {
                power: 1,
                toughness: 0,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::ChosenColorOfSource,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Pirate's Cutlass — {3} Equipment. When it enters, attach it to target Pirate
/// you control. Equipped creature gets +2/+1. Equip {2}.
pub fn pirates_cutlass() -> CardDefinition {
    CardDefinition {
        name: "Pirate's Cutlass",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::HasCreatureType(CreatureType::Pirate).and(R::ControlledByYou)),
        })],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 1,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Adventuring Gear — {1} Equipment. Landfall — whenever a land you control
/// enters, equipped creature gets +2/+2 until end of turn. Equip {1}.
pub fn adventuring_gear() -> CardDefinition {
    CardDefinition {
        name: "Adventuring Gear",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Gnarlback Rhino — {2}{G}{G} 4/4 Rhino. Trample; whenever you cast a spell
/// that targets it, draw a card.
pub fn gnarlback_rhino() -> CardDefinition {
    CardDefinition {
        name: "Gnarlback Rhino",
        cost: cost(&[generic(2), crate::mana::g(), crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourControl),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Mold Adder — {G} 1/1 Fungus Snake. Whenever an opponent casts a blue or
/// black spell, you may put a +1/+1 counter on it.
pub fn mold_adder() -> CardDefinition {
    CardDefinition {
        name: "Mold Adder",
        cost: cost(&[crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fungus, CreatureType::Snake],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::Blue).or(R::HasColor(Color::Black)),
                },
            ),
            effect: Effect::MayDo {
                description: "put a +1/+1 counter on Mold Adder".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..Default::default()
    }
}
