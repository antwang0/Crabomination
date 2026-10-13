//! Commander: the 2026-10-13 rescan's gaps for Princess Yue and Etrata,
//! Deadly Fugitive. Tests in `tests/recent_b/cmdr_edhrec54.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EntersAsCopy, EventKind, EventScope, EventSpec, Keyword,
    Predicate, Prototype, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef};
use crate::mana::{b, cost, generic, u, ManaCost};

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

fn scry_two() -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ..Default::default()
    }
}

// ── Princess Yue ─────────────────────────────────────────────────────────────

/// Princess Yue — {2}{U} legendary 3/2 Human Noble Ally. {T}: scry 2. When she
/// dies, if she was a nonland creature, she returns tapped as a legendary land
/// named Moon that keeps her abilities and gains "{T}: Add {C}." The return
/// rides a back face, as the Ojer gods' does: a land can't die, so it doesn't
/// loop.
pub fn princess_yue() -> CardDefinition {
    let moon = CardDefinition {
        name: "Moon",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![scry_two(), crate::sets::tap_add_colorless()],
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        back_face: Some(Box::new(moon)),
        activated_abilities: vec![scry_two()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land.negate() },
            ),
            effect: Effect::ReturnSelfTransformedTappedToOwner,
        }],
        ..creature(
            "Princess Yue",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Ally],
            3,
            2,
        )
    }
}

/// Hulking Metamorph — {9} 7/7 artifact creature Shapeshifter; prototype
/// {2}{U}{U} 3/3 (CR 702.160). It may enter as a copy of an artifact or
/// creature you control, except it's an artifact creature too and keeps this
/// creature's own power and toughness.
pub fn hulking_metamorph() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        prototype: Some(Box::new(Prototype { cost: cost(&[generic(2), u(), u()]), power: 3, toughness: 3 })),
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Artifact.or(R::Creature).and(R::ControlledByYou),
            extra_card_types: vec![CardType::Artifact, CardType::Creature],
            keep_own_pt: true,
            ..Default::default()
        }),
        ..creature("Hulking Metamorph", cost(&[generic(9)]), vec![CreatureType::Shapeshifter], 7, 7)
    }
}

// ── Etrata, Deadly Fugitive ──────────────────────────────────────────────────

/// Guildsworn Prowler — {1}{B} 2/1 Tiefling Rogue Assassin, deathtouch. When it
/// dies, if it wasn't blocking, draw a card (CR 603.10 — a blocker that just
/// died still reads as blocking).
pub fn guildsworn_prowler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsBlocking.negate() },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Guildsworn Prowler",
            cost(&[generic(1), b()]),
            vec![CreatureType::Tiefling, CreatureType::Rogue, CreatureType::Assassin],
            2,
            1,
        )
    }
}

/// Hookblade Veteran — {U} 1/2 Human Assassin with flying during your turn.
pub fn hookblade_veteran() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has flying.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Flying }),
            },
        }],
        ..creature("Hookblade Veteran", cost(&[u()]), vec![CreatureType::Human, CreatureType::Assassin], 1, 2)
    }
}

/// Become Anonymous — {2}{U}{U} instant. Exile target nontoken creature you
/// own and the top two cards of your library in a face-down pile, shuffle it,
/// then cloak those cards tapped (CR 702.182).
pub fn become_anonymous() -> CardDefinition {
    CardDefinition {
        name: "Become Anonymous",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CloakPileWithTop {
            what: target_filtered(R::Creature.and(R::NotToken).and(R::OwnedByYou)),
            count: Value::Const(2),
        },
        ..Default::default()
    }
}
