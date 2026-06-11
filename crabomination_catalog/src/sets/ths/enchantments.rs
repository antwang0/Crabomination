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
            keywords: vec![Keyword::Lifelink], scale: None, triggered_abilities: vec![], ..Default::default() }),
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

/// Two-color god frame: devotion threshold 7 (CR 700.5).
fn god2(
    name: &'static str,
    cost_: crate::mana::ManaCost,
    colors: Vec<Color>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as your devotion to its colors is less than seven, this isn't a creature.",
            effect: StaticEffect::NotCreatureWhileDevotionBelow { colors: colors.clone(), threshold: 7 },
        }],
        ..god(name, cost_, colors, power, toughness)
    }
}

/// Heliod, God of the Sun — {3}{W} 5/6. Other creatures you control have
/// vigilance. {2}{W}{W}: Create a 2/1 white Cleric enchantment creature token.
pub fn heliod_god_of_the_sun() -> CardDefinition {
    use crate::card::TokenDefinition;
    let cleric = TokenDefinition {
        name: "Cleric".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Enchantment, CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cleric], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "As long as your devotion to white is less than five, Heliod isn't a creature.",
                effect: StaticEffect::NotCreatureWhileDevotionBelow {
                    colors: vec![Color::White],
                    threshold: 5,
                },
            },
            StaticAbility {
                description: "Other creatures you control have vigilance.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    keyword: Keyword::Vigilance,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w(), w()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: cleric,
            },
            ..Default::default()
        }],
        ..god("Heliod, God of the Sun", cost(&[generic(3), w()]), vec![Color::White], 5, 6)
    }
}

/// Purphoros, God of the Forge — {3}{R} 6/5. Whenever another creature you
/// control enters, deal 2 to each opponent. {2}{R}: your creatures +1/+0.
pub fn purphoros_god_of_the_forge() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::PumpPT {
                what: your_creatures(),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..god("Purphoros, God of the Forge", cost(&[generic(3), r()]), vec![Color::Red], 6, 5)
    }
}

/// Xenagos, God of Revels — {3}{R}{G} 6/5. At the beginning of combat on
/// your turn, another target creature you control gains haste and +X/+0
/// where X is its power.
pub fn xenagos_god_of_revels() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::PowerOf(Box::new(Selector::Target(0))),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..god2("Xenagos, God of Revels", cost(&[generic(3), r(), g()]), vec![Color::Red, Color::Green], 6, 5)
    }
}

/// Phenax, God of Deception — {3}{U}{B} 4/7. Creatures you control have
/// "{T}: Target player mills X cards, where X is this creature's toughness."
pub fn phenax_god_of_deception() -> CardDefinition {
    let mill_ability = crate::card::ActivatedAbility {
        tap_cost: true,
        effect: Effect::Mill {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::ToughnessOf(Box::new(Selector::This)),
        },
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "As long as your devotion to blue and black is less than seven, Phenax isn't a creature.",
                effect: StaticEffect::NotCreatureWhileDevotionBelow {
                    colors: vec![Color::Blue, Color::Black],
                    threshold: 7,
                },
            },
            StaticAbility {
                description: "Creatures you control have \"{T}: Target player mills X cards, where X is this creature's toughness.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: your_creatures(),
                    ability: mill_ability,
                },
            },
        ],
        ..god("Phenax, God of Deception", cost(&[generic(3), u(), b()]), vec![Color::Blue, Color::Black], 4, 7)
    }
}

/// Pharika, God of Affliction — {1}{B}{G} 5/5. {B}{G}: Exile target creature
/// card from a graveyard; its owner creates a 1/1 B/G deathtouch Snake
/// enchantment creature token.
pub fn pharika_god_of_affliction() -> CardDefinition {
    use crate::card::TokenDefinition;
    let snake = TokenDefinition {
        name: "Snake".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Enchantment, CardType::Creature],
        colors: vec![Color::Black, Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), g()]),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    count: Value::Const(1),
                    definition: snake,
                },
                Effect::Exile {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::InGraveyard),
                    ),
                },
            ]),
            ..Default::default()
        }],
        ..god2("Pharika, God of Affliction", cost(&[generic(1), b(), g()]), vec![Color::Black, Color::Green], 5, 5)
    }
}

