//! SNC/DMU gap batch — a Treasure-engine Cat and a kicker-reanimator. All on
//! existing primitives. Tests in `tests/recent_b/recent270.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::treasure_token;
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r};

/// Black Market Tycoon — {R}{G} 2/2 Cat Rogue. Upkeep: deals 2 damage to you
/// for each Treasure you control. {T}: create a Treasure token.
pub fn black_market_tycoon() -> CardDefinition {
    CardDefinition {
        name: "Black Market Tycoon",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::You),
                amount: Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(Value::CountOf(Box::new(Selector::ControlledBy {
                        who: PlayerRef::You,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Treasure),
                    }))),
                ),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(treasure_token()),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Balduvian Atrocity — {2}{B} 2/3 Phyrexian Berserker, menace. Kicker {R}.
/// When it enters, if it was kicked, return a creature card with mana value 3
/// or less from your graveyard to the battlefield with haste; sacrifice it at
/// the next end step.
pub fn balduvian_atrocity() -> CardDefinition {
    CardDefinition {
        name: "Balduvian Atrocity",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Menace, Keyword::Kicker(cost(&[r()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        R::Creature.and(R::InGraveyard).and(R::ManaValueAtMost(3)),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::SacrificePermanent {
                        what: Selector::Target(0),
                    }),
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}
