//! Commander: 2026-10-19 (`016Xh16H`) — Sidar Jabari's flanking knights and
//! Rakka Mar's Elementals. Tests in `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, SpendRestriction, cost, generic, r, w};
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

fn knight(name: &'static str, mana: ManaCost, keywords: Vec<Keyword>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        keywords,
        ..creature(name, mana, vec![CreatureType::Human, CreatureType::Knight], p, t)
    }
}

// ── Sidar Jabari ────────────────────────────────────────────────────────────

/// Benalish Cavalry — {1}{W} 2/2 flanking Human Knight.
pub fn benalish_cavalry() -> CardDefinition {
    knight("Benalish Cavalry", cost(&[generic(1), w()]), vec![Keyword::Flanking], 2, 2)
}

/// Cavalry Master — {2}{W}{W} 3/3 flanking; your other creatures with
/// flanking have another instance of it (each triggers separately, CR 702.25b).
pub fn cavalry_master() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control with flanking have flanking.",
            effect: StaticEffect::OtherFlankersHaveFlanking,
        }],
        ..knight("Cavalry Master", cost(&[generic(2), w(), w()]), vec![Keyword::Flanking], 3, 3)
    }
}

/// Gustcloak Cavalier — {3}{W}{W} 2/2 flanking. Attacking, you may tap target
/// creature; becoming blocked, you may untap it and remove it from combat.
pub fn gustcloak_cavalier() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::MayDo {
                description: "Tap target creature?".into(),
                body: Box::new(Effect::Tap { what: target_filtered(R::Creature) }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Untap and remove it from combat?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Untap { what: Selector::This, up_to: None },
                        Effect::RemoveFromCombat { what: Selector::This },
                    ])),
                },
            },
        ],
        ..knight("Gustcloak Cavalier", cost(&[generic(3), w(), w()]), vec![Keyword::Flanking], 2, 2)
    }
}

/// Knight of Sursi — {3}{W} 2/2 flying, flanking; suspend 3—{W}.
pub fn knight_of_sursi() -> CardDefinition {
    knight(
        "Knight of Sursi",
        cost(&[generic(3), w()]),
        vec![Keyword::Flying, Keyword::Flanking, Keyword::Suspend(3, cost(&[w()]))],
        2,
        2,
    )
}

/// Riftmarked Knight — {1}{W}{W} 2/2, protection from black, flanking;
/// suspend 3—{1}{W}{W}. Its last time counter coming off in exile makes a
/// 2/2 black Knight with flanking, protection from white and haste
/// (CR 702.62b — the trigger functions from exile).
pub fn riftmarked_knight() -> CardDefinition {
    let token = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flanking, Keyword::Protection(Color::White), Keyword::Haste],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![
            Keyword::Protection(Color::Black),
            Keyword::Flanking,
            Keyword::Suspend(3, cost(&[generic(1), w(), w()])),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterRemoved(CounterType::Time), EventScope::SelfSource)
                .while_suspended()
                .with_filter(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::WithCounter(CounterType::Time).negate(),
                }),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(token) },
        }],
        ..creature(
            "Riftmarked Knight",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Knight],
            2,
            2,
        )
    }
}

// ── Rakka Mar ───────────────────────────────────────────────────────────────

fn elemental() -> R {
    R::HasCreatureType(CreatureType::Elemental)
}

/// Chandra's Embercat — {1}{R} 2/2 Elemental Cat. {T}: {R} to spend on an
/// Elemental spell (the Chandra-planeswalker half isn't modelled).
pub fn chandras_embercat() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Red])),
                    SpendRestriction::CreatureOfType(CreatureType::Elemental),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Chandra's Embercat",
            cost(&[generic(1), r()]),
            vec![CreatureType::Elemental, CreatureType::Cat],
            2,
            2,
        )
    }
}

/// Lavakin Brawler — {3}{R} 2/4. Attacking, +1/+0 for each Elemental you
/// control.
pub fn lavakin_brawler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::count(Selector::EachPermanent(elemental().and(R::ControlledByYou))),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Lavakin Brawler",
            cost(&[generic(3), r()]),
            vec![CreatureType::Elemental, CreatureType::Warrior],
            2,
            4,
        )
    }
}

/// Rockslide Elemental — {2}{R} 1/1 first strike; another creature dying
/// may put a +1/+1 counter on it.
pub fn rockslide_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource },
            ),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on Rockslide Elemental?".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature("Rockslide Elemental", cost(&[generic(2), r()]), vec![CreatureType::Elemental], 1, 1)
    }
}

/// Scampering Scorcher — {3}{R} 1/1. ETB two 1/1 red Elementals; Elementals
/// you control gain haste until end of turn.
pub fn scampering_scorcher() -> CardDefinition {
    let token = TokenDefinition {
        name: "Elemental".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(token) },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(elemental().and(R::ControlledByYou)),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature("Scampering Scorcher", cost(&[generic(3), r()]), vec![CreatureType::Elemental], 1, 1)
    }
}

/// Thunderkin Awakener — {1}{R} 1/2 haste Elemental Shaman. Attacking,
/// return target Elemental creature card with lesser toughness from your
/// graveyard tapped and attacking; sacrifice it at the next end step.
pub fn thunderkin_awakener() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    elemental().and(R::Creature).and(R::ToughnessLessThanSource).from_your_graveyard(),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::JoinCombatAttacking { what: Selector::LastMoved },
            Effect::SacrificeAtNextEndStep { what: Selector::LastMoved },
        ]))],
        ..creature(
            "Thunderkin Awakener",
            cost(&[generic(1), r()]),
            vec![CreatureType::Elemental, CreatureType::Shaman],
            1,
            2,
        )
    }
}

/// Chandra, Novice Pyromancer — {3}{R}, loyalty 5. +1: Elementals you
/// control get +2/+0; −1: add {R}{R}; −2: 2 damage to any target.
pub fn chandra_novice_pyromancer() -> CardDefinition {
    CardDefinition {
        name: "Chandra, Novice Pyromancer",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Chandra], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(elemental().and(R::ControlledByYou)),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red, Color::Red]) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature.or(R::Planeswalker).or(R::Player)),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
