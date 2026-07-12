//! A Wilds of Eldraine (WOE) wave of fresh cards: Rat aristocrats, enchantment-
//! matters value, Bargain/Celebration payoffs, and removal. All ride existing
//! primitives. Tests in `crabomination/src/tests/recent139.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, MayPlayDuration,
    Predicate, SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{deal, etb, on_attack, target, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, ZoneDest, ZoneRef,
};
use crate::game::effects::treasure_token;
use crate::mana::{b, cost, generic, r, u, x, Color};

use super::woe_roles::wicked_role;

/// 1/1 black Rat token with "This token can't block."
fn rat_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    }
}

/// Every Rat you control (layer-agnostic team selector).
fn your_rats() -> Selector {
    Selector::EachMatching {
        zone: ZoneRef::Battlefield,
        filter: R::HasCreatureType(CreatureType::Rat).and(R::ControlledByYou),
    }
}

// ── White/Blue ────────────────────────────────────────────────────────────────

/// Misleading Motes — {3}{U} Instant. Target creature's owner puts it on their
/// choice of the top or bottom of their library.
pub fn misleading_motes() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Misleading Motes",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(R::Creature),
            to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::OwnerChoice },
        },
        ..Default::default()
    }
}

// ── Black ─────────────────────────────────────────────────────────────────────

/// Taken by Nightmares — {2}{B}{B} Instant. Exile target creature. If you
/// control an enchantment, scry 2.
pub fn taken_by_nightmares() -> CardDefinition {
    CardDefinition {
        name: "Taken by Nightmares",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Creature) },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Enchantment.and(R::ControlledByYou)),
                    n: Value::ONE,
                },
                then: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Faerie Fencing — {X}{B} Instant. Target creature gets -X/-X until end of
/// turn, plus an additional -3/-3 if you control a Faerie.
pub fn faerie_fencing() -> CardDefinition {
    CardDefinition {
        name: "Faerie Fencing",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                toughness: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Faerie).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(-3),
                    toughness: Value::Const(-3),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Shatter the Oath — {3}{B}{B} Sorcery. Destroy target creature or enchantment,
/// then create a Wicked Role attached to a creature you control.
/// (The optional second target is auto-picked — the highest-power creature you
/// control receives the Role.)
pub fn shatter_the_oath() -> CardDefinition {
    CardDefinition {
        name: "Shatter the Oath",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature.or(R::Enchantment)) },
            Effect::CreateTokenAttachedTo {
                target: Selector::take(
                    Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    Value::ONE,
                ),
                definition: wicked_role(),
            },
        ]),
        ..Default::default()
    }
}

/// Lord Skitter's Blessing — {1}{B} Enchantment. ETB: create a Wicked Role
/// attached to target creature you control. At your draw step, if you control an
/// enchanted creature, lose 1 life and draw an additional card.
pub fn lord_skitters_blessing() -> CardDefinition {
    CardDefinition {
        name: "Lord Skitter's Blessing",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateTokenAttachedTo {
                target: target_filtered(R::Creature.and(R::ControlledByYou)),
                definition: wicked_role(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Draw), EventScope::ActivePlayer)
                    .with_filter(Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(R::Creature.and(R::IsEnchanted).and(R::ControlledByYou)),
                        n: Value::ONE,
                    }),
                effect: Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
            },
        ],
        ..Default::default()
    }
}

// ── Red ──────────────────────────────────────────────────────────────────────

/// Flick a Coin — {2}{R} Instant. Deals 1 damage to any target, create a
/// Treasure, then draw a card.
pub fn flick_a_coin() -> CardDefinition {
    CardDefinition {
        name: "Flick a Coin",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            deal(1, target()),
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: treasure_token() },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Frantic Firebolt — {2}{R} Instant. Deals X damage to target creature, where
/// X is 2 plus the number of instant and sorcery cards in your graveyard.
/// (The "…and/or have an Adventure" graveyard clause is approximated away.)
pub fn frantic_firebolt() -> CardDefinition {
    CardDefinition {
        name: "Frantic Firebolt",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::Sum(vec![
                Value::Const(2),
                Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                },
            ]),
        },
        ..Default::default()
    }
}

/// Ogre Chitterlord — {4}{R}{R} 6/5 Ogre Warrior with menace. Whenever it enters
/// or attacks, create two Rats; then if you control five or more Rats, each Rat
/// you control gets +2/+0 until end of turn.
pub fn ogre_chitterlord() -> CardDefinition {
    let body = || {
        Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: rat_token() },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast { sel: your_rats(), n: Value::Const(5) },
                then: Box::new(Effect::PumpPT {
                    what: your_rats(),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ])
    };
    CardDefinition {
        name: "Ogre Chitterlord",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(body()), on_attack(body())],
        ..Default::default()
    }
}

/// Redcap Gutter-Dweller — {2}{R}{R} 3/3 Goblin Warrior with menace. ETB: create
/// two Rats. At your upkeep, you may sacrifice another creature; if you do, put
/// a +1/+1 counter on it and exile the top card of your library to play this turn.
pub fn redcap_gutter_dweller() -> CardDefinition {
    CardDefinition {
        name: "Redcap Gutter-Dweller",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: rat_token() }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), EventScope::ActivePlayer),
                effect: Effect::MaySacrifice {
                    description: "sacrifice another creature".into(),
                    filter: R::Creature.and(R::OtherThanSource),
                    count: Value::ONE,
                    then: Box::new(Effect::Seq(vec![
                        Effect::AddCounter {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                        Effect::ExileTopAndGrantMayPlay {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            duration: MayPlayDuration::EndOfThisTurn,
                            pay_any_color: false, pay_own_cost: false,
                            uncast_penalty: None,
                        },
                    ])),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Tattered Ratter — {1}{R} 2/2 Human Peasant. Whenever a Rat you control
/// becomes blocked, it gets +2/+0 until end of turn.
pub fn tattered_ratter() -> CardDefinition {
    CardDefinition {
        name: "Tattered Ratter",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Rat),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

// ── Green ─────────────────────────────────────────────────────────────────────

/// Redtooth Vanguard — {1}{G} 3/1 Elf Warrior with trample. Whenever an
/// enchantment you control enters, you may pay {2} to return this card from your
/// graveyard to your hand.
pub fn redtooth_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Redtooth Vanguard",
        cost: cost(&[generic(1), crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::FromYourGraveyard)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment.and(R::ControlledByYou),
                }),
            effect: Effect::MayPay {
                description: "return Redtooth Vanguard from your graveyard to your hand".into(),
                mana_cost: crate::mana::cost(&[generic(2)]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}
