//! Commander: 2026-10-18 (`01MWhJbe`) closers, first batch. Tests in
//! `tests/recent_b/cmdr_01mw_a.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef};
use crate::mana::{cost, g, generic, r, u, w};
use std::sync::Arc;

/// Ixalli's Diviner — {1}{G} 0/3 Human Druid; when it enters, it explores.
pub fn ixallis_diviner() -> CardDefinition {
    CardDefinition {
        name: "Ixalli's Diviner",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Druid], ..Default::default() },
        power: 0,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Explore { who: Selector::This })],
        ..Default::default()
    }
}

/// Origin of Iron Man — {4}{U} Saga. I: tap up to one target creature; it
/// doesn't untap while this Saga remains. II: draw two. III: you may put an
/// artifact card with mana value 5 or less from your hand onto the battlefield.
pub fn origin_of_iron_man() -> CardDefinition {
    CardDefinition {
        name: "Origin of Iron Man",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::TapAndLockWhileSourcePresent {
                        what: Selector::Target(0),
                        while_you_control: false,
                    }),
                },
            ),
            (2, Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
            (
                3,
                Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Artifact.and(R::ManaValueAtMost(5)),
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
            ),
        ],
        ..Default::default()
    }
}

/// Odric, Blood-Cursed — {1}{R}{W} legendary 3/3 Vampire Soldier. When it
/// enters, create X Blood tokens, X = how many of twelve evergreen keywords
/// appear among creatures you control (each counted once).
pub fn odric_blood_cursed() -> CardDefinition {
    use crate::game::effects::blood_token;
    let has = |kw: Keyword| {
        Value::Min(
            Box::new(Value::count(Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::HasKeyword(kw)),
            ))),
            Box::new(Value::ONE),
        )
    };
    let x = Value::Sum(
        [
            Keyword::Flying,
            Keyword::FirstStrike,
            Keyword::DoubleStrike,
            Keyword::Deathtouch,
            Keyword::Haste,
            Keyword::Hexproof,
            Keyword::Indestructible,
            Keyword::Lifelink,
            Keyword::Menace,
            Keyword::Reach,
            Keyword::Trample,
            Keyword::Vigilance,
        ]
        .into_iter()
        .map(has)
        .collect(),
    );
    CardDefinition {
        name: "Odric, Blood-Cursed",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire, CreatureType::Soldier], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: x,
            definition: Arc::new(blood_token()),
        })],
        ..Default::default()
    }
}

/// Kellan, Daring Traveler // Journey On — {1}{W} legendary 2/3 Human Faerie
/// Scout. Attacking reveals your top card: a creature card with mana value 3
/// or less goes to hand, anything else may be milled. Adventure {G} sorcery:
/// create 1 + (opponents who control an artifact) Map tokens.
pub fn kellan_daring_traveler() -> CardDefinition {
    use crate::game::effects::map_token;
    CardDefinition {
        name: "Kellan, Daring Traveler",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Faerie, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::RevealTopAndDrawIf {
                who: PlayerRef::You,
                reveal_filter: R::Creature.and(R::ManaValueAtMost(3)),
                may_graveyard_miss: true,
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Journey On",
            cost: cost(&[g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Sum(vec![
                    Value::ONE,
                    Value::OpponentsControllingAnyOf(Box::new(Selector::EachPermanent(R::Artifact))),
                ]),
                definition: Arc::new(map_token()),
            },
        })),
        ..Default::default()
    }
}

/// Invasion of Regatha // Disciples of the Inferno — {2}{R} Siege, defense 5.
/// ETB: 4 damage to another target battle or opponent and 1 damage to up to
/// one target creature. Back: 4/4 prowess Human Monk; a noncreature source you
/// control deals +2 to a creature, battle, or opponent.
pub fn invasion_of_regatha() -> CardDefinition {
    use crate::card::{BattleSubtype, StaticAbility, StaticEffect};
    let disciples = CardDefinition {
        name: "Disciples of the Inferno",
        card_types: vec![CardType::Creature],
        color_indicator: vec![crate::mana::Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Monk], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Prowess],
        static_abilities: vec![StaticAbility {
            description: "If a noncreature source you control would deal damage to a creature, battle, or opponent, it deals that much damage plus 2 instead.",
            effect: StaticEffect::NoncreatureSourceDamageBonus { amount: 2 },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Regatha",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() },
        defense: 5,
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::HasCardType(CardType::Battle).and(R::OtherThanSource).or(R::OpponentPlayer),
                    },
                    amount: Value::Const(4),
                },
                Effect::DealDamage { to: Selector::TargetFiltered { slot: 1, filter: R::Creature }, amount: Value::ONE },
            ])),
        })],
        back_face: Some(Box::new(disciples)),
        ..Default::default()
    }
}

/// Thorin, Mountain-king — {3}{R} legendary 3/4 trample Dwarf Noble. When it
/// enters, attach any number of target Equipment you control to target
/// creature you control; when one or more become attached this way, that
/// creature deals damage equal to its power to up to one target creature.
pub fn thorin_mountain_king() -> CardDefinition {
    use crate::card::{ArtifactSubtype, Predicate};
    CardDefinition {
        name: "Thorin, Mountain-king",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Noble], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        // CR 601.2c — slot 0 is the creature, slots 1.. the Equipment
        // (`SlotGroups`, the Spinning Wheel Kick shape).
        triggered_abilities: vec![etb(Effect::Seq(vec![
                Effect::SlotGroups(vec![
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 1,
                        filter: R::Creature.and(R::ControlledByYou),
                        effect: Box::new(Effect::Noop),
                    },
                    Effect::ApplyToTargets {
                        max_targets: 15,
                        min_targets: 0,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou),
                        effect: Box::new(Effect::Noop),
                    },
                ]),
                Effect::AttachAnyNumberTo { what: Selector::TargetsFrom(1), to: Selector::Target(0) },
                // CR 603.12 — "When one or more Equipment become attached …
                // this way": a reflexive trigger targeting after the attach;
                // "that creature" is the attached Equipment's host (the carry
                // keeps the moved Equipment).
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::count(Selector::LastMoved), Value::ONE),
                    then: Box::new(Effect::ReflexiveTrigger {
                        body: Box::new(Effect::OptionalTargets {
                            min: 0,
                            body: Box::new(Effect::DealDamageEqualToPower {
                                source: Selector::AttachedTo(Box::new(Selector::LastMoved)),
                                target: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                            }),
                        }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]))],
        ..Default::default()
    }
}

/// Dwarven Hammer — {2}{R} Equipment: +3/+0 and trample; equip {3}. When it
/// enters, you may pay {2}: a 2/1 red Dwarf Berserker, then attach it to that.
pub fn dwarven_hammer() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus, TokenDefinition};
    let berserker = TokenDefinition {
        name: "Dwarf Berserker".into(),
        power: 2,
        toughness: 1,
        colors: vec![crate::mana::Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Berserker], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Dwarven Hammer",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 3, keywords: vec![Keyword::Trample], ..Default::default() }),
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {2} to create a 2/1 Dwarf Berserker and attach Dwarven Hammer to it?".into(),
            mana_cost: cost(&[generic(2)]),
            body: Box::new(Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(berserker) },
                Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
            ])),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Bombur, Gentle Dreamer — {2}{R} legendary 5/3 Dwarf Bard; storied; it
