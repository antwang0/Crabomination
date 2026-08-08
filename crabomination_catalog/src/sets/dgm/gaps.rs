//! Dragon's Maze (DGM) gap cards — guild legends, mythics, and remaining
//! commons/uncommons on existing (or newly-added) primitives. Tests in
//! `classic_sets/dgm`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EntersAsCopy,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    battalion, cast_is_noncreature, etb, target_any, target_filtered, unleash,
};
use crate::effect::{Duration, ExtraManaKind, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w, x};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// Sire of Insanity — {4}{B}{R} 6/4. At the beginning of each end step, each
/// player discards their hand.
pub fn sire_of_insanity() -> CardDefinition {
    CardDefinition {
        name: "Sire of Insanity",
        cost: cost(&[generic(4), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Demon]),
        power: 6,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(100), // capped at hand size — "their hand"
                random: false,
            },
        }],
        ..Default::default()
    }
}

/// Savageborn Hydra — {X}{R}{G} 0/0 Hydra with double strike. Enters with X
/// +1/+1 counters. {1}{R/G}: put a +1/+1 counter on it (sorcery speed).
pub fn savageborn_hydra() -> CardDefinition {
    CardDefinition {
        name: "Savageborn Hydra",
        cost: cost(&[x(), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Hydra]),
        keywords: vec![Keyword::DoubleStrike],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), hybrid(Color::Red, Color::Green)]),
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Exava, Rakdos Blood Witch — {2}{B}{R} 3/3 with first strike, haste, Unleash.
/// Each other creature you control with a +1/+1 counter has haste.
pub fn exava_rakdos_blood_witch() -> CardDefinition {
    CardDefinition {
        name: "Exava, Rakdos Blood Witch",
        cost: cost(&[generic(2), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Cleric]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![unleash()],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control with a +1/+1 counter on them have haste.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature
                    .and(R::ControlledByYou)
                    .and(R::OtherThanSource)
                    .and(R::WithCounter(CounterType::PlusOnePlusOne)),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Haste],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Ruric Thar, the Unbowed — {4}{R}{G} 6/6 with vigilance, reach. Attacks each
/// combat if able. Whenever a player casts a noncreature spell, deals 6 damage
/// to that player.
pub fn ruric_thar_the_unbowed() -> CardDefinition {
    CardDefinition {
        name: "Ruric Thar, the Unbowed",
        cost: cost(&[generic(4), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Ogre, CreatureType::Warrior]),
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Vigilance, Keyword::Reach, Keyword::MustAttack],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(cast_is_noncreature()),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(6),
            },
        }],
        ..Default::default()
    }
}

/// Lavinia of the Tenth — {3}{W}{U} 4/4 with protection from red. ETB: detain
/// each nonland permanent your opponents control with mana value 4 or less.
pub fn lavinia_of_the_tenth() -> CardDefinition {
    CardDefinition {
        name: "Lavinia of the Tenth",
        cost: cost(&[generic(3), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Soldier]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Protection(Color::Red)],
        triggered_abilities: vec![etb(Effect::Detain {
            what: Selector::EachPermanent(
                R::Nonland
                    .and(R::ControlledByOpponent)
                    .and(R::ManaValueAtMost(4)),
            ),
        })],
        ..Default::default()
    }
}

