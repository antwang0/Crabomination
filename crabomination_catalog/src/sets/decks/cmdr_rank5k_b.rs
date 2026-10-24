//! Commander: two-card gaps of EDHREC's commanders ranked 4501-5500 (the
//! 2026-10-14 scan). Tests in `tests/recent_b/cmdr_rank5k_b.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, MayPlayDuration, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector};
use crate::mana::{Color, ManaCost, b, cost, generic, r, u, w};
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

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: name.into(),
            power: p,
            toughness: t,
            colors,
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: types, ..Default::default() },
            keywords,
            ..Default::default()
        }),
    }
}

fn noncreature_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::Not(Box::new(R::Creature)))),
        effect,
    }
}

/// Cast Away Doubt — {2}{B} sorcery. Draw two cards; 2 damage to each player.
pub fn cast_away_doubt() -> CardDefinition {
    CardDefinition {
        name: "Cast Away Doubt",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            draw(2),
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Machine Man, Model X-51 — {4} legendary 2/3 Robot Hero artifact creature.
/// Whenever you cast a noncreature spell, a +1/+1 counter on him and flying
/// until end of turn.
pub fn machine_man_model_x_51() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![noncreature_cast(Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Flying, duration: Duration::EndOfTurn },
        ]))],
        ..legend("Machine Man, Model X-51", cost(&[generic(4)]), vec![CreatureType::Robot, CreatureType::Hero], 2, 3)
    }
}

/// The Echoverse Fulcrum — {2} legendary artifact. It enters looting one.
/// {5}, {T}, exile it: destroy all creatures, as a sorcery.
pub fn the_echoverse_fulcrum() -> CardDefinition {
    CardDefinition {
        name: "The Echoverse Fulcrum",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            draw(1),
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Valorous Steed — {4}{W} 3/3 Unicorn, vigilance. It enters with a 2/2 white
/// Knight token with vigilance.
pub fn valorous_steed() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(token(
            "Knight",
            2,
            2,
            vec![Color::White],
            vec![CreatureType::Knight],
            vec![Keyword::Vigilance],
        ))],
        ..creature("Valorous Steed", cost(&[generic(4), w()]), vec![CreatureType::Unicorn], 3, 3)
    }
}

/// Turn Aside — {U} instant. Counter target spell that targets a permanent
/// you control.
pub fn turn_aside() -> CardDefinition {
    CardDefinition {
        name: "Turn Aside",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(
                R::IsSpellOnStack.and(R::SpellTargetsMatching(Box::new(R::Permanent.and(R::ControlledByYou)))),
            ),
        },
        ..Default::default()
    }
}

/// Whirlermaker — {3} artifact. {4}, {T}: a 1/1 colorless flying Thopter
/// artifact creature token.
pub fn whirlermaker() -> CardDefinition {
    CardDefinition {
        name: "Whirlermaker",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Thopter".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Makeshift Munitions — {1}{R} enchantment. {1}, sacrifice an artifact or
/// creature: 1 damage to any target.
pub fn makeshift_munitions() -> CardDefinition {
    CardDefinition {
        name: "Makeshift Munitions",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Artifact.or(R::Creature), 1)),
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Chase Stein, Runaway — {1}{R} legendary 2/2 Human Hero. {T}, discard a
/// card: exile the top card; you may play it until the end of your next turn.
pub fn chase_stein_runaway() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..legend("Chase Stein, Runaway", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Hero], 2, 2)
    }
}

/// Okoye, Dora Milaje Leader — {3}{W} legendary 3/2 Human Warrior Hero. She
/// enters with two 1/1 white Soldier tokens; attacking creature tokens you
/// control have first strike.
pub fn okoye_dora_milaje_leader() -> CardDefinition {
    let soldier = || token("Soldier", 1, 1, vec![Color::White], vec![CreatureType::Soldier], vec![]);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![soldier(), soldier()]))],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Attacking creature tokens you control have first strike.",
            effect: crate::card::StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::IsToken).and(R::IsAttacking).and(R::ControlledByYou),
                ),
                keyword: Keyword::FirstStrike,
            },
        }],
        ..legend(
            "Okoye, Dora Milaje Leader",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero],
            3,
            2,
        )
    }
}

