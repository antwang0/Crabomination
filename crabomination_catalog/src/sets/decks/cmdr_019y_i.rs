//! Commander: 2026-10-17 (`019Y6J4R`) closers, ninth batch. Tests in
//! `tests/recent_b/cmdr_019y_i.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::mana::{SpendRestriction, cost, g, generic, r, u, w};
use std::sync::Arc;

fn yours() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::ControlledByYou))
}

/// The Brothers' War — {3}{R} Saga. I: two tapped Powerstones. II: two target
/// players; until your next turn each creature they control attacks the other
/// chosen player each combat if able (the creatures they control as it
/// resolves). III: X damage to any target and X to any other target, X the
/// artifacts you control.
pub fn the_brothers_war() -> CardDefinition {
    let powerstone = TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() };
    let artifacts = || Value::count(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)));
    let player = |slot| Selector::TargetFiltered { slot, filter: R::Player };
    CardDefinition {
        name: "The Brothers' War",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(powerstone) }),
            (
                2,
                Effect::Seq(vec![
                    Effect::MustAttackPlayerFor {
                        attacker: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                        defender: player(1),
                        duration: Duration::UntilNextTurn,
                    },
                    Effect::MustAttackPlayerFor {
                        attacker: Selector::ControlledBy { who: PlayerRef::Target(1), filter: R::Creature },
                        defender: player(0),
                        duration: Duration::UntilNextTurn,
                    },
                ]),
            ),
            (
                3,
                Effect::Seq(vec![
                    Effect::DealDamage { to: target_any(), amount: artifacts() },
                    Effect::DealDamage {
                        to: Selector::TargetFiltered { slot: 1, filter: R::any_target().and(R::OtherThanTargetSlot(0)) },
                        amount: artifacts(),
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Overwhelming Encounter — {3}{G}{G} sorcery. Your creatures gain vigilance
/// and trample until end of turn; roll a d20: 1-9 +2/+2 until end of turn;
/// 10-19 two +1/+1 counters each; 20 four each.
pub fn overwhelming_encounter() -> CardDefinition {
    let counters = |n| Effect::AddCounter { what: yours(), kind: CounterType::PlusOnePlusOne, amount: Value::Const(n) };
    CardDefinition {
        name: "Overwhelming Encounter",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword { what: yours(), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: yours(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            Effect::RollDie {
                sides: 20,
                count: Value::ONE,
                modifier: Value::Const(0),
                reroll_at_most: 0,
                ignore_lowest: 0,
                results: vec![
                    (
                        1,
                        9,
                        Effect::PumpPT { what: yours(), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
                    ),
                    (10, 19, counters(2)),
                    (20, 20, counters(4)),
                ],
                on_doubles: None,
            },
        ]),
        ..Default::default()
    }
}

/// Basri, Devoted Paladin — {4}{W}{W} planeswalker, loyalty 4. +1: a +1/+1
/// counter on up to one target creature, which gains vigilance. −1: this turn,
/// each attacking creature gets a +1/+1 counter. −6: your creatures get +2/+2
/// and flying until end of turn.
pub fn basri_devoted_paladin() -> CardDefinition {
    CardDefinition {
        name: "Basri, Devoted Paladin",
        cost: cost(&[generic(4), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Basri], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::Seq(vec![
                        Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                        Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                    ])),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::OnMatchingAttacksThisTurn {
                    filter: R::Creature,
                    body: Box::new(Effect::AddCounter {
                        what: Selector::TriggerSource,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::Seq(vec![
                    Effect::PumpPT { what: yours(), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: yours(), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hold for Ransom — {1}{W} Aura. Enchanted creature can't attack or block
/// and has "{7}: Hold for Ransom's controller sacrifices it and draws a card.
/// Activate only as a sorcery."
pub fn hold_for_ransom() -> CardDefinition {
    let ransom = || Selector::MatchingAmong {
        inner: Box::new(Selector::AttachedToMe(Box::new(Selector::This))),
        filter: R::HasName("Hold for Ransom".into()),
    };
    CardDefinition {
        name: "Hold for Ransom",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has \"{7}: Hold for Ransom's controller sacrifices it and draws a card. Activate only as a sorcery.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    mana_cost: cost(&[generic(7)]),
                    sorcery_speed: true,
                    effect: Effect::Seq(vec![
                        Effect::Draw {
                            who: Selector::Player(PlayerRef::ControllerOf(Box::new(ransom()))),
                            amount: Value::ONE,
                        },
                        Effect::SacrificePermanent { what: ransom() },
                    ]),
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Renowned Weaponsmith — {1}{U} 1/3 Human Artificer. {T}: {C}{C} for artifact
/// spells and artifact abilities only. {U}, {T}: search for a card named
/// Heart-Piercer Bow or Vial of Dragonfire.
pub fn renowned_weaponsmith() -> CardDefinition {
    CardDefinition {
        name: "Renowned Weaponsmith",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Artificer], ..Default::default() },
        power: 1,
        toughness: 3,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(Box::new(ManaPayload::Colorless(Value::Const(2))), SpendRestriction::ArtifactOnly),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                tap_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasName("Heart-Piercer Bow".into()).or(R::HasName("Vial of Dragonfire".into())),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Herdchaser Dragon — {5}{G} 3/3 flying, trample Dragon, megamorph {5}{G}{G}.
/// Turned face up: a +1/+1 counter on each other Dragon creature you control.
pub fn herdchaser_dragon() -> CardDefinition {
    CardDefinition {
        name: "Herdchaser Dragon",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Megamorph(cost(&[generic(5), g(), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Dragon)).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Lurking Green Dragon — {3}{G} 4/4 flying Dragon; can't attack unless the
/// defending player controls a creature with flying.
pub fn lurking_green_dragon() -> CardDefinition {
    CardDefinition {
        name: "Lurking Green Dragon",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![
            Keyword::Flying,
            Keyword::CanAttackOnlyIfDefenderControls(Box::new(R::Creature.and(R::HasKeyword(Keyword::Flying)))),
        ],
        ..Default::default()
    }
}

/// Spare Supplies — {2} artifact, enters tapped. ETB: draw a card. {2}, {T},
/// sacrifice it: draw a card.
pub fn spare_supplies() -> CardDefinition {
    CardDefinition {
        name: "Spare Supplies",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}
