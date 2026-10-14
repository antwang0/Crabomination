//! Commander: one-card gaps of EDHREC's commanders ranked 4501-5500 (the
//! 2026-10-14 scan), first file. Tests in `tests/recent_b/cmdr_rank5k_a.rs`.

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, landfall, on_attack, on_dies, target_filtered};
use crate::card::MayPlayDuration;
use crate::effect::{DelayedTriggerKind, Effect, ManaPayload, PlayerRef, Predicate, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, r, u, w, x};
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

fn aura(name: &'static str, mana: ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        ..Default::default()
    }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn la(loyalty_cost: i32, effect: Effect) -> LoyaltyAbility {
    LoyaltyAbility { loyalty_cost, effect, ..Default::default() }
}

/// Inundate — {3}{U}{U}{U} sorcery. Return all nonblue creatures to their
/// owners' hands.
pub fn inundate() -> CardDefinition {
    CardDefinition {
        name: "Inundate",
        cost: cost(&[generic(3), u(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Blue))))),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        ..Default::default()
    }
}

/// Luminous Bonds — {2}{W} Aura. Enchanted creature can't attack or block.
pub fn luminous_bonds() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        ..aura("Luminous Bonds", cost(&[generic(2), w()]), R::Creature)
    }
}

/// Awakened Awareness — {X}{U}{U} Aura, enchant artifact or creature. It
/// enters putting X +1/+1 counters on the enchanted permanent, which as a
/// creature has base power and toughness 1/1.
pub fn awakened_awareness() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::AttachedTo(Box::new(Selector::This)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::XFromCost,
        })],
        equipped_bonus: Some(EquipBonus { set_base_pt: Some((1, 1)), ..Default::default() }),
        ..aura("Awakened Awareness", cost(&[x(), u(), u()]), R::Artifact.or(R::Creature))
    }
}

/// Miasmic Mummy — {1}{B} 2/2 Zombie Jackal. When it enters, each player
/// discards a card.
pub fn miasmic_mummy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::ONE,
            random: false,
        })],
        ..creature("Miasmic Mummy", cost(&[generic(1), b()]), vec![CreatureType::Zombie, CreatureType::Jackal], 2, 2)
    }
}

/// Prowling Felidar — {3}{W} 2/3 Cat Beast, vigilance. Landfall: a +1/+1
/// counter on it.
pub fn prowling_felidar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![landfall(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature("Prowling Felidar", cost(&[generic(3), w()]), vec![CreatureType::Cat, CreatureType::Beast], 2, 3)
    }
}

/// Hedgewitch's Mask — {W} Equipment. Equipped creature gets +1/+1 and can't
/// be blocked by creatures with power 4 or greater. Equip {2}.
pub fn hedgewitchs_mask() -> CardDefinition {
    CardDefinition {
        name: "Hedgewitch's Mask",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::CantBeBlockedByPowerAtLeast(4)],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Zhang Liao, Hero of Hefei — {4}{B}{B} legendary 3/3 Human Soldier.
/// Whenever it deals damage to an opponent, that opponent discards a card.
pub fn zhang_liao_hero_of_hefei() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer }),
            effect: Effect::Discard { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE, random: false },
        }],
        ..legend(
            "Zhang Liao, Hero of Hefei",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// The Mountain-king's Return — {2}{W} Saga. I: recruit (draw, discard; a
/// nonland discard makes a 1/1 white Human Soldier). II: return a creature
/// card with mana value 3 or less from your graveyard to the battlefield.
/// III: a +1/+1 counter on up to one target creature.
pub fn the_mountain_kings_return() -> CardDefinition {
    let recruit = Effect::Seq(vec![
        draw(1),
        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        Effect::If {
            cond: Predicate::DiscardedNonlandThisEffect { who: PlayerRef::You },
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Human Soldier".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::White],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Human, CreatureType::Soldier],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            }),
            else_: Box::new(Effect::Noop),
        },
    ]);
    CardDefinition {
        name: "The Mountain-king's Return",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, recruit),
            (
                2,
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(3))),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ),
            (
                3,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::AddCounter {
                        what: target_filtered(R::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Black Tom Cassidy — {1}{G} legendary 1/3 Mutant Villain. {T}: add {G}; if
/// you control another Mutant, you gain 1 life.
pub fn black_tom_cassidy() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green]) },
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Mutant).and(R::ControlledByYou).and(R::OtherThanSource),
                    )),
                    then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Black Tom Cassidy",
            cost(&[generic(1), g()]),
            vec![CreatureType::Mutant, CreatureType::Villain],
            1,
            3,
        )
    }
}

