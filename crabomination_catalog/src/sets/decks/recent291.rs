//! Ravnica guild batch: the hybrid Guildmage cycle (two activated abilities
//! each), the Convoke Saproling makers, and a handful of guild removal /
//! reanimation spells. All reuse existing primitives — activated abilities,
//! `Effect::PumpPT`/`GrantKeyword`/`Tap`/`CounterAbility`, Convoke, Aura
//! `EquipBonus`, and `ExileLastCreatedTokensAtNextEndStep`.
//! Tests in `recent_b/recent291`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, Keyword, SelectionRequirement as R, Selector, Subtypes, TokenDefinition, Value,
    Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, TriggeredAbility,
    ZoneDest,
};
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w};

/// A vanilla 1/1 green Saproling token.
fn saproling_token() -> TokenDefinition {
    TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Saproling],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// A 2/1 red Goblin token with haste (Rakdos Guildmage's temp).
fn hasty_goblin_token() -> TokenDefinition {
    TokenDefinition {
        name: "Goblin".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

// ── Guildmage cycle ─────────────────────────────────────────────────────────

/// Selesnya Guildmage — {G/W}{G/W} 2/2 Elf Wizard. {3}{G}: make a Saproling;
/// {3}{W}: creatures you control get +1/+1 until end of turn.
pub fn selesnya_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Selesnya Guildmage",
        cost: cost(&[
            hybrid(Color::Green, Color::White),
            hybrid(Color::Green, Color::White),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), g()]),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: saproling_token(),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), w()]),
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dimir Guildmage — {U/B}{U/B} 2/2 Human Wizard. {3}{U}: target player draws
/// (sorcery speed); {3}{B}: target player discards (sorcery speed).
pub fn dimir_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Dimir Guildmage",
        cost: cost(&[
            hybrid(Color::Blue, Color::Black),
            hybrid(Color::Blue, Color::Black),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), u()]),
                sorcery_speed: true,
                effect: Effect::Draw {
                    who: Selector::Target(0),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), b()]),
                sorcery_speed: true,
                effect: Effect::Discard {
                    who: Selector::Target(0),
                    amount: Value::ONE,
                    random: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Boros Guildmage — {R/W}{R/W} 2/2 Human Wizard. {1}{R}: target creature gains
/// haste; {1}{W}: target creature gains first strike (both until end of turn).
pub fn boros_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Boros Guildmage",
        cost: cost(&[
            hybrid(Color::Red, Color::White),
            hybrid(Color::Red, Color::White),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w()]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Gruul Guildmage — {R/G}{R/G} 2/2 Human Shaman. {3}{R}, Sacrifice a land: deal
/// 2 damage to target player or planeswalker; {3}{G}: target creature +2/+2 EOT.
pub fn gruul_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Gruul Guildmage",
        cost: cost(&[
            hybrid(Color::Red, Color::Green),
            hybrid(Color::Red, Color::Green),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), r()]),
                sac_other_filter: Some((R::Land, 1)),
                effect: Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), g()]),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Orzhov Guildmage — {W/B}{W/B} 2/2 Human Wizard. {2}{W}: target player gains 1
/// life; {2}{B}: each player loses 1 life.
pub fn orzhov_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Orzhov Guildmage",
        cost: cost(&[
            hybrid(Color::White, Color::Black),
            hybrid(Color::White, Color::Black),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                effect: Effect::GainLife {
                    who: Selector::Target(0),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rakdos Guildmage — {B/R}{B/R} 2/2 Zombie Shaman. {3}{B}, Discard a card:
/// target creature gets -2/-2 EOT; {3}{R}: make a 2/1 haste Goblin, exiled at
/// the next end step.
pub fn rakdos_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Rakdos Guildmage",
        cost: cost(&[
            hybrid(Color::Black, Color::Red),
            hybrid(Color::Black, Color::Red),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), b()]),
                discard_cost: Some((R::Any, 1)),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), r()]),
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: hasty_goblin_token(),
                    },
                    Effect::ExileLastCreatedTokensAtNextEndStep,
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Azorius Guildmage — {W/U}{W/U} 2/2 Vedalken Wizard. {2}{W}: tap target
/// creature; {2}{U}: counter target activated ability.
pub fn azorius_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Azorius Guildmage",
        cost: cost(&[
            hybrid(Color::White, Color::Blue),
            hybrid(Color::White, Color::Blue),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                effect: Effect::Tap {
                    what: target_filtered(R::Creature),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                effect: Effect::CounterAbility {
                    what: Selector::Target(0),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Saproling makers & guild spells ─────────────────────────────────────────

/// Fists of Ironwood — {1}{G} Aura. Enchant creature. ETB: make two Saprolings.
/// Enchanted creature has trample.
pub fn fists_of_ironwood() -> CardDefinition {
    CardDefinition {
        name: "Fists of Ironwood",
        cost: cost(&[generic(1), g()]),
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
            keywords: vec![Keyword::Trample],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: saproling_token(),
        })],
        ..Default::default()
    }
}

/// Scatter the Seeds — {3}{G}{G} Instant with Convoke. Create three Saprolings.
pub fn scatter_the_seeds() -> CardDefinition {
    CardDefinition {
        name: "Scatter the Seeds",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Convoke],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: saproling_token(),
        },
        ..Default::default()
    }
}

