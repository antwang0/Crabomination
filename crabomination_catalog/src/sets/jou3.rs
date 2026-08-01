//! Journey into Nyx (JOU) wave 3 — the bestow tail, the Auras, and the rares
//! that each wanted one primitive. Tests in `classic_sets/jou`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, ConditionalEquipBonus, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, Keyword, PlaneswalkerSubtype, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, heroic, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ExtraManaKind, LoyaltyAbility, PlayerRef,
    Selector, ZoneDest, ZoneRef,
};
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    mana: ManaCost,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: ct,
            ..Default::default()
        },
        power: p,
        toughness: t,
        keywords: kw,
        ..Default::default()
    }
}

/// A JOU bestow creature: an enchantment creature that can be cast as an Aura.
fn bestow_creature(
    name: &'static str,
    mana: ManaCost,
    bestow_cost: ManaCost,
    pt: (i32, i32),
    ct: Vec<CreatureType>,
    bonus: EquipBonus,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        bestow: Some(bestow_cost),
        equipped_bonus: Some(bonus),
        ..creature(name, mana, pt.0, pt.1, ct, vec![])
    }
}

/// A plain Aura with a resolution-time attach and a static bonus.
fn aura(name: &'static str, mana: ManaCost, enchant: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(enchant),
        },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

// ── Bestow ───────────────────────────────────────────────────────────────────

/// Sightless Brawler — {1}{W} 3/2 Human Warrior. Bestow {4}{W}; it and its
/// host can't attack alone.
pub fn sightless_brawler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantAttackAlone],
        ..bestow_creature(
            "Sightless Brawler",
            cost(&[generic(1), w()]),
            cost(&[generic(4), w()]),
            (3, 2),
            vec![CreatureType::Human, CreatureType::Warrior],
            EquipBonus {
                power: 3,
                toughness: 2,
                keywords: vec![Keyword::CantAttackAlone],
                ..Default::default()
            },
        )
    }
}

/// Spirespine — {2}{G} 4/1 Beast. Bestow {4}{G}; it and its host block each
/// combat if able.
pub fn spirespine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustBlock],
        ..bestow_creature(
            "Spirespine",
            cost(&[generic(2), g()]),
            cost(&[generic(4), g()]),
            (4, 1),
            vec![CreatureType::Beast],
            EquipBonus {
                power: 4,
                toughness: 1,
                keywords: vec![Keyword::MustBlock],
                ..Default::default()
            },
        )
    }
}

/// Crystalline Nautilus — {2}{U} 4/4 Nautilus. Bestow {3}{U}{U}; it and its
/// host are sacrificed when targeted.
pub fn crystalline_nautilus() -> CardDefinition {
    let sac_when_targeted = TriggeredAbility {
        event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
        effect: Effect::SacrificeSource,
    };
    CardDefinition {
        triggered_abilities: vec![sac_when_targeted.clone()],
        ..bestow_creature(
            "Crystalline Nautilus",
            cost(&[generic(2), u()]),
            cost(&[generic(3), u(), u()]),
            (4, 4),
            vec![CreatureType::Nautilus],
            EquipBonus {
                power: 4,
                toughness: 4,
                triggered_abilities: vec![sac_when_targeted],
                triggers_on_equipment: true,
                ..Default::default()
            },
        )
    }
}

/// Hypnotic Siren — {U} 1/1 Siren with flying. Bestow {5}{U}{U}; the bestowed
/// host is stolen and gets +1/+1 and flying.
pub fn hypnotic_siren() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        ..bestow_creature(
            "Hypnotic Siren",
            cost(&[u()]),
            cost(&[generic(5), u(), u()]),
            (1, 1),
            vec![CreatureType::Siren],
            EquipBonus {
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Flying],
                ..Default::default()
            },
        )
    }
}

// ── Auras ────────────────────────────────────────────────────────────────────

/// Armament of Nyx — {2}{W} Aura. The enchanted creature has double strike
/// while it's an enchantment; otherwise its damage is prevented.
pub fn armament_of_nyx() -> CardDefinition {
    aura(
        "Armament of Nyx",
        cost(&[generic(2), w()]),
        R::Creature,
        EquipBonus {
            conditional: vec![
                ConditionalEquipBonus {
                    host_filter: R::Enchantment,
                    keywords: vec![Keyword::DoubleStrike],
                    ..Default::default()
                },
                ConditionalEquipBonus {
                    host_filter: R::Enchantment.negate(),
                    keywords: vec![Keyword::DealsNoCombatDamage],
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
    )
}

/// An Aura that gives +1/+1 and cashes itself in on combat damage to destroy
/// a permanent (Flamespeaker's Will, Mortal Obstinacy).
fn sac_on_damage_aura(name: &'static str, mana: ManaCost, kill: R) -> CardDefinition {
    aura(
        name,
        mana,
        R::Creature.and(R::ControlledByYou),
        EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: format!("Sacrifice {name} to destroy a permanent?"),
                    body: Box::new(Effect::Seq(vec![
                        Effect::SacrificeSource,
                        Effect::Destroy {
                            what: target_filtered(kill),
                        },
                    ])),
                },
            }],
            triggers_on_equipment: true,
            ..Default::default()
        },
    )
}