/// doesn't untap during your untap step unless you have an enduring story.
pub fn bombur_gentle_dreamer() -> CardDefinition {
    use crate::card::{Predicate, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Bombur, Gentle Dreamer",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Bard], ..Default::default() },
        power: 5,
        toughness: 3,
        static_abilities: vec![
            StaticAbility {
                description: "Storied (If you control three or more artifacts, legendaries, and/or Sagas, you have an enduring story for the rest of the game.)",
                effect: StaticEffect::Storied,
            },
            StaticAbility {
                description: "Bombur doesn't untap during your untap step unless you have an enduring story.",
                effect: StaticEffect::PreventUntapGlobal {
                    applies_to: Selector::This,
                    condition: Some(Predicate::Not(Box::new(Predicate::HasEnduringStory { who: PlayerRef::You }))),
                },
            },
        ],
        ..Default::default()
    }
}

/// Iron Hills — land, enters tapped; {T}: add {R} or {W}. {2}{R}{W}, {T},
/// sacrifice it: two +1/+1 counters on target Dwarf you control; sorcery speed.
pub fn iron_hills() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use crate::mana::Color;
    let mut d = crate::sets::dual_land_untyped("Iron Hills", Color::Red, Color::White, vec![]);
    d.static_abilities.push(crate::sets::enters_tapped());
    d.activated_abilities.push(ActivatedAbility {
        mana_cost: cost(&[generic(2), r(), w()]),
        tap_cost: true,
        sac_cost: true,
        sorcery_speed: true,
        effect: Effect::AddCounter {
            what: crate::effect::shortcut::target_filtered(
                R::Creature.and(R::HasCreatureType(CreatureType::Dwarf)).and(R::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        },
        ..Default::default()
    });
    d
}

/// Vow to Erebor — {1}{W} instant. Untap target creature you control; it gets
/// +2/+2 until end of turn; if it's a Dwarf, you may attach an Equipment you
/// control to it.
pub fn vow_to_erebor() -> CardDefinition {
    use crate::card::{ArtifactSubtype, Predicate};
    use crate::effect::shortcut::{choose_some_then, chosen_one};
    use crate::effect::Duration;
    let it = || Selector::Target(0);
    CardDefinition {
        name: "Vow to Erebor",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Untap { what: crate::effect::shortcut::target_filtered(R::Creature.and(R::ControlledByYou)), up_to: None },
            Effect::PumpPT { what: it(), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
            Effect::If {
                cond: Predicate::EntityMatches { what: it(), filter: R::HasCreatureType(CreatureType::Dwarf) },
                then: Box::new(choose_some_then(
                    Selector::EachPermanent(R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou)),
                    PlayerRef::You,
                    Value::ONE,
                    true,
                    Effect::Attach { what: chosen_one(), to: it() },
                )),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter — {2}{U}{B}
/// legendary 2/5 flying Robot; More Than Meets the Eye {5}{U}{B}. Combat
/// damage to a player: it connives, then converts at power 5 or more. Back:
/// 5/5 flying living-metal Vehicle; combat damage to a player converts it and
/// adds a beginning phase after this combat (CR 500.8).
pub fn cyclonus_the_saboteur() -> CardDefinition {
    use crate::card::{AlternativeCost, ArtifactSubtype, Predicate};
    let hit = |effect: Effect| TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect,
    };
    let fighter = CardDefinition {
        name: "Cyclonus, Cybertronian Fighter",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        color_indicator: vec![crate::mana::Color::Blue, crate::mana::Color::Black],
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::LivingMetal, Keyword::Flying],
        triggered_abilities: vec![hit(Effect::Seq(vec![
            Effect::Transform { what: Selector::This },
            Effect::AdditionalBeginningPhase { count: Value::ONE },
        ]))],
        ..Default::default()
    };
    CardDefinition {
        name: "Cyclonus, the Saboteur",
        cost: cost(&[generic(2), u(), crate::mana::b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(5), u(), crate::mana::b()]),
            converted: true,
            ..Default::default()
        }),
        triggered_abilities: vec![hit(Effect::Seq(vec![
            Effect::Connive { what: Selector::This, amount: Value::ONE },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::PowerOf(Box::new(Selector::This)), Value::Const(5)),
                then: Box::new(Effect::Transform { what: Selector::This }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        back_face: Some(Box::new(fighter)),
        ..Default::default()
    }
}

/// Graviton, Fundamental Force — {2}{U} legendary 3/3 Human Villain. Whenever
/// you draw your second card each turn, choose one: target creature gains
/// flying until end of turn; or tap target creature.
pub fn graviton_fundamental_force() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    use crate::effect::Duration;
    CardDefinition {
        name: "Graviton, Fundamental Force",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Villain], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
            effect: Effect::ChooseMode(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::Tap { what: target_filtered(R::Creature) },
            ]),
        }],
        ..Default::default()
    }
}

/// Wicked Slumber — {3}{U} instant, convoke. Tap up to two target creatures
/// and put two stun counters among them (one each, or both on a lone target).
pub fn wicked_slumber() -> CardDefinition {
    use crate::card::{CounterType, Predicate};
    let stun = |what: Selector| Effect::AddCounter { what, kind: CounterType::Stun, amount: Value::ONE };
    CardDefinition {
        name: "Wicked Slumber",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Convoke],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![Effect::Tap { what: Selector::Target(0) }, stun(Selector::Target(0))])),
            },
            Effect::If {
                cond: Predicate::ValueAtMost(Value::count(Selector::AllTargets), Value::ONE),
                then: Box::new(stun(Selector::AllTargets)),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Tetsuo, Imperial Champion — {U}{B}{R} legendary 3/3 Human Samurai. When it
/// attacks equipped, choose one, N = the greatest mana value among Equipment
/// attached to it (an Aura's doesn't count): N damage to any target; or cast an
/// instant or sorcery with mana value N or less from your hand free.
pub fn tetsuo_imperial_champion() -> CardDefinition {
    use crate::card::Predicate;
    let n = || {
        Value::HighestManaValueAmong(Box::new(Selector::MatchingAmong {
            inner: Box::new(Selector::AttachedToMe(Box::new(Selector::This))),
            filter: R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment),
        }))
    };
    CardDefinition {
        name: "Tetsuo, Imperial Champion",
        cost: cost(&[u(), crate::mana::b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Samurai], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::IsEquipped }),
            effect: Effect::ChooseMode(vec![
                Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: n() },
                Effect::MayCastFromHandFreeMatching {
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    max_mv: n(),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Unscythe, Killer of Kings — {U}{B}{B}{R} legendary Equipment: +3/+3 and
/// first strike; equip {2}. Whenever a creature dealt damage by the equipped
/// creature this turn dies, you may exile it; if you do, a 2/2 black Zombie.
pub fn unscythe_killer_of_kings() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus, Predicate, TokenDefinition};
    use crate::effect::ZoneDest;
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        colors: vec![crate::mana::Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Unscythe, Killer of Kings",
        cost: cost(&[u(), crate::mana::b(), crate::mana::b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::FirstStrike],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::DamagedThisTurnByMatching(Box::new(R::IsHostOfSource)),
            }),
            effect: Effect::MayDo {
                description: "Exile that card to create a 2/2 Zombie?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(Value::count(Selector::LastMoved), Value::ONE),
                        then: Box::new(Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: Arc::new(zombie),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Thorin, King of Durin's Folk — {3}{R}{W} legendary 4/4 Dwarf Noble. It or
/// another Dwarf you control entering makes a Treasure; other Dwarves you
/// control get +1/+0 for each artifact token you control.
pub fn thorin_king_of_durins_folk() -> CardDefinition {
    use crate::card::{Predicate, StaticAbility, StaticEffect};
    let dwarf = || R::Creature.and(R::HasCreatureType(CreatureType::Dwarf));
    CardDefinition {
        name: "Thorin, King of Durin's Folk",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Noble], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: dwarf() }),
            effect: crate::effect::shortcut::mint_treasures(1),
        }],
        static_abilities: vec![StaticAbility {
            description: "Other Dwarves you control get +1/+0 for each artifact token you control.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::EachPermanent(dwarf().and(R::ControlledByYou).and(R::OtherThanSource)),
                power: Value::count(Selector::EachPermanent(R::Artifact.and(R::IsToken).and(R::ControlledByYou))),
                toughness: Value::ZERO,
            },
        }],
        ..Default::default()
    }
}

