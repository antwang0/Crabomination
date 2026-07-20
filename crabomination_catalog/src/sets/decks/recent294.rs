//! Ravnica batch 4: Simic Graft, Orzhov/Boros creatures, and utility spells.
//! Reuses existing primitives — Graft (`enters_with_counters` + `graft()`),
//! `WithCounter` regen targets, `PreventNextDamage`, `EachPermanent` group
//! grants, and `ChooseColorForSelf` mana. Tests in `recent_b/recent_294`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{draw, etb, graft, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// A 1/1 black Bat token with flying (Skeletal Vampire's brood).
fn bat_token() -> TokenDefinition {
    TokenDefinition {
        name: "Bat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

// ── Simic ───────────────────────────────────────────────────────────────────

/// Simic Ragworm — {3}{G} 3/3 Worm. {U}: Untap this creature.
pub fn simic_ragworm() -> CardDefinition {
    CardDefinition {
        name: "Simic Ragworm",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Worm], ..Default::default() },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Untap { what: Selector::This, up_to: None },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sporeback Troll — {3}{G} 0/0 Troll Mutant, Graft 2. {1}{G}: Regenerate
/// target creature with a +1/+1 counter on it.
pub fn sporeback_troll() -> CardDefinition {
    CardDefinition {
        name: "Sporeback Troll",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Mutant],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![graft()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Regenerate {
                what: target_filtered(R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Silhana Starfletcher — {2}{G} 1/3 Elf Druid Archer, reach. As it enters,
/// choose a color. {T}: Add one mana of the chosen color.
pub fn silhana_starfletcher() -> CardDefinition {
    CardDefinition {
        name: "Silhana Starfletcher",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid, CreatureType::Archer],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::ChooseColorForSelf)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::ChosenColorOfSource },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Plaxmanta — {1}{U} 2/2 Beast, flash. When it enters, creatures you control
/// gain shroud until end of turn; then sacrifice it unless {G} was spent to
/// cast it.
pub fn plaxmanta() -> CardDefinition {
    CardDefinition {
        name: "Plaxmanta",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            etb(Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Shroud,
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::Not(Box::new(Predicate::SourceCastWithColorSpent {
                        color: Color::Green,
                        at_least: 1,
                    }))),
                effect: Effect::SacrificePermanent { what: Selector::This },
            },
        ],
        ..Default::default()
    }
}

// ── Orzhov / Boros / Rakdos ─────────────────────────────────────────────────

/// Skeletal Vampire — {4}{B}{B} 3/3 Vampire Skeleton, flying. When it enters,
/// create two 1/1 black Bat tokens with flying. {3}{B}{B}, Sacrifice a Bat:
/// Create two Bats. Sacrifice a Bat: Regenerate this creature.
pub fn skeletal_vampire() -> CardDefinition {
    let make_bats =
        Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: bat_token() };
    CardDefinition {
        name: "Skeletal Vampire",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Skeleton],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(make_bats.clone())],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), b(), b()]),
                sac_other_filter: Some((R::HasCreatureType(CreatureType::Bat), 1)),
                effect: make_bats,
                ..Default::default()
            },
            ActivatedAbility {
                sac_other_filter: Some((R::HasCreatureType(CreatureType::Bat), 1)),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Divebomber Griffin — {3}{W}{W} 3/2 Griffin, flying. {T}, Sacrifice this
/// creature: It deals 3 damage to target attacking or blocking creature.
pub fn divebomber_griffin() -> CardDefinition {
    CardDefinition {
        name: "Divebomber Griffin",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Griffin], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Steeple Roc — {4}{W} 3/1 Bird with flying and first strike.
pub fn steeple_roc() -> CardDefinition {
    CardDefinition {
        name: "Steeple Roc",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        ..Default::default()
    }
}

/// Snapping Drake — {3}{U} 3/2 Drake with flying.
pub fn snapping_drake() -> CardDefinition {
    CardDefinition {
        name: "Snapping Drake",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Drake], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Scorched Rusalka — {R} 1/1 Spirit. {R}, Sacrifice a creature: This creature
/// deals 1 damage to target player or planeswalker.
pub fn scorched_rusalka() -> CardDefinition {
    CardDefinition {
        name: "Scorched Rusalka",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Withstand — {2}{W} Instant. Prevent the next 3 damage that would be dealt to
/// any target this turn, then draw a card.
pub fn withstand() -> CardDefinition {
    CardDefinition {
        name: "Withstand",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PreventNextDamage { target: target_any(), amount: Value::Const(3) },
            draw(1),
        ]),
        ..Default::default()
    }
}
