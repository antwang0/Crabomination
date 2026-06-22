//! Tarkir: Dragonstorm **Omen** cards (CR 702.183). Each is a Dragon creature
//! whose card may instead be cast from hand as an instant/sorcery "Omen" half
//! for the listed cost (`GameAction::CastOmen`); on resolution or counter the
//! card is shuffled into its owner's library. The creature lives on the parent
//! [`CardDefinition`]; the Omen half lives in `omen`. Tracked in `DECK_FEATURES.md`.

use crate::card::{
    Adventure, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement, Subtypes, Supertype, TriggeredAbility, Value, WardCost,
};
use crate::card::LandType;
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, x};

/// Dragon subtype shorthand for the Omen creature faces.
fn dragon() -> Subtypes {
    Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() }
}

/// Marang River Regent — {4}{U}{U} Dragon 6/7, Flying. ETB returns up to two
/// other target nonland permanents to their owners' hands. Omen — Coil and
/// Catch {3}{U}: draw three cards, then discard a card.
pub fn marang_river_regent() -> CardDefinition {
    CardDefinition {
        name: "Marang River Regent",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 6,
        toughness: 7,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                filter: SelectionRequirement::Nonland
                    .and(SelectionRequirement::OtherThanSource),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
        }],
        omen: Some(Box::new(Adventure {
            name: "Coil and Catch",
            cost: cost(&[generic(3), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            ]),
        })),
        ..Default::default()
    }
}

/// Bloomvine Regent — {3}{G}{G} Dragon 4/5, Flying. Whenever this or another
/// Dragon you control enters, gain 3 life. Omen — Claim Territory {2}{G}:
/// search up to two basic Forests, one to the battlefield tapped, one to hand.
pub fn bloomvine_regent() -> CardDefinition {
    let basic_forest = SelectionRequirement::HasLandType(LandType::Forest)
        .and(SelectionRequirement::HasSupertype(Supertype::Basic));
    CardDefinition {
        name: "Bloomvine Regent",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        }],
        omen: Some(Box::new(Adventure {
            name: "Claim Territory",
            cost: cost(&[generic(2), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: basic_forest.clone(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    count: Value::Const(1),
                },
                Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: basic_forest,
                    to: ZoneDest::Hand(PlayerRef::You),
                    count: Value::Const(1),
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Scavenger Regent — {3}{B} Dragon 4/4, Flying, Ward—Discard a card. Omen —
/// Exude Toxin {X}{B}{B}: each non-Dragon creature gets -X/-X until end of turn.
pub fn scavenger_regent() -> CardDefinition {
    CardDefinition {
        name: "Scavenger Regent",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Discard(1))],
        omen: Some(Box::new(Adventure {
            name: "Exude Toxin",
            cost: cost(&[x(), b(), b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(
                        SelectionRequirement::HasCreatureType(CreatureType::Dragon).negate(),
                    ),
                ),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                    toughness: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                    duration: Duration::EndOfTurn,
                }),
            },
        })),
        ..Default::default()
    }
}

/// Dirgur Island Dragon — {5}{U} Dragon 4/4, Flying, Ward {2}. Omen —
/// Skimming Strike {1}{U}: tap up to one target creature, then draw a card.
pub fn dirgur_island_dragon() -> CardDefinition {
    CardDefinition {
        name: "Dirgur Island Dragon",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::generic(2))],
        omen: Some(Box::new(Adventure {
            name: "Skimming Strike",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 1,
                    filter: SelectionRequirement::Creature,
                    effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
        })),
        ..Default::default()
    }
}

/// Twinmaw Stormbrood — {5}{W} Dragon 5/4, Flying. ETB gain 5 life. Omen —
/// Charring Bite {1}{R}: deal 5 damage to target creature without flying.
pub fn twinmaw_stormbrood() -> CardDefinition {
    CardDefinition {
        name: "Twinmaw Stormbrood",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
        }],
        omen: Some(Box::new(Adventure {
            name: "Charring Bite",
            cost: cost(&[generic(1), r()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying).negate()),
                ),
                amount: Value::Const(5),
            },
        })),
        ..Default::default()
    }
}

/// Disruptive Stormbrood — {4}{G} Dragon 3/3, Flying. ETB destroy up to one
/// target artifact or enchantment. Omen — Petty Revenge {1}{B}: destroy target
/// creature with power 3 or less.
pub fn disruptive_stormbrood() -> CardDefinition {
    CardDefinition {
        name: "Disruptive Stormbrood",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: dragon(),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                filter: SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
        }],
        omen: Some(Box::new(Adventure {
            name: "Petty Revenge",
            cost: cost(&[generic(1), b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(3)),
                ),
            },
        })),
        ..Default::default()
    }
}