/// Winged Shepherd — {5}{W} 3/3 Angel; flying, vigilance; cycling {W}.
pub fn winged_shepherd() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Cycling(cost(&[w()]))],
        ..creature("Winged Shepherd", cost(&[generic(5), w()]), vec![CreatureType::Angel], 3, 3)
    }
}

/// Masked Meower — {R} 1/1 Spider Cat Hero, haste. Discard a card, sacrifice
/// it: draw a card.
pub fn masked_meower() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            sac_cost: true,
            effect: draw(1),
            ..Default::default()
        }],
        ..creature(
            "Masked Meower",
            cost(&[r()]),
            vec![CreatureType::Spider, CreatureType::Cat, CreatureType::Hero],
            1,
            1,
        )
    }
}

/// Sun Ce, Young Conquerer — {3}{U}{U} legendary 3/3 Human Soldier,
/// horsemanship. When he enters, you may return target creature to its
/// owner's hand.
pub fn sun_ce_young_conquerer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Horsemanship],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature),
                to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..legend(
            "Sun Ce, Young Conquerer",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Thor, God of Thunder — {3}{R}{R} legendary 5/5 flying God Warrior Hero.
/// When he enters, exile an Equipment, instant, or sorcery card from your
/// graveyard; you may play it until the end of your next turn. Whenever you
/// cast a noncreature spell, he deals damage equal to its mana value to any
/// target.
pub fn thor_god_of_thunder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment)
                            .or(R::HasCardType(CardType::Instant))
                            .or(R::HasCardType(CardType::Sorcery))
                            .and(R::InYourGraveyard),
                    ),
                    to: crate::effect::ZoneDest::Exile,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ])),
            noncreature_cast(Effect::DealDamage {
                to: crate::effect::shortcut::target_any(),
                amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            }),
        ],
        ..legend(
            "Thor, God of Thunder",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::God, CreatureType::Warrior, CreatureType::Hero],
            5,
            5,
        )
    }
}

/// Heroes' Hangout — {R} sorcery. Choose one — Date Night: exile the top two,
/// play one of them until the end of your next turn; Patrol Night: one or two
/// target creatures each get +1/+0 and first strike until end of turn.
pub fn heroes_hangout() -> CardDefinition {
    CardDefinition {
        name: "Heroes' Hangout",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            crate::sets::impulse_choose_one(2, MayPlayDuration::EndOfControllersNextTurn),
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 1,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::Target(0),
                        power: Value::ONE,
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::FirstStrike,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        ]),
        ..Default::default()
    }
}

/// Extract the Truth — {1}{B} sorcery. Choose one — target opponent reveals
/// their hand, you may choose a creature, enchantment, or planeswalker card
/// and they discard it; or target opponent sacrifices an enchantment.
pub fn extract_the_truth() -> CardDefinition {
    CardDefinition {
        name: "Extract the Truth",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::DiscardChosen {
                from: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: R::Creature.or(R::Enchantment).or(R::Planeswalker),
            },
            Effect::Sacrifice { who: target_filtered(R::OpponentPlayer), count: Value::ONE, filter: R::Enchantment },
        ]),
        ..Default::default()
    }
}

/// Runed Stalactite — {1} Equipment. Equipped creature gets +1/+1 and is every
/// creature type. Equip {2}.
pub fn runed_stalactite() -> CardDefinition {
    CardDefinition {
        name: "Runed Stalactite",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Changeling],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Blades of Velis Vel — {1}{R} Kindred instant — Shapeshifter, changeling.
/// Up to two target creatures each get +2/+0 and gain all creature types
/// until end of turn.
pub fn blades_of_velis_vel() -> CardDefinition {
    CardDefinition {
        name: "Blades of Velis Vel",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
        keywords: vec![Keyword::Changeling],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Changeling, duration: Duration::EndOfTurn },
            ])),
        },
        ..Default::default()
    }
}

