//! Dissension gap batch: a dozen guild commons/uncommons on existing
//! primitives (block restrictions, sac-fetch, all-colors, defender-drop, the
//! "sacrifice unless {C} was spent" convoke-color rider). Tests in
//! `recent_b/recent_302`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, Subtypes, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Haazda Exonerator — {W} 1/1 Human Cleric. {T}, Sacrifice this creature:
/// Destroy target Aura.
pub fn haazda_exonerator() -> CardDefinition {
    CardDefinition {
        name: "Haazda Exonerator",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::HasEnchantmentSubtype(
                    crate::card::EnchantmentSubtype::Aura,
                )),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ogre Gatecrasher — {3}{R} 3/3 Ogre Rogue. When it enters, destroy target
/// creature with defender.
pub fn ogre_gatecrasher() -> CardDefinition {
    CardDefinition {
        name: "Ogre Gatecrasher",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Defender))),
        })],
        ..Default::default()
    }
}

/// Whiptail Moloch — {4}{R} 6/3 Lizard. When it enters, it deals 3 damage to
/// target creature you control.
pub fn whiptail_moloch() -> CardDefinition {
    CardDefinition {
        name: "Whiptail Moloch",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 6,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
            amount: Value::Const(3),
        })],
        ..Default::default()
    }
}

/// Utvara Scalper — {1}{R} 1/2 Goblin Scout. Flying; attacks each combat if able.
pub fn utvara_scalper() -> CardDefinition {
    CardDefinition {
        name: "Utvara Scalper",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::MustAttack],
        ..Default::default()
    }
}

/// Gnat Alley Creeper — {2}{R} 3/1 Human Rogue. Can't be blocked by creatures
/// with flying.
pub fn gnat_alley_creeper() -> CardDefinition {
    CardDefinition {
        name: "Gnat Alley Creeper",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::HasKeyword(
            Keyword::Flying,
        )))],
        ..Default::default()
    }
}

/// Silkwing Scout — {2}{U} 2/1 Faerie Scout. Flying; {G}, Sacrifice this
/// creature: Search your library for a basic land, put it onto the battlefield
/// tapped, then shuffle.
pub fn silkwing_scout() -> CardDefinition {
    CardDefinition {
        name: "Silkwing Scout",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vesper Ghoul — {2}{B} 1/1 Zombie Druid. {T}, Pay 1 life: Add one mana of any
/// color.
pub fn vesper_ghoul() -> CardDefinition {
    CardDefinition {
        name: "Vesper Ghoul",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 1,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Patagia Viper — {3}{G} 2/1 Snake. Flying; ETB create two 1/1 green and blue
/// Snakes; ETB sacrifice it unless {U} was spent to cast it.
pub fn patagia_viper() -> CardDefinition {
    CardDefinition {
        name: "Patagia Viper",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: TokenDefinition {
                    name: "Snake".into(),
                    colors: vec![Color::Green, Color::Blue],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Snake],
                        ..Default::default()
                    },
                    power: 1,
                    toughness: 1,
                    ..Default::default()
                },
            }),
            sac_unless_color_spent(Color::Blue),
        ],
        ..Default::default()
    }
}

/// Squealing Devil — {1}{R} 2/1 Devil. Fear; ETB you may pay {X}, target
/// creature gets +X/+0; ETB sacrifice it unless {B} was spent to cast it.
pub fn squealing_devil() -> CardDefinition {
    CardDefinition {
        name: "Squealing Devil",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Devil],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Fear],
        triggered_abilities: vec![
            etb(Effect::MayPayX {
                description: "pay {X}: target creature gets +X/+0".into(),
                body: Box::new(Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::XFromCost,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
            }),
            sac_unless_color_spent(Color::Black),
        ],
        ..Default::default()
    }
}

/// Slaughterhouse Bouncer — {4}{B} 3/3 Ogre Warrior. Hellbent — when it dies,
/// if you have no cards in hand, target creature gets -3/-3 until end of turn.
pub fn slaughterhouse_bouncer() -> CardDefinition {
    CardDefinition {
        name: "Slaughterhouse Bouncer",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(
                Predicate::HellbentActive {
                    who: PlayerRef::You,
                },
            ),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Transguild Courier — {4} 3/3 Golem artifact creature that is all colors.
pub fn transguild_courier() -> CardDefinition {
    CardDefinition {
        name: "Transguild Courier",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        static_abilities: vec![crate::card::StaticAbility {
            description: "Transguild Courier is all colors.",
            effect: crate::card::StaticEffect::GrantAllColors {
                applies_to: Selector::This,
            },
        }],
        ..Default::default()
    }
}

/// Wakestone Gargoyle — {3}{W} 3/4 Gargoyle. Defender, flying; {1}{W}: Creatures
/// you control with defender can attack this turn as though they didn't have
/// defender.
pub fn wakestone_gargoyle() -> CardDefinition {
    CardDefinition {
        name: "Wakestone Gargoyle",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gargoyle],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Defender, Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::AttackDespiteDefenderThisTurn {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasKeyword(Keyword::Defender)),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// "When this creature enters, sacrifice it unless {color} was spent to cast
/// it" — the convoke-color rider (`Predicate::SourceCastWithColorSpent`).
fn sac_unless_color_spent(color: Color) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(
            Predicate::Not(Box::new(Predicate::SourceCastWithColorSpent {
                color,
                at_least: 1,
            })),
        ),
        effect: Effect::SacrificePermanent {
            what: Selector::This,
        },
    }
}
