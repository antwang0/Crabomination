//! Betrayers of Kamigawa (BOK) gap closure, wave 2. Tests in `classic_sets/bok2`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, SpellSubtype, StaticAbility, Subtypes, Supertype, TriggeredAbility,
    Value, WardCost,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, StaticEffect, ZoneDest,
    shortcut::{etb, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

use super::bok::{arcane_instant, creature, instant, legend, sorcery};

/// "At the beginning of your upkeep, `effect`."
pub(crate) fn on_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect,
    }
}

/// "You control a [land type]."
fn controls_land(ty: LandType) -> Predicate {
    Predicate::SelectorExists(Selector::ControlledBy {
        who: PlayerRef::You,
        filter: R::HasLandType(ty),
    })
}

// ── Ninja (CR 702.49) ───────────────────────────────────────────────────────

/// Higure, the Still Wind — {3}{U}{U} 3/4 Ninja with ninjutsu {2}{U}{U}.
/// Combat damage tutors a Ninja; {2} makes a Ninja unblockable.
pub fn higure_the_still_wind() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(2), u(), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Search your library for a Ninja card?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Ninja),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Ninja))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Higure, the Still Wind",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Human, CreatureType::Ninja],
            3,
            4,
        )
    }
}

/// Ink-Eyes, Servant of Oni — {4}{B}{B} 5/4 Ninja with ninjutsu {3}{B}{B}.
/// Combat damage reanimates a creature out of that player's graveyard.
pub fn ink_eyes_servant_of_oni() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(3), b(), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Reanimate a creature from that player's graveyard?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::take(
                        Selector::CardsInZone {
                            who: PlayerRef::Target(0),
                            zone: crate::card::Zone::Graveyard,
                            filter: R::Creature,
                        },
                        Value::ONE,
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..legend(
            "Ink-Eyes, Servant of Oni",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Rat, CreatureType::Ninja],
            5,
            4,
        )
    }
}

/// Okiba-Gang Shinobi — {3}{B}{B} 3/2 Ninja with ninjutsu {3}{B}. Combat
/// damage makes that player discard two cards.
pub fn okiba_gang_shinobi() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(3), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
                random: false,
            },
        }],
        ..creature(
            "Okiba-Gang Shinobi",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Rat, CreatureType::Ninja],
            3,
            2,
        )
    }
}

/// Walker of Secret Ways — {2}{U} 1/2 Ninja with ninjutsu {1}{U}. Combat
/// damage reveals that player's hand; {1}{U} rebuys a Ninja on your turn.
pub fn walker_of_secret_ways() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LookAtHand { who: Selector::Player(PlayerRef::Target(0)) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Move {
                what: target_filtered(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Ninja))
                        .and(R::ControlledByYou),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            ..Default::default()
        }],
        ..creature(
            "Walker of Secret Ways",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Ninja],
            1,
            2,
        )
    }
}

// ── Glasskites — counter the first spell or ability to target them ──────────

/// Jetting Glasskite — {4}{U}{U} 4/4 flier that counters the first spell or
/// ability to target it each turn.
pub fn jetting_glasskite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CounterFirstTargetingEachTurn],
        ..creature(
            "Jetting Glasskite",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Shimmering Glasskite — {3}{U} 2/3 flier that counters the first spell or
/// ability to target it each turn.
pub fn shimmering_glasskite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CounterFirstTargetingEachTurn],
        ..creature(
            "Shimmering Glasskite",
            cost(&[generic(3), u()]),
            vec![CreatureType::Spirit],
            2,
            3,
        )
    }
}

/// Kira, Great Glass-Spinner — {1}{U}{U} 2/2 flier. Your creatures each
/// counter the first spell or ability to target them each turn.
pub fn kira_great_glass_spinner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control counter the first spell or ability \
                          that targets them each turn.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::ControlledByYou),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::CounterFirstTargetingEachTurn],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..legend(
            "Kira, Great Glass-Spinner",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Spirit],
            2,
            2,
        )
    }
}

