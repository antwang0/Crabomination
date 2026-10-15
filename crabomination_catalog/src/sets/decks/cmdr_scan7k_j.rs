//! Commander: two-card gaps of the 2026-10-15 full-list EDHREC scan, second
//! pass. Tests in `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};
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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn token(name: &str, color: Option<Color>, types: Vec<CreatureType>, p: i32, t: i32) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: color.into_iter().collect(),
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn mint(def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(def) }
}

fn on_your_instant_or_sorcery(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: instant_or_sorcery() }),
        effect,
    }
}

// ── Tura Kennerüd, Skyknight / Raff, Weatherlight Stalwart ─────────────────

/// Tura Kennerüd, Skyknight — {2}{W}{U}{U} legendary 3/3 Human Knight; flying.
/// Whenever you cast an instant or sorcery spell, a 1/1 white Soldier.
pub fn tura_kennerud_skyknight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_your_instant_or_sorcery(mint(token(
            "Soldier",
            Some(Color::White),
            vec![CreatureType::Soldier],
            1,
            1,
        )))],
        ..legend(
            "Tura Kennerüd, Skyknight",
            cost(&[generic(2), w(), u(), u()]),
            vec![CreatureType::Human, CreatureType::Knight],
            3,
            3,
        )
    }
}

/// Esgaroth Garrison — {4}{W} */5 Human Soldier; its power is the number of
/// creatures you control. ETB: recruit (draw, then discard; a nonland discard
/// makes a 1/1 white Human Soldier).
pub fn esgaroth_garrison() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Esgaroth Garrison's power is equal to the number of creatures you control.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: Value::count(yours(R::Creature)),
                toughness: Value::Const(5),
            },
        }],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            Effect::If {
                cond: Predicate::DiscardedNonlandThisEffect { who: PlayerRef::You },
                then: Box::new(mint(token(
                    "Human Soldier",
                    Some(Color::White),
                    vec![CreatureType::Human, CreatureType::Soldier],
                    1,
                    1,
                ))),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature("Esgaroth Garrison", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Soldier], 0, 5)
    }
}

/// Stonybrook Schoolmaster — {2}{W} 1/2 Merfolk Wizard. Whenever it becomes
/// tapped, you may create a 1/1 blue Merfolk Wizard.
pub fn stonybrook_schoolmaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Create a 1/1 blue Merfolk Wizard?".into(),
                body: Box::new(mint(token(
                    "Merfolk Wizard",
                    Some(Color::Blue),
                    vec![CreatureType::Merfolk, CreatureType::Wizard],
                    1,
                    1,
                ))),
            },
        }],
        ..creature(
            "Stonybrook Schoolmaster",
            cost(&[generic(2), w()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            1,
            2,
        )
    }
}

// ── Jolrael, Mwonvuli Recluse ──────────────────────────────────────────────

/// Dissenter's Deliverance — {1}{G} instant: destroy target artifact.
/// Cycling {G}.
pub fn dissenters_deliverance() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[g()]))],
        ..spell(
            "Dissenter's Deliverance",
            cost(&[generic(1), g()]),
            CardType::Instant,
            Effect::Destroy { what: target_filtered(R::Artifact) },
        )
    }
}

/// Beneath the Sands — {2}{G} sorcery: a basic land onto the battlefield
/// tapped. Cycling {2}.
pub fn beneath_the_sands() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..spell(
            "Beneath the Sands",
            cost(&[generic(2), g()]),
            CardType::Sorcery,
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        )
    }
}

// ── Bebop & Rocksteady ─────────────────────────────────────────────────────

