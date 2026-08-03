//! Zendikar (ZEN) gap closure, wave 3 — the last of the set.
//! Tests in `classic_sets/zen3`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, AlternativeCost, ArtifactSubtype, CardDefinition,
    CardType, CounterType, CreatureType, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    LandType, LoyaltyAbility, Predicate, SelectionRequirement as R, SpellSubtype, StaticAbility,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest,
    shortcut::{etb, landfall, on_attack, rally, target_any, target_filtered},
};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

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

fn ally(
    name: &'static str,
    c: crate::mana::ManaCost,
    mut types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    types.push(CreatureType::Ally);
    creature(name, c, types, p, t)
}

fn instant(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn equipment(
    name: &'static str,
    c: crate::mana::ManaCost,
    equip: crate::mana::ManaCost,
    bonus: EquipBonus,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// The ZEN Expeditions: landfall banks quest counters, three of them cash in.
fn expedition(name: &'static str, c: crate::mana::ManaCost, payoff: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: format!("Put a quest counter on {name}"),
            body: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Quest,
                amount: Value::Const(1),
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Quest, 3)),
            sac_cost: true,
            effect: payoff,
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn lands_you_control(kind: LandType) -> Value {
    Value::count(Selector::EachPermanent(R::HasLandType(kind).and(R::ControlledByYou)))
}

// ── Vanilla / keyword-only bodies ───────────────────────────────────────────

/// Shatterskull Giant — {2}{R}{R} 4/3 Giant Warrior.
pub fn shatterskull_giant() -> CardDefinition {
    creature(
        "Shatterskull Giant",
        cost(&[generic(2), r(), r()]),
        vec![CreatureType::Giant, CreatureType::Warrior],
        4,
        3,
    )
}

/// Shepherd of the Lost — {4}{W} 3/3 Angel with flying, first strike, vigilance.
pub fn shepherd_of_the_lost() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Vigilance],
        ..creature("Shepherd of the Lost", cost(&[generic(4), w()]), vec![CreatureType::Angel], 3, 3)
    }
}

/// Sky Ruin Drake — {4}{U} 2/5 Drake with flying.
pub fn sky_ruin_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature("Sky Ruin Drake", cost(&[generic(4), u()]), vec![CreatureType::Drake], 2, 5)
    }
}

/// Stonework Puma — {3} 2/2 Artifact Creature — Cat Ally.
pub fn stonework_puma() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..ally("Stonework Puma", cost(&[generic(3)]), vec![CreatureType::Cat], 2, 2)
    }
}

/// Zendikar Farguide — {4}{G} 3/3 Elemental with forestwalk.
pub fn zendikar_farguide() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        ..creature("Zendikar Farguide", cost(&[generic(4), g()]), vec![CreatureType::Elemental], 3, 3)
    }
}

/// Sphinx of Jwar Isle — {4}{U}{U} 5/5 Sphinx with flying and shroud; you may
/// look at the top card of your library any time.
pub fn sphinx_of_jwar_isle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Shroud],
        static_abilities: vec![StaticAbility {
            description: "You may look at the top card of your library any time.",
            effect: StaticEffect::MayLookAtOwnLibraryTop,
        }],
        ..creature("Sphinx of Jwar Isle", cost(&[generic(4), u(), u()]), vec![CreatureType::Sphinx], 5, 5)
    }
}

// ── Landfall ────────────────────────────────────────────────────────────────

/// Windrider Eel — {3}{U} 2/2 Fish with flying; landfall pumps it.
pub fn windrider_eel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![landfall(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Windrider Eel", cost(&[generic(3), u()]), vec![CreatureType::Fish], 2, 2)
    }
}

/// Shoal Serpent — {5}{U} 5/5 Serpent with defender; landfall sheds it.
pub fn shoal_serpent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![landfall(Effect::LoseKeyword { duration: Duration::EndOfTurn,
            what: Selector::This,
            keyword: Keyword::Defender,
        })],
        ..creature("Shoal Serpent", cost(&[generic(5), u()]), vec![CreatureType::Serpent], 5, 5)
    }
}

/// Surrakar Marauder — {1}{B} 2/1 Surrakar; landfall grants intimidate.
pub fn surrakar_marauder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Intimidate,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Surrakar Marauder", cost(&[generic(1), b()]), vec![CreatureType::Surrakar], 2, 1)
    }
}

