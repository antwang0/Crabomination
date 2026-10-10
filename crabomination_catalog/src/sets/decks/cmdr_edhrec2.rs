//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat, second file (`cmdr_edhrec` is the
//! first). Henzie "Toolbox" Torre, Frodo, Adventurous Hobbit + Sam, Loyal
//! Attendant, K'rrik, Son of Yawgmoth, Voja, Jaws of the Conclave and Choco, Seeker of
//! Paradise. Tests in `tests/recent_b/cmdr_edhrec2.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, MayPlayDuration, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, on_dies};
use crate::effect::{Effect, LookPick, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, mono_hybrid, u, w, Color};
use crabomination_base::tokens::{food_token, treasure_token};
use std::sync::Arc;

fn creature_types(types: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: types, ..Default::default() }
}

fn legend(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(types),
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn treasure() -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(treasure_token()) }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

/// Ancient Brass Dragon — {5}{B}{B} 7/6 Elder Dragon, flying. Combat damage
/// to a player rolls a d20; when you do (CR 603.7), any number of target
/// creature cards with total mana value X or less from graveyards enter
/// under your control, X the result. The budget is a CR 601.2c target
/// restriction across the slots, with X bound to the roll (`WithX`).
pub fn ancient_brass_dragon() -> CardDefinition {
    let reanimate = Effect::ApplyToTargets {
        max_targets: 8,
        min_targets: 0,
        filter: R::Creature.from_any_graveyard().and(R::SlotsTotalManaValueAtMostX),
        effect: Box::new(Effect::Move {
            what: Selector::Target(0),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        }),
    };
    CardDefinition {
        name: "Ancient Brass Dragon",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Elder, CreatureType::Dragon]),
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::RollDie {
                sides: 20,
                count: Value::ONE,
                modifier: Value::Const(0),
                reroll_at_most: 0,
                results: vec![(
                    1,
                    20,
                    Effect::WithX {
                        x: Value::LastDieRoll,
                        body: Box::new(Effect::ReflexiveTrigger { body: Box::new(reanimate) }),
                    },
                )],
                ignore_lowest: 0,
                on_doubles: None,
            },
        }],
        ..Default::default()
    }
}

/// Ojer Kaslem, Deepest Growth // Temple of Cultivation — {3}{G}{G} 6/5
/// legendary God, trample. Combat damage to a player reveals that many cards;
/// you may put a creature card and/or a land card from among them onto the
/// battlefield (one of each at most), the rest to the bottom in a random
/// order. Dies: returns transformed and tapped. The Temple taps for {G} and
/// transforms back for {2}{G}, {T} at sorcery speed with ten permanents.
pub fn ojer_kaslem_deepest_growth() -> CardDefinition {
    let temple = CardDefinition {
        name: "Temple of Cultivation",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Green),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), g()]),
                sorcery_speed: true,
                condition: Some(Predicate::ValueAtLeast(
                    Value::count(Selector::EachPermanent(R::ControlledByYou)),
                    Value::Const(10),
                )),
                effect: Effect::Transform { what: Selector::This },
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::LookPickToHand(Box::new(LookPick {
                    count: Value::TriggerEventAmount,
                    pick_filter: Some(R::Creature.or(R::Land)),
                    take: Some(Value::Const(2)),
                    one_each: vec![R::Creature, R::Land],
                    to_battlefield: true,
                    optional: true,
                    rest_bottom_random: true,
                    ..Default::default()
                })),
            },
            on_dies(Effect::ReturnSelfTransformedTappedToOwner),
        ],
        back_face: Some(Box::new(temple)),
        ..legend("Ojer Kaslem, Deepest Growth", cost(&[generic(3), g(), g()]), vec![CreatureType::God], 6, 5)
    }
}

/// Primeval Herald — {3}{G} 3/1 Elf Scout, trample. Enters or attacks: you
/// may search for a basic land card, put it onto the battlefield tapped.
pub fn primeval_herald() -> CardDefinition {
    let fetch = || Effect::MayDo {
        description: "Search for a basic land card?".into(),
        body: Box::new(Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        }),
    };
    CardDefinition {
        name: "Primeval Herald",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Elf, CreatureType::Scout]),
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(fetch()), on_attack(fetch())],
        ..Default::default()
    }
}

