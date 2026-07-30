//! Ravnica Allegiance (RNA) — 2019. Commons/uncommons on existing primitives.
//! Tests in `classic_sets/rna`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    adapt, afterlife, deal, draw, each_creature, each_your_creature, etb, etb_scry, on_attack,
    riot, spectacle, target_any, target_filtered,
};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Predicate, RevealMissDest, Selector,
    StaticEffect, ZoneDest, ZoneRef,
};
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w, x};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

fn body(
    name: &'static str,
    mana: crate::mana::ManaCost,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: creatures(ct),
        power: p,
        toughness: t,
        keywords: kw,
        ..Default::default()
    }
}

/// Catacomb Crocodile — {4}{B} 3/7 Crocodile.
pub fn catacomb_crocodile() -> CardDefinition {
    body(
        "Catacomb Crocodile",
        cost(&[generic(4), b()]),
        3,
        7,
        vec![CreatureType::Crocodile],
        vec![],
    )
}

/// Azorius Knight-Arbiter — {3}{W}{U} 2/5 Human Knight. Vigilance; can't be
/// blocked.
pub fn azorius_knight_arbiter() -> CardDefinition {
    body(
        "Azorius Knight-Arbiter",
        cost(&[generic(3), w(), u()]),
        2,
        5,
        vec![CreatureType::Human, CreatureType::Knight],
        vec![Keyword::Vigilance, Keyword::Unblockable],
    )
}

/// Carrion Imp — {3}{B} 2/3 Imp with flying. ETB may exile a creature card from
/// a graveyard; if you do, gain 2 life.
pub fn carrion_imp() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exile target creature card from a graveyard; gain 2 life.".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InGraveyard)),
                    to: ZoneDest::Exile,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ])),
        })],
        ..body(
            "Carrion Imp",
            cost(&[generic(3), b()]),
            2,
            3,
            vec![CreatureType::Imp],
            vec![Keyword::Flying],
        )
    }
}

/// Civic Stalwart — {3}{W} 3/3 Elephant Soldier. ETB creatures you control get
/// +1/+1 until end of turn.
pub fn civic_stalwart() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..body(
            "Civic Stalwart",
            cost(&[generic(3), w()]),
            3,
            3,
            vec![CreatureType::Elephant, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Blade Juggler — {4}{B} 3/2 Human Rogue with Spectacle {2}{B}. ETB deals 1
/// damage to you and you draw a card.
pub fn blade_juggler() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(spectacle(cost(&[generic(2), b()]))),
        triggered_abilities: vec![etb(Effect::Seq(vec![deal(1, Selector::You), draw(1)]))],
        ..body(
            "Blade Juggler",
            cost(&[generic(4), b()]),
            3,
            2,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Devkarin Dissident — {1}{G} 2/2 Elf Warrior. {4}{G}: +2/+2 until end of turn.
pub fn devkarin_dissident() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body(
            "Devkarin Dissident",
            cost(&[generic(1), g()]),
            2,
            2,
            vec![CreatureType::Elf, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Passwall Adept — {1}{U} 1/3 Human Wizard. {2}{U}: target creature can't be
/// blocked this turn.
pub fn passwall_adept() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body(
            "Passwall Adept",
            cost(&[generic(1), u()]),
            1,
            3,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Rakdos Firewheeler — {B}{B}{R}{R} 4/3 Human Rogue. ETB deals 2 to target
/// opponent and 2 to up to one target creature or planeswalker.
pub fn rakdos_firewheeler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::OpponentPlayer,
                    },
                    amount: Value::Const(2),
                },
                Effect::DealDamage {
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.or(R::HasCardType(CardType::Planeswalker)),
                    },
                    amount: Value::Const(2),
                },
            ])),
        })],
        ..body(
            "Rakdos Firewheeler",
            cost(&[b(), b(), r(), r()]),
            4,
            3,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Gyre Engineer — {1}{G}{U} 1/1 Vedalken Wizard. {T}: Add {G}{U}. Whenever you
/// activate an adapt ability, untap Gyre Engineer.
pub fn gyre_engineer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Green, Color::Blue]),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AdaptAbilityActivated, EventScope::YourControl),
            effect: Effect::Untap {
                what: Selector::This,
                up_to: None,
            },
        }],
        ..body(
            "Gyre Engineer",
            cost(&[generic(1), g(), u()]),
            1,
            1,
            vec![CreatureType::Vedalken, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Bring to Trial — {2}{W} Sorcery. Exile target creature with power 4 or
/// greater.
pub fn bring_to_trial() -> CardDefinition {
    CardDefinition {
        name: "Bring to Trial",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(R::Creature.and(R::PowerAtLeast(4))),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Burn Bright — {2}{R} Instant. Creatures you control get +2/+0 until end of
/// turn.
pub fn burn_bright() -> CardDefinition {
    CardDefinition {
        name: "Burn Bright",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Applied Biomancy — {G}{U} Instant. Choose one or both — target creature gets
/// +1/+1 until end of turn; and/or return target creature to its owner's hand.
pub fn applied_biomancy() -> CardDefinition {
    CardDefinition {
        name: "Applied Biomancy",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Arrester's Zeal — {W} Instant. Target creature gets +2/+2 until end of turn.
/// Addendum — if cast during your main phase, it also gains flying.
pub fn arresters_zeal() -> CardDefinition {
    CardDefinition {
        name: "Arrester's Zeal",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::YourMainPhase,
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Arrester's Admonition — {2}{U} Instant. Return target creature to its owner's
/// hand. Addendum — if cast during your main phase, draw a card.
pub fn arresters_admonition() -> CardDefinition {
    CardDefinition {
        name: "Arrester's Admonition",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::If {
                cond: Predicate::YourMainPhase,
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Ironshell Beetle — {1}{G} 1/1 Insect. ETB put a +1/+1 counter on target
/// creature.
pub fn ironshell_beetle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..body(
            "Ironshell Beetle",
            cost(&[generic(1), g()]),
            1,
            1,
            vec![CreatureType::Insect],
            vec![],
        )
    }
}

/// Vizkopa Vampire — {2}{W/B} 3/1 Vampire with lifelink.
pub fn vizkopa_vampire() -> CardDefinition {
    body(
        "Vizkopa Vampire",
        cost(&[generic(2), hybrid(Color::White, Color::Black)]),
        3,
        1,
        vec![CreatureType::Vampire],
        vec![Keyword::Lifelink],
    )
}

/// Rubblebelt Recluse — {4}{R} 6/5 Ogre Berserker that attacks each combat if
/// able.
pub fn rubblebelt_recluse() -> CardDefinition {
    body(
        "Rubblebelt Recluse",
        cost(&[generic(4), r()]),
        6,
        5,
        vec![CreatureType::Ogre, CreatureType::Berserker],
        vec![Keyword::MustAttack],
    )
}

/// Rakdos Trumpeter — {1}{B} 1/3 Human Shaman with menace. {3}{R}: +2/+0 until
/// end of turn.
pub fn rakdos_trumpeter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body(
            "Rakdos Trumpeter",
            cost(&[generic(1), b()]),
            1,
            3,
            vec![CreatureType::Human, CreatureType::Shaman],
            vec![Keyword::Menace],
        )
    }
}

/// Griffin Protector — {3}{W} 2/3 Griffin with flying. Whenever another creature
/// you control enters, it gets +1/+1 until end of turn.
pub fn griffin_protector() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..body(
            "Griffin Protector",
            cost(&[generic(3), w()]),
            2,
            3,
            vec![CreatureType::Griffin],
            vec![Keyword::Flying],
        )
    }
}

/// A vanilla token creature body of `colors`, P/T, and creature types.
fn token(
    name: &'static str,
    colors: Vec<Color>,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords: kw,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: creatures(ct),
        ..Default::default()
    }
}

/// Tithe Taker — {1}{W} 2/1 Human Soldier with Afterlife 1. During your turn,
/// opponents' spells and non-mana abilities cost {1} more.
pub fn tithe_taker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(1)],
        static_abilities: vec![StaticAbility {
            description: "During your turn, opponents' spells and non-mana abilities cost {1} more.",
            effect: StaticEffect::OpponentActivityCostsMoreOnYourTurn { amount: 1 },
        }],
        ..body(
            "Tithe Taker",
            cost(&[generic(1), w()]),
            2,
            1,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Imperious Oligarch — {W}{B} 2/1 Human Cleric with vigilance and Afterlife 1.
pub fn imperious_oligarch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(1)],
        ..body(
            "Imperious Oligarch",
            cost(&[w(), b()]),
            2,
            1,
            vec![CreatureType::Human, CreatureType::Cleric],
            vec![Keyword::Vigilance],
        )
    }
}

/// Rampaging Rendhorn — {4}{G} 4/4 Beast with Riot.
pub fn rampaging_rendhorn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![riot()],
        ..body(
            "Rampaging Rendhorn",
            cost(&[generic(4), g()]),
            4,
            4,
            vec![CreatureType::Beast],
            vec![],
        )
    }
}

/// Spear Spewer — {R} 0/2 Goblin Warrior with defender. {T}: deal 1 damage to
/// each player.
pub fn spear_spewer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..body(
            "Spear Spewer",
            cost(&[r()]),
            0,
            2,
            vec![CreatureType::Goblin, CreatureType::Warrior],
            vec![Keyword::Defender],
        )
    }
}

/// Vindictive Vampire — {3}{B} 2/3 Vampire. Whenever another creature you
/// control dies, deal 1 damage to each opponent and gain 1 life.
pub fn vindictive_vampire() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::OtherThanSource,
                },
            ),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..body(
            "Vindictive Vampire",
            cost(&[generic(3), b()]),
            2,
            3,
            vec![CreatureType::Vampire],
            vec![],
        )
    }
}

/// Sauroform Hybrid — {1}{G} 2/2 Human Lizard Warrior. {4}{G}{G}: Adapt 4.
pub fn sauroform_hybrid() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g(), g()]),
            effect: adapt(4),
            ..Default::default()
        }],
        ..body(
            "Sauroform Hybrid",
            cost(&[generic(1), g()]),
            2,
            2,
            vec![
                CreatureType::Human,
                CreatureType::Lizard,
                CreatureType::Warrior,
            ],
            vec![],
        )
    }
}

/// Skitter Eel — {3}{U} 3/3 Fish Crab. {2}{U}: Adapt 2.
pub fn skitter_eel() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: adapt(2),
            ..Default::default()
        }],
        ..body(
            "Skitter Eel",
            cost(&[generic(3), u()]),
            3,
            3,
            vec![CreatureType::Fish, CreatureType::Crab],
            vec![],
        )
    }
}

/// Titanic Brawl — {1}{G} Instant. Costs {1} less if it targets a creature you
/// control with a +1/+1 counter. Target creature you control fights a creature
/// you don't control.
pub fn titanic_brawl() -> CardDefinition {
    CardDefinition {
        name: "Titanic Brawl",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_cost_if_target: Some((
            R::Creature
                .and(R::ControlledByYou)
                .and(R::WithCounter(CounterType::PlusOnePlusOne)),
            cost(&[generic(1)]),
        )),
        effect: Effect::Fight {
            attacker: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.and(R::ControlledByYou),
            },
            defender: Selector::TargetFiltered {
                slot: 1,
                filter: R::Creature.and(R::ControlledByOpponent),
            },
        },
        ..Default::default()
    }
}

/// Rakdos Roustabout — {1}{B}{R} 3/2 Ogre Warrior. Whenever it becomes blocked,
/// it deals 1 damage to the player it's attacking.
pub fn rakdos_roustabout() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::ONE,
            },
        }],
        ..body(
            "Rakdos Roustabout",
            cost(&[generic(1), b(), r()]),
            3,
            2,
            vec![CreatureType::Ogre, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Consign to the Pit — {5}{B} Sorcery. Destroy target creature and deal 2
/// damage to that creature's controller.
pub fn consign_to_the_pit() -> CardDefinition {
    CardDefinition {
        name: "Consign to the Pit",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(2),
            },
            Effect::Destroy {
                what: target_filtered(R::Creature),
            },
        ]),
        ..Default::default()
    }
}

