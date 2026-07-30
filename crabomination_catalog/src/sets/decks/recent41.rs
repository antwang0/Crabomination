//! Prevention / graveyard-hate / green-tempo staples. Anchors two CR rules:
//! Mark of Asylum's new `StaticEffect::PreventNoncombatDamageToYourCreatures`
//! (CR 615) and Dryad Militant's instant-and-sorcery-only graveyard redirect
//! (CR 614.6 — `ExileCardsBoundForGraveyard.card_types`). Tests in
//! `tests/recent41.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{ActivatedAbility, Duration, ManaPayload, PlayerRef, Predicate};
use crate::mana::{Color, b, cost, g, generic, hybrid, r};

/// Mark of Asylum — {1}{W} Enchantment. Prevent all noncombat damage that
/// would be dealt to creatures you control.
pub fn mark_of_asylum() -> CardDefinition {
    CardDefinition {
        name: "Mark of Asylum",
        cost: cost(&[generic(1), crate::mana::w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Prevent all noncombat damage that would be dealt to creatures you control.",
            effect: StaticEffect::PreventNoncombatDamageToYourCreatures,
        }],
        ..Default::default()
    }
}

/// Dryad Militant — {G/W} 2/1 Dryad Soldier. Instant/sorcery cards bound for
/// any graveyard are exiled instead (CR 614.6).
pub fn dryad_militant() -> CardDefinition {
    CardDefinition {
        name: "Dryad Militant",
        cost: cost(&[hybrid(Color::Green, Color::White)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "If an instant or sorcery card would be put into a graveyard from anywhere, exile it instead.",
            effect: StaticEffect::ExileCardsBoundForGraveyard {
                opponents_only: false,
                own_only: false,
                colors: None,
                card_types: Some(vec![CardType::Instant, CardType::Sorcery]),
                void_counter: false,
            },
        }],
        ..Default::default()
    }
}

/// Plated Geopede — {1}{R} 1/1 Insect. First strike. Landfall — whenever a land
/// you control enters, it gets +2/+2 until end of turn.
pub fn plated_geopede() -> CardDefinition {
    CardDefinition {
        name: "Plated Geopede",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Land),
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Scale Up — {G} Sorcery. Until end of turn, target creature you control
/// becomes a 6/4 Wurm. (Overload is approximated away — base mode only.)
pub fn scale_up() -> CardDefinition {
    CardDefinition {
        name: "Scale Up",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::BecomeCreature {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(6),
            toughness: Value::Const(4),
            creature_types: vec![CreatureType::Wurm],
            keywords: vec![],
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Spawning Pool — Land that enters tapped. `{T}: Add {B}.` `{1}{B}: becomes a
/// 1/1 black Skeleton until end of turn (still a land).` (The animated Skeleton's
/// `{B}: Regenerate` grant is approximated away.)
pub fn spawning_pool() -> CardDefinition {
    CardDefinition {
        name: "Spawning Pool",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: Selector::This,
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Black]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    creature_types: vec![CreatureType::Skeleton],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
