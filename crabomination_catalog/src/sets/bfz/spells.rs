//! BFZ instants, sorceries, enchantments and artifacts — Awaken, Converge and
//! the Eldrazi removal suite.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, Keyword,
    SelectionRequirement as R, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::{awaken, each_your_creature, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color};

fn spell(name: &'static str, c: crate::mana::ManaCost, ty: CardType, e: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![ty], effect: e, ..Default::default() }
}

fn kor_ally_token() -> TokenDefinition {
    TokenDefinition {
        name: "Kor Ally".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Ally],
            ..Default::default()
        },
        ..Default::default()
    }
}

// ── Plain removal / utility ─────────────────────────────────────────────────

/// Demon's Grasp — {4}{B} Sorcery. Target creature gets -5/-5 until end of turn.
pub fn demons_grasp() -> CardDefinition {
    spell(
        "Demon's Grasp",
        cost(&[generic(4), b()]),
        CardType::Sorcery,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-5),
            toughness: Value::Const(-5),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Reclaiming Vines — {2}{G}{G} Sorcery. Destroy target artifact, enchantment
/// or land.
pub fn reclaiming_vines() -> CardDefinition {
    spell(
        "Reclaiming Vines",
        cost(&[generic(2), g(), g()]),
        CardType::Sorcery,
        Effect::Destroy {
            what: target_filtered(R::Artifact.or(R::Enchantment).or(R::Land)),
        },
    )
}

/// Volcanic Upheaval — {3}{R} Instant. Destroy target land.
pub fn volcanic_upheaval() -> CardDefinition {
    spell(
        "Volcanic Upheaval",
        cost(&[generic(3), r()]),
        CardType::Instant,
        Effect::Destroy { what: target_filtered(R::Land) },
    )
}

/// Outnumber — {R} Instant. Damage to target creature equal to the number of
/// creatures you control.
pub fn outnumber() -> CardDefinition {
    spell(
        "Outnumber",
        cost(&[r()]),
        CardType::Instant,
        Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::count(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
        },
    )
}

/// Stonefury — {3}{R}{R} Instant. Damage to target creature equal to the number
/// of lands you control.
pub fn stonefury() -> CardDefinition {
    spell(
        "Stonefury",
        cost(&[generic(3), r(), r()]),
        CardType::Instant,
        Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByYou))),
        },
    )
}

/// Natural Connection — {2}{G} Instant. Search for a basic land onto the
/// battlefield tapped.
pub fn natural_connection() -> CardDefinition {
    spell(
        "Natural Connection",
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
    )
}

/// Nissa's Renewal — {5}{G} Sorcery. Up to three basics tapped, gain 7 life.
pub fn nissas_renewal() -> CardDefinition {
    spell(
        "Nissa's Renewal",
        cost(&[generic(5), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::Const(3),
            },
            crate::effect::shortcut::gain_life(7),
        ]),
    )
}

/// Seek the Wilds — {1}{G} Sorcery. Look at four, take a creature or land card.
pub fn seek_the_wilds() -> CardDefinition {
    spell(
        "Seek the Wilds",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(R::Creature.or(R::Land)),
            rest_to_graveyard: false,
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: false,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
            rest_to_exile: false,
        },
    )
}

/// Tandem Tactics — {1}{W} Instant. Up to two creatures get +1/+2; gain 2 life.
pub fn tandem_tactics() -> CardDefinition {
    spell(
        "Tandem Tactics",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(1),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                }),
            },
            crate::effect::shortcut::gain_life(2),
        ]),
    )
}

/// Lithomancer's Focus — {W} Instant. Target creature gets +2/+2 until end of
/// turn. (The colorless-source damage prevention needs a filtered prevention
/// shield the engine doesn't have; tracked in TODO.md.)
pub fn lithomancers_focus() -> CardDefinition {
    spell(
        "Lithomancer's Focus",
        cost(&[w()]),
        CardType::Instant,
        crate::effect::shortcut::pump_target(2, 2),
    )
}

/// Swarm Surge — {2}{B} Sorcery. Devoid. Your creatures get +2/+0; colorless
/// ones also gain first strike.
pub fn swarm_surge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        ..spell(
            "Swarm Surge",
            cost(&[generic(2), b()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: each_your_creature(),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::Colorless).and(R::ControlledByYou),
                    ),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
        )
    }
}

