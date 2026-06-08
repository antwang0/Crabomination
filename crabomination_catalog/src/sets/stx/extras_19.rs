//! Remaining real STX (Strixhaven 2021) printed cards — final sweep. These
//! ride existing primitives plus the new `SelectionRequirement::EnteredThisTurn`
//! and `Duration::Permanent` land-animation. Each ships with a test in
//! `crate::tests::stx`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate, Selector, SelectionRequirement,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    WardCost, Zone,
};
use crate::effect::shortcut::{dies_gain_life, draw, etb, magecraft, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// Emergent Sequence — {1}{G} Sorcery. Search your library for a basic land,
/// put it onto the battlefield tapped, then shuffle. That land becomes a 0/0
/// green-and-blue Fractal creature that's still a land. Put a +1/+1 counter on
/// it for each land you had enter under your control this turn.
pub fn emergent_sequence() -> CardDefinition {
    CardDefinition {
        name: "Emergent Sequence",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::BecomeCreature {
                what: Selector::LastMoved,
                power: Value::Const(0),
                toughness: Value::Const(0),
                creature_types: vec![CreatureType::Fractal],
                keywords: vec![],
                duration: Duration::Permanent,
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Land
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::EnteredThisTurn),
                )),
            },
        ]),
        ..Default::default()
    }
}

/// Augmenter Pugilist — {1}{G}{G} 3/3 Troll Druid with trample. While you
/// control eight or more lands it gets +5/+5. (Front face of the Augmenter
/// Pugilist // Echoing Equation MDFC; the back-face copy effect is a
/// continuous layer-1 rewrite not yet modeled, so only the creature ships.)
pub fn augmenter_pugilist() -> CardDefinition {
    CardDefinition {
        name: "Augmenter Pugilist",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "As long as you control eight or more lands, this gets +5/+5.",
            effect: StaticEffect::PumpSelfIf {
                condition: crate::card::Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(8),
                },
                power: 5,
                toughness: 5,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Flamethrower Sonata — {1}{R} Sorcery, back face of Torrent Sculptor.
/// Discard a card, then draw a card. When you discard an instant or sorcery
/// card this way, deal damage equal to its mana value to target creature or
/// planeswalker you don't control. (A target is chosen on cast; the damage is
/// skipped when the discard isn't an instant/sorcery.)
fn flamethrower_sonata() -> CardDefinition {
    let is = SelectionRequirement::HasCardType(CardType::Instant)
        .or(SelectionRequirement::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Flamethrower Sonata",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            draw(1),
            Effect::If {
                cond: Predicate::SelectorExists(Selector::DiscardedThisResolution {
                    filter: is.clone(),
                }),
                then: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::ManaValueOf(Box::new(Selector::DiscardedThisResolution {
                        filter: is,
                    })),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Torrent Sculptor // Flamethrower Sonata — {2}{U}{U} 2/2 Merfolk Wizard with
/// Ward {2}. ETB: exile an instant or sorcery card from your graveyard and put
/// +1/+1 counters on this equal to half that card's mana value, rounded up.
pub fn torrent_sculptor() -> CardDefinition {
    let is = SelectionRequirement::HasCardType(CardType::Instant)
        .or(SelectionRequirement::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Torrent Sculptor",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: is,
                    }),
                    count: Box::new(Value::Const(1)),
                },
                to: ZoneDest::Exile,
            },
            // ceil(mv/2) = floor((mv+1)/2).
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::HalfDown(Box::new(Value::Sum(vec![
                    Value::ManaValueOf(Box::new(Selector::LastMoved)),
                    Value::Const(1),
                ]))),
            },
        ]))],
        back_face: Some(Box::new(flamethrower_sonata())),
        ..Default::default()
    }
}

/// 1/1 black-and-green Pest token with "When this token dies, you gain 1 life."
/// (Valentin's reflexive; distinct from SOS's attack-trigger Pest.)
fn valentin_pest_token() -> TokenDefinition {
    TokenDefinition {
        name: "Pest".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pest], ..Default::default() },
        triggered_abilities: vec![dies_gain_life(1)],
        ..Default::default()
    }
}