// ── Splice onto Arcane (CR 702.47) ──────────────────────────────────────────

/// Horobi's Whisper — {1}{B}{B} Arcane instant. Destroy target nonblack
/// creature if you control a Swamp. Splice—exile four graveyard cards.
pub fn horobis_whisper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Splice(cost(&[]), SpellSubtype::Arcane)],
        splice_extra_cost: Some(AdditionalCastCost::ExileFromGraveyard {
            filter: R::Any,
            count: 4,
        }),
        ..arcane_instant(
            "Horobi's Whisper",
            cost(&[generic(1), b(), b()]),
            Effect::If {
                cond: controls_land(LandType::Swamp),
                then: Box::new(Effect::Destroy {
                    what: target_filtered(
                        R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black)))),
                    ),
                }),
                else_: Box::new(Effect::Noop),
            },
        )
    }
}

/// Hundred-Talon Strike — {W} Arcane instant. Target creature gets +1/+0 and
/// first strike. Splice—tap an untapped white creature you control.
pub fn hundred_talon_strike() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Splice(cost(&[]), SpellSubtype::Arcane)],
        splice_extra_cost: Some(AdditionalCastCost::TapPermanents {
            filter: R::Creature.and(R::HasColor(Color::White)),
            count: 1,
        }),
        ..arcane_instant(
            "Hundred-Talon Strike",
            cost(&[w()]),
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
        )
    }
}

/// Torrent of Stone — {3}{R} Arcane instant. 4 damage to target creature.
/// Splice—sacrifice two Mountains.
pub fn torrent_of_stone() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Splice(cost(&[]), SpellSubtype::Arcane)],
        splice_extra_cost: Some(AdditionalCastCost::SacrificePermanent {
            filter: R::HasLandType(LandType::Mountain),
            count: 2,
        }),
        ..arcane_instant(
            "Torrent of Stone",
            cost(&[generic(3), r()]),
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(4),
            },
        )
    }
}

/// Roar of Jukai — {2}{G} Arcane instant. Each blocked creature gets +2/+2 if
/// you control a Forest. Splice—an opponent gains 5 life.
pub fn roar_of_jukai() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Splice(cost(&[]), SpellSubtype::Arcane)],
        splice_extra_cost: Some(AdditionalCastCost::OpponentGainsLife { amount: 5 }),
        ..arcane_instant(
            "Roar of Jukai",
            cost(&[generic(2), g()]),
            Effect::If {
                cond: controls_land(LandType::Forest),
                then: Box::new(Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::IsBlocked)),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        )
    }
}

/// Veil of Secrecy — {1}{U} Arcane instant. Target creature gains shroud and
/// can't be blocked. Splice—return a blue creature you control to hand.
pub fn veil_of_secrecy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Splice(cost(&[]), SpellSubtype::Arcane)],
        splice_extra_cost: Some(AdditionalCastCost::ReturnToHand {
            filter: R::Creature.and(R::HasColor(Color::Blue)),
            count: 1,
            count_x: false,
        }),
        ..arcane_instant(
            "Veil of Secrecy",
            cost(&[generic(1), u()]),
            Effect::GrantKeywords {
                what: target_filtered(R::Creature),
                keywords: vec![Keyword::Shroud, Keyword::Unblockable],
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Overblaze — {3}{R} Arcane instant. Target permanent's damage is doubled
/// this turn. Splice onto Arcane {2}{R}{R}.
pub fn overblaze() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Splice(cost(&[generic(2), r(), r()]), SpellSubtype::Arcane)],
        ..arcane_instant(
            "Overblaze",
            cost(&[generic(3), r()]),
            Effect::DoubleDamageFromSourceThisTurn { what: target_filtered(R::Permanent) },
        )
    }
}

// ── Other spells ────────────────────────────────────────────────────────────

