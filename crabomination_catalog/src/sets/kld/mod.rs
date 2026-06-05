//! Kaladesh (KLD/AER) energy cards.
//!
//! Showcases the engine's energy ({E}) resource: `Effect::AddEnergy`
//! pools energy on the controller (`Player.energy`) and `Effect::PayEnergy`
//! spends it at resolution. Each card here is built on those two
//! primitives plus existing ones — no per-card energy plumbing.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement, Selector, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::{Duration, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, ManaCost};

/// {E}{E}{E}: Put a +1/+1 counter on this creature (energy-only activated
/// ability via `PayEnergy`). The ability itself is free; the player commits
/// by activating and the energy is consumed on resolution.
fn pay_energy_counter(amount: u32) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: false,
        mana_cost: ManaCost::default(),
        effect: Effect::PayEnergy {
            amount,
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        },
        once_per_turn: false,
        sorcery_speed: false,
        sac_cost: false,
        condition: None,
        life_cost: 0,
        from_graveyard: false,
        exile_self_cost: false,
        exile_other_filter: None,
        self_counter_cost_reduction: None,
        sac_other_filter: None,
        tap_other_filter: None,
        from_hand: false,
    }
}

/// Attune with Aether — {G} Sorcery. Search your library for a basic land
/// card, put it into your hand, then shuffle. You get {E}{E}.
pub fn attune_with_aether() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Attune with Aether",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::AddEnergy(Value::Const(2)),
        ]),
        ..Default::default()
    }
}

/// Rogue Refiner — {1}{G}{U} 3/2 Elf Artificer. When this enters, draw a
/// card and you get {E}{E}.
pub fn rogue_refiner() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Rogue Refiner",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::AddEnergy(Value::Const(2)),
        ]))],
        ..Default::default()
    }
}

/// Longtusk Cub — {1}{G} 2/2 Cat. Whenever this deals combat damage to a
/// player, you get {E}{E}. {E}{E}{E}: put a +1/+1 counter on this.
pub fn longtusk_cub() -> CardDefinition {
    CardDefinition {
        name: "Longtusk Cub",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddEnergy(Value::Const(2)),
        }],
        activated_abilities: vec![pay_energy_counter(3)],
        ..Default::default()
    }
}

/// Bristling Hydra — {2}{R}{G} 4/3 Hydra. When this enters, you get
/// {E}{E}{E}. {E}{E}{E}: put a +1/+1 counter on this and it gains hexproof
/// until end of turn.
pub fn bristling_hydra() -> CardDefinition {
    use crate::effect::shortcut::etb;
    let mut grow = pay_energy_counter(3);
    grow.effect = Effect::PayEnergy {
        amount: 3,
        then: Box::new(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ])),
    };
    CardDefinition {
        name: "Bristling Hydra",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hydra],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(3)))],
        activated_abilities: vec![grow],
        ..Default::default()
    }
}

/// Glint-Sleeve Siphoner — {1}{B} 1/2 Vampire Rogue with Menace. Whenever
/// it attacks, you get {E}. At the beginning of your upkeep, you may pay
/// {E}{E}; if you do, draw a card and lose 1 life.
pub fn glint_sleeve_siphoner() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Glint-Sleeve Siphoner",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            on_attack(Effect::AddEnergy(Value::Const(1))),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::PayEnergy {
                    amount: 2,
                    then: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                        Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
                    ])),
                },
            },
        ],
        ..Default::default()
    }
}

/// Aether Hub — Land. When it enters, you get {E}. {T}: Add one mana of any
/// color. (The printed "{T}: Add {C}" / "{T}, Pay {E}: Add any color" split
/// collapses to a single any-color tap — energy can't gate a mana ability
/// in this engine.)
pub fn aether_hub() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Aether Hub",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(1)))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None,
            from_hand: false,
        }],
        ..Default::default()
    }
}

/// Servant of the Conduit — {1}{G} 2/2 Elf Druid. When it enters, you get
/// {E}{E}. {T}: Add one mana of any color. (The printed "{T}, Pay {E}: Add
/// one mana of any color" collapses to a free tap — energy can't gate a
/// mana ability in this engine; the ETB energy is faithful.)
pub fn servant_of_the_conduit() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Servant of the Conduit",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(2)))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None,
            from_hand: false,
        }],
        ..Default::default()
    }
}

