//! Modern Horizons 3 (MH3), batch 2 — Eldrazi/colorless matters, adapt/modified
//! payoffs, living-weapon equipment, and modal/overload spells. All ride
//! existing engine primitives. Tests in `tests/mh3b.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{adapt, drain, etb, on_attack, on_cast, target_filtered, unearth};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest};
use crate::mana::{b, colorless, cost, g, generic, r, u, w, Color};

fn germ() -> TokenDefinition {
    TokenDefinition {
        name: "Phyrexian Germ".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian], ..Default::default() },
        ..Default::default()
    }
}

/// Living-weapon ETB: mint a Germ and attach this Equipment to it.
fn living_weapon() -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: germ() },
        Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
    ]))
}

// ── Eldrazi / colorless ──────────────────────────────────────────────────────

/// Eldrazi Ravager — {5}{C} 6/6 Eldrazi. Annihilator 1. Sacrifice two Eldrazi:
/// return this from your graveyard to your hand. Cycling {2}.
pub fn eldrazi_ravager() -> CardDefinition {
    CardDefinition {
        name: "Eldrazi Ravager",
        cost: cost(&[generic(5), colorless(1)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Annihilator(1), Keyword::Cycling(cost(&[generic(2)]))],
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Eldrazi), 2)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Breaker of Creation — {6}{C}{C} 8/4 Eldrazi. When you cast this, gain 1 life
/// for each colorless permanent you control. Hexproof from each color.
/// Annihilator 2.
pub fn breaker_of_creation() -> CardDefinition {
    CardDefinition {
        name: "Breaker of Creation",
        cost: cost(&[generic(6), colorless(1), colorless(1)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 8,
        toughness: 4,
        keywords: vec![
            Keyword::Annihilator(2),
            Keyword::HexproofFromColor(Color::White),
            Keyword::HexproofFromColor(Color::Blue),
            Keyword::HexproofFromColor(Color::Black),
            Keyword::HexproofFromColor(Color::Red),
            Keyword::HexproofFromColor(Color::Green),
        ],
        triggered_abilities: vec![on_cast(Effect::GainLife {
            who: Selector::You,
            amount: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(R::ControlledByYou.and(R::Colorless))),
                filter: R::ControlledByYou.and(R::Colorless),
            },
        })],
        ..Default::default()
    }
}

/// Drownyard Lurker — {7} 7/7 Eldrazi Trilobite. Vigilance. When you cast or
/// cycle this, create a 0/1 Eldrazi Spawn. Cycling {2}{U}.
pub fn drownyard_lurker() -> CardDefinition {
    let make_spawn = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: crate::game::effects::eldrazi_spawn_token(),
    };
    CardDefinition {
        name: "Drownyard Lurker",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Trilobite],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Vigilance, Keyword::Cycling(cost(&[generic(2), u()]))],
        triggered_abilities: vec![
            on_cast(make_spawn()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
                effect: make_spawn(),
            },
        ],
        ..Default::default()
    }
}

/// Emrakul's Messenger — {1}{U} 2/1 Devoid Eldrazi Faerie Rogue. Flying.
/// Whenever you draw your second card each turn, create a 0/1 Eldrazi Spawn.
pub fn emrakuls_messenger() -> CardDefinition {
    CardDefinition {
        name: "Emrakul's Messenger",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Devoid, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::Triggerer, n: 2 })
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crate::game::effects::eldrazi_spawn_token(),
            },
        }],
        ..Default::default()
    }
}

/// Petrifying Meddler — {4}{U} 4/5 Devoid Eldrazi. Reach. When you cast this,
/// tap up to one target creature and put a stun counter on it.
pub fn petrifying_meddler() -> CardDefinition {
    CardDefinition {
        name: "Petrifying Meddler",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Devoid, Keyword::Reach],
        triggered_abilities: vec![on_cast(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::Stun,
                amount: Value::ONE,
            },
        ]))],
        ..Default::default()
    }
}

/// Sage of the Unknowable — {1}{U} 0/4 Human Wizard. {T}: Add {C}, spendable
/// only on a colorless spell or to activate an ability.
pub fn sage_of_the_unknowable() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Sage of the Unknowable",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::ONE)),
                    crate::mana::SpendRestriction::ColorlessSpellsOrAbilities,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Riddle Gate Gargoyle — {W}{U} 2/2 Artifact Gargoyle. Flying. ETB: get
