//! Commander: three-card gaps of EDHREC's commanders ranked 4501-5500 (the
//! 2026-10-14 rescan, after `cmdr_rank5k_c`). Tests in
//! `tests/recent_b/cmdr_rank5k_d.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, sneak, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, cost, generic, r, u, w};
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

fn noncreature_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::Not(Box::new(R::Creature)))),
        effect,
    }
}

/// Smaug's Fury — {1}{R} instant. Target creature gets +3/+0 and gains reach
/// and first strike until end of turn.
pub fn smaugs_fury() -> CardDefinition {
    CardDefinition {
        name: "Smaug's Fury",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Reach, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Looming Altisaur — {3}{W} 1/7 Dinosaur.
pub fn looming_altisaur() -> CardDefinition {
    creature("Looming Altisaur", cost(&[generic(3), w()]), vec![CreatureType::Dinosaur], 1, 7)
}

/// Blue Marvel, Adam Brashear — {3}{U}{U} legendary 3/5 Human Scientist Hero;
/// flying, ward {2}. Whenever you draw your second card each turn, a +1/+1
/// counter on him.
pub fn blue_marvel_adam_brashear() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legend(
            "Blue Marvel, Adam Brashear",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Hero],
            3,
            5,
        )
    }
}

/// Giant-Sized Flying Ant — {3}{U} 3/2 Insect; flash, flying. When it enters,
/// choose one — tap target nonland permanent; or untap one.
pub fn giant_sized_flying_ant() -> CardDefinition {
    let nonland = || target_filtered(R::Permanent.and(R::Nonland));
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Tap { what: nonland() },
            Effect::Untap { what: nonland(), up_to: None },
        ]))],
        ..creature("Giant-Sized Flying Ant", cost(&[generic(3), u()]), vec![CreatureType::Insect], 3, 2)
    }
}

/// The Immortal Weapons — {4}{R} legendary 4/4 Human Warrior Hero. When they
/// enter, return an instant or sorcery card from your graveyard to your hand.
/// Whenever you cast a noncreature spell, target creature gets +2/+0 and
/// menace until end of turn.
pub fn the_immortal_weapons() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Move {
                what: target_filtered(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::InYourGraveyard),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            noncreature_cast(Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Menace, duration: Duration::EndOfTurn },
            ])),
        ],
        ..legend(
            "The Immortal Weapons",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero],
            4,
            4,
        )
    }
}

/// Flock Impostor — {2}{W} 2/2 Shapeshifter; changeling, flash, flying. When
/// it enters, return up to one other target creature you control to its
/// owner's hand.
pub fn flock_impostor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling, Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..creature("Flock Impostor", cost(&[generic(2), w()]), vec![CreatureType::Shapeshifter], 2, 2)
    }
}

/// Triumphant Reckoning — {6}{W}{W}{W} sorcery. Return all artifact,
/// enchantment, and planeswalker cards from your graveyard to the battlefield.
pub fn triumphant_reckoning() -> CardDefinition {
    CardDefinition {
        name: "Triumphant Reckoning",
        cost: cost(&[generic(6), w(), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ReturnAllMatchingFromGraveyardToBattlefield {
            who: PlayerRef::You,
            filter: R::Artifact.or(R::Enchantment).or(R::Planeswalker),
            sacrifice_eot: false,
        },
        ..Default::default()
    }
}

/// Surveillance Room — land. It enters and you surveil 1. {T}: {C}. {1}, {T}:
/// one mana of any color.
pub fn surveillance_room() -> CardDefinition {
    CardDefinition {
        name: "Surveillance Room",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE })],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Iron Fist, Hero for Hire — {1}{R} legendary 2/2 Human Mercenary Hero;
/// trample, prowess. Power-up — {7}{R}: 5 damage divided among up to five
/// targets, then five +1/+1 counters on him.
pub fn iron_fist_hero_for_hire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Prowess],
        triggered_abilities: vec![crate::effect::shortcut::prowess()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7), r()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::DealDamageDivided {
                    total: Value::Const(5),
                    filter: R::any_target(),
                    max_targets: 5,
                    retaliate_to_source: false,
                },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(5) },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Iron Fist, Hero for Hire",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Mercenary, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Leonardo, Cutting Edge — {1}{W} legendary 1/1 Mutant Ninja Turtle; sneak
