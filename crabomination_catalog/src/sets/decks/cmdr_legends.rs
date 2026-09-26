//! Most-built commanders missing from the catalog (COMMANDER_BACKLOG §1):
//! Aragorn, the Uniter; Zur the Enchanter; Flubs, the Fool; Galadriel,
//! Light of Valinor; Arcades, the Strategist; Tiamat; Sauron, the Dark Lord;
//! High Perfect Morcant; Jodah, the Unifier; Thranduil; Queza; Child of
//! Alara; Liesa; Urtet. Each is built from primitives
//! other cards already exercise (Jodah adds a filter to cascade's walk and
//! `Predicate::CastSpellFromHand`).
//!
//! - **Galadriel** — alliance's "one that hasn't been chosen this turn" is
//!   `Effect::ChooseUnchosenModeThisTurn` (Teval's Judgment).
//! - **Arcades** — Doran's `AssignsCombatDamageByToughness` grant, narrowed
//!   to defenders, plus the defender-can-attack static.
//! - **Tiamat** — `SearchUpToN`; the "different names" rider is moot in a
//!   singleton deck (CR 903.5b), which is where Tiamat is played.

use std::sync::Arc;

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn legend(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        ..Default::default()
    }
}

fn on_cast_color(color: Color, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::HasColor(color))),
        effect,
    }
}

/// Aragorn, the Uniter — {R}{G}{W}{U} 5/5. Whenever you cast a white spell,
/// create a 1/1 white Human Soldier; blue, scry 2; red, 3 damage to target
/// opponent; green, target creature gets +4/+4 until end of turn. A
/// multicolored spell fires each of its colors (CR 105.2).
pub fn aragorn_the_uniter() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            on_cast_color(
                Color::White,
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(soldier) },
            ),
            on_cast_color(Color::Blue, Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
            on_cast_color(
                Color::Red,
                Effect::DealDamage { to: target_filtered(R::Player.and(R::OpponentPlayer)), amount: Value::Const(3) },
            ),
            on_cast_color(
                Color::Green,
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
        ..legend(
            "Aragorn, the Uniter",
            cost(&[r(), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Noble],
            5,
            5,
        )
    }
}

/// Zur the Enchanter — {1}{W}{U}{B} 1/4 flier. Whenever Zur attacks, you may
/// search your library for an enchantment card with mana value 3 or less,
/// put it onto the battlefield, then shuffle.
pub fn zur_the_enchanter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Search {
            who: PlayerRef::You,
            filter: R::Enchantment.and(R::ManaValueAtMost(3)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        ..legend(
            "Zur the Enchanter",
            cost(&[generic(1), w(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            4,
        )
    }
}

/// Flubs, the Fool — {G}{U}{R} 0/5. You may play an additional land on each
/// of your turns. Whenever you play a land or cast a spell, draw a card if
/// you have no cards in hand; otherwise, discard a card.
pub fn flubs_the_fool() -> CardDefinition {
    let draw_or_discard = || Effect::If {
        cond: Predicate::ValueAtLeast(Value::Const(0), Value::HandSizeOf(PlayerRef::You)),
        then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        else_: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
                effect: draw_or_discard(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                effect: draw_or_discard(),
            },
        ],
        ..legend("Flubs, the Fool", cost(&[g(), u(), r()]), vec![CreatureType::Frog, CreatureType::Scout], 0, 5)
    }
}

/// Galadriel, Light of Valinor — {2}{G}{W}{U} 3/3. Alliance — whenever
/// another creature you control enters, choose one that hasn't been chosen
/// this turn: add {G}{G}{G}; a +1/+1 counter on each creature you control;
/// or scry 2, then draw a card.
pub fn galadriel_light_of_valinor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::ChooseUnchosenModeThisTurn {
                modes: vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Colors(vec![Color::Green, Color::Green, Color::Green]),
                    },
                    Effect::AddCounter {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::Seq(vec![
                        Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ]),
                ],
            },
        }],
        ..legend(
            "Galadriel, Light of Valinor",
            cost(&[generic(2), g(), w(), u()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            3,
            3,
        )
    }
}

