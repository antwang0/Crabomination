//! Worldwake (WWK) gap closure — the Traps, the Ally/landfall rares and the
//! remaining commons. Tests in `classic_sets/wwk`.

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EntersAsCopy, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    LandType, MayPlayDuration, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, StaticEffect, ZoneDest,
    shortcut::{etb, landfall, rally, target_any, target_filtered},
};
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> crate::card::CardDefinition {
    crate::card::CardDefinition {
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
) -> crate::card::CardDefinition {
    types.push(CreatureType::Ally);
    creature(name, c, types, p, t)
}

fn aura(name: &'static str, c: crate::mana::ManaCost) -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Allies you control (the recurring Ally-matters count).
fn ally_count() -> Value {
    Value::count(Selector::EachPermanent(
        R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
    ))
}

// ── Traps ───────────────────────────────────────────────────────────────────

/// A Trap's alternative cost: "If [condition], you may pay [cost] rather than
/// pay this spell's mana cost."
fn trap(cost_paid: crate::mana::ManaCost, condition: Predicate) -> AlternativeCost {
    AlternativeCost { mana_cost: cost_paid, condition: Some(condition), ..Default::default() }
}

/// An attacking creature matching `filter` (the two board-state Trap gates).
fn attacker_matching(filter: R) -> Predicate {
    Predicate::SelectorExists(Selector::EachPermanent(
        R::Creature.and(R::IsAttacking).and(filter),
    ))
}

/// Nemesis Trap — {4}{B}{B} Instant. {B}{B} if a white creature is attacking.
/// Exile target attacking creature and take a copy until end of turn.
pub fn nemesis_trap() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Nemesis Trap",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Trap],
            ..Default::default()
        },
        alternative_cost: Some(trap(
            cost(&[b(), b()]),
            attacker_matching(R::HasColor(Color::White)),
        )),
        effect: Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::Const(1),
                source: target_filtered(R::Creature.and(R::IsAttacking)),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
            Effect::Exile { what: Selector::Target(0) },
            Effect::ExileLastCreatedTokensAtNextEndStep,
        ]),
        ..Default::default()
    }
}

/// Permafrost Trap — {2}{U}{U} Instant. {U} if an opponent had a green creature
/// enter this turn. Tap up to two target creatures; they skip their next untap.
pub fn permafrost_trap() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Permafrost Trap",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Trap],
            ..Default::default()
        },
        alternative_cost: Some(trap(
            cost(&[u()]),
            Predicate::CreatureEnteredThisTurnMatching {
                who: PlayerRef::EachOpponent,
                filter: R::HasColor(Color::Green),
            },
        )),
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::Target(0) },
                Effect::SkipNextUntap { what: Selector::Target(0) },
            ])),
        },
        ..Default::default()
    }
}

/// Refraction Trap — {3}{W} Instant. {W} if an opponent cast a red instant or
/// sorcery this turn. Prevent the next 3 damage from a source of your choice
/// and fire it back at any target.
pub fn refraction_trap() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Refraction Trap",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Trap],
            ..Default::default()
        },
        alternative_cost: Some(trap(
            cost(&[w()]),
            Predicate::CastSpellThisTurnWith {
                who: PlayerRef::EachOpponent,
                colors: vec![Color::Red],
                types: vec![CardType::Instant, CardType::Sorcery],
            },
        )),
        effect: Effect::PreventNextFromChosenSourceToTeam {
                    gain_life_colors: vec![],
            amount: Value::Const(3),
            to: target_any(),
            one_event: false,
        },
        ..Default::default()
    }
}

/// Ricochet Trap — {3}{R} Instant. {R} if an opponent cast a blue spell this
/// turn. Change the target of target spell with a single target.
pub fn ricochet_trap() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Ricochet Trap",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Trap],
            ..Default::default()
        },
        alternative_cost: Some(trap(
            cost(&[r()]),
            Predicate::CastSpellThisTurnWith {
                who: PlayerRef::EachOpponent,
                colors: vec![Color::Blue],
                types: vec![],
            },
        )),
        effect: Effect::ChooseNewTargetsForSpell {
            what: target_filtered(R::IsSpellOnStack.and(R::SpellWithSingleTarget)),
        },
        ..Default::default()
    }
}