/// Bebop, Warthog Warrior — {4}{B} legendary 5/4 Boar Mutant Warrior; menace.
/// Rhinos you control have menace. Swampcycling {2}.
pub fn bebop_warthog_warrior() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Landcycling(cost(&[generic(2)]), LandType::Swamp)],
        static_abilities: vec![StaticAbility {
            description: "Rhinos you control have menace.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::HasCreatureType(CreatureType::Rhino)),
                keyword: Keyword::Menace,
            },
        }],
        ..legend(
            "Bebop, Warthog Warrior",
            cost(&[generic(4), b()]),
            vec![CreatureType::Boar, CreatureType::Mutant, CreatureType::Warrior],
            5,
            4,
        )
    }
}

/// Rocksteady, Crash Courser — {4}{G}{G} legendary 7/7 Rhino Mutant; can't be
/// blocked by more than one creature, nor can Boars you control.
/// Forestcycling {2}.
pub fn rocksteady_crash_courser() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CantBeBlockedByMoreThanOne,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Forest),
        ],
        static_abilities: vec![StaticAbility {
            description: "Boars you control can't be blocked by more than one creature.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::HasCreatureType(CreatureType::Boar)),
                keyword: Keyword::CantBeBlockedByMoreThanOne,
            },
        }],
        ..legend(
            "Rocksteady, Crash Courser",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Rhino, CreatureType::Mutant],
            7,
            7,
        )
    }
}

// ── Madame Null, Power Broker ──────────────────────────────────────────────

/// Baleful Ammit — {2}{B} 4/3 Crocodile Demon; lifelink. ETB: a -1/-1 counter
/// on target creature you control.
pub fn baleful_ammit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            kind: CounterType::MinusOneMinusOne,
            amount: Value::ONE,
        })],
        ..creature("Baleful Ammit", cost(&[generic(2), b()]), vec![CreatureType::Crocodile, CreatureType::Demon], 4, 3)
    }
}

/// Fell Beast of Mordor — {2}{B}{B} 3/3 Drake Beast; flying, devour 1.
/// Whenever it enters or attacks, target opponent loses X life and you gain
/// X, where X is its +1/+1 counters.
pub fn fell_beast_of_mordor() -> CardDefinition {
    let drain = || {
        let n = Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne };
        Effect::Seq(vec![
            Effect::LoseLife { who: target_filtered(R::Player.and(R::ControlledByOpponent)), amount: n.clone() },
            Effect::GainLife { who: Selector::You, amount: n },
        ])
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Devour(1)],
        triggered_abilities: vec![etb(drain()), on_attack(drain())],
        ..creature(
            "Fell Beast of Mordor",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Drake, CreatureType::Beast],
            3,
            3,
        )
    }
}

// ── The Destined Black Mage ────────────────────────────────────────────────

/// The Destined Black Mage — {2}{B} legendary 3/2 Human Wizard; deathtouch.
/// {B}, {T}: another target creature you control gains deathtouch until end
/// of turn. Whenever you cast a noncreature spell, 1 damage to each opponent
/// (3 with a full party).
pub fn the_destined_black_mage() -> CardDefinition {
    let ping = |n: i32| Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(n) };
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Not(Box::new(R::Creature)),
            }),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(Value::PartyCount, Value::Const(4)),
                then: Box::new(ping(3)),
                else_: Box::new(ping(1)),
            },
        }],
        ..legend(
            "The Destined Black Mage",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            2,
        )
    }
}

/// Stalactite Dagger — {2} Equipment. ETB: a 1/1 colorless Shapeshifter
/// token with changeling. Equipped creature gets +1/+1 and is all creature
/// types. Equip {2}.
pub fn stalactite_dagger() -> CardDefinition {
    CardDefinition {
        name: "Stalactite Dagger",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Equipped creature is all creature types.",
            effect: StaticEffect::EveryCreatureType { applies_to: Selector::AttachedTo(Box::new(Selector::This)) },
        }],
        triggered_abilities: vec![etb(mint(TokenDefinition {
            keywords: vec![Keyword::Changeling],
            ..token("Shapeshifter", None, vec![CreatureType::Shapeshifter], 1, 1)
        }))],
        ..Default::default()
    }
}

// ── Iron Fist, Living Weapon ───────────────────────────────────────────────

