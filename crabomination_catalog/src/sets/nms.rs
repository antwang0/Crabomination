//! Nemesis (NMS), first wave. Tests in `classic_sets/nms`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, ZoneDest,
    shortcut::{etb, target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: ManaCost,
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

fn artifact(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..enchantment(name, c)
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

/// A 1/1 Human Spellshaper whose one ability is `{cost}, {T}, Discard a card:`.
fn spellshaper(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    ability_cost: ManaCost,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ability_cost,
            discard_cost: Some((R::Any, 1)),
            effect,
            ..Default::default()
        }],
        ..creature(name, c, types, 1, 1)
    }
}

/// "You control a [land type]" — the gate on the NMS free/alternative costs.
fn controls_land(who_is_you: bool, land: LandType) -> Predicate {
    let ctrl = if who_is_you { R::ControlledByYou } else { R::ControlledByOpponent };
    Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::HasLandType(land).and(ctrl)),
        n: Value::ONE,
    }
}

/// "[filter] get +P/+T" — every seat's matching permanents when `all_players`,
/// otherwise the controller's.
fn anthem(filter: R, power: i32, toughness: i32, all_players: bool) -> StaticEffect {
    StaticEffect::AnthemForFilter {
        filter,
        power,
        toughness,
        keywords: vec![],
        opponents: false,
        all_players,
        only_your_turn: false,
        scale_by_counters_on_self: None,
    }
}

// ── White ────────────────────────────────────────────────────────────────────

/// Angelic Favor — {3}{W}. A combat-trick Angel; a Plains lets you tap a
/// creature instead of paying. (The "only during combat" window is dropped.)
pub fn angelic_favor() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            tap_creatures: Some((R::Creature, 1)),
            condition: Some(controls_land(true, LandType::Plains)),
            ..Default::default()
        }),
        ..instant(
            "Angelic Favor",
            cost(&[generic(3), w()]),
            Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Angel".into(),
                        power: 4,
                        toughness: 4,
                        colors: vec![Color::White],
                        card_types: vec![CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Angel],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Flying],
                        ..Default::default()
                    },
                },
                Effect::ExileLastCreatedTokensAtNextEndStep,
            ]),
        )
    }
}

/// Avenger en-Dal — {1}{W} Spellshaper. Exiles an attacker; its controller is
/// paid back in life.
pub fn avenger_en_dal() -> CardDefinition {
    spellshaper(
        "Avenger en-Dal",
        cost(&[generic(1), w()]),
        vec![CreatureType::Human, CreatureType::Spellshaper],
        cost(&[generic(2), w()]),
        Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::ToughnessOf(Box::new(Selector::Target(0))),
            },
            Effect::Exile { what: target_filtered(R::IsAttacking) },
        ]),
    )
}

/// Blinding Angel — {3}{W}{W}. Connecting costs the defender their next combat.
pub fn blinding_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::SkipNextCombatPhase { who: PlayerRef::Target(0) },
        }],
        ..creature("Blinding Angel", cost(&[generic(3), w(), w()]), vec![CreatureType::Angel], 2, 4)
    }
}

/// Chieftain en-Dal — {1}{W}{W}. Its swing arms the whole attack with first
/// strike.
pub fn chieftain_en_dal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::IsAttacking),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Chieftain en-Dal",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Defiant Falcon — {1}{W}. A flier that fetches a cheap Rebel onto the board.
pub fn defiant_falcon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![rebel_tutor(cost(&[generic(4)]), CreatureType::Rebel, 3)],
        ..creature(
            "Defiant Falcon",
            cost(&[generic(1), w()]),
            vec![CreatureType::Rebel, CreatureType::Bird],
            1,
            1,
        )
    }
}

