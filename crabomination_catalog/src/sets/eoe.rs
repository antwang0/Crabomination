//! Edge of Eternities — Exhaust (CR 702.177). "Exhaust — [Cost]: [Effect]"
//! means "[Cost]: [Effect]. Activate only once" (per game). Modeled via the
//! `ActivatedAbility.exhaust` flag + `CardInstance.exhausted_abilities`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement, StaticAbility, Subtypes,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_dies, target, target_filtered, warp};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, Value};
use crate::mana::{b, cost, g, generic, r, u, w, x};
use crabomination_base::tokens::lander_token;

/// Shared exhaust ability: "Exhaust — [cost]: Put N +1/+1 counters on this."
fn exhaust_self_counters(mana: crate::mana::ManaCost, n: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        exhaust: true,
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(n),
        },
        ..Default::default()
    }
}

/// Camera Launcher — {3} Artifact Creature — Construct 2/2. "Exhaust — {3}:
/// Put a +1/+1 counter on this creature. Create a 1/1 colorless Thopter
/// artifact creature token with flying."
pub fn camera_launcher() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Camera Launcher",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: thopter },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hazard of the Dunes — {3}{G} 4/4 Wurm. Trample, reach. "Exhaust — {6}{G}:
/// Put three +1/+1 counters on this creature."
pub fn hazard_of_the_dunes() -> CardDefinition {
    CardDefinition {
        name: "Hazard of the Dunes",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Reach],
        activated_abilities: vec![exhaust_self_counters(cost(&[generic(6), g()]), 3)],
        ..Default::default()
    }
}

/// Prowcatcher Specialist — {1}{R} 2/1 Goblin Warrior. Haste. "Exhaust —
/// {3}{R}: Put two +1/+1 counters on this creature."
pub fn prowcatcher_specialist() -> CardDefinition {
    CardDefinition {
        name: "Prowcatcher Specialist",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![exhaust_self_counters(cost(&[generic(3), r()]), 2)],
        ..Default::default()
    }
}

/// Greenbelt Guardian — {1}{G} 2/2 Elf Ranger. "{G}: Target creature gains
/// trample until end of turn." plus "Exhaust — {3}{G}: Put three +1/+1
/// counters on this creature."
pub fn greenbelt_guardian() -> CardDefinition {
    CardDefinition {
        name: "Greenbelt Guardian",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Ranger],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::GrantKeyword {
                    what: target(),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            exhaust_self_counters(cost(&[generic(3), g()]), 3),
        ],
        ..Default::default()
    }
}

