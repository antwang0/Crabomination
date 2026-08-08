//! Odyssey (ODY) gap-closing wave 6: the Shrine and Lhurgoyf cycles, the
//! artifact shell and the last utility. Tests in `classic_sets/ody`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{draw, target_any, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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

fn artifact_creature(name: &'static str, c: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn threshold() -> Predicate {
    Predicate::ThresholdActive { who: PlayerRef::You }
}

/// The Shrine payoff count: copies of the just-cast spell's name in every
/// graveyard.
fn shrine_count() -> Value {
    Value::CardsNamedLikeTriggerSpellInAllGraveyards
}

/// The shared Shrine shell: "Whenever a player casts a spell, …X…".
fn shrine(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect,
        }],
        ..enchantment(name, c)
    }
}

// ── The Shrine cycle ────────────────────────────────────────────────────────

pub fn aven_shrine() -> CardDefinition {
    shrine(
        "Aven Shrine",
        cost(&[generic(1), w(), w()]),
        Effect::GainLife {
            who: Selector::Player(PlayerRef::TriggerEventPlayer),
            amount: shrine_count(),
        },
    )
}

pub fn cabal_shrine() -> CardDefinition {
    shrine(
        "Cabal Shrine",
        cost(&[generic(1), b(), b()]),
        Effect::Discard {
            who: Selector::Player(PlayerRef::TriggerEventPlayer),
            amount: shrine_count(),
            random: false,
        },
    )
}

pub fn dwarven_shrine() -> CardDefinition {
    shrine(
        "Dwarven Shrine",
        cost(&[generic(1), r(), r()]),
        Effect::DealDamage {
            to: Selector::Player(PlayerRef::TriggerEventPlayer),
            amount: Value::Times(Box::new(Value::Const(2)), Box::new(shrine_count())),
        },
    )
}

pub fn nantuko_shrine() -> CardDefinition {
    shrine(
        "Nantuko Shrine",
        cost(&[generic(1), g(), g()]),
        Effect::CreateToken {
            who: PlayerRef::TriggerEventPlayer,
            count: shrine_count(),
            definition: Box::new(TokenDefinition {
                name: "Squirrel".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Squirrel],
                    ..Default::default()
                },
                ..Default::default()
            }),
        },
    )
}

// ── The Lhurgoyf cycle ──────────────────────────────────────────────────────

fn vore(name: &'static str, c: ManaCost, ty: CardType, keywords: Vec<Keyword>) -> CardDefinition {
    CardDefinition {
        keywords,
        dynamic_pt: Some(DynamicPt::CardTypeInAllGraveyards(ty)),
        ..creature(name, c, vec![CreatureType::Lhurgoyf], 0, 0)
    }
}

pub fn cantivore() -> CardDefinition {
    vore(
        "Cantivore",
        cost(&[generic(1), w(), w()]),
        CardType::Enchantment,
        vec![Keyword::Vigilance],
    )
}

pub fn cognivore() -> CardDefinition {
    vore(
        "Cognivore",
        cost(&[generic(6), u(), u()]),
        CardType::Instant,
        vec![Keyword::Flying],
    )
}

pub fn magnivore() -> CardDefinition {
    vore("Magnivore", cost(&[generic(2), r(), r()]), CardType::Sorcery, vec![Keyword::Haste])
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Millikin — {2} 0/1 that mills itself into mana.
pub fn millikin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::ONE },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
            ]),
            ..Default::default()
        }],
        ..artifact_creature("Millikin", cost(&[generic(2)]), 0, 1)
    }
}

/// Limestone Golem — {6} 3/4 that cashes out for a card.
pub fn limestone_golem() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..artifact_creature("Limestone Golem", cost(&[generic(6)]), 3, 4)
    }
}

/// Junk Golem — {4} 0/0 that eats a counter each upkeep and a card to refill.
pub fn junk_golem() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::SourceHasCountersAtLeast {
                    counter: CounterType::PlusOnePlusOne,
                    n: 1,
                },
                then: Box::new(Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::SacrificeSource),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..artifact_creature("Junk Golem", cost(&[generic(4)]), 0, 0)
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Mystic Crusader — {1}{W}{W} 2/1 that dodges removal and flies past
/// Threshold.
pub fn mystic_crusader() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black), Keyword::Protection(Color::Red)],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Threshold — this creature gets +1/+1 and has flying.",
            effect: crate::card::StaticEffect::PumpSelfIf {
                condition: threshold(),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..creature(
            "Mystic Crusader",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Nomad, CreatureType::Mystic],
            2,
            1,
        )
    }
}

/// Iridescent Angel — {5}{W}{U} 4/4 flier nothing coloured can touch.
pub fn iridescent_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Protection(Color::White),
            Keyword::Protection(Color::Blue),
            Keyword::Protection(Color::Black),
            Keyword::Protection(Color::Red),
            Keyword::Protection(Color::Green),
        ],
        ..creature(
            "Iridescent Angel",
            cost(&[generic(5), w(), u()]),
            vec![CreatureType::Angel],
            4,
            4,
        )
    }
}

