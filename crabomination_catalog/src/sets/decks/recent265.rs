//! DMU / SNC / MID / NEO gap batch — vanilla-ish creatures, firebreathing,
//! kicker ETBs, a Backup creature, an anthem lord, and small value bodies, all
//! on existing primitives. Tests in `tests/recent_b/recent265.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{add_any_one_color, backup, each_your_creature, etb, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
};
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color};

/// Bonebreaker Giant — {4}{R} 4/4 Giant vanilla.
pub fn bonebreaker_giant() -> CardDefinition {
    CardDefinition {
        name: "Bonebreaker Giant",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 4,
        toughness: 4,
        ..Default::default()
    }
}

/// Gnottvold Recluse — {2}{G} 4/2 Spider with reach.
pub fn gnottvold_recluse() -> CardDefinition {
    CardDefinition {
        name: "Gnottvold Recluse",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        ..Default::default()
    }
}

/// Deathbloom Gardener — {2}{G} 1/1 Elf Druid with deathtouch + any-color dork.
pub fn deathbloom_gardener() -> CardDefinition {
    CardDefinition {
        name: "Deathbloom Gardener",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: add_any_one_color(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Battlefly Swarm — {B} 1/1 Phyrexian Insect. Flying; {B}: gains deathtouch EOT.
pub fn battlefly_swarm() -> CardDefinition {
    CardDefinition {
        name: "Battlefly Swarm",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Duct Crawler — {R} 1/1 Insect. {1}{R}: target creature can't block this.
pub fn duct_crawler() -> CardDefinition {
    CardDefinition {
        name: "Duct Crawler",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::CantBlockSourceThisTurn { target: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Charismatic Vanguard — {2}{W} 3/2 Dwarf Soldier. {4}{W}: your team +1/+1 EOT.
pub fn charismatic_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Charismatic Vanguard",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: Effect::PumpPT {
                what: each_your_creature(),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cabaretti Initiate — {G} 1/2 Raccoon Citizen. {2}{R/W}: double strike EOT.
pub fn cabaretti_initiate() -> CardDefinition {
    CardDefinition {
        name: "Cabaretti Initiate",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Raccoon, CreatureType::Citizen],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), hybrid(Color::Red, Color::White)]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Serpent-Blade Assailant — {2}{G} 2/1 Elf Warrior. Backup 1, deathtouch.
pub fn serpent_blade_assailant() -> CardDefinition {
    CardDefinition {
        name: "Serpent-Blade Assailant",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![backup(1, vec![Keyword::Deathtouch])],
        ..Default::default()
    }
}

/// Rhox Pikemaster — {2}{W}{W} 3/3 Rhino Soldier. First strike; other Soldiers
/// you control have first strike.
pub fn rhox_pikemaster() -> CardDefinition {
    CardDefinition {
        name: "Rhox Pikemaster",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "Other Soldier creatures you control have first strike.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Soldier)
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                keyword: Keyword::FirstStrike,
            },
        }],
        ..Default::default()
    }
}

/// Witty Roastmaster — {2}{R} 3/2 Devil Citizen. Whenever another creature you
/// control enters, deal 1 to each opponent.
pub fn witty_roastmaster() -> CardDefinition {
    CardDefinition {
        name: "Witty Roastmaster",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Devil, CreatureType::Citizen],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Yavimaya Iconoclast — {1}{G} 3/2 Elf. Trample; kicker {R}; ETB if kicked,
/// +1/+1 and haste until end of turn.
pub fn yavimaya_iconoclast() -> CardDefinition {
    CardDefinition {
        name: "Yavimaya Iconoclast",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample, Keyword::Kicker(cost(&[r()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Vineshaper Prodigy — {1}{G} 2/2 Elf Druid. Kicker {1}{U}; ETB if kicked,
/// look at top three, take one, rest to bottom.
pub fn vineshaper_prodigy() -> CardDefinition {
    CardDefinition {
        name: "Vineshaper Prodigy",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Kicker(cost(&[generic(1), u()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: false,
                pick_filter: None,
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Shield-Wall Sentinel — {4} 1/3 Golem artifact creature. Defender; ETB may
/// search library for a defender creature to hand.
pub fn shield_wall_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Shield-Wall Sentinel",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for a creature card with defender?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::Creature.and(R::HasKeyword(Keyword::Defender)),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..Default::default()
    }
}

/// Kami of Industry — {4}{R} 3/6 Spirit. ETB return an artifact card with mana
/// value 3 or less from your graveyard to the battlefield with haste; sacrifice
/// it at the next end step.
pub fn kami_of_industry() -> CardDefinition {
    CardDefinition {
        name: "Kami of Industry",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 3,
        toughness: 6,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    R::HasCardType(CardType::Artifact)
                        .and(R::InGraveyard)
                        .and(R::ManaValueAtMost(3)),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::SacrificePermanent { what: Selector::Target(0) }),
            },
        ]))],
        ..Default::default()
    }
}

/// Wingmantle Chaplain — {3}{W} 0/3 Human Cleric. Defender; ETB makes a Bird
/// per defender you control, and mints a Bird whenever another defender enters.
pub fn wingmantle_chaplain() -> CardDefinition {
    let bird = || TokenDefinition {
        name: "Bird".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Wingmantle Chaplain",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(
                        R::Creature.and(R::HasKeyword(Keyword::Defender)).and(R::ControlledByYou),
                    )),
                    filter: R::Creature,
                },
                definition: bird(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::HasKeyword(Keyword::Defender)).and(R::OtherThanSource),
                    }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: bird(),
                },
            },
        ],
        ..Default::default()
    }
}
