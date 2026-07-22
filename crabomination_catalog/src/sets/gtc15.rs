//! Gatecrash (GTC) wave 15: Gate-matters payoffs, combat-static beaters, and
//! the blue Primordial. Tests in `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    StaticAbility, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, StaticEffect};
use crate::effect::Predicate;
use crate::mana::{b, cost, g, generic, r, u, w, Color};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}

fn aura() -> Subtypes {
    Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() }
}

/// Alms Beast — {2}{W}{B} 6/6 Beast. Creatures blocking or blocked by it have
/// lifelink.
pub fn alms_beast() -> CardDefinition {
    CardDefinition {
        name: "Alms Beast",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Beast]),
        power: 6,
        toughness: 6,
        static_abilities: vec![StaticAbility {
            description: "Creatures blocking or blocked by this creature have lifelink.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::Permanent },
                applies_to: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Lifelink],
            },
        }],
        ..Default::default()
    }
}

/// Hold the Gates — {2}{W} Enchantment. Creatures you control get +0/+1 for
/// each Gate you control and have vigilance.
pub fn hold_the_gates() -> CardDefinition {
    CardDefinition {
        name: "Hold the Gates",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control get +0/+1 for each Gate you control.",
                effect: StaticEffect::PumpTeamByControlledPermanents {
                    applies_to: R::Creature.and(R::ControlledByYou),
                    count_filter: R::HasLandType(crate::card::LandType::Gate),
                    per_power: 0,
                    per_toughness: 1,
                    count_graveyard: false,
                },
            },
            StaticAbility {
                description: "Creatures you control have vigilance.",
                effect: StaticEffect::PumpTeamIf {
                    condition: Predicate::EntityMatches { what: Selector::This, filter: R::Permanent },
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::Vigilance],
                },
            },
        ],
        ..Default::default()
    }
}

/// Way of the Thief — {3}{U} Aura. Enchant creature; it gets +2/+2 and can't be
/// blocked as long as you control a Gate.
pub fn way_of_the_thief() -> CardDefinition {
    CardDefinition {
        name: "Way of the Thief",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 2, ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature can't be blocked as long as you control a Gate.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasLandType(crate::card::LandType::Gate).and(R::ControlledByYou),
                    ),
                    n: Value::Const(1),
                },
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Unblockable],
            },
        }],
        ..Default::default()
    }
}

/// Diluvian Primordial — {5}{U}{U} 5/5 Avatar, Flying. ETB: for each opponent,
/// you may cast up to one target instant or sorcery card from that player's
/// graveyard without paying its mana cost; exile it if it would leave the stack.
pub fn diluvian_primordial() -> CardDefinition {
    CardDefinition {
        name: "Diluvian Primordial",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Avatar]),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::HasCardType(CardType::Instant)
                .or(R::HasCardType(CardType::Sorcery))
                .and(R::InOpponentGraveyard),
            effect: Box::new(Effect::CastWithoutPayingImmediate {
                what: Selector::Target(0),
                source_zone: crate::card::Zone::Graveyard,
                exile_after: true,
                copy: false,
            }),
        })],
        ..Default::default()
    }
}

/// Five-Alarm Fire — {1}{R}{R} Enchantment. Whenever a creature you control
/// deals combat damage, put a blaze (charge) counter on this; remove five to
/// deal 5 damage to any target.
pub fn five_alarm_fire() -> CardDefinition {
    let gain_counter = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::YourControl),
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Charge,
            amount: Value::ONE,
        },
    };
    CardDefinition {
        name: "Five-Alarm Fire",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            gain_counter(EventKind::DealsCombatDamageToPlayer),
            gain_counter(EventKind::DealsCombatDamageToCreature),
        ],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Charge, 5)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(5) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Simic Manipulator — {1}{U}{U} 0/1 Mutant Wizard. Evolve; {T}, remove X +1/+1
/// counters from it: gain control of target creature with power ≤ X.
pub fn simic_manipulator() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Simic Manipulator",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Mutant, CreatureType::Wizard]),
        power: 0,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::evolve()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_x: Some(CounterType::PlusOnePlusOne),
            effect: Effect::GainControl {
                what: target_filtered(R::Creature.and(R::PowerAtMostXFromCost)),
                to: Some(PlayerRef::You),
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vizkopa Guildmage — {W}{B} 2/2 Human Wizard. {1}{W}{B}: target creature gains
/// lifelink until end of turn. {1}{W}{B}: until end of turn, whenever you gain
/// life, each opponent loses that much life.
pub fn vizkopa_guildmage() -> CardDefinition {
    use crate::effect::Duration;
    let wb = || cost(&[generic(1), w(), b()]);
    CardDefinition {
        name: "Vizkopa Guildmage",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: wb(),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: wb(),
                effect: Effect::WheneverYouGainLifeThisTurn {
                    body: Box::new(Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::TriggerEventAmount,
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Borborygmos Enraged — {4}{R}{R}{G}{G} 7/6 Legendary Cyclops. Trample; on
/// combat damage to a player, reveal top three, lands to hand, rest to gy.
/// Discard a land card: deal 3 damage to any target.
pub fn borborygmos_enraged() -> CardDefinition {
    CardDefinition {
        name: "Borborygmos Enraged",
        cost: cost(&[generic(4), r(), r(), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: creatures(vec![CreatureType::Cyclops]),
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::RevealTopTakeMatchingRestToGraveyard {
                who: PlayerRef::You,
                count: Value::Const(3),
                filter: R::Land,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Land, 1)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mystic Genesis — {2}{G}{U}{U} Instant. Counter target spell; create an X/X
/// green Ooze token, where X is that spell's mana value.
pub fn mystic_genesis() -> CardDefinition {
    let mv = || Value::ManaValueOf(Box::new(Selector::Target(0)));
    CardDefinition {
        name: "Mystic Genesis",
        cost: cost(&[generic(2), g(), u(), u()]),
        card_types: vec![CardType::Instant],
        // Mint the Ooze first (reads the still-on-stack spell's MV), then counter.
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Ooze".into(),
                    power: 0,
                    toughness: 0,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: creatures(vec![CreatureType::Ooze]),
                    dynamic_pt: Some((mv(), mv())),
                    ..Default::default()
                },
            },
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
        ]),
        ..Default::default()
    }
}

/// Duskmantle Guildmage — {U}{B} 2/2 Human Wizard. {1}{U}{B}: until end of turn,
/// whenever a card is put into an opponent's graveyard, that player loses 1 life.
/// {2}{U}{B}: target player mills two cards.
pub fn duskmantle_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Duskmantle Guildmage",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u(), b()]),
                effect: Effect::WheneverCardEntersOpponentGraveyardThisTurn {
                    body: Box::new(Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::ONE,
                    }),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u(), b()]),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Armored Transport — {3} 2/1 Construct. Prevent all combat damage that would
/// be dealt to it by creatures blocking it.
pub fn armored_transport() -> CardDefinition {
    CardDefinition {
        name: "Armored Transport",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creatures(vec![CreatureType::Construct]),
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to this creature by creatures blocking it.",
            effect: StaticEffect::PreventAllCombatDamageToThisFromBlockers,
        }],
        ..Default::default()
    }
}

/// Tin Street Market — {4}{R} Aura. Enchant land; it gains "{T}, Discard a card:
/// Draw a card."
pub fn tin_street_market() -> CardDefinition {
    CardDefinition {
        name: "Tin Street Market",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}, Discard a card: Draw a card.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    discard_cost: Some((R::Any, 1)),
                    effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

