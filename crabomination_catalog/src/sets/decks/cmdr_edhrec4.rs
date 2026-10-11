//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat, fourth file (`cmdr_edhrec3` is the
//! third). Shroofus Sproutsire, Miirym, Sentinel Wyrm, Tiamat, Wolverine,
//! Captain Howler and Magda. Tests in `tests/recent_b/cmdr_edhrec4.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::game::TurnStep;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color, SpendRestriction};
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

// ── Wolverine, Best There Is (RG Mutants and fights) ─────────────────────────

fn mutant_hero(name: &'static str, mana: crate::mana::ManaCost, p: i32, t: i32) -> CardDefinition {
    legend(name, mana, vec![CreatureType::Mutant, CreatureType::Berserker, CreatureType::Hero], p, t)
}

/// "Double all damage [this] would deal" (CR 614.2): the static's own
/// permanent is the filter's source, so "not other than the source" is it.
fn doubles_own_damage(description: &'static str) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::DoubleDamageFromControlledMatching { filter: R::OtherThanSource.negate() },
    }
}

/// Wolverine, Best There Is — {1}{R}{G} 2/2 legendary Mutant Berserker Hero.
/// Doubles all damage it would deal; at each end step, a +1/+1 counter if it
/// dealt damage to another creature this turn; {1}{G}: regenerate.
pub fn wolverine_best_there_is() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![doubles_own_damage("Double all damage Wolverine would deal.")],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::SourceDamagedAnotherCreatureThisTurn),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..mutant_hero("Wolverine, Best There Is", cost(&[generic(1), r(), g()]), 2, 2)
    }
}

/// Mjölnir, Hammer of Thor — {3}{R} legendary Equipment. Entering, 4 damage
/// to up to one target creature; equipped creature deals double damage;
/// equip worthy {1} (a legendary non-Villain that's red and/or white);
/// {2}{R}, discard it: 2 damage to each creature.
pub fn mjolnir_hammer_of_thor() -> CardDefinition {
    let worthy = R::Creature
        .and(R::HasSupertype(Supertype::Legendary))
        .and(R::HasCreatureType(CreatureType::Villain).negate())
        .and(R::HasColor(Color::Red).or(R::HasColor(Color::White)));
    CardDefinition {
        name: "Mjölnir, Hammer of Thor",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        equip_filtered_cost: Some((worthy, cost(&[generic(1)]))),
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) }),
        })],
        static_abilities: vec![StaticAbility {
            description: "Double all damage equipped creature would deal.",
            effect: StaticEffect::AttachedDealsDoubleDamage,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Neyith of the Dire Hunt — {2}{G}{G} 3/3 legendary Human Warrior. One or
/// more of your creatures fighting or becoming blocked draws a card. At the
/// beginning of combat on your turn, you may pay {2}{R/G}: double target
/// creature's power until end of turn, and it must be blocked this combat.
pub fn neyith_of_the_dire_hunt() -> CardDefinition {
    let draw = || Effect::Draw { who: Selector::You, amount: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Fights, EventScope::YourControl).once_per_batch(),
                effect: draw(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl).once_per_batch(),
                effect: draw(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::MayPay {
                    description: "Pay {2}{R/G} to double a creature's power?".into(),
                    mana_cost: cost(&[generic(2), crate::mana::hybrid(Color::Red, Color::Green)]),
                    body: Box::new(Effect::Seq(vec![
                        Effect::DoublePower { what: target_filtered(R::Creature), times: Value::ONE, duration: Duration::EndOfTurn },
                        Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::MustBeBlocked, duration: Duration::EndOfCombat },
                    ])),
                    else_: None,
                },
            },
        ],
        ..legend(
            "Neyith of the Dire Hunt",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Wolverine, Claws Out — {3}{G} 2/5 legendary Mutant Berserker Hero. It may
/// assign combat damage as though it weren't blocked (CR 510.1c); a Mutant
/// you control attacking doubles its power until end of turn.
pub fn wolverine_claws_out() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::AssignsDamageAsThoughUnblocked],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCreatureType(CreatureType::Mutant),
            }),
            effect: Effect::DoublePower { what: Selector::TriggerSource, times: Value::ONE, duration: Duration::EndOfTurn },
        }],
        ..mutant_hero("Wolverine, Claws Out", cost(&[generic(3), g()]), 2, 5)
    }
}

/// Wolverine, Fierce Fighter — {2}{R}{G} 3/5 legendary Mutant Berserker
/// Hero, haste. Entering, it fights up to one other target creature. Damage
/// dealt to it heals the damage already marked on it (CR 614.1a).
pub fn wolverine_fierce_fighter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Fight {
                attacker: Selector::This,
                defender: target_filtered(R::Creature.and(R::OtherThanSource)),
            }),
        })],
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to Wolverine, instead that damage is dealt, but all other damage already dealt to him is healed.",
            effect: StaticEffect::NewDamageHealsOldDamage,
        }],
        ..mutant_hero("Wolverine, Fierce Fighter", cost(&[generic(2), r(), g()]), 3, 5)
    }
}

/// Ancient Animus — {1}{G} instant. A +1/+1 counter on target creature you
/// control if it's legendary, then it fights target creature an opponent
/// controls.
pub fn ancient_animus() -> CardDefinition {
    CardDefinition {
        name: "Ancient Animus",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::HasSupertype(Supertype::Legendary) },
                then: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Fight {
                attacker: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
            },
        ]),
        ..Default::default()
    }
}

/// Epic Fight — {2}{G} sorcery. Choose one or both: double target creature's
/// power and toughness until end of turn; target creature you control fights
/// target creature an opponent controls.
pub fn epic_fight() -> CardDefinition {
    CardDefinition {
        name: "Epic Fight",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            min: 1,
            max: 2,
            allow_repeats: false,
            modes: vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::PowerOf(Box::new(Selector::Target(0))),
                    toughness: Value::ToughnessOf(Box::new(Selector::Target(0))),
                    duration: Duration::EndOfTurn,
                },
                Effect::Fight {
                    attacker: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                    defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
                },
            ],
        },
        ..Default::default()
    }
}

// ── Captain Howler, Sea Scourge (UR discard) ─────────────────────────────────

fn you_discard() -> EventSpec {
    EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl)
}

/// Brass's Tunnel-Grinder // Tecutlan, the Searing Rift — {2}{R} legendary
/// artifact. Entering, discard any number of cards, then draw that many plus
/// one. At your end step, if you descended this turn, a bore counter; at
/// three, remove them and transform. Tecutlan: {T}: Add {R}; a permanent
/// spell cast with its mana discovers X, X = that spell's mana value.
pub fn brasss_tunnel_grinder() -> CardDefinition {
    let tecutlan = CardDefinition {
        name: "Tecutlan, the Searing Rift",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![crate::card::LandType::Cave], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::OfColor(Color::Red, Value::ONE)),
                    SpendRestriction::PermanentSpellDiscover,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Brass's Tunnel-Grinder",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::DiscardAnyNumber { who: Selector::You, filter: R::Any, max: None },
                Effect::Draw { who: Selector::You, amount: Value::Sum(vec![Value::CardsDiscardedThisEffect, Value::ONE]) },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(Predicate::DescendedThisTurn { who: PlayerRef::You }),
                effect: Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Bore, amount: Value::ONE },
                    Effect::If {
                        cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Bore, n: 3 },
                        then: Box::new(Effect::Seq(vec![
                            Effect::RemoveCounter { what: Selector::This, kind: CounterType::Bore, amount: Value::Const(3) },
                            Effect::Transform { what: Selector::This },
                        ])),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        back_face: Some(Box::new(tecutlan)),
        ..Default::default()
    }
}

