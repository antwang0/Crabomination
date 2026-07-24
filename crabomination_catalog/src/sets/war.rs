//! War of the Spark (WAR) — 2019. Commons/uncommons on existing primitives.
//! Tests in `classic_sets/war`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, PlaneswalkerSubtype, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::card::SelectionRequirement as R;
use crate::effect::shortcut::{cast_is_noncreature, deal, draw, etb, on_attack, on_dies, target_any, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, LoyaltyAbility, ManaPayload, PlayerRef, PlayerStaticTarget, Predicate, Selector, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}

/// "instant or sorcery card" filter.
fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
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

// ── Spells ──────────────────────────────────────────────────────────────────

/// Battlefield Promotion — {1}{W} Instant. Put a +1/+1 counter on target
/// creature. It gains first strike until end of turn. You gain 2 life.
pub fn battlefield_promotion() -> CardDefinition {
    CardDefinition {
        name: "Battlefield Promotion",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Rally of Wings — {1}{W} Instant. Untap all creatures you control. Creatures
/// you control with flying get +2/+2 until end of turn.
pub fn rally_of_wings() -> CardDefinition {
    CardDefinition {
        name: "Rally of Wings",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Untap { what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature }, up_to: None },
            Effect::PumpPT {
                what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature.and(R::HasKeyword(Keyword::Flying)) },
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Callous Dismissal — {1}{U} Sorcery. Return target nonland permanent to its
/// owner's hand. Amass Zombies 1.
pub fn callous_dismissal() -> CardDefinition {
    CardDefinition {
        name: "Callous Dismissal",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Nonland), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            Effect::Amass { who: PlayerRef::You, count: Value::ONE, extra_type: Some(CreatureType::Zombie) },
        ]),
        ..Default::default()
    }
}

/// Contentious Plan — {1}{U} Sorcery. Proliferate. Draw a card.
pub fn contentious_plan() -> CardDefinition {
    CardDefinition {
        name: "Contentious Plan",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Proliferate,
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Relentless Advance — {3}{U} Sorcery. Amass Zombies 3.
pub fn relentless_advance() -> CardDefinition {
    CardDefinition {
        name: "Relentless Advance",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Amass { who: PlayerRef::You, count: Value::Const(3), extra_type: Some(CreatureType::Zombie) },
        ..Default::default()
    }
}

/// Sorin's Thirst — {B}{B} Instant. Deal 2 damage to target creature and you
/// gain 2 life.
pub fn sorins_thirst() -> CardDefinition {
    CardDefinition {
        name: "Sorin's Thirst",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(2) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Unlikely Aid — {1}{B} Instant. Target creature gets +2/+0 and gains
/// indestructible until end of turn.
pub fn unlikely_aid() -> CardDefinition {
    CardDefinition {
        name: "Unlikely Aid",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT { what: target_filtered(R::Creature), power: Value::Const(2), toughness: Value::Const(0), duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Blindblast — {2}{R} Instant. Deal 1 damage to target creature. That creature
/// can't block this turn. Draw a card.
pub fn blindblast() -> CardDefinition {
    CardDefinition {
        name: "Blindblast",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Stealth Mission — {2}{U} Sorcery. Put two +1/+1 counters on target creature
/// you control. That creature can't be blocked this turn.
pub fn stealth_mission() -> CardDefinition {
    CardDefinition {
        name: "Stealth Mission",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter { what: target_filtered(R::Creature.and(R::ControlledByYou)), kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Ob Nixilis's Cruelty — {2}{B} Instant. Target creature gets -5/-5 until end
/// of turn. If that creature would die this turn, exile it instead.
pub fn ob_nixiliss_cruelty() -> CardDefinition {
    CardDefinition {
        name: "Ob Nixilis's Cruelty",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT { what: target_filtered(R::Creature), power: Value::Const(-5), toughness: Value::Const(-5), duration: Duration::EndOfTurn },
            Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
        ]),
        ..Default::default()
    }
}

// ── More creatures ──────────────────────────────────────────────────────────

/// Invading Manticore — {5}{R} 4/5 Zombie Manticore. When it enters, amass Zombies 2.
pub fn invading_manticore() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Amass { who: PlayerRef::You, count: Value::Const(2), extra_type: Some(CreatureType::Zombie) })],
        ..vanilla("Invading Manticore", cost(&[generic(5), r()]), 4, 5, vec![CreatureType::Zombie, CreatureType::Manticore])
    }
}

/// A "Zombie tokens you control have `kw`" static grant.
fn zombie_tokens_have(kw: Keyword, descr: &'static str) -> StaticAbility {
    StaticAbility {
        description: descr,
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Zombie).and(R::IsToken).and(R::ControlledByYou),
            ),
            keyword: kw,
        },
    }
}

/// Vizier of the Scorpion — {2}{B} 1/1 Zombie Wizard. ETB amass Zombies 1;
/// Zombie tokens you control have deathtouch.
pub fn vizier_of_the_scorpion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Amass { who: PlayerRef::You, count: Value::ONE, extra_type: Some(CreatureType::Zombie) })],
        static_abilities: vec![zombie_tokens_have(Keyword::Deathtouch, "Zombie tokens you control have deathtouch.")],
        ..vanilla("Vizier of the Scorpion", cost(&[generic(2), b()]), 1, 1, vec![CreatureType::Zombie, CreatureType::Wizard])
    }
}

/// Dreadhorde Twins — {3}{R} 2/2 Zombie Jackal Warrior. ETB amass Zombies 2;
/// Zombie tokens you control have trample.
pub fn dreadhorde_twins() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Amass { who: PlayerRef::You, count: Value::Const(2), extra_type: Some(CreatureType::Zombie) })],
        static_abilities: vec![zombie_tokens_have(Keyword::Trample, "Zombie tokens you control have trample.")],
        ..vanilla("Dreadhorde Twins", cost(&[generic(3), r()]), 2, 2, vec![CreatureType::Zombie, CreatureType::Jackal, CreatureType::Warrior])
    }
}

/// Tithebearer Giant — {5}{B} 4/5 Giant Warrior. When it enters, draw a card and lose 1 life.
pub fn tithebearer_giant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
        ]))],
        ..vanilla("Tithebearer Giant", cost(&[generic(5), b()]), 4, 5, vec![CreatureType::Giant, CreatureType::Warrior])
    }
}

/// Goblin Assault Team — {3}{R} 4/1 Goblin Warrior with haste. When it dies, put
/// a +1/+1 counter on target creature you control.
pub fn goblin_assault_team() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_dies(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..vanilla("Goblin Assault Team", cost(&[generic(3), r()]), 4, 1, vec![CreatureType::Goblin, CreatureType::Warrior])
    }
}

/// Shriekdiver — {2}{B} 2/1 Zombie Bird Warrior with flying. {1}: This creature
/// gains haste until end of turn.
pub fn shriekdiver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..vanilla("Shriekdiver", cost(&[generic(2), b()]), 2, 1, vec![CreatureType::Zombie, CreatureType::Bird, CreatureType::Warrior])
    }
}

/// Chainwhip Cyclops — {4}{R} 4/4 Cyclops Warrior. {3}{R}: Target creature can't
/// block this turn.
pub fn chainwhip_cyclops() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            effect: Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..vanilla("Chainwhip Cyclops", cost(&[generic(4), r()]), 4, 4, vec![CreatureType::Cyclops, CreatureType::Warrior])
    }
}

/// Law-Rune Enforcer — {W} 1/2 Human Soldier. {1}, {T}: Tap target creature with
/// mana value 2 or greater.
pub fn law_rune_enforcer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature.and(R::ManaValueAtLeast(2))) },
            ..Default::default()
        }],
        ..vanilla("Law-Rune Enforcer", cost(&[w()]), 1, 2, vec![CreatureType::Human, CreatureType::Soldier])
    }
}

/// Ahn-Crop Invader — {2}{R} 2/2 Zombie Minotaur Warrior. During your turn it has
/// first strike. {1}, Sacrifice another creature: It gets +2/+0 until end of turn.
pub fn ahn_crop_invader() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has first strike.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::FirstStrike }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::PumpPT { what: Selector::This, power: Value::Const(2), toughness: Value::Const(0), duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..vanilla("Ahn-Crop Invader", cost(&[generic(2), r()]), 2, 2, vec![CreatureType::Zombie, CreatureType::Minotaur, CreatureType::Warrior])
    }
}

/// Makeshift Battalion — {2}{W} 3/2 Human Soldier. Battalion — whenever it and at
/// least two other creatures attack, put a +1/+1 counter on it.
pub fn makeshift_battalion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::battalion(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..vanilla("Makeshift Battalion", cost(&[generic(2), w()]), 3, 2, vec![CreatureType::Human, CreatureType::Soldier])
    }
}

/// Spark Reaper — {2}{B} 2/3 Zombie. {3}, Sacrifice a creature or planeswalker:
/// You gain 1 life and draw a card.
pub fn spark_reaper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sac_other_filter: Some((R::Creature.or(R::HasCardType(CardType::Planeswalker)), 1)),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..vanilla("Spark Reaper", cost(&[generic(2), b()]), 2, 3, vec![CreatureType::Zombie])
    }
}