/// Slingbow Trap — {3}{G} Instant. {G} if a black creature with flying is
/// attacking. Destroy target attacking creature with flying.
pub fn slingbow_trap() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Slingbow Trap",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Trap],
            ..Default::default()
        },
        alternative_cost: Some(trap(
            cost(&[g()]),
            attacker_matching(R::HasColor(Color::Black).and(R::HasKeyword(Keyword::Flying))),
        )),
        effect: Effect::Destroy {
            what: target_filtered(
                R::Creature.and(R::IsAttacking).and(R::HasKeyword(Keyword::Flying)),
            ),
        },
        ..Default::default()
    }
}

/// Stone Idol Trap — {5}{R} Instant, {1} less per attacking creature. Make a
/// 6/12 trampling Construct until your next end step.
pub fn stone_idol_trap() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Stone Idol Trap",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Trap],
            ..Default::default()
        },
        affinity_filter: Some(R::Creature.and(R::IsAttacking)),
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Box::new(TokenDefinition {
                    name: "Construct".into(),
                    power: 6,
                    toughness: 12,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Construct],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Trample],
                    ..Default::default()
                }),
            },
            Effect::ExileLastCreatedTokensAtNextEndStep,
        ]),
        ..Default::default()
    }
}

// ── Allies ──────────────────────────────────────────────────────────────────

/// Agadeem Occultist — {2}{B} 0/2 Human Shaman Ally. {T}: reanimate a creature
/// card from an opponent's graveyard with mana value at most your Ally count.
pub fn agadeem_occultist() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(
                    R::Creature
                        .and(R::InOpponentGraveyard)
                        .and(R::ManaValueAtMostControlledCount(Box::new(R::HasCreatureType(
                            CreatureType::Ally,
                        )))),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..ally("Agadeem Occultist", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Shaman], 0, 2)
    }
}

/// Jwari Shapeshifter — {1}{U} Shapeshifter Ally. May enter as a copy of any
/// Ally creature on the battlefield.
pub fn jwari_shapeshifter() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.and(R::HasCreatureType(CreatureType::Ally)),
            ..Default::default()
        }),
        ..ally("Jwari Shapeshifter", cost(&[generic(1), u()]), vec![CreatureType::Shapeshifter], 0, 0)
    }
}

/// Talus Paladin — {3}{W} 2/3 Human Knight Ally. Rally: your Allies may gain
/// lifelink and this may grow.
pub fn talus_paladin() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        triggered_abilities: vec![rally(Effect::Seq(vec![
            Effect::MayDo {
                description: "Allies you control gain lifelink".into(),
                body: Box::new(Effect::GrantKeyword {
                    what: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
                    ),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                }),
            },
            Effect::MayDo {
                description: "Put a +1/+1 counter on Talus Paladin".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            },
        ]))],
        ..ally("Talus Paladin", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 3)
    }
}

/// Tuktuk Scrapper — {3}{R} 2/2 Goblin Artificer Ally. Rally: destroy target
/// artifact and shock its controller for your Ally count.
pub fn tuktuk_scrapper() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        triggered_abilities: vec![rally(Effect::MayDo {
            description: "Destroy target artifact".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Artifact) },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::InGraveyard,
                    },
                    then: Box::new(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                        amount: ally_count(),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
        ..ally("Tuktuk Scrapper", cost(&[generic(3), r()]), vec![CreatureType::Goblin, CreatureType::Artificer], 2, 2)
    }
}

/// Vastwood Animist — {2}{G} 1/1 Elf Shaman Ally. {T}: a land you control
/// becomes an X/X Elemental, X = your Ally count.
pub fn vastwood_animist() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeCreature {
                what: target_filtered(R::Land.and(R::ControlledByYou)),
                power: ally_count(),
                toughness: ally_count(),
                creature_types: vec![CreatureType::Elemental],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..ally("Vastwood Animist", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Shaman], 1, 1)
    }
}

