//! Commander: the cards the **Enhanced Evolution** precon (C20, Otrimi, the
//! Ever-Playful) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_otrimi.rs`.
//!
//! Residuals (each also on its card):
//! - **Capricopian** — the attacked player's "{2}: put a +1/+1 counter on it,
//!   then reselect which player it's attacking" isn't offered.
//! - **Manascape Refractor** — mana of any color isn't spendable on the
//!   borrowed abilities' activation costs.
//! - **Mindleecher** — the exiled cards may be cast with mana of any type
//!   (the face-down exile primitive's Gonti spend).
//! - **Vastwood Hydra** — the counters are distributed among up to three
//!   target creatures you control, not "any number".

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EventKind, EventScope,
    EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies, on_mutate, partner_with_search, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{ManaCost, b, cost, g, generic, u, x};

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

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn plus_counters(what: Selector, amount: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount }
}

/// "This creature enters with X +1/+1 counters on it."
fn x_hydra(def: CardDefinition) -> CardDefinition {
    CardDefinition { enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)), ..def }
}

fn combat_damage_to_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

/// Otrimi, the Ever-Playful — {3}{B}{G}{U} 6/6 Nightmare Beast. Mutate
/// {1}{B}{G}{U}; trample; combat damage to a player returns a creature card
/// with mutate from your graveyard to your hand.
pub fn otrimi_the_ever_playful() -> CardDefinition {
    legend(CardDefinition {
        keywords: vec![Keyword::Trample],
        mutate: Some(cost(&[generic(1), b(), g(), u()])),
        can_be_commander: true,
        triggered_abilities: vec![combat_damage_to_player(Effect::Move {
            what: target_filtered(R::Creature.and(R::HasMutate).and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(
            "Otrimi, the Ever-Playful",
            cost(&[generic(3), b(), g(), u()]),
            vec![CreatureType::Nightmare, CreatureType::Beast],
            6,
            6,
        )
    })
}

/// Animist's Awakening — {X}{G} sorcery. Reveal the top X; lands onto the
/// battlefield tapped, the rest on the bottom in a random order. Spell
/// mastery (CR 207.2c, read as it resolves): the lands enter untapped.
pub fn animists_awakening() -> CardDefinition {
    let mastery = Predicate::ValueAtLeast(
        Value::CardsInGraveyardMatching {
            who: PlayerRef::You,
            filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
        },
        Value::Const(2),
    );
    CardDefinition {
        name: "Animist's Awakening",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: mastery,
            then: Box::new(Effect::RevealTopPutLandsRestBottomRandom { count: Value::XFromCost, tapped: false }),
            else_: Box::new(Effect::RevealTopPutLandsRestBottomRandom { count: Value::XFromCost, tapped: true }),
        },
        ..Default::default()
    }
}

/// Boneyard Mycodrax — {2}{B} */* Fungus: the other creature cards in your
/// graveyard. Scavenge {4}{B} (CR 702.96): exiled as the cost, so the count
/// it moves no longer includes itself.
pub fn boneyard_mycodrax() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::BasePlusCreaturesInControllerGraveyard { base: 0 }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: plus_counters(
                target_filtered(R::Creature),
                Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
            ),
            ..Default::default()
        }],
        ..creature("Boneyard Mycodrax", cost(&[generic(2), b()]), vec![CreatureType::Fungus], 0, 0)
    }
}

/// Capricopian — {X}{G} 0/0 Goat Hydra with X +1/+1 counters.
/// Residual: the attacked player's "{2}: put a +1/+1 counter on it, then
/// reselect which player it's attacking" isn't offered.
pub fn capricopian() -> CardDefinition {
    x_hydra(creature("Capricopian", cost(&[x(), g()]), vec![CreatureType::Goat, CreatureType::Hydra], 0, 0))
}

/// Cazur, Ruthless Stalker — {3}{G} 3/3. Partner with Ukkima; a creature you
/// control that deals combat damage to a player gets a +1/+1 counter.
pub fn cazur_ruthless_stalker() -> CardDefinition {
    legend(CardDefinition {
        keywords: vec![Keyword::PartnerWith("Ukkima, Stalking Shadow".into())],
        triggered_abilities: vec![
            partner_with_search("Ukkima, Stalking Shadow"),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
                effect: plus_counters(Selector::TriggerSource, Value::ONE),
            },
        ],
        ..creature(
            "Cazur, Ruthless Stalker",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    })
}

