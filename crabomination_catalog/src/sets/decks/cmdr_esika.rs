//! Commander: the cards the **From Cute to Brute** Secret Lair Commander
//! deck (SLD, Esika, God of the Tree) needed beyond what the catalog had —
//! almost all double-faced. Tests in `tests/recent_b/cmdr_esika.rs`.
//!
//! Residuals (each also on its card):
//! - **Archangel Avacyn** — transforms at *your* next upkeep, not the next
//!   upkeep of any player.
//! - **Arlinn, the Pack's Hope** — the +1's flash and extra +1/+1 counter
//!   last this turn, not until your next turn.
//! - **Azor's Gateway** — transforms off five cards exiled with it, not five
//!   different mana values; the exiled card is the engine's pick.
//! - **Chandra, Fire of Kaladesh** — flips when an opponent has lost 3 or
//!   more life this turn, not after Chandra herself dealt 3 damage.
//! - **Cosima, God of the Voyage** — the voyage ability isn't implemented.
//! - **Gideon, Battle-Forged** (Kytheon's back) — the +2 lure isn't
//!   implemented; the +1's indestructible lasts until end of turn; the 0
//!   doesn't prevent damage to him.
//! - **Jace, Telepath Unbound** — the +1 lasts until end of turn.
//! - **Journey to Eternity** — returns the creature, but not itself
//!   transformed.
//! - **Liliana, Defiant Necromancer** — the −8 emblem isn't implemented.
//! - **Ludevic, Necrogenius** — transforms for {U}{U}{B}{B} exiling one
//!   creature card; Olag is a plain 4/4 with counters, not a copy.
//! - **Nicol Bolas, the Arisen** — the −12 isn't implemented.
//! - **The Ringhart Crest** — its mana isn't restricted.
//! - **Tibalt, Cosmic Impostor** (Valki's back) — cards it exiles can't be
//!   played; the emblem isn't implemented.
//! - **Withengar Unbound** (Elbrus's back) — "whenever a player loses the
//!   game" isn't implemented.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, LoyaltyAbility,
    PlaneswalkerSubtype, SelectionRequirement as R, Selector, StateTriggeredAbility, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, LookPick, ManaPayload, PlayerRef, Predicate, RevealMissDest, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};
use crate::sets::{enters_tapped, tap_add, tap_add_any_color};
use crabomination_base::tokens::{clue_token, treasure_token};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

/// A transformed back face: no mana cost, a color indicator instead.
fn back(def: CardDefinition, colors: Vec<Color>) -> Box<CardDefinition> {
    Box::new(CardDefinition { color_indicator: colors, ..def })
}

fn walker(name: &'static str, mana: ManaCost, subtype: PlaneswalkerSubtype, loyalty: u32, abilities: Vec<LoyaltyAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![subtype], ..Default::default() },
        base_loyalty: loyalty,
        loyalty_abilities: abilities,
        ..Default::default()
    }
}

fn loyalty(cost: i32, effect: Effect) -> LoyaltyAbility {
    LoyaltyAbility { loyalty_cost: cost, effect, ..Default::default() }
}

fn land(name: &'static str, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition { name, supertypes: vec![Supertype::Legendary], card_types: vec![CardType::Land], activated_abilities: abilities, ..Default::default() }
}

fn token(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, p: i32, t: i32, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.to_string(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        keywords,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

fn make(who: PlayerRef, n: i32, def: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who, count: Value::Const(n), definition: def }
}

fn wolf() -> Arc<TokenDefinition> {
    token("Wolf", vec![Color::Green], vec![CreatureType::Wolf], 2, 2, vec![])
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn transform_self() -> Effect {
    Effect::Transform { what: Selector::This }
}

fn tap(effect: Effect) -> ActivatedAbility {
    ActivatedAbility { tap_cost: true, effect, ..Default::default() }
}

fn add_colorless() -> ActivatedAbility {
    tap(Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) })
}

fn snow_dual(name: &'static str, types: [LandType; 2], colors: [Color; 2]) -> CardDefinition {
    CardDefinition {
        name,
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: types.to_vec(), ..Default::default() },
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![tap_add(colors[0]), tap_add(colors[1])],
        ..Default::default()
    }
}

fn player_or_walker() -> Selector {
    target_filtered(R::Player.or(R::Planeswalker))
}

