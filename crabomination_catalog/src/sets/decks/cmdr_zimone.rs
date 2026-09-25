//! Commander: the cards the **Jump Scare!** precon (DSC, Zimone, Mystery
//! Unraveler) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_zimone.rs`.
//!
//! Residuals (each also on its card):
//! - **Ashaya, Soul of the Wild** — its P/T counts printed lands only, not
//!   the creatures it makes lands.
//! - **Deathmist Raptor** — it returns face up; the face-down option isn't
//!   offered.
//! - **Disorienting Choice** — the targets' controllers decide through the
//!   engine's may-prompt, and the lands found are the engine's pick.
//! - **Zimone, Mystery Unraveler** and **Zimone's Hypothesis** — the
//!   permanent turned face up / the creature given the counter is the
//!   engine's pick, not a free choice (the Hypothesis targets it).

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LandType, RoomDoor,
    RoomDoors, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered, target_n};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, u};
use crate::sets::{enters_tapped, tap_add};

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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn manifest_dread() -> Effect {
    Effect::ManifestDread { who: PlayerRef::You }
}

fn manifest_top() -> Effect {
    Effect::Manifest { who: PlayerRef::You, amount: Value::ONE }
}

/// "Whenever a face-down creature you control enters".
fn face_down_enters(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
            Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::FaceDown.and(R::Creature) },
        ),
        effect,
    }
}

/// "Whenever a permanent you control is turned face up, if it's a creature".
fn creature_turned_up(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
        effect,
    }
}

/// Ashaya, Soul of the Wild — */* equal to your lands; nontoken creatures you
/// control are Forest lands too.
///
/// ⚠ Residual: its P/T counts printed lands only, not the creatures it makes
/// lands.
pub fn ashaya_soul_of_the_wild() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::NotToken).and(R::ControlledByYou));
    legendary(CardDefinition {
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatching { base_p: 0, base_t: 0, filter: Box::new(R::Land) }),
        static_abilities: vec![
            StaticAbility {
                description: "Nontoken creatures you control are lands in addition to their other types.",
                effect: StaticEffect::AddCardTypeToMatching {
                    applies_to: yours(),
                    card_type: CardType::Land,
                    artifact_subtype: None,
                },
            },
            StaticAbility {
                description: "Nontoken creatures you control are Forests in addition to their other types.",
                effect: StaticEffect::LandTypeChanger { applies_to: yours(), land_type: LandType::Forest, replace: false },
            },
        ],
        ..creature("Ashaya, Soul of the Wild", cost(&[generic(3), g(), g()]), vec![CreatureType::Elemental], 0, 0)
    })
}

/// Curator Beastie — reach; colorless creatures you control enter with two
/// more +1/+1 counters; entering or attacking, manifest dread.
pub fn curator_beastie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Colorless creatures you control enter with two additional +1/+1 counters on them.",
            effect: StaticEffect::MatchingEntersWithExtraCounters {
                filter: R::Creature.and(R::Colorless).and(R::ControlledByYou),
                kind: CounterType::PlusOnePlusOne,
                amount: 2,
            },
        }],
        triggered_abilities: vec![etb(manifest_dread()), on_attack(manifest_dread())],
        ..creature("Curator Beastie", cost(&[generic(4), g(), g()]), vec![CreatureType::Beast], 6, 6)
    }
}

/// Deathmist Raptor — deathtouch; megamorph {4}{G}; a permanent of yours
/// turned face up, you may return it from your graveyard.
///
/// ⚠ Residual: it returns face up; the face-down option isn't offered.
pub fn deathmist_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Megamorph(cost(&[generic(4), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::FromYourGraveyard),
            effect: Effect::MayDo {
                description: "Return Deathmist Raptor from your graveyard to the battlefield?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..creature(
            "Deathmist Raptor",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Dinosaur, CreatureType::Beast],
            3,
            3,
        )
    }
}

