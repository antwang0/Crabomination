//! Modern-staple instants (interaction).

use super::no_abilities;
use crate::card::{CardDefinition, CardType, Effect, Keyword, SelectionRequirement, Subtypes};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, u, w};

/// Path to Exile — {W} Instant. Exile target creature; its controller may
/// search their library for a basic land card, put it onto the battlefield
/// tapped, then shuffle.
///
/// Approximation: the basic-land tutor half is omitted (the engine has no
/// "search and put onto battlefield tapped" effect that's also player-
/// directed at the *target's controller*). Single-step exile is wired.
pub fn path_to_exile() -> CardDefinition {
    CardDefinition {
        name: "Path to Exile",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(SelectionRequirement::Creature),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Fatal Push — {B} Instant. Destroy target creature with mana value 2 or
/// less. (Revolt clause — destroying a creature with mana value 4 or less
/// if a permanent left the battlefield this turn — is omitted; the base
/// half is what matters for the bulk of plays.)
pub fn fatal_push() -> CardDefinition {
    CardDefinition {
        name: "Fatal Push",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(2)),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Spell Pierce — {U} Instant. Counter target noncreature spell unless its
/// controller pays {2}.
pub fn spell_pierce() -> CardDefinition {
    CardDefinition {
        name: "Spell Pierce",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::Noncreature),
            ),
            mana_cost: cost(&[generic(2)]),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Mana Leak — {1}{U} Instant. Counter target spell unless its controller
/// pays {3}.
pub fn mana_leak() -> CardDefinition {
    CardDefinition {
        name: "Mana Leak",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(3)]),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Doom Blade — {1}{B} Instant. Destroy target nonblack creature.
pub fn doom_blade() -> CardDefinition {
    CardDefinition {
        name: "Doom Blade",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasColor(Color::Black).negate()),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Vapor Snag — {U} Instant. Return target creature to its owner's hand.
/// That creature's controller loses 1 life.
///
/// Order: life loss first, then bounce — by the time we resolve life loss,
/// the targeted creature is still on the battlefield so
/// `ControllerOf(Target(0))` finds it. (Owner is stable across zone changes
/// while controller is not.)
pub fn vapor_snag() -> CardDefinition {
    CardDefinition {
        name: "Vapor Snag",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(1),
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Blossoming Defense — {G} Instant. Target creature you control gets +2/+2
/// and gains hexproof until end of turn.
pub fn blossoming_defense() -> CardDefinition {
    CardDefinition {
        name: "Blossoming Defense",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