/// Ukkima, Stalking Shadow — {1}{U}{B} 2/2 Whale Wolf. Partner with Cazur;
/// can't be blocked; leaving, it deals its power (CR 603.10, last known) to
/// target player and you gain that much.
pub fn ukkima_stalking_shadow() -> CardDefinition {
    let power = Value::PowerOf(Box::new(Selector::This));
    legend(CardDefinition {
        keywords: vec![Keyword::PartnerWith("Cazur, Ruthless Stalker".into()), Keyword::Unblockable],
        triggered_abilities: vec![
            partner_with_search("Cazur, Ruthless Stalker"),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::Player), amount: power.clone() },
                    Effect::GainLife { who: Selector::You, amount: power },
                ]),
            },
        ],
        ..creature(
            "Ukkima, Stalking Shadow",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Whale, CreatureType::Wolf],
            2,
            2,
        )
    })
}

/// Endless Sands — Desert. {T}: {C}. {2},{T}: exile target creature you
/// control. {4},{T}, sacrifice: return each creature card exiled with it
/// under its owner's control.
pub fn endless_sands() -> CardDefinition {
    CardDefinition {
        name: "Endless Sands",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Desert], ..Default::default() },
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::ExileLinked { what: target_filtered(R::Creature.and(R::ControlledByYou)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hungering Hydra — {X}{G} 0/0 with X +1/+1 counters. Can't be blocked by
/// more than one creature; damage dealt to it becomes that many counters.
pub fn hungering_hydra() -> CardDefinition {
    x_hydra(CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: plus_counters(Selector::This, Value::TriggerEventAmount),
        }],
        ..creature("Hungering Hydra", cost(&[x(), g()]), vec![CreatureType::Hydra], 0, 0)
    })
}

/// Manascape Refractor — {3} artifact; enters tapped; has all activated
/// abilities of all lands on the battlefield.
/// Residual: mana of any color isn't spendable on those abilities' costs.
pub fn manascape_refractor() -> CardDefinition {
    CardDefinition {
        name: "Manascape Refractor",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![crate::sets::enters_tapped(), StaticAbility {
            description: "Has all activated abilities of all lands on the battlefield.".into(),
            effect: StaticEffect::HasActivatedAbilitiesOfBattlefieldLands,
        }],
        ..Default::default()
    }
}

/// Mindleecher — {4}{B}{B} 5/5 Nightmare. Mutate {4}{B}; flying; mutating,
/// it exiles the top card of each opponent's library face down, playable by
/// you while exiled.
/// Residual: those cards may be cast with mana of any type.
pub fn mindleecher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        mutate: Some(cost(&[generic(4), b()])),
        triggered_abilities: vec![on_mutate(Effect::ForEachOpponent {
            body: Box::new(Effect::ExileTopFaceDownGrantPlay {
                library: PlayerRef::Triggerer,
                grantee: PlayerRef::You,
            }),
        })],
        ..creature("Mindleecher", cost(&[generic(4), b(), b()]), vec![CreatureType::Nightmare], 5, 5)
    }
}

/// Nissa, Steward of Elements — {X}{G}{U} planeswalker, loyalty X. +2: scry
/// 2. 0: the top card may enter if it's a land or a creature with mana value
/// at most her loyalty. −6: up to two lands you control untap and become 5/5
/// flying, hasty Elementals until end of turn.
pub fn nissa_steward_of_elements() -> CardDefinition {
    CardDefinition {
        name: "Nissa, Steward of Elements",
        cost: cost(&[x(), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Nissa], ..Default::default() },
        // CR 306.5b — X loyalty rides in as loyalty counters.
        enters_with_counters: Some((CounterType::Loyalty, Value::XFromCost)),
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                x_cost: false,
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                x_cost: false,
                effect: Effect::LookTopMayPutLandOrCreatureMvAtMost {
                    max_mv: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Loyalty },
                },
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                x_cost: false,
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Land.and(R::ControlledByYou),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Untap { what: Selector::Target(0), up_to: None },
                        Effect::BecomeCreature {
                            what: Selector::Target(0),
                            power: Value::Const(5),
                            toughness: Value::Const(5),
                            creature_types: vec![CreatureType::Elemental],
                            keywords: vec![Keyword::Flying, Keyword::Haste],
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
            },
        ],
        ..Default::default()
    }
}

/// Pouncing Shoreshark — {4}{U} 4/3 Shark Beast. Mutate {3}{U}; flash;
/// mutating, you may bounce a creature an opponent controls.
pub fn pouncing_shoreshark() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        mutate: Some(cost(&[generic(3), u()])),
        triggered_abilities: vec![on_mutate(Effect::MayDo {
            description: "Return target creature an opponent controls to its owner's hand?".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            }),
        })],
        ..creature(
            "Pouncing Shoreshark",
            cost(&[generic(4), u()]),
            vec![CreatureType::Shark, CreatureType::Beast],
            4,
            3,
        )
    }
}

