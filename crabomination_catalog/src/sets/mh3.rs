//! Modern Horizons 3 (MH3) — 2024. A Modern-legal supplement; this module
//! collects single-faced cards that ride existing engine primitives
//! (energy, devoid, divided damage, keyword counters, modal spells).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Selector, Subtypes, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{
    add_any_one_color, add_colorless, etb, mint_treasures, on_attack, target_filtered,
};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::mana::{b, colorless, cost, g, generic, r, u, w};

/// Accursed Marauder — {1}{B} 2/1 Zombie Warrior. ETB: each player sacrifices
/// a nontoken creature of their choice.
pub fn accursed_marauder() -> CardDefinition {
    CardDefinition {
        name: "Accursed Marauder",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature.and(SelectionRequirement::NotToken),
        })],
        ..Default::default()
    }
}

/// Faithful Watchdog — {G}{W} Dog with vigilance that enters with three
/// +1/+1 counters (printed 0/0).
pub fn faithful_watchdog() -> CardDefinition {
    CardDefinition {
        name: "Faithful Watchdog",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Vigilance],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        ..Default::default()
    }
}

/// Nightshade Dryad — {1}{G} 1/2 Dryad with deathtouch and two mana abilities:
/// "{T}: Add {C}." and "{T}: Add one mana of any color."
pub fn nightshade_dryad() -> CardDefinition {
    CardDefinition {
        name: "Nightshade Dryad",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: add_colorless(1), ..Default::default() },
            ActivatedAbility { tap_cost: true, effect: add_any_one_color(1), ..Default::default() },
        ],
        ..Default::default()
    }
}

/// Serum Visionary — {2}{U} 2/2 Vedalken Wizard. ETB: draw a card, then scry 2.
pub fn serum_visionary() -> CardDefinition {
    CardDefinition {
        name: "Serum Visionary",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]))],
        ..Default::default()
    }
}

/// Wing It — {1}{W} Instant. Target creature gets +2/+2, gains a flying
/// counter, then scry 1.
pub fn wing_it() -> CardDefinition {
    CardDefinition {
        name: "Wing It",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::AddKeywordCounter {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Flying,
                amount: Value::Const(1),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Gift of the Viper — {G} Instant. Put a +1/+1, a reach, and a deathtouch
/// counter on target creature, then untap it.
pub fn gift_of_the_viper() -> CardDefinition {
    CardDefinition {
        name: "Gift of the Viper",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::AddKeywordCounter {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Reach,
                amount: Value::Const(1),
            },
            Effect::AddKeywordCounter {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Deathtouch,
                amount: Value::Const(1),
            },
            Effect::Untap {
                what: target_filtered(SelectionRequirement::Creature),
                up_to: None,
            },
        ]),
        ..Default::default()
    }
}

/// Null Elemental Blast — {C} Instant. Choose one — counter target
/// multicolored spell; or destroy target multicolored permanent.
pub fn null_elemental_blast() -> CardDefinition {
    CardDefinition {
        name: "Null Elemental Blast",
        cost: cost(&[crate::mana::colorless(1)]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack.and(SelectionRequirement::Multicolored),
                ),
            },
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Multicolored),
            },
        ]),
        ..Default::default()
    }
}

