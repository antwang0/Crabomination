//! Commander: two-card gaps of the 2026-10-15 full-list EDHREC scan. Tests
//! in `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, MayPlayDuration, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, investigate, on_dies, target_any, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn plus_one(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

fn entering(filter: R, scope: EventScope) -> EventSpec {
    EventSpec::new(EventKind::EntersBattlefield, scope)
        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter })
}

/// "Return up to one target [filter] to its owner's hand."
fn bounce_up_to_one(filter: R) -> Effect {
    Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::Move { what: target_filtered(filter), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
    }
}

// ── Slinza, the Spiked Stampede ────────────────────────────────────────────

/// Slinza, the Spiked Stampede — {4}{G} legendary 5/5 Beast. Beast spells you
/// cast cost {2} less; each other Beast you control enters with an extra
/// +1/+1 counter. Whenever Slinza or another creature with power 4 or greater
/// enters, you may pay {1}{R/G}; when you do, Slinza fights target creature
/// you don't control.
pub fn slinza_the_spiked_stampede() -> CardDefinition {
    let beast = || R::HasCreatureType(CreatureType::Beast);
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Beast spells you cast cost {2} less to cast.",
                effect: StaticEffect::CostReduction { filter: beast(), amount: 2 },
            },
            StaticAbility {
                description: "Each other Beast creature you control enters with an additional +1/+1 counter on it.",
                effect: StaticEffect::MatchingEntersWithExtraCounters {
                    filter: beast().and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                    kind: CounterType::PlusOnePlusOne,
                    amount: 1,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            // Any creature's entry, not only yours: the printed trigger has no
            // controller clause. Slinza herself is a 5/5, so the filter covers her.
            event: entering(R::Creature.and(R::PowerAtLeast(4)), EventScope::AnyPlayer),
            effect: Effect::MayPay {
                description: "Pay {1}{R/G} to have Slinza fight?".into(),
                mana_cost: cost(&[generic(1), hybrid(Color::Red, Color::Green)]),
                body: Box::new(Effect::ReflexiveTrigger {
                    body: Box::new(Effect::Fight {
                        attacker: Selector::This,
                        defender: target_filtered(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
                    }),
                }),
                else_: None,
            },
        }],
        ..legend("Slinza, the Spiked Stampede", cost(&[generic(4), g()]), vec![CreatureType::Beast], 5, 5)
    }
}

/// Woodland Liege — {2}{G} 2/2 Elf Druid Noble. Whenever a Beast you control
/// enters, draw a card.
pub fn woodland_liege() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: entering(R::HasCreatureType(CreatureType::Beast), EventScope::YourControl),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Woodland Liege",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Druid, CreatureType::Noble],
            2,
            2,
        )
    }
}

// ── Zagras, Thief of Heartbeats ────────────────────────────────────────────

/// Zagras, Thief of Heartbeats — {4}{B}{R} legendary 4/4 Vampire Rogue; costs
/// {1} less per creature in your party. Flying, deathtouch, haste. Other
/// creatures you control have deathtouch. Whenever a creature you control
/// deals combat damage to a planeswalker, destroy that planeswalker.
pub fn zagras_thief_of_heartbeats() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Haste],
        static_abilities: vec![
            StaticAbility {
                description: "This spell costs {1} less to cast for each creature in your party.",
                effect: StaticEffect::SelfCostReducedByValue { amount: Value::PartyCount },
            },
            StaticAbility {
                description: "Other creatures you control have deathtouch.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: yours(R::Creature.and(R::OtherThanSource)),
                    keyword: Keyword::Deathtouch,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            // The damaged planeswalker is bound to target slot 0.
            event: EventSpec::new(EventKind::DealsCombatDamageToPlaneswalker, EventScope::YourControl),
            effect: Effect::Destroy { what: Selector::Target(0) },
        }],
        ..legend(
            "Zagras, Thief of Heartbeats",
            cost(&[generic(4), b(), r()]),
            vec![CreatureType::Vampire, CreatureType::Rogue],
            4,
            4,
        )
    }
}

