//! Commander: EDHREC ranks 2001-2500, one-card gaps (second batch) — Ginger,
//! Kunoros, Urabrask the Hidden, Saryth, Sokka Bold Boomeranger, Thalia's
//! deck, Sylvan Awakening's, Grakmaw, Ink-Eyes, Syr Carah, Sarevok, Lavinia
//! of the Tenth. Tests in `tests/recent_b/cmdr_edhrec56.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, w, ManaCost};

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

/// Staunch Throneguard — {5} 2/5 artifact creature Construct, vigilance.
/// Enters: you become the monarch.
pub fn staunch_throneguard() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::BecomeMonarch { who: PlayerRef::You })],
        ..creature("Staunch Throneguard", cost(&[generic(5)]), vec![CreatureType::Construct], 2, 5)
    }
}

/// Rambunctious Mutt — {3}{W}{W} 3/4 Dog. Enters: destroy target artifact or
/// enchantment an opponent controls.
pub fn rambunctious_mutt() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(R::Artifact.or(R::Enchantment).and(R::ControlledByOpponent)),
        })],
        ..creature("Rambunctious Mutt", cost(&[generic(3), w(), w()]), vec![CreatureType::Dog], 3, 4)
    }
}

/// Rage Reflection — {4}{R}{R} enchantment. Creatures you control have double
/// strike.
pub fn rage_reflection() -> CardDefinition {
    CardDefinition {
        name: "Rage Reflection",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have double strike.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::DoubleStrike,
            },
        }],
        ..Default::default()
    }
}

/// Scattershot Archer — {G} 1/2 Elf Archer. {T}: 1 damage to each creature
/// with flying.
pub fn scattershot_archer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature("Scattershot Archer", cost(&[g()]), vec![CreatureType::Elf, CreatureType::Archer], 1, 2)
    }
}

/// Mechanical Glider — {1} Equipment. Enters: attach it to target creature you
/// control. Equipped creature has flying. Equip {2}.
pub fn mechanical_glider() -> CardDefinition {
    CardDefinition {
        name: "Mechanical Glider",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Flying], ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        ..Default::default()
    }
}

/// Thalia, the Survivor — {3}{W} legendary 3/4 Human Soldier, lifelink.
/// Noncreature spells your opponents cast cost {1} more.
pub fn thalia_the_survivor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells your opponents cast cost {1} more to cast.",
            effect: StaticEffect::OpponentSpellsCostMore { filter: R::Noncreature, amount: 1 },
        }],
        ..legend("Thalia, the Survivor", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Soldier], 3, 4)
    }
}

/// Sylvan Awakening — {2}{G} sorcery. Until your next turn, all lands you
/// control become 2/2 Elemental creatures with reach, indestructible and
/// haste; they're still lands.
pub fn sylvan_awakening() -> CardDefinition {
    CardDefinition {
        name: "Sylvan Awakening",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::BecomeCreature {
            what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::Const(2),
            creature_types: vec![CreatureType::Elemental],
            keywords: vec![Keyword::Reach, Keyword::Indestructible, Keyword::Haste],
            duration: Duration::UntilNextTurn,
        },
        ..Default::default()
    }
}

/// Skyclave Shadowcat — {3}{B} 3/3 Cat Horror. {1}{B}, sacrifice another
/// creature: a +1/+1 counter on it. Whenever a creature you control with a
/// +1/+1 counter on it dies, draw a card (CR 603.10a — read as it last
/// existed).
pub fn skyclave_shadowcat() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Creature.and(R::OtherThanSource), 1)),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::TriggerSourceHadCounter(CounterType::PlusOnePlusOne)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature("Skyclave Shadowcat", cost(&[generic(3), b()]), vec![CreatureType::Cat, CreatureType::Horror], 3, 3)
    }
}

/// Mukotai Ambusher — {3}{B} 3/2 artifact creature Rat Ninja. Ninjutsu {1}{B};
/// lifelink.
pub fn mukotai_ambusher() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), b()])), Keyword::Lifelink],
        ..creature("Mukotai Ambusher", cost(&[generic(3), b()]), vec![CreatureType::Rat, CreatureType::Ninja], 3, 2)
    }
}

/// Syr Carah, the Bold — {3}{R}{R} legendary 3/3 Human Knight. Whenever she or
/// an instant or sorcery spell you control deals damage to a player, exile the
/// top card of your library; you may play it this turn. {T}: 1 damage to any
/// target.
pub fn syr_carah_the_bold() -> CardDefinition {
    let impulse = || Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::ONE,
        duration: crate::card::MayPlayDuration::EndOfThisTurn,
        pay_any_color: false,
        max_mana_value: None,
        pay_own_cost: true,
        uncast_penalty: None,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource),
                effect: impulse(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YourInstantOrSorceryDealtDamageToPlayer, EventScope::YourControl),
                effect: impulse(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_filtered(R::any_target()), amount: Value::ONE },
            ..Default::default()
        }],
        ..legend("Syr Carah, the Bold", cost(&[generic(3), r(), r()]), vec![CreatureType::Human, CreatureType::Knight], 3, 3)
    }
}

/// Sarevok, Deathbringer — {3}{B} legendary 3/4 Human Knight. At the beginning
/// of each player's end step, if no permanents left the battlefield this turn,
/// that player loses X life, X Sarevok's power. Choose a Background.
pub fn sarevok_deathbringer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::NoPermanentLeftBattlefieldThisTurn),
            effect: Effect::If {
                cond: Predicate::NoPermanentLeftBattlefieldThisTurn,
                then: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..legend("Sarevok, Deathbringer", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Knight], 3, 4)
    }
}

/// Tax Collector — {1}{W} 2/2 Human Advisor. Enters, choose one: until your
/// next turn, spells your opponents cast cost {1} more; or detain target
/// creature an opponent controls (CR 701.35).
pub fn tax_collector() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::SpellTaxUntilYourNextTurn { amount: 1, filter: R::Any },
            Effect::Detain { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
        ]))],
        ..creature("Tax Collector", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Advisor], 2, 2)
    }
}