/// Flames of the Blood Hand — {2}{R} Instant. 4 unpreventable damage to a
/// player or planeswalker; that player gains no life this turn.
pub fn flames_of_the_blood_hand() -> CardDefinition {
    instant(
        "Flames of the Blood Hand",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::DamageCantBePreventedThisTurn,
            Effect::LifeGainLockThisTurn { who: target_filtered(R::Player) },
            Effect::DealDamage { to: target_filtered(R::Player), amount: Value::Const(4) },
        ]),
    )
}

/// Sway of the Stars — {8}{U}{U} Sorcery. Everyone shuffles everything away,
/// draws seven, and resets to 7 life.
pub fn sway_of_the_stars() -> CardDefinition {
    sorcery(
        "Sway of the Stars",
        cost(&[generic(8), u(), u()]),
        Effect::Seq(vec![
            Effect::ShuffleEverythingOwnedIntoLibrary { who: PlayerRef::EachPlayer },
            Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(7) },
            Effect::SetLifeTotal {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(7),
            },
        ]),
    )
}

/// Twist Allegiance — {6}{R} Sorcery. You and target opponent swap creature
/// armies for the turn; they untap and gain haste.
pub fn twist_allegiance() -> CardDefinition {
    sorcery(
        "Twist Allegiance",
        cost(&[generic(6), r()]),
        Effect::ExchangeCreatureControlWith {
            who: target_filtered(R::OpponentPlayer),
            duration: Duration::EndOfTurn,
        },
    )
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Akki Raider — {1}{R} 2/1. Grows whenever a land hits a graveyard from the
/// battlefield.
pub fn akki_raider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Akki Raider",
            cost(&[generic(1), r()]),
            vec![CreatureType::Goblin, CreatureType::Warrior],
            2,
            1,
        )
    }
}

/// Empty-Shrine Kannushi — {W} 1/1 with protection from the colors of
/// permanents you control.
pub fn empty_shrine_kannushi() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ProtectionFromMatching(Box::new(
            R::SharesColorWithPermanentYouControl,
        ))],
        ..creature(
            "Empty-Shrine Kannushi",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Chisei, Heart of Oceans — {2}{U}{U} 4/4 flier that eats a counter off one
/// of your permanents every upkeep or dies.
pub fn chisei_heart_of_oceans() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_upkeep(Effect::UnlessPlayerPays {
            who: PlayerRef::You,
            cost: WardCost::RemoveCounterFromPermanent,
            then: Box::new(Effect::SacrificePermanent { what: Selector::This }),
            if_paid: None,
        })],
        ..legend(
            "Chisei, Heart of Oceans",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Spirit],
            4,
            4,
        )
    }
}

/// Ogre Marauder — {1}{B}{B} 3/1. Its attacks are unblockable unless the
/// defending player sacrifices a creature.
pub fn ogre_marauder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_attack(
            Effect::UnlessPlayerPays {
                who: PlayerRef::DefendingPlayer,
                cost: WardCost::SacrificeCreature,
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                }),
                if_paid: None,
            },
        )],
        ..creature(
            "Ogre Marauder",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Ogre, CreatureType::Warrior],
            3,
            1,
        )
    }
}

/// Shirei, Shizo's Caretaker — {4}{B} 2/2. Your little creatures come back at
/// the next end step.
pub fn shirei_shizos_caretaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtMost(1)),
                },
            ),
            effect: Effect::MayDo {
                description: "Return it at the beginning of the next end step?".into(),
                body: Box::new(Effect::AtNextEndStep {
                    body: Box::new(Effect::If {
                        cond: Predicate::SourceOnBattlefield,
                        then: Box::new(Effect::Move {
                            what: Selector::TriggerSource,
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::You,
                                tapped: false,
                            },
                        }),
                        else_: Box::new(Effect::Noop),
                    }),
                }),
            },
        }],
        ..legend(
            "Shirei, Shizo's Caretaker",
            cost(&[generic(4), b()]),
            vec![CreatureType::Spirit],
            2,
            2,
        )
    }
}

