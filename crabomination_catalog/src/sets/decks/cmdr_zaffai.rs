//! Commander: the cards the **Prismari Performance** precon (C21, Zaffai,
//! Thunder Conductor) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Apex of Power** — the exiled cards may also be played as lands.
//! - **Dazzling Sphinx** — a found card you don't cast stays in exile.
//! - **Muse Vortex** — the uncast non-instant/sorcery cards go to the bottom
//!   in exile order, not a random one.
//! - **Radiant Performer** — copies spells only, not abilities.
//! - **Zaffai** — one trigger per copy event, however many copies it made.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, MayPlayDuration, CardType, CounterType, CreatureType, DynamicPt, Keyword,
    LandType, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Selector, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::catalog::sets::{enters_tapped, tap_add};
use crate::effect::shortcut::{cast_is_instant_or_sorcery, magecraft, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition, ManaPayload,
    PlayerRef, Predicate, RevealMissDest, StaticAbility, StaticEffect, ZoneDest,
    ZoneRef,
};
use crate::mana::{Color, ManaCost, SpendRestriction, cost, generic, r, u, x};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn basic_landcycling(c: ManaCost) -> Keyword {
    Keyword::Typecycling(Box::new((c, R::IsBasicLand)))
}

fn cast_free(what: Selector, copy: bool) -> Effect {
    Effect::CastWithoutPayingImmediate {
        reduce_generic: 0,
        pay_own_cost: false,
        what,
        source_zone: Zone::Exile,
        exile_after: false,
        copy,
    }
}

/// Apex of Power — exile your top seven, cast spells from among them this
/// turn; cast from hand, add ten mana of one color. Residual: the exiled
/// cards may also be played as lands.
pub fn apex_of_power() -> CardDefinition {
    spell(
        "Apex of Power",
        cost(&[generic(7), r(), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(7),
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            Effect::If {
                cond: Predicate::CastFromHand,
                then: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(10)),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Dazzling Sphinx — flying; hitting a player, they exile until an instant
/// or sorcery, which you may cast free; the rest go to the bottom.
/// Residual: a found card you don't cast stays in exile.
pub fn dazzling_sphinx() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::RevealUntilFind {
                    who: PlayerRef::TriggerEventPlayer,
                    find: instant_or_sorcery(),
                    to: ZoneDest::Exile,
                    cap: Value::Const(60),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::BottomRandom,
                },
                cast_free(Selector::ExiledThisResolution { filter: instant_or_sorcery() }, false),
            ]),
        }],
        ..creature("Dazzling Sphinx", cost(&[generic(3), u(), u()]), vec![CreatureType::Sphinx], 4, 5)
    }
}

/// Desert of the Fervent — enters tapped; {T}: {R}; cycling {1}{R}.
pub fn desert_of_the_fervent() -> CardDefinition {
    CardDefinition {
        name: "Desert of the Fervent",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Desert], ..Default::default() },
        keywords: vec![Keyword::Cycling(cost(&[generic(1), r()]))],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![tap_add(Color::Red)],
        ..Default::default()
    }
}

/// Elementalist's Palette — an {X} spell adds two charge counters; {T}: any
/// color; {T}: {C} per charge counter, only on costs with {X}.
pub fn elementalists_palette() -> CardDefinition {
    CardDefinition {
        name: "Elementalist's Palette",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasXInCost },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::Const(2),
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colorless(Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Charge,
                        })),
                        SpendRestriction::XCostsOnly,
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Erratic Cyclops — trample; an instant or sorcery gives it +X/+0 (its mana
/// value).
pub fn erratic_cyclops() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Erratic Cyclops",
            cost(&[generic(3), r()]),
            vec![CreatureType::Cyclops, CreatureType::Shaman],
            0,
            8,
        )
    }
}

