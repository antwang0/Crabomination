//! Commander: 2026-10-19 (`016Xh16H`) — the five- and six-gap decks of simple
//! cards: Bucky Barnes's white weenies, Óin the Brave's red recursion, Lin
//! Sivvi's Rebels, Klaw's hand attack, Kytheon's Gideons. Tests in
//! `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LoyaltyAbility, MayPlayDuration, PlaneswalkerSubtype,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Zone,
};
use crate::effect::shortcut::{blocks, boast, etb, if_discarded, on_attack, on_dies, renown, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, generic, r, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn white_token(name: &str, creature_type: CreatureType, flying: bool) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: 1,
        toughness: 1,
        keywords: if flying { vec![Keyword::Flying] } else { vec![] },
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![creature_type], ..Default::default() },
        ..Default::default()
    }
}

// ── Bucky Barnes, Eager Ally ────────────────────────────────────────────────

/// Alaborn Zealot — {W} 1/1 Human Soldier. When it blocks a creature, destroy
/// both creatures.
pub fn alaborn_zealot() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![blocks(Effect::Seq(vec![
            Effect::Destroy { what: Selector::BlockedAttacker },
            Effect::Destroy { what: Selector::This },
        ]))],
        ..creature("Alaborn Zealot", cost(&[w()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 1)
    }
}

/// Apostle of Purifying Light — {1}{W} 2/1, protection from black; {2}:
/// exile target card from a graveyard.
pub fn apostle_of_purifying_light() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Exile { what: target_filtered(R::Any.from_any_graveyard()) },
            ..Default::default()
        }],
        ..creature(
            "Apostle of Purifying Light",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            1,
        )
    }
}

/// Icatian Crier — {2}{W} 1/1 Spellshaper. {1}{W}, {T}, discard a card: two
/// 1/1 white Citizens.
pub fn icatian_crier() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: Arc::new(white_token("Citizen", CreatureType::Citizen, false)),
            },
            ..Default::default()
        }],
        ..creature(
            "Icatian Crier",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Mausoleum Guard — {3}{W} 2/2 Human Scout; dies: two 1/1 white flying
/// Spirits.
pub fn mausoleum_guard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: Arc::new(white_token("Spirit", CreatureType::Spirit, true)),
        })],
        ..creature("Mausoleum Guard", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Scout], 2, 2)
    }
}

/// Siege Striker — {2}{W} 1/1 double strike; attacking, tap any number of
/// your untapped creatures for +1/+1 each (the Orphans of the Wheat shape).
pub fn siege_striker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![on_attack(Effect::TapAnyNumberThenPumpPerTapped {
            filter: R::Creature.and(R::ControlledByYou).and(R::Untapped).and(R::OtherThanSource),
            power: 1,
            toughness: 1,
        })],
        ..creature("Siege Striker", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 1)
    }
}

/// Wandering Champion — {1}{W} 3/1 Human Monk. Combat damage to a player,
/// with a blue or red permanent of yours out (CR 603.4): you may loot.
pub fn wandering_champion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource).with_filter(
                Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::ControlledByYou.and(R::HasColor(Color::Blue).or(R::HasColor(Color::Red))),
                    ),
                    n: Value::ONE,
                },
            ),
            effect: Effect::MayDo {
                description: "Discard a card to draw a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    if_discarded(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                ])),
            },
        }],
        ..creature("Wandering Champion", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Monk], 3, 1)
    }
}

// ── Óin the Brave ───────────────────────────────────────────────────────────

/// Firewing Phoenix — {3}{R} 4/2 flier; {1}{R}{R}{R}: return it from your
/// graveyard to your hand.
pub fn firewing_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r(), r(), r()]),
            from_graveyard: true,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..creature("Firewing Phoenix", cost(&[generic(3), r()]), vec![CreatureType::Phoenix], 4, 2)
    }
}

/// Goblin Settler — {3}{R} 1/1; ETB destroy target land.
pub fn goblin_settler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Destroy { what: target_filtered(R::Land) })],
        ..creature("Goblin Settler", cost(&[generic(3), r()]), vec![CreatureType::Goblin], 1, 1)
    }
}

/// Hound of Griselbrand — {2}{R}{R} 2/2 double strike, undying.
pub fn hound_of_griselbrand() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike, Keyword::Undying],
        ..creature(
            "Hound of Griselbrand",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Dog],
            2,
            2,
        )
    }
}

