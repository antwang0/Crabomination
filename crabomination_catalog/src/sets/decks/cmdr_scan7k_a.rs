//! Commander: one-card gaps of the 2026-10-15 full-list EDHREC scan (all
//! 6,799 commanders) — each card here is the last one an unseated average
//! deck was missing. Tests in `tests/recent_b/cmdr_rescan_a.rs`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    LoyaltyAbility, MayPlayDuration, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, mint_token, mint_treasures, on_attack, target_any, target_filtered, target_n};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, w};

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

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn your_step(step: TurnStep) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl)
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn soldier() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    }
}

fn counter_on_your_creature() -> Effect {
    Effect::AddCounter {
        what: target_filtered(R::Creature.and(R::ControlledByYou)),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    }
}

/// Elspeth, Knight-Errant — {2}{W}{W} planeswalker, loyalty 4. +1: a 1/1
/// Soldier. +1: target creature gets +3/+3 and flying until end of turn.
/// −8: emblem "Artifacts, creatures, enchantments, and lands you control
/// have indestructible." (Myojin of Cleansing Fire.)
pub fn elspeth_knight_errant() -> CardDefinition {
    CardDefinition {
        name: "Elspeth, Knight-Errant",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Elspeth], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility { loyalty_cost: 1, effect: mint_token(soldier(), 1), ..Default::default() },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature),
                        power: Value::Const(3),
                        toughness: Value::Const(3),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Elspeth, Knight-Errant".into(),
                    triggered: vec![],
                    statics: vec![StaticAbility {
                        description: "Artifacts, creatures, enchantments, and lands you control have indestructible.",
                        effect: StaticEffect::GrantKeyword {
                            applies_to: yours(
                                R::HasCardType(CardType::Artifact)
                                    .or(R::Creature)
                                    .or(R::HasCardType(CardType::Enchantment))
                                    .or(R::HasCardType(CardType::Land)),
                            ),
                            keyword: Keyword::Indestructible,
                        },
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Nighthawk, Dark Defender — {1}{W} legendary 2/1 Human Hero; flying.
/// Whenever it or another Hero you control enters, target creature gets
/// +1/+1 until end of turn. (Its own deck.)
pub fn nighthawk_dark_defender() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCreatureType(CreatureType::Hero) },
            ),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..legend("Nighthawk, Dark Defender", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Hero], 2, 1)
    }
}

/// Training Regimen — {3}{G} enchantment. Creatures you control with +1/+1
/// counters have trample; at the beginning of combat on your turn, a +1/+1
/// counter on target creature you control. (Rocksteady, Mutant Marauder.)
pub fn training_regimen() -> CardDefinition {
    CardDefinition {
        name: "Training Regimen",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with +1/+1 counters on them have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))),
                keyword: Keyword::Trample,
            },
        }],
        triggered_abilities: vec![TriggeredAbility { event: your_step(TurnStep::BeginCombat), effect: counter_on_your_creature() }],
        ..Default::default()
    }
}

/// S.H.I.E.L.D. Helicarrier — {4} 4/5 Vehicle; flying; ETB two 1/1 white
/// Soldiers; crew 6. (The Thanos-Copter.)
pub fn s_h_i_e_l_d_helicarrier() -> CardDefinition {
    CardDefinition {
        name: "S.H.I.E.L.D. Helicarrier",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Crew(6)],
        triggered_abilities: vec![etb(mint_token(soldier(), 2))],
        ..Default::default()
    }
}

/// Heroes for Hire — {3}{R}{R} enchantment. ETB three Treasures. Sacrifice a
/// Treasure: exile your top card; you may play it this turn. (Captain
/// Lannery Storm.)
pub fn heroes_for_hire() -> CardDefinition {
    CardDefinition {
        name: "Heroes for Hire",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(mint_treasures(3))],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Treasure), 1)),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Greatsword of Tyr — {1}{W} Equipment. Whenever equipped creature attacks,
/// put a +1/+1 counter on it and tap up to one target creature defending
/// player controls. Equip {W}. (Raiyuu, Storm's Edge.)
pub fn greatsword_of_tyr() -> CardDefinition {
    equipment(
        "Greatsword of Tyr",
        cost(&[generic(1), w()]),
        cost(&[w()]),
        EquipBonus {
            triggered_abilities: vec![on_attack(Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature.and(R::ControlledByDefendingPlayer),
                    effect: Box::new(Effect::Tap { what: target_n(0) }),
                },
            ]))],
            ..Default::default()
        },
    )
}

/// Livaan, Cultist of Tiamat — {2}{R} legendary 1/3 Dragon Shaman. Whenever
/// you cast a noncreature spell, target creature gets +X/+0 until end of
/// turn, X that spell's mana value. Choose a Background. (The Scarlet Witch.)
pub fn livaan_cultist_of_tiamat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Not(Box::new(R::Creature)),
            }),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..legend("Livaan, Cultist of Tiamat", cost(&[generic(2), r()]), vec![CreatureType::Dragon, CreatureType::Shaman], 1, 3)
    }
}

/// The Black Arrow — {3} legendary Equipment; flash. ETB: 1 damage to any
/// target; a Dragon dealt damage this way is destroyed. Equipped creature gets
/// +1/+1 and has reach. Equip {1}. (Riven Turnbull.)
pub fn the_black_arrow() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(1)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::ONE },
            Effect::Destroy {
                what: Selector::DamagedThisResolution { filter: R::HasCreatureType(CreatureType::Dragon) },
            },
        ]))],
        ..equipment(
            "The Black Arrow",
            cost(&[generic(3)]),
            cost(&[generic(1)]),
            EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::Reach], ..Default::default() },
        )
    }
}

