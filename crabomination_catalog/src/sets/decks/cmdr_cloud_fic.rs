//! Commander: the cards the **Limit Break** precon (FIC, Final Fantasy VII,
//! Cloud, Ex-SOLDIER) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_cloud_fic.rs`.
//!
//! Residuals (each also on its card):
//! - **Professor Hojo** — no first-targeting-ability discount; the draw
//!   fires for any ability of a permanent that targets your creature.
//! - **Helitrooper** — its {2} equip discount applies to every equip you
//!   activate, not only those targeting it (as FIN's Cloud).
//! - **Lifestream's Blessing** — X is read as it resolves, not as it's cast.
//! - **Yuffie, Materia Hunter** — the Equipment attached is the engine's
//!   pick among yours.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, ConditionalEquipBonus, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, PlayerTally, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r, w};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        supertypes: vec![Supertype::Legendary],
        ..Default::default()
    }
}

fn nonlegendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![], ..def }
}

fn spell(name: &'static str, mana: ManaCost, ty: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![ty], effect, ..Default::default() }
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

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn equipment_or_vehicle() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Equipment).or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
}

fn historic() -> R {
    R::Artifact
        .or(R::HasSupertype(Supertype::Legendary))
        .or(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga))
}

fn begin_combat_on_your_turn(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
        effect,
    }
}

fn soldier() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    }
}

fn pump_power(what: Selector, n: Value) -> Effect {
    Effect::PumpPT { what, power: n, toughness: Value::Const(0), duration: Duration::EndOfTurn }
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

/// Avalanche of Sector 7 — as strong as the opponents' artifacts, and it
/// pings a player who activates one.
pub fn avalanche_of_sector_7() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Its power is equal to the number of artifacts your opponents control.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: Value::count(Selector::EachPermanent(R::Artifact.and(R::ControlledByOpponent))),
                toughness: Value::Const(3),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivated, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Avalanche of Sector 7",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            0,
            3,
        )
    }
}

/// Bugenhagen, Wise Elder — draws each upkeep you have a 7-power creature;
/// taps for any color.
pub fn bugenhagen_wise_elder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![super::super::tap_add_any_color()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(
                Predicate::SelectorExists(Selector::EachPermanent(yours(R::Creature).and(R::PowerAtLeast(7)))),
            ),
            effect: draw(Value::ONE),
        }],
        ..creature(
            "Bugenhagen, Wise Elder",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            1,
            3,
        )
    }
}

/// Cait Sith, Fortune Teller — each combat: scry 1, exile the top card to
/// play this turn, and a creature of yours gets +X/+0 for its mana value.
pub fn cait_sith_fortune_teller() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![begin_combat_on_your_turn(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            Effect::Reflexive {
                body: Box::new(pump_power(target_filtered(yours(R::Creature)), Value::LastExiledManaValue)),
            },
        ]))],
        ..creature(
            "Cait Sith, Fortune Teller",
            cost(&[generic(3), r()]),
            vec![CreatureType::Cat, CreatureType::Moogle],
            3,
            3,
        )
    }
}

/// Cid, Freeflier Pilot — Equipment and Vehicles cost {1} less; flies on your
/// turn; buys an Equipment or Vehicle back from your graveyard.
pub fn cid_freeflier_pilot() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Equipment and Vehicle spells you cast cost {1} less to cast.",
                effect: StaticEffect::CostReduction { filter: equipment_or_vehicle(), amount: 1 },
            },
            StaticAbility {
                description: "Jump — During your turn, Cid has flying.",
                effect: StaticEffect::WhileYourTurn {
                    inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Flying }),
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(equipment_or_vehicle().and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature(
            "Cid, Freeflier Pilot",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Pilot],
            2,
            2,
        )
    }
}

/// Conformer Shuriken — the equipped creature taps a defender as it attacks
/// and grows by how much stronger that creature was.
pub fn conformer_shuriken() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..equipment(
            "Conformer Shuriken",
            cost(&[generic(2)]),
            cost(&[generic(2)]),
            EquipBonus {
                triggered_abilities: vec![on_attack(Effect::Seq(vec![
                    Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)) },
                    Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::PowerGreaterThanSource },
                        then: Box::new(Effect::AddCounter {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::Diff(
                                Box::new(Value::PowerOf(Box::new(Selector::Target(0)))),
                                Box::new(Value::PowerOf(Box::new(Selector::This))),
                            ),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]))],
                ..Default::default()
            },
        )
    }
}

/// Elena, Turk Recruit — returns a non-Assassin historic card; grows with
/// each historic spell.
pub fn elena_turk_recruit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Move {
                what: target_filtered(
                    historic()
                        .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Assassin))))
                        .and(R::InYourGraveyard),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(historic())),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..creature(
            "Elena, Turk Recruit",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Assassin],
            1,
            4,
        )
    }
}

