//! Ravnica batch 7: Boros aggro (team pumps, Radiance +1/+1 reusing
//! `Selector::RadianceGroup`), a Dimir miller, and an upkeep-bounce aura.
//! Tests in `recent_b/recent_297`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, cost, generic, r, w};

fn soldier_token() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            ..Default::default()
        },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

fn your_creatures() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::ControlledByYou))
}

// ── Boros Radiance / aggro ──────────────────────────────────────────────────

/// Wojek Siren — {W} Instant. Radiance — target creature and each other
/// creature that shares a color with it get +1/+1 until end of turn.
pub fn wojek_siren() -> CardDefinition {
    CardDefinition {
        name: "Wojek Siren",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: Selector::RadianceGroup {
                subject: Box::new(target_filtered(R::Creature)),
            },
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Flame-Kin Zealot — {1}{R}{R}{W} 2/2 Elemental Berserker. When it enters,
/// creatures you control get +1/+1 and gain haste until end of turn.
pub fn flame_kin_zealot() -> CardDefinition {
    CardDefinition {
        name: "Flame-Kin Zealot",
        cost: cost(&[generic(1), r(), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: your_creatures(),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: your_creatures(),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Agrus Kos, Wojek Veteran — {3}{R}{W} legendary 3/3 Human Soldier. Whenever
/// he attacks, attacking red creatures get +2/+0 and attacking white creatures
/// get +0/+2 until end of turn.
pub fn agrus_kos_wojek_veteran() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Agrus Kos, Wojek Veteran",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::IsAttacking.and(R::HasColor(Color::Red))),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::EachPermanent(R::IsAttacking.and(R::HasColor(Color::White))),
                power: Value::Const(0),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Sunhome Guildmage — {R}{W} 2/2 Human Wizard. {1}{R}{W}: Creatures you
/// control get +1/+0 until end of turn. {2}{R}{W}: Create a 1/1 red and white
/// Soldier creature token with haste.
pub fn sunhome_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Sunhome Guildmage",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r(), w()]),
                effect: Effect::PumpPT {
                    what: your_creatures(),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r(), w()]),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(soldier_token()),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Dimir ───────────────────────────────────────────────────────────────────

/// Necromancer's Assistant — {2}{B} 3/1 Zombie. When it enters, mill three cards.
pub fn necromancers_assistant() -> CardDefinition {
    use crate::mana::b;
    CardDefinition {
        name: "Necromancer's Assistant",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Mill {
                who: Selector::You,
                amount: Value::Const(3),
            },
        }],
        ..Default::default()
    }
}

// ── Aura ────────────────────────────────────────────────────────────────────

/// Mark of Eviction — {U} Aura. Enchant creature. At the beginning of your
/// upkeep, return enchanted creature (and this Aura) to their owners' hands.
pub fn mark_of_eviction() -> CardDefinition {
    use crate::card::EquipBonus;
    use crate::mana::u;
    CardDefinition {
        name: "Mark of Eviction",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                // Return the creature first (This is still on the battlefield,
                // so `AttachedToMe` resolves) then bounce this Aura itself.
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::AttachedToMe(Box::new(Selector::This)),
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    },
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    },
                ]),
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}
