//! Gatecrash (GTC) wave 11: a filtered fog, two combat-damage graveyard/library
//! payoffs, a symmetric upkeep punisher, a dies-into-recursion Angel, and a
//! blink-plus-Cipher trick. Tests in `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Subtypes, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{on_dies, target_filtered};
use crate::effect::{PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, u, w};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// Hindervines — {2}{G} Instant. Prevent all combat damage this turn dealt by
/// creatures with no +1/+1 counters on them.
pub fn hindervines() -> CardDefinition {
    CardDefinition {
        name: "Hindervines",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PreventCombatDamageExceptDealtBy {
            except: R::WithCounter(CounterType::PlusOnePlusOne),
        },
        ..Default::default()
    }
}

/// Lord of the Void — {4}{B}{B}{B} 7/7 Demon. Flying; combat damage to a player
/// exiles the top seven cards of that player's library, then puts a creature
/// card from among them onto the battlefield under your control.
pub fn lord_of_the_void() -> CardDefinition {
    CardDefinition {
        name: "Lord of the Void",
        cost: cost(&[generic(4), b(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Demon]),
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(7),
                    link_to_source: false,
                    face_down: false,
                },
                Effect::Move {
                    what: Selector::one_of(Selector::ExiledThisResolution {
                        filter: R::Creature,
                    }),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Duskmantle Seer — {2}{U}{B} 4/4 Vampire Wizard. Flying; at your upkeep, each
/// player reveals their top card, loses life equal to its mana value, then puts
/// it into their hand.
pub fn duskmantle_seer() -> CardDefinition {
    CardDefinition {
        name: "Duskmantle Seer",
        cost: cost(&[generic(2), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Vampire, CreatureType::Wizard]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::Seq(vec![
                    // Read the top card's MV before it moves to hand.
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::ManaValueOf(Box::new(Selector::TopOfLibrary {
                            who: PlayerRef::Triggerer,
                            count: Value::ONE,
                        })),
                    },
                    Effect::Move {
                        what: Selector::TopOfLibrary {
                            who: PlayerRef::Triggerer,
                            count: Value::ONE,
                        },
                        to: ZoneDest::Hand(PlayerRef::Triggerer),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Deathpact Angel — {3}{W}{B}{B} 5/5 Angel. Flying; when it dies, create a 1/1
/// white and black Cleric that can sacrifice itself to return a Deathpact Angel
/// from your graveyard.
pub fn deathpact_angel() -> CardDefinition {
    let cleric = TokenDefinition {
        name: "Cleric".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Black],
        subtypes: creatures(vec![CreatureType::Cleric]),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w(), b(), b()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::HasName("Deathpact Angel".into()),
                }),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Deathpact Angel",
        cost: cost(&[generic(3), w(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Angel]),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(cleric),
        })],
        ..Default::default()
    }
}

/// Voidwalk — {3}{U} Sorcery. Exile target creature; return it at the next end
/// step under its owner's control. Cipher.
pub fn voidwalk() -> CardDefinition {
    CardDefinition {
        name: "Voidwalk",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileReturnToOwnerNextEndStep {
                what: target_filtered(R::Creature),
                tapped: false,
            },
            Effect::Cipher,
        ]),
        ..Default::default()
    }
}
