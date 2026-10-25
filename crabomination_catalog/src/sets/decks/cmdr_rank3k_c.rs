//! Commander: one-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), third file. Tests in `tests/recent_b/cmdr_rank3k_c.rs`.

use crate::card::{
    ActivatedAbility, Adventure, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{enrage, etb, on_attack, prowess, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};
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

fn your_creature() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn counter_on(what: Selector, n: i32) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: Value::Const(n) }
}

/// Traxos, Academy Guardian — {3}{U} legendary 1/5 Dragon Construct artifact
/// creature; flying, vigilance, prowess; {2} less if you've cast a noncreature
/// spell this turn.
pub fn traxos_academy_guardian() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Prowess],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {2} less to cast if you've cast a noncreature spell this turn.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::NoncreatureSpellsCastThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
                amount: 2,
            },
        }],
        triggered_abilities: vec![prowess()],
        ..legend(
            "Traxos, Academy Guardian",
            cost(&[generic(3), u()]),
            vec![CreatureType::Dragon, CreatureType::Construct],
            1,
            5,
        )
    }
}

/// Zahid, Djinn of the Lamp — {4}{U}{U} legendary 5/6 flyer; you may pay
/// {3}{U} and tap an untapped artifact you control instead.
pub fn zahid_djinn_of_the_lamp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(3), u()]),
            tap_creatures: Some((R::Artifact, 1)),
            ..Default::default()
        }),
        ..legend("Zahid, Djinn of the Lamp", cost(&[generic(4), u(), u()]), vec![CreatureType::Djinn], 5, 6)
    }
}

/// Temporary Truce — {1}{W} sorcery: each player may draw up to two; 2 life per
/// card declined.
pub fn temporary_truce() -> CardDefinition {
    spell(
        "Temporary Truce",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::EachPlayerDrawsUpToElseGainsLife { max: 2, life_per_card: 2 },
    )
}

/// Targ Nar, Demon-Fang Gnoll — {R}{G} legendary 2/2. Pack tactics: attacking
/// with total power 6+, attackers get +1/+0. {2}{R}{G}: double its power and
/// toughness until end of turn.
pub fn targ_nar_demon_fang_gnoll() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::AttackedWithTotalPowerAtLeast { who: PlayerRef::You, at_least: 6 }),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r(), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::ToughnessOf(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend("Targ Nar, Demon-Fang Gnoll", cost(&[r(), g()]), vec![CreatureType::Gnoll], 2, 2)
    }
}

/// Sardian Cliffstomper — {1}{R} 0/4 Minotaur Barbarian; on your turn with four
/// or more Mountains, +X/+0 where X is your Mountains.
pub fn sardian_cliffstomper() -> CardDefinition {
    let mountains = || Value::count(Selector::EachPermanent(R::HasLandType(crate::card::LandType::Mountain).and(R::ControlledByYou)));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as it's your turn and you control four or more Mountains, this creature gets +X/+0.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::All(vec![
                    Predicate::IsTurnOf(PlayerRef::You),
                    Predicate::ValueAtLeast(mountains(), Value::Const(4)),
                ]),
                inner: Box::new(StaticEffect::PumpSelfByValue { amount: mountains(), per_power: 1, per_toughness: 0 }),
            },
        }],
        ..creature(
            "Sardian Cliffstomper",
            cost(&[generic(1), r()]),
            vec![CreatureType::Minotaur, CreatureType::Barbarian],
            0,
            4,
        )
    }
}

/// Kingpin, Wilson Fisk — {3}{B} legendary 3/6 menace. Whenever you sacrifice
/// Kingpin or another creature, two Treasures; once each turn.
pub fn kingpin_wilson_fisk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl).once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
        }],
        ..legend(
            "Kingpin, Wilson Fisk",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Villain],
            3,
            6,
        )
    }
}

/// Ka-Zar of the Savage Land — {4}{G} legendary 3/2. You may look at the top
/// card of your library any time and play lands from it. ETB: Zabu, a
/// legendary 2/2 green Cat with landfall "a +1/+1 counter on Zabu".
pub fn ka_zar_of_the_savage_land() -> CardDefinition {
    let zabu = Arc::new(TokenDefinition {
        name: "Zabu".into(),
        power: 2,
        toughness: 2,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(trigger_is(R::Land)),
            effect: counter_on(Selector::This, 1),
        }],
        ..Default::default()
    });
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may play lands from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::Land },
            },
        ],
        triggered_abilities: vec![etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: zabu })],
        ..legend(
            "Ka-Zar of the Savage Land",
            cost(&[generic(4), g()]),
            vec![CreatureType::Human, CreatureType::Barbarian, CreatureType::Hero],
            3,
            2,
        )
    }
}