/// Flamespeaker's Will — {R} Aura. +1/+1; on combat damage, sacrifice it to
/// destroy target artifact.
pub fn flamespeakers_will() -> CardDefinition {
    sac_on_damage_aura("Flamespeaker's Will", cost(&[r()]), R::Artifact)
}

/// Mortal Obstinacy — {W} Aura. +1/+1; on combat damage, sacrifice it to
/// destroy target enchantment.
pub fn mortal_obstinacy() -> CardDefinition {
    sac_on_damage_aura("Mortal Obstinacy", cost(&[w()]), R::Enchantment)
}

// ── Enchantments and artifacts ───────────────────────────────────────────────

/// Dictate of Karametra — {3}{G}{G} Enchantment with flash. Every land tapped
/// for mana produces an extra mana of a type it made.
pub fn dictate_of_karametra() -> CardDefinition {
    CardDefinition {
        name: "Dictate of Karametra",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "Whenever a player taps a land for mana, that player adds one mana of \
                          any type that land produced.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: false,
                filter: R::Any,
                extra: ExtraManaKind::Mirror,
                while_monarch: false,
            },
        }],
        ..Default::default()
    }
}

/// Deserter's Quarters — {2} Artifact. {6}, {T}: Tap target creature; it stays
/// tapped while this artifact does.
pub fn deserters_quarters() -> CardDefinition {
    CardDefinition {
        name: "Deserter's Quarters",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            tap_cost: true,
            effect: Effect::TapAndUntapLock {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Godsend — {1}{W}{W} legendary Equipment. +3/+3; exile a creature it blocks
/// or is blocked by, and lock opponents out of that name. Equip {3}.
pub fn godsend() -> CardDefinition {
    CardDefinition {
        name: "Godsend",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Exile a creature Godsend's bearer is in combat with?".into(),
                    body: Box::new(Effect::ExileWithSource {
                        what: Selector::CreaturesInCombatWith(Box::new(Selector::TriggerSource)),
                    }),
                },
            }],
            triggers_on_equipment: true,
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't cast spells with the same name as a card exiled \
                          with this Equipment.",
            effect: StaticEffect::OpponentsCantCastNamed,
        }],
        ..Default::default()
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Battlefield Thaumaturge — {1}{U} 2/1 Human Wizard. Your instants and
/// sorceries cost {1} less per creature they target; heroic grants hexproof.
pub fn battlefield_thaumaturge() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each instant and sorcery spell you cast costs {1} less to cast for \
                          each creature it targets.",
            effect: StaticEffect::YourISSpellsCostLessPerTargetCreature { amount: 1 },
        }],
        triggered_abilities: vec![heroic(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Hexproof,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Battlefield Thaumaturge",
            cost(&[generic(1), u()]),
            2,
            1,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Dakra Mystic — {U} 1/1 Merfolk Wizard. {U}, {T}: reveal each top card; mill
/// them, or every player draws.
pub fn dakra_mystic() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::MayDoElse {
                description: "Put the revealed cards into their owners' graveyards?".into(),
                body: Box::new(Effect::Mill {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Draw {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(1),
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Dakra Mystic",
            cost(&[u()]),
            1,
            1,
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            vec![],
        )
    }
}

/// "Inspired — whenever this creature becomes untapped, …" (CR 702.108).
fn inspired(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
        effect,
    }
}

/// Daring Thief — {2}{U} 2/3 Human Rogue. Inspired: swap a nonland permanent
/// you control for an opponent's permanent sharing a card type.
pub fn daring_thief() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired(Effect::MayDo {
            description: "Exchange control of two permanents?".into(),
            body: Box::new(Effect::ExchangeControlChoosing {
                filter: R::Permanent.and(R::ControlledByOpponent),
                with: target_filtered(R::Nonland.and(R::ControlledByYou)),
            }),
        })],
        ..creature(
            "Daring Thief",
            cost(&[generic(2), u()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Disciple of Deceit — {U}{B} 1/3 Human Rogue. Inspired: discard a nonland
/// card to tutor a card with the same mana value.
pub fn disciple_of_deceit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![inspired(Effect::MayDiscardMatching {
            description: "Discard a nonland card to tutor for its mana value?".into(),
            count: Value::Const(1),
            filter: R::Nonland,
            then: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::ManaValueEqualsDiscardedThisEffect,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: None,
        })],
        ..creature(
            "Disciple of Deceit",
            cost(&[u(), b()]),
            1,
            3,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Nessian Game Warden — {3}{G}{G} 4/5 Beast. ETB: look at the top X (X =
/// Forests you control), take a creature card.
pub fn nessian_game_warden() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::count(Selector::EachMatching {
                zone: ZoneRef::Battlefield,
                filter: R::HasLandType(crate::card::LandType::Forest).and(R::ControlledByYou),
            }),
            rest_to_graveyard: false,
            pick_filter: Some(R::Creature),
            take: Some(Value::Const(1)),
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: true,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
            rest_to_exile: false,
        })],
        ..creature(
            "Nessian Game Warden",
            cost(&[generic(3), g(), g()]),
            4,
            5,
            vec![CreatureType::Beast],
            vec![],
        )
    }
}

/// Prophetic Flamespeaker — {1}{R}{R} 1/3 Human Shaman with double strike and
/// trample. Combat damage to a player exiles the top card to play this turn.
pub fn prophetic_flamespeaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(1),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        ..creature(
            "Prophetic Flamespeaker",
            cost(&[generic(1), r(), r()]),
            1,
            3,
            vec![CreatureType::Human, CreatureType::Shaman],
            vec![Keyword::DoubleStrike, Keyword::Trample],
        )
    }
}

