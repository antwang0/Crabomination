//! Torment (TOR) — the closing wave: the Dreams cycle, the Possessed cycle,
//! the Nightmare Horrors and the Threshold commons. Tests in
//! `classic_sets/tor2`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, ExileReturnZone, Keyword, LandType, Predicate,
    SelectionRequirement as R, StateTriggeredAbility, StaticAbility, Subtypes, TriggeredAbility,
    Zone,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, StaticEffect, Value,
    ZoneDest,
    shortcut::{draw, etb, target_any, target_filtered},
};
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

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        ..enchantment(name, c)
    }
}

fn threshold() -> Predicate {
    Predicate::ThresholdActive { who: PlayerRef::You }
}

/// "Threshold — as long as there are seven or more cards in your graveyard,
/// this creature has [ability]."
fn threshold_grant(description: &'static str, ability: TriggeredAbility) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::WhileCondition {
            condition: threshold(),
            inner: Box::new(StaticEffect::GrantTriggeredAbility {
                filter: R::IsSource,
                ability: Box::new(ability),
            }),
        },
    }
}

fn upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(crate::game::TurnStep::Upkeep),
            EventScope::YourControl,
        ),
        effect,
    }
}

/// "When this leaves the battlefield, …" — the Nightmare Horrors' give-back.
fn leaves(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
        effect,
    }
}

// ── The Dreams cycle ────────────────────────────────────────────────────────

/// The five "discard X cards" spells share one additional cost.
fn dreams(name: &'static str, c: ManaCost, sorcery_speed: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::DiscardXFromCost],
        ..if sorcery_speed { sorcery(name, c, effect) } else { instant(name, c, effect) }
    }
}

/// Restless Dreams — {B}. Discard X, buy back X creature cards.
pub fn restless_dreams() -> CardDefinition {
    dreams(
        "Restless Dreams",
        cost(&[x(), b()]),
        true,
        Effect::MoveChosen {
            from: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::Creature,
            },
            filter: None,
            count: Value::XFromCost,
            up_to: true,
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Nostalgic Dreams — {G}{G}. Discard X, buy back X cards, then exile itself.
pub fn nostalgic_dreams() -> CardDefinition {
    dreams(
        "Nostalgic Dreams",
        cost(&[x(), g(), g()]),
        true,
        Effect::Seq(vec![
            Effect::MoveChosen {
                from: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::Any,
                },
                filter: None,
                count: Value::XFromCost,
                up_to: true,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Move { what: Selector::This, to: ZoneDest::Exile },
        ]),
    )
}

/// Turbulent Dreams — {U}{U}. Discard X, bounce X nonland permanents.
pub fn turbulent_dreams() -> CardDefinition {
    dreams(
        "Turbulent Dreams",
        cost(&[x(), u(), u()]),
        true,
        Effect::MoveChosen {
            from: Selector::EachPermanent(R::Nonland),
            filter: None,
            count: Value::XFromCost,
            up_to: true,
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Vengeful Dreams — {W}{W}. Discard X, exile X attackers.
pub fn vengeful_dreams() -> CardDefinition {
    dreams(
        "Vengeful Dreams",
        cost(&[x(), w(), w()]),
        false,
        Effect::MoveChosen {
            from: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
            filter: None,
            count: Value::XFromCost,
            up_to: true,
            to: ZoneDest::Exile,
        },
    )
}

/// Insidious Dreams — {3}{B}. Discard X, stack X cards on top.
pub fn insidious_dreams() -> CardDefinition {
    dreams(
        "Insidious Dreams",
        cost(&[generic(3), b()]),
        false,
        Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::Any,
            to: ZoneDest::Library {
                who: PlayerRef::You,
                pos: crate::effect::LibraryPosition::Top,
            },
            count: Value::XFromCost,
        },
    )
}

/// Devastating Dreams — {R}{R}. Discard X at random, then a symmetrical
/// land- and creature-wipe scaled to X.
pub fn devastating_dreams() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::DiscardXRandomFromCost],
        ..sorcery(
            "Devastating Dreams",
            cost(&[x(), r(), r()]),
            Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    count: Value::XFromCost,
                    filter: R::Land,
                },
                Effect::ForEach {
                    selector: Selector::EachPermanent(R::Creature),
                    body: Box::new(Effect::DealDamage {
                        to: Selector::TriggerSource,
                        amount: Value::XFromCost,
                    }),
                },
            ]),
        )
    }
}

