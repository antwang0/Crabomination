//! Gatecrash (GTC) wave 16: the remaining primitive-gated guild cards. Tests in
//! `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, MayPlayDuration, SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::mana::{b, cost, generic, hybrid, r, u, w, x, Color};

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

/// Bane Alley Broker — {1}{U}{B} 0/3 Human Rogue. {T}: draw, then stash a card
/// from hand face down under it; {U}{B}, {T}: return a stashed card to its
/// owner's hand.
pub fn bane_alley_broker() -> CardDefinition {
    CardDefinition {
        name: "Bane Alley Broker",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    Effect::ExileChosenFromHand {
                        from: Selector::You,
                        count: Value::Const(1),
                        filter: R::Any,
                        link_to_source: true,
                        face_down: true,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[u(), b()]),
                effect: Effect::Move {
                    what: Selector::one_of(Selector::CardExiledWithSource),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                ..Default::default()
            },
        ],
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