/// Duskmantle Operative — {1}{B} 2/2 Human Rogue. Can't be blocked by creatures
/// with power 4 or greater.
pub fn duskmantle_operative() -> CardDefinition {
    keyworded("Duskmantle Operative", cost(&[generic(1), b()]), 2, 2,
        vec![CreatureType::Human, CreatureType::Rogue], vec![Keyword::CantBeBlockedByPowerAtLeast(4)])
}

/// Vraska's Finisher — {2}{B} 3/2 Gorgon Assassin. When it enters, destroy target
/// creature or planeswalker an opponent controls that was dealt damage this turn.
pub fn vraskas_finisher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(
                R::ControlledByOpponent
                    .and(R::DealtDamageThisTurn)
                    .and(R::Creature.or(R::HasCardType(CardType::Planeswalker))),
            ),
        })],
        ..vanilla("Vraska's Finisher", cost(&[generic(2), b()]), 3, 2, vec![CreatureType::Gorgon, CreatureType::Assassin])
    }
}

// ── Batch 2 (2026-07-23): more commons/uncommons, artifacts, a Gate ───────────

/// Trusted Pegasus — {2}{W} 2/2 Pegasus with flying. Whenever it attacks, target
/// attacking creature without flying gains flying until end of turn.
pub fn trusted_pegasus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(R::IsAttacking.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying))))),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..vanilla("Trusted Pegasus", cost(&[generic(2), w()]), 2, 2, vec![CreatureType::Pegasus])
    }
}

/// Topple the Statue — {2}{W} Instant. Tap target permanent. If it's an
/// artifact, destroy it. Draw a card.
pub fn topple_the_statue() -> CardDefinition {
    CardDefinition {
        name: "Topple the Statue",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Permanent) },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Artifact },
                then: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                else_: Box::new(Effect::Noop),
            },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Eternal Skylord — {4}{U} 3/3 Zombie Wizard. ETB amass Zombies 2. Zombie
/// tokens you control have flying.
pub fn eternal_skylord() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Amass { who: PlayerRef::You, count: Value::Const(2), extra_type: Some(CreatureType::Zombie) })],
        static_abilities: vec![StaticAbility {
            description: "Zombie tokens you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Zombie).and(R::IsToken).and(R::ControlledByYou)),
                keyword: Keyword::Flying,
            },
        }],
        ..vanilla("Eternal Skylord", cost(&[generic(4), u()]), 3, 3, vec![CreatureType::Zombie, CreatureType::Wizard])
    }
}

/// Spellkeeper Weird — {2}{U} 1/4 Weird. {2}, {T}, Sacrifice this creature:
/// Return target instant or sorcery card from your graveyard to your hand.
pub fn spellkeeper_weird() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(instant_or_sorcery().and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..vanilla("Spellkeeper Weird", cost(&[generic(2), u()]), 1, 4, vec![CreatureType::Weird])
    }
}

/// Crush Dissent — {3}{U} Instant. Counter target spell unless its controller
/// pays {2}. Amass Zombies 2.
pub fn crush_dissent() -> CardDefinition {
    CardDefinition {
        name: "Crush Dissent",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(2)]),
                exile: false,
                extra_generic: None,
            },
            Effect::Amass { who: PlayerRef::You, count: Value::Const(2), extra_type: Some(CreatureType::Zombie) },
        ]),
        ..Default::default()
    }
}

/// No Escape — {2}{U} Instant. Counter target creature or planeswalker spell,
/// exiling it instead. Scry 1.
pub fn no_escape() -> CardDefinition {
    CardDefinition {
        name: "No Escape",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpellToZone {
                what: target_filtered(R::IsSpellOnStack.and(R::Creature.or(R::Planeswalker))),
                zone: crate::effect::CounteredSpellZone::Exile,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Bond of Insight — {3}{U} Sorcery. Each player mills four cards. Return up to
/// two instant and/or sorcery cards from your graveyard to your hand. Exile it.
pub fn bond_of_insight() -> CardDefinition {
    CardDefinition {
        name: "Bond of Insight",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(4) },
            Effect::ReturnGraveyardCardsToHand { filter: instant_or_sorcery(), max: Value::Const(2) },
            Effect::ExileResolvingSpell,
        ]),
        ..Default::default()
    }
}

/// Jace's Triumph — {2}{U} Sorcery. Draw two cards; three instead if you control
/// a Jace planeswalker.
pub fn jaces_triumph() -> CardDefinition {
    CardDefinition {
        name: "Jace's Triumph",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(R::HasPlaneswalkerType(PlaneswalkerSubtype::Jace).and(R::ControlledByYou)),
                n: Value::ONE,
            },
            then: Box::new(draw(3)),
            else_: Box::new(draw(2)),
        },
        ..Default::default()
    }
}

/// Saheeli's Silverwing — {4} 2/3 Drake artifact with flying. ETB look at the
/// top card of target opponent's library.
pub fn saheelis_silverwing() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::LookAtTop { who: PlayerRef::Target(0), amount: Value::ONE })],
        ..CardDefinition {
            name: "Saheeli's Silverwing",
            cost: cost(&[generic(4)]),
            subtypes: creatures(vec![CreatureType::Drake]),
            power: 2,
            toughness: 3,
            // ETB needs a target opponent; declared via the trigger's target slot.
            ..Default::default()
        }
    }
}

/// Dreadmalkin — {B} 1/1 Zombie Cat with menace. {2}{B}, Sacrifice another
/// creature or planeswalker: Put two +1/+1 counters on this creature.
pub fn dreadmalkin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_other_filter: Some((R::Creature.or(R::Planeswalker), 1)),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..vanilla("Dreadmalkin", cost(&[b()]), 1, 1, vec![CreatureType::Zombie, CreatureType::Cat])
    }
}

/// Lazotep Reaver — {1}{B} 1/2 Zombie Beast. ETB amass Zombies 1.
pub fn lazotep_reaver() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Amass { who: PlayerRef::You, count: Value::ONE, extra_type: Some(CreatureType::Zombie) })],
        ..vanilla("Lazotep Reaver", cost(&[generic(1), b()]), 1, 2, vec![CreatureType::Zombie, CreatureType::Beast])
    }
}

/// Aid the Fallen — {1}{B} Sorcery. Choose one or both — return target creature
/// card and/or target planeswalker card from your graveyard to your hand.
pub fn aid_the_fallen() -> CardDefinition {
    let ret = |filter: R| Effect::Move { what: target_filtered(filter.and(R::InYourGraveyard)), to: ZoneDest::Hand(PlayerRef::You) };
    CardDefinition {
        name: "Aid the Fallen",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![ret(R::Creature), ret(R::Planeswalker)],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Price of Betrayal — {B} Sorcery. Remove up to five counters from target
/// artifact, creature, planeswalker, or opponent.
pub fn price_of_betrayal() -> CardDefinition {
    CardDefinition {
        name: "Price of Betrayal",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RemoveCountersUpTo {
            what: target_filtered(R::Artifact.or(R::Creature).or(R::Planeswalker).or(R::OpponentPlayer)),
            amount: Value::Const(5),
        },
        ..Default::default()
    }
}

/// Cyclops Electromancer — {4}{R} 4/2 Cyclops Wizard. ETB deal X damage to
/// target creature an opponent controls, X = instant/sorcery cards in your gy.
pub fn cyclops_electromancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            amount: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: instant_or_sorcery() },
        })],
        ..vanilla("Cyclops Electromancer", cost(&[generic(4), r()]), 4, 2, vec![CreatureType::Cyclops, CreatureType::Wizard])
    }
}

/// Spellgorger Weird — {2}{R} 2/2 Weird. Whenever you cast a noncreature spell,
/// put a +1/+1 counter on this creature.
pub fn spellgorger_weird() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(cast_is_noncreature()),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..vanilla("Spellgorger Weird", cost(&[generic(2), r()]), 2, 2, vec![CreatureType::Weird])
    }
}

/// Tibalt's Rager — {1}{R} 1/2 Devil. When it dies, it deals 1 damage to any
/// target. {1}{R}: This creature gets +2/+0 until end of turn.
pub fn tibalts_rager() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(deal(1, target_any()))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT { what: Selector::This, power: Value::Const(2), toughness: Value::ZERO, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..vanilla("Tibalt's Rager", cost(&[generic(1), r()]), 1, 2, vec![CreatureType::Devil])
    }
}

/// Turret Ogre — {3}{R} 4/3 Ogre Warrior with reach. ETB — if you control
/// another creature with power 4 or greater, deal 2 damage to each opponent.
pub fn turret_ogre() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource).and(R::PowerAtLeast(4))),
                n: Value::ONE,
            },
            then: Box::new(deal(2, Selector::Player(PlayerRef::EachOpponent))),
            else_: Box::new(Effect::Noop),
        })],
        ..vanilla("Turret Ogre", cost(&[generic(3), r()]), 4, 3, vec![CreatureType::Ogre, CreatureType::Warrior])
    }
}

/// Heartfire — {1}{R} Instant. As an additional cost, sacrifice a creature or
/// planeswalker. Deals 4 damage to any target.
pub fn heartfire() -> CardDefinition {
    CardDefinition {
        name: "Heartfire",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: (R::Creature.or(R::Planeswalker)).and(R::ControlledByYou),
            count: 1,
        }],
        effect: deal(4, target_any()),
        ..Default::default()
    }
}