/// Scorchmark — {1}{R} Instant. Deal 2 damage to target creature; if it would
/// die this turn, exile it instead.
pub fn scorchmark() -> CardDefinition {
    CardDefinition {
        name: "Scorchmark",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn {
                what: target_filtered(R::Creature),
            },
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Senate Guildmage — {W}{U} 2/2 Human Wizard. {W}, {T}: gain 2 life. {U}, {T}:
/// draw a card, then discard a card.
pub fn senate_guildmage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                tap_cost: true,
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    draw(1),
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..body(
            "Senate Guildmage",
            cost(&[w(), u()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Undercity Scavenger — {3}{B} 3/3 Ogre Warrior. ETB you may sacrifice another
/// creature; if you do, put two +1/+1 counters on it, then scry 2.
pub fn undercity_scavenger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Sacrifice another creature: two +1/+1 counters and scry 2.".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Creature.and(R::OtherThanSource),
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
            ])),
        })],
        ..body(
            "Undercity Scavenger",
            cost(&[generic(3), b()]),
            3,
            3,
            vec![CreatureType::Ogre, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Gatebreaker Ram — {2}{G} 2/2 Sheep. +1/+1 for each Gate you control; while
/// you control two or more Gates it has vigilance and trample.
pub fn gatebreaker_ram() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Gets +1/+1 for each Gate you control.",
                effect: StaticEffect::PumpSelfByControlledPermanents {
                    filter: R::HasLandType(LandType::Gate),
                    per_power: 1,
                    per_toughness: 1,
                },
            },
            StaticAbility {
                description: "While you control two or more Gates, has vigilance and trample.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(
                            R::HasLandType(LandType::Gate).and(R::ControlledByYou),
                        ),
                        n: Value::Const(2),
                    },
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::Vigilance, Keyword::Trample],
                },
            },
        ],
        ..body(
            "Gatebreaker Ram",
            cost(&[generic(2), g()]),
            2,
            2,
            vec![CreatureType::Sheep],
            vec![],
        )
    }
}

/// Feral Maaka — {1}{R} 2/2 Cat.
pub fn feral_maaka() -> CardDefinition {
    body(
        "Feral Maaka",
        cost(&[generic(1), r()]),
        2,
        2,
        vec![CreatureType::Cat],
        vec![],
    )
}

/// Rubble Slinger — {2}{R/G} 2/3 Human Warrior with reach.
pub fn rubble_slinger() -> CardDefinition {
    body(
        "Rubble Slinger",
        cost(&[generic(2), hybrid(Color::Red, Color::Green)]),
        2,
        3,
        vec![CreatureType::Human, CreatureType::Warrior],
        vec![Keyword::Reach],
    )
}

/// Watchful Giant — {5}{W} 3/6 Giant Soldier. ETB create a 1/1 white Human.
pub fn watchful_giant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: token(
                "Human",
                vec![Color::White],
                1,
                1,
                vec![CreatureType::Human],
                vec![],
            ),
        })],
        ..body(
            "Watchful Giant",
            cost(&[generic(5), w()]),
            3,
            6,
            vec![CreatureType::Giant, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Faerie Duelist — {1}{U} 1/2 Faerie Rogue with flash and flying. ETB target
/// creature an opponent controls gets -2/-0 until end of turn.
pub fn faerie_duelist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            power: Value::Const(-2),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        })],
        ..body(
            "Faerie Duelist",
            cost(&[generic(1), u()]),
            1,
            2,
            vec![CreatureType::Faerie, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Coral Commando — {2}{U} 3/2 Merfolk Warrior.
pub fn coral_commando() -> CardDefinition {
    body(
        "Coral Commando",
        cost(&[generic(2), u()]),
        3,
        2,
        vec![CreatureType::Merfolk, CreatureType::Warrior],
        vec![],
    )
}

/// Windstorm Drake — {4}{U} 3/3 Drake with flying. Other creatures you control
/// with flying get +1/+0.
pub fn windstorm_drake() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control with flying get +1/+0.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasKeyword(Keyword::Flying).and(R::OtherThanSource),
                power: 1,
                toughness: 0,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..body(
            "Windstorm Drake",
            cost(&[generic(4), u()]),
            3,
            3,
            vec![CreatureType::Drake],
            vec![Keyword::Flying],
        )
    }
}

/// Drill Bit — {2}{B} Sorcery with Spectacle {B}. Target player reveals their
/// hand; you choose a nonland card; that player discards it.
pub fn drill_bit() -> CardDefinition {
    CardDefinition {
        name: "Drill Bit",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        alternative_cost: Some(spectacle(cost(&[b()]))),
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::Target(0)),
            count: Value::ONE,
            filter: R::Nonland,
        },
        ..Default::default()
    }
}

/// Burning-Tree Vandal — {2}{R} 2/1 Human Rogue with Riot. Whenever it attacks,
/// you may discard a card; if you do, draw a card.
pub fn burning_tree_vandal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            riot(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Discard a card, then draw a card.".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard {
                            who: Selector::You,
                            amount: Value::ONE,
                            random: false,
                        },
                        draw(1),
                    ])),
                },
            },
        ],
        ..body(
            "Burning-Tree Vandal",
            cost(&[generic(2), r()]),
            2,
            1,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Ghor-Clan Wrecker — {3}{R} 2/2 Human Warrior with Riot and menace.
pub fn ghor_clan_wrecker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![riot()],
        ..body(
            "Ghor-Clan Wrecker",
            cost(&[generic(3), r()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Warrior],
            vec![Keyword::Menace],
        )
    }
}

/// Sprouting Renewal — {2}{G} Sorcery with convoke. Choose one — create a 2/2
/// green-and-white Elf Knight with vigilance; or destroy target artifact or
/// enchantment.
pub fn sprouting_renewal() -> CardDefinition {
    CardDefinition {
        name: "Sprouting Renewal",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Convoke],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: token(
                        "Elf Knight",
                        vec![Color::Green, Color::White],
                        2,
                        2,
                        vec![CreatureType::Elf, CreatureType::Knight],
                        vec![Keyword::Vigilance],
                    ),
                },
                Effect::Destroy {
                    what: target_filtered(R::Artifact.or(R::Enchantment)),
                },
            ],
            min: 1,
            max: 1,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Open the Gates — {G} Sorcery. Search your library for a basic land or Gate
/// card, reveal it, put it into your hand, then shuffle.
pub fn open_the_gates() -> CardDefinition {
    CardDefinition {
        name: "Open the Gates",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand.or(R::HasLandType(LandType::Gate)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Cindervines — {R}{G} Enchantment. Whenever an opponent casts a noncreature
/// spell, deal 1 damage to that player. {1}, Sacrifice this: destroy target
/// artifact or enchantment and deal 2 damage to that permanent's controller.
pub fn cindervines() -> CardDefinition {
    CardDefinition {
        name: "Cindervines",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Not(Box::new(R::Creature)),
                },
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(2),
                },
                Effect::Destroy {
                    what: target_filtered(R::Artifact.or(R::Enchantment)),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Summary Judgment — {1}{W} Instant. Deal 3 damage to target tapped creature;
/// Addendum — 5 damage instead if cast during your main phase.
pub fn summary_judgment() -> CardDefinition {
    CardDefinition {
        name: "Summary Judgment",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::YourMainPhase,
            then: Box::new(Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::Tapped)),
                amount: Value::Const(5),
            }),
            else_: Box::new(Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::Tapped)),
                amount: Value::Const(3),
            }),
        },
        ..Default::default()
    }
}

/// Haazda Officer — {2}{W} 3/2 Human Soldier. ETB target creature you control
/// gets +1/+1 until end of turn.
pub fn haazda_officer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..body(
            "Haazda Officer",
            cost(&[generic(2), w()]),
            3,
            2,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Twilight Panther — {W} 1/2 Cat Spirit. {B}: gains deathtouch until end of turn.
pub fn twilight_panther() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body(
            "Twilight Panther",
            cost(&[w()]),
            1,
            2,
            vec![CreatureType::Cat, CreatureType::Spirit],
            vec![],
        )
    }
}

/// Vedalken Mesmerist — {1}{U} 2/1 Vedalken Wizard. Whenever it attacks, target
/// creature an opponent controls gets -2/-0 until end of turn.
pub fn vedalken_mesmerist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..body(
            "Vedalken Mesmerist",
            cost(&[generic(1), u()]),
            2,
            1,
            vec![CreatureType::Vedalken, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Chillbringer — {4}{U} 3/3 Elemental with flying. ETB tap target creature an
/// opponent controls; it doesn't untap during its controller's next untap step.
pub fn chillbringer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::ONE,
            },
        ]))],
        ..body(
            "Chillbringer",
            cost(&[generic(4), u()]),
            3,
            3,
            vec![CreatureType::Elemental],
            vec![Keyword::Flying],
        )
    }
}

/// Grotesque Demise — {2}{B} Instant. Exile target creature with power 3 or less.
pub fn grotesque_demise() -> CardDefinition {
    CardDefinition {
        name: "Grotesque Demise",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(R::Creature.and(R::PowerAtMost(3))),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Noxious Groodion — {2}{B} 2/2 Beast with deathtouch.
pub fn noxious_groodion() -> CardDefinition {
    body(
        "Noxious Groodion",
        cost(&[generic(2), b()]),
        2,
        2,
        vec![CreatureType::Beast],
        vec![Keyword::Deathtouch],
    )
}

/// Cavalcade of Calamity — {1}{R} Enchantment. Whenever a creature you control
/// with power 1 or less attacks, deal 1 damage to the player it's attacking.
pub fn cavalcade_of_calamity() -> CardDefinition {
    CardDefinition {
        name: "Cavalcade of Calamity",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtMost(1)),
                },
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Rubble Reading — {3}{R} Sorcery. Destroy target land, then scry 2.
pub fn rubble_reading() -> CardDefinition {
    CardDefinition {
        name: "Rubble Reading",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Land),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Regenesis — {3}{G}{G} Instant. Return up to two target permanent cards from
/// your graveyard to your hand.
pub fn regenesis() -> CardDefinition {
    CardDefinition {
        name: "Regenesis",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::InGraveyard.and(R::Not(Box::new(
                R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            ))),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
        ..Default::default()
    }
}

/// Steeple Creeper — {2}{G} 4/2 Frog Snake. {3}{U}: gains flying until end of turn.
pub fn steeple_creeper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body(
            "Steeple Creeper",
            cost(&[generic(2), g()]),
            4,
            2,
            vec![CreatureType::Frog, CreatureType::Snake],
            vec![],
        )
    }
}

/// Gruul Beastmaster — {3}{G} 2/2 Human Shaman with Riot. Whenever it attacks,
/// another target creature you control gets +X/+0 until end of turn, where X is
/// this creature's power.
pub fn gruul_beastmaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            riot(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: target_filtered(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    power: Value::PowerOf(Box::new(Selector::This)),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..body(
            "Gruul Beastmaster",
            cost(&[generic(3), g()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Trollbred Guardian — {4}{G} 5/5 Troll Frog Warrior. {2}{G}: Adapt 2. Each
/// creature you control with a +1/+1 counter has trample.
pub fn trollbred_guardian() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: adapt(2),
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a +1/+1 counter has trample.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::WithCounter(CounterType::PlusOnePlusOne),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Trample],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..body(
            "Trollbred Guardian",
            cost(&[generic(4), g()]),
            5,
            5,
            vec![
                CreatureType::Troll,
                CreatureType::Frog,
                CreatureType::Warrior,
            ],
            vec![],
        )
    }
}

/// Loxodon Restorer — {4}{W}{W} 3/4 Elephant Cleric with convoke. ETB gain 4 life.
pub fn loxodon_restorer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(4),
        })],
        ..body(
            "Loxodon Restorer",
            cost(&[generic(4), w(), w()]),
            3,
            4,
            vec![CreatureType::Elephant, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Syndicate Messenger — {3}{W} 2/3 Bird with flying and Afterlife 1.
pub fn syndicate_messenger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(1)],
        ..body(
            "Syndicate Messenger",
            cost(&[generic(3), w()]),
            2,
            3,
            vec![CreatureType::Bird],
            vec![Keyword::Flying],
        )
    }
}

/// Prying Eyes — {4}{U}{U} Instant. Draw four cards, then discard two cards.
pub fn prying_eyes() -> CardDefinition {
    CardDefinition {
        name: "Prying Eyes",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            draw(4),
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(2),
                random: false,
            },
        ]),
        ..Default::default()
    }
}

// ══════════════════════════════════════════════════════════════════════════
// Batch 4 (2026-07-24): Locket cycle, adapt/scry creatures, spectacle, spells
// ══════════════════════════════════════════════════════════════════════════

/// The RNA guild Locket cycle — {3} artifacts that tap for one of two colors
/// and sacrifice (paying four hybrid mana) to draw two cards.
fn locket(name: &'static str, c1: Color, c2: Color) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![c1, c2], Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[
                    hybrid(c1, c2),
                    hybrid(c1, c2),
                    hybrid(c1, c2),
                    hybrid(c1, c2),
                ]),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