/// Turntimber Basilisk — {1}{G}{G} 2/1 Basilisk with deathtouch; landfall
/// lures a creature into blocking it.
pub fn turntimber_basilisk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: "Target creature blocks this creature this turn if able".into(),
            body: Box::new(Effect::MustBlockTarget {
                blocker: target_filtered(R::Creature),
                attacker: Selector::This,
            }),
        })],
        ..creature("Turntimber Basilisk", cost(&[generic(1), g(), g()]), vec![CreatureType::Basilisk], 2, 1)
    }
}

/// Roil Elemental — {3}{U}{U}{U} 3/2 Elemental with flying; landfall steals a
/// creature for as long as you control this.
pub fn roil_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: "Gain control of target creature".into(),
            body: Box::new(Effect::GainControlWhileSourceRemains {
                what: target_filtered(R::Creature),
            }),
        })],
        ..creature("Roil Elemental", cost(&[generic(3), u(), u(), u()]), vec![CreatureType::Elemental], 3, 2)
    }
}

/// Soul Stair Expedition — {B}; three quest counters return two creature cards.
pub fn soul_stair_expedition() -> CardDefinition {
    expedition(
        "Soul Stair Expedition",
        cost(&[b()]),
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::InYourGraveyard),
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
        },
    )
}

/// Sunspring Expedition — {W}; three quest counters gain 8 life.
pub fn sunspring_expedition() -> CardDefinition {
    expedition(
        "Sunspring Expedition",
        cost(&[w()]),
        Effect::GainLife { who: Selector::You, amount: Value::Const(8) },
    )
}

/// Zektar Shrine Expedition — {1}{R}; three quest counters make a 7/1 trample
/// haste Elemental that's exiled at the next end step.
pub fn zektar_shrine_expedition() -> CardDefinition {
    expedition(
        "Zektar Shrine Expedition",
        cost(&[generic(1), r()]),
        Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Elemental".into(),
                    power: 7,
                    toughness: 1,
                    keywords: vec![Keyword::Trample, Keyword::Haste],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Elemental],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            Effect::ExileLastCreatedTokensAtNextEndStep,
        ]),
    )
}

// ── Allies ──────────────────────────────────────────────────────────────────

/// The Rally shape shared by Umara Raptor and Tuktuk Grunts: "you may put a
/// +1/+1 counter on this creature".
fn rally_may_grow() -> TriggeredAbility {
    rally(Effect::MayDo {
        description: "Put a +1/+1 counter on this creature".into(),
        body: Box::new(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        }),
    })
}

/// Umara Raptor — {2}{U} 1/1 Bird Ally with flying; Rally grows it.
pub fn umara_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![rally_may_grow()],
        ..ally("Umara Raptor", cost(&[generic(2), u()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Tuktuk Grunts — {4}{R} 2/2 Goblin Warrior Ally with haste; Rally grows it.
pub fn tuktuk_grunts() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![rally_may_grow()],
        ..ally(
            "Tuktuk Grunts",
            cost(&[generic(4), r()]),
            vec![CreatureType::Goblin, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Turntimber Ranger — {3}{G}{G} 2/2 Elf Scout Ranger Ally; Rally mints a Wolf
/// and grows it.
pub fn turntimber_ranger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(Effect::MayDo {
            description: "Create a 2/2 green Wolf and put a +1/+1 counter on this".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Wolf".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Green],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Wolf],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ])),
        })],
        ..ally(
            "Turntimber Ranger",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Scout, CreatureType::Ranger],
            2,
            2,
        )
    }
}

/// Tajuru Archer — {2}{G} 1/2 Elf Archer Ally; Rally snipes a flier for the
/// number of Allies you control.
pub fn tajuru_archer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(Effect::MayDo {
            description: "Deal damage equal to your Ally count to target creature with flying"
                .into(),
            body: Box::new(Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                amount: Value::count(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
                )),
            }),
        })],
        ..ally(
            "Tajuru Archer",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Archer],
            1,
            2,
        )
    }
}

// ── Kicker ──────────────────────────────────────────────────────────────────

/// Torch Slinger — {2}{R} 2/2 Goblin Shaman with kicker {1}{R}; kicked, it
/// bolts a creature for 2.
pub fn torch_slinger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), r()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(2),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Torch Slinger",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Shaman],
            2,
            2,
        )
    }
}