/// Chandra's Triumph — {1}{R} Instant. Deals 3 damage to target creature or
/// planeswalker an opponent controls; 5 instead if you control a Chandra.
pub fn chandras_triumph() -> CardDefinition {
    CardDefinition {
        name: "Chandra's Triumph",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered((R::Creature.or(R::Planeswalker)).and(R::ControlledByOpponent)),
            amount: Value::IfAtLeast {
                value: Box::new(Value::count(Selector::EachPermanent(
                    R::HasPlaneswalkerType(PlaneswalkerSubtype::Chandra).and(R::ControlledByYou),
                ))),
                threshold: 1,
                then: Box::new(Value::Const(5)),
                else_: Box::new(Value::Const(3)),
            },
        },
        ..Default::default()
    }
}

/// Samut's Sprint — {R} Instant. Target creature gets +2/+1 and gains haste
/// until end of turn. Scry 1.
pub fn samuts_sprint() -> CardDefinition {
    CardDefinition {
        name: "Samut's Sprint",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT { what: target_filtered(R::Creature), power: Value::Const(2), toughness: Value::ONE, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Nahiri's Stoneblades — {1}{R} Instant. Up to two target creatures each get
/// +2/+0 until end of turn.
pub fn nahiris_stoneblades() -> CardDefinition {
    CardDefinition {
        name: "Nahiri's Stoneblades",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::PumpPT { what: Selector::Target(0), power: Value::Const(2), toughness: Value::ZERO, duration: Duration::EndOfTurn }),
        },
        ..Default::default()
    }
}

/// Sarkhan's Catharsis — {4}{R} Instant. Deals 5 damage to target player or
/// planeswalker.
pub fn sarkhans_catharsis() -> CardDefinition {
    CardDefinition {
        name: "Sarkhan's Catharsis",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        effect: deal(5, target_filtered(R::Player.or(R::Planeswalker))),
        ..Default::default()
    }
}

/// Arboreal Grazer — {G} 0/3 Sloth Beast with reach. ETB you may put a land card
/// from your hand onto the battlefield tapped.
pub fn arboreal_grazer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: R::Land,
            count: Value::ONE,
            tapped: true,
            haste: false,
            sacrifice_eot: false,
        })],
        ..vanilla("Arboreal Grazer", cost(&[g()]), 0, 3, vec![CreatureType::Sloth, CreatureType::Beast])
    }
}

/// Bloom Hulk — {3}{G} 4/4 Plant Elemental. ETB proliferate.
pub fn bloom_hulk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Proliferate)],
        ..vanilla("Bloom Hulk", cost(&[generic(3), g()]), 4, 4, vec![CreatureType::Plant, CreatureType::Elemental])
    }
}

/// Centaur Nurturer — {3}{G} 2/4 Centaur Druid. ETB gain 3 life. {T}: Add one
/// mana of any color.
pub fn centaur_nurturer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(3) })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..vanilla("Centaur Nurturer", cost(&[generic(3), g()]), 2, 4, vec![CreatureType::Centaur, CreatureType::Druid])
    }
}

/// Challenger Troll — {4}{G} 6/5 Troll. Each creature you control with power 4
/// or greater can't be blocked by more than one creature.
pub fn challenger_troll() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your power-4+ creatures can't be blocked by more than one creature.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4))),
                keyword: Keyword::CantBeBlockedByMoreThanOne,
            },
        }],
        ..vanilla("Challenger Troll", cost(&[generic(4), g()]), 6, 5, vec![CreatureType::Troll])
    }
}

/// Evolution Sage — {2}{G} 3/2 Elf Druid. Landfall — whenever a land you control
/// enters, proliferate.
pub fn evolution_sage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Land,
            }),
            effect: Effect::Proliferate,
        }],
        ..vanilla("Evolution Sage", cost(&[generic(2), g()]), 3, 2, vec![CreatureType::Elf, CreatureType::Druid])
    }
}

/// Pollenbright Druid — {1}{G} 1/1 Elf Druid. ETB choose one — put a +1/+1
/// counter on target creature; or proliferate.
pub fn pollenbright_druid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseModesCast {
            modes: vec![
                Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::Proliferate,
            ],
            min: 1,
            max: 1,
            allow_repeats: false,
        })],
        ..vanilla("Pollenbright Druid", cost(&[generic(1), g()]), 1, 1, vec![CreatureType::Elf, CreatureType::Druid])
    }
}

/// Primordial Wurm — {4}{G}{G} 7/6 Wurm.
pub fn primordial_wurm() -> CardDefinition {
    vanilla("Primordial Wurm", cost(&[generic(4), g(), g()]), 7, 6, vec![CreatureType::Wurm])
}

/// Thundering Ceratok — {4}{G} 4/5 Rhino with trample. ETB other creatures you
/// control gain trample until end of turn.
pub fn thundering_ceratok() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: Selector::OtherCreaturesControlledByControllerOf(Box::new(Selector::This)),
            keyword: Keyword::Trample,
            duration: Duration::EndOfTurn,
        })],
        ..vanilla("Thundering Ceratok", cost(&[generic(4), g()]), 4, 5, vec![CreatureType::Rhino])
    }
}

/// Wardscale Crocodile — {4}{G} 5/3 Crocodile with hexproof.
pub fn wardscale_crocodile() -> CardDefinition {
    keyworded("Wardscale Crocodile", cost(&[generic(4), g()]), 5, 3, vec![CreatureType::Crocodile], vec![Keyword::Hexproof])
}

/// Kraul Stinger — {2}{G} 2/2 Insect Assassin with deathtouch.
pub fn kraul_stinger() -> CardDefinition {
    keyworded("Kraul Stinger", cost(&[generic(2), g()]), 2, 2, vec![CreatureType::Insect, CreatureType::Assassin], vec![Keyword::Deathtouch])
}

/// Kronch Wrangler — {1}{G} 2/1 Human Warrior with trample. Whenever a creature
/// you control with power 4 or greater enters, put a +1/+1 counter on this.
pub fn kronch_wrangler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::PowerAtLeast(4)),
            }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..vanilla("Kronch Wrangler", cost(&[generic(1), g()]), 2, 1, vec![CreatureType::Human, CreatureType::Warrior])
    }
}

/// Steady Aim — {1}{G} Instant. Untap target creature. It gets +1/+4 and gains
/// reach until end of turn.
pub fn steady_aim() -> CardDefinition {
    CardDefinition {
        name: "Steady Aim",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            Effect::PumpPT { what: Selector::Target(0), power: Value::ONE, toughness: Value::Const(4), duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Reach, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Forced Landing — {1}{G} Instant. Put target creature with flying on the
/// bottom of its owner's library.
pub fn forced_landing() -> CardDefinition {
    CardDefinition {
        name: "Forced Landing",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))), pos: LibraryPosition::Bottom },
        },
        ..Default::default()
    }
}

// ── Artifacts ─────────────────────────────────────────────────────────────────

/// Guild Globe — {2} Artifact. ETB draw a card. {2}, {T}, Sacrifice: Add two
/// mana of different colors.
pub fn guild_globe() -> CardDefinition {
    CardDefinition {
        name: "Guild Globe",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(draw(1))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::Const(2)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Iron Bully — {3} 1/1 Golem artifact with menace. ETB put a +1/+1 counter on
/// target creature.
pub fn iron_bully() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE })],
        ..CardDefinition {
            name: "Iron Bully",
            cost: cost(&[generic(3)]),
            subtypes: creatures(vec![CreatureType::Golem]),
            power: 1,
            toughness: 1,
            ..Default::default()
        }
    }
}

/// Mana Geode — {3} Artifact. ETB scry 1. {T}: Add one mana of any color.
pub fn mana_geode() -> CardDefinition {
    CardDefinition {
        name: "Mana Geode",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Prismite — {2} 2/1 Golem artifact. {2}: Add one mana of any color.
pub fn prismite() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..CardDefinition {
            name: "Prismite",
            cost: cost(&[generic(2)]),
            subtypes: creatures(vec![CreatureType::Golem]),
            power: 2,
            toughness: 1,
            ..Default::default()
        }
    }
}

/// God-Pharaoh's Statue — {6} Legendary Artifact. Opponents' spells cost {2}
/// more. At your end step, each opponent loses 1 life.
pub fn god_pharaohs_statue() -> CardDefinition {
    CardDefinition {
        name: "God-Pharaoh's Statue",
        cost: cost(&[generic(6)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Spells your opponents cast cost {2} more.",
            effect: StaticEffect::OpponentSpellsCostMore { filter: R::Any, amount: 2 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Gateway Plaza — Gate land. Enters tapped; sacrifice it unless you pay {1}.
/// {T}: Add one mana of any color.
pub fn gateway_plaza() -> CardDefinition {
    CardDefinition {
        name: "Gateway Plaza",
        cost: cost(&[]),
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![crate::card::LandType::Gate], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![etb(Effect::SacrificeSourceUnlessPay { cost: cost(&[generic(1)]) })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Multicolor ────────────────────────────────────────────────────────────────

/// Huatli's Raptor — {G}{W} 2/3 Dinosaur with vigilance. ETB proliferate.
pub fn huatlis_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::Proliferate)],
        ..vanilla("Huatli's Raptor", cost(&[g(), w()]), 2, 3, vec![CreatureType::Dinosaur])
    }
}

/// Elite Guardmage — {2}{W}{U} 2/3 Human Wizard with flying. ETB gain 3 life and
/// draw a card.
pub fn elite_guardmage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![Effect::GainLife { who: Selector::You, amount: Value::Const(3) }, draw(1)]))],
        ..vanilla("Elite Guardmage", cost(&[generic(2), w(), u()]), 2, 3, vec![CreatureType::Human, CreatureType::Wizard])
    }
}