/// Glint-Horn Buccaneer — {1}{R}{R} 2/4 Minotaur Pirate, haste. Each card you
/// discard deals 1 damage to each opponent. {1}{R}, discard a card: draw,
/// only while it's attacking.
pub fn glint_horn_buccaneer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: you_discard(),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            discard_cost: Some((R::Any, 1)),
            condition: Some(Predicate::EntityMatches { what: Selector::This, filter: R::IsAttacking }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Glint-Horn Buccaneer", cost(&[generic(1), r(), r()]), vec![CreatureType::Minotaur, CreatureType::Pirate], 2, 4)
    }
}

/// Ivora, Insatiable Heir — {1}{R} 1/1 legendary Vampire Warrior, trample.
/// Entering and on combat damage to a player, a Blood token; each card you
/// discard puts a +1/+1 counter on it.
pub fn ivora_insatiable_heir() -> CardDefinition {
    let blood = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(crabomination_base::tokens::blood_token()),
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            etb(blood()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: blood(),
            },
            TriggeredAbility {
                event: you_discard(),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..legend("Ivora, Insatiable Heir", cost(&[generic(1), r()]), vec![CreatureType::Vampire, CreatureType::Warrior], 1, 1)
    }
}

/// Mary Read and Anne Bonny — {1}{U}{R} 3/3 legendary Human Assassin Pirate,
/// haste. {T}: loot. Discarding an Island, Pirate or Vehicle card makes a
/// tapped Treasure.
pub fn mary_read_and_anne_bonny() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: you_discard().with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasLandType(crate::card::LandType::Island)
                    .or(R::HasCreatureType(CreatureType::Pirate))
                    .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
            }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::tapped_treasure_token()),
            },
        }],
        ..legend(
            "Mary Read and Anne Bonny",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Pirate],
            3,
            3,
        )
    }
}

/// Cool but Rude — {1}{R} Class. Whenever you attack, you may discard a card
/// to draw one. Level 2 ({1}{R}): each card you discard deals 2 damage to each
/// opponent. Level 3 ({1}{R}): on reaching it, tutor a card, then discard a
/// card at random.
pub fn cool_but_rude() -> CardDefinition {
    let level_up = |from: u8| ActivatedAbility {
        mana_cost: cost(&[generic(1), r()]),
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    CardDefinition {
        name: "Cool but Rude",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Class], ..Default::default() },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: Effect::MayDiscard {
                    description: "Discard a card to draw a card?".into(),
                    count: Value::ONE,
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: None,
                },
            },
            TriggeredAbility {
                event: you_discard().with_filter(Predicate::SourceClassLevelAtLeast(2)),
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::ClassLevelReached, EventScope::SelfSource)
                    .with_filter(Predicate::SourceClassLevelIs(3)),
                effect: Effect::Seq(vec![
                    Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Hand(PlayerRef::You) },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: true },
                ]),
            },
        ],
        activated_abilities: vec![level_up(1), level_up(2)],
        ..Default::default()
    }
}

/// Kaito, Cunning Infiltrator — {1}{U}{U} planeswalker, loyalty 3. A creature
/// you control dealing combat damage to a player puts a loyalty counter on
/// him. +1: up to one target creature you control can't be blocked this turn;
/// loot. −2: a 2/1 blue Ninja. −9: an emblem making a Ninja whenever a player
/// casts a spell.
pub fn kaito_cunning_infiltrator() -> CardDefinition {
    let ninja = || TokenDefinition {
        name: "Ninja".into(),
        power: 2,
        toughness: 1,
        colors: vec![Color::Blue],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ninja], ..Default::default() },
        ..Default::default()
    };
    let make_ninja = move || Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(ninja()) };
    let ability = |loyalty_cost: i32, effect: Effect| LoyaltyAbility { loyalty_cost, effect, ..Default::default() };
    CardDefinition {
        name: "Kaito, Cunning Infiltrator",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Kaito], ..Default::default() },
        base_loyalty: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Loyalty, amount: Value::ONE },
        }],
        loyalty_abilities: vec![
            ability(
                1,
                Effect::Seq(vec![
                    Effect::OptionalTargets {
                        min: 0,
                        body: Box::new(Effect::GrantKeyword {
                            what: target_filtered(R::Creature.and(R::ControlledByYou)),
                            keyword: Keyword::Unblockable,
                            duration: Duration::EndOfTurn,
                        }),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
            ),
            ability(-2, make_ninja()),
            ability(
                -9,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Kaito, Cunning Infiltrator".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
                        effect: make_ninja(),
                    }],
                    statics: vec![],
                },
            ),
        ],
        ..Default::default()
    }
}

/// Change of Fortune — {3}{R} sorcery. Discard your hand, then draw a card
/// for each card you've discarded this turn (this discard included).
pub fn change_of_fortune() -> CardDefinition {
    CardDefinition {
        name: "Change of Fortune",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
            Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisTurn(PlayerRef::You) },
        ]),
        ..Default::default()
    }
}

// ── Magda, Brazen Outlaw (R Dwarves and Vehicles) ────────────────────────────

fn vehicle(name: &'static str, mana: crate::mana::ManaCost, p: i32, t: i32, crew: u32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: p,
        toughness: t,
        keywords: vec![Keyword::Crew(crew)],
        ..Default::default()
    }
}

/// Clown Car — {X} 1/1 Vehicle, crew 2. Entering, roll X d6: each odd result
/// makes a 1/1 white Clown Robot artifact creature, each even one a +1/+1
/// counter on it.
pub fn clown_car() -> CardDefinition {
    let clown = TokenDefinition {
        name: "Clown Robot".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Clown, CreatureType::Robot], ..Default::default() },
        ..Default::default()
    };
    let odd = move || Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(clown.clone()) };
    let even = || Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::RollDie {
            sides: 6,
            count: Value::XFromCost,
            modifier: Value::Const(0),
            reroll_at_most: 0,
            ignore_lowest: 0,
            results: vec![(1, 1, odd()), (2, 2, even()), (3, 3, odd()), (4, 4, even()), (5, 5, odd()), (6, 6, even())],
            on_doubles: None,
        })],
        ..vehicle("Clown Car", cost(&[x()]), 1, 1, 2)
    }
}