/// Mogg Mob — {R}{R}{R} 3/3 Goblin. "Sacrifice this creature: It deals 3
/// damage divided as you choose among one, two, or three targets."
pub fn mogg_mob() -> CardDefinition {
    CardDefinition {
        name: "Mogg Mob",
        cost: cost(&[r(), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::DealDamageDivided {
                total: Value::Const(3),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
                max_targets: 3,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Retrofitted Transmogrant — {B} 1/1 Artifact Zombie. "{3}{B}: Return this
/// card from your graveyard to the battlefield tapped with two +1/+1
/// counters on it."
pub fn retrofitted_transmogrant() -> CardDefinition {
    CardDefinition {
        name: "Retrofitted Transmogrant",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            from_graveyard: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sarpadian Simulacrum — {R} 1/1 Artifact Goblin with haste. "{3}{R},
/// Sacrifice this creature: It deals 4 damage to target creature."
pub fn sarpadian_simulacrum() -> CardDefinition {
    CardDefinition {
        name: "Sarpadian Simulacrum",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(4),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Consuming Corruption — {B}{B} Instant. Deals X damage to target creature
/// or planeswalker and you gain X life, where X is the number of Swamps you
/// control.
pub fn consuming_corruption() -> CardDefinition {
    let swamps = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(
            SelectionRequirement::HasLandType(crate::card::LandType::Swamp)
                .and(SelectionRequirement::ControlledByYou),
        )),
        filter: SelectionRequirement::Any,
    };
    CardDefinition {
        name: "Consuming Corruption",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: swamps.clone(),
            },
            Effect::GainLife { who: Selector::You, amount: swamps },
        ]),
        ..Default::default()
    }
}

/// Fanged Flames — {1}{R} Sorcery (devoid). Deals 4 damage to target creature
/// or planeswalker; if it would die this turn, exile it instead.
pub fn fanged_flames() -> CardDefinition {
    CardDefinition {
        name: "Fanged Flames",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Devoid],
        // Install the "exile if it would die" replacement before the damage.
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(4),
            },
        ]),
        ..Default::default()
    }
}

/// Solstice Zealot — {2}{W} 2/3 Rhino Cleric. ETB: get {E}{E}. "{T}, Pay {E}:
/// Tap target creature."
pub fn solstice_zealot() -> CardDefinition {
    CardDefinition {
        name: "Solstice Zealot",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(2)))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            energy_cost: 1,
            effect: Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tempest Harvester — {1}{U} 2/1 Merfolk Wizard. ETB: get {E}{E}. "{T}, Pay
/// {E}: Draw a card, then discard a card."
pub fn tempest_harvester() -> CardDefinition {
    CardDefinition {
        name: "Tempest Harvester",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(2)))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            energy_cost: 1,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Warren Soultrader — {2}{B} 3/3 Zombie Goblin Wizard. "Pay 1 life,
/// Sacrifice another creature: Create a Treasure token."
pub fn warren_soultrader() -> CardDefinition {
    CardDefinition {
        name: "Warren Soultrader",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Goblin, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: mint_treasures(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Snapping Voidcraw — {1}{G}{U} 1/3 Eldrazi Turtle (devoid). "{T}: Add
/// {C}{C}." and "{3}{C}, {T}: Draw a card."
pub fn snapping_voidcraw() -> CardDefinition {
    CardDefinition {
        name: "Snapping Voidcraw",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Turtle],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Devoid],
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: add_colorless(2), ..Default::default() },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), colorless(1)]),
                tap_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Solar Transformer — {2} Artifact. Enters tapped; ETB get {E}{E}{E}.
/// "{T}: Add {C}." and "{T}, Pay {E}: Add one mana of any color."
pub fn solar_transformer() -> CardDefinition {
    CardDefinition {
        name: "Solar Transformer",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            super::etb_tap(),
            etb(Effect::AddEnergy(Value::Const(3))),
        ],
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: add_colorless(1), ..Default::default() },
            ActivatedAbility {
                tap_cost: true,
                energy_cost: 1,
                effect: add_any_one_color(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Roil Cartographer — {1}{U} 1/3 Merfolk Rogue. Landfall: get {E}. "{T}, Pay
/// six {E}: Draw three cards."
pub fn roil_cartographer() -> CardDefinition {
    CardDefinition {
        name: "Roil Cartographer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::AddEnergy(Value::Const(1)),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            energy_cost: 6,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Horrid Shadowspinner — {1}{U}{B} 2/3 Horror with lifelink. "Whenever this
/// attacks, you may draw cards equal to its power. If you do, discard that
/// many cards."
pub fn horrid_shadowspinner() -> CardDefinition {
    let power = Value::PowerOf(Box::new(Selector::This));
    CardDefinition {
        name: "Horrid Shadowspinner",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "draw cards equal to its power, then discard that many".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: power.clone() },
                Effect::Discard { who: Selector::You, amount: power, random: false },
            ])),
        })],
        ..Default::default()
    }
}

