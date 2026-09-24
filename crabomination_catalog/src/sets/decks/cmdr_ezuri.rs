//! Commander: the cards the **Swell the Host** precon (C15, Ezuri, Claw of
//! Progress) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_ezuri.rs`.
//!
//! Residuals (each also on its card):
//! - **Verdant Confluence** — the mode picks are the card's default (two
//!   counters, two basics); `Effect::ChooseN` has no cast-time mode choice.
//! - **Skullwinder** — "choose an opponent" is the engine's pick (fewest
//!   creatures, as for Sylvan Offering).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, etb_draw, graft, myriad, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, cost, g, generic, hybrid, u};
use crate::sets::{enters_tapped, tap_add, tap_add_colorless};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn may_draw_on_connect() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::MayDo {
            description: "Draw a card?".into(),
            body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
        },
    }
}

fn eot_pump(what: Selector, n: Value) -> Effect {
    Effect::PumpPT { what, power: n.clone(), toughness: n, duration: Duration::EndOfTurn }
}

/// Broodbirth Viper — myriad; may draw on combat damage to a player.
pub fn broodbirth_viper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![myriad(), may_draw_on_connect()],
        ..creature("Broodbirth Viper", cost(&[generic(4), u()]), vec![CreatureType::Snake], 3, 3)
    }
}

/// Caller of the Pack — 8/6 trample, myriad.
pub fn caller_of_the_pack() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![myriad()],
        ..creature("Caller of the Pack", cost(&[generic(5), g(), g()]), vec![CreatureType::Beast], 8, 6)
    }
}

/// Chameleon Colossus — changeling, protection from black; {2}{G}{G}: +X/+X
/// until end of turn, X its power as the ability resolves.
pub fn chameleon_colossus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling, Keyword::Protection(Color::Black)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), g()]),
            effect: eot_pump(Selector::This, Value::PowerOf(Box::new(Selector::This))),
            ..Default::default()
        }],
        ..creature(
            "Chameleon Colossus",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Shapeshifter],
            4,
            4,
        )
    }
}

/// Great Oak Guardian — flash, reach; on entry, creatures target player
/// controls get +2/+2 until end of turn and untap.
pub fn great_oak_guardian() -> CardDefinition {
    let theirs = || Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature };
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Reach],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            eot_pump(theirs(), Value::Const(2)),
            Effect::Untap { what: theirs(), up_to: None },
        ]))],
        ..creature("Great Oak Guardian", cost(&[generic(5), g()]), vec![CreatureType::Treefolk], 4, 5)
    }
}

/// Illusory Ambusher — flash; whenever it's dealt damage, draw that many.
pub fn illusory_ambusher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..creature(
            "Illusory Ambusher",
            cost(&[generic(4), u()]),
            vec![CreatureType::Cat, CreatureType::Illusion],
            4,
            1,
        )
    }
}

/// Kaseto, Orochi Archmage — {G}{U}: target creature can't be blocked this
/// turn; a Snake also gets +2/+2.
pub fn kaseto_orochi_archmage() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), u()]),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::HasCreatureType(CreatureType::Snake),
                    },
                    then: Box::new(eot_pump(Selector::Target(0), Value::Const(2))),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Kaseto, Orochi Archmage",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Snake, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Llanowar Reborn — enters tapped, taps for {G}; graft 1 (CR 702.58).
pub fn llanowar_reborn() -> CardDefinition {
    CardDefinition {
        name: "Llanowar Reborn",
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![tap_add(Color::Green)],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        triggered_abilities: vec![graft()],
        ..Default::default()
    }
}

/// Lorescale Coatl — whenever you draw a card, a +1/+1 counter on it.
pub fn lorescale_coatl() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..creature("Lorescale Coatl", cost(&[generic(1), g(), u()]), vec![CreatureType::Snake], 2, 2)
    }
}