/// Gandalf of the Secret Fire — {1}{U}{R}{W} legendary 3/4 Avatar Wizard. An
/// instant or sorcery you cast from your hand during an opponent's turn is
/// exiled with three time counters as it resolves, and gains suspend.
pub fn gandalf_of_the_secret_fire() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Gandalf of the Secret Fire",
        cost: cost(&[generic(1), u(), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Avatar, CreatureType::Wizard], ..Default::default() },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastFromHand,
                Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                },
            ])),
            effect: Effect::SuspendSpellOnResolve { what: Selector::TriggerSource, time_counters: 3 },
        }],
        ..Default::default()
    }
}

/// Araumi of the Dead Tide — {1}{U}{B} legendary 1/4 Merfolk Wizard. {T},
/// exile cards from your graveyard equal to your opponent count: target
/// creature card in your graveyard gains encore until end of turn, its encore
/// cost equal to its mana cost (CR 702.141).
pub fn araumi_of_the_dead_tide() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Araumi of the Dead Tide",
        cost: cost(&[generic(1), u(), crate::mana::b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard], ..Default::default() },
        power: 1,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_other_filter: Some((R::Any, 1)),
            exile_other_count: Some(Value::OpponentCount),
            effect: Effect::GrantEncoreThisTurn {
                what: crate::effect::shortcut::target_filtered(R::Creature.from_your_graveyard()),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Cloning of Shredder — {4}{B}{B} Saga. I: exile target creature card
/// from your graveyard and create a token copy of it. II, III: create a token
/// copy of a card exiled with this Saga. Each copy isn't legendary and is a
/// Mutant in addition to its other types.
pub fn the_cloning_of_shredder() -> CardDefinition {
    use crate::effect::ZoneDest;
    let copy = |source: Selector| Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source,
        extra_creature_types: vec![CreatureType::Mutant],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary: true,
        legendary: false,
        extra_keywords: vec![],
        no_mana_cost: false,
        enters_with_counters: None,
        remove_keywords: vec![],
    };
    CardDefinition {
        name: "The Cloning of Shredder",
        cost: cost(&[generic(4), crate::mana::b(), crate::mana::b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::Move {
                        what: crate::effect::shortcut::target_filtered(R::Creature.from_your_graveyard()),
                        to: ZoneDest::ExileWithSourceStamp,
                    },
                    copy(Selector::LastMoved),
                ]),
            ),
            (2, copy(Selector::CardExiledWithSource)),
            (3, copy(Selector::CardExiledWithSource)),
        ],
        ..Default::default()
    }
}

/// Mind Carver — {B} Equipment; ETB attach it to target creature you control.
/// +1/+0, or +3/+1 instead while an opponent has eight or more cards in their
/// graveyard. Equip {2}{B}.
pub fn mind_carver() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus, Predicate};
    CardDefinition {
        name: "Mind Carver",
        cost: cost(&[crate::mana::b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2), crate::mana::b()]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            conditional_pt: Some((
                2,
                1,
                Predicate::ValueAtLeast(Value::GreatestGraveyardSizeAmong(PlayerRef::EachOpponent), Value::Const(8)),
            )),
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: crate::effect::shortcut::target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        ..Default::default()
    }
}

