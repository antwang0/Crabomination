//! Commander: the cards the **Heads I Win, Tails You Lose** Secret Lair deck
//! (Zndrsplt, Eye of Wisdom + Okaun, Eye of Chaos) needed beyond what the
//! catalog had. Tests in `tests/recent_b/cmdr_zndrsplt.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::catalog::sets::{reveal_or_tapped_land, tap_add, tap_add_colorless};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, PlayerRef, Predicate, StaticAbility, StaticEffect,
    ZoneDest,
};
use crate::mana::{Color, cost, generic, r, u};

/// Bloodsworn Steward — {2}{R}{R} Creature — Vampire Knight 4/4. Flying.
/// Commander creatures you control get +2/+2 and have haste.
pub fn bloodsworn_steward() -> CardDefinition {
    CardDefinition {
        name: "Bloodsworn Steward",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you control get +2/+2 and have haste.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::IsCommander),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Haste],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Boompile — {4} Artifact. {T}: Flip a coin. If you win the flip, destroy all
/// nonland permanents.
pub fn boompile() -> CardDefinition {
    CardDefinition {
        name: "Boompile",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::Destroy { what: Selector::EachPermanent(R::Nonland) }),
                on_tails: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Desolate Lighthouse — Land. {T}: Add {C}. {1}{U}{R}, {T}: Draw a card, then
/// discard a card.
pub fn desolate_lighthouse() -> CardDefinition {
    CardDefinition {
        name: "Desolate Lighthouse",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u(), r()]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Flamekin Village — Land. As this land enters, you may reveal an Elemental
/// card from your hand. If you don't, this land enters tapped. {T}: Add {R}.
/// {R}, {T}: Target creature gains haste until end of turn.
pub fn flamekin_village() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_add(Color::Red),
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                tap_cost: true,
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..reveal_or_tapped_land(
            "Flamekin Village",
            "As this land enters, you may reveal an Elemental card from your hand. If you \
             don't, this land enters tapped.",
            R::HasCreatureType(CreatureType::Elemental),
            Color::Red,
            Color::Red,
        )
    }
}

/// Footfall Crater — {R} Enchantment — Aura. Enchant land. Enchanted land has
/// "{T}: Target creature gains trample and haste until end of turn." Cycling {1}.
pub fn footfall_crater() -> CardDefinition {
    CardDefinition {
        name: "Footfall Crater",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Cycling(cost(&[generic(1)]))],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}: Target creature gains trample and haste until \
                          end of turn.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::GrantKeywords {
                        what: target_filtered(R::Creature),
                        keywords: vec![Keyword::Trample, Keyword::Haste],
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Frenetic Sliver — {1}{U}{R} Creature — Sliver 2/2. All Slivers have "{0}: If
/// this permanent is on the battlefield, flip a coin. If you win the flip,
/// exile this permanent and return it to the battlefield under its owner's
/// control at the beginning of the next end step. If you lose the flip,
/// sacrifice it."
pub fn frenetic_sliver() -> CardDefinition {
    CardDefinition {
        name: "Frenetic Sliver",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sliver], ..Default::default() },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "All Slivers have \"{0}: flip a coin. If you win, exile this permanent \
                          and return it at the next end step. If you lose, sacrifice it.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Sliver)),
                ability: ActivatedAbility {
                    effect: Effect::FlipCoin {
                        count: Value::ONE,
                        on_heads: Box::new(Effect::ExileReturnToOwnerNextEndStep {
                            what: Selector::This,
                            tapped: false,
                        }),
                        on_tails: Box::new(Effect::SacrificeSource),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Daretti, Scrap Savant — {3}{R} Legendary Planeswalker — Daretti, loyalty 3.
/// +2: Discard up to two cards, then draw that many cards. −2: Sacrifice an
/// artifact. If you do, return target artifact card from your graveyard to the
/// battlefield. −10: You get an emblem with "Whenever an artifact is put into
/// your graveyard from the battlefield, return that card to the battlefield at
/// the beginning of the next end step." Daretti can be your commander.
pub fn daretti_scrap_savant() -> CardDefinition {
    let emblem_trigger = TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureOrArtifactDied, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Artifact,
            }),
        effect: Effect::DelayUntilWithCapture {
            kind: DelayedTriggerKind::NextEndStep,
            capture: Selector::TriggerSource,
            body: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        },
    };
    CardDefinition {
        name: "Daretti, Scrap Savant",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Daretti],
            ..Default::default()
        },
        base_loyalty: 3,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    Effect::DiscardAnyNumber {
                        who: Selector::You,
                        filter: R::Any,
                        max: Some(Value::Const(2)),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisEffect },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Artifact },
                    Effect::If {
                        cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                        then: Box::new(Effect::Move {
                            what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Daretti, Scrap Savant".into(),
                    triggered: vec![emblem_trigger],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// "At the beginning of combat on your turn, flip a coin until you lose a flip."
fn flip_at_combat() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat),
            EventScope::YourControl,
        ),
        effect: Effect::FlipUntilLoss { per_win: Box::new(Effect::Noop) },
    }
}

/// "Whenever a player wins a coin flip, …" — CR 705.1, any player's flip.
fn on_any_flip_won(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::WonCoinFlip, EventScope::AnyPlayer), effect }
}

fn eye(name: &'static str, partner: &str, mana: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        keywords: vec![Keyword::PartnerWith(partner.into())],
        ..Default::default()
    }
}

/// Zndrsplt, Eye of Wisdom — {4}{U} Legendary Creature — Homunculus 1/4.
/// Partner with Okaun, Eye of Chaos. At the beginning of combat on your turn,
/// flip a coin until you lose a flip. Whenever a player wins a coin flip,
/// draw a card.
pub fn zndrsplt_eye_of_wisdom() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { creature_types: vec![CreatureType::Homunculus], ..Default::default() },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![
            crate::effect::shortcut::partner_with_search("Okaun, Eye of Chaos"),
            flip_at_combat(),
            on_any_flip_won(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        ],
        ..eye("Zndrsplt, Eye of Wisdom", "Okaun, Eye of Chaos", cost(&[generic(4), u()]))
    }
}

/// Okaun, Eye of Chaos — {4}{R} Legendary Creature — Cyclops Berserker 3/3.
/// Partner with Zndrsplt, Eye of Wisdom. At the beginning of combat on your
/// turn, flip a coin until you lose a flip. Whenever a player wins a coin
/// flip, double Okaun's power and toughness until end of turn.
pub fn okaun_eye_of_chaos() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cyclops, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            crate::effect::shortcut::partner_with_search("Zndrsplt, Eye of Wisdom"),
            flip_at_combat(),
            // Doubling is +current: both amounts are read before the pump.
            on_any_flip_won(Effect::PumpPT {
                what: Selector::This,
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::ToughnessOf(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
            }),
        ],
        ..eye("Okaun, Eye of Chaos", "Zndrsplt, Eye of Wisdom", cost(&[generic(4), r()]))
    }
}
