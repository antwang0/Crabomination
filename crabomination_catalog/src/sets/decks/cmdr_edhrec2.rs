//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat, second file (`cmdr_edhrec` is the
//! first). Henzie "Toolbox" Torre, Frodo, Adventurous Hobbit + Sam, Loyal
//! Attendant, K'rrik, Son of Yawgmoth, Voja, Jaws of the Conclave, Choco, Seeker of
//! Paradise, Light-Paws, Emperor's Voice, Yurlok of Scorch Thrash, Rocco, Street Chef,
//! Tinybones, Bauble Burglar and Indominus Rex, Alpha. Tests in `tests/recent_b/cmdr_edhrec2.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, EnchantmentSubtype, EquipBonus, EquipScale,
    MayPlayDuration, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, on_dies};
use crate::effect::{Effect, LookPick, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, mono_hybrid, r, u, w, Color};
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
            description: "If you would create a Food token, instead create a Food token and a Treasure token.",
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
            description: "If you would draw a card, exile the top card of your library face down instead.",
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
            description: "Spells you cast from exile have convoke.",
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
                description: "Each other Warrior creature you control enters with an additional +1/+1 counter on it.",
                effect: StaticEffect::MatchingEntersWithExtraCounters {
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Warrior)).and(R::OtherThanSource),
                    kind: CounterType::PlusOnePlusOne,
                    amount: 1,
                },
            },
            StaticAbility {
                description: "Each creature you control with a +1/+1 counter on it has trample.",
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
            description: "Spells you cast cost {1} less to cast for each card type they share with cards exiled with this creature.",
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
            description: "This land enters tapped unless you control a legendary creature.",
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
                description: "This spell costs {2} less to cast as long as you've drawn two or more cards this turn.",
                effect: StaticEffect::SelfCostReducedIf {
                    condition: Predicate::ValueAtLeast(Value::CardsDrawnThisTurn(PlayerRef::You), Value::Const(2)),
                    amount: 2,
                },
            },
            StaticAbility {
                description: "Other Birds you control have vigilance.",
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
            description: "This spell costs {X} less to cast, where X is the total power of creatures you control with flying.",
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
            description: "Creature spells with flying you cast cost {1} less to cast.",
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

fn aura_on(name: &'static str, mana: crate::mana::ManaCost, enchant: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: crate::effect::shortcut::target_filtered(enchant) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// Helm of the Gods — {1} Equipment. Equipped creature gets +1/+1 for each
/// enchantment you control. Equip {1}.
pub fn helm_of_the_gods() -> CardDefinition {
    CardDefinition {
        name: "Helm of the Gods",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::Enchantment.and(R::ControlledByYou),
                per_power: 1,
                per_toughness: 1,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Armored Ascension — {3}{W} Aura. Enchanted creature gets +1/+1 for each
/// Plains you control and has flying.
pub fn armored_ascension() -> CardDefinition {
    aura_on(
        "Armored Ascension",
        cost(&[generic(3), w()]),
        R::Creature,
        EquipBonus {
            keywords: vec![Keyword::Flying],
            scale: Some(EquipScale {
                filter: R::Land.and(R::HasLandType(LandType::Plains)).and(R::ControlledByYou),
                per_power: 1,
                per_toughness: 1,
                ..Default::default()
            }),
            ..Default::default()
        },
    )
}

/// Battle Mastery — {2}{W} Aura. Enchanted creature has double strike.
pub fn battle_mastery() -> CardDefinition {
    aura_on(
        "Battle Mastery",
        cost(&[generic(2), w()]),
        R::Creature,
        EquipBonus { keywords: vec![Keyword::DoubleStrike], ..Default::default() },
    )
}

/// Benevolent Blessing — {1}{W} Aura, flash. As it enters, choose a color;
/// enchanted creature has protection from that color, which doesn't remove
/// Auras you control already on it (CR 702.16k; protection never sheds
/// Equipment in this engine, so that half holds by construction).
pub fn benevolent_blessing() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has protection from the chosen color.",
            effect: StaticEffect::GrantProtectionFromChosenColor {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..aura_on(
            "Benevolent Blessing",
            cost(&[generic(1), w()]),
            R::Creature,
            EquipBonus { protection_keeps_yours: true, ..Default::default() },
        )
    }
}

/// With Great Power . . . — {3}{W} Aura, enchant creature you control. +2/+2
/// for each Aura and Equipment attached to it; all damage that would be dealt
/// to you is dealt to it instead.
pub fn with_great_power() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All damage that would be dealt to you is dealt to enchanted creature instead.",
            effect: StaticEffect::RedirectControllerDamageToEquippedCreature,
        }],
        ..aura_on(
            "With Great Power . . .",
            cost(&[generic(3), w()]),
            R::Creature.and(R::ControlledByYou),
            EquipBonus {
                scale: Some(EquipScale {
                    filter: R::Any,
                    per_power: 2,
                    per_toughness: 2,
                    count_host_attachments: Some(
                        R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                            .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
    }
}

/// Rebuff the Wicked — {W} Instant. Counter target spell that targets a
/// permanent you control.
pub fn rebuff_the_wicked() -> CardDefinition {
    CardDefinition {
        name: "Rebuff the Wicked",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: crate::effect::shortcut::target_filtered(R::SpellTargetsMatching(Box::new(
                R::Permanent.and(R::ControlledByYou),
            ))),
        },
        ..Default::default()
    }
}

/// Yurlok of Scorch Thrash — {1}{B}{R}{G} 4/4 legendary Lizard Shaman,
/// vigilance. A player losing unspent mana loses that much life. {1}, {T}:
/// each player adds {B}{R}{G}.
pub fn yurlok_of_scorch_thrash() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "A player losing unspent mana causes that player to lose that much life.",
            effect: StaticEffect::PlayersLoseLifeForUnspentMana,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::EachPlayer,
                body: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Black, Color::Red, Color::Green]),
                }),
            },
            ..Default::default()
        }],
        ..legend(
            "Yurlok of Scorch Thrash",
            cost(&[generic(1), b(), r(), g()]),
            vec![CreatureType::Lizard, CreatureType::Shaman],
            4,
            4,
        )
    }
}

