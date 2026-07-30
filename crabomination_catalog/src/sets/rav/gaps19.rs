//! Ravnica (RAV) gap wave 19: radiance lifegain plus a batch of
//! graveyard/library utility. Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, Keyword, SelectionRequirement as R, Subtypes, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, TriggeredAbility,
    ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, x};

/// Brightflame — {X}{R}{R}{W}{W} Sorcery. Radiance — deal X damage to target
/// creature and each other creature that shares a color with it; gain life
/// equal to the damage dealt this way.
pub fn brightflame() -> CardDefinition {
    CardDefinition {
        name: "Brightflame",
        cost: cost(&[x(), r(), r(), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::RadianceDamage {
                subject: target_filtered(R::Creature),
                amount: Value::XFromCost,
            },
            Effect::GainLife {
                who: Selector::Player(PlayerRef::You),
                amount: Value::DamageDealtThisResolution,
            },
        ]),
        ..Default::default()
    }
}

/// Lurking Informant — {1}{U/B} 1/2 Human Rogue. {2}, {T}: look at the top
/// card of target player's library; you may put it into their graveyard.
pub fn lurking_informant() -> CardDefinition {
    use crate::mana::Color;
    use crate::mana::hybrid;
    CardDefinition {
        name: "Lurking Informant",
        cost: cost(&[generic(1), hybrid(Color::Blue, Color::Black)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::MayDo {
                description: "Put that card into that player's graveyard?".into(),
                body: Box::new(Effect::Mill {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gleancrawler — {3}{B/G}{B/G}{B/G} 6/6 Insect Horror with trample. At your
/// end step, return every creature card that hit your graveyard from the
/// battlefield this turn to your hand.
pub fn gleancrawler() -> CardDefinition {
    use crate::mana::Color;
    use crate::mana::hybrid;
    let bg = || hybrid(Color::Black, Color::Green);
    CardDefinition {
        name: "Gleancrawler",
        cost: cost(&[generic(3), bg(), bg(), bg()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Horror],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::YourControl,
            ),
            effect: Effect::Move {
                what: Selector::CardsInZone {
                    zone: crate::card::Zone::Graveyard,
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::PutIntoGraveyardFromBattlefieldThisTurn),
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..Default::default()
    }
}

/// Mindmoil — {4}{R} Enchantment. Whenever you cast a spell, put your hand on
/// the bottom of your library in any order, then draw that many cards.
pub fn mindmoil() -> CardDefinition {
    CardDefinition {
        name: "Mindmoil",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::BottomHandThenDrawThatMany {
                who: PlayerRef::You,
            },
        }],
        ..Default::default()
    }
}

/// Leashling — {6} 3/3 Dog artifact creature. Put a card from your hand on top
/// of your library: return this creature to its owner's hand.
pub fn leashling() -> CardDefinition {
    CardDefinition {
        name: "Leashling",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Seq(vec![
                Effect::PutCardFromHandOnTopOfLibrary {
                    who: Selector::Player(PlayerRef::You),
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ]),
            condition: Some(Predicate::SelectorExists(Selector::CardsInZone {
                zone: crate::card::Zone::Hand,
                who: PlayerRef::You,
                filter: R::Any,
            })),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Plague Boiler — {3} Artifact. Grows a plague counter each of your upkeeps
/// ({1}{B}{G} adds or removes one); at three it sacrifices itself and wipes
/// every nonland permanent.
pub fn plague_boiler() -> CardDefinition {
    CardDefinition {
        name: "Plague Boiler",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b(), g()]),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Plague,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b(), g()]),
                effect: Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Plague,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Plague,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec {
                    filter: Some(Predicate::SourceHasCountersAtLeast {
                        counter: CounterType::Plague,
                        n: 3,
                    }),
                    ..EventSpec::new(
                        EventKind::CounterAdded(CounterType::Plague),
                        EventScope::SelfSource,
                    )
                },
                effect: Effect::Seq(vec![
                    Effect::SacrificeSource,
                    Effect::Destroy {
                        what: Selector::EachPermanent(R::Nonland),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Bloodletter Quill — {3} Artifact. {2}, {T}, add a blood counter: draw a
/// card, then lose 1 life per blood counter. {U}{B}: remove a blood counter.
pub fn bloodletter_quill() -> CardDefinition {
    CardDefinition {
        name: "Bloodletter Quill",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Blood,
                        amount: Value::ONE,
                    },
                    Effect::Draw {
                        who: Selector::Player(PlayerRef::You),
                        amount: Value::ONE,
                    },
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::You),
                        amount: Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Blood,
                        },
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u(), b()]),
                effect: Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Blood,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Drake Familiar — {1}{U} 2/1 Drake with flying. When it enters, sacrifice it
/// unless you return an enchantment to its owner's hand.
pub fn drake_familiar() -> CardDefinition {
    CardDefinition {
        name: "Drake Familiar",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(
                R::Enchantment.and(R::ControlledByYou),
            )),
            then: Box::new(Effect::MoveChosen {
                from: Selector::EachPermanent(R::Enchantment.and(R::ControlledByYou)),
                filter: None,
                count: Value::ONE,
                up_to: false,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
            else_: Box::new(Effect::SacrificeSource),
        })],
        ..Default::default()
    }
}

/// Thoughtpicker Witch — {B} 1/1 Human Wizard. {1}, Sacrifice a creature: look
/// at the top two cards of target opponent's library, then exile one of them.
pub fn thoughtpicker_witch() -> CardDefinition {
    CardDefinition {
        name: "Thoughtpicker Witch",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::LookTopExileOneOfN {
                who: PlayerRef::Target(0),
                count: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Razia's Purification — {4}{R}{W} Sorcery. Each player chooses three
/// permanents they control, then sacrifices the rest.
pub fn razias_purification() -> CardDefinition {
    CardDefinition {
        name: "Razia's Purification",
        cost: cost(&[generic(4), r(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::EachPlayerKeepsNSacrificesRest {
            keep: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Instill Furor — {1}{R} Aura. Enchant creature. The enchanted creature must
/// attack each turn or be sacrificed at its controller's end step.
pub fn instill_furor() -> CardDefinition {
    CardDefinition {
        name: "Instill Furor",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                filter: Some(Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    filter: R::AttackedThisTurn,
                }))),
                ..EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
            },
            effect: Effect::SacrificePermanent {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        equipped_bonus: Some(EquipBonus::default()),
        ..Default::default()
    }
}