pub fn azorius_locket() -> CardDefinition {
    locket("Azorius Locket", Color::White, Color::Blue)
}
pub fn orzhov_locket() -> CardDefinition {
    locket("Orzhov Locket", Color::White, Color::Black)
}
pub fn rakdos_locket() -> CardDefinition {
    locket("Rakdos Locket", Color::Black, Color::Red)
}
pub fn gruul_locket() -> CardDefinition {
    locket("Gruul Locket", Color::Red, Color::Green)
}
pub fn simic_locket() -> CardDefinition {
    locket("Simic Locket", Color::Green, Color::Blue)
}

/// Aeromunculus — {1}{G}{U} 2/3 Homunculus Mutant with flying. {2}{G}{U}: Adapt 1.
pub fn aeromunculus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), u()]),
            effect: adapt(1),
            ..Default::default()
        }],
        ..body(
            "Aeromunculus",
            cost(&[generic(1), g(), u()]),
            2,
            3,
            vec![CreatureType::Homunculus, CreatureType::Mutant],
            vec![Keyword::Flying],
        )
    }
}

/// Sage's Row Savant — {1}{U} 2/1 Vedalken Wizard. ETB scry 2.
pub fn sages_row_savant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_scry(2)],
        ..body(
            "Sage's Row Savant",
            cost(&[generic(1), u()]),
            2,
            1,
            vec![CreatureType::Vedalken, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Senate Griffin — {2}{W/U}{W/U} 3/2 Griffin with flying. ETB scry 1.
pub fn senate_griffin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_scry(1)],
        ..body(
            "Senate Griffin",
            cost(&[
                generic(2),
                hybrid(Color::White, Color::Blue),
                hybrid(Color::White, Color::Blue),
            ]),
            3,
            2,
            vec![CreatureType::Griffin],
            vec![Keyword::Flying],
        )
    }
}

/// Sylvan Brushstrider — {2}{G} 3/2 Beast. ETB gain 2 life.
pub fn sylvan_brushstrider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..body(
            "Sylvan Brushstrider",
            cost(&[generic(2), g()]),
            3,
            2,
            vec![CreatureType::Beast],
            vec![],
        )
    }
}

/// Wrecking Beast — {5}{G}{G} 6/6 Beast with riot and trample.
pub fn wrecking_beast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![riot()],
        ..body(
            "Wrecking Beast",
            cost(&[generic(5), g(), g()]),
            6,
            6,
            vec![CreatureType::Beast],
            vec![Keyword::Trample],
        )
    }
}

/// Thirsting Shade — {B} 1/1 Shade with lifelink. {2}{B}: +1/+1 until end of turn.
pub fn thirsting_shade() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body(
            "Thirsting Shade",
            cost(&[b()]),
            1,
            1,
            vec![CreatureType::Shade],
            vec![Keyword::Lifelink],
        )
    }
}

/// Senate Courier — {2}{U} 1/4 Bird with flying. {1}{W}: gains vigilance until EOT.
pub fn senate_courier() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body(
            "Senate Courier",
            cost(&[generic(2), u()]),
            1,
            4,
            vec![CreatureType::Bird],
            vec![Keyword::Flying],
        )
    }
}

/// Enraged Ceratok — {2}{G}{G} 4/4 Rhino. Can't be blocked by creatures with
/// power 2 or less.
pub fn enraged_ceratok() -> CardDefinition {
    body(
        "Enraged Ceratok",
        cost(&[generic(2), g(), g()]),
        4,
        4,
        vec![CreatureType::Rhino],
        vec![Keyword::CantBeBlockedByPowerAtMost(2)],
    )
}

/// Debtors' Transport — {5}{B} 5/3 Thrull with afterlife 2.
pub fn debtors_transport() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(2)],
        ..body(
            "Debtors' Transport",
            cost(&[generic(5), b()]),
            5,
            3,
            vec![CreatureType::Thrull],
            vec![],
        )
    }
}

/// Spikewheel Acrobat — {3}{R} 5/2 Human Rogue with Spectacle {2}{R}.
pub fn spikewheel_acrobat() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(spectacle(cost(&[generic(2), r()]))),
        ..body(
            "Spikewheel Acrobat",
            cost(&[generic(3), r()]),
            5,
            2,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Dagger Caster — {3}{R} 2/3 Lizard Rogue. ETB deals 1 damage to each opponent
/// and 1 damage to each creature your opponents control.
pub fn dagger_caster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            deal(1, Selector::Player(PlayerRef::EachOpponent)),
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                amount: Value::Const(1),
            },
        ]))],
        ..body(
            "Dagger Caster",
            cost(&[generic(3), r()]),
            2,
            3,
            vec![CreatureType::Lizard, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Footlight Fiend — {B/R} 1/1 Devil. When it dies, deals 1 damage to any target.
pub fn footlight_fiend() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::dies_ping_any(1)],
        ..body(
            "Footlight Fiend",
            cost(&[hybrid(Color::Black, Color::Red)]),
            1,
            1,
            vec![CreatureType::Devil],
            vec![],
        )
    }
}

/// Storm Strike — {R} Instant. Target creature gets +1/+0 and gains first strike
/// until end of turn. Scry 1.
pub fn storm_strike() -> CardDefinition {
    CardDefinition {
        name: "Storm Strike",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Stony Strength — {G} Instant. Put a +1/+1 counter on target creature you
/// control; untap that creature.
pub fn stony_strength() -> CardDefinition {
    CardDefinition {
        name: "Stony Strength",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
        ]),
        ..Default::default()
    }
}

/// Ragefire — {1}{R} Sorcery. Deals 3 damage to target creature.
pub fn ragefire() -> CardDefinition {
    CardDefinition {
        name: "Ragefire",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: deal(3, target_filtered(R::Creature)),
        ..Default::default()
    }
}

/// Deface — {R} Sorcery. Choose one — destroy target artifact; or destroy target
/// creature with defender.
pub fn deface() -> CardDefinition {
    CardDefinition {
        name: "Deface",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(R::Artifact),
            },
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Defender))),
            },
        ]),
        ..Default::default()
    }
}

/// Elite Arrester — {W} 0/3 Human Soldier. {1}{U}, {T}: Tap target creature.
pub fn elite_arrester() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..body(
            "Elite Arrester",
            cost(&[w()]),
            0,
            3,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Wall of Lost Thoughts — {1}{U} 0/4 Wall with defender. ETB target player mills 4.
pub fn wall_of_lost_thoughts() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Mill {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(4),
        })],
        ..body(
            "Wall of Lost Thoughts",
            cost(&[generic(1), u()]),
            0,
            4,
            vec![CreatureType::Wall],
            vec![Keyword::Defender],
        )
    }
}

/// Thought Collapse — {1}{U}{U} Instant. Counter target spell; its controller mills 3.
pub fn thought_collapse() -> CardDefinition {
    CardDefinition {
        name: "Thought Collapse",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::Any),
            },
            Effect::Mill {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

/// Skatewing Spy — {3}{U} 2/3 Vedalken Rogue Mutant. {5}{U}: Adapt 2. Each
/// creature you control with a +1/+1 counter has flying.
pub fn skatewing_spy() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u()]),
            effect: adapt(2),
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a +1/+1 counter has flying.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::WithCounter(CounterType::PlusOnePlusOne),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Flying],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..body(
            "Skatewing Spy",
            cost(&[generic(3), u()]),
            2,
            3,
            vec![
                CreatureType::Vedalken,
                CreatureType::Rogue,
                CreatureType::Mutant,
            ],
            vec![],
        )
    }
}

/// Spirit of the Spires — {3}{W} 2/4 Spirit with flying. Other creatures you
/// control with flying get +0/+1.
pub fn spirit_of_the_spires() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control with flying get +0/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasKeyword(Keyword::Flying).and(R::OtherThanSource),
                power: 0,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..body(
            "Spirit of the Spires",
            cost(&[generic(3), w()]),
            2,
            4,
            vec![CreatureType::Spirit],
            vec![Keyword::Flying],
        )
    }
}

/// Shimmer of Possibility — {1}{U} Sorcery. Look at the top four cards of your
/// library. Put one of them into your hand and the rest on the bottom in a
/// random order.
pub fn shimmer_of_possibility() -> CardDefinition {
    CardDefinition {
        name: "Shimmer of Possibility",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: false,
            pick_filter: None,
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: false,
            picked_lands_to_battlefield: false,
            rest_bottom_random: true,
        },
        ..Default::default()
    }
}

/// Dead Revels — {3}{B} Sorcery with Spectacle {1}{B}. Return up to two creature
/// cards from your graveyard to your hand.
pub fn dead_revels() -> CardDefinition {
    CardDefinition {
        name: "Dead Revels",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        alternative_cost: Some(spectacle(cost(&[generic(1), b()]))),
        effect: Effect::ReturnGraveyardCardsToHand {
            filter: R::Creature,
            max: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Clamor Shaman — {2}{R} 1/1 Goblin Shaman with riot. Whenever it attacks,
/// target creature an opponent controls can't block this turn.
pub fn clamor_shaman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            riot(),
            on_attack(Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            }),
        ],
        ..body(
            "Clamor Shaman",
            cost(&[generic(2), r()]),
            1,
            1,
            vec![CreatureType::Goblin, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Resolute Watchdog — {W} 1/3 Dog with defender. {1}, Sacrifice this creature:
/// Target creature you control gains indestructible until end of turn.
pub fn resolute_watchdog() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body(
            "Resolute Watchdog",
            cost(&[w()]),
            1,
            3,
            vec![CreatureType::Dog],
            vec![Keyword::Defender],
        )
    }
}

/// Tenth District Veteran — {2}{W} 2/3 Human Soldier with vigilance. Whenever it
/// attacks, untap another target creature you control.
pub fn tenth_district_veteran() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Untap {
            what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            up_to: None,
        })],
        ..body(
            "Tenth District Veteran",
            cost(&[generic(2), w()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![Keyword::Vigilance],
        )
    }
}

/// Silhana Wayfinder — {1}{G} 2/1 Elf Scout. ETB look at the top four cards; you
/// may reveal a creature or land from among them and put it on top of your
/// library. Put the rest on the bottom in a random order.
pub fn silhana_wayfinder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: R::Creature.or(R::Land),
            to: ZoneDest::Library {
                who: PlayerRef::You,
                pos: LibraryPosition::Top,
            },
            cap: Value::Const(4),
            life_per_revealed: 0,
            miss_dest: RevealMissDest::BottomRandom,
        })],
        ..body(
            "Silhana Wayfinder",
            cost(&[generic(1), g()]),
            2,
            1,
            vec![CreatureType::Elf, CreatureType::Scout],
            vec![],
        )
    }
}

// ══════════════════════════════════════════════════════════════════════════
// Batch 5 (2026-07-24): auras, guildmages, ETB value, addendum spells
// ══════════════════════════════════════════════════════════════════════════

/// Aura helper: attaches to a creature and grants a flat P/T + keyword bonus.
/// `card_kw` are keywords on the Aura spell itself (e.g. Flash).
fn aura(
    name: &'static str,
    mana: crate::mana::ManaCost,
    power: i32,
    toughness: i32,
    granted: Vec<Keyword>,
    card_kw: Vec<Keyword>,
) -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        keywords: card_kw,
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power,
            toughness,
            keywords: granted,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Basilica Bell-Haunt — {W}{W}{B}{B} 3/4 Spirit. ETB each opponent discards a
/// card and you gain 3 life.
pub fn basilica_bell_haunt() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
                random: false,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
        ]))],
        ..body(
            "Basilica Bell-Haunt",
            cost(&[w(), w(), b(), b()]),
            3,
            4,
            vec![CreatureType::Spirit],
            vec![],
        )
    }
}

/// Orzhov Enforcer — {1}{B} 1/2 Human Rogue with deathtouch and afterlife 1.
pub fn orzhov_enforcer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(1)],
        ..body(
            "Orzhov Enforcer",
            cost(&[generic(1), b()]),
            1,
            2,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![Keyword::Deathtouch],
        )
    }
}

/// Bloodmist Infiltrator — {2}{B} 3/1 Vampire. Whenever it attacks, you may
/// sacrifice another creature; if you do, it can't be blocked this turn.
pub fn bloodmist_infiltrator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MaySacrifice {
            description: "Sacrifice another creature: this can't be blocked this turn.".into(),
            filter: R::Creature.and(R::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            }),
            else_: None,
        })],
        ..body(
            "Bloodmist Infiltrator",
            cost(&[generic(2), b()]),
            3,
            1,
            vec![CreatureType::Vampire],
            vec![],
        )
    }
}

