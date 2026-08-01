//! Mercadian Masques (MMQ) gap closure, sixth (final) wave. Tests in
//! `classic_sets/mmq6`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, SelectionRequirement as R, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::{Effect, ManaPayload, PlayerRef, Selector, shortcut::target_filtered};
use crate::game::TurnStep;
use crate::mana::{ManaCost, SpendRestriction, cost, g, generic, r, u, w};

fn artifact(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// Spellshaper body: 1/1 Human Spellshaper whose one ability discards a card.
fn spellshaper(name: &'static str, c: ManaCost, ability: ActivatedAbility) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Spellshaper],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            ..ability
        }],
        ..Default::default()
    }
}

/// Charm Peddler — {W} Spellshaper. Shields target creature from the next
/// damage a chosen source would deal it.
pub fn charm_peddler() -> CardDefinition {
    spellshaper(
        "Charm Peddler",
        cost(&[w()]),
        ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: Some(target_filtered(R::Creature)),
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
            },
            ..Default::default()
        },
    )
}

/// Cho-Arrim Alchemist — {W} Spellshaper. Soaks the next hit from a chosen
/// source and converts it to life.
pub fn cho_arrim_alchemist() -> CardDefinition {
    spellshaper(
        "Cho-Arrim Alchemist",
        cost(&[w()]),
        ActivatedAbility {
            mana_cost: cost(&[generic(1), w(), w()]),
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: None,
                gain_life: true,
                redirect_to: None,
                whole_turn: false,
            },
            ..Default::default()
        },
    )
}

/// General's Regalia — {3}. Bounces the next hit from a chosen source onto a
/// creature you control.
pub fn generals_regalia() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: Some(target_filtered(R::And(
                    Box::new(R::Creature),
                    Box::new(R::ControlledByYou),
                ))),
                whole_turn: false,
            },
            ..Default::default()
        }],
        ..artifact("General's Regalia", cost(&[generic(3)]))
    }
}

/// Bargaining Table — {5}. Draws a card for the price of an opponent's hand.
pub fn bargaining_table() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            generic_cost_value: Some(Value::CardsInHandMatching {
                who: PlayerRef::EachOpponent,
                filter: R::Any,
            }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..artifact("Bargaining Table", cost(&[generic(5)]))
    }
}

/// Food Chain — {2}{G}. Eat a creature, get its mana value plus one back,
/// spendable only on creature spells.
pub fn food_chain() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            exile_permanent_cost: Some((R::Creature, 1)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyOneColor(Value::Sum(vec![
                        Value::ONE,
                        Value::ExiledForCostManaValue,
                    ]))),
                    SpendRestriction::CreatureOnly,
                ),
            },
            ..Default::default()
        }],
        ..enchantment("Food Chain", cost(&[generic(2), g()]))
    }
}

/// Caller of the Hunt — {2}{G}. A `*/*` that counts the creature type named
/// as it enters (the printed choice is an additional cast cost).
pub fn caller_of_the_hunt() -> CardDefinition {
    CardDefinition {
        name: "Caller of the Hunt",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        dynamic_pt: Some(DynamicPt::CreaturesOfSourceChosenType),
        ..Default::default()
    }
}

/// Charisma — {U}{U}{U} Aura. Anything the host bites changes sides for as
/// long as Charisma sticks around.
pub fn charisma() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::EnchantedBySource),
            effect: Effect::GainControlWhileSourceRemains { what: Selector::Target(0) },
        }],
        ..enchantment("Charisma", cost(&[u(), u(), u()]))
    }
}

/// Blood Oath — {3}{R}. Name a card type; three damage per copy in the
/// victim's hand.
pub fn blood_oath() -> CardDefinition {
    CardDefinition {
        name: "Blood Oath",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseCardTypeRevealHandDamage {
            who: PlayerRef::Target(0),
            per: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Crooked Scales — {4}. Keep flipping (and paying) until the coin picks a
/// victim.
pub fn crooked_scales() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4)]),
            effect: Effect::CoinFlipDestroyLoop {
                win: Selector::Target(0),
                lose: Selector::Target(1),
                repeat_cost: cost(&[generic(3)]),
            },
            ..Default::default()
        }],
        ..artifact("Crooked Scales", cost(&[generic(4)]))
    }
}

/// Kyren Archive — {3}. Banks library cards face down; cash the whole stash
/// into hand for {5} and your hand.
pub fn kyren_archive() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Book],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Exile the top card of your library face down?".into(),
                body: Box::new(Effect::ExileTopOfLibrary {
                    who: Selector::You,
                    amount: Value::ONE,
                    link_to_source: true,
                    face_down: true,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            discard_hand_cost: true,
            sac_cost: true,
            effect: Effect::ReturnLinkedExilesToHand,
            ..Default::default()
        }],
        ..artifact("Kyren Archive", cost(&[generic(3)]))
    }
}

/// Mercadian Lift — {2}. Crank winch counters, then cash X of them in for a
/// creature with mana value X out of hand.
pub fn mercadian_lift() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Winch,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_x: Some(CounterType::Winch),
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::And(
                        Box::new(R::Creature),
                        Box::new(R::ManaValueExactlyXFromCost),
                    ),
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                },
                ..Default::default()
            },
        ],
        ..artifact("Mercadian Lift", cost(&[generic(2)]))
    }
}

/// Spiritual Focus — {1}{W}. Turns an opponent's discard spells into life and
/// cards.
pub fn spiritual_focus() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::OpponentCausedYouToDiscard,
                EventScope::YourControl,
            ),
            effect: Effect::seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                Effect::MayDo {
                    description: "Draw a card?".into(),
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            ]),
        }],
        ..enchantment("Spiritual Focus", cost(&[generic(1), w()]))
    }
}

/// Thieves' Auction — {4}{R}{R}{R}. Everything goes into the pot and gets
/// drafted back out, tapped.
pub fn thieves_auction() -> CardDefinition {
    CardDefinition {
        name: "Thieves' Auction",
        cost: cost(&[generic(4), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ThievesAuction,
        ..Default::default()
    }
}