/// Horizon Stone — {5} Artifact. If you would lose unspent mana, that mana
/// becomes colorless instead.
pub fn horizon_stone() -> CardDefinition {
    CardDefinition {
        name: "Horizon Stone",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "If you would lose unspent mana, that mana becomes colorless instead.",
            effect: StaticEffect::UnspentManaBecomesColorless,
        }],
        ..Default::default()
    }
}

/// Umbral Mantle — {3} Equipment. Equipped creature has "{3}, {Q}: this
/// creature gets +2/+2 until end of turn." Equip {0}.
pub fn umbral_mantle() -> CardDefinition {
    CardDefinition {
        name: "Umbral Mantle",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(0)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                untap_self_cost: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: crate::effect::Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Belbe, Corrupted Observer — {B}{G} 2/2 legendary Phyrexian Zombie Elf. At
/// the beginning of each postcombat main phase, the active player adds
/// {C}{C} for each of your opponents who lost life this turn.
pub fn belbe_corrupted_observer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::AnyPlayer),
            effect: Effect::AddMana {
                who: PlayerRef::ActivePlayer,
                pool: ManaPayload::Colorless(Value::Times(
                    Box::new(Value::OpponentsWhoLostLifeThisTurn),
                    Box::new(Value::Const(2)),
                )),
            },
        }],
        ..legend(
            "Belbe, Corrupted Observer",
            cost(&[b(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Elf],
            2,
            2,
        )
    }
}

/// Lavaleaper — {3}{R} 4/4 Elemental. All creatures have haste. Whenever a
/// player taps a basic land for mana, that player adds one more of a type it
/// produced (CR 605.1b triggered mana ability).
pub fn lavaleaper() -> CardDefinition {
    CardDefinition {
        name: "Lavaleaper",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Elemental]),
        power: 4,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "All creatures have haste.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Creature),
                    keyword: Keyword::Haste,
                },
            },
            StaticAbility {
                description: "Whenever a player taps a basic land for mana, that player adds one mana of any type that land produced.",
                effect: StaticEffect::ExtraManaOnLandTap {
                    enchanted_only: false,
                    filter: R::IsBasicLand,
                    extra: crate::effect::ExtraManaKind::Mirror,
                    while_monarch: false,
                },
            },
        ],
        ..Default::default()
    }
}