/// Archangel Avacyn // Avacyn, the Purifier — flash, flying, vigilance;
/// entering makes your creatures indestructible this turn; a non-Angel of
/// yours dying transforms her at the next upkeep, and the Purifier burns
/// everything else for 3.
///
/// ⚠ Residual: transforms at *your* next upkeep.
pub fn archangel_avacyn() -> CardDefinition {
    let purifier = CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)), amount: Value::Const(3) },
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) },
            ]),
        }],
        keywords: vec![Keyword::Flying],
        ..legendary(creature("Avacyn, the Purifier", ManaCost::default(), vec![CreatureType::Angel], 6, 5))
    };
    legendary(CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![
            etb(Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Angel).negate(),
                }),
                effect: Effect::AtYourNextUpkeep { body: Box::new(transform_self()) },
            },
        ],
        back_face: Some(back(purifier, vec![Color::Red])),
        ..creature("Archangel Avacyn", cost(&[generic(3), w(), w()]), vec![CreatureType::Angel], 4, 4)
    })
}

/// Arlinn Kord // Arlinn, Embraced by the Moon.
pub fn arlinn_kord() -> CardDefinition {
    let embraced = walker(
        "Arlinn, Embraced by the Moon",
        ManaCost::default(),
        PlaneswalkerSubtype::Arlinn,
        0,
        vec![
            loyalty(
                1,
                Effect::Seq(vec![
                    Effect::PumpPT { what: yours(R::Creature), power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                ]),
            ),
            loyalty(-1, Effect::Seq(vec![Effect::DealDamage { to: target_any(), amount: Value::Const(3) }, transform_self()])),
            loyalty(
                -6,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Arlinn, Embraced by the Moon".into(),
                    triggered: vec![],
                    statics: vec![
                        StaticAbility {
                            description: "Creatures you control have haste.",
                            effect: StaticEffect::GrantKeyword { applies_to: yours(R::Creature), keyword: Keyword::Haste },
                        },
                        StaticAbility {
                            description: "Creatures you control have \"{T}: This creature deals damage equal to its power to any target.\"",
                            effect: StaticEffect::GrantActivatedAbility {
                                applies_to: yours(R::Creature),
                                ability: tap(Effect::DealDamage { to: target_any(), amount: Value::PowerOf(Box::new(Selector::This)) }),
                                condition: None,
                            },
                        },
                    ],
                },
            ),
        ],
    );
    CardDefinition {
        back_face: Some(back(embraced, vec![Color::Red, Color::Green])),
        ..walker(
            "Arlinn Kord",
            cost(&[generic(2), r(), g()]),
            PlaneswalkerSubtype::Arlinn,
            3,
            vec![
                loyalty(
                    1,
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: R::Creature,
                        effect: Box::new(Effect::Seq(vec![
                            Effect::PumpPT { what: Selector::Target(0), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
                            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                        ])),
                    },
                ),
                loyalty(0, Effect::Seq(vec![make(PlayerRef::You, 1, wolf()), transform_self()])),
            ],
        )
    }
}

/// Arlinn, the Pack's Hope // Arlinn, the Moon's Fury — daybound; −3: two
/// Wolves. Night: +2: {R}{G}; 0: a 5/5 trampling, indestructible, hasty
/// Werewolf.
///
/// ⚠ Residual: the +1's flash and extra counter last this turn, not until
/// your next turn.
pub fn arlinn_the_packs_hope() -> CardDefinition {
    let fury = CardDefinition {
        keywords: vec![Keyword::Nightbound],
        ..walker(
            "Arlinn, the Moon's Fury",
            ManaCost::default(),
            PlaneswalkerSubtype::Arlinn,
            4,
            vec![
                loyalty(2, Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red, Color::Green]) }),
                loyalty(
                    0,
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(5),
                        toughness: Value::Const(5),
                        creature_types: vec![CreatureType::Werewolf],
                        keywords: vec![Keyword::Trample, Keyword::Indestructible, Keyword::Haste],
                        duration: Duration::EndOfTurn,
                    },
                ),
            ],
        )
    };
    CardDefinition {
        keywords: vec![Keyword::Daybound],
        back_face: Some(back(fury, vec![Color::Red, Color::Green])),
        ..walker(
            "Arlinn, the Pack's Hope",
            cost(&[generic(2), r(), g()]),
            PlaneswalkerSubtype::Arlinn,
            4,
            vec![
                loyalty(
                    1,
                    Effect::Seq(vec![
                        Effect::GrantCreatureSpellsFlashThisTurn { who: PlayerRef::You },
                        Effect::CreaturesEnterWithExtraCounterThisTurn { who: PlayerRef::You },
                    ]),
                ),
                loyalty(-3, make(PlayerRef::You, 2, wolf())),
            ],
        )
    }
}

