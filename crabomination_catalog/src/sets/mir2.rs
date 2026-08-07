//! Mirage (MIR), second wave. Tests in `classic_sets/mir`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{
    combat_partner_punisher, etb, on_attack, target_any, target_filtered,
};
use crate::effect::{
    Duration, Effect, LookPick, PlayerRef, Predicate, Selector, StaticEffect, Value, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Sorcery], ..instant(name, c, effect) }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, enchant: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, c)
    }
}

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
        ..Default::default()
    }
}

/// The Mirage guildmages: two off-colour tap abilities on a 1/1 Wizard.
fn guildmage(
    name: &'static str,
    c: ManaCost,
    a: ActivatedAbility,
    b_: ActivatedAbility,
) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![a, b_],
        ..creature(name, c, vec![CreatureType::Human, CreatureType::Wizard], 1, 1)
    }
}

fn tap_pump(mana: ManaCost, power: i32, toughness: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        tap_cost: true,
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// "{T}: Grant target creature [keyword] until end of turn."
fn tap_grant(mana: ManaCost, keyword: Keyword) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        tap_cost: true,
        effect: Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// "{T}: Put target creature you control on top of its owner's library."
fn tap_stack_own_creature(mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        tap_cost: true,
        effect: Effect::Move {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: crate::effect::LibraryPosition::Top,
            },
        },
        ..Default::default()
    }
}

/// "{T}: This deals 1 damage to any target and 1 damage to you."
fn tap_ping_self(mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        tap_cost: true,
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::ONE },
            Effect::DealDamage { to: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// "…blocks a creature without flying" — the blocked creature rides in as the
/// trigger's subject.
fn nonflying_subject() -> Predicate {
    Predicate::EntityMatchesAny {
        what: Selector::TriggerSource,
        filter: R::Not(Box::new(R::HasKeyword(Keyword::Flying))),
    }
}

fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

// ── Guildmages ──────────────────────────────────────────────────────────────

/// Armorer Guildmage — {R} 1/1 with black and green pumps.
pub fn armorer_guildmage() -> CardDefinition {
    guildmage(
        "Armorer Guildmage",
        cost(&[r()]),
        tap_pump(cost(&[b()]), 1, 0),
        tap_pump(cost(&[g()]), 0, 1),
    )
}

/// Civic Guildmage — {W} 1/1 that toughens or recycles a creature.
pub fn civic_guildmage() -> CardDefinition {
    guildmage(
        "Civic Guildmage",
        cost(&[w()]),
        tap_pump(cost(&[g()]), 0, 1),
        tap_stack_own_creature(cost(&[u()])),
    )
}

/// Granger Guildmage — {G} 1/1 that pings or grants first strike.
pub fn granger_guildmage() -> CardDefinition {
    guildmage(
        "Granger Guildmage",
        cost(&[g()]),
        tap_ping_self(cost(&[r()])),
        tap_grant(cost(&[w()]), Keyword::FirstStrike),
    )
}

/// Shadow Guildmage — {B} 1/1 that recycles a creature or pings.
pub fn shadow_guildmage() -> CardDefinition {
    guildmage(
        "Shadow Guildmage",
        cost(&[b()]),
        tap_stack_own_creature(cost(&[u()])),
        tap_ping_self(cost(&[r()])),
    )
}

/// Shaper Guildmage — {U} 1/1 that grants first strike or a pump.
pub fn shaper_guildmage() -> CardDefinition {
    guildmage(
        "Shaper Guildmage",
        cost(&[u()]),
        tap_grant(cost(&[w()]), Keyword::FirstStrike),
        tap_pump(cost(&[b()]), 1, 0),
    )
}

// ── The Charms ──────────────────────────────────────────────────────────────

/// Ivory Charm — {W} modal: shrink the board, tap, or prevent 1.
pub fn ivory_charm() -> CardDefinition {
    instant(
        "Ivory Charm",
        cost(&[w()]),
        Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: Value::Const(-2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::PreventNextDamage { target: target_any(), amount: Value::ONE },
        ]),
    )
}

