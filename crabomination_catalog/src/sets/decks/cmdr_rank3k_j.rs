//! Commander: two-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), tenth file. Tests in `tests/recent_b/cmdr_rank3k_j.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{prowess, target_filtered};
use crate::effect::{Effect, PlayerRef, RevealMissDest, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, cost, generic, r, u};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
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

// ── Ruric Thar, Biomagus ─────────────────────────────────────────────────────

/// Ruric Thar, Biomagus — {4}{U}{U} legendary 4/6 flying Ogre Crab Wizard;
/// prowess, prowess. Whenever it becomes the target of a spell or ability an
/// opponent controls, draw a card.
pub fn ruric_thar_biomagus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Prowess],
        triggered_abilities: vec![
            prowess(),
            prowess(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::OpponentControl),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            },
        ],
        ..legend(
            "Ruric Thar, Biomagus",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Ogre, CreatureType::Crab, CreatureType::Wizard],
            4,
            6,
        )
    }
}

/// "Each player may discard their hand and draw seven cards."
fn each_player_may_wheel() -> Effect {
    Effect::EachPlayerDoes {
        who: PlayerRef::EachPlayer,
        body: Box::new(Effect::MayDo {
            description: "Discard your hand and draw seven cards?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(7),
                },
            ])),
        }),
    }
}

/// Variable Chaser // Arc of Fortune — {2}{U} 2/3 flying, prowess Human
/// Wizard that enters prepared. Arc of Fortune ({2}{U} sorcery): each player
/// may discard their hand and draw seven cards.
pub fn variable_chaser() -> CardDefinition {
    let arc = spell(
        "Arc of Fortune",
        cost(&[generic(2), u()]),
        CardType::Sorcery,
        each_player_may_wheel(),
    );
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Prowess],
        triggered_abilities: vec![prowess()],
        prepare_spell: Some(Arc::new(arc)),
        enters_with_counters: Some((CounterType::Prepared, Value::ONE)),
        ..creature(
            "Variable Chaser",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

// ── Headliner Scarlett ───────────────────────────────────────────────────────

/// Pyre Rhymer // Molten Tide — {1}{R}{R} 3/3 prowess Elemental Sorcerer that
/// enters prepared. Molten Tide ({R} instant): until end of turn, whenever you
/// tap a Mountain for mana, add an additional {R}.
pub fn pyre_rhymer() -> CardDefinition {
    let tide = spell(
        "Molten Tide",
        cost(&[r()]),
        CardType::Instant,
        Effect::ExtraManaOnLandTapThisTurn {
            land: LandType::Mountain,
            extra: Color::Red,
        },
    );
    CardDefinition {
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![prowess()],
        prepare_spell: Some(Arc::new(tide)),
        enters_with_counters: Some((CounterType::Prepared, Value::ONE)),
        ..creature(
            "Pyre Rhymer",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Sorcerer],
            3,
            3,
        )
    }
}

/// Identity Echo — {2}{R} enchantment. {3}{R}: exile target creature or
/// planeswalker you control; reveal until a creature or planeswalker card, put
/// it onto the battlefield and the rest on the bottom in a random order.
/// Sorcery speed.
pub fn identity_echo() -> CardDefinition {
    let kind = || R::Creature.or(R::Planeswalker);
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Exile {
                    what: target_filtered(kind().and(R::ControlledByYou)),
                },
                Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: kind(),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                    cap: Value::Const(1000),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::BottomRandom,
                },
            ]),
            ..Default::default()
        }],
        ..spell(
            "Identity Echo",
            cost(&[generic(2), r()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}
