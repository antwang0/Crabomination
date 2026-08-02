//! BFZ creatures — landfall, Rally allies, Eldrazi Drones/Processors.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes, Value,
};
use crate::effect::shortcut::{
    each_your_creature, etb, ingest, landfall, on_attack, rally, rally_grant, target_any,
    target_filtered,
};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{b, cost, g, generic, r, u, w, x};
use crabomination_base::tokens::eldrazi_scion_token;

fn creature(
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

/// A landfall creature that pumps itself `(dp, dt)` until end of turn.
fn landfall_self_pump(
    base: CardDefinition,
    (dp, dt): (i32, i32),
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        keywords,
        triggered_abilities: vec![landfall(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(dp),
            toughness: Value::Const(dt),
            duration: Duration::EndOfTurn,
        })],
        ..base
    }
}

// ── Vanilla / keyword-only ──────────────────────────────────────────────────

/// Broodhunter Wurm — {3}{G} 4/3 Wurm.
pub fn broodhunter_wurm() -> CardDefinition {
    creature("Broodhunter Wurm", cost(&[generic(3), g()]), vec![CreatureType::Wurm], 4, 3)
}

/// Cloud Manta — {3}{U} 3/2 Fish with flying.
pub fn cloud_manta() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature("Cloud Manta", cost(&[generic(3), u()]), vec![CreatureType::Fish], 3, 2)
    }
}

/// Kitesail Scout — {W} 1/1 Kor Scout with flying.
pub fn kitesail_scout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature(
            "Kitesail Scout",
            cost(&[w()]),
            vec![CreatureType::Kor, CreatureType::Scout],
            1,
            1,
        )
    }
}

/// Shadow Glider — {2}{W} 2/2 Kor Soldier with flying.
pub fn shadow_glider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature(
            "Shadow Glider",
            cost(&[generic(2), w()]),
            vec![CreatureType::Kor, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Shatterskull Recruit — {3}{R}{R} 4/4 Giant Warrior Ally with menace.
pub fn shatterskull_recruit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        ..creature(
            "Shatterskull Recruit",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Giant, CreatureType::Warrior, CreatureType::Ally],
            4,
            4,
        )
    }
}

// ── Landfall ────────────────────────────────────────────────────────────────

/// Scythe Leopard — {G} 1/1 Cat. Landfall: +1/+1 until end of turn.
pub fn scythe_leopard() -> CardDefinition {
    landfall_self_pump(
        creature("Scythe Leopard", cost(&[g()]), vec![CreatureType::Cat], 1, 1),
        (1, 1),
        vec![],
    )
}

/// Makindi Sliderunner — {1}{R} 2/1 Beast with trample. Landfall: +1/+1 EOT.
pub fn makindi_sliderunner() -> CardDefinition {
    landfall_self_pump(
        creature("Makindi Sliderunner", cost(&[generic(1), r()]), vec![CreatureType::Beast], 2, 1),
        (1, 1),
        vec![Keyword::Trample],
    )
}

/// Valakut Predator — {2}{R} 2/2 Elemental. Landfall: +2/+2 until end of turn.
pub fn valakut_predator() -> CardDefinition {
    landfall_self_pump(
        creature("Valakut Predator", cost(&[generic(2), r()]), vec![CreatureType::Elemental], 2, 2),
        (2, 2),
        vec![],
    )
}

/// Wave-Wing Elemental — {5}{U} 3/4 flier. Landfall: +2/+2 until end of turn.
pub fn wave_wing_elemental() -> CardDefinition {
    landfall_self_pump(
        creature("Wave-Wing Elemental", cost(&[generic(5), u()]), vec![CreatureType::Elemental], 3, 4),
        (2, 2),
        vec![Keyword::Flying],
    )
}

/// Geyserfield Stalker — {4}{B} 3/2 Elemental with menace. Landfall: +2/+2 EOT.
pub fn geyserfield_stalker() -> CardDefinition {
    landfall_self_pump(
        creature("Geyserfield Stalker", cost(&[generic(4), b()]), vec![CreatureType::Elemental], 3, 2),
        (2, 2),
        vec![Keyword::Menace],
    )
}