/// Lawmage's Binding — {1}{W}{U} Aura with flash. Enchanted creature can't
/// attack or block, and its activated abilities can't be activated.
pub fn lawmages_binding() -> CardDefinition {
    aura(
        "Lawmage's Binding",
        cost(&[generic(1), w(), u()]),
        0,
        0,
        vec![
            Keyword::CantAttack,
            Keyword::CantBlock,
            Keyword::CantActivateAbilities,
        ],
        vec![Keyword::Flash],
    )
}

/// Sky Tether — {W} Aura. Enchanted creature has defender and loses flying.
pub fn sky_tether() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Sky Tether",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Defender],
            remove_keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Slimebind — {1}{U} Aura with flash. Enchanted creature gets -4/-0.
pub fn slimebind() -> CardDefinition {
    aura(
        "Slimebind",
        cost(&[generic(1), u()]),
        -4,
        0,
        vec![],
        vec![Keyword::Flash],
    )
}

/// Sentinel's Mark — {1}{W} Aura with flash. Enchanted creature gets +1/+2 and
/// has vigilance. Addendum — if cast during your main phase, it gains lifelink
/// until end of turn.
pub fn sentinels_mark() -> CardDefinition {
    let mut def = aura(
        "Sentinel's Mark",
        cost(&[generic(1), w()]),
        1,
        2,
        vec![Keyword::Vigilance],
        vec![Keyword::Flash],
    );
    // Addendum: the engine auto-attaches the aura, so grant lifelink to the
    // host via a self-ETB trigger (its `attached_to` link is live by then).
    def.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::If {
            cond: Predicate::YourMainPhase,
            then: Box::new(Effect::GrantKeyword {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        },
    }];
    def
}

/// Sphinx of Foresight — {2}{U}{U} 4/4 Sphinx with flying. At the beginning of
/// your upkeep, scry 1. (The opening-hand reveal → first-upkeep scry 3 rider is
/// approximated by the recurring upkeep scry.)
pub fn sphinx_of_foresight() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        }],
        ..body(
            "Sphinx of Foresight",
            cost(&[generic(2), u(), u()]),
            4,
            4,
            vec![CreatureType::Sphinx],
            vec![Keyword::Flying],
        )
    }
}

/// Cult Guildmage — {B}{R} 2/2 Human Shaman. {3}{B}, {T}: target player
/// discards (sorcery-speed). {R}, {T}: deal 1 to target opponent or planeswalker.
pub fn cult_guildmage() -> CardDefinition {
    let opp_or_pw = R::OpponentPlayer.or(R::HasCardType(CardType::Planeswalker));
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), b()]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                    random: false,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                tap_cost: true,
                effect: deal(1, target_filtered(opp_or_pw)),
                ..Default::default()
            },
        ],
        ..body(
            "Cult Guildmage",
            cost(&[b(), r()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Syndicate Guildmage — {W}{B} 2/2 Human Cleric. {1}{W}, {T}: tap target
/// creature with power 4+. {4}{B}, {T}: deal 2 to target opponent or planeswalker.
pub fn syndicate_guildmage() -> CardDefinition {
    let opp_or_pw = R::OpponentPlayer.or(R::HasCardType(CardType::Planeswalker));
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w()]),
                tap_cost: true,
                effect: Effect::Tap {
                    what: target_filtered(R::Creature.and(R::PowerAtLeast(4))),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4), b()]),
                tap_cost: true,
                effect: deal(2, target_filtered(opp_or_pw)),
                ..Default::default()
            },
        ],
        ..body(
            "Syndicate Guildmage",
            cost(&[w(), b()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Expose to Daylight — {2}{W} Instant. Destroy target artifact or enchantment.
/// Scry 1.
pub fn expose_to_daylight() -> CardDefinition {
    CardDefinition {
        name: "Expose to Daylight",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment)),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Rally to Battle — {3}{W} Instant. Creatures you control get +1/+3 until end
/// of turn. Untap them.
pub fn rally_to_battle() -> CardDefinition {
    CardDefinition {
        name: "Rally to Battle",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                up_to: None,
            },
        ]),
        ..Default::default()
    }
}

/// Code of Constraint — {2}{U} Instant. Target creature gets -4/-0 until end of
/// turn. Draw a card. Addendum — if cast during your main phase, tap that
/// creature and it doesn't untap during its controller's next untap step.
pub fn code_of_constraint() -> CardDefinition {
    CardDefinition {
        name: "Code of Constraint",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-4),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            draw(1),
            Effect::If {
                cond: Predicate::YourMainPhase,
                then: Box::new(Effect::Seq(vec![
                    Effect::Tap {
                        what: Selector::Target(0),
                    },
                    Effect::SkipNextUntap {
                        what: Selector::Target(0),
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ══════════════════════════════════════════════════════════════════════════
// Batch 6 (2026-07-24): rares/uncommons — adapt payoffs, riot, tokens, control
// ══════════════════════════════════════════════════════════════════════════

/// Rubblebelt Runner — {1}{R}{G} 3/3 Lizard Warrior. Can't be blocked by
/// creature tokens.
pub fn rubblebelt_runner() -> CardDefinition {
    body(
        "Rubblebelt Runner",
        cost(&[generic(1), r(), g()]),
        3,
        3,
        vec![CreatureType::Lizard, CreatureType::Warrior],
        vec![Keyword::CantBeBlockedBy(Box::new(R::IsToken))],
    )
}

/// Frilled Mystic — {G}{G}{U}{U} 3/2 Elf Lizard Wizard with flash. ETB you may
/// counter target spell.
pub fn frilled_mystic() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Counter target spell.".into(),
            body: Box::new(Effect::CounterSpell {
                what: target_filtered(R::Any),
            }),
        })],
        ..body(
            "Frilled Mystic",
            cost(&[g(), g(), u(), u()]),
            3,
            2,
            vec![
                CreatureType::Elf,
                CreatureType::Lizard,
                CreatureType::Wizard,
            ],
            vec![],
        )
    }
}

/// Zegana, Utopian Speaker — {2}{G}{U} 4/4 legendary Merfolk Wizard. ETB draw a
/// card if you control another creature with a +1/+1 counter. {4}{G}{U}: Adapt
/// 4. Each creature you control with a +1/+1 counter has trample.
pub fn zegana_utopian_speaker() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource)
                        .and(R::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                n: Value::ONE,
            },
            then: Box::new(draw(1)),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g(), u()]),
            effect: adapt(4),
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a +1/+1 counter has trample.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::WithCounter(CounterType::PlusOnePlusOne),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Trample],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..body(
            "Zegana, Utopian Speaker",
            cost(&[generic(2), g(), u()]),
            4,
            4,
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Ill-Gotten Inheritance — {3}{B} Enchantment. At the beginning of your upkeep,
/// deals 1 to each opponent and you gain 1. {5}{B}, Sacrifice it: deals 4 to
/// target opponent and you gain 4.
pub fn ill_gotten_inheritance() -> CardDefinition {
    CardDefinition {
        name: "Ill-Gotten Inheritance",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                deal(1, Selector::Player(PlayerRef::EachOpponent)),
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                deal(4, target_filtered(R::OpponentPlayer)),
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(4),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Biogenic Ooze — {3}{G}{G} 2/2 Ooze. ETB create a 2/2 green Ooze. At each of
/// your end steps put a +1/+1 counter on each Ooze you control. {1}{G}{G}{G}:
/// create a 2/2 green Ooze.
pub fn biogenic_ooze() -> CardDefinition {
    let make_ooze = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: token(
            "Ooze",
            vec![Color::Green],
            2,
            2,
            vec![CreatureType::Ooze],
            vec![],
        ),
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(make_ooze()),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::End),
                    EventScope::YourControl,
                ),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Ooze).and(R::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g(), g(), g()]),
            effect: make_ooze(),
            ..Default::default()
        }],
        ..body(
            "Biogenic Ooze",
            cost(&[generic(3), g(), g()]),
            2,
            2,
            vec![CreatureType::Ooze],
            vec![],
        )
    }
}

/// Sunder Shaman — {R}{R}{G}{G} 5/5 Giant Shaman. Can't be blocked by more than
/// one creature. Whenever it deals combat damage to a player, destroy target
/// artifact or enchantment. (The "that player controls" restriction is
/// approximated as any artifact/enchantment.)
pub fn sunder_shaman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment)),
            },
        }],
        ..body(
            "Sunder Shaman",
            cost(&[r(), r(), g(), g()]),
            5,
            5,
            vec![CreatureType::Giant, CreatureType::Shaman],
            vec![Keyword::CantBeBlockedByMoreThanOne],
        )
    }
}

/// Skarrgan Hellkite — {3}{R}{R} 4/4 Dragon with riot and flying. {3}{R}: deals
/// 2 damage divided among one or two targets. Activate only if it has a +1/+1
/// counter on it.
pub fn skarrgan_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![riot()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            condition: Some(Predicate::EntityMatches {
                what: Selector::This,
                filter: R::WithCounter(CounterType::PlusOnePlusOne),
            }),
            effect: Effect::DealDamageDivided {
                retaliate_to_source: false,
                total: Value::Const(2),
                filter: R::Creature.or(R::Player).or(R::Planeswalker),
                max_targets: 2,
            },
            ..Default::default()
        }],
        ..body(
            "Skarrgan Hellkite",
            cost(&[generic(3), r(), r()]),
            4,
            4,
            vec![CreatureType::Dragon],
            vec![],
        )
    }
}

// ── RNA batch 7 (modern_decks) ──────────────────────────────────────────────

/// W/B 1/1 flying Spirit — the Orzhov afterlife/token body.
fn wb_spirit() -> TokenDefinition {
    token(
        "Spirit",
        vec![Color::White, Color::Black],
        1,
        1,
        vec![CreatureType::Spirit],
        vec![Keyword::Flying],
    )
}

/// Humongulus — {4}{U} 2/5 Homunculus with hexproof.
pub fn humongulus() -> CardDefinition {
    body(
        "Humongulus",
        cost(&[generic(4), u()]),
        2,
        5,
        vec![CreatureType::Homunculus],
        vec![Keyword::Hexproof],
    )
}

/// Gravel-Hide Goblin — {1}{R} 2/1 Goblin Shaman. {3}{G}: +2/+2 until end of turn.
pub fn gravel_hide_goblin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body(
            "Gravel-Hide Goblin",
            cost(&[generic(1), r()]),
            2,
            1,
            vec![CreatureType::Goblin, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Seraph of the Scales — {2}{W}{B} 4/3 Angel with flying and Afterlife 2.
/// {W}: gains vigilance until end of turn. {B}: gains deathtouch until end of turn.
pub fn seraph_of_the_scales() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(2)],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..body(
            "Seraph of the Scales",
            cost(&[generic(2), w(), b()]),
            4,
            3,
            vec![CreatureType::Angel],
            vec![Keyword::Flying],
        )
    }
}

/// Orzhov Racketeers — {4}{B} 3/2 Human Rogue with Afterlife 2. Whenever it
/// deals combat damage to a player, that player discards a card.
pub fn orzhov_racketeers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Discard {
                    who: Selector::Player(PlayerRef::DefendingPlayer),
                    amount: Value::Const(1),
                    random: false,
                },
            },
            afterlife(2),
        ],
        ..body(
            "Orzhov Racketeers",
            cost(&[generic(4), b()]),
            3,
            2,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Gutterbones — {B} 2/1 Skeleton Warrior. Enters tapped. {1}{B}: return this
/// from your graveyard to your hand. Activate only during your turn and only if
/// an opponent lost life this turn.
pub fn gutterbones() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            from_graveyard: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::PlayerLostLifeThisTurn {
                    who: PlayerRef::EachOpponent,
                },
            ])),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..body(
            "Gutterbones",
            cost(&[b()]),
            2,
            1,
            vec![CreatureType::Skeleton, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Knight of the Last Breath — {5}{W}{B} 4/4 Giant Knight with Afterlife 3.
/// {3}, Sacrifice another nontoken creature: create a 1/1 W/B Spirit with flying.
pub fn knight_of_the_last_breath() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(3)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sac_other_filter: Some((R::Creature.and(R::NotToken), 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: wb_spirit(),
            },
            ..Default::default()
        }],
        ..body(
            "Knight of the Last Breath",
            cost(&[generic(5), w(), b()]),
            4,
            4,
            vec![CreatureType::Giant, CreatureType::Knight],
            vec![],
        )
    }
}

