//! Commander: the 2026-10-16 rescan's shared gaps (The Temporal Anchor's five
//! decks first). Tests in `tests/recent_b/cmdr_oct16_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardId, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, MayPlayDuration, Predicate,
    SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, etb_drain_and_scry, on_attack, sneak, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, KeywordAct, LookPick, PlayerRef, Selector, StateTriggeredAbility, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legendary(d: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..d }
}

fn scry(n: i32) -> Effect {
    Effect::Scry { who: PlayerRef::You, amount: Value::Const(n) }
}

/// The Temporal Anchor — {3}{U}{U}{U} legendary artifact. Upkeep: scry 2.
/// Bottoming cards while scrying exiles that many from the library's bottom;
/// during your turn you may play the cards it exiled.
pub fn the_temporal_anchor() -> CardDefinition {
    legendary(CardDefinition {
        name: "The Temporal Anchor",
        cost: cost(&[generic(3), u(), u(), u()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: scry(2),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Performed(KeywordAct::ScryBottomed), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::ExileBottomOfLibraryLinked { amount: Value::TriggerEventAmount },
                    Effect::GrantMayPlay {
                        what: Selector::CardExiledWithSource,
                        duration: MayPlayDuration::HolderTurnsWhileSource { holder: 0, source: CardId(0), flash: false },
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                ]),
            },
        ],
        ..Default::default()
    })
}

/// Glorfindel, Dauntless Rescuer — {2}{G} legendary 3/2 Elf Noble. Whenever
/// you scry it gets +1/+1 and either must be blocked or can't be blocked by
/// more than one creature this turn.
pub fn glorfindel_dauntless_rescuer() -> CardDefinition {
    let mode = |keyword| {
        Effect::Seq(vec![
            Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::This, keyword, duration: Duration::EndOfTurn },
        ])
    };
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Scried, EventScope::YourControl),
            effect: Effect::ChooseMode(vec![mode(Keyword::MustBeBlocked), mode(Keyword::CantBeBlockedByMoreThanOne)]),
        }],
        ..creature("Glorfindel, Dauntless Rescuer", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Noble], 3, 2)
    })
}

/// Track Down — {1}{G} sorcery. Scry 3, then reveal the top card; draw it if
/// it's a creature or land card.
pub fn track_down() -> CardDefinition {
    CardDefinition {
        name: "Track Down",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            scry(3),
            Effect::RevealTopAndDrawIf {
                who: PlayerRef::You,
                reveal_filter: R::Creature.or(R::Land),
                may_graveyard_miss: false,
            },
        ]),
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, instant: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if instant { CardType::Instant } else { CardType::Sorcery }],
        effect,
        ..Default::default()
    }
}

fn aura(name: &'static str, mana: ManaCost, host: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: Selector::TargetFiltered { slot: 0, filter: host } },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn turtle(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    legendary(creature(name, mana, vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle], p, t))
}

/// "Target creature you control" in slot 0, "target creature an opponent
/// controls" in slot 1, fighting.
fn yours_fights_theirs() -> Effect {
    Effect::Fight {
        attacker: Selector::Target(0),
        defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
    }
}

fn your_creature() -> Selector {
    Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) }
}

// ── Saprolings ──────────────────────────────────────────────────────────────

/// Spore Swarm — {3}{G} instant. Create three 1/1 green Saprolings.
pub fn spore_swarm() -> CardDefinition {
    let saproling = TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    };
    spell(
        "Spore Swarm",
        cost(&[generic(3), g()]),
        true,
        Effect::CreateToken { who: PlayerRef::You, count: Value::Const(3), definition: Arc::new(saproling) },
    )
}

// ── TMNT ────────────────────────────────────────────────────────────────────

/// Ooze Spill — {1}{U}{U} instant. Counter target spell; create a Mutagen.
pub fn ooze_spill() -> CardDefinition {
    spell(
        "Ooze Spill",
        cost(&[generic(1), u(), u()]),
        true,
        Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(crabomination_base::tokens::mutagen_token()) },
        ]),
    )
}

/// Splinter's Technique — {3}{B} sorcery; sneak {1}{B}. Tutor a card.
pub fn splinters_technique() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sneak(cost(&[generic(1), b()]))],
        alternative_cost: Some(sneak(cost(&[generic(1), b()]))),
        ..spell(
            "Splinter's Technique",
            cost(&[generic(3), b()]),
            false,
            Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Hand(PlayerRef::You) },
        )
    }
}