/// Pledge of Unity — {1}{G}{W} Instant. Put a +1/+1 counter on each creature you
/// control. Gain 1 life for each creature you control.
pub fn pledge_of_unity() -> CardDefinition {
    CardDefinition {
        name: "Pledge of Unity",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::GainLife { who: Selector::You, amount: Value::count(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))) },
        ]),
        ..Default::default()
    }
}

/// Leyline Prowler — {1}{B}{G} 2/3 Nightmare Beast with deathtouch and lifelink.
/// {T}: Add one mana of any color.
pub fn leyline_prowler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..vanilla("Leyline Prowler", cost(&[generic(1), b(), g()]), 2, 3, vec![CreatureType::Nightmare, CreatureType::Beast])
    }
}

/// Dreadhorde Butcher — {B}{R} 1/1 Zombie Warrior with haste. Combat damage to a
/// player or planeswalker → +1/+1 counter. Dies → deals its power to any target.
pub fn dreadhorde_butcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
            on_dies(Effect::DealDamage { to: target_any(), amount: Value::PowerOf(Box::new(Selector::This)) }),
        ],
        ..vanilla("Dreadhorde Butcher", cost(&[b(), r()]), 1, 1, vec![CreatureType::Zombie, CreatureType::Warrior])
    }
}

/// Rubblebelt Rioters — {1}{R}{G} 0/4 Human Berserker with haste. Whenever it
/// attacks, it gets +X/+0, X = greatest power among creatures you control.
pub fn rubblebelt_rioters() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        })],
        ..vanilla("Rubblebelt Rioters", cost(&[generic(1), r(), g()]), 0, 4, vec![CreatureType::Human, CreatureType::Berserker])
    }
}

/// Tenth District Legionnaire — {R}{W} 2/2 Human Soldier with haste. Whenever you
/// cast a spell that targets it, put a +1/+1 counter on it, then scry 1.
pub fn tenth_district_legionnaire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::CastSpellTargetsSource),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            ]),
        }],
        ..vanilla("Tenth District Legionnaire", cost(&[r(), w()]), 2, 2, vec![CreatureType::Human, CreatureType::Soldier])
    }
}

/// Angrath's Rampage — {B}{R} Sorcery. Choose one — target player sacrifices an
/// artifact, a creature, or a planeswalker of their choice.
pub fn angraths_rampage() -> CardDefinition {
    let edict = |filter: R| Effect::Sacrifice { who: target_filtered(R::Player), count: Value::ONE, filter };
    CardDefinition {
        name: "Angrath's Rampage",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![edict(R::Artifact), edict(R::Creature), edict(R::Planeswalker)],
            min: 1,
            max: 1,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Ral's Outburst — {2}{U}{R} Instant. Deals 3 damage to any target. Look at the
/// top two cards of your library; one to hand, the other to your graveyard.
pub fn rals_outburst() -> CardDefinition {
    CardDefinition {
        name: "Ral's Outburst",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            deal(3, target_any()),
            Effect::LookTopKeepOneRestToGraveyard { count: Value::Const(2), who: Some(PlayerRef::You), exile_rest: false },
        ]),
        ..Default::default()
    }
}

/// Invade the City — {1}{U}{R} Sorcery. Amass Zombies X, X = instant and sorcery
/// cards in your graveyard.
pub fn invade_the_city() -> CardDefinition {
    CardDefinition {
        name: "Invade the City",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Amass {
            who: PlayerRef::You,
            count: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: instant_or_sorcery() },
            extra_type: Some(CreatureType::Zombie),
        },
        ..Default::default()
    }
}

/// Soul Diviner — {U}{B} 2/3 Zombie Wizard. {T}, Remove a counter from an
/// artifact, creature, land, or planeswalker you control: Draw a card.
pub fn soul_diviner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_among_filter: Some((
                None,
                1,
                (R::Artifact.or(R::Creature).or(R::Land).or(R::Planeswalker)).and(R::ControlledByYou),
            )),
            effect: draw(1),
            ..Default::default()
        }],
        ..vanilla("Soul Diviner", cost(&[u(), b()]), 2, 3, vec![CreatureType::Zombie, CreatureType::Wizard])
    }
}

// ── Planeswalkers (2026-07-23) ────────────────────────────────────────────────

fn walker(name: &'static str, mana: crate::mana::ManaCost, sub: PlaneswalkerSubtype, loyalty: u32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![sub], ..Default::default() },
        base_loyalty: loyalty,
        ..Default::default()
    }
}

/// 1/1 red Devil token with "When this token dies, it deals 1 damage to any
/// target."
fn devil_token() -> TokenDefinition {
    TokenDefinition {
        name: "Devil".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Devil], ..Default::default() },
        triggered_abilities: vec![on_dies(Effect::DealDamage { to: target_any(), amount: Value::ONE })],
        ..Default::default()
    }
}

/// Tibalt, Rakish Instigator — {2}{R} loyalty 5. Static: your opponents can't
/// gain life. −2: make a 1/1 red Devil (death-ping).
pub fn tibalt_rakish_instigator() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife { target: PlayerStaticTarget::EachOpponent },
        }],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -2,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: devil_token() },
            ..Default::default()
        }],
        ..walker("Tibalt, Rakish Instigator", cost(&[generic(2), r()]), PlaneswalkerSubtype::Tibalt, 5)
    }
}

/// Teyo, the Shieldmage — {2}{W} loyalty 5. Static: you have hexproof. −2: make
/// a 0/3 white Wall with defender.
pub fn teyo_the_shieldmage() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You have hexproof.",
            effect: StaticEffect::ControllerHasHexproof,
        }],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -2,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Wall".into(),
                    power: 0,
                    toughness: 3,
                    keywords: vec![Keyword::Defender],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..walker("Teyo, the Shieldmage", cost(&[generic(2), w()]), PlaneswalkerSubtype::Teyo, 5)
    }
}

/// Kasmina, Enigmatic Mentor — {3}{U} loyalty 5. Static: opponents' spells that
/// target your creature/planeswalker cost {2} more. −2: make a 2/2 Wizard, loot.
pub fn kasmina_enigmatic_mentor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells your opponents cast that target a creature or planeswalker you control cost {2} more.",
            effect: StaticEffect::TaxOpponentSpellsTargeting {
                target_filter: (R::Creature.or(R::Planeswalker)).and(R::ControlledByYou),
                amount: 2,
            },
        }],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -2,
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Wizard".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Blue],
                        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
                        ..Default::default()
                    },
                },
                draw(1),
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        ..walker("Kasmina, Enigmatic Mentor", cost(&[generic(3), u()]), PlaneswalkerSubtype::Kasmina, 5)
    }
}

/// The Wanderer — {3}{W} loyalty 5. Static: prevent all noncombat damage to you
/// and your permanents. −2: exile target creature with power 4 or greater.
pub fn the_wanderer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all noncombat damage that would be dealt to you and other permanents you control.",
            effect: StaticEffect::PreventNoncombatDamageToYouAndYourPermanents,
        }],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -2,
            effect: Effect::Exile { what: target_filtered(R::Creature.and(R::PowerAtLeast(4))) },
            ..Default::default()
        }],
        ..walker("The Wanderer", cost(&[generic(3), w()]), PlaneswalkerSubtype::Wanderer, 5)
    }
}

/// Ob Nixilis, the Hate-Twisted — {3}{B}{B} loyalty 5. Whenever an opponent
/// draws a card, deal 1 to them. −2: destroy target creature; its controller
/// draws two cards.
pub fn ob_nixilis_the_hate_twisted() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::Triggerer), amount: Value::ONE },
        }],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -2,
            effect: Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Creature) },
                Effect::Draw { who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))), amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..walker("Ob Nixilis, the Hate-Twisted", cost(&[generic(3), b(), b()]), PlaneswalkerSubtype::Nixilis, 5)
    }
}

/// Jaya, Venerated Firemage — {4}{R} loyalty 5. Static: another red source you
/// control deals +1 damage. −2: deal 2 to any target.
pub fn jaya_venerated_firemage() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If another red source you control would deal damage to a permanent or player, it deals that much damage plus 1 instead.",
            effect: StaticEffect::YourColorSourcesDealExtraDamage { color: Color::Red, amount: 1 },
        }],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -2,
            effect: deal(2, target_any()),
            ..Default::default()
        }],
        ..walker("Jaya, Venerated Firemage", cost(&[generic(4), r()]), PlaneswalkerSubtype::Jaya, 5)
    }
}

// ── Batch 3 (2026-07-23): legends, Bonds, and modal spells ────────────────────

