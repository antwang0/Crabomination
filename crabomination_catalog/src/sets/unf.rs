//! Unfinity (UNF) — Attractions (CR 717) and the cards that open them.
//! Attraction cards live in the Attraction deck, not the main deck; a visit
//! ability fires when the controller's d6 roll matches one of the card's
//! lit-up numbers. Tests in `classic_sets/unf`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, Value, ZoneDest,
    shortcut::{draw, etb, target_filtered},
};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// An Attraction: an artifact with no mana cost that only ever enters from the
/// Attraction deck, plus its lit-up numbers and its visit ability.
fn attraction(name: &'static str, lights: &[u8], visit: Effect) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Attraction],
            ..Default::default()
        },
        attraction_lights: lights.to_vec(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::VisitedAttraction, EventScope::SelfSource),
            effect: visit,
        }],
        ..Default::default()
    }
}

fn balloon_token() -> TokenDefinition {
    TokenDefinition {
        name: "Balloon".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Balloon],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn clown_robot_token() -> TokenDefinition {
    TokenDefinition {
        name: "Clown Robot".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot],
            ..Default::default()
        },
        ..Default::default()
    }
}

// ── Attractions ─────────────────────────────────────────────────────────────

/// Balloon Stand — Visit: make a flying Balloon, or sacrifice one to grant
/// flying.
pub fn balloon_stand() -> CardDefinition {
    attraction(
        "Balloon Stand",
        &[2, 6],
        Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(balloon_token()),
            },
            Effect::MaySacrifice {
                description: "Sacrifice a Balloon?".into(),
                filter: R::HasCreatureType(CreatureType::Balloon),
                count: Value::ONE,
                then: Box::new(Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                }),
                else_: None,
            },
        ]),
    )
}

