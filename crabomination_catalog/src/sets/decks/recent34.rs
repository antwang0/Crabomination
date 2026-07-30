//! Quest-counter enchantments (Zendikar) plus a few long-missing staples.
//! Quests accrue a `CounterType::Quest` on a trigger, then either gate a
//! static payoff or pay an activated ability via `remove_counter_cost` +
//! `sac_cost`. The printed "you may put a quest counter" is collapsed to a
//! mandatory add (the catalog convention for harmless optional accrual).
//! Tests in `tests/recent34.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{draw, target_filtered};
use crate::effect::{Duration, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Add one quest counter to this enchantment.
fn add_quest() -> Effect {
    Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Quest,
        amount: Value::Const(1),
    }
}

/// Quest for the Goblin Lord — {R} Enchantment. Whenever a Goblin you control
/// enters, put a quest counter; while it has five or more, creatures you
/// control get +2/+0.
pub fn quest_for_the_goblin_lord() -> CardDefinition {
    CardDefinition {
        name: "Quest for the Goblin Lord",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Goblin),
                }),
            effect: add_quest(),
        }],
        static_abilities: vec![StaticAbility {
            description: "While this has five or more quest counters, creatures you control get +2/+0.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Quest,
                    },
                    Value::Const(5),
                ),
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: 2,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Quest for the Gravelord — {B} Enchantment. Whenever a creature dies, put a
/// quest counter. Remove three quest counters and sacrifice it: create a 5/5
/// black Zombie Giant.
pub fn quest_for_the_gravelord() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie Giant".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Giant],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Quest for the Gravelord",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: add_quest(),
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Quest, 3)),
            sac_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: zombie,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Quest for the Gemblades — {1}{G} Enchantment. Whenever a creature you
/// control deals combat damage to a creature, put a quest counter. Remove a
/// quest counter and sacrifice it: put four +1/+1 counters on target creature.
pub fn quest_for_the_gemblades() -> CardDefinition {
    CardDefinition {
        name: "Quest for the Gemblades",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToCreature,
                EventScope::YourControl,
            ),
            effect: add_quest(),
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Quest, 1)),
            sac_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(4),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Quest for Ancient Secrets — {U} Enchantment. Whenever a card is put into
/// your graveyard from anywhere, put a quest counter. Remove five quest
/// counters and sacrifice it: target player shuffles their graveyard into
/// their library.
pub fn quest_for_ancient_secrets() -> CardDefinition {
    CardDefinition {
        name: "Quest for Ancient Secrets",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::YourControl),
            effect: add_quest(),
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Quest, 5)),
            sac_cost: true,
            effect: Effect::ShuffleGraveyardIntoLibrary {
                who: PlayerRef::Target(0),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Quest for the Holy Relic — {W} Enchantment. Whenever you cast a creature
/// spell, put a quest counter. Remove five quest counters and sacrifice it:
/// fetch an Equipment onto the battlefield attached to a creature you control
/// (auto-pick: your greatest-power creature).
pub fn quest_for_the_holy_relic() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    CardDefinition {
        name: "Quest for the Holy Relic",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Creature)),
            effect: add_quest(),
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Quest, 5)),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::Attach {
                    what: Selector::LastMoved,
                    to: Selector::GreatestPowerYouControl,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Magebane Lizard — {1}{R} 1/4 Lizard. Whenever a player casts a noncreature
/// spell, deal damage to that player equal to the number of noncreature spells
/// they've cast this turn.
pub fn magebane_lizard() -> CardDefinition {
    CardDefinition {
        name: "Magebane Lizard",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::CastSpellMatches(SelectionRequirement::Noncreature),
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::NoncreatureSpellsCastThisTurn(PlayerRef::Triggerer),
            },
        }],
        ..Default::default()
    }
}

/// Atog — {1}{R} 1/2 Atog. Sacrifice an artifact: this creature gets +2/+2
/// until end of turn.
pub fn atog() -> CardDefinition {
    CardDefinition {
        name: "Atog",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Atog],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((
                SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                1,
            )),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Origin Spellbomb — {1} Artifact. {1}, {T}, Sacrifice this: create a 1/1
/// colorless Myr. When it's put into a graveyard from the battlefield, you may
/// pay {W} to draw a card.
pub fn origin_spellbomb() -> CardDefinition {
    let myr = TokenDefinition {
        name: "Myr".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Myr],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Origin Spellbomb",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: myr,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {W}: draw a card.".into(),
                mana_cost: cost(&[w()]),
                body: Box::new(draw(1)),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Land Tax — {W} Enchantment. At the beginning of your upkeep, if an opponent
/// controls more lands than you, search your library for up to three basic land
/// cards, reveal them, and put them into your hand.
pub fn land_tax() -> CardDefinition {
    CardDefinition {
        name: "Land Tax",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::OpponentControlsMoreLandsThanYou),
            effect: Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(3),
            },
        }],
        ..Default::default()
    }
}
