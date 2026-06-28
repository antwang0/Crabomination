use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword,
    SelectionRequirement, Selector, Subtypes, Value,
};
use crate::effect::Duration;
use crate::effect::shortcut::target_filtered;
use crate::mana::{cost, g, generic, u, w};

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

/// Pride of the Clouds — {W}{U} 1/1 Elemental Cat with flying. Gets +1/+1 for
/// each other flyer (approximated as flyers you control). Forecast — {2}{W}{U}
/// from hand, once each turn in your upkeep: create a 1/1 white-and-blue Bird
/// with flying.
pub fn pride_of_the_clouds() -> CardDefinition {
    use crate::card::{DynamicPt, TokenDefinition};
    use crate::effect::{PlayerRef, Predicate};
    use crate::mana::Color;
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Pride of the Clouds",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Cat],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some(DynamicPt::BasePlusOtherFlyersControlled { base: 1 }),
        // CR 702.56 — Forecast: hand-activated, once each turn, only in your
        // upkeep (gated via the `condition` predicate; the per-turn budget
        // rides the hand once-per-turn path).
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w(), u()]),
            from_hand: true,
            once_per_turn: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::CurrentStepIs(TurnStep::Upkeep),
            ])),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Bird".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::White, Color::Blue],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
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