/// Wu Scout — {1}{U} 1/1 Human Soldier Scout, horsemanship. When it enters,
/// look at target opponent's hand.
pub fn wu_scout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Horsemanship],
        triggered_abilities: vec![etb(Effect::LookAtHand { who: target_filtered(R::OpponentPlayer) })],
        ..creature(
            "Wu Scout",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Scout],
            1,
            1,
        )
    }
}

/// Shuri, Vibranium Technologist — {2}{W/U} legendary 2/2 Human Artificer
/// Hero, vigilance. When she enters, choose one — a 1/1 colorless flying
/// Robot Hero artifact creature token; or draw a card.
pub fn shuri_vibranium_technologist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Robot Hero".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Robot, CreatureType::Hero],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                }),
            },
            draw(1),
        ]))],
        ..legend(
            "Shuri, Vibranium Technologist",
            cost(&[generic(2), crate::mana::hybrid(Color::White, Color::Blue)]),
            vec![CreatureType::Human, CreatureType::Artificer, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Spinneret and Spiderling — {R} legendary 1/2 Spider Human Hero. Whenever
/// you attack with two or more Spiders, a +1/+1 counter on it. Whenever it
/// deals 4 or more damage, exile the top card; you may play it until the end
/// of your next turn.
pub fn spinneret_and_spiderling() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(Predicate::ValueAtLeast(
                    Value::count(Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Spider).and(R::IsAttacking).and(R::ControlledByYou),
                    )),
                    Value::Const(2),
                )),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource)
                    .with_filter(Predicate::ValueAtLeast(Value::TriggerEventAmount, Value::Const(4))),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            },
        ],
        ..legend(
            "Spinneret and Spiderling",
            cost(&[r()]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            1,
            2,
        )
    }
}

/// Wingnut, Bat on the Belfry — {1}{R} legendary 1/2 Bat Mutant. Alliance:
/// flying, menace, or haste (your choice) until end of turn. Whenever it
/// attacks, each other attacking creature gets +1/+0 until end of turn.
pub fn wingnut_bat_on_the_belfry() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
                effect: Effect::ChooseMode(
                    [Keyword::Flying, Keyword::Menace, Keyword::Haste]
                        .into_iter()
                        .map(|keyword| Effect::GrantKeyword { what: Selector::This, keyword, duration: Duration::EndOfTurn })
                        .collect(),
                ),
            },
            crate::effect::shortcut::battle_cry(1),
        ],
        ..legend(
            "Wingnut, Bat on the Belfry",
            cost(&[generic(1), r()]),
            vec![CreatureType::Bat, CreatureType::Mutant],
            1,
            2,
        )
    }
}

/// Hulkling, Young Avenger — {2}{R}{R} legendary 4/4 flying Kree Skrull Hero.
/// Whenever you cast a noncreature spell, he becomes a copy of up to one
/// other target creature until end of turn, except his name, 4/4, flying and
/// this ability.
pub fn hulkling_young_avenger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![noncreature_cast(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: target_filtered(R::Creature.and(R::OtherThanSource)),
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                },
                Effect::AmendCopy {
                    what: Selector::This,
                    keep_activated: vec![],
                    keep_triggered: vec![0],
                    pt: Some((4, 4)),
                    keep_name: true,
                    legendary: false,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![],
                    creature_types: vec![],
                },
            ])),
        })],
        ..legend(
            "Hulkling, Young Avenger",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Kree, CreatureType::Skrull, CreatureType::Hero],
            4,
            4,
        )
    }
}

/// Night Nurse, Healer of Heroes — {1}{W} legendary 2/1 Human Doctor Hero;
/// flash, lifelink. When she enters, return target permanent card put into
/// your graveyard this turn to your hand.
pub fn night_nurse_healer_of_heroes() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::PermanentCard.and(R::PutIntoGraveyardThisTurn).and(R::InYourGraveyard)),
            to: crate::effect::ZoneDest::Hand(PlayerRef::You),
        })],
        ..legend(
            "Night Nurse, Healer of Heroes",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Doctor, CreatureType::Hero],
            2,
            1,
        )
    }
}

