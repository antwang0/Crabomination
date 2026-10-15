//! Commander: two-card gaps of the 2026-10-15 full-list EDHREC scan, fourth
//! batch. Tests in `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{afflict, etb, extort, on_you_attack, target_filtered, unearth};
use crate::effect::{
    Duration, Effect, LibraryPosition, LookPick, ManaPayload, PlayerRef, Selector, ZoneDest,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, w};
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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn mill(n: i32) -> Effect {
    Effect::Mill { who: Selector::You, amount: Value::Const(n) }
}

/// The Kingpin of Crime — {1}{W}{B} legendary 1/5 Human Villain; extort.
/// Whenever you attack, you may pay 2 life; if you do, until end of turn your
/// creatures with toughness greater than power assign combat damage by
/// toughness. (The set is fixed as the trigger resolves.)
pub fn the_kingpin_of_crime() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            extort(),
            on_you_attack(Effect::MayPayLife {
                description: "Pay 2 life so your sturdier creatures deal damage by toughness?".into(),
                amount: Value::Const(2),
                body: Box::new(Effect::GrantKeyword {
                    what: yours(R::Creature.and(R::ToughnessGreaterThanPower)),
                    keyword: Keyword::AssignsCombatDamageByToughness,
                    duration: Duration::EndOfTurn,
                }),
                else_: None,
            }),
        ],
        ..legend("The Kingpin of Crime", cost(&[generic(1), w(), b()]), vec![CreatureType::Human, CreatureType::Villain], 1, 5)
    }
}

/// Fisk Tower — land; enters tapped; ETB gain 1 life; {T}: {W} or {B}.
pub fn fisk_tower() -> CardDefinition {
    let tap = |c: Color| ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![c]) },
        ..Default::default()
    };
    CardDefinition {
        name: "Fisk Tower",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::ONE })],
        activated_abilities: vec![tap(Color::White), tap(Color::Black)],
        ..Default::default()
    }
}

/// Liliana the Repentant — {1}{B} legendary 2/2 Human Warlock. Whenever
/// another creature or planeswalker you control enters, mill two. Exhaust —
/// {5}{B}: return target creature or planeswalker card from your graveyard to
/// the battlefield; a +1/+1 counter on Liliana (sorcery).
pub fn liliana_the_repentant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: (R::Creature.or(R::Planeswalker)).and(R::OtherThanSource),
                },
            ),
            effect: mill(2),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b()]),
            exhaust: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered((R::Creature.or(R::Planeswalker)).and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..legend("Liliana the Repentant", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Warlock], 2, 2)
    }
}

/// Dark Matter Manipulator — {B} 1/2 Human Warlock. ETB mill three. +2/+0
/// for every seven cards in your graveyard. (Liliana the Repentant.)
pub fn dark_matter_manipulator() -> CardDefinition {
    let sevens = Value::DivDown(Box::new(Value::GraveyardSizeOf(PlayerRef::You)), 7);
    CardDefinition {
        triggered_abilities: vec![etb(mill(3))],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +2/+0 for every seven cards in your graveyard.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::This,
                power: Value::Times(Box::new(sevens), Box::new(Value::Const(2))),
                toughness: Value::Const(0),
            },
        }],
        ..creature("Dark Matter Manipulator", cost(&[b()]), vec![CreatureType::Human, CreatureType::Warlock], 1, 2)
    }
}

/// Aang, A Lot to Learn — {2}{G/W} legendary 3/2 Human Avatar Ally;
/// vigilance while a Lesson card is in your graveyard; whenever another
/// creature you control dies, a +1/+1 counter on him.
pub fn aang_a_lot_to_learn() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Aang has vigilance as long as there's a Lesson card in your graveyard.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Vigilance,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::HasSpellSubtype(crate::card::SpellSubtype::Lesson),
                    },
                    n: Value::ONE,
                },
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::OtherThanSource,
            }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legend(
            "Aang, A Lot to Learn",
            cost(&[generic(2), hybrid(Color::Green, Color::White)]),
            vec![CreatureType::Human, CreatureType::Avatar, CreatureType::Ally],
            3,
            2,
        )
    }
}

/// Algae Gharial — {3}{G} 1/1 Crocodile; shroud; whenever another creature
/// dies, you may put a +1/+1 counter on it. (Aang, A Lot to Learn.)
pub fn algae_gharial() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shroud],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::OtherThanSource,
            }),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on Algae Gharial?".into(),
                body: Box::new(Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
            },
        }],
        ..creature("Algae Gharial", cost(&[generic(3), g()]), vec![CreatureType::Crocodile], 1, 1)
    }
}

