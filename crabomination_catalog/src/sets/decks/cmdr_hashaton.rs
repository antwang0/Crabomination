//! Commander: the cards the **Eternal Might** precon (DRC, Hashaton,
//! Scarab's Fist) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_hashaton.rs`.
//!
//! Residuals (each also on its card):
//! - **God-Pharaoh's Gift** — exiles your greatest-power creature card
//!   (the pick is the engine's).
//! - **Rot Hulk** — the returns are the greatest-power Zombie cards, not
//!   targets chosen on entry.
//! - **Gate to the Afterlife** — the loot's "you may draw" is always taken.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, generic, u, w};
use crate::sets::{enters_tapped, tap_add};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn zombie() -> R {
    R::HasCreatureType(CreatureType::Zombie)
}

fn zombie_token(color: Color, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Zombie".into(),
        power: if color == Color::White { 1 } else { 2 },
        toughness: if color == Color::White { 1 } else { 2 },
        keywords,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    })
}

/// "Whenever another Zombie you control enters, …"
fn another_zombie_enters(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
            Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(zombie()) },
        ),
        effect,
    }
}

/// "Create a token that's a copy of [source], except it's a 4/4 black
/// Zombie."
fn black_zombie_copy(source: Selector, tapped: bool) -> Effect {
    Effect::Seq(vec![
        Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::Const(1),
            source,
            extra_creature_types: vec![CreatureType::Zombie],
            extra_card_types: vec![],
            override_pt: Some((4, 4)),
            override_colors: Some(vec![Color::Black]),
            enters_tapped: tapped,
            non_legendary: false,
            legendary: false,
            extra_keywords: vec![],
        },
        // Per the rulings, a Zombie *instead of* its other creature types
        // (unlike eternalize) — CR 707.9b.
        Effect::SetCopiableCreatureTypes { what: Selector::LastCreatedToken, creature_types: vec![CreatureType::Zombie] },
    ])
}

/// Your greatest-power card in `zone` matching `filter`.
fn best_in(zone: Zone, filter: R) -> Selector {
    Selector::TakeGreatestPower {
        inner: Box::new(Selector::CardsInZone { who: PlayerRef::You, zone, filter }),
        count: Box::new(Value::Const(1)),
    }
}

/// Binding Mummy — whenever another Zombie you control enters, you may tap
/// target artifact or creature.
pub fn binding_mummy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![another_zombie_enters(Effect::MayDo {
            description: "Tap target artifact or creature?".into(),
            body: Box::new(Effect::Tap { what: target_filtered(R::Artifact.or(R::Creature)) }),
        })],
        ..creature("Binding Mummy", cost(&[generic(1), w()]), vec![CreatureType::Zombie], 2, 2)
    }
}

/// Crowded Crypt — {T}: {B}; a corpse counter per creature of yours that
/// dies; {4}{B}{B}, {T}, sacrifice it: a decayed 2/2 Zombie per corpse
/// counter.
pub fn crowded_crypt() -> CardDefinition {
    CardDefinition {
        name: "Crowded Crypt",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Corpse, amount: Value::Const(1) },
        }],
        activated_abilities: vec![
            tap_add(Color::Black),
            ActivatedAbility {
                mana_cost: cost(&[generic(4), b(), b()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Corpse },
                    definition: zombie_token(Color::Black, vec![Keyword::Decayed]),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// An Amonkhet cycling Desert: enters tapped, taps for `color`, cycling
/// {1}{color}.
fn cycling_desert(name: &'static str, color: Color, pip: fn() -> crate::mana::ManaSymbol) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Desert], ..Default::default() },
        keywords: vec![Keyword::Cycling(cost(&[generic(1), pip()]))],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![tap_add(color)],
        ..Default::default()
    }
}

pub fn desert_of_the_glorified() -> CardDefinition {
    cycling_desert("Desert of the Glorified", Color::Black, b)
}

pub fn desert_of_the_mindful() -> CardDefinition {
    cycling_desert("Desert of the Mindful", Color::Blue, u)
}

pub fn desert_of_the_true() -> CardDefinition {
    cycling_desert("Desert of the True", Color::White, w)
}

/// Gate to the Afterlife — a nontoken creature of yours dying gains 1 life,
/// then you may loot; {2}, {T}, sacrifice it with six creature cards in your
/// graveyard: fetch God-Pharaoh's Gift from graveyard, hand or library.
pub fn gate_to_the_afterlife() -> CardDefinition {
    let gift = || R::HasName("God-Pharaoh's Gift".into());
    let from = |zone| Selector::CardsInZone { who: PlayerRef::You, zone, filter: gift() };
    let to_bf = || ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false };
    CardDefinition {
        name: "Gate to the Afterlife",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Not(Box::new(R::IsToken)) },
            ),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                Effect::MayDo {
                    description: "Draw a card, then discard a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                        Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                    ])),
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            condition: Some(Predicate::ValueAtLeast(
                Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::Creature,
                })),
                Value::Const(6),
            )),
            effect: Effect::If {
                cond: Predicate::SelectorExists(from(Zone::Graveyard)),
                then: Box::new(Effect::Move { what: from(Zone::Graveyard), to: to_bf() }),
                else_: Box::new(Effect::If {
                    cond: Predicate::SelectorExists(from(Zone::Hand)),
                    then: Box::new(Effect::Move { what: from(Zone::Hand), to: to_bf() }),
                    else_: Box::new(Effect::Search { who: PlayerRef::You, filter: gift(), to: to_bf() }),
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// God-Pharaoh's Gift — at the beginning of combat on your turn, you may
/// exile a creature card from your graveyard to make a hasty 4/4 black
/// Zombie token copy of it.
pub fn god_pharaohs_gift() -> CardDefinition {
    CardDefinition {
        name: "God-Pharaoh's Gift",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::SelectorExists(best_in(Zone::Graveyard, R::Creature)),
                then: Box::new(Effect::MayDo {
                    description: "Exile a creature card for a 4/4 Zombie copy?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move { what: best_in(Zone::Graveyard, R::Creature), to: ZoneDest::Exile },
                        black_zombie_copy(Selector::LastMoved, false),
                        Effect::GrantKeyword {
                            what: Selector::LastCreatedToken,
                            keyword: Keyword::Haste,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Hashaton, Scarab's Fist — whenever you discard a creature card, you may
/// pay {2}{U} for a tapped 4/4 black Zombie token copy of it.
pub fn hashaton_scarabs_fist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::MayPay {
                description: "Pay {2}{U} for a 4/4 Zombie copy of the discarded creature?".into(),
                mana_cost: cost(&[generic(2), u()]),
                body: Box::new(black_zombie_copy(Selector::TriggerSource, true)),
                else_: None,
            },
        }],
        ..creature(
            "Hashaton, Scarab's Fist",
            cost(&[w(), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// On Wings of Gold — your Zombies and tokens get +1/+1 and flying; whenever
/// one or more cards leave your graveyard, a 1/1 white Zombie.
pub fn on_wings_of_gold() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(zombie().or(R::IsToken)));
    CardDefinition {
        name: "On Wings of Gold",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control that are Zombies and/or tokens get +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: yours(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Creatures you control that are Zombies and/or tokens have flying.",
                effect: StaticEffect::GrantKeyword { applies_to: yours(), keyword: Keyword::Flying },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl).once_per_batch(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: zombie_token(Color::White, vec![]),
            },
        }],
        ..Default::default()
    }
}

/// Plague Belcher — menace; enters putting two -1/-1 counters on a creature
/// you control; whenever another Zombie you control dies, each opponent
/// loses 1 life.
pub fn plague_belcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::Const(2),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: zombie() },
                ),
                effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(1) },
            },
        ],
        ..creature("Plague Belcher", cost(&[generic(2), b()]), vec![CreatureType::Zombie, CreatureType::Beast], 5, 4)
    }
}

