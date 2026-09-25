//! Commander: the cards the **Timeless Wisdom** precon (C20, Gavi, Nest
//! Warden) needed beyond what the catalog had (Descend upon the Sinful
//! landed first with Desert Bloom, `cmdr_yuma.rs`). Tests in
//! `tests/recent_b/cmdr_gavi.rs`.
//!
//! Residuals (each also on its card):
//! - **Akim, the Soaring Wind** — "the first time each turn" counts from when
//!   Akim is on the battlefield: tokens made earlier that turn don't use it up.
//! - **Crystalline Resonance** — the copy lasts until it copies again, not
//!   until your next turn.
//! - **Ethereal Forager** — the returned card is the first linked instant or
//!   sorcery, not a chosen one.
//! - **Nimble Obstructionist** — "you don't control" reads the ability's
//!   source permanent's controller.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, MayPlayDuration, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, partner_with_search, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, hybrid, r, u, w};
use crate::sets::tap_add_colorless;
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn permanent(name: &'static str, mana: ManaCost, kind: CardType) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], ..Default::default() }
}

fn token(name: &str, colors: Vec<Color>, p: i32, t: i32, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

fn make(definition: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition }
}

/// "When you cycle this card, …" (CR 702.29c).
fn on_cycle_this(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource), effect }
}

/// "Whenever you cycle a card, …" — a permanent's watch on its controller.
fn on_you_cycle(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::CardCycled, EventScope::YourControl), effect }
}

/// "Whenever you discard a [filter] card, …" (cycling discards too).
fn on_you_discard(filter: Option<R>, effect: Effect) -> TriggeredAbility {
    let mut event = EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl);
    if let Some(filter) = filter {
        event = event.with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter });
    }
    TriggeredAbility { event, effect }
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

// ── Commander ────────────────────────────────────────────────────────────────

/// Gavi, Nest Warden — the first card you cycle each turn may cycle for {0};
/// your second card drawn each turn makes a 2/2 red and white Dinosaur Cat.
pub fn gavi_nest_warden() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may pay {0} rather than pay the cycling cost of the first card you cycle each turn.",
            effect: StaticEffect::FirstCyclingEachTurnFree,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SecondCardDrawnThisTurn, EventScope::YourControl),
            effect: make(token(
                "Dinosaur Cat",
                vec![Color::Red, Color::White],
                2,
                2,
                vec![CreatureType::Dinosaur, CreatureType::Cat],
                vec![],
            )),
        }],
        ..legendary(creature(
            "Gavi, Nest Warden",
            cost(&[generic(2), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            2,
            5,
        ))
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Akim, the Soaring Wind — flying; your first tokens each turn add a 1/1
/// flying Bird; {3}{U}{R}{W}: your creature tokens gain double strike.
/// Residual: "the first time each turn" is counted from Akim's arrival — the
/// trigger is once per turn, so tokens made before it entered don't spend it.
pub fn akim_the_soaring_wind() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TokenCreated, EventScope::YourControl).once_per_turn(),
            effect: make(token("Bird", vec![Color::White], 1, 1, vec![CreatureType::Bird], vec![Keyword::Flying])),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u(), r(), w()]),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::IsToken).and(R::ControlledByYou)),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legendary(creature(
            "Akim, the Soaring Wind",
            cost(&[generic(2), u(), r(), w()]),
            vec![CreatureType::Bird, CreatureType::Dinosaur],
            3,
            4,
        ))
    }
}

/// Brallin, Skyshark Rider — partner with Shabraz; each discard grows it and
/// pings each opponent; {R}: target Shark gains trample.
pub fn brallin_skyshark_rider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Shabraz, the Skyshark".into())],
        triggered_abilities: vec![
            partner_with_search("Shabraz, the Skyshark"),
            on_you_discard(
                None,
                Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                ]),
            ),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Shark))),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legendary(creature(
            "Brallin, Skyshark Rider",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            3,
            3,
        ))
    }
}

/// Shabraz, the Skyshark — partner with Brallin; flying; each card you draw
/// grows it and gains 1 life; {W/U}: target Human gains flying.
pub fn shabraz_the_skyshark() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Brallin, Skyshark Rider".into()), Keyword::Flying],
        triggered_abilities: vec![
            partner_with_search("Brallin, Skyshark Rider"),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ]),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[hybrid(Color::White, Color::Blue)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Human))),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legendary(creature(
            "Shabraz, the Skyshark",
            cost(&[generic(3), w(), u()]),
            vec![CreatureType::Shark, CreatureType::Bird],
            3,
            3,
        ))
    }
}