/// Flywheel Racer — {2} 3/2 Vehicle, vigilance, crew 1. {T}: one mana of any
/// color, only while it's a creature.
pub fn flywheel_racer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Crew(1)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::EntityMatches { what: Selector::This, filter: R::Creature }),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..vehicle("Flywheel Racer", cost(&[generic(2)]), 3, 2, 1)
    }
}

/// High-Speed Hoverbike — {2} 2/2 Vehicle, flash, flying, crew 1. Entering,
/// tap up to one target creature.
pub fn high_speed_hoverbike() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Crew(1)],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Tap { what: target_filtered(R::Creature) }),
        })],
        ..vehicle("High-Speed Hoverbike", cost(&[generic(2)]), 2, 2, 1)
    }
}

/// Dwarven Armorer — {R} 0/2 Dwarf. {R}, {T}, discard a card: a +0/+1 or a
/// +1/+0 counter on target creature.
pub fn dwarven_armorer() -> CardDefinition {
    let counter = |kind| Effect::AddCounter { what: target_filtered(R::Creature), kind, amount: Value::ONE };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::ChooseMode(vec![counter(CounterType::PlusZeroPlusOne), counter(CounterType::PlusOnePlusZero)]),
            ..Default::default()
        }],
        ..creature("Dwarven Armorer", cost(&[r()]), vec![CreatureType::Dwarf], 0, 2)
    }
}

/// Dwarven Mauler — {R} 2/1 Dwarf Warrior. Equip abilities you activate that
/// target it cost {2} less.
pub fn dwarven_mauler() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Equip abilities you activate that target this creature cost {2} less to activate.",
            effect: StaticEffect::EquipCostReductionTargetingSelf { amount: 2 },
        }],
        ..creature("Dwarven Mauler", cost(&[r()]), vec![CreatureType::Dwarf, CreatureType::Warrior], 2, 1)
    }
}

/// Vault Robber — {1}{R} 1/3 Dwarf Rogue. {1}, {T}, exile a creature card
/// from your graveyard: a Treasure.
pub fn vault_robber() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            exile_other_filter: Some((R::Creature, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
            ..Default::default()
        }],
        ..creature("Vault Robber", cost(&[generic(1), r()]), vec![CreatureType::Dwarf, CreatureType::Rogue], 1, 3)
    }
}

/// Óin the Brave — {1}{R} 1/3 legendary Dwarf Warrior, storied. With an
/// enduring story it gets +1/+0 and haste. {1}, {T}, discard a card: draw.
pub fn oin_the_brave() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Storied (If you control three or more artifacts, legendaries, and/or Sagas, you have an enduring story for the rest of the game.)",
                effect: StaticEffect::Storied,
            },
            StaticAbility {
                description: "As long as you have an enduring story, Óin gets +1/+0 and has haste.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::HasEnduringStory { who: PlayerRef::You },
                    inner: Box::new(StaticEffect::PumpPT { applies_to: Selector::This, power: 1, toughness: 0 }),
                },
            },
            StaticAbility {
                description: "As long as you have an enduring story, Óin has haste.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::HasEnduringStory { who: PlayerRef::You },
                    inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Haste }),
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..legend("Óin the Brave", cost(&[generic(1), r()]), vec![CreatureType::Dwarf, CreatureType::Warrior], 1, 3)
    }
}

// ── Lorehold, the Historian (RW spellslinger) ───────────────────────────────

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

/// Goliath Daydreamer — {2}{R}{R} 4/4 Giant Wizard. An instant or sorcery
/// spell you cast from your hand is exiled with a dream counter as it resolves
/// instead of going to your graveyard; attacking, you may cast a spell from
/// among cards you own in exile with dream counters without paying its mana
/// cost.
pub fn goliath_daydreamer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: instant_or_sorcery().and(R::Not(Box::new(R::SpellNotCastFromHand))),
                }),
                effect: Effect::ExileSpellOnResolveWithCounter { what: Selector::TriggerSource, counter: CounterType::Dream },
            },
            crate::effect::shortcut::on_attack(Effect::MayCastFromExileFree {
                filter: R::OwnedByYou.and(R::WithCounter(CounterType::Dream)),
            }),
        ],
        ..creature("Goliath Daydreamer", cost(&[generic(2), r(), r()]), vec![CreatureType::Giant, CreatureType::Wizard], 4, 4)
    }
}

/// Invoke Calamity — {1}{R}{R}{R}{R} Instant. Cast up to two instant and/or
/// sorcery spells with total mana value 6 or less from your graveyard and/or
/// hand free; they're exiled instead of going to your graveyard. Exile Invoke
/// Calamity.
pub fn invoke_calamity() -> CardDefinition {
    CardDefinition {
        name: "Invoke Calamity",
        cost: cost(&[generic(1), r(), r(), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CastUpToFreeFromHandAndGraveyard { count: 2, filter: instant_or_sorcery(), total_mana_value: 6 },
        exile_on_resolve: true,
        ..Default::default()
    }
}

/// Olórin's Searing Light — {2}{R}{W} Instant. Each opponent exiles a creature
/// with the greatest power among creatures they control. Spell mastery: it
/// deals each of them damage equal to the exiled creature's power (as it last
/// existed). The opponents choose one at a time in APNAP order, each exile
/// happening as it is chosen.
pub fn olorins_searing_light() -> CardDefinition {
    let that_player = || PlayerRef::Triggerer;
    CardDefinition {
        name: "Olórin's Searing Light",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ForEachOpponent {
            body: Box::new(Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy { who: that_player(), filter: R::Creature }),
                then: Box::new(Effect::Seq(vec![
                    Effect::PlayerExilesPermanents {
                        who: that_player(),
                        count: Value::ONE,
                        filter: R::Creature.and(R::HasGreatestPowerAmongControlled(Box::new(R::Creature))),
                    },
                    Effect::If {
                        cond: crate::effect::shortcut::spell_mastery_gate(),
                        then: Box::new(Effect::DealDamage {
                            to: Selector::Player(that_player()),
                            amount: Value::LastExiledPower,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
                else_: Box::new(Effect::Noop),
            }),
        },
        ..Default::default()
    }
}

/// Soulfire Eruption — {6}{R}{R}{R} Sorcery. For each of any number of target
/// creatures, planeswalkers and/or players: exile the top card of your library
/// and deal damage equal to its mana value to that target. You may play the
/// exiled cards until the end of your next turn.
pub fn soulfire_eruption() -> CardDefinition {
    CardDefinition {
        name: "Soulfire Eruption",
        cost: cost(&[generic(6), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: R::Creature.or(R::Planeswalker).or(R::Player),
            effect: Box::new(Effect::Seq(vec![
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
                Effect::DealDamage { to: Selector::Target(0), amount: Value::LastExiledManaValue },
            ])),
        },
        ..Default::default()
    }
}

// ── High Perfect Morcant (BG Elves) ─────────────────────────────────────────

fn elf() -> R {
    R::HasCreatureType(CreatureType::Elf)
}

fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl), effect }
}

/// Eclipsed Elf — {B/G}{B/G}{B/G} 3/2 Elf Scout. Entering, look at the top
/// four: you may take an Elf, Swamp or Forest card; the rest go to the bottom
/// in a random order.
pub fn eclipsed_elf() -> CardDefinition {
    use crate::card::LandType;
    use crate::mana::hybrid;
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(crate::effect::LookPick {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(elf().or(R::HasLandType(LandType::Swamp)).or(R::HasLandType(LandType::Forest))),
            take: Some(Value::ONE),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })))],
        ..creature(
            "Eclipsed Elf",
            crate::mana::ManaCost::new(vec![
                hybrid(Color::Black, Color::Green),
                hybrid(Color::Black, Color::Green),
                hybrid(Color::Black, Color::Green),
            ]),
            vec![CreatureType::Elf, CreatureType::Scout],
            3,
            2,
        )
    }
}

