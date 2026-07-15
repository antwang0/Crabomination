//! A seventh wave of staples — mana dorks, card-selection sorceries, value
//! creatures, and utility lands that filled remaining gaps. Each card has a
//! functionality test in `crabomination/src/tests/recent7.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement, Selector, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::mana::{b, cost, g, generic, u, w, Color};

// ── White ────────────────────────────────────────────────────────────────

/// Mardu Woe-Reaper — {W} 2/1 Human Warrior. Whenever this or another Warrior
/// you control enters, you may exile a creature card from a graveyard; if you
/// do, gain 1 life.
pub fn mardu_woe_reaper() -> CardDefinition {
    CardDefinition {
        name: "Mardu Woe-Reaper",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Warrior),
                }),
            effect: Effect::MayDo {
                description: "Exile a creature card from a graveyard to gain 1 life?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(SelectionRequirement::Creature),
                        to: ZoneDest::Exile,
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ])),
            },
        }],
        ..Default::default()
    }
}

// ── Blue ─────────────────────────────────────────────────────────────────

/// Peek — {U} Instant. Look at target player's hand, then draw a card. (The
/// look is information-only; the engine resolves it as a cantrip.)
pub fn peek() -> CardDefinition {
    CardDefinition {
        name: "Peek",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        ..Default::default()
    }
}

/// Pieces of the Puzzle — {2}{U} Sorcery. Reveal the top five cards; put up to
/// two instant/sorcery cards into your hand and the rest into your graveyard.
pub fn pieces_of_the_puzzle() -> CardDefinition {
    CardDefinition {
        name: "Pieces of the Puzzle",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_to_graveyard: true,
            pick_filter: Some(
                SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            ),
            take: Some(Value::Const(2)),
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: false,
        },
        ..Default::default()
    }
}

// ── Black ────────────────────────────────────────────────────────────────

/// Ransack the Lab — {1}{B} Sorcery. Look at the top three cards; put one into
/// your hand and the rest into your graveyard.
pub fn ransack_the_lab() -> CardDefinition {
    CardDefinition {
        name: "Ransack the Lab",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: true,
            pick_filter: None,
            take: Some(Value::ONE),
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: false,
        },
        ..Default::default()
    }
}

// ── Green ────────────────────────────────────────────────────────────────

/// Leaf Gilder — {1}{G} 2/1 Elf Druid. {T}: Add {G}.
pub fn leaf_gilder() -> CardDefinition {
    CardDefinition {
        name: "Leaf Gilder",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Green, Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Quirion Elves — {1}{G} 1/1 Elf Druid. As it enters, choose a color. {T}: Add
/// {G}. {T}: Add one mana of the chosen color.
pub fn quirion_elves() -> CardDefinition {
    CardDefinition {
        name: "Quirion Elves",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::ChooseColorForSelf)],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Green, Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::ChosenColorOfSource },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Skyshroud Elf — {1}{G} 1/1 Elf Druid. {T}: Add {G}. {1}: Add {R} or {W}.
/// (The single "add {R} or {W}" mode is modeled as two {1} mana abilities.)
pub fn skyshroud_elf() -> CardDefinition {
    CardDefinition {
        name: "Skyshroud Elf",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Green, Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Red, Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::White, Value::ONE) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Briar Shield — {G} Aura. Enchant creature; +1/+1. Sacrifice this Aura:
/// enchanted creature gets +3/+3 until end of turn.
pub fn briar_shield() -> CardDefinition {
    CardDefinition {
        name: "Briar Shield",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        // Pump the host first, then sacrifice the Aura — capturing the enchanted
        // creature while the Aura is still attached (sac-as-cost would clear the
        // `AttachedTo` link before the pump resolves).
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::SacrificeSource,
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Krosan Tusker — {5}{G}{G} 6/5 Boar Beast. Cycling {2}{G}; when you cycle it,
/// you may search your library for a basic land and put it into your hand.
pub fn krosan_tusker() -> CardDefinition {
    CardDefinition {
        name: "Krosan Tusker",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar, CreatureType::Beast],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Cycling(cost(&[generic(2), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

// ── Lands ────────────────────────────────────────────────────────────────

/// Phyrexian Tower — Legendary Land. {T}: Add {C}. {T}, Sacrifice a creature:
/// Add {B}{B}.
pub fn phyrexian_tower() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Tower",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((SelectionRequirement::Creature, 1)),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Black, Value::Const(2)) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
