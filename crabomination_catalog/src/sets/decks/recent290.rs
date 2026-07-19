//! A modern gap batch reusing existing primitives: threshold-gated untap
//! (Krosan Restorer), dies-reanimate-as-Treasure (Vraska, the Silencer, via
//! the `TriggerSource` reanimate + `Effect::BecomeTreasure` mechanism), a
//! descend punisher (Zoyowa Lava-Tongue), control-donation of a Treasure
//! (Discerning Financier, via `GainControl { to: Some(..) }`), landfall pump
//! (Grove Rumbler), an ETB -1/-1 (Blister Beetle), tapped-only removal (Swift
//! Response), and delirium-scaled counters (Might Beyond Reason).
//! Tests in `recent_b/recent290`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::game::effects::treasure_token;
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, w};

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
/// sacrifice a permanent; Zoyowa deals 3 damage to each who didn't (the damage
/// hits only the defaulting opponent via `PlayerRef::Triggerer`).
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
                    to: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(3),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Discerning Financier — {2}{W} 2/3 Human Noble. At your upkeep, if an opponent
/// controls more lands than you, create a Treasure. {2}{W}: another player gains
/// control of target Treasure you control; you draw a card. (The recipient is an
/// opponent — exact in 1v1.)
pub fn discerning_financier() -> CardDefinition {
    CardDefinition {
        name: "Discerning Financier",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer)
                .with_filter(Predicate::OpponentControlsMoreLandsThanYou),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: treasure_token(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::Seq(vec![
                Effect::GainControl {
                    what: target_filtered(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Treasure).and(R::ControlledByYou)),
                    to: Some(PlayerRef::EachOpponent),
                    duration: Duration::Permanent,
                },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Grove Rumbler — {2}{R}{G} 3/3 Elemental. Trample. Landfall — whenever a land
/// you control enters, it gets +2/+2 until end of turn.
pub fn grove_rumbler() -> CardDefinition {
    CardDefinition {
        name: "Grove Rumbler",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
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

/// Blister Beetle — {1}{B} 1/1 Insect. When it enters, target creature gets
/// -1/-1 until end of turn.
pub fn blister_beetle() -> CardDefinition {
    CardDefinition {
        name: "Blister Beetle",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Swift Response — {1}{W} Instant. Destroy target tapped creature.
pub fn swift_response() -> CardDefinition {
    CardDefinition {
        name: "Swift Response",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::Tapped)) },
        ..Default::default()
    }
}

/// Might Beyond Reason — {3}{G} Instant. Put two +1/+1 counters on target
/// creature — three instead with delirium (4+ card types in your graveyard).
pub fn might_beyond_reason() -> CardDefinition {
    CardDefinition {
        name: "Might Beyond Reason",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::IfPred {
                pred: Box::new(Predicate::DeliriumActive { who: PlayerRef::You }),
                then: Box::new(Value::Const(3)),
                else_: Box::new(Value::Const(2)),
            },
        },
        ..Default::default()
    }
}