/// Swell of Growth — {1}{G} Instant. +2/+2, then you may put a land from hand
/// onto the battlefield.
pub fn swell_of_growth() -> CardDefinition {
    spell(
        "Swell of Growth",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            crate::effect::shortcut::pump_target(2, 2),
            Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Land,
                count: Value::Const(1),
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
        ]),
    )
}

/// Grip of Desolation — {4}{B}{B} Instant. Devoid. Exile target creature and
/// target land.
pub fn grip_of_desolation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        ..spell(
            "Grip of Desolation",
            cost(&[generic(4), b(), b()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Creature) },
                Effect::Exile {
                    what: Selector::TargetFiltered { slot: 1, filter: R::Land },
                },
            ]),
        )
    }
}

/// Transgress the Mind — {1}{B} Sorcery. Devoid. Target player reveals their
/// hand; exile a card with mana value 3 or greater from it.
pub fn transgress_the_mind() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        ..spell(
            "Transgress the Mind",
            cost(&[generic(1), b()]),
            CardType::Sorcery,
            Effect::ExileChosenFromHand {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::Const(1),
                filter: R::ManaValueAtLeast(3),
                face_down: false,
                link_to_source: false,
            },
        )
    }
}

/// Horribly Awry — {1}{U} Instant. Devoid. Counter a creature spell with mana
/// value 4 or less; exile it instead of binning it.
pub fn horribly_awry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        ..spell(
            "Horribly Awry",
            cost(&[generic(1), u()]),
            CardType::Instant,
            Effect::CounterSpellToZone {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::IsSpellOnStack.and(R::Creature).and(R::ManaValueAtMost(4)),
                },
                zone: crate::effect::CounteredSpellZone::Exile,
            },
        )
    }
}

/// Spell Shrivel — {2}{U} Instant. Devoid. Counter unless its controller pays
/// {4}; exile it instead of binning it.
pub fn spell_shrivel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        ..spell(
            "Spell Shrivel",
            cost(&[generic(2), u()]),
            CardType::Instant,
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(4)]),
                exile: true,
                extra_generic: None,
            },
        )
    }
}

/// Turn Against — {4}{R} Instant. Devoid. Gain control of target creature until
/// end of turn; untap it and it gains haste.
pub fn turn_against() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        ..spell(
            "Turn Against",
            cost(&[generic(4), r()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::GainControl {
                    what: target_filtered(R::Creature),
                    to: None,
                    duration: Duration::EndOfTurn,
                },
                Effect::Untap { what: Selector::Target(0), up_to: None },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        )
    }
}

/// Grave Birthing — {2}{B} Instant. Devoid. An opponent exiles a graveyard
/// card; create an Eldrazi Scion; draw a card.
pub fn grave_birthing() -> CardDefinition {
    use crabomination_base::tokens::eldrazi_scion_token;
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        ..spell(
            "Grave Birthing",
            cost(&[generic(2), b()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::ExileChosenFromHandOrGraveyard {
                    who: PlayerRef::Target(0),
                    filter: R::Any,
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: eldrazi_scion_token(),
                },
                crate::effect::shortcut::draw(1),
            ]),
        )
    }
}

/// Ugin's Insight — {3}{U}{U} Sorcery. Scry X (the greatest mana value among
/// your permanents), then draw three cards.
pub fn ugins_insight() -> CardDefinition {
    spell(
        "Ugin's Insight",
        cost(&[generic(3), u(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::GreatestManaValueAmongPermanents(PlayerRef::You),
            },
            crate::effect::shortcut::draw(3),
        ]),
    )
}

/// Roil's Retribution — {3}{W}{W} Instant. 5 damage divided among any number of
/// target attacking or blocking creatures.
pub fn roils_retribution() -> CardDefinition {
    spell(
        "Roil's Retribution",
        cost(&[generic(3), w(), w()]),
        CardType::Instant,
        Effect::DealDamageDivided {
            total: Value::Const(5),
            max_targets: 5,
            filter: R::Creature.and(R::IsAttacking.or(R::IsBlocking)),
            retaliate_to_source: false,
        },
    )
}

