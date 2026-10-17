//! Commander: 2026-10-17 (`3chree`) closers, sixth batch — one-deck closers
//! from the EDHREC long tail. Tests in `tests/recent_b/cmdr_oct17_v.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    LoyaltyAbility, MayPlayDuration, PlaneswalkerSubtype, Predicate, SelectionRequirement as R,
    SplitCard, SplitHalf, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
    Zone,
};
use crate::effect::shortcut::{blocks, embalm, etb, on_attack, target_filtered, unearth};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

/// Dire Fleet Interloper — {3}{B} 2/2 Human Pirate. Menace. ETB: it explores.
pub fn dire_fleet_interloper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::Explore {
            who: Selector::This,
        })],
        ..creature(
            "Dire Fleet Interloper",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            2,
            2,
        )
    }
}

/// Jungle Wayfinder — {2}{G} 3/3 Elf Warrior. ETB: each player may search
/// their library for a basic land card, put it into their hand, then shuffle.
pub fn jungle_wayfinder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::EachPlayerDoes {
            who: PlayerRef::EachPlayer,
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature(
            "Jungle Wayfinder",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Littjara Glade-Warden — {3}{G} 3/3 Shapeshifter, changeling. {2}{G}, {T},
/// exile a creature card from your graveyard: two +1/+1 counters on target
/// creature. Sorcery speed.
pub fn littjara_glade_warden() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            tap_cost: true,
            sorcery_speed: true,
            exile_other_filter: Some((R::Creature, 1)),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Littjara Glade-Warden",
            cost(&[generic(3), g()]),
            vec![CreatureType::Shapeshifter],
            3,
            3,
        )
    }
}

/// Warehouse Thief — {3}{R} 4/2 Tiefling Rogue. {2}, {T}, sacrifice an
/// artifact or creature (itself included): exile the top card of your
/// library; until the end of your next turn, you may play it.
pub fn warehouse_thief() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_other_filter: Some((R::Artifact.or(R::Creature), 1)),
            sac_other_may_be_source: true,
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(1),
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..creature(
            "Warehouse Thief",
            cost(&[generic(3), r()]),
            vec![CreatureType::Tiefling, CreatureType::Rogue],
            4,
            2,
        )
    }
}

/// Sarkhan Vol — {2}{R}{G} planeswalker, loyalty 4. +1: your creatures +1/+1
/// and haste until EOT. −2: Threaten target creature. −6: five 4/4 red flying
/// Dragons.
pub fn sarkhan_vol() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    let dragon = Arc::new(TokenDefinition {
        name: "Dragon".into(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        ..Default::default()
    });
    CardDefinition {
        name: "Sarkhan Vol",
        cost: cost(&[generic(2), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Sarkhan],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: yours(),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: yours(),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::GainControl {
                        what: target_filtered(R::Creature),
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
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(5),
                    definition: dragon,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Mine Worker — {2} 2/1 artifact creature Assembly-Worker. {T}: gain 1 life,
/// or 3 if you control creatures named Power Plant Worker and Tower Worker.
pub fn mine_worker() -> CardDefinition {
    let named = |n: &str| {
        Predicate::SelectorExists(Selector::EachPermanent(
            R::Creature
                .and(R::ControlledByYou)
                .and(R::HasName(n.to_string())),
        ))
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::If {
                cond: Predicate::All(vec![named("Power Plant Worker"), named("Tower Worker")]),
                then: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                }),
                else_: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Mine Worker",
            cost(&[generic(2)]),
            vec![CreatureType::AssemblyWorker],
            2,
            1,
        )
    }
}

/// Rimewind Taskmage — {1}{U} 1/2 Human Wizard. {1}, {T}: you may tap or untap
/// target permanent. Activate only if you control four or more snow permanents.
pub fn rimewind_taskmage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            condition: Some(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(R::IsSnow.and(R::ControlledByYou)),
                n: Value::Const(4),
            }),
            effect: Effect::MayDo {
                description: "Tap or untap target permanent?".into(),
                body: Box::new(Effect::TapOrUntap {
                    what: target_filtered(R::Permanent),
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Rimewind Taskmage",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            2,
        )
    }
}

/// Coral Trickster — {1}{U} 2/1 Merfolk Rogue. Morph {U}. When it's turned
/// face up, you may tap or untap target permanent.
pub fn coral_trickster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Tap or untap target permanent?".into(),
                body: Box::new(Effect::TapOrUntap {
                    what: target_filtered(R::Permanent),
                }),
            },
        }],
        ..creature(
            "Coral Trickster",
            cost(&[generic(1), u()]),
            vec![CreatureType::Merfolk, CreatureType::Rogue],
            2,
            1,
        )
    }
}