/// Mirror Match — cast only during the declare blockers step; a blocking
/// token copy of each creature attacking you or your planeswalkers, exiled
/// at end of combat.
pub fn mirror_match() -> CardDefinition {
    CardDefinition {
        name: "Mirror Match",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Instant],
        cast_condition: Some(Predicate::CurrentStepIs(TurnStep::DeclareBlockers)),
        effect: Effect::CopyAttackersAsBlockers,
        ..Default::default()
    }
}

/// Ohran Viper — destroys a creature it dealt combat damage at end of
/// combat; may draw on combat damage to a player.
pub fn ohran_viper() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
                effect: Effect::AtEndOfCombat {
                    body: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                },
            },
            may_draw_on_connect(),
        ],
        ..creature("Ohran Viper", cost(&[generic(1), g(), g()]), vec![CreatureType::Snake], 1, 3)
    }
}

/// Skullwinder — deathtouch; on entry return target card from your graveyard
/// to hand, then a chosen opponent returns one of theirs.
pub fn skullwinder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Player.negate().from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::ChooseOpponentThen {
                then: Box::new(Effect::AsPlayer {
                    who: PlayerRef::ChosenPlayerOfSource,
                    body: Box::new(Effect::ReturnGraveyardCardsToHand {
                        filter: R::Any,
                        max: Value::Const(1),
                    }),
                }),
            },
        ]))],
        ..creature("Skullwinder", cost(&[generic(2), g()]), vec![CreatureType::Snake], 1, 3)
    }
}

/// Stingerfling Spider — reach; on entry may destroy target creature with
/// flying.
pub fn stingerfling_spider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Destroy target creature with flying?".into(),
            body: Box::new(Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            }),
        })],
        ..creature("Stingerfling Spider", cost(&[generic(4), g()]), vec![CreatureType::Spider], 2, 5)
    }
}

/// Synthetic Destiny — exile all creatures you control; at the next end
/// step, reveal until that many creature cards, put them all onto the
/// battlefield and shuffle the rest in.
pub fn synthetic_destiny() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Synthetic Destiny",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::WithX {
            x: Value::CountOf(Box::new(yours())),
            body: Box::new(Effect::Seq(vec![
                Effect::Exile { what: yours() },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::RevealUntilMatchingToBattlefield {
                        filter: R::Creature,
                        count: Value::XFromCost,
                        rest_bottom: false,
                    }),
                },
            ])),
        },
        ..Default::default()
    }
}

/// Thelonite Hermit — all Saprolings get +1/+1; morph {3}{G}{G}; turned face
/// up, four 1/1 green Saprolings.
pub fn thelonite_hermit() -> CardDefinition {
    let saproling = Arc::new(TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), g(), g()]))],
        static_abilities: vec![StaticAbility {
            description: "All Saprolings get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Saproling)),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(4),
                definition: saproling,
            },
        }],
        ..creature(
            "Thelonite Hermit",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Shaman],
            1,
            1,
        )
    }
}

/// Verdant Confluence — choose three, repeats allowed (CR 700.2d). The
/// default picks are two basics and two counters on a creature; the
/// graveyard-return mode is there for a mode-picking seat.
pub fn verdant_confluence() -> CardDefinition {
    CardDefinition {
        name: "Verdant Confluence",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![0, 2, 2],
            modes: vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                Effect::Move {
                    what: target_filtered(R::PermanentCard.from_your_graveyard()),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
            ],
        },
        ..Default::default()
    }
}

/// Wistful Selkie — {G/U}{G/U}{G/U} 2/2; draws a card on entry.
pub fn wistful_selkie() -> CardDefinition {
    let gu = || hybrid(Color::Green, Color::Blue);
    CardDefinition {
        triggered_abilities: vec![etb_draw(1)],
        ..creature(
            "Wistful Selkie",
            cost(&[gu(), gu(), gu()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Zoetic Cavern — taps for {C}; morph {2}. Face down it is a 2/2 creature,
/// turned up a land again (CR 708.2, 702.37).
pub fn zoetic_cavern() -> CardDefinition {
    CardDefinition {
        name: "Zoetic Cavern",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::Morph(cost(&[generic(2)]))],
        activated_abilities: vec![tap_add_colorless()],
        ..Default::default()
    }
}
