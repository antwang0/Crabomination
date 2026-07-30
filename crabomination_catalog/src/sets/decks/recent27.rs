//! A twenty-seventh wave — Bloomburrow (BLB), Final Fantasy (FIN), and a
//! Duskmourn straggler, all on existing primitives: vanilla keyword bodies,
//! ETB/dies token mints, attack-trigger drains, and board-count self-pumps.
//! Tests in `crabomination/src/tests/recent27.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, Keyword, Predicate,
    SelectionRequirement, Selector, StaticAbility, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::{
    etb, etb_draw, on_attack, on_attack_drain, on_attack_gain_life, on_dies, target_filtered,
};
use crate::effect::{Duration, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, hybrid, u, w};

/// A 1/1 colorless Hero token (FIN).
fn hero_token() -> TokenDefinition {
    TokenDefinition {
        name: "Hero".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hero],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// A 1/1 white Rabbit token (BLB).
fn rabbit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Rabbit".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn creature(
    name: &'static str,
    cst: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: cst,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power,
        toughness,
        keywords,
        ..Default::default()
    }
}

/// Brightblade Stoat — {1}{W} 2/2 Weasel Soldier, first strike + lifelink.
pub fn brightblade_stoat() -> CardDefinition {
    creature(
        "Brightblade Stoat",
        cost(&[generic(1), w()]),
        vec![CreatureType::Weasel, CreatureType::Soldier],
        2,
        2,
        vec![Keyword::FirstStrike, Keyword::Lifelink],
    )
}

/// Shrike Force — {2}{W} 1/3 Bird Knight, flying + double strike + vigilance.
pub fn shrike_force() -> CardDefinition {
    creature(
        "Shrike Force",
        cost(&[generic(2), w()]),
        vec![CreatureType::Bird, CreatureType::Knight],
        1,
        3,
        vec![Keyword::Flying, Keyword::DoubleStrike, Keyword::Vigilance],
    )
}

/// Pond Prophet — {G/U}{G/U} 1/1 Frog Advisor. When it enters, draw a card.
pub fn pond_prophet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_draw(1)],
        ..creature(
            "Pond Prophet",
            cost(&[
                hybrid(Color::Green, Color::Blue),
                hybrid(Color::Green, Color::Blue),
            ]),
            vec![CreatureType::Frog, CreatureType::Advisor],
            1,
            1,
            vec![],
        )
    }
}

/// Hecteyes — {1}{B} 1/1 Ooze Horror. When it enters, each opponent discards a card.
pub fn hecteyes() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: false,
        })],
        ..creature(
            "Hecteyes",
            cost(&[generic(1), crate::mana::b()]),
            vec![CreatureType::Ooze, CreatureType::Horror],
            1,
            1,
            vec![],
        )
    }
}

/// Moonrise Cleric — {1}{W/B}{W/B} 2/3 Bat Cleric, flying. Attack → gain 1 life.
pub fn moonrise_cleric() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack_gain_life(1)],
        ..creature(
            "Moonrise Cleric",
            cost(&[
                generic(1),
                hybrid(Color::White, Color::Black),
                hybrid(Color::White, Color::Black),
            ]),
            vec![CreatureType::Bat, CreatureType::Cleric],
            2,
            3,
            vec![Keyword::Flying],
        )
    }
}

/// Agate-Blade Assassin — {1}{B} 1/3 Lizard Assassin. Attack → defending player
/// loses 1 life and you gain 1 life.
pub fn agate_blade_assassin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack_drain(1)],
        ..creature(
            "Agate-Blade Assassin",
            cost(&[generic(1), crate::mana::b()]),
            vec![CreatureType::Lizard, CreatureType::Assassin],
            1,
            3,
            vec![],
        )
    }
}

/// Gigantoad — {3}{G} 4/4 Frog. +2/+2 while you control seven or more lands.
pub fn gigantoad() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::card::StaticAbility {
            description: "Gets +2/+2 while you control seven or more lands.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(7),
                },
                power: 2,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..creature(
            "Gigantoad",
            cost(&[generic(3), g()]),
            vec![CreatureType::Frog],
            4,
            4,
            vec![],
        )
    }
}

/// Loporrit Scout — {2}{G} 3/2 Rabbit Scout. Whenever another creature you
/// control enters, this creature gets +1/+1 until end of turn.
pub fn loporrit_scout() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Loporrit Scout",
            cost(&[generic(2), g()]),
            vec![CreatureType::Rabbit, CreatureType::Scout],
            3,
            2,
            vec![],
        )
    }
}

/// Head of the Homestead — {3}{G/W}{G/W} 3/2 Rabbit Citizen. When it enters,
/// create two 1/1 white Rabbit creature tokens.
pub fn head_of_the_homestead() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: rabbit_token(),
        })],
        ..creature(
            "Head of the Homestead",
            cost(&[
                generic(3),
                hybrid(Color::Green, Color::White),
                hybrid(Color::Green, Color::White),
            ]),
            vec![CreatureType::Rabbit, CreatureType::Citizen],
            3,
            2,
            vec![],
        )
    }
}

/// Dragoon's Wyvern — {2}{U} 2/1 Drake, flying. When it enters, create a 1/1
/// colorless Hero creature token.
pub fn dragoons_wyvern() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: hero_token(),
        })],
        ..creature(
            "Dragoon's Wyvern",
            cost(&[generic(2), u()]),
            vec![CreatureType::Drake],
            2,
            1,
            vec![Keyword::Flying],
        )
    }
}