/// `{cost}, {T}: Search your library for a [type] permanent card with mana
/// value `max` or less, put it onto the battlefield, then shuffle.`
fn rebel_tutor(c: ManaCost, kind: CreatureType, max: u32) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        mana_cost: c,
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::PermanentCard
                .and(R::HasCreatureType(kind))
                .and(R::ManaValueAtMost(max)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Lawbringer — {2}{W}. Sacrifices itself to exile a red creature.
pub fn lawbringer() -> CardDefinition {
    color_bringer("Lawbringer", Color::Red)
}

/// Lightbringer — {2}{W}. The black-hating half of the cycle.
pub fn lightbringer() -> CardDefinition {
    color_bringer("Lightbringer", Color::Black)
}

fn color_bringer(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Exile { what: target_filtered(R::Creature.and(R::HasColor(color))) },
            ..Default::default()
        }],
        ..creature(
            name,
            cost(&[generic(2), w()]),
            vec![CreatureType::Kor, CreatureType::Rebel],
            2,
            2,
        )
    }
}

/// Lashknife — {1}{W} Aura. First strike, castable by tapping a creature.
pub fn lashknife() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            tap_creatures: Some((R::Creature, 1)),
            condition: Some(controls_land(true, LandType::Plains)),
            ..Default::default()
        }),
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![Keyword::FirstStrike],
            ..Default::default()
        }),
        ..aura("Lashknife", cost(&[generic(1), w()]))
    }
}

/// Netter en-Dal — {W} Spellshaper. Keeps one creature home for the turn.
pub fn netter_en_dal() -> CardDefinition {
    spellshaper(
        "Netter en-Dal",
        cost(&[w()]),
        vec![CreatureType::Human, CreatureType::Spellshaper],
        cost(&[w()]),
        Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::CantAttack,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Noble Stand — {4}{W}. Every block is worth two life.
pub fn noble_stand() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::YourControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..enchantment("Noble Stand", cost(&[generic(4), w()]))
    }
}

/// Off Balance — {W}. Benches a creature for the turn.
pub fn off_balance() -> CardDefinition {
    instant(
        "Off Balance",
        cost(&[w()]),
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantAttack,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Silkenfist Fighter — {1}{W}. Untaps when blocked, so it keeps blocking.
pub fn silkenfist_fighter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![untap_when_blocked()],
        ..creature(
            "Silkenfist Fighter",
            cost(&[generic(1), w()]),
            vec![CreatureType::Kor, CreatureType::Soldier],
            1,
            3,
        )
    }
}

/// Silkenfist Order — {3}{W}{W}. The bigger Silkenfist.
pub fn silkenfist_order() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![untap_when_blocked()],
        ..creature(
            "Silkenfist Order",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Kor, CreatureType::Soldier],
            3,
            5,
        )
    }
}

fn untap_when_blocked() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
        effect: Effect::Untap { what: Selector::This, up_to: None },
    }
}

/// Sivvi's Ruse — {2}{W}{W}. A one-sided fog, free against a Mountain.
pub fn sivvis_ruse() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            condition: Some(Predicate::All(vec![
                controls_land(false, LandType::Mountain),
                controls_land(true, LandType::Plains),
            ])),
            ..Default::default()
        }),
        ..instant(
            "Sivvi's Ruse",
            cost(&[generic(2), w(), w()]),
            Effect::PreventAllDamageThisTurn {
                target: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                redirect_to: None,
            },
        )
    }
}

/// Topple — {2}{W}. Exiles the biggest thing on the table.
pub fn topple() -> CardDefinition {
    sorcery(
        "Topple",
        cost(&[generic(2), w()]),
        Effect::Exile { what: target_filtered(R::Creature.and(R::HasGreatestPowerAmongAllCreatures)) },
    )
}

/// Voice of Truth — {3}{W}. A white Angel that white can't touch.
pub fn voice_of_truth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::White)],
        ..creature("Voice of Truth", cost(&[generic(3), w()]), vec![CreatureType::Angel], 2, 2)
    }
}

// ── Blue ─────────────────────────────────────────────────────────────────────