/// Elvish Harbinger — {2}{G} 1/2 Elf Druid. Entering, search for an Elf card
/// and put it on top after shuffling. {T}: Add one mana of any color.
pub fn elvish_harbinger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: elf(),
            to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..creature("Elvish Harbinger", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Druid], 1, 2)
    }
}

/// Crumbling Ashes — {1}{B} Enchantment. At the beginning of your upkeep,
/// destroy target creature with a -1/-1 counter on it.
pub fn crumbling_ashes() -> CardDefinition {
    CardDefinition {
        name: "Crumbling Ashes",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![your_upkeep(Effect::Destroy {
            what: target_filtered(R::Creature.and(R::WithCounter(CounterType::MinusOneMinusOne))),
        })],
        ..Default::default()
    }
}

/// Morcant's Eyes — {1}{G} Kindred Enchantment — Elf. Upkeep: surveil 1.
/// {4}{G}{G}, sacrifice it: X 2/2 black and green Elves, X = Elf cards in
/// your graveyard; sorcery speed.
pub fn morcants_eyes() -> CardDefinition {
    let elf_token = TokenDefinition {
        name: "Elf".into(),
        power: 2,
        toughness: 2,
        colors: vec![Color::Black, Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Morcant's Eyes",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Kindred, CardType::Enchantment],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        triggered_abilities: vec![your_upkeep(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g(), g()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: elf() },
                definition: Arc::new(elf_token),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tyvar Kell — {2}{G}{G} Tyvar, 3 loyalty. Elves you control have "{T}: Add
/// {B}." +1: a +1/+1 counter on up to one target Elf, untap it, deathtouch
/// until end of turn. 0: a 1/1 green Elf Warrior. −6: an emblem — an Elf spell
/// you cast gains haste and you draw two.
pub fn tyvar_kell() -> CardDefinition {
    let ability = |loyalty_cost: i32, effect: Effect| LoyaltyAbility { loyalty_cost, effect, ..Default::default() };
    let warrior = TokenDefinition {
        name: "Elf Warrior".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Warrior], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Tyvar Kell",
        cost: cost(&[generic(2), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Tyvar], ..Default::default() },
        base_loyalty: 3,
        static_abilities: vec![StaticAbility {
            description: "Elves you control have \"{T}: Add {B}.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(elf().and(R::ControlledByYou)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Black]) },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        loyalty_abilities: vec![
            ability(
                1,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::AddCounter {
                            what: target_filtered(elf()),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                        Effect::Untap { what: Selector::Target(0), up_to: None },
                        Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
                    ])),
                },
            ),
            ability(0, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(warrior) }),
            ability(
                -6,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Tyvar Kell".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: elf() }),
                        effect: Effect::Seq(vec![
                            Effect::GrantKeywordsToSpell { what: Selector::TriggerSource, keywords: vec![Keyword::Haste] },
                            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                        ]),
                    }],
                    statics: vec![],
                },
            ),
        ],
        ..Default::default()
    }
}

/// Trystan's Command — {4}{B}{G} Kindred Sorcery — Elf. Choose two: a token
/// copy of target Elf you control; return one or two permanent cards from your
/// graveyard to your hand (chosen as it resolves, not targeted); destroy
/// target creature or enchantment; creatures target player controls get
/// +3/+3 until end of turn and untap.
pub fn trystans_command() -> CardDefinition {
    CardDefinition {
        name: "Trystan's Command",
        cost: cost(&[generic(4), b(), g()]),
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        effect: Effect::ChooseModesCast {
            min: 2,
            max: 2,
            allow_repeats: false,
            modes: vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(elf().and(R::ControlledByYou)),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                    no_mana_cost: false,
                    enters_with_counters: None,
                    remove_keywords: vec![],
                },
                Effect::MoveChosen {
                    from: Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Graveyard, filter: R::Permanent },
                    filter: None,
                    count: Value::Const(2),
                    up_to: true,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Destroy { what: target_filtered(R::Creature.or(R::Enchantment)) },
                Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                        power: Value::Const(3),
                        toughness: Value::Const(3),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Untap {
                        what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                        up_to: None,
                    },
                ]),
            ],
        },
        ..Default::default()
    }
}

// ── Iroh, Grand Lotus's EDHREC gaps ─────────────────────────────────────────

/// Chakra Meditation — {2}{U} Enchantment. ETB: return up to one target
/// instant or sorcery card from your graveyard to your hand. Whenever you cast
/// an instant or sorcery spell, draw a card, then discard a card unless there
/// are three or more Lesson cards in your graveyard.
pub fn chakra_meditation() -> CardDefinition {
    use crate::card::SpellSubtype;
    use crate::effect::shortcut::magecraft;
    let lessons = Value::CardsInGraveyardMatching {
        who: PlayerRef::You,
        filter: R::HasSpellSubtype(SpellSubtype::Lesson),
    };
    CardDefinition {
        name: "Chakra Meditation",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::InYourGraveyard.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
            }),
            magecraft(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(lessons, Value::Const(3)),
                    then: Box::new(Effect::Noop),
                    else_: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
                },
            ])),
        ],
        ..Default::default()
    }
}