/// Fiery Encore — discard, draw; a nonland discard deals its mana value to a
/// creature or planeswalker. Storm.
pub fn fiery_encore() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..spell(
            "Fiery Encore",
            cost(&[generic(4), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::LastDiscardedManaValue, Value::ONE),
                    then: Box::new(Effect::Reflexive {
                        body: Box::new(Effect::DealDamage {
                            to: target_filtered(R::Creature.or(R::Planeswalker)),
                            amount: Value::LastDiscardedManaValue,
                        }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Fiery Fall — 5 damage to target creature; basic landcycling {1}{R}.
pub fn fiery_fall() -> CardDefinition {
    CardDefinition {
        keywords: vec![basic_landcycling(cost(&[generic(1), r()]))],
        ..spell(
            "Fiery Fall",
            cost(&[generic(5), r()]),
            CardType::Instant,
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(5) },
        )
    }
}

/// Inferno Project — trample; enters with counters equal to the total mana
/// value of instants and sorceries in your graveyard.
pub fn inferno_project() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::TotalManaValueOf(Box::new(Selector::EachMatching {
                zone: ZoneRef::Graveyard(PlayerRef::You),
                filter: instant_or_sorcery(),
            })),
        )),
        ..creature("Inferno Project", cost(&[generic(6), r()]), vec![CreatureType::Elemental], 0, 0)
    }
}

/// Inspiring Refrain — draw two, then exile it with three time counters;
/// suspend 3—{2}{U}.
pub fn inspiring_refrain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(3, cost(&[generic(2), u()]))],
        ..spell(
            "Inspiring Refrain",
            cost(&[generic(4), u(), u()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::ExileSelfSuspended,
            ]),
        )
    }
}

/// Jaya Ballard — +1: {R}{R}{R} for instants and sorceries; +1: discard up
/// to three, draw that many; −8: an emblem giving your graveyard instants
/// and sorceries flashback.
pub fn jaya_ballard() -> CardDefinition {
    CardDefinition {
        name: "Jaya Ballard",
        cost: cost(&[generic(2), r(), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Jaya],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Red, Color::Red, Color::Red])),
                        SpendRestriction::InstantSorceryOnly,
                    ),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::DiscardAnyNumber {
                        who: Selector::You,
                        filter: R::Any,
                        max: Some(Value::Const(3)),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisEffect },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Jaya Ballard".into(),
                    triggered: vec![],
                    statics: vec![StaticAbility {
                        description: "You may cast instant and sorcery spells from your graveyard. \
                                      If a spell cast this way would be put into your graveyard, \
                                      exile it instead.",
                        effect: StaticEffect::GraveyardInstantsSorceriesHaveFlashback,
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Living Lore — as it enters, exile an instant or sorcery card from your
/// graveyard; its P/T is that card's mana value; dealing combat damage, you
/// may sacrifice it to cast the card free.
pub fn living_lore() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::ExiledWithSourceManaValue),
        as_enters_effect: Some(Effect::AsEntersExileFromYourGraveyard {
            count: Value::ONE,
            filter: instant_or_sorcery(),
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamage, EventScope::SelfSource),
            effect: Effect::MaySacrificeSource {
                description: "Sacrifice Living Lore to cast the exiled card?".into(),
                then: Box::new(cast_free(Selector::CardExiledWithSource, false)),
                else_: None,
            },
        }],
        ..creature("Living Lore", cost(&[generic(3), u()]), vec![CreatureType::Avatar], 0, 0)
    }
}

/// Muse Vortex — exile your top X; you may cast an instant or sorcery with
/// mana value X or less free; the other instants and sorceries to hand, the
/// rest to the bottom. Residual: the bottom order isn't random.
pub fn muse_vortex() -> CardDefinition {
    let exiled = |filter: R| Selector::EachMatching { zone: ZoneRef::Exile, filter: R::ExiledWithSource.and(filter) };
    spell(
        "Muse Vortex",
        cost(&[x(), u(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ExileLinked {
                what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::XFromCost },
            },
            Effect::CastAnyOrderWithoutPaying {
                what: Selector::CardExiledWithSource,
                source_zone: Zone::Exile,
                filter: Some(instant_or_sorcery().and(R::ManaValueAtMostXFromCost)),
                cap: Some(Value::ONE),
                total_mana_value: None,
            },
            Effect::Move { what: exiled(instant_or_sorcery()), to: ZoneDest::Hand(PlayerRef::You) },
            Effect::Move {
                what: exiled(R::Any),
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Bottom },
            },
        ]),
    )
}

