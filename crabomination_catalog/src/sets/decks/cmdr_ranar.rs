//! Commander: the cards the **Phantom Premonition** precon (KHC, Ranar the
//! Ever-Watchful) needed beyond what the catalog had (Inspired Sphinx and
//! Thunderclap Wyvern landed with First Flight first). Tests in
//! `tests/recent_b/cmdr_ranar.rs`.
//!
//! Residuals (each also on its card):
//! - **Cosmic Intervention** — the exile-instead replacement covers the
//!   permanents you control as it resolves, and the end-step return brings
//!   back the cards you own that a this-turn "exile it instead" moved.
//! - **Niko Defies Destiny** — chapter III's target is the first card with
//!   foretell in your graveyard the engine's picker takes.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest, ZoneRef,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, cost, generic, u, w};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn flyer_token(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, card_types: Vec<CardType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types,
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

/// "Whenever one or more cards are put into exile from your hand or a spell
/// or ability you control exiles one or more permanents from the battlefield."
fn on_exile_from_hand_or_by_you(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CardsExiledFromHandOrByYou, EventScope::YourControl),
        effect,
    }
}

/// Arcane Artisan — {2}{U},{T}: target player draws, then exiles a card from
/// their hand; a creature card exiled becomes a token copy for them. When
/// Artisan leaves, its tokens are exiled at the next end step.
pub fn arcane_artisan() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Seq(vec![
                Effect::Draw { who: target_filtered(R::Player), amount: Value::ONE },
                Effect::ExileFromHandCopyCreature { who: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::ExileAtNextEndStep { what: Selector::TokensCreatedBySource },
        }],
        ..creature("Arcane Artisan", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Wizard], 0, 3)
    }
}

/// Cosmic Intervention — this turn, your permanents that would die are exiled
/// instead and return at the next end step. Foretell {1}{W}.
/// Residual: covers the permanents you control as it resolves; the return
/// takes the cards you own a this-turn "exile it instead" moved.
pub fn cosmic_intervention() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[generic(1), w()])),
        ..spell(
            "Cosmic Intervention",
            cost(&[generic(3), w()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::ExileIfWouldDieThisTurn { what: yours(R::Permanent) },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Move {
                        what: Selector::EachMatching {
                            zone: ZoneRef::Exile,
                            filter: R::ExiledInsteadOfDyingThisTurn.and(R::OwnedByYou),
                        },
                        to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
                    }),
                },
            ]),
        )
    }
}

/// Ethereal Valkyrie — flying; entering or attacking, draw a card, then
/// foretell a card from hand for its mana cost less {2}.
pub fn ethereal_valkyrie() -> CardDefinition {
    let body = || Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::ONE },
        Effect::ForetellFromHand { reduce: 2 },
    ]);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: body(),
            },
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: body() },
        ],
        ..creature(
            "Ethereal Valkyrie",
            cost(&[generic(4), w(), u()]),
            vec![CreatureType::Spirit, CreatureType::Angel],
            4,
            4,
        )
    }
}

/// Gates of Istfell — enters tapped; {T}: add {W}; {2}{W}{U}{U},{T},
/// sacrifice: gain 2 life and draw two.
pub fn gates_of_istfell() -> CardDefinition {
    CardDefinition {
        name: "Gates of Istfell",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::White, Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(2), w(), u(), u()]),
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                    Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Glacial Floodplain — snow Plains Island that enters tapped.
pub fn glacial_floodplain() -> CardDefinition {
    let mut d = crate::sets::dual_land_with(
        "Glacial Floodplain",
        LandType::Plains,
        LandType::Island,
        Color::White,
        Color::Blue,
        vec![],
    );
    d.supertypes.push(Supertype::Snow);
    d.static_abilities.push(crate::sets::enters_tapped());
    d
}

/// Hero of Bretagard — that many +1/+1 counters whenever cards leave your
/// hand for exile or your spells and abilities exile permanents; five
/// counters: flying and an Angel; ten: indestructible and a God.
pub fn hero_of_bretagard() -> CardDefinition {
    let at = |n: i32| Predicate::ValueAtLeast(
        Value::TotalCountersOn { what: Box::new(Selector::This) },
        Value::Const(n),
    );
    let gated = |n: i32, inner: StaticEffect| StaticEffect::WhileCondition { condition: at(n), inner: Box::new(inner) };
    CardDefinition {
        triggered_abilities: vec![on_exile_from_hand_or_by_you(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::TriggerEventAmount,
        })],
        static_abilities: vec![
            static_(
                "As long as Hero of Bretagard has five or more counters on it, it has flying.",
                gated(5, StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Flying }),
            ),
            static_(
                "As long as Hero of Bretagard has five or more counters on it, it is an Angel.",
                gated(
                    5,
                    StaticEffect::AddCreatureTypeToMatching { applies_to: Selector::This, creature_type: CreatureType::Angel },
                ),
            ),
            static_(
                "As long as Hero of Bretagard has ten or more counters on it, it has indestructible.",
                gated(10, StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Indestructible }),
            ),
            static_(
                "As long as Hero of Bretagard has ten or more counters on it, it is a God.",
                gated(
                    10,
                    StaticEffect::AddCreatureTypeToMatching { applies_to: Selector::This, creature_type: CreatureType::God },
                ),
            ),
        ],
        ..creature("Hero of Bretagard", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Warrior], 1, 1)
    }
}

/// Iron Verdict — 5 damage to target tapped creature. Foretell {W}.
pub fn iron_verdict() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[w()])),
        ..spell(
            "Iron Verdict",
            cost(&[generic(2), w()]),
            CardType::Instant,
            Effect::DealDamage { to: target_filtered(R::Creature.and(R::Tapped)), amount: Value::Const(5) },
        )
    }
}

