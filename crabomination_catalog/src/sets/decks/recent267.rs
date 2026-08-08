//! Gap batch — a sac-for-draw Goblin, a Powerstone ramp body, two combat
//! tricks (grant death-return / team pump + death tokens), a protection trick,
//! and an exile-until-leaves Samurai with Plainscycling. All on existing
//! primitives. Tests in `tests/recent_b/recent267.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, ExileReturnZone, Keyword, LandType, SelectionRequirement as R, Subtypes,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, w};

/// Akki Scrapchomper — {R} 1/1 Phyrexian Goblin. {1}{R}, {T}, Sacrifice an
/// artifact or land: Draw a card.
pub fn akki_scrapchomper() -> CardDefinition {
    CardDefinition {
        name: "Akki Scrapchomper",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Goblin],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            tap_cost: true,
            sac_other_filter: Some((R::Artifact.or(R::Land), 1)),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Argothian Opportunist — {2}{G} 3/2 Human Scout. ETB: create a tapped
/// Powerstone token.
pub fn argothian_opportunist() -> CardDefinition {
    CardDefinition {
        name: "Argothian Opportunist",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(crate::game::effects::powerstone_token()),
            },
            Effect::Tap {
                what: Selector::LastCreatedToken,
            },
        ]))],
        ..Default::default()
    }
}

/// Ashnod's Intervention — {B} Instant. Until end of turn, target creature gets
/// +2/+0 and gains "When this creature dies, return it to its owner's hand."
/// (The "or is exiled from the battlefield" trigger clause is approximated by
/// the death half.)
pub fn ashnods_intervention() -> CardDefinition {
    CardDefinition {
        name: "Ashnod's Intervention",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                    effect: Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    },
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Gnawing Crescendo — {2}{R} Instant. Creatures you control get +2/+0 until
/// end of turn. Whenever a nontoken creature you control dies this turn, create
/// a 1/1 black Rat creature token with "This token can't block."
pub fn gnawing_crescendo() -> CardDefinition {
    let rat = TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    };
    CardDefinition {
        name: "Gnawing Crescendo",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Creature,
                },
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::CreaturesYouControlDyingThisTurn {
                body: Box::new(Effect::If {
                    cond: crate::card::Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::NotToken,
                    },
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Box::new(rat),
                    }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Angelic Intervention — {1}{W} Instant. Target creature you control gains
/// protection from the color of your choice until end of turn and gets a +1/+1
/// counter. (The "or planeswalker" / "or colorless" riders are approximated.)
pub fn angelic_intervention() -> CardDefinition {
    CardDefinition {
        name: "Angelic Intervention",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantProtectionFromChosenColor {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                duration: Duration::EndOfTurn,
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Alabaster Host Intercessor — {5}{W} 3/4 Phyrexian Samurai. ETB: exile
/// target creature an opponent controls until this leaves. Plainscycling {2}.
pub fn alabaster_host_intercessor() -> CardDefinition {
    CardDefinition {
        name: "Alabaster Host Intercessor",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Samurai],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Plains)],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}
