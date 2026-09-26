//! Commander: the cards the **Mind Seize** precon (C13, Jeleva, Nephalia's
//! Scourge) needed beyond what the catalog had (Curse of Chaos and Terra
//! Ravager are Nature of the Beast's, `cmdr_marath.rs`). Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **True-Name Nemesis** — the chosen player is the engine's most hostile
//!   opponent, not the controller's pick.
//! - **Eye of Doom** — each player's doom counter goes where the engine picks.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    Keyword, LandType, LevelBand, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{draw, etb, on_attack, target_filtered};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, u, x};

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
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

/// An Aura Curse that enchants a player.
fn curse(name: &'static str, mana: ManaCost, trigger: TriggeredAbility) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![trigger],
        ..Default::default()
    }
}

/// "Whenever a player attacks enchanted player with one or more creatures,
/// that attacking player may `body`" — `body` runs as the attacker.
fn attacker_of_cursed_may(description: &str, body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(
            Predicate::AttackedDefenderWithCountAtLeast {
                who: PlayerRef::ActivePlayer,
                defender: PlayerRef::EnchantedPlayer,
                at_least: 1,
                include_planeswalkers: false,
            },
        ),
        effect: Effect::MayDoBy {
            who: PlayerRef::ActivePlayer,
            description: description.into(),
            body: Box::new(body),
        },
    }
}

/// Jeleva, Nephalia's Scourge — flying; on entry each player exiles the top
/// X cards of their library (X: mana spent on her); on attack, you may cast
/// an instant or sorcery exiled with her for free.
pub fn jeleva_nephalias_scourge() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::ExileTopOfLibrary {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::CastSpellManaSpent,
                link_to_source: true,
                face_down: false,
            }),
            on_attack(Effect::MayCastExiledWithSource { filter: instant_or_sorcery() }),
        ],
        ..creature(
            "Jeleva, Nephalia's Scourge",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Vampire, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Arcane Melee — instant and sorcery spells cost {2} less, for everyone.
pub fn arcane_melee() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells cost {2} less to cast.",
            effect: StaticEffect::AllPlayersCostReduction { filter: instant_or_sorcery(), amount: 2 },
        }],
        ..spell("Arcane Melee", cost(&[generic(4), u()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Baleful Force — at the beginning of each upkeep, you draw a card and lose
/// 1 life.
pub fn baleful_force() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                draw(1),
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..creature(
            "Baleful Force",
            cost(&[generic(5), b(), b(), b()]),
            vec![CreatureType::Elemental],
            7,
            7,
        )
    }
}

/// Curse of Shallow Graves — a player attacking the cursed player may make a
/// tapped 2/2 Zombie.
pub fn curse_of_shallow_graves() -> CardDefinition {
    curse(
        "Curse of Shallow Graves",
        cost(&[generic(2), b()]),
        attacker_of_cursed_may(
            "Create a tapped 2/2 black Zombie?",
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: std::sync::Arc::new(TokenDefinition {
                    name: "Zombie".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Zombie],
                        ..Default::default()
                    },
                    tapped: true,
                    ..Default::default()
                }),
            },
        ),
    )
}

