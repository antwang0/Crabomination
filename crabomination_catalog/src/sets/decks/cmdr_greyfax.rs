//! Commander: the cards the **Forces of the Imperium** Warhammer 40,000
//! Commander deck (40K, Inquisitor Greyfax) needed beyond what the catalog
//! had. Tests in `tests/recent_b/cmdr_greyfax.rs`.
//!
//! Residuals (each also on its card):
//! - **Callidus Assassin** — enters untapped; its granted trigger destroys a
//!   creature that shares a name with another permanent, not only the copy's.
//! - **Cybernetica Datasmith** — the two target players may be the same.
//! - **Inquisitor Eisenhorn** — the first-draw reveal is automatic.
//! - **Neyam Shai Murad** — the returned card may come from any opponent's
//!   graveyard, and the card you take is the engine's pick, not theirs.
//! - **Redemptor Dreadnought** — the optional graveyard exile (and the
//!   attack pump it feeds) isn't offered.
//! - **Triumph of Saint Katherine** — Praesidium Protectiva isn't
//!   implemented.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EntersAsCopy, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, PlayerTally,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{battle_cry, etb, on_attack, on_dies, on_other_dies, target_any, target_filtered};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, u, w, x};
use crabomination_base::tokens::clue_token;
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn vehicle(name: &'static str, mana: ManaCost, p: i32, t: i32, crew: u32) -> CardDefinition {
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

/// The deck's 2/2 white Astartes Warrior with vigilance.
fn astartes() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Astartes Warrior".to_string(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Vigilance],
        subtypes: Subtypes { creature_types: vec![CreatureType::Astartes, CreatureType::Warrior], ..Default::default() },
        ..Default::default()
    })
}

fn make(n: Value, def: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: def }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn investigate(n: Value) -> Effect {
    make(n, Arc::new(clue_token()))
}

/// Assault Intercessor — first strike, menace; an opponent's creature dying
/// costs that player 2 life.
pub fn assault_intercessor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(2) },
        }],
        ..creature("Assault Intercessor", cost(&[generic(1), w(), b()]), vec![CreatureType::Astartes, CreatureType::Warrior], 3, 2)
    }
}

/// Belisarius Cawl — tap two artifacts: an Astartes; tap X creatures: dig X
/// for an artifact.
pub fn belisarius_cawl() -> CardDefinition {
    legendary(CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                tap_n_filter: Some((R::Artifact.and(R::OtherThanSource), 2)),
                effect: make(Value::ONE, astartes()),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                tap_n_filter: Some((R::Creature.and(R::OtherThanSource), 0)),
                tap_n_x: true,
                effect: Effect::LookPickToHand(Box::new(LookPick {
                    who: PlayerRef::You,
                    count: Value::XFromCost,
                    pick_filter: Some(R::Artifact),
                    optional: true,
                    rest_bottom_random: true,
                    ..Default::default()
                })),
                ..Default::default()
            },
        ],
        ..creature("Belisarius Cawl", cost(&[generic(2), w(), u()]), vec![CreatureType::Human], 2, 4)
    })
}

/// Birth of the Imperium — Saga: Astartes per opponent; each opponent
/// sacrifices a creature; draw two per opponent with fewer creatures.
pub fn birth_of_the_imperium() -> CardDefinition {
    CardDefinition {
        name: "Birth of the Imperium",
        cost: cost(&[generic(2), w(), u(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, make(Value::OpponentCount, astartes())),
            (
                2,
                Effect::Sacrifice { who: Selector::Player(PlayerRef::EachOpponent), count: Value::ONE, filter: R::Creature },
            ),
            (
                3,
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Times(
                        Box::new(Value::Const(2)),
                        Box::new(Value::PlayersWithFewerTally(PlayerTally::CreaturesControlled)),
                    ),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Callidus Assassin — flash; may enter as a copy of any creature, with
/// "destroy up to one other creature with this name".
///
/// ⚠ Residual: enters untapped; the trigger reads "shares a name with
/// another permanent".
pub fn callidus_assassin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_triggered: vec![etb(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::OtherThanSource).and(R::SharesNameWithAnotherPermanent),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            })],
            ..Default::default()
        }),
        ..creature(
            "Callidus Assassin",
            cost(&[generic(4), u(), b()]),
            vec![CreatureType::Human, CreatureType::Shapeshifter, CreatureType::Assassin],
            3,
            3,
        )
    }
}

/// Celestine, the Living Saint — flying, lifelink; your end step returns a
/// creature card with mana value up to the life you gained this turn.
pub fn celestine_the_living_saint() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMostLifeGainedThisTurn)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..creature("Celestine, the Living Saint", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 4)
    })
}

