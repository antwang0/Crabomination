//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat, fourth file (`cmdr_edhrec3` is the
//! third). Shroofus Sproutsire, Miirym, Sentinel Wyrm and Tiamat. Tests in
//! `tests/recent_b/cmdr_edhrec4.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::game::TurnStep;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color, SpendRestriction};
use std::sync::Arc;

fn creature(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn legend(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn saproling() -> TokenDefinition {
    TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    }
}

fn saprolings(n: i32) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(saproling()) }
}

fn is_saproling() -> R {
    R::HasCreatureType(CreatureType::Saproling)
}

// ── Shroofus Sproutsire (BG) ─────────────────────────────────────────────────

/// "At the beginning of your upkeep, put a spore counter on this creature."
fn spore_upkeep() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
        effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Spore, amount: Value::ONE },
    }
}

/// "Remove three spore counters from this creature: Create a 1/1 green
/// Saproling creature token."
fn spore_saproling() -> ActivatedAbility {
    ActivatedAbility {
        remove_counter_cost: Some((CounterType::Spore, 3)),
        effect: saprolings(1),
        ..Default::default()
    }
}

/// "Sacrifice a Saproling: [effect]" (`mana` adds `{mana}` to the cost).
fn sac_saproling(n: u32, mana: crate::mana::ManaCost, effect: Effect) -> ActivatedAbility {
    ActivatedAbility { mana_cost: mana, sac_other_filter: Some((is_saproling(), n)), effect, ..Default::default() }
}

/// A Thallid: spore counters each upkeep, three of them for a Saproling,
/// plus its own Saproling outlet.
fn thallid(name: &'static str, mana: crate::mana::ManaCost, p: i32, t: i32, outlet: ActivatedAbility) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![spore_upkeep()],
        activated_abilities: vec![spore_saproling(), outlet],
        ..creature(name, mana, vec![CreatureType::Fungus], p, t)
    }
}

/// Psychotrope Thallid — {2}{G} 1/1 Fungus. {1}, sacrifice a Saproling: draw.
pub fn psychotrope_thallid() -> CardDefinition {
    thallid(
        "Psychotrope Thallid",
        cost(&[generic(2), g()]),
        1,
        1,
        sac_saproling(1, cost(&[generic(1)]), Effect::Draw { who: Selector::You, amount: Value::ONE }),
    )
}

/// Thallid Germinator — {2}{G} 2/2 Fungus. Sacrifice a Saproling: target
/// creature gets +1/+1 until end of turn.
pub fn thallid_germinator() -> CardDefinition {
    thallid(
        "Thallid Germinator",
        cost(&[generic(2), g()]),
        2,
        2,
        sac_saproling(
            1,
            crate::mana::ManaCost::default(),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        ),
    )
}

/// Utopia Mycon — {G} 0/2 Fungus. Sacrifice a Saproling: add one mana of
/// any color (a mana ability, CR 605.1a).
pub fn utopia_mycon() -> CardDefinition {
    thallid(
        "Utopia Mycon",
        cost(&[g()]),
        0,
        2,
        sac_saproling(
            1,
            crate::mana::ManaCost::default(),
            Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
        ),
    )
}

/// Undercellar Myconid — {2}{G} 1/2 Fungus. Entering or dying, a Saproling.
/// {T}: add one mana of any color.
pub fn undercellar_myconid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(saprolings(1)),
            TriggeredAbility { event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource), effect: saprolings(1) },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..creature("Undercellar Myconid", cost(&[generic(2), g()]), vec![CreatureType::Fungus], 1, 2)
    }
}

