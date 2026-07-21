//! Ravnica block (RAV/GPT) gap cards: the guild bounce-land cycle plus simple
//! creatures and spells filling the `set_gaps.py` remainder.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{b, cost, generic, r, u, w, Color};

use super::super::etb_tap;

/// A Karoo bounce-land (CR — Ravnica block): enters tapped, returns a land you
/// control to hand on entry, and taps for two guild colors at once.
fn bounce_land(name: &'static str, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![a, b]) },
            ..Default::default()
        }],
        triggered_abilities: vec![
            etb_tap(),
            etb(Effect::Move {
                what: target_filtered(R::Land.and(R::ControlledByYou)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        ],
        ..Default::default()
    }
}

pub fn dimir_aqueduct() -> CardDefinition { bounce_land("Dimir Aqueduct", Color::Blue, Color::Black) }
pub fn golgari_rot_farm() -> CardDefinition {
    bounce_land("Golgari Rot Farm", Color::Black, Color::Green)
}
pub fn selesnya_sanctuary() -> CardDefinition {
    bounce_land("Selesnya Sanctuary", Color::Green, Color::White)
}
pub fn boros_garrison() -> CardDefinition { bounce_land("Boros Garrison", Color::Red, Color::White) }
pub fn gruul_turf() -> CardDefinition { bounce_land("Gruul Turf", Color::Red, Color::Green) }
pub fn izzet_boilerworks() -> CardDefinition {
    bounce_land("Izzet Boilerworks", Color::Blue, Color::Red)
}
pub fn orzhov_basilica() -> CardDefinition {
    bounce_land("Orzhov Basilica", Color::White, Color::Black)
}

/// Benevolent Ancestor — {2}{W} 0/4 Spirit with Defender. `{T}: Prevent the
/// next 1 damage that would be dealt to any target this turn.`
pub fn benevolent_ancestor() -> CardDefinition {
    CardDefinition {
        name: "Benevolent Ancestor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage { target: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Carrion Howler — {3}{B} 2/2 Zombie Wolf. `Pay 1 life: This creature gets
/// +2/-1 until end of turn.`
pub fn carrion_howler() -> CardDefinition {
    CardDefinition {
        name: "Carrion Howler",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wolf],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Conclave Phalanx — {4}{W} 2/4 Human Soldier with Convoke. When it enters,
/// you gain 1 life for each creature you control.
pub fn conclave_phalanx() -> CardDefinition {
    CardDefinition {
        name: "Conclave Phalanx",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::CreatureCountControlledBy(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Dogpile — {3}{R} Instant. Deals damage to any target equal to the number of
/// attacking creatures you control.
pub fn dogpile() -> CardDefinition {
    CardDefinition {
        name: "Dogpile",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_any(),
            amount: Value::count(Selector::EachPermanent(R::IsAttacking.and(R::ControlledByYou))),
        },
        ..Default::default()
    }
}

/// Dimir Cutpurse — {1}{U}{B} 2/2 Spirit. Whenever it deals combat damage to a
/// player, that player discards a card and you draw a card.
pub fn dimir_cutpurse() -> CardDefinition {
    CardDefinition {
        name: "Dimir Cutpurse",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                    random: false,
                },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Clinging Darkness — {1}{B} Aura. Enchant creature. Enchanted creature gets
/// -4/-1.
pub fn clinging_darkness() -> CardDefinition {
    CardDefinition {
        name: "Clinging Darkness",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: -4, toughness: -1, ..Default::default() }),
        ..Default::default()
    }
}

/// Consult the Necrosages — {1}{U}{B} Sorcery. Choose one — target player draws
/// two cards; or target player discards two cards.
pub fn consult_the_necrosages() -> CardDefinition {
    CardDefinition {
        name: "Consult the Necrosages",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::Draw {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(2),
                },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(2),
                    random: false,
                },
            ],
            min: 1,
            max: 1,
            allow_repeats: false,
        },
        ..Default::default()
    }
}
