//! Commander: the cards the **Tyranid Swarm** precon (40K, The Swarmlord)
//! needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_swarmlord.rs`.
//!
//! Residuals (each also on its card):
//! - **Ghyrson Starn, Kelermorph**, **The Red Terror** — only a permanent
//!   source's damage is seen (not an instant's or sorcery's).
//! - **Hierophant Bio-Titan** — the caster always removes the counters that
//!   buy the most discount (up to five), from the creatures carrying the
//!   most; it doesn't choose.
//! - **Magus Lucea Kane** — only the next spell with {X} is copied, not an
//!   ability with {X}.
//! - **The First Tyrannic War** — chapter I's counters are put on after the
//!   creature enters, not as it enters.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, evolve, on_attack, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, u, x, Color, ManaCost, SpendRestriction};
use crate::sets::{tap_add_any_color, tap_add_colorless};

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

fn tyranid(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    creature(name, mana, vec![CreatureType::Tyranid], p, t)
}

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

/// CR 702.156 — Ravenous: enters with X +1/+1 counters, and draws a card on
/// entering if X is 5 or more (read off its counters, as the catalog's
/// other Ravenous creatures do).
fn ravenous(def: CardDefinition) -> CardDefinition {
    let mut triggered_abilities = vec![etb(Effect::If {
        cond: Predicate::ValueAtLeast(
            Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
            Value::Const(5),
        ),
        then: Box::new(draw(1)),
        else_: Box::new(Effect::Noop),
    })];
    triggered_abilities.extend(def.triggered_abilities.clone());
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities,
        ..def
    }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn plus(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn token(name: &str, color: Color, p: i32, t: i32, keywords: Vec<Keyword>, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn make(t: TokenDefinition, n: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(t) }
}

fn warrior() -> TokenDefinition {
    token("Tyranid Warrior", Color::Green, 3, 3, vec![Keyword::Trample], vec![CreatureType::Tyranid, CreatureType::Warrior])
}

fn gargoyle() -> TokenDefinition {
    token("Tyranid Gargoyle", Color::Blue, 1, 1, vec![Keyword::Flying], vec![CreatureType::Tyranid, CreatureType::Gargoyle])
}

fn basic_land_tapped() -> Effect {
    Effect::Search {
        who: PlayerRef::You,
        filter: R::IsBasicLand,
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
    }
}

fn combat_damage(scope: EventScope, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, scope), effect }
}

fn anthem(filter: R, keywords: Vec<Keyword>) -> StaticEffect {
    StaticEffect::AnthemForFilter {
        filter,
        power: 0,
        toughness: 0,
        keywords,
        opponents: false,
        all_players: false,
        only_your_turn: false,
        scale_by_counters_on_self: None,
    }
}

/// Aberrant — ravenous, trample; combat damage to a player destroys one of
/// that player's artifacts or enchantments.
pub fn aberrant() -> CardDefinition {
    ravenous(CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![combat_damage(
            EventScope::SelfSource,
            Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment).and(R::ControlledByTriggerPlayer)),
            },
        )],
        ..creature("Aberrant", cost(&[x(), generic(1), g()]), vec![CreatureType::Tyranid, CreatureType::Mutant], 0, 0)
    })
}

/// Acolyte Hybrid — attacking, destroy up to one target artifact; its
/// controller draws a card.
pub fn acolyte_hybrid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Artifact) },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::PermanentsDestroyedThisResolution, Value::ONE),
                    then: Box::new(Effect::Draw {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::DestroyedThisResolution {
                            filter: R::Artifact,
                        }))),
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
        ..creature("Acolyte Hybrid", cost(&[generic(2), r()]), vec![CreatureType::Tyranid, CreatureType::Human], 2, 2)
    }
}

/// Atalan Jackal — trample, haste; combat damage to a player may fetch a
/// basic land tapped.
pub fn atalan_jackal() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![combat_damage(
            EventScope::SelfSource,
            Effect::MayDo { description: "Search for a basic land?".into(), body: Box::new(basic_land_tapped()) },
        )],
        ..creature(
            "Atalan Jackal",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Human, CreatureType::Tyranid, CreatureType::Scout],
            2,
            2,
        )
    }
}

