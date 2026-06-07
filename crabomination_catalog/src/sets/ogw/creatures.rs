use crate::card::{CardDefinition, CardType, CreatureType, Effect, Keyword, Subtypes};
use crate::effect::shortcut::{
    dies_mint_token, etb_mint_token, ingest, on_attack, on_cast, on_dies, prowess_trigger,
    target_filtered,
};
use crate::mana::{b, cost, g, generic, r, u};
use crabomination_base::tokens::{eldrazi_10_10_token, eldrazi_scion_token, eldrazi_spawn_token};

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

/// Salvage Drone — {U} 1/1 Eldrazi Drone. Devoid, Ingest; when it dies,
/// you may draw a card. (The "if you do, lose 1 life" rider is dropped.)
pub fn salvage_drone() -> CardDefinition {
    use crate::effect::{Selector, Value};
    CardDefinition {
        triggered_abilities: vec![
            ingest(),
            on_dies(Effect::MayDo {
                description: "draw a card".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            }),
        ],
        ..drone("Salvage Drone", cost(&[u()]), 1, 1)
    }
}

/// Skitterskin — {3}{B} 4/3 Eldrazi Drone. Devoid, can't block, {1}{B}:
/// Regenerate this creature. (The "only if you control another colorless
/// creature" gate is dropped — colorless-matters filter gap.)
pub fn skitterskin() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::Selector;
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::CantBlock],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..drone("Skitterskin", cost(&[generic(3), b()]), 4, 3)
    }
}

/// Mindmelter — {1}{U}{B} 2/2 Eldrazi Drone. Devoid, can't be blocked,
/// {3}{C}: target opponent discards a card (sorcery-speed). (The printed
/// "exiles a card from their hand" is approximated as a discard.)
pub fn mindmelter() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{PlayerRef, Selector, Value};
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::Unblockable],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), crate::mana::colorless(1)]),
            sorcery_speed: true,
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            },
            ..Default::default()
        }],
        ..drone("Mindmelter", cost(&[generic(1), u(), b()]), 2, 2)
    }
}

/// Deepfathom Skulker — {5}{U} 4/4 colorless Eldrazi. Devoid. Whenever a
/// creature you control deals combat damage to a player, you may draw a
/// card. {3}{C}: target creature can't be blocked this turn.
pub fn deepfathom_skulker() -> CardDefinition {
    use crate::card::{ActivatedAbility, EventKind, EventScope, EventSpec, SelectionRequirement,
        TriggeredAbility};
    use crate::effect::{Duration, Selector, Value};
    let draw_on_dmg = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
        effect: Effect::MayDo {
            description: "draw a card".into(),
            body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
        },
    };
    CardDefinition {
        name: "Deepfathom Skulker",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![draw_on_dmg],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), crate::mana::colorless(1)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Culling Drone — {1}{B} 2/2 Eldrazi Drone. Devoid, Ingest.
pub fn culling_drone() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![ingest()],
        ..drone("Culling Drone", cost(&[generic(1), b()]), 2, 2)
    }
}

/// Benthic Infiltrator — {2}{U} 1/4 Eldrazi Drone. Devoid, Ingest, can't
/// be blocked.
pub fn benthic_infiltrator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::Unblockable],
        triggered_abilities: vec![ingest()],
        ..drone("Benthic Infiltrator", cost(&[generic(2), u()]), 1, 4)
    }
}

/// Maw of Kozilek — {3}{R} 2/5 Eldrazi Drone. Devoid, {C}: +2/-2 EOT.
pub fn maw_of_kozilek() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{Duration, Selector, Value};
    CardDefinition {
        toughness: 5,
        power: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::colorless(1)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..drone("Maw of Kozilek", cost(&[generic(3), r()]), 2, 5)
    }
}