/// Azor's Gateway // Sanctum of the Sun — {1}, {T}: draw, then exile a card
/// from hand; with enough exiled, gain 5, untap, transform. The Sanctum taps
/// for mana equal to your life.
///
/// ⚠ Residual: counts five cards exiled with it, not five mana values; the
/// exiled card is the engine's pick.
pub fn azors_gateway() -> CardDefinition {
    let sanctum = land(
        "Sanctum of the Sun",
        vec![tap(Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::LifeOf(PlayerRef::You)) })],
    );
    CardDefinition {
        name: "Azor's Gateway",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::ExileLinked {
                    what: Selector::Take {
                        inner: Box::new(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Hand, filter: R::Any }),
                        count: Box::new(Value::ONE),
                    },
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::CardsExiledWithSourceCount, Value::Const(5)),
                    then: Box::new(Effect::Seq(vec![
                        Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
                        Effect::Untap { what: Selector::This, up_to: None },
                        transform_self(),
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        back_face: Some(Box::new(sanctum)),
        ..Default::default()
    }
}

/// Chandra, Fire of Kaladesh // Chandra, Roaring Flame.
///
/// ⚠ Residual: flips when an opponent has lost 3+ life this turn, not after
/// Chandra herself dealt 3 damage.
pub fn chandra_fire_of_kaladesh() -> CardDefinition {
    let flame = walker(
        "Chandra, Roaring Flame",
        ManaCost::default(),
        PlaneswalkerSubtype::Chandra,
        4,
        vec![
            loyalty(1, Effect::DealDamage { to: player_or_walker(), amount: Value::Const(2) }),
            loyalty(-2, Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(2) }),
            loyalty(
                -7,
                Effect::Seq(vec![
                    Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(6) },
                    Effect::CreateEmblem {
                        who: PlayerRef::EachOpponent,
                        name: "Chandra, Roaring Flame".into(),
                        triggered: vec![TriggeredAbility {
                            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::You), amount: Value::Const(3) },
                        }],
                        statics: vec![],
                    },
                ]),
            ),
        ],
    );
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::HasColor(Color::Red))),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        activated_abilities: vec![tap(Effect::Seq(vec![
            Effect::DealDamage { to: player_or_walker(), amount: Value::ONE },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::LifeLostThisTurn(PlayerRef::EachOpponent), Value::Const(3)),
                then: Box::new(Effect::ExileSelfReturnTransformed),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        back_face: Some(back(flame, vec![Color::Red])),
        ..creature(
            "Chandra, Fire of Kaladesh",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            2,
            2,
        )
    })
}

/// Cosima, God of the Voyage // The Omenkeel.
///
/// ⚠ Residual: the voyage ability isn't implemented.
pub fn cosima_god_of_the_voyage() -> CardDefinition {
    let omenkeel = CardDefinition {
        name: "The Omenkeel",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasArtifactSubtype(ArtifactSubtype::Vehicle) },
            ),
            effect: Effect::ExileTopOfLibrary {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::TriggerEventAmount,
                link_to_source: true,
                face_down: false,
            },
        }],
        ..Default::default()
    };
    legendary(CardDefinition {
        back_face: Some(Box::new(omenkeel)),
        ..creature("Cosima, God of the Voyage", cost(&[generic(2), u()]), vec![CreatureType::God], 2, 4)
    })
}