// ── The Possessed cycle ─────────────────────────────────────────────────────

/// "Threshold — this creature gets +1/+1, is black, and has
/// '{2}{B}, {T}: Destroy target [color] creature.'"
fn possessed(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    keyword: Keyword,
    prey: Color,
) -> CardDefinition {
    CardDefinition {
        keywords: vec![keyword],
        static_abilities: vec![
            StaticAbility {
                description: "Threshold — +1/+1 and black.",
                effect: StaticEffect::PumpSelfIf {
                    condition: threshold(),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                },
            },
            StaticAbility {
                description: "Threshold — this creature is black.",
                effect: StaticEffect::WhileCondition {
                    condition: threshold(),
                    inner: Box::new(StaticEffect::SetColorOfMatching {
                        applies_to: Selector::This,
                        color: Color::Black,
                    }),
                },
            },
            StaticAbility {
                description: "Threshold — {2}{B}, {T}: Destroy a creature.",
                effect: StaticEffect::WhileCondition {
                    condition: threshold(),
                    inner: Box::new(StaticEffect::GrantActivatedAbility {
                        applies_to: Selector::This,
                        condition: None,
                        ability: ActivatedAbility {
                            mana_cost: cost(&[generic(2), b()]),
                            tap_cost: true,
                            effect: Effect::Destroy {
                                what: target_filtered(R::Creature.and(R::HasColor(prey))),
                            },
                            ..Default::default()
                        },
                    }),
                },
            },
        ],
        ..creature(name, c, types, 3, 3)
    }
}

/// Possessed Aven — {2}{U}{U} 3/3 flier that eats blue past Threshold.
pub fn possessed_aven() -> CardDefinition {
    possessed(
        "Possessed Aven",
        cost(&[generic(2), u(), u()]),
        vec![CreatureType::Bird, CreatureType::Soldier, CreatureType::Horror],
        Keyword::Flying,
        Color::Blue,
    )
}

/// Possessed Barbarian — {2}{R}{R} 3/3 first striker that eats red.
pub fn possessed_barbarian() -> CardDefinition {
    possessed(
        "Possessed Barbarian",
        cost(&[generic(2), r(), r()]),
        vec![CreatureType::Human, CreatureType::Barbarian, CreatureType::Horror],
        Keyword::FirstStrike,
        Color::Red,
    )
}

/// Possessed Centaur — {2}{G}{G} 3/3 trampler that eats green.
pub fn possessed_centaur() -> CardDefinition {
    possessed(
        "Possessed Centaur",
        cost(&[generic(2), g(), g()]),
        vec![CreatureType::Centaur, CreatureType::Horror],
        Keyword::Trample,
        Color::Green,
    )
}

/// Possessed Nomad — {2}{W}{W} 3/3 vigilant that eats white.
pub fn possessed_nomad() -> CardDefinition {
    possessed(
        "Possessed Nomad",
        cost(&[generic(2), w(), w()]),
        vec![CreatureType::Human, CreatureType::Nomad, CreatureType::Horror],
        Keyword::Vigilance,
        Color::White,
    )
}

// ── Nightmare Horrors ───────────────────────────────────────────────────────