/// Serpentine Spike — {5}{R}{R} Sorcery. Devoid. 2/3/4 damage to three
/// different creatures; any that would die are exiled instead.
pub fn serpentine_spike() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        ..spell(
            "Serpentine Spike",
            cost(&[generic(5), r(), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::ExileIfWouldDieThisTurn { what: Selector::AllTargets },
                Effect::DealDamage {
                    to: target_filtered(R::Creature),
                    amount: Value::Const(2),
                },
                Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                    amount: Value::Const(3),
                },
                Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 2, filter: R::Creature },
                    amount: Value::Const(4),
                },
            ]),
        )
    }
}

/// Unnatural Aggression — {2}{G} Instant. Devoid. Your creature fights an
/// opponent's; the opponent's is exiled instead of dying.
pub fn unnatural_aggression() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        ..spell(
            "Unnatural Aggression",
            cost(&[generic(2), g()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::ExileIfWouldDieThisTurn {
                    what: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    },
                },
                Effect::Fight {
                    attacker: target_filtered(R::Creature.and(R::ControlledByYou)),
                    defender: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    },
                },
            ]),
        )
    }
}

// ── Awaken ──────────────────────────────────────────────────────────────────

/// Clutch of Currents — {U} Sorcery. Bounce a creature. Awaken 3—{4}{U}.
pub fn clutch_of_currents() -> CardDefinition {
    let body = Effect::Move {
        what: target_filtered(R::Creature),
        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
    };
    CardDefinition {
        alternative_cost: Some(awaken(3, cost(&[generic(4), u()]), 1, body.clone())),
        ..spell("Clutch of Currents", cost(&[u()]), CardType::Sorcery, body)
    }
}

/// Rush of Ice — {U} Sorcery. Tap a creature; it doesn't untap next untap step.
/// Awaken 3—{4}{U}.
pub fn rush_of_ice() -> CardDefinition {
    let body = Effect::Seq(vec![
        Effect::Tap { what: target_filtered(R::Creature) },
        Effect::SkipNextUntap { what: Selector::Target(0) },
    ]);
    CardDefinition {
        alternative_cost: Some(awaken(3, cost(&[generic(4), u()]), 1, body.clone())),
        ..spell("Rush of Ice", cost(&[u()]), CardType::Sorcery, body)
    }
}

/// Boiling Earth — {1}{R} Sorcery. 1 damage to each creature your opponents
/// control. Awaken 4—{6}{R}.
pub fn boiling_earth() -> CardDefinition {
    let body = Effect::DealDamage {
        to: crate::effect::shortcut::each_opponent_creature(),
        amount: Value::Const(1),
    };
    CardDefinition {
        alternative_cost: Some(awaken(4, cost(&[generic(6), r()]), 0, body.clone())),
        ..spell("Boiling Earth", cost(&[generic(1), r()]), CardType::Sorcery, body)
    }
}

/// Earthen Arms — {1}{G} Sorcery. Two +1/+1 counters on target permanent.
/// Awaken 4—{6}{G}.
pub fn earthen_arms() -> CardDefinition {
    let body = Effect::AddCounter {
        what: target_filtered(R::Permanent),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(2),
    };
    CardDefinition {
        alternative_cost: Some(awaken(4, cost(&[generic(6), g()]), 1, body.clone())),
        ..spell("Earthen Arms", cost(&[generic(1), g()]), CardType::Sorcery, body)
    }
}

/// Rising Miasma — {3}{B} Sorcery. All creatures get -2/-2. Awaken 3—{5}{B}{B}.
pub fn rising_miasma() -> CardDefinition {
    let body = Effect::PumpPT {
        what: crate::effect::shortcut::each_creature(),
        power: Value::Const(-2),
        toughness: Value::Const(-2),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        alternative_cost: Some(awaken(3, cost(&[generic(5), b(), b()]), 0, body.clone())),
        ..spell("Rising Miasma", cost(&[generic(3), b()]), CardType::Sorcery, body)
    }
}