/// Shredder, Unrelenting — {4}{B} legendary 6/4 deathtouch Human Ninja;
/// sneak {3}{B}. Entering or attacking, another creature you control gains
/// deathtouch until end of turn.
pub fn shredder_unrelenting() -> CardDefinition {
    let grant = || Effect::GrantKeyword {
        what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
        keyword: Keyword::Deathtouch,
        duration: Duration::EndOfTurn,
    };
    legendary(CardDefinition {
        keywords: vec![Keyword::Sneak(cost(&[generic(3), b()])), Keyword::Deathtouch],
        alternative_cost: Some(sneak(cost(&[generic(3), b()]))),
        triggered_abilities: vec![etb(grant()), on_attack(grant())],
        ..creature("Shredder, Unrelenting", cost(&[generic(4), b()]), vec![CreatureType::Human, CreatureType::Ninja], 6, 4)
    })
}

/// Disappear — at the beginning of your end step, if a permanent left the
/// battlefield under your control this turn, a +1/+1 counter on this.
fn disappear_counter() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
            .with_filter(Predicate::RevoltActive { who: PlayerRef::You }),
        effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
    }
}

/// Michelangelo, Game Master — {2}{G} legendary 3/3; Disappear: a +1/+1
/// counter.
pub fn michelangelo_game_master() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![disappear_counter()],
        ..turtle("Michelangelo, Game Master", cost(&[generic(2), g()]), 3, 3)
    }
}

/// Michelangelo's Technique — {4}{G} sorcery; sneak {3}{G}. Look at the top
/// eight: up to two creature cards with total mana value 6 or less onto the
/// battlefield, the rest on the bottom in a random order.
pub fn michelangelos_technique() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sneak(cost(&[generic(3), g()]))],
        alternative_cost: Some(sneak(cost(&[generic(3), g()]))),
        ..spell(
            "Michelangelo's Technique",
            cost(&[generic(4), g()]),
            false,
            Effect::LookPickToHand(Box::new(LookPick {
                count: Value::Const(8),
                take: Some(Value::Const(2)),
                pick_filter: Some(R::Creature),
                to_battlefield: true,
                optional: true,
                rest_bottom_random: true,
                total_mana_value_cap: Some(Value::Const(6)),
                ..Default::default()
            })),
        )
    }
}

/// Michelangelo, Improviser — {3}{G} legendary 4/4; sneak {2}{G}{G}. Combat
/// damage to a player: you may put a creature card and/or a land card from
/// your hand onto the battlefield.
pub fn michelangelo_improviser() -> CardDefinition {
    let put = |filter: R, what: &str| Effect::MayDo {
        description: format!("Put a {what} card from your hand onto the battlefield?"),
        body: Box::new(Effect::Move {
            what: Selector::take(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Hand, filter }, Value::ONE),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        }),
    };
    CardDefinition {
        keywords: vec![Keyword::Sneak(cost(&[generic(2), g(), g()]))],
        alternative_cost: Some(sneak(cost(&[generic(2), g(), g()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![put(R::Creature, "creature"), put(R::Land, "land")]),
        }],
        ..turtle("Michelangelo, Improviser", cost(&[generic(3), g()]), 4, 4)
    }
}

/// Koya, Death from Above — {2}{W} legendary 2/1 flying Mutant Ninja Bird.
/// Enters: exile up to one other target creature; at the next end step,
/// unless you pay {3}{B}, it returns under its owner's control.
pub fn koya_death_from_above() -> CardDefinition {
    let back = Effect::Move {
        what: Selector::Target(0),
        to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))), tapped: false },
    };
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Creature.and(R::OtherThanSource)) },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::MayPay {
                        description: "Pay {3}{B} to keep it exiled?".into(),
                        mana_cost: cost(&[generic(3), b()]),
                        body: Box::new(Effect::Noop),
                        else_: Some(Box::new(back)),
                    }),
                },
            ])),
        })],
        ..creature(
            "Koya, Death from Above",
            cost(&[generic(2), w()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Bird],
            2,
            1,
        )
    })
}

/// Dream Beavers — {B} 1/1 flying Beaver Nightmare. Enters: drain 1, scry 1.
pub fn dream_beavers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb_drain_and_scry(1, 1)],
        ..creature("Dream Beavers", cost(&[b()]), vec![CreatureType::Beaver, CreatureType::Nightmare], 1, 1)
    }
}

// ── Ezio ────────────────────────────────────────────────────────────────────

