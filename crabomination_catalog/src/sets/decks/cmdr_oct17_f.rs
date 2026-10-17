//! Commander: the 2026-10-17 (`01Htv6cF`) researched-blocker cards — Bilbo,
//! Luckwearer (the cross-slot shared card type), Feldon, Ronom Excavator (an
//! exile-N-choose-one impulse), Warbringer (a dash discount). Tests in
//! `tests/recent_b/cmdr_oct17_b.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::dash;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{ManaCost, cost, generic, r, u};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

/// Bilbo, Luckwearer — {1}{U} legendary 1/1 Halfling Rogue, can't be blocked;
/// combat damage to a player: draw, then discard. Adventure — Burglar's Plot
/// {4}{U} sorcery: exchange control of two target nonland permanents that
/// share a card type.
pub fn bilbo_luckwearer() -> CardDefinition {
    let nonland = || R::Permanent.and(R::Nonland);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        }],
        adventure: Some(Box::new(Adventure {
            name: "Burglar's Plot",
            cost: cost(&[generic(4), u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::ExchangeControl {
                a: Selector::TargetFiltered { slot: 0, filter: nonland() },
                b: Selector::TargetFiltered {
                    slot: 1,
                    filter: nonland().and(R::OtherThanTargetSlot(0)).and(R::SharesCardTypeWithTargetSlot(0)),
                },
            },
        })),
        ..creature("Bilbo, Luckwearer", cost(&[generic(1), u()]), vec![CreatureType::Halfling, CreatureType::Rogue], 1, 1)
    }
}

/// Feldon, Ronom Excavator — {1}{R} legendary 2/2 haste, can't block.
/// Whenever it's dealt damage, exile that many cards from your top; choose
/// one — you may play it until the end of your next turn.
pub fn feldon_ronom_excavator() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Haste, Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::TriggerEventAmount },
                    to: ZoneDest::Exile,
                },
                Effect::ChooseOneAmong {
                    what: Selector::ExiledThisResolution { filter: R::Any },
                    chooser: PlayerRef::You,
                    chosen: Box::new(Effect::GrantMayPlay {
                        what: Selector::SeparatedPile { chosen: true },
                        duration: MayPlayDuration::EndOfControllersNextTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    }),
                    other: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..creature("Feldon, Ronom Excavator", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 2)
    }
}

/// Warbringer — {3}{R} 3/3 Orc Berserker. Dash costs you pay cost {2} less
/// while it's on the battlefield. Dash {2}{R}.
pub fn warbringer() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(dash(cost(&[generic(2), r()]))),
        static_abilities: vec![StaticAbility {
            description: "Dash costs you pay cost {2} less.",
            effect: StaticEffect::ControllerDashCostsLess { amount: 2 },
        }],
        ..creature("Warbringer", cost(&[generic(3), r()]), vec![CreatureType::Orc, CreatureType::Berserker], 3, 3)
    }
}

/// Bilbo Baggins, Burglar — {2}{U} legendary 2/1 Halfling Rogue. ETB: draw a
/// card. Adventure — Take a Glance {U} sorcery: scry 2.
pub fn bilbo_baggins_burglar() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![crate::effect::shortcut::etb_draw(1)],
        adventure: Some(Box::new(Adventure {
            name: "Take a Glance",
            cost: cost(&[u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        })),
        ..creature("Bilbo Baggins, Burglar", cost(&[generic(2), u()]), vec![CreatureType::Halfling, CreatureType::Rogue], 2, 1)
    }
}

/// Riddles in the Dark — {2}{U} Instant. Look at the top four, split them into
/// a face-down and a face-up pile; an opponent picks one for your hand, the
/// other goes to your graveyard (the Fact or Fiction split).
pub fn riddles_in_the_dark() -> CardDefinition {
    CardDefinition {
        name: "Riddles in the Dark",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::FactOrFiction { count: Value::Const(4), to_bottom: false },
        ..Default::default()
    }
}

/// Bilbo's Burglaring — {4}{U}{U} Sorcery. For each opponent, gain control of
/// up to one target artifact that player controls.
pub fn bilbos_burglaring() -> CardDefinition {
    CardDefinition {
        name: "Bilbo's Burglaring",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 15,
                min_targets: 0,
                filter: R::Artifact.and(R::ControlledByOpponent),
                effect: Box::new(Effect::GainControl { what: Selector::Target(0), to: None, duration: crate::effect::Duration::Permanent }),
            }),
        },
        ..Default::default()
    }
}