/// Heidegger, Shinra Executive — each combat a creature gets +1/+0 per
/// Soldier; each end step Soldiers for every opponent with more creatures.
pub fn heidegger_shinra_executive() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            begin_combat_on_your_turn(pump_power(
                target_filtered(yours(R::Creature)),
                Value::count(Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Soldier)))),
            )),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::PlayersWithGreaterTally(PlayerTally::CreaturesControlled),
                    definition: Arc::new(soldier()),
                },
            },
        ],
        ..creature(
            "Heidegger, Shinra Executive",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Helitrooper — lends flying to another attacker; its equip discount.
/// Residual: the discount covers every equip you activate.
pub fn helitrooper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![StaticAbility {
            description: "Equip abilities you activate that target this creature cost {2} less to activate.",
            effect: StaticEffect::EquipCostReduction { amount: 2 },
        }],
        ..nonlegendary(creature(
            "Helitrooper",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            2,
        ))
    }
}

/// Hero's Heirloom — +2/+1, and trample and haste on a legendary creature.
pub fn heros_heirloom() -> CardDefinition {
    equipment(
        "Hero's Heirloom",
        cost(&[generic(2)]),
        cost(&[generic(2)]),
        EquipBonus {
            power: 2,
            toughness: 1,
            conditional: vec![ConditionalEquipBonus {
                host_filter: R::HasSupertype(Supertype::Legendary),
                keywords: vec![Keyword::Trample, Keyword::Haste],
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Lifestream's Blessing — draw your greatest power; foretold, gain twice
/// that. Residual: X is read as it resolves.
pub fn lifestreams_blessing() -> CardDefinition {
    let x = || Value::GreatestPowerControlled { who: PlayerRef::You };
    CardDefinition {
        foretell_cost: Some(cost(&[generic(4), g()])),
        ..spell(
            "Lifestream's Blessing",
            cost(&[generic(4), g(), g()]),
            CardType::Instant,
            Effect::Seq(vec![
                draw(x()),
                Effect::If {
                    cond: Predicate::SpellWasCastFromExile,
                    then: Box::new(Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Times(Box::new(x()), Box::new(Value::Const(2))),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Professor Hojo — draws once a turn when an ability targets your creature.
/// Residual: no discount; any permanent's ability counts.
pub fn professor_hojo() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourCreatureTargeted)
                .caused_by(R::OnBattlefield)
                .once_per_turn(),
            effect: draw(Value::ONE),
        }],
        ..creature(
            "Professor Hojo",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            2,
            2,
        )
    }
}

/// Red XIII, Proud Warrior — your other modified creatures get vigilance
/// and trample; it returns an Aura or Equipment card on entry.
pub fn red_xiii_proud_warrior() -> CardDefinition {
    let modified = || Selector::EachPermanent(yours(R::Creature).and(R::IsModified).and(R::OtherThanSource));
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        static_abilities: vec![
            StaticAbility {
                description: "Other modified creatures you control have vigilance.",
                effect: StaticEffect::GrantKeyword { applies_to: modified(), keyword: Keyword::Vigilance },
            },
            StaticAbility {
                description: "Other modified creatures you control have trample.",
                effect: StaticEffect::GrantKeyword { applies_to: modified(), keyword: Keyword::Trample },
            },
        ],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                    .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment))
                    .and(R::InYourGraveyard),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(
            "Red XIII, Proud Warrior",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Beast, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// SOLDIER Military Program — each combat a Soldier or counters on two
/// Soldiers; both with a commander.
pub fn soldier_military_program() -> CardDefinition {
    let modes = || {
        vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(soldier()) },
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: yours(R::HasCreatureType(CreatureType::Soldier)),
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        ]
    };
    CardDefinition {
        triggered_abilities: vec![begin_combat_on_your_turn(Effect::If {
            cond: Predicate::YouControlACommander,
            then: Box::new(Effect::ChooseN { picks: vec![0, 1], modes: modes() }),
            else_: Box::new(Effect::ChooseMode(modes())),
        })],
        ..spell("SOLDIER Military Program", cost(&[generic(2), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Sephiroth, Fallen Hero — attacking, a cell counter and every modified
/// creature of yours becomes 7/5; returns by sacrificing a modified creature.
pub fn sephiroth_fallen_hero() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::MayDo {
                description: "Put a cell counter on target creature?".into(),
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::Cell,
                    amount: Value::ONE,
                }),
            },
            Effect::SetBasePT {
                what: Selector::EachPermanent(yours(R::Creature).and(R::IsModified)),
                power: Value::Const(7),
                toughness: Value::Const(5),
                duration: Duration::EndOfTurn,
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            from_graveyard: true,
            sac_other_filter: Some((R::Creature.and(R::IsModified), 1)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature(
            "Sephiroth, Fallen Hero",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Human, CreatureType::Avatar, CreatureType::Soldier],
            7,
            5,
        )
    }
}

/// Summon: Kujata — a Saga Ox: 3 damage to two creatures, three can't
/// block, then rummage for damage to each opponent.
pub fn summon_kujata() -> CardDefinition {
    CardDefinition {
        name: "Summon: Kujata",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Ox],
            ..Default::default()
        },
        power: 7,
        toughness: 5,
        keywords: vec![Keyword::Trample, Keyword::Haste],
        saga_chapters: vec![
            (
                1,
                Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) }),
                },
            ),
            (
                2,
                Effect::ApplyToTargets {
                    max_targets: 3,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::CantBlock,
                        duration: Duration::EndOfTurn,
                    }),
                },
            ),
            (
                3,
                Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    draw(Value::Const(2)),
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::LastDiscardedManaValue,
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Summoning Materia — look at your top card; while attached, cast
/// creatures from it; +2/+2, vigilance and "{T}: Add {G}".
pub fn summoning_materia() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "As long as this Equipment is attached to a creature, you may cast creature spells from the top of your library.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::EntityMatches { what: Selector::This, filter: R::AttachedToCreature },
                    inner: Box::new(StaticEffect::PlayFromLibraryTop { filter: R::Creature }),
                },
            },
        ],
        ..equipment(
            "Summoning Materia",
            cost(&[generic(2), g()]),
            cost(&[generic(2)]),
            EquipBonus {
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Vigilance],
                activated_abilities: vec![super::super::tap_add(Color::Green)],
                ..Default::default()
            },
        )
    }
}