/// Rug of Smothering — {3} 1/3 Construct artifact creature, flying. Whenever
/// a player casts a spell, they lose 1 life for each spell they've cast this
/// turn.
pub fn rug_of_smothering() -> CardDefinition {
    CardDefinition {
        name: "Rug of Smothering",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Construct]),
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::SpellsCastThisTurn(PlayerRef::Triggerer),
            },
        }],
        ..Default::default()
    }
}

/// Power Surge — {R}{R} Enchantment. At the beginning of each player's
/// upkeep, it deals X damage to that player, X the untapped lands they
/// controlled as the turn began.
pub fn power_surge() -> CardDefinition {
    CardDefinition {
        name: "Power Surge",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::UntappedLandsActivePlayerHadAtTurnStart,
            },
        }],
        ..Default::default()
    }
}

/// "Whenever [scope] plays a land from exile or casts a spell from exile" —
/// the land half reads a *played* land (event amount ≥ 1) that came from
/// exile, the spell half the cast's own flag.
fn from_exile_triggers(scope: EventScope, effect: Effect) -> Vec<TriggeredAbility> {
    vec![
        TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, scope).with_filter(Predicate::All(vec![
                Predicate::ValueAtLeast(Value::TriggerEventAmount, Value::ONE),
                trigger_is(R::EnteredFromExileThisTurn),
            ])),
            effect: effect.clone(),
        },
        TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, scope).with_filter(Predicate::CastSpellFromExile),
            effect,
        },
    ]
}

/// Rocco, Street Chef — {R}{G}{W} 2/4 legendary Elf Druid. Your end step:
/// each player exiles their top card and may play it until *your* next end
/// step (`UntilSeatsNextEndStep`). Whenever a player plays a land or casts a
/// spell from exile, you put a +1/+1 counter on target creature and create a
/// Food.
pub fn rocco_street_chef() -> CardDefinition {
    let payoff = Effect::Seq(vec![
        Effect::AddCounter {
            what: crate::effect::shortcut::target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(food_token()) },
    ]);
    let mut triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
        effect: Effect::EachPlayerDoes {
            who: PlayerRef::EachPlayer,
            body: Box::new(Effect::Seq(vec![
                // Each seat's grant names only that seat's card.
                Effect::ClearLastMoved,
                Effect::Move {
                    what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                    to: ZoneDest::Exile,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: MayPlayDuration::UntilSeatsNextEndStep { seat: 0 },
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ])),
        },
    }];
    triggered_abilities.extend(from_exile_triggers(EventScope::AnyPlayer, payoff));
    CardDefinition {
        triggered_abilities,
        ..legend("Rocco, Street Chef", cost(&[r(), g(), w()]), vec![CreatureType::Elf, CreatureType::Druid], 2, 4)
    }
}

/// Pia Nalaar, Consul of Revival — {R}{W} 2/3 legendary Human Artificer.
/// Thopters you control have haste; whenever you play a land or cast a spell
/// from exile, create a 1/1 flying Thopter artifact creature token.
pub fn pia_nalaar_consul_of_revival() -> CardDefinition {
    let thopter = crate::card::TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Thopter]),
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Thopters you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Thopter)).and(R::ControlledByYou),
                ),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: from_exile_triggers(
            EventScope::YourControl,
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(thopter) },
        ),
        ..legend(
            "Pia Nalaar, Consul of Revival",
            cost(&[r(), w()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            3,
        )
    }
}

