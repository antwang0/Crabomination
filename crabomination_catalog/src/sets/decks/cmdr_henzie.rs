//! Commander: the cards the **Riveteers Rampage** precon (NCC, Henzie
//! "Toolbox" Torre) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_henzie.rs`.
//!
//! Residuals (each also on its card):
//! - **Henzie** — a spell with its own blitz uses its printed blitz cost, not
//!   a choice between that and Henzie's.
//! - **First Responder** — the returned creature is targeted, not chosen.
//! - **Mezzio Mugger** — the exiled cards may be cast with mana of any type.
//! - **Next of Kin** — the creature card comes from your hand only (not the
//!   command zone).
//! - **Protection Racket** — every opponent is offered each revealed card,
//!   not only the one whose pass of the process it is.
//! - **The Beamtown Bullies** — the opponent whose turn it is isn't
//!   targeted.
//! - **Turf War** — the contested lands are chosen, not targeted; the
//!   stolen land is the engine's pick.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{blitz, etb, on_attack, on_dies, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, PlayerRef, Predicate, RevealMissDest, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r};
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

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn your_end_step() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
}

fn plus_counters(what: Selector, amount: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount }
}

fn treasure() -> Arc<TokenDefinition> {
    Arc::new(crabomination_base::tokens::treasure_token())
}

/// Henzie "Toolbox" Torre — {B}{R}{G} 3/3 Devil Rogue. Creature spells with
/// mana value 4 or greater have blitz at their mana cost (CR 702.152); blitz
/// costs {1} less per commander cast from the command zone.
/// Residual: a spell with its own blitz uses its printed one.
pub fn henzie_toolbox_torre() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        can_be_commander: true,
        static_abilities: vec![
            StaticAbility {
                description: "Each creature spell you cast with mana value 4 or greater has blitz.".into(),
                effect: StaticEffect::GrantBlitzToSpells { filter: R::Creature.and(R::ManaValueAtLeast(4)) },
            },
            StaticAbility {
                description: "Blitz costs you pay cost {1} less for each time you've cast your commander.".into(),
                effect: StaticEffect::BlitzCostLessPerCommanderCast,
            },
        ],
        ..creature(
            "Henzie \"Toolbox\" Torre",
            cost(&[b(), r(), g()]),
            vec![CreatureType::Devil, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Bellowing Mauler — {4}{B} 4/6. At your end step each player loses 4 life
/// unless they sacrifice a nontoken creature.
pub fn bellowing_mauler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_end_step(),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::EachPlayer,
                body: Box::new(Effect::UnlessPlayerPays {
                    who: PlayerRef::You,
                    cost: WardCost::SacrificeMatchingN(Box::new(R::Creature.and(R::NotToken)), 1),
                    then: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(4) }),
                    if_paid: None,
                }),
            },
        }],
        ..creature(
            "Bellowing Mauler",
            cost(&[generic(4), b()]),
            vec![CreatureType::Ogre, CreatureType::Warrior],
            4,
            6,
        )
    }
}

/// Caldaia Guardian — {3}{G} 4/3. It or another creature you control with
/// mana value 4 or greater dying makes two 1/1 Citizens. Blitz {2}{G}.
pub fn caldaia_guardian() -> CardDefinition {
    let citizen = TokenDefinition {
        name: "Citizen".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Citizen], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        alternative_cost: Some(blitz(cost(&[generic(2), g()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::ManaValueAtLeast(4) },
            ),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(citizen) },
        }],
        ..creature(
            "Caldaia Guardian",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            3,
        )
    }
}

/// Dodgy Jalopy — {2}{G} */5 Vehicle: power is the greatest mana value
/// among creatures you control. Trample, crew 3, scavenge {2}{G}.
pub fn dodgy_jalopy() -> CardDefinition {
    let greatest = Value::ManaValueOf(Box::new(Selector::GreatestManaValueControlledMatching {
        who: PlayerRef::You,
        filter: R::Creature,
    }));
    CardDefinition {
        name: "Dodgy Jalopy",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 0,
        toughness: 5,
        keywords: vec![Keyword::Trample, Keyword::Crew(3)],
        static_abilities: vec![StaticAbility {
            description: "Its power is equal to the greatest mana value among creatures you control.".into(),
            effect: StaticEffect::SelfBasePtFromValue { power: greatest.clone(), toughness: Value::Const(5) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: plus_counters(target_filtered(R::Creature), greatest),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Evolutionary Leap — {1}{G} enchantment. {G}, sacrifice a creature: reveal
/// until a creature card, take it, the rest on the bottom in a random order.
pub fn evolutionary_leap() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: R::Creature,
                to: ZoneDest::Hand(PlayerRef::You),
                cap: Value::Const(1000),
                life_per_revealed: 0,
                miss_dest: RevealMissDest::BottomRandom,
            },
            ..Default::default()
        }],
        ..enchantment("Evolutionary Leap", cost(&[generic(1), g()]))
    }
}

