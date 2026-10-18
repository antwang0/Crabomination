//! Commander: 2026-10-18 (`019Y6J4R`) closers, nineteenth batch. Tests in
//! `tests/recent_b/cmdr_019y_s.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Subtypes, Supertype,
    TriggeredAbility,
};
use crate::effect::Effect;
use crate::mana::{Color, cost, generic, hybrid};

/// Don & Raph, Hard Science — {1}{U/R}{U/R} legendary 2/4 menace Mutant Ninja
/// Turtle. Whenever they attack, the next noncreature spell you cast this turn
/// has affinity for artifacts (CR 702.41a).
pub fn don_raph_hard_science() -> CardDefinition {
    let ur = || hybrid(Color::Blue, Color::Red);
    CardDefinition {
        name: "Don & Raph, Hard Science",
        cost: cost(&[generic(1), ur(), ur()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::NextNoncreatureSpellHasAffinityForArtifacts,
        }],
        ..Default::default()
    }
}

/// Mosswood Dreadknight // Dread Whispers — {1}{G} 3/2 trample Human Knight.
/// When it dies, you may cast it from your graveyard as an Adventure until the
/// end of your next turn. Dread Whispers ({1}{B} sorcery — Adventure): you
/// draw a card and you lose 1 life.
pub fn mosswood_dreadknight() -> CardDefinition {
    CardDefinition {
        name: "Mosswood Dreadknight",
        cost: cost(&[generic(1), crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Knight], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::GrantAdventureFromGraveyard)],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Dread Whispers",
            cost: cost(&[generic(1), crate::mana::b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::Draw { who: crate::effect::Selector::You, amount: crate::card::Value::ONE },
                Effect::LoseLife { who: crate::effect::Selector::You, amount: crate::card::Value::ONE },
            ]),
        })),
        ..Default::default()
    }
}

/// Jace's Machinations — {2}{U} instant. Until end of turn, you may activate
/// loyalty abilities of Jace planeswalkers you control at instant speed.
/// Empower Jace 8.
pub fn jaces_machinations() -> CardDefinition {
    CardDefinition {
        name: "Jace's Machinations",
        cost: cost(&[generic(2), crate::mana::u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::JaceLoyaltyAtInstantSpeedThisTurn,
            Effect::EmpowerJace { who: crate::effect::PlayerRef::You, count: crate::card::Value::Const(8) },
        ]),
        ..Default::default()
    }
}

/// Overwrite the Multiverse — {4}{B}{B} sorcery. Exile all creatures. Empower
/// Jace X, where X is the number of creatures exiled this way.
pub fn overwrite_the_multiverse() -> CardDefinition {
    use crate::card::{SelectionRequirement as R, Value};
    use crate::effect::{PlayerRef, Selector};
    CardDefinition {
        name: "Overwrite the Multiverse",
        cost: cost(&[generic(4), crate::mana::b(), crate::mana::b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Exile { what: Selector::EachPermanent(R::Creature) },
            Effect::EmpowerJace {
                who: PlayerRef::You,
                count: Value::count(Selector::ExiledThisResolution { filter: R::Any }),
            },
        ]),
        ..Default::default()
    }
}

/// Way of the Warlord — {2}{R} legendary enchantment. ETB: empower Jace 5.
/// Planeswalkers you control have "[−4]: This planeswalker deals 2 damage to
/// up to one target creature or planeswalker and 2 damage to target player."
/// (The player slot is written first: the positional target list can't leave
/// an earlier optional slot empty.)
pub fn way_of_the_warlord() -> CardDefinition {
    use crate::card::{LoyaltyAbility, SelectionRequirement as R, StaticAbility, Value};
    use crate::effect::{PlayerRef, Selector, StaticEffect};
    CardDefinition {
        name: "Way of the Warlord",
        cost: cost(&[generic(2), crate::mana::r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::EmpowerJace {
            who: PlayerRef::You,
            count: Value::Const(5),
        })],
        static_abilities: vec![StaticAbility {
            description: "Planeswalkers you control have \"[−4]: This planeswalker deals 2 damage to up to one target creature or planeswalker and 2 damage to target player.\"",
            effect: StaticEffect::PlaneswalkersHaveLoyaltyAbilities {
                abilities: vec![LoyaltyAbility {
                    loyalty_cost: -4,
                    effect: Effect::OptionalTargets {
                        min: 1,
                        body: Box::new(Effect::Seq(vec![
                            Effect::DealDamage {
                                to: Selector::TargetFiltered { slot: 0, filter: R::Player },
                                amount: Value::Const(2),
                            },
                            Effect::DealDamage {
                                to: Selector::TargetFiltered { slot: 1, filter: R::Creature.or(R::Planeswalker) },
                                amount: Value::Const(2),
                            },
                        ])),
                    },
                    ..Default::default()
                }],
            },
        }],
        ..Default::default()
    }
}
