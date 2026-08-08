//! Oath of the Gatewatch (OGW) gap wave 3 — the Eldrazi/devoid shell, the
//! Oaths, and the remaining spells. Tests in `classic_sets/ogw`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, MayPlayDuration, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, Value,
};
use crate::effect::shortcut::{cast_colorless, deal, draw, etb, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Selector,
    StaticEffect, TriggeredAbility, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{b, colorless, cost, g, generic, r, u, w};
use crabomination_base::tokens::eldrazi_scion_token;

fn types(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// Abstruse Interference — {2}{U} Instant. Devoid. Counter target spell unless
/// its controller pays {1}; make a Scion either way.
pub fn abstruse_interference() -> CardDefinition {
    CardDefinition {
        name: "Abstruse Interference",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Seq(vec![
            Effect::CounterUnlessPaid {
                what: Selector::Target(0),
                mana_cost: cost(&[generic(1)]),
                exile: false,
                extra_generic: None,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: Box::new(eldrazi_scion_token()),
                count: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Prophet of Distortion — {U} 1/2 Eldrazi Drone. Devoid. {3}{C}: Draw a card.
pub fn prophet_of_distortion() -> CardDefinition {
    CardDefinition {
        name: "Prophet of Distortion",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Eldrazi, CreatureType::Drone]),
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Devoid],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), colorless(1)]),
            effect: draw(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Thought Harvester — {3}{U} 2/4 Eldrazi Drone. Devoid, flying; each colorless
/// spell you cast makes an opponent exile their top card.
pub fn thought_harvester() -> CardDefinition {
    CardDefinition {
        name: "Thought Harvester",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Eldrazi, CreatureType::Drone]),
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Devoid, Keyword::Flying],
        triggered_abilities: vec![cast_colorless(Effect::ExileTopOfLibrary {
            who: target_filtered(R::OpponentPlayer),
            amount: Value::Const(1),
            link_to_source: false,
            face_down: false,
        })],
        ..Default::default()
    }
}

/// Dread Defiler — {6}{B} 6/8 Eldrazi. Devoid. {3}{C}, exile a creature card
/// from your graveyard: target opponent loses that much life.
pub fn dread_defiler() -> CardDefinition {
    CardDefinition {
        name: "Dread Defiler",
        cost: cost(&[generic(6), b()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Eldrazi]),
        power: 6,
        toughness: 8,
        keywords: vec![Keyword::Devoid],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), colorless(1)]),
            exile_other_filter: Some((R::Creature.and(R::InYourGraveyard), 1)),
            effect: Effect::LoseLife {
                who: target_filtered(R::OpponentPlayer),
                amount: Value::SacrificedPower,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Eldrazi Obligator — {2}{R} 3/1 Eldrazi with haste. Devoid. On cast, may pay
/// {1}{C} to steal a creature for the turn.
pub fn eldrazi_obligator() -> CardDefinition {
    CardDefinition {
        name: "Eldrazi Obligator",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Eldrazi]),
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Devoid, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {1}{C} to gain control of target creature?".into(),
                mana_cost: cost(&[generic(1), colorless(1)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::GainControl {
                        what: target_filtered(R::Creature),
                        to: None,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Untap {
                        what: Selector::Target(0),
                        up_to: None,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Vile Redeemer — {2}{G} 3/3 Eldrazi with flash. Devoid. On cast, may pay {C}
/// to make a Scion per nontoken creature you lost this turn.
pub fn vile_redeemer() -> CardDefinition {
    CardDefinition {
        name: "Vile Redeemer",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Eldrazi]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Devoid, Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {C} for a Scion per creature you lost this turn?".into(),
                mana_cost: cost(&[colorless(1)]),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    definition: Box::new(eldrazi_scion_token()),
                    count: Value::CreaturesDiedThisTurn(PlayerRef::You),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Overwhelming Denial — {2}{U}{U} Instant. Surge {U}{U}. Can't be countered;
/// counter target spell.
pub fn overwhelming_denial() -> CardDefinition {
    CardDefinition {
        name: "Overwhelming Denial",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::CounterSpell {
            what: Selector::Target(0),
        },
        alternative_cost: Some(crate::effect::shortcut::surge(cost(&[u(), u()]), false)),
        ..Default::default()
    }
}

/// Grip of the Roil — {2}{U} Instant. Surge {1}{U}. Tap target creature, it
/// skips its next untap, and draw a card.
pub fn grip_of_the_roil() -> CardDefinition {
    CardDefinition {
        name: "Grip of the Roil",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(R::Creature),
            },
            Effect::SkipNextUntap {
                what: Selector::Target(0),
            },
            draw(1),
        ]),
        alternative_cost: Some(crate::effect::shortcut::surge(cost(&[generic(1), u()]), false)),
        ..Default::default()
    }
}

/// Roiling Waters — {5}{U}{U} Sorcery. Bounce up to two opposing creatures;
/// target player draws two.
pub fn roiling_waters() -> CardDefinition {
    CardDefinition {
        name: "Roiling Waters",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            },
            Effect::Draw {
                who: Selector::TargetFiltered {
                    slot: 2,
                    filter: R::Player,
                },
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Null Caller — {3}{B} 2/4 Vampire Shaman. {3}{B}, exile a creature card from
/// your graveyard: make a tapped 2/2 black Zombie.
pub fn null_caller() -> CardDefinition {
    CardDefinition {
        name: "Null Caller",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Vampire, CreatureType::Shaman]),
        power: 2,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            exile_other_filter: Some((R::Creature.and(R::InYourGraveyard), 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                definition: Box::new(crate::card::TokenDefinition {
                    name: "Zombie".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![crate::mana::Color::Black],
                    subtypes: types(vec![CreatureType::Zombie]),
                    tapped: true,
                    ..Default::default()
                }),
                count: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Baloth Null — {4}{B}{G} 4/5 Zombie Beast. ETB returns up to two creature
/// cards from your graveyard to hand.
pub fn baloth_null() -> CardDefinition {
    CardDefinition {
        name: "Baloth Null",
        cost: cost(&[generic(4), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Zombie, CreatureType::Beast]),
        power: 4,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::InYourGraveyard),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..Default::default()
    }
}

/// Seed Guardian — {2}{G}{G} 3/4 Elemental with reach. On death, make an X/X
/// Elemental where X is the creature cards in your graveyard.
pub fn seed_guardian() -> CardDefinition {
    let gy_creatures = || Value::CardsInGraveyardMatching {
        who: PlayerRef::You,
        filter: R::Creature,
    };
    let token = crate::card::TokenDefinition {
        name: "Elemental".into(),
        power: 0,
        toughness: 0,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Green],
        subtypes: types(vec![CreatureType::Elemental]),
        dynamic_pt: Some((gy_creatures(), gy_creatures())),
        ..Default::default()
    };
    CardDefinition {
        name: "Seed Guardian",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Elemental]),
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            definition: Box::new(token),
            count: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Harvester Troll — {3}{G} 2/3 Troll. ETB you may sacrifice a creature or
/// land for two +1/+1 counters.
pub fn harvester_troll() -> CardDefinition {
    CardDefinition {
        name: "Harvester Troll",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Troll]),
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Sacrifice a creature or land for two +1/+1 counters?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: R::Creature.or(R::Land),
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Zendikar Resurgent — {5}{G}{G} Enchantment. Lands you tap for mana produce
/// an extra mana of a type they made; creature spells you cast draw a card.
pub fn zendikar_resurgent() -> CardDefinition {
    CardDefinition {
        name: "Zendikar Resurgent",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::TappedForMana, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Land,
                    }),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyTypeTriggerSourceProduces,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    },
                ),
                effect: draw(1),
            },
        ],
        ..Default::default()
    }
}