/// Power of Fire — {1}{R} Aura. Enchanted creature has "{T}: This creature
/// deals 1 damage to any target."
pub fn power_of_fire() -> CardDefinition {
    CardDefinition {
        name: "Power of Fire",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has \"{T}: This creature deals 1 damage to any target.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

// ── Feroz, Ulgrotha's Warden ───────────────────────────────────────────────

/// Feroz, Ulgrotha's Warden — {3}{G}{W}{U} legendary planeswalker, loyalty 5;
/// can be your commander. +2: two 1/1 blue flying Birds. 0: draw a card, then
/// you may put a permanent card with mana value 4 or less from your hand onto
/// the battlefield. −7: a Feroz's Ban token ("Creature spells cost {2} more
/// to cast"). The token's printed {6} isn't modelled — tokens carry no mana
/// cost here, so it reads mana value 0.
pub fn feroz_ulgrothas_warden() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Bird".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        ..Default::default()
    };
    let ban = TokenDefinition {
        name: "Feroz's Ban".into(),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Creature spells cost {2} more to cast.",
            effect: StaticEffect::AdditionalCost { filter: R::Creature, amount: 2 },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Feroz, Ulgrotha's Warden",
        cost: cost(&[generic(3), g(), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Feroz], ..Default::default() },
        base_loyalty: 5,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(bird) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: R::PermanentCard.and(R::ManaValueAtMost(4)),
                        count: Value::ONE,
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                        return_eot: false,
                        then: None,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(ban) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Oath of Ajani — {G}{W} legendary enchantment. ETB: a +1/+1 counter on each
/// creature you control. Planeswalker spells you cast cost {1} less.
pub fn oath_of_ajani() -> CardDefinition {
    CardDefinition {
        name: "Oath of Ajani",
        cost: cost(&[g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(plus_one(yours(R::Creature), Value::ONE))],
        static_abilities: vec![StaticAbility {
            description: "Planeswalker spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Planeswalker, amount: 1 },
        }],
        ..Default::default()
    }
}

// ── The Spot, Living Portal ────────────────────────────────────────────────

/// The Spot, Living Portal — {3}{W}{B} legendary 4/4 Human Scientist Villain.
/// ETB: exile up to one target nonland permanent and up to one target nonland
/// permanent card from a graveyard. When he dies, put him on the bottom of
/// his owner's library; if you do, return the cards exiled with him to their
/// owners' hands.
pub fn the_spot_living_portal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::ExileLinked { what: target_filtered(R::Permanent.and(R::Nonland)) },
                    Effect::ExileLinked {
                        what: Selector::TargetFiltered {
                            slot: 1,
                            filter: R::PermanentCard.and(R::Nonland).and(R::InGraveyard),
                        },
                    },
                ])),
            }),
            on_dies(Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::Move { what: Selector::This, to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Bottom } },
                // "If you do": a commander sent to the command zone instead
                // never reaches the library, and the cards stay exiled.
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::count(Selector::LastMoved), Value::ONE),
                    then: Box::new(Effect::Move { what: Selector::CardExiledWithSource, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        ],
        ..legend(
            "The Spot, Living Portal",
            cost(&[generic(3), w(), b()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Villain],
            4,
            4,
        )
    }
}

/// The Spot's Portal — {2}{B} instant. Put target creature on the bottom of
/// its owner's library. You lose 2 life unless you control a Villain.
pub fn the_spots_portal() -> CardDefinition {
    CardDefinition {
        name: "The Spot's Portal",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Creature), to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Bottom } },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: yours(R::HasCreatureType(CreatureType::Villain)),
                    n: Value::ONE,
                },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(2) }),
            },
        ]),
        ..Default::default()
    }
}

// ── Jarvis, Earth's Mightiest Butler ───────────────────────────────────────

/// Justice, Vance Astrovik — {2}{U} legendary 2/2 Mutant Hero; flying. ETB:
/// return up to one target nonland, nontoken permanent to its owner's hand.
/// Whenever another nonland permanent you control is returned to its owner's
/// hand, a +1/+1 counter on Justice.
pub fn justice_vance_astrovik() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(bounce_up_to_one(R::Permanent.and(R::Nonland).and(R::Not(Box::new(R::IsToken))))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentReturnedToHand, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Nonland.and(R::OtherThanSource) },
                ),
                effect: plus_one(Selector::This, Value::ONE),
            },
        ],
        ..legend("Justice, Vance Astrovik", cost(&[generic(2), u()]), vec![CreatureType::Mutant, CreatureType::Hero], 2, 2)
    }
}

/// Spider-Man 2099, Miguel O'Hara — {4}{U} legendary 4/4 Spider Human Hero.
/// ETB: return up to one target creature to its owner's hand. Whenever one or
/// more creatures you control deal combat damage to a player, draw a card.
pub fn spider_man_2099_miguel_ohara() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(bounce_up_to_one(R::Creature)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch(),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..legend(
            "Spider-Man 2099, Miguel O'Hara",
            cost(&[generic(4), u()]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            4,
            4,
        )
    }
}

// ── Suki, Courageous Rescuer ───────────────────────────────────────────────

/// Agent 13, Sharon Carter — {2}{W} legendary 3/2 Human Spy Hero. Whenever a
/// creature you control attacks alone, investigate.
pub fn agent_13_sharon_carter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::AttackingAlone),
            effect: investigate(1),
        }],
        ..legend(
            "Agent 13, Sharon Carter",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Spy, CreatureType::Hero],
            3,
            2,
        )
    }
}

