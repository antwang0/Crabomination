//! Commander: 2026-10-19 (`016Xh16H`) — The Space Family Goblinson's dice
//! (CR 706). Tests in `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, may_once_each_turn, on_attack, target_filtered};
use crate::effect::{Duration, Effect, KeywordAct, PlayerRef, Predicate, Selector, Value};
use crate::mana::{Color, ManaCost, cost, g, generic, r};
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

/// "Roll a d`sides`" with one results arm per `(low, high, effect)`.
fn roll(sides: u8, results: Vec<(u8, u8, Effect)>) -> Effect {
    Effect::RollDie {
        sides,
        count: Value::ONE,
        modifier: Value::Const(0),
        reroll_at_most: 0,
        ignore_lowest: 0,
        results,
        on_doubles: None,
    }
}

/// "Whenever you roll a die" — one event per die (CR 706.2).
fn per_die() -> EventSpec {
    EventSpec::new(EventKind::Performed(KeywordAct::DieResult), EventScope::YourControl)
}

fn dice_rolled_this_turn() -> Value {
    Value::DiceRolledThisTurn(PlayerRef::You)
}

fn plus_one_counters(amount: Value) -> Effect {
    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount }
}

/// The Space Family Goblinson — {2}{R}{G} 1/1 legendary Goblin Guest. Has
/// trample while you've rolled three or more dice this turn; whenever you
/// roll a die, a +1/+1 counter.
pub fn the_space_family_goblinson() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Trample as long as you've rolled three or more dice this turn.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Trample,
                condition: Predicate::ValueAtLeast(dice_rolled_this_turn(), Value::Const(3)),
            },
        }],
        triggered_abilities: vec![TriggeredAbility { event: per_die(), effect: plus_one_counters(Value::ONE) }],
        ..creature(
            "The Space Family Goblinson",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Goblin, CreatureType::Guest],
            1,
            1,
        )
    }
}

/// Atomwheel Acrobats — {3}{G} 3/2 Elf Performer. Whenever you roll a 1 or
/// 2, that many +1/+1 counters; {2}{G}: roll a d6.
pub fn atomwheel_acrobats() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: per_die().with_filter(Predicate::Not(Box::new(Predicate::DieResultAtLeast(3)))),
            effect: plus_one_counters(Value::TriggerEventAmount),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: roll(6, vec![]),
            ..Default::default()
        }],
        ..creature(
            "Atomwheel Acrobats",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Performer],
            3,
            2,
        )
    }
}

/// Resolute Veggiesaur — {2}{G} 3/3 trample Dinosaur Employee. Whenever you
/// roll your third die each turn, a +1/+1 counter.
pub fn resolute_veggiesaur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: per_die().with_filter(Predicate::All(vec![
                Predicate::ValueAtLeast(dice_rolled_this_turn(), Value::Const(3)),
                Predicate::Not(Box::new(Predicate::ValueAtLeast(dice_rolled_this_turn(), Value::Const(4)))),
            ])),
            effect: plus_one_counters(Value::ONE),
        }],
        ..creature(
            "Resolute Veggiesaur",
            cost(&[generic(2), g()]),
            vec![CreatureType::Dinosaur, CreatureType::Employee],
            3,
            3,
        )
    }
}

fn goblins(n: i32) -> Effect {
    let token = TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        ..Default::default()
    };
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(token) }
}

/// Swarming Goblins — {4}{R} 4/3 Goblin. ETB roll a d20: 1-9 one 1/1 red
/// Goblin, 10-19 two, 20 three.
pub fn swarming_goblins() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(roll(20, vec![(1, 9, goblins(1)), (10, 19, goblins(2)), (20, 20, goblins(3))]))],
        ..creature("Swarming Goblins", cost(&[generic(4), r()]), vec![CreatureType::Goblin], 4, 3)
    }
}