/// Disorienting Choice — per opponent, up to one of their artifacts or
/// enchantments; its controller may exile it; for each one still there, you
/// fetch a land onto the battlefield tapped.
///
/// ⚠ Residual: the controllers decide through the engine's may-prompt, and
/// the lands found are the engine's pick.
pub fn disorienting_choice() -> CardDefinition {
    CardDefinition {
        name: "Disorienting Choice",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ForEachOpponentTarget {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Artifact.or(R::Enchantment).and(R::ControlledByOpponent),
                    effect: Box::new(Effect::MayDoBy {
                        who: PlayerRef::ControllerOf(Box::new(target_n(0))),
                        description: "Exile your permanent?".into(),
                        body: Box::new(Effect::Exile { what: target_n(0) }),
                    }),
                }),
            },
            Effect::Repeat {
                count: Value::CountOf(Box::new(Selector::MatchingAmong {
                    inner: Box::new(Selector::AllTargets),
                    filter: R::Permanent,
                })),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Land,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Experimental Lab // Staff Room — {3}{G} // {2}{G} Room. Experimental Lab:
/// unlocking it manifests dread, then two +1/+1 counters and a trample
/// counter go on that creature. Staff Room: a creature of yours dealing
/// combat damage to a player turns face up or gets a +1/+1 counter.
pub fn experimental_lab_staff_room() -> CardDefinition {
    CardDefinition {
        name: "Experimental Lab // Staff Room",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Room], ..Default::default() },
        room: Some(Box::new(RoomDoors {
            left: RoomDoor {
                name: "Experimental Lab".into(),
                cost: cost(&[generic(3), g()]),
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DoorUnlocked, EventScope::SelfSource),
                    effect: Effect::Seq(vec![
                        manifest_dread(),
                        Effect::AddCounter {
                            what: Selector::LastMoved,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::Const(2),
                        },
                        Effect::AddKeywordCounter {
                            what: Selector::LastMoved,
                            keyword: Keyword::Trample,
                            amount: Value::ONE,
                        },
                    ]),
                }],
                ..Default::default()
            },
            right: RoomDoor {
                name: "Staff Room".into(),
                cost: cost(&[generic(2), g()]),
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
                    effect: Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::FaceDown },
                        then: Box::new(Effect::TurnFaceUpFree { what: Selector::TriggerSource, if_cant: None }),
                        else_: Box::new(Effect::AddCounter {
                            what: Selector::TriggerSource,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        }),
                    },
                }],
                ..Default::default()
            },
        })),
        ..Default::default()
    }
}

/// Giggling Skitterspike — indestructible; attacking, blocking or becoming
/// the target of a spell, it deals damage equal to its power to each
/// opponent; {5}: monstrosity 5.
pub fn giggling_skitterspike() -> CardDefinition {
    let ping = || Effect::DealDamage {
        to: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::PowerOf(Box::new(Selector::This)),
    };
    let on = |kind: EventKind| TriggeredAbility { event: EventSpec::new(kind, EventScope::SelfSource), effect: ping() };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![on(EventKind::Attacks), on(EventKind::Blocks), on(EventKind::BecameTarget)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: Effect::Monstrosity { n: Value::Const(5) },
            ..Default::default()
        }],
        ..creature("Giggling Skitterspike", cost(&[generic(4)]), vec![CreatureType::Toy], 1, 1)
    }
}

/// Glitch Interpreter — with no face-down permanents it bounces itself and
/// manifests dread; colorless creatures connecting draw you a card.
pub fn glitch_interpreter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::Not(Box::new(Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::FaceDown,
                }))),
                then: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                    manifest_dread(),
                ])),
                else_: Box::new(Effect::Seq(vec![])),
            }),
            TriggeredAbility {
                event: EventSpec {
                    once_per_batch: true,
                    ..EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                        Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::Creature.and(R::Colorless),
                        },
                    )
                },
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..creature(
            "Glitch Interpreter",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Kefnet the Mindful — flying, indestructible; attacks and blocks only with
/// seven or more cards in hand; {3}{U}: draw, then you may return a land.
pub fn kefnet_the_mindful() -> CardDefinition {
    let short_handed = || Predicate::ValueAtMost(Value::HandSizeOf(PlayerRef::You), Value::Const(6));
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Indestructible],
        static_abilities: vec![
            StaticAbility {
                description: "Kefnet can't attack unless you have seven or more cards in hand.",
                effect: StaticEffect::SelfHasKeywordIf { keyword: Keyword::CantAttack, condition: short_handed() },
            },
            StaticAbility {
                description: "Kefnet can't block unless you have seven or more cards in hand.",
                effect: StaticEffect::SelfHasKeywordIf { keyword: Keyword::CantBlock, condition: short_handed() },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::MayDo {
                    description: "Return a land you control to its owner's hand?".into(),
                    body: Box::new(Effect::Move {
                        what: Selector::Take {
                            inner: Box::new(Selector::ControlledBy { who: PlayerRef::You, filter: R::Land }),
                            count: Box::new(Value::ONE),
                        },
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Kefnet the Mindful", cost(&[generic(2), u()]), vec![CreatureType::God], 5, 5)
    })
}

/// Kheru Spellsnatcher — morph {4}{U}{U}; turned face up, it counters target
/// spell, exiling it, and you may cast it free while it stays exiled.
pub fn kheru_spellsnatcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(4), u(), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::CounterSpellExileMayPlayFree { what: target_filtered(R::IsSpellOnStack) },
        }],
        ..creature(
            "Kheru Spellsnatcher",
            cost(&[generic(3), u()]),
            vec![CreatureType::Snake, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Kianne, Corrupted Memory — even power: noncreature spells have flash; odd:
/// creature spells do; each card you draw grows it.
pub fn kianne_corrupted_memory() -> CardDefinition {
    let odd = || Predicate::ValueIsOdd(Value::PowerOf(Box::new(Selector::This)));
    legendary(CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "As long as Kianne's power is even, you may cast noncreature spells as though they had flash.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::Not(Box::new(odd())),
                    inner: Box::new(StaticEffect::ControllerSpellsHaveFlash { filter: R::Creature.negate() }),
                },
            },
            StaticAbility {
                description: "As long as Kianne's power is odd, you may cast creature spells as though they had flash.",
                effect: StaticEffect::WhileCondition {
                    condition: odd(),
                    inner: Box::new(StaticEffect::ControllerSpellsHaveFlash { filter: R::Creature }),
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature(
            "Kianne, Corrupted Memory",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Illusion],
            2,
            2,
        )
    })
}

/// Scroll of Fate — {T}: manifest a card from your hand.
pub fn scroll_of_fate() -> CardDefinition {
    CardDefinition {
        name: "Scroll of Fate",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ManifestFromHand { who: Selector::You, count: Value::ONE, controller_draws: false },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Shriekwood Devourer — trample; attacking, untap up to X lands, X the
/// greatest power among your attackers.
pub fn shriekwood_devourer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::Untap {
                what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Land.and(R::Tapped) },
                up_to: Some(Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(
                    R::IsAttacking.and(R::ControlledByYou),
                )))),
            },
        }],
        ..creature("Shriekwood Devourer", cost(&[generic(5), g(), g()]), vec![CreatureType::Treefolk], 7, 5)
    }
}

