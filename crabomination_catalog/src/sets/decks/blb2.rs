//! Bloomburrow and Duskmourn gap batch — the Gift artifact and sorcery, the
//! Bat/Bird legends, Grievous Wound and The Jolly Balloon Man. Tests in
//! `tests/recent_b/blb2.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, Gift, Keyword, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, PlayerStaticTarget, Predicate, Selector,
    ZoneDest,
};
use crate::mana::{Color, ManaCost, b, cost, generic, r, u, w};

fn legend(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// A tapped 1/1 blue Fish — the Bloomburrow gift token.
fn tapped_fish() -> TokenDefinition {
    TokenDefinition {
        name: "Fish".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish],
            ..Default::default()
        },
        tapped: true,
        ..Default::default()
    }
}

/// Starforged Sword — {4} Equipment. Gift a tapped Fish; if promised it
/// straps itself on. Equipped creature gets +3/+3 and loses flying.
pub fn starforged_sword() -> CardDefinition {
    CardDefinition {
        name: "Starforged Sword",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            remove_keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        gift: Some(Box::new(Gift {
            label: "a tapped Fish",
            gifted_effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::EachOpponent,
                    count: Value::ONE,
                    definition: Box::new(tapped_fish()),
                },
                Effect::AttachSourceTo {
                    host: target_filtered(R::Creature.and(R::ControlledByYou)),
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Cruelclaw's Heist — {B}{B} Sorcery. Gift a card. Strip a nonland card from
/// an opponent's hand; the gift buys you the right to cast it.
pub fn cruelclaws_heist() -> CardDefinition {
    let strip = || Effect::ExileChosenFromHand {
        from: Selector::Player(PlayerRef::Target(0)),
        count: Value::ONE,
        filter: R::Nonland,
        link_to_source: true,
        face_down: false,
    };
    CardDefinition {
        name: "Cruelclaw's Heist",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: strip(),
        gift: Some(Box::new(Gift {
            label: "a card",
            gifted_effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
                strip(),
                Effect::GrantMayPlay {
                    what: Selector::CardExiledWithSource,
                    duration: crate::card::MayPlayDuration::WhileExiled,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: true,
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Grievous Wound — {3}{B}{B} Aura. Enchanted player can't gain life and
/// halves on every hit.
pub fn grievous_wound() -> CardDefinition {
    CardDefinition {
        name: "Grievous Wound",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted player can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife {
                target: PlayerStaticTarget::EnchantedPlayer,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::AnyPlayer)
                .with_filter(Predicate::SamePlayer(
                    PlayerRef::TriggerEventPlayer,
                    PlayerRef::EnchantedPlayer,
                )),
            effect: Effect::LoseHalfLife {
                who: Selector::Player(PlayerRef::EnchantedPlayer),
                rounded_up: true,
            },
        }],
        ..Default::default()
    }
}

/// The Jolly Balloon Man — {1}{R}{W} 1/4 haste Clown. {1}, {T}: blow up a
/// creature you control into a 1/1 flying hasty Balloon copy for the turn.
pub fn the_jolly_balloon_man() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    },
                    extra_creature_types: vec![CreatureType::Balloon],
                    extra_card_types: Vec::new(),
                    override_pt: Some((1, 1)),
                    override_colors: Some(vec![Color::Red]),
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![Keyword::Flying, Keyword::Haste],
                },
                Effect::SacrificeLastCreatedTokensAtNextEndStep,
            ]),
            ..Default::default()
        }],
        ..legend(
            "The Jolly Balloon Man",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Human, CreatureType::Clown],
            1,
            4,
        )
    }
}



/// Zoraline, Cosmos Caller — {1}{W}{B} 3/3 flying vigilance Bat Cleric. Bats
/// drain, and she buys back a cheap permanent whenever she enters or attacks.
pub fn zoraline_cosmos_caller() -> CardDefinition {
    let rebuy = || Effect::MayPay {
        description: "Pay {W}{B} and 2 life to reanimate?".into(),
        mana_cost: cost(&[w(), b()]),
        body: Box::new(Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::PermanentCard
                        .and(R::ManaValueAtMost(3))
                        .and(R::InYourGraveyard),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: CounterType::Finality,
                amount: Value::ONE,
            },
        ])),
        else_: None,
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Bat).and(R::ControlledByYou),
                    },
                ),
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            },
            etb(rebuy()),
            on_attack(rebuy()),
        ],
        ..legend(
            "Zoraline, Cosmos Caller",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Bat, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Kastral, the Windcrested — {3}{W}{U} 4/5 flier. Bird combat damage buys a
/// Bird, a team pump, or a card.
pub fn kastral_the_windcrested() -> CardDefinition {
    let birds = || R::HasCreatureType(CreatureType::Bird).and(R::ControlledByYou);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Bird),
                }),
            effect: Effect::ChooseMode(vec![
                Effect::MayDo {
                    description: "Put a Bird onto the battlefield?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::SearchZones {
                            who: PlayerRef::You,
                            zones: vec![crate::card::Zone::Hand, crate::card::Zone::Graveyard],
                            filter: R::Creature.and(R::HasCreatureType(CreatureType::Bird)),
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::You,
                                tapped: false,
                            },
                        },
                        Effect::AddCounter {
                            what: Selector::LastMoved,
                            kind: CounterType::Finality,
                            amount: Value::ONE,
                        },
                    ])),
                },
                Effect::AddCounter {
                    what: Selector::EachPermanent(birds()),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..legend(
            "Kastral, the Windcrested",
            cost(&[generic(3), w(), u()]),
            vec![CreatureType::Bird, CreatureType::Scout],
            4,
            5,
        )
    }
}
