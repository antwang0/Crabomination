//! Commander: 2026-10-19 (`01DJHkWk`) — the LOTR Goblin cluster (Azog,
//! Goblin-town, …) and the Marvel Villain decks' second-tier gaps. Tests in
//! `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r};
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

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: chapters,
        ..spell(name, mana, CardType::Enchantment, Effect::Noop)
    }
}

/// CR 701.47a — amass Goblins `n`.
fn amass(n: Value) -> Effect {
    Effect::Amass {
        who: PlayerRef::You,
        count: n,
        extra_type: Some(CreatureType::Goblin),
    }
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn eot_pump(what: Selector, power: i32, toughness: i32) -> Effect {
    Effect::PumpPT {
        what,
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    }
}

fn grant(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword {
        what,
        keyword,
        duration: Duration::EndOfTurn,
    }
}

fn villain_token() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: "Villain".into(),
            power: 2,
            toughness: 1,
            card_types: vec![CardType::Creature],
            colors: vec![Color::Black],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Villain],
                ..Default::default()
            },
            keywords: vec![Keyword::Menace],
            ..Default::default()
        }),
    }
}

fn drain_target_opponent(n: i32) -> Effect {
    Effect::Drain {
        from: target_filtered(R::OpponentPlayer),
        to: Selector::You,
        amount: Value::Const(n),
    }
}

// ── LOTR Goblins ────────────────────────────────────────────────────────────

/// Azog, Moria's Ruin — {2}{B} legendary 1/3 Goblin Soldier. Enters: destroy
/// up to one other target creature; its controller amasses Goblins X (its
/// power); if it was yours, draw a card.
pub fn azog_morias_ruin() -> CardDefinition {
    let it = || Box::new(Selector::Target(0));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            // X (its power) is read before the destroy, while it is a permanent.
            body: Box::new(Effect::WithX {
                x: Value::PowerOf(it()),
                body: Box::new(Effect::Seq(vec![
                    Effect::Destroy {
                        what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    },
                    Effect::Amass {
                        who: PlayerRef::ControllerOf(it()),
                        count: Value::XFromCost,
                        extra_type: Some(CreatureType::Goblin),
                    },
                    Effect::If {
                        cond: Predicate::SamePlayer(PlayerRef::ControllerOf(it()), PlayerRef::You),
                        then: Box::new(Effect::Draw {
                            who: Selector::You,
                            amount: Value::ONE,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            }),
        })],
        ..creature(
            "Azog, Moria's Ruin",
            cost(&[generic(2), b()]),
            vec![CreatureType::Goblin, CreatureType::Soldier],
            1,
            3,
        )
    }
}

/// Fearsome Goblin Pair — {2}{B/R} 1/1 Goblin Soldier; dying, amass Goblins 4.
pub fn fearsome_goblin_pair() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(amass(Value::Const(4)))],
        ..creature(
            "Fearsome Goblin Pair",
            cost(&[generic(2), hybrid(Color::Black, Color::Red)]),
            vec![CreatureType::Goblin, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Gathering of Darkness — {3}{B} sorcery. Return up to one target creature
/// card from your graveyard to your hand; amass Goblins 3.
pub fn gathering_of_darkness() -> CardDefinition {
    spell(
        "Gathering of Darkness",
        cost(&[generic(3), b()]),
        CardType::Sorcery,
        Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                amass(Value::Const(3)),
            ])),
        },
    )
}

