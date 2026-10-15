//! Commander: two-card gaps of the 2026-10-15 full-list EDHREC scan, third
//! pass. Tests in `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{AttackingTokenCleanup, Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r, u, w};
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

fn aura(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..Default::default()
    }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
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

fn mint(count: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(def) }
}

fn wall() -> TokenDefinition {
    TokenDefinition { keywords: vec![Keyword::Defender], ..token("Wall", None, vec![CreatureType::Wall], 0, 4) }
}

fn insect() -> TokenDefinition {
    token("Insect", Some(Color::Green), vec![CreatureType::Insect], 1, 1)
}

fn plus_one(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

// ── The Fantastic Four ─────────────────────────────────────────────────────

/// The Fantastic Four — {R}{G}{W}{U} legendary 4/4 Human Hero. When they enter
/// and whenever you cast a spell with power, toughness, or mana value 4,
/// choose one that hasn't been chosen this turn: a 0/4 Wall with defender; 3
/// damage to each opponent; two +1/+1 counters on target creature; draw.
pub fn the_fantastic_four() -> CardDefinition {
    let modes = || Effect::ChooseUnchosenModeThisTurn {
        modes: vec![
            mint(Value::ONE, wall()),
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) },
            plus_one(target_filtered(R::Creature), Value::Const(2)),
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ],
    };
    let exactly_four = R::ManaValueExactly(4)
        .or(R::PowerAtLeast(4).and(R::Not(Box::new(R::PowerAtLeast(5)))))
        .or(R::ToughnessAtLeast(4).and(R::Not(Box::new(R::ToughnessAtLeast(5)))));
    CardDefinition {
        triggered_abilities: vec![
            etb(modes()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: exactly_four }),
                effect: modes(),
            },
        ],
        ..legend(
            "The Fantastic Four",
            cost(&[r(), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Hero],
            4,
            4,
        )
    }
}

/// Wall Off — {3}{W} instant, {1} less per creature your opponents control: a
/// 0/4 Wall with defender, and you gain 4 life.
pub fn wall_off() -> CardDefinition {
    CardDefinition {
        name: "Wall Off",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each creature your opponents control.",
            effect: StaticEffect::SelfCostReducedPerPermanentMatching {
                filter: R::Creature.and(R::ControlledByOpponent),
                per: 1,
            },
        }],
        effect: Effect::Seq(vec![
            mint(Value::ONE, wall()),
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        ]),
        ..Default::default()
    }
}

// ── Zetalpa, Primal Dawn ───────────────────────────────────────────────────

/// Hedron Matrix — {4} Equipment. Equipped creature gets +X/+X, where X is its
/// mana value. Equip {4}.
pub fn hedron_matrix() -> CardDefinition {
    let mv = || Value::ManaValueOf(Box::new(enchanted()));
    CardDefinition {
        name: "Hedron Matrix",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        static_abilities: vec![StaticAbility {
            description: "Equipped creature gets +X/+X, where X is its mana value.",
            effect: StaticEffect::PumpPTByValue { applies_to: enchanted(), power: mv(), toughness: mv() },
        }],
        ..Default::default()
    }
}

/// Solid Footing — {W} Aura with flash. Enchanted creature gets +1/+1; while
/// it has vigilance, it assigns combat damage equal to its toughness.
pub fn solid_footing() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "As long as enchanted creature has vigilance, it assigns combat damage equal to its toughness rather than its power.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::EntityMatches { what: enchanted(), filter: R::HasKeyword(Keyword::Vigilance) },
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: enchanted(),
                    keyword: Keyword::AssignsCombatDamageByToughness,
                }),
            },
        }],
        ..aura("Solid Footing", cost(&[w()]))
    }
}

// ── Erestor of the Council ─────────────────────────────────────────────────

fn on_your_scry(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::Scried, EventScope::YourControl), effect }
}

/// Lost Isle Calling — {1}{U} enchantment. Whenever you scry, a verse counter
/// here. {4}{U}{U}, exile it: draw a card per verse counter; with seven or
/// more, take an extra turn after this one. Sorcery speed.
pub fn lost_isle_calling() -> CardDefinition {
    let verses = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Verse };
    CardDefinition {
        name: "Lost Isle Calling",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![on_your_scry(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Verse,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), u(), u()]),
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: verses() },
                Effect::If {
                    cond: Predicate::ValueAtLeast(verses(), Value::Const(7)),
                    then: Box::new(Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Council's Deliberation — {1}{U} instant: draw a card. Whenever you scry,
/// if you control an Island and this is in your graveyard, you may exile it;
/// if you do, draw a card.
pub fn councils_deliberation() -> CardDefinition {
    CardDefinition {
        name: "Council's Deliberation",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Scried, EventScope::FromYourGraveyard).with_filter(
                Predicate::SelectorCountAtLeast { sel: yours(R::HasLandType(LandType::Island)), n: Value::ONE },
            ),
            effect: Effect::MayDo {
                description: "Exile Council's Deliberation from your graveyard to draw a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ])),
            },
        }],
        ..Default::default()
    }
}