/// Bounce Chamber — Visit: bounce the lowest-toughness creature you don't
/// control.
pub fn bounce_chamber() -> CardDefinition {
    attraction(
        "Bounce Chamber",
        &[2, 6],
        Effect::Move {
            what: Selector::MatchingAmong {
                inner: Box::new(Selector::LeastToughnessAmongAll),
                filter: R::Creature.and(R::ControlledByOpponent),
            },
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Bumper Cars — Visit: target creature must be blocked this turn if able.
pub fn bumper_cars() -> CardDefinition {
    attraction(
        "Bumper Cars",
        &[2, 3, 6],
        Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::MustBeBlocked,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Clown Extruder — Visit: create a 1/1 white Clown Robot artifact creature.
pub fn clown_extruder() -> CardDefinition {
    attraction(
        "Clown Extruder",
        &[2, 6],
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(clown_robot_token()),
        },
    )
}

/// Concession Stand — Visit: create a Food token.
pub fn concession_stand() -> CardDefinition {
    attraction(
        "Concession Stand",
        &[2, 6],
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(crabomination_base::tokens::food_token()),
        },
    )
}

/// Foam Weapons Kiosk — Visit: +1/+1 counter and vigilance on a creature you
/// control.
pub fn foam_weapons_kiosk() -> CardDefinition {
    attraction(
        "Foam Weapons Kiosk",
        &[2, 6],
        Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Fortune Teller — Visit: scry 1.
pub fn fortune_teller() -> CardDefinition {
    attraction(
        "Fortune Teller",
        &[2, 3, 6],
        Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
    )
}

/// Information Booth — Visit: draw a card.
pub fn information_booth() -> CardDefinition {
    attraction("Information Booth", &[2, 6], draw(1))
}

/// Kiddie Coaster — Visit: creatures you control get +1/+0.
pub fn kiddie_coaster() -> CardDefinition {
    attraction(
        "Kiddie Coaster",
        &[2, 3, 6],
        Effect::PumpPT {
            what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
            power: Value::ONE,
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Roller Coaster — Visit: creatures you control get +2/+0.
pub fn roller_coaster() -> CardDefinition {
    attraction(
        "Roller Coaster",
        &[2, 6],
        Effect::PumpPT {
            what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
            power: Value::Const(2),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Merry-Go-Round — Visit: your small creatures gain horsemanship.
pub fn merry_go_round() -> CardDefinition {
    attraction(
        "Merry-Go-Round",
        &[2, 6],
        Effect::GrantKeyword {
            what: Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::Creature.and(R::PowerAtMost(2)),
            },
            keyword: Keyword::Horsemanship,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Spinny Ride — Visit: tap target creature an opponent controls.
pub fn spinny_ride() -> CardDefinition {
    attraction(
        "Spinny Ride",
        &[2, 3, 6],
        Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
    )
}

/// Trash Bin — Visit: mill two, then a random card comes back from the
/// graveyard.
pub fn trash_bin() -> CardDefinition {
    attraction(
        "Trash Bin",
        &[2, 6],
        Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(2) },
            Effect::Move {
                what: Selector::RandomAmong(R::InYourGraveyard),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
    )
}

/// Swinging Ship — Visit: an extra combat phase, with the attackers untapped
/// on the way in.
pub fn swinging_ship() -> CardDefinition {
    attraction(
        "Swinging Ship",
        &[2, 6],
        Effect::Seq(vec![
            Effect::AdditionalCombatPhaseAfterMain { count: Value::ONE },
            Effect::AtEachCombatThisTurn {
                body: Box::new(Effect::Untap {
                    what: Selector::EachPermanent(R::Creature.and(R::AttackedThisTurn)),
                    up_to: None,
                }),
            },
        ]),
    )
}

// ── Cards that open Attractions (CR 701.51) ─────────────────────────────────

/// "Lifetime" Pass Holder — {B} 2/1 Zombie; enters tapped, opens an Attraction
/// on death, and climbs back out of the graveyard on a 6.
pub fn lifetime_pass_holder() -> CardDefinition {
    CardDefinition {
        name: "\"Lifetime\" Pass Holder",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![crate::card::StaticAbility {
            description: "This creature enters tapped.",
            effect: crate::effect::StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::OpenAnAttraction)],
        ..Default::default()
    }
}

fn etb_opener(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        keywords,
        triggered_abilities: vec![etb(Effect::OpenAnAttraction)],
        ..Default::default()
    }
}

/// Deadbeat Attendant — {1}{B} 1/1 Vampire. ETB: open an Attraction.
pub fn deadbeat_attendant() -> CardDefinition {
    etb_opener(
        "Deadbeat Attendant",
        cost(&[generic(1), b()]),
        vec![CreatureType::Vampire],
        1,
        1,
        vec![],
    )
}

/// Petting Zookeeper — {2}{G} 0/4 Elf with reach. ETB: open an Attraction.
pub fn petting_zookeeper() -> CardDefinition {
    etb_opener(
        "Petting Zookeeper",
        cost(&[generic(2), g()]),
        vec![CreatureType::Elf],
        0,
        4,
        vec![Keyword::Reach],
    )
}

/// Seasoned Buttoneer — {2}{U} 2/2 Vedalken. ETB: open an Attraction.
pub fn seasoned_buttoneer() -> CardDefinition {
    etb_opener(
        "Seasoned Buttoneer",
        cost(&[generic(2), u()]),
        vec![CreatureType::Vedalken],
        2,
        2,
        vec![],
    )
}

/// Rad Rascal — {3}{R} 3/3 Devil. ETB: open an Attraction.
pub fn rad_rascal() -> CardDefinition {
    etb_opener(
        "Rad Rascal",
        cost(&[generic(3), r()]),
        vec![CreatureType::Devil],
        3,
        3,
        vec![],
    )
}

/// Quick Fixer — {2}{B} 2/3 Azra with menace; opens an Attraction on combat
/// damage to a player.
pub fn quick_fixer() -> CardDefinition {
    CardDefinition {
        name: "Quick Fixer",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::OpenAnAttraction,
        }],
        ..Default::default()
    }
}

/// Coming Attraction — {2}{G} Sorcery. Fetch a basic land tapped, then open an
/// Attraction.
pub fn coming_attraction() -> CardDefinition {
    CardDefinition {
        name: "Coming Attraction",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::OpenAnAttraction,
        ]),
        ..Default::default()
    }
}

/// The Most Dangerous Gamer — {2}{B}{G} 2/2 legendary deathtouch; opens an
/// Attraction on entry and on attack, growing with each one opened.
pub fn the_most_dangerous_gamer() -> CardDefinition {
    CardDefinition {
        name: "The Most Dangerous Gamer",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::OpenAnAttraction,
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ])),
            crate::effect::shortcut::on_attack(Effect::Seq(vec![
                Effect::OpenAnAttraction,
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ])),
        ],
        ..Default::default()
    }
}

/// Complaints Clerk — {3}{W} 3/3 Sloth Beast. ETB: open an Attraction.
pub fn complaints_clerk() -> CardDefinition {
    etb_opener(
        "Complaints Clerk",
        cost(&[generic(3), w()]),
        vec![CreatureType::Sloth, CreatureType::Beast],
        3,
        3,
        vec![],
    )
}