/// Arcades, the Strategist — {1}{G}{W}{U} 3/5 flying, vigilance. Whenever a
/// creature you control with defender enters, draw a card. Each creature
/// you control with defender assigns combat damage equal to its toughness
/// (CR 510.1c) and can attack as though it didn't have defender.
pub fn arcades_the_strategist() -> CardDefinition {
    let defender = || R::Creature.and(R::HasKeyword(Keyword::Defender));
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: defender() }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "Each creature you control with defender assigns combat damage equal to its toughness rather than its power.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(defender().and(R::ControlledByYou)),
                    keyword: Keyword::AssignsCombatDamageByToughness,
                },
            },
            StaticAbility {
                description: "Creatures you control with defender can attack as though they didn't have defender.",
                effect: StaticEffect::YourCreaturesCanAttackAsThoughNoDefender,
            },
        ],
        ..legend(
            "Arcades, the Strategist",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Elder, CreatureType::Dragon],
            3,
            5,
        )
    }
}

/// Tiamat — {2}{W}{U}{B}{R}{G} 7/7 Dragon God, flying. When it enters, if
/// you cast it, search your library for up to five Dragon cards not named
/// Tiamat with different names, put them into your hand, then shuffle.
pub fn tiamat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SourceWasCast,
            then: Box::new(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::HasCreatureType(CreatureType::Dragon).and(R::HasName("Tiamat".into()).negate()),
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(5),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..legend(
            "Tiamat",
            cost(&[generic(2), w(), u(), b(), r(), g()]),
            vec![CreatureType::Dragon, CreatureType::God],
            7,
            7,
        )
    }
}

/// Sauron, the Dark Lord — {3}{U}{B}{R} 7/6 Avatar Horror. Ward—sacrifice a
/// legendary artifact or legendary creature. Whenever an opponent casts a
/// spell, amass Orcs 1 (CR 701.47). Whenever an Army you control deals
/// combat damage to a player, the Ring tempts you (CR 701.54). Whenever the
/// Ring tempts you, you may discard your hand; if you do, draw four.
pub fn sauron_the_dark_lord() -> CardDefinition {
    let legendary_fodder = R::HasSupertype(Supertype::Legendary).and(R::Artifact.or(R::Creature));
    CardDefinition {
        keywords: vec![Keyword::Ward(crate::card::WardCost::SacrificeMatching(Box::new(legendary_fodder)))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
                effect: Effect::Amass { who: PlayerRef::You, count: Value::ONE, extra_type: Some(CreatureType::Orc) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Army),
                    },
                ),
                effect: Effect::RingTempts { who: PlayerRef::You },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::RingTempted, EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Discard your hand to draw four cards?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard {
                            who: Selector::You,
                            amount: Value::HandSizeOf(PlayerRef::You),
                            random: false,
                        },
                        Effect::Draw { who: Selector::You, amount: Value::Const(4) },
                    ])),
                },
            },
        ],
        ..legend(
            "Sauron, the Dark Lord",
            cost(&[generic(3), u(), b(), r()]),
            vec![CreatureType::Avatar, CreatureType::Horror],
            7,
            6,
        )
    }
}

/// High Perfect Morcant — {2}{B}{G} 4/4 Elf Noble. Whenever it or another
/// Elf you control enters, each opponent blights 1 (CR 701.68). Tap three
/// untapped Elves you control: proliferate, only as a sorcery. (The tap
/// cost picks from Morcant's *other* Elves; Morcant itself isn't offered.)
pub fn high_perfect_morcant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCreatureType(CreatureType::Elf) },
            ),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::EachOpponent,
                body: Box::new(Effect::Blight { n: Value::ONE }),
            },
        }],
        activated_abilities: vec![crate::card::ActivatedAbility {
            tap_n_filter: Some((R::HasCreatureType(CreatureType::Elf), 3)),
            sorcery_speed: true,
            effect: Effect::Proliferate,
            ..Default::default()
        }],
        ..legend(
            "High Perfect Morcant",
            cost(&[generic(2), b(), g()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            4,
            4,
        )
    }
}

