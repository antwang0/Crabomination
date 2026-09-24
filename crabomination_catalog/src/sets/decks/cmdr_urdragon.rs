//! Commander: the cards the **Draconic Domination** precon (C17, The
//! Ur-Dragon) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_urdragon.rs`.
//!
//! Residuals (each also on its card):
//! - **Orator of Ojutai** — the Dragon check reads your board and hand as it
//!   enters; there is no reveal.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnterMode, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{bolster, dash, etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost, SpendRestriction};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

fn dragon() -> R {
    R::HasCreatureType(CreatureType::Dragon)
}

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

/// "Whenever a Dragon you control attacks" — one fire per attacking Dragon.
fn dragon_attacks(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: dragon() }),
        effect,
    }
}

fn dragon_token(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, pt: i32) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: pt,
        toughness: pt,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

/// Boneyard Scourge — flying; a Dragon of yours dying lets you pay {1}{B} to
/// return this from your graveyard.
pub fn boneyard_scourge() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::FromYourGraveyard).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: dragon().and(R::ControlledByYou).and(R::OtherThanSource),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {1}{B} to return Boneyard Scourge?".into(),
                mana_cost: cost(&[generic(1), b()]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
                else_: None,
            },
        }],
        ..creature(
            "Boneyard Scourge",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Dragon],
            4,
            3,
        )
    }
}

/// Broodmate Dragon — flying; enters with a 4/4 flying red Dragon token.
pub fn broodmate_dragon() -> CardDefinition {
    let token = dragon_token("Dragon", vec![Color::Red], vec![CreatureType::Dragon], 4);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(token),
        })],
        ..creature("Broodmate Dragon", cost(&[generic(3), b(), r(), g()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Crucible of the Spirit Dragon — {C}; {1}, {T}: a storage counter; {T},
/// remove X storage counters: X mana in any combination of colors, only for
/// Dragon spells or Dragons' abilities.
pub fn crucible_of_the_spirit_dragon() -> CardDefinition {
    CardDefinition {
        name: "Crucible of the Spirit Dragon",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Storage, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_x: Some(CounterType::Storage),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyColors(Value::XFromCost)),
                        SpendRestriction::CreatureOfTypeOrItsAbility(CreatureType::Dragon),
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dromoka, the Eternal — flying; each attacking Dragon of yours bolsters 2.
pub fn dromoka_the_eternal() -> CardDefinition {
    legend_dragon(
        "Dromoka, the Eternal",
        cost(&[generic(3), g(), w()]),
        5,
        5,
        dragon_attacks(bolster(2)),
    )
}

fn legend_dragon(name: &'static str, mana: ManaCost, p: i32, t: i32, trigger: TriggeredAbility) -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![trigger],
        ..creature(name, mana, vec![CreatureType::Dragon], p, t)
    })
}

/// Fortunate Few — each player spares one nonland permanent they don't
/// control; every other nonland permanent is destroyed.
pub fn fortunate_few() -> CardDefinition {
    CardDefinition {
        name: "Fortunate Few",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::EachPlayerSparesOneTheyDontControl,
        ..Default::default()
    }
}

/// Kolaghan, the Storm's Fury — flying; each attacking Dragon of yours gives
/// your creatures +1/+0; dash {3}{B}{R}.
pub fn kolaghan_the_storms_fury() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(dash(cost(&[generic(3), b(), r()]))),
        ..legend_dragon(
            "Kolaghan, the Storm's Fury",
            cost(&[generic(3), b(), r()]),
            4,
            5,
            dragon_attacks(Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
        )
    }
}

fn siege(name: &'static str, mana: ManaCost, khans: TriggeredAbility, dragons: EnterMode) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        enter_modes: Some(vec![
            EnterMode { label: "Khans".into(), triggered_abilities: vec![khans], ..Default::default() },
            dragons,
        ]),
        ..Default::default()
    }
}

/// Monastery Siege — Khans: loot on your draw step. Dragons: opponents'
/// spells that target you or your permanents cost {2} more.
pub fn monastery_siege() -> CardDefinition {
    siege(
        "Monastery Siege",
        cost(&[generic(2), u()]),
        TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        },
        EnterMode {
            label: "Dragons".into(),
            static_abilities: vec![StaticAbility {
                description: "Spells your opponents cast that target you or a permanent you control cost {2} more.",
                effect: StaticEffect::TaxOpponentSpellsTargeting {
                    target_filter: R::YouPlayer.or(R::Permanent.and(R::ControlledByYou)),
                    amount: 2,
                },
            }],
            ..Default::default()
        },
    )
}

/// O-Kagachi, Vengeful Kami — flying, trample; combat damage to a player who
/// attacked you during their last turn exiles a nonland permanent of theirs.
pub fn o_kagachi_vengeful_kami() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::Player(PlayerRef::TriggerEventPlayer),
                    filter: R::PlayerAttackedYouLastTurn,
                },
            ),
            effect: Effect::Exile { what: target_filtered(R::Nonland.and(R::ControlledByTriggerPlayer)) },
        }],
        ..creature(
            "O-Kagachi, Vengeful Kami",
            cost(&[generic(1), w(), u(), b(), r(), g()]),
            vec![CreatureType::Dragon, CreatureType::Spirit],
            6,
            6,
        )
    })
}

/// Ojutai, Soul of Winter — flying, vigilance; each attacking Dragon of yours
/// taps an opponent's nonland permanent, which skips its next untap.
pub fn ojutai_soul_of_winter() -> CardDefinition {
    let mut def = legend_dragon(
        "Ojutai, Soul of Winter",
        cost(&[generic(5), w(), u()]),
        5,
        6,
        dragon_attacks(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Nonland.and(R::ControlledByOpponent)) },
            Effect::SkipNextUntap { what: Selector::Target(0) },
        ])),
    );
    def.keywords.push(Keyword::Vigilance);
    def
}