/// {W}, lifelink. Whenever you gain life, a +1/+1 counter on him.
pub fn leonardo_cutting_edge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sneak(cost(&[w()])), Keyword::Lifelink],
        alternative_cost: Some(sneak(cost(&[w()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legend(
            "Leonardo, Cutting Edge",
            cost(&[generic(1), w()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            1,
            1,
        )
    }
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

fn your_creatures() -> Selector {
    Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature }
}

/// Moku, Meandering Drummer — {1}{R} legendary 2/2 Human Bard Ally. Whenever
/// you cast a noncreature spell, you may pay {1}; if you do, he gets +2/+1 and
/// creatures you control gain haste until end of turn.
pub fn moku_meandering_drummer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![noncreature_cast(Effect::MayPay {
            description: "Pay {1} to pump Moku and give your creatures haste?".into(),
            mana_cost: cost(&[generic(1)]),
            body: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: your_creatures(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            ])),
            else_: None,
        })],
        ..legend(
            "Moku, Meandering Drummer",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Bard, CreatureType::Ally],
            2,
            2,
        )
    }
}

/// Dragon Throne of Tarkir — {4} legendary Equipment. Equipped creature has
/// defender and "{2}, {T}: other creatures you control gain trample and get
/// +X/+X until end of turn, X its power". Equip {3}.
pub fn dragon_throne_of_tarkir() -> CardDefinition {
    let others = || Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature.and(R::OtherThanSource) };
    let power = || Value::PowerOf(Box::new(Selector::This));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..equipment(
            "Dragon Throne of Tarkir",
            cost(&[generic(4)]),
            cost(&[generic(3)]),
            EquipBonus {
                keywords: vec![Keyword::Defender],
                activated_abilities: vec![ActivatedAbility {
                    mana_cost: cost(&[generic(2)]),
                    tap_cost: true,
                    effect: Effect::Seq(vec![
                        Effect::GrantKeyword { what: others(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                        Effect::PumpPT { what: others(), power: power(), toughness: power(), duration: Duration::EndOfTurn },
                    ]),
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
    }
}

/// Pang Tong, "Young Phoenix" — {1}{W}{W} legendary 1/2 Human Advisor. {T}:
/// target creature gets +0/+2 until end of turn; only during your turn,
/// before attackers are declared.
pub fn pang_tong_young_phoenix() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::Any(vec![
                    Predicate::CurrentStepIs(TurnStep::Upkeep),
                    Predicate::CurrentStepIs(TurnStep::Draw),
                    Predicate::CurrentStepIs(TurnStep::PreCombatMain),
                    Predicate::CurrentStepIs(TurnStep::BeginCombat),
                ]),
            ])),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(0),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Pang Tong, \"Young Phoenix\"",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            2,
        )
    }
}

/// Gauntlets of Light — {2}{W} Aura. +0/+2, assigns combat damage equal to its
/// toughness, and "{2}{W}: Untap this creature".
pub fn gauntlets_of_light() -> CardDefinition {
    CardDefinition {
        name: "Gauntlets of Light",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            toughness: 2,
            keywords: vec![Keyword::AssignsCombatDamageByToughness],
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// The Wondrous Wasp — {1}{U} legendary 2/1 Human Hero; flash, flying. When
/// it enters, tap up to one target creature; it loses all abilities for as
/// long as the Wasp remains on the battlefield.
pub fn the_wondrous_wasp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::LoseAllAbilities { what: Selector::Target(0), duration: Duration::WhileSourceOnBattlefield },
            ])),
        })],
        ..legend("The Wondrous Wasp", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Hero], 2, 1)
    }
}