/// Voracious Null — {2}{B} 2/2 Zombie. "{1}{B}, Sacrifice another creature:
/// Put two +1/+1 counters on this creature. Activate only as a sorcery."
pub fn voracious_null() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType, SelectionRequirement};
    use crate::effect::{Selector, Value};
    CardDefinition {
        name: "Voracious Null",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sorcery_speed: true,
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vile Aggregate — {2}{R} */5 Eldrazi Drone. Devoid, Trample, Ingest;
/// its power equals the number of colorless creatures you control
/// (characteristic-defining, wired via `DynamicPt::ColorlessCreaturesControlled`).
pub fn vile_aggregate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::Trample],
        toughness: 5,
        triggered_abilities: vec![ingest()],
        ..drone("Vile Aggregate", cost(&[generic(2), r()]), 0, 5)
    }
}

/// Eldrazi Aggressor — {2}{R} 2/3 Eldrazi Drone. Devoid; has haste as long as
/// you control another colorless creature.
pub fn eldrazi_aggressor() -> CardDefinition {
    use crate::card::{SelectionRequirement, StaticAbility};
    use crate::effect::{Predicate, Selector, StaticEffect, Value};
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        static_abilities: vec![StaticAbility {
            description: "Has haste as long as you control another colorless creature.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::Colorless)
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    n: Value::Const(1),
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Haste],
            },
        }],
        ..drone("Eldrazi Aggressor", cost(&[generic(2), r()]), 2, 3)
    }
}

/// Reaver Drone — {B} 2/1 Eldrazi Drone. Devoid; at the beginning of your
/// upkeep, you lose 1 life unless you control another colorless creature.
pub fn reaver_drone() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, SelectionRequirement, TriggeredAbility};
    use crate::game::types::TurnStep;
    use crate::effect::{Predicate, Selector, Value};
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::Colorless)
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    n: Value::Const(1),
                },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(1) }),
            },
        }],
        ..drone("Reaver Drone", cost(&[b()]), 2, 1)
    }
}

/// Eldrazi Mimic — {2} 2/1 Eldrazi. Whenever another colorless creature you
/// control enters, you may set this creature's base P/T to that creature's
/// until end of turn.
pub fn eldrazi_mimic() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, SelectionRequirement, TriggeredAbility};
    use crate::effect::{Duration, Predicate, Selector, Value};
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::Colorless)
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::MayDo {
                description: "Set base P/T to the entering creature's?".into(),
                body: Box::new(Effect::SetBasePT {
                    what: Selector::This,
                    power: Value::PowerOf(Box::new(Selector::TriggerSource)),
                    toughness: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..colossus("Eldrazi Mimic", cost(&[generic(2)]), 2, 1)
    }
}

/// Havoc Sower — {3}{B} 3/3 Eldrazi Drone. Devoid; {1}{C}: +2/+1 until end of turn.
pub fn havoc_sower() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{Selector, Value};
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), crate::mana::colorless(1)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: crate::effect::Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..drone("Havoc Sower", cost(&[generic(3), b()]), 3, 3)
    }
}

/// Defiant Bloodlord — {5}{B}{B} 4/5 Vampire. Flying; whenever you gain life,
/// target opponent loses that much life.
pub fn defiant_bloodlord() -> CardDefinition {
    use crate::card::{CreatureType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::{PlayerRef, Selector, Value};
    CardDefinition {
        name: "Defiant Bloodlord",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..Default::default()
    }
}

/// Cinder Hellion — {4}{R} 4/4 Hellion. Trample; ETB deal 2 damage to target
/// opponent or planeswalker.
pub fn cinder_hellion() -> CardDefinition {
    use crate::card::{CreatureType, SelectionRequirement};
    use crate::effect::shortcut::etb;
    use crate::effect::Value;
    CardDefinition {
        name: "Cinder Hellion",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hellion],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Player.or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Void Grafter — {1}{G}{U} 2/4 Eldrazi Drone. Devoid, Flash; ETB another
/// target creature you control gains hexproof until end of turn.
pub fn void_grafter() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::shortcut::etb;
    use crate::effect::Duration;
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::Flash],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            keyword: Keyword::Hexproof,
            duration: Duration::EndOfTurn,
        })],
        ..drone("Void Grafter", cost(&[g(), u()]), 2, 4)
    }
}

