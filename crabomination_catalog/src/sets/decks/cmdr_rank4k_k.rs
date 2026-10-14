//! Commander: gap cards of EDHREC's commanders ranked 3501-4500 (the
//! 2026-10-14 scan), eleventh file. Tests in `tests/recent_b/cmdr_rank4k_k.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    CumulativeUpkeepCost, EventKind, Predicate,
    EventScope, EventSpec, Keyword, LandType, MayPlayDuration, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    ChainCopyCost, Duration, Effect, ExtraManaKind, PlayerRef, Selector, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

// ── Witch-king, Bringer of Ruin / Interceptor, Shadow's Hound ────────────────

/// Gollum, Scheming Guide — {1}{B} legendary 2/1 Halfling Horror. Whenever it
/// attacks, look at the top two cards of your library and put them back in any
/// order, then choose land or nonland; an opponent guesses whether the top card
/// is the chosen kind, and it's revealed. Right: remove Gollum from combat.
/// Wrong: draw a card and Gollum can't be blocked this turn.
pub fn gollum_scheming_guide() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::RearrangeTop {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::GuessTopCardKind {
                right: Box::new(Effect::RemoveFromCombat {
                    what: Selector::This,
                }),
                wrong: Box::new(Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Unblockable,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        ]))],
        ..creature(
            "Gollum, Scheming Guide",
            cost(&[generic(1), b()]),
            vec![CreatureType::Halfling, CreatureType::Horror],
            2,
            1,
        )
    }
}

/// Poison-Blade Mentor — {1}{B} 2/1 deathtouch Human Assassin. Whenever it
/// attacks, another target Assassin you control gains deathtouch until end of
/// turn.
pub fn poison_blade_mentor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(
                R::HasCreatureType(CreatureType::Assassin)
                    .and(R::ControlledByYou)
                    .and(R::OtherThanSource),
            ),
            keyword: Keyword::Deathtouch,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Poison-Blade Mentor",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Assassin],
            2,
            1,
        )
    }
}

// ── Denethor, Stone Seer ─────────────────────────────────────────────────────

/// Mudslide — {2}{R} enchantment. Creatures without flying don't untap during
/// their controllers' untap steps. At the beginning of each player's upkeep,
/// that player may pay {2} for each of their tapped creatures without flying;
/// untap each one paid for.
pub fn mudslide() -> CardDefinition {
    let grounded = || R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying))));
    CardDefinition {
        name: "Mudslide",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures without flying don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntapGlobal {
                applies_to: Selector::EachPermanent(grounded()),
                condition: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::ForEach {
                selector: Selector::ControlledBy {
                    who: PlayerRef::ActivePlayer,
                    filter: grounded().and(R::Tapped),
                },
                body: Box::new(Effect::MayPayBy {
                    who: PlayerRef::ActivePlayer,
                    description: "Pay {2} to untap this creature?".into(),
                    mana_cost: cost(&[generic(2)]),
                    body: Box::new(Effect::Untap {
                        what: Selector::TriggerSource,
                        up_to: None,
                    }),
                    else_: None,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Myojin of Seeing Winds ───────────────────────────────────────────────────

/// String of Disappearances — {U} instant. Return target creature to its
/// owner's hand. Then that creature's controller may pay {U}{U} to copy this
/// spell and choose a new target for the copy.
pub fn string_of_disappearances() -> CardDefinition {
    CardDefinition {
        name: "String of Disappearances",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::MayCopyThisSpell {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                cost: ChainCopyCost::Mana(cost(&[u(), u()])),
            },
        ]),
        ..Default::default()
    }
}

// ── Wiccan, Young Avenger ────────────────────────────────────────────────────

/// Asgardian Inspiration — {R} sorcery. Exile the top card of your library;
/// you may play it this turn. From your graveyard: whenever a source you
/// control deals noncombat damage to an opponent, you may pay {2} to return
/// this card to your hand.
pub fn asgardian_inspiration() -> CardDefinition {
    CardDefinition {
        name: "Asgardian Inspiration",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::ONE,
            duration: MayPlayDuration::EndOfThisTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PlayerDealtNoncombatDamage,
                EventScope::FromYourGraveyardSourceDamagedOpponent,
            ),
            effect: Effect::MayPay {
                description: "Pay {2} to return Asgardian Inspiration to your hand?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

// ── Kodama of the North Tree ─────────────────────────────────────────────────

/// Blessing of Frost — {3}{G} snow sorcery. Distribute X +1/+1 counters among
/// any number of creatures you control, where X is the amount of {S} spent to
/// cast it. Then draw a card for each creature you control with power 4 or
/// greater.
pub fn blessing_of_frost() -> CardDefinition {
    CardDefinition {
        name: "Blessing of Frost",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DistributeCountersAmong {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                total: Value::SnowManaSpentToCastSource,
                counter: CounterType::PlusOnePlusOne,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                ))),
            },
        ]),
        ..Default::default()
    }
}

// ── The Duke, Rebel Sentry ───────────────────────────────────────────────────

