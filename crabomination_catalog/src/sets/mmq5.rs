//! Mercadian Masques (MMQ) gap closure, fifth wave. Tests in
//! `classic_sets/mmq5`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CreatureType, DynamicPt,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest, shortcut::target_filtered};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..enchantment(name, c)
    }
}

fn artifact(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

/// Statecraft — {3}{U}. Combat damage to and from your creatures is sealed off.
pub fn statecraft() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to and dealt by creatures you control.",
            effect: StaticEffect::PreventAllCombatDamageToAndFromYourCreatures,
        }],
        ..enchantment("Statecraft", cost(&[generic(3), u()]))
    }
}

/// Insubordination — {B}{B} Aura that bites its host's controller for 2 each
/// end step unless that creature attacked.
pub fn insubordination() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::All(vec![
                    Predicate::IsTurnOf(PlayerRef::ControllerOf(Box::new(host()))),
                    Predicate::Not(Box::new(Predicate::EntityMatches {
                        what: host(),
                        filter: R::AttackedThisTurn,
                    })),
                ])),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(host()))),
                amount: Value::Const(2),
            },
        }],
        ..aura("Insubordination", cost(&[b(), b()]))
    }
}

/// Barbed Wire — {3}. Pings each player on their upkeep; {2} buys off a point
/// of its own damage.
pub fn barbed_wire() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PreventNextDamageFromSourceThisTurn { amount: Value::ONE },
            ..Default::default()
        }],
        ..artifact("Barbed Wire", cost(&[generic(3)]))
    }
}

/// Battle Squadron — {3}{R}{R} flying `*`/`*` sized by your creature count.
pub fn battle_squadron() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some(DynamicPt::CreaturesControlled { base: 0 }),
        ..creature(
            "Battle Squadron",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Goblin],
            0,
            0,
        )
    }
}

/// Bribery — {3}{U}{U}. Steal a creature straight out of an opponent's library.
pub fn bribery() -> CardDefinition {
    sorcery(
        "Bribery",
        cost(&[generic(3), u(), u()]),
        Effect::SearchPickedBy {
            who: PlayerRef::Target(0),
            picker: PlayerRef::You,
            filter: R::Creature,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Extortion — {3}{B}{B}. Look at a hand and strip two cards from it.
pub fn extortion() -> CardDefinition {
    sorcery(
        "Extortion",
        cost(&[generic(3), b(), b()]),
        Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::Target(0)),
            count: Value::Const(2),
            filter: R::Any,
        },
    )
}

/// Renounce — {1}{W}. Sacrifice as much as you like for 2 life apiece.
pub fn renounce() -> CardDefinition {
    instant(
        "Renounce",
        cost(&[generic(1), w()]),
        Effect::SacrificeAnyNumber {
            who: PlayerRef::You,
            filter: R::Any,
            per_each: Box::new(Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            }),
        },
    )
}

/// Invigorate — {2}{G}. Free if you have a Forest and let an opponent gain 3.
pub fn invigorate() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            opponent_gains_life: 3,
            condition: Some(Predicate::SelectorExists(Selector::EachPermanent(
                R::ControlledByYou.and(R::HasLandType(crate::card::LandType::Forest)),
            ))),
            ..Default::default()
        }),
        ..instant(
            "Invigorate",
            cost(&[generic(2), g()]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Orim's Cure — {1}{W}. Tap a creature instead of paying; shield 4 damage.
pub fn orims_cure() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            tap_creatures: Some((R::Creature, 1)),
            condition: Some(Predicate::SelectorExists(Selector::EachPermanent(
                R::ControlledByYou.and(R::HasLandType(crate::card::LandType::Plains)),
            ))),
            ..Default::default()
        }),
        ..instant(
            "Orim's Cure",
            cost(&[generic(1), w()]),
            Effect::PreventNextDamage {
                target: crate::effect::shortcut::target_any(),
                amount: Value::Const(4),
            },
        )
    }
}

/// Ramosian Rally — {3}{W}. The team pumps; tapping a creature can pay for it.
pub fn ramosian_rally() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            tap_creatures: Some((R::Creature, 1)),
            condition: Some(Predicate::SelectorExists(Selector::EachPermanent(
                R::ControlledByYou.and(R::HasLandType(crate::card::LandType::Plains)),
            ))),
            ..Default::default()
        }),
        ..instant(
            "Ramosian Rally",
            cost(&[generic(3), w()]),
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Ferocity — {1}{G} Aura that grows its host every time it meets a blocker.
pub fn ferocity() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Put a +1/+1 counter on the enchanted creature?".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: crate::card::CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
            }],
            ..Default::default()
        }),
        ..aura("Ferocity", cost(&[generic(1), g()]))
    }
}