/// Guul Draz Assassin — {B} 1/1 leveler. Level up {1}{B}. LEVEL 2-3: 2/2,
/// {B}, {T}: target creature gets -2/-2. LEVEL 4+: 4/4, {B}, {T}: -4/-4.
pub fn guul_draz_assassin() -> CardDefinition {
    use crate::card::LevelBand;
    let levels = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Level };
    let shrink = |n: i32, cond: Predicate| ActivatedAbility {
        mana_cost: cost(&[b()]),
        tap_cost: true,
        condition: Some(cond),
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-n),
            toughness: Value::Const(-n),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        level_bands: vec![
            LevelBand { min: 2, max: Some(3), power: 2, toughness: 2, keywords: vec![] },
            LevelBand { min: 4, max: None, power: 4, toughness: 4, keywords: vec![] },
        ],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                sorcery_speed: true,
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Level, amount: Value::ONE },
                ..Default::default()
            },
            shrink(
                2,
                Predicate::All(vec![
                    Predicate::ValueAtLeast(levels(), Value::Const(2)),
                    Predicate::Not(Box::new(Predicate::ValueAtLeast(levels(), Value::Const(4)))),
                ]),
            ),
            shrink(4, Predicate::ValueAtLeast(levels(), Value::Const(4))),
        ],
        ..creature("Guul Draz Assassin", cost(&[b()]), vec![CreatureType::Vampire, CreatureType::Assassin], 1, 1)
    }
}

/// Green Goblin, Back for More — {3}{B}{B} legendary 4/4 flying, menace. At the
/// beginning of combat on your turn, you may discard a card; if you do, each
/// opponent discards a card. Mayhem {3}{B}{B}.
pub fn green_goblin_back_for_more() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Menace, Keyword::Mayhem(cost(&[generic(3), b(), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::MayDiscard {
                description: "Discard a card to make each opponent discard?".into(),
                count: Value::ONE,
                then: Box::new(Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                    random: false,
                }),
                else_: None,
            },
        }],
        ..legend(
            "Green Goblin, Back for More",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Goblin, CreatureType::Human, CreatureType::Villain],
            4,
            4,
        )
    }
}

/// Geralf's Masterpiece — {3}{U}{U} 7/7 flying Zombie Horror; -1/-1 for each
/// card in your hand. {3}{U}, discard three cards: return it from your
/// graveyard to the battlefield tapped.
pub fn geralfs_masterpiece() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "This creature gets -1/-1 for each card in your hand.",
            effect: StaticEffect::PumpSelfByValue { amount: Value::HandSizeOf(PlayerRef::You), per_power: -1, per_toughness: -1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            from_graveyard: true,
            discard_cost: Some((R::Any, 3)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature(
            "Geralf's Masterpiece",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Zombie, CreatureType::Horror],
            7,
            7,
        )
    }
}

/// Faramir, Field Commander — {3}{W} legendary 3/3. End step, if a creature
/// died under your control this turn: draw. The Ring tempts you and you chose
/// another creature as your Ring-bearer: a 1/1 white Human Soldier.
pub fn faramir_field_commander() -> CardDefinition {
    let soldier = Arc::new(TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                    Predicate::CreaturesDiedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
                ),
                effect: draw(Value::ONE),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::RingTempted, EventScope::YourControl).with_filter(trigger_is(R::OtherThanSource)),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: soldier },
            },
        ],
        ..legend(
            "Faramir, Field Commander",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Ebondeath, Dracolich — {2}{B}{B} legendary 5/2, flash, flying; enters tapped;
/// castable from your graveyard if a creature not named Ebondeath died this
/// turn.
pub fn ebondeath_dracolich() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::GraveyardCast],
        flashback_condition: Some(Predicate::CreatureDiedThisTurnMatching {
            filter: R::HasName("Ebondeath, Dracolich".into()).negate(),
        }),
        static_abilities: vec![crate::sets::enters_tapped()],
        ..legend(
            "Ebondeath, Dracolich",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Dragon],
            5,
            2,
        )
    }
}