/// Commissar Severina Raine — attacking drains each opponent by the other
/// attackers; {2}, sacrifice another creature: gain 2, draw.
pub fn commissar_severina_raine() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![on_attack(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::CountOf(Box::new(Selector::EachPermanent(
                R::Creature.and(R::IsAttacking).and(R::OtherThanSource),
            ))),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Creature.and(R::OtherThanSource), 1)),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..creature("Commissar Severina Raine", cost(&[generic(1), w(), b()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    })
}

/// Company Commander — a 1/1 Soldier per opponent; attacking gives your
/// creatures deathtouch.
pub fn company_commander() -> CardDefinition {
    let soldier = Arc::new(TokenDefinition {
        name: "Soldier".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: vec![
            etb(make(Value::OpponentCount, soldier)),
            on_attack(Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn }),
        ],
        ..creature("Company Commander", cost(&[generic(2), w(), b()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 4)
    }
}

/// Cybernetica Datasmith — protection from Robots; {U}, {T}: one player
/// draws, another gets a 4/4 Robot that can't block.
///
/// ⚠ Residual: the two target players may be the same.
pub fn cybernetica_datasmith() -> CardDefinition {
    let robot = Arc::new(TokenDefinition {
        name: "Robot".to_string(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::CantBlock],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::ProtectionFromCreatureType(CreatureType::Robot)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: target_filtered(R::Player), amount: Value::ONE },
                Effect::CreateToken { who: PlayerRef::Target(1), count: Value::ONE, definition: robot },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Cybernetica Datasmith",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            0,
            1,
        )
    }
}

/// Defenders of Humanity — X Astartes on entry; later, with no creatures on
/// your turn, pay again and exile it for X more.
pub fn defenders_of_humanity() -> CardDefinition {
    CardDefinition {
        name: "Defenders of Humanity",
        cost: cost(&[x(), generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(make(Value::XFromCost, astartes()))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), generic(2), w()]),
            exile_self_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::Not(Box::new(Predicate::SelectorExists(yours(R::Creature)))),
            ])),
            effect: make(Value::XFromCost, astartes()),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Deny the Witch — counter a spell or ability; its controller loses life
/// equal to your creatures.
pub fn deny_the_witch() -> CardDefinition {
    CardDefinition {
        name: "Deny the Witch",
        cost: cost(&[generic(1), w(), u(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(target_filtered(
                    R::IsSpellOnStack.or(R::HasAbilityOnStack),
                )))),
                amount: Value::CreatureCountControlledBy(PlayerRef::You),
            },
            Effect::CounterSpellOrAbility { what: Selector::Target(0) },
        ]),
        ..Default::default()
    }
}

/// Epistolary Librarian — attacking lets you cast a spell with mana value up
/// to the number of attackers free from hand.
pub fn epistolary_librarian() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MayCastFromHandFreeMatching {
            filter: R::Nonland,
            max_mv: Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::IsAttacking)))),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Epistolary Librarian", cost(&[generic(2), w(), u()]), vec![CreatureType::Astartes, CreatureType::Wizard], 3, 4)
    }
}

/// Exterminatus — opponents' nonland permanents lose indestructible; destroy
/// all nonland permanents.
pub fn exterminatus() -> CardDefinition {
    CardDefinition {
        name: "Exterminatus",
        cost: cost(&[generic(5), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LoseKeyword {
                what: Selector::EachPermanent(R::Nonland.and(R::ControlledByOpponent)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::Destroy { what: Selector::EachPermanent(R::Nonland) },
        ]),
        ..Default::default()
    }
}

/// For the Emperor! — +2/+2, vigilance and lifelink for your creatures.
pub fn for_the_emperor() -> CardDefinition {
    CardDefinition {
        name: "For the Emperor!",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT { what: yours(R::Creature), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Grey Knight Paragon — flash; entering destroys an attacking creature,
/// exiling a Demon instead.
pub fn grey_knight_paragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 1,
            filter: R::Creature.and(R::IsAttacking),
            effect: Box::new(Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::HasCreatureType(CreatureType::Demon) },
                then: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
                else_: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            }),
        })],
        ..creature("Grey Knight Paragon", cost(&[generic(4), w()]), vec![CreatureType::Astartes, CreatureType::Knight], 4, 4)
    }
}

/// Inquisitor Eisenhorn — an instant or sorcery as your first draw makes
/// Cherubael; combat damage investigates that many times.
///
/// ⚠ Residual: the reveal is automatic.
pub fn inquisitor_eisenhorn() -> CardDefinition {
    let cherubael = Arc::new(TokenDefinition {
        name: "Cherubael".to_string(),
        power: 4,
        toughness: 4,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        ..Default::default()
    });
    legendary(CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::FirstCardDrawnThisTurn, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    },
                ),
                effect: make(Value::ONE, cherubael),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: investigate(Value::TriggerEventAmount),
            },
        ],
        ..creature("Inquisitor Eisenhorn", cost(&[generic(2), u(), b()]), vec![CreatureType::Human, CreatureType::Inquisitor], 2, 3)
    })
}