/// {E}{E}{E}. Whenever you attack, you may pay {E}{E}; if you do, target
/// creature you control gains lifelink until end of turn.
pub fn riddle_gate_gargoyle() -> CardDefinition {
    CardDefinition {
        name: "Riddle Gate Gargoyle",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gargoyle], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::Const(3))),
            crate::effect::shortcut::on_you_attack(Effect::MayDo {
                description: "Pay {E}{E}?".into(),
                body: Box::new(Effect::PayEnergy {
                    amount: 2,
                    // CR 603.7 — the lifelink target is chosen *after* the {E}{E}
                    // is paid, via the reflexive "when you do" payoff.
                    then: Box::new(Effect::Reflexive {
                        body: Box::new(Effect::GrantKeyword {
                            what: target_filtered(R::Creature.and(R::ControlledByYou)),
                            keyword: Keyword::Lifelink,
                            duration: Duration::EndOfTurn,
                        }),
                    }),
                }),
            }),
        ],
        ..Default::default()
    }
}

/// Thraben Charm — {1}{W} Instant. Choose one — deal damage equal to twice the
/// creatures you control to target creature; destroy target enchantment; or
/// exile graveyards. (The graveyard mode wipes all graveyards.)
pub fn thraben_charm() -> CardDefinition {
    CardDefinition {
        name: "Thraben Charm",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(Value::CreatureCountControlledBy(PlayerRef::You)),
                ),
            },
            Effect::Destroy { what: target_filtered(R::Enchantment) },
            Effect::ExileAllGraveyards { filter: None, opponents_only: false },
        ]),
        ..Default::default()
    }
}

/// Voidpouncer — {1}{R} 3/1 Devoid Eldrazi. Kicker {2}{C}. If kicked, it enters
/// with two +1/+1 counters, a trample counter, and haste.
pub fn voidpouncer() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Voidpouncer",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Devoid, Keyword::Kicker(cost(&[generic(2), colorless(1)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
                    Effect::AddKeywordCounter { what: Selector::This, keyword: Keyword::Trample, amount: Value::ONE },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Scurry of Gremlins — {2}{R}{W} Enchantment. ETB: make two 1/1 red Gremlins,
/// then get {E} equal to the number of creatures you control. Pay {E}{E}{E}{E}:
/// creatures you control get +1/+0 and gain haste until end of turn.
pub fn scurry_of_gremlins() -> CardDefinition {
    let gremlin = TokenDefinition {
        name: "Gremlin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gremlin], ..Default::default() },
        ..Default::default()
    };
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Scurry of Gremlins",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: gremlin },
            Effect::AddEnergy(Value::CreatureCountControlledBy(PlayerRef::You)),
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[]),
            effect: Effect::PayEnergy {
                amount: 4,
                then: Box::new(Effect::Seq(vec![
                    Effect::PumpPT { what: yours(), power: Value::ONE, toughness: Value::Const(0), duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: yours(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ])),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Warped Tusker — {7} 6/8 Eldrazi Boar Beast. Reach. When you cast or cycle
/// this, create a 0/1 Eldrazi Spawn. Cycling {2}{G}.
pub fn warped_tusker() -> CardDefinition {
    let spawn = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: crate::game::effects::eldrazi_spawn_token(),
    };
    CardDefinition {
        name: "Warped Tusker",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Boar, CreatureType::Beast],
            ..Default::default()
        },
        power: 6,
        toughness: 8,
        keywords: vec![Keyword::Reach, Keyword::Cycling(cost(&[generic(2), g()]))],
        triggered_abilities: vec![
            on_cast(spawn()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
                effect: spawn(),
            },
        ],
        ..Default::default()
    }
}

