//! Commander: the 2026-10-13 rescan's two-card gaps, third pass — Blor,
//! Phlage, Minsc, Tetsuko, Commander Mustard, Kassandra, Ashling the Pilgrim,
//! Parnesse and friends (`cmdr_edhrec25` is the previous batch). Tests in
//! `tests/recent_b/cmdr_edhrec26.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, x, Color, ManaCost};
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

fn on(kind: EventKind, scope: EventScope, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(kind, scope), effect }
}

fn source_is(kind: EventKind, scope: EventScope, filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(kind, scope)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect,
    }
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

fn mint(definition: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn soldier() -> R {
    R::HasCreatureType(CreatureType::Soldier)
}

// ── Blor the Impervious ──────────────────────────────────────────────────────

/// Blor the Impervious — {3}{G}{G} 3/3 legendary Frog Warrior, trample,
/// hexproof, indestructible.
pub fn blor_the_impervious() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Hexproof, Keyword::Indestructible],
        ..legend("Blor the Impervious", cost(&[generic(3), g(), g()]), vec![CreatureType::Frog, CreatureType::Warrior], 3, 3)
    }
}

/// Necklace of Girion — {2}{G} legendary artifact. A green spell cast, or a
/// Forest entering under your control, puts a +1/+1 counter on target
/// creature you control. {T}: add {G}.
pub fn necklace_of_girion() -> CardDefinition {
    let grow = || Effect::AddCounter {
        what: target_filtered(R::Creature.and(R::ControlledByYou)),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    CardDefinition {
        name: "Necklace of Girion",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            source_is(EventKind::SpellCast, EventScope::YourControl, R::HasColor(Color::Green), grow()),
            source_is(EventKind::EntersBattlefield, EventScope::YourControl, R::HasLandType(LandType::Forest), grow()),
        ],
        activated_abilities: vec![crate::sets::tap_add(Color::Green)],
        ..Default::default()
    }
}

// ── Phlage, Titan of Fire's Fury ─────────────────────────────────────────────

/// Bishop of Rebirth — {3}{W}{W} 3/4 Vampire Cleric, vigilance. Whenever it
/// attacks, you may return target creature card with mana value 3 or less
/// from your graveyard to the battlefield.
pub fn bishop_of_rebirth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_attack(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::ManaValueAtMost(3)).from_your_graveyard()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        })],
        ..creature("Bishop of Rebirth", cost(&[generic(3), w(), w()]), vec![CreatureType::Vampire, CreatureType::Cleric], 3, 4)
    }
}

/// Brought Back — {W}{W} instant. Return up to two target permanent cards in
/// your graveyard that went there from the battlefield this turn, tapped.
pub fn brought_back() -> CardDefinition {
    CardDefinition {
        name: "Brought Back",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::PermanentCard.and(R::PutIntoGraveyardFromBattlefieldThisTurn).from_your_graveyard(),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
        },
        ..Default::default()
    }
}

// ── Minsc, Beloved Ranger ────────────────────────────────────────────────────

/// Minsc, Beloved Ranger — {R}{G}{W} 3/3 legendary Human Ranger. ETB: Boo, a
/// legendary 1/1 red Hamster with trample and haste. {X}: target creature you
/// control has base P/T X/X and is a Giant too until end of turn (sorcery
/// speed).
pub fn minsc_beloved_ranger() -> CardDefinition {
    let boo = Arc::new(TokenDefinition {
        supertypes: vec![Supertype::Legendary],
        ..(*token("Boo", 1, 1, vec![Color::Red], vec![CreatureType::Hamster], vec![Keyword::Trample, Keyword::Haste])).clone()
    });
    let mine = || target_filtered(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        triggered_abilities: vec![etb(mint(boo))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::SetBasePT {
                    what: mine(),
                    power: Value::XFromCost,
                    toughness: Value::XFromCost,
                    duration: Duration::EndOfTurn,
                },
                Effect::AddCreatureTypes {
                    what: Selector::Target(0),
                    creature_types: vec![CreatureType::Giant],
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..legend("Minsc, Beloved Ranger", cost(&[r(), g(), w()]), vec![CreatureType::Human, CreatureType::Ranger], 3, 3)
    }
}

/// Arena Rector — {3}{W} 1/2 Human Cleric. When it dies, you may exile it; if
/// you do, put a planeswalker card from your library onto the battlefield.
pub fn arena_rector() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on(
            EventKind::CreatureDied,
            EventScope::SelfSource,
            Effect::MayDo {
                description: "Exile Arena Rector to fetch a planeswalker?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: R::Planeswalker,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                ])),
            },
        )],
        ..creature("Arena Rector", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 2)
    }
}