/// Pyromancer's Goggles — {T}: {R}; spent on a red instant or sorcery, copy
/// that spell (new targets allowed).
pub fn pyromancers_goggles() -> CardDefinition {
    CardDefinition {
        name: "Pyromancer's Goggles",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Red])),
                    SpendRestriction::RedInstantSorceryCopy,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Radiant Performer — flash; cast from hand, entering copies target spell
/// with a single target for each other permanent or player it could target.
/// Residual: spells only, not abilities.
pub fn radiant_performer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::CastFromHand),
            effect: Effect::CopySpellForEachOtherLegalTarget {
                what: target_filtered(R::SpellWithSingleTarget),
            },
        }],
        ..creature(
            "Radiant Performer",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Reinterpret — counter target spell; you may cast a spell of equal or
/// lesser mana value from your hand free.
pub fn reinterpret() -> CardDefinition {
    spell(
        "Reinterpret",
        cost(&[generic(2), u(), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::MayCastFromHandFreeMatching {
                filter: R::Nonland,
                max_mv: Value::CounteredSpellManaValue,
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Surge to Victory — exile an instant or sorcery from your graveyard; your
/// creatures get +X/+0 (its mana value); this turn each one that connects
/// casts a free copy of it.
pub fn surge_to_victory() -> CardDefinition {
    let mv = || Value::ManaValueOf(Box::new(Selector::CardExiledWithSource));
    spell(
        "Surge to Victory",
        cost(&[generic(4), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ExileLinked {
                what: target_filtered(instant_or_sorcery().and(R::InYourGraveyard)),
            },
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: mv(),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::CreaturesYouControlDealingCombatDamageThisTurn {
                body: Box::new(cast_free(Selector::CardExiledWithSource, true)),
            },
        ]),
    )
}

/// Traumatic Visions — counter target spell; basic landcycling {1}{U}.
pub fn traumatic_visions() -> CardDefinition {
    CardDefinition {
        keywords: vec![basic_landcycling(cost(&[generic(1), u()]))],
        ..spell(
            "Traumatic Visions",
            cost(&[generic(3), u(), u()]),
            CardType::Instant,
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
        )
    }
}

/// Zaffai, Thunder Conductor — magecraft: scry 1; mana value 5+, a 4/4
/// Elemental; 10+, 10 damage to a random opponent. Residual: one trigger per
/// copy event, however many copies it made.
pub fn zaffai_thunder_conductor() -> CardDefinition {
    let elemental = Arc::new(TokenDefinition {
        name: "Elemental".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        ..Default::default()
    });
    let mv_at_least = |n: i32| {
        Predicate::ValueAtLeast(Value::ManaValueOf(Box::new(Selector::TriggerSource)), Value::Const(n))
    };
    let body = Effect::Seq(vec![
        Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        Effect::If {
            cond: mv_at_least(5),
            then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: elemental }),
            else_: Box::new(Effect::Noop),
        },
        Effect::If {
            cond: mv_at_least(10),
            then: Box::new(Effect::DealDamage {
                to: Selector::Player(PlayerRef::RandomOpponent),
                amount: Value::Const(10),
            }),
            else_: Box::new(Effect::Noop),
        },
    ]);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            magecraft(body.clone()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCopied, EventScope::YourControl)
                    .with_filter(cast_is_instant_or_sorcery()),
                effect: body,
            },
        ],
        ..creature(
            "Zaffai, Thunder Conductor",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            1,
            4,
        )
    }
}
