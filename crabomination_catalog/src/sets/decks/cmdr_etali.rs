//! Commander: the Etali, Primal Conqueror batch — Etali plus the cards
//! EDHREC lists as commonly played with it that the catalog lacked.
//! Tests in `tests/recent_b/cmdr_etali.rs`.
//!
//! Residuals (approximated or omitted clauses; each card's doc has the detail):
//! - Hunting Velociraptor: a Dinosaur with a printed alternative cost keeps
//!   that one instead of the granted prowl (a card carries one alt cost).
//! - _____ Goblin: the name stickers are the engine's stand-in sheets (not
//!   Unfinity's printed inserts), drawn lazily; the sticker's position in the
//!   name is not offered (it fills the blank).
//! - Passionate Archaeologist: gated on your controlling your own commander,
//!   its trigger is printed on the Background (it deals the damage, once,
//!   however many commanders you have).
//! - Mirage Phalanx: the two granted begin-combat triggers are one trigger on
//!   the Phalanx copying both halves of the pair; "loses soulbond" isn't
//!   modeled.
//! - Delina, Wild Mage: "you may roll again" always re-rolls, capped at five
//!   extra rolls. Flamerush Rider: the copy attacks the Rider's defender.
//! - Kindle the Inner Flame / Chandra, Flameshaper +1: the token's "at the
//!   beginning of the end step, sacrifice" is a next-end-step delayed
//!   sacrifice; Kindle's "behold three Elementals" is a count gate (nothing is
//!   revealed). Chandra's −4 splits among at most eight targets.
//! - Cursed Mirror: "as this enters" is an ETB trigger.
//! - Nalfeshnee: a copied permanent spell keeps its target; foretold / plotted
//!   casts aren't stamped "cast from exile" by the engine and don't trigger it.
//! - Eternal Scourge: "cast this card from exile" is a {3} sorcery-speed
//!   activation from exile (the Squee shape), not a cast.
//! - Evendo Brushrazer: the play-from-exile window opens on the triggering
//!   sacrifice (on your turn) rather than as a static.
//! - Tibalt's Trickery: "different name" dropped; the random 1–3 is a d3 roll;
//!   an uncounterable target skips the mill/cascade half.
//! - Delayed Blast Fireball: "cast from exile" is read as "not cast from hand".
//! - World War Hulk I: casts a red or green creature from hand for free as the
//!   chapter resolves, instead of discounting the next such spell this turn.
//! - Heartwood Crafter: its {C} isn't barred from hand-cast spells.
//! - Open the Omenpaths: the mana pays creature spells only (not
//!   enchantments), and the two colors may coincide.
//! - Tinder Wall: the damage targets any blocked creature.
//! - Arena of Glory: exert is a "doesn't untap next untap step" rider on the
//!   mana ability, not a cost.
//! - Game Trail: the reveal is an ETB check (the Snarl shape).
//! - Pia, Aether Ascetic / Formidable Speaker: the discarded card is the
//!   engine's pick. Kogla and Yidaro: the shuffle-back moves the card from any
//!   zone.
//! - Valakut Awakening: the chosen cards are shuffled in, not bottomed.
//! - Cream of the Crop: scry X (may keep more than one card on top).
//! - Minsc & Boo: the −2's target is chosen on activation (not a reflexive
//!   trigger) and the Hamster draw precedes the damage; no Minsc subtype.
//! - Invasion of Ikoria: always searches library and graveyard together;
//!   Zilortha's "you may" is always taken.
//! - Strionic Resonator: targets the trigger's source permanent (so not a
//!   trigger whose source has left), and the copy keeps its targets.
//!
//! New primitives: `Value::InstantsOrSorceriesCastThisTurn` (Rionya),
//! `Value::TotalManaValueOfOtherSpellsCastThisTurn` (Call Forth the Tempest),
//! `Selector::SoulbondPartner` (Mirage Phalanx), and flash grants read
//! through `active_static` so a gated one works (Radagast of Rhosgobel);
//! `Effect::PutCommanderOntoBattlefield` + `ZoneDest::Command` (Hellkite
//! Courser), `StaticEffect::GrantProwlToSpells` (Hunting Velociraptor),
//! `StaticEffect::LegendRuleDoesntApplyToYourPermanents` and
//! `StaticEffect::PumpPerSameNameCreatureYouControl` (Mirror Box).

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, BattleSubtype, CardDefinition, CardType,
    CounterType, CreatureType, EnchantmentSubtype, EquipBonus, Keyword, LandType, LoyaltyAbility,
    MayPlayDuration, PlaneswalkerSubtype, SelectionRequirement as R,
    StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, WardCost, Zone,
};
use crate::effect::shortcut::{
    add_colorless, add_mana, blitz, cascade, dash, each_opponent, each_opponent_creature, etb,
    mint_treasures, myriad, on_attack, target_any, target_filtered,
};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition,
    ManaPayload, PlayerRef, Predicate, Selector, StaticEffect, Value, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, SpendRestriction, cost, g, generic, hybrid, phyrexian, r, x};
use super::super::{dual_land_with, tap_add};

/// A legendary creature body.
fn a_legend(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// A nonlegendary creature body.
fn a_creature(
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

/// "another target creature you control".
fn a_another_creature_you_control() -> R {
    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)
}

/// `CreateTokenCopyOf` with every rider off except the ones passed in.
fn a_copy_of(
    source: Selector,
    count: Value,
    enters_tapped: bool,
    non_legendary: bool,
    extra_keywords: Vec<Keyword>,
) -> Effect {
    Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count,
        source,
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped,
        non_legendary,
        legendary: false,
        extra_keywords,
    }
}

/// "Create a token that's a copy of [source], except it has haste and 'At the
/// beginning of the end step, sacrifice this token.'" (Kindle the Inner Flame,
/// Chandra, Flameshaper's +1). The printed trigger fires at the first end step
/// the token sees, which is the next one — modeled as the next-end-step
/// sacrifice (`SacrificeLastCreatedTokensAtNextEndStep`, Urabrask's Forge).
fn a_copy_haste_sac_at_end_step(source: Selector) -> Effect {
    Effect::Seq(vec![
        a_copy_of(source, Value::ONE, false, false, vec![Keyword::Haste]),
        Effect::SacrificeLastCreatedTokensAtNextEndStep,
    ])
}

/// "Create a token that's a copy of [source] that's tapped and attacking …
/// exile it at end of combat" — Calamity's shape (tapped copy + join combat)
/// with Kiki-Jiki's bound-token delayed exile, keyed to end of combat.
/// `non_legendary` is Delina's "except it's not legendary".
fn a_temp_attacking_copy(source: Selector, non_legendary: bool) -> Effect {
    Effect::Seq(vec![
        a_copy_of(source, Value::ONE, true, non_legendary, vec![]),
        Effect::JoinCombatAttacking { what: Selector::LastCreatedTokens },
        Effect::DelayUntil {
            kind: DelayedTriggerKind::EndOfCombat,
            body: Box::new(Effect::Exile { what: Selector::LastCreatedToken }),
        },
    ])
}

/// Delina's d20: 1–14 makes one attacking copy; 15–20 makes one and rolls
/// again. `depth` bounds the re-roll chain (a finite stand-in for "you may
/// roll again" — each extra roll only ever adds a token, so rolling is
/// always taken).
fn a_delina_roll(depth: u8) -> Effect {
    let copy = || {
        a_temp_attacking_copy(target_filtered(R::Creature.and(R::ControlledByYou)), true)
    };
    let high = if depth == 0 {
        copy()
    } else {
        Effect::Seq(vec![copy(), a_delina_roll(depth - 1)])
    };
    Effect::RollDie {
        sides: 20,
        count: Value::ONE,
        modifier: Value::Const(0),
        reroll_at_most: 0,
        results: vec![(1, 14, copy()), (15, 20, high)],
        on_doubles: None,
    }
}

// ── Cards ───────────────────────────────────────────────────────────────────