/// Voltaic Whip — {2}{B} Equipment: +2/+0 and "whenever this creature
/// attacks alone, you draw a card and lose 1 life." Equip {2}.
pub fn voltaic_whip() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus, Predicate};
    CardDefinition {
        name: "Voltaic Whip",
        cost: cost(&[generic(2), crate::mana::b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::AttackingAlone),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                ]),
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Ajani Vengeant — {2}{R}{W} planeswalker, loyalty 3. +1: target permanent
/// doesn't untap during its controller's next untap step. −2: 3 damage to any
/// target and you gain 3 life. −7: destroy all lands target player controls.
pub fn ajani_vengeant() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype};
    use crate::effect::shortcut::{target_any, target_filtered};
    CardDefinition {
        name: "Ajani Vengeant",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Ajani], ..Default::default() },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::SkipNextUntap { what: target_filtered(R::Permanent) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Destroy { what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Land } },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Detritivore — {2}{R}{R} */* Lhurgoyf: power and toughness each equal the
/// nonbasic land cards in your opponents' graveyards. Suspend X—{X}{3}{R},
/// X can't be 0. Whenever a time counter is removed from it while it's
/// exiled, destroy target nonbasic land.
pub fn detritivore() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::effect::ZoneRef;
    let nonbasic = R::Land.and(R::Not(Box::new(R::IsBasicLand)));
    CardDefinition {
        name: "Detritivore",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Lhurgoyf], ..Default::default() },
        keywords: vec![Keyword::Suspend(0, cost(&[crate::mana::x(), generic(3), r()]))],
        static_abilities: vec![StaticAbility {
            description: "Detritivore's power and toughness are each equal to the number of nonbasic land cards in your opponents' graveyards.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::count(Selector::EachMatching {
                    zone: ZoneRef::Graveyard(PlayerRef::EachOpponent),
                    filter: nonbasic.clone(),
                }),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterRemoved(crate::card::CounterType::Time), EventScope::SelfSource)
                .while_suspended(),
            effect: Effect::Destroy { what: crate::effect::shortcut::target_filtered(nonbasic) },
        }],
        ..Default::default()
    }
}

/// Donald Blake, Guise of Thor — {1}{W} legendary 1/3 lifelink Human Doctor.
/// Power-up {4}{W}{W}: two +1/+1 counters and a flying counter; he becomes a
/// God Warrior Hero (CR 702.193).
pub fn donald_blake_guise_of_thor() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use crate::effect::Duration;
    CardDefinition {
        name: "Donald Blake, Guise of Thor",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Doctor], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w(), w()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
                Effect::AddKeywordCounter { what: Selector::This, keyword: Keyword::Flying, amount: Value::ONE },
                Effect::BecomeCreatureType {
                    what: Selector::This,
                    creature_types: vec![CreatureType::God, CreatureType::Warrior, CreatureType::Hero],
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// End Hostilities — {3}{W}{W} sorcery. Destroy all creatures and all
/// permanents attached to creatures.
pub fn end_hostilities() -> CardDefinition {
    CardDefinition {
        name: "End Hostilities",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy { what: Selector::EachPermanent(R::Creature.or(R::AttachedToCreature)) },
        ..Default::default()
    }
}

/// Animating Faerie // Bring to Life — {2}{U} 2/2 flying Faerie. Adventure
/// {2}{U} sorcery: target noncreature artifact you control becomes a 0/0
/// artifact creature with four +1/+1 counters.
pub fn animating_faerie() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::Duration;
    CardDefinition {
        name: "Animating Faerie",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        adventure: Some(Box::new(Adventure {
            name: "Bring to Life",
            cost: cost(&[generic(2), u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::BecomeCreature {
                    what: crate::effect::shortcut::target_filtered(
                        R::Artifact.and(R::Not(Box::new(R::Creature))).and(R::ControlledByYou),
                    ),
                    power: Value::ZERO,
                    toughness: Value::ZERO,
                    creature_types: vec![],
                    keywords: vec![],
                    duration: Duration::Permanent,
                },
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::Const(4) },
            ]),
        })),
        ..Default::default()
    }
}

/// Blinding Spray — {4}{U} instant. Creatures your opponents control get
/// -4/-0 until end of turn. Draw a card.
pub fn blinding_spray() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Blinding Spray",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-4),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Built to Last — {W} instant. Target creature gets +2/+2 until end of turn;
/// an artifact creature also gains indestructible.
pub fn built_to_last() -> CardDefinition {
    use crate::card::Predicate;
    use crate::effect::Duration;
    CardDefinition {
        name: "Built to Last",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: crate::effect::shortcut::target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Artifact },
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Arcee, Sharpshooter // Arcee, Acrobatic Coupe — {1}{R}{W} legendary 2/2
/// first-strike Robot; More Than Meets the Eye {R}{W}. {1}, remove X ≥ 1
/// +1/+1 counters: X damage to target creature, then convert. Back: 2/2
/// living-metal Vehicle; a spell you cast targeting creatures or Vehicles you
/// control puts that many +1/+1 counters on it, then converts it.
pub fn arcee_sharpshooter() -> CardDefinition {
    use crate::card::{ActivatedAbility, AlternativeCost, ArtifactSubtype, CounterType, Predicate};
    let yours = || R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)).and(R::ControlledByYou);
    let coupe = CardDefinition {
        name: "Arcee, Acrobatic Coupe",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        color_indicator: vec![crate::mana::Color::Red, crate::mana::Color::White],
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::LivingMetal],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellTargetsMatch(yours())),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CastSpellTargetsMatching(yours()),
                },
                Effect::Transform { what: Selector::This },
            ]),
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Arcee, Sharpshooter",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        alternative_cost: Some(AlternativeCost { mana_cost: cost(&[r(), w()]), converted: true, ..Default::default() }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_x: Some(CounterType::PlusOnePlusOne),
            x_nonzero: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: crate::effect::shortcut::target_filtered(R::Creature),
                    amount: Value::XFromCost,
                },
                Effect::Transform { what: Selector::This },
            ]),
            ..Default::default()
        }],
        back_face: Some(Box::new(coupe)),
        ..Default::default()
    }
}

/// Dáin of the Ancient Halls — {3}{R}{W} legendary 4/5 vigilance, haste Dwarf
/// Noble. Whenever it attacks, it deals damage equal to the Dwarves you
/// control to each opponent.
pub fn dain_of_the_ancient_halls() -> CardDefinition {
    CardDefinition {
        name: "Dáin of the Ancient Halls",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Noble], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Vigilance, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::count(Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Dwarf)).and(R::ControlledByYou),
                )),
            },
        }],
        ..Default::default()
    }
}

/// Nori, Teller of Tales — {1}{R/W} legendary 2/2 Dwarf Bard. Whenever Nori
/// attacks, target attacking creature gains first strike until end of turn.
pub fn nori_teller_of_tales() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Nori, Teller of Tales",
        cost: cost(&[generic(1), crate::mana::hybrid(crate::mana::Color::Red, crate::mana::Color::White)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Bard], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: crate::effect::shortcut::target_filtered(R::Creature.and(R::IsAttacking)),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Turtle Van — {3} 4/4 Vehicle, crew 1. Whenever it attacks, a +1/+1 counter
/// on target creature that crewed it this turn; then if that creature is a
/// Mutant, Ninja, or Turtle, double its +1/+1 counters.
pub fn turtle_van() -> CardDefinition {
    use crate::card::{ArtifactSubtype, CounterType, Predicate};
    let ninja_turtle = R::HasCreatureType(CreatureType::Mutant)
        .or(R::HasCreatureType(CreatureType::Ninja))
        .or(R::HasCreatureType(CreatureType::Turtle));
    CardDefinition {
        name: "Turtle Van",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: crate::effect::shortcut::target_filtered(R::Creature.and(R::CrewedSourceThisTurn)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::Target(0), filter: ninja_turtle },
                    then: Box::new(Effect::DoubleCountersOnEach {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Zoo Escapees — {1}{G} 2/2 Boar Rhino. When it leaves the battlefield,
/// create a Mutagen token.
pub fn zoo_escapees() -> CardDefinition {
    CardDefinition {
        name: "Zoo Escapees",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Boar, CreatureType::Rhino], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::mutagen_token()),
            },
        }],
        ..Default::default()
    }
}

/// Super Suit — {1}{U} Equipment, flash. When it enters, attach it to target
/// creature you control and untap that creature. +1/+2. Equip {2}.
pub fn super_suit() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Super Suit",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 2, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Attach {
                what: Selector::This,
                to: crate::effect::shortcut::target_filtered(R::Creature.and(R::ControlledByYou)),
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]))],
        ..Default::default()
    }
}

/// Vision Quest — {X}{U}{R} sorcery. Search your library and/or graveyard for
/// an artifact creature card with mana value X or less and put it onto the
/// battlefield with X +1/+1 counters; at X 4 or more it gains haste this turn.
pub fn vision_quest() -> CardDefinition {
    use crate::card::{CounterType, Predicate, Zone};
    use crate::effect::{Duration, ZoneDest};
    CardDefinition {
        name: "Vision Quest",
        cost: cost(&[crate::mana::x(), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SearchZones {
                who: PlayerRef::You,
                zones: vec![Zone::Library, Zone::Graveyard],
                filter: R::Artifact.and(R::Creature).and(R::ManaValueAtMostXFromCost),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::PlusOnePlusOne, amount: Value::XFromCost },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(4)),
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::LastMoved,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Force of Rage — {1}{R}{R} instant; if it's not your turn, you may exile a
/// red card from your hand instead of paying. Two 3/1 red trample, haste
/// Elementals, sacrificed at the beginning of your next upkeep.
pub fn force_of_rage() -> CardDefinition {
    use crate::card::{AlternativeCost, TokenDefinition};
    use crate::mana::Color;
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 3,
        toughness: 1,
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        keywords: vec![Keyword::Trample, Keyword::Haste],
        ..Default::default()
    };
    CardDefinition {
        name: "Force of Rage",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(elemental) },
            Effect::SacrificeAtNextUpkeep { what: Selector::LastCreatedTokens },
        ]),
        alternative_cost: Some(AlternativeCost {
            mana_cost: crate::mana::ManaCost::default(),
            exile_filter: Some(R::HasColor(Color::Red)),
            not_your_turn_only: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Incendiary Flow — {1}{R} sorcery. 3 damage to any target; a creature dealt
/// damage this way that would die this turn is exiled instead.
pub fn incendiary_flow() -> CardDefinition {
    use crate::effect::shortcut::target_any;
    CardDefinition {
        name: "Incendiary Flow",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: target_any() },
            Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Lithomantic Barrage — {R} sorcery; can't be countered. 1 damage to target
/// creature or planeswalker, 5 instead if it's white and/or blue.
pub fn lithomantic_barrage() -> CardDefinition {
    use crate::card::Predicate;
    use crate::mana::Color;
    let target = || crate::effect::shortcut::target_filtered(R::Creature.or(R::Planeswalker));
    CardDefinition {
        name: "Lithomantic Barrage",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: R::HasColor(Color::White).or(R::HasColor(Color::Blue)),
            },
            then: Box::new(Effect::DealDamage { to: target(), amount: Value::Const(5) }),
            else_: Box::new(Effect::DealDamage { to: target(), amount: Value::ONE }),
        },
        ..Default::default()
    }
}

/// Misty Mountains Raider — {4}{R} 4/4 Goblin Soldier. Whenever you attack,
/// amass Goblins 2.
pub fn misty_mountains_raider() -> CardDefinition {
    CardDefinition {
        name: "Misty Mountains Raider",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin, CreatureType::Soldier], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::Amass { who: PlayerRef::You, count: Value::Const(2), extra_type: Some(CreatureType::Goblin) },
        }],
        ..Default::default()
    }
}

/// Tidings of War — {R} sorcery. Amass Goblins 1, or 3 if cast from a
/// graveyard. Flashback {3}{R}.
pub fn tidings_of_war() -> CardDefinition {
    use crate::card::Predicate;
    let amass = |n: i32| Effect::Amass { who: PlayerRef::You, count: Value::Const(n), extra_type: Some(CreatureType::Goblin) };
    CardDefinition {
        name: "Tidings of War",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), r()]))],
        effect: Effect::If { cond: Predicate::CastFromGraveyard, then: Box::new(amass(3)), else_: Box::new(amass(1)) },
        ..Default::default()
    }
}

/// Warbeast of Gorgoroth — {4}{R} 5/4 Beast. Whenever it or another creature
/// you control with power 4 or greater dies, amass Orcs 2.
pub fn warbeast_of_gorgoroth() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Warbeast of Gorgoroth",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::PowerAtLeast(4) }),
            effect: Effect::Amass { who: PlayerRef::You, count: Value::Const(2), extra_type: Some(CreatureType::Orc) },
        }],
        ..Default::default()
    }
}

/// Giant's Ire — {3}{R} Kindred sorcery — Giant. 4 damage to target player or
/// planeswalker; if you control a Giant, draw a card.
pub fn giants_ire() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Giant's Ire",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: crate::effect::shortcut::target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::Const(4),
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Giant),
                }),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Squash — {4}{R} instant; {3} less if you control a Giant. 6 damage to
/// target creature or planeswalker.
pub fn squash() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Squash",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {3} less to cast if you control a Giant.",
            effect: StaticEffect::SelfCostReducedIfControlEach {
                filters: vec![R::HasCreatureType(CreatureType::Giant)],
                amount: 3,
            },
        }],
        effect: Effect::DealDamage {
            to: crate::effect::shortcut::target_filtered(R::Creature.or(R::Planeswalker)),
            amount: Value::Const(6),
        },
        ..Default::default()
    }
}

/// Surtland Flinger — {3}{R}{R} 4/6 Giant Berserker. Attacking, you may
/// sacrifice another creature; when you do, it deals damage equal to that
/// creature's power to any target — twice that if it was a Giant.
pub fn surtland_flinger() -> CardDefinition {
    use crate::card::Predicate;
    use crate::effect::shortcut::target_any;
    let fling = |amount: Value| Effect::DealDamage { to: target_any(), amount };
    CardDefinition {
        name: "Surtland Flinger",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant, CreatureType::Berserker], ..Default::default() },
        power: 4,
        toughness: 6,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MaySacrifice {
                description: "Sacrifice another creature to fling it?".into(),
                filter: R::Creature.and(R::OtherThanSource).and(R::ControlledByYou),
                count: Value::ONE,
                then: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::SacrificedCard,
                            filter: R::HasCreatureType(CreatureType::Giant),
                        },
                        then: Box::new(fling(Value::Sum(vec![Value::SacrificedPower, Value::SacrificedPower]))),
                        else_: Box::new(fling(Value::SacrificedPower)),
                    }),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Arcane Infusion — {U}{R} instant. Look at the top four; you may reveal an
/// instant or sorcery among them and take it; the rest go to the bottom in a
/// random order. Flashback {3}{U}{R}.
pub fn arcane_infusion() -> CardDefinition {
    use crate::effect::LookPick;
    CardDefinition {
        name: "Arcane Infusion",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), u(), r()]))],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Hypothesizzle — {3}{U}{R} instant. Draw two; then you may discard a
/// nonland card, and when you do, 4 damage to target creature.
pub fn hypothesizzle() -> CardDefinition {
    CardDefinition {
        name: "Hypothesizzle",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::MayDiscardMatching {
                description: "Discard a nonland card for 4 damage to a creature?".into(),
                count: Value::ONE,
                filter: R::Nonland,
                then: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::DealDamage {
                        to: crate::effect::shortcut::target_filtered(R::Creature),
                        amount: Value::Const(4),
                    }),
                }),
                else_: None,
            },
        ]),
        ..Default::default()
    }
}

/// Master the Way — {3}{U}{R} sorcery. Draw a card, then deal damage equal to
/// the cards in your hand to any target.
pub fn master_the_way() -> CardDefinition {
    CardDefinition {
        name: "Master the Way",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::HandSizeOf(PlayerRef::You) },
        ]),
        ..Default::default()
    }
}

fn wall(name: &'static str, mana: crate::mana::ManaCost, toughness: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness,
        keywords: vec![Keyword::Defender],
        ..Default::default()
    }
}

/// Glacial Wall — {2}{U} 0/7 Wall, defender.
pub fn glacial_wall() -> CardDefinition {
    wall("Glacial Wall", cost(&[generic(2), u()]), 7)
}

/// Wall of Mist — {1}{U} 0/5 Wall, defender.
pub fn wall_of_mist() -> CardDefinition {
    wall("Wall of Mist", cost(&[generic(1), u()]), 5)
}

/// Biolume Egg // Biolume Serpent — {2}{U} 0/4 defender Serpent Egg; ETB
/// scry 2; when you sacrifice it, it returns transformed at the beginning of
/// the next end step. Back: 4/4 Serpent; sacrifice two Islands: it can't be
/// blocked this turn.
pub fn biolume_egg() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    use crate::effect::Duration;
    let serpent = CardDefinition {
        name: "Biolume Serpent",
        card_types: vec![CardType::Creature],
        color_indicator: vec![crate::mana::Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Serpent], ..Default::default() },
        power: 4,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasLandType(LandType::Island), 2)),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Biolume Egg",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Serpent, CreatureType::Egg], ..Default::default() },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![
            etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::SelfSource),
                effect: Effect::AtNextEndStep { body: Box::new(Effect::ExileSelfReturnTransformed) },
            },
        ],
        back_face: Some(Box::new(serpent)),
        ..Default::default()
    }
}

/// Divine Congregation — {3}{W} sorcery. You gain 2 life for each creature
/// target player controls. Suspend 5—{1}{W}.
pub fn divine_congregation() -> CardDefinition {
    CardDefinition {
        name: "Divine Congregation",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Suspend(5, cost(&[generic(1), w()]))],
        effect: Effect::GainLife {
            who: Selector::You,
            amount: Value::Sum(vec![
                Value::count(Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature }),
                Value::count(Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature }),
            ]),
        },
        ..Default::default()
    }
}

/// Ivory Giant — {5}{W}{W} 3/4 Giant. When it enters, tap all nonwhite
/// creatures. Suspend 5—{W}.
pub fn ivory_giant() -> CardDefinition {
    CardDefinition {
        name: "Ivory Giant",
        cost: cost(&[generic(5), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Suspend(5, cost(&[w()]))],
        triggered_abilities: vec![etb(Effect::Tap {
            what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasColor(crate::mana::Color::White))))),
        })],
        ..Default::default()
    }
}

/// Pardic Dragon — {4}{R}{R} 4/4 flying Dragon; {R}: +1/+0 until end of turn.
/// Suspend 2—{R}{R}. Whenever an opponent casts a spell while it's suspended,
/// that player may put a time counter on it.
pub fn pardic_dragon() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use crate::effect::Duration;
    CardDefinition {
        name: "Pardic Dragon",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Suspend(2, cost(&[r(), r()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ZERO, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).while_suspended(),
            effect: Effect::MayDoBy {
                who: PlayerRef::Triggerer,
                description: "Put a time counter on the suspended Pardic Dragon?".into(),
                body: Box::new(Effect::AddCounter { what: Selector::This, kind: CounterType::Time, amount: Value::ONE }),
            },
        }],
        ..Default::default()
    }
}

fn reach_spider(name: &'static str, toughness: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 1,
        toughness,
        keywords: vec![Keyword::Reach],
        ..Default::default()
    }
}

/// Hitchclaw Recluse — {2}{G} 1/4 reach Spider.
pub fn hitchclaw_recluse() -> CardDefinition {
    reach_spider("Hitchclaw Recluse", 4)
}

/// Sporecap Spider — {2}{G} 1/5 reach Spider.
pub fn sporecap_spider() -> CardDefinition {
    reach_spider("Sporecap Spider", 5)
}

/// Tadeas, Juniper Ascendant — {2}{G}{W} legendary 1/3 reach Human Monk;
/// hexproof unless attacking. A reach creature of yours attacking untaps and
/// gains skulk this combat. One or more of your creatures dealing combat
/// damage to a player draws a card.
pub fn tadeas_juniper_ascendant() -> CardDefinition {
    use crate::card::{Predicate, StaticAbility, StaticEffect};
    use crate::effect::Duration;
    CardDefinition {
        name: "Tadeas, Juniper Ascendant",
        cost: cost(&[generic(2), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Monk], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Tadeas has hexproof unless it's attacking.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsAttacking,
                })),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Hexproof],
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasKeyword(Keyword::Reach),
                }),
                effect: Effect::Seq(vec![
                    Effect::Untap { what: Selector::TriggerSource, up_to: None },
                    Effect::GrantKeyword { what: Selector::TriggerSource, keyword: Keyword::Skulk, duration: Duration::EndOfCombat },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch(),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Super Strength — {4}{G} Aura: enchanted creature gets +4/+4 and has
/// trample and ward {1}.
pub fn super_strength() -> CardDefinition {
    use crate::card::{EquipBonus, WardCost};
    CardDefinition {
        name: "Super Strength",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: crate::effect::shortcut::target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 4,
            toughness: 4,
            keywords: vec![Keyword::Trample, Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Thing Swing — {3}{G} instant; {2} less if it targets a Hero you control.
/// Target creature you control deals damage equal to twice its power to
/// target creature an opponent controls.
pub fn thing_swing() -> CardDefinition {
    let power = || Value::PowerOf(Box::new(Selector::Target(0)));
    CardDefinition {
        name: "Thing Swing",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((R::HasCreatureType(CreatureType::Hero).and(R::ControlledByYou), 2)),
        effect: Effect::DealDamageFrom {
            source: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
            to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
            amount: Value::Sum(vec![power(), power()]),
        },
        ..Default::default()
    }
}

/// Wakandan Royal Guard — {4}{G} 4/4 vigilance Human Soldier Hero. When it
/// enters, a +1/+1 counter on target creature — two if it's another Hero.
pub fn wakandan_royal_guard() -> CardDefinition {
    use crate::card::{CounterType, Predicate};
    let counters = |n: i32| Effect::AddCounter {
        what: crate::effect::shortcut::target_filtered(R::Creature),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
    };
    CardDefinition {
        name: "Wakandan Royal Guard",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: R::HasCreatureType(CreatureType::Hero).and(R::OtherThanSource),
            },
            then: Box::new(counters(2)),
            else_: Box::new(counters(1)),
        })],
        ..Default::default()
    }
}

/// Arc Spitter — {1} Equipment: equipped creature has "{1}: this creature
/// deals 1 damage to target creature that's blocking it." Equip {1}.
pub fn arc_spitter() -> CardDefinition {
    use crate::card::{ActivatedAbility, ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Arc Spitter",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::DealDamage {
                    to: crate::effect::shortcut::target_filtered(R::Creature.and(R::BlockingOrBlockedBySource)),
                    amount: Value::ONE,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Goblin Javelineer — {R} 1/1 haste Goblin Warrior. Whenever it becomes
/// blocked, it deals 1 damage to target creature blocking it.
pub fn goblin_javelineer() -> CardDefinition {
    CardDefinition {
        name: "Goblin Javelineer",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin, CreatureType::Warrior], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: crate::effect::shortcut::target_filtered(R::Creature.and(R::BlockingOrBlockedBySource)),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Tormentor's Helm — {R} Equipment: +1/+1; whenever the equipped creature
/// becomes blocked, it deals 1 damage to the defending player. Equip {1}.
pub fn tormentors_helm() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Tormentor's Helm",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::ONE },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn wolf_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Wolf".into(),
        power: 2,
        toughness: 2,
        colors: vec![crate::mana::Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        ..Default::default()
    }
}

/// Pack Guardian — {2}{G}{G} 4/3 flash Wolf Spirit. When it enters, you may
/// discard a land card; if you do, a 2/2 green Wolf.
pub fn pack_guardian() -> CardDefinition {
    CardDefinition {
        name: "Pack Guardian",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf, CreatureType::Spirit], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::MayDiscardMatching {
            description: "Discard a land card to create a 2/2 Wolf?".into(),
            count: Value::ONE,
            filter: R::Land,
            then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(wolf_token()) }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Witchstalker — {1}{G}{G} 3/3 hexproof Wolf. Whenever an opponent casts a
/// blue or black spell during your turn, a +1/+1 counter on it.
pub fn witchstalker() -> CardDefinition {
    use crate::card::{CounterType, Predicate};
    use crate::mana::Color;
    CardDefinition {
        name: "Witchstalker",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Hexproof],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::Blue).or(R::HasColor(Color::Black)),
                },
            ])),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Primal Adversary — {2}{G} 4/3 trample Wolf. When it enters, you may pay
/// {1}{G} any number of times; when you pay one or more times, a reflexive
/// trigger (CR 603.12) puts that many +1/+1 counters on it, then up to that
/// many target lands you control become 3/3 haste Wolf creatures that are
/// still lands (the Bloodthirsty Adversary shape).
pub fn primal_adversary() -> CardDefinition {
    use crate::card::{CounterType, Predicate};
    use crate::effect::Duration;
    CardDefinition {
        name: "Primal Adversary",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::MayPayRepeatedly {
                who: PlayerRef::You,
                description: "Pay {1}{G} (again) for Primal Adversary?".into(),
                mana_cost: cost(&[generic(1), g()]),
                body: Box::new(Effect::Noop),
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::TimesPaidThisEffect, Value::ONE),
                then: Box::new(Effect::WithX {
                    x: Value::TimesPaidThisEffect,
                    body: Box::new(Effect::ReflexiveTrigger {
                        body: Box::new(Effect::Seq(vec![
                            Effect::AddCounter {
                                what: Selector::This,
                                kind: CounterType::PlusOnePlusOne,
                                amount: Value::XFromCost,
                            },
                            Effect::CapTargetsAtX {
                                body: Box::new(Effect::ApplyToTargets {
                                    max_targets: 8,
                                    min_targets: 0,
                                    filter: R::Land.and(R::ControlledByYou),
                                    effect: Box::new(Effect::BecomeCreature {
                                        what: Selector::Target(0),
                                        power: Value::Const(3),
                                        toughness: Value::Const(3),
                                        creature_types: vec![CreatureType::Wolf],
                                        keywords: vec![Keyword::Haste],
                                        duration: Duration::Permanent,
                                    }),
                                }),
                            },
                        ])),
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}

/// Somberwald Alpha — {3}{G} 3/2 Wolf. A creature you control becoming
/// blocked gets +1/+1 until end of turn. {1}{G}: target creature you control
/// gains trample until end of turn.
pub fn somberwald_alpha() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::Duration;
    CardDefinition {
        name: "Somberwald Alpha",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::GrantKeyword {
                what: crate::effect::shortcut::target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Beetle, Legacy Criminal — {3}{U} legendary 3/3 flying Human Rogue Villain.
/// {1}{U}, exile it from your graveyard: a +1/+1 counter on target creature,
/// which gains flying until end of turn; sorcery speed.
pub fn beetle_legacy_criminal() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use crate::effect::Duration;
    CardDefinition {
        name: "Beetle, Legacy Criminal",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Villain],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: crate::effect::shortcut::target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ghostform — {1}{U} sorcery. Up to two target creatures can't be blocked
/// this turn.
pub fn ghostform() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Ghostform",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Kolaghan Forerunners — {2}{R} */3 trample Human Berserker; power equals
/// the creatures you control. Dash {2}{R}.
pub fn kolaghan_forerunners() -> CardDefinition {
    use crate::card::{AlternativeCost, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Kolaghan Forerunners",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Berserker], ..Default::default() },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Kolaghan Forerunners's power is equal to the number of creatures you control.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::count(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        alternative_cost: Some(AlternativeCost { mana_cost: cost(&[generic(2), r()]), dash: true, ..Default::default() }),
        ..Default::default()
    }
}

