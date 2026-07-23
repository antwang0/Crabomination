//! War of the Spark (WAR) — 2019. Commons/uncommons on existing primitives.
//! Tests in `classic_sets/war`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, StaticAbility, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{cast_is_noncreature, etb, on_dies};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect};
use crate::mana::{b, cost, generic, r, u, w};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}

/// A plain creature body (no abilities).
fn vanilla(name: &'static str, mana: crate::mana::ManaCost, p: i32, t: i32, ct: Vec<CreatureType>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: creatures(ct),
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// A keyworded creature body.
fn keyworded(name: &'static str, mana: crate::mana::ManaCost, p: i32, t: i32, ct: Vec<CreatureType>, kw: Vec<Keyword>) -> CardDefinition {
    CardDefinition { keywords: kw, ..vanilla(name, mana, p, t, ct) }
}

/// Ironclad Krovod — {3}{W} 2/5 Beast.
pub fn ironclad_krovod() -> CardDefinition {
    vanilla("Ironclad Krovod", cost(&[generic(3), w()]), 2, 5, vec![CreatureType::Beast])
}

/// Naga Eternal — {2}{U} 3/2 Zombie Snake.
pub fn naga_eternal() -> CardDefinition {
    vanilla("Naga Eternal", cost(&[generic(2), u()]), 3, 2, vec![CreatureType::Zombie, CreatureType::Snake])
}

/// Lazotep Behemoth — {4}{B} 5/4 Zombie Hippo.
pub fn lazotep_behemoth() -> CardDefinition {
    vanilla("Lazotep Behemoth", cost(&[generic(4), b()]), 5, 4, vec![CreatureType::Zombie, CreatureType::Hippo])
}

/// Goblin Assailant — {1}{R} 2/2 Goblin Warrior.
pub fn goblin_assailant() -> CardDefinition {
    vanilla("Goblin Assailant", cost(&[generic(1), r()]), 2, 2, vec![CreatureType::Goblin, CreatureType::Warrior])
}

/// Enforcer Griffin — {4}{W} 3/4 Griffin with flying.
pub fn enforcer_griffin() -> CardDefinition {
    keyworded("Enforcer Griffin", cost(&[generic(4), w()]), 3, 4, vec![CreatureType::Griffin], vec![Keyword::Flying])
}

/// Banehound — {B} 1/1 Nightmare Dog with lifelink and haste.
pub fn banehound() -> CardDefinition {
    keyworded("Banehound", cost(&[b()]), 1, 1, vec![CreatureType::Nightmare, CreatureType::Dog], vec![Keyword::Lifelink, Keyword::Haste])
}

/// Charity Extractor — {3}{B} 1/5 Human Knight with lifelink.
pub fn charity_extractor() -> CardDefinition {
    keyworded("Charity Extractor", cost(&[generic(3), b()]), 1, 5, vec![CreatureType::Human, CreatureType::Knight], vec![Keyword::Lifelink])
}

/// Sunblade Angel — {5}{W} 3/3 Angel with flying, first strike, vigilance, lifelink.
pub fn sunblade_angel() -> CardDefinition {
    keyworded("Sunblade Angel", cost(&[generic(5), w()]), 3, 3, vec![CreatureType::Angel],
        vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Vigilance, Keyword::Lifelink])
}

/// Raging Kronch — {2}{R} 4/3 Beast that can't attack alone.
pub fn raging_kronch() -> CardDefinition {
    keyworded("Raging Kronch", cost(&[generic(2), r()]), 4, 3, vec![CreatureType::Beast], vec![Keyword::CantAttackAlone])
}

/// Bulwark Giant — {5}{W} 3/6 Giant Soldier. When it enters, you gain 5 life.
pub fn bulwark_giant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(5) })],
        ..vanilla("Bulwark Giant", cost(&[generic(5), w()]), 3, 6, vec![CreatureType::Giant, CreatureType::Soldier])
    }
}

/// Loxodon Sergeant — {3}{W} 3/3 Elephant Soldier with vigilance. When it
/// enters, other creatures you control gain vigilance until end of turn.
pub fn loxodon_sergeant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: Selector::OtherCreaturesControlledByControllerOf(Box::new(Selector::This)),
            keyword: Keyword::Vigilance,
            duration: Duration::EndOfTurn,
        })],
        ..vanilla("Loxodon Sergeant", cost(&[generic(3), w()]), 3, 3, vec![CreatureType::Elephant, CreatureType::Soldier])
    }
}

/// Kiora's Dambreaker — {5}{U} 5/6 Leviathan. When it enters, proliferate.
pub fn kioras_dambreaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Proliferate)],
        ..vanilla("Kiora's Dambreaker", cost(&[generic(5), u()]), 5, 6, vec![CreatureType::Leviathan])
    }
}

/// Martyr for the Cause — {1}{W} 2/2 Human Soldier. When it dies, proliferate.
pub fn martyr_for_the_cause() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Proliferate)],
        ..vanilla("Martyr for the Cause", cost(&[generic(1), w()]), 2, 2, vec![CreatureType::Human, CreatureType::Soldier])
    }
}

