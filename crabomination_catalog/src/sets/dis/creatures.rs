use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword,
    SelectionRequirement, Selector, Subtypes, Value,
};
use crate::effect::Duration;
use crate::effect::shortcut::{forecast, target_filtered};
use crate::mana::{cost, g, generic, r, u, w, Color};

/// Azorius First-Wing — {1}{W}{U} 2/2 Bird Soldier Flying
pub fn azorius_first_wing() -> CardDefinition {
    CardDefinition {
        name: "Azorius First-Wing",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Griffin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Aquastrand Spider — {1}{G/U} 0/0 Spider, Reach, Graft 2.
pub fn aquastrand_spider() -> CardDefinition {
    CardDefinition {
        name: "Aquastrand Spider",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        keywords: vec![Keyword::Reach],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

/// Plaxcaster Frogling — {2}{G/U} 0/0 Frog Beast, Graft 3.
pub fn plaxcaster_frogling() -> CardDefinition {
    CardDefinition {
        name: "Plaxcaster Frogling",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Mutant],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

/// Cytoplast Root-Kin — {2}{G}{G} 0/0 Mutant, Graft 4. ETB puts a +1/+1
/// counter on each other creature you control that already has one.
pub fn cytoplast_root_kin() -> CardDefinition {
    CardDefinition {
        name: "Cytoplast Root-Kin",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource)
                        .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            }),
            crate::effect::shortcut::graft(),
        ],
        ..Default::default()
    }
}

/// Simic Initiate — {G/U} 0/0 Merfolk Wizard, Graft 1.
pub fn simic_initiate() -> CardDefinition {
    CardDefinition {
        name: "Simic Initiate",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mutant],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

/// Vigean Graftmage — {1}{G/U} 0/0 Vedalken Wizard, Graft 2.
/// "{1}{U}: Untap target creature with a +1/+1 counter on it."
pub fn vigean_graftmage() -> CardDefinition {
    CardDefinition {
        name: "Vigean Graftmage",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Wizard],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Untap {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                up_to: None,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

/// Helium Squirter — {4}{G/U} 0/0 Mutant, Graft 3.
/// "{1}: Target creature with a +1/+1 counter on it gains flying until end
/// of turn."
pub fn helium_squirter() -> CardDefinition {
    CardDefinition {
        name: "Helium Squirter",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

/// Assault Zeppelid — {2}{G}{U} 3/3 Beast with flying and trample.
pub fn assault_zeppelid() -> CardDefinition {
    CardDefinition {
        name: "Assault Zeppelid",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        ..Default::default()
    }
}

/// Sky Hussar — {3}{W}{U} 4/3 Human Knight with flying. When it enters, untap
/// all creatures you control. Forecast — Tap two untapped white and/or blue
/// creatures you control: Draw a card.
pub fn sky_hussar() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    let wu_creature = SelectionRequirement::Creature
        .and(SelectionRequirement::ControlledByYou)
        .and(
            SelectionRequirement::HasColor(Color::White)
                .or(SelectionRequirement::HasColor(Color::Blue)),
        );
    CardDefinition {
        name: "Sky Hussar",
        cost: cost(&[generic(3), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                up_to: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((wu_creature, 2)),
            ..forecast(cost(&[]), Effect::Draw { who: Selector::You, amount: Value::ONE })
        }],
        ..Default::default()
    }
}

/// Stalking Vengeance — {5}{R}{R} 5/5 Avatar with haste. Whenever another
/// creature you control dies, it deals damage equal to its power to any target.
/// (Modeled as each opponent — faithful in 1v1; the dead creature's power
/// carries via its die snapshot, CR 603.10.)
pub fn stalking_vengeance() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::PlayerRef;
    CardDefinition {
        name: "Stalking Vengeance",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Avatar], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::DealDamageEqualToPower {
                source: Selector::TriggerSource,
                target: Selector::Player(PlayerRef::EachOpponent),
            },
        }],
        ..Default::default()
    }
}

/// Kill-Suit Cultist — {R} 1/1 Goblin Berserker. Attacks each combat if able.
/// `{B}, Sacrifice this creature: The next time damage would be dealt to target
/// creature this turn, destroy that creature instead.`
pub fn kill_suit_cultist() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::mana::b;
    CardDefinition {
        name: "Kill-Suit Cultist",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Berserker],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::MustAttack],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_cost: true,
            effect: Effect::ReplaceNextDamageWithDestroy {
                target: target_filtered(SelectionRequirement::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Azorius Herald — {2}{W} 2/1 Spirit. Can't be blocked. When it enters, gain 4
/// life; and sacrifice it unless {U} was spent to cast it.
pub fn azorius_herald() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Predicate, TriggeredAbility};
    CardDefinition {
        name: "Azorius Herald",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::SourceCastWithColorSpent { color: Color::Blue, at_least: 1 },
                    then: Box::new(Effect::Noop),
                    else_: Box::new(Effect::SacrificeSource),
                },
            },
        ],
        ..Default::default()
    }
}