/// Loki, God of Lies — {1}{R}{R} legendary 3/3 God Sorcerer Villain. Whenever
/// you cast a spell that targets only a single creature, gain control of it
/// until end of turn; on your turn, untap it and it gains haste.
pub fn loki_god_of_lies() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellTargetsOnlyOneMatching(R::Creature)),
            effect: Effect::Seq(vec![
                Effect::GainControl { what: Selector::CastSpellTarget(0), to: None, duration: Duration::EndOfTurn },
                Effect::If {
                    cond: Predicate::IsTurnOf(PlayerRef::You),
                    then: Box::new(Effect::Seq(vec![
                        Effect::Untap { what: Selector::CastSpellTarget(0), up_to: None },
                        Effect::GrantKeyword {
                            what: Selector::CastSpellTarget(0),
                            keyword: Keyword::Haste,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..legend(
            "Loki, God of Lies",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::God, CreatureType::Sorcerer, CreatureType::Villain],
            3,
            3,
        )
    }
}

/// Bionic Blow — {X}{R}{R} sorcery. Target creature you control gets +X/+0
/// until end of turn, then deals damage equal to its power to up to one other
/// target creature.
pub fn bionic_blow() -> CardDefinition {
    spell(
        "Bionic Blow",
        cost(&[x(), r(), r()]),
        CardType::Sorcery,
        Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::XFromCost,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::DealDamageEqualToPower {
                    source: Selector::Target(0),
                    target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::OtherThanTargetSlot(0)) },
                },
            ])),
        },
    )
}

// ── Urza, Powerstone Prodigy ───────────────────────────────────────────────

/// Network Terminal — {3} artifact. {T}: add one mana of any color. {1}, {T},
/// tap another untapped artifact you control: draw, then discard.
pub fn network_terminal() -> CardDefinition {
    CardDefinition {
        name: "Network Terminal",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            crate::catalog::sets::tap_add_any_color(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                tap_others_cost: Some((R::Artifact.and(R::OtherThanSource), 1)),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Urza's Command — {2}{U}{U} instant; choose two: creatures you don't
/// control get -2/-0 until end of turn; a tapped Powerstone; a tapped 0/0
/// Construct that gets +1/+1 per artifact you control; scry 1, then draw.
pub fn urzas_command() -> CardDefinition {
    let construct = TokenDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        // Live, not a mint-time `dynamic_pt`: the count tracks the board.
        static_abilities: vec![StaticAbility {
            description: "This token gets +1/+1 for each artifact you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents { filter: R::Artifact, per_power: 1, per_toughness: 1 },
        }],
        tapped: true,
        ..token("Construct", None, vec![CreatureType::Construct], 0, 0)
    };
    spell(
        "Urza's Command",
        cost(&[generic(2), u(), u()]),
        CardType::Instant,
        Effect::ChooseModesCast {
            min: 2,
            max: 2,
            allow_repeats: false,
            modes: vec![
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
                    power: Value::Const(-2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                mint(TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() }),
                mint(construct),
                Effect::Seq(vec![
                    Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
            ],
        },
    )
}

// ── Astonishing Spider-Man ─────────────────────────────────────────────────

/// Astonishing Spider-Man — {2}{U/R}{U/R} legendary 4/4 Spider Human Hero;
/// reach. Attacking, you may discard your hand; if you do, draw a card for
/// each card you've discarded this turn.
pub fn astonishing_spider_man() -> CardDefinition {
    use crate::mana::hybrid;
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Discard your hand to draw a card per card discarded this turn?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
                Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisTurn(PlayerRef::You) },
            ])),
        })],
        ..legend(
            "Astonishing Spider-Man",
            cost(&[generic(2), hybrid(Color::Blue, Color::Red), hybrid(Color::Blue, Color::Red)]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            4,
            4,
        )
    }
}