/// Priest of the Crossing — flying; at the beginning of each end step, X
/// +1/+1 counters on each creature you control, X the creatures that died
/// under your control this turn.
pub fn priest_of_the_crossing() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CreaturesDiedThisTurn(PlayerRef::You),
            },
        }],
        ..creature(
            "Priest of the Crossing",
            cost(&[generic(3), w()]),
            vec![CreatureType::Zombie, CreatureType::Bird, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Prophet of the Scarab — vigilance; on entry, draw the greater of your
/// Zombies and the Zombie cards in your graveyard.
pub fn prophet_of_the_scarab() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Max(
                    Box::new(Value::CountOf(Box::new(Selector::EachPermanent(zombie().and(R::ControlledByYou))))),
                    Box::new(Value::CountOf(Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: zombie(),
                    }))),
                ),
            },
        }],
        ..creature(
            "Prophet of the Scarab",
            cost(&[generic(4), u()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Rhet-Tomb Mystic — flying; each creature card in your hand has cycling
/// {1}{U}.
pub fn rhet_tomb_mystic() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Each creature card in your hand has cycling {1}{U}.",
            effect: StaticEffect::GrantCyclingToYourHandCards { filter: R::Creature, cost: cost(&[generic(1), u()]) },
        }],
        ..creature(
            "Rhet-Tomb Mystic",
            cost(&[generic(1), u()]),
            vec![CreatureType::Zombie, CreatureType::Bird, CreatureType::Wizard],
            2,
            1,
        )
    }
}

/// Rot Hulk — menace; on entry, return up to X Zombie cards from your
/// graveyard to the battlefield, X the number of opponents.
pub fn rot_hulk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Repeat {
                count: Value::OpponentCount,
                body: Box::new(Effect::Move {
                    what: best_in(Zone::Graveyard, R::Creature.and(zombie())),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..creature("Rot Hulk", cost(&[generic(5), b(), b()]), vec![CreatureType::Zombie], 5, 5)
    }
}

/// Temmet, Naktamun's Will — vigilance, menace; whenever you attack, loot;
/// whenever you draw a card, Zombies you control get +1/+1 until end of turn.
pub fn temmet_naktamuns_will() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance, Keyword::Menace],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(zombie().and(R::ControlledByYou)),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..creature(
            "Temmet, Naktamun's Will",
            cost(&[generic(2), w(), u(), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Wayward Servant — whenever another Zombie you control enters, each
/// opponent loses 1 life and you gain 1 life.
pub fn wayward_servant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![another_zombie_enters(Effect::Seq(vec![
            Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(1) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..creature("Wayward Servant", cost(&[w(), b()]), vec![CreatureType::Zombie], 2, 2)
    }
}

/// Wizened Mentor — whenever an opponent activates a non-mana ability of a
/// permanent, you create a 1/1 white Zombie (once each turn).
pub fn wizened_mentor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivated, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Permanent })
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: zombie_token(Color::White, vec![]),
            },
        }],
        ..creature("Wizened Mentor", cost(&[generic(1), w()]), vec![CreatureType::Zombie, CreatureType::Cleric], 2, 2)
    }
}
