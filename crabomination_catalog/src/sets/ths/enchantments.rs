use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::game::types::TurnStep;
use crate::effect::{Duration, PlayerRef, PlayerStaticTarget, shortcut::etb, shortcut::target_filtered};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Hopeful Eidolon — {W} Enchantment Creature — Spirit 1/1 Lifelink.
/// Bestow {3}{W} (CR 702.103): as an Aura it grants +1/+1 and lifelink.
pub fn hopeful_eidolon() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Hopeful Eidolon",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        bestow: Some(cost(&[generic(3), w()])),
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Lifelink], scale: None, triggered_abilities: vec![] }),
        ..Default::default()
    }
}

/// Gray Merchant of Asphodel — {3}{B}{B} Creature — Zombie 2/4. ETB: each
/// opponent loses life equal to your devotion to black and you gain that
/// much. Uses the new `Value::DevotionTo` primitive (CR 700.5).
pub fn gray_merchant_of_asphodel() -> CardDefinition {
    CardDefinition {
        name: "Gray Merchant of Asphodel",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::DevotionTo(vec![Color::Black]),
        })],
        ..Default::default()
    }
}

/// Shared god-frame helper: a Legendary Enchantment Creature — God that
/// isn't a creature unless its controller's devotion to `colors` ≥ 5
/// (CR 700.5). Indestructible.
fn god(
    name: &'static str,
    cost_: crate::mana::ManaCost,
    colors: Vec<Color>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost_,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::God],
            ..Default::default()
        },
        power,
        toughness,
        keywords: vec![Keyword::Indestructible],
        static_abilities: vec![StaticAbility {
            description: "As long as your devotion to its color is less than five, this isn't a creature.",
            effect: StaticEffect::NotCreatureWhileDevotionBelow { colors, threshold: 5 },
        }],
        ..Default::default()
    }
}

/// Nylea, God of the Hunt — {3}{G} 6/6. Indestructible God; isn't a
/// creature while devotion to green < 5. Other creatures you control get
/// +2/+0. {3}{G}: Target creature gains trample until end of turn.
pub fn nylea_god_of_the_hunt() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "As long as your devotion to green is less than five, Nylea isn't a creature.",
                effect: StaticEffect::NotCreatureWhileDevotionBelow {
                    colors: vec![Color::Green],
                    threshold: 5,
                },
            },
            StaticAbility {
                description: "Other creatures you control get +2/+0.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    power: 2,
                    toughness: 0,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..god("Nylea, God of the Hunt", cost(&[generic(3), g()]), vec![Color::Green], 6, 6)
    }
}

/// Thassa, God of the Sea — {2}{U} 5/5. Indestructible God; isn't a
/// creature while devotion to blue < 5. At the beginning of your upkeep,
/// scry 1. {1}{U}: Target creature you control can't be blocked this turn.
pub fn thassa_god_of_the_sea() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..god("Thassa, God of the Sea", cost(&[generic(2), u()]), vec![Color::Blue], 5, 5)
    }
}

/// Erebos, God of the Dead — {3}{B} 5/7. Indestructible God; isn't a
/// creature while devotion to black < 5. You can't gain life. {1}{B}, Pay
/// 2 life, Sacrifice another creature: Draw a card.
pub fn erebos_god_of_the_dead() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "As long as your devotion to black is less than five, Erebos isn't a creature.",
                effect: StaticEffect::NotCreatureWhileDevotionBelow {
                    colors: vec![Color::Black],
                    threshold: 5,
                },
            },
            StaticAbility {
                description: "You can't gain life.",
                effect: StaticEffect::PlayerCannotGainLife { target: PlayerStaticTarget::Controller },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(1), b()]),
            life_cost: 2,
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..god("Erebos, God of the Dead", cost(&[generic(3), b()]), vec![Color::Black], 5, 7)
    }
}

/// Your-creatures static-anthem helper for the Theros "god weapon"
/// Legendary Enchantments: one `StaticAbility` over `Creature ∧
/// ControlledByYou`.
fn god_weapon(
    name: &'static str,
    cost_: crate::mana::ManaCost,
    description: &'static str,
    effect: StaticEffect,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost_,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility { description, effect }],
        ..Default::default()
    }
}

fn your_creatures() -> Selector {
    Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    )
}

/// Spear of Heliod — {1}{W}{W} Legendary Enchantment. Creatures you control
/// get +1/+1. {1}{W}{W}, {T}: Destroy target creature that dealt damage to
/// you this turn.
pub fn spear_of_heliod() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(1), w(), w()]),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::DealtDamageToControllerThisTurn),
                ),
            },
            ..Default::default()
        }],
        ..god_weapon(
            "Spear of Heliod",
            cost(&[generic(1), w(), w()]),
            "Creatures you control get +1/+1.",
            StaticEffect::PumpPT { applies_to: your_creatures(), power: 1, toughness: 1 },
        )
    }
}

/// Whip of Erebos — {2}{B}{B} Legendary Enchantment. Creatures you control
/// have lifelink. {2}{B}{B}, {T}: Return target creature card from your
/// graveyard to the battlefield. It gains haste. Exile it at the next end
/// step.
pub fn whip_of_erebos() -> CardDefinition {
    use crate::effect::{DelayedTriggerKind, ZoneDest};
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(2), b(), b()]),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::InGraveyard),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
                },
            ]),
            ..Default::default()
        }],
        ..god_weapon(
            "Whip of Erebos",
            cost(&[generic(2), b(), b()]),
            "Creatures you control have lifelink.",
            StaticEffect::GrantKeyword { applies_to: your_creatures(), keyword: Keyword::Lifelink },
        )
    }
}

/// Hammer of Purphoros — {2}{R} Legendary Enchantment. Creatures you control
/// have haste. {1}{R}, Sacrifice a land: Create a 3/3 colorless Golem
/// artifact creature token. Activate only as a sorcery.
pub fn hammer_of_purphoros() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(1), r()]),
            sorcery_speed: true,
            sac_other_filter: Some((SelectionRequirement::Land, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::golem_3_3_token(),
            },
            ..Default::default()
        }],
        ..god_weapon(
            "Hammer of Purphoros",
            cost(&[generic(2), r()]),
            "Creatures you control have haste.",
            StaticEffect::GrantKeyword { applies_to: your_creatures(), keyword: Keyword::Haste },
        )
    }
}
