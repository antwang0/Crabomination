//! DSK/FDN gap batch 4 — a soft counter, a basic-land tutor rock, and a
//! delirium punisher. Tests in `tests/recent205.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    Subtypes, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, TriggeredAbility,
    ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, u};

/// Don't Make a Sound — {1}{U} Instant. Counter target spell unless its controller
/// pays {2}. (The reflexive surveil-2 rider is approximated away.)
pub fn dont_make_a_sound() -> CardDefinition {
    CardDefinition {
        name: "Don't Make a Sound",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack),
            mana_cost: cost(&[generic(2)]),
            exile: false,
            extra_generic: None,
        },
        ..Default::default()
    }
}

/// Keys to the House — {1} Artifact. {1}, {T}, Sacrifice: search your library for
/// a basic land card, put it into your hand, then shuffle. (The Room lock/unlock
/// mode is approximated away.)
pub fn keys_to_the_house() -> CardDefinition {
    CardDefinition {
        name: "Keys to the House",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Osseous Sticktwister — {1}{B} 2/2 Artifact Creature — Scarecrow, Lifelink.
/// Delirium — at your end step, if four+ card types are in your graveyard, each
/// opponent sacrifices a nonland permanent or discards a card; each opponent who
/// does neither takes damage equal to this creature's power.
pub fn osseous_sticktwister() -> CardDefinition {
    CardDefinition {
        name: "Osseous Sticktwister",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Scarecrow],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::DeliriumActive {
                who: PlayerRef::You,
            }),
            effect: Effect::Punisher {
                chooser: Selector::Player(PlayerRef::EachOpponent),
                options: vec![
                    Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::You),
                        count: Value::ONE,
                        filter: R::Nonland,
                    },
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::You),
                        amount: Value::ONE,
                        random: false,
                    },
                ],
                otherwise: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                }),
            },
        }],
        ..Default::default()
    }
}