/// Tempest Owl — {1}{U} 1/2 Bird with flying and kicker {4}{U}; kicked, it taps
/// up to three permanents.
pub fn tempest_owl() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[generic(4), u()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::ApplyToTargets {
                max_targets: 3,
                min_targets: 0,
                filter: R::Permanent,
                effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Tempest Owl", cost(&[generic(1), u()]), vec![CreatureType::Bird], 1, 2)
    }
}

/// Vampire's Bite — {B} instant with kicker {2}{B}: +3/+0, and lifelink if kicked.
pub fn vampires_bite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2), b()]))],
        ..instant(
            "Vampire's Bite",
            cost(&[b()]),
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(3),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Lifelink,
                        duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Unstable Footing — {R} instant with kicker {3}{R}: damage can't be prevented
/// this turn, and kicked it burns a player or planeswalker for 5.
pub fn unstable_footing() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3), r()]))],
        ..instant(
            "Unstable Footing",
            cost(&[r()]),
            Effect::Seq(vec![
                Effect::DamageCantBePreventedThisTurn,
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::DealDamage {
                        to: target_filtered(R::Player.or(R::Planeswalker)),
                        amount: Value::Const(5),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Blood Tribute — {4}{B}{B} sorcery; Kicker—tap an untapped Vampire you
/// control. Halves an opponent's life, and kicked you gain that much.
pub fn blood_tribute() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::TapPermanents {
            filter: R::HasCreatureType(CreatureType::Vampire).and(R::ControlledByYou),
            count: 1,
        }),
        ..sorcery(
            "Blood Tribute",
            cost(&[generic(4), b(), b()]),
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::Drain {
                    from: Selector::Player(PlayerRef::Target(0)),
                    to: Selector::You,
                    amount: Value::HalfLifeRoundedUp(PlayerRef::Target(0)),
                }),
                else_: Box::new(Effect::LoseHalfLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    rounded_up: true,
                }),
            },
        )
    }
}

/// Gigantiform — {3}{G}{G} Aura with kicker {4}; enchanted creature is a base
/// 8/8 with trample, and kicked it fetches the next copy.
pub fn gigantiform() -> CardDefinition {
    CardDefinition {
        name: "Gigantiform",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Kicker(cost(&[generic(4)]))],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((8, 8)),
            keywords: vec![Keyword::Trample],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::MayDo {
                description: "Search your library for a card named Gigantiform".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: R::HasName("Gigantiform".into()),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    // The fetched Aura enters attached; it joins this one's host.
                    Effect::Attach {
                        what: Selector::LastMoved,
                        to: Selector::AttachedTo(Box::new(Selector::This)),
                    },
                ])),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Seismic Shudder — {1}{R}: 1 damage to each creature without flying.
pub fn seismic_shudder() -> CardDefinition {
    instant(
        "Seismic Shudder",
        cost(&[generic(1), r()]),
        Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasKeyword(
                Keyword::Flying,
            ))))),
            amount: Value::Const(1),
        },
    )
}

/// Slaughter Cry — {2}{R}: target creature gets +3/+0 and first strike.
pub fn slaughter_cry() -> CardDefinition {
    instant(
        "Slaughter Cry",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
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

/// Windborne Charge — {2}{W}{W}: two of your creatures get +2/+2 and flying.
pub fn windborne_charge() -> CardDefinition {
    sorcery(
        "Windborne Charge",
        cost(&[generic(2), w(), w()]),
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Creature.and(R::ControlledByYou),
            effect: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
    )
}

/// Spire Barrage — {4}{R}: damage equal to your Mountain count.
pub fn spire_barrage() -> CardDefinition {
    sorcery(
        "Spire Barrage",
        cost(&[generic(4), r()]),
        Effect::DealDamage { to: target_any(), amount: lands_you_control(LandType::Mountain) },
    )
}

/// Tanglesap — {1}{G}: prevent all combat damage dealt this turn by creatures
/// without trample.
pub fn tanglesap() -> CardDefinition {
    instant(
        "Tanglesap",
        cost(&[generic(1), g()]),
        Effect::PreventCombatDamageExceptDealtBy { except: R::HasKeyword(Keyword::Trample) },
    )
}

/// Shieldmate's Blessing — {W}: prevent the next 3 damage to any target.
pub fn shieldmates_blessing() -> CardDefinition {
    instant(
        "Shieldmate's Blessing",
        cost(&[w()]),
        Effect::PreventNextDamage { target: target_any(), amount: Value::Const(3) },
    )
}

/// Summoner's Bane — {2}{U}{U}: counter a creature spell, make a 2/2 Illusion.
pub fn summoners_bane() -> CardDefinition {
    instant(
        "Summoner's Bane",
        cost(&[generic(2), u(), u()]),
        Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Creature))),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Illusion".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Blue],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Illusion],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        ]),
    )
}