/// Air Bladder — {U} Aura. Grants flight but grounds the blocking.
pub fn air_bladder() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
            ..Default::default()
        }),
        ..aura("Air Bladder", cost(&[u()]))
    }
}

/// Cloudskate — {1}{U}. A fading flier.
pub fn cloudskate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Fading(3)],
        ..creature("Cloudskate", cost(&[generic(1), u()]), vec![CreatureType::Illusion], 2, 2)
    }
}

/// Dominate — {X}{1}{U}{U}. Steals anything cheap enough.
pub fn dominate() -> CardDefinition {
    instant(
        "Dominate",
        cost(&[x(), generic(1), u(), u()]),
        Effect::GainControl {
            what: target_filtered(R::Creature.and(R::ManaValueAtMostXFromCost)),
            to: None,
            duration: Duration::Permanent,
        },
    )
}

/// Ensnare — {3}{U}. Taps the table; two Islands buy it for free.
pub fn ensnare() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            return_to_hand: Some((R::HasLandType(LandType::Island), 2)),
            ..Default::default()
        }),
        ..instant(
            "Ensnare",
            cost(&[generic(3), u()]),
            Effect::Tap { what: Selector::EachPermanent(R::Creature) },
        )
    }
}

/// Infiltrate — {U}. One creature can't be blocked this turn.
pub fn infiltrate() -> CardDefinition {
    instant(
        "Infiltrate",
        cost(&[u()]),
        Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Unblockable,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Jolting Merfolk — {2}{U}. Fading 4 spent one tap at a time.
pub fn jolting_merfolk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fading(4)],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Fade, 1)),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature("Jolting Merfolk", cost(&[generic(2), u(), u()]), vec![CreatureType::Merfolk], 2, 2)
    }
}

/// Rootwater Commando — {2}{U}. Islandwalking Merfolk.
pub fn rootwater_commando() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        ..creature("Rootwater Commando", cost(&[generic(2), u()]), vec![CreatureType::Merfolk], 2, 2)
    }
}

/// Seahunter — {2}{U}{U}. Fishes a Merfolk out of the library.
pub fn seahunter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::PermanentCard.and(R::HasCreatureType(CreatureType::Merfolk)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Seahunter",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            2,
            2,
        )
    }
}

/// Sliptide Serpent — {4}{U}{U}. Blinks itself out of trouble.
pub fn sliptide_serpent() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            return_self_cost: false,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature("Sliptide Serpent", cost(&[generic(4), u(), u()]), vec![CreatureType::Serpent], 4, 4)
    }
}

/// Stronghold Biologist — {2}{U} Spellshaper. Counters creature spells.
pub fn stronghold_biologist() -> CardDefinition {
    spellshaper(
        "Stronghold Biologist",
        cost(&[generic(2), u()]),
        vec![CreatureType::Human, CreatureType::Spellshaper],
        cost(&[u(), u()]),
        Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::Creature)) },
    )
}

/// Stronghold Machinist — {2}{U} Spellshaper. Counters everything else.
pub fn stronghold_machinist() -> CardDefinition {
    spellshaper(
        "Stronghold Machinist",
        cost(&[generic(2), u()]),
        vec![CreatureType::Human, CreatureType::Spellshaper],
        cost(&[u(), u()]),
        Effect::CounterSpell {
            what: target_filtered(R::IsSpellOnStack.and(R::Not(Box::new(R::Creature)))),
        },
    )
}

/// Stronghold Zeppelin — {2}{U}{U}. Flies, but only blocks fliers.
pub fn stronghold_zeppelin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        ..creature("Stronghold Zeppelin", cost(&[generic(2), u(), u()]), vec![CreatureType::Human], 3, 3)
    }
}

/// Submerge — {4}{U}. Free against a Forest; buries a creature on top.
pub fn submerge() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            condition: Some(Predicate::All(vec![
                controls_land(false, LandType::Forest),
                controls_land(true, LandType::Island),
            ])),
            ..Default::default()
        }),
        ..instant(
            "Submerge",
            cost(&[generic(4), u()]),
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: crate::effect::LibraryPosition::Top,
                },
            },
        )
    }
}

