//! Commander: the cards the **Scions & Spellcraft** precon (FIC, Y'shtola,
//! Night's Blessed) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_yshtola.rs`.
//!
//! Residuals (each also on its card):
//! - **Blue Mage's Cane** — the copy costs the card's own mana cost, not
//!   {3}, and the graveyard card isn't exiled.
//! - **Estinien Varlineau** — counts opponents dealt combat damage by any
//!   creature, not only by it or a Dragon.
//! - **Hildibrand Manderville** — dying doesn't let you cast it from the
//!   graveyard as an Adventure.
//! - **Urianger Augurelt** — the card is exiled face up; a land played from
//!   exile gains no life; its spells get no {2} discount.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword, LandType,
    MayPlayDuration, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, partner_with_search, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, u, w, x};
use crate::sets::{enters_tapped, tap_add};
use crabomination_base::tokens::treasure_token;
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

fn token(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, p: i32, t: i32, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.to_string(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        keywords,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

/// The 1/1 colorless Hero token (CR 702.182).
fn hero() -> Arc<TokenDefinition> {
    token("Hero", vec![], vec![CreatureType::Hero], 1, 1, vec![])
}

/// Job select: mint a Hero and attach this Equipment to it (CR 702.182).
fn job_select() -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: hero() },
        Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
    ]))
}

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        triggered_abilities: vec![job_select()],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// "Whenever you cast a noncreature spell, [effect]".
fn on_cast_noncreature(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::Noncreature)),
        effect,
    }
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

/// Alisaie Leveilleur — partner with Alphinaud; first strike; dualcast: your
/// second spell each turn costs {2} less.
pub fn alisaie_leveilleur() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::PartnerWith("Alphinaud Leveilleur".into()), Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "The second spell you cast each turn costs {2} less to cast.",
            effect: StaticEffect::CostReductionNthSpell { filter: R::Any, nth: 2, amount: 2 },
        }],
        triggered_abilities: vec![partner_with_search("Alphinaud Leveilleur")],
        ..creature("Alisaie Leveilleur", cost(&[generic(2), w()]), vec![CreatureType::Elf, CreatureType::Wizard], 3, 2)
    })
}

/// Alphinaud Leveilleur — partner with Alisaie; vigilance; eukrasia: your
/// second spell each turn draws a card.
pub fn alphinaud_leveilleur() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::PartnerWith("Alisaie Leveilleur".into()), Keyword::Vigilance],
        triggered_abilities: vec![
            partner_with_search("Alisaie Leveilleur"),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::Const(2) }),
                effect: draw(Value::ONE),
            },
        ],
        ..creature("Alphinaud Leveilleur", cost(&[generic(3), u()]), vec![CreatureType::Elf, CreatureType::Wizard], 2, 4)
    })
}

/// Ardbert, Warrior of Darkness — white spells grow your legends with
/// vigilance, black spells with menace.
pub fn ardbert_warrior_of_darkness() -> CardDefinition {
    let rally = |color: Color, keyword: Keyword| TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::HasColor(color))),
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: yours(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::GrantKeyword {
                what: yours(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
                keyword,
                duration: Duration::EndOfTurn,
            },
        ]),
    };
    legendary(CardDefinition {
        triggered_abilities: vec![rally(Color::White, Keyword::Vigilance), rally(Color::Black, Keyword::Menace)],
        ..creature(
            "Ardbert, Warrior of Darkness",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Spirit, CreatureType::Warrior],
            2,
            2,
        )
    })
}

/// Blue Mage's Cane — job select; equipped creature gets +0/+2, is a Wizard,
/// and attacking copies an instant or sorcery from the defending player's
/// graveyard to cast; equip {2}.
///
/// ⚠ Residual: the copy costs the card's own mana cost, not {3}, and the
/// graveyard card isn't exiled.
pub fn blue_mages_cane() -> CardDefinition {
    equipment(
        "Blue Mage's Cane",
        cost(&[generic(2), u()]),
        cost(&[generic(2)]),
        EquipBonus {
            toughness: 2,
            add_creature_types: vec![CreatureType::Wizard],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::CastWithoutPayingImmediate {
                    what: target_filtered(instant_or_sorcery().and(R::InGraveyard).and(R::OwnedByDefendingPlayer)),
                    source_zone: Zone::Graveyard,
                    exile_after: false,
                    copy: true,
                    reduce_generic: 0,
                    pay_own_cost: true,
                },
            }],
            ..Default::default()
        },
    )
}

