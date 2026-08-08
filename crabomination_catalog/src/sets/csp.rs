//! Coldsnap (CSP) — the Ice Age block's late third set. Tests in
//! `classic_sets/csp`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{draw, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, PlayerRef, Predicate, Selector, StaticEffect, Value, ZoneDest,
};
use crate::mana::{Color, ManaCost, ManaSymbol, b, cost, g, generic, r, snow_mana, u, w, x};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// A snow creature — the supertype is what every `{S}` cost and nonsnow filter
/// in the set keys off.
fn snow_creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        ..creature(name, c, types, p, t)
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Sorcery], ..instant(name, c, effect) }
}

/// "{S}: this creature gets/gains …" — the set's signature one-pip pump.
fn snow_ability(pips: usize, effect: Effect, once_per_turn: bool) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&vec![snow_mana(); pips]),
        effect,
        once_per_turn,
        ..Default::default()
    }
}

fn pump_self(power: i32, toughness: i32) -> Effect {
    Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    }
}

fn grant_self(keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what: Selector::This, keyword, duration: Duration::EndOfTurn }
}

/// The snow tapland cycle: enters tapped, taps for either of two colours.
fn snow_dual(name: &'static str, a: Color, second: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Snow],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ChooseMode(vec![
                crate::effect::shortcut::add_mana(vec![a]),
                crate::effect::shortcut::add_mana(vec![second]),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Arctic Flats — {G} or {W}.
pub fn arctic_flats() -> CardDefinition {
    snow_dual("Arctic Flats", Color::Green, Color::White)
}

/// Boreal Shelf — {W} or {U}.
pub fn boreal_shelf() -> CardDefinition {
    snow_dual("Boreal Shelf", Color::White, Color::Blue)
}

/// Frost Marsh — {U} or {B}.
pub fn frost_marsh() -> CardDefinition {
    snow_dual("Frost Marsh", Color::Blue, Color::Black)
}

/// Highland Weald — {R} or {G}.
pub fn highland_weald() -> CardDefinition {
    snow_dual("Highland Weald", Color::Red, Color::Green)
}

/// Boreal Centaur — one snow pip a turn for +1/+1.
pub fn boreal_centaur() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![snow_ability(1, pump_self(1, 1), true)],
        ..snow_creature(
            "Boreal Centaur",
            cost(&[generic(1), g()]),
            vec![CreatureType::Centaur, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Boreal Griffin — a snow pip buys first strike.
pub fn boreal_griffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![snow_ability(1, grant_self(Keyword::FirstStrike), false)],
        ..snow_creature(
            "Boreal Griffin",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Griffin],
            3,
            2,
        )
    }
}

/// Chilling Shade — the snow-powered Shade.
pub fn chilling_shade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![snow_ability(1, pump_self(1, 1), false)],
        ..snow_creature(
            "Chilling Shade",
            cost(&[generic(2), b()]),
            vec![CreatureType::Shade],
            1,
            1,
        )
    }
}

/// Frost Raptor — two snow pips for shroud.
pub fn frost_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![snow_ability(2, grant_self(Keyword::Shroud), false)],
        ..snow_creature("Frost Raptor", cost(&[generic(2), u()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Goblin Rimerunner — a blocker-remover that can also rush in.
pub fn goblin_rimerunner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            snow_ability(1, grant_self(Keyword::Haste), false),
        ],
        ..snow_creature(
            "Goblin Rimerunner",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Adarkar Windform — grounds a flier for a snow pip and a generic.
pub fn adarkar_windform() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), snow_mana()]),
            effect: Effect::LoseKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..snow_creature(
            "Adarkar Windform",
            cost(&[generic(4), u()]),
            vec![CreatureType::Illusion],
            3,
            3,
        )
    }
}

/// Frostweb Spider — grows every time it catches something in the air.
pub fn frostweb_spider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                filter: Some(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasKeyword(Keyword::Flying),
                }),
                ..EventSpec::new(EventKind::Blocks, EventScope::SelfSource)
            },
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..snow_creature("Frostweb Spider", cost(&[generic(2), g()]), vec![CreatureType::Spider], 1, 3)
    }
}

