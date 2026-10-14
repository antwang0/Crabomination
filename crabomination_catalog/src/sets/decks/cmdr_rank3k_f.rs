//! Commander: one- and two-card gaps of EDHREC's commanders ranked 2501-3500
//! (the 2026-10-14 scan), sixth file. Tests in `tests/recent_b/cmdr_rank3k_f.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R,
    SoulbondBonus, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color, ManaCost};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn counters(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

fn pump_eot(what: Selector, p: Value, t: Value) -> Effect {
    Effect::PumpPT { what, power: p, toughness: t, duration: Duration::EndOfTurn }
}

fn heroes() -> R {
    R::Creature.and(R::HasCreatureType(CreatureType::Hero))
}

/// Imperious Mindbreaker — {1}{U}{U} 1/4 Human Wizard, soulbond. While paired,
/// each of the pair has "Whenever this creature attacks, each opponent mills
/// cards equal to its toughness."
pub fn imperious_mindbreaker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Soulbond],
        soulbond_bonus: Some(SoulbondBonus {
            triggered_abilities: vec![on_attack(Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ToughnessOf(Box::new(Selector::This)),
            })],
            ..Default::default()
        }),
        ..creature("Imperious Mindbreaker", cost(&[generic(1), u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 4)
    }
}

/// Mizzix, Replica Rider — {4}{R} legendary 4/5 flyer. Whenever you cast a
/// spell from anywhere other than your hand, you may pay {1}{U/R}; if you do,
/// copy it (new targets allowed); a permanent copy gains haste and is
/// sacrificed at your end step.
pub fn mizzix_replica_rider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(R::SpellNotCastFromHand)),
            effect: Effect::MayPay {
                description: "Pay {1}{U/R} to copy that spell?".into(),
                mana_cost: cost(&[generic(1), hybrid(Color::Blue, Color::Red)]),
                body: Box::new(Effect::CopySpellWithRiders {
                    what: Selector::TriggerSource,
                    count: Value::ONE,
                    grant_haste: true,
                    sacrifice_eot: true,
                    may_choose_new_targets: true,
                }),
                else_: None,
            },
        }],
        ..legend("Mizzix, Replica Rider", cost(&[generic(4), r()]), vec![CreatureType::Goblin, CreatureType::Wizard], 4, 5)
    }
}

/// Dual Strike — {R}{R} instant: when you next cast an instant or sorcery spell
/// with mana value 4 or less this turn, copy it (new targets allowed).
/// Foretell {R}.
pub fn dual_strike() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[r()])),
        ..spell(
            "Dual Strike",
            cost(&[r(), r()]),
            CardType::Instant,
            Effect::OnYourNextSpellMatchingThisTurn {
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::ManaValueAtMost(4)),
                body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
            },
        )
    }
}

/// Embermaw Hellion — {3}{R}{R} 4/5 trample Hellion. Another red source you
/// control deals that much damage plus 1 to a permanent or player.
pub fn embermaw_hellion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "If another red source you control would deal damage to a permanent or player, it deals that much damage plus 1 instead.",
            effect: StaticEffect::YourColorSourcesDealExtraDamage { color: Color::Red, amount: 1 },
        }],
        ..creature("Embermaw Hellion", cost(&[generic(3), r(), r()]), vec![CreatureType::Hellion], 4, 5)
    }
}

/// Ajani's Anguish — {X}{R} enchantment. ETB: X damage to any target.
/// Creatures you control have trample.
pub fn ajanis_anguish() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DealDamage { to: target_filtered(R::Any), amount: Value::XFromCost })],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have trample.",
            effect: StaticEffect::GrantKeyword { applies_to: yours(R::Creature), keyword: Keyword::Trample },
        }],
        ..spell("Ajani's Anguish", cost(&[x(), r()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Iron Man, Tony Stark — {3}{R}{R} legendary 3/3 flying artifact creature.
/// Attacking creatures you control get +1/+0. Whenever you cast a red spell, a
/// 2/1 colorless flying Robot Hero artifact creature.
pub fn iron_man_tony_stark() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT { applies_to: yours(R::Creature.and(R::IsAttacking)), power: 1, toughness: 0 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(R::HasColor(Color::Red))),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Robot Hero".into(),
                    power: 2,
                    toughness: 1,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    keywords: vec![Keyword::Flying],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Robot, CreatureType::Hero], ..Default::default() },
                    ..Default::default()
                }),
            },
        }],
        ..legend("Iron Man, Tony Stark", cost(&[generic(3), r(), r()]), vec![CreatureType::Human, CreatureType::Hero], 3, 3)
    }
}

