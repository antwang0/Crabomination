//! Commander: the 2026-10-21 (`01R3xh4r`) one-gap pair closers. Tests in
//! `tests/recent_b/cmdr_01r3_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, Prototype, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    TriggeredAbility, Zone,
};
use crate::effect::shortcut::{craft, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{Color, cost, g, generic, hybrid, r, u, w};

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

fn creature(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Duskrider Peregrine — {5}{W} 3/3 Bird; flying, protection from black,
/// suspend 3—{1}{W}.
pub fn duskrider_peregrine() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Protection(Color::Black),
            Keyword::Suspend(3, cost(&[generic(1), w()])),
        ],
        ..creature("Duskrider Peregrine", cost(&[generic(5), w()]), vec![CreatureType::Bird], 3, 3)
    }
}

/// Deem Worthy — {4}{R} instant: 7 damage to target creature. Cycling
/// {3}{R}; when you cycle it, you may have it deal 2 damage to target creature
/// (CR 702.29c).
pub fn deem_worthy() -> CardDefinition {
    CardDefinition {
        name: "Deem Worthy",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(3), r()]))],
        effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(7) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Deal 2 damage to target creature?".into(),
                body: Box::new(Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(2) }),
            },
        }],
        ..Default::default()
    }
}

/// Dire Wolves — {2}{G} 2/2 Wolf; banding as long as you control a Plains.
pub fn dire_wolves() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has banding as long as you control a Plains.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Banding,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::HasLandType(LandType::Plains).and(R::ControlledByYou)),
                    n: Value::ONE,
                },
            },
        }],
        ..creature("Dire Wolves", cost(&[generic(2), g()]), vec![CreatureType::Wolf], 2, 2)
    }
}

/// Nature's Blessing — {2}{G}{W} enchantment. {G}{W}, discard a card: put a
/// +1/+1 counter on target creature, or it gains banding, first strike or
/// trample indefinitely.
pub fn natures_blessing() -> CardDefinition {
    let creature = || target_filtered(R::Creature);
    let gain = |keyword| Effect::GrantKeyword { what: creature(), keyword, duration: Duration::Permanent };
    CardDefinition {
        name: "Nature's Blessing",
        cost: cost(&[generic(2), g(), w()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), w()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::ChooseMode(vec![
                Effect::AddCounter { what: creature(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                gain(Keyword::Banding),
                gain(Keyword::FirstStrike),
                gain(Keyword::Trample),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Disciple of Caelus Nin — {4}{W} 3/4 Human Wizard. When it enters, starting
/// with you, each player chooses up to five permanents they control; every
/// other permanent but this one phases out. Permanents can't phase in.
pub fn disciple_of_caelus_nin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::EachPlayerKeepsUpToPermanentsRestPhaseOut { keep: 5 },
        }],
        static_abilities: vec![StaticAbility {
            description: "Permanents can't phase in.",
            effect: StaticEffect::PermanentsCantPhaseIn,
        }],
        ..creature(
            "Disciple of Caelus Nin",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            4,
        )
    }
}