/// Voltstorm Angel — {3}{W}{W} 4/4 Angel. Flying. ETB: get {E}{E}{E}. At combat
/// on your turn, you may pay {E}{E}. When you do, choose one — this gains
/// vigilance and lifelink EOT; or other creatures you control get +1/+1 EOT.
pub fn voltstorm_angel() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Voltstorm Angel",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::Const(3))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
                effect: Effect::MayDo {
                    description: "Pay {E}{E}?".into(),
                    body: Box::new(Effect::PayEnergy {
                        amount: 2,
                        then: Box::new(Effect::ChooseMode(vec![
                            Effect::Seq(vec![
                                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
                            ]),
                            Effect::PumpPT {
                                what: Selector::EachPermanent(
                                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                                ),
                                power: Value::ONE,
                                toughness: Value::ONE,
                                duration: Duration::EndOfTurn,
                            },
                        ])),
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Triton Wavebreaker — {U} 1/1 Enchantment Creature. Bestow {1}{U}. Prowess.
/// As an Aura it grants +1/+1 and prowess.
pub fn triton_wavebreaker() -> CardDefinition {
    CardDefinition {
        name: "Triton Wavebreaker",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Prowess],
        bestow: Some(cost(&[generic(1), u()])),
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Prowess],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Strix Serenade — {U} Instant. Counter target artifact, creature, or
/// planeswalker spell. Its controller creates a 2/2 blue Bird with flying.
pub fn strix_serenade() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Bird".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Strix Serenade",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            // Mint the Bird for the countered spell's controller before the
            // spell leaves the stack, so `ControllerOf(Target(0))` resolves.
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::ONE,
                definition: bird,
            },
            Effect::CounterSpell {
                what: target_filtered(
                    R::IsSpellOnStack
                        .and(R::Artifact.or(R::Creature).or(R::Planeswalker)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Indebted Spirit — {W} 1/1 Enchantment Creature. Bestow {2}{W}. Afterlife 1.
/// As an Aura it grants +1/+1 and afterlife 1.
pub fn indebted_spirit() -> CardDefinition {
    use crate::effect::shortcut::afterlife;
    CardDefinition {
        name: "Indebted Spirit",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![afterlife(1)],
        bestow: Some(cost(&[generic(2), w()])),
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![afterlife(1)],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Temperamental Oozewagg — {3}{G} 4/4 Ooze Brushwagg. {2}{G}: Adapt 2.
/// Modified creatures you control have trample.
pub fn temperamental_oozewagg() -> CardDefinition {
    CardDefinition {
        name: "Temperamental Oozewagg",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ooze, CreatureType::Brushwagg],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: adapt(2),
            ..Default::default()
        }],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Modified creatures you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::IsModified),
                ),
                keyword: Keyword::Trample,
            },
        }],
        ..Default::default()
    }
}

/// Kithkin Billyrider — {2}{W} 1/3 Kithkin Knight. Double strike.
pub fn kithkin_billyrider() -> CardDefinition {
    CardDefinition {
        name: "Kithkin Billyrider",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kithkin, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::DoubleStrike],
        ..Default::default()
    }
}

/// Territory Culler — {4}{G} 7/5 Devoid Eldrazi. Reach. Landfall: look at the
/// top card; if it's a creature you may put it into your hand, otherwise you
/// may put it into your graveyard.
pub fn territory_culler() -> CardDefinition {
    CardDefinition {
        name: "Territory Culler",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 7,
        toughness: 5,
        keywords: vec![Keyword::Devoid, Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::ONE,
                rest_to_graveyard: true,
                pick_filter: Some(R::Creature),
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: true,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
            },
        }],
        ..Default::default()
    }
}

/// Quest for the Necropolis — {B} Enchantment. Landfall: put a quest counter on
/// it. {5}{B}, Sacrifice this (sorcery speed): reanimate a creature from a
/// graveyard; costs {1} less per quest counter.
pub fn quest_for_the_necropolis() -> CardDefinition {
    CardDefinition {
        name: "Quest for the Necropolis",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Quest, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b()]),
            sac_cost: true,
            sorcery_speed: true,
            self_counter_cost_reduction: Some(CounterType::Quest),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Utter Insignificance — {1}{U} Aura. Flash. Enchant creature. Enchanted
/// creature loses all abilities and has base P/T 1/1. {2}{C}: Exile it.
pub fn utter_insignificance() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Utter Insignificance",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((1, 1)),
            remove_abilities: true,
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), colorless(1)]),
            effect: Effect::Move {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                to: ZoneDest::Exile,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Trickster's Elk — {2}{G} 3/3 Enchantment Creature — Elk. Bestow {1}{G}. As
/// an Aura the enchanted creature loses all abilities and is a green 3/3 Elk.
pub fn tricksters_elk() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Trickster's Elk",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elk], ..Default::default() },
        power: 3,
        toughness: 3,
        bestow: Some(cost(&[generic(1), g()])),
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((3, 3)),
            remove_abilities: true,
            set_colors: Some(vec![Color::Green]),
            set_creature_types: Some(vec![CreatureType::Elk]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Siege Smash — {1}{R} Instant. Split second. Choose one — destroy target
/// artifact; or target creature gets +3/+2 and gains trample until end of turn.
pub fn siege_smash() -> CardDefinition {
    CardDefinition {
        name: "Siege Smash",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::SplitSecond],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            // The chosen mode owns slot 0; pump and trample grant share it.
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(3),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
        ]),
        ..Default::default()
    }
}