/// Jodah, the Unifier — {W}{U}{B}{R}{G} 5/5. Legendary creatures you control
/// get +X/+X, X = the number of legendary creatures you control. Whenever
/// you cast a legendary spell from your hand, exile from the top until a
/// legendary nonland card with lesser mana value; you may cast it free; the
/// rest go to the bottom (cascade's walk, CR 702.85a, narrowed to
/// legendaries).
pub fn jodah_the_unifier() -> CardDefinition {
    let my_legends = || Selector::EachPermanent(
        R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou),
    );
    let x = || Value::CountOf(Box::new(my_legends()));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Legendary creatures you control get +X/+X, where X is the number of legendary creatures you control.",
            effect: StaticEffect::PumpPTByValue { applies_to: my_legends(), power: x(), toughness: x() },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(R::HasSupertype(Supertype::Legendary)),
                Predicate::CastSpellFromHand,
            ])),
            effect: Effect::Cascade {
                max_mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                filter: Some(R::HasSupertype(Supertype::Legendary)),
            },
        }],
        ..legend(
            "Jodah, the Unifier",
            cost(&[w(), u(), b(), r(), g()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            5,
            5,
        )
    }
}

/// Thranduil, the Elvenking — {2}{B}{G}{U} 5/6. Has all activated abilities
/// of all Elf cards in your graveyard. Whenever another legendary Elf you
/// control enters, draw two cards, then discard a card.
pub fn thranduil_the_elvenking() -> CardDefinition {
    let elf = || R::HasCreatureType(CreatureType::Elf);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Thranduil has all activated abilities of all Elf cards in your graveyard.",
            effect: StaticEffect::HasActivatedAbilitiesOfYourGraveyardMatching { filter: elf() },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: elf().and(R::HasSupertype(Supertype::Legendary)),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        }],
        ..legend(
            "Thranduil, the Elvenking",
            cost(&[generic(2), b(), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            5,
            6,
        )
    }
}

/// Queza, Augur of Agonies — {1}{W}{U}{B} 3/4 Octopus Advisor. Whenever you
/// draw a card, target opponent loses 1 life and you gain 1 life.
pub fn queza_augur_of_agonies() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::Drain {
                from: target_filtered(R::Player.and(R::OpponentPlayer)),
                to: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..legend(
            "Queza, Augur of Agonies",
            cost(&[generic(1), w(), u(), b()]),
            vec![CreatureType::Octopus, CreatureType::Advisor],
            3,
            4,
        )
    }
}

/// Child of Alara — {W}{U}{B}{R}{G} 6/6 trample. When it dies, destroy all
/// nonland permanents; they can't be regenerated (CR 701.19c).
pub fn child_of_alara() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::DestroyNoRegen {
            what: Selector::EachPermanent(R::Nonland),
        })],
        ..legend("Child of Alara", cost(&[w(), u(), b(), r(), g()]), vec![CreatureType::Avatar], 6, 6)
    }
}

/// Liesa, Shroud of Dusk — {2}{W}{W}{B} 5/5 flying, lifelink. Whenever a
/// player casts a spell, they lose 2 life. (The commander-tax rider — pay 2
/// life per previous command-zone cast instead of {2} — is not modelled:
/// the tax is paid in mana, CR 903.8.)
pub fn liesa_shroud_of_dusk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(2) },
        }],
        ..legend(
            "Liesa, Shroud of Dusk",
            cost(&[generic(2), w(), w(), b()]),
            vec![CreatureType::Angel],
            5,
            5,
        )
    }
}

/// Urtet, Remnant of Memnarch — {3} 2/2 Legendary Artifact Creature — Myr.
/// Whenever you cast a Myr spell, create a 1/1 Myr artifact creature token.
/// At the beginning of combat on your turn, untap each Myr you control.
/// {W}{U}{B}{R}{G}, {T}: three +1/+1 counters on each Myr you control, only
/// during your turn.
pub fn urtet_remnant_of_memnarch() -> CardDefinition {
    use crate::game::types::TurnStep;
    let myr = || R::HasCreatureType(CreatureType::Myr);
    let my_myr = || Selector::EachPermanent(myr().and(R::ControlledByYou));
    let token = TokenDefinition {
        name: "Myr".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Myr], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(myr())),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(token) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::Untap { what: my_myr(), up_to: None },
            },
        ],
        activated_abilities: vec![crate::card::ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::AddCounter { what: my_myr(), kind: CounterType::PlusOnePlusOne, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..legend("Urtet, Remnant of Memnarch", cost(&[generic(3)]), vec![CreatureType::Myr], 2, 2)
    }
}

