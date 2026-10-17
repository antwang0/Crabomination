//! Commander: 2026-10-17 (`019Y6J4R`) closers, twelfth batch — each the one
//! missing card of an otherwise-buildable EDHREC average deck. Tests in
//! `tests/recent_b/cmdr_019y_l.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value, Zone,
};
use crate::effect::shortcut::{etb, evoke, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, u, w};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn team_pump(p: i32, t: i32) -> Effect {
    Effect::PumpPT {
        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
        power: Value::Const(p),
        toughness: Value::Const(t),
        duration: Duration::EndOfTurn,
    }
}

/// Struggle for Skemfar — {3}{G} sorcery. A +1/+1 counter on target creature
/// you control, then it fights up to one target creature you don't control.
/// Foretell {G}.
pub fn struggle_for_skemfar() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[g()])),
        ..spell(
            "Struggle for Skemfar",
            cost(&[generic(3), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Fight {
                        attacker: Selector::Target(0),
                        defender: Selector::TargetFiltered {
                            slot: 1,
                            filter: R::Creature.and(R::ControlledByYou.negate()),
                        },
                    }),
                },
            ]),
        )
    }
}

/// Shepherd of the Cosmos — {4}{W}{W} 3/3 flying Angel Warrior. ETB: return
/// target permanent card with mana value 2 or less from your graveyard to the
/// battlefield. Foretell {3}{W}.
pub fn shepherd_of_the_cosmos() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        foretell_cost: Some(cost(&[generic(3), w()])),
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::InGraveyard.and(R::OwnedByYou).and(R::Permanent).and(R::ManaValueAtMost(2)),
            ),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        ..creature(
            "Shepherd of the Cosmos",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Angel, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Infernal Sovereign — {4}{B}{B} 6/6 flying trample Demon. Skip your draw
/// step. Whenever you play a land or cast a spell, draw a card and lose 1 life.
pub fn infernal_sovereign() -> CardDefinition {
    let pay = || Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::ONE }]);
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Skip your draw step.",
            effect: StaticEffect::ControllerSkipsDrawStep,
        }],
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl), effect: pay() },
            TriggeredAbility { event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl), effect: pay() },
        ],
        ..creature("Infernal Sovereign", cost(&[generic(4), b(), b()]), vec![CreatureType::Demon], 6, 6)
    }
}

/// Demonic Collusion — {3}{B}{B} sorcery. Buyback—discard two cards. Search
/// your library for a card, put it into your hand.
pub fn demonic_collusion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Buyback(ManaCost::new(vec![]))],
        buyback_additional_cost: Some(AdditionalCastCost::Discard { count: 2, filter: None }),
        ..spell(
            "Demonic Collusion",
            cost(&[generic(3), b(), b()]),
            CardType::Sorcery,
            Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Hand(PlayerRef::You) },
        )
    }
}

/// You Hear Something on Watch — {1}{W} instant. Choose one: creatures you
/// control get +1/+1 until end of turn; or 5 damage to target attacking
/// creature.
pub fn you_hear_something_on_watch() -> CardDefinition {
    spell(
        "You Hear Something on Watch",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            team_pump(1, 1),
            Effect::DealDamage { to: target_filtered(R::Creature.and(R::IsAttacking)), amount: Value::Const(5) },
        ]),
    )
}

/// Sigil of Myrkul — {2}{B} enchantment. Beginning of combat on your turn:
/// mill a card. When you do, if four or more creature cards are in your
/// graveyard, a +1/+1 counter on target creature you control and it gains
/// deathtouch until end of turn.
pub fn sigil_of_myrkul() -> CardDefinition {
    CardDefinition {
        name: "Sigil of Myrkul",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountOf(Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::Creature,
                        })),
                        Value::Const(4),
                    ),
                    then: Box::new(Effect::ReflexiveTrigger {
                        body: Box::new(Effect::Seq(vec![
                            Effect::AddCounter {
                                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                                kind: CounterType::PlusOnePlusOne,
                                amount: Value::ONE,
                            },
                            Effect::GrantKeyword {
                                what: Selector::Target(0),
                                keyword: Keyword::Deathtouch,
                                duration: Duration::EndOfTurn,
                            },
                        ])),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Graven Lore — {3}{U}{U} snow instant. Scry X, where X is the {S} spent to
/// cast it, then draw three cards.
pub fn graven_lore() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        ..spell(
            "Graven Lore",
            cost(&[generic(3), u(), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Scry { who: PlayerRef::You, amount: Value::SnowManaSpentToCastSource },
                draw(3),
            ]),
        )
    }
}

