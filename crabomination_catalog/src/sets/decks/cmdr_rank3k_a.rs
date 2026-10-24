//! Commander: one-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), first file. Tests in `tests/recent_b/cmdr_rank3k_a.rs`.

use crate::card::{
    ActivatedAbility, Adventure, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered, unearth};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};
use std::sync::Arc;

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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn token(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, p: i32, t: i32) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

fn make(n: Value, def: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: def }
}

/// Wriggling Grub — {1}{B} 1/1 Worm; dies: two 1/1 black and green Worms.
pub fn wriggling_grub() -> CardDefinition {
    let worm = token("Worm", vec![Color::Black, Color::Green], vec![CreatureType::Worm], 1, 1);
    CardDefinition {
        triggered_abilities: vec![on_dies(make(Value::Const(2), worm))],
        ..creature("Wriggling Grub", cost(&[generic(1), b()]), vec![CreatureType::Worm], 1, 1)
    }
}

/// Lady Zhurong, Warrior Queen — {4}{G} legendary 4/3; horsemanship.
pub fn lady_zhurong_warrior_queen() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Horsemanship],
        ..legend(
            "Lady Zhurong, Warrior Queen",
            cost(&[generic(4), g()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Warrior],
            4,
            3,
        )
    }
}

/// Lu Bu, Master-at-Arms — {5}{R} legendary 4/3; haste, horsemanship.
pub fn lu_bu_master_at_arms() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Horsemanship],
        ..legend(
            "Lu Bu, Master-at-Arms",
            cost(&[generic(5), r()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Warrior],
            4,
            3,
        )
    }
}

/// Elvenking's Harper — {1}{U} 2/2 Elf Bard. {4}{U}: target creature can't be
/// blocked this turn.
pub fn elvenkings_harper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Elvenking's Harper", cost(&[generic(1), u()]), vec![CreatureType::Elf, CreatureType::Bard], 2, 2)
    }
}

/// Fell the Pheasant — {1}{G} instant: 5 damage to target creature with
/// flying, and a Food.
pub fn fell_the_pheasant() -> CardDefinition {
    spell(
        "Fell the Pheasant",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                amount: Value::Const(5),
            },
            make(Value::ONE, Arc::new(crabomination_base::tokens::food_token())),
        ]),
    )
}

/// Hornswoggle — {2}{U} instant: counter target creature spell; you create a
/// Treasure.
pub fn hornswoggle() -> CardDefinition {
    spell(
        "Hornswoggle",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Creature))) },
            make(Value::ONE, Arc::new(crabomination_base::tokens::treasure_token())),
        ]),
    )
}

/// Angelsong — {1}{W} instant: prevent all combat damage this turn. Cycling {2}.
pub fn angelsong() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..spell("Angelsong", cost(&[generic(1), w()]), CardType::Instant, Effect::PreventAllCombatDamageThisTurn)
    }
}

/// Djeru's Resolve — {W} instant: untap target creature and prevent all damage
/// that would be dealt to it this turn. Cycling {2}.
pub fn djerus_resolve() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..spell(
            "Djeru's Resolve",
            cost(&[w()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Untap { what: target_filtered(R::Creature), up_to: None },
                Effect::PreventAllDamageThisTurn { target: Selector::Target(0), redirect_to: None },
            ]),
        )
    }
}

/// Simulacrum Shaper — {1}{G}{G} 2/2 Elf Druid. ETB: you may fetch a basic land
/// tapped. Dies: draw a card.
pub fn simulacrum_shaper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Search your library for a basic land card?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
            }),
            on_dies(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        ],
        ..creature("Simulacrum Shaper", cost(&[generic(1), g(), g()]), vec![CreatureType::Elf, CreatureType::Druid], 2, 2)
    }
}

/// Super Intelligence — {U} Aura: at the beginning of the upkeep of enchanted
/// creature's controller, that player draws a card (granted to the host, whose
/// controller is that player).
pub fn super_intelligence() -> CardDefinition {
    CardDefinition {
        name: "Super Intelligence",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Archive Arbiter — {6} 4/4 flying Sphinx artifact creature. ETB, choose one:
/// destroy target noncreature, nonland permanent; or gain 4 life.
pub fn archive_arbiter() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Permanent.and(R::Creature.negate()).and(R::Land.negate())) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        ]))],
        ..creature("Archive Arbiter", cost(&[generic(6)]), vec![CreatureType::Sphinx], 4, 4)
    }
}

