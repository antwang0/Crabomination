//! Magic 2011 (M11) gap closure. Tests in `classic_sets/m11`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest,
    shortcut::{etb, on_attack, on_dies, target_filtered},
};
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn instant(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn aura(name: &'static str, c: crate::mana::ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// A `{cost}: this gets +P/+T until end of turn` pump ability.
fn self_pump(c: crate::mana::ManaCost, power: i32, toughness: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: c,
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Vanilla / keyword-only bodies ───────────────────────────────────────────

/// Armored Cancrix — {4}{U} 2/5 Crab.
pub fn armored_cancrix() -> CardDefinition {
    creature("Armored Cancrix", cost(&[generic(4), u()]), vec![CreatureType::Crab], 2, 5)
}

/// Maritime Guard — {1}{U} 1/3 Merfolk Soldier.
pub fn maritime_guard() -> CardDefinition {
    creature(
        "Maritime Guard",
        cost(&[generic(1), u()]),
        vec![CreatureType::Merfolk, CreatureType::Soldier],
        1,
        3,
    )
}

/// Nether Horror — {3}{B} 4/2 Horror.
pub fn nether_horror() -> CardDefinition {
    creature("Nether Horror", cost(&[generic(3), b()]), vec![CreatureType::Horror], 4, 2)
}

/// Stone Golem — {5} 4/4 Artifact Creature — Golem.
pub fn stone_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature("Stone Golem", cost(&[generic(5)]), vec![CreatureType::Golem], 4, 4)
    }
}

/// Cloud Crusader — {2}{W}{W} 2/3 Human Knight with flying and first strike.
pub fn cloud_crusader() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        ..creature(
            "Cloud Crusader",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            3,
        )
    }
}

/// Sacred Wolf — {2}{G} 3/1 Wolf with hexproof.
pub fn sacred_wolf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        ..creature("Sacred Wolf", cost(&[generic(2), g()]), vec![CreatureType::Wolf], 3, 1)
    }
}

/// Wall of Vines — {G} 0/3 Plant Wall with defender and reach.
pub fn wall_of_vines() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Reach],
        ..creature(
            "Wall of Vines",
            cost(&[g()]),
            vec![CreatureType::Plant, CreatureType::Wall],
            0,
            3,
        )
    }
}

/// Rotting Legion — {4}{B} 4/5 Zombie that enters tapped.
pub fn rotting_legion() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        ..creature("Rotting Legion", cost(&[generic(4), b()]), vec![CreatureType::Zombie], 4, 5)
    }
}

// ── Creatures with abilities ────────────────────────────────────────────────

/// Arc Runner — {2}{R} 5/1 Elemental Ox with haste; it burns out at end of turn.
pub fn arc_runner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::AnyPlayer,
            ),
            effect: Effect::SacrificeSource,
        }],
        ..creature(
            "Arc Runner",
            cost(&[generic(2), r()]),
            vec![CreatureType::Elemental, CreatureType::Ox],
            5,
            1,
        )
    }
}

/// Bloodcrazed Goblin — {R} 2/2 Goblin Berserker that needs first blood.
pub fn bloodcrazed_goblin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantAttackUnlessOpponentDamaged],
        ..creature(
            "Bloodcrazed Goblin",
            cost(&[r()]),
            vec![CreatureType::Goblin, CreatureType::Berserker],
            2,
            2,
        )
    }
}

/// Harbor Serpent — {4}{U}{U} 5/5 Serpent with islandwalk that needs five
/// Islands on the battlefield to attack.
pub fn harbor_serpent() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Landwalk(LandType::Island),
            Keyword::CantAttackUnlessLandCount(LandType::Island, 5),
        ],
        ..creature("Harbor Serpent", cost(&[generic(4), u(), u()]), vec![CreatureType::Serpent], 5, 5)
    }
}

/// Earth Servant — {5}{R} 4/4 Elemental that grows a toughness per Mountain.
pub fn earth_servant() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +0/+1 for each Mountain you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::HasLandType(LandType::Mountain),
                per_power: 0,
                per_toughness: 1,
            },
        }],
        ..creature("Earth Servant", cost(&[generic(5), r()]), vec![CreatureType::Elemental], 4, 4)
    }
}