/// Hawkeye, Trick Shot — {3}{R} legendary 3/4 reach, first strike. Whenever he
/// or another Hero you control enters, it deals damage equal to the Heroes you
/// control to any target.
pub fn hawkeye_trick_shot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(trigger_is(heroes())),
            effect: Effect::DealDamageFrom {
                source: Selector::TriggerSource,
                to: target_filtered(R::Any),
                amount: Value::count(yours(heroes())),
            },
        }],
        ..legend(
            "Hawkeye, Trick Shot",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Archer, CreatureType::Hero],
            3,
            4,
        )
    }
}

/// Doomskar Warrior — {2}{G}{G} 4/3 trample, backup 1. Combat damage to a player
/// or battle: look at that many cards; you may reveal a creature or land card
/// and put it into your hand; the rest on the bottom in a random order.
pub fn doomskar_warrior() -> CardDefinition {
    let dig = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::TriggerEventAmount,
            pick_filter: Some(R::Creature.or(R::Land)),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })),
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![crate::effect::shortcut::backup_with(1, vec![Keyword::Trample], vec![dig.clone()]), dig],
        ..creature("Doomskar Warrior", cost(&[generic(2), g(), g()]), vec![CreatureType::Human, CreatureType::Warrior], 4, 3)
    }
}

/// Hamlet Captain — {1}{G} 2/2 Human Warrior. Attacks or blocks: other Humans
/// you control get +1/+1 until end of turn.
pub fn hamlet_captain() -> CardDefinition {
    let rally = || pump_eot(yours(R::Creature.and(R::HasCreatureType(CreatureType::Human)).and(R::OtherThanSource)), Value::ONE, Value::ONE);
    CardDefinition {
        triggered_abilities: vec![on_attack(rally()), crate::effect::shortcut::blocks(rally())],
        ..creature("Hamlet Captain", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 2)
    }
}

/// Tigra, Feline Fury — {1}{G} legendary 2/1 flash, trample; whenever you gain
/// life, a +1/+1 counter on her.
pub fn tigra_feline_fury() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: counters(Selector::This, Value::ONE),
        }],
        ..legend("Tigra, Feline Fury", cost(&[generic(1), g()]), vec![CreatureType::Cat, CreatureType::Human, CreatureType::Hero], 2, 1)
    }
}

/// Staff of the Wild Magus — {3} artifact: whenever you cast a green spell or a
/// Forest you control enters, you gain 1 life.
pub fn staff_of_the_wild_magus() -> CardDefinition {
    let gain = || Effect::GainLife { who: Selector::You, amount: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(R::HasColor(Color::Green))),
                effect: gain(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(trigger_is(R::HasLandType(LandType::Forest))),
                effect: gain(),
            },
        ],
        ..spell("Staff of the Wild Magus", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Donatello, Way with Machines — {2}{U} legendary 1/2 flyer; whenever an
/// artifact you control enters, a +1/+1 counter on him.
pub fn donatello_way_with_machines() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(trigger_is(R::Artifact)),
            effect: counters(Selector::This, Value::ONE),
        }],
        ..legend(
            "Donatello, Way with Machines",
            cost(&[generic(2), u()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            1,
            2,
        )
    }
}