// ── Tetsuko Umezawa, Fugitive ────────────────────────────────────────────────

/// Eternal of Harsh Truths — {2}{U} 1/3 Zombie Cleric. Afflict 2 (CR
/// 702.130: becoming blocked costs the defending player 2 life). Attacking
/// unblocked draws a card.
pub fn eternal_of_harsh_truths() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on(
                EventKind::BecomesBlocked,
                EventScope::SelfSource,
                Effect::LoseLife { who: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::Const(2) },
            ),
            on(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource, draw(1)),
        ],
        ..creature("Eternal of Harsh Truths", cost(&[generic(2), u()]), vec![CreatureType::Zombie, CreatureType::Cleric], 1, 3)
    }
}

/// Surrakar Spellblade — {1}{U}{U} 2/1 Surrakar. Casting an instant or
/// sorcery may add a charge counter; its combat damage to a player may draw
/// one card per charge counter.
pub fn surrakar_spellblade() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            source_is(
                EventKind::SpellCast,
                EventScope::YourControl,
                R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                Effect::MayDo {
                    description: "Put a charge counter on Surrakar Spellblade?".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::ONE,
                    }),
                },
            ),
            on(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
                Effect::MayDo {
                    description: "Draw a card per charge counter?".into(),
                    body: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
                    }),
                },
            ),
        ],
        ..creature("Surrakar Spellblade", cost(&[generic(1), u(), u()]), vec![CreatureType::Surrakar], 2, 1)
    }
}

// ── Sarulf, Realm Eater (Oblivion's Hunger) ──────────────────────────────────

/// Oblivion's Hunger — {1}{B} instant. Target creature you control gains
/// indestructible until end of turn; draw a card if it has a +1/+1 counter.
pub fn oblivions_hunger() -> CardDefinition {
    CardDefinition {
        name: "Oblivion's Hunger",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CountersOn { what: Box::new(Selector::Target(0)), kind: CounterType::PlusOnePlusOne },
                    Value::ONE,
                ),
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ── Commander Mustard ────────────────────────────────────────────────────────

/// Commander Mustard — {3}{R}{W} 5/5 legendary Human Soldier, vigilance.
/// Other Soldiers you control have vigilance, trample and haste. {2}{R}{W}:
/// until end of turn, Soldiers you control gain "Whenever this creature
/// attacks, it deals 1 damage to defending player."
pub fn commander_mustard() -> CardDefinition {
    let others = || yours(soldier().and(R::OtherThanSource));
    let lord = |keyword| StaticAbility {
        description: "Other Soldiers you control have vigilance, trample, and haste.",
        effect: StaticEffect::GrantKeyword { applies_to: others(), keyword },
    };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![lord(Keyword::Vigilance), lord(Keyword::Trample), lord(Keyword::Haste)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r(), w()]),
            effect: Effect::ForEach {
                selector: yours(soldier()),
                body: Box::new(Effect::GrantTriggeredAbility {
                    what: Selector::TriggerSource,
                    trigger: Box::new(on_attack(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::DefendingPlayer),
                        amount: Value::ONE,
                    })),
                    duration: Duration::EndOfTurn,
                }),
            },
            ..Default::default()
        }],
        ..legend("Commander Mustard", cost(&[generic(3), r(), w()]), vec![CreatureType::Human, CreatureType::Soldier], 5, 5)
    }
}

/// Baird, Argivian Recruiter — {R}{W} 2/2 legendary Human Soldier. At your
/// end step, with a creature whose power exceeds its base power, a 1/1 white
/// Soldier.
pub fn baird_argivian_recruiter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                Predicate::SelectorExists(yours(R::Creature.and(R::PowerGreaterThanBasePower))),
            ),
            effect: mint(token("Soldier", 1, 1, vec![Color::White], vec![CreatureType::Soldier], vec![])),
        }],
        ..legend("Baird, Argivian Recruiter", cost(&[r(), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

// ── Kassandra, Eagle Bearer ──────────────────────────────────────────────────

/// Kassandra, Eagle Bearer — {1}{R}{W} 2/2 legendary Human Assassin Warrior,
/// haste. ETB: The Spear of Leonidas from your graveyard, hand or library
/// onto the battlefield. A creature of yours with a legendary Equipment
/// attached hitting a player draws a card.
pub fn kassandra_eagle_bearer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            etb(Effect::SearchZones {
                who: PlayerRef::You,
                zones: vec![Zone::Graveyard, Zone::Hand, Zone::Library],
                filter: R::HasName("The Spear of Leonidas".into()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .dealt_by(R::EquippedByMatching(Box::new(R::HasSupertype(Supertype::Legendary)))),
                effect: draw(1),
            },
        ],
        ..legend(
            "Kassandra, Eagle Bearer",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Eivor, Battle-Ready — {3}{R}{W} 5/5 legendary Human Assassin Warrior,
/// vigilance, haste. Attacking, it deals damage equal to your Equipment count
/// to each opponent.
pub fn eivor_battle_ready() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::count(yours(R::HasArtifactSubtype(ArtifactSubtype::Equipment))),
        })],
        ..legend(
            "Eivor, Battle-Ready",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Warrior],
            5,
            5,
        )
    }
}