/// First Responder — {3}{G} 3/3 vigilance. At your end step you may return
/// another creature you control to hand, then it gets +1/+1 counters equal
/// to that creature's power (read before it leaves — the same number).
/// Residual: the returned creature is targeted, not chosen.
pub fn first_responder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: your_end_step(),
            effect: Effect::MayDo {
                description: "Return another creature you control to its owner's hand?".into(),
                body: Box::new(Effect::Seq(vec![
                    // Its power is read while it is still on the battlefield;
                    // the Move below declares the slot.
                    plus_counters(Selector::This, Value::PowerOf(Box::new(Selector::Target(0)))),
                    Effect::Move {
                        what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    },
                ])),
            },
        }],
        ..creature(
            "First Responder",
            cost(&[generic(3), g()]),
            vec![CreatureType::Ogre, CreatureType::Citizen],
            3,
            3,
        )
    }
}

/// Grime Gorger — {2}{B}{G} 3/3 menace. Attacking, it exiles up to one card
/// of each card type from the defending player's graveyard and grows by one
/// counter per card.
pub fn grime_gorger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::ExileOnePerCardTypeFromGraveyardGrow {
            who: PlayerRef::DefendingPlayer,
        })],
        ..creature("Grime Gorger", cost(&[generic(2), b(), g()]), vec![CreatureType::Horror], 3, 3)
    }
}

/// Industrial Advancement — {3}{R} enchantment. At your end step you may
/// sacrifice a creature; if you do, look at the top X (its mana value) and
/// you may put a creature card from among them onto the battlefield, the
/// rest on the bottom in a random order.
pub fn industrial_advancement() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_end_step(),
            effect: Effect::MayDo {
                description: "Sacrifice a creature?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::SacrificeAndRemember { who: PlayerRef::You, filter: R::Creature },
                    Effect::LookTopPutMatchingOntoBattlefield {
                        count: Value::SacrificedManaValue,
                        filter: R::Creature,
                        then: None,
                        max: Some(1),
                        tapped: false,
                        exile_rest: false,
                        rest_to_graveyard: false,
                    },
                ])),
            },
        }],
        ..enchantment("Industrial Advancement", cost(&[generic(3), r()]))
    }
}

/// Jolene, the Plunder Queen — {2}{R}{G} 2/2. A player attacking one of your
/// opponents makes a Treasure; your Treasure creations make one more;
/// sacrifice five Treasures: five +1/+1 counters.
pub fn jolene_the_plunder_queen() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::OpponentOfYoursAttacked),
            effect: Effect::CreateToken { who: PlayerRef::Target(0), count: Value::ONE, definition: treasure() },
        }],
        static_abilities: vec![StaticAbility {
            description: "If you would create Treasure tokens, create those plus an additional Treasure.".into(),
            effect: StaticEffect::TreasureCreationAddsTreasure,
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Treasure), 5)),
            effect: plus_counters(Selector::This, Value::Const(5)),
            ..Default::default()
        }],
        ..creature(
            "Jolene, the Plunder Queen",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Kresh the Bloodbraided — {2}{B}{R}{G} 3/3. Another creature dying, you
/// may put its power (last known, CR 603.10) in +1/+1 counters on Kresh.
pub fn kresh_the_bloodbraided() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        can_be_commander: true,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::Not(
                Box::new(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSource }),
            )),
            effect: Effect::MayDo {
                description: "Put +1/+1 counters equal to its power on Kresh?".into(),
                body: Box::new(plus_counters(Selector::This, Value::PowerOf(Box::new(Selector::TriggerSource)))),
            },
        }],
        ..creature(
            "Kresh the Bloodbraided",
            cost(&[generic(2), b(), r(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Mezzio Mugger — {4}{R} 3/3 Lizard Rogue. Attacking, it exiles the top
/// card of each player's library, playable this turn. Blitz {2}{R}.
/// Residual: those cards may be cast with mana of any type.
pub fn mezzio_mugger() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(blitz(cost(&[generic(2), r()]))),
        triggered_abilities: vec![on_attack(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::EachPlayer,
            count: Value::ONE,
            duration: crate::card::MayPlayDuration::EndOfThisTurn,
            pay_any_color: true,
            max_mana_value: None,
            pay_own_cost: false,
            uncast_penalty: None,
        })],
        ..creature(
            "Mezzio Mugger",
            cost(&[generic(4), r()]),
            vec![CreatureType::Lizard, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Next of Kin — {2}{G} Aura. Enchanted creature dying, you may put a
/// creature card with lesser mana value from your hand onto the battlefield;
/// if you do, this returns attached to it at the next end step.
/// Residual: the command zone isn't offered.
pub fn next_of_kin() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature.and(R::ManaValueLessThanEventAmount),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: Some(Box::new(Effect::DelayUntilWithCapture {
                    kind: DelayedTriggerKind::NextEndStep,
                    capture: Selector::LastMoved,
                    body: Box::new(Effect::ReturnSelfAttachedToTarget),
                })),
            },
        }],
        ..enchantment("Next of Kin", cost(&[generic(2), g()]))
    }
}

