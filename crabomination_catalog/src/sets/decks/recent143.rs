//! A Wilds of Eldraine (WOE) Adventure wave. All ride existing primitives
//! (Adventure cast-mode, Role tokens, mill-then-take, `Value::LifeGainedThisTurn`,
//! `CantBeBlockedByPowerAtMost`). Tests in `crabomination/src/tests/recent143.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{deal, etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, ZoneDest};
use crate::game::effects::food_token;
use crate::mana::{b, cost, g, generic, r, u, w};

use super::woe_roles::{cursed_role, monster_role};

/// Ferocious Werefox // Guard Change — {3}{G} 4/3 Elf Fox Warrior with trample.
/// Adventure {1}{G} Instant: hang a Monster Role on a creature you control.
pub fn ferocious_werefox() -> CardDefinition {
    CardDefinition {
        name: "Ferocious Werefox",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Fox, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        adventure: Some(Box::new(Adventure {
            name: "Guard Change",
            cost: cost(&[generic(1), g()]),
            card_types: vec![CardType::Instant],
            effect: Effect::CreateTokenAttachedTo {
                target: target_filtered(R::Creature.and(R::ControlledByYou)),
                definition: monster_role(),
            },
        })),
        ..Default::default()
    }
}

/// Pollen-Shield Hare // Hare Raising — {1}{W} 2/2 Rabbit; creature tokens you
/// control get +1/+1. Adventure {G} Sorcery: a creature you control gains
/// vigilance and +X/+X, X = creatures you control.
pub fn pollen_shield_hare() -> CardDefinition {
    CardDefinition {
        name: "Pollen-Shield Hare",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::IsToken),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Hare Raising",
            cost: cost(&[g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::count(Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Battlefield,
                        filter: R::Creature.and(R::ControlledByYou),
                    }),
                    toughness: Value::count(Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Battlefield,
                        filter: R::Creature.and(R::ControlledByYou),
                    }),
                    duration: Duration::EndOfTurn,
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Frolicking Familiar // Blow Off Steam — {2}{U} 2/2 Otter Wizard with flying;
/// grows when you cast an instant or sorcery. Adventure {R} Instant: 1 damage.
pub fn frolicking_familiar() -> CardDefinition {
    CardDefinition {
        name: "Frolicking Familiar",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_instant_or_sorcery()),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Blow Off Steam",
            cost: cost(&[r()]),
            card_types: vec![CardType::Instant],
            effect: deal(1, target_any()),
        })),
        ..Default::default()
    }
}

/// Gumdrop Poisoner // Tempt with Treats — {2}{B} 3/2 Human Warlock with
/// lifelink; ETB up to one creature gets -X/-X, X = life you gained this turn.
/// Adventure {B} Instant: create a Food.
pub fn gumdrop_poisoner() -> CardDefinition {
    let minus_x = || {
        Value::Times(
            Box::new(Value::LifeGainedThisTurn(PlayerRef::You)),
            Box::new(Value::Const(-1)),
        )
    };
    CardDefinition {
        name: "Gumdrop Poisoner",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: minus_x(),
            toughness: minus_x(),
            duration: Duration::EndOfTurn,
        })],
        adventure: Some(Box::new(Adventure {
            name: "Tempt with Treats",
            cost: cost(&[b()]),
            card_types: vec![CardType::Instant],
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: food_token(),
            },
        })),
        ..Default::default()
    }
}

/// Vantress Transmuter // Croaking Curse — {3}{U} 3/4 Human Wizard. Adventure
/// {1}{U} Sorcery: tap a creature and hang a Cursed Role (it becomes 1/1).
pub fn vantress_transmuter() -> CardDefinition {
    CardDefinition {
        name: "Vantress Transmuter",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        adventure: Some(Box::new(Adventure {
            name: "Croaking Curse",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(R::Creature),
                },
                Effect::CreateTokenAttachedTo {
                    target: Selector::Target(0),
                    definition: cursed_role(),
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Picklock Prankster // Free the Fae — {1}{U} 1/3 Faerie Rogue with flying and
/// vigilance. Adventure {1}{U} Instant: mill four, then take an instant,
/// sorcery, or Faerie from among them.
pub fn picklock_prankster() -> CardDefinition {
    CardDefinition {
        name: "Picklock Prankster",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        adventure: Some(Box::new(Adventure {
            name: "Free the Fae",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::MillThenToHandN {
                amount: Value::Const(4),
                filter: R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .or(R::HasCreatureType(CreatureType::Faerie)),
                take: Value::ONE,
            },
        })),
        ..Default::default()
    }
}

/// Stormkeld Vanguard // Bear Down — {4}{G}{G} 6/7 Giant Warrior that can't be
/// blocked by power 2 or less. Adventure {1}{G} Sorcery: destroy an
/// artifact or enchantment.
pub fn stormkeld_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Stormkeld Vanguard",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 6,
        toughness: 7,
        keywords: vec![Keyword::CantBeBlockedByPowerAtMost(2)],
        adventure: Some(Box::new(Adventure {
            name: "Bear Down",
            cost: cost(&[generic(1), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment)),
            },
        })),
        ..Default::default()
    }
}

/// Scalding Viper // Steam Clean — {1}{R} 2/1 Elemental Snake; pings an opponent
/// who casts a mana value 3-or-less spell. Adventure {1}{U} Sorcery: bounce a
/// nonland permanent.
pub fn scalding_viper() -> CardDefinition {
    CardDefinition {
        name: "Scalding Viper",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Snake],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::CastSpellMatches(R::ManaValueAtMost(3))),
            effect: deal(1, Selector::Player(PlayerRef::Triggerer)),
        }],
        adventure: Some(Box::new(Adventure {
            name: "Steam Clean",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Move {
                what: target_filtered(R::Nonland),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        })),
        ..Default::default()
    }
}
