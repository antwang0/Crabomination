//! Commander: EDHREC ranks 2001-2500, one-card gaps (third batch) — Sigarda
//! Champion of Light, Shang-Chi, Lyzolda, Multani / Jolrael, Pashalik Mons,
//! Sabin, Rassilon, Tori D'Avenant, Shanna, Whisper, Márton Stromgald,
//! Braulios, Imaryll and Frodo. Tests in `tests/recent_b/cmdr_edhrec57.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, exalted, on_attack, target_filtered, unearth};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color, ManaCost};
use std::sync::Arc;

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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn token(name: &str, color: Color, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: 1,
        toughness: 1,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn lands_you_control() -> Value {
    Value::count(Selector::ControlledBy { who: PlayerRef::You, filter: R::Land })
}

/// Archangel Elspeth — {2}{W}{W} legendary planeswalker Elspeth, loyalty 4.
/// +1: a 1/1 white Soldier with lifelink. −2: two +1/+1 counters on target
/// creature; it becomes an Angel too and gains flying. −6: return every
/// nonland permanent card with mana value 3 or less from your graveyard to
/// the battlefield.
pub fn archangel_elspeth() -> CardDefinition {
    CardDefinition {
        name: "Archangel Elspeth",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Elspeth], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(token("Soldier", Color::White, vec![CreatureType::Soldier], vec![Keyword::Lifelink])),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(R::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                    Effect::AddCreatureTypes {
                        what: Selector::Target(0),
                        creature_types: vec![CreatureType::Angel],
                        duration: Duration::Permanent,
                    },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::Permanent },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::Move {
                    what: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: R::Permanent.and(R::Land.negate()).and(R::ManaValueAtMost(3)),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Magus of the Order — {2}{G}{G} 3/3 Human Wizard. {G}, {T}, sacrifice it and
/// another green creature: search your library for a green creature card, put
/// it onto the battlefield, then shuffle.
pub fn magus_of_the_order() -> CardDefinition {
    let green = || R::Creature.and(R::HasColor(Color::Green));
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            sac_cost: true,
            sac_other_filter: Some((green().and(R::OtherThanSource), 1)),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: green(),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature("Magus of the Order", cost(&[generic(2), g(), g()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 3)
    }
}

/// Manaforge Cinder — {B/R} 1/1 Elemental Shaman. {1}: add {B} or {R};
/// activate no more than three times each turn.
pub fn manaforge_cinder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            max_activations_per_turn: Some(3),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![Color::Black, Color::Red], Value::ONE) },
            ..Default::default()
        }],
        ..creature("Manaforge Cinder", cost(&[hybrid(Color::Black, Color::Red)]), vec![CreatureType::Elemental, CreatureType::Shaman], 1, 1)
    }
}

/// Boundless Realms — {6}{G} sorcery. Search for up to X basic land cards, X
/// the number of lands you control, put them onto the battlefield tapped,
/// then shuffle.
pub fn boundless_realms() -> CardDefinition {
    CardDefinition {
        name: "Boundless Realms",
        cost: cost(&[generic(6), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::Land.and(R::HasSupertype(Supertype::Basic)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            count: lands_you_control(),
        },
        ..Default::default()
    }
}

/// Goblin Warrens — {2}{R} enchantment. {2}{R}, sacrifice two Goblins: create
/// three 1/1 red Goblins.
pub fn goblin_warrens() -> CardDefinition {
    CardDefinition {
        name: "Goblin Warrens",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Goblin), 2)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: Arc::new(token("Goblin", Color::Red, vec![CreatureType::Goblin], vec![])),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cathedral of War — land, enters tapped. Exalted. {T}: add {C}.
pub fn cathedral_of_war() -> CardDefinition {
    CardDefinition {
        name: "Cathedral of War",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![exalted()],
        activated_abilities: vec![crate::sets::tap_add_colorless()],
        ..Default::default()
    }
}

/// Depart the Realm — {1}{U} instant. Return target nonland permanent to its
/// owner's hand. Foretell {U}.
pub fn depart_the_realm() -> CardDefinition {
    CardDefinition {
        name: "Depart the Realm",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        foretell_cost: Some(cost(&[u()])),
        effect: Effect::Move {
            what: target_filtered(R::Permanent.and(R::Land.negate())),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        ..Default::default()
    }
}

/// Honored Crop-Captain — {R}{W} 3/2 Human Warrior. Whenever it attacks, other
/// attacking creatures get +1/+0 until end of turn.
pub fn honored_crop_captain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
            power: Value::ONE,
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Honored Crop-Captain", cost(&[r(), w()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 2)
    }
}

/// Ironroot Warlord — {1}{G}{W} */5 Treefolk Soldier, power the number of
/// creatures you control. {3}{G}{W}: create a 1/1 white Soldier.
pub fn ironroot_warlord() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Ironroot Warlord's power is equal to the number of creatures you control.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: Value::count(Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature }),
                toughness: Value::Const(5),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g(), w()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(token("Soldier", Color::White, vec![CreatureType::Soldier], vec![])),
            },
            ..Default::default()
        }],
        ..creature("Ironroot Warlord", cost(&[generic(1), g(), w()]), vec![CreatureType::Treefolk, CreatureType::Soldier], 0, 5)
    }
}