/// Exhaustion — {2}{U} sorcery. Creatures and lands target opponent controls
/// don't untap during their next untap step.
pub fn exhaustion() -> CardDefinition {
    CardDefinition {
        name: "Exhaustion",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::SkipNextUntap {
                what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature.or(R::Land) },
            }),
        },
        ..Default::default()
    }
}

/// Spider-Slayer, Hatred Honed — {2} legendary 2/1 Human Villain artifact
/// creature. Whenever it deals damage to a Spider, destroy that creature.
/// {6}, exile it from your graveyard: two tapped 1/1 flying Robot artifact
/// creature tokens.
pub fn spider_slayer_hatred_honed() -> CardDefinition {
    let robot = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: "Robot".into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Artifact, CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
    };
    let tapped_robot = || Effect::Seq(vec![robot(), Effect::Tap { what: Selector::LastCreatedToken }]);
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasCreatureType(CreatureType::Spider),
                },
            ),
            effect: Effect::Destroy { what: Selector::Target(0) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Seq(vec![tapped_robot(), tapped_robot()]),
            ..Default::default()
        }],
        ..legend("Spider-Slayer, Hatred Honed", cost(&[generic(2)]), vec![CreatureType::Human, CreatureType::Villain], 2, 1)
    }
}

/// Dependable Quinjet — {3} 3/3 flying Vehicle. {T}: one mana of any color.
/// Crew 4.
pub fn dependable_quinjet() -> CardDefinition {
    CardDefinition {
        name: "Dependable Quinjet",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Crew(4)],
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        ..Default::default()
    }
}

/// Iron Fist, Living Weapon — {2}{R} legendary 2/3 Human Warrior Hero.
/// Whenever you cast a spell that targets a creature you control, he gains
/// "{T}: deals damage equal to his power to any other target" until end of
/// turn.
pub fn iron_fist_living_weapon() -> CardDefinition {
    let strike = ActivatedAbility {
        tap_cost: true,
        effect: Effect::DealDamage {
            to: target_filtered(R::any_target().and(R::OtherThanSource)),
            amount: Value::PowerOf(Box::new(Selector::This)),
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellTargetsMatch(R::Creature.and(R::ControlledByYou))),
            effect: Effect::GainActivatedAbility {
                what: Selector::This,
                ability: Box::new(strike),
                duration: Duration::EndOfTurn,
            },
        }],
        ..legend(
            "Iron Fist, Living Weapon",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero],
            2,
            3,
        )
    }
}

/// Dwarven Mattock — {2} Equipment. When it enters, attach it to target Dwarf
/// you control. Equipped creature gets +2/+2 and has ward {1}. Equip {3}.
pub fn dwarven_mattock() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::HasCreatureType(CreatureType::Dwarf).and(R::ControlledByYou)),
        })],
        ..equipment(
            "Dwarven Mattock",
            cost(&[generic(2)]),
            cost(&[generic(3)]),
            EquipBonus {
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Ward(WardCost::generic(1))],
                ..Default::default()
            },
        )
    }
}

/// Michiko's Reign of Truth // Portrait of Michiko — {1}{W} Saga. I, II:
/// target creature gets +1/+1 per artifact and/or enchantment you control
/// until end of turn. III: it returns transformed, a 0/0 Human Noble
/// enchantment creature with +1/+1 per artifact and/or enchantment you control.
pub fn michikos_reign_of_truth() -> CardDefinition {
    let count = || Value::count(Selector::EachPermanent(R::Artifact.or(R::Enchantment).and(R::ControlledByYou)));
    let pump = || Effect::PumpPT { what: target_filtered(R::Creature), power: count(), toughness: count(), duration: Duration::EndOfTurn };
    let portrait = CardDefinition {
        name: "Portrait of Michiko",
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Noble], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each artifact and/or enchantment you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::Artifact.or(R::Enchantment),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Michiko's Reign of Truth",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![(1, pump()), (2, pump()), (3, Effect::ExileSelfReturnTransformed)],
        back_face: Some(Box::new(portrait)),
        ..Default::default()
    }
}

