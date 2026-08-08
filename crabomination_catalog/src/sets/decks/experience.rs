//! Experience-counter commanders (Commander 2015 cycle). Ships the
//! `Player.experience` resource, `Effect::AddExperience`,
//! `Value::ControllerExperience`, `StaticEffect::CostReductionPerControllerExperience`
//! (Mizzix), and `DynamicPt::ControllerExperience` (Kalemne, Daxos's token).
//! Tests in `tests/experience.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, Effect,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{cast_is_instant_or_sorcery, target_filtered};
use crate::effect::{PlayerRef, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Mizzix of the Izmagnus — {2}{U}{R} 2/2 Legendary Goblin Wizard. Cast an
/// instant or sorcery → get an experience counter; those spells cost {X} less,
/// X = your experience. (The printed "with mana value greater than your
/// experience" runaway gate is approximated as any I/S cast.)
pub fn mizzix_of_the_izmagnus() -> CardDefinition {
    CardDefinition {
        name: "Mizzix of the Izmagnus",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::AddExperience(Value::Const(1)),
        }],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {X} less, where X is the number of experience counters you have.",
            effect: StaticEffect::CostReductionPerControllerExperience {
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            },
        }],
        ..Default::default()
    }
}

/// Ezuri, Claw of Progress — {2}{G}{U} 3/3 Legendary Elf Warrior. Creature with
/// power 1 or less enters under your control → get an experience counter. At
/// beginning of combat on your turn, put X +1/+1 counters on target creature
/// (X = your experience).
pub fn ezuri_claw_of_progress() -> CardDefinition {
    CardDefinition {
        name: "Ezuri, Claw of Progress",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::PowerAtMost(1)),
                    }),
                effect: Effect::AddExperience(Value::Const(1)),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ControllerExperience,
                },
            },
        ],
        ..Default::default()
    }
}

/// A white/black Spirit enchantment-creature token whose P/T track the
/// controller's experience at mint time (Daxos the Returned). Live-updating
/// P/T would need a card-level `DynamicPt` on the token; the mint-time snapshot
/// matches how the engine stamps other CDA tokens (Shark Typhoon).
fn daxos_spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 0,
        toughness: 0,
        card_types: vec![CardType::Enchantment, CardType::Creature],
        colors: vec![Color::White, Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        dynamic_pt: Some((Value::ControllerExperience, Value::ControllerExperience)),
        ..Default::default()
    }
}

/// Daxos the Returned — {2}{W}{B} 2/2 Legendary Enchantment Creature — God.
/// Cast an enchantment → get an experience counter. {1}{W}{B}: create a Spirit
/// token with power/toughness each equal to your experience.
pub fn daxos_the_returned() -> CardDefinition {
    CardDefinition {
        name: "Daxos the Returned",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier, CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Enchantment),
                },
            ),
            effect: Effect::AddExperience(Value::Const(1)),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w(), b()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Box::new(daxos_spirit_token()),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Meren of Clan Nel Toth — {2}{B}{G} 3/4 Legendary Human Shaman. Another
/// creature you control dies → get an experience counter. Your end step →
/// target creature card in your graveyard: return it to the battlefield if its
/// mana value ≤ your experience, otherwise to your hand.
pub fn meren_of_clan_nel_toth() -> CardDefinition {
    CardDefinition {
        name: "Meren of Clan Nel Toth",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
                effect: Effect::AddExperience(Value::Const(1)),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::ValueAtMost(
                        Value::ManaValueOf(Box::new(Selector::Target(0))),
                        Value::ControllerExperience,
                    ),
                    then: Box::new(Effect::Move {
                        what: target_filtered(R::Creature.and(R::OwnedByYou)),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    }),
                    else_: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Kalemne, Disciple of Iroas — {3}{W}{W} 2/4 Legendary Giant Soldier,
/// vigilance. Cast a creature spell with mana value 5+ → get an experience
/// counter. Kalemne gets +1/+1 for each experience counter you have.
pub fn kalemne_disciple_of_iroas() -> CardDefinition {
    CardDefinition {
        name: "Kalemne, Disciple of Iroas",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::DoubleStrike, Keyword::Vigilance],
        dynamic_pt: Some(DynamicPt::ControllerExperience {
            base_p: 2,
            base_t: 4,
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::ManaValueAtLeast(5)),
                },
            ),
            effect: Effect::AddExperience(Value::Const(1)),
        }],
        ..Default::default()
    }
}