/// Drogskol Cavalry — {5}{W}{W} 4/4 flying Spirit Knight. Another Spirit you
/// control enters: gain 2 life. {3}{W}: a 1/1 white flying Spirit.
pub fn drogskol_cavalry() -> CardDefinition {
    let spirit = Arc::new(TokenDefinition {
        keywords: vec![Keyword::Flying],
        ..(*token("Spirit", vec![Color::White], vec![CreatureType::Spirit], 1, 1)).clone()
    });
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Spirit)),
                },
            ),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: make(Value::ONE, spirit),
            ..Default::default()
        }],
        ..creature(
            "Drogskol Cavalry",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Spirit, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// Deadeye Plunderers — {3}{U}{B} 3/3 Human Pirate; +1/+1 for each artifact you
/// control. {2}{U}{B}: create a Treasure.
pub fn deadeye_plunderers() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each artifact you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents { filter: R::Artifact, per_power: 1, per_toughness: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), b()]),
            effect: make(Value::ONE, Arc::new(crabomination_base::tokens::treasure_token())),
            ..Default::default()
        }],
        ..creature(
            "Deadeye Plunderers",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            3,
            3,
        )
    }
}

/// Barrage of Expendables — {R} enchantment. {R}, sacrifice a creature: 1 damage
/// to any target.
pub fn barrage_of_expendables() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::DealDamage { to: target_filtered(R::any_target()), amount: Value::ONE },
            ..Default::default()
        }],
        ..spell("Barrage of Expendables", cost(&[r()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Havoc Jester — {4}{R} 5/5 Devil. Whenever you sacrifice a permanent, it
/// deals 1 damage to any target.
pub fn havoc_jester() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl),
            effect: Effect::DealDamage { to: target_filtered(R::Any), amount: Value::ONE },
        }],
        ..creature("Havoc Jester", cost(&[generic(4), r()]), vec![CreatureType::Devil], 5, 5)
    }
}

/// Graceful Reprieve — {1}{W} instant: when target creature dies this turn,
/// return that card to the battlefield under its owner's control.
pub fn graceful_reprieve() -> CardDefinition {
    spell(
        "Graceful Reprieve",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::WhenTargetDiesThisTurn {
            body: Box::new(Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                    tapped: false,
                },
            }),
            slot: 0,
            filter: Some(R::Creature),
        },
    )
}

/// Germinate Recruits — {2}{W} instant: X 2/2 colorless Wizard Soldier tokens
/// named Cadet, X = the life you gained this turn.
pub fn germinate_recruits() -> CardDefinition {
    let cadet = token("Cadet", vec![], vec![CreatureType::Wizard, CreatureType::Soldier], 2, 2);
    spell(
        "Germinate Recruits",
        cost(&[generic(2), w()]),
        CardType::Instant,
        make(Value::LifeGainedThisTurn(PlayerRef::You), cadet),
    )
}

/// Platoon Dispenser — {5} 4/6 Construct artifact creature. End step, if you
/// control two or more other creatures: draw. {3}{W}: a 1/1 colorless Soldier
/// artifact creature. Unearth {2}{W}{W}.
pub fn platoon_dispenser() -> CardDefinition {
    let soldier = Arc::new(TokenDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..(*token("Soldier", vec![], vec![CreatureType::Soldier], 1, 1)).clone()
    });
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                Predicate::ValueAtLeast(
                    Value::count(Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource))),
                    Value::Const(2),
                ),
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![
            ActivatedAbility { mana_cost: cost(&[generic(3), w()]), effect: make(Value::ONE, soldier), ..Default::default() },
            unearth(cost(&[generic(2), w(), w()])),
        ],
        ..creature("Platoon Dispenser", cost(&[generic(5)]), vec![CreatureType::Construct], 4, 6)
    }
}

/// Gigantomancer — {7}{G} 1/1 Human Shaman. {1}: target creature you control
/// has base power and toughness 7/7 until end of turn.
pub fn gigantomancer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::SetBasePT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(7),
                toughness: Value::Const(7),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Gigantomancer", cost(&[generic(7), g()]), vec![CreatureType::Human, CreatureType::Shaman], 1, 1)
    }
}

/// Feral Hydra — {X}{G} 0/0 Hydra Beast; enters with X +1/+1 counters. {3}: a
/// +1/+1 counter on it; any player may activate this ability.
pub fn feral_hydra() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            any_player: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Feral Hydra", cost(&[crate::mana::x(), g()]), vec![CreatureType::Hydra, CreatureType::Beast], 0, 0)
    }
}

/// Wilderland Scrounger — {4}{G} 3/6 Wolf. Ferocious — whenever it attacks
/// while you control a creature with power 4 or greater, a +1/+1 counter on
/// each creature you control.
pub fn wilderland_scrounger() -> CardDefinition {
    let mut t = on_attack(Effect::AddCounter {
        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    });
    t.event = t.event.with_filter(Predicate::FerociousActive { who: PlayerRef::You });
    CardDefinition {
        triggered_abilities: vec![t],
        ..creature("Wilderland Scrounger", cost(&[generic(4), g()]), vec![CreatureType::Wolf], 3, 6)
    }
}