/// Nyxborn Hydra — {X}{G} 0/1 Enchantment Creature. Reach, trample. Enters with
/// X +1/+1 counters. Bestow {X}{G}{G}; as an Aura it grants +1/+1 per +1/+1
/// counter on it, plus reach and trample.
pub fn nyxborn_hydra() -> CardDefinition {
    use crate::card::EquipScale;
    use crate::mana::x;
    let scale = || EquipScale {
        filter: R::Any,
        per_power: 1,
        per_toughness: 1,
        count_self_counters: Some(CounterType::PlusOnePlusOne),
        ..Default::default()
    };
    CardDefinition {
        name: "Nyxborn Hydra",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hydra], ..Default::default() },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        bestow: Some(cost(&[x(), g(), g()])),
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Reach, Keyword::Trample],
            scale: Some(scale()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Glyph Elemental — {1}{W} 2/2 Enchantment Creature. Bestow {1}{W}. Landfall:
/// put a +1/+1 counter on it. As an Aura it grants +1/+1 per +1/+1 counter on it.
pub fn glyph_elemental() -> CardDefinition {
    use crate::card::EquipScale;
    CardDefinition {
        name: "Glyph Elemental",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        bestow: Some(cost(&[generic(1), w()])),
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::Any,
                per_power: 1,
                per_toughness: 1,
                count_self_counters: Some(CounterType::PlusOnePlusOne),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Skoa, Embermage — {4}{R}{R} 4/4 Legendary Goblin Wizard. ETB: deal 4 to any
/// target. Grandeur (discard another Skoa, sacrifice two Mountains): deal 4 to
/// any target.
pub fn skoa_embermage() -> CardDefinition {
    use crate::card::{LandType, Supertype};
    let bolt = || Effect::DealDamage {
        to: crate::effect::shortcut::target_any(),
        amount: Value::Const(4),
    };
    CardDefinition {
        name: "Skoa, Embermage",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(bolt())],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::HasName("Skoa, Embermage".to_string()), 1)),
            sac_other_filter: Some((R::HasLandType(LandType::Mountain), 2)),
            effect: bolt(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Eviscerator's Insight — {1}{B} Instant. Additional cost: sacrifice an
/// artifact or creature. Draw two cards. Flashback {4}{B}.
pub fn eviscerators_insight() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    CardDefinition {
        name: "Eviscerator's Insight",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(4), b()]))],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Artifact.or(R::Creature),
            count: 1,
        }],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Copycrook — {2}{U}{U} 0/0 Shapeshifter Rogue. May enter as a copy of any
/// creature, except it also has "Whenever this attacks, it connives."
pub fn copycrook() -> CardDefinition {
    use crate::card::EntersAsCopy;
    CardDefinition {
        name: "Copycrook",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter, CreatureType::Rogue],
            ..Default::default()
        },
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_triggered: vec![on_attack(crate::effect::shortcut::connive(1))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Aether Spike — {1}{U} Instant. Choose target spell. You get {E}{E}, then
/// pay any amount of {E}. Counter it unless its controller pays {1} for each
/// {E} paid this way.
pub fn aether_spike() -> CardDefinition {
    CardDefinition {
        name: "Aether Spike",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddEnergy(Value::Const(2)),
            Effect::PayAnyEnergy {
                then: Box::new(Effect::CounterUnlessPaid {
                    what: target_filtered(R::IsSpellOnStack),
                    mana_cost: cost(&[]),
                    exile: false,
                    extra_generic: Some(Value::EnergyPaidThisEffect),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Corrupted Conscience — {3}{U}{U} Aura. Enchant creature. You control the
/// enchanted creature and it has infect.
pub fn corrupted_conscience() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, StaticAbility};
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Corrupted Conscience",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![crate::effect::shortcut::etb(
            Effect::GainControlWhileSourceRemains { what: enchanted() },
        )],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has infect.",
            effect: StaticEffect::GrantKeyword { applies_to: enchanted(), keyword: Keyword::Infect },
        }],
        ..Default::default()
    }
}

/// Ghostfire Slice — {2}{R} Devoid Instant. Costs {2} less if an opponent
/// controls a multicolored permanent. Deals 4 damage to any target.
pub fn ghostfire_slice() -> CardDefinition {
    CardDefinition {
        name: "Ghostfire Slice",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        self_cost_reduction_if_control: vec![(R::Multicolored.and(R::ControlledByOpponent), 2)],
        effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(4) },
        ..Default::default()
    }
}