/// Urabrask, Heretic Praetor — {3}{R}{R} 4/4 legendary Phyrexian Praetor,
/// haste. Your upkeep: exile your top card, playable this turn. Each
/// opponent's upkeep: their next draw this turn exiles their top card
/// instead, playable this turn (CR 121.2a).
pub fn urabrask_heretic_praetor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::OpponentControl),
                effect: Effect::NextDrawThisTurnBecomesImpulse { who: PlayerRef::ActivePlayer },
            },
        ],
        ..legend(
            "Urabrask, Heretic Praetor",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Praetor],
            4,
            4,
        )
    }
}

/// Yotian Dissident — {G}{W} 1/1 Human Artificer. Whenever an artifact you
/// control enters, put a +1/+1 counter on target creature you control.
pub fn yotian_dissident() -> CardDefinition {
    CardDefinition {
        name: "Yotian Dissident",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Artificer]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(trigger_is(R::Artifact)),
            effect: Effect::AddCounter {
                what: crate::effect::shortcut::target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Quintorius Kand — {3}{R}{W} legendary Planeswalker, loyalty 4. Whenever
/// you cast a spell from exile, it deals 2 damage to each opponent and you
/// gain 2 life. +1: a 3/2 red and white Spirit. −3: discover 4. −6: exile any
/// number of target cards from your graveyard, add {R} for each, and you may
/// play them this turn.
pub fn quintorius_kand() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype};
    let spirit = crate::card::TokenDefinition {
        name: "Spirit".into(),
        power: 3,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        subtypes: creature_types(vec![CreatureType::Spirit]),
        ..Default::default()
    };
    let exiled = || Selector::ExiledThisResolution { filter: R::Any };
    CardDefinition {
        name: "Quintorius Kand",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Quintorius], ..Default::default() },
        base_loyalty: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::CastSpellFromExile),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(spirit) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Discover { n: Value::Const(4), filter: None },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::Seq(vec![
                    Effect::ApplyToTargets {
                        max_targets: 16,
                        min_targets: 0,
                        filter: R::InYourGraveyard,
                        effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
                    },
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::OfColor(Color::Red, Value::count(exiled())),
                    },
                    Effect::GrantMayPlay {
                        what: exiled(),
                        duration: MayPlayDuration::EndOfThisTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Avatar's Wrath — {2}{W}{W} Sorcery. Choose up to one target creature,
/// then airbend every other creature (CR 701.65). Until your next turn, your
/// opponents can't cast spells from anywhere but their hands. Exile Avatar's
/// Wrath.
pub fn avatars_wrath() -> CardDefinition {
    CardDefinition {
        name: "Avatar's Wrath",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                // Declares the spared creature's slot; a friendly verb, so
                // the picker spares its own best creature.
                Effect::PumpPT {
                    what: crate::effect::shortcut::target_filtered(R::Creature),
                    power: Value::Const(0),
                    toughness: Value::Const(0),
                    duration: crate::effect::Duration::EndOfTurn,
                },
                Effect::Airbend { what: Selector::EachPermanentExceptTargets(R::Creature) },
                Effect::OpponentsCantCastFromNonHandUntilYourNextTurn,
            ])),
        },
        exile_on_resolve: true,
        ..Default::default()
    }
}

fn each_opponent_discards(n: i32) -> Effect {
    Effect::Discard { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(n), random: false }
}

/// Aclazotz, Deepest Betrayal // Temple of the Dead — {3}{B}{B} 4/4 legendary
/// Bat God, flying, lifelink. Attacks: each opponent discards a card; for each
/// who can't, you draw. An opponent discarding a land card makes a 1/1 flying
/// Bat. Dies: returns transformed and tapped. The Temple taps for {B} and
/// transforms back for {2}{B}, {T} at sorcery speed while a player has one or
/// fewer cards in hand.
pub fn aclazotz_deepest_betrayal() -> CardDefinition {
    let bat = crate::card::TokenDefinition {
        name: "Bat".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: creature_types(vec![CreatureType::Bat]),
        ..Default::default()
    };
    let temple = CardDefinition {
        name: "Temple of the Dead",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Black),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), b()]),
                sorcery_speed: true,
                condition: Some(Predicate::ForAnyPlayer {
                    who: PlayerRef::EachPlayer,
                    pred: Box::new(Predicate::ValueAtMost(Value::HandSizeOf(PlayerRef::Triggerer), Value::ONE)),
                }),
                effect: Effect::Transform { what: Selector::This },
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![
            on_attack(Effect::Seq(vec![
                each_opponent_discards(1),
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Diff(Box::new(Value::OpponentCount), Box::new(Value::CardsDiscardedThisEffect)),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl)
                    .with_filter(trigger_is(R::Land)),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(bat) },
            },
            on_dies(Effect::ReturnSelfTransformedTappedToOwner),
        ],
        back_face: Some(Box::new(temple)),
        ..legend("Aclazotz, Deepest Betrayal", cost(&[generic(3), b(), b()]), vec![CreatureType::Bat, CreatureType::God], 4, 4)
    }
}