/// Jaxis, the Troublemaker — {3}{R} 2/3 Legendary Human Warrior. {R}, {T},
/// Discard a card: create a token copy of another target creature you control;
/// it gains haste and "When this token dies, draw a card"; sacrifice it at the
/// beginning of the next end step. Sorcery speed. Blitz {1}{R}.
pub fn jaxis_the_troublemaker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[r()]),
            discard_cost: Some((R::Any, 1)),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                a_copy_of(
                    target_filtered(a_another_creature_you_control()),
                    Value::ONE,
                    false,
                    false,
                    vec![],
                ),
                Effect::GrantKeyword {
                    what: Selector::LastCreatedTokens,
                    keyword: Keyword::Haste,
                    duration: Duration::Permanent,
                },
                Effect::GrantTriggeredAbility {
                    what: Selector::LastCreatedTokens,
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                    }),
                    duration: Duration::Permanent,
                },
                Effect::SacrificeLastCreatedTokensAtNextEndStep,
            ]),
            ..Default::default()
        }],
        alternative_cost: Some(blitz(cost(&[generic(1), r()]))),
        ..a_legend(
            "Jaxis, the Troublemaker",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// Rionya, Fire Dancer — {3}{R}{R} 3/4 Legendary Human Wizard. At the beginning
/// of combat on your turn, create X token copies of another target creature you
/// control, X = 1 + instant and sorcery spells you've cast this turn. They gain
/// haste; exile them at the beginning of the next end step.
pub fn rionya_fire_dancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                a_copy_of(
                    target_filtered(a_another_creature_you_control()),
                    Value::Sum(vec![
                        Value::ONE,
                        Value::InstantsOrSorceriesCastThisTurn(PlayerRef::You),
                    ]),
                    false,
                    false,
                    vec![],
                ),
                Effect::GrantKeyword {
                    what: Selector::LastCreatedTokens,
                    keyword: Keyword::Haste,
                    duration: Duration::Permanent,
                },
                Effect::ExileLastCreatedTokensAtNextEndStep,
            ]),
        }],
        ..a_legend(
            "Rionya, Fire Dancer",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Orthion, Hero of Lavabrink — {3}{R} 3/3 Legendary Human Soldier.
/// {1}{R}, {T}: one hasty token copy of another target creature you control,
/// sacrificed at the next end step. {6}{R}{R}{R}, {T}: five of them. Both
/// sorcery speed. (Heat Shimmer / Devastating Onslaught's haste-sac primitive.)
pub fn orthion_hero_of_lavabrink() -> CardDefinition {
    let ability = |mana: crate::mana::ManaCost, n: i32| ActivatedAbility {
        tap_cost: true,
        mana_cost: mana,
        sorcery_speed: true,
        effect: Effect::CreateTokenCopiesHasteSac {
            who: PlayerRef::You,
            count: Value::Const(n),
            source: target_filtered(a_another_creature_you_control()),
        },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            ability(cost(&[generic(1), r()]), 1),
            ability(cost(&[generic(6), r(), r(), r()]), 5),
        ],
        ..a_legend(
            "Orthion, Hero of Lavabrink",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Delina, Wild Mage — {3}{R} 3/2 Legendary Elf Shaman. Whenever Delina
/// attacks, choose target creature you control, then roll a d20. 1–14: create a
/// tapped and attacking non-legendary token copy of it that's exiled at end of
/// combat. 15–20: create one of those tokens; you may roll again.
/// Approximation: "you may roll again" always re-rolls (each roll only adds a
/// token), bounded at five extra rolls. The token's own "At end of combat,
/// exile this token" is a delayed end-of-combat exile bound to it — a copy of
/// the token would not inherit it. Under a token doubler only the last minted
/// token of each roll is bound to the exile.
pub fn delina_wild_mage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(a_delina_roll(5))],
        ..a_legend(
            "Delina, Wild Mage",
            cost(&[generic(3), r()]),
            vec![CreatureType::Elf, CreatureType::Shaman],
            3,
            2,
        )
    }
}

/// Flamerush Rider — {4}{R} 3/3 Human Warrior. Whenever it attacks, create a
/// token copy of another target attacking creature, tapped and attacking;
/// exile the token at end of combat. Dash {2}{R}{R}.
/// Approximation: the copy attacks the defender Flamerush Rider is attacking
/// (the controller's choice of defender is collapsed). Under a token doubler
/// only the last minted token is bound to the end-of-combat exile.
pub fn flamerush_rider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(a_temp_attacking_copy(
            target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
            false,
        ))],
        alternative_cost: Some(dash(cost(&[generic(2), r(), r()]))),
        ..a_creature(
            "Flamerush Rider",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Mirage Phalanx — {4}{R}{R} 4/4 Human Soldier. Soulbond. While paired, each
/// of the pair has "At the beginning of combat on your turn, create a token
/// that's a copy of this creature, except it has haste and loses soulbond.
/// Exile it at end of combat."
///
/// The two granted triggers are folded into one printed on Mirage Phalanx:
/// while it is paired (`Selector::SoulbondPartner` is non-empty), your
/// beginning of combat makes a hasty copy of it and one of its partner, each
/// exiled at end of combat.
/// Approximation: one trigger instead of two (a response between the two
/// copies isn't possible), and "loses soulbond" is not modeled — a token copy
/// of Mirage Phalanx keeps Soulbond, so it may pair as it enters.
pub fn mirage_phalanx() -> CardDefinition {
    let temp_copy = |source: Selector| {
        [
            a_copy_of(source, Value::ONE, false, false, vec![Keyword::Haste]),
            Effect::DelayUntil {
                kind: DelayedTriggerKind::EndOfCombat,
                body: Box::new(Effect::Exile { what: Selector::LastCreatedToken }),
            },
        ]
    };
    CardDefinition {
        keywords: vec![Keyword::Soulbond],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            )
            .with_filter(Predicate::ValueAtLeast(
                Value::CountOf(Box::new(Selector::SoulbondPartner)),
                Value::ONE,
            )),
            effect: Effect::Seq(
                temp_copy(Selector::This)
                    .into_iter()
                    .chain(temp_copy(Selector::SoulbondPartner))
                    .collect(),
            ),
        }],
        ..a_creature(
            "Mirage Phalanx",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Flameshadow Conjuring — {3}{R} Enchantment. Whenever a nontoken creature you
/// control enters, you may pay {R}. If you do, create a token copy of it; the
/// token gains haste and is exiled at the beginning of the next end step.
pub fn flameshadow_conjuring() -> CardDefinition {
    CardDefinition {
        name: "Flameshadow Conjuring",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken),
                }),
            effect: Effect::MayPay {
                description: "Pay {R} to copy the creature?".into(),
                mana_cost: cost(&[r()]),
                body: Box::new(Effect::Seq(vec![
                    a_copy_of(Selector::TriggerSource, Value::ONE, false, false, vec![]),
                    Effect::GrantKeyword {
                        what: Selector::LastCreatedTokens,
                        keyword: Keyword::Haste,
                        duration: Duration::Permanent,
                    },
                    Effect::ExileLastCreatedTokensAtNextEndStep,
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Molten Echoes — {2}{R}{R} Enchantment. As it enters, choose a creature type.
/// Whenever a nontoken creature of the chosen type you control enters, create a
/// token copy of it; the token gains haste and is exiled at the beginning of
/// the next end step.
pub fn molten_echoes() -> CardDefinition {
    CardDefinition {
        name: "Molten Echoes",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::TriggerObjectIsChosenType,
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::NotToken),
                    },
                ])),
            effect: Effect::Seq(vec![
                a_copy_of(Selector::TriggerSource, Value::ONE, false, false, vec![]),
                Effect::GrantKeyword {
                    what: Selector::LastCreatedTokens,
                    keyword: Keyword::Haste,
                    duration: Duration::Permanent,
                },
                Effect::ExileLastCreatedTokensAtNextEndStep,
            ]),
        }],
        ..Default::default()
    }
}

