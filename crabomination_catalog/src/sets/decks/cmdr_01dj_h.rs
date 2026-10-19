//! Commander: 2026-10-19 (`01DJHkWk`) — four-gap decks whose missing cards
//! are all simple: Zog, Taranika, Gert and Old Lace, She-Hulk Wallbreaker,
//! Wolfsbane, Thor. Tests in `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..def
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn aura(name: &'static str, mana: ManaCost, enchant: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        equipped_bonus: Some(bonus),
        ..spell(
            name,
            mana,
            CardType::Enchantment,
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(enchant),
            },
        )
    }
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn pump(what: Selector, power: i32, toughness: i32) -> Effect {
    Effect::PumpPT {
        what,
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    }
}

fn grant(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword {
        what,
        keyword,
        duration: Duration::EndOfTurn,
    }
}

fn self_ability(mana: ManaCost, effect: Effect) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        effect,
        ..Default::default()
    }
}

fn your_combat(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(TurnStep::BeginCombat),
            EventScope::YourControl,
        ),
        effect,
    }
}

// ── Zog, Triceraton Castaway ────────────────────────────────────────────────

/// Burning Sun's Avatar — {3}{R}{R}{R} 6/6 Dinosaur Avatar. Enters: 3 damage
/// to target opponent or planeswalker and 3 to up to one target creature.
pub fn burning_suns_avatar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::OpponentPlayer.or(R::Planeswalker)),
                    amount: Value::Const(3),
                },
                Effect::DealDamage {
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature,
                    },
                    amount: Value::Const(3),
                },
            ])),
        })],
        ..creature(
            "Burning Sun's Avatar",
            cost(&[generic(3), r(), r(), r()]),
            vec![CreatureType::Dinosaur, CreatureType::Avatar],
            6,
            6,
        )
    }
}

/// Craven Giant — {2}{R} 4/1 Giant that can't block.
pub fn craven_giant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        ..creature(
            "Craven Giant",
            cost(&[generic(2), r()]),
            vec![CreatureType::Giant],
            4,
            1,
        )
    }
}

/// Raging Minotaur — {2}{R}{R} 3/3 haste Minotaur Berserker.
pub fn raging_minotaur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        ..creature(
            "Raging Minotaur",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Minotaur, CreatureType::Berserker],
            3,
            3,
        )
    }
}

/// Rampaging Raptor — {2}{R}{R} 4/4 trample, haste Dinosaur. {2}{R}: +2/+0.
/// Its combat damage to an opponent deals that much to target planeswalker
/// that player controls or battle that player protects.
pub fn rampaging_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        activated_abilities: vec![self_ability(
            cost(&[generic(2), r()]),
            pump(Selector::This, 2, 0),
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::PlayerIsOpponent {
                    who: PlayerRef::TriggerEventPlayer,
                }),
            effect: Effect::DealDamage {
                to: target_filtered(
                    R::Planeswalker
                        .or(R::HasCardType(CardType::Battle))
                        .and(R::ControlledByTriggerPlayer),
                ),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature(
            "Rampaging Raptor",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Dinosaur],
            4,
            4,
        )
    }
}

// ── Taranika, Akroan Veteran ────────────────────────────────────────────────

/// Icatian Infantry — {W} 1/1 Human Soldier. {1}: first strike; {1}: banding;
/// each until end of turn.
pub fn icatian_infantry() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            self_ability(
                cost(&[generic(1)]),
                grant(Selector::This, Keyword::FirstStrike),
            ),
            self_ability(cost(&[generic(1)]), grant(Selector::This, Keyword::Banding)),
        ],
        ..creature(
            "Icatian Infantry",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Kjeldoran Knight — {W}{W} 1/1 banding Human Knight. {1}{W}: +1/+0;
/// {W}{W}: +0/+2; each until end of turn.
pub fn kjeldoran_knight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Banding],
        activated_abilities: vec![
            self_ability(cost(&[generic(1), w()]), pump(Selector::This, 1, 0)),
            self_ability(cost(&[w(), w()]), pump(Selector::This, 0, 2)),
        ],
        ..creature(
            "Kjeldoran Knight",
            cost(&[w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            1,
            1,
        )
    }
}

/// Kjeldoran Skyknight — {2}{W} 1/1 flying, first strike, banding Human Knight.
pub fn kjeldoran_skyknight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Banding],
        ..creature(
            "Kjeldoran Skyknight",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            1,
            1,
        )
    }
}

/// Shield Bearer — {1}{W} 0/3 banding Human Soldier.
pub fn shield_bearer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Banding],
        ..creature(
            "Shield Bearer",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            0,
            3,
        )
    }
}

// ── Gert and Old Lace / She-Hulk, Wallbreaker ───────────────────────────────

/// Experimental Armor — {2}{R} Equipment: +1/+1, flying and haste. Equip {2}.
pub fn experimental_armor() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying, Keyword::Haste],
            ..Default::default()
        }),
        ..spell(
            "Experimental Armor",
            cost(&[generic(2), r()]),
            CardType::Artifact,
            Effect::Noop,
        )
    }
}