/// Water Servant — {2}{U}{U} 3/4 Elemental that shifts its stats either way.
pub fn water_servant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            self_pump(cost(&[u()]), 1, -1),
            self_pump(cost(&[u()]), -1, 1),
        ],
        ..creature("Water Servant", cost(&[generic(2), u(), u()]), vec![CreatureType::Elemental], 3, 4)
    }
}

/// Nightwing Shade — {4}{B} 2/2 Shade with flying and the Shade pump.
pub fn nightwing_shade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![self_pump(cost(&[generic(1), b()]), 1, 1)],
        ..creature("Nightwing Shade", cost(&[generic(4), b()]), vec![CreatureType::Shade], 2, 2)
    }
}

/// Gargoyle Sentinel — {3} 3/3 Artifact Creature — Gargoyle; {3} turns the
/// wall into a flier.
pub fn gargoyle_sentinel() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Seq(vec![
                Effect::LoseKeyword { duration: Duration::EndOfTurn, what: Selector::This, keyword: Keyword::Defender },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Gargoyle Sentinel", cost(&[generic(3)]), vec![CreatureType::Gargoyle], 3, 3)
    }
}

/// Scroll Thief — {2}{U} 1/3 Merfolk Rogue that draws off connecting.
pub fn scroll_thief() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..creature(
            "Scroll Thief",
            cost(&[generic(2), u()]),
            vec![CreatureType::Merfolk, CreatureType::Rogue],
            1,
            3,
        )
    }
}

/// Merfolk Spy — {U} 1/1 Merfolk Rogue with islandwalk; connecting strips a
/// random card out of hiding.
pub fn merfolk_spy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            // "Reveals a card at random from their hand" is modeled as a
            // full hand look (knowledge-only either way).
            effect: Effect::LookAtHand { who: Selector::Player(PlayerRef::Target(0)) },
        }],
        ..creature(
            "Merfolk Spy",
            cost(&[u()]),
            vec![CreatureType::Merfolk, CreatureType::Rogue],
            1,
            1,
        )
    }
}

/// Phantom Beast — {3}{U} 4/5 Illusion Beast that pops when targeted.
pub fn phantom_beast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::SacrificeSource,
        }],
        ..creature(
            "Phantom Beast",
            cost(&[generic(3), u()]),
            vec![CreatureType::Illusion, CreatureType::Beast],
            4,
            5,
        )
    }
}

/// Phylactery Lich — {B}{B}{B} 5/5 indestructible Zombie anchored to an
/// artifact; lose every phylactery counter and it goes with them.
pub fn phylactery_lich() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Indestructible],
        as_enters_effect: Some(Effect::AddCounter {
            what: Selector::Take {
                inner: Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))),
                count: Box::new(Value::Const(1)),
            },
            kind: CounterType::Phylactery,
            amount: Value::Const(1),
        }),
        sacrifice_when: Some(Predicate::Not(Box::new(Predicate::SelectorExists(
            Selector::EachPermanent(
                R::WithCounter(CounterType::Phylactery).and(R::ControlledByYou),
            ),
        )))),
        ..creature("Phylactery Lich", cost(&[b(), b(), b()]), vec![CreatureType::Zombie], 5, 5)
    }
}

/// Roc Egg — {2}{W} 0/3 Bird Egg with defender; it hatches into a 3/3 flier.
pub fn roc_egg() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Bird".into(),
                power: 3,
                toughness: 3,
                keywords: vec![Keyword::Flying],
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Bird],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..creature(
            "Roc Egg",
            cost(&[generic(2), w()]),
            vec![CreatureType::Bird, CreatureType::Egg],
            0,
            3,
        )
    }
}

