//! Return to Ravnica (RTR) gap wave 4: more creatures — hybrids, unleash +
//! regenerate, a token anthem, an edict, detain, and cast/attack payoffs on
//! existing primitives. Tests in `classic_sets/rtr`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility, Value,
};
use crate::card::{EventKind, EventScope, EventSpec};
use crate::effect::shortcut::{etb, on_attack, target_filtered, unleash};
use crate::effect::{Duration, PlayerRef, Selector, StaticEffect};
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w};

/// Risen Sanctuary — {5}{G}{W} 8/8 Elemental with vigilance.
pub fn risen_sanctuary() -> CardDefinition {
    CardDefinition {
        name: "Risen Sanctuary",
        cost: cost(&[generic(5), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    }
}

/// Rakdos Shred-Freak — {B/R}{B/R} 2/1 Human Berserker with haste.
pub fn rakdos_shred_freak() -> CardDefinition {
    CardDefinition {
        name: "Rakdos Shred-Freak",
        cost: cost(&[
            hybrid(Color::Black, Color::Red),
            hybrid(Color::Black, Color::Red),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

/// Golgari Longlegs — {3}{B/G}{B/G} 5/4 Insect (vanilla).
pub fn golgari_longlegs() -> CardDefinition {
    CardDefinition {
        name: "Golgari Longlegs",
        cost: cost(&[
            generic(3),
            hybrid(Color::Black, Color::Green),
            hybrid(Color::Black, Color::Green),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        ..Default::default()
    }
}

/// Grim Roustabout — {1}{B} 1/1 Skeleton Warrior with unleash. {1}{B}:
/// regenerate this.
pub fn grim_roustabout() -> CardDefinition {
    CardDefinition {
        name: "Grim Roustabout",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Unleash],
        triggered_abilities: vec![unleash()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Frostburn Weird — {U/R}{U/R} 1/4 Weird. {U/R}: +1/-1 until end of turn.
pub fn frostburn_weird() -> CardDefinition {
    CardDefinition {
        name: "Frostburn Weird",
        cost: cost(&[
            hybrid(Color::Blue, Color::Red),
            hybrid(Color::Blue, Color::Red),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Weird],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[hybrid(Color::Blue, Color::Red)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rakdos Ringleader — {4}{B}{R} 3/1 Skeleton Warrior with first strike.
/// Combat damage to a player makes them discard at random; {B}: regenerate.
pub fn rakdos_ringleader() -> CardDefinition {
    CardDefinition {
        name: "Rakdos Ringleader",
        cost: cost(&[generic(4), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::ONE,
                random: true,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Soulsworn Spirit — {3}{U} 2/1 Spirit that can't be blocked. When it enters,
/// detain target creature an opponent controls.
pub fn soulsworn_spirit() -> CardDefinition {
    CardDefinition {
        name: "Soulsworn Spirit",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![etb(Effect::Detain {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        })],
        ..Default::default()
    }
}

/// Skymark Roc — {2}{W}{U} 3/3 Bird with flying. Whenever it attacks, you may
/// return target creature an opponent controls with toughness 2 or less to hand.
pub fn skymark_roc() -> CardDefinition {
    CardDefinition {
        name: "Skymark Roc",
        cost: cost(&[generic(2), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Return an opponent's toughness-2-or-less creature to hand".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(
                    R::Creature
                        .and(R::ControlledByOpponent)
                        .and(R::ToughnessAtMost(2)),
                ),
                to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..Default::default()
    }
}

/// Phantom General — {3}{W} 2/3 Spirit Soldier. Creature tokens you control get
/// +1/+1.
pub fn phantom_general() -> CardDefinition {
    CardDefinition {
        name: "Phantom General",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::IsToken),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Slum Reaper — {3}{B} 4/2 Horror. When it enters, each player sacrifices a
/// creature of their choice.
pub fn slum_reaper() -> CardDefinition {
    CardDefinition {
        name: "Slum Reaper",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::ONE,
            filter: R::Creature,
        })],
        ..Default::default()
    }
}

/// Chaos Imps — {4}{R}{R} 6/5 Imp with flying and unleash; has trample while it
/// has a +1/+1 counter on it.
pub fn chaos_imps() -> CardDefinition {
    CardDefinition {
        name: "Chaos Imps",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Imp],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Unleash],
        triggered_abilities: vec![unleash()],
        static_abilities: vec![StaticAbility {
            description: "This creature has trample as long as it has a +1/+1 counter on it.",
            effect: StaticEffect::SelfHasKeywordWhileCountersAtLeast {
                kind: crate::card::CounterType::PlusOnePlusOne,
                n: 1,
                keyword: Keyword::Trample,
            },
        }],
        ..Default::default()
    }
}