/// Budding Insurgent — {2}{G} 3/3 vigilant Dryad Scout. Sacrifice it: destroy
/// target artifact or enchantment; if it was a legendary enchantment, draw.
/// Sorcery speed.
pub fn budding_insurgent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::Enchantment.and(R::HasSupertype(Supertype::Legendary)),
                    },
                    then: Box::new(Effect::Seq(vec![
                        Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ])),
                    else_: Box::new(Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) }),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Budding Insurgent", cost(&[generic(2), g()]), vec![CreatureType::Dryad, CreatureType::Scout], 3, 3)
    }
}

/// Nephalia Smuggler — {U} 1/1 Human Rogue. {3}{U}, {T}: exile another target
/// creature you control, then return it under your control.
pub fn nephalia_smuggler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)) },
                Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ]),
            ..Default::default()
        }],
        ..creature("Nephalia Smuggler", cost(&[u()]), vec![CreatureType::Human, CreatureType::Rogue], 1, 1)
    }
}

/// Skemfar Avenger — {1}{B} 3/1 Elf Berserker. Whenever another nontoken Elf or
/// Berserker you control dies, you draw a card and lose 1 life.
pub fn skemfar_avenger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken.and(
                        R::HasCreatureType(CreatureType::Elf).or(R::HasCreatureType(CreatureType::Berserker)),
                    ),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..creature("Skemfar Avenger", cost(&[generic(1), b()]), vec![CreatureType::Elf, CreatureType::Berserker], 3, 1)
    }
}

/// Casey Jones, Vigilante — {1}{R}{R} legendary 4/3 Human Berserker. ETB: draw
/// three; at the beginning of your next upkeep, discard three at random.
pub fn casey_jones_vigilante() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::AtYourNextUpkeep {
                body: Box::new(Effect::Discard { who: Selector::You, amount: Value::Const(3), random: true }),
            },
        ]))],
        ..legend(
            "Casey Jones, Vigilante",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Berserker],
            4,
            3,
        )
    }
}

/// Rise of the Deathbringer — {4}{B} instant. Choose one: draw cards equal to
/// the greatest power among creatures you control and lose that much life
/// (the cards actually drawn); or all creatures get -3/-3 until end of turn.
pub fn rise_of_the_deathbringer() -> CardDefinition {
    spell(
        "Rise of the Deathbringer",
        cost(&[generic(4), b()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::GreatestPowerControlled { who: PlayerRef::You } },
                Effect::LoseLife { who: Selector::You, amount: Value::CardsDrawnThisEffect },
            ]),
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Kindred Judgment — {5}{W}{W} sorcery: choose a creature type; destroy all
/// creatures that aren't of it.
pub fn kindred_judgment() -> CardDefinition {
    spell(
        "Kindred Judgment",
        cost(&[generic(5), w(), w()]),
        CardType::Sorcery,
        Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::Destroy {
                what: Selector::EachPermanent(R::Creature.and(R::IsSourceChosenCreatureType.negate())),
            }),
        },
    )
}

/// Ashling, the Extinguisher — {2}{B}{B} legendary 4/4. Combat damage to a
/// player: choose target creature that player controls; they sacrifice it.
pub fn ashling_the_extinguisher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::SacrificePermanent { what: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)) },
        }],
        ..legend(
            "Ashling, the Extinguisher",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Elemental, CreatureType::Shaman],
            4,
            4,
        )
    }
}

/// Dong Zhou, the Tyrant — {4}{R} legendary 3/3. ETB: target creature an
/// opponent controls deals damage equal to its power to that player.
pub fn dong_zhou_the_tyrant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DealDamageFrom {
            source: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
            amount: Value::PowerOf(Box::new(Selector::Target(0))),
        })],
        ..legend(
            "Dong Zhou, the Tyrant",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Ghost Lantern // Bind Spirit — {B} Equipment: whenever a creature you control
/// dies, a +1/+1 counter on the equipped creature. Equip {1}. Adventure {1}{B}
/// instant: return target creature card from your graveyard to your hand.
pub fn ghost_lantern() -> CardDefinition {
    CardDefinition {
        name: "Ghost Lantern",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        adventure: Some(Box::new(Adventure {
            name: "Bind Spirit",
            cost: cost(&[generic(1), b()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Move {
                what: target_filtered(R::InYourGraveyard.and(R::Creature)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        })),
        ..Default::default()
    }
}
