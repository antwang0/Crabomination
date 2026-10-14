//! Commander: two-card gaps of EDHREC's top-2000 commanders (the 2026-10-13
//! rescan), thirteenth file — Hans Eriksson, Garland (Nephalia Smuggler is
//! `cmdr_rank3k_a`'s), Denry Klin and The
//! Wandering Rescuer.
//! Tests in `tests/recent_b/cmdr_rank2k_m.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes,
    Supertype, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, LibraryPosition, LookPick, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, ManaCost};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn top() -> Selector {
    Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE }
}

/// Hans Eriksson — {2}{R}{G} legendary 1/4 Human Scout. Whenever it attacks,
/// reveal the top card of your library: a creature card enters tapped and
/// attacking (the defender Hans attacks) and fights Hans; anything else goes
/// to your hand.
pub fn hans_eriksson() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::RevealTopOfLibrary { who: PlayerRef::You },
            Effect::If {
                cond: Predicate::EntityMatches { what: top(), filter: R::Creature },
                then: Box::new(Effect::Seq(vec![
                    Effect::Move { what: top(), to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true } },
                    Effect::JoinCombatAttacking { what: Selector::LastMoved },
                    Effect::Fight { attacker: Selector::LastMoved, defender: Selector::This },
                ])),
                else_: Box::new(Effect::Move { what: top(), to: ZoneDest::Hand(PlayerRef::You) }),
            },
        ]))],
        ..creature("Hans Eriksson", cost(&[generic(2), r(), g()]), vec![CreatureType::Human, CreatureType::Scout], 1, 4)
    }
}

/// Mwonvuli Beast Tracker — {1}{G}{G} 2/1 Human Scout; entering, search for a
/// creature card with deathtouch, hexproof, reach or trample, reveal it,
/// shuffle and put it on top.
pub fn mwonvuli_beast_tracker() -> CardDefinition {
    let kw = |k| R::HasKeyword(k);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::Creature.and(
                kw(Keyword::Deathtouch).or(kw(Keyword::Hexproof)).or(kw(Keyword::Reach)).or(kw(Keyword::Trample)),
            ),
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
        })],
        ..creature(
            "Mwonvuli Beast Tracker",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Human, CreatureType::Scout],
            2,
            1,
        )
    }
}

/// Treacherous Urge — {4}{B} instant. Target opponent reveals their hand; you
/// may put a creature card from it onto the battlefield under your control.
/// It gains haste; sacrifice it at the beginning of the next end step.
pub fn treacherous_urge() -> CardDefinition {
    CardDefinition {
        name: "Treacherous Urge",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LookAtHand { who: target_filtered(R::OpponentPlayer) },
            Effect::RevealHand { who: PlayerRef::Target(0) },
            Effect::MoveChosen {
                from: Selector::CardsInZone { who: PlayerRef::Target(0), zone: Zone::Hand, filter: R::Creature },
                filter: None,
                count: Value::ONE,
                up_to: true,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Haste, duration: Duration::Permanent },
            Effect::SacrificeAtNextEndStep { what: Selector::LastMoved },
        ]),
        ..Default::default()
    }
}

/// Keensight Mentor — {2}{W} 1/4 Human Cleric. Entering, a vigilance counter
/// on target non-Human creature you control. {1}{W}, {T}: a +1/+1 counter on
/// each creature you control with vigilance.
pub fn keensight_mentor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddKeywordCounter {
            what: target_filtered(
                R::Creature.and(R::ControlledByYou).and(R::Not(Box::new(R::HasCreatureType(CreatureType::Human)))),
            ),
            keyword: Keyword::Vigilance,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::HasKeyword(Keyword::Vigilance))),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature("Keensight Mentor", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 4)
    }
}

/// Knight-Errant of Eos — {4}{W} 4/4 convoke Human Knight. Entering, look at
/// the top six: up to two creature cards with mana value X or less (X = the
/// creatures that convoked it) to hand, then shuffle.
pub fn knight_errant_of_eos() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![etb(Effect::WithX {
            x: Value::count(Selector::CreaturesThatConvokedSource),
            body: Box::new(Effect::Seq(vec![
                Effect::LookPickToHand(Box::new(LookPick {
                    who: PlayerRef::You,
                    count: Value::Const(6),
                    take: Some(Value::Const(2)),
                    pick_filter: Some(R::Creature.and(R::ManaValueAtMostXFromCost)),
                    optional: true,
                    ..Default::default()
                })),
                Effect::ShuffleLibrary { who: PlayerRef::You },
            ])),
        })],
        ..creature("Knight-Errant of Eos", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Knight], 4, 4)
    }
}

/// Zephyr Singer — {2}{U}{U} 3/4 convoke, flying, vigilance Siren Pirate.
/// Entering, a flying counter on each creature that convoked it.
pub fn zephyr_singer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke, Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::AddKeywordCounter {
            what: Selector::CreaturesThatConvokedSource,
            keyword: Keyword::Flying,
            amount: Value::ONE,
        })],
        ..creature("Zephyr Singer", cost(&[generic(2), u(), u()]), vec![CreatureType::Siren, CreatureType::Pirate], 3, 4)
    }
}