/// Kindle the Inner Flame — {3}{R} Kindred Sorcery — Elemental. Create a token
/// copy of target creature you control, except it has haste and "At the
/// beginning of the end step, sacrifice this token." Flashback—{1}{R}, Behold
/// three Elementals.
/// Approximation: the token's end-step sacrifice is the next-end-step delayed
/// sacrifice (same first firing; a copy of the token wouldn't inherit it).
/// "Behold three Elementals" is a flashback *gate* — you control and/or hold
/// at least three Elemental permanents/cards in hand — nothing is revealed or
/// chosen, and one card can't be both beheld and counted twice (the counts
/// are disjoint zones, so it can't anyway).
pub fn kindle_the_inner_flame() -> CardDefinition {
    CardDefinition {
        name: "Kindle the Inner Flame",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        keywords: vec![Keyword::Flashback(cost(&[generic(1), r()]))],
        flashback_condition: Some(Predicate::ValueAtLeast(
            Value::Sum(vec![
                Value::CountOf(Box::new(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Elemental).and(R::ControlledByYou),
                ))),
                Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Hand,
                    filter: R::HasCreatureType(CreatureType::Elemental),
                })),
            ]),
            Value::Const(3),
        )),
        effect: a_copy_haste_sac_at_end_step(target_filtered(
            R::Creature.and(R::ControlledByYou),
        )),
        ..Default::default()
    }
}

/// Chandra, Flameshaper — {5}{R}{R} Legendary Planeswalker — Chandra, loyalty 6.
/// +2: Add {R}{R}{R}. Exile the top three cards of your library; choose one;
/// you may play it this turn. +1: token copy of target creature you control,
/// except it has haste and "At the beginning of the end step, sacrifice this
/// token." −4: 8 damage divided among any number of target creatures and/or
/// planeswalkers.
/// Approximation: +1's end-step sacrifice is the next-end-step delayed
/// sacrifice. −4's "any number" is capped at eight target slots (8 damage can't
/// usefully split wider).
pub fn chandra_flameshaper() -> CardDefinition {
    CardDefinition {
        name: "Chandra, Flameshaper",
        cost: cost(&[generic(5), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Chandra],
            ..Default::default()
        },
        base_loyalty: 6,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Colors(vec![Color::Red, Color::Red, Color::Red]),
                    },
                    Effect::ExileTopOfLibrary {
                        who: Selector::You,
                        amount: Value::Const(3),
                        link_to_source: false,
                        face_down: false,
                    },
                    Effect::ChooseOneAmong {
                        what: Selector::LastMoved,
                        chooser: PlayerRef::You,
                        chosen: Box::new(Effect::GrantMayPlay {
                            what: Selector::SeparatedPile { chosen: true },
                            duration: crate::card::MayPlayDuration::EndOfThisTurn,
                            to_owner: false,
                            exile_after: false,
                            pay_own_cost: true,
                            any_color: false,
                        }),
                        other: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: a_copy_haste_sac_at_end_step(target_filtered(
                    R::Creature.and(R::ControlledByYou),
                )),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -4,
                effect: Effect::DealDamageDivided {
                    total: Value::Const(8),
                    filter: R::Creature.or(R::Planeswalker),
                    max_targets: 8,
                    retaliate_to_source: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Blade of Selves — {2} Artifact — Equipment. Equipped creature has myriad.
/// Equip {4}. (The equip-granted trigger fires off the host, so
/// `Effect::Myriad` copies the equipped creature.)
pub fn blade_of_selves() -> CardDefinition {
    CardDefinition {
        name: "Blade of Selves",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![myriad()],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Mirror Box — {3} Artifact. The legend rule doesn't apply to permanents you
/// control. Each legendary creature you control gets +1/+1. Each nontoken
/// creature you control gets +1/+1 for each other creature you control with the
/// same name.
pub fn mirror_box() -> CardDefinition {
    CardDefinition {
        name: "Mirror Box",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "The \"legend rule\" doesn't apply to permanents you control.",
                effect: StaticEffect::LegendRuleDoesntApplyToYourPermanents,
            },
            StaticAbility {
                description: "Each legendary creature you control gets +1/+1.",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                    opponents: false,
                    all_players: false,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
            StaticAbility {
                description: "Each nontoken creature you control gets +1/+1 for each other \
                              creature you control with the same name as that creature.",
                effect: StaticEffect::PumpPerSameNameCreatureYouControl { power: 1, toughness: 1 },
            },
        ],
        ..Default::default()
    }
}

/// Cursed Mirror — {2}{R} Artifact. {T}: Add {R}. As it enters, you may have
/// it become a copy of any creature on the battlefield until end of turn,
/// except it has haste.
/// Approximation: "as this enters" is an ETB trigger (it can be responded to,
/// and the copy lands after other ETB triggers have been put on the stack).
/// The creature is chosen, not targeted (`ChooseOneAmong`), so hexproof /
/// shroud creatures are legal picks, as printed.
pub fn cursed_mirror() -> CardDefinition {
    CardDefinition {
        name: "Cursed Mirror",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![crate::sets::tap_add(Color::Red)],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Have Cursed Mirror become a copy of a creature until end of turn?"
                .into(),
            body: Box::new(Effect::ChooseOneAmong {
                what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                chooser: PlayerRef::You,
                chosen: Box::new(Effect::Seq(vec![
                    Effect::BecomeCopyOfFor {
                        what: Selector::This,
                        source: Selector::SeparatedPile { chosen: true },
                        duration: Duration::EndOfTurn,
                        non_legendary: false,
                    },
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ])),
                other: Box::new(Effect::Noop),
            }),
        })],
        ..Default::default()
    }
}

/// Hellkite Courser — {4}{R}{R} 6/5 Dragon. Flying. When it enters, you may put
/// a commander you own from the command zone onto the battlefield; it gains
/// haste; return it to the command zone at the beginning of the next end step.
/// `Effect::PutCommanderOntoBattlefield`: not a cast (no tax); with two
/// commanders in the command zone you choose one. The haste is an
/// until-end-of-turn grant (the commander goes home in the end step anyway).
/// CR 400.7 — the return reads the object that entered here
/// (`Predicate::SourceIsSameObjectOnBattlefield`): a commander that left and
/// was recast before the end step is a new object and stays; one that only
/// transformed is the same object and goes home.
pub fn hellkite_courser() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Put a commander you own from the command zone onto the battlefield?"
                .into(),
            body: Box::new(Effect::PutCommanderOntoBattlefield {
                who: PlayerRef::You,
                haste: true,
                return_at_end_step: true,
            }),
        })],
        ..a_creature(
            "Hellkite Courser",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Dragon],
            6,
            5,
        )
    }
}

/// Sanctum of Eternity — Land. {T}: Add {C}. {2}, {T}: Return target commander
/// you own from the battlefield to your hand. Activate only during your turn.
/// The CR 903.9b hand→command-zone replacement still offers its owner the
/// command zone instead, as for any commander.
pub fn sanctum_of_eternity() -> CardDefinition {
    CardDefinition {
        name: "Sanctum of Eternity",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::Move {
                    what: target_filtered(R::IsCommander.and(R::OwnedByYou)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Passionate Archaeologist — {1}{R} Legendary Enchantment — Background.
/// Commander creatures you own have "Whenever you cast a spell from exile, this
/// creature deals damage equal to that spell's mana value to target opponent."
/// Approximation: the trigger is printed on the Background and gated on your
/// controlling your own commander (`Predicate::ControlsOwnCommander`) — the
/// damage's *source* is the Background rather than the commander (matters only
/// for lifelink/deathtouch/infect-style riders on the commander), and two
/// commanders on the battlefield trigger once rather than twice.
pub fn passionate_archaeologist() -> CardDefinition {
    CardDefinition {
        name: "Passionate Archaeologist",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Background],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::CastSpellFromExile,
                    Predicate::ControlsOwnCommander { who: PlayerRef::You },
                ]),
            ),
            effect: Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer),
                amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}