/// Blood Baron of Vizkopa — {3}{W}{B} 4/4 with lifelink, protection from white
/// and from black. While you have 30+ life and an opponent has 10 or less, it
/// gets +6/+6 and has flying.
pub fn blood_baron_of_vizkopa() -> CardDefinition {
    use crate::effect::StaticEffect::PumpSelfIf;
    CardDefinition {
        name: "Blood Baron of Vizkopa",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Vampire]),
        power: 4,
        toughness: 4,
        keywords: vec![
            Keyword::Lifelink,
            Keyword::Protection(Color::White),
            Keyword::Protection(Color::Black),
        ],
        static_abilities: vec![StaticAbility {
            description: "While you have 30+ life and an opponent has 10 or less, +6/+6 and flying.",
            effect: PumpSelfIf {
                condition: Predicate::All(vec![
                    Predicate::PlayerLifeAtLeast {
                        who: PlayerRef::You,
                        life: 30,
                    },
                    Predicate::PlayerLifeAtMost {
                        who: PlayerRef::EachOpponent,
                        life: 10,
                    },
                ]),
                power: 6,
                toughness: 6,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..Default::default()
    }
}

/// Mirko Vosk, Mind Drinker — {3}{U}{B} 2/4 flyer. Combat damage to a player:
/// that player reveals from the top until four lands, then mills them all.
pub fn mirko_vosk_mind_drinker() -> CardDefinition {
    CardDefinition {
        name: "Mirko Vosk, Mind Drinker",
        cost: cost(&[generic(3), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Vampire]),
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MillUntilLands {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                lands: Value::Const(4),
            },
        }],
        ..Default::default()
    }
}