/// Ruinous Path — {1}{B}{B} Sorcery. Destroy target creature or planeswalker.
/// Awaken 4—{5}{B}{B}.
pub fn ruinous_path() -> CardDefinition {
    let body = Effect::Destroy {
        what: target_filtered(R::Creature.or(R::Planeswalker)),
    };
    CardDefinition {
        alternative_cost: Some(awaken(4, cost(&[generic(5), b(), b()]), 1, body.clone())),
        ..spell("Ruinous Path", cost(&[generic(1), b(), b()]), CardType::Sorcery, body)
    }
}

/// Scatter to the Winds — {1}{U}{U} Instant. Counter target spell.
/// Awaken 3—{4}{U}{U}.
pub fn scatter_to_the_winds() -> CardDefinition {
    let body = crate::effect::shortcut::counter_target_spell();
    CardDefinition {
        alternative_cost: Some(awaken(3, cost(&[generic(4), u(), u()]), 1, body.clone())),
        ..spell("Scatter to the Winds", cost(&[generic(1), u(), u()]), CardType::Instant, body)
    }
}

/// Planar Outburst — {3}{W}{W} Sorcery. Destroy all nonland creatures.
/// Awaken 4—{5}{W}{W}{W}.
pub fn planar_outburst() -> CardDefinition {
    let body = Effect::DestroyNoRegen {
        what: Selector::EachPermanent(R::Creature.and(R::Nonland)),
    };
    CardDefinition {
        alternative_cost: Some(awaken(4, cost(&[generic(5), w(), w(), w()]), 0, body.clone())),
        ..spell("Planar Outburst", cost(&[generic(3), w(), w()]), CardType::Sorcery, body)
    }
}

/// Encircling Fissure — {2}{W} Instant. Prevent all combat damage target
/// opponent's creatures would deal this turn. Awaken 2—{4}{W}.
pub fn encircling_fissure() -> CardDefinition {
    let body = Effect::PreventCombatDamageByTargetThisTurn {
        target: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
    };
    CardDefinition {
        alternative_cost: Some(awaken(2, cost(&[generic(4), w()]), 1, body.clone())),
        ..spell("Encircling Fissure", cost(&[generic(2), w()]), CardType::Instant, body)
    }
}

// ── Converge ────────────────────────────────────────────────────────────────

/// Radiant Flames — {2}{R} Sorcery. Converge: X damage to each creature.
pub fn radiant_flames() -> CardDefinition {
    spell(
        "Radiant Flames",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::DealDamage {
            to: crate::effect::shortcut::each_creature(),
            amount: Value::ConvergedValue,
        },
    )
}

/// Unified Front — {3}{W} Sorcery. Converge: a 1/1 Kor Ally per color spent.
pub fn unified_front() -> CardDefinition {
    spell(
        "Unified Front",
        cost(&[generic(3), w()]),
        CardType::Sorcery,
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ConvergedValue,
            definition: kor_ally_token(),
        },
    )
}

/// Brilliant Spectrum — {3}{U} Sorcery. Converge: draw X, then discard two.
pub fn brilliant_spectrum() -> CardDefinition {
    spell(
        "Brilliant Spectrum",
        cost(&[generic(3), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ConvergedValue },
            crate::effect::shortcut::discard(Selector::You, 2, false),
        ]),
    )
}