/// Crookclaw Transmuter — {3}{U} 3/1 Bird Wizard; flash, flying. When it
/// enters, switch target creature's power and toughness until end of turn.
pub fn crookclaw_transmuter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::SwitchPT { what: target_filtered(R::Creature), duration: Duration::EndOfTurn })],
        ..creature("Crookclaw Transmuter", cost(&[generic(3), u()]), vec![CreatureType::Bird, CreatureType::Wizard], 3, 1)
    }
}

/// Profaner of the Dead — {3}{U} 3/3 Snake Wizard, exploit. When it exploits
/// a creature, return each creature your opponents control with toughness
/// less than the exploited creature's to its owner's hand.
pub fn profaner_of_the_dead() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::exploit(Effect::WithX {
            x: Value::Diff(Box::new(Value::SacrificedToughness), Box::new(Value::ONE)),
            body: Box::new(Effect::Move {
                what: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByOpponent).and(R::ToughnessAtMostXFromCost),
                ),
                to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..creature("Profaner of the Dead", cost(&[generic(3), u()]), vec![CreatureType::Snake, CreatureType::Wizard], 3, 3)
    }
}

/// Captain America, Skybound — {3}{W}{W} legendary 3/5 Human Soldier Hero;
/// flying, vigilance. Battalion: a +1/+1 counter on each attacking creature,
/// and they gain indestructible until end of turn.
pub fn captain_america_skybound() -> CardDefinition {
    let attackers = || Selector::EachPermanent(R::Creature.and(R::IsAttacking).and(R::ControlledByYou));
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![crate::effect::shortcut::battalion(Effect::Seq(vec![
            Effect::AddCounter { what: attackers(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::GrantKeyword { what: attackers(), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
        ]))],
        ..legend(
            "Captain America, Skybound",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            3,
            5,
        )
    }
}

/// The Mirari Conjecture — {4}{U} Saga. I: return an instant card from your
/// graveyard to your hand. II: the same for a sorcery card. III: until end of
/// turn, copy each instant or sorcery spell you cast (new targets allowed).
pub fn the_mirari_conjecture() -> CardDefinition {
    let regrow = |t| Effect::Move {
        what: target_filtered(R::HasCardType(t).and(R::InYourGraveyard)),
        to: crate::effect::ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        name: "The Mirari Conjecture",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (1, regrow(CardType::Instant)),
            (2, regrow(CardType::Sorcery)),
            (
                3,
                Effect::TriggersThisTurn {
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                            Predicate::EntityMatches {
                                what: Selector::TriggerSource,
                                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                            },
                        ),
                        effect: Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
                    }],
                },
            ),
        ],
        ..Default::default()
    }
}

/// Spider-Gwen, Free Spirit — {2}{R} legendary 2/3 Spider Human Hero, reach.
/// Whenever she becomes tapped, you may discard a card; if you do, draw one.
pub fn spider_gwen_free_spirit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Discard a card to draw a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    crate::effect::shortcut::if_discarded(draw(1)),
                ])),
            },
        }],
        ..legend(
            "Spider-Gwen, Free Spirit",
            cost(&[generic(2), r()]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            2,
            3,
        )
    }
}

/// Spider-Verse — {3}{R}{R} enchantment. The legend rule doesn't apply to
/// Spiders you control. Once each turn, when you cast a spell from anywhere
/// other than your hand, you may copy it (new targets allowed; a permanent
/// copy gains haste).
pub fn spider_verse() -> CardDefinition {
    CardDefinition {
        name: "Spider-Verse",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![crate::card::StaticAbility {
            description: "The \"legend rule\" doesn't apply to Spiders you control.",
            effect: crate::card::StaticEffect::LegendRuleDoesntApplyToYourMatching(R::HasCreatureType(
                CreatureType::Spider,
            )),
        }],
        // "Do this only once each turn": spent when the copy is taken.
        triggered_abilities: vec![crate::effect::shortcut::once_each_turn_on_take(TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::SpellNotCastFromHand }),
            effect: Effect::MayDo {
                description: "Copy that spell?".into(),
                body: Box::new(Effect::CopySpellWithRiders {
                    what: Selector::TriggerSource,
                    count: Value::ONE,
                    grant_haste: true,
                    sacrifice_eot: false,
                    may_choose_new_targets: true,
                }),
            },
        })],
        ..Default::default()
    }
}
