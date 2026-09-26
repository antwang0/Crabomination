//! Top-1000 Commander staples missing from the catalog
//! (COMMANDER_BACKLOG §2, EDHREC rank 409–899). Each is built from
//! primitives other cards already exercise:
//!
//! - **Xorn / Peregrin Took** — the token-creation replacement statics
//!   Jolene and the Academy Manufactor family use (CR 614.1a).
//! - **Lotho** — Ledger Shredder's "second spell each turn" filter.
//! - **Boromir** — reads the cast spell's `mana_spent`, threaded onto the
//!   trigger (Freestrider Commando's gate).
//! - **Imp's Mischief** — Swerve's retarget plus a life loss.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn legendary_creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        ..Default::default()
    }
}

fn mine(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// Xorn — {2}{R} 3/2 Elemental. "If you would create one or more Treasure
/// tokens, instead create those tokens plus an additional Treasure token."
pub fn xorn() -> CardDefinition {
    CardDefinition {
        name: "Xorn",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 3,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "If you would create one or more Treasure tokens, instead create those tokens plus an additional Treasure token.",
            effect: StaticEffect::TreasureCreationAddsTreasure,
        }],
        ..Default::default()
    }
}

/// Peregrin Took — {2}{G} 2/3 Legendary Halfling Citizen. Token creations
/// under your control make an additional Food; sacrifice three Foods: draw
/// a card.
pub fn peregrin_took() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If one or more tokens would be created under your control, those tokens plus an additional Food token are created instead.",
            effect: StaticEffect::TokenCreationAddsToken { definition: crabomination_base::tokens::food_token() },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Food), 3)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..legendary_creature(
            "Peregrin Took",
            cost(&[generic(2), g()]),
            vec![CreatureType::Halfling, CreatureType::Citizen],
            2,
            3,
        )
    }
}

/// Flowering of the White Tree — {W}{W} Legendary Enchantment. Legendary
/// creatures you control get +2/+1 and have ward {1}; nonlegendary
/// creatures you control get +1/+1.
pub fn flowering_of_the_white_tree() -> CardDefinition {
    let legendary = || R::Creature.and(R::HasSupertype(Supertype::Legendary));
    let nonlegendary = R::Creature.and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary))));
    CardDefinition {
        name: "Flowering of the White Tree",
        cost: cost(&[w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Legendary creatures you control get +2/+1.",
                effect: StaticEffect::PumpPT { applies_to: mine(legendary()), power: 2, toughness: 1 },
            },
            StaticAbility {
                description: "Legendary creatures you control have ward {1}.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: mine(legendary()),
                    keyword: Keyword::Ward(WardCost::generic(1)),
                },
            },
            StaticAbility {
                description: "Nonlegendary creatures you control get +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: mine(nonlegendary), power: 1, toughness: 1 },
            },
        ],
        ..Default::default()
    }
}

/// Lotho, Corrupt Shirriff — {W}{B} 2/1 Legendary Halfling Rogue.
/// "Whenever a player casts their second spell each turn, you lose 1 life
/// and create a Treasure token."
pub fn lotho_corrupt_shirriff() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::SpellsCastThisTurnEquals { who: PlayerRef::Triggerer, count: Value::Const(2) },
            ),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                },
            ]),
        }],
        ..legendary_creature(
            "Lotho, Corrupt Shirriff",
            cost(&[w(), b()]),
            vec![CreatureType::Halfling, CreatureType::Rogue],
            2,
            1,
        )
    }
}

/// Borne Upon a Wind — {1}{U} Instant. You may cast spells this turn as
/// though they had flash. Draw a card.
pub fn borne_upon_a_wind() -> CardDefinition {
    CardDefinition {
        name: "Borne Upon a Wind",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantSpellsFlashThisTurn { who: PlayerRef::You },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Imp's Mischief — {1}{B} Instant. Change the target of target spell with
/// a single target (CR 115.7a); you lose life equal to that spell's mana
/// value.
pub fn imps_mischief() -> CardDefinition {
    CardDefinition {
        name: "Imp's Mischief",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ChangeSpellTarget { what: target_filtered(R::IsSpellOnStack) },
            Effect::LoseLife { who: Selector::You, amount: Value::ManaValueOf(Box::new(Selector::Target(0))) },
        ]),
        ..Default::default()
    }
}