/// Cunning Lethemancer — {2}{B} 2/2 Human Wizard. Your upkeep: each player
/// discards a card.
pub fn cunning_lethemancer() -> CardDefinition {
    CardDefinition {
        name: "Cunning Lethemancer",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Discard { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE, random: false },
        }],
        ..Default::default()
    }
}

/// Fell Specter — {3}{B} 1/3 Specter, flying. Enters: target opponent
/// discards a card. Whenever an opponent discards a card, that player loses 2
/// life.
pub fn fell_specter() -> CardDefinition {
    CardDefinition {
        name: "Fell Specter",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Specter]),
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::Discard {
                who: crate::effect::shortcut::target_filtered(R::OpponentPlayer),
                amount: Value::ONE,
                random: false,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl),
                effect: Effect::LoseLife { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(2) },
            },
        ],
        ..Default::default()
    }
}

/// The Raven Man — {1}{B} 2/1 legendary Human Wizard. Each end step, if a
/// player discarded a card this turn: a 1/1 black flying Bird that can't
/// block. {3}{B}, {T}: each opponent discards a card (sorcery speed).
pub fn the_raven_man() -> CardDefinition {
    let bird = crate::card::TokenDefinition {
        name: "Bird".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::CantBlock],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: creature_types(vec![CreatureType::Bird]),
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                Predicate::ValueAtLeast(Value::CardsDiscardedThisTurn(PlayerRef::EachPlayer), Value::ONE),
            ),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(bird) },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            mana_cost: cost(&[generic(3), b()]),
            effect: each_opponent_discards(1),
            ..Default::default()
        }],
        ..legend("The Raven Man", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 1)
    }
}

/// Tinybones, Pocket Nuisance — {2}{B} 2/1 legendary Skeleton Rogue. Enters:
/// each opponent discards a card. Whenever a player discards one or more
/// cards, it deals 1 damage to each opponent.
pub fn tinybones_pocket_nuisance() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(each_opponent_discards(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DiscardedOneOrMore, EventScope::AnyPlayer),
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            },
        ],
        ..legend(
            "Tinybones, Pocket Nuisance",
            cost(&[generic(2), b()]),
            vec![CreatureType::Skeleton, CreatureType::Rogue],
            2,
            1,
        )
    }
}

/// Tinybones, Trinket Thief — {1}{B} 1/2 legendary Skeleton Rogue. Each end
/// step, if an opponent discarded a card this turn, you draw a card and lose
/// 1 life. {4}{B}{B}: each opponent with no cards in hand loses 10 life.
pub fn tinybones_trinket_thief() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                Predicate::ValueAtLeast(Value::CardsDiscardedThisTurn(PlayerRef::EachOpponent), Value::ONE),
            ),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b(), b()]),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::EachOpponent,
                body: Box::new(Effect::If {
                    cond: Predicate::HellbentActive { who: PlayerRef::You },
                    then: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(10) }),
                    else_: Box::new(Effect::Noop),
                }),
            },
            ..Default::default()
        }],
        ..legend(
            "Tinybones, Trinket Thief",
            cost(&[generic(1), b()]),
            vec![CreatureType::Skeleton, CreatureType::Rogue],
            1,
            2,
        )
    }
}

