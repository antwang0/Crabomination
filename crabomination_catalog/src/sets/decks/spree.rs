//! Outlaws of Thunder Junction Spree spells (CR 702.172). Each card's effect
//! is `Effect::Spree`; cast via `GameAction::CastSpellSpree`, choosing one or
//! more modes whose mana costs fold into the total cost. Tests in
//! `tests/spree.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{CardDefinition, CardType, CounterType, Keyword, LandType, TokenDefinition};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Selector, SpreeMode, Value, ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w};

fn spree(modes: Vec<SpreeMode>) -> Effect {
    Effect::Spree { modes }
}

fn mode(c: crate::mana::ManaCost, effect: Effect) -> SpreeMode {
    SpreeMode { cost: c, effect }
}

/// Explosive Derailment — {R} Instant. Spree: +{2} deal 4 to target creature;
/// +{2} destroy target artifact.
pub fn explosive_derailment() -> CardDefinition {
    CardDefinition {
        name: "Explosive Derailment",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: spree(vec![
            mode(
                cost(&[generic(2)]),
                Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
            ),
            mode(
                cost(&[generic(2)]),
                Effect::Destroy { what: target_filtered(R::Artifact) },
            ),
        ]),
        ..Default::default()
    }
}

/// Insatiable Avarice — {B} Sorcery. Spree: +{2} search your library for a card
/// and put it on top; +{B}{B} target player draws three cards and loses 3 life.
pub fn insatiable_avarice() -> CardDefinition {
    CardDefinition {
        name: "Insatiable Avarice",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: spree(vec![
            mode(
                cost(&[generic(2)]),
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Any,
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                },
            ),
            mode(
                cost(&[b(), b()]),
                Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::Const(3),
                    },
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::Const(3),
                    },
                ]),
            ),
        ]),
        ..Default::default()
    }
}

/// Rustler Rampage — {W} Instant. Spree: +{1} untap all creatures target player
/// controls; +{1} target creature gains double strike until end of turn.
pub fn rustler_rampage() -> CardDefinition {
    CardDefinition {
        name: "Rustler Rampage",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: spree(vec![
            mode(
                cost(&[generic(1)]),
                Effect::Untap {
                    what: Selector::ControlledBy {
                        who: PlayerRef::Target(0),
                        filter: R::Creature,
                    },
                    up_to: None,
                },
            ),
            mode(
                cost(&[generic(1)]),
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            ),
        ]),
        ..Default::default()
    }
}

/// Requisition Raid — {W} Sorcery. Spree: +{1} destroy target artifact; +{1}
/// destroy target enchantment; +{1} put a +1/+1 counter on each creature target
/// player controls.
pub fn requisition_raid() -> CardDefinition {
    CardDefinition {
        name: "Requisition Raid",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: spree(vec![
            mode(cost(&[generic(1)]), Effect::Destroy { what: target_filtered(R::Artifact) }),
            mode(cost(&[generic(1)]), Effect::Destroy { what: target_filtered(R::Enchantment) }),
            mode(
                cost(&[generic(1)]),
                Effect::AddCounter {
                    what: Selector::ControlledBy {
                        who: PlayerRef::Target(0),
                        filter: R::Creature,
                    },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ),
        ]),
        ..Default::default()
    }
}

/// Caught in the Crossfire — {R}{R} Instant. Spree: +{1} deal 2 to each outlaw
/// creature; +{1} deal 2 to each non-outlaw creature.
pub fn caught_in_the_crossfire() -> CardDefinition {
    CardDefinition {
        name: "Caught in the Crossfire",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Instant],
        effect: spree(vec![
            mode(
                cost(&[generic(1)]),
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature.and(R::IsOutlaw)),
                    amount: Value::Const(2),
                },
            ),
            mode(
                cost(&[generic(1)]),
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature.and(R::IsOutlaw.negate())),
                    amount: Value::Const(2),
                },
            ),
        ]),
        ..Default::default()
    }
}

/// Rush of Dread — {1}{B}{B} Sorcery. Spree: +{1} target opponent sacrifices
/// half the creatures they control (rounded up); +{2} discards half their hand;
/// +{2} loses half their life.
pub fn rush_of_dread() -> CardDefinition {
    CardDefinition {
        name: "Rush of Dread",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: spree(vec![
            mode(
                cost(&[generic(1)]),
                Effect::SacrificeHalf {
                    who: Selector::Player(PlayerRef::Target(0)),
                    filter: R::Creature,
                    rounded_up: true,
                },
            ),
            mode(
                cost(&[generic(2)]),
                Effect::DiscardHalf {
                    who: Selector::Player(PlayerRef::Target(0)),
                    rounded_up: true,
                },
            ),
            mode(
                cost(&[generic(2)]),
                Effect::LoseHalfLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    rounded_up: true,
                },
            ),
        ]),
        ..Default::default()
    }
}

/// Phantom Interference — {U} Instant. Spree: +{3} create a 2/2 white Spirit
/// with flying; +{1} counter target spell unless its controller pays {2}.
pub fn phantom_interference() -> CardDefinition {
    CardDefinition {
        name: "Phantom Interference",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: spree(vec![
            mode(
                cost(&[generic(3)]),
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Spirit".into(),
                        power: 2,
                        toughness: 2,
                        keywords: vec![Keyword::Flying],
                        card_types: vec![CardType::Creature],
                        colors: vec![crate::mana::Color::White],
                        subtypes: crate::card::Subtypes {
                            creature_types: vec![crate::card::CreatureType::Spirit],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
            ),
            mode(
                cost(&[generic(1)]),
                Effect::CounterUnlessPaid {
                    what: target_filtered(R::IsSpellOnStack),
                    mana_cost: cost(&[generic(2)]),
                    exile: false,
                    extra_generic: None,
                },
            ),
        ]),
        ..Default::default()
    }
}

/// Three Steps Ahead — {U} Instant. Spree: +{1}{U} counter target spell; +{3}
/// create a token copy of target artifact or creature you control; +{2} draw
/// two cards, then discard a card.
pub fn three_steps_ahead() -> CardDefinition {
    CardDefinition {
        name: "Three Steps Ahead",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: spree(vec![
            mode(
                cost(&[generic(1), u()]),
                Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            ),
            mode(
                cost(&[generic(3)]),
                Effect::CreateTokenCopyOf {
                    extra_keywords: vec![],
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(
                        R::Artifact.or(R::Creature).and(R::ControlledByYou),
                    ),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                },
            ),
            mode(
                cost(&[generic(2)]),
                Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
            ),
        ]),
        ..Default::default()
    }
}

/// Dance of the Tumbleweeds — {1}{G} Sorcery. Spree: +{1} search your library
/// for a basic land or Desert card and put it onto the battlefield; +{3} create
/// an X/X green Elemental, where X is the number of lands you control.
pub fn dance_of_the_tumbleweeds() -> CardDefinition {
    let lands = Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByYou)));
    CardDefinition {
        name: "Dance of the Tumbleweeds",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: spree(vec![
            mode(
                cost(&[generic(1)]),
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand.or(R::HasLandType(LandType::Desert)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ),
            mode(
                cost(&[generic(3)]),
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Elemental".into(),
                        power: 0,
                        toughness: 0,
                        card_types: vec![CardType::Creature],
                        colors: vec![crate::mana::Color::Green],
                        subtypes: crate::card::Subtypes {
                            creature_types: vec![crate::card::CreatureType::Elemental],
                            ..Default::default()
                        },
                        dynamic_pt: Some((lands.clone(), lands)),
                        ..Default::default()
                    },
                },
            ),
        ]),
        ..Default::default()
    }
}