/// Etali, Primal Conqueror // Etali, Primal Sickness — {5}{R}{R} Legendary
/// Creature — Elder Dinosaur 7/7. Trample. When Etali enters, each player
/// exiles cards from the top of their library until they exile a nonland
/// card; you may cast any number of spells from among the nonland cards
/// exiled this way without paying their mana costs. {9}{G/P}: Transform
/// Etali. Activate only as a sorcery. Back: Legendary Creature — Phyrexian
/// Elder Dinosaur 11/11, trample, indestructible; whenever it deals combat
/// damage to a player, they get that many poison counters.
///
/// The ETB is `EachPlayerDoes(EachPlayer, ExileTopUntilNonland)` then
/// `CastAnyOrderWithoutPaying` over the nonland cards exiled this resolution.
/// The back face's colour indicator (red and green) is the printed card's.
pub fn etali_primal_conqueror() -> CardDefinition {
    let back = CardDefinition {
        name: "Etali, Primal Sickness",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Red, Color::Green],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Phyrexian,
                CreatureType::Elder,
                CreatureType::Dinosaur,
            ],
            ..Default::default()
        },
        power: 11,
        toughness: 11,
        keywords: vec![Keyword::Trample, Keyword::Indestructible],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddPoison {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Etali, Primal Conqueror",
        cost: cost(&[generic(5), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::EachPlayerDoes {
                who: PlayerRef::EachPlayer,
                body: Box::new(Effect::ExileTopUntilNonland { who: PlayerRef::You }),
            },
            Effect::CastAnyOrderWithoutPaying {
                what: Selector::ExiledThisResolution { filter: R::Nonland },
                source_zone: Zone::Exile,
                filter: None,
                cap: None,
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(9), phyrexian(Color::Green)]),
            sorcery_speed: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(back)),
        ..Default::default()
    }
}

