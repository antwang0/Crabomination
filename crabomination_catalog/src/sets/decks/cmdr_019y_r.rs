//! Commander: 2026-10-18 (`019Y6J4R`) closers, eighteenth batch. Tests in
//! `tests/recent_b/cmdr_019y_r.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword, LandType,
    Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{Color, ManaCost, b, cost, generic, u};

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

/// Veiling Oddity — {3}{U} 2/3 Illusion. Suspend 4—{1}{U}. When the last
/// time counter is removed from this card while it's exiled, creatures can't
/// be blocked this turn (CR 702.62b — the trigger functions from exile; the
/// creatures there as it resolves, CR 611.2c).
pub fn veiling_oddity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(4, cost(&[generic(1), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterRemoved(CounterType::Time), EventScope::SelfSource)
                .while_suspended()
                .with_filter(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::WithCounter(CounterType::Time).negate(),
                }),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Veiling Oddity", cost(&[generic(3), u()]), vec![CreatureType::Illusion], 2, 3)
    }
}

/// Jhessian Zombies — {4}{U}{B} 2/4 fear Zombie. Islandcycling {2},
/// swampcycling {2}: two typecycling abilities at one cost, written as one
/// whose search takes an Island or a Swamp card — the same choice, and
/// `GameAction::Landcycle` reads only a card's first cycling keyword.
pub fn jhessian_zombies() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Fear,
            Keyword::Typecycling(Box::new((
                cost(&[generic(2)]),
                R::HasLandType(LandType::Island).or(R::HasLandType(LandType::Swamp)),
            ))),
        ],
        ..creature("Jhessian Zombies", cost(&[generic(4), u(), b()]), vec![CreatureType::Zombie], 2, 4)
    }
}

/// Chaotic Transformation — {5}{R} sorcery. Exile up to one target artifact,
/// creature, enchantment, planeswalker and/or land. For each permanent exiled
/// this way, its controller reveals cards from the top of their library until
/// one shares a card type with it, puts that card onto the battlefield, then
/// shuffles. The five "up to one" slots are one target group whose targets
/// take distinct types (`R::TargetsTakeDistinctTypeSlots`); each is exiled and
/// replaced in turn.
pub fn chaotic_transformation() -> CardDefinition {
    let types = vec![CardType::Artifact, CardType::Creature, CardType::Enchantment, CardType::Planeswalker, CardType::Land];
    let any_of = types.iter().map(|t| R::HasCardType(t.clone())).reduce(|a, b| a.or(b)).unwrap_or(R::Any);
    CardDefinition {
        name: "Chaotic Transformation",
        cost: cost(&[generic(5), crate::mana::r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 5,
            min_targets: 0,
            filter: any_of.and(R::TargetsTakeDistinctTypeSlots(types)),
            effect: Box::new(Effect::ExileThenRevealUntilSharedType { what: Selector::Target(0) }),
        },
        ..Default::default()
    }
}

/// Fishing Pole — {1} Equipment. Equipped creature has "{1}, {T}, Tap Fishing
/// Pole: Put a bait counter on Fishing Pole." Whenever equipped creature
/// becomes untapped, remove a bait counter from this Equipment; if you do,
/// create a 1/1 blue Fish creature token. Equip {2}.
pub fn fishing_pole() -> CardDefinition {
    let this_pole = || R::HasName("Fishing Pole".into()).and(R::AttachedToThisSource);
    let fish = crate::card::TokenDefinition {
        name: "Fish".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fish], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Fishing Pole",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            activated_abilities: vec![crate::effect::ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                tap_others_cost: Some((this_pole(), 1)),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(this_pole()),
                    kind: CounterType::Bait,
                    amount: crate::card::Value::ONE,
                },
                ..Default::default()
            }],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::This, filter: R::WithCounter(CounterType::Bait) },
                    then: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter { what: Selector::This, kind: CounterType::Bait, amount: crate::card::Value::ONE },
                        Effect::CreateToken {
                            who: PlayerRef::You,
                            count: crate::card::Value::ONE,
                            definition: std::sync::Arc::new(fish),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            }],
            triggers_on_equipment: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}