/// Corrupted Shapeshifter — {3}{U} Devoid Eldrazi Shapeshifter */*. As it
/// enters, it becomes your choice of a 3/3 flyer, a 2/5 with vigilance, or a
/// 0/12 with defender (`enters_as_choice`).
pub fn corrupted_shapeshifter() -> CardDefinition {
    use crate::card::EntersChoiceMode;
    CardDefinition {
        name: "Corrupted Shapeshifter",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Shapeshifter],
            ..Default::default()
        },
        keywords: vec![Keyword::Devoid],
        enters_as_choice: Some(vec![
            EntersChoiceMode { power: 3, toughness: 3, keywords: vec![Keyword::Flying] },
            EntersChoiceMode { power: 2, toughness: 5, keywords: vec![Keyword::Vigilance] },
            EntersChoiceMode { power: 0, toughness: 12, keywords: vec![Keyword::Defender] },
        ]),
        ..Default::default()
    }
}

/// Hope-Ender Coatl — {2}{U} 2/2 Devoid Eldrazi Snake. Flash, Flying. When you
/// cast this, counter target spell an opponent controls unless they pay {1}.
pub fn hope_ender_coatl() -> CardDefinition {
    CardDefinition {
        name: "Hope-Ender Coatl",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Snake],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Devoid, Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![on_cast(Effect::CounterUnlessPaid {
            what: Selector::TargetFiltered { slot: 0, filter: R::ControlledByOpponent },
            mana_cost: cost(&[generic(1)]),
            exile: false,
            extra_generic: None,
        })],
        ..Default::default()
    }
}

// ── Adapt / modified matters ─────────────────────────────────────────────────

/// Dreamdrinker Vampire — {1}{B} 2/1 Vampire. Lifelink. {1}{B}: Adapt 1.
/// Whenever one or more +1/+1 counters are put on this, it gains menace EOT.
pub fn dreamdrinker_vampire() -> CardDefinition {
    CardDefinition {
        name: "Dreamdrinker Vampire",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility { mana_cost: cost(&[generic(1), b()]), effect: adapt(1), ..Default::default() }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Evolution Witness — {2}{G} 2/1 Elf Shaman Mutant. {1}{G}: Adapt 2. Whenever
/// one or more +1/+1 counters are put on this, return target permanent card
/// from your graveyard to your hand.
pub fn evolution_witness() -> CardDefinition {
    CardDefinition {
        name: "Evolution Witness",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman, CreatureType::Mutant],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility { mana_cost: cost(&[generic(1), g()]), effect: adapt(2), ..Default::default() }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::InYourGraveyard.and(R::PermanentCard),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Expanding Ooze — {1}{B}{G} 3/3 Ooze. {B}{G}: Adapt 1. Whenever this attacks,
/// put a +1/+1 counter on target modified creature you control.
pub fn expanding_ooze() -> CardDefinition {
    CardDefinition {
        name: "Expanding Ooze",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ooze], ..Default::default() },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility { mana_cost: cost(&[b(), g()]), effect: adapt(1), ..Default::default() }],
        triggered_abilities: vec![on_attack(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::IsModified)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Envoy of the Ancestors — {2}{W} 2/3 Human Cleric. Outlast {W}. Modified
/// creatures you control have lifelink.
pub fn envoy_of_the_ancestors() -> CardDefinition {
    use crate::effect::shortcut::outlast;
    CardDefinition {
        name: "Envoy of the Ancestors",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![outlast(cost(&[w()]))],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Modified creatures you control have lifelink.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::IsModified),
                ),
                keyword: Keyword::Lifelink,
            },
        }],
        ..Default::default()
    }
}

/// Guardian of the Forgotten — {3}{W} 4/4 Elephant Warrior. Vigilance. Whenever
/// a modified creature you control dies, manifest the top card of your library.
pub fn guardian_of_the_forgotten() -> CardDefinition {
    CardDefinition {
        name: "Guardian of the Forgotten",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsModified },
            ),
            effect: Effect::Manifest { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

// ── Value creatures ──────────────────────────────────────────────────────────

/// Grim Servant — {3}{B} 3/2 Zombie Warlock. Menace. ETB: search your library
/// for a card with mana value ≤ your devotion to black to your hand, then
/// shuffle. You lose 3 life.
pub fn grim_servant() -> CardDefinition {
    CardDefinition {
        name: "Grim Servant",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::ManaValueAtMostDevotion(Color::Black),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
        ]))],
        ..Default::default()
    }
}