/// Trickster Mage — {U} Spellshaper. Flips one permanent either way.
pub fn trickster_mage() -> CardDefinition {
    spellshaper(
        "Trickster Mage",
        cost(&[u()]),
        vec![CreatureType::Human, CreatureType::Spellshaper],
        cost(&[u()]),
        Effect::TapOrUntap { what: target_filtered(R::Permanent) },
    )
}

/// Wandering Eye — {2}{U}. Nobody hides their hand.
pub fn wandering_eye() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Players play with their hands revealed.",
            effect: StaticEffect::OpponentsPlayWithHandsRevealed,
        }],
        ..creature("Wandering Eye", cost(&[generic(2), u()]), vec![CreatureType::Illusion], 1, 3)
    }
}

// ── Black ────────────────────────────────────────────────────────────────────

/// Ascendant Evincar — {4}{B}{B}. A black anthem and a nonblack sweeper.
pub fn ascendant_evincar() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Other black creatures get +1/+1.",
                effect: anthem(R::Creature.and(R::HasColor(Color::Black)), 1, 1, true),
            },
            StaticAbility {
                description: "Nonblack creatures get -1/-1.",
                effect: anthem(
                    R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black)))),
                    -1,
                    -1,
                    true,
                ),
            },
        ],
        ..creature(
            "Ascendant Evincar",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Vampire, CreatureType::Noble],
            3,
            3,
        )
    }
}

/// Battlefield Percher — {3}{B}{B}. A pumpable anti-air Bird.
pub fn battlefield_percher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Battlefield Percher", cost(&[generic(3), b(), b()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Belbe's Percher — {2}{B}. The vanilla half of the Percher pair.
pub fn belbes_percher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        ..creature("Belbe's Percher", cost(&[generic(2), b()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Carrion Wall — {1}{B}{B}. A Wall that keeps coming back.
pub fn carrion_wall() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Carrion Wall", cost(&[generic(1), b(), b()]), vec![CreatureType::Wall], 3, 2)
    }
}

/// Dark Triumph — {4}{B}. A Swamp lets you pay in creatures instead of mana.
pub fn dark_triumph() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            sacrifice_permanents: Some((R::Creature, 1)),
            condition: Some(controls_land(true, LandType::Swamp)),
            ..Default::default()
        }),
        ..instant(
            "Dark Triumph",
            cost(&[generic(4), b()]),
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Death Pit Offering — {2}{B}{B}. Wipes your board, then makes the rebuild
/// enormous.
pub fn death_pit_offering() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SacrificeAllMatching {
            who: Selector::You,
            filter: R::Creature,
        })],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +2/+2.",
            effect: anthem(R::Creature, 2, 2, false),
        }],
        ..enchantment("Death Pit Offering", cost(&[generic(2), b(), b()]))
    }
}

/// Murderous Betrayal — {B}{B}{B}. Half your life for any nonblack creature.
pub fn murderous_betrayal() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b()]),
            half_life_cost: true,
            effect: Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
            },
            ..Default::default()
        }],
        ..enchantment("Murderous Betrayal", cost(&[b(), b(), b()]))
    }
}

/// Phyrexian Driver — {2}{B}. A one-shot Mercenary anthem.
pub fn phyrexian_driver() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Mercenary).and(R::ControlledByYou),
            ),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Phyrexian Driver",
            cost(&[generic(2), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Mercenary],
            1,
            1,
        )
    }
}

/// Phyrexian Prowler — {3}{B}. Burns its fade counters for size.
pub fn phyrexian_prowler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fading(3)],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Fade, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Phyrexian Prowler",
            cost(&[generic(3), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Mercenary],
            3,
            3,
        )
    }
}