/// Biophagus — {T}: one mana of any color; a creature spell it funds enters
/// with an extra +1/+1 counter.
pub fn biophagus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                    SpendRestriction::CreatureCastCounter,
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Biophagus",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Tyranid, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Bone Sabres — equipped creature attacking gets four +1/+1 counters.
pub fn bone_sabres() -> CardDefinition {
    CardDefinition {
        name: "Bone Sabres",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![on_attack(plus(Selector::This, Value::Const(4)))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Broodlord — ravenous; entering, distribute X +1/+1 counters among your
/// other creatures.
pub fn broodlord() -> CardDefinition {
    ravenous(CardDefinition {
        triggered_abilities: vec![etb(Effect::DistributeCounters {
            total: Value::XFromCost,
            counter: CounterType::PlusOnePlusOne,
            filter: yours(R::Creature).and(R::OtherThanSource),
            max_targets: 8,
        })],
        ..tyranid("Broodlord", cost(&[x(), generic(3), g()]), 3, 3)
    })
}

/// Cave of Temptation — {T}: {C}; {1}, {T}: any color; {4}, {T}, sacrifice:
/// two +1/+1 counters on target creature (sorcery speed).
pub fn cave_of_temptation() -> CardDefinition {
    CardDefinition {
        name: "Cave of Temptation",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility { mana_cost: cost(&[generic(1)]), ..tap_add_any_color() },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: plus(target_filtered(R::Creature), Value::Const(2)),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Clamavus — each creature you control gets +1/+1 for each +1/+1 counter
/// on it.
pub fn clamavus() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature you control gets +1/+1 for each +1/+1 counter on it.",
            effect: StaticEffect::PumpTeamByControlledPermanents {
                applies_to: R::Creature,
                count_filter: R::Creature,
                per_power: 1,
                per_toughness: 1,
                count_graveyard: false,
                exclude_self: false,
                per_own_color: false,
                per_own_counter: Some(CounterType::PlusOnePlusOne),
            },
        }],
        ..creature(
            "Clamavus",
            cost(&[generic(4), g()]),
            vec![CreatureType::Human, CreatureType::Tyranid, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Deathleaper, Terror Weapon — flash, haste; your creatures that entered
/// this turn have double strike.
pub fn deathleaper_terror_weapon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control that entered this turn have double strike.",
            effect: anthem(yours(R::Creature).and(R::EnteredThisTurn), vec![Keyword::DoubleStrike]),
        }],
        ..legend(tyranid("Deathleaper, Terror Weapon", cost(&[generic(2), r(), g()]), 3, 3))
    }
}

/// Exocrine — ravenous; entering, X damage to each player and each other
/// creature.
pub fn exocrine() -> CardDefinition {
    ravenous(CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::XFromCost },
            Effect::DealDamage { to: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)), amount: Value::XFromCost },
        ]))],
        ..tyranid("Exocrine", cost(&[x(), generic(2), r()]), 2, 2)
    })
}

/// Gargoyle Flock — flying; your end step, if a creature entered under your
/// control this turn, a 1/1 flying Gargoyle.
pub fn gargoyle_flock() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::CreatureEnteredThisTurn { who: PlayerRef::You }),
            effect: make(gargoyle(), Value::ONE),
        }],
        ..creature(
            "Gargoyle Flock",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Tyranid, CreatureType::Gargoyle],
            2,
            2,
        )
    }
}

/// Genestealer Locus — a creature attacking you gets -1/-0; one attacking
/// your opponents gets +0/+1.
pub fn genestealer_locus() -> CardDefinition {
    let pump = |p: i32, t: i32| Effect::PumpPT {
        what: Selector::TriggerSource,
        power: Value::Const(p),
        toughness: Value::Const(t),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
                effect: pump(-1, 0),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer)
                    .with_filter(trigger_is(R::IsAttackingAnOpponent)),
                effect: pump(0, 1),
            },
        ],
        ..creature(
            "Genestealer Locus",
            cost(&[generic(3), u()]),
            vec![CreatureType::Tyranid, CreatureType::Human],
            3,
            3,
        )
    }
}

/// Genestealer Patriarch — attacking, an infection counter on a defending
/// creature; a creature with an infection counter dying gives you a Tyranid
/// copy of it.
pub fn genestealer_patriarch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                kind: CounterType::Infection,
                amount: Value::ONE,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                    .with_filter(trigger_is(R::WithCounter(CounterType::Infection))),
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![CreatureType::Tyranid],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
            },
        ],
        ..tyranid("Genestealer Patriarch", cost(&[generic(4), u()]), 4, 4)
    }
}

/// Ghyrson Starn, Kelermorph — ward {2}; another source of yours dealing
/// exactly 1 damage has Ghyrson deal 2 more to the same recipient.
/// Residual: only a permanent source's damage is seen.
pub fn ghyrson_starn_kelermorph() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::ValueEquals(Value::TriggerEventAmount, Value::ONE),
                Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf)),
            ])),
            effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
        }],
        ..legend(creature(
            "Ghyrson Starn, Kelermorph",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Tyranid, CreatureType::Human],
            3,
            2,
        ))
    }
}

