//! Modern Horizons 2 sweep, batch 5 — converge payoffs, discard-matters,
//! delirium, token synergy. Tests in `tests/mh2d.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement, Selector, StaticAbility,
    Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    draw, etb, investigate, mint_treasures, target_any, target_filtered,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, StaticEffect, ZoneDest, ZoneRef,
};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, x, Color};

use SelectionRequirement as R;

/// Burdened Aerialist — {2}{U} 3/1. ETB Treasure; whenever you sacrifice a
/// token, this gains flying this turn.
pub fn burdened_aerialist() -> CardDefinition {
    CardDefinition {
        name: "Burdened Aerialist",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![
            etb(mint_treasures(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::IsToken,
                    }),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Combine Chrysalis — {G}{U}. Creature tokens you control have flying;
/// {2}{G}{U}, {T}, Sacrifice a token: create a 4/4 Beast. Sorcery only.
pub fn combine_chrysalis() -> CardDefinition {
    let beast = TokenDefinition {
        name: "Beast".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Combine Chrysalis",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::IsToken).and(R::ControlledByYou),
                ),
                keyword: Keyword::Flying,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), u()]),
            tap_cost: true,
            sac_other_filter: Some((R::IsToken, 1)),
            sorcery_speed: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: beast },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dihada's Ploy — {1}{U}{B} instant. Draw two, discard one, gain life equal
/// to your discards this turn. Jump-start.
pub fn dihadas_ploy() -> CardDefinition {
    CardDefinition {
        name: "Dihada's Ploy",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::JumpStart],
        effect: Effect::Seq(vec![
            draw(2),
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::CardsDiscardedThisTurn(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Fae Offering — {2}{G}. Each end step, if you cast a creature and a
/// noncreature spell this turn: Clue + Food + Treasure.
pub fn fae_offering() -> CardDefinition {
    CardDefinition {
        name: "Fae Offering",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::All(vec![
                    Predicate::CreaturesCastThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::ONE,
                    },
                    Predicate::ValueAtLeast(
                        Value::NoncreatureSpellsCastThisTurn(PlayerRef::You),
                        Value::ONE,
                    ),
                ])),
            effect: Effect::Seq(vec![
                investigate(1),
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: crabomination_base::tokens::food_token(),
                },
                mint_treasures(1),
            ]),
        }],
        ..Default::default()
    }
}

/// Flay Essence — {1}{B}{B} sorcery. Exile target creature or planeswalker;
/// gain life equal to the counters on it.
pub fn flay_essence() -> CardDefinition {
    CardDefinition {
        name: "Flay Essence",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::TotalCountersOn { what: Box::new(Selector::Target(0)) },
            },
            Effect::Exile { what: target_filtered(R::Creature.or(R::Planeswalker)) },
        ]),
        ..Default::default()
    }
}

/// Flourishing Strike — {1}{G} instant. Choose one: 5 damage to target flyer;
/// or target creature +3/+3. Entwine {2}{G}.
pub fn flourishing_strike() -> CardDefinition {
    CardDefinition {
        name: "Flourishing Strike",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Entwine(cost(&[generic(2), g()]))],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                amount: Value::Const(5),
                to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Gilt-Blade Prowler — {2}{B} 2/3. {1}, {T}, Pay 1 life: draw a card.
/// Activate only if you've discarded a card this turn.
pub fn gilt_blade_prowler() -> CardDefinition {
    CardDefinition {
        name: "Gilt-Blade Prowler",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            life_cost: 1,
            condition: Some(Predicate::DiscardedThisTurn { who: PlayerRef::You }),
            effect: draw(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Glinting Creeper — {4}{G} 0/0. Converge — enters with two +1/+1 counters
/// per color spent; can't be blocked by power ≤ 2.
pub fn glinting_creeper() -> CardDefinition {
    CardDefinition {
        name: "Glinting Creeper",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::CantBeBlockedByPowerAtMost(2)],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Times(Box::new(Value::Const(2)), Box::new(Value::ConvergedValue)),
        )),
        ..Default::default()
    }
}

/// Glorious Enforcer — {5}{W}{W} 5/5 flying, lifelink. Each combat, if an
/// opponent has less life than you: double strike this turn.
pub fn glorious_enforcer() -> CardDefinition {
    CardDefinition {
        name: "Glorious Enforcer",
        cost: cost(&[generic(5), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::PlayerHasLessLifeThanOpponent {
                who: PlayerRef::EachOpponent,
            }),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Gouged Zealot — {3}{R} 4/3 reach. Delirium — attacks: 1 damage to each
/// creature the defending player controls (modeled: each opponent creature).
pub fn gouged_zealot() -> CardDefinition {
    CardDefinition {
        name: "Gouged Zealot",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cyclops, CreatureType::Berserker],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::DeliriumActive { who: PlayerRef::You }),
            effect: Effect::DealDamage {
                amount: Value::ONE,
                to: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            },
        }],
        ..Default::default()
    }
}