/// Plague Witch — {1}{B} Spellshaper. Shrinks a creature.
pub fn plague_witch() -> CardDefinition {
    spellshaper(
        "Plague Witch",
        cost(&[generic(1), b()]),
        vec![CreatureType::Elf, CreatureType::Spellshaper],
        cost(&[b()]),
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Rathi Fiend — {3}{B}. Drains everyone, then hunts a Mercenary.
pub fn rathi_fiend() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(3),
        })],
        activated_abilities: vec![rebel_tutor(cost(&[generic(3)]), CreatureType::Mercenary, 3)],
        ..creature(
            "Rathi Fiend",
            cost(&[generic(3), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror, CreatureType::Mercenary],
            2,
            2,
        )
    }
}

/// Rathi Intimidator — {1}{B}{B}. Fear plus the Mercenary chain.
pub fn rathi_intimidator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        activated_abilities: vec![rebel_tutor(cost(&[generic(2)]), CreatureType::Mercenary, 2)],
        ..creature(
            "Rathi Intimidator",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror, CreatureType::Mercenary],
            2,
            1,
        )
    }
}

/// Spineless Thug — {1}{B}. Two power, no defense.
pub fn spineless_thug() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        ..creature(
            "Spineless Thug",
            cost(&[generic(1), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Mercenary],
            2,
            2,
        )
    }
}

/// Spiteful Bully — {1}{B}. A 3/3 that shoots your own board every upkeep.
pub fn spiteful_bully() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByYou)),
                amount: Value::Const(3),
            },
        }],
        ..creature(
            "Spiteful Bully",
            cost(&[generic(1), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Mercenary],
            3,
            3,
        )
    }
}

/// Stronghold Discipline — {2}{B}{B}. Everyone pays for their board.
pub fn stronghold_discipline() -> CardDefinition {
    sorcery(
        "Stronghold Discipline",
        cost(&[generic(2), b(), b()]),
        Effect::LoseLifePerControlled {
            who: Selector::Player(PlayerRef::EachPlayer),
            filter: R::Creature,
            per: Value::ONE,
        },
    )
}

/// Volrath the Fallen — {3}{B}{B}{B}. Discards creatures to grow.
pub fn volrath_the_fallen() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            discard_cost: Some((R::Creature, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::LastDiscardedManaValue,
                toughness: Value::LastDiscardedManaValue,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Volrath the Fallen",
            cost(&[generic(3), b(), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Shapeshifter],
            6,
            4,
        )
    }
}

// ── Red ──────────────────────────────────────────────────────────────────────

/// Ancient Hydra — {4}{R}. Fading 5, spent one ping at a time.
pub fn ancient_hydra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fading(5)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_cost: Some((CounterType::Fade, 1)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Ancient Hydra", cost(&[generic(4), r()]), vec![CreatureType::Hydra], 5, 1)
    }
}