/// Dennick, Pious Apprentice // Dennick, Pious Apparition — lifelink; cards
/// in graveyards can't be targeted; disturb {2}{W}{U}; the Apparition flies
/// and investigates once a turn when creature cards hit graveyards.
pub fn dennick_pious_apprentice() -> CardDefinition {
    let apparition = CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).once_per_turn(),
            effect: make(PlayerRef::You, 1, Arc::new(clue_token())),
        }],
        ..legendary(creature(
            "Dennick, Pious Apparition",
            ManaCost::default(),
            vec![CreatureType::Spirit, CreatureType::Soldier],
            3,
            2,
        ))
    };
    legendary(CardDefinition {
        keywords: vec![Keyword::Lifelink, Keyword::Disturb(cost(&[generic(2), w(), u()]))],
        static_abilities: vec![StaticAbility {
            description: "Cards in graveyards can't be the targets of spells or abilities.",
            effect: StaticEffect::GraveyardCardsUntargetable,
        }],
        back_face: Some(back(apparition, vec![Color::White, Color::Blue])),
        ..creature(
            "Dennick, Pious Apprentice",
            cost(&[w(), u()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            3,
        )
    })
}

/// Dowsing Dagger // Lost Vale — an opponent gets two 0/2 defender Plants;
/// equipped creature gets +2/+1 and connecting may transform it into a land
/// that taps for three mana of one color; equip {2}.
pub fn dowsing_dagger() -> CardDefinition {
    let vale = CardDefinition {
        supertypes: vec![],
        ..land("Lost Vale", vec![tap(Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(3)) })])
    };
    CardDefinition {
        name: "Dowsing Dagger",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(make(
            PlayerRef::Target(0),
            2,
            token("Plant", vec![Color::Green], vec![CreatureType::Plant], 0, 2, vec![Keyword::Defender]),
        ))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 1,
            triggers_on_equipment: true,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo { description: "Transform Dowsing Dagger?".into(), body: Box::new(transform_self()) },
            }],
            ..Default::default()
        }),
        back_face: Some(Box::new(vale)),
        ..Default::default()
    }
}

/// Elbrus, the Binding Blade // Withengar Unbound — +1/+0; connecting
/// unattaches and transforms it into a 13/13.
///
/// ⚠ Residual: Withengar's "whenever a player loses the game" isn't
/// implemented.
pub fn elbrus_the_binding_blade() -> CardDefinition {
    let withengar = CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Intimidate, Keyword::Trample],
        ..legendary(creature("Withengar Unbound", ManaCost::default(), vec![CreatureType::Demon], 13, 13))
    };
    CardDefinition {
        name: "Elbrus, the Binding Blade",
        cost: cost(&[generic(7)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            triggers_on_equipment: true,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![Effect::Unattach { what: Selector::This }, transform_self()]),
            }],
            ..Default::default()
        }),
        back_face: Some(back(withengar, vec![Color::Black])),
        ..Default::default()
    }
}

/// Esika, God of the Tree // The Prismatic Bridge.
pub fn esika_god_of_the_tree() -> CardDefinition {
    let bridge = CardDefinition {
        name: "The Prismatic Bridge",
        cost: cost(&[w(), u(), b(), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: R::Creature.or(R::Planeswalker),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                cap: Value::Const(100),
                life_per_revealed: 0,
                miss_dest: RevealMissDest::BottomRandom,
            },
        }],
        ..Default::default()
    };
    legendary(CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![tap_add_any_color()],
        static_abilities: vec![
            StaticAbility {
                description: "Other legendary creatures you control have vigilance.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: yours(R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::OtherThanSource)),
                    keyword: Keyword::Vigilance,
                },
            },
            StaticAbility {
                description: "Other legendary creatures you control have \"{T}: Add one mana of any color.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: yours(R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::OtherThanSource)),
                    ability: tap_add_any_color(),
                    condition: None,
                },
            },
        ],
        back_face: Some(Box::new(bridge)),
        ..creature("Esika, God of the Tree", cost(&[generic(1), g(), g()]), vec![CreatureType::God], 1, 4)
    })
}

/// Garruk Relentless // Garruk, the Veil-Cursed.
pub fn garruk_relentless() -> CardDefinition {
    let cursed = walker(
        "Garruk, the Veil-Cursed",
        ManaCost::default(),
        PlaneswalkerSubtype::Garruk,
        0,
        vec![
            loyalty(1, make(PlayerRef::You, 1, token("Wolf", vec![Color::Black], vec![CreatureType::Wolf], 1, 1, vec![Keyword::Deathtouch]))),
            loyalty(
                -1,
                Effect::Seq(vec![
                    Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature },
                    Effect::Search { who: PlayerRef::You, filter: R::Creature, to: ZoneDest::Hand(PlayerRef::You) },
                ]),
            ),
            loyalty(-3, {
                let n = || Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature };
                Effect::Seq(vec![
                    Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                    Effect::PumpPT { what: yours(R::Creature), power: n(), toughness: n(), duration: Duration::EndOfTurn },
                ])
            }),
        ],
    );
    CardDefinition {
        state_trigger: Some(StateTriggeredAbility {
            condition: Predicate::Not(Box::new(Predicate::SourceHasCountersAtLeast { counter: CounterType::Loyalty, n: 3 })),
            effect: transform_self(),
        }),
        back_face: Some(back(cursed, vec![Color::Black, Color::Green])),
        ..walker(
            "Garruk Relentless",
            cost(&[generic(3), g()]),
            PlaneswalkerSubtype::Garruk,
            3,
            vec![
                loyalty(
                    0,
                    Effect::Seq(vec![
                        Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(3) },
                        Effect::DealDamageFrom {
                            source: Selector::Target(0),
                            to: Selector::This,
                            amount: Value::PowerOf(Box::new(Selector::Target(0))),
                        },
                    ]),
                ),
                loyalty(0, make(PlayerRef::You, 1, wolf())),
            ],
        )
    }
}

