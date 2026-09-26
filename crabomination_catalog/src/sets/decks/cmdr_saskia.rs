//! Commander: the cards the **Open Hostility** precon (C16, Saskia the
//! Unyielding) needed beyond what the catalog had (Dauntless Escort landed with
//! Token Triumph first). Tests in
//! `tests/recent_b/cmdr_saskia.rs`.
//!
//! Residuals (each also on its card):
//! - **Saskia the Unyielding** — "choose a player" is the engine's most
//!   hostile opponent (the card allows any player, you included).
//! - **Brutal Hordechief** — its creatures-block ability makes each
//!   opponent's creature block if able; *how* they block is still their
//!   controller's choice, not yours.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, PlayerStaticTarget, Predicate, RevealMissDest, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, w, x};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn static_(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

/// Ankle Shanker — haste; whenever it attacks, creatures you control gain
/// first strike and deathtouch until end of turn.
pub fn ankle_shanker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantKeywords {
                what: yours(R::Creature),
                keywords: vec![Keyword::FirstStrike, Keyword::Deathtouch],
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Ankle Shanker",
            cost(&[generic(2), r(), w(), b()]),
            vec![CreatureType::Goblin, CreatureType::Berserker],
            2,
            2,
        )
    }
}

/// Brutal Hordechief — each attacking creature of yours drains its defending
/// player for 1; {3}{R/W}{R/W}: opponents' creatures block this turn if able.
/// Residual: the blocks themselves stay their controllers' choice.
pub fn brutal_hordechief() -> CardDefinition {
    let rw = || hybrid(Color::Red, Color::White);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), rw(), rw()]),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                keyword: Keyword::MustBlock,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Brutal Hordechief",
            cost(&[generic(3), b()]),
            vec![CreatureType::Orc, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Charging Cinderhorn — haste; at each player's end step, if no creatures
/// attacked this turn, a fury counter, then that many damage to that player.
pub fn charging_cinderhorn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                Predicate::Not(Box::new(Predicate::PlayerAttackedThisTurn { who: PlayerRef::ActivePlayer })),
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Fury, amount: Value::ONE },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Fury },
                },
            ]),
        }],
        ..creature(
            "Charging Cinderhorn",
            cost(&[generic(3), r()]),
            vec![CreatureType::Elemental, CreatureType::Ox],
            4,
            2,
        )
    }
}

/// Conqueror's Flail — +1/+1 per color among your permanents; while attached,
/// opponents can't cast spells during your turn. Equip {2}.
pub fn conquerors_flail() -> CardDefinition {
    let n = || Value::DistinctColorsAmong(Box::new(yours(R::Any)));
    CardDefinition {
        name: "Conqueror's Flail",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        static_abilities: vec![
            static_(
                "Equipped creature gets +1/+1 for each color among permanents you control.",
                StaticEffect::PumpPTByValue {
                    applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                    power: n(),
                    toughness: n(),
                },
            ),
            static_(
                "As long as Conqueror's Flail is attached to a creature, your opponents can't cast spells during your turn.",
                StaticEffect::OpponentsCantCastDuringYourTurnWhileAttached,
            ),
        ],
        ..Default::default()
    }
}