/// Mitotic Slime — {4}{G} 4/4 Ooze that splits twice on the way out.
pub fn mitotic_slime() -> CardDefinition {
    fn ooze(power: i32, then: Option<TriggeredAbility>) -> TokenDefinition {
        TokenDefinition {
            name: "Ooze".into(),
            power,
            toughness: power,
            card_types: vec![CardType::Creature],
            colors: vec![Color::Green],
            subtypes: Subtypes { creature_types: vec![CreatureType::Ooze], ..Default::default() },
            triggered_abilities: then.into_iter().collect(),
            ..Default::default()
        }
    }
    let small = on_dies(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(2),
        definition: ooze(1, None),
    });
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: ooze(2, Some(small)),
        })],
        ..creature("Mitotic Slime", cost(&[generic(4), g()]), vec![CreatureType::Ooze], 4, 4)
    }
}

/// Hoarding Dragon — {3}{R}{R} 4/4 Dragon with flying; it swallows an artifact
/// and coughs it up when it dies.
pub fn hoarding_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Search your library for an artifact card and exile it".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: R::Artifact,
                        to: ZoneDest::Exile,
                    },
                    // Stamp the find as exiled-with-this so the death trigger
                    // can name it (CR 607.2 linked abilities).
                    Effect::ExileWithSource { what: Selector::LastMoved },
                ])),
            }),
            on_dies(Effect::MayDo {
                description: "Put the exiled card into its owner's hand".into(),
                body: Box::new(Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            }),
        ],
        ..creature("Hoarding Dragon", cost(&[generic(3), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Ancient Hellkite — {4}{R}{R}{R} 6/6 Dragon with flying that machine-guns the
/// blockers while attacking.
pub fn ancient_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            condition: Some(Predicate::EntityMatches {
                what: Selector::This,
                filter: R::IsAttacking,
            }),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..creature(
            "Ancient Hellkite",
            cost(&[generic(4), r(), r(), r()]),
            vec![CreatureType::Dragon],
            6,
            6,
        )
    }
}

/// Cyclops Gladiator — {1}{R}{R}{R} 4/4 Cyclops Warrior that duels a blocker
/// on the way in.
pub fn cyclops_gladiator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Fight target creature defending player controls".into(),
            body: Box::new(Effect::Fight {
                attacker: Selector::This,
                defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            }),
        })],
        ..creature(
            "Cyclops Gladiator",
            cost(&[generic(1), r(), r(), r()]),
            vec![CreatureType::Cyclops, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Demon of Death's Gate — {6}{B}{B}{B} 9/9 Demon with flying and trample;
/// 6 life and three black creatures get it out early.
pub fn demon_of_deaths_gate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        alternative_cost: Some(crate::card::AlternativeCost {
            life_cost: 6,
            sacrifice_permanents: Some((
                R::Creature.and(R::HasColor(Color::Black)).and(R::ControlledByYou),
                3,
            )),
            ..Default::default()
        }),
        ..creature(
            "Demon of Death's Gate",
            cost(&[generic(6), b(), b(), b()]),
            vec![CreatureType::Demon],
            9,
            9,
        )
    }
}

/// Fire Servant — {3}{R}{R} 4/3 Elemental that doubles your red burn.
pub fn fire_servant() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your red instant and sorcery spells deal double damage.",
            effect: StaticEffect::YourColorSpellDamageDoubled { color: Color::Red },
        }],
        ..creature("Fire Servant", cost(&[generic(3), r(), r()]), vec![CreatureType::Elemental], 4, 3)
    }
}

/// Gaea's Revenge — {5}{G}{G} 8/5 Elemental with haste that green alone can
/// answer.
pub fn gaeas_revenge() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Haste,
            Keyword::CantBeCountered,
            Keyword::HexproofExceptColors(vec![Color::Green]),
        ],
        ..creature("Gaea's Revenge", cost(&[generic(5), g(), g()]), vec![CreatureType::Elemental], 8, 5)
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Ajani's Mantra — {1}{W}; a life a turn.
pub fn ajanis_mantra() -> CardDefinition {
    CardDefinition {
        name: "Ajani's Mantra",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Gain 1 life".into(),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(1) }),
            },
        }],
        ..Default::default()
    }
}

/// Dark Tutelage — {2}{B}; each upkeep the top card comes off at the cost of
/// its mana value in life.
pub fn dark_tutelage() -> CardDefinition {
    CardDefinition {
        name: "Dark Tutelage",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::RevealTopToHandLoseMv { who: PlayerRef::You, you_gain: false },
        }],
        ..Default::default()
    }
}

