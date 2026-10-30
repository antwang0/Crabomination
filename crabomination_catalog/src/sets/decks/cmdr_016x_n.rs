//! Commander: 2026-10-19 (`016Xh16H`) — The Unluckiest Planeswalker's and
//! Lynde, Cheerful Tormentor's Curses. Tests in
//! `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope,
    EventSpec, Keyword, LoyaltyAbility, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    DelayedTriggerKind, Effect, ManaPayload, PlayerRef, Predicate, Selector, Value,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, u, w};
use std::sync::Arc;

fn curse_types() -> Subtypes {
    Subtypes {
        enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
        ..Default::default()
    }
}

fn is_curse() -> R {
    R::HasEnchantmentSubtype(EnchantmentSubtype::Curse)
}

/// An "enchant player" Curse with `triggers`.
fn curse(name: &'static str, mana: ManaCost, triggers: Vec<TriggeredAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: curse_types(),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: triggers,
        ..Default::default()
    }
}

/// "At the beginning of enchanted player's `step`, `effect`."
fn at_enchanted_players(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(step), EventScope::AnyPlayer)
            .with_filter(Predicate::IsTurnOf(PlayerRef::EnchantedPlayer)),
        effect,
    }
}

fn enchanted() -> Selector {
    Selector::Player(PlayerRef::EnchantedPlayer)
}

fn curses_on_enchanted() -> Value {
    Value::AttachedToPlayerMatching { who: PlayerRef::EnchantedPlayer, filter: is_curse() }
}

fn enchanted_player_casts() -> Predicate {
    Predicate::SamePlayer(PlayerRef::Triggerer, PlayerRef::EnchantedPlayer)
}

/// Curse of Thirst — {4}{B}. At the beginning of enchanted player's upkeep,
/// it deals that player damage equal to the Curses attached to them.
pub fn curse_of_thirst() -> CardDefinition {
    curse(
        "Curse of Thirst",
        cost(&[generic(4), b()]),
        vec![at_enchanted_players(TurnStep::Upkeep, Effect::DealDamage { to: enchanted(), amount: curses_on_enchanted() })],
    )
}

/// Curse of Obsession — {4}{R}. Enchanted player draws two more in their
/// draw step and discards their hand at their end step.
pub fn curse_of_obsession() -> CardDefinition {
    curse(
        "Curse of Obsession",
        cost(&[generic(4), r()]),
        vec![
            at_enchanted_players(TurnStep::Draw, Effect::Draw { who: enchanted(), amount: Value::Const(2) }),
            at_enchanted_players(
                TurnStep::End,
                Effect::Discard {
                    who: enchanted(),
                    amount: Value::HandSizeOf(PlayerRef::EnchantedPlayer),
                    random: false,
                },
            ),
        ],
    )
}

/// Curse of Shaken Faith — {1}{R}. Whenever enchanted player casts a spell
/// other than their first this turn, or copies a spell, it deals 2 damage to
/// them.
pub fn curse_of_shaken_faith() -> CardDefinition {
    curse(
        "Curse of Shaken Faith",
        cost(&[generic(1), r()]),
        vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                    enchanted_player_casts(),
                    Predicate::Not(Box::new(Predicate::SpellsCastThisTurnEquals {
                        who: PlayerRef::EnchantedPlayer,
                        count: Value::ONE,
                    })),
                ])),
                effect: Effect::DealDamage { to: enchanted(), amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCopied, EventScope::AnyPlayer).with_filter(enchanted_player_casts()),
                effect: Effect::DealDamage { to: enchanted(), amount: Value::Const(2) },
            },
        ],
    )
}