/// Sapphire Charm — {U} modal: a delayed draw, flight, or a phase-out.
pub fn sapphire_charm() -> CardDefinition {
    instant(
        "Sapphire Charm",
        cost(&[u()]),
        Effect::ChooseMode(vec![
            Effect::AtNextTurnsUpkeep {
                body: Box::new(Effect::Draw {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                }),
            },
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::PhaseOut {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                until_source_leaves: false,
            },
        ]),
    )
}

/// Ebony Charm — {B} modal: a drain, graveyard hate, or fear.
pub fn ebony_charm() -> CardDefinition {
    instant(
        "Ebony Charm",
        cost(&[b()]),
        Effect::ChooseMode(vec![
            crate::effect::shortcut::drain(1),
            Effect::ExileFromGraveyard {
                who: PlayerRef::Target(0),
                count: Value::Const(3),
                filter: R::Any,
            },
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Fear,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Chaos Charm — {R} modal: kill a Wall, ping a creature, or grant haste.
pub fn chaos_charm() -> CardDefinition {
    instant(
        "Chaos Charm",
        cost(&[r()]),
        Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Wall))),
            },
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Seedling Charm — {G} modal: bounce an Aura, regenerate, or grant trample.
pub fn seedling_charm() -> CardDefinition {
    instant(
        "Seedling Charm",
        cost(&[g()]),
        Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(R::Enchantment.and(R::HasEnchantmentSubtype(
                    EnchantmentSubtype::Aura,
                ))),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Regenerate {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Green))),
            },
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

// ── Combat-shaped creatures ─────────────────────────────────────────────────

/// Brushwagg — {1}{G}{G} 3/2 that turtles up the moment combat starts.
pub fn brushwagg() -> CardDefinition {
    let shrink = Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(-2),
        toughness: Value::Const(2),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: shrink.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: shrink,
            },
        ],
        ..creature("Brushwagg", cost(&[generic(1), g(), g()]), vec![CreatureType::Brushwagg], 3, 2)
    }
}

/// Jungle Wurm — {3}{G}{G} 5/5 that shrinks per extra blocker (reverse
/// rampage).
pub fn jungle_wurm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Diff(
                    Box::new(Value::ONE),
                    Box::new(Value::BlockersOf(Box::new(Selector::This))),
                ),
                toughness: Value::Diff(
                    Box::new(Value::ONE),
                    Box::new(Value::BlockersOf(Box::new(Selector::This))),
                ),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Jungle Wurm", cost(&[generic(3), g(), g()]), vec![CreatureType::Wurm], 5, 5)
    }
}

/// Crimson Roc — {4}{R} 2/2 flier that punishes a ground blocker.
pub fn crimson_roc() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource)
                .with_filter(nonflying_subject()),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..creature("Crimson Roc", cost(&[generic(4), r()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Dread Specter — {3}{B} 2/2 that kills any nonblack creature it meets.
pub fn dread_specter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: combat_partner_punisher(R::Not(Box::new(R::HasColor(Color::Black)))),
        ..creature("Dread Specter", cost(&[generic(3), b()]), vec![CreatureType::Specter], 2, 2)
    }
}

/// Rock Basilisk — {4}{R}{G} 4/5 whose gaze kills anything but a Wall.
pub fn rock_basilisk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: combat_partner_punisher(R::Not(Box::new(R::HasCreatureType(
            CreatureType::Wall,
        )))),
        ..creature(
            "Rock Basilisk",
            cost(&[generic(4), r(), g()]),
            vec![CreatureType::Basilisk],
            4,
            5,
        )
    }
}

/// Lead Golem — {5} 3/5 artifact creature that stays down after attacking.
pub fn lead_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![on_attack(Effect::SkipNextUntap { what: Selector::This })],
        ..creature("Lead Golem", cost(&[generic(5)]), vec![CreatureType::Golem], 3, 5)
    }
}

/// Gravebane Zombie — {3}{B} 3/2 that goes back on top instead of dying.
pub fn gravebane_zombie() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If this creature would die, put it on top of its owner's library instead.",
            effect: StaticEffect::DiesToLibraryTopInstead {
                filter: R::HasName("Gravebane Zombie".into()),
            },
        }],
        ..creature("Gravebane Zombie", cost(&[generic(3), b()]), vec![CreatureType::Zombie], 3, 2)
    }
}