/// Jace's Erasure — {1}{U}; every draw chips a card off a library.
pub fn jaces_erasure() -> CardDefinition {
    CardDefinition {
        name: "Jace's Erasure",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Target player mills a card".into(),
                body: Box::new(Effect::Mill {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(1),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Leyline of Anticipation — {2}{U}{U}; everything you cast has flash.
pub fn leyline_of_anticipation() -> CardDefinition {
    CardDefinition {
        name: "Leyline of Anticipation",
        opening_hand: Some(crate::effect::OpeningHandEffect::StartInPlay {
            tapped: false,
            extra: Effect::Noop,
        }),
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "You may cast spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash { filter: R::Any },
        }],
        ..Default::default()
    }
}

/// Leyline of Punishment — {2}{R}{R}; no life gain, no prevention.
pub fn leyline_of_punishment() -> CardDefinition {
    CardDefinition {
        name: "Leyline of Punishment",
        opening_hand: Some(crate::effect::OpeningHandEffect::StartInPlay {
            tapped: false,
            extra: Effect::Noop,
        }),
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Players can't gain life.",
                effect: StaticEffect::PlayerCannotGainLife {
                    target: crate::effect::PlayerStaticTarget::EachPlayer,
                },
            },
            StaticAbility {
                description: "Damage can't be prevented.",
                effect: StaticEffect::DamageCantBePrevented,
            },
        ],
        ..Default::default()
    }
}

/// Leyline of Vitality — {2}{G}{G}; a tougher team and a life per arrival.
pub fn leyline_of_vitality() -> CardDefinition {
    CardDefinition {
        name: "Leyline of Vitality",
        opening_hand: Some(crate::effect::OpeningHandEffect::StartInPlay {
            tapped: false,
            extra: Effect::Noop,
        }),
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +0/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature,
                power: 0,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::MayDo {
                description: "Gain 1 life".into(),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(1) }),
            },
        }],
        ..Default::default()
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Wild Evocation — {5}{R}; each upkeep its controller's opponent — and they —
/// flip a random card off the top of their hand into play.
pub fn wild_evocation() -> CardDefinition {
    CardDefinition {
        name: "Wild Evocation",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::RandomHandCardDeployOrCastFree {
                who: Selector::Player(PlayerRef::ActivePlayer),
            },
        }],
        ..Default::default()
    }
}

/// Dryad's Favor — {G} Aura granting forestwalk.
pub fn dryads_favor() -> CardDefinition {
    aura(
        "Dryad's Favor",
        cost(&[g()]),
        EquipBonus { keywords: vec![Keyword::Landwalk(LandType::Forest)], ..Default::default() },
    )
}

/// Volcanic Strength — {1}{R} Aura: +2/+2 and mountainwalk.
pub fn volcanic_strength() -> CardDefinition {
    aura(
        "Volcanic Strength",
        cost(&[generic(1), r()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Landwalk(LandType::Mountain)],
            ..Default::default()
        },
    )
}

/// Quag Sickness — {2}{B} Aura: −1/−1 per Swamp you control.
pub fn quag_sickness() -> CardDefinition {
    aura(
        "Quag Sickness",
        cost(&[generic(2), b()]),
        EquipBonus {
            scale: Some(EquipScale {
                filter: R::HasLandType(LandType::Swamp),
                per_power: -1,
                per_toughness: -1,
                ..Default::default()
            }),
            ..Default::default()
        },
    )
}