/// Hadana's Climb // Winged Temple of Orazca.
pub fn hadanas_climb() -> CardDefinition {
    let temple = land(
        "Winged Temple of Orazca",
        vec![
            tap_add_any_color(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), g(), u()]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(R::Creature.and(R::ControlledByYou)),
                        keyword: Keyword::Flying,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::PumpPT {
                        what: Selector::Target(0),
                        power: Value::PowerOf(Box::new(Selector::Target(0))),
                        toughness: Value::PowerOf(Box::new(Selector::Target(0))),
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
    );
    CardDefinition {
        name: "Hadana's Climb",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn { what: Box::new(Selector::Target(0)), kind: CounterType::PlusOnePlusOne },
                        Value::Const(3),
                    ),
                    then: Box::new(transform_self()),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        back_face: Some(Box::new(temple)),
        ..Default::default()
    }
}

/// Jace, Vryn's Prodigy // Jace, Telepath Unbound.
///
/// ⚠ Residual: the +1 lasts until end of turn.
pub fn jace_vryns_prodigy() -> CardDefinition {
    let unbound = walker(
        "Jace, Telepath Unbound",
        ManaCost::default(),
        PlaneswalkerSubtype::Jace,
        5,
        vec![
            loyalty(
                1,
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::PumpPT {
                        what: Selector::Target(0),
                        power: Value::Const(-2),
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    }),
                },
            ),
            loyalty(
                -3,
                Effect::GrantMayPlay {
                    what: target_filtered(
                        R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::InYourGraveyard),
                    ),
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: true,
                    pay_own_cost: true,
                    any_color: false,
                },
            ),
            loyalty(
                -9,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Jace, Telepath Unbound".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                        effect: Effect::Mill { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(5) },
                    }],
                    statics: vec![],
                },
            ),
        ],
    );
    legendary(CardDefinition {
        activated_abilities: vec![tap(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::GraveyardSizeOf(PlayerRef::You), Value::Const(5)),
                then: Box::new(Effect::ExileSelfReturnTransformed),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        back_face: Some(back(unbound, vec![Color::Blue])),
        ..creature("Jace, Vryn's Prodigy", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Wizard], 0, 2)
    })
}

/// Journey to Eternity // Atzal, Cave of Eternity — enchanted creature of
/// yours dying comes back.
///
/// ⚠ Residual: the Aura doesn't return transformed.
pub fn journey_to_eternity() -> CardDefinition {
    let atzal = land(
        "Atzal, Cave of Eternity",
        vec![
            tap_add_any_color(),
            ActivatedAbility {
                mana_cost: cost(&[generic(3), b(), g()]),
                tap_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
    );
    CardDefinition {
        name: "Journey to Eternity",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false } },
            }],
            ..Default::default()
        }),
        back_face: Some(Box::new(atzal)),
        ..Default::default()
    }
}

/// Kinnan, Bonder Prodigy — nonland mana permanents make an extra mana;
/// {5}{G}{U}: put a non-Human creature from the top five onto the
/// battlefield.
pub fn kinnan_bonder_prodigy() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Nonland }),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyTypeTriggerSourceProduces },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g(), u()]),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(5),
                pick_filter: Some(R::Creature.and(R::HasCreatureType(CreatureType::Human).negate())),
                to_battlefield: true,
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..creature("Kinnan, Bonder Prodigy", cost(&[g(), u()]), vec![CreatureType::Human, CreatureType::Druid], 2, 2)
    })
}