/// The Princess Takes Flight — {2}{W} Saga. I: exile up to one target
/// creature. II: target creature you control gets +2/+2 and flying until end
/// of turn. III: return the exiled card under its owner's control.
pub fn the_princess_takes_flight() -> CardDefinition {
    CardDefinition {
        name: "The Princess Takes Flight",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::ExileLinked { what: target_filtered(R::Creature) }),
                },
            ),
            (
                2,
                Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature.and(R::ControlledByYou)),
                        power: Value::Const(2),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
                ]),
            ),
            (
                3,
                Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
                },
            ),
        ],
        ..Default::default()
    }
}

/// Victor Timely, Wily Tycoon — {4}{U} legendary 3/4 Human Artificer Villain.
/// When he enters, you may cast an artifact, instant, or sorcery card with
/// mana value 4 or less from your graveyard free; it's exiled instead of
/// going to the graveyard.
pub fn victor_timely_wily_tycoon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CastWithoutPayingImmediate {
            what: target_filtered(
                R::Artifact
                    .or(R::HasCardType(CardType::Instant))
                    .or(R::HasCardType(CardType::Sorcery))
                    .and(R::ManaValueAtMost(4))
                    .from_your_graveyard(),
            ),
            source_zone: Zone::Graveyard,
            exile_after: true,
            copy: false,
            reduce_generic: 0,
            pay_own_cost: false,
        })],
        ..legend(
            "Victor Timely, Wily Tycoon",
            cost(&[generic(4), u()]),
            vec![CreatureType::Human, CreatureType::Artificer, CreatureType::Villain],
            3,
            4,
        )
    }
}

/// S.H.I.E.L.D. Flying Car — {2}{U} 3/3 flying Vehicle, flash. When it
/// enters, exile up to one target creature you control; it returns at the
/// beginning of the next end step. Crew 1.
pub fn shield_flying_car() -> CardDefinition {
    CardDefinition {
        name: "S.H.I.E.L.D. Flying Car",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Crew(1)],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::ExileReturnToOwnerNextEndStep {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                tapped: false,
            }),
        })],
        ..Default::default()
    }
}

/// Nightmares and Daydreams — {2}{U} Saga. I-III: until your next turn,
/// whenever you cast an instant or sorcery spell, target player mills cards
/// equal to its mana value. IV: draw a card, three if a graveyard has twenty
/// or more cards.
pub fn nightmares_and_daydreams() -> CardDefinition {
    let mill = || Effect::GrantTriggeredAbility {
        what: Selector::This,
        trigger: Box::new(TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            }),
            effect: Effect::Mill {
                who: target_filtered(R::Player),
                amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        }),
        duration: Duration::UntilNextTurn,
    };
    CardDefinition {
        name: "Nightmares and Daydreams",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, mill()),
            (2, mill()),
            (3, mill()),
            (
                4,
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::MaxGraveyardSize, Value::Const(20)),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(3) }),
                    else_: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Vulture, Feathered Fiend — {2}{U}{U} legendary 2/4 flying Human Artificer
/// Villain. Whenever one or more creatures you control with flying deal
/// combat damage to a player, a +1/+1 counter on each of them and draw a card.
pub fn vulture_feathered_fiend() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .once_per_batch()
                .dealt_by(R::Creature.and(R::HasKeyword(Keyword::Flying)).and(R::ControlledByYou)),
            effect: Effect::OverTriggerBatch {
                ids: vec![],
                body: Box::new(Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::TriggerBatch, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ])),
            },
        }],
        ..legend(
            "Vulture, Feathered Fiend",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Artificer, CreatureType::Villain],
            2,
            4,
        )
    }
}