/// Inquisitor Greyfax — vigilance; other creatures get +1/+0 and vigilance;
/// {1}, {T}: tap an opponent's creature and investigate.
pub fn inquisitor_greyfax() -> CardDefinition {
    let others = || yours(R::Creature.and(R::OtherThanSource));
    legendary(CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![
            StaticAbility {
                description: "Other creatures you control get +1/+0.",
                effect: StaticEffect::PumpPT { applies_to: others(), power: 1, toughness: 0 },
            },
            StaticAbility {
                description: "Other creatures you control have vigilance.",
                effect: StaticEffect::GrantKeyword { applies_to: others(), keyword: Keyword::Vigilance },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
                investigate(Value::ONE),
            ]),
            ..Default::default()
        }],
        ..creature(
            "Inquisitor Greyfax",
            cost(&[generic(1), w(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Inquisitor],
            3,
            3,
        )
    })
}

/// Inquisitorial Rosette — the equipped creature attacking brings an
/// attacking Astartes, then attackers gain menace; equip {3}.
pub fn inquisitorial_rosette() -> CardDefinition {
    CardDefinition {
        name: "Inquisitorial Rosette",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![on_attack(Effect::Seq(vec![
                Effect::CreateTokenAttacking {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: astartes(),
                    cleanup: Default::default(),
                    defender: None,
                },
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Knight Paladin — trample; entering deals 4 to each opponent; crew 1.
pub fn knight_paladin() -> CardDefinition {
    let mut def = vehicle("Knight Paladin", cost(&[generic(5)]), 6, 6, 1);
    def.keywords.push(Keyword::Trample);
    def.triggered_abilities =
        vec![etb(Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(4) })];
    def
}

/// Marneus Calgar — double strike; tokens entering draw a card; {6}: two
/// Astartes.
pub fn marneus_calgar() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsToken })
                .once_per_batch(),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            effect: make(Value::Const(2), astartes()),
            ..Default::default()
        }],
        ..creature("Marneus Calgar", cost(&[generic(2), w(), u(), b()]), vec![CreatureType::Astartes, CreatureType::Warrior], 3, 5)
    })
}

/// Neyam Shai Murad — connecting may trade graveyard cards: that player gets
/// a permanent card back to hand, you reanimate one of yours.
///
/// ⚠ Residual: any opponent's graveyard; the engine picks your card.
pub fn neyam_shai_murad() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Trade graveyard permanents with that player?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(R::PermanentCard.and(R::InOpponentGraveyard)),
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    },
                    Effect::Move {
                        what: Selector::Take {
                            inner: Box::new(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::PermanentCard }),
                            count: Box::new(Value::ONE),
                        },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                ])),
            },
        }],
        ..creature("Neyam Shai Murad", cost(&[generic(2), w(), b()]), vec![CreatureType::Human, CreatureType::Rogue], 3, 3)
    })
}

/// Primaris Chaplain — battle cry; attacking makes it indestructible.
pub fn primaris_chaplain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            battle_cry(1),
            on_attack(Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Indestructible, duration: Duration::EndOfTurn }),
        ],
        ..creature("Primaris Chaplain", cost(&[generic(2), w(), b()]), vec![CreatureType::Astartes, CreatureType::Cleric], 3, 3)
    }
}

/// Primaris Eliminator — entering destroys a creature, or shrinks a player's
/// creatures by -2/-2.
pub fn primaris_eliminator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::PumpPT {
                what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature("Primaris Eliminator", cost(&[generic(4), b()]), vec![CreatureType::Astartes, CreatureType::Warrior], 3, 2)
    }
}

/// Reaver Titan — protection from mana value 3 or less; attacking deals 5 to
/// each opponent; crew 4.
pub fn reaver_titan() -> CardDefinition {
    let mut def = vehicle("Reaver Titan", cost(&[generic(7)]), 10, 10, 4);
    def.keywords.push(Keyword::ProtectionFromMatching(Box::new(R::ManaValueAtMost(3))));
    def.triggered_abilities =
        vec![on_attack(Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(5) })];
    def
}

/// Redemptor Dreadnought — a 4/4 trampler.
///
/// ⚠ Residual: the optional graveyard exile and its attack pump aren't
/// offered.
pub fn redemptor_dreadnought() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Trample],
        ..creature("Redemptor Dreadnought", cost(&[generic(5)]), vec![CreatureType::Astartes, CreatureType::Dreadnought], 4, 4)
    }
}

/// Sanguinary Priest — lifelink; another creature of yours dying pings.
pub fn sanguinary_priest() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![on_other_dies(Effect::DealDamage { to: target_any(), amount: Value::ONE })],
        ..creature("Sanguinary Priest", cost(&[generic(3), b()]), vec![CreatureType::Astartes, CreatureType::Cleric], 2, 4)
    }
}

