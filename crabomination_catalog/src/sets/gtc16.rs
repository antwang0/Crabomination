//! Gatecrash (GTC) wave 16: the remaining primitive-gated guild cards. Tests in
//! `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, MayPlayDuration, SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect};
use crate::mana::{cost, generic, hybrid, r, w, x, Color};

/// Aurelia's Fury — {X}{R}{W} Instant. Deals X damage divided among any number
/// of targets; each creature dealt damage this way is tapped, and each player
/// dealt damage this way can't cast noncreature spells this turn.
pub fn aurelias_fury() -> CardDefinition {
    CardDefinition {
        name: "Aurelia's Fury",
        cost: cost(&[x(), r(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamageDivided {
                total: Value::XFromCost,
                filter: R::Creature.or(R::Player).or(R::Planeswalker),
                max_targets: 20,
            },
            // `DamagedThisResolution` yields only creatures + players; Tap
            // ignores the players and the noncreature lock ignores the
            // creatures.
            Effect::Tap { what: Selector::DamagedThisResolution { filter: R::Creature } },
            Effect::CantCastNoncreatureThisTurn {
                who: Selector::DamagedThisResolution { filter: R::Creature },
            },
        ]),
        ..Default::default()
    }
}

/// Nightveil Specter — {U/B}{U/B}{U/B} 2/3 Specter. Flying; combat damage to a
/// player exiles that player's top library card, and its controller may play
/// lands / cast spells from among cards exiled with it (modeled as a
/// while-exiled may-play grant paying the card's own cost).
pub fn nightveil_specter() -> CardDefinition {
    let ub = || hybrid(Color::Blue, Color::Black);
    CardDefinition {
        name: "Nightveil Specter",
        cost: cost(&[ub(), ub(), ub()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Specter], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::DefendingPlayer,
                count: Value::Const(1),
                duration: MayPlayDuration::WhileExiled,
                pay_any_color: false,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        ..Default::default()
    }
}

/// Glaring Spotlight — {1} Artifact. Opponents' hexproof creatures can be
/// targeted by your spells and abilities; {3}, Sacrifice it: your creatures
/// gain hexproof and can't be blocked this turn.
pub fn glaring_spotlight() -> CardDefinition {
    CardDefinition {
        name: "Glaring Spotlight",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Opponents' hexproof creatures can be targeted by your spells and abilities.",
            effect: StaticEffect::IgnoreOpponentsCreatureHexproof,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sac_cost: true,
            effect: Effect::GrantKeywords {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keywords: vec![Keyword::Hexproof, Keyword::Unblockable],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
