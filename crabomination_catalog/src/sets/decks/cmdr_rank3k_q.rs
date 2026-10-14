//! Commander: two-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), seventeenth file. Tests in `tests/recent_b/cmdr_rank3k_q.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, Subtypes, Supertype,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{ManaCost, cost, generic, w};

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

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

// ── Rhys, the Evermore ───────────────────────────────────────────────────────

/// Rescue Girl, First Responder — {2}{W} legendary 1/3 flying Human Cleric.
/// {T}: return another target permanent you control to its owner's hand;
/// activate only during your turn.
pub fn rescue_girl_first_responder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Move {
                what: target_filtered(R::Permanent.and(R::ControlledByYou).and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            ..Default::default()
        }],
        ..legend(
            "Rescue Girl, First Responder",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            3,
        )
    }
}

/// Morningtide's Light — {3}{W} sorcery. Exile any number of target creatures;
/// at the beginning of the next end step return them to the battlefield
/// tapped under their owners' control. Until your next turn, prevent all
/// damage that would be dealt to you. Exile Morningtide's Light.
pub fn morningtides_light() -> CardDefinition {
    CardDefinition {
        name: "Morningtide's Light",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::ExileReturnToOwnerNextEndStep {
                    what: Selector::Target(0),
                    tapped: true,
                }),
            },
            Effect::PreventAllDamageToPlayerUntilTheirNextTurn {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}
