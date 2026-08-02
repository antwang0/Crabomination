//! Judgment (JUD) — 2002, the set that closes the Odyssey block: the white
//! Nomad/Cleric shell, Threshold payoffs and the Dwarf tribe. Tests in
//! `classic_sets/jud`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, Keyword, LandType, Predicate, SelectionRequirement as R,
    StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Zone,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition, ManaPayload, PlayerRef,
    Selector, StaticEffect, Value, ZoneDest,
    shortcut::{draw, etb, target_any, target_filtered},
};
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

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, c)
    }
}

fn threshold() -> Predicate {
    Predicate::ThresholdActive { who: PlayerRef::You }
}

/// "Threshold — this creature gets +P/+T and has [keywords]."
fn threshold_pump(power: i32, toughness: i32, keywords: Vec<Keyword>) -> StaticAbility {
    StaticAbility {
        description: "Threshold — this creature gets a bonus.",
        effect: StaticEffect::PumpSelfIf { condition: threshold(), power, toughness, keywords },
    }
}

/// "Sacrifice this creature: [effect]."
fn sac_ability(effect: Effect) -> ActivatedAbility {
    ActivatedAbility { sac_cost: true, effect, ..Default::default() }
}

// ── White ───────────────────────────────────────────────────────────────────

/// Ancestor's Chosen — {5}{W}{W} 4/4 first strike. Life for your graveyard.
pub fn ancestors_chosen() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::GraveyardSizeOf(PlayerRef::You),
        })],
        ..creature(
            "Ancestor's Chosen",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            4,
            4,
        )
    }
}

/// Battlewise Aven — {3}{W} 2/2 flier that hardens past Threshold.
pub fn battlewise_aven() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![threshold_pump(1, 1, vec![Keyword::FirstStrike])],
        ..creature(
            "Battlewise Aven",
            cost(&[generic(3), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Benevolent Bodyguard — {W} 1/1 that trades itself for a colour of
/// protection.
pub fn benevolent_bodyguard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![sac_ability(Effect::GrantProtectionFromChosenColor {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Benevolent Bodyguard",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Border Patrol — {4}{W} 1/6 vigilance.
pub fn border_patrol() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        ..creature(
            "Border Patrol",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Nomad],
            1,
            6,
        )
    }
}

/// Cagemail — {1}{W} Aura. A bigger creature that can't swing.
pub fn cagemail() -> CardDefinition {
    aura(
        "Cagemail",
        cost(&[generic(1), w()]),
        EquipBonus { power: 2, toughness: 2, keywords: vec![Keyword::CantAttack], ..Default::default() },
    )
}

/// Chastise — {3}{W}. Kill an attacker and bank its power.
pub fn chastise() -> CardDefinition {
    instant(
        "Chastise",
        cost(&[generic(3), w()]),
        Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
            Effect::Destroy { what: target_filtered(R::Creature.and(R::IsAttacking)) },
        ]),
    )
}

/// Commander Eesha — {2}{W}{W} 2/4 flier that creatures simply can't touch.
pub fn commander_eesha() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::ProtectionFromCreatures],
        ..creature(
            "Commander Eesha",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            4,
        )
    }
}

/// Folk Medicine — {2}{G}. A life per body, twice.
pub fn folk_medicine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), w()]))],
        ..instant(
            "Folk Medicine",
            cost(&[generic(2), g()]),
            Effect::GainLife {
                who: Selector::You,
                amount: Value::CreatureCountControlledBy(PlayerRef::You),
            },
        )
    }
}

/// Funeral Pyre — {W}. Trade a graveyard card for its owner's Spirit.
pub fn funeral_pyre() -> CardDefinition {
    instant(
        "Funeral Pyre",
        cost(&[w()]),
        Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::InGraveyard), to: ZoneDest::Exile },
            Effect::CreateToken {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Spirit".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    keywords: vec![Keyword::Flying],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Spirit],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        ]),
    )
}

/// Lead Astray — {1}{W}. Tap up to two creatures.
pub fn lead_astray() -> CardDefinition {
    instant(
        "Lead Astray",
        cost(&[generic(1), w()]),
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
        },
    )
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Aven Fogbringer — {3}{U} 2/1 flier that bounces a land.
pub fn aven_fogbringer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Land),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..creature(
            "Aven Fogbringer",
            cost(&[generic(3), u()]),
            vec![CreatureType::Bird, CreatureType::Wizard],
            2,
            1,
        )
    }
}

/// Cephalid Inkshrouder — {2}{U} 2/1. A discard buys shroud and evasion.
pub fn cephalid_inkshrouder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Shroud,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Cephalid Inkshrouder", cost(&[generic(2), u()]), vec![CreatureType::Octopus], 2, 1)
    }
}

/// Defy Gravity — {U}. Flying, twice.
pub fn defy_gravity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[u()]))],
        ..instant(
            "Defy Gravity",
            cost(&[u()]),
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Envelop — {U}. Counter a sorcery.
pub fn envelop() -> CardDefinition {
    instant(
        "Envelop",
        cost(&[u()]),
        Effect::CounterSpell {
            what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Sorcery))),
        },
    )
}