/// Fungal Plots — {1}{G} enchantment. {1}{G}, exile a creature card from your
/// graveyard: a Saproling. Sacrifice two Saprolings: gain 2 life and draw.
pub fn fungal_plots() -> CardDefinition {
    CardDefinition {
        name: "Fungal Plots",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), g()]),
                exile_other_filter: Some((R::Creature, 1)),
                effect: saprolings(1),
                ..Default::default()
            },
            sac_saproling(
                2,
                crate::mana::ManaCost::default(),
                Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Might of the Masses — {G} instant. Target creature gets +1/+1 until end of
/// turn for each creature you control (counted on resolution).
pub fn might_of_the_masses() -> CardDefinition {
    let n = || Value::PermanentCountControlledByMatching(PlayerRef::You, R::Creature);
    CardDefinition {
        name: "Might of the Masses",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT { what: target_filtered(R::Creature), power: n(), toughness: n(), duration: Duration::EndOfTurn },
        ..Default::default()
    }
}

// ── Miirym, Sentinel Wyrm and Tiamat (Dragons) ───────────────────────────────

fn dragon() -> R {
    R::HasCreatureType(CreatureType::Dragon)
}

fn your_dragons() -> Selector {
    Selector::EachPermanent(R::Creature.and(dragon()).and(R::ControlledByYou))
}

fn red_dragon_4_4() -> TokenDefinition {
    TokenDefinition {
        name: "Dragon".into(),
        power: 4,
        toughness: 4,
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// "{T}: Add {G}" whose mana carries `rider` (CR 106.6).
fn green_with_rider(rider: SpendRestriction) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Restricted(Box::new(ManaPayload::OfColor(Color::Green, Value::ONE)), rider),
        },
        ..Default::default()
    }
}

/// Jade Orb of Dragonkind — {2}{G} artifact. {T}: Add {G}; a Dragon creature
/// spell it pays for enters with an additional +1/+1 counter and gains
/// hexproof until your next turn.
pub fn jade_orb_of_dragonkind() -> CardDefinition {
    CardDefinition {
        name: "Jade Orb of Dragonkind",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![green_with_rider(SpendRestriction::DragonCreatureCounterHexproof)],
        ..Default::default()
    }
}

/// Scaled Nurturer — {1}{G} 0/2 Dragon Druid. {T}: Add {G}; a Dragon creature
/// spell it pays for gains you 2 life.
pub fn scaled_nurturer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![green_with_rider(SpendRestriction::DragonCreatureGainTwo)],
        ..creature("Scaled Nurturer", cost(&[generic(1), g()]), vec![CreatureType::Dragon, CreatureType::Druid], 0, 2)
    }
}

/// Earthquake Dragon — {14}{G} 10/10 Elemental Dragon, flying, trample.
/// Costs {X} less, X = the total mana value of Dragons you control. {2}{G},
/// sacrifice a land: return it from your graveyard to your hand.
pub fn earthquake_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast, where X is the total mana value of Dragons you control.",
            effect: StaticEffect::SelfCostReducedByValue {
                amount: Value::TotalManaValueOf(Box::new(Selector::EachPermanent(dragon().and(R::ControlledByYou)))),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            from_graveyard: true,
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..creature(
            "Earthquake Dragon",
            cost(&[generic(14), g()]),
            vec![CreatureType::Elemental, CreatureType::Dragon],
            10,
            10,
        )
    }
}

/// Korlessa, Scale Singer — {G}{U} 1/4 legendary Dragon Bard. You may look
/// at the top card of your library any time and cast Dragon spells from it.
pub fn korlessa_scale_singer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may cast Dragon spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: dragon().and(R::Nonland) },
            },
        ],
        ..legend("Korlessa, Scale Singer", cost(&[g(), u()]), vec![CreatureType::Dragon, CreatureType::Bard], 1, 4)
    }
}

/// Thrakkus the Butcher — {3}{R}{G} 3/4 legendary Dragon Peasant, trample.
/// Attacking, it doubles the power of each Dragon you control until end of
/// turn (CR 701.10).
pub fn thrakkus_the_butcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DoublePower { what: your_dragons(), times: Value::ONE, duration: Duration::EndOfTurn },
        }],
        ..legend(
            "Thrakkus the Butcher",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::Dragon, CreatureType::Peasant],
            3,
            4,
        )
    }
}