/// Nalfeshnee — {5}{R} Creature — Beast Demon 4/6. Flying. Whenever you cast a
/// spell from exile, copy it. You may choose new targets for the copy. If it's
/// a permanent spell, the copy gains haste and "At the beginning of the end
/// step, sacrifice this permanent."
///
/// Permanent spells take `CopySpellWithRiders` (haste + next-end-step
/// sacrifice on the resulting token), everything else takes
/// `CopySpellMayChooseTargets`.
/// Approximation: a permanent spell's copy keeps the original's target (the
/// riders copy path doesn't offer a retarget). That only matters for Auras.
/// Approximation: the engine sets `cast_from_exile` only on the generic
/// may-play/free cast-from-zone path, so a foretold (`CastForetold`) or plotted
/// spell doesn't trigger it.
pub fn nalfeshnee() -> CardDefinition {
    CardDefinition {
        name: "Nalfeshnee",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Demon],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellFromExile),
            effect: Effect::If {
                cond: Predicate::CastSpellMatches(R::Permanent),
                then: Box::new(Effect::CopySpellWithRiders {
                    what: Selector::TriggerSource,
                    count: Value::ONE,
                    grant_haste: true,
                    sacrifice_eot: true,
                }),
                else_: Box::new(Effect::CopySpellMayChooseTargets {
                    what: Selector::TriggerSource,
                    count: Value::ONE,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Keeper of Secrets — {5}{R} Creature — Demon 6/4. First strike, haste.
/// Symphony of Pain — whenever you cast a spell from anywhere other than your
/// hand, this deals damage equal to that spell's mana value to target
/// opponent.
///
/// The filter is Graham O'Brien's / Unstable Amulet's
/// `EntityMatches(TriggerSource, SpellNotCastFromHand)`. The damage reads the
/// spell's MV off the stack (`ManaValueOf(TriggerSource)`).
pub fn keeper_of_secrets() -> CardDefinition {
    CardDefinition {
        name: "Keeper of Secrets",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 6,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::SpellNotCastFromHand,
                },
            ),
            effect: Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer),
                amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}

/// Eternal Scourge — {3} Creature — Eldrazi Horror 3/3. You may cast this card
/// from exile. When it becomes the target of a spell or ability an opponent
/// controls, exile it.
///
/// The exile trigger is `BecameTarget` + `OpponentControl`. The dispatcher
/// already requires the targeted permanent to be this creature, and for
/// `BecameTarget` the scope reads the caster (the Tenured Concocter shape).
/// Approximation: the "cast this card from exile" permission uses the Squee,
/// the Immortal shape: a `from_exile` sorcery-speed {3} activation that
/// `Move`s the card onto the battlefield. It isn't a *cast*, so it can't be
/// countered and doesn't fire cast triggers (including Nalfeshnee / Keeper of
/// Secrets).
pub fn eternal_scourge() -> CardDefinition {
    CardDefinition {
        name: "Eternal Scourge",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::OpponentControl),
            effect: Effect::Exile { what: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sorcery_speed: true,
            from_exile: true,
            effect: Effect::Move {
                what: Selector::This,
                to: crate::effect::ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Professional Face-Breaker — {2}{R} Creature — Human Warrior 2/3. Menace.
/// Whenever one or more creatures you control deal combat damage to a player,
/// create a Treasure token. Sacrifice a Treasure: Exile the top card of your
/// library. You may play that card this turn.
///
/// The batched trigger is Frostcliff Siege's
/// `DealsCombatDamageToPlayer`/`YourControl` + `once_per_batch()`. The impulse
/// is `ExileTopAndGrantMayPlay { pay_own_cost: true }` (the card isn't free).
pub fn professional_face_breaker() -> CardDefinition {
    CardDefinition {
        name: "Professional Face-Breaker",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .once_per_batch(),
            effect: mint_treasures(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Treasure), 1)),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfThisTurn,
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

/// Evendo Brushrazer — {2}{R} Creature — Insect Warrior 2/2. Whenever you
/// sacrifice a nontoken permanent, exile the top card of your library. During
/// your turn, as long as you've sacrificed a nontoken permanent this turn, you
/// may play cards exiled with this creature. {T}, Sacrifice a land: Add {R}{R}.
///
/// The trigger exiles the top card linked to this creature
/// (`ExileTopOfLibrary { link_to_source }`). On your turn, the same trigger
/// then grants a play-this-turn permission, paying the card's own cost, to
/// *every* card exiled with it (`GrantMayPlay(CardExiledWithSource)`). The
/// trigger is itself the "you've sacrificed a nontoken permanent this turn"
/// event, so from the first such sacrifice on your turn every linked card is
/// playable until end of turn. That covers cards exiled on earlier turns and
/// on opponents' turns.
/// Approximation: the static is event-driven. A nontoken sacrifice made on
/// your turn while this creature wasn't on the battlefield doesn't open the
/// window, and the window stays open for the rest of the turn even if this
/// creature leaves.
pub fn evendo_brushrazer() -> CardDefinition {
    CardDefinition {
        name: "Evendo Brushrazer",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken,
                }),
            effect: Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: Selector::You,
                    amount: Value::ONE,
                    link_to_source: true,
                    face_down: false,
                },
                Effect::If {
                    cond: Predicate::IsTurnOf(PlayerRef::You),
                    then: Box::new(Effect::GrantMayPlay {
                        what: Selector::CardExiledWithSource,
                        duration: MayPlayDuration::EndOfThisTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Red, Color::Red]),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tibalt's Trickery — {1}{R} Instant. Counter target spell. Choose 1, 2, or 3
/// at random. Its controller mills that many cards, then exiles cards from the
/// top of their library until they exile a nonland card with a different name
/// than that spell. They may cast that card without paying its mana cost. Then
/// they put the exiled cards on the bottom of their library in a random order.
///
/// After the counter, `EachPlayerDoes(CounteredSpellController)` runs the rest
/// as that player (Fold into Aether's player ref). The random 1-3 is a d3
/// `RollDie` feeding `Mill`. The exile-until / may-cast-free / bottom-the-rest
/// sequence is exactly `Cascade` with no mana-value cap.
/// Approximation: "with a different name than that spell" is dropped, so a
/// nonland card sharing the countered spell's name stops the exile too.
/// Approximation: the random choice is a die roll. It asks the decider
/// (AutoDecider always takes the midpoint, 2) and fires "whenever you roll a
/// die" payoffs.
/// Approximation: if the target spell can't be countered,
/// `CounteredSpellController` stays unset and the mill/cascade half is
/// skipped. The printed card still has that player mill and cascade.
pub fn tibalts_trickery() -> CardDefinition {
    let mill = |n: i32| Effect::Mill { who: Selector::You, amount: Value::Const(n) };
    CardDefinition {
        name: "Tibalt's Trickery",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::EachPlayerDoes {
                who: PlayerRef::CounteredSpellController,
                body: Box::new(Effect::Seq(vec![
                    Effect::RollDie {
                        sides: 3,
                        count: Value::ONE,
                        modifier: Value::Const(0),
                        reroll_at_most: 0,
                        results: vec![(1, 1, mill(1)), (2, 2, mill(2)), (3, 3, mill(3))],
                        on_doubles: None,
                    },
                    Effect::Cascade { max_mv: Value::Const(1000) },
                ])),
            },
        ]),
        ..Default::default()
    }
}

/// Delayed Blast Fireball — {1}{R}{R} Instant. Deals 2 damage to each opponent
/// and each creature they control. If this spell was cast from exile, it deals
/// 5 damage to each opponent and each creature they control instead. Foretell
/// {4}{R}{R}.
///
/// Approximation: "cast from exile" reads "not cast from hand"
/// (`Not(Predicate::CastFromHand)`, as Antiquities on the Loose). The printed exile
/// casts (foretold, impulse, free cast from exile) all qualify. A cast from a
/// graveyard or library, or a copy, also gets the 5-damage branch.
pub fn delayed_blast_fireball() -> CardDefinition {
    let blast = |n: i32| {
        Effect::Seq(vec![
            Effect::DealDamage { to: each_opponent(), amount: Value::Const(n) },
            Effect::DealDamage { to: each_opponent_creature(), amount: Value::Const(n) },
        ])
    };
    CardDefinition {
        name: "Delayed Blast Fireball",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        foretell_cost: Some(cost(&[generic(4), r(), r()])),
        effect: Effect::If {
            cond: Predicate::Not(Box::new(Predicate::CastFromHand)),
            then: Box::new(blast(5)),
            else_: Box::new(blast(2)),
        },
        ..Default::default()
    }
}

/// Call Forth the Tempest — {5}{R}{R}{R} Sorcery. Cascade, cascade. Deals
/// damage to each creature your opponents control equal to the total mana
/// value of other spells you've cast this turn.
///
/// The double cascade is Maelstrom Wanderer's `cascade(8), cascade(8)`; the
/// damage reads `Value::TotalManaValueOfOtherSpellsCastThisTurn` (the cascade
/// hits are cast before the spell resolves, so they count).
pub fn call_forth_the_tempest() -> CardDefinition {
    CardDefinition {
        name: "Call Forth the Tempest",
        cost: cost(&[generic(5), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        triggered_abilities: vec![cascade(8), cascade(8)],
        effect: Effect::DealDamage {
            to: each_opponent_creature(),
            amount: Value::TotalManaValueOfOtherSpellsCastThisTurn(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Escape to the Wilds — {3}{R}{G} Sorcery. Exile the top five cards of your
/// library. You may play cards exiled this way until the end of your next
/// turn. You may play an additional land this turn.
pub fn escape_to_the_wilds() -> CardDefinition {
    CardDefinition {
        name: "Escape to the Wilds",
        cost: cost(&[generic(3), r(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(5),
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            Effect::GrantExtraLandPlay { who: PlayerRef::You, count: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Rishkar's Expertise — {4}{G}{G} Sorcery. Draw cards equal to the greatest
/// power among creatures you control. You may cast a spell with mana value 5
/// or less from your hand without paying its mana cost.
pub fn rishkars_expertise() -> CardDefinition {
    CardDefinition {
        name: "Rishkar's Expertise",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::GreatestPowerControlled { who: PlayerRef::You },
            },
            Effect::CastFromHandWithoutPaying { filter: Some(R::ManaValueAtMost(5)) },
        ]),
        ..Default::default()
    }
}

/// World War Hulk — {3}{G}{G} Enchantment — Saga. I — The next red or green
/// creature spell you cast this turn can be cast without paying its mana cost.
/// II — Put three +1/+1 counters on target creature you control. III — Choose
/// target creature you control. Until end of turn, double its power and
/// toughness and it gains trample.
///
/// III doubles by pumping +P/+T equal to the target's current P/T (the Tifa's
/// Limit Break shape).
/// Approximation: chapter I casts a red or green creature spell from your hand
/// for free as the chapter resolves (`CastFromHandWithoutPaying`). The printed
/// discount waits for the next such spell this turn, from any zone (the
/// command zone included).
pub fn world_war_hulk() -> CardDefinition {
    let yours = || R::Creature.and(R::ControlledByYou);
    CardDefinition {
        name: "World War Hulk",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::CastFromHandWithoutPaying {
                    filter: Some(
                        R::Creature.and(R::HasColor(Color::Red).or(R::HasColor(Color::Green))),
                    ),
                },
            ),
            (
                2,
                Effect::AddCounter {
                    what: target_filtered(yours()),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(3),
                },
            ),
            (
                3,
                Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(yours()),
                        power: Value::PowerOf(Box::new(Selector::Target(0))),
                        toughness: Value::ToughnessOf(Box::new(Selector::Target(0))),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// A plain creature frame.
fn c_creature(
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

/// A bare land frame (group C private helper).
fn c_land(name: &'static str) -> CardDefinition {
    CardDefinition { name, card_types: vec![CardType::Land], ..Default::default() }
}

/// `Sacrifice this creature: Add {R}{R}.` — Reckless Barbarian, Tinder Wall.
/// A mana ability (CR 605.1a: adds mana, no target).
fn c_sac_for_rr() -> ActivatedAbility {
    ActivatedAbility {
        sac_cost: true,
        effect: add_mana(vec![Color::Red, Color::Red]),
        ..Default::default()
    }
}

/// "The first creature spell you cast each turn costs {2} less to cast." —
/// Radagast of Rhosgobel, Shadow in the Warp (Conduit of Ruin's primitive).
fn c_first_creature_costs_two_less() -> StaticAbility {
    StaticAbility {
        description: "The first creature spell you cast each turn costs {2} less to cast.",
        effect: StaticEffect::CostReductionFirstCreatureSpell { amount: 2 },
    }
}

// ── Heartwood Crafter // Soul Tether ────────────────────────────────────────

/// Heartwood Crafter // Soul Tether — {G} Creature — Elf Artificer 1/1 //
/// {2}{R/G} Sorcery (SOS preparation card). This creature enters prepared.
/// `{T}: Add {C}. This mana can't be spent to cast spells from your hand.`
/// Prepare spell (cast a copy via `GameAction::CastPrepareSpell`): create a
/// Heartwood token — a red and green artifact with "{T}: Add {R} or {G}."
///
/// Modeled on Studious First-Year (`prepare_spell` + `enters_with_counters:
/// Prepared`).
///
/// Approximation: the Crafter's {C} is unrestricted. The engine's
/// `SpellKind` does not record the zone a spell is cast from, so there is no
/// `SpendRestriction` for "can't be spent to cast spells from your hand";
/// the mana can therefore also pay for hand casts.
pub fn heartwood_crafter() -> CardDefinition {
    let heartwood = TokenDefinition {
        name: "Heartwood".into(),
        card_types: vec![CardType::Artifact],
        colors: vec![Color::Red, Color::Green],
        activated_abilities: vec![tap_add(Color::Red), tap_add(Color::Green)],
        ..Default::default()
    };
    let soul_tether = CardDefinition {
        name: "Soul Tether",
        cost: cost(&[generic(2), hybrid(Color::Red, Color::Green)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: std::sync::Arc::new(heartwood),
        },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: add_colorless(1),
            ..Default::default()
        }],
        prepare_spell: Some(std::sync::Arc::new(soul_tether)),
        enters_with_counters: Some((CounterType::Prepared, Value::Const(1))),
        ..c_creature(
            "Heartwood Crafter",
            cost(&[g()]),
            vec![CreatureType::Elf, CreatureType::Artificer],
            1,
            1,
        )
    }
}

// ── Rituals / mana creatures ────────────────────────────────────────────────

/// Dragon's Desire — {2}{R}{R} Sorcery. Add {R} for each artifact your
/// opponents control. (Counts across every opponent in a pod.)
pub fn dragons_desire() -> CardDefinition {
    CardDefinition {
        name: "Dragon's Desire",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColor(
                Color::Red,
                Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Artifact.and(R::ControlledByOpponent),
                ))),
            ),
        },
        ..Default::default()
    }
}

/// Somberwald Sage — {2}{G} Creature — Human Druid 0/1. `{T}: Add three mana
/// of any one color. Spend this mana only to cast creature spells.`
pub fn somberwald_sage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyOneColor(Value::Const(3))),
                    SpendRestriction::CreatureOnly,
                ),
            },
            ..Default::default()
        }],
        ..c_creature(
            "Somberwald Sage",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            0,
            1,
        )
    }
}

/// Gwenna, Eyes of Gaea — {2}{G} Legendary Creature — Elf Druid Scout 2/3.
/// `{T}: Add two mana in any combination of colors. Spend this mana only to
/// cast creature spells or activate abilities of creature sources.` Whenever
/// you cast a creature spell with power 5 or greater, put a +1/+1 counter on
/// Gwenna and untap it.
pub fn gwenna_eyes_of_gaea() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyColors(Value::Const(2))),
                    SpendRestriction::CreatureSpellsOrAbilities,
                ),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtLeast(5)),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::Untap { what: Selector::This, up_to: None },
            ]),
        }],
        ..c_creature(
            "Gwenna, Eyes of Gaea",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Druid, CreatureType::Scout],
            2,
            3,
        )
    }
}