/// Den Protector — creatures with less power can't block it; megamorph
/// {1}{G}; turned face up, return target card from your graveyard to hand.
pub fn den_protector() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByPowerLess, Keyword::Megamorph(cost(&[generic(1), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(R::Any.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..creature("Den Protector", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 1)
    }
}

/// Divergent Transformations — undaunted; exile two target creatures, then
/// each one's controller reveals to a creature card and puts it onto the
/// battlefield, shuffling the rest in.
pub fn divergent_transformations() -> CardDefinition {
    let reveal = |slot: u8| Effect::RevealUntilFind {
        who: PlayerRef::ControllerOf(Box::new(Selector::Target(slot))),
        find: R::Creature,
        to: ZoneDest::Battlefield {
            controller: PlayerRef::ControllerOf(Box::new(Selector::Target(slot))),
            tapped: false,
        },
        cap: Value::Const(500),
        life_per_revealed: 0,
        miss_dest: RevealMissDest::ShuffleIntoLibrary,
    };
    CardDefinition {
        static_abilities: vec![static_(
            "Undaunted (This spell costs {1} less to cast for each opponent.)",
            StaticEffect::SelfCostReducedPerOpponent { per: 1 },
        )],
        ..spell(
            "Divergent Transformations",
            cost(&[generic(6), r()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Exile { what: Selector::TargetFiltered { slot: 0, filter: R::Creature } },
                Effect::Exile { what: Selector::TargetFiltered { slot: 1, filter: R::Creature } },
                reveal(0),
                reveal(1),
            ]),
        )
    }
}

/// Everlasting Torment — players can't gain life, damage can't be prevented,
/// and all damage is dealt as though its source had wither.
pub fn everlasting_torment() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            static_(
                "Players can't gain life.",
                StaticEffect::PlayerCannotGainLife { target: PlayerStaticTarget::EachPlayer },
            ),
            static_("Damage can't be prevented.", StaticEffect::DamageCantBePrevented),
            static_(
                "All damage is dealt as though its source had wither.",
                StaticEffect::AllDamageDealtAsThoughWither,
            ),
        ],
        ..spell(
            "Everlasting Torment",
            cost(&[generic(2), hybrid(Color::Black, Color::Red)]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

/// Lavalanche — X damage to target player or planeswalker and each creature
/// that player (or that planeswalker's controller) controls.
pub fn lavalanche() -> CardDefinition {
    spell(
        "Lavalanche",
        cost(&[x(), b(), r(), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Player.or(R::Planeswalker)), amount: Value::XFromCost },
            Effect::DealDamage {
                to: Selector::ControlledBy {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    filter: R::Creature,
                },
                amount: Value::XFromCost,
            },
        ]),
    )
}

/// Mirror Entity — changeling; {X}: until end of turn your creatures have base
/// power and toughness X/X and gain all creature types (a granted
/// Changeling, which type lords read).
pub fn mirror_entity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            effect: Effect::Seq(vec![
                Effect::SetBasePT {
                    what: yours(R::Creature),
                    power: Value::XFromCost,
                    toughness: Value::XFromCost,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: yours(R::Creature),
                    keyword: Keyword::Changeling,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Mirror Entity", cost(&[generic(2), w()]), vec![CreatureType::Shapeshifter], 1, 1)
    }
}

/// Primeval Protector — costs {1} less per creature your opponents control;
/// enters: a +1/+1 counter on each other creature you control.
pub fn primeval_protector() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_(
            "This spell costs {1} less to cast for each creature your opponents control.",
            StaticEffect::SelfCostReducedByValue {
                amount: Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)))),
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: yours(R::Creature.and(R::OtherThanSource)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Primeval Protector", cost(&[generic(10), g()]), vec![CreatureType::Avatar], 10, 10)
    }
}

/// Saskia the Unyielding — vigilance, haste; as it enters, choose a player;
/// whenever a creature you control deals combat damage to a player, it deals
/// that much damage to the chosen player.
pub fn saskia_the_unyielding() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Haste],
        as_enters_effect: Some(Effect::ChoosePlayerForSource { opponent: false }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::DealDamageFrom {
                source: Selector::TriggerSource,
                to: Selector::Player(PlayerRef::ChosenPlayerOfSource),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature(
            "Saskia the Unyielding",
            cost(&[b(), r(), g(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            4,
        )
    })
}

/// Stonehoof Chieftain — trample, indestructible; whenever another creature
/// you control attacks, it gains trample and indestructible until end of turn.
pub fn stonehoof_chieftain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Indestructible],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours),
            effect: Effect::GrantKeywords {
                what: Selector::TriggerSource,
                keywords: vec![Keyword::Trample, Keyword::Indestructible],
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Stonehoof Chieftain",
            cost(&[generic(7), g()]),
            vec![CreatureType::Centaur, CreatureType::Warrior],
            8,
            8,
        )
    }
}

/// Tana, the Bloodsower — trample; combat damage to a player makes that many
/// 1/1 green Saprolings. Partner.
pub fn tana_the_bloodsower() -> CardDefinition {
    let saproling = Arc::new(TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    });
    legendary(CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::TriggerEventAmount, definition: saproling },
        }],
        ..creature(
            "Tana, the Bloodsower",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            2,
            2,
        )
    })
}

/// Tymna the Weaver — lifelink; at each of your postcombat main phases, you
/// may pay X life, X = opponents dealt combat damage this turn, to draw X.
/// Partner.
pub fn tymna_the_weaver() -> CardDefinition {
    let n = || Value::PlayersDealtCombatDamageThisTurn(PlayerRef::EachOpponent);
    legendary(CardDefinition {
        keywords: vec![Keyword::Lifelink, Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(n(), Value::ONE)),
            effect: Effect::MayPayLife {
                description: "Pay life equal to the opponents dealt combat damage this turn to draw that many?".into(),
                amount: n(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: n() }),
                else_: None,
            },
        }],
        ..creature(
            "Tymna the Weaver",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    })
}

/// Wilderness Elemental — trample; power = nonbasic lands your opponents
/// control.
pub fn wilderness_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        dynamic_pt: Some(crate::card::DynamicPt::PermanentsOnBattlefieldMatching {
            base_p: 0,
            base_t: 3,
            filter: Box::new(
                R::Land.and(R::Not(Box::new(R::HasSupertype(Supertype::Basic)))).and(R::ControlledByOpponent),
            ),
        }),
        ..creature("Wilderness Elemental", cost(&[generic(1), r(), g()]), vec![CreatureType::Elemental], 0, 3)
    }
}