/// "When this enters, target player loses N life. When this leaves the
/// battlefield, that player gains N life."
fn life_swing(name: &'static str, c: ManaCost, p: i32, t: i32, amount: i32) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::RememberPlayerOnSource { who: PlayerRef::Target(0) },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(amount),
                },
            ])),
            leaves(Effect::GainLife {
                who: Selector::Player(PlayerRef::ChosenPlayerOfSource),
                amount: Value::Const(amount),
            }),
        ],
        ..creature(name, c, vec![CreatureType::Nightmare, CreatureType::Horror], p, t)
    }
}

/// Soul Scourge — {4}{B} 3/2 flier. Three life, refunded when it leaves.
pub fn soul_scourge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..life_swing("Soul Scourge", cost(&[generic(4), b()]), 3, 2, 3)
    }
}

/// Laquatus's Champion — {4}{B}{B} 6/3. Six life, refunded when it leaves;
/// {B} regenerates it.
pub fn laquatuss_champion() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..life_swing("Laquatus's Champion", cost(&[generic(4), b(), b()]), 6, 3, 6)
    }
}

/// Hypnox — {8}{B}{B}{B} 8/8 flier that holds an opponent's hand hostage.
pub fn hypnox() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::TriggerSourceEnteredByCast),
                effect: Effect::ExileUntilSourceLeaves {
                    what: Selector::CardsInZone {
                        who: PlayerRef::Target(0),
                        zone: Zone::Hand,
                        filter: R::Any,
                    },
                    return_to: ExileReturnZone::Hand,
                },
            },
        ],
        ..creature(
            "Hypnox",
            cost(&[generic(8), b(), b(), b()]),
            vec![CreatureType::Nightmare, CreatureType::Horror],
            8,
            8,
        )
    }
}

/// Petradon — {6}{R}{R} 5/6 that sits on two lands until it leaves.
pub fn petradon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Land,
            effect: Box::new(Effect::ExileUntilSourceLeaves {
                what: Selector::Target(0),
                return_to: ExileReturnZone::Battlefield,
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Petradon",
            cost(&[generic(6), r(), r()]),
            vec![CreatureType::Nightmare, CreatureType::Beast],
            5,
            6,
        )
    }
}

// ── Threshold creatures ─────────────────────────────────────────────────────

/// Seton's Scout — {1}{G} 2/1 reach; 4/3 past Threshold.
pub fn setons_scout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Threshold — +2/+2.",
            effect: StaticEffect::PumpSelfIf {
                condition: threshold(),
                power: 2,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..creature(
            "Seton's Scout",
            cost(&[generic(1), g()]),
            vec![
                CreatureType::Centaur,
                CreatureType::Druid,
                CreatureType::Scout,
                CreatureType::Archer,
            ],
            2,
            1,
        )
    }
}

/// Nantuko Blightcutter — {2}{G} 2/2 pro-black that grows on the opponents'
/// black permanents past Threshold.
pub fn nantuko_blightcutter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        static_abilities: vec![StaticAbility {
            description: "Threshold — +1/+1 per black permanent your opponents control.",
            effect: StaticEffect::WhileCondition {
                condition: threshold(),
                inner: Box::new(StaticEffect::PumpSelfByValue {
                    amount: Value::CountOf(Box::new(Selector::EachPermanent(
                        R::HasColor(Color::Black).and(R::ControlledByOpponent),
                    ))),
                    per_power: 1,
                    per_toughness: 1,
                }),
            },
        }],
        ..creature(
            "Nantuko Blightcutter",
            cost(&[generic(2), g()]),
            vec![CreatureType::Insect, CreatureType::Druid],
            2,
            2,
        )
    }
}

/// Gloomdrifter — {3}{B} 2/2 flier whose arrival shrinks the nonblack board
/// past Threshold.
pub fn gloomdrifter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![threshold_grant(
            "Threshold — when this enters, nonblack creatures get -2/-2.",
            etb(Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Black).negate())),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            }),
        )],
        ..creature(
            "Gloomdrifter",
            cost(&[generic(3), b()]),
            vec![CreatureType::Zombie, CreatureType::Minion],
            2,
            2,
        )
    }
}