// ── Multikicker ─────────────────────────────────────────────────────────────

/// Marshal's Anthem — {2}{W}{W} Enchantment with multikicker {1}{W}. Anthem;
/// ETB returns one creature card per kick from your graveyard.
pub fn marshals_anthem() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Marshal's Anthem",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Multikicker(cost(&[generic(1), w()]))],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![etb(Effect::CapTargetsAt {
            amount: Value::TimesKicked,
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 4,
                min_targets: 0,
                filter: R::Creature.and(R::InYourGraveyard),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            }),
        })],
        ..Default::default()
    }
}

/// Spell Contortion — {2}{U} Instant with multikicker {1}{U}. Counter unless
/// {2} is paid; draw one card per kick.
pub fn spell_contortion() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Spell Contortion",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Multikicker(cost(&[generic(1), u()]))],
        effect: Effect::Seq(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(2)]),
                exile: false,
                extra_generic: None,
            },
            Effect::Draw { who: Selector::You, amount: Value::TimesKicked },
        ]),
        ..Default::default()
    }
}

/// Strength of the Tajuru — {X}{G}{G} Instant with multikicker {1}. X +1/+1
/// counters on one creature plus one more creature per kick.
pub fn strength_of_the_tajuru() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Strength of the Tajuru",
        cost: cost(&[x(), g(), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Multikicker(cost(&[generic(1)]))],
        effect: Effect::CapTargetsAt {
            amount: Value::Sum(vec![Value::Const(1), Value::TimesKicked]),
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 5,
                min_targets: 1,
                filter: R::Creature,
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::XFromCost,
                }),
            }),
        },
        ..Default::default()
    }
}

/// Voyager Drake — {3}{U} 3/3 Drake with flying and multikicker {U}. ETB grants
/// flying to one creature per kick.
pub fn voyager_drake() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Multikicker(cost(&[u()]))],
        triggered_abilities: vec![etb(Effect::CapTargetsAt {
            amount: Value::TimesKicked,
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 4,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                }),
            }),
        })],
        ..creature("Voyager Drake", cost(&[generic(3), u()]), vec![CreatureType::Drake], 3, 3)
    }
}

/// Rumbling Aftershocks — {4}{R} Enchantment. Whenever you cast a kicked spell,
/// it may deal damage equal to the kick count to any target.
pub fn rumbling_aftershocks() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Rumbling Aftershocks",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellWasKicked),
            effect: Effect::MayDo {
                description: "Deal damage equal to the kick count".into(),
                body: Box::new(Effect::DealDamage {
                    to: target_any(),
                    amount: Value::CastSpellTimesKicked,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Landfall / untappers ────────────────────────────────────────────────────

/// Scrib Nibblers — {2}{B} 1/1 Rat. {T}: exile a player's top card, gaining 1
/// life off a land. Landfall untaps it.
pub fn scrib_nibblers() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(1),
                    link_to_source: false,
                    face_down: false,
                },
                Effect::If {
                    cond: Predicate::EntityMatchesAny {
                        what: Selector::LastMoved,
                        filter: R::Land,
                    },
                    then: Box::new(Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: "Untap Scrib Nibblers".into(),
            body: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
        })],
        ..creature("Scrib Nibblers", cost(&[generic(2), b()]), vec![CreatureType::Rat], 1, 1)
    }
}

/// Tideforce Elemental — {2}{U} 2/1 Elemental. {U}, {T}: tap or untap another
/// creature. Landfall untaps it.
pub fn tideforce_elemental() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::MayDo {
                description: "Tap or untap target creature".into(),
                body: Box::new(Effect::TapOrUntap {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                }),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: "Untap Tideforce Elemental".into(),
            body: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
        })],
        ..creature("Tideforce Elemental", cost(&[generic(2), u()]), vec![CreatureType::Elemental], 2, 1)
    }
}

