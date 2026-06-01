//! Token-definition factories (Food / Treasure / Blood / Clue) and the
//! converter that lifts a runtime `TokenDefinition` into a `CardDefinition`
//! suitable for use as a battlefield permanent.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Selector, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{ManaPayload, PlayerRef};
use crate::mana::{Color, ManaCost, ManaSymbol};

/// CR 111.4 — synthesize a token's display name from its subtypes when no
/// explicit name was given. The result is the joined subtype names plus
/// " Token" (e.g. `Subtypes { creature_types: [Spirit] }` → `"Spirit Token"`).
/// Falls back to bare `"Token"` when the token has no subtypes at all.
fn derive_name_from_subtypes(subtypes: &Subtypes) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.extend(subtypes.creature_types.iter().map(|s| format!("{:?}", s)));
    parts.extend(subtypes.artifact_subtypes.iter().map(|s| format!("{:?}", s)));
    parts.extend(subtypes.enchantment_subtypes.iter().map(|s| format!("{:?}", s)));
    parts.extend(subtypes.land_types.iter().map(|s| format!("{:?}", s)));
    parts.extend(subtypes.planeswalker_subtypes.iter().map(|s| format!("{:?}", s)));
    if parts.is_empty() {
        "Token".to_string()
    } else {
        format!("{} Token", parts.join(" "))
    }
}

pub fn token_to_card_definition(token: &TokenDefinition) -> CardDefinition {
    // CR 111.4: if the spell/ability that creates the token didn't specify
    // a name, the name is its subtypes plus the word "Token". Every catalog
    // factory ships a name today; this fallback covers future code paths
    // (e.g. copy-token primitives) that might construct a `TokenDefinition`
    // without filling in `name`.
    let resolved_name = if token.name.is_empty() {
        derive_name_from_subtypes(&token.subtypes)
    } else {
        token.name.clone()
    };
    CardDefinition {
        // CardDefinition.name is &'static str; tokens carry an owned
        // String (so they round-trip through serde), so we leak a copy
        // here to extend its lifetime. The leak is bounded by the
        // number of unique token names produced over a session.
        name: crate::static_str_serde::intern(resolved_name),
        cost: ManaCost::default(),
        supertypes: token.supertypes.clone(),
        card_types: token.card_types.clone(),
        subtypes: token.subtypes.clone(),
        power: token.power,
        toughness: token.toughness,
        keywords: token.keywords.clone(),
        effect: Effect::Noop,
        activated_abilities: token.activated_abilities.clone(),
        triggered_abilities: token.triggered_abilities.clone(),
        ..Default::default()
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
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
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
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
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
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
    }
}

// ── College / set tokens ─────────────────────────────────────────────────────
//
// These engine-baked creature tokens are referenced both by the catalog
// (cards that mint them) and by the `effect::shortcut::mint_*` builders, so
// they live in the base crate to break the catalog↔effect cycle.

/// Strixhaven Pest token: 1/1 black-and-green creature with
/// "When this creature dies, you gain 1 life." Shared by Pest
/// Summoning, Tend the Pests, Eyetwitch, Hunt for Specimens, etc.
pub fn stx_pest_token() -> TokenDefinition {
    TokenDefinition {
        name: "Pest".to_string(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
    }
}

/// 1/1 white-and-black Inkling creature token with flying. Used by several
/// SOS Silverquill / White cards.
pub fn inkling_token() -> TokenDefinition {
    TokenDefinition {
        name: "Inkling".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Black],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

/// 0/0 green-and-blue Fractal creature token. Used by Quandrix scaling
/// payoffs; lives or dies based on the +1/+1 counters its creator stamps on.
pub fn fractal_token() -> TokenDefinition {
    TokenDefinition {
        name: "Fractal".into(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::Blue],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

/// 2/2 red-and-white Spirit creature token. Used by Lorehold-flavoured
/// SOS cards (Group Project, Living History's ETB, etc.).
pub fn spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 2,
        toughness: 2,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

/// 2/2 red-and-white Spirit creature token with flying. Used by Lorehold
/// cards (and SOS Group Project / Living History) that mint a Spirit body.
pub fn lorehold_spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![],
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
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![],
    }
}