/// Belligerent Whiptail — {3}{R} 4/2 Wurm. Landfall: gains first strike EOT.
pub fn belligerent_whiptail() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::FirstStrike,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Belligerent Whiptail", cost(&[generic(3), r()]), vec![CreatureType::Wurm], 4, 2)
    }
}

/// Jaddi Offshoot — {G} 0/3 Plant with defender. Landfall: you gain 1 life.
pub fn jaddi_offshoot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![landfall(crate::effect::shortcut::gain_life(1))],
        ..creature("Jaddi Offshoot", cost(&[g()]), vec![CreatureType::Plant], 0, 3)
    }
}

/// Tunneling Geopede — {2}{R} 3/2 Insect. Landfall: 1 damage to each opponent.
pub fn tunneling_geopede() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..creature("Tunneling Geopede", cost(&[generic(2), r()]), vec![CreatureType::Insect], 3, 2)
    }
}

/// Undergrowth Champion — {1}{G}{G} 2/2 Elemental. Damage is replaced by
/// removing a +1/+1 counter; landfall grows it.
pub fn undergrowth_champion() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Damage is prevented by removing a +1/+1 counter instead.",
            effect: StaticEffect::PreventDamageByRemovingCounters {
                kind: CounterType::PlusOnePlusOne,
                single: false,
            },
        }],
        triggered_abilities: vec![landfall(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..creature(
            "Undergrowth Champion",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Elemental],
            2,
            2,
        )
    }
}

/// Oran-Rief Hydra — {4}{G}{G} 5/5 trampler. Landfall: a +1/+1 counter, two if
/// the land is a Forest.
pub fn oran_rief_hydra() -> CardDefinition {
    use crate::card::{LandType, Predicate};
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![landfall(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasLandType(LandType::Forest),
            },
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            }),
            else_: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..creature(
            "Oran-Rief Hydra",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Hydra],
            5,
            5,
        )
    }
}

/// Akoum Hellkite — {4}{R}{R} 4/4 flier. Landfall: 1 damage to any target, 2 if
/// the land is a Mountain.
pub fn akoum_hellkite() -> CardDefinition {
    use crate::card::{LandType, Predicate};
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![landfall(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasLandType(LandType::Mountain),
            },
            then: Box::new(Effect::DealDamage { to: target_any(), amount: Value::Const(2) }),
            else_: Box::new(Effect::DealDamage { to: target_any(), amount: Value::Const(1) }),
        })],
        ..creature(
            "Akoum Hellkite",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Dragon],
            4,
            4,
        )
    }
}

// ── Rally allies ────────────────────────────────────────────────────────────

fn ally(
    name: &'static str,
    c: crate::mana::ManaCost,
    mut types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    types.push(CreatureType::Ally);
    creature(name, c, types, p, t)
}

/// Hero of Goma Fada — {4}{W} 4/3 Human Knight Ally. Rally: indestructible EOT.
pub fn hero_of_goma_fada() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally_grant(Keyword::Indestructible)],
        ..ally(
            "Hero of Goma Fada",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            4,
            3,
        )
    }
}

/// Lantern Scout — {2}{W} 3/2 Human Scout Ally. Rally: lifelink EOT.
pub fn lantern_scout() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally_grant(Keyword::Lifelink)],
        ..ally(
            "Lantern Scout",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Scout],
            3,
            2,
        )
    }
}

/// Makindi Patrol — {2}{W} 2/3 Human Knight Ally. Rally: vigilance EOT.
pub fn makindi_patrol() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally_grant(Keyword::Vigilance)],
        ..ally(
            "Makindi Patrol",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            3,
        )
    }
}

/// Ondu Champion — {2}{R}{R} 4/3 Minotaur Warrior Ally. Rally: trample EOT.
pub fn ondu_champion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally_grant(Keyword::Trample)],
        ..ally(
            "Ondu Champion",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Minotaur, CreatureType::Warrior],
            4,
            3,
        )
    }
}

/// Firemantle Mage — {2}{R} 2/2 Human Shaman Ally. Rally: menace EOT.
pub fn firemantle_mage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally_grant(Keyword::Menace)],
        ..ally(
            "Firemantle Mage",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            2,
            2,
        )
    }
}