/// Bola Warrior — {1}{R} Spellshaper. Clears a blocker out of the way.
pub fn bola_warrior() -> CardDefinition {
    spellshaper(
        "Bola Warrior",
        cost(&[generic(1), r()]),
        vec![CreatureType::Human, CreatureType::Spellshaper, CreatureType::Warrior],
        cost(&[r()]),
        Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Downhill Charge — {2}{R}. Sacrifice a Mountain for a Mountain-sized pump.
pub fn downhill_charge() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            sacrifice_permanents: Some((R::HasLandType(LandType::Mountain), 1)),
            ..Default::default()
        }),
        ..instant(
            "Downhill Charge",
            cost(&[generic(2), r()]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::count(Selector::EachPermanent(
                    R::HasLandType(LandType::Mountain).and(R::ControlledByYou),
                )),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Flowstone Crusher — {3}{R}. Trades toughness for power on demand.
pub fn flowstone_crusher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![flowstone_pump(cost(&[r()]), Selector::This)],
        ..creature("Flowstone Crusher", cost(&[generic(3), r(), r()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Flowstone Overseer — {2}{R}{R}{R}. Pumps anything, not just itself.
pub fn flowstone_overseer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![flowstone_pump(
            cost(&[r(), r()]),
            target_filtered(R::Creature),
        )],
        ..creature(
            "Flowstone Overseer",
            cost(&[generic(2), r(), r(), r()]),
            vec![CreatureType::Beast],
            4,
            4,
        )
    }
}

/// Flowstone Wall — {2}{R}. A 0/6 that can shrink into a beater.
pub fn flowstone_wall() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![flowstone_pump(cost(&[r()]), Selector::This)],
        ..creature("Flowstone Wall", cost(&[generic(2), r()]), vec![CreatureType::Wall], 0, 6)
    }
}

fn flowstone_pump(c: ManaCost, what: Selector) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: c,
        effect: Effect::PumpPT {
            what,
            power: Value::ONE,
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Flowstone Strike — {1}{R}. +1/-1 and haste.
pub fn flowstone_strike() -> CardDefinition {
    instant(
        "Flowstone Strike",
        cost(&[generic(1), r()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Flowstone Surge — {1}{R}. A whole-board flowstone anthem.
pub fn flowstone_surge() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/-1.",
            effect: anthem(R::Creature, 1, -1, false),
        }],
        ..enchantment("Flowstone Surge", cost(&[generic(1), r()]))
    }
}

/// Flowstone Thopter — {7}. Buys flying with its own toughness.
pub fn flowstone_thopter() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Flowstone Thopter", cost(&[generic(7)]), vec![CreatureType::Thopter], 4, 4)
    }
}

/// Mogg Alarm — {1}{R}{R}. Two Goblins; two Mountains buy it instead.
pub fn mogg_alarm() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            sacrifice_permanents: Some((R::HasLandType(LandType::Mountain), 2)),
            ..Default::default()
        }),
        ..sorcery(
            "Mogg Alarm",
            cost(&[generic(1), r(), r()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: TokenDefinition {
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
                },
            },
        )
    }
}

/// Mogg Salvage — {2}{R}. Free artifact removal against an Island.
pub fn mogg_salvage() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            condition: Some(Predicate::All(vec![
                controls_land(false, LandType::Island),
                controls_land(true, LandType::Mountain),
            ])),
            ..Default::default()
        }),
        ..instant(
            "Mogg Salvage",
            cost(&[generic(2), r()]),
            Effect::Destroy { what: target_filtered(R::Artifact) },
        )
    }
}

/// Moggcatcher — {2}{R}{R}. Fetches Goblins onto the battlefield.
pub fn moggcatcher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::PermanentCard.and(R::HasCreatureType(CreatureType::Goblin)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Moggcatcher",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            2,
            2,
        )
    }
}

/// Shrieking Mogg — {1}{R}. A hasty Goblin that taps the rest of the table.
pub fn shrieking_mogg() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::Tap {
            what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
        })],
        ..creature("Shrieking Mogg", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 1, 1)
    }
}

// ── Green ────────────────────────────────────────────────────────────────────

/// Blastoderm — {2}{G}{G}. Three turns of untouchable 5/5.
pub fn blastoderm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shroud, Keyword::Fading(3)],
        ..creature("Blastoderm", cost(&[generic(2), g(), g()]), vec![CreatureType::Beast], 5, 5)
    }
}

/// Coiling Woodworm — {2}{G}. A `*`/1 sized by every Forest in play.
pub fn coiling_woodworm() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::LandsOfTypeInPlayPower {
            land_type: LandType::Forest,
            base_t: 1,
        }),
        ..creature(
            "Coiling Woodworm",
            cost(&[generic(2), g()]),
            vec![CreatureType::Insect, CreatureType::Worm],
            0,
            1,
        )
    }
}

/// Mossdog — {G}. Grows every time an opponent points something at it.
pub fn mossdog() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::BecameTarget,
                EventScope::YourPermanentTargetedByOpponent,
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Mossdog", cost(&[g()]), vec![CreatureType::Plant, CreatureType::Dog], 1, 1)
    }
}

