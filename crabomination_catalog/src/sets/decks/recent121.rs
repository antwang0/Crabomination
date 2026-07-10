//! A Bloomburrow / Duskmourn batch: a graveyard-tuck changeling, a graveyard
//! recursion body, a reveal-until-land ramp ETB, a begin-combat pump, a modal
//! flash creature, a punisher Aura, a delirium fight, an uncounterable reanimating
//! Wurm, and a delirium-discounted removal spell. Tests in `tests/recent121.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    Keyword, Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition, PlayerRef, RevealMissDest,
    Selector, Value, ZoneDest,
};
use crate::mana::{b, cost, g, generic, r};

/// Barkform Harvester — {3} artifact Shapeshifter 2/3 with changeling and reach.
/// {2}: put target card from your graveyard on the bottom of your library.
pub fn barkform_harvester() -> CardDefinition {
    CardDefinition {
        name: "Barkform Harvester",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Changeling, Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Move {
                what: target_filtered(R::Any),
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Bottom },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bonebind Orator — {1}{B} 2/2 Squirrel Warlock Bard. {3}{B}, exile this from
/// your graveyard: return another target creature card from your graveyard to
/// your hand.
pub fn bonebind_orator() -> CardDefinition {
    CardDefinition {
        name: "Bonebind Orator",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Warlock, CreatureType::Bard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Clifftop Lookout — {2}{G} 1/2 Frog Scout with reach. ETB: reveal from the top
/// until you reveal a land, put it onto the battlefield tapped, the rest on the
/// bottom in a random order.
pub fn clifftop_lookout() -> CardDefinition {
    CardDefinition {
        name: "Clifftop Lookout",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: R::Land,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            cap: Value::Const(60),
            life_per_revealed: 0,
            miss_dest: RevealMissDest::BottomRandom,
        })],
        ..Default::default()
    }
}

/// Brambleguard Captain — {3}{R} 2/3 Mouse Soldier. At the beginning of combat
/// on your turn, target creature you control gets +X/+0, where X is this
/// creature's power.
pub fn brambleguard_captain() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Brambleguard Captain",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mouse, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl)
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Downwind Ambusher — {3}{B} 4/2 Skunk Assassin with flash. ETB: choose one —
/// target creature an opponent controls gets -1/-1; or destroy target creature
/// an opponent controls that was dealt damage this turn.
pub fn downwind_ambusher() -> CardDefinition {
    CardDefinition {
        name: "Downwind Ambusher",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skunk, CreatureType::Assassin],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::Destroy {
                what: target_filtered(
                    R::Creature.and(R::ControlledByOpponent).and(R::DealtDamageThisTurn),
                ),
            },
        ]))],
        ..Default::default()
    }
}

/// Cracked Skull — {2}{B} Aura. Enchant creature. When it enters, look at an
/// opponent's hand and make them discard a chosen nonland card. When the
/// enchanted creature is dealt damage, destroy it.
pub fn cracked_skull() -> CardDefinition {
    CardDefinition {
        name: "Cracked Skull",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![
            etb(Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Nonland,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::EnchantedBySource),
                effect: Effect::Destroy { what: Selector::TriggerSource },
            },
        ],
        ..Default::default()
    }
}

/// Beastie Beatdown — {R}{G} Sorcery. Delirium — if there are four or more card
/// types in your graveyard, put two +1/+1 counters on the creature you control.
/// Then the creature you control deals damage equal to its power to target
/// creature an opponent controls.
pub fn beastie_beatdown() -> CardDefinition {
    CardDefinition {
        name: "Beastie Beatdown",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::DeliriumActive { who: PlayerRef::You },
                then: Box::new(Effect::AddCounter {
                    what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::DealDamageEqualToPower {
                source: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
            },
        ]),
        ..Default::default()
    }
}

/// Balustrade Wurm — {3}{G}{G} 5/5 Wurm. Can't be countered; trample, haste.
/// Delirium — {2}{G}{G}: return this from your graveyard to the battlefield with
/// a finality counter (sorcery speed).
pub fn balustrade_wurm() -> CardDefinition {
    CardDefinition {
        name: "Balustrade Wurm",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::CantBeCountered, Keyword::Trample, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), g()]),
            from_graveyard: true,
            sorcery_speed: true,
            condition: Some(Predicate::DeliriumActive { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::Finality,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Drag to the Roots — {2}{B}{G} Instant. Delirium — costs {2} less while four
/// or more card types are in your graveyard. Destroy target nonland permanent.
pub fn drag_to_the_roots() -> CardDefinition {
    CardDefinition {
        name: "Drag to the Roots",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_delirium: Some(2),
        effect: Effect::Destroy { what: target_filtered(R::Nonland.and(R::Permanent)) },
        ..Default::default()
    }
}