/// Marionette Apprentice — {1}{B} 1/2 Human Artificer. Fabricate 1. Whenever
/// another creature or artifact you control is put into a graveyard from the
/// battlefield, each opponent loses 1 life.
pub fn marionette_apprentice() -> CardDefinition {
    use crate::effect::shortcut::fabricate;
    CardDefinition {
        name: "Marionette Apprentice",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![
            fabricate(1),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureOrArtifactDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::OtherThanSource,
                    }),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Molten Gatekeeper — {2}{R} 2/3 Golem. Whenever another creature you control
/// enters, this deals 1 damage to each opponent. Unearth {R}.
pub fn molten_gatekeeper() -> CardDefinition {
    CardDefinition {
        name: "Molten Gatekeeper",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 2,
        toughness: 3,
        activated_abilities: vec![unearth(cost(&[r()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                },
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Kami of Jealous Thirst — {2}{B} 1/3 Spirit. Deathtouch. {4}{B}: each
/// opponent loses 2 life and you gain 2 life. Activate only once each turn.
pub fn kami_of_jealous_thirst() -> CardDefinition {
    CardDefinition {
        name: "Kami of Jealous Thirst",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            once_per_turn: true,
            effect: drain(2),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Infernal Captor — {3}{R} 3/3 Devil Rogue. Exploit. When this exploits a
/// creature, gain control of target artifact or creature until end of turn.
/// Untap it. It gains haste until end of turn.
pub fn infernal_captor() -> CardDefinition {
    use crate::effect::shortcut::exploit;
    CardDefinition {
        name: "Infernal Captor",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Devil, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![exploit(Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Artifact.or(R::Creature)),
                to: Some(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: target_filtered(R::Artifact.or(R::Creature)), up_to: None },
            Effect::GrantKeyword {
                what: target_filtered(R::Artifact.or(R::Creature)),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

// ── Equipment (living weapon) ────────────────────────────────────────────────

/// Colossal Dreadmask — {4}{G}{G} Equipment. Living weapon. Equipped creature
/// gets +6/+6 and has trample. Equip {3}{G}{G}.
pub fn colossal_dreadmask() -> CardDefinition {
    CardDefinition {
        name: "Colossal Dreadmask",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3), g(), g()]))],
        equipped_bonus: Some(EquipBonus { power: 6, toughness: 6, keywords: vec![Keyword::Trample], ..Default::default() }),
        triggered_abilities: vec![living_weapon()],
        ..Default::default()
    }
}

/// Drossclaw — {1}{B} Equipment. Living weapon. Equipped creature gets +1/+1.
/// Whenever equipped creature attacks, each opponent loses 1 life. Equip {2}.
pub fn drossclaw() -> CardDefinition {
    CardDefinition {
        name: "Drossclaw",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![on_attack(Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            })],
            ..Default::default()
        }),
        triggered_abilities: vec![living_weapon()],
        ..Default::default()
    }
}

// ── Spells ───────────────────────────────────────────────────────────────────

/// Horrific Assault — {G} Sorcery. Target creature you control deals damage
/// equal to its power to target creature or planeswalker you don't control. If
/// you control an Eldrazi, you gain 3 life.
pub fn horrific_assault() -> CardDefinition {
    CardDefinition {
        name: "Horrific Assault",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamageEqualToPower {
                source: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                target: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent),
                },
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Eldrazi).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
                then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Brainsurge — {2}{U} Instant. Draw four cards, then put two cards from your
/// hand on top of your library in any order.
pub fn brainsurge() -> CardDefinition {
    CardDefinition {
        name: "Brainsurge",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(4) },
            Effect::PutOnLibraryFromHand { who: PlayerRef::You, count: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Fangs of Kalonia — {1}{G} Sorcery. Put a +1/+1 counter on target creature
/// you control, then double the number of +1/+1 counters on each creature that
/// had a counter put on it this way. Overload {4}{G}{G}.
pub fn fangs_of_kalonia() -> CardDefinition {
    use crate::card::AlternativeCost;
    let each = Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Fangs of Kalonia",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::DoubleCountersOnEach {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
            },
        ]),
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(4), g(), g()]),
            effect_override: Some(Effect::Seq(vec![
                Effect::AddCounter { what: each.clone(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::DoubleCountersOnEach { what: each, kind: CounterType::PlusOnePlusOne },
            ])),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Gravedig — {1}{B} Sorcery. Choose one — target player creates a 2/2 black
/// Zombie; or return target creature card from your graveyard to your hand.
/// Entwine {2}.
pub fn gravedig() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Gravedig",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Entwine(cost(&[generic(2)]))],
        effect: Effect::ChooseMode(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: zombie },
            Effect::Move {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::InYourGraveyard) },
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

// ── Batch 2 ──────────────────────────────────────────────────────────────────

use crate::card::StaticAbility;
use crate::effect::shortcut::{amass_zombies, modular_dies};
use crate::game::TurnStep;

fn spirit_flyer() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Metastatic Evangel — {1}{W} 3/1 Phyrexian Human Cleric. Whenever another
/// nontoken creature you control enters, proliferate.
pub fn metastatic_evangel() -> CardDefinition {
    CardDefinition {
        name: "Metastatic Evangel",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken).and(R::OtherThanSource),
                },
            ),
            effect: Effect::Proliferate,
        }],
        ..Default::default()
    }
}