/// Wall of Resistance — {1}{W} 0/3 flying Wall that hardens under fire.
pub fn wall_of_resistance() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::ValueAtLeast(
                    Value::DamageDealtToSourceThisTurn,
                    Value::ONE,
                )),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusZeroPlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Wall of Resistance", cost(&[generic(1), w()]), vec![CreatureType::Wall], 0, 3)
    }
}

/// Wall of Corpses — {1}{B} 0/2 Wall that takes its attacker with it.
pub fn wall_of_corpses() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
            },
            ..Default::default()
        }],
        ..creature("Wall of Corpses", cost(&[generic(1), b()]), vec![CreatureType::Wall], 0, 2)
    }
}

/// Radiant Essence — {1}{G}{W} 2/3 that grows against a black board.
pub fn radiant_essence() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+2 as long as an opponent controls a black permanent.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Permanent.and(R::ControlledByOpponent).and(R::HasColor(Color::Black)),
                    ),
                    n: Value::ONE,
                },
                power: 1,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..creature(
            "Radiant Essence",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Spirit],
            2,
            3,
        )
    }
}

/// Uktabi Wildcats — {4}{G} whose body is your Forest count.
pub fn uktabi_wildcats() -> CardDefinition {
    let forests = || {
        Value::CountOf(Box::new(Selector::EachPermanent(
            R::HasLandType(LandType::Forest).and(R::ControlledByYou),
        )))
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature's power and toughness are each equal to the number of Forests you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: forests(), toughness: forests() },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_other_filter: Some((R::HasLandType(LandType::Forest), 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Uktabi Wildcats", cost(&[generic(4), g()]), vec![CreatureType::Cat], 0, 0)
    }
}

/// Merfolk Seer — {2}{U} 2/2 that can buy a card on the way out.
pub fn merfolk_seer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::MayPay {
            description: "Pay {1}{U}?".into(),
            mana_cost: cost(&[generic(1), u()]),
            body: Box::new(crate::effect::shortcut::draw(1)),
            else_: None,
        })],
        ..creature(
            "Merfolk Seer",
            cost(&[generic(2), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Pyric Salamander — {1}{R} 1/1 that burns itself out.
pub fn pyric_salamander() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::SacrificeAtNextEndStep { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..creature("Pyric Salamander", cost(&[generic(1), r()]), vec![CreatureType::Salamander], 1, 1)
    }
}

/// Sewer Rats — {B} 1/1 that can be pumped three times a turn.
pub fn sewer_rats() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            life_cost: 1,
            max_activations_per_turn: Some(3),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Sewer Rats", cost(&[b()]), vec![CreatureType::Rat], 1, 1)
    }
}

/// Locust Swarm — {3}{G} 1/1 flier that regenerates and can untap itself.
pub fn locust_swarm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                once_per_turn: true,
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            },
        ],
        ..creature("Locust Swarm", cost(&[generic(3), g()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Harmattan Efreet — {2}{U}{U} 2/2 flier that lends flight.
pub fn harmattan_efreet() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u(), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Harmattan Efreet",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Efreet],
            2,
            2,
        )
    }
}

/// Burning Palm Efreet — {2}{R}{R} 2/2 anti-air gun.
pub fn burning_palm_efreet() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r(), r()]),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                    amount: Value::Const(2),
                },
                Effect::LoseKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Burning Palm Efreet",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Efreet],
            2,
            2,
        )
    }
}

/// Goblin Tinkerer — {1}{R} 1/2 that trades with an artifact.
pub fn goblin_tinkerer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Artifact) },
                Effect::DealDamage {
                    to: Selector::This,
                    amount: Value::TotalManaValueOf(Box::new(Selector::Target(0))),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Goblin Tinkerer",
            cost(&[generic(1), r()]),
            vec![CreatureType::Goblin, CreatureType::Artificer],
            1,
            2,
        )
    }
}

/// Subterranean Spirit — {3}{R}{R} 3/3 that scorches the ground.
pub fn subterranean_spirit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Red)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Subterranean Spirit",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Spirit],
            3,
            3,
        )
    }
}