/// Curse of Surveillance — {4}{U}. At the beginning of enchanted player's
/// upkeep, you draw a card per Curse attached to them (the "any number of
/// target players other than that player" is you).
pub fn curse_of_surveillance() -> CardDefinition {
    let mut c = curse(
        "Curse of Surveillance",
        cost(&[generic(4), u()]),
        vec![at_enchanted_players(TurnStep::Upkeep, Effect::Draw { who: Selector::You, amount: curses_on_enchanted() })],
    );
    c.triggered_abilities[0].event = c.triggered_abilities[0].event.clone().with_filter(Predicate::All(vec![
        Predicate::IsTurnOf(PlayerRef::EnchantedPlayer),
        Predicate::Not(Box::new(Predicate::SamePlayer(PlayerRef::You, PlayerRef::EnchantedPlayer))),
    ]));
    c
}

/// Torment of Scarabs — {3}{B}. At the beginning of enchanted player's
/// upkeep, they lose 3 life unless they sacrifice a nonland permanent or
/// discard a card.
pub fn torment_of_scarabs() -> CardDefinition {
    curse(
        "Torment of Scarabs",
        cost(&[generic(3), b()]),
        vec![at_enchanted_players(
            TurnStep::Upkeep,
            Effect::Punisher {
                chooser: enchanted(),
                options: vec![
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Permanent.and(R::Nonland) },
                ],
                otherwise: Box::new(Effect::LoseLife { who: enchanted(), amount: Value::Const(3) }),
            },
        )],
    )
}

/// Curse of Echoes — {4}{U}. Whenever enchanted player casts an instant or
/// sorcery spell, each other player may copy it with new targets.
pub fn curse_of_echoes() -> CardDefinition {
    curse(
        "Curse of Echoes",
        cost(&[generic(4), u()]),
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                enchanted_player_casts(),
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                },
            ])),
            effect: Effect::EachOtherPlayerMayCopy { what: Selector::TriggerSource, except: PlayerRef::EnchantedPlayer },
        }],
    )
}

/// Curse of Silence — {W}. As it enters, name a card; enchanted player's
/// spells with that name cost {2} more; when they cast one, you may
/// sacrifice it to draw a card.
pub fn curse_of_silence() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::NameCard { what: Selector::This, restrict_to: None }),
        static_abilities: vec![StaticAbility {
            description: "Spells with the chosen name enchanted player casts cost {2} more to cast.",
            effect: StaticEffect::EnchantedPlayerNamedSpellTax { amount: 2 },
        }],
        ..curse(
            "Curse of Silence",
            cost(&[w()]),
            vec![TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                    enchanted_player_casts(),
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NameChosenForSource },
                ])),
                effect: Effect::MayDo {
                    description: "Sacrifice Curse of Silence to draw a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::SacrificeSource,
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ])),
                },
            }],
        )
    }
}

/// Curse of Misfortunes — {4}{B}. At the beginning of your upkeep, you may
/// search for a Curse not named like one on enchanted player and put it
/// onto the battlefield attached to them.
pub fn curse_of_misfortunes() -> CardDefinition {
    curse(
        "Curse of Misfortunes",
        cost(&[generic(4), b()]),
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Search for a Curse to attach to the enchanted player?".into(),
                body: Box::new(Effect::SearchAttachToPlayer {
                    filter: is_curse(),
                    host: PlayerRef::EnchantedPlayer,
                    distinct_names: true,
                }),
            },
        }],
    )
}