/// Pathmaker Initiate — {1}{R} 2/1 Human Wizard. {T}: target creature with
/// power 2 or less can't be blocked this turn. (Mannichi.)
pub fn pathmaker_initiate() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Pathmaker Initiate", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 1)
    }
}

/// Wildfire Eternal — {3}{R} 1/4 Zombie Jackal Cleric; afflict 4. Whenever it
/// attacks and isn't blocked, you may cast an instant or sorcery spell from
/// your hand without paying its mana cost. (Mannichi.)
pub fn wildfire_eternal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            afflict(4),
            TriggeredAbility {
                event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
                effect: Effect::MayCastFromHandFreeMatching {
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    max_mv: Value::Const(99),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..creature(
            "Wildfire Eternal",
            cost(&[generic(3), r()]),
            vec![CreatureType::Zombie, CreatureType::Jackal, CreatureType::Cleric],
            1,
            4,
        )
    }
}

/// Leech Gauntlet — {1}{B} 2/2 Equipment Leech artifact creature; lifelink;
/// equipped creature has lifelink; reconfigure {4}. (Naomi, Pillar of Order.)
pub fn leech_gauntlet() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Leech],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Lifelink, Keyword::Reconfigure(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Lifelink], ..Default::default() }),
        ..creature("Leech Gauntlet", cost(&[generic(1), b()]), vec![], 2, 2)
    }
}

/// Twisted Embrace — {2}{B}{B} Aura on an artifact or creature you control.
/// ETB: destroy target creature or planeswalker an opponent controls. A
/// creature it enchants gets +1/+1. (Naomi, Pillar of Order.)
pub fn twisted_embrace() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Twisted Embrace",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered((R::HasCardType(CardType::Artifact).or(R::Creature)).and(R::ControlledByYou)),
        },
        static_abilities: vec![StaticAbility {
            description: "As long as enchanted permanent is a creature, it gets +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: host(), power: 1, toughness: 1 },
        }],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered((R::Creature.or(R::Planeswalker)).and(R::ControlledByOpponent)),
        })],
        ..Default::default()
    }
}

/// Loyal Tutor — {W} instant: a planeswalker card from your library onto the
/// top after shuffling. (Basri, Tomorrow's Champion.)
pub fn loyal_tutor() -> CardDefinition {
    CardDefinition {
        name: "Loyal Tutor",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::Planeswalker,
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
        },
        ..Default::default()
    }
}

/// Worzel, the Protector — {1}{W}{W}{W} planeswalker, loyalty 4; can be your
/// commander. 0: a 1/1 white Cat with "{T}: a loyalty counter on each
/// planeswalker you control." −1: look at the top six, a planeswalker or
/// basic Plains card to hand, the rest on the bottom. −8: ten 1/1 green
/// flying Faerie Scryb Sprites.
pub fn worzel_the_protector() -> CardDefinition {
    let cat = TokenDefinition {
        name: "Cat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddCounter {
                what: yours(R::Planeswalker),
                kind: CounterType::Loyalty,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    let sprite = TokenDefinition {
        name: "Scryb Sprites".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Worzel, the Protector",
        cost: cost(&[generic(1), w(), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Worzel], ..Default::default() },
        base_loyalty: 4,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(cat) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::LookPickToHand(Box::new(LookPick {
                    who: PlayerRef::You,
                    count: Value::Const(6),
                    pick_filter: Some(R::Planeswalker.or(R::IsBasicLand.and(R::HasLandType(LandType::Plains)))),
                    optional: true,
                    ..Default::default()
                })),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(10), definition: Arc::new(sprite) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Vithian Stinger — {2}{R} 0/1 Human Shaman. {T}: 1 damage to any target.
/// Unearth {1}{R}. (Red Hulk.)
pub fn vithian_stinger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::ONE },
                ..Default::default()
            },
            unearth(cost(&[generic(1), r()])),
        ],
        ..creature("Vithian Stinger", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Shaman], 0, 1)
    }
}

/// Photon Blast Barrage — {X}{R}{R} sorcery. When you cast it, copy it X
/// times (new targets allowed). 1 damage to target creature. (Red Hulk.)
pub fn photon_blast_barrage() -> CardDefinition {
    CardDefinition {
        name: "Photon Blast Barrage",
        cost: cost(&[crate::mana::x(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        copies_on_cast_x: true,
        effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
        ..Default::default()
    }
}
