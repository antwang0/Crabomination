//! Gatecrash (GTC) wave 5: X-scaled mass reanimation, a win condition, a
//! self-copying beater, and removal. Tests in `classic_sets/gtc`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Selector, ZoneDest, ZoneRef};
use crate::mana::{b, cost, generic, g, hybrid, u, w, x, Color};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}

/// Immortal Servitude — {X}{W/B}{W/B}{W/B} Sorcery. Return each creature card
/// with mana value X from your graveyard to the battlefield.
pub fn immortal_servitude() -> CardDefinition {
    let wb = || hybrid(Color::White, Color::Black);
    CardDefinition {
        name: "Immortal Servitude",
        cost: cost(&[x(), wb(), wb(), wb()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::EachMatching {
                zone: ZoneRef::Graveyard(PlayerRef::You),
                filter: R::Creature.and(R::ManaValueExactlyXFromCost),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Biovisionary — {1}{G}{U} 2/3 Human Wizard. At the beginning of the end step,
/// if you control four or more creatures named Biovisionary, you win the game.
pub fn biovisionary() -> CardDefinition {
    CardDefinition {
        name: "Biovisionary",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::HasName("Biovisionary".into())),
                    ),
                    n: Value::Const(4),
                },
                then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Giant Adephage — {5}{G}{G} 7/7 Insect with trample. Whenever it deals combat
/// damage to a player, create a token that's a copy of it.
pub fn giant_adephage() -> CardDefinition {
    CardDefinition {
        name: "Giant Adephage",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Insect]),
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Executioner's Swing — {W}{B} Instant. Target creature that dealt damage this
/// turn gets -5/-5 until end of turn.
pub fn executioners_swing() -> CardDefinition {
    CardDefinition {
        name: "Executioner's Swing",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::DealtDamageThisTurn)),
            power: Value::Const(-5),
            toughness: Value::Const(-5),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}