/// Wistfulness — {3}{G/U}{G/U} 6/5 Elemental Incarnation. ETB if {G}{G} was
/// spent: exile target artifact or enchantment an opponent controls. ETB if
/// {U}{U} was spent: draw two, then discard one. Evoke {G/U}{G/U}.
pub fn wistfulness() -> CardDefinition {
    let gu = || hybrid(Color::Green, Color::Blue);
    let mut green = etb(Effect::Exile {
        what: target_filtered(R::Artifact.or(R::Enchantment).and(R::ControlledByOpponent)),
    });
    green.event = green.event.with_filter(Predicate::SourceCastWithColorSpent { color: Color::Green, at_least: 2 });
    let mut blue = etb(Effect::Seq(vec![
        draw(2),
        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
    ]));
    blue.event = blue.event.with_filter(Predicate::SourceCastWithColorSpent { color: Color::Blue, at_least: 2 });
    CardDefinition {
        alternative_cost: Some(evoke(cost(&[gu(), gu()]))),
        triggered_abilities: vec![green, blue],
        ..creature(
            "Wistfulness",
            cost(&[generic(3), gu(), gu()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            6,
            5,
        )
    }
}

/// Virtus's Maneuver — {2}{B} sorcery. For each player, friend or foe: each
/// friend returns a creature card from their graveyard to their hand; each foe
/// sacrifices a creature.
pub fn virtuss_maneuver() -> CardDefinition {
    spell(
        "Virtus's Maneuver",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::FriendOrFoe {
            friend: Box::new(Effect::MoveChosen {
                from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature },
                filter: None,
                count: Value::ONE,
                up_to: false,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            foe: Box::new(Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature }),
        },
    )
}

/// Prudent Fateseer // Peer Review — {1}{W/U}{W/U} 1/4 Dwarf Wizard; enters
/// prepared. Whenever you scry or surveil, creatures you control get +1/+0
/// until end of turn (once each turn). Peer Review ({2}{W/U} sorcery): a 2/2
/// colorless Wizard Soldier token named Cadet; surveil 1.
pub fn prudent_fateseer() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    let cadet = TokenDefinition {
        name: "Cadet".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wizard, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        enters_with_counters: Some((CounterType::Prepared, Value::ONE)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ScriedOrSurveiled, EventScope::YourControl).once_per_turn(),
            effect: team_pump(1, 0),
        }],
        prepare_spell: Some(Arc::new(CardDefinition {
            name: "Peer Review",
            cost: cost(&[generic(2), wu()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(cadet) },
                Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
            ]),
            ..Default::default()
        })),
        ..creature(
            "Prudent Fateseer",
            cost(&[generic(1), wu(), wu()]),
            vec![CreatureType::Dwarf, CreatureType::Wizard],
            1,
            4,
        )
    }
}

/// Lotus Ring — {3} indestructible Equipment. Equipped creature gets +3/+3,
/// has vigilance and "{T}, Sacrifice this creature: Add three mana of any one
/// color." Equip {3}.
pub fn lotus_ring() -> CardDefinition {
    CardDefinition {
        name: "Lotus Ring",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Indestructible, Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Vigilance],
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(3)) },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Feywild Trickster — {2}{U} 2/2 Gnome Warlock. Whenever you roll one or
/// more dice, create a 1/1 blue Faerie Dragon token with flying.
pub fn feywild_trickster() -> CardDefinition {
    let dragon = TokenDefinition {
        name: "Faerie Dragon".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie, CreatureType::Dragon], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::RolledDice, EventScope::YourControl),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(dragon) },
        }],
        ..creature("Feywild Trickster", cost(&[generic(2), u()]), vec![CreatureType::Gnome, CreatureType::Warlock], 2, 2)
    }
}

/// Infinite Hourglass — {4} artifact. Your upkeep: a time counter on it. All
/// creatures get +1/+0 for each time counter on it. {3}: remove a time
/// counter; any player may activate this, only during any upkeep step.
pub fn infinite_hourglass() -> CardDefinition {
    CardDefinition {
        name: "Infinite Hourglass",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Time, amount: Value::ONE },
        }],
        static_abilities: vec![StaticAbility {
            description: "All creatures get +1/+0 for each time counter on Infinite Hourglass.",
            effect: StaticEffect::PumpPTPerCounterOnSource {
                applies_to: Selector::EachPermanent(R::Creature),
                kind: CounterType::Time,
                per_power: 1,
                per_toughness: 0,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            any_player: true,
            condition: Some(Predicate::CurrentStepIs(TurnStep::Upkeep)),
            effect: Effect::RemoveCounter { what: Selector::This, kind: CounterType::Time, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}