/// Tomb Hex — {2}{B} Instant. −2/−2, or −4/−4 with landfall.
pub fn tomb_hex() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Tomb Hex",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(
                R::Land.and(R::ControlledByYou).and(R::EnteredThisTurn),
            )),
            then: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-4),
                toughness: Value::Const(-4),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

// ── Combat-damage riders ────────────────────────────────────────────────────

/// Hammer of Ruin — {2} Equipment. +2/+0; on connect, destroy an Equipment the
/// damaged player controls. Equip {2}.
pub fn hammer_of_ruin() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Hammer of Ruin",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Destroy target Equipment that player controls".into(),
                    body: Box::new(Effect::Destroy {
                        what: target_filtered(
                            R::HasArtifactSubtype(ArtifactSubtype::Equipment)
                                .and(R::ControlledByTriggerPlayer),
                        ),
                    }),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Mordant Dragon — {3}{R}{R}{R} 5/5 Dragon with flying and firebreathing. On
/// connect, it may repeat the damage onto a creature that player controls.
pub fn mordant_dragon() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Deal that much damage to a creature that player controls".into(),
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)),
                    amount: Value::TriggerEventAmount,
                }),
            },
        }],
        ..creature("Mordant Dragon", cost(&[generic(3), r(), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Shoreline Salvager — {3}{B} 3/3 Surrakar. On connect, draw a card if you
/// control an Island.
pub fn shoreline_salvager() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasLandType(LandType::Island).and(R::ControlledByYou),
                ))),
            effect: Effect::MayDo {
                description: "Draw a card".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            },
        }],
        ..creature("Shoreline Salvager", cost(&[generic(3), b()]), vec![CreatureType::Surrakar], 3, 3)
    }
}

/// Slavering Nulls — {1}{R} 2/1 Goblin Zombie. On connect, make that player
/// discard if you control a Swamp.
pub fn slavering_nulls() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasLandType(LandType::Swamp).and(R::ControlledByYou),
                ))),
            effect: Effect::MayDo {
                description: "That player discards a card".into(),
                body: Box::new(Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(1),
                    random: false,
                }),
            },
        }],
        ..creature("Slavering Nulls", cost(&[generic(1), r()]), vec![CreatureType::Goblin, CreatureType::Zombie], 2, 1)
    }
}

/// Thada Adel, Acquisitor — {1}{U}{U} 2/2 Merfolk Rogue with islandwalk. On
/// connect, steal an artifact out of that player's library for the turn.
pub fn thada_adel_acquisitor() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::SearchPickedBy {
                    who: PlayerRef::Target(0),
                    picker: PlayerRef::You,
                    filter: R::Artifact,
                    to: ZoneDest::Exile,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ]),
        }],
        ..creature("Thada Adel, Acquisitor", cost(&[generic(1), u(), u()]), vec![CreatureType::Merfolk, CreatureType::Rogue], 2, 2)
    }
}

/// Wrexial, the Risen Deep — {3}{U}{U}{B} 5/8 Kraken with islandwalk and
/// swampwalk. On connect, free-cast an instant or sorcery from that graveyard.
pub fn wrexial_the_risen_deep() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![
            Keyword::Landwalk(LandType::Island),
            Keyword::Landwalk(LandType::Swamp),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CastWithoutPayingImmediate {
                reduce_generic: 0,
                                pay_own_cost: false,
                what: target_filtered(
                    (R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))
                        .and(R::InGraveyard)
                        .and(R::ControlledByTriggerPlayer),
                ),
                source_zone: Zone::Graveyard,
                exile_after: true,
                copy: false,
            },
        }],
        ..creature("Wrexial, the Risen Deep", cost(&[generic(3), u(), u(), b()]), vec![CreatureType::Kraken], 5, 8)
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Dead Reckoning — {1}{B}{B} Sorcery. Put a creature card from your graveyard
/// on top of your library; it deals its power to target creature.
pub fn dead_reckoning() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Dead Reckoning",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::MayDo {
            description: "Put that creature card on top of your library".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                },
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Library {
                        who: PlayerRef::You,
                        pos: crate::effect::LibraryPosition::Top,
                    },
                },
            ])),
        },
        ..Default::default()
    }
}

