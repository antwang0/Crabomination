//! Commander: 2026-10-19 (`01DJHkWk`) — Inga Rune-Eyes' blue sacrifice deck
//! and Namor's Merfolk. Tests in `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, exploit, on_dies, target_filtered, token_copy_of};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, u, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn blue_token(name: &str, types: Vec<CreatureType>, p: i32, t: i32) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn merfolk() -> R {
    R::HasCreatureType(CreatureType::Merfolk)
}

fn my_merfolk() -> R {
    R::Creature.and(R::ControlledByYou).and(merfolk())
}

fn loot() -> Effect {
    Effect::Seq(vec![
        Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        },
        Effect::Discard {
            who: Selector::You,
            amount: Value::ONE,
            random: false,
        },
    ])
}

// ── Inga Rune-Eyes ──────────────────────────────────────────────────────────

/// Jeskai Sage — {1}{U} 1/1 prowess Human Monk; dying, draw a card.
pub fn jeskai_sage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        })],
        ..creature(
            "Jeskai Sage",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Monk],
            1,
            1,
        )
    }
}

/// Oracle of Tragedy — {1}{U} 1/3 Human Wizard. Entering or dying, choose one:
/// draw then discard; or shuffle up to four target cards with mana value 3 or
/// greater from your graveyard into your library.
pub fn oracle_of_tragedy() -> CardDefinition {
    let modes = || {
        Effect::ChooseMode(vec![
            loot(),
            Effect::ApplyToTargets {
                max_targets: 4,
                min_targets: 0,
                filter: R::InYourGraveyard.and(R::ManaValueAtLeast(3)),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Library {
                        who: PlayerRef::You,
                        pos: LibraryPosition::Shuffled,
                    },
                }),
            },
        ])
    };
    CardDefinition {
        triggered_abilities: vec![etb(modes()), on_dies(modes())],
        ..creature(
            "Oracle of Tragedy",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Sidisi's Faithful — {U} 0/4 Snake Wizard, exploit: return target creature
/// to its owner's hand.
pub fn sidisis_faithful() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![exploit(Effect::Move {
            what: target_filtered(R::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        })],
        ..creature(
            "Sidisi's Faithful",
            cost(&[u()]),
            vec![CreatureType::Snake, CreatureType::Wizard],
            0,
            4,
        )
    }
}

/// Stitcher's Apprentice — {1}{U} 1/2 Homunculus. {1}{U}, {T}: a 2/2 blue
/// Homunculus, then sacrifice a creature.
pub fn stitchers_apprentice() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(blue_token(
                        "Homunculus",
                        vec![CreatureType::Homunculus],
                        2,
                        2,
                    )),
                },
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Creature,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Stitcher's Apprentice",
            cost(&[generic(1), u()]),
            vec![CreatureType::Homunculus],
            1,
            2,
        )
    }
}

/// Homarid Spawning Bed — {U}{U} enchantment. {1}{U}{U}, sacrifice a blue
/// creature: X 1/1 blue Camarids, X the sacrificed creature's mana value.
pub fn homarid_spawning_bed() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u(), u()]),
            sac_other_filter: Some((R::Creature.and(R::HasColor(Color::Blue)), 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::SacrificedManaValue,
                definition: Arc::new(blue_token("Camarid", vec![CreatureType::Camarid], 1, 1)),
            },
            ..Default::default()
        }],
        ..spell(
            "Homarid Spawning Bed",
            cost(&[u(), u()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── Namor, Atlantean King ───────────────────────────────────────────────────

/// Deepchannel Duelist — {W}{U} 2/2 Merfolk Soldier. Your end step untaps
/// target Merfolk you control; other Merfolk you control get +1/+1.
pub fn deepchannel_duelist() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Merfolk you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(my_merfolk().and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::YourControl,
            ),
            effect: Effect::Untap {
                what: target_filtered(my_merfolk()),
                up_to: None,
            },
        }],
        ..creature(
            "Deepchannel Duelist",
            cost(&[w(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Deepway Navigator — {W}{U} flash 2/2 Merfolk Wizard. Entering, untap each
/// other Merfolk you control. Merfolk you control get +1/+0 while you've
/// attacked with three or more Merfolk this turn.
pub fn deepway_navigator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Untap {
            what: Selector::EachPermanent(my_merfolk().and(R::OtherThanSource)),
            up_to: None,
        })],
        static_abilities: vec![StaticAbility {
            description: "As long as you attacked with three or more Merfolk this turn, Merfolk you control get +1/+0.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(my_merfolk().and(R::AttackedThisTurn)),
                    n: Value::Const(3),
                },
                applies_to: Selector::EachPermanent(my_merfolk()),
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..creature(
            "Deepway Navigator",
            cost(&[w(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Sygg, River Guide — {W}{U} legendary 2/2 islandwalk Merfolk Wizard. {1}{W}:
/// target Merfolk you control gains protection from the color of your choice
/// until end of turn.
pub fn sygg_river_guide() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::GrantProtectionFromChosenColor {
                what: target_filtered(my_merfolk()),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Sygg, River Guide",
            cost(&[w(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Summon the School — {3}{W} Kindred sorcery — Merfolk. Two 1/1 blue Merfolk
/// Wizards. Tap four untapped Merfolk you control: return it from your
/// graveyard to your hand.
pub fn summon_the_school() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            tap_n_filter: Some((my_merfolk(), 4)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..spell(
            "Summon the School",
            cost(&[generic(3), w()]),
            CardType::Sorcery,
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: Arc::new(blue_token(
                    "Merfolk Wizard",
                    vec![CreatureType::Merfolk, CreatureType::Wizard],
                    1,
                    1,
                )),
            },
        )
    }
}

/// Sygg's Command — {1}{W}{U} Kindred sorcery — Merfolk. Choose two: a token
/// copy of target Merfolk you control; creatures target player controls gain
/// lifelink; target player draws a card; tap target creature and put a stun
/// counter on it.
pub fn syggs_command() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk],
            ..Default::default()
        },
        ..spell(
            "Sygg's Command",
            cost(&[generic(1), w(), u()]),
            CardType::Sorcery,
            Effect::ChooseModesCast {
                modes: vec![
                    token_copy_of(PlayerRef::You, Value::ONE, target_filtered(my_merfolk())),
                    Effect::TargetPlayerThen {
                        filter: R::Player,
                        then: Box::new(Effect::GrantKeyword {
                            what: Selector::ControlledBy {
                                who: PlayerRef::Target(0),
                                filter: R::Creature,
                            },
                            keyword: Keyword::Lifelink,
                            duration: Duration::EndOfTurn,
                        }),
                    },
                    Effect::Draw {
                        who: target_filtered(R::Player),
                        amount: Value::ONE,
                    },
                    Effect::Seq(vec![
                        Effect::Tap {
                            what: target_filtered(R::Creature),
                        },
                        Effect::AddCounter {
                            what: Selector::Target(0),
                            kind: CounterType::Stun,
                            amount: Value::ONE,
                        },
                    ]),
                ],
                min: 2,
                max: 2,
                allow_repeats: false,
            },
        )
    }
}