/// Ethereal Forager — delve, flying; attacking, it may return an instant or
/// sorcery card it delved to its owner's hand.
/// Residual: the returned card is the first linked instant or sorcery, not a
/// chosen one.
pub fn ethereal_forager() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Delve, Keyword::Flying],
        links_delved_cards: true,
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Return an instant or sorcery card exiled with Ethereal Forager to its owner's hand?".into(),
            body: Box::new(Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::MatchingAmong {
                        inner: Box::new(Selector::CardExiledWithSource),
                        filter: instant_or_sorcery(),
                    }),
                    count: Box::new(Value::ONE),
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..creature(
            "Ethereal Forager",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Elemental, CreatureType::Whale],
            3,
            3,
        )
    }
}

/// Herald of the Forgotten — flying; cast, it returns any number of
/// permanent cards with cycling from your graveyard to the battlefield.
pub fn herald_of_the_forgotten() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            // CR 603.4 — "if you cast it" is an intervening if.
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceWasCast),
            effect: Effect::ApplyToTargets {
                max_targets: 20,
                min_targets: 0,
                filter: R::PermanentCard.and(R::HasCyclingAbility).and(R::InYourGraveyard),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..creature(
            "Herald of the Forgotten",
            cost(&[generic(6), w(), w()]),
            vec![CreatureType::Cat, CreatureType::Beast],
            6,
            6,
        )
    }
}

/// Nimble Obstructionist — flash, flying; cycling {2}{U}; cycled, it
/// counters target activated or triggered ability you don't control.
/// Residual: "you don't control" reads the ability's source permanent.
pub fn nimble_obstructionist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Cycling(cost(&[generic(2), u()]))],
        triggered_abilities: vec![on_cycle_this(Effect::CounterAbility {
            what: target_filtered(R::HasAbilityOnStack.and(R::ControlledByOpponent)),
        })],
        ..creature(
            "Nimble Obstructionist",
            cost(&[generic(2), u()]),
            vec![CreatureType::Bird, CreatureType::Wizard],
            3,
            1,
        )
    }
}

/// Rooting Moloch — enters: exile a cycling card from your graveyard, playable
/// until the end of your next turn; cycling {2}.
pub fn rooting_moloch() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::HasCyclingAbility.and(R::InYourGraveyard)),
                to: ZoneDest::Exile,
            },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: MayPlayDuration::EndOfControllersNextTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        ]))],
        ..creature("Rooting Moloch", cost(&[generic(4), r()]), vec![CreatureType::Lizard], 4, 4)
    }
}

/// Spellpyre Phoenix — flying; enters: may return an instant or sorcery card
/// with cycling from your graveyard; each end step after you cycled two or
/// more cards, it returns from your graveyard to your hand.
pub fn spellpyre_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Return an instant or sorcery card with cycling to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(instant_or_sorcery().and(R::HasCyclingAbility).and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            }),
            TriggeredAbility {
                // CR 603.4 — "if you cycled two or more cards this turn".
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::FromYourGraveyardAnyPlayer)
                    .with_filter(Predicate::ValueAtLeast(
                        Value::CardsCycledThisTurn(PlayerRef::You),
                        Value::Const(2),
                    )),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            },
        ],
        ..creature("Spellpyre Phoenix", cost(&[generic(3), r(), r()]), vec![CreatureType::Phoenix], 4, 2)
    }
}

/// Surly Badgersaur — a discarded creature card grows it, a land card makes a
/// Treasure, anything else has it fight up to one creature you don't control.
pub fn surly_badgersaur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_you_discard(
                Some(R::Creature),
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ),
            on_you_discard(Some(R::Land), crate::effect::shortcut::mint_treasures(1)),
            on_you_discard(
                Some(R::Noncreature.and(R::Nonland)),
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Fight {
                        attacker: Selector::This,
                        defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    }),
                },
            ),
        ],
        ..creature(
            "Surly Badgersaur",
            cost(&[generic(3), r()]),
            vec![CreatureType::Badger, CreatureType::Dinosaur],
            3,
            3,
        )
    }
}