/// Feral Contest — {3}{G} Sorcery. Grow a creature you control; another
/// creature must block it this turn.
pub fn feral_contest() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Feral Contest",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::MustBlockTarget {
                blocker: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                attacker: Selector::Target(0),
            },
        ]),
        ..Default::default()
    }
}

/// Mire's Toll — {B} Sorcery. Target player reveals one card per Swamp you
/// control; you pick one to discard.
pub fn mires_toll() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Mire's Toll",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DiscardChosenFromRevealed {
            from: Selector::Player(PlayerRef::Target(0)),
            reveal: Value::count(Selector::EachPermanent(
                R::HasLandType(LandType::Swamp).and(R::ControlledByYou),
            )),
        },
        ..Default::default()
    }
}

/// Treasure Hunt — {1}{U} Sorcery. Reveal until a nonland card; take all of it.
pub fn treasure_hunt() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Treasure Hunt",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: R::Nonland,
            to: ZoneDest::Hand(PlayerRef::You),
            cap: Value::Const(60),
            life_per_revealed: 0,
            miss_dest: crate::effect::RevealMissDest::WithFind,
        },
        ..Default::default()
    }
}

/// Urge to Feed — {B}{B} Instant. −3/−3, and any Vampires you tap for it grow.
pub fn urge_to_feed() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Urge to Feed",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
            Effect::TapAnyNumberThenCounters {
                filter: R::HasCreatureType(CreatureType::Vampire),
                counter: CounterType::PlusOnePlusOne,
            },
        ]),
        ..Default::default()
    }
}

/// Terastodon — {6}{G}{G} 9/9 Elephant. ETB blows up three noncreature
/// permanents, paying each controller a 3/3 Elephant.
pub fn terastodon() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Destroy up to three target noncreature permanents".into(),
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 3,
                min_targets: 0,
                filter: R::Permanent.and(R::Noncreature),
                effect: Box::new(Effect::DestroyThenVictimControllersMakeToken {
                    what: Selector::Target(0),
                    definition: Box::new(TokenDefinition {
                        name: "Elephant".into(),
                        power: 3,
                        toughness: 3,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Green],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Elephant],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    no_regen: false,
                }),
            }),
        })],
        ..creature("Terastodon", cost(&[generic(6), g(), g()]), vec![CreatureType::Elephant], 9, 9)
    }
}

/// Surrakar Banisher — {4}{U} 3/3 Surrakar. ETB may bounce a tapped creature.
pub fn surrakar_banisher() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return target tapped creature to its owner's hand".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::Tapped)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            }),
        })],
        ..creature("Surrakar Banisher", cost(&[generic(4), u()]), vec![CreatureType::Surrakar], 3, 3)
    }
}

// ── Statics / enchantments ──────────────────────────────────────────────────

/// Horizon Drake — {1}{U}{U} 3/1 Drake with flying and protection from lands.
pub fn horizon_drake() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::ProtectionFromCardType(CardType::Land)],
        ..creature("Horizon Drake", cost(&[generic(1), u(), u()]), vec![CreatureType::Drake], 3, 1)
    }
}

/// Summit Apes — {3}{G} 5/2 Ape with menace while you control a Mountain.
pub fn summit_apes() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Has menace as long as you control a Mountain",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Menace,
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasLandType(LandType::Mountain).and(R::ControlledByYou),
                )),
            },
        }],
        ..creature("Summit Apes", cost(&[generic(3), g()]), vec![CreatureType::Ape], 5, 2)
    }
}

/// Terra Eternal — {2}{W} Enchantment. All lands have indestructible.
pub fn terra_eternal() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Terra Eternal",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "All lands have indestructible",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Land),
                keyword: Keyword::Indestructible,
            },
        }],
        ..Default::default()
    }
}

