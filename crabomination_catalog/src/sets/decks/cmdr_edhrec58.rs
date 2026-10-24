//! Commander: EDHREC ranks 2001-2500, one-card gaps (fourth batch) — Jaya
//! Ballard, Teyo's Sigarda deck, Black Widow, Danitha, Kang, Felidar Umbra's
//! Tiana, Dream Salvage's Gwendlyn, Halana (Kogla and Alena // Halana), Bower
//! Passage's Ludevic // Sidar Kondo, Idyllic Grange's Nils, Dragonlord
//! Silumgar, Maximus, Zuko Conflicted, Jaws of Stone's Ben-Ben. Tests in
//! `tests/recent_b/cmdr_edhrec58.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};

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

/// Jaya Ballard, Task Mage — {1}{R}{R} legendary 2/2 Human Spellshaper. Each
/// ability costs {T} and a discard: {R} destroys target blue permanent;
/// {1}{R} deals 3 damage to any target (a creature dealt damage this way
/// can't be regenerated this turn); {5}{R}{R} deals 6 to each creature and
/// each player.
pub fn jaya_ballard_task_mage() -> CardDefinition {
    let shaper = |mana: ManaCost, effect: Effect| ActivatedAbility {
        mana_cost: mana,
        tap_cost: true,
        discard_cost: Some((R::Any, 1)),
        effect,
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            shaper(cost(&[r()]), Effect::Destroy { what: target_filtered(R::Permanent.and(R::HasColor(Color::Blue))) }),
            shaper(
                cost(&[generic(1), r()]),
                Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::any_target()), amount: Value::Const(3) },
                    Effect::GrantDamageDeniesRegenerationThisTurn { what: Selector::Target(0) },
                ]),
            ),
            shaper(
                cost(&[generic(5), r(), r()]),
                Effect::Seq(vec![
                    Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(6) },
                    Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(6) },
                ]),
            ),
        ],
        ..legend(
            "Jaya Ballard, Task Mage",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            2,
            2,
        )
    }
}

/// Teyo, Lightshield Expert — {1}{W} legendary 1/1 Human Cleric, flash. Enters:
/// target permanent you control gains hexproof until end of turn, with a
/// +1/+1 counter if it's a creature and a loyalty counter if it's a
/// planeswalker.
pub fn teyo_lightshield_expert() -> CardDefinition {
    let is = |filter: R| Predicate::EntityMatches { what: Selector::Target(0), filter };
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: is(R::Creature),
                then: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::If {
                cond: is(R::Planeswalker),
                then: Box::new(Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Loyalty, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..legend("Teyo, Lightshield Expert", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 1)
    }
}

/// Black Widow, Intel Expert — {2}{W} legendary 3/3 Human Spy Hero, first
/// strike. Combat damage to an opponent: you and that player each draw two.
pub fn black_widow_intel_expert() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::Draw { who: Selector::Player(PlayerRef::TriggerEventPlayer), amount: Value::Const(2) },
            ]),
        }],
        ..legend(
            "Black Widow, Intel Expert",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Spy, CreatureType::Hero],
            3,
            3,
        )
    }
}

/// Danitha, Sword of Hope — {2}{W} legendary 2/2 Human Knight, first strike.
/// Whenever you cast an Equipment spell or a spell that targets a creature
/// you control, draw a card; once each turn.
pub fn danitha_sword_of_hope() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::Any(vec![
                    Predicate::CastSpellMatches(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment)),
                    Predicate::CastSpellTargetsMatch(R::Creature.and(R::ControlledByYou)),
                ]))
                .once_per_turn(),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legend("Danitha, Sword of Hope", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

/// Kang the Conqueror — {2}{U}{U} legendary 4/5 Human Villain, flying.
/// Power-up — {5}{U}{U}{U}: a +1/+1 counter on Kang; take an extra turn after
/// this one (CR 702.193); during that turn, power-up abilities can't be
/// activated.
pub fn kang_the_conqueror() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u(), u(), u()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
                Effect::PowerUpLockedDuringExtraTurn { who: PlayerRef::You },
            ]),
            ..Default::default()
        }],
        ..legend("Kang the Conqueror", cost(&[generic(2), u(), u()]), vec![CreatureType::Human, CreatureType::Villain], 4, 5)
    }
}