/// Seedguide Ash — {4}{G} 4/4 Treefolk Druid. Dies: you may search for up to
/// three Forest cards (any with the type), put them onto the battlefield tapped.
pub fn seedguide_ash() -> CardDefinition {
    CardDefinition {
        name: "Seedguide Ash",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Treefolk, CreatureType::Druid]),
        power: 4,
        toughness: 4,
        triggered_abilities: vec![on_dies(Effect::MayDo {
            description: "Search for up to three Forest cards?".into(),
            body: Box::new(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::Land.and(R::HasLandType(LandType::Forest)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::Const(3),
            }),
        })],
        ..Default::default()
    }
}

/// Birthing Ritual — {1}{G} Enchantment. Your end step, if you control a
/// creature: look at the top seven; you may sacrifice a creature, and if you
/// do you may put a creature card with mana value at most 1 + the sacrificed
/// creature's from among them onto the battlefield. The rest go to the bottom
/// in a random order either way.
pub fn birthing_ritual() -> CardDefinition {
    let seven = || Value::Const(7);
    CardDefinition {
        name: "Birthing Ritual",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Creature,
                })),
            effect: Effect::Seq(vec![
                Effect::LookAtTop { who: PlayerRef::You, amount: seven() },
                Effect::MaySacrifice {
                    description: "Sacrifice a creature to put a creature card from among them onto the battlefield?"
                        .into(),
                    filter: R::Creature,
                    count: Value::ONE,
                    then: Box::new(Effect::LookPickToHand(Box::new(LookPick {
                        count: seven(),
                        pick_filter: Some(R::Creature.and(R::ManaValueAtMostSacrificedPlus(1))),
                        to_battlefield: true,
                        optional: true,
                        rest_bottom_random: true,
                        ..Default::default()
                    }))),
                    // Declined: nothing is put onto the battlefield, but the
                    // seven still go to the bottom.
                    else_: Some(Box::new(Effect::LookPickToHand(Box::new(LookPick {
                        count: seven(),
                        pick_filter: Some(R::Not(Box::new(R::Any))),
                        optional: true,
                        rest_bottom_random: true,
                        ..Default::default()
                    })))),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Bag End Banquet — {6} Artifact. Enters with three Food tokens beside it;
/// {T}: add {C} for each Food you control.
pub fn bag_end_banquet() -> CardDefinition {
    CardDefinition {
        name: "Bag End Banquet",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: Arc::new(food_token()),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::count(Selector::EachPermanent(
                    R::ControlledByYou.and(R::HasArtifactSubtype(ArtifactSubtype::Food)),
                ))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Belladonna Took — {1}{W} 2/2 legendary Halfling Citizen. Whenever a token
/// you control enters: the first resolution this turn gains 1 life, the
/// second draws a card, the third puts a +1/+1 counter on each creature you
/// control; later ones do nothing.
pub fn belladonna_took() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(trigger_is(R::IsToken)),
            effect: Effect::NthResolutionThisTurn {
                branches: vec![
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::AddCounter {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                ],
            },
        }],
        ..legend("Belladonna Took", cost(&[generic(1), w()]), vec![CreatureType::Halfling, CreatureType::Citizen], 2, 2)
    }
}

/// Bilbo, Fellow Conspirator — {2}{G} 2/3 legendary Halfling Citizen. If you
/// would create a Food token, instead create a Food token and a Treasure
/// token (CR 614.1a; the Treasure isn't re-replaced, CR 614.5).
pub fn bilbo_fellow_conspirator() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would create a Food token, instead create a Food token and a Treasure token."
                .into(),
            effect: StaticEffect::TokenNamedAlsoMints { name: "Food".into(), also: treasure_token() },
        }],
        ..legend(
            "Bilbo, Fellow Conspirator",
            cost(&[generic(2), g()]),
            vec![CreatureType::Halfling, CreatureType::Citizen],
            2,
            3,
        )
    }
}

/// The Sackville-Bagginses — {1}{B} 2/2 legendary Halfling Citizen. Enters:
/// you may sacrifice another creature or artifact; if you do, draw a card and
/// create a Treasure. Whenever you sacrifice a token, target opponent loses 1
/// life.
pub fn the_sackville_bagginses() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::MaySacrifice {
                description: "Sacrifice another creature or artifact to draw a card and create a Treasure?".into(),
                filter: R::Creature.or(R::Artifact).and(R::OtherThanSource),
                count: Value::ONE,
                then: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    treasure(),
                ])),
                else_: None,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(trigger_is(R::IsToken)),
                effect: Effect::LoseLife {
                    who: crate::effect::shortcut::target_filtered(R::OpponentPlayer),
                    amount: Value::ONE,
                },
            },
        ],
        ..legend(
            "The Sackville-Bagginses",
            cost(&[generic(1), b()]),
            vec![CreatureType::Halfling, CreatureType::Citizen],
            2,
            2,
        )
    }
}

