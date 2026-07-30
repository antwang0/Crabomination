//! Guildpact (GPT) gap wave 8: the Magemark enchanted-matters cycle,
//! Petrified Wood-Kin's scaling bloodthirst, and the rare utility tail.
//! Tests in `classic_sets/gpt`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    Keyword, MayPlayDuration, SelectionRequirement as R, StaticAbility, Subtypes, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector,
    StaticEffect, TriggeredAbility, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w};
/// Petrified Wood-Kin — {6}{G} 3/3 Elemental Warrior. Can't be countered,
/// Bloodthirst X (X = damage dealt to your opponents this turn), protection
/// from instants.
pub fn petrified_wood_kin() -> CardDefinition {
    CardDefinition {
        name: "Petrified Wood-Kin",
        cost: cost(&[generic(6), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::CantBeCountered, Keyword::ProtectionFromInstants],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::DamageTakenThisTurn(PlayerRef::EachOpponent),
        })],
        ..Default::default()
    }
}

/// Beastmaster's Magemark — {2}{G} Aura. Enchant creature. Your enchanted
/// creatures get +1/+1; when one becomes blocked it gets +1/+1 until end of
/// turn for each creature blocking it.
pub fn beastmasters_magemark() -> CardDefinition {
    CardDefinition {
        name: "Beastmaster's Magemark",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        static_abilities: vec![StaticAbility {
            description: "Creatures you control that are enchanted get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::IsEnchanted),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::EnchantedBySource),
            effect: Effect::PumpPT {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::BlockersOf(Box::new(Selector::AttachedTo(Box::new(Selector::This)))),
                toughness: Value::BlockersOf(Box::new(Selector::AttachedTo(Box::new(
                    Selector::This,
                )))),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Necromancer's Magemark — {2}{B} Aura. Enchant creature. Your enchanted
/// creatures get +1/+1 and return to their owners' hands instead of dying.
pub fn necromancers_magemark() -> CardDefinition {
    CardDefinition {
        name: "Necromancer's Magemark",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control that are enchanted get +1/+1.",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Creature.and(R::IsEnchanted),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                    opponents: false,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
            StaticAbility {
                description: "If a creature you control that's enchanted would die, return it to its owner's hand instead.",
                effect: StaticEffect::DiesToOwnersHandInstead {
                    filter: R::Creature.and(R::ControlledByYou).and(R::IsEnchanted),
                },
            },
        ],
        ..Default::default()
    }
}

/// Nivix, Aerie of the Firemind — Land. {T}: Add {C}. {2}{U}{R}, {T}: exile the
/// top card of your library; you may cast it until your next turn if it's an
/// instant or sorcery.
pub fn nivix_aerie_of_the_firemind() -> CardDefinition {
    CardDefinition {
        name: "Nivix, Aerie of the Firemind",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u(), r()]),
                tap_cost: true,
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Moratorium Stone — {1} Artifact. {2}, {T}: exile target card from a
/// graveyard. {2}{W}{B}, {T}, sacrifice it: exile a nonland card from a
/// graveyard along with every other copy of that name anywhere.
pub fn moratorium_stone() -> CardDefinition {
    CardDefinition {
        name: "Moratorium Stone",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::InGraveyard),
                    to: ZoneDest::Exile,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w(), b()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::ExileSameNameAsTarget {
                    what: target_filtered(R::InGraveyard.and(R::Nonland)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Living Inferno — {6}{R}{R} 8/5 Elemental. {T}: divide its power in damage
/// among any number of target creatures; each of them deals its power back.
pub fn living_inferno() -> CardDefinition {
    CardDefinition {
        name: "Living Inferno",
        cost: cost(&[generic(6), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 8,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamageDivided {
                total: Value::PowerOf(Box::new(Selector::This)),
                filter: R::Creature,
                max_targets: 8,
                retaliate_to_source: true,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mizzium Transreliquat — {3} Artifact. {3}: become a copy of target artifact
/// until end of turn. {1}{U}{R}: become that copy permanently, keeping this
/// ability.
pub fn mizzium_transreliquat() -> CardDefinition {
    CardDefinition {
        name: "Mizzium Transreliquat",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: target_filtered(R::Artifact),
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u(), r()]),
                effect: Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: target_filtered(R::Artifact),
                    duration: Duration::Permanent,
                    non_legendary: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Killer Instinct — {4}{R}{G} Enchantment. At your upkeep, reveal the top card
/// of your library; if it's a creature, put it onto the battlefield with haste
/// and sacrifice it at the next end step.
pub fn killer_instinct() -> CardDefinition {
    CardDefinition {
        name: "Killer Instinct",
        cost: cost(&[generic(4), r(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::RevealTopDeployIfMatch {
                filter: R::Creature,
                haste: true,
                sacrifice_at_next_end_step: true,
            },
        }],
        ..Default::default()
    }
}

/// Sword of the Paruns — {4} Equipment. Your tapped creatures get +2/+0 while
/// the equipped creature is tapped and your untapped ones get +0/+2 while it
/// is untapped. {3}: tap or untap the equipped creature. Equip {3}.
pub fn sword_of_the_paruns() -> CardDefinition {
    CardDefinition {
        name: "Sword of the Paruns",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        static_abilities: vec![
            StaticAbility {
                description: "As long as equipped creature is tapped, tapped creatures you control get +2/+0.",
                effect: StaticEffect::AnthemForFilterIf {
                    filter: R::Creature.and(R::Tapped),
                    power: 2,
                    toughness: 0,
                    keywords: vec![],
                    condition: Predicate::EntityMatches {
                        what: Selector::AttachedTo(Box::new(Selector::This)),
                        filter: R::Tapped,
                    },
                },
            },
            StaticAbility {
                description: "As long as equipped creature is untapped, untapped creatures you control get +0/+2.",
                effect: StaticEffect::AnthemForFilterIf {
                    filter: R::Creature.and(R::Untapped),
                    power: 0,
                    toughness: 2,
                    keywords: vec![],
                    condition: Predicate::EntityMatches {
                        what: Selector::AttachedTo(Box::new(Selector::This)),
                        filter: R::Untapped,
                    },
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::TapOrUntap {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Predatory Focus — {3}{G}{G} Sorcery. Your creatures may assign their combat
/// damage this turn as though they weren't blocked.
pub fn predatory_focus() -> CardDefinition {
    CardDefinition {
        name: "Predatory Focus",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            keyword: Keyword::AssignsDamageAsThoughUnblocked,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}
