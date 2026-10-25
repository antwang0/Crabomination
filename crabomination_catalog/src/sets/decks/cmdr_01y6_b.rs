//! Commander: 2026-10-25 (`01Y6ZRFx`) — the Irini Sengir deck's closers, the
//! anti-green / anti-white black cards. Tests in `tests/recent_b/cmdr_01y6_b.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{declare_target_opponent, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, generic};

fn green_or_white() -> R {
    R::HasColor(Color::Green).or(R::HasColor(Color::White))
}

fn creature(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn enchantment(name: &'static str, mana: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// Dunerider Outlaw — {B}{B} 1/1 Human Rebel Rogue, protection from green.
/// At the beginning of each end step, if it dealt damage to an opponent this
/// turn, put a +1/+1 counter on it (CR 603.4; read as "a player").
pub fn dunerider_outlaw() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Green)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::DamagedAPlayerThisTurn }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature(
            "Dunerider Outlaw",
            cost(&[b(), b()]),
            vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Rogue],
            1,
            1,
        )
    }
}

/// Knight of Stromgald — {B}{B} 2/1 Human Knight, protection from white.
/// {B}: first strike until end of turn. {B}{B}: +1/+0 until end of turn.
pub fn knight_of_stromgald() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::White)],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b(), b()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature("Knight of Stromgald", cost(&[b(), b()]), vec![CreatureType::Human, CreatureType::Knight], 2, 1)
    }
}

/// Lifebane Zombie — {1}{B}{B} 3/1 Zombie Warrior, intimidate. ETB: target
/// opponent reveals their hand; you exile a green or white creature card
/// from it.
pub fn lifebane_zombie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Intimidate],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                declare_target_opponent(),
                Effect::RevealHand { who: PlayerRef::Target(0) },
                Effect::ExileChosenFromHand {
                    from: Selector::Player(PlayerRef::Target(0)),
                    count: Value::ONE,
                    filter: R::Creature.and(green_or_white()),
                    link_to_source: false,
                    face_down: false,
                },
            ]),
        }],
        ..creature("Lifebane Zombie", cost(&[generic(1), b(), b()]), vec![CreatureType::Zombie, CreatureType::Warrior], 3, 1)
    }
}

/// Curse of Wizardry — {2}{B}{B} enchantment. As it enters, choose a color.
/// Whenever a player casts a spell of the chosen color, that player loses 1
/// life.
pub fn curse_of_wizardry() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellSharesChosenColorOfSource),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::Triggerer), amount: Value::ONE },
        }],
        ..enchantment("Curse of Wizardry", cost(&[generic(2), b(), b()]))
    }
}

/// Dystopia — {1}{B}{B} enchantment, cumulative upkeep—pay 1 life. At the
/// beginning of each player's upkeep, that player sacrifices a green or white
/// permanent of their choice.
pub fn dystopia() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Life(1))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::ActivePlayer),
                count: Value::ONE,
                filter: R::Permanent.and(green_or_white()),
            },
        }],
        ..enchantment("Dystopia", cost(&[generic(1), b(), b()]))
    }
}

/// Gloom — {2}{B} enchantment. White spells cost {3} more to cast; activated
/// abilities of white enchantments cost {3} more to activate.
pub fn gloom() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "White spells cost {3} more to cast.",
                effect: StaticEffect::AdditionalCost { filter: R::HasColor(Color::White), amount: 3 },
            },
            StaticAbility {
                description: "Activated abilities of white enchantments cost {3} more to activate.",
                effect: StaticEffect::FilteredActivationTax {
                    filter: R::HasCardType(CardType::Enchantment).and(R::HasColor(Color::White)),
                    amount: 3,
                },
            },
        ],
        ..enchantment("Gloom", cost(&[generic(2), b()]))
    }
}

/// Leshrac's Sigil — {B}{B} enchantment. Whenever an opponent casts a green
/// spell, you may pay {B}{B} to look at their hand and make them discard a
/// card of your choice. {B}{B}: return it to its owner's hand.
pub fn leshracs_sigil() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasColor(Color::Green) },
            ),
            effect: Effect::MayPay {
                description: "Pay {B}{B} to make them discard a card of your choice?".into(),
                mana_cost: cost(&[b(), b()]),
                body: Box::new(Effect::DiscardChosen {
                    from: Selector::Player(PlayerRef::Triggerer),
                    count: Value::ONE,
                    filter: R::Any,
                }),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b()]),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            ..Default::default()
        }],
        ..enchantment("Leshrac's Sigil", cost(&[b(), b()]))
    }
}

/// Tourach's Chant — {1}{B}{B} enchantment. Upkeep: sacrifice it unless you
/// pay {B}. Whenever a player puts a Forest onto the battlefield, it deals 3
/// damage to that player unless they put a -1/-1 counter on a creature they
/// control.
pub fn tourachs_chant() -> CardDefinition {
    let that_player = || PlayerRef::ControllerOf(Box::new(Selector::TriggerSource));
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[b()]) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasLandType(LandType::Forest) },
                ),
                effect: Effect::UnlessPlayerPays {
                    who: that_player(),
                    cost: WardCost::Blight(1),
                    then: Box::new(Effect::DealDamage {
                        to: Selector::Player(that_player()),
                        amount: Value::Const(3),
                    }),
                    if_paid: None,
                },
            },
        ],
        ..enchantment("Tourach's Chant", cost(&[generic(1), b(), b()]))
    }
}

/// Noxious Grasp — {1}{B} instant. Destroy target green or white creature or
/// planeswalker; you gain 1 life.
pub fn noxious_grasp() -> CardDefinition {
    CardDefinition {
        name: "Noxious Grasp",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature.or(R::Planeswalker).and(green_or_white())) },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Virtue's Ruin — {2}{B} sorcery. Destroy all white creatures.
pub fn virtues_ruin() -> CardDefinition {
    CardDefinition {
        name: "Virtue's Ruin",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::White))) },
        ..Default::default()
    }
}