/// Corpse Connoisseur — {4}{B} 3/3 Zombie Wizard. Enters: you may search for a
/// creature card and put it into your graveyard. Unearth {3}{B}.
pub fn corpse_connoisseur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Search { who: PlayerRef::You, filter: R::Creature, to: ZoneDest::Graveyard })],
        activated_abilities: vec![unearth(cost(&[generic(3), b()]))],
        ..creature("Corpse Connoisseur", cost(&[generic(4), b()]), vec![CreatureType::Zombie, CreatureType::Wizard], 3, 3)
    }
}

/// Márton Stromgald — {2}{R}{R} legendary 1/1 Human Knight. Attacks: other
/// attackers get +1/+1 for each attacking creature other than Márton. Blocks:
/// the same for other blockers.
pub fn marton_stromgald() -> CardDefinition {
    let rally = |state: fn() -> R| {
        let others = || R::Creature.and(state()).and(R::OtherThanSource);
        let n = || Value::count(Selector::EachPermanent(others()));
        Effect::PumpPT { what: Selector::EachPermanent(others()), power: n(), toughness: n(), duration: Duration::EndOfTurn }
    };
    CardDefinition {
        triggered_abilities: vec![
            on_attack(rally(|| R::IsAttacking)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: rally(|| R::IsBlocking),
            },
        ],
        ..legend("Márton Stromgald", cost(&[generic(2), r(), r()]), vec![CreatureType::Human, CreatureType::Knight], 1, 1)
    }
}

/// Braulios of Pheres Band — {3}{G}{G} legendary */* Centaur Scout, P/T each
/// the number of lands you control. Attacks: draw a card, then you may put a
/// land card from your hand onto the battlefield.
pub fn braulios_of_pheres_band() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Braulios's power and toughness are each equal to the number of lands you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: lands_you_control(), toughness: lands_you_control() },
        }],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Land,
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
        ]))],
        ..legend(
            "Braulios of Pheres Band",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Centaur, CreatureType::Scout],
            0,
            0,
        )
    }
}

/// Imaryll, Elfhame Elite — {2}{G}{G} legendary 3/3 Elf Warrior. Attacks: +X/+X
/// until end of turn, X the number of other Elves you control. It may assign
/// its combat damage as though it weren't blocked.
pub fn imaryll_elfhame_elite() -> CardDefinition {
    let elves = || {
        Value::count(Selector::ControlledBy {
            who: PlayerRef::You,
            filter: R::HasCreatureType(CreatureType::Elf).and(R::OtherThanSource),
        })
    };
    CardDefinition {
        keywords: vec![Keyword::AssignsDamageAsThoughUnblocked],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: elves(),
            toughness: elves(),
            duration: Duration::EndOfTurn,
        })],
        ..legend("Imaryll, Elfhame Elite", cost(&[generic(2), g(), g()]), vec![CreatureType::Elf, CreatureType::Warrior], 3, 3)
    }
}

/// Frodo, Determined Hero — {1}{W} legendary 2/2 Halfling Warrior. Enters or
/// attacks: you may attach target Equipment you control with mana value 2 or
/// 3 to it. During your turn, prevent all damage that would be dealt to it.
pub fn frodo_determined_hero() -> CardDefinition {
    let gear = || {
        Effect::Attach {
            what: target_filtered(
                R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment)
                    .and(R::ControlledByYou)
                    .and(R::ManaValueAtLeast(2))
                    .and(R::ManaValueAtMost(3)),
            ),
            to: Selector::This,
        }
    };
    CardDefinition {
        triggered_abilities: vec![etb(gear()), on_attack(gear())],
        static_abilities: vec![StaticAbility {
            description: "During your turn, prevent all damage that would be dealt to Frodo.",
            effect: StaticEffect::WhileYourTurn { inner: Box::new(StaticEffect::PreventAllDamageToThis) },
        }],
        ..legend("Frodo, Determined Hero", cost(&[generic(1), w()]), vec![CreatureType::Halfling, CreatureType::Warrior], 2, 2)
    }
}