/// Flash of Insight — {X}{1}{U}. Dig X, keep one.
pub fn flash_of_insight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), u()]))],
        flashback_additional_cost: vec![AdditionalCastCost::ExileFromGraveyard {
            filter: R::HasColor(Color::Blue),
            count: 1,
        }],
        ..instant(
            "Flash of Insight",
            cost(&[x(), generic(1), u()]),
            Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::XFromCost,
                rest_to_graveyard: false,
                pick_filter: None,
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
                rest_to_exile: false,
            },
        )
    }
}

/// Grip of Amnesia — {1}{U}. A counter their graveyard can buy off, and a card.
pub fn grip_of_amnesia() -> CardDefinition {
    instant(
        "Grip of Amnesia",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::CounterUnless {
                what: target_filtered(R::IsSpellOnStack),
                cost: crate::card::WardCost::ExileFromGraveyard(u32::MAX),
            },
            draw(1),
        ]),
    )
}

/// Hapless Researcher — {U} 1/1 that cashes itself in for a rummage.
pub fn hapless_researcher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![sac_ability(Effect::Seq(vec![
            draw(1),
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
        ]))],
        ..creature(
            "Hapless Researcher",
            cost(&[u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Keep Watch — {2}{U}. A card per attacker.
pub fn keep_watch() -> CardDefinition {
    instant(
        "Keep Watch",
        cost(&[generic(2), u()]),
        Effect::Draw {
            who: Selector::You,
            amount: Value::CountOf(Box::new(Selector::EachPermanent(
                R::Creature.and(R::IsAttacking),
            ))),
        },
    )
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Cabal Trainee — {B} 1/1 that trades itself for a shrink.
pub fn cabal_trainee() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![sac_ability(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Cabal Trainee",
            cost(&[b()]),
            vec![CreatureType::Human, CreatureType::Minion],
            1,
            1,
        )
    }
}

/// Earsplitting Rats — {3}{B} 2/1. A symmetrical discard, and a discard
/// regenerates it.
pub fn earsplitting_rats() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(1),
            random: false,
        })],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Earsplitting Rats", cost(&[generic(3), b()]), vec![CreatureType::Rat], 2, 1)
    }
}

/// Guiltfeeder — {3}{B}{B} 0/4 Fear. An unblocked swing drains their
/// graveyard.
pub fn guiltfeeder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::GraveyardSizeOf(PlayerRef::DefendingPlayer),
            },
        }],
        ..creature("Guiltfeeder", cost(&[generic(3), b(), b()]), vec![CreatureType::Horror], 0, 4)
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Arcane Teachings — {2}{R} Aura. +2/+2 and a tap-to-ping.
pub fn arcane_teachings() -> CardDefinition {
    aura(
        "Arcane Teachings",
        cost(&[generic(2), r()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage { to: target_any(), amount: Value::Const(1) },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Dwarven Bloodboiler — {R}{R}{R} 2/2 that taps its kin to pump.
pub fn dwarven_bloodboiler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::ControlledByYou.and(R::HasCreatureType(CreatureType::Dwarf))),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Dwarven Bloodboiler", cost(&[r(), r(), r()]), vec![CreatureType::Dwarf], 2, 2)
    }
}

/// Ember Shot — {6}{R}. Three damage and a card.
pub fn ember_shot() -> CardDefinition {
    instant(
        "Ember Shot",
        cost(&[generic(6), r()]),
        Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            draw(1),
        ]),
    )
}

/// Firecat Blitz — {X}{R}{R}. X hasty Cats for one swing.
pub fn firecat_blitz() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[r(), r()]))],
        flashback_additional_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::HasLandType(LandType::Mountain),
            count: 1,
        }],
        ..sorcery(
            "Firecat Blitz",
            cost(&[x(), r(), r()]),
            Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::XFromCost,
                    definition: TokenDefinition {
                        name: "Elemental Cat".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Red],
                        keywords: vec![Keyword::Haste],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Elemental, CreatureType::Cat],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                Effect::ExileLastCreatedTokensAtNextEndStep,
            ]),
        )
    }
}

/// Flaring Pain — {1}{R}. Switch prevention off, twice.
pub fn flaring_pain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[r()]))],
        ..instant(
            "Flaring Pain",
            cost(&[generic(1), r()]),
            Effect::TurnOffDamagePreventionThisTurn { what: Selector::None },
        )
    }
}

/// Goretusk Firebeast — {5}{R} 2/2 that arrives with a four-point punch.
pub fn goretusk_firebeast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::Player.or(R::Planeswalker)),
            amount: Value::Const(4),
        })],
        ..creature(
            "Goretusk Firebeast",
            cost(&[generic(5), r()]),
            vec![CreatureType::Elemental, CreatureType::Boar, CreatureType::Beast],
            2,
            2,
        )
    }
}

/// Jeska, Warrior Adept — {2}{R}{R} 3/1 first strike, haste, tap-to-ping.
pub fn jeska_warrior_adept() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(1) },
            ..Default::default()
        }],
        ..creature(
            "Jeska, Warrior Adept",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Human, CreatureType::Barbarian, CreatureType::Warrior],
            3,
            1,
        )
    }
}