/// Vizier of Tumbling Sands — {T}: untap another target permanent; cycling
/// {1}{U}; cycled, untap target permanent.
pub fn vizier_of_tumbling_sands() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(1), u()]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Untap { what: target_filtered(R::Permanent.and(R::OtherThanSource)), up_to: None },
            ..Default::default()
        }],
        triggered_abilities: vec![on_cycle_this(Effect::Untap {
            what: target_filtered(R::Permanent),
            up_to: None,
        })],
        ..creature(
            "Vizier of Tumbling Sands",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            3,
        )
    }
}

// ── Noncreature spells ───────────────────────────────────────────────────────

/// Abandoned Sarcophagus — cast cycling spells from your graveyard; your
/// cycling cards that weren't cycled are exiled instead of hitting it.
pub fn abandoned_sarcophagus() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "You may cast spells that have a cycling ability from your graveyard.",
                effect: StaticEffect::CastFromGraveyardMatching { filter: R::HasCyclingAbility },
            },
            StaticAbility {
                description: "If a card that has a cycling ability would be put into your graveyard from \
                              anywhere and it wasn't cycled, exile it instead.",
                effect: StaticEffect::ExileOwnCyclingCardsUnlessCycled,
            },
        ],
        ..permanent("Abandoned Sarcophagus", cost(&[generic(3)]), CardType::Artifact)
    }
}

/// Astral Drift — cycling {2}{W}; cycling it, or cycling another card while
/// it's out, may flicker target creature until the next end step.
pub fn astral_drift() -> CardDefinition {
    let flicker = || Effect::MayDo {
        description: "Exile target creature until the next end step?".into(),
        body: Box::new(Effect::ExileReturnToOwnerNextEndStep { what: target_filtered(R::Creature), tapped: false }),
    };
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2), w()]))],
        triggered_abilities: vec![on_cycle_this(flicker()), on_you_cycle(flicker())],
        ..permanent("Astral Drift", cost(&[generic(2), w()]), CardType::Enchantment)
    }
}

/// Crystalline Resonance — whenever you cycle, it may become a copy of another
/// target permanent, keeping this ability.
/// Residual: the copy lasts until it copies again, not until your next turn.
pub fn crystalline_resonance() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_you_cycle(Effect::MayDo {
            description: "Have Crystalline Resonance become a copy of another target permanent?".into(),
            body: Box::new(Effect::BecomeCopyOf {
                what: Selector::This,
                source: target_filtered(R::Permanent.and(R::OtherThanSource)),
                extra_creature_types: vec![],
                keep_own_triggered: true,
                keep_own_activated: false,
            }),
        })],
        ..permanent("Crystalline Resonance", cost(&[generic(2), u()]), CardType::Enchantment)
    }
}

/// Drake Haven — whenever you cycle or discard a card, you may pay {1} for a
/// 2/2 blue flying Drake.
pub fn drake_haven() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_you_discard(
            None,
            Effect::MayPay {
                description: "Pay {1} to create a 2/2 blue Drake with flying?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(make(token(
                    "Drake",
                    vec![Color::Blue],
                    2,
                    2,
                    vec![CreatureType::Drake],
                    vec![Keyword::Flying],
                ))),
                else_: None,
            },
        )],
        ..permanent("Drake Haven", cost(&[generic(2), u()]), CardType::Enchantment)
    }
}

/// New Perspectives — enters: draw three; with seven or more cards in hand,
/// cycling costs may be {0}.
pub fn new_perspectives() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as you have seven or more cards in hand, you may pay {0} rather than pay cycling costs.",
            effect: StaticEffect::CyclingFreeWhileHandAtLeast(7),
        }],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(3) })],
        ..permanent("New Perspectives", cost(&[generic(5), u()]), CardType::Enchantment)
    }
}

/// Tectonic Reformation — land cards in your hand have cycling {R}; cycling
/// {2}.
pub fn tectonic_reformation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        static_abilities: vec![StaticAbility {
            description: "Each land card in your hand has cycling {R}.",
            effect: StaticEffect::GrantCyclingToYourHandCards { filter: R::Land, cost: cost(&[r()]) },
        }],
        ..permanent("Tectonic Reformation", cost(&[generic(1), r()]), CardType::Enchantment)
    }
}

// ── Land ─────────────────────────────────────────────────────────────────────

/// Hostile Desert — {T}: {C}; {2}, exile a land card from your graveyard: it
/// becomes a 3/4 Elemental creature until end of turn.
pub fn hostile_desert() -> CardDefinition {
    CardDefinition {
        name: "Hostile Desert",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Desert], ..Default::default() },
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                exile_other_filter: Some((R::Land, 1)),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(4),
                    creature_types: vec![CreatureType::Elemental],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
