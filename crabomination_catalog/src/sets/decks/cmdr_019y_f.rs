//! Commander: 2026-10-17 (`019Y6J4R`) closers, sixth batch. Tests in
//! `tests/recent_b/cmdr_019y_f.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LandType, MayPlayDuration, Predicate,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{declare_target_opponent, etb_loot, sneak};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{ManaCost, b, cost, generic, r, u};
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
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn treasures(n: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: Arc::new(crabomination_base::tokens::treasure_token()),
    }
}

/// Conch Horn — {2} artifact. {1}, {T}, sacrifice it: draw two, then put a card
/// from your hand on top of your library.
pub fn conch_horn() -> CardDefinition {
    CardDefinition {
        name: "Conch Horn",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::PutOnLibraryFromHand { who: PlayerRef::You, count: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lunar Force — {2}{U} enchantment. When an opponent casts a spell, sacrifice
/// it and counter that spell.
pub fn lunar_force() -> CardDefinition {
    CardDefinition {
        name: "Lunar Force",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::Seq(vec![
                Effect::SacrificeSource,
                Effect::CounterSpell { what: Selector::TriggerSource },
            ]),
        }],
        ..Default::default()
    }
}

/// Oroku Saki, Shredder Rising — {2}{B} legendary 3/1 Human Ninja, sneak
/// {1}{B}. Combat damage to a player: draw a card and lose 1 life.
pub fn oroku_saki_shredder_rising() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Sneak(cost(&[generic(1), b()]))],
        alternative_cost: Some(sneak(cost(&[generic(1), b()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..creature(
            "Oroku Saki, Shredder Rising",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Ninja],
            3,
            1,
        )
    }
}

/// Korvold and the Noble Thief — {3}{R} Saga. I, II: a Treasure. III: exile
/// the top three cards of target opponent's library; you may play them this
/// turn.
pub fn korvold_and_the_noble_thief() -> CardDefinition {
    CardDefinition {
        name: "Korvold and the Noble Thief",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, treasures(1)),
            (2, treasures(1)),
            (
                3,
                Effect::Seq(vec![
                    declare_target_opponent(),
                    Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::Target(0),
                        count: Value::Const(3),
                        duration: MayPlayDuration::EndOfThisTurn,
                        pay_any_color: false,
                        max_mana_value: None,
                        pay_own_cost: true,
                        uncast_penalty: None,
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Stockman, Mad Fly-entist — {4}{U} legendary 3/4 flying. ETB: draw, then
/// discard. Islandcycling {2}.
pub fn stockman_mad_fly_entist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Landcycling(cost(&[generic(2)]), LandType::Island)],
        triggered_abilities: vec![etb_loot()],
        ..creature(
            "Stockman, Mad Fly-entist",
            cost(&[generic(4), u()]),
            vec![CreatureType::Insect, CreatureType::Mutant, CreatureType::Scientist],
            3,
            4,
        )
    }
}

/// Thorin, Company's Leader — {4}{R} legendary 4/5 Dwarf Warrior. A Dwarf you
/// control dealing combat damage to a player or battle makes two Treasures.
/// {10}: your creatures gain double strike until end of turn.
pub fn thorin_companys_leader() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).or_battle().with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Dwarf),
                },
            ),
            effect: treasures(2),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(10)]),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Thorin, Company's Leader",
            cost(&[generic(4), r()]),
            vec![CreatureType::Dwarf, CreatureType::Warrior],
            4,
            5,
        )
    }
}

/// Brain in a Jar — {2} artifact. {1}, {T}: a charge counter, then you may cast
/// an instant or sorcery with mana value equal to its charge counters from
/// your hand free. {3}, {T}, remove X charge counters: scry X.
pub fn brain_in_a_jar() -> CardDefinition {
    let charges = Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge };
    CardDefinition {
        name: "Brain in a Jar",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
                    Effect::WithX {
                        x: charges,
                        body: Box::new(Effect::MayCastFromHandFreeMatching {
                            filter: R::HasCardType(CardType::Instant)
                                .or(R::HasCardType(CardType::Sorcery))
                                .and(R::ManaValueExactlyXFromCost),
                            max_mv: Value::XFromCost,
                            else_: Box::new(Effect::Noop),
                        }),
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                remove_counter_x: Some(CounterType::Charge),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::XFromCost },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