/// Volcanic Wind — {4}{R}{R}. Damage equal to the board's creature count,
/// divided as you choose.
pub fn volcanic_wind() -> CardDefinition {
    sorcery(
        "Volcanic Wind",
        cost(&[generic(4), r(), r()]),
        Effect::DealDamageDivided {
            total: Value::CountOf(Box::new(Selector::EachPermanent(R::Creature))),
            filter: R::Creature,
            max_targets: 6,
            retaliate_to_source: false,
        },
    )
}

/// Puppet's Verdict — {1}{R}{R}. A coin flip decides which half of the board
/// dies.
pub fn puppets_verdict() -> CardDefinition {
    instant(
        "Puppet's Verdict",
        cost(&[generic(1), r(), r()]),
        Effect::FlipCoin {
            count: Value::ONE,
            on_heads: Box::new(Effect::Destroy {
                what: Selector::EachPermanent(R::Creature.and(R::PowerAtMost(2))),
            }),
            on_tails: Box::new(Effect::Destroy {
                what: Selector::EachPermanent(R::Creature.and(R::PowerAtLeast(3))),
            }),
        },
    )
}

/// Nether Spirit — {1}{B}{B}. Crawls back each upkeep while it's alone in the
/// graveyard.
pub fn nether_spirit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::FromYourGraveyard,
            )
            .with_filter(Predicate::ValueAtMost(
                Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: R::Creature,
                },
                Value::ONE,
            )),
            effect: Effect::MayDo {
                description: "Return Nether Spirit to the battlefield?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..creature(
            "Nether Spirit",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Spirit],
            2,
            2,
        )
    }
}

/// Aerial Caravan — {4}{U}{U} 4/3 flier that impulse-draws for {1}{U}{U}.
pub fn aerial_caravan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u(), u()]),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..creature(
            "Aerial Caravan",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            3,
        )
    }
}

/// Saprazzan Bailiff — {3}{U}{U} 2/2 that jails every graveyard artifact and
/// enchantment, then hands them back when it leaves.
pub fn saprazzan_bailiff() -> CardDefinition {
    let filter = || R::Artifact.or(R::Enchantment);
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::ExileAllGraveyards {
                    filter: Some(filter()),
                    opponents_only: false,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::ReturnGraveyardCardsToHand {
                    filter: filter(),
                    max: Value::Const(99),
                },
            },
        ],
        ..creature(
            "Saprazzan Bailiff",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Merfolk],
            2,
            2,
        )
    }
}

/// Karn's Touch — {U}{U}. A noncreature artifact stands up as a creature sized
/// by its mana value.
pub fn karns_touch() -> CardDefinition {
    instant(
        "Karn's Touch",
        cost(&[u(), u()]),
        Effect::BecomeCreature {
            what: target_filtered(R::Artifact.and(R::Not(Box::new(R::Creature)))),
            power: Value::ManaValueOf(Box::new(Selector::Target(0))),
            toughness: Value::ManaValueOf(Box::new(Selector::Target(0))),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::EndOfTurn,
        },
    )
}

/// Toymaker — {2} 1/1 Spellshaper that animates an artifact from hand.
pub fn toymaker() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::BecomeCreature {
                what: target_filtered(R::Artifact.and(R::Not(Box::new(R::Creature)))),
                power: Value::ManaValueOf(Box::new(Selector::Target(0))),
                toughness: Value::ManaValueOf(Box::new(Selector::Target(0))),
                creature_types: vec![],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Toymaker", cost(&[generic(2)]), vec![CreatureType::Spellshaper], 1, 1)
    }
}

/// Indentured Djinn — {1}{U}{U} 4/4 flier that pays the table three cards.
pub fn indentured_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
        }],
        ..creature(
            "Indentured Djinn",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Djinn],
            4,
            4,
        )
    }
}

/// Hired Giant — {3}{R} 4/4 that ramps everyone else a land.
pub fn hired_giant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::EachOpponent,
                filter: R::Land,
                to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
            },
        }],
        ..creature("Hired Giant", cost(&[generic(3), r()]), vec![CreatureType::Giant], 4, 4)
    }
}