/// Boromir, Warden of the Tower — {2}{W} 3/3 Legendary Human Soldier.
/// Vigilance. Whenever an opponent casts a spell, if no mana was spent to
/// cast it, counter that spell. Sacrifice Boromir: creatures you control
/// gain indestructible until end of turn; the Ring tempts you.
pub fn boromir_warden_of_the_tower() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            // CR 603.4 intervening "if": the cast spell's `mana_spent` rides
            // on the trigger.
            effect: Effect::If {
                cond: Predicate::Not(Box::new(Predicate::CastSpellManaSpentAtLeast(1))),
                then: Box::new(Effect::CounterSpell { what: Selector::TriggerSource }),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: mine(R::Creature),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::RingTempts { who: PlayerRef::You },
            ]),
            ..Default::default()
        }],
        ..legendary_creature(
            "Boromir, Warden of the Tower",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Jaheira, Friend of the Forest — {2}{G} 2/3 Legendary Human Elf Druid.
/// Tokens you control have "{T}: Add {G}." Choose a Background.
pub fn jaheira_friend_of_the_forest() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        static_abilities: vec![StaticAbility {
            description: "Tokens you control have \"{T}: Add {G}.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: mine(R::IsToken),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green]) },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..legendary_creature(
            "Jaheira, Friend of the Forest",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Elf, CreatureType::Druid],
            2,
            3,
        )
    }
}

/// Torment of Hailfire — {X}{B}{B} Sorcery. Repeat X times: each opponent
/// loses 3 life unless that player sacrifices a nonland permanent or
/// discards a card.
///
/// The victim's choice is heuristic (`Effect::Punisher` takes the first
/// affordable option): pay the 3 life while it leaves them above zero, then
/// discard, then sacrifice, and only then take the loss that kills them.
/// Paying life first is listed as an option because it *is* the default;
/// running it as the chooser keeps "that player" correct in a pod.
pub fn torment_of_hailfire() -> CardDefinition {
    let three = || Effect::LoseLife { who: Selector::You, amount: Value::Const(3) };
    CardDefinition {
        name: "Torment of Hailfire",
        cost: cost(&[crate::mana::x(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Repeat {
            count: Value::XFromCost,
            body: Box::new(Effect::Punisher {
                chooser: Selector::Player(PlayerRef::EachOpponent),
                options: vec![
                    three(),
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Permanent.and(R::Nonland) },
                ],
                otherwise: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(3),
                }),
            }),
        },
        ..Default::default()
    }
}

/// High Tide — {U} Instant. Until end of turn, whenever a player taps an
/// Island for mana, that player adds an additional {U} (CR 605.1b — a
/// triggered mana ability; Bubbling Muck's turn-scoped primitive).
pub fn high_tide() -> CardDefinition {
    CardDefinition {
        name: "High Tide",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ExtraManaOnLandTapThisTurn { land: crate::card::LandType::Island, extra: Color::Blue },
        ..Default::default()
    }
}

/// Beseech the Mirror — {1}{B}{B}{B} Sorcery. Bargain. Search your library
/// for a card, exile it face down, then shuffle. If this spell was
/// bargained, you may cast the exiled card without paying its mana cost if
/// its mana value is 4 or less (CR 702.176). Put it into your hand if it
/// wasn't cast this way.
pub fn beseech_the_mirror() -> CardDefinition {
    CardDefinition {
        name: "Beseech the Mirror",
        cost: cost(&[generic(1), b(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Bargain],
        effect: Effect::Seq(vec![
            Effect::Search { who: PlayerRef::You, filter: R::Any, to: crate::effect::ZoneDest::Exile },
            Effect::If {
                cond: Predicate::SpellWasBargained,
                then: Box::new(Effect::CastWithoutPayingImmediate {
                    what: Selector::ExiledThisResolution { filter: R::Nonland.and(R::ManaValueAtMost(4)) },
                    source_zone: crate::card::Zone::Exile,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Move {
                what: Selector::ExiledThisResolution { filter: R::InExile },
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}