/// Immortal Phoenix — {4}{R}{R} 5/3 flier; dies: back to its owner's hand.
pub fn immortal_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..creature("Immortal Phoenix", cost(&[generic(4), r(), r()]), vec![CreatureType::Phoenix], 5, 3)
    }
}

/// Pyre Hound — {3}{R} 2/3 trample; a +1/+1 counter whenever you cast an
/// instant or sorcery spell.
pub fn pyre_hound() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                },
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Pyre Hound", cost(&[generic(3), r()]), vec![CreatureType::Elemental, CreatureType::Dog], 2, 3)
    }
}

/// Tuskeri Firewalker — {2}{R} 3/2. Boast — {1}: exile your top card; you may
/// play it this turn (CR 702.142).
pub fn tuskeri_firewalker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![boast(
            cost(&[generic(1)]),
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        )],
        ..creature(
            "Tuskeri Firewalker",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Berserker],
            3,
            2,
        )
    }
}

// ── Lin Sivvi, Defiant Hero ─────────────────────────────────────────────────

fn rebel() -> R {
    R::HasCreatureType(CreatureType::Rebel)
}

/// Amrou Scout — {1}{W} 2/1 Kithkin Rebel Scout. {4}, {T}: put a Rebel
/// permanent card with mana value 3 or less from your library onto the
/// battlefield.
pub fn amrou_scout() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: rebel().and(R::PermanentCard).and(R::ManaValueAtMost(3)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Amrou Scout",
            cost(&[generic(1), w()]),
            vec![CreatureType::Kithkin, CreatureType::Rebel, CreatureType::Scout],
            2,
            1,
        )
    }
}

/// Amrou Seekers — {2}{W} 2/2 Kithkin Rebel; blockable only by artifact
/// and/or white creatures.
pub fn amrou_seekers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(R::Artifact.or(R::HasColor(Color::White))))],
        ..creature("Amrou Seekers", cost(&[generic(2), w()]), vec![CreatureType::Kithkin, CreatureType::Rebel], 2, 2)
    }
}

/// Ramosian Revivalist — {3}{W} 2/2 Human Rebel Cleric. {6}, {T}: return a
/// Rebel permanent card with mana value 5 or less from your graveyard.
pub fn ramosian_revivalist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(rebel().and(R::PermanentCard).and(R::ManaValueAtMost(5)).from_your_graveyard()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Ramosian Revivalist",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Zealot il-Vec — {2}{W} 1/1 shadow Human Rebel. Unblocked, it may deal 1
/// damage to target creature instead of its combat damage.
pub fn zealot_il_vec() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shadow],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Deal 1 damage to target creature instead of combat damage?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
                    Effect::PreventAllDamageFromTargetThisTurn {
                        what: Selector::This,
                        gain_life: false,
                        next_instance_only: false,
                    },
                ])),
            },
        }],
        ..creature("Zealot il-Vec", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Rebel], 1, 1)
    }
}

/// Hallowed Burial — {3}{W}{W} sorcery. Every creature goes to the bottom of
/// its owner's library.
pub fn hallowed_burial() -> CardDefinition {
    CardDefinition {
        name: "Hallowed Burial",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::EachPermanent(R::Creature),
            to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Bottom },
        },
        ..Default::default()
    }
}

// ── Klaw, Sonic Subjugator ──────────────────────────────────────────────────

/// Cranial Archive — {2} artifact. {2}, exile it: target player shuffles
/// their graveyard into their library; draw a card.
pub fn cranial_archive() -> CardDefinition {
    CardDefinition {
        name: "Cranial Archive",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            exile_self_cost: true,
            effect: Effect::Seq(vec![
                // The player slot carries its filter (CR 115.1): a bare
                // `PlayerRef::Target(0)` slot is invisible to the walker.
                Effect::TargetPlayerThen {
                    filter: R::Player,
                    then: Box::new(Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::Target(0) }),
                },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Abattoir Ghoul — {3}{B} 3/2 first strike Zombie. A creature it damaged
/// this turn dying gains you life equal to that creature's toughness.
pub fn abattoir_ghoul() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::DamagedBySourceThisTurn },
            ),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..creature("Abattoir Ghoul", cost(&[generic(3), b()]), vec![CreatureType::Zombie], 3, 2)
    }
}