/// Momentary Blink — flicker target creature you control. Flashback {3}{U}.
pub fn momentary_blink() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(3), u()]))],
        ..spell(
            "Momentary Blink",
            cost(&[generic(1), w()]),
            CardType::Instant,
            Effect::ExileAndReturnToOwner { what: target_filtered(R::Creature.and(R::ControlledByYou)) },
        )
    }
}

/// Niko Defies Destiny — I: 2 life per foretold card you own in exile; II:
/// {W}{U} for foretelling or foretell spells; III: return target card with
/// foretell from your graveyard to your hand.
pub fn niko_defies_destiny() -> CardDefinition {
    CardDefinition {
        name: "Niko Defies Destiny",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Times(
                        Box::new(Value::Const(2)),
                        Box::new(Value::ForetoldCardsOwnedInExile(PlayerRef::You)),
                    ),
                },
            ),
            (
                2,
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::White, Color::Blue])),
                        SpendRestriction::ForetellOnly,
                    ),
                },
            ),
            (
                3,
                Effect::Move {
                    what: target_filtered(R::HasForetell.from_your_graveyard()),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Ranar the Ever-Watchful — flying, vigilance; your first foretell each turn
/// is free; a 1/1 flying Spirit whenever cards leave your hand for exile or
/// your spells and abilities exile permanents.
pub fn ranar_the_ever_watchful() -> CardDefinition {
    let spirit = Arc::new(flyer_token("Spirit", vec![Color::White], vec![CreatureType::Spirit], vec![CardType::Creature]));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![static_(
            "The first card you foretell each turn costs {0} to foretell.",
            StaticEffect::FirstForetellEachTurnFree,
        )],
        triggered_abilities: vec![on_exile_from_hand_or_by_you(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: spirit,
        })],
        ..creature(
            "Ranar the Ever-Watchful",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Spirit, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// Replicating Ring — snow artifact; {T}: any color; a night counter each
/// upkeep, and at eight they come off for eight Replicated Rings.
pub fn replicating_ring() -> CardDefinition {
    let ring = Arc::new(TokenDefinition {
        name: "Replicated Ring".into(),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Snow],
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        ..Default::default()
    });
    CardDefinition {
        name: "Replicating Ring",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Night, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Night },
                        Value::Const(8),
                    ),
                    then: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter {
                            what: Selector::This,
                            kind: CounterType::Night,
                            amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Night },
                        },
                        Effect::CreateToken { who: PlayerRef::You, count: Value::Const(8), definition: ring },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Spectral Deluge — return each opponent's creature with toughness at most
/// your Island count to its owner's hand. Foretell {1}{U}{U}.
pub fn spectral_deluge() -> CardDefinition {
    let islands = Value::CountOf(Box::new(yours(R::HasLandType(LandType::Island))));
    CardDefinition {
        foretell_cost: Some(cost(&[generic(1), u(), u()])),
        ..spell(
            "Spectral Deluge",
            cost(&[generic(4), u(), u()]),
            CardType::Sorcery,
            Effect::ForEach {
                selector: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                body: Box::new(Effect::If {
                    cond: Predicate::ValueAtLeast(islands, Value::ToughnessOf(Box::new(Selector::TriggerSource))),
                    then: Box::new(Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                    }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        )
    }
}

/// Stoic Farmer — enters: search for a basic Plains; onto the battlefield
/// tapped if an opponent has more lands, else to hand. Foretell {1}{W}.
pub fn stoic_farmer() -> CardDefinition {
    let plains = R::IsBasicLand.and(R::HasLandType(LandType::Plains));
    CardDefinition {
        foretell_cost: Some(cost(&[generic(1), w()])),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::OpponentControlsMoreLandsThanYou,
                then: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: plains.clone(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
                else_: Box::new(Effect::Search { who: PlayerRef::You, filter: plains, to: ZoneDest::Hand(PlayerRef::You) }),
            },
        }],
        ..creature("Stoic Farmer", cost(&[generic(3), w()]), vec![CreatureType::Dwarf, CreatureType::Peasant], 3, 3)
    }
}

/// Surtland Elementalist — reveal a Giant card or pay {2} to cast it;
/// attacking, you may cast an instant or sorcery from hand for free.
pub fn surtland_elementalist() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::RevealFromHandOrPay {
            filter: R::HasCreatureType(CreatureType::Giant),
            pay: 2,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CastFromHandWithoutPaying {
                filter: Some(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
            },
        }],
        ..creature(
            "Surtland Elementalist",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Giant, CreatureType::Wizard],
            8,
            8,
        )
    }
}

/// Tales of the Ancestors — each player draws up to the most cards in hand.
/// Foretell {1}{U}.
pub fn tales_of_the_ancestors() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[generic(1), u()])),
        ..spell(
            "Tales of the Ancestors",
            cost(&[generic(3), u()]),
            CardType::Sorcery,
            Effect::EachPlayerDoes {
                who: PlayerRef::EachPlayer,
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Diff(
                        Box::new(Value::HandSizeOf(PlayerRef::MostCardsInHand)),
                        Box::new(Value::HandSizeOf(PlayerRef::You)),
                    ),
                }),
            },
        )
    }
}

/// Warhorn Blast — your creatures get +2/+1 until end of turn. Foretell {2}{W}.
pub fn warhorn_blast() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[generic(2), w()])),
        ..spell(
            "Warhorn Blast",
            cost(&[generic(4), w()]),
            CardType::Instant,
            Effect::PumpPT {
                what: yours(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

