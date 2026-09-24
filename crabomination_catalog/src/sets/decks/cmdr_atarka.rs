//! Commander: the cards the **Draconic Destruction** starter deck (SCD,
//! Atarka, World Render) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_atarka.rs`.
//!
//! Residuals (each also on its card):
//! - **Atarka Monument** — animated, it is a colorless Dragon (the printed
//!   red-and-green isn't applied).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnterMode, EventKind, EventScope,
    EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, cost, g, generic, r};
use std::sync::Arc;

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

fn dragon(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Flying], ..creature(name, mana, vec![CreatureType::Dragon], p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn is_dragon() -> R {
    R::HasCreatureType(CreatureType::Dragon)
}

fn tap_for(pool: ManaPayload) -> ActivatedAbility {
    ActivatedAbility { tap_cost: true, effect: Effect::AddMana { who: PlayerRef::You, pool }, ..Default::default() }
}

fn dragon_token(name: &str, legendary: bool) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: if legendary { 4 } else { 5 },
        toughness: if legendary { 4 } else { 5 },
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        supertypes: if legendary { vec![Supertype::Legendary] } else { vec![] },
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        ..Default::default()
    })
}

/// "Whenever a Dragon you control enters" — this one included.
fn dragon_enters(scope: EventScope, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, scope).with_filter(Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: R::Creature.and(is_dragon()).and(R::ControlledByYou),
        }),
        effect,
    }
}

/// Atarka Monument — {T}: {R} or {G}; {4}{R}{G}: a 4/4 flying Dragon artifact
/// creature until end of turn.
/// Residual: the animated Dragon is colorless.
pub fn atarka_monument() -> CardDefinition {
    CardDefinition {
        name: "Atarka Monument",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            tap_for(ManaPayload::OfColor(Color::Red, Value::ONE)),
            tap_for(ManaPayload::OfColor(Color::Green, Value::ONE)),
            ActivatedAbility {
                mana_cost: cost(&[generic(4), r(), g()]),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    creature_types: vec![CreatureType::Dragon],
                    keywords: vec![Keyword::Flying],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Atarka, World Render — flying, trample; an attacking Dragon of yours gains
/// double strike until end of turn.
pub fn atarka_world_render() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: is_dragon() }),
            effect: Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Atarka, World Render", cost(&[generic(5), r(), g()]), vec![CreatureType::Dragon], 6, 4)
    }
}

/// Crucible of Fire — your Dragon creatures get +3/+3.
pub fn crucible_of_fire() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_(
            "Dragon creatures you control get +3/+3.",
            StaticEffect::PumpPT { applies_to: yours(R::Creature.and(is_dragon())), power: 3, toughness: 3 },
        )],
        ..spell("Crucible of Fire", cost(&[generic(3), r()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Demanding Dragon — flying; enters: 5 damage to target opponent unless they
/// sacrifice a creature.
pub fn demanding_dragon() -> CardDefinition {
    // The Browbeat shape: the target is declared by `otherwise` (it runs in
    // the controller's context); the accepter is bound to slot 0.
    CardDefinition {
        triggered_abilities: vec![etb(Effect::PlayersMayAccept {
            who: PlayerRef::Target(0),
            description: "Sacrifice a creature rather than take 5 damage?".into(),
            on_accept: Box::new(Effect::Sacrifice { who: Selector::Target(0), count: Value::ONE, filter: R::Creature }),
            if_any: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer),
                amount: Value::Const(5),
            }),
        })],
        ..dragon("Demanding Dragon", cost(&[generic(3), r(), r()]), 5, 5)
    }
}

/// Draconic Disciple — {T}: any color; {7},{T}, sacrifice it: a 5/5 flying
/// Dragon.
pub fn draconic_disciple() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            crate::sets::tap_add_any_color(),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(7)]),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: dragon_token("Dragon", false),
                },
                ..Default::default()
            },
        ],
        ..creature("Draconic Disciple", cost(&[generic(1), r(), g()]), vec![CreatureType::Human, CreatureType::Shaman], 2, 2)
    }
}

