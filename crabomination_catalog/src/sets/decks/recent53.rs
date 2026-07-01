//! Monarch, artifact hate, and white-weenie staples. Tests in
//! `tests/recent53.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, w, x, Color, SpendRestriction};

/// By Force — {X}{R} Sorcery. Destroy X target artifacts.
pub fn by_force() -> CardDefinition {
    CardDefinition {
        name: "By Force",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DestroyTargets { filter: R::Artifact },
        ..Default::default()
    }
}

/// Palace Jailer — {2}{W}{W} 2/2 Human Soldier. ETB: become the monarch, then
/// exile target creature an opponent controls until an opponent becomes the
/// monarch.
pub fn palace_jailer() -> CardDefinition {
    CardDefinition {
        name: "Palace Jailer",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            etb(Effect::ExileUntilOpponentMonarch {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            }),
        ],
        ..Default::default()
    }
}

/// Loxodon Smiter — {1}{G}{W} 4/4 Elephant Soldier. Can't be countered. (The
/// discard→battlefield replacement is approximated to the uncounterable body.)
pub fn loxodon_smiter() -> CardDefinition {
    CardDefinition {
        name: "Loxodon Smiter",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::CantBeCountered],
        ..Default::default()
    }
}

/// Leonin Vanguard — {W} 1/1 Cat Soldier. At the beginning of combat on your
/// turn, if you control three or more creatures, it gets +1/+1 until end of
/// turn and you gain 1 life.
pub fn leonin_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Leonin Vanguard",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            R::Creature.and(R::ControlledByYou),
                        )),
                        filter: R::Creature.and(R::ControlledByYou),
                    },
                    Value::Const(3),
                ),
                then: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GainLife { who: Selector::Player(PlayerRef::You), amount: Value::ONE },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Marchesa's Decree — {3}{B} Enchantment. ETB become the monarch; whenever a
/// creature attacks you or a planeswalker you control, that creature's
/// controller loses 1 life.
pub fn marchesas_decree() -> CardDefinition {
    use crate::mana::b;
    CardDefinition {
        name: "Marchesa's Decree",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Giada, Font of Hope — {1}{W} 2/2 Legendary Angel. Flying, vigilance. Each
/// other Angel you control enters with an additional +1/+1 counter for each
/// Angel you already control. {T}: Add {W}, spend only to cast an Angel spell.
pub fn giada_font_of_hope() -> CardDefinition {
    CardDefinition {
        name: "Giada, Font of Hope",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Each other Angel you control enters with an additional +1/+1 counter on it for each Angel you already control.",
            effect: StaticEffect::TypeEntersWithCountersPerControlled {
                creature_type: CreatureType::Angel,
                kind: CounterType::PlusOnePlusOne,
                per: R::Creature.and(R::HasCreatureType(CreatureType::Angel)).and(R::ControlledByYou),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::White])),
                    SpendRestriction::CreatureOfType(CreatureType::Angel),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Old Rutstein — {1}{B}{G} 1/4 Legendary Human Peasant. When it enters and at
/// the beginning of your upkeep, mill a card: land → Treasure, creature → 1/1
/// green Insect, else → Blood.
pub fn old_rutstein() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::b;
    use crabomination_base::tokens::{blood_token, treasure_token};
    let insect = || TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        ..Default::default()
    };
    let mill_branch = move || Effect::MillThenBranchByType {
        land: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: treasure_token() }),
        creature: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: insect() }),
        noncreature: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: blood_token() }),
    };
    CardDefinition {
        name: "Old Rutstein",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![
            etb(mill_branch()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: mill_branch(),
            },
        ],
        ..Default::default()
    }
}

/// Custodi Lich — {3}{B}{B} 4/2 Zombie Cleric. ETB become the monarch; each
/// opponent sacrifices a creature. (The printed "whenever you become the
/// monarch, target player sacrifices" is approximated to the ETB edict.)
pub fn custodi_lich() -> CardDefinition {
    use crate::mana::b;
    CardDefinition {
        name: "Custodi Lich",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            etb(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Creature,
            }),
        ],
        ..Default::default()
    }
}
