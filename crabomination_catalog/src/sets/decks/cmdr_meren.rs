//! Commander: the cards the **Plunder the Graves** precon (C15, Meren of Clan
//! Nel Toth) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_meren.rs`.
//!
//! Residuals (each also on its card):
//! - **Wretched Confluence** — the mode picks are the card's default (a card
//!   for you, -2/-2 on a creature, a creature card back); `ChooseN` has no
//!   cast-time mode choice.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{devour, myriad, on_attack, target_filtered, unearth};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic};
use crate::sets::tap_add_colorless;
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

fn creature_cards_in_graveyard(who: PlayerRef) -> Value {
    Value::CountOf(Box::new(Selector::CardsInZone { who, zone: Zone::Graveyard, filter: R::Creature }))
}

fn token(name: &str, color: Color, t: CreatureType, p: i32, tough: i32, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: tough,
        keywords,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: vec![t], ..Default::default() },
        ..Default::default()
    })
}

/// Banshee of the Dread Choir — myriad; connecting, that player discards.
pub fn banshee_of_the_dread_choir() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            myriad(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Discard {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::Const(1),
                    random: false,
                },
            },
        ],
        ..creature(
            "Banshee of the Dread Choir",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Blood Bairn — sacrifice another creature: +2/+2 until end of turn.
pub fn blood_bairn() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Blood Bairn", cost(&[generic(2), b()]), vec![CreatureType::Vampire], 2, 2)
    }
}

/// Bloodspore Thrinax — devour 1; your other creatures enter with as many
/// extra +1/+1 counters as it has.
pub fn bloodspore_thrinax() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(devour(1)),
        static_abilities: vec![StaticAbility {
            description: "Each other creature you control enters with an additional X +1/+1 counters.",
            effect: StaticEffect::OtherCreaturesEnterWithCountersEqualToSourceCounters {
                kind: CounterType::PlusOnePlusOne,
            },
        }],
        ..creature("Bloodspore Thrinax", cost(&[generic(2), g(), g()]), vec![CreatureType::Lizard], 2, 2)
    }
}

/// Centaur Vinecrasher — trample; enters with a counter per land card in all
/// graveyards; from your graveyard, {G}{G} brings it back to hand when a land
/// card hits a graveyard.
pub fn centaur_vinecrasher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CountOf(Box::new(Selector::CardsInZone {
                who: PlayerRef::EachPlayer,
                zone: Zone::Graveyard,
                filter: R::Land,
            })),
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::FromYourGraveyardAnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::MayPay {
                description: "Pay {G}{G} to return Centaur Vinecrasher to your hand?".into(),
                mana_cost: cost(&[g(), g()]),
                body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) }),
                else_: None,
            },
        }],
        ..creature(
            "Centaur Vinecrasher",
            cost(&[generic(3), g()]),
            vec![CreatureType::Plant, CreatureType::Centaur],
            1,
            1,
        )
    }
}

/// Cloudthresher — flash, reach; on entry 2 damage to each flier and each
/// player; evoke {2}{G}{G}.
pub fn cloudthresher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Reach],
        alternative_cost: Some(crate::effect::shortcut::evoke(cost(&[generic(2), g(), g()]))),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                amount: Value::Const(2),
            },
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) },
        ]))],
        ..creature(
            "Cloudthresher",
            cost(&[generic(2), g(), g(), g(), g()]),
            vec![CreatureType::Elemental],
            7,
            7,
        )
    }
}

/// Corpse Augur — dying, you draw and lose X, X the creature cards in target
/// player's graveyard.
pub fn corpse_augur() -> CardDefinition {
    let x = || creature_cards_in_graveyard(PlayerRef::Target(0));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: x() },
                    Effect::LoseLife { who: Selector::You, amount: x() },
                ])),
            },
        }],
        ..creature(
            "Corpse Augur",
            cost(&[generic(3), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            4,
            2,
        )
    }
}

/// Extractor Demon — flying; another creature leaving the battlefield may
/// mill target player two; unearth {2}{B}.
pub fn extractor_demon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                },
            ),
            effect: Effect::MayDo {
                description: "Have target player mill two cards?".into(),
                body: Box::new(Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(2) }),
            },
        }],
        activated_abilities: vec![unearth(cost(&[generic(2), b()]))],
        ..creature("Extractor Demon", cost(&[generic(4), b(), b()]), vec![CreatureType::Demon], 5, 5)
    }
}