/// Karametra, God of Harvests — {3}{G}{W} 6/7. Whenever you cast a creature
/// spell, you may search for a Forest or Plains card → battlefield tapped.
pub fn karametra_god_of_harvests() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::MayDo {
                description: "Search your library for a Forest or Plains card and put it onto the battlefield tapped?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Land.and(
                        SelectionRequirement::HasLandType(crate::card::LandType::Forest)
                            .or(SelectionRequirement::HasLandType(crate::card::LandType::Plains)),
                    ),
                    to: crate::effect::ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
            },
        }],
        ..god2("Karametra, God of Harvests", cost(&[generic(3), g(), w()]), vec![Color::Green, Color::White], 6, 7)
    }
}

/// Mogis, God of Slaughter — {2}{B}{R} 7/5. At each opponent's upkeep, deals
/// 2 damage to that player unless they sacrifice a creature.
pub fn mogis_god_of_slaughter() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::OpponentControl,
            ),
            effect: Effect::Punisher {
                chooser: Selector::Player(PlayerRef::ActivePlayer),
                options: vec![Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::You),
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature,
                }],
                otherwise: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::Const(2),
                }),
            },
        }],
        ..god2("Mogis, God of Slaughter", cost(&[generic(2), b(), r()]), vec![Color::Black, Color::Red], 7, 5)
    }
}

/// Athreos, God of Passage — {1}{W}{B} 5/4. Whenever another creature you
/// control dies, return it to your hand unless an opponent pays 3 life.
pub fn athreos_god_of_passage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::EachOpponent,
                cost: crate::card::WardCost::Life(3),
                then: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(
                        Selector::TriggerSource,
                    ))),
                }),
            },
        }],
        ..god2("Athreos, God of Passage", cost(&[generic(1), w(), b()]), vec![Color::White, Color::Black], 5, 4)
    }
}

/// Iroas, God of Victory — {2}{R}{W} 7/4. Creatures you control have menace;
/// prevent all damage that would be dealt to attacking creatures you control.
pub fn iroas_god_of_victory() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "As long as your devotion to red and white is less than seven, Iroas isn't a creature.",
                effect: StaticEffect::NotCreatureWhileDevotionBelow {
                    colors: vec![Color::Red, Color::White],
                    threshold: 7,
                },
            },
            StaticAbility {
                description: "Creatures you control have menace.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: your_creatures(),
                    keyword: Keyword::Menace,
                },
            },
            StaticAbility {
                description: "Prevent all damage that would be dealt to attacking creatures you control.",
                effect: StaticEffect::PreventDamageToYourAttackers,
            },
        ],
        ..god("Iroas, God of Victory", cost(&[generic(2), r(), w()]), vec![Color::Red, Color::White], 7, 4)
    }
}

/// Kruphix, God of Horizons — {3}{G}{U} 4/7. You have no maximum hand size;
/// unspent mana becomes colorless instead of emptying.
pub fn kruphix_god_of_horizons() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "As long as your devotion to green and blue is less than seven, Kruphix isn't a creature.",
                effect: StaticEffect::NotCreatureWhileDevotionBelow {
                    colors: vec![Color::Green, Color::Blue],
                    threshold: 7,
                },
            },
            StaticAbility {
                description: "You have no maximum hand size.",
                effect: StaticEffect::NoMaximumHandSize,
            },
            StaticAbility {
                description: "If you would lose unspent mana, that mana becomes colorless instead.",
                effect: StaticEffect::UnspentManaBecomesColorless,
            },
        ],
        ..god("Kruphix, God of Horizons", cost(&[generic(3), g(), u()]), vec![Color::Green, Color::Blue], 4, 7)
    }
}

/// Ephara, God of the Polis — {2}{W}{U} 6/5. At the beginning of each
/// upkeep, if another creature entered under your control last turn, draw.
pub fn ephara_god_of_the_polis() -> CardDefinition {
    use crate::card::Predicate;
    use crate::game::types::TurnStep;
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::AnotherCreatureEnteredControlLastTurn {
                    who: PlayerRef::You,
                }),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..god2("Ephara, God of the Polis", cost(&[generic(2), w(), u()]), vec![Color::White, Color::Blue], 6, 5)
    }
}