/// Sphinx of the Guildpact — {7} Artifact Creature — Sphinx 5/5. All colors,
/// flying, hexproof from monocolored.
pub fn sphinx_of_the_guildpact() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        color_indicator: vec![
            Color::White,
            Color::Blue,
            Color::Black,
            Color::Red,
            Color::Green,
        ],
        keywords: vec![Keyword::Flying, Keyword::HexproofFromMonocolored],
        ..body(
            "Sphinx of the Guildpact",
            cost(&[generic(7)]),
            5,
            5,
            vec![CreatureType::Sphinx],
            vec![],
        )
    }
}

/// Azorius Skyguard — {4}{W}{U} 3/3 Human Knight with flying and first strike.
/// Creatures your opponents control get -1/-0.
pub fn azorius_skyguard() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures your opponents control get -1/-0.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature,
                power: -1,
                toughness: 0,
                keywords: vec![],
                opponents: true,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..body(
            "Azorius Skyguard",
            cost(&[generic(4), w(), u()]),
            3,
            3,
            vec![CreatureType::Human, CreatureType::Knight],
            vec![Keyword::Flying, Keyword::FirstStrike],
        )
    }
}

/// Charging War Boar — {1}{R}{G} 3/1 Boar with haste. As long as you control a
/// Domri planeswalker, it gets +1/+1 and has trample.
pub fn charging_war_boar() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as you control a Domri planeswalker, this gets +1/+1 and has trample.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasPlaneswalkerType(crate::card::PlaneswalkerSubtype::Domri)
                        .and(R::ControlledByYou),
                )),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Trample],
            },
        }],
        ..body(
            "Charging War Boar",
            cost(&[generic(1), r(), g()]),
            3,
            1,
            vec![CreatureType::Boar],
            vec![Keyword::Haste],
        )
    }
}

/// Dovin's Automaton — {4} Artifact Creature — Homunculus 3/3. As long as you
/// control a Dovin planeswalker, it gets +2/+2 and has vigilance.
pub fn dovins_automaton() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "As long as you control a Dovin planeswalker, this gets +2/+2 and has vigilance.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasPlaneswalkerType(crate::card::PlaneswalkerSubtype::Dovin)
                        .and(R::ControlledByYou),
                )),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Vigilance],
            },
        }],
        ..body(
            "Dovin's Automaton",
            cost(&[generic(4)]),
            3,
            3,
            vec![CreatureType::Homunculus],
            vec![],
        )
    }
}

/// The Haunt of Hightower — {4}{B}{B} Legendary 3/3 Vampire with flying and
/// lifelink. Whenever it attacks, defending player discards a card. Whenever a
/// card is put into an opponent's graveyard from anywhere, put a +1/+1 counter on it.
pub fn the_haunt_of_hightower() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            on_attack(Effect::Discard {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::Const(1),
                random: false,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::OpponentControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..body(
            "The Haunt of Hightower",
            cost(&[generic(4), b(), b()]),
            3,
            3,
            vec![CreatureType::Vampire],
            vec![Keyword::Flying, Keyword::Lifelink],
        )
    }
}

/// Get the Point — {3}{B}{R} Instant. Destroy target creature. Scry 1.
pub fn get_the_point() -> CardDefinition {
    CardDefinition {
        name: "Get the Point",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Creature),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Root Snare — {1}{G} Instant. Prevent all combat damage that would be dealt
/// this turn.
pub fn root_snare() -> CardDefinition {
    CardDefinition {
        name: "Root Snare",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PreventAllCombatDamageThisTurn,
        ..Default::default()
    }
}

/// Kaya's Wrath — {W}{W}{B}{B} Sorcery. Destroy all creatures. You gain life
/// equal to the number of creatures you controlled that were destroyed this way.
pub fn kayas_wrath() -> CardDefinition {
    CardDefinition {
        name: "Kaya's Wrath",
        cost: cost(&[w(), w(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(R::Creature),
                body: Box::new(Effect::Destroy {
                    what: Selector::TriggerSource,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Rampage of the Clans — {3}{G} Instant. Destroy all artifacts and
/// enchantments. For each permanent destroyed this way, its controller creates
/// a 3/3 green Centaur creature token.
pub fn rampage_of_the_clans() -> CardDefinition {
    CardDefinition {
        name: "Rampage of the Clans",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(R::Artifact.or(R::Enchantment)),
            body: Box::new(Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    count: Value::Const(1),
                    definition: token(
                        "Centaur",
                        vec![Color::Green],
                        3,
                        3,
                        vec![CreatureType::Centaur],
                        vec![],
                    ),
                },
                Effect::Destroy {
                    what: Selector::TriggerSource,
                },
            ])),
        },
        ..Default::default()
    }
}

/// Macabre Mockery — {2}{B}{R} Instant. Put target creature card from an
/// opponent's graveyard onto the battlefield under your control. It gets +2/+0
/// and gains haste. Sacrifice it at the beginning of the next end step.
pub fn macabre_mockery() -> CardDefinition {
    CardDefinition {
        name: "Macabre Mockery",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InOpponentGraveyard)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::AtNextEndStep {
                body: Box::new(Effect::SacrificePermanent {
                    what: Selector::Target(0),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Justiciar's Portal — {1}{W} Instant. Exile target creature you control, then
/// return that card to the battlefield under its owner's control. It gains
/// first strike until end of turn.
pub fn justiciars_portal() -> CardDefinition {
    CardDefinition {
        name: "Justiciar's Portal",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
            },
            Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Goblin Gathering — {2}{R} Sorcery. Create a number of 1/1 red Goblin
/// creature tokens equal to two plus the number of cards named Goblin Gathering
/// in your graveyard.
pub fn goblin_gathering() -> CardDefinition {
    CardDefinition {
        name: "Goblin Gathering",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Sum(vec![
                Value::Const(2),
                Value::count(Selector::EachMatching {
                    zone: ZoneRef::Graveyard(PlayerRef::You),
                    filter: R::HasName("Goblin Gathering".into()),
                }),
            ]),
            definition: token(
                "Goblin",
                vec![Color::Red],
                1,
                1,
                vec![CreatureType::Goblin],
                vec![],
            ),
        },
        ..Default::default()
    }
}

/// Gates Ablaze — {2}{R} Sorcery. Deals X damage to each creature, where X is
/// the number of Gates you control.
pub fn gates_ablaze() -> CardDefinition {
    CardDefinition {
        name: "Gates Ablaze",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: each_creature(),
            amount: Value::count(Selector::EachPermanent(
                R::HasLandType(LandType::Gate).and(R::ControlledByYou),
            )),
        },
        ..Default::default()
    }
}

/// Undercity's Embrace — {2}{B} Instant. Target opponent sacrifices a creature
/// of their choice. If you control a creature with power 4 or greater, you gain
/// 4 life. (The single "target opponent" is modeled as each opponent — exact in 1v1.)
pub fn undercitys_embrace() -> CardDefinition {
    CardDefinition {
        name: "Undercity's Embrace",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: R::Creature,
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                )),
                then: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(4),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Glass of the Guildpact — {2} Artifact. Multicolored creatures you control
/// get +1/+1.
pub fn glass_of_the_guildpact() -> CardDefinition {
    CardDefinition {
        name: "Glass of the Guildpact",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Multicolored creatures you control get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::Multicolored),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// High Alert — {1}{W}{U} Enchantment. Each creature you control assigns combat
/// damage equal to its toughness rather than its power. Creatures you control
/// can attack as though they didn't have defender. {2}{W}{U}: untap target creature.
pub fn high_alert() -> CardDefinition {
    CardDefinition {
        name: "High Alert",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Each creature you control assigns combat damage equal to its toughness rather than its power.",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Creature,
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::AssignsCombatDamageByToughness],
                    opponents: false,
                    all_players: false,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
            StaticAbility {
                description: "Creatures you control can attack as though they didn't have defender.",
                effect: StaticEffect::YourCreaturesCanAttackAsThoughNoDefender,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w(), u()]),
            effect: Effect::Untap {
                what: target_filtered(R::Creature),
                up_to: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Depose // Deploy — {1}{W/U} // {2}{W}{U} Instant // Instant. Depose taps a
/// target creature and draws a card; Deploy makes two 1/1 flying Thopters and
/// gains 1 life for each creature you control.
pub fn depose_deploy() -> CardDefinition {
    CardDefinition {
        name: "Depose // Deploy",
        cost: cost(&[generic(1), hybrid(Color::White, Color::Blue)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(R::Creature),
            },
            draw(1),
        ]),
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(2), w(), u()]),
                card_types: vec![CardType::Instant],
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(2),
                        definition: token(
                            "Thopter",
                            vec![],
                            1,
                            1,
                            vec![CreatureType::Thopter],
                            vec![Keyword::Flying],
                        ),
                    },
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::count(each_your_creature()),
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Consecrate // Consume — {1}{W/B} // {2}{W}{B} Instant // Sorcery. Consecrate
/// exiles a card from a graveyard and draws; Consume makes a player sacrifice
/// their greatest-power creature and gains you life equal to its power.
pub fn consecrate_consume() -> CardDefinition {
    CardDefinition {
        name: "Consecrate // Consume",
        cost: cost(&[generic(1), hybrid(Color::White, Color::Black)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(R::InGraveyard),
            },
            draw(1),
        ]),
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(2), w(), b()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::SacrificeGreatestMV {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        count: Value::ONE,
                        filter: R::Creature,
                        by_power: true,
                    },
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::SacrificedPower,
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Warrant // Warden — {W/U}{W/U} // {3}{W}{U} Instant // Sorcery. Warrant puts
/// a target attacking or blocking creature on top of its owner's library;
/// Warden makes a 4/4 flying, vigilant Sphinx.
pub fn warrant_warden() -> CardDefinition {
    CardDefinition {
        name: "Warrant // Warden",
        cost: cost(&[
            hybrid(Color::White, Color::Blue),
            hybrid(Color::White, Color::Blue),
        ]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOfMoved,
                pos: LibraryPosition::Top,
            },
        },
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(3), w(), u()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: token(
                        "Sphinx",
                        vec![Color::White, Color::Blue],
                        4,
                        4,
                        vec![CreatureType::Sphinx],
                        vec![Keyword::Flying, Keyword::Vigilance],
                    ),
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Thrash // Threat — {R/G}{R/G} // {2}{R}{G} Instant // Sorcery. Thrash has
/// target creature you control deal damage equal to its power to a creature or
/// planeswalker you don't control; Threat makes a 4/4 trampling Beast.
pub fn thrash_threat() -> CardDefinition {
    CardDefinition {
        name: "Thrash // Threat",
        cost: cost(&[
            hybrid(Color::Red, Color::Green),
            hybrid(Color::Red, Color::Green),
        ]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamageEqualToPower {
            source: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.and(R::ControlledByYou),
            },
            target: Selector::TargetFiltered {
                slot: 1,
                filter: (R::Creature.or(R::Planeswalker)).and(R::ControlledByOpponent),
            },
        },
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(2), r(), g()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: token(
                        "Beast",
                        vec![Color::Red, Color::Green],
                        4,
                        4,
                        vec![CreatureType::Beast],
                        vec![Keyword::Trample],
                    ),
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Collision // Colossus — {1}{R/G} // {R}{G} Instant // Instant. Collision
/// deals 6 damage to a target creature with flying; Colossus gives a creature
/// +4/+2 and trample until end of turn.
pub fn collision_colossus() -> CardDefinition {
    CardDefinition {
        name: "Collision // Colossus",
        cost: cost(&[generic(1), hybrid(Color::Red, Color::Green)]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            amount: Value::Const(6),
        },
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[r(), g()]),
                card_types: vec![CardType::Instant],
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature),
                        power: Value::Const(4),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Carnival // Carnage — {B/R} // {2}{B}{R} Instant // Sorcery. Carnival deals 1
/// damage to a target creature or planeswalker and 1 to its controller; Carnage
/// deals 3 to a target opponent, who discards two cards.
pub fn carnival_carnage() -> CardDefinition {
    CardDefinition {
        name: "Carnival // Carnage",
        cost: cost(&[hybrid(Color::Black, Color::Red)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::Planeswalker)),
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(1),
            },
        ]),
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(2), b(), r()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::TargetFiltered {
                            slot: 0,
                            filter: R::OpponentPlayer,
                        },
                        amount: Value::Const(3),
                    },
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(
                            Selector::TargetFiltered {
                                slot: 0,
                                filter: R::OpponentPlayer,
                            },
                        ))),
                        amount: Value::Const(2),
                        random: false,
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

// ── RNA batch 8 (modern_decks) ──────────────────────────────────────────────

/// Clan Guildmage — {R}{G} 2/2 Human Shaman. {1}{R}, {T}: target creature can't
/// block this turn. {2}{G}, {T}: target land you control becomes a 4/4
/// Elemental with haste until end of turn. It's still a land.
pub fn clan_guildmage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                tap_cost: true,
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g()]),
                tap_cost: true,
                effect: Effect::BecomeCreature {
                    what: target_filtered(R::Land.and(R::ControlledByYou)),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    creature_types: vec![CreatureType::Elemental],
                    keywords: vec![Keyword::Haste],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..body(
            "Clan Guildmage",
            cost(&[r(), g()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Tin Street Dodger — {R} 1/1 Goblin Rogue with haste. {R}: this creature can't
/// be blocked this turn except by creatures with defender.
pub fn tin_street_dodger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::CantBeBlockedExceptBy(Box::new(R::HasKeyword(Keyword::Defender))),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..body(
            "Tin Street Dodger",
            cost(&[r()]),
            1,
            1,
            vec![CreatureType::Goblin, CreatureType::Rogue],
            vec![Keyword::Haste],
        )
    }
}

/// Fireblade Artist — {B}{R} 2/2 Human Shaman with haste. At the beginning of
/// your upkeep, you may sacrifice a creature. When you do, this creature deals 2
/// damage to target opponent or planeswalker.
pub fn fireblade_artist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MaySacrifice {
                description: "You may sacrifice a creature; if you do, deal 2 damage to target opponent or planeswalker.".into(),
                filter: R::Creature,
                count: Value::Const(1),
                then: Box::new(Effect::DealDamage {
                    to: target_filtered(R::OpponentPlayer.or(R::Planeswalker)),
                    amount: Value::Const(2),
                }),
                else_: None,
            },
        }],
        ..body("Fireblade Artist", cost(&[b(), r()]), 2, 2, vec![CreatureType::Human, CreatureType::Shaman], vec![Keyword::Haste])
    }
}

/// Saruli Caretaker — {G} 0/3 Dryad with defender. {T}, Tap an untapped
/// creature you control: add one mana of any color.
pub fn saruli_caretaker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            tap_other_filter: Some(R::Creature.and(R::ControlledByYou)),
            effect: crate::effect::shortcut::add_any_one_color(1),
            ..Default::default()
        }],
        ..body(
            "Saruli Caretaker",
            cost(&[g()]),
            0,
            3,
            vec![CreatureType::Dryad],
            vec![Keyword::Defender],
        )
    }
}