/// Champions from Beyond — X Heroes; attacking with four or more creatures
/// scries 2 and draws; with eight or more, the attackers get +4/+4.
pub fn champions_from_beyond() -> CardDefinition {
    let attack_with = |n: u32, effect: Effect| TriggeredAbility {
        event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource)
            .with_filter(Predicate::AttackedWithCountAtLeast { who: PlayerRef::You, at_least: n }),
        effect,
    };
    CardDefinition {
        name: "Champions from Beyond",
        cost: cost(&[x(), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::XFromCost, definition: hero() }),
            attack_with(4, Effect::Seq(vec![Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }, draw(Value::ONE)])),
            attack_with(
                8,
                Effect::PumpPT {
                    what: yours(R::Creature.and(R::IsAttacking)),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
        ..Default::default()
    }
}

/// Dancer's Chakrams — job select; equipped creature gets +2/+2, lifelink,
/// is a Performer, and your other commanders get +2/+2 and lifelink; equip
/// {3}.
pub fn dancers_chakrams() -> CardDefinition {
    // "Other commanders": not the one wearing the Chakrams.
    let commanders = || yours(R::Creature.and(R::IsCommander).and(R::IsHostOfSource.negate()));
    let attached = || Predicate::EntityMatches { what: Selector::This, filter: R::AttachedToCreature };
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Other commanders you control get +2/+2.",
                effect: StaticEffect::WhileCondition {
                    condition: attached(),
                    inner: Box::new(StaticEffect::PumpPT { applies_to: commanders(), power: 2, toughness: 2 }),
                },
            },
            StaticAbility {
                description: "Other commanders you control have lifelink.",
                effect: StaticEffect::WhileCondition {
                    condition: attached(),
                    inner: Box::new(StaticEffect::GrantKeyword { applies_to: commanders(), keyword: Keyword::Lifelink }),
                },
            },
        ],
        ..equipment(
            "Dancer's Chakrams",
            cost(&[generic(3), w()]),
            cost(&[generic(3)]),
            EquipBonus {
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Lifelink],
                add_creature_types: vec![CreatureType::Performer],
                ..Default::default()
            },
        )
    }
}

/// Emet-Selch of the Third Seat — graveyard spells cost {2} less; once a
/// turn, opponents losing life let you cast an instant or sorcery from your
/// graveyard, exiled after.
pub fn emet_selch_of_the_third_seat() -> CardDefinition {
    legendary(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells you cast from your graveyard cost {2} less to cast.",
            effect: StaticEffect::GraveyardCastCostReduction { amount: 2 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_batch: true,
                ..EventSpec::new(EventKind::LifeLost, EventScope::OpponentControl).once_per_turn()
            },
            // The cast is optional on its own: a paid cast the controller
            // declines (or can't afford) is skipped.
            effect: Effect::CastWithoutPayingImmediate {
                what: target_filtered(instant_or_sorcery().and(R::InYourGraveyard)),
                source_zone: Zone::Graveyard,
                exile_after: true,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: true,
            },
        }],
        ..creature(
            "Emet-Selch of the Third Seat",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Elder, CreatureType::Wizard],
            3,
            4,
        )
    })
}

/// Estinien Varlineau — noncreature spells grow it and give it flying; at
/// your second main phase, draw and lose life per opponent it or a Dragon
/// damaged in combat.
///
/// ⚠ Residual: counts opponents dealt combat damage by any creature.
pub fn estinien_varlineau() -> CardDefinition {
    let x = || Value::PlayersDealtCombatDamageThisTurn(PlayerRef::EachOpponent);
    legendary(CardDefinition {
        triggered_abilities: vec![
            on_cast_noncreature(Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::YourControl),
                effect: Effect::Seq(vec![draw(x()), Effect::LoseLife { who: Selector::You, amount: x() }]),
            },
        ],
        ..creature(
            "Estinien Varlineau",
            cost(&[generic(2), w(), b()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            3,
            3,
        )
    })
}

/// Eye of Nidhogg — enchanted creature is a black 4/2 Dragon with flying
/// and deathtouch, and is goaded; it returns to hand from the graveyard.
pub fn eye_of_nidhogg() -> CardDefinition {
    CardDefinition {
        name: "Eye of Nidhogg",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((4, 2)),
            set_colors: Some(vec![Color::Black]),
            add_creature_types: vec![CreatureType::Dragon],
            keywords: vec![Keyword::Flying, Keyword::Deathtouch],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature is goaded.",
            effect: StaticEffect::AttachedIsGoaded,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
        }],
        ..Default::default()
    }
}

