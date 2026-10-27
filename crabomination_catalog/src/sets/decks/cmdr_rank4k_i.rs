//! Commander: three-card gaps of EDHREC's commanders ranked 3501-4500 that
//! share cards (the 2026-10-14 scan), ninth file. Tests in
//! `tests/recent_b/cmdr_rank4k_i.rs`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r, u, w, x};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: n,
        definition: Arc::new(def),
    }
}

fn shrine() -> R {
    R::HasEnchantmentSubtype(EnchantmentSubtype::Shrine)
}

fn shrines() -> Value {
    Value::count(yours(shrine()))
}

fn your_end_step(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
        effect,
    }
}

fn pay_one(description: &str, body: Effect) -> Effect {
    Effect::MayPay {
        description: description.into(),
        mana_cost: cost(&[generic(1)]),
        body: Box::new(body),
        else_: None,
    }
}

/// A legendary enchantment creature — Shrine.
fn go_shintai(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Shrine],
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// A legendary enchantment — Shrine.
fn sanctum(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Shrine],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn dwarf_token() -> TokenDefinition {
    TokenDefinition {
        name: "Dwarf".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn tap_and_stun(what: Selector) -> Effect {
    Effect::Seq(vec![
        Effect::Tap { what: what.clone() },
        Effect::AddCounter {
            what,
            kind: CounterType::Stun,
            amount: Value::ONE,
        },
    ])
}

// ── The Go-Shintai cycle and their Shrines ───────────────────────────────────

/// Go-Shintai of Shared Purpose — {3}{W} legendary 1/3 vigilance Shrine
/// enchantment creature. At the beginning of your end step, you may pay {1};
/// if you do, a 1/1 colorless Spirit per Shrine you control.
pub fn go_shintai_of_shared_purpose() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![your_end_step(pay_one(
            "Pay {1} for a Spirit per Shrine?",
            make(shrines(), spirit),
        ))],
        ..go_shintai("Go-Shintai of Shared Purpose", cost(&[generic(3), w()]), 1, 3)
    }
}

/// Go-Shintai of Ancient Wars — {2}{R} legendary 2/2 first strike Shrine. At
/// the beginning of your end step, you may pay {1}; when you do, X damage to
/// target player or planeswalker, X = Shrines you control.
pub fn go_shintai_of_ancient_wars() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![your_end_step(pay_one(
            "Pay {1} to burn a player or planeswalker per Shrine?",
            Effect::Reflexive {
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: shrines(),
                }),
            },
        ))],
        ..go_shintai("Go-Shintai of Ancient Wars", cost(&[generic(2), r()]), 2, 2)
    }
}

/// Go-Shintai of Boundless Vigor — {1}{G} legendary 1/1 trample Shrine. At the
/// beginning of your end step, you may pay {1}; when you do, a +1/+1 counter
/// on target Shrine per Shrine you control.
pub fn go_shintai_of_boundless_vigor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![your_end_step(pay_one(
            "Pay {1} to grow a Shrine?",
            Effect::Reflexive {
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(shrine()),
                    kind: CounterType::PlusOnePlusOne,
                    amount: shrines(),
                }),
            },
        ))],
        ..go_shintai("Go-Shintai of Boundless Vigor", cost(&[generic(1), g()]), 1, 1)
    }
}

/// Shrine Steward — {5} 3/2 Construct artifact creature. ETB: you may search
/// your library for an Aura or Shrine card and put it into your hand.
pub fn shrine_steward() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for an Aura or Shrine card?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).or(shrine()),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature(
            "Shrine Steward",
            cost(&[generic(5)]),
            vec![CreatureType::Construct],
            3,
            2,
        )
    }
}

/// Sanctum of Tranquil Light — {W} legendary Shrine. {5}{W}: tap target
/// creature; {1} less for each Shrine you control.
pub fn sanctum_of_tranquil_light() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), w()]),
            cost_reduction_per: Some(shrine().and(R::ControlledByYou)),
            effect: Effect::Tap {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..sanctum("Sanctum of Tranquil Light", cost(&[w()]))
    }
}

/// Sanctum of Shattered Heights — {2}{R} legendary Shrine. {1}, discard a land
/// or Shrine card: X damage to target creature or planeswalker, X = Shrines
/// you control.
pub fn sanctum_of_shattered_heights() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            discard_cost: Some((R::Land.or(shrine()), 1)),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::Planeswalker)),
                amount: shrines(),
            },
            ..Default::default()
        }],
        ..sanctum("Sanctum of Shattered Heights", cost(&[generic(2), r()]))
    }
}

