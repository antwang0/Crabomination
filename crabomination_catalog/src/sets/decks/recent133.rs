//! A Wilds of Eldraine (WOE) wave: Rat tokens, Wicked/Cursed Roles, Food/Clue
//! artifacts, and Celebration payoffs. All ride existing primitives. Tests in
//! `crabomination/src/tests/recent133.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::game::effects::food_token;
use crate::mana::{b, cost, g, generic, r, w, Color};
use super::woe_roles::{cursed_role, wicked_role};

/// 1/1 black Rat token with "This token can't block."
fn rat_token() -> TokenDefinition {
    TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    }
}



// ── Black ─────────────────────────────────────────────────────────────────────

/// Rat Out — {B} Instant. Up to one target creature gets -1/-1 until end of
/// turn. Create a Rat token.
pub fn rat_out() -> CardDefinition {
    CardDefinition {
        name: "Rat Out",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                filter: R::Creature,
                effect: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                }),
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: rat_token() },
        ]),
        ..Default::default()
    }
}

/// Eriette's Whisper — {3}{B} Sorcery. Target opponent discards two cards.
/// Create a Wicked Role token attached to up to one target creature you control.
pub fn eriettes_whisper() -> CardDefinition {
    CardDefinition {
        name: "Eriette's Whisper",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
                random: false,
            },
            Effect::CreateTokenAttachedTo {
                target: target_filtered(R::Creature.and(R::ControlledByYou)),
                definition: wicked_role(),
            },
        ]),
        ..Default::default()
    }
}

// ── Red ───────────────────────────────────────────────────────────────────────

/// Witch's Mark — {1}{R} Sorcery. You may discard a card; if you do, draw two.
/// Create a Wicked Role token attached to up to one target creature you control.
pub fn witchs_mark() -> CardDefinition {
    CardDefinition {
        name: "Witch's Mark",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::MayDiscard {
                description: "Discard a card to draw two?".into(),
                count: Value::ONE,
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
                else_: None,
            },
            Effect::CreateTokenAttachedTo {
                target: target_filtered(R::Creature.and(R::ControlledByYou)),
                definition: wicked_role(),
            },
        ]),
        ..Default::default()
    }
}

/// Edgewall Pack — {3}{R} 3/3 Dog with menace. ETB: create a Rat token.
pub fn edgewall_pack() -> CardDefinition {
    CardDefinition {
        name: "Edgewall Pack",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: rat_token(),
        })],
        ..Default::default()
    }
}

/// Boundary Lands Ranger — {1}{R} 2/2 Human Ranger. At the beginning of combat
/// on your turn, if you control a creature with power 4 or greater, you may
/// discard a card; if you do, draw a card.
pub fn boundary_lands_ranger() -> CardDefinition {
    CardDefinition {
        name: "Boundary Lands Ranger",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ranger],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat), EventScope::ActivePlayer)
                .with_filter(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                    ),
                    n: Value::ONE,
                }),
            effect: Effect::MayDiscard {
                description: "Discard a card to draw a card?".into(),
                count: Value::ONE,
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

// ── Green ─────────────────────────────────────────────────────────────────────

/// Spider Food — {2}{G} Sorcery. Destroy up to one target artifact, enchantment,
/// or creature with flying. Create a Food token.
pub fn spider_food() -> CardDefinition {
    CardDefinition {
        name: "Spider Food",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                filter: R::Artifact
                    .or(R::Enchantment)
                    .or(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: food_token() },
        ]),
        ..Default::default()
    }
}

// ── White ─────────────────────────────────────────────────────────────────────

/// Cursed Courtier — {2}{W} 3/3 Human Noble with lifelink. ETB: create a Cursed
/// Role token attached to it (it becomes 1/1).
pub fn cursed_courtier() -> CardDefinition {
    CardDefinition {
        name: "Cursed Courtier",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::CreateTokenAttachedTo {
            target: Selector::This,
            definition: cursed_role(),
        })],
        ..Default::default()
    }
}

/// Dutiful Griffin — {3}{W}{W} 4/4 Griffin with flying. {2}{W}, Sacrifice two
/// enchantments: Return this card from your graveyard to your hand.
pub fn dutiful_griffin() -> CardDefinition {
    CardDefinition {
        name: "Dutiful Griffin",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Griffin], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            mana_cost: cost(&[generic(2), w()]),
            sac_other_filter: Some((R::Enchantment, 2)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tuinvale Guide — {3}{W} 2/3 Faerie Scout with flying. Celebration — gets
/// +1/+0 and has lifelink while two or more nonland permanents entered under
/// your control this turn.
pub fn tuinvale_guide() -> CardDefinition {
    CardDefinition {
        name: "Tuinvale Guide",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Celebration — Tuinvale Guide gets +1/+0 and has lifelink while two or more nonland permanents entered under your control this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::CelebrationActive { who: PlayerRef::You },
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::Lifelink],
            },
        }],
        ..Default::default()
    }
}

// ── Artifact ──────────────────────────────────────────────────────────────────

/// Candy Trail — {1} Artifact — Food Clue. ETB: scry 2. {2}, {T}, Sacrifice
/// this artifact: You gain 3 life and draw a card.
pub fn candy_trail() -> CardDefinition {
    CardDefinition {
        name: "Candy Trail",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Food, ArtifactSubtype::Clue],
            ..Default::default()
        },
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