/// Fandaniel, Telophoroi Ascian — instants and sorceries surveil 1; at your
/// end step each opponent sacrifices a nontoken creature or loses 2 life per
/// instant and sorcery in your graveyard.
pub fn fandaniel_telophoroi_ascian() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(instant_or_sorcery())),
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::ForEach {
                    selector: Selector::Player(PlayerRef::EachOpponent),
                    body: Box::new(Effect::Punisher {
                        chooser: Selector::Player(PlayerRef::Triggerer),
                        options: vec![Effect::Sacrifice {
                            who: Selector::Player(PlayerRef::Triggerer),
                            count: Value::ONE,
                            filter: R::Creature.and(R::NotToken),
                        }],
                        otherwise: Box::new(Effect::LoseLife {
                            who: Selector::Player(PlayerRef::Triggerer),
                            amount: Value::Times(
                                Box::new(Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: instant_or_sorcery() }),
                                Box::new(Value::Const(2)),
                            ),
                        }),
                    }),
                },
            },
        ],
        ..creature(
            "Fandaniel, Telophoroi Ascian",
            cost(&[generic(4), b()]),
            vec![CreatureType::Elder, CreatureType::Wizard],
            4,
            5,
        )
    })
}

/// G'raha Tia, Scion Reborn — lifelink; once a turn, a noncreature spell lets
/// you pay life equal to its mana value for a Hero with that many counters.
pub fn graha_tia_scion_reborn() -> CardDefinition {
    let mv = || Value::ManaValueOf(Box::new(Selector::TriggerSource));
    legendary(CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Noncreature))
                .once_per_turn(),
            effect: Effect::MayPayLife {
                description: "Pay life equal to its mana value for a Hero?".into(),
                amount: mv(),
                body: Box::new(Effect::Seq(vec![
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: hero() },
                    Effect::AddCounter { what: Selector::LastCreatedToken, kind: CounterType::PlusOnePlusOne, amount: mv() },
                ])),
                else_: None,
            },
        }],
        ..creature(
            "G'raha Tia, Scion Reborn",
            cost(&[w(), u(), b()]),
            vec![CreatureType::Cat, CreatureType::Wizard],
            2,
            3,
        )
    })
}

/// Hermes, Overseer of Elpis — noncreature spells make 1/1 flying vigilant
/// Birds; attacking with Birds scries 2.
pub fn hermes_overseer_of_elpis() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![
            on_cast_noncreature(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: token("Bird", vec![Color::Blue], vec![CreatureType::Bird], 1, 1, vec![Keyword::Flying, Keyword::Vigilance]),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                    Predicate::AttackedWithCreatureMatching {
                        who: PlayerRef::You,
                        filter: R::HasCreatureType(CreatureType::Bird),
                    },
                ),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            },
        ],
        ..creature(
            "Hermes, Overseer of Elpis",
            cost(&[generic(3), u()]),
            vec![CreatureType::Elder, CreatureType::Wizard],
            2,
            4,
        )
    })
}

/// Hildibrand Manderville // Gentleman's Rise — creature tokens you control
/// get +1/+1; the Adventure makes a 2/2 Zombie.
///
/// ⚠ Residual: dying doesn't let you cast it from the graveyard as an
/// Adventure.
pub fn hildibrand_manderville() -> CardDefinition {
    legendary(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: yours(R::Creature.and(R::IsToken)), power: 1, toughness: 1 },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Gentleman's Rise",
            cost: cost(&[generic(2), b()]),
            card_types: vec![CardType::Instant],
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: token("Zombie", vec![Color::Black], vec![CreatureType::Zombie], 2, 2, vec![]),
            },
        })),
        ..creature(
            "Hildibrand Manderville",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Detective],
            2,
            2,
        )
    })
}