/// Great Ugly-Looking Goblin // Clap! Snap! — {5}{B} 4/4 Goblin Soldier; your
/// creatures with +1/+1 counters have menace. Adventure {1}{B}: amass
/// Goblins 2.
pub fn great_ugly_looking_goblin() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a +1/+1 counter on it has menace.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    mine().and(R::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                keyword: Keyword::Menace,
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Clap! Snap!",
            cost: cost(&[generic(1), b()]),
            card_types: vec![CardType::Sorcery],
            effect: amass(Value::Const(2)),
        })),
        ..creature(
            "Great Ugly-Looking Goblin",
            cost(&[generic(5), b()]),
            vec![CreatureType::Goblin, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Goblin-town — Land. Enters tapped; {T}: {B} or {R}. {2}{B}{R}, {T},
/// sacrifice it: two +1/+1 counters on target Goblin or Orc you control;
/// sorcery speed.
pub fn goblin_town() -> CardDefinition {
    let mut d = crate::sets::dual_land_untyped("Goblin-town", Color::Black, Color::Red, vec![]);
    d.static_abilities.push(crate::sets::enters_tapped());
    d.activated_abilities.push(ActivatedAbility {
        tap_cost: true,
        sac_cost: true,
        sorcery_speed: true,
        mana_cost: cost(&[generic(2), b(), r()]),
        effect: Effect::AddCounter {
            what: target_filtered(mine().and(
                R::HasCreatureType(CreatureType::Goblin).or(R::HasCreatureType(CreatureType::Orc)),
            )),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        },
        ..Default::default()
    });
    d
}

/// Down, Down to Goblin-town — {2}{B} Saga. I: target opponent reveals their
/// hand; you choose a nonland card to discard. II: amass Goblins 1. III, IV:
/// drain target opponent for 1.
pub fn down_down_to_goblin_town() -> CardDefinition {
    saga(
        "Down, Down to Goblin-town",
        cost(&[generic(2), b()]),
        vec![
            (
                1,
                Effect::DiscardChosen {
                    from: target_filtered(R::OpponentPlayer),
                    count: Value::ONE,
                    filter: R::Nonland,
                },
            ),
            (2, amass(Value::ONE)),
            (3, drain_target_opponent(1)),
            (4, drain_target_opponent(1)),
        ],
    )
}

/// Ravening Warg — {1}{B} 2/2 deathtouch Wolf. Ferocious — attacking while
/// you control a creature with power 4 or greater, gain 2 life.
pub fn ravening_warg() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::PowerAtLeast(4)),
                }),
            ),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        }],
        ..creature(
            "Ravening Warg",
            cost(&[generic(1), b()]),
            vec![CreatureType::Wolf],
            2,
            2,
        )
    }
}

/// Grim Wanderer — {1}{B} flash 5/3 Goblin Warlock. Tragic Backstory — cast
/// only if a creature died this turn.
pub fn grim_wanderer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        cast_condition: Some(Predicate::CreaturesDiedThisTurnTotalAtLeast {
            at_least: Value::ONE,
        }),
        ..creature(
            "Grim Wanderer",
            cost(&[generic(1), b()]),
            vec![CreatureType::Goblin, CreatureType::Warlock],
            5,
            3,
        )
    }
}

/// Goblin Plate Mail — {1}{B/R} Equipment, +1/+0 and menace, equip {4}.
/// Enters: amass Goblins 1, then attach to the amassed Army.
pub fn goblin_plate_mail() -> CardDefinition {
    CardDefinition {
        name: "Goblin Plate Mail",
        cost: cost(&[generic(1), hybrid(Color::Black, Color::Red)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            keywords: vec![Keyword::Menace],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            amass(Value::ONE),
            Effect::Attach {
                what: Selector::This,
                to: Selector::EachPermanent(mine().and(R::HasCreatureType(CreatureType::Army))),
            },
        ]))],
        ..Default::default()
    }
}

/// Front Porch Sentries — {1}{B} 2/2 Goblin Soldier. Dying, target creature
/// an opponent controls gets -1/-1 until end of turn.
pub fn front_porch_sentries() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(eot_pump(
            target_filtered(R::Creature.and(R::ControlledByOpponent)),
            -1,
            -1,
        ))],
        ..creature(
            "Front Porch Sentries",
            cost(&[generic(1), b()]),
            vec![CreatureType::Goblin, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Knucklebone Witch — {B} 1/1 Goblin Shaman. A Goblin of yours dying lets
/// you put a +1/+1 counter on it.
pub fn knucklebone_witch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Goblin),
                },
            ),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on Knucklebone Witch?".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature(
            "Knucklebone Witch",
            cost(&[b()]),
            vec![CreatureType::Goblin, CreatureType::Shaman],
            1,
            1,
        )
    }
}