/// Swallow Whole — {W} sorcery. As an additional cost, tap an untapped
/// creature you control. Exile target tapped creature. Put a +1/+1 counter on
/// the creature tapped to pay the additional cost.
pub fn swallow_whole() -> CardDefinition {
    CardDefinition {
        name: "Swallow Whole",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::TapPermanents {
            filter: R::Creature,
            count: 1,
        }],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(R::Creature.and(R::Tapped)),
            },
            Effect::AddCounter {
                what: Selector::CostTappedPermanentOfSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

// ── Autumn Willow, Harmony ───────────────────────────────────────────────────

/// Autumn Willow, Harmony — {3}{G}{G} legendary 3/3 hexproof Avatar. ETB:
/// create a 1/1 green Forest Dryad land creature token. Whenever you tap a
/// land creature for mana, add an additional {G}.
pub fn autumn_willow_harmony() -> CardDefinition {
    let dryad = TokenDefinition {
        name: "Forest Dryad".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Land, CardType::Creature],
        colors: vec![Color::Green],
        activated_abilities: vec![crate::sets::tap_add(Color::Green)],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad],
            land_types: vec![LandType::Forest],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(dryad),
        })],
        static_abilities: vec![StaticAbility {
            description: "Whenever you tap a land creature for mana, add an additional {G}.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: false,
                filter: R::Creature.and(R::Land).and(R::ControlledByYou),
                extra: ExtraManaKind::Fixed(Color::Green),
                while_monarch: false,
            },
        }],
        supertypes: vec![Supertype::Legendary],
        ..creature(
            "Autumn Willow, Harmony",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Avatar],
            3,
            3,
        )
    }
}

// ── Donnie & April, Adorkable Duo ────────────────────────────────────────────

/// Donnie & April, Adorkable Duo — {4}{U} legendary 3/3 Mutant Ninja Human
/// Turtle. ETB, choose one or both (each mode targets a different player):
/// target player draws two cards; target player returns an artifact,
/// instant, or sorcery card from their graveyard to their hand. (Residual:
/// the "different player" constraint isn't enforced.)
pub fn donnie_and_april_adorkable_duo() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                Effect::Draw {
                    who: target_filtered(R::Player),
                    amount: Value::Const(2),
                },
                Effect::ForEach {
                    selector: target_filtered(R::Player),
                    body: Box::new(Effect::Move {
                        what: Selector::Take {
                            inner: Box::new(Selector::CardsInZone {
                                who: PlayerRef::Triggerer,
                                zone: Zone::Graveyard,
                                filter: R::HasCardType(CardType::Artifact)
                                    .or(R::HasCardType(CardType::Instant))
                                    .or(R::HasCardType(CardType::Sorcery)),
                            }),
                            count: Box::new(Value::ONE),
                        },
                        to: ZoneDest::Hand(PlayerRef::Triggerer),
                    }),
                },
            ],
        })],
        supertypes: vec![Supertype::Legendary],
        ..creature(
            "Donnie & April, Adorkable Duo",
            cost(&[generic(4), u()]),
            vec![
                CreatureType::Mutant,
                CreatureType::Ninja,
                CreatureType::Human,
                CreatureType::Turtle,
            ],
            3,
            3,
        )
    }
}

// ── Oriss, Samite Guardian ───────────────────────────────────────────────────

/// Oriss, Samite Guardian — {1}{W}{W} legendary 1/3 Human Cleric. {T}: prevent
/// all damage that would be dealt to target creature this turn. Grandeur —
/// discard another card named Oriss: target player can't cast spells this
/// turn, and creatures that player controls can't attack this turn.
pub fn oriss_samite_guardian() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::PreventAllDamageThisTurn {
                    target: target_filtered(R::Creature),
                    redirect_to: None,
                },
                ..Default::default()
            },
            ActivatedAbility {
                discard_cost: Some((R::HasName("Oriss, Samite Guardian".to_string()), 1)),
                effect: Effect::TargetPlayerThen {
                    filter: R::Player,
                    then: Box::new(Effect::Seq(vec![
                        Effect::SilencePlayersThisTurn {
                            who: PlayerRef::Target(0),
                        },
                        Effect::GrantKeyword {
                            what: Selector::ControlledBy {
                                who: PlayerRef::Target(0),
                                filter: R::Creature,
                            },
                            keyword: Keyword::CantAttack,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
                ..Default::default()
            },
        ],
        supertypes: vec![Supertype::Legendary],
        ..creature(
            "Oriss, Samite Guardian",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            3,
        )
    }
}

/// Weathered Bodyguards — {5}{W} 2/5 Human Soldier. As long as it's untapped,
/// all combat damage that would be dealt to you by unblocked creatures is
/// dealt to it instead. Morph {3}{W}.
pub fn weathered_bodyguards() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), w()]))],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is untapped, all combat damage that would be dealt to you by unblocked creatures is dealt to this creature instead.",
            effect: StaticEffect::AbsorbUnblockedCombatDamageWhileUntapped,
        }],
        ..creature(
            "Weathered Bodyguards",
            cost(&[generic(5), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            5,
        )
    }
}

// ── Tetsuko Umezawa, Pursuer ─────────────────────────────────────────────────

/// Tetsuko Umezawa, Pursuer — {3}{R} legendary 2/4 Human Mercenary; double
/// strike, prowess. Whenever a creature an opponent controls with power or
/// toughness 1 or less blocks, Tetsuko deals 1 damage to that creature's
/// controller.
pub fn tetsuko_umezawa_pursuer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike, Keyword::Prowess],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtMost(1).or(R::ToughnessAtMost(1))),
                },
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        supertypes: vec![Supertype::Legendary],
        ..creature(
            "Tetsuko Umezawa, Pursuer",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            2,
            4,
        )
    }
}

/// Varchild's War-Riders — {1}{R} 3/4 Human Warrior; trample, rampage 1.
/// Cumulative upkeep—have an opponent create a 1/1 red Survivor creature
/// token.
pub fn varchilds_war_riders() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CumulativeUpkeep(CumulativeUpkeepCost::OpponentCreatesSurvivor),
            Keyword::Trample,
            Keyword::Rampage(1),
        ],
        ..creature(
            "Varchild's War-Riders",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            4,
        )
    }
}
