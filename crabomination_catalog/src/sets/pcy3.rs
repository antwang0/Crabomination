//! Prophecy (PCY), third wave — the Rhystic cycle and its neighbours. Tests
//! in `classic_sets/pcy3`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, TriggeredAbility, Value, WardCost,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector,
    shortcut::{target_any, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

/// "Unless any player pays {n}, [then]" — the Rhystic tax shape.
fn unless_anyone_pays(n: u32, then: Effect) -> Effect {
    Effect::UnlessPlayerPays {
        who: PlayerRef::EachOpponent,
        cost: WardCost::Mana(cost(&[generic(n)])),
        then: Box::new(then),
        if_paid: None,
    }
}

/// Rethink — {2}{U}. A hard counter at their own price.
pub fn rethink() -> CardDefinition {
    instant(
        "Rethink",
        cost(&[generic(2), u()]),
        Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack),
            mana_cost: ManaCost::default(),
            exile: false,
            extra_generic: Some(Value::ManaValueOf(Box::new(Selector::Target(0)))),
        },
    )
}

/// Spiketail Drake — {3}{U}{U} 3/3 flier that trades itself for a counter.
pub fn spiketail_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(3)]),
                exile: false,
                extra_generic: None,
            },
            ..Default::default()
        }],
        ..creature("Spiketail Drake", cost(&[generic(3), u(), u()]), vec![CreatureType::Drake], 3, 3)
    }
}

/// Rhystic Cave — a land that fixes any colour unless someone objects.
pub fn rhystic_cave() -> CardDefinition {
    CardDefinition {
        name: "Rhystic Cave",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: unless_anyone_pays(
                1,
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
            ),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rhystic Deluge — {2}{U}. A tapper nobody wants to pay off.
pub fn rhystic_deluge() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                cost: WardCost::Mana(cost(&[generic(1)])),
                then: Box::new(Effect::Tap { what: target_filtered(R::Creature) }),
                if_paid: None,
            },
            ..Default::default()
        }],
        ..enchantment("Rhystic Deluge", cost(&[generic(2), u()]))
    }
}

/// Rhystic Lightning — {2}{R}. Four damage, or two if they pay.
pub fn rhystic_lightning() -> CardDefinition {
    instant(
        "Rhystic Lightning",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            unless_anyone_pays(
                2,
                Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
            ),
        ]),
    )
}

/// Rhystic Scrying — {2}{U}{U}. Three cards, unless they buy the discard.
pub fn rhystic_scrying() -> CardDefinition {
    sorcery(
        "Rhystic Scrying",
        cost(&[generic(2), u(), u()]),
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::UnlessPlayerPays {
                who: PlayerRef::EachOpponent,
                cost: WardCost::Mana(cost(&[generic(2)])),
                then: Box::new(Effect::Noop),
                if_paid: Some(Box::new(Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(3),
                    random: false,
                })),
            },
        ]),
    )
}

/// Rhystic Shield — {1}{W}. A combat trick that scales with their mana.
pub fn rhystic_shield() -> CardDefinition {
    instant(
        "Rhystic Shield",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ZERO,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            unless_anyone_pays(
                2,
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: Value::ZERO,
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
            ),
        ]),
    )
}

/// Rhystic Syphon — {3}{B}{B}. Five points of drain at a {3} toll.
pub fn rhystic_syphon() -> CardDefinition {
    sorcery(
        "Rhystic Syphon",
        cost(&[generic(3), b(), b()]),
        Effect::UnlessPlayerPays {
            who: PlayerRef::Target(0),
            cost: WardCost::Mana(cost(&[generic(3)])),
            then: Box::new(Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(5),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
            ])),
            if_paid: None,
        },
    )
}

/// Rhystic Tutor — {2}{B}. Demonic Tutor if nobody has two mana up.
pub fn rhystic_tutor() -> CardDefinition {
    sorcery(
        "Rhystic Tutor",
        cost(&[generic(2), b()]),
        unless_anyone_pays(
            2,
            Effect::Search {
                who: PlayerRef::You,
                filter: R::Any,
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            },
        ),
    )
}

/// Soul Strings — {X}{B}. Two creatures back unless they pay X.
pub fn soul_strings() -> CardDefinition {
    CardDefinition {
        name: "Soul Strings",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::UnlessPlayerPays {
            who: PlayerRef::EachOpponent,
            cost: WardCost::GenericXFromCost,
            then: Box::new(Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::Creature,
                    },
                    Value::Const(2),
                ),
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            }),
            if_paid: None,
        },
        ..Default::default()
    }
}

/// Rhystic Circle — {2}{W}{W}. A Circle of Protection anyone can switch off.
pub fn rhystic_circle() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: unless_anyone_pays(
                1,
                Effect::PreventNextDamageFromChosenSource {
                    filter: R::Any,
                    reflect: false,
                    to: None,
                    gain_life: false,
                    redirect_to: None,
                    whole_turn: false,
                },
            ),
            ..Default::default()
        }],
        ..enchantment("Rhystic Circle", cost(&[generic(2), w(), w()]))
    }
}

/// Samite Sanctuary — {2}{W}. A prevention shield anyone can buy.
pub fn samite_sanctuary() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            any_player: true,
            effect: Effect::PreventNextDamage {
                target: target_filtered(R::Creature),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Samite Sanctuary", cost(&[generic(2), w()]))
    }
}

