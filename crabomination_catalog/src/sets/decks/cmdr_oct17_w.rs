//! Commander: 2026-10-17 (`3chree`) closers, seventh batch — one-deck closers
//! from the EDHREC long tail. Tests in `tests/recent_b/cmdr_oct17_w.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility,
    PlaneswalkerSubtype, Predicate, SelectionRequirement as R, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{
    cast_is_instant_or_sorcery, dash, declare_target_opponent, each_opponent,
    each_opponent_creature, encore, etb, if_discarded, on_attack, prowess, raid_etb, target_any,
    target_filtered,
};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, ManaSymbol, b, cost, g, generic, r, u, w};
use std::sync::Arc;

use crabomination_base::tokens::{food_token, treasure_token};

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

/// Keep Out — {1}{W} instant. Choose one: 4 damage to target tapped
/// creature; or destroy target enchantment.
pub fn keep_out() -> CardDefinition {
    spell(
        "Keep Out",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::Tapped)),
                amount: Value::Const(4),
            },
            Effect::Destroy {
                what: target_filtered(R::Enchantment),
            },
        ]),
    )
}

/// Knockout Blow — {2}{W} instant, costs {2} less if it targets a red
/// creature. 4 damage to target attacking or blocking creature; gain 2 life.
pub fn knockout_blow() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if_target: Some((R::HasColor(Color::Red), 2)),
        ..spell(
            "Knockout Blow",
            cost(&[generic(2), w()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                    amount: Value::Const(4),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
        )
    }
}

/// Ghoulcaller's Chant — {B} sorcery. Choose one: return target creature card
/// from your graveyard to your hand; or return two target Zombie cards.
pub fn ghoulcallers_chant() -> CardDefinition {
    spell(
        "Ghoulcaller's Chant",
        cost(&[b()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(R::Creature.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 2,
                filter: R::HasCreatureType(CreatureType::Zombie).from_your_graveyard(),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        ]),
    )
}

/// Voracious Wurm — {1}{G} 2/2 Wurm. Enters with X +1/+1 counters, X = life
/// you've gained this turn.
pub fn voracious_wurm() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::LifeGainedThisTurn(PlayerRef::You),
        )),
        ..creature(
            "Voracious Wurm",
            cost(&[generic(1), g()]),
            vec![CreatureType::Wurm],
            2,
            2,
        )
    }
}

/// Mouth of Ronom — snow land. {T}: {C}. {4}{S}, {T}, sacrifice it: 4 damage
/// to target creature.
pub fn mouth_of_ronom() -> CardDefinition {
    CardDefinition {
        name: "Mouth of Ronom",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Snow],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: ManaCost {
                    symbols: vec![generic(4), ManaSymbol::Snow],
                },
                tap_cost: true,
                sac_cost: true,
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature),
                    amount: Value::Const(4),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Oath of the Grey Host — {3}{B} Saga. I: you and target opponent each
/// create a Food. II: each opponent loses 3; Treasure. III: three tapped 1/1
/// white flying Spirits.
pub fn oath_of_the_grey_host() -> CardDefinition {
    let food = Arc::new(food_token());
    let spirit = Arc::new(TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        tapped: true,
        ..Default::default()
    });
    CardDefinition {
        name: "Oath of the Grey Host",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    declare_target_opponent(),
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: food.clone(),
                    },
                    Effect::CreateToken {
                        who: PlayerRef::Target(0),
                        count: Value::ONE,
                        definition: food,
                    },
                ]),
            ),
            (
                2,
                Effect::Seq(vec![
                    Effect::LoseLife {
                        who: each_opponent(),
                        amount: Value::Const(3),
                    },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(treasure_token()),
                    },
                ]),
            ),
            (
                3,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                    definition: spirit,
                },
            ),
        ],
        ..Default::default()
    }
}

/// Sarkhan, Dragonsoul — {4}{R}{R} loyalty 5. +2: 1 damage to each opponent
/// and each creature they control. −3: 4 damage to target player or
/// planeswalker. −9: any number of Dragon creature cards onto the battlefield.
pub fn sarkhan_dragonsoul() -> CardDefinition {
    CardDefinition {
        name: "Sarkhan, Dragonsoul",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Sarkhan],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: each_opponent(),
                        amount: Value::ONE,
                    },
                    Effect::DealDamage {
                        to: each_opponent_creature(),
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: Value::Const(4),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -9,
                effect: Effect::SearchAnyNumber {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Dragon)),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}