/// Gutless Ghoul — creatures cash in for two life.
pub fn gutless_ghoul() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..snow_creature("Gutless Ghoul", cost(&[generic(2), b()]), vec![CreatureType::Zombie], 2, 2)
    }
}

/// Chill to the Bone — removal that can't touch the snow half of the set.
pub fn chill_to_the_bone() -> CardDefinition {
    instant(
        "Chill to the Bone",
        cost(&[generic(3), b()]),
        Effect::Destroy {
            what: target_filtered(R::Creature.and(R::HasSupertype(Supertype::Snow).negate())),
        },
    )
}

/// Cryoclasm — a Plains or Island, and three to its controller.
pub fn cryoclasm() -> CardDefinition {
    sorcery(
        "Cryoclasm",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(3),
            },
            Effect::Destroy {
                what: target_filtered(
                    R::HasLandType(LandType::Plains).or(R::HasLandType(LandType::Island)),
                ),
            },
        ]),
    )
}

/// Into the North — tutors the snow land the deck is built around.
pub fn into_the_north() -> CardDefinition {
    sorcery(
        "Into the North",
        cost(&[generic(1), g()]),
        Effect::Search {
            who: PlayerRef::You,
            filter: R::Land.and(R::HasSupertype(Supertype::Snow)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
    )
}

/// Kjeldoran Outrider — a white mana buys a point of toughness.
pub fn kjeldoran_outrider() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: pump_self(0, 1),
            ..Default::default()
        }],
        ..creature(
            "Kjeldoran Outrider",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Karplusan Strider — untouchable by the two colours that want it gone.
pub fn karplusan_strider() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::ProtectionFromMatching(Box::new(
                R::HasColor(Color::Blue).or(R::HasColor(Color::Black)),
            )),
        ],
        ..creature("Karplusan Strider", cost(&[generic(3), g()]), vec![CreatureType::Yeti], 3, 4)
    }
}

/// Kjeldoran Gargoyle — every point it deals comes back as life.
pub fn kjeldoran_gargoyle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..creature(
            "Kjeldoran Gargoyle",
            cost(&[generic(5), w()]),
            vec![CreatureType::Gargoyle],
            3,
            3,
        )
    }
}

/// Bull Aurochs — the herd pumps itself as it charges.
pub fn bull_aurochs() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(R::Creature)),
                    filter: R::IsAttacking
                        .and(R::HasCreatureType(CreatureType::Aurochs))
                        .and(R::OtherThanSource),
                },
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Bull Aurochs", cost(&[generic(1), g()]), vec![CreatureType::Aurochs], 2, 1)
    }
}

/// Balduvian Rage — a burst of power now and a card next upkeep.
pub fn balduvian_rage() -> CardDefinition {
    instant(
        "Balduvian Rage",
        cost(&[x(), r()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                power: Value::XFromCost,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::DelayUntil {
                kind: crate::effect::DelayedTriggerKind::YourNextUpkeep,
                body: Box::new(draw(1)),
            },
        ]),
    )
}

/// Disciple of Tevesh Szat — a repeatable shrink, or one big one on the way out.
pub fn disciple_of_tevesh_szat() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(4), b(), b()]),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-6),
                    toughness: Value::Const(-6),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Disciple of Tevesh Szat",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            3,
            1,
        )
    }
}

/// Drelnoch — blocking it hands you two cards.
pub fn drelnoch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Draw two cards?".into(),
                body: Box::new(draw(2)),
            },
        }],
        ..creature(
            "Drelnoch",
            cost(&[generic(4), u()]),
            vec![CreatureType::Yeti, CreatureType::Mutant],
            3,
            3,
        )
    }
}

/// Karplusan Wolverine — pings whatever gets in its way.
pub fn karplusan_wolverine() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Have Karplusan Wolverine deal 1 damage?".into(),
                body: Box::new(Effect::DealDamage { to: target_any(), amount: Value::ONE }),
            },
        }],
        ..snow_creature(
            "Karplusan Wolverine",
            cost(&[r()]),
            vec![CreatureType::Wolverine, CreatureType::Beast],
            1,
            1,
        )
    }
}