/// "When this dies, you may put it into its owner's library third from the top."
/// (The God-Eternal recursion clause; the "or is exiled from the battlefield"
/// half is approximated as death-only.)
fn god_eternal_recur() -> TriggeredAbility {
    on_dies(Effect::MayDo {
        description: "Put it into its owner's library third from the top.".into(),
        body: Box::new(Effect::Move {
            what: Selector::TriggerSource,
            to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)), pos: LibraryPosition::FromTop(2) },
        }),
    })
}

/// God-Eternal Bontu — {3}{B}{B} 5/6 Zombie God with menace. ETB sacrifice any
/// number of other permanents, then draw that many. Death → library third.
pub fn god_eternal_bontu() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            etb(Effect::SacrificeAnyNumber {
                who: PlayerRef::You,
                filter: R::OtherThanSource.and(R::ControlledByYou),
                per_each: Box::new(draw(1)),
            }),
            god_eternal_recur(),
        ],
        ..vanilla("God-Eternal Bontu", cost(&[generic(3), b(), b()]), 5, 6, vec![CreatureType::Zombie, CreatureType::God])
    }
}

/// God-Eternal Oketra — {3}{W}{W} 3/6 Zombie God with double strike. Whenever
/// you cast a creature spell, make a 4/4 black Zombie Warrior with vigilance.
/// Death → library third.
pub fn god_eternal_oketra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Creature),
                }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Zombie Warrior".into(),
                        power: 4,
                        toughness: 4,
                        keywords: vec![Keyword::Vigilance],
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Black],
                        subtypes: creatures(vec![CreatureType::Zombie, CreatureType::Warrior]),
                        ..Default::default()
                    },
                },
            },
            god_eternal_recur(),
        ],
        ..vanilla("God-Eternal Oketra", cost(&[generic(3), w(), w()]), 3, 6, vec![CreatureType::Zombie, CreatureType::God])
    }
}

/// Fblthp, the Lost — {1}{U} 1/1 Homunculus. ETB draw a card. When it becomes
/// the target of a spell, shuffle it into its owner's library. (The "entered
/// from library → draw two" rider is approximated as one.)
pub fn fblthp_the_lost() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            etb(draw(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
                effect: Effect::ShuffleSelfIntoLibrary,
            },
        ],
        ..vanilla("Fblthp, the Lost", cost(&[generic(1), u()]), 1, 1, vec![CreatureType::Homunculus])
    }
}

/// Bond of Revival — {4}{B} Sorcery. Return target creature card from your
/// graveyard to the battlefield. It gains haste until your next turn.
pub fn bond_of_revival() -> CardDefinition {
    CardDefinition {
        name: "Bond of Revival",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Haste, duration: Duration::UntilNextTurn },
        ]),
        ..Default::default()
    }
}

/// Bond of Passion — {4}{R}{R} Sorcery. Gain control of target creature until
/// end of turn; untap it; it gains haste. Deals 2 damage to any other target.
pub fn bond_of_passion() -> CardDefinition {
    CardDefinition {
        name: "Bond of Passion",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::EndOfTurn },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Deathsprout — {1}{B}{B}{G} Instant. Destroy target creature. Search your
/// library for a basic land, put it onto the battlefield tapped, then shuffle.
pub fn deathsprout() -> CardDefinition {
    CardDefinition {
        name: "Deathsprout",
        cost: cost(&[generic(1), b(), b(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::Search {
                who: PlayerRef::You,
                filter: R::Land.and(R::HasSupertype(Supertype::Basic)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        ]),
        ..Default::default()
    }
}

/// Ravnica at War — {3}{W} Sorcery. Exile all multicolored permanents.
pub fn ravnica_at_war() -> CardDefinition {
    CardDefinition {
        name: "Ravnica at War",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Exile { what: Selector::EachPermanent(R::Multicolored) },
        ..Default::default()
    }
}

/// Wanderer's Strike — {4}{W} Sorcery. Exile target creature, then proliferate.
pub fn wanderers_strike() -> CardDefinition {
    CardDefinition {
        name: "Wanderer's Strike",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Creature) },
            Effect::Proliferate,
        ]),
        ..Default::default()
    }
}

/// Courage in Crisis — {2}{G} Sorcery. Put a +1/+1 counter on target creature,
/// then proliferate.
pub fn courage_in_crisis() -> CardDefinition {
    CardDefinition {
        name: "Courage in Crisis",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::Proliferate,
        ]),
        ..Default::default()
    }
}

/// Casualties of War — {2}{B}{B}{G}{G} Sorcery. Choose one or more — destroy
/// target artifact / creature / enchantment / land / planeswalker.
pub fn casualties_of_war() -> CardDefinition {
    let destroy = |filter: R| Effect::Destroy { what: target_filtered(filter) };
    CardDefinition {
        name: "Casualties of War",
        cost: cost(&[generic(2), b(), b(), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![
                destroy(R::Artifact),
                destroy(R::Creature),
                destroy(R::Enchantment),
                destroy(R::Land),
                destroy(R::Planeswalker),
            ],
            min: 1,
            max: 5,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Finale of Glory — {X}{W}{W} Sorcery. Create X 2/2 white Soldiers with
/// vigilance. If X ≥ 10, also create X 4/4 white Angels with flying, vigilance.
pub fn finale_of_glory() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: creatures(vec![CreatureType::Soldier]),
        ..Default::default()
    };
    let angel = TokenDefinition {
        name: "Angel".into(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: creatures(vec![CreatureType::Angel]),
        ..Default::default()
    };
    CardDefinition {
        name: "Finale of Glory",
        cost: cost(&[crate::mana::x(), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::XFromCost, definition: soldier },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(10)),
                then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::XFromCost, definition: angel }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ── Batch 4 (2026-07-23) ──────────────────────────────────────────────────────

/// Guildpact Informant — {2}{U} 1/1 Faerie Rogue with flying. Whenever it deals
/// combat damage to a player, proliferate.
pub fn guildpact_informant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Proliferate,
        }],
        ..vanilla("Guildpact Informant", cost(&[generic(2), u()]), 1, 1, vec![CreatureType::Faerie, CreatureType::Rogue])
    }
}

/// Teyo's Lightshield — {2}{W} 0/3 Illusion. ETB put a +1/+1 counter on target
/// creature you control.
pub fn teyos_lightshield() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..vanilla("Teyo's Lightshield", cost(&[generic(2), w()]), 0, 3, vec![CreatureType::Illusion])
    }
}

/// Roalesk, Apex Hybrid — {2}{G}{G}{U} 4/5 Human Mutant with flying and trample.
/// ETB two +1/+1 counters on another target creature you control. Dies →
/// proliferate twice.
pub fn roalesk_apex_hybrid() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            }),
            on_dies(Effect::Seq(vec![Effect::Proliferate, Effect::Proliferate])),
        ],
        ..vanilla("Roalesk, Apex Hybrid", cost(&[generic(2), g(), g(), u()]), 4, 5, vec![CreatureType::Human, CreatureType::Mutant])
    }
}

/// Jace's Projection — {2}{U}{U} 2/2 Wizard Illusion. Whenever you draw a card,
/// put a +1/+1 counter on it. {3}{U}: put a loyalty counter on target Jace.
pub fn jaces_projection() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::AddCounter {
                what: target_filtered(R::HasPlaneswalkerType(PlaneswalkerSubtype::Jace)),
                kind: CounterType::Loyalty,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..vanilla("Jace's Projection", cost(&[generic(2), u(), u()]), 2, 2, vec![CreatureType::Wizard, CreatureType::Illusion])
    }
}

// ── Batch 5 (2026-07-23) ──────────────────────────────────────────────────────

/// Silent Submersible — {U}{U} 2/3 Vehicle, Crew 2. Whenever it deals combat
/// damage to a player, draw a card.
pub fn silent_submersible() -> CardDefinition {
    CardDefinition {
        name: "Silent Submersible",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: draw(1),
        }],
        ..Default::default()
    }
}

/// Storrev, Devkarin Lich — {1}{B}{B}{G} 5/4 Zombie Elf Wizard with trample.
/// Whenever it deals combat damage to a player, return a creature or
/// planeswalker card from your graveyard to your hand. (The "wasn't put there
/// this combat" rider is approximated.)
pub fn storrev_devkarin_lich() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered((R::Creature.or(R::Planeswalker)).and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..vanilla("Storrev, Devkarin Lich", cost(&[generic(1), b(), b(), g()]), 5, 4, vec![CreatureType::Zombie, CreatureType::Elf, CreatureType::Wizard])
    }
}

// ── Batch 8 (2026-07-23): search / discard commons ────────────────────────────