/// Liberated Dwarf — {R} 1/1 that boosts a green creature on the way out.
pub fn liberated_dwarf() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::HasColor(Color::Green))),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Liberated Dwarf", cost(&[r()]), vec![CreatureType::Dwarf], 1, 1)
    }
}

/// Lightning Surge — {3}{R}{R}. Four, or an unpreventable six past Threshold.
pub fn lightning_surge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), r(), r()]))],
        ..sorcery(
            "Lightning Surge",
            cost(&[generic(3), r(), r()]),
            Effect::If {
                cond: threshold(),
                then: Box::new(Effect::Seq(vec![
                    Effect::TurnOffDamagePreventionThisTurn { what: Selector::None },
                    Effect::DealDamage { to: target_any(), amount: Value::Const(6) },
                ])),
                else_: Box::new(Effect::DealDamage { to: target_any(), amount: Value::Const(4) }),
            },
        )
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Battlefield Scrounger — {3}{G}{G} 3/3 that rebuys its graveyard for a pump.
pub fn battlefield_scrounger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            once_per_turn: true,
            condition: Some(threshold()),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Battlefield Scrounger",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Centaur],
            3,
            3,
        )
    }
}

/// Canopy Claws — {G}. Ground a flier, twice.
pub fn canopy_claws() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[g()]))],
        ..instant(
            "Canopy Claws",
            cost(&[g()]),
            Effect::LoseKeywordThisTurn {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
            },
        )
    }
}

/// Centaur Rootcaster — {3}{G} 2/2 that ramps off a connection.
pub fn centaur_rootcaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        }],
        ..creature(
            "Centaur Rootcaster",
            cost(&[generic(3), g()]),
            vec![CreatureType::Centaur, CreatureType::Druid],
            2,
            2,
        )
    }
}

/// Crush of Wurms — {6}{G}{G}{G}. Three 6/6s, twice.
pub fn crush_of_wurms() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(9), g(), g(), g()]))],
        ..sorcery(
            "Crush of Wurms",
            cost(&[generic(6), g(), g(), g()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: TokenDefinition {
                    name: "Wurm".into(),
                    power: 6,
                    toughness: 6,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Wurm],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )
    }
}

/// Epic Struggle — {2}{G}{G}. Twenty creatures wins the game.
pub fn epic_struggle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            )
            .with_filter(Predicate::SelectorCountAtLeast {
                sel: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                n: Value::Const(20),
            }),
            effect: Effect::WinGame { who: PlayerRef::You },
        }],
        ..enchantment("Epic Struggle", cost(&[generic(2), g(), g()]))
    }
}

/// Exoskeletal Armor — {1}{G} Aura sized by every graveyard's dead.
pub fn exoskeletal_armor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +X/+X for the dead.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::attached_to(Selector::This),
                power: Value::CardsInAllGraveyardsMatching { filter: R::Creature },
                toughness: Value::CardsInAllGraveyardsMatching { filter: R::Creature },
            },
        }],
        ..aura("Exoskeletal Armor", cost(&[generic(1), g()]), EquipBonus::default())
    }
}

/// Giant Warthog — {5}{G} 5/5 trample.
pub fn giant_warthog() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature(
            "Giant Warthog",
            cost(&[generic(5), g()]),
            vec![CreatureType::Boar, CreatureType::Beast],
            5,
            5,
        )
    }
}

/// Grizzly Fate — {3}{G}{G}. Two Bears, or four past Threshold — twice.
pub fn grizzly_fate() -> CardDefinition {
    let bears = |n: i32| Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: TokenDefinition {
            name: "Bear".into(),
            power: 2,
            toughness: 2,
            card_types: vec![CardType::Creature],
            colors: vec![Color::Green],
            subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
            ..Default::default()
        },
    };
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), g(), g()]))],
        ..sorcery(
            "Grizzly Fate",
            cost(&[generic(3), g(), g()]),
            Effect::If {
                cond: threshold(),
                then: Box::new(bears(4)),
                else_: Box::new(bears(2)),
            },
        )
    }
}

/// Harvester Druid — {1}{G} 1/1 that taps for any colour your lands make.
pub fn harvester_druid() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColorYouCouldProduce,
            },
            ..Default::default()
        }],
        ..creature(
            "Harvester Druid",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Krosan Reclamation — {1}{G}. Shuffle two cards back, twice.
pub fn krosan_reclamation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), g()]))],
        ..instant(
            "Krosan Reclamation",
            cost(&[generic(1), g()]),
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::InGraveyard,
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOfMoved,
                        pos: LibraryPosition::Shuffled,
                    },
                }),
            },
        )
    }
}

/// Krosan Wayfarer — {G} 1/1 that trades itself for a land drop.
pub fn krosan_wayfarer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![sac_ability(Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: R::Land,
            count: Value::Const(1),
            tapped: false,
            haste: false,
            sacrifice_eot: false,
            return_eot: false,
        })],
        ..creature(
            "Krosan Wayfarer",
            cost(&[g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            1,
        )
    }
}

// ── Land ────────────────────────────────────────────────────────────────────

