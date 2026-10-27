//! Commander: the 2026-10-13 rescan's two-card gaps, fourth pass — Sarulf,
//! Svella, Neyith, Hidetsugu, Etrata, Shanid, Mr. Orfeo, Acererak
//! (`cmdr_edhrec26` is the previous batch). Tests in
//! `tests/recent_b/cmdr_edhrec27.rs`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_any, target_filtered};
use crate::effect::{Effect, LookPick, ManaPayload, PlayerRef, StaticEffect, ZoneDest, ZoneRef};
use crate::mana::{b, cost, g, generic, r, Color, ManaCost};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn on(kind: EventKind, scope: EventScope, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(kind, scope), effect }
}

fn theirs() -> R {
    R::Creature.and(R::Not(Box::new(R::ControlledByYou)))
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

// ── Sarulf, Realm Eater ──────────────────────────────────────────────────────

/// Sarulf, Realm Eater — {1}{B}{G} 3/3 legendary Wolf. An opponent's
/// permanent going to a graveyard from the battlefield puts a +1/+1 counter
/// on it. Your upkeep, with counters: you may remove them all and exile each
/// other nonland permanent with mana value up to the number removed.
pub fn sarulf_realm_eater() -> CardDefinition {
    let counters = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne };
    CardDefinition {
        triggered_abilities: vec![
            on(
                EventKind::PermanentDied,
                EventScope::OpponentControl,
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), EventScope::YourControl)
                    .with_filter(Predicate::ValueAtLeast(counters(), Value::ONE)),
                effect: Effect::MayDo {
                    description: "Remove Sarulf's counters to exile cheaper permanents?".into(),
                    body: Box::new(Effect::WithX {
                        x: counters(),
                        body: Box::new(Effect::Seq(vec![
                            Effect::RemoveCounter {
                                what: Selector::This,
                                kind: CounterType::PlusOnePlusOne,
                                amount: Value::XFromCost,
                            },
                            Effect::Exile {
                                what: Selector::EachPermanent(
                                    R::Nonland.and(R::OtherThanSource).and(R::ManaValueAtMostXFromCost),
                                ),
                            },
                        ])),
                    }),
                },
            },
        ],
        ..legend("Sarulf, Realm Eater", cost(&[generic(1), b(), g()]), vec![CreatureType::Wolf], 3, 3)
    }
}

// ── Svella, Ice Shaper ───────────────────────────────────────────────────────

/// Thunderous Debut — {6}{G}{G} sorcery, bargain. Look at the top twenty;
/// reveal up to two creature cards, onto the battlefield if bargained, else
/// into your hand. Then shuffle.
pub fn thunderous_debut() -> CardDefinition {
    let dig = |to_battlefield: bool| {
        Effect::Seq(vec![
            Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(20),
                take: Some(Value::Const(2)),
                pick_filter: Some(R::Creature),
                to_battlefield,
                optional: true,
                ..Default::default()
            })),
            Effect::ShuffleLibrary { who: PlayerRef::You },
        ])
    };
    CardDefinition {
        name: "Thunderous Debut",
        cost: cost(&[generic(6), g(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Bargain],
        effect: Effect::If {
            cond: Predicate::SpellWasBargained,
            then: Box::new(dig(true)),
            else_: Box::new(dig(false)),
        },
        ..Default::default()
    }
}

// ── Neyith of the Dire Hunt ──────────────────────────────────────────────────

/// Arni Slays the Troll — {R}{G} Saga. I: a creature you control fights up to
/// one you don't. II: add {R} and two +1/+1 counters on up to one creature
/// you control. III: gain life equal to your greatest power.
pub fn arni_slays_the_troll() -> CardDefinition {
    CardDefinition {
        name: "Arni Slays the Troll",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::OptionalTargets {
                    min: 1,
                    body: Box::new(Effect::Fight {
                        attacker: target_filtered(mine()),
                        defender: Selector::TargetFiltered { slot: 1, filter: theirs() },
                    }),
                },
            ),
            (
                2,
                Effect::Seq(vec![
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Red, Value::ONE) },
                    Effect::OptionalTargets {
                        min: 0,
                        body: Box::new(Effect::AddCounter {
                            what: target_filtered(mine()),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::Const(2),
                        }),
                    },
                ]),
            ),
            (3, Effect::GainLife { who: Selector::You, amount: Value::GreatestPowerControlled { who: PlayerRef::You } }),
        ],
        ..Default::default()
    }
}

// ── Hidetsugu, Devouring Chaos ───────────────────────────────────────────────