/// Hraesvelgr of the First Brood — flying, vigilance, ward {2}; entering and
/// on each noncreature spell, a creature gets +1/+0 and can't be blocked.
pub fn hraesvelgr_of_the_first_brood() -> CardDefinition {
    let aid = || {
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
        ])
    };
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![etb(aid()), on_cast_noncreature(aid())],
        ..creature(
            "Hraesvelgr of the First Brood",
            cost(&[generic(4), u()]),
            vec![CreatureType::Elder, CreatureType::Dragon],
            5,
            5,
        )
    })
}

/// Idyllic Beachfront — Land — Plains Island, enters tapped.
pub fn idyllic_beachfront() -> CardDefinition {
    CardDefinition {
        name: "Idyllic Beachfront",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Plains, LandType::Island], ..Default::default() },
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![tap_add(Color::White), tap_add(Color::Blue)],
        ..Default::default()
    }
}

/// Into the Story — costs {3} less if an opponent has seven or more cards
/// in their graveyard; draw four.
pub fn into_the_story() -> CardDefinition {
    CardDefinition {
        name: "Into the Story",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {3} less to cast if an opponent has seven or more cards in their graveyard.",
            effect: StaticEffect::SelfCostReducedIfPredicate {
                amount: 3,
                condition: Predicate::ValueAtLeast(
                    Value::GreatestGraveyardSizeAmong(PlayerRef::EachOpponent),
                    Value::Const(7),
                ),
            },
        }],
        effect: draw(Value::Const(4)),
        ..Default::default()
    }
}

/// Krile Baldesion — lifelink; once a turn, a noncreature spell returns a
/// creature card of the same mana value from your graveyard.
pub fn krile_baldesion() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Noncreature))
                .once_per_turn(),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::ManaValueEqualsTriggerAmount).and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..creature("Krile Baldesion", cost(&[w(), u()]), vec![CreatureType::Dwarf, CreatureType::Wizard], 2, 1)
    })
}

/// Lyse Hext — prowess; noncreature spells cost {1} less; double strike
/// after two noncreature spells this turn.
pub fn lyse_hext() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Prowess],
        static_abilities: vec![
            StaticAbility {
                description: "Noncreature spells you cast cost {1} less to cast.",
                effect: StaticEffect::CostReduction { filter: R::Noncreature, amount: 1 },
            },
            StaticAbility {
                description: "As long as you've cast two or more noncreature spells this turn, Lyse Hext has double strike.",
                effect: StaticEffect::SelfHasKeywordWhilePredicate {
                    keyword: Keyword::DoubleStrike,
                    condition: Predicate::NoncreatureSpellsCastThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(2) },
                },
            },
        ],
        ..creature(
            "Lyse Hext",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Monk],
            2,
            2,
        )
    })
}

/// Observed Stasis — flash; enchant an opponent's creature: it leaves combat,
/// you draw per tapped creature its controller has, and it loses all
/// abilities and can't attack or block.
pub fn observed_stasis() -> CardDefinition {
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Observed Stasis",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
        equipped_bonus: Some(EquipBonus {
            remove_abilities: true,
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::RemoveFromCombat { what: enchanted() },
            draw(Value::PermanentCountControlledByMatching(
                PlayerRef::ControllerOf(Box::new(enchanted())),
                R::Creature.and(R::Tapped),
            )),
        ]))],
        ..Default::default()
    }
}

/// Papalymo Totolymo — noncreature spells drain 1; {4}, {T}, sacrifice it:
/// each opponent who lost life this turn sacrifices their biggest creature.
pub fn papalymo_totolymo() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![on_cast_noncreature(Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachOpponent),
                body: Box::new(Effect::If {
                    cond: Predicate::PlayerLostLifeThisTurn { who: PlayerRef::Triggerer },
                    then: Box::new(Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::Triggerer),
                        count: Value::ONE,
                        filter: R::Creature.and(R::HasGreatestPowerAmongControlled(Box::new(R::Creature))),
                    }),
                    else_: Box::new(Effect::Noop),
                }),
            },
            ..Default::default()
        }],
        ..creature("Papalymo Totolymo", cost(&[w(), b()]), vec![CreatureType::Dwarf, CreatureType::Wizard], 1, 2)
    })
}

