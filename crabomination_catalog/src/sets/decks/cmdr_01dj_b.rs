//! Commander: 2026-10-19 (`01DJHkWk`) two-deck-gap closers — Daredevil,
//! Gabriel Angelfire, Casey & Raph, Splinter. Tests in
//! `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword,
    MayPlayDuration, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{etb, mint_treasures, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{ManaCost, b, cost, g, generic, r, w};

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

/// Sneak [cost] that remembers it was paid (CR 702.190): `SpellWasKicked`.
fn sneak(mana: ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost {
        marks_kicked: true,
        ..crate::effect::shortcut::sneak(mana)
    }
}

/// Fight to the Death — {R}{W} instant. Destroy all blocking creatures and all
/// blocked creatures.
pub fn fight_to_the_death() -> CardDefinition {
    spell(
        "Fight to the Death",
        cost(&[r(), w()]),
        CardType::Instant,
        Effect::Destroy {
            what: Selector::EachPermanent(R::Creature.and(R::IsBlocking.or(R::IsBlocked))),
        },
    )
}

/// The Super Hero Civil War — {3}{R}{W} Saga. I: gain control of up to two
/// target creatures with total mana value 6 or less while this Saga remains.
/// II: your creatures get +1/+1 and vigilance until end of turn. III: target
/// creature you control fights up to one other target creature.
pub fn the_super_hero_civil_war() -> CardDefinition {
    let team = || Selector::EachPermanent(mine());
    CardDefinition {
        name: "The Super Hero Civil War",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature.and(R::SlotsTotalManaValueAtMost(6)),
                    effect: Box::new(Effect::GainControlWhileSourceRemains {
                        what: Selector::Target(0),
                    }),
                },
            ),
            (
                2,
                Effect::Seq(vec![
                    Effect::PumpPT {
                        what: team(),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: team(),
                        keyword: Keyword::Vigilance,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            ),
            (
                3,
                Effect::OptionalTargets {
                    min: 1,
                    body: Box::new(Effect::Fight {
                        attacker: target_filtered(mine()),
                        defender: Selector::TargetFiltered {
                            slot: 1,
                            filter: R::Creature,
                        },
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Helm of Chatzuk — {1} artifact. {1}, {T}: target creature gains banding
/// until end of turn.
pub fn helm_of_chatzuk() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Banding,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..spell(
            "Helm of Chatzuk",
            cost(&[generic(1)]),
            CardType::Artifact,
            Effect::Noop,
        )
    }
}

/// Revenge of the Hunted — {4}{G}{G} sorcery. Target creature gets +6/+6 and
/// trample, and all creatures able to block it do so, until end of turn.
/// Miracle {G}.
pub fn revenge_of_the_hunted() -> CardDefinition {
    let it = || Selector::Target(0);
    CardDefinition {
        miracle: Some(cost(&[g()])),
        ..spell(
            "Revenge of the Hunted",
            cost(&[generic(4), g(), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(6),
                    toughness: Value::Const(6),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: it(),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: it(),
                    keyword: Keyword::AllMustBlock,
                    duration: Duration::EndOfTurn,
                },
            ]),
        )
    }
}

/// Crossover Collaboration — {2}{R} instant. Teamwork 2. Exile your top two;
/// you may play them until the end of your next turn. Teamwork: a Treasure.
pub fn crossover_collaboration() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower {
            filter: mine(),
            power: 2,
        }),
        ..spell(
            "Crossover Collaboration",
            cost(&[generic(2), r()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(mint_treasures(1)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Storm of Steel — {3}{R} sorcery. 2 damage to each of one or two targets.
pub fn storm_of_steel() -> CardDefinition {
    spell(
        "Storm of Steel",
        cost(&[generic(3), r()]),
        CardType::Sorcery,
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 1,
            filter: R::any_target(),
            effect: Box::new(Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(2),
            }),
        },
    )
}

/// Karai, Future of the Foot — {1}{W}{B} legendary 3/3 Human Ninja. Sneak
/// {2}{W}{B}. Her combat damage to a player returns target creature card from
/// your graveyard to your hand — to the battlefield if she was sneaked in this
/// turn.
pub fn karai_future_of_the_foot() -> CardDefinition {
    let back = target_filtered(R::Creature.and(R::InYourGraveyard));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        alternative_cost: Some(sneak(cost(&[generic(2), w(), b()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::All(vec![
                    Predicate::SpellWasKicked,
                    Predicate::EntityMatches {
                        what: Selector::This,
                        filter: R::EnteredThisTurn,
                    },
                ]),
                then: Box::new(Effect::Move {
                    what: back.clone(),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
                else_: Box::new(Effect::Move {
                    what: back,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..creature(
            "Karai, Future of the Foot",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Human, CreatureType::Ninja],
            3,
            3,
        )
    }
}

/// Turncoat Kunoichi — {2}{W} 3/2 Mutant Ninja Fox. Sneak {2}{W}{B}. Enters:
/// exile target creature an opponent controls until this leaves — for good if
/// its sneak cost was paid.
pub fn turncoat_kunoichi() -> CardDefinition {
    let victim = || target_filtered(R::Creature.and(R::ControlledByOpponent));
    CardDefinition {
        alternative_cost: Some(sneak(cost(&[generic(2), w(), b()]))),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Move {
                what: victim(),
                to: ZoneDest::Exile,
            }),
            else_: Box::new(Effect::ExileUntilSourceLeaves {
                what: victim(),
                return_to: ExileReturnZone::Battlefield,
            }),
        })],
        ..creature(
            "Turncoat Kunoichi",
            cost(&[generic(2), w()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Fox],
            3,
            2,
        )
    }
}
