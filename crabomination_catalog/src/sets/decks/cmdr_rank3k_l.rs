//! Commander: two-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), twelfth file. Tests in `tests/recent_b/cmdr_rank3k_l.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, Predicate,
    SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, u};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: n,
        definition: Arc::new(def),
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

// ── Melira, Sylvok Outcast ───────────────────────────────────────────────────

/// Witherscale Wurm — {4}{G}{G} 9/9. Whenever it blocks or becomes blocked by
/// a creature, that creature gains wither until end of turn. Whenever it deals
/// damage to an opponent, remove all -1/-1 counters from it.
pub fn witherscale_wurm() -> CardDefinition {
    let wither = |what| Effect::GrantKeyword {
        what,
        keyword: Keyword::Wither,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: wither(Selector::BlockedAttacker),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: wither(Selector::BlockingCreatures),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource)
                    .with_filter(Predicate::PlayerIsOpponent {
                        who: PlayerRef::TriggerEventPlayer,
                    }),
                effect: Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::MinusOneMinusOne,
                    },
                },
            },
        ],
        ..creature(
            "Witherscale Wurm",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Wurm],
            9,
            9,
        )
    }
}

/// Vivien on the Hunt — {4}{G}{G} planeswalker, loyalty 4. +2: you may
/// sacrifice a creature; if you do, search for a creature card with mana value
/// one more and put it onto the battlefield. +1: mill five, then put any number
/// of creature cards milled this way into your hand. −1: a 4/4 green Rhino
/// Warrior.
pub fn vivien_on_the_hunt() -> CardDefinition {
    let loyalty = |cost: i32, effect: Effect| crate::card::LoyaltyAbility {
        loyalty_cost: cost,
        effect,
        ..Default::default()
    };
    let rhino = TokenDefinition {
        name: "Rhino Warrior".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Vivien],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            loyalty(
                2,
                Effect::MaySacrifice {
                    description:
                        "Sacrifice a creature to search for one with mana value one higher?".into(),
                    filter: R::Creature,
                    count: Value::ONE,
                    then: Box::new(Effect::Search {
                        who: PlayerRef::You,
                        filter: R::Creature.and(R::ManaValueEqualsSacrificedPlus(1)),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    }),
                    else_: None,
                },
            ),
            loyalty(
                1,
                Effect::MillThenToHandN {
                    amount: Value::Const(5),
                    filter: R::Creature,
                    take: Value::Const(5),
                    otherwise: None,
                },
            ),
            loyalty(-1, make(Value::ONE, rhino)),
        ],
        ..spell(
            "Vivien on the Hunt",
            cost(&[generic(4), g(), g()]),
            CardType::Planeswalker,
            Effect::Noop,
        )
    }
}

// ── Rona, Disciple of Gix ────────────────────────────────────────────────────

fn while_rona() -> MayPlayDuration {
    MayPlayDuration::WhileSourceOnBattlefield {
        source: crate::card::CardId(0),
        one_spell_a_turn: false,
    }
}

/// Rona, Disciple of Gix — {1}{U}{B} legendary 2/2 Human Artificer. ETB: you
/// may exile target historic card from your graveyard. You may cast spells
/// from among cards exiled with Rona. {4}, {T}: exile the top card of your
/// library. (Residual: a land exiled with Rona can be played too.)
pub fn rona_disciple_of_gix() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::Move {
                    what: target_filtered(R::historic().from_your_graveyard()),
                    to: ZoneDest::ExileWithSourceStamp,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: while_rona(),
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
                // "You may cast SPELLS from among cards exiled with Rona."
                Effect::RestrictMayPlayToCasting { what: Selector::LastMoved },
            ])),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: while_rona(),
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
                Effect::RestrictMayPlayToCasting { what: Selector::ExiledThisResolution { filter: R::Any } },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Rona, Disciple of Gix",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            2,
        )
    }
}

/// The Antiquities War — {3}{U} Saga. I, II: look at the top five; you may
/// reveal an artifact card and put it into your hand; the rest on the bottom
/// in a random order. III: artifacts you control become artifact creatures
/// with base power and toughness 5/5 until end of turn.
pub fn the_antiquities_war() -> CardDefinition {
    let dig = || {
        Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(5),
            pick_filter: Some(R::Artifact),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        }))
    };
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (1, dig()),
            (2, dig()),
            (
                3,
                Effect::BecomeCreature {
                    what: yours(R::Artifact),
                    power: Value::Const(5),
                    toughness: Value::Const(5),
                    creature_types: vec![],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
        ..spell(
            "The Antiquities War",
            cost(&[generic(3), u()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}
