//! Commander: 2026-10-25 (`01Y6ZRFx`) — the Major Teroh deck's closers, the
//! anti-black / anti-red enchantments. Tests in `tests/recent_b/cmdr_01y6_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, CumulativeUpkeepCost, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{declare_target_opponent, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, generic, w};

fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl), effect }
}

fn swamp() -> R {
    R::HasLandType(LandType::Swamp)
}

/// Kormus Bell — {4} artifact. All Swamps are 1/1 black creatures that are
/// still lands.
pub fn kormus_bell() -> CardDefinition {
    CardDefinition {
        name: "Kormus Bell",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "All Swamps are 1/1 black creatures that are still lands.",
            effect: StaticEffect::MatchingLandsAreCreatures {
                filter: swamp(),
                power: 1,
                toughness: 1,
                keywords: vec![],
                creature_types: vec![],
                colors: vec![Color::Black],
            },
        }],
        ..Default::default()
    }
}

/// Order of the White Shield — {W}{W} 2/1 Human Knight, protection from
/// black. {W}: first strike until end of turn. {W}{W}: +1/+0 until end of turn.
pub fn order_of_the_white_shield() -> CardDefinition {
    CardDefinition {
        name: "Order of the White Shield",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Knight], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Protection(Color::Black)],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w(), w()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Drought — {2}{W}{W} enchantment. Upkeep: sacrifice it unless you pay
/// {W}{W}. Spells and activated abilities cost an additional "Sacrifice a
/// Swamp" per black mana symbol in their costs.
pub fn drought() -> CardDefinition {
    CardDefinition {
        name: "Drought",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![your_upkeep(Effect::SacrificeSourceUnlessPay { cost: cost(&[w(), w()]) })],
        static_abilities: vec![StaticAbility {
            description: "Spells and activated abilities cost an additional \"Sacrifice a Swamp\" for each black mana symbol in their costs.",
            effect: StaticEffect::SacrificeSwampPerBlackSymbolToCast,
        }],
        ..Default::default()
    }
}

/// Karma — {2}{W}{W} enchantment. At the beginning of each player's upkeep,
/// it deals that player damage equal to the number of Swamps they control.
pub fn karma() -> CardDefinition {
    CardDefinition {
        name: "Karma",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::PermanentCountControlledByMatching(PlayerRef::ActivePlayer, swamp()),
            },
        }],
        ..Default::default()
    }
}

/// Mangara's Equity — {1}{W}{W} enchantment. As it enters, choose black or
/// red. Upkeep: sacrifice it unless you pay {1}{W}. Whenever a creature of
/// the chosen color deals damage to you or a white creature you control, it
/// deals that much damage to that creature.
pub fn mangaras_equity() -> CardDefinition {
    let dealer = || R::Creature.and(R::HasChosenColorOfSource);
    let hit_back = || Effect::DealDamage { to: Selector::TriggerSource, amount: Value::TriggerEventAmount };
    CardDefinition {
        name: "Mangara's Equity",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(Effect::ChooseColorForSelfAmong(vec![Color::Black, Color::Red])),
        triggered_abilities: vec![
            your_upkeep(Effect::SacrificeSourceUnlessPay { cost: cost(&[generic(1), w()]) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PlayerDamaged, EventScope::SourceDamagedYou).dealt_by(dealer()),
                effect: hit_back(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::AnyPlayer)
                    .dealt_by(dealer())
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::Creature.and(R::HasColor(Color::White)).and(R::ControlledByYou),
                    }),
                effect: hit_back(),
            },
        ],
        ..Default::default()
    }
}

/// Royal Decree — {2}{W}{W} enchantment, cumulative upkeep {W}. Whenever a
/// Swamp, Mountain, black permanent, or red permanent becomes tapped, it
/// deals 1 damage to that permanent's controller.
pub fn royal_decree() -> CardDefinition {
    let watched = swamp()
        .or(R::HasLandType(LandType::Mountain))
        .or(R::HasColor(Color::Black))
        .or(R::HasColor(Color::Red));
    CardDefinition {
        name: "Royal Decree",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[w()])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: watched }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Inquisitor's Snare — {1}{W} instant. Prevent all damage target attacking
/// or blocking creature would deal this turn; destroy it if it's black or red.
pub fn inquisitors_snare() -> CardDefinition {
    CardDefinition {
        name: "Inquisitor's Snare",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PreventAllDamageByTargetThisTurn {
                target: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasColor(Color::Black).or(R::HasColor(Color::Red)),
                },
                then: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Starlight — {1}{W} sorcery. You gain 3 life for each black creature
/// target opponent controls.
pub fn starlight() -> CardDefinition {
    CardDefinition {
        name: "Starlight",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            declare_target_opponent(),
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Times(
                    Box::new(Value::Const(3)),
                    Box::new(Value::PermanentCountControlledByMatching(
                        PlayerRef::Target(0),
                        R::Creature.and(R::HasColor(Color::Black)),
                    )),
                ),
            },
        ]),
        ..Default::default()
    }
}
