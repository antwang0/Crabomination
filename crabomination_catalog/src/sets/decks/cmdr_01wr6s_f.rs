//! Commander: closers for The Duras Sisters' and L'Rell, Fierce Visionary's
//! EDHREC average decks (scan of 2026-11-27). Tests in
//! `tests/recent_b/cmdr_01wr6s_a.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CreatureType,
    EnchantmentSubtype, EquipBonus, Keyword, LandType, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, Subtypes, TokenDefinition,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, PlayerRef, StaticEffect, Value};
use crate::mana::{Color, ManaCost, b, cost, generic, r};

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// A 2/1 red Klingon Warrior creature token with haste.
fn klingon_warrior() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Klingon Warrior".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Klingon, CreatureType::Warrior], ..Default::default() },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    })
}

/// Qo'noS — land, Planet. Enters tapped; gain 1 life; {T}: {B} or {R}.
pub fn qonos() -> CardDefinition {
    let mut d = crate::sets::tapland_untyped("Qo'noS", Color::Black, Color::Red, crate::sets::etb_gain_one());
    d.subtypes.land_types.push(LandType::Planet);
    d
}

/// Perils of the Past — {1}{R} instant. Choose one — target creature gets
/// +3/+2 until end of turn; or destroy target artifact.
pub fn perils_of_the_past() -> CardDefinition {
    spell(
        "Perils of the Past",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::Destroy { what: target_filtered(R::Artifact) },
        ]),
    )
}

/// Gin'tak Charge — {R} sorcery. 2 damage to any target.
pub fn gintak_charge() -> CardDefinition {
    spell(
        "Gin'tak Charge",
        cost(&[r()]),
        CardType::Sorcery,
        Effect::DealDamage { to: target_filtered(R::any_target()), amount: Value::Const(2) },
    )
}

/// The Destruction of Praxis — {3}{B}{B} sorcery. All creatures get -X/-X
/// until end of turn, X = your differently named lands; with X >= 6, two 2/1
/// red Klingon Warrior tokens with haste.
pub fn the_destruction_of_praxis() -> CardDefinition {
    spell(
        "The Destruction of Praxis",
        cost(&[generic(3), b(), b()]),
        CardType::Sorcery,
        Effect::WithX {
            x: Value::DistinctNamesControlledMatching(R::Land),
            body: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature),
                    power: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost)),
                    toughness: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost)),
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(6)),
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(2),
                        definition: klingon_warrior(),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        },
    )
}

/// Eject the Warp Core — {1}{B} instant. Kicker—sacrifice an artifact or
/// creature. Target creature gets -2/-2 until end of turn, -6/-6 if kicked.
pub fn eject_the_warp_core() -> CardDefinition {
    let shrink = |n: i32| Effect::PumpPT {
        what: target_filtered(R::Creature),
        power: Value::Const(-n),
        toughness: Value::Const(-n),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::SacrificePermanent { filter: R::Artifact.or(R::Creature), count: 1 }),
        ..spell(
            "Eject the Warp Core",
            cost(&[generic(1), b()]),
            CardType::Instant,
            Effect::If { cond: Predicate::SpellWasKicked, then: Box::new(shrink(6)), else_: Box::new(shrink(2)) },
        )
    }
}

/// The Wrath of Khan — {2}{R} Saga. I — 4 damage to target creature an
/// opponent controls; at the beginning of the next combat this turn, gain
/// control of it until end of turn, untap it, it gains haste. II — you may
/// exile a card from your hand; if you do, draw a card. III — each opponent
/// sacrifices a nontoken permanent.
pub fn the_wrath_of_khan() -> CardDefinition {
    let it = || Selector::Target(0);
    CardDefinition {
        name: "The Wrath of Khan",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::DealDamage {
                        to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                        amount: Value::Const(4),
                    },
                    Effect::DelayUntil {
                        kind: DelayedTriggerKind::NextCombatThisTurn,
                        body: Box::new(Effect::Seq(vec![
                            Effect::GainControl { what: it(), to: Some(PlayerRef::You), duration: Duration::EndOfTurn },
                            Effect::Untap { what: it(), up_to: None },
                            Effect::GrantKeyword { what: it(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                        ])),
                    },
                ]),
            ),
            (
                2,
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Hand,
                        filter: R::Any,
                    }),
                    then: Box::new(Effect::MayDo {
                        description: "Exile a card from your hand to draw a card?".into(),
                        body: Box::new(Effect::Seq(vec![
                            Effect::ExileChosenFromHand {
                                from: Selector::You,
                                count: Value::ONE,
                                filter: R::Any,
                                link_to_source: false,
                                face_down: false,
                            },
                            Effect::Draw { who: Selector::You, amount: Value::ONE },
                        ])),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ),
            (
                3,
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: R::Permanent.and(R::Not(Box::new(R::IsToken))),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Bat'leth — {2}{B} Equipment. ETB: target creature an opponent controls
/// gets -2/-2 until end of turn. Equipped creature gets +2/+0. Equip {2}.
pub fn batleth() -> CardDefinition {
    CardDefinition {
        name: "Bat'leth",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 0, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Savvy Profiteer — {1}{R} 2/1 Ferengi Rogue. {T}: {R}. {3}, {T}: a
/// Treasure token.
pub fn savvy_profiteer() -> CardDefinition {
    CardDefinition {
        name: "Savvy Profiteer",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ferengi, CreatureType::Rogue], ..Default::default() },
        power: 2,
        toughness: 1,
        activated_abilities: vec![
            crate::sets::tap_add(Color::Red),
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(crate::game::effects::treasure_token()),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Genesis Effect — {4}{R} instant. 6 damage to target creature; destroy
/// target land you control; a tapped New Planet land token ("{T}: one mana
/// of any color").
pub fn genesis_effect() -> CardDefinition {
    let planet = Arc::new(TokenDefinition {
        name: "New Planet".into(),
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Planet], ..Default::default() },
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        tapped: true,
        ..Default::default()
    });
    spell(
        "Genesis Effect",
        cost(&[generic(4), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(6) },
            Effect::Destroy {
                what: Selector::TargetFiltered { slot: 1, filter: R::Land.and(R::ControlledByYou) },
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: planet },
        ]),
    )
}

/// Red Alert! — {2}{R} instant; costs {2} less while a creature is attacking
/// you. Creatures you control get +2/+0 until end of turn.
pub fn red_alert() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This spell costs {2} less to cast if a creature is attacking you.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(R::Creature.and(R::IsAttackingYou))),
                amount: 2,
            },
        }],
        ..spell(
            "Red Alert!",
            cost(&[generic(2), r()]),
            CardType::Instant,
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        )
    }
}