/// Asmodeus the Archfiend — {4}{B}{B} 6/6 legendary Devil God. Binding
/// Contract: your draws exile the top card face down instead, with Asmodeus
/// (CR 121.2a). {B}{B}{B}: draw seven. {B}: return every card exiled with
/// Asmodeus to its owner's hand, then lose that much life.
pub fn asmodeus_the_archfiend() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card, exile the top card of your library face down instead.".into(),
            effect: StaticEffect::ReplaceDrawWithExileFaceDownWithSource,
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b(), b(), b()]),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(7) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::Seq(vec![
                    Effect::Move { what: Selector::CardExiledWithSource, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                    Effect::LoseLife { who: Selector::You, amount: Value::count(Selector::LastMoved) },
                ]),
                ..Default::default()
            },
        ],
        ..legend("Asmodeus the Archfiend", cost(&[generic(4), b(), b()]), vec![CreatureType::Devil, CreatureType::God], 6, 6)
    }
}

/// Hoarding Broodlord — {5}{B}{B}{B} 7/6 Dragon, convoke, flying. Enters:
/// search for any card, exile it face down; you may play it for as long as it
/// stays exiled. Spells you cast from exile have convoke.
pub fn hoarding_broodlord() -> CardDefinition {
    CardDefinition {
        name: "Hoarding Broodlord",
        cost: cost(&[generic(5), b(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Dragon]),
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Convoke, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Spells you cast from exile have convoke.".into(),
            effect: StaticEffect::ExileCastSpellsHaveConvoke { filter: R::Any },
        }],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExileFaceDown {
                body: Box::new(Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Exile }),
            },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: MayPlayDuration::WhileExiled,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        ]))],
        ..Default::default()
    }
}

/// Razaketh, the Foulblooded — {5}{B}{B}{B} 8/8 legendary Demon, flying,
/// trample. Pay 2 life, sacrifice another creature: search for a card, put it
/// into your hand.
pub fn razaketh_the_foulblooded() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 2,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..legend("Razaketh, the Foulblooded", cost(&[generic(5), b(), b(), b()]), vec![CreatureType::Demon], 8, 8)
    }
}

/// Beseech the Queen — {2/B}{2/B}{2/B} Sorcery (mana value 6). Search for a
/// card with mana value at most the number of lands you control.
pub fn beseech_the_queen() -> CardDefinition {
    CardDefinition {
        name: "Beseech the Queen",
        cost: cost(&[mono_hybrid(2, Color::Black), mono_hybrid(2, Color::Black), mono_hybrid(2, Color::Black)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::WithX {
            x: Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByYou))),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::ManaValueAtMostXFromCost,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
        ..Default::default()
    }
}

/// Dark Petition — {3}{B}{B} Sorcery. Search for a card, put it into your
/// hand. Spell mastery (CR 207.2c): with two or more instant and/or sorcery
/// cards in your graveyard, add {B}{B}{B}.
pub fn dark_petition() -> CardDefinition {
    CardDefinition {
        name: "Dark Petition",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Hand(PlayerRef::You) },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    },
                    Value::Const(2),
                ),
                then: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Black, Color::Black, Color::Black]),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

fn wolf_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Wolf".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: creature_types(vec![CreatureType::Wolf]),
        ..Default::default()
    }
}

fn wolves(count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(wolf_token()) }
}

fn wolf_or_werewolf() -> R {
    R::HasCreatureType(CreatureType::Wolf).or(R::HasCreatureType(CreatureType::Werewolf))
}

/// Bramblewood Paragon — {1}{G} 2/2 Elf Warrior. Each other Warrior creature
/// you control enters with an additional +1/+1 counter; each creature you
/// control with a +1/+1 counter has trample.
pub fn bramblewood_paragon() -> CardDefinition {
    CardDefinition {
        name: "Bramblewood Paragon",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Elf, CreatureType::Warrior]),
        power: 2,
        toughness: 2,
        static_abilities: vec![
            StaticAbility {
                description: "Each other Warrior creature you control enters with an additional +1/+1 counter on it."
                    .into(),
                effect: StaticEffect::MatchingEntersWithExtraCounters {
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Warrior)).and(R::OtherThanSource),
                    kind: CounterType::PlusOnePlusOne,
                    amount: 1,
                },
            },
            StaticAbility {
                description: "Each creature you control with a +1/+1 counter on it has trample.".into(),
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::WithCounter(CounterType::PlusOnePlusOne)),
                    ),
                    keyword: Keyword::Trample,
                },
            },
        ],
        ..Default::default()
    }
}

