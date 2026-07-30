//! Shadowmoor / Eventide — Conspire (CR 702.78). As you cast a Conspire
//! spell you may tap two untapped creatures you control sharing a color with
//! it to copy it once (the copy may choose new targets). Cast via
//! `GameAction::CastSpellConspire`.

use crate::card::{CardDefinition, CardType, Keyword, SelectionRequirement};
use crate::effect::shortcut::{deal, pump_target, target, target_filtered};
use crate::effect::{Effect, LibraryPosition, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, cost, g, generic, hybrid, r, u, w};

/// Burn Trail — {3}{R} Sorcery. "Burn Trail deals 3 damage to any target.
/// Conspire."
pub fn burn_trail() -> CardDefinition {
    CardDefinition {
        name: "Burn Trail",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: deal(3, target()),
        ..Default::default()
    }
}

/// Barkshell Blessing — {G/W} Instant. "Target creature gets +2/+2 until end
/// of turn. Conspire." (Modeled with the green half of the hybrid pip.)
pub fn barkshell_blessing() -> CardDefinition {
    CardDefinition {
        name: "Barkshell Blessing",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Conspire],
        effect: pump_target(2, 2),
        ..Default::default()
    }
}

/// Memory Sluice — {U/B} Sorcery. "Target player mills four cards. Conspire."
pub fn memory_sluice() -> CardDefinition {
    CardDefinition {
        name: "Memory Sluice",
        cost: cost(&[hybrid(Color::Blue, Color::Black)]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: Effect::Mill {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Gleeful Sabotage — {1}{G} Sorcery. "Destroy target artifact or enchantment.
/// Conspire."
pub fn gleeful_sabotage() -> CardDefinition {
    CardDefinition {
        name: "Gleeful Sabotage",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::HasCardType(CardType::Artifact)
                    .or(SelectionRequirement::HasCardType(CardType::Enchantment)),
            ),
        },
        ..Default::default()
    }
}

/// Ghastly Discovery — {2}{U} Sorcery. "Draw two cards, then discard a card.
/// Conspire."
pub fn ghastly_discovery() -> CardDefinition {
    CardDefinition {
        name: "Ghastly Discovery",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
            },
        ]),
        ..Default::default()
    }
}

/// Disturbing Plot — {1}{B} Sorcery. "Return target creature card from a
/// graveyard to its owner's hand. Conspire."
pub fn disturbing_plot() -> CardDefinition {
    CardDefinition {
        name: "Disturbing Plot",
        cost: cost(&[generic(1), crate::mana::b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        ..Default::default()
    }
}

/// Mine Excavation — {1}{W} Sorcery. "Return target artifact or enchantment
/// card from a graveyard to its owner's hand. Conspire."
pub fn mine_excavation() -> CardDefinition {
    CardDefinition {
        name: "Mine Excavation",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::HasCardType(CardType::Artifact)
                    .or(SelectionRequirement::HasCardType(CardType::Enchantment)),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        ..Default::default()
    }
}

/// Rally the Galadhrim — {2}{G}{U} Sorcery. "Create a token that's a copy of
/// target creature you control. Conspire."
pub fn rally_the_galadhrim() -> CardDefinition {
    CardDefinition {
        name: "Rally the Galadhrim",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: Effect::CreateTokenCopyOf {
            extra_keywords: vec![],
            who: PlayerRef::You,
            count: Value::Const(1),
            source: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            extra_creature_types: vec![],
            extra_card_types: vec![],
            override_pt: None,
            override_colors: None,
            enters_tapped: false,
            non_legendary: false,
            legendary: false,
        },
        ..Default::default()
    }
}

/// Aethertow — {3}{W/U} Instant. "Put target attacking or blocking creature on
/// top of its owner's library. Conspire."
pub fn aethertow() -> CardDefinition {
    CardDefinition {
        name: "Aethertow",
        cost: cost(&[generic(3), hybrid(Color::White, Color::Blue)]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Conspire],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::IsAttacking.or(SelectionRequirement::IsBlocking),
            ),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOfMoved,
                pos: LibraryPosition::Top,
            },
        },
        ..Default::default()
    }
}

/// Giantbaiting — {2}{R/G} Sorcery. "Create a 4/4 red and green Giant Warrior
/// creature token with haste. Exile it at the beginning of the next end step.
/// Conspire."
pub fn giantbaiting() -> CardDefinition {
    use crate::card::{CreatureType, Subtypes, TokenDefinition};
    use crate::effect::DelayedTriggerKind;
    let giant = TokenDefinition {
        name: "Giant Warrior".into(),
        power: 4,
        toughness: 4,
        colors: vec![Color::Red, Color::Green],
        keywords: vec![Keyword::Haste],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Giantbaiting",
        cost: cost(&[generic(2), hybrid(Color::Red, Color::Green)]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: giant,
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::Exile {
                    what: Selector::LastCreatedToken,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Traitor's Roar — {4}{B/R} Sorcery. "Tap target untapped creature. It deals
/// damage equal to its power to its controller. Conspire."
pub fn traitors_roar() -> CardDefinition {
    CardDefinition {
        name: "Traitor's Roar",
        cost: cost(&[generic(4), hybrid(Color::Black, Color::Red)]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Conspire],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::Untapped),
                ),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}