/// Ral, Izzet Viceroy — {3}{U}{R} loyalty 5. +1: look at top two, one to hand,
/// other to graveyard. −3: damage to target creature = instant/sorcery cards
/// you own in exile and graveyard. −8: emblem, instant/sorcery cast: 4 damage
/// to any target and draw two.
pub fn ral_izzet_viceroy() -> CardDefinition {
    let spells = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Ral, Izzet Viceroy",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Ral],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::LookPickToHand(Box::new(LookPick {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    rest_to_graveyard: true,
                    ..Default::default()
                })),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature),
                    amount: Value::Sum(vec![
                        Value::CardsInExileOwnedMatching {
                            who: PlayerRef::You,
                            filter: spells(),
                        },
                        Value::CardsInGraveyardMatching {
                            who: PlayerRef::You,
                            filter: spells(),
                        },
                    ]),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Ral, Izzet Viceroy".into(),
                    statics: vec![],
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                            .with_filter(cast_is_instant_or_sorcery()),
                        effect: Effect::Seq(vec![
                            Effect::DealDamage {
                                to: target_any(),
                                amount: Value::Const(4),
                            },
                            Effect::Draw {
                                who: Selector::You,
                                amount: Value::Const(2),
                            },
                        ]),
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Molting Snakeskin — {B} Aura. Enchanted creature gets +2/+0 and has
/// "{2}{B}: Regenerate this creature."
pub fn molting_snakeskin() -> CardDefinition {
    CardDefinition {
        name: "Molting Snakeskin",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                effect: Effect::Regenerate {
                    what: Selector::This,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Briarblade Adept — {4}{B} 3/4 Elf Assassin. Attacks: target creature an
/// opponent controls gets -1/-1 until end of turn. Encore {3}{B}.
pub fn briarblade_adept() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![encore(cost(&[generic(3), b()]))],
        ..creature(
            "Briarblade Adept",
            cost(&[generic(4), b()]),
            vec![CreatureType::Elf, CreatureType::Assassin],
            3,
            4,
        )
    }
}

/// Shadowed Caravel — {2} 2/2 Vehicle, crew 2. Whenever a creature you
/// control explores, put a +1/+1 counter on it.
pub fn shadowed_caravel() -> CardDefinition {
    CardDefinition {
        name: "Shadowed Caravel",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Explored, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Bounty of Might — {4}{G}{G} instant. Three separate "target creature gets
/// +3/+3 until end of turn" clauses (the same creature may be chosen for each).
pub fn bounty_of_might() -> CardDefinition {
    let pump = |slot: u8| Effect::PumpPT {
        what: Selector::TargetFiltered {
            slot,
            filter: R::Creature,
        },
        power: Value::Const(3),
        toughness: Value::Const(3),
        duration: Duration::EndOfTurn,
    };
    spell(
        "Bounty of Might",
        cost(&[generic(4), g(), g()]),
        CardType::Instant,
        Effect::Seq(vec![pump(0), pump(1), pump(2)]),
    )
}

/// Scout the Wilderness — {2}{G} sorcery, kicker {1}{W}. Basic land onto the
/// battlefield tapped; if kicked, two 1/1 white Soldier tokens.
pub fn scout_the_wilderness() -> CardDefinition {
    let soldier = Arc::new(TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), w()]))],
        ..spell(
            "Scout the Wilderness",
            cost(&[generic(2), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(2),
                        definition: soldier,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Shipwreck Looter — {1}{U} 2/1 Human Pirate. Raid — ETB, if you attacked this
/// turn, you may draw a card; if you do, discard a card.
pub fn shipwreck_looter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![raid_etb(Effect::MayDo {
            description: "Draw a card, then discard a card?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
            ])),
        })],
        ..creature(
            "Shipwreck Looter",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            2,
            1,
        )
    }
}

/// Ambuscade Shaman — {2}{B} 2/2 Orc Shaman. It or another creature you
/// control entering gets +2/+2 until end of turn. Dash {3}{B}.
pub fn ambuscade_shaman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        alternative_cost: Some(dash(cost(&[generic(3), b()]))),
        ..creature(
            "Ambuscade Shaman",
            cost(&[generic(2), b()]),
            vec![CreatureType::Orc, CreatureType::Shaman],
            2,
            2,
        )
    }
}