/// Krosan Verge — enters tapped, taps for {C}, and cashes in for a Forest and
/// a Plains.
pub fn krosan_verge() -> CardDefinition {
    CardDefinition {
        name: "Krosan Verge",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
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
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: R::HasLandType(LandType::Forest),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    },
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: R::HasLandType(LandType::Plains),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Multicolor / gold ───────────────────────────────────────────────────────

/// Hunting Grounds — {G}{W}. Past Threshold, every opposing spell deploys a
/// creature from your hand.
pub fn hunting_grounds() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Threshold — deploy a creature whenever an opponent casts.",
            effect: StaticEffect::WhileCondition {
                condition: threshold(),
                inner: Box::new(StaticEffect::GrantTriggeredAbility {
                    filter: R::IsSource,
                    ability: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
                        effect: Effect::MayDo {
                            description: "Put a creature card from your hand onto the battlefield?"
                                .into(),
                            body: Box::new(Effect::PutFromHandOntoBattlefield {
                                who: PlayerRef::You,
                                filter: R::Creature,
                                count: Value::Const(1),
                                tapped: false,
                                haste: false,
                                sacrifice_eot: false,
                                return_eot: false,
                            }),
                        },
                    }),
                }),
            },
        }],
        ..enchantment("Hunting Grounds", cost(&[g(), w()]))
    }
}

/// Anurid Brushhopper — {1}{G}{W} 3/4 that blinks itself out of removal.
pub fn anurid_brushhopper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 2)),
            effect: Effect::Seq(vec![
                Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOfMoved,
                            tapped: false,
                        },
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Anurid Brushhopper",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Frog, CreatureType::Beast],
            3,
            4,
        )
    }
}

/// Balthor the Defiled — {2}{B}{B} 2/2. A Minion lord that mass-reanimates
/// black and red on the way out.
pub fn balthor_the_defiled() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Minion creatures get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Minion)),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b(), b()]),
            exile_self_cost: true,
            effect: Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::Move {
                    what: Selector::CardsInZone {
                        who: PlayerRef::Triggerer,
                        zone: Zone::Graveyard,
                        filter: R::Creature
                            .and(R::HasColor(Color::Black).or(R::HasColor(Color::Red))),
                    },
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOfMoved,
                        tapped: false,
                    },
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Balthor the Defiled",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Dwarf],
            2,
            2,
        )
    }
}

// ── The Phantom cycle ───────────────────────────────────────────────────────

/// "Enters with N +1/+1 counters. If damage would be dealt to this creature,
/// prevent that damage and remove a +1/+1 counter from it."
fn phantom(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    counters: i32,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        keywords,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(counters))),
        static_abilities: vec![StaticAbility {
            description: "Damage to this is prevented; remove a +1/+1 counter instead.",
            effect: StaticEffect::PreventDamageByRemovingCounters {
                kind: CounterType::PlusOnePlusOne,
                single: true,
            },
        }],
        ..creature(name, c, types, 0, 0)
    }
}

/// Phantom Nomad — {1}{W} 0/0 with two counters.
pub fn phantom_nomad() -> CardDefinition {
    phantom(
        "Phantom Nomad",
        cost(&[generic(1), w()]),
        vec![CreatureType::Spirit, CreatureType::Nomad],
        2,
        vec![],
    )
}

/// Phantom Flock — {3}{W}{W} 0/0 flier with three counters.
pub fn phantom_flock() -> CardDefinition {
    phantom(
        "Phantom Flock",
        cost(&[generic(3), w(), w()]),
        vec![CreatureType::Bird, CreatureType::Soldier, CreatureType::Spirit],
        3,
        vec![Keyword::Flying],
    )
}

/// Phantom Tiger — {2}{G} 1/0 with two counters.
pub fn phantom_tiger() -> CardDefinition {
    CardDefinition {
        power: 1,
        ..phantom(
            "Phantom Tiger",
            cost(&[generic(2), g()]),
            vec![CreatureType::Cat, CreatureType::Spirit],
            2,
            vec![],
        )
    }
}

/// Phantom Centaur — {2}{G}{G} 2/0 pro-black with three counters.
pub fn phantom_centaur() -> CardDefinition {
    CardDefinition {
        power: 2,
        ..phantom(
            "Phantom Centaur",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Centaur, CreatureType::Spirit],
            3,
            vec![Keyword::Protection(Color::Black)],
        )
    }
}

/// Phantom Nantuko — {2}{G} 0/0 trampler that grows itself.
pub fn phantom_nantuko() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..phantom(
            "Phantom Nantuko",
            cost(&[generic(2), g()]),
            vec![CreatureType::Insect, CreatureType::Spirit],
            2,
            vec![Keyword::Trample],
        )
    }
}

/// Phantom Nishoba — {5}{G}{W} 0/0 trampling lifelink with seven counters.
pub fn phantom_nishoba() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..phantom(
            "Phantom Nishoba",
            cost(&[generic(5), g(), w()]),
            vec![CreatureType::Cat, CreatureType::Beast, CreatureType::Spirit],
            7,
            vec![Keyword::Trample],
        )
    }
}

// ── The Advocate cycle ──────────────────────────────────────────────────────