/// Muster the Departed — {2}{W} Enchantment. ETB: create a 1/1 white flying
/// Spirit. Morbid — at the beginning of your end step, if a creature died this
/// turn, populate.
pub fn muster_the_departed() -> CardDefinition {
    CardDefinition {
        name: "Muster the Departed",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: spirit_flyer() }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::SelfSource)
                    .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::If {
                    cond: Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE },
                    then: Box::new(Effect::Populate { who: PlayerRef::You }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Obstinate Gargoyle — {1}{W}{B} 2/2 Gargoyle. Flying while modified. Persist.
pub fn obstinate_gargoyle() -> CardDefinition {
    CardDefinition {
        name: "Obstinate Gargoyle",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gargoyle], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Persist],
        static_abilities: vec![StaticAbility {
            description: "This creature has flying as long as it's modified.",
            effect: StaticEffect::SelfHasKeywordWhile { keyword: Keyword::Flying, condition: R::IsModified },
        }],
        ..Default::default()
    }
}

/// Arcbound Condor — {2}{B}{B} 0/0 Artifact Bird. Flying. Modular 3. Whenever
/// another artifact you control enters, target creature an opponent controls
/// gets -1/-1 until end of turn.
pub fn arcbound_condor() -> CardDefinition {
    CardDefinition {
        name: "Arcbound Condor",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        keywords: vec![Keyword::Flying, Keyword::Modular(3)],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![
            modular_dies(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Artifact.and(R::OtherThanSource),
                    },
                ),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Kozilek's Unsealing — {2}{U} Devoid Enchantment. Cast a creature with MV
/// 4–6: create two Eldrazi Spawn. Cast a creature with MV 7+: draw three.
pub fn kozileks_unsealing() -> CardDefinition {
    let cast_creature_mv = |lo: u32, hi: Option<u32>, effect: Effect| {
        let mut f = R::Creature.and(R::ManaValueAtLeast(lo));
        if let Some(h) = hi {
            f = f.and(R::ManaValueAtMost(h));
        }
        TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: f }),
            effect,
        }
    };
    CardDefinition {
        name: "Kozilek's Unsealing",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![
            cast_creature_mv(
                4,
                Some(6),
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: crate::game::effects::eldrazi_spawn_token(),
                },
            ),
            cast_creature_mv(7, None, Effect::Draw { who: Selector::You, amount: Value::Const(3) }),
        ],
        ..Default::default()
    }
}

/// Mindless Conscription — {2}{B} Enchantment. When it enters and whenever you
/// draw your third card each turn, amass Zombies 3.
pub fn mindless_conscription() -> CardDefinition {
    CardDefinition {
        name: "Mindless Conscription",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(amass_zombies(3)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                    .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::Triggerer, n: 3 })
                    .once_per_turn(),
                effect: amass_zombies(3),
            },
        ],
        ..Default::default()
    }
}

