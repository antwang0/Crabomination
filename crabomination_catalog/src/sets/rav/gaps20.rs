//! Ravnica (RAV) gap wave 20: Auras that steal and fly, the shared-keyword
//! sweep, and library manipulation. Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    Keyword, SelectionRequirement as R, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, TriggeredAbility,
};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn saproling() -> TokenDefinition {
    TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Saproling],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Dream Leash — {3}{U}{U} Aura. Enchant permanent; you can only aim it at a
/// tapped permanent, and you control what it enchants.
pub fn dream_leash() -> CardDefinition {
    CardDefinition {
        name: "Dream Leash",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Permanent.and(R::Tapped)),
        },
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        ..Default::default()
    }
}

/// Auratouched Mage — {5}{W} 3/3 Human Wizard. When it enters, search your
/// library for an Aura that could enchant it and attach it.
pub fn auratouched_mage() -> CardDefinition {
    CardDefinition {
        name: "Auratouched Mage",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::SearchAuraAttachToSource)],
        ..Default::default()
    }
}

/// Flame Fusillade — {3}{R} Sorcery. Until end of turn, your permanents gain
/// "{T}: This permanent deals 1 damage to any target."
pub fn flame_fusillade() -> CardDefinition {
    CardDefinition {
        name: "Flame Fusillade",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GainActivatedAbility {
            what: Selector::EachPermanent(R::ControlledByYou),
            ability: Box::new(ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: crate::effect::shortcut::target_any(),
                    amount: Value::ONE,
                },
                ..Default::default()
            }),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Pollenbright Wings — {4}{G}{W} Aura. Enchant creature; it has flying and
/// mints a Saproling per point of combat damage it deals to a player.
pub fn pollenbright_wings() -> CardDefinition {
    CardDefinition {
        name: "Pollenbright Wings",
        cost: cost(&[generic(4), g(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::EnchantedBySource,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::PowerOf(Box::new(Selector::AttachedTo(Box::new(Selector::This)))),
                definition: Box::new(saproling()),
            },
        }],
        ..Default::default()
    }
}

/// Chant of Vitu-Ghazi — {6}{W}{W} Instant with convoke. Prevent all damage
/// creatures would deal this turn.
///
/// The printed "gain life equal to the damage prevented" rider needs a
/// per-point prevention hook the global combat-damage fog doesn't offer.
pub fn chant_of_vitu_ghazi() -> CardDefinition {
    CardDefinition {
        name: "Chant of Vitu-Ghazi",
        cost: cost(&[generic(6), w(), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Convoke],
        effect: Effect::PreventAllCombatDamageThisTurn,
        ..Default::default()
    }
}

/// Moonlight Bargain — {3}{B}{B} Instant. Look at the top five cards; each one
/// hits your graveyard unless you pay 2 life. The rest go to your hand.
pub fn moonlight_bargain() -> CardDefinition {
    CardDefinition {
        name: "Moonlight Bargain",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::LookTopEachPayLifeOrBin {
            count: Value::Const(5),
            life: 2,
        },
        ..Default::default()
    }
}

/// Tunnel Vision — {5}{U} Sorcery. Name a card; target player mills until it
/// turns up (and it goes back on top), or shuffles if it never does.
pub fn tunnel_vision() -> CardDefinition {
    CardDefinition {
        name: "Tunnel Vision",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::NameCardRevealUntilThenBin {
            who: PlayerRef::Target(0),
        },
        ..Default::default()
    }
}

/// Concerted Effort — {2}{W}{W} Enchantment. At each upkeep, your creatures
/// share flying, fear, first/double strike, trample and vigilance if any one of
/// them has it. (Landwalk and protection are keyed to a specific land type /
/// colour, so they aren't in the shared set.)
pub fn concerted_effort() -> CardDefinition {
    CardDefinition {
        name: "Concerted Effort",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::ShareKeywordsAmongYourCreatures {
                keywords: vec![
                    Keyword::Flying,
                    Keyword::Fear,
                    Keyword::FirstStrike,
                    Keyword::DoubleStrike,
                    Keyword::Trample,
                    Keyword::Vigilance,
                ],
            },
        }],
        ..Default::default()
    }
}