/// Brood Butcher — {3}{B}{G} 3/3 Eldrazi Drone. Devoid; ETB make a Scion;
/// {B}{G}, sacrifice a creature: target creature gets -1/-1 until end of turn.
pub fn brood_butcher() -> CardDefinition {
    use crate::card::{ActivatedAbility, SelectionRequirement};
    use crate::effect::shortcut::pump_target;
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![etb_mint_token(eldrazi_scion_token(), 1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), g()]),
            sac_other_filter: Some((
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                1,
            )),
            effect: pump_target(-1, -1),
            ..Default::default()
        }],
        ..drone("Brood Butcher", cost(&[generic(3), b(), g()]), 3, 3)
    }
}

/// Lifespring Druid — {2}{G} 2/1 Elf Druid. {T}: Add one mana of any color.
pub fn lifespring_druid() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{ManaPayload, PlayerRef, Value};
    CardDefinition {
        name: "Lifespring Druid",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dread Drone — {4}{B} 4/1 Eldrazi Drone. Devoid, ETB make two 0/1
/// Eldrazi Spawn.
pub fn dread_drone() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_mint_token(eldrazi_spawn_token(), 2)],
        ..drone("Dread Drone", cost(&[generic(4), b()]), 4, 1)
    }
}

/// Slaughter Drone — {1}{B} 2/2 Eldrazi Drone. Devoid, {C}: gains
/// deathtouch until end of turn.
pub fn slaughter_drone() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{Duration, Selector};
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::colorless(1)]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..drone("Slaughter Drone", cost(&[generic(1), b()]), 2, 2)
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

// ── Eldrazi titans & colossi (cast triggers + Annihilator) ──────────────────

/// Colorless Eldrazi body shared by the big titans/colossi below.
fn colossus(name: &'static str, c: crate::mana::ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Ulamog, the Infinite Gyre — {11} 10/10 Legendary. Cast → destroy target
/// permanent; Indestructible; Annihilator 4; dies → shuffle graveyard into
/// library.
pub fn ulamog_the_infinite_gyre() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::PlayerRef;
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Indestructible, Keyword::Annihilator(4)],
        triggered_abilities: vec![
            on_cast(Effect::Destroy { what: target_filtered(SelectionRequirement::Permanent) }),
            on_dies(Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::You }),
        ],
        ..colossus("Ulamog, the Infinite Gyre", cost(&[generic(11)]), 10, 10)
    }
}

/// Kozilek, Butcher of Truth — {10} 12/12 Legendary. Cast → draw four cards;
/// Annihilator 4; dies → shuffle graveyard into library.
pub fn kozilek_butcher_of_truth() -> CardDefinition {
    use crate::effect::{PlayerRef, Selector, Value};
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Annihilator(4)],
        triggered_abilities: vec![
            on_cast(Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::Const(4) }),
            on_dies(Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::You }),
        ],
        ..colossus("Kozilek, Butcher of Truth", cost(&[generic(10)]), 12, 12)
    }
}

/// Pathrazer of Ulamog — {11} 9/9. Annihilator 3; can't be blocked except by
/// three or more creatures.
pub fn pathrazer_of_ulamog() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Annihilator(3), Keyword::CantBeBlockedExceptByN(3)],
        ..colossus("Pathrazer of Ulamog", cost(&[generic(11)]), 9, 9)
    }
}

/// Ulamog's Crusher — {8} 8/8. Annihilator 2; attacks each combat if able.
pub fn ulamogs_crusher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Annihilator(2), Keyword::MustAttack],
        ..colossus("Ulamog's Crusher", cost(&[generic(8)]), 8, 8)
    }
}