/// Open the Omenpaths — {2}{R} Instant. Choose one — • Add two mana of any
/// one color and two mana of any other color. Spend this mana only to cast
/// creature or enchantment spells. • Creatures you control get +1/+0 until
/// end of turn.
///
/// Approximation: the mana is restricted to creature spells only
/// (`SpendRestriction::CreatureOnly` — there is no creature-or-enchantment
/// restriction), so it can't pay for an enchantment spell; and the two
/// pairs are two independent "two of any one color" picks, so the second
/// color isn't forced to differ from the first.
pub fn open_the_omenpaths() -> CardDefinition {
    let pair = || Effect::AddMana {
        who: PlayerRef::You,
        pool: ManaPayload::Restricted(
            Box::new(ManaPayload::AnyOneColor(Value::Const(2))),
            SpendRestriction::CreatureOnly,
        ),
    };
    CardDefinition {
        name: "Open the Omenpaths",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![pair(), pair()]),
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Infernal Plunge — {R} Sorcery. As an additional cost to cast this spell,
/// sacrifice a creature. Add {R}{R}{R}. (Culling the Weak's shape.)
pub fn infernal_plunge() -> CardDefinition {
    CardDefinition {
        name: "Infernal Plunge",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        effect: add_mana(vec![Color::Red, Color::Red, Color::Red]),
        ..Default::default()
    }
}

/// Reckless Barbarian — {1}{R} Creature — Dragon Barbarian 2/2. Sacrifice
/// this creature: Add {R}{R}.
pub fn reckless_barbarian() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![c_sac_for_rr()],
        ..c_creature(
            "Reckless Barbarian",
            cost(&[generic(1), r()]),
            vec![CreatureType::Dragon, CreatureType::Barbarian],
            2,
            2,
        )
    }
}

/// Tinder Wall — {G} Creature — Plant Wall 0/3. Defender. Sacrifice this
/// creature: Add {R}{R}. `{R}, Sacrifice this creature: It deals 2 damage to
/// target creature it's blocking.`
///
/// Approximation: the damage ability targets any *blocked* creature
/// (`R::IsBlocked`), not specifically one Tinder Wall is blocking. The
/// source is sacrificed as a cost, and `BlockingOrBlockedBySource` reads the
/// live `block_map` (the Wall's entry is gone once it's sacrificed) while
/// `BlockedBySourceThisTurn` is gated on the source still being on the
/// battlefield, so the exact "it's blocking" filter would fizzle on resolution.
pub fn tinder_wall() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![
            c_sac_for_rr(),
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                sac_cost: true,
                effect: Effect::DealDamage {
                    to: crate::effect::shortcut::target_filtered(R::Creature.and(R::IsBlocked)),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..c_creature(
            "Tinder Wall",
            cost(&[g()]),
            vec![CreatureType::Plant, CreatureType::Wall],
            0,
            3,
        )
    }
}

// ── Cost reducers ───────────────────────────────────────────────────────────

/// Radagast of Rhosgobel — {2}{G}{G} Legendary Creature — Avatar Wizard 2/5.
/// The first creature spell you cast each turn costs {2} less to cast and
/// can be cast as though it had flash.
///
/// The discount is `CostReductionFirstCreatureSpell` (Conduit of Ruin).
/// The flash half is `WhileCondition { no creature spell cast yet this turn,
/// ControllerSpellsHaveFlash(Creature) }`; `battlefield_grants_flash` peels
/// the gate through `active_static`.
pub fn radagast_of_rhosgobel() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            c_first_creature_costs_two_less(),
            StaticAbility {
                description: "The first creature spell you cast each turn can be cast as \
                              though it had flash.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::Not(Box::new(Predicate::CreaturesCastThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::ONE,
                    })),
                    inner: Box::new(StaticEffect::ControllerSpellsHaveFlash {
                        filter: R::Creature,
                    }),
                },
            },
        ],
        ..c_creature(
            "Radagast of Rhosgobel",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            2,
            5,
        )
    }
}

/// Shadow in the Warp — {1}{R}{G} Enchantment. The first creature spell you
/// cast each turn costs {2} less to cast. Whenever an opponent casts their
/// first noncreature spell each turn, this enchantment deals 2 damage to that
/// player.
///
/// The trigger is Esper Sentinel's shape: per *opponent* (the caster's own
/// noncreature-spell count equals one), not once per turn, so it fires for
/// each opponent in a pod.
pub fn shadow_in_the_warp() -> CardDefinition {
    let caster = || PlayerRef::ControllerOf(Box::new(Selector::TriggerSource));
    CardDefinition {
        name: "Shadow in the Warp",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![c_first_creature_costs_two_less()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Noncreature,
                    },
                    Predicate::ValueEquals(
                        Value::NoncreatureSpellsCastThisTurn(caster()),
                        Value::ONE,
                    ),
                ]),
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(caster()),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

// ── Hexing Squelcher ────────────────────────────────────────────────────────

/// Hexing Squelcher — {1}{R} Creature — Goblin Sorcerer 2/2. This spell
/// can't be countered. Ward—Pay 2 life. Spells you control can't be
/// countered. Other creatures you control have "Ward—Pay 2 life."
/// (Winter, Cursed Rider's granted-ward shape; Last Word's keyword.)
pub fn hexing_squelcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered, Keyword::Ward(WardCost::Life(2))],
        static_abilities: vec![
            StaticAbility {
                description: "Spells you control can't be countered.",
                effect: StaticEffect::SpellsUncounterable { filter: R::Any },
            },
            StaticAbility {
                description: "Other creatures you control have \"Ward—Pay 2 life.\"",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    keyword: Keyword::Ward(WardCost::Life(2)),
                },
            },
        ],
        ..c_creature(
            "Hexing Squelcher",
            cost(&[generic(1), r()]),
            vec![CreatureType::Goblin, CreatureType::Sorcerer],
            2,
            2,
        )
    }
}

// ── _____ Goblin (Unfinity sticker card) ────────────────────────────────────

/// _____ Goblin — {2}{R} Creature — Goblin Guest 2/2. When this creature
/// enters, you may put a name sticker on it. Add {R} for each unique vowel on
/// that sticker. (The vowels are A, E, I, O, U, and Y.)
///
/// `Effect::PutNameSticker` (CR 123.3/123.6): the ballot is the controller's
/// unused name stickers, richest first, plus "No sticker"; the chosen word
/// fills the blank ("Audacious Goblin") and the sticker stays taken while the
/// card is in a public zone. The mana reads `Value::NameStickerUniqueVowels`,
/// 0 when no sticker went on.
///
/// Approximations: the sticker sheets are the engine's stand-in collection
/// (`sticker::DEFAULT_STICKER_SHEETS`), and the name position isn't offered —
/// the sticker always fills the blank.
pub fn blank_goblin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PutNameSticker { what: Selector::This, optional: true },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Red, Value::NameStickerUniqueVowels),
            },
        ]))],
        ..c_creature(
            "_____ Goblin",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Guest],
            2,
            2,
        )
    }
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// Arena of Glory — Land. This land enters tapped unless you control a
/// Mountain. `{T}: Add {R}.` `{R}, {T}, Exert this land: Add {R}{R}. If that
/// mana is spent on a creature spell, it gains haste until end of turn.`
///
/// The haste rider is `SpendRestriction::CreatureHaste` (Generator
/// Servant). Approximation: the exert cost is folded into the mana ability's
/// effect as `SkipNextUntap` on itself (the ability stays a mana ability —
/// CR 605.1a allows a non-targeting rider beside the mana).
pub fn arena_of_glory() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless you control a Mountain.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasLandType(LandType::Mountain).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
            },
        }],
        activated_abilities: vec![
            tap_add(Color::Red),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[r()]),
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Restricted(
                            Box::new(ManaPayload::Colors(vec![Color::Red, Color::Red])),
                            SpendRestriction::CreatureHaste,
                        ),
                    },
                    Effect::SkipNextUntap { what: Selector::This },
                ]),
                ..Default::default()
            },
        ],
        ..c_land("Arena of Glory")
    }
}