/// Vessel of Malignity — {1}{B} enchantment. {1}{B}, sacrifice it: target
/// opponent exiles two cards from their hand. Sorcery speed.
pub fn vessel_of_malignity() -> CardDefinition {
    CardDefinition {
        name: "Vessel of Malignity",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::ExileFromHand { who: target_filtered(R::OpponentPlayer), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Appetite for Brains — {B} sorcery. Target opponent reveals their hand;
/// you exile a card from it with mana value 4 or greater.
pub fn appetite_for_brains() -> CardDefinition {
    CardDefinition {
        name: "Appetite for Brains",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ExileChosenFromHand {
            from: target_filtered(R::OpponentPlayer),
            count: Value::ONE,
            filter: R::ManaValueAtLeast(4),
            link_to_source: false,
            face_down: false,
        },
        ..Default::default()
    }
}

/// Lay Bare the Heart — {1}{B} sorcery. Target opponent reveals their hand;
/// you choose a nonlegendary, nonland card and they discard it.
pub fn lay_bare_the_heart() -> CardDefinition {
    CardDefinition {
        name: "Lay Bare the Heart",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DiscardChosen {
            from: target_filtered(R::OpponentPlayer),
            count: Value::ONE,
            filter: R::Nonland.and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary)))),
        },
        ..Default::default()
    }
}

// ── Kytheon, Hero of Akros ──────────────────────────────────────────────────

/// Kytheon's Irregulars — {2}{W}{W} 4/3, renown 1; {W}{W}: tap target
/// creature.
pub fn kytheons_irregulars() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![renown(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), w()]),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature(
            "Kytheon's Irregulars",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            3,
        )
    }
}

/// Gideon's Intervention — {2}{W}{W} enchantment. As it enters, choose a card
/// name: opponents can't cast spells with it, and damage to you and your
/// permanents from sources with it is prevented.
pub fn gideons_intervention() -> CardDefinition {
    CardDefinition {
        name: "Gideon's Intervention",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(Effect::NameCard { what: Selector::This, restrict_to: None }),
        static_abilities: vec![
            StaticAbility {
                description: "Your opponents can't cast spells with the chosen name.",
                effect: StaticEffect::OpponentsCantCastNamed,
            },
            StaticAbility {
                description: "Prevent all damage that would be dealt to you and permanents you control by sources with the chosen name.",
                effect: StaticEffect::PreventDamageToYouAndYoursFromNamed,
            },
        ],
        ..Default::default()
    }
}

/// Gideon's Resolve — {4}{W} enchantment. ETB may find Gideon, Martial
/// Paragon in your library and/or graveyard; creatures you control get +1/+1.
pub fn gideons_resolve() -> CardDefinition {
    CardDefinition {
        name: "Gideon's Resolve",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for Gideon, Martial Paragon?".into(),
            body: Box::new(Effect::SearchZones {
                who: PlayerRef::You,
                zones: vec![Zone::Library, Zone::Graveyard],
                filter: R::HasName("Gideon, Martial Paragon".into()),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature,
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Gideon's Phalanx — {5}{W}{W} instant. Four 2/2 white Knights with
/// vigilance; spell mastery — your creatures gain indestructible this turn.
pub fn gideons_phalanx() -> CardDefinition {
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Gideon's Phalanx",
        cost: cost(&[generic(5), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(4), definition: Arc::new(knight) },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    },
                    Value::Const(2),
                ),
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Gideon, the Oathsworn — {4}{W}{W}, loyalty 4. Attacking with two or more
/// non-Gideon creatures puts a +1/+1 counter on each; +2: a 5/5 Soldier
/// this turn with damage to him prevented; −9: exile him and every creature
/// your opponents control.
pub fn gideon_the_oathsworn() -> CardDefinition {
    let non_gideon_attackers = || {
        R::Creature
            .and(R::IsAttacking)
            .and(R::ControlledByYou)
            .and(R::Not(Box::new(R::HasPlaneswalkerType(PlaneswalkerSubtype::Gideon))))
    };
    CardDefinition {
        name: "Gideon, the Oathsworn",
        cost: cost(&[generic(4), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Gideon], ..Default::default() },
        base_loyalty: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(
                Predicate::SelectorCountAtLeast { sel: Selector::EachPermanent(non_gideon_attackers()), n: Value::Const(2) },
            ),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(non_gideon_attackers()),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(5),
                        toughness: Value::Const(5),
                        creature_types: vec![CreatureType::Soldier],
                        keywords: vec![],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::PreventAllDamageThisTurn { target: Selector::This, redirect_to: None },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -9,
                effect: Effect::Seq(vec![
                    Effect::Exile { what: Selector::This },
                    Effect::Exile { what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)) },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