/// Goliath Truck — Vehicle 4/4, crew 2; attacking, two +1/+1 counters on
/// another attacking creature.
pub fn goliath_truck() -> CardDefinition {
    CardDefinition {
        name: "Goliath Truck",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![on_attack(plus(
            target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
            Value::Const(2),
        ))],
        ..Default::default()
    }
}

/// Haruspex — another creature dying gives it a +1/+1 counter; {T}, remove
/// X counters: X mana of any one color.
pub fn haruspex() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf))),
            effect: plus(Selector::This, Value::ONE),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_x: Some(CounterType::PlusOnePlusOne),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::XFromCost) },
            ..Default::default()
        }],
        ..tyranid("Haruspex", cost(&[generic(3), g()]), 2, 2)
    }
}

/// Hierophant Bio-Titan — removing +1/+1 counters from your creatures makes
/// it {2} cheaper each; reach, vigilance, ward {2}; power 2 or less can't
/// block it.
/// Residual: the counters that buy the most discount (up to five) are always
/// removed, from the creatures carrying the most.
pub fn hierophant_bio_titan() -> CardDefinition {
    let removable = || {
        Value::Min(
            Box::new(Value::CountersOn {
                what: Box::new(Selector::EachPermanent(yours(R::Creature))),
                kind: CounterType::PlusOnePlusOne,
            }),
            Box::new(Value::Const(5)),
        )
    };
    CardDefinition {
        keywords: vec![
            Keyword::Reach,
            Keyword::Vigilance,
            Keyword::Ward(WardCost::generic(2)),
            Keyword::CantBeBlockedByPowerAtMost(2),
        ],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {2} less to cast for each +1/+1 counter removed this way.",
            effect: StaticEffect::SelfCostReducedByValue {
                amount: Value::Times(Box::new(removable()), Box::new(Value::Const(2))),
            },
        }],
        additional_cast_cost: vec![AdditionalCastCost::RemoveCountersAmong {
            kind: CounterType::PlusOnePlusOne,
            count: removable(),
        }],
        ..tyranid("Hierophant Bio-Titan", cost(&[generic(10), g(), g()]), 12, 12)
    }
}

/// Hormagaunt Horde — ravenous; from your graveyard, a land entering under
/// your control lets you pay {2}{G} to return it to hand.
pub fn hormagaunt_horde() -> CardDefinition {
    ravenous(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::FromYourGraveyard)
                .with_filter(trigger_is(yours(R::Land))),
            effect: Effect::MayPay {
                description: "Pay {2}{G} to return Hormagaunt Horde to your hand?".into(),
                mana_cost: cost(&[generic(2), g()]),
                body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) }),
                else_: None,
            },
        }],
        ..tyranid("Hormagaunt Horde", cost(&[x(), g()]), 1, 1)
    })
}

/// Lictor — flash; entering, if a creature entered under an opponent's
/// control this turn, a 3/3 trample Tyranid Warrior.
pub fn lictor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::AnOpponentHadCreaturesEnterAtLeast(1)),
            effect: make(warrior(), Value::ONE),
        }],
        ..tyranid("Lictor", cost(&[generic(3), g()]), 3, 3)
    }
}

/// Magus Lucea Kane — each combat on your turn, a +1/+1 counter on target
/// creature; {T}: {C}{C}, and the next spell with {X} this turn is copied.
/// Residual: an ability with {X} isn't copied.
pub fn magus_lucea_kane() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: plus(target_filtered(R::Creature), Value::ONE),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(2)) },
                Effect::OnYourNextSpellMatchingThisTurn {
                    filter: R::HasXInCost,
                    body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
                },
            ]),
            ..Default::default()
        }],
        ..legend(creature(
            "Magus Lucea Kane",
            cost(&[generic(1), g(), u(), r()]),
            vec![CreatureType::Human, CreatureType::Tyranid, CreatureType::Wizard],
            1,
            1,
        ))
    }
}

/// Malanthrope — flying; entering, exile target player's graveyard and take
/// a +1/+1 counter per creature card exiled.
pub fn malanthrope() -> CardDefinition {
    let yard = |filter: R| Selector::CardsInZone { who: PlayerRef::Target(0), zone: Zone::Graveyard, filter };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            // Declares slot 0 as "target player" (an inert life loss of 0).
            Effect::LoseLife { who: target_filtered(R::Player), amount: Value::Const(0) },
            plus(Selector::This, Value::CountOf(Box::new(yard(R::Creature)))),
            Effect::Move { what: yard(R::Any), to: ZoneDest::Exile },
        ]))],
        ..tyranid("Malanthrope", cost(&[generic(1), g(), u()]), 2, 2)
    }
}