/// Dread — {3}{B}{B}{B} 6/6 Elemental Incarnation, fear. Whenever a creature an
/// opponent controls deals damage to you, destroy it. Put into a graveyard from
/// anywhere: shuffled into its owner's library.
/// (Residual: like No Mercy, your own creatures' damage is not watched.)
pub fn dread() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PlayerDamaged, EventScope::OpponentSourceDamagedYou),
                effect: Effect::Destroy { what: Selector::TriggerSource },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Shuffled },
                },
            },
        ],
        ..creature(
            "Dread",
            cost(&[generic(3), b(), b(), b()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            6,
            6,
        )
    }
}

/// Dawn of a New Age — {1}{W} enchantment; enters with a hope counter per
/// creature you control. Your end step: remove a hope counter; if you do, draw.
/// Then with none left, sacrifice it and gain 4 life.
pub fn dawn_of_a_new_age() -> CardDefinition {
    let hope = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Hope };
    CardDefinition {
        enters_with_counters: Some((CounterType::Hope, Value::count(Selector::EachPermanent(your_creature())))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::ValueAtLeast(hope(), Value::ONE),
                    then: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter { what: Selector::This, kind: CounterType::Hope, amount: Value::ONE },
                        draw(Value::ONE),
                    ])),
                    else_: Box::new(Effect::Noop),
                },
                Effect::If {
                    cond: Predicate::Not(Box::new(Predicate::ValueAtLeast(hope(), Value::ONE))),
                    then: Box::new(Effect::Seq(vec![
                        Effect::SacrificeSource,
                        Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..spell("Dawn of a New Age", cost(&[generic(1), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Cloudkill — {4}{B}{B} sorcery: all creatures get -X/-X until end of turn,
/// X the greatest mana value of a commander you own on the battlefield or in
/// the command zone.
pub fn cloudkill() -> CardDefinition {
    let x = || Value::GreatestCommanderManaValueInPlayOrCommandZone(PlayerRef::You);
    spell(
        "Cloudkill",
        cost(&[generic(4), b(), b()]),
        CardType::Sorcery,
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: Value::Negate(Box::new(x())),
            toughness: Value::Negate(Box::new(x())),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Bofur, Reliable Guardian // Concerted Care — {W} legendary 1/1 lifelink
/// Dwarf Scout. Adventure {1}{W} instant: target artifact or creature you
/// control gains hexproof and indestructible until end of turn.
pub fn bofur_reliable_guardian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        adventure: Some(Box::new(Adventure {
            name: "Concerted Care",
            cost: cost(&[generic(1), w()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Artifact.or(R::Creature).and(R::ControlledByYou)),
                    keyword: Keyword::Hexproof,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
        })),
        ..legend("Bofur, Reliable Guardian", cost(&[w()]), vec![CreatureType::Dwarf, CreatureType::Scout], 1, 1)
    }
}

/// Black Panther, Most Dangerous — {2}{W} legendary 3/3. Whenever he is dealt
/// damage, he deals that much damage to any other target. Power-up {5}{W}{W}:
/// two +1/+1 counters on him; other creatures you control get +2/+2 until end
/// of turn.
pub fn black_panther_most_dangerous() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![enrage(Effect::DealDamage {
            // "any other target": only a creature can be himself.
            to: target_filtered(R::Creature.and(R::OtherThanSource).or(R::Player).or(R::Planeswalker).or(R::HasCardType(CardType::Battle))),
            amount: Value::TriggerEventAmount,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), w(), w()]),
            power_up: true,
            effect: Effect::Seq(vec![
                counter_on(Selector::This, 2),
                Effect::PumpPT {
                    what: Selector::EachPermanent(your_creature().and(R::OtherThanSource)),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Black Panther, Most Dangerous",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero],
            3,
            3,
        )
    }
}

/// Bane's Contingency — {1}{U}{U} instant: counter target spell; if it targets
/// a commander you control, also scry 2 and draw a card.
pub fn banes_contingency() -> CardDefinition {
    spell(
        "Bane's Contingency",
        cost(&[generic(1), u(), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::SpellTargetsMatching(Box::new(R::IsCommander.and(R::ControlledByYou))),
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
                    Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                    draw(Value::ONE),
                ])),
                else_: Box::new(Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) }),
            },
        ]),
    )
}

/// Ayara's Oathsworn — {1}{B} 2/2 menace Human Knight. Combat damage to a
/// player: with fewer than four +1/+1 counters, add one; then at exactly four,
/// tutor a card.
pub fn ayaras_oathsworn() -> CardDefinition {
    let counters = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne };
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::Not(Box::new(Predicate::ValueAtLeast(counters(), Value::Const(4))))),
            effect: Effect::Seq(vec![
                counter_on(Selector::This, 1),
                Effect::If {
                    cond: Predicate::All(vec![
                        Predicate::ValueAtLeast(counters(), Value::Const(4)),
                        Predicate::Not(Box::new(Predicate::ValueAtLeast(counters(), Value::Const(5)))),
                    ]),
                    then: Box::new(Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Hand(PlayerRef::You) }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..creature("Ayara's Oathsworn", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

/// April, Reporter of the Weird — {2}{U} legendary 2/2. Combat damage to a
/// player: draw that many, then discard a card.
pub fn april_reporter_of_the_weird() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                draw(Value::TriggerEventAmount),
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        }],
        ..legend(
            "April, Reporter of the Weird",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Detective],
            2,
            2,
        )
    }
}