/// Dwarven Castle Guard — {1}{W} 2/1 Dwarf Soldier. When it dies, create a 1/1
/// colorless Hero creature token.
pub fn dwarven_castle_guard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: hero_token(),
        })],
        ..creature(
            "Dwarven Castle Guard",
            cost(&[generic(1), w()]),
            vec![CreatureType::Dwarf, CreatureType::Soldier],
            2,
            1,
            vec![],
        )
    }
}

/// Coeurl — {1}{W} 2/2 Cat Beast. {1}{W}, {T}: Tap target nonenchantment creature.
pub fn coeurl() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature.and(
                    SelectionRequirement::Not(Box::new(SelectionRequirement::HasCardType(
                        CardType::Enchantment,
                    ))),
                )),
            },
            ..Default::default()
        }],
        ..creature(
            "Coeurl",
            cost(&[generic(1), w()]),
            vec![CreatureType::Cat, CreatureType::Beast],
            2,
            2,
            vec![],
        )
    }
}

/// Ahriman — {2}{B} 2/2 Eye Horror, flying + deathtouch. {3}, Sacrifice another
/// creature or artifact: Draw a card.
pub fn ahriman() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sac_other_filter: Some((
                SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
                1,
            )),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..creature(
            "Ahriman",
            cost(&[generic(2), b()]),
            vec![CreatureType::Eye, CreatureType::Horror],
            2,
            2,
            vec![Keyword::Flying, Keyword::Deathtouch],
        )
    }
}

/// Gaelicat — {2}{W} 1/3 Cat, flying + vigilance. +2/+0 while you control two or
/// more artifacts.
pub fn gaelicat() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Gets +2/+0 while you control two or more artifacts.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(2),
                },
                power: 2,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..creature(
            "Gaelicat",
            cost(&[generic(2), w()]),
            vec![CreatureType::Cat],
            1,
            3,
            vec![Keyword::Flying, Keyword::Vigilance],
        )
    }
}

/// Scorpion Sentinel — {1}{U} 1/4 Artifact Creature — Robot Scorpion. +3/+0
/// while you control seven or more lands.
pub fn scorpion_sentinel() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "Gets +3/+0 while you control seven or more lands.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(7),
                },
                power: 3,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..creature(
            "Scorpion Sentinel",
            cost(&[generic(1), u()]),
            vec![CreatureType::Robot, CreatureType::Scorpion],
            1,
            4,
            vec![],
        )
    }
}

/// Thistledown Players — {2}{W} 3/3 Mouse Bard. Whenever it attacks, untap
/// target nonland permanent.
pub fn thistledown_players() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Untap {
            what: target_filtered(SelectionRequirement::Nonland),
            up_to: None,
        })],
        ..creature(
            "Thistledown Players",
            cost(&[generic(2), w()]),
            vec![CreatureType::Mouse, CreatureType::Bard],
            3,
            3,
            vec![],
        )
    }
}

/// Warren Elder — {1}{W} 2/2 Rabbit Cleric. {3}{W}: Creatures you control get
/// +1/+1 until end of turn.
pub fn warren_elder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Warren Elder",
            cost(&[generic(1), w()]),
            vec![CreatureType::Rabbit, CreatureType::Cleric],
            2,
            2,
            vec![],
        )
    }
}

/// Jumbo Cactuar — {5}{G}{G} 1/7 Plant. Whenever it attacks, it gets +9999/+0
/// until end of turn (10,000 Needles).
pub fn jumbo_cactuar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(9999),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Jumbo Cactuar",
            cost(&[generic(5), g(), g()]),
            vec![CreatureType::Plant],
            1,
            7,
            vec![],
        )
    }
}

/// Outlaw Medic — {1}{W} 1/3 Human Rogue, lifelink. When it dies, draw a card.
pub fn outlaw_medic() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..creature(
            "Outlaw Medic",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            1,
            3,
            vec![Keyword::Lifelink],
        )
    }
}

/// Sterling Supplier — {4}{W} 3/4 Bird Soldier, flying. When it enters, put a
/// +1/+1 counter on another target creature you control.
pub fn sterling_supplier() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            kind: crate::card::CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..creature(
            "Sterling Supplier",
            cost(&[generic(4), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            3,
            4,
            vec![Keyword::Flying],
        )
    }
}

/// Shrieking Drake — {U} 1/1 Drake, flying. When it enters, return a creature
/// you control to its owner's hand.
pub fn shrieking_drake() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..creature(
            "Shrieking Drake",
            cost(&[u()]),
            vec![CreatureType::Drake],
            1,
            1,
            vec![Keyword::Flying],
        )
    }
}

/// Oasis Gardener — {3} 2/2 Artifact Creature — Scarecrow. ETB: gain 2 life.
/// {T}: Add one mana of any color.
pub fn oasis_gardener() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..creature(
            "Oasis Gardener",
            cost(&[generic(3)]),
            vec![CreatureType::Scarecrow],
            2,
            2,
            vec![],
        )
    }
}

/// Discerning Peddler — {1}{R} 2/2 Human Rogue. When it enters, you may discard
/// a card. If you do, draw a card.
pub fn discerning_peddler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "discard a card, then draw a card".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ])),
        })],
        ..creature(
            "Discerning Peddler",
            cost(&[generic(1), crate::mana::r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            2,
            vec![],
        )
    }
}
