//! Commander: 2026-10-17 (`019Y6J4R`) closers, fifth batch. Tests in
//! `tests/recent_b/cmdr_019y_e.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

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

/// Pack tactics (CR 207.2c ability word): "whenever this creature attacks, if
/// you attacked with creatures with total power 6 or greater this combat".
fn pack_tactics(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
            .with_filter(Predicate::AttackedWithTotalPowerAtLeast { who: PlayerRef::You, at_least: 6 }),
        effect,
    }
}

/// The time-counter pair Fury Charm / Timecrafting / Shivan Sand-Mage print.
fn suspended() -> R {
    R::InExile.and(R::IsSuspended)
}

/// Arc Blade — {3}{R}{R} sorcery. 2 damage to any target; exile it with three
/// time counters (CR 702.62: with suspend, it's suspended again). Suspend 3—{2}{R}.
pub fn arc_blade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(3, cost(&[generic(2), r()]))],
        exile_on_resolve_time_counters: 3,
        ..spell(
            "Arc Blade",
            cost(&[generic(3), r(), r()]),
            CardType::Sorcery,
            Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
        )
    }
}

/// Find the Path — {2}{G} Aura, enchant land. ETB: venture into the dungeon.
/// Enchanted land has "{T}: Add {G}{G}."
pub fn find_the_path() -> CardDefinition {
    CardDefinition {
        name: "Find the Path",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        triggered_abilities: vec![etb(Effect::Venture)],
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}: Add {G}{G}.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green, Color::Green]) },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Intrepid Outlander — {1}{G} 2/3 reach Orc Ranger. Pack tactics: venture.
pub fn intrepid_outlander() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![pack_tactics(Effect::Venture)],
        ..creature("Intrepid Outlander", cost(&[generic(1), g()]), vec![CreatureType::Orc, CreatureType::Ranger], 2, 3)
    }
}

/// Shivan Sand-Mage — {2}{R}{R} 3/2 Lizard Shaman. ETB, choose one: remove two
/// time counters from target permanent or suspended card; or put two on target
/// permanent with a time counter or suspended card. Suspend 4—{R}.
pub fn shivan_sand_mage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(4, cost(&[r()]))],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::RemoveCounter {
                what: target_filtered(R::OnBattlefield.or(suspended())),
                kind: CounterType::Time,
                amount: Value::Const(2),
            },
            Effect::AddCounter {
                what: target_filtered(R::OnBattlefield.and(R::WithCounter(CounterType::Time)).or(suspended())),
                kind: CounterType::Time,
                amount: Value::Const(2),
            },
        ]))],
        ..creature("Shivan Sand-Mage", cost(&[generic(2), r(), r()]), vec![CreatureType::Lizard, CreatureType::Shaman], 3, 2)
    }
}

/// Anchovy & Banana Pizza — {2}{B}{B} Food artifact. ETB: destroy target
/// creature. {2}, {T}, sacrifice it: gain 3 life.
pub fn anchovy_and_banana_pizza() -> CardDefinition {
    CardDefinition {
        name: "Anchovy & Banana Pizza",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Food], ..Default::default() },
        triggered_abilities: vec![etb(Effect::Destroy { what: target_filtered(R::Creature) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mastermind's Acquisition — {2}{B}{B} sorcery. Choose one: tutor any card to
/// hand; or a card you own from outside the game (none in Commander, CR 903.11).
pub fn masterminds_acquisition() -> CardDefinition {
    spell(
        "Mastermind's Acquisition",
        cost(&[generic(2), b(), b()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Hand(PlayerRef::You) },
            Effect::WishToHand { filter: R::Any },
        ]),
    )
}

/// Tiger-Tribe Hunter — {3}{R}{R} 4/4 trample Human Barbarian. Pack tactics:
/// you may sacrifice another creature; when you do, it deals damage equal to
/// that creature's power to target creature.
pub fn tiger_tribe_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![pack_tactics(Effect::MayDo {
            description: "Sacrifice another creature to fling its power at a creature?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::OtherThanSource),
                },
                Effect::ReflexiveTrigger {
                    body: Box::new(Effect::DealDamage {
                        to: target_filtered(R::Creature),
                        amount: Value::SacrificedPower,
                    }),
                },
            ])),
        })],
        ..creature(
            "Tiger-Tribe Hunter",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Human, CreatureType::Barbarian],
            4,
            4,
        )
    }
}

/// Disappearing Act — {1}{U}{U} instant. As an additional cost, return a
/// permanent you control to its owner's hand. Counter target spell.
pub fn disappearing_act() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::ReturnToHand { filter: R::Permanent, count: 1, count_x: false }],
        ..spell(
            "Disappearing Act",
            cost(&[generic(1), u(), u()]),
            CardType::Instant,
            Effect::CounterSpell { what: Selector::Target(0) },
        )
    }
}

/// Abuelo's Awakening — {X}{3}{W} sorcery. Return target artifact or non-Aura
/// enchantment card from your graveyard to the battlefield with X additional
/// +1/+1 counters; it's a 1/1 flying Spirit creature in addition to its types.
pub fn abuelos_awakening() -> CardDefinition {
    spell(
        "Abuelo's Awakening",
        cost(&[x(), generic(3), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    R::InGraveyard.and(R::OwnedByYou).and(
                        R::Artifact.or(
                            R::HasCardType(CardType::Enchantment)
                                .and(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).negate()),
                        ),
                    ),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::BecomeCreature {
                what: Selector::Target(0),
                power: Value::ONE,
                toughness: Value::ONE,
                creature_types: vec![CreatureType::Spirit],
                keywords: vec![Keyword::Flying],
                duration: Duration::Permanent,
            },
            Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::XFromCost },
        ]),
    )
}

/// Brilliance Unleashed — {4}{U}{R} sorcery. Choose one or both: 5 damage to
/// target creature; return target artifact card from your graveyard to the
/// battlefield, a 3/3 flying Robot artifact creature if it isn't a creature.
pub fn brilliance_unleashed() -> CardDefinition {
    spell(
        "Brilliance Unleashed",
        cost(&[generic(4), u(), r()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(5) },
                Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(R::InGraveyard.and(R::OwnedByYou).and(R::Artifact)),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Creature },
                        then: Box::new(Effect::Noop),
                        else_: Box::new(Effect::BecomeCreature {
                            what: Selector::Target(0),
                            power: Value::Const(3),
                            toughness: Value::Const(3),
                            creature_types: vec![CreatureType::Robot],
                            keywords: vec![Keyword::Flying],
                            duration: Duration::Permanent,
                        }),
                    },
                ]),
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
    )
}