/// Soul Charmer — {2}{W} 2/2. Its bite pays you unless they pay first.
pub fn soul_charmer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                cost: WardCost::Mana(cost(&[generic(2)])),
                then: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                }),
                if_paid: None,
            },
        }],
        ..creature(
            "Soul Charmer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            2,
            2,
        )
    }
}

/// Shrouded Serpent — {4}{U}{U}{U} 4/4. Unblockable at a {4} toll.
pub fn shrouded_serpent() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::DefendingPlayer,
                cost: WardCost::Mana(cost(&[generic(4)])),
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                }),
                if_paid: None,
            },
        }],
        ..creature(
            "Shrouded Serpent",
            cost(&[generic(4), u(), u(), u()]),
            vec![CreatureType::Serpent],
            4,
            4,
        )
    }
}

/// Ribbon Snake — {1}{U}{U} 2/3 flier anyone can ground.
pub fn ribbon_snake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            any_player: true,
            effect: Effect::LoseKeyword { duration: Duration::EndOfTurn,
                what: Selector::This,
                keyword: Keyword::Flying,
            },
            ..Default::default()
        }],
        ..creature("Ribbon Snake", cost(&[generic(1), u(), u()]), vec![CreatureType::Snake], 2, 3)
    }
}

/// Rib Cage Spider — {2}{G} 1/4 reach.
pub fn rib_cage_spider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        ..creature("Rib Cage Spider", cost(&[generic(2), g()]), vec![CreatureType::Spider], 1, 4)
    }
}

/// Spitting Spider — {3}{G}{G} 3/5 reach.
pub fn spitting_spider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        ..creature("Spitting Spider", cost(&[generic(3), g(), g()]), vec![CreatureType::Spider], 3, 5)
    }
}

/// Ridgeline Rager — {2}{R} 1/2 firebreather.
pub fn ridgeline_rager() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Ridgeline Rager", cost(&[generic(2), r()]), vec![CreatureType::Beast], 1, 2)
    }
}

/// Root Cage — {1}{G}. Mercenaries stay tapped.
pub fn root_cage() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Mercenaries don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Mercenary)),
            },
        }],
        ..enchantment("Root Cage", cost(&[generic(1), g()]))
    }
}

/// Scoria Cat — {3}{R}{R} 3/3. A 6/6 once you're tapped out.
pub fn scoria_cat() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +3/+3 as long as you control no untapped lands.",
            effect: StaticEffect::PumpSelfIf {
                condition: crate::card::Predicate::Not(Box::new(
                    crate::card::Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(
                            R::Land.and(R::Untapped).and(R::ControlledByYou),
                        ),
                        n: Value::ONE,
                    },
                )),
                power: 3,
                toughness: 3,
                keywords: vec![],
            },
        }],
        ..creature("Scoria Cat", cost(&[generic(3), r(), r()]), vec![CreatureType::Cat], 3, 3)
    }
}

/// Silt Crawler — {2}{G} 3/3. A big body that costs you the turn's mana.
pub fn silt_crawler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
            },
        }],
        ..creature("Silt Crawler", cost(&[generic(2), g()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Snag — {3}{G}. A one-sided fog you can pitch a Forest for.
pub fn snag() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            discard_filters: vec![(R::HasLandType(LandType::Forest), 1)],
            ..Default::default()
        }),
        ..instant(
            "Snag",
            cost(&[generic(3), g()]),
            Effect::PreventAllCombatDamageByMatchingThisTurn { filter: R::IsUnblocked },
        )
    }
}

/// Mercenary Informer — {2}{W} 2/1. Bottoms Mercenaries, ducks black removal.
pub fn mercenary_informer() -> CardDefinition {
    informer(
        "Mercenary Informer",
        cost(&[generic(2), w()]),
        vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Mercenary],
        2,
        1,
        Color::Black,
        cost(&[generic(2), w()]),
        CreatureType::Mercenary,
    )
}

/// Rebel Informer — {2}{B} 1/2. The mirror half of the cycle.
pub fn rebel_informer() -> CardDefinition {
    informer(
        "Rebel Informer",
        cost(&[generic(2), b()]),
        vec![CreatureType::Human, CreatureType::Mercenary, CreatureType::Rebel],
        1,
        2,
        Color::White,
        cost(&[generic(3)]),
        CreatureType::Rebel,
    )
}

#[allow(clippy::too_many_arguments)]
fn informer(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    safe_from: Color,
    ability_cost: ManaCost,
    bottoms: CreatureType,
) -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::HexproofFromColor(safe_from)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ability_cost,
            effect: Effect::Move {
                what: target_filtered(
                    R::HasCreatureType(bottoms).and(R::Not(Box::new(R::IsToken))),
                ),
                to: crate::effect::ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: crate::effect::LibraryPosition::Bottom,
                },
            },
            ..Default::default()
        }],
        ..creature(name, c, types, p, t)
    }
}

/// Reveille Squad — {2}{W}{W} 3/3. Untaps your whole board on their attack.
pub fn reveille_squad() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::Attacks,
                EventScope::ControllerAttackedByOpponent,
            )
            .with_filter(crate::card::Predicate::EntityMatches {
                what: Selector::This,
                filter: R::Untapped,
            }),
            effect: Effect::MayDo {
                description: "Untap all creatures you control?".to_string(),
                body: Box::new(Effect::Untap {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    up_to: None,
                }),
            },
        }],
        ..creature(
            "Reveille Squad",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            3,
            3,
        )
    }
}
