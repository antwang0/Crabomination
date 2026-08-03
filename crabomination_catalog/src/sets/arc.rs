//! Archenemy (ARC) — scheme cards for the CR 904 variant. Schemes live face
//! down in the archenemy's scheme deck and are set in motion off the top at
//! the start of each of their precombat main phases. Tests in
//! `classic_sets/arc`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, StaticEffect, Value, shortcut::drain,
};
use crate::mana::Color;
use crate::game::types::TurnStep;

/// A scheme shell: the card type plus the "when you set this scheme in
/// motion" trigger every non-ongoing scheme is built around.
fn scheme(name: &'static str, on_set_in_motion: Effect) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Scheme],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SetInMotion, EventScope::SelfSource),
            effect: on_set_in_motion,
        }],
        ..Default::default()
    }
}

/// An ongoing scheme: statics that function from the command zone plus the
/// trigger that abandons it (CR 904.11).
fn ongoing_scheme(
    name: &'static str,
    statics: Vec<StaticAbility>,
    abandon: TriggeredAbility,
) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Scheme],
        supertypes: vec![Supertype::Ongoing],
        static_abilities: statics,
        triggered_abilities: vec![abandon],
        ..Default::default()
    }
}

fn token(
    name: &str,
    p: i32,
    t: i32,
    colors: Vec<Color>,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

/// I Delight in Your Convulsions — a three-point drain off the top of the
/// scheme deck.
pub fn i_delight_in_your_convulsions() -> CardDefinition {
    scheme("I Delight in Your Convulsions", drain(3))
}

/// Delight in the Hunt — a 3/3 Horror plus a fog for your board.
pub fn delight_in_the_hunt() -> CardDefinition {
    scheme(
        "Delight in the Hunt",
        Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: token(
                    "Horror",
                    3,
                    3,
                    vec![Color::Black],
                    vec![CreatureType::Horror],
                    vec![],
                ),
            },
            Effect::PreventAllDamageThisTurn {
                target: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                redirect_to: None,
            },
        ]),
    )
}

/// Evil Comes to Fruition — seven Plants, or seven Elementals off ten lands.
pub fn evil_comes_to_fruition() -> CardDefinition {
    scheme(
        "Evil Comes to Fruition",
        Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::PermanentCountControlledByMatching(PlayerRef::You, R::Land),
                Value::Const(10),
            ),
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(7),
                definition: token(
                    "Elemental",
                    3,
                    3,
                    vec![Color::Green],
                    vec![CreatureType::Elemental],
                    vec![],
                ),
            }),
            else_: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(7),
                definition: token(
                    "Plant",
                    0,
                    1,
                    vec![Color::Green],
                    vec![CreatureType::Plant],
                    vec![],
                ),
            }),
        },
    )
}

/// Kneel Before My Legions — a Scarecrow, or a team pump.
pub fn kneel_before_my_legions() -> CardDefinition {
    scheme(
        "Kneel Before My Legions",
        Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Scarecrow".into(),
                    power: 4,
                    toughness: 4,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Scarecrow],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Vigilance],
                    ..Default::default()
                },
            },
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// All in Good Time — an extra turn off the scheme deck.
pub fn all_in_good_time() -> CardDefinition {
    scheme(
        "All in Good Time",
        Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
    )
}

/// Embrace My Diabolical Vision — everyone reshuffles; you draw seven and they
/// draw four.
pub fn embrace_my_diabolical_vision() -> CardDefinition {
    scheme(
        "Embrace My Diabolical Vision",
        Effect::Seq(vec![
            Effect::ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef::EachPlayer },
            Effect::Draw { who: Selector::You, amount: Value::Const(7) },
            Effect::Draw {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(4),
            },
        ]),
    )
}

/// Fear My Authority — an ongoing anthem you keep paying for.
pub fn fear_my_authority() -> CardDefinition {
    ongoing_scheme(
        "Fear My Authority",
        vec![StaticAbility {
            description: "Creatures you control get +2/+2 and have fear.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::ControlledByYou),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Fear],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayPayLife {
                description: "Pay 3 life to keep Fear My Authority?".into(),
                amount: Value::Const(3),
                body: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::AbandonThisScheme)),
            },
        },
    )
}

/// I Bask in Your Silent Awe — an ongoing spell lock that lapses when your
/// opponents stop casting.
pub fn i_bask_in_your_silent_awe() -> CardDefinition {
    ongoing_scheme(
        "I Bask in Your Silent Awe",
        vec![StaticAbility {
            description: "Each opponent can't cast more than one spell each turn.",
            effect: StaticEffect::OpponentsOneSpellPerTurn,
        }],
        TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::Not(Box::new(Predicate::OpponentCastSpellSinceYourTurn { who: PlayerRef::You }))),
            effect: Effect::AbandonThisScheme,
        },
    )
}