/// Dragon's Hoard — a gold counter per Dragon of yours entering; {T}, remove
/// one: draw; {T}: any color.
pub fn dragons_hoard() -> CardDefinition {
    CardDefinition {
        name: "Dragon's Hoard",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![dragon_enters(
            EventScope::YourControl,
            Effect::AddCounter { what: Selector::This, kind: CounterType::Gold, amount: Value::ONE },
        )],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Gold, 1)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
            crate::sets::tap_add_any_color(),
        ],
        ..Default::default()
    }
}

/// Drakuseth, Maw of Flames — flying; attacking, 4 damage to any target and
/// 3 to each of up to two other targets.
pub fn drakuseth_maw_of_flames() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::OptionalTargets {
                min: 1,
                body: Box::new(Effect::Seq(vec![
                    Effect::DealDamage { to: target_any(), amount: Value::Const(4) },
                    Effect::DealDamage {
                        to: Selector::TargetFiltered { slot: 1, filter: R::Any.and(R::OtherThanTargetSlot(0)) },
                        amount: Value::Const(3),
                    },
                    Effect::DealDamage {
                        to: Selector::TargetFiltered {
                            slot: 2,
                            filter: R::Any.and(R::OtherThanTargetSlot(0)).and(R::OtherThanTargetSlot(1)),
                        },
                        amount: Value::Const(3),
                    },
                ])),
            },
        }],
        ..dragon("Drakuseth, Maw of Flames", cost(&[generic(4), r(), r(), r()]), 7, 7)
    }
}

/// Foe-Razer Regent — flying; enters: you may have it fight target creature
/// you don't control; a creature of yours that fights gets two +1/+1 counters
/// at the next end step.
pub fn foe_razer_regent() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Fight target creature you don't control?".into(),
                body: Box::new(Effect::Fight {
                    attacker: Selector::This,
                    defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Fights, EventScope::YourControl),
                effect: Effect::DelayUntilWithCapture {
                    kind: DelayedTriggerKind::NextEndStep,
                    capture: Selector::TriggerSource,
                    body: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    }),
                },
            },
        ],
        ..dragon("Foe-Razer Regent", cost(&[generic(5), g(), g()]), 4, 5)
    }
}

/// Frontier Siege — Khans: {G}{G} at each of your main phases; Dragons: a
/// flyer of yours entering may fight target creature you don't control.
pub fn frontier_siege() -> CardDefinition {
    let main = |step: TurnStep| TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl),
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Green, Value::Const(2)) },
    };
    CardDefinition {
        name: "Frontier Siege",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        enter_modes: Some(vec![
            EnterMode {
                label: "Khans",
                triggered_abilities: vec![main(TurnStep::PreCombatMain), main(TurnStep::PostCombatMain)],
                ..Default::default()
            },
            EnterMode {
                label: "Dragons",
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                        Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
                        },
                    ),
                    effect: Effect::MayDo {
                        description: "Have it fight target creature you don't control?".into(),
                        body: Box::new(Effect::Fight {
                            attacker: Selector::TriggerSource,
                            defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                        }),
                    },
                }],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Harbinger of the Hunt — flying; {2}{R}: 1 damage to each creature without
/// flying; {2}{G}: 1 damage to each other flyer.
pub fn harbinger_of_the_hunt() -> CardDefinition {
    let ping = |mana: ManaCost, filter: R| ActivatedAbility {
        mana_cost: mana,
        effect: Effect::DealDamage { to: Selector::EachPermanent(R::Creature.and(filter)), amount: Value::ONE },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            ping(cost(&[generic(2), r()]), R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
            ping(cost(&[generic(2), g()]), R::HasKeyword(Keyword::Flying).and(R::OtherThanSource)),
        ],
        ..dragon("Harbinger of the Hunt", cost(&[generic(3), r(), g()]), 5, 3)
    }
}

