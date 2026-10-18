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

/// Twists and Turns // Mycoid Maze — {G} enchantment. If a creature you
/// control would explore, instead you scry 1, then that creature explores. ETB:
/// target creature you control explores. When a land you control enters, if
/// you control seven or more lands, transform it. Back: Mycoid Maze, a Cave
/// land — {T}: add {G}; {3}{G}, {T}: look at the top four, you may reveal a
/// creature card from among them into your hand, the rest on the bottom.
pub fn twists_and_turns() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType, Predicate, SelectionRequirement as R, StaticAbility, Value};
    use crate::effect::{LookPick, ManaPayload, PlayerRef, Selector, StaticEffect};
    let maze = CardDefinition {
        name: "Mycoid Maze",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Cave], ..Default::default() },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Green, Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), crate::mana::g()]),
                tap_cost: true,
                effect: Effect::LookPickToHand(Box::new(LookPick {
                    who: PlayerRef::You,
                    count: Value::Const(4),
                    pick_filter: Some(R::Creature),
                    optional: true,
                    ..Default::default()
                })),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let lands = Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByYou)));
    CardDefinition {
        name: "Twists and Turns",
        cost: cost(&[crate::mana::g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If a creature you control would explore, instead you scry 1, then that creature explores.",
            effect: StaticEffect::ScryBeforeExploring,
        }],
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::Explore {
                who: crate::effect::shortcut::target_filtered(R::Creature.and(R::ControlledByYou)),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
                    Predicate::ValueAtLeast(lands, Value::Const(7)),
                ])),
                effect: Effect::Transform { what: Selector::This },
            },
        ],
        back_face: Some(Box::new(maze)),
        ..Default::default()
    }
}

/// Kitesail Skirmisher — {3}{U} 3/1 flying Human Pirate. Whenever it attacks,
/// another target creature attacking the same player or planeswalker gains
/// flying until end of turn. Encore {4}{U}.
pub fn kitesail_skirmisher() -> CardDefinition {
    use crate::card::SelectionRequirement as R;
    use crate::effect::Duration;
    CardDefinition {
        name: "Kitesail Skirmisher",
        cost: cost(&[generic(3), crate::mana::u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Pirate], ..Default::default() },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: crate::effect::shortcut::target_filtered(
                    R::Creature.and(R::OtherThanSource).and(R::IsAttackingSameAsSource),
                ),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![crate::effect::shortcut::encore(cost(&[generic(4), crate::mana::u()]))],
        ..Default::default()
    }
}

/// Conqueror's Galleon // Conqueror's Foothold — {4} 2/10 Vehicle, crew 4.
/// When it attacks, exile it at end of combat, then return it to the
/// battlefield transformed under your control. Back: a land — {T}: {C};
/// {2},{T}: draw, then discard; {4},{T}: draw; {6},{T}: return target card
/// from your graveyard to your hand.
pub fn conquerors_galleon() -> CardDefinition {
    use crate::card::{ActivatedAbility, SelectionRequirement as R, Value};
    use crate::effect::{ManaPayload, PlayerRef, Selector, ZoneDest};
    let draw = || Effect::Draw { who: Selector::You, amount: Value::ONE };
    let tap = |mana: u32, effect: Effect| ActivatedAbility {
        mana_cost: cost(&[generic(mana)]),
        tap_cost: true,
        effect,
        ..Default::default()
    };
    let foothold = CardDefinition {
        name: "Conqueror's Foothold",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            tap(2, Effect::Seq(vec![draw(), Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }])),
            tap(4, draw()),
            tap(
                6,
                Effect::Move {
                    what: crate::effect::shortcut::target_filtered(R::InGraveyard.and(R::OwnedByYou)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
        ],
        ..Default::default()
    };
    CardDefinition {
        name: "Conqueror's Galleon",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle], ..Default::default() },
        power: 2,
        toughness: 10,
        keywords: vec![Keyword::Crew(4)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat { body: Box::new(Effect::ExileSelfReturnTransformed) },
        }],
        back_face: Some(Box::new(foothold)),
        ..Default::default()
    }
}