/// Lisette, Dean of the Root — {2}{G}{G} 4/4 Human Druid (back of Valentin).
/// Whenever you gain life, you may pay {1}; if you do, put a +1/+1 counter on
/// each creature you control and those creatures gain trample until end of turn.
fn lisette_dean_of_the_root() -> CardDefinition {
    let yours = || Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "Lisette, Dean of the Root",
        cost: cost(&[generic(2), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1}: +1/+1 counter on each creature you control + trample EOT."
                    .into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: yours(),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    Effect::GrantKeyword {
                        what: yours(),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Valentin, Dean of the Vein // Lisette, Dean of the Root — {B} 1/1 Legendary
/// Vampire Warlock with menace + lifelink. If a nontoken creature an opponent
/// controls would die, exile it instead; when you do, you may pay {2} to make a
/// 1/1 BG Pest with "when this dies, you gain 1 life."
pub fn valentin_dean_of_the_vein() -> CardDefinition {
    CardDefinition {
        name: "Valentin, Dean of the Vein",
        cost: cost(&[b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "If a nontoken creature an opponent controls would die, exile it instead. \
                When you do, you may pay {2}: create a 1/1 BG Pest.",
            effect: StaticEffect::ExileDyingOpponentCreatures {
                when_you_do: Some(Box::new(Effect::MayPay {
                    description: "Pay {2} to create a 1/1 BG Pest.".into(),
                    mana_cost: cost(&[generic(2)]),
                    body: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                        definition: valentin_pest_token(),
                    }),
                })),
            },
        }],
        back_face: Some(Box::new(lisette_dean_of_the_root())),
        ..Default::default()
    }
}

/// Lukka, Wayward Bonder — {4}{R}{R} Lukka planeswalker (back of Mila), 5
/// loyalty. +1: you may discard a card; if you do, draw a card (two if a
/// creature card was discarded). −2: reanimate a creature card with haste,
/// exiled at your next upkeep. −7: an emblem firing power-damage on each
/// creature you control entering.
fn lukka_wayward_bonder() -> CardDefinition {
    CardDefinition {
        name: "Lukka, Wayward Bonder",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Lukka],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::MayDo {
                    description: "Discard a card; draw one (two if it was a creature).".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                        Effect::If {
                            cond: Predicate::ValueAtLeast(
                                Value::CreatureCardsDiscardedThisEffect,
                                Value::Const(1),
                            ),
                            then: Box::new(draw(2)),
                            else_: Box::new(draw(1)),
                        },
                    ])),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(SelectionRequirement::Creature),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::DelayUntil {
                        kind: DelayedTriggerKind::YourNextUpkeep,
                        body: Box::new(Effect::Exile { what: Selector::Target(0) }),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Lukka, Wayward Bonder".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                            .with_filter(Predicate::EntityMatches {
                                what: Selector::TriggerSource,
                                filter: SelectionRequirement::Creature,
                            }),
                        effect: Effect::DealDamage {
                            to: target_filtered(
                                SelectionRequirement::Creature
                                    .or(SelectionRequirement::Player)
                                    .or(SelectionRequirement::Planeswalker),
                            ),
                            amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                        },
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Mila, Crafty Companion // Lukka, Wayward Bonder — {1}{W}{W} 2/3 Legendary
/// Fox. When an opponent attacks a planeswalker you control, add a loyalty
/// counter to each planeswalker you control; when a permanent you control
/// becomes the target of an opponent's spell or ability, you may draw a card.
pub fn mila_crafty_companion() -> CardDefinition {
    CardDefinition {
        name: "Mila, Crafty Companion",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fox],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![
            // Over-fires slightly: ControllerAttackedByOpponent also covers
            // attacks on the player, not just their planeswalkers.
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Planeswalker
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::Loyalty,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::YourPermanentTargetedByOpponent),
                effect: Effect::MayDo {
                    description: "Draw a card.".into(),
                    body: Box::new(draw(1)),
                },
            },
        ],
        back_face: Some(Box::new(lukka_wayward_bonder())),
        ..Default::default()
    }
}

/// IS-cost-reduction static shared by the Rowan // Will scholars.
fn is_costs_one_less() -> StaticAbility {
    StaticAbility {
        description: "Instant and sorcery spells you cast cost {1} less to cast.",
        effect: StaticEffect::CostReduction {
            filter: SelectionRequirement::HasCardType(CardType::Instant)
                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            amount: 1,
        },
    }
}