/// Tifa, Martial Artist — melee; a 7-power hit untaps your team and, in the
/// first combat, adds another.
pub fn tifa_martial_artist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Melee],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::PowerAtLeast(7) })
                .once_per_batch(),
            effect: Effect::Seq(vec![
                Effect::Untap { what: Selector::EachPermanent(yours(R::Creature)), up_to: None },
                Effect::If {
                    cond: Predicate::IsFirstCombatPhaseThisTurn,
                    then: Box::new(Effect::AdditionalCombatPhase { count: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..creature(
            "Tifa, Martial Artist",
            cost(&[generic(1), r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Monk],
            4,
            4,
        )
    }
}

/// Ultimate Magic: Holy — your permanents are indestructible; foretold, you
/// take no damage this turn.
pub fn ultimate_magic_holy() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[generic(2), w()])),
        ..spell(
            "Ultimate Magic: Holy",
            cost(&[generic(2), w()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::ControlledByYou),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: Predicate::SpellWasCastFromExile,
                    then: Box::new(Effect::PreventAllDamageToPlayerThisTurn { who: PlayerRef::You }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Ultimate Magic: Meteor — 7 to each creature; foretold, each opponent also
/// loses an artifact or land.
pub fn ultimate_magic_meteor() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[generic(5), r()])),
        ..spell(
            "Ultimate Magic: Meteor",
            cost(&[generic(5), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(7) },
                Effect::If {
                    cond: Predicate::SpellWasCastFromExile,
                    then: Box::new(Effect::DestroyOnePerOpponent { filter: R::Artifact.or(R::Land) }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Vincent, Vengeful Atoner — grows when your creatures connect; at 7 power
/// its hits splash every other opponent.
pub fn vincent_vengeful_atoner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch(),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                    .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer }),
                effect: Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::This, filter: R::PowerAtLeast(7) },
                    then: Box::new(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachOpponentExceptTriggerer),
                        amount: Value::TriggerEventAmount,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..creature("Vincent, Vengeful Atoner", cost(&[generic(2), r()]), vec![CreatureType::Assassin], 3, 3)
    }
}

/// Wrecking Ball Arm — the equipped creature is a 7/7 that small creatures
/// can't block; equip a legendary creature for {3}.
pub fn wrecking_ball_arm() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        equip_filtered_cost: Some((R::HasSupertype(Supertype::Legendary), cost(&[generic(3)]))),
        ..equipment(
            "Wrecking Ball Arm",
            cost(&[generic(2)]),
            cost(&[generic(7)]),
            EquipBonus {
                set_base_pt: Some((7, 7)),
                keywords: vec![Keyword::CantBeBlockedByPowerAtMost(2)],
                ..Default::default()
            },
        )
    }
}

/// Yuffie, Materia Hunter — ninjutsu; steals a noncreature artifact while
/// you control her, then may pick up an Equipment of yours.
/// Residual: the Equipment is the engine's pick.
pub fn yuffie_materia_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), r()]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainControlWhileYouControlSource {
                what: target_filtered(R::Artifact.and(R::Noncreature)),
            },
            Effect::MayDo {
                description: "Attach an Equipment you control to Yuffie?".into(),
                body: Box::new(Effect::Attach {
                    what: Selector::Take {
                        inner: Box::new(Selector::EachPermanent(yours(R::HasArtifactSubtype(ArtifactSubtype::Equipment)))),
                        count: Box::new(Value::ONE),
                    },
                    to: Selector::This,
                }),
            },
        ]))],
        ..creature(
            "Yuffie, Materia Hunter",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Ninja],
            3,
            3,
        )
    }
}