/// Reaper's Scythe — job select; a soul counter per player who lost life
/// each end step; equipped creature gets +1/+1 per soul counter and is an
/// Assassin; equip {2}.
pub fn reapers_scythe() -> CardDefinition {
    let mut def = equipment(
        "Reaper's Scythe",
        cost(&[generic(2), b()]),
        cost(&[generic(2)]),
        EquipBonus {
            add_creature_types: vec![CreatureType::Assassin],
            scale: Some(EquipScale {
                filter: R::Any,
                per_power: 1,
                per_toughness: 1,
                count_self_counters: Some(CounterType::Soul),
                ..Default::default()
            }),
            ..Default::default()
        },
    );
    def.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
        effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Soul, amount: Value::PlayersWhoLostLifeThisTurn },
    });
    def
}

/// Summon: Good King Mog XII — flying, lifelink Saga creature. I: two 1/2
/// lifelink Moogles. II, III: noncreature spells this turn copy a non-Saga
/// token you control. IV: two +1/+1 counters on each other Moogle.
pub fn summon_good_king_mog_xii() -> CardDefinition {
    let copy_a_token = || Effect::OnEachSpellCastThisTurn {
        body: Box::new(Effect::If {
            cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Noncreature },
            then: Box::new(Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::Take {
                    inner: Box::new(yours(R::IsToken.and(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga).negate()))),
                    count: Box::new(Value::ONE),
                },
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            }),
            else_: Box::new(Effect::Noop),
        }),
    };
    CardDefinition {
        name: "Summon: Good King Mog XII",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Moogle],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        saga_chapters: vec![
            (
                1,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: token("Moogle", vec![Color::White], vec![CreatureType::Moogle], 1, 2, vec![Keyword::Lifelink]),
                },
            ),
            (2, copy_a_token()),
            (3, copy_a_token()),
            (
                4,
                Effect::AddCounter {
                    what: yours(R::HasCreatureType(CreatureType::Moogle).and(R::OtherThanSource)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Tataru Taru — entering, you draw and an opponent may draw; once a turn,
/// an opponent drawing outside their turn makes you a tapped Treasure.
pub fn tataru_taru() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                draw(Value::ONE),
                Effect::MayDoBy {
                    who: PlayerRef::Target(0),
                    description: "Draw a card?".into(),
                    body: Box::new(Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE }),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl)
                    .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::Triggerer))))
                    .once_per_turn(),
                effect: Effect::Seq(vec![
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(treasure_token()) },
                    Effect::Tap { what: Selector::LastCreatedTokens },
                ]),
            },
        ],
        ..creature("Tataru Taru", cost(&[generic(1), w()]), vec![CreatureType::Dwarf, CreatureType::Advisor], 0, 3)
    })
}

/// Thancred Waters — flash; entering, another legendary permanent of yours
/// is indestructible while Thancred stays; noncreature spells make Thancred
/// indestructible this turn.
pub fn thancred_waters() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            etb(Effect::GrantKeyword {
                what: target_filtered(R::Permanent.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: Keyword::Indestructible,
                duration: Duration::WhileSourceOnBattlefield,
            }),
            on_cast_noncreature(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            }),
        ],
        ..creature("Thancred Waters", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 5)
    })
}

/// Transpose — loot, lose 1 life; cast from hand, a 0/1 Wizard that pings
/// on your noncreature spells. Rebound.
pub fn transpose() -> CardDefinition {
    let wizard = TokenDefinition {
        triggered_abilities: vec![on_cast_noncreature(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
        })],
        ..(*token("Wizard", vec![Color::Black], vec![CreatureType::Wizard], 0, 1, vec![])).clone()
    };
    CardDefinition {
        name: "Transpose",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Rebound],
        effect: Effect::Seq(vec![
            draw(Value::ONE),
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            Effect::If {
                cond: Predicate::CastFromHand,
                then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(wizard) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Urianger Augurelt — casting from exile gains 2 life; {T}: you may exile
/// your top card; {T}: this turn you may play the cards exiled with it.
///
/// ⚠ Residual: the card is exiled face up; a land played from exile gains no
/// life; its spells get no {2} discount.
pub fn urianger_augurelt() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::CastSpellFromExile),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::MayDo {
                    description: "Exile the top card of your library?".into(),
                    body: Box::new(Effect::ExileLinked {
                        what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                    }),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::GrantMayPlay {
                    what: Selector::CardExiledWithSource,
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
                ..Default::default()
            },
        ],
        ..creature("Urianger Augurelt", cost(&[w(), u()]), vec![CreatureType::Elf, CreatureType::Advisor], 1, 3)
    })
}
