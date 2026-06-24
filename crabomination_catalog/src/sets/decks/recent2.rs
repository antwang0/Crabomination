//! A second wave of recent-set staples filling small gaps (DFT / MKM / NEO /
//! WOE / DSK / ELD …). Each card has a functionality test in
//! `crabomination/src/tests/recent2.rs`.

use crate::card::{
    Adventure, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// Tangle — {1}{G} Instant. Prevent all combat damage this turn; each attacking
/// creature doesn't untap during its controller's next untap step.
pub fn tangle() -> CardDefinition {
    CardDefinition {
        name: "Tangle",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PreventAllCombatDamageThisTurn,
            Effect::SkipNextUntap {
                what: Selector::EachPermanent(SelectionRequirement::IsAttacking),
            },
        ]),
        ..Default::default()
    }
}

/// March of Otherworldly Light — {X}{W} Instant. Exile target artifact, creature,
/// or enchantment with mana value X or less. (The "exile white cards from hand
/// to reduce the cost" additional cost is dropped.)
pub fn march_of_otherworldly_light() -> CardDefinition {
    CardDefinition {
        name: "March of Otherworldly Light",
        cost: cost(&[generic(0), w()]), // {X}{W}; X paid as generic at cast time
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Or(
                    Box::new(SelectionRequirement::Or(
                        Box::new(SelectionRequirement::Artifact),
                        Box::new(SelectionRequirement::Creature),
                    )),
                    Box::new(SelectionRequirement::Enchantment),
                )
                .and(SelectionRequirement::ManaValueAtMostXFromCost),
            },
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Disdainful Stroke — {1}{U} Instant. Counter target spell with mana value 4
/// or greater.
pub fn disdainful_stroke() -> CardDefinition {
    CardDefinition {
        name: "Disdainful Stroke",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::ManaValueAtLeast(4)),
            ),
        },
        ..Default::default()
    }
}

/// Flame Lash — {3}{R} Instant. Deals 4 damage to any target.
pub fn flame_lash() -> CardDefinition {
    CardDefinition {
        name: "Flame Lash",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(4) },
        ..Default::default()
    }
}

/// Virtue of Persistence // Locthwain Scorn — {5}{B}{B} Enchantment with an
/// Adventure. Enchantment: at the beginning of your upkeep, put target creature
/// card from a graveyard onto the battlefield under your control. Adventure
/// (Locthwain Scorn {1}{B} Sorcery): target creature gets -3/-3; you gain 2 life.
pub fn virtue_of_persistence() -> CardDefinition {
    CardDefinition {
        name: "Virtue of Persistence",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::InGraveyard),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Locthwain Scorn",
            cost: cost(&[generic(1), b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-3),
                    toughness: Value::Const(-3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
        })),
        ..Default::default()
    }
}

/// Scrabbling Skullcrab — {U} 0/3 Crab Skeleton. Eerie — whenever an enchantment
/// you control enters, target player mills two cards. (The "fully unlock a Room"
/// half is dropped — Rooms aren't modeled.)
pub fn scrabbling_skullcrab() -> CardDefinition {
    CardDefinition {
        name: "Scrabbling Skullcrab",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Crab, CreatureType::Skeleton],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                }),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Conduit of Worlds — {2}{G}{G} Artifact. You may play lands from your
/// graveyard. (The "{T}: cast a nonland permanent from your graveyard if you
/// haven't cast a spell this turn" half is dropped — the one-spell lock isn't
/// modeled.)
pub fn conduit_of_worlds() -> CardDefinition {
    CardDefinition {
        name: "Conduit of Worlds",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "You may play lands from your graveyard.",
            effect: StaticEffect::MayPlayLandsFromGraveyard,
        }],
        ..Default::default()
    }
}

/// Hush — {3}{G} Sorcery. Destroy all enchantments. Cycling {2}.
pub fn hush() -> CardDefinition {
    CardDefinition {
        name: "Hush",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::DestroyNoRegen {
            what: Selector::EachPermanent(SelectionRequirement::Enchantment),
        },
        ..Default::default()
    }
}

