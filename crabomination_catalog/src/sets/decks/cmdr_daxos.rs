//! Commander: the cards the **Call the Spirits** precon (C15, Daxos the
//! Returned) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_daxos.rs`.
//!
//! Residuals (each also on its card):
//! - **Righteous Confluence** — the three picks are offered as one choice of
//!   the four non-targeting combinations (Knights and life); the "exile target
//!   enchantment" mode is not offered.
//! - **Sandstone Oracle** — the chosen opponent is the one with the most cards
//!   in hand (the pick that draws the most).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, PlayerTally,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{myriad, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, b, cost, generic, w};
use crate::sets::{enters_tapped, etb_scry_one, tap_add};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
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

fn enchantment(name: &'static str, mana: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// An Aura that attaches to target creature on resolution.
fn creature_aura(name: &'static str, mana: crate::mana::ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, mana)
    }
}

fn token(name: &str, colors: Vec<Color>, t: CreatureType, p: i32, tough: i32, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: tough,
        keywords,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: vec![t], ..Default::default() },
        ..Default::default()
    })
}

/// Dawnglare Invoker — flying; {8}: tap all creatures target player controls.
pub fn dawnglare_invoker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: Effect::Tap {
                what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
            },
            ..Default::default()
        }],
        ..creature("Dawnglare Invoker", cost(&[generic(2), w()]), vec![CreatureType::Kor, CreatureType::Wizard], 2, 1)
    }
}

/// Daxos's Torment — constellation: it becomes a 5/5 flying, hasty Demon
/// creature in addition to its other types until end of turn.
pub fn daxoss_torment() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Enchantment },
            ),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(5),
                toughness: Value::Const(5),
                creature_types: vec![CreatureType::Demon],
                keywords: vec![Keyword::Flying, Keyword::Haste],
                duration: Duration::EndOfTurn,
            },
        }],
        ..enchantment("Daxos's Torment", cost(&[generic(3), b()]))
    }
}

/// Deadly Tempest — destroy all creatures; each player loses life equal to
/// the number of creatures they controlled that were destroyed this way.
pub fn deadly_tempest() -> CardDefinition {
    CardDefinition {
        name: "Deadly Tempest",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(R::Creature),
                body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
            },
            Effect::EachPlayerDoes {
                who: PlayerRef::EachPlayer,
                body: Box::new(Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::CreaturesDestroyedThisResolutionControlledBy(PlayerRef::You),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Fallen Ideal — enchanted creature has flying and "Sacrifice a creature:
/// this creature gets +2/+1 until end of turn"; returns to its owner's hand
/// when put into a graveyard from the battlefield.
pub fn fallen_ideal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        ..creature_aura(
            "Fallen Ideal",
            cost(&[generic(2), b()]),
            EquipBonus {
                keywords: vec![Keyword::Flying],
                activated_abilities: vec![ActivatedAbility {
                    sac_other_may_be_source: true,
                    sac_other_filter: Some((R::Creature, 1)),
                    effect: Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(2),
                        toughness: Value::Const(1),
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
    }
}

/// Grave Peril — when a nonblack creature enters, sacrifice this; if you do,
/// destroy that creature.
pub fn grave_peril() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black)))),
                },
            ),
            // "If you do": a second trigger after the first sacrifice finds
            // nothing to sacrifice and destroys nothing.
            effect: Effect::If {
                cond: Predicate::SourceOnBattlefield,
                then: Box::new(Effect::Seq(vec![
                    Effect::SacrificeSource,
                    Effect::Destroy { what: Selector::TriggerSource },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..enchantment("Grave Peril", cost(&[generic(1), b()]))
    }
}

/// Herald of the Host — flying, vigilance, myriad.
pub fn herald_of_the_host() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![myriad()],
        ..creature("Herald of the Host", cost(&[generic(3), w(), w()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Karlov of the Ghost Council — whenever you gain life, two +1/+1 counters;
/// {W}{B}, remove six +1/+1 counters: exile target creature.
pub fn karlov_of_the_ghost_council() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), b()]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 6)),
            effect: Effect::Exile { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature(
            "Karlov of the Ghost Council",
            cost(&[w(), b()]),
            vec![CreatureType::Spirit, CreatureType::Advisor],
            2,
            2,
        )
    }
}