/// Isshin, Two Heavens as One — {R}{W}{B} 3/4 Human Samurai. If a creature
/// attacking causes a triggered ability of a permanent you control to
/// trigger, that ability triggers an additional time (CR 603.2d; Wulfgar's
/// `DoubleControllerAttackTriggers`).
pub fn isshin_two_heavens_as_one() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a creature attacking causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerAttackTriggers,
        }],
        ..legend(
            "Isshin, Two Heavens as One",
            cost(&[r(), w(), b()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            3,
            4,
        )
    }
}

/// Tergrid, God of Fright // Tergrid's Lantern — {3}{B}{B} 4/5 menace God.
/// Whenever an opponent sacrifices a nontoken permanent or discards a
/// permanent card, you may put that card from a graveyard onto the
/// battlefield under your control. The Lantern ({3}{B} legendary artifact):
/// {T}: target player loses 3 life unless they sacrifice a nonland permanent
/// or discard a card; {3}{B}: untap it.
pub fn tergrid_god_of_fright() -> CardDefinition {
    let steal = || Effect::MayDo {
        description: "Put that card onto the battlefield under your control?".into(),
        body: Box::new(Effect::Move {
            what: Selector::TriggerSource,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        }),
    };
    let lantern = CardDefinition {
        name: "Tergrid's Lantern",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            crate::card::ActivatedAbility {
                tap_cost: true,
                effect: Effect::Punisher {
                    chooser: target_filtered(R::Player),
                    options: vec![
                        Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Permanent.and(R::Nonland) },
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    ],
                    otherwise: Box::new(Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::Const(3),
                    }),
                },
                ..Default::default()
            },
            crate::card::ActivatedAbility {
                mana_cost: cost(&[generic(3), b()]),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::OpponentControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsToken.negate() },
                ),
                effect: steal(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Permanent },
                ),
                effect: steal(),
            },
        ],
        back_face: Some(Box::new(lantern)),
        ..legend("Tergrid, God of Fright", cost(&[generic(3), b(), b()]), vec![CreatureType::God], 4, 5)
    }
}

/// Najeela, the Blade-Blossom — {2}{R} 3/2 Human Warrior. Whenever a Warrior
/// attacks, you may have its controller create a 1/1 white Warrior token
/// tapped and attacking (CR 508.3a). {W}{U}{B}{R}{G}: untap all attacking
/// creatures; they gain trample, lifelink and haste until end of turn; after
/// this phase there is an additional combat phase (CR 505.1b). Activate
/// only during combat.
///
/// The "you may" is taken for your own Warriors and declined for an
/// opponent's (a token attacking beside their Warrior never helps you), so
/// the trigger is scoped to Warriors you control.
pub fn najeela_the_blade_blossom() -> CardDefinition {
    use crate::game::types::TurnStep;
    let warrior = TokenDefinition {
        name: "Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Warrior], ..Default::default() },
        ..Default::default()
    };
    let attackers = || Selector::EachPermanent(R::Creature.and(R::IsAttacking));
    let during_combat = Predicate::Any(
        [
            TurnStep::BeginCombat,
            TurnStep::DeclareAttackers,
            TurnStep::DeclareBlockers,
            TurnStep::FirstStrikeDamage,
            TurnStep::CombatDamage,
            TurnStep::EndCombat,
        ]
        .into_iter()
        .map(Predicate::CurrentStepIs)
        .collect(),
    );
    let grant = |keyword| Effect::GrantKeyword { what: attackers(), keyword, duration: Duration::EndOfTurn };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCreatureType(CreatureType::Warrior),
            }),
            effect: Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(warrior),
                cleanup: Default::default(),
                defender: Some(PlayerRef::DefendingPlayer),
            },
        }],
        activated_abilities: vec![crate::card::ActivatedAbility {
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            condition: Some(during_combat),
            effect: Effect::Seq(vec![
                Effect::Untap { what: attackers(), up_to: None },
                grant(Keyword::Trample),
                grant(Keyword::Lifelink),
                grant(Keyword::Haste),
                Effect::AdditionalCombatPhase { count: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Najeela, the Blade-Blossom",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            2,
        )
    }
}
