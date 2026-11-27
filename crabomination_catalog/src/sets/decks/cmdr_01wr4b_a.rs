//! Commander: closers for the colour-chooser pairs (Faceless One, The
//! Prismatic Piper) and three Doctor pairs (scan of 2026-11-27). Tests in
//! `tests/recent_b/cmdr_01wr4b_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Value, ZoneDest};
use crate::mana::{Color, ManaCost, cost, g, generic, r, u, w};

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

fn loot() -> Effect {
    Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::ONE },
        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
    ])
}

/// Blossom Dryad — {2}{G} 2/2 Dryad. {T}: untap target land.
pub fn blossom_dryad() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Untap { what: target_filtered(R::Land), up_to: None },
            ..Default::default()
        }],
        ..creature("Blossom Dryad", cost(&[generic(2), g()]), vec![CreatureType::Dryad], 2, 2)
    }
}

/// Scroll of Griselbrand — {1} artifact. {1}, sacrifice it: target opponent
/// discards a card; if you control a Demon, that player loses 3 life.
pub fn scroll_of_griselbrand() -> CardDefinition {
    CardDefinition {
        name: "Scroll of Griselbrand",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Discard { who: target_filtered(R::OpponentPlayer), amount: Value::ONE, random: false },
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::ControlledBy {
                        who: PlayerRef::You,
                        filter: R::HasCreatureType(CreatureType::Demon),
                    }),
                    then: Box::new(Effect::LoseLife { who: Selector::Target(0), amount: Value::Const(3) }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Channel Harm — {5}{W} instant. Prevent all damage sources you don't
/// control would deal to you and your permanents this turn; Channel Harm
/// deals what it prevents to target creature. (The "you may" is always
/// taken.)
pub fn channel_harm() -> CardDefinition {
    spell(
        "Channel Harm",
        cost(&[generic(5), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PreventOpposingDamageThisTurnReflecting {
                target: Selector::You,
                reflect_to: Some(target_filtered(R::Creature)),
            },
            Effect::PreventOpposingDamageThisTurnReflecting {
                target: Selector::EachPermanent(R::ControlledByYou),
                reflect_to: Some(Selector::Target(0)),
            },
        ]),
    )
}

/// Maximize Altitude — {U} sorcery. Target creature gets +1/+1 and gains
/// flying until end of turn. Jump-start.
pub fn maximize_altitude() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::JumpStart],
        ..spell(
            "Maximize Altitude",
            cost(&[u()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            ]),
        )
    }
}

/// Vantress Paladin — {3}{U} 2/2 flying Human Knight. Adamant: enters with a
/// +1/+1 counter if at least three blue mana was spent to cast it.
pub fn vantress_paladin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SourceCastWithColorSpent { color: Color::Blue, at_least: 3 }),
                then: Box::new(Value::ONE),
                else_: Box::new(Value::Const(0)),
            },
        )),
        ..creature("Vantress Paladin", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

/// Startling Development — {1}{U} instant. Until end of turn, target
/// creature becomes a blue Serpent with base power and toughness 4/4.
/// Cycling {1}.
pub fn startling_development() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(1)]))],
        ..spell(
            "Startling Development",
            cost(&[generic(1), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::BecomeCreature {
                    what: target_filtered(R::Creature),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    creature_types: vec![CreatureType::Serpent],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                Effect::BecomeColor {
                    what: Selector::Target(0),
                    colors: vec![Color::Blue],
                    duration: Duration::EndOfTurn,
                    additive: false,
                },
            ]),
        )
    }
}

/// Pattern Matcher — {4} 3/3 artifact Golem. ETB: you may search your
/// library for a card with the same name as another creature you control,
/// put it into your hand, then shuffle.
pub fn pattern_matcher() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for a card named like another creature you control?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::SameNameAsACreatureYouControl,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature("Pattern Matcher", cost(&[generic(4)]), vec![CreatureType::Golem], 3, 3)
    }
}

/// Tomebound Lich — {1}{U}{B} 1/3 deathtouch lifelink Zombie Wizard. When it
/// enters or deals combat damage to a player, draw a card, then discard one.
pub fn tomebound_lich() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![
            etb(loot()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: loot(),
            },
        ],
        ..creature(
            "Tomebound Lich",
            cost(&[generic(1), u(), crate::mana::b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Over the Top — {5}{R}{R} sorcery. Each player reveals cards off the top
/// of their library equal to the nonland permanents they control, puts the
/// permanent cards onto the battlefield and the rest into their graveyard.
pub fn over_the_top() -> CardDefinition {
    spell("Over the Top", cost(&[generic(5), r(), r()]), CardType::Sorcery, Effect::OverTheTop)
}

/// Alaborn Musketeer — {1}{W} 2/1 reach Human Soldier.
pub fn alaborn_musketeer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        ..creature("Alaborn Musketeer", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 1)
    }
}

/// Sparring Mummy — {3}{W} 3/3 Zombie. ETB: untap target creature.
pub fn sparring_mummy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Untap { what: target_filtered(R::Creature), up_to: None })],
        ..creature("Sparring Mummy", cost(&[generic(3), w()]), vec![CreatureType::Zombie], 3, 3)
    }
}

/// The Weakstone's Subjugation / Marooned shape: a {U} Aura for an artifact
/// or creature; ETB you may pay {3} to tap it; it doesn't untap in its
/// controller's untap step.
fn pay_to_tap_lock_aura(name: &'static str) -> CardDefinition {
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name,
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Artifact.or(R::Creature)) },
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {3} to tap the enchanted permanent?".into(),
            mana_cost: cost(&[generic(3)]),
            body: Box::new(Effect::Tap { what: enchanted() }),
            else_: None,
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted permanent doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap { applies_to: enchanted() },
        }],
        ..Default::default()
    }
}

/// Weakstone's Subjugation — {U} Aura (artifact or creature): pay {3} on
/// entry to tap it; it doesn't untap.
pub fn weakstones_subjugation() -> CardDefinition {
    pay_to_tap_lock_aura("Weakstone's Subjugation")
}

/// Marooned — Weakstone's Subjugation that also can't be beamed up (Scotty's
/// return cost skips the enchanted creature).
pub fn marooned() -> CardDefinition {
    pay_to_tap_lock_aura("Marooned")
}

/// Spellweaver Volute — {3}{U}{U} Aura: enchant instant card in a graveyard.
/// Whenever you cast a sorcery, copy the enchanted card; you may cast the
/// copy free, and if you do, exile the card and move to another instant
/// card in a graveyard. The enchanted card is a link (`Effect::
/// SpellweaverVolute`); with none to move to, the Aura goes to the graveyard.
pub fn spellweaver_volute() -> CardDefinition {
    CardDefinition {
        name: "Spellweaver Volute",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        cast_condition: Some(Predicate::SelectorExists(Selector::CardsInZone {
            who: PlayerRef::EachPlayer,
            zone: crate::card::Zone::Graveyard,
            filter: R::HasCardType(CardType::Instant),
        })),
        triggered_abilities: vec![
            etb(Effect::SpellweaverVolute { attach: true }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCardType(CardType::Sorcery) }),
                effect: Effect::SpellweaverVolute { attach: false },
            },
        ],
        ..Default::default()
    }
}