/// Chasm Guide — {3}{R} 3/2 Goblin Scout Ally. Rally: haste EOT.
pub fn chasm_guide() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally_grant(Keyword::Haste)],
        ..ally(
            "Chasm Guide",
            cost(&[generic(3), r()]),
            vec![CreatureType::Goblin, CreatureType::Scout],
            3,
            2,
        )
    }
}

/// Resolute Blademaster — {3}{R}{W} 2/2 Human Soldier Ally. Rally: double
/// strike EOT.
pub fn resolute_blademaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally_grant(Keyword::DoubleStrike)],
        ..ally(
            "Resolute Blademaster",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Kalastria Healer — {1}{B} 1/2 Vampire Cleric Ally. Rally: drain 1.
pub fn kalastria_healer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(crate::effect::shortcut::drain(1))],
        ..ally(
            "Kalastria Healer",
            cost(&[generic(1), b()]),
            vec![CreatureType::Vampire, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Tajuru Beastmaster — {5}{G} 5/5 Elf Warrior Ally. Rally: your creatures get
/// +1/+1 until end of turn.
pub fn tajuru_beastmaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(Effect::PumpPT {
            what: each_your_creature(),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..ally(
            "Tajuru Beastmaster",
            cost(&[generic(5), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            5,
            5,
        )
    }
}

/// Kor Entanglers — {4}{W} 3/4 Kor Soldier Ally. Rally: tap a creature an
/// opponent controls.
pub fn kor_entanglers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![rally(Effect::Tap {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        })],
        ..ally(
            "Kor Entanglers",
            cost(&[generic(4), w()]),
            vec![CreatureType::Kor, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Grovetender Druids — {2}{G}{W} 3/3 Elf Druid Ally. Rally: you may pay {1}
/// for a 1/1 green Plant.
pub fn grovetender_druids() -> CardDefinition {
    use crate::card::TokenDefinition;
    let plant = TokenDefinition {
        name: "Plant".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![rally(Effect::MayPay {
            description: "Pay {1} to create a 1/1 green Plant?".into(),
            mana_cost: cost(&[generic(1)]),
            body: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: plant,
            }),
            else_: None,
        })],
        ..ally(
            "Grovetender Druids",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Angelic Captain — {3}{R}{W} 4/3 Angel Ally with flying. Attacks: +1/+1 for
/// each other attacking Ally.
pub fn angelic_captain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::count(Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Ally)
                    .and(R::IsAttacking)
                    .and(R::OtherThanSource),
            )),
            toughness: Value::count(Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Ally)
                    .and(R::IsAttacking)
                    .and(R::OtherThanSource),
            )),
            duration: Duration::EndOfTurn,
        })],
        ..ally("Angelic Captain", cost(&[generic(3), r(), w()]), vec![CreatureType::Angel], 4, 3)
    }
}