/// Rising Populace — {2}{W} 2/2 Human. Whenever another creature or planeswalker
/// you control dies, put a +1/+1 counter on Rising Populace.
pub fn rising_populace() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnotherOfYours).with_filter(
                crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: crate::card::SelectionRequirement::Creature
                        .or(crate::card::SelectionRequirement::HasCardType(CardType::Planeswalker)),
                },
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..vanilla("Rising Populace", cost(&[generic(2), w()]), 2, 2, vec![CreatureType::Human])
    }
}

/// Grateful Apparition — {1}{W} 1/1 Spirit with flying. Whenever it deals combat
/// damage to a player, proliferate. (The printed "or planeswalker" is player-only.)
pub fn grateful_apparition() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Proliferate,
        }],
        ..vanilla("Grateful Apparition", cost(&[generic(1), w()]), 1, 1, vec![CreatureType::Spirit])
    }
}

/// A "whenever you cast a noncreature spell, this gets +power/+0 until end of
/// turn" trigger, optionally scrying 1 afterward (Burning Prophet).
fn noncreature_pump(power: i32, scry: bool) -> TriggeredAbility {
    let pump = Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(power),
        toughness: Value::Const(0),
        duration: Duration::EndOfTurn,
    };
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(cast_is_noncreature()),
        effect: if scry {
            Effect::Seq(vec![pump, Effect::Scry { who: PlayerRef::You, amount: Value::ONE }])
        } else {
            pump
        },
    }
}

/// Sky Theater Strix — {1}{U} 1/2 Bird with flying. Whenever you cast a
/// noncreature spell, it gets +1/+0 until end of turn.
pub fn sky_theater_strix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![noncreature_pump(1, false)],
        ..vanilla("Sky Theater Strix", cost(&[generic(1), u()]), 1, 2, vec![CreatureType::Bird])
    }
}

/// Burning Prophet — {1}{R} 1/3 Human Wizard. Whenever you cast a noncreature
/// spell, it gets +1/+0 until end of turn, then scry 1.
pub fn burning_prophet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![noncreature_pump(1, true)],
        ..vanilla("Burning Prophet", cost(&[generic(1), r()]), 1, 3, vec![CreatureType::Human, CreatureType::Wizard])
    }
}

/// Thunder Drake — {3}{U} 2/3 Elemental Drake with flying. Whenever you cast
/// your second spell each turn, put a +1/+1 counter on it.
pub fn thunder_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::flurry(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..vanilla("Thunder Drake", cost(&[generic(3), u()]), 2, 3, vec![CreatureType::Elemental, CreatureType::Drake])
    }
}

/// Erratic Visionary — {1}{U} 1/3 Human Wizard. {1}{U}, {T}: Draw a card, then
/// discard a card.
pub fn erratic_visionary() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        ..vanilla("Erratic Visionary", cost(&[generic(1), u()]), 1, 3, vec![CreatureType::Human, CreatureType::Wizard])
    }
}

/// Vampire Opportunist — {1}{B} 2/1 Vampire. {6}{B}: Each opponent loses 2 life
/// and you gain 2 life.
pub fn vampire_opportunist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), b()]),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..vanilla("Vampire Opportunist", cost(&[generic(1), b()]), 2, 1, vec![CreatureType::Vampire])
    }
}

/// War Screecher — {1}{W} 1/3 Bird with flying. {5}{W}, {T}: Other creatures you
/// control get +1/+1 until end of turn.
pub fn war_screecher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), w()]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::OtherCreaturesControlledByControllerOf(Box::new(Selector::This)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..vanilla("War Screecher", cost(&[generic(1), w()]), 1, 3, vec![CreatureType::Bird])
    }
}

/// Pouncing Lynx — {1}{W} 2/1 Cat. During your turn, it has first strike.
pub fn pouncing_lynx() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has first strike.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::This,
                    keyword: Keyword::FirstStrike,
                }),
            },
        }],
        ..vanilla("Pouncing Lynx", cost(&[generic(1), w()]), 2, 1, vec![CreatureType::Cat])
    }
}

/// Ashiok's Skulker — {4}{U} 3/5 Nightmare. {3}{U}: This creature can't be
/// blocked this turn.
pub fn ashioks_skulker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..vanilla("Ashiok's Skulker", cost(&[generic(4), u()]), 3, 5, vec![CreatureType::Nightmare])
    }
}

/// Grim Initiate — {R} 1/1 Zombie Warrior with first strike. When it dies,
/// amass Zombies 1.
pub fn grim_initiate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![on_dies(Effect::Amass {
            who: PlayerRef::You,
            count: Value::ONE,
            extra_type: Some(CreatureType::Zombie),
        })],
        ..vanilla("Grim Initiate", cost(&[r()]), 1, 1, vec![CreatureType::Zombie, CreatureType::Warrior])
    }
}

/// Herald of the Dreadhorde — {3}{B} 3/2 Zombie Warrior. When it dies, amass
/// Zombies 2.
pub fn herald_of_the_dreadhorde() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Amass {
            who: PlayerRef::You,
            count: Value::Const(2),
            extra_type: Some(CreatureType::Zombie),
        })],
        ..vanilla("Herald of the Dreadhorde", cost(&[generic(3), b()]), 3, 2, vec![CreatureType::Zombie, CreatureType::Warrior])
    }
}