/// Hidetsugu, Devouring Chaos — {3}{B} 4/4 legendary Ogre Demon. {B},
/// sacrifice a creature: scry 2. {2}{R}, {T}: exile your top card, playable
/// this turn; a nonland card exiled this way deals damage equal to its mana
/// value to any target.
pub fn hidetsugu_devouring_chaos() -> CardDefinition {
    let exiled = || Selector::ExiledThisResolution { filter: R::Nonland };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                sac_other_filter: Some((R::Creature, 1)),
                sac_other_may_be_source: true,
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        duration: MayPlayDuration::EndOfThisTurn,
                        pay_any_color: false,
                        max_mana_value: None,
                        pay_own_cost: true,
                        uncast_penalty: None,
                    },
                    Effect::If {
                        cond: Predicate::SelectorExists(exiled()),
                        then: Box::new(Effect::WithX {
                            x: Value::ManaValueOf(Box::new(exiled())),
                            body: Box::new(Effect::ReflexiveTrigger {
                                body: Box::new(Effect::DealDamage { to: target_any(), amount: Value::XFromCost }),
                            }),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..legend("Hidetsugu, Devouring Chaos", cost(&[generic(3), b()]), vec![CreatureType::Ogre, CreatureType::Demon], 4, 4)
    }
}

// ── Etrata, the Silencer ─────────────────────────────────────────────────────

/// Ravenloft Adventurer — {3}{B} 3/4 Human Rogue Assassin. ETB: take the
/// initiative. An opponent's creature that would die is exiled with a hit
/// counter instead (CR 614). Attacking, after you've completed a dungeon, the
/// defending player loses 1 life per card they own in exile with a hit counter.
pub fn ravenloft_adventurer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a creature an opponent controls would die, instead exile it and put a hit counter on it.",
            effect: StaticEffect::ExileDyingOpponentCreaturesWithCounter { counter: CounterType::Hit },
        }],
        triggered_abilities: vec![
            etb(Effect::TakeInitiative { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                    .with_filter(Predicate::ValueAtLeast(Value::DungeonsCompleted, Value::ONE)),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::DefendingPlayer),
                    amount: Value::count(Selector::EachMatching {
                        zone: ZoneRef::Exile,
                        filter: R::WithCounter(CounterType::Hit).and(R::OwnedByDefendingPlayer),
                    }),
                },
            },
        ],
        ..creature(
            "Ravenloft Adventurer",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Assassin],
            3,
            4,
        )
    }
}

// ── Shanid, Sleepers' Scourge ────────────────────────────────────────────────

/// J. Jonah Jameson — {2}{R} 2/2 legendary Human Citizen. ETB: suspect up to
/// one target creature. A creature of yours with menace attacking makes a
/// Treasure.
pub fn j_jonah_jameson() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Suspect { what: target_filtered(R::Creature) }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasKeyword(Keyword::Menace),
                }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                },
            },
        ],
        ..legend("J. Jonah Jameson", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Citizen], 2, 2)
    }
}

/// Éomer, Marshal of Rohan — {2}{R}{R} 4/4 legendary Human Knight, haste.
/// Once a turn, when one or more other attacking legendary creatures you
/// control die, untap all your creatures; an additional combat follows.
pub fn eomer_marshal_of_rohan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_turn: true,
                ..EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasSupertype(Supertype::Legendary).and(R::IsAttacking),
                    },
                )
            },
            effect: Effect::Seq(vec![
                Effect::Untap { what: Selector::EachPermanent(mine()), up_to: None },
                Effect::AdditionalCombatPhase { count: Value::ONE },
            ]),
        }],
        ..legend("Éomer, Marshal of Rohan", cost(&[generic(2), r(), r()]), vec![CreatureType::Human, CreatureType::Knight], 4, 4)
    }
}

// ── Mr. Orfeo, the Boulder ───────────────────────────────────────────────────

/// Ziatora's Envoy — {1}{B}{R}{G} 5/4 Lizard Warrior, trample, blitz
/// {2}{B}{R}{G}. Its combat damage to a player looks at your top card: a land
/// is played (a land play), a spell with mana value at most the damage is
/// cast free, and otherwise the card is put into your hand (not drawn).
pub fn ziatoras_envoy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(2), b(), r(), g()]),
            blitz: true,
            ..Default::default()
        }),
        triggered_abilities: vec![on(
            EventKind::DealsCombatDamageToPlayer,
            EventScope::SelfSource,
            Effect::WithX {
                x: Value::TriggerEventAmount,
                body: Box::new(Effect::CastTopFreeIfElseDraw {
                    filter: R::Nonland.and(R::ManaValueAtMostXFromCost),
                    land_or_hand: true,
                }),
            },
        )],
        ..creature("Ziatora's Envoy", cost(&[generic(1), b(), r(), g()]), vec![CreatureType::Lizard, CreatureType::Warrior], 5, 4)
    }
}

// ── Acererak the Archlich ────────────────────────────────────────────────────

/// Acererak the Archlich — {2}{B} 5/5 legendary Zombie Wizard. ETB, before
/// you've completed a dungeon: return it to hand and venture. Attacking, each
/// opponent sacrifices a creature or you make a 2/2 black Zombie.
pub fn acererak_the_archlich() -> CardDefinition {
    let zombie = Arc::new(TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    });
    let token_for_acererak = Effect::CreateToken {
        who: PlayerRef::ControllerOf(Box::new(Selector::This)),
        count: Value::ONE,
        definition: zombie,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::Not(Box::new(Predicate::CompletedDungeon {
                        who: PlayerRef::You,
                        name: "Tomb of Annihilation".into(),
                    }))),
                effect: Effect::Seq(vec![
                    Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))) },
                    Effect::Venture,
                ]),
            },
            on_attack(Effect::EachPlayerDoes {
                who: PlayerRef::EachOpponent,
                body: Box::new(Effect::If {
                    cond: Predicate::SelectorExists(Selector::EachPermanent(mine())),
                    then: Box::new(Effect::MayDoElse {
                        description: "Sacrifice a creature (or Acererak's controller makes a Zombie)?".into(),
                        body: Box::new(Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature }),
                        else_: Box::new(token_for_acererak.clone()),
                    }),
                    else_: Box::new(token_for_acererak),
                }),
            }),
        ],
        ..legend("Acererak the Archlich", cost(&[generic(2), b()]), vec![CreatureType::Zombie, CreatureType::Wizard], 5, 5)
    }
}