/// Unseen Walker — {1}{G} 1/1 forestwalker that lends the trick.
pub fn unseen_walker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g(), g()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Landwalk(LandType::Forest),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Unseen Walker", cost(&[generic(1), g()]), vec![CreatureType::Dryad], 1, 1)
    }
}

/// Rashida Scalebane — {3}{W}{W} 3/4 dragonslayer.
pub fn rashida_scalebane() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                },
                Effect::DestroyNoRegen {
                    what: target_filtered(
                        R::Creature
                            .and(R::HasCreatureType(CreatureType::Dragon))
                            .and(R::IsAttacking.or(R::IsBlocking)),
                    ),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Rashida Scalebane",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Shauku's Minion — {1}{B}{R} 2/2 that snipes white creatures.
pub fn shaukus_minion() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), r()]),
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::HasColor(Color::White))),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Shauku's Minion",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Human, CreatureType::Minion],
            2,
            2,
        )
    }
}

/// Zombie Mob — {2}{B}{B} that eats your graveyard to enter huge.
pub fn zombie_mob() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
        )),
        triggered_abilities: vec![etb(Effect::ExileFromGraveyard {
            who: PlayerRef::You,
            count: Value::Const(99),
            filter: R::Creature,
        })],
        ..creature("Zombie Mob", cost(&[generic(2), b(), b()]), vec![CreatureType::Zombie], 2, 0)
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Ancestral Memories — {2}{U}{U}{U}: two of seven, the rest binned.
pub fn ancestral_memories() -> CardDefinition {
    sorcery(
        "Ancestral Memories",
        cost(&[generic(2), u(), u(), u()]),
        Effect::LookPickToHand(Box::new(LookPick {
            count: Value::Const(7),
            take: Some(Value::Const(2)),
            rest_to_graveyard: true,
            ..Default::default()
        })),
    )
}

/// Kaervek's Hex — {3}{B} sorcery: 1 to each nonblack creature, 2 to green.
pub fn kaerveks_hex() -> CardDefinition {
    sorcery(
        "Kaervek's Hex",
        cost(&[generic(3), b()]),
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black)))),
                ),
                amount: Value::ONE,
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Green))),
                amount: Value::ONE,
            },
        ]),
    )
}

/// Tropical Storm — {X}{G} sorcery: X to fliers, one more to blue ones.
pub fn tropical_storm() -> CardDefinition {
    sorcery(
        "Tropical Storm",
        cost(&[crate::mana::x(), g()]),
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                amount: Value::XFromCost,
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Blue))),
                amount: Value::ONE,
            },
        ]),
    )
}

/// Seeds of Innocence — {1}{G}{G}: every artifact dies, its controller paid
/// in life.
pub fn seeds_of_innocence() -> CardDefinition {
    sorcery(
        "Seeds of Innocence",
        cost(&[generic(1), g(), g()]),
        Effect::DestroyAllNoRegenGainControllerLifePerManaValue { filter: R::Artifact },
    )
}

/// Painful Memories — {1}{B}: stack a card off an opponent's hand.
pub fn painful_memories() -> CardDefinition {
    sorcery(
        "Painful Memories",
        cost(&[generic(1), b()]),
        Effect::TopChosenFromHand {
            from: Selector::Player(PlayerRef::Target(0)),
            count: Value::ONE,
            filter: R::Any,
        },
    )
}

/// Ether Well — {3}{U}: stack a creature (a red one may be bottomed instead).
pub fn ether_well() -> CardDefinition {
    instant(
        "Ether Well",
        cost(&[generic(3), u()]),
        Effect::Move {
            what: target_filtered(R::Creature),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: crate::effect::LibraryPosition::Top,
            },
        },
    )
}

/// Soul Rend — {1}{B}: kills a white creature and cantrips next upkeep.
pub fn soul_rend() -> CardDefinition {
    instant(
        "Soul Rend",
        cost(&[generic(1), b()]),
        Effect::Seq(vec![
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::HasColor(Color::White))),
            },
            Effect::AtNextTurnsUpkeep {
                body: Box::new(crate::effect::shortcut::draw(1)),
            },
        ]),
    )
}

