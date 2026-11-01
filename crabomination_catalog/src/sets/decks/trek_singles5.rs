//! Star Trek (TRK/TRC) — the fifth batch of EDHREC Star Trek closers: the two
//! missing cards of five single-commander average decks (Vash, Sylvia Tilly,
//! Lt. Arex, Fleet Captain Pike, Sam Kirk; scan of 2026-11-01). Tests in
//! `tests/recent_b/trek_singles5.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EventKind, EventScope, EventSpec,
    Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, investigate, target_filtered};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, b, cost, g, generic, u, x};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

/// "The number of differently named lands you control" (strange new worlds).
fn land_names() -> Value {
    Value::DistinctNamesControlledMatching(R::Land)
}

/// Sands of Delirium — {3} artifact. {X}, {T}: target player mills X.
pub fn sands_of_delirium() -> CardDefinition {
    CardDefinition {
        name: "Sands of Delirium",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            effect: Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::XFromCost },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fleet Captain Christopher Pike — {2}{U} legendary 0/6 Human Officer.
/// {1},{T}: untap two other target creatures; {2},{T}: tap two others.
pub fn fleet_captain_christopher_pike() -> CardDefinition {
    let two = |mana: u32, body: Effect| ActivatedAbility {
        mana_cost: cost(&[generic(mana)]),
        tap_cost: true,
        effect: Effect::ApplyToTargets {
            min_targets: 2,
            max_targets: 2,
            filter: R::Creature.and(R::OtherThanSource),
            effect: Box::new(body),
        },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            two(1, Effect::Untap { what: Selector::Target(0), up_to: None }),
            two(2, Effect::Tap { what: Selector::Target(0) }),
        ],
        ..legend(
            "Fleet Captain Christopher Pike",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Officer],
            0,
            6,
        )
    }
}

/// Sam Kirk, Xenoanthropologist — {3}{U} legendary 3/3 Human Scientist. An
/// opponent casting a creature spell makes you investigate.
pub fn sam_kirk_xenoanthropologist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: investigate(1),
        }],
        ..legend(
            "Sam Kirk, Xenoanthropologist",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            3,
            3,
        )
    }
}

/// Out of the Way — {3}{U} instant, {2} less if it targets a green permanent.
/// Return target nonland permanent an opponent controls to its owner's hand;
/// draw a card.
pub fn out_of_the_way() -> CardDefinition {
    CardDefinition {
        name: "Out of the Way",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((R::HasColor(crate::mana::Color::Green), 2)),
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Lt. Arex, Ever-Adapting — {5}{G}{U} legendary */* Edosian Officer with
/// trample; P/T = differently named lands you control. Entering draws three,
/// then you may put up to three lands from hand tapped.
pub fn lt_arex_ever_adapting() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        dynamic_pt: Some(DynamicPt::DistinctLandNamesControlled),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Land,
                count: Value::Const(3),
                tapped: true,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
        ]))],
        ..legend(
            "Lt. Arex, Ever-Adapting",
            cost(&[generic(5), g(), u()]),
            vec![CreatureType::Edosian, CreatureType::Officer],
            0,
            0,
        )
    }
}

/// Warp Speed — {2}{G} enchantment. Lands you control enter untapped; entering
/// tutors a basic land or Planet card to hand; {7}{U}{U}, sacrifice it: draw
/// two, {X} less where X is the differently named lands you control.
pub fn warp_speed() -> CardDefinition {
    CardDefinition {
        name: "Warp Speed",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Lands you control enter untapped.",
            effect: StaticEffect::LandsEnterUntapped,
        }],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::Land.and(R::HasSupertype(Supertype::Basic)).or(R::HasLandType(LandType::Planet)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7), u(), u()]),
            sac_cost: true,
            cost_reduction_value: Some(land_names()),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Storm the Festival — {3}{G}{G}{G} sorcery, flashback {7}{G}{G}{G}. Look at
/// the top five; put up to two permanent cards with mana value 5 or less onto
/// the battlefield; the rest go to the bottom in a random order.
pub fn storm_the_festival() -> CardDefinition {
    CardDefinition {
        name: "Storm the Festival",
        cost: cost(&[generic(3), g(), g(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(7), g(), g(), g()]))],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(5),
            pick_filter: Some(R::Permanent.and(R::ManaValueAtMost(5))),
            take: Some(Value::Const(2)),
            to_battlefield: true,
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Vash, Freelance Archaeologist — {1}{B}{G} legendary 2/4 Human Rogue
/// Scientist. Upkeep: surveil 1. A card leaving your graveyard draws a card,
/// once each turn.
pub fn vash_freelance_archaeologist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl).once_per_turn(),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..legend(
            "Vash, Freelance Archaeologist",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Scientist],
            2,
            4,
        )
    }
}

/// Blastoneuron — {3}{B} 2/2 flying Jellyfish Leech. Entering exiles target
/// creature card with mana value 3 or less from your graveyard for a hasty
/// token copy sacrificed at the next end step; one or more cards leaving your
/// graveyard drains each opponent for 1.
pub fn blastoneuron() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(
                        R::Creature.and(R::InGraveyard).and(R::OwnedByYou).and(R::ManaValueAtMost(3)),
                    ),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                    no_mana_cost: false,
                    enters_with_counters: None,
                    remove_keywords: vec![],
                },
                Effect::Exile { what: Selector::Target(0) },
                Effect::GrantKeyword { what: Selector::LastCreatedTokens, keyword: Keyword::Haste, duration: Duration::Permanent },
                Effect::SacrificeAtNextEndStep { what: Selector::LastCreatedTokens },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl).once_per_batch(),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::ONE,
                },
            },
        ],
        ..creature("Blastoneuron", cost(&[generic(3), b()]), vec![CreatureType::Jellyfish, CreatureType::Leech], 2, 2)
    }
}

/// Sylvia Tilly, Joy of Discovery — {2}{G} legendary 2/2 Human Scientist
/// Artificer. Entering surveils 2. Beginning of combat on your turn: you may
/// pay {1} and exile a card from your graveyard; when you do, a +1/+1 counter
/// on target creature (CR 603.12).
pub fn sylvia_tilly_joy_of_discovery() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                // Both halves are the cost: with an empty graveyard nothing is
                // offered.
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(Value::GraveyardSizeOf(PlayerRef::You), Value::ONE),
                    then: Box::new(Effect::MayPay {
                        description: "Pay {1} and exile a card from your graveyard?".into(),
                        mana_cost: cost(&[generic(1)]),
                        body: Box::new(Effect::Seq(vec![
                            Effect::ExileFromGraveyard { who: PlayerRef::You, count: Value::ONE, filter: R::Any },
                            Effect::ReflexiveTrigger {
                                body: Box::new(Effect::AddCounter {
                                    what: target_filtered(R::Creature),
                                    kind: CounterType::PlusOnePlusOne,
                                    amount: Value::ONE,
                                }),
                            },
                        ])),
                        else_: None,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..legend(
            "Sylvia Tilly, Joy of Discovery",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Artificer],
            2,
            2,
        )
    }
}
