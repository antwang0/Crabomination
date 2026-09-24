//! Commander: the cards the **20 Ways to Win** Secret Lair deck (SLD,
//! Go-Shintai of Life's Origin) needed beyond what the catalog had (Tragic
//! Arrogance is Silverquill Statement's, `cmdr_breena.rs`). Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{draw, etb, mint_treasures, target_filtered};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, ExtraManaKind, ManaPayload, PlayerRef, Predicate,
    ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
use crate::sets::{enters_tapped, tap_add, tap_add_colorless};

fn gate_types() -> Subtypes {
    Subtypes { land_types: vec![LandType::Gate], ..Default::default() }
}

fn gate() -> R {
    R::HasLandType(LandType::Gate)
}

/// A Gate that enters tapped, chooses a color other than its own, and taps
/// for its color or the chosen one.
fn thriving_gate(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: gate_types(),
        static_abilities: vec![enters_tapped()],
        as_enters_effect: Some(Effect::ChooseColorForSelfOtherThan(color)),
        activated_abilities: vec![
            tap_add(color),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::ChosenColorOfSource },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// "At the beginning of your upkeep, if `cond`, you win the game."
fn upkeep_win(cond: Predicate) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
            .with_filter(cond.clone()),
        effect: Effect::If {
            cond,
            then: Box::new(Effect::WinGame { who: PlayerRef::You }),
            else_: Box::new(Effect::Noop),
        },
    }
}

fn greatest_power_at_least(n: i32) -> Predicate {
    Predicate::ValueAtLeast(Value::GreatestPowerControlled { who: PlayerRef::You }, Value::Const(n))
}

pub fn black_dragon_gate() -> CardDefinition {
    thriving_gate("Black Dragon Gate", Color::Black)
}

pub fn citadel_gate() -> CardDefinition {
    thriving_gate("Citadel Gate", Color::White)
}

pub fn cliffgate() -> CardDefinition {
    thriving_gate("Cliffgate", Color::Red)
}

pub fn manor_gate() -> CardDefinition {
    thriving_gate("Manor Gate", Color::Green)
}

pub fn sea_gate() -> CardDefinition {
    thriving_gate("Sea Gate", Color::Blue)
}

/// Baldur's Gate — {T}: {C}; {2}, {T}: X mana of one color, X the other
/// Gates you control.
pub fn baldurs_gate() -> CardDefinition {
    CardDefinition {
        name: "Baldur's Gate",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        subtypes: gate_types(),
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::CountOf(Box::new(Selector::EachPermanent(
                        gate().and(R::ControlledByYou).and(R::OtherThanSource),
                    )))),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Gond Gate — your Gates enter untapped; {T}: {C}; {T}: a color a Gate of
/// yours could produce.
pub fn gond_gate() -> CardDefinition {
    CardDefinition {
        name: "Gond Gate",
        card_types: vec![CardType::Land],
        subtypes: gate_types(),
        static_abilities: vec![StaticAbility {
            description: "Gates you control enter untapped.",
            effect: StaticEffect::MatchingEnterUntapped { filter: gate() },
        }],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyColorAGateYouControlCouldProduce,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Heap Gate — {T}: {C}; {1}, {T}: any color; {1}, {T}, tap another Gate: a
/// Treasure.
pub fn heap_gate() -> CardDefinition {
    CardDefinition {
        name: "Heap Gate",
        card_types: vec![CardType::Land],
        subtypes: gate_types(),
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                tap_other_filter: Some(gate().and(R::Untapped)),
                effect: mint_treasures(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Go-Shintai of Life's Origin — WUBRG, {T}: an enchantment card from your
/// graveyard onto the battlefield; it or another nontoken Shrine entering
/// makes a 1/1 colorless Shrine enchantment creature token.
pub fn go_shintai_of_lifes_origin() -> CardDefinition {
    let shrine = || R::HasEnchantmentSubtype(EnchantmentSubtype::Shrine);
    CardDefinition {
        name: "Go-Shintai of Life's Origin",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Shrine],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: shrine().and(R::NotToken),
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: std::sync::Arc::new(TokenDefinition {
                    name: "Shrine".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Enchantment, CardType::Creature],
                    subtypes: Subtypes {
                        enchantment_subtypes: vec![EnchantmentSubtype::Shrine],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Enchantment.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Happily Ever After — each player gains 5 and draws on entry; you win at
/// your upkeep with five colors among your permanents, six card types among
/// your permanents and graveyard, and at least your starting life.
pub fn happily_ever_after() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(5),
                },
                Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
            ])),
            upkeep_win(Predicate::All(vec![
                Predicate::ValueAtLeast(
                    Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(
                        R::ControlledByYou,
                    ))),
                    Value::Const(5),
                ),
                Predicate::ValueAtLeast(
                    Value::CardTypesAmongPermanentsAndGraveyard(PlayerRef::You),
                    Value::Const(6),
                ),
                Predicate::ValueAtLeast(Value::LifeOf(PlayerRef::You), Value::StartingLifeTotal),
            ])),
        ],
        ..enchantment("Happily Ever After", cost(&[generic(2), w()]))
    }
}