/// Jolt — {2}{U}: tap or untap anything, then cantrip next upkeep.
pub fn jolt() -> CardDefinition {
    instant(
        "Jolt",
        cost(&[generic(2), u()]),
        Effect::Seq(vec![
            Effect::TapOrUntap {
                what: target_filtered(R::Artifact.or(R::Creature).or(R::Land)),
            },
            Effect::AtNextTurnsUpkeep {
                body: Box::new(crate::effect::shortcut::draw(1)),
            },
        ]),
    )
}

/// Early Harvest — {1}{G}{G}: untap a player's basics.
pub fn early_harvest() -> CardDefinition {
    instant(
        "Early Harvest",
        cost(&[generic(1), g(), g()]),
        Effect::Untap {
            what: Selector::ControlledBy {
                who: PlayerRef::Target(0),
                filter: R::IsBasicLand,
            },
            up_to: None,
        },
    )
}

/// Soulshriek — {B}: a graveyard-sized swing that costs the creature.
pub fn soulshriek() -> CardDefinition {
    instant(
        "Soulshriek",
        cost(&[b()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: R::Creature,
                },
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::SacrificeAtNextEndStep { what: Selector::Target(0) },
        ]),
    )
}

/// Waiting in the Weeds — {1}{G}{G}: a Cat per untapped Forest, for everyone.
pub fn waiting_in_the_weeds() -> CardDefinition {
    sorcery(
        "Waiting in the Weeds",
        cost(&[generic(1), g(), g()]),
        Effect::EachPlayerCreatesTokenPerControlled {
            filter: R::HasLandType(LandType::Forest).and(R::Untapped),
            definition: TokenDefinition {
                name: "Cat".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Cat],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
    )
}

/// Phyrexian Tribute — {2}{B}: two bodies for an artifact.
pub fn phyrexian_tribute() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 2,
        }],
        ..sorcery(
            "Phyrexian Tribute",
            cost(&[generic(2), b()]),
            Effect::Destroy { what: target_filtered(R::Artifact) },
        )
    }
}

/// Carrion — {1}{B}{B}: a swarm of Insects sized by the body you fed it.
pub fn carrion() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..instant(
            "Carrion",
            cost(&[generic(1), b(), b()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: TokenDefinition {
                    name: "Insect".into(),
                    power: 0,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Insect],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )
    }
}

// ── Auras & enchantments ────────────────────────────────────────────────────

/// Soar — {1}{U} Aura for +0/+1 and flight.
pub fn soar() -> CardDefinition {
    aura(
        "Soar",
        cost(&[generic(1), u()]),
        R::Creature,
        EquipBonus { toughness: 1, keywords: vec![Keyword::Flying], ..Default::default() },
    )
}

/// Lightning Reflexes — {1}{R} Aura for +1/+0 and first strike.
pub fn lightning_reflexes() -> CardDefinition {
    aura(
        "Lightning Reflexes",
        cost(&[generic(1), r()]),
        R::Creature,
        EquipBonus { power: 1, keywords: vec![Keyword::FirstStrike], ..Default::default() },
    )
}

/// Armor of Thorns — {1}{G} Aura for +2/+2 on a nonblack body.
pub fn armor_of_thorns() -> CardDefinition {
    aura(
        "Armor of Thorns",
        cost(&[generic(1), g()]),
        R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black)))),
        EquipBonus { power: 2, toughness: 2, ..Default::default() },
    )
}

/// Grave Servitude — {1}{B} Aura: +3/-1 and black.
pub fn grave_servitude() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature is black.",
            effect: StaticEffect::SetColorOfMatching {
                applies_to: Selector::attached_to(Selector::This),
                color: Color::Black,
            },
        }],
        ..aura(
            "Grave Servitude",
            cost(&[generic(1), b()]),
            R::Creature,
            EquipBonus { power: 3, toughness: -1, ..Default::default() },
        )
    }
}

/// Binding Agony — {1}{B} Aura that reflects damage onto the controller.
pub fn binding_agony() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::EnchantedBySource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(
                    Selector::attached_to(Selector::This),
                ))),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..aura("Binding Agony", cost(&[generic(1), b()]), R::Creature, EquipBonus::default())
    }
}

