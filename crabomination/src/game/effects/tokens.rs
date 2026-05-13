//! Token-definition factories (Food / Treasure / Blood / Clue) and the
//! converter that lifts a runtime `TokenDefinition` into a `CardDefinition`
//! suitable for use as a battlefield permanent.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, Effect, Selector, Subtypes,
    TokenDefinition, Value,
};
use crate::effect::{ManaPayload, PlayerRef};
use crate::mana::{ManaCost, ManaSymbol};

pub fn token_to_card_definition(token: &TokenDefinition) -> CardDefinition {
    CardDefinition {
        // CardDefinition.name is &'static str; tokens carry an owned
        // String (so they round-trip through serde), so we leak a copy
        // here to extend its lifetime. The leak is bounded by the
        // number of unique token names produced over a session.
        name: crate::static_str_serde::intern(token.name.clone()),
        cost: ManaCost::default(),
        supertypes: token.supertypes.clone(),
        card_types: token.card_types.clone(),
        subtypes: token.subtypes.clone(),
        power: token.power,
        toughness: token.toughness,
        base_loyalty: 0,
        keywords: token.keywords.clone(),
        static_abilities: vec![],
        effect: Effect::Noop,
        activated_abilities: token.activated_abilities.clone(),
        triggered_abilities: token.triggered_abilities.clone(),
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

pub fn food_token() -> TokenDefinition {
    TokenDefinition {
        name: "Food".into(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Artifact],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Food],
            ..Default::default()
        },
        // {2}, {T}, Sacrifice this artifact: Gain 3 life.
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost {
                symbols: vec![ManaSymbol::Generic(2)],
            },
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
        }],
        triggered_abilities: vec![],
    }
}

pub fn treasure_token() -> TokenDefinition {
    TokenDefinition {
        name: "Treasure".into(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Artifact],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Treasure],
            ..Default::default()
        },
        // {T}, Sacrifice this artifact: Add one mana of any color.
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
        }],
        triggered_abilities: vec![],
    }
}

pub fn blood_token() -> TokenDefinition {
    TokenDefinition {
        name: "Blood".into(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Artifact],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Blood],
            ..Default::default()
        },
        // {1}, {T}, Discard a card, Sacrifice this artifact: Draw a card.
        // Discard isn't expressible as a cost yet, so it lives in the
        // resolution sequence; AutoDecider picks the first hand card.
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost {
                symbols: vec![ManaSymbol::Generic(1)],
            },
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
        }],
        triggered_abilities: vec![],
    }
}

pub fn clue_token() -> TokenDefinition {
    TokenDefinition {
        name: "Clue".into(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Artifact],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Clue],
            ..Default::default()
        },
        // {2}, Sacrifice this artifact: Draw a card.
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost {
                symbols: vec![ManaSymbol::Generic(2)],
            },
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
        }],
        triggered_abilities: vec![],
    }
}
