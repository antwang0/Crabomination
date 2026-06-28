//! A twenty-seventh wave — Bloomburrow (BLB), Final Fantasy (FIN), and a
//! Duskmourn straggler, all on existing primitives: vanilla keyword bodies,
//! ETB/dies token mints, attack-trigger drains, and board-count self-pumps.
//! Tests in `crabomination/src/tests/recent27.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, Predicate, SelectionRequirement,
    Selector, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::{etb, etb_draw, on_attack_drain, on_attack_gain_life, on_dies};
use crate::effect::{Duration, PlayerRef, StaticEffect};
use crate::mana::{cost, generic, g, hybrid, u, w, Color};

/// A 1/1 colorless Hero token (FIN).
fn hero_token() -> TokenDefinition {
    TokenDefinition {
        name: "Hero".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hero], ..Default::default() },
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
        subtypes: Subtypes { creature_types: vec![CreatureType::Rabbit], ..Default::default() },
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
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        keywords,
        ..Default::default()
    }
}

/// Brightblade Stoat — {1}{W} 2/2 Weasel Soldier, first strike + lifelink.
pub fn brightblade_stoat() -> CardDefinition {
    creature(
        "Brightblade Stoat", cost(&[generic(1), w()]),
        vec![CreatureType::Weasel, CreatureType::Soldier], 2, 2,
        vec![Keyword::FirstStrike, Keyword::Lifelink],
    )
}

/// Shrike Force — {2}{W} 1/3 Bird Knight, flying + double strike + vigilance.
pub fn shrike_force() -> CardDefinition {
    creature(
        "Shrike Force", cost(&[generic(2), w()]),
        vec![CreatureType::Bird, CreatureType::Knight], 1, 3,
        vec![Keyword::Flying, Keyword::DoubleStrike, Keyword::Vigilance],
    )
}

/// Pond Prophet — {G/U}{G/U} 1/1 Frog Advisor. When it enters, draw a card.
pub fn pond_prophet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_draw(1)],
        ..creature(
            "Pond Prophet",
            cost(&[hybrid(Color::Green, Color::Blue), hybrid(Color::Green, Color::Blue)]),
            vec![CreatureType::Frog, CreatureType::Advisor], 1, 1, vec![],
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
            "Hecteyes", cost(&[generic(1), crate::mana::b()]),
            vec![CreatureType::Ooze, CreatureType::Horror], 1, 1, vec![],
        )
    }
}

/// Moonrise Cleric — {1}{W/B}{W/B} 2/3 Bat Cleric, flying. Attack → gain 1 life.
pub fn moonrise_cleric() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack_gain_life(1)],
        ..creature(
            "Moonrise Cleric",
            cost(&[generic(1), hybrid(Color::White, Color::Black), hybrid(Color::White, Color::Black)]),
            vec![CreatureType::Bat, CreatureType::Cleric], 2, 3, vec![Keyword::Flying],
        )
    }
}

/// Agate-Blade Assassin — {1}{B} 1/3 Lizard Assassin. Attack → defending player
/// loses 1 life and you gain 1 life.
pub fn agate_blade_assassin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack_drain(1)],
        ..creature(
            "Agate-Blade Assassin", cost(&[generic(1), crate::mana::b()]),
            vec![CreatureType::Lizard, CreatureType::Assassin], 1, 3, vec![],
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
        ..creature("Gigantoad", cost(&[generic(3), g()]), vec![CreatureType::Frog], 4, 4, vec![])
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
        ..creature("Loporrit Scout", cost(&[generic(2), g()]), vec![CreatureType::Rabbit, CreatureType::Scout], 3, 2, vec![])
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
            cost(&[generic(3), hybrid(Color::Green, Color::White), hybrid(Color::Green, Color::White)]),
            vec![CreatureType::Rabbit, CreatureType::Citizen], 3, 2, vec![],
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
        ..creature("Dragoon's Wyvern", cost(&[generic(2), u()]), vec![CreatureType::Drake], 2, 1, vec![Keyword::Flying])
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
        ..creature("Dwarven Castle Guard", cost(&[generic(1), w()]), vec![CreatureType::Dwarf, CreatureType::Soldier], 2, 1, vec![])
    }
}