/// Scarlet Witch, Wanda Maximoff — {2}{R} legendary 2/3 menace Mutant Warlock
/// Hero.
pub fn scarlet_witch_wanda_maximoff() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        ..legend(creature(
            "Scarlet Witch, Wanda Maximoff",
            cost(&[generic(2), r()]),
            vec![
                CreatureType::Mutant,
                CreatureType::Warlock,
                CreatureType::Hero,
            ],
            2,
            3,
        ))
    }
}

/// Warriors of Wakanda — {3}{G} 4/4 trample Human Warrior.
pub fn warriors_of_wakanda() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature(
            "Warriors of Wakanda",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Furious Strength — {2}{R} Aura: +2/+2 and menace.
pub fn furious_strength() -> CardDefinition {
    aura(
        "Furious Strength",
        cost(&[generic(2), r()]),
        R::Creature,
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Menace],
            ..Default::default()
        },
    )
}

/// Living Lightning, Charged Up — {4}{R} legendary 3/3 flying, haste
/// Elemental Hero. At the beginning of combat on your turn, another target
/// creature you control gets +1/+0 and haste until end of turn.
pub fn living_lightning_charged_up() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![your_combat(Effect::Seq(vec![
            pump(target_filtered(mine().and(R::OtherThanSource)), 1, 0),
            grant(Selector::Target(0), Keyword::Haste),
        ]))],
        ..legend(creature(
            "Living Lightning, Charged Up",
            cost(&[generic(4), r()]),
            vec![CreatureType::Elemental, CreatureType::Hero],
            3,
            3,
        ))
    }
}

/// Sif's Spearmaster — {2}{R} 1/4 God Warrior Hero. {T}: damage equal to its
/// power to target opponent.
pub fn sifs_spearmaster() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer),
                amount: Value::PowerOf(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        ..creature(
            "Sif's Spearmaster",
            cost(&[generic(2), r()]),
            vec![CreatureType::God, CreatureType::Warrior, CreatureType::Hero],
            1,
            4,
        )
    }
}

// ── Wolfsbane, Highland Hero ────────────────────────────────────────────────

/// Foe-liage — {3}{G} 3/3 Plant Mutant. Whenever a land enters during your
/// turn, put a +1/+1 counter on it.
pub fn foe_liage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Land,
                    },
                    Predicate::IsTurnOf(PlayerRef::You),
                ]),
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Foe-liage",
            cost(&[generic(3), g()]),
            vec![CreatureType::Plant, CreatureType::Mutant],
            3,
            3,
        )
    }
}

/// Primordial Pachyderm — {3}{G} 4/4 reach, trample Elephant Avatar; gain 2
/// life as it enters.
pub fn primordial_pachyderm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Trample],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..creature(
            "Primordial Pachyderm",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elephant, CreatureType::Avatar],
            4,
            4,
        )
    }
}

/// Accelerated Evolution — {2}{G} flash Aura on a creature you control:
/// +2/+2; entering, the creature gains hexproof until end of turn.
pub fn accelerated_evolution() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(grant(
            Selector::AttachedTo(Box::new(Selector::This)),
            Keyword::Hexproof,
        ))],
        ..aura(
            "Accelerated Evolution",
            cost(&[generic(2), g()]),
            mine(),
            EquipBonus {
                power: 2,
                toughness: 2,
                ..Default::default()
            },
        )
    }
}

/// Beast Mode — {1}{G} instant, teamwork 1. Target creature gets +2/+2 and
/// trample; with teamwork, also a +1/+1 counter.
pub fn beast_mode() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower {
            filter: mine(),
            power: 1,
        }),
        ..spell(
            "Beast Mode",
            cost(&[generic(1), g()]),
            CardType::Instant,
            Effect::Seq(vec![
                pump(target_filtered(R::Creature), 2, 2),
                grant(Selector::Target(0), Keyword::Trample),
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

// ── Thor, Asgard's Avenger ──────────────────────────────────────────────────

/// Molten Lavamancer — {2}{R} 2/3 prowess Elemental Warlock Villain. Once each
/// turn, a source of yours dealing noncombat damage to an opponent during
/// your turn makes a 1/1 red Elemental.
pub fn molten_lavamancer() -> CardDefinition {
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PlayerDealtNoncombatDamage,
                EventScope::YourSourceDamagedOpponent,
            )
            .with_filter(Predicate::IsTurnOf(PlayerRef::You))
            .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(elemental),
            },
        }],
        ..creature(
            "Molten Lavamancer",
            cost(&[generic(2), r()]),
            vec![
                CreatureType::Elemental,
                CreatureType::Warlock,
                CreatureType::Villain,
            ],
            2,
            3,
        )
    }
}

/// Weaver of Lightning — {2}{R} 1/4 reach Human Shaman. Your instant or
/// sorcery spells deal 1 damage to target creature an opponent controls.
pub fn weaver_of_lightning() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                ),
            ),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Weaver of Lightning",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            1,
            4,
        )
    }
}

/// Mjölnir's Might — {3}{R} sorcery. 4 damage to target player; exile your
/// top card, playable until the end of your next turn.
pub fn mjolnirs_might() -> CardDefinition {
    spell(
        "Mjölnir's Might",
        cost(&[generic(3), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Player),
                amount: Value::Const(4),
            },
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        ]),
    )
}