/// Cemetery Prowler — {1}{G}{G} 3/4 Wolf, vigilance. Enters or attacks: exile
/// a card from a graveyard (your choice), linked to it. Your spells cost {1}
/// less for each card type they share with cards exiled with it.
pub fn cemetery_prowler() -> CardDefinition {
    let exile_one = || Effect::ChooseOneAmong {
        what: Selector::CardsInZone { who: PlayerRef::EachPlayer, zone: crate::card::Zone::Graveyard, filter: R::Any },
        chooser: PlayerRef::You,
        chosen: Box::new(Effect::Move {
            what: Selector::SeparatedPile { chosen: true },
            to: ZoneDest::ExileWithSourceStamp,
        }),
        other: Box::new(Effect::Noop),
    };
    CardDefinition {
        name: "Cemetery Prowler",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Wolf]),
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Spells you cast cost {1} less to cast for each card type they share with cards exiled with this creature.".into(),
            effect: StaticEffect::CostReductionPerTypeSharedWithExiled,
        }],
        triggered_abilities: vec![etb(exile_one()), on_attack(exile_one())],
        ..Default::default()
    }
}

/// Druid of the Anima — {1}{G} 1/1 Elf Druid. {T}: add {R}, {G}, or {W}.
pub fn druid_of_the_anima() -> CardDefinition {
    CardDefinition {
        name: "Druid of the Anima",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Elf, CreatureType::Druid]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Red, Color::Green, Color::White], Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hollowhenge Overlord — {4}{G}{G} 4/4 Wolf, flash. Your upkeep: a 2/2 Wolf
/// for each Wolf or Werewolf creature you control.
pub fn hollowhenge_overlord() -> CardDefinition {
    CardDefinition {
        name: "Hollowhenge Overlord",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Wolf]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: wolves(Value::count(Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(wolf_or_werewolf()),
            ))),
        }],
        ..Default::default()
    }
}

/// Wolf-Skull Shaman — {1}{G} 2/2 Elf Shaman. Kinship: your upkeep, look at
/// the top card; if it shares a creature type with this creature you may
/// reveal it, and if you do, create a 2/2 Wolf.
pub fn wolf_skull_shaman() -> CardDefinition {
    CardDefinition {
        name: "Wolf-Skull Shaman",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Elf, CreatureType::Shaman]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                    filter: R::SharesCreatureTypeWithSource,
                },
                then: Box::new(Effect::MayDo {
                    description: "Reveal the top card to create a 2/2 Wolf?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RevealTopOfLibrary { who: PlayerRef::You },
                        wolves(Value::ONE),
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Howling Moon — {2}{G} Enchantment. Beginning of combat on your turn:
/// target Wolf or Werewolf you control gets +2/+2. Whenever an opponent casts
/// their second spell each turn, create a 2/2 Wolf.
pub fn howling_moon() -> CardDefinition {
    CardDefinition {
        name: "Howling Moon",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: crate::effect::shortcut::target_filtered(
                        R::Creature.and(R::ControlledByYou).and(wolf_or_werewolf()),
                    ),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: crate::effect::Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                    Predicate::SpellsCastThisTurnEquals { who: PlayerRef::Triggerer, count: Value::Const(2) },
                ),
                effect: wolves(Value::ONE),
            },
        ],
        ..Default::default()
    }
}

fn bird() -> R {
    R::Creature.and(R::HasCreatureType(CreatureType::Bird))
}