/// Whirlwind, Killer Cyclone — {2}{R} legendary 2/3 haste Mutant Villain.
/// Whenever one or more creatures you control that entered this turn attack a
/// player, target creature that player controls can't block this turn.
pub fn whirlwind_killer_cyclone() -> CardDefinition {
    use crate::card::Predicate;
    use crate::effect::Duration;
    CardDefinition {
        name: "Whirlwind, Killer Cyclone",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mutant, CreatureType::Villain], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::EnteredThisTurn.and(R::IsAttackingOpponentPlayer),
                })
                .once_per_batch(),
            effect: Effect::GrantKeyword {
                what: crate::effect::shortcut::target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Hulk, Gamma Goliath — {3}{R}{G} legendary 6/5 reach, trample Gamma
/// Berserker Hero. Power-up abilities of your other creatures cost {3} less.
/// Power-up {6}{R}{G}: five +1/+1 counters (CR 702.193).
pub fn hulk_gamma_goliath() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Hulk, Gamma Goliath",
        cost: cost(&[generic(3), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gamma, CreatureType::Berserker, CreatureType::Hero],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Power-up abilities of other creatures you control cost {3} less to activate.",
            effect: StaticEffect::OtherPowerUpsCostLess { amount: 3 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), r(), g()]),
            power_up: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(5) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Abomination, Terrifying Titan — {3}{R/G} legendary 4/4 trample Gamma