/// Souvenir Snatcher — {4}{U} 4/4 Bird. Mutate {5}{U}; flying; mutating, it
/// gains control of target noncreature artifact.
pub fn souvenir_snatcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        mutate: Some(cost(&[generic(5), u()])),
        triggered_abilities: vec![on_mutate(Effect::GainControl {
            what: target_filtered(R::Artifact.and(R::Noncreature)),
            to: None,
            duration: Duration::Permanent,
        })],
        ..creature("Souvenir Snatcher", cost(&[generic(4), u()]), vec![CreatureType::Bird], 4, 4)
    }
}

/// Tidal Barracuda — {3}{U} 3/4 Fish. Any player may cast spells as though
/// they had flash; your opponents can't cast spells during your turn.
pub fn tidal_barracuda() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Any player may cast spells as though they had flash.".into(),
                effect: StaticEffect::AnyPlayerSpellsHaveFlash { filter: R::Any },
            },
            StaticAbility {
                description: "Your opponents can't cast spells during your turn.".into(),
                effect: StaticEffect::OpponentsCantCastDuringYourTurn,
            },
        ],
        ..creature("Tidal Barracuda", cost(&[generic(3), u()]), vec![CreatureType::Fish], 3, 4)
    }
}

/// Vastwood Hydra — {X}{G}{G} 0/0 with X +1/+1 counters. Dying, you may
/// distribute its +1/+1 counters (last known, CR 603.10) among creatures you
/// control.
/// Residual: among up to three target creatures, not "any number".
pub fn vastwood_hydra() -> CardDefinition {
    x_hydra(CardDefinition {
        triggered_abilities: vec![on_dies(Effect::MayDo {
            description: "Distribute its +1/+1 counters among creatures you control?".into(),
            body: Box::new(Effect::DistributeCounters {
                total: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                counter: CounterType::PlusOnePlusOne,
                filter: R::Creature.and(R::ControlledByYou),
                max_targets: 3,
            }),
        })],
        ..creature("Vastwood Hydra", cost(&[x(), g(), g()]), vec![CreatureType::Hydra], 0, 0)
    })
}

/// Villainous Wealth — {X}{B}{G}{U} sorcery. Target opponent exiles their top
/// X; you may cast any number of spells with mana value X or less from among
/// them free (CR 601.2).
pub fn villainous_wealth() -> CardDefinition {
    CardDefinition {
        name: "Villainous Wealth",
        cost: cost(&[x(), b(), g(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // "Target opponent": a zero-card draw names slot 0's filter.
            Effect::Draw { who: Selector::TargetFiltered { slot: 0, filter: R::OpponentPlayer }, amount: Value::Const(0) },
            Effect::ExileLinked {
                what: Selector::TopOfLibrary {
                    who: PlayerRef::Target(0),
                    count: Value::XFromCost,
                },
            },
            Effect::CastAnyOrderWithoutPaying {
                what: Selector::CardExiledWithSource,
                source_zone: crate::card::Zone::Exile,
                filter: Some(R::Nonland.and(R::ManaValueAtMostXFromCost)),
                cap: None,
            },
        ]),
        ..Default::default()
    }
}

/// Vorapede — {2}{G}{G}{G} 5/4 Insect. Vigilance, trample, undying.
pub fn vorapede() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Undying],
        ..creature("Vorapede", cost(&[generic(2), g(), g(), g()]), vec![CreatureType::Insect], 5, 4)
    }
}

/// Wydwen, the Biting Gale — {2}{U}{B} 3/3 Faerie Wizard. Flash, flying;
/// {U}{B}, pay 1 life: return it to its owner's hand.
pub fn wydwen_the_biting_gale() -> CardDefinition {
    legend(CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), b()]),
            life_cost: 1,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))) },
            ..Default::default()
        }],
        ..creature(
            "Wydwen, the Biting Gale",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Faerie, CreatureType::Wizard],
            3,
            3,
        )
    })
}

/// Yavimaya Dryad — {1}{G}{G} 2/1. Forestwalk; entering, you may search for
/// a Forest card and put it onto the battlefield tapped under target
/// player's control.
pub fn yavimaya_dryad() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            // Names the target player (a zero-card draw declares the slot).
            Effect::Draw { who: target_filtered(R::Player), amount: Value::Const(0) },
            Effect::MayDo {
                description: "Search your library for a Forest card?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasLandType(LandType::Forest),
                    to: ZoneDest::Battlefield { controller: PlayerRef::Target(0), tapped: true },
                }),
            },
        ]))],
        ..creature("Yavimaya Dryad", cost(&[generic(1), g(), g()]), vec![CreatureType::Dryad], 2, 1)
    }
}
