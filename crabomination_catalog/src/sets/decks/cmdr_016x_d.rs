//! Commander: 2026-10-19 (`016Xh16H`) — Glissa, Herald of Predation's
//! incubators and the Chief of the Wilds / The Chief Warg Wolves. Tests in
//! `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ExtraManaKind, PlayerRef, Predicate, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid};
use crate::sets::{dual_land_untyped, enters_tapped};
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

fn incubate(n: Value) -> Effect {
    Effect::Incubate { who: PlayerRef::You, amount: n }
}

fn incubators_you_control() -> R {
    R::IsToken.and(R::HasName("Incubator".into())).and(R::ControlledByYou)
}

// ── Glissa, Herald of Predation ─────────────────────────────────────────────

/// Elvish Vatkeeper — {1}{B}{G} 3/3 Phyrexian Elf. ETB incubate 2; {5}:
/// transform target Incubator token you control and double its +1/+1
/// counters.
pub fn elvish_vatkeeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(incubate(Value::Const(2)))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: Effect::Seq(vec![
                Effect::Transform { what: target_filtered(incubators_you_control()) },
                Effect::DoubleCountersOnEach { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Elvish Vatkeeper",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Elf],
            3,
            3,
        )
    }
}

/// Blighted Burgeoning — {2}{G} Aura, enchant land. ETB incubate 2; the land
/// adds one more mana of any color when tapped for mana.
pub fn blighted_burgeoning() -> CardDefinition {
    CardDefinition {
        name: "Blighted Burgeoning",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        triggered_abilities: vec![etb(incubate(Value::Const(2)))],
        static_abilities: vec![StaticAbility {
            description: "Whenever enchanted land is tapped for mana, its controller adds an additional one mana of any color.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: true,
                filter: R::Land,
                extra: ExtraManaKind::AnyColor,
                while_monarch: false,
            },
        }],
        ..Default::default()
    }
}

/// Gift of Compleation — {1}{B} enchantment. ETB incubate 3; a Phyrexian
/// you control dying surveils 1.
pub fn gift_of_compleation() -> CardDefinition {
    CardDefinition {
        name: "Gift of Compleation",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(incubate(Value::Const(3))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Phyrexian),
                    },
                ),
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Glistening Dawn — {2}{G}{G} sorcery. Incubate X twice, X = lands you
/// control.
pub fn glistening_dawn() -> CardDefinition {
    let x = || Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByYou)));
    CardDefinition {
        name: "Glistening Dawn",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![incubate(x()), incubate(x())]),
        ..Default::default()
    }
}

/// Traumatic Revelation — {1}{B} sorcery. Target opponent reveals their hand;
/// you take a creature or battle card from it — with none taken, incubate 3.
pub fn traumatic_revelation() -> CardDefinition {
    CardDefinition {
        name: "Traumatic Revelation",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DiscardChosen {
                from: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: R::Creature.or(R::HasCardType(CardType::Battle)),
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::CardsDiscardedThisEffect, Value::ONE),
                then: Box::new(Effect::Noop),
                else_: Box::new(incubate(Value::Const(3))),
            },
        ]),
        ..Default::default()
    }
}

// ── Chief of the Wilds / The Chief Warg ─────────────────────────────────────

fn wolf() -> R {
    R::HasCreatureType(CreatureType::Wolf)
}

/// Chief of the Wilds — {2}{B}{G} 4/4 legendary menace Wolf. Another Wolf
/// entering under your control puts two +1/+1 counters on it; triggered
/// abilities of your other Wolves and your battles trigger an additional time.
pub fn chief_of_the_wilds() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: wolf() },
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "If a triggered ability of another Wolf you control triggers, that ability triggers an additional time.",
                effect: StaticEffect::DoubleControllerTriggersOfType { types: vec![CreatureType::Wolf], exclude_source: true },
            },
            StaticAbility {
                description: "If a triggered ability of a battle you control triggers, that ability triggers an additional time.",
                effect: StaticEffect::DoubleControllerTriggersMatching { filter: R::HasCardType(CardType::Battle), include_spells: false },
            },
        ],
        ..creature("Chief of the Wilds", cost(&[generic(2), b(), g()]), vec![CreatureType::Wolf], 4, 4)
    }
}

/// Chief Warg's Company — {1}{B}{G} 5/3 trample Wolf; can't attack unless you
/// control two or more other Wolves; each upkeep, a 2/2 green Wolf.
pub fn chief_wargs_company() -> CardDefinition {
    let alone = Predicate::Not(Box::new(Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(wolf().and(R::ControlledByYou).and(R::OtherThanSource)),
        n: Value::Const(2),
    }));
    let token = TokenDefinition {
        name: "Wolf".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This creature can't attack unless you control two or more other Wolves.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate { keyword: Keyword::CantAttack, condition: alone },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(token) },
        }],
        ..creature("Chief Warg's Company", cost(&[generic(1), b(), g()]), vec![CreatureType::Wolf], 5, 3)
    }
}

/// Duskwatch Hunter — {2}{B/G} 3/1 Wolf; can't be blocked by tokens; ETB a
/// +1/+1 counter on target creature.
pub fn duskwatch_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::IsToken))],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature("Duskwatch Hunter", cost(&[generic(2), hybrid(Color::Black, Color::Green)]), vec![CreatureType::Wolf], 3, 1)
    }
}

/// Nighthowl Pursuer — {B} 1/1 menace Wolf. Ferocious — attacking while you
/// control a power-4 creature, it gets +2/+2.
pub fn nighthowl_pursuer() -> CardDefinition {
    let mut t = on_attack(Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(2),
        toughness: Value::Const(2),
        duration: Duration::EndOfTurn,
    });
    t.event = t.event.with_filter(Predicate::FerociousActive { who: PlayerRef::You });
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![t],
        ..creature("Nighthowl Pursuer", cost(&[b()]), vec![CreatureType::Wolf], 1, 1)
    }
}

/// The Chief Warg — {2}{B}{G} 3/3 legendary menace Wolf. Ferocious — whenever
/// you attack while you control a power-4 creature, draw a card and lose 1.
pub fn the_chief_warg() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                .with_filter(Predicate::FerociousActive { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..creature("The Chief Warg", cost(&[generic(2), b(), g()]), vec![CreatureType::Wolf], 3, 3)
    }
}

/// Mirkwood — land, enters tapped; {T}: {B} or {G}. {2}{B}{G}, {T}, sacrifice
/// it: two +1/+1 counters on target Bear, Spider or Wolf you control. Sorcery
/// speed.
pub fn mirkwood() -> CardDefinition {
    let mut d = dual_land_untyped("Mirkwood", Color::Black, Color::Green, vec![]);
    d.static_abilities.push(enters_tapped());
    d.activated_abilities.push(ActivatedAbility {
        mana_cost: cost(&[generic(2), b(), g()]),
        tap_cost: true,
        sac_cost: true,
        sorcery_speed: true,
        effect: Effect::AddCounter {
            what: target_filtered(
                R::HasCreatureType(CreatureType::Bear)
                    .or(R::HasCreatureType(CreatureType::Spider))
                    .or(wolf())
                    .and(R::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        },
        ..Default::default()
    });
    d
}

/// Desolation Prowler — {1}{B} 2/2 Wolf. Pay 2 life: +2/+2 until end of
/// turn, once each turn.
pub fn desolation_prowler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            life_cost: 2,
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Desolation Prowler", cost(&[generic(1), b()]), vec![CreatureType::Wolf], 2, 2)
    }
}