/// Thirst — {2}{U} Aura that pins a creature down for {U} an upkeep.
pub fn thirst() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::attached_to(Selector::This),
            },
        }],
        triggered_abilities: vec![
            etb(Effect::Tap { what: Selector::attached_to(Selector::This) }),
            TriggeredAbility {
                event: your_upkeep(),
                effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[u()]) },
            },
        ],
        ..aura("Thirst", cost(&[generic(2), u()]), R::Creature, EquipBonus::default())
    }
}

/// Grim Feast — {1}{B}{G}: their dead feed you, at a drip of your own life.
pub fn grim_feast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: your_upkeep(),
                effect: Effect::DealDamage { to: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                },
            },
        ],
        ..enchantment("Grim Feast", cost(&[generic(1), b(), g()]))
    }
}

/// Forsaken Wastes — {2}{B} world enchantment: no lifegain, and a toll.
pub fn forsaken_wastes() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::World],
        static_abilities: vec![StaticAbility {
            description: "Players can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife {
                target: crate::effect::PlayerStaticTarget::EachPlayer,
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::Const(5),
                },
            },
        ],
        ..enchantment("Forsaken Wastes", cost(&[generic(2), b()]))
    }
}

/// Reparations — {1}{W}{U}: their targeted spells replace your cards.
pub fn reparations() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::CastSpellTargetsMatch(
                    R::YouPlayer.or(R::Creature.and(R::ControlledByYou)),
                )),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(crate::effect::shortcut::draw(1)),
            },
        }],
        ..enchantment("Reparations", cost(&[generic(1), w(), u()]))
    }
}

/// Unfulfilled Desires — {1}{U}{B}: rummage on life.
pub fn unfulfilled_desires() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            life_cost: 1,
            effect: Effect::Seq(vec![
                crate::effect::shortcut::draw(1),
                crate::effect::shortcut::discard(Selector::You, 1, false),
            ]),
            ..Default::default()
        }],
        ..enchantment("Unfulfilled Desires", cost(&[generic(1), u(), b()]))
    }
}

/// Cadaverous Bloom — {3}{B}{G}: exile cards from hand for double mana.
pub fn cadaverous_bloom() -> CardDefinition {
    let bloom = |c: Color| ActivatedAbility {
        exile_from_hand_cost: Some(R::Any),
        effect: crate::effect::shortcut::add_mana(vec![c, c]),
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![bloom(Color::Black), bloom(Color::Green)],
        ..enchantment("Cadaverous Bloom", cost(&[generic(3), b(), g()]))
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Razor Pendulum — {4}: a metronome that only ticks for the desperate.
pub fn razor_pendulum() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::PlayerLifeAtMost {
                    who: PlayerRef::ActivePlayer,
                    life: 5,
                }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(2),
            },
        }],
        ..artifact("Razor Pendulum", cost(&[generic(4)]), vec![])
    }
}

/// Chariot of the Sun — {3}: flight at the cost of a paper body.
pub fn chariot_of_the_sun() -> CardDefinition {
    artifact(
        "Chariot of the Sun",
        cost(&[generic(3)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::SetBasePT {
                    what: Selector::Target(0),
                    power: Value::PowerOf(Box::new(Selector::Target(0))),
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Lion's Eye Diamond — {0}: three mana for your whole hand.
pub fn lions_eye_diamond() -> CardDefinition {
    artifact(
        "Lion's Eye Diamond",
        ManaCost::default(),
        vec![ActivatedAbility {
            sac_cost: true,
            discard_hand_cost: true,
            effect: crate::effect::shortcut::add_any_one_color(3),
            ..Default::default()
        }],
    )
}

/// Ersatz Gnomes — {3} 1/1 that launders colour off a spell or permanent.
pub fn ersatz_gnomes() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeColor {
                what: target_filtered(R::Permanent),
                colors: vec![],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            ..Default::default()
        }],
        ..creature("Ersatz Gnomes", cost(&[generic(3)]), vec![CreatureType::Gnome], 1, 1)
    }
}