/// Trapmaker's Snare — {1}{U}: tutor a Trap to hand.
pub fn trapmakers_snare() -> CardDefinition {
    instant(
        "Trapmaker's Snare",
        cost(&[generic(1), u()]),
        Effect::Search {
            who: PlayerRef::You,
            filter: R::HasSpellSubtype(SpellSubtype::Trap),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Trapfinder's Trick — {1}{U}: target player reveals their hand and discards
/// all Trap cards.
pub fn trapfinders_trick() -> CardDefinition {
    sorcery(
        "Trapfinder's Trick",
        cost(&[generic(1), u()]),
        Effect::RevealHandDiscardAllMatching {
            who: PlayerRef::Target(0),
            filter: R::HasSpellSubtype(SpellSubtype::Trap),
        },
    )
}

/// Punishing Fire — {1}{R}: 2 damage anywhere, and it buys itself back out of
/// the graveyard whenever an opponent gains life.
pub fn punishing_fire() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::FromYourGraveyard)
                .from_opponent(),
            effect: Effect::MayPay {
                description: "Return Punishing Fire from your graveyard to your hand".into(),
                mana_cost: cost(&[r()]),
                body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) }),
                else_: None,
            },
        }],
        ..instant(
            "Punishing Fire",
            cost(&[generic(1), r()]),
            Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
        )
    }
}

// ── Traps ───────────────────────────────────────────────────────────────────

/// Summoning Trap — {4}{G}{G}; free if an opponent countered a creature spell
/// you cast this turn. Digs seven for a creature.
pub fn summoning_trap() -> CardDefinition {
    CardDefinition {
        name: "Summoning Trap",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Trap], ..Default::default() },
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[]),
            condition: Some(Predicate::CreatureSpellCounteredByOpponentThisTurn { who: PlayerRef::You }),
            ..Default::default()
        }),
        effect: Effect::LookTopPutMatchingOntoBattlefield {
            count: Value::Const(7),
            filter: R::Creature,
            then: None,
            max: Some(1),
            tapped: false,
            exile_rest: false,
        },
        ..Default::default()
    }
}

/// Cobra Trap — {4}{G}{G}; {G} if an opponent's spell or ability destroyed a
/// noncreature permanent of yours this turn. Four Snakes.
pub fn cobra_trap() -> CardDefinition {
    CardDefinition {
        name: "Cobra Trap",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Trap], ..Default::default() },
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[g()]),
            condition: Some(Predicate::NoncreaturePermanentDestroyedByOpponentThisTurn { who: PlayerRef::You }),
            ..Default::default()
        }),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(4),
            definition: TokenDefinition {
                name: "Snake".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Snake],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Spidersilk Net — {0} Equipment: +0/+2 and reach, equip {2}.
pub fn spidersilk_net() -> CardDefinition {
    equipment(
        "Spidersilk Net",
        cost(&[]),
        cost(&[generic(2)]),
        EquipBonus { toughness: 2, keywords: vec![Keyword::Reach], ..Default::default() },
    )
}

/// Trailblazer's Boots — {2} Equipment: nonbasic landwalk, equip {2}.
pub fn trailblazers_boots() -> CardDefinition {
    equipment(
        "Trailblazer's Boots",
        cost(&[generic(2)]),
        cost(&[generic(2)]),
        EquipBonus {
            keywords: vec![Keyword::LandwalkFiltered(Box::new(R::IsNonbasicLand))],
            ..Default::default()
        },
    )
}

/// Eternity Vessel — {6} artifact; enters with charge counters equal to your
/// life total, and landfall may reset your life to that count.
pub fn eternity_vessel() -> CardDefinition {
    CardDefinition {
        name: "Eternity Vessel",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Charge, Value::LifeOf(PlayerRef::You))),
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: "Your life total becomes the number of charge counters".into(),
            body: Box::new(Effect::SetLifeTotal {
                who: Selector::You,
                amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
            }),
        })],
        ..Default::default()
    }
}