/// Liliana's Contract — draw four and lose 4 on entry; you win at your
/// upkeep with four differently-named Demons.
pub fn lilianas_contract() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                draw(4),
                Effect::LoseLife { who: Selector::You, amount: Value::Const(4) },
            ])),
            upkeep_win(Predicate::ValueAtLeast(
                Value::DistinctNamesControlledMatching(
                    R::Creature.and(R::HasCreatureType(CreatureType::Demon)),
                ),
                Value::Const(4),
            )),
        ],
        ..enchantment("Liliana's Contract", cost(&[generic(3), b(), b()]))
    }
}

/// Mayael's Aria — each upkeep: counters at power 5, 10 life at power 10, a
/// win at power 20.
pub fn mayaels_aria() -> CardDefinition {
    let step = |n: i32, then: Effect| Effect::If {
        cond: greatest_power_at_least(n),
        then: Box::new(then),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                step(
                    5,
                    Effect::AddCounter {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                ),
                step(10, Effect::GainLife { who: Selector::You, amount: Value::Const(10) }),
                step(20, Effect::WinGame { who: PlayerRef::You }),
            ]),
        }],
        ..enchantment("Mayael's Aria", cost(&[r(), g(), w()]))
    }
}

/// Mechanized Production — enchant an artifact you control; each upkeep a
/// token copy of it, then a win with eight same-named artifacts.
pub fn mechanized_production() -> CardDefinition {
    let yours = || R::Artifact.and(R::ControlledByYou);
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        attach_only_filter: Some(yours()),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(yours()) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::AttachedTo(Box::new(Selector::This)),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
                Effect::If {
                    cond: Predicate::ControlsSameNamedAtLeast {
                        who: PlayerRef::You,
                        filter: R::Artifact,
                        at_least: 8,
                    },
                    then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..enchantment("Mechanized Production", cost(&[generic(2), u(), u()]))
    }
}

/// Revel in Riches — a Treasure per opponent's creature dying; you win at
/// your upkeep with ten Treasures.
pub fn revel_in_riches() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: mint_treasures(1),
            },
            upkeep_win(Predicate::ValueAtLeast(
                Value::CountOf(Box::new(Selector::EachPermanent(
                    R::HasArtifactSubtype(ArtifactSubtype::Treasure).and(R::ControlledByYou),
                ))),
                Value::Const(10),
            )),
        ],
        ..enchantment("Revel in Riches", cost(&[generic(4), b()]))
    }
}

/// The World Tree — enters tapped, {T}: {G}; with six lands your lands tap
/// for any color; WWUUBBRRGG, {T}, sacrifice it: any number of Gods.
pub fn the_world_tree() -> CardDefinition {
    let your_lands = || R::Land.and(R::ControlledByYou);
    CardDefinition {
        name: "The World Tree",
        card_types: vec![CardType::Land],
        static_abilities: vec![
            enters_tapped(),
            StaticAbility {
                description: "As long as you control six or more lands, lands you control have \"{T}: Add one mana of any color.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::EachPermanent(your_lands()),
                    ability: ActivatedAbility {
                        tap_cost: true,
                        effect: Effect::AddMana {
                            who: PlayerRef::You,
                            pool: ManaPayload::AnyOneColor(Value::ONE),
                        },
                        ..Default::default()
                    },
                    condition: Some(Predicate::ValueAtLeast(
                        Value::CountOf(Box::new(Selector::EachPermanent(your_lands()))),
                        Value::Const(6),
                    )),
                },
            },
        ],
        activated_abilities: vec![
            tap_add(Color::Green),
            ActivatedAbility {
                mana_cost: cost(&[w(), w(), u(), u(), b(), b(), r(), r(), g(), g()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::SearchAnyNumber {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::God),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Trace of Abundance — enchant land; it has shroud and taps for an extra
/// mana of any color.
pub fn trace_of_abundance() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted land has shroud.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                    keyword: Keyword::Shroud,
                },
            },
            StaticAbility {
                description: "Whenever enchanted land is tapped for mana, its controller adds an additional one mana of any color.",
                effect: StaticEffect::ExtraManaOnLandTap {
                    enchanted_only: true,
                    filter: R::Land,
                    extra: ExtraManaKind::AnyColor,
                    while_monarch: false,
                },
            },
        ],
        ..enchantment("Trace of Abundance", cost(&[hybrid(Color::Red, Color::White), g()]))
    }
}