/// Gate Colossus — {8} Artifact Creature — Construct 8/8. Affinity for Gates.
/// Can't be blocked by creatures with power 2 or less. Whenever a Gate you
/// control enters, you may put this card from your graveyard on top of your library.
pub fn gate_colossus() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        affinity_filter: Some(R::HasLandType(LandType::Gate)),
        keywords: vec![Keyword::CantBeBlockedByPowerAtMost(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::FromYourGraveyard)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasLandType(LandType::Gate),
                }),
            effect: Effect::MayDo {
                description: "Put Gate Colossus from your graveyard on top of your library.".into(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library {
                        who: PlayerRef::You,
                        pos: LibraryPosition::Top,
                    },
                }),
            },
        }],
        ..body(
            "Gate Colossus",
            cost(&[generic(8)]),
            8,
            8,
            vec![CreatureType::Construct],
            vec![],
        )
    }
}

/// Persistent Petitioners — {1}{U} 1/3 Human Advisor. {1}, {T}: target player
/// mills a card. Tap four untapped Advisors you control: target player mills
/// twelve cards.
pub fn persistent_petitioners() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_n_filter: Some((R::HasCreatureType(CreatureType::Advisor), 4)),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(12),
                },
                ..Default::default()
            },
        ],
        ..body(
            "Persistent Petitioners",
            cost(&[generic(1), u()]),
            1,
            3,
            vec![CreatureType::Human, CreatureType::Advisor],
            vec![],
        )
    }
}

/// Bedeck // Bedazzle — {B/R}{B/R} // {4}{B}{R} Instant // Instant. Bedeck gives
/// a creature +3/-3; Bedazzle destroys a nonbasic land and deals 2 to a target
/// opponent or planeswalker.
pub fn bedeck_bedazzle() -> CardDefinition {
    CardDefinition {
        name: "Bedeck // Bedazzle",
        cost: cost(&[
            hybrid(Color::Black, Color::Red),
            hybrid(Color::Black, Color::Red),
        ]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        },
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(4), b(), r()]),
                card_types: vec![CardType::Instant],
                effect: Effect::Seq(vec![
                    Effect::Destroy {
                        what: Selector::TargetFiltered {
                            slot: 0,
                            filter: R::Land
                                .and(R::NotToken)
                                .and(R::Not(Box::new(R::IsBasicLand))),
                        },
                    },
                    Effect::DealDamage {
                        to: Selector::TargetFiltered {
                            slot: 1,
                            filter: R::OpponentPlayer.or(R::Planeswalker),
                        },
                        amount: Value::Const(2),
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Incubation // Incongruity — {G/U} // {1}{G}{U} Sorcery // Instant. Incubation
/// looks at the top five, revealing a creature to hand; Incongruity exiles a
/// creature and its controller creates a 3/3 green Frog Lizard.
pub fn incubation_incongruity() -> CardDefinition {
    CardDefinition {
        name: "Incubation // Incongruity",
        cost: cost(&[hybrid(Color::Green, Color::Blue)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_to_graveyard: false,
            pick_filter: Some(R::Creature),
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: true,
            picked_lands_to_battlefield: false,
            rest_bottom_random: true,
        },
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(1), g(), u()]),
                card_types: vec![CardType::Instant],
                effect: Effect::Seq(vec![
                    Effect::Exile {
                        what: target_filtered(R::Creature),
                    },
                    Effect::CreateToken {
                        who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                        count: Value::Const(1),
                        definition: token(
                            "Frog Lizard",
                            vec![Color::Green],
                            3,
                            3,
                            vec![CreatureType::Frog, CreatureType::Lizard],
                            vec![],
                        ),
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Repudiate // Replicate — {G/U}{G/U} // {1}{G}{U} Instant // Sorcery.
/// Repudiate counters a target activated or triggered ability; Replicate makes
/// a token copy of a target creature you control.
pub fn repudiate_replicate() -> CardDefinition {
    CardDefinition {
        name: "Repudiate // Replicate",
        cost: cost(&[
            hybrid(Color::Green, Color::Blue),
            hybrid(Color::Green, Color::Blue),
        ]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterAbility {
            what: target_filtered(R::Permanent),
        },
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(1), g(), u()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: target_filtered(R::Creature.and(R::ControlledByYou)),
                    extra_keywords: vec![],
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Sharktocrab — {2}{G}{U} 4/4 Shark Octopus Crab. {2}{G}{U}: Adapt 1. Whenever
/// one or more +1/+1 counters are put on it, tap target creature an opponent
/// controls; it doesn't untap during its controller's next untap step.
pub fn sharktocrab() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), u()]),
            effect: adapt(1),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::SelfSource,
            ),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..body(
            "Sharktocrab",
            cost(&[generic(2), g(), u()]),
            4,
            4,
            vec![
                CreatureType::Shark,
                CreatureType::Octopus,
                CreatureType::Crab,
            ],
            vec![],
        )
    }
}

/// Growth-Chamber Guardian — {1}{G} 2/2 Elf Crab Warrior. {2}{G}: Adapt 2.
/// Whenever one or more +1/+1 counters are put on it, you may search your
/// library for a card named Growth-Chamber Guardian, reveal it, put it into
/// your hand, then shuffle.
pub fn growth_chamber_guardian() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: adapt(2),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::SelfSource,
            ),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasName("Growth-Chamber Guardian".into()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..body(
            "Growth-Chamber Guardian",
            cost(&[generic(1), g()]),
            2,
            2,
            vec![CreatureType::Elf, CreatureType::Crab, CreatureType::Warrior],
            vec![],
        )
    }
}

// ── Batch 9 (2026-07-24): spectacle payoffs, threaten, wraths, targeting/──────
//    deathtouch statics, addendum, defender-adapt, modal bounce. ──────────────

/// Rix Maadi Reveler — {1}{R} 2/2 Human Shaman with Spectacle {2}{B}{R}. ETB:
/// discard a card, then draw a card; if the spectacle cost was paid, instead
/// discard your hand, then draw three.
pub fn rix_maadi_reveler() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::card::AlternativeCost {
            awaken: false,
            marks_kicked: true,
            ..spectacle(cost(&[generic(2), b(), r()]))
        }),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
            ])),
            else_: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ])),
        })],
        ..body(
            "Rix Maadi Reveler",
            cost(&[generic(1), r()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Rafter Demon — {2}{B}{R} 4/2 Demon with Spectacle {3}{B}{R}. ETB, if the
/// spectacle cost was paid, each opponent discards a card.
pub fn rafter_demon() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::card::AlternativeCost {
            awaken: false,
            marks_kicked: true,
            ..spectacle(cost(&[generic(3), b(), r()]))
        }),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
                random: false,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..body(
            "Rafter Demon",
            cost(&[generic(2), b(), r()]),
            4,
            2,
            vec![CreatureType::Demon],
            vec![],
        )
    }
}

/// Hackrobat — {1}{B}{R} 2/3 Human Rogue with Spectacle {B}{R}. {B}: gains
/// deathtouch until end of turn. {R}: +2/-2 until end of turn.
pub fn hackrobat() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(spectacle(cost(&[b(), r()]))),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..body(
            "Hackrobat",
            cost(&[generic(1), b(), r()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Gruul Spellbreaker — {1}{R}{G} 3/3 Ogre Warrior with Riot and trample.
/// During your turn, you and this creature have hexproof.
pub fn gruul_spellbreaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![riot()],
        static_abilities: vec![
            StaticAbility {
                description: "During your turn, you have hexproof.",
                effect: StaticEffect::WhileYourTurn {
                    inner: Box::new(StaticEffect::ControllerHasHexproof),
                },
            },
            StaticAbility {
                description: "During your turn, this creature has hexproof.",
                effect: StaticEffect::WhileYourTurn {
                    inner: Box::new(StaticEffect::GrantKeyword {
                        applies_to: Selector::This,
                        keyword: Keyword::Hexproof,
                    }),
                },
            },
        ],
        ..body(
            "Gruul Spellbreaker",
            cost(&[generic(1), r(), g()]),
            3,
            3,
            vec![CreatureType::Ogre, CreatureType::Warrior],
            vec![Keyword::Trample],
        )
    }
}

/// Smelt-Ward Ignus — {1}{R} 2/1 Elemental. {2}{R}, Sacrifice this creature:
/// Gain control of target creature with power 3 or less until end of turn,
/// untap it, it gains haste. Activate only as a sorcery.
pub fn smelt_ward_ignus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::GainControl {
                    what: target_filtered(R::Creature.and(R::PowerAtMost(3))),
                    to: None,
                    duration: Duration::EndOfTurn,
                },
                Effect::Untap {
                    what: Selector::Target(0),
                    up_to: None,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..body(
            "Smelt-Ward Ignus",
            cost(&[generic(1), r()]),
            2,
            1,
            vec![CreatureType::Elemental],
            vec![],
        )
    }
}

/// Sphinx of New Prahv — {W}{W}{U}{U} 4/3 Sphinx with flying and vigilance.
/// Spells your opponents cast that target it cost {2} more.
pub fn sphinx_of_new_prahv() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells your opponents cast that target this creature cost {2} more to cast.",
            effect: StaticEffect::TaxOpponentSpellsTargetingThis { amount: 2 },
        }],
        ..body(
            "Sphinx of New Prahv",
            cost(&[w(), w(), u(), u()]),
            4,
            3,
            vec![CreatureType::Sphinx],
            vec![Keyword::Flying, Keyword::Vigilance],
        )
    }
}

/// Pestilent Spirit — {2}{B} 3/2 Spirit with menace and deathtouch. Instant and
/// sorcery spells you control have deathtouch.
pub fn pestilent_spirit() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you control have deathtouch.",
            effect: StaticEffect::YourISSpellsHaveDeathtouch,
        }],
        ..body(
            "Pestilent Spirit",
            cost(&[generic(2), b()]),
            3,
            2,
            vec![CreatureType::Spirit],
            vec![Keyword::Menace, Keyword::Deathtouch],
        )
    }
}

