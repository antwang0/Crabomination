//! Ravnica batch 9: Dimir/Golgari/Azorius utility + a targeted land-animator.
//! Tests in `recent_b/recent_299`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, LandType,
    SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::card::{EventKind, EventScope, EventSpec};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef};
use crate::mana::{b, cost, g, generic, u, w, ManaCost};

// ── Golgari ─────────────────────────────────────────────────────────────────

/// Woodwraith Corrupter — {3}{B}{B}{G} 3/6 Elemental Horror. {1}{B}{G}, {T}:
/// Target Forest becomes a 4/4 Elemental Horror creature. It's still a land.
pub fn woodwraith_corrupter() -> CardDefinition {
    CardDefinition {
        name: "Woodwraith Corrupter",
        cost: cost(&[generic(3), b(), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 6,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), g()]),
            tap_cost: true,
            effect: Effect::BecomeCreature {
                what: target_filtered(R::HasLandType(LandType::Forest)),
                power: Value::Const(4),
                toughness: Value::Const(4),
                creature_types: vec![CreatureType::Elemental, CreatureType::Horror],
                keywords: vec![],
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bond of Agony — {X}{B} Sorcery. As an additional cost, pay X life. Each
/// other player loses X life.
pub fn bond_of_agony() -> CardDefinition {
    CardDefinition {
        name: "Bond of Agony",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        additional_cost_pay_x_life: true,
        effect: Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

// ── Azorius / Dimir ─────────────────────────────────────────────────────────

/// Enemy of the Guildpact — {4}{B} 4/2 Spirit with protection from multicolored.
pub fn enemy_of_the_guildpact() -> CardDefinition {
    CardDefinition {
        name: "Enemy of the Guildpact",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::ProtectionFromMulticolored],
        ..Default::default()
    }
}

/// Court Hussar — {2}{U} 1/3 Vedalken Knight with Vigilance. When it enters,
/// look at the top three cards of your library, put one into your hand, and the
/// rest on the bottom.
pub fn court_hussar() -> CardDefinition {
    CardDefinition {
        name: "Court Hussar",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: false,
                pick_filter: None,
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
            },
        }],
        ..Default::default()
    }
}

/// Overrule — {X}{W}{U} Instant. Counter target spell unless its controller
/// pays {X}. You gain X life.
pub fn overrule() -> CardDefinition {
    CardDefinition {
        name: "Overrule",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterUnlessPaid {
                what: Selector::Target(0),
                mana_cost: ManaCost::default(),
                exile: false,
                extra_generic: Some(Value::XFromCost),
            },
            Effect::GainLife { who: Selector::You, amount: Value::XFromCost },
        ]),
        ..Default::default()
    }
}