/// Gideon's Battle Cry — {2}{W}{W} Sorcery. Put a +1/+1 counter on each creature
/// you control; you may tutor a card named Gideon, the Oathsworn to hand.
pub fn gideons_battle_cry() -> CardDefinition {
    CardDefinition {
        name: "Gideon's Battle Cry",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::SearchLibraryOrGraveyard {
                who: PlayerRef::You,
                filter: R::HasName("Gideon, the Oathsworn".into()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Desperate Lunge — {1}{W} Instant. Target creature gets +2/+2 and gains flying
/// until end of turn. You gain 2 life.
pub fn desperate_lunge() -> CardDefinition {
    CardDefinition {
        name: "Desperate Lunge",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Davriel's Shadowfugue — {3}{B} Sorcery. Target player discards two cards and
/// loses 2 life.
pub fn davriels_shadowfugue() -> CardDefinition {
    CardDefinition {
        name: "Davriel's Shadowfugue",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(2), random: false },
            Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Ignite the Beacon — {4}{W} Instant. Search your library for up to two
/// planeswalker cards, reveal them, put them into your hand, then shuffle.
pub fn ignite_the_beacon() -> CardDefinition {
    CardDefinition {
        name: "Ignite the Beacon",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::Planeswalker,
            to: ZoneDest::Hand(PlayerRef::You),
            count: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Nissa's Triumph — {G}{G} Sorcery. Search for up to two basic Forests (up to
/// three lands instead if you control a Nissa planeswalker), reveal, to hand.
pub fn nissas_triumph() -> CardDefinition {
    CardDefinition {
        name: "Nissa's Triumph",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::count(Selector::EachPermanent(
                    R::HasPlaneswalkerType(PlaneswalkerSubtype::Nissa).and(R::ControlledByYou),
                )),
                Value::Const(1),
            ),
            then: Box::new(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::Land,
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(3),
            }),
            else_: Box::new(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand.and(R::HasLandType(crate::card::LandType::Forest)),
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(2),
            }),
        },
        ..Default::default()
    }
}

// ── Batch 7 (2026-07-23): simple commons/uncommons on existing primitives ─────

/// Ugin's Conjurant — {X} 0/0 Spirit Monk. Enters with X +1/+1 counters; damage
/// to it while it has a +1/+1 counter is prevented by removing that many.
pub fn ugins_conjurant() -> CardDefinition {
    CardDefinition {
        name: "Ugin's Conjurant",
        cost: cost(&[x()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Spirit, CreatureType::Monk]),
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to this while it has a +1/+1 counter, prevent it and remove that many +1/+1 counters.",
            effect: StaticEffect::PreventDamageByRemovingCounters { kind: CounterType::PlusOnePlusOne },
        }],
        ..Default::default()
    }
}

/// Arlinn's Wolf — {2}{G} 3/2 Wolf. Can't be blocked by power-2-or-less creatures.
pub fn arlinns_wolf() -> CardDefinition {
    keyworded("Arlinn's Wolf", cost(&[generic(2), g()]), 3, 2, vec![CreatureType::Wolf], vec![Keyword::CantBeBlockedByPowerAtMost(2)])
}

/// Domri's Ambush — {R}{G} Sorcery. Put a +1/+1 counter on target creature you
/// control, then it deals damage equal to its power to a creature or
/// planeswalker you don't control.
pub fn domris_ambush() -> CardDefinition {
    CardDefinition {
        name: "Domri's Ambush",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::DealDamageEqualToPower {
                source: Selector::Target(0),
                target: Selector::TargetFiltered { slot: 1, filter: (R::Creature.or(R::Planeswalker)).and(R::ControlledByOpponent) },
            },
        ]),
        ..Default::default()
    }
}

/// Spark Harvest — {B} Sorcery. Additional cost: sacrifice a creature or pay
/// {3}{B}. Destroy target creature or planeswalker. (The colored half of the
/// alt-cost is approximated as {4} generic.)
pub fn spark_harvest() -> CardDefinition {
    CardDefinition {
        name: "Spark Harvest",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificeOrPay { filter: R::Creature, pay: 4 }],
        effect: Effect::Destroy { what: target_filtered(R::Creature.or(R::Planeswalker)) },
        ..Default::default()
    }
}

/// Toll of the Invasion — {2}{B} Sorcery. Target opponent reveals their hand;
/// you choose a nonland card, they discard it. Then amass Zombies 1.
pub fn toll_of_the_invasion() -> CardDefinition {
    CardDefinition {
        name: "Toll of the Invasion",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: R::Nonland,
            },
            Effect::Amass { who: PlayerRef::You, count: Value::ONE, extra_type: Some(CreatureType::Zombie) },
        ]),
        ..Default::default()
    }
}

/// Eternal Taskmaster — {1}{B} 2/3 Zombie. Enters tapped. When it attacks, you
/// may pay {2}{B}; if you do, return target creature card from your graveyard.
pub fn eternal_taskmaster() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![on_attack(Effect::MayPay {
            description: "Pay {2}{B} to return a creature card from your graveyard to your hand?".into(),
            mana_cost: cost(&[generic(2), b()]),
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: None,
        })],
        ..vanilla("Eternal Taskmaster", cost(&[generic(1), b()]), 2, 3, vec![CreatureType::Zombie])
    }
}

/// Living Twister — {R}{R}{G} 2/5 Elemental. {1}{R}, discard a land card: deal
/// 2 damage to any target. {G}: return a tapped land you control to hand.
pub fn living_twister() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                discard_cost: Some((R::Land, 1)),
                effect: deal(2, target_any()),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::Move {
                    what: target_filtered(R::Land.and(R::ControlledByYou).and(R::Tapped)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..vanilla("Living Twister", cost(&[r(), r(), g()]), 2, 5, vec![CreatureType::Elemental])
    }
}

/// Lazotep Plating — {1}{U} Instant. Amass Zombies 1, then you and permanents
/// you control gain hexproof until end of turn. (Modeled as hexproof from all
/// five colors; colorless sources are an approximation.)
pub fn lazotep_plating() -> CardDefinition {
    CardDefinition {
        name: "Lazotep Plating",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Amass { who: PlayerRef::You, count: Value::ONE, extra_type: Some(CreatureType::Zombie) },
            Effect::GrantHexproofFromColorThisTurn {
                who: Selector::You,
                colors: vec![Color::White, Color::Blue, Color::Black, Color::Red, Color::Green],
            },
        ]),
        ..Default::default()
    }
}

// ── Batch 6 (2026-07-23): planeswalker-matters + counter payoffs ──────────────

/// Bioessence Hydra — {3}{G}{U} 4/4 Hydra Mutant with trample. Enters with a
/// +1/+1 counter per loyalty counter on your planeswalkers; whenever loyalty
/// counters are put on planeswalkers you control, grows by that many.
pub fn bioessence_hydra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CountersOn {
                what: Box::new(Selector::EachPermanent(R::Planeswalker.and(R::ControlledByYou))),
                kind: CounterType::Loyalty,
            },
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::Loyalty), EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Planeswalker,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..vanilla("Bioessence Hydra", cost(&[generic(3), g(), u()]), 4, 4, vec![CreatureType::Hydra, CreatureType::Mutant])
    }
}

/// Charmed Stray — {W} 1/1 Cat with lifelink. ETB puts a +1/+1 counter on each
/// other creature you control named Charmed Stray.
pub fn charmed_stray() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::HasName("Charmed Stray".into())).and(R::OtherThanSource),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..vanilla("Charmed Stray", cost(&[w()]), 1, 1, vec![CreatureType::Cat])
    }
}

/// Band Together — {2}{G} Instant. Up to two target creatures you control each
/// deal damage equal to their power to another target creature. (The "another"
/// exclusion is approximated as a plain creature slot.)
pub fn band_together() -> CardDefinition {
    let mine = R::Creature.and(R::ControlledByYou);
    CardDefinition {
        name: "Band Together",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::DealDamageEqualToPower {
                    source: Selector::TargetFiltered { slot: 1, filter: mine.clone() },
                    target: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                },
                Effect::DealDamageEqualToPower {
                    source: Selector::TargetFiltered { slot: 2, filter: mine },
                    target: Selector::Target(0),
                },
            ])),
        },
        ..Default::default()
    }
}

/// Mowu, Loyal Companion — {3}{G} 3/3 Legendary Dog with vigilance and trample.
/// If one or more +1/+1 counters would be put on it, that many plus one are put
/// on it instead.
pub fn mowu_loyal_companion() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on this, that many plus one are put on it instead.",
            effect: StaticEffect::ExtraPlusOneCounterOnSelf,
        }],
        ..vanilla("Mowu, Loyal Companion", cost(&[generic(3), g()]), 3, 3, vec![CreatureType::Dog])
    }
}

/// Vivien's Grizzly — {2}{G} 2/3 Bear Spirit. {3}{G}: Look at the top card of
/// your library; if it's a creature or planeswalker card you may reveal it and
/// put it into your hand, otherwise it goes to the bottom.
pub fn viviens_grizzly() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::LookTopMayRevealMatchToHandElseBottom { filter: R::Creature.or(R::Planeswalker) },
            ..Default::default()
        }],
        ..vanilla("Vivien's Grizzly", cost(&[generic(2), g()]), 2, 3, vec![CreatureType::Bear, CreatureType::Spirit])
    }
}

/// Command the Dreadhorde — {4}{B}{B} Sorcery. Choose any number of creature
/// and/or planeswalker cards in graveyards; take damage equal to their total
/// mana value, then put them onto the battlefield under your control.
pub fn command_the_dreadhorde() -> CardDefinition {
    CardDefinition {
        name: "Command the Dreadhorde",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CommandTheDreadhorde,
        ..Default::default()
    }
}

/// Kaya's Ghostform — {B} Aura. Enchant creature or planeswalker you control.
/// When the enchanted permanent dies or is exiled, return that card to the
/// battlefield under your control.
pub fn kayas_ghostform() -> CardDefinition {
    let recur = || Effect::Move {
        what: Selector::TriggerSource,
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    };
    CardDefinition {
        name: "Kaya's Ghostform",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered((R::Creature.or(R::Planeswalker)).and(R::ControlledByYou)),
        },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
                effect: recur(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardExiled, EventScope::EnchantedBySource),
                effect: recur(),
            },
        ],
        ..Default::default()
    }
}