/// Elemental Teachings — {4}{G} Instant — Lesson. Search for up to four land
/// cards with different names; an opponent chooses two for your graveyard, the
/// rest enter tapped; then shuffle.
pub fn elemental_teachings() -> CardDefinition {
    use crate::card::SpellSubtype;
    CardDefinition {
        name: "Elemental Teachings",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() },
        effect: Effect::Seq(vec![
            Effect::SearchSplitWithOpponent {
                count: 4,
                filter: R::Land,
                mv_at_most_x: false,
                graveyard: false,
                denied: ZoneDest::Graveyard,
                rest: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::ShuffleLibrary { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}

/// Overwhelming Victory — {4}{R} Instant — Lesson. 5 damage to target
/// creature; each creature you control gains trample and gets +X/+0 until end
/// of turn, X the excess damage dealt this way.
pub fn overwhelming_victory() -> CardDefinition {
    use crate::card::SpellSubtype;
    let mine = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Overwhelming Victory",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() },
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(5) },
            Effect::GrantKeyword { what: mine(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            Effect::PumpPT {
                what: mine(),
                power: Value::ExcessDamageDealtThisResolution,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Secret of Bloodbending — {U}{U}{U}{U} Sorcery — Lesson. You may waterbend
/// {10}. You control target opponent during their next combat phase (CR 723.2),
/// or during their next turn if the waterbend was paid. Exile it.
pub fn secret_of_bloodbending() -> CardDefinition {
    use crate::card::{SpellSubtype, Waterbend};
    CardDefinition {
        name: "Secret of Bloodbending",
        cost: cost(&[u(), u(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() },
        waterbend: Some(Waterbend { amount: Value::Const(10), optional: true }),
        exile_on_resolve: true,
        effect: Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::If {
                cond: Predicate::SpellWasWaterbend,
                then: Box::new(Effect::ControlPlayerNextTurn { who: PlayerRef::Target(0) }),
                else_: Box::new(Effect::ControlPlayerNextCombat { who: PlayerRef::Target(0) }),
            }),
        },
        ..Default::default()
    }
}

// ── Galadriel, Light of Valinor's EDHREC gaps ───────────────────────────────

/// Aragorn and Arwen, Wed — {4}{G}{W} 3/6 vigilance. Entering or attacking: a
/// +1/+1 counter on each other creature you control, then 1 life per other
/// creature you control.
pub fn aragorn_and_arwen_wed() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    let others = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource));
    let body = || {
        Effect::Seq(vec![
            Effect::AddCounter { what: others(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::GainLife { who: Selector::You, amount: Value::count(others()) },
        ])
    };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(body()), on_attack(body())],
        ..legend(
            "Aragorn and Arwen, Wed",
            cost(&[generic(4), g(), w()]),
            vec![CreatureType::Human, CreatureType::Elf, CreatureType::Noble],
            3,
            6,
        )
    }
}

/// Arwen Undómiel — {G}{U} 2/2. Whenever you scry, a +1/+1 counter on target
/// creature. {4}{G}{U}: Scry 2.
pub fn arwen_undomiel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Scried, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g(), u()]),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..legend("Arwen Undómiel", cost(&[g(), u()]), vec![CreatureType::Elf, CreatureType::Noble], 2, 2)
    }
}

/// Elrond, Master of Healing — {2}{G}{U} 4/4. Whenever you scry, a +1/+1
/// counter on each of up to X target creatures, X the cards looked at; a
/// countered creature of yours targeted by an opponent may draw you a card.
pub fn elrond_master_of_healing() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Scried, EventScope::YourControl),
                // CR 603.3d — X is the scry's `looked_at`, bound as the
                // trigger is put on the stack.
                effect: Effect::CapTargetsAt {
                    amount: Value::TriggerEventAmount,
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 8,
                        min_targets: 0,
                        filter: R::Creature,
                        effect: Box::new(Effect::AddCounter {
                            what: Selector::Target(0),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        }),
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::YourPermanentTargetedByOpponent)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne)),
                    }),
                effect: Effect::MayDo {
                    description: "Elrond: draw a card?".into(),
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            },
        ],
        ..legend(
            "Elrond, Master of Healing",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            4,
            4,
        )
    }
}

/// Galadriel of Lothlórien — {1}{G}{U} 3/3. The Ring tempting you onto another
/// bearer scries 3; whenever you scry, reveal the top card and a land enters
/// tapped.
pub fn galadriel_of_lothlorien() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::RingTempted, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource },
                ),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Scried, EventScope::YourControl),
                effect: Effect::RevealTopDeployIfMatch {
                    filter: R::Land,
                    haste: false,
                    sacrifice_at_next_end_step: false,
                    miss_to_graveyard: false,
                    tapped: true,
                },
            },
        ],
        ..legend(
            "Galadriel of Lothlórien",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            3,
            3,
        )
    }
}

/// Arboreal Alliance — {X}{G}{G} Enchantment. ETB: an X/X green Treefolk.
/// Whenever you attack with one or more Elves, populate.
pub fn arboreal_alliance() -> CardDefinition {
    let treefolk = Arc::new(TokenDefinition {
        name: "Treefolk".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Treefolk], ..Default::default() },
        dynamic_pt: Some((Value::XFromCost, Value::XFromCost)),
        ..Default::default()
    });
    CardDefinition {
        name: "Arboreal Alliance",
        cost: cost(&[x(), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: treefolk }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                    Predicate::AttackedWithCreatureMatching { who: PlayerRef::You, filter: R::HasCreatureType(CreatureType::Elf) },
                ),
                effect: Effect::Populate { who: PlayerRef::You },
            },
        ],
        ..Default::default()
    }
}

/// Galadriel's Dismissal — {W} Instant, kicker {2}{W}. Target creature phases
/// out; kicked, each creature target player controls phases out instead (the
/// kicked cast targets only the player).
pub fn galadriels_dismissal() -> CardDefinition {
    CardDefinition {
        name: "Galadriel's Dismissal",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Kicker(cost(&[generic(2), w()]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(Effect::PhaseOut {
                    what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                    until_source_leaves: false,
                }),
            }),
            else_: Box::new(Effect::PhaseOut { what: target_filtered(R::Creature), until_source_leaves: false }),
        },
        ..Default::default()
    }
}

// ── Skullbriar, the Walking Grave's EDHREC gaps ─────────────────────────────

/// A Mentor of the Duskmourn cycle: entering, a `kw` counter on a target
/// non-Human creature you control; `{1}{C}, {T}`: a +1/+1 counter on each
/// creature you control with `kw`.
fn kw_mentor(
    name: &'static str,
    mana: crate::mana::ManaCost,
    activation: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    kw: Keyword,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddKeywordCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::HasCreatureType(CreatureType::Human).negate())),
            keyword: kw.clone(),
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: activation,
            tap_cost: true,
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::HasKeyword(kw))),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(name, mana, types, p, t)
    }
}

/// Duskfang Mentor — {2}{B} 1/3 Human Cleric: a lifelink counter; {1}{B}, {T}.
pub fn duskfang_mentor() -> CardDefinition {
    kw_mentor(
        "Duskfang Mentor",
        cost(&[generic(2), b()]),
        cost(&[generic(1), b()]),
        vec![CreatureType::Human, CreatureType::Cleric],
        1,
        3,
        Keyword::Lifelink,
    )
}

/// Hornbash Mentor — {2}{G} 3/3 Human Warrior: a trample counter; {2}{G}, {T}.
pub fn hornbash_mentor() -> CardDefinition {
    kw_mentor(
        "Hornbash Mentor",
        cost(&[generic(2), g()]),
        cost(&[generic(2), g()]),
        vec![CreatureType::Human, CreatureType::Warrior],
        3,
        3,
        Keyword::Trample,
    )
}