/// Earthen Goo — grows with every upkeep it survives.
pub fn earthen_goo() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Trample,
            Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(ManaCost::new(vec![
                ManaSymbol::Hybrid(Color::Red, Color::Green),
            ]))),
        ],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each age counter on it.",
            effect: StaticEffect::PumpPTPerCounterOnSource {
                applies_to: Selector::This,
                kind: CounterType::Age,
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..creature("Earthen Goo", cost(&[generic(2), r()]), vec![CreatureType::Ooze], 2, 2)
    }
}

/// Arctic Nishoba — the longer it lives, the more it pays out.
pub fn arctic_nishoba() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Trample,
            Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(ManaCost::new(vec![
                ManaSymbol::Hybrid(Color::Green, Color::White),
            ]))),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Times(
                    Box::new(Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Age,
                    }),
                    Box::new(Value::Const(2)),
                ),
            },
        }],
        ..creature(
            "Arctic Nishoba",
            cost(&[generic(5), g()]),
            vec![CreatureType::Cat, CreatureType::Warrior],
            6,
            6,
        )
    }
}

/// Jötun Owl Keeper — leaves a bird for every upkeep it paid.
pub fn jotun_owl_keeper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(ManaCost::new(
            vec![ManaSymbol::Hybrid(Color::White, Color::Blue)],
        )))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Age,
                },
                definition: Box::new(TokenDefinition {
                    name: "Bird".to_string(),
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Bird],
                        ..Default::default()
                    },
                    power: 1,
                    toughness: 1,
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                }),
            },
        }],
        ..creature("Jötun Owl Keeper", cost(&[generic(2), w()]), vec![CreatureType::Giant], 3, 3)
    }
}

/// Gelid Shackles — a one-mana lock, with a snow pip to shut off attacks too.
pub fn gelid_shackles() -> CardDefinition {
    CardDefinition {
        name: "Gelid Shackles",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Snow],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantBlock, Keyword::CantActivateTapAbilities],
            ..Default::default()
        }),
        activated_abilities: vec![snow_ability(
            1,
            Effect::GrantKeyword {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                keyword: Keyword::Defender,
                duration: Duration::EndOfTurn,
            },
            false,
        )],
        ..Default::default()
    }
}

/// Freyalise's Radiance — the snow hoser; its own upkeep is the clock.
pub fn freyalises_radiance() -> CardDefinition {
    CardDefinition {
        name: "Freyalise's Radiance",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[generic(2)])))],
        static_abilities: vec![StaticAbility {
            description: "Snow permanents don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(R::HasSupertype(Supertype::Snow)),
            },
        }],
        ..Default::default()
    }
}



/// Kjeldoran Javelineer — its age counters are its ammunition.
pub fn kjeldoran_javelineer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Age,
                },
            },
            ..Default::default()
        }],
        ..creature(
            "Kjeldoran Javelineer",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            2,
        )
    }
}

/// Feast of Flesh — every copy already binned makes the next one bigger.
pub fn feast_of_flesh() -> CardDefinition {
    let x = || {
        Value::Sum(vec![Value::ONE, Value::CardsNamedLikeSourceInAllGraveyards])
    };
    sorcery(
        "Feast of Flesh",
        cost(&[b()]),
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: x() },
            Effect::GainLife { who: Selector::You, amount: x() },
        ]),
    )
}

/// Kjeldoran War Cry — the same escalating-copies trick, as a team pump.
pub fn kjeldoran_war_cry() -> CardDefinition {
    instant(
        "Kjeldoran War Cry",
        cost(&[generic(1), w()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::Sum(vec![Value::ONE, Value::CardsNamedLikeSourceInAllGraveyards]),
            toughness: Value::Sum(vec![Value::ONE, Value::CardsNamedLikeSourceInAllGraveyards]),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Balduvian Fallen — the upkeep it pays powers the swing.
pub fn balduvian_fallen() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        ..creature("Balduvian Fallen", cost(&[generic(3), b()]), vec![CreatureType::Zombie], 3, 5)
    }
}