/// Windbrisk Raptor — {5}{W}{W} 5/7 flying Bird. Attacking creatures you
/// control have lifelink.
pub fn windbrisk_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![static_ab(
            "Attacking creatures you control have lifelink.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::IsAttacking).and(R::ControlledByYou)),
                keyword: Keyword::Lifelink,
            },
        )],
        ..creature("Windbrisk Raptor", cost(&[generic(5), w(), w()]), vec![CreatureType::Bird], 5, 7)
    }
}

/// Automated Warfare System — {2}{B} 2/2 flying Construct artifact creature.
/// Whenever it attacks, you may sacrifice another creature or artifact; if you
/// do, draw a card.
pub fn automated_warfare_system() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::MaySacrifice {
            description: "Sacrifice another creature or artifact to draw a card.".into(),
            filter: R::Creature.or(R::Artifact).and(R::OtherThanSource).and(R::ControlledByYou),
            count: Value::ONE,
            then: Box::new(draw(1)),
            else_: None,
        })],
        ..creature("Automated Warfare System", cost(&[generic(2), b()]), vec![CreatureType::Construct], 2, 2)
    }
}

/// Runed Servitor — {2} 2/2 Construct artifact creature. When it dies, each
/// player draws a card.
pub fn runed_servitor() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::ONE,
        })],
        ..creature("Runed Servitor", cost(&[generic(2)]), vec![CreatureType::Construct], 2, 2)
    }
}

/// Winter Soldier, Bucky Barnes — {W} legendary 2/2 Human Soldier Hero; it
/// enters tapped.
pub fn winter_soldier_bucky_barnes() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Winter Soldier enters tapped.",
            StaticEffect::EntersTapped { applies_to: Selector::This },
        )],
        ..legend(
            "Winter Soldier, Bucky Barnes",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Gandalf, Spark Starter — {4}{R}{R} legendary 4/3 Avatar Wizard, reach.
/// When he enters, 3 damage divided among one, two, or three targets.
pub fn gandalf_spark_starter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::DealDamageDivided {
            total: Value::Const(3),
            filter: R::any_target(),
            max_targets: 3,
            retaliate_to_source: false,
        })],
        ..legend(
            "Gandalf, Spark Starter",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            4,
            3,
        )
    }
}

/// Shameless Charlatan — {1}{U} legendary enchantment Background. Commander
/// creatures you own have "{2}{U}: This creature becomes a copy of another
/// target creature." The grant is not copiable, so the copy keeps it.
pub fn shameless_charlatan() -> CardDefinition {
    CardDefinition {
        name: "Shameless Charlatan",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        static_abilities: vec![static_ab(
            "Commander creatures you own have \"{2}{U}: This creature becomes a copy of another target creature.\"",
            StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Creature.and(R::IsCommander).and(R::OwnedByYou)),
                ability: ActivatedAbility {
                    mana_cost: cost(&[generic(2), u()]),
                    effect: Effect::BecomeCopyOf {
                        what: Selector::This,
                        source: target_filtered(R::Creature.and(R::IsSource.negate())),
                        extra_creature_types: vec![],
                        keep_own_triggered: false,
                        keep_own_activated: false,
                        keep_name: false,
                    },
                    ..Default::default()
                },
                condition: None,
            },
        )],
        ..Default::default()
    }
}

/// Fugitive Droid — {U} 1/1 Robot Scientist artifact creature. Unblockable if
/// an artifact entered under your control this turn. {U}, sacrifice it:
/// counter target spell that targets an artifact or creature you control.
pub fn fugitive_droid() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![static_ab(
            "This creature can't be blocked if an artifact entered the battlefield under your control this turn.",
            StaticEffect::WhileCondition {
                condition: Predicate::ArtifactEnteredThisTurn { who: PlayerRef::You },
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Unblockable }),
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::SpellTargetsMatching(Box::new(
                    R::Artifact.or(R::Creature).and(R::ControlledByYou),
                )))),
            },
            ..Default::default()
        }],
        ..creature("Fugitive Droid", cost(&[u()]), vec![CreatureType::Robot, CreatureType::Scientist], 1, 1)
    }
}

/// Patrician's Scorn — {3}{W} instant. Free if you've cast another white
/// spell this turn. Destroy all enchantments.
pub fn patricians_scorn() -> CardDefinition {
    CardDefinition {
        name: "Patrician's Scorn",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            condition: Some(Predicate::CastSpellThisTurnWith {
                who: PlayerRef::You,
                colors: vec![Color::White],
                types: vec![],
            }),
            ..Default::default()
        }),
        effect: Effect::Destroy { what: Selector::EachPermanent(R::Enchantment) },
        ..Default::default()
    }
}