/// Warren Weirding — {1}{B} Kindred sorcery — Goblin. Target player
/// sacrifices a creature; if it was a Goblin, they create two 1/1 black Goblin
/// Rogues with haste until end of turn.
pub fn warren_weirding() -> CardDefinition {
    let rogue = TokenDefinition {
        name: "Goblin Rogue".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Rogue],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        ..spell(
            "Warren Weirding",
            cost(&[generic(1), b()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Sacrifice {
                    who: target_filtered(R::Player),
                    count: Value::ONE,
                    filter: R::Creature,
                },
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::SacrificedThisResolution {
                        filter: R::HasCreatureType(CreatureType::Goblin),
                    }),
                    then: Box::new(Effect::Seq(vec![
                        Effect::ClearLastMoved,
                        Effect::CreateToken {
                            who: PlayerRef::Target(0),
                            count: Value::Const(2),
                            definition: Arc::new(rogue),
                        },
                        grant(Selector::LastCreatedTokens, Keyword::Haste),
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Rush the Room — {R} instant. Target creature gets +1/+0 and first strike;
/// a Goblin or Orc also gains haste.
pub fn rush_the_room() -> CardDefinition {
    let it = || Selector::Target(0);
    spell(
        "Rush the Room",
        cost(&[r()]),
        CardType::Instant,
        Effect::Seq(vec![
            eot_pump(target_filtered(R::Creature), 1, 0),
            grant(it(), Keyword::FirstStrike),
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: it(),
                    filter: R::HasCreatureType(CreatureType::Goblin)
                        .or(R::HasCreatureType(CreatureType::Orc)),
                },
                then: Box::new(grant(it(), Keyword::Haste)),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

// ── Marvel Villains, second tier ────────────────────────────────────────────

/// Klaw, Sonic Subjugator — {2}{B} legendary 2/2 Human Rogue Villain. Sonic
/// Attack — enters: target player reveals 1 + (creature cards in your
/// graveyard) cards from hand; you choose one to discard.
pub fn klaw_sonic_subjugator() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::DiscardChosenFromRevealed {
            from: target_filtered(R::Player),
            reveal: Value::Sum(vec![
                Value::ONE,
                Value::count(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::Creature,
                }),
            ]),
        })],
        ..creature(
            "Klaw, Sonic Subjugator",
            cost(&[generic(2), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Rogue,
                CreatureType::Villain,
            ],
            2,
            2,
        )
    }
}

/// Hour of Defeat — {3}{B} instant. Destroy target creature; surveil 1.
pub fn hour_of_defeat() -> CardDefinition {
    spell(
        "Hour of Defeat",
        cost(&[generic(3), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Creature),
            },
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        ]),
    )
}

/// Evil's Thrall — {2}{R} sorcery. Gain control of target creature until end
/// of turn — until the end of your next turn if you control a Villain with
/// greater mana value; untap it; it gains haste.
pub fn evils_thrall() -> CardDefinition {
    let it = || Selector::Target(0);
    let steal = |duration| Effect::GainControl {
        what: it(),
        to: None,
        duration,
    };
    spell(
        "Evil's Thrall",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            // Untapping first is the same board as untapping after the steal.
            Effect::Untap {
                what: target_filtered(R::Creature),
                up_to: None,
            },
            Effect::If {
                cond: Predicate::Not(Box::new(Predicate::ValueAtMost(
                    Value::HighestManaValueAmong(Box::new(Selector::EachPermanent(
                        mine().and(R::HasCreatureType(CreatureType::Villain)),
                    ))),
                    Value::ManaValueOf(Box::new(it())),
                ))),
                then: Box::new(steal(Duration::UntilEndOfYourNextTurn)),
                else_: Box::new(steal(Duration::EndOfTurn)),
            },
            grant(it(), Keyword::Haste),
        ]),
    )
}

/// Trickster's Stratagem — {3}{U} sorcery. The owner of target creature an
/// opponent controls puts it second from the top or on the bottom of their
/// library; then up to one target creature you control connives.
pub fn tricksters_stratagem() -> CardDefinition {
    spell(
        "Trickster's Stratagem",
        cost(&[generic(3), crate::mana::u()]),
        CardType::Sorcery,
        Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                        pos: LibraryPosition::SecondFromTopOrBottom,
                    },
                },
                Effect::Connive {
                    what: Selector::TargetFiltered {
                        slot: 1,
                        filter: mine(),
                    },
                    amount: Value::ONE,
                },
            ])),
        },
    )
}

