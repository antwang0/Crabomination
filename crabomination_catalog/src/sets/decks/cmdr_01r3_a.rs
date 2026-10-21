//! Commander: the 2026-10-21 (`01R3xh4r`) one-gap pair closers. Tests in
//! `tests/recent_b/cmdr_01r3_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Prototype, SelectionRequirement as R, Subtypes, TriggeredAbility, Zone,
};
use crate::effect::shortcut::{craft, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{Color, cost, generic, hybrid, u};

/// Arcane Proxy — {7} 4/3 artifact creature Wizard; prototype {1}{U}{U} 2/1
/// (CR 702.160). When it enters, if you cast it, exile target instant or
/// sorcery card with mana value <= its power from your graveyard, copy it, and
/// you may cast the copy without paying its mana cost (CR 707.12).
pub fn arcane_proxy() -> CardDefinition {
    let spell = R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Arcane Proxy",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        power: 4,
        toughness: 3,
        prototype: Some(Box::new(Prototype { cost: cost(&[generic(1), u(), u()]), power: 2, toughness: 1 })),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceWasCast),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(spell.and(R::InYourGraveyard).and(R::ManaValueAtMostSourcePower)),
                    to: ZoneDest::Exile,
                },
                Effect::CastWithoutPayingImmediate {
                    reduce_generic: 0,
                    pay_own_cost: false,
                    what: Selector::LastMoved,
                    source_zone: Zone::Exile,
                    exile_after: false,
                    copy: true,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Worldpurge — {4}{W/U}{W/U}{W/U}{W/U} sorcery. Return all permanents to
/// their owners' hands; each player keeps up to seven cards in hand and
/// shuffles the rest into their library; each player loses all unspent mana.
pub fn worldpurge() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    CardDefinition {
        name: "Worldpurge",
        cost: cost(&[generic(4), wu(), wu(), wu(), wu()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move { what: Selector::EachPermanent(R::Any), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            Effect::EachPlayerKeepsUpToFromHandShufflesRest { keep: 7 },
            Effect::LoseUnspentMana { who: PlayerRef::EachPlayer },
        ]),
        ..Default::default()
    }
}

/// Braided Net // Braided Quipu — {2}{U} artifact; enters with three net
/// counters. {T}, remove a net counter: tap another target nonland permanent;
/// its activated abilities can't be activated for as long as it remains tapped
/// (CR 611.2c, `Duration::WhileTapped`). Craft with an artifact {1}{U}.
/// Braided Quipu: {3}{U}, {T}: draw a card for each artifact you control, then
/// put it into its owner's library third from the top.
pub fn braided_net() -> CardDefinition {
    let quipu = CardDefinition {
        name: "Braided Quipu",
        card_types: vec![CardType::Artifact],
        color_indicator: vec![Color::Blue],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)))),
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::This)), pos: LibraryPosition::FromTop(2) },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    };
    let other = R::Permanent.and(R::Land.negate()).and(R::OtherThanSource);
    CardDefinition {
        name: "Braided Net",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Net, Value::Const(3))),
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Net, 1)),
                effect: Effect::Seq(vec![
                    Effect::Tap { what: target_filtered(other) },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::CantActivateAbilities,
                        duration: Duration::WhileTapped,
                    },
                ]),
                ..Default::default()
            },
            craft(cost(&[generic(1), u()]), R::Artifact, 1),
        ],
        back_face: Some(Box::new(quipu)),
        ..Default::default()
    }
}
