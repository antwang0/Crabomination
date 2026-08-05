//! OTJ gap batch on existing primitives: Map the Frontier (land tutor),
//! Neutralize the Guards (-1/-1 sweep + surveil), Rise of the Varmints (Plot
//! token swarm), Overzealous Muscle (crime-gated indestructible), and Outlaws'
//! Fury (team pump + outlaw impulse). Tests in
//! `crabomination/src/tests/recent188.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, LandType,
    MayPlayDuration, SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r};

/// A 2/1 green Varmint token.
fn varmint_token() -> TokenDefinition {
    TokenDefinition {
        name: "Varmint".to_string(),
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Varmint],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        ..Default::default()
    }
}

/// Map the Frontier — {3}{G} Sorcery. Search your library for up to two basic
/// land and/or Desert cards, put them onto the battlefield tapped, then shuffle.
pub fn map_the_frontier() -> CardDefinition {
    CardDefinition {
        name: "Map the Frontier",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::IsBasicLand.or(R::HasLandType(LandType::Desert)),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: true,
            },
            count: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Neutralize the Guards — {2}{B} Instant. Creatures target opponent controls get
/// -1/-1 until end of turn. Surveil 2.
pub fn neutralize_the_guards() -> CardDefinition {
    CardDefinition {
        name: "Neutralize the Guards",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Rise of the Varmints — {3}{G} Sorcery. Create X 2/1 green Varmint tokens,
/// where X is the number of creature cards in your graveyard. Plot {2}{G}.
pub fn rise_of_the_varmints() -> CardDefinition {
    CardDefinition {
        name: "Rise of the Varmints",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        plot_cost: Some(cost(&[generic(2), g()])),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::CardsInGraveyardMatching {
                who: PlayerRef::You,
                filter: R::Creature,
            },
            definition: varmint_token(),
        },
        ..Default::default()
    }
}

/// Overzealous Muscle — {4}{B} 5/4 Ogre Mercenary. Whenever you commit a crime
/// during your turn, it gains indestructible until end of turn.
pub fn overzealous_muscle() -> CardDefinition {
    CardDefinition {
        name: "Overzealous Muscle",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Outlaws' Fury — {2}{R} Instant. Creatures you control get +2/+0 until end of
/// turn. If you control an outlaw, exile the top card of your library; until the
/// end of your next turn you may play it.
pub fn outlaws_fury() -> CardDefinition {
    CardDefinition {
        name: "Outlaws' Fury",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    R::IsOutlaw.and(R::ControlledByYou),
                )),
                then: Box::new(Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}
