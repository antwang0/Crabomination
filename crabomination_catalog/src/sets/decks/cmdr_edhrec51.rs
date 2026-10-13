//! Commander: the 2026-10-13 rescan's gaps for The Fourteenth Doctor + Clara
//! Oswald. Tests in `tests/recent_b/cmdr_edhrec51.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EntersAsCopy, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
    WardCost,
};
use crate::effect::shortcut::etb;
use crate::effect::{Duration, Effect, PlayerRef};
use crate::mana::{cost, generic, hybrid, r, u, w, Color, ManaCost};

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn doctor() -> R {
    R::HasCreatureType(CreatureType::Doctor)
}

/// The Fourteenth Doctor — {R/G}{W}{U} legendary 3/4 Time Lord Doctor. Cast:
/// reveal the top fourteen, Doctors to your graveyard, the rest to the bottom
/// at random. It may enter as a copy of a Doctor card put into your graveyard
/// from your library this turn, gaining haste this turn.
pub fn the_fourteenth_doctor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::RevealTopMatchingToGraveyard { who: PlayerRef::You, count: Value::Const(14), filter: doctor() },
        }],
        enters_as_copy: Some(EntersAsCopy {
            filter: doctor().and(R::InYourGraveyard).and(R::PutIntoGraveyardFromLibraryThisTurn),
            from_graveyards: true,
            extra_triggered: vec![etb(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            })],
            ..Default::default()
        }),
        ..legend(
            "The Fourteenth Doctor",
            cost(&[hybrid(Color::Red, Color::Green), w(), u()]),
            vec![CreatureType::TimeLord, CreatureType::Doctor],
            3,
            4,
        )
    }
}

/// Rose Noble — {3}{U} legendary 2/3 Human, ward {2}. Casting a Doctor spell
/// or a creature spell with doctor's companion draws a card. Doctor's
/// companion.
pub fn rose_noble() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))), Keyword::DoctorsCompanion],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::CastSpellMatches(
                doctor().or(R::Creature.and(R::HasKeyword(Keyword::DoctorsCompanion))),
            )),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legend("Rose Noble", cost(&[generic(3), u()]), vec![CreatureType::Human], 2, 3)
    }
}

/// The Fifteenth Doctor — {2}{U}{R} legendary 3/3 Time Lord Doctor. Entering
/// or attacking: mill three, then an artifact card with mana value 2 or 3 from
/// among them may go to hand. Your first spell each turn, if nonartifact, has
/// improvise.
pub fn the_fifteenth_doctor() -> CardDefinition {
    let mill = || Effect::MillThenToHand {
        amount: Value::Const(3),
        filter: R::Artifact.and(R::ManaValueAtLeast(2)).and(R::ManaValueAtMost(3)),
        otherwise: None,
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(mill()),
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: mill() },
        ],
        static_abilities: vec![StaticAbility {
            description: "The first nonartifact spell you cast each turn has improvise.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::Not(Box::new(Predicate::SpellsCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::ONE,
                })),
                inner: Box::new(StaticEffect::GrantImproviseToSpells { filter: R::Not(Box::new(R::Artifact)) }),
            },
        }],
        ..legend(
            "The Fifteenth Doctor",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::TimeLord, CreatureType::Doctor],
            3,
            3,
        )
    }
}
