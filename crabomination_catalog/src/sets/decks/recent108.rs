//! MH3-era value staples batch. Tests in `tests/recent108.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Selector, Subtypes, Supertype, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate, RevealMissDest, ZoneDest};
use crate::mana::{cost, g, generic, u};

/// Urza's Cave — Land. {T}: {C}; {3},{T},sac: fetch a land tapped.
pub fn urzas_cave() -> CardDefinition {
    CardDefinition {
        name: "Urza's Cave",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Land,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Fallaji Archaeologist — {1}{U} 0/3. ETB: mill 3, take a noncreature
/// nonland to hand, else grow.
pub fn fallaji_archaeologist() -> CardDefinition {
    CardDefinition {
        name: "Fallaji Archaeologist",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::MillThenToHand {
            amount: Value::Const(3),
            filter: SelectionRequirement::Noncreature.and(SelectionRequirement::Nonland),
            otherwise: Some(Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            })),
        })],
        ..Default::default()
    }
}

/// Sleep-Cursed Faerie — {U} 3/3 flier, ward {2}; enters tapped with three
/// stun counters; {1}{U}: untap it.
pub fn sleep_cursed_faerie() -> CardDefinition {
    CardDefinition {
        name: "Sleep-Cursed Faerie",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![
            Keyword::Flying,
            Keyword::Ward(crate::card::WardCost::Mana(cost(&[generic(2)]))),
        ],
        enters_with_counters: Some((CounterType::Stun, Value::Const(3))),
        triggered_abilities: vec![etb(Effect::Tap {
            what: Selector::This,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Untap {
                what: Selector::This,
                up_to: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Manabond — {G}. Your end step: may dump your hand's lands onto the
/// battlefield and discard the rest.
pub fn manabond() -> CardDefinition {
    CardDefinition {
        name: "Manabond",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Put all lands from your hand onto the battlefield and discard \
                              the rest?"
                    .into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: crate::card::Zone::Hand,
                            filter: SelectionRequirement::Land,
                        },
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::HandSizeOf(PlayerRef::You),
                        random: false,
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Nissa, Resurgent Animist — {2}{G} 3/3. Landfall: add any color; the
/// second land each turn also digs up an Elf or Elemental.
pub fn nissa_resurgent_animist() -> CardDefinition {
    CardDefinition {
        name: "Nissa, Resurgent Animist",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
            },
            // "The second time this resolves each turn" ≈ the second land
            // drop this turn.
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl)
                    .with_filter(Predicate::ValueAtLeast(
                        Value::LandsPlayedThisTurn(PlayerRef::You),
                        Value::Const(2),
                    ))
                    .once_per_turn(),
                effect: Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: SelectionRequirement::HasCreatureType(CreatureType::Elf).or(
                        SelectionRequirement::HasCreatureType(CreatureType::Elemental),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                    cap: Value::Const(999),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::BottomRandom,
                },
            },
        ],
        ..Default::default()
    }
}
