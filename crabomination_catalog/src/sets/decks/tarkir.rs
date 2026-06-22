//! Tarkir: Dragonstorm (TDM) — assorted non-Omen cards. The set's Omen Dragon
//! cycle lives in `decks::omen`; this module collects the straightforward
//! spells/creatures that ride existing primitives. Tracked in `DECK_FEATURES.md`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, Keyword,
    SelectionRequirement, Selector, Subtypes, Value,
};
use crate::effect::shortcut::{etb, mobilize, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef};
use crate::mana::{cost, g, generic, mono_hybrid, u, w, Color};

/// Sarkhan's Resolve — {1}{G} Instant. Choose one — target creature gets +3/+3
/// until end of turn; or destroy target creature with flying.
pub fn sarkhans_resolve() -> CardDefinition {
    CardDefinition {
        name: "Sarkhan's Resolve",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Dragonback Lancer — {3}{W} 3/3 Human Soldier with Flying and Mobilize 1.
pub fn dragonback_lancer() -> CardDefinition {
    CardDefinition {
        name: "Dragonback Lancer",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![mobilize(1)],
        ..Default::default()
    }
}

/// Sibsig Appraiser — {2}{U} 2/1 Zombie Advisor. ETB: look at the top two cards,
/// put one into your hand and the other into your graveyard.
pub fn sibsig_appraiser() -> CardDefinition {
    CardDefinition {
        name: "Sibsig Appraiser",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(2),
            rest_to_graveyard: true,
            pick_filter: None,
            take: None,
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Defibrillating Current — {2/R}{2/W}{2/B} Sorcery. Deal 4 damage to target
/// creature or planeswalker and gain 2 life.
pub fn defibrillating_current() -> CardDefinition {
    CardDefinition {
        name: "Defibrillating Current",
        cost: cost(&[
            mono_hybrid(2, Color::Red),
            mono_hybrid(2, Color::White),
            mono_hybrid(2, Color::Black),
        ]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(4),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Mardu Devotee — {W} 1/2 Human Scout. ETB: scry 2. `{1}: Add {R}, {W}, or
/// {B}. Activate only once each turn.`
pub fn mardu_devotee() -> CardDefinition {
    CardDefinition {
        name: "Mardu Devotee",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            once_per_turn: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Red, Color::White, Color::Black], Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