/// Junk Winder — {5}{U}{U} 5/6, affinity for tokens. Token ETB: tap + stun
/// target nonland permanent an opponent controls.
pub fn junk_winder() -> CardDefinition {
    CardDefinition {
        name: "Junk Winder",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Serpent], ..Default::default() },
        power: 5,
        toughness: 6,
        affinity_filter: Some(R::IsToken),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsToken,
                }),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(
                        R::Permanent.and(R::Land.negate()).and(R::ControlledByOpponent),
                    ),
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Lazotep Chancellor — {U}{B} 1/3. Whenever you discard a card, you may pay
/// {1}: amass Zombies 2.
pub fn lazotep_chancellor() -> CardDefinition {
    CardDefinition {
        name: "Lazotep Chancellor",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1} to amass Zombies 2?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(crate::effect::shortcut::amass_zombies(2)),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Lucid Dreams — {3}{U}{U} sorcery. Draw X = card types in your graveyard.
pub fn lucid_dreams() -> CardDefinition {
    CardDefinition {
        name: "Lucid Dreams",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::CardTypesInGraveyard(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Magus of the Bridge — {B}{B}{B} 4/4. Nontoken creature into your graveyard
/// from the battlefield: 2/2 Zombie. Creature into an opponent's graveyard:
/// exile this.
pub fn magus_of_the_bridge() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Magus of the Bridge",
        cost: cost(&[b(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::NotToken,
                    }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: zombie,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::Exile { what: Selector::This },
            },
        ],
        ..Default::default()
    }
}

/// Mystic Redaction — {2}{U}. Upkeep: scry 1. Whenever you discard a card,
/// each opponent mills two.
pub fn mystic_redaction() -> CardDefinition {
    CardDefinition {
        name: "Mystic Redaction",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
            },
        ],
        ..Default::default()
    }
}

/// Necromancer's Familiar — {3}{B} 3/1 flying. Hellbent — lifelink with an
/// empty hand. {B}, Discard a card: indestructible this turn; tap it.
pub fn necromancers_familiar() -> CardDefinition {
    CardDefinition {
        name: "Necromancer's Familiar",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Hellbent — This creature has lifelink as long as you have no cards in hand.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Lifelink,
                condition: Predicate::HellbentActive { who: PlayerRef::You },
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::Tap { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nykthos Paragon — {4}{W}{W} 4/6. Lifegain: you may put that many +1/+1
/// counters on each creature you control. Once each turn.
pub fn nykthos_paragon() -> CardDefinition {
    CardDefinition {
        name: "Nykthos Paragon",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::MayDo {
                description: "put that many +1/+1 counters on each creature you control".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::EachMatching {
                        zone: ZoneRef::Battlefield,
                        filter: R::Creature.and(R::ControlledByYou),
                    },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::TriggerEventAmount,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Prophetic Titan — {4}{U}{R} 4/4. ETB choose one; delirium: choose both —
/// 4 damage to any target / dig 4 keep 1.
pub fn prophetic_titan() -> CardDefinition {
    let bolt = Effect::DealDamage { amount: Value::Const(4), to: target_any() };
    let dig = Effect::LookPickToHand {
        who: PlayerRef::You,
        count: Value::Const(4),
        rest_to_graveyard: false,
        pick_filter: None,
        take: None,
        to_battlefield: false,
        gain_life_if_pick: None,
        gain_life_greatest_power_rest: false,
    };
    CardDefinition {
        name: "Prophetic Titan",
        cost: cost(&[generic(4), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::DeliriumActive { who: PlayerRef::You },
            then: Box::new(Effect::Seq(vec![bolt.clone(), dig.clone()])),
            else_: Box::new(Effect::ChooseMode(vec![bolt, dig])),
        })],
        ..Default::default()
    }
}

/// Radiant Epicure — {4}{B} 5/5. Converge — ETB each opponent loses X and
/// you gain X, X = colors spent.
pub fn radiant_epicure() -> CardDefinition {
    CardDefinition {
        name: "Radiant Epicure",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ConvergedValue,
            },
            Effect::GainLife { who: Selector::You, amount: Value::ConvergedValue },
        ]))],
        ..Default::default()
    }
}