/// Snake Cult Initiation — {3}{B} Aura. Enchanted creature has poisonous 3.
pub fn snake_cult_initiation() -> CardDefinition {
    CardDefinition {
        name: "Snake Cult Initiation",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Poisonous(3)],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Drowner Initiate — {U} 1/1 Merfolk Wizard. Whenever a player casts a blue
/// spell, you may pay {1}; if you do, target player mills two cards.
pub fn drowner_initiate() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellMatches(R::HasColor(Color::Blue))),
            effect: Effect::MayPay {
                description: "Pay {1} to have target player mill two?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Mill {
                    who: target_filtered(R::Player),
                    amount: Value::Const(2),
                }),
                else_: None,
            },
        }],
        ..creature(
            "Drowner Initiate",
            cost(&[u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Dragon's Eye Savants — {1}{U} 0/6 Human Wizard. Morph—Reveal a blue card in
/// your hand. Turned face up: look at target opponent's hand.
pub fn dragons_eye_savants() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MorphCost(Box::new(WardCost::RevealMatching(
            Box::new(R::HasColor(Color::Blue)),
        )))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::LookAtHand {
                who: target_filtered(R::OpponentPlayer),
            },
        }],
        ..creature(
            "Dragon's Eye Savants",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            0,
            6,
        )
    }
}

/// Goblin Morningstar — {1}{R} Equipment: +1/+0 and trample; equip {1}{R}.
/// ETB: roll a d20 — 1–9 a 1/1 red Goblin; 10–20 a 1/1 red Goblin, then
/// attach this Equipment to it.
pub fn goblin_morningstar() -> CardDefinition {
    let goblin = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: "Goblin".into(),
            power: 1,
            toughness: 1,
            colors: vec![Color::Red],
            card_types: vec![CardType::Creature],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Goblin],
                ..Default::default()
            },
            ..Default::default()
        }),
    };
    CardDefinition {
        name: "Goblin Morningstar",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1), r()]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::RollDie {
            sides: 20,
            count: Value::ONE,
            modifier: Value::ZERO,
            reroll_at_most: 0,
            ignore_lowest: 0,
            results: vec![
                (1, 9, goblin()),
                (
                    10,
                    20,
                    Effect::Seq(vec![
                        goblin(),
                        Effect::Attach {
                            what: Selector::This,
                            to: Selector::LastCreatedToken,
                        },
                    ]),
                ),
            ],
            on_doubles: None,
        })],
        ..Default::default()
    }
}

/// Pompous Battlemage // Improvised Act — {R} 1/1 prowess Goblin Sorcerer that
/// enters prepared. Improvised Act ({R} sorcery): you may discard a card; if
/// you do, draw a card.
pub fn pompous_battlemage() -> CardDefinition {
    let act = spell(
        "Improvised Act",
        cost(&[r()]),
        CardType::Sorcery,
        Effect::MayDo {
            description: "Discard a card to draw a card?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                if_discarded(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            ])),
        },
    );
    CardDefinition {
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![prowess()],
        prepare_spell: Some(Arc::new(act)),
        enters_with_counters: Some((CounterType::Prepared, Value::ONE)),
        ..creature(
            "Pompous Battlemage",
            cost(&[r()]),
            vec![CreatureType::Goblin, CreatureType::Sorcerer],
            1,
            1,
        )
    }
}

/// Isareth the Awakener — {1}{B}{B} legendary 3/3 Human Wizard, deathtouch.
/// Attacks: you may pay {X}; when you do, return target creature card with mana
/// value X from your graveyard with a corpse counter, exiled if it would leave.
pub fn isareth_the_awakener() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_attack(Effect::MayPayX {
            description: "Pay {X} to return a creature card with mana value X?".into(),
            body: Box::new(Effect::Reflexive {
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(
                            R::Creature
                                .and(R::InYourGraveyard)
                                .and(R::ManaValueExactlyXFromCost),
                        ),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                    Effect::AddCounter {
                        what: Selector::LastMoved,
                        kind: CounterType::Corpse,
                        amount: Value::ONE,
                    },
                    Effect::ExileIfLeavesBattlefield {
                        what: Selector::LastMoved,
                    },
                ])),
            }),
        })],
        ..creature(
            "Isareth the Awakener",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    }
}