/// Grim Backwoods — taps for {C}; {2}{B}{G}, {T}, sacrifice a creature: draw.
pub fn grim_backwoods() -> CardDefinition {
    CardDefinition {
        name: "Grim Backwoods",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b(), g()]),
                tap_cost: true,
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Kessig Cagebreakers — attacking, a tapped and attacking 2/2 Wolf per
/// creature card in your graveyard.
pub fn kessig_cagebreakers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::CreateTokenAttacking {
            who: PlayerRef::You,
            count: creature_cards_in_graveyard(PlayerRef::You),
            definition: token("Wolf", Color::Green, CreatureType::Wolf, 2, 2, vec![]),
            cleanup: Default::default(),
            defender: None,
        })],
        ..creature(
            "Kessig Cagebreakers",
            cost(&[generic(4), g()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            4,
        )
    }
}

/// Mazirek, Kraul Death Priest — flying; whenever a player sacrifices another
/// permanent, a +1/+1 counter on each creature you control.
pub fn mazirek_kraul_death_priest() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource },
            ),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..creature(
            "Mazirek, Kraul Death Priest",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Insect, CreatureType::Shaman],
            2,
            2,
        )
    }
}

/// Mycoloth — devour 2; each upkeep, a Saproling per +1/+1 counter on it.
pub fn mycoloth() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(devour(2)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                definition: token("Saproling", Color::Green, CreatureType::Saproling, 1, 1, vec![]),
            },
        }],
        ..creature("Mycoloth", cost(&[generic(3), g(), g()]), vec![CreatureType::Fungus], 4, 4)
    }
}

/// Sever the Bloodline — exile target creature and every creature sharing its
/// name; flashback {5}{B}{B}.
pub fn sever_the_bloodline() -> CardDefinition {
    CardDefinition {
        name: "Sever the Bloodline",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(5), b(), b()]))],
        effect: Effect::Exile {
            what: Selector::SharingNameWith(Box::new(Selector::TargetFiltered { slot: 0, filter: R::Creature })),
        },
        ..Default::default()
    }
}

/// Spider Spawning — a 1/2 reach Spider per creature card in your graveyard;
/// flashback {6}{B}.
pub fn spider_spawning() -> CardDefinition {
    CardDefinition {
        name: "Spider Spawning",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(6), b()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: creature_cards_in_graveyard(PlayerRef::You),
            definition: token("Spider", Color::Green, CreatureType::Spider, 1, 2, vec![Keyword::Reach]),
        },
        ..Default::default()
    }
}

/// Thief of Blood — flying; as it enters, it takes every counter on every
/// permanent as +1/+1 counters.
pub fn thief_of_blood() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        as_enters_effect: Some(Effect::Seq(vec![
            Effect::RemoveAllCounters { what: Selector::EachPermanent(R::Any) },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersRemovedThisEffect,
            },
        ])),
        ..creature("Thief of Blood", cost(&[generic(4), b(), b()]), vec![CreatureType::Vampire], 1, 1)
    }
}

/// Tribute to the Wild — each opponent sacrifices an artifact or enchantment.
pub fn tribute_to_the_wild() -> CardDefinition {
    CardDefinition {
        name: "Tribute to the Wild",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: R::Artifact.or(R::Enchantment),
        },
        ..Default::default()
    }
}

/// Vivid Marsh — enters tapped with two charge counters; {B}, or a counter for
/// any color.
pub fn vivid_marsh() -> CardDefinition {
    super::cmdr_aesi::vivid("Vivid Marsh", Color::Black)
}

/// Wretched Confluence — choose three, repeats allowed (CR 700.2d). Default
/// picks: draw-and-lose for you, -2/-2 on a creature, a creature card back.
pub fn wretched_confluence() -> CardDefinition {
    CardDefinition {
        name: "Wretched Confluence",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![0, 1, 2],
            modes: vec![
                Effect::Seq(vec![
                    Effect::Draw { who: target_filtered(R::Player), amount: Value::Const(1) },
                    Effect::LoseLife { who: Selector::Target(0), amount: Value::Const(1) },
                ]),
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                },
                Effect::Move {
                    what: target_filtered(R::Creature.from_your_graveyard()),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ],
        },
        ..Default::default()
    }
}