/// Pardic Arsonist — {2}{R}{R} 3/3 that arrives with a Lava Spike past
/// Threshold.
pub fn pardic_arsonist() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![threshold_grant(
            "Threshold — when this enters, it deals 3 damage to any target.",
            etb(Effect::DealDamage { to: target_any(), amount: Value::Const(3) }),
        )],
        ..creature(
            "Pardic Arsonist",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Human, CreatureType::Barbarian],
            3,
            3,
        )
    }
}

/// Teroh's Vanguard — {3}{W} 2/3 flash that shields the team past Threshold.
pub fn terohs_vanguard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![threshold_grant(
            "Threshold — when this enters, your creatures gain protection from black.",
            etb(Effect::GrantKeyword {
                what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                keyword: Keyword::Protection(Color::Black),
                duration: Duration::EndOfTurn,
            }),
        )],
        ..creature(
            "Teroh's Vanguard",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Nomad],
            2,
            3,
        )
    }
}

/// Reborn Hero — {2}{W} 2/2 vigilance that buys itself back past Threshold.
pub fn reborn_hero() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![threshold_grant(
            "Threshold — when this dies, you may pay {W}{W} to return it.",
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::MayPay {
                    description: "Pay {W}{W} to return Reborn Hero?".into(),
                    mana_cost: cost(&[w(), w()]),
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    else_: None,
                },
            },
        )],
        ..creature(
            "Reborn Hero",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

// ── Other creatures ─────────────────────────────────────────────────────────

/// Barbarian Outcast — {1}{R} 2/2 that needs a Swamp to stay.
pub fn barbarian_outcast() -> CardDefinition {
    CardDefinition {
        sacrifice_when: Some(Predicate::Not(Box::new(Predicate::SelectorExists(
            Selector::ControlledBy { who: PlayerRef::You, filter: R::HasLandType(LandType::Swamp) },
        )))),
        ..creature(
            "Barbarian Outcast",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Barbarian, CreatureType::Beast],
            2,
            2,
        )
    }
}

/// "Whenever this attacks or blocks, any player may exile N cards from their
/// graveyard. If a player does, this assigns no combat damage this turn."
fn carrion(name: &'static str, c: ManaCost, types: Vec<CreatureType>, p: i32, t: i32, n: i32)
-> CardDefinition {
    CardDefinition {
        triggered_abilities: [EventKind::Attacks, EventKind::Blocks]
            .map(|kind| TriggeredAbility {
                event: EventSpec::new(kind, EventScope::SelfSource),
                effect: Effect::AnyPlayerMayExileFromGraveyard {
                    count: Value::Const(n),
                    then: Box::new(Effect::PreventCombatDamageByTargetThisTurn {
                        target: Selector::This,
                    }),
                },
            })
            .into(),
        ..creature(name, c, types, p, t)
    }
}

/// Carrion Rats — {B} 2/1 that any player can blank for a graveyard card.
pub fn carrion_rats() -> CardDefinition {
    carrion("Carrion Rats", cost(&[b()]), vec![CreatureType::Rat], 2, 1, 1)
}

/// Carrion Wurm — {3}{B}{B} 6/5 blanked for three graveyard cards.
pub fn carrion_wurm() -> CardDefinition {
    carrion(
        "Carrion Wurm",
        cost(&[generic(3), b(), b()]),
        vec![CreatureType::Zombie, CreatureType::Wurm],
        6,
        5,
        3,
    )
}

/// Longhorn Firebeast — {2}{R} 3/2 an opponent can trade 5 life to kill.
pub fn longhorn_firebeast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDoBy {
            who: PlayerRef::EachOpponent,
            description: "Take 5 damage from Longhorn Firebeast to destroy it?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamage { to: Selector::You, amount: Value::Const(5) },
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::This))),
                    count: Value::Const(1),
                    filter: R::IsSource,
                },
            ])),
        })],
        ..creature(
            "Longhorn Firebeast",
            cost(&[generic(2), r()]),
            vec![CreatureType::Elemental, CreatureType::Ox, CreatureType::Beast],
            3,
            2,
        )
    }
}