/// Thundering Mightmare — {4}{G} 3/3 Horse Spirit, soulbond. Each of the pair
/// has "Whenever an opponent casts a spell, put a +1/+1 counter on this
/// creature" (granted to the partner the Mirage Phalanx way).
pub fn thundering_mightmare() -> CardDefinition {
    let grow = TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(Predicate::ValueAtLeast(
            Value::CountOf(Box::new(Selector::SoulbondPartner)),
            Value::ONE,
        )),
        effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
    };
    CardDefinition {
        keywords: vec![Keyword::Soulbond],
        static_abilities: vec![StaticAbility {
            description: "As long as Thundering Mightmare is paired with another creature, that creature has the counter trigger too.",
            effect: StaticEffect::GrantTriggeredAbility { filter: R::PairedWithSource, ability: Box::new(grow.clone()) },
        }],
        triggered_abilities: vec![grow],
        ..creature("Thundering Mightmare", cost(&[generic(4), g()]), vec![CreatureType::Horse, CreatureType::Spirit], 3, 3)
    }
}

/// Call of the Death-Dweller — {2}{B} Sorcery. Up to two creature cards with
/// total mana value 3 or less return from your graveyard; a deathtouch counter
/// on either of them, then a menace counter on either of them.
pub fn call_of_the_death_dweller() -> CardDefinition {
    let counter = |kw: Keyword| Effect::ChooseOneAmong {
        what: Selector::LastMoved,
        chooser: PlayerRef::You,
        chosen: Box::new(Effect::AddKeywordCounter { what: Selector::SeparatedPile { chosen: true }, keyword: kw, amount: Value::ONE }),
        other: Box::new(Effect::Noop),
    };
    CardDefinition {
        name: "Call of the Death-Dweller",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature.and(R::InYourGraveyard).and(R::SlotsTotalManaValueAtMost(3)),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
            counter(Keyword::Deathtouch),
            counter(Keyword::Menace),
        ]),
        ..Default::default()
    }
}

/// Increasing Savagery — {2}{G}{G} Sorcery, flashback {5}{G}{G}. Five +1/+1
/// counters on target creature; ten if cast from a graveyard.
pub fn increasing_savagery() -> CardDefinition {
    let put = |n: i32| Effect::AddCounter {
        what: target_filtered(R::Creature),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
    };
    CardDefinition {
        name: "Increasing Savagery",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(5), g(), g()]))],
        effect: Effect::If { cond: Predicate::CastFromGraveyard, then: Box::new(put(10)), else_: Box::new(put(5)) },
        ..Default::default()
    }
}

/// Visions of Dominance — {2}{G} Sorcery. A +1/+1 counter on target creature,
/// then double its +1/+1 counters; flashback {8}{G}{G}, {X} less where X is
/// your commander's greatest mana value.
pub fn visions_of_dominance() -> CardDefinition {
    CardDefinition {
        name: "Visions of Dominance",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(8), g(), g()]))],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast this way, where X is the greatest mana value of a commander you own on the battlefield or in the command zone.",
            effect: StaticEffect::SelfFlashbackCostsLess {
                amount: Value::GreatestCommanderManaValueInPlayOrCommandZone(PlayerRef::You),
            },
        }],
        effect: Effect::Seq(vec![
            Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::DoubleCountersOnEach { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne },
        ]),
        ..Default::default()
    }
}

// ── Aragorn, the Uniter's EDHREC gaps ───────────────────────────────────────

fn your_legend() -> R {
    R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou)
}

/// Rivendell — legendary land; enters tapped unless you control a legendary
/// creature; {T}: Add {U}; {1}{U}, {T}: Scry 2, only with a legendary
/// creature.
pub fn rivendell() -> CardDefinition {
    let legend_here = || Predicate::SelectorExists(Selector::EachPermanent(your_legend()));
    CardDefinition {
        name: "Rivendell",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Rivendell enters tapped unless you control a legendary creature.",
            effect: StaticEffect::EntersTappedUnless { applies_to: Selector::This, condition: legend_here() },
        }],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Blue),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                tap_cost: true,
                condition: Some(legend_here()),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Aragorn, Hornburg Hero — {1}{R}{G}{W} 4/4. Attacking creatures you control
/// have first strike and renown 1; a renowned creature of yours that deals
/// combat damage to a player doubles its +1/+1 counters.
pub fn aragorn_hornburg_hero() -> CardDefinition {
    use crate::effect::shortcut::renown;
    let attacking = || Selector::EachPermanent(R::Creature.and(R::IsAttacking).and(R::ControlledByYou));
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Attacking creatures you control have first strike.",
                effect: StaticEffect::GrantKeyword { applies_to: attacking(), keyword: Keyword::FirstStrike },
            },
            StaticAbility {
                description: "Attacking creatures you control have renown 1.",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: R::Creature.and(R::IsAttacking).and(R::ControlledByYou),
                    ability: Box::new(renown(1)),
                },
            },
            StaticAbility {
                description: "Whenever a renowned creature you control deals combat damage to a player, double the number of +1/+1 counters on it.",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: R::Creature.and(R::ControlledByYou),
                    ability: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                            .with_filter(Predicate::SourceIsRenowned),
                        effect: Effect::DoubleCountersOnEach { what: Selector::This, kind: CounterType::PlusOnePlusOne },
                    }),
                },
            },
        ],
        ..legend(
            "Aragorn, Hornburg Hero",
            cost(&[generic(1), r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Faramir, Prince of Ithilien — {2}{W}{U} 3/3. At your end step, choose an
/// opponent; at their next end step, draw if they didn't attack you that
/// turn, else three 1/1 white Human Soldiers.
pub fn faramir_prince_of_ithilien() -> CardDefinition {
    use crate::effect::DelayedTriggerKind;
    let soldier = Arc::new(TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::ChooseOpponentThen {
                then: Box::new(Effect::DelayUntilWithCapture {
                    kind: DelayedTriggerKind::TargetsNextEndStep,
                    capture: Selector::Player(PlayerRef::ChosenPlayerOfSource),
                    body: Box::new(Effect::If {
                        cond: Predicate::PlayerAttackedYouThisTurn(PlayerRef::Target(0)),
                        then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(3), definition: soldier }),
                        else_: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    }),
                }),
            },
        }],
        ..legend(
            "Faramir, Prince of Ithilien",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Human, CreatureType::Noble],
            3,
            3,
        )
    }
}

/// Katilda, Dawnhart Prime — {G}{W} 1/1. Protection from Werewolves; your
/// Humans have "{T}: Add one mana of any of this creature's colors";
/// {4}{G}{W}, {T}: a +1/+1 counter on each creature you control.
pub fn katilda_dawnhart_prime() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        keywords: vec![Keyword::ProtectionFromCreatureType(CreatureType::Werewolf)],
        static_abilities: vec![StaticAbility {
            description: "Human creatures you control have \"{T}: Add one mana of any of this creature's colors.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Human)).and(R::ControlledByYou)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColorOfSource },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g(), w()]),
            tap_cost: true,
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend("Katilda, Dawnhart Prime", cost(&[g(), w()]), vec![CreatureType::Human, CreatureType::Warlock], 1, 1)
    }
}

