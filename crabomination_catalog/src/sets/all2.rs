//! Alliances (ALL) — first wave. Tests in `classic_sets/all`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::{on_dies, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn aura(name: &'static str, c: ManaCost, enchants: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchants) },
        ..Default::default()
    }
}

/// The Alliances cantrip rider: "Draw a card at the beginning of the next
/// turn's upkeep."
fn cantrip_next_upkeep() -> Effect {
    Effect::AtNextTurnsUpkeep {
        body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
    }
}

/// Agent of Stromgald — a Knight that launders red into black.
pub fn agent_of_stromgald() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Black]),
            },
            ..Default::default()
        }],
        ..creature(
            "Agent of Stromgald",
            cost(&[r()]),
            vec![CreatureType::Human, CreatureType::Knight],
            1,
            1,
        )
    }
}

/// Arcane Denial — a counterspell that pays its victim back.
pub fn arcane_denial() -> CardDefinition {
    CardDefinition {
        name: "Arcane Denial",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::AtNextTurnsUpkeep {
                body: Box::new(Effect::Draw {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(2),
                }),
            },
            cantrip_next_upkeep(),
        ]),
        ..Default::default()
    }
}

/// Astrolabe — two of a colour now, a card later.
pub fn astrolabe() -> CardDefinition {
    CardDefinition {
        name: "Astrolabe",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                crate::effect::shortcut::add_any_one_color(2),
                cantrip_next_upkeep(),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Balduvian Horde — five power for a random card off the top of your hand.
pub fn balduvian_horde() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(
            Effect::SacrificeSourceUnlessCost {
                cost: crate::card::WardCost::DiscardRandom(1),
            },
        )],
        ..creature(
            "Balduvian Horde",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Human, CreatureType::Barbarian],
            5,
            5,
        )
    }
}

/// Carrier Pigeons — a flier that mails you a card next upkeep.
pub fn carrier_pigeons() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::etb(cantrip_next_upkeep())],
        ..creature("Carrier Pigeons", cost(&[generic(3), w()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Enslaved Scout — buys mountainwalk by the turn.
pub fn enslaved_scout() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Landwalk(LandType::Mountain),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Enslaved Scout",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Scout],
            2,
            2,
        )
    }
}

/// Errand of Duty — a banding Knight at instant speed.
pub fn errand_of_duty() -> CardDefinition {
    CardDefinition {
        name: "Errand of Duty",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(TokenDefinition {
                name: "Knight".into(),
                power: 1,
                toughness: 1,
                colors: vec![Color::White],
                card_types: vec![CardType::Creature],
                keywords: vec![Keyword::Banding],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Knight],
                    ..Default::default()
                },
                ..Default::default()
            }),
        },
        ..Default::default()
    }
}

/// Feast or Famine — a Zombie, or a burial.
pub fn feast_or_famine() -> CardDefinition {
    CardDefinition {
        name: "Feast or Famine",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Zombie".into(),
                    power: 2,
                    toughness: 2,
                    colors: vec![Color::Black],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Zombie],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
            Effect::DestroyNoRegen {
                what: target_filtered(
                    R::Creature
                        .and(R::Not(Box::new(R::Artifact)))
                        .and(R::Not(Box::new(R::HasColor(Color::Black)))),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Fevered Strength — a pump that replaces itself.
pub fn fevered_strength() -> CardDefinition {
    CardDefinition {
        name: "Fevered Strength",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            crate::effect::shortcut::pump_target(2, 0),
            cantrip_next_upkeep(),
        ]),
        ..Default::default()
    }
}

/// Foresight — bury three cards from your library, then cantrip.
pub fn foresight() -> CardDefinition {
    CardDefinition {
        name: "Foresight",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Exile },
            Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Exile },
            Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Exile },
            cantrip_next_upkeep(),
        ]),
        ..Default::default()
    }
}

/// Fyndhorn Druid — pays out when it dies having been blocked.
pub fn fyndhorn_druid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::WasBlockedThisTurn,
                },
            ),
            effect: crate::effect::shortcut::gain_life(4),
        }],
        ..creature(
            "Fyndhorn Druid",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            2,
            2,
        )
    }
}

/// "…blocks or becomes blocked": one trigger per side, each gated on the host's
/// actual combat role so a blocked attacker doesn't fire both.
fn host_matches(filter: R) -> Predicate {
    Predicate::EntityMatches {
        what: Selector::AttachedTo(Box::new(Selector::This)),
        filter,
    }
}

/// Gift of the Woods — a combat trick that stays on the board.
pub fn gift_of_the_woods() -> CardDefinition {
    let payoff = || {
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::ZERO,
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            crate::effect::shortcut::gain_life(1),
        ])
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::EnchantedBySource)
                    .with_filter(host_matches(R::IsBlocking)),
                effect: payoff(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::EnchantedBySource)
                    .with_filter(host_matches(R::IsBlocked)),
                effect: payoff(),
            },
        ],
        ..aura("Gift of the Woods", cost(&[g()]), R::Creature)
    }
}

/// Inheritance — every death is a card, for a price.
pub fn inheritance() -> CardDefinition {
    CardDefinition {
        name: "Inheritance",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::MayPay {
                description: "Pay {3} to draw a card?".into(),
                mana_cost: cost(&[generic(3)]),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Insidious Bookworms — a one-drop that takes a card with it.
pub fn insidious_bookworms() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::MayPay {
            description: "Pay {1}{B} to strip a card at random?".into(),
            mana_cost: cost(&[generic(1), b()]),
            body: Box::new(Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: true,
            }),
            else_: None,
        })],
        ..creature("Insidious Bookworms", cost(&[b()]), vec![CreatureType::Worm], 1, 1)
    }
}

/// Juniper Order Advocate — an anthem that only works standing up.
pub fn juniper_order_advocate() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is untapped, green creatures you control \
                          get +1/+1.",
            effect: StaticEffect::AnthemForFilterIf {
                filter: R::Creature.and(R::HasColor(Color::Green)),
                power: 1,
                toughness: 1,
                keywords: vec![],
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::Untapped,
                },
                all_players: false,
            },
        }],
        ..creature(
            "Juniper Order Advocate",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            1,
            2,
        )
    }
}

/// Kaysa — an anthem for the whole green team.
pub fn kaysa() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Each green creature you control gets +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::HasColor(Color::Green)),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..creature(
            "Kaysa",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            2,
            3,
        )
    }
}