/// Iwamori of the Open Fist — {2}{G}{G} 5/5 trample. Its arrival lets each
/// opponent deploy a legend from hand.
pub fn iwamori_of_the_open_fist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::EachOpponent,
            filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
            count: Value::ONE,
            tapped: false,
            haste: false,
            sacrifice_eot: false,
                return_eot: false,
                then: None,
        })],
        ..legend(
            "Iwamori of the Open Fist",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Human, CreatureType::Monk],
            5,
            5,
        )
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

fn aura(name: &'static str, c: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..Default::default()
    }
}

/// Blessing of Leeches — {2}{B} Aura with flash. Upkeep costs you a life;
/// {0} regenerates the enchanted creature.
pub fn blessing_of_leeches() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![on_upkeep(Effect::LoseLife {
            who: Selector::You,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Regenerate { what: Selector::attached_to(Selector::This) },
            ..Default::default()
        }],
        ..aura("Blessing of Leeches", cost(&[generic(2), b()]))
    }
}

/// Mark of the Oni — {2}{B} Aura. You control the enchanted creature; it goes
/// away at the end step unless you control a Demon.
pub fn mark_of_the_oni() -> CardDefinition {
    CardDefinition {
        effect: Effect::Seq(vec![
            Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
            Effect::GainControl {
                what: target_filtered(R::Creature),
                to: None,
                duration: Duration::Permanent,
            },
        ]),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::SelectorExists(
                    Selector::ControlledBy {
                        who: PlayerRef::You,
                        filter: R::HasCreatureType(CreatureType::Demon),
                    },
                )))),
            effect: Effect::SacrificePermanent { what: Selector::This },
        }],
        ..aura("Mark of the Oni", cost(&[generic(2), b()]))
    }
}

/// Kumano's Blessing — {2}{R} Aura with flash. Creatures the enchanted
/// creature damages are exiled instead of dying.
pub fn kumanos_blessing() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::SelfSource),
                effect: Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
            }],
            ..Default::default()
        }),
        ..aura("Kumano's Blessing", cost(&[generic(2), r()]))
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Slumbering Tora — {3} Artifact. {2}, discard a Spirit or Arcane card: it
/// becomes an X/X Cat, X being the discarded card's mana value.
pub fn slumbering_tora() -> CardDefinition {
    CardDefinition {
        name: "Slumbering Tora",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            discard_cost: Some((
                R::HasCreatureType(CreatureType::Spirit)
                    .or(R::HasSpellSubtype(SpellSubtype::Arcane)),
                1,
            )),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::LastDiscardedManaValue,
                toughness: Value::LastDiscardedManaValue,
                creature_types: vec![CreatureType::Cat],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Neko-Te — {3} Equipment. Damage from the equipped creature locks creatures
/// down and drains players.
pub fn neko_te() -> CardDefinition {
    CardDefinition {
        name: "Neko-Te",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![
                TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::DealsDamageToCreature,
                        EventScope::SelfSource,
                    ),
                    effect: Effect::Seq(vec![
                        Effect::Tap { what: Selector::Target(0) },
                        Effect::SkipNextUntap { what: Selector::Target(0) },
                    ]),
                },
                TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::DealsCombatDamageToPlayer,
                        EventScope::SelfSource,
                    ),
                    effect: Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::ONE,
                    },
                },
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Flip (CR 710) ───────────────────────────────────────────────────────────

/// Budoka Pupil // Ichiga, Who Topples Oaks — {1}{G}{G} 2/2 ki-flip Monk;
/// Ichiga is a 4/3 trampler whose ki counters pump.
pub fn budoka_pupil() -> CardDefinition {
    let ichiga = CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((crate::card::CounterType::Ki, 1)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Ichiga, Who Topples Oaks",
            cost(&[]),
            vec![CreatureType::Spirit],
            4,
            3,
        )
    };
    CardDefinition {
        triggered_abilities: super::chk::ki_flip_triggers(),
        flip_face: Some(Box::new(ichiga)),
        ..creature(
            "Budoka Pupil",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Human, CreatureType::Monk],
            2,
            2,
        )
    }
}

