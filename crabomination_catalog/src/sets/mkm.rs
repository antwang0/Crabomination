//! Murders at Karlov Manor (MKM) — 2024. Detective set introducing the
//! Suspect (CR 701.60) and Collect Evidence (CR 701.59) keyword actions.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::card::{ActivatedAbility, TokenDefinition};
use crate::effect::shortcut::{draw, etb, lose_life, on_attack, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate};
use crate::mana::{b, cost, g, generic, r, u, w, Color};
use crate::game::effects::clue_token;

/// Repeat Offender — {1}{B} 2/1 Human Assassin. "{2}{B}: If this creature is
/// suspected, put a +1/+1 counter on it. Otherwise, suspect it."
pub fn repeat_offender() -> CardDefinition {
    CardDefinition {
        name: "Repeat Offender",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::If {
                cond: Predicate::SourceIsSuspected,
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Suspect { what: Selector::This }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Reasonable Doubt — {1}{U} Instant. "Counter target spell unless its
/// controller pays {2}. Suspect up to one target creature." (The "up to one"
/// rider is modeled as a required creature target.)
pub fn reasonable_doubt() -> CardDefinition {
    CardDefinition {
        name: "Reasonable Doubt",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                mana_cost: cost(&[generic(2)]),
            },
            Effect::Suspect {
                what: Selector::TargetFiltered { slot: 1, filter: SelectionRequirement::Creature },
            },
        ]),
        ..Default::default()
    }
}

/// Sample Collector — {2}{G} 2/3 Troll Detective. "Whenever this attacks, you
/// may collect evidence 3. When you do, put a +1/+1 counter on target
/// creature you control."
pub fn sample_collector() -> CardDefinition {
    CardDefinition {
        name: "Sample Collector",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Detective],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::CollectEvidence {
            amount: Value::Const(3),
            then: Box::new(Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..Default::default()
    }
}

/// Barbed Servitor — {3}{B} 1/1 Artifact Creature — Construct. Indestructible;
/// ETB suspect itself; combat damage to a player → draw + lose 1 life; when
/// dealt damage, each opponent loses that much life (modeled as each opponent
/// rather than a single target).
pub fn barbed_servitor() -> CardDefinition {
    CardDefinition {
        name: "Barbed Servitor",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![
            etb(Effect::Suspect { what: Selector::This }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![draw(1), lose_life(1, Selector::You)]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::TriggerEventAmount,
                },
            },
        ],
        ..Default::default()
    }
}

// ── Investigate (Clue tokens) ────────────────────────────────────────────────

/// Deduce — {1}{U} Instant. "Draw a card. Investigate." (Investigate mints a
/// Clue token via `clue_token()`.)
pub fn deduce() -> CardDefinition {
    CardDefinition {
        name: "Deduce",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            draw(1),
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: clue_token() },
        ]),
        ..Default::default()
    }
}

/// Novice Inspector — {W} 1/2 Human Detective. "When this enters, investigate."
pub fn novice_inspector() -> CardDefinition {
    CardDefinition {
        name: "Novice Inspector",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: clue_token(),
        })],
        ..Default::default()
    }
}

/// Izoni, Center of the Web — {4}{B}{G} 5/4 Legendary Elf Detective with
/// menace. "Whenever Izoni enters or attacks, you may collect evidence 4. If
/// you do, create two 2/1 black and green Spider tokens with menace and reach."
/// (The sacrifice-four-tokens activated ability is omitted.)
pub fn izoni_center_of_the_web() -> CardDefinition {
    let spider = || TokenDefinition {
        name: "Spider".into(),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace, Keyword::Reach],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        ..Default::default()
    };
    let collect = || Effect::CollectEvidence {
        amount: Value::Const(4),
        then: Box::new(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: spider(),
        }),
    };
    CardDefinition {
        name: "Izoni, Center of the Web",
        cost: cost(&[generic(4), b(), g()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Detective],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(collect()), on_attack(collect())],
        ..Default::default()
    }
}

// ── More MKM ─────────────────────────────────────────────────────────────────

/// A 2/2 white-and-blue Detective creature token (Person of Interest, Inside
/// Source).
fn detective_token() -> TokenDefinition {
    TokenDefinition {
        name: "Detective".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Detective], ..Default::default() },
        ..Default::default()
    }
}

/// Cold Case Cracker — {3}{U} 3/3 Spirit Detective with flying. "When this
/// dies, investigate."
pub fn cold_case_cracker() -> CardDefinition {
    use crate::effect::shortcut::on_dies;
    CardDefinition {
        name: "Cold Case Cracker",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Detective],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: clue_token(),
        })],
        ..Default::default()
    }
}

/// Not on My Watch — {1}{W} Instant. "Exile target attacking creature."
pub fn not_on_my_watch() -> CardDefinition {
    CardDefinition {
        name: "Not on My Watch",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::IsAttacking),
            },
        },
        ..Default::default()
    }
}

/// Person of Interest — {3}{R} 2/2 Human Rogue. "When this enters, suspect it.
/// Create a 2/2 white and blue Detective creature token."
pub fn person_of_interest() -> CardDefinition {
    CardDefinition {
        name: "Person of Interest",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Suspect { what: Selector::This },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: detective_token(),
            },
        ]))],
        ..Default::default()
    }
}

/// Get a Leg Up — {G} Instant. "Until end of turn, target creature gets +1/+1
/// for each creature you control and gains reach."
pub fn get_a_leg_up() -> CardDefinition {
    use crate::effect::Duration;
    let count = Value::CreatureCountControlledBy(PlayerRef::You);
    CardDefinition {
        name: "Get a Leg Up",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: count.clone(),
                toughness: count,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Inside Source — {2}{W} 1/1 Human Citizen. "When this enters, create a 2/2
/// white and blue Detective creature token." (The pump-a-Detective activated
/// ability is omitted.)
pub fn inside_source() -> CardDefinition {
    CardDefinition {
        name: "Inside Source",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: detective_token(),
        })],
        ..Default::default()
    }
}

/// Slimy Dualleech — {3}{B} 2/4 Leech. "At the beginning of combat on your
/// turn, target creature you control with power 2 or less gets +1/+0 and gains
/// deathtouch until end of turn."
pub fn slimy_dualleech() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec};
    use crate::effect::Duration;
    let target = || Selector::TargetFiltered {
        slot: 0,
        filter: SelectionRequirement::Creature
            .and(SelectionRequirement::ControlledByYou)
            .and(SelectionRequirement::PowerAtMost(2)),
    };
    CardDefinition {
        name: "Slimy Dualleech",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Leech], ..Default::default() },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target(),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target(),
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}
