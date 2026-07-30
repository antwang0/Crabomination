//! Ravnica batch 3: guild spells and creatures exercising established
//! primitives — Transmute (`shortcut::transmute`), Haunt
//! (`Effect::HauntCreature`), Graft, Dredge, and the counter/bounce/library
//! effects. Tests in `recent_b/recent_293`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_dies, target_filtered, transmute};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// A 1/1 black Bat token with flying (Belfry Spirit's haunt payoff).
fn bat_token() -> TokenDefinition {
    TokenDefinition {
        name: "Bat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bat],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

// ── Dimir: Transmute ────────────────────────────────────────────────────────

/// Muddle the Mixture — {U}{U} Instant. Counter target instant or sorcery
/// spell. Transmute {1}{U}{U}.
pub fn muddle_the_mixture() -> CardDefinition {
    CardDefinition {
        name: "Muddle the Mixture",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(
                R::IsSpellOnStack
                    .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
            ),
        },
        activated_abilities: vec![transmute(cost(&[generic(1), u(), u()]), 2)],
        ..Default::default()
    }
}

/// Dizzy Spell — {U} Instant. Target creature gets -3/-0 until end of turn.
/// Transmute {1}{U}{U}.
pub fn dizzy_spell() -> CardDefinition {
    CardDefinition {
        name: "Dizzy Spell",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        activated_abilities: vec![transmute(cost(&[generic(1), u(), u()]), 1)],
        ..Default::default()
    }
}

/// Shred Memory — {1}{B} Instant. Exile up to four target cards from a single
/// graveyard. Transmute {1}{B}{B}.
pub fn shred_memory() -> CardDefinition {
    CardDefinition {
        name: "Shred Memory",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ExileUpToNFromGraveyards {
            count: Value::Const(4),
            of: None,
            single: true,
        },
        activated_abilities: vec![transmute(cost(&[generic(1), b(), b()]), 2)],
        ..Default::default()
    }
}

/// Clutch of the Undercity — {1}{U}{U}{B} Instant. Return target permanent to
/// its owner's hand; its controller loses 3 life. Transmute {1}{U}{B}.
pub fn clutch_of_the_undercity() -> CardDefinition {
    CardDefinition {
        name: "Clutch of the Undercity",
        cost: cost(&[generic(1), u(), u(), b()]),
        card_types: vec![CardType::Instant],
        // Lose life first (controller still on the battlefield), then bounce.
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(3),
            },
            Effect::Move {
                what: target_filtered(R::Permanent),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        ]),
        activated_abilities: vec![transmute(cost(&[generic(1), u(), b()]), 4)],
        ..Default::default()
    }
}

/// Brainspoil — {3}{B}{B} Sorcery. Destroy target creature that isn't
/// enchanted; it can't be regenerated. Transmute {1}{B}{B}.
pub fn brainspoil() -> CardDefinition {
    CardDefinition {
        name: "Brainspoil",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DestroyNoRegen {
            what: target_filtered(R::Creature.and(R::IsEnchanted.negate())),
        },
        activated_abilities: vec![transmute(cost(&[generic(1), b(), b()]), 5)],
        ..Default::default()
    }
}

/// Dimir Infiltrator — {U}{B} 1/3 Spirit. Can't be blocked. Transmute {1}{U}{B}.
pub fn dimir_infiltrator() -> CardDefinition {
    CardDefinition {
        name: "Dimir Infiltrator",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Unblockable],
        activated_abilities: vec![transmute(cost(&[generic(1), u(), b()]), 2)],
        ..Default::default()
    }
}

/// Netherborn Phalanx — {5}{B} 2/4 Horror. When it enters, each opponent loses
/// 1 life for each creature they control. Transmute {1}{B}{B}.
pub fn netherborn_phalanx() -> CardDefinition {
    CardDefinition {
        name: "Netherborn Phalanx",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::CreatureCountControlledBy(PlayerRef::EachOpponent),
            },
        }],
        activated_abilities: vec![transmute(cost(&[generic(1), b(), b()]), 6)],
        ..Default::default()
    }
}

// ── Orzhov: Haunt ───────────────────────────────────────────────────────────

/// Blind Hunter — {2}{W}{B} 2/2 Bat. Flying, haunt. When it enters or the
/// creature it haunts dies, target opponent loses 2 life and you gain 2 life.
pub fn blind_hunter() -> CardDefinition {
    let drain = Effect::Seq(vec![
        Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        },
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        },
    ]);
    CardDefinition {
        name: "Blind Hunter",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bat],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: drain.clone(),
            },
            on_dies(Effect::HauntCreature {
                body: Box::new(drain),
            }),
        ],
        ..Default::default()
    }
}