/// Quest for Renewal — {1}{G} Enchantment. Quest counters off your creatures
/// tapping; at four, untap them during each other player's untap step.
pub fn quest_for_renewal() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Quest for Renewal",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::MayDo {
                description: "Put a quest counter on Quest for Renewal".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Quest,
                    amount: Value::Const(1),
                }),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "With four quest counters, untap your creatures each other untap step",
            effect: StaticEffect::WhileCountersAtLeast {
                kind: CounterType::Quest,
                n: 4,
                inner: Box::new(StaticEffect::UntapYoursEachUntapStepFiltered(R::Creature)),
            },
        }],
        ..Default::default()
    }
}

/// Kazuul, Tyrant of the Cliffs — {3}{R}{R} 5/4 Ogre Warrior. Attacking him
/// costs {3} or gives him a 3/3 Ogre.
pub fn kazuul_tyrant_of_the_cliffs() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::Target(0),
                cost: WardCost::Mana(cost(&[generic(3)])),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: Box::new(TokenDefinition {
                        name: "Ogre".into(),
                        power: 3,
                        toughness: 3,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Red],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Ogre],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                }),
                if_paid: None,
            },
        }],
        ..creature("Kazuul, Tyrant of the Cliffs", cost(&[generic(3), r(), r()]), vec![CreatureType::Ogre, CreatureType::Warrior], 5, 4)
    }
}

/// Razor Boomerang — {3} Equipment granting "{T}, Unattach: 1 damage to any
/// target; return this to its owner's hand." Equip {2}.
pub fn razor_boomerang() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Razor Boomerang",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::DealDamage { to: target_any(), amount: Value::Const(1) },
                    Effect::Move {
                        what: Selector::AttachmentGranting,
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    },
                ]),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Vapor Snare — {4}{U} Aura. You control the enchanted creature; each upkeep,
/// bounce a land you control or lose the Aura.
pub fn vapor_snare() -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![
            TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GainControlWhileSourceRemains {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
        },
            TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Land.and(R::ControlledByYou),
                )),
                then: Box::new(Effect::MayDoElse {
                    description: "Return a land you control to its owner's hand".into(),
                    body: Box::new(Effect::MoveChosen {
                        from: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                        filter: None,
                        count: Value::Const(1),
                        up_to: false,
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                    else_: Box::new(Effect::SacrificeSource),
                }),
                else_: Box::new(Effect::SacrificeSource),
            },
        }],
        ..aura("Vapor Snare", cost(&[generic(4), u()]))
    }
}

/// Vastwood Zendikon — {4}{G} Aura. The land is a 6/4 Elemental; when it dies
/// the land card goes back to hand.
pub fn vastwood_zendikon() -> crate::card::CardDefinition {
    super::wwk::zendikon(
        "Vastwood Zendikon",
        cost(&[generic(4), g()]),
        (6, 4),
        vec![CreatureType::Elemental],
        vec![],
    )
}

/// Wind Zendikon — {U} Aura. The land is a 2/2 flying Elemental; when it dies
/// the land card goes back to hand.
pub fn wind_zendikon() -> crate::card::CardDefinition {
    super::wwk::zendikon(
        "Wind Zendikon",
        cost(&[u()]),
        (2, 2),
        vec![CreatureType::Elemental],
        vec![Keyword::Flying],
    )
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// Sejiri Steppe — enters tapped, taps for {W}; ETB gives a creature you
/// control protection from a color of your choice.
pub fn sejiri_steppe() -> crate::card::CardDefinition {
    super::wwk::tapped_etb_land(
        "Sejiri Steppe",
        Color::White,
        Effect::GrantProtectionFromChosenColor {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Smoldering Spires — enters tapped, taps for {R}; ETB stops a creature from
/// blocking this turn.
pub fn smoldering_spires() -> crate::card::CardDefinition {
    super::wwk::tapped_etb_land(
        "Smoldering Spires",
        Color::Red,
        Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        },
    )
}