// ── Hidetsugu, Devouring Chaos / Acererak (Cabal Paladin) ────────────────────

/// Shadow of Mortality — {13}{B}{B} 7/7 Avatar. Costs {X} less, X = how far
/// your life is below your starting life total.
pub fn shadow_of_mortality() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If your life total is less than your starting life total, this spell costs {X} less to cast, where X is the difference.",
            effect: StaticEffect::SelfCostReducedByValue {
                amount: Value::Max(
                    Box::new(Value::Const(0)),
                    Box::new(Value::Sum(vec![
                        Value::StartingLifeTotal,
                        Value::Negate(Box::new(Value::LifeOf(PlayerRef::You))),
                    ])),
                ),
            },
        }],
        ..creature("Shadow of Mortality", cost(&[generic(13), b(), b()]), vec![CreatureType::Avatar], 7, 7)
    }
}

/// Cabal Paladin — {3}{B} 4/2 Human Knight. Casting a historic spell deals 2
/// damage to each opponent.
pub fn cabal_paladin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::historic())),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
        }],
        ..creature("Cabal Paladin", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Knight], 4, 2)
    }
}

// ── Svella / Neyith / Mr. Orfeo pieces ───────────────────────────────────────

/// Rime Tender — {1}{G} 2/2 snow Human Druid. {T}: untap another target snow
/// permanent.
pub fn rime_tender() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Untap {
                what: target_filtered(R::HasSupertype(Supertype::Snow).and(R::OtherThanSource)),
                up_to: None,
            },
            ..Default::default()
        }],
        ..creature("Rime Tender", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Druid], 2, 2)
    }
}

/// Freelance Muscle — {4}{G} 4/4 Rhino Warrior. Attacking or blocking, it
/// gets +X/+X, X the greatest power or toughness among your other creatures.
pub fn freelance_muscle() -> CardDefinition {
    let others = || yours(R::Creature.and(R::OtherThanSource));
    let x = || Value::GreatestPowerOrToughnessAmong(Box::new(others()));
    let pump = || Effect::PumpPT { what: Selector::This, power: x(), toughness: x(), duration: Duration::EndOfTurn };
    CardDefinition {
        triggered_abilities: vec![on_attack(pump()), on(EventKind::Blocks, EventScope::SelfSource, pump())],
        ..creature("Freelance Muscle", cost(&[generic(4), g()]), vec![CreatureType::Rhino, CreatureType::Warrior], 4, 4)
    }
}

/// Hexhaven Invigorator — {G}{G}{G}{G} 6/6 Chimera Horror, vigilance. When
/// it's dealt damage, you may search for up to that many lands, tapped.
pub fn hexhaven_invigorator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on(
            EventKind::DealtDamage,
            EventScope::SelfSource,
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::Land,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::TriggerEventAmount,
            },
        )],
        ..creature("Hexhaven Invigorator", cost(&[g(), g(), g(), g()]), vec![CreatureType::Chimera, CreatureType::Horror], 6, 6)
    }
}

// ── Ashling the Pilgrim / Parnesse ───────────────────────────────────────────

/// Magebane Armor — {3} Equipment. +2/+4 and loses flying; noncombat damage
/// to it is prevented. Equip {2}.
pub fn magebane_armor() -> CardDefinition {
    CardDefinition {
        name: "Magebane Armor",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 4,
            remove_keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Prevent all noncombat damage that would be dealt to equipped creature.",
            effect: StaticEffect::PreventNoncombatDamageToMatching { filter: R::IsHostOfSource },
        }],
        ..Default::default()
    }
}

/// Dual Casting — {1}{R} Aura. Enchanted creature has "{R}, {T}: copy target
/// instant or sorcery spell you control; you may choose new targets."
pub fn dual_casting() -> CardDefinition {
    CardDefinition {
        name: "Dual Casting",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[r()]),
                tap_cost: true,
                effect: Effect::CopySpellMayChooseTargets {
                    what: target_filtered(
                        R::IsSpellOnStack
                            .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))
                            .and(R::ControlledByYou),
                    ),
                    count: Value::ONE,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}