/// Belfry Spirit — {3}{W}{W} 1/1 Spirit. Flying, haunt. When it enters or the
/// creature it haunts dies, create two 1/1 black Bat tokens with flying.
pub fn belfry_spirit() -> CardDefinition {
    let make_bats = Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(2),
        definition: bat_token(),
    };
    CardDefinition {
        name: "Belfry Spirit",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: make_bats.clone(),
            },
            on_dies(Effect::HauntCreature {
                body: Box::new(make_bats),
            }),
        ],
        ..Default::default()
    }
}

// ── Boros / Gruul ───────────────────────────────────────────────────────────

/// Sunhome Enforcer — {2}{R}{W} 2/4 Giant Soldier. Whenever it deals combat
/// damage to a player, you gain that much life. {1}{R}: +1/+0 until end of turn.
pub fn sunhome_enforcer() -> CardDefinition {
    CardDefinition {
        name: "Sunhome Enforcer",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::TriggerEventAmount,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rumbling Slum — {1}{R}{G}{G} 5/5 Elemental. At the beginning of your upkeep,
/// it deals 1 damage to each player.
pub fn rumbling_slum() -> CardDefinition {
    CardDefinition {
        name: "Rumbling Slum",
        cost: cost(&[generic(1), r(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

// ── Simic: Graft / big fatties ──────────────────────────────────────────────

/// Simic Sky Swallower — {5}{G}{U} 6/6 Leviathan. Flying, trample, shroud.
pub fn simic_sky_swallower() -> CardDefinition {
    CardDefinition {
        name: "Simic Sky Swallower",
        cost: cost(&[generic(5), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Leviathan],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Shroud],
        ..Default::default()
    }
}

/// Novijen Sages — {4}{U}{U} 0/0 Human Advisor Mutant. Graft 4. {1}, Remove two
/// +1/+1 counters from among creatures you control: Draw a card.
pub fn novijen_sages() -> CardDefinition {
    CardDefinition {
        name: "Novijen Sages",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Advisor,
                CreatureType::Mutant,
            ],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_among_filter: Some((
                Some(CounterType::PlusOnePlusOne),
                2,
                R::Creature.and(R::ControlledByYou),
            )),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Orzhov auras / legends & bounce spells ──────────────────────────────────

/// Pillory of the Sleepless — {1}{W}{B} Aura. Enchant creature. Enchanted
/// creature can't attack or block and its controller loses 1 life at the
/// beginning of their upkeep.
pub fn pillory_of_the_sleepless() -> CardDefinition {
    CardDefinition {
        name: "Pillory of the Sleepless",
        cost: cost(&[generic(1), w(), b()]),
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
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Ghost Council of Orzhova — {W}{W}{B}{B} legendary 4/4 Spirit. When it enters,
/// each opponent loses 1 life and you gain 1 life. {1}, Sacrifice a creature:
/// Exile it, return it at the beginning of the next end step.
pub fn ghost_council_of_orzhova() -> CardDefinition {
    CardDefinition {
        name: "Ghost Council of Orzhova",
        cost: cost(&[w(), w(), b(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::ExileReturnNextEndStep {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Seeds of Strength — {G}{W} Instant. Up to three target creatures each get
/// +1/+1 until end of turn.
pub fn seeds_of_strength() -> CardDefinition {
    let pump = |slot: u8| Effect::PumpPT {
        what: Selector::TargetFiltered {
            slot,
            filter: R::Creature,
        },
        power: Value::ONE,
        toughness: Value::ONE,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Seeds of Strength",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![pump(0), pump(1), pump(2)]),
        ..Default::default()
    }
}

/// Vedalken Dismisser — {5}{U} 2/2 Vedalken Wizard. When it enters, put target
/// creature on top of its owner's library.
pub fn vedalken_dismisser() -> CardDefinition {
    CardDefinition {
        name: "Vedalken Dismisser",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::Top,
                },
            },
        }],
        ..Default::default()
    }
}

/// Nightmare Void — {3}{B} Sorcery. Target player reveals their hand; you choose
/// a card from it and they discard it. Dredge 2.
pub fn nightmare_void() -> CardDefinition {
    CardDefinition {
        name: "Nightmare Void",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Dredge(2)],
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::EachOpponent),
            count: Value::ONE,
            filter: R::Any,
        },
        ..Default::default()
    }
}

/// Vedalken Entrancer — {3}{U} 1/4 Vedalken Wizard. {U}, {T}: Target player
/// mills two cards.
pub fn vedalken_entrancer() -> CardDefinition {
    CardDefinition {
        name: "Vedalken Entrancer",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::Mill {
                who: target_filtered(R::Player),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Beacon Hawk — {1}{W} 1/1 Bird. Flying. Whenever it deals combat damage to a
/// player, you may untap target creature. {W}: +0/+1 until end of turn.
pub fn beacon_hawk() -> CardDefinition {
    CardDefinition {
        name: "Beacon Hawk",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Untap target creature".into(),
                body: Box::new(Effect::Untap {
                    what: target_filtered(R::Creature),
                    up_to: None,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(0),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