// ── The Astonishing Ant-Man ────────────────────────────────────────────────

/// The Astonishing Ant-Man — {G}{U} legendary 1/1 Human Scientist Hero.
/// Whenever you draw a card, a +1/+1 counter on him. {2}{G}, {T}, remove any
/// number of +1/+1 counters from him: that many 1/1 green Insects.
pub fn the_astonishing_ant_man() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: plus_one(Selector::This, Value::ONE),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            tap_cost: true,
            remove_counter_x: Some(CounterType::PlusOnePlusOne),
            effect: mint(Value::XFromCost, insect()),
            ..Default::default()
        }],
        ..legend(
            "The Astonishing Ant-Man",
            cost(&[g(), u()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Hero],
            1,
            1,
        )
    }
}

/// Ant-Man, Colony Commander — {1}{G}{U} legendary 2/2 Human Rogue Hero.
/// Attacking, you may pay {1}; when you do, a +1/+1 counter on target
/// creature. Whenever you put a +1/+1 counter on a creature, a 1/1 green
/// Insect — only once each turn.
pub fn ant_man_colony_commander() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::MayPay {
                description: "Pay {1} to put a +1/+1 counter on a creature?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::ReflexiveTrigger {
                    body: Box::new(plus_one(target_filtered(R::Creature), Value::ONE)),
                }),
                else_: None,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::YouPutCounters)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature })
                    .once_per_turn(),
                effect: mint(Value::ONE, insect()),
            },
        ],
        ..legend(
            "Ant-Man, Colony Commander",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Hero],
            2,
            2,
        )
    }
}

// ── Geist of Saint Traft ───────────────────────────────────────────────────

fn attack_with_flyer(name: &str, types: Vec<CreatureType>, cleanup: AttackingTokenCleanup) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::EnchantedBySource),
        effect: Effect::CreateTokenAttacking {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(TokenDefinition {
                keywords: vec![Keyword::Flying],
                ..token(name, Some(Color::White), types, 4, 4)
            }),
            cleanup,
            defender: None,
        },
    }
}

/// Dorothea, Vengeful Victim — {W}{U} legendary 4/4 Spirit; flying. When she
/// attacks or blocks, sacrifice her at end of combat. Disturb {1}{W}{U}: back
/// face Dorothea's Retribution, an Aura whose creature has "attacking, a
/// tapped and attacking 4/4 white flying Spirit, sacrificed at end of combat"
/// (exiled instead of going to a graveyard — the disturb rider).
pub fn dorothea_vengeful_victim() -> CardDefinition {
    let retribution = CardDefinition {
        color_indicator: vec![Color::White, Color::Blue],
        triggered_abilities: vec![attack_with_flyer(
            "Spirit",
            vec![CreatureType::Spirit],
            AttackingTokenCleanup::SacrificeAtEndOfCombat,
        )],
        ..aura("Dorothea's Retribution", ManaCost::default())
    };
    let sac = || Effect::SacrificeAtEndOfCombat { what: Selector::This };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Disturb(cost(&[generic(1), w(), u()]))],
        triggered_abilities: vec![
            on_attack(sac()),
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: sac() },
        ],
        back_face: Some(Box::new(retribution)),
        ..legend("Dorothea, Vengeful Victim", cost(&[w(), u()]), vec![CreatureType::Spirit], 4, 4)
    }
}

/// Invocation of Saint Traft — {1}{W}{U} Aura. Enchanted creature has
/// "attacking, a tapped and attacking 4/4 white flying Angel, exiled at end
/// of combat."
pub fn invocation_of_saint_traft() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![attack_with_flyer(
            "Angel",
            vec![CreatureType::Angel],
            AttackingTokenCleanup::ExileAtEndOfCombat,
        )],
        ..aura("Invocation of Saint Traft", cost(&[generic(1), w(), u()]))
    }
}

// ── Storm, Queen of Wakanda ────────────────────────────────────────────────