/// Megatherium — {2}{G} 4/4 trample that costs {1} per card in hand to keep.
pub fn megatherium() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::SacrificeSourceUnlessPayValue {
                generic: Value::HandSizeOf(PlayerRef::You),
            },
        }],
        ..creature("Megatherium", cost(&[generic(2), g()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Extravagant Spirit — {3}{U} 4/4 flier that bills you each upkeep for your
/// hand size.
pub fn extravagant_spirit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::SacrificeSourceUnlessPayValue {
                generic: Value::HandSizeOf(PlayerRef::You),
            },
        }],
        ..creature(
            "Extravagant Spirit",
            cost(&[generic(3), u()]),
            vec![CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Common Cause — {2}{W}. A monochrome board of nonartifact creatures gets
/// +2/+2.
pub fn common_cause() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Nonartifact creatures get +2/+2 as long as they all share a color.",
            effect: StaticEffect::AnthemForFilterIf {
                filter: R::Creature.and(R::Not(Box::new(R::Artifact))),
                power: 2,
                toughness: 2,
                keywords: vec![],
                condition: Predicate::AllMatchingShareAColor(
                    R::Creature.and(R::Not(Box::new(R::Artifact))),
                ),
                all_players: true,
            },
        }],
        ..enchantment("Common Cause", cost(&[generic(2), w()]))
    }
}

/// Crumbling Sanctuary — {5}. Damage to players mills exile instead.
pub fn crumbling_sanctuary() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to a player, that player exiles that many cards from the top of their library instead.",
            effect: StaticEffect::PlayerDamageBecomesExileFromLibrary,
        }],
        ..artifact("Crumbling Sanctuary", cost(&[generic(5)]))
    }
}

/// Instigator — {1}{B} 1/1 Spellshaper that forces a player's board to attack.
pub fn instigator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), b()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                keyword: Keyword::MustAttack,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Instigator",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// War Tax — {2}{U}. Buy your opponents out of attacking for a turn.
pub fn war_tax() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), u()]),
            effect: Effect::AddAttackTaxThisTurn { amount: Value::XFromCost },
            ..Default::default()
        }],
        ..enchantment("War Tax", cost(&[generic(2), u()]))
    }
}

/// War Cadence — {2}{R}. The same toll, charged to blockers.
pub fn war_cadence() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), r()]),
            effect: Effect::AddBlockTaxThisTurn { amount: Value::XFromCost },
            ..Default::default()
        }],
        ..enchantment("War Cadence", cost(&[generic(2), r()]))
    }
}

/// Foster — {2}{G}{G}. Every creature death can dig you a replacement.
pub fn foster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1} to dig for a creature card?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: R::Creature,
                    to: ZoneDest::Hand(PlayerRef::You),
                    cap: Value::Const(99),
                    life_per_revealed: 0,
                    miss_dest: crate::effect::RevealMissDest::Graveyard,
                }),
                else_: None,
            },
        }],
        ..enchantment("Foster", cost(&[generic(2), g(), g()]))
    }
}

/// Horn of Plenty — {6}. Every spell can be turned into a card, for {1}.
pub fn horn_of_plenty() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::MayPay {
                description: "Pay {1} to draw a card at the beginning of the next end step?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::DelayUntil {
                    kind: crate::effect::DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Draw {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::ONE,
                    }),
                }),
                else_: None,
            },
        }],
        ..artifact("Horn of Plenty", cost(&[generic(6)]))
    }
}

/// Monkey Cage — {5}. The next creature to enter pops it open.
pub fn monkey_cage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::Seq(vec![
                Effect::SacrificeSource,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                    definition: crate::card::TokenDefinition {
                        name: "Monkey".into(),
                        power: 2,
                        toughness: 2,
                        colors: vec![Color::Green],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Monkey],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
            ]),
        }],
        ..artifact("Monkey Cage", cost(&[generic(5)]))
    }
}

/// Credit Voucher — {2}. Trade any part of your hand in for fresh cards.
pub fn credit_voucher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::ShuffleAnyNumberFromHandThenDraw { who: PlayerRef::You },
            ..Default::default()
        }],
        ..artifact("Credit Voucher", cost(&[generic(2)]))
    }
}

/// Assembly Hall — {5}. Tutors a second copy of a creature you're holding.
pub fn assembly_hall() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::SearchSameNameAs {
                who: PlayerRef::You,
                subject: Selector::ChosenCardInHand(R::Creature),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..artifact("Assembly Hall", cost(&[generic(5)]))
    }
}

/// Clear the Land — {2}{G}. Everyone digs five for lands and burns the rest.
pub fn clear_the_land() -> CardDefinition {
    sorcery(
        "Clear the Land",
        cost(&[generic(2), g()]),
        Effect::EachPlayerRevealTopNKeepLandsExileRest { count: Value::Const(5) },
    )
}