/// Spider-Sense — {1}{U} instant; web-slinging {U}. Counter target instant
/// spell, sorcery spell, or triggered ability.
pub fn spider_sense() -> CardDefinition {
    CardDefinition {
        // CR 702.188 — web-slinging: {U} and return a tapped creature you
        // control to its owner's hand.
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[u()]),
            return_to_hand: Some((R::Creature.and(R::Tapped), 1)),
            ..Default::default()
        }),
        ..spell(
            "Spider-Sense",
            cost(&[generic(1), u()]),
            CardType::Instant,
            Effect::CounterSpellOrAbility {
                what: target_filtered(R::IsSpellOnStack.and(instant_or_sorcery()).or(R::HasTriggeredAbilityOnStack)),
            },
        )
    }
}

// ── Sachi, Daughter of Seshiro ─────────────────────────────────────────────

/// Centaur Omenreader — {3}{G} snow 3/3 Centaur Shaman. While it's tapped,
/// creature spells you cast cost {2} less.
pub fn centaur_omenreader() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is tapped, creature spells you cast cost {2} less to cast.",
            effect: StaticEffect::CostReductionWhile {
                filter: R::Creature,
                amount: 2,
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::Tapped },
            },
        }],
        ..creature("Centaur Omenreader", cost(&[generic(3), g()]), vec![CreatureType::Centaur, CreatureType::Shaman], 3, 3)
    }
}

/// Leaf-Crowned Elder — {2}{G}{G} 3/5 Treefolk Shaman. Kinship: at your
/// upkeep, if the top card of your library shares a creature type with it, you
/// may reveal it and play it without paying its mana cost. (A land never has
/// a creature type, so "play" only ever casts here.)
pub fn leaf_crowned_elder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            // The kinship test runs here, with the Elder as the source: the
            // reveal-and-cast walk reads its filter without one.
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                    filter: R::SharesCreatureTypeWithSource,
                },
                then: Box::new(Effect::RevealTopMayCastFreeKeepOnTop { filter: R::Any }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature(
            "Leaf-Crowned Elder",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Treefolk, CreatureType::Shaman],
            3,
            5,
        )
    }
}

// ── Agatha of the Vile Cauldron ────────────────────────────────────────────

/// Elder of Laurels — {2}{G} 2/3 Human Advisor. {3}{G}: target creature gets
/// +X/+X until end of turn, where X is the number of creatures you control.
pub fn elder_of_laurels() -> CardDefinition {
    let n = Value::count(yours(R::Creature));
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: n.clone(),
                toughness: n,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Elder of Laurels", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Advisor], 2, 3)
    }
}

/// Swashbuckler's Whip — {1} Equipment. Equipped creature has reach, "{2},
/// {T}: tap target artifact or creature," and "{8}, {T}: discover 10."
/// Equip {1}.
pub fn swashbucklers_whip() -> CardDefinition {
    let grant = |ability: ActivatedAbility| StaticEffect::GrantActivatedAbility {
        applies_to: Selector::AttachedTo(Box::new(Selector::This)),
        ability,
        condition: None,
    };
    CardDefinition {
        name: "Swashbuckler's Whip",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Reach], ..Default::default() }),
        static_abilities: vec![
            StaticAbility {
                description: "Equipped creature has \"{2}, {T}: Tap target artifact or creature.\"",
                effect: grant(ActivatedAbility {
                    mana_cost: cost(&[generic(2)]),
                    tap_cost: true,
                    effect: Effect::Tap { what: target_filtered(R::Artifact.or(R::Creature)) },
                    ..Default::default()
                }),
            },
            StaticAbility {
                description: "Equipped creature has \"{8}, {T}: Discover 10.\"",
                effect: grant(ActivatedAbility {
                    mana_cost: cost(&[generic(8)]),
                    tap_cost: true,
                    effect: Effect::Discover { n: Value::Const(10), filter: None },
                    ..Default::default()
                }),
            },
        ],
        ..Default::default()
    }
}