/// Pippin, Guard of the Citadel — {W}{U} 2/2 vigilance, ward {1}. {T}: another
/// target creature you control gains protection from the card type of your
/// choice until end of turn.
pub fn pippin_guard_of_the_citadel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Ward(crate::card::WardCost::generic(1))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantProtectionFromChosenCardType {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Pippin, Guard of the Citadel",
            cost(&[w(), u()]),
            vec![CreatureType::Halfling, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Éowyn, Fearless Knight — {2}{R}{W} 3/4 haste. Entering, exile target
/// creature an opponent controls with greater power; your legendary creatures
/// gain protection from each of its colors until end of turn.
pub fn eowyn_fearless_knight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        // The colours are read before the exile takes the creature away.
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GrantProtectionFromColorsOf {
                what: Selector::EachPermanent(your_legend()),
                of: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::PowerGreaterThanSource)),
                duration: Duration::EndOfTurn,
            },
            Effect::Exile { what: Selector::Target(0) },
        ]))],
        ..legend("Éowyn, Fearless Knight", cost(&[generic(2), r(), w()]), vec![CreatureType::Human, CreatureType::Knight], 3, 4)
    }
}

/// Friendly Rivalry — {R}{G} Instant. Target creature you control and up to
/// one other target legendary creature you control each deal damage equal to
/// their power to target creature you don't control. (Slots: yours, theirs,
/// then the optional legend.)
pub fn friendly_rivalry() -> CardDefinition {
    CardDefinition {
        name: "Friendly Rivalry",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::OptionalTargets {
            min: 2,
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamageFrom {
                    source: target_filtered(R::Creature.and(R::ControlledByYou)),
                    to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou.negate()) },
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                },
                Effect::DealDamageFrom {
                    source: Selector::TargetFiltered { slot: 2, filter: your_legend().and(R::NameDiffersFromOtherTargets) },
                    to: Selector::Target(1),
                    amount: Value::PowerOf(Box::new(Selector::Target(2))),
                },
            ])),
        },
        ..Default::default()
    }
}

// ── Ragost, Deft Gastronaut's EDHREC gaps ───────────────────────────────────

fn artifact(name: &'static str, mana: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

/// "When this artifact enters or is put into a graveyard from the
/// battlefield, `effect`" — the two triggers of the Statue / Schematic shape.
fn enters_or_dies(effect: Effect) -> Vec<TriggeredAbility> {
    vec![
        etb(effect.clone()),
        TriggeredAbility { event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource), effect },
    ]
}

/// Caduceus, Staff of Hermes — {2}{W} legendary Equipment. Lifelink; at 30 or
/// more life the bearer gets +5/+5, indestructible and "prevent all damage
/// that would be dealt to this creature". Equip {W}{W}.
pub fn caduceus_staff_of_hermes() -> CardDefinition {
    use crate::card::{ConditionalEquipBonus, EquipBonus};
    let thirty = || Predicate::PlayerLifeAtLeast { who: PlayerRef::You, life: 30 };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[w(), w()]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Lifelink],
            conditional: vec![ConditionalEquipBonus {
                host_filter: R::Creature,
                power: 5,
                toughness: 5,
                keywords: vec![Keyword::Indestructible],
                condition: Some(thirty()),
                set_base_pt: None,
                activated_abilities: vec![],
            }],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "As long as you have 30 or more life, prevent all damage that would be dealt to equipped creature.",
            effect: StaticEffect::WhileCondition {
                condition: thirty(),
                inner: Box::new(StaticEffect::PreventAllDamageToEnchanted),
            },
        }],
        ..artifact("Caduceus, Staff of Hermes", cost(&[generic(2), w()]))
    }
}

/// Implement of Improvement — {1}. {W}, sacrifice: gain 2 life; put into a
/// graveyard from the battlefield, draw a card.
pub fn implement_of_improvement() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            sac_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..artifact("Implement of Improvement", cost(&[generic(1)]))
    }
}

/// Nimblewright Schematic — {2}. Entering or dying, a 1/1 colorless Construct
/// artifact creature token.
pub fn nimblewright_schematic() -> CardDefinition {
    let construct = Arc::new(TokenDefinition {
        name: "Construct".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: enters_or_dies(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: construct }),
        ..artifact("Nimblewright Schematic", cost(&[generic(2)]))
    }
}

/// Prized Statue — {2}. Entering or dying, a Treasure.
pub fn prized_statue() -> CardDefinition {
    CardDefinition {
        triggered_abilities: enters_or_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(crabomination_base::tokens::treasure_token()),
        }),
        ..artifact("Prized Statue", cost(&[generic(2)]))
    }
}

/// Transmutation Font — {5}. {T}: a Blood, Clue or Food token. {3}, {T},
/// sacrifice three artifact tokens with different names: search for an
/// artifact card onto the battlefield; sorcery speed.
pub fn transmutation_font() -> CardDefinition {
    use crabomination_base::tokens::{blood_token, clue_token, food_token};
    let make = |t: TokenDefinition| Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(t) };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::ChooseMode(vec![make(blood_token()), make(clue_token()), make(food_token())]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sorcery_speed: true,
                sac_other_filter: Some((R::Artifact.and(R::IsToken), 3)),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Artifact,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..artifact("Transmutation Font", cost(&[generic(5)]))
    }
}

/// Apothecary White — {3}{W} 3/4 vigilance. Whenever you attack, a Food per
/// player being attacked; {W}, {T}, tap X untapped Foods: X 1/1 white Humans.
pub fn apothecary_white() -> CardDefinition {
    use crate::effect::shortcut::on_you_attack;
    let human = Arc::new(TokenDefinition {
        name: "Human".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_you_attack(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::OpponentsAttackedThisCombat,
            definition: Arc::new(crabomination_base::tokens::food_token()),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            tap_n_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Food), 0)),
            tap_n_x: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::XFromCost, definition: human },
            ..Default::default()
        }],
        ..legend("Apothecary White", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Cleric], 3, 4)
    }
}

/// Ratchet, Rescue Racer — the Vehicle back: living metal, lifelink; one or
/// more of your nontoken artifacts going to a graveyard converts it, once a
/// turn.
pub fn ratchet_rescue_racer() -> CardDefinition {
    CardDefinition {
        name: "Ratchet, Rescue Racer",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        color_indicator: vec![Color::White],
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::LivingMetal, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.and(R::NotToken) })
                .once_per_turn(),
            effect: Effect::Transform { what: Selector::This },
        }],
        ..Default::default()
    }
}