/// Sundering Vitae — {2}{G} Instant with Convoke. Destroy target artifact or
/// enchantment.
pub fn sundering_vitae() -> CardDefinition {
    CardDefinition {
        name: "Sundering Vitae",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Convoke],
        effect: Effect::Destroy {
            what: target_filtered(R::Artifact.or(R::Enchantment)),
        },
        ..Default::default()
    }
}

/// Golgari Rotwurm — {3}{B}{G} 5/4 Zombie Wurm. {B}, Sacrifice a creature:
/// target player loses 1 life.
pub fn golgari_rotwurm() -> CardDefinition {
    CardDefinition {
        name: "Golgari Rotwurm",
        cost: cost(&[generic(3), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wurm],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::LoseLife {
                who: Selector::Target(0),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wrecking Ball — {2}{B}{R} Instant. Destroy target creature or land.
pub fn wrecking_ball() -> CardDefinition {
    CardDefinition {
        name: "Wrecking Ball",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.or(R::Land)),
        },
        ..Default::default()
    }
}

/// Streetbreaker Wurm — {3}{R}{G} 6/4 Wurm (vanilla).
pub fn streetbreaker_wurm() -> CardDefinition {
    CardDefinition {
        name: "Streetbreaker Wurm",
        cost: cost(&[generic(3), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 6,
        toughness: 4,
        ..Default::default()
    }
}

/// Ghor-Clan Savage — {3}{G}{G} 2/3 Centaur Berserker with Bloodthirst 3.
pub fn ghor_clan_savage() -> CardDefinition {
    CardDefinition {
        name: "Ghor-Clan Savage",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Bloodthirst(3)],
        triggered_abilities: vec![crate::effect::shortcut::bloodthirst(3)],
        ..Default::default()
    }
}

/// Recollect — {2}{G} Sorcery. Return target card from your graveyard to your
/// hand.
pub fn recollect() -> CardDefinition {
    CardDefinition {
        name: "Recollect",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(R::InYourGraveyard),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Mortipede — {3}{B} 4/1 Insect. {2}{G}: All creatures able to block this
/// creature this turn do so (a one-turn Lure on itself).
pub fn mortipede() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Mortipede",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::MustBlockSource {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Disembowel — {X}{B} Instant. Destroy target creature with mana value X.
pub fn disembowel() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Disembowel",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.and(R::ManaValueExactlyXFromCost)),
        },
        ..Default::default()
    }
}

/// Vigor Mortis — {2}{B}{B} Sorcery. Return target creature card from your
/// graveyard to the battlefield; if {G} was spent to cast this spell, it enters
/// with an extra +1/+1 counter (`Predicate::ManaSpentOfColorAtLeast`).
pub fn vigor_mortis() -> CardDefinition {
    CardDefinition {
        name: "Vigor Mortis",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast {
                    color: Color::Green,
                    at_least: 1,
                },
                then: Box::new(Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Aura Mutation — {G}{W} Instant. Destroy target enchantment. Create X 1/1
/// Saprolings, where X is that enchantment's mana value. (X is read before the
/// destroy so it reflects the enchantment's printed mana value.)
pub fn aura_mutation() -> CardDefinition {
    CardDefinition {
        name: "Aura Mutation",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ManaValueOf(Box::new(Selector::Target(0))),
                definition: saproling_token(),
            },
            Effect::Destroy {
                what: target_filtered(R::Enchantment),
            },
        ]),
        ..Default::default()
    }
}

/// Golgari Guildmage — {B/G}{B/G} 2/2 Elf Shaman. {4}{B}, Sacrifice a creature:
/// return target creature card from your graveyard to your hand; {4}{G}: put a
/// +1/+1 counter on target creature.
pub fn golgari_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Golgari Guildmage",
        cost: cost(&[
            hybrid(Color::Black, Color::Green),
            hybrid(Color::Black, Color::Green),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(4), b()]),
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4), g()]),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Simic Guildmage — {G/U}{G/U} 2/2 Elf Wizard. {1}{G}: move a +1/+1 counter
/// between two target creatures; {1}{U}: reattach a target Aura to another
/// permanent (`Effect::Attach` re-anchors). The "same controller" targeting
/// clause is approximated (both slots are open).
pub fn simic_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Simic Guildmage",
        cost: cost(&[
            hybrid(Color::Green, Color::Blue),
            hybrid(Color::Green, Color::Blue),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), g()]),
                effect: Effect::MoveCounter {
                    from: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature,
                    },
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature,
                    },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                effect: Effect::Attach {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura),
                    },
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Permanent,
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Necromantic Thirst — {2}{B}{B} Aura. Enchant creature. Whenever enchanted
/// creature deals combat damage to a player, you may return a creature card
/// from your graveyard to your hand (granted via `EquipBonus.triggered_abilities`).
pub fn necromantic_thirst() -> CardDefinition {
    CardDefinition {
        name: "Necromantic Thirst",
        cost: cost(&[generic(2), b(), b()]),
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
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Return a creature card from your graveyard to your hand"
                        .to_string(),
                    body: Box::new(Effect::Move {
                        what: Selector::one_of(Selector::CardsInZone {
                            zone: Zone::Graveyard,
                            who: PlayerRef::You,
                            filter: R::Creature,
                        }),
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}