/// Keranos, God of Storms — {3}{U}{R} 6/5. Reveal the first card you draw on
/// each of your turns: land → draw a card; nonland → deal 3 to any target.
pub fn keranos_god_of_storms() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::IsTurnOf(PlayerRef::You),
                    Predicate::ValueAtMost(
                        Value::CardsDrawnThisTurn(PlayerRef::You),
                        Value::Const(1),
                    ),
                ])),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Land,
                },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: Box::new(Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(3),
                }),
            },
        }],
        ..god2("Keranos, God of Storms", cost(&[generic(3), u(), r()]), vec![Color::Blue, Color::Red], 6, 5)
    }
}

/// Thassa, Deep-Dwelling — {3}{U} 6/5. End-step flicker of up to one other
/// creature you control; {3}{U}: tap another target creature.
pub fn thassa_deep_dwelling() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Exile another creature you control, then return it?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Exile {
                        what: target_filtered(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou)
                                .and(SelectionRequirement::OtherThanSource),
                        ),
                    },
                    Effect::Move {
                        what: Selector::Target(0),
                        to: crate::effect::ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                ])),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                ),
            },
            ..Default::default()
        }],
        ..god("Thassa, Deep-Dwelling", cost(&[generic(3), u()]), vec![Color::Blue], 6, 5)
    }
}

/// Erebos, Bleak-Hearted — {3}{B} 5/6. Another creature you control dies →
/// may pay 2 life to draw; {1}{B}, sac another creature: target gets -2/-1.
pub fn erebos_bleak_hearted() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::MayDo {
                description: "Pay 2 life to draw a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ])),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..god("Erebos, Bleak-Hearted", cost(&[generic(3), b()]), vec![Color::Black], 5, 6)
    }
}

/// Purphoros, Bronze-Blooded — {4}{R} 7/6. Other creatures you control have
/// haste; {2}{R}: sneak a red or artifact creature from hand (sac at end step).
pub fn purphoros_bronze_blooded() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "As long as your devotion to red is less than five, Purphoros isn't a creature.",
                effect: StaticEffect::NotCreatureWhileDevotionBelow {
                    colors: vec![Color::Red],
                    threshold: 5,
                },
            },
            StaticAbility {
                description: "Other creatures you control have haste.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    keyword: Keyword::Haste,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature.and(
                    SelectionRequirement::HasColor(Color::Red).or(SelectionRequirement::Artifact),
                ),
                count: Value::Const(1),
                tapped: false,
                haste: false,
                sacrifice_eot: true,
            },
            ..Default::default()
        }],
        ..god("Purphoros, Bronze-Blooded", cost(&[generic(4), r()]), vec![Color::Red], 7, 6)
    }
}

/// Nylea, Keen-Eyed — {3}{G} 5/6. Creature spells cost {1} less; {2}{G}:
/// reveal the top card, take it if it's a creature.
pub fn nylea_keen_eyed() -> CardDefinition {
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
                description: "Creature spells you cast cost {1} less to cast.",
                effect: StaticEffect::CostReduction {
                    filter: SelectionRequirement::Creature,
                    amount: 1,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::RevealTopAndDrawIf {
                who: PlayerRef::You,
                reveal_filter: SelectionRequirement::Creature,
                may_graveyard_miss: true,
            },
            ..Default::default()
        }],
        ..god("Nylea, Keen-Eyed", cost(&[generic(3), g()]), vec![Color::Green], 5, 6)
    }
}

/// Klothys, God of Destiny — {1}{R}{G} 4/5. At your first main, exile target
/// graveyard card: land → add {R} or {G}; else gain 2 and ping opponents 2.
pub fn klothys_god_of_destiny() -> CardDefinition {
    use crate::card::Predicate;
    use crate::effect::ManaPayload;
    use crate::game::types::TurnStep;
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::Exile {
                    what: target_filtered(SelectionRequirement::InGraveyard),
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: SelectionRequirement::Land,
                    },
                    then: Box::new(Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::OfColors(
                            vec![Color::Red, Color::Green],
                            Value::Const(1),
                        ),
                    }),
                    else_: Box::new(Effect::Seq(vec![
                        Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                        Effect::DealDamage {
                            to: Selector::Player(PlayerRef::EachOpponent),
                            amount: Value::Const(2),
                        },
                    ])),
                },
            ]),
        }],
        ..god2("Klothys, God of Destiny", cost(&[generic(1), r(), g()]), vec![Color::Red, Color::Green], 4, 5)
    }
}