/// Llanowar Greenwidow — {2}{G} 4/3 Spider with reach and trample. {7}{G},
/// exile from graveyard isn't required — return it from your graveyard to the
/// battlefield tapped (sorcery speed). (The Domain cost reduction and the
/// "exile if it would leave" rider are dropped.)
pub fn llanowar_greenwidow() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Llanowar Greenwidow",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7), g()]),
            from_graveyard: true,
            sorcery_speed: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Searchlight Companion — {3} 1/1 Artifact Drone with flying. ETB create a
/// 1/1 colorless Spirit token.
pub fn searchlight_companion() -> CardDefinition {
    CardDefinition {
        name: "Searchlight Companion",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Drone], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crabomination_base::tokens::spirit_token(),
        })],
        ..Default::default()
    }
}

/// Resolute Reinforcements — {1}{W} 1/1 Human Soldier with flash. ETB create a
/// 1/1 white Soldier token.
pub fn resolute_reinforcements() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Resolute Reinforcements",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: soldier,
        })],
        ..Default::default()
    }
}

/// Jewel Thief — {2}{G} 3/3 Cat Rogue with vigilance and trample. ETB create a
/// Treasure token.
pub fn jewel_thief() -> CardDefinition {
    CardDefinition {
        name: "Jewel Thief",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crabomination_base::tokens::treasure_token(),
        })],
        ..Default::default()
    }
}

/// Sweettooth Witch — {2}{B} 3/2 Human Warlock. ETB create a Food token. (The
/// "{2}, Sacrifice a Food: target player loses 3 life" ability is dropped.)
pub fn sweettooth_witch() -> CardDefinition {
    CardDefinition {
        name: "Sweettooth Witch",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crabomination_base::tokens::food_token(),
        })],
        ..Default::default()
    }
}

/// Ambush Paratrooper — {1}{W} 1/2 Human Soldier with flash and flying. {5}:
/// creatures you control get +1/+1 until end of turn.
pub fn ambush_paratrooper() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::Duration;
    CardDefinition {
        name: "Ambush Paratrooper",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Glistening Deluge — {1}{B}{B} Sorcery. All creatures get -1/-1; green and/or
/// white creatures get an additional -2/-2 until end of turn.
pub fn glistening_deluge() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Glistening Deluge",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::EachPermanent(SelectionRequirement::Creature.and(
                    SelectionRequirement::Or(
                        Box::new(SelectionRequirement::HasColor(Color::Green)),
                        Box::new(SelectionRequirement::HasColor(Color::White)),
                    ),
                )),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Faerie Dreamthief — {B} 1/1 Faerie Warlock with flying. ETB surveil 1.
/// {2}{B}, exile this card from your graveyard: draw a card.
pub fn faerie_dreamthief() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Faerie Dreamthief",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vinereap Mentor — {B}{G} 3/2 Squirrel Druid. When it enters or dies, create
/// a Food token.
pub fn vinereap_mentor() -> CardDefinition {
    use crate::effect::shortcut::on_dies;
    let food = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: crabomination_base::tokens::food_token(),
    };
    CardDefinition {
        name: "Vinereap Mentor",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(food()), on_dies(food())],
        ..Default::default()
    }
}

/// Topiary Panther — {4}{G}{G} 6/5 Plant Cat with trample. Basic landcycling
/// {1}{G}.
pub fn topiary_panther() -> CardDefinition {
    CardDefinition {
        name: "Topiary Panther",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Cat],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![
            Keyword::Trample,
            Keyword::Typecycling(Box::new((cost(&[generic(1), g()]), SelectionRequirement::IsBasicLand))),
        ],
        ..Default::default()
    }
}