/// Jennifer Walters // The Sensational She-Hulk — {1}{W} legendary 2/3 Human
/// Advisor Hero. Your opponents can't cast spells during your turn.
/// {3}{G}{W}{W}: transform her (sorcery speed). Back: 6/6 Gamma Hero with
/// reach and trample, the same lock, and "whenever a creature you control is
/// dealt damage, she may deal that much damage to any target" once each turn.
pub fn jennifer_walters() -> CardDefinition {
    let lock = || StaticAbility {
        description: "Your opponents can't cast spells during your turn.",
        effect: StaticEffect::OpponentsCantCastDuringYourTurn,
    };
    let she_hulk = CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Trample],
        static_abilities: vec![lock()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature })
                .once_per_turn(),
            effect: Effect::MayDo {
                description: "Have The Sensational She-Hulk deal that much damage to any target?".into(),
                body: Box::new(Effect::DealDamage { to: target_filtered(R::any_target()), amount: Value::TriggerEventAmount }),
            },
        }],
        color_indicator: vec![Color::Green, Color::White],
        ..legend("The Sensational She-Hulk", ManaCost::default(), vec![CreatureType::Gamma, CreatureType::Hero], 6, 6)
    };
    CardDefinition {
        static_abilities: vec![lock()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g(), w(), w()]),
            sorcery_speed: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(she_hulk)),
        ..legend(
            "Jennifer Walters",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Advisor, CreatureType::Hero],
            2,
            3,
        )
    }
}

/// Storm, Windrider — {1}{G}{W}{W} legendary 4/4 Mutant Hero; flying.
/// Creatures with flying can't attack you or block creatures you control.
/// Whenever you cast a spell that targets one or more creatures, those
/// creatures gain flying until end of turn.
pub fn storm_windrider() -> CardDefinition {
    let flyers = || R::Creature.and(R::HasKeyword(Keyword::Flying));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures with flying can't attack you.",
                effect: StaticEffect::CreaturesCantAttackController { protect_planeswalkers: false, filter: Some(flyers()) },
            },
            StaticAbility {
                description: "Creatures with flying can't block creatures you control.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: yours(R::Creature),
                    keyword: Keyword::CantBeBlockedBy(Box::new(R::HasKeyword(Keyword::Flying))),
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellTargetsMatch(R::Creature)),
            effect: Effect::ForEach {
                selector: Selector::AllCastSpellTargets,
                body: Box::new(Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                    then: Box::new(Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::Flying,
                        duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        }],
        ..legend(
            "Storm, Windrider",
            cost(&[generic(1), g(), w(), w()]),
            vec![CreatureType::Mutant, CreatureType::Hero],
            4,
            4,
        )
    }
}

// ── Niambi, Esteemed Speaker ───────────────────────────────────────────────

/// Niambi, Esteemed Speaker — {W}{U} legendary 2/1 Human Cleric; flash. ETB:
/// you may return another target creature you control to its owner's hand;
/// if you do, gain life equal to its mana value. {1}{W}{U}, {T}, discard a
/// legendary card: draw two cards.
pub fn niambi_esteemed_speaker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                },
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ])),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w(), u()]),
            tap_cost: true,
            discard_cost: Some((R::HasSupertype(Supertype::Legendary), 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..legend("Niambi, Esteemed Speaker", cost(&[w(), u()]), vec![CreatureType::Human, CreatureType::Cleric], 2, 1)
    }
}

/// Linvala, Shield of Sea Gate — {1}{W}{U} legendary 3/3 Angel Wizard; flying.
/// At the beginning of combat on your turn, if you have a full party, target
/// nonland permanent an opponent controls can't attack or block and its
/// activated abilities can't be activated until your next turn. Sacrifice
/// Linvala: creatures you control gain your choice of hexproof or
/// indestructible until end of turn.
pub fn linvala_shield_of_sea_gate() -> CardDefinition {
    let lock = |k: Keyword| Effect::GrantKeyword { what: Selector::Target(0), keyword: k, duration: Duration::UntilNextTurn };
    let team = |k: Keyword| Effect::GrantKeyword { what: yours(R::Creature), keyword: k, duration: Duration::EndOfTurn };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(Value::PartyCount, Value::Const(4))),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
                    keyword: Keyword::CantAttack,
                    duration: Duration::UntilNextTurn,
                },
                lock(Keyword::CantBlock),
                lock(Keyword::CantActivateAbilities),
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::ChooseMode(vec![team(Keyword::Hexproof), team(Keyword::Indestructible)]),
            ..Default::default()
        }],
        ..legend(
            "Linvala, Shield of Sea Gate",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Angel, CreatureType::Wizard],
            3,
            3,
        )
    }
}