/// Captain America, Super-Soldier — {1}{W}{W} legendary 3/2 Human Soldier
/// Hero; first strike; enters with a shield counter. While he has one, you
/// and other Heroes you control have hexproof. (Captain Marvel, Earth's
/// Protector.)
pub fn captain_america_super_soldier() -> CardDefinition {
    let shielded = |inner| StaticEffect::WhileCountersAtLeast { kind: CounterType::Shield, n: 1, inner: Box::new(inner) };
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        enters_with_counters: Some((CounterType::Shield, Value::ONE)),
        static_abilities: vec![
            StaticAbility { description: "As long as he has a shield counter, you have hexproof.", effect: shielded(StaticEffect::ControllerHasHexproof) },
            StaticAbility {
                description: "As long as he has a shield counter, other Heroes you control have hexproof.",
                effect: shielded(StaticEffect::GrantKeyword {
                    applies_to: yours(R::HasCreatureType(CreatureType::Hero).and(R::OtherThanSource)),
                    keyword: Keyword::Hexproof,
                }),
            },
        ],
        ..legend(
            "Captain America, Super-Soldier",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            3,
            2,
        )
    }
}

/// Gollum, Silent Slinker // Meager Meal — {3}{B} legendary 4/3 Halfling
/// Horror with menace; Adventure {B} sorcery: a +1/+1 counter on up to one
/// target creature, and target player gains 2 life. (Both Gollum decks.)
pub fn gollum_silent_slinker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        adventure: Some(Box::new(Adventure {
            name: "Meager Meal",
            cost: cost(&[b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::SlotGroups(vec![
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::AddCounter { what: target_n(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
                },
                Effect::GainLife { who: target_filtered(R::Player), amount: Value::Const(2) },
            ]),
        })),
        ..legend("Gollum, Silent Slinker", cost(&[generic(3), b()]), vec![CreatureType::Halfling, CreatureType::Horror], 4, 3)
    }
}

/// Origin of Thor — {2}{R} Saga. I: you may discard a card; if you do, draw
/// two. II: this turn, whenever you cast a spell, a +1/+1 counter on target
/// creature you control. III: target creature you control deals damage equal
/// to its power to each opponent. (Thor, God of Thunder.)
pub fn origin_of_thor() -> CardDefinition {
    CardDefinition {
        name: "Origin of Thor",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::MayDiscard {
                    description: "You may discard a card. If you do, draw two cards.".into(),
                    count: Value::ONE,
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
                    else_: None,
                },
            ),
            (
                2,
                Effect::TriggersThisTurn {
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                        effect: counter_on_your_creature(),
                    }],
                },
            ),
            (
                3,
                Effect::DealDamageFrom {
                    source: target_filtered(R::Creature.and(R::ControlledByYou)),
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Triskaidekaphobia — {3}{B} enchantment. At the beginning of your upkeep,
/// choose one — each player with exactly 13 life loses the game, then each
/// player gains 1 life; or the same, then each player loses 1 life.
/// (Gallowbraid.)
pub fn triskaidekaphobia() -> CardDefinition {
    let thirteen_lose = || Effect::EachPlayerDoes {
        who: PlayerRef::EachPlayer,
        body: Box::new(Effect::If {
            cond: Predicate::ValueEquals(Value::LifeOf(PlayerRef::You), Value::Const(13)),
            then: Box::new(Effect::LoseGame { who: PlayerRef::You }),
            else_: Box::new(Effect::Seq(vec![])),
        }),
    };
    let each = || Selector::Player(PlayerRef::EachPlayer);
    CardDefinition {
        name: "Triskaidekaphobia",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::Upkeep),
            effect: Effect::ChooseMode(vec![
                Effect::Seq(vec![thirteen_lose(), Effect::GainLife { who: each(), amount: Value::ONE }]),
                Effect::Seq(vec![thirteen_lose(), Effect::LoseLife { who: each(), amount: Value::ONE }]),
            ]),
        }],
        ..Default::default()
    }
}

/// Avatar of Growth — {4}{G}{G} 4/4 Elemental Avatar; costs {1} less per
/// opponent; trample. ETB: each player searches for up to two basic lands and
/// puts them onto the battlefield. (Shizuko, Caller of Autumn.)
pub fn avatar_of_growth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each opponent you have.",
            effect: StaticEffect::SelfCostReducedPerOpponent { per: 1 },
        }],
        triggered_abilities: vec![etb(Effect::EachPlayerDoes {
            who: PlayerRef::EachPlayer,
            body: Box::new(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                count: Value::Const(2),
            }),
        })],
        ..creature("Avatar of Growth", cost(&[generic(4), g(), g()]), vec![CreatureType::Elemental, CreatureType::Avatar], 4, 4)
    }
}

/// Dragonrage — {2}{R} instant. Add {R} per attacking creature you control;
/// until end of turn they gain "{R}: this creature gets +1/+0 until end of
/// turn." (Hulk, Bruce Banner.)
pub fn dragonrage() -> CardDefinition {
    let attackers = R::Creature.and(R::IsAttacking).and(R::ControlledByYou);
    CardDefinition {
        name: "Dragonrage",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Red, Value::count(Selector::EachPermanent(attackers.clone()))),
            },
            Effect::GrantActivatedAbilityToMatching {
                filter: attackers,
                ability: Box::new(ActivatedAbility {
                    mana_cost: cost(&[r()]),
                    effect: Effect::PumpPT {
                        what: Selector::This,
                        power: Value::ONE,
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