/// Necromancer's Covenant — exile all creature cards from target player's
/// graveyard, then a 2/2 Zombie per card; Zombies you control have lifelink.
pub fn necromancers_covenant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0), filter: Some(R::Creature) },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::CountOf(Box::new(Selector::ExiledThisResolution { filter: R::Creature })),
                    definition: token("Zombie", vec![Color::Black], CreatureType::Zombie, 2, 2, vec![]),
                },
            ]),
        }],
        static_abilities: vec![StaticAbility {
            description: "Zombies you control have lifelink.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Zombie).and(R::ControlledByYou),
                ),
                keyword: Keyword::Lifelink,
            },
        }],
        ..enchantment("Necromancer's Covenant", cost(&[generic(3), w(), b(), b()]))
    }
}

/// New Benalia — enters tapped; scry 1 on entry; {T}: add {W}.
pub fn new_benalia() -> CardDefinition {
    CardDefinition {
        name: "New Benalia",
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        triggered_abilities: vec![etb_scry_one()],
        activated_abilities: vec![tap_add(Color::White)],
        ..Default::default()
    }
}

/// Oreskos Explorer — on entry, search for up to X Plains cards to hand,
/// where X is the number of players who control more lands than you.
pub fn oreskos_explorer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Repeat {
                count: Value::PlayersWithGreaterTally(PlayerTally::LandsControlled),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasLandType(LandType::Plains),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..creature("Oreskos Explorer", cost(&[generic(1), w()]), vec![CreatureType::Cat, CreatureType::Scout], 2, 2)
    }
}

/// Righteous Confluence — choose three, repeats allowed: a 2/2 vigilant
/// Knight, exile target enchantment, gain 5 life. Offered as one choice of
/// the non-targeting combinations; the exile mode is not offered (residual).
pub fn righteous_confluence() -> CardDefinition {
    let knights = |n: i32| Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: token("Knight", vec![Color::White], CreatureType::Knight, 2, 2, vec![Keyword::Vigilance]),
    };
    let life = |n: i32| Effect::GainLife { who: Selector::You, amount: Value::Const(n) };
    CardDefinition {
        name: "Righteous Confluence",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![0],
            modes: vec![
                knights(3),
                Effect::Seq(vec![knights(2), life(5)]),
                Effect::Seq(vec![knights(1), life(10)]),
                life(15),
            ],
        },
        ..Default::default()
    }
}

/// Sandstone Oracle — flying; on entry, choose an opponent: if they have more
/// cards in hand than you, draw the difference. The opponent is the one with
/// the most cards in hand.
pub fn sandstone_oracle() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Max(
                    Box::new(Value::Const(0)),
                    Box::new(Value::Diff(
                        Box::new(Value::HandSizeOf(PlayerRef::MostCardsInHand)),
                        Box::new(Value::HandSizeOf(PlayerRef::You)),
                    )),
                ),
            },
        }],
        ..creature("Sandstone Oracle", cost(&[generic(7)]), vec![CreatureType::Sphinx], 4, 4)
    }
}

/// Vivid Meadow — the white Vivid land.
pub fn vivid_meadow() -> CardDefinition {
    super::cmdr_aesi::vivid("Vivid Meadow", Color::White)
}

/// Vow of Duty — +2/+2, vigilance, can't attack you or your planeswalkers.
pub fn vow_of_duty() -> CardDefinition {
    creature_aura(
        "Vow of Duty",
        cost(&[generic(2), w()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Vigilance, Keyword::CantAttackAuraController],
            ..Default::default()
        },
    )
}

/// Vow of Malice — +2/+2, intimidate, can't attack you or your planeswalkers.
pub fn vow_of_malice() -> CardDefinition {
    creature_aura(
        "Vow of Malice",
        cost(&[generic(2), b()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Intimidate, Keyword::CantAttackAuraController],
            ..Default::default()
        },
    )
}
