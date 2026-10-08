//! Commander: most-built commanders missing from the catalog
//! (COMMANDER_BACKLOG §1), each with the cards its EDHREC average deck
//! needed beyond what the catalog had, seated as a pod deck. Tests in
//! `tests/recent_b/cmdr_most_built.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::if_discarded;
use crate::card::MayPlayDuration;
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, Color};

// ── Ob Nixilis, Captive Kingpin (BR) ────────────────────────────────────────

/// Ob Nixilis, Captive Kingpin — flying, trample; one or more opponents each
/// losing exactly 1 life grows it and impulse-draws a card until your next
/// end step.
pub fn ob_nixilis_captive_kingpin() -> CardDefinition {
    CardDefinition {
        name: "Ob Nixilis, Captive Kingpin",
        cost: cost(&[generic(2), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeLost, EventScope::OpponentControl).once_per_batch_losing_exactly(1),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::UntilYourNextEndStep,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Mount Doom — legendary land: {T}, pay 1 life for {B} or {R}; {1}{B}{R},
/// {T}: 1 damage to each opponent; {5}{B}{R}, {T}, sacrifice it and a
/// legendary artifact: choose up to two creatures (not targets), destroy the
/// rest, as a sorcery.
pub fn mount_doom() -> CardDefinition {
    CardDefinition {
        name: "Mount Doom",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                life_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::Black, Color::Red], Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b(), r()]),
                tap_cost: true,
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5), b(), r()]),
                tap_cost: true,
                sac_cost: true,
                sac_other_filter: Some((R::Artifact.and(R::HasSupertype(Supertype::Legendary)), 1)),
                sorcery_speed: true,
                effect: Effect::ChooseSomeAmong {
                    what: Selector::EachPermanent(R::Creature),
                    chooser: PlayerRef::You,
                    count: Value::Const(2),
                    up_to: true,
                    chosen: Box::new(Effect::Noop),
                    other: Box::new(Effect::Destroy { what: Selector::SeparatedPile { chosen: false } }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Manabarbs — whenever a player taps a land for mana, 1 damage to that
/// player (a separate trigger per land, not a mana ability).
pub fn manabarbs() -> CardDefinition {
    CardDefinition {
        name: "Manabarbs",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Shadow of the Goblin — first main phase: discard a card, and if you do,
/// draw one; a land played or a spell cast from anywhere but your hand deals
/// 1 damage to each opponent.
pub fn shadow_of_the_goblin() -> CardDefinition {
    let ping = || Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE };
    CardDefinition {
        name: "Shadow of the Goblin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    if_discarded(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                ]),
            },
            TriggeredAbility {
                // CR 305.1 — `LandPlayed`'s amount is 2 for a land PLAYED from
                // anywhere but the hand (1 from hand, 0 put onto the battlefield).
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl)
                    .with_filter(Predicate::ValueEquals(Value::TriggerEventAmount, Value::Const(2))),
                effect: ping(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::SpellNotCastFromHand },
                ),
                effect: ping(),
            },
        ],
        ..Default::default()
    }
}