/// "{T}: Return `n` target cards from an opponent's graveyard to their hand.
/// [payoff]" — the Judgment Advocates buy their effect with graveyard hate in
/// reverse.
fn advocate(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    give_back: u8,
    payoff: Effect,
) -> CardDefinition {
    let mut steps: Vec<Effect> = (0..give_back)
        .map(|slot| Effect::Move {
            what: Selector::TargetFiltered {
                slot,
                filter: R::InGraveyard.and(R::OwnedByYou.negate()),
            },
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })
        .collect();
    steps.push(payoff);
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(steps),
            ..Default::default()
        }],
        ..creature(name, c, types, p, t)
    }
}

/// Forcemage Advocate — {1}{G} 2/1. One card back for a +1/+1 counter.
pub fn forcemage_advocate() -> CardDefinition {
    advocate(
        "Forcemage Advocate",
        cost(&[generic(1), g()]),
        vec![CreatureType::Centaur, CreatureType::Shaman],
        2,
        1,
        1,
        Effect::AddCounter {
            what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    )
}

/// Nullmage Advocate — {2}{G} 2/3. Two cards back for a Naturalize.
pub fn nullmage_advocate() -> CardDefinition {
    advocate(
        "Nullmage Advocate",
        cost(&[generic(2), g()]),
        vec![CreatureType::Insect, CreatureType::Druid],
        2,
        3,
        2,
        Effect::Destroy {
            what: Selector::TargetFiltered {
                slot: 2,
                filter: R::Artifact.or(R::Enchantment),
            },
        },
    )
}

/// Pulsemage Advocate — {2}{W} 1/3. Three cards back for a reanimation.
pub fn pulsemage_advocate() -> CardDefinition {
    advocate(
        "Pulsemage Advocate",
        cost(&[generic(2), w()]),
        vec![CreatureType::Human, CreatureType::Cleric],
        1,
        3,
        3,
        Effect::Move {
            what: Selector::TargetFiltered {
                slot: 3,
                filter: R::InGraveyard.and(R::Creature).and(R::OwnedByYou),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Shieldmage Advocate — {2}{W} 1/3. One card back for a damage shield.
pub fn shieldmage_advocate() -> CardDefinition {
    advocate(
        "Shieldmage Advocate",
        cost(&[generic(2), w()]),
        vec![CreatureType::Human, CreatureType::Cleric],
        1,
        3,
        1,
        Effect::PreventAllDamageFromChosenSourceThisTurn {
            filter: R::Any,
            gain_life_from_colors: vec![],
        },
    )
}

// ── The rest of wave 2 ──────────────────────────────────────────────────────

/// Aven Warcraft — {2}{W}. +0/+2 for the team, plus a colour of protection
/// past Threshold.
pub fn aven_warcraft() -> CardDefinition {
    instant(
        "Aven Warcraft",
        cost(&[generic(2), w()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                power: Value::Const(0),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: threshold(),
                then: Box::new(Effect::GrantProtectionFromChosenColor {
                    what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Laquatus's Disdain — {1}{U}. Counter a flashback, and a card.
pub fn laquatuss_disdain() -> CardDefinition {
    instant(
        "Laquatus's Disdain",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::SpellNotCastFromHand)),
            },
            draw(1),
        ]),
    )
}

/// Masked Gorgon — {4}{B} 5/5 that green and white can't touch, and that
/// can't be touched back past Threshold.
pub fn masked_gorgon() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Green and white creatures have protection from Gorgons.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        R::Creature
                            .and(R::HasColor(Color::Green).or(R::HasColor(Color::White))),
                    ),
                    keyword: Keyword::ProtectionFromMatching(Box::new(R::HasCreatureType(
                        CreatureType::Gorgon,
                    ))),
                },
            },
            threshold_pump(0, 0, vec![
                Keyword::Protection(Color::Green),
                Keyword::Protection(Color::White),
            ]),
        ],
        ..creature("Masked Gorgon", cost(&[generic(4), b()]), vec![CreatureType::Gorgon], 5, 5)
    }
}

/// Mirari's Wake — {3}{G}{W}. An anthem and a mana doubler in one.
pub fn miraris_wake() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Whenever you tap a land for mana, add one of the same.",
                effect: StaticEffect::ExtraManaOnLandTap {
                    enchanted_only: false,
                    filter: R::Land.and(R::ControlledByYou),
                    extra: crate::effect::ExtraManaKind::Mirror,
                    while_monarch: false,
                },
            },
        ],
        ..enchantment("Mirari's Wake", cost(&[generic(3), g(), w()]))
    }
}

/// Mirror Wall — {3}{U} 3/4 Defender that white mana lets off the leash.
pub fn mirror_wall() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::LoseKeywordThisTurn {
                what: Selector::This,
                keyword: Keyword::Defender,
            },
            ..Default::default()
        }],
        ..creature("Mirror Wall", cost(&[generic(3), u()]), vec![CreatureType::Wall], 3, 4)
    }
}

/// Nantuko Monastery — a land that stands up as a 4/4 first striker past
/// Threshold.
pub fn nantuko_monastery() -> CardDefinition {
    CardDefinition {
        name: "Nantuko Monastery",
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
                mana_cost: cost(&[g(), w()]),
                condition: Some(threshold()),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    creature_types: vec![CreatureType::Insect, CreatureType::Monk],
                    keywords: vec![Keyword::FirstStrike],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Nantuko Tracer — {1}{G} 2/1 that bottoms a graveyard card.
pub fn nantuko_tracer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Put a card from a graveyard on the bottom of its owner's library?".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(R::InGraveyard),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::Bottom,
                },
            }),
        })],
        ..creature(
            "Nantuko Tracer",
            cost(&[generic(1), g()]),
            vec![CreatureType::Insect, CreatureType::Druid],
            2,
            1,
        )
    }
}