/// Confessor — {W} 1/1 that skims a life off every discard.
pub fn confessor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::AnyPlayer),
            effect: Effect::MayDo {
                description: "Gain 1 life?".into(),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..creature("Confessor", cost(&[w()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 1)
    }
}

/// Chainflinger — {3}{R} 2/2 pinger that upgrades past Threshold.
pub fn chainflinger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                tap_cost: true,
                effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                tap_cost: true,
                condition: Some(threshold()),
                effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..creature("Chainflinger", cost(&[generic(3), r()]), vec![CreatureType::Beast], 2, 2)
    }
}

/// Ashen Firebeast — {6}{R}{R} 6/6 repeatable ground sweep.
pub fn ashen_firebeast() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Ashen Firebeast",
            cost(&[generic(6), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Beast],
            6,
            6,
        )
    }
}

/// Atogatog — {W}{U}{B}{R}{G} 5/5 that eats its own kind.
pub fn atogatog() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature.and(R::HasCreatureType(CreatureType::Atog)), 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::SacrificedPower,
                toughness: Value::SacrificedPower,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Atogatog", cost(&[w(), u(), b(), r(), g()]), vec![CreatureType::Atog], 5, 5)
    }
}

/// Lithatog — {1}{R}{G} 1/2 Atog that eats artifacts and lands.
pub fn lithatog() -> CardDefinition {
    let eat = |filter: R| ActivatedAbility {
        sac_other_filter: Some((filter, 1)),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![eat(R::Artifact), eat(R::Land)],
        ..creature("Lithatog", cost(&[generic(1), r(), g()]), vec![CreatureType::Atog], 1, 2)
    }
}

/// Diligent Farmhand — {G} 1/1 that cashes itself in for a basic.
pub fn diligent_farmhand() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature(
            "Diligent Farmhand",
            cost(&[g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            1,
        )
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Kirtar's Wrath — {4}{W}{W} a wrath that leaves Spirits past Threshold.
pub fn kirtars_wrath() -> CardDefinition {
    let wipe = Effect::DestroyNoRegen { what: Selector::EachPermanent(R::Creature) };
    sorcery(
        "Kirtar's Wrath",
        cost(&[generic(4), w(), w()]),
        Effect::If {
            cond: threshold(),
            then: Box::new(Effect::Seq(vec![
                wipe.clone(),
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: Box::new(TokenDefinition {
                        name: "Spirit".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White],
                        keywords: vec![Keyword::Flying],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Spirit],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
            ])),
            else_: Box::new(wipe),
        },
    )
}

/// Decimate — {2}{R}{G} one of each, all at once.
pub fn decimate() -> CardDefinition {
    sorcery(
        "Decimate",
        cost(&[generic(2), r(), g()]),
        Effect::Seq(vec![
            Effect::Destroy { what: Selector::TargetFiltered { slot: 0, filter: R::Artifact } },
            Effect::Destroy { what: Selector::TargetFiltered { slot: 1, filter: R::Creature } },
            Effect::Destroy { what: Selector::TargetFiltered { slot: 2, filter: R::Enchantment } },
            Effect::Destroy { what: Selector::TargetFiltered { slot: 3, filter: R::Land } },
        ]),
    )
}

/// Execute — {2}{B} kills a white creature and replaces itself.
pub fn execute() -> CardDefinition {
    instant(
        "Execute",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::HasColor(Color::White))),
            },
            draw(1),
        ]),
    )
}

/// Fervent Denial — {3}{U}{U} a counterspell with flashback.
pub fn fervent_denial() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), u(), u()]))],
        ..instant(
            "Fervent Denial",
            cost(&[generic(3), u(), u()]),
            Effect::CounterSpell { what: Selector::Target(0) },
        )
    }
}

/// Morbid Hunger — {4}{B}{B} drain three, twice.
pub fn morbid_hunger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(7), b(), b()]))],
        ..sorcery(
            "Morbid Hunger",
            cost(&[generic(4), b(), b()]),
            Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ]),
        )
    }
}

/// Ancestral Tribute — {5}{W}{W} two life per graveyard card, twice.
pub fn ancestral_tribute() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(9), w(), w(), w()]))],
        ..sorcery(
            "Ancestral Tribute",
            cost(&[generic(5), w(), w()]),
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: R::Any,
                    }),
                ),
            },
        )
    }
}

/// Acceptable Losses — {3}{R} five damage for a random card.
pub fn acceptable_losses() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::DiscardRandom { count: 1 }],
        ..sorcery(
            "Acceptable Losses",
            cost(&[generic(3), r()]),
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(5),
            },
        )
    }
}

/// Animal Boneyard — {2}{W} makes the enchanted land eat creatures for life.
pub fn animal_boneyard() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::SacrificedToughness,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..enchantment("Animal Boneyard", cost(&[generic(2), w()]))
    }
}