/// Dynavolt Tower — {3} Artifact. Whenever you cast an instant or sorcery
/// spell, you get {E}{E}. {5}, {T}, Pay {E}{E}{E}{E}{E}: deal 4 damage to
/// any target.
pub fn dynavolt_tower() -> CardDefinition {
    use crate::effect::shortcut::{magecraft, target_any};
    CardDefinition {
        name: "Dynavolt Tower",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        triggered_abilities: vec![magecraft(Effect::AddEnergy(Value::Const(2)))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(5)]),
            effect: Effect::PayEnergy {
                amount: 5,
                then: Box::new(Effect::DealDamage { to: target_any(), amount: Value::Const(4) }),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None,
            from_hand: false,
        }],
        ..Default::default()
    }
}

/// Aether Swooper — {1}{U} 1/2 Vedalken Artificer with Flying. When it
/// enters, you get {E}{E}. Whenever it attacks, you may pay {E}{E}; if you
/// do, create a 1/1 colorless Thopter artifact creature token with flying.
pub fn aether_swooper() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::effect::shortcut::{etb, on_attack};
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Aether Swooper",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::Const(2))),
            on_attack(Effect::PayEnergy {
                amount: 2,
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: thopter,
                }),
            }),
        ],
        ..Default::default()
    }
}

/// Sage of Shaila's Claim — {1}{G} 1/3 Elf Druid. When it enters, you get
/// {E}{E}{E}.
pub fn sage_of_shailas_claim() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Sage of Shaila's Claim",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(3)))],
        ..Default::default()
    }
}

/// Live Fast — {1}{B} Sorcery. You draw two cards, you lose 2 life, and you
/// get {E}{E}.
pub fn live_fast() -> CardDefinition {
    CardDefinition {
        name: "Live Fast",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            Effect::AddEnergy(Value::Const(2)),
        ]),
        ..Default::default()
    }
}

/// Highspire Infusion — {1}{G} Instant. Target creature gets +3/+3 until end
/// of turn. You get {E}{E}.
pub fn highspire_infusion() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    use crate::effect::Duration;
    CardDefinition {
        name: "Highspire Infusion",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::AddEnergy(Value::Const(2)),
        ]),
        ..Default::default()
    }
}

/// Glimmer of Genius — {3}{U} Instant. Scry 2, then draw two cards. You get
/// {E}{E}.
pub fn glimmer_of_genius() -> CardDefinition {
    CardDefinition {
        name: "Glimmer of Genius",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::AddEnergy(Value::Const(2)),
        ]),
        ..Default::default()
    }
}

/// Woodweaver's Puzzleknot — {2} Artifact. When it enters, you get {E}{E}{E}
/// and gain 3 life. {2}, Sacrifice: you get {E}{E}{E} and gain 3 life.
pub fn woodweavers_puzzleknot() -> CardDefinition {
    use crate::effect::shortcut::etb;
    let payoff = || Effect::Seq(vec![
        Effect::AddEnergy(Value::Const(3)),
        Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
    ]);
    CardDefinition {
        name: "Woodweaver's Puzzleknot",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(payoff())],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2)]),
            effect: payoff(),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None,
            from_hand: false,
        }],
        ..Default::default()
    }
}

/// Glassblower's Puzzleknot — {2} Artifact. When it enters, you get {E}{E}
/// and draw a card. {2}, Sacrifice: you get {E}{E} and draw a card.
pub fn glassblowers_puzzleknot() -> CardDefinition {
    use crate::effect::shortcut::etb;
    let payoff = || Effect::Seq(vec![
        Effect::AddEnergy(Value::Const(2)),
        Effect::Draw { who: Selector::You, amount: Value::Const(1) },
    ]);
    CardDefinition {
        name: "Glassblower's Puzzleknot",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(payoff())],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2)]),
            effect: payoff(),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None,
            from_hand: false,
        }],
        ..Default::default()
    }
}

/// Aether Poisoner — {1}{B} 1/1 Rogue with Deathtouch. When it enters, you
/// get {E}. Whenever it deals combat damage to a player, you get {E}.
pub fn aether_poisoner() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Aether Poisoner",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::Const(1))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::AddEnergy(Value::Const(1)),
            },
        ],
        ..Default::default()
    }
}