/// Nomad Mythmaker — {2}{W} 2/2 that replays Auras out of any graveyard.
pub fn nomad_mythmaker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::AttachAuraFromGraveyardTo {
                aura: target_filtered(
                    R::InGraveyard.and(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
                ),
                host: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByYou),
                },
            },
            ..Default::default()
        }],
        ..creature(
            "Nomad Mythmaker",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Nomad, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Quiet Speculation — {1}{U}. Stock a graveyard with flashback.
pub fn quiet_speculation() -> CardDefinition {
    sorcery(
        "Quiet Speculation",
        cost(&[generic(1), u()]),
        Effect::SearchUpToN {
            who: PlayerRef::Target(0),
            filter: R::HasFlashback,
            to: ZoneDest::Graveyard,
            count: Value::Const(3),
        },
    )
}

/// Rats' Feast — {X}{B}. Eat X cards out of one graveyard.
pub fn rats_feast() -> CardDefinition {
    sorcery(
        "Rats' Feast",
        cost(&[x(), b()]),
        Effect::MoveChosen {
            from: Selector::CardsInZone {
                who: PlayerRef::Target(0),
                zone: Zone::Graveyard,
                filter: R::Any,
            },
            filter: None,
            count: Value::XFromCost,
            up_to: true,
            to: ZoneDest::Exile,
        },
    )
}

/// Ray of Revelation — {1}{W}. Naturalize an enchantment, twice.
pub fn ray_of_revelation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[g()]))],
        ..instant(
            "Ray of Revelation",
            cost(&[generic(1), w()]),
            Effect::Destroy { what: target_filtered(R::Enchantment) },
        )
    }
}

/// Serene Sunset — {X}{G}. Fog X attackers.
pub fn serene_sunset() -> CardDefinition {
    instant(
        "Serene Sunset",
        cost(&[x(), g()]),
        Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::PreventCombatDamageByTargetThisTurn {
                target: Selector::Target(0),
            }),
        },
    )
}

// ── Wave 3 ──────────────────────────────────────────────────────────────────

/// The Judgment punisher shape: "any player may have this deal N damage to
/// them. If no one does, [otherwise]."
fn punisher(who: PlayerRef, amount: i32, otherwise: Effect) -> Effect {
    Effect::AnyPlayerMayTakeDamageElse {
        who,
        amount: Value::Const(amount),
        otherwise: Box::new(otherwise),
    }
}

/// Book Burning — {1}{R}. Six damage, or six cards.
pub fn book_burning() -> CardDefinition {
    sorcery(
        "Book Burning",
        cost(&[generic(1), r()]),
        punisher(
            PlayerRef::EachPlayer,
            6,
            Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(6) },
        ),
    )
}

/// Breaking Point — {1}{R}{R}. Six damage, or a Wrath.
pub fn breaking_point() -> CardDefinition {
    sorcery(
        "Breaking Point",
        cost(&[generic(1), r(), r()]),
        punisher(
            PlayerRef::EachPlayer,
            6,
            Effect::DestroyNoRegen { what: Selector::EachPermanent(R::Creature) },
        ),
    )
}

/// Dwarven Driller — {3}{R} 2/2. A land, or two damage to its controller.
pub fn dwarven_driller() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: punisher(
                PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                2,
                Effect::Destroy { what: target_filtered(R::Land) },
            ),
            ..Default::default()
        }],
        ..creature("Dwarven Driller", cost(&[generic(3), r()]), vec![CreatureType::Dwarf], 2, 2)
    }
}

/// Dwarven Scorcher — {R} 1/1. A ping, or two to the face.
pub fn dwarven_scorcher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![sac_ability(punisher(
            PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
            2,
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(1) },
        ))],
        ..creature("Dwarven Scorcher", cost(&[r()]), vec![CreatureType::Dwarf], 1, 1)
    }
}

/// Silver Seraph — {5}{W}{W}{W} 6/6 flier that anthems the team past
/// Threshold.
pub fn silver_seraph() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Threshold — other creatures you control get +2/+2.",
            effect: StaticEffect::WhileCondition {
                condition: threshold(),
                inner: Box::new(StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    power: 2,
                    toughness: 2,
                }),
            },
        }],
        ..creature(
            "Silver Seraph",
            cost(&[generic(5), w(), w(), w()]),
            vec![CreatureType::Angel],
            6,
            6,
        )
    }
}