/// Sister Hospitaller — reanimate a creature card and gain its mana value.
pub fn sister_hospitaller() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::GainLife { who: Selector::You, amount: Value::ManaValueOf(Box::new(Selector::Target(0))) },
        ]))],
        ..creature("Sister Hospitaller", cost(&[generic(4), w(), b()]), vec![CreatureType::Human, CreatureType::Cleric], 3, 2)
    }
}

/// Sister Repentia — dying gains 2 and draws two; miracle {W}{B}.
pub fn sister_repentia() -> CardDefinition {
    CardDefinition {
        miracle: Some(cost(&[w(), b()])),
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..creature("Sister Repentia", cost(&[generic(3), w(), b()]), vec![CreatureType::Human, CreatureType::Warrior], 5, 1)
    }
}

/// Sister of Silence — flash; entering counters an instant, a sorcery or an
/// ability.
pub fn sister_of_silence() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::CounterSpellOrAbility {
            what: target_filtered(
                R::IsSpellOnStack
                    .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))
                    .or(R::HasAbilityOnStack),
            ),
        })],
        ..creature("Sister of Silence", cost(&[generic(4), u()]), vec![CreatureType::Human, CreatureType::Knight], 3, 3)
    }
}

/// Space Marine Scout — first strike, vigilance; behind on lands, fetch a
/// Plains tapped.
pub fn space_marine_scout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::OpponentControlsMoreLandsThanYou),
            effect: Effect::MayDo {
                description: "Search for a Plains card?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasLandType(LandType::Plains),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
            },
        }],
        ..creature("Space Marine Scout", cost(&[generic(2), w()]), vec![CreatureType::Astartes, CreatureType::Scout], 2, 1)
    }
}

/// The Flesh Is Weak — a +1/+1 counter on each creature you control; your
/// countered creatures are artifacts; nonartifact creatures get -1/-1.
pub fn the_flesh_is_weak() -> CardDefinition {
    CardDefinition {
        name: "The Flesh Is Weak",
        cost: cost(&[generic(2), w(), u(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: yours(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control with +1/+1 counters on them are artifacts in addition to their other types.",
                effect: StaticEffect::AddCardTypeToMatching {
                    applies_to: yours(R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))),
                    card_type: CardType::Artifact,
                    artifact_subtype: None,
                },
            },
            StaticAbility {
                description: "Nonartifact creatures get -1/-1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::HasCardType(CardType::Artifact).negate())),
                    power: -1,
                    toughness: -1,
                },
            },
        ],
        ..Default::default()
    }
}

/// The Golden Throne — a loss exiles it instead and sets your life to 1;
/// {T}, sacrifice a creature: three mana in any colors.
pub fn the_golden_throne() -> CardDefinition {
    CardDefinition {
        name: "The Golden Throne",
        cost: cost(&[generic(4)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "If you would lose the game, instead exile The Golden Throne and your life total becomes 1.",
            effect: StaticEffect::ReplaceControllerLossWithExileSelf,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::Const(3)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Thunderhawk Gunship — flying; two Astartes on entry; attacking gives your
/// attackers flying; crew 2.
pub fn thunderhawk_gunship() -> CardDefinition {
    let mut def = vehicle("Thunderhawk Gunship", cost(&[generic(6)]), 6, 6, 2);
    def.keywords.push(Keyword::Flying);
    def.triggered_abilities = vec![
        etb(make(Value::Const(2), astartes())),
        on_attack(Effect::GrantKeyword {
            what: yours(R::Creature.and(R::IsAttacking)),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        }),
    ];
    def
}

/// Thunderwolf Cavalry — first strike; connecting puts a +1/+1 counter on
/// each other creature you control.
pub fn thunderwolf_cavalry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: yours(R::Creature.and(R::OtherThanSource)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Thunderwolf Cavalry", cost(&[generic(4), w()]), vec![CreatureType::Astartes, CreatureType::Warrior], 4, 4)
    }
}

/// Triumph of Saint Katherine — lifelink; miracle {1}{W}.
///
/// ⚠ Residual: Praesidium Protectiva isn't implemented.
pub fn triumph_of_saint_katherine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        miracle: Some(cost(&[generic(1), w()])),
        ..creature("Triumph of Saint Katherine", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Warrior], 5, 5)
    }
}

/// Vexilus Praetor — flash, vigilance; your commanders have protection from
/// everything.
pub fn vexilus_praetor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Commanders you control have protection from everything.",
            effect: StaticEffect::GrantKeyword { applies_to: yours(R::IsCommander), keyword: Keyword::ProtectionFromEverything },
        }],
        ..creature("Vexilus Praetor", cost(&[generic(3), w()]), vec![CreatureType::Custodes, CreatureType::Warrior], 3, 4)
    }
}