/// Krang, Master Mind — {6}{U}{U} legendary 1/4 artifact creature, affinity for
/// artifacts. ETB: with fewer than four cards in hand, draw the difference.
/// +1/+0 for each other artifact you control.
pub fn krang_master_mind() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        affinity_filter: Some(R::Artifact),
        static_abilities: vec![StaticAbility {
            description: "Krang gets +1/+0 for each other artifact you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::Artifact.and(R::OtherThanSource),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Diff(Box::new(Value::Const(4)), Box::new(Value::HandSizeOf(PlayerRef::You))),
        })],
        ..legend("Krang, Master Mind", cost(&[generic(6), u(), u()]), vec![CreatureType::Utrom, CreatureType::Warrior], 1, 4)
    }
}

/// Iroh, Firebending Instructor — {2}{R} legendary 2/2. Attacks: attacking
/// creatures get +1/+1 until end of turn.
pub fn iroh_firebending_instructor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(pump_eot(
            Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
            Value::ONE,
            Value::ONE,
        ))],
        ..legend(
            "Iroh, Firebending Instructor",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Ally],
            2,
            2,
        )
    }
}

/// Reckless Blaze — {3}{R}{R} sorcery — Lesson: 5 damage to each creature;
/// whenever a creature you control dealt damage this way dies this turn, add
/// {R}. (Residual: the {R} rider watches every creature of yours that dies
/// this turn, not only the ones it damaged.)
pub fn reckless_blaze() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { spell_subtypes: vec![crate::card::SpellSubtype::Lesson], ..Default::default() },
        ..spell(
            "Reckless Blaze",
            cost(&[generic(3), r(), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::WheneverCreatureDiesThisTurn {
                    filter: R::ControlledByYou,
                    body: Box::new(Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red]) }),
                },
                Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(5) },
            ]),
        )
    }
}

/// Casey Jones, Asphalt Hooligan — {2}{R} legendary 2/2 double strike. {4}:
/// double his power until end of turn; any player may activate this ability.
pub fn casey_jones_asphalt_hooligan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            any_player: true,
            effect: pump_eot(Selector::This, Value::PowerOf(Box::new(Selector::This)), Value::ZERO),
            ..Default::default()
        }],
        ..legend(
            "Casey Jones, Asphalt Hooligan",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Berserker],
            2,
            2,
        )
    }
}

/// Improvised Arsenal — {1}{R} Equipment: +1/+0 for each artifact you control.
/// {4}{R}: a token copy of it. Equip {R}.
pub fn improvised_arsenal() -> CardDefinition {
    CardDefinition {
        name: "Improvised Arsenal",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[r()]))],
        equipped_bonus: Some(EquipBonus::default()),
        static_abilities: vec![StaticAbility {
            description: "Equipped creature gets +1/+0 for each artifact you control.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::count(yours(R::Artifact)),
                toughness: Value::ZERO,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), r()]),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
                no_mana_cost: false,
                enters_with_counters: None,
                remove_keywords: vec![],
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Black Widow, Daring Operative — {4}{B} legendary 3/4 menace. ETB: mill
/// three. Combat damage to a player: each opponent loses life equal to the
/// creature cards in your graveyard.
pub fn black_widow_daring_operative() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            etb(Effect::Mill { who: Selector::You, amount: Value::Const(3) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
                },
            },
        ],
        ..legend(
            "Black Widow, Daring Operative",
            cost(&[generic(4), b()]),
            vec![CreatureType::Human, CreatureType::Spy, CreatureType::Hero],
            3,
            4,
        )
    }
}

/// Koth of the Homestead — {2}{W} legendary 2/3. Landfall: gain 1 life.
/// Whenever a Plains you control enters, a +1/+1 counter on target creature.
pub fn koth_of_the_homestead() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(trigger_is(R::Land)),
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(trigger_is(R::HasLandType(LandType::Plains))),
                effect: counters(target_filtered(R::Creature), Value::ONE),
            },
        ],
        ..legend("Koth of the Homestead", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Citizen], 2, 3)
    }
}