/// Kolvori, God of Kinship // The Ringhart Crest.
///
/// ⚠ Residual: the Crest's mana isn't restricted.
pub fn kolvori_god_of_kinship() -> CardDefinition {
    let crest = CardDefinition {
        name: "The Ringhart Crest",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![tap_add(Color::Green)],
        ..Default::default()
    };
    let three_legends = || Predicate::SelectorCountAtLeast {
        sel: yours(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
        n: Value::Const(3),
    };
    legendary(CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "As long as you control three or more legendary creatures, Kolvori gets +4/+2.",
                effect: StaticEffect::WhileCondition {
                    condition: three_legends(),
                    inner: Box::new(StaticEffect::PumpPT { applies_to: Selector::This, power: 4, toughness: 2 }),
                },
            },
            StaticAbility {
                description: "As long as you control three or more legendary creatures, Kolvori has vigilance.",
                effect: StaticEffect::WhileCondition {
                    condition: three_legends(),
                    inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Vigilance }),
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            tap_cost: true,
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(6),
                pick_filter: Some(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
                ..Default::default()
            })),
            ..Default::default()
        }],
        back_face: Some(Box::new(crest)),
        ..creature("Kolvori, God of Kinship", cost(&[generic(2), g(), g()]), vec![CreatureType::God], 2, 4)
    })
}

/// Kytheon, Hero of Akros // Gideon, Battle-Forged.
///
/// ⚠ Residual: Gideon's +2 lure isn't implemented; the +1's indestructible
/// lasts until end of turn; his 0 doesn't prevent damage to him.
pub fn kytheon_hero_of_akros() -> CardDefinition {
    let gideon = walker(
        "Gideon, Battle-Forged",
        ManaCost::default(),
        PlaneswalkerSubtype::Gideon,
        3,
        vec![
            loyalty(2, Effect::Noop),
            loyalty(
                1,
                Effect::Seq(vec![
                    Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
                    Effect::Untap { what: Selector::Target(0), up_to: None },
                ]),
            ),
            loyalty(
                0,
                Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    creature_types: vec![CreatureType::Human, CreatureType::Soldier],
                    keywords: vec![Keyword::Indestructible],
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
    );
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::EndCombat), EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::This, filter: R::IsAttacking },
                Predicate::AttackedWithCountAtLeast { who: PlayerRef::You, at_least: 3 },
            ])),
            effect: Effect::ExileSelfReturnTransformed,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        back_face: Some(back(gideon, vec![Color::White])),
        ..creature("Kytheon, Hero of Akros", cost(&[w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 1)
    })
}

/// Liliana, Heretical Healer // Liliana, Defiant Necromancer.
///
/// ⚠ Residual: the −8 emblem isn't implemented.
pub fn liliana_heretical_healer() -> CardDefinition {
    let necromancer = walker(
        "Liliana, Defiant Necromancer",
        ManaCost::default(),
        PlaneswalkerSubtype::Liliana,
        3,
        vec![
            loyalty(2, Effect::Discard { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE, random: false }),
            LoyaltyAbility {
                loyalty_cost: 0,
                x_cost: true,
                effect: Effect::Move {
                    what: target_filtered(
                        R::Creature
                            .and(R::HasSupertype(Supertype::Legendary).negate())
                            .and(R::ManaValueExactlyXFromCost)
                            .and(R::InYourGraveyard),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            },
        ],
    );
    legendary(CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken }),
            effect: Effect::Seq(vec![
                Effect::ExileSelfReturnTransformed,
                make(PlayerRef::You, 1, token("Zombie", vec![Color::Black], vec![CreatureType::Zombie], 2, 2, vec![])),
            ]),
        }],
        back_face: Some(back(necromancer, vec![Color::Black])),
        ..creature(
            "Liliana, Heretical Healer",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            3,
        )
    })
}

/// Ludevic, Necrogenius // Olag, Ludevic's Hubris.
///
/// ⚠ Residual: transforms for {U}{U}{B}{B} exiling one creature card; Olag
/// is a 4/4 with counters, not a copy.
pub fn ludevic_necrogenius() -> CardDefinition {
    let olag = CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legendary(creature("Olag, Ludevic's Hubris", ManaCost::default(), vec![CreatureType::Zombie], 4, 4))
    };
    let mill = || Effect::Mill { who: Selector::You, amount: Value::ONE };
    legendary(CardDefinition {
        triggered_abilities: vec![etb(mill()), on_attack(mill())],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u(), b(), b()]),
            sorcery_speed: true,
            exile_other_filter: Some((R::Creature.and(R::InYourGraveyard), 1)),
            effect: transform_self(),
            ..Default::default()
        }],
        back_face: Some(back(olag, vec![Color::Blue, Color::Black])),
        ..creature("Ludevic, Necrogenius", cost(&[u(), b()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 3)
    })
}

