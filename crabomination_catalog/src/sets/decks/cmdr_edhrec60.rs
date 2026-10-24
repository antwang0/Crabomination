//! Commander: EDHREC ranks 2001-2500 — Kid Loki's four decks (Krang the
//! All-Powerful, Rikku, Danny Pink, Shang-Chi and the Ten Rings), Mikaeus the
//! Lunarch, Kwende, and the easy two-card gaps (Patron of the Kitsune,
//! Tinybones, Evin, Butch DeLoria (Evin and Zhentarim Bandit are `cmdr_rank2k_f`'s), Tibor and Lumia). Tests in
//! `tests/recent_b/cmdr_edhrec60.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef};
use crate::mana::{b, cost, generic, r, u, w, Color, ManaCost};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn second_draw(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl), effect }
}

fn grow_self() -> Effect {
    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }
}

fn treasure(tapped: bool) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition { tapped, ..crabomination_base::tokens::treasure_token() }),
    }
}

fn each_opponent_discards() -> Effect {
    Effect::Discard { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE, random: false }
}

/// Kid Loki — {U} legendary 1/1 God Hero Villain. Each creature you control
/// that you've put a +1/+1 counter on this turn has hexproof. Whenever you
/// draw your second card each turn, put a +1/+1 counter on Kid Loki.
pub fn kid_loki() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature you control that you've put one or more +1/+1 counters on this turn has \
                          hexproof.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::YouPutPlusOneCounterOnThisTurn),
                ),
                keyword: Keyword::Hexproof,
            },
        }],
        triggered_abilities: vec![second_draw(grow_self())],
        ..legend("Kid Loki", cost(&[u()]), vec![CreatureType::God, CreatureType::Hero, CreatureType::Villain], 1, 1)
    }
}

/// Thopter Mechanic — {1}{U} 2/1 Human Artificer. Your second draw each turn
/// grows it; when it dies, a 1/1 colorless flying Thopter.
pub fn thopter_mechanic() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            second_draw(grow_self()),
            on_dies(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Thopter".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                }),
            }),
        ],
        ..creature("Thopter Mechanic", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 1)
    }
}

/// Pym Particles — {U} sorcery. Target creature gains vigilance and can't be
/// blocked this turn. Draw a card.
pub fn pym_particles() -> CardDefinition {
    CardDefinition {
        name: "Pym Particles",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Basri's Solidarity — {1}{W} sorcery. A +1/+1 counter on each creature you
/// control.
pub fn basris_solidarity() -> CardDefinition {
    CardDefinition {
        name: "Basri's Solidarity",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::AddCounter {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
        ..Default::default()
    }
}

/// Chaplain of Alms // Chapel Shieldgeist — {W} 1/1 Human Cleric, first strike,
/// ward {1}, disturb {3}{W}. The back: a white 2/1 Spirit Cleric with flying
/// and first strike; each creature you control has ward {1} (CR 702.146).
pub fn chaplain_of_alms() -> CardDefinition {
    let ward1 = || Keyword::Ward(WardCost::Mana(cost(&[generic(1)])));
    let shieldgeist = CardDefinition {
        color_indicator: vec![Color::White],
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control has ward {1}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: ward1(),
            },
        }],
        ..creature("Chapel Shieldgeist", ManaCost::default(), vec![CreatureType::Spirit, CreatureType::Cleric], 2, 1)
    };
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, ward1(), Keyword::Disturb(cost(&[generic(3), w()]))],
        back_face: Some(Box::new(shieldgeist)),
        ..creature("Chaplain of Alms", cost(&[w()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 1)
    }
}

/// Clockwork Fox — {3} 3/2 artifact creature Fox. Leaves: you draw two and
/// each opponent draws one.
pub fn clockwork_fox() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            ]),
        }],
        ..creature("Clockwork Fox", cost(&[generic(3)]), vec![CreatureType::Fox], 3, 2)
    }
}

/// Silverchase Fox — {1}{W} 2/2 Fox. {1}{W}, sacrifice it: exile target
/// enchantment.
pub fn silverchase_fox() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            sac_cost: true,
            effect: Effect::Exile { what: target_filtered(R::Enchantment) },
            ..Default::default()
        }],
        ..creature("Silverchase Fox", cost(&[generic(1), w()]), vec![CreatureType::Fox], 2, 2)
    }
}

/// Rank Rat — {1}{B} 1/1 Zombie Rat. Enters: each opponent discards a card.
pub fn rank_rat() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(each_opponent_discards())],
        ..creature("Rank Rat", cost(&[generic(1), b()]), vec![CreatureType::Zombie, CreatureType::Rat], 1, 1)
    }
}

/// Stony-Voiced Goblins — {1}{B} 1/1 Goblin Bard. Enters: each opponent
/// discards a card.
pub fn stony_voiced_goblins() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(each_opponent_discards())],
        ..creature("Stony-Voiced Goblins", cost(&[generic(1), b()]), vec![CreatureType::Goblin, CreatureType::Bard], 1, 1)
    }
}

/// Ruthless Knave — {2}{B} 3/2 Orc Pirate. {2}{B}, sacrifice a creature: two
/// Treasures. Sacrifice three Treasures: draw a card.
pub fn ruthless_knave() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::Seq(vec![treasure(false), treasure(false)]),
                ..Default::default()
            },
            ActivatedAbility {
                sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Treasure), 3)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..creature("Ruthless Knave", cost(&[generic(2), b()]), vec![CreatureType::Orc, CreatureType::Pirate], 3, 2)
    }
}

/// Cloak and Dagger — {2} Kindred Artifact — Rogue Equipment. Equipped
/// creature gets +2/+0 and has shroud. Whenever a Rogue creature you control
/// enters, you may attach this to it. Equip {3}.
pub fn cloak_and_dagger() -> CardDefinition {
    CardDefinition {
        name: "Cloak and Dagger",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Kindred, CardType::Artifact],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rogue],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 2, keywords: vec![Keyword::Shroud], ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Rogue)),
                },
            ),
            effect: Effect::MayDo {
                description: "Attach Cloak and Dagger to the entering Rogue?".into(),
                body: Box::new(Effect::Attach { what: Selector::This, to: Selector::TriggerSource }),
            },
        }],
        ..Default::default()
    }
}

/// Firemind's Research — {U}{R} enchantment. Your instant and sorcery spells
/// add charge counters; {1}{U}, remove two: draw a card; {1}{R}, remove five:
/// 5 damage to any target.
pub fn fireminds_research() -> CardDefinition {
    CardDefinition {
        name: "Firemind's Research",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::CastSpellMatches(
                R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            )),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                remove_counter_cost: Some((CounterType::Charge, 2)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                remove_counter_cost: Some((CounterType::Charge, 5)),
                effect: Effect::DealDamage { to: target_filtered(R::Any), amount: Value::Const(5) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ionize — {1}{U}{R} instant. Counter target spell; 2 damage to that spell's
/// controller.
pub fn ionize() -> CardDefinition {
    CardDefinition {
        name: "Ionize",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}