/// Hidden Footblade — {R} flash Equipment. Enters attached to a creature you
/// control, which gains first strike this turn; +1/+0 and haste; equip {2}.
pub fn hidden_footblade() -> CardDefinition {
    CardDefinition {
        name: "Hidden Footblade",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
        ]))],
        equipped_bonus: Some(EquipBonus { power: 1, keywords: vec![Keyword::Haste], ..Default::default() }),
        ..Default::default()
    }
}

/// Detained by Legionnaires — {1}{W} Aura; enchanted creature can't attack or
/// block.
pub fn detained_by_legionnaires() -> CardDefinition {
    aura(
        "Detained by Legionnaires",
        cost(&[generic(1), w()]),
        R::Creature,
        EquipBonus { keywords: vec![Keyword::CantAttack, Keyword::CantBlock], ..Default::default() },
    )
}

// ── Iwamori ─────────────────────────────────────────────────────────────────

/// Pitiless Fists — {3}{G} Aura (creature you control), +2/+2. Enters: the
/// enchanted creature fights up to one creature an opponent controls.
pub fn pitiless_fists() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Fight {
                attacker: Selector::AttachedTo(Box::new(Selector::This)),
                defender: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByOpponent) },
            }),
        })],
        ..aura(
            "Pitiless Fists",
            cost(&[generic(3), g()]),
            R::Creature.and(R::ControlledByYou),
            EquipBonus { power: 2, toughness: 2, ..Default::default() },
        )
    }
}

/// Outmuscle — {3}{G} sorcery. A +1/+1 counter on your creature, then it
/// fights a creature you don't control; adamant green: it gains
/// indestructible until end of turn.
pub fn outmuscle() -> CardDefinition {
    spell(
        "Outmuscle",
        cost(&[generic(3), g()]),
        false,
        Effect::Seq(vec![
            Effect::AddCounter { what: your_creature(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast { color: Color::Green, at_least: 3 },
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Fight {
                attacker: Selector::Target(0),
                defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
            },
        ]),
    )
}

// ── Kogla and Yidaro ────────────────────────────────────────────────────────

/// Gorilla War Cry — {1}{R} instant, cast only during combat before blockers.
/// All creatures gain menace this turn; draw a card next upkeep.
pub fn gorilla_war_cry() -> CardDefinition {
    CardDefinition {
        cast_only_before_blockers: true,
        ..spell(
            "Gorilla War Cry",
            cost(&[generic(1), r()]),
            true,
            Effect::Seq(vec![
                Effect::GrantKeyword { what: Selector::EachPermanent(R::Creature), keyword: Keyword::Menace, duration: Duration::EndOfTurn },
                Effect::AtNextTurnsUpkeep { body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }) },
            ]),
        )
    }
}

/// Hog-Monkey Rampage — {1}{R/G} instant. Your creature gets a +1/+1 counter
/// if its power is 4 or greater, then fights an opponent's creature.
pub fn hog_monkey_rampage() -> CardDefinition {
    spell(
        "Hog-Monkey Rampage",
        cost(&[generic(1), hybrid(Color::Red, Color::Green)]),
        true,
        Effect::Seq(vec![
            Effect::If {
                // The slot is declared in `then`: CR 601.2c's cast-time walker
                // reads effect positions, not a condition.
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::PowerAtLeast(4) },
                then: Box::new(Effect::AddCounter { what: your_creature(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
            yours_fights_theirs(),
        ]),
    )
}

// ── Ramirez DePietro ────────────────────────────────────────────────────────

/// Deadeye Tracker — {B} 1/1 Human Pirate. {1}{B}, {T}: exile two target
/// cards from an opponent's graveyard; it explores.
pub fn deadeye_tracker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 2,
                    filter: R::InOpponentGraveyard.and(R::SameGraveyardAsTargetSlot(0)),
                    effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
                },
                Effect::Explore { who: Selector::This },
            ]),
            ..Default::default()
        }],
        ..creature("Deadeye Tracker", cost(&[b()]), vec![CreatureType::Human, CreatureType::Pirate], 1, 1)
    }
}

/// Pirate Ship — {4}{U} 4/3 Human Pirate. Attacks only a player with an
/// Island; {T}: 1 damage to any target; sacrificed when you control no
/// Islands.
pub fn pirate_ship() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CanAttackOnlyIfDefenderControls(Box::new(R::HasLandType(LandType::Island)))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        state_trigger: Some(StateTriggeredAbility {
            condition: Predicate::Not(Box::new(Predicate::SelectorExists(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Island),
            }))),
            effect: Effect::SacrificeSource,
        }),
        ..creature("Pirate Ship", cost(&[generic(4), u()]), vec![CreatureType::Human, CreatureType::Pirate], 4, 3)
    }
}
