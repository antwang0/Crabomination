//! Kamigawa: Neon Dynasty (NEO) batch. Rides existing primitives: from-hand
//! Channel abilities (`from_hand` + `discard_self_cost`), the artifact-spell
//! cost reducer, restricted mana, becomes-tapped / attacks-alone triggers,
//! Ninjutsu, and Reconfigure. Tests in `tests/recent95.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    Selector, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{deal, etb, mint_treasures, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect};
use crate::mana::{Color, SpendRestriction, cost, generic, r, u, w};

/// A 2/2 white Samurai token with vigilance (Imperial Oath, Experimental
/// Synthesizer).
fn samurai_token() -> TokenDefinition {
    TokenDefinition {
        name: "Samurai".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Samurai],
            ..Default::default()
        },
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    }
}

/// Golden-Tail Disciple — {2}{W} 2/3 Fox Monk Enchantment Creature with lifelink.
pub fn golden_tail_disciple() -> CardDefinition {
    CardDefinition {
        name: "Golden-Tail Disciple",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fox, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    }
}

/// Automated Artificer — {2} 1/3 Artificer artifact creature. {T}: Add {C}.
/// Spend this mana only to activate an ability or cast an artifact spell.
pub fn automated_artificer() -> CardDefinition {
    CardDefinition {
        name: "Automated Artificer",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::Const(1))),
                    SpendRestriction::NoNonartifactSpells,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Network Disruptor — {U} 1/1 Moonfolk Rogue artifact creature, flying. ETB:
/// tap target permanent.
pub fn network_disruptor() -> CardDefinition {
    CardDefinition {
        name: "Network Disruptor",
        cost: cost(&[u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Moonfolk, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Tap {
            what: target_filtered(R::Permanent),
        })],
        ..Default::default()
    }
}

/// Enthusiastic Mechanaut — {U}{R} 2/2 Goblin Artificer artifact creature,
/// flying. Artifact spells you cast cost {1} less to cast.
pub fn enthusiastic_mechanaut() -> CardDefinition {
    CardDefinition {
        name: "Enthusiastic Mechanaut",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Artifact spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::Artifact,
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Imperial Oath — {5}{W} Sorcery. Create three 2/2 white Samurai tokens with
/// vigilance, then scry 3.
pub fn imperial_oath() -> CardDefinition {
    CardDefinition {
        name: "Imperial Oath",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: Box::new(samurai_token()),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

/// Twinshot Sniper — {3}{R} 2/3 Goblin Archer artifact creature, reach. ETB: it
/// deals 2 damage to any target. Channel — {1}{R}, Discard this card: deals 2
/// damage to any target.
pub fn twinshot_sniper() -> CardDefinition {
    CardDefinition {
        name: "Twinshot Sniper",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Archer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(deal(2, target_any()))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            from_hand: true,
            discard_self_cost: true,
            effect: deal(2, target_any()),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Moonfolk Puzzlemaker — {2}{U} 1/4 Moonfolk Wizard artifact creature, flying.
/// Whenever it becomes tapped, scry 1.
pub fn moonfolk_puzzlemaker() -> CardDefinition {
    CardDefinition {
        name: "Moonfolk Puzzlemaker",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Moonfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Jukai Preserver — {3}{G} 3/3 Human Druid Enchantment Creature. ETB: put a
/// +1/+1 counter on target creature you control. Channel — {2}{G}, Discard this
/// card: put a +1/+1 counter on target creature you control. (The channel's "up
/// to two target creatures" is modeled as a single target.)
pub fn jukai_preserver() -> CardDefinition {
    use crate::mana::g;
    let counter_one = || Effect::AddCounter {
        what: target_filtered(R::Creature.and(R::ControlledByYou)),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    };
    CardDefinition {
        name: "Jukai Preserver",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(counter_one())],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: counter_one(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Selfless Samurai — {1}{W} 2/2 Fox Samurai. Whenever a Samurai or Warrior you
/// control attacks alone, it gains lifelink until end of turn. Sacrifice this
/// creature: another target creature you control gains indestructible until end
/// of turn.
pub fn selfless_samurai() -> CardDefinition {
    CardDefinition {
        name: "Selfless Samurai",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fox, CreatureType::Samurai],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::AttackingAlone,
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Samurai)
                            .or(R::HasCreatureType(CreatureType::Warrior)),
                    },
                ]),
            ),
            effect: Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Prosperous Thief — {2}{U} 3/2 Human Ninja. Ninjutsu {1}{U}. Whenever it deals
/// combat damage to a player, create a Treasure token. (The printed trigger fires
/// off any Ninja/Rogue you control; modeled as this creature's own combat damage.)
pub fn prosperous_thief() -> CardDefinition {
    CardDefinition {
        name: "Prosperous Thief",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ninja],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: mint_treasures(1),
        }],
        ..Default::default()
    }
}

/// Bronzeplate Boar — {2}{R} 3/2 Equipment Boar artifact creature, trample.
/// Equipped creature gets +3/+2 and has trample. Reconfigure {5}.
pub fn bronzeplate_boar() -> CardDefinition {
    CardDefinition {
        name: "Bronzeplate Boar",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample, Keyword::Reconfigure(cost(&[generic(5)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 2,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        }),
        ..Default::default()
    }
}