/// Villain. Power-up {5}{R/G}{R/G}: a +1/+1 counter, then it fights up to one
/// target creature an opponent controls.
pub fn abomination_terrifying_titan() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use crate::mana::{hybrid, Color};
    let rg = || hybrid(Color::Red, Color::Green);
    CardDefinition {
        name: "Abomination, Terrifying Titan",
        cost: cost(&[generic(3), rg()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gamma, CreatureType::Villain], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), rg(), rg()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Fight {
                        attacker: Selector::This,
                        defender: crate::effect::shortcut::target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Team Tactics — {1}{R} instant. Teamwork 1. Target creature gains double
/// strike until end of turn, and trample too if teamwork was paid.
pub fn team_tactics() -> CardDefinition {
    use crate::card::{AdditionalCastCost, Predicate};
    use crate::effect::Duration;
    CardDefinition {
        name: "Team Tactics",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower { filter: R::Creature.and(R::ControlledByYou), power: 1 }),
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: crate::effect::shortcut::target_filtered(R::Creature),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Countersculpt — {U}{U} instant; as an additional cost, behold a Jace or pay
/// {1}. Counter target spell. Empower Jace 1.
pub fn countersculpt() -> CardDefinition {
    use crate::card::{AdditionalCastCost, PlaneswalkerSubtype};
    CardDefinition {
        name: "Countersculpt",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::BeholdOrPay {
            filter: R::HasPlaneswalkerType(PlaneswalkerSubtype::Jace),
            pay: 1,
        }],
        effect: Effect::Seq(vec![
            crate::effect::shortcut::counter_target_spell(),
            Effect::EmpowerJace { who: PlayerRef::You, count: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Alhammarret, High Arbiter — {5}{U}{U} legendary 5/5 flying Sphinx. As it
/// enters, name a nonland card; your opponents can't cast spells with that
/// name while it's on the battlefield.
pub fn alhammarret_high_arbiter() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Alhammarret, High Arbiter",
        cost: cost(&[generic(5), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sphinx], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        as_enters_effect: Some(Effect::NameCard { what: Selector::This, restrict_to: Some(R::Nonland) }),
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't cast spells with the chosen name.",
            effect: StaticEffect::OpponentsCantCastNamed,
        }],
        ..Default::default()
    }
}

/// Infinite Coursework — {2}{U} Aura. When it enters, tap the enchanted
/// creature; it becomes unprepared. The enchanted creature loses all
/// abilities and doesn't untap during its controller's untap step.
pub fn infinite_coursework() -> CardDefinition {
    use crate::card::{CounterType, EquipBonus, StaticAbility, StaticEffect};
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Infinite Coursework",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: crate::effect::shortcut::target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { remove_abilities: true, ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntapGlobal { applies_to: host(), condition: None },
        }],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: host() },
            Effect::RemoveCounter {
                what: host(),
                kind: CounterType::Prepared,
                amount: Value::CountersOn { what: Box::new(host()), kind: CounterType::Prepared },
            },
        ]))],
        ..Default::default()
    }
}