/// Bitterheart Witch — {4}{B} 1/2 deathtouch Human Shaman. Dying, you may
/// search for a Curse and put it onto the battlefield attached to target
/// player.
pub fn bitterheart_witch() -> CardDefinition {
    CardDefinition {
        name: "Bitterheart Witch",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::MayDo {
                    description: "Search for a Curse to attach to that player?".into(),
                    body: Box::new(Effect::SearchAttachToPlayer {
                        filter: is_curse(),
                        host: PlayerRef::Target(0),
                        distinct_names: false,
                    }),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Accursed Witch // Infectious Curse — {3}{B} 4/2 Human Shaman; spells your
/// opponents cast targeting it cost {1} less; dying, it returns transformed
/// attached to target opponent. Infectious Curse: your spells targeting
/// enchanted player cost {1} less; at the beginning of enchanted player's
/// upkeep, they lose 1 life and you gain 1.
pub fn accursed_witch() -> CardDefinition {
    let infectious = CardDefinition {
        name: "Infectious Curse",
        card_types: vec![CardType::Enchantment],
        subtypes: curse_types(),
        static_abilities: vec![StaticAbility {
            description: "Spells you cast that target enchanted player cost {1} less to cast.",
            effect: StaticEffect::CostReductionForYourSpellsTargetingThis {
                amount: 1,
                any_caster: false,
                opponents_only: false,
            },
        }],
        triggered_abilities: vec![at_enchanted_players(
            TurnStep::Upkeep,
            Effect::Seq(vec![
                Effect::LoseLife { who: enchanted(), amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        )],
        ..Default::default()
    };
    CardDefinition {
        name: "Accursed Witch",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        back_face: Some(Box::new(infectious)),
        static_abilities: vec![StaticAbility {
            description: "Spells your opponents cast that target this creature cost {1} less to cast.",
            effect: StaticEffect::CostReductionForYourSpellsTargetingThis {
                amount: 1,
                any_caster: false,
                opponents_only: true,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::ReturnSelfTransformedAttachedTo { host: Selector::Player(PlayerRef::Target(0)) }),
            },
        }],
        ..Default::default()
    }
}

/// Lynde, Cheerful Tormentor — {1}{U}{B}{R} 2/4 legendary deathtouch Human
/// Warlock. A Curse put into your graveyard from the battlefield returns
/// attached to you at the next end step; at your upkeep you may move a Curse
/// on you to an opponent and, if you do, draw two.
pub fn lynde_cheerful_tormentor() -> CardDefinition {
    CardDefinition {
        name: "Lynde, Cheerful Tormentor",
        cost: cost(&[generic(1), u(), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: is_curse().and(R::OwnedByYou) },
                ),
                effect: Effect::DelayUntilWithCapture {
                    kind: DelayedTriggerKind::NextEndStep,
                    capture: Selector::TriggerSource,
                    body: Box::new(Effect::ReturnToBattlefieldAttachedToPlayer {
                        what: Selector::Target(0),
                        host: PlayerRef::You,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Attach a Curse attached to you to an opponent and draw two?".into(),
                    body: Box::new(Effect::ReattachPlayerAurasToOpponents {
                        from: PlayerRef::You,
                        filter: is_curse(),
                        all: false,
                        then: Some(Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) })),
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// The Unluckiest Planeswalker — {2}{R} legendary planeswalker, loyalty 4;
/// Rulebreaker (Aura cards of any color identity). +1: one mana of any
/// color; your next Aura spell this turn makes a 1/1 red Gremlin. −3:
/// discard your hand, draw twice the Auras you control, then move each Aura
/// attached to you to an opponent.
pub fn the_unluckiest_planeswalker() -> CardDefinition {
    let gremlin = TokenDefinition {
        name: "Gremlin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gremlin], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "The Unluckiest Planeswalker",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        base_loyalty: 4,
        can_be_commander: true,
        static_abilities: vec![StaticAbility {
            description: "Rulebreaker — Aura cards of any color identity, and any basic land cards.",
            effect: StaticEffect::RulebreakerAnyColorIdentity { cards: crate::effect::RulebreakerCards::Auras },
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                    Effect::OnYourNextSpellMatchingThisTurn {
                        filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura),
                        body: Box::new(Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: Arc::new(gremlin),
                        }),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Times(
                            Box::new(Value::Const(2)),
                            Box::new(Value::count(Selector::EachPermanent(
                                R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).and(R::ControlledByYou),
                            ))),
                        ),
                    },
                    Effect::ReattachPlayerAurasToOpponents {
                        from: PlayerRef::You,
                        filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura),
                        all: true,
                        then: None,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
