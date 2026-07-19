//! A modern gap batch reusing existing primitives: threshold-gated untap
//! (Krosan Restorer), one-shot counter doubling (`Effect::DoubleCountersOnEach`
//! — Bristly Bill), and dies-reanimate-as-Treasure (Vraska, the Silencer, via
//! the `TriggerSource` reanimate + `Effect::BecomeTreasure` mechanism).
//! Tests in `recent_b/recent290`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r};

/// Krosan Restorer — {2}{G} 1/2 Human Druid. {T}: Untap target land.
/// Threshold — {T}: Untap up to three target lands (activate with 7+ cards in
/// your graveyard). (The three-land version targets any lands, auto-picked.)
pub fn krosan_restorer() -> CardDefinition {
    CardDefinition {
        name: "Krosan Restorer",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Untap { what: target_filtered(R::Land), up_to: None },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::ThresholdActive { who: PlayerRef::You }),
                effect: Effect::Untap {
                    what: Selector::EachPermanent(R::Land),
                    up_to: Some(Value::Const(3)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Vraska, the Silencer — {1}{B}{G} Legendary Gorgon Assassin 3/3. Deathtouch.
/// Whenever a nontoken creature an opponent controls dies, you may pay {1} to
/// return that card to the battlefield tapped under your control as a Treasure
/// artifact (losing all other card types).
pub fn vraska_the_silencer() -> CardDefinition {
    CardDefinition {
        name: "Vraska, the Silencer",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gorgon, CreatureType::Assassin],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken,
                }),
            effect: Effect::MayPay {
                description: "Pay {1} to steal the dead creature as a Treasure?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    },
                    Effect::BecomeTreasure { what: Selector::LastMoved },
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Zoyowa Lava-Tongue — {B}{R} Legendary Goblin Warlock 2/2. Deathtouch. At your
/// end step, if you descended this turn, each opponent may discard a card or
/// sacrifice a permanent; Zoyowa deals 3 damage to each who didn't. (The 3
/// damage targets each opponent — exact in 1v1; in multiplayer the punisher
/// payoff can't yet single out only the defaulting opponent.)
pub fn zoyowa_lava_tongue() -> CardDefinition {
    CardDefinition {
        name: "Zoyowa Lava-Tongue",
        cost: cost(&[b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                .with_filter(Predicate::DescendedThisTurn { who: PlayerRef::You }),
            effect: Effect::Punisher {
                chooser: Selector::Player(PlayerRef::EachOpponent),
                options: vec![
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::You),
                        amount: Value::ONE,
                        random: false,
                    },
                    Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::You),
                        count: Value::ONE,
                        filter: R::Permanent,
                    },
                ],
                otherwise: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                }),
            },
        }],
        ..Default::default()
    }
}
