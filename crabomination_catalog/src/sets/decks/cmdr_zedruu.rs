//! Commander: the cards the **Political Puppets** precon (CMD, Zedruu the
//! Greathearted) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_zedruu.rs`.
//!
//! Residuals (each also on its card):
//! - **Jötun Grunt** — the graveyard and the two cards for each installment
//!   are the engine's pick (an opponent's fullest graveyard, its highest mana
//!   values).
//! - **Ruhan of the Fomori** — the random opponent is re-drawn each combat,
//!   as printed, but stored on Ruhan (so a later effect that also stores a
//!   player on it overwrites the pick).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_filtered, target_n};
use crate::effect::{CounteredSpellZone, Duration, Effect, PlayerRef, Predicate};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, hybrid, r, u, w, x};
use crabomination_base::tokens::eldrazi_spawn_token;
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn spell(name: &'static str, mana: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn step(s: TurnStep, scope: EventScope) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(s), scope)
}

fn vow(name: &'static str, mana: ManaCost, keyword: Keyword) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![keyword, Keyword::CantAttackAuraController],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Brion Stoutarm — lifelink; {R},{T}, sacrifice another creature: damage
/// equal to its power to target player or planeswalker.
pub fn brion_stoutarm() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[r()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::SacrificedPower,
            },
            ..Default::default()
        }],
        ..creature(
            "Brion Stoutarm",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            4,
            4,
        )
    })
}

/// Crescendo of War — a strife counter each upkeep; attackers get +1/+0 per
/// counter, and so do your blockers.
pub fn crescendo_of_war() -> CardDefinition {
    let per = |applies_to| StaticEffect::PumpPTPerCounterOnSource {
        applies_to,
        kind: CounterType::Strife,
        per_power: 1,
        per_toughness: 0,
    };
    CardDefinition {
        name: "Crescendo of War",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: step(TurnStep::Upkeep, EventScope::AnyPlayer),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Strife, amount: Value::ONE },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "Attacking creatures get +1/+0 for each strife counter on this enchantment.",
                effect: per(Selector::EachPermanent(R::Creature.and(R::IsAttacking))),
            },
            StaticAbility {
                description: "Blocking creatures you control get +1/+0 for each strife counter on this enchantment.",
                effect: per(Selector::EachPermanent(R::Creature.and(R::IsBlocking).and(R::ControlledByYou))),
            },
        ],
        ..Default::default()
    }
}

/// Dominus of Fealty — flying; at your upkeep you may take target permanent
/// until end of turn, untapped and hasty.
pub fn dominus_of_fealty() -> CardDefinition {
    let ur = || hybrid(Color::Blue, Color::Red);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: step(TurnStep::Upkeep, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Gain control of target permanent until end of turn?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::GainControl {
                        what: target_filtered(R::Permanent.and(R::Not(Box::new(R::ControlledByYou)))),
                        to: None,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Untap { what: target_n(0), up_to: None },
                    Effect::GrantKeyword { what: target_n(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ])),
            },
        }],
        ..creature(
            "Dominus of Fealty",
            cost(&[ur(), ur(), ur(), ur(), ur()]),
            vec![CreatureType::Spirit, CreatureType::Avatar],
            4,
            4,
        )
    }
}

/// Jötun Grunt — cumulative upkeep: two cards from a single graveyard to the
/// bottom of their owner's library.
pub fn jotun_grunt() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::GraveyardCardsToBottom(2))],
        ..creature("Jötun Grunt", cost(&[generic(1), w()]), vec![CreatureType::Giant, CreatureType::Soldier], 4, 4)
    }
}

/// Martyr's Bond — this or another nonland permanent of yours going to a
/// graveyard makes each opponent sacrifice one sharing a card type with it.
pub fn martyrs_bond() -> CardDefinition {
    CardDefinition {
        name: "Martyr's Bond",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Not(Box::new(R::Land)) },
            ),
            effect: Effect::EachOpponentSacrificesSharingTypeWith { what: Selector::TriggerSource },
        }],
        ..Default::default()
    }
}

/// Nin, the Pain Artist — {X}{U}{R},{T}: X damage to target creature; its
/// controller draws X.
pub fn nin_the_pain_artist() -> CardDefinition {
    legendary(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[x(), u(), r()]),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::XFromCost },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(target_n(0)))),
                    amount: Value::XFromCost,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Nin, the Pain Artist",
            cost(&[u(), r()]),
            vec![CreatureType::Vedalken, CreatureType::Wizard],
            1,
            1,
        )
    })
}