/// Scuttlegator — {4}{G/U}{G/U} 6/6 Crab Turtle Crocodile with defender.
/// {6}{G/U}{G/U}: Adapt 3. As long as it has a +1/+1 counter, it can attack as
/// though it didn't have defender.
pub fn scuttlegator() -> CardDefinition {
    let gu = || hybrid(Color::Green, Color::Blue);
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), gu(), gu()]),
            effect: adapt(3),
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature has a +1/+1 counter on it, it can attack as though it didn't have defender.",
            effect: StaticEffect::CanAttackIgnoringDefenderWhile {
                condition: Predicate::SourceHasCountersAtLeast {
                    counter: CounterType::PlusOnePlusOne,
                    n: 1,
                },
            },
        }],
        ..body(
            "Scuttlegator",
            cost(&[generic(4), gu(), gu()]),
            6,
            6,
            vec![
                CreatureType::Crab,
                CreatureType::Turtle,
                CreatureType::Crocodile,
            ],
            vec![Keyword::Defender],
        )
    }
}

/// Angelic Exaltation — {3}{W} Enchantment. Whenever a creature you control
/// attacks alone, it gets +X/+X until end of turn, where X is the number of
/// creatures you control.
pub fn angelic_exaltation() -> CardDefinition {
    CardDefinition {
        name: "Angelic Exaltation",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::AttackingAlone),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::CreatureCountControlledBy(PlayerRef::You),
                toughness: Value::CreatureCountControlledBy(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Ethereal Absolution — {4}{W}{B} Enchantment. Creatures you control get
/// +1/+1; creatures your opponents control get -1/-1. {2}{W}{B}: Exile target
/// card from an opponent's graveyard; if it was a creature card, create a 1/1
/// W/B Spirit with flying.
pub fn ethereal_absolution() -> CardDefinition {
    CardDefinition {
        name: "Ethereal Absolution",
        cost: cost(&[generic(4), w(), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Creatures your opponents control get -1/-1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                    power: -1,
                    toughness: -1,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w(), b()]),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::InGraveyard.and(R::ControlledByOpponent)),
                    to: ZoneDest::Exile,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountMatching {
                            sel: Box::new(Selector::LastMoved),
                            filter: R::Creature,
                        },
                        Value::ONE,
                    ),
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: TokenDefinition {
                            name: "Spirit".into(),
                            power: 1,
                            toughness: 1,
                            colors: vec![Color::White, Color::Black],
                            card_types: vec![CardType::Creature],
                            subtypes: creatures(vec![CreatureType::Spirit]),
                            keywords: vec![Keyword::Flying],
                            ..Default::default()
                        },
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cry of the Carnarium — {1}{B}{B} Sorcery. All creatures get -2/-2 until end
/// of turn. If a creature would die this turn, exile it instead. (The exile of
/// creatures already in graveyards this turn is elided.)
pub fn cry_of_the_carnarium() -> CardDefinition {
    CardDefinition {
        name: "Cry of the Carnarium",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::ExileIfWouldDieThisTurn {
                what: Selector::EachPermanent(R::Creature),
            },
        ]),
        ..Default::default()
    }
}

/// Pitiless Pontiff — {W}{B} 2/2 Vampire Cleric. {1}, Sacrifice another
/// creature: This creature gains deathtouch and indestructible until end of
/// turn.
pub fn pitiless_pontiff() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..body(
            "Pitiless Pontiff",
            cost(&[w(), b()]),
            2,
            2,
            vec![CreatureType::Vampire, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Unbreakable Formation — {2}{W} Instant. Creatures you control gain
/// indestructible until end of turn. Addendum — if cast during your main
/// phase, put a +1/+1 counter on each of those creatures and they gain
/// vigilance until end of turn.
pub fn unbreakable_formation() -> CardDefinition {
    CardDefinition {
        name: "Unbreakable Formation",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: each_your_creature(),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::YourMainPhase,
                then: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: each_your_creature(),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::GrantKeyword {
                        what: each_your_creature(),
                        keyword: Keyword::Vigilance,
                        duration: Duration::EndOfTurn,
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Flames of the Raze-Boar — {5}{R} Instant. Deals 4 damage to target creature
/// an opponent controls. Then deals 2 damage to each other creature that
/// player controls if you control a creature with power 4 or greater. (The
/// second wave hits all that player's creatures; the "other" exclusion of the
/// 4-damage target — usually already dead — is elided.)
pub fn flames_of_the_raze_boar() -> CardDefinition {
    CardDefinition {
        name: "Flames of the Raze-Boar",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                amount: Value::Const(4),
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
                    Value::Const(4),
                ),
                then: Box::new(Effect::DealDamage {
                    to: Selector::ControlledBy {
                        who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                        filter: R::Creature,
                    },
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Swirling Torrent — {5}{U} Sorcery. Choose one or both — put target creature
/// on top of its owner's library; and/or return target creature to its owner's
/// hand.
pub fn swirling_torrent() -> CardDefinition {
    CardDefinition {
        name: "Swirling Torrent",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOfMoved,
                        pos: LibraryPosition::Top,
                    },
                },
                Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Mesmerizing Benthid — {3}{U}{U} 4/5 Octopus. ETB create two 0/2 blue Illusion
/// tokens whose block stuns the blocked creature. It has hexproof as long as
/// you control an Illusion.
pub fn mesmerizing_benthid() -> CardDefinition {
    let illusion = || TokenDefinition {
        name: "Illusion".into(),
        power: 0,
        toughness: 2,
        colors: vec![Color::Blue],
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Illusion]),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::BlockedAttacker,
                kind: CounterType::Stun,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: illusion(),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: illusion(),
            },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "This creature has hexproof as long as you control an Illusion.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Hexproof,
                condition: Predicate::ValueAtLeast(
                    Value::count(Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Illusion).and(R::ControlledByYou),
                    )),
                    Value::Const(1),
                ),
            },
        }],
        ..body(
            "Mesmerizing Benthid",
            cost(&[generic(3), u(), u()]),
            4,
            5,
            vec![CreatureType::Octopus],
            vec![],
        )
    }
}

/// Immolation Shaman — {1}{R} 1/3 Lizard Shaman. Whenever an opponent activates
/// a non-mana ability of an artifact, creature, or land, deal 1 to that player.
/// {3}{R}{R}: +3/+3 and menace until end of turn.
pub fn immolation_shaman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivated, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.or(R::Creature).or(R::Land),
                }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r(), r()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..body(
            "Immolation Shaman",
            cost(&[generic(1), r()]),
            1,
            3,
            vec![CreatureType::Lizard, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Screaming Shield — {1} Equipment. Equipped creature gets +0/+3 and has
/// "{2}, {T}: Target player mills three cards." Equip {3}.
pub fn screaming_shield() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Screaming Shield",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 0,
            toughness: 3,
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(3),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Clear the Stage — {4}{B} Instant. Target creature gets -3/-3 until end of
/// turn. If you control a creature with power 4 or greater, you may return up
/// to one creature card from your graveyard to your hand. (The return is a
/// resolution-time pick rather than a chosen target.)
pub fn clear_the_stage() -> CardDefinition {
    CardDefinition {
        name: "Clear the Stage",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
                    Value::Const(4),
                ),
                then: Box::new(Effect::ReturnGraveyardCardsToHand {
                    filter: R::Creature,
                    max: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Domri's Nodorog — {3}{R}{G} 5/2 Beast with Riot. ETB you may search your
/// library for a card named Domri, City Smasher, reveal it, and put it into
/// your hand, then shuffle. (The graveyard half is elided.)
pub fn domris_nodorog() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            riot(),
            etb(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasName("Domri, City Smasher".into()),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        ],
        ..body(
            "Domri's Nodorog",
            cost(&[generic(3), r(), g()]),
            5,
            2,
            vec![CreatureType::Beast],
            vec![Keyword::Trample],
        )
    }
}

/// Bolrac-Clan Crusher — {3}{R}{G} 4/4 Ogre Warrior. {T}, Remove a +1/+1
/// counter from a creature you control: deal 2 damage to any target.
pub fn bolrac_clan_crusher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_among_filter: Some((
                Some(CounterType::PlusOnePlusOne),
                1,
                R::Creature.and(R::ControlledByYou),
            )),
            effect: deal(2, target_any()),
            ..Default::default()
        }],
        ..body(
            "Bolrac-Clan Crusher",
            cost(&[generic(3), r(), g()]),
            4,
            4,
            vec![CreatureType::Ogre, CreatureType::Warrior],
            vec![],
        )
    }
}

/// Dovin's Acuity — {1}{W}{U} Enchantment. ETB gain 2 life and draw a card.
/// Whenever you cast an instant spell during your main phase, you may return
/// this to its owner's hand.
pub fn dovins_acuity() -> CardDefinition {
    CardDefinition {
        name: "Dovin's Acuity",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCardType(CardType::Instant),
                    },
                ),
                effect: Effect::If {
                    cond: Predicate::YourMainPhase,
                    then: Box::new(Effect::MayDo {
                        description: "Return Dovin's Acuity to its owner's hand.".into(),
                        body: Box::new(Effect::Move {
                            what: Selector::This,
                            to: ZoneDest::Hand(PlayerRef::You),
                        }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Dovin's Dismissal — {2}{W}{U} Instant. Put up to one target tapped creature
/// on top of its owner's library. You may search your library for a card named
/// Dovin, Architect of Law and put it into your hand, then shuffle. (The
/// graveyard half is elided.)
pub fn dovins_dismissal() -> CardDefinition {
    CardDefinition {
        name: "Dovin's Dismissal",
        cost: cost(&[generic(2), w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::Tapped),
                    },
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOfMoved,
                        pos: LibraryPosition::Top,
                    },
                }),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: R::HasName("Dovin, Architect of Law".into()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Eyes Everywhere — {2}{U} Enchantment. At the beginning of your upkeep, scry
/// 1. {5}{U}: Exchange control of this enchantment and target nonland
/// permanent. Activate only as a sorcery.
pub fn eyes_everywhere() -> CardDefinition {
    CardDefinition {
        name: "Eyes Everywhere",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u()]),
            sorcery_speed: true,
            effect: Effect::ExchangeControl {
                a: Selector::This,
                b: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Nonland,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Revival // Revenge — {W/B}{W/B} // {4}{W}{B} Sorcery // Sorcery. Revival
/// returns a creature with mana value 3 or less from your graveyard to the
/// battlefield; Revenge makes each opponent lose half their life (rounded up)
/// and you gain that much. (The gain reads life the opponent lost this turn —
/// exact in a duel.)
pub fn revival_revenge() -> CardDefinition {
    CardDefinition {
        name: "Revival // Revenge",
        cost: cost(&[
            hybrid(Color::White, Color::Black),
            hybrid(Color::White, Color::Black),
        ]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(R::Creature.and(R::InGraveyard).and(R::ManaValueAtMost(3))),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(4), w(), b()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::LoseHalfLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        rounded_up: true,
                    },
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::LifeLostThisTurn(PlayerRef::EachOpponent),
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Plaza of Harmony — Gate land. ETB: gain 3 life if you control two or more
/// Gates. {T}: add {C}. {T}: add one mana of any color a Gate you control could
/// produce (approximated to any color).
pub fn plaza_of_harmony() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Plaza of Harmony",
        cost: cost(&[]),
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Gate],
            ..Default::default()
        },
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    R::HasLandType(LandType::Gate).and(R::ControlledByYou),
                ),
                n: Value::Const(2),
            },
            then: Box::new(Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasLandType(LandType::Gate).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                }),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Emergency Powers — {5}{W}{U} Instant. Each player shuffles their hand and
/// graveyard into their library, then draws seven, then exile this. Addendum —
/// cast in your main phase: you may put a permanent card with mana value 7 or
/// less from your hand onto the battlefield.
pub fn emergency_powers() -> CardDefinition {
    CardDefinition {
        name: "Emergency Powers",
        cost: cost(&[generic(5), w(), u()]),
        card_types: vec![CardType::Instant],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::ShuffleHandAndGraveyardIntoLibrary {
                who: PlayerRef::EachPlayer,
            },
            Effect::Draw {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(7),
            },
            Effect::If {
                cond: Predicate::YourMainPhase,
                then: Box::new(Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Permanent.and(R::ManaValueAtMost(7)),
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Awaken the Erstwhile — {3}{B}{B} Sorcery. Each player discards their hand,
/// then creates that many 2/2 black Zombie tokens.
pub fn awaken_the_erstwhile() -> CardDefinition {
    CardDefinition {
        name: "Awaken the Erstwhile",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::EachPlayerDiscardsHandMakeTokens {
            token: token(
                "Zombie",
                vec![Color::Black],
                2,
                2,
                vec![CreatureType::Zombie],
                vec![],
            ),
        },
        ..Default::default()
    }
}

/// Hydroid Krasis — {X}{G}{U} 0/0 Jellyfish Hydra Beast with flying + trample.
/// Enters with X +1/+1 counters. When you cast it, gain half X life and draw
/// half X cards (rounded down), even if it's countered.
pub fn hydroid_krasis() -> CardDefinition {
    CardDefinition {
        name: "Hydroid Krasis",
        cost: cost(&[x(), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![
            CreatureType::Jellyfish,
            CreatureType::Hydra,
            CreatureType::Beast,
        ]),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::HalvedRoundDown(Box::new(Value::XFromCost)),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::HalvedRoundDown(Box::new(Value::XFromCost)),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Kaya, Orzhov Usurper — {1}{W}{B} loyalty-3 Planeswalker. +1: exile up to two
/// target cards from a single graveyard. −1: exile target nonland permanent with
/// mana value 1 or less. −5: deal damage to target player equal to the cards
/// they own in exile and gain that much. (The +1 "gain 2 if a creature was
/// exiled" rider and the −5's "target player" → opponent are minor collapses.)
pub fn kaya_orzhov_usurper() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype};
    CardDefinition {
        name: "Kaya, Orzhov Usurper",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Kaya],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::ExileUpToNFromGraveyards {
                    count: Value::Const(2),
                    of: None,
                    single: true,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::Move {
                    what: target_filtered(R::Nonland.and(R::ManaValueAtMost(1))),
                    to: ZoneDest::Exile,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -5,
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::TargetFiltered {
                            slot: 0,
                            filter: R::OpponentPlayer,
                        },
                        amount: Value::CardsInExileOwnedBy(PlayerRef::Target(0)),
                    },
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::CardsInExileOwnedBy(PlayerRef::Target(0)),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Biomancer's Familiar — {G}{U} 2/2 Mutant. Activated abilities of creatures
/// you control cost {2} less (never below one mana). (Its {T} adapt-reset rider
/// is omitted — no "adapt as though it had no counters" primitive yet.)
pub fn biomancers_familiar() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Activated abilities of creatures you control cost {2} less to activate.",
            effect: StaticEffect::YourCreatureActivatedAbilitiesCostLess { amount: 2 },
        }],
        ..body(
            "Biomancer's Familiar",
            cost(&[g(), u()]),
            2,
            2,
            vec![CreatureType::Mutant],
            vec![],
        )
    }
}

/// Incubation Druid — {1}{G} 0/2 Elf Druid. {T}: add one mana of any color
/// (any type a land could produce, approximated); three instead while it has a
/// +1/+1 counter. {3}{G}{G}: Adapt 3.
pub fn incubation_druid() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::IfPred {
                        pred: Box::new(Predicate::EntityMatches {
                            what: Selector::This,
                            filter: R::WithCounter(CounterType::PlusOnePlusOne),
                        }),
                        then: Box::new(Value::Const(3)),
                        else_: Box::new(Value::ONE),
                    }),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), g(), g()]),
                effect: adapt(3),
                ..Default::default()
            },
        ],
        ..body(
            "Incubation Druid",
            cost(&[generic(1), g()]),
            0,
            2,
            vec![CreatureType::Elf, CreatureType::Druid],
            vec![],
        )
    }
}

