//! Commander: the cards the **Sneak Attack** precon (ZNC, Anowon, the Ruin
//! Thief) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    Zone,
};
use crate::effect::shortcut::{etb, on_attack, prowl, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, b, cost, generic, u, x};
use std::sync::Arc;

fn rogue(
    name: &'static str,
    mana: crate::mana::ManaCost,
    mut types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    types.push(CreatureType::Rogue);
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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// "An opponent has eight or more cards in their graveyard."
fn opponent_graveyard_eight() -> Predicate {
    Predicate::ValueAtLeast(
        Value::GreatestGraveyardSizeAmong(PlayerRef::EachOpponent),
        Value::Const(8),
    )
}

/// "Whenever this creature deals combat damage to a player, …"
fn on_combat_damage(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect,
    }
}

fn draw_one() -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::ONE }
}

/// Anowon, the Ruin Thief — a Rogue lord whose Rogues mill the players they
/// hit card for card, drawing when a creature is milled.
pub fn anowon_the_ruin_thief() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Other Rogues you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: yours(
                    R::Creature.and(R::HasCreatureType(CreatureType::Rogue)).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        // CR 603.2c — one fire per damaged player, milling the batch's total.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Rogue),
                })
                .once_per_batch_summing_damage(),
            effect: Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::TriggerEventAmount,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::CreatureCardsMilledThisEffect, Value::ONE),
                    then: Box::new(draw_one()),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..rogue(
            "Anowon, the Ruin Thief",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Vampire],
            2,
            4,
        )
    }
}

/// Enigma Thief — prowl; on entry, bounce up to one nonland permanent from
/// each opponent.
pub fn enigma_thief() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(prowl(
            cost(&[generic(3), u()]),
            vec![CreatureType::Sphinx, CreatureType::Rogue],
        )),
        triggered_abilities: vec![etb(Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Permanent.and(R::Nonland).and(R::ControlledByOpponent),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            }),
        })],
        ..rogue("Enigma Thief", cost(&[generic(5), u(), u()]), vec![CreatureType::Sphinx], 5, 5)
    }
}

/// Marang River Prowler — can't block, can't be blocked, and castable from
/// the graveyard while you control a black or green permanent.
pub fn marang_river_prowler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock, Keyword::Unblockable, Keyword::GraveyardCast],
        flashback_condition: Some(Predicate::SelectorExists(yours(
            R::Permanent.and(R::HasColor(Color::Black).or(R::HasColor(Color::Green))),
        ))),
        ..rogue("Marang River Prowler", cost(&[generic(2), u()]), vec![CreatureType::Human], 2, 1)
    }
}

/// Master Thief — steal an artifact for as long as it stays. ⚠ The steal
/// ends when Master Thief leaves the battlefield, not when you lose control
/// of it.
pub fn master_thief() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: target_filtered(R::Artifact),
        })],
        ..rogue("Master Thief", cost(&[generic(2), u(), u()]), vec![CreatureType::Human], 2, 2)
    }
}

/// Merfolk Windrobber — mills a card per hit, and cashes in for a card once
/// an opponent's graveyard reaches eight.
pub fn merfolk_windrobber() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_combat_damage(Effect::Mill {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            condition: Some(opponent_graveyard_eight()),
            effect: draw_one(),
            ..Default::default()
        }],
        ..rogue("Merfolk Windrobber", cost(&[u()]), vec![CreatureType::Merfolk], 1, 1)
    }
}

/// Nightveil Sprite — a flier that surveils as it attacks.
pub fn nightveil_sprite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE })],
        ..rogue("Nightveil Sprite", cost(&[generic(1), u()]), vec![CreatureType::Faerie], 1, 2)
    }
}

/// Open into Wonder — X target creatures are unblockable and draw on a hit
/// this turn.
pub fn open_into_wonder() -> CardDefinition {
    let draw_on_hit = Box::new(on_combat_damage(draw_one()));
    CardDefinition {
        name: "Open into Wonder",
        cost: cost(&[x(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Unblockable,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantTriggeredAbility {
                        what: Selector::Target(0),
                        trigger: draw_on_hit,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            }),
        },
        ..Default::default()
    }
}

/// Price of Fame — destroy a creature and surveil 2; {2} cheaper aimed at a
/// legend.
pub fn price_of_fame() -> CardDefinition {
    CardDefinition {
        name: "Price of Fame",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((R::HasSupertype(Supertype::Legendary), 2)),
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Rise from the Grave — reanimate from any graveyard, as a black Zombie in
/// addition to its other colors and types.
pub fn rise_from_the_grave() -> CardDefinition {
    CardDefinition {
        name: "Rise from the Grave",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCreatureTypes {
                what: Selector::LastMoved,
                creature_types: vec![CreatureType::Zombie],
                duration: Duration::Permanent,
            },
            Effect::BecomeColor {
                what: Selector::LastMoved,
                colors: vec![Color::Black],
                duration: Duration::Permanent,
                additive: true,
            },
        ]),
        ..Default::default()
    }
}

/// Scytheclaw — living weapon, +1/+1, and a hit halves the player's life
/// (rounded up).
pub fn scytheclaw() -> CardDefinition {
    let germ = Arc::new(TokenDefinition {
        name: "Phyrexian Germ".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        name: "Scytheclaw",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![on_combat_damage(Effect::LoseHalfLife {
                who: Selector::Player(PlayerRef::Target(0)),
                rounded_up: true,
            })],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: germ },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        ..Default::default()
    }
}

/// Soaring Thought-Thief — flash flier; Rogues get +1/+0 once an opponent's
/// graveyard reaches eight, and each Rogue attack mills every opponent two.
pub fn soaring_thought_thief() -> CardDefinition {
    let rogues = || R::Creature.and(R::HasCreatureType(CreatureType::Rogue));
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "As long as an opponent has eight or more cards in their graveyard, Rogues you control get +1/+0.",
            effect: StaticEffect::PumpTeamIf {
                condition: opponent_graveyard_eight(),
                applies_to: yours(rogues()),
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: rogues() })
                .once_per_batch(),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        }],
        ..rogue(
            "Soaring Thought-Thief",
            cost(&[u(), b()]),
            vec![CreatureType::Human],
            1,
            3,
        )
    }
}

/// Soul Manipulation — counter a creature spell, rebuy a creature card, or
/// both.
pub fn soul_manipulation() -> CardDefinition {
    CardDefinition {
        name: "Soul Manipulation",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::CounterSpell {
                    what: target_filtered(R::IsSpellOnStack.and(R::Creature)),
                },
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Sure-Footed Infiltrator — tap another Rogue to slip through; draws on a
/// hit.
pub fn sure_footed_infiltrator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature.and(R::HasCreatureType(CreatureType::Rogue))),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![on_combat_damage(draw_one())],
        ..rogue(
            "Sure-Footed Infiltrator",
            cost(&[generic(3), u()]),
            vec![CreatureType::Merfolk],
            2,
            3,
        )
    }
}

/// Whispersteel Dagger — +2/+0, and a hit opens the damaged player's
/// graveyard for a creature spell this turn with any-color mana. ⚠ Every
/// creature card there is castable, not only one.
pub fn whispersteel_dagger() -> CardDefinition {
    CardDefinition {
        name: "Whispersteel Dagger",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            triggered_abilities: vec![on_combat_damage(Effect::GrantMayPlay {
                what: Selector::CardsInZone {
                    who: PlayerRef::Target(0),
                    zone: Zone::Graveyard,
                    filter: R::Creature,
                },
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: false,
                any_color: true,
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}