/// Raving Visionary — {1}{U} 1/1. {U}, {T}: loot. Delirium — {2}{U}, {T}:
/// draw a card (only with 4+ card types in your graveyard).
pub fn raving_visionary() -> CardDefinition {
    CardDefinition {
        name: "Raving Visionary",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    draw(1),
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                tap_cost: true,
                condition: Some(Predicate::DeliriumActive { who: PlayerRef::You }),
                effect: draw(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Recalibrate — {1}{U} instant. Bounce target creature; draw if you've
/// discarded this turn.
pub fn recalibrate() -> CardDefinition {
    CardDefinition {
        name: "Recalibrate",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::If {
                cond: Predicate::DiscardedThisTurn { who: PlayerRef::You },
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Revolutionist — {5}{R} 3/3. ETB return target instant or sorcery from
/// your graveyard to your hand. Madness {3}{R}.
pub fn revolutionist() -> CardDefinition {
    CardDefinition {
        name: "Revolutionist",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Madness(cost(&[generic(3), r()]))],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Sanctuary Raptor — {3} 2/1 flying. Attacks with 3+ tokens under your
/// control: +2/+0 and first strike this turn.
pub fn sanctuary_raptor() -> CardDefinition {
    CardDefinition {
        name: "Sanctuary Raptor",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::ValueAtLeast(
                    Value::count(Selector::EachMatching {
                        zone: ZoneRef::Battlefield,
                        filter: R::IsToken.and(R::ControlledByYou),
                    }),
                    Value::Const(3),
                ),
            ),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Scour the Desert — {3}{W}{W} sorcery. Exile target creature card from
/// your graveyard; create X 1/1 Birds with flying, X = its toughness.
pub fn scour_the_desert() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Bird".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Scour the Desert",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ToughnessOf(Box::new(Selector::Target(0))),
                definition: bird,
            },
            Effect::Move { what: target_filtered(R::Creature), to: ZoneDest::Exile },
        ]),
        ..Default::default()
    }
}

/// Scuttletide — {1}{U}. {1}, Discard a card: 0/3 Crab. Delirium — Crabs you
/// control get +1/+1.
pub fn scuttletide() -> CardDefinition {
    let crab = TokenDefinition {
        name: "Crab".into(),
        power: 0,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Crab], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Scuttletide",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: crab },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Delirium — Crabs you control get +1/+1 as long as there are four or more card types among cards in your graveyard.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::DeliriumActive { who: PlayerRef::You },
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Crab).and(R::ControlledByYou),
                ),
                power: 1,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Skyblade's Boon — {1}{W} Aura. +1/+1 and flying; {2}{W}: return it to
/// your hand from the battlefield or your graveyard.
pub fn skyblades_boon() -> CardDefinition {
    CardDefinition {
        name: "Skyblade's Boon",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                from_graveyard: true,
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Smell Fear — {1}{G} sorcery. Proliferate, then target creature you
/// control fights target creature you don't control.
pub fn smell_fear() -> CardDefinition {
    CardDefinition {
        name: "Smell Fear",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Proliferate,
            Effect::Fight {
                attacker: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Specimen Collector — {4}{U} 2/1. ETB a 1/1 Squirrel and a 0/3 Crab. Dies:
/// token copy of target token you control.
pub fn specimen_collector() -> CardDefinition {
    let squirrel = TokenDefinition {
        name: "Squirrel".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Squirrel], ..Default::default() },
        ..Default::default()
    };
    let crab = TokenDefinition {
        name: "Crab".into(),
        power: 0,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Crab], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Specimen Collector",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: squirrel,
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: crab },
            ])),
            crate::effect::shortcut::on_dies(Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(R::IsToken.and(R::ControlledByYou)),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            }),
        ],
        ..Default::default()
    }
}

/// Spreading Insurrection — {4}{R} sorcery with storm. Threaten target
/// creature you don't control.
pub fn spreading_insurrection() -> CardDefinition {
    CardDefinition {
        name: "Spreading Insurrection",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Storm],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Sweep the Skies — {X}{U}{U} sorcery. Converge — a 1/1 Thopter with flying
/// per color of mana spent.
pub fn sweep_the_skies() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Sweep the Skies",
        cost: cost(&[x(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ConvergedValue,
            definition: thopter,
        },
        ..Default::default()
    }
}

/// Tourach's Canticle — {3}{B} sorcery. Target opponent reveals their hand;
/// you pick a discard, then they discard one at random.
pub fn tourachs_canticle() -> CardDefinition {
    CardDefinition {
        name: "Tourach's Canticle",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: R::Any,
            },
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: true,
            },
        ]),
        ..Default::default()
    }
}

/// Unbounded Potential — {1}{W} instant. Choose one: +1/+1 counter on up to
/// two targets; or proliferate. Entwine {3}{W}.
pub fn unbounded_potential() -> CardDefinition {
    CardDefinition {
        name: "Unbounded Potential",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Entwine(cost(&[generic(3), w()]))],
        effect: Effect::ChooseMode(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                filter: R::Creature,
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
            Effect::Proliferate,
        ]),
        ..Default::default()
    }
}

/// Sanctum Weaver — {1}{G} 0/2. {T}: Add X mana of any one color, where X is
/// the number of enchantments you control.
pub fn sanctum_weaver() -> CardDefinition {
    CardDefinition {
        name: "Sanctum Weaver",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dryad], ..Default::default() },
        power: 0,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::count(Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Enchantment.and(R::ControlledByYou),
                })),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Void Mirror — {2} artifact. Whenever a player casts a spell, if no
/// colored mana was spent to cast it, counter that spell.
pub fn void_mirror() -> CardDefinition {
    CardDefinition {
        name: "Void Mirror",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellNoColoredManaSpent),
            effect: Effect::CounterSpell { what: Selector::TriggerSource },
        }],
        ..Default::default()
    }
}

/// Vectis Gloves — {2} Equipment. +2/+0 and artifact landwalk; equip {2}.
pub fn vectis_gloves() -> CardDefinition {
    CardDefinition {
        name: "Vectis Gloves",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 2,
            keywords: vec![Keyword::LandwalkFiltered(Box::new(R::Land.and(R::Artifact)))],
            ..Default::default()
        }),
        ..Default::default()
    }
}