/// Tajic, Blade of the Legion — {2}{R}{W} 2/2, indestructible. Battalion:
/// whenever Tajic and 2+ other creatures attack, Tajic gets +5/+5.
pub fn tajic_blade_of_the_legion() -> CardDefinition {
    CardDefinition {
        name: "Tajic, Blade of the Legion",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Soldier]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![battalion(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(5),
            toughness: Value::Const(5),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Vorel of the Hull Clade — {1}{G}{U} 1/4. {G}{U}, {T}: double the number of
/// each kind of counter on target artifact, creature, or land.
pub fn vorel_of_the_hull_clade() -> CardDefinition {
    CardDefinition {
        name: "Vorel of the Hull Clade",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Merfolk]),
        power: 1,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), u()]),
            tap_cost: true,
            effect: Effect::DoubleAllCountersOn {
                what: target_filtered(R::Artifact.or(R::Creature).or(R::Land)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Zhur-Taa Ancient — {3}{R}{G} 7/5. Whenever a player taps a land for mana,
/// that player adds one mana of any type that land produced.
pub fn zhur_taa_ancient() -> CardDefinition {
    CardDefinition {
        name: "Zhur-Taa Ancient",
        cost: cost(&[generic(3), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Beast]),
        power: 7,
        toughness: 5,
        static_abilities: vec![StaticAbility {
            description: "A land tapped for mana produces one extra of that type.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: false,
                filter: R::Any,
                extra: ExtraManaKind::Mirror,
                while_monarch: false,
            },
        }],
        ..Default::default()
    }
}

/// Smelt-Ward Gatekeepers — {3}{R} 2/4. ETB with two+ Gates: gain control of
/// target creature an opponent controls until end of turn, untap it, and it
/// gains haste.
pub fn smelt_ward_gatekeepers() -> CardDefinition {
    CardDefinition {
        name: "Smelt-Ward Gatekeepers",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Warrior]),
        power: 2,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::If {
            cond: super::creatures::two_gates(),
            then: Box::new(Effect::Seq(vec![
                Effect::GainControl {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
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
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Scion of Vitu-Ghazi — {3}{W}{W} 4/4. ETB, if you cast it: create a 1/1 white
/// Bird with flying, then populate.
pub fn scion_of_vitu_ghazi() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Bird".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: creatures(vec![CreatureType::Bird]),
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Scion of Vitu-Ghazi",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elemental]),
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SourceWasCast,
            then: Box::new(Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(bird),
                },
                Effect::Populate {
                    who: PlayerRef::You,
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Rot Farm Skeleton — {2}{B}{G} 4/1 that can't block. {2}{B}{G}, Mill four
/// cards: return this from your graveyard to the battlefield (sorcery speed).
pub fn rot_farm_skeleton() -> CardDefinition {
    CardDefinition {
        name: "Rot Farm Skeleton",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Plant, CreatureType::Skeleton]),
        power: 4,
        toughness: 1,
        keywords: vec![Keyword::CantBlock],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), g()]),
            from_graveyard: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(4),
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gleam of Battle — {4}{R}{W} Enchantment. Whenever a creature you control
/// attacks, put a +1/+1 counter on it.
pub fn gleam_of_battle() -> CardDefinition {
    CardDefinition {
        name: "Gleam of Battle",
        cost: cost(&[generic(4), r(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Debt to the Deathless — {X}{W}{W}{B}{B} Sorcery. Each opponent loses two
/// times X life; you gain life equal to the life lost this way.
pub fn debt_to_the_deathless() -> CardDefinition {
    CardDefinition {
        name: "Debt to the Deathless",
        cost: cost(&[x(), w(), w(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(2))),
        },
        ..Default::default()
    }
}

/// Obzedat's Aid — {3}{W}{B} Sorcery. Return target permanent card from your
/// graveyard to the battlefield.
pub fn obzedats_aid() -> CardDefinition {
    CardDefinition {
        name: "Obzedat's Aid",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(R::PermanentCard.and(R::InYourGraveyard)),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        ..Default::default()
    }
}

/// Drown in Filth — {B}{G} Sorcery. Mill four; target creature gets -1/-1 until
/// end of turn for each land card in your graveyard.
pub fn drown_in_filth() -> CardDefinition {
    let lands_in_gy = || Value::CardsInGraveyardMatching {
        who: PlayerRef::You,
        filter: R::Land,
    };
    CardDefinition {
        name: "Drown in Filth",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::You,
                amount: Value::Const(4),
            },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Diff(Box::new(Value::Const(0)), Box::new(lands_in_gy())),
                toughness: Value::Diff(Box::new(Value::Const(0)), Box::new(lands_in_gy())),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Blast of Genius — {4}{U}{R} Sorcery. Draw three cards, then discard a card.
/// Deals damage equal to the discarded card's mana value to any target.
pub fn blast_of_genius() -> CardDefinition {
    CardDefinition {
        name: "Blast of Genius",
        cost: cost(&[generic(4), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
            },
            Effect::DealDamage {
                to: target_any(),
                amount: Value::GreatestDiscardedManaValueThisEffect,
            },
        ]),
        ..Default::default()
    }
}

/// Pyrewild Shaman — {2}{R} 3/1. Bloodrush — {1}{R}, Discard this card: target
/// attacking creature gets +3/+1. Whenever one or more creatures you control
/// deal combat damage to a player, if this is in your graveyard, you may pay
/// {3} to return it to your hand.
pub fn pyrewild_shaman() -> CardDefinition {
    CardDefinition {
        name: "Pyrewild Shaman",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Goblin, CreatureType::Shaman]),
        power: 3,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                power: Value::Const(3),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::FromYourGraveyard,
            ),
            effect: Effect::MayPay {
                description: "pay {3}: return Pyrewild Shaman to your hand".into(),
                mana_cost: cost(&[generic(3)]),
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

/// Maze's End — Land. Enters tapped. {T}: Add {C}. {3}, {T}, Return to hand:
/// search for a Gate, put it onto the battlefield; if you control ten or more
/// Gates with different names, you win the game.
pub fn mazes_end() -> CardDefinition {
    CardDefinition {
        name: "Maze's End",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                return_self_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: R::HasLandType(crate::card::LandType::Gate),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(
                            Value::DistinctlyNamedGatesControlled,
                            Value::Const(10),
                        ),
                        then: Box::new(Effect::WinGame {
                            who: PlayerRef::You,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Progenitor Mimic — {4}{G}{U} 0/0 Shapeshifter. Enters as a copy of any
/// creature on the battlefield, except at the beginning of your upkeep, if it
/// isn't a token, create a token that's a copy of it.
pub fn progenitor_mimic() -> CardDefinition {
    CardDefinition {
        name: "Progenitor Mimic",
        cost: cost(&[generic(4), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Shapeshifter]),
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_triggered: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::NotToken,
                }),
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}