/// Amber Gristle O'Maul — {3}{R} legendary 3/3 haste, choose a Background.
/// Attacks: you may discard your hand; if you do, draw a card for each player
/// being attacked.
pub fn amber_gristle_omaul() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::ChooseABackground],
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Discard your hand to draw a card for each player being attacked?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
                draw(Value::OpponentsAttackedThisCombat),
            ])),
        })],
        ..legend(
            "Amber Gristle O'Maul",
            cost(&[generic(3), r()]),
            vec![CreatureType::Dwarf, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Sly Requisitioner — {4}{B} 2/2 Human Artificer, improvise. Whenever a
/// nontoken artifact you control is put into a graveyard from the battlefield,
/// a 1/1 colorless Servo artifact creature.
pub fn sly_requisitioner() -> CardDefinition {
    let servo = Arc::new(TokenDefinition {
        name: "Servo".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Servo], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Improvise],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                .with_filter(trigger_is(R::Artifact.and(R::NotToken))),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: servo },
        }],
        ..creature(
            "Sly Requisitioner",
            cost(&[generic(4), b()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            2,
        )
    }
}

/// Subjugator Angel — {4}{W}{W} 4/3 flyer; ETB: tap all creatures your
/// opponents control.
pub fn subjugator_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Tap {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
        })],
        ..creature("Subjugator Angel", cost(&[generic(4), w(), w()]), vec![CreatureType::Angel], 4, 3)
    }
}

/// Shattered Perception — {2}{R} sorcery: discard your hand, draw that many.
/// Flashback {5}{R}.
pub fn shattered_perception() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), r()]))],
        ..spell(
            "Shattered Perception",
            cost(&[generic(2), r()]),
            CardType::Sorcery,
            Effect::DiscardHandDrawThatMany { who: Selector::You },
        )
    }
}

/// Mister Hyde, Monster Within — {2}{G} legendary 2/2. Upkeep, choose one: a
/// +1/+1 counter on him; or remove a counter from a creature you control, and
/// if you do, draw a card.
pub fn mister_hyde_monster_within() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::ChooseMode(vec![
                counter_on(Selector::This, 1),
                Effect::RemoveAnyCounterFromAmongThen { filter: your_creature(), then: Box::new(draw(Value::ONE)) },
            ]),
        }],
        ..legend(
            "Mister Hyde, Monster Within",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Villain],
            2,
            2,
        )
    }
}

/// Nazar, the Velvet Fang — {3}{B} legendary 3/3 menace. Whenever you gain
/// life, a feeding counter on Nazar. Attacks: you may remove three feeding
/// counters; if you do, draw three and lose 3 life.
pub fn nazar_the_velvet_fang() -> CardDefinition {
    let feeding = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Feeding };
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Feeding, amount: Value::ONE },
            },
            on_attack(Effect::If {
                cond: Predicate::ValueAtLeast(feeding(), Value::Const(3)),
                then: Box::new(Effect::MayDo {
                    description: "Remove three feeding counters to draw three and lose 3 life?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter { what: Selector::This, kind: CounterType::Feeding, amount: Value::Const(3) },
                        draw(Value::Const(3)),
                        Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            }),
        ],
        ..legend(
            "Nazar, the Velvet Fang",
            cost(&[generic(3), b()]),
            vec![CreatureType::Vampire, CreatureType::Warlock],
            3,
            3,
        )
    }
}