/// Ratchet, Field Medic — {2}{W} 2/4 Robot, More Than Meets the Eye {1}{W},
/// lifelink. Whenever you gain life, you may convert it; when you do, return
/// target artifact card with mana value at most the life gained this turn
/// from your graveyard to the battlefield tapped.
pub fn ratchet_field_medic() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Ratchet, Field Medic",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        alternative_cost: Some(AlternativeCost { mana_cost: cost(&[generic(1), w()]), converted: true, ..Default::default() }),
        back_face: Some(Box::new(ratchet_rescue_racer())),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Convert Ratchet and return an artifact?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Transform { what: Selector::This },
                    Effect::ReflexiveTrigger {
                        body: Box::new(Effect::Move {
                            what: target_filtered(R::Artifact.and(R::InYourGraveyard).and(R::ManaValueAtMostLifeGainedThisTurn)),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                        }),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Spirit Loop — {1}{W} Aura, enchant creature you control. Whenever the
/// enchanted creature deals damage, gain that much life; put into a graveyard
/// from the battlefield, it returns to its owner's hand.
pub fn spirit_loop() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Spirit Loop",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
        equipped_bonus: Some(EquipBonus::default()),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsDamage, EventScope::EnchantedBySource),
                effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))) },
            },
        ],
        ..Default::default()
    }
}

// ── Iron Man, Titan of Innovation's EDHREC gaps ─────────────────────────────

/// Arc Reactor — {5} artifact, improvise; enters tapped; {T}: Add {C}{C}{C}.
pub fn arc_reactor() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        keywords: vec![Keyword::Improvise],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(3)) },
            ..Default::default()
        }],
        ..artifact("Arc Reactor", cost(&[generic(5)]))
    }
}

/// Esoteric Duplicator — {2}{U} artifact Clue. Whenever you sacrifice it or
/// another artifact, you may pay {2}: at the next end step, a token copy of
/// that artifact. {2}, sacrifice: draw a card.
pub fn esoteric_duplicator() -> CardDefinition {
    use crate::effect::DelayedTriggerKind;
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Clue], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
            effect: Effect::MayPay {
                description: "Pay {2} to copy the sacrificed artifact at the next end step?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::DelayUntilWithCapture {
                    kind: DelayedTriggerKind::NextEndStep,
                    capture: Selector::TriggerSource,
                    body: Box::new(Effect::CreateTokenCopyOf {
                        extra_keywords: vec![],
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: Selector::Target(0),
                        extra_creature_types: vec![],
                        extra_card_types: vec![],
                        override_pt: None,
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        no_mana_cost: false,
                        enters_with_counters: None,
                        remove_keywords: vec![],
                    }),
                }),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..artifact("Esoteric Duplicator", cost(&[generic(2), u()]))
    }
}

/// Ugin's Nexus — {5} legendary artifact. Extra turns are skipped; if it
/// would be put into a graveyard from the battlefield, exile it instead and
/// take an extra turn after this one.
pub fn ugins_nexus() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "If a player would begin an extra turn, that player skips that turn instead.",
            effect: StaticEffect::PlayersSkipExtraTurns,
        }],
        dies_to_exile: true,
        dies_to_exile_extra_turn: true,
        ..artifact("Ugin's Nexus", cost(&[generic(5)]))
    }
}

/// Iron Man, Bleeding Edge — {3}{U}{U} 3/5 flying. Whenever you cast an
/// artifact spell, you may copy it, the copy not legendary; once each turn.
pub fn iron_man_bleeding_edge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Artifact))
                .once_per_turn(),
            effect: Effect::MayDo {
                description: "Copy that artifact spell (the copy isn't legendary)?".into(),
                body: Box::new(Effect::CopySpellNonLegendary { what: Selector::TriggerSource }),
            },
        }],
        ..CardDefinition {
            card_types: vec![CardType::Artifact, CardType::Creature],
            ..legend("Iron Man, Bleeding Edge", cost(&[generic(3), u(), u()]), vec![CreatureType::Human, CreatureType::Hero], 3, 5)
        }
    }
}

/// Shuri, Wakandan Inventor — {1}{U} 2/1. Artifact spells you cast cost {1}
/// less; {1}, {T}: target artifact you control becomes a copy of a second
/// target artifact you control until end of turn, not legendary; sorcery
/// speed.
pub fn shuri_wakandan_inventor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Artifact spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Artifact, amount: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::BecomeCopyOfFor {
                what: target_filtered(R::Artifact.and(R::ControlledByYou)),
                source: Selector::TargetFiltered { slot: 1, filter: R::Artifact.and(R::ControlledByYou) },
                duration: Duration::EndOfTurn,
                non_legendary: true,
            },
            ..Default::default()
        }],
        ..legend(
            "Shuri, Wakandan Inventor",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Artificer, CreatureType::Hero],
            2,
            1,
        )
    }
}

/// The Invincible Iron Man — Tony Stark's back face: 5/5 flying, haste; at
/// the beginning of combat on your turn, you may put an artifact card from
/// your hand onto the battlefield, attaching it to this if it's an Equipment.
pub fn the_invincible_iron_man() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Put an artifact card from your hand onto the battlefield?".into(),
                body: Box::new(Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Artifact,
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: Some(Box::new(Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::LastMoved,
                            filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
                        },
                        then: Box::new(Effect::Attach { what: Selector::LastMoved, to: Selector::This }),
                        else_: Box::new(Effect::Noop),
                    })),
                }),
            },
        }],
        ..legend("The Invincible Iron Man", cost(&[generic(4), u(), r()]), vec![CreatureType::Human, CreatureType::Hero], 5, 5)
    }
}

/// Tony Stark — {1}{U} 1/3. {1}, {T}: look at the top four, you may reveal an
/// artifact card into your hand, the rest on the bottom in a random order;
/// {4}{U}{R}: transform it, sorcery speed.
pub fn tony_stark() -> CardDefinition {
    use crate::effect::LookPick;
    CardDefinition {
        back_face: Some(Box::new(the_invincible_iron_man())),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::LookPickToHand(Box::new(LookPick {
                    who: PlayerRef::You,
                    count: Value::Const(4),
                    pick_filter: Some(R::Artifact),
                    optional: true,
                    rest_bottom_random: true,
                    ..Default::default()
                })),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4), u(), r()]),
                sorcery_speed: true,
                effect: Effect::Transform { what: Selector::This },
                ..Default::default()
            },
        ],
        ..legend("Tony Stark", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Artificer, CreatureType::Hero], 1, 3)
    }
}

/// Vault of Catlacan — Storm the Vault's back: legendary land; {T}: any
/// color; {T}: {U} for each artifact you control.
pub fn vault_of_catlacan() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Vault of Catlacan",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(Color::Blue, Value::PermanentCountControlledByMatching(PlayerRef::You, R::Artifact)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Storm the Vault — {2}{U}{R} legendary enchantment. Whenever one or more of
/// your creatures deal combat damage to a player, a Treasure; at your end
/// step with five or more artifacts, transform it.
pub fn storm_the_vault() -> CardDefinition {
    CardDefinition {
        name: "Storm the Vault",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        back_face: Some(Box::new(vault_of_catlacan())),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature })
                    .once_per_batch(),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                    Predicate::ValueAtLeast(Value::PermanentCountControlledByMatching(PlayerRef::You, R::Artifact), Value::Const(5)),
                ),
                effect: Effect::Transform { what: Selector::This },
            },
        ],
        ..Default::default()
    }
}