/// Unmask — {3}{B}. Free if you pitch a black card; strips their best nonland.
pub fn unmask() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            exile_filter: Some(R::HasColor(Color::Black)),
            ..Default::default()
        }),
        ..sorcery(
            "Unmask",
            cost(&[generic(3), b()]),
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: R::Not(Box::new(R::Land)),
            },
        )
    }
}

/// Unnatural Hunger — {3}{B}{B} Aura. Each upkeep the host's controller feeds
/// it a creature or takes the host's power in damage.
pub fn unnatural_hunger() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::IsTurnOf(PlayerRef::ControllerOf(Box::new(host())))),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::ControllerOf(Box::new(host())),
                cost: crate::card::WardCost::SacrificeMatching(Box::new(
                    R::Creature.and(R::OtherThanSource),
                )),
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(host()))),
                    amount: Value::PowerOf(Box::new(host())),
                }),
            },
        }],
        ..aura("Unnatural Hunger", cost(&[generic(3), b(), b()]))
    }
}

/// Blood Hound — {2}{R} 1/1 that swells with every point of damage you take.
pub fn blood_hound() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PlayerDamaged, EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Put that many +1/+1 counters on Blood Hound?".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: crate::card::CounterType::PlusOnePlusOne,
                        amount: Value::TriggerEventAmount,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::RemoveCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::Const(99),
                },
            },
        ],
        ..creature("Blood Hound", cost(&[generic(2), r()]), vec![CreatureType::Dog], 1, 1)
    }
}

/// Lava Runner — {1}{R}{R} 2/2 haste. Pointing anything at it costs a land.
pub fn lava_runner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                count: Value::ONE,
                filter: R::Land,
            },
        }],
        ..creature("Lava Runner", cost(&[generic(1), r(), r()]), vec![CreatureType::Lizard], 2, 2)
    }
}

/// Mercadia's Downfall — {2}{R}. The city's own streets arm the attackers.
pub fn mercadias_downfall() -> CardDefinition {
    instant(
        "Mercadia's Downfall",
        cost(&[generic(2), r()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::IsAttacking),
            // "Defending player" reads as the opponent — exact heads-up.
            power: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(R::Land.and(R::ControlledByOpponent))),
                filter: R::Not(Box::new(R::IsBasicLand)),
            },
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Erithizon — {2}{G}{G} 4/4 that hands the defender's board a counter.
pub fn erithizon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Erithizon", cost(&[generic(2), g(), g()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Deepwood Elder — {G}{G} 2/2 Spellshaper that turns lands into Forests.
pub fn deepwood_elder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), g(), g()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 1,
                    filter: R::Land,
                    effect: Box::new(Effect::BecomeBasicLand {
                        what: Selector::Target(0),
                        land_type: crate::card::LandType::Forest,
                        duration: Duration::EndOfTurn,
                    }),
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Deepwood Elder",
            cost(&[g(), g()]),
            vec![CreatureType::Dryad, CreatureType::Spellshaper],
            2,
            2,
        )
    }
}

/// Cowardice — {3}{U}{U}. Point anything at a creature and it flees home.
pub fn cowardice() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..enchantment("Cowardice", cost(&[generic(3), u(), u()]))
    }
}

/// Crag Saurian — {R}{R}{R} 4/4 that defects to whoever damages it.
pub fn crag_saurian() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::GainControl {
                what: Selector::This,
                to: Some(PlayerRef::LastDamagerControllerOf(Box::new(Selector::This))),
                duration: Duration::Permanent,
            },
        }],
        ..creature("Crag Saurian", cost(&[r(), r(), r()]), vec![CreatureType::Lizard], 4, 4)
    }
}


/// Diplomatic Escort — {1}{U} 1/1 Spellshaper that swats anything aimed at a
/// creature.
pub fn diplomatic_escort() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::CounterSpell { what: target_filtered(R::SpellTargetsCreature) },
            ..Default::default()
        }],
        ..creature(
            "Diplomatic Escort",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Jeweled Torque — {2}. Names a colour and sells you 2 life per spell of it.
pub fn jeweled_torque() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::ChooseColorForSelf,
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                    .with_filter(Predicate::CastSpellSharesChosenColorOfSource),
                effect: Effect::MayPay {
                    description: "Pay {2} to gain 2 life?".into(),
                    mana_cost: cost(&[generic(2)]),
                    body: Box::new(Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(2),
                    }),
                    else_: None,
                },
            },
        ],
        ..artifact("Jeweled Torque", cost(&[generic(2)]))
    }
}

/// Ley Line — {3}{G}. Every upkeep hands the active player a counter to place.
pub fn ley_line() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on target creature?".into(),
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..enchantment("Ley Line", cost(&[generic(3), g()]))
    }
}