/// Instrument of the Bards — {G} legendary artifact. Upkeep: you may put a
/// harmony counter on it. {3}{G}, {T}: search for a creature card with mana
/// value equal to its harmony counters, reveal it, put it into your hand; if
/// it's legendary, create a Treasure. Then shuffle.
pub fn instrument_of_the_bards() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Put a harmony counter on Instrument of the Bards?".into(),
                body: Box::new(Effect::AddCounter { what: Selector::This, kind: CounterType::Harmony, amount: Value::ONE }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::ManaValueEqualsSourceCounters(CounterType::Harmony)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::LastMoved,
                        filter: R::HasSupertype(Supertype::Legendary),
                    },
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(crabomination_base::tokens::treasure_token()),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..spell("Instrument of the Bards", cost(&[g()]), CardType::Artifact, Effect::Noop)
    }
}

/// Popular Entertainer — {1}{R} legendary Background. Commander creatures you
/// own have "Whenever one or more creatures you control deal combat damage to
/// a player, goad target creature that player controls."
pub fn popular_entertainer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Background],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you own have \"Whenever one or more creatures you control deal combat damage to a player, goad target creature that player controls.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::IsCommander).and(R::OwnedByYou),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch(),
                    effect: Effect::Goad { what: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)) },
                }),
            },
        }],
        ..spell("Popular Entertainer", cost(&[generic(1), r()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Pelargir Survivor — {1}{U} 1/3 Human Peasant. {T}: one mana of any color,
/// spent only on an instant or sorcery spell. {5}{U}, {T}: target player mills
/// three.
pub fn pelargir_survivor() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        crate::mana::SpendRestriction::InstantSorceryOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5), u()]),
                tap_cost: true,
                effect: Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(3) },
                ..Default::default()
            },
        ],
        ..creature("Pelargir Survivor", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Peasant], 1, 3)
    }
}

/// Baru, Fist of Krosa — {3}{G}{G} legendary 4/4. Whenever a Forest enters,
/// green creatures you control get +1/+1 and trample until end of turn.
/// Grandeur — discard another Baru: an X/X green Wurm, X = lands you control.
pub fn baru_fist_of_krosa() -> CardDefinition {
    let green = || Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Green)).and(R::ControlledByYou));
    let lands = || Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByYou)));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(trigger_is(R::HasLandType(crate::card::LandType::Forest))),
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: green(), power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: green(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::HasName("Baru, Fist of Krosa".to_string()), 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Wurm".into(),
                    static_abilities: vec![crate::card::StaticAbility {
                        description: "This creature's power and toughness are each equal to the number of lands you control.",
                        effect: crate::card::StaticEffect::SelfBasePtFromValue { power: lands(), toughness: lands() },
                    }],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        ..legend("Baru, Fist of Krosa", cost(&[generic(3), g(), g()]), vec![CreatureType::Human, CreatureType::Druid], 4, 4)
    }
}

/// Leatherhead, Swamp Stalker — {2}{G}{G} legendary 5/4 trample; enters with a
/// hexproof counter. Combat damage to a player: you may remove a counter from
/// her; when you do, destroy target artifact or enchantment that player
/// controls. The counter is an as-enters replacement (CR 614.1c).
pub fn leatherhead_swamp_stalker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Leatherhead enters with a hexproof counter on her.",
            effect: crate::effect::StaticEffect::SelfEntersWithKeywordCounter { keyword: Keyword::Hexproof, if_cast_from_hand: false },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                    .with_filter(Predicate::Not(Box::new(Predicate::EntityMatches {
                        what: Selector::This,
                        filter: R::HasNoCounters,
                    }))),
                effect: Effect::MayDo {
                    description: "Remove a counter to destroy an artifact or enchantment?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RemoveAnyCounter { what: Selector::This },
                        Effect::Reflexive {
                            body: Box::new(Effect::Destroy {
                                what: target_filtered(
                                    R::Artifact.or(R::Enchantment).and(R::ControlledByTriggerPlayer),
                                ),
                            }),
                        },
                    ])),
                },
            },
        ],
        ..legend(
            "Leatherhead, Swamp Stalker",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Crocodile, CreatureType::Mutant, CreatureType::Rogue],
            5,
            4,
        )
    }
}

/// The Duke, Rebel Sentry — {W} legendary 0/1; enters with a +1/+1 counter.
/// {T}, remove a counter from him: a +1/+1 counter on another target creature
/// you control; it gains hexproof until end of turn. (Residual: the cost
/// removes a +1/+1 counter, the only kind he enters with.)
pub fn the_duke_rebel_sentry() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::Seq(vec![
                counter_on(target_filtered(your_creature().and(R::OtherThanSource)), 1),
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Hexproof, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..legend(
            "The Duke, Rebel Sentry",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Ally],
            0,
            1,
        )
    }
}
