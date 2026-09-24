//! Commander: the cards the **Token Triumph** starter deck (SCD, Emmara,
//! Soul of the Accord) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_emmara.rs`.
//!
//! No open residuals.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, cost, g, generic, w, x};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn token(name: &str, color: Color, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: 1,
        toughness: 1,
        keywords,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

fn lifelink_soldier() -> Arc<TokenDefinition> {
    token("Soldier", Color::White, vec![CreatureType::Soldier], vec![Keyword::Lifelink])
}

fn your_creatures() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn other_creatures_get_one() -> StaticAbility {
    StaticAbility {
        description: "Other creatures you control get +1/+1.",
        effect: StaticEffect::PumpPT {
            applies_to: Selector::EachPermanent(your_creatures().and(R::OtherThanSource)),
            power: 1,
            toughness: 1,
        },
    }
}

/// Camaraderie — gain X life and draw X cards, X your creatures; your
/// creatures get +1/+1 until end of turn.
pub fn camaraderie() -> CardDefinition {
    let x = || Value::CountOf(Box::new(Selector::EachPermanent(your_creatures())));
    CardDefinition {
        name: "Camaraderie",
        cost: cost(&[generic(4), g(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: x() },
            Effect::Draw { who: Selector::You, amount: x() },
            Effect::PumpPT {
                what: Selector::EachPermanent(your_creatures()),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Champion of Lambholt — creatures with power less than its power can't
/// block creatures you control; another creature of yours entering gives it
/// a +1/+1 counter.
pub fn champion_of_lambholt() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures with power less than this creature's power can't block creatures you control.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(your_creatures()),
                keyword: Keyword::CantBeBlockedByPowerLessThanGreatestAmong(Box::new(
                    R::Creature.and(R::HasName("Champion of Lambholt".into())),
                )),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(1) },
        }],
        ..creature(
            "Champion of Lambholt",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            1,
            1,
        )
    }
}

/// Citywide Bust — destroy all creatures with toughness 4 or greater.
pub fn citywide_bust() -> CardDefinition {
    CardDefinition {
        name: "Citywide Bust",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.and(R::ToughnessAtLeast(4))),
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        },
        ..Default::default()
    }
}

/// Dauntless Escort — sacrifice it: your creatures gain indestructible until
/// end of turn.
pub fn dauntless_escort() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(your_creatures()),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Dauntless Escort", cost(&[generic(1), g(), w()]), vec![CreatureType::Rhino, CreatureType::Soldier], 3, 3)
    }
}

/// Emmara, Soul of the Accord — whenever it becomes tapped, a 1/1 lifelink
/// Soldier.
pub fn emmara_soul_of_the_accord() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: lifelink_soldier() },
        }],
        ..creature("Emmara, Soul of the Accord", cost(&[g(), w()]), vec![CreatureType::Elf, CreatureType::Cleric], 2, 2)
    }
}

/// Harvest Season — up to X basic lands onto the battlefield tapped, X your
/// tapped creatures.
pub fn harvest_season() -> CardDefinition {
    CardDefinition {
        name: "Harvest Season",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::Land.and(R::HasSupertype(Supertype::Basic)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            count: Value::CountOf(Box::new(Selector::EachPermanent(your_creatures().and(R::Tapped)))),
        },
        ..Default::default()
    }
}

/// Jade Mage — {2}{G}: a 1/1 Saproling.
pub fn jade_mage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: token("Saproling", Color::Green, vec![CreatureType::Saproling], vec![]),
            },
            ..Default::default()
        }],
        ..creature("Jade Mage", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Shaman], 2, 1)
    }
}

/// Jaspera Sentinel — reach; {T}, tap an untapped creature you control: one
/// mana of any color.
pub fn jaspera_sentinel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            tap_permanents_cost: Some((R::Creature.and(R::OtherThanSource), 1)),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(1)) },
            ..Default::default()
        }],
        ..creature("Jaspera Sentinel", cost(&[g()]), vec![CreatureType::Elf, CreatureType::Rogue], 1, 2)
    }
}

/// Leafkin Druid — {T}: {G}, or {G}{G} with four or more creatures.
pub fn leafkin_druid() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Green,
                    Value::IfAtLeast {
                        value: Box::new(Value::CountOf(Box::new(Selector::EachPermanent(your_creatures())))),
                        threshold: 4,
                        then: Box::new(Value::Const(2)),
                        else_: Box::new(Value::Const(1)),
                    },
                ),
            },
            ..Default::default()
        }],
        ..creature("Leafkin Druid", cost(&[generic(1), g()]), vec![CreatureType::Elemental, CreatureType::Druid], 0, 3)
    }
}

/// Loyal Guardian — trample; lieutenant: at the beginning of combat on your
/// turn, if you control your commander, a +1/+1 counter on each creature you
/// control.
pub fn loyal_guardian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::ControlsOwnCommander { who: PlayerRef::You },
                then: Box::new(Effect::AddCounter {
                    what: Selector::EachPermanent(your_creatures()),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Loyal Guardian", cost(&[generic(4), g()]), vec![CreatureType::Rhino], 4, 4)
    }
}

/// Maja, Bretagard Protector — other creatures you control get +1/+1;
/// landfall: a 1/1 Human Warrior.
pub fn maja_bretagard_protector() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![other_creatures_get_one()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: token("Human Warrior", Color::White, vec![CreatureType::Human, CreatureType::Warrior], vec![]),
            },
        }],
        ..creature(
            "Maja, Bretagard Protector",
            cost(&[generic(2), g(), w(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// March of the Multitudes — convoke; X 1/1 lifelink Soldiers.
pub fn march_of_the_multitudes() -> CardDefinition {
    CardDefinition {
        name: "March of the Multitudes",
        cost: cost(&[x(), g(), w(), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Convoke],
        effect: Effect::CreateToken { who: PlayerRef::You, count: Value::XFromCost, definition: lifelink_soldier() },
        ..Default::default()
    }
}

/// Presence of Gond — enchanted creature has "{T}: create a 1/1 Elf
/// Warrior".
pub fn presence_of_gond() -> CardDefinition {
    CardDefinition {
        name: "Presence of Gond",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: token("Elf Warrior", Color::Green, vec![CreatureType::Elf, CreatureType::Warrior], vec![]),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Trostani Discordant — other creatures you control get +1/+1; enters with
/// two 1/1 lifelink Soldiers; at your end step each player gains control of
/// all creatures they own.
pub fn trostani_discordant() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![other_creatures_get_one()],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: lifelink_soldier() },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::EachPlayerDoes {
                    who: PlayerRef::EachPlayer,
                    body: Box::new(Effect::GainControl {
                        what: Selector::OwnedBy {
                            who: PlayerRef::You,
                            filter: R::Creature.and(R::Not(Box::new(R::ControlledByYou))),
                        },
                        to: None,
                        duration: Duration::Permanent,
                    }),
                },
            },
        ],
        ..creature("Trostani Discordant", cost(&[generic(3), g(), w()]), vec![CreatureType::Dryad], 1, 4)
    }
}

/// Valor in Akros — whenever a creature you control enters, your creatures
/// get +1/+1 until end of turn.
pub fn valor_in_akros() -> CardDefinition {
    CardDefinition {
        name: "Valor in Akros",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(your_creatures()),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}