/// Aetherstream Leopard — {3}{G} 4/3 Cat. When it enters, you get {E}{E}.
/// {E}{E}{E}{E}: this creature can't be blocked this turn.
pub fn aetherstream_leopard() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::effect::Duration;
    let mut sneak = pay_energy_counter(4);
    sneak.effect = Effect::PayEnergy {
        amount: 4,
        then: Box::new(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Unblockable,
            duration: Duration::EndOfTurn,
        }),
    };
    CardDefinition {
        name: "Aetherstream Leopard",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(2)))],
        activated_abilities: vec![sneak],
        ..Default::default()
    }
}

/// Riparian Tiger — {3}{G} 5/4 Cat. When it enters, you get {E}{E}. {E}{E}:
/// this creature gains hexproof until end of turn.
pub fn riparian_tiger() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::effect::Duration;
    let mut guard = pay_energy_counter(2);
    guard.effect = Effect::PayEnergy {
        amount: 2,
        then: Box::new(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Hexproof,
            duration: Duration::EndOfTurn,
        }),
    };
    CardDefinition {
        name: "Riparian Tiger",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(2)))],
        activated_abilities: vec![guard],
        ..Default::default()
    }
}

/// Voltaic Brawler — {R}{G} 3/1 Human Warrior with Trample and Menace.
/// Whenever it attacks, you may pay {E}{E}; if you do, it gets +1/+1 until
/// end of turn.
pub fn voltaic_brawler() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    use crate::effect::Duration;
    CardDefinition {
        name: "Voltaic Brawler",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Trample, Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::PayEnergy {
            amount: 2,
            then: Box::new(Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            }),
        })],
        ..Default::default()
    }
}

/// Aetherborn Marauder — {3}{B} 2/3 Aetherborn. Whenever you get one or more
/// {E}, put two +1/+1 counters on it (`EventKind::EnergyGained`, CR 107.16).
pub fn aetherborn_marauder() -> CardDefinition {
    CardDefinition {
        name: "Aetherborn Marauder",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Aetherborn],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EnergyGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Lathnu Hellion — {2}{R} 4/4 Hellion with Haste. ETB you get {E}{E}; at
/// the beginning of your upkeep, sacrifice it unless you pay {E}{E}
/// (`Effect::PayEnergyOrElse`, CR 107.16).
pub fn lathnu_hellion() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lathnu Hellion",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hellion],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::Const(2))),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::PayEnergyOrElse {
                    amount: 2,
                    otherwise: Box::new(Effect::SacrificeSource),
                },
            },
        ],
        ..Default::default()
    }
}

/// Greenbelt Rampager — {G} 3/4 Elephant. ETB: you get {E}{E}, then pay
/// {E}{E} or return it to its owner's hand (`PayEnergyOrElse`, CR 107.16).
pub fn greenbelt_rampager() -> CardDefinition {
    use crate::effect::shortcut::etb;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Greenbelt Rampager",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddEnergy(Value::Const(2)),
            Effect::PayEnergyOrElse {
                amount: 2,
                otherwise: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                }),
            },
        ]))],
        ..Default::default()
    }
}

/// Thriving Rhino — {3}{G} 3/3 Rhino. Whenever it attacks, you may pay
/// {E}{E}; if you do, it gets +2/+2 until end of turn.
pub fn thriving_rhino() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    use crate::effect::Duration;
    CardDefinition {
        name: "Thriving Rhino",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::PayEnergy {
            amount: 2,
            then: Box::new(Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
        })],
        ..Default::default()
    }
}

/// Every KLD factory, for snapshot name→factory registration.
pub fn all_kld_card_factories() -> &'static [crate::CardFactory] {
    &[
        aetherborn_marauder,
        lathnu_hellion,
        greenbelt_rampager,
        thriving_rhino,
        attune_with_aether,
        rogue_refiner,
        longtusk_cub,
        bristling_hydra,
        glint_sleeve_siphoner,
        aether_hub,
        servant_of_the_conduit,
        dynavolt_tower,
        aether_swooper,
        sage_of_shailas_claim,
        live_fast,
        highspire_infusion,
        glimmer_of_genius,
        woodweavers_puzzleknot,
        glassblowers_puzzleknot,
        aether_poisoner,
        aetherstream_leopard,
        riparian_tiger,
        voltaic_brawler,
    ]
}