// ── Creatures with abilities ────────────────────────────────────────────────

/// Timbermaw Larva — {3}{G} 2/2 Beast; it grows per Forest when it attacks.
pub fn timbermaw_larva() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: lands_you_control(LandType::Forest),
            toughness: lands_you_control(LandType::Forest),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Timbermaw Larva", cost(&[generic(3), g()]), vec![CreatureType::Beast], 2, 2)
    }
}

/// Gomazoa — {2}{U} 0/3 Jellyfish with defender and flying; tapping it sends
/// itself and everything it blocks to the top of their libraries.
pub fn gomazoa() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::EachPermanent(R::Creature.and(R::InCombatWithSource)),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOfMoved,
                        pos: crate::effect::LibraryPosition::Top,
                    },
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOfMoved,
                        pos: crate::effect::LibraryPosition::Top,
                    },
                },
            ]),
            ..Default::default()
        }],
        ..creature("Gomazoa", cost(&[generic(2), u()]), vec![CreatureType::Jellyfish], 0, 3)
    }
}

/// Kalitas, Bloodchief of Ghet — {5}{B}{B} 5/5 Vampire Warrior; his removal
/// leaves a Vampire the same size behind.
pub fn kalitas_bloodchief_of_ghet() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[b(), b(), b()]),
            effect: Effect::Seq(vec![
                Effect::DestroyAndRemember { what: target_filtered(R::Creature) },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::PermanentsDestroyedThisResolution,
                        Value::Const(1),
                    ),
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                        definition: TokenDefinition {
                            name: "Vampire".into(),
                            power: 0,
                            toughness: 0,
                            card_types: vec![CardType::Creature],
                            colors: vec![Color::Black],
                            subtypes: Subtypes {
                                creature_types: vec![CreatureType::Vampire],
                                ..Default::default()
                            },
                            dynamic_pt: Some((
                                Value::SacrificedPower,
                                Value::SacrificedToughness,
                            )),
                            ..Default::default()
                        },
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Kalitas, Bloodchief of Ghet",
            cost(&[generic(5), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Warrior],
            5,
            5,
        )
    }
}

/// Lullmage Mentor — {1}{U}{U} 2/2 Merfolk Wizard; your counterspells breed
/// Merfolk, and seven Merfolk counter a spell.
pub fn lullmage_mentor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCountered, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Create a 1/1 blue Merfolk".into(),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Merfolk".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Blue],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Merfolk],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_permanents_cost: Some((
                R::HasCreatureType(CreatureType::Merfolk).and(R::ControlledByYou),
                7,
            )),
            effect: Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            ..Default::default()
        }],
        ..creature(
            "Lullmage Mentor",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// World Queller — {3}{W}{W} 4/4 Avatar; each upkeep it names a card type and
/// every player sacrifices one.
pub fn world_queller() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Choose a card type; each player sacrifices a permanent of it".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::ChooseCardTypeForSource,
                    Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::EachPlayer),
                        count: Value::Const(1),
                        filter: R::IsSourceChosenCardType,
                    },
                ])),
            },
        }],
        ..creature("World Queller", cost(&[generic(3), w(), w()]), vec![CreatureType::Avatar], 4, 4)
    }
}

/// Obsidian Fireheart — {1}{R}{R}{R} 4/4 Elemental; it sets lands ablaze, and
/// they keep burning after it's gone.
pub fn obsidian_fireheart() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r(), r()]),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Land.and(R::Not(Box::new(R::WithCounter(
                        CounterType::Blaze,
                    ))))),
                    kind: CounterType::Blaze,
                    amount: Value::Const(1),
                },
                Effect::GrantTriggeredAbility {
                    what: Selector::Target(0),
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                            EventScope::YourControl,
                        )
                        .with_filter(Predicate::SourceHasCountersAtLeast {
                            counter: CounterType::Blaze,
                            n: 1,
                        }),
                        effect: Effect::DealDamage {
                            to: Selector::Player(PlayerRef::You),
                            amount: Value::Const(1),
                        },
                    }),
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Obsidian Fireheart",
            cost(&[generic(1), r(), r(), r()]),
            vec![CreatureType::Elemental],
            4,
            4,
        )
    }
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// Sejiri Refuge — the WU life-gain refuge.
pub fn sejiri_refuge() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::White]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Blue]),
                },
                ..Default::default()
            },
        ],
        ..super::wwk::tapped_etb_land(
            "Sejiri Refuge",
            Color::White,
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        )
    }
}

