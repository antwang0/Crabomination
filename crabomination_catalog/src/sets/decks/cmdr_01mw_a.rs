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
/// enters, attach any number of Equipment you control to target creature you
/// control; if one or more became attached, that creature deals damage equal
/// to its power to up to one target creature.
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
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::AttachAnyNumberTo {
                    what: Selector::EachPermanent(
                        R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou),
                    ),
                    to: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::count(Selector::LastMoved), Value::ONE),
                    then: Box::new(Effect::DealDamageEqualToPower {
                        source: Selector::Target(0),
                        target: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
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
    use crate::card::Predicate;
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
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::Triggerer, n: 2 })
                .once_per_turn(),
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
/// attacks equipped, choose one, N = the greatest mana value among what's
/// attached to it: N damage to any target; or cast an instant or sorcery with
/// mana value N or less from your hand free.
pub fn tetsuo_imperial_champion() -> CardDefinition {
    use crate::card::Predicate;
    let n = || Value::HighestManaValueAmong(Box::new(Selector::AttachedToMe(Box::new(Selector::This))));
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