/// Arterial Flow — {1}{B}{B} Sorcery. Each opponent discards two cards; if
/// you control a Vampire, each opponent loses 2 life and you gain 2 life.
pub fn arterial_flow() -> CardDefinition {
    CardDefinition {
        name: "Arterial Flow",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            each_opponent_discards(2),
            Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Vampire),
                }),
                then: Box::new(Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Mind Rake — {2}{B} Sorcery. Target player discards two cards. Overload
/// {1}{B} (CR 702.96): each player discards two.
pub fn mind_rake() -> CardDefinition {
    CardDefinition {
        name: "Mind Rake",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Discard {
            who: crate::effect::shortcut::target_filtered(R::Player),
            amount: Value::Const(2),
            random: false,
        },
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(1), b()]),
            effect_override: Some(Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(2),
                random: false,
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Vicious Rumors — {B} Sorcery. 1 damage to each opponent; each opponent
/// discards a card, then mills a card; you gain 1 life.
pub fn vicious_rumors() -> CardDefinition {
    CardDefinition {
        name: "Vicious Rumors",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            each_opponent_discards(1),
            Effect::Mill { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Indominus Rex, Alpha — {1}{U/B}{U/B}{G}{G} 6/6 legendary Dinosaur Mutant.
/// As it enters (CR 614.12), discard any number of creature cards; it enters
/// with a counter of each listed keyword a discarded card has (CR 122.1b).
/// Enters: draw a card for each counter on it.
pub fn indominus_rex_alpha() -> CardDefinition {
    use Keyword as K;
    let kinds = [
        K::Flying, K::FirstStrike, K::DoubleStrike, K::Deathtouch, K::Hexproof, K::Haste,
        K::Indestructible, K::Lifelink, K::Menace, K::Reach, K::Trample, K::Vigilance,
    ];
    let mut as_enters = vec![Effect::DiscardAnyNumber {
        who: Selector::You,
        filter: R::Creature,
        max: None,
    }];
    as_enters.extend(kinds.into_iter().map(|kw| Effect::If {
        cond: Predicate::SelectorExists(Selector::DiscardedThisResolution { filter: R::HasKeyword(kw.clone()) }),
        then: Box::new(Effect::AddKeywordCounter { what: Selector::This, keyword: kw, amount: Value::ONE }),
        else_: Box::new(Effect::Noop),
    }));
    CardDefinition {
        as_enters_effect: Some(Effect::Seq(as_enters)),
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::TotalCountersOn { what: Box::new(Selector::This) },
        })],
        ..legend(
            "Indominus Rex, Alpha",
            cost(&[
                generic(1),
                crate::mana::hybrid(Color::Blue, Color::Black),
                crate::mana::hybrid(Color::Blue, Color::Black),
                g(),
                g(),
            ]),
            vec![CreatureType::Dinosaur, CreatureType::Mutant],
            6,
            6,
        )
    }
}

/// Gurmag Swiftwing — {1}{B} 1/2 Bat, flying, first strike, haste.
pub fn gurmag_swiftwing() -> CardDefinition {
    CardDefinition {
        name: "Gurmag Swiftwing",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Bat]),
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Haste],
        ..Default::default()
    }
}

/// Hit-Monkey — {3}{G} 3/3 legendary Monkey Assassin. Can't be countered;
/// reach, vigilance, deathtouch, hexproof, haste.
pub fn hit_monkey() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CantBeCountered,
            Keyword::Reach,
            Keyword::Vigilance,
            Keyword::Deathtouch,
            Keyword::Hexproof,
            Keyword::Haste,
        ],
        ..legend("Hit-Monkey", cost(&[generic(3), g()]), vec![CreatureType::Monkey, CreatureType::Assassin], 3, 3)
    }
}

/// Mirri the Cursed — {2}{B}{B} 3/2 legendary Vampire Cat, flying, first
/// strike, haste. Combat damage to a creature puts a +1/+1 counter on her.
pub fn mirri_the_cursed() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legend("Mirri the Cursed", cost(&[generic(2), b(), b()]), vec![CreatureType::Vampire, CreatureType::Cat], 3, 2)
    }
}