/// Nicol Bolas, the Ravager // Nicol Bolas, the Arisen.
///
/// ⚠ Residual: the −12 isn't implemented.
pub fn nicol_bolas_the_ravager() -> CardDefinition {
    let arisen = walker(
        "Nicol Bolas, the Arisen",
        ManaCost::default(),
        PlaneswalkerSubtype::Bolas,
        7,
        vec![
            loyalty(2, Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
            loyalty(-3, Effect::DealDamage { to: target_filtered(R::Creature.or(R::Planeswalker)), amount: Value::Const(10) }),
            loyalty(
                -4,
                Effect::Move {
                    what: target_filtered(R::Creature.or(R::Planeswalker).and(R::InGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ),
        ],
    );
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
            random: false,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), u(), b(), r()]),
            sorcery_speed: true,
            effect: Effect::ExileSelfReturnTransformed,
            ..Default::default()
        }],
        back_face: Some(back(arisen, vec![Color::Blue, Color::Black, Color::Red])),
        ..creature(
            "Nicol Bolas, the Ravager",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Elder, CreatureType::Dragon],
            4,
            4,
        )
    })
}

/// Nissa, Vastwood Seer // Nissa, Sage Animist.
pub fn nissa_vastwood_seer() -> CardDefinition {
    let animist = walker(
        "Nissa, Sage Animist",
        ManaCost::default(),
        PlaneswalkerSubtype::Nissa,
        3,
        vec![
            loyalty(
                1,
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::MatchingAmong {
                        inner: Box::new(Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE }),
                        filter: R::Land,
                    }),
                    then: Box::new(Effect::Move {
                        what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    else_: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            ),
            loyalty(
                -2,
                make(
                    PlayerRef::You,
                    1,
                    Arc::new(TokenDefinition {
                        supertypes: vec![Supertype::Legendary],
                        ..(*token("Ashaya, the Awoken World", vec![Color::Green], vec![CreatureType::Elemental], 4, 4, vec![])).clone()
                    }),
                ),
            ),
            loyalty(
                -7,
                Effect::Seq(vec![
                    Effect::Untap { what: yours(R::Land), up_to: Some(Value::Const(6)) },
                    Effect::BecomeCreature {
                        what: Selector::Take { inner: Box::new(yours(R::Land)), count: Box::new(Value::Const(6)) },
                        power: Value::Const(6),
                        toughness: Value::Const(6),
                        creature_types: vec![CreatureType::Elemental],
                        keywords: vec![],
                        duration: Duration::Permanent,
                    },
                ]),
            ),
        ],
    );
    legendary(CardDefinition {
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Search for a basic Forest card?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand.and(R::HasLandType(LandType::Forest)),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl).with_filter(Predicate::SelectorCountAtLeast {
                    sel: yours(R::Land),
                    n: Value::Const(7),
                }),
                effect: Effect::ExileSelfReturnTransformed,
            },
        ],
        back_face: Some(back(animist, vec![Color::Green])),
        ..creature("Nissa, Vastwood Seer", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Scout], 2, 2)
    })
}

/// Rimewood Falls — Snow Land — Forest Island, enters tapped.
pub fn rimewood_falls() -> CardDefinition {
    snow_dual("Rimewood Falls", [LandType::Forest, LandType::Island], [Color::Green, Color::Blue])
}

/// Sisay, Weatherlight Captain — +1/+1 per color among your other
/// legendary permanents; {W}{U}{B}{R}{G}: fetch a legendary permanent with
/// mana value less than its power.
pub fn sisay_weatherlight_captain() -> CardDefinition {
    let colors = || {
        Value::DistinctColorsAmong(Box::new(yours(R::Permanent.and(R::HasSupertype(Supertype::Legendary)).and(R::OtherThanSource))))
    };
    legendary(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Sisay gets +1/+1 for each color among other legendary permanents you control.",
            effect: StaticEffect::PumpSelfByValue { amount: colors(), per_power: 1, per_toughness: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::PermanentCard.and(R::HasSupertype(Supertype::Legendary)).and(R::ManaValueLessThanSourcePower),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Sisay, Weatherlight Captain",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    })
}

/// Thaumatic Compass // Spires of Orazca.
pub fn thaumatic_compass() -> CardDefinition {
    let spires = CardDefinition {
        supertypes: vec![],
        ..land(
            "Spires of Orazca",
            vec![
                add_colorless(),
                tap(Effect::Seq(vec![
                    Effect::Untap { what: target_filtered(R::Creature.and(R::IsAttacking).and(R::ControlledByOpponent)), up_to: None },
                    Effect::RemoveFromCombat { what: Selector::Target(0) },
                ])),
            ],
        )
    };
    CardDefinition {
        name: "Thaumatic Compass",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Search { who: PlayerRef::You, filter: R::IsBasicLand, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::SelectorCountAtLeast { sel: yours(R::Land), n: Value::Const(7) }),
            effect: transform_self(),
        }],
        back_face: Some(Box::new(spires)),
        ..Default::default()
    }
}

