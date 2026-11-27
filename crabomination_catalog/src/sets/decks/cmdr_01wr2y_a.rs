//! Commander: Radha, Coalition Warlord's EDHREC average deck (scan of
//! 2026-11-27) — Radha, Bramblesnap, Veteran's Voice, Encounter the Unknown.
//! Tests in `tests/recent_b/cmdr_01wr2y_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope,
    EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, Selector, Subtypes, Supertype,
    TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Value};
use crate::mana::{cost, g, generic, r};

/// Radha, Coalition Warlord — {2}{R}{G} legendary 3/3 Elf Warrior. Domain —
/// whenever it becomes tapped, another target creature you control gets +X/+X
/// until end of turn, X = basic land types among lands you control.
pub fn radha_coalition_warlord() -> CardDefinition {
    let x = || Value::DomainCount(PlayerRef::You);
    CardDefinition {
        name: "Radha, Coalition Warlord",
        cost: cost(&[generic(2), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Warrior], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                power: x(),
                toughness: x(),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Bramblesnap — {1}{G} 1/1 Elemental, trample. Tap an untapped creature you
/// control (itself included, summoning sick or not — not a {T} cost): it gets
/// +1/+1 until end of turn.
pub fn bramblesnap() -> CardDefinition {
    CardDefinition {
        name: "Bramblesnap",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::Creature.and(R::ControlledByYou), 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Veteran's Voice — {R} Aura (creature you control). Tap enchanted creature:
/// target creature other than it gets +2/+1 until end of turn (only while the
/// enchanted creature is untapped, which the tap cost already demands).
pub fn veterans_voice() -> CardDefinition {
    CardDefinition {
        name: "Veteran's Voice",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::IsHostOfSource, 1)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::IsHostOfSource)))),
                power: Value::Const(2),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Encounter the Unknown — {1}{R}{R} instant. Choose one — exile the top two
/// cards of your library, you may play them until the end of your next turn;
/// or 2 damage to each creature and each player.
pub fn encounter_the_unknown() -> CardDefinition {
    CardDefinition {
        name: "Encounter the Unknown",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(2),
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            Effect::Seq(vec![
                Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(2) },
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) },
            ]),
        ]),
        ..Default::default()
    }
}