/// Valgavoth's Faithful — {B} 1/1 Human Cleric. {3}{B}, sacrifice this creature:
/// return target creature card from your graveyard to the battlefield. Sorcery
/// speed.
pub fn valgavoths_faithful() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Valgavoth's Faithful",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Charforger — {1}{B}{R} 2/3 Phyrexian Beast. ETB create a 1/1 red Phyrexian
/// Goblin. Whenever another creature you control dies, put a +1/+1 counter on
/// it. (The "or artifact" half of the death watch is dropped.)
pub fn charforger() -> CardDefinition {
    let goblin = TokenDefinition {
        name: "Phyrexian Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Goblin],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Charforger",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: goblin }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Voracious Vermin — {2}{B} 2/1 Rat. ETB create a 1/1 black Rat that can't
/// block. Whenever another creature you control dies, put a +1/+1 counter on it.
pub fn voracious_vermin() -> CardDefinition {
    let rat = TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    };
    CardDefinition {
        name: "Voracious Vermin",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: rat }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Mocking Sprite — {2}{U} 2/1 Faerie Rogue with flying. Instant and sorcery
/// spells you cast cost {1} less.
pub fn mocking_sprite() -> CardDefinition {
    CardDefinition {
        name: "Mocking Sprite",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::Or(
                    Box::new(SelectionRequirement::HasCardType(CardType::Instant)),
                    Box::new(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Ancestral Reminiscence — {3}{U} Sorcery. Draw three cards, then discard a card.
pub fn ancestral_reminiscence() -> CardDefinition {
    CardDefinition {
        name: "Ancestral Reminiscence",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
        ]),
        ..Default::default()
    }
}

/// Charge — {W} Instant. Creatures you control get +1/+1 until end of turn.
pub fn charge() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Charge",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Heroic Reinforcements — {2}{R}{W} Sorcery. Create two 1/1 white Soldiers; until
/// end of turn, creatures you control get +1/+1 and gain haste.
pub fn heroic_reinforcements() -> CardDefinition {
    use crate::effect::Duration;
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    let your_creatures = || Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "Heroic Reinforcements",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: soldier },
            Effect::PumpPT {
                what: your_creatures(),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: your_creatures(),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Pyrewood Gearhulk — {2}{R}{R}{G}{G} 7/7 Construct with vigilance and menace.
/// ETB: other creatures you control get +2/+2 and gain vigilance and menace
/// until end of turn.
pub fn pyrewood_gearhulk() -> CardDefinition {
    use crate::effect::Duration;
    let others = || Selector::EachPermanent(
        SelectionRequirement::Creature
            .and(SelectionRequirement::ControlledByYou)
            .and(SelectionRequirement::OtherThanSource),
    );
    CardDefinition {
        name: "Pyrewood Gearhulk",
        cost: cost(&[generic(2), r(), r(), g(), g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Vigilance, Keyword::Menace],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: others(),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: others(), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: others(), keyword: Keyword::Menace, duration: Duration::EndOfTurn },
        ]))],
        ..Default::default()
    }
}

/// Beastbond Outcaster — {2}{G} 3/3 Human Druid. ETB: if you control a creature
/// with power 4 or greater, draw a card. Plot {1}{G}.
pub fn beastbond_outcaster() -> CardDefinition {
    CardDefinition {
        name: "Beastbond Outcaster",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        plot_cost: Some(cost(&[generic(1), g()])),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::PowerAtLeast(4)),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Mindwhisker — {2}{U} 3/2 Rat Wizard. At the beginning of your upkeep, surveil 1.
pub fn mindwhisker() -> CardDefinition {
    CardDefinition {
        name: "Mindwhisker",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Lord Skitter, Sewer King — {2}{B} 3/3 Legendary Rat Noble. Whenever another
/// Rat you control enters, exile a card from an opponent's graveyard. At the
/// beginning of combat on your turn, create a 1/1 black Rat that can't block.
pub fn lord_skitter_sewer_king() -> CardDefinition {
    let rat = TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    };
    CardDefinition {
        name: "Lord Skitter, Sewer King",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Rat)
                            .and(SelectionRequirement::OtherThanSource),
                    }),
                effect: Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::InGraveyard
                            .and(SelectionRequirement::ControlledByOpponent),
                    },
                    to: ZoneDest::Exile,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: rat,
                },
            },
        ],
        ..Default::default()
    }
}
