//! Commander: the 2026-10-16 (`claude/modern_decks`, third leg) rescan's
//! short-text gaps — Black Widow's vanilla bodies, X-23's two Marvel heroes,
//! The Howling Abomination. Tests in `tests/recent_b/cmdr_oct16_e.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{deal, each_opponent, etb, gain_life, target_filtered};
use crate::sets::tap_add_colorless;
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{ManaCost, b, cost, g, generic, r};

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

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn pump_self(p: i32, t: i32) -> Effect {
    Effect::PumpPT { what: Selector::This, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

// ── Black Widow, Natasha Romanoff (vanilla bodies) ──────────────────────────

/// Black Widow, Natasha Romanoff — {1}{R} legendary 2/2 Human Assassin Hero.
pub fn black_widow_natasha_romanoff() -> CardDefinition {
    legend(creature(
        "Black Widow, Natasha Romanoff",
        cost(&[generic(1), r()]),
        vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Hero],
        2,
        2,
    ))
}

/// Happy Hogan, Dauntless Driver — {R} legendary 2/1 Human Pilot.
pub fn happy_hogan_dauntless_driver() -> CardDefinition {
    legend(creature("Happy Hogan, Dauntless Driver", cost(&[r()]), vec![CreatureType::Human, CreatureType::Pilot], 2, 1))
}

/// Falkenrath Reaver — {1}{R} 2/2 Vampire.
pub fn falkenrath_reaver() -> CardDefinition {
    creature("Falkenrath Reaver", cost(&[generic(1), r()]), vec![CreatureType::Vampire], 2, 2)
}

/// Trained Orgg — {6}{R} 6/6 Orgg.
pub fn trained_orgg() -> CardDefinition {
    creature("Trained Orgg", cost(&[generic(6), r()]), vec![CreatureType::Orgg], 6, 6)
}

/// The Whizzer, Classic Speedster — {3}{R} legendary 3/3 Human Hero, first
/// strike, haste.
pub fn the_whizzer_classic_speedster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        ..legend(creature("The Whizzer, Classic Speedster", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Hero], 3, 3))
    }
}

// ── X-23's heroes ───────────────────────────────────────────────────────────

/// Storm, Shaker of Skies — {4}{G}{G} legendary 4/3 flying Mutant Hero. ETB:
/// destroy up to one target artifact, enchantment, or creature with flying.
pub fn storm_shaker_of_skies() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment).or(R::Creature.and(R::HasKeyword(Keyword::Flying)))),
            }),
        })],
        ..legend(creature(
            "Storm, Shaker of Skies",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Mutant, CreatureType::Hero],
            4,
            3,
        ))
    }
}

/// Wolfsbane, Highland Hero — {1}{G} legendary 2/2 trample Mutant Werewolf
/// Hero. {2}{G}: +2/+2 until end of turn, once each turn.
pub fn wolfsbane_highland_hero() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            once_per_turn: true,
            effect: pump_self(2, 2),
            ..Default::default()
        }],
        ..legend(creature(
            "Wolfsbane, Highland Hero",
            cost(&[generic(1), g()]),
            vec![CreatureType::Mutant, CreatureType::Werewolf, CreatureType::Hero],
            2,
            2,
        ))
    }
}

// ── The Howling Abomination ─────────────────────────────────────────────────

/// The Howling Abomination — {3}{R}{G} legendary 5/5 haste Human Beast
/// Warrior; trample while you've cast three or more spells this turn. When it
/// becomes the target of a spell: +2/+2 until end of turn and 2 damage to each
/// opponent.
pub fn the_howling_abomination() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "Has trample as long as you've cast three or more spells this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpellsCastThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(3) },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Trample],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource).caused_by(R::IsSpellOnStack),
            effect: Effect::Seq(vec![pump_self(2, 2), deal(2, each_opponent())]),
        }],
        ..legend(creature(
            "The Howling Abomination",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::Human, CreatureType::Beast, CreatureType::Warrior],
            5,
            5,
        ))
    }
}

// ── Radioactive Man ─────────────────────────────────────────────────────────

/// Radioactive Man — {4}{B} legendary 3/5 deathtouch Human Scientist Villain.
/// Combat damage to a player: that player loses half their life, rounded up.
pub fn radioactive_man() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LoseHalfLife { who: Selector::Player(PlayerRef::Target(0)), rounded_up: true },
        }],
        ..legend(creature(
            "Radioactive Man",
            cost(&[generic(4), b()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Villain],
            3,
            5,
        ))
    }
}

/// Hexhaven Dueling Arena — Land. {T}: Add {C}. {2},{T}, sorcery speed: a
/// creature that attacked this turn becomes prepared. {4},{T}: any creature
/// becomes prepared. (Only creatures with prepare spells can.)
pub fn hexhaven_dueling_arena() -> CardDefinition {
    let prepare = |mana: u32, filter: R, sorcery_speed: bool| ActivatedAbility {
        tap_cost: true,
        mana_cost: cost(&[generic(mana)]),
        sorcery_speed,
        effect: Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::HasPrepareSpell).and(filter)),
            kind: CounterType::Prepared,
            amount: Value::ONE,
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Hexhaven Dueling Arena",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_colorless(), prepare(2, R::AttackedThisTurn, true), prepare(4, R::Any, false)],
        ..Default::default()
    }
}

/// Rampart Hunter — {3}{B} 3/3 deathtouch Horror. ETB: target creature gets
/// +2/+2 and gains deathtouch until end of turn.
pub fn rampart_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
        ]))],
        ..creature("Rampart Hunter", cost(&[generic(3), b()]), vec![CreatureType::Horror], 3, 3)
    }
}

/// Extended Absence — {3}{B} instant. Exile target creature or planeswalker;
/// 1 damage to each opponent and you gain 1 life.
pub fn extended_absence() -> CardDefinition {
    CardDefinition {
        name: "Extended Absence",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Creature.or(R::Planeswalker)) },
            deal(1, each_opponent()),
            gain_life(1),
        ]),
        ..Default::default()
    }
}