/// Echo Mage — level up {1}{U}; LEVEL 2-3 2/4, {U}{U}, {T}: copy an instant
/// or sorcery spell; LEVEL 4+ 2/5, copy it twice.
pub fn echo_mage() -> CardDefinition {
    let copy = |n: i32, condition: Predicate| ActivatedAbility {
        mana_cost: cost(&[u(), u()]),
        tap_cost: true,
        condition: Some(condition),
        effect: Effect::CopySpellMayChooseTargets {
            what: target_filtered(R::IsSpellOnStack.and(instant_or_sorcery())),
            count: Value::Const(n),
        },
        ..Default::default()
    };
    let level = |n: u32| Predicate::SourceHasCountersAtLeast { counter: CounterType::Level, n };
    CardDefinition {
        level_bands: vec![
            LevelBand { min: 2, max: Some(3), power: 2, toughness: 4, keywords: vec![] },
            LevelBand { min: 4, max: None, power: 2, toughness: 5, keywords: vec![] },
        ],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                sorcery_speed: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Level,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            copy(1, Predicate::All(vec![level(2), Predicate::Not(Box::new(level(4)))])),
            copy(2, level(4)),
        ],
        ..creature(
            "Echo Mage",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Eye of Doom — on entry each player puts a doom counter on a nonland
/// permanent; {2}, {T}, sacrifice it: destroy each permanent with a doom
/// counter.
///
/// Approximation: each player's counter goes where the engine picks.
pub fn eye_of_doom() -> CardDefinition {
    CardDefinition {
        name: "Eye of Doom",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::EachPlayerDoes {
            who: PlayerRef::EachPlayer,
            body: Box::new(Effect::ChooseOneAmong {
                what: Selector::EachPermanent(R::Permanent.and(R::Nonland)),
                chooser: PlayerRef::You,
                chosen: Box::new(Effect::AddCounter {
                    what: Selector::SeparatedPile { chosen: true },
                    kind: CounterType::Doom,
                    amount: Value::ONE,
                }),
                other: Box::new(Effect::Noop),
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(R::WithCounter(CounterType::Doom)),
                body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fissure Vent — choose one or both: destroy an artifact; destroy a
/// nonbasic land.
pub fn fissure_vent() -> CardDefinition {
    spell(
        "Fissure Vent",
        cost(&[generic(3), r(), r()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            min: 1,
            max: 2,
            allow_repeats: false,
            modes: vec![
                Effect::Destroy { what: target_filtered(R::Artifact) },
                Effect::Destroy { what: target_filtered(R::IsNonbasicLand) },
            ],
        },
    )
}

/// Hooded Horror — can't be blocked while the defending player controls the
/// most creatures (or ties).
pub fn hooded_horror() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedIfDefenderHasMostCreatures],
        ..creature("Hooded Horror", cost(&[generic(4), b()]), vec![CreatureType::Horror], 4, 4)
    }
}

/// Incendiary Command — choose two: 4 damage to a player or planeswalker; 2
/// to each creature; destroy a nonbasic land; each player wheels their hand.
pub fn incendiary_command() -> CardDefinition {
    spell(
        "Incendiary Command",
        cost(&[generic(3), r(), r()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            min: 2,
            max: 2,
            allow_repeats: false,
            modes: vec![
                Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: Value::Const(4),
                },
                Effect::ForEach {
                    selector: Selector::EachPermanent(R::Creature),
                    body: Box::new(Effect::DealDamage {
                        to: Selector::TriggerSource,
                        amount: Value::Const(2),
                    }),
                },
                Effect::Destroy { what: target_filtered(R::IsNonbasicLand) },
                Effect::DiscardHandDrawThatMany { who: Selector::Player(PlayerRef::EachPlayer) },
            ],
        },
    )
}

/// Molten Disaster — kicker {R} (split second if kicked); X damage to each
/// creature without flying and each player.
pub fn molten_disaster() -> CardDefinition {
    let hit = |selector: Selector| Effect::ForEach {
        selector,
        body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::XFromCost }),
    };
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[r()])), Keyword::SplitSecondIfKicked],
        ..spell(
            "Molten Disaster",
            cost(&[x(), r(), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                hit(Selector::EachPermanent(
                    R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                )),
                hit(Selector::Player(PlayerRef::EachPlayer)),
            ]),
        )
    }
}

/// Obelisk of Grixis — {T}: {U}, {B} or {R}.
pub fn obelisk_of_grixis() -> CardDefinition {
    CardDefinition {
        name: "Obelisk of Grixis",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Blue, Color::Black, Color::Red], Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Phthisis — destroy a creature; its controller loses life equal to its
/// power plus toughness. Suspend 5—{1}{B}.
pub fn phthisis() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(5, cost(&[generic(1), b()]))],
        ..spell(
            "Phthisis",
            cost(&[generic(3), b(), b(), b(), b()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(target_filtered(
                        R::Creature,
                    )))),
                    amount: Value::Sum(vec![
                        Value::PowerOf(Box::new(Selector::Target(0))),
                        Value::ToughnessOf(Box::new(Selector::Target(0))),
                    ]),
                },
                Effect::Destroy { what: Selector::Target(0) },
            ]),
        )
    }
}

/// Price of Knowledge — players have no maximum hand size; each opponent's
/// upkeep deals them damage equal to their hand size.
pub fn price_of_knowledge() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Players have no maximum hand size.",
            effect: StaticEffect::AllPlayersNoMaximumHandSize,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::OpponentControl,
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::HandSizeOf(PlayerRef::ActivePlayer),
            },
        }],
        ..spell("Price of Knowledge", cost(&[generic(6), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Thraximundar — haste; attacking, the defending player sacrifices a
/// creature; any creature sacrifice may grow it.
pub fn thraximundar() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            on_attack(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                count: Value::ONE,
                filter: R::Creature,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::AnyPlayer),
                effect: Effect::MayDo {
                    description: "Put a +1/+1 counter on Thraximundar?".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
            },
        ],
        ..creature(
            "Thraximundar",
            cost(&[generic(4), u(), b(), r()]),
            vec![CreatureType::Zombie, CreatureType::Assassin],
            6,
            6,
        )
    }
}

/// True-Name Nemesis — as it enters, choose a player; protection from that
/// player.
pub fn true_name_nemesis() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ProtectionFromChosenPlayer],
        as_enters_effect: Some(Effect::ChoosePlayerForSource { opponent: false }),
        ..creature(
            "True-Name Nemesis",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Rogue],
            3,
            1,
        )
    }
}

/// Urza's Factory — {T}: {C}; {7}, {T}: a 2/2 Assembly-Worker.
pub fn urzas_factory() -> CardDefinition {
    CardDefinition {
        name: "Urza's Factory",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Urza], ..Default::default() },
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(7)]),
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: std::sync::Arc::new(TokenDefinition {
                        name: "Assembly-Worker".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::AssemblyWorker],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