/// Fall of the Thran — {5}{W} Saga. I: destroy all lands. II, III: each player
/// returns two land cards from their graveyard to the battlefield.
pub fn fall_of_the_thran() -> CardDefinition {
    let regrow = || Effect::EachPlayerDoes {
        who: PlayerRef::EachPlayer,
        body: Box::new(Effect::Move {
            what: Selector::Take {
                inner: Box::new(Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Graveyard, filter: R::Land }),
                count: Box::new(Value::Const(2)),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        }),
    };
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::Destroy { what: Selector::EachPermanent(R::Land) }),
            (2, regrow()),
            (3, regrow()),
        ],
        ..spell("Fall of the Thran", cost(&[generic(5), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Lim-Dûl the Necromancer — {5}{B}{B} legendary 4/4. Whenever a creature an
/// opponent controls dies, you may pay {1}{B}; if you do, return that card to
/// the battlefield under your control, a Zombie in addition to its other types.
/// {1}{B}: regenerate target Zombie.
pub fn lim_dul_the_necromancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
            effect: Effect::MayPay {
                description: "Pay {1}{B} to return it as your Zombie?".into(),
                mana_cost: cost(&[generic(1), b()]),
                body: Box::new(Effect::Seq(vec![
                    Effect::ClearLastMoved,
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::AddCreatureTypes {
                        what: Selector::LastMoved,
                        creature_types: vec![CreatureType::Zombie],
                        duration: Duration::Permanent,
                    },
                ])),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Zombie))),
            },
            ..Default::default()
        }],
        ..legend("Lim-Dûl the Necromancer", cost(&[generic(5), b(), b()]), vec![CreatureType::Human, CreatureType::Wizard], 4, 4)
    }
}

/// Fraying Omnipotence — {3}{B}{B} sorcery: each player loses half their life,
/// then discards half their hand, then sacrifices half their creatures —
/// rounding up each time.
pub fn fraying_omnipotence() -> CardDefinition {
    let all = || Selector::Player(PlayerRef::EachPlayer);
    spell(
        "Fraying Omnipotence",
        cost(&[generic(3), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::LoseHalfLife { who: all(), rounded_up: true },
            Effect::DiscardHalf { who: all(), rounded_up: true },
            Effect::SacrificeHalf { who: all(), filter: R::Creature, rounded_up: true },
        ]),
    )
}

/// Thor, Guardian of Midgard — {3}{R}{R} legendary 5/5 flyer. Whenever a source
/// you control deals noncombat damage to an opponent, you may exile that many
/// cards from the top of your library; you may play them this turn.
pub fn thor_guardian_of_midgard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDealtNoncombatDamage, EventScope::YourSourceDamagedOpponent),
            effect: Effect::MayDo {
                description: "Exile that many cards from the top of your library to play this turn?".into(),
                body: Box::new(Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::TriggerEventAmount,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                }),
            },
        }],
        ..legend(
            "Thor, Guardian of Midgard",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::God, CreatureType::Warrior, CreatureType::Hero],
            5,
            5,
        )
    }
}

/// Getaway Car — {3} 4/3 haste Vehicle, crew 1. Whenever it attacks or blocks,
/// return up to one target creature that crewed it this turn to its owner's
/// hand.
pub fn getaway_car() -> CardDefinition {
    let bounce = || Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::Move {
            what: target_filtered(R::Creature.and(R::CrewedSourceThisTurn)),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        }),
    };
    CardDefinition {
        name: "Getaway Car",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Haste, Keyword::Crew(1)],
        triggered_abilities: vec![on_attack(bounce()), crate::effect::shortcut::blocks(bounce())],
        ..Default::default()
    }
}

/// Magus of the Future — {2}{U}{U}{U} 2/3 Human Wizard. Play with the top card
/// of your library revealed; you may play lands and cast spells from the top
/// of your library.
pub fn magus_of_the_future() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility { description: "Play with the top card of your library revealed.", effect: StaticEffect::TopOfLibraryRevealed },
            StaticAbility {
                description: "You may play lands and cast spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::Any },
            },
        ],
        ..creature("Magus of the Future", cost(&[generic(2), u(), u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 3)
    }
}