/// Artisan of Kozilek — {9} 10/9. Cast → return target creature card from your
/// graveyard to the battlefield; Annihilator 2. (The target's "from your
/// graveyard" zone gate is dropped — same approximation as Disentomb.)
pub fn artisan_of_kozilek() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::{PlayerRef, ZoneDest};
    CardDefinition {
        keywords: vec![Keyword::Annihilator(2)],
        triggered_abilities: vec![on_cast(Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        ..colossus("Artisan of Kozilek", cost(&[generic(9)]), 10, 9)
    }
}

/// Desolation Twin — {10} 10/10. Cast → create a 10/10 colorless Eldrazi token.
pub fn desolation_twin() -> CardDefinition {
    use crate::effect::{PlayerRef, Value};
    CardDefinition {
        triggered_abilities: vec![on_cast(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: eldrazi_10_10_token(),
        })],
        ..colossus("Desolation Twin", cost(&[generic(10)]), 10, 10)
    }
}

/// Hand of Emrakul — {9} 7/7. Annihilator 1. (Its "sacrifice four Eldrazi
/// Spawn rather than pay this spell's mana cost" alt-cost is dropped — no
/// sacrifice-N-of-a-type alternative-cost primitive yet.)
pub fn hand_of_emrakul() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Annihilator(1)],
        ..colossus("Hand of Emrakul", cost(&[generic(9)]), 7, 7)
    }
}

/// Bane of Bala Ged — {7} 7/5. Whenever it attacks, defending player exiles
/// two permanents they control (the affected player chooses).
pub fn bane_of_bala_ged() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::{PlayerRef, Value};
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::PlayerExilesPermanents {
            who: PlayerRef::DefendingPlayer,
            count: Value::Const(2),
            filter: SelectionRequirement::Permanent,
        })],
        ..colossus("Bane of Bala Ged", cost(&[generic(7)]), 7, 5)
    }
}

/// Birthing Hulk — {6}{G} 5/4 Eldrazi Drone. Devoid; ETB create two Eldrazi
/// Scions. (Its Awaken {7}{G} alt-cast is dropped.)
pub fn birthing_hulk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_mint_token(eldrazi_scion_token(), 2)],
        ..drone("Birthing Hulk", cost(&[generic(6), g()]), 5, 4)
    }
}

/// Drowner of Hope — {5}{U} 5/5 Eldrazi. Devoid; ETB create two Eldrazi
/// Scions. Sacrifice an Eldrazi Scion: tap target creature.
pub fn drowner_of_hope() -> CardDefinition {
    use crate::card::{ActivatedAbility, CreatureType, SelectionRequirement};
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![etb_mint_token(eldrazi_scion_token(), 2)],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((
                SelectionRequirement::HasCreatureType(CreatureType::Scion),
                1,
            )),
            effect: Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            ..Default::default()
        }],
        ..colossus("Drowner of Hope", cost(&[generic(5), u()]), 5, 5)
    }
}

/// Kozilek's Shrieker — {2}{B} 3/2 Eldrazi Drone. Devoid; {C}: +1/+0 and gains
/// menace until end of turn.
pub fn kozileks_shrieker() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{Duration, Selector, Value};
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::colorless(1)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..drone("Kozilek's Shrieker", cost(&[generic(2), b()]), 3, 2)
    }
}

/// Sifter of Skulls — {3}{B} 4/3 Eldrazi. Devoid; whenever another nontoken
/// creature you control dies, create a 1/1 Eldrazi Scion.
pub fn sifter_of_skulls() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, SelectionRequirement, TriggeredAbility};
    use crate::effect::{PlayerRef, Predicate, Selector, Value};
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::NotToken
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: eldrazi_scion_token(),
            },
        }],
        ..colossus("Sifter of Skulls", cost(&[generic(3), b()]), 4, 3)
    }
}

