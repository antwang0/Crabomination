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