/// Stitch Together — {B}{B}. A creature back to hand, or straight to play
/// past Threshold.
pub fn stitch_together() -> CardDefinition {
    sorcery(
        "Stitch Together",
        cost(&[b(), b()]),
        Effect::If {
            cond: threshold(),
            then: Box::new(Effect::Move {
                what: target_filtered(R::InGraveyard.and(R::Creature).and(R::OwnedByYou)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
            else_: Box::new(Effect::Move {
                what: target_filtered(R::InGraveyard.and(R::Creature).and(R::OwnedByYou)),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
    )
}

/// Sudden Strength — {3}{G}. +3/+3 and a card.
pub fn sudden_strength() -> CardDefinition {
    instant(
        "Sudden Strength",
        cost(&[generic(3), g()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Swelter — {3}{R}. Two damage to each of two creatures.
pub fn swelter() -> CardDefinition {
    sorcery(
        "Swelter",
        cost(&[generic(3), r()]),
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Creature,
            effect: Box::new(Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(2),
            }),
        },
    )
}

/// Swirling Sandstorm — {3}{R}. A five-point ground sweep, past Threshold.
pub fn swirling_sandstorm() -> CardDefinition {
    sorcery(
        "Swirling Sandstorm",
        cost(&[generic(3), r()]),
        Effect::If {
            cond: threshold(),
            then: Box::new(Effect::ForEach {
                selector: Selector::EachPermanent(
                    R::Creature.and(R::HasKeyword(Keyword::Flying).negate()),
                ),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(5),
                }),
            }),
            else_: Box::new(Effect::Noop),
        },
    )
}

/// Thriss, Nantuko Primus — {5}{G}{G} 5/5 that hands out +5/+5.
pub fn thriss_nantuko_primus() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(5),
                toughness: Value::Const(5),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Thriss, Nantuko Primus",
            cost(&[generic(5), g(), g()]),
            vec![CreatureType::Insect, CreatureType::Druid],
            5,
            5,
        )
    }
}

/// Toxic Stench — {1}{B}. A shrink, or a kill past Threshold.
pub fn toxic_stench() -> CardDefinition {
    instant(
        "Toxic Stench",
        cost(&[generic(1), b()]),
        Effect::If {
            cond: threshold(),
            then: Box::new(Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Black).negate())),
            }),
            else_: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Black).negate())),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            }),
        },
    )
}

/// Trained Pronghorn — {1}{W} 1/1 that discards for a shield.
pub fn trained_pronghorn() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::PreventAllDamageThisTurn {
                target: Selector::This,
                redirect_to: None,
            },
            ..Default::default()
        }],
        ..creature("Trained Pronghorn", cost(&[generic(1), w()]), vec![CreatureType::Antelope], 1, 1)
    }
}

/// "Threshold — this creature gets +2/+2 and has 'When this dies, you lose
/// `life` life.'"
fn treacherous(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    life: i32,
) -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            threshold_pump(2, 2, vec![]),
            StaticAbility {
                description: "Threshold — when this dies, you lose life.",
                effect: StaticEffect::WhileCondition {
                    condition: threshold(),
                    inner: Box::new(StaticEffect::GrantTriggeredAbility {
                        filter: R::IsSource,
                        ability: Box::new(TriggeredAbility {
                            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                            effect: Effect::LoseLife {
                                who: Selector::You,
                                amount: Value::Const(life),
                            },
                        }),
                    }),
                },
            },
        ],
        ..creature(name, c, types, p, t)
    }
}

/// Treacherous Werewolf — {2}{B} 2/2 that costs you 4 on the way out past
/// Threshold.
pub fn treacherous_werewolf() -> CardDefinition {
    treacherous(
        "Treacherous Werewolf",
        cost(&[generic(2), b()]),
        vec![CreatureType::Werewolf, CreatureType::Minion],
        2,
        2,
        4,
    )
}

/// Treacherous Vampire — {4}{B} 4/4 flier that eats its own graveyard to
/// stay in combat.
pub fn treacherous_vampire() -> CardDefinition {
    let feed = || TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::SacrificeSourceUnlessCost {
            cost: crate::card::WardCost::ExileFromGraveyard(1),
        },
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            feed(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                ..feed()
            },
        ],
        ..treacherous(
            "Treacherous Vampire",
            cost(&[generic(4), b()]),
            vec![CreatureType::Vampire],
            4,
            4,
            6,
        )
    }
}

/// Tunneler Wurm — {6}{G}{G} 6/6 that discards to regenerate.
pub fn tunneler_wurm() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Tunneler Wurm", cost(&[generic(6), g(), g()]), vec![CreatureType::Wurm], 6, 6)
    }
}

/// Venomous Vines — {2}{G}{G}. Kill anything wearing an Aura.
pub fn venomous_vines() -> CardDefinition {
    sorcery(
        "Venomous Vines",
        cost(&[generic(2), g(), g()]),
        Effect::Destroy { what: target_filtered(R::IsEnchanted) },
    )
}

/// Vigilant Sentry — {1}{W}{W} 2/2 that becomes a combat trick past
/// Threshold.
pub fn vigilant_sentry() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            threshold_pump(1, 1, vec![]),
            StaticAbility {
                description: "Threshold — {T}: a combatant gets +3/+3.",
                effect: StaticEffect::WhileCondition {
                    condition: threshold(),
                    inner: Box::new(StaticEffect::GrantActivatedAbility {
                        applies_to: Selector::This,
                        condition: None,
                        ability: ActivatedAbility {
                            tap_cost: true,
                            effect: Effect::PumpPT {
                                what: target_filtered(
                                    R::Creature.and(R::IsAttacking.or(R::IsBlocking)),
                                ),
                                power: Value::Const(3),
                                toughness: Value::Const(3),
                                duration: Duration::EndOfTurn,
                            },
                            ..Default::default()
                        },
                    }),
                },
            },
        ],
        ..creature(
            "Vigilant Sentry",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Nomad],
            2,
            2,
        )
    }
}

