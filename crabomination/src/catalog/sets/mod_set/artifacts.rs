//! Modern-staple / cube artifacts.

use super::no_abilities;
use crate::card::{ActivatedAbility, CardDefinition, CardType, Effect, Keyword, Subtypes};
use crate::effect::{ManaPayload, PlayerRef, Value};
use crate::mana::{ManaCost, cost, generic};

/// Ornithopter — {0} Artifact Creature 0/2 with Flying. Pure vanilla; no
/// abilities beyond Flying.
pub fn ornithopter() -> CardDefinition {
    CardDefinition {
        name: "Ornithopter",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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

/// Ornithopter of Paradise — {1} Artifact Creature 0/2 with Flying. {T}: Add
/// one mana of any color. Reuses `ManaPayload::AnyOneColor` so the engine
/// surfaces the color choice via the `ChooseColor` decision.
pub fn ornithopter_of_paradise() -> CardDefinition {
    CardDefinition {
        name: "Ornithopter of Paradise",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Millstone — {2} Artifact. {2}, {T}: Target player puts the top two cards
/// of their library into their graveyard.
pub fn millstone() -> CardDefinition {
    use crate::card::{Selector, Value};
    use crate::effect::PlayerRef;
    CardDefinition {
        name: "Millstone",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Mind Stone — {2} Artifact. {T}: Add {C}. {1}, {T}, Sacrifice this:
/// Draw a card.
///
/// Both abilities are wired: the first is a vanilla mana ability, the
/// second uses the new `sac_cost` field on `ActivatedAbility` so paying
/// the cost sacrifices Mind Stone before the Draw resolves.
pub fn mind_stone() -> CardDefinition {
    use crate::card::Selector;
    CardDefinition {
        name: "Mind Stone",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: crate::card::Value::Const(1),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: true,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Aether Spellbomb — {1} Artifact. {U}, Sacrifice this artifact: Return
/// target creature to its owner's hand. {1}, Sacrifice this artifact:
/// Draw a card. Both activated abilities use the new `sac_cost` field;
/// the first targets a creature on the battlefield (the bounce is wired
/// via the existing `Move(Target → Hand(OwnerOf(Target)))` pattern from
/// Vapor Snag).
pub fn aether_spellbomb() -> CardDefinition {
    use crate::card::{Selector, Value};
    use crate::effect::shortcut::target_filtered;
    use crate::effect::ZoneDest;
    use crate::card::SelectionRequirement;
    use crate::mana::u;
    CardDefinition {
        name: "Aether Spellbomb",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            // {U}, Sacrifice this: Return target creature to its owner's hand.
            ActivatedAbility {
                tap_cost: false,
                mana_cost: cost(&[u()]),
                effect: Effect::Move {
                    what: target_filtered(SelectionRequirement::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: true,
            },
            // {1}, Sacrifice this: Draw a card.
            ActivatedAbility {
                tap_cost: false,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: true,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Fellwar Stone — {2} Artifact. {T}: Add one mana of any color a land an
/// opponent controls could produce.
///
/// Approximation: the "matches an opponent's land colors" restriction is
/// dropped — this version produces any color, like Birds of Paradise. In
/// practice opponents' mana bases at the table cover most colors anyway,
/// and the engine has no per-source mana-color provenance tracking yet.
/// Acceptable until a `ManaPayload::AnyColorOpponentCanProduce` lands.
pub fn fellwar_stone() -> CardDefinition {
    CardDefinition {
        name: "Fellwar Stone",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
