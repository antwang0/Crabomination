use crate::card::{CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes};
use crate::effect::shortcut::{dies_mint_token, etb_mint_token, ingest, prowess_trigger};
use crate::mana::{b, cost, g, generic, r, u};
use crabomination_base::tokens::eldrazi_scion_token;

/// Eldrazi Drone body shared by the Scion-making creatures below.
fn drone(name: &'static str, c: crate::mana::ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Drone],
            ..Default::default()
        },
        power: p,
        toughness: t,
        keywords: vec![Keyword::Devoid],
        ..Default::default()
    }
}

/// Stormchaser Mage — {1}{U}{R} 1/3 Flying Haste Prowess
pub fn stormchaser_mage() -> CardDefinition {
    CardDefinition {
        name: "Stormchaser Mage",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::Prowess],
        effect: Effect::Noop,
        triggered_abilities: vec![prowess_trigger()],
        ..Default::default()
    }
}

/// Mist Intruder — {1}{U} 1/2 Eldrazi Drone. Devoid, Flying, Ingest.
pub fn mist_intruder() -> CardDefinition {
    CardDefinition {
        name: "Mist Intruder",
        cost: cost(&[crate::mana::generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Drone],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Devoid, Keyword::Flying],
        triggered_abilities: vec![ingest()],
        ..Default::default()
    }
}

/// Breaker of Armies — {8} 10/8 colorless Eldrazi. All creatures able to
/// block it do so (CR 509.1c — `Keyword::AllMustBlock`).
pub fn breaker_of_armies() -> CardDefinition {
    CardDefinition {
        name: "Breaker of Armies",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 10,
        toughness: 8,
        keywords: vec![Keyword::AllMustBlock],
        ..Default::default()
    }
}

/// Eldrazi Devastator — {8} 8/9 colorless Eldrazi. Trample. (Naturally
/// colorless from its generic-only cost — no Devoid keyword needed.)
pub fn eldrazi_devastator() -> CardDefinition {
    CardDefinition {
        name: "Eldrazi Devastator",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 8,
        toughness: 9,
        keywords: vec![Keyword::Trample],
        ..Default::default()
    }
}

/// Warden of Geometries — {3}{C} 2/4 Eldrazi Drone. Devoid, {T}: Add {C}.
pub fn warden_of_geometries() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{ManaPayload, PlayerRef, Value};
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..drone("Warden of Geometries", cost(&[generic(3), crate::mana::colorless(1)]), 2, 4)
    }
}

/// Cultivator Drone — {3}{C} 2/2 Eldrazi Drone. Devoid, {T}: Add {C}{C}.
/// (The "spend only to cast colorless spells" restriction is dropped.)
pub fn cultivator_drone() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{ManaPayload, PlayerRef, Value};
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(2)),
            },
            ..Default::default()
        }],
        ..drone("Cultivator Drone", cost(&[generic(3), crate::mana::colorless(1)]), 2, 2)
    }
}

/// Kozilek's Channeler — {5} 4/4 colorless Eldrazi. {T}: Add {C}{C}.
pub fn kozileks_channeler() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{ManaPayload, PlayerRef, Value};
    CardDefinition {
        name: "Kozilek's Channeler",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 4,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(2)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Scion Summoner — {2}{G} 2/2 Eldrazi Drone. Devoid, ETB make a Scion.
pub fn scion_summoner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_mint_token(eldrazi_scion_token(), 1)],
        ..drone("Scion Summoner", cost(&[generic(2), g()]), 2, 2)
    }
}

/// Brood Monitor — {4}{G}{G} 3/3 Eldrazi Drone. Devoid, ETB make three Scions.
pub fn brood_monitor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_mint_token(eldrazi_scion_token(), 3)],
        ..drone("Brood Monitor", cost(&[generic(4), g(), g()]), 3, 3)
    }
}

/// Eldrazi Skyspawner — {2}{U} 2/1 Eldrazi Drone. Devoid, Flying, ETB make
/// a 1/1 Eldrazi Scion.
pub fn eldrazi_skyspawner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::Flying],
        triggered_abilities: vec![etb_mint_token(eldrazi_scion_token(), 1)],
        ..drone("Eldrazi Skyspawner", cost(&[generic(2), u()]), 2, 1)
    }
}

/// Incubator Drone — {3}{U} 2/3 Eldrazi Drone. Devoid, ETB make a Scion.
pub fn incubator_drone() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_mint_token(eldrazi_scion_token(), 1)],
        ..drone("Incubator Drone", cost(&[generic(3), u()]), 2, 3)
    }
}

/// Eyeless Watcher — {3}{G} 1/1 Eldrazi Drone. Devoid, ETB make two Scions.
pub fn eyeless_watcher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_mint_token(eldrazi_scion_token(), 2)],
        ..drone("Eyeless Watcher", cost(&[generic(3), g()]), 1, 1)
    }
}

/// Blisterpod — {G} 1/1 Eldrazi Drone. Devoid, dies → make a Scion.
pub fn blisterpod() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![dies_mint_token(eldrazi_scion_token(), 1)],
        ..drone("Blisterpod", cost(&[g()]), 1, 1)
    }
}

/// Catacomb Sifter — {1}{B}{G} 2/3 Eldrazi Drone. Devoid, ETB make a Scion;
/// whenever another creature you control dies, scry 1.
pub fn catacomb_sifter() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, SelectionRequirement, TriggeredAbility};
    use crate::effect::{PlayerRef, Predicate, Selector, Value};
    let scry_on_death = TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::OtherThanSource,
            }),
        effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
    };
    CardDefinition {
        triggered_abilities: vec![etb_mint_token(eldrazi_scion_token(), 1), scry_on_death],
        ..drone("Catacomb Sifter", cost(&[generic(1), b(), g()]), 2, 3)
    }
}

/// Sludge Crawler — {B} 1/1 Eldrazi Drone. Devoid, Ingest, {2}: +1/+1 EOT.
pub fn sludge_crawler() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{Selector, Value};
    CardDefinition {
        name: "Sludge Crawler",
        cost: cost(&[crate::mana::b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Drone],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![ingest()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::generic(2)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: crate::effect::Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
