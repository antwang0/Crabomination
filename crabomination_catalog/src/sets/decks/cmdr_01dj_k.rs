//! Commander: 2026-10-19 (`01DJHkWk`) — Rocksteady's boars, Foggy Nelson's
//! white weenies, Spider-Woman's tappers. Tests in
//! `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, ModesWiden,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, cost, g, generic, hybrid, u, w};
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

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn plus_one(what: Selector, n: Value) -> Effect {
    Effect::AddCounter {
        what,
        kind: CounterType::PlusOnePlusOne,
        amount: n,
    }
}

fn boar(extra: Vec<TriggeredAbility>, p: i32, t: i32) -> TokenDefinition {
    TokenDefinition {
        name: "Boar".into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar],
            ..Default::default()
        },
        triggered_abilities: extra,
        ..Default::default()
    }
}

// ── Rocksteady, Crash Courser ───────────────────────────────────────────────

/// Carefree Swinemaster — {2}{G} 1/4 Gnome Ranger. Attacking, you may pay
/// {1}{G} for a 2/2 green Boar tapped and attacking.
pub fn carefree_swinemaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MayPay {
            description: "Pay {1}{G} for an attacking Boar?".into(),
            mana_cost: cost(&[generic(1), g()]),
            body: Box::new(Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(boar(vec![], 2, 2)),
                cleanup: Default::default(),
                defender: None,
            }),
            else_: None,
        })],
        ..creature(
            "Carefree Swinemaster",
            cost(&[generic(2), g()]),
            vec![CreatureType::Gnome, CreatureType::Ranger],
            1,
            4,
        )
    }
}

/// Festerhide Boar — {3}{G} 3/3 trample Boar. Morbid — enters with two +1/+1
/// counters if a creature died this turn.
pub fn festerhide_boar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::CreaturesDiedThisTurnTotalAtLeast {
                    at_least: Value::ONE,
                }),
                then: Box::new(Value::Const(2)),
                else_: Box::new(Value::Const(0)),
            },
        )),
        ..creature(
            "Festerhide Boar",
            cost(&[generic(3), g()]),
            vec![CreatureType::Boar],
            3,
            3,
        )
    }
}

/// Nettle Swine — {3}{G} 4/3 Boar.
pub fn nettle_swine() -> CardDefinition {
    creature(
        "Nettle Swine",
        cost(&[generic(3), g()]),
        vec![CreatureType::Boar],
        4,
        3,
    )
}

/// Trufflesnout — {2}{G} 2/2 Boar. Entering: a +1/+1 counter on it, or gain 4
/// life.
pub fn trufflesnout() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            plus_one(Selector::This, Value::ONE),
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
            },
        ]))],
        ..creature(
            "Trufflesnout",
            cost(&[generic(2), g()]),
            vec![CreatureType::Boar],
            2,
            2,
        )
    }
}

/// Wolf's Quarry — {4}{G}{G} sorcery. Three 1/1 green Boars that leave a Food
/// when they die.
pub fn wolfs_quarry() -> CardDefinition {
    let food = Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(crabomination_base::tokens::food_token()),
    };
    spell(
        "Wolf's Quarry",
        cost(&[generic(4), g(), g()]),
        CardType::Sorcery,
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: Arc::new(boar(vec![on_dies(food)], 1, 1)),
        },
    )
}

// ── Foggy Nelson, On Retainer ───────────────────────────────────────────────

/// Action News Crew — {1}{W} 2/2 vigilance Human Citizen. Channel — {6},
/// discard it: a +1/+1 counter on each creature you control; draw a card.
pub fn action_news_crew() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                plus_one(Selector::EachPermanent(mine()), Value::ONE),
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Action News Crew",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Citizen],
            2,
            2,
        )
    }
}

/// Blessed Spirits — {2}{W} 2/2 flying Spirit; your enchantment spells put a
/// +1/+1 counter on it.
pub fn blessed_spirits() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Enchantment)),
            effect: plus_one(Selector::This, Value::ONE),
        }],
        ..creature(
            "Blessed Spirits",
            cost(&[generic(2), w()]),
            vec![CreatureType::Spirit],
            2,
            2,
        )
    }
}