/// Felidar Umbra — {1}{W} Aura. Enchanted creature has lifelink. {1}{W}:
/// attach it to target creature you control. Umbra armor (CR 702.89).
pub fn felidar_umbra() -> CardDefinition {
    CardDefinition {
        name: "Felidar Umbra",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        keywords: vec![Keyword::UmbraArmor],
        equipped_bonus: Some(crate::card::EquipBonus { keywords: vec![Keyword::Lifelink], ..Default::default() }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dream Salvage — {U/B} instant. Draw cards equal to the number of cards
/// target opponent discarded this turn.
pub fn dream_salvage() -> CardDefinition {
    CardDefinition {
        name: "Dream Salvage",
        cost: cost(&[crate::mana::hybrid(Color::Blue, Color::Black)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisTurn(PlayerRef::Target(0)) },
        ..Default::default()
    }
}

/// Halana, Kessig Ranger — {3}{G} legendary 3/4 Human Archer Ranger, reach,
/// partner. Whenever another creature you control enters, you may pay {2};
/// when you do, that creature deals damage equal to its power to target
/// creature.
pub fn halana_kessig_ranger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::OtherThanSource) },
            ),
            effect: Effect::MayPay {
                description: "Pay {2} to have it deal damage equal to its power to target creature?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::ReflexiveTrigger {
                    body: Box::new(Effect::DealDamageEqualToPower {
                        source: Selector::TriggerSource,
                        target: target_filtered(R::Creature),
                    }),
                }),
                else_: None,
            },
        }],
        ..legend(
            "Halana, Kessig Ranger",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Archer, CreatureType::Ranger],
            3,
            4,
        )
    }
}

/// Bower Passage — {1}{G} enchantment. Creatures with flying can't block
/// creatures you control.
pub fn bower_passage() -> CardDefinition {
    CardDefinition {
        name: "Bower Passage",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures with flying can't block creatures you control.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::CantBeBlockedBy(Box::new(R::HasKeyword(Keyword::Flying))),
            },
        }],
        ..Default::default()
    }
}

/// Idyllic Grange — land — Plains. Enters tapped unless you control three or
/// more other Plains. When it enters untapped, put a +1/+1 counter on target
/// creature you control.
pub fn idyllic_grange() -> CardDefinition {
    let plains = || R::HasLandType(LandType::Plains).and(R::ControlledByYou).and(R::OtherThanSource);
    CardDefinition {
        name: "Idyllic Grange",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Plains], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless you control three or more other Plains.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::ValueAtLeast(Value::count(Selector::EachPermanent(plains())), Value::Const(3)),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::Untapped }),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![crate::sets::tap_add(Color::White)],
        ..Default::default()
    }
}

/// Dragonlord Silumgar — {4}{U}{B} legendary 3/5 Elder Dragon, flying,
/// deathtouch. Enters: gain control of target creature or planeswalker for as
/// long as you control Silumgar.
pub fn dragonlord_silumgar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::GainControlWhileYouControlSource {
            what: target_filtered(R::Creature.or(R::Planeswalker)),
        })],
        ..legend(
            "Dragonlord Silumgar",
            cost(&[generic(4), u(), b()]),
            vec![CreatureType::Elder, CreatureType::Dragon],
            3,
            5,
        )
    }
}

/// Maximus, Knight Apparent — {3}{R} legendary 4/4 Human Knight, trample.
/// Enters: you may search for an Equipment card with mana value 2, reveal it,
/// put it into your hand. {1}, sacrifice an artifact: you get {E}{E}.
pub fn maximus_knight_apparent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment)
                .and(R::ManaValueAtLeast(2))
                .and(R::ManaValueAtMost(2)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::AddEnergy(Value::Const(2)),
            ..Default::default()
        }],
        ..legend("Maximus, Knight Apparent", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Knight], 4, 4)
    }
}

/// Zuko, Conflicted — {B}{R} legendary 2/3 Human Rogue. At the beginning of
/// your first main phase, choose one that hasn't been chosen and you lose 2
/// life: draw a card; a +1/+1 counter on Zuko; add {R}; or exile Zuko, then
/// return him under an opponent's control.
pub fn zuko_conflicted() -> CardDefinition {
    let pay = |e: Effect| Effect::Seq(vec![e, Effect::LoseLife { who: Selector::You, amount: Value::Const(2) }]);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
            effect: Effect::ChooseUnchosenMode {
                modes: vec![
                    pay(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    pay(Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
                    pay(Effect::AddMana { who: PlayerRef::You, pool: crate::effect::ManaPayload::Colors(vec![Color::Red]) }),
                    pay(Effect::Seq(vec![
                        Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                        Effect::Move {
                            what: Selector::LastMoved,
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::OpponentOf(Box::new(PlayerRef::You)),
                                tapped: false,
                            },
                        },
                    ])),
                ],
            },
        }],
        ..legend("Zuko, Conflicted", cost(&[b(), r()]), vec![CreatureType::Human, CreatureType::Rogue], 2, 3)
    }
}

/// Jaws of Stone — {5}{R} sorcery. X damage divided as you choose among any
/// number of targets, X the number of Mountains you control as you cast it.
pub fn jaws_of_stone() -> CardDefinition {
    CardDefinition {
        name: "Jaws of Stone",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamageDivided {
            total: Value::count(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Mountain),
            }),
            filter: R::Any,
            max_targets: 8,
            retaliate_to_source: false,
        },
        ..Default::default()
    }
}