/// Gurzigost — {3}{G}{G} 6/8 that eats its own graveyard to survive.
pub fn gurzigost() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![upkeep(Effect::SacrificeSourceUnlessCost {
            cost: crate::card::WardCost::BottomFromGraveyard(2),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), g()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::AssignsDamageAsThoughUnblocked,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Gurzigost", cost(&[generic(3), g(), g()]), vec![CreatureType::Beast], 6, 8)
    }
}

/// Nantuko Cultivator — {3}{G} 2/2 that trades excess lands for counters and
/// cards.
pub fn nantuko_cultivator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: Value::CardsInHandMatching { who: PlayerRef::You, filter: R::Land },
                random: false,
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CardsDiscardedThisEffect,
            },
            Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisEffect },
        ]))],
        ..creature(
            "Nantuko Cultivator",
            cost(&[generic(3), g()]),
            vec![CreatureType::Insect, CreatureType::Druid],
            2,
            2,
        )
    }
}

/// Pitchstone Wall — {2}{R} 2/5 Defender that can cash itself in to rebuy a
/// discard.
pub fn pitchstone_wall() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::MaySacrificeSource {
                description: "Sacrifice Pitchstone Wall to return the discarded card?".into(),
                then: Box::new(Effect::Move {
                    what: Selector::LastMoved,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
                else_: None,
            },
        }],
        ..creature("Pitchstone Wall", cost(&[generic(2), r()]), vec![CreatureType::Wall], 2, 5)
    }
}

/// Stern Judge — {2}{W} 2/2 that taxes everyone per Swamp.
pub fn stern_judge() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LoseLifePerControlled {
                who: Selector::Player(PlayerRef::EachPlayer),
                filter: R::HasLandType(LandType::Swamp),
                per: Value::Const(1),
            },
            ..Default::default()
        }],
        ..creature(
            "Stern Judge",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Zombie Trailblazer — {B}{B}{B} 2/2 that taps its friends to make Swamps
/// and swampwalkers.
pub fn zombie_trailblazer() -> CardDefinition {
    let tap_a_zombie = || Some(R::ControlledByYou.and(R::HasCreatureType(CreatureType::Zombie)));
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_other_filter: tap_a_zombie(),
                effect: Effect::BecomeBasicLand {
                    what: target_filtered(R::Land),
                    land_type: LandType::Swamp,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_other_filter: tap_a_zombie(),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Landwalk(LandType::Swamp),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Zombie Trailblazer",
            cost(&[b(), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Scout],
            2,
            2,
        )
    }
}

/// Shambling Swarm — {1}{B}{B}{B} 3/3 whose death spreads three -1/-1
/// counters that wear off at end of turn.
pub fn shambling_swarm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DistributeCounters {
                total: Value::Const(3),
                counter: CounterType::MinusOneMinusOne,
                filter: R::Creature,
                max_targets: 3,
            },
        }],
        ..creature("Shambling Swarm", cost(&[generic(1), b(), b(), b()]), vec![CreatureType::Horror], 3, 3)
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Dawn of the Dead — {2}{B}{B}{B}. A life a turn buys a hasty rental from
/// your graveyard.
pub fn dawn_of_the_dead() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            upkeep(Effect::LoseLife { who: Selector::You, amount: Value::Const(1) }),
            upkeep(Effect::MayDo {
                description: "Reanimate a creature until end of turn?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::MoveChosen {
                        from: Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::Creature,
                        },
                        filter: None,
                        count: Value::Const(1),
                        up_to: false,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::GrantKeyword {
                        what: Selector::LastMoved,
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::AtNextEndStep {
                        body: Box::new(Effect::Move {
                            what: Selector::LastMoved,
                            to: ZoneDest::Exile,
                        }),
                    },
                ])),
            }),
        ],
        ..enchantment("Dawn of the Dead", cost(&[generic(2), b(), b(), b()]))
    }
}