/// Flash Thompson, Spider-Fan — {1}{W} legendary 2/2 Human Citizen, flash.
/// When he enters, one or both: tap target creature; untap target creature.
pub fn flash_thompson_spider_fan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            ],
        })],
        ..legend(
            "Flash Thompson, Spider-Fan",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Citizen],
            2,
            2,
        )
    }
}

/// Ruric Thar, Magecrusher — {5}{G}{G} legendary 7/7 Ogre Warrior; can't be
/// countered; reach, vigilance, trample; hexproof as long as it hasn't dealt
/// combat damage yet.
pub fn ruric_thar_magecrusher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered, Keyword::Reach, Keyword::Vigilance, Keyword::Trample],
        static_abilities: vec![static_ab(
            "Ruric Thar has hexproof as long as they haven't dealt combat damage yet.",
            StaticEffect::WhileCondition {
                condition: Predicate::Not(Box::new(Predicate::SourceHasDealtCombatDamage)),
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Hexproof }),
            },
        )],
        ..legend(
            "Ruric Thar, Magecrusher",
            cost(&[generic(5), g(), g()]),
            vec![CreatureType::Ogre, CreatureType::Warrior],
            7,
            7,
        )
    }
}

/// Chandra, Chill of Compliance — {1}{U}{U} planeswalker, loyalty 3. +1:
/// surveil 1, a noncreature nonland card it binned goes to your hand. +1: add
/// {U} for noncreature spells. −X: tap an artifact or creature and put X stun
/// counters on it. −6: an emblem drawing a card per spell you cast.
pub fn chandra_chill_of_compliance() -> CardDefinition {
    CardDefinition {
        name: "Chandra, Chill of Compliance",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Chandra], ..Default::default() },
        base_loyalty: 3,
        loyalty_abilities: vec![
            // The surveilled card is the top one, so it is bound first and
            // read where the surveil left it (Enlightened Confidant's shape).
            la(
                1,
                Effect::ForEach {
                    selector: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                    body: Box::new(Effect::Seq(vec![
                        Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                        Effect::If {
                            cond: Predicate::EntityMatches {
                                what: Selector::TriggerSource,
                                filter: R::InYourGraveyard
                                    .and(R::Not(Box::new(R::Creature)))
                                    .and(R::Not(Box::new(R::Land))),
                            },
                            then: Box::new(Effect::Move {
                                what: Selector::TriggerSource,
                                to: ZoneDest::Hand(PlayerRef::You),
                            }),
                            else_: Box::new(Effect::Noop),
                        },
                    ])),
                },
            ),
            la(
                1,
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Blue])),
                        SpendRestriction::NoncreatureSpellsOnly,
                    ),
                },
            ),
            LoyaltyAbility {
                x_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Tap { what: target_filtered(R::Artifact.or(R::Creature)) },
                    Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::XFromCost },
                ]),
                ..Default::default()
            },
            la(
                -6,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Chandra, Chill of Compliance".into(),
                    statics: vec![],
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                        effect: draw(1),
                    }],
                },
            ),
        ],
        ..Default::default()
    }
}

/// Jaya, Fiery Negotiator — {2}{R}{R} planeswalker, loyalty 4. +1: a 1/1 red
/// Monk with prowess. −1: exile the top two, play one this turn. −2: whenever
/// you attack this turn, Jaya deals damage equal to the attackers to target
/// creature an opponent controls. −8: an emblem copying red instants and
/// sorceries twice.
pub fn jaya_fiery_negotiator() -> CardDefinition {
    let monk = TokenDefinition {
        name: "Monk".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Monk], ..Default::default() },
        keywords: vec![Keyword::Prowess],
        ..Default::default()
    };
    CardDefinition {
        name: "Jaya, Fiery Negotiator",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Jaya], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            la(1, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(monk) }),
            la(-1, crate::sets::impulse_choose_one(2, MayPlayDuration::EndOfThisTurn)),
            la(
                -2,
                Effect::DelayUntilWithCapture {
                    kind: DelayedTriggerKind::EachAttackThisTurn,
                    capture: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    body: Box::new(Effect::DealDamage {
                        to: Selector::Target(0),
                        amount: Value::CountOf(Box::new(Selector::EachPermanent(
                            R::Creature.and(R::IsAttacking).and(R::ControlledByYou),
                        ))),
                    }),
                },
            ),
            la(
                -8,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Jaya, Fiery Negotiator".into(),
                    statics: vec![],
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                            Predicate::CastSpellMatches(
                                R::HasColor(Color::Red).and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                            ),
                        ),
                        effect: Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::Const(2) },
                    }],
                },
            ),
        ],
        ..Default::default()
    }
}