// ── Batch 9 (2026-07-24): the hybrid planeswalker cycle, God-Eternal Rhonas,
//    and modal/utility spells ───────────────────────────────────────────────

/// "Creatures you control have [keyword]." — the hybrid-cycle static anthem.
fn your_creatures_have(keyword: Keyword, description: &'static str) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            keyword,
        },
    }
}

/// Angrath, Captain of Chaos — {2}{B/R}{B/R} loyalty 5. Static: creatures you
/// control have menace. −2: Amass Zombies 2.
pub fn angrath_captain_of_chaos() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![your_creatures_have(Keyword::Menace, "Creatures you control have menace.")],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -2,
            effect: Effect::Amass { who: PlayerRef::You, count: Value::Const(2), extra_type: Some(CreatureType::Zombie) },
            ..Default::default()
        }],
        ..walker("Angrath, Captain of Chaos", cost(&[generic(2), hybrid(Color::Black, Color::Red), hybrid(Color::Black, Color::Red)]), PlaneswalkerSubtype::Angrath, 5)
    }
}

/// Huatli, the Sun's Heart — {2}{G/W} loyalty 7. Static: your creatures assign
/// combat damage by toughness. −3: gain life = greatest toughness you control.
pub fn huatli_the_suns_heart() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![your_creatures_have(
            Keyword::AssignsCombatDamageByToughness,
            "Each creature you control assigns combat damage equal to its toughness rather than its power.",
        )],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -3,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::GreatestToughnessYouControl)),
            },
            ..Default::default()
        }],
        ..walker("Huatli, the Sun's Heart", cost(&[generic(2), hybrid(Color::Green, Color::White)]), PlaneswalkerSubtype::Huatli, 7)
    }
}

/// Kiora, Behemoth Beckoner — {2}{G/U} loyalty 7. Whenever a power-4+ creature
/// you control enters, draw. −1: untap target permanent.
pub fn kiora_behemoth_beckoner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::PowerAtLeast(4)),
            }),
            effect: draw(1),
        }],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -1,
            effect: Effect::Untap { what: target_filtered(R::Permanent), up_to: None },
            ..Default::default()
        }],
        ..walker("Kiora, Behemoth Beckoner", cost(&[generic(2), hybrid(Color::Green, Color::Blue)]), PlaneswalkerSubtype::Kiora, 7)
    }
}

/// Samut, Tyrant Smasher — {2}{R/G}{R/G} loyalty 5. Static: creatures you
/// control have haste. −1: target creature gets +2/+1 and gains haste; scry 1.
pub fn samut_tyrant_smasher() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![your_creatures_have(Keyword::Haste, "Creatures you control have haste.")],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -1,
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: target_filtered(R::Creature), power: Value::Const(2), toughness: Value::Const(1), duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..walker("Samut, Tyrant Smasher", cost(&[generic(2), hybrid(Color::Red, Color::Green), hybrid(Color::Red, Color::Green)]), PlaneswalkerSubtype::Samut, 5)
    }
}

/// God-Eternal Rhonas — {3}{G}{G} 5/5 Zombie God with deathtouch. ETB double
/// the power of each other creature you control and grant vigilance (EOT).
/// Death → library third.
pub fn god_eternal_rhonas() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::DoublePower {
                    what: Selector::EachPermanent(R::OtherThanSource.and(R::Creature).and(R::ControlledByYou)),
                    times: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::OtherThanSource.and(R::Creature).and(R::ControlledByYou)),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
            ])),
            god_eternal_recur(),
        ],
        ..vanilla("God-Eternal Rhonas", cost(&[generic(3), g(), g()]), 5, 5, vec![CreatureType::Zombie, CreatureType::God])
    }
}

/// Tolsimir, Friend to Wolves — {2}{G}{G}{W} 3/3 legendary Elf Scout. ETB make
/// Voja, a legendary 3/3 GW Wolf. Whenever a Wolf you control enters, gain 3
/// life and it fights up to one target creature you don't control.
pub fn tolsimir_friend_to_wolves() -> CardDefinition {
    let voja = TokenDefinition {
        name: "Voja, Friend to Elves".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::White],
        supertypes: vec![Supertype::Legendary],
        subtypes: creatures(vec![CreatureType::Wolf]),
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: voja }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Wolf),
                }),
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                    Effect::OptionalTargets {
                        min: 0,
                        body: Box::new(Effect::Fight {
                            attacker: Selector::TriggerSource,
                            defender: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByOpponent) },
                        }),
                    },
                ]),
            },
        ],
        ..vanilla("Tolsimir, Friend to Wolves", cost(&[generic(2), g(), g(), w()]), 3, 3, vec![CreatureType::Elf, CreatureType::Scout])
    }
}

/// Enter the God-Eternals — {2}{U}{U}{B} Sorcery. Deal 4 to target creature and
/// gain that much life; target player mills four; Amass Zombies 4.
pub fn enter_the_god_eternals() -> CardDefinition {
    CardDefinition {
        name: "Enter the God-Eternals",
        cost: cost(&[generic(2), u(), u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            deal(4, target_filtered(R::Creature)),
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            Effect::Mill { who: Selector::Player(PlayerRef::Target(1)), amount: Value::Const(4) },
            Effect::Amass { who: PlayerRef::You, count: Value::Const(4), extra_type: Some(CreatureType::Zombie) },
        ]),
        ..Default::default()
    }
}

/// Role Reversal — {U}{U}{R} Sorcery. Exchange control of two target permanents.
/// (The printed "that share a permanent type" restriction is not enforced at
/// cast time.)
pub fn role_reversal() -> CardDefinition {
    CardDefinition {
        name: "Role Reversal",
        cost: cost(&[u(), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ExchangeControl {
            a: Selector::TargetFiltered { slot: 0, filter: R::Permanent },
            b: Selector::TargetFiltered { slot: 1, filter: R::Permanent },
        },
        ..Default::default()
    }
}

/// Heartwarming Redemption — {2}{R}{W} Instant. Discard your hand, draw that
/// many cards plus one, then gain life equal to the number of cards in hand.
pub fn heartwarming_redemption() -> CardDefinition {
    CardDefinition {
        name: "Heartwarming Redemption",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DiscardHandDrawThatMany { who: Selector::You },
            draw(1),
            Effect::GainLife { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You) },
        ]),
        ..Default::default()
    }
}

/// Vivien's Arkbow — {1}{G} legendary Artifact. {X}, {T}, Discard a card: look
/// at the top X cards; you may put a creature with mana value X or less onto the
/// battlefield; bottom the rest in a random order.
pub fn viviens_arkbow() -> CardDefinition {
    CardDefinition {
        name: "Vivien's Arkbow",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[x()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::LookTopPutMatchingOntoBattlefield {
                count: Value::XFromCost,
                filter: R::Creature.and(R::ManaValueAtMostXFromCost),
                then: None,
                max: Some(1),
                tapped: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Oath of Kaya — {1}{W}{B} legendary Enchantment. ETB deal 3 to any target and
/// gain 3. Whenever an opponent attacks a planeswalker you control, deal 2 to
/// that player and gain 2. (The attack trigger over-fires on attacks against
/// you, mirroring `ControllerAttackedByOpponent`.)
pub fn oath_of_kaya() -> CardDefinition {
    CardDefinition {
        name: "Oath of Kaya",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                deal(3, target_any()),
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
                effect: Effect::Seq(vec![
                    Effect::DealDamage { to: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(2) },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Interplanar Beacon — Land. Whenever you cast a planeswalker spell, gain 1
/// life. {T}: Add {C}. {1}, {T}: Add two mana of any colors, spendable only on
/// planeswalker spells. (The "two *different* colors" rider is not enforced.)
pub fn interplanar_beacon() -> CardDefinition {
    CardDefinition {
        name: "Interplanar Beacon",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCardType(CardType::Planeswalker),
            }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyColors(Value::Const(2))),
                        crate::mana::SpendRestriction::PlaneswalkerSpellsOnly,
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Emergence Zone — Land. {T}: Add {C}. {1}, {T}, Sacrifice: you may cast
/// spells this turn as though they had flash.
pub fn emergence_zone() -> CardDefinition {
    CardDefinition {
        name: "Emergence Zone",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::GrantSorceriesAsFlash { who: PlayerRef::You },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Mobilized District — Land. {T}: Add {C}. {4}: becomes a 3/3 Citizen with
/// vigilance until end of turn (still a land). (The per-legendary cost reduction
/// is dropped.)
pub fn mobilized_district() -> CardDefinition {
    CardDefinition {
        name: "Mobilized District",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![CreatureType::Citizen],
                    keywords: vec![Keyword::Vigilance],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rescuer Sphinx — {2}{U}{U} 3/2 flying Sphinx. As it enters, you may return a
/// nonland permanent you control to its owner's hand; if you do, it gets a
/// +1/+1 counter. (Modeled as a reflexive ETB rather than an as-enters
/// replacement, so the counter lands just after it enters.)
pub fn rescuer_sphinx() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return a nonland permanent you control to its owner's hand?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Nonland.and(R::ControlledByYou).and(R::OtherThanSource)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ])),
        })],
        ..vanilla("Rescuer Sphinx", cost(&[generic(2), u(), u()]), 3, 2, vec![CreatureType::Sphinx])
    }
}

/// Devouring Hellion — {2}{R} 2/2 Hellion. Devour-style: as it enters, you may
/// sacrifice any number of creatures and/or planeswalkers; it enters with twice
/// that many +1/+1 counters (`SacrificeAnyNumber` over creatures/PWs, ×2).
pub fn devouring_hellion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SacrificeAnyNumber {
            who: PlayerRef::You,
            filter: (R::Creature.or(R::Planeswalker)).and(R::OtherThanSource),
            per_each: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            }),
        })],
        ..vanilla("Devouring Hellion", cost(&[generic(2), r()]), 2, 2, vec![CreatureType::Hellion])
    }
}

/// Nahiri, Storm of Stone — {2}{R/W}{R/W} loyalty 6. Static: during your turn,
/// creatures you control have first strike and your equip abilities cost {1}
/// less. −X: Nahiri deals X damage to target tapped creature.
pub fn nahiri_storm_of_stone() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "During your turn, creatures you control have first strike.",
                effect: StaticEffect::WhileYourTurn {
                    inner: Box::new(StaticEffect::GrantKeyword {
                        applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                        keyword: Keyword::FirstStrike,
                    }),
                },
            },
            StaticAbility {
                description: "During your turn, equip abilities you activate cost {1} less.",
                effect: StaticEffect::WhileYourTurn { inner: Box::new(StaticEffect::EquipCostReduction { amount: 1 }) },
            },
        ],
        loyalty_abilities: vec![LoyaltyAbility {
            x_cost: true,
            effect: Effect::DealDamage { to: target_filtered(R::Creature.and(R::Tapped)), amount: Value::XFromCost },
            ..Default::default()
        }],
        ..walker("Nahiri, Storm of Stone", cost(&[generic(2), hybrid(Color::Red, Color::White), hybrid(Color::Red, Color::White)]), PlaneswalkerSubtype::Nahiri, 6)
    }
}