/// Hire a Crew — {2}{R} instant. A 2/1 menace Villain, then your creatures
/// get +1/+0 until end of turn.
pub fn hire_a_crew() -> CardDefinition {
    spell(
        "Hire a Crew",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            villain_token(),
            eot_pump(Selector::EachPermanent(mine()), 1, 0),
        ]),
    )
}

/// The Coming of Galactus — {2}{B}{B}{G} Saga. I: destroy up to one target
/// nonland permanent. II, III: each opponent loses 2. IV: Galactus, a
/// legendary 16/16 flying, trampling Elder Alien that destroys target land
/// when it attacks.
pub fn the_coming_of_galactus() -> CardDefinition {
    let lose2 = || Effect::LoseLife {
        who: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::Const(2),
    };
    let galactus = TokenDefinition {
        name: "Galactus".into(),
        power: 16,
        toughness: 16,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Alien],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: target_filtered(R::Land),
            },
        }],
        ..Default::default()
    };
    saga(
        "The Coming of Galactus",
        cost(&[generic(2), b(), b(), g()]),
        vec![
            (
                1,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Destroy {
                        what: target_filtered(R::Permanent.and(R::Nonland)),
                    }),
                },
            ),
            (2, lose2()),
            (3, lose2()),
            (
                4,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(galactus),
                },
            ),
        ],
    )
}

/// Infernal Rebirth — {3}{B} instant. Return one or two target creature cards
/// from your graveyard to your hand. Basic landcycling {2}.
pub fn infernal_rebirth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Typecycling(Box::new((
            cost(&[generic(2)]),
            R::IsBasicLand,
        )))],
        ..spell(
            "Infernal Rebirth",
            cost(&[generic(3), b()]),
            CardType::Instant,
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 1,
                filter: R::Creature.and(R::InYourGraveyard),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        )
    }
}

/// HYDRA Disintegrator — {4}{B} Equipment, +3/+3, equip {4}. Enters: a 2/1
/// menace Villain, then attach to it.
pub fn hydra_disintegrator() -> CardDefinition {
    CardDefinition {
        name: "HYDRA Disintegrator",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            villain_token(),
            Effect::Attach {
                what: Selector::This,
                to: Selector::LastCreatedToken,
            },
        ]))],
        ..Default::default()
    }
}

/// Stegron the Dinosaur Man — {4}{R} legendary 5/4 menace Dinosaur Villain.
/// Dinosaur Formula — {1}{R}, discard it: target creature you control gets
/// +3/+1 and becomes a Dinosaur until end of turn.
pub fn stegron_the_dinosaur_man() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                eot_pump(target_filtered(mine()), 3, 1),
                Effect::AddCreatureTypes {
                    what: Selector::Target(0),
                    creature_types: vec![CreatureType::Dinosaur],
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Stegron the Dinosaur Man",
            cost(&[generic(4), r()]),
            vec![CreatureType::Dinosaur, CreatureType::Villain],
            5,
            4,
        )
    }
}