/// Sylvan Shepherd — {2}{G} 2/3 vigilance Human Druid. Attacking, roll a d20:
/// 1-9 gain 1, 10-19 gain 2, 20 gain 5.
pub fn sylvan_shepherd() -> CardDefinition {
    let gain = |n: i32| Effect::GainLife { who: Selector::You, amount: Value::Const(n) };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_attack(roll(20, vec![(1, 9, gain(1)), (10, 19, gain(2)), (20, 20, gain(5))]))],
        ..creature("Sylvan Shepherd", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Druid], 2, 3)
    }
}

/// Vegetation Abomination — {2}{G} 2/2 deathtouch artifact creature Plant
/// Mutant Food. {T}, sacrifice it: roll a d6; gain life equal to the result.
pub fn vegetation_abomination() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Mutant],
            artifact_subtypes: vec![ArtifactSubtype::Food],
            ..Default::default()
        },
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: roll(6, vec![(1, 6, Effect::GainLife { who: Selector::You, amount: Value::LastDieRoll })]),
            ..Default::default()
        }],
        ..creature("Vegetation Abomination", cost(&[generic(2), g()]), vec![], 2, 2)
    }
}

/// Priority Boarding — {2}{R} enchantment. Whenever you roll a die, you may
/// reveal the top card of your library (once each turn); if its mana value is
/// less than the result, you may exile it and play it this turn.
pub fn priority_boarding() -> CardDefinition {
    let top = Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        name: "Priority Boarding",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![may_once_each_turn(
            EventSpec::new(EventKind::RolledDice, EventScope::YourControl),
            "Reveal the top card of your library?",
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::Sum(vec![Value::TriggerEventAmount, Value::Const(-1)]),
                    Value::ManaValueOf(Box::new(top)),
                ),
                then: Box::new(Effect::MayDo {
                    description: "Exile it? You may play it this turn.".into(),
                    body: Box::new(Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        duration: MayPlayDuration::EndOfThisTurn,
                        pay_any_color: false,
                        max_mana_value: None,
                        pay_own_cost: true,
                        uncast_penalty: None,
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
        )],
        ..Default::default()
    }
}

/// Farideh's Fireball — {3}{R}{R} instant. 5 damage to target creature or
/// planeswalker; roll a d20: 1-9 2 damage to each player, 10-20 to each
/// opponent.
pub fn faridehs_fireball() -> CardDefinition {
    let splash = |who: PlayerRef| Effect::DealDamage { to: Selector::Player(who), amount: Value::Const(2) };
    CardDefinition {
        name: "Farideh's Fireball",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature.or(R::Planeswalker)), amount: Value::Const(5) },
            roll(20, vec![(1, 9, splash(PlayerRef::EachPlayer)), (10, 20, splash(PlayerRef::EachOpponent))]),
        ]),
        ..Default::default()
    }
}

/// Critical Hit — {1}{R} instant. Target creature gains double strike until
/// end of turn. When you roll a natural 20, return it from your graveyard to
/// your hand.
pub fn critical_hit() -> CardDefinition {
    CardDefinition {
        name: "Critical Hit",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::DoubleStrike,
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![TriggeredAbility {
            // A natural 20 is the face, not the result (a d20 at its max face).
            event: EventSpec::new(EventKind::RolledNaturalMax, EventScope::FromYourGraveyard)
                .with_filter(Predicate::ValueAtLeast(Value::LastRollFaceCount(20), Value::ONE)),
            effect: Effect::Move { what: Selector::This, to: crate::effect::ZoneDest::Hand(PlayerRef::You) },
        }],
        ..Default::default()
    }
}

/// Slight Malfunction — {1}{R} sorcery. Choose one — destroy target
/// artifact; or roll a d6 and deal 1 damage to each of up to that many
/// creatures (opponents' creatures, not targeted).
pub fn slight_malfunction() -> CardDefinition {
    CardDefinition {
        name: "Slight Malfunction",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            roll(
                6,
                vec![(
                    1,
                    6,
                    Effect::DealDamage {
                        to: Selector::take(
                            Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                            Value::LastDieRoll,
                        ),
                        amount: Value::ONE,
                    },
                )],
            ),
        ]),
        ..Default::default()
    }
}
