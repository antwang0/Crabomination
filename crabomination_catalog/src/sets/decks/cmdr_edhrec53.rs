//! Commander: the 2026-10-13 rescan's gaps for The Earth King and The
//! Muckslinger Gang. Tests in `tests/recent_b/cmdr_edhrec53.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, Color, ManaCost};
use std::sync::Arc;

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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn forest() -> R {
    R::HasLandType(LandType::Forest)
}

// ── The Earth King ───────────────────────────────────────────────────────────

/// Roiling Canopy — land, enters tapped. Whenever a Forest you control enters,
/// if you control at least five other Forests, target creature you control
/// gets +3/+3 until end of turn. {T}: add {G}.
pub fn roiling_canopy() -> CardDefinition {
    // CR 603.4 — "five other Forests" besides the one entering: six in all.
    let enough = || {
        Predicate::ValueAtLeast(
            Value::count(Selector::ControlledBy { who: PlayerRef::You, filter: forest() }),
            Value::Const(6),
        )
    };
    CardDefinition {
        name: "Roiling Canopy",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::All(
                vec![Predicate::EntityMatches { what: Selector::TriggerSource, filter: forest() }, enough()],
            )),
            effect: Effect::If {
                cond: enough(),
                then: Box::new(Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![crate::sets::tap_add(Color::Green)],
        ..Default::default()
    }
}

/// Old-Growth Troll — {G}{G}{G} 4/4 Troll Warrior, trample. When it dies, if
/// it was a creature, it returns as an Aura enchanting a Forest you control:
/// the Forest has "{T}: add {G}{G}" and "{1}, {T}, sacrifice this land:
/// create a tapped 4/4 green Troll Warrior with trample."
pub fn old_growth_troll() -> CardDefinition {
    let forest_you_control = || forest().and(R::ControlledByYou);
    let troll_token = TokenDefinition {
        name: "Troll Warrior".into(),
        power: 4,
        toughness: 4,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Troll, CreatureType::Warrior], ..Default::default() },
        keywords: vec![Keyword::Trample],
        tapped: true,
        ..Default::default()
    };
    let grant = |ability: ActivatedAbility| StaticAbility {
        description: "Enchanted Forest has '{T}: Add {G}{G}' and '{1}, {T}, Sacrifice this land: Create a tapped \
                      4/4 green Troll Warrior creature token with trample.'",
        effect: StaticEffect::GrantActivatedAbility {
            applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            ability,
            condition: None,
        },
    };
    let aura = CardDefinition {
        name: "Old-Growth Troll",
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(forest_you_control()) },
        keywords: vec![Keyword::Trample],
        static_abilities: vec![
            grant(ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green, Color::Green]) },
                ..Default::default()
            }),
            grant(ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                sac_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(troll_token),
                },
                ..Default::default()
            }),
        ],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        back_face: Some(Box::new(aura)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::ReturnSelfTransformedAttachedTo {
                host: Selector::take(Selector::EachPermanent(forest_you_control()), Value::ONE),
            },
        }],
        ..creature("Old-Growth Troll", cost(&[g(), g(), g()]), vec![CreatureType::Troll, CreatureType::Warrior], 4, 4)
    }
}

/// Sandman, Shifting Scoundrel — {1}{G}{G} legendary */* Sand Elemental
/// Villain, P/T each the number of lands you control; can't be blocked by
/// power 2 or less. {3}{G}{G}: return this card and target land card from
/// your graveyard to the battlefield tapped.
pub fn sandman_shifting_scoundrel() -> CardDefinition {
    let lands = || Value::count(Selector::ControlledBy { who: PlayerRef::You, filter: R::Land });
    let tapped = || ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true };
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByPowerAtMost(2)],
        static_abilities: vec![StaticAbility {
            description: "Sandman's power and toughness are each equal to the number of lands you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: lands(), toughness: lands() },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g(), g()]),
            from_graveyard: true,
            effect: Effect::Seq(vec![
                Effect::Move { what: target_filtered(R::Land.from_your_graveyard()), to: tapped() },
                Effect::Move { what: Selector::This, to: tapped() },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Sandman, Shifting Scoundrel",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Sand, CreatureType::Elemental, CreatureType::Villain],
            0,
            0,
        )
    }
}

// ── The Muckslinger Gang ─────────────────────────────────────────────────────

/// The Muckslinger Gang — {1}{B}{R}{G} legendary 3/3 Goblin Lizard Beast.
/// Each other creature you control gets +3/+3 while it's not attacking or
/// blocking. 🟡 "Creature spells you control and creature cards you own in
/// any zone other than the battlefield or the stack get +3/+3" is not
/// modelled (INCOMPLETE_CARDS).
pub fn the_muckslinger_gang() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each other creature you control gets +3/+3 as long as it's not attacking or blocking.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource)
                        .and(R::IsAttacking.negate())
                        .and(R::IsBlocking.negate()),
                ),
                power: 3,
                toughness: 3,
            },
        }],
        ..legend(
            "The Muckslinger Gang",
            cost(&[generic(1), b(), r(), g()]),
            vec![CreatureType::Goblin, CreatureType::Lizard, CreatureType::Beast],
            3,
            3,
        )
    }
}

/// Soul Immolation — {3}{R}{R} sorcery. As an additional cost, blight X (X at
/// most the greatest toughness among your creatures). It deals X damage to
/// each opponent and each creature they control.
pub fn soul_immolation() -> CardDefinition {
    CardDefinition {
        name: "Soul Immolation",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::BlightX],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::XFromCost },
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                amount: Value::XFromCost,
            },
        ]),
        ..Default::default()
    }
}