/// Pyromancer's Assault — {3}{R} Enchantment. Your second spell each turn
/// deals 2 damage to any target.
pub fn pyromancers_assault() -> CardDefinition {
    CardDefinition {
        name: "Pyromancer's Assault",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![crate::effect::shortcut::flurry(deal(2, target_any()))],
        ..Default::default()
    }
}

/// Oath of Chandra — {1}{R} legendary Enchantment. ETB 3 damage to a creature
/// an opponent controls; each end step after a planeswalker of yours entered,
/// 2 damage to each opponent.
pub fn oath_of_chandra() -> CardDefinition {
    CardDefinition {
        name: "Oath of Chandra",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            etb(deal(
                3,
                target_filtered(R::Creature.and(R::ControlledByOpponent)),
            )),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::AnyPlayer,
                )
                .with_filter(Predicate::PlaneswalkerEnteredThisTurn { who: PlayerRef::You }),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
            },
        ],
        ..Default::default()
    }
}

/// Oath of Gideon — {2}{W} legendary Enchantment. ETB two 1/1 Kor Allies; your
/// planeswalkers enter with an extra loyalty counter.
pub fn oath_of_gideon() -> CardDefinition {
    CardDefinition {
        name: "Oath of Gideon",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: Box::new(crate::card::TokenDefinition {
                name: "Kor Ally".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![crate::mana::Color::White],
                subtypes: types(vec![CreatureType::Kor, CreatureType::Ally]),
                ..Default::default()
            }),
            count: Value::Const(2),
        })],
        static_abilities: vec![StaticAbility {
            description: "Each planeswalker you control enters with an additional loyalty counter.",
            effect: StaticEffect::PlaneswalkersEnterWithExtraLoyalty { amount: 1 },
        }],
        ..Default::default()
    }
}

/// Oath of Jace — {2}{U} legendary Enchantment. ETB draw three, discard two;
/// each upkeep scry per planeswalker you control.
pub fn oath_of_jace() -> CardDefinition {
    CardDefinition {
        name: "Oath of Jace",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                draw(3),
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(2),
                    random: false,
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            R::HasCardType(CardType::Planeswalker).and(R::ControlledByYou),
                        )),
                        filter: R::Any,
                    },
                },
            },
        ],
        ..Default::default()
    }
}