/// Pacesetter Paragon — {2}{R} 2/3 Human Pilot. "Exhaust — {2}{R}: Put a
/// +1/+1 counter on this creature. It gains double strike until end of turn."
pub fn pacesetter_paragon() -> CardDefinition {
    CardDefinition {
        name: "Pacesetter Paragon",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Keen Buccaneer — {2}{U} 2/3 Octopus Pirate. Vigilance. "Exhaust — {1}{U}:
/// Draw a card, then discard a card. Put a +1/+1 counter on this creature."
pub fn keen_buccaneer() -> CardDefinition {
    CardDefinition {
        name: "Keen Buccaneer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Octopus, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skystreak Engineer — {1}{U} 1/3 Human Pilot. Flying. "Exhaust — {4}{U}:
/// Put two +1/+1 counters on this creature."
pub fn skystreak_engineer() -> CardDefinition {
    CardDefinition {
        name: "Skystreak Engineer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![exhaust_self_counters(cost(&[generic(4), u()]), 2)],
        ..Default::default()
    }
}

/// Mai, Jaded Edge — {1}{R} 1/3 Legendary Human Noble. Prowess. "Exhaust —
/// {3}: Put a double strike counter on Mai."
pub fn mai_jaded_edge() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Mai, Jaded Edge",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Prowess],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            exhaust: true,
            effect: Effect::AddKeywordCounter {
                what: Selector::This,
                keyword: Keyword::DoubleStrike,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stampeding Scurryfoot — {G} 1/1 Mouse. "Exhaust — {3}{G}: Put a +1/+1
/// counter on this creature. Create a 3/3 green Elephant creature token."
pub fn stampeding_scurryfoot() -> CardDefinition {
    use crabomination_base::mana::Color;
    let elephant = TokenDefinition {
        name: "Elephant".into(),
        power: 3,
        toughness: 3,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Stampeding Scurryfoot",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mouse], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: elephant },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mindspring Merfolk — {U} 1/1 Merfolk Wizard. "Exhaust — {X}{U}{U}, {T}:
/// Draw X cards. Put a +1/+1 counter on each Merfolk creature you control."
pub fn mindspring_merfolk() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::mana::x;
    CardDefinition {
        name: "Mindspring Merfolk",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[x(), u(), u()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::XFromCost },
                Effect::AddCounter {
                    what: Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Merfolk)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Edge of Eternities — Warp (cast for a cheaper cost, exile at the next end
// step, recast from exile), Void (a nonland permanent left the battlefield or a
// spell was warped this turn), and Lander tokens (sac for a basic land).
// ─────────────────────────────────────────────────────────────────────────────

/// Bygone Colossus — {9} Artifact Creature — Robot Giant 9/9. Warp {3}.
pub fn bygone_colossus() -> CardDefinition {
    CardDefinition {
        name: "Bygone Colossus",
        cost: cost(&[generic(9)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Giant],
            ..Default::default()
        },
        power: 9,
        toughness: 9,
        alternative_cost: Some(warp(cost(&[generic(3)]))),
        ..Default::default()
    }
}

/// Codecracker Hound — {2}{U} Creature — Dog 2/1. ETB: look at the top two
/// cards, put one in hand and the other in your graveyard. Warp {2}{U}.
pub fn codecracker_hound() -> CardDefinition {
    CardDefinition {
        name: "Codecracker Hound",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(2),
            rest_to_graveyard: true,
            pick_filter: None,
            take: Some(Value::Const(1)),
            to_battlefield: false,
        })],
        alternative_cost: Some(warp(cost(&[generic(2), u()]))),
        ..Default::default()
    }
}

/// Nova Hellkite — {3}{R}{R} Creature — Dragon 4/5, flying, haste. ETB: deal 1
/// damage to target creature an opponent controls. Warp {2}{R}.
pub fn nova_hellkite() -> CardDefinition {
    CardDefinition {
        name: "Nova Hellkite",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![etb(Effect::DealDamage {
            amount: Value::Const(1),
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        alternative_cost: Some(warp(cost(&[generic(2), r()]))),
        ..Default::default()
    }
}

/// Drix Fatemaker — {3}{G} Creature — Drix Wizard 3/2. ETB: put a +1/+1 counter
/// on target creature. Each creature you control with a +1/+1 counter on it has
/// trample. Warp {1}{G}.
pub fn drix_fatemaker() -> CardDefinition {
    CardDefinition {
        name: "Drix Fatemaker",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drix, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target(),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a +1/+1 counter on it has trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                keyword: Keyword::Trample,
            },
        }],
        alternative_cost: Some(warp(cost(&[generic(1), g()]))),
        ..Default::default()
    }
}

/// Broodguard Elite — {X}{G}{G} Creature — Insect Knight, enters with X +1/+1
/// counters. When it leaves the battlefield, put its counters on target creature
/// you control. Warp {X}{G}.
pub fn broodguard_elite() -> CardDefinition {
    CardDefinition {
        name: "Broodguard Elite",
        cost: cost(&[x(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Knight],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::MoveAllCounters {
                from: Selector::This,
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            },
        }],
        alternative_cost: Some(warp(cost(&[x(), g()]))),
        ..Default::default()
    }
}

/// All-Fates Stalker — {3}{W} Creature — Drix Assassin 2/3. ETB: exile target
/// creature until this leaves the battlefield. Warp {1}{W}. (The "up to one
/// non-Assassin" rider is approximated as a plain target creature.)
pub fn all_fates_stalker() -> CardDefinition {
    CardDefinition {
        name: "All-Fates Stalker",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drix, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(SelectionRequirement::Creature),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        alternative_cost: Some(warp(cost(&[generic(1), w()]))),
        ..Default::default()
    }
}

/// Eusocial Engineering — {3}{G}{G} Enchantment. Landfall — whenever a land you
/// control enters, create a 2/2 colorless Robot artifact creature token. Warp
/// {1}{G}.
pub fn eusocial_engineering() -> CardDefinition {
    let robot = TokenDefinition {
        name: "Robot".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Eusocial Engineering",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Land,
                }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: robot,
            },
        }],
        alternative_cost: Some(warp(cost(&[generic(1), g()]))),
        ..Default::default()
    }
}

/// Decode Transmissions — {2}{B} Sorcery. You draw two cards and lose 2 life.
/// Void — if a nonland permanent left the battlefield this turn or a spell was
/// warped this turn, instead you draw two cards and each opponent loses 2 life.
pub fn decode_transmissions() -> CardDefinition {
    use crate::effect::shortcut::{draw, lose_life};
    CardDefinition {
        name: "Decode Transmissions",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::VoidActive { who: PlayerRef::You },
            then: Box::new(Effect::Seq(vec![
                draw(2),
                lose_life(2, Selector::Player(PlayerRef::EachOpponent)),
            ])),
            else_: Box::new(Effect::Seq(vec![draw(2), lose_life(2, Selector::You)])),
        },
        ..Default::default()
    }
}

/// Elegy Acolyte — {2}{B}{B} Creature — Human Cleric 4/4, lifelink. Whenever a
/// creature you control deals combat damage to a player, draw a card and lose 1
/// life. Void — at the beginning of your end step, if Void is active, draw a card
/// and lose 1 life. (The "one or more" batch is modeled per-creature.)
pub fn elegy_acolyte() -> CardDefinition {
    use crate::effect::shortcut::{draw, lose_life};
    let draw_lose = || Effect::Seq(vec![draw(1), lose_life(1, Selector::You)]);
    CardDefinition {
        name: "Elegy Acolyte",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
                effect: draw_lose(),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::VoidActive { who: PlayerRef::You },
                    then: Box::new(draw_lose()),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Biomechan Engineer — {G}{U} Creature — Insect Artificer 2/2. ETB: create a
/// Lander token.
pub fn biomechan_engineer() -> CardDefinition {
    CardDefinition {
        name: "Biomechan Engineer",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Biotech Specialist — {R}{G} Creature — Insect Scientist 1/3. ETB: create a
/// Lander token.
pub fn biotech_specialist() -> CardDefinition {
    CardDefinition {
        name: "Biotech Specialist",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Scientist],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Beamsaw Prospector — {1}{B} Creature — Human Artificer 2/1. When it dies,
/// create a Lander token.
pub fn beamsaw_prospector() -> CardDefinition {
    CardDefinition {
        name: "Beamsaw Prospector",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Bioengineered Future — {1}{G}{G} Enchantment. ETB: create a Lander token.
pub fn bioengineered_future() -> CardDefinition {
    CardDefinition {
        name: "Bioengineered Future",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Blooming Stinger — {1}{G} Creature — Plant Scorpion 2/2, deathtouch. ETB:
/// another target creature you control gains deathtouch until end of turn.
pub fn blooming_stinger() -> CardDefinition {
    CardDefinition {
        name: "Blooming Stinger",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Scorpion],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            keyword: Keyword::Deathtouch,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Biosynthic Burst — {1}{G} Instant. Put a +1/+1 counter on target creature you
/// control; it gains reach, trample, and indestructible until end of turn, then
/// untap it.
pub fn biosynthic_burst() -> CardDefinition {
    CardDefinition {
        name: "Biosynthic Burst",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GrantKeyword { what: target(), keyword: Keyword::Reach, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: target(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            Effect::GrantKeyword {
                what: target(),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: target(), up_to: None },
        ]),
        ..Default::default()
    }
}

/// Cosmic Epiphany — {4}{U}{U} Sorcery. Draw cards equal to the number of instant
/// and sorcery cards in your graveyard.
pub fn cosmic_epiphany() -> CardDefinition {
    CardDefinition {
        name: "Cosmic Epiphany",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::CardsInGraveyardMatching {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            },
        },
        ..Default::default()
    }
}

/// Beyond the Quiet — {3}{W}{W} Sorcery. Exile all creatures. (Spacecraft aren't
/// modeled as a distinct type; the creature half is faithful.)
pub fn beyond_the_quiet() -> CardDefinition {
    CardDefinition {
        name: "Beyond the Quiet",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::Exile { what: Selector::TriggerSource }),
        },
        ..Default::default()
    }
}

/// Singularity Rupture — {3}{U}{B}{B} Sorcery. Destroy all creatures, then each
/// opponent mills half their library, rounded down. ("Any number of target
/// players" is approximated as each opponent.)
pub fn singularity_rupture() -> CardDefinition {
    CardDefinition {
        name: "Singularity Rupture",
        cost: cost(&[generic(3), u(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature),
                body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
            },
            Effect::MillHalf {
                who: Selector::Player(PlayerRef::EachOpponent),
                rounded_up: false,
            },
        ]),
        ..Default::default()
    }
}

/// Voyager Quickwelder — {2}{W} Artifact Creature — Robot Artificer 2/4. Artifact
/// spells you cast cost {1} less to cast.
pub fn voyager_quickwelder() -> CardDefinition {
    CardDefinition {
        name: "Voyager Quickwelder",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Artifact spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: SelectionRequirement::Artifact, amount: 1 },
        }],
        ..Default::default()
    }
}

/// Memory Guardian — {4}{U} Artifact Creature — Robot Artificer 3/4, flying.
/// Affinity for artifacts.
pub fn memory_guardian() -> CardDefinition {
    CardDefinition {
        name: "Memory Guardian",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        affinity_filter: Some(SelectionRequirement::Artifact),
        ..Default::default()
    }
}

/// Perimeter Patrol — {2}{G} Creature — Human Soldier 3/3. Whenever an artifact
/// you control enters, this creature gets +1/+0 until end of turn.
pub fn perimeter_patrol() -> CardDefinition {
    CardDefinition {
        name: "Perimeter Patrol",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Hulldrifter — {3}{U}{U} Artifact — Vehicle 3/2, flying. ETB: draw two cards.
/// Crew 3.
pub fn hulldrifter() -> CardDefinition {
    CardDefinition {
        name: "Hulldrifter",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Crew(3)],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(2) })],
        ..Default::default()
    }
}

/// Tidal Terror — {4}{U}{U} Creature — Octopus 5/6. Islandcycling {2}. (The
/// tap-two-to-be-unblockable attack rider is omitted.)
pub fn tidal_terror() -> CardDefinition {
    CardDefinition {
        name: "Tidal Terror",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Octopus], ..Default::default() },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Island)],
        ..Default::default()
    }
}