/// Unfathomable Truths — {4}{U} Instant (devoid). Draw three cards and create
/// a 0/1 colorless Eldrazi Spawn token.
pub fn unfathomable_truths() -> CardDefinition {
    CardDefinition {
        name: "Unfathomable Truths",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::eldrazi_spawn_token(),
            },
        ]),
        ..Default::default()
    }
}

/// 3/3 colorless Phyrexian Golem artifact creature token.
fn phyrexian_golem_token() -> TokenDefinition {
    TokenDefinition {
        name: "Phyrexian Golem".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Golem],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Phyrexian Ironworks — {2}{R} Artifact. "Whenever you attack, you get {E}."
/// "{T}, Pay {E}{E}{E}: Create a 3/3 Phyrexian Golem. Activate only as a
/// sorcery." The energy trigger rides `Attacks/YourControl` (per attacker,
/// the standard "whenever you attack" approximation).
pub fn phyrexian_ironworks() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Ironworks",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::AddEnergy(Value::Const(1)),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            energy_cost: 3,
            sorcery_speed: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: phyrexian_golem_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Breathe Your Last — {1}{B}{B} Instant. Destroy target creature or
/// planeswalker; you gain 1 life for each of its colors.
pub fn breathe_your_last() -> CardDefinition {
    let target = || target_filtered(
        SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
    );
    CardDefinition {
        name: "Breathe Your Last",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Instant],
        // Count colors before destroying so the life gain reads the live object.
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ColorCountOf(Box::new(target())),
            },
            Effect::Destroy { what: target() },
        ]),
        ..Default::default()
    }
}

/// Fowl Strike — {1}{G} Instant. Destroy target creature with flying.
/// Reinforce 2—{2}{G}.
pub fn fowl_strike() -> CardDefinition {
    CardDefinition {
        name: "Fowl Strike",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Reinforce(2, cost(&[generic(2), g()]))],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::HasKeyword(Keyword::Flying)),
            ),
        },
        ..Default::default()
    }
}

/// Aerie Auxiliary — {3}{W} 3/3 Bird Soldier with flying. ETB: support 2.
pub fn aerie_auxiliary() -> CardDefinition {
    use crate::effect::shortcut::support;
    CardDefinition {
        name: "Aerie Auxiliary",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(support(2))],
        ..Default::default()
    }
}

/// Scurrilous Sentry — {3}{B} 2/3 Human Knight Rogue with menace. Connives
/// whenever it enters or attacks.
pub fn scurrilous_sentry() -> CardDefinition {
    use crate::effect::shortcut::connive;
    CardDefinition {
        name: "Scurrilous Sentry",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(connive(1)), on_attack(connive(1))],
        ..Default::default()
    }
}

/// Titans' Vanguard — {3}{R}{G} 5/5 Eldrazi (devoid) with trample. When you
/// cast it and whenever it attacks, put a +1/+1 counter on each colorless
/// creature you control.
pub fn titans_vanguard() -> CardDefinition {
    let pump = || Effect::AddCounter {
        what: Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::Colorless)
                .and(SelectionRequirement::ControlledByYou),
        ),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    };
    CardDefinition {
        name: "Titans' Vanguard",
        cost: cost(&[generic(3), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Devoid, Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
                effect: pump(),
            },
            on_attack(pump()),
        ],
        ..Default::default()
    }
}

/// Wither and Bloom — {1}{B} Instant. Target creature gets -3/-3. "{1}{B},
/// Exile this card from your graveyard: Put a +1/+1 counter on target
/// creature you control. Activate only as a sorcery."
pub fn wither_and_bloom() -> CardDefinition {
    CardDefinition {
        name: "Wither and Bloom",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Thriving Skyclaw — {2}{R}{R} 3/2 Cat Dragon with flying. ETB: get
/// {E}{E}{E}. Whenever it attacks, you may pay {E}{E}{E} to grow it.
pub fn thriving_skyclaw() -> CardDefinition {
    CardDefinition {
        name: "Thriving Skyclaw",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Dragon],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::Const(3))),
            on_attack(Effect::MayDo {
                description: "pay {E}{E}{E} to put a +1/+1 counter on it".into(),
                body: Box::new(Effect::PayEnergy {
                    amount: 3,
                    then: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    }),
                }),
            }),
        ],
        ..Default::default()
    }
}