/// Sanctum of Fruitful Harvest — {2}{G} legendary Shrine. At the beginning of
/// your first main phase, add X mana of any one color, X = Shrines you
/// control.
pub fn sanctum_of_fruitful_harvest() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::YourControl,
            ),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(shrines()),
            },
        }],
        ..sanctum("Sanctum of Fruitful Harvest", cost(&[generic(2), g()]))
    }
}

// ── Kíli the Resourceful / Dáin / Augusta, Order Returned ────────────────────

/// Fíli the Pathfinder — {3}{W} legendary 2/2 Dwarf Scout, storied. With an
/// enduring story, creatures you control get +1/+1. Whenever Fíli or another
/// nontoken Dwarf you control enters, a 2/2 red Dwarf.
pub fn fili_the_pathfinder() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Storied (If you control three or more artifacts, legendaries, and/or Sagas, you have an enduring story for the rest of the game.)",
                effect: StaticEffect::Storied,
            },
            StaticAbility {
                description: "As long as you have an enduring story, creatures you control get +1/+1.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::HasEnduringStory { who: PlayerRef::You },
                    inner: Box::new(StaticEffect::PumpPT {
                        applies_to: yours(R::Creature),
                        power: 1,
                        toughness: 1,
                    }),
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::Any(vec![
                    Predicate::TriggerSourceIsSelf,
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Dwarf)
                            .and(R::Not(Box::new(R::IsToken))),
                    },
                ])),
            effect: make(Value::ONE, dwarf_token()),
        }],
        ..legend(
            "Fíli the Pathfinder",
            cost(&[generic(3), w()]),
            vec![CreatureType::Dwarf, CreatureType::Scout],
            2,
            2,
        )
    }
}

/// Iron Hills Blacksmith — {1}{W} 1/1 double strike Dwarf Artificer. ETB: an
/// Axe Equipment token ("Equipped creature gets +1/+0", equip {2}).
pub fn iron_hills_blacksmith() -> CardDefinition {
    let axe = TokenDefinition {
        name: "Axe".into(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            ..Default::default()
        }),
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![etb(make(Value::ONE, axe))],
        ..creature(
            "Iron Hills Blacksmith",
            cost(&[generic(1), w()]),
            vec![CreatureType::Dwarf, CreatureType::Artificer],
            1,
            1,
        )
    }
}

/// An Unexpected Party // At the Door — {2}{W}{W} enchantment. As it enters,
/// choose a creature type; creatures you control of that type get +2/+2.
/// Adventure ({X}{2}{W} sorcery): X 2/2 red Dwarves.
pub fn an_unexpected_party() -> CardDefinition {
    CardDefinition {
        name: "An Unexpected Party",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        static_abilities: vec![StaticAbility {
            description: "Creatures you control of the chosen type get +2/+2.",
            effect: StaticEffect::PumpPT {
                applies_to: yours(R::Creature.and(R::IsSourceChosenCreatureType)),
                power: 2,
                toughness: 2,
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "At the Door",
            cost: cost(&[x(), generic(2), w()]),
            card_types: vec![CardType::Sorcery],
            effect: make(Value::XFromCost, dwarf_token()),
        })),
        ..Default::default()
    }
}

/// Dwarven Shortsword — {3}{W} Equipment. ETB: a 2/2 red Dwarf, then attach it
/// to that token. Equipped creature gets +1/+2. Equip {2}.
pub fn dwarven_shortsword() -> CardDefinition {
    CardDefinition {
        name: "Dwarven Shortsword",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            make(Value::ONE, dwarf_token()),
            Effect::Attach {
                what: Selector::This,
                to: Selector::LastCreatedTokens,
            },
        ]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 2,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Ori, Plate Stacker — {5}{W}{W} legendary 3/3 Dwarf Bard. ETB: destroy all
/// artifacts and enchantments your opponents control; gain 1 life per
/// permanent destroyed this way.
pub fn ori_plate_stacker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::EachPermanent(
                    (R::Artifact.or(R::Enchantment)).and(R::ControlledByOpponent),
                ),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::PermanentsDestroyedThisResolution,
            },
        ]))],
        ..legend(
            "Ori, Plate Stacker",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Dwarf, CreatureType::Bard],
            3,
            3,
        )
    }
}