// ── EOE batch 2 — more Warp / Void / Lander ──────────────────────────────────

/// Germinating Wurm — {4}{G} Creature — Plant Wurm 5/5. ETB: gain 2 life. Warp
/// {1}{G}.
pub fn germinating_wurm() -> CardDefinition {
    use crate::effect::shortcut::gain_life;
    CardDefinition {
        name: "Germinating Wurm",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Wurm],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![etb(gain_life(2))],
        alternative_cost: Some(warp(cost(&[generic(1), g()]))),
        ..Default::default()
    }
}

/// Knight Luminary — {3}{W} Creature — Human Knight 3/2. ETB: make a 1/1 white
/// Human Soldier. Warp {1}{W}.
pub fn knight_luminary() -> CardDefinition {
    use crate::mana::Color;
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Knight Luminary",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: soldier,
        })],
        alternative_cost: Some(warp(cost(&[generic(1), w()]))),
        ..Default::default()
    }
}

/// Memorial Team Leader — {3}{R} Creature — Kavu Soldier 4/3. During your turn,
/// other creatures you control get +1/+0. Warp {1}{R}.
pub fn memorial_team_leader() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Memorial Team Leader",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "During your turn, other creatures you control get +1/+0.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::IsTurnOf(PlayerRef::You),
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        alternative_cost: Some(warp(cost(&[generic(1), r()]))),
        ..Default::default()
    }
}

