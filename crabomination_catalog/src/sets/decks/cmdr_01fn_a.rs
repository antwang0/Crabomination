//! Commander: the 2026-10-20 (`01FnAVVR`) closers — Blim, Comedic Genius's
//! graveyard engines. Tests in `tests/recent_b/cmdr_01fn_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StateTriggeredAbility, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, mint_token};
use crate::game::types::TurnStep;
use crate::effect::{Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{Color, b, cost, generic};

/// Immortal Coil — {2}{B}{B} artifact. {T}, exile two cards from your
/// graveyard: draw a card. Damage to you is prevented and exiles a graveyard
/// card per point; an empty graveyard loses you the game (CR 603.8).
pub fn immortal_coil() -> CardDefinition {
    CardDefinition {
        name: "Immortal Coil",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_other_filter: Some((R::Any, 2)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to you, prevent that damage. Exile a card from your graveyard for each 1 damage prevented this way.",
            effect: StaticEffect::PreventDamageToYouExileFromGraveyard,
        }],
        state_trigger: Some(StateTriggeredAbility {
            condition: Predicate::ValueEquals(Value::GraveyardSizeOf(PlayerRef::You), Value::Const(0)),
            effect: Effect::LoseGame { who: PlayerRef::You },
        }),
        ..Default::default()
    }
}

/// Priest of the Blood Rite — {3}{B}{B} 2/2 Human Cleric. ETB: a 5/5 black
/// flying Demon. At the beginning of your upkeep, you lose 2 life.
pub fn priest_of_the_blood_rite() -> CardDefinition {
    let demon = TokenDefinition {
        name: "Demon".into(),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Priest of the Blood Rite",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Cleric], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            etb(mint_token(demon, 1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            },
        ],
        ..Default::default()
    }
}

/// Colfenor's Plans — {2}{B}{B} enchantment. ETB: exile your top seven face
/// down; you may play them while it remains. Skip your draw step; you can't
/// cast more than one spell each turn.
pub fn colfenors_plans() -> CardDefinition {
    CardDefinition {
        name: "Colfenor's Plans",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::ExileFaceDown {
                body: Box::new(Effect::Move {
                    what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(7) },
                    to: ZoneDest::ExileWithSourceStamp,
                }),
            },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: crate::card::MayPlayDuration::WhileSourceOnBattlefield {
                    source: crate::card::CardId(0),
                    one_spell_a_turn: false,
                },
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        ]))],
        static_abilities: vec![
            StaticAbility { description: "Skip your draw step.", effect: StaticEffect::ControllerSkipsDrawStep },
            StaticAbility {
                description: "You can't cast more than one spell each turn.",
                effect: StaticEffect::ControllerOneSpellPerTurn,
            },
        ],
        ..Default::default()
    }
}

/// Forbidden Crypt — {3}{B}{B} enchantment. Your draws become "return a
/// graveyard card to hand, or lose"; your cards bound for the graveyard are
/// exiled instead.
pub fn forbidden_crypt() -> CardDefinition {
    CardDefinition {
        name: "Forbidden Crypt",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "If you would draw a card, return a card from your graveyard to your hand instead. If you can't, you lose the game.",
                effect: StaticEffect::ReplaceDrawWithGraveyardReturn,
            },
            StaticAbility {
                description: "If a card would be put into your graveyard from anywhere, exile that card instead.",
                effect: StaticEffect::ExileCardsBoundForGraveyard {
                    opponents_only: false,
                    own_only: true,
                    colors: None,
                    card_types: None,
                    void_counter: false,
                    stamp_source: false,
                },
            },
        ],
        ..Default::default()
    }
}