/// Mawloc — ravenous; entering, it fights up to one opposing creature,
/// which is exiled instead if it would die this turn.
pub fn mawloc() -> CardDefinition {
    ravenous(CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::ExileIfWouldDieThisTurn { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
                Effect::Fight { attacker: Selector::This, defender: Selector::Target(0) },
            ])),
        })],
        ..tyranid("Mawloc", cost(&[x(), r(), g()]), 2, 2)
    })
}

/// Nexos — your basic lands have "{T}: Add {C}{C}; spend only on costs with
/// {X}".
pub fn nexos() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Basic lands you control have \"{T}: Add {C}{C}. Spend this mana only on costs that contain {X}.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(yours(R::IsBasicLand)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Restricted(
                            Box::new(ManaPayload::Colorless(Value::Const(2))),
                            SpendRestriction::XCostsOnly,
                        ),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..creature(
            "Nexos",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Tyranid, CreatureType::Advisor],
            2,
            2,
        )
    }
}

/// Old One Eye — trample, and your other creatures have trample; entering,
/// a 5/5 Tyranid; from your graveyard, at your first main phase, discard
/// two to return it to hand.
pub fn old_one_eye() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have trample.",
            effect: anthem(yours(R::Creature).and(R::OtherThanSource), vec![Keyword::Trample]),
        }],
        triggered_abilities: vec![
            etb(make(token("Tyranid", Color::Green, 5, 5, vec![], vec![CreatureType::Tyranid]), Value::ONE)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::FromYourGraveyard),
                effect: Effect::MayDiscard {
                    description: "Discard two cards to return Old One Eye to your hand?".into(),
                    count: Value::Const(2),
                    then: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) }),
                    else_: None,
                },
            },
        ],
        ..legend(tyranid("Old One Eye", cost(&[generic(5), g()]), 6, 6))
    }
}

/// Purestrain Genestealer — enters with two +1/+1 counters; attacking, it
/// may trade one for a basic land.
pub fn purestrain_genestealer() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                Value::ONE,
            ),
            then: Box::new(Effect::MayDo {
                description: "Remove a +1/+1 counter to search for a basic land?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::RemoveCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    basic_land_tapped(),
                ])),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..tyranid("Purestrain Genestealer", cost(&[generic(2), g()]), 1, 1)
    }
}

/// Ravener — flash, ravenous; entering, target creature attacks target
/// opponent this turn if able.
pub fn ravener() -> CardDefinition {
    ravenous(CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::MustAttackPlayerThisTurn {
            attacker: target_filtered(R::Creature),
            defender: Selector::TargetFiltered { slot: 1, filter: R::OpponentPlayer },
        })],
        ..tyranid("Ravener", cost(&[x(), g(), u()]), 0, 0)
    })
}

/// Screamer-Killer — trample; casting a creature spell of mana value 5+
/// deals 5 damage to any target.
pub fn screamer_killer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature.and(R::ManaValueAtLeast(5)))),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(5) },
        }],
        ..tyranid("Screamer-Killer", cost(&[generic(4), r()]), 5, 5)
    }
}

/// Sporocyst — ravenous, defender; entering, up to X basic lands tapped.
pub fn sporocyst() -> CardDefinition {
    ravenous(CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![etb(Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            count: Value::XFromCost,
        })],
        ..tyranid("Sporocyst", cost(&[x(), x(), g()]), 0, 0)
    })
}

/// Tervigon — ravenous, trample; combat damage to a player makes that many
/// 1/1 Tyranids.
pub fn tervigon() -> CardDefinition {
    ravenous(CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![combat_damage(
            EventScope::SelfSource,
            make(token("Tyranid", Color::Green, 1, 1, vec![], vec![CreatureType::Tyranid]), Value::TriggerEventAmount),
        )],
        ..tyranid("Tervigon", cost(&[x(), generic(1), g()]), 0, 0)
    })
}

/// The First Tyrannic War — I: a creature card from your hand onto the
/// battlefield, with a counter per land if it has {X}; II, III: double the
/// counters on target creature you control.
/// Residual: chapter I's counters go on after it enters.
pub fn the_first_tyrannic_war() -> CardDefinition {
    let double = || Effect::DoubleAllCountersOn { what: target_filtered(yours(R::Creature)) };
    CardDefinition {
        name: "The First Tyrannic War",
        cost: cost(&[generic(2), g(), u(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Creature,
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: Some(Box::new(Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::LastMoved, filter: R::HasXInCost },
                        then: Box::new(plus(
                            Selector::LastMoved,
                            Value::PermanentCountControlledByMatching(PlayerRef::You, R::Land),
                        )),
                        else_: Box::new(Effect::Noop),
                    })),
                },
            ),
            (2, double()),
            (3, double()),
        ],
        ..Default::default()
    }
}

