//! Commander: The Celestial Toymaker's piles (`01FnAVVR`). Tests in
//! `tests/recent_b/cmdr_01fn_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{declare_target_opponent, on_attack};
use crate::effect::{Effect, LibraryPosition, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, u, w, x};

fn pile(chosen: bool) -> Selector {
    Selector::SeparatedPile { chosen }
}

fn top(n: Value) -> Selector {
    Selector::TopOfLibrary { who: PlayerRef::You, count: n }
}

fn to(what: Selector, dest: ZoneDest) -> Effect {
    Effect::Move { what, to: dest }
}

fn bottom() -> ZoneDest {
    ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Bottom }
}

/// The Celestial Toymaker — {W}{U}{B} legendary 2/4 Rogue Artificer. On
/// attack, the top three are exiled in two piles you make; the defending
/// player picks one for your hand. Each end step, each opponent loses 2 per
/// pile/guess spell or ability this turn. ⚠ The headless defender sees the
/// face-down pile's names (INCOMPLETE_CARDS).
pub fn the_celestial_toymaker() -> CardDefinition {
    CardDefinition {
        name: "The Celestial Toymaker",
        cost: cost(&[w(), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rogue, CreatureType::Artificer], ..Default::default() },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![
            on_attack(Effect::SeparateIntoPiles {
                what: top(Value::Const(3)),
                splitter: PlayerRef::You,
                chooser: PlayerRef::DefendingPlayer,
                chosen: Box::new(to(pile(true), ZoneDest::Hand(PlayerRef::You))),
                other: Box::new(to(pile(false), ZoneDest::Exile)),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Times(Box::new(Value::Const(2)), Box::new(Value::PileOrGuessEffectsThisTurn)),
                },
            },
        ],
        ..Default::default()
    }
}

/// Phyrexian Portal — {3} artifact. {3}: with ten or more cards in your
/// library, target opponent splits the top ten; you exile one pile, tutor a
/// card from the other, and shuffle the rest back. ⚠ The piles are face
/// down, but the headless chooser reads their names (INCOMPLETE_CARDS).
pub fn phyrexian_portal() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Portal",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Seq(vec![
                declare_target_opponent(),
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::LibrarySizeOf(PlayerRef::You), Value::Const(10)),
                    then: Box::new(Effect::Seq(vec![
                        Effect::SeparateIntoPiles {
                            what: top(Value::Const(10)),
                            splitter: PlayerRef::Target(0),
                            chooser: PlayerRef::You,
                            chosen: Box::new(to(pile(true), ZoneDest::Exile)),
                            other: Box::new(Effect::ChooseOneAmong {
                                what: pile(false),
                                chooser: PlayerRef::You,
                                chosen: Box::new(to(pile(true), ZoneDest::Hand(PlayerRef::You))),
                                other: Box::new(Effect::Noop),
                            }),
                        },
                        Effect::ShuffleLibrary { who: PlayerRef::You },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sphinx of Clear Skies — {3}{U}{U} 5/5 flying Sphinx, ward {2}. Domain —
/// combat damage to a player: an opponent splits your top X (X = basic land
/// types among your lands); one pile to hand, the other to the graveyard.
pub fn sphinx_of_clear_skies() -> CardDefinition {
    CardDefinition {
        name: "Sphinx of Clear Skies",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sphinx], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Ward(crate::card::WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::FactOrFiction { count: Value::DomainCount(PlayerRef::You), to_bottom: false },
        }],
        ..Default::default()
    }
}

/// Epiphany at the Drownyard — {X}{U} instant. You split the top X+1; an
/// opponent picks the pile for your hand, the other goes to your graveyard.
pub fn epiphany_at_the_drownyard() -> CardDefinition {
    CardDefinition {
        name: "Epiphany at the Drownyard",
        cost: cost(&[x(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::SeparateIntoPiles {
            what: top(Value::Sum(vec![Value::XFromCost, Value::ONE])),
            splitter: PlayerRef::You,
            chooser: PlayerRef::HostileOpponent,
            chosen: Box::new(to(pile(true), ZoneDest::Hand(PlayerRef::You))),
            other: Box::new(to(pile(false), ZoneDest::Graveyard)),
        },
        ..Default::default()
    }
}

/// Truth or Tale — {1}{U} instant. You split the top five; an opponent picks
/// a pile, you take one card from it, and every other card goes to the
/// bottom. The unchosen pile moves first: the inner pick re-uses the piles.
pub fn truth_or_tale() -> CardDefinition {
    CardDefinition {
        name: "Truth or Tale",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::SeparateIntoPiles {
            what: top(Value::Const(5)),
            splitter: PlayerRef::You,
            chooser: PlayerRef::HostileOpponent,
            chosen: Box::new(Effect::Seq(vec![
                to(pile(false), bottom()),
                Effect::ChooseOneAmong {
                    what: pile(true),
                    chooser: PlayerRef::You,
                    chosen: Box::new(to(pile(true), ZoneDest::Hand(PlayerRef::You))),
                    other: Box::new(to(pile(false), bottom())),
                },
            ])),
            other: Box::new(Effect::Noop),
        },
        ..Default::default()
    }
}
