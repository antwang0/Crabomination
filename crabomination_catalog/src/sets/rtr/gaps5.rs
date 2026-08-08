//! Return to Ravnica (RTR) gap wave 6: scavenge/unleash creatures, a life-lost
//! X-counter body, and the Aura package (stat-drain + upkeep drain, granted
//! activated abilities, an ETB-token Aura). Tests in `classic_sets/rtr`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{scavenge, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w};

/// Deadbridge Goliath — {2}{G}{G} 5/5 Insect. Scavenge {4}{G}{G}.
pub fn deadbridge_goliath() -> CardDefinition {
    CardDefinition {
        name: "Deadbridge Goliath",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        activated_abilities: vec![scavenge(cost(&[generic(4), g(), g()]))],
        ..Default::default()
    }
}

/// Archweaver — {5}{G}{G} 5/5 Spider with reach and trample.
pub fn archweaver() -> CardDefinition {
    CardDefinition {
        name: "Archweaver",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        ..Default::default()
    }
}

/// Lotleth Troll — {B}{G} 2/1 Zombie Troll with trample. `Discard a creature
/// card: Put a +1/+1 counter on this creature.` `{B}: Regenerate this creature.`
pub fn lotleth_troll() -> CardDefinition {
    CardDefinition {
        name: "Lotleth Troll",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Troll],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![
            ActivatedAbility {
                discard_cost: Some((R::Creature, 1)),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::Regenerate {
                    what: Selector::This,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Cryptborn Horror — {1}{B/R}{B/R} 0/0 Horror with trample. Enters with X +1/+1
/// counters, X = total life your opponents lost this turn.
pub fn cryptborn_horror() -> CardDefinition {
    CardDefinition {
        name: "Cryptborn Horror",
        cost: cost(&[
            generic(1),
            hybrid(Color::Black, Color::Red),
            hybrid(Color::Black, Color::Red),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::LifeLostThisTurn(PlayerRef::EachOpponent),
        )),
        ..Default::default()
    }
}

/// Hellhole Flailer — {1}{B}{R} 3/2 Ogre Warrior with unleash. `{2}{B}{R},
/// Sacrifice this creature: It deals damage equal to its power to target player
/// or planeswalker.` (The sac-as-cost stamps `Value::SacrificedPower`.)
pub fn hellhole_flailer() -> CardDefinition {
    CardDefinition {
        name: "Hellhole Flailer",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Unleash],
        triggered_abilities: vec![crate::effect::shortcut::unleash()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), r()]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::SacrificedPower,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// A stat-drain Aura that also drains the enchanted creature's controller each
/// upkeep (CR 702.6e aura-granted step trigger, keyed on the host controller).
fn drain_aura(
    name: &'static str,
    mana: crate::mana::ManaCost,
    pt: (i32, i32),
    loss: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
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
            power: pt.0,
            toughness: pt.1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Const(loss),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Soul Tithe — {1}{W} Aura. Enchant nonland permanent. At the beginning of the
/// enchanted permanent's controller's upkeep, that player sacrifices it unless
/// they pay {X}, where X is its mana value (CR 701.16).
pub fn soul_tithe() -> CardDefinition {
    CardDefinition {
        name: "Soul Tithe",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Permanent.and(R::Nonland)),
        },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::SacrificeSourceUnlessPayManaValue,
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Chronic Flooding — {1}{U} Aura. Enchant land. Whenever enchanted land becomes
/// tapped, its controller mills three cards.
pub fn chronic_flooding() -> CardDefinition {
    CardDefinition {
        name: "Chronic Flooding",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Land),
        },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                effect: Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Stab Wound — {2}{B} Aura. Enchanted creature gets -2/-2; its controller loses
/// 2 life at the beginning of their upkeep.
pub fn stab_wound() -> CardDefinition {
    drain_aura("Stab Wound", cost(&[generic(2), b()]), (-2, -2), 2)
}

/// An Aura that pumps the enchanted creature and grants it a mana-activated
/// evergreen keyword until end of turn (Pursuit of Flight, Deviant Glee).
fn granted_keyword_aura(
    name: &'static str,
    mana: crate::mana::ManaCost,
    pt: (i32, i32),
    ability_cost: crate::mana::ManaCost,
    keyword: Keyword,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
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
            power: pt.0,
            toughness: pt.1,
            activated_abilities: vec![ActivatedAbility {
                mana_cost: ability_cost,
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Pursuit of Flight — {1}{R} Aura. Enchanted creature gets +2/+2 and has
/// "{U}: This creature gains flying until end of turn."
pub fn pursuit_of_flight() -> CardDefinition {
    granted_keyword_aura(
        "Pursuit of Flight",
        cost(&[generic(1), r()]),
        (2, 2),
        cost(&[u()]),
        Keyword::Flying,
    )
}

/// Deviant Glee — {B} Aura. Enchanted creature gets +2/+1 and has "{R}: This
/// creature gains trample until end of turn."
pub fn deviant_glee() -> CardDefinition {
    granted_keyword_aura(
        "Deviant Glee",
        cost(&[b()]),
        (2, 1),
        cost(&[r()]),
        Keyword::Trample,
    )
}

/// Knightly Valor — {4}{W} Aura. ETB: create a 2/2 white Knight token with
/// vigilance. Enchanted creature gets +2/+2 and has vigilance.
pub fn knightly_valor() -> CardDefinition {
    CardDefinition {
        name: "Knightly Valor",
        cost: cost(&[generic(4), w()]),
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
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Vigilance],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Knight".into(),
                    power: 2,
                    toughness: 2,
                    keywords: vec![Keyword::Vigilance],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Knight],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
        }],
        ..Default::default()
    }
}