/// Ravager Wurm — {3}{R}{G}{G} 4/5 Wurm with Riot. ETB, choose up to one — it
/// fights target creature you don't control. (The "destroy a land with a
/// non-mana activated ability" mode is omitted — no such land filter yet.)
pub fn ravager_wurm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            riot(),
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Fight {
                    attacker: Selector::This,
                    defender: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    },
                }),
            }),
        ],
        ..body(
            "Ravager Wurm",
            cost(&[generic(3), r(), g(), g()]),
            4,
            5,
            vec![CreatureType::Wurm],
            vec![],
        )
    }
}

/// Rakdos, the Showstopper — {4}{B}{R} 6/6 Demon with flying + trample. ETB:
/// flip a coin for each creature that isn't a Demon, Devil, or Imp; destroy
/// each whose coin comes up tails.
pub fn rakdos_the_showstopper() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::CoinFlipEachCreatureDestroyOnTails {
            exclude_types: vec![CreatureType::Demon, CreatureType::Devil, CreatureType::Imp],
        })],
        ..body(
            "Rakdos, the Showstopper",
            cost(&[generic(4), b(), r()]),
            6,
            6,
            vec![CreatureType::Demon],
            vec![Keyword::Flying, Keyword::Trample],
        )
    }
}

/// Deputy of Detention — {1}{W}{U} 1/3 Vedalken Wizard. ETB: exile target
/// nonland permanent an opponent controls until Deputy leaves. (Same-name
/// grouping is approximated to the single target, as with Detention Sphere.)
pub fn deputy_of_detention() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Nonland.and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..body(
            "Deputy of Detention",
            cost(&[generic(1), w(), u()]),
            1,
            3,
            vec![CreatureType::Vedalken, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Prime Speaker Vannifar — {2}{G}{U} 2/4 Elf Ooze Wizard. {T}, sacrifice
/// another creature (sorcery speed): search your library for a creature with
/// mana value 1 greater, put it onto the battlefield, then shuffle.
pub fn prime_speaker_vannifar() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::OtherThanSource),
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::ManaValueEqualsSacrificedPlus(1)),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ]),
            ..Default::default()
        }],
        ..body(
            "Prime Speaker Vannifar",
            cost(&[generic(2), g(), u()]),
            2,
            4,
            vec![CreatureType::Elf, CreatureType::Ooze, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Font of Agonies — {B} Enchantment. Whenever you pay life, put that many
/// blood counters on it. {1}{B}, remove four blood counters: destroy target
/// creature.
pub fn font_of_agonies() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Font of Agonies",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PaidLife, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Blood,
                amount: Value::TriggerEventAmount,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            remove_counter_cost: Some((CounterType::Blood, 4)),
            effect: Effect::Destroy {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rumbling Ruin — {5}{R} 6/6 Elemental. ETB: count the +1/+1 counters on
/// creatures you control; opponents' creatures with power ≤ that number can't
/// block this turn.
pub fn rumbling_ruin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OpponentWeakCreaturesCantBlockByYourCounters)],
        ..body(
            "Rumbling Ruin",
            cost(&[generic(5), r()]),
            6,
            6,
            vec![CreatureType::Elemental],
            vec![],
        )
    }
}

/// Verity Circle — {2}{U} Enchantment. Whenever a creature an opponent controls
/// becomes tapped (not as an attacker), you may draw a card. {4}{U}: Tap target
/// creature without flying.
pub fn verity_circle() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Verity Circle",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer)
                .not_as_attacker()
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::ControlledByOpponent),
                }),
            effect: Effect::MayDo {
                description: "Draw a card.".into(),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), u()]),
            effect: Effect::Tap {
                what: target_filtered(
                    R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Combine Guildmage — {G}{U} 2/2 Merfolk Wizard. {1}{G}, {T}: this turn, each
/// creature you control enters with an additional +1/+1 counter. {1}{U}, {T}:
/// move a +1/+1 counter from target creature you control onto another.
pub fn combine_guildmage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), g()]),
                tap_cost: true,
                effect: Effect::CreaturesEnterWithExtraCounterThisTurn {
                    who: PlayerRef::You,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                tap_cost: true,
                effect: Effect::MoveCounters {
                    from: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::ControlledByYou),
                    },
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByYou),
                    },
                    counter: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..body(
            "Combine Guildmage",
            cost(&[g(), u()]),
            2,
            2,
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Forbidding Spirit — {1}{W}{W} 3/3 Spirit Cleric. ETB: until your next turn,
/// creatures can't attack you or your planeswalkers unless their controller
/// pays {2} for each.
pub fn forbidding_spirit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::TaxAttackersUntilYourNextTurn {
            amount: Value::Const(2),
        })],
        ..body(
            "Forbidding Spirit",
            cost(&[generic(1), w(), w()]),
            3,
            3,
            vec![CreatureType::Spirit, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Galloping Lizrog — {3}{G}{U} 3/3 Frog Lizard with trample. ETB: remove any
/// number of +1/+1 counters from among creatures you control; put twice that
/// many on this creature.
pub fn galloping_lizrog() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DoubleP1P1CountersFromYourCreatures)],
        ..body(
            "Galloping Lizrog",
            cost(&[generic(3), g(), u()]),
            3,
            3,
            vec![CreatureType::Frog, CreatureType::Lizard],
            vec![Keyword::Trample],
        )
    }
}

/// Angel of Grace — {3}{W}{W} 5/4 Angel with flash + flying. ETB: until end of
/// turn, damage that would reduce your life below 1 reduces it to 1 instead.
/// {4}{W}{W}, exile from graveyard: your life total becomes 10.
pub fn angel_of_grace() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CantLoseThisTurn { damage_floor: true })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w(), w()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::SetLifeTotal {
                who: Selector::You,
                amount: Value::Const(10),
            },
            ..Default::default()
        }],
        ..body(
            "Angel of Grace",
            cost(&[generic(3), w(), w()]),
            5,
            4,
            vec![CreatureType::Angel],
            vec![],
        )
    }
}

/// Rhythm of the Wild — {1}{R}{G} Enchantment. Creature spells you control
/// can't be countered; nontoken creatures you control have riot.
pub fn rhythm_of_the_wild() -> CardDefinition {
    use crate::effect::shortcut::riot;
    CardDefinition {
        name: "Rhythm of the Wild",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Creature spells you control can't be countered.",
                effect: StaticEffect::CreatureSpellsCantBeCountered,
            },
            StaticAbility {
                description: "Nontoken creatures you control have riot.",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: R::Creature.and(R::ControlledByYou).and(R::NotToken),
                    ability: Box::new(riot()),
                },
            },
        ],
        ..Default::default()
    }
}

/// Nikya of the Old Ways — {3}{R}{G} 5/5 Centaur Druid. You can't cast
/// noncreature spells. Whenever you tap a land for mana, add one mana of any
/// type that land produced.
pub fn nikya_of_the_old_ways() -> CardDefinition {
    use crate::effect::ExtraManaKind;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "You can't cast noncreature spells.",
                effect: StaticEffect::ControllerCantCastNoncreatureSpells,
            },
            StaticAbility {
                description: "Whenever you tap a land for mana, add one mana of any type that land produced.",
                effect: StaticEffect::ExtraManaOnLandTap {
                    enchanted_only: false,
                    filter: crate::card::SelectionRequirement::Land,
                    extra: ExtraManaKind::Mirror,
                    while_monarch: false,
                },
            },
        ],
        ..body(
            "Nikya of the Old Ways",
            cost(&[generic(3), r(), g()]),
            5,
            5,
            vec![CreatureType::Centaur, CreatureType::Druid],
            vec![],
        )
    }
}

/// Knight of Sorrows — {4}{W} 3/3 Human Knight. Can block an additional
/// creature each combat; afterlife 1.
pub fn knight_of_sorrows() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![afterlife(1)],
        ..body(
            "Knight of Sorrows",
            cost(&[generic(4), w()]),
            3,
            3,
            vec![CreatureType::Human, CreatureType::Knight],
            vec![Keyword::CanBlockAdditional(1)],
        )
    }
}

/// Valor Made Real — {W} Instant. Target creature can block any number of
/// creatures this turn.
pub fn valor_made_real() -> CardDefinition {
    CardDefinition {
        name: "Valor Made Real",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::CanBlockAnyNumber,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Lumbering Battlement — {4}{W} 4/5 Beast with vigilance. ETB: exile any
/// number of other nontoken creatures you control until it leaves; it gets
/// +2/+2 for each card exiled with it.
pub fn lumbering_battlement() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileAnyNumberUntilSourceLeaves {
            filter: R::Creature.and(R::NotToken),
        })],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +2/+2 for each card exiled with it.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::CardsExiledWithSourceCount,
                per_power: 2,
                per_toughness: 2,
            },
        }],
        ..body(
            "Lumbering Battlement",
            cost(&[generic(4), w()]),
            4,
            5,
            vec![CreatureType::Beast],
            vec![Keyword::Vigilance],
        )
    }
}