/// Take Possession — {5}{U}{U} Aura, split second. Enchant permanent; you
/// control enchanted permanent.
pub fn take_possession() -> CardDefinition {
    CardDefinition {
        name: "Take Possession",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::SplitSecond],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Permanent),
        },
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::attached_to(Selector::This),
        })],
        ..Default::default()
    }
}

/// Second Wind — {2}{U} Aura, enchant creature. {T}: tap enchanted creature.
/// {T}: untap enchanted creature.
pub fn second_wind() -> CardDefinition {
    CardDefinition {
        name: "Second Wind",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Tap {
                    what: Selector::attached_to(Selector::This),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Untap {
                    what: Selector::attached_to(Selector::This),
                    up_to: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Shield of the Righteous — {W}{U} Equipment. +0/+2 and vigilance; whenever
/// equipped creature blocks a creature, that creature doesn't untap during
/// its controller's next untap step. Equip {2}.
pub fn shield_of_the_righteous() -> CardDefinition {
    CardDefinition {
        name: "Shield of the Righteous",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            toughness: 2,
            keywords: vec![Keyword::Vigilance],
            triggered_abilities: vec![blocks(Effect::SkipNextUntap {
                what: Selector::BlockedAttacker,
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Temmet, Vizier of Naktamun — {W}{U} legendary 2/2 Human Cleric. Beginning
/// of combat on your turn: target creature token you control gets +1/+1 and
/// can't be blocked this turn. Embalm {3}{W}{U}.
pub fn temmet_vizier_of_naktamun() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::IsToken).and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        activated_abilities: vec![embalm(cost(&[generic(3), w(), u()]))],
        ..creature(
            "Temmet, Vizier of Naktamun",
            cost(&[w(), u()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Hypnotic Grifter — {U} 1/2 Human Rogue. {3}: it connives.
pub fn hypnotic_grifter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Connive {
                what: Selector::This,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Hypnotic Grifter",
            cost(&[u()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            1,
            2,
        )
    }
}

/// Calciderm — {2}{W}{W} 5/5 Beast. Shroud. Vanishing 4.
pub fn calciderm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shroud, Keyword::Vanishing(4)],
        ..creature(
            "Calciderm",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Beast],
            5,
            5,
        )
    }
}

/// Maximize Velocity — {R} sorcery: target creature gets +1/+1 and gains haste
/// until end of turn. Jump-start.
pub fn maximize_velocity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::JumpStart],
        ..spell(
            "Maximize Velocity",
            cost(&[r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ONE,
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
}

/// Mu Yanling — {4}{U}{U} planeswalker, loyalty 5. +2: target creature can't be
/// blocked this turn. −3: draw two. −10: tap all opponents' creatures; extra turn.
pub fn mu_yanling() -> CardDefinition {
    CardDefinition {
        name: "Mu Yanling",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Yanling],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::Seq(vec![
                    Effect::Tap {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                    },
                    Effect::TakeExtraTurn {
                        who: PlayerRef::You,
                        count: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Kayla's Command — {1}{W}{W} sorcery, choose two: 2/2 Construct artifact token;
/// +1/+1 counter + double strike on a creature you control (chosen, not
/// targeted); basic Plains to hand; gain 2 and scry 2.
pub fn kaylas_command() -> CardDefinition {
    let construct = Arc::new(TokenDefinition {
        name: "Construct".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        ..Default::default()
    });
    let picked = || Selector::SeparatedPile { chosen: true };
    spell(
        "Kayla's Command",
        cost(&[generic(1), w(), w()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: construct,
                },
                Effect::ChooseOneAmong {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    chooser: PlayerRef::You,
                    chosen: Box::new(Effect::Seq(vec![
                        Effect::AddCounter {
                            what: picked(),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                        Effect::GrantKeyword {
                            what: picked(),
                            keyword: Keyword::DoubleStrike,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                    other: Box::new(Effect::Noop),
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasLandType(LandType::Plains).and(R::HasSupertype(Supertype::Basic)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Seq(vec![
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(2),
                    },
                    Effect::Scry {
                        who: PlayerRef::You,
                        amount: Value::Const(2),
                    },
                ]),
            ],
            min: 2,
            max: 2,
            allow_repeats: false,
        },
    )
}

/// Gideon, Martial Paragon — {4}{W} loyalty 5. +2: untap your creatures, they get
/// +1/+1. 0: 5/5 indestructible Human Soldier, prevent damage to him. −10: your
/// creatures +2/+2; tap all opponents' creatures.
pub fn gideon_martial_paragon() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Gideon, Martial Paragon",
        cost: cost(&[generic(4), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Gideon],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    Effect::Untap {
                        what: yours(),
                        up_to: None,
                    },
                    Effect::PumpPT {
                        what: yours(),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(5),
                        toughness: Value::Const(5),
                        creature_types: vec![CreatureType::Human, CreatureType::Soldier],
                        keywords: vec![Keyword::Indestructible],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::PreventAllDamageThisTurn {
                        target: Selector::This,
                        redirect_to: None,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: yours(),
                        power: Value::Const(2),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Tap {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Terror Ballista — {7} 5/3 artifact Construct, menace. Attacks: you may
/// sacrifice another creature; when you do, destroy target creature an opponent
/// controls. Unearth {3}{B}{B}.
pub fn terror_ballista() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::MaySacrifice {
            description:
                "Sacrifice another creature to destroy target creature an opponent controls?".into(),
            filter: R::Creature.and(R::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::Reflexive {
                body: Box::new(Effect::Destroy {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                }),
            }),
            else_: None,
        })],
        activated_abilities: vec![unearth(cost(&[generic(3), b(), b()]))],
        ..creature(
            "Terror Ballista",
            cost(&[generic(7)]),
            vec![CreatureType::Construct],
            5,
            3,
        )
    }
}

/// Ipnu Rivulet — Desert. {T}: {C}. {T}, pay 1 life: {U}. {1}{U}, {T},
/// sacrifice a Desert: target player mills four.
pub fn ipnu_rivulet() -> CardDefinition {
    CardDefinition {
        name: "Ipnu Rivulet",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Desert],
            ..Default::default()
        },
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                life_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Blue]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                tap_cost: true,
                sac_other_filter: Some((R::HasLandType(LandType::Desert), 1)),
                sac_other_may_be_source: true,
                effect: Effect::Mill {
                    who: target_filtered(R::Player),
                    amount: Value::Const(4),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Moonring Island — Island, enters tapped. {U}, {T}: look at the top card of
/// target player's library; only with two or more blue permanents.
pub fn moonring_island() -> CardDefinition {
    CardDefinition {
        name: "Moonring Island",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Island],
            ..Default::default()
        },
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Blue),
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                tap_cost: true,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::HasColor(Color::Blue).and(R::ControlledByYou)),
                    n: Value::Const(2),
                }),
                // The "lose 0 life" step declares slot 0 as a player target.
                effect: Effect::Seq(vec![
                    Effect::LoseLife { who: target_filtered(R::Player), amount: Value::Const(0) },
                    Effect::LookAtTop { who: PlayerRef::Target(0), amount: Value::ONE },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Appeal // Authority — Appeal {G} sorcery: target creature gains trample and
/// +X/+X (X = creatures you control). Authority {1}{W} aftermath: tap up to two
/// target opponents' creatures; your creatures gain vigilance until end of turn.
pub fn appeal_authority() -> CardDefinition {
    let theirs = |slot: u8| Selector::TargetFiltered {
        slot,
        filter: R::Creature.and(R::ControlledByOpponent),
    };
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(1), w()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::Tap { what: theirs(0) },
                        Effect::Tap { what: theirs(1) },
                        Effect::GrantKeyword {
                            what: yours(),
                            keyword: Keyword::Vigilance,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
            },
            fuse: false,
            aftermath: true,
        })),
        ..spell(
            "Appeal // Authority",
            cost(&[g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::CountOf(Box::new(yours())),
                    toughness: Value::CountOf(Box::new(yours())),
                    duration: Duration::EndOfTurn,
                },
            ]),
        )
    }
}

/// Dig Up the Body — {2}{B} instant, casualty 1. Mill two, then you may return a
/// creature card from your graveyard to your hand.
pub fn dig_up_the_body() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Casualty(1)],
        ..spell(
            "Dig Up the Body",
            cost(&[generic(2), b()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::MoveChosen {
                    from: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: R::Creature,
                    },
                    filter: None,
                    count: Value::ONE,
                    up_to: true,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ]),
        )
    }
}

/// Temur Charger — {1}{G} 3/1 Horse. Morph—Reveal a green card in your hand.
/// Turned face up: target creature gains trample until end of turn.
pub fn temur_charger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MorphCost(Box::new(WardCost::RevealMatching(
            Box::new(R::HasColor(Color::Green)),
        )))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Temur Charger",
            cost(&[generic(1), g()]),
            vec![CreatureType::Horse],
            3,
            1,
        )
    }
}