/// Will, Scholar of Frost — {4}{U} Will planeswalker (back of Rowan), 4 loyalty.
/// IS spells you cast cost {1} less. +1: target creature has base 0/2 until your
/// next turn. −3: draw two. −7: exile up to five target permanents. (The −7's
/// "controller makes a 4/4" compensation collapses to a single targeted exile.)
fn will_scholar_of_frost() -> CardDefinition {
    CardDefinition {
        name: "Will, Scholar of Frost",
        cost: cost(&[generic(4), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Will],
            ..Default::default()
        },
        static_abilities: vec![is_costs_one_less()],
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::SetBasePT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(0),
                    toughness: Value::Const(2),
                    duration: Duration::UntilNextTurn,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: draw(2),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Exile { what: target_filtered(SelectionRequirement::Permanent) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rowan, Scholar of Sparks // Will, Scholar of Frost — {2}{R} Rowan
/// planeswalker, 2 loyalty. IS spells you cast cost {1} less. +1: deal 1 to each
/// opponent (3 instead if you've drawn three or more cards this turn). −4: an
/// emblem with "Whenever you cast an instant or sorcery spell, you may pay {2};
/// if you do, copy it and may choose new targets."
pub fn rowan_scholar_of_sparks() -> CardDefinition {
    CardDefinition {
        name: "Rowan, Scholar of Sparks",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Rowan],
            ..Default::default()
        },
        static_abilities: vec![is_costs_one_less()],
        base_loyalty: 2,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CardsDrawnThisTurn(PlayerRef::You),
                        Value::Const(3),
                    ),
                    then: Box::new(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(3),
                    }),
                    else_: Box::new(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(1),
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -4,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Rowan, Scholar of Sparks".into(),
                    triggered: vec![magecraft(Effect::MayPay {
                        description: "Pay {2} to copy that spell (you may choose new targets).".into(),
                        mana_cost: cost(&[generic(2)]),
                        body: Box::new(Effect::CopySpellMayChooseTargets {
                            what: Selector::TriggerSource,
                            count: Value::Const(1),
                        }),
                    })],
                },
                ..Default::default()
            },
        ],
        back_face: Some(Box::new(will_scholar_of_frost())),
        ..Default::default()
    }
}

/// Awaken the Blood Avatar — {6}{B}{R} Sorcery, back face of Extus. Each
/// opponent sacrifices a creature of their choice; create a 3/6 black-and-red
/// Avatar with haste and an attack trigger that deals 3 to each opponent.
/// (The optional "sacrifice any number of creatures, {2} less each" cast cost
/// is dropped — the engine has no variable-sacrifice cost reduction yet.)
fn awaken_the_blood_avatar() -> CardDefinition {
    let avatar = TokenDefinition {
        name: "Avatar".into(),
        power: 3,
        toughness: 6,
        keywords: vec![Keyword::Haste],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Avatar], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Awaken the Blood Avatar",
        cost: cost(&[generic(6), b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: avatar },
        ]),
        ..Default::default()
    }
}

/// Extus, Oriq Overlord // Awaken the Blood Avatar — {1}{W}{B}{B} 2/4 Legendary
/// Human Warlock with double strike. Magecraft: return target nonlegendary
/// creature card from your graveyard to your hand.
pub fn extus_oriq_overlord() -> CardDefinition {
    CardDefinition {
        name: "Extus, Oriq Overlord",
        cost: cost(&[generic(1), w(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![magecraft(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::Not(Box::new(
                    SelectionRequirement::HasSupertype(Supertype::Legendary),
                ))),
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        back_face: Some(Box::new(awaken_the_blood_avatar())),
        ..Default::default()
    }
}

/// Search for Blex — {2}{B}{B} Sorcery, back face of Blex. Look at the top five
/// cards of your library; put any number into your hand and the rest into your
/// graveyard. Lose 3 life for each card put into your hand this way.
fn search_for_blex() -> CardDefinition {
    CardDefinition {
        name: "Search for Blex",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DigToHandLoseLife {
            count: Value::Const(5),
            life_per_card: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Blex, Vexing Pest // Search for Blex — {2}{G} 3/2 Legendary Pest. Anthem for
/// your Pests/Bats/Insects/Snakes/Spiders; dies → gain 4 life.
pub fn blex_vexing_pest() -> CardDefinition {
    let kin = SelectionRequirement::HasCreatureType(CreatureType::Pest)
        .or(SelectionRequirement::HasCreatureType(CreatureType::Bat))
        .or(SelectionRequirement::HasCreatureType(CreatureType::Insect))
        .or(SelectionRequirement::HasCreatureType(CreatureType::Snake))
        .or(SelectionRequirement::HasCreatureType(CreatureType::Spider));
    CardDefinition {
        name: "Blex, Vexing Pest",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Other Pests, Bats, Insects, Snakes, and Spiders you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    kin.and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![dies_gain_life(4)],
        back_face: Some(Box::new(search_for_blex())),
        ..Default::default()
    }
}