/// Orator of Ojutai — defender, flying; enters: draw if you control a Dragon
/// or have one to reveal. Residual: the check reads board and hand as it
/// enters.
pub fn orator_of_ojutai() -> CardDefinition {
    let has_dragon = Predicate::Any(vec![
        Predicate::SelectorExists(Selector::EachPermanent(dragon().and(R::ControlledByYou))),
        Predicate::SelectorExists(Selector::CardsInZone {
            who: PlayerRef::You,
            zone: crate::card::Zone::Hand,
            filter: dragon(),
        }),
    ]);
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(has_dragon),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature("Orator of Ojutai", cost(&[generic(1), w()]), vec![CreatureType::Bird, CreatureType::Monk], 0, 4)
    }
}

/// Palace Siege — Khans: your upkeep returns a creature card to hand.
/// Dragons: your upkeep drains each opponent for 2.
pub fn palace_siege() -> CardDefinition {
    siege(
        "Palace Siege",
        cost(&[generic(3), b(), b()]),
        TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::Move {
                what: target_filtered(R::Creature.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        },
        EnterMode {
            label: "Dragons".into(),
            triggered_abilities: vec![TriggeredAbility {
                event: your_upkeep(),
                effect: Effect::DrainLifeLost {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(2),
                },
            }],
            ..Default::default()
        },
    )
}

/// Scalelord Reckoner — flying; a Dragon of yours targeted by an opponent's
/// spell or ability destroys a nonland permanent that player controls.
pub fn scalelord_reckoner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                actor_is_opponent: true,
                ..EventSpec::new(EventKind::BecameTarget, EventScope::YourCreatureTargeted)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: dragon() })
            },
            effect: Effect::Destroy { what: target_filtered(R::Nonland.and(R::ControlledByTriggerPlayer)) },
        }],
        ..creature("Scalelord Reckoner", cost(&[generic(3), w(), w()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Scion of the Ur-Dragon — flying; {2}: put a Dragon permanent card from
/// your library into your graveyard and become a copy of it until end of turn.
pub fn scion_of_the_ur_dragon() -> CardDefinition {
    legendary(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::PermanentCard.and(dragon()),
                    to: ZoneDest::Graveyard,
                },
                Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: Selector::LastMoved,
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Scion of the Ur-Dragon",
            cost(&[w(), u(), b(), r(), g()]),
            vec![CreatureType::Dragon, CreatureType::Avatar],
            4,
            4,
        )
    })
}

/// Silumgar, the Drifting Death — flying, hexproof; each attacking Dragon of
/// yours gives the defending player's creatures -1/-1.
pub fn silumgar_the_drifting_death() -> CardDefinition {
    let mut def = legend_dragon(
        "Silumgar, the Drifting Death",
        cost(&[generic(4), u(), b()]),
        3,
        7,
        dragon_attacks(Effect::PumpPT {
            what: Selector::ControlledBy { who: PlayerRef::DefendingPlayer, filter: R::Creature },
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        }),
    );
    def.keywords.push(Keyword::Hexproof);
    def
}

/// Spellbound Dragon — flying; attacking loots and pumps it by the discarded
/// card's mana value.
pub fn spellbound_dragon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            Effect::PumpPT {
                what: Selector::This,
                power: Value::LastDiscardedManaValue,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature("Spellbound Dragon", cost(&[generic(3), u(), r()]), vec![CreatureType::Dragon], 3, 5)
    }
}

/// Taigam, Ojutai Master — your instants, sorceries and Dragon spells can't be
/// countered; an instant or sorcery cast from hand after Taigam attacked this
/// turn gains rebound.
pub fn taigam_ojutai_master() -> CardDefinition {
    let instant_or_sorcery = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Instant, sorcery, and Dragon spells you control can't be countered.",
            effect: StaticEffect::SpellsCantBeCounteredMatching {
                filter: instant_or_sorcery().or(dragon()).and(R::ControlledByYou),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(instant_or_sorcery()),
                Predicate::CastFromHand,
                Predicate::EntityMatches { what: Selector::This, filter: R::AttackedThisTurn },
            ])),
            effect: Effect::GrantKeywordsToSpell { what: Selector::TriggerSource, keywords: vec![Keyword::Rebound] },
        }],
        keywords: vec![],
        ..legendary(creature(
            "Taigam, Ojutai Master",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Human, CreatureType::Monk],
            3,
            4,
        ))
    }
}

/// Wasitora, Nekoru Queen — flying, trample; combat damage to a player makes
/// them sacrifice a creature, or gives you a 3/3 flying Cat Dragon if they
/// can't.
pub fn wasitora_nekoru_queen() -> CardDefinition {
    let cat_dragon = dragon_token(
        "Cat Dragon",
        vec![Color::Black, Color::Red, Color::Green],
        vec![CreatureType::Cat, CreatureType::Dragon],
        3,
    );
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::TriggerEventPlayer,
                    filter: R::Creature,
                }),
                then: Box::new(Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    count: Value::ONE,
                    filter: R::Creature,
                }),
                else_: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(cat_dragon),
                }),
            },
        }],
        ..creature(
            "Wasitora, Nekoru Queen",
            cost(&[generic(2), b(), r(), g()]),
            vec![CreatureType::Cat, CreatureType::Dragon],
            5,
            4,
        )
    })
}