/// The Red Terror — a red source of yours dealing damage gives it a +1/+1
/// counter.
/// Residual: only a permanent source's damage is seen.
pub fn the_red_terror() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::YourControl)
                .with_filter(trigger_is(R::HasColor(Color::Red)))
                .once_per_batch(),
            effect: plus(Selector::This, Value::ONE),
        }],
        ..legend(tyranid("The Red Terror", cost(&[generic(3), r()]), 4, 3))
    }
}

/// The Swarmlord — enters with two +1/+1 counters per command-zone cast of
/// your commander; a creature of yours with a counter dying draws a card.
pub fn the_swarmlord() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Times(Box::new(Value::CommanderCastsFromCommandZone(PlayerRef::You)), Box::new(Value::Const(2))),
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::TriggerSourceHadCounters),
            effect: draw(1),
        }],
        ..legend(tyranid("The Swarmlord", cost(&[generic(3), g(), u(), r()]), 5, 5))
    }
}

/// Toxicrene — reach, deathtouch; all lands tap for any color and lose
/// their other abilities.
pub fn toxicrene() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Deathtouch],
        static_abilities: vec![
            StaticAbility {
                description: "All lands lose all other abilities.",
                effect: StaticEffect::MatchingLoseAllAbilities { applies_to: Selector::EachPermanent(R::Land) },
            },
            StaticAbility {
                description: "All lands have \"{T}: Add one mana of any color.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::EachPermanent(R::Land),
                    ability: tap_add_any_color(),
                    condition: None,
                },
            },
        ],
        ..tyranid("Toxicrene", cost(&[generic(3), g()]), 2, 4)
    }
}

/// Trygon Prime — attacking, a +1/+1 counter on it and on up to one other
/// attacker, which can't be blocked this turn.
pub fn trygon_prime() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            plus(Selector::This, Value::ONE),
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    plus(target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)), Value::ONE),
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
                ])),
            },
        ]))],
        ..tyranid("Trygon Prime", cost(&[generic(2), g(), u()]), 4, 4)
    }
}

/// Tyranid Harridan — flying, ward {4}; your Tyranids dealing combat damage
/// to a player make 1/1 flying Gargoyles.
pub fn tyranid_harridan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::generic(4))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(trigger_is(R::HasCreatureType(CreatureType::Tyranid))),
            effect: make(gargoyle(), Value::ONE),
        }],
        ..tyranid("Tyranid Harridan", cost(&[generic(4), g(), u()]), 4, 4)
    }
}

/// Tyranid Invasion — a 3/3 trample Tyranid Warrior per opponent.
pub fn tyranid_invasion() -> CardDefinition {
    CardDefinition {
        name: "Tyranid Invasion",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: make(warrior(), Value::OpponentCount),
        ..Default::default()
    }
}

/// Tyranid Prime — evolve, and your other creatures have evolve.
pub fn tyranid_prime() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![evolve()],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have evolve.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: yours(R::Creature).and(R::OtherThanSource),
                ability: Box::new(evolve()),
            },
        }],
        ..tyranid("Tyranid Prime", cost(&[generic(1), g(), u()]), 0, 4)
    }
}

/// Venomthrope — flying, deathtouch, hexproof.
pub fn venomthrope() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Hexproof],
        ..tyranid("Venomthrope", cost(&[generic(1), g(), u()]), 2, 2)
    }
}

/// Winged Hive Tyrant — flying, haste; your other creatures with counters
/// have flying and haste.
pub fn winged_hive_tyrant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control with counters on them have flying and haste.",
            effect: anthem(
                yours(R::Creature).and(R::OtherThanSource).and(R::WithAnyCounter),
                vec![Keyword::Flying, Keyword::Haste],
            ),
        }],
        ..tyranid("Winged Hive Tyrant", cost(&[generic(3), u(), r()]), 4, 4)
    }
}

/// Zoanthrope — ravenous, flying, ward {2}; entering, X damage to any
/// target.
pub fn zoanthrope() -> CardDefinition {
    ravenous(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![etb(Effect::DealDamage { to: target_any(), amount: Value::XFromCost })],
        ..tyranid("Zoanthrope", cost(&[x(), u(), r()]), 0, 0)
    })
}