/// Last Laugh — {2}{B}{B}. Every death pings the whole table; it sacrifices
/// itself once the creatures are gone.
pub fn last_laugh() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf))),
            effect: Effect::Seq(vec![
                Effect::ForEach {
                    selector: Selector::EachPermanent(R::Creature),
                    body: Box::new(Effect::DealDamage {
                        to: Selector::TriggerSource,
                        amount: Value::Const(1),
                    }),
                },
                Effect::ForEach {
                    selector: Selector::Player(PlayerRef::EachPlayer),
                    body: Box::new(Effect::DealDamage {
                        to: Selector::TriggerSource,
                        amount: Value::Const(1),
                    }),
                },
            ]),
        }],
        sacrifice_when: Some(Predicate::Not(Box::new(Predicate::SelectorExists(
            Selector::EachPermanent(R::Creature),
        )))),
        ..enchantment("Last Laugh", cost(&[generic(2), b(), b()]))
    }
}

/// Hypochondria — {1}{W}. Two ways to buy a 3-point shield.
pub fn hypochondria() -> CardDefinition {
    let shield = || Effect::PreventNextDamage { target: target_any(), amount: Value::Const(3) };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                discard_cost: Some((R::Any, 1)),
                effect: shield(),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                sac_cost: true,
                effect: shield(),
                ..Default::default()
            },
        ],
        ..enchantment("Hypochondria", cost(&[generic(1), w()]))
    }
}

/// Transcendence — {3}{W}{W}{W}. Life loss becomes life gain, 0 stops being
/// lethal, and 20 becomes lethal instead.
pub fn transcendence() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You don't lose the game for having 0 or less life.",
            effect: StaticEffect::ControllerDoesntLoseFromLife,
        }],
        state_trigger: Some(StateTriggeredAbility {
            condition: Predicate::PlayerLifeAtLeast { who: PlayerRef::You, life: 20 },
            effect: Effect::LoseGame { who: PlayerRef::You },
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeLost, EventScope::YourControl),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Times(
                    Box::new(Value::TriggerEventAmount),
                    Box::new(Value::Const(2)),
                ),
            },
        }],
        ..enchantment("Transcendence", cost(&[generic(3), w(), w(), w()]))
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Stupefying Touch — {1}{U} Aura. A card now, and the host's abilities are
/// switched off.
pub fn stupefying_touch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(draw(1))],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature's activated abilities can't be activated.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::attached_to(Selector::This),
                keyword: Keyword::CantActivateAbilities,
            },
        }],
        ..aura("Stupefying Touch", cost(&[generic(1), u()]), R::Creature)
    }
}

/// Shade's Form — {1}{B}{B} Aura. A Shade pump, and the host comes back
/// under your control when it dies.
pub fn shades_form() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..aura("Shade's Form", cost(&[generic(1), b(), b()]), R::Creature)
    }
}