/// Spurnmage Advocate — {W} 1/1. Two cards back to kill an attacker.
pub fn spurnmage_advocate() -> CardDefinition {
    advocate(
        "Spurnmage Advocate",
        cost(&[w()]),
        vec![CreatureType::Human, CreatureType::Nomad],
        1,
        1,
        2,
        Effect::Destroy {
            what: Selector::TargetFiltered {
                slot: 2,
                filter: R::Creature.and(R::IsAttacking),
            },
        },
    )
}

/// Spirit Cairn — {2}{W}. Every discard is a Spirit for {W}.
pub fn spirit_cairn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::AnyPlayer),
            effect: Effect::MayPay {
                description: "Pay {W} for a Spirit?".into(),
                mana_cost: cost(&[w()]),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Spirit".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White],
                        keywords: vec![Keyword::Flying],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Spirit],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                }),
                else_: None,
            },
        }],
        ..enchantment("Spirit Cairn", cost(&[generic(2), w()]))
    }
}

/// Soulcatchers' Aerie — {1}{W}. Every dead Bird makes the flock bigger.
pub fn soulcatchers_aerie() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Bird),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Feather,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Bird creatures get +1/+1 for each feather counter.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Bird)),
                power: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Feather,
                },
                toughness: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Feather,
                },
            },
        }],
        ..enchantment("Soulcatchers' Aerie", cost(&[generic(1), w()]))
    }
}

// ── The Wormfang cycle ──────────────────────────────────────────────────────

/// "When this enters, exile [what]. When it leaves the battlefield, return
/// the exiled cards."
fn wormfang(
    body: CardDefinition,
    hostage: Selector,
    back_to: crate::card::ExileReturnZone,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: hostage,
            return_to: back_to,
        })],
        ..body
    }
}

/// Wormfang Behemoth — {3}{U}{U} 5/5 that holds your hand hostage.
pub fn wormfang_behemoth() -> CardDefinition {
    wormfang(
        creature(
            "Wormfang Behemoth",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Nightmare, CreatureType::Fish, CreatureType::Beast],
            5,
            5,
        ),
        Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Hand, filter: R::Any },
        crate::card::ExileReturnZone::Hand,
    )
}

/// Wormfang Newt — {1}{U} 2/2 that holds a land hostage.
pub fn wormfang_newt() -> CardDefinition {
    wormfang(
        creature(
            "Wormfang Newt",
            cost(&[generic(1), u()]),
            vec![CreatureType::Nightmare, CreatureType::Salamander, CreatureType::Beast],
            2,
            2,
        ),
        Selector::Take {
            inner: Box::new(Selector::ControlledBy { who: PlayerRef::You, filter: R::Land }),
            count: Box::new(Value::Const(1)),
        },
        crate::card::ExileReturnZone::Battlefield,
    )
}

/// Wormfang Turtle — {2}{U} 2/4 that holds a land hostage.
pub fn wormfang_turtle() -> CardDefinition {
    wormfang(
        creature(
            "Wormfang Turtle",
            cost(&[generic(2), u()]),
            vec![CreatureType::Nightmare, CreatureType::Turtle, CreatureType::Beast],
            2,
            4,
        ),
        Selector::Take {
            inner: Box::new(Selector::ControlledBy { who: PlayerRef::You, filter: R::Land }),
            count: Box::new(Value::Const(1)),
        },
        crate::card::ExileReturnZone::Battlefield,
    )
}

/// Wormfang Drake — {2}{U} 3/4 flier that holds one of your creatures
/// hostage.
pub fn wormfang_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExileUntilSourceLeaves {
                what: Selector::Take {
                    inner: Box::new(Selector::ControlledBy {
                        who: PlayerRef::You,
                        filter: R::Creature.and(R::OtherThanSource),
                    }),
                    count: Box::new(Value::Const(1)),
                },
                return_to: crate::card::ExileReturnZone::Battlefield,
            },
            Effect::If {
                cond: Predicate::Not(Box::new(Predicate::SelectorExists(
                    Selector::CardExiledWithSource,
                ))),
                then: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: R::IsSource,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature(
            "Wormfang Drake",
            cost(&[generic(2), u()]),
            vec![CreatureType::Nightmare, CreatureType::Drake],
            3,
            4,
        )
    }
}

/// Worldgorger Dragon — {3}{R}{R}{R} 7/7 that eats your whole board while
/// it's out.
pub fn worldgorger_dragon() -> CardDefinition {
    wormfang(
        CardDefinition {
            keywords: vec![Keyword::Flying, Keyword::Trample],
            ..creature(
                "Worldgorger Dragon",
                cost(&[generic(3), r(), r(), r()]),
                vec![CreatureType::Nightmare, CreatureType::Dragon],
                7,
                7,
            )
        },
        Selector::ControlledBy {
            who: PlayerRef::You,
            filter: R::Permanent.and(R::OtherThanSource),
        },
        crate::card::ExileReturnZone::Battlefield,
    )
}