/// Soaring Seacliff — enters tapped, taps for {U}, and gives a creature flying.
pub fn soaring_seacliff() -> CardDefinition {
    super::wwk::tapped_etb_land(
        "Soaring Seacliff",
        Color::Blue,
        Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Teetering Peaks — enters tapped, taps for {R}, and pumps a creature +2/+0.
pub fn teetering_peaks() -> CardDefinition {
    super::wwk::tapped_etb_land(
        "Teetering Peaks",
        Color::Red,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Turntimber Grove — enters tapped, taps for {G}, and pumps a creature +1/+1.
pub fn turntimber_grove() -> CardDefinition {
    super::wwk::tapped_etb_land(
        "Turntimber Grove",
        Color::Green,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Oran-Rief, the Vastwood — enters tapped, taps for {G}, and counters up every
/// green creature that entered this turn.
pub fn oran_rief_the_vastwood() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Green]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::HasColor(Color::Green)).and(R::EnteredThisTurn),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
        ],
        ..super::wwk::tapped_etb_land("Oran-Rief, the Vastwood", Color::Green, Effect::Noop)
    }
}

/// Magosi, the Waterveil — enters tapped, taps for {U}; bank an eon counter to
/// skip a turn, then cash it in for an extra one.
pub fn magosi_the_waterveil() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Blue]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[u()]),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Eon,
                        amount: Value::Const(1),
                    },
                    Effect::SkipTurns { who: PlayerRef::You, count: Value::Const(1) },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Eon, 1)),
                bounce_self_cost: true,
                effect: Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::Const(1) },
                ..Default::default()
            },
        ],
        ..super::wwk::tapped_etb_land("Magosi, the Waterveil", Color::Blue, Effect::Noop)
    }
}

// ── Planeswalkers ───────────────────────────────────────────────────────────

/// Chandra Ablaze — {4}{R}{R} loyalty 5.
pub fn chandra_ablaze() -> CardDefinition {
    CardDefinition {
        name: "Chandra Ablaze",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Planeswalker],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Chandra],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::If {
                        cond: Predicate::LastDiscardedWasColor(Color::Red),
                        then: Box::new(Effect::DealDamage {
                            to: target_any(),
                            amount: Value::Const(4),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::Const(100),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::Const(3),
                    },
                ]),
            },
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: -7,
                effect: Effect::CastAnyOrderWithoutPaying {
                    what: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::HasColor(Color::Red).and(
                            R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                        ),
                    },
                    source_zone: crate::card::Zone::Graveyard,
                    filter: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Nissa Revane — {2}{G}{G} loyalty 2.
pub fn nissa_revane() -> CardDefinition {
    CardDefinition {
        name: "Nissa Revane",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Planeswalker],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Nissa],
            ..Default::default()
        },
        base_loyalty: 2,
        loyalty_abilities: vec![
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: 1,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasName("Nissa's Chosen".into()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            },
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: 1,
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Times(
                        Box::new(Value::Const(2)),
                        Box::new(Value::count(Selector::EachPermanent(
                            R::HasCreatureType(CreatureType::Elf).and(R::ControlledByYou),
                        ))),
                    ),
                },
            },
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: -7,
                effect: Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Elf)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    count: Value::Const(99),
                },
            },
        ],
        ..Default::default()
    }
}

/// Sorin Markov — {3}{B}{B}{B} loyalty 4.
pub fn sorin_markov() -> CardDefinition {
    CardDefinition {
        name: "Sorin Markov",
        cost: cost(&[generic(3), b(), b(), b()]),
        card_types: vec![CardType::Planeswalker],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Sorin],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            },
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: -3,
                effect: Effect::SetLifeTotal {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(10),
                },
            },
            LoyaltyAbility {
                x_cost: false,
                loyalty_cost: -7,
                effect: Effect::ControlPlayerNextTurn { who: PlayerRef::Target(0) },
            },
        ],
        ..Default::default()
    }
}