/// Floating Shield — {2}{W} Aura. A colour of protection for the host, or a
/// one-shot save for anyone.
pub fn floating_shield() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseColorForSelf)],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has protection from the chosen color.",
            effect: StaticEffect::GrantProtectionFromChosenColor {
                applies_to: Selector::attached_to(Selector::This),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantProtectionFromChosenColor {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..aura("Floating Shield", cost(&[generic(2), w()]), R::Creature)
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Kamahl's Sledge — {5}{R}{R}. Four to a creature, and four to its
/// controller past Threshold.
pub fn kamahls_sledge() -> CardDefinition {
    sorcery(
        "Kamahl's Sledge",
        cost(&[generic(5), r(), r()]),
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
            Effect::If {
                cond: threshold(),
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(4),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Violent Eruption — {1}{R}{R}{R}. Four damage spread anywhere, castable
/// off a discard.
pub fn violent_eruption() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Madness(cost(&[generic(1), r(), r()]))],
        ..instant(
            "Violent Eruption",
            cost(&[generic(1), r(), r(), r()]),
            Effect::DealDamageDivided {
                total: Value::Const(4),
                filter: R::Creature.or(R::Player).or(R::Planeswalker),
                max_targets: 4,
                retaliate_to_source: false,
            },
        )
    }
}

/// Temporary Insanity — {3}{R}. Steal a creature small enough for your
/// graveyard.
pub fn temporary_insanity() -> CardDefinition {
    instant(
        "Temporary Insanity",
        cost(&[generic(3), r()]),
        Effect::Seq(vec![
            Effect::Untap {
                what: target_filtered(R::Creature.and(R::PowerLessThanYourGraveyardCount)),
                up_to: None,
            },
            Effect::GainControl {
                what: Selector::Target(0),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Spirit Flare — {3}{W}. Tap one of yours to shoot an attacker or blocker.
pub fn spirit_flare() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), w()]))],
        flashback_additional_cost: vec![AdditionalCastCost::PayLife { amount: 3 }],
        ..instant(
            "Spirit Flare",
            cost(&[generic(3), w()]),
            Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::Untapped)) },
                Effect::DealDamage {
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature
                            .and(R::ControlledByOpponent)
                            .and(R::IsAttacking.or(R::IsBlocking)),
                    },
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                },
            ]),
        )
    }
}

/// False Memories — {1}{U}. Seven cards of fuel, repossessed at end of turn.
pub fn false_memories() -> CardDefinition {
    instant(
        "False Memories",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(7) },
            Effect::AtNextEndStep {
                body: Box::new(Effect::MoveChosen {
                    from: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: R::Any,
                    },
                    filter: None,
                    count: Value::Const(7),
                    up_to: true,
                    to: ZoneDest::Exile,
                }),
            },
        ]),
    )
}

/// Plagiarize — {3}{U}. Take over a player's draws for the turn.
pub fn plagiarize() -> CardDefinition {
    instant(
        "Plagiarize",
        cost(&[generic(3), u()]),
        Effect::RedirectDrawsThisTurn { from: PlayerRef::Target(0) },
    )
}

/// Equal Treatment — {1}{W}. Every damage event this turn is exactly 2, and
/// a card.
pub fn equal_treatment() -> CardDefinition {
    instant(
        "Equal Treatment",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::DamageBecomesThisTurn { at_least: 1, becomes: 2 },
            draw(1),
        ]),
    )
}

/// Flaming Gambit — {X}{R}. X to a player, who may take it on a creature
/// instead.
pub fn flaming_gambit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[x(), r(), r()]))],
        ..instant(
            "Flaming Gambit",
            cost(&[x(), r()]),
            Effect::DamageTargetPlayerMayRedirect { amount: Value::XFromCost },
        )
    }
}

/// Radiate — {3}{R}{R}. Fork a single-target spell onto everything else it
/// could hit.
pub fn radiate() -> CardDefinition {
    instant(
        "Radiate",
        cost(&[generic(3), r(), r()]),
        Effect::CopySpellForEachOtherTarget {
            what: target_filtered(
                R::IsSpellOnStack
                    .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))
                    .and(R::SpellWithSingleTarget),
            ),
        },
    )
}

/// Retraced Image — {U}. Replay a card you already have a copy of.
pub fn retraced_image() -> CardDefinition {
    sorcery("Retraced Image", cost(&[u()]), Effect::RevealAndReplayNamedPermanent)
}

/// Parallel Evolution — {3}{G}{G}. Every creature token doubles.
pub fn parallel_evolution() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(4), g(), g(), g()]))],
        ..sorcery(
            "Parallel Evolution",
            cost(&[generic(3), g(), g()]),
            Effect::CopyEachCreatureToken,
        )
    }
}