/// Primal Cocoon — {G} Aura: a counter each upkeep, gone the moment the host
/// joins combat.
pub fn primal_cocoon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::AddCounter {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::EnchantedBySource),
                effect: Effect::SacrificeSource,
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::EnchantedBySource),
                effect: Effect::SacrificeSource,
            },
        ],
        ..aura("Primal Cocoon", cost(&[g()]), EquipBonus::default())
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Crystal Ball — {3}; {1}, {T}: Scry 2.
pub fn crystal_ball() -> CardDefinition {
    CardDefinition {
        name: "Crystal Ball",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Brittle Effigy — {1}; {4}, {T}, exile it: exile target creature.
pub fn brittle_effigy() -> CardDefinition {
    CardDefinition {
        name: "Brittle Effigy",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4)]),
            exile_self_cost: true,
            effect: Effect::Exile { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Warlord's Axe — {3} Equipment: +3/+1, equip {4}.
pub fn warlords_axe() -> CardDefinition {
    CardDefinition {
        name: "Warlord's Axe",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus { power: 3, toughness: 1, ..Default::default() }),
        ..Default::default()
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Blood Tithe — {3}{B}: drain each opponent for 3.
pub fn blood_tithe() -> CardDefinition {
    sorcery(
        "Blood Tithe",
        cost(&[generic(3), b()]),
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(3),
        },
    )
}

/// Call to Mind — {2}{U}: buy back an instant or sorcery.
pub fn call_to_mind() -> CardDefinition {
    sorcery(
        "Call to Mind",
        cost(&[generic(2), u()]),
        Effect::Move {
            what: target_filtered(
                R::InYourGraveyard
                    .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Diminish — {U}: target creature has base power and toughness 1/1.
pub fn diminish() -> CardDefinition {
    instant(
        "Diminish",
        cost(&[u()]),
        Effect::SetBasePT {
            what: target_filtered(R::Creature),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Stabbing Pain — {B}: −1/−1 and a tap.
pub fn stabbing_pain() -> CardDefinition {
    instant(
        "Stabbing Pain",
        cost(&[b()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::Tap { what: Selector::Target(0) },
        ]),
    )
}

/// Thunder Strike — {1}{R}: +2/+0 and first strike.
pub fn thunder_strike() -> CardDefinition {
    instant(
        "Thunder Strike",
        cost(&[generic(1), r()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Incite — {R}: target creature turns red and has to swing.
pub fn incite() -> CardDefinition {
    instant(
        "Incite",
        cost(&[r()]),
        Effect::Seq(vec![
            Effect::BecomeColor {
                what: target_filtered(R::Creature),
                colors: vec![Color::Red],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::MustAttack,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Combust — {1}{R}: uncounterable, unpreventable 5 to a white or blue creature.
pub fn combust() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        ..instant(
            "Combust",
            cost(&[generic(1), r()]),
            Effect::Seq(vec![
                Effect::DamageCantBePreventedThisTurn,
                Effect::DealDamage {
                    to: target_filtered(
                        R::Creature.and(R::HasColor(Color::White).or(R::HasColor(Color::Blue))),
                    ),
                    amount: Value::Const(5),
                },
            ]),
        )
    }
}

/// Autumn's Veil — {G}: your spells dodge blue and black answers this turn.
pub fn autumns_veil() -> CardDefinition {
    instant(
        "Autumn's Veil",
        cost(&[g()]),
        Effect::Seq(vec![
            Effect::GrantSpellsUncounterableThisTurn { who: Selector::You },
            Effect::GrantHexproofFromColorThisTurn {
                who: Selector::You,
                colors: vec![Color::Blue, Color::Black],
            },
        ]),
    )
}

/// Hunters' Feast — {3}{G}: any number of target players gain 6.
pub fn hunters_feast() -> CardDefinition {
    sorcery(
        "Hunters' Feast",
        cost(&[generic(3), g()]),
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Player,
            effect: Box::new(Effect::GainLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(6),
            }),
        },
    )
}

/// Destructive Force — {5}{R}{R}: five lands each, 5 to every creature.
pub fn destructive_force() -> CardDefinition {
    sorcery(
        "Destructive Force",
        cost(&[generic(5), r(), r()]),
        Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::Const(5),
                filter: R::Land,
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature),
                amount: Value::Const(5),
            },
        ]),
    )
}

/// Mass Polymorph — {5}{U}: trade your board in for whatever's on top.
pub fn mass_polymorph() -> CardDefinition {
    sorcery("Mass Polymorph", cost(&[generic(5), u()]), Effect::MassPolymorph)
}

/// Time Reversal — {3}{U}{U}: everybody reshuffles and draws seven; it exiles
/// itself.
pub fn time_reversal() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..sorcery(
            "Time Reversal",
            cost(&[generic(3), u(), u()]),
            Effect::Seq(vec![
                Effect::ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef::EachPlayer },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(7),
                },
            ]),
        )
    }
}

/// Vengeful Archon — {4}{W}{W}{W} 7/7 Archon with flying; {X} turns damage
/// aimed at you around onto a player or planeswalker. (The redirected damage
/// keeps its original source rather than becoming the Archon's.)
pub fn vengeful_archon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            effect: Effect::RedirectNextDamage {
                target: Selector::You,
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::XFromCost,
            },
            ..Default::default()
        }],
        ..creature(
            "Vengeful Archon",
            cost(&[generic(4), w(), w(), w()]),
            vec![CreatureType::Archon],
            7,
            7,
        )
    }
}

/// Mystifying Maze — a land that blinks an attacker back tapped.
pub fn mystifying_maze() -> CardDefinition {
    CardDefinition {
        name: "Mystifying Maze",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                effect: Effect::ExileReturnToOwnerNextEndStep {
                    what: target_filtered(
                        R::Creature.and(R::IsAttacking).and(R::ControlledByOpponent),
                    ),
                    tapped: true,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Stormtide Leviathan — {5}{U}{U}{U} 8/8 Leviathan with islandwalk that
/// floods the world: every land is an Island and the grounded can't attack.
pub fn stormtide_leviathan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        static_abilities: vec![
            StaticAbility {
                description: "All lands are Islands in addition to their other types.",
                effect: StaticEffect::LandTypeChanger {
                    applies_to: Selector::EachPermanent(R::Land),
                    land_type: LandType::Island,
                    replace: false,
                },
            },
            StaticAbility {
                description: "Creatures without flying or islandwalk can't attack.",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Creature
                        .and(R::Not(Box::new(R::HasKeyword(Keyword::Flying))))
                        .and(R::Not(Box::new(R::HasKeyword(Keyword::Landwalk(
                            LandType::Island,
                        ))))),
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::CantAttack],
                    opponents: false,
                    all_players: true,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
        ],
        ..creature(
            "Stormtide Leviathan",
            cost(&[generic(5), u(), u(), u()]),
            vec![CreatureType::Leviathan],
            8,
            8,
        )
    }
}

/// Angelic Arbiter — {5}{W}{W} 5/6 flier. An opponent gets one or the other:
/// casting a spell shuts off their attacks, attacking shuts off their spells.
pub fn angelic_arbiter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Each opponent who cast a spell this turn can't attack with creatures.",
                effect: StaticEffect::OpponentsWhoCastCantAttack,
            },
            StaticAbility {
                description: "Each opponent who attacked with a creature this turn can't cast spells.",
                effect: StaticEffect::OpponentsWhoAttackedCantCast,
            },
        ],
        ..creature("Angelic Arbiter", cost(&[generic(5), w(), w()]), vec![CreatureType::Angel], 5, 6)
    }
}

/// Conundrum Sphinx — {2}{U}{U} 4/4 flier. On attack every player names a card,
/// then reveals their top: a hit goes to hand, a miss to the bottom.
pub fn conundrum_sphinx() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::EachPlayerNamesCard { who: PlayerRef::EachPlayer },
            Effect::EachPlayerRevealTopKeepIfNamed { who: PlayerRef::EachPlayer },
        ]))],
        ..creature("Conundrum Sphinx", cost(&[generic(2), u(), u()]), vec![CreatureType::Sphinx], 4, 4)
    }
}

/// Necrotic Plague — {2}{B}{B} Aura. Enchanted creature sacrifices itself at its
/// controller's upkeep, and the Plague hops to a creature that player doesn't
/// control when it dies.
pub fn necrotic_plague() -> CardDefinition {
    CardDefinition {
        name: "Necrotic Plague",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has \"At the beginning of your upkeep, sacrifice this creature.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::IsHostOfSource,
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                        EventScope::YourControl,
                    ),
                    effect: Effect::SacrificeSource,
                }),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::ReturnSelfAttachedToChoiceOf {
                chooser: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}