/// Skaab Ruinator — flying; exile three creature cards from your graveyard
/// to cast it; castable from your graveyard.
pub fn skaab_ruinator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::GraveyardCast],
        additional_cast_cost: vec![AdditionalCastCost::ExileFromGraveyard { filter: R::Creature, count: 3 }],
        ..creature(
            "Skaab Ruinator",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Zombie, CreatureType::Horror],
            5,
            6,
        )
    }
}

/// Tangled Islet — Land — Forest Island, enters tapped.
pub fn tangled_islet() -> CardDefinition {
    CardDefinition {
        name: "Tangled Islet",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Forest, LandType::Island], ..Default::default() },
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![tap_add(Color::Green), tap_add(Color::Blue)],
        ..Default::default()
    }
}

/// Temur War Shaman — entering, manifest the top card; a creature of yours
/// turned face up may fight target creature you don't control.
pub fn temur_war_shaman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(manifest_top()),
            creature_turned_up(Effect::MayDo {
                description: "Have it fight target creature you don't control?".into(),
                body: Box::new(Effect::Fight {
                    attacker: Selector::TriggerSource,
                    defender: target_filtered(R::Creature.and(R::ControlledByYou.negate())),
                }),
            }),
        ],
        ..creature(
            "Temur War Shaman",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            4,
            5,
        )
    }
}

/// Trail of Mystery — a face-down creature entering may fetch a basic land to
/// hand; a creature turned face up gets +2/+2 until end of turn.
pub fn trail_of_mystery() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            face_down_enters(Effect::MayDo {
                description: "Search for a basic land card?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            }),
            creature_turned_up(Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
        ],
        ..enchantment("Trail of Mystery", cost(&[generic(1), g()]))
    }
}

/// They Came from the Pipes — entering, manifest dread twice; a face-down
/// creature of yours entering draws a card.
pub fn they_came_from_the_pipes() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![manifest_dread(), manifest_dread()])),
            face_down_enters(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        ],
        ..enchantment("They Came from the Pipes", cost(&[generic(4), u()]))
    }
}

/// Whisperwood Elemental — at your end step, manifest the top card; sacrifice
/// it: face-up nontoken creatures you control gain "dies: manifest the top
/// card" until end of turn.
pub fn whisperwood_elemental() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: manifest_top(),
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantTriggeredAbility {
                what: Selector::EachPermanent(
                    R::Creature.and(R::NotToken).and(R::FaceDown.negate()).and(R::ControlledByYou),
                ),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                    effect: manifest_top(),
                }),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Whisperwood Elemental", cost(&[generic(3), g(), g()]), vec![CreatureType::Elemental], 4, 4)
    }
}

/// Zimone's Hypothesis — you may put a +1/+1 counter on a creature, then
/// choose odd or even: each creature with power of that parity returns to
/// its owner's hand.
///
/// ⚠ Residual: the counter's creature is chosen as a target.
pub fn zimones_hypothesis() -> CardDefinition {
    let bounce = |odd: bool| Effect::Move {
        what: Selector::EachPermanent(R::Creature.and(R::PowerParity { odd })),
        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
    };
    CardDefinition {
        name: "Zimone's Hypothesis",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
            Effect::ChooseMode(vec![bounce(true), bounce(false)]),
        ]),
        ..Default::default()
    }
}

/// Zimone, Mystery Unraveler — landfall: the first resolution each turn
/// manifests dread; later ones may turn a permanent you control face up.
///
/// ⚠ Residual: the permanent turned up is the engine's pick.
pub fn zimone_mystery_unraveler() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::EscalatingThisTurn {
                modes: vec![
                    manifest_dread(),
                    Effect::MayDo {
                        description: "Turn a permanent you control face up?".into(),
                        body: Box::new(Effect::TurnFaceUpFree {
                            what: Selector::Take {
                                inner: Box::new(Selector::ControlledBy {
                                    who: PlayerRef::You,
                                    filter: R::FaceDown.and(R::Creature),
                                }),
                                count: Box::new(Value::ONE),
                            },
                            if_cant: None,
                        }),
                    },
                ],
            },
        }],
        ..creature(
            "Zimone, Mystery Unraveler",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    })
}