/// Quarry Colossus — {5}{W}{W} 5/6 Giant. ETB: bury a creature beneath the top
/// X cards of its owner's library, X = Plains you control.
pub fn quarry_colossus() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::PutIntoLibraryBeneathTop {
            what: target_filtered(R::Creature),
            count: Value::count(Selector::EachMatching {
                zone: ZoneRef::Battlefield,
                filter: R::HasLandType(crate::card::LandType::Plains).and(R::ControlledByYou),
            }),
        })],
        ..creature(
            "Quarry Colossus",
            cost(&[generic(5), w(), w()]),
            5,
            6,
            vec![CreatureType::Giant],
            vec![],
        )
    }
}

/// Sage of Hours — {1}{U} 1/1 Human Wizard. Heroic grows it; remove all its
/// +1/+1 counters for an extra turn per five removed.
pub fn sage_of_hours() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Seq(vec![
                Effect::TakeExtraTurn {
                    who: PlayerRef::You,
                    count: Value::DivDown(
                        Box::new(Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::PlusOnePlusOne,
                        }),
                        5,
                    ),
                },
                Effect::RemoveAllCounters {
                    what: Selector::This,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Sage of Hours",
            cost(&[generic(1), u()]),
            1,
            1,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Scourge of Fleets — {5}{U}{U} 6/6 Kraken. ETB: bounce each opposing
/// creature with toughness X or less, X = Islands you control.
pub fn scourge_of_fleets() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent).and(
                R::ToughnessAtMostYourCount(Box::new(
                    R::HasLandType(crate::card::LandType::Island).and(R::ControlledByYou),
                )),
            )),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..creature(
            "Scourge of Fleets",
            cost(&[generic(5), u(), u()]),
            6,
            6,
            vec![CreatureType::Kraken],
            vec![],
        )
    }
}

/// Stormchaser Chimera — {2}{U}{R} 2/3 Chimera with flying. {2}{U}{R}: scry 1,
/// then get +X/+0 where X is the new top card's mana value.
pub fn stormchaser_chimera() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), r()]),
            effect: Effect::Seq(vec![
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ManaValueOf(Box::new(Selector::TopOfLibrary {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                    })),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Stormchaser Chimera",
            cost(&[generic(2), u(), r()]),
            2,
            3,
            vec![CreatureType::Chimera],
            vec![Keyword::Flying],
        )
    }
}

/// Ajani, Mentor of Heroes — {3}{G}{W} planeswalker, loyalty 4.
pub fn ajani_mentor_of_heroes() -> CardDefinition {
    CardDefinition {
        name: "Ajani, Mentor of Heroes",
        cost: cost(&[generic(3), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Ajani],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::DistributeCounters {
                    total: Value::Const(3),
                    counter: CounterType::PlusOnePlusOne,
                    filter: R::Creature.and(R::ControlledByYou),
                    max_targets: 3,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::LookPickToHand {
                    who: PlayerRef::You,
                    count: Value::Const(4),
                    rest_to_graveyard: false,
                    pick_filter: Some(
                        R::Creature
                            .or(R::Planeswalker)
                            .or(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
                    ),
                    take: Some(Value::Const(1)),
                    to_battlefield: false,
                    gain_life_if_pick: None,
                    gain_life_greatest_power_rest: false,
                    optional: true,
                    picked_lands_to_battlefield: false,
                    rest_bottom_random: false,
                    rest_to_exile: false,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(100),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