/// Elder Cathar — {2}{W} 2/2 Human Soldier. Dying, a +1/+1 counter on target
/// creature you control — two if it's a Human.
pub fn elder_cathar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(plus_one(
            target_filtered(mine()),
            Value::IfPred {
                pred: Box::new(Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasCreatureType(CreatureType::Human),
                }),
                then: Box::new(Value::Const(2)),
                else_: Box::new(Value::ONE),
            },
        ))],
        ..creature(
            "Elder Cathar",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Bound in Silence — {2}{W} Kindred Enchantment — Rebel Aura: the enchanted
/// creature can't attack or block.
pub fn bound_in_silence() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            creature_types: vec![CreatureType::Rebel],
            ..Default::default()
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        ..spell(
            "Bound in Silence",
            cost(&[generic(2), w()]),
            CardType::Enchantment,
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Creature),
            },
        )
    }
}

/// Angel's Mercy — {2}{W}{W} instant. You gain 7 life.
pub fn angels_mercy() -> CardDefinition {
    spell(
        "Angel's Mercy",
        cost(&[generic(2), w(), w()]),
        CardType::Instant,
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(7),
        },
    )
}

// ── Spider-Woman, Secret Agent ──────────────────────────────────────────────

/// Spider-Woman, Secret Agent — {3}{W/U} flash legendary 1/4 Spider Human Spy
/// Hero. Entering, tap target creature an opponent controls; it can't untap
/// for as long as you control her.
pub fn spider_woman_secret_agent() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::TapAndLockWhileSourcePresent {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            while_you_control: true,
        })],
        ..creature(
            "Spider-Woman, Secret Agent",
            cost(&[generic(3), hybrid(Color::White, Color::Blue)]),
            vec![
                CreatureType::Spider,
                CreatureType::Human,
                CreatureType::Spy,
                CreatureType::Hero,
            ],
            1,
            4,
        )
    }
}

/// Raft Security Officer — {1}{W} 1/3 Human Soldier. {2}, {T}: tap target
/// creature; {1} less if it has power 3 or less.
pub fn raft_security_officer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            cost_reduction_if_targets: Some((R::PowerAtMost(3), 1)),
            effect: Effect::Tap {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..creature(
            "Raft Security Officer",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            3,
        )
    }
}

/// Frozen in Ice — {2}{U} Aura. Entering, tap the creature; it loses all
/// abilities and doesn't untap during its controller's untap step.
pub fn frozen_in_ice() -> CardDefinition {
    let it = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        equipped_bonus: Some(EquipBonus {
            remove_abilities: true,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Tap { what: it() })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature can't become untapped.",
            effect: StaticEffect::PreventUntap { applies_to: it() },
        }],
        ..spell(
            "Frozen in Ice",
            cost(&[generic(2), u()]),
            CardType::Enchantment,
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Creature),
            },
        )
    }
}

/// Atlantis Attacks — {5}{U}{U} sorcery, teamwork 4. Choose one — both with
/// teamwork: target player creates a 6/5 blue hexproof Leviathan; return one
/// or two target nonland permanents to their owners' hands.
pub fn atlantis_attacks() -> CardDefinition {
    let leviathan = TokenDefinition {
        name: "Leviathan".into(),
        power: 6,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Leviathan],
            ..Default::default()
        },
        keywords: vec![Keyword::Hexproof],
        ..Default::default()
    };
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower {
            filter: mine(),
            power: 4,
        }),
        modes_widen: Some(ModesWiden::Kicked),
        ..spell(
            "Atlantis Attacks",
            cost(&[generic(5), u(), u()]),
            CardType::Sorcery,
            Effect::ChooseModesCast {
                modes: vec![
                    Effect::TargetPlayerThen {
                        filter: R::Player,
                        then: Box::new(Effect::CreateToken {
                            who: PlayerRef::Target(0),
                            count: Value::ONE,
                            definition: Arc::new(leviathan),
                        }),
                    },
                    Effect::ApplyToTargets {
                        max_targets: 2,
                        min_targets: 1,
                        filter: R::Permanent.and(R::Nonland),
                        effect: Box::new(Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                        }),
                    },
                ],
                min: 1,
                max: 1,
                allow_repeats: false,
            },
        )
    }
}