/// Dauntless Scrapbot — {3} Artifact Creature — Robot 3/1. ETB: exile each
/// opponent's graveyard, then create a Lander token.
pub fn dauntless_scrapbot() -> CardDefinition {
    CardDefinition {
        name: "Dauntless Scrapbot",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExileAllGraveyards { filter: None, opponents_only: true },
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: lander_token() },
        ]))],
        ..Default::default()
    }
}

/// Edge Rover — {G} Artifact Creature — Robot Scout 2/2, reach. When it dies,
/// each player creates a Lander token.
pub fn edge_rover() -> CardDefinition {
    CardDefinition {
        name: "Edge Rover",
        cost: cost(&[g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::EachPlayer,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Galactic Wayfarer — {2}{G} Creature — Human Scout 3/3. ETB: create a Lander.
pub fn galactic_wayfarer() -> CardDefinition {
    CardDefinition {
        name: "Galactic Wayfarer",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Glacier Godmaw — {5}{G}{G} Creature — Leviathan 6/6, trample. ETB: create a
/// Lander.
pub fn glacier_godmaw() -> CardDefinition {
    CardDefinition {
        name: "Glacier Godmaw",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Leviathan], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Kav Landseeker — {3}{R} Creature — Kavu Soldier 4/3, menace. ETB: create a
/// Lander. (The "sacrifice it next end step" rider is dropped.)
pub fn kav_landseeker() -> CardDefinition {
    CardDefinition {
        name: "Kav Landseeker",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Emergency Eject — {2}{W} Instant. Destroy target nonland permanent; its
/// controller creates a Lander token.
pub fn emergency_eject() -> CardDefinition {
    CardDefinition {
        name: "Emergency Eject",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: lander_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Kavaron Skywarden — {4}{R} Creature — Kavu Soldier 4/5, reach. Void — at the
/// beginning of your end step, if Void is active, put a +1/+1 counter on this.
pub fn kavaron_skywarden() -> CardDefinition {
    CardDefinition {
        name: "Kavaron Skywarden",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::VoidActive { who: PlayerRef::You },
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Mechanozoa — {4}{U}{U} Artifact Creature — Robot Jellyfish 5/5. ETB: tap
/// target artifact or creature an opponent controls and put a stun counter on it.
pub fn mechanozoa() -> CardDefinition {
    CardDefinition {
        name: "Mechanozoa",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Jellyfish],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Creature)
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Exalted Sunborn — {3}{W}{W} Creature — Angel Wizard 4/5, flying, lifelink. If
/// one or more tokens would be created under your control, twice that many are
/// created instead. Warp {1}{W}.
pub fn exalted_sunborn() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Exalted Sunborn",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "If one or more tokens would be created under your control, twice that many are created instead.",
            effect: StaticEffect::DoubleTokens,
        }],
        alternative_cost: Some(warp(cost(&[generic(1), w()]))),
        ..Default::default()
    }
}