/// Wrathful Red Dragon — {3}{R}{R} 5/5 Dragon, flying. A Dragon you control
/// dealt damage deals that much to any target that isn't a Dragon.
pub fn wrathful_red_dragon() -> CardDefinition {
    let not_dragon = || dragon().negate();
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: dragon() }),
            effect: Effect::DealDamageFrom {
                source: Selector::TriggerSource,
                to: target_filtered(
                    R::Creature.and(not_dragon()).or(R::Player).or(R::Planeswalker.and(not_dragon())),
                ),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature("Wrathful Red Dragon", cost(&[generic(3), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Sarkhan Unbroken — {2}{G}{U}{R} planeswalker, loyalty 4. +1: draw, then
/// add one mana of any color. −2: a 4/4 red flying Dragon. −8: search for
/// any number of Dragon creature cards and put them onto the battlefield.
pub fn sarkhan_unbroken() -> CardDefinition {
    let ability = |loyalty_cost: i32, effect: Effect| LoyaltyAbility { loyalty_cost, effect, ..Default::default() };
    CardDefinition {
        name: "Sarkhan Unbroken",
        cost: cost(&[generic(2), g(), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Sarkhan], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            ability(
                1,
                Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ]),
            ),
            ability(
                -2,
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(red_dragon_4_4()) },
            ),
            ability(
                -8,
                Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: R::Creature.and(dragon()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    count: Value::Const(100),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Ancient Gold Dragon — {5}{W}{W} 7/10 Elder Dragon, flying. Its combat
/// damage to a player rolls a d20: that many 1/1 blue flying Faerie Dragons.
pub fn ancient_gold_dragon() -> CardDefinition {
    let faerie_dragon = TokenDefinition {
        name: "Faerie Dragon".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Blue],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie, CreatureType::Dragon], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
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
                    Effect::CreateToken { who: PlayerRef::You, count: Value::LastDieRoll, definition: Arc::new(faerie_dragon) },
                )],
                ignore_lowest: 0,
                on_doubles: None,
            },
        }],
        ..creature(
            "Ancient Gold Dragon",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Elder, CreatureType::Dragon],
            7,
            10,
        )
    }
}

/// Rith, Liberated Primeval — {2}{R}{G}{W} 5/5 legendary Dragon, flying,
/// ward {2}; other Dragons you control have ward {2}. At your end step, a
/// 4/4 Dragon if an opponent's creature or planeswalker was dealt excess
/// damage this turn (CR 120.10).
pub fn rith_liberated_primeval() -> CardDefinition {
    let ward = || Keyword::Ward(crate::card::WardCost::generic(2));
    CardDefinition {
        keywords: vec![Keyword::Flying, ward()],
        static_abilities: vec![StaticAbility {
            description: "Other Dragons you control have ward {2}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(dragon().and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: ward(),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::ExcessDamagedPermanentThisTurn { who: PlayerRef::EachOpponent }),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(red_dragon_4_4()) },
        }],
        ..legend(
            "Rith, Liberated Primeval",
            cost(&[generic(2), r(), g(), w()]),
            vec![CreatureType::Dragon],
            5,
            5,
        )
    }
}

/// Rivaz of the Claw — {1}{B}{R} 3/3 legendary Lizard Warlock, menace.
/// {T}: two mana in any combination of colors for Dragon creature spells.
/// Once during each of your turns, cast a Dragon creature spell from your
/// graveyard; a Dragon creature spell you cast from your graveyard gains
/// "When this creature dies, exile it."
pub fn rivaz_of_the_claw() -> CardDefinition {
    let dragon_creature = || R::Creature.and(dragon());
    CardDefinition {
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyColors(Value::Const(2))),
                    SpendRestriction::CreatureSpellOfType(CreatureType::Dragon),
                ),
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Once during each of your turns, you may cast a Dragon creature spell from your graveyard.",
            effect: StaticEffect::GraveyardCastOncePerTurn {
                filter: dragon_creature(),
                exile_after: false,
                mv_at_most_counters: None,
                enters_tapped: false,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(dragon_creature()),
                Predicate::CastFromGraveyard,
            ])),
            effect: Effect::SpellGainsTrigger {
                what: Selector::TriggerSource,
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                    effect: Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                }),
            },
        }],
        ..legend(
            "Rivaz of the Claw",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Lizard, CreatureType::Warlock],
            3,
            3,
        )
    }
}

/// Dragon's Fire — {1}{R} instant. 3 damage to target creature or
/// planeswalker; with a Dragon beheld as the optional additional cost, that
/// Dragon's power instead (CR 701.67's behold).
pub fn dragons_fire() -> CardDefinition {
    CardDefinition {
        name: "Dragon's Fire",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::BeholdPower { filter: dragon(), at_least: 4 }],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.or(R::Planeswalker)),
            amount: Value::Max(Box::new(Value::Const(3)), Box::new(Value::XFromCost)),
        },
        ..Default::default()
    }
}