/// Reckless Cohort — {1}{R} 2/2 Human Warrior Ally. Attacks each combat if able
/// unless you control another Ally.
pub fn reckless_cohort() -> CardDefinition {
    use crate::card::{Predicate, StaticAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Attacks each combat if able unless you control another Ally.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::Not(Box::new(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Ally)
                            .and(R::ControlledByYou)
                            .and(R::OtherThanSource),
                    ),
                    n: Value::Const(1),
                })),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::MustAttack],
            },
        }],
        ..ally(
            "Reckless Cohort",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Veteran Warleader — {1}{G}{W} */* Human Soldier Ally, its P/T the number of
/// creatures you control. Tap another untapped Ally: gain first strike,
/// vigilance or trample (one ability per choice).
pub fn veteran_warleader() -> CardDefinition {
    use crate::card::DynamicPt;
    let grant = |keyword| ActivatedAbility {
        tap_other_filter: Some(R::HasCreatureType(CreatureType::Ally).and(R::OtherThanSource)),
        effect: Effect::GrantKeyword {
            what: Selector::This,
            keyword,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        dynamic_pt: Some(DynamicPt::CreaturesControlled { base: 0 }),
        activated_abilities: vec![
            grant(Keyword::FirstStrike),
            grant(Keyword::Vigilance),
            grant(Keyword::Trample),
        ],
        ..ally(
            "Veteran Warleader",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            0,
            0,
        )
    }
}

// ── Lifegain-matters ────────────────────────────────────────────────────────

fn on_life_gain(effect: Effect) -> crate::card::TriggeredAbility {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    TriggeredAbility {
        event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
        effect,
    }
}

/// Bloodbond Vampire — {2}{B}{B} 3/3 Vampire Shaman Ally. Whenever you gain
/// life, put a +1/+1 counter on it.
pub fn bloodbond_vampire() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_life_gain(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..ally(
            "Bloodbond Vampire",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Shaman],
            3,
            3,
        )
    }
}

/// Kalastria Nightwatch — {4}{B} 4/5 Vampire Warrior Ally. Whenever you gain
/// life, it gains flying until end of turn.
pub fn kalastria_nightwatch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_life_gain(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..ally(
            "Kalastria Nightwatch",
            cost(&[generic(4), b()]),
            vec![CreatureType::Vampire, CreatureType::Warrior],
            4,
            5,
        )
    }
}

/// Nirkana Assassin — {2}{B} 2/3 Vampire Assassin Ally. Whenever you gain life,
/// it gains deathtouch until end of turn.
pub fn nirkana_assassin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_life_gain(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Deathtouch,
            duration: Duration::EndOfTurn,
        })],
        ..ally(
            "Nirkana Assassin",
            cost(&[generic(2), b()]),
            vec![CreatureType::Vampire, CreatureType::Assassin],
            2,
            3,
        )
    }
}

/// Serene Steward — {1}{W} 2/2 Human Cleric Ally. Whenever you gain life, you
/// may pay {W} to put a +1/+1 counter on target creature.
pub fn serene_steward() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_life_gain(Effect::MayPay {
            description: "Pay {W} to put a +1/+1 counter on target creature?".into(),
            mana_cost: cost(&[w()]),
            body: Box::new(Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            else_: None,
        })],
        ..ally(
            "Serene Steward",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

// ── Eldrazi (Devoid / Ingest / Process) ─────────────────────────────────────

fn eldrazi(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Devoid], ..creature(name, c, types, p, t) }
}

/// Silent Skimmer — {3}{B} 0/4 Eldrazi Drone. Devoid, flying; attacks: the
/// defending player loses 2 life.
pub fn silent_skimmer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::Flying],
        triggered_abilities: vec![on_attack(crate::effect::shortcut::lose_life(
            2,
            Selector::Player(PlayerRef::EachOpponent),
        ))],
        ..eldrazi(
            "Silent Skimmer",
            cost(&[generic(3), b()]),
            vec![CreatureType::Eldrazi, CreatureType::Drone],
            0,
            4,
        )
    }
}

/// Tide Drifter — {1}{U} 0/5 Eldrazi Drone. Devoid; other colorless creatures
/// you control get +0/+1.
pub fn tide_drifter() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other colorless creatures you control get +0/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::Colorless).and(R::OtherThanSource),
                power: 0,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..eldrazi(
            "Tide Drifter",
            cost(&[generic(1), u()]),
            vec![CreatureType::Eldrazi, CreatureType::Drone],
            0,
            5,
        )
    }
}

/// Fathom Feeder — {U}{B} 1/1 Eldrazi Drone. Devoid, deathtouch, ingest;
/// {3}{U}{B}: draw a card and each opponent exiles their top card.
pub fn fathom_feeder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::Deathtouch],
        triggered_abilities: vec![ingest()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u(), b()]),
            effect: Effect::Seq(vec![
                crate::effect::shortcut::draw(1),
                Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                    link_to_source: false,
                    face_down: false,
                },
            ]),
            ..Default::default()
        }],
        ..eldrazi(
            "Fathom Feeder",
            cost(&[u(), b()]),
            vec![CreatureType::Eldrazi, CreatureType::Drone],
            1,
            1,
        )
    }
}

/// Forerunner of Slaughter — {B}{R} 3/2 Eldrazi Drone. Devoid; {1}: target
/// colorless creature gains haste until end of turn.
pub fn forerunner_of_slaughter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::Colorless)),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..eldrazi(
            "Forerunner of Slaughter",
            cost(&[b(), r()]),
            vec![CreatureType::Eldrazi, CreatureType::Drone],
            3,
            2,
        )
    }
}

/// Dust Stalker — {2}{B}{R} 5/3 Eldrazi. Devoid, haste; bounces itself at each
/// end step while you control no other colorless creature.
pub fn dust_stalker() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Predicate, TriggeredAbility};
    use crate::effect::ZoneDest;
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::AnyPlayer,
            )
            .with_filter(Predicate::Not(Box::new(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    R::Creature
                        .and(R::Colorless)
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                n: Value::Const(1),
            }))),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..eldrazi(
            "Dust Stalker",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Eldrazi],
            5,
            3,
        )
    }
}

/// Barrage Tyrant — {4}{R} 5/3 Eldrazi. Devoid; {2}{R}, Sacrifice another
/// colorless creature: it deals that creature's power in damage to any target.
pub fn barrage_tyrant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_other_filter: Some((
                R::Creature.and(R::Colorless).and(R::OtherThanSource),
                1,
            )),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::SacrificedPower,
            },
            ..Default::default()
        }],
        ..eldrazi("Barrage Tyrant", cost(&[generic(4), r()]), vec![CreatureType::Eldrazi], 5, 3)
    }
}

/// Deathless Behemoth — {6} 6/6 Eldrazi with vigilance. Sacrifice two Eldrazi
/// Scions to return it from your graveyard to your hand.
pub fn deathless_behemoth() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            sorcery_speed: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Scion), 2)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature("Deathless Behemoth", cost(&[generic(6)]), vec![CreatureType::Eldrazi], 6, 6)
    }
}

fn processor(
    name: &'static str,
    c: crate::mana::ManaCost,
    p: i32,
    t: i32,
) -> CardDefinition {
    eldrazi(name, c, vec![CreatureType::Eldrazi, CreatureType::Processor], p, t)
}

/// Cryptic Cruiser — {3}{U} 3/3 Eldrazi Processor. Devoid; {2}{U}, process one:
/// tap target creature.
pub fn cryptic_cruiser() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            process_cost: Some(1),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..processor("Cryptic Cruiser", cost(&[generic(3), u()]), 3, 3)
    }
}

/// Oracle of Dust — {4}{U} 3/5 Eldrazi Processor. Devoid; {2}, process one:
/// draw a card, then discard a card.
pub fn oracle_of_dust() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            process_cost: Some(1),
            effect: Effect::Seq(vec![
                crate::effect::shortcut::draw(1),
                crate::effect::shortcut::discard(Selector::You, 1, false),
            ]),
            ..Default::default()
        }],
        ..processor("Oracle of Dust", cost(&[generic(4), u()]), 3, 5)
    }
}

/// Void Attendant — {2}{G} 2/3 Eldrazi Processor. Devoid; {1}{G}, process one:
/// create an Eldrazi Scion.
pub fn void_attendant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            process_cost: Some(1),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: eldrazi_scion_token(),
            },
            ..Default::default()
        }],
        ..processor("Void Attendant", cost(&[generic(2), g()]), 2, 3)
    }
}

/// Ulamog's Reclaimer — {4}{U} 2/5 Eldrazi Processor. Devoid; ETB: process one
/// → return target instant or sorcery card from your graveyard to your hand.
pub fn ulamogs_reclaimer() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Process {
            count: 1,
            then: Box::new(Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::InYourGraveyard.and(
                        R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    ),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..processor("Ulamog's Reclaimer", cost(&[generic(4), u()]), 2, 5)
    }
}

/// Ulamog's Nullifier — {2}{U}{B} 2/3 Eldrazi Processor. Devoid, flash, flying;
/// ETB: process two → counter target spell.
pub fn ulamogs_nullifier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid, Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Process {
            count: 2,
            then: Box::new(crate::effect::shortcut::counter_target_spell()),
        })],
        ..processor("Ulamog's Nullifier", cost(&[generic(2), u(), b()]), 2, 3)
    }
}

/// Ulamog's Despoiler — {6} 5/5 Eldrazi Processor. As it enters you may process
/// two; if you do it enters with four +1/+1 counters.
pub fn ulamogs_despoiler() -> CardDefinition {
    CardDefinition {
        keywords: vec![],
        as_enters_effect: Some(Effect::Process {
            count: 2,
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(4),
            }),
        }),
        ..creature(
            "Ulamog's Despoiler",
            cost(&[generic(6)]),
            vec![CreatureType::Eldrazi, CreatureType::Processor],
            5,
            5,
        )
    }
}

// ── Misc ────────────────────────────────────────────────────────────────────

/// Beastcaller Savant — {1}{G} 1/1 Elf Shaman Ally with haste. {T}: one mana of
/// any color, spend only on a creature spell.
pub fn beastcaller_savant() -> CardDefinition {
    use crate::mana::SpendRestriction;
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::Restricted(
                    Box::new(crate::effect::ManaPayload::AnyOneColor(Value::Const(1))),
                    SpendRestriction::CreatureOnly,
                ),
            },
            ..Default::default()
        }],
        ..ally(
            "Beastcaller Savant",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Shaman],
            1,
            1,
        )
    }
}

/// Coralhelm Guide — {1}{U} 2/1 Merfolk Scout Ally. {4}{U}: target creature
/// can't be blocked this turn.
pub fn coralhelm_guide() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..ally(
            "Coralhelm Guide",
            cost(&[generic(1), u()]),
            vec![CreatureType::Merfolk, CreatureType::Scout],
            2,
            1,
        )
    }
}

/// Stone Haven Medic — {1}{W} 1/3 Kor Cleric. {W}, {T}: you gain 1 life.
pub fn stone_haven_medic() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: crate::effect::shortcut::gain_life(1),
            ..Default::default()
        }],
        ..creature(
            "Stone Haven Medic",
            cost(&[generic(1), w()]),
            vec![CreatureType::Kor, CreatureType::Cleric],
            1,
            3,
        )
    }
}

/// Valakut Invoker — {2}{R} 2/3 Human Shaman. {8}: 3 damage to any target.
pub fn valakut_invoker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            ..Default::default()
        }],
        ..creature(
            "Valakut Invoker",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            2,
            3,
        )
    }
}

/// Rot Shambler — {1}{G} 1/1 Fungus. Whenever another creature you control
/// dies, put a +1/+1 counter on it.
pub fn rot_shambler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_other_dies(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..creature("Rot Shambler", cost(&[generic(1), g()]), vec![CreatureType::Fungus], 1, 1)
    }
}

/// Windrider Patrol — {3}{U}{U} 4/3 Merfolk Wizard with flying. Combat damage
/// to a player: scry 2.
pub fn windrider_patrol() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        }],
        ..creature(
            "Windrider Patrol",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            4,
            3,
        )
    }
}

/// Angel of Renewal — {5}{W} 4/4 Angel Ally with flying. ETB: gain 1 life per
/// creature you control.
pub fn angel_of_renewal() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::count(Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou),
            )),
        })],
        ..ally("Angel of Renewal", cost(&[generic(5), w()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Drana's Emissary — {1}{W}{B} 2/2 Vampire Cleric Ally with flying. Upkeep:
/// drain 1.
pub fn dranas_emissary() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: crate::effect::shortcut::drain(1),
        }],
        ..ally(
            "Drana's Emissary",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Vampire, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Skyrider Elf — {X}{G}{U} 0/0 Elf Warrior Ally with flying. Converge: enters
/// with a +1/+1 counter per color of mana spent.
pub fn skyrider_elf() -> CardDefinition {
    CardDefinition {
        cost: cost(&[x(), g(), u()]),
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ConvergedValue)),
        ..ally(
            "Skyrider Elf",
            cost(&[x(), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            0,
            0,
        )
    }
}

/// Woodland Wanderer — {3}{G} 2/2 Elemental with vigilance and trample.
/// Converge: enters with a +1/+1 counter per color of mana spent.
pub fn woodland_wanderer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ConvergedValue)),
        ..creature(
            "Woodland Wanderer",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elemental],
            2,
            2,
        )
    }
}
