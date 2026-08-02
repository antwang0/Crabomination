//! Gap batch — "up to one target" / optional-fight staples (OTJ/MKM/M21/BLB)
//! plus the from-hand discard-activated primitive. Showcases the new
//! `Effect::OptionalTargets` wrapper (Primal Might, Boom Box). Tests in
//! `tests/recent229.rs`.

use crate::card::CardDefinition;
use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardType, CounterType, CreatureType, EquipBonus, Keyword,
    SelectionRequirement as R, Subtypes, TokenDefinition,
};
use crate::effect::shortcut::{etb, investigate, mint_treasures};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, TriggeredAbility,
    Value,
};
use crate::mana::{Color, b, cost, g, generic, u, w, x};

/// Primal Might — {X}{G} Sorcery. Target creature you control gets +X/+X until
/// end of turn. Then it fights up to one target creature you don't control.
pub fn primal_might() -> CardDefinition {
    let mine = Selector::TargetFiltered {
        slot: 0,
        filter: R::Creature.and(R::ControlledByYou),
    };
    let theirs = Selector::TargetFiltered {
        slot: 1,
        filter: R::Creature.and(R::ControlledByOpponent),
    };
    CardDefinition {
        name: "Primal Might",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Sorcery],
        // slot 1 (the fight defender) is "up to one" — declinable.
        effect: Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: mine.clone(),
                    power: Value::XFromCost,
                    toughness: Value::XFromCost,
                    duration: Duration::EndOfTurn,
                },
                Effect::Fight {
                    attacker: mine,
                    defender: theirs,
                },
            ])),
        },
        ..Default::default()
    }
}

/// Boom Box — {2} Artifact. {6}, {T}, Sacrifice this artifact: Destroy up to
/// one target artifact, up to one target creature, and up to one target land.
pub fn boom_box() -> CardDefinition {
    let art = Selector::TargetFiltered {
        slot: 0,
        filter: R::Artifact,
    };
    let cre = Selector::TargetFiltered {
        slot: 1,
        filter: R::Creature,
    };
    let land = Selector::TargetFiltered {
        slot: 2,
        filter: R::Land,
    };
    CardDefinition {
        name: "Boom Box",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            tap_cost: true,
            sac_cost: true,
            // all three slots optional (min 0).
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::Destroy { what: art },
                    Effect::Destroy { what: cre },
                    Effect::Destroy { what: land },
                ])),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Prizefight — {1}{G} Instant. Target creature you control fights target
/// creature you don't control. Create a Treasure token.
pub fn prizefight() -> CardDefinition {
    let mine = Selector::TargetFiltered {
        slot: 0,
        filter: R::Creature.and(R::ControlledByYou),
    };
    let theirs = Selector::TargetFiltered {
        slot: 1,
        filter: R::Creature.and(R::ControlledByOpponent),
    };
    CardDefinition {
        name: "Prizefight",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Fight {
                attacker: mine,
                defender: theirs,
            },
            mint_treasures(1),
        ]),
        ..Default::default()
    }
}

/// Out Cold — {3}{U} Instant. This spell can't be countered. Tap up to two
/// target creatures and put a stun counter on each of them. Investigate.
pub fn out_cold() -> CardDefinition {
    CardDefinition {
        name: "Out Cold",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::Tap {
                        what: Selector::Target(0),
                    },
                    Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::Stun,
                        amount: Value::ONE,
                    },
                ])),
            },
            investigate(1),
        ]),
        ..Default::default()
    }
}

/// Harvester of Misery — {3}{B}{B} 5/4 Spirit. Menace. ETB other creatures get
/// -2/-2 until end of turn. {1}{B}, Discard this card: Target creature gets
/// -2/-2 until end of turn.
pub fn harvester_of_misery() -> CardDefinition {
    CardDefinition {
        name: "Harvester of Misery",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature,
                },
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn dog_token() -> TokenDefinition {
    TokenDefinition {
        name: "Dog".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Krovod Haunch — {W} Artifact — Food Equipment. Equipped creature gets +2/+0.
/// {2}, {T}, Sacrifice this Equipment: You gain 3 life. When this Equipment is
/// put into a graveyard from the battlefield, you may pay {1}{W}. If you do,
/// create two 1/1 white Dog creature tokens. Equip {2}.
pub fn krovod_haunch() -> CardDefinition {
    CardDefinition {
        name: "Krovod Haunch",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Food, ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 0,
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {1}{W} to create two 1/1 white Dog tokens?".into(),
                mana_cost: cost(&[generic(1), w()]),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: dog_token(),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}