/// Madame Web, Clairvoyant — {4}{U}{U} legendary 4/4 Mutant Advisor. Look at
/// your top card any time; cast Spider spells and noncreature spells from it.
/// Whenever you attack, you may mill a card.
pub fn madame_web_clairvoyant() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Madame Web, Clairvoyant",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mutant, CreatureType::Advisor], ..Default::default() },
        power: 4,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may cast Spider spells and noncreature spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop {
                    filter: R::HasCreatureType(CreatureType::Spider).or(R::Not(Box::new(R::Creature))).and(R::Nonland),
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Mill a card?".into(),
                body: Box::new(Effect::Mill { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..Default::default()
    }
}

/// Jetfire, Ingenious Scientist // Jetfire, Air Guardian — {4}{U} legendary
/// 3/4 flying Robot; More Than Meets the Eye {3}{U}. Remove X ≥ 1 +1/+1
/// counters from among your artifacts: target player adds X {C} that can't
/// cast nonartifact spells, then convert. Back: 3/4 flying living-metal Vehicle; {U}{U}{U}:
/// convert, then adapt 3.
pub fn jetfire_ingenious_scientist() -> CardDefinition {
    use crate::card::{ActivatedAbility, AlternativeCost, ArtifactSubtype, CounterType};
    use crate::effect::ManaPayload;
    let guardian = CardDefinition {
        name: "Jetfire, Air Guardian",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        color_indicator: vec![crate::mana::Color::Blue],
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::LivingMetal, Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u(), u()]),
            effect: Effect::Seq(vec![
                Effect::Transform { what: Selector::This },
                crate::effect::shortcut::adapt(3),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Jetfire, Ingenious Scientist",
        cost: cost(&[generic(4), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(AlternativeCost { mana_cost: cost(&[generic(3), u()]), converted: true, ..Default::default() }),
        activated_abilities: vec![ActivatedAbility {
            remove_counter_among_x: Some((Some(CounterType::PlusOnePlusOne), R::Artifact.and(R::ControlledByYou))),
            x_nonzero: true,
            // "Target player adds that much {C}": targeted, so not a mana
            // ability (CR 605.1a) — it uses the stack.
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::Target(0),
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colorless(Value::XFromCost)),
                        crate::mana::SpendRestriction::NoNonartifactSpells,
                    ),
                },
                Effect::Transform { what: Selector::This },
            ]),
            ..Default::default()
        }],
        back_face: Some(Box::new(guardian)),
        ..Default::default()
    }
}