/// Protection Racket — {2}{B} enchantment. At your upkeep, for each opponent:
/// reveal your top card; they may pay life equal to its mana value to exile
/// it, else it goes to your hand.
/// Residual: every opponent is offered each card.
pub fn protection_racket() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::ForEachOpponent {
                body: Box::new(Effect::RevealTopPayOrTake {
                    count: Value::ONE,
                    life: Value::ManaValueOf(Box::new(Selector::TopOfLibrary {
                        who: PlayerRef::You,
                        count: Value::ONE,
                    })),
                }),
            },
        }],
        ..enchantment("Protection Racket", cost(&[generic(2), b()]))
    }
}

/// Riveteers Confluence — {2}{B}{R}{G} sorcery; choose three, repeats
/// allowed (CR 700.2d): draw and lose 1 life; 1 damage to each creature and
/// planeswalker you don't control; a land from hand or graveyard onto the
/// battlefield tapped.
pub fn riveteers_confluence() -> CardDefinition {
    CardDefinition {
        name: "Riveteers Confluence",
        cost: cost(&[generic(2), b(), r(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![0, 1, 2],
            modes: vec![
                Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                ]),
                Effect::DealDamage {
                    to: Selector::EachPermanent(
                        R::Creature.or(R::Planeswalker).and(R::ControlledByYou.negate()),
                    ),
                    amount: Value::ONE,
                },
                Effect::Seq(vec![
                    Effect::PutFromHandOrGraveyardOntoBattlefield { filter: R::Land },
                    Effect::Tap { what: Selector::LastMoved },
                ]),
            ],
        },
        ..Default::default()
    }
}

/// The Beamtown Bullies — {1}{B}{R}{G} 4/4 vigilance, haste. {T}: the
/// opponent whose turn it is puts a nonlegendary creature card from your
/// graveyard onto the battlefield under their control; it gains haste, is
/// goaded (CR 701.15), and is exiled at the next end step.
/// Residual: that opponent isn't targeted.
pub fn the_beamtown_bullies() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        R::Creature.and(R::HasSupertype(Supertype::Legendary).negate()).and(R::InYourGraveyard),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::ActivePlayer, tapped: false },
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                Effect::Goad { what: Selector::Target(0) },
                Effect::DelayUntilWithCapture {
                    kind: DelayedTriggerKind::NextEndStep,
                    capture: Selector::Target(0),
                    body: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "The Beamtown Bullies",
            cost(&[generic(1), b(), r(), g()]),
            vec![CreatureType::Ogre, CreatureType::Devil, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Turf War — {4}{R} enchantment. Entering, a contested counter on a land
/// each player controls; a creature dealing combat damage to a player takes
/// one of their contested lands and untaps it.
/// Residual: the lands are chosen, not targeted.
pub fn turf_war() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::ContestOneLandPerPlayer),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer),
                effect: Effect::TakeContestedLand,
            },
        ],
        ..enchantment("Turf War", cost(&[generic(4), r()]))
    }
}

/// Wave of Rats — {3}{B} 4/2 trample. Dying after it dealt combat damage to
/// a player this turn, it returns. Blitz {4}{B}.
pub fn wave_of_rats() -> CardDefinition {
    let mut dies = on_dies(Effect::Move {
        what: Selector::This,
        to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOf(Box::new(Selector::This)), tapped: false },
    });
    dies.event = dies
        .event
        .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::DamagedAPlayerThisTurn });
    CardDefinition {
        keywords: vec![Keyword::Trample],
        alternative_cost: Some(blitz(cost(&[generic(4), b()]))),
        triggered_abilities: vec![dies],
        ..creature("Wave of Rats", cost(&[generic(3), b()]), vec![CreatureType::Rat], 4, 2)
    }
}

/// Weathered Sentinels — {3} 2/5 artifact Wall. Defender, reach, vigilance,
/// trample; it can attack players who attacked you during their last turn
/// (CR 508.1a); attacking, +3/+3 and indestructible until end of turn.
pub fn weathered_sentinels() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender, Keyword::Reach, Keyword::Vigilance, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Can attack players who attacked you during their last turn.".into(),
            effect: StaticEffect::CanAttackPlayersWhoAttackedYouLastTurn,
        }],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
        ]))],
        ..creature("Weathered Sentinels", cost(&[generic(3)]), vec![CreatureType::Wall], 2, 5)
    }
}

/// World Shaper — {3}{G} 3/3 Merfolk Shaman. Attacking, you may mill three;
/// dying, every land card in your graveyard returns tapped.
pub fn world_shaper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::MayDo {
                description: "Mill three cards?".into(),
                body: Box::new(Effect::Mill { who: Selector::You, amount: Value::Const(3) }),
            }),
            on_dies(Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::Land,
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
        ],
        ..creature(
            "World Shaper",
            cost(&[generic(3), g()]),
            vec![CreatureType::Merfolk, CreatureType::Shaman],
            3,
            3,
        )
    }
}