/// Morbius the Living Vampire — {2}{U}{B} 3/1 legendary Vampire Scientist
/// Villain, flying, vigilance, lifelink. {U}{B}, exile it from your
/// graveyard: look at the top three, one to hand, the rest on the bottom in
/// any order.
pub fn morbius_the_living_vampire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::LookPickToHand(Box::new(LookPick { count: Value::Const(3), ..Default::default() })),
                Effect::OrderLibraryBottom { who: PlayerRef::You, count: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Morbius the Living Vampire",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Vampire, CreatureType::Scientist, CreatureType::Villain],
            3,
            1,
        )
    }
}

/// Nightveil Predator — {U}{U}{B}{B} 3/3 Vampire, flying, deathtouch,
/// hexproof.
pub fn nightveil_predator() -> CardDefinition {
    CardDefinition {
        name: "Nightveil Predator",
        cost: cost(&[u(), u(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Vampire]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Hexproof],
        ..Default::default()
    }
}

/// Vengeful Reaper — {3}{B} 2/3 Angel Cleric, flying, deathtouch, haste.
/// Foretell {1}{B} (CR 702.143).
pub fn vengeful_reaper() -> CardDefinition {
    CardDefinition {
        name: "Vengeful Reaper",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Angel, CreatureType::Cleric]),
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Haste],
        foretell_cost: Some(cost(&[generic(1), b()])),
        ..Default::default()
    }
}