/// Chocobo Camp — Land. Enters tapped unless you control a legendary
/// creature. {T}: add {G}; when you next cast a Bird creature spell this
/// turn, it enters with an additional +1/+1 counter (CR 603.7e). {2}{G}{G},
/// {T}: a 2/2 green Bird token with "whenever a land you control enters, this
/// token gets +1/+0 until end of turn."
pub fn chocobo_camp() -> CardDefinition {
    let chocobo = crate::card::TokenDefinition {
        name: "Bird".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: creature_types(vec![CreatureType::Bird]),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(trigger_is(R::Land)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: crate::effect::Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Chocobo Camp",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless you control a legendary creature.".into(),
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou),
                )),
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green]) },
                    Effect::OnYourNextSpellMatchingThisTurn {
                        filter: bird(),
                        body: Box::new(Effect::SpellEntersWithCounters {
                            what: Selector::TriggerSource,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        }),
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), g(), g()]),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(chocobo) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Gwaihir the Windlord — {4}{W}{U} 4/4 legendary Bird Noble, flying,
/// vigilance. Costs {2} less with two or more cards drawn this turn; other
/// Birds you control have vigilance.
pub fn gwaihir_the_windlord() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![
            StaticAbility {
                description: "This spell costs {2} less to cast as long as you've drawn two or more cards this turn."
                    .into(),
                effect: StaticEffect::SelfCostReducedIf {
                    condition: Predicate::ValueAtLeast(Value::CardsDrawnThisTurn(PlayerRef::You), Value::Const(2)),
                    amount: 2,
                },
            },
            StaticAbility {
                description: "Other Birds you control have vigilance.".into(),
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(bird().and(R::ControlledByYou).and(R::OtherThanSource)),
                    keyword: Keyword::Vigilance,
                },
            },
        ],
        ..legend(
            "Gwaihir the Windlord",
            cost(&[generic(4), w(), u()]),
            vec![CreatureType::Bird, CreatureType::Noble],
            4,
            4,
        )
    }
}

/// Tawnos, the Toymaker — {3}{G}{U} 3/5 legendary Human Artificer. Whenever
/// you cast a Beast or Bird creature spell, you may copy it, the copy an
/// artifact in addition to its other types (CR 707.9b, 707.10f: it becomes a
/// token).
pub fn tawnos_the_toymaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(
                R::Creature.and(R::HasCreatureType(CreatureType::Beast).or(R::HasCreatureType(CreatureType::Bird))),
            )),
            effect: Effect::MayDo {
                description: "Copy that spell (the copy is also an artifact)?".into(),
                body: Box::new(Effect::CopySpellAddingTypes {
                    what: Selector::TriggerSource,
                    types: vec![CardType::Artifact],
                }),
            },
        }],
        ..legend(
            "Tawnos, the Toymaker",
            cost(&[generic(3), g(), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            5,
        )
    }
}

/// The Lord of the Eagles — {7}{U}{U} 8/8 legendary Bird Noble, flash,
/// flying. Costs {X} less, X the total power of creatures you control with
/// flying.
pub fn the_lord_of_the_eagles() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast, where X is the total power of creatures you control with flying.".into(),
            effect: StaticEffect::SelfCostReducedByValue {
                amount: Value::PowerOf(Box::new(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::HasKeyword(Keyword::Flying)),
                ))),
            },
        }],
        ..legend(
            "The Lord of the Eagles",
            cost(&[generic(7), u(), u()]),
            vec![CreatureType::Bird, CreatureType::Noble],
            8,
            8,
        )
    }
}

/// Watcher of the Spheres — {W}{U} 2/2 Bird Wizard, flying. Your creature
/// spells with flying cost {1} less; another creature with flying entering
/// under your control gives it +1/+1 until end of turn.
pub fn watcher_of_the_spheres() -> CardDefinition {
    let flier = || R::Creature.and(R::HasKeyword(Keyword::Flying));
    CardDefinition {
        name: "Watcher of the Spheres",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Bird, CreatureType::Wizard]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Creature spells with flying you cast cost {1} less to cast.".into(),
            effect: StaticEffect::CostReduction { filter: flier(), amount: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(trigger_is(flier().and(R::OtherThanSource))),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: crate::effect::Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Flurry of Wings — {G}{W}{U} Instant. Create X 1/1 white Bird Soldier
/// tokens with flying, X the number of attacking creatures.
pub fn flurry_of_wings() -> CardDefinition {
    CardDefinition {
        name: "Flurry of Wings",
        cost: cost(&[g(), w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::count(Selector::EachPermanent(R::Creature.and(R::IsAttacking))),
            definition: Arc::new(crate::card::TokenDefinition {
                name: "Bird Soldier".into(),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Flying],
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: creature_types(vec![CreatureType::Bird, CreatureType::Soldier]),
                ..Default::default()
            }),
        },
        ..Default::default()
    }
}