/// Pawn of Ulamog — {1}{B}{B} 2/2 Vampire Shaman. Whenever this or another
/// nontoken creature you control dies, create a 0/1 Eldrazi Spawn. (The "may"
/// collapses to always.)
pub fn pawn_of_ulamog() -> CardDefinition {
    use crate::card::{CreatureType, EventKind, EventScope, EventSpec, SelectionRequirement, TriggeredAbility};
    use crate::effect::{PlayerRef, Predicate, Selector, Value};
    CardDefinition {
        name: "Pawn of Ulamog",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::NotToken,
                }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: eldrazi_spawn_token(),
            },
        }],
        ..Default::default()
    }
}

/// Vestige of Emrakul — {3}{R} 3/4 Eldrazi Drone. Devoid, Trample.
pub fn vestige_of_emrakul() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::Trample],
        ..drone("Vestige of Emrakul", cost(&[generic(3), r()]), 3, 4)
    }
}

/// Stalking Drone — {1}{G} 2/2 Eldrazi Drone. Devoid; {C}: +1/+2 until end of
/// turn, once each turn.
pub fn stalking_drone() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{Duration, Selector, Value};
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::colorless(1)]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..drone("Stalking Drone", cost(&[generic(1), g()]), 2, 2)
    }
}

/// Nettle Drone — {2}{R} 3/1 Eldrazi Drone. Devoid; {T}: deal 1 damage to each
/// opponent. Whenever you cast a colorless spell, untap it.
pub fn nettle_drone() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::{cast_colorless, each_opponent};
    use crate::effect::{Selector, Value};
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: each_opponent(), amount: Value::Const(1) },
            ..Default::default()
        }],
        triggered_abilities: vec![cast_colorless(Effect::Untap {
            what: Selector::This,
            up_to: None,
        })],
        ..drone("Nettle Drone", cost(&[generic(2), r()]), 3, 1)
    }
}

/// Ruination Guide — {2}{U} 3/2 Eldrazi Drone. Devoid, Ingest; other colorless
/// creatures you control get +1/+0. (The Devoid-aware Colorless filter means
/// the anthem reaches Devoid creatures with colored pips.)
pub fn ruination_guide() -> CardDefinition {
    use crate::card::{SelectionRequirement, StaticAbility};
    use crate::effect::{Selector, StaticEffect};
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![ingest()],
        static_abilities: vec![StaticAbility {
            description: "Other colorless creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::Colorless)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        ..drone("Ruination Guide", cost(&[generic(2), u()]), 3, 2)
    }
}

/// Dominator Drone — {2}{B} 3/2 Eldrazi Drone. Devoid, Ingest; ETB, if you
/// control another colorless creature, target opponent loses 2 life.
pub fn dominator_drone() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::shortcut::etb;
    use crate::effect::{PlayerRef, Predicate, Selector, Value};
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![
            ingest(),
            etb(Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::Colorless)
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    n: Value::Const(1),
                },
                then: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            }),
        ],
        ..drone("Dominator Drone", cost(&[generic(2), b()]), 3, 2)
    }
}

/// Blinding Drone — {1}{U} 1/3 Eldrazi Drone. Devoid; {C}, {T}: tap target
/// creature.
pub fn blinding_drone() -> CardDefinition {
    use crate::card::{ActivatedAbility, SelectionRequirement};
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::colorless(1)]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            ..Default::default()
        }],
        ..drone("Blinding Drone", cost(&[generic(1), u()]), 1, 3)
    }
}

/// Kozilek's Translator — {4}{B} 3/5 Eldrazi Drone. Devoid; "Pay 1 life:
/// Add {C}." Activate only once each turn.
pub fn kozileks_translator() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{ManaPayload, PlayerRef, Value};
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            once_per_turn: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..drone("Kozilek's Translator", cost(&[generic(4), b()]), 3, 5)
    }
}