/// Shadow of the Grave — {1}{B} Instant. Return to your hand every card in
/// your graveyard you cycled or discarded this turn (cycling discards).
pub fn shadow_of_the_grave() -> CardDefinition {
    CardDefinition {
        name: "Shadow of the Grave",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: R::DiscardedThisTurn,
            },
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Luxior, Giada's Gift — {1} legendary Equipment. Equipped creature gets
/// +1/+1 for each counter on it; the equipped permanent isn't a planeswalker
/// and is a creature in addition to its other types (loyalty abilities still
/// activate). Equip planeswalker {1}; equip {3}.
pub fn luxior_giadas_gift() -> CardDefinition {
    CardDefinition {
        name: "Luxior, Giada's Gift",
        cost: cost(&[generic(1)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equip_filtered_cost: Some((R::Planeswalker, cost(&[generic(1)]))),
        equipped_bonus: Some(EquipBonus {
            add_card_types: vec![CardType::Creature],
            remove_card_types: vec![CardType::Planeswalker],
            scale: Some(EquipScale {
                filter: R::Any,
                per_power: 1,
                per_toughness: 1,
                count_host_counters: true,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Liliana, Waker of the Dead — {2}{B}{B} legendary Planeswalker, loyalty 4.
/// +1: each player discards a card; each opponent who can't loses 3 life.
/// −3: target creature gets -X/-X, X the cards in your graveyard. −7: an
/// emblem — at the beginning of combat on your turn, put target creature card
/// from a graveyard onto the battlefield under your control; it gains haste.
pub fn liliana_waker_of_the_dead() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype};
    let graveyard = Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Any };
    CardDefinition {
        name: "Liliana, Waker of the Dead",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Liliana], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Discard { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE, random: false },
                    Effect::EachPlayerDoes {
                        who: PlayerRef::EachOpponent,
                        body: Box::new(Effect::If {
                            cond: Predicate::Not(Box::new(Predicate::DiscardedThisEffect { who: PlayerRef::You })),
                            then: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(3) }),
                            else_: Box::new(Effect::Noop),
                        }),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::PumpPT {
                    what: crate::effect::shortcut::target_filtered(R::Creature),
                    power: Value::Negate(Box::new(graveyard.clone())),
                    toughness: Value::Negate(Box::new(graveyard)),
                    duration: crate::effect::Duration::EndOfTurn,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Liliana, Waker of the Dead".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                        effect: Effect::Seq(vec![
                            Effect::Move {
                                what: crate::effect::shortcut::target_filtered(R::Creature.from_any_graveyard()),
                                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                            },
                            Effect::GrantKeyword {
                                what: Selector::LastMoved,
                                keyword: Keyword::Haste,
                                duration: crate::effect::Duration::Permanent,
                            },
                        ]),
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// "A third, rounded up" of `v` (Pox): `(v + 2) / 3`.
fn third_up(v: Value) -> Value {
    Value::DivDown(Box::new(Value::Sum(vec![v, Value::Const(2)])), 3)
}

/// Pox — {B}{B}{B} Sorcery. Each player loses a third of their life, then
/// discards a third of their hand, then sacrifices a third of their creatures,
/// then a third of their lands — each rounded up, each counted as that step
/// begins, each player choosing what they sacrifice.
pub fn pox() -> CardDefinition {
    let each = |body: Effect| Effect::ForEach { selector: Selector::Player(PlayerRef::EachPlayer), body: Box::new(body) };
    let theirs = |filter: R| Value::count(Selector::ControlledBy { who: PlayerRef::Triggerer, filter });
    CardDefinition {
        name: "Pox",
        cost: cost(&[b(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            each(Effect::LoseLife {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: third_up(Value::LifeOf(PlayerRef::Triggerer)),
            }),
            each(Effect::Discard {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: third_up(Value::HandSizeOf(PlayerRef::Triggerer)),
                random: false,
            }),
            each(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::Triggerer),
                count: third_up(theirs(R::Creature)),
                filter: R::Creature,
            }),
            each(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::Triggerer),
                count: third_up(theirs(R::Land)),
                filter: R::Land,
            }),
        ]),
        ..Default::default()
    }
}

/// Shadowborn Apostle — {B} 1/1 Human Cleric. A deck can have any number of
/// them (CR 903.5b). {B}, sacrifice six creatures named Shadowborn Apostle:
/// search for a Demon creature card, put it onto the battlefield.
pub fn shadowborn_apostle() -> CardDefinition {
    CardDefinition {
        name: "Shadowborn Apostle",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Cleric]),
        power: 1,
        toughness: 1,
        static_abilities: vec![crate::sets::deck_may_have_copies(None)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((R::Creature.and(R::HasName("Shadowborn Apostle".into())), 6)),
            sac_other_may_be_source: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Creature.and(R::HasCreatureType(CreatureType::Demon)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Taborax, Hope's Demise — {2}{B} 2/2 legendary Demon Cleric, flying;
/// lifelink with five or more +1/+1 counters. Another nontoken creature of
/// yours dying puts a +1/+1 counter on it; if that was a Cleric you may draw,
/// losing 1 life if you do.
pub fn taborax_hopes_demise() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Taborax has lifelink as long as it has five or more +1/+1 counters on it.",
            effect: StaticEffect::SelfHasKeywordWhile {
                keyword: Keyword::Lifelink,
                condition: R::WithCounterAtLeast(CounterType::PlusOnePlusOne, 5),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(trigger_is(R::Not(Box::new(R::IsToken)))),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Cleric),
                    },
                    then: Box::new(Effect::MayDo {
                        description: "Draw a card and lose 1 life?".into(),
                        body: Box::new(Effect::Seq(vec![
                            Effect::Draw { who: Selector::You, amount: Value::ONE },
                            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                        ])),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..legend("Taborax, Hope's Demise", cost(&[generic(2), b()]), vec![CreatureType::Demon, CreatureType::Cleric], 2, 2)
    }
}

/// Secret Salvage — {3}{B}{B} Sorcery. Exile target nonland card from your
/// graveyard; search for any number of cards with its name, reveal them, put
/// them into your hand.
pub fn secret_salvage() -> CardDefinition {
    CardDefinition {
        name: "Secret Salvage",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: crate::effect::shortcut::target_filtered(R::Nonland.and(R::InYourGraveyard)),
                to: ZoneDest::Exile,
            },
            Effect::SearchAnyNumber { who: PlayerRef::You, filter: R::SameNameAsTarget, to: ZoneDest::Hand(PlayerRef::You) },
        ]),
        ..Default::default()
    }
}
