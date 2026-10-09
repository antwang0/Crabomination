//! Commander: the cards the **Abzan Armor** precon (TDC, Felothar the
//! Steadfast) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_felothar.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, on_becomes_monstrous, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, w, Color, ManaCost};

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn yours() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn defender() -> R {
    R::HasKeyword(Keyword::Defender)
}

fn by_toughness(applies_to: Selector) -> StaticEffect {
    StaticEffect::GrantKeyword { applies_to, keyword: Keyword::AssignsCombatDamageByToughness }
}

fn team_pump(p: i32, t: i32) -> Effect {
    Effect::PumpPT {
        what: Selector::EachPermanent(yours()),
        power: Value::Const(p),
        toughness: Value::Const(t),
        duration: Duration::EndOfTurn,
    }
}

/// Felothar the Steadfast — your creatures deal combat damage by toughness
/// and can attack despite defender; {3}, {T}, sacrifice another creature:
/// draw its toughness, discard its power.
pub fn felothar_the_steadfast() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Each creature you control assigns combat damage equal to its toughness rather than its power.",
                effect: by_toughness(Selector::EachPermanent(yours())),
            },
            StaticAbility {
                description: "Creatures you control can attack as though they didn't have defender.",
                effect: StaticEffect::YourCreaturesCanAttackAsThoughNoDefender,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::SacrificedToughness },
                Effect::Discard { who: Selector::You, amount: Value::SacrificedPower, random: false },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Felothar the Steadfast",
            cost(&[generic(1), w(), b(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            0,
            5,
        )
    }
}

/// Arbor Adherent — any color; or X of one color, X the greatest toughness
/// among other creatures you control (`GreatestToughnessYouControl` skips the
/// source).
pub fn arbor_adherent() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ToughnessOf(Box::new(Selector::GreatestToughnessYouControl))),
                },
                ..Default::default()
            },
        ],
        ..creature("Arbor Adherent", cost(&[generic(3), g()]), vec![CreatureType::Dog, CreatureType::Druid], 2, 4)
    }
}

/// Assault Formation — your creatures deal combat damage by toughness; {G}:
/// a defender may attack; {2}{G}: +0/+1 to your team.
pub fn assault_formation() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature you control assigns combat damage equal to its toughness rather than its power.",
            effect: by_toughness(Selector::EachPermanent(yours())),
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(defender())),
                    keyword: Keyword::AttacksAsThoughNoDefender,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility { mana_cost: cost(&[generic(2), g()]), effect: team_pump(0, 1), ..Default::default() },
        ],
        ..enchantment("Assault Formation", cost(&[generic(1), g()]))
    }
}

/// Baldin, Century Herdmaster — on your turn every creature deals combat
/// damage by toughness; attacking gives up to one hundred target creatures
/// +0/+X for your hand size (the bot takes its own board, Baldin included).
pub fn baldin_century_herdmaster() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, each creature assigns combat damage equal to its toughness rather than its power.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::IsTurnOf(PlayerRef::You),
                inner: Box::new(by_toughness(Selector::EachPermanent(R::Creature))),
            },
        }],
        triggered_abilities: vec![on_attack(Effect::ApplyToTargets {
            max_targets: 100,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::ZERO,
                toughness: Value::HandSizeOf(PlayerRef::You),
                duration: Duration::EndOfTurn,
            }),
        })],
        ..legend(
            "Baldin, Century Herdmaster",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            0,
            7,
        )
    }
}

/// Behind the Scenes — your creatures have skulk; {4}{W}: +1/+1.
pub fn behind_the_scenes() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have skulk.",
            effect: StaticEffect::GrantKeyword { applies_to: Selector::EachPermanent(yours()), keyword: Keyword::Skulk },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: team_pump(1, 1),
            ..Default::default()
        }],
        ..enchantment("Behind the Scenes", cost(&[generic(2), b()]))
    }
}

/// Betor, Ancestor's Voice — flying, lifelink; your end step grows up to one
/// other target creature of yours by the life you gained and reanimates up to
/// one target creature card no bigger than the life you lost — one trigger,
/// two optional slots (an empty slot 0 holds its place, `game/target_hole.rs`).
pub fn betor_ancestors_voice() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(yours().and(R::OtherThanSource)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::LifeGainedThisTurn(PlayerRef::You),
                    },
                    Effect::Move {
                        what: Selector::TargetFiltered {
                            slot: 1,
                            filter: R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMostLifeLostThisTurn),
                        },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                ])),
            },
        }],
        ..legend(
            "Betor, Ancestor's Voice",
            cost(&[generic(2), w(), b(), g()]),
            vec![CreatureType::Spirit, CreatureType::Dragon],
            3,
            5,
        )
    }
}

