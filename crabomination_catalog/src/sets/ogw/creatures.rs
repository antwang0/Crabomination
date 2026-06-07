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

/// Murk Strider — {3}{U} 3/2 Eldrazi Processor. Devoid. ETB: process one → if
/// you do, return target creature to its owner's hand.
pub fn murk_strider() -> CardDefinition {
    use crate::effect::shortcut::{etb, return_target_to_hand};
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Process {
            count: 1,
            then: Box::new(return_target_to_hand()),
        })],
        ..processor("Murk Strider", cost(&[generic(3), u()]), 3, 2)
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

/// Canopy Gorger — {4}{G}{G} 6/5 Wurm.
pub fn canopy_gorger() -> CardDefinition {
    use crate::card::CreatureType;
    CardDefinition {
        name: "Canopy Gorger",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
        power: 6,
        toughness: 5,
        ..Default::default()
    }
}

/// Mammoth Spider — {4}{G} 3/5 Spider. Reach.
pub fn mammoth_spider() -> CardDefinition {
    use crate::card::CreatureType;
    CardDefinition {
        name: "Mammoth Spider",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        ..Default::default()
    }
}

/// Murasa Ranger — {3}{G} 3/3 Human Warrior Ranger. Landfall — you may pay
/// {3}{G}; if you do, put two +1/+1 counters on this creature.
pub fn murasa_ranger() -> CardDefinition {
    use crate::card::{CounterType, CreatureType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::{Selector, Value};
    CardDefinition {
        name: "Murasa Ranger",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ranger],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {3}{G} to put two +1/+1 counters on Murasa Ranger?".into(),
                mana_cost: cost(&[generic(3), g()]),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Snapping Gnarlid — {1}{G} 2/2 Beast. Landfall — gets +1/+1 EOT when a land
/// you control enters.
pub fn snapping_gnarlid() -> CardDefinition {
    landfall_pump("Snapping Gnarlid", cost(&[generic(1), g()]), 2, 2, 1, 1)
}

/// Territorial Baloth — {4}{G} 4/4 Beast. Landfall — gets +2/+2 EOT.
pub fn territorial_baloth() -> CardDefinition {
    landfall_pump("Territorial Baloth", cost(&[generic(4), g()]), 4, 4, 2, 2)
}

/// Shared body for the Beast landfall-pumps above.
fn landfall_pump(
    name: &'static str, c: crate::mana::ManaCost, p: i32, t: i32, dp: i32, dt: i32,
) -> CardDefinition {
    use crate::card::{CreatureType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::{Duration, Selector, Value};
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: p,
        toughness: t,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(dp),
                toughness: Value::Const(dt),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Oran-Rief Invoker — {1}{G} 2/2 Human Shaman. {8}: +5/+5 and gains trample EOT.
pub fn oran_rief_invoker() -> CardDefinition {
    use crate::card::{ActivatedAbility, CreatureType};
    use crate::effect::{Duration, Selector, Value};
    CardDefinition {
        name: "Oran-Rief Invoker",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(5),
                    toughness: Value::Const(5),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lavastep Raider — {R} 1/2 Goblin Warrior. {2}{R}: +2/+0 until end of turn.
pub fn lavastep_raider() -> CardDefinition {
    use crate::card::{ActivatedAbility, CreatureType};
    use crate::effect::{Duration, Selector, Value};
    CardDefinition {
        name: "Lavastep Raider",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cliffside Lookout — {W} 1/1 Kor Scout Ally. {4}{W}: creatures you control
/// get +1/+1 until end of turn.
pub fn cliffside_lookout() -> CardDefinition {
    use crate::card::{ActivatedAbility, CreatureType};
    use crate::effect::shortcut::each_your_creature;
    use crate::effect::{Duration, Selector, Value};
    CardDefinition {
        name: "Cliffside Lookout",
        cost: cost(&[crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Scout, CreatureType::Ally],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), crate::mana::w()]),
            effect: Effect::ForEach {
                selector: each_your_creature(),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mountain Yeti — {2}{R}{R} 3/3 Yeti. Mountainwalk, Protection from white.
pub fn mountain_yeti() -> CardDefinition {
    use crate::card::{CreatureType, LandType};
    use crate::mana::Color;
    CardDefinition {
        name: "Mountain Yeti",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Yeti], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Landwalk(LandType::Mountain), Keyword::Protection(Color::White)],
        ..Default::default()
    }
}

/// Wasteland Scorpion — {2}{B} 2/2 Scorpion. Deathtouch, Cycling {2}.
pub fn wasteland_scorpion() -> CardDefinition {
    use crate::card::CreatureType;
    use crate::mana::ManaCost;
    CardDefinition {
        name: "Wasteland Scorpion",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Scorpion],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch, Keyword::Cycling(ManaCost::new(vec![generic(2)]))],
        ..Default::default()
    }
}

/// Expedition Raptor — {3}{W}{W} 2/2 Bird. Flying; ETB support 2.
pub fn expedition_raptor() -> CardDefinition {
    use crate::card::CreatureType;
    use crate::effect::shortcut::{etb, support};
    CardDefinition {
        name: "Expedition Raptor",
        cost: cost(&[generic(3), crate::mana::w(), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(support(2))],
        ..Default::default()
    }
}

/// Felidar Cub — {1}{W} 2/2 Cat Beast. Sacrifice this creature: destroy target
/// enchantment.
pub fn felidar_cub() -> CardDefinition {
    use crate::card::{ActivatedAbility, CreatureType, SelectionRequirement};
    CardDefinition {
        name: "Felidar Cub",
        cost: cost(&[generic(1), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(SelectionRequirement::Enchantment),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tajuru Pathwarden — {4}{G} 5/4 Elf Warrior Ally. Vigilance, Trample.
pub fn tajuru_pathwarden() -> CardDefinition {
    use crate::card::CreatureType;
    CardDefinition {
        name: "Tajuru Pathwarden",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior, CreatureType::Ally],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        ..Default::default()
    }
}

/// Courier Griffin — {3}{W} 2/3 Griffin. Flying; ETB gain 2 life.
pub fn courier_griffin() -> CardDefinition {
    use crate::card::CreatureType;
    use crate::effect::shortcut::etb;
    use crate::effect::{Selector, Value};
    CardDefinition {
        name: "Courier Griffin",
        cost: cost(&[generic(3), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Griffin],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(2) })],
        ..Default::default()
    }
}

/// Vampire Envoy — {2}{B} 1/4 Vampire Cleric Ally. Flying; whenever this
/// creature becomes tapped, you gain 1 life.
pub fn vampire_envoy() -> CardDefinition {
    use crate::card::{CreatureType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::{Selector, Value};
    CardDefinition {
        name: "Vampire Envoy",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric, CreatureType::Ally],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Eldrazi Displacer — {2}{W} 3/3 Eldrazi. Devoid; {2}{C}: exile another
/// target creature, then return it to the battlefield tapped under its
/// owner's control.
pub fn eldrazi_displacer() -> CardDefinition {
    use crate::card::{ActivatedAbility, SelectionRequirement};
    use crate::effect::{PlayerRef, Selector, ZoneDest};
    CardDefinition {
        name: "Eldrazi Displacer",
        cost: cost(&[generic(2), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Devoid],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), crate::mana::colorless(1)]),
            effect: Effect::Seq(vec![
                Effect::Exile {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                },
                Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                        tapped: true,
                    },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cliffhaven Vampire — {2}{W}{B} 2/4 Vampire Warrior Ally. Flying; whenever
/// you gain life, each opponent loses 1 life.
pub fn cliffhaven_vampire() -> CardDefinition {
    use crate::card::{CreatureType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::{PlayerRef, Selector, Value};
    CardDefinition {
        name: "Cliffhaven Vampire",
        cost: cost(&[generic(2), crate::mana::w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warrior, CreatureType::Ally],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Kor Scythemaster — {2}{W} 3/1 Kor Soldier Ally. Has first strike while
/// attacking.
pub fn kor_scythemaster() -> CardDefinition {
    use crate::card::{CreatureType, SelectionRequirement, StaticAbility};
    use crate::effect::{Predicate, Selector, StaticEffect};
    CardDefinition {
        name: "Kor Scythemaster",
        cost: cost(&[generic(2), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Soldier, CreatureType::Ally],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Has first strike as long as it's attacking.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: SelectionRequirement::IsAttacking,
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike],
            },
        }],
        ..Default::default()
    }
}

/// Affa Protector — {2}{W} 1/4 Human Soldier Ally. Vigilance.
pub fn affa_protector() -> CardDefinition {
    use crate::card::CreatureType;
    CardDefinition {
        name: "Affa Protector",
        cost: cost(&[generic(2), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Ally],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    }
}

/// Ghostly Sentinel — {4}{W} 3/3 Kor Spirit. Flying, Vigilance.
pub fn ghostly_sentinel() -> CardDefinition {
    use crate::card::CreatureType;
    CardDefinition {
        name: "Ghostly Sentinel",
        cost: cost(&[generic(4), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..Default::default()
    }
}

/// Saddleback Lagac — {3}{G} 3/1 Lizard. ETB support 2 (a +1/+1 counter on
/// each of up to two other target creatures).
pub fn saddleback_lagac() -> CardDefinition {
    use crate::card::CreatureType;
    use crate::effect::shortcut::{etb, support};
    CardDefinition {
        name: "Saddleback Lagac",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![etb(support(2))],
        ..Default::default()
    }
}

/// Loam Larva — {1}{G} 1/3 Insect. ETB you may search your library for a basic
/// land card and put it on top (shuffling first).
pub fn loam_larva() -> CardDefinition {
    use crate::card::{CreatureType, SelectionRequirement};
    use crate::effect::shortcut::etb;
    use crate::effect::{LibraryPosition, PlayerRef, ZoneDest};
    CardDefinition {
        name: "Loam Larva",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for a basic land and put it on top?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            }),
        })],
        ..Default::default()
    }
}

/// Stormrider Spirit — {4}{U} 3/3 Spirit. Flash, Flying.
pub fn stormrider_spirit() -> CardDefinition {
    use crate::card::CreatureType;
    CardDefinition {
        name: "Stormrider Spirit",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        ..Default::default()
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

/// Thought-Knot Seer — {3}{C} 4/4 Eldrazi. ETB: target opponent reveals their
/// hand, you choose a nonland card and exile it. LTB: that player draws a card.
pub fn thought_knot_seer() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, SelectionRequirement,
        TriggeredAbility};
    use crate::effect::{PlayerRef, Selector, Value};
    CardDefinition {
        name: "Thought-Knot Seer",
        cost: cost(&[generic(3), crate::mana::colorless(1)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::ExileChosenFromHand {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::Const(1),
                    filter: SelectionRequirement::Nonland,
                },
            },
            // "That player draws a card" — modeled as each opponent (exact in 1v1).
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Sky Scourer — {1}{B} 1/2 Eldrazi Drone. Devoid, Flying; whenever you cast a
/// colorless spell, it gets +1/+0 until end of turn.
pub fn sky_scourer() -> CardDefinition {
    use crate::effect::shortcut::cast_colorless;
    use crate::effect::{Duration, Selector, Value};
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::Flying],
        triggered_abilities: vec![cast_colorless(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..drone("Sky Scourer", cost(&[crate::mana::generic(1), b()]), 1, 2)
    }
}

/// Tajuru Stalwart — {2}{G} 0/1 Elf Scout Ally. Converge — enters with a +1/+1
/// counter for each color of mana spent to cast it.
pub fn tajuru_stalwart() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::Value;
    CardDefinition {
        name: "Tajuru Stalwart",
        cost: cost(&[crate::mana::generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Scout, CreatureType::Ally],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ConvergedValue)),
        ..Default::default()
    }
}

/// Kozilek's Pathfinder — {6} 5/5 Eldrazi. {C}: Target creature can't block
/// this creature this turn.
pub fn kozileks_pathfinder() -> CardDefinition {
    use crate::card::{ActivatedAbility, SelectionRequirement};
    CardDefinition {
        name: "Kozilek's Pathfinder",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 5,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::colorless(1)]),
            effect: Effect::CantBlockSourceThisTurn {
                target: target_filtered(SelectionRequirement::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Visions of Brutality — {1}{B} Devoid Aura. Enchanted creature can't block,
/// and whenever it deals combat damage, its controller loses that much life.
pub fn visions_of_brutality() -> CardDefinition {
    use crate::card::{EquipBonus, EnchantmentSubtype, EventKind, EventScope, EventSpec,
        SelectionRequirement, TriggeredAbility};
    use crate::effect::{Selector, Value};
    CardDefinition {
        name: "Visions of Brutality",
        cost: cost(&[crate::mana::generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Devoid],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 0,
            toughness: 0,
            keywords: vec![Keyword::CantBlock],
            scale: None,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::LoseLife { who: Selector::You, amount: Value::TriggerEventAmount },
            }],
        }),
        ..Default::default()
    }
}

/// Akoum Stonewaker — {1}{R} 2/1 Human Shaman. Landfall — when a land you
/// control enters, you may pay {2}{R} to create a 3/1 red Elemental with
/// trample and haste, exiled at the next end step.
pub fn akoum_stonewaker() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TokenDefinition, TriggeredAbility};
    use crate::effect::{DelayedTriggerKind, Selector, Value};
    use crabomination_base::mana::Color;
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Trample, Keyword::Haste],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Akoum Stonewaker",
        cost: cost(&[crate::mana::generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {2}{R}: create a 3/1 Elemental (exiled at end of turn)".into(),
                mana_cost: cost(&[crate::mana::generic(2), r()]),
                body: Box::new(Effect::Seq(vec![
                    Effect::CreateToken {
                        who: crate::effect::PlayerRef::You,
                        count: Value::Const(1),
                        definition: elemental,
                    },
                    Effect::DelayUntil {
                        kind: DelayedTriggerKind::NextEndStep,
                        body: Box::new(Effect::Exile { what: Selector::LastCreatedToken }),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Loathsome Catoblepas — {5}{B} 3/3 Beast. {2}{G}: must be blocked this turn
/// if able. When it dies, target creature an opponent controls gets -3/-3 EOT.
pub fn loathsome_catoblepas() -> CardDefinition {
    use crate::card::{ActivatedAbility, Keyword, SelectionRequirement};
    use crate::effect::{Duration, Selector, Value};
    CardDefinition {
        name: "Loathsome Catoblepas",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::MustBeBlocked,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![on_dies(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Gravity Negator — {3}{U} 2/3 Eldrazi Drone. Devoid, Flying. Whenever it
/// attacks, you may pay {C}; if you do, another target creature gains flying
/// until end of turn.
pub fn gravity_negator() -> CardDefinition {
    use crate::card::{Keyword, SelectionRequirement};
    use crate::effect::Duration;
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::MayPay {
            description: "Pay {C}: another target creature gains flying".into(),
            mana_cost: cost(&[crate::mana::colorless(1)]),
            body: Box::new(Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            }),
        })],
        ..drone("Gravity Negator", cost(&[generic(3), u()]), 2, 3)
    }
}

/// Sea Gate Wreckage — Land. {T}: Add {C}. {2}{C}, {T}: Draw a card — activate
/// only if you have no cards in hand.
pub fn sea_gate_wreckage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{Predicate, PlayerRef, Selector, Value};
    CardDefinition {
        name: "Sea Gate Wreckage",
        cost: crate::mana::ManaCost::default(),
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), crate::mana::colorless(1)]),
                condition: Some(Predicate::ValueEquals(
                    Value::HandSizeOf(PlayerRef::You),
                    Value::Const(0),
                )),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Spawning Bed — Land. {T}: Add {C}. {6}, {T}, Sacrifice this land: Create
/// three 1/1 Eldrazi Scion tokens.
pub fn spawning_bed() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{PlayerRef, Value};
    CardDefinition {
        name: "Spawning Bed",
        cost: crate::mana::ManaCost::default(),
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(6)]),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                    definition: eldrazi_scion_token(),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Cinder Barrens — Land. Enters tapped; {T}: Add {B} or {R}.
pub fn cinder_barrens() -> CardDefinition {
    use crate::card::{ActivatedAbility, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::{ManaPayload, PlayerRef, Selector, Value};
    let tap_for = |color| ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColor(color, Value::Const(1)),
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Cinder Barrens",
        cost: crate::mana::ManaCost::default(),
        card_types: vec![CardType::Land],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap { what: Selector::This },
        }],
        activated_abilities: vec![tap_for(crate::mana::Color::Black), tap_for(crate::mana::Color::Red)],
        ..Default::default()
    }
}

/// Crumbling Vestige — Land. Enters tapped; ETB add one mana of any color;
/// {T}: Add {C}.
pub fn crumbling_vestige() -> CardDefinition {
    use crate::card::{ActivatedAbility, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::{ManaPayload, PlayerRef, Selector, Value};
    CardDefinition {
        name: "Crumbling Vestige",
        cost: crate::mana::ManaCost::default(),
        card_types: vec![CardType::Land],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap { what: Selector::This },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyColors(Value::Const(1)),
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Walker of the Wastes — {4}{C} 4/4 Eldrazi. Trample; gets +1/+1 for each
/// land you control named Wastes.
pub fn walker_of_the_wastes() -> CardDefinition {
    use crate::card::{SelectionRequirement, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Walker of the Wastes",
        cost: cost(&[generic(4), crate::mana::colorless(1)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Gets +1/+1 for each land you control named Wastes.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: SelectionRequirement::HasName("Wastes".into())
                    .and(SelectionRequirement::ControlledByYou),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..Default::default()
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
