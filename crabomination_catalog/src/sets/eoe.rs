//! Edge of Eternities — Exhaust (CR 702.177). "Exhaust — [Cost]: [Effect]"
//! means "[Cost]: [Effect]. Activate only once" (per game). Modeled via the
//! `ActivatedAbility.exhaust` flag + `CardInstance.exhausted_abilities`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, Subtypes,
    TokenDefinition,
};
use crate::effect::shortcut::target;
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{cost, g, generic, r, u};

/// Shared exhaust ability: "Exhaust — [cost]: Put N +1/+1 counters on this."
fn exhaust_self_counters(mana: crate::mana::ManaCost, n: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        exhaust: true,
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(n),
        },
        ..Default::default()
    }
}

/// Camera Launcher — {3} Artifact Creature — Construct 2/2. "Exhaust — {3}:
/// Put a +1/+1 counter on this creature. Create a 1/1 colorless Thopter
/// artifact creature token with flying."
pub fn camera_launcher() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Camera Launcher",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: thopter },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hazard of the Dunes — {3}{G} 4/4 Wurm. Trample, reach. "Exhaust — {6}{G}:
/// Put three +1/+1 counters on this creature."
pub fn hazard_of_the_dunes() -> CardDefinition {
    CardDefinition {
        name: "Hazard of the Dunes",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Reach],
        activated_abilities: vec![exhaust_self_counters(cost(&[generic(6), g()]), 3)],
        ..Default::default()
    }
}

/// Prowcatcher Specialist — {1}{R} 2/1 Goblin Warrior. Haste. "Exhaust —
/// {3}{R}: Put two +1/+1 counters on this creature."
pub fn prowcatcher_specialist() -> CardDefinition {
    CardDefinition {
        name: "Prowcatcher Specialist",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![exhaust_self_counters(cost(&[generic(3), r()]), 2)],
        ..Default::default()
    }
}

/// Greenbelt Guardian — {1}{G} 2/2 Elf Ranger. "{G}: Target creature gains
/// trample until end of turn." plus "Exhaust — {3}{G}: Put three +1/+1
/// counters on this creature."
pub fn greenbelt_guardian() -> CardDefinition {
    CardDefinition {
        name: "Greenbelt Guardian",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Ranger],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::GrantKeyword {
                    what: target(),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            exhaust_self_counters(cost(&[generic(3), g()]), 3),
        ],
        ..Default::default()
    }
}

/// Pacesetter Paragon — {2}{R} 2/3 Human Pilot. "Exhaust — {2}{R}: Put a
/// +1/+1 counter on this creature. It gains double strike until end of turn."
pub fn pacesetter_paragon() -> CardDefinition {
    CardDefinition {
        name: "Pacesetter Paragon",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Keen Buccaneer — {2}{U} 2/3 Octopus Pirate. Vigilance. "Exhaust — {1}{U}:
/// Draw a card, then discard a card. Put a +1/+1 counter on this creature."
pub fn keen_buccaneer() -> CardDefinition {
    CardDefinition {
        name: "Keen Buccaneer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Octopus, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skystreak Engineer — {1}{U} 1/3 Human Pilot. Flying. "Exhaust — {4}{U}:
/// Put two +1/+1 counters on this creature."
pub fn skystreak_engineer() -> CardDefinition {
    CardDefinition {
        name: "Skystreak Engineer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![exhaust_self_counters(cost(&[generic(4), u()]), 2)],
        ..Default::default()
    }
}

/// Mai, Jaded Edge — {1}{R} 1/3 Legendary Human Noble. Prowess. "Exhaust —
/// {3}: Put a double strike counter on Mai."
pub fn mai_jaded_edge() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Mai, Jaded Edge",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Prowess],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            exhaust: true,
            effect: Effect::AddKeywordCounter {
                what: Selector::This,
                keyword: Keyword::DoubleStrike,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stampeding Scurryfoot — {G} 1/1 Mouse. "Exhaust — {3}{G}: Put a +1/+1
/// counter on this creature. Create a 3/3 green Elephant creature token."
pub fn stampeding_scurryfoot() -> CardDefinition {
    use crabomination_base::mana::Color;
    let elephant = TokenDefinition {
        name: "Elephant".into(),
        power: 3,
        toughness: 3,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Stampeding Scurryfoot",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mouse], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: elephant },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mindspring Merfolk — {U} 1/1 Merfolk Wizard. "Exhaust — {X}{U}{U}, {T}:
/// Draw X cards. Put a +1/+1 counter on each Merfolk creature you control."
pub fn mindspring_merfolk() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::mana::x;
    CardDefinition {
        name: "Mindspring Merfolk",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[x(), u(), u()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::XFromCost },
                Effect::AddCounter {
                    what: Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Merfolk)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