/// Blight Pile — defender; {2}{B}, {T}: each opponent loses a life per
/// defender you control.
pub fn blight_pile() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::count(Selector::EachPermanent(yours().and(defender()))),
            },
            ..Default::default()
        }],
        ..creature("Blight Pile", cost(&[generic(1), b()]), vec![CreatureType::Phyrexian], 3, 3)
    }
}

/// Canopy Gargantuan — flying, ward {2}; your upkeep gives each other
/// creature of yours counters equal to its toughness.
pub fn canopy_gargantuan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(yours().and(R::OtherThanSource)),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                }),
            },
        }],
        ..creature("Canopy Gargantuan", cost(&[generic(5), g(), g()]), vec![CreatureType::Dragon], 7, 7)
    }
}

/// Colfenor's Urn — keeps your dead toughness-4+ creatures; three of them
/// come back at an end step.
pub fn colfenors_urn() -> CardDefinition {
    CardDefinition {
        name: "Colfenor's Urn",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::ToughnessAtLeast(4) },
                ),
                effect: Effect::MayDo {
                    description: "Exile it with Colfenor's Urn?".into(),
                    body: Box::new(Effect::ExileWithSource { what: Selector::TriggerSource }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                    Predicate::SelectorCountAtLeast { sel: Selector::CardExiledWithSource, n: Value::Const(3) },
                ),
                effect: Effect::Seq(vec![
                    Effect::SacrificeSource,
                    // "If you do": a second trigger finds it already gone.
                    Effect::If {
                        cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                        then: Box::new(Effect::Seq(vec![
                            Effect::Move {
                                what: Selector::CardExiledWithSource,
                                to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
                            },
                        ])),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Dragonlord Dromoka — can't be countered; flying, lifelink; opponents
/// can't cast spells during your turn.
pub fn dragonlord_dromoka() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered, Keyword::Flying, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't cast spells during your turn.",
            effect: StaticEffect::OpponentsCantCastDuringYourTurn,
        }],
        ..legend(
            "Dragonlord Dromoka",
            cost(&[generic(4), g(), w()]),
            vec![CreatureType::Elder, CreatureType::Dragon],
            5,
            7,
        )
    }
}

/// Indomitable Ancients — a 2/10 Treefolk Warrior.
pub fn indomitable_ancients() -> CardDefinition {
    creature(
        "Indomitable Ancients",
        cost(&[generic(2), w(), w()]),
        vec![CreatureType::Treefolk, CreatureType::Warrior],
        2,
        10,
    )
}

/// Indulging Patrician — flying, lifelink; after 3 life gained in your turn,
/// each opponent loses 3 at your end step.
pub fn indulging_patrician() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer).with_filter(
                Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(3) },
            ),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) },
        }],
        ..creature(
            "Indulging Patrician",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Vampire, CreatureType::Noble],
            1,
            4,
        )
    }
}

/// Jaws of Defeat — a creature of yours entering drains an opponent by the
/// gap between its power and toughness.
pub fn jaws_of_defeat() -> CardDefinition {
    let p = || Value::PowerOf(Box::new(Selector::TriggerSource));
    let t = || Value::ToughnessOf(Box::new(Selector::TriggerSource));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::LoseLife {
                who: target_filtered(R::OpponentPlayer),
                amount: Value::Max(Box::new(Value::Diff(Box::new(p()), Box::new(t()))), Box::new(Value::Diff(Box::new(t()), Box::new(p())))),
            },
        }],
        ..enchantment("Jaws of Defeat", cost(&[generic(3), b()]))
    }
}

/// Protector of the Wastes — flying; entering or becoming monstrous exiles
/// up to two artifacts and/or enchantments of different players;
/// monstrosity 3.
pub fn protector_of_the_wastes() -> CardDefinition {
    let exile = || Effect::ForEachOpponentTarget {
        body: Box::new(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Artifact.or(R::Enchantment).and(R::ControlledByOpponent),
            effect: Box::new(Effect::Exile { what: Selector::Target(0) }),
        }),
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(exile()), on_becomes_monstrous(exile())],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: Effect::Monstrosity { n: Value::Const(3) },
            ..Default::default()
        }],
        ..creature("Protector of the Wastes", cost(&[generic(4), w(), w()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Radiant Grove — Forest Plains; enters tapped.
pub fn radiant_grove() -> CardDefinition {
    CardDefinition {
        name: "Radiant Grove",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Forest, LandType::Plains], ..Default::default() },
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![crate::sets::tap_add(Color::Green), crate::sets::tap_add(Color::White)],
        ..Default::default()
    }
}

/// Rampart Architect — entering or attacking makes a 1/3 Wall; a defender of
/// yours dying fetches a basic land.
pub fn rampart_architect() -> CardDefinition {
    let wall = Arc::new(TokenDefinition {
        name: "Wall".into(),
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        ..Default::default()
    });
    let make_wall = || Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: wall.clone() };
    CardDefinition {
        triggered_abilities: vec![
            etb(make_wall()),
            on_attack(make_wall()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: defender() }),
                effect: Effect::MayDo {
                    description: "Search for a basic land?".into(),
                    body: Box::new(Effect::Search {
                        who: PlayerRef::You,
                        filter: R::IsBasicLand,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    }),
                },
            },
        ],
        ..creature(
            "Rampart Architect",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elephant, CreatureType::Advisor],
            3,
            4,
        )
    }
}