// ── Samurai ─────────────────────────────────────────────────────────────────

/// Fumiko the Lowblood — {2}{R}{R} 3/2. Bushido X (X = attacking creatures),
/// and your opponents' creatures have to swing.
pub fn fumiko_the_lowblood() -> CardDefinition {
    let bushido_x = |kind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::count(Selector::EachPermanent(R::Creature.and(R::IsAttacking))),
            toughness: Value::count(Selector::EachPermanent(R::Creature.and(R::IsAttacking))),
            duration: Duration::EndOfTurn,
        },
    };
    CardDefinition {
        triggered_abilities: vec![
            bushido_x(EventKind::Blocks),
            bushido_x(EventKind::BecomesBlocked),
        ],
        static_abilities: vec![StaticAbility {
            description: "Creatures your opponents control attack each combat if able.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::MustAttack],
                opponents: true,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..legend(
            "Fumiko the Lowblood",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            3,
            2,
        )
    }
}

/// Kentaro, the Smiling Cat — {1}{W} 2/1 with bushido 1. Your Samurai spells
/// may be cast for generic mana equal to their mana value.
pub fn kentaro_the_smiling_cat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Bushido(1)],
        static_abilities: vec![StaticAbility {
            description: "You may pay {X} rather than pay the mana cost for Samurai \
                          spells you cast, where X is that spell's mana value.",
            effect: StaticEffect::GenericAlternativeCostForFilter {
                filter: R::HasCreatureType(CreatureType::Samurai),
            },
        }],
        ..legend(
            "Kentaro, the Smiling Cat",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            2,
            1,
        )
    }
}

/// Opal-Eye, Konda's Yojimbo — {1}{W}{W} 1/4 defender with bushido 1. Tapping
/// pulls a source's next damage onto itself; {1}{W} shrugs 1 off.
pub fn opal_eye_kondas_yojimbo() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Bushido(1)],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::PreventNextFromChosenSourceToTeam {
                    gain_life_colors: vec![],
                    amount: Value::Const(i32::MAX),
                    to: Selector::This,
                    one_event: true,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w()]),
                effect: Effect::PreventNextDamage {
                    target: Selector::This,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..legend(
            "Opal-Eye, Konda's Yojimbo",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Fox, CreatureType::Samurai],
            1,
            4,
        )
    }
}

/// Toshiro Umezawa — {1}{B}{B} 2/2 with bushido 1. An opponent's creature
/// dying lets you flash back an instant out of your graveyard.
pub fn toshiro_umezawa() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Bushido(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::GrantMayPlay {
                what: target_filtered(R::HasCardType(CardType::Instant).and(R::InYourGraveyard)),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: true,
                pay_own_cost: true,
                any_color: false,
            },
        }],
        ..legend(
            "Toshiro Umezawa",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            2,
            2,
        )
    }
}

// ── Remaining spells, auras and artifacts ───────────────────────────────────

/// Aura Barbs — {2}{R} Arcane instant. Every enchantment shocks its
/// controller, then every Aura shocks the creature it's on.
pub fn aura_barbs() -> CardDefinition {
    arcane_instant(
        "Aura Barbs",
        cost(&[generic(2), r()]),
        Effect::EnchantmentsBiteControllersAndHosts { amount: Value::Const(2) },
    )
}

/// Minamo's Meddling — {2}{U}{U} Instant. Counter target spell; its controller
/// discards every card sharing a name with one spliced onto it.
pub fn minamos_meddling() -> CardDefinition {
    instant(
        "Minamo's Meddling",
        cost(&[generic(2), u(), u()]),
        Effect::CounterSpellDiscardSplicedNames { what: target_filtered(R::IsSpellOnStack) },
    )
}