/// Political Triumph — {W} enchantment — Plan. Whenever a creature you control
/// enters, scry 1 and put a plan counter here. When the fourth plan counter is
/// put on it, sacrifice it, draw a card, and put a +1/+1 counter on each
/// creature you control.
pub fn political_triumph() -> CardDefinition {
    CardDefinition {
        name: "Political Triumph",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Plan], ..Default::default() },
        triggered_abilities: vec![
            TriggeredAbility {
                event: entering(R::Creature, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Plan, amount: Value::ONE },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::Plan), EventScope::SelfSource)
                    .with_filter(Predicate::SourceHasCountersAtLeast { counter: CounterType::Plan, n: 4 }),
                effect: Effect::Seq(vec![
                    Effect::SacrificeSource,
                    // Only the fourth counter triggers (CR 603.2): a fifth
                    // landing before this resolves finds it already gone.
                    Effect::If {
                        cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                        then: Box::new(Effect::Seq(vec![
                            Effect::Draw { who: Selector::You, amount: Value::ONE },
                            plus_one(yours(R::Creature), Value::ONE),
                        ])),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

// ── Azula, On the Hunt ─────────────────────────────────────────────────────

/// Fire Nation Sentinels — {3}{B}{B} 4/4 Human Soldier. Whenever a nontoken
/// creature an opponent controls dies, a +1/+1 counter on each creature you
/// control.
pub fn fire_nation_sentinels() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Not(Box::new(R::IsToken)) },
            ),
            effect: plus_one(yours(R::Creature), Value::ONE),
        }],
        ..creature(
            "Fire Nation Sentinels",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Fire Nation Occupation — {2}{B} enchantment. ETB, and whenever you cast a
/// spell during an opponent's turn: a 2/2 red Soldier with firebending 1.
pub fn fire_nation_occupation() -> CardDefinition {
    let soldier = Arc::new(TokenDefinition {
        name: "Soldier".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Firebending(1)],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    });
    let mint = || Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: soldier.clone() };
    CardDefinition {
        name: "Fire Nation Occupation",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(mint()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
                effect: mint(),
            },
        ],
        ..Default::default()
    }
}

// ── Prince Imrahil the Fair ────────────────────────────────────────────────

/// Bard the Bowman — {1}{W}{U} legendary 1/3 Human Archer; reach. Whenever you
/// draw your second card each turn, a +1/+1 counter on target creature; it
/// gains lifelink until end of turn.
pub fn bard_the_bowman() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
            effect: Effect::Seq(vec![
                plus_one(target_filtered(R::Creature), Value::ONE),
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
            ]),
        }],
        ..legend("Bard the Bowman", cost(&[generic(1), w(), u()]), vec![CreatureType::Human, CreatureType::Archer], 1, 3)
    }
}

/// Raff, Weatherlight Stalwart — {W}{U} legendary 1/3 Human Wizard. Whenever
/// you cast an instant or sorcery spell, you may tap two untapped creatures
/// you control; if you do, draw a card. {3}{W}{W}: creatures you control get
/// +1/+1 and gain vigilance until end of turn.
pub fn raff_weatherlight_stalwart() -> CardDefinition {
    let untapped = || R::Creature.and(R::Untapped).and(R::ControlledByYou);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            }),
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast { sel: Selector::EachPermanent(untapped()), n: Value::Const(2) },
                then: Box::new(Effect::MayDo {
                    description: "Tap two untapped creatures you control to draw a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::TapUpToValue { count: Value::Const(2), filter: untapped(), skip_untap: false, exact: true },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w(), w()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: yours(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Raff, Weatherlight Stalwart",
            cost(&[w(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            3,
        )
    }
}

// ── Captain Ripley Vance ───────────────────────────────────────────────────

/// Leering Emblem — {2} Equipment. Whenever you cast a spell, equipped
/// creature gets +2/+2 until end of turn. Equip {2}.
pub fn leering_emblem() -> CardDefinition {
    CardDefinition {
        name: "Leering Emblem",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Vance's Blasting Cannons // Spitfire Bastion — {3}{R} legendary
/// enchantment. Upkeep: exile the top card of your library; if it's nonland,
/// you may cast it this turn. Whenever you cast your third spell in a turn,
/// you may transform it. Back: legendary land, {T}: add {R}; {2}{R}, {T}: 3
/// damage to any target.
pub fn vances_blasting_cannons() -> CardDefinition {
    let bastion = CardDefinition {
        name: "Spitfire Bastion",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::catalog::sets::tap_add(Color::Red),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                tap_cost: true,
                effect: Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    CardDefinition {
        name: "Vance's Blasting Cannons",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::ClearLastMoved,
                    Effect::ExileTopOfLibrary {
                        who: Selector::You,
                        amount: Value::ONE,
                        link_to_source: false,
                        face_down: false,
                    },
                    Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::LastMoved, filter: R::Nonland },
                        then: Box::new(Effect::GrantMayPlay {
                            what: Selector::LastMoved,
                            duration: MayPlayDuration::EndOfThisTurn,
                            to_owner: false,
                            exile_after: false,
                            pay_own_cost: true,
                            any_color: false,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::Const(3) },
                ),
                effect: Effect::MayDo {
                    description: "Transform Vance's Blasting Cannons?".into(),
                    body: Box::new(Effect::Transform { what: Selector::This }),
                },
            },
        ],
        back_face: Some(Box::new(bastion)),
        ..Default::default()
    }
}