/// Essence Reliquary — {2}{W} Artifact. {T}: return another target permanent
/// you control to its owner's hand. Activate only during your turn.
pub fn essence_reliquary() -> CardDefinition {
    CardDefinition {
        name: "Essence Reliquary",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Move {
                what: target_filtered(R::Permanent.and(R::ControlledByYou).and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Etched Slith — {1}{B} 1/1 Artifact Phyrexian Slith. Menace. Whenever it
/// deals combat damage to a player, put a +1/+1 counter on it.
pub fn etched_slith() -> CardDefinition {
    CardDefinition {
        name: "Etched Slith",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Slith],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}


// ── Batch 3 ──────────────────────────────────────────────────────────────────

use crate::effect::shortcut::target_any;

/// Cyclops Superconductor — {1}{U}{R} 2/2 Cyclops Wizard. Prowess. ETB: get
/// {E}{E}{E}. Dies: you may pay {E}{E}{E}; if you do, it deals damage equal to
/// its power to any target.
pub fn cyclops_superconductor() -> CardDefinition {
    CardDefinition {
        name: "Cyclops Superconductor",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cyclops, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::Const(3))),
            crate::effect::shortcut::on_dies(Effect::MayDo {
                description: "Pay {E}{E}{E} to deal damage equal to its power?".into(),
                body: Box::new(Effect::PayEnergy {
                    amount: 3,
                    then: Box::new(Effect::DealDamageEqualToPower {
                        source: Selector::This,
                        target: target_any(),
                    }),
                }),
            }),
        ],
        ..Default::default()
    }
}

/// Electrozoa — {2}{U} 3/1 Jellyfish. Flash, Flying. ETB: get {E}{E}. At the
/// beginning of your first main phase, tap it unless you pay {E}.
pub fn electrozoa() -> CardDefinition {
    CardDefinition {
        name: "Electrozoa",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Jellyfish], ..Default::default() },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::Const(2))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::SelfSource)
                    .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::PayEnergyOrElse {
                    amount: 1,
                    otherwise: Box::new(Effect::Tap { what: Selector::This }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Dreamtide Whale — {2}{U} 7/5 Whale. Vanishing 2. Whenever a player casts
/// their second spell each turn, proliferate.
pub fn dreamtide_whale() -> CardDefinition {
    CardDefinition {
        name: "Dreamtide Whale",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Whale], ..Default::default() },
        power: 7,
        toughness: 5,
        keywords: vec![Keyword::Vanishing(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::SpellsCastThisTurnEquals { who: PlayerRef::Triggerer, count: Value::Const(2) },
            ),
            effect: Effect::Proliferate,
        }],
        ..Default::default()
    }
}

/// Etherium Pteramander — {B} 1/1 Artifact Salamander Drake. Flying; can block
/// only fliers. {6}{B}: Adapt 4, {1} cheaper per other artifact you control.
pub fn etherium_pteramander() -> CardDefinition {
    CardDefinition {
        name: "Etherium Pteramander",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Salamander, CreatureType::Drake],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), b()]),
            cost_reduction_per: Some(R::Artifact.and(R::OtherThanSource)),
            effect: adapt(4),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Not Forgotten — {1}{W} Sorcery. Put target card from a graveyard on its
/// owner's choice of the top or bottom of their library. Create a 1/1 white
/// flying Spirit.
pub fn not_forgotten() -> CardDefinition {
    CardDefinition {
        name: "Not Forgotten",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::InGraveyard),
                to: crate::effect::ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: crate::effect::LibraryPosition::OwnerChoice,
                },
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: spirit_flyer() },
        ]),
        ..Default::default()
    }
}

// ── Batch 4 (MH3 Flare cycle) ────────────────────────────────────────────────

/// The Flare alt-cost — "you may sacrifice a nontoken [color] creature rather
/// than pay this spell's mana cost."
fn flare_cost(color: Color) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost {
        mana_cost: cost(&[]),
        sacrifice_permanents: Some((R::Creature.and(R::NotToken).and(R::HasColor(color)), 1)),
        ..Default::default()
    }
}

/// Flare of Cultivation — {1}{G}{G} Sorcery (or sac a nontoken green creature).
/// Search for up to two basics: one to the battlefield tapped, one to hand.
pub fn flare_of_cultivation() -> CardDefinition {
    CardDefinition {
        name: "Flare of Cultivation",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: crate::effect::ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::Search { who: PlayerRef::You, filter: R::IsBasicLand, to: ZoneDest::Hand(PlayerRef::You) },
        ]),
        alternative_cost: Some(flare_cost(Color::Green)),
        ..Default::default()
    }
}

/// Flare of Fortitude — {2}{W}{W} Instant (or sac a nontoken white creature).
/// Until end of turn, your life total can't change and permanents you control
/// gain hexproof and indestructible.
pub fn flare_of_fortitude() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::ControlledByYou);
    CardDefinition {
        name: "Flare of Fortitude",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LifeLockThisTurn { who: Selector::Player(PlayerRef::You) },
            Effect::GrantKeyword { what: yours(), keyword: Keyword::Hexproof, duration: Duration::EndOfTurn },
            Effect::GrantKeyword {
                what: yours(),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        alternative_cost: Some(flare_cost(Color::White)),
        ..Default::default()
    }
}
