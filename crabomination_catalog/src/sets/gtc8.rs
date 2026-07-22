//! Gatecrash (GTC) wave 8: the Dimir Cipher package, a counter-tax, an edict,
//! graveyard hate, and Angelic Skirmisher's each-combat keyword grant. Tests in
//! `classic_sets/gtc`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_filtered};
use crate::effect::{Duration, PlayerRef, Predicate, Selector};
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color};
use crate::game::TurnStep;

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}
fn your_creatures() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::ControlledByYou))
}

// ── Dimir (Cipher) ────────────────────────────────────────────────────────────

/// Mental Vapors — {3}{B} Sorcery. Target player discards a card. Cipher.
pub fn mental_vapors() -> CardDefinition {
    CardDefinition {
        name: "Mental Vapors",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: target_filtered(R::Player),
                amount: Value::ONE,
                random: false,
            },
            Effect::Cipher,
        ]),
        ..Default::default()
    }
}

/// Call of the Nightwing — {2}{U}{B} Sorcery. Create a 1/1 blue-and-black Horror
/// with flying. Cipher.
pub fn call_of_the_nightwing() -> CardDefinition {
    CardDefinition {
        name: "Call of the Nightwing",
        cost: cost(&[generic(2), u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Horror".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Blue, Color::Black],
                    subtypes: creatures(vec![CreatureType::Horror]),
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                },
            },
            Effect::Cipher,
        ]),
        ..Default::default()
    }
}

/// Shadow Alley Denizen — {B} 1/1 Vampire Rogue. Whenever another black creature
/// you control enters, target creature gains intimidate until end of turn.
pub fn shadow_alley_denizen() -> CardDefinition {
    CardDefinition {
        name: "Shadow Alley Denizen",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Vampire, CreatureType::Rogue]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasColor(Color::Black)),
                }),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Intimidate,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

// ── Izzet / Simic tempo ───────────────────────────────────────────────────────

/// Spell Rupture — {1}{U} Instant. Counter target spell unless its controller
/// pays {X}, where X is the greatest power among creatures you control.
pub fn spell_rupture() -> CardDefinition {
    CardDefinition {
        name: "Spell Rupture",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(R::Any),
            mana_cost: cost(&[]),
            exile: false,
            extra_generic: Some(Value::PowerOf(Box::new(Selector::GreatestPowerYouControl))),
        },
        ..Default::default()
    }
}

// ── Boros ─────────────────────────────────────────────────────────────────────

/// Angelic Skirmisher — {4}{W}{W} 4/4 Angel. Flying. At the beginning of each
/// combat, choose first strike, vigilance, or lifelink; creatures you control
/// gain that ability until end of turn.
pub fn angelic_skirmisher() -> CardDefinition {
    let grant = |kw: Keyword| Effect::GrantKeyword {
        what: your_creatures(),
        keyword: kw,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Angelic Skirmisher",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Angel]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer),
            effect: Effect::ChooseMode(vec![
                grant(Keyword::FirstStrike),
                grant(Keyword::Vigilance),
                grant(Keyword::Lifelink),
            ]),
        }],
        ..Default::default()
    }
}

// ── Golgari / Rakdos utility ──────────────────────────────────────────────────

/// Serene Remembrance — {G} Sorcery. Shuffle up to three target cards from a
/// single graveyard into their owners' libraries. Modeled as a targeted
/// player's graveyard (up to three cards picked at resolution).
pub fn serene_remembrance() -> CardDefinition {
    CardDefinition {
        name: "Serene Remembrance",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ShuffleGraveyardCardsIntoLibrary {
            who: PlayerRef::Target(0),
            filter: R::Any,
            max: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Structural Collapse — {5}{R} Sorcery. Target player sacrifices an artifact
/// and a land of their choice, then takes 2 damage.
pub fn structural_collapse() -> CardDefinition {
    let sac = |filter: R| Effect::Sacrifice {
        who: Selector::Player(PlayerRef::Target(0)),
        count: Value::ONE,
        filter,
    };
    CardDefinition {
        name: "Structural Collapse",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            sac(R::Artifact),
            sac(R::Land),
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Coerced Confession — {4}{U/B} Sorcery. Target player mills four cards; you
/// draw a card for each creature card put into their graveyard this way.
pub fn coerced_confession() -> CardDefinition {
    CardDefinition {
        name: "Coerced Confession",
        cost: cost(&[generic(4), hybrid(Color::Blue, Color::Black)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::MillThenDrawPerType {
            who: target_filtered(R::Player),
            amount: Value::Const(4),
            filter: R::Creature,
        },
        ..Default::default()
    }
}