/// Shining Shoal — {X}{W}{W} Arcane instant, castable by exiling a white card
/// with mana value X. The next X damage from a chosen source is redirected.
pub fn shining_shoal() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::card::AlternativeCost {
            exile_filter: Some(R::HasColor(Color::White).and(R::ManaValueExactlyXFromCost)),
            ..Default::default()
        }),
        ..arcane_instant(
            "Shining Shoal",
            cost(&[crate::mana::x(), w(), w()]),
            Effect::PreventNextFromChosenSourceToTeam {
                    gain_life_colors: vec![],
                amount: Value::XFromCost,
                to: crate::effect::shortcut::target_any(),
                one_event: false,
            },
        )
    }
}

/// Mark of Sakiko — {1}{G} Aura. The enchanted creature's combat damage to a
/// player becomes green mana that survives the phase.
pub fn mark_of_sakiko() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::AddManaKeptThisTurnCount {
                    who: PlayerRef::You,
                    color: Color::Green,
                    amount: Value::TriggerEventAmount,
                },
            }],
            ..Default::default()
        }),
        ..aura("Mark of Sakiko", cost(&[generic(1), g()]))
    }
}

/// Ornate Kanzashi — {5} Artifact. {2}, {T}: an opponent exiles their top
/// card and you may play it this turn.
pub fn ornate_kanzashi() -> CardDefinition {
    CardDefinition {
        name: "Ornate Kanzashi",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::Target(0),
                count: Value::ONE,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blinding Powder — {1} Equipment. Unattaching it shrugs off all combat
/// damage headed at the equipped creature.
pub fn blinding_powder() -> CardDefinition {
    CardDefinition {
        name: "Blinding Powder",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                unattach_cost: true,
                effect: Effect::PreventCombatDamageToTargetThisTurn {
                    target: Selector::This,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Shuriken — {1} Equipment. Tapping and unattaching it flings 2 damage, and
/// a non-Ninja thrower loses the Equipment to the victim's controller.
pub fn shuriken() -> CardDefinition {
    CardDefinition {
        name: "Shuriken",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                unattach_cost: true,
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: target_filtered(R::Creature),
                        amount: Value::Const(2),
                    },
                    Effect::If {
                        cond: Predicate::Not(Box::new(Predicate::EntityMatches {
                            what: Selector::This,
                            filter: R::HasCreatureType(CreatureType::Ninja),
                        })),
                        then: Box::new(Effect::GainControl {
                            what: Selector::AttachmentGranting,
                            to: Some(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                            duration: Duration::Permanent,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Clash of Realities — {3}{R} Enchantment. Spirits and non-Spirits each enter
/// pointing 3 damage at the other camp.
pub fn clash_of_realities() -> CardDefinition {
    let bite = |target: R| {
        Box::new(TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Deal 3 damage?".into(),
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(target),
                    amount: Value::Const(3),
                }),
            },
        })
    };
    let spirit = R::HasCreatureType(CreatureType::Spirit);
    CardDefinition {
        name: "Clash of Realities",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "All Spirits have \"When this permanent enters, you may have it \
                              deal 3 damage to target non-Spirit creature.\"",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: spirit.clone(),
                    ability: bite(R::Creature.and(R::Not(Box::new(spirit.clone())))),
                },
            },
            StaticAbility {
                description: "Non-Spirit creatures have \"When this creature enters, you may \
                              have it deal 3 damage to target Spirit creature.\"",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: R::Creature.and(R::Not(Box::new(spirit.clone()))),
                    ability: bite(R::Creature.and(spirit)),
                },
            },
        ],
        ..Default::default()
    }
}

/// Tomorrow, Azami's Familiar — {5}{U} 1/5. Your draws become "look at three,
/// keep one, bottom the rest".
pub fn tomorrow_azamis_familiar() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card, look at the top three cards of your \
                          library instead. Put one into your hand and the rest on the bottom.",
            effect: StaticEffect::ReplaceDrawWithLookN { count: 3 },
        }],
        ..legend(
            "Tomorrow, Azami's Familiar",
            cost(&[generic(5), u()]),
            vec![CreatureType::Spirit],
            1,
            5,
        )
    }
}