/// Eagle of the Great Shelf — {4}{W} 2/5 flying Bird Soldier. Whenever it
/// attacks, +1/+1 until end of turn for each other creature you control.
pub fn eagle_of_the_great_shelf() -> CardDefinition {
    let others = || {
        Value::PermanentCountControlledByMatching(PlayerRef::You, R::Creature.and(R::OtherThanSource))
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: others(),
            toughness: others(),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Eagle of the Great Shelf",
            cost(&[generic(4), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            5,
        )
    }
}

/// Lake-town Toymaker — {3}{W} 3/4 Human Artificer. At the beginning of combat
/// on your turn, if you've drawn two or more cards this turn, another target
/// creature you control gets +3/+0 and first strike until end of turn.
pub fn lake_town_toymaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            )
            .with_filter(Predicate::PlayerDrewAtLeastThisTurn {
                who: PlayerRef::You,
                n: 2,
            }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    power: Value::Const(3),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..creature(
            "Lake-town Toymaker",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            4,
        )
    }
}

// ── Monstrosity of the Lake / K-9, Mark I ────────────────────────────────────

/// Hapatra, the Desert Frost — {3}{U} legendary 4/3 Human Wizard. ETB: for
/// each opponent, tap up to one target creature that player controls and put
/// a stun counter on it. {2}{U}: untap target creature.
pub fn hapatra_the_desert_frost() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 15,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(tap_and_stun(Selector::Target(0))),
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Untap {
                what: target_filtered(R::Creature),
                up_to: None,
            },
            ..Default::default()
        }],
        ..legend(
            "Hapatra, the Desert Frost",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            4,
            3,
        )
    }
}

/// Referee Squad — {2}{U} 2/2 convoke, vigilance Homunculus. ETB: tap target
/// creature an opponent controls and put a stun counter on it.
pub fn referee_squad() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke, Keyword::Vigilance],
        triggered_abilities: vec![etb(tap_and_stun(target_filtered(
            R::Creature.and(R::ControlledByOpponent),
        )))],
        ..creature(
            "Referee Squad",
            cost(&[generic(2), u()]),
            vec![CreatureType::Homunculus],
            2,
            2,
        )
    }
}

/// Seasoned Cryomancer — {1}{U}{U} 2/2 Human Wizard. ETB: draw two, then
/// discard two; when you discard one or more nonland cards this way, tap up to
/// that many target creatures and stun each. {3}{U}{U}, exile it from your
/// graveyard: draw two.
pub fn seasoned_cryomancer() -> CardDefinition {
    let nonland_discarded = || Value::count(Selector::DiscardedThisResolution { filter: R::Nonland });
    let freeze = |n: u8| Effect::Reflexive {
        body: Box::new(Effect::ApplyToTargets {
            max_targets: n,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(tap_and_stun(Selector::Target(0))),
        }),
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(2),
                random: false,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(nonland_discarded(), Value::Const(2)),
                then: Box::new(freeze(2)),
                else_: Box::new(Effect::If {
                    cond: Predicate::ValueAtLeast(nonland_discarded(), Value::ONE),
                    then: Box::new(freeze(1)),
                    else_: Box::new(Effect::Noop),
                }),
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u(), u()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Seasoned Cryomancer",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// The Blue Spirit — {3}{U} legendary 2/4 Human Rogue Ally. You may cast the
/// first creature spell you cast each turn as though it had flash. Whenever a
/// nontoken creature you control enters during combat, draw a card.
pub fn the_blue_spirit() -> CardDefinition {
    let in_combat = Predicate::Any(
        [
            TurnStep::BeginCombat,
            TurnStep::DeclareAttackers,
            TurnStep::DeclareBlockers,
            TurnStep::CombatDamage,
            TurnStep::EndCombat,
        ]
        .into_iter()
        .map(Predicate::CurrentStepIs)
        .collect(),
    );
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may cast the first creature spell you cast each turn as though it had flash.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::SpellsOfTypeCastThisTurnAtMost {
                    who: PlayerRef::You,
                    card_type: CardType::Creature,
                    n: 0,
                },
                inner: Box::new(StaticEffect::ControllerSpellsHaveFlash { filter: R::Creature }),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::Not(Box::new(R::IsToken))),
                    },
                    in_combat,
                ])),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..legend(
            "The Blue Spirit",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Ally],
            2,
            4,
        )
    }
}

/// Arwen's Gift — {3}{U} sorcery; {1} less with two or more legendary
/// creatures. Scry 2, then draw two.
pub fn arwens_gift() -> CardDefinition {
    CardDefinition {
        name: "Arwen's Gift",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        self_cost_reduction_if: Some((
            Predicate::ValueAtLeast(
                Value::PermanentCountControlledByMatching(
                    PlayerRef::You,
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                ),
                Value::Const(2),
            ),
            1,
        )),
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}