/// Flayer Drone — {1}{B}{R} 3/1 Eldrazi Drone. Devoid, First strike; whenever
/// another colorless creature you control enters, target opponent loses 1 life.
pub fn flayer_drone() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, SelectionRequirement, TriggeredAbility};
    use crate::effect::{PlayerRef, Predicate, Selector, Value};
    let drain = TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::Colorless)
                    .and(SelectionRequirement::OtherThanSource),
            }),
        effect: Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        },
    };
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::FirstStrike],
        triggered_abilities: vec![drain],
        ..drone("Flayer Drone", cost(&[generic(1), b(), r()]), 3, 1)
    }
}

/// Kozilek's Sentinel — {1}{R} 1/4 Eldrazi Drone. Devoid; whenever you cast a
/// colorless spell, it gets +1/+0 until end of turn.
pub fn kozileks_sentinel() -> CardDefinition {
    use crate::effect::shortcut::cast_colorless;
    use crate::effect::{Duration, Selector, Value};
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![cast_colorless(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..drone("Kozilek's Sentinel", cost(&[generic(1), r()]), 1, 4)
    }
}

/// Spawnsire of Ulamog — {10} 7/11. Annihilator 1; {4}: create two Eldrazi
/// Spawn. (The {20} "cast Eldrazi from outside the game" ability is dropped —
/// no wish/sideboard-cast primitive.)
pub fn spawnsire_of_ulamog() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{PlayerRef, Value};
    CardDefinition {
        keywords: vec![Keyword::Annihilator(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: eldrazi_spawn_token(),
            },
            ..Default::default()
        }],
        ..colossus("Spawnsire of Ulamog", cost(&[generic(10)]), 7, 11)
    }
}

/// Matter Reshaper — {2}{C} 3/2 Eldrazi. Dies → reveal top card; if it's a
/// permanent with mana value 3 or less, put it onto the battlefield, otherwise
/// into your hand.
pub fn matter_reshaper() -> CardDefinition {
    use crate::effect::{PlayerRef, Value};
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::RevealTopPutPermanentMvElseHand {
            who: PlayerRef::You,
            max_mv: Value::Const(3),
        })],
        ..colossus("Matter Reshaper", cost(&[generic(2), crate::mana::colorless(1)]), 3, 2)
    }
}

/// Eldrazi Processor body shared by the process-matters creatures below.
fn processor(name: &'static str, c: crate::mana::ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Processor],
            ..Default::default()
        },
        power: p,
        toughness: t,
        keywords: vec![Keyword::Devoid],
        ..Default::default()
    }
}

/// Wasteland Strangler — {2}{B} 3/2 Eldrazi Processor. Devoid. ETB: process
/// one → if you do, target creature gets -3/-3 until end of turn.
pub fn wasteland_strangler() -> CardDefinition {
    use crate::effect::shortcut::{etb, pump_target};
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Process {
            count: 1,
            then: Box::new(pump_target(-3, -3)),
        })],
        ..processor("Wasteland Strangler", cost(&[generic(2), b()]), 3, 2)
    }
}

/// Mind Raker — {3}{B} 3/3 Eldrazi Processor. Devoid. ETB: process one → if
/// you do, each opponent discards a card.
pub fn mind_raker() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::effect::{PlayerRef, Selector, Value};
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Process {
            count: 1,
            then: Box::new(Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            }),
        })],
        ..processor("Mind Raker", cost(&[generic(3), b()]), 3, 3)
    }
}

/// Blight Herder — {5} 4/5 Eldrazi Processor. Cast trigger: process two → if
/// you do, create three 1/1 Eldrazi Scion tokens.
pub fn blight_herder() -> CardDefinition {
    use crate::effect::{PlayerRef, Value};
    CardDefinition {
        triggered_abilities: vec![on_cast(Effect::Process {
            count: 2,
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: eldrazi_scion_token(),
            }),
        })],
        keywords: vec![],
        ..processor("Blight Herder", cost(&[generic(5)]), 4, 5)
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