/// Haven of the Spirit Dragon — {T}: {C}; {T}: any color for Dragon creature
/// spells; {2},{T}, sacrifice it: return a Dragon creature or Ugin card from
/// your graveyard to hand.
pub fn haven_of_the_spirit_dragon() -> CardDefinition {
    CardDefinition {
        name: "Haven of the Spirit Dragon",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            tap_for(ManaPayload::Restricted(
                Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                SpendRestriction::CreatureSpellOfType(CreatureType::Dragon),
            )),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::Move {
                    what: target_filtered(
                        R::Creature
                            .and(is_dragon())
                            .or(R::HasPlaneswalkerType(PlaneswalkerSubtype::Ugin))
                            .and(R::InYourGraveyard),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hunter's Insight — until end of turn, target creature you control drawing
/// you cards equal to its combat damage to a player or planeswalker.
pub fn hunters_insight() -> CardDefinition {
    let draw = |kind: EventKind| crate::card::TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
    };
    spell(
        "Hunter's Insight",
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GrantTriggeredAbility {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                trigger: Box::new(draw(EventKind::DealsCombatDamageToPlayer)),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(draw(EventKind::DealsCombatDamageToPlaneswalker)),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Provoke the Trolls — 3 damage to any target; a creature dealt damage this
/// way gets +5/+0 until end of turn.
pub fn provoke_the_trolls() -> CardDefinition {
    spell(
        "Provoke the Trolls",
        cost(&[generic(3), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Creature },
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(5),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Sarkhan, the Dragonspeaker — +1: a 4/4 flying, indestructible, hasty
/// Dragon until end of turn; −3: 4 damage to target creature; −6: an emblem
/// with two extra draws and an end-step hand discard.
pub fn sarkhan_the_dragonspeaker() -> CardDefinition {
    CardDefinition {
        name: "Sarkhan, the Dragonspeaker",
        cost: cost(&[generic(3), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Sarkhan], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    creature_types: vec![CreatureType::Dragon],
                    keywords: vec![Keyword::Flying, Keyword::Indestructible, Keyword::Haste],
                    duration: Duration::EndOfTurn,
                },
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Sarkhan, the Dragonspeaker".into(),
                    triggered: vec![
                        TriggeredAbility {
                            event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::YourControl),
                            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                        },
                        TriggeredAbility {
                            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                            effect: Effect::Discard {
                                who: Selector::You,
                                amount: Value::HandSizeOf(PlayerRef::You),
                                random: false,
                            },
                        },
                    ],
                    statics: vec![],
                },
                x_cost: false,
            },
        ],
        ..Default::default()
    }
}

/// Scourge of Valkas — flying; it or another Dragon of yours entering deals
/// damage equal to your Dragon count to any target; {R}: +1/+0.
pub fn scourge_of_valkas() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![dragon_enters(
            EventScope::YourControl,
            Effect::DealDamageFrom {
                source: Selector::TriggerSource,
                to: target_any(),
                amount: Value::CountOf(Box::new(yours(R::Creature.and(is_dragon())))),
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..dragon("Scourge of Valkas", cost(&[generic(2), r(), r(), r()]), 4, 4)
    }
}

/// Spit Flame — 4 damage to target creature; from your graveyard, a Dragon of
/// yours entering lets you pay {R} to return it to hand.
pub fn spit_flame() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![dragon_enters(
            EventScope::FromYourGraveyard,
            Effect::MayPay {
                description: "Pay {R} to return Spit Flame to your hand?".into(),
                mana_cost: cost(&[r()]),
                body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) }),
                else_: None,
            },
        )],
        ..spell(
            "Spit Flame",
            cost(&[generic(2), r()]),
            CardType::Instant,
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
        )
    }
}

/// Thunderbreak Regent — flying; an opponent targeting a Dragon of yours takes
/// 3 damage.
pub fn thunderbreak_regent() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourPermanentTargetedByOpponent)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: is_dragon() }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::Const(3),
            },
        }],
        ..dragon("Thunderbreak Regent", cost(&[generic(2), r(), r()]), 4, 4)
    }
}

/// Unleash Fury — double target creature's power until end of turn.
pub fn unleash_fury() -> CardDefinition {
    spell(
        "Unleash Fury",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::PowerOf(Box::new(Selector::Target(0))),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Verix Bladewing — kicker {3}; flying; entering kicked, it brings Karox
/// Bladewing, a legendary 4/4 flying Dragon token.
pub fn verix_bladewing() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[generic(3)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasKicked),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: dragon_token("Karox Bladewing", true),
            },
        }],
        ..creature("Verix Bladewing", cost(&[generic(2), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}