/// Spire Garden — Land. This land enters tapped unless you have two or more
/// opponents. `{T}: Add {R} or {G}.` (Battlebond "Luxury Suite" cycle;
/// `Value::OpponentCount` counts opponents still in the game.)
pub fn spire_garden() -> CardDefinition {
    super::super::cmdr::crowd_land("Spire Garden", Color::Red, Color::Green)
}

/// Rockfall Vale — Land (slow land). This land enters tapped unless you
/// control two or more other lands. `{T}: Add {R} or {G}.` A replacement
/// (`EntersTappedUnless`, Gingerbread Cabin's `OtherThanSource` count) rather
/// than `lands.rs`'s `slow_land` ETB trigger.
pub fn rockfall_vale() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless you control two or more other lands.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Land.and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    n: Value::Const(2),
                },
            },
        }],
        activated_abilities: vec![tap_add(Color::Red), tap_add(Color::Green)],
        ..c_land("Rockfall Vale")
    }
}

/// Game Trail — Land. As this land enters, you may reveal a Mountain or
/// Forest card from your hand. If you don't, this land enters tapped.
/// `{T}: Add {R} or {G}.` (The Snarl shape — `Effect::IfRevealFromHand`.)
///
/// Approximation: like the Snarls, the check is a self ETB trigger rather
/// than an as-enters replacement, and the reveal is automatic when a match
/// exists.
pub fn game_trail() -> CardDefinition {
    crate::sets::land_type_reveal_land(
        "Game Trail",
        "As this land enters, you may reveal a Mountain or Forest card from your hand. If you don't, this land enters tapped.",
        LandType::Mountain,
        LandType::Forest,
        Color::Red,
        Color::Green,
    )
}

/// Fire-Lit Thicket — Land (Shadowmoor allied filter land). `{T}: Add {C}.`
/// `{R/G}, {T}: Add {R}{R}, {R}{G}, or {G}{G}.` The second ability is two pips
/// each chosen from {R, G} (`ManaPayload::OfColors`), which is exactly the
/// three printed options — the shape `sets::hybrid_filter_land` now gives all ten.
pub fn fire_lit_thicket() -> CardDefinition {
    crate::sets::hybrid_filter_land("Fire-Lit Thicket", Color::Red, Color::Green)
}

/// Wooded Ridgeline — Land — Mountain Forest. ({T}: Add {R} or {G}.) This
/// land enters tapped. (Typed tapland: `dual_land_with` plus the
/// `EntersTapped` replacement.)
pub fn wooded_ridgeline() -> CardDefinition {
    let mut d = dual_land_with(
        "Wooded Ridgeline",
        LandType::Mountain,
        LandType::Forest,
        Color::Red,
        Color::Green,
        vec![],
    );
    d.static_abilities.push(StaticAbility {
        description: "This land enters tapped.",
        effect: StaticEffect::EntersTapped { applies_to: Selector::This },
    });
    d
}

/// "When this enters, you may discard a card. If you do, search your library
/// for a [filter] card, reveal it, put it into your hand, then shuffle." —
/// Pia, Aether Ascetic and Formidable Speaker (Kickoff Celebrations'
/// `MayDiscard` gate + a `Search` to hand).
fn discard_to_tutor(filter: R, what: &str) -> TriggeredAbility {
    etb(Effect::MayDiscard {
        description: format!("Discard a card to search for {what} card?"),
        count: Value::ONE,
        then: Box::new(Effect::Search {
            who: PlayerRef::You,
            filter,
            to: ZoneDest::Hand(PlayerRef::You),
        }),
        else_: None,
    })
}

/// Pia, Aether Ascetic — {2}{G} Legendary Creature — Human Druid 2/2. When Pia
/// enters, you may discard a card. If you do, search your library for an
/// enchantment card, reveal it, put it into your hand, then shuffle.
/// Approximation: the discarded card is the engine's `MayDiscard` auto-pick
/// (highest mana value), not a free choice.
pub fn pia_aether_ascetic() -> CardDefinition {
    CardDefinition {
        name: "Pia, Aether Ascetic",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![discard_to_tutor(
            R::HasCardType(CardType::Enchantment),
            "an enchantment",
        )],
        ..Default::default()
    }
}

/// Formidable Speaker — {2}{G} Creature — Elf Druid 2/4. When this creature
/// enters, you may discard a card. If you do, search your library for a
/// creature card, reveal it, put it into your hand, then shuffle. {1}, {T}:
/// Untap another target permanent.
/// Approximation: the discarded card is the engine's `MayDiscard` auto-pick
/// (highest mana value), not a free choice.
pub fn formidable_speaker() -> CardDefinition {
    CardDefinition {
        name: "Formidable Speaker",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![discard_to_tutor(R::Creature, "a creature")],
        // Kiora's Follower's untap, plus the {1}.
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Untap {
                what: target_filtered(R::Permanent.and(R::OtherThanSource)),
                up_to: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Curse-Marred Demon — {2}{R}{R} Creature — Demon 4/4. Flying, trample. When
/// this creature enters, search your library for a card, put it into your
/// hand, shuffle, then discard a card at random.
pub fn curse_marred_demon() -> CardDefinition {
    CardDefinition {
        name: "Curse-Marred Demon",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::Any,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::ONE,
                random: true,
            },
        ]))],
        ..Default::default()
    }
}