/// Refreshing Rain — {3}{G}. Free lifegain against a Swamp.
pub fn refreshing_rain() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            condition: Some(Predicate::All(vec![
                controls_land(false, LandType::Swamp),
                controls_land(true, LandType::Forest),
            ])),
            ..Default::default()
        }),
        ..instant(
            "Refreshing Rain",
            cost(&[generic(3), g()]),
            Effect::GainLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(6),
            },
        )
    }
}

/// Reverent Silence — {3}{G}. Blow up every enchantment, or gift 6 life each.
pub fn reverent_silence() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            opponent_gains_life: 6,
            condition: Some(controls_land(true, LandType::Forest)),
            ..Default::default()
        }),
        ..sorcery(
            "Reverent Silence",
            cost(&[generic(3), g()]),
            Effect::Destroy { what: Selector::EachPermanent(R::Enchantment) },
        )
    }
}

/// Rhox — {4}{G}{G}. Assigns damage as though unblocked, and regenerates.
pub fn rhox() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::AssignsDamageAsThoughUnblocked],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Rhox",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Rhino, CreatureType::Beast],
            5,
            5,
        )
    }
}

/// Skyshroud Behemoth — {5}{G}{G}. A 10/10 on a two-turn fuse.
pub fn skyshroud_behemoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fading(2)],
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        ..creature(
            "Skyshroud Behemoth",
            cost(&[generic(5), g(), g()]),
            vec![CreatureType::Beast],
            10,
            10,
        )
    }
}

/// Skyshroud Cutter — {3}{G}. A Forest turns it into "everyone gains 5".
pub fn skyshroud_cutter() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            opponent_gains_life: 5,
            condition: Some(controls_land(true, LandType::Forest)),
            ..Default::default()
        }),
        ..creature("Skyshroud Cutter", cost(&[generic(3), g()]), vec![CreatureType::Beast], 2, 2)
    }
}

/// Skyshroud Poacher — {2}{G}{G}. Pulls Elves straight onto the board.
pub fn skyshroud_poacher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::PermanentCard.and(R::HasCreatureType(CreatureType::Elf)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Skyshroud Poacher",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            2,
            2,
        )
    }
}

/// Skyshroud Ridgeback — {G}. A 2/3 for one, gone in two turns.
pub fn skyshroud_ridgeback() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fading(2)],
        ..creature("Skyshroud Ridgeback", cost(&[g()]), vec![CreatureType::Beast], 2, 3)
    }
}

/// Stampede Driver — {G} Spellshaper. A team pump with trample.
pub fn stampede_driver() -> CardDefinition {
    spellshaper(
        "Stampede Driver",
        cost(&[g()]),
        vec![CreatureType::Human, CreatureType::Spellshaper],
        cost(&[generic(1), g()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Treetop Bracers — {1}{G} Aura. +1/+1 and only fliers can block.
pub fn treetop_bracers() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))))],
            ..Default::default()
        }),
        ..aura("Treetop Bracers", cost(&[generic(1), g()]))
    }
}

/// Woodripper — {3}{G}{G}. Fading fuel for artifact destruction.
pub fn woodripper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fading(3)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_cost: Some((CounterType::Fade, 1)),
            effect: Effect::Destroy { what: target_filtered(R::Artifact) },
            ..Default::default()
        }],
        ..creature("Woodripper", cost(&[generic(3), g(), g()]), vec![CreatureType::Beast], 4, 6)
    }
}

// ── Artifacts / lands ────────────────────────────────────────────────────────

/// Aether Barrier — {2}{U}. Every creature spell comes with an upkeep toll.
pub fn aether_barrier() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::Triggerer,
                cost: WardCost::Mana(cost(&[generic(1)])),
                then: Box::new(Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Triggerer),
                    filter: R::Permanent,
                    count: Value::ONE,
                }),
                if_paid: None,
            },
        }],
        ..enchantment("Aether Barrier", cost(&[generic(2), u()]))
    }
}