/// Mizzium Tank — {1}{R}{R} 3/2 Vehicle with trample, Crew 1. Whenever you cast
/// a noncreature spell, it becomes an artifact creature and gets +1/+1 (EOT).
pub fn mizzium_tank() -> CardDefinition {
    CardDefinition {
        name: "Mizzium Tank",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample, Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(cast_is_noncreature()),
            effect: Effect::Seq(vec![
                Effect::AnimateAsCreature { what: Selector::This, duration: Duration::EndOfTurn },
                Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
            ]),
        }],
        ..Default::default()
    }
}

/// Narset's Reversal — {U}{U} Instant. Copy target instant or sorcery spell
/// (may choose new targets), then return it to its owner's hand. (Modeled as a
/// copy + Remand-style return; genuinely uncounterable spells are still bounced.)
pub fn narsets_reversal() -> CardDefinition {
    CardDefinition {
        name: "Narset's Reversal",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CopySpellMayChooseTargets {
                what: target_filtered(R::IsSpellOnStack.and(instant_or_sorcery())),
                count: Value::ONE,
            },
            Effect::CounterSpellToZone { what: Selector::Target(0), zone: crate::effect::CounteredSpellZone::OwnerHand },
        ]),
        ..Default::default()
    }
}

/// Storm the Citadel — {4}{G} Sorcery. Until end of turn, creatures you control
/// get +2/+2 and gain "whenever this creature deals combat damage to a player or
/// planeswalker, destroy target artifact or enchantment an opponent controls."
/// (The "defending player controls" restriction is widened to any opponent.)
pub fn storm_the_citadel() -> CardDefinition {
    let yours = Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Storm the Citadel",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT { what: yours.clone(), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
            Effect::GrantTriggeredAbility {
                what: yours,
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: Effect::Destroy { what: target_filtered((R::Artifact.or(R::Enchantment)).and(R::ControlledByOpponent)) },
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Ashiok, Dream Render — {1}{U/B}{U/B} loyalty 5. Static: opponents' spells and
/// abilities can't cause their controller to search their library. −1: target
/// player mills four, then exile each opponent's graveyard.
pub fn ashiok_dream_render() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells and abilities your opponents control can't cause their controller to search their library.",
            effect: StaticEffect::OpponentsCantSearchLibraries,
        }],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -1,
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(4) },
                Effect::ExileAllGraveyards { filter: None, opponents_only: true },
            ]),
            ..Default::default()
        }],
        ..walker("Ashiok, Dream Render", cost(&[generic(1), hybrid(Color::Blue, Color::Black), hybrid(Color::Blue, Color::Black)]), PlaneswalkerSubtype::Ashiok, 5)
    }
}

/// Parhelion II — {6}{W}{W} legendary Vehicle 5/5 with flying, first strike,
/// vigilance. Crew 4. When it attacks, make two 4/4 flying-vigilance Angels
/// attacking (they stay).
pub fn parhelion_ii() -> CardDefinition {
    CardDefinition {
        name: "Parhelion II",
        cost: cost(&[generic(6), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Vigilance, Keyword::Crew(4)],
        triggered_abilities: vec![on_attack(Effect::CreateTokenAttacking {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: TokenDefinition {
                name: "Angel".into(),
                power: 4,
                toughness: 4,
                keywords: vec![Keyword::Flying, Keyword::Vigilance],
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: creatures(vec![CreatureType::Angel]),
                ..Default::default()
            },
            cleanup: crate::effect::AttackingTokenCleanup::None,
        })],
        ..Default::default()
    }
}

/// Dreadhorde Invasion — {1}{B} Enchantment. Upkeep: lose 1 life and Amass
/// Zombies 1. Whenever a Zombie token you control with power 6+ attacks, it
/// gains lifelink until end of turn.
pub fn dreadhorde_invasion() -> CardDefinition {
    CardDefinition {
        name: "Dreadhorde Invasion",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                    Effect::Amass { who: PlayerRef::You, count: Value::ONE, extra_type: Some(CreatureType::Zombie) },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsToken.and(R::HasCreatureType(CreatureType::Zombie)).and(R::PowerAtLeast(6)),
                }),
                effect: Effect::GrantKeyword { what: Selector::TriggerSource, keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
            },
        ],
        ..Default::default()
    }
}

/// Kaya, Bane of the Dead — {3}{W/B}{W/B}{W/B} loyalty 7. Static: you may target
/// opponents' hexproof permanents and players as though they lacked hexproof.
/// −3: exile target creature.
pub fn kaya_bane_of_the_dead() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your opponents and permanents your opponents control with hexproof can be the targets of spells and abilities you control as though they didn't have hexproof.",
            effect: StaticEffect::IgnoreOpponentsHexproof,
        }],
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -3,
            effect: Effect::Exile { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..walker("Kaya, Bane of the Dead", cost(&[generic(3), hybrid(Color::White, Color::Black), hybrid(Color::White, Color::Black), hybrid(Color::White, Color::Black)]), PlaneswalkerSubtype::Kaya, 7)
    }
}

/// Tamiyo's Epiphany — {3}{U} Sorcery. Scry 4, then draw two cards.
pub fn tamiyos_epiphany() -> CardDefinition {
    CardDefinition {
        name: "Tamiyo's Epiphany",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(4) },
            draw(2),
        ]),
        ..Default::default()
    }
}

/// Commence the Endgame — {4}{U}{U} Instant. Can't be countered. Draw two, then
/// Amass Zombies X where X = cards in your hand.
pub fn commence_the_endgame() -> CardDefinition {
    CardDefinition {
        name: "Commence the Endgame",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Seq(vec![
            draw(2),
            Effect::Amass { who: PlayerRef::You, count: Value::HandSizeOf(PlayerRef::You), extra_type: Some(CreatureType::Zombie) },
        ]),
        ..Default::default()
    }
}

/// Teferi's Time Twist — {1}{U} Instant. Exile target permanent you control;
/// return it at the next end step with an extra +1/+1 counter if it's a creature.
pub fn teferis_time_twist() -> CardDefinition {
    CardDefinition {
        name: "Teferi's Time Twist",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ExileReturnNextEndStep { what: target_filtered(R::Permanent.and(R::ControlledByYou)) },
        ..Default::default()
    }
}

/// Planewide Celebration — {5}{G}{G} Sorcery. Choose four (repeats allowed):
/// make an all-color 2/2 Citizen; return a permanent card from your graveyard
/// to hand; proliferate; gain 4 life.
pub fn planewide_celebration() -> CardDefinition {
    let citizen = TokenDefinition {
        name: "Citizen".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Blue, Color::Black, Color::Red, Color::Green],
        subtypes: creatures(vec![CreatureType::Citizen]),
        ..Default::default()
    };
    CardDefinition {
        name: "Planewide Celebration",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: citizen },
                Effect::Move { what: target_filtered(R::PermanentCard.and(R::InYourGraveyard)), to: ZoneDest::Hand(PlayerRef::You) },
                Effect::Proliferate,
                Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            ],
            min: 4,
            max: 4,
            allow_repeats: true,
        },
        ..Default::default()
    }
}