/// Temur Sabertooth — {2}{G}{G} Creature — Cat 4/3. {1}{G}: You may return
/// another creature you control to its owner's hand. If you do, this creature
/// gains indestructible until end of turn.
/// The "you may" is only offered when another creature exists (so the
/// indestructible rider can't come for free); the returned creature is chosen
/// at resolution (`PlayerReturnsPermanentsToHand`, not a target).
pub fn temur_sabertooth() -> CardDefinition {
    let others = R::Creature.and(R::ControlledByYou).and(R::OtherThanSource);
    CardDefinition {
        name: "Temur Sabertooth",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::count(Selector::EachPermanent(others.clone())),
                    Value::ONE,
                ),
                then: Box::new(Effect::MayDo {
                    description: "Return another creature you control to hand?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::PlayerReturnsPermanentsToHand {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            filter: others,
                            up_to: false,
                        },
                        Effect::GrantKeyword {
                            what: Selector::This,
                            keyword: Keyword::Indestructible,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kogla and Yidaro — {2}{R}{R}{G}{G} Legendary Creature — Ape Dinosaur Turtle
/// 7/7. When it enters, choose one — it gains trample and haste until end of
/// turn; or it fights target creature you don't control. {2}{R}{G}, Discard
/// this card: Destroy up to one target artifact or enchantment. Shuffle this
/// card into your library from your graveyard, then draw a card.
/// Approximation: "creature you don't control" is `ControlledByOpponent` (the
/// sibling fight-ETB convention); the shuffle-back moves the card from
/// wherever it is (normally the graveyard it was just discarded to).
pub fn kogla_and_yidaro() -> CardDefinition {
    CardDefinition {
        name: "Kogla and Yidaro",
        cost: cost(&[generic(2), r(), r(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ape, CreatureType::Dinosaur, CreatureType::Turtle],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::GrantKeywords {
                what: Selector::This,
                keywords: vec![Keyword::Trample, Keyword::Haste],
                duration: Duration::EndOfTurn,
            },
            Effect::Fight {
                attacker: Selector::This,
                defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
        ]))],
        // Trumpeting Carnosaur's "Discard this card:" from-hand shape.
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r(), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Destroy {
                        what: target_filtered(R::Artifact.or(R::Enchantment)),
                    }),
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Shuffled },
                },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Valakut Stoneforge — the land back of Valakut Awakening: enters tapped,
/// {T}: Add {R}. (The ZNR `znr_mdfc_land` shape; that helper is private to
/// `modern.rs`.)
fn valakut_stoneforge() -> CardDefinition {
    CardDefinition {
        name: "Valakut Stoneforge",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![crate::catalog::sets::tap_add(Color::Red)],
        ..Default::default()
    }
}

/// Valakut Awakening // Valakut Stoneforge — {2}{R} Instant // Land (modal
/// DFC). Put any number of cards from your hand on the bottom of your library,
/// then draw that many cards plus one. Back: enters tapped, {T}: Add {R}.
/// Approximation: the chosen cards are *shuffled into* the library rather than
/// put on the bottom (`ShuffleAnyNumberFromHandThenDraw`, Credit Voucher's
/// primitive), and the pick is asked through the synchronous decider.
pub fn valakut_awakening() -> CardDefinition {
    CardDefinition {
        name: "Valakut Awakening",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ShuffleAnyNumberFromHandThenDraw { who: PlayerRef::You },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        back_face: Some(Box::new(valakut_stoneforge())),
        ..Default::default()
    }
}

/// Last March of the Ents — {6}{G}{G} Sorcery. This spell can't be countered.
/// Draw cards equal to the greatest toughness among creatures you control,
/// then put any number of creature cards from your hand onto the battlefield.
pub fn last_march_of_the_ents() -> CardDefinition {
    CardDefinition {
        name: "Last March of the Ents",
        cost: cost(&[generic(6), g(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Seq(vec![
            // `GreatestToughnessYouControl` skips the effect's source — the
            // sorcery on the stack, never a creature — so it is "among
            // creatures you control" here.
            Effect::Draw {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::GreatestToughnessYouControl)),
            },
            Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature,
                count: Value::count(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Hand,
                    filter: R::Creature,
                }),
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
        ]),
        ..Default::default()
    }
}

/// Cream of the Crop — {1}{G} Enchantment. Whenever a creature you control
/// enters, you may look at the top X cards of your library, where X is that
/// creature's power. If you do, put one of those cards on top of your library
/// and the rest on the bottom in any order.
/// Approximation: modeled as scry X (`Scry`), which additionally lets you keep
/// more than one card on top, and counts as a scry for "whenever you scry"
/// triggers.
pub fn cream_of_the_crop() -> CardDefinition {
    CardDefinition {
        name: "Cream of the Crop",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::MayDo {
                description: "Look at the top X cards of your library?".into(),
                body: Box::new(Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Elemental Bond — {2}{G} Enchantment. Whenever a creature you control with
/// power 3 or greater enters, draw a card.
pub fn elemental_bond() -> CardDefinition {
    CardDefinition {
        name: "Elemental Bond",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtLeast(3)),
                }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Minsc & Boo, Timeless Heroes — {2}{R}{G} Legendary Planeswalker — Minsc,
/// loyalty 3. When Minsc & Boo enters and at the beginning of your upkeep, you
/// may create Boo, a legendary 1/1 red Hamster creature token with trample and
/// haste. +1: Put three +1/+1 counters on up to one target creature with
/// trample or haste. −2: Sacrifice a creature. When you do, Minsc & Boo deals
/// X damage to any target, where X is that creature's power. If the
/// sacrificed creature was a Hamster, draw X cards. Minsc & Boo, Timeless
/// Heroes can be your commander.
/// Approximation: the −2's damage target is chosen on activation rather than
/// by the reflexive "when you do" trigger, and the Hamster draw resolves
/// before the damage (so the sacrificed token is still readable). With no
/// creature to sacrifice X is 0.
/// Omitted: the Minsc planeswalker subtype (`PlaneswalkerSubtype` has no
/// `Minsc`).
pub fn minsc_boo_timeless_heroes() -> CardDefinition {
    let boo = std::sync::Arc::new(TokenDefinition {
        name: "Boo".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Trample, Keyword::Haste],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hamster],
            ..Default::default()
        },
        ..Default::default()
    });
    let may_boo = || Effect::MayDo {
        description: "Create Boo, a legendary 1/1 Hamster with trample and haste?".into(),
        body: Box::new(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: boo.clone(),
        }),
    };
    CardDefinition {
        name: "Minsc & Boo, Timeless Heroes",
        cost: cost(&[generic(2), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        base_loyalty: 3,
        can_be_commander: true,
        triggered_abilities: vec![
            etb(may_boo()),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: may_boo(),
            },
        ],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::AddCounter {
                        what: target_filtered(
                            R::Creature
                                .and(R::HasKeyword(Keyword::Trample).or(R::HasKeyword(Keyword::Haste))),
                        ),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(3),
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    // `Sacrifice` routes through `sacrifice_one`, which stamps
                    // `SacrificedPower` and `SacrificedCard` (Serendib Djinn).
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::ONE,
                        filter: R::Creature,
                    },
                    Effect::If {
                        cond: Predicate::EntityMatchesAny {
                            what: Selector::SacrificedCard,
                            filter: R::HasCreatureType(CreatureType::Hamster),
                        },
                        then: Box::new(Effect::Draw {
                            who: Selector::You,
                            amount: Value::SacrificedPower,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                    Effect::DealDamage { to: target_any(), amount: Value::SacrificedPower },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Invasion of Ikoria // Zilortha, Apex of Ikoria — {X}{G}{G} Battle — Siege,
/// defense 6. When this Siege enters, search your library and/or graveyard for
/// a non-Human creature card with mana value X or less and put it onto the
/// battlefield. If you search your library this way, shuffle. Back: Zilortha,
/// Apex of Ikoria — Legendary 8/8 Dinosaur with reach; for each non-Human
/// creature you control, you may have that creature assign its combat damage
/// as though it weren't blocked.
/// Approximation: the library is always searched (and shuffled) alongside the
/// graveyard; Zilortha's "you may" is always taken (a blocked non-Human
/// attacker always assigns its damage to the player, never to its blockers).
pub fn invasion_of_ikoria() -> CardDefinition {
    let zilortha = CardDefinition {
        name: "Zilortha, Apex of Ikoria",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Non-Human creatures you control may assign combat damage as though they weren't blocked.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Human)))),
                ),
                keyword: Keyword::AssignsDamageAsThoughUnblocked,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Ikoria",
        cost: cost(&[x(), g(), g()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes {
            battle_subtypes: vec![BattleSubtype::Siege],
            ..Default::default()
        },
        defense: 6,
        // Dune Drifter's ETB reads the cast X the same way.
        triggered_abilities: vec![etb(Effect::SearchZones {
            who: PlayerRef::You,
            zones: vec![Zone::Library, Zone::Graveyard],
            filter: R::Creature
                .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Human))))
                .and(R::ManaValueAtMostXFromCost),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        back_face: Some(Box::new(zilortha)),
        ..Default::default()
    }
}

/// Strionic Resonator — {2} Artifact. {2}, {T}: Copy target triggered ability
/// you control. You may choose new targets for the copy.
/// Approximation (Gogo's `CopyAbility`): the target is the ability's *source
/// permanent*, so a trigger whose source has left the battlefield (a dies
/// trigger) can't be targeted; the copy keeps the original's targets.
pub fn strionic_resonator() -> CardDefinition {
    CardDefinition {
        name: "Strionic Resonator",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::CopyAbility {
                what: target_filtered(R::HasAbilityOnStack.and(R::ControlledByYou)),
                times: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hunting Velociraptor — {2}{R} Creature — Dinosaur 3/2. First strike.
/// Dinosaur spells you cast have prowl {2}{R} (`StaticEffect::
/// GrantProwlToSpells`, read by `effective_alternative_cost`; the prowl gate
/// uses the spell's own creature types).
/// Approximation: a Dinosaur with a printed alternative cost keeps that one
/// instead (a card carries one alternative cost).
pub fn hunting_velociraptor() -> CardDefinition {
    CardDefinition {
        name: "Hunting Velociraptor",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "Dinosaur spells you cast have prowl {2}{R}.",
            effect: StaticEffect::GrantProwlToSpells {
                filter: R::HasCreatureType(CreatureType::Dinosaur),
                cost: cost(&[generic(2), r()]),
            },
        }],
        ..Default::default()
    }
}