/// Infuse with the Elements — {3}{G} Instant. Converge: X +1/+1 counters on a
/// creature; it gains trample.
pub fn infuse_with_the_elements() -> CardDefinition {
    spell(
        "Infuse with the Elements",
        cost(&[generic(3), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ConvergedValue,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Exert Influence — {4}{U} Sorcery. Converge: gain control of target creature
/// if its power is at most the number of colors spent.
pub fn exert_influence() -> CardDefinition {
    use crate::card::Predicate;
    spell(
        "Exert Influence",
        cost(&[generic(4), u()]),
        CardType::Sorcery,
        Effect::If {
            cond: Predicate::ValueAtMost(
                Value::PowerOf(Box::new(Selector::Target(0))),
                Value::ConvergedValue,
            ),
            then: Box::new(Effect::GainControl {
                what: target_filtered(R::Creature),
                to: None,
                duration: Duration::Permanent,
            }),
            else_: Box::new(Effect::Noop),
        },
    )
}

// ── Enchantments & artifacts ────────────────────────────────────────────────

/// Dampening Pulse — {3}{U} Enchantment. Creatures your opponents control get
/// -1/-0.
pub fn dampening_pulse() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Dampening Pulse",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures your opponents control get -1/-0.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature,
                power: -1,
                toughness: 0,
                keywords: vec![],
                opponents: true,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Molten Nursery — {2}{R} Enchantment. Devoid. Whenever you cast a colorless
/// spell, it deals 1 damage to any target.
pub fn molten_nursery() -> CardDefinition {
    CardDefinition {
        name: "Molten Nursery",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![crate::effect::shortcut::cast_colorless(Effect::DealDamage {
            to: target_any(),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// From Beyond — {3}{G} Enchantment. Devoid. Upkeep: an Eldrazi Scion;
/// {1}{G}, Sacrifice: tutor an Eldrazi card to hand.
pub fn from_beyond() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crabomination_base::tokens::eldrazi_scion_token;
    CardDefinition {
        name: "From Beyond",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: eldrazi_scion_token(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasCreatureType(CreatureType::Eldrazi),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tightening Coils — {1}{U} Aura. Enchanted creature gets -6/-0 and loses
/// flying.
pub fn tightening_coils() -> CardDefinition {
    CardDefinition {
        name: "Tightening Coils",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: -6,
            remove_keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Aligned Hedron Network — {4} Artifact. ETB: exile all creatures with power 5
/// or greater until it leaves.
pub fn aligned_hedron_network() -> CardDefinition {
    CardDefinition {
        name: "Aligned Hedron Network",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![crate::effect::shortcut::etb(
            Effect::ExileUntilSourceLeaves {
                what: Selector::EachPermanent(R::Creature.and(R::PowerAtLeast(5))),
                return_to: crate::card::ExileReturnZone::Battlefield,
            },
        )],
        ..Default::default()
    }
}

/// Pathway Arrows — {1} Equipment. Equipped creature has "{2}, {T}: 1 damage to
/// target creature; tap it if it's colorless." Equip {2}.
pub fn pathway_arrows() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Pathway Arrows",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: target_filtered(R::Creature),
                        amount: Value::Const(1),
                    },
                    Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::Target(0),
                            filter: R::Colorless,
                        },
                        then: Box::new(Effect::Tap { what: Selector::Target(0) }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Slab Hammer — {2} Equipment. Whenever equipped creature attacks, you may
/// bounce a land you control for +2/+2. Equip {2}.
pub fn slab_hammer() -> CardDefinition {
    CardDefinition {
        name: "Slab Hammer",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::MayDo {
                description: "Return a land you control to hand for +2/+2?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::MoveChosen {
                        from: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                        filter: None,
                        count: Value::Const(1),
                        up_to: false,
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    },
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(2),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    },
                ])),
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Titan's Presence — {3} Instant. Reveal a colorless creature card from hand
/// as an additional cost; exile target creature with power ≤ its power.
pub fn titans_presence() -> CardDefinition {
    use crate::card::{AdditionalCastCost, Predicate};
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::RevealFromHand {
            filter: R::Creature.and(R::Colorless),
        }],
        ..spell(
            "Titan's Presence",
            cost(&[generic(3)]),
            CardType::Instant,
            Effect::If {
                cond: Predicate::ValueAtMost(
                    Value::PowerOf(Box::new(Selector::Target(0))),
                    Value::RevealedForCostPower,
                ),
                then: Box::new(Effect::Exile { what: target_filtered(R::Creature) }),
                else_: Box::new(Effect::Noop),
            },
        )
    }
}

/// Quarantine Field — {X}{X}{W}{W} Enchantment. Enters with X isolation
/// counters; exiles that many opponents' nonland permanents until it leaves.
pub fn quarantine_field() -> CardDefinition {
    CardDefinition {
        name: "Quarantine Field",
        cost: cost(&[x(), x(), w(), w()]),
        card_types: vec![CardType::Enchantment],
        enters_with_counters: Some((CounterType::Charge, Value::XFromCost)),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 0,
            filter: R::Nonland.and(R::Permanent).and(R::ControlledByOpponent),
            effect: Box::new(Effect::ExileUntilSourceLeaves {
                what: Selector::Target(0),
                return_to: crate::card::ExileReturnZone::Battlefield,
            }),
        })],
        ..Default::default()
    }
}