/// Bonds of Mortality — {1}{G} Enchantment. ETB draw a card; {G} strips
/// hexproof and indestructible from your opponents' creatures.
pub fn bonds_of_mortality() -> CardDefinition {
    CardDefinition {
        name: "Bonds of Mortality",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(draw(1))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::Seq(vec![
                Effect::LoseKeyword { duration: Duration::EndOfTurn,
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                    keyword: Keyword::Hexproof,
                },
                Effect::LoseKeyword { duration: Duration::EndOfTurn,
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                    keyword: Keyword::Indestructible,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mina and Denn, Wildborn — {2}{R}{G} 4/4 legendary Elf Ally. Extra land drop;
/// {R}{G} + bounce a land: target creature gains trample.
pub fn mina_and_denn_wildborn() -> CardDefinition {
    CardDefinition {
        name: "Mina and Denn, Wildborn",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Elf, CreatureType::Ally]),
        supertypes: vec![Supertype::Legendary],
        power: 4,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r(), g()]),
            bounce_other_filter: Some((R::Land.and(R::ControlledByYou), 1)),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gift of Tusks — {U} Instant. Target creature loses all abilities and becomes
/// a green 3/3 Elephant until end of turn.
pub fn gift_of_tusks() -> CardDefinition {
    CardDefinition {
        name: "Gift of Tusks",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LoseAllAbilities {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
            },
            Effect::SetBasePT {
                what: Selector::Target(0),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::BecomeCreatureType {
                what: Selector::Target(0),
                creature_types: vec![CreatureType::Elephant],
                duration: Duration::EndOfTurn,
            },
            Effect::BecomeColor {
                what: Selector::Target(0),
                colors: vec![crate::mana::Color::Green],
                duration: Duration::EndOfTurn,
                additive: false,
            },
        ]),
        ..Default::default()
    }
}

/// Consuming Sinkhole — {3}{R} Instant. Devoid. Exile a land creature, or deal
/// 4 damage to a player or planeswalker.
pub fn consuming_sinkhole() -> CardDefinition {
    CardDefinition {
        name: "Consuming Sinkhole",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::Land)),
                to: ZoneDest::Exile,
            },
            deal(
                4,
                target_filtered(R::Player.or(R::HasCardType(CardType::Planeswalker))),
            ),
        ]),
        ..Default::default()
    }
}

/// Ruin in Their Wake — {1}{G} Sorcery. Devoid. Fetch a basic land — onto the
/// battlefield tapped if you control a Wastes, otherwise to hand.
pub fn ruin_in_their_wake() -> CardDefinition {
    CardDefinition {
        name: "Ruin in Their Wake",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        effect: Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(R::Land.and(R::ControlledByYou).and(R::HasName("Wastes".into()))),
                n: Value::Const(1),
            },
            then: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            }),
            else_: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
        ..Default::default()
    }
}

/// Press into Service — {4}{R} Sorcery. Support 2, then steal a creature for
/// the turn.
pub fn press_into_service() -> CardDefinition {
    CardDefinition {
        name: "Press into Service",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SupportCounters {
                max_targets: 2,
                filter: R::Creature,
            },
            Effect::GainControl {
                what: Selector::TargetFiltered {
                    slot: 2,
                    filter: R::Creature,
                },
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(2),
                up_to: None,
            },
            Effect::GrantKeyword {
                what: Selector::Target(2),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Goblin Dark-Dwellers — {3}{R}{R} 4/4 Goblin with menace. ETB free-casts a
/// cheap instant or sorcery from your graveyard, exiling it after.
pub fn goblin_dark_dwellers() -> CardDefinition {
    CardDefinition {
        name: "Goblin Dark-Dwellers",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Goblin]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::GrantMayPlay {
            what: target_filtered(
                R::InYourGraveyard
                    .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))
                    .and(R::ManaValueAtMost(3)),
            ),
            duration: MayPlayDuration::EndOfThisTurn,
            exile_after: true,
            to_owner: false,
            pay_own_cost: false,
            any_color: false,
        })],
        ..Default::default()
    }
}

/// The Embodiment cycle — a land-animating Elemental that shares its keyword
/// with your land creatures and animates a land on each landfall.
fn embodiment(
    name: &'static str,
    c: crate::mana::ManaCost,
    p: i32,
    t: i32,
    keyword: Keyword,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Elemental]),
        power: p,
        toughness: t,
        keywords: vec![keyword.clone()],
        static_abilities: vec![StaticAbility {
            description: "Land creatures you control have this creature's keyword.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::Land).and(R::ControlledByYou),
                power: 0,
                toughness: 0,
                keywords: vec![keyword],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Animate a land you control as a 3/3 Elemental?".into(),
                body: Box::new(crate::effect::shortcut::animate_land(0, 3)),
            },
        }],
        ..Default::default()
    }
}

pub fn embodiment_of_fury() -> CardDefinition {
    embodiment(
        "Embodiment of Fury",
        cost(&[generic(3), r()]),
        4,
        3,
        Keyword::Trample,
    )
}

pub fn embodiment_of_insight() -> CardDefinition {
    embodiment(
        "Embodiment of Insight",
        cost(&[generic(4), g()]),
        4,
        4,
        Keyword::Vigilance,
    )
}