/// Arbor Armament — {G} instant. A +1/+1 counter on target creature; it gains
/// reach until end of turn.
pub fn arbor_armament() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::Duration;
    CardDefinition {
        name: "Arbor Armament",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: crate::effect::shortcut::target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Reach, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Fairgrounds Trumpeter — {2}{G} 2/2 Elephant. At the beginning of each end
/// step, if you put a +1/+1 counter on a permanent you control this turn, a
/// +1/+1 counter on it.
pub fn fairgrounds_trumpeter() -> CardDefinition {
    use crate::card::{CounterType, Predicate};
    use crate::game::TurnStep;
    CardDefinition {
        name: "Fairgrounds Trumpeter",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elephant], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                Predicate::ValueAtLeast(Value::PlusOneCountersPutOnYourCreaturesThisTurn(PlayerRef::You), Value::ONE),
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Graft Surgeon — {2}{W} 2/2 Human Cleric; enters with a +1/+1 counter. When
/// it dies, put its counters on up to one target creature you control.
pub fn graft_surgeon() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Graft Surgeon",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Cleric], ..Default::default() },
        power: 2,
        toughness: 2,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::MoveAllCounters {
                from: Selector::This,
                to: crate::effect::shortcut::target_filtered(R::Creature.and(R::ControlledByYou)),
            }),
        })],
        ..Default::default()
    }
}

/// Predictive Preparations — {1}{W} sorcery. A +1/+1 counter on each of one or
/// two target creatures. Flashback {3}{W}.
pub fn predictive_preparations() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Predictive Preparations",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), w()]))],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 1,
            filter: R::Creature,
            effect: Box::new(Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
        },
        ..Default::default()
    }
}

/// Alex Wilder, Runaway — {1}{R} legendary 1/3 Human Hero Villain. It or
/// another creature of yours entering, if cast from anywhere but your hand,
/// gets +2/+0 and haste until end of turn — the command zone counts (CR 903.8).
/// Escape—{2}{R}, exile three others.
pub fn alex_wilder_runaway() -> CardDefinition {
    use crate::card::Predicate;
    use crate::effect::Duration;
    CardDefinition {
        name: "Alex Wilder, Runaway",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Hero, CreatureType::Villain],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Escape(cost(&[generic(2), r()]), 3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::EnteredByCast).and(R::Not(Box::new(R::WasCastFromHand))),
            }),
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: Selector::TriggerSource, power: Value::Const(2), toughness: Value::ZERO, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::TriggerSource, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            ]),
        }],
        ..Default::default()
    }
}

/// Invasion of Karsus // Refraction Elemental — {2}{R}{R} Siege, defense 4.
/// ETB: 3 damage to each creature and each planeswalker. Back: 4/4 Elemental,
/// ward—pay 2 life; whenever you cast a spell, 2 damage to each opponent.
pub fn invasion_of_karsus() -> CardDefinition {
    use crate::card::{BattleSubtype, WardCost};
    let elemental = CardDefinition {
        name: "Refraction Elemental",
        card_types: vec![CardType::Creature],
        color_indicator: vec![crate::mana::Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Ward(WardCost::Life(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Karsus",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() },
        defense: 4,
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.or(R::Planeswalker)),
            amount: Value::Const(3),
        })],
        back_face: Some(Box::new(elemental)),
        ..Default::default()
    }
}

/// Druid of the Emerald Grove — {3}{G} 2/2 Dwarf Druid. When it enters, search
/// for up to two basic lands, then roll a d20: 1-9 both to hand; 10-19 one onto
/// the battlefield tapped, the other to hand; 20 both onto the battlefield
/// tapped.
pub fn druid_of_the_emerald_grove() -> CardDefinition {
    use crate::effect::ZoneDest;
    let basic = || R::Land.and(R::IsBasicLand);
    let to_hand = || Effect::Search { who: PlayerRef::You, filter: basic(), to: ZoneDest::Hand(PlayerRef::You) };
    let to_bf = || Effect::Search {
        who: PlayerRef::You,
        filter: basic(),
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
    };
    let one_search = |a, b| Effect::OneSearch { body: Box::new(Effect::Seq(vec![a, b])) };
    CardDefinition {
        name: "Druid of the Emerald Grove",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Druid], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::RollDie {
            sides: 20,
            count: Value::ONE,
            modifier: Value::ZERO,
            reroll_at_most: 0,
            ignore_lowest: 0,
            results: vec![
                (1, 9, one_search(to_hand(), to_hand())),
                (10, 19, one_search(to_bf(), to_hand())),
                (20, 20, one_search(to_bf(), to_bf())),
            ],
            on_doubles: None,
        })],
        ..Default::default()
    }
}

/// The Lady of Otaria — {3}{R}{G} legendary 5/5 Avatar; you may tap three
/// untapped Dwarves you control instead of paying its mana cost. At the
/// beginning of each end step, if a land you controlled went to a graveyard
/// from the battlefield this turn, reveal the top four and take any number of
/// Dwarf cards; the rest go to the bottom in a random order.
pub fn the_lady_of_otaria() -> CardDefinition {
    use crate::card::{AlternativeCost, Predicate};
    use crate::effect::{LookPick, ZoneRef};
    use crate::game::TurnStep;
    let dwarf = || R::HasCreatureType(CreatureType::Dwarf);
    CardDefinition {
        name: "The Lady of Otaria",
        cost: cost(&[generic(3), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Avatar], ..Default::default() },
        power: 5,
        toughness: 5,
        alternative_cost: Some(AlternativeCost {
            mana_cost: crate::mana::ManaCost::default(),
            tap_creatures: Some((R::Creature.and(dwarf()), 3)),
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                Predicate::SelectorExists(Selector::EachMatching {
                    zone: ZoneRef::Graveyard(PlayerRef::You),
                    filter: R::Land.and(R::PutIntoGraveyardFromBattlefieldThisTurn),
                }),
            ),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(4),
                pick_filter: Some(dwarf()),
                take: Some(Value::Const(4)),
                optional: true,
                rest_bottom_random: true,
                ..Default::default()
            })),
        }],
        ..Default::default()
    }
}