/// Belbe's Armor — {3}. Trades a creature's power for toughness.
pub fn belbes_armor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[x()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Times(Box::new(Value::Const(-1)), Box::new(Value::XFromCost)),
                toughness: Value::XFromCost,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Belbe's Armor", cost(&[generic(3)]))
    }
}

/// Belbe's Portal — {5}. Names a type, then deploys it out of hand.
pub fn belbes_portal() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature.and(R::IsSourceChosenCreatureType),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
            },
            ..Default::default()
        }],
        ..artifact("Belbe's Portal", cost(&[generic(5)]))
    }
}

/// Complex Automaton — {4}. A 4/4 that bounces itself once you're developed.
pub fn complex_automaton() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Permanent.and(R::ControlledByYou)),
                    n: Value::Const(7),
                }),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..creature("Complex Automaton", cost(&[generic(4)]), vec![CreatureType::Golem], 4, 4)
    }
}

/// Flint Golem — {4}. Blocking it costs the defender their library top.
pub fn flint_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::Const(3),
            },
        }],
        ..creature("Flint Golem", cost(&[generic(4)]), vec![CreatureType::Golem], 2, 3)
    }
}

/// Rejuvenation Chamber — {3}. Two turns of two life.
pub fn rejuvenation_chamber() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fading(2)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..artifact("Rejuvenation Chamber", cost(&[generic(3)]))
    }
}

/// Rusting Golem — {4}. A `*`/`*` that shrinks as its fade counters go.
pub fn rusting_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Fading(5)],
        dynamic_pt: Some(DynamicPt::BasePlusCountersOnSelf {
            counter_type: CounterType::Fade,
            base_p: 0,
            base_t: 0,
            per_p: 1,
            per_t: 1,
        }),
        ..creature("Rusting Golem", cost(&[generic(4)]), vec![CreatureType::Golem], 0, 0)
    }
}

/// Viseling — {4}. Punishes a fat hand every upkeep.
pub fn viseling() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::OpponentControl,
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Sum(vec![
                    Value::HandSizeOf(PlayerRef::ActivePlayer),
                    Value::Const(-4),
                ]),
            },
        }],
        ..creature("Viseling", cost(&[generic(4)]), vec![CreatureType::Phyrexian, CreatureType::Construct], 2, 2)
    }
}

/// Rackling — {4}. Viseling's mirror: it punishes an *empty* hand.
pub fn rackling() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::OpponentControl,
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Sum(vec![
                    Value::Const(3),
                    Value::Times(
                        Box::new(Value::Const(-1)),
                        Box::new(Value::HandSizeOf(PlayerRef::ActivePlayer)),
                    ),
                ]),
            },
        }],
        ..creature("Rackling", cost(&[generic(4)]), vec![CreatureType::Phyrexian, CreatureType::Construct], 2, 2)
    }
}

/// Kor Haven — a legendary land that blanks one attacker's damage.
pub fn kor_haven() -> CardDefinition {
    CardDefinition {
        name: "Kor Haven",
        card_types: vec![CardType::Land],
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1), w()]),
                effect: Effect::PreventCombatDamageByTargetThisTurn {
                    target: target_filtered(R::IsAttacking),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rath's Edge — a legendary land that grinds lands into damage.
pub fn raths_edge() -> CardDefinition {
    CardDefinition {
        name: "Rath's Edge",
        card_types: vec![CardType::Land],
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                sac_other_filter: Some((R::Land, 1)),
                effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Terrain Generator — a land that unloads extra basics from hand.
pub fn terrain_generator() -> CardDefinition {
    CardDefinition {
        name: "Terrain Generator",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    count: Value::ONE,
                    tapped: true,
                    haste: false,
                    sacrifice_eot: false,
                return_eot: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