/// Numot, the Devastator — flying; combat damage to a player: you may pay
/// {2}{R} to destroy up to two target lands.
pub fn numot_the_devastator() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {2}{R} to destroy up to two lands?".into(),
                mana_cost: cost(&[generic(2), r()]),
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Land,
                    effect: Box::new(Effect::Destroy { what: target_n(0) }),
                }),
                else_: None,
            },
        }],
        ..creature(
            "Numot, the Devastator",
            cost(&[generic(3), u(), r(), w()]),
            vec![CreatureType::Dragon],
            6,
            6,
        )
    })
}

/// Perilous Research — draw two, then sacrifice a permanent.
pub fn perilous_research() -> CardDefinition {
    spell(
        "Perilous Research",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Permanent },
        ]),
    )
}

/// Pollen Lullaby — prevent all combat damage this turn; clash, and on a win
/// the clashed opponent's creatures skip their next untap.
pub fn pollen_lullaby() -> CardDefinition {
    spell(
        "Pollen Lullaby",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::PreventAllCombatDamageThisTurn,
            Effect::ClashWithOpponent {
                on_win: Box::new(Effect::CreaturesDontUntapNextUntapStep {
                    who: Selector::Player(PlayerRef::ChosenPlayerOfSource),
                }),
            },
        ]),
    )
}

/// Prison Term — enchanted creature can't attack, block or activate; you may
/// move it onto a creature an opponent's creature entering.
pub fn prison_term() -> CardDefinition {
    CardDefinition {
        name: "Prison Term",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock, Keyword::CantActivateAbilities],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            ),
            effect: Effect::MayDo {
                description: "Move Prison Term onto the entering creature?".into(),
                body: Box::new(Effect::Attach { what: Selector::This, to: Selector::TriggerSource }),
            },
        }],
        ..Default::default()
    }
}

/// Rapacious One — trample; combat damage to a player makes that many
/// Eldrazi Spawn.
pub fn rapacious_one() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                definition: Arc::new(eldrazi_spawn_token()),
            },
        }],
        ..creature("Rapacious One", cost(&[generic(5), r()]), vec![CreatureType::Eldrazi, CreatureType::Drone], 5, 4)
    }
}

/// Ruhan of the Fomori — at the beginning of combat on your turn, an opponent
/// chosen at random is the one Ruhan attacks if able.
pub fn ruhan_of_the_fomori() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::MustAttackChosenPlayer],
        triggered_abilities: vec![TriggeredAbility {
            event: step(TurnStep::BeginCombat, EventScope::YourControl),
            effect: Effect::RememberPlayerOnSource { who: PlayerRef::RandomOpponent },
        }],
        ..creature(
            "Ruhan of the Fomori",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            7,
            7,
        )
    })
}

/// Scattering Stroke — counter target spell; clash, and on a win add {C}
/// equal to its mana value at your next main phase.
pub fn scattering_stroke() -> CardDefinition {
    spell(
        "Scattering Stroke",
        cost(&[generic(2), u(), u()]),
        Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::ClashWithOpponent {
                on_win: Box::new(Effect::AddManaAtNextMainPhase {
                    amount: Value::CounteredSpellManaValue,
                    any_color: false,
                }),
            },
        ]),
    )
}

/// Spell Crumple — counter target spell onto the bottom of its owner's
/// library; this goes to the bottom of its owner's too.
pub fn spell_crumple() -> CardDefinition {
    CardDefinition {
        library_bottom_on_resolve: true,
        ..spell(
            "Spell Crumple",
            cost(&[generic(1), u(), u()]),
            Effect::CounterSpellToZone {
                what: target_filtered(R::IsSpellOnStack),
                zone: CounteredSpellZone::OwnerLibraryBottom,
            },
        )
    }
}

/// Vow of Flight — +2/+2, flying, can't attack you or your planeswalkers.
pub fn vow_of_flight() -> CardDefinition {
    vow("Vow of Flight", cost(&[generic(2), u()]), Keyword::Flying)
}

/// Whirlpool Whelm — clash, then bounce target creature; on a win it goes on
/// top of its owner's library instead (the "may" always taken).
pub fn whirlpool_whelm() -> CardDefinition {
    let owner = || PlayerRef::OwnerOf(Box::new(target_n(0)));
    spell(
        "Whirlpool Whelm",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::ClashWithOpponent {
                on_win: Box::new(Effect::Move {
                    what: target_n(0),
                    to: crate::effect::ZoneDest::Library { who: owner(), pos: crate::effect::LibraryPosition::Top },
                }),
            },
            // The target slot is declared here: a clash payoff is not walked
            // for cast-time targets.
            Effect::If {
                cond: Predicate::EntityMatches { what: target_n(0), filter: R::OnBattlefield },
                then: Box::new(Effect::Move {
                    what: target_filtered(R::Creature),
                    to: crate::effect::ZoneDest::Hand(owner()),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}