/// Reunion of the House — creature cards with total power 10 or less back
/// from your graveyard; exile it.
pub fn reunion_of_the_house() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Reunion of the House",
            cost(&[generic(5), w(), w()]),
            CardType::Sorcery,
            Effect::ReturnGraveyardCreaturesUpToTotalPower { max_total: Value::Const(10) },
        )
    }
}

/// Slaughter the Strong — each player keeps creatures with total power 4 or
/// less and sacrifices the rest.
pub fn slaughter_the_strong() -> CardDefinition {
    spell(
        "Slaughter the Strong",
        cost(&[generic(1), w(), w()]),
        CardType::Sorcery,
        Effect::EachPlayerKeepsTotalPowerAtMost { max: 4 },
    )
}

/// Tip the Scales — sacrifice a creature; when you do, all creatures get
/// -X/-X, X its toughness (a reflexive trigger, CR 603.12).
pub fn tip_the_scales() -> CardDefinition {
    let minus_x = || Value::Diff(Box::new(Value::ZERO), Box::new(Value::XFromCost));
    spell(
        "Tip the Scales",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::SacrificeAndRemember { who: PlayerRef::You, filter: R::Creature },
            Effect::If {
                cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                then: Box::new(Effect::WithX {
                    x: Value::SacrificedToughness,
                    body: Box::new(Effect::ReflexiveTrigger {
                        body: Box::new(Effect::PumpPT {
                            what: Selector::EachPermanent(R::Creature),
                            power: minus_x(),
                            toughness: minus_x(),
                            duration: Duration::EndOfTurn,
                        }),
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Towering Titan — enters with counters equal to your other creatures'
/// total toughness; sacrifice a defender: everything tramples.
pub fn towering_titan() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::TotalToughnessControlled)),
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature.and(defender()), 1)),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Towering Titan", cost(&[generic(4), g(), g()]), vec![CreatureType::Giant], 0, 0)
    }
}

/// Tree of Redemption — defender; {T}: exchange your life total with its
/// toughness.
pub fn tree_of_redemption() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ExchangeLifeWithSourceToughness,
            ..Default::default()
        }],
        ..creature("Tree of Redemption", cost(&[generic(3), g()]), vec![CreatureType::Plant], 0, 13)
    }
}

/// Walking Bulwark — defender; {2}: a defender gains haste, may attack, and
/// deals combat damage by toughness this turn.
pub fn walking_bulwark() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(defender())),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::AttacksAsThoughNoDefender,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::AssignsCombatDamageByToughness,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Walking Bulwark", cost(&[generic(1)]), vec![CreatureType::Golem], 0, 3)
    }
}

/// Will of the Abzan — any number of target opponents each sacrifice their
/// greatest-power creature and lose 3, or reanimate a creature card from your
/// graveyard; both with a commander. ("Any number of target opponents" is
/// modeled as each opponent: a `ChooseModesCast` mode can't yet own a
/// variable number of target slots.)
pub fn will_of_the_abzan() -> CardDefinition {
    let modes = || {
        vec![
            Effect::ForEachOpponent {
                body: Box::new(Effect::Seq(vec![
                    Effect::SacrificeGreatestMV {
                        who: Selector::Player(PlayerRef::Triggerer),
                        count: Value::ONE,
                        filter: R::Creature,
                        by_power: true,
                    },
                    Effect::LoseLife { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(3) },
                ])),
            },
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        ]
    };
    spell(
        "Will of the Abzan",
        cost(&[generic(3), b()]),
        CardType::Sorcery,
        Effect::ChooseModesCast { modes: modes(), min: 1, max: 1, allow_repeats: false },
    )
    .widening_modes(crate::card::ModesWiden::If(crate::effect::Predicate::YouControlACommander))
}