/// Treasure Map // Treasure Cove.
pub fn treasure_map() -> CardDefinition {
    let cove = CardDefinition {
        supertypes: vec![],
        ..land(
            "Treasure Cove",
            vec![
                add_colorless(),
                ActivatedAbility {
                    tap_cost: true,
                    sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Treasure), 1)),
                    effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ..Default::default()
                },
            ],
        )
    };
    CardDefinition {
        name: "Treasure Map",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                Effect::AddCounter { what: Selector::This, kind: CounterType::Landmark, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Landmark, n: 3 },
                    then: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter { what: Selector::This, kind: CounterType::Landmark, amount: Value::Const(3) },
                        transform_self(),
                        make(PlayerRef::You, 3, Arc::new(treasure_token())),
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        back_face: Some(Box::new(cove)),
        ..Default::default()
    }
}

/// Valki, God of Lies // Tibalt, Cosmic Impostor.
///
/// ⚠ Residual: Tibalt's exiled cards can't be played; the emblem isn't
/// implemented.
pub fn valki_god_of_lies() -> CardDefinition {
    let tibalt = walker(
        "Tibalt, Cosmic Impostor",
        cost(&[generic(5), b(), r()]),
        PlaneswalkerSubtype::Tibalt,
        5,
        vec![
            loyalty(
                2,
                Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                    link_to_source: true,
                    face_down: false,
                },
            ),
            loyalty(-3, Effect::Move { what: target_filtered(R::Artifact.or(R::Creature)), to: ZoneDest::Exile }),
            loyalty(
                -8,
                Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::EachMatching {
                            zone: crate::effect::ZoneRef::Graveyard(PlayerRef::EachPlayer),
                            filter: R::Any,
                        },
                        to: ZoneDest::Exile,
                    },
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red, Color::Red, Color::Red]) },
                ]),
            ),
        ],
    );
    legendary(CardDefinition {
        triggered_abilities: vec![etb(Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachOpponent),
            body: Box::new(Effect::ExileUntilSourceLeaves {
                what: Selector::Take {
                    inner: Box::new(Selector::CardsInZone { who: PlayerRef::Triggerer, zone: Zone::Hand, filter: R::Creature }),
                    count: Box::new(Value::ONE),
                },
                return_to: crate::card::ExileReturnZone::Hand,
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            effect: Effect::BecomeCopyOfExiledCard {
                what: Selector::MatchingAmong {
                    inner: Box::new(Selector::CardExiledWithSource),
                    filter: R::ManaValueExactlyXFromCost,
                },
                base_pt: None,
            },
            ..Default::default()
        }],
        back_face: Some(Box::new(tibalt)),
        ..creature("Valki, God of Lies", cost(&[generic(1), b()]), vec![CreatureType::God], 2, 1)
    })
}

/// Voldaren Pariah // Abolisher of Bloodlines — flying; madness {B}{B}{B};
/// sacrifice three other creatures: transform, and the Abolisher makes an
/// opponent sacrifice three.
pub fn voldaren_pariah() -> CardDefinition {
    let abolisher = CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource),
            effect: Effect::Sacrifice { who: Selector::Player(PlayerRef::Target(0)), count: Value::Const(3), filter: R::Creature },
        }],
        ..creature("Abolisher of Bloodlines", ManaCost::default(), vec![CreatureType::Eldrazi, CreatureType::Vampire], 6, 5)
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Madness(cost(&[b(), b(), b()]))],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature.and(R::OtherThanSource), 3)),
            effect: transform_self(),
            ..Default::default()
        }],
        back_face: Some(back(abolisher, vec![Color::Black])),
        ..creature(
            "Voldaren Pariah",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Horror],
            3,
            3,
        )
    }
}

/// Woodland Chasm — Snow Land — Swamp Forest, enters tapped.
pub fn woodland_chasm() -> CardDefinition {
    snow_dual("Woodland Chasm", [LandType::Swamp, LandType::Forest], [Color::Black, Color::Green])
}